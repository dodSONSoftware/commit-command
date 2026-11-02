import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { Bridge } from "../services/bridge";
import { RemoteInspector } from "./RemoteInspector";
afterEach(cleanup);
it("only inspects scoped local configuration on demand and renders destinations as inert text", async () => {
  const request = vi.fn<Bridge["request"]>().mockResolvedValue({
    kind: "remotes",
    value: {
      branch: "refs/heads/main",
      config_digest: "d",
      credential_helper_configured: true,
      generation: "g",
      issues: ["No single configured upstream"],
      observed_at_ms: 0,
      proxy_configured: false,
      ssh_command_configured: false,
      tls_verification_disabled: false,
      remotes: [
        {
          name: "origin",
          fetch: [
            {
              display: "https://example.invalid/<img src=x>",
              transport: "https",
              credentials_hidden: true,
              restrictions: ["Embedded fields hidden"],
            },
          ],
          push: [],
          origins: [
            {
              scope: "local",
              file: { encoding: "utf8", value: "/tmp/repo/.git/config" },
            },
          ],
          fetch_refspec_count: 1,
          push_refspec_count: 0,
          issues: ["A mutation requires one unambiguous destination"],
        },
      ],
    },
  });
  const bridge: Bridge = {
    epoch: 1,
    request,
    capabilities: async () => ({ writes_enabled: false }),
    writeStatus: async () => false,
    pick: async () => null,
    cancel: async () => {},
  };
  const { container } = render(
    <RemoteInspector
      bridge={bridge}
      workspace="ws"
      worktree="repo"
      busy={false}
      act={async (action) => action()}
    />,
  );
  expect(request).not.toHaveBeenCalled();
  fireEvent.click(screen.getByText("Remotes and upstream"));
  fireEvent.click(
    screen.getByRole("button", { name: "Inspect remote destinations" }),
  );
  await screen.findByText("https://example.invalid/<img src=x>");
  expect(request).toHaveBeenCalledExactlyOnceWith({
    kind: "remotes",
    workspace: "ws",
    worktree: "repo",
  });
  expect(container.querySelector("img")).toBeNull();
  expect(screen.getByText(/does not authorize/)).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: /^(Push|Fetch)/ })).toBeNull();
});

it("requires an explicit remote and full branch before preparing a native network review", async () => {
  const request = vi.fn<Bridge["request"]>().mockImplementation(async (r) => {
    if (r.kind === "remotes")
      return {
        kind: "remotes",
        value: {
          branch: "refs/heads/main",
          config_digest: "d",
          credential_helper_configured: false,
          generation: "g",
          issues: [],
          observed_at_ms: 0,
          proxy_configured: false,
          ssh_command_configured: false,
          tls_verification_disabled: false,
          remotes: [
            {
              name: "origin",
              fetch: [],
              push: [],
              origins: [],
              fetch_refspec_count: 1,
              push_refspec_count: 0,
              issues: [],
            },
          ],
        },
      };
    if (r.kind === "remote_observations")
      return { kind: "remote_observations", value: { items: [], next: null } };
    return {
      kind: "review",
      value: {
        id: "review",
        epoch: 1,
        expires_at_ms: Date.now() + 600000,
        title: "Review observation",
        summary: "",
        content: "",
      },
    };
  });
  const bridge: Bridge = {
    epoch: 1,
    request,
    capabilities: async () => ({ writes_enabled: true }),
    writeStatus: async () => false,
    pick: async () => null,
    cancel: async () => {},
  };
  render(
    <RemoteInspector
      bridge={bridge}
      workspace="ws"
      worktree="repo"
      busy={false}
      enabled
      act={async (action) => action()}
    />,
  );
  fireEvent.click(screen.getByText("Remotes and upstream"));
  const prepare = screen.getByRole("button", {
    name: "Review remote observation",
  });
  expect(prepare).toBeDisabled();
  fireEvent.click(
    screen.getByRole("button", { name: "Inspect remote destinations" }),
  );
  await screen.findByText(/Suggested push remote:/);
  fireEvent.change(screen.getByLabelText("Remote to observe"), {
    target: { value: "origin" },
  });
  expect(prepare).toBeDisabled();
  fireEvent.change(screen.getByLabelText("Full remote branch reference"), {
    target: { value: "refs/heads/release" },
  });
  fireEvent.click(prepare);
  await screen.findByText("No recorded observations for this repository.");
  expect(request).toHaveBeenCalledWith({
    kind: "review_remote_observation",
    workspace: "ws",
    worktree: "repo",
    remote: "origin",
    reference: "refs/heads/release",
  });
  expect(request).toHaveBeenCalledWith({
    kind: "remote_observations",
    workspace: "ws",
    worktree: "repo",
    offset: 0,
  });
  expect(
    request.mock.calls.every(([r]) =>
      ["remotes", "review_remote_observation", "remote_observations"].includes(
        r.kind,
      ),
    ),
  ).toBe(true);
});
