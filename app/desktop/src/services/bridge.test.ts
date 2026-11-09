import { beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { NativeBridge } from "./bridge";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { initialState, initialBulkComposer } from "../stores/model";
vi.mock("@tauri-apps/api/webviewWindow", () => ({
  getCurrentWebviewWindow: vi.fn(),
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
beforeEach(() => vi.mocked(invoke).mockReset());
it("sends scoped requests without approval tokens or booleans", async () => {
  vi.mocked(invoke)
    .mockResolvedValueOnce("job")
    .mockResolvedValueOnce({
      state: "complete",
      response: {
        version: 1,
        epoch: 1,
        data: {
          kind: "workspaces",
          value: { workspaces: [], active: null, recovery: [] },
        },
      },
    });
  const bridge = new NativeBridge();
  await bridge.request({ kind: "workspaces" });
  const args = vi.mocked(invoke).mock.calls[0][1];
  if (!args || typeof args !== "object" || !("payload" in args))
    throw new Error("Missing request payload");
  const body = JSON.parse(String(args.payload));
  expect(body).toEqual({
    version: 1,
    epoch: 0,
    request: { kind: "workspaces" },
  });
  expect(bridge.epoch).toBe(1);
  expect(vi.mocked(invoke).mock.calls.map((c) => c[0])).toEqual([
    "dispatch",
    "job_status",
  ]);
});
it("rejects late responses instead of replacing current workspace state", async () => {
  vi.mocked(invoke)
    .mockResolvedValueOnce("job")
    .mockResolvedValueOnce({
      state: "complete",
      response: { version: 1, epoch: 1, data: { kind: "done" } },
    });
  const bridge = new NativeBridge();
  bridge.epoch = 2;
  await expect(bridge.request({ kind: "workspaces" })).rejects.toThrow("Stale");
  expect(bridge.epoch).toBe(2);
});
it("surfaces uncertain backend errors without retrying an operation", async () => {
  vi.mocked(invoke)
    .mockResolvedValueOnce("job")
    .mockResolvedValueOnce({
      state: "complete",
      error: { kind: "uncertain_outcome", message: "Inspect recovery" },
    });
  const bridge = new NativeBridge();
  await expect(bridge.request({ kind: "workspaces" })).rejects.toThrow(
    "uncertain_outcome",
  );
  expect(invoke).toHaveBeenCalledTimes(2);
});

it("cancels the earliest outstanding Core job instead of a later queued background request", async () => {
  let count = 0;
  const pending = new Map<string, (value: unknown) => void>();
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    const job =
      args && typeof args === "object" && "job" in args ? args.job : undefined;
    if (command === "dispatch") return `job-${++count}`;
    if (command === "job_status" && job === "native-write")
      return { state: "idle" };
    if (command === "job_status")
      return new Promise((resolve) => {
        pending.set(String(job), resolve);
      });
    return undefined;
  });
  const bridge = new NativeBridge();
  const first = bridge.request({ kind: "workspaces" });
  const second = bridge.request({ kind: "workspaces" });
  await Promise.resolve();
  await Promise.resolve();
  await bridge.cancel();
  expect(invoke).toHaveBeenCalledWith("cancel_job", { job: "job-1" });
  const result = {
    state: "complete",
    response: {
      version: 1,
      epoch: 1,
      data: {
        kind: "workspaces",
        value: { workspaces: [], active: null, recovery: [] },
      },
    },
  };
  pending.get("job-1")!(result);
  pending.get("job-2")!(result);
  await Promise.all([first, second]);
});

it("observes native completion revisions even when a fast operation is already idle", async () => {
  vi.mocked(invoke)
    .mockResolvedValueOnce({ state: "idle", response: { revision: 0 } })
    .mockResolvedValueOnce({ state: "idle", response: { revision: 2 } })
    .mockResolvedValueOnce({ state: "running", response: { revision: 2 } })
    .mockResolvedValueOnce({ state: "idle", response: { revision: -1 } });
  const bridge = new NativeBridge();
  expect(await bridge.writeStatus()).toBe(false);
  expect(bridge.writeRevision).toBe(0);
  expect(await bridge.writeStatus()).toBe(false);
  expect(bridge.writeRevision).toBe(2);
  expect(await bridge.writeStatus()).toBe(true);
  await expect(bridge.writeStatus()).rejects.toThrow(
    "Invalid native completion revision",
  );
  expect(bridge.writeRevision).toBe(2);
});

it("listens on the current native window and sends only the exact one-use close snapshot", async () => {
  const dispose = vi.fn();
  const listen = vi
    .fn<
      (
        event: string,
        handler: (event: { payload: string }) => Promise<void>,
      ) => Promise<() => void>
    >()
    .mockResolvedValue(dispose);
  vi.mocked(getCurrentWebviewWindow).mockReturnValue({
    listen,
  } as unknown as ReturnType<typeof getCurrentWebviewWindow>);
  vi.mocked(invoke)
    .mockResolvedValueOnce(undefined)
    .mockRejectedValueOnce(new Error("Close cancelled"));
  const changed = vi.fn();
  const snapshot = {
    version: 1 as const,
    epoch: 4,
    workspace: "ws",
    navigation: initialState,
    bulk: initialBulkComposer,
  };
  const bridge = new NativeBridge();
  const unlisten = await bridge.onCloseRequested(() => snapshot, changed);
  expect(listen.mock.calls[0][0]).toBe("gitfleet-close-request");
  await listen.mock.calls[0][1]({ payload: "native-token" });
  expect(invoke).toHaveBeenCalledWith("close_snapshot", {
    token: "native-token",
    payload: JSON.stringify(snapshot),
  });
  expect(changed).toHaveBeenLastCalledWith(true);
  await listen.mock.calls[0][1]({ payload: "retry-token" });
  expect(changed).toHaveBeenLastCalledWith(false, "Error: Close cancelled");
  unlisten();
  expect(dispose).toHaveBeenCalledOnce();
});

it("opens native AI token management using only a saved connection identifier", async () => {
  vi.mocked(invoke).mockResolvedValueOnce(undefined);
  const bridge = new NativeBridge();
  await bridge.manageAiToken("saved-connection-id");
  expect(invoke).toHaveBeenCalledExactlyOnceWith("manage_ai_token", {
    connection: "saved-connection-id",
  });
});
