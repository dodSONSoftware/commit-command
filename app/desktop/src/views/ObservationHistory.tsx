import type { ObservationPage } from "../services/response.generated";
import { bytes, pathText } from "../stores/model";
const text = (value: string) =>
  bytes(Array.from(new TextEncoder().encode(value)));

export function ObservationRecords({
  items,
}: {
  items: ObservationPage["items"];
}) {
  return (
    <>
      {items.map((observation) => (
        <article className="bulk-result" key={observation.plan.id}>
          <h3>
            {observation.phase} · {text(observation.plan.remote)} ·{" "}
            {text(observation.plan.reference)}
          </h3>
          <p>
            <code>{observation.plan.id}</code> ·{" "}
            {pathText(observation.plan.metadata.root)}
          </p>
          <p>
            Prepared{" "}
            <time>
              {new Date(observation.plan.created_at_ms).toISOString()}
            </time>
          </p>
          <p>{text(observation.plan.destination)}</p>
          {observation.phase === "complete" && (
            <p>
              Observed{" "}
              {observation.observed_at_ms != null
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
    </>
  );
}
