import { afterEach, beforeEach, expect, it, vi } from "vitest";
import {
  cleanup,
  render,
  screen,
  waitFor,
  fireEvent,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { Bridge, Payload } from "../services/bridge";
import type { Request } from "../services/request.generated";
import type { AiCapturedContext, AiJob } from "../services/response.generated";
import { AiMessages, type AiEditorDraft } from "./AiMessages";
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
const captured: AiCapturedContext = {
  snapshot_fingerprint: "snapshot",
  generation: "generation",
  bundle: {
    payload: '{"chunks":[{"id":"f1.1","text":"+selected change"}]}',
    fingerprint: "context",
    complete: false,
    warnings: ["Inspect omissions"],
    accounting: [
      {
        id: "f1",
        kind: "selected_diff",
        origin: "file.txt",
        input_bytes: 20,
        input_lines: 2,
        included_lines: 1,
        redacted_lines: 1,
        omitted_lines: 1,
        reason: "Byte budget",
      },
    ],
  },
};
function job(): AiJob {
  return {
    id: "job",
    phase: "complete",
    error: null,
    error_kind: null,
    review: {
      endpoint: "http://127.0.0.1:1/",
      route: "http://127.0.0.1:1/api/chat",
      model: "fixture-model",
      purpose: "assisted_messages",
      request_bytes: 100,
      fingerprint: "request",
      authentication: "none",
      credential: null,
      unencrypted: true,
      source: {
        profile: "profile",
        profile_revision: "revision",
        template: "template",
        template_revision: "revision",
        context_fingerprint: "context",
        snapshot_fingerprint: "snapshot",
        complete: false,
        warnings: [],
        accounting: [],
      },
    },
    result: {
      kind: "candidates",
      value: {
        model: "fixture-model",
        elapsed_ms: 1,
        input_tokens: null,
        output_tokens: null,
        context_fingerprint: "context",
        snapshot_fingerprint: "snapshot",
        context_complete: false,
        warnings: ["Model-reported coverage"],
        candidates: [
          {
            valid: true,
            validation_errors: [],
            unreported_chunks: ["f1.2"],
            candidate: {
              message: {
                subject: "fix: selected change",
                body: "Observed only",
              },
              rationale: "Explain the change",
              summary: "One selected change",
              concerns: ["Incomplete coverage"],
              covered_chunks: ["f1.1"],
            },
          },
          {
            valid: false,
            validation_errors: ["Invalid template"],
            unreported_chunks: [],
            candidate: {
              message: { subject: "invalid subject", body: "" },
              rationale: "Alternate wording",
              summary: "Change",
              concerns: [],
              covered_chunks: [],
            },
          },
        ],
      },
    },
  };
}
const editorDraft: AiEditorDraft = {
  connection: {
    id: null,
    name: "Unsaved connection",
    provider: "ollama",
    endpoint: "http://127.0.0.1:1/",
    timeout_ms: 3000,
    authentication: "none",
  },
  profile: {
    id: null,
    name: "Unsaved profile",
    connection: "editor-connection",
    model: "unsaved-model",
    system_message: "Unsaved system",
    parameters: {},
    template: "template",
    context_bytes: 16384,
  },
  template: {
    id: "00000000-0000-4000-8000-000000000001",
    revision: "00000000-0000-4000-8000-000000000001",
    name: "Unsaved template",
    rules: {
      style: "plain",
      subject_max_chars: 72,
      allowed_types: [],
      allow_scope: false,
      forbid_trailing_period: false,
      instructions: "Unsaved template instructions",
    },
    examples: [],
  },
  instructions: ["Unsaved scoped instructions"],
};
function fixture(fail = false, editor = false, stale = false) {
  const request = vi.fn(async (r: Request): Promise<Payload> => {
    if (r.kind === "preview_ai_context")
      return { kind: "ai_context", value: captured };
    if (r.kind === "review_ai_messages" || r.kind === "review_ai_editor_test") {
      if (fail) throw new Error("AI unavailable; no fallback");
      return {
        kind: "executed",
        value: {
          result: { ai_job: "job" },
          ai_job_id: "job",
          ai_request_fingerprint: "request",
        },
      };
    }
    if (r.kind === "ai_jobs") {
      const result = job();
      if (editor) result.review.purpose = "editor_test";
      const old = structuredClone(result);
      old.id = "stale";
      old.review.fingerprint = "previous-request";
      old.review.model = "previous-profile-model";
      return { kind: "ai_jobs", value: stale ? [old, result] : [result] };
    }
    throw new Error(`Unexpected request ${r.kind}`);
  });
  const bridge: Bridge = {
    epoch: 1,
    request,
    capabilities: async () => ({ writes_enabled: false }),
    writeStatus: async () => false,
    pick: async () => null,
    cancel: async () => {},
  };
  const onUse = vi.fn();
  const onUseBody = vi.fn();
  render(
    <>
      <textarea
        aria-label="Manual draft"
        defaultValue="Manual message preserved"
      />
      <AiMessages
        bridge={bridge}
        workspace="ws"
        worktree="wt"
        selected={[
          { path: { encoding: "utf8", value: "file.txt" }, source: "index" },
        ]}
        busy={false}
        onUse={onUse}
        onUseBody={onUseBody}
        editor={editor ? editorDraft : undefined}
      />
    </>,
  );
  return { request, onUse, onUseBody, user: userEvent.setup() };
}
it("allows optional context inspection without contacting AI or changing the manual draft", async () => {
  const { request, onUse, user } = fixture();
  expect(request).not.toHaveBeenCalled();
  await user.click(
    screen.getByRole("button", { name: "AI message suggestions…" }),
  );
  expect(request).not.toHaveBeenCalled();
  expect(
    screen.getByRole("button", { name: "Generate message suggestions" }),
  ).toBeEnabled();
  await user.click(
    screen.getByRole("button", {
      name: "Capture filtered AI context · no transfer",
    }),
  );
  expect(
    await screen.findByRole("region", { name: "Exact filtered AI context" }),
  ).toHaveTextContent(captured.bundle.payload);
  expect(request).toHaveBeenCalledOnce();
  expect(request.mock.calls[0][0].kind).toBe("preview_ai_context");
  expect(onUse).not.toHaveBeenCalled();
  expect(screen.getByRole("textbox", { name: "Manual draft" })).toHaveValue(
    "Manual message preserved",
  );
  fireEvent.change(screen.getByLabelText("AI context byte budget"), {
    target: { value: 2048 },
  });
  expect(
    screen.queryByRole("region", { name: "Exact filtered AI context" }),
  ).toBeNull();
  expect(
    screen.getByRole("button", { name: "Generate message suggestions" }),
  ).toBeEnabled();
});
it("uses only a deliberately selected valid suggestion to replace the draft", async () => {
  const { request, onUse, user } = fixture();
  await user.click(
    screen.getByRole("button", { name: "AI message suggestions…" }),
  );
  await user.click(
    screen.getByRole("button", {
      name: "Capture filtered AI context · no transfer",
    }),
  );
  await user.click(
    screen.getByRole("button", { name: "Generate message suggestions" }),
  );
  expect(
    await screen.findByRole("region", { name: "AI candidate 1" }),
  ).toHaveTextContent("Incomplete coverage");
  expect(onUse).not.toHaveBeenCalled();
  expect(
    screen.getByRole("button", {
      name: "Use title and description from candidate 2",
    }),
  ).toBeDisabled();
  expect(
    request.mock.calls.find(([r]) => r.kind === "review_ai_messages")?.[0],
  ).toMatchObject({
    snapshot_fingerprint: "snapshot",
    context_fingerprint: "context",
    one_off_profile: null,
  });
  await user.click(
    screen.getByRole("button", {
      name: "Use title and description from candidate 1",
    }),
  );
  expect(onUse).toHaveBeenCalledWith("fix: selected change\n\nObserved only\n");
  await waitFor(() =>
    expect(
      screen.getByRole("button", { name: "AI message suggestions…" }),
    ).toHaveFocus(),
  );
  expect(
    request.mock.calls.some(([r]) =>
      ["propose", "review_version_action", "review_trust"].includes(r.kind),
    ),
  ).toBe(false);
});
it("keeps the manual draft and shows failures without applying a fallback", async () => {
  const { onUse, user } = fixture(true);
  await user.click(
    screen.getByRole("button", { name: "AI message suggestions…" }),
  );
  await user.click(
    screen.getByRole("button", {
      name: "Capture filtered AI context · no transfer",
    }),
  );
  await user.click(
    screen.getByRole("button", { name: "Generate message suggestions" }),
  );
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "AI unavailable; no fallback",
  );
  expect(onUse).not.toHaveBeenCalled();
  expect(screen.getByRole("textbox", { name: "Manual draft" })).toHaveValue(
    "Manual message preserved",
  );
});

it("reviews current unsaved editors without saving settings or offering draft replacement", async () => {
  const { request, onUse, user } = fixture(false, true);
  await user.click(screen.getByRole("button", { name: "Test with AI…" }));
  expect(
    screen.queryByRole("button", {
      name: "Load optional AI profiles · no transfer",
    }),
  ).not.toBeInTheDocument();
  await user.click(
    screen.getByRole("button", {
      name: "Capture filtered AI context · no transfer",
    }),
  );
  fireEvent.change(
    screen.getByRole("textbox", { name: "One-off message instructions" }),
    { target: { value: "Extra unsaved instructions" } },
  );
  // Instruction edits deliberately invalidate the old preview.
  expect(
    screen.getByRole("button", { name: "Generate message suggestions" }),
  ).toBeEnabled();
  await user.click(
    screen.getByRole("button", {
      name: "Capture filtered AI context · no transfer",
    }),
  );
  await user.click(
    screen.getByRole("button", { name: "Generate message suggestions" }),
  );
  expect(request).toHaveBeenCalledWith(
    expect.objectContaining({
      kind: "review_ai_editor_test",
      workspace: "ws",
      worktree: "wt",
      connection: editorDraft.connection,
      profile: editorDraft.profile,
      template: editorDraft.template,
      instructions: [
        "Unsaved scoped instructions",
        "Extra unsaved instructions",
      ],
      snapshot_fingerprint: "snapshot",
      context_fingerprint: "context",
    }),
  );
  await screen.findByText("Editor test · complete");
  expect(
    screen.queryByRole("button", {
      name: /Use title and description from candidate/,
    }),
  ).not.toBeInTheDocument();
  expect(
    request.mock.calls.some(
      ([r]) => r.kind === "save_ai_settings" || r.kind === "ai_settings",
    ),
  ).toBe(false);
  expect(onUse).not.toHaveBeenCalled();
  expect(screen.getByRole("textbox", { name: "Manual draft" })).toHaveValue(
    "Manual message preserved",
  );
});

for (const editor of [false, true]) {
  it(`excludes a previous profile result with identical source in ${editor ? "editor tests" : "assisted generation"}`, async () => {
    const { user } = fixture(false, editor, true);
    await user.click(
      screen.getByRole("button", {
        name: editor ? "Test with AI…" : "AI message suggestions…",
      }),
    );
    await user.click(
      screen.getByRole("button", {
        name: "Capture filtered AI context · no transfer",
      }),
    );
    await user.click(
      screen.getByRole("button", { name: "Generate message suggestions" }),
    );
    await screen.findByRole("article", { name: "Message generation job" });
    expect(
      screen.queryByRole("article", { name: "Message generation stale" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByText(/previous-profile-model/),
    ).not.toBeInTheDocument();
  });
}

it("can use only the generated description without replacing the title", async () => {
  const { user, onUse, onUseBody } = fixture();
  await user.click(
    screen.getByRole("button", { name: "AI message suggestions…" }),
  );
  await user.click(
    screen.getByRole("button", {
      name: "Capture filtered AI context · no transfer",
    }),
  );
  await user.click(
    screen.getByRole("button", { name: "Generate message suggestions" }),
  );
  await user.click(
    await screen.findByRole("button", {
      name: "Use description only from candidate 1",
    }),
  );
  expect(onUseBody).toHaveBeenCalledWith("Observed only");
  expect(onUse).not.toHaveBeenCalled();
});
