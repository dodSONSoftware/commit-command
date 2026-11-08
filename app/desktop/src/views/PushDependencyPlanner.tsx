import { useState } from "react";
import type { Bridge } from "../services/bridge";
import { value } from "../services/bridge";
import type {
  ChildReachability,
  ChildReachabilitySelection,
  Observation,
  PlannedPush,
  PushDependencyPlan,
  PushBatch,
  StoredWorktree,
} from "../services/response.generated";
import { bytes, pathText } from "../stores/model";
import { ChildReachabilityForm } from "./ChildReachabilityForm";
import { GuidedPushQueue } from "./GuidedPushQueue";
import { PushPreviewResult } from "./PushPreviewResult";
const bindingKey = (b: ChildReachabilitySelection) =>
  JSON.stringify([b.path, b.child_oid, b.gitmodules_blob]);
export function PushDependencyPlanner({
  bridge,
  workspace,
  entries,
  busy,
  act,
  enabled = false,
  activeBatch,
  onBatch,
  revision = 0,
}: {
  enabled?: boolean;
  activeBatch?: PushBatch | null;
  onBatch?: (batch: PushBatch) => void;
  revision?: number;
  bridge: Bridge;
  workspace: string;
  entries: StoredWorktree[];
  busy: boolean;
  act: (f: () => Promise<void>) => Promise<void>;
}) {
  const [localBatch, setLocalBatch] = useState<PushBatch | null>(null);
  const [savedQueues, setSavedQueues] = useState<PushBatch[]>([]);
  const [queueNext, setQueueNext] = useState<number | null>(null);
  const batch = activeBatch ?? localBatch;
  const setBatch = onBatch ?? setLocalBatch;
  const [selected, setSelected] = useState<string[]>([]);
  const [history, setHistory] = useState<Record<string, Observation[]>>({});
  const [observations, setObservations] = useState<Record<string, string>>({});
  const [sources, setSources] = useState<PlannedPush[]>([]);
  const [bindings, setBindings] = useState<
    Record<string, ChildReachabilitySelection[]>
  >({});
  const [hooks, setHooks] = useState<Record<string, "execute" | "bypass_all">>(
    {},
  );
  const [plan, setPlan] = useState<PushDependencyPlan | null>(null);
  const invalidate = () => {
    setSources([]);
    setBindings({});
    setPlan(null);
  };
  const label = (id: string) => {
    const entry = entries.find((e) => e.id === id);
    return entry ? pathText(entry.display_path) : id;
  };
  const save = (parent: string, evidence: ChildReachability) => {
    setPlan(null);
    setBindings((old) => ({
      ...old,
      [parent]: [
        ...(old[parent] ?? []).filter(
          (b) => bindingKey(b) !== bindingKey(evidence.selection),
        ),
        evidence.selection,
      ],
    }));
  };
  return (
    <details>
      <summary>Plan push dependencies locally</summary>
      <p>
        Select repositories independently of commit selection. This planner
        contacts no server and authorizes no push. Observe intended branches
        separately first. Unselected children are never added automatically.
      </p>
      {entries.map((entry) => (
        <div key={entry.id}>
          <label>
            <input
              type="checkbox"
              checked={selected.includes(entry.id)}
              disabled={busy || entry.availability !== "available"}
              onChange={() => {
                invalidate();
                setSelected((old) =>
                  old.includes(entry.id)
                    ? old.filter((id) => id !== entry.id)
                    : [...old, entry.id],
                );
              }}
            />
            Plan push for {label(entry.id)}
          </label>
          {selected.includes(entry.id) && (
            <>
              <button
                disabled={busy}
                onClick={() =>
                  void act(async () => {
                    invalidate();
                    setObservations((old) => ({ ...old, [entry.id]: "" }));
                    setHistory((old) => ({ ...old, [entry.id]: [] }));
                    const data = value(
                      await bridge.request({
                        kind: "remote_observations",
                        workspace,
                        worktree: entry.id,
                        offset: 0,
                      }),
                      "remote_observations",
                    );
                    setHistory((old) => ({
                      ...old,
                      [entry.id]: data.items.filter(
                        (o) => o.phase === "complete" && o.oid,
                      ),
                    }));
                  })
                }
              >
                Load branch observations for {label(entry.id)}
              </button>
              <label>
                Planned branch for {label(entry.id)}
                <select
                  value={observations[entry.id] ?? ""}
                  disabled={busy}
                  onChange={(e) => {
                    invalidate();
                    setObservations((old) => ({
                      ...old,
                      [entry.id]: e.target.value,
                    }));
                  }}
                >
                  <option value="">
                    Choose an approved observation explicitly
                  </option>
                  {(history[entry.id] ?? []).map((o) => (
                    <option key={o.plan.id} value={o.plan.id}>
                      {o.plan.destination} · {o.plan.reference} · {o.oid?.hex} ·
                      observed{" "}
                      {o.observed_at_ms != null
                        ? new Date(o.observed_at_ms).toISOString()
                        : "Unknown time"}
                    </option>
                  ))}
                </select>
              </label>
            </>
          )}
        </div>
      ))}
      <button
        disabled={
          busy ||
          !selected.length ||
          selected.length > 32 ||
          selected.some((id) => !observations[id])
        }
        onClick={() =>
          void act(async () => {
            invalidate();
            const next: PlannedPush[] = [];
            for (const id of selected) {
              next.push({
                worktree: id,
                preview: value(
                  await bridge.request({
                    kind: "push_preview",
                    workspace,
                    worktree: id,
                    observation: observations[id],
                  }),
                  "push_preview",
                ),
              });
            }
            setSources(next);
          })
        }
      >
        Inspect selected push sources
      </button>
      {sources.map((source) => (
        <section
          key={source.worktree}
          aria-label={`Push dependencies for ${label(source.worktree)}`}
        >
          <h3>{label(source.worktree)}</h3>
          <PushPreviewResult preview={source.preview} />
          <label>
            Pre-push hook policy for {label(source.worktree)}
            <select
              value={hooks[source.worktree] ?? "execute"}
              disabled={busy}
              onChange={(e) => {
                const selected = e.target.value;
                if (selected === "execute" || selected === "bypass_all")
                  setHooks((old) => ({ ...old, [source.worktree]: selected }));
              }}
            >
              <option value="execute">Execute supported pre-push hook</option>
              <option value="bypass_all">
                Explicitly bypass pre-push hook
              </option>
            </select>
          </label>
          <button
            disabled={busy || !enabled}
            onClick={() =>
              void act(async () => {
                await bridge.request({
                  kind: "review_push_check",
                  workspace,
                  selection: {
                    worktree: source.worktree,
                    observation: source.preview.observation_id,
                    source_oid: source.preview.source_oid,
                    children: bindings[source.worktree] ?? [],
                  },
                  hooks: hooks[source.worktree] ?? "execute",
                });
              })
            }
          >
            Review pre-push check for {label(source.worktree)}
          </button>
          <p>
            This separate approval can run the reviewed hook. It publishes
            nothing and grants no push authority. Every required child must
            already be reachable remotely; a conditional plan is insufficient.
            Hook effects may change files or contact servers.
          </p>
          <button
            disabled={busy || !enabled}
            onClick={() =>
              void act(async () => {
                await bridge.request({
                  kind: "review_push_publication",
                  workspace,
                  selection: {
                    worktree: source.worktree,
                    observation: source.preview.observation_id,
                    source_oid: source.preview.source_oid,
                    children: bindings[source.worktree] ?? [],
                  },
                  hooks: hooks[source.worktree] ?? "execute",
                });
              })
            }
          >
            Review exact push publication for {label(source.worktree)}
          </button>
          <p>
            Publication requires its own exact native approval. It runs a fresh
            reviewed check and attempts one fast-forward update of the approved
            committed source. If transport is interrupted, inspect the
            destination through a separately approved observation before any new
            push review.
          </p>

          {source.preview.outgoing_gitlinks.length > 0 && (
            <ChildReachabilityForm
              bridge={bridge}
              workspace={workspace}
              worktree={source.worktree}
              preview={source.preview}
              busy={busy}
              act={act}
              onEvidence={(e) => save(source.worktree, e)}
            />
          )}
          <h4>
            Child selections in this plan ·{" "}
            {(bindings[source.worktree] ?? []).length}
          </h4>
          {(bindings[source.worktree] ?? []).map((binding) => (
            <p key={bindingKey(binding)}>
              {bytes(binding.path)} · {label(binding.child_worktree)} ·{" "}
              {binding.child_oid.hex}{" "}
              <button
                disabled={busy}
                onClick={() => {
                  setPlan(null);
                  setBindings((old) => ({
                    ...old,
                    [source.worktree]: (old[source.worktree] ?? []).filter(
                      (b) => bindingKey(b) !== bindingKey(binding),
                    ),
                  }));
                }}
              >
                Remove child selection {bytes(binding.path)}
              </button>
            </p>
          ))}
        </section>
      ))}
      <button
        disabled={busy || !sources.length}
        onClick={() =>
          void act(async () => {
            setPlan(null);
            setPlan(
              value(
                await bridge.request({
                  kind: "push_dependency_plan",
                  workspace,
                  selections: sources.map((source) => ({
                    worktree: source.worktree,
                    observation: source.preview.observation_id,
                    source_oid: source.preview.source_oid,
                    children: bindings[source.worktree] ?? [],
                  })),
                }),
                "push_dependency_plan",
              ),
            );
          })
        }
      >
        Build local dependency plan
      </button>
      <button
        disabled={busy || !sources.length || sources.length !== selected.length}
        onClick={() =>
          void act(async () => {
            setBatch(
              value(
                await bridge.request({
                  kind: "create_push_batch",
                  workspace,
                  selections: sources.map((source) => ({
                    selection: {
                      worktree: source.worktree,
                      observation: source.preview.observation_id,
                      source_oid: source.preview.source_oid,
                      children: bindings[source.worktree] ?? [],
                    },
                    hooks: hooks[source.worktree] ?? "execute",
                  })),
                }),
                "push_batch",
              ),
            );
          })
        }
      >
        Create guided push queue
      </button>
      <button
        disabled={busy}
        onClick={() =>
          void act(async () => {
            const page = value(
              await bridge.request({
                kind: "push_batches",
                workspace,
                offset: 0,
              }),
              "push_batches",
            );
            setSavedQueues(page.items);
            setQueueNext(page.next ?? null);
          })
        }
      >
        Load guided push queues
      </button>
      {savedQueues.length > 0 && (
        <ul aria-label="Saved guided push queues">
          {savedQueues.map((saved) => (
            <li key={saved.plan.id}>
              <button disabled={busy} onClick={() => setBatch(saved)}>
                Inspect queue from{" "}
                {new Date(saved.plan.created_at_ms).toISOString()} ·{" "}
                {saved.phase.replaceAll("_", " ")}
              </button>
            </li>
          ))}
        </ul>
      )}
      {queueNext !== null && (
        <button
          disabled={busy}
          onClick={() =>
            void act(async () => {
              const page = value(
                await bridge.request({
                  kind: "push_batches",
                  workspace,
                  offset: queueNext,
                }),
                "push_batches",
              );
              setSavedQueues((old) => {
                const ids = new Set(old.map((v) => v.plan.id));
                return [
                  ...old,
                  ...page.items.filter((v) => !ids.has(v.plan.id)),
                ];
              });
              setQueueNext(page.next ?? null);
            })
          }
        >
          Load more guided push queues
        </button>
      )}
      {batch && (
        <GuidedPushQueue
          bridge={bridge}
          workspace={workspace}
          batch={batch}
          sources={sources}
          bindings={bindings}
          enabled={enabled}
          busy={busy}
          act={act}
          revision={revision}
          onChange={setBatch}
          label={label}
        />
      )}
      {plan && (
        <section aria-label="Push dependency planning result">
          <h3>
            {plan.order.length
              ? "Conditional review order"
              : "Dependency planning blocked"}
          </h3>
          <p role="status">
            No Git writes or network operations were executed. Selected children
            must be reviewed separately; availability must be checked again
            after any child publication and before the parent operation. This
            plan grants no push authority.
          </p>
          {plan.issues.map((issue, i) => (
            <p role="alert" key={i}>
              {issue}
            </p>
          ))}
          {plan.cycle.length > 0 && (
            <p role="alert">
              Selected dependency cycle: {plan.cycle.map(label).join(" → ")}
            </p>
          )}
          <ol>
            {plan.order.map((id) => (
              <li key={id}>{label(id)}</li>
            ))}
          </ol>
          {plan.dependencies.map((edge, i) => (
            <p key={i}>
              {label(edge.parent_worktree)} · {bytes(edge.path)} ·{" "}
              {edge.child_worktree
                ? label(edge.child_worktree)
                : "No child selected"}{" "}
              ·{" "}
              {edge.state === "already_reachable"
                ? "Reachable at observed time"
                : edge.state === "selected_child_required"
                  ? "Selected child must publish first; remote availability remains unproven"
                  : "Blocked"}
              {edge.issue ? ` · ${edge.issue}` : ""}
            </p>
          ))}
        </section>
      )}
    </details>
  );
}
