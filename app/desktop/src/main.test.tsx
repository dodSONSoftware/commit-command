import { expect, it, vi } from "vitest";

vi.mock("./views/App", () => ({
  App: () => <main aria-label="Application shell">Commit Command</main>,
}));
vi.mock("./services/bridge", () => ({ NativeBridge: class {} }));

it("commits the application shell before bootstrap module completion", async () => {
  document.body.innerHTML =
    '<div id="root"><p role="status">Starting Commit Command…</p></div>';
  await import("./main");
  expect(
    document.querySelector('[aria-label="Application shell"]'),
  ).not.toBeNull();
  expect(document.querySelector('[role="status"]')).toBeNull();
});
