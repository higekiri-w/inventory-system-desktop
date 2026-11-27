import React from "react";
import ReactDOM from "react-dom/client";
import { RouterProvider } from "@tanstack/react-router";
import { QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import "./styles/globals.css";
import { TextFieldContextMenu } from "./components/layout/TextFieldContextMenu";
import { router } from "./lib/app-router";
import { createAppQueryClient } from "./lib/query-client";
import { installWindowCloseGuard } from "./lib/window-close-listener";

// UI-USW-D5 の 2: native の window の close（×・Alt+F4）の listener を、render の前に 1 回だけ登録する
installWindowCloseGuard();

/// TanStack Router + Query 初期化（ADR-001 / ADR-003 / 2026-04-20 採用）。既定は src/lib/query-client.ts
const queryClient = createAppQueryClient();

const rootElement = document.getElementById("root");
if (!rootElement) {
  throw new Error("Root element #root not found in index.html");
}

ReactDOM.createRoot(rootElement).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      {/* UI-CTX-D1: root の errorComponent（RootLayout の外）も同じ Trigger の木に入れるため RouterProvider を包む */}
      <TextFieldContextMenu>
        <RouterProvider router={router} />
      </TextFieldContextMenu>
      {import.meta.env.DEV && <ReactQueryDevtools buttonPosition="bottom-left" />}
    </QueryClientProvider>
  </React.StrictMode>,
);
