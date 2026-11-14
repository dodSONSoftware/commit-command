import { useEffect, useState } from "react";

export function WorkingStatus({ label }: { label: string }) {
  const [started] = useState(Date.now);
  const [seconds, setSeconds] = useState(0);
  useEffect(() => {
    const timer = window.setInterval(() => {
      setSeconds(Math.floor((Date.now() - started) / 1000));
    }, 1000);
    return () => window.clearInterval(timer);
  }, [started]);
  return (
    <span role="status" className="working-status">
      <span className="working-spinner" aria-hidden="true" />
      <span>{label}</span>
      {seconds >= 5 && <span aria-live="off">{seconds}s</span>}
    </span>
  );
}
