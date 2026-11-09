import { afterEach, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { Bridge } from "../services/bridge";
import type { AiTemplate, AiJob } from "../services/response.generated";
import { AiDiagnostics } from "./AiDiagnostics";
afterEach(cleanup);
const connection = {
  id: null,
  name: "Unsaved connection",
  provider: "ollama" as const,
  endpoint: "http://127.0.0.1:11434/",
  timeout_ms: 1234,
};
const profile = {
  id: null,
  name: "Unsaved profile",
  connection: "unsaved",
  model: "unsaved-model",
  system_message: "Unsaved system text",
  parameters: { temperature: 0.3 },
  template: "template",
  context_bytes: 16384,
};
const template: AiTemplate = {
  id: "template",
  revision: "revision",
  name: "Unsaved template",
  rules: {
    style: "plain",
    subject_max_chars: 64,
    allowed_types: [],
    allow_scope: false,
    forbid_trailing_period: true,
    instructions: "Unsaved rule",
  },
  examples: [],
};
function fixture(jobs: AiJob[] = []) {
  const request = vi.fn<Bridge["request"]>(async (r) =>
    r.kind === "ai_jobs" ? { kind: "ai_jobs", value: jobs } : { kind: "done" },
  );
  const bridge: Bridge = {
    epoch: 1,
    request,
    capabilities: async () => ({ writes_enabled: false }),
    writeStatus: async () => false,
    pick: async () => null,
    cancel: async () => {},
  };
  const onModels = vi.fn();
  const props = {
    bridge,
    connection,
    profile,
    template,
    active: true,
    onModels,
  };
  const view = render(<AiDiagnostics {...props} />);
  return { request, onModels, user: userEvent.setup(), view, props };
}
it("does no work until requested and reviews the exact unsaved synthetic inputs", async () => {
  const { request, user } = fixture();
  expect(request).not.toHaveBeenCalled();
  await user.click(
    screen.getByRole("button", { name: "Review synthetic test inference" }),
  );
  expect(request).toHaveBeenCalledWith({
    kind: "review_ai_inference",
    connection,
    profile,
    template,
  });
  expect(
    request.mock.calls.every(([r]) =>
      ["review_ai_inference", "ai_jobs"].includes(r.kind),
    ),
  ).toBe(true);
});
it("model discovery only populates models for the currently edited destination", async () => {
  const jobs: AiJob[] = [
    {
      id: "other",
      phase: "complete",
      error: null,
      error_kind: null,
      review: {
        authentication: "none",
        endpoint: "http://other.test/",
        route: "http://other.test/api/tags",
        fingerprint: "other",
        purpose: "discover",
        request_bytes: 0,
        unencrypted: true,
        model: null,
      },
      result: { kind: "models", value: [{ id: "wrong-model", size: null }] },
    },
    {
      id: "current",
      phase: "complete",
      error: null,
      error_kind: null,
      review: {
        authentication: "none",
        endpoint: connection.endpoint,
        route: connection.endpoint + "api/tags",
        fingerprint: "current",
        purpose: "discover",
        request_bytes: 0,
        unencrypted: true,
        model: null,
      },
      result: { kind: "models", value: [{ id: "exact-model", size: 12 }] },
    },
  ];
  const { request, onModels, user, view, props } = fixture(jobs);
  await user.click(
    screen.getByRole("button", { name: "Show AI diagnostic results" }),
  );
  await waitFor(() =>
    expect(onModels).toHaveBeenCalledWith([{ id: "exact-model", size: 12 }]),
  );
  expect(onModels).toHaveBeenCalledTimes(1);
  view.rerender(<AiDiagnostics {...props} active={false} />);
  const count = request.mock.calls.length;
  await new Promise((resolve) => setTimeout(resolve, 550));
  expect(request.mock.calls).toHaveLength(count);
});
it("cancels only the chosen provider job", async () => {
  const jobs: AiJob[] = [
    {
      id: "running",
      phase: "running",
      error: null,
      error_kind: null,
      review: {
        authentication: "none",
        endpoint: connection.endpoint,
        route: connection.endpoint + "api/chat",
        fingerprint: "running",
        purpose: "inference",
        request_bytes: 100,
        unencrypted: true,
        model: "unsaved-model",
      },
      result: null,
    },
  ];
  const { request, user } = fixture(jobs);
  await user.click(
    screen.getByRole("button", { name: "Show AI diagnostic results" }),
  );
  await user.click(
    await screen.findByRole("button", { name: "Cancel this AI request" }),
  );
  expect(request).toHaveBeenCalledWith({
    kind: "cancel_ai_job",
    id: "running",
  });
});
