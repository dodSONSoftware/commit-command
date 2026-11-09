import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { Bridge } from "../services/bridge";
import type { AiSettingsSnapshot } from "../services/response.generated";
import { AiSettings } from "./AiSettings";
beforeEach(() => {
  Object.defineProperty(HTMLDialogElement.prototype, "showModal", {
    configurable: true,
    value: function (this: HTMLDialogElement) {
      this.setAttribute("open", "");
    },
  });
  Object.defineProperty(HTMLDialogElement.prototype, "close", {
    configurable: true,
    value: function (this: HTMLDialogElement) {
      this.removeAttribute("open");
      this.dispatchEvent(new Event("close"));
    },
  });
});
afterEach(() => {
  cleanup();
  Reflect.deleteProperty(HTMLDialogElement.prototype, "showModal");
  Reflect.deleteProperty(HTMLDialogElement.prototype, "close");
});
function snapshot(): AiSettingsSnapshot {
  return {
    fingerprint: "fixture-fingerprint",
    settings: {
      version: 1,
      revision: "revision",
      connections: [],
      profiles: [],
      assignments: [],
      instructions: [],
      global_concurrency: 1,
      connection_concurrency: 1,
      templates: [
        {
          id: "00000000-0000-4000-8000-000000000001",
          revision: "00000000-0000-4000-8000-000000000001",
          name: "Conventional commits",
          rules: {
            style: "conventional",
            subject_max_chars: 72,
            allowed_types: ["fix", "feat"],
            allow_scope: true,
            forbid_trailing_period: true,
            instructions: "Only observed changes",
          },
          examples: [],
        },
      ],
    },
  };
}
function fixture() {
  const request = vi
    .fn()
    .mockResolvedValue({ kind: "ai_settings", value: snapshot() });
  const bridge: Bridge = {
    epoch: 1,
    request,
    capabilities: async () => ({ writes_enabled: false }),
    writeStatus: async () => false,
    pick: async () => null,
    cancel: async () => {},
  };
  render(
    <>
      <textarea
        aria-label="Manual message"
        defaultValue="Manual draft preserved"
      />
      <AiSettings bridge={bridge} workspace={null} disabled={false} />
    </>,
  );
  return { request, user: userEvent.setup() };
}
it("loads no AI configuration until explicitly opened and preserves manual drafts", async () => {
  const { request, user } = fixture();
  expect(request).not.toHaveBeenCalled();
  await user.click(screen.getByRole("button", { name: "AI settings…" }));
  expect(
    await screen.findByRole("heading", { name: "Connections" }),
  ).toBeVisible();
  expect(request.mock.calls.map((c) => c[0].kind)).toEqual(["ai_settings"]);
  expect(screen.getByLabelText("Model identifier")).toHaveValue("");
  await user.click(screen.getByRole("button", { name: "Close AI settings" }));
  expect(screen.getByLabelText("Manual message")).toHaveValue(
    "Manual draft preserved",
  );
  expect(screen.getByRole("button", { name: "AI settings…" })).toHaveFocus();
});
it("saves only explicit nonsecret connection metadata with editor fingerprints", async () => {
  const { request, user } = fixture();
  await user.click(screen.getByRole("button", { name: "AI settings…" }));
  await user.type(
    await screen.findByLabelText("Connection name"),
    "Local fixture",
  );
  await user.click(
    screen.getByRole("button", { name: "Save connection metadata" }),
  );
  expect(request).toHaveBeenLastCalledWith({
    kind: "save_ai_settings",
    fingerprint: "fixture-fingerprint",
    revision: "revision",
    change: {
      kind: "connection",
      id: null,
      name: "Local fixture",
      provider: "ollama",
      endpoint: "http://127.0.0.1:11434/",
      timeout_ms: 30000,
    },
  });
  expect(request.mock.calls).toHaveLength(2);
});
it("static preview uses current unsaved rules and never saves or infers", async () => {
  const { request, user } = fixture();
  await user.click(screen.getByRole("button", { name: "AI settings…" }));
  await user.selectOptions(
    await screen.findByLabelText("Edit message template"),
    "00000000-0000-4000-8000-000000000001",
  );
  await user.clear(screen.getByLabelText("Subject maximum characters"));
  await user.type(screen.getByLabelText("Subject maximum characters"), "48");
  await user.type(
    screen.getByLabelText("Allowed conventional types (comma-separated)"),
    ",docs",
  );
  request.mockResolvedValueOnce({
    kind: "ai_template_preview",
    value: { subject: "fix: static sample", body: "No model ran" },
  });
  await user.click(
    screen.getByRole("button", { name: "Preview unsaved template without AI" }),
  );
  expect(request.mock.calls[1][0]).toMatchObject({
    kind: "ai_template_preview",
    template: {
      rules: { subject_max_chars: 48, allowed_types: ["fix", "feat", "docs"] },
    },
  });
  expect(await screen.findByText("fix: static sample")).toBeVisible();
  expect(request.mock.calls).toHaveLength(2);
});
it("failed settings load stays local and offers close without changing the manual draft", async () => {
  const { request, user } = fixture();
  request.mockRejectedValueOnce(new Error("AI settings cannot be read safely"));
  await user.click(screen.getByRole("button", { name: "AI settings…" }));
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "AI settings cannot be read safely",
  );
  await user.click(screen.getByRole("button", { name: "Close AI settings" }));
  expect(screen.getByLabelText("Manual message")).toHaveValue(
    "Manual draft preserved",
  );
});
