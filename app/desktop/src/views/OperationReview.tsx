import { useLayoutEffect, useRef, useState } from "react";
import { value, type Bridge } from "../services/bridge";
import type { Request } from "../services/request.generated";
import type {
  ReviewDisplay,
  ReviewedOperation,
} from "../services/response.generated";

const labels: Record<ReviewedOperation, string> = {
  bulk_commit: "Commit reviewed changes",
  push: "Push reviewed commit",
  pre_push_check: "Run pre-push check",
  observe_branch: "Observe branch",
  download_branch: "Download branch",
  import_branch: "Import captured branch",
  set_upstream: "Set upstream",
};
const operations = {
  review_batch: "bulk_commit",
  review_push_publication: "push",
  review_push_batch_item: "push",
  review_push_check: "pre_push_check",
  review_remote_observation: "observe_branch",
  review_fetch_capture: "download_branch",
  review_fetch_import: "import_branch",
  review_upstream_setup: "set_upstream",
} as const satisfies Partial<Record<Request["kind"], ReviewedOperation>>;

export function useOperationReview(bridge: Bridge, scope: string) {
  const dialog = useRef<HTMLDialogElement>(null);
  const context = useRef<AbortController | null>(null);
  const decision = useRef<((execute: boolean) => void) | null>(null);
  const [display, setDisplay] = useState<ReviewDisplay | null>(null);
  const [pushResult, setPushResult] = useState<{
    scope: string;
    message: string;
  } | null>(null);
  useLayoutEffect(() => {
    const active = new AbortController();
    context.current = active;
    return () => {
      active.abort();
      decision.current?.(false);
    };
  }, [bridge, scope]);

  useLayoutEffect(() => {
    if (!display) return;
    const element = dialog.current!;
    element.showModal();
    const expiry = window.setTimeout(
      () => decision.current?.(false),
      Math.max(0, display.expires_at_ms - Date.now()),
    );
    return () => {
      window.clearTimeout(expiry);
      element.close();
    };
  }, [display]);

  async function review(
    request: Extract<Request, { kind: keyof typeof operations }>,
  ) {
    const active = context.current;
    if (!active || active.signal.aborted) return;
    setPushResult(null);
    const plan = value(await bridge.request(request), "review");
    const operation = operations[request.kind];
    if (plan.operation !== operation)
      throw new Error("Unexpected operation in prepared review");
    let execute = false;
    if (
      !active.signal.aborted &&
      !decision.current &&
      plan.expires_at_ms > Date.now()
    ) {
      execute = await new Promise<boolean>((resolve) => {
        decision.current = (answer) => {
          decision.current = null;
          setDisplay(null);
          resolve(answer);
        };
        setDisplay(plan);
      });
    }
    if (!execute || active.signal.aborted) {
      await bridge.request({
        kind: "cancel_reviewed_operation",
        id: plan.id,
        operation,
      });
      return;
    }
    const outcome = value(
      await bridge.request({
        kind: "execute_reviewed_operation",
        id: plan.id,
        operation,
      }),
      "executed",
    ).result;
    if (
      operation === "push" &&
      !active.signal.aborted &&
      outcome &&
      typeof outcome === "object" &&
      "phase" in outcome
    ) {
      const issue =
        "issue" in outcome &&
        outcome.issue &&
        typeof outcome.issue === "object" &&
        "message" in outcome.issue
          ? String(outcome.issue.message)
          : "";
      setPushResult({
        scope,
        message: `Push result: ${String(outcome.phase).replaceAll("_", " ")}${issue ? ". " + issue : ""}`,
      });
    }
  }

  const reviewDialog = (
    <>
      {pushResult?.scope === scope && <p role="status">{pushResult.message}</p>}
      {display && display.operation && (
        <dialog
          className="operation-review"
          ref={dialog}
          aria-label={display.title}
          onCancel={(event) => {
            event.preventDefault();
            decision.current?.(false);
          }}
        >
          <h2>{display.title}</h2>
          <div className="review-content" tabIndex={0}>
            {display.operation !== "bulk_commit" && (
              <pre>{display.summary}</pre>
            )}
            <pre>{display.content}</pre>
          </div>
          <footer>
            <button
              type="button"
              autoFocus
              onClick={() => decision.current?.(false)}
            >
              Cancel review
            </button>
            <button
              type="button"
              className="primary"
              onClick={() =>
                decision.current?.(Date.now() < display.expires_at_ms)
              }
            >
              {labels[display.operation]}
            </button>
          </footer>
        </dialog>
      )}
    </>
  );
  return { review, reviewDialog };
}
