import { AppPreferencesProvider } from "./AppPreferences";
import {
  changeRows,
  actionSelection,
  pathKey,
} from "../stores/changeSelection";
import { writeFeedback } from "../services/writeFeedback";
import { WorkingStatus } from "./WorkingStatus";
import { FileStatus } from "./FileStatus";
import { WorkspaceTabs } from "./WorkspaceTabs";
import { SettingsLauncher } from "./SettingsLauncher";
import { RepositoryIcon } from "./RepositoryIcon";
import { ThemeToggle } from "./ThemeToggle";
import { CommitMessageEditor } from "./CommitMessageEditor";
import { AiCommitFill } from "./AiCommitFill";
import { AiSettings } from "./AiSettings";
import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import type { Bridge } from "../services/bridge";
import { ApplicationBrand } from "./ApplicationBrand";
import { PushDependencyPlanner } from "./PushDependencyPlanner";
import { value } from "../services/bridge";
import type {
  Workspace,
  StoredWorktree,
  DesktopState,
  BulkComposerState,
  Diff,
  NativePath,
  Commit as CommitDetail,
  RecoveryNotice,
  ScanSummary,
  RemoteRecoveryNotice,
  PushBatch,
} from "../services/response.generated";
import type {
  ContentSource,
  WriteRequest,
} from "../services/request.generated";
import {
  initialState,
  initialBulkComposer,
  bytes,
  pathText,
  relative,
  repoName,
  toggleSelection,
  openTab,
  updateTab,
  filtered,
  overviewStatus,
  attentionChoices,
  matchesAttention,
  collapseTree,
  sortChoices,
  sortChoice,
  toggleSort,
  type SortColumn,
  buildTree,
  type TreeNode,
} from "../stores/model";
import { WorkspaceActivity } from "./WorkspaceActivity";
import { HistoryGraph } from "./HistoryGraph";
import { BulkCommit } from "./BulkCommit";
import { scanIsContinuing, WorkspaceScanStatus } from "./WorkspaceScanStatus";
import { RepositoryIdentityRecovery } from "./RepositoryIdentityRecovery";
import { RepositoryRemoteStatus } from "./RepositoryRemoteStatus";
import { RemoteInspector } from "./RemoteInspector";
import { OperationHistory } from "./OperationHistory";
export function App({ bridge }: { bridge: Bridge }) {
  return (
    <AppPreferencesProvider bridge={bridge}>
      <AppContent bridge={bridge} />
    </AppPreferencesProvider>
  );
}
function AppContent({ bridge }: { bridge: Bridge }) {
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [workspace, setWorkspace] = useState<string | null>(null);
  const [scan, setScan] = useState<ScanSummary | null>(null);
  const [allEntries, setEntries] = useState<StoredWorktree[]>([]);
  const entries = allEntries.filter((e) => !e.superseded_by);
  const [nav, setNav] = useState<DesktopState>(initialState);
  const [uiScale, setUiScale] = useState(100);
  const [bulkComposer, setBulkComposer] =
    useState<BulkComposerState>(initialBulkComposer);
  const [pendingActions, setPendingActions] = useState(0);
  const busy = pendingActions > 0;
  const [error, setError] = useState("");
  const [writeNotice, setWriteNotice] = useState<{
    message: string;
    workspace: string;
    worktree: string;
  } | null>(null);
  const [closing, setClosing] = useState(false);
  const [loading, setLoading] = useState(true);
  const [monitor, setMonitor] = useState("Not monitoring");
  const attention = nav.attention ?? "all";
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
  const [publicationRecovery, setPublicationRecovery] = useState<
    RemoteRecoveryNotice[]
  >([]);
  const [batchRecovery, setBatchRecovery] = useState<RemoteRecoveryNotice[]>(
    [],
  );
  const [versionRecovery, setVersionRecovery] = useState<
    RemoteRecoveryNotice[]
  >([]);
  const troubleshooting = useRef<HTMLDetailsElement>(null);
  const [guidedPushBatch, setGuidedPushBatch] = useState<PushBatch | null>(
    null,
  );
  const recoveryNotices = [
    ...recovery,
    ...importRecovery,
    ...upstreamRecovery,
    ...publicationRecovery,
    ...batchRecovery,
    ...versionRecovery,
  ];
  const context = useRef<string | null>(null);
  const contextRevision = useRef(0);
  const draftRevisions = useRef(new Map<string, number>());
  const updateDraft = useCallback(
    (id: string, message: string, scope: string | null) => {
      if (context.current !== scope) return;
      draftRevisions.current.set(id, (draftRevisions.current.get(id) ?? 0) + 1);
      setNav((current) => ({
        ...current,
        drafts: { ...current.drafts, [id]: message },
      }));
    },
    [],
  );
  function beginDraftUpdate(id: string, scope: string | null) {
    let revision = draftRevisions.current.get(id) ?? 0;
    const generation = contextRevision.current;
    return (message: string) => {
      if (
        context.current !== scope ||
        contextRevision.current !== generation ||
        (draftRevisions.current.get(id) ?? 0) !== revision
      )
        return false;
      updateDraft(id, message, scope);
      revision = draftRevisions.current.get(id) ?? 0;
      return true;
    };
  }
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
  const entry = allEntries.find((e) => e.id === nav.active_tab);
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
      const currentIds = new Set(
        data.entries
          .filter((entry) => !entry.superseded_by)
          .map((entry) => entry.id),
      );
      setNav((current) => {
        const selected = current.selected.filter((id) => currentIds.has(id));
        return selected.length === current.selected.length
          ? current
          : { ...current, selected };
      });
      setScan(data.scan ?? null);
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
    setPublicationRecovery(data.push_publication_recovery ?? []);
    setBatchRecovery(data.push_batch_recovery ?? []);
    setVersionRecovery(data.version_action_recovery ?? []);
    const ws = data.active ?? null;
    if (context.current !== ws) {
      loaded.current = false;
      context.current = ws;
      contextRevision.current++;
      setWorkspace(ws);
      setBulkComposer(initialBulkComposer);
      setEntries([]);
      setScan(null);
      setMonitor("Not monitoring");
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
    let live = true;
    let unlisten: (() => void) | undefined;
    void bridge
      .onThemeChanged?.((theme) => {
        if (live) setNav((current) => ({ ...current, theme }));
      })
      .then((stop) => {
        if (live) unlisten = stop;
        else stop();
      })
      .catch((e) => setError(String(e)));
    return () => {
      live = false;
      unlisten?.();
    };
  }, [bridge]);
  useEffect(() => {
    bridge
      .capabilities()
      .then((c) => {
        setWritesEnabled(c.writes_enabled);
        setUiScale(c.ui_scale_percent ?? 100);
      })
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
        .then(() => {
          if (context.current === workspace)
            setError((current) =>
              current.startsWith("Layout and drafts were not saved:")
                ? ""
                : current,
            );
        })
        .catch((e) => {
          if (context.current === workspace)
            setError(
              `Layout and drafts were not saved: ${e instanceof Error ? e.message : String(e)}`,
            );
        });
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
      contextRevision.current++;
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
      contextRevision.current++;
      await load();
    } finally {
      setLoading(false);
    }
  }
  async function propose(
    request: WriteRequest,
    fileCount: number,
    owner?: symbol,
  ) {
    if (!workspace || !entry) return false;
    const revision = draftRevisions.current.get(entry.id) ?? 0;
    const generation = contextRevision.current;
    setWriteNotice(null);
    const response = await bridge.request(
      {
        kind: "propose",
        workspace,
        worktree: entry.id,
        request,
      },
      owner,
    );
    const feedback = writeFeedback(response, request, fileCount);
    if (feedback.error) throw new Error(feedback.message);
    if (
      request.kind === "commit" &&
      feedback.completed &&
      context.current === workspace &&
      contextRevision.current === generation &&
      (draftRevisions.current.get(entry.id) ?? 0) === revision
    )
      updateDraft(entry.id, "", workspace);
    setWriteNotice({
      message: feedback.message,
      workspace,
      worktree: entry.id,
    });
    return feedback.completed === true;
  }
  async function historyList() {
    setShowOperations(true);
    setShowActivity(false);
    setShowBulk(false);
  }
  const visible = filtered(entries, nav.filter, attention, nav.sort);
  const counts = entries.reduce(
    (s, e) => ({
      dirty: s.dirty + Number(matchesAttention(e, "modified")),
      issues: s.issues + Number(matchesAttention(e, "issues")),
    }),
    { dirty: 0, issues: 0 },
  );
  return (
    <div
      className="shell"
      data-density={nav.density ?? "comfortable"}
      inert={closing || loading}
    >
      <header className="topbar">
        <ApplicationBrand bridge={bridge} />
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
        {bridge.openSettings ? (
          <SettingsLauncher
            disabled={busy || writeBusy}
            open={() =>
              bridge.openSettings!({
                workspace,
                worktree: entry?.id,
                theme: nav.theme,
              })
            }
          />
        ) : (
          <AiSettings
            bridge={bridge}
            workspace={workspace}
            worktree={entry?.id}
            disabled={busy || writeBusy}
            uiScale={uiScale}
            onUiScaleChange={
              bridge.setUiScale
                ? async (percent) => {
                    const applied = await bridge.setUiScale!(percent);
                    setUiScale(applied);
                  }
                : undefined
            }
            theme={nav.theme}
            onThemeChange={(theme) =>
              setNav((current) => ({ ...current, theme }))
            }
          />
        )}
        <ThemeToggle
          theme={nav.theme}
          onChange={(theme) => setNav((current) => ({ ...current, theme }))}
        />
      </header>
      <WorkspaceTabs
        nav={nav}
        entries={allEntries}
        onChange={setNav}
        overviewActive={
          !nav.active_tab && !showOperations && !showActivity && !showBulk
        }
        repositoryActive={!showOperations && !showActivity && !showBulk}
        onPathAction={(repository, action) =>
          void act(async () => {
            if (!bridge.repositoryPathAction)
              throw new Error(
                "Repository path actions are unavailable in this build",
              );
            if (!workspace) throw new Error("Workspace is unavailable");
            await bridge.repositoryPathAction(
              { workspace, worktree: repository.id },
              action,
            );
          })
        }
        onSelect={(id) => {
          setNav((current) => ({ ...current, active_tab: id }));
          setShowOperations(false);
          setShowActivity(false);
          setShowBulk(false);
        }}
      />
      <div className="body">
        <aside
          id="workspace-sidebar"
          className="sidebar"
          hidden={nav.sidebar_collapsed}
          style={{ width: nav.sidebar_width }}
        >
          <div className="workspace-heading">
            <span className="eyebrow">ACTIVE WORKSPACE</span>
            <h2>{activeWorkspace?.name ?? "Welcome"}</h2>
            {activeWorkspace ? (
              <button
                className="path repository-folder-link"
                title="Open folder in Finder"
                onClick={() =>
                  void act(async () => {
                    if (!bridge.repositoryPathAction)
                      throw new Error("Finder is unavailable");
                    await bridge.repositoryPathAction(
                      { workspace: activeWorkspace.id },
                      "open",
                    );
                  })
                }
              >
                {root}
              </button>
            ) : (
              <p className="path">Choose a local project folder</p>
            )}
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
            <details className="workspace-settings" key={workspace}>
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
          {workspace && (
            <details
              ref={troubleshooting}
              className="workspace-troubleshooting"
            >
              <summary>Workspace troubleshooting</summary>
              <WorkspaceScanStatus
                scan={scan}
                busy={busy}
                onContinue={() =>
                  void act(async () => {
                    await bridge.request({ kind: "continue_scan", workspace });
                    await inventory(workspace);
                  })
                }
              />
            </details>
          )}
        </aside>
        <div
          className="splitter"
          hidden={nav.sidebar_collapsed}
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
                sidebar_width: Math.round(
                  Math.max(180, Math.min(420, e.clientX)),
                ),
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
              {batchRecovery
                .filter((notice) => notice.workspace_id === workspace)
                .map((notice) => (
                  <button
                    key={notice.operation_id}
                    disabled={busy || writeBusy}
                    onClick={() =>
                      void act(async () => {
                        await bridge.request({
                          kind: "close_interrupted_push_batch",
                          workspace: notice.workspace_id,
                          id: notice.operation_id,
                        });
                        await load();
                      })
                    }
                  >
                    Close interrupted push queue · {notice.operation_id}
                  </button>
                ))}
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
          {writeNotice &&
            writeNotice.workspace === workspace &&
            writeNotice.worktree === entry?.id && (
              <div role="status" className="operation-feedback">
                <span>{writeNotice.message}</span>
                <button
                  aria-label="Dismiss result"
                  onClick={() => setWriteNotice(null)}
                >
                  ×
                </button>
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
          {workspace && scan && !scan.complete && !scanIsContinuing(scan) && (
            <div role="alert" className="error">
              <span>
                <strong>△ Discovery incomplete</strong>
                {scan.issues.length > 0 && ` — ${scan.issues[0].error.message}`}
              </span>
              <button
                onClick={() => {
                  setNav((current) => ({
                    ...current,
                    sidebar_collapsed: false,
                  }));
                  if (troubleshooting.current)
                    troubleshooting.current.open = true;
                }}
              >
                Show scan details
              </button>
            </div>
          )}
          {workspace && (
            <RepositoryIdentityRecovery
              entries={allEntries}
              drafts={nav.drafts}
              busy={busy}
              onRescan={() =>
                act(async () => {
                  await bridge.request({ kind: "scan", workspace });
                  await inventory(workspace);
                })
              }
              onOpen={(replacement) => {
                setNav(openTab(nav, replacement));
                setShowOperations(false);
                setShowActivity(false);
                setShowBulk(false);
              }}
            />
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
              setDraft={(id, text) => updateDraft(id, text, workspace)}
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
              state={nav.activity}
              onStateChange={(activity) =>
                setNav((current) => ({ ...current, activity }))
              }
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
                enabled={writesEnabled}
                key={workspace}
                bridge={bridge}
                workspace={workspace}
                entries={allEntries}
                busy={busy}
                act={act}
                revision={resultsRevision}
              />
            )
          ) : !nav.active_tab ? (
            <>
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
                  symbol={<RepositoryIcon />}
                />
                <Metric label="With changes" count={counts.dirty} symbol="◫" />
                <Metric
                  label="Selected"
                  count={nav.selected.length}
                  symbol="✓"
                />
                <Metric
                  label="Needs attention"
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
                  onChange={(e) => {
                    const choice = attentionChoices.find(
                      (item) => item.value === e.target.value,
                    );
                    if (choice) setNav({ ...nav, attention: choice.value });
                  }}
                >
                  {attentionChoices.map((choice) => (
                    <option key={choice.value} value={choice.value}>
                      {choice.label}
                    </option>
                  ))}
                </select>
                <select
                  aria-label="Sort repositories"
                  value={nav.sort}
                  onChange={(e) => {
                    const choice = sortChoices.find(
                      (item) => item.value === e.target.value,
                    );
                    if (choice) setNav({ ...nav, sort: choice.value });
                  }}
                >
                  {sortChoices
                    .filter((choice) => !choice.value.startsWith("trust"))
                    .map((choice) => (
                      <option key={choice.value} value={choice.value}>
                        {choice.label}
                      </option>
                    ))}
                </select>
                <select
                  aria-label="Overview density"
                  value={nav.density ?? "comfortable"}
                  onChange={(event) => {
                    const density = event.target.value;
                    if (density === "comfortable" || density === "compact")
                      setNav({ ...nav, density });
                  }}
                >
                  <option value="comfortable">Comfortable</option>
                  <option value="compact">Compact</option>
                </select>
              </div>
              <div className="selection-bar">
                <span>{nav.selected.length} explicitly selected</span>
              </div>
              {nav.layout === "table" ? (
                <div className="table-wrap">
                  <table>
                    <thead>
                      <tr>
                        <th scope="col">Select</th>
                        <OverviewHeader
                          column="path"
                          sort={nav.sort}
                          onSort={(sort) => setNav({ ...nav, sort })}
                        >
                          Repository / relative path
                        </OverviewHeader>
                        <OverviewHeader
                          column="branch"
                          sort={nav.sort}
                          onSort={(sort) => setNav({ ...nav, sort })}
                        >
                          Branch / HEAD
                        </OverviewHeader>
                        <OverviewHeader
                          column="staged"
                          sort={nav.sort}
                          onSort={(sort) => setNav({ ...nav, sort })}
                        >
                          Staged
                        </OverviewHeader>
                        <OverviewHeader
                          column="unstaged"
                          sort={nav.sort}
                          onSort={(sort) => setNav({ ...nav, sort })}
                        >
                          Unstaged
                        </OverviewHeader>
                        <OverviewHeader
                          column="untracked"
                          sort={nav.sort}
                          onSort={(sort) => setNav({ ...nav, sort })}
                        >
                          Untracked
                        </OverviewHeader>
                        <OverviewHeader
                          column="conflicts"
                          sort={nav.sort}
                          onSort={(sort) => setNav({ ...nav, sort })}
                        >
                          Conflicts
                        </OverviewHeader>
                        <OverviewHeader
                          column="last_commit"
                          sort={nav.sort}
                          onSort={(sort) => setNav({ ...nav, sort })}
                        >
                          Last commit
                        </OverviewHeader>
                        <OverviewHeader
                          column="ahead"
                          sort={nav.sort}
                          onSort={(sort) => setNav({ ...nav, sort })}
                        >
                          Ahead (cached)
                        </OverviewHeader>
                        <OverviewHeader
                          column="behind"
                          sort={nav.sort}
                          onSort={(sort) => setNav({ ...nav, sort })}
                        >
                          Behind (cached)
                        </OverviewHeader>
                        <th scope="col">Remote status</th>
                        <OverviewHeader
                          column="status"
                          sort={nav.sort}
                          onSort={(sort) => setNav({ ...nav, sort })}
                        >
                          Status
                        </OverviewHeader>
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
                          <td>{e.status?.counts.conflicted ?? "—"}</td>
                          <td>
                            <span>
                              {e.status?.last_commit
                                ? bytes(e.status.last_commit.message).split(
                                    "\n",
                                  )[0]
                                : "—"}
                            </span>
                            {e.status?.last_commit && (
                              <small className="table-path">
                                Committed{" "}
                                {new Date(
                                  e.status.last_commit.committer_time * 1000,
                                ).toLocaleString()}
                              </small>
                            )}
                            <small className="table-path">
                              {e.status
                                ? `Observed ${new Date(e.status.observed_at_ms).toLocaleTimeString()}`
                                : "Not observed"}
                            </small>
                          </td>
                          <td title="Local comparison against cached upstream; not a live server observation">
                            {e.status?.tracking?.ahead ?? "—"}
                          </td>
                          <td title="Local comparison against cached upstream; not a live server observation">
                            {e.status?.tracking?.behind ?? "—"}
                          </td>
                          <td>
                            <RepositoryRemoteStatus snapshot={e.status} />
                          </td>
                          <td>
                            <span
                              className={`badge ${e.availability !== "available" || e.issue || !e.status || e.status.counts.conflicted ? "warning" : e.status && e.status.counts.staged + e.status.counts.unstaged + e.status.counts.untracked > 0 ? "modified" : "success"}`}
                            >
                              {overviewStatus(e) ?? "Not observed"}
                            </span>

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
                      collapsed={nav.collapsed ?? []}
                      collapse={(key, collapsed) =>
                        setNav((current) =>
                          collapseTree(current, key, collapsed),
                        )
                      }
                      open={(e) => setNav(openTab(nav, e))}
                      select={(id) => setNav(toggleSelection(nav, id))}
                    />
                  ))}
                </div>
              )}
            </>
          ) : entry && tab && !entry.superseded_by ? (
            <Repository
              key={`${workspace}/${entry.id}`}
              bridge={bridge}
              workspace={workspace!}
              entry={entry}
              nav={nav}
              setNav={setNav}
              setDraft={(message) => updateDraft(entry.id, message, workspace)}
              beginDraftUpdate={() => beginDraftUpdate(entry.id, workspace)}
              act={act}
              busy={busy || writeBusy}
              propose={propose}
              resultsRevision={resultsRevision}
              writeEnabled={writesEnabled}
              pushPlanning={
                workspace ? (
                  <PushDependencyPlanner
                    enabled={writesEnabled}
                    activeBatch={
                      guidedPushBatch?.plan.workspace === workspace
                        ? guidedPushBatch
                        : null
                    }
                    onBatch={setGuidedPushBatch}
                    revision={resultsRevision}
                    key={`${workspace}:${resultsRevision}`}
                    bridge={bridge}
                    workspace={workspace}
                    entries={entries}
                    busy={busy}
                    act={act}
                  />
                ) : null
              }
            />
          ) : (
            <div className="empty">
              <h1>Repository unavailable</h1>
              <p>
                This tab and its draft are preserved. Review repository identity
                recovery above or rescan; no other repository is substituted.
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
                ? "Git operation in progress"
                : !writesEnabled
                  ? "Git operations unavailable in this build"
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
  symbol: ReactNode;
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
  collapsed,
  collapse,
  open,
  select,
}: {
  node: TreeNode;
  selected: string[];
  collapsed: readonly string[];
  collapse: (key: string, collapsed: boolean) => void;
  open: (e: StoredWorktree) => void;
  select: (id: string) => void;
}) {
  return (
    <div className="tree-node">
      {node.children.length ? (
        <details
          open={!collapsed.includes(node.key)}
          onToggle={(event) => {
            if (event.target === event.currentTarget)
              collapse(node.key, !event.currentTarget.open);
          }}
        >
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
                <RepositoryIcon /> {repoName(node.entry)}
              </button>
            </div>
          )}
          {node.children.map((n) => (
            <Tree
              key={n.key}
              node={n}
              selected={selected}
              collapsed={collapsed}
              collapse={collapse}
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
          <button onClick={() => open(node.entry!)}>
            <RepositoryIcon /> {node.label}
          </button>
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
  setDraft,
  beginDraftUpdate,
  act,
  busy,
  propose,
  writeEnabled,
  resultsRevision,
  pushPlanning,
}: {
  bridge: Bridge;
  workspace: string;
  entry: StoredWorktree;
  nav: DesktopState;
  setNav: (n: DesktopState) => void;
  setDraft: (message: string) => void;
  beginDraftUpdate: () => (message: string) => boolean;
  act: (f: () => Promise<void>) => Promise<void>;
  busy: boolean;
  propose: (
    r: WriteRequest,
    fileCount: number,
    owner?: symbol,
  ) => Promise<boolean>;
  writeEnabled: boolean;
  resultsRevision: number;
  pushPlanning?: ReactNode;
}) {
  const tab = nav.tabs.find((t) => t.worktree === entry.id)!;
  const [historyRefresh, setHistoryRefresh] = useState(0);
  const [staging, setStaging] = useState<{
    kind: "stage" | "unstage";
    owner: symbol;
    count: number;
  } | null>(null);
  const [selection, setSelection] = useState(new Set<string>());
  const [selectionTransfer, setSelectionTransfer] = useState<{
    before: string;
    target: ContentSource;
    paths: Set<string>;
  } | null>(null);
  const [cancellingStage, setCancellingStage] = useState(false);
  const [patch, setPatch] = useState<Diff | null>(null);
  const [preview, setPreview] = useState("");
  const [policyOverride, setPolicy] = useState<"execute" | "bypass_all" | null>(
    null,
  );
  const policy = policyOverride ?? "execute";
  const [commitDetail, setCommitDetail] = useState<CommitDetail | null>(null);
  const [signing, setSigning] = useState<"configured" | "unsigned">(
    "configured",
  );
  const area = useRef<HTMLDivElement>(null);
  const changesArea = useRef<HTMLDivElement>(null);
  const [changesHeight, setChangesHeight] = useState<number>();
  const fitChanges = useCallback(() => {
    const grid = changesArea.current;
    if (!grid) return;
    const bottom =
      document.querySelector("footer")?.getBoundingClientRect().top ??
      window.innerHeight;
    setChangesHeight(
      Math.max(300, Math.floor(bottom - grid.getBoundingClientRect().top - 24)),
    );
  }, []);
  useLayoutEffect(fitChanges);
  useEffect(() => {
    window.addEventListener("resize", fitChanges);
    document.addEventListener("toggle", fitChanges, true);
    return () => {
      window.removeEventListener("resize", fitChanges);
      document.removeEventListener("toggle", fitChanges, true);
    };
  }, [fitChanges]);
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
  async function diff(path: NativePath, source: ContentSource) {
    setNav(updateTab(nav, entry.id, { path, source, scroll: 0 }));
  }
  function select(next: Set<string>) {
    setSelectionTransfer(null);
    setSelection(next);
  }
  const writeBlocked = busy || !writeEnabled;
  const changes = entry.status?.changes;
  const rows = useMemo(() => changeRows(changes ?? []), [changes]);
  const selectableRows = rows.filter((row) => !row.change.conflict);
  const checkedRows = selectableRows.filter((row) => selection.has(row.key));
  const stageSelection = actionSelection(checkedRows, "stage");
  const unstageSelection = actionSelection(checkedRows, "unstage");
  const commitSelection = actionSelection(checkedRows, "commit");
  const firstFileSource = rows.some((row) => row.source === "index")
    ? "index"
    : "working_tree";
  const someFilesSelected = checkedRows.length > 0;
  const allFilesSelected =
    someFilesSelected && checkedRows.length === selectableRows.length;

  const observedAt = entry.status?.observed_at_ms;
  const changeContext = useMemo(
    () => JSON.stringify([observedAt, changes]),
    [observedAt, changes],
  );
  const [selectedChangeContext, setSelectedChangeContext] =
    useState(changeContext);
  const transfer =
    selectionTransfer?.before !== changeContext ? selectionTransfer : null;
  if (selectedChangeContext !== changeContext || transfer) {
    setSelectedChangeContext(changeContext);
    const next = new Set<string>();
    for (const row of selectableRows) {
      // A rename can split into two rows or form from two rows after staging.
      if (
        transfer &&
        row.paths.every((path) => transfer.paths.has(pathKey(path)))
      ) {
        if (row.source === transfer.target) next.add(row.key);
      } else if (selection.has(row.key)) {
        next.add(row.key);
      }
    }
    setSelection(next);
    if (transfer) setSelectionTransfer(null);
  }
  async function changeStaging(kind: "stage" | "unstage") {
    const { count, selected } =
      kind === "stage" ? stageSelection : unstageSelection;
    if (!count) return;
    const paths = selected.map((item) => item.path);
    const owner = Symbol(kind);
    setStaging({ kind, count, owner });
    try {
      await act(async () => {
        if (await propose({ kind, paths, hooks: policy }, count, owner)) {
          setSelectionTransfer({
            before: changeContext,
            target: kind === "stage" ? "index" : "working_tree",
            paths: new Set(paths.map(pathKey)),
          });
        }
      });
    } finally {
      setStaging(null);
      setCancellingStage(false);
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
          <button
            className="path repository-folder-link"
            title="Open folder in Finder"
            onClick={() =>
              void act(async () => {
                if (!bridge.repositoryPathAction)
                  throw new Error("Finder is unavailable");
                await bridge.repositoryPathAction(
                  { workspace, worktree: entry.id },
                  "open",
                );
              })
            }
          >
            {pathText(entry.display_path)}
          </button>
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
        pushPlanning={pushPlanning}
      />
      <div className="repository-tabs">
        <button
          aria-pressed={tab.view === "changes"}
          className={tab.view === "changes" ? "active" : ""}
          onClick={() => setNav(updateTab(nav, entry.id, { view: "changes" }))}
        >
          Changes <span>{changes?.length ?? 0}</span>
        </button>
        <button
          aria-pressed={tab.view === "history"}
          className={tab.view === "history" ? "active" : ""}
          onClick={() => setNav(updateTab(nav, entry.id, { view: "history" }))}
        >
          History
        </button>
      </div>
      {entry.issue && (
        <p role="alert" className="error">
          {entry.issue.message}
        </p>
      )}
      {tab.view === "changes" ? (
        <div
          className="changes-grid"
          ref={changesArea}
          style={{ height: changesHeight }}
        >
          <div className="files">
            <div className="section-title">
              <h2>Changed files</h2>
              <span>{commitSelection.count} selected</span>
            </div>
            <div className="file-actions">
              <button
                disabled={writeBlocked || !stageSelection.count}
                aria-busy={staging?.kind === "stage"}
                onClick={() => void changeStaging("stage")}
              >
                {staging?.kind === "stage" ? "Staging…" : "Stage"}
              </button>
              <button
                disabled={writeBlocked || !unstageSelection.count}
                aria-busy={staging?.kind === "unstage"}
                onClick={() => void changeStaging("unstage")}
              >
                {staging?.kind === "unstage" ? "Unstaging…" : "Unstage"}
              </button>
              {staging && (
                <div className="operation-progress">
                  <WorkingStatus
                    label={
                      cancellingStage
                        ? "Cancelling…"
                        : `${staging.kind === "stage" ? "Staging" : "Unstaging"} ${staging.count} ${staging.count === 1 ? "file" : "files"}…`
                    }
                  />
                  <button
                    disabled={cancellingStage}
                    onClick={() => {
                      setCancellingStage(true);
                      void act(async () => {
                        try {
                          await bridge.cancel(staging.owner);
                        } catch (error) {
                          setCancellingStage(false);
                          throw error;
                        }
                      });
                    }}
                  >
                    {cancellingStage ? "Cancelling…" : "Cancel"}
                  </button>
                </div>
              )}
            </div>
            <div
              className="file-list"
              tabIndex={0}
              aria-label="Changed file list"
            >
              {(["index", "working_tree"] as const).map((source) => (
                <div className="file-group" key={source}>
                  <h3>
                    {source === "index"
                      ? "Staged · HEAD ↔ index"
                      : "Unstaged / untracked · index ↔ worktree"}
                  </h3>
                  {source === firstFileSource && (
                    <div className="file-row file-selection-toggle">
                      <input
                        type="checkbox"
                        aria-label="Select all changed files"
                        title={
                          allFilesSelected
                            ? "Clear file selection"
                            : "Select all files"
                        }
                        disabled={busy || !selectableRows.length}
                        checked={allFilesSelected}
                        ref={(input) => {
                          if (input)
                            input.indeterminate =
                              someFilesSelected && !allFilesSelected;
                        }}
                        onChange={() =>
                          select(
                            new Set(
                              allFilesSelected
                                ? []
                                : selectableRows.map((row) => row.key),
                            ),
                          )
                        }
                      />
                    </div>
                  )}
                  {rows
                    .filter((row) => row.source === source)
                    .map(({ change: c, key, original }) => (
                      <div className="file-row" key={key}>
                        <input
                          type="checkbox"
                          aria-label={`Include ${pathText(c.path)} from ${source}`}
                          disabled={c.conflict || busy}
                          checked={!c.conflict && selection.has(key)}
                          onChange={() => {
                            const next = new Set(selection);
                            if (next.has(key)) next.delete(key);
                            else next.add(key);
                            select(next);
                          }}
                        />
                        <button
                          onClick={() => void act(() => diff(c.path, source))}
                        >
                          {original && `${pathText(original)} → `}
                          {pathText(c.path)}
                        </button>
                        <FileStatus change={c} source={source} />
                      </div>
                    ))}
                </div>
              ))}
              {!changes?.length && (
                <p className="muted">No observed changes.</p>
              )}
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
            <div className="composer-content">
              <CommitMessageEditor
                actions={
                  <AiCommitFill
                    key={JSON.stringify([
                      workspace,
                      entry.id,
                      commitSelection.selected,
                    ])}
                    bridge={bridge}
                    workspace={workspace}
                    worktree={entry.id}
                    selected={commitSelection.selected}
                    busy={busy}
                    onUse={setDraft}
                    beginUpdate={beginDraftUpdate}
                  />
                }
                label="Message"
                value={draft}
                rows={8}
                placeholder={"Describe the change\n\nOptional explanation…"}
                onChange={setDraft}
              />
              <label>
                Hooks
                <select
                  value={policy}
                  onChange={(e) => setPolicy(e.target.value as typeof policy)}
                >
                  <option value="execute">Execute supported hooks</option>
                  <option value="bypass_all">Skip hooks</option>
                </select>
              </label>
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
            </div>
            <div className="commit-actions">
              <button
                className="primary"
                disabled={
                  writeBlocked || !commitSelection.count || !draft.trim()
                }
                onClick={() =>
                  void act(async () => {
                    await propose(
                      {
                        kind: "commit",
                        selected: commitSelection.selected,
                        message: draft,
                        hooks: policy,
                        signing,
                      },
                      commitSelection.count,
                    );
                  })
                }
              >
                Commit · {commitSelection.count}{" "}
                {commitSelection.count === 1 ? "file" : "files"}
              </button>
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

function OverviewHeader({
  column,
  sort,
  onSort,
  children,
}: {
  column: SortColumn;
  sort: DesktopState["sort"];
  onSort: (sort: DesktopState["sort"]) => void;
  children: React.ReactNode;
}) {
  const choice = sortChoice(sort);
  const active = choice.column === column;
  return (
    <th
      scope="col"
      aria-sort={
        active ? (choice.descending ? "descending" : "ascending") : "none"
      }
    >
      <button
        className="overview-sort"
        onClick={() => onSort(toggleSort(sort, column))}
      >
        {children}
        {active ? (choice.descending ? " ↓" : " ↑") : ""}
      </button>
    </th>
  );
}
