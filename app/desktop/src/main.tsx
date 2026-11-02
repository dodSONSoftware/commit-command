import React from "react";
import { createRoot } from "react-dom/client";
import { App } from "./views/App";
import { NativeBridge } from "./services/bridge";
import "./styles/app.css";
createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App bridge={new NativeBridge()} />
  </React.StrictMode>,
);
