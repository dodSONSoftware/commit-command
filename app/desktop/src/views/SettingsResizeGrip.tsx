import { useRef } from "react";
import { LogicalSize } from "@tauri-apps/api/dpi";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

export function SettingsResizeGrip({
  onError,
  label = "Settings",
}: {
  onError: (message: string) => void;
  label?: string;
}) {
  const drag = useRef<{
    x: number;
    y: number;
    toX: number;
    toY: number;
    width?: number;
    height?: number;
    released: boolean;
  } | null>(null);
  function resize(start: NonNullable<typeof drag.current>) {
    if (start.width === undefined || start.height === undefined) return;
    void getCurrentWebviewWindow()
      .setSize(
        new LogicalSize(
          Math.max(620, start.width + start.toX - start.x),
          Math.max(420, start.height + start.toY - start.y),
        ),
      )
      .catch((error) => onError(String(error)));
  }
  return (
    <button
      type="button"
      className="settings-resize-grip"
      aria-label={`Resize ${label} window`}
      title={`Drag to resize ${label}, or focus here and use the arrow keys`}
      onClick={(event) => event.currentTarget.focus()}
      onKeyDown={(event) => {
        const step = 20;
        const width =
          event.key === "ArrowRight"
            ? step
            : event.key === "ArrowLeft"
              ? -step
              : 0;
        const height =
          event.key === "ArrowDown"
            ? step
            : event.key === "ArrowUp"
              ? -step
              : 0;
        if (!width && !height) return;
        event.preventDefault();
        const window = getCurrentWebviewWindow();
        void Promise.all([window.innerSize(), window.scaleFactor()])
          .then(([size, factor]) =>
            window.setSize(
              new LogicalSize(
                Math.max(620, size.width / factor + width),
                Math.max(420, size.height / factor + height),
              ),
            ),
          )
          .catch((error) => onError(String(error)));
      }}
      onPointerDown={(event) => {
        if (event.button !== 0) return;
        event.preventDefault();
        event.currentTarget.focus();
        event.currentTarget.setPointerCapture(event.pointerId);
        const start = {
          x: event.screenX,
          y: event.screenY,
          toX: event.screenX,
          toY: event.screenY,
          released: false,
        };
        drag.current = start;
        const window = getCurrentWebviewWindow();
        void Promise.all([window.innerSize(), window.scaleFactor()])
          .then(([size, factor]) => {
            if (drag.current !== start) return;
            drag.current.width = size.width / factor;
            drag.current.height = size.height / factor;
            resize(drag.current);
            if (drag.current.released) drag.current = null;
          })
          .catch((error) => onError(String(error)));
      }}
      onPointerMove={(event) => {
        const start = drag.current;
        if (!start || start.released) return;
        start.toX = event.screenX;
        start.toY = event.screenY;
        resize(start);
      }}
      onPointerUp={(event) => {
        const start = drag.current;
        if (!start) return;
        start.toX = event.screenX;
        start.toY = event.screenY;
        start.released = true;
        resize(start);
        if (start.width !== undefined) drag.current = null;
      }}
      onPointerCancel={() => {
        drag.current = null;
      }}
      onLostPointerCapture={() => {
        if (drag.current) {
          drag.current.released = true;
          if (drag.current.width !== undefined) drag.current = null;
        }
      }}
    >
      <svg
        aria-hidden="true"
        width="16"
        height="16"
        viewBox="0 0 16 16"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
      >
        <path d="M3 13 13 3M8 13l5-5M12 13l1-1" />
      </svg>
    </button>
  );
}
