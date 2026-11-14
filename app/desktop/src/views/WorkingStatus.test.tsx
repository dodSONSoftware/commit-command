import { act, cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { WorkingStatus } from "./WorkingStatus";
afterEach(() => {
  cleanup();
  vi.useRealTimers();
});
it("shows elapsed time for long operations and releases its timer on completion", () => {
  vi.useFakeTimers();
  const view = render(<WorkingStatus label="Staging 12 files…" />);
  expect(screen.getByRole("status")).toHaveTextContent("Staging 12 files…");
  expect(screen.queryByText("0s")).toBeNull();
  act(() => vi.advanceTimersByTime(5000));
  expect(screen.getByText("5s")).toBeVisible();
  view.rerender(<WorkingStatus label="Waiting for the model…" />);
  act(() => vi.advanceTimersByTime(1000));
  expect(screen.getByRole("status")).toHaveTextContent(
    "Waiting for the model…6s",
  );
  view.unmount();
  expect(vi.getTimerCount()).toBe(0);
});
