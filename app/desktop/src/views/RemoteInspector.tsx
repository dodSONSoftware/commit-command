import { useState } from "react";
import { value, type Bridge } from "../services/bridge";
import type {
  Inspection,
  ObservationPage,
} from "../services/response.generated";
import { bytes, pathText } from "../stores/model";
const text = (value: string) =>
  bytes(Array.from(new TextEncoder().encode(value)));
export function RemoteInspector({
  bridge,
  workspace,
  worktree,
  busy,
  act,
  enabled = false,
}: {
  bridge: Bridge;
  workspace: string;
  worktree: string;
  busy: boolean;
  act: (action: () => Promise<void>) => Promise<void>;
  enabled?: boolean;
}) {
  const [report, setReport] = useState<Inspection | null>(null);
  const [remote, setRemote] = useState("");
  const [reference, setReference] = useState("");
  const [observations, setObservations] = useState<ObservationPage | null>(
    null,
  );
  async function refresh(offset = 0) {
    const page = value(
      await bridge.request({
        kind: "remote_observations",
        workspace,
        worktree,
        offset,
      }),
      "remote_observations",
    );
    setObservations((previous) =>
      offset && previous
        ? { items: [...previous.items, ...page.items], next: page.next }
        : page,
    );
  }
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
      <h3>Observe one remote branch</h3>
      <p>
        A separate native review authorizes a network read. Observation does not
        fetch objects or update local tracking refs. This initial transport uses
        no credentials; private-server authentication remains unavailable.
      </p>
      <form
        className="activity-filters"
        onSubmit={(event) => {
          event.preventDefault();
          if (
            !enabled ||
            busy ||
            !remote ||
            !reference.startsWith("refs/heads/")
          )
            return;
          void act(async () => {
            await bridge.request({
              kind: "review_remote_observation",
              workspace,
              worktree,
              remote,
              reference,
            });
            await refresh();
          });
        }}
      >
        <label>
          Remote to observe
          <select
            value={remote}
            onChange={(event) => setRemote(event.target.value)}
          >
            <option value="">Choose an inspected remote</option>
            {report?.remotes.map((item) => (
              <option key={item.name} value={item.name}>
                {text(item.name)}
              </option>
            ))}
          </select>
        </label>
        <label>
          Full remote branch reference
          <input
            maxLength={1024}
            value={reference}
            placeholder={report?.branch ?? "refs/heads/main"}
            onChange={(event) => setReference(event.target.value)}
          />
        </label>
        <button
          disabled={
            !enabled ||
            busy ||
            !report?.remotes.some((item) => item.name === remote) ||
            !reference.startsWith("refs/heads/")
          }
        >
          Review remote observation
        </button>
      </form>
      <button disabled={busy} onClick={() => void act(() => refresh())}>
        Refresh observation results
      </button>
      {observations?.items.length === 0 && (
        <p>No recorded observations for this repository.</p>
      )}
      {observations?.items.map((observation) => (
        <article className="bulk-result" key={observation.plan.id}>
          <h3>
            {observation.phase} · {text(observation.plan.remote)} ·{" "}
            {text(observation.plan.reference)}
          </h3>
          <p>{text(observation.plan.destination)}</p>
          {observation.phase === "complete" && (
            <p>
              Observed{" "}
              {observation.observed_at_ms
                ? new Date(observation.observed_at_ms).toISOString()
                : "unknown time"}{" "}
              · {observation.oid?.hex ?? "Reference absent"}. This is a past
              server observation.
            </p>
          )}
          {observation.issue && (
            <p role="alert">
              {observation.issue.kind}: {text(observation.issue.message)}
            </p>
          )}
        </article>
      ))}
      {observations?.next != null && (
        <button
          disabled={busy}
          onClick={() => void act(() => refresh(observations.next!))}
        >
          Load older observations
        </button>
      )}
    </details>
  );
}
