// src/features/stock-inquiry/components/StockRecheckBadge.tsx
//
// D-118 b2（ADR SPEC-STK-CNT-D3）: 商品の画面の在庫の欄。写しが要再確認なら範囲（b1 の formatStockOrRange）と
// 「要再確認」の①状態の badge（warning の tone・icon・文字。色だけにしない）。要再確認でなければ今の 1 つの値。
// 設計: docs/function-design/50・51・58・66 の D-118 の注記、design-system/02 の badge 3 種構成

import { TriangleAlertIcon } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { formatStockOrRange } from "@/features/stocktake/lib/stocktake-formatters";
import type { ProductWithRelations } from "@/lib/bindings";

type StockCopy = Pick<
  ProductWithRelations,
  "stock_quantity" | "stock_unit" | "stock_recheck" | "stock_range_low" | "stock_range_high"
>;

export function StockRecheckBadge() {
  return (
    <Badge variant="outline" tone="warning" className="font-medium">
      <TriangleAlertIcon aria-hidden="true" />
      要再確認
    </Badge>
  );
}

/** 在庫の欄（b1 の formatStockOrRange の文字と、要再確認なら badge） */
export function StockWithRecheck({ product }: { product: StockCopy }) {
  return (
    <span className="inline-flex flex-wrap items-center justify-end gap-2">
      <span>
        {formatStockOrRange({
          stock: product.stock_quantity,
          stock_range_low: product.stock_range_low,
          stock_range_high: product.stock_range_high,
          stock_unit: product.stock_unit,
        })}
      </span>
      {product.stock_recheck ? <StockRecheckBadge /> : null}
    </span>
  );
}
