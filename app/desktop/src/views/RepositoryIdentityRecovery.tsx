import type { StoredWorktree } from "../services/response.generated";
import { pathText } from "../stores/model";

export function RepositoryIdentityRecovery({
  entries,
  drafts,
  busy,
  onRescan,
  onOpen,
}: {
  entries: StoredWorktree[];
  drafts: { [id: string]: string };
  busy: boolean;
  onRescan: () => Promise<void>;
  onOpen: (entry: StoredWorktree) => void;
}) {
  const historical = entries.filter((e) => e.superseded_by);
  const unresolved = entries.filter(
    (e) => !e.superseded_by && e.availability !== "available",
  );
  if (!historical.length && !unresolved.length) return null;
  return (
    <section
      className="identity-recovery"
      aria-label="Repository identity recovery"
    >
      <h2>Repository identity recovery</h2>
      <p>
        A repository may have been replaced, moved, or become inaccessible.
        Rescan locally to review what is present. Git files are never changed by
        recovery.
      </p>
      <button disabled={busy} onClick={() => void onRescan()}>
        Rescan and review
      </button>
      {unresolved.length > 0 && (
        <div>
          <h3>Needs inspection · {unresolved.length}</h3>
          <p>
            These locations remain unresolved. A failed or incomplete scan does
            not establish a replacement. Check the path, mounted volume and
            access, then rescan.
          </p>
          <ul>
            {unresolved.map((e) => (
              <li key={e.id}>
                {pathText(e.metadata.root)} —{" "}
                {e.issue?.message ?? e.availability}
              </li>
            ))}
          </ul>
        </div>
      )}
      {historical.length > 0 && (
        <details>
          <summary>
            Historical repository identities · {historical.length}
          </summary>
          <p>
            Rescan inspected a different identity at the same path. Historical
            entries are excluded from the active list. Drafts and operation
            history stay attached to their original identities; trust and
            approvals are never transferred.
          </p>
          {historical.map((old) => {
            const current = entries.find((e) => e.id === old.superseded_by);
            return (
              <article key={old.id}>
                <h3>{pathText(old.metadata.root)}</h3>
                <p>Historical identity: {old.id}</p>
                <p>Replacement identity: {old.superseded_by}</p>
                {current?.availability === "available" &&
                !current.superseded_by ? (
                  <button disabled={busy} onClick={() => onOpen(current)}>
                    Open inspected replacement
                  </button>
                ) : (
                  <p>
                    The previously inspected replacement is currently
                    unavailable. Rescan after resolving its location.
                  </p>
                )}
                {drafts[old.id] && (
                  <label>
                    Preserved historical draft (copy manually to recover)
                    <textarea readOnly value={drafts[old.id]} />
                  </label>
                )}
              </article>
            );
          })}
        </details>
      )}
    </section>
  );
}
