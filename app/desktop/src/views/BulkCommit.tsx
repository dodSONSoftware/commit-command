import { useOperationReview } from "./OperationReview";
import { CommitMessageEditor } from "./CommitMessageEditor";
import {
  useCallback,
  useEffect,
  useState,
  type Dispatch,
  type SetStateAction,
} from "react";
import { value, type Bridge } from "../services/bridge";
import type { BatchSelection } from "../services/request.generated";
import type {
  BatchPage,
  StoredWorktree,
  BulkComposerState,
} from "../services/response.generated";
import { publicationStatus } from "../services/publication";
import { pathText, repoName } from "../stores/model";

type Configuration = Pick<
  BatchSelection,
  "mode" | "excluded" | "hooks" | "signing"
>;
const defaults: Configuration = {
  mode: "review_all",
  excluded: [],
  hooks: "execute",
  signing: "configured",
};

export function BulkCommit({
  bridge,
  workspace,
  entries,
  selected,
  drafts,
  composer,
  setComposer,
  setDraft,
  setSelected,
  enabled,
  busy,
  act,
  revision,
}: {
  revision?: number;
  bridge: Bridge;
  workspace: string;
  entries: StoredWorktree[];
  selected: string[];
  drafts: Record<string, string>;
  composer: BulkComposerState;
  setComposer: Dispatch<SetStateAction<BulkComposerState>>;
  setDraft: (id: string, text: string) => void;
  setSelected: (ids: string[]) => void;
  enabled: boolean;
  busy: boolean;
  act: (action: () => Promise<void>) => Promise<void>;
}) {
  const { review, reviewDialog } = useOperationReview(bridge, workspace);
  const shared = composer.message;
  const configuration = composer.configurations;
  const [additionalPaths, setAdditionalPaths] = useState<
    Record<string, string>
  >({});
  const [results, setResults] = useState<BatchPage | null>(null);
  const chosen = selected.map((id) => entries.find((e) => e.id === id));
  const valid =
    chosen.length > 0 &&
    chosen.length <= 20 &&
    chosen.every((entry) => entry && (drafts[entry.id] || shared).trim());
  function change(id: string, update: Partial<Configuration>) {
    setComposer((current) => ({
      ...current,
      configurations: {
        ...current.configurations,
        [id]: { ...(current.configurations[id] ?? defaults), ...update },
      },
    }));
  }
  const readPage = useCallback(
    async (offset = 0) =>
      value(
        await bridge.request({ kind: "batches", workspace, offset }),
        "batches",
      ),
    [bridge, workspace],
  );
  useEffect(() => {
    if (revision == null) return;
    let live = true;
    void act(async () => {
      const page = await readPage();
      if (live) setResults(page);
    });
    return () => {
      live = false;
    };
  }, [revision, readPage, act]);
  async function refresh(offset = 0) {
    const page = await readPage(offset);
    setResults((current) =>
      offset && current
        ? { items: [...current.items, ...page.items], next: page.next }
        : page,
    );
  }
  return (
    <section className="bulk-commit">
      <div className="page-title">
        <div>
          <h1>Bulk Commit</h1>
          <p>
            Only explicitly selected worktrees participate. Commits run
            sequentially and stop on first failure; completed commits are
            preserved.
          </p>
        </div>
        <button disabled={busy} onClick={() => void act(() => refresh())}>
          Refresh batch results
        </button>
      </div>
      {reviewDialog}
      {!enabled && (
        <p role="status">Bulk Commit requires a write-enabled desktop build.</p>
      )}
      <p>
        Selected submodules run before their selected parents. After a child
        commit, prepare a fresh review for the parent's changed gitlink. Nested
        independent repositories are never selected implicitly.
      </p>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          if (!valid || busy || !enabled) return;
          void act(async () => {
            await review({
              kind: "review_batch",
              workspace,
              selections: chosen.flatMap((entry) =>
                entry
                  ? [
                      {
                        worktree: entry.id,
                        message: drafts[entry.id] || shared,
                        ...(configuration[entry.id] ?? defaults),
                      },
                    ]
                  : [],
              ),
            });
            await refresh();
          });
        }}
      >
        <CommitMessageEditor
          layoutKey="bulk"
          label="Shared commit message"
          value={shared}
          disabled={busy}
          onChange={(message) =>
            setComposer((current) => ({ ...current, message }))
          }
        />
        <p>
          {selected.length} explicitly selected · maximum 20 per review. A
          nonempty repository draft overrides the shared message.
        </p>
        {chosen.map((entry, index) => {
          if (!entry)
            return (
              <p role="alert" key={selected[index]}>
                Selected worktree unavailable: {selected[index]}. Return to
                Overview to update selection.
              </p>
            );
          const config = configuration[entry.id] ?? defaults;
          const paths = new Map(
            (entry.status?.changes ?? [])
              .flatMap((c) => [
                c.path,
                ...(c.original_path ? [c.original_path] : []),
              ])
              .concat(config.excluded)
              .map((p) => [JSON.stringify(p), p]),
          );
          const additional = additionalPaths[entry.id] ?? "";
          const addExclusion = () => {
            if (!additional) return;
            const path: BatchSelection["excluded"][number] = {
              encoding: "utf8",
              value: additional,
            };
            if (
              !config.excluded.some(
                (p) => JSON.stringify(p) === JSON.stringify(path),
              )
            )
              change(entry.id, { excluded: [...config.excluded, path] });
            setAdditionalPaths((current) => ({ ...current, [entry.id]: "" }));
          };
          return (
            <fieldset key={entry.id} disabled={busy}>
              <legend>{repoName(entry)}</legend>
              <p className="path">{pathText(entry.display_path)}</p>
              <CommitMessageEditor
                layoutKey="bulk"
                label={`Message override · ${repoName(entry)}`}
                rows={3}
                value={drafts[entry.id] ?? ""}
                placeholder="Use shared message"
                onChange={(message) => setDraft(entry.id, message)}
              />
              <div className="activity-filters">
                <label>
                  Content · {repoName(entry)}
                  <select
                    value={config.mode}
                    onChange={(e) =>
                      change(entry.id, {
                        mode: e.target.value as Configuration["mode"],
                      })
                    }
                  >
                    <option value="review_all">
                      Review all eligible changes
                    </option>
                    <option value="index_only">
                      Index only · preserve partial staging
                    </option>
                  </select>
                </label>
                <label>
                  Hooks · {repoName(entry)}
                  <select
                    value={config.hooks}
                    onChange={(e) =>
                      change(entry.id, {
                        hooks: e.target.value as Configuration["hooks"],
                      })
                    }
                  >
                    <option value="execute">Execute supported hooks</option>
                    <option value="bypass_all">
                      Bypass all · explicit review
                    </option>
                  </select>
                </label>
                <label>
                  Signing · {repoName(entry)}
                  <select
                    value={config.signing}
                    onChange={(e) =>
                      change(entry.id, {
                        signing: e.target.value as Configuration["signing"],
                      })
                    }
                  >
                    <option value="configured">
                      Configured signing policy
                    </option>
                    <option value="unsigned">Explicitly unsigned</option>
                  </select>
                </label>
              </div>
              <button
                type="button"
                onClick={() => change(entry.id, { excluded: [] })}
              >
                Clear path exclusions · {repoName(entry)}
              </button>
              <p>
                {config.excluded.length} stored exclusions. Clear exclusions if
                a path is no longer eligible.
              </p>
              <details>
                <summary>
                  Exclude specific changed paths · {repoName(entry)}
                </summary>
                <p>
                  Core determines eligibility from fresh status. An exclusion
                  that no longer applies blocks this worktree until reviewed
                  again.
                </p>
                <p>
                  A changed submodule gitlink may appear only in the fresh bulk
                  review. Cancel that review to exclude its exact relative path
                  here. Core checks every exclusion before preparing a new
                  review.
                </p>
                <label>
                  Additional exclusion · {repoName(entry)}
                  <input
                    value={additional}
                    maxLength={4096}
                    onChange={(event) =>
                      setAdditionalPaths((current) => ({
                        ...current,
                        [entry.id]: event.target.value,
                      }))
                    }
                    onKeyDown={(event) => {
                      if (event.key === "Enter") {
                        event.preventDefault();
                        addExclusion();
                      }
                    }}
                  />
                </label>
                <button
                  type="button"
                  disabled={!additional}
                  onClick={addExclusion}
                >
                  Add exclusion · {repoName(entry)}
                </button>
                {[...paths].map(([key, path]) => (
                  <label className="bulk-path" key={key}>
                    <input
                      type="checkbox"
                      checked={config.excluded.some(
                        (p) => JSON.stringify(p) === key,
                      )}
                      onChange={(event) =>
                        change(entry.id, {
                          excluded: event.target.checked
                            ? [...config.excluded, path]
                            : config.excluded.filter(
                                (p) => JSON.stringify(p) !== key,
                              ),
                        })
                      }
                    />
                    Exclude {pathText(path)}
                  </label>
                ))}
              </details>
            </fieldset>
          );
        })}
        <button className="primary" disabled={!enabled || busy || !valid}>
          Review bulk commit
        </button>
      </form>
      <h2>Durable batch results</h2>
      <p>
        Results refresh after native operations finish. Closing a review or
        restarting never resumes a batch automatically.
      </p>
      {results?.items.length === 0 && (
        <p>No recorded batches in this workspace.</p>
      )}
      {results?.items.map(({ record, outcomes }) => (
        <article className="bulk-result" key={record.id}>
          <h3>
            {record.phase} · {new Date(record.created_at_ms).toISOString()}
          </h3>
          <p className="path">{record.id}</p>
          {record.issue && (
            <p role="alert">
              {record.issue.kind}: {record.issue.message}
            </p>
          )}
          <ol>
            {record.items.map((item) => {
              const outcome = outcomes.find(
                (o) => o.operation_id === item.operation,
              );
              return (
                <li key={item.worktree}>
                  <span className="path">{pathText(item.path)}</span> ·{" "}
                  {outcome?.phase ?? "blocked"} · {publicationStatus(outcome)}
                  {item.issue && (
                    <p>
                      {item.issue.kind}: {item.issue.message}
                    </p>
                  )}
                  {outcome?.reason && <p>{outcome.reason}</p>}
                </li>
              );
            })}
          </ol>
          <button
            disabled={
              busy ||
              record.phase === "proposed" ||
              record.phase === "executing" ||
              record.items.every((item) =>
                outcomes.some(
                  (o) => o.operation_id === item.operation && o.published,
                ),
              )
            }
            onClick={() => {
              const unfinished = record.items
                .filter(
                  (item) =>
                    !outcomes.some(
                      (o) => o.operation_id === item.operation && o.published,
                    ),
                )
                .map((item) => item.worktree);
              setSelected(unfinished);
            }}
          >
            Select unfinished worktrees for fresh review
          </button>
        </article>
      ))}
      {results?.next != null && (
        <button
          disabled={busy}
          onClick={() => void act(() => refresh(results.next!))}
        >
          Load older batches
        </button>
      )}
    </section>
  );
}
