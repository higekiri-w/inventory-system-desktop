import { useBlocker } from "@tanstack/react-router";

import { consumeMainNavScroll } from "@/lib/main-nav-scroll";
import { useWindowCloseGuard } from "@/lib/window-close-guard";

export interface UnsavedChangesWarning {
  isBlocked: boolean;
  continueEditing: () => void;
  discardAndProceed: () => void;
}

/// UI-USW-D1: feature-local な dirty 判定を TanStack Router の resolver 型 blocker と
/// beforeunload へ接続する。破棄確認の表示は共通 UnsavedChangesDialog が担う。
/// 同じ isDirty で native の window の close の止め（UI-USW-D5）も持つ（確認の窓は RootLayout の WindowCloseDialog）。
export function useUnsavedChangesWarning(isDirty: boolean): UnsavedChangesWarning {
  const blocker = useBlocker({
    shouldBlockFn: () => isDirty,
    withResolver: true,
    enableBeforeUnload: () => isDirty,
  });
  useWindowCloseGuard("dirty", isDirty);

  return {
    isBlocked: blocker.status === "blocked",
    continueEditing: () => {
      if (blocker.status === "blocked") {
        // onRendered は取り消した遷移では発火しないため、主ナビ marker をここで破棄する。
        consumeMainNavScroll();
        blocker.reset();
      }
    },
    discardAndProceed: () => {
      if (blocker.status === "blocked") blocker.proceed();
    },
  };
}
