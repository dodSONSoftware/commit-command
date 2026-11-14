import { afterEach, beforeEach, expect, it, vi } from "vitest";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { SettingsLauncher } from "./SettingsLauncher";
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
    },
  });
});
afterEach(() => {
  cleanup();
  Reflect.deleteProperty(HTMLDialogElement.prototype, "showModal");
  Reflect.deleteProperty(HTMLDialogElement.prototype, "close");
});
it("blocks the owner until the native dialog closes and then restores focus", async () => {
  let finish = () => {};
  const open = vi
    .fn()
    .mockImplementationOnce(
      () =>
        new Promise<void>((resolve) => {
          finish = resolve;
        }),
    )
    .mockResolvedValue(undefined);
  render(<SettingsLauncher disabled={false} open={open} />);
  const button = screen.getByRole("button", { name: "Settings…" });
  await userEvent.click(button);
  const dialog = screen.getByRole("dialog", { name: "Settings is open" });
  expect(dialog).toHaveAttribute("open");
  const cancel = new Event("cancel", { cancelable: true });
  fireEvent(dialog, cancel);
  expect(cancel.defaultPrevented).toBe(true);
  expect(dialog).toHaveAttribute("open");
  await act(async () => finish());
  expect(dialog).not.toHaveAttribute("open");
  expect(button).toHaveFocus();
});
it("unblocks the owner and reports a native opening failure", async () => {
  render(
    <SettingsLauncher
      disabled={false}
      open={async () => {
        throw new Error("Cannot open Settings");
      }}
    />,
  );
  await userEvent.click(screen.getByRole("button", { name: "Settings…" }));
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Cannot open Settings",
  );
  expect(screen.queryByRole("dialog")).toBeNull();
});
