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

it("prepares a branch download only from a completed observation with an OID and keeps ordinary builds gated", async () => {
  const { fixtureEntry } = await import("../testdata");
  const plan = {
    id: "observed",
    workspace: "ws",
    worktree: "repo",
    metadata: fixtureEntry("repo", "repo").metadata,
    remote: "origin",
    reference: "refs/heads/main",
    destination: "https://example.invalid/repo.git",
    transport: "https" as const,
    config_digest: "digest",
    generation: "generation",
    created_at_ms: 0,
    tools: [],
    local_target: null,
    local_binding: null,
  };
  const oid = { format: "sha1" as const, hex: "a".repeat(40) };
  const request = vi.fn<Bridge["request"]>().mockImplementation(async (r) => {
    if (r.kind === "remote_observations")
      return {
        kind: "remote_observations",
        value: {
          items: [
            { plan, phase: "complete", oid, observed_at_ms: 0, issue: null },
            {
              plan: { ...plan, id: "absent" },
              phase: "complete",
              oid: null,
              observed_at_ms: 0,
              issue: null,
            },
          ],
          next: null,
        },
      };
    if (r.kind === "push_preview")
      return {
        kind: "push_preview",
        value: {
          source_ref: "refs/heads/main",
          source_oid: oid,
          remote: "origin",
          destination: plan.destination,
          destination_ref: "refs/heads/main",
          observed_oid: oid,
          observation_id: plan.id,
          observed_at_ms: 0,
          generated_at_ms: 0,
          generation: "generation",
          config_digest: "digest",
          relation: "up_to_date",
          outgoing_count: 0,
          incoming_count: 0,
          outgoing: [],
          outgoing_truncated: false,
          upstream_matches: false,
        },
      };
    if (r.kind === "fetch_captures")
      return {
        kind: "fetch_captures",
        value: {
          items: [
            {
              plan: {
                ...plan,
                id: "capture",
                expected_oid: oid,
                observation_id: "observed",
              },
              phase: "complete",
              artifact: {
                bundle_digest: "checksum",
                bundle_bytes: 100,
                object_count: 3,
                object_bytes: 200,
              },
              captured_at_ms: null,
              issue: null,
            },
          ],
          next: null,
        },
      };
    return { kind: "done" };
  });
  const bridge: Bridge = {
    epoch: 1,
    request,
    capabilities: async () => ({ writes_enabled: true }),
    writeStatus: async () => false,
    pick: async () => null,
    cancel: async () => {},
  };
  const props = {
    bridge,
    workspace: "ws",
    worktree: "repo",
    busy: false,
    act: async (action: () => Promise<void>) => action(),
  };
  const { rerender } = render(<RemoteInspector {...props} enabled />);
  fireEvent.click(screen.getByText("Remotes and upstream"));
  fireEvent.click(
    screen.getByRole("button", { name: "Refresh observation results" }),
  );
  const button = await screen.findByRole("button", {
    name: "Review branch download",
  });
  expect(
    screen.getAllByRole("button", { name: "Review branch download" }),
  ).toHaveLength(1);
  fireEvent.click(button);
  await screen.findByText("capture");
  expect(request).toHaveBeenCalledWith({
    kind: "review_fetch_capture",
    workspace: "ws",
    worktree: "repo",
    observation: "observed",
  });
  expect(
    screen.getByText(/Downloading alone does not update/),
  ).toBeInTheDocument();
  const importButton = screen.getByRole("button", {
    name: "Review captured branch import",
  });
  fireEvent.click(importButton);
  await vi.waitFor(() =>
    expect(request).toHaveBeenCalledWith({
      kind: "review_fetch_import",
      workspace: "ws",
      worktree: "repo",
      capture: "capture",
    }),
  );
  rerender(<RemoteInspector {...props} enabled={false} />);
  expect(importButton).toBeDisabled();
  const previewButton = screen.getByRole("button", {
    name: "Inspect local push preview",
  });
  expect(previewButton).toBeEnabled();
  fireEvent.click(previewButton);
  await screen.findByRole("region", { name: "Local push comparison" });
  expect(request).toHaveBeenCalledWith({
    kind: "push_preview",
    workspace: "ws",
    worktree: "repo",
    observation: "observed",
  });
  rerender(<RemoteInspector {...props} worktree="other" enabled={false} />);
  expect(
    screen.queryByRole("region", { name: "Local push comparison" }),
  ).toBeNull();

  expect(
    screen.getByRole("button", { name: "Review branch download" }),
  ).toBeDisabled();
  expect(
    request.mock.calls.every(([r]) =>
      [
        "remote_observations",
        "review_fetch_capture",
        "review_fetch_import",
        "push_preview",
        "fetch_captures",
      ].includes(r.kind),
    ),
  ).toBe(true);
});
