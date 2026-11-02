import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactElement } from "react";
import { beforeEach, expect, it, vi } from "vitest";
import { renderWithRouter } from "@/test/render-with-router";
import { commands } from "@/lib/bindings";
import type { CountRecoveryTarget, RollbackPreview } from "@/lib/bindings";
import { ResultStep, RollbackResultCard } from "./ResultStep";

vi.mock("@/lib/bindings", () => ({
  commands: { previewRollbackCsvImport: vi.fn() },
}));

const mockPreview = vi.mocked(commands.previewRollbackCsvImport);

function previewOf(overrides: Partial<RollbackPreview> = {}): RollbackPreview {
  return {
    csv_import_id: 42,
    settlement_no: 2,
    recheck_targets: [],
    affected_stocktakes: [],
    ...overrides,
  };
}

/** 取消の確認は開いたときに取消の前の読取り（useQuery）を行う */
function renderWithQuery(ui: ReactElement) {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return renderWithRouter(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>);
}

beforeEach(() => {
  mockPreview.mockReset();
  mockPreview.mockResolvedValue({ status: "ok", data: previewOf() });
});

// 55 の D-118 の節: 範囲は 55 の例（4〜5）と違う値にし、画面が mock の値を出すことを見る
function recheckTarget(overrides: Partial<CountRecoveryTarget> = {}): CountRecoveryTarget {
  return {
    product_code: "P1-SYN",
    product_name: "合成の糸",
    stock_unit: "pcs",
    stock: 9,
    stock_range_low: 9,
    stock_range_high: 11,
    stocktake_item_id: 501,
    stocktake_id: 77,
    action: "active_count",
    block_reason: null,
    recount_reasons: ["undetermined_settlement"],
    ...overrides,
  };
}

it("test_import_result_req401_rollback_dialog_identifies_exact_import_and_sibling_survival", async () => {
  // REQ-401 / I-U7 / SPEC-SDI-D7: rollback対象ID/date/amountとsibling残存をexact表示する。
  // 55 UI-07-D18（Matrix T2）: 確認は共通の部品で、文は売上と在庫の変動を取り消して在庫を求め直す形
  const user = userEvent.setup();
  const onRollback = vi.fn();
  renderWithQuery(
    <ResultStep
      result={{
        csv_import_id: 42,
        status: "completed",
        total_items: 2,
        total_amount: -300,
        skipped_count: 0,
        settlement_gaps: [],
        settlement_date_after_today: false,
        recheck_targets: [],
      }}
      settlementDate="2026-03-21"
      filename="Z004_0002.CSV"
      onRollback={onRollback}
      isRollingBack={false}
    />,
  );
  expect(await screen.findByText("2026-03-21")).toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: "取り消す" }));
  const dialog = screen.getByRole("alertdialog");
  expect(dialog).toHaveTextContent(
    "この取込みだけを取り消します。同じ日の他の取込みは残ります。この取込みの売上と在庫の変動を取り消し、在庫を記録から求め直します。",
  );
  // UI-07-D14 の規定項目（Scope 2 構造化: ID / 精算日 / filename / 件数 / 金額）が
  // ラベル付きで全て残っていることを確認する（T2）。
  expect(within(dialog).getByText("取込み ID")).toBeInTheDocument();
  expect(within(dialog).getByText("42")).toBeInTheDocument();
  expect(within(dialog).getByText("精算日")).toBeInTheDocument();
  expect(within(dialog).getByText("2026-03-21")).toBeInTheDocument();
  expect(dialog).toHaveTextContent("Z004_0002.CSV");
  expect(within(dialog).getByText("取込み件数")).toBeInTheDocument();
  expect(dialog).toHaveTextContent("2 件");
  expect(within(dialog).getByText("合計金額")).toBeInTheDocument();
  expect(dialog).toHaveTextContent("¥-300");
  expect(within(dialog).getByRole("button", { name: "取り消す" })).toHaveAttribute(
    "data-variant",
    "destructive",
  );
  expect(await within(dialog).findByText("精算 2")).toBeInTheDocument();
  await user.click(within(dialog).getByRole("button", { name: "取り消す" }));
  expect(onRollback).toHaveBeenCalledTimes(1);
});

it("REQ-401: 結果の画面の「取り消す」は共通の確認で、結果の取込みID で読む", async () => {
  // 55 UI-07-D18（Matrix U7）: 結果の画面の確認も取消の前の読取りを結果の取込みID で行い、精算回数と要再確認の
  // 一覧を出す。旧い確認の文は残らない
  const user = userEvent.setup();
  mockPreview.mockResolvedValue({
    status: "ok",
    data: previewOf({
      csv_import_id: 57,
      settlement_no: 103,
      recheck_targets: [recheckTarget({ product_name: "合成の取消の糸" })],
    }),
  });
  renderWithQuery(
    <ResultStep
      result={{
        csv_import_id: 57,
        status: "completed",
        total_items: 1,
        total_amount: 1000,
        skipped_count: 0,
        settlement_gaps: [],
        settlement_date_after_today: false,
        recheck_targets: [],
      }}
      settlementDate="2026-03-22"
      filename="Z004_0103.CSV"
      onRollback={vi.fn()}
      isRollingBack={false}
    />,
  );
  await user.click(await screen.findByRole("button", { name: "取り消す" }));
  const dialog = screen.getByRole("alertdialog");

  expect(await within(dialog).findByText("精算 103")).toBeInTheDocument();
  expect(mockPreview).toHaveBeenCalledWith(57);
  expect(
    within(dialog).getByText(
      "取り消した後、次の商品は数え直しが必要です（今すでに必要な商品を含みます）",
    ),
  ).toBeInTheDocument();
  expect(within(dialog).getByText("合成の取消の糸")).toBeInTheDocument();
  expect(dialog).not.toHaveTextContent("この取込みによる在庫数も元に戻ります。");
});

it.each([
  [
    "completed",
    "成功",
    "success",
    ["border-success-border", "bg-success-soft", "text-success-strong"],
  ],
  [
    "completed_partial",
    "部分成功",
    "warning",
    ["border-warning-border", "bg-warning-soft", "text-warning-strong"],
  ],
] as const)("SC5 / REQ-401: %s has its own tone and icon", async (status, label, tone, classes) => {
  renderWithRouter(
    <ResultStep
      result={{
        csv_import_id: 42,
        status,
        total_items: 1,
        total_amount: 500,
        skipped_count: status === "completed_partial" ? 1 : 0,
        settlement_gaps: [],
        settlement_date_after_today: false,
        recheck_targets: [],
      }}
      settlementDate="2026-03-21"
      filename="synthetic.csv"
      onRollback={vi.fn()}
      isRollingBack={false}
    />,
  );
  const badge = (await screen.findByText(label)).closest('[data-slot="badge"]');
  expect(badge).toHaveAttribute("data-variant", "outline");
  expect(badge).toHaveAttribute("data-tone", tone);
  expect(badge).toHaveClass(...classes);
  expect(badge?.querySelector('svg[aria-hidden="true"]')).toBeInTheDocument();
});

it("U11 import result lists recheck targets with range and recount link", async () => {
  renderWithRouter(
    <ResultStep
      result={{
        csv_import_id: 43,
        status: "completed",
        total_items: 1,
        total_amount: 1000,
        skipped_count: 0,
        settlement_gaps: [],
        settlement_date_after_today: false,
        recheck_targets: [
          recheckTarget(),
          recheckTarget({
            product_code: "P4-SYN",
            product_name: "合成の紐",
            stock_unit: "m",
            stock: 450,
            stock_range_low: 450,
            stock_range_high: 520,
            recount_reasons: ["same_date_movement", "conflicting_counts"],
          }),
        ],
      }}
      settlementDate="2026-03-21"
      filename="Z004_0102.CSV"
      onRollback={vi.fn()}
      isRollingBack={false}
    />,
  );
  const list = await screen.findByRole("region", { name: "数え直しが必要な商品" });
  expect(within(list).getByText("棚の数を確かめて入れ直してください。")).toBeInTheDocument();
  const row = within(list).getByRole("row", { name: /合成の糸/ });
  expect(within(row).getByText("9〜11 個")).toBeInTheDocument();
  expect(within(row).getByText("要再確認")).toBeInTheDocument();
  expect(within(row).getByText("数えた日に売り買い（入出庫）がありました")).toBeInTheDocument();
  expect(within(row).getByRole("link", { name: "棚卸しの画面で入れ直す" })).toHaveAttribute(
    "href",
    "/stocktake?product=P1-SYN",
  );
  const lengthRow = within(list).getByRole("row", { name: /合成の紐/ });
  expect(within(lengthRow).getByText("4.5〜5.2 m")).toBeInTheDocument();
  expect(
    within(lengthRow).getByText(
      "数えた日に売り買い（入出庫）がありました。同じ日に違う数が保存されています",
    ),
  ).toBeInTheDocument();
});

it("U11 import result without recheck targets shows no list", async () => {
  renderWithRouter(
    <ResultStep
      result={{
        csv_import_id: 44,
        status: "completed",
        total_items: 1,
        total_amount: 1000,
        skipped_count: 0,
        settlement_gaps: [],
        settlement_date_after_today: false,
        recheck_targets: [],
      }}
      settlementDate="2026-03-21"
      filename="Z004_0101.CSV"
      onRollback={vi.fn()}
      isRollingBack={false}
    />,
  );
  expect(await screen.findByText("取込み完了")).toBeInTheDocument();
  expect(screen.queryByRole("region", { name: "数え直しが必要な商品" })).not.toBeInTheDocument();
});

it("U11 rollback result lists recheck targets", async () => {
  renderWithRouter(
    <RollbackResultCard
      result={{
        success: true,
        voided_sale_count: 3,
        voided_movement_count: 2,
        recheck_targets: [recheckTarget({ stock: 7, stock_range_low: 7, stock_range_high: 8 })],
      }}
    />,
  );
  expect(await screen.findByText("取込みを取り消しました")).toBeInTheDocument();
  expect(screen.getByText("3 件")).toBeInTheDocument();
  expect(screen.getByText("2 件")).toBeInTheDocument();
  const list = screen.getByRole("region", { name: "数え直しが必要な商品" });
  expect(within(list).getByText("7〜8 個")).toBeInTheDocument();
});
