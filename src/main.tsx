import React from "react";
import ReactDOM from "react-dom/client";
import { RouterProvider } from "@tanstack/react-router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./styles/globals.css";
import { router } from "./lib/app-router";

// probe: Contract Probe の build だけ（PR #250。PR にしない・merge しない）。
// 画面の上端の帯に、× を受けた時刻と起動時の sessionStorage の印を出す。帯は #root を下へ押し下げ、画面の操作を隠さない。
const PROBE_MARK_KEY = "probe-window-close-mark";
const probeBand = document.createElement("div");
probeBand.setAttribute("role", "log");
probeBand.style.cssText =
  "flex:none;max-height:5.5rem;overflow-y:auto;padding:2px 8px;background:#fef08a;color:#000;" +
  "border-bottom:2px solid #a16207;font-size:13px;line-height:1.35;font-family:monospace;";
document.body.style.display = "flex";
document.body.style.flexDirection = "column";
document.body.prepend(probeBand);
function probeLine(text: string): void {
  const line = document.createElement("div");
  line.textContent = text;
  probeBand.appendChild(line);
  probeBand.scrollTop = probeBand.scrollHeight;
}
function probeTime(): string {
  const d = new Date();
  const ms = String(d.getMilliseconds()).padStart(3, "0");
  return `${d.toLocaleTimeString("ja-JP", { hour12: false })}.${ms}`;
}
let probeMark: string | null = null;
try {
  probeMark = window.sessionStorage.getItem(PROBE_MARK_KEY);
} catch (e: unknown) {
  console.warn("probe: sessionStorage の読取りに失敗", e);
}
probeLine(`起動時の印: ${probeMark !== null ? "有" : "無"}`);
try {
  window.sessionStorage.setItem(PROBE_MARK_KEY, probeTime());
} catch (e: unknown) {
  console.warn("probe: sessionStorage の書込みに失敗", e);
}
let probeLastCloseAt = 0;
getCurrentWindow()
  .onCloseRequested((event) => {
    const now = Date.now();
    const withinThreeSeconds = now - probeLastCloseAt <= 3_000;
    probeLastCloseAt = now;
    probeLine(`probe: × を受けました（${probeTime()}）`);
    // 直前の × から 3 秒以内の × だけは止めない（onCloseRequested の包みが destroy で閉じる）
    if (!withinThreeSeconds) event.preventDefault();
  })
  .catch((e: unknown) => {
    probeLine(`probe: listener の登録に失敗しました（${String(e)}）`);
  });

/// TanStack Router + Query 初期化（ADR-001 / ADR-003 / 2026-04-20 採用）
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 60_000, // 1 min
      gcTime: 5 * 60_000, // 5 min
      retry: 1,
      refetchOnWindowFocus: false, // desktop app
    },
  },
});

const rootElement = document.getElementById("root");
if (!rootElement) {
  throw new Error("Root element #root not found in index.html");
}
// probe: 帯の下の残りの高さを #root に渡す（globals.css の height: 100% より flex-basis が優先される）
rootElement.style.flex = "1 1 0";
rootElement.style.minHeight = "0";

ReactDOM.createRoot(rootElement).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
      {import.meta.env.DEV && <ReactQueryDevtools buttonPosition="bottom-left" />}
    </QueryClientProvider>
  </React.StrictMode>,
);
