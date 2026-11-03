import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type CSSProperties,
} from "react";
import type { Bridge } from "../services/bridge";
import { PushDependencyPlanner } from "./PushDependencyPlanner";
import { value } from "../services/bridge";
import type {
  Workspace,
  StoredWorktree,
  DesktopState,
  BulkComposerState,
  TrustState,
  Diff,
  Change,
  NativePath,
  Commit as CommitDetail,
  RecoveryNotice,
  RemoteRecoveryNotice,
} from "../services/response.generated";
import type { WriteRequest } from "../services/request.generated";
import {
  initialState,
  initialBulkComposer,
  bytes,
  pathText,
  relative,
  repoName,
  toggleSelection,
  openTab,
  closeTab,
  updateTab,
  filtered,
  buildTree,
  type TreeNode,
} from "../stores/model";
import { WorkspaceActivity } from "./WorkspaceActivity";
import { HistoryGraph } from "./HistoryGraph";
import { BulkCommit } from "./BulkCommit";
import { RemoteInspector } from "./RemoteInspector";
import { OperationHistory } from "./OperationHistory";
type Source = "index" | "working_tree";
export function App({ bridge }: { bridge: Bridge }) {
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [workspace, setWorkspace] = useState<string | null>(null);
  const [entries, setEntries] = useState<StoredWorktree[]>([]);
  const [nav, setNav] = useState<DesktopState>(initialState);
  const [bulkComposer, setBulkComposer] =
    useState<BulkComposerState>(initialBulkComposer);
  const [trust, setTrust] = useState<TrustState[]>([]);
  const [pendingActions, setPendingActions] = useState(0);
  const busy = pendingActions > 0;
  const [error, setError] = useState("");
  const [closing, setClosing] = useState(false);
  const [loading, setLoading] = useState(true);
  const [monitor, setMonitor] = useState("Not monitoring");
  const [attention, setAttention] = useState("all");
  const [showOperations, setShowOperations] = useState(false);
  const [showActivity, setShowActivity] = useState(false);
  const [showBulk, setShowBulk] = useState(false);
  const [writeBusy, setWriteBusy] = useState(false);
  const [writesEnabled, setWritesEnabled] = useState(false);
  const [recovery, setRecovery] = useState<RecoveryNotice[]>([]);
  const [importRecovery, setImportRecovery] = useState<RemoteRecoveryNotice[]>(
    [],
  );
  const [upstreamRecovery, setUpstreamRecovery] = useState<
    RemoteRecoveryNotice[]
  >([]);
  const recoveryNotices = [...recovery, ...importRecovery, ...upstreamRecovery];
  const context = useRef<string | null>(null);
  const loaded = useRef(false);
  const polling = useRef(false);
  const seenWriteRevision = useRef(0);
  const [resultsRevision, setResultsRevision] = useState(0);
  const latest = useRef(nav);
  const latestBulk = useRef(bulkComposer);
  useEffect(() => {
    latestBulk.current = bulkComposer;
  }, [bulkComposer]);
  useEffect(() => {
    latest.current = nav;
  }, [nav]);
  useEffect(() => {
    if (!bridge.onCloseRequested) return;
    let live = true;
    let unsubscribe: (() => void) | undefined;
    bridge
      .onCloseRequested(
        () => {
          if (!live) throw new Error("Close listener is no longer active");
          if (context.current && !loaded.current)
            throw new Error(
              "Workspace presentation state has not loaded; latest drafts cannot be saved",
            );
          const workspace = loaded.current ? context.current : null;
          return {
            version: 1,
            epoch: bridge.epoch,
            workspace,
            navigation: workspace ? latest.current : null,
            bulk: workspace ? latestBulk.current : null,
          };
        },
        (pending, message) => {
          if (!live) return;
          setClosing(pending);
          if (message) setError(message);
        },
      )
      .then((dispose) => {
        if (live) unsubscribe = dispose;
        else dispose();
      })
      .catch((error) => {
        if (live) setError(`Close draft-saving unavailable: ${String(error)}`);
      });
    return () => {
      live = false;
      unsubscribe?.();
    };
  }, [bridge]);
  const activeWorkspace = workspaces.find((w) => w.id === workspace);
  const root = activeWorkspace ? pathText(activeWorkspace.root) : "";
  const entry = entries.find((e) => e.id === nav.active_tab);
  const tab = nav.tabs.find((t) => t.worktree === nav.active_tab);
  const act = useCallback(async (action: () => Promise<void>) => {
    setPendingActions((count) => count + 1);
    setError("");
    try {
      await action();
    } catch (e) {
      setError(String(e));
    } finally {
      setPendingActions((count) => count - 1);
    }
  }, []);
  const inventory = useCallback(
    async (ws: string) => {
      const data = value(
        await bridge.request({ kind: "inventory", workspace: ws }),
        "inventory",
      );
      if (context.current !== ws) return;
      setEntries(data.entries);
      setMonitor(
        data.monitor.running
          ? (data.monitor.degraded ?? "Monitoring active workspace")
          : "Monitoring stopped",
      );
    },
    [bridge],
  );
  const load = useCallback(async () => {
    const data = value(
      await bridge.request({ kind: "workspaces" }),
      "workspaces",
    );
    setWorkspaces(data.workspaces);
    setRecovery(data.recovery);
    setImportRecovery(data.fetch_import_recovery);
    setUpstreamRecovery(data.upstream_recovery ?? []);
    const ws = data.active ?? null;
    if (context.current !== ws) {
      loaded.current = false;
      context.current = ws;
      setWorkspace(ws);
      setBulkComposer(initialBulkComposer);
      setEntries([]);
      setMonitor("Not monitoring");
      setTrust([]);
      setShowOperations(false);
      setShowActivity(false);
      setShowBulk(false);
      if (ws) {
        const state = value(
          await bridge.request({ kind: "load_state", workspace: ws }),
          "state",
        );
        const composer = value(
          await bridge.request({ kind: "load_bulk_composer", workspace: ws }),
          "bulk_composer",
        );
        if (context.current === ws) {
          setNav(state);
          setBulkComposer(composer);
          loaded.current = true;
        }
      } else {
        setNav(initialState);
      }
    }
    if (ws) await inventory(ws);
  }, [bridge, inventory]);
  useEffect(() => {
    bridge
      .capabilities()
      .then((c) => setWritesEnabled(c.writes_enabled))
      .catch((e) => setError(String(e)));
  }, [bridge]);
  useEffect(() => {
    let live = true;
    Promise.resolve().then(async () => {
      if (!live) return;
      setLoading(true);
      try {
        await load();
      } catch (error) {
        if (live) setError(`Desktop connection unavailable: ${String(error)}`);
      } finally {
        if (live) setLoading(false);
      }
    });
    return () => {
      live = false;
    };
  }, [load]);
  useEffect(() => {
    if (!workspace || closing) return;
    let live = true;
    const id = setInterval(() => {
      if (busy || polling.current) return;
      polling.current = true;
      void (async () => {
        try {
          const running = await bridge.writeStatus();
          if (!live || context.current !== workspace) return;
          setWriteBusy(running);
          // The Core owner is occupied until the write settles. Do not queue reads.
          if (running) return;
          const revision = bridge.writeRevision ?? 0;
          if (revision !== seenWriteRevision.current) {
            await load();
            if (!live || context.current !== workspace) return;
            seenWriteRevision.current = revision;
            setResultsRevision((current) => current + 1);
          } else {
            await inventory(workspace);
          }
          if (!live || context.current !== workspace) return;
          if (writesEnabled) {
            const data = await bridge.request({
              kind: "trust_status",
              workspace,
            });
            if (live && context.current === workspace)
              setTrust(value(data, "trust"));
          }
        } catch (e) {
          if (live && context.current === workspace)
            setMonitor(`Refresh unavailable: ${String(e)}`);
        } finally {
          polling.current = false;
        }
      })();
    }, 3000);
    return () => {
      live = false;
      clearInterval(id);
    };
  }, [workspace, busy, inventory, bridge, writesEnabled, load, closing]);
  useEffect(() => {
    if (!workspace || !loaded.current || closing) return;
    const id = setTimeout(() => {
      bridge
        .request({ kind: "save_state", workspace, state: nav })
        .catch((e) => setError(`Navigation was not saved: ${String(e)}`));
    }, 500);
    return () => clearTimeout(id);
  }, [nav, workspace, bridge, closing]);
  useEffect(() => {
    if (!workspace || !loaded.current || closing) return;
    const id = setTimeout(() => {
      bridge
        .request({ kind: "save_bulk_composer", workspace, state: bulkComposer })
        .catch((e) => {
          if (context.current === workspace)
            setError(`Bulk composer was not saved: ${String(e)}`);
        });
    }, 500);
    return () => clearTimeout(id);
  }, [bulkComposer, workspace, bridge, closing]);
  useEffect(() => {
    document.documentElement.dataset.theme = nav.theme;
  }, [nav.theme]);
  async function savePresentation() {
    if (workspace && loaded.current) {
      await bridge.request({
        kind: "save_state",
        workspace,
        state: latest.current,
      });
      await bridge.request({
        kind: "save_bulk_composer",
        workspace,
        state: latestBulk.current,
      });
    }
  }
  async function switchWorkspace(id: string) {
    setLoading(true);
    try {
      await savePresentation();
      await bridge.request({ kind: "activate", workspace: id });
      context.current = null;
      await load();
      setShowOperations(false);
      setShowActivity(false);
      setShowBulk(false);
    } finally {
      setLoading(false);
    }
  }
  async function create() {
    setLoading(true);
    try {
      const selection = await bridge.pick();
      if (!selection) return;
      await savePresentation();
      const name =
        selection.display.split("/").filter(Boolean).at(-1) ?? "Workspace";
      await bridge.request({ kind: "create", name, picker: selection.picker });
      context.current = null;
      await load();
    } finally {
      setLoading(false);
    }
  }
  async function refreshTrust() {
    if (!workspace) return;
    setTrust(
      value(await bridge.request({ kind: "trust_status", workspace }), "trust"),
    );
  }
  async function propose(request: WriteRequest) {
    if (!workspace || !entry) return;
    await bridge.request({
      kind: "propose",
      workspace,
      worktree: entry.id,
      request,
    });
  }
  async function trustSelected(ids: string[]) {
    if (!workspace) return;
    await bridge.request({ kind: "review_trust", workspace, worktrees: ids });
  }
  async function historyList() {
    setShowOperations(true);
    setShowActivity(false);
    setShowBulk(false);
  }
  const visible = filtered(entries, nav.filter, attention, nav.sort);
  const counts = entries.reduce(
    (s, e) => ({
      dirty:
        s.dirty +
        Number(
          !!e.status &&
            e.status.counts.staged +
              e.status.counts.unstaged +
              e.status.counts.untracked >
              0,
        ),
      issues:
        s.issues +
        Number(e.availability !== "available" || !!e.status?.counts.conflicted),
    }),
    { dirty: 0, issues: 0 },
  );
  const selectedTrust = trust.find((t) => t.worktree === entry?.id);
  return (
    <div className="shell" inert={closing || loading}>
      <header className="topbar">
        <div className="brand">
          <span className="brand-icon">⑂</span>
          <strong>GitFleet</strong>
          <span className="edition">LOCAL WORKSPACE</span>
        </div>
        <div className="workspace-switch">
          <label htmlFor="workspace">Workspace</label>
          <select
            id="workspace"
            value={workspace ?? ""}
            disabled={busy || writeBusy}
            onChange={(e) => void act(() => switchWorkspace(e.target.value))}
          >
            <option value="" disabled>
              Select a workspace
            </option>
            {workspaces.map((w) => (
              <option key={w.id} value={w.id}>
                {w.name}
              </option>
            ))}
          </select>
          <button onClick={() => void act(create)} disabled={busy || writeBusy}>
            Open folder…
          </button>
        </div>
        <select
          aria-label="Color theme"
          value={nav.theme}
          onChange={(e) => setNav({ ...nav, theme: e.target.value })}
        >
          <option value="system">System</option>
          <option value="dark">Dark</option>
          <option value="light">Light</option>
        </select>
      </header>
      <nav className="tabs" aria-label="Open tabs">
        <button
          className={
            !nav.active_tab && !showOperations && !showActivity && !showBulk
              ? "active"
              : ""
          }
          onClick={() => {
            setNav({ ...nav, active_tab: null });
            setShowOperations(false);
            setShowActivity(false);
            setShowBulk(false);
          }}
        >
          ▦ Overview
        </button>
        {nav.tabs.map((t) => {
          const e = entries.find((e) => e.id === t.worktree);
          return (
            <div
              className={`tab ${nav.active_tab === t.worktree && !showOperations && !showActivity && !showBulk ? "active" : ""}`}
              key={t.worktree}
            >
              <button
                onClick={() => {
                  setNav({ ...nav, active_tab: t.worktree });
                  setShowOperations(false);
                  setShowActivity(false);
                  setShowBulk(false);
                }}
              >
                {e ? repoName(e) : "Unavailable repository"}
              </button>
              <button
                aria-label={`Close ${e ? repoName(e) : "unavailable repository"} tab`}
                onClick={() => setNav(closeTab(nav, t.worktree))}
              >
                ×
              </button>
            </div>
          );
        })}
      </nav>
      <div className="body">
        <aside className="sidebar" style={{ width: nav.sidebar_width }}>
          <div className="workspace-heading">
            <span className="eyebrow">ACTIVE WORKSPACE</span>
            <h2>{activeWorkspace?.name ?? "Welcome"}</h2>
            <p className="path">{root || "Choose a local project folder"}</p>
          </div>
          <button
            className={
              !nav.active_tab && !showOperations && !showActivity && !showBulk
                ? "nav-item chosen"
                : "nav-item"
            }
            onClick={() => {
              setNav({ ...nav, active_tab: null });
              setShowOperations(false);
              setShowActivity(false);
              setShowBulk(false);
            }}
          >
            ▦ Workspace Overview
          </button>
          <button
            className={`nav-item ${showOperations ? "chosen" : ""}`}
            disabled={!workspace}
            onClick={() => void act(historyList)}
          >
            ◷ Operation results
          </button>
          <button
            className={`nav-item ${showActivity ? "chosen" : ""}`}
            disabled={!workspace || busy}
            onClick={() => {
              setShowActivity(true);
              setShowBulk(false);
              setShowOperations(false);
            }}
          >
            Workspace Activity
          </button>
          <button
            className={`nav-item ${showBulk ? "chosen" : ""}`}
            disabled={!workspace || busy}
            onClick={() => {
              setShowOperations(false);
              setShowActivity(false);
              setShowBulk(true);
            }}
          >
            Bulk Commit
          </button>
          <div className="sidebar-label">
            REPOSITORIES <span>{entries.length}</span>
          </div>
          <div className="repo-navigation">
            {entries.map((e) => (
              <button
                className={`repo-link ${nav.active_tab === e.id ? "chosen" : ""}`}
                key={e.id}
                onClick={() => {
                  setNav(openTab(nav, e));
                  setShowOperations(false);
                  setShowActivity(false);
                  setShowBulk(false);
                }}
              >
                <span
                  className={`dot ${e.availability !== "available" ? "bad" : e.status && e.status.counts.staged + e.status.counts.unstaged > 0 ? "changed" : "clean"}`}
                />
                <span>{repoName(e)}</span>
                <small>
                  {e.status?.counts.staged
                    ? `${e.status.counts.staged} staged`
                    : ""}
                </small>
              </button>
            ))}
          </div>
          {workspace && (
            <details className="workspace-settings">
              <summary>Workspace settings</summary>
              <label>
                Name
                <input
                  key={workspace}
                  defaultValue={activeWorkspace?.name}
                  onBlur={(e) => {
                    if (e.target.value !== activeWorkspace?.name)
                      void act(async () => {
                        await bridge.request({
                          kind: "update",
                          workspace,
                          name: e.target.value,
                        });
                        await load();
                      });
                  }}
                />
              </label>
              <button
                disabled={busy || writeBusy}
                onClick={() =>
                  void act(async () => {
                    const picked = await bridge.pick();
                    if (picked) {
                      await bridge.request({
                        kind: "update",
                        workspace,
                        picker: picked.picker,
                      });
                      await load();
                    }
                  })
                }
              >
                Change root…
              </button>
              <label>
                Excluded directory names
                <input
                  defaultValue={activeWorkspace?.discovery.excluded_names.join(
                    ", ",
                  )}
                  onBlur={(e) =>
                    void act(async () => {
                      await bridge.request({
                        kind: "update",
                        workspace,
                        exclusions: {
                          excluded_names: e.target.value
                            .split(",")
                            .map((s) => s.trim())
                            .filter(Boolean),
                          excluded_patterns:
                            activeWorkspace?.discovery.excluded_patterns ?? [],
                        },
                      });
                      await load();
                    })
                  }
                />
              </label>
              <button
                className="danger subtle"
                disabled={busy || writeBusy}
                onClick={() =>
                  void act(async () => {
                    await bridge.request({ kind: "remove", workspace });
                    await load();
                  })
                }
              >
                Remove saved definition
              </button>
              <p>Repository files are never deleted.</p>
            </details>
          )}
        </aside>
        <div
          className="splitter"
          role="separator"
          tabIndex={0}
          aria-label="Sidebar width"
          aria-valuemin={180}
          aria-valuemax={420}
          aria-valuenow={nav.sidebar_width}
          aria-orientation="vertical"
          onPointerDown={(e) => {
            e.currentTarget.setPointerCapture(e.pointerId);
          }}
          onPointerMove={(e) => {
            if (e.currentTarget.hasPointerCapture(e.pointerId))
              setNav({
                ...nav,
                sidebar_width: Math.max(180, Math.min(420, e.clientX)),
              });
          }}
          onPointerUp={(e) => {
            if (e.currentTarget.hasPointerCapture(e.pointerId))
              e.currentTarget.releasePointerCapture(e.pointerId);
          }}
          onKeyDown={(e) => {
            if (e.key === "ArrowRight" || e.key === "ArrowLeft")
              setNav({
                ...nav,
                sidebar_width: Math.max(
                  180,
                  Math.min(
                    420,
                    nav.sidebar_width + (e.key === "ArrowRight" ? 20 : -20),
                  ),
                ),
              });
          }}
        />
        <main className="content">
          {recoveryNotices.length > 0 && (
            <div className="recovery-alert" role="status">
              <strong>
                Showing {recoveryNotices.length} recorded operation(s) needing
                attention
              </strong>
              <span>
                Inspection is read-only. Recovery never replays commits or
                hooks.
              </span>
              <button
                disabled={!workspace || busy}
                onClick={() => void act(historyList)}
              >
                Inspect operation results
              </button>
              {recoveryNotices
                .filter((n) => n.workspace_id !== workspace)
                .map((n) => (
                  <button
                    key={n.operation_id}
                    disabled={busy || writeBusy}
                    onClick={() =>
                      void act(() => switchWorkspace(n.workspace_id))
                    }
                  >
                    Open affected workspace ·{" "}
                    {workspaces.find((w) => w.id === n.workspace_id)?.name ??
                      n.workspace_id}
                  </button>
                ))}
            </div>
          )}
          {error && (
            <div role="alert" className="error">
              <span>{error}</span>
              <button aria-label="Dismiss error" onClick={() => setError("")}>
                ×
              </button>
            </div>
          )}
          {showBulk && workspace ? (
            <BulkCommit
              revision={resultsRevision}
              key={workspace}
              bridge={bridge}
              workspace={workspace}
              entries={entries}
              selected={nav.selected}
              drafts={nav.drafts}
              composer={bulkComposer}
              setComposer={setBulkComposer}
              setDraft={(id, text) =>
                setNav((current) => ({
                  ...current,
                  drafts: { ...current.drafts, [id]: text },
                }))
              }
              setSelected={(ids) =>
                setNav((current) => ({ ...current, selected: ids }))
              }
              enabled={writesEnabled}
              busy={busy || writeBusy}
              act={act}
            />
          ) : showActivity && workspace ? (
            <WorkspaceActivity
              key={workspace}
              bridge={bridge}
              workspace={workspace}
              entries={entries}
              act={act}
              openCommit={(id, commit) => {
                const e = entries.find((e) => e.id === id);
                if (e) {
                  setNav(
                    updateTab(openTab(nav, e), id, {
                      view: "history",
                      commit,
                      parent: null,
                    }),
                  );
                  setShowActivity(false);
                  setShowBulk(false);
                }
              }}
            />
          ) : showOperations ? (
            workspace && (
              <OperationHistory
                key={workspace}
                bridge={bridge}
                workspace={workspace}
                entries={entries}
                busy={busy}
                act={act}
                revision={resultsRevision}
              />
            )
          ) : !nav.active_tab ? (
            <>
              {workspace && (
                <PushDependencyPlanner
                  enabled={writesEnabled}
                  key={`${workspace}:${resultsRevision}`}
                  bridge={bridge}
                  workspace={workspace}
                  entries={entries}
                  busy={busy}
                  act={act}
                />
              )}
              <div className="page-title">
                <div>
                  <span className="eyebrow">YOUR PROJECT, IN VIEW</span>
                  <h1>Workspace Overview</h1>
                  <p>
                    {workspace
                      ? "A local view across independent repositories."
                      : "Open a folder to discover your repositories."}
                  </p>
                </div>
                <button
                  disabled={!workspace || busy}
                  onClick={() =>
                    void act(async () => {
                      if (workspace) {
                        await bridge.request({ kind: "scan", workspace });
                        await inventory(workspace);
                      }
                    })
                  }
                >
                  ↻ Rescan
                </button>
              </div>
              <div className="metrics">
                <Metric
                  label="Repositories"
                  count={entries.length}
                  symbol="▣"
                />
                <Metric label="With changes" count={counts.dirty} symbol="◫" />
                <Metric
                  label="Selected"
                  count={nav.selected.length}
                  symbol="✓"
                />
                <Metric
                  label="Need attention"
                  count={counts.issues}
                  symbol="△"
                  warning
                />
              </div>
              <div className="toolbar">
                <div className="segmented" aria-label="Overview layout">
                  <button
                    aria-pressed={nav.layout === "table"}
                    className={nav.layout === "table" ? "active" : ""}
                    onClick={() => setNav({ ...nav, layout: "table" })}
                  >
                    Table
                  </button>
                  <button
                    aria-pressed={nav.layout === "tree"}
                    className={nav.layout === "tree" ? "active" : ""}
                    onClick={() => setNav({ ...nav, layout: "tree" })}
                  >
                    Tree
                  </button>
                </div>
                <input
                  className="search"
                  aria-label="Filter repositories"
                  placeholder="Search repositories, paths, branches…"
                  value={nav.filter}
                  onChange={(e) => setNav({ ...nav, filter: e.target.value })}
                />
                <select
                  aria-label="Repository attention filter"
                  value={attention}
                  onChange={(e) => setAttention(e.target.value)}
                >
                  <option value="all">All repositories</option>
                  <option value="modified">With changes</option>
                  <option value="issues">Needs attention</option>
                </select>
                <select
                  aria-label="Sort repositories"
                  value={nav.sort}
                  onChange={(e) => setNav({ ...nav, sort: e.target.value })}
                >
                  <option value="path">Path</option>
                  <option value="changes">Changes</option>
                </select>
              </div>
              <div className="selection-bar">
                <span>{nav.selected.length} explicitly selected</span>
                <button
                  disabled={
                    !writesEnabled ||
                    !nav.selected.length ||
                    nav.selected.length > 20 ||
                    busy ||
                    writeBusy
                  }
                  onClick={() => void act(() => trustSelected(nav.selected))}
                >
                  Review trust for selected
                </button>
                <button
                  onClick={() => void act(refreshTrust)}
                  disabled={!workspace || busy}
                >
                  Inspect trust status
                </button>
              </div>
              {nav.layout === "table" ? (
                <div className="table-wrap">
                  <table>
                    <thead>
                      <tr>
                        <th scope="col">Select</th>
                        <th scope="col">Repository / relative path</th>
                        <th scope="col">Branch / HEAD</th>
                        <th scope="col">Staged</th>
                        <th scope="col">Unstaged</th>
                        <th scope="col">Untracked</th>
                        <th scope="col">Last commit</th>
                        <th scope="col">Tracking</th>
                        <th scope="col">Trust / status</th>
                      </tr>
                    </thead>
                    <tbody>
                      {visible.map((e) => (
                        <tr key={e.id}>
                          <td>
                            <input
                              type="checkbox"
                              aria-label={`Select ${relative(e, root)}`}
                              checked={nav.selected.includes(e.id)}
                              onChange={() =>
                                setNav(toggleSelection(nav, e.id))
                              }
                            />
                          </td>
                          <td>
                            <button
                              className="repo-name"
                              onClick={() => setNav(openTab(nav, e))}
                            >
                              {repoName(e)}
                            </button>
                            <small className="table-path">
                              {relative(e, root)}
                            </small>
                          </td>
                          <td>
                            <span className="branch">
                              ⑂{" "}
                              {e.status?.branch?.replace("refs/heads/", "") ??
                                (e.status?.detached
                                  ? "Detached HEAD"
                                  : "Unavailable")}
                            </span>
                          </td>
                          <td>{e.status?.counts.staged ?? "—"}</td>
                          <td>{e.status?.counts.unstaged ?? "—"}</td>
                          <td>{e.status?.counts.untracked ?? "—"}</td>
                          <td>
                            <span>
                              {e.status?.last_commit
                                ? bytes(e.status.last_commit.message).split(
                                    "\n",
                                  )[0]
                                : "—"}
                            </span>
                            <small className="table-path">
                              {e.status
                                ? `Observed ${new Date(e.status.observed_at_ms).toLocaleTimeString()}`
                                : "Not observed"}
                            </small>
                          </td>
                          <td>
                            <span>
                              {e.status?.tracking
                                ? `↑ ${e.status.tracking.ahead ?? "?"} ↓ ${e.status.tracking.behind ?? "?"}`
                                : "—"}
                            </span>
                            {e.status?.tracking && (
                              <small className="table-path">
                                Local tracking refs
                              </small>
                            )}
                          </td>
                          <td>
                            <span
                              className={`badge ${e.availability !== "available" || e.status?.counts.conflicted ? "warning" : e.status && e.status.counts.staged + e.status.counts.unstaged + e.status.counts.untracked > 0 ? "modified" : "success"}`}
                            >
                              {e.availability !== "available"
                                ? e.availability
                                : e.status?.counts.conflicted
                                  ? "Conflicted"
                                  : e.status &&
                                      e.status.counts.staged +
                                        e.status.counts.unstaged +
                                        e.status.counts.untracked >
                                        0
                                    ? "Changes"
                                    : "Clean"}
                            </span>
                            <small className="table-path">
                              {trust.find((t) => t.worktree === e.id)
                                ?.validity ?? "Trust not inspected"}
                            </small>
                            {e.issue && (
                              <small className="issue">{e.issue.message}</small>
                            )}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                  {visible.length === 0 && (
                    <div className="empty">
                      <h2>
                        {workspace
                          ? "No matching repositories"
                          : "Your workspace starts here"}
                      </h2>
                      <p>
                        {workspace
                          ? "Adjust filters or rescan the selected root."
                          : "Choose a project folder. Discovery and monitoring do not modify Git repositories."}
                      </p>
                      {!workspace && (
                        <button
                          className="primary"
                          onClick={() => void act(create)}
                        >
                          Open workspace folder…
                        </button>
                      )}
                    </div>
                  )}
                </div>
              ) : (
                <div className="tree">
                  {buildTree(visible, root).map((node) => (
                    <Tree
                      key={node.key}
                      node={node}
                      selected={nav.selected}
                      open={(e) => setNav(openTab(nav, e))}
                      select={(id) => setNav(toggleSelection(nav, id))}
                    />
                  ))}
                </div>
              )}
            </>
          ) : entry && tab ? (
            <Repository
              key={`${workspace}/${entry.id}`}
              bridge={bridge}
              workspace={workspace!}
              entry={entry}
              nav={nav}
              setNav={setNav}
              act={act}
              busy={busy || writeBusy}
              propose={propose}
              trust={selectedTrust}
              inspectTrust={refreshTrust}
              trustRepo={() => trustSelected([entry.id])}
              resultsRevision={resultsRevision}
              writeEnabled={writesEnabled}
            />
          ) : (
            <div className="empty">
              <h1>Repository unavailable</h1>
              <p>
                This tab is preserved. Rescan or resolve its root; no other
                repository is substituted.
              </p>
            </div>
          )}
        </main>
      </div>
      <footer>
        <span className="monitor">
          <span className="dot clean" />
          {monitor}
        </span>
        <span>
          {closing
            ? "Saving drafts and closing…"
            : busy
              ? "Core request in progress"
              : writeBusy
                ? "Controlled write in progress"
                : !writesEnabled
                  ? "Read-only desktop · native write acceptance pending"
                  : "Local inspection · no automatic remote operations"}
        </span>
        <button
          onClick={() =>
            void act(async () => {
              await bridge.cancel();
              setWriteBusy(false);
            })
          }
          disabled={!busy && !writeBusy}
        >
          Cancel current operation
        </button>
        <span>{entries.length} repositories</span>
      </footer>
    </div>
  );
}
function Metric({
  label,
  count,
  symbol,
  warning = false,
}: {
  label: string;
  count: number;
  symbol: string;
  warning?: boolean;
}) {
  return (
    <div className={`metric ${warning ? "warning" : ""}`}>
      <span className="metric-symbol">{symbol}</span>
      <div>
        <strong>{count}</strong>
        <span>{label}</span>
      </div>
    </div>
  );
}
function Tree({
  node,
  selected,
  open,
  select,
}: {
  node: TreeNode;
  selected: string[];
  open: (e: StoredWorktree) => void;
  select: (id: string) => void;
}) {
  return (
    <div className="tree-node">
      {node.children.length ? (
        <details open>
          <summary>{node.label}</summary>
          {node.entry && (
            <div className="tree-repo">
              <input
                type="checkbox"
                aria-label={`Select ${node.label}`}
                checked={selected.includes(node.entry.id)}
                onChange={() => select(node.entry!.id)}
              />
              <button onClick={() => open(node.entry!)}>
                {repoName(node.entry)}
              </button>
            </div>
          )}
          {node.children.map((n) => (
            <Tree
              key={n.key}
              node={n}
              selected={selected}
              open={open}
              select={select}
            />
          ))}
        </details>
      ) : node.entry ? (
        <div className="tree-repo">
          <input
            type="checkbox"
            aria-label={`Select ${node.label}`}
            checked={selected.includes(node.entry.id)}
            onChange={() => select(node.entry!.id)}
          />
          <button onClick={() => open(node.entry!)}>▣ {node.label}</button>
          <span className="muted">
            {node.entry.status?.branch ?? node.entry.availability}
          </span>
        </div>
      ) : (
        <span>{node.label}</span>
      )}
    </div>
  );
}
function Repository({
  bridge,
  workspace,
  entry,
  nav,
  setNav,
  act,
  busy,
  propose,
  trust,
  inspectTrust,
  trustRepo,
  writeEnabled,
  resultsRevision,
}: {
  bridge: Bridge;
  workspace: string;
  entry: StoredWorktree;
  nav: DesktopState;
  setNav: (n: DesktopState) => void;
  act: (f: () => Promise<void>) => Promise<void>;
  busy: boolean;
  propose: (r: WriteRequest) => Promise<void>;
  trust?: TrustState;
  inspectTrust: () => Promise<void>;
  trustRepo: () => Promise<void>;
  writeEnabled: boolean;
  resultsRevision: number;
}) {
  const tab = nav.tabs.find((t) => t.worktree === entry.id)!;
  const [historyRefresh, setHistoryRefresh] = useState(0);
  const [patch, setPatch] = useState<Diff | null>(null);
  const [preview, setPreview] = useState("");
  const [selection, setSelection] = useState<
    Map<string, { path: NativePath; source: Source }>
  >(new Map());
  const [policyOverride, setPolicy] = useState<"execute" | "bypass_all" | null>(
    null,
  );
  const policy = policyOverride ?? trust?.policy ?? "execute";
  const [commitDetail, setCommitDetail] = useState<CommitDetail | null>(null);
  const [signing, setSigning] = useState<"configured" | "unsigned">(
    "configured",
  );
  const area = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (area.current) area.current.scrollTop = tab.scroll;
  }, [entry.id, tab.scroll]);
  const selectedPath = JSON.stringify(tab.path);
  const observedUntracked =
    entry.status?.changes.some(
      (c) => JSON.stringify(c.path) === selectedPath && c.index === "?",
    ) ?? false;
  const inspectionContext = JSON.stringify([
    tab.view,
    tab.view === "history"
      ? [tab.commit, tab.parent]
      : [tab.path, tab.source, observedUntracked],
  ]);
  const [inspectedContext, setInspectedContext] = useState(inspectionContext);
  // Discard the previous selection's content before rendering the new context.
  if (inspectedContext !== inspectionContext) {
    setInspectedContext(inspectionContext);
    setPatch(null);
    setPreview("");
    setCommitDetail(null);
  }
  useEffect(() => {
    if (tab.view !== "changes" || !tab.path) return;
    let live = true;
    const path = JSON.parse(selectedPath) as NativePath;
    void act(async () => {
      const request = observedUntracked
        ? bridge.request({
            kind: "preview",
            workspace,
            worktree: entry.id,
            path,
          })
        : bridge.request({
            kind: "diff",
            workspace,
            worktree: entry.id,
            mode: { kind: tab.source === "index" ? "staged" : "unstaged" },
            path,
          });
      await request
        .then((data) => {
          if (!live) return;
          if (data.kind === "preview") {
            setPatch(null);
            setPreview(
              data.value.binary
                ? "Binary untracked file"
                : `Untracked working-file preview\n${bytes(data.value.bytes)}`,
            );
          } else {
            setPatch(value(data, "diff"));
            setPreview("");
          }
        })
        .catch((e) => {
          if (live) setPreview(String(e));
        });
    });
    return () => {
      live = false;
    };
  }, [
    bridge,
    workspace,
    entry.id,
    tab.view,
    tab.path,
    selectedPath,
    tab.source,
    observedUntracked,
    act,
  ]);
  useEffect(() => {
    if (tab.view !== "history" || !tab.commit) return;
    const oid = tab.commit;
    let live = true;
    void act(async () => {
      await bridge
        .request({
          kind: "commit",
          workspace,
          worktree: entry.id,
          oid,
        })
        .then(async (data) => {
          const commit = value(data, "commit");
          if (!live) return;
          setCommitDetail(commit);
          setPreview(
            `${bytes(commit.message)}\n\nAuthor: ${bytes(commit.author)}\nParents: ${commit.parents.map((p) => p.hex).join(", ") || "Root commit"}`,
          );
          const parent = tab.parent
            ? commit.parents.find((p) => p.hex === tab.parent)
            : commit.parents[0];
          if (tab.parent && !parent)
            throw new Error("Saved comparison parent is no longer valid");
          const diff = value(
            await bridge.request({
              kind: "diff",
              workspace,
              worktree: entry.id,
              mode: {
                kind: "historical",
                commit: commit.oid,
                parent: parent ?? null,
              },
            }),
            "diff",
          );
          if (live) setPatch(diff);
        })
        .catch((e) => {
          if (live) setPreview(String(e));
        });
    });
    return () => {
      live = false;
    };
  }, [bridge, workspace, entry.id, tab.view, tab.commit, tab.parent, act]);
  async function diff(path: NativePath, source: Source) {
    setNav(updateTab(nav, entry.id, { path, source, scroll: 0 }));
  }
  function select(change: Change, source: Source) {
    const key = `${JSON.stringify(change.path)}:${source}`;
    const next = new Map(selection);
    if (next.has(key)) next.delete(key);
    else {
      for (const [k, v] of next)
        if (JSON.stringify(v.path) === JSON.stringify(change.path))
          next.delete(k);
      next.set(key, { path: change.path, source });
    }
    setSelection(next);
  }
  function request(kind: "stage" | "unstage" | "commit"): WriteRequest {
    const selected = selectedChanges;
    return kind === "commit"
      ? {
          kind: "commit",
          selected,
          message: nav.drafts[entry.id] ?? "",
          hooks: policy,
          signing,
        }
      : { kind, paths: selected.map((s) => s.path), hooks: policy };
  }
  const writeBlocked = busy || !writeEnabled;
  const changes = entry.status?.changes ?? [];
  const selectedChanges = [...selection.values()].filter((selected) =>
    changes.some(
      (change) =>
        JSON.stringify(change.path) === JSON.stringify(selected.path) &&
        (selected.source === "index"
          ? ![".", "?", "!"].includes(change.index)
          : change.worktree !== "." || change.index === "?"),
    ),
  );
  const changeContext = JSON.stringify(changes);
  const [selectedChangeContext, setSelectedChangeContext] =
    useState(changeContext);
  if (selectedChangeContext !== changeContext) {
    setSelectedChangeContext(changeContext);
    if (selectedChanges.length !== selection.size) {
      const keys = new Set(
        selectedChanges.map((s) => `${JSON.stringify(s.path)}:${s.source}`),
      );
      setSelection(new Map([...selection].filter(([key]) => keys.has(key))));
    }
  }
  const draft = nav.drafts[entry.id] ?? "";
  return (
    <section className="repository">
      <div className="page-title">
        <div>
          <span className="eyebrow">REPOSITORY</span>
          <h1>
            {repoName(entry)}{" "}
            <span className="badge modified">
              {entry.status?.branch?.replace("refs/heads/", "") ??
                "Detached / unavailable"}
            </span>
          </h1>
          <p className="path">{pathText(entry.display_path)}</p>
        </div>
        <button
          disabled={busy}
          onClick={() =>
            void act(async () => {
              await bridge.request({
                kind: "refresh",
                workspace,
                worktree: entry.id,
              });
              setHistoryRefresh(historyRefresh + 1);
            })
          }
        >
          ↻ Local refresh
        </button>
      </div>
      <RemoteInspector
        revision={resultsRevision}
        bridge={bridge}
        workspace={workspace}
        worktree={entry.id}
        busy={busy}
        act={act}
        enabled={writeEnabled}
      />
      <div className="repository-tabs">
        <button
          aria-pressed={tab.view === "changes"}
          className={tab.view === "changes" ? "active" : ""}
          onClick={() => setNav(updateTab(nav, entry.id, { view: "changes" }))}
        >
          Changes <span>{changes.length}</span>
        </button>
        <button
          aria-pressed={tab.view === "history"}
          className={tab.view === "history" ? "active" : ""}
          onClick={() => setNav(updateTab(nav, entry.id, { view: "history" }))}
        >
          History
        </button>
      </div>
      <div className="trust-inspection">
        <button onClick={() => void act(inspectTrust)}>
          Inspect trust & eligibility
        </button>
        <span className="muted">
          {trust ? `${trust.grant} · ${trust.validity}` : "Trust not inspected"}
        </span>
      </div>
      {entry.issue && (
        <p role="alert" className="error">
          {entry.issue.message}
        </p>
      )}
      {tab.view === "changes" ? (
        <div
          className="changes-grid"
          style={
            { "--inspector-width": `${nav.inspector_width}px` } as CSSProperties
          }
        >
          <div className="files">
            <div className="section-title">
              <h2>Changed files</h2>
              <span>{selectedChanges.length} selected</span>
            </div>
            {(["index", "working_tree"] as const).map((source) => (
              <div className="file-group" key={source}>
                <h3>
                  {source === "index"
                    ? "Staged · HEAD ↔ index"
                    : "Unstaged / untracked · index ↔ worktree"}
                </h3>
                {changes
                  .filter((c) =>
                    source === "index"
                      ? c.index !== "." && c.index !== "?" && c.index !== "!"
                      : c.worktree !== "." || c.index === "?",
                  )
                  .map((c, i) => (
                    <div className="file-row" key={`${source}/${i}`}>
                      <input
                        type="checkbox"
                        aria-label={`Include ${pathText(c.path)} from ${source}`}
                        disabled={c.conflict || busy}
                        checked={selection.has(
                          `${JSON.stringify(c.path)}:${source}`,
                        )}
                        onChange={() => select(c, source)}
                      />
                      <button
                        onClick={() => void act(() => diff(c.path, source))}
                      >
                        {pathText(c.path)}
                      </button>
                      <span
                        className={`file-status ${c.conflict ? "bad" : ""}`}
                      >
                        {c.conflict
                          ? "!"
                          : source === "index"
                            ? c.index
                            : c.worktree === "."
                              ? "?"
                              : c.worktree}
                      </span>
                    </div>
                  ))}
              </div>
            ))}
            {!changes.length && <p className="muted">No observed changes.</p>}
            <div className="file-actions">
              <button
                disabled={writeBlocked || !selectedChanges.length}
                onClick={() => void act(() => propose(request("stage")))}
              >
                Review stage
              </button>
              <button
                disabled={writeBlocked || !selectedChanges.length}
                onClick={() => void act(() => propose(request("unstage")))}
              >
                Review unstage
              </button>
            </div>
          </div>
          <div className="diff-panel">
            <div className="section-title">
              <h2>
                {tab.path ? pathText(tab.path) : "Select a file to inspect"}
              </h2>
              {patch && <span className="badge">{patch.mode.kind}</span>}
            </div>
            <div
              className="patch"
              ref={area}
              onScroll={(e) => {
                if (Math.abs(e.currentTarget.scrollTop - tab.scroll) > 100)
                  setNav(
                    updateTab(nav, entry.id, {
                      scroll: e.currentTarget.scrollTop,
                    }),
                  );
              }}
            >
              {patch ? (
                <pre>{bytes(patch.patch)}</pre>
              ) : (
                <div className="empty">
                  <p>Diffs are read through restricted Core services.</p>
                  <p>
                    Binary contents, mode changes and gitlinks are reported
                    explicitly.
                  </p>
                </div>
              )}
              {preview && <pre>{preview}</pre>}
            </div>
            {tab.path && (
              <button
                onClick={() =>
                  void act(async () => {
                    const p = value(
                      await bridge.request({
                        kind: "preview",
                        workspace,
                        worktree: entry.id,
                        path: tab.path!,
                      }),
                      "preview",
                    );
                    setPreview(
                      p.binary
                        ? "Binary file — text preview unavailable"
                        : bytes(p.bytes),
                    );
                  })
                }
              >
                Read bounded working-file preview
              </button>
            )}
          </div>
          <aside className="composer">
            <div className="section-title">
              <h2>Commit composer</h2>
            </div>
            <label>
              Inspector width
              <input
                type="range"
                min={240}
                max={800}
                step={20}
                value={nav.inspector_width}
                onChange={(e) =>
                  setNav({ ...nav, inspector_width: Number(e.target.value) })
                }
              />
            </label>
            <label>
              Message
              <textarea
                rows={8}
                value={draft}
                maxLength={512 * 1024}
                placeholder={"Describe the change\n\nOptional explanation…"}
                onChange={(e) =>
                  setNav({
                    ...nav,
                    drafts: { ...nav.drafts, [entry.id]: e.target.value },
                  })
                }
              />
            </label>
            <label>
              Hooks
              <select
                value={policy}
                onChange={(e) => setPolicy(e.target.value as typeof policy)}
              >
                <option value="execute">Execute supported hooks</option>
                <option value="bypass_all">Bypass all — explicit review</option>
              </select>
            </label>
            <button
              className="subtle"
              disabled={writeBlocked}
              onClick={() =>
                void act(async () => {
                  await bridge.request({
                    kind: "hook_policy",
                    workspace,
                    worktree: entry.id,
                    policy,
                  });
                  await inspectTrust();
                })
              }
            >
              Remember hook preference
            </button>
            <label>
              Signing
              <select
                value={signing}
                onChange={(e) => setSigning(e.target.value as typeof signing)}
              >
                <option value="configured">Use configured signing</option>
                <option value="unsigned">Explicitly unsigned</option>
              </select>
            </label>
            <p className="review-note">
              The next step opens an authoritative Rust review. A separate
              native approval is required.
            </p>
            <button
              className="primary"
              disabled={
                writeBlocked || !selectedChanges.length || !draft.trim()
              }
              onClick={() => void act(() => propose(request("commit")))}
            >
              Review commit · {selectedChanges.length} files
            </button>
            <div className="trust-card">
              <h3>Trust & execution</h3>
              <p>
                {trust?.reason ??
                  "Trust is checked again by Core for every operation."}
              </p>
              <button
                disabled={writeBlocked}
                onClick={() => void act(trustRepo)}
              >
                Review trust
              </button>
              <button
                disabled={writeBlocked}
                onClick={() =>
                  void act(async () => {
                    await bridge.request({
                      kind: "revoke",
                      workspace,
                      worktree: entry.id,
                    });
                    await inspectTrust();
                  })
                }
              >
                Revoke trust
              </button>
              <p>
                Trusted hooks and signers run as your user. They are not
                sandboxed.
              </p>
            </div>
          </aside>
        </div>
      ) : (
        <div className="history-grid">
          <HistoryGraph
            key={`${workspace}/${entry.id}/${tab.filter}/${historyRefresh}`}
            bridge={bridge}
            act={act}
            workspace={workspace}
            worktree={entry.id}
            selected={tab.commit}
            reference={tab.filter}
            onReference={(filter) =>
              setNav(updateTab(nav, entry.id, { filter }))
            }
            onSelect={(commit) =>
              setNav(updateTab(nav, entry.id, { commit, parent: null }))
            }
          />
          <div className="history-inspector">
            <pre>
              {preview ||
                "Select a commit for its exact message, parents and changes."}
            </pre>
            {patch && (
              <>
                <div className="section-title">
                  <h2>Files in selected parent comparison</h2>
                </div>
                {patch.files.map((f, i) => (
                  <button
                    key={i}
                    onClick={() =>
                      setNav(updateTab(nav, entry.id, { path: f.path }))
                    }
                  >
                    {pathText(f.path)} · {f.change}
                  </button>
                ))}
                <pre>{bytes(patch.patch)}</pre>
              </>
            )}
            {commitDetail?.parents.map((p, i) => (
              <button
                key={p.hex}
                onClick={() =>
                  setNav(updateTab(nav, entry.id, { parent: p.hex }))
                }
              >
                Compare parent {i + 1} · {p.hex.slice(0, 8)}
              </button>
            ))}
          </div>
        </div>
      )}
    </section>
  );
}
