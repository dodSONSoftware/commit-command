import { expect, it, vi } from "vitest";
import { fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
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
