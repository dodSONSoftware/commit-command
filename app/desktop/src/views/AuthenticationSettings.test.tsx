import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { AuthenticationSettings } from "./AuthenticationSettings";
import type { Bridge } from "../services/bridge";
import type { Settings } from "../services/response.generated";
afterEach(cleanup);
function setup(
  enabled = true,
  settings: Settings = { revision: "one", profiles: [], assignments: [] },
) {
  const request = vi.fn<Bridge["request"]>(async (r) =>
    r.kind === "authentication_settings"
      ? { kind: "authentication_settings", value: settings }
      : {
          kind: "review",
          value: {
            id: "r",
            epoch: 1,
            expires_at_ms: Date.now() + 600000,
            title: "Review",
            summary: "",
            content: "",
          },
        },
  );
  const bridge: Bridge = {
    epoch: 1,
    request,
    capabilities: async () => ({ writes_enabled: enabled }),
    writeStatus: async () => false,
    pick: async () => null,
    cancel: async () => {},
  };
  render(
    <AuthenticationSettings
      bridge={bridge}
      workspace="workspace"
      worktree="worktree"
      remotes={["origin"]}
      busy={false}
      enabled={enabled}
      act={async (action) => action()}
    />,
  );
  return request;
}
it("proposes only nonsecret profile fields, without network or credential requests", async () => {
  const request = setup();
  await screen.findByText("No saved authentication profiles.");
  fireEvent.change(screen.getByLabelText("Profile name"), {
    target: { value: "Gitea" },
  });
  fireEvent.change(screen.getByLabelText("Credential helper executable"), {
    target: { value: "/tools/helper" },
  });
  fireEvent.change(screen.getByLabelText("Nonsecret account"), {
    target: { value: "account" },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Review profile settings" }),
  );
  await waitFor(() =>
    expect(request).toHaveBeenCalledWith({
      kind: "review_authentication",
      change: {
        kind: "save",
        id: null,
        name: "Gitea",
        policy: {
          kind: "https",
          helper: "/tools/helper",
          account: "account",
          ca_file: null,
        },
      },
    }),
  );
  expect(request.mock.calls.map(([r]) => r.kind)).toEqual([
    "authentication_settings",
    "review_authentication",
  ]);
  expect(document.querySelector('input[type="password"]')).toBeNull();
});
it("requires explicit remote and profile selection and keeps assigned profiles from deletion", async () => {
  const request = setup(true, {
    revision: "one",
    profiles: [
      {
        id: "p",
        name: "<img src=x>",
        policy: {
          kind: "https",
          helper: "/tools/helper",
          account: "account",
          ca_file: null,
        },
        tools: [],
      },
    ],
    assignments: [
      {
        workspace: "workspace",
        worktree: "worktree",
        remote: "origin",
        destination: "https://example.invalid/repo.git",
        profile: "p",
      },
    ],
  });
  const assignment = screen.getByRole("button", {
    name: "Review remote assignment",
  });
  await screen.findByText(/Assigned origin/);
  expect(assignment).toBeDisabled();
  expect(
    screen.getByRole("button", { name: "Review delete <img src=x>" }),
  ).toBeDisabled();
  expect(document.querySelector("img")).toBeNull();
  fireEvent.change(screen.getByLabelText("Remote for authentication"), {
    target: { value: "origin" },
  });
  fireEvent.change(screen.getByLabelText("Authentication profile"), {
    target: { value: "p" },
  });
  fireEvent.click(assignment);
  await waitFor(() =>
    expect(request).toHaveBeenCalledWith({
      kind: "review_authentication",
      change: {
        kind: "assign",
        workspace: "workspace",
        worktree: "worktree",
        remote: "origin",
        profile: "p",
      },
    }),
  );
});
it("ordinary read-only builds can inspect settings without proposing changes", async () => {
  const request = setup(false);
  await screen.findByText("No saved authentication profiles.");
  expect(
    screen.getByRole("button", { name: "Review profile settings" }),
  ).toBeDisabled();
  expect(
    screen.getByRole("button", { name: "Review remote assignment" }),
  ).toBeDisabled();
  expect(request).toHaveBeenCalledTimes(1);
});
