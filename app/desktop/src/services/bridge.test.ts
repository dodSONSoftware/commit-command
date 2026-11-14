import { beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { NativeBridge, SettingsBridge } from "./bridge";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { Request } from "./request.generated";
import { initialState, initialBulkComposer } from "../stores/model";
vi.mock("@tauri-apps/api/webviewWindow", () => ({
  getCurrentWebviewWindow: vi.fn(),
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
beforeEach(() => {
  vi.mocked(invoke).mockReset();
});
it("changes UI scale through the bounded native settings command", async () => {
  vi.mocked(invoke).mockResolvedValue(125);
  expect(await new NativeBridge().setUiScale(125)).toBe(125);
  expect(invoke).toHaveBeenCalledExactlyOnceWith("set_ui_scale", {
    percent: 125,
  });
});
it("selects AI archives through the fixed native picker without accepting paths", async () => {
  vi.mocked(invoke).mockResolvedValue(null);
  await new NativeBridge().pickAiImport();
  expect(invoke).toHaveBeenCalledExactlyOnceWith("pick_ai_import");
});
it("uses only fixed native directory picker purposes", async () => {
  vi.mocked(invoke).mockResolvedValue(null);
  const bridge = new NativeBridge();
  await bridge.pick();
  expect(invoke).toHaveBeenLastCalledWith("pick_directory", {
    purpose: "workspace",
  });
  await bridge.pick("backup");
  expect(invoke).toHaveBeenLastCalledWith("pick_directory", {
    purpose: "backup",
  });
});
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

it("cancels only the owned Stage job while an earlier read is pending", async () => {
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
  const owner = Symbol("stage");
  const second = bridge.request(
    {
      kind: "propose",
      workspace: "ws",
      worktree: "wt",
      request: {
        kind: "stage",
        paths: [{ encoding: "utf8", value: "file.txt" }],
        hooks: "execute",
      },
    },
    owner,
  );
  await Promise.resolve();
  await Promise.resolve();
  await bridge.cancel(owner);
  expect(invoke).toHaveBeenCalledWith("cancel_job", { job: "job-2" });
  expect(invoke).not.toHaveBeenCalledWith("cancel_job", { job: "job-1" });
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
  vi.mocked(invoke).mockClear();
  await bridge.cancel(owner);
  expect(invoke).not.toHaveBeenCalled();
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

it("opens the native Settings window with the current workspace context", async () => {
  vi.mocked(invoke).mockResolvedValue(undefined);
  await new NativeBridge().openSettings({
    workspace: "ws",
    worktree: "wt",
    theme: "dark",
  });
  expect(invoke).toHaveBeenCalledExactlyOnceWith("open_settings", {
    workspace: "ws",
    worktree: "wt",
    theme: "dark",
  });
});

it("bootstraps and resynchronizes the separate Settings connection before loading its forms", async () => {
  const requests: Array<{ epoch: number; request: Request }> = [];
  let currentEpoch = 8;
  let last: Request = { kind: "app_preferences" };
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    if (command === "dispatch") {
      const envelope = JSON.parse(
        String((args as { payload: string }).payload),
      );
      requests.push(envelope);
      last = envelope.request;
      if (last.kind !== "app_preferences" && envelope.epoch !== currentEpoch)
        throw new Error("stale_review");
      return "settings-job";
    }
    if (command === "job_status")
      return {
        state: "complete",
        response: {
          version: 1,
          epoch: currentEpoch,
          data:
            last.kind === "app_preferences"
              ? {
                  kind: "app_preferences",
                  value: {
                    textarea_heights: {},
                    generation_timeout_seconds: 60,
                  },
                }
              : {
                  kind: "ai_settings",
                  value: {
                    fingerprint: "f",
                    settings: {
                      version: 1,
                      revision: "r",
                      connections: [],
                      profiles: [],
                      templates: [],
                      assignments: [],
                      instructions: [],
                      global_concurrency: 1,
                      connection_concurrency: 1,
                    },
                  },
                },
        },
      };
    throw new Error(`Unexpected command ${command}`);
  });
  const bridge = new SettingsBridge();
  expect((await bridge.request({ kind: "ai_settings" })).kind).toBe(
    "ai_settings",
  );
  currentEpoch = 9;
  await bridge.request({ kind: "ai_settings" });
  expect(requests.map((r) => [r.epoch, r.request.kind])).toEqual([
    [0, "app_preferences"],
    [8, "ai_settings"],
    [8, "app_preferences"],
    [9, "ai_settings"],
  ]);
  expect(bridge.epoch).toBe(9);
});
it("does not silently refresh or replay a stale Settings save", async () => {
  vi.mocked(invoke)
    .mockResolvedValueOnce("save-job")
    .mockResolvedValueOnce({
      state: "complete",
      error: { kind: "stale_review", message: "Application context changed" },
    });
  const bridge = new SettingsBridge();
  await expect(
    bridge.request({
      kind: "save_ai_settings",
      fingerprint: "old",
      revision: "old",
      change: { kind: "concurrency", global: 1, per_connection: 1 },
    }),
  ).rejects.toThrow("stale_review");
  expect(
    vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "dispatch"),
  ).toHaveLength(1);
});

it.each([true, false])(
  "waits for the Settings flush before acknowledging quit, saved=%s",
  async (saved) => {
    const handlers = new Map<
      string,
      (event: { payload: string }) => Promise<void> | void
    >();
    const dispose = vi.fn();
    const listen = vi.fn(
      async (
        event: string,
        handler: (event: { payload: string }) => Promise<void> | void,
      ) => {
        handlers.set(event, handler);
        return dispose;
      },
    );
    vi.mocked(getCurrentWebviewWindow).mockReturnValue({
      listen,
    } as unknown as ReturnType<typeof getCurrentWebviewWindow>);
    vi.mocked(invoke).mockResolvedValue(undefined);
    let finish!: (saved: boolean) => void;
    const changed = vi.fn();
    const stop = await new SettingsBridge().onSettingsCloseRequested(
      () =>
        new Promise((resolve) => {
          finish = resolve;
        }),
      changed,
    );
    const pending = handlers.get("gitfleet-settings-close-request")!({
      payload: "native-settings-token",
    });
    expect(changed).toHaveBeenLastCalledWith(true);
    expect(invoke).not.toHaveBeenCalled();
    finish(saved);
    await pending;
    expect(invoke).toHaveBeenCalledExactlyOnceWith("settings_flush_done", {
      token: "native-settings-token",
      saved,
    });
    handlers.get("gitfleet-close-cancelled")!({ payload: "" });
    expect(changed).toHaveBeenLastCalledWith(false);
    stop();
    expect(dispose).toHaveBeenCalledTimes(2);
  },
);
