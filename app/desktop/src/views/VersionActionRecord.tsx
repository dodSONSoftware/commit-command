import type { Bridge } from "../services/bridge";
import type { VersionInvocation } from "../services/response.generated";
import { pathText } from "../stores/model";
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
  return (
    <section aria-label="Version Action record">
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
