import { ReviewedHttpsAuthentication } from "./ReviewedHttpsAuthentication";
import type { FetchCapture } from "../services/response.generated";
import { bytes, pathText } from "../stores/model";
const text = (value: string) =>
  bytes(Array.from(new TextEncoder().encode(value)));
export function FetchCaptureRecords({
  items,
  onImport,
  importDisabled = false,
}: {
  items: FetchCapture[];
  onImport?: (capture: string) => void;
  importDisabled?: boolean;
}) {
  return (
    <>
      {items.map((capture) => (
        <article className="bulk-result" key={capture.plan.id}>
          <h3>
            {capture.phase} · {text(capture.plan.remote)} ·{" "}
            {text(capture.plan.reference)}
          </h3>
          <ReviewedHttpsAuthentication scope={capture.plan.authentication} />
          <p>
            <code>{capture.plan.id}</code> ·{" "}
            {pathText(capture.plan.metadata.root)}
          </p>
          <p>
            {text(capture.plan.destination)} · Expected commit:{" "}
            <code>{capture.plan.expected_oid.hex}</code>
          </p>
          {capture.artifact && (
            <>
              <p>
                Downloaded {capture.artifact.object_count} objects ·{" "}
                {capture.artifact.object_bytes} uncompressed bytes ·{" "}
                {capture.artifact.bundle_bytes} bundle bytes.
              </p>
              <p>
                Bundle checksum: <code>{capture.artifact.bundle_digest}</code>
              </p>
            </>
          )}
          {capture.captured_at_ms != null && (
            <p>
              Captured{" "}
              <time>{new Date(capture.captured_at_ms).toISOString()}</time>
            </p>
          )}
          <p>
            Downloading alone does not update the repository or tracking refs.
            Import requires a separate review; see its recorded result.
          </p>
          {onImport && capture.phase === "complete" && capture.artifact && (
            <button
              disabled={importDisabled}
              onClick={() => onImport(capture.plan.id)}
            >
              Review captured branch import
            </button>
          )}
          {capture.issue && (
            <p role="alert">
              {capture.issue.kind}: {text(capture.issue.message)}
            </p>
          )}
        </article>
      ))}
    </>
  );
}
