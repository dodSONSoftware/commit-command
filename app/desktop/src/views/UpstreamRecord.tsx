import type { UpstreamOperation } from "../services/response.generated";
import { bytes, pathText } from "../stores/model";
const text = (value: string) =>
  bytes(Array.from(new TextEncoder().encode(value)));
const unresolved = [
  "lock_pending",
  "lock_created",
  "lock_ready",
  "publication_pending",
  "config_published",
  "uncertain",
  "interrupted",
];
export function UpstreamRecord({
  item,
  busy = false,
  onRecover,
}: {
  item: UpstreamOperation;
  busy?: boolean;
  onRecover?: (id: string) => void;
}) {
  return (
    <article className="bulk-result">
      <h3>
        {item.phase.replaceAll("_", " ")} · {text(item.plan.source_ref)}
      </h3>
      <p>
        {pathText(item.plan.metadata.root)} · <code>{item.plan.id}</code>
      </p>
      <p>
        Upstream: {text(item.plan.remote)} · {text(item.plan.reference)} ·{" "}
        {text(item.plan.destination)}
      </p>
      <p>
        Configuration result: {item.configuration.replaceAll("_", " ")}. Shared
        by linked worktrees.
      </p>
      {item.phase === "observed_configuration" && (
        <p>
          Recovery observed the approved configuration; it does not attribute
          uncertain publication or renew trust.
        </p>
      )}
      {(item.configuration === "updated" ||
        item.configuration === "matches_target") && (
        <p>
          Configuration changes require a fresh trust review before further
          controlled writes.
        </p>
      )}
      {item.issue && (
        <p role="alert">
          {item.issue.kind}: {text(item.issue.message)}
        </p>
      )}
      {onRecover && unresolved.includes(item.phase) && (
        <button disabled={busy} onClick={() => onRecover(item.plan.id)}>
          Inspect upstream recovery
        </button>
      )}
    </article>
  );
}
