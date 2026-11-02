import { Outlet, useRouterState } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";

import { Toaster } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { navigation } from "@/config/navigation";
import { useAutoBackupCheck } from "@/features/backup-restore/useAutoBackupCheck";
import { PrintUnsupportedNotice } from "@/features/daily-sales/components/PrintUnsupportedNotice";
import { ShortcutsDialog, useShortcutsDialog } from "@/features/shortcuts";

import { Sidebar } from "./Sidebar";

/// UI-12 共通レイアウトのルート。
/// 設計: docs/function-design/52-ui-shared-layout.md §52.1 / §52.5
/// - 240px サイドバー + メイン Outlet の grid
/// - Sonner Toaster を bottom-right に常駐 (3 秒で自動消去)
/// - dev のみ TanStackRouterDevtools (button 位置 bottom-left、main.tsx の ReactQueryDevtools と同位置だが
///   ボタン重なりは許容、必要なら top-left に再調整)
/// - useRouterState({ select }) で pathname を取得 + deriveTitle() + useEffect で
///   document.title を `在庫管理システム - <画面名>` 形式に更新。
///   Phase 2 で route head() の動的 title を導入する際は useMatches() + 最深 match.head?.title に切替。

const APP_TITLE = "在庫管理システム";

/// 純関数: pathname から画面タイトルを引く。navigation 配列を走査し to === pathname の項目の title を返す。
/// 該当なし or ホーム ("/") は空文字を返す (呼び出し元で APP_TITLE 単独表記にフォールバック)。
/// Phase 2 以降で route head() の動的 title (loader data 由来) を導入する場合、本関数を deprecate し
/// matches[最深].head?.title を優先する形に差し替える。
export function deriveTitle(pathname: string): string {
  if (pathname === "/") return "";
  for (const area of navigation) {
    for (const item of area.items) {
      if (item.to === pathname) return item.title;
    }
  }
  return "";
}

export function RootLayout() {
  const pathname = useRouterState({ select: (s) => s.location.pathname });
  const title = deriveTitle(pathname);
  const formatted = title.length > 0 ? `${APP_TITLE} - ${title}` : APP_TITLE;
  // §54.1 接続点: useShortcutsDialog で open state + global Ctrl+/ keydown listener。
  // ShortcutsDialog は <Toaster /> と並列、TooltipProvider 直下に mount (Portal 注入で layout 非影響)。
  const { open: shortcutsOpen, setOpen: setShortcutsOpen } = useShortcutsDialog();
  // UI-11b-D13: 自動バックアップの確認を画面に依らず 60 秒ごとに動かす（hook 本体と state は backup-restore feature）。
  useAutoBackupCheck();

  useEffect(() => {
    // §52.5: WSL2 WebKitGTK では document.title が OS ウィンドウタイトルに rebind されない (2026-04-21 実機確認)。
    // Tauri ネイティブ API で明示的に setTitle して OS タスクバーに反映する。
    // 失敗してもアプリ動作は継続する (ブラウザ側 document.title は更新済み)。
    document.title = formatted;
    getCurrentWindow()
      .setTitle(formatted)
      .catch((e: unknown) => {
        console.warn("Tauri window.setTitle failed:", e);
      });
  }, [formatted]);

  return (
    <TooltipProvider>
      <div className="grid h-full grid-cols-[240px_minmax(0,1fr)] overflow-hidden bg-background text-foreground">
        <aside className="min-h-0 overflow-hidden border-r border-border bg-muted">
          <Sidebar />
        </aside>
        <main data-scroll-restoration-id="main" className="min-h-0 min-w-0 overflow-auto">
          <Outlet />
        </main>
      </div>
      <Toaster position="bottom-right" richColors closeButton duration={3000} />
      <ShortcutsDialog open={shortcutsOpen} onOpenChange={setShortcutsOpen} />
      {/* UI-09a-D18: どの画面の Ctrl+P でも、白紙の代わりに印刷の案内を 1 行出す（印刷のときだけ） */}
      <PrintUnsupportedNotice />
      {import.meta.env.DEV && <TanStackRouterDevtools position="bottom-left" />}
    </TooltipProvider>
  );
}
