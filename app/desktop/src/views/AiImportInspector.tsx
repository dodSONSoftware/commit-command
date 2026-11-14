import { useEffect, useMemo, useRef, useState } from "react";
import { value, type Bridge } from "../services/bridge";
import type {
  AiImportSource,
  AiImportTargets,
} from "../services/response.generated";
import { pathText, safeText } from "../stores/model";

export function archiveTimestamp(milliseconds: number): string {
  if (!Number.isSafeInteger(milliseconds) || milliseconds < 0)
    return "Unsupported source timestamp";
  const date = new Date(milliseconds);
  return Number.isFinite(date.getTime())
    ? date.toISOString()
    : "Unsupported source timestamp";
}

export function AiImportInspector({
  bridge,
  disabled,
}: {
  bridge: Bridge;
  disabled: boolean;
}) {
  const [source, setSource] = useState<AiImportSource | null>(null);
  const [targets, setTargets] = useState<AiImportTargets | null>(null);
  const [filter, setFilter] = useState("");
  const [appliedFilter, setAppliedFilter] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState("");
  const sequence = useRef(0);
  useEffect(
    () => () => {
      sequence.current++;
    },
    [],
  );
  const metadata = useMemo(
    () => (source ? safeText(JSON.stringify(source.settings, null, 2)) : ""),
    [source],
  );
  async function choose() {
    if (!bridge.pickAiImport) return;
    const current = ++sequence.current;
    setPending(true);
    setError("");
    setSource(null);
    setTargets(null);
    try {
      const selected = await bridge.pickAiImport();
      if (!selected || current !== sequence.current) return;
      const imported = value(
        await bridge.request({
          kind: "inspect_ai_import",
          source: selected.source,
        }),
        "ai_import_source",
      );
      if (imported.source !== selected.source)
        throw new Error("Archive selection changed; choose the backup again.");
      if (current === sequence.current) {
        setSource(imported);
        setFilter("");
        setAppliedFilter("");
      }
    } catch (error) {
      if (current === sequence.current) setError(String(error));
    } finally {
      if (current === sequence.current) setPending(false);
    }
  }
  async function search(query: string, offset = 0) {
    if (!source) return;
    const current = ++sequence.current;
    setError("");
    setTargets(null);
    if (
      new TextEncoder().encode(query).byteLength > 512 ||
      query.includes("\0")
    ) {
      setError(
        "Use at most 512 UTF-8 bytes for target search, without NUL characters.",
      );
      setPending(false);
      return;
    }
    setPending(true);
    try {
      const page = value(
        await bridge.request({
          kind: "ai_import_targets",
          source: source.source,
          filter: query,
          offset,
        }),
        "ai_import_targets",
      );
      if (current === sequence.current) {
        setTargets(page);
        setAppliedFilter(query);
      }
    } catch (error) {
      if (current === sequence.current) setError(String(error));
    } finally {
      if (current === sequence.current) setPending(false);
    }
  }
  return (
    <section className="ai-import-inspector" aria-label="AI backup inspection">
      <h3>Inspect an AI configuration backup</h3>
      <p>
        Read-only inspection verifies an unencrypted ZIP archive. Imported
        authentication and destination approvals are cleared, revisions are
        regenerated, and assignments remain inactive until explicitly mapped and
        approved.
      </p>
      <button
        disabled={disabled || pending || !bridge.pickAiImport}
        onClick={() => void choose()}
      >
        Choose AI backup to inspect…
      </button>
      {!bridge.pickAiImport && (
        <p>Native archive selection is unavailable in this host.</p>
      )}
      {pending && <p role="status">Loading inspection data…</p>}
      {error && <p role="alert">{error}</p>}
      {source && (
        <>
          <h4>{pathText(source.path)}</h4>
          <dl>
            <dt>Source application version</dt>
            <dd>{safeText(source.manifest.application_version)}</dd>
            <dt>Archive format</dt>
            <dd>
              {source.manifest.version} · {source.manifest.kind}
            </dd>
            <dt>Created · UTC</dt>
            <dd>{archiveTimestamp(source.manifest.created_at_ms)}</dd>
            <dt>Source settings revision</dt>
            <dd>
              {safeText(source.manifest.settings_revision ?? "Not declared")}
            </dd>
            <dt>Archive SHA-256</dt>
            <dd>
              <code>{source.archive_fingerprint}</code>
            </dd>
            <dt>Manifest SHA-256</dt>
            <dd>
              <code>{source.manifest_fingerprint}</code>
            </dd>
          </dl>
          <table>
            <caption>Verified archive components</caption>
            <thead>
              <tr>
                <th scope="col">Path</th>
                <th scope="col">Bytes</th>
                <th scope="col">SHA-256</th>
              </tr>
            </thead>
            <tbody>
              {source.manifest.components.map((component) => (
                <tr key={component.path}>
                  <th scope="row">{safeText(component.path)}</th>
                  <td>{component.bytes}</td>
                  <td>
                    <code>{component.sha256}</code>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          <details>
            <summary>Sanitized inactive AI metadata</summary>
            <p>
              Control characters are escaped for display. The declared manifest
              identifies the original archive; these settings have fresh import
              revisions.
            </p>
            <pre>{metadata}</pre>
          </details>
          <form
            onSubmit={(event) => {
              event.preventDefault();
              void search(filter);
            }}
          >
            <label>
              Find current mapping targets
              <input
                aria-label="Find current AI import mapping targets"
                value={filter}
                maxLength={512}
                onChange={(event) => setFilter(event.target.value)}
              />
            </label>
            <button disabled={disabled || pending}>
              Search mapping targets
            </button>
          </form>
          {targets && (
            <>
              <p>
                Search: {safeText(appliedFilter) || "all current targets"} ·{" "}
                {targets.total} matching targets · {targets.items.length} on
                this page. Mapping requires a separate explicit choice.
              </p>
              <ul aria-label="Current AI import mapping targets">
                {targets.items.map((target) => (
                  <li key={JSON.stringify(target.scope)}>
                    {safeText(target.name)}
                    {target.path && ` · ${pathText(target.path)}`}
                    <code> · {safeText(JSON.stringify(target.scope))}</code>
                  </li>
                ))}
              </ul>
              {targets.next !== null && targets.next !== undefined && (
                <button
                  disabled={disabled || pending}
                  onClick={() => void search(appliedFilter, targets.next ?? 0)}
                >
                  Next mapping target page
                </button>
              )}
            </>
          )}
        </>
      )}
    </section>
  );
}
