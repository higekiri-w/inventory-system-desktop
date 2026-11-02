import { screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { UseQueryResult } from "@tanstack/react-query";
import type { StockDetail } from "@/lib/bindings";

import { renderWithRouter } from "@/test/render-with-router";
import { StockDetailContent } from "./StockDetailContent";
import { StockDetailCard } from "./StockDetailCard";
import { makeMockProductWithRelations, makeMockStockDetail } from "../lib/test-fixtures";

// U3: 要再確認の一覧は hook ごと差し替える（一覧の読み方は StockRecheckNotice.test.tsx の U10）
vi.mock("@/features/stocktake/hooks/useRecheckProducts", () => ({
  useRecheckProducts: () => ({
    data: [
      {
        product_code: "U3-REC",
        product_name: "要再確認の商品",
        stock_unit: "pcs",
        stock: 9,
        stock_range_low: 7,
        stock_range_high: 12,
        stocktake_item_id: null,
        stocktake_id: null,
        action: "count_now",
        block_reason: null,
        recount_reasons: ["same_date_movement"],
      },
    ],
  }),
}));

describe("StockDetailContent (REQ-301 -> REQ-303)", () => {
  it.each([StockDetailContent, StockDetailCard])(
    "⑰ SC5 / UIDISP-D5: %s preserves detail padding",
    async (Component) => {
      const data = makeMockStockDetail({
        product: makeMockProductWithRelations({ product_code: "BT0002", name: "ボタン #02" }),
      });
      renderWithRouter(
        <Component
          query={{ isLoading: false, isError: false, data } as UseQueryResult<StockDetail>}
        />,
      );
      const cta = await screen.findByRole("link", { name: "商品修正" });
      expect(screen.getByText("ボタン #02").closest('[data-slot="card-header"]')).toHaveClass(
        "pt-4",
      );
      expect(cta.parentElement).toHaveClass("pb-4");
      expect(screen.getByRole("link", { name: "入庫記録" })).toBeVisible();
    },
  );

  it("REQ-301 / UI-06a-D7: 在庫変動履歴 link が現在の /stock URL を returnTo で送る", async () => {
    const data = makeMockStockDetail({
      product: makeMockProductWithRelations({ product_code: "BT0002", name: "ボタン #02" }),
    });

    renderWithRouter(
      <StockDetailContent
        query={{ isLoading: false, isError: false, data } as UseQueryResult<StockDetail>}
      />,
      "/stock?q=BT&selected=BT0002",
    );

    expect(await screen.findByRole("link", { name: "在庫変動履歴" })).toHaveAttribute(
      "href",
      "/stock/BT0002/movements?returnTo=%2Fstock%3Fq%3DBT%26selected%3DBT0002",
    );
  });

  it("Matrix C19 / SPEC-UNIT-D5: 原価 8333 の商品の詳細の原価は ¥83.33（8,333 を含まない）", async () => {
    const data = makeMockStockDetail({
      product: makeMockProductWithRelations({ product_code: "C19-A", cost_price_centi: 8333 }),
    });
    const { container } = renderWithRouter(
      <StockDetailContent
        query={{ isLoading: false, isError: false, data } as UseQueryResult<StockDetail>}
      />,
    );
    expect(await screen.findByText("¥83.33")).toBeInTheDocument();
    expect(container).not.toHaveTextContent("8,333");
  });

  it("REQ-301: StockDetailContent shows active product edit link", async () => {
    const data = makeMockStockDetail({
      product: makeMockProductWithRelations({ product_code: "BT0002", name: "ボタン #02" }),
    });

    renderWithRouter(
      <StockDetailContent
        query={{ isLoading: false, isError: false, data } as UseQueryResult<StockDetail>}
      />,
    );

    expect(await screen.findByRole("link", { name: "商品修正" })).toHaveAttribute(
      "href",
      "/products/BT0002/edit",
    );
  });

  it("REQ-301: StockDetailContent shows active receiving link", async () => {
    const data = makeMockStockDetail({
      product: makeMockProductWithRelations({ product_code: "BT0002", name: "ボタン #02" }),
    });

    renderWithRouter(
      <StockDetailContent
        query={{ isLoading: false, isError: false, data } as UseQueryResult<StockDetail>}
      />,
    );

    expect(await screen.findByRole("link", { name: "入庫記録" })).toHaveAttribute(
      "href",
      "/inventory/receiving",
    );
  });
});

describe("StockDetailContent の在庫数（D-118 b2、58）", () => {
  it("U3 detail stock row shows range and recheck badge", async () => {
    // REQ-301 / SPEC-STK-CNT-D3（Matrix U3）: 詳細の「在庫数」に範囲と badge、その下に要再確認の案内
    const data = makeMockStockDetail({
      product: makeMockProductWithRelations({
        product_code: "U3-REC",
        stock_quantity: 9,
        stock_recheck: true,
        stock_range_low: 7,
        stock_range_high: 12,
      }),
    });
    renderWithRouter(
      <StockDetailContent
        query={{ isLoading: false, isError: false, data } as UseQueryResult<StockDetail>}
      />,
    );
    const stockRow = (await screen.findByText("在庫数")).parentElement;
    if (stockRow === null) throw new Error("row not found");
    expect(stockRow).toHaveTextContent("7〜12 個");
    expect(stockRow).toHaveTextContent("要再確認");
    expect(screen.getByText("数え直しが必要です")).toBeInTheDocument();
    expect(screen.getByText("数えた日に売り買い（入出庫）がありました")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "棚卸しの画面で入れ直す" })).toBeInTheDocument();
  });

  it("U3 detail of product without recheck shows single stock and no notice", async () => {
    // REQ-301（Matrix U3 の否定）: 要再確認でない商品は 1 つの値、badge と案内なし
    const data = makeMockStockDetail({
      product: makeMockProductWithRelations({ product_code: "U3-PLN", stock_quantity: 3 }),
    });
    renderWithRouter(
      <StockDetailContent
        query={{ isLoading: false, isError: false, data } as UseQueryResult<StockDetail>}
      />,
    );
    expect(await screen.findByText("3 個")).toBeInTheDocument();
    expect(screen.queryByText("要再確認")).not.toBeInTheDocument();
    expect(screen.queryByText("数え直しが必要です")).not.toBeInTheDocument();
  });
});
