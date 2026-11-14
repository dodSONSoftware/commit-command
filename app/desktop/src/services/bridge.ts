import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import type {
  DesktopState,
  BulkComposerState,
  NativePath,
} from "./response.generated";
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
export interface AiSuggestionsContext {
  workspace: string;
  worktree: string;
  selected: Extract<
    Request,
    { kind: "preview_ai_context" }
  >["spec"]["selected"];
  theme: string;
}
export interface AiSuggestion {
  message: string;
  body_only: boolean;
}
export interface Bridge {
  registerPreferencesFlush?(flush: () => Promise<void>): () => void;
  epoch: number;
  openAiSuggestions?(
    context: AiSuggestionsContext,
  ): Promise<AiSuggestion | null>;

  openSettings?(context: {
    workspace: string | null;
    worktree?: string;
    theme: string;
  }): Promise<void>;
  onThemeChanged?(changed: (theme: string) => void): Promise<() => void>;
  onAboutRequested?(show: () => void): Promise<() => void>;
  openProjectRepository?(): Promise<void>;
  repositoryPathAction?(
    target: { workspace: string; worktree?: string },
    action: "copy" | "reveal" | "open",
  ): Promise<void>;
  manageAiToken?(connection: string): Promise<void>;
  pickAiImport?(): Promise<{ source: string; display: NativePath } | null>;

  onCloseRequested?(
    snapshot: () => CloseSnapshot,
    changed: (closing: boolean, error?: string) => void,
  ): Promise<() => void>;
  writeRevision?: number;
  capabilities(): Promise<{
    writes_enabled: boolean;
    ui_scale_percent?: number;
  }>;
  setUiScale?(percent: number): Promise<number>;
  writeStatus(): Promise<boolean>;
  request(request: Request, owner?: symbol): Promise<Payload>;
  pick(
    purpose?: "workspace" | "backup",
  ): Promise<{ picker: string; display: string } | null>;
  pickAuthenticationFile?(
    kind: "helper" | "known_hosts" | "public_identity" | "ca_file",
  ): Promise<string | null>;
  cancel(owner?: symbol): Promise<void>;
}
interface Job {
  state: string;
  response?: Response | null;
  error?: { kind: string; message: string } | null;
}
export class NativeBridge implements Bridge {
  private preferencesFlush?: () => Promise<void>;
  registerPreferencesFlush(flush: () => Promise<void>) {
    this.preferencesFlush = flush;
    return () => {
      this.preferencesFlush = undefined;
    };
  }
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
          await this.preferencesFlush?.();
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
  openAiSuggestions(context: AiSuggestionsContext) {
    return invoke<AiSuggestion | null>("open_ai_suggestions", { context });
  }
  openSettings(context: {
    workspace: string | null;
    worktree?: string;
    theme: string;
  }) {
    return invoke<void>("open_settings", {
      ...context,
      worktree: context.worktree ?? null,
    });
  }
  onThemeChanged(changed: (theme: string) => void) {
    return getCurrentWebviewWindow().listen<string>(
      "commit-command-theme",
      ({ payload }) => changed(payload),
    );
  }
  onAboutRequested(show: () => void) {
    return getCurrentWebviewWindow().listen("commit-command-about", show);
  }
  openProjectRepository() {
    return invoke<void>("open_project_repository");
  }
  repositoryPathAction(
    target: { workspace: string; worktree?: string },
    action: "copy" | "reveal" | "open",
  ) {
    return invoke<void>("repository_path_action", { target, action });
  }
  manageAiToken(connection: string) {
    return invoke<void>("manage_ai_token", { connection });
  }
  pickAiImport() {
    return invoke<{ source: string; display: NativePath } | null>(
      "pick_ai_import",
    );
  }
  capabilities() {
    return invoke<{ writes_enabled: boolean; ui_scale_percent?: number }>(
      "desktop_capabilities",
    );
  }
  setUiScale(percent: number) {
    return invoke<number>("set_ui_scale", { percent });
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
  private owned = new Map<symbol, Promise<string>>();
  private pending: string[] = [];
  async request(request: Request, owner?: symbol): Promise<Payload> {
    const epoch = this.epoch;
    const dispatched = invoke<string>("dispatch", {
      payload: JSON.stringify({ version: 1, epoch, request }),
    });
    if (owner) this.owned.set(owner, dispatched);
    let job: string | undefined;
    try {
      job = await dispatched;
      this.pending.push(job);
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
      if (owner) this.owned.delete(owner);
      this.pending = this.pending.filter((id) => id !== job);
    }
  }
  pick(purpose: "workspace" | "backup" = "workspace") {
    return invoke<{ picker: string; display: string } | null>(
      "pick_directory",
      { purpose },
    );
  }
  pickAuthenticationFile(
    kind: "helper" | "known_hosts" | "public_identity" | "ca_file",
  ) {
    return invoke<string | null>("pick_authentication_file", { kind });
  }
  async cancel(owner?: symbol) {
    if (owner) {
      const dispatch = this.owned.get(owner);
      if (dispatch) await invoke("cancel_job", { job: await dispatch });
      return;
    }
    if (await this.writeStatus())
      await invoke("cancel_job", { job: "native-write" });
    else if (this.pending.length)
      await invoke("cancel_job", { job: this.pending[0] });
  }
}
/** Each Settings webview has its own epoch; synchronize before a settings reload. */
export class SettingsBridge extends NativeBridge {
  async onSettingsCloseRequested(
    flush: () => Promise<boolean>,
    changed: (closing: boolean, error?: string) => void,
  ) {
    const window = getCurrentWebviewWindow();
    const close = await window.listen<string>(
      "gitfleet-settings-close-request",
      async ({ payload: token }) => {
        changed(true);
        let saved = false;
        try {
          saved = await flush();
          if (!saved)
            changed(true, "The latest AI settings could not be saved.");
        } catch (error) {
          changed(true, String(error));
        }
        try {
          await invoke("settings_flush_done", { token, saved });
        } catch (error) {
          changed(false, String(error));
        }
      },
    );
    try {
      const cancelled = await window.listen("gitfleet-close-cancelled", () =>
        changed(false),
      );
      return () => {
        close();
        cancelled();
      };
    } catch (error) {
      close();
      throw error;
    }
  }

  override async request(request: Request): Promise<Payload> {
    if (request.kind === "ai_settings") {
      await super.request({ kind: "app_preferences" });
    }
    // Never replay saves or native reviews against a newer application context.
    return super.request(request);
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
