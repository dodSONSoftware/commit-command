import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { UpstreamOperation } from "../services/response.generated";
import { fixtureEntry } from "../testdata";
import { UpstreamRecord } from "./UpstreamRecord";
afterEach(cleanup);
const item: UpstreamOperation = {
  plan: {
    id: "upstream",
    workspace: "ws",
    worktree: "repo",
    metadata: fixtureEntry("repo", "repo").metadata,
    remote: "origin",
    reference: "refs/heads/target",
    destination: "https://example.invalid/<img>",
    transport: "https",
    config_digest: "config",
    generation: "generation",
    created_at_ms: 0,
    tools: [],
    local_target: null,
    local_binding: null,
    source_ref: "refs/heads/main",
    source_oid: { format: "sha1", hex: "a".repeat(40) },
    observation_id: "observation",
    observed_oid: { format: "sha1", hex: "b".repeat(40) },
    observed_at_ms: 0,
    expected_config: {
      identity: "identity",
      digest: "before",
      mode: 420,
      group: 20,
      attributes_digest: "attrs",
    },
    candidate_digest: "after",
  },
  phase: "uncertain",
  configuration: "updated",
  issue: { kind: "storage", message: "Failure <script>" },
};
it("shows real configuration effects and requests only fresh guarded inspection", () => {
  const recover = vi.fn();
  const { container, rerender } = render(
    <UpstreamRecord item={item} onRecover={recover} />,
  );
  expect(screen.getByText(/Configuration result:/)).toHaveTextContent(
    "updated",
  );
  expect(screen.getByText(/fresh trust review/)).toBeInTheDocument();
  expect(container.querySelector("img,script")).toBeNull();
  fireEvent.click(
    screen.getByRole("button", { name: "Inspect upstream recovery" }),
  );
  expect(recover).toHaveBeenCalledExactlyOnceWith("upstream");
  rerender(
    <UpstreamRecord
      item={{
        ...item,
        phase: "observed_configuration",
        configuration: "matches_target",
        issue: null,
      }}
      onRecover={recover}
    />,
  );
  expect(screen.queryByRole("button")).toBeNull();
  expect(
    screen.getByText(/does not attribute uncertain publication/),
  ).toBeInTheDocument();
});
