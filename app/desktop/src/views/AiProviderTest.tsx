import { useEffect, useRef, useState } from "react";
import { value, type Bridge } from "../services/bridge";
import type { Request } from "../services/request.generated";
import type { AiInstalledModel } from "../services/response.generated";
export type ProviderConnection = Extract<
  Request,
  { kind: "review_ai_discovery" }
>["connection"];
// Saved connections include persistence metadata that the request DTO rejects.
export function connectionDraft(
  connection: ProviderConnection,
): ProviderConnection {
  return {
    id: connection.id,
    name: connection.name,
    provider: connection.provider,
    endpoint: connection.endpoint,
    timeout_ms: connection.timeout_ms,
    authentication: connection.authentication ?? "none",
  };
}
export function providerModelKey(connection: ProviderConnection): string {
  let endpoint = connection.endpoint;
  try {
    endpoint = new URL(endpoint.endsWith("/") ? endpoint : `${endpoint}/`).href;
  } catch {
    /* Core reports invalid endpoints. */
  }
  return JSON.stringify([
    connection.id ?? null,
    connection.provider,
    endpoint,
    connection.authentication ?? "none",
    connection.timeout_ms,
  ]);
}
function testError(error: unknown, endpoint: string): string {
  const message =
    error instanceof Error
      ? error.message
      : typeof error === "string"
        ? error
        : "Provider test failed";
  if (
    message.includes("Malformed or unsupported request") ||
    message.includes("Unknown request field")
  )
    return `The app rejected an invalid connection-test request before contacting ${endpoint}. This is an app error; it does not mean the AI server is unavailable.`;
  return message.includes("AI provider connection failed")
    ? `Could not connect to ${endpoint}. Check that the AI server is running and the server address is correct. ${message}`
    : message;
}
export function AiProviderTest({
  bridge,
  connection,
  active,
  onResult,
  onError,
  buttonLabel = "Test provider",
}: {
  bridge: Bridge;
  connection: ProviderConnection;
  active: boolean;
  onResult: (models: AiInstalledModel[] | null) => void;
  onError?: (message: string) => void;
  buttonLabel?: string;
}) {
  const [pending, setPending] = useState(false);
  const [ticket, setTicket] = useState<{ id: string; expires: number } | null>(
    null,
  );
  const [status, setStatus] = useState("");
  const [error, setError] = useState("");
  useEffect(() => {
    onError?.(error);
    return () => onError?.("");
  }, [error, onError]);

  const sequence = useRef(0);
  const resultCallback = useRef(onResult);
  useEffect(() => {
    resultCallback.current = onResult;
  }, [onResult]);
  useEffect(
    () => () => {
      sequence.current++;
    },
    [],
  );
  useEffect(() => {
    if (!active || !ticket) return;
    let live = true;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const poll = async () => {
      try {
        if (Date.now() >= ticket.expires)
          throw new Error("Provider test expired. Test the provider again.");
        const jobs = value(
          await bridge.request({ kind: "ai_jobs" }),
          "ai_jobs",
        );
        if (!live) return;
        const job = jobs.find(
          (job) => job.id === ticket.id && job.review.purpose === "discover",
        );
        if (job?.phase === "complete") {
          if (job.result?.kind !== "models")
            throw new Error("The provider did not return a model list.");
          const models = [
            ...new Map(
              job.result.value.map((model) => [model.id, model]),
            ).values(),
          ];
          resultCallback.current(models);
          setStatus(
            `Provider reachable · ${models.length} ${models.length === 1 ? "model" : "models"} available.`,
          );
          setTicket(null);
          return;
        }
        if (job?.phase === "failed" || job?.phase === "cancelled")
          throw new Error(job.error || "Provider test was cancelled.");
        if (job) setStatus("Testing provider and reading its model list…");
      } catch (error) {
        if (live) {
          resultCallback.current(null);
          setError(testError(error, connection.endpoint));
          setStatus("");
          setTicket(null);
        }
        return;
      }
      if (live) timer = setTimeout(() => void poll(), 500);
    };
    void poll();
    return () => {
      live = false;
      if (timer) clearTimeout(timer);
    };
  }, [active, ticket, bridge, connection.endpoint]);
  async function test() {
    const current = ++sequence.current;
    setPending(true);
    setTicket(null);
    setError("");
    setStatus("");
    onResult(null);
    try {
      const execution = value(
        await bridge.request({
          kind: "review_ai_discovery",
          connection: connectionDraft(connection),
        }),
        "executed",
      );
      if (current !== sequence.current) return;
      if (!execution.ai_job_id)
        throw new Error("Provider test did not start a job.");
      setStatus("Testing provider and reading its model list…");
      setTicket({
        id: execution.ai_job_id,
        expires: Date.now() + Math.max(60000, connection.timeout_ms + 30000),
      });
    } catch (error) {
      if (current === sequence.current)
        setError(testError(error, connection.endpoint));
    } finally {
      if (current === sequence.current) setPending(false);
    }
  }
  return (
    <div className="provider-test">
      <button
        type="button"
        disabled={
          pending ||
          !!ticket ||
          !connection.name.trim() ||
          !connection.endpoint.trim()
        }
        onClick={() => void test()}
      >
        {pending || ticket ? (
          <>
            <span className="working-spinner" aria-hidden="true" />
            Connecting…
          </>
        ) : (
          buttonLabel
        )}
      </button>
      <p>
        {buttonLabel === "Connect & Load Models"
          ? "Checks that the server is reachable and retrieves its available models. A successful connection unlocks the Model and Model settings tabs. This loads the model list; it does not load an AI model into memory or send repository content."
          : "Check this provider and discover its model tags. Uses the current fields and sends no repository content."}
      </p>
      {status && <p role="status">{status}</p>}
      {error && !onError && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
    </div>
  );
}
