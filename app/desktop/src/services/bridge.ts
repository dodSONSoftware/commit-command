import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { DesktopState, BulkComposerState } from "./response.generated";
import type { Request } from "./request.generated";
import type { Response } from "./response.generated";
export type Payload = Response["data"];
export interface CloseSnapshot {
  version: 1;
  epoch: number;
  workspace: string | null;
  navigation: DesktopState | null;
  bulk: BulkComposerState | null;
}
export interface Bridge {
  epoch: number;
  onAboutRequested?(show: () => void): Promise<() => void>;
  openProjectRepository?(): Promise<void>;
  manageAiToken?(connection: string): Promise<void>;

  onCloseRequested?(
    snapshot: () => CloseSnapshot,
    changed: (closing: boolean, error?: string) => void,
  ): Promise<() => void>;
  writeRevision?: number;
  capabilities(): Promise<{ writes_enabled: boolean }>;
  writeStatus(): Promise<boolean>;
  request(request: Request): Promise<Payload>;
  pick(): Promise<{ picker: string; display: string } | null>;
  pickAuthenticationFile?(
    kind: "helper" | "known_hosts" | "public_identity" | "ca_file",
  ): Promise<string | null>;
  cancel(): Promise<void>;
}
interface Job {
  state: string;
  response?: Response | null;
  error?: { kind: string; message: string } | null;
}
export class NativeBridge implements Bridge {
  epoch = 0;
  writeRevision = 0;
  async onCloseRequested(
    snapshot: () => CloseSnapshot,
    changed: (closing: boolean, error?: string) => void,
  ) {
    return getCurrentWebviewWindow().listen<string>(
      "gitfleet-close-request",
      async ({ payload: token }) => {
        changed(true);
        try {
          await invoke("close_snapshot", {
            token,
            payload: JSON.stringify(snapshot()),
          });
        } catch (error) {
          changed(false, String(error));
        }
      },
    );
  }
  onAboutRequested(show: () => void) {
    return getCurrentWebviewWindow().listen("commit-command-about", show);
  }
  openProjectRepository() {
    return invoke<void>("open_project_repository");
  }
  manageAiToken(connection: string) {
    return invoke<void>("manage_ai_token", { connection });
  }
  capabilities() {
    return invoke<{ writes_enabled: boolean }>("desktop_capabilities");
  }
  async writeStatus() {
    const status = await invoke<{
      state: string;
      response?: { revision: number } | null;
    }>("job_status", { job: "native-write" });
    if (status.response) {
      const revision = status.response.revision;
      if (!Number.isSafeInteger(revision) || revision < 0)
        throw new Error("Invalid native completion revision");
      this.writeRevision = revision;
    }
    return status.state === "running";
  }
  private pending: string[] = [];
  async request(request: Request): Promise<Payload> {
    const epoch = this.epoch;
    const job = await invoke<string>("dispatch", {
      payload: JSON.stringify({ version: 1, epoch, request }),
    });
    this.pending.push(job);
    try {
      for (;;) {
        const result = await invoke<Job>("job_status", { job });
        if (result.state === "complete") {
          if (result.error)
            throw new Error(`${result.error.kind}: ${result.error.message}`);
          const response = result.response;
          if (
            !response ||
            response.version !== 1 ||
            !Number.isSafeInteger(response.epoch) ||
            response.epoch < this.epoch
          )
            throw new Error("Stale or invalid desktop response");
          this.epoch = response.epoch;
          return response.data;
        }
        await new Promise((resolve) => setTimeout(resolve, 100));
      }
    } finally {
      this.pending = this.pending.filter((id) => id !== job);
    }
  }
  pick() {
    return invoke<{ picker: string; display: string } | null>("pick_directory");
  }
  pickAuthenticationFile(
    kind: "helper" | "known_hosts" | "public_identity" | "ca_file",
  ) {
    return invoke<string | null>("pick_authentication_file", { kind });
  }
  async cancel() {
    if (await this.writeStatus())
      await invoke("cancel_job", { job: "native-write" });
    else if (this.pending.length)
      await invoke("cancel_job", { job: this.pending[0] });
  }
}
type PayloadValues = {
  [K in Payload["kind"]]: Extract<Payload, { kind: K }>["value"];
};
export function value<T extends Payload["kind"]>(
  data: Payload,
  kind: T,
): PayloadValues[T] {
  if (data.kind !== kind)
    throw new Error(`Expected ${kind}, received ${data.kind}`);
  // The runtime discriminant above checks the generated Rust union before projection.
  return data.value as PayloadValues[T];
}
