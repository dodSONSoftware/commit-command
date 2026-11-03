import { useState } from "react";
import { value, type Bridge } from "../services/bridge";
import type {
  ChildReachability,
  Observation,
  PushPreview,
  StoredWorktree,
} from "../services/response.generated";
import { bytes, pathText } from "../stores/model";
export function ChildReachabilityForm({
  bridge,
  workspace,
  worktree,
  preview,
  busy,
  act,
  onEvidence,
}: {
  bridge: Bridge;
  workspace: string;
  worktree: string;
  preview: PushPreview;
  busy: boolean;
  act: (action: () => Promise<void>) => Promise<void>;
  onEvidence?: (evidence: ChildReachability) => void;
}) {
  const [entries, setEntries] = useState<StoredWorktree[]>([]);
  const [child, setChild] = useState("");
  const [observations, setObservations] = useState<Observation[]>([]);
  const [observation, setObservation] = useState("");
  const [selected, setSelected] = useState("");
  const [result, setResult] = useState<ChildReachability | null>(null);
  const choices = preview.outgoing_gitlinks.flatMap((r) =>
    r.mappings
      .filter((m) => !m.issue && m.gitmodules_blob && m.destination)
      .map((m) => ({ requirement: r, mapping: m })),
  );
  const choice = selected === "" ? undefined : choices[Number(selected)];
  return (
    <section aria-label="Child remote reachability evidence">
      <h4>Check child availability from approved observations</h4>
      <p>
        This local check contacts no server. First explicitly observe the
        intended branch in the selected child repository and, if objects are
        missing, separately download/import it. Observations must be at most
        five minutes old. A result is historical evidence, never push approval.
      </p>
      <label>
        Historical child requirement
        <select
          value={selected}
          disabled={busy}
          onChange={(e) => {
            setSelected(e.target.value);
            setResult(null);
          }}
        >
          <option value="">Choose a historical mapping explicitly</option>
          {choices.map((c, i) => (
            <option key={i} value={i}>
              {bytes(c.requirement.path)} · {c.requirement.oid.hex} ·{" "}
              {c.mapping.destination?.display} ·{" "}
              {c.mapping.gitmodules_blob?.hex}
            </option>
          ))}
        </select>
      </label>
      <button
        disabled={busy}
        onClick={() =>
          void act(async () => {
            setResult(null);
            setChild("");
            setObservation("");
            setObservations([]);
            setEntries(
              value(
                await bridge.request({ kind: "inventory", workspace }),
                "inventory",
              ).entries.filter((e) => e.id !== worktree),
            );
          })
        }
      >
        Load child repository choices
      </button>
      <label>
        Child repository
        <select
          disabled={busy}
          value={child}
          onChange={(e) => {
            setChild(e.target.value);
            setObservation("");
            setObservations([]);
            setResult(null);
          }}
        >
          <option value="">Choose a child repository explicitly</option>
          {entries.map((entry) => (
            <option key={entry.id} value={entry.id}>
              {pathText(entry.metadata.root)}
            </option>
          ))}
        </select>
      </label>
      <button
        disabled={busy || !child}
        onClick={() =>
          void act(async () => {
            setResult(null);
            setObservation("");
            setObservations(
              value(
                await bridge.request({
                  kind: "remote_observations",
                  workspace,
                  worktree: child,
                  offset: 0,
                }),
                "remote_observations",
              ).items.filter((o) => o.phase === "complete" && o.oid),
            );
          })
        }
      >
        Load approved child observations
      </button>
      <label>
        Child branch observation
        <select
          disabled={busy}
          value={observation}
          onChange={(e) => {
            setObservation(e.target.value);
            setResult(null);
          }}
        >
          <option value="">Choose an approved observation explicitly</option>
          {observations.map((o) => (
            <option key={o.plan.id} value={o.plan.id}>
              {o.plan.destination} · {o.plan.reference} · {o.oid?.hex} ·{" "}
              {new Date(o.observed_at_ms ?? 0).toISOString()}
            </option>
          ))}
        </select>
      </label>
      <button
        disabled={busy || !child || !observation || !choice}
        onClick={() =>
          void act(async () => {
            setResult(null);
            if (!choice?.mapping.gitmodules_blob) return;
            setResult(
              value(
                await bridge.request({
                  kind: "child_reachability",
                  workspace,
                  worktree,
                  selection: {
                    parent_observation: preview.observation_id,
                    source_oid: preview.source_oid,
                    path: choice.requirement.path,
                    child_oid: choice.requirement.oid,
                    gitmodules_blob: choice.mapping.gitmodules_blob,
                    child_worktree: child,
                    child_observation: observation,
                  },
                }),
                "child_reachability",
              ),
            );
          })
        }
      >
        Check child ancestry locally
      </button>
      {result && (
        <p role="status">
          {result.reachable
            ? "Reachable from observed child branch"
            : "Not reachable from observed child branch"}{" "}
          · {result.child_destination} · {result.child_reference} · observed{" "}
          {new Date(result.child_observed_at_ms).toISOString()}. This result
          grants no push authority and must be revalidated before execution.
        </p>
      )}
      {result && onEvidence && (
        <button disabled={busy} onClick={() => onEvidence(result)}>
          Use checked child selection for planning
        </button>
      )}
    </section>
  );
}
