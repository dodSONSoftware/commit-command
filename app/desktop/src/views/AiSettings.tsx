import { useRef, useState, type KeyboardEvent } from "react";
import type { Bridge } from "../services/bridge";
import { SingleAiSetup } from "./SingleAiSetup";

export function AiSettings({
  bridge,
  disabled,
  uiScale = 100,
  onUiScaleChange,
  theme = "system",
  onThemeChange,
  standalone = false,
  onRequestClose,
  onFlushReady,
}: {
  bridge: Bridge;
  workspace: string | null;
  worktree?: string;
  disabled: boolean;
  uiScale?: number;
  onUiScaleChange?: (percent: number) => Promise<void>;
  theme?: string;
  onThemeChange?: (theme: string) => void;
  standalone?: boolean;
  onRequestClose?: () => void;
  onFlushReady?: (flush: () => Promise<boolean>) => void;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const flushAi = useRef<() => Promise<boolean>>(async () => true);
  async function closeSettings() {
    if (!(await flushAi.current())) {
      setTab("ai");
      return;
    }
    if (onRequestClose) onRequestClose();
    else dialog.current?.close();
  }

  const trigger = useRef<HTMLButtonElement>(null);
  const content = useRef<HTMLDivElement>(null);
  const [scaleDraft, setScaleDraft] = useState<number | null>(null);
  const scaleRequest = useRef<number | null>(null);
  const scaleRunning = useRef(false);
  const displayedScale = scaleDraft ?? uiScale;
  async function changeScale(percent: number) {
    if (!onUiScaleChange) return;
    setScaleDraft(percent);
    scaleRequest.current = percent;
    if (scaleRunning.current) return;
    scaleRunning.current = true;
    setError("");
    try {
      // Keep native zoom/save calls ordered; rapid dragging keeps only the latest target.
      while (scaleRequest.current !== null) {
        const next = scaleRequest.current;
        scaleRequest.current = null;
        try {
          await onUiScaleChange(next);
        } catch (error) {
          setError(
            error instanceof Error ? error.message : "Could not save UI scale",
          );
          if (scaleRequest.current === null) setScaleDraft(null);
        }
      }
    } finally {
      scaleRunning.current = false;
    }
  }
  const [tab, setTab] = useState<"appearance" | "ai">("appearance");
  const [open, setOpen] = useState(standalone);
  const [error, setError] = useState("");
  const [aiError, setAiError] = useState("");
  const tabs = useRef<Array<HTMLButtonElement | null>>([]);
  const pending = false;
  function navigate(event: KeyboardEvent<HTMLButtonElement>, index: number) {
    if (
      ![
        "ArrowUp",
        "ArrowDown",
        "ArrowLeft",
        "ArrowRight",
        "Home",
        "End",
      ].includes(event.key)
    )
      return;
    event.preventDefault();
    const next = event.key === "Home" ? 0 : event.key === "End" ? 1 : 1 - index;
    setTab(next ? "ai" : "appearance");
    tabs.current[next]?.focus();
  }
  return (
    <>
      {!standalone && (
        <button
          ref={trigger}
          disabled={disabled}
          onClick={() => {
            setOpen(true);
            dialog.current?.showModal();
          }}
        >
          Settings…
        </button>
      )}
      <dialog
        ref={dialog}
        open={standalone || undefined}
        className="ai-settings-dialog"
        aria-labelledby="ai-settings-title"
        onCancel={(event) => {
          event.preventDefault();
          void closeSettings();
        }}
        onClose={() => {
          setOpen(false);
          trigger.current?.focus();
        }}
      >
        <header>
          <div>
            <span className="settings-eyebrow">Settings</span>
            <h2 id="ai-settings-title">
              {tab === "appearance" ? "Appearance" : "AI"}
            </h2>
          </div>
          <button onClick={() => void closeSettings()}>Close settings</button>
        </header>
        {(error || aiError) && (
          <div role="alert" className="error settings-error-banner">
            <span>{[error, aiError].filter(Boolean).join("\n")}</span>
          </div>
        )}
        <div className="settings-layout">
          <div
            className="settings-sidebar"
            role="tablist"
            aria-label="Settings sections"
            aria-orientation="vertical"
          >
            {(
              [
                ["appearance", "Appearance"],
                ["ai", "AI"],
              ] as const
            ).map(([id, label], i) => (
              <button
                key={id}
                role="tab"
                id={`settings-tab-${id}`}
                aria-controls={`settings-panel-${id}`}
                aria-selected={tab === id}
                tabIndex={tab === id ? 0 : -1}
                ref={(el) => {
                  tabs.current[i] = el;
                }}
                onKeyDown={(e) => navigate(e, i)}
                onClick={() => setTab(id)}
              >
                {label}
              </button>
            ))}
          </div>
          <div className="settings-content" ref={content}>
            <div
              role="tabpanel"
              id="settings-panel-appearance"
              aria-labelledby="settings-tab-appearance"
              hidden={tab !== "appearance"}
            >
              <section aria-label="Appearance settings">
                <h3>Appearance</h3>
                <label>
                  Color theme
                  <select
                    aria-label="Settings color theme"
                    value={theme}
                    onChange={(event) => onThemeChange?.(event.target.value)}
                    disabled={!onThemeChange}
                  >
                    <option value="system">System</option>
                    <option value="dark">Dark</option>
                    <option value="light">Light</option>
                  </select>
                </label>
                <label>
                  UI scale
                  <span className="ui-scale-control">
                    <input
                      type="range"
                      aria-label="UI scale"
                      min={75}
                      max={200}
                      step={5}
                      value={displayedScale}
                      aria-valuetext={`${displayedScale}%`}
                      disabled={pending || !onUiScaleChange}
                      onChange={(event) =>
                        void changeScale(Number(event.target.value))
                      }
                    />
                    <output aria-live="off">{displayedScale}%</output>
                  </span>
                </label>
                <p>
                  Scale application content, including settings. Saved for this
                  app on this computer.
                </p>
                <button
                  type="button"
                  disabled={
                    pending || !onUiScaleChange || displayedScale === 100
                  }
                  onClick={() => void changeScale(100)}
                >
                  Reset UI scale to 100%
                </button>
              </section>
            </div>
            <div
              role="tabpanel"
              id="settings-panel-ai"
              aria-labelledby="settings-tab-ai"
              hidden={tab !== "ai"}
            >
              {open && (
                <SingleAiSetup
                  bridge={bridge}
                  onError={setAiError}
                  onFlushReady={(flush) => {
                    const complete = async () => {
                      const saved = await flush();
                      if (!saved) setTab("ai");
                      return saved;
                    };
                    flushAi.current = complete;
                    onFlushReady?.(complete);
                  }}
                />
              )}
            </div>
          </div>
        </div>
      </dialog>
    </>
  );
}
