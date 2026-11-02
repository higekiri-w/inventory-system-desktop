// src/features/csv-import/components/RollbackConfirmDialog.tsx
//
// Z004 の取込みの取消の確認（取込みの結果の画面と CSV取込みの詳細が同じものを使う）。
// 開くたびに取消の前の読取り（previewRollbackCsvImport）を行い、読めるまで「取り消す」を押せない。
// 設計: docs/function-design/55-ui-csv-import.md §55.11 UI-07-D18

import { useQuery } from "@tanstack/react-query";
import { Loader2 } from "lucide-react";
import { useState } from "react";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { RecheckTargetList } from "@/features/stocktake/components/RecheckTargetList";
import { commands } from "@/lib/bindings";
import { describeError } from "@/lib/describe-error";
import { CMD_ERROR_KIND, isInvokeError, unwrapResult } from "@/lib/invoke";
import { queryKeys } from "@/lib/query-keys";

export interface RollbackConfirmDialogProps {
  csvImportId: number;
  settlementDate: string;
  filename: string;
  totalItems: number;
  totalAmount: number;
  /** 入口の button の文言（結果の画面は「取り消す」、詳細は「この取込みを取り消す」） */
  triggerLabel: string;
  onConfirm: () => void;
  isRollingBack: boolean;
}

export function RollbackConfirmDialog({
  triggerLabel,
  isRollingBack,
  ...props
}: RollbackConfirmDialogProps) {
  const [open, setOpen] = useState(false);
  return (
    <AlertDialog open={open} onOpenChange={setOpen}>
      <AlertDialogTrigger asChild>
        <Button variant="outline" disabled={isRollingBack}>
          {isRollingBack && <Loader2 className="mr-2 size-4 animate-spin" aria-hidden="true" />}
          {isRollingBack ? "取り消し中…" : triggerLabel}
        </Button>
      </AlertDialogTrigger>
      {/* 中身は開いている間だけ mount し、読取りをそのたびに行う */}
      {open ? <RollbackConfirmContent {...props} /> : null}
    </AlertDialog>
  );
}

function RollbackConfirmContent({
  csvImportId,
  settlementDate,
  filename,
  totalItems,
  totalAmount,
  onConfirm,
}: Omit<RollbackConfirmDialogProps, "triggerLabel" | "isRollingBack">) {
  const previewQuery = useQuery({
    queryKey: queryKeys.inventoryRecords.csvImportRollbackPreview(csvImportId),
    queryFn: () =>
      unwrapResult(commands.previewRollbackCsvImport(csvImportId), {
        source: "commands",
        cmd: "preview_rollback_csv_import",
      }),
    staleTime: 0,
    gcTime: 0,
    retry: 0,
  });
  // 読み直しの間と読み直しの失敗では前の値を出さない（cache を使い回さない。失敗でも data は残る）
  const preview =
    previewQuery.isSuccess && !previewQuery.isFetching ? previewQuery.data : undefined;
  const error = previewQuery.isFetching ? null : previewQuery.error;

  return (
    <AlertDialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-2xl">
      <AlertDialogHeader>
        <AlertDialogTitle>取込みを取り消しますか？</AlertDialogTitle>
        <AlertDialogDescription>
          この取込みだけを取り消します。同じ日の他の取込みは残ります。この取込みの売上と在庫の変動を取り消し、在庫を記録から求め直します。
        </AlertDialogDescription>
        {/* DSR-16: 囲み階層が AlertDialogContent と二重にならないよう border-t の行区切りだけにする */}
        <dl className="grid grid-cols-2 gap-x-4 gap-y-2 border-t pt-3 text-sm">
          <div>
            <dt className="text-muted-foreground">取込み ID</dt>
            <dd className="font-medium">{csvImportId}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">精算日</dt>
            <dd className="font-medium">{settlementDate}</dd>
          </div>
          {preview ? (
            <div>
              <dt className="text-muted-foreground">精算回数</dt>
              <dd className="font-medium">精算 {preview.settlement_no}</dd>
            </div>
          ) : null}
          <div className="col-span-2">
            <dt className="text-muted-foreground">ファイル名</dt>
            <dd className="font-medium break-words">{filename}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">取込み件数</dt>
            <dd className="font-medium">{totalItems.toLocaleString()} 件</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">合計金額</dt>
            <dd className="font-medium">¥{totalAmount.toLocaleString()}</dd>
          </div>
        </dl>
      </AlertDialogHeader>

      <div className="space-y-3 text-sm">
        {preview ? (
          <>
            {preview.recheck_targets.length > 0 ? (
              <RecheckTargetList
                title="取り消した後、次の商品は数え直しが必要です（今すでに必要な商品を含みます）"
                targets={preview.recheck_targets}
              />
            ) : (
              <p>取り消しても、数え直しが必要になる商品はありません</p>
            )}
            <p>
              {`正しい版のファイルがあれば、取り消した後に「商品別CSV取込み（Z004）」で取り込み直してください（精算 ${String(preview.settlement_no)}）。取り込み直した結果に、数え直しが必要な商品が出ることがあります。`}
            </p>
            {preview.affected_stocktakes.map((stocktake) => (
              <p key={stocktake.stocktake_id}>
                {`完了した棚卸し（${stocktake.completed_at.slice(0, 10)}）の評価額は変わりません。棚卸しの記録の詳細の「差異」「ロス原価」と「完了の後に入った記録による差」が変わることがあります。`}
              </p>
            ))}
          </>
        ) : error ? (
          <p role="alert" className="text-destructive">
            {isInvokeError(error) && error.cmdError.kind === CMD_ERROR_KIND.VALIDATION
              ? describeError(error)
              : "取り消した後のようすを読み込めませんでした。閉じてから、もう一度開いてください"}
          </p>
        ) : (
          <p role="status" className="flex items-center gap-2">
            <Loader2 className="size-4 animate-spin" aria-hidden="true" />
            取り消した後のようすを確かめています…
          </p>
        )}
      </div>

      <AlertDialogFooter>
        <AlertDialogCancel>キャンセル</AlertDialogCancel>
        <AlertDialogAction variant="destructive" disabled={!preview} onClick={onConfirm}>
          取り消す
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  );
}
