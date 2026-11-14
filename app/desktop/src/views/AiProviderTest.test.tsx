import { afterEach, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { Bridge } from "../services/bridge";
import type { AiJob } from "../services/response.generated";
import { AiProviderTest, providerModelKey } from "./AiProviderTest";
afterEach(cleanup);
const connection = {
  id: "saved",
  name: "Local AI",
  provider: "ollama" as const,
  endpoint: "http://127.0.0.1:11434/",
  timeout_ms: 30000,
  authentication: "none" as const,
};
function fixture(
  options: { failed?: boolean; empty?: boolean; deferred?: boolean } = {},
) {
  const result: AiJob = {
    id: "current-job",
    phase: options.failed ? "failed" : "complete",
    error: options.failed ? "Connection refused" : null,
    review: {
      endpoint: connection.endpoint,
      route: connection.endpoint + "api/tags",
      fingerprint: "exact-test",
      purpose: "discover",
      request_bytes: 0,
      authentication: "none",
      unencrypted: true,
    },
    result: options.failed
      ? null
      : {
          kind: "models",
          value: options.empty
            ? []
            : [{ id: "qwen3.5:9b-q4_K_M" }, { id: "llama3.2:latest" }],
        },
  };
  const stale = structuredClone(result);
  stale.id = "old-job";
  stale.review.fingerprint = "old-test";
  stale.phase = "complete";
  stale.result = { kind: "models", value: [{ id: "wrong-model" }] };
  let reviewed = false;
  const previousSame = structuredClone(stale);
  previousSame.id = "previous-same-provider";
  previousSame.review.fingerprint = "exact-test";
  const request = vi.fn<Bridge["request"]>(async (r) => {
    if (r.kind === "review_ai_discovery") {
      reviewed = true;
      return {
        kind: "executed",
        value: {
          result: { ai_job: "current-job" },
          ai_job_id: "current-job",
          ai_request_fingerprint: "exact-test",
        },
      };
    }
    if (r.kind === "ai_jobs")
      return {
        kind: "ai_jobs",
        value:
          !reviewed || options.deferred
            ? [previousSame, stale]
            : [previousSame, stale, result],
      };
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
  const onResult = vi.fn();
  const view = render(
    <AiProviderTest
      bridge={bridge}
      connection={connection}
      active
      onResult={onResult}
    />,
  );
  return { request, onResult, view, user: userEvent.setup(), bridge };
}
it("only tests on request and returns models from the exact started test", async () => {
  const { request, onResult, user } = fixture();
  expect(request).not.toHaveBeenCalled();
  await user.click(screen.getByRole("button", { name: "Test provider" }));
  expect(request).toHaveBeenCalledWith({
    kind: "review_ai_discovery",
    connection,
  });
  await screen.findByText("Provider reachable · 2 models available.");
  expect(onResult.mock.calls).toEqual([
    [null],
    [[{ id: "qwen3.5:9b-q4_K_M" }, { id: "llama3.2:latest" }]],
  ]);
  expect(
    request.mock.calls.every(([r]) =>
      ["review_ai_discovery", "ai_jobs"].includes(r.kind),
    ),
  ).toBe(true);
});
it("reports an unreachable provider and clears its previous model list", async () => {
  const { onResult, user } = fixture({ failed: true });
  await user.click(screen.getByRole("button", { name: "Test provider" }));
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Connection refused",
  );
  expect(onResult).toHaveBeenLastCalledWith(null);
  expect(screen.getByRole("button", { name: "Test provider" })).toBeEnabled();
});
it("distinguishes a reachable provider with no installed models from failure", async () => {
  const { onResult, user } = fixture({ empty: true });
  await user.click(screen.getByRole("button", { name: "Test provider" }));
  await screen.findByText("Provider reachable · 0 models available.");
  expect(onResult).toHaveBeenLastCalledWith([]);
  expect(screen.queryByRole("alert")).toBeNull();
});
it("ignores older results even for the same endpoint", async () => {
  const { request, onResult, user, view } = fixture({ deferred: true });
  await user.click(screen.getByRole("button", { name: "Test provider" }));
  await waitFor(() =>
    expect(request).toHaveBeenCalledWith({ kind: "ai_jobs" }),
  );
  expect(onResult.mock.calls).toEqual([[null]]);
  view.unmount();
});
it("keys model lists by exact connection, provider, endpoint and authentication", () => {
  const key = providerModelKey(connection);
  expect(
    providerModelKey({ ...connection, endpoint: "http://127.0.0.1:11434" }),
  ).toBe(key);
  for (const change of [
    { id: "other" },
    { provider: "compatible" as const },
    { endpoint: "http://127.0.0.1:11435/" },
    { authentication: "stored_bearer" as const },
  ])
    expect(providerModelKey({ ...connection, ...change })).not.toBe(key);
});

it("strips saved-record metadata before starting a provider test", async () => {
  const { view, bridge, onResult, user, request } = fixture();
  const saved = {
    ...connection,
    revision: "stored-revision",
    endpoint_approval: "stored-metadata",
  };
  view.rerender(
    <AiProviderTest
      bridge={bridge}
      connection={saved}
      active
      onResult={onResult}
    />,
  );
  await user.click(screen.getByRole("button", { name: "Test provider" }));
  await screen.findByText(/Provider reachable/);
  expect(request).toHaveBeenCalledWith({
    kind: "review_ai_discovery",
    connection,
  });
});
it("explains local request rejection without blaming server availability", async () => {
  const { request, user } = fixture();
  request.mockRejectedValueOnce("Malformed or unsupported request");
  await user.click(screen.getByRole("button", { name: "Test provider" }));
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "before contacting http://127.0.0.1:11434/",
  );
  expect(screen.getByRole("alert")).toHaveTextContent("app error");
});
it("starts a fresh live request on every click, even after a successful test", async () => {
  const { user, request } = fixture();
  await user.click(screen.getByRole("button", { name: "Test provider" }));
  await screen.findByText(/Provider reachable/);
  await user.click(screen.getByRole("button", { name: "Test provider" }));
  await screen.findByText(/Provider reachable/);
  expect(
    request.mock.calls.filter(([r]) => r.kind === "review_ai_discovery"),
  ).toHaveLength(2);
});
