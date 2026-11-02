// src/features/stock-movements/lib/movement-formatters.ts
//
// REQ-303 / UI-06c-D4-D5: movement 種別と増減の operator-facing 表示。

import {
  formatLengthOrPlain,
  formatStockDisplay,
} from "@/features/stock-inquiry/lib/format-stock-display";
import type { MovementType, ProductStockUnit, StockCountHistory } from "@/lib/bindings";

const movementTypeLabels: Record<MovementType, string> = {
  receiving: "入庫",
  return: "返品・交換",
  sale_auto: "POS売上",
  sale_manual: "手動販売",
  disposal: "廃棄・破損",
  stocktake: "棚卸し",
};

export function formatMovementType(movementType: MovementType): string {
  return movementTypeLabels[movementType];
}

// 長さの商品は m の形、個数の商品と単位の分からない行は今どおり整数（SPEC-UNIT-D3 の表の 6）
export function formatMovementQuantity(
  quantity: number,
  unit?: ProductStockUnit,
): { value: string; label: string } {
  const value = formatLengthOrPlain(quantity, unit);
  if (quantity > 0) {
    return { value: `+${value}`, label: "増加" };
  }
  if (quantity < 0) {
    return { value, label: "減少" };
  }
  return { value, label: "変動なし" };
}

export function formatMovementDateTime(value: string): string {
  const match = /^(\d{4}-\d{2}-\d{2})[T ](\d{2}:\d{2}:\d{2})/.exec(value);
  if (match) {
    return `${match[1]} ${match[2]}`;
  }
  return value;
}

// D-118 b2: 計数の行の種別の文（65）。{数} は在庫の欄と同じ形を行の単位で（66）
export function formatCountRow(count: StockCountHistory, businessDate: string): string {
  const label = count.count_kind === "fresh" ? "数を入れ直し" : "数を確認";
  return `${label}（${formatStockDisplay(count.count_value, count.stock_unit)}、${businessDate}）`;
}

// D-118 b2: 「日付」の列の下の保存の日時（YYYY-MM-DD HH:mm。66）
export function formatSavedAt(value: string): string {
  const match = /^(\d{4}-\d{2}-\d{2})[T ](\d{2}:\d{2})/.exec(value);
  return `保存 ${match ? `${match[1]} ${match[2]}` : value}`;
}
