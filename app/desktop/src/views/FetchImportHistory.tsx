import type { FetchImport } from "../services/response.generated";
import { bytes, pathText } from "../stores/model";
const text = (value: string) =>
  bytes(Array.from(new TextEncoder().encode(value)));
const unresolved = [
  "importing_objects",
  "publication_pending",
  "reference_prepared",
  "reference_updated",
  "uncertain",
  "interrupted",
];
export function FetchImportRecords({
  items,
  onRecover,
  busy = false,
}: {
  items: FetchImport[];
  onRecover?: (id: string) => void;
  busy?: boolean;
}) {
  return (
    <>
      {items.map((item) => (
        <article className="bulk-result" key={item.plan.id}>
          <h3>
            {item.phase.replaceAll("_", " ")} · {text(item.plan.tracking_ref)}
          </h3>
          <p>
            {pathText(item.plan.metadata.root)} · <code>{item.plan.id}</code>
          </p>
          <p>
            Reviewed old value:{" "}
            <code>{item.plan.expected_old?.hex ?? "absent"}</code> · Captured
            target: <code>{item.plan.expected_oid.hex}</code>
          </p>
          <p>
            Object import recorded: {String(item.objects_imported)} · Ref
            result: {item.reference_status.replaceAll("_", " ")}
          </p>
          <p>
            Bundle checksum: <code>{item.plan.bundle_digest}</code>
          </p>
          {item.phase === "observed_target" && (
            <p>
              The destination matched the target during recovery inspection.
              This observation does not attribute an uncertain publication.
            </p>
          )}
          {item.issue && (
            <p role="alert">
              {item.issue.kind}: {text(item.issue.message)}
            </p>
          )}
          {onRecover && unresolved.includes(item.phase) && (
            <button disabled={busy} onClick={() => onRecover(item.plan.id)}>
              Inspect captured branch recovery
            </button>
          )}
        </article>
      ))}
    </>
  );
}
