// src/lib/query-client.ts
//
// app の QueryClient の既定（docs/UI_TECH_STACK.md §2 の TanStack Query の補強事項の 4、§6.11 UI-USW-D5 の 8）。
// query も mutation も local の DB を読み書きする Tauri の command で、回線に頼る画面は無い。
// 既定の networkMode（"online"）では、回線が切れた（window の offline の event）後に始めた読込み・保存・
// 取込み・取消が、回線が戻るまで command を送らずに止まる。"always" では回線が戻ったときの再取得
// （refetchOnReconnect）も既定で無くなるので、refetchOnReconnect は書かない。

import { QueryClient } from "@tanstack/react-query";

/** 呼ぶたびに新しい QueryClient を返す（main.tsx は起動で 1 回、test は test ごとに呼ぶ） */
export function createAppQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: 60_000, // 1 min
        gcTime: 5 * 60_000, // 5 min
        retry: 1,
        refetchOnWindowFocus: false, // desktop app
        networkMode: "always",
      },
      mutations: {
        networkMode: "always",
      },
    },
  });
}
