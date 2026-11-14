import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { AiImportInspector, archiveTimestamp } from "./AiImportInspector";
import type { Bridge, Payload } from "../services/bridge";
import type {
  AiImportSource,
  AiImportTarget,
} from "../services/response.generated";
afterEach(cleanup);
function source(token = "native-source"): AiImportSource {
  return {
    source: token,
    path: { encoding: "utf8", value: `/fixture/${token}.zip` },
    archive_fingerprint: "a".repeat(64),
    manifest_fingerprint: "b".repeat(64),
    available_scopes: [{ kind: "application" }],
    manifest: {
      version: 1,
      kind: "ai_configuration",
      application_version: "fixture\u202e",
      created_at_ms: 1709164800000,
      settings_revision: "00000000-0000-4000-8000-000000000003",
      database_schema: null,
      includes_history: false,
      recovery_policy: "not_included",
      components: [
        {
          path: "ai/settings.json",
          component_type: "regular_file",
          bytes: 2,
          sha256: "c".repeat(64),
        },
      ],
    },
    settings: {
      version: 1,
      revision: "00000000-0000-4000-8000-000000000002",
      connections: [],
      profiles: [],
      templates: [],
      assignments: [],
      instructions: [
        { scope: { kind: "application" }, text: "<img src=x>\u202e" },
      ],
      global_concurrency: 1,
      connection_concurrency: 1,
    },
  };
}
function fixture() {
  const request = vi.fn<Bridge["request"]>();
  const pickAiImport = vi
    .fn<NonNullable<Bridge["pickAiImport"]>>()
    .mockResolvedValue(null);
  const bridge: Bridge = {
    epoch: 1,
    request,
    pickAiImport,
    capabilities: async () => ({ writes_enabled: false }),
    writeStatus: async () => false,
    pick: async () => null,
    cancel: async () => {},
  };
  return { bridge, request, pickAiImport };
}
it("does no implicit reads or mutations and treats picker cancellation as no selection", async () => {
  const { bridge, request, pickAiImport } = fixture();
  render(<AiImportInspector bridge={bridge} disabled={false} />);
  expect(request).not.toHaveBeenCalled();
  const choose = screen.getByRole("button", {
    name: "Choose AI backup to inspect…",
  });
  fireEvent.click(choose);
  await waitFor(() => expect(choose).toBeEnabled());
  expect(pickAiImport).toHaveBeenCalledTimes(1);
  expect(request).not.toHaveBeenCalled();
  expect(screen.queryByRole("alert")).not.toBeInTheDocument();
});
it("shows verified manifest and escaped metadata, with bounded explicit target navigation", async () => {
  const { bridge, request, pickAiImport } = fixture();
  const imported = source();
  const targets: AiImportTarget[] = Array.from({ length: 100 }, (_, n) => ({
    scope: {
      kind: "repository",
      workspace: "00000000-0000-4000-8000-000000000001",
      worktree: `00000000-0000-4000-8000-${(n + 1).toString(16).padStart(12, "0")}`,
    },
    name: "Repository",
    path: { encoding: "utf8", value: `/fixture/repo-${n}` },
  }));
  pickAiImport.mockResolvedValue({
    source: imported.source,
    display: imported.path,
  });
  request.mockImplementation(async (query) => {
    if (query.kind === "inspect_ai_import")
      return { kind: "ai_import_source", value: imported };
    if (query.kind === "ai_import_targets")
      return {
        kind: "ai_import_targets",
        value: {
          items: query.offset === 0 ? targets : [targets[0]],
          total: 101,
          next: query.offset === 0 ? 100 : null,
        },
      };
    throw new Error("Unexpected mutating request");
  });
  const { container } = render(
    <AiImportInspector bridge={bridge} disabled={false} />,
  );
  fireEvent.click(
    screen.getByRole("button", { name: "Choose AI backup to inspect…" }),
  );
  await screen.findByRole("heading", { name: "/fixture/native-source.zip" });
  expect(screen.getByText("2024-02-29T00:00:00.000Z")).toBeInTheDocument();
  expect(screen.getByText("fixture\\u202e")).toBeInTheDocument();
  expect(
    screen.getByRole("table", { name: "Verified archive components" }),
  ).toHaveTextContent("ai/settings.json");
  fireEvent.click(screen.getByText("Sanitized inactive AI metadata"));
  expect(container.querySelector("pre")).toHaveTextContent(
    "<img src=x>\\u202e",
  );
  expect(container.querySelector("img")).toBeNull();
  expect(request.mock.calls.map((call) => call[0].kind)).toEqual([
    "inspect_ai_import",
  ]);
  const search = screen.getByRole("textbox", {
    name: "Find current AI import mapping targets",
  });
  fireEvent.change(search, { target: { value: "fixture" } });
  fireEvent.click(
    screen.getByRole("button", { name: "Search mapping targets" }),
  );
  await screen.findByRole("button", { name: "Next mapping target page" });
  expect(
    screen.getByRole("list", { name: "Current AI import mapping targets" })
      .children,
  ).toHaveLength(100);
  fireEvent.change(search, { target: { value: "unsaved query" } });
  fireEvent.click(
    screen.getByRole("button", { name: "Next mapping target page" }),
  );
  await waitFor(() =>
    expect(request).toHaveBeenLastCalledWith({
      kind: "ai_import_targets",
      source: "native-source",
      filter: "fixture",
      offset: 100,
    }),
  );
  await waitFor(() =>
    expect(
      screen.queryByRole("button", { name: "Next mapping target page" }),
    ).not.toBeInTheDocument(),
  );
  expect(
    request.mock.calls.every((call) =>
      ["inspect_ai_import", "ai_import_targets"].includes(call[0].kind),
    ),
  ).toBe(true);
});
it("rejects mismatched source replies and discards delayed replies after context replacement", async () => {
  const { bridge, request, pickAiImport } = fixture();
  const imported = source();
  pickAiImport.mockResolvedValue({
    source: imported.source,
    display: imported.path,
  });
  request.mockResolvedValueOnce({
    kind: "ai_import_source",
    value: source("wrong-source"),
  });
  const view = render(
    <AiImportInspector key="first" bridge={bridge} disabled={false} />,
  );
  fireEvent.click(
    screen.getByRole("button", { name: "Choose AI backup to inspect…" }),
  );
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Archive selection changed",
  );
  let resolve!: (value: Payload) => void;
  request.mockReturnValueOnce(
    new Promise((done) => {
      resolve = done;
    }),
  );
  fireEvent.click(
    screen.getByRole("button", { name: "Choose AI backup to inspect…" }),
  );
  await waitFor(() => expect(request).toHaveBeenCalledTimes(2));
  view.rerender(
    <AiImportInspector key="second" bridge={bridge} disabled={false} />,
  );
  await act(async () => {
    resolve({ kind: "ai_import_source", value: imported });
  });
  expect(
    screen.queryByRole("heading", { name: "/fixture/native-source.zip" }),
  ).not.toBeInTheDocument();
  expect(screen.queryByRole("alert")).not.toBeInTheDocument();
});
it("bounds hostile declared timestamps and disables selection without native support", () => {
  expect(archiveTimestamp(Number.MAX_SAFE_INTEGER)).toBe(
    "Unsupported source timestamp",
  );
  expect(archiveTimestamp(Infinity)).toBe("Unsupported source timestamp");
  expect(archiveTimestamp(-1)).toBe("Unsupported source timestamp");
  const { bridge } = fixture();
  delete bridge.pickAiImport;
  render(<AiImportInspector bridge={bridge} disabled={false} />);
  expect(
    screen.getByRole("button", { name: "Choose AI backup to inspect…" }),
  ).toBeDisabled();
});
