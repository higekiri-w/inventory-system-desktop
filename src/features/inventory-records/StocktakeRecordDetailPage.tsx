// REQ-206 / REQ-207: 棚卸し記録詳細。

import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { AlertTriangle, ArrowLeft, ClipboardList, Info } from "lucide-react";

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
// 数量の桁区切りと単位名は共通formatterに揃える。
import {
  formatCentiYen,
  formatCostYen,
  formatStockDisplay,
} from "@/features/stock-inquiry/lib/format-stock-display";
import type {
  AfterCompletionDiff,
  AfterCompletionDiffItem,
  ProductStockUnit,
  StocktakeStatus,
} from "@/lib/bindings";
import { commands } from "@/lib/bindings";
import { describeError } from "@/lib/describe-error";
import { unwrapResult } from "@/lib/invoke";
import { queryKeys } from "@/lib/query-keys";
import { returnToLinkProps } from "@/lib/return-to";
import { formatDateTime } from "@/lib/date-time";
import { formatYen } from "./types";

export interface StocktakeRecordDetailPageProps {
  stocktakeId: number;
  returnTo?: string;
}

const STATUS_LABELS: Record<StocktakeStatus, string> = {
  in_progress: "進行中",
  completed: "完了",
};

function formatSignedQuantity(value: number, unit: ProductStockUnit): string {
  const sign = value > 0 ? "+" : "";
  return `${sign}${formatStockDisplay(value, unit)}`;
}

/** 円の十進の文字列（符号つき。範囲の決まらない読取りの金額。35「完了」）を `¥1,000`・`-¥500` の形にする */
function formatYenString(value: string): string {
  const match = /^(-?)([0-9]+)$/.exec(value);
  if (match === null) return "—";
  const grouped = match[2].replace(/^0+(?=\d)/, "").replace(/\B(?=(\d{3})+$)/g, ",");
  return `${match[1]}¥${grouped}`;
}

function formatRange(low: number, high: number, unit: ProductStockUnit): string {
  return `${formatStockDisplay(low, unit)}〜${formatStockDisplay(high, unit)}`;
}

function formatOptionalQuantity(value: number | null, unit: ProductStockUnit): string {
  return value === null ? "—" : formatStockDisplay(value, unit);
}

export function StocktakeRecordDetailPage({
  stocktakeId,
  returnTo,
}: StocktakeRecordDetailPageProps) {
  const backLinkProps = returnToLinkProps(returnTo) ?? { to: "/inventory/records", search: {} };
  const detailQuery = useQuery({
    queryKey: queryKeys.inventoryRecords.stocktakeDetail(stocktakeId),
    queryFn: () =>
      unwrapResult(commands.getStocktakeRecord(stocktakeId), {
        source: "commands",
        cmd: "get_stocktake_record",
      }),
    staleTime: 30_000,
    gcTime: 5 * 60_000,
    retry: 0,
  });

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
        <PageHeader title="棚卸し詳細" />
        <Alert variant="destructive">
          <AlertTitle>
            {describeError(detailQuery.error, "棚卸し記録を読み込めませんでした")}
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

  return (
    <PageShell>
      <PageHeader
        title={`棚卸し #${String(detail.id)}`}
        actions={
          <Button asChild variant="outline">
            <Link {...backLinkProps}>
              <ArrowLeft aria-hidden="true" />
              前の画面へ戻る
            </Link>
          </Button>
        }
      />

      <section className="rounded-md border p-4">
        <div className="grid gap-3 text-sm sm:grid-cols-4 lg:grid-cols-7">
          <div>
            <span className="text-muted-foreground">開始日時</span>
            <div className="font-mono font-medium tabular-nums">
              {formatDateTime(detail.started_at)}
            </div>
          </div>
          <div>
            <span className="text-muted-foreground">完了日時</span>
            <div className="font-mono font-medium tabular-nums">
              {detail.completed_at === null ? "—" : formatDateTime(detail.completed_at)}
            </div>
          </div>
          <div>
            <span className="text-muted-foreground">状態</span>
            <div>
              <Badge variant="outline">{STATUS_LABELS[detail.status]}</Badge>
            </div>
          </div>
          <div>
            <span className="text-muted-foreground">対象商品数</span>
            <div className="font-medium">{detail.item_count.toLocaleString("ja-JP")} 件</div>
          </div>
          <div className="sm:col-span-2">
            <span className="text-muted-foreground">仕入原価総額</span>
            <div className="font-medium">
              {detail.total_cost === null ? "—" : formatYen(detail.total_cost)}
            </div>
          </div>
        </div>
      </section>

      <section className="space-y-3 rounded-md border p-4">
        <h2 className="text-xl font-semibold">明細</h2>
        {detail.items.length === 0 ? (
          <EmptyState
            icon={ClipboardList}
            title="明細はありません"
            description="この棚卸しの対象の商品はありません"
          />
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>商品コード</TableHead>
                <TableHead>商品名</TableHead>
                <TableHead>部門</TableHead>
                <TableHead className="text-right">数</TableHead>
                <TableHead>数えた日</TableHead>
                <TableHead className="text-right">差異</TableHead>
                <TableHead className="text-right">評価数量</TableHead>
                <TableHead className="text-right">評価原価</TableHead>
                <TableHead className="text-right">ロス原価</TableHead>
                <TableHead>操作</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {detail.items.map((item) => (
                <TableRow key={item.product_code}>
                  <TableCell className="font-mono font-medium">{item.product_code}</TableCell>
                  <TableCell className="min-w-[12rem] whitespace-normal">
                    {item.product_name}
                  </TableCell>
                  <TableCell>{item.department_name}</TableCell>
                  <TableCell className="text-right tabular-nums">
                    {formatOptionalQuantity(item.count_value, item.stock_unit)}
                  </TableCell>
                  <TableCell className="tabular-nums">
                    {item.count_business_date ?? "—"}
                    {item.count_kind === "fresh" ? (
                      <Badge variant="outline" className="ml-2">
                        入れ直し
                      </Badge>
                    ) : null}
                  </TableCell>
                  <TableCell className="text-right font-medium tabular-nums">
                    {item.difference === null
                      ? "—"
                      : formatSignedQuantity(item.difference, item.stock_unit)}
                  </TableCell>
                  <TableCell className="text-right tabular-nums">
                    {formatOptionalQuantity(item.valuation_quantity, item.stock_unit)}
                  </TableCell>
                  <TableCell className="text-right tabular-nums">
                    {item.valuation_cost_price_centi === null
                      ? "—"
                      : formatCostYen(item.valuation_cost_price_centi)}
                  </TableCell>
                  <TableCell className="text-right tabular-nums">
                    {/* BIZ が求めた 1/100 円の十進の文字列をそのまま出す（画面で掛けない。65） */}
                    {item.loss_cost_centi === null ? "—" : formatCentiYen(item.loss_cost_centi)}
                  </TableCell>
                  <TableCell className="space-y-1">
                    <Link
                      className="block font-medium text-primary underline-offset-4 hover:underline"
                      to="/stock/$code/movements"
                      params={{ code: item.product_code }}
                    >
                      {item.product_code} の在庫変動履歴
                    </Link>
                    <Link
                      className="block font-medium text-primary underline-offset-4 hover:underline"
                      to="/stocktake"
                      search={{ product: item.product_code }}
                    >
                      {item.product_code} の今の数を入れ直す
                    </Link>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </section>

      {detail.after_completion !== null ? (
        <AfterCompletionSection diff={detail.after_completion} />
      ) : null}
    </PageShell>
  );
}

/** 65・35「完了」: 完了の後に入った記録による差（参考）。確定の差と要再確認の差を分け、確定の差だけ合計する */
function AfterCompletionSection({ diff }: { diff: AfterCompletionDiff }) {
  return (
    <section className="space-y-4 rounded-md border p-4" aria-label="完了の後に入った記録による差">
      <h2 className="text-xl font-semibold">
        完了の後に入った記録による差（参考。評価額は変わりません）
      </h2>
      <p className="flex items-center gap-1.5 text-sm text-muted-foreground">
        <Info aria-hidden="true" className="size-4" />
        完了した評価額は書き換えません。完了の後に入れた、完了の日より前の記録による差を参考として出します。
      </p>
      <div className="space-y-2">
        <h3 className="font-semibold">確定の差</h3>
        {diff.confirmed.length === 0 ? (
          <p className="text-sm">確定の差はありません</p>
        ) : (
          <AfterCompletionTable items={diff.confirmed} kind="confirmed" />
        )}
        <p className="font-medium">確定の差の合計: {formatYenString(diff.confirmed_total_yen)}</p>
      </div>
      {diff.recheck.length > 0 ? (
        <div className="space-y-2">
          <h3 className="flex items-center gap-1.5 font-semibold">
            <AlertTriangle aria-hidden="true" className="size-4" />
            要再確認の差（合計に入れません）
          </h3>
          <AfterCompletionTable items={diff.recheck} kind="recheck" />
        </div>
      ) : null}
    </section>
  );
}

function AfterCompletionTable({
  items,
  kind,
}: {
  items: AfterCompletionDiffItem[];
  kind: "confirmed" | "recheck";
}) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>商品コード</TableHead>
          <TableHead>商品名</TableHead>
          <TableHead className="text-right">評価数量</TableHead>
          <TableHead className="text-right">完了の時点の在庫</TableHead>
          <TableHead className="text-right">評価額の差</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {items.map((item) => {
          const hasRange =
            kind === "recheck" &&
            item.stock_at_completion_low !== null &&
            item.stock_at_completion_high !== null;
          return (
            <TableRow key={item.product_code}>
              <TableCell className="font-mono font-medium">{item.product_code}</TableCell>
              <TableCell>{item.product_name}</TableCell>
              <TableCell className="text-right tabular-nums">
                {formatStockDisplay(item.valuation_quantity, item.stock_unit)}
              </TableCell>
              <TableCell className="text-right tabular-nums">
                {hasRange &&
                item.stock_at_completion_low !== null &&
                item.stock_at_completion_high !== null
                  ? `${formatRange(item.stock_at_completion_low, item.stock_at_completion_high, item.stock_unit)} 要再確認`
                  : formatStockDisplay(item.stock_at_completion, item.stock_unit)}
              </TableCell>
              <TableCell className="text-right tabular-nums">
                {kind === "recheck" &&
                item.valuation_diff_low_yen !== null &&
                item.valuation_diff_high_yen !== null
                  ? `${formatYenString(item.valuation_diff_low_yen)}〜${formatYenString(item.valuation_diff_high_yen)}`
                  : formatYenString(item.valuation_diff_yen)}
              </TableCell>
            </TableRow>
          );
        })}
      </TableBody>
    </Table>
  );
}
