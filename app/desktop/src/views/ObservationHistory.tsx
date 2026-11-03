import { ReviewedHttpsAuthentication } from "./ReviewedHttpsAuthentication";
import type { ObservationPage } from "../services/response.generated";
import { bytes, pathText } from "../stores/model";
const text = (value: string) =>
  bytes(Array.from(new TextEncoder().encode(value)));

export function ObservationRecords({
  items,
  onFetch,
  fetchDisabled = true,
  onPreview,
  previewDisabled = false,
  onUpstream,
  upstreamDisabled = true,
}: {
  items: ObservationPage["items"];
  onFetch?: (observation: string) => void;
  fetchDisabled?: boolean;
  onPreview?: (observation: string) => void;
  previewDisabled?: boolean;
  onUpstream?: (observation: string) => void;
  upstreamDisabled?: boolean;
}) {
  return (
    <>
      {items.map((observation) => (
        <article className="bulk-result" key={observation.plan.id}>
          <h3>
            {observation.phase} · {text(observation.plan.remote)} ·{" "}
            {text(observation.plan.reference)}
          </h3>
          <ReviewedHttpsAuthentication
            scope={observation.plan.authentication}
          />
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
          {onPreview && observation.phase === "complete" && observation.oid && (
            <button
              disabled={previewDisabled}
              onClick={() => onPreview(observation.plan.id)}
            >
              Inspect local push preview
            </button>
          )}
          {onUpstream &&
            observation.phase === "complete" &&
            observation.oid && (
              <button
                disabled={upstreamDisabled}
                onClick={() => onUpstream(observation.plan.id)}
              >
                Review upstream setup
              </button>
            )}
          {onFetch && observation.phase === "complete" && observation.oid && (
            <button
              disabled={fetchDisabled}
              onClick={() => onFetch(observation.plan.id)}
            >
              Review branch download
            </button>
          )}
        </article>
      ))}
    </>
  );
}
