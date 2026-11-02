// src/features/stock-inquiry/components/StockRecheckNotice.test.tsx
//
// D-118 b2（Matrix U10）: 要再確認の案内（58・66 の D-118 の注記、packet の owner の判断事項 (ii)）。

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { commands } from "@/lib/bindings";
import type { CountRecoveryTarget, RecountReason } from "@/lib/bindings";
import { queryKeys } from "@/lib/query-keys";
import { renderWithRouter } from "@/test/render-with-router";
import { StockRecheckNotice } from "./StockRecheckNotice";

vi.mock("@/lib/bindings", () => ({
  commands: { listRecheckProducts: vi.fn() },
}));

const mockListRecheckProducts = vi.mocked(commands.listRecheckProducts);

function target(productCode: string, reasons: RecountReason[]): CountRecoveryTarget {
  return {
    product_code: productCode,
    product_name: `商品 ${productCode}`,
    stock_unit: "pcs",
    stock: 9,
    stock_range_low: 7,
    stock_range_high: 12,
    stocktake_item_id: null,
    stocktake_id: null,
    action: "count_now",
    block_reason: null,
    recount_reasons: reasons,
  };
}

function renderNotice(stockRecheck: boolean) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const rendered = renderWithRouter(
    <QueryClientProvider client={qc}>
      <StockRecheckNotice productCode="U10-REC" stockRecheck={stockRecheck} />
    </QueryClientProvider>,
  );
  // 一覧の読取りが終わる（成功・失敗）まで待つ
  const settled = (status: "success" | "error") =>
    waitFor(() => {
      expect(qc.getQueryState(queryKeys.stocktake.recheckList())?.status).toBe(status);
    });
  return { ...rendered, qc, settled };
}

const OTHER_REASON = "同じ日に違う数が保存されています";
const OWN_REASON = "数えた日に売り買い（入出庫）がありました";

async function expectGuideAndLink() {
  expect(await screen.findByText("数え直しが必要です")).toBeInTheDocument();
  expect(
    screen.getByText("棚卸しの画面で、棚の数を数えて入れ直してください。"),
  ).toBeInTheDocument();
  expect(screen.getByRole("link", { name: "棚卸しの画面で入れ直す" })).toHaveAttribute(
    "href",
    "/stocktake?product=U10-REC",
  );
}

beforeEach(() => {
  mockListRecheckProducts.mockReset();
});

describe("StockRecheckNotice（D-118 b2）", () => {
  it("U10 recheck notice shows reasons from recheck list and stocktake link", async () => {
    // REQ-205 / REQ-301 / REQ-303（Matrix U10）: 当該の target の理由だけを b1 の文で、題・案内・link と出す
    mockListRecheckProducts.mockResolvedValue({
      status: "ok",
      data: [
        target("OTHER", ["conflicting_counts"]),
        target("U10-REC", ["undetermined_settlement", "same_date_movement"]),
      ],
    });
    renderNotice(true);

    await expectGuideAndLink();
    expect(await screen.findAllByText(OWN_REASON)).toHaveLength(1);
    expect(screen.queryByText(OTHER_REASON)).not.toBeInTheDocument();
    expect(screen.getByRole("alert")).toHaveAttribute("data-variant", "warning");
  });

  it("U10 product without recheck shows nothing and does not read the list", () => {
    // REQ-301（Matrix U10 の否定）: 要再確認でない商品は何も出さない
    const { container } = renderNotice(false);
    expect(container).not.toHaveTextContent("数え直しが必要です");
    expect(mockListRecheckProducts).not.toHaveBeenCalled();
  });

  it("U10 list failure or missing target still shows guide and link without reasons", async () => {
    // REQ-301（Matrix U10）: 一覧の読取りの失敗でも詳細を止めず、理由なしで案内と link を出す
    mockListRecheckProducts.mockResolvedValue({
      status: "error",
      error: {
        kind: "internal",
        message: "読取りに失敗しました",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    const failed = renderNotice(true);
    // 失敗が確定した後に題・案内・link を見る（失敗で案内全体を消す回帰を拾う）
    await failed.settled("error");
    await expectGuideAndLink();
    expect(screen.queryByText(OWN_REASON)).not.toBeInTheDocument();
    failed.unmount();

    mockListRecheckProducts.mockResolvedValue({
      status: "ok",
      data: [target("OTHER", ["conflicting_counts"])],
    });
    const missing = renderNotice(true);
    await missing.settled("success");
    await expectGuideAndLink();
    expect(screen.queryByText(OTHER_REASON)).not.toBeInTheDocument();
    expect(screen.queryByText(OWN_REASON)).not.toBeInTheDocument();
  });

  it("U10 refetch failure after success hides cached reasons but keeps guide and link", async () => {
    // REQ-301（Matrix U10）: 保存の後の再取得が失敗したら、cache に残った古い理由を出さない
    mockListRecheckProducts.mockResolvedValueOnce({
      status: "ok",
      data: [target("U10-REC", ["conflicting_counts"])],
    });
    const notice = renderNotice(true);
    expect(await screen.findByText(OTHER_REASON)).toBeInTheDocument();

    mockListRecheckProducts.mockResolvedValue({
      status: "error",
      error: {
        kind: "internal",
        message: "読取りに失敗しました",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    await notice.qc.invalidateQueries({ queryKey: queryKeys.stocktake.recheckList() });
    await notice.settled("error");
    await waitFor(() => {
      expect(screen.queryByText(OTHER_REASON)).not.toBeInTheDocument();
    });
    await expectGuideAndLink();
  });
});
