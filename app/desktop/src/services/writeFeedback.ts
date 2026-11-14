import type { Payload } from "./bridge";
import type { WriteRequest } from "./request.generated";

/** An executed request can contain a failed outcome; transport success is not Git success. */
export function writeFeedback(
  response: Payload,
  request: WriteRequest,
  fileCount: number,
) {
  const action =
    request.kind === "stage"
      ? "stage files"
      : request.kind === "unstage"
        ? "unstage files"
        : "create the commit";
  const result = response.kind === "executed" ? response.value.result : null;
  if (
    !result ||
    typeof result !== "object" ||
    !("phase" in result) ||
    !("published" in result)
  )
    return {
      error: true,
      message: `Could not confirm whether Git finished. Refresh the repository before retrying.`,
    };
  const reason =
    "reason" in result && typeof result.reason === "string"
      ? ` ${result.reason}`
      : "";
  if (result.phase === "cancelled_before_publication")
    return {
      error: false,
      completed: false,
      message: `${request.kind === "stage" ? "Staging" : request.kind === "unstage" ? "Unstaging" : "Commit"} cancelled. ${request.kind === "commit" ? "No commit was created." : "The staging area was not changed."}`,
    };
  const indexEffect = `${fileCount} ${fileCount === 1 ? "file" : "files"} ${request.kind === "stage" ? "staged" : "unstaged"}.`;
  if (
    request.kind !== "commit" &&
    result.phase === "published_with_post_processing_failure" &&
    "index_reconciled" in result &&
    result.index_reconciled === true
  ) {
    return {
      error: true,
      message: `${indexEffect}${reason || " The post-index-change hook failed."}`,
    };
  }
  if (result.phase === "complete") {
    return {
      error: false,
      completed: true,
      message: request.kind === "commit" ? "Commit created." : indexEffect,
    };
  }
  if (result.published)
    return {
      error: true,
      message: `The commit was created, but follow-up work did not finish. Do not commit again. Open Operation results for recovery.${reason}`,
    };
  if (!["blocked", "needs_reauthorization"].includes(String(result.phase)))
    return {
      error: true,
      message: `Git's result could not be confirmed. Open Operation results before retrying.${reason}`,
    };
  return {
    error: true,
    message: `Could not ${action}. ${request.kind === "commit" ? "No commit was created." : "The staging area was not changed."}${reason}`,
  };
}
