import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import TrayMenu from "./tray/TrayMenu";
import "./index.css";

// The same page serves the tray menu's window (marked by the app; ?tray in a browser).
const tray = "__EASYWSL_TRAY__" in window || new URLSearchParams(location.search).has("tray");
if (tray) document.documentElement.classList.add("tray");

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>{tray ? <TrayMenu /> : <App />}</React.StrictMode>,
);
