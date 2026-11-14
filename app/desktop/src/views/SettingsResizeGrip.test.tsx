import { afterEach, expect, it, vi } from "vitest";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { SettingsResizeGrip } from "./SettingsResizeGrip";
vi.mock("@tauri-apps/api/webviewWindow", () => ({
  getCurrentWebviewWindow: vi.fn(),
}));
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
  Reflect.deleteProperty(HTMLElement.prototype, "setPointerCapture");
});
it("keeps the final size of a quick drag while native measurements are pending", async () => {
  vi.stubGlobal("PointerEvent", MouseEvent);
  Object.defineProperty(HTMLElement.prototype, "setPointerCapture", {
    configurable: true,
    value: vi.fn(),
  });
  let measured: (size: { width: number; height: number }) => void = () => {};
  const setSize = vi.fn().mockResolvedValue(undefined);
  vi.mocked(getCurrentWebviewWindow).mockReturnValue({
    innerSize: () =>
      new Promise((resolve) => {
        measured = resolve;
      }),
    scaleFactor: async () => 2,
    setSize,
  } as unknown as ReturnType<typeof getCurrentWebviewWindow>);
  render(<SettingsResizeGrip onError={vi.fn()} />);
  const grip = screen.getByRole("button", { name: "Resize Settings window" });
  fireEvent.pointerDown(grip, { button: 0, screenX: 900, screenY: 700 });
  fireEvent.pointerMove(grip, { screenX: 800, screenY: 600 });
  fireEvent.pointerUp(grip, { screenX: 800, screenY: 600 });
  fireEvent.lostPointerCapture(grip);
  await act(async () => measured({ width: 1920, height: 1520 }));
  await waitFor(() =>
    expect(setSize).toHaveBeenLastCalledWith(
      expect.objectContaining({ width: 860, height: 660 }),
    ),
  );
  const count = setSize.mock.calls.length;
  fireEvent.pointerMove(grip, { screenX: 700, screenY: 500 });
  expect(setSize).toHaveBeenCalledTimes(count);
});
