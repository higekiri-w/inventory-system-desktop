// src/features/stocktake/lib/stocktake-formatters.ts
//
// REQ-205 / UI-10-D10 / D-118: 棚卸しの一覧・要再確認の一覧・取込みの結果の表示用純関数。
// 差異は BIZ が求めた値（35「読取り」の `difference`）を出し、画面で計算しない。

import { formatStockDisplay } from "@/features/stock-inquiry/lib/format-stock-display";
import type { ProductStockUnit, RecountReason, SettlementGap } from "@/lib/bindings";
import { MAX_STOCK_QUANTITY, type QuantityField } from "@/lib/unit-quantity";

/**
 * 差異を符号付き文字列にする（例: 3 → "+3"、-2 → "-2"、0 → "0"）。未入力は「—」。
 */
export function formatListDifference(difference: number | null): string {
  if (difference === null) return "—";
  if (difference > 0) return `+${String(difference)}`;
  return String(difference);
}

/**
 * 棚卸しの日時（開始・完了）を表示する。null は「—」、値ありは `T` 区切りをスペースに変換する
 * （`src/features/stock-movements/lib/movement-formatters.ts` の `formatMovementDateTime` と同じ変換）。
 */
export function formatCountedAt(value: string | null): string {
  if (value === null) return "—";
  const match = /^(\d{4}-\d{2}-\d{2})[T ](\d{2}:\d{2}:\d{2})/.exec(value);
  if (match) {
    return `${match[1]} ${match[2]}`;
  }
  return value;
}

/** 計数の数量欄（Z1: 0 以上 在庫の上限まで。m の商品は m の小数 2 桁を cm に直す。SPEC-UNIT-D3） */
export const STOCK_COUNT_FIELD: QuantityField = {
  min: 0,
  max: MAX_STOCK_QUANTITY,
  tooLarge: "数が大きすぎます",
  integerError: "0以上の数値を入力してください",
};

/** 写しの在庫（要再確認なら範囲「4〜5」の形。長さの商品は m。73・55） */
export function formatStockOrRange(target: {
  stock: number;
  stock_range_low: number | null;
  stock_range_high: number | null;
  stock_unit: ProductStockUnit;
}): string {
  if (target.stock_range_low === null || target.stock_range_high === null) {
    return formatStockDisplay(target.stock, target.stock_unit);
  }
  const high = formatStockDisplay(target.stock_range_high, target.stock_unit);
  const low = formatStockDisplay(target.stock_range_low, target.stock_unit);
  // 単位の語は後ろに 1 回だけ付ける（「9〜10 個」「4.5〜5 m」）
  const unitSuffix = high.slice(high.lastIndexOf(" "));
  return `${low.endsWith(unitSuffix) ? low.slice(0, -unitSuffix.length) : low}〜${high}`;
}

/** 要再確認の理由の文（40 の `RecountReason`。理由 1・2 は 55 の文と同じ語で、重なれば 1 つにする） */
export function formatRecountReasons(reasons: readonly RecountReason[]): string[] {
  const labels = reasons.map((reason) => {
    switch (reason) {
      case "undetermined_settlement":
      case "same_date_movement":
        return "数えた日に売り買い（入出庫）がありました";
      case "conflicting_counts":
        return "同じ日に違う数が保存されています";
    }
  });
  return [...new Set(labels)];
}

/** 欠けた精算回数の区間の文（32・55 の知らせ。1 つなら「精算 n」） */
export function formatSettlementGap(gap: SettlementGap): string {
  return gap.from === gap.to
    ? `精算 ${String(gap.from)}`
    : `精算 ${String(gap.from)}〜${String(gap.to)}`;
}
