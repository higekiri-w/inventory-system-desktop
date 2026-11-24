// src/features/ej-receipts/lib/receipt-format.ts
//
// UI-09c-D2・D4 の表示の純関数と、控えの文の字形の指定（この 1 か所だけに置く）。
// 設計: docs/function-design/80-ui-ej-receipts.md §80.4

import type { EjRecordKindCode, EjRecordModeCode } from "@/lib/bindings";

/**
 * 控えの文の字形。1 行 24 バイト（半角 1・全角 2）の並びを崩さない Windows の和文の等幅を先頭に置く
 * （`font-mono` は和文の等幅を持たない）。どれが当たったかは L3-1 で確かめて 80 §80.4 に書く
 */
export const EJ_RECEIPT_FONT_FAMILY = '"BIZ UDGothic", "MS Gothic", monospace';

const KIND_LABEL: Record<EjRecordKindCode, string> = {
  sale: "取引",
  cancelled: "取引中止",
  cash_movement: "入金・出金",
  receipt: "領収書",
  settlement: "精算",
  inspection: "点検",
  settings: "設定",
  training: "練習",
  unclassified: "その他",
};

/** 種類の label（返品はモードが戻の取引） */
export function ejKindLabel(kind: EjRecordKindCode, mode: EjRecordModeCode): string {
  if (kind === "sale" && mode === "return") return "返品";
  return KIND_LABEL[kind];
}

/** 合計（復元できなかった記録は「—」、0 円は「0円」） */
export function formatEjAmount(amount: number | null): string {
  return amount === null ? "—" : `${amount.toLocaleString("ja-JP")}円`;
}

/** `YYYY-MM-DD HH:MM` の時刻（`HH:MM`） */
export function ejTime(printedAt: string): string {
  return printedAt.slice(11, 16);
}
