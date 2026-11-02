import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, createRouter, RouterProvider } from "@tanstack/react-router";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { MovementRecord, StocktakeRecordDetail } from "@/lib/bindings";
import { commands } from "@/lib/bindings";
import { routeTree } from "@/routeTree.gen";
import { renderWithRouter } from "@/test/render-with-router";
import { StocktakeRecordDetailPage } from "./StocktakeRecordDetailPage";

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ setTitle: vi.fn().mockResolvedValue(undefined) }),
}));

vi.mock("@/features/stock-movements/StockMovementsPage", async () => {
  const { MovementTable } = await vi.importActual<
    typeof import("@/features/stock-movements/components/MovementTable")
  >("@/features/stock-movements/components/MovementTable");
  return {
    StockMovementsPage: () => (
      <MovementTable movements={[makeMovement()]} stockUnitByProductCode={new Map()} />
    ),
  };
});

vi.mock("@/lib/bindings", () => ({
  commands: { getStocktakeRecord: vi.fn() },
}));

const mockGetStocktakeRecord = vi.mocked(commands.getStocktakeRecord);

function makeMovement(): MovementRecord {
  return {
    id: 701,
    product_code: "SRD-001",
    movement_type: "stocktake",
    quantity: -2,
    stock_after: 8,
    reference_type: "stocktake",
    reference_id: 51,
    source: { label: "棚卸し #51", route: "/stocktake/records/51" },
    note: "synthetic correction",
    created_at: "2026-08-27T18:00:00",
  };
}

function makeDetail(overrides: Partial<StocktakeRecordDetail> = {}): StocktakeRecordDetail {
  return {
    id: 51,
    started_at: "2026-08-27T09:00:00",
    completed_at: "2026-08-27T18:00:00",
    status: "completed",
    total_cost: 2400,
    item_count: 3,
    items: [
      {
        product_code: "SRD-001",
        product_name: "合成棚卸し商品",
        department_name: "テスト部門",
        stock_unit: "pcs",
        count_value: 8,
        count_kind: "default",
        count_business_date: "2026-08-27",
        difference: -2,
        valuation_quantity: 8,
        valuation_cost_price_centi: 30000,
        loss_cost_centi: "60000",
      },
    ],
    after_completion: null,
    ...overrides,
  };
}

function makeQueryClient() {
  return new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
}

function renderWithClient(ui: ReactNode) {
  return renderWithRouter(
    <QueryClientProvider client={makeQueryClient()}>{ui}</QueryClientProvider>,
  );
}

beforeEach(() => mockGetStocktakeRecord.mockReset());

describe("StocktakeRecordDetailPage (REQ-206 / REQ-207)", () => {
  it("REQ-206: completed のヘッダ・明細・商品リンクを表示する", async () => {
    mockGetStocktakeRecord.mockResolvedValue({ status: "ok", data: makeDetail() });

    renderWithClient(<StocktakeRecordDetailPage stocktakeId={51} />);

    expect(await screen.findByRole("heading", { name: "棚卸し #51" })).toBeInTheDocument();
    expect(screen.getByText("完了")).toBeInTheDocument();
    expect(screen.getByText("2026-08-27 09:00:00")).toBeInTheDocument();
    // 完了日時の 1 か所（「関連する在庫変動」の節は外した。65）
    expect(screen.getAllByText("2026-08-27 18:00:00")).toHaveLength(1);
    expect(screen.getByText("3 件")).toBeInTheDocument();
    expect(screen.queryByText("補正件数")).not.toBeInTheDocument();
    expect(screen.getByText("¥2,400")).toBeInTheDocument();
    const row = screen.getByRole("row", { name: /SRD-001 合成棚卸し商品/ });
    expect(within(row).getByText("テスト部門")).toBeInTheDocument();
    expect(within(row).getAllByText("8 個")).toHaveLength(2);
    expect(within(row).getByText("2026-08-27")).toBeInTheDocument();
    expect(within(row).getByText("-2 個")).toBeInTheDocument();
    expect(within(row).getByText("¥300")).toBeInTheDocument();
    expect(within(row).getByText("600.00 円")).toBeInTheDocument();
    expect(within(row).getByRole("link", { name: "SRD-001 の在庫変動履歴" })).toHaveAttribute(
      "href",
      "/stock/SRD-001/movements",
    );
    expect(within(row).getByRole("link", { name: "SRD-001 の今の数を入れ直す" })).toHaveAttribute(
      "href",
      "/stocktake?product=SRD-001",
    );
    expect(screen.queryByRole("button", { name: /取消|訂正/ })).not.toBeInTheDocument();
  });

  it("U13 record detail shows counts and after-completion diff", async () => {
    // 65・35「完了」: 明細ごとの数・差異・評価数量・BIZ のロス原価、完了の後の差（確定と要再確認を分け合計）、
    // 「在庫変動」の節は無い。mock の値は画面で求められない値にする（ロス原価は -2 × 300 円と違う）
    const base = makeDetail().items[0];
    mockGetStocktakeRecord.mockResolvedValue({
      status: "ok",
      data: makeDetail({
        items: [
          { ...base, loss_cost_centi: "123456" },
          {
            ...base,
            product_code: "SRD-002",
            product_name: "入れ直した糸",
            count_value: 450,
            count_kind: "fresh",
            count_business_date: "2026-08-26",
            difference: -50,
            valuation_quantity: 450,
            stock_unit: "m",
            valuation_cost_price_centi: 15000,
            loss_cost_centi: "7500",
          },
        ],
        after_completion: {
          confirmed: [
            {
              product_code: "SRD-003",
              product_name: "後から入庫した糸",
              stock_unit: "pcs",
              valuation_quantity: 8,
              stock_at_completion: 9,
              stock_at_completion_low: null,
              stock_at_completion_high: null,
              valuation_diff_yen: "-500",
              valuation_diff_low_yen: null,
              valuation_diff_high_yen: null,
            },
          ],
          recheck: [
            {
              product_code: "SRD-004",
              product_name: "決まらない糸",
              stock_unit: "pcs",
              valuation_quantity: 8,
              stock_at_completion: 8,
              stock_at_completion_low: 7,
              stock_at_completion_high: 9,
              valuation_diff_yen: "0",
              valuation_diff_low_yen: "-500",
              valuation_diff_high_yen: "500",
            },
          ],
          confirmed_total_yen: "-1234500",
        },
      }),
    });
    renderWithClient(<StocktakeRecordDetailPage stocktakeId={51} />);

    const row = await screen.findByRole("row", { name: /SRD-001 合成棚卸し商品/ });
    expect(within(row).getByText("1,234.56 円")).toBeInTheDocument();
    expect(within(row).queryByText("600.00 円")).not.toBeInTheDocument();
    const freshRow = screen.getByRole("row", { name: /SRD-002 入れ直した糸/ });
    expect(within(freshRow).getAllByText("4.5 m")).toHaveLength(2);
    expect(within(freshRow).getByText("入れ直し")).toBeInTheDocument();
    expect(within(freshRow).getByText("2026-08-26")).toBeInTheDocument();
    expect(within(freshRow).getByText("-0.5 m")).toBeInTheDocument();
    expect(within(freshRow).getByText("75.00 円")).toBeInTheDocument();

    const after = screen.getByRole("region", { name: "完了の後に入った記録による差" });
    expect(
      within(after).getByRole("heading", {
        name: "完了の後に入った記録による差（参考。評価額は変わりません）",
      }),
    ).toBeInTheDocument();
    const confirmedRow = within(after).getByRole("row", { name: /SRD-003 後から入庫した糸/ });
    expect(within(confirmedRow).getByText("9 個")).toBeInTheDocument();
    expect(within(confirmedRow).getByText("-¥500")).toBeInTheDocument();
    expect(within(after).getByText("確定の差の合計: -¥1,234,500")).toBeInTheDocument();
    expect(within(after).getByText("要再確認の差（合計に入れません）")).toBeInTheDocument();
    const recheckRow = within(after).getByRole("row", { name: /SRD-004 決まらない糸/ });
    expect(within(recheckRow).getByText("7 個〜9 個 要再確認")).toBeInTheDocument();
    expect(within(recheckRow).getByText("-¥500〜¥500")).toBeInTheDocument();
    expect(screen.getByText("¥2,400")).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "関連する在庫変動" })).not.toBeInTheDocument();
  });

  it("U13 after-completion diff with no confirmed rows shows an empty list and ¥0", async () => {
    mockGetStocktakeRecord.mockResolvedValue({
      status: "ok",
      data: makeDetail({
        after_completion: { confirmed: [], recheck: [], confirmed_total_yen: "0" },
      }),
    });
    renderWithClient(<StocktakeRecordDetailPage stocktakeId={51} />);
    const after = await screen.findByRole("region", { name: "完了の後に入った記録による差" });
    expect(within(after).getByText("確定の差はありません")).toBeInTheDocument();
    expect(within(after).getByText("確定の差の合計: ¥0")).toBeInTheDocument();
    expect(within(after).queryByText("要再確認の差（合計に入れません）")).not.toBeInTheDocument();
  });

  it("Matrix C18: 評価原価 8333 は ¥83.33、上限のロス原価は BIZ の 1/100 円の文字列のまま、ロスの無い行は —", async () => {
    const base = makeDetail().items[0];
    mockGetStocktakeRecord.mockResolvedValue({
      status: "ok",
      data: makeDetail({
        items: [
          {
            ...base,
            product_code: "C18-A",
            product_name: "小数の原価",
            valuation_cost_price_centi: 8333,
            loss_cost_centi: "16666",
          },
          {
            ...base,
            product_code: "C18-B",
            product_name: "上限の原価",
            count_value: 0,
            difference: -999999999,
            valuation_quantity: 0,
            valuation_cost_price_centi: 99999999,
            loss_cost_centi: "99999998900000001",
          },
          {
            ...base,
            product_code: "C18-C",
            product_name: "ロスなし",
            count_value: 10,
            difference: 0,
            valuation_quantity: 10,
            loss_cost_centi: null,
          },
        ],
      }),
    });
    renderWithClient(<StocktakeRecordDetailPage stocktakeId={51} />);
    const rowA = await screen.findByRole("row", { name: /C18-A 小数の原価/ });
    expect(within(rowA).getByText("¥83.33")).toBeInTheDocument();
    expect(within(rowA).getByText("166.66 円")).toBeInTheDocument();
    const rowB = screen.getByRole("row", { name: /C18-B 上限の原価/ });
    expect(within(rowB).getByText("999,999,989,000,000.01 円")).toBeInTheDocument();
    const rowC = screen.getByRole("row", { name: /C18-C ロスなし/ });
    expect(within(rowC).getAllByText("—")).toHaveLength(1);
    expect(within(rowC).queryByText("0.00 円")).not.toBeInTheDocument();
  });

  it("REQ-206: 正の差異を符号付きで表示する", async () => {
    const positiveItem = {
      ...makeDetail().items[0],
      count_value: 12,
      difference: 2,
      loss_cost_centi: null,
    };
    mockGetStocktakeRecord.mockResolvedValue({
      status: "ok",
      data: makeDetail({ items: [positiveItem] }),
    });

    renderWithClient(<StocktakeRecordDetailPage stocktakeId={51} />);

    const row = await screen.findByRole("row", { name: /SRD-001 合成棚卸し商品/ });
    expect(within(row).getByText("+2 個")).toBeInTheDocument();
  });

  it("REQ-207: movement の元記録をクリックすると棚卸し詳細 route を描画する", async () => {
    mockGetStocktakeRecord.mockResolvedValue({ status: "ok", data: makeDetail() });
    const user = userEvent.setup();
    const router = createRouter({
      routeTree,
      history: createMemoryHistory({ initialEntries: ["/stock/SRD-001/movements"] }),
    });
    render(
      <QueryClientProvider client={makeQueryClient()}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    );

    await user.click(await screen.findByRole("link", { name: "棚卸し #51" }));

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/stocktake/records/51");
    });
    await waitFor(() => {
      expect(mockGetStocktakeRecord).toHaveBeenCalledWith(51);
    });
    expect(await screen.findByRole("heading", { name: "棚卸し #51" })).toBeInTheDocument();
  });

  it("REQ-206: NotFound を利用者向け日本語で表示する", async () => {
    mockGetStocktakeRecord.mockResolvedValue({
      status: "error",
      error: {
        kind: "not_found",
        message: "棚卸し記録が見つかりません: 404",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });

    renderWithClient(<StocktakeRecordDetailPage stocktakeId={404} />);

    expect(await screen.findByText("棚卸し記録が見つかりません: 404")).toBeInTheDocument();
  });

  it("REQ-206: in_progress を空の明細と算定前原価で正常表示する", async () => {
    mockGetStocktakeRecord.mockResolvedValue({
      status: "ok",
      data: makeDetail({
        completed_at: null,
        status: "in_progress",
        total_cost: null,
        items: [],
        after_completion: null,
      }),
    });

    renderWithClient(<StocktakeRecordDetailPage stocktakeId={51} />);

    expect(await screen.findByText("進行中")).toBeInTheDocument();
    expect(screen.getByText("明細はありません")).toBeInTheDocument();
    expect(
      screen.queryByText("完了の後に入った記録による差（参考。評価額は変わりません）"),
    ).toBeNull();
    expect(screen.getAllByText("—").length).toBeGreaterThanOrEqual(2);
  });

  it.each([
    [
      "/stock/SRD-001/movements?type=stocktake&page=2",
      "/stock/SRD-001/movements?type=stocktake&page=2",
    ],
    ["https://example.invalid/escape", "/inventory/records"],
    ["//example.invalid/escape", "/inventory/records"],
  ])("REQ-207: returnTo %s を安全に %s へ正規化する", async (returnTo, expected) => {
    mockGetStocktakeRecord.mockResolvedValue({ status: "ok", data: makeDetail() });

    renderWithClient(<StocktakeRecordDetailPage stocktakeId={51} returnTo={returnTo} />);

    expect(await screen.findByRole("link", { name: "前の画面へ戻る" })).toHaveAttribute(
      "href",
      expected,
    );
  });
});

it.each([
  ["pcs", "1,234 個", "0 個"],
  ["m", "12.34 m", "0 m"],
] as const)(
  "⑮ SC19: 棚卸し詳細の %s と未入力・差異ゼロを表示する",
  async (unit, expected, zero) => {
    const detail = makeDetail();
    detail.items = [
      {
        ...detail.items[0],
        stock_unit: unit,
        valuation_quantity: 1234,
        count_value: null,
        count_business_date: null,
        difference: 0,
        loss_cost_centi: null,
      },
    ];
    mockGetStocktakeRecord.mockResolvedValue({ status: "ok", data: detail });
    renderWithClient(<StocktakeRecordDetailPage stocktakeId={51} />);
    const row = (await screen.findByText(expected)).closest("tr");
    if (row === null) throw new Error("expected table structure");
    // 未計数は数・数えた日・ロス原価が「—」
    expect(within(row).getAllByText("—")).toHaveLength(3);
    expect(within(row).getByText(zero)).toBeInTheDocument();
    expect(within(row).queryByText("+" + zero)).not.toBeInTheDocument();
  },
);
