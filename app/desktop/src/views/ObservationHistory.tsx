import { useCallback, useEffect, useRef, useState } from "react";
import { value, type Bridge } from "../services/bridge";
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

export function WorkspaceObservationHistory({
  bridge,
  workspace,
  busy,
  act,
  revision,
}: {
  bridge: Bridge;
  workspace: string;
  busy: boolean;
  act: (action: () => Promise<void>) => Promise<void>;
  revision: number;
}) {
  const [page, setPage] = useState<ObservationPage | null>(null);
  const requestSequence = useRef(0);
  const load = useCallback(
    async (offset = 0) => {
      const sequence = ++requestSequence.current;
      const data = value(
        await bridge.request({
          kind: "remote_observations",
          workspace,
          worktree: null,
          offset,
        }),
        "remote_observations",
      );
      if (sequence === requestSequence.current)
        setPage((previous) =>
          offset && previous
            ? { items: [...previous.items, ...data.items], next: data.next }
            : data,
        );
    },
    [bridge, workspace],
  );
  useEffect(() => {
    const sequence = requestSequence;
    void act(() => load());
    return () => {
      ++sequence.current;
    };
  }, [act, load, revision]);
  return (
    <section aria-label="Workspace remote observation history">
      <h2>Remote observations</h2>
      <p>
        Recorded network reads across this workspace. Local operation filters
        above do not filter these records. Reading history contacts no server
        and cannot repeat an operation. Observed references describe past server
        state; they do not establish synchronization.
      </p>
      <button disabled={busy} onClick={() => void act(() => load())}>
        Refresh remote observation history
      </button>
      {page?.items.length === 0 && (
        <p>No recorded remote observations in this workspace.</p>
      )}
      <ObservationRecords items={page?.items ?? []} />
      {page?.next != null && (
        <button
          disabled={busy}
          onClick={() => void act(() => load(page.next!))}
        >
          Load older workspace observations
        </button>
      )}
    </section>
  );
}
