import { useRef, useState } from "react";
import { value, type Bridge } from "../services/bridge";
import type {
  Increment,
  PrereleasePolicy,
  VersionPreview,
} from "../services/response.generated";
export function VersionAction({
  bridge,
  workspace,
  worktree,
  busy,
  act,
}: {
  bridge: Bridge;
  workspace: string;
  worktree: string;
  busy: boolean;
  act: (f: () => Promise<void>) => Promise<void>;
}) {
  const [path, setPath] = useState("version");
  const [increment, setIncrement] = useState<Increment>("patch");
  const [prerelease, setPrerelease] = useState<PrereleasePolicy>("reject");
  const [result, setResult] = useState<VersionPreview | null>(null);
  const sequence = useRef(0);
  function invalidate() {
    sequence.current++;
    setResult(null);
  }
  return (
    <details>
      <summary>Preview version update</summary>
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
            if (sequence.current === requestSequence) setResult(next);
          })
        }
      >
        Read version update preview
      </button>
      {result && (
        <section aria-label="Version update preview">
          <p>
            Old version: <code>{result.old}</code>
          </p>
          <p>
            New version: <code>{result.new}</code>
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
