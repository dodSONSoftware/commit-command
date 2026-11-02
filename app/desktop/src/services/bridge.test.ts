import { beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { NativeBridge } from "./bridge";
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
