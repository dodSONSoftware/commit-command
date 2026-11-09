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
import { AiMessages } from "./AiMessages";
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
function fixture(fail = false) {
  const request = vi.fn(async (r: Request): Promise<Payload> => {
    if (r.kind === "preview_ai_context")
      return { kind: "ai_context", value: captured };
    if (r.kind === "review_ai_messages") {
      if (fail) throw new Error("AI unavailable; no fallback");
      return { kind: "done", value: null };
    }
    if (r.kind === "ai_jobs") return { kind: "ai_jobs", value: [job()] };
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
      />
    </>,
  );
  return { request, onUse, user: userEvent.setup() };
}
it("requires explicit capture and source review without contacting AI or changing the manual draft", async () => {
  const { request, onUse, user } = fixture();
  expect(request).not.toHaveBeenCalled();
  await user.click(
    screen.getByRole("button", { name: "AI message suggestions…" }),
  );
  expect(request).not.toHaveBeenCalled();
  expect(
    screen.getByRole("button", { name: "Review exact AI source transfer" }),
  ).toBeDisabled();
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
    screen.getByRole("button", { name: "Review exact AI source transfer" }),
  ).toBeDisabled();
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
    screen.getByRole("button", { name: "Review exact AI source transfer" }),
  );
  expect(
    await screen.findByRole("region", { name: "AI candidate 1" }),
  ).toHaveTextContent("Incomplete coverage");
  expect(onUse).not.toHaveBeenCalled();
  expect(
    screen.getByRole("button", { name: "Replace draft with candidate 2" }),
  ).toBeDisabled();
  expect(
    request.mock.calls.find(([r]) => r.kind === "review_ai_messages")?.[0],
  ).toMatchObject({
    snapshot_fingerprint: "snapshot",
    context_fingerprint: "context",
    one_off_profile: null,
  });
  await user.click(
    screen.getByRole("button", { name: "Replace draft with candidate 1" }),
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
    screen.getByRole("button", { name: "Review exact AI source transfer" }),
  );
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "AI unavailable; no fallback",
  );
  expect(onUse).not.toHaveBeenCalled();
  expect(screen.getByRole("textbox", { name: "Manual draft" })).toHaveValue(
    "Manual message preserved",
  );
});
