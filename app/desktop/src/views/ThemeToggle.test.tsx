import { afterEach, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ThemeToggle } from "./ThemeToggle";
afterEach(cleanup);
it("switches between light and dark with an accessible action label", async () => {
  const onChange = vi.fn();
  const { rerender } = render(<ThemeToggle theme="dark" onChange={onChange} />);
  await userEvent.click(
    screen.getByRole("button", { name: "Switch to light mode" }),
  );
  expect(onChange).toHaveBeenLastCalledWith("light");
  rerender(<ThemeToggle theme="light" onChange={onChange} />);
  await userEvent.click(
    screen.getByRole("button", { name: "Switch to dark mode" }),
  );
  expect(onChange).toHaveBeenLastCalledWith("dark");
});
it("toggles away from the system’s effective theme", async () => {
  vi.stubGlobal("matchMedia", () => ({
    matches: false,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
  }));
  try {
    const onChange = vi.fn();
    render(<ThemeToggle theme="system" onChange={onChange} />);
    await userEvent.click(
      screen.getByRole("button", { name: "Switch to dark mode" }),
    );
    expect(onChange).toHaveBeenCalledWith("dark");
  } finally {
    vi.unstubAllGlobals();
  }
});
