import { afterEach, expect, it, vi } from "vitest";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import type { Payload } from "../services/bridge";
import type { Bridge } from "../services/bridge";
import { AiCommitFill } from "./AiCommitFill";
afterEach(cleanup);
function fixture(
  configured: boolean,
  selected = true,
  beginUpdate?: () => (message: string) => boolean,
) {
  const onUse = vi.fn();
  const request = vi.fn<Bridge["request"]>();
  const ready = () =>
    request.mockResolvedValue({
      kind: "effective_ai_settings",
      value: {
        connection: {
          id: "c",
          revision: "r",
          name: "Local",
          provider: "ollama",
          endpoint: "http://127.0.0.1:1/",
          endpoint_approval: "",
          timeout_ms: 3000,
        },
        profile: {
          id: "p",
          revision: "r",
          name: "Profile",
          connection: "c",
          model: "model",
          system_message: "",
          parameters: {},
          template: "t",
          context_bytes: 16384,
        },
        template: {
          id: "t",
          revision: "r",
          name: "Plain",
          examples: [],
          rules: {
            style: "plain",
            subject_max_chars: 72,
            allowed_types: [],
            allow_scope: false,
            forbid_trailing_period: false,
            instructions: "",
          },
        },
        instructions: [],
      },
    });
  if (configured) ready();
  else request.mockRejectedValue(new Error("No AI assignment"));
  const bridge: Bridge = {
    epoch: 1,
    request,
    capabilities: async () => ({ writes_enabled: false }),
    writeStatus: async () => false,
    pick: async () => null,
    cancel: async () => {},
  };
  const view = render(
    <AiCommitFill
      onUse={onUse}
      beginUpdate={beginUpdate}
      bridge={bridge}
      workspace="ws"
      worktree="wt"
      selected={
        selected
          ? [{ path: { encoding: "utf8", value: "file.txt" }, source: "index" }]
          : []
      }
      busy={false}
    />,
  );
  return {
    request,
    ready,
    onUse,
    switchWorkspace: () =>
      view.rerender(
        <AiCommitFill
          onUse={onUse}
          beginUpdate={beginUpdate}
          bridge={bridge}
          key="new-workspace/new-worktree"
          workspace="new-workspace"
          worktree="new-worktree"
          selected={[
            { path: { encoding: "utf8", value: "file.txt" }, source: "index" },
          ]}
          busy={false}
        />,
      ),
  };
}
it("greys out filling when AI configuration is missing and rechecks when returning to the app", async () => {
  const { request, ready } = fixture(false);
  const button = screen.getByRole("button", { name: "Fill with AI" });
  expect(button).toBeDisabled();
  await screen.findByText(/Connect your server/);
  expect(button).toHaveAttribute("title", expect.stringContaining("Settings"));
  ready();
  fireEvent(window, new Event("focus"));
  await waitFor(() => expect(button).toBeEnabled());
  expect(
    request.mock.calls.every(([r]) => r.kind === "effective_ai_settings"),
  ).toBe(true);
});
it("requires selected changes even when local AI configuration is available", async () => {
  fixture(true, false);
  await screen.findByText("Select changed files to generate a commit message.");
  expect(screen.getByRole("button", { name: "Fill with AI" })).toBeDisabled();
});

async function generation(
  failed = false,
  beginUpdate?: () => (message: string) => boolean,
) {
  const f = fixture(true, true, beginUpdate);
  const settings = await f.request.mock.results[0].value;
  let loading = false;
  f.request.mockImplementation(async (request) => {
    if (request.kind === "effective_ai_settings") return settings;
    if (request.kind === "review_ai_model_load") {
      loading = true;
      return {
        kind: "executed",
        value: {
          result: null,
          ai_job_id: "loading",
          ai_request_fingerprint: "load-request",
        },
      };
    }
    if (request.kind === "ai_jobs" && loading) {
      loading = false;
      return {
        kind: "ai_jobs",
        value: [
          {
            id: "loading",
            phase: "complete",
            error: null,
            review: {
              fingerprint: "load-request",
              endpoint: "http://localhost/",
              route: "http://localhost/api/chat",
              model: "model",
              purpose: "load_model",
              request_bytes: 1,
              authentication: "none",
              unencrypted: true,
            },
            result: { kind: "model_loaded", value: { model: "model" } },
          },
        ],
      };
    }
    if (request.kind === "preview_ai_context")
      return {
        kind: "ai_context",
        value: {
          snapshot_fingerprint: "snapshot",
          generation: "g",
          bundle: {
            payload: "selected diff",
            fingerprint: "context",
            complete: true,
            warnings: [],
            accounting: [],
          },
        },
      };
    if (request.kind === "review_ai_messages")
      return {
        kind: "executed",
        value: {
          result: null,
          ai_job_id: "job",
          ai_request_fingerprint: "request",
        },
      };
    if (request.kind === "ai_jobs")
      return {
        kind: "ai_jobs",
        value: [
          {
            id: "job",
            phase: failed ? "failed" : "complete",
            error: failed ? "Server is unavailable" : null,
            review: {
              fingerprint: "request",
              endpoint: "http://localhost/",
              route: "http://localhost/api/chat",
              model: "model",
              purpose: "assisted_messages",
              request_bytes: 1,
              authentication: "none",
              unencrypted: true,
            },
            result: failed
              ? null
              : {
                  kind: "candidates",
                  value: {
                    model: "model",
                    elapsed_ms: 1,
                    context_fingerprint: "context",
                    snapshot_fingerprint: "snapshot",
                    context_complete: true,
                    warnings: [],
                    candidates: [
                      {
                        valid: true,
                        validation_errors: [],
                        unreported_chunks: [],
                        candidate: {
                          message: {
                            subject: "fix: update navigation",
                            body: "Describe selected changes.",
                          },
                          rationale: "",
                          summary: "",
                          concerns: [],
                          covered_chunks: [],
                        },
                      },
                    ],
                  },
                },
          },
        ],
      };
    return { kind: "done" };
  });
  return f;
}
it("fills the message from one click without a dialog or extra confirmation", async () => {
  const f = await generation();
  const button = screen.getByRole("button", { name: "Fill with AI" });
  await waitFor(() => expect(button).toBeEnabled());
  fireEvent.click(button);
  await waitFor(() =>
    expect(f.onUse).toHaveBeenCalledWith(
      "fix: update navigation\n\nDescribe selected changes.\n",
    ),
  );
  expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  expect(
    f.request.mock.calls.filter(([r]) => r.kind === "review_ai_messages"),
  ).toHaveLength(1);
  const capture = f.request.mock.calls.find(
    ([r]) => r.kind === "preview_ai_context",
  )?.[0];
  expect(capture).toMatchObject({
    spec: {
      branch_commits: 0,
      file_commits: 0,
      nearby_source: [],
      documentation: [],
    },
  });
});
it("shows failures beside the button and preserves the draft", async () => {
  const f = await generation(true);
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Fill with AI" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Fill with AI" }));
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Server is unavailable",
  );
  expect(f.onUse).not.toHaveBeenCalled();
  expect(screen.getByRole("button", { name: "Fill with AI" })).toBeEnabled();
});

it("shows the actual AI step while waiting and clears feedback on completion", async () => {
  const f = await generation();
  const original = f.request.getMockImplementation()!;
  let resumeCapture!: () => void;
  let resumeModel!: () => void;
  const captureGate = new Promise<void>((resolve) => {
    resumeCapture = resolve;
  });
  const modelGate = new Promise<void>((resolve) => {
    resumeModel = resolve;
  });
  f.request.mockImplementation(async (request) => {
    if (request.kind === "preview_ai_context") await captureGate;
    if (
      request.kind === "ai_jobs" &&
      f.request.mock.calls.some(([r]) => r.kind === "review_ai_messages")
    )
      await modelGate;
    return original(request);
  });
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Fill with AI" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Fill with AI" }));
  expect(await screen.findByText("Reading selected changes…")).toBeVisible();
  expect(
    screen.getByRole("button", { name: "Generating message…" }),
  ).toBeDisabled();
  await act(async () => resumeCapture());
  expect(await screen.findByText("Waiting for the model…")).toBeVisible();
  await act(async () => resumeModel());
  await waitFor(() => expect(f.onUse).toHaveBeenCalledOnce());
  expect(screen.queryByRole("status")).toBeNull();
  expect(screen.getByRole("button", { name: "Fill with AI" })).toBeEnabled();
});

it.each(["capture", "model"])(
  "cancels AI during %s and never applies a late response",
  async (phase) => {
    const f = await generation();
    const original = f.request.getMockImplementation()!;
    let resume!: () => void;
    const gate = new Promise<void>((resolve) => {
      resume = resolve;
    });
    f.request.mockImplementation(async (request) => {
      if (
        (request.kind === "preview_ai_context" && phase === "capture") ||
        (request.kind === "ai_jobs" &&
          phase === "model" &&
          f.request.mock.calls.some(([r]) => r.kind === "review_ai_messages"))
      )
        await gate;
      return original(request);
    });
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Fill with AI" }),
      ).toBeEnabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Fill with AI" }));
    await screen.findByText(
      phase === "capture"
        ? "Reading selected changes…"
        : "Waiting for the model…",
    );
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(screen.getByRole("button", { name: "Cancelling…" })).toBeDisabled();
    if (phase === "model")
      await waitFor(() =>
        expect(f.request).toHaveBeenCalledWith({
          kind: "cancel_ai_job",
          id: "job",
        }),
      );
    await act(async () => resume());
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Fill with AI" }),
      ).toBeEnabled(),
    );
    expect(f.onUse).not.toHaveBeenCalled();
    expect(screen.queryByRole("status")).toBeNull();
    if (phase === "capture")
      expect(
        f.request.mock.calls.some(([r]) => r.kind === "review_ai_messages"),
      ).toBe(false);
  },
);

it("waits for confirmed loading before reading or sending selected changes", async () => {
  const f = await generation();
  const original = f.request.getMockImplementation()!;
  let resume!: () => void;
  const gate = new Promise<void>((resolve) => {
    resume = resolve;
  });
  f.request.mockImplementation(async (request) => {
    if (
      request.kind === "ai_jobs" &&
      !f.request.mock.calls.some(([r]) => r.kind === "preview_ai_context")
    )
      await gate;
    return original(request);
  });
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Fill with AI" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Fill with AI" }));
  expect(await screen.findByText("Loading model…")).toBeVisible();
  expect(
    f.request.mock.calls.some(
      ([r]) =>
        r.kind === "preview_ai_context" || r.kind === "review_ai_messages",
    ),
  ).toBe(false);
  await act(async () => resume());
  await waitFor(() => expect(f.onUse).toHaveBeenCalledOnce());
});

it("reports a model loading failure without sending repository changes", async () => {
  const f = await generation();
  const original = f.request.getMockImplementation()!;
  f.request.mockImplementation(async (request) => {
    const reply = await original(request);
    if (reply.kind === "ai_jobs") {
      const job = reply.value[0];
      job.phase = "failed";
      job.result = null;
      job.error =
        "Model loading timed out after 120 seconds. The server has not confirmed that the model loaded.";
    }
    return reply;
  });
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Fill with AI" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Fill with AI" }));
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Model loading timed out after 120 seconds",
  );
  expect(f.onUse).not.toHaveBeenCalled();
  expect(
    f.request.mock.calls.some(
      ([r]) =>
        r.kind === "preview_ai_context" || r.kind === "review_ai_messages",
    ),
  ).toBe(false);
});

it("cancels model loading without capturing or sending selected changes", async () => {
  const f = await generation();
  const original = f.request.getMockImplementation()!;
  let resume!: () => void;
  const gate = new Promise<void>((resolve) => {
    resume = resolve;
  });
  f.request.mockImplementation(async (request) => {
    if (request.kind === "ai_jobs") await gate;
    return original(request);
  });
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Fill with AI" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Fill with AI" }));
  await screen.findByText("Loading model…");
  fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  await waitFor(() =>
    expect(f.request).toHaveBeenCalledWith({
      kind: "cancel_ai_job",
      id: "loading",
    }),
  );
  await act(async () => resume());
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Fill with AI" })).toBeEnabled(),
  );
  expect(f.onUse).not.toHaveBeenCalled();
  expect(
    f.request.mock.calls.some(
      ([r]) =>
        r.kind === "preview_ai_context" || r.kind === "review_ai_messages",
    ),
  ).toBe(false);
});

it("streams into the composer before completion and shows Ollama performance metrics", async () => {
  const f = await generation();
  const original = f.request.getMockImplementation()!;
  let polls = 0;
  f.request.mockImplementation(async (r) => {
    const response = await original(r);
    if (r.kind === "ai_jobs" && response.kind === "ai_jobs") {
      for (const job of response.value) {
        if (job.id !== "job") continue;
        polls++;
        job.progress = {
          message:
            polls === 1
              ? "fix: update"
              : "fix: update navigation\n\nDescribe selected changes.",
          total_duration_ms: polls === 1 ? null : 3000,
          tokens_per_second: polls === 1 ? null : 10,
          input_tokens: polls === 1 ? null : 100,
          output_tokens: polls === 1 ? null : 20,
        };
        if (polls === 1) {
          job.phase = "running";
          job.result = null;
        }
      }
    }
    return response;
  });
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Fill with AI" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Fill with AI" }));
  await waitFor(() => expect(f.onUse).toHaveBeenCalledWith("fix: update"));
  expect(screen.getByRole("button", { name: "Cancel" })).toBeEnabled();
  expect(
    screen.getByRole("button", { name: "Generating message…" }),
  ).toBeDisabled();
  await waitFor(() =>
    expect(f.onUse).toHaveBeenCalledWith(
      "fix: update navigation\n\nDescribe selected changes.\n",
    ),
  );
  const info = screen.getByLabelText("AI generation information");
  expect(info).toHaveTextContent("3.0 s");
  expect(info).toHaveTextContent("10.0 tokens/s");
  expect(info).toHaveTextContent("Input tokens100");
  expect(info).toHaveTextContent("Output tokens20");
});

it("retains streamed text and statistics when the server hits its output limit", async () => {
  const f = await generation(true);
  const original = f.request.getMockImplementation()!;
  f.request.mockImplementation(async (r) => {
    const response = await original(r);
    if (r.kind === "ai_jobs" && response.kind === "ai_jobs") {
      for (const job of response.value) {
        if (job.id !== "job") continue;
        job.error =
          "The AI server stopped generation at an output/context limit.";
        job.progress = {
          message: "fix: partial message",
          total_duration_ms: 2000,
          tokens_per_second: 5,
          input_tokens: 100,
          output_tokens: 10,
        };
      }
    }
    return response;
  });
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Fill with AI" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Fill with AI" }));
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "The partial message remains in the editor",
  );
  expect(f.onUse).toHaveBeenLastCalledWith("fix: partial message");
  expect(screen.getByLabelText("AI generation information")).toHaveTextContent(
    "Output tokens10",
  );
});

it("cancels streaming without applying late text or a completed replacement", async () => {
  const f = await generation();
  const original = f.request.getMockImplementation()!;
  let cancelled = false;
  f.request.mockImplementation(async (r) => {
    if (r.kind === "cancel_ai_job") cancelled = true;
    const response = await original(r);
    if (r.kind === "ai_jobs" && response.kind === "ai_jobs") {
      for (const job of response.value) {
        if (job.id !== "job") continue;
        job.phase = cancelled ? "cancelled" : "running";
        job.result = null;
        job.progress = {
          message: cancelled ? "late content" : "fix: partial message",
          total_duration_ms: null,
          tokens_per_second: null,
          input_tokens: null,
          output_tokens: null,
        };
      }
    }
    return response;
  });
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Fill with AI" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Fill with AI" }));
  await waitFor(() =>
    expect(f.onUse).toHaveBeenCalledWith("fix: partial message"),
  );
  fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Fill with AI" })).toBeEnabled(),
  );
  expect(f.onUse).toHaveBeenCalledTimes(1);
  expect(f.request).toHaveBeenCalledWith({ kind: "cancel_ai_job", id: "job" });
});

it("stops late streamed updates when a manual edit supersedes the generation", async () => {
  let owns = true;
  const messages: string[] = [];
  const update = vi.fn((message: string) => {
    if (!owns) return false;
    messages.push(message);
    return true;
  });
  const f = await generation(false, () => update);
  const original = f.request.getMockImplementation()!;
  let polls = 0;
  f.request.mockImplementation(async (request) => {
    const response = await original(request);
    if (request.kind === "ai_jobs" && response.kind === "ai_jobs") {
      for (const job of response.value) {
        if (job.id !== "job") continue;
        polls++;
        job.phase = "running";
        job.result = null;
        job.progress = {
          message: polls === 1 ? "partial" : "late replacement",
          total_duration_ms: null,
          tokens_per_second: null,
          input_tokens: null,
          output_tokens: null,
        };
      }
    }
    return response;
  });
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Fill with AI" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Fill with AI" }));
  await waitFor(() => expect(messages).toEqual(["partial"]));
  owns = false;
  await waitFor(() => expect(update).toHaveBeenCalledWith("late replacement"));
  expect(messages).toEqual(["partial"]);
  expect(f.request).toHaveBeenCalledWith({ kind: "cancel_ai_job", id: "job" });
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Fill with AI" })).toBeEnabled(),
  );
  owns = true;
  f.request.mockImplementation(original);
  fireEvent.click(screen.getByRole("button", { name: "Fill with AI" }));
  await waitFor(() => expect(messages).toHaveLength(2));
  expect(messages).not.toContain("late replacement");
});

it("rejects an old workspace response while a newer generation can complete", async () => {
  const f = await generation();
  const original = f.request.getMockImplementation()!;
  let finish: ((value: Payload) => void) | undefined;
  let old = true;
  f.request.mockImplementation(async (request) => {
    const response = await original(request);
    if (
      old &&
      request.kind === "ai_jobs" &&
      response.kind === "ai_jobs" &&
      response.value.some((j) => j.id === "job")
    ) {
      return new Promise<Payload>((resolve) => {
        finish = resolve;
      });
    }
    return response;
  });
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Fill with AI" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Fill with AI" }));
  await waitFor(() => expect(finish).toBeDefined());
  f.switchWorkspace();
  old = false;
  const response = await original({ kind: "ai_jobs" });
  await act(async () => finish!(response));
  expect(f.onUse).not.toHaveBeenCalled();
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Fill with AI" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Fill with AI" }));
  await waitFor(() => expect(f.onUse).toHaveBeenCalledOnce());
});
