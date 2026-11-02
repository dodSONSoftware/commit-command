import { useState } from "react";
import { value, type Bridge } from "../services/bridge";
import type { Inspection } from "../services/response.generated";
import { bytes, pathText } from "../stores/model";
const text = (value: string) =>
  bytes(Array.from(new TextEncoder().encode(value)));
export function RemoteInspector({
  bridge,
  workspace,
  worktree,
  busy,
  act,
}: {
  bridge: Bridge;
  workspace: string;
  worktree: string;
  busy: boolean;
  act: (action: () => Promise<void>) => Promise<void>;
}) {
  const [report, setReport] = useState<Inspection | null>(null);
  return (
    <details className="remote-inspector">
      <summary>Remotes and upstream</summary>
      <p>
        Local configuration inspection only. No server is contacted and no
        credential helper or transport command is executed. Cached tracking
        references do not establish the current server state.
      </p>
      <button
        disabled={busy}
        onClick={() =>
          void act(async () => {
            setReport(
              value(
                await bridge.request({ kind: "remotes", workspace, worktree }),
                "remotes",
              ),
            );
          })
        }
      >
        Inspect remote destinations
      </button>
      {report && (
        <>
          <p>
            Inspected{" "}
            <time>{new Date(report.observed_at_ms).toISOString()}</time> ·{" "}
            {report.branch ? text(report.branch) : "Detached HEAD"}
          </p>
          <p>
            Configured upstream:{" "}
            {report.upstream_remote && report.upstream_ref
              ? `${text(report.upstream_remote)} · ${text(report.upstream_ref)}`
              : "None or ambiguous"}
          </p>
          <p>
            Suggested push remote:{" "}
            {report.suggested_push_remote
              ? text(report.suggested_push_remote)
              : "None"}
            . Destination selection and authorization remain explicit.
          </p>
          {report.issues.map((issue, i) => (
            <p role="status" key={i}>
              {text(issue)}
            </p>
          ))}
          {(report.credential_helper_configured ||
            report.ssh_command_configured ||
            report.proxy_configured ||
            report.tls_verification_disabled) && (
            <p role="status">
              Credential helper, SSH command, proxy or TLS overrides are
              configured. Inspection does not authorize them for network
              operations.
            </p>
          )}
          {report.remotes.length === 0 && <p>No configured remotes.</p>}
          {report.remotes.map((remote) => (
            <article className="bulk-result" key={remote.name}>
              <h3>{text(remote.name)}</h3>
              <dl>
                {(["fetch", "push"] as const).map((direction) => (
                  <div key={direction}>
                    <dt>
                      {direction === "fetch"
                        ? "Fetch destinations"
                        : "Push destinations"}
                    </dt>
                    {remote[direction].map((destination, i) => (
                      <dd key={i}>
                        <code>{text(destination.display)}</code> ·{" "}
                        {destination.transport}
                        {destination.credentials_hidden && (
                          <p>Embedded login or sensitive URL fields hidden.</p>
                        )}
                        {destination.restrictions.map((restriction, j) => (
                          <p role="status" key={j}>
                            {text(restriction)}
                          </p>
                        ))}
                      </dd>
                    ))}
                  </div>
                ))}
              </dl>
              <p>
                {remote.fetch_refspec_count} configured fetch refspecs ·{" "}
                {remote.push_refspec_count} configured push refspecs
              </p>
              {remote.issues.map((issue, i) => (
                <p role="status" key={i}>
                  {text(issue)}
                </p>
              ))}
              <details>
                <summary>Configuration origins · {text(remote.name)}</summary>
                <ul>
                  {remote.origins.map((origin, i) => (
                    <li key={i}>
                      {origin.scope} ·{" "}
                      {origin.file
                        ? pathText(origin.file)
                        : "Command configuration"}
                    </li>
                  ))}
                </ul>
              </details>
            </article>
          ))}
        </>
      )}
    </details>
  );
}
