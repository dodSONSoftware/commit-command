import type { StoredWorktree, Workspace } from "./services/response.generated";
export const fixtureWorkspace: Workspace = {
  id: "ws",
  name: "Fixture workspace",
  root: { encoding: "utf8", value: "/tmp/project" },
  display_root: { encoding: "utf8", value: "/tmp/project" },
  discovery: { excluded_names: ["node_modules"], excluded_patterns: [] },
};
export function fixtureEntry(id: string, path: string): StoredWorktree {
  const root = { encoding: "utf8" as const, value: `/tmp/project/${path}` };
  return {
    id,
    repository_id: `repository-${id}`,
    display_path: root,
    availability: "available",
    issue: null,
    untrusted: true,
    metadata: {
      root,
      git_dir: { encoding: "utf8", value: `${root.value}/.git` },
      common_dir: { encoding: "utf8", value: `${root.value}/.git` },
      root_binding: id,
      git_binding: id,
      common_binding: id,
      object_format: "sha1",
    },
    status: {
      head: "1".repeat(40),
      head_oid: { hex: "1".repeat(40), format: "sha1" },
      branch: "refs/heads/main",
      unborn: false,
      detached: false,
      counts: { staged: 1, unstaged: 1, untracked: 0, conflicted: 0 },
      changes: [
        {
          path: { encoding: "utf8", value: "file.txt" },
          original_path: null,
          index: "M",
          worktree: "M",
          conflict: false,
          submodule: "N...",
          head_mode: "100644",
          index_mode: "100644",
          worktree_mode: "100644",
        },
      ],
      tracking: null,
      operations: [],
      last_commit: null,
      status: "MM file.txt",
      generation: "generation",
      observed_at_ms: 1,
    },
  };
}
