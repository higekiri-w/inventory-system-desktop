// src/features/products/components/ProductTable.tsx
//
// UI-01a-D6: 商品コードと商品名を併記し、在庫単位と廃番状態を明示する。

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Link } from "@tanstack/react-router";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { PRODUCT_CODE_CELL_WRAPPER_WIDTH_CLASS } from "@/components/patterns/ListShell";
import { StockWithRecheck } from "@/features/stock-inquiry/components/StockRecheckBadge";
import { formatCostYen } from "@/features/stock-inquiry/lib/format-stock-display";
import type { ProductWithRelations } from "@/lib/bindings";
import { CircleCheck, CircleMinus, Clock3 } from "lucide-react";

export interface ProductTableProps {
  items: ProductWithRelations[];
  returnTo?: string;
}

const yenFormatter = new Intl.NumberFormat("ja-JP", {
  style: "currency",
  currency: "JPY",
  maximumFractionDigits: 0,
});

export function ProductTable({ items, returnTo = "/products" }: ProductTableProps) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead className="min-w-36">
            <div className={PRODUCT_CODE_CELL_WRAPPER_WIDTH_CLASS}>商品コード</div>
          </TableHead>
          <TableHead>商品名</TableHead>
          <TableHead>部門</TableHead>
          <TableHead className="text-right">売価</TableHead>
          <TableHead className="text-right">原価</TableHead>
          <TableHead className="text-right">在庫数</TableHead>
          <TableHead>PLU</TableHead>
          <TableHead className="text-right">操作</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {items.map((item) => (
          <TableRow
            key={item.product_code}
            className={item.is_discontinued ? "text-muted-foreground" : undefined}
          >
            <TableCell className="min-w-36 font-mono text-sm font-medium">
              <div
                className={`${PRODUCT_CODE_CELL_WRAPPER_WIDTH_CLASS} break-all whitespace-normal`}
              >
                {item.product_code}
              </div>
            </TableCell>
            <TableCell className="min-w-[14rem] whitespace-normal">
              <div className="flex flex-wrap items-center gap-2">
                <span className="font-medium">{item.name}</span>
                {item.is_discontinued && <Badge variant="secondary">廃番</Badge>}
              </div>
              {item.jan_code !== null && (
                <div className="text-xs text-muted-foreground">JAN {item.jan_code}</div>
              )}
            </TableCell>
            <TableCell>{item.department_name}</TableCell>
            <TableCell className="text-right tabular-nums">
              {yenFormatter.format(item.selling_price)}
            </TableCell>
            <TableCell className="text-right tabular-nums">
              {formatCostYen(item.cost_price_centi)}
            </TableCell>
            <TableCell className="text-right tabular-nums">
              <StockWithRecheck product={item} />
            </TableCell>
            <TableCell>
              {!item.plu_target ? (
                <Badge variant="outline" className="gap-1 whitespace-nowrap">
                  <CircleMinus className="size-3" aria-hidden="true" />
                  対象外
                </Badge>
              ) : item.plu_dirty ? (
                // 未反映は注意、反映済みは完了報告。既存の時計/check iconと対にする。
                <Badge variant="outline" tone="warning" className="gap-1 whitespace-nowrap">
                  <Clock3 className="size-3" aria-hidden="true" />
                  未反映
                </Badge>
              ) : (
                <Badge variant="outline" tone="success" className="gap-1 whitespace-nowrap">
                  <CircleCheck className="size-3" aria-hidden="true" />
                  反映済み
                </Badge>
              )}
            </TableCell>
            <TableCell className="text-right">
              <Button type="button" variant="outline" size="sm" asChild>
                <Link
                  to="/products/$code/edit"
                  params={{ code: item.product_code }}
                  search={{ returnTo }}
                >
                  修正
                </Link>
              </Button>
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
