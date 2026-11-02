// src/features/stock-movements/components/StockHistoryTable.tsx
//
// REQ-303 / REQ-207 / D-118 b2: 在庫変動の画面の履歴の表（movement と計数の行を BIZ の順のまま描く）。
// 種別の label・増減・元記録の書き方は MovementTable と共有する。
// 設計: docs/function-design/66-ui-stock-movements.md の D-118 の注記

import { Badge } from "@/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { formatLengthOrPlain } from "@/features/stock-inquiry/lib/format-stock-display";
import type { ProductStockUnit, StockHistoryItem } from "@/lib/bindings";
import { formatCountRow, formatMovementType, formatSavedAt } from "../lib/movement-formatters";
import { MovementQuantity, MovementSourceLink } from "./MovementTable";

export interface StockHistoryTableProps {
  items: StockHistoryItem[];
  /// movement の行の商品の単位（`product_code → stock_unit`。UI-06c-D5）。計数の行は行の単位で描く
  stockUnitByProductCode: ReadonlyMap<string, ProductStockUnit>;
  returnTo?: string;
}

export function StockHistoryTable({
  items,
  stockUnitByProductCode,
  returnTo,
}: StockHistoryTableProps) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>日付</TableHead>
          <TableHead>種別</TableHead>
          <TableHead>増減</TableHead>
          <TableHead className="text-right">変動後在庫</TableHead>
          <TableHead>元記録</TableHead>
          <TableHead>備考</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {items.map((item) => {
          const { movement, count } = item;
          const unit = movement ? stockUnitByProductCode.get(movement.product_code) : undefined;
          const note = movement?.note?.trim() ? movement.note : null;
          return (
            <TableRow key={`${item.kind}-${String(movement?.id ?? count?.id)}`}>
              <TableCell className="font-mono tabular-nums">
                <div>
                  {item.business_date}
                  {/* 精算の番号を付けるのは Z004 の movement の行だけ（44） */}
                  {movement && item.settlement_no !== null
                    ? `（精算 ${String(item.settlement_no)}）`
                    : null}
                </div>
                <div className="text-xs text-muted-foreground">{formatSavedAt(item.saved_at)}</div>
              </TableCell>
              <TableCell>
                <Badge variant="outline">
                  {movement
                    ? formatMovementType(movement.movement_type)
                    : count
                      ? formatCountRow(count, item.business_date)
                      : null}
                </Badge>
              </TableCell>
              <TableCell>
                {movement ? <MovementQuantity quantity={movement.quantity} unit={unit} /> : "—"}
              </TableCell>
              <TableCell className="text-right font-mono tabular-nums">
                {movement ? formatLengthOrPlain(movement.stock_after, unit) : "—"}
              </TableCell>
              <TableCell>
                <MovementSourceLink
                  source={movement?.source ?? count?.source ?? null}
                  returnTo={returnTo}
                />
              </TableCell>
              <TableCell
                className="max-w-80 break-words whitespace-normal"
                title={note ?? undefined}
              >
                {note ?? "—"}
              </TableCell>
            </TableRow>
          );
        })}
      </TableBody>
    </Table>
  );
}
