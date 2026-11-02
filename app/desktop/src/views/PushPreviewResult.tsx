import type { PushPreview } from "../services/response.generated";
import { bytes } from "../stores/model";
const text = (value: string) =>
  bytes(Array.from(new TextEncoder().encode(value)));
export function PushPreviewResult({ preview }: { preview: PushPreview }) {
  return (
    <section aria-label="Local push comparison">
      <h3>Local push preview · {preview.relation.replaceAll("_", " ")}</h3>
      <p role="status">
        Local comparison with a past server observation. No server was contacted
        by this preview. This is not push approval; actual push requires fresh
        server evidence and a separate exact review.
      </p>
      <p>
        {text(preview.source_ref)} · <code>{preview.source_oid.hex}</code>
      </p>
      <p>
        {text(preview.remote)} · {text(preview.destination)} ·{" "}
        {text(preview.destination_ref)}
      </p>
      <p>
        Observed destination commit: <code>{preview.observed_oid.hex}</code> at{" "}
        {new Date(preview.observed_at_ms).toISOString()}.
      </p>
      <p>
        Generated {new Date(preview.generated_at_ms).toISOString()} ·{" "}
        {preview.outgoing_count} outgoing · {preview.incoming_count} incoming
        commits.
      </p>
      <p>
        Configured upstream matches this selection:{" "}
        {String(preview.upstream_matches)}.
      </p>
      {(preview.relation === "behind" || preview.relation === "diverged") && (
        <p role="alert">
          The selected update is not a fast forward relative to the observed
          commit. Normal push is blocked; no force action is available.
        </p>
      )}
      <details>
        <summary>
          Outgoing commits · {preview.outgoing.length} of{" "}
          {preview.outgoing_count}
          {preview.outgoing_truncated ? " (display limited)" : ""}
        </summary>
        {preview.outgoing.map((commit) => (
          <p key={commit.oid.hex}>
            <code>{commit.oid.hex}</code> · {bytes(commit.subject)}
            {commit.subject_truncated ? " (subject truncated)" : ""}
          </p>
        ))}
      </details>
    </section>
  );
}
