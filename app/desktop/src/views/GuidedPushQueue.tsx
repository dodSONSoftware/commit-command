import { useOperationReview } from "./OperationReview";
import { useCallback, useEffect, useState } from "react";
import { value, type Bridge } from "../services/bridge";
import type {
  ChildReachabilitySelection,
  PlannedPush,
  PushBatch,
} from "../services/response.generated";
export function GuidedPushQueue({
  bridge,
  workspace,
  batch,
  sources,
  bindings,
  enabled,
  busy,
  act,
  revision,
  onChange,
  label,
}: {
  bridge: Bridge;
  workspace: string;
  batch: PushBatch;
  sources: PlannedPush[];
  bindings: Record<string, ChildReachabilitySelection[]>;
  enabled: boolean;
  busy: boolean;
  act: (f: () => Promise<void>) => Promise<void>;
  revision: number;
  onChange: (value: PushBatch) => void;
  label: (id: string) => string;
}) {
  const { review, reviewDialog } = useOperationReview(bridge, batch.plan.id);
  const [error, setError] = useState("");
  const id = batch.plan.id;
  const refresh = useCallback(async () => {
    const next = value(
      await bridge.request({ kind: "push_batch", workspace, id }),
      "push_batch",
    );
    onChange(next);
  }, [bridge, workspace, id, onChange]);
  useEffect(() => {
    let live = true;
    void bridge
      .request({ kind: "push_batch", workspace, id })
      .then((data) => {
        if (live) {
          onChange(value(data, "push_batch"));
          setError("");
        }
      })
      .catch((error) => {
        if (live) setError(String(error));
      });
    return () => {
      live = false;
    };
  }, [bridge, workspace, id, revision, onChange]);
  const next = batch.items.find(
    (i) => !["complete", "skipped_observed_current"].includes(i.phase),
  );
  const source = sources.find((s) => s.worktree === next?.worktree);
  return (
    <section aria-label="Guided push queue">
      <h3>Guided push queue · {batch.phase.replaceAll("_", " ")}</h3>
      {reviewDialog}
      <p>
        This saved order grants no publication authority. Each item requires a
        fresh review and an explicit Push. Publication failure, cancellation or
        uncertainty stops the queue; successes are preserved. Restart never
        resumes it.
      </p>
      <ol>
        {batch.items.map((item) => (
          <li key={item.worktree}>
            {label(item.worktree)} ·{" "}
            {item.phase === "skipped_observed_current"
              ? "Skipped: source matched the recorded observation; no push"
              : item.phase.replaceAll("_", " ")}
            {item.publication && (
              <>
                {" "}
                · <code>{item.publication}</code>
              </>
            )}
          </li>
        ))}
      </ol>
      {batch.issue && <p role="alert">{batch.issue.message}</p>}
      {error && <p role="alert">{error}</p>}
      {batch.phase === "ready" && next && (
        <>
          <p>
            Next: {label(next.worktree)}. Select and inspect this source above.
            After a selected child publishes, separately observe its destination
            again and save the fresh child evidence before reviewing its parent.
            Hook policy is frozen when the queue is created.
          </p>
          <button
            disabled={busy || !enabled || next.phase !== "pending" || !source}
            onClick={() =>
              void act(async () => {
                if (!source) return;
                await review({
                  kind: "review_push_batch_item",
                  workspace,
                  batch: id,
                  selection: {
                    worktree: source.worktree,
                    observation: source.preview.observation_id,
                    source_oid: source.preview.source_oid,
                    children: bindings[source.worktree] ?? [],
                  },
                });
                await refresh();
              })
            }
          >
            Review next guided push for {label(next.worktree)}
          </button>
        </>
      )}
      {["stopped", "interrupted"].includes(batch.phase) && (
        <p>
          This queue cannot continue. Inspect individual outcomes, resolve
          uncertain publication through fresh observation, then create a new
          plan from explicitly selected unfinished sources.
        </p>
      )}
      <button disabled={busy} onClick={() => void act(refresh)}>
        Refresh guided push result
      </button>
    </section>
  );
}
