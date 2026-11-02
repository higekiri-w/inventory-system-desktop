// src/features/csv-import/components/PreviewStep.tsx
//
// Step 2/3: プレビュー確認 + 取込み / 選び直し CTA + 知らせ（欠けた精算・日付のずれ）。
// 設計: docs/function-design/55-ui-csv-import.md §55.1 / §55.4 step 6-10 / §55.5

import { TriangleAlertIcon } from "lucide-react";
import { FilePicker, type PickedFile } from "@/components/FilePicker";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { formatSettlementGap } from "@/features/stocktake/lib/stocktake-formatters";
import type { PreviewData } from "@/lib/bindings";
import { ErrorRowsTable } from "./ErrorRowsTable";

export interface PreviewStepProps {
  preview: PreviewData;
  filename: string;
  onConfirm: () => void;
  onReselect: (file: PickedFile) => void;
  isImporting: boolean;
}

export function PreviewStep({
  preview,
  filename,
  onConfirm,
  onReselect,
  isImporting,
}: PreviewStepProps) {
  const { file_info, matched_summary, error_summary } = preview;
  const hasNotice = preview.settlement_gaps.length > 0 || preview.settlement_date_after_today;

  return (
    <div className="space-y-4">
      {/* 55 の D-118 の節: 知らせは取込みを止めない。Z004 は同じ日の追加の確認を出さない（ADR D4） */}
      {hasNotice ? (
        <Alert variant="warning" aria-label="取込みの前の知らせ">
          <TriangleAlertIcon aria-hidden="true" />
          <AlertTitle>確かめてから取り込んでください</AlertTitle>
          <AlertDescription className="text-warning-strong">
            <ul className="ml-4 list-disc">
              {preview.settlement_gaps.map((gap) => (
                <li key={`${String(gap.from)}-${String(gap.to)}`}>
                  {formatSettlementGap(gap)} が取り込まれていません
                </li>
              ))}
              {preview.settlement_date_after_today ? (
                <li>レジと PC の日付がずれている可能性があります</li>
              ) : null}
            </ul>
          </AlertDescription>
        </Alert>
      ) : null}

      <Card>
        <CardHeader className="flex flex-row items-center justify-between gap-4">
          <CardTitle>ファイル情報</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2 text-sm">
          <div>
            <span className="text-muted-foreground">精算日: </span>
            <span className="font-medium">{file_info.settlement_date}</span>
          </div>
          <div>
            <span className="text-muted-foreground">元ファイル名: </span>
            <span>{filename}</span>
          </div>
          <div>
            <span className="text-muted-foreground">ファイル hash: </span>
            <code className="text-xs">{file_info.file_hash.slice(0, 8)}…</code>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>紐付け結果</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3 text-sm">
          <div>
            紐付け成功:{" "}
            <span className="font-medium">{matched_summary.count.toLocaleString()}</span> 件
          </div>
          <div>
            合計金額:{" "}
            <span className="font-medium">¥{matched_summary.total_amount.toLocaleString()}</span>
          </div>
          {matched_summary.warnings.length > 0 && (
            <Alert>
              <AlertTitle>警告 {matched_summary.warnings.length} 件</AlertTitle>
              <AlertDescription>
                <ul className="ml-4 list-disc">
                  {matched_summary.warnings.map((w, i) => (
                    <li key={i}>{w}</li>
                  ))}
                </ul>
              </AlertDescription>
            </Alert>
          )}
        </CardContent>
      </Card>

      {error_summary.count > 0 && <ErrorRowsTable errorSummary={error_summary} />}

      <div className="flex flex-wrap gap-2">
        <Button
          onClick={() => {
            onConfirm();
          }}
          disabled={isImporting}
        >
          取り込む
        </Button>
        <FilePicker
          accept=".csv,.txt"
          ariaLabel="ファイルを選び直す（商品別CSV）"
          buttonLabel="ファイルを選び直す"
          dialogFilterName="CSV / TXT"
          dropEnabled={false}
          disabled={isImporting}
          onSelect={onReselect}
        />
      </div>
    </div>
  );
}
