// src/features/inventory-records/CsvImportRecordDetailPage.tsx
//
// REQ-206 / REQ-207: CSV取込み記録詳細。有効な取込みは header から取り消せる（65 §65.5a TRACE-D13）。

import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useState } from "react";
import {
  AlertTriangle,
  ArrowLeft,
  CheckCircle2,
  CircleAlert,
  FileWarning,
  PackageSearch,
} from "lucide-react";

import { EmptyState } from "@/components/patterns/EmptyState";
import { PageHeader } from "@/components/patterns/PageHeader";
import { PageShell } from "@/components/patterns/PageShell";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { RollbackConfirmDialog } from "@/features/csv-import/components/RollbackConfirmDialog";
import { RollbackResultCard } from "@/features/csv-import/components/ResultStep";
import { useRollbackCsvImport } from "@/features/csv-import/hooks/useRollbackCsvImport";
// 数量の桁区切りと単位名は共通formatterに揃える。
import { formatStockDisplay } from "@/features/stock-inquiry/lib/format-stock-display";
import { MovementTable, stockUnitMap } from "@/features/stock-movements/components/MovementTable";
import type { CsvImportStatus, RollbackResult } from "@/lib/bindings";
import { commands } from "@/lib/bindings";
import { describeError } from "@/lib/describe-error";
import { unwrapResult } from "@/lib/invoke";
import { queryKeys } from "@/lib/query-keys";
import { returnToLinkProps } from "@/lib/return-to";
import { formatDateTime } from "@/lib/date-time";
import { formatYen } from "./types";

export interface CsvImportRecordDetailPageProps {
  importId: number;
  returnTo?: string;
}

const STATUS_LABELS: Record<CsvImportStatus, string> = {
  completed: "成功",
  completed_partial: "部分成功",
  rolled_back: "取消済み",
};

// 完了=報告、部分成功=注意、取消=警告。他の取込み画面と同じ意味にそろえる。
const STATUS_TONE: Record<CsvImportStatus, "success" | "warning" | "destructive"> = {
  completed: "success",
  completed_partial: "warning",
  rolled_back: "destructive",
};
const STATUS_ICON = {
  completed: CheckCircle2,
  completed_partial: AlertTriangle,
  rolled_back: CircleAlert,
};

const ERROR_TYPE_LABELS = {
  unmatched_product: "商品未一致",
  invalid_format: "形式エラー",
  invalid_jan: "JANエラー",
  invalid_number: "数値エラー",
} as const;

export function CsvImportRecordDetailPage({ importId, returnTo }: CsvImportRecordDetailPageProps) {
  // 取消の結果と取消の mutation は表示している取込みのものだけ（65 §65.5a）。同じ route の param が
  // 変わっても component は再 mount されないので、取込みID で作り直す
  return <CsvImportRecordDetailBody key={importId} importId={importId} returnTo={returnTo} />;
}

function CsvImportRecordDetailBody({ importId, returnTo }: CsvImportRecordDetailPageProps) {
  const backLinkProps = returnToLinkProps(returnTo) ?? { to: "/inventory/records", search: {} };
  const detailQuery = useQuery({
    queryKey: queryKeys.inventoryRecords.csvImportDetail(importId),
    queryFn: () =>
      unwrapResult(commands.getCsvImportRecord(importId), {
        source: "commands",
        cmd: "get_csv_import_record",
      }),
    staleTime: 30_000,
    gcTime: 5 * 60_000,
    retry: 0,
  });
  // 取消の結果（55 の「取消の結果」）。Toast・C9 の再取得・失敗の Toast は共有の hook（55 UI-07-D18）
  const [rollbackResult, setRollbackResult] = useState<RollbackResult | null>(null);
  const rollbackMutation = useRollbackCsvImport({ onSuccess: setRollbackResult });

  if (detailQuery.isLoading) {
    return (
      <PageShell>
        <Skeleton className="h-8 w-64" />
        <Skeleton className="h-32 w-full" />
        <Skeleton className="h-48 w-full" />
      </PageShell>
    );
  }

  if (detailQuery.isError) {
    return (
      <PageShell>
        <PageHeader title="CSV取込み詳細" />
        <Alert variant="destructive">
          <AlertTitle>
            {describeError(detailQuery.error, "CSV取込み記録を読み込めませんでした")}
          </AlertTitle>
          <AlertDescription>
            記録IDを確認するか、在庫変動履歴から開き直してください。
          </AlertDescription>
        </Alert>
        <Button asChild variant="outline">
          <Link {...backLinkProps}>
            <ArrowLeft aria-hidden="true" />
            前の画面へ戻る
          </Link>
        </Button>
      </PageShell>
    );
  }

  const detail = detailQuery.data;
  if (!detail) return null;
  const StatusIcon = STATUS_ICON[detail.status];

  return (
    <PageShell>
      <PageHeader
        title={`CSV取込み #${String(detail.id)}`}
        actions={
          <>
            <Button asChild variant="outline">
              <Link {...backLinkProps}>
                <ArrowLeft aria-hidden="true" />
                前の画面へ戻る
              </Link>
            </Button>
            {detail.status !== "rolled_back" ? (
              <RollbackConfirmDialog
                csvImportId={detail.id}
                settlementDate={detail.settlement_date}
                filename={detail.filename}
                totalItems={detail.total_items}
                totalAmount={detail.total_amount}
                triggerLabel="この取込みを取り消す"
                onConfirm={() => {
                  rollbackMutation.mutate(detail.id);
                }}
                isRollingBack={rollbackMutation.isPending}
              />
            ) : null}
          </>
        }
      />

      {rollbackResult !== null ? (
        <div className="space-y-4">
          <RollbackResultCard result={rollbackResult} />
          <Button asChild variant="outline">
            <Link to="/csv-import" search={{ tab: "z004" }}>
              商品別CSV取込み（Z004）を開く
            </Link>
          </Button>
        </div>
      ) : null}

      {detail.status === "rolled_back" ? (
        <Alert>
          <AlertTitle>この取込みは取消済みです</AlertTitle>
          <AlertDescription>
            取消前の明細は記録として表示し、取り消された明細であることを行ごとに示します。
          </AlertDescription>
        </Alert>
      ) : null}

      <section className="rounded-md border p-4">
        <div className="grid gap-3 text-sm sm:grid-cols-4 lg:grid-cols-8">
          <div>
            <span className="text-muted-foreground">精算日</span>
            <div className="font-medium">{detail.settlement_date}</div>
          </div>
          <div className="sm:col-span-2">
            <span className="text-muted-foreground">ファイル名</span>
            <div className="font-medium break-all">{detail.filename}</div>
          </div>
          <div>
            <span className="text-muted-foreground">状態</span>
            <div>
              <Badge variant="outline" tone={STATUS_TONE[detail.status]}>
                <StatusIcon aria-hidden="true" />
                {STATUS_LABELS[detail.status]}
              </Badge>
            </div>
          </div>
          <div>
            <span className="text-muted-foreground">明細数</span>
            <div className="font-medium">{detail.total_items.toLocaleString("ja-JP")} 件</div>
          </div>
          <div>
            <span className="text-muted-foreground">金額合計</span>
            <div className="font-medium">{formatYen(detail.total_amount)}</div>
          </div>
          <div>
            <span className="text-muted-foreground">スキップ件数</span>
            <div className="font-medium">{detail.skipped_count.toLocaleString("ja-JP")} 件</div>
          </div>
          <div>
            <span className="text-muted-foreground">記録日時</span>
            <div className="font-mono font-medium tabular-nums">
              {formatDateTime(detail.imported_at)}
            </div>
          </div>
        </div>
      </section>

      <section className="space-y-3 rounded-md border p-4">
        <h2 className="text-xl font-semibold">明細</h2>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>商品コード</TableHead>
              <TableHead>商品名</TableHead>
              <TableHead>部門</TableHead>
              <TableHead className="text-right">数量</TableHead>
              <TableHead className="text-right">金額</TableHead>
              <TableHead>状態</TableHead>
              <TableHead className="text-right">在庫変動</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {detail.items.map((item) => (
              <TableRow key={item.id}>
                <TableCell className="font-mono font-medium">{item.product_code}</TableCell>
                <TableCell className="min-w-[12rem] whitespace-normal">
                  {item.product_name}
                </TableCell>
                <TableCell>{item.department_name}</TableCell>
                <TableCell className="text-right tabular-nums">
                  {formatStockDisplay(item.quantity, item.stock_unit)}
                </TableCell>
                <TableCell className="text-right tabular-nums">{formatYen(item.amount)}</TableCell>
                <TableCell>
                  {item.is_voided ? (
                    <Badge variant="outline" tone="destructive">
                      <CircleAlert aria-hidden="true" />
                      明細取消済み
                    </Badge>
                  ) : (
                    <span className="text-muted-foreground">有効</span>
                  )}
                </TableCell>
                <TableCell className="text-right">
                  <Link
                    className="font-medium text-primary underline-offset-4 hover:underline"
                    to="/stock/$code/movements"
                    params={{ code: item.product_code }}
                  >
                    {item.product_code} の在庫変動履歴
                  </Link>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </section>

      <section className="space-y-3 rounded-md border p-4">
        <h2 className="text-xl font-semibold">取込みエラー行</h2>
        {detail.error_rows.length === 0 ? (
          <EmptyState
            icon={FileWarning}
            title="取込みエラーはありません"
            description="この取込みでスキップされたエラー行はありません"
          />
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="text-right">行番号</TableHead>
                <TableHead>JAN</TableHead>
                <TableHead>商品名</TableHead>
                <TableHead>数量</TableHead>
                <TableHead>金額</TableHead>
                <TableHead>種別</TableHead>
                <TableHead>内容</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {detail.error_rows.map((row) => (
                <TableRow key={`${String(row.line_no)}-${row.name}`}>
                  <TableCell className="text-right font-mono tabular-nums">{row.line_no}</TableCell>
                  <TableCell className="font-mono">{row.normalized_jan ?? "—"}</TableCell>
                  <TableCell>{row.name}</TableCell>
                  <TableCell>{row.raw_quantity}</TableCell>
                  <TableCell>{row.raw_amount}</TableCell>
                  <TableCell>{ERROR_TYPE_LABELS[row.error_type]}</TableCell>
                  <TableCell className="min-w-[14rem] whitespace-normal">
                    {row.error_message}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </section>

      <section className="space-y-3 rounded-md border p-4">
        <h2 className="text-xl font-semibold">関連する在庫変動</h2>
        {detail.movements.length === 0 ? (
          <EmptyState
            icon={PackageSearch}
            title="関連する在庫変動がありません"
            description="この記録に紐づく有効な在庫変動は見つかりません"
          />
        ) : (
          <MovementTable
            movements={detail.movements}
            stockUnitByProductCode={stockUnitMap(detail.items)}
          />
        )}
      </section>
    </PageShell>
  );
}
