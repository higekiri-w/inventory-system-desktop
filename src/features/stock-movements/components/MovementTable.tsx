// src/features/stock-movements/components/MovementTable.tsx
//
// REQ-303 / REQ-207: 業務記録詳細の「関連する在庫変動」の movement table（在庫変動の画面は StockHistoryTable。増減と元記録の欄を共有する）。
// 設計: docs/function-design/66-ui-stock-movements.md §66.5

import { ArrowDown, ArrowRight, ArrowUp } from "lucide-react";
import { Link } from "@tanstack/react-router";

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
import type {
  MovementRecord,
  MovementSourceLink as MovementSourceLinkData,
  ProductStockUnit,
} from "@/lib/bindings";
import {
  formatMovementDateTime,
  formatMovementQuantity,
  formatMovementType,
} from "../lib/movement-formatters";

export interface MovementTableProps {
  movements: MovementRecord[];
  /// 行の商品の単位（`product_code → stock_unit`）。対応に無い商品コードの行は整数で出す
  stockUnitByProductCode: ReadonlyMap<string, ProductStockUnit>;
  returnTo?: string;
}

/// 商品か記録詳細の明細から、表に渡す単位の対応を作る
export function stockUnitMap(
  items: readonly { product_code: string; stock_unit: ProductStockUnit }[],
): ReadonlyMap<string, ProductStockUnit> {
  return new Map(items.map((item) => [item.product_code, item.stock_unit]));
}

function QuantityDirectionIcon({ quantity }: { quantity: number }) {
  if (quantity > 0) return <ArrowUp aria-hidden="true" className="text-success" />;
  if (quantity < 0) return <ArrowDown aria-hidden="true" className="text-destructive" />;
  return <ArrowRight aria-hidden="true" className="text-muted-foreground" />;
}

/// 増減の欄（在庫変動の画面の履歴の表と共有する。66）
export function MovementQuantity({
  quantity,
  unit,
}: {
  quantity: number;
  unit?: ProductStockUnit;
}) {
  const formatted = formatMovementQuantity(quantity, unit);
  return (
    <div className="flex items-center gap-2">
      <QuantityDirectionIcon quantity={quantity} />
      <span className="font-mono font-medium tabular-nums">{formatted.value}</span>
      <span className="text-xs text-muted-foreground">{formatted.label}</span>
    </div>
  );
}

// movement.source.route は DTO 由来の runtime 文字列 (compile-time に既知の route union
// ではない)。batch A packet P2-2 の採用案どおり <Link to={string}> で SPA 遷移のみ保証し、
// to は pathname のみ・returnTo は search object 経由で付与する。
function sourceLinkProps(
  route: string,
  returnTo: string | undefined,
): { to: string; search?: { returnTo: string } } {
  if (returnTo === undefined) return { to: route };
  return { to: route, search: { returnTo } };
}

/// 元記録の欄（在庫変動の画面の履歴の表と共有する。66）
export function MovementSourceLink({
  source,
  returnTo,
}: {
  source: MovementSourceLinkData | null;
  returnTo?: string;
}) {
  return source ? (
    <Link
      className="font-medium text-primary underline-offset-4 hover:underline"
      {...sourceLinkProps(source.route, returnTo)}
    >
      {source.label}
    </Link>
  ) : (
    <span className="text-muted-foreground">元記録なし</span>
  );
}

export function MovementTable({ movements, stockUnitByProductCode, returnTo }: MovementTableProps) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>日時</TableHead>
          <TableHead>種別</TableHead>
          <TableHead>増減</TableHead>
          <TableHead className="text-right">変動後在庫</TableHead>
          <TableHead>元記録</TableHead>
          <TableHead>備考</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {movements.map((movement) => {
          const unit = stockUnitByProductCode.get(movement.product_code);
          return (
            <TableRow key={movement.id}>
              <TableCell className="font-mono tabular-nums">
                {formatMovementDateTime(movement.created_at)}
              </TableCell>
              <TableCell>
                <Badge variant="outline">{formatMovementType(movement.movement_type)}</Badge>
              </TableCell>
              <TableCell>
                <MovementQuantity quantity={movement.quantity} unit={unit} />
              </TableCell>
              <TableCell className="text-right font-mono tabular-nums">
                {formatLengthOrPlain(movement.stock_after, unit)}
              </TableCell>
              <TableCell>
                <MovementSourceLink source={movement.source} returnTo={returnTo} />
              </TableCell>
              <TableCell
                className="max-w-80 break-words whitespace-normal"
                title={movement.note?.trim() ? movement.note : undefined}
              >
                {movement.note?.trim() ? movement.note : "—"}
              </TableCell>
            </TableRow>
          );
        })}
      </TableBody>
    </Table>
  );
}
