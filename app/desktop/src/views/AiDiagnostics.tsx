import { useEffect, useRef, useState } from "react";
import { value, type Bridge } from "../services/bridge";
import type { Request } from "../services/request.generated";
import type {
  AiInstalledModel,
  AiJob,
  AiTemplate,
} from "../services/response.generated";
type Connection = Extract<
  Request,
  { kind: "review_ai_discovery" }
>["connection"];
type Profile = Extract<Request, { kind: "review_ai_inference" }>["profile"];
export function AiDiagnostics({
  bridge,
  connection,
  profile,
  template,
  active,
  onModels,
}: {
  bridge: Bridge;
  connection: Connection;
  profile: Profile;
  template: AiTemplate | null;
  active: boolean;
  onModels: (models: AiInstalledModel[]) => void;
}) {
  const [jobs, setJobs] = useState<AiJob[]>([]);
  const [error, setError] = useState("");
  const [watch, setWatch] = useState(false);
  const [pending, setPending] = useState(false);
  const seen = useRef(new Set<string>());
  const modelsCallback = useRef(onModels);
  useEffect(() => {
    modelsCallback.current = onModels;
  }, [onModels]);
  let endpointKey = "";
  try {
    endpointKey = new URL(
      connection.endpoint.endsWith("/")
        ? connection.endpoint
        : `${connection.endpoint}/`,
    ).href;
  } catch {
    /* Invalid unsaved endpoints are refused by Core before review. */
  }
  useEffect(() => {
    if (!active || !watch) return;
    let live = true;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const poll = async () => {
      try {
        const items = value(
          await bridge.request({ kind: "ai_jobs" }),
          "ai_jobs",
        );
        if (!live) return;
        setJobs(items);
        for (const job of items) {
          if (
            job.review.endpoint === endpointKey &&
            job.phase === "complete" &&
            job.result?.kind === "models" &&
            !seen.current.has(job.id)
          ) {
            seen.current.add(job.id);
            modelsCallback.current(job.result.value);
          }
        }
      } catch (e) {
        if (live)
          setError(
            e instanceof Error ? e.message : "AI job status unavailable",
          );
      } finally {
        if (live) timer = setTimeout(() => void poll(), 500);
      }
    };
    void poll();
    return () => {
      live = false;
      if (timer) clearTimeout(timer);
    };
  }, [active, watch, bridge, endpointKey]);
  async function review(kind: "discover" | "inference") {
    setPending(true);
    setError("");
    try {
      if (kind === "discover")
        await bridge.request({ kind: "review_ai_discovery", connection });
      else if (template)
        await bridge.request({
          kind: "review_ai_inference",
          connection,
          profile,
          template,
        });
      setWatch(true);
    } catch (e) {
      setError(e instanceof Error ? e.message : "AI diagnostic review failed");
    } finally {
      setPending(false);
    }
  }
  return (
    <section aria-label="AI provider diagnostics">
      <h3>Provider diagnostics</h3>
      <p>
        Uses the current connection and profile editors, including unsaved
        settings. Each request opens an exact native destination review.
        Inference uses a fixed synthetic diff and the selected unsaved template.
        These diagnostics capture no repository content and cannot stage, commit
        or run an Action.
      </p>
      <button
        disabled={pending || !connection.name || !connection.endpoint}
        onClick={() => void review("discover")}
      >
        Review connection test &amp; discover models
      </button>
      <button
        disabled={pending || !connection.name || !profile.model || !template}
        onClick={() => void review("inference")}
      >
        Review synthetic test inference
      </button>
      <button onClick={() => setWatch((w) => !w)}>
        {watch ? "Pause AI result updates" : "Show AI diagnostic results"}
      </button>
      {error && <p role="alert">{error}</p>}
      {jobs
        .filter((job) => job.review.purpose !== "assisted_messages")
        .map((job) => (
          <article key={job.id} aria-label={`AI diagnostic ${job.id}`}>
            <h4>
              {job.review.purpose === "discover"
                ? "Model discovery"
                : "Synthetic inference"}{" "}
              · {job.phase}
            </h4>
            <p>{job.review.route}</p>
            <p>
              Model: {job.review.model ?? "Discovery"} · Request:{" "}
              {job.review.fingerprint}
            </p>
            {(job.phase === "queued" || job.phase === "running") && (
              <button
                onClick={() =>
                  void bridge
                    .request({ kind: "cancel_ai_job", id: job.id })
                    .catch((e) =>
                      setError(
                        e instanceof Error
                          ? e.message
                          : "AI cancellation failed",
                      ),
                    )
                }
              >
                Cancel this AI request
              </button>
            )}
            {job.error && <p role="alert">{job.error}</p>}
            {job.result?.kind === "models" && (
              <p>
                {job.result.value.length} installed models:{" "}
                {job.result.value.map((m) => m.id).join(", ") ||
                  "none reported"}
              </p>
            )}
            {job.result?.kind === "completion" && (
              <>
                <p>
                  {job.result.value.elapsed_ms} ms · Input tokens:{" "}
                  {job.result.value.input_tokens ?? "not reported"} · Output
                  tokens: {job.result.value.output_tokens ?? "not reported"}
                </p>
                <pre>{job.result.value.content}</pre>
                <p>
                  Diagnostic response only. No commit message was selected or
                  applied.
                </p>
              </>
            )}
          </article>
        ))}
    </section>
  );
}
