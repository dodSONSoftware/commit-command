import { useCallback, useEffect, useRef, useState } from "react";
import { value, type Bridge } from "../services/bridge";
import type { OperationAuditQuery } from "../services/request.generated";
import type {
  AuditEntry,
  OperationAuditPage,
  StoredWorktree,
} from "../services/response.generated";
import { bytes, pathText, repoName } from "../stores/model";
import { ObservationRecords } from "./ObservationHistory";
import { FetchImportRecords } from "./FetchImportHistory";
import { FetchCaptureRecords } from "./FetchCaptureHistory";

const kinds = [
  "stage",
  "unstage",
  "commit",
  "batch_commit",
  "remote_observation",
  "fetch_capture",
  "fetch_import",
] as const;
const operationName = (kind: string) =>
  kind === "fetch_capture"
    ? "branch download"
    : kind === "fetch_import"
      ? "captured branch import"
      : kind.replaceAll("_", " ");
const results = [
  "pending",
  "complete",
  "blocked",
  "cancelled",
  "interrupted",
  "needs_attention",
] as const;
const emptyQuery: OperationAuditQuery = {
  offset: 0,
  worktree: null,
  operation_kind: null,
  result: null,
  since_ms: null,
  until_ms: null,
};
const text = (value: string) =>
  bytes(Array.from(new TextEncoder().encode(value)));
export function OperationHistory({
  bridge,
  workspace,
  entries,
  busy,
  act,
  revision,
}: {
  bridge: Bridge;
  workspace: string;
  entries: StoredWorktree[];
  busy: boolean;
  act: (action: () => Promise<void>) => Promise<void>;
  revision: number;
}) {
  const [page, setPage] = useState<OperationAuditPage | null>(null);
  const [kind, setKind] = useState<OperationAuditQuery["operation_kind"]>(null);
  const [result, setResult] = useState<OperationAuditQuery["result"]>(null);
  const [worktree, setWorktree] = useState("");
  const [since, setSince] = useState("");
  const [until, setUntil] = useState("");
  const applied = useRef<OperationAuditQuery>(emptyQuery);
  const sequence = useRef(0);
  const load = useCallback(
    async (query: OperationAuditQuery) => {
      const request = ++sequence.current;
      if (!query.offset) setPage(null);
      const data = value(
        await bridge.request({ kind: "operation_audit", workspace, query }),
        "operation_audit",
      );
      if (request === sequence.current)
        setPage((previous) =>
          query.offset && previous
            ? { items: [...previous.items, ...data.items], next: data.next }
            : data,
        );
    },
    [bridge, workspace],
  );
  useEffect(() => {
    const requests = sequence;
    void act(() => load({ ...applied.current, offset: 0 }));
    return () => {
      ++requests.current;
    };
  }, [act, load, revision]);
  const recover = (operation: string) =>
    void act(async () => {
      await bridge.request({ kind: "review_recovery", workspace, operation });
    });
  return (
    <section>
      <div className="page-title">
        <div>
          <span className="eyebrow">DURABLE CORE RECORDS</span>
          <h1>Operation results</h1>
          <p>
            Local Git operations, commit batches, remote observations and branch
            downloads and imports in one recorded chronology. Reading history
            cannot repeat an operation or contact a server.
          </p>
        </div>
        <button
          disabled={busy}
          onClick={() =>
            void act(() => load({ ...applied.current, offset: 0 }))
          }
        >
          Refresh
        </button>
      </div>
      <form
        className="activity-filters"
        onSubmit={(event) => {
          event.preventDefault();
          const query: OperationAuditQuery = {
            offset: 0,
            operation_kind: kind,
            result,
            worktree: worktree || null,
            since_ms: since ? Date.parse(`${since}T00:00:00Z`) : null,
            until_ms: until ? Date.parse(`${until}T23:59:59.999Z`) : null,
          };
          applied.current = query;
          void act(() => load(query));
        }}
      >
        <label>
          Operation type
          <select
            aria-label="Audit operation type"
            value={kind ?? ""}
            onChange={(e) =>
              setKind(kinds.find((kind) => kind === e.target.value) ?? null)
            }
          >
            <option value="">All types</option>
            {kinds.map((kind) => (
              <option key={kind} value={kind}>
                {operationName(kind)}
              </option>
            ))}
          </select>
        </label>
        <label>
          Result
          <select
            aria-label="Audit result"
            value={result ?? ""}
            onChange={(e) =>
              setResult(
                results.find((result) => result === e.target.value) ?? null,
              )
            }
          >
            <option value="">All results</option>
            {results.map((result) => (
              <option key={result} value={result}>
                {result.replaceAll("_", " ")}
              </option>
            ))}
          </select>
        </label>
        <label>
          Repository
          <select
            aria-label="Audit repository"
            value={worktree}
            onChange={(e) => setWorktree(e.target.value)}
          >
            <option value="">Entire workspace</option>
            {entries.map((entry) => (
              <option key={entry.id} value={entry.id}>
                {repoName(entry)}
              </option>
            ))}
          </select>
        </label>
        <label>
          Since UTC
          <input
            aria-label="Audit since"
            type="date"
            value={since}
            onChange={(e) => setSince(e.target.value)}
          />
        </label>
        <label>
          Until UTC
          <input
            aria-label="Audit until"
            type="date"
            value={until}
            onChange={(e) => setUntil(e.target.value)}
          />
        </label>
        <button disabled={busy}>Apply audit filters</button>
      </form>
      <p>
        Results describe recorded outcomes. Interrupted reviews carry no
        authority. Needs attention includes partial batches, unresolved
        publication, reconciliation and post-processing failures; inspect exact
        details below. Remote OIDs describe past server observations, not
        current synchronization.
      </p>
      <div className="operations">
        {page?.items.length === 0 && (
          <p>No recorded operations match these filters.</p>
        )}
        {page?.items.map((item) => (
          <section
            key={`${item.operation_kind}:${item.id}`}
            aria-label={`${operationName(item.operation_kind)} · ${item.id}`}
          >
            <h2>
              {operationName(item.operation_kind)} ·{" "}
              {item.result.replaceAll("_", " ")}
            </h2>
            <p>
              <code>{item.id}</code> ·{" "}
              <time>{new Date(item.created_at_ms).toISOString()}</time>
            </p>
            {item.record.kind === "local" && (
              <LocalRecord record={item.record.value} onRecover={recover} />
            )}
            {item.record.kind === "remote_observation" && (
              <ObservationRecords items={[item.record.value]} />
            )}
            {item.record.kind === "fetch_capture" && (
              <FetchCaptureRecords items={[item.record.value]} />
            )}
            {item.record.kind === "fetch_import" && (
              <FetchImportRecords
                items={[item.record.value]}
                busy={busy}
                onRecover={(operation) =>
                  void act(async () => {
                    await bridge.request({
                      kind: "review_fetch_import_recovery",
                      workspace,
                      operation,
                    });
                  })
                }
              />
            )}
            {item.record.kind === "batch" && (
              <article>
                <p>
                  Batch phase:{" "}
                  {item.record.value.record.phase.replaceAll("_", " ")}. A
                  partial batch may include published commits; inspect each
                  item.
                </p>
                {item.record.value.record.issue && (
                  <p role="alert">
                    {text(item.record.value.record.issue.message)}
                  </p>
                )}
                {item.record.value.record.items.map((entry) => (
                  <p key={entry.worktree}>
                    {pathText(entry.path)} ·{" "}
                    {entry.operation ?? "No operation prepared"}
                    {entry.issue && ` · ${text(entry.issue.message)}`}
                  </p>
                ))}
                {item.record.value.outcomes.map((outcome) => (
                  <p key={outcome.operation_id}>
                    {outcome.operation_id} ·{" "}
                    {outcome.phase.replaceAll("_", " ")} · Published:{" "}
                    {String(outcome.published)} · Index reconciled:{" "}
                    {String(outcome.index_reconciled)}
                    {outcome.reason && ` · ${text(outcome.reason)}`}
                  </p>
                ))}
              </article>
            )}
          </section>
        ))}
        {page?.next != null && (
          <button
            disabled={busy}
            onClick={() =>
              void act(() => load({ ...applied.current, offset: page.next! }))
            }
          >
            Load more results
          </button>
        )}
      </div>
    </section>
  );
}

function LocalRecord({
  record,
  onRecover,
}: {
  record: AuditEntry;
  onRecover: (operation: string) => void;
}) {
  const op = record.outcome;
  return (
    <article>
      <span
        className={`badge ${op.phase === "complete" ? "success" : "warning"}`}
      >
        {op.phase.replaceAll("_", " ")}
      </span>
      <p>{text(record.subject)}</p>
      <details>
        <summary>Recorded phases</summary>
        {record.events.map((event, i) => (
          <p key={i}>
            {new Date(event.at_ms).toLocaleString()} · {event.phase} ·
            Published: {String(event.published)}
          </p>
        ))}
        {!record.events_complete && (
          <p>Phase history exceeds the display limit.</p>
        )}
      </details>
      <p>
        {op.reason
          ? text(op.reason)
          : op.published
            ? "Commit published; inspect reconciliation and post-processing below."
            : "No commit publication recorded."}
      </p>
      <p>
        Published: {String(op.published)} · Index reconciled:{" "}
        {String(op.index_reconciled)}
      </p>
      {op.new_head && <code>{op.new_head}</code>}
      {op.post_processing.map((result, i) => (
        <p key={i}>{text(result)}</p>
      ))}
      {![
        "complete",
        "published_with_post_processing_failure",
        "cancelled_before_publication",
        "needs_reauthorization",
        "proposed",
        "approved",
      ].includes(op.phase) && (
        <button onClick={() => onRecover(op.operation_id)}>
          Inspect guarded recovery
        </button>
      )}
    </article>
  );
}
