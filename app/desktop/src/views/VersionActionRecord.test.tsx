import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { Bridge } from "../services/bridge";
import type { VersionInvocation } from "../services/response.generated";
import { fixtureEntry } from "../testdata";
import { VersionActionRecord } from "./VersionActionRecord";
afterEach(cleanup);
function item(
  phase: VersionInvocation["phase"] = "applied",
): VersionInvocation {
  return {
    phase,
    applied: phase === "applied",
    observed: phase === "applied" ? "applied" : null,
    definition_approved_at_ms: 1,
    candidate: null,
    finished_at_ms: 2,
    issue: null,
    plan: {
      id: "exact-invocation",
      workspace: "ws",
      worktree: "wt",
      metadata: fixtureEntry("wt", "repo").metadata,
      path: { encoding: "utf8", value: "version" },
      version: {
        old: "1.4.2",
        new: "1.4.3",
        increment: "patch",
        prerelease: "reject",
        before: [49],
        after: [50],
        before_hash: "old",
        after_hash: "new",
      },
      file: {
        device: 1,
        inode: 2,
        mode: 33188,
        length: 6,
        modified_seconds: 1,
        modified_nanos: 0,
        changed_seconds: 1,
        changed_nanos: 0,
      },
      root_device: 1,
      root_inode: 3,
      parent_device: 1,
      parent_inode: 3,
      config_digest: "c",
      index_digest: "i",
      branch: "refs/heads/main",
      head: "a".repeat(40),
      created_at_ms: 1,
      digest: "fixed",
    },
  };
}
function setup(value = item(), enabled = true) {
  const request = vi
    .fn<Bridge["request"]>()
    .mockResolvedValue({ kind: "done" });
  const bridge: Bridge = {
    epoch: 1,
    request,
    capabilities: async () => ({ writes_enabled: enabled }),
    writeStatus: async () => false,
    pick: async () => null,
    cancel: async () => {},
  };
  render(
    <VersionActionRecord
      item={value}
      bridge={bridge}
      workspace="ws"
      enabled={enabled}
      busy={false}
      act={(f) => f()}
    />,
  );
  return request;
}
it("cannot stage from history in the ordinary read-only build", () => {
  const request = setup(item(), false);
  fireEvent.click(
    screen.getByRole("button", { name: "Review staging exact version result" }),
  );
  expect(request).not.toHaveBeenCalled();
});
it("requests only a separate review of the fixed successful invocation", async () => {
  const request = setup();
  expect(request).not.toHaveBeenCalled();
  fireEvent.click(
    screen.getByRole("button", { name: "Review staging exact version result" }),
  );
  await waitFor(() =>
    expect(request).toHaveBeenCalledExactlyOnceWith({
      kind: "review_version_stage",
      workspace: "ws",
      operation: "exact-invocation",
    }),
  );
});
it("unconfirmed results offer observation recovery and no retry or staging", async () => {
  const request = setup(item("unconfirmed"));
  expect(
    screen.queryByRole("button", { name: /staging|retry|apply/i }),
  ).toBeNull();
  fireEvent.click(
    screen.getByRole("button", {
      name: "Review observed version Action outcome",
    }),
  );
  await waitFor(() =>
    expect(request).toHaveBeenCalledExactlyOnceWith({
      kind: "review_version_recovery",
      workspace: "ws",
      operation: "exact-invocation",
    }),
  );
});
it("conflicting recovery never offers to apply, stage, or retry", () => {
  const value = item("reconciled");
  value.observed = "conflict";
  const request = setup(value);
  expect(screen.queryByRole("button")).toBeNull();
  expect(request).not.toHaveBeenCalled();
  expect(screen.getByText(/conflict/)).toBeTruthy();
});
