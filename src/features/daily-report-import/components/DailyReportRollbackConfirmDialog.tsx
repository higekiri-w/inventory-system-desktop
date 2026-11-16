// src/features/daily-report-import/components/DailyReportRollbackConfirmDialog.tsx
//
// 日報の取込みの取消の確認（取込みの結果の画面と「日報の取込みの記録」の行が同じものを使う）。
// 確認の項目は UI-07-D14 の日報の項目（取込みID・対象日・ファイル名・総売上・純売上）。
// 設計: docs/function-design/55-ui-csv-import.md §55.12 UI-07-D19 / UI-07-D14

import { Loader2 } from "lucide-react";
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

/** 日報の金額の書き方（結果の画面・確認・記録の一覧で同じ。null は「未取得」） */
export function formatMoney(value: number | null) {
  return value === null ? "未取得" : `¥${value.toLocaleString()}`;
}

export interface DailyReportRollbackConfirmDialogProps {
  id: number;
  reportDate: string;
  filenames: string[];
  grossAmount: number | null;
  netAmount: number | null;
  onConfirm: () => void;
  /** この取込みの取消の応答を待つ間（入口の button に「取り消し中…」と spinner） */
  isRollingBack: boolean;
  /** ほかの取消の応答を待つ間など、入口を押せないとき */
  disabled?: boolean;
  size?: "default" | "sm";
  /**
   * 入口の button の accessible name で取り消す取込みを名指す（例「取込みID 11（2026-03-21）」→
   * 「取込みID 11（2026-03-21）を取り消す」）。同じ button が並ぶ記録の一覧の行だけが渡す
   */
  targetLabel?: string;
}

export function DailyReportRollbackConfirmDialog({
  id,
  reportDate,
  filenames,
  grossAmount,
  netAmount,
  onConfirm,
  isRollingBack,
  disabled = false,
  size = "default",
  targetLabel,
}: DailyReportRollbackConfirmDialogProps) {
  const visibleLabel = isRollingBack ? "取り消し中…" : "取り消す";
  return (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <Button
          variant="outline"
          size={size}
          disabled={isRollingBack || disabled}
          // 見える文を名前に含める（label in name）
          aria-label={targetLabel === undefined ? undefined : `${targetLabel}を${visibleLabel}`}
        >
          {isRollingBack && <Loader2 className="mr-2 size-4 animate-spin" aria-hidden="true" />}
          {visibleLabel}
        </Button>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>日報取込みを取り消しますか？</AlertDialogTitle>
          <AlertDialogDescription>
            この取込みだけを取り消します。同じ日の他の取込みは残ります。取消しても在庫数は変わりません。
          </AlertDialogDescription>
          {/* DSR-16: 単一レコード確認の definition list は原則適合。囲み階層が
              AlertDialogContent と二重にならないよう、独立した box border は持たせず
              border-t の行区切りのみにする（gated Amendment 3 最小調整）。 */}
          <dl className="grid grid-cols-2 gap-x-4 gap-y-2 border-t pt-3 text-sm">
            <div>
              <dt className="text-muted-foreground">取込み ID</dt>
              <dd className="font-medium">{id}</dd>
            </div>
            <div>
              <dt className="text-muted-foreground">対象日</dt>
              <dd className="font-medium">{reportDate}</dd>
            </div>
            <div className="col-span-2">
              <dt className="text-muted-foreground">ファイル名</dt>
              <dd className="font-medium">
                <ul className="space-y-0.5">
                  {filenames.map((name) => (
                    <li key={name} className="break-words">
                      {name}
                    </li>
                  ))}
                </ul>
              </dd>
            </div>
            <div>
              <dt className="text-muted-foreground">総売上</dt>
              <dd className="font-medium">{formatMoney(grossAmount)}</dd>
            </div>
            <div>
              <dt className="text-muted-foreground">純売上</dt>
              <dd className="font-medium">{formatMoney(netAmount)}</dd>
            </div>
          </dl>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>キャンセル</AlertDialogCancel>
          <AlertDialogAction variant="destructive" onClick={onConfirm}>
            取り消す
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
