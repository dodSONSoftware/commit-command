import {
  AppPreferencesProvider,
  useAppPreferences,
} from "./views/AppPreferences";
import { AiMessages } from "./views/AiMessages";
import type { AiSuggestionsContext } from "./services/bridge";
import { SettingsResizeGrip } from "./views/SettingsResizeGrip";
import { useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { SettingsBridge } from "./services/bridge";
import { AiSettings } from "./views/AiSettings";
import "./styles/app.css";

declare global {
  interface Window {
    __COMMIT_COMMAND_SETTINGS__: {
      ai?: AiSuggestionsContext;
      workspace: string | null;
      worktree?: string;
      theme: string;
      uiScale: number;
    };
  }
}
const context = window.__COMMIT_COMMAND_SETTINGS__;
const bridge = new SettingsBridge();
function SettingsWindow() {
  const { flush: flushPreferences } = useAppPreferences();
  const [ready, setReady] = useState(!context.ai);
  const [theme, setTheme] = useState(context.theme);
  const [scale, setScale] = useState(context.uiScale);
  const [error, setError] = useState("");
  const [closing, setClosing] = useState(false);
  const flushAi = useRef<() => Promise<boolean>>(async () => true);
  useEffect(() => {
    if (context.ai)
      void bridge
        .request({ kind: "app_preferences" })
        .then(() => setReady(true))
        .catch((e) => setError(String(e)));
  }, []);
  useEffect(() => {
    const window = getCurrentWebviewWindow();
    let waiting = false;
    const listener = window.onCloseRequested(async (event) => {
      if (waiting) {
        event.preventDefault();
        return;
      }
      waiting = true;
      try {
        await flushPreferences();
        if (!(await flushAi.current())) {
          event.preventDefault();
          waiting = false;
        }
        // Tauri destroys the window when this handler returns without preventing close.
      } catch (e) {
        event.preventDefault();
        waiting = false;
        setError(String(e));
      }
    });
    return () => {
      void listener.then((unlisten) => unlisten());
    };
  }, [flushPreferences]);

  useEffect(() => {
    const listener = bridge.onSettingsCloseRequested(
      async () => {
        await flushPreferences();
        return flushAi.current();
      },
      (closing, error) => {
        setClosing(closing);
        if (error) setError(error);
      },
    );
    return () => {
      void listener.then((stop) => stop());
    };
  }, [flushPreferences]);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);
  return (
    <main className="settings-window" inert={closing}>
      {error && (
        <p role="alert" className="error settings-error-banner">
          {error}
        </p>
      )}
      {context.ai ? (
        ready && (
          <AiMessages
            bridge={bridge}
            workspace={context.ai.workspace}
            worktree={context.ai.worktree}
            selected={context.ai.selected}
            busy={false}
            standalone
            onRequestClose={() =>
              void getCurrentWebviewWindow()
                .close()
                .catch((e) => setError(String(e)))
            }
            onUse={(message) =>
              void invoke("use_ai_suggestion", {
                suggestion: { message, body_only: false },
              }).catch((e) => setError(String(e)))
            }
            onUseBody={(message) =>
              void invoke("use_ai_suggestion", {
                suggestion: { message, body_only: true },
              }).catch((e) => setError(String(e)))
            }
          />
        )
      ) : (
        <AiSettings
          bridge={bridge}
          workspace={context.workspace}
          worktree={context.worktree}
          disabled={false}
          standalone
          onFlushReady={(flush) => {
            flushAi.current = flush;
          }}
          onRequestClose={() =>
            void getCurrentWebviewWindow()
              .close()
              .catch((e) => setError(String(e)))
          }
          theme={theme}
          onThemeChange={(next) => {
            void invoke("set_settings_theme", { theme: next })
              .then(() => setTheme(next))
              .catch((e) => setError(String(e)));
          }}
          uiScale={scale}
          onUiScaleChange={async (next) => {
            setScale(await bridge.setUiScale(next));
          }}
        />
      )}
      <SettingsResizeGrip
        onError={setError}
        label={context.ai ? "AI message" : "Settings"}
      />
    </main>
  );
}
createRoot(document.getElementById("root")!).render(
  <AppPreferencesProvider bridge={bridge}>
    <SettingsWindow />
  </AppPreferencesProvider>,
);
