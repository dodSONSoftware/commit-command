import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { Bridge } from "../services/bridge";
import type { VersionPreview } from "../services/response.generated";
import { VersionAction } from "./VersionAction";
afterEach(cleanup);
const bridge = (request: Bridge["request"]): Bridge => ({
  epoch: 1,
  request,
  capabilities: async () => ({ writes_enabled: false }),
  writeStatus: async () => false,
  pick: async () => null,
  cancel: async () => {},
});
const result: VersionPreview = {
  old: "1.4.2",
  new: "1.4.3",
  increment: "patch",
  prerelease: "reject",
  before_hash: "old",
  after_hash: "new",
  before: [49],
  after: [50],
};
function setup(busy = false) {
  const request = vi
    .fn<Bridge["request"]>()
    .mockResolvedValue({ kind: "version_preview", value: result });
  render(
    <VersionAction
      bridge={bridge(request)}
      workspace="ws"
      worktree="wt"
      busy={busy}
      act={(f) => f()}
    />,
  );
  return request;
}
it("does nothing until explicitly requested and needs no AI setup", async () => {
  const request = setup();
  expect(request).not.toHaveBeenCalled();
  fireEvent.click(screen.getByText("Preview version update"));
  fireEvent.click(
    screen.getByRole("button", { name: "Read version update preview" }),
  );
  await screen.findByRole("region", { name: "Version update preview" });
  expect(screen.getByText("1.4.3")).toBeTruthy();
  expect(request.mock.calls.map((c) => c[0])).toEqual([
    {
      kind: "preview_version_file",
      workspace: "ws",
      worktree: "wt",
      path: { encoding: "utf8", value: "version" },
      increment: "patch",
      prerelease: "reject",
    },
  ]);
  expect(
    screen.queryByRole("button", { name: /apply|stage|commit/i }),
  ).toBeNull();
});
it("binds the chosen path, numeric increment and prerelease policy", async () => {
  const request = setup();
  fireEvent.change(
    screen.getByLabelText("Version file relative to this repository"),
    { target: { value: "release/version" } },
  );
  fireEvent.change(screen.getByLabelText("Version increment"), {
    target: { value: "minor" },
  });
  fireEvent.change(screen.getByLabelText("Prerelease input policy"), {
    target: { value: "discard_and_increment" },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Read version update preview" }),
  );
  await waitFor(() => expect(request).toHaveBeenCalledOnce());
  expect(request.mock.calls[0][0]).toMatchObject({
    path: { encoding: "utf8", value: "release/version" },
    increment: "minor",
    prerelease: "discard_and_increment",
  });
});
it("drops a result after its inputs change", async () => {
  setup();
  fireEvent.click(
    screen.getByRole("button", { name: "Read version update preview" }),
  );
  await screen.findByRole("region", { name: "Version update preview" });
  fireEvent.change(screen.getByLabelText("Version increment"), {
    target: { value: "major" },
  });
  expect(
    screen.queryByRole("region", { name: "Version update preview" }),
  ).toBeNull();
});
it("does not display late results for superseded input", async () => {
  let resolve!: (value: Awaited<ReturnType<Bridge["request"]>>) => void;
  const request = vi.fn<Bridge["request"]>().mockImplementation(
    () =>
      new Promise((r) => {
        resolve = r;
      }),
  );
  render(
    <VersionAction
      bridge={bridge(request)}
      workspace="ws"
      worktree="wt"
      busy={false}
      act={(f) => f()}
    />,
  );
  fireEvent.click(
    screen.getByRole("button", { name: "Read version update preview" }),
  );
  fireEvent.change(
    screen.getByLabelText("Version file relative to this repository"),
    { target: { value: "different" } },
  );
  resolve({ kind: "version_preview", value: result });
  await waitFor(() => expect(request).toHaveBeenCalledOnce());
  expect(
    screen.queryByRole("region", { name: "Version update preview" }),
  ).toBeNull();
});
it("cannot issue previews while the host is busy", () => {
  const request = setup(true);
  fireEvent.click(
    screen.getByRole("button", { name: "Read version update preview" }),
  );
  expect(request).not.toHaveBeenCalled();
});
