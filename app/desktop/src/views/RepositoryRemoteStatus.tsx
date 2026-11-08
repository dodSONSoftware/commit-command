import type { Snapshot } from "../services/response.generated";

export function RepositoryRemoteStatus({
  snapshot,
}: {
  snapshot?: Snapshot | null;
}) {
  const remotes = snapshot?.remotes;
  const tracking = snapshot?.tracking;
  const comparable = tracking?.ahead != null && tracking.behind != null;
  const ahead = tracking?.ahead ?? 0;
  const behind = tracking?.behind ?? 0;
  const noRemotes = remotes?.length === 0;
  const known = remotes != null;
  const label = noRemotes
    ? "○ No remote"
    : !known
      ? "? Remote status unknown"
      : !tracking
        ? "◇ No upstream"
        : !comparable
          ? "? Comparison unavailable"
          : ahead > 0 && behind > 0
            ? `↕ Diverged · ↑ ${ahead} ↓ ${behind}`
            : behind > 0
              ? `↓ Behind ${behind}`
              : ahead > 0
                ? `↑ Ahead ${ahead}`
                : "✓ Up to date";
  const tone =
    noRemotes || !known || !tracking || !comparable
      ? ""
      : behind > 0
        ? "warning"
        : ahead > 0
          ? "modified"
          : "success";
  return (
    <div className="remote-status">
      <span className={`badge ${tone}`}>{label}</span>
      {tracking && !noRemotes && (
        <small className="table-path">Compared with local tracking ref</small>
      )}
      <details>
        <summary>Remote details</summary>
        <p>
          {known
            ? noRemotes
              ? "No remotes configured."
              : `Configured remotes: ${remotes.join(", ")}`
            : "Remote configuration has not been observed. Refresh locally to inspect it."}
        </p>
        {snapshot && (
          <>
            <p>
              Branch:{" "}
              {snapshot.detached
                ? "Detached HEAD"
                : (snapshot.branch ?? "Unborn branch")}
            </p>
            <p>Upstream: {tracking?.reference ?? "Not configured"}</p>
            {tracking && (
              <p>
                {comparable
                  ? `${ahead} local-only commits; ${behind} upstream-only commits.`
                  : "Commit counts unavailable. The upstream ref may be missing locally."}
              </p>
            )}
            <p>
              Locally inspected{" "}
              {new Date(snapshot.observed_at_ms).toLocaleString()}.
            </p>
          </>
        )}
        <p>
          Counts use locally recorded refs; the server may have changed. Local
          refresh does not contact remotes. Open the repository’s Remotes and
          upstream panel for configuration and separately reviewed remote
          operations.
        </p>
      </details>
    </div>
  );
}
