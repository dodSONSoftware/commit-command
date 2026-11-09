import { useEffect, useRef, useState } from "react";
import { value, type Bridge } from "../services/bridge";
import type { Request, AiCaptureSpec } from "../services/request.generated";
import type {
  AiCapturedContext,
  AiJob,
  AiProfile,
} from "../services/response.generated";
type Selection = Extract<
  Request,
  { kind: "preview_ai_context" }
>["spec"]["selected"];
export function AiMessages({
  bridge,
  workspace,
  worktree,
  selected,
  busy,
  onUse,
}: {
  bridge: Bridge;
  workspace: string;
  worktree: string;
  selected: Selection;
  busy: boolean;
  onUse: (message: string) => void;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const sequence = useRef(0);
  const [open, setOpen] = useState(false);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState("");
  const [nearby, setNearby] = useState("");
  const [docs, setDocs] = useState("");
  const [exclusions, setExclusions] = useState("");
  const [budget, setBudget] = useState(16384);
  const [branchHistory, setBranchHistory] = useState(20);
  const [fileHistory, setFileHistory] = useState(10);
  const [instructions, setInstructions] = useState("");
  const [profiles, setProfiles] = useState<AiProfile[]>([]);
  const [profile, setProfile] = useState("");
  const [captured, setCaptured] = useState<{
    spec: AiCaptureSpec;
    result: AiCapturedContext;
  } | null>(null);
  const [watch, setWatch] = useState(false);
  const [jobs, setJobs] = useState<AiJob[]>([]);
  function invalidate() {
    sequence.current++;
    setCaptured(null);
    setJobs([]);
    setWatch(false);
  }
  useEffect(() => {
    if (!open || !watch || !captured) return;
    let live = true;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const poll = async () => {
      try {
        const items = value(
          await bridge.request({ kind: "ai_jobs" }),
          "ai_jobs",
        );
        if (live)
          setJobs(
            items.filter(
              (j) =>
                j.review.source?.snapshot_fingerprint ===
                  captured.result.snapshot_fingerprint &&
                j.review.source?.context_fingerprint ===
                  captured.result.bundle.fingerprint,
            ),
          );
      } catch (e) {
        if (live) setError(String(e));
      } finally {
        if (live) timer = setTimeout(() => void poll(), 500);
      }
    };
    void poll();
    return () => {
      live = false;
      if (timer) clearTimeout(timer);
    };
  }, [bridge, open, watch, captured]);
  async function run(task: () => Promise<void>) {
    setPending(true);
    setError("");
    try {
      await task();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setPending(false);
    }
  }
  const paths = (text: string) =>
    text
      .split("\n")
      .filter(Boolean)
      .map((s) => ({ encoding: "utf8" as const, value: s }));
  async function capture() {
    invalidate();
    const current = sequence.current;
    const spec: AiCaptureSpec = {
      selected,
      nearby_source: paths(nearby),
      documentation: paths(docs),
      branch_commits: branchHistory,
      file_commits: fileHistory,
      policy: {
        max_bytes: budget,
        excluded_patterns: exclusions.split("\n").filter(Boolean),
      },
    };
    const result = value(
      await bridge.request({
        kind: "preview_ai_context",
        workspace,
        worktree,
        spec,
      }),
      "ai_context",
    );
    if (current === sequence.current) setCaptured({ spec, result });
  }
  return (
    <>
      <button
        ref={trigger}
        disabled={busy || !selected.length}
        onClick={() => {
          setOpen(true);
          dialog.current?.showModal();
        }}
      >
        AI message suggestions…
      </button>
      <dialog
        ref={dialog}
        className="ai-settings-dialog"
        aria-label="AI context and message suggestions"
        onClose={() => {
          setOpen(false);
          trigger.current?.focus();
        }}
      >
        <header>
          <h2>AI context and message suggestions</h2>
          <button onClick={() => dialog.current?.close()}>
            Close AI suggestions
          </button>
        </header>
        <p>
          AI is optional. Capture reads only the selected paths and explicit
          related context. It sends no request and requires no trust or AI
          setup. Every source transfer requires a separate exact native
          destination review. Suggestions only replace a draft when you choose
          one; staging, commits and Actions require their normal reviews.
        </p>
        <p>
          Suggestions describe the captured snapshot. Recapture after repository
          changes; a later commit review uses the current selected content.
        </p>
        <p>
          {selected.length} selected {selected.length === 1 ? "path" : "paths"}.
          Secret screening is defense in depth. Inspect the exact filtered
          payload and omissions before approving transfer.
        </p>
        <section aria-label="AI context capture options">
          <label>
            Related tracked source paths (one per line)
            <textarea
              value={nearby}
              maxLength={8192}
              disabled={pending}
              onChange={(e) => {
                invalidate();
                setNearby(e.target.value);
              }}
            />
          </label>
          <label>
            Related tracked documentation paths (one per line)
            <textarea
              value={docs}
              maxLength={8192}
              disabled={pending}
              onChange={(e) => {
                invalidate();
                setDocs(e.target.value);
              }}
            />
          </label>
          <label>
            Additional AI exclusion patterns (one per line)
            <textarea
              value={exclusions}
              maxLength={16384}
              disabled={pending}
              onChange={(e) => {
                invalidate();
                setExclusions(e.target.value);
              }}
            />
          </label>
          <label>
            AI context byte budget
            <input
              type="number"
              min={1024}
              max={1048576}
              value={budget}
              disabled={pending}
              onChange={(e) => {
                invalidate();
                setBudget(Number(e.target.value));
              }}
            />
          </label>
          <label>
            Current-branch history count
            <input
              type="number"
              min={0}
              max={20}
              value={branchHistory}
              disabled={pending}
              onChange={(e) => {
                invalidate();
                setBranchHistory(Number(e.target.value));
              }}
            />
          </label>
          <label>
            Affected-file history count
            <input
              type="number"
              min={0}
              max={10}
              value={fileHistory}
              disabled={pending}
              onChange={(e) => {
                invalidate();
                setFileHistory(Number(e.target.value));
              }}
            />
          </label>
          <button
            disabled={pending || !selected.length}
            onClick={() => void run(capture)}
          >
            Capture filtered AI context · no transfer
          </button>
        </section>
        {captured && (
          <section aria-label="Exact filtered AI context">
            <h3>Exact filtered context user data</h3>
            <p>Context fingerprint: {captured.result.bundle.fingerprint}</p>
            <p>Snapshot: {captured.result.snapshot_fingerprint}</p>
            <p>
              {captured.result.bundle.complete
                ? "All captured lines included without redaction."
                : "Incomplete or redacted coverage. Automatic commits are unavailable."}{" "}
              Provider framing, effective system message, template and
              instructions appear together in the exact native transfer review.
            </p>
            {captured.result.bundle.warnings.map((w) => (
              <p key={w}>{w}</p>
            ))}
            <pre>{captured.result.bundle.payload}</pre>
            <h3>Local coverage and omissions</h3>
            {captured.result.bundle.accounting.map((row) => (
              <article key={row.id}>
                <p>
                  {row.id} · {row.origin} · {row.kind}
                </p>
                <p>
                  {row.input_bytes} input bytes · {row.input_lines} input lines
                  · {row.included_lines} included · {row.redacted_lines}{" "}
                  screened lines · {row.omitted_lines} omitted
                </p>
                {row.reason && <p>{row.reason}</p>}
              </article>
            ))}
          </section>
        )}
        <section aria-label="Optional AI generation">
          <button
            disabled={pending}
            onClick={() =>
              void run(async () =>
                setProfiles(
                  value(
                    await bridge.request({ kind: "ai_settings" }),
                    "ai_settings",
                  ).settings.profiles,
                ),
              )
            }
          >
            Load optional AI profiles · no transfer
          </button>
          <label>
            Message generation profile
            <select
              value={profile}
              disabled={pending}
              onChange={(e) => {
                invalidate();
                setProfile(e.target.value);
              }}
            >
              <option value="">Use inherited assignment</option>
              {profiles.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name} · {p.model}
                </option>
              ))}
            </select>
          </label>
          <label>
            One-off message instructions
            <textarea
              maxLength={16384}
              value={instructions}
              disabled={pending}
              onChange={(e) => {
                invalidate();
                setInstructions(e.target.value);
              }}
            />
          </label>
          <p>
            Missing profiles/models/providers fail this AI request without
            fallback. Your existing draft remains unchanged. A smaller profile
            budget requires a fresh smaller capture.
          </p>
          <button
            disabled={pending || !captured}
            onClick={() =>
              void run(async () => {
                if (!captured) return;
                await bridge.request({
                  kind: "review_ai_messages",
                  workspace,
                  worktree,
                  spec: captured.spec,
                  snapshot_fingerprint: captured.result.snapshot_fingerprint,
                  context_fingerprint: captured.result.bundle.fingerprint,
                  one_off_profile: profile || null,
                  instructions,
                });
                setWatch(true);
              })
            }
          >
            Review exact AI source transfer
          </button>
          <button disabled={!captured} onClick={() => setWatch((w) => !w)}>
            {watch ? "Pause message result updates" : "Show message results"}
          </button>
          {error && <p role="alert">{error}</p>}
          {jobs.map((job) => (
            <article key={job.id} aria-label={`Message generation ${job.id}`}>
              <h3>Message generation · {job.phase}</h3>
              <p>
                {job.review.route} · {job.review.model}
              </p>
              {job.error && <p role="alert">{job.error}</p>}
              {(job.phase === "queued" || job.phase === "running") && (
                <button
                  onClick={() =>
                    void run(async () => {
                      await bridge.request({
                        kind: "cancel_ai_job",
                        id: job.id,
                      });
                    })
                  }
                >
                  Cancel message generation
                </button>
              )}
              {job.result?.kind === "candidates" && (
                <>
                  <p>
                    {job.result.value.elapsed_ms} ms · Input tokens:{" "}
                    {job.result.value.input_tokens ?? "not reported"} · Output
                    tokens: {job.result.value.output_tokens ?? "not reported"}
                  </p>
                  {job.result.value.warnings.map((w) => (
                    <p key={w}>{w}</p>
                  ))}
                  {job.result.value.candidates.map((item, index) => (
                    <section
                      key={index}
                      aria-label={`AI candidate ${index + 1}`}
                    >
                      <h4>
                        Candidate {index + 1} ·{" "}
                        {item.valid ? "template valid" : "template invalid"}
                      </h4>
                      <pre>
                        {item.candidate.message.subject}
                        {item.candidate.message.body
                          ? `\n\n${item.candidate.message.body}`
                          : ""}
                      </pre>
                      <p>Rationale: {item.candidate.rationale}</p>
                      <p>Summary: {item.candidate.summary}</p>
                      {item.candidate.concerns.map((c, i) => (
                        <p key={i}>Concern: {c}</p>
                      ))}
                      <p>
                        Model-reported chunks:{" "}
                        {item.candidate.covered_chunks.join(", ") || "None"}
                      </p>
                      <p>
                        Unreported chunks:{" "}
                        {item.unreported_chunks.join(", ") || "None"}
                      </p>
                      {item.validation_errors.map((e) => (
                        <p role="alert" key={e}>
                          {e}
                        </p>
                      ))}
                      <button
                        disabled={!item.valid || pending || busy}
                        onClick={() => {
                          onUse(
                            `${item.candidate.message.subject}${item.candidate.message.body ? `\n\n${item.candidate.message.body}` : ""}\n`,
                          );
                          dialog.current?.close();
                        }}
                      >
                        Replace draft with candidate {index + 1}
                      </button>
                    </section>
                  ))}
                </>
              )}
            </article>
          ))}
        </section>
      </dialog>
    </>
  );
}
