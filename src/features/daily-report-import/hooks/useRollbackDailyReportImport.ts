// src/features/daily-report-import/hooks/useRollbackDailyReportImport.ts
//
// 日報の取込みの取消の mutation（取込みの結果の画面と「日報の取込みの記録」が同じものを使う）。
// 設計: docs/function-design/55-ui-csv-import.md §55.12 UI-07-D19 / §55.9 / §55.7

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useBlocker } from "@tanstack/react-router";
import { toast } from "sonner";
import { commands, type DailyReportRollbackResult } from "@/lib/bindings";
import { invalidateByContract, invalidationContract } from "@/lib/invalidation-contract";
import { unwrapResult } from "@/lib/invoke";

export function useRollbackDailyReportImport({
  onSuccess,
}: {
  onSuccess?: (result: DailyReportRollbackResult) => void;
} = {}) {
  const queryClient = useQueryClient();
  const mutation = useMutation({
    mutationFn: (dailyReportImportId: number) =>
      unwrapResult(commands.rollbackDailyReportImport(dailyReportImportId), {
        source: "commands",
        cmd: "rollback_daily_report_import",
      }),
    retry: 0,
    onSuccess: async (result) => {
      onSuccess?.(result);
      // 取消は破壊的操作の完了通知で、既定 3 秒（RootLayout.tsx の Toaster duration）では非IT高齢operatorが
      // 読み切れないおそれがあるため、この通知だけ長く表示する（Toaster 全体設定は不変）。
      toast.success("日報取込みを取り消しました", { duration: 8000 });
      // D-052 C10: 一覧・日次売上・月次売上を再取得する
      await invalidateByContract(queryClient, invalidationContract.dailyReportImport());
    },
    onError: () => {
      // §55.9: トーストだけで画面の状態は据え置き（同じ取込みID のまま再試行できる）
      toast.error("取り消しに失敗しました。もう一度お試しください");
    },
  });
  // §55.7: 取消の応答を待つ間は route の移動とリロード等の page の離脱を止める（2 つの入口に共通。native の window の close は保証しない）。
  // pending は onSuccess / onError の後に外れるので、Toast と再取得の後に解ける
  useBlocker({
    shouldBlockFn: () => mutation.isPending,
    enableBeforeUnload: () => mutation.isPending,
  });
  return mutation;
}
