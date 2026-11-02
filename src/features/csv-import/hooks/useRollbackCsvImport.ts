// src/features/csv-import/hooks/useRollbackCsvImport.ts
//
// Z004 の取込みの取消の mutation（取込みの結果の画面と CSV取込みの詳細が同じものを使う）。
// 設計: docs/function-design/55-ui-csv-import.md §55.11 UI-07-D18 / §55.9 / §55.7

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useBlocker } from "@tanstack/react-router";
import { toast } from "sonner";
import { commands, type RollbackResult } from "@/lib/bindings";
import { invalidateByContract, invalidationContract } from "@/lib/invalidation-contract";
import { unwrapResult } from "@/lib/invoke";

export function useRollbackCsvImport({
  onSuccess,
}: {
  onSuccess: (result: RollbackResult) => void;
}) {
  const queryClient = useQueryClient();
  const mutation = useMutation({
    mutationFn: (csvImportId: number) =>
      unwrapResult(commands.rollbackCsvImport(csvImportId), {
        source: "commands",
        cmd: "rollback_csv_import",
      }),
    retry: 0,
    onSuccess: async (result) => {
      onSuccess(result);
      // 取消は破壊的操作の完了通知で、既定 3 秒（RootLayout.tsx の Toaster duration）では非IT高齢operatorが
      // 読み切れないおそれがあるため、この通知だけ長く表示する（Toaster 全体設定は不変）。
      toast.success("取込みを取り消しました", { duration: 8000 });
      await invalidateByContract(queryClient, invalidationContract.csvImportRollback());
    },
    onError: () => {
      // §55.9 取消の失敗: トーストだけで画面の状態は据え置き（同じ取込みID のまま再試行できる）
      toast.error("取り消しに失敗しました。もう一度お試しください");
    },
  });
  // §55.7: 取消の応答を待つ間は route の移動とリロード等の page の離脱を止める（2 つの入口に共通。native の window の close は §55.7 の「止める範囲」のとおり保証しない）。
  // pending は onSuccess / onError の後に外れるので、Toast と再取得の後に解ける
  useBlocker({
    shouldBlockFn: () => mutation.isPending,
    enableBeforeUnload: () => mutation.isPending,
  });
  return mutation;
}
