// src/features/stock-inquiry/components/ProductListTable.tsx
//
// 商品一覧テーブル。行クリックで selected URL state を更新し、選択行の直下に
// 詳細を colSpan 展開行としてインライン描画する（StockDetailContent 共用、§58.7）。
// source prop を deriveStockState へ引き渡し、状態列で Badge + icon + 日本語ラベルを表示する。
// 在庫数の色は補助シグナルとして残し、在庫数は単位付き（生地は cm）。
// 設計: docs/function-design/58-ui-stock-inquiry.md §58.7 / §58.8 / §58.10

import { Fragment } from "react";
import type { UseQueryResult } from "@tanstack/react-query";
import type { ProductWithRelations, StockDetail } from "@/lib/bindings";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { cn } from "@/lib/utils";
import type { StockStatus } from "../types";
import { deriveStockState } from "../lib/derive-stock-state";
import { StockDetailContent } from "./StockDetailContent";
import { StockWithRecheck } from "./StockRecheckBadge";
import { StockStatusBadge } from "./StockStatusBadge";

export interface ProductListTableProps {
  items: ProductWithRelations[];
  source: "search" | "low_stock";
  selected: string | null;
  /** 選択行直下のインライン展開に描画する詳細 query（list 成功時のみ展開、§58.8）。 */
  detailQuery: UseQueryResult<StockDetail>;
  onSelect: (productCode: string | null) => void;
}

const STOCK_CLASS: Record<StockStatus, string> = {
  ok: "",
  low: "text-warning-emphasis font-medium",
  stockout: "text-destructive font-medium",
};

// D-CE7: 進行中の地（#E6F0F0）の上では -emphasis が 4.33:1 で AA に届かないため、同じ注意・確認の家族の -strong へ。
const OPEN_LOW_STOCK_CLASS = "text-warning-strong font-medium";

const priceFormatter = new Intl.NumberFormat("ja-JP", {
  style: "currency",
  currency: "JPY",
});

export function ProductListTable({
  items,
  source,
  selected,
  detailQuery,
  onSelect,
}: ProductListTableProps) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>商品コード</TableHead>
          <TableHead>商品名</TableHead>
          <TableHead>部門</TableHead>
          <TableHead>取引先</TableHead>
          <TableHead className="w-24">状態</TableHead>
          <TableHead className="text-right">在庫数</TableHead>
          <TableHead className="text-right">売価</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {items.map((item) => {
          const status = deriveStockState(item, source);
          const isSelected = item.product_code === selected;
          return (
            <Fragment key={item.product_code}>
              <TableRow
                data-state={isSelected ? "selected" : undefined}
                className={
                  isSelected
                    ? "cursor-pointer border-l-4 border-l-ongoing hover:bg-ongoing-soft data-[state=selected]:bg-ongoing-soft"
                    : "cursor-pointer border-l-4 border-l-transparent"
                }
                onClick={() => {
                  onSelect(isSelected ? null : item.product_code);
                }}
              >
                <TableCell className="font-mono text-sm font-medium">{item.product_code}</TableCell>
                <TableCell>{item.name}</TableCell>
                <TableCell className="text-muted-foreground">{item.department_name}</TableCell>
                <TableCell className="text-muted-foreground">{item.supplier_name ?? "—"}</TableCell>
                <TableCell>
                  <StockStatusBadge status={status} />
                </TableCell>
                <TableCell
                  className={cn(
                    "text-right tabular-nums",
                    isSelected && status === "low" ? OPEN_LOW_STOCK_CLASS : STOCK_CLASS[status],
                  )}
                >
                  <StockWithRecheck product={item} />
                </TableCell>
                <TableCell className="text-right tabular-nums">
                  {priceFormatter.format(item.selling_price)}
                </TableCell>
              </TableRow>
              {isSelected && (
                // 展開行は選択行と一体に見せるため、進行中のバーと地を明示固定する（D-CE7）
                // （table primitive の data-state / has-aria-expanded トリガに依存しない、New-1）。
                <TableRow className="border-l-4 border-l-ongoing bg-ongoing-soft hover:bg-ongoing-soft">
                  {/* table primitive 既定の whitespace-nowrap を打ち消し、詳細コンテンツを通常折り返し
                      させる（Codex 実装レビュー Round 1 P2-1、長い商品名 / CTA 群の横はみ出し防止） */}
                  <TableCell colSpan={7} className="p-0 align-top whitespace-normal">
                    <StockDetailContent query={detailQuery} />
                  </TableCell>
                </TableRow>
              )}
            </Fragment>
          );
        })}
      </TableBody>
    </Table>
  );
}
