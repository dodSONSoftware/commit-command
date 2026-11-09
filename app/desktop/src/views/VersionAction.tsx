import { VersionActionRecord } from "./VersionActionRecord";
import { useCallback, useEffect, useRef, useState } from "react";
import { value, type Bridge } from "../services/bridge";
import type {
  Increment,
  PrereleasePolicy,
  VersionPreview,
  VersionPage,
} from "../services/response.generated";
export function VersionAction({
  bridge,
  workspace,
  worktree,
  busy,
  act,
  enabled = false,
  revision = 0,
}: {
  enabled?: boolean;
  revision?: number;
  bridge: Bridge;
  workspace: string;
  worktree: string;
  busy: boolean;
  act: (f: () => Promise<void>) => Promise<void>;
}) {
  const [path, setPath] = useState("version");
  const [increment, setIncrement] = useState<Increment>("patch");
  const [prerelease, setPrerelease] = useState<PrereleasePolicy>("reject");
  const [result, setResult] = useState<{
    revision: number;
    value: VersionPreview;
  } | null>(null);
  const sequence = useRef(0);
  const [records, setRecords] = useState<VersionPage | null>(null);
  const hasLoaded = useRef(false);
  const [recordsError, setRecordsError] = useState("");
  const refresh = useCallback(async () => {
    hasLoaded.current = true;
    setRecordsError("");
    setRecords(
      value(
        await bridge.request({
          kind: "version_actions",
          workspace,
          worktree,
          offset: 0,
        }),
        "version_actions",
      ),
    );
  }, [bridge, workspace, worktree]);
  useEffect(() => {
    sequence.current++;
    if (!hasLoaded.current) return;
    let live = true;
    void bridge
      .request({ kind: "version_actions", workspace, worktree, offset: 0 })
      .then((data) => {
        if (live) {
          setRecords(value(data, "version_actions"));
          setRecordsError("");
        }
      })
      .catch((error) => {
        if (live) setRecordsError(String(error));
      });
    return () => {
      live = false;
    };
  }, [bridge, workspace, worktree, revision]);

  function invalidate() {
    sequence.current++;
    setResult(null);
  }
  return (
    <details>
      <summary>Version file Action</summary>
      <p>
        Read one version file and calculate an explicit increment without AI.
        This preview changes no files or staging and grants no execution
        authority. Supported format: one SemVer 2.0 value, optionally ending in
        LF or CRLF.
      </p>
      <label>
        Version file relative to this repository
        <input
          value={path}
          maxLength={4096}
          disabled={busy}
          onChange={(e) => {
            invalidate();
            setPath(e.target.value);
          }}
        />
      </label>
      <label>
        Version increment
        <select
          value={increment}
          disabled={busy}
          onChange={(e) => {
            const v = e.target.value;
            if (v === "major" || v === "minor" || v === "patch") {
              invalidate();
              setIncrement(v);
            }
          }}
        >
          <option value="patch">Patch</option>
          <option value="minor">Minor</option>
          <option value="major">Major</option>
        </select>
      </label>
      <label>
        Prerelease input policy
        <select
          value={prerelease}
          disabled={busy}
          onChange={(e) => {
            const v = e.target.value;
            if (v === "reject" || v === "discard_and_increment") {
              invalidate();
              setPrerelease(v);
            }
          }}
        >
          <option value="reject">Reject prerelease input</option>
          <option value="discard_and_increment">
            Discard prerelease and increment its numeric base
          </option>
        </select>
      </label>
      <p>
        Incrementing clears build metadata. A prerelease is rejected unless you
        explicitly choose to discard it and increment its numeric base. The
        exact result is shown below.
      </p>
      <button
        disabled={busy || !path}
        onClick={() =>
          void act(async () => {
            const requestSequence = ++sequence.current;
            setResult(null);
            const next = value(
              await bridge.request({
                kind: "preview_version_file",
                workspace,
                worktree,
                path: { encoding: "utf8", value: path },
                increment,
                prerelease,
              }),
              "version_preview",
            );
            if (sequence.current === requestSequence)
              setResult({ revision, value: next });
          })
        }
      >
        Read version update preview
      </button>
      <button
        disabled={busy || !enabled || !path}
        onClick={() =>
          void act(async () => {
            invalidate();
            await bridge.request({
              kind: "review_version_action",
              workspace,
              worktree,
              path: { encoding: "utf8", value: path },
              increment,
              prerelease,
            });
          })
        }
      >
        Review exact version Action
      </button>
      <p>
        Applying this typed Action requires trust and a fresh exact native
        approval. It replaces only the reviewed file. Staging and committing are
        separately reviewed; hooks, AI and network are not run by the Action.
      </p>
      <button disabled={busy} onClick={() => void act(refresh)}>
        Load version Action results
      </button>
      {recordsError && <p role="alert">{recordsError}</p>}
      {records?.items.map((item) => (
        <VersionActionRecord
          key={item.plan.id}
          item={item}
          bridge={bridge}
          workspace={workspace}
          enabled={enabled}
          busy={busy}
          act={act}
        />
      ))}
      {records?.next != null && (
        <button
          disabled={busy}
          onClick={() =>
            void act(async () => {
              const next = records.next;
              if (next == null) return;
              const page = value(
                await bridge.request({
                  kind: "version_actions",
                  workspace,
                  worktree,
                  offset: next,
                }),
                "version_actions",
              );
              setRecords((old) =>
                old
                  ? {
                      items: [
                        ...old.items,
                        ...page.items.filter(
                          (i) =>
                            !old.items.some((v) => v.plan.id === i.plan.id),
                        ),
                      ],
                      next: page.next,
                    }
                  : page,
              );
            })
          }
        >
          Load more version Action results
        </button>
      )}
      {result?.revision === revision && (
        <section aria-label="Version update preview">
          <p>
            Old version: <code>{result.value.old}</code>
          </p>
          <p>
            New version: <code>{result.value.new}</code>
          </p>
          <p>
            Preview only. Re-read after any external edit; no file or index has
            been changed.
          </p>
        </section>
      )}
    </details>
  );
}
