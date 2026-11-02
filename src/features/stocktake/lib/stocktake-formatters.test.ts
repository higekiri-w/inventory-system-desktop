// src/features/stocktake/lib/stocktake-formatters.test.ts
//
// REQ-205 / UI-10-D10

import { describe, expect, it } from "vitest";

import {
  formatCountedAt,
  formatListDifference,
  formatRecountReasons,
  formatSettlementGap,
  formatStockOrRange,
} from "./stocktake-formatters";

describe("formatListDifference", () => {
  it("req205 null は「—」を返す", () => {
    expect(formatListDifference(null)).toBe("—");
  });

  it("req205 正数には + を付ける", () => {
    expect(formatListDifference(3)).toBe("+3");
  });

  it("req205 負数はそのまま表示する", () => {
    expect(formatListDifference(-3)).toBe("-3");
  });

  it("req205 0 はそのまま表示する", () => {
    expect(formatListDifference(0)).toBe("0");
  });
});

describe("formatCountedAt", () => {
  it("req205 null は「—」を返す", () => {
    expect(formatCountedAt(null)).toBe("—");
  });

  it("req205 T区切りをスペースに変換する", () => {
    expect(formatCountedAt("2026-10-01T09:05:00")).toBe("2026-10-01 09:05:00");
  });
});

describe("formatStockOrRange (73・55 の範囲)", () => {
  it("req205 範囲が無ければ写しの在庫を単位つきで出す", () => {
    expect(
      formatStockOrRange({
        stock: 7,
        stock_range_low: null,
        stock_range_high: null,
        stock_unit: "pcs",
      }),
    ).toBe("7 個");
  });

  it("req205 範囲は「9〜11 個」、長さは m の形「4.5〜5.2 m」", () => {
    expect(
      formatStockOrRange({ stock: 9, stock_range_low: 9, stock_range_high: 11, stock_unit: "pcs" }),
    ).toBe("9〜11 個");
    expect(
      formatStockOrRange({
        stock: 450,
        stock_range_low: 450,
        stock_range_high: 520,
        stock_unit: "m",
      }),
    ).toBe("4.5〜5.2 m");
  });
});

describe("formatRecountReasons / formatSettlementGap", () => {
  it("req205 理由 1・2 は同じ文で 1 つにまとめ、理由 3 は別の文", () => {
    expect(
      formatRecountReasons(["undetermined_settlement", "same_date_movement", "conflicting_counts"]),
    ).toEqual(["数えた日に売り買い（入出庫）がありました", "同じ日に違う数が保存されています"]);
  });

  it("req401 欠けの区間は 1 つなら「精算 n」、続けば「精算 n〜m」", () => {
    expect(formatSettlementGap({ from: 103, to: 103 })).toBe("精算 103");
    expect(formatSettlementGap({ from: 105, to: 199 })).toBe("精算 105〜199");
  });
});
