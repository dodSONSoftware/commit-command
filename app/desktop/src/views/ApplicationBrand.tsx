import { useCallback, useEffect, useRef, useState } from "react";
import type { Bridge } from "../services/bridge";
import { version } from "../../package.json";
import darkBanner from "../assets/brand/banner-dark.svg";
import lightBanner from "../assets/brand/banner-light.svg";
import license from "../assets/brand/LICENSE.txt?raw";

function Banner() {
  return (
    <>
      <img className="brand-banner brand-banner-dark" src={darkBanner} alt="" />
      <img
        className="brand-banner brand-banner-light"
        src={lightBanner}
        alt=""
      />
    </>
  );
}

export function ApplicationBrand({ bridge }: { bridge: Bridge }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const initialFocus = useRef<HTMLHeadingElement>(null);
  const previousFocus = useRef<HTMLElement | null>(null);
  const [showLicense, setShowLicense] = useState(false);
  const [error, setError] = useState("");
  const open = useCallback((invoker?: HTMLElement) => {
    if (dialog.current?.open) return;
    previousFocus.current =
      invoker ??
      (document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null);
    setShowLicense(false);
    setError("");
    dialog.current?.showModal();
    initialFocus.current?.focus();
  }, []);
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void bridge
      .onAboutRequested?.(() => open())
      .then((stop) => {
        if (disposed) stop();
        else unlisten = stop;
      });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [bridge, open]);
  return (
    <>
      <button
        type="button"
        className="brand-button"
        aria-label="About Commit Command"
        title="About Commit Command"
        onClick={(event) => open(event.currentTarget)}
      >
        <Banner />
      </button>
      <dialog
        className="about-dialog"
        ref={dialog}
        aria-labelledby="about-title"
        onClose={() => previousFocus.current?.focus()}
      >
        <div
          className="about-banner"
          aria-label="Commit Command — Git workspace manager for macOS"
        >
          <Banner />
        </div>
        <hr />
        <h2 id="about-title" ref={initialFocus} tabIndex={-1}>
          Commit Command
        </h2>
        <p className="about-version">version {version}</p>
        <p className="about-description">
          Local-first Git workspace manager for macOS.
        </p>
        <dl className="about-details">
          <dt>Developer</dt>
          <dd>Randel Dodson</dd>
          <dt>Copyright</dt>
          <dd>© 2026 Randel Dodson</dd>
          <dt>License</dt>
          <dd>MIT License</dd>
        </dl>
        {error && <p role="alert">{error}</p>}
        <div className="about-actions">
          <button
            type="button"
            onClick={() => setShowLicense((value) => !value)}
            aria-expanded={showLicense}
            aria-controls="about-license-text"
          >
            {showLicense ? "Close license" : "View License"}
          </button>
          <button
            type="button"
            disabled={!bridge.openProjectRepository}
            onClick={() => {
              setError("");
              void bridge
                .openProjectRepository?.()
                .catch(() =>
                  setError(
                    "The repository could not be opened. Please try again.",
                  ),
                );
            }}
          >
            GitHub repository…
          </button>
          <button
            type="button"
            className="primary about-ok"
            onClick={() => dialog.current?.close()}
          >
            OK
          </button>
        </div>
        {showLicense && (
          <textarea
            id="about-license-text"
            aria-label="MIT License"
            className="about-license"
            readOnly
            value={license}
            rows={12}
          />
        )}
      </dialog>
    </>
  );
}
