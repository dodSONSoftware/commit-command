import { afterEach, beforeEach, expect, it, vi } from "vitest";
import {
  act,
  cleanup,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { Bridge } from "../services/bridge";
import { ApplicationBrand } from "./ApplicationBrand";
// JSDOM has no modal dialog methods. Native focus trapping/Escape are verified in the app.
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
function bridge(): Bridge {
  return {
    epoch: 1,
    request: vi.fn(),
    capabilities: async () => ({ writes_enabled: false }),
    writeStatus: async () => false,
    pick: async () => null,
    cancel: async () => {},
    openProjectRepository: vi.fn().mockResolvedValue(undefined),
  };
}
it("opens the branded About dialog, shows the local license and restores focus", async () => {
  const user = userEvent.setup();
  render(<ApplicationBrand bridge={bridge()} />);
  const brand = screen.getByRole("button", { name: "About Commit Command" });
  await user.click(brand);
  const dialog = screen.getByRole("dialog", { name: "Commit Command" });
  expect(within(dialog).getByText("version 0.1.1")).toBeVisible();
  expect(within(dialog).getByText("Randy Dodson")).toBeVisible();
  await user.click(
    within(dialog).getByRole("button", { name: "View license…" }),
  );
  expect(
    within(dialog).getByText(/Permission is hereby granted, free of charge/),
  ).toBeVisible();
  await user.click(within(dialog).getByRole("button", { name: "Back" }));
  expect(
    within(dialog).getByText("Local-first Git workspace manager for macOS."),
  ).toBeVisible();
  await user.click(within(dialog).getByRole("button", { name: "OK" }));
  expect(screen.queryByRole("dialog")).toBeNull();
  expect(brand).toHaveFocus();
});
it("uses the same dialog for the application menu and unsubscribes on unmount", async () => {
  const host = bridge();
  let show: () => void = () => {};
  const stop = vi.fn();
  host.onAboutRequested = vi.fn(async (callback) => {
    show = callback;
    return stop;
  });
  const { unmount } = render(<ApplicationBrand bridge={host} />);
  await waitFor(() => expect(host.onAboutRequested).toHaveBeenCalledOnce());
  act(() => show());
  expect(screen.getByRole("dialog", { name: "Commit Command" })).toBeVisible();
  unmount();
  expect(stop).toHaveBeenCalledOnce();
  expect(host.request).not.toHaveBeenCalled();
});
it("opens only the fixed native repository action and reports launch failure", async () => {
  const host = bridge();
  host.openProjectRepository = vi
    .fn()
    .mockRejectedValue(new Error("unavailable"));
  const user = userEvent.setup();
  render(<ApplicationBrand bridge={host} />);
  await user.click(
    screen.getByRole("button", { name: "About Commit Command" }),
  );
  await user.click(screen.getByRole("button", { name: "GitHub repository…" }));
  await screen.findByRole("alert");
  expect(host.openProjectRepository).toHaveBeenCalledWith();
  expect(host.request).not.toHaveBeenCalled();
});
