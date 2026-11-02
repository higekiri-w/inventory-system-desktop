// src/features/stock-movements/components/StockHistoryTable.test.tsx
//
// D-118 b2（Matrix U6・U7）: 在庫変動の画面の履歴の表（計数の行と「日付」の列）。
// 設計: docs/function-design/66-ui-stock-movements.md の D-118 の注記、65 の D-118 の節

import { screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import type { MovementRecord, StockCountHistory, StockHistoryItem } from "@/lib/bindings";
import { renderWithRouter } from "@/test/render-with-router";
import { StockHistoryTable } from "./StockHistoryTable";

function countItem(
  count: Partial<StockCountHistory> & Pick<StockCountHistory, "id">,
  row: Partial<StockHistoryItem> = {},
): StockHistoryItem {
  return {
    kind: "count",
    business_date: "2026-10-02",
    settlement_no: null,
    saved_at: "2026-10-02T09:00:00",
    movement: null,
    count: {
      count_value: 8,
      stock_unit: "pcs",
      count_kind: "default",
      stocktake_id: null,
      source: null,
      ...count,
    },
    ...row,
  };
}

function movementItem(
  movement: Partial<MovementRecord> & Pick<MovementRecord, "id">,
  row: Partial<StockHistoryItem> = {},
): StockHistoryItem {
  return {
    kind: "movement",
    business_date: "2026-10-02",
    settlement_no: null,
    saved_at: "2026-10-02T09:00:00",
    count: null,
    movement: {
      product_code: "U6-PCS",
      movement_type: "receiving",
      quantity: 1,
      stock_after: 9,
      reference_type: null,
      reference_id: null,
      source: null,
      note: null,
      created_at: "2026-10-02T09:00:00",
      ...movement,
    },
    ...row,
  };
}

function rowOf(text: string): HTMLElement {
  const row = screen.getByText(text).closest("tr");
  if (row === null) throw new Error(`row not found: ${text}`);
  return row;
}

describe("StockHistoryTable（D-118 b2）", () => {
  it("U6 count rows show 65 wording with dash quantity and stocktake link", async () => {
    // REQ-303 / REQ-207 / SPEC-UNIT-D3（Matrix U6）: 計数の行は 65 の文（{数} は行の単位で在庫の欄の形）、
    // 増減と変動後在庫は「—」、元記録は棚卸しの link か「元記録なし」
    renderWithRouter(
      <StockHistoryTable
        items={[
          countItem(
            {
              id: 1,
              count_value: 8,
              count_kind: "fresh",
              stocktake_id: 5,
              source: { label: "棚卸し #5", route: "/stocktake/records/5" },
            },
            { settlement_no: 102 },
          ),
          countItem(
            { id: 2, count_value: 450, stock_unit: "m", count_kind: "default" },
            { business_date: "2026-10-01" },
          ),
        ]}
        stockUnitByProductCode={new Map()}
      />,
    );

    await screen.findByText("数を入れ直し（8 個、2026-10-02）");
    const fresh = rowOf("数を入れ直し（8 個、2026-10-02）");
    expect(fresh).not.toHaveTextContent("（精算");
    expect(within(fresh).getAllByText("—")).toHaveLength(3);
    expect(within(fresh).getByRole("link", { name: "棚卸し #5" })).toHaveAttribute(
      "href",
      "/stocktake/records/5",
    );
    const length = rowOf("数を確認（4.5 m、2026-10-01）");
    expect(within(length).getByText("元記録なし")).toBeInTheDocument();
    expect(length).not.toHaveTextContent("450");
  });

  it("U7 date column shows business date settlement number and saved time", async () => {
    // REQ-303（Matrix U7）: 「日付」は業務日、Z004 の行だけ「（精算 {n}）」、下に保存の日時。
    // movement の行の種別・増減・元記録は MovementTable と同じ書き方
    renderWithRouter(
      <StockHistoryTable
        items={[
          countItem({ id: 3, count_kind: "fresh" }, { settlement_no: 102 }),
          movementItem(
            {
              id: 11,
              movement_type: "sale_auto",
              quantity: -1,
              stock_after: 8,
              source: { label: "CSV取込み #3", route: "/csv-import/records/3" },
              created_at: "2026-10-02T09:15:42",
            },
            {
              business_date: "2026-10-01",
              settlement_no: 101,
              saved_at: "2026-10-02T09:15:42",
            },
          ),
        ]}
        stockUnitByProductCode={new Map([["U6-PCS", "pcs"]])}
      />,
    );

    await screen.findByText("POS売上");
    const z004 = rowOf("POS売上");
    expect(within(z004).getByText("2026-10-01（精算 101）")).toBeInTheDocument();
    expect(within(z004).getByText("保存 2026-10-02 09:15")).toBeInTheDocument();
    expect(within(z004).getByText("-1")).toBeInTheDocument();
    expect(within(z004).getByText("減少")).toBeInTheDocument();
    expect(within(z004).getByRole("link", { name: "CSV取込み #3" })).toBeInTheDocument();
    const fresh = rowOf("数を入れ直し（8 個、2026-10-02）");
    expect(fresh).not.toHaveTextContent("（精算");
    expect(within(fresh).getByText("保存 2026-10-02 09:00")).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: "日付" })).toBeInTheDocument();
  });
});
