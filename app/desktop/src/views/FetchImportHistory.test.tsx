import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { FetchImport } from "../services/response.generated";
import { fixtureEntry } from "../testdata";
import { FetchImportRecords } from "./FetchImportHistory";
afterEach(cleanup);
const item: FetchImport = {
  plan: {
    id: "import",
    workspace: "ws",
    worktree: "repo",
    metadata: fixtureEntry("repo", "repo").metadata,
    remote: "origin",
    reference: "refs/heads/main",
    destination: "https://example.invalid/repo.git",
    transport: "https",
    config_digest: "config",
    generation: "generation",
    created_at_ms: 0,
    tools: [],
    local_target: null,
    local_binding: null,
    capture_id: "capture",
    expected_oid: { hex: "a".repeat(40), format: "sha1" },
    expected_old: null,
    bundle_digest: "checksum",
    bundle_bytes: 100,
    tracking_ref: "refs/remotes/origin/<img>",
  },
  phase: "uncertain",
  objects_imported: true,
  reference_status: "not_attempted",
  issue: { kind: "uncertain_outcome", message: "Unexpected <script>" },
};
it("shows exact publication uncertainty and requests only guarded inspection", () => {
  const recover = vi.fn();
  const { container, rerender } = render(
    <FetchImportRecords items={[item]} onRecover={recover} />,
  );
  expect(screen.getByText(/Ref result:/)).toHaveTextContent("not attempted");
  expect(container.querySelector("script,img")).toBeNull();
  fireEvent.click(
    screen.getByRole("button", { name: "Inspect captured branch recovery" }),
  );
  expect(recover).toHaveBeenCalledExactlyOnceWith("import");
  rerender(
    <FetchImportRecords
      items={[
        {
          ...item,
          phase: "observed_target",
          reference_status: "matches_target",
          issue: null,
        },
      ]}
      onRecover={recover}
    />,
  );
  expect(screen.queryByRole("button")).toBeNull();
  expect(
    screen.getByText(/does not attribute an uncertain publication/),
  ).toBeInTheDocument();
});
