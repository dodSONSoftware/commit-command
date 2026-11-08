import React from "react";
import { createRoot } from "react-dom/client";
import { flushSync } from "react-dom";
import { App } from "./views/App";
import { NativeBridge } from "./services/bridge";
import "./styles/app.css";
// The native host shows the window when document loading finishes. Concurrent
// root.render() can return before the shell commits, leaving that window empty.
flushSync(() => {
  createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
      <App bridge={new NativeBridge()} />
    </React.StrictMode>,
  );
});
