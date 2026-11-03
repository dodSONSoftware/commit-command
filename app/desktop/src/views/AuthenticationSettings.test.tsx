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
  picker?: Bridge["pickAuthenticationFile"],
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
    pickAuthenticationFile: picker,
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

it("local suggestions require an explicit choice and never select an account or remote", async () => {
  const request = setup();
  await screen.findByText("No saved authentication profiles.");
  request.mockImplementation(async (r) =>
    r.kind === "authentication_choices"
      ? {
          kind: "authentication_choices",
          value: {
            helpers: [{ label: "macOS Keychain", path: "/tools/keychain" }],
            public_identities: [],
            known_hosts: null,
            agent_socket: null,
            notes: ["No credentials queried"],
          },
        }
      : {
          kind: "authentication_settings",
          value: { revision: "one", profiles: [], assignments: [] },
        },
  );
  fireEvent.click(
    screen.getByRole("button", { name: "Find local authentication tools" }),
  );
  await screen.findByText("No credentials queried");
  expect(screen.getByLabelText("Credential helper executable")).toHaveValue("");
  fireEvent.change(screen.getByLabelText("Discovered credential helper"), {
    target: { value: "/tools/keychain" },
  });
  expect(screen.getByLabelText("Credential helper executable")).toHaveValue(
    "/tools/keychain",
  );
  expect(screen.getByLabelText("Nonsecret account")).toHaveValue("");
  expect(
    screen.getByRole("button", { name: "Review remote assignment" }),
  ).toBeDisabled();
  expect(request.mock.calls.map(([r]) => r.kind)).toEqual([
    "authentication_settings",
    "authentication_choices",
  ]);
});
it("native file selection changes only the draft and cancellation preserves its current path", async () => {
  const picker = vi
    .fn<NonNullable<Bridge["pickAuthenticationFile"]>>()
    .mockResolvedValueOnce("/tools/helper's file")
    .mockResolvedValueOnce(null);
  const request = setup(
    true,
    { revision: "one", profiles: [], assignments: [] },
    picker,
  );
  await screen.findByText("No saved authentication profiles.");
  const browse = screen.getByRole("button", {
    name: "Browse credential helper executable…",
  });
  fireEvent.click(browse);
  await waitFor(() =>
    expect(screen.getByLabelText("Credential helper executable")).toHaveValue(
      "/tools/helper's file",
    ),
  );
  fireEvent.click(browse);
  await waitFor(() => expect(picker).toHaveBeenCalledTimes(2));
  expect(screen.getByLabelText("Credential helper executable")).toHaveValue(
    "/tools/helper's file",
  );
  expect(picker).toHaveBeenCalledWith("helper");
  expect(request).toHaveBeenCalledTimes(1);
});
