import { useRef, useState } from "react";

/** Block the owner webview while the native owned dialog handles settings. */
export function SettingsLauncher({
  disabled,
  open,
}: {
  disabled: boolean;
  open: () => Promise<void>;
}) {
  const guard = useRef<HTMLDialogElement>(null);
  const button = useRef<HTMLButtonElement>(null);
  const [error, setError] = useState("");
  async function launch() {
    setError("");
    guard.current?.showModal();
    try {
      await open();
    } catch (error) {
      setError(String(error));
    } finally {
      guard.current?.close();
      button.current?.focus();
    }
  }
  return (
    <>
      <button ref={button} disabled={disabled} onClick={() => void launch()}>
        Settings…
      </button>
      <dialog
        className="settings-owner-guard"
        aria-label="Settings is open"
        ref={guard}
        onCancel={(event) => {
          event.preventDefault();
          void open().catch((error) => setError(String(error)));
        }}
      >
        <span className="sr-only">
          Close Settings to return to the workspace.
        </span>
      </dialog>
      {error && <span role="alert">{error}</span>}
    </>
  );
}
