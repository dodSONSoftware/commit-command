import { useEffect, useMemo, useState } from "react";
import { value, type Bridge } from "../services/bridge";
import type {
  ActivityReport,
  StoredWorktree,
} from "../services/response.generated";
import { bytes, repoName } from "../stores/model";
export function utcCalendar(now: Date): string[] {
  const end = Date.UTC(
    now.getUTCFullYear(),
    now.getUTCMonth(),
    now.getUTCDate(),
  );
  const year = now.getUTCFullYear() - 1;
  const lastDay = new Date(
    Date.UTC(year, now.getUTCMonth() + 1, 0),
  ).getUTCDate();
  const start = Date.UTC(
    year,
    now.getUTCMonth(),
    Math.min(now.getUTCDate(), lastDay),
  );
  const days: string[] = [];
  for (let t = start; t <= end; t += 86400000)
    days.push(new Date(t).toISOString().slice(0, 10));
  return days;
}
export function WorkspaceActivity({
  bridge,
  workspace,
  entries,
  act,
  openCommit,
}: {
  bridge: Bridge;
  workspace: string;
  entries: StoredWorktree[];
  act: (action: () => Promise<void>) => Promise<void>;
  openCommit: (worktree: string, oid: string) => void;
}) {
  const [report, setReport] = useState<ActivityReport | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState("");
  const [mode, setMode] = useState("timeline");
  const [worktree, setWorktree] = useState("");
  const [author, setAuthor] = useState("");
  const [reference, setReference] = useState("");
  const [filter, setFilter] = useState(() => ({
    worktree: "",
    author: "",
    reference: "",
    at: Date.now(),
  }));
  const now = useMemo(() => new Date(filter.at), [filter.at]);
  const days = utcCalendar(now);
  useEffect(() => {
    let live = true;
    void act(async () => {
      const until = Math.floor(now.getTime() / 1000);
      try {
        const data = value(
          await bridge.request({
            kind: "activity",
            workspace,
            worktree: filter.worktree || null,
            author: filter.author || null,
            reference: filter.reference || null,
            since: Math.floor(Date.parse(utcCalendar(now)[0]) / 1000),
            until,
          }),
          "activity",
        );
        if (live) {
          setReport(data);
          setLoadError("");
        }
      } catch (error) {
        if (live) setLoadError(String(error));
        throw error;
      } finally {
        if (live) setLoading(false);
      }
    });
    return () => {
      live = false;
    };
  }, [bridge, workspace, filter, now, act]);
  const recent =
    report?.items.filter(
      (i) =>
        i.commit.committer_time >=
        Date.parse(days[Math.max(0, days.length - 7)]) / 1000,
    ) ?? [];
  const count = new Map<string, number>();
  for (const item of report?.items ?? []) {
    const day = new Date(item.commit.committer_time * 1000)
      .toISOString()
      .slice(0, 10);
    count.set(day, (count.get(day) ?? 0) + 1);
  }
  const name = (id: string) => {
    const entry = entries.find((e) => e.id === id);
    return entry ? repoName(entry) : id;
  };
  const commitButton = (item: ActivityReport["items"][number]) => (
    <button
      key={`${item.repository}/${item.commit.oid.hex}`}
      onClick={() => openCommit(item.worktree, item.commit.oid.hex)}
    >
      <strong>{name(item.worktree)}</strong> · {bytes(item.commit.subject)} ·{" "}
      <time>{new Date(item.commit.committer_time * 1000).toISOString()}</time>
    </button>
  );
  return (
    <section className="workspace-activity">
      <h1>Workspace Activity</h1>
      <p>
        Git-derived commits across all local branches. Dates use committer time
        in UTC. Linked worktrees share one repository identity; lanes do not
        imply ancestry between repositories.
      </p>
      <form
        className="activity-filters"
        onSubmit={(e) => {
          e.preventDefault();
          setLoading(true);
          setFilter({
            worktree,
            author,
            reference,
            at: Date.now(),
          });
        }}
      >
        <label>
          Repository
          <select
            aria-label="Activity repository"
            value={worktree}
            onChange={(e) => setWorktree(e.target.value)}
          >
            <option value="">Entire workspace</option>
            {entries.map((e) => (
              <option key={e.id} value={e.id}>
                {repoName(e)}
              </option>
            ))}
          </select>
        </label>
        <label>
          Author
          <input
            aria-label="Activity author"
            value={author}
            onChange={(e) => setAuthor(e.target.value)}
          />
        </label>
        <label>
          Full branch or tag reference
          <input
            aria-label="Activity reference"
            placeholder="All local branches"
            value={reference}
            onChange={(e) => setReference(e.target.value)}
          />
        </label>
        <button disabled={loading}>Apply activity filters</button>
      </form>
      <nav aria-label="Activity view">
        {["timeline", "lanes", "heatmap"].map((view) => (
          <button
            key={view}
            aria-pressed={mode === view}
            onClick={() => setMode(view)}
          >
            {view === "heatmap"
              ? "Heatmap · 12 months"
              : `${view === "timeline" ? "Timeline" : "Lanes"} · 7 days`}
          </button>
        ))}
      </nav>
      {report && !report.complete && (
        <p role="status">
          Partial activity: query limits or unavailable repositories prevented
          complete coverage.
        </p>
      )}
      {report?.issues.map((issue, i) => (
        <p role="alert" key={i}>
          {issue.kind}: {issue.message}
        </p>
      ))}
      {loading && <p>Loading activity…</p>}
      {loadError && <p role="alert">Activity unavailable: {loadError}</p>}
      {mode === "timeline" && (
        <div className="activity-timeline">
          {recent.map(commitButton)}
          {report && recent.length === 0 && (
            <p>No commits in the last seven days.</p>
          )}
        </div>
      )}
      {mode === "lanes" &&
        [...new Set(recent.map((item) => item.repository))].map(
          (repository) => {
            const items = recent.filter((i) => i.repository === repository);
            return (
              <section className="activity-lane" key={repository}>
                <h2>{name(items[0].worktree)}</h2>
                <div className="activity-lane-days">
                  {days.slice(-7).map((day) => (
                    <section key={day}>
                      <h3>{day}</h3>
                      {items
                        .filter(
                          (item) =>
                            new Date(item.commit.committer_time * 1000)
                              .toISOString()
                              .slice(0, 10) === day,
                        )
                        .map(commitButton)}
                    </section>
                  ))}
                </div>
              </section>
            );
          },
        )}
      {mode === "heatmap" && (
        <>
          <p>
            {days[0]} through {days.at(-1)} · UTC · {report?.items.length ?? 0}{" "}
            observed commits
          </p>
          <div className="activity-heatmap" aria-label="Contribution heatmap">
            {days.map((day) => (
              <div
                key={day}
                className={`heatmap-day intensity-${Math.min(count.get(day) ?? 0, 4)}`}
                title={`${day}: ${count.get(day) ?? 0} commits`}
                aria-label={`${day}: ${count.get(day) ?? 0} commits`}
              />
            ))}
          </div>
        </>
      )}
    </section>
  );
}
