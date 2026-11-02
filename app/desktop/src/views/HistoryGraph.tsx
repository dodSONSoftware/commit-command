import { useEffect, useRef, useState } from "react";
import { value, type Bridge, type Payload } from "../services/bridge";
import { bytes } from "../stores/model";
type GraphPage = Extract<Payload, { kind: "graph" }>["value"];
type GraphRow = GraphPage["rows"][number];

export function GraphLines({ row }: { row: GraphRow }) {
  const x = (lane: number) => 12 + lane * 18;
  const color = (lane: number) =>
    ["#4f9fff", "#61d7a0", "#e9ba61", "#ce9aff"][lane % 4];
  return (
    <svg
      className="commit-graph"
      width={Math.max(row.before.length, row.after.length, 1) * 18 + 12}
      height={34}
      aria-hidden="true"
    >
      {row.before.map((id, lane) => {
        const next = row.after.findIndex((p) => p.hex === id.hex);
        return lane === row.lane || next < 0 ? null : (
          <line
            key={id.hex}
            x1={x(lane)}
            y1={0}
            x2={x(next)}
            y2={34}
            stroke={color(lane)}
            strokeWidth={2}
          />
        );
      })}
      {!row.tip && (
        <line
          x1={x(row.lane)}
          y1={0}
          x2={x(row.lane)}
          y2={12}
          stroke={color(row.lane)}
          strokeWidth={2}
        />
      )}
      {row.commit.parents.map((parent) => (
        <line
          key={parent.hex}
          x1={x(row.lane)}
          y1={12}
          x2={x(row.after.findIndex((id) => id.hex === parent.hex))}
          y2={34}
          stroke={color(row.lane)}
          strokeWidth={2}
        />
      ))}
      <circle cx={x(row.lane)} cy={12} r={4} fill={color(row.lane)} />
    </svg>
  );
}
export function HistoryGraph({
  bridge,
  act,
  workspace,
  worktree,
  selected,
  reference,
  onReference,
  onSelect,
}: {
  bridge: Bridge;
  act: (action: () => Promise<void>) => Promise<void>;
  workspace: string;
  worktree: string;
  selected?: string | null;
  reference: string;
  onReference: (reference: string) => void;
  onSelect: (oid: string) => void;
}) {
  const [page, setPage] = useState<GraphPage | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(true);
  const [revision, setRevision] = useState(0);
  const [scroll, setScroll] = useState(0);
  const viewport = useRef<HTMLDivElement>(null);
  const selectedIndex =
    page?.rows.findIndex((r) => r.commit.oid.hex === selected) ?? -1;
  useEffect(() => {
    if (selectedIndex >= 0 && viewport.current)
      viewport.current.scrollTop = selectedIndex * 34;
  }, [selectedIndex]);
  const first = Math.max(0, Math.floor(scroll / 34) - 8);

  const scope = reference
    ? { kind: "reference" as const, reference }
    : { kind: "local_branches" as const };
  useEffect(() => {
    let live = true;
    void act(async () => {
      await bridge
        .request({
          kind: "graph",
          workspace,
          worktree,
          scope: reference
            ? { kind: "reference", reference }
            : { kind: "local_branches" },
          cursor: null,
        })
        .then((data) => {
          if (live) {
            setPage(value(data, "graph"));
            setError("");
          }
        })
        .catch((e) => {
          if (live) setError(String(e));
        })
        .finally(() => {
          if (live) setBusy(false);
        });
    });
    return () => {
      live = false;
    };
  }, [bridge, workspace, worktree, reference, revision, act]);
  async function more() {
    if (!page?.next) return;
    setBusy(true);
    try {
      const next = value(
        await bridge.request({
          kind: "graph",
          workspace,
          worktree,
          scope,
          cursor: page.next,
        }),
        "graph",
      );
      setPage({ ...next, rows: [...page.rows, ...next.rows] });
      setError("");
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }
  return (
    <div className="history-list">
      <div className="section-title">
        <h2>Local commit history</h2>
        <span className="muted">Actual parent relationships</span>
      </div>
      <label>
        History reference
        <select
          aria-label="History reference"
          value={reference}
          onChange={(e) => onReference(e.target.value)}
        >
          <option value="">All local branches</option>
          {page?.references
            .filter((r) => r.object_type === "commit" || r.peeled)
            .map((r) => (
              <option key={r.name} value={r.name}>
                {r.name}
              </option>
            ))}
        </select>
      </label>
      <button
        disabled={busy}
        onClick={() => {
          setBusy(true);
          setPage(null);
          setRevision(revision + 1);
        }}
      >
        Reload graph
      </button>
      {error && <p role="alert">{error}</p>}
      {page?.shallow_boundaries.length ? (
        <p className="muted">
          Shallow history boundaries are present; missing parents are not
          fabricated.
        </p>
      ) : null}
      <div
        className="graph-viewport"
        ref={viewport}
        aria-label="Commit graph"
        onScroll={(e) => setScroll(e.currentTarget.scrollTop)}
      >
        <div
          style={{
            height: (page?.rows.length ?? 0) * 34,
            position: "relative",
          }}
        >
          <div style={{ position: "absolute", top: first * 34, width: "100%" }}>
            {page?.rows.slice(first, first + 40).map((row) => (
              <button
                className={`commit-row ${selected === row.commit.oid.hex ? "selected" : ""}`}
                key={row.commit.oid.hex}
                onClick={() => onSelect(row.commit.oid.hex)}
              >
                <GraphLines row={row} />
                <code>{row.commit.oid.hex.slice(0, 8)}</code>
                <span>{bytes(row.commit.message).split("\n")[0]}</span>
                <small>
                  {new Date(
                    row.commit.committer_time * 1000,
                  ).toLocaleDateString()}
                </small>
              </button>
            ))}
          </div>
        </div>
      </div>
      {page?.next && (
        <button disabled={busy} onClick={() => void act(more)}>
          Load more
        </button>
      )}
      {page?.rows.length === 0 && <p>No local commits yet.</p>}
      {!page && !error && <p>Loading graph…</p>}
    </div>
  );
}
