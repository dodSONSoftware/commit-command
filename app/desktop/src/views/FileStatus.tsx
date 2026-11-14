import type { Change } from "../services/response.generated";

const labels: Record<string, string> = {
  M: "Modified",
  A: "Added",
  D: "Deleted",
  R: "Renamed",
  C: "Copied",
  T: "Type changed",
  U: "Unmerged",
  "?": "Untracked",
  "!": "Ignored",
  ".": "Unchanged",
  DD: "Unmerged: both deleted",
  AU: "Unmerged: added by us",
  UD: "Unmerged: deleted by them",
  UA: "Unmerged: added by them",
  DU: "Unmerged: deleted by us",
  AA: "Unmerged: both added",
  UU: "Unmerged: both modified",
};

export function FileStatus({
  change,
  source,
}: {
  change: Pick<Change, "index" | "worktree" | "conflict">;
  source: "index" | "working_tree";
}) {
  const code = change.conflict
    ? change.index + change.worktree
    : change.index === "?"
      ? "?"
      : source === "index"
        ? change.index
        : change.worktree;
  const label = labels[code] ?? "Unknown status";
  return (
    <span
      className={`file-status ${change.conflict ? "bad" : ""}`}
      role="img"
      aria-label={label}
      title={label}
    >
      {code}
    </span>
  );
}
