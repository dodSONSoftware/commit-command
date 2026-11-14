import type { SelectedPath, WriteRequest } from "../services/request.generated";
import type { Change, NativePath } from "../services/response.generated";

export function pathKey(path: NativePath) {
  return JSON.stringify([path.encoding, path.value]);
}

export function changeRows(changes: Change[]) {
  return changes.flatMap((change) =>
    (["index", "working_tree"] as const).flatMap((source) => {
      const code =
        source === "index" || change.index === "?"
          ? change.index
          : change.worktree;
      if (code === "." || code === "!" || (source === "index" && code === "?"))
        return [];
      const original =
        code === "R" || code === "C" ? change.original_path : null;
      return [
        {
          key: JSON.stringify([
            source,
            code,
            pathKey(change.path),
            original && pathKey(original),
          ]),
          change,
          source,
          original,
          paths:
            code === "R" && original ? [original, change.path] : [change.path],
        },
      ];
    }),
  );
}

export function actionSelection(
  rows: ReturnType<typeof changeRows>,
  kind: WriteRequest["kind"],
) {
  const files = new Set<string>();
  const selected = new Map<string, SelectedPath>();
  const requiredSource =
    kind === "stage"
      ? "working_tree"
      : kind === "unstage"
        ? "index"
        : undefined;
  for (const { change, source, paths } of rows) {
    if (change.conflict || (requiredSource && source !== requiredSource))
      continue;
    files.add(pathKey(change.path));
    for (const path of paths) {
      const key = pathKey(path);
      // With both sides checked, use working bytes only for paths that side supplies.
      if (source === "working_tree" || !selected.has(key))
        selected.set(key, { path, source });
    }
  }
  return { count: files.size, selected: [...selected.values()] };
}
