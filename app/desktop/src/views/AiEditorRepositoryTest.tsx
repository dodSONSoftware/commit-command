import { useState } from "react";
import { value, type Bridge } from "../services/bridge";
import type { Request } from "../services/request.generated";
import type { StoredWorktree } from "../services/response.generated";
import { pathText, repoName } from "../stores/model";
import { AiMessages, type AiEditorDraft } from "./AiMessages";

type Selection = Extract<
  Request,
  { kind: "preview_ai_context" }
>["spec"]["selected"];
export function AiEditorRepositoryTest({
  bridge,
  workspace,
  worktree,
  editor,
  disabled,
}: {
  bridge: Bridge;
  workspace: string;
  worktree?: string;
  editor: AiEditorDraft;
  disabled: boolean;
}) {
  const [entries, setEntries] = useState<StoredWorktree[]>([]);
  const [repository, setRepository] = useState(worktree ?? "");
  const [selected, setSelected] = useState<ReadonlySet<string>>(new Set());
  const [pending, setPending] = useState(false);
  const [error, setError] = useState("");
  const [query, setQuery] = useState("");
  const entry = entries.find((item) => item.id === repository);
  const choices: Selection = (entry?.status?.changes ?? []).flatMap(
    (change) => {
      if (change.conflict) return [];
      const choices: Selection = [];
      if (![".", "?", "!"].includes(change.index))
        choices.push({ path: change.path, source: "index" });
      if (change.worktree !== "." || change.index === "?")
        choices.push({ path: change.path, source: "working_tree" });
      return choices;
    },
  );
  const selectedChoices = choices.filter((choice) =>
    selected.has(JSON.stringify(choice)),
  );
  const selectedSources = new Map(
    selectedChoices.map((choice) => [
      JSON.stringify(choice.path),
      choice.source,
    ]),
  );
  const matching = choices.filter((choice) =>
    pathText(choice.path).toLowerCase().includes(query.toLowerCase()),
  );
  async function load() {
    setPending(true);
    setError("");
    try {
      const inventory = value(
        await bridge.request({ kind: "inventory", workspace }),
        "inventory",
      );
      setEntries(
        inventory.entries.filter(
          (item) => item.availability === "available" && !item.superseded_by,
        ),
      );
      setSelected(new Set());
    } catch (error) {
      setError(
        error instanceof Error
          ? error.message
          : "Repository inventory unavailable",
      );
    } finally {
      setPending(false);
    }
  }
  return (
    <section aria-label="Test unsaved AI editors with repository content">
      <h3>Test with repository context</h3>
      <p>
        Choose a repository and exact change sources. Capture is local and
        read-only; sending content requires a separate exact native review. This
        test uses the shown unsaved editors, does not save settings or replace a
        draft, and cannot execute an Action. The connection editor above
        supplies the test destination; saved profile assignments are not used.
      </p>
      <button disabled={pending || disabled} onClick={() => void load()}>
        Load repositories for AI test · no transfer
      </button>
      <label>
        Repository for AI test
        <select
          value={repository}
          disabled={pending || disabled}
          onChange={(event) => {
            setRepository(event.target.value);
            setSelected(new Set());
          }}
        >
          <option value="">Choose a repository</option>
          {entries.map((item) => (
            <option key={item.id} value={item.id}>
              {repoName(item)} · {pathText(item.display_path)}
            </option>
          ))}
        </select>
      </label>
      {choices.length > 0 && (
        <p>
          Choose one content source per path. Deselect it before choosing the
          other source for that path.
        </p>
      )}
      <label>
        Filter AI test paths
        <input
          value={query}
          onChange={(event) => setQuery(event.target.value)}
        />
      </label>
      <p>
        {selectedChoices.length} explicitly selected · maximum 128. Showing{" "}
        {Math.min(256, matching.length)} of {matching.length} matching sources;
        filter by path to see others. Filtering does not deselect paths.
      </p>
      {matching.slice(0, 256).map((choice) => {
        const key = JSON.stringify(choice);
        const chosenSource = selectedSources.get(JSON.stringify(choice.path));
        const alternateSelected =
          chosenSource !== undefined && chosenSource !== choice.source;
        return (
          <label key={key}>
            <input
              type="checkbox"
              disabled={
                pending ||
                disabled ||
                alternateSelected ||
                (selectedChoices.length >= 128 && !selected.has(key))
              }
              checked={selected.has(key)}
              aria-label={`Include AI test ${pathText(choice.path)} from ${choice.source}`}
              onChange={() =>
                setSelected((current) => {
                  const next = new Set(current);
                  if (next.has(key)) next.delete(key);
                  else next.add(key);
                  return next;
                })
              }
            />
            {pathText(choice.path)} ·{" "}
            {choice.source === "index"
              ? "Staged: HEAD to index"
              : "Working tree compared with HEAD"}
          </label>
        );
      })}
      {entry && choices.length === 0 && (
        <p>
          No supported non-conflicted changes in this observation. Refresh the
          inventory after changing files.
        </p>
      )}
      {error && <p role="alert">{error}</p>}
      <AiMessages
        key={JSON.stringify([
          workspace,
          repository,
          [...selected].sort(),
          editor,
        ])}
        bridge={bridge}
        workspace={workspace}
        worktree={repository}
        editor={editor}
        selected={selectedChoices}
        busy={pending || disabled || !entry}
      />
    </section>
  );
}
