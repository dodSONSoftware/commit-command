import { useState } from "react";
import { value, type Bridge } from "../services/bridge";
import type {
  RetainedVersionFile,
  VersionInvocation,
} from "../services/response.generated";
import { bytes, pathText } from "../stores/model";
export function VersionActionRecord({
  item,
  bridge,
  workspace,
  enabled,
  busy,
  act,
}: {
  item: VersionInvocation;
  bridge: Bridge;
  workspace: string;
  enabled: boolean;
  busy: boolean;
  act: (f: () => Promise<void>) => Promise<void>;
}) {
  const [retained, setRetained] = useState<RetainedVersionFile | null>(null);
  return (
    <section
      className="version-action-record"
      aria-label="Version Action record"
    >
      <h3>Version Action · {item.phase.replaceAll("_", " ")}</h3>
      <p>
        {pathText(item.plan.path)} · {item.plan.version.old} →{" "}
        {item.plan.version.new}
      </p>
      <p>
        <code>{item.plan.id}</code>
      </p>
      <p>
        {item.applied
          ? "Exact file replacement completed; the previous file is retained in private recovery storage. Staging and committing require separate reviews."
          : "No completed replacement receipt. This record grants no mutation or retry authority."}
      </p>
      {item.observed && (
        <p>
          Observed file state: {item.observed.replaceAll("_", " ")}. Recovery
          records observed bytes; it never repeats an increment or overwrites an
          external edit.
        </p>
      )}
      {item.issue && <p role="alert">{item.issue.message}</p>}
      {item.definition_approved_at_ms != null && (
        <button
          disabled={busy}
          onClick={() =>
            void act(async () => {
              setRetained(null);
              setRetained(
                value(
                  await bridge.request({
                    kind: "version_retained_file",
                    workspace,
                    operation: item.plan.id,
                  }),
                  "version_retained_file",
                ),
              );
            })
          }
        >
          Inspect retained version file
        </button>
      )}
      {retained?.operation === item.plan.id && (
        <section aria-label="Retained version file inspection">
          <p>
            Read-only inspection at{" "}
            {new Date(retained.read_at_ms).toISOString()}. No file was restored,
            staged or changed.
          </p>
          <p>
            {retained.matches_recorded_before
              ? "Retained bytes match the recorded original."
              : "Retained bytes differ from the recorded original; inspect this candidate or external edit before manual reconciliation."}
          </p>
          <p>
            Location: <code>{pathText(retained.location)}</code>
          </p>
          <p>
            {retained.bytes.length} bytes · fingerprint{" "}
            <code>{retained.fingerprint}</code>
          </p>
          <pre>{bytes(retained.bytes)}</pre>
        </section>
      )}

      {item.phase === "unconfirmed" && (
        <button
          disabled={busy || !enabled}
          onClick={() =>
            void act(async () => {
              await bridge.request({
                kind: "review_version_recovery",
                workspace,
                operation: item.plan.id,
              });
            })
          }
        >
          Review observed version Action outcome
        </button>
      )}
      {["applied", "reconciled"].includes(item.phase) &&
        item.observed === "applied" && (
          <button
            disabled={busy || !enabled}
            onClick={() =>
              void act(async () => {
                await bridge.request({
                  kind: "review_version_stage",
                  workspace,
                  operation: item.plan.id,
                });
              })
            }
          >
            Review staging exact version result
          </button>
        )}
    </section>
  );
}
