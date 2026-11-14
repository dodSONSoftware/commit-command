import { WorkingStatus } from "./WorkingStatus";
import { useEffect, useRef, useState } from "react";
import { value, type Bridge } from "../services/bridge";
import type { AiCaptureSpec } from "../services/request.generated";
import type { AiStreamProgress } from "../services/response.generated";

export function AiCommitFill({
  bridge,
  workspace,
  worktree,
  selected,
  busy,
  onUse,
  beginUpdate,
}: {
  bridge: Bridge;
  workspace: string;
  worktree: string;
  selected: AiCaptureSpec["selected"];
  busy: boolean;
  onUse: (message: string) => void;
  beginUpdate?: () => (message: string) => boolean;
}) {
  const [unavailable, setUnavailable] = useState<string | null>(
    "Checking AI configuration…",
  );
  const [generating, setGenerating] = useState(false);
  const [cancelling, setCancelling] = useState(false);
  const cancelRequested = useRef(false);
  const [progress, setProgress] = useState("");
  const [error, setError] = useState("");
  const [metrics, setMetrics] = useState<AiStreamProgress | null>(null);
  const [elapsed, setElapsed] = useState<number | null>(null);
  const started = useRef(0);
  useEffect(() => {
    if (!generating) return;
    const timer = window.setInterval(
      () => setElapsed(Date.now() - started.current),
      250,
    );
    return () => window.clearInterval(timer);
  }, [generating]);
  const sequence = useRef(0);
  const job = useRef<string | null>(null);
  useEffect(() => {
    const generation = sequence;
    const pendingJob = job;
    let live = true;
    const refresh = async () => {
      try {
        value(
          await bridge.request({
            kind: "effective_ai_settings",
            workspace,
            worktree,
          }),
          "effective_ai_settings",
        );
        if (live) setUnavailable(null);
      } catch {
        if (live)
          setUnavailable(
            "Connect your server and choose a model in Settings → AI.",
          );
      }
    };
    void refresh();
    window.addEventListener("focus", refresh);
    return () => {
      live = false;
      generation.current++;
      window.removeEventListener("focus", refresh);
      if (pendingJob.current)
        void bridge
          .request({ kind: "cancel_ai_job", id: pendingJob.current })
          .catch(() => {});
    };
  }, [bridge, workspace, worktree]);

  async function generate() {
    const current = ++sequence.current;
    const update =
      beginUpdate?.() ??
      ((message: string) => {
        onUse(message);
        return true;
      });
    const apply = (message: string) => {
      if (!update(message)) {
        cancelRequested.current = true;
        if (job.current)
          void bridge
            .request({ kind: "cancel_ai_job", id: job.current })
            .catch(() => {});
        return false;
      }
      return true;
    };
    cancelRequested.current = false;
    setCancelling(false);
    started.current = Date.now();
    setElapsed(0);
    setMetrics(null);
    setGenerating(true);
    setProgress("Checking AI settings…");
    setError("");
    try {
      const settings = value(
        await bridge.request({
          kind: "effective_ai_settings",
          workspace,
          worktree,
        }),
        "effective_ai_settings",
      );
      if (current !== sequence.current || cancelRequested.current) return;
      if (settings.connection.provider === "ollama") {
        setProgress("Loading model…");
        const loading = value(
          await bridge.request({
            kind: "review_ai_model_load",
            workspace,
            worktree,
          }),
          "executed",
        );
        if (!loading.ai_job_id || !loading.ai_request_fingerprint)
          throw new Error("Model loading did not start. Try again.");
        const id = loading.ai_job_id;
        job.current = id;
        if (current !== sequence.current || cancelRequested.current) {
          await bridge.request({ kind: "cancel_ai_job", id });
          return;
        }
        const loadDeadline = Date.now() + 130000;
        for (;;) {
          const jobs = value(
            await bridge.request({ kind: "ai_jobs" }),
            "ai_jobs",
          );
          if (current !== sequence.current) return;
          const result = jobs.find(
            (item) =>
              item.id === id &&
              item.review.fingerprint === loading.ai_request_fingerprint,
          );
          if (
            result &&
            ["complete", "failed", "cancelled"].includes(result.phase)
          ) {
            if (cancelRequested.current || result.phase === "cancelled") return;
            if (result.phase === "failed")
              throw new Error(
                result.error ??
                  "The AI server could not load the selected model.",
              );
            if (
              result.result?.kind !== "model_loaded" ||
              result.result.value.model !== settings.profile.model
            )
              throw new Error(
                "The AI server did not confirm that the selected model loaded.",
              );
            break;
          }
          if (Date.now() >= loadDeadline) {
            await bridge.request({ kind: "cancel_ai_job", id });
            throw new Error(
              "Model loading timed out. The AI server did not confirm that the model loaded.",
            );
          }
          await new Promise((resolve) => setTimeout(resolve, 400));
        }
        job.current = null;
        if (cancelRequested.current) return;
      }
      const spec: AiCaptureSpec = {
        selected,
        nearby_source: [],
        documentation: [],
        branch_commits: 0,
        file_commits: 0,
        policy: {
          max_bytes: settings.profile.context_bytes,
          excluded_patterns: [],
        },
      };
      setProgress("Reading selected changes…");
      const captured = value(
        await bridge.request({
          kind: "preview_ai_context",
          workspace,
          worktree,
          spec,
        }),
        "ai_context",
      );
      if (current !== sequence.current || cancelRequested.current) return;
      setProgress("Starting message generation…");
      const execution = value(
        await bridge.request({
          kind: "review_ai_messages",
          workspace,
          worktree,
          spec,
          snapshot_fingerprint: captured.snapshot_fingerprint,
          context_fingerprint: captured.bundle.fingerprint,
          one_off_profile: null,
          instructions:
            "Generate one concise commit message describing the selected changes.",
        }),
        "executed",
      );
      if (!execution.ai_job_id || !execution.ai_request_fingerprint)
        throw new Error("The AI request did not start. Try again.");
      const id = execution.ai_job_id;
      job.current = id;
      if (current !== sequence.current || cancelRequested.current) {
        await bridge.request({ kind: "cancel_ai_job", id });
        return;
      }
      setProgress("Waiting for the model…");
      const deadline = Date.now() + settings.connection.timeout_ms + 30000;
      let streamed = "";
      while (current === sequence.current) {
        const jobs = value(
          await bridge.request({ kind: "ai_jobs" }),
          "ai_jobs",
        );
        if (current !== sequence.current) return;
        const result = jobs.find(
          (item) =>
            item.id === id &&
            item.review.fingerprint === execution.ai_request_fingerprint,
        );
        if (!cancelRequested.current && result?.progress) {
          setMetrics(result.progress);
          if (result.progress.message && result.progress.message !== streamed) {
            streamed = result.progress.message;
            setProgress("Writing commit message…");
            if (!apply(streamed)) return;
          }
        }
        if (
          cancelRequested.current &&
          result &&
          ["failed", "cancelled", "complete"].includes(result.phase)
        )
          return;
        if (result?.phase === "failed")
          throw new Error(
            (result.error ?? "Message generation failed. Try again.") +
              (streamed ? " The partial message remains in the editor." : ""),
          );
        if (result?.phase === "cancelled") return;
        if (result?.phase === "complete") {
          if (result.result?.kind !== "candidates")
            throw new Error("The AI returned no commit message. Try again.");
          const candidate = result.result.value.candidates.find(
            (item) => item.valid,
          );
          if (!candidate)
            throw new Error(
              "The AI returned an invalid commit message. Try again.",
            );
          const { subject, body } = candidate.candidate.message;
          if (!result.progress)
            setMetrics({
              message: `${subject}${body ? `\n\n${body}` : ""}`,
              total_duration_ms: result.result.value.elapsed_ms,
              tokens_per_second: null,
              input_tokens: result.result.value.input_tokens ?? null,
              output_tokens: result.result.value.output_tokens ?? null,
            });
          apply(`${subject}${body ? `\n\n${body}` : ""}\n`);
          return;
        }
        if (Date.now() >= deadline) {
          await bridge.request({ kind: "cancel_ai_job", id });
          throw new Error("Message generation timed out. Try again.");
        }
        await new Promise((resolve) => setTimeout(resolve, 400));
      }
    } catch (e) {
      if (current === sequence.current && !cancelRequested.current)
        setError(e instanceof Error ? e.message : String(e));
    } finally {
      if (current === sequence.current) {
        job.current = null;
        setElapsed(Date.now() - started.current);
        setGenerating(false);
        setCancelling(false);
      }
    }
  }
  async function cancel() {
    cancelRequested.current = true;
    setCancelling(true);
    setProgress("Cancelling…");
    try {
      if (job.current)
        await bridge.request({ kind: "cancel_ai_job", id: job.current });
      else await bridge.cancel();
    } catch (error) {
      setError(
        `Could not cancel: ${error instanceof Error ? error.message : String(error)}`,
      );
      setCancelling(false);
    }
  }
  const reason =
    unavailable ??
    (busy
      ? "Wait for the current operation to finish."
      : !selected.length
        ? "Select changed files to generate a commit message."
        : null);
  return (
    <div className="ai-commit-fill">
      <button
        disabled={!!reason || generating}
        aria-busy={generating}
        title={reason ?? undefined}
        onClick={() => void generate()}
      >
        {generating ? "Generating message…" : "Fill with AI"}
      </button>
      <dl className="ai-generation-info" aria-label="AI generation information">
        <div>
          <dt>Total duration</dt>
          <dd>
            {metrics?.total_duration_ms != null
              ? `${(metrics.total_duration_ms / 1000).toFixed(1)} s`
              : elapsed != null
                ? `${(elapsed / 1000).toFixed(1)} s`
                : "—"}
          </dd>
        </div>
        <div>
          <dt>Generation speed</dt>
          <dd>
            {metrics?.tokens_per_second != null
              ? `${metrics.tokens_per_second.toFixed(1)} tokens/s`
              : "—"}
          </dd>
        </div>
        <div>
          <dt>Input tokens</dt>
          <dd>{metrics?.input_tokens?.toLocaleString() ?? "—"}</dd>
        </div>
        <div>
          <dt>Output tokens</dt>
          <dd>{metrics?.output_tokens?.toLocaleString() ?? "—"}</dd>
        </div>
      </dl>
      {generating && (
        <div className="operation-progress">
          <WorkingStatus label={progress} />
          <button disabled={cancelling} onClick={() => void cancel()}>
            {cancelling ? "Cancelling…" : "Cancel"}
          </button>
        </div>
      )}
      {!generating && reason && <small>{reason}</small>}
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
    </div>
  );
}
