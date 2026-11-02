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
