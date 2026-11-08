import { afterEach, expect, it, vi } from "vitest";
import { fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
afterEach(() => {
  vi.clearAllTimers();
  vi.useRealTimers();
  vi.mocked(invoke).mockReset();
  vi.resetModules();
});
it("renders authoritative content only as text and sends no approval boolean", async () => {
  document.body.innerHTML = '<main id="review"></main>';
  vi.mocked(invoke)
    .mockResolvedValueOnce({
      id: "immutable-review",
      can_confirm: true,
      epoch: 1,
      expires_at_ms: Date.now() + 600000,
      title: "Review controlled write",
      summary: "Actual Rust summary",
      content: "<script>evil()</script><img src=x onerror=evil()>",
    })
    .mockResolvedValueOnce({ phase: "complete" });
  await import("./review");
  await waitFor(() =>
    expect(document.querySelector("button.primary")).toBeTruthy(),
  );
  expect(document.querySelector("#review script")).toBeNull();
  expect(document.querySelector("#review img")).toBeNull();
  expect(document.getElementById("review")!.textContent).toContain(
    "<script>evil()</script>",
  );
  fireEvent.click(document.querySelector("button.primary")!);
  await waitFor(() =>
    expect(invoke).toHaveBeenCalledWith("review_confirm", {
      review: "immutable-review",
    }),
  );
  expect(vi.mocked(invoke).mock.calls[1][1]).not.toHaveProperty("accepted");
});

it("disables an expired authoritative review and never opens native confirmation", async () => {
  vi.useFakeTimers();
  document.body.innerHTML = '<main id="review"></main>';
  vi.mocked(invoke).mockResolvedValueOnce({
    id: "expires",
    can_confirm: true,
    epoch: 1,
    expires_at_ms: Date.now() + 1000,
    title: "Review",
    summary: "Bounded review",
    content: "Exact stored plan",
  });
  await import("./review");
  const confirm = document.querySelector<HTMLButtonElement>("button.primary")!;
  expect(confirm.disabled).toBe(false);
  await vi.advanceTimersByTimeAsync(1000);
  expect(confirm.disabled).toBe(true);
  expect(document.querySelector('[role="status"]')).toHaveTextContent(
    "Review expired",
  );
  expect(document.querySelector("button")).toHaveTextContent("Close review");
  fireEvent.click(confirm);
  expect(invoke).toHaveBeenCalledTimes(1);
});
it("does not label an executing operation expired when its review deadline passes", async () => {
  vi.useFakeTimers();
  document.body.innerHTML = '<main id="review"></main>';
  let finish: ((result: unknown) => void) | undefined;
  vi.mocked(invoke)
    .mockResolvedValueOnce({
      id: "in-flight",
      can_confirm: true,
      epoch: 1,
      expires_at_ms: Date.now() + 1000,
      title: "Review",
      summary: "Bounded review",
      content: "Exact stored plan",
    })
    .mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          finish = resolve;
        }),
    );
  await import("./review");
  fireEvent.click(document.querySelector("button.primary")!);
  await vi.advanceTimersByTimeAsync(2000);
  expect(document.querySelector('[role="status"]')).toHaveTextContent(
    "Waiting for native approval or the approved operation to finish",
  );
  expect(document.querySelector('[role="status"]')).not.toHaveTextContent(
    "Review expired",
  );
  finish!({ phase: "complete" });
  await Promise.resolve();
  expect(document.querySelector('[role="status"]')).toHaveTextContent(
    "Core result",
  );
});

it("rejects a click at the deadline even before the expiry timer callback runs", async () => {
  vi.useFakeTimers();
  document.body.innerHTML = '<main id="review"></main>';
  const deadline = Date.now() + 1000;
  vi.mocked(invoke).mockResolvedValueOnce({
    id: "deadline-race",
    can_confirm: true,
    epoch: 1,
    expires_at_ms: deadline,
    title: "Review",
    summary: "Bounded review",
    content: "Exact stored plan",
  });
  await import("./review");
  // Move Date without running timers to exercise the click guard independently.
  vi.setSystemTime(deadline);
  fireEvent.click(document.querySelector("button.primary")!);
  expect(invoke).toHaveBeenCalledTimes(1);
  expect(document.querySelector('[role="status"]')).toHaveTextContent(
    "Review expired",
  );
});
