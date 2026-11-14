import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { Bridge } from "../services/bridge";
import { AiSettings } from "./AiSettings";
beforeEach(() => {
  HTMLDialogElement.prototype.showModal = function () {
    this.setAttribute("open", "");
  };
  HTMLDialogElement.prototype.close = function () {
    this.removeAttribute("open");
    this.dispatchEvent(new Event("close"));
  };
});
afterEach(cleanup);
function fixture(
  onUiScaleChange = vi
    .fn<(n: number) => Promise<void>>()
    .mockResolvedValue(undefined),
) {
  const bridge = {
    request: vi.fn().mockRejectedValue(new Error("Unavailable")),
  } as unknown as Bridge;
  render(
    <AiSettings
      bridge={bridge}
      workspace={null}
      disabled={false}
      uiScale={125}
      onUiScaleChange={onUiScaleChange}
    />,
  );
  return { user: userEvent.setup(), onUiScaleChange };
}
it("has only Appearance and AI, with keyboard navigation", async () => {
  const { user } = fixture();
  await user.click(screen.getByRole("button", { name: "Settings…" }));
  expect(screen.getAllByRole("tab").map((t) => t.textContent)).toEqual([
    "Appearance",
    "AI",
  ]);
  await user.click(screen.getByRole("tab", { name: "AI" }));
  expect(screen.getByRole("heading", { name: "AI" })).toBeInTheDocument();
  await user.keyboard("{Home}");
  expect(screen.getByRole("tab", { name: "Appearance" })).toHaveFocus();
  expect(
    screen.queryByText(/profiles|advanced AI|configuration backups/i),
  ).toBeNull();
});
it("changes scale live, resets, and reports a failed save", async () => {
  const { user, onUiScaleChange } = fixture();
  await user.click(screen.getByRole("button", { name: "Settings…" }));
  const slider = screen.getByRole("slider", { name: "UI scale" });
  fireEvent.input(slider, { target: { value: "150" } });
  expect(onUiScaleChange).toHaveBeenLastCalledWith(150);
  await user.click(
    screen.getByRole("button", { name: "Reset UI scale to 100%" }),
  );
  expect(onUiScaleChange).toHaveBeenLastCalledWith(100);
  onUiScaleChange.mockRejectedValueOnce(new Error("Scale save failed"));
  fireEvent.input(slider, { target: { value: "200" } });
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Scale save failed",
  );
});
it("keeps errors in a top-level banner outside the scrolling settings content", async () => {
  const { user } = fixture();
  await user.click(screen.getByRole("button", { name: "Settings…" }));
  const banner = await screen.findByRole("alert");
  expect(banner).toHaveClass("error", "settings-error-banner");
  expect(banner.closest(".settings-content")).toBeNull();
  expect(banner.parentElement).toBe(screen.getByRole("dialog"));
  expect(banner.previousElementSibling?.tagName).toBe("HEADER");
  await user.click(screen.getByRole("tab", { name: "AI" }));
  expect(screen.getByRole("alert")).toBe(banner);
});
