import type { Outcome } from "./response.generated";

export function publicationStatus(
  outcome: Pick<Outcome, "published" | "phase"> | undefined,
): string {
  if (outcome?.published) return "Published";
  if (
    outcome &&
    [
      "publication_pending",
      "reference_updated",
      "index_renamed",
      "head_log_written",
      "published",
      "index_reconciled",
      "post_processing",
      "needs_manual_reconciliation",
      "uncertain_outcome",
    ].includes(outcome.phase)
  )
    return "Publication unconfirmed — inspect guarded recovery";
  return outcome ? "Not published" : "No publication result recorded";
}
