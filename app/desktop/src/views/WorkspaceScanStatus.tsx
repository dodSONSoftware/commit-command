import type { ScanSummary } from "../services/response.generated";
import { pathText } from "../stores/model";

export function WorkspaceScanStatus({
  scan,
  onContinue,
  busy = false,
}: {
  scan?: ScanSummary | null;
  onContinue?: () => void;
  busy?: boolean;
}) {
  return (
    <section className="workspace-scan" aria-label="Workspace discovery status">
      <p role="status">
        <span
          className={`badge ${!scan || !scan.complete ? "warning" : "success"}`}
        >
          {!scan
            ? "? Discovery not yet scanned"
            : scan.complete
              ? "✓ Discovery complete"
              : "△ Discovery incomplete"}
        </span>
        {scan &&
          ` · ${scan.issues.length} scan issue${scan.issues.length === 1 ? "" : "s"} · ${scan.skipped_caches.length} tagged cache${scan.skipped_caches.length === 1 ? "" : "s"} skipped`}
      </p>
      {scan?.progress && (
        <div>
          <p role="status">
            {scan.progress.inspected_repositories} repositories inspected ·{" "}
            {scan.progress.pending_directories} directories pending ·{" "}
            {scan.progress.phase}
          </p>
          <button disabled={busy || !onContinue} onClick={onContinue}>
            Continue discovery
          </button>
        </div>
      )}
      {!scan ? (
        <p>Run Rescan to inspect the workspace.</p>
      ) : (
        <details>
          <summary>Discovery details</summary>
          <p>
            Last recorded discovery scan, within workspace exclusions.
            Repository attention counts do not include scan-level issues. Use
            Rescan to retry; no Git files are changed.
          </p>
          {!scan.complete && (
            <p>
              Some locations could not be inspected. Known repositories can
              remain available while discovery is incomplete.
            </p>
          )}
          {scan.issues.length > 0 && (
            <ul>
              {scan.issues.map((issue, index) => (
                <li key={index}>
                  <strong>{pathText(issue.path)}</strong> — {issue.error.kind}:{" "}
                  {issue.error.message}
                </li>
              ))}
            </ul>
          )}
          {scan.skipped_caches.length > 0 && (
            <>
              <p>
                These subtrees carry a valid CACHEDIR.TAG signature and were
                skipped as generated caches. Open a cache directory as its own
                workspace to inspect it explicitly.
              </p>
              <ul>
                {scan.skipped_caches.map((path, index) => (
                  <li key={index}>{pathText(path)}</li>
                ))}
              </ul>
            </>
          )}
        </details>
      )}
    </section>
  );
}
