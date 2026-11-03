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
      <section aria-label="Source child commit requirements">
        <h4>Child commits in source · {preview.source_gitlinks.length}</h4>
        <p>
          These gitlinks come from the exact source commit. Child checkouts and
          remote availability were not inspected. Parent push still requires
          separate proof of child reachability on the intended child remotes,
          including the outgoing-history requirements below.
        </p>
        {preview.source_gitlinks.map((link) => (
          <p key={JSON.stringify(link.path)}>
            {bytes(link.path)} · <code>{link.oid.hex}</code>
          </p>
        ))}
      </section>
      <details>
        <summary>
          Child requirements across outgoing history ·{" "}
          {preview.outgoing_gitlinks.length}
        </summary>
        <p>
          Unique path and child-commit pairs from every outgoing commit tree,
          including links later replaced or removed. These are local
          requirements; availability on intended child remotes is still
          unproven. The example parent commit records one occurrence, not every
          occurrence.
        </p>
        {preview.outgoing_gitlinks.map((link) => (
          <div key={JSON.stringify([link.path, link.oid.hex])}>
            <p>
              {bytes(link.path)} · <code>{link.oid.hex}</code> · example parent{" "}
              <code>{link.example_commit.hex}</code>
            </p>
            {link.mappings.map((mapping, i) => (
              <p key={i}>
                Historical mapping ·{" "}
                {mapping.gitmodules_blob?.hex ?? "missing .gitmodules"} ·{" "}
                {mapping.destination
                  ? text(mapping.destination.display)
                  : "No usable destination"}
                {mapping.issue
                  ? ` · ${text(mapping.issue)}`
                  : " · Requires explicit child selection and observation"}
                {mapping.declared_url &&
                  ` · Declared ${text(mapping.declared_url)}`}
                {mapping.resolution_base &&
                  ` · Relative to selected parent ${text(mapping.resolution_base)}`}
              </p>
            ))}
          </div>
        ))}
      </details>
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
