// src/features/monthly-sales/MonthlySalesPage.test.tsx
//
// REQ-502 P2-2: search.sortBy/sortDir URL state → useMonthlySalesReport 連動 +
// handleSortChange (同列再 click desc toggle / 別列 asc) の Page level 結線テスト。
// container component (MonthlySalesPage) import 経由で、canonical owner
// `src/components/sales/SortableHeader.tsx` の結線を確認する。
// 設計: docs/plans/2026-05-19-pr-66-codex-r1-p2-fixes.md §2 commit 2

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, within } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import { commands } from "@/lib/bindings";
import type { MonthlySalesReport } from "@/lib/bindings";

import { MonthlySalesPage } from "./MonthlySalesPage";
import type { MonthlySalesSearch } from "./MonthlySalesPage";

// TabsHeader は内部で TanStack Router `<Link>` を使うため RouterProvider context が必要。
// Page test の責務は sort 結線で TabsHeader 単体の動作確認は別 (scope 外)、null mock で置換。
vi.mock("@/components/sales/TabsHeader", () => ({
  TabsHeader: () => null,
}));

// useExportFile は内部で commands binding を使う、Page test では export 関連は scope 外。
vi.mock("@/lib/hooks/useExportFile", () => ({
  useExportFile: () => ({ exportFile: vi.fn(), isExporting: false }),
}));

vi.mock("@/lib/bindings", () => ({
  commands: {
    getMonthlySales: vi.fn(),
  },
}));

const mockGetMonthlySales = vi.mocked(commands.getMonthlySales);

function buildReport(): MonthlySalesReport {
  return {
    month: "2026-05",
    mode: "by_product",
    items: [
      {
        key: "P001",
        label: "商品A",
        count_points: 1,
        length_cm: 0,
        stock_unit: "pcs",
        amount: 1000,
        ranking: 1,
      },
      {
        key: "P002",
        label: "商品B",
        count_points: 5,
        length_cm: 0,
        stock_unit: "pcs",
        amount: 5000,
        ranking: 2,
      },
      {
        key: "P003",
        label: "商品C",
        count_points: 2,
        length_cm: 0,
        stock_unit: "pcs",
        amount: 3000,
        ranking: 3,
      },
    ],
    prev_month_comparison: null,
    official_department_totals: null,
  };
}

function renderPage(search: MonthlySalesSearch, onSearchChange = vi.fn()) {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  return {
    onSearchChange,
    ...render(
      <QueryClientProvider client={qc}>
        <MonthlySalesPage search={search} onSearchChange={onSearchChange} />
      </QueryClientProvider>,
    ),
  };
}

beforeEach(() => {
  mockGetMonthlySales.mockResolvedValue({ status: "ok", data: buildReport() });
});

// B0 characterization: 月度空結果の EmptyState DOM 固定（意図的差分③）
// bare div → EmptyState 標準 UI に置換。既存 2 文（当月データなし / 月を変更して…）を
// title(h3) + description(p) に正規移植（文言維持）する。
describe("MonthlySalesPage (B0 empty-state characterization)", () => {
  it("B0-monthly-empty: items=[] のとき EmptyState の title「当月データなし」と description「月を変更してお試しください。」が表示される", async () => {
    mockGetMonthlySales.mockResolvedValue({
      status: "ok",
      data: {
        month: "2026-06",
        mode: "by_product",
        items: [],
        prev_month_comparison: null,
        official_department_totals: null,
      },
    });

    renderPage({ month: "2026-06", mode: "by_product" });

    // 既存 2 文が title(h3) + description(p) に分割されて表示される
    expect(await screen.findByRole("heading", { name: "当月データなし" })).toBeInTheDocument();
    expect(screen.getByText("月を変更してお試しください。")).toBeInTheDocument();
  });
});

describe("MonthlySalesPage_req502 official department totals", () => {
  it("test_monthly_sales_page_official_department_totals_req502", async () => {
    mockGetMonthlySales.mockResolvedValue({
      status: "ok",
      data: {
        ...buildReport(),
        official_department_totals: [
          { department_id: 1, label: "その他小物", quantity: 6, count: 3, amount: 4000 },
        ],
      },
    });

    renderPage({ month: "2026-05", mode: "by_product" });

    expect(
      await screen.findByRole("heading", { name: "公式部門集計（レジ日報由来）" }),
    ).toBeInTheDocument();
    expect(screen.getByText("日報取込み済み日の Z005 部門別売上合計です。")).toBeInTheDocument();
    expect(screen.getByText("その他小物")).toBeInTheDocument();
    expect(screen.getByText("¥4,000")).toBeInTheDocument();
    expect(screen.getByText("商品A")).toBeInTheDocument();
  });

  it("test_monthly_sales_page_no_official_note_req502", async () => {
    renderPage({ month: "2026-05", mode: "by_product" });

    expect(await screen.findByText("この月のレジ日報は未取込みです。")).toBeInTheDocument();
    // ⑰ SC3 / UIDISP-D3: 対象 warning 内だけで Z004 との対比を検証する。
    const warning = screen.getByRole("status");
    expect(within(warning).getByText("この月のレジ日報は未取込みです。")).toBeInTheDocument();
    expect(within(warning).getByText("商品別売上 CSV（Z004）の取込みとは別です。")).toHaveAttribute(
      "data-slot",
      "alert-description",
    );
    // SC20 / DSR-08: 既存文言・roleを保ってwarning variantへ移行する。
    expect(
      screen.getByText("この月のレジ日報は未取込みです。").closest('[data-slot="alert"]'),
    ).toHaveAttribute("data-variant", "warning");
    expect(
      screen.getByText("この月のレジ日報は未取込みです。").closest('[data-slot="alert"]'),
    ).toHaveAttribute("role", "status");
    // SC22: 元の文をAlertTitleとして保持する。
    expect(screen.getByText("この月のレジ日報は未取込みです。")).toHaveAttribute(
      "data-slot",
      "alert-title",
    );
    expect(
      screen
        .getByText("この月のレジ日報は未取込みです。")
        .closest('[data-slot="alert"]')
        ?.querySelector('svg[aria-hidden="true"]'),
    ).toBeInTheDocument();
    expect(screen.getByText("商品A")).toBeInTheDocument();
  });
});

describe("MonthlySalesPage (REQ-502 sort URL state 接続)", () => {
  it("REQ-502: search.sortBy='amount' sortDir='desc' を渡すと ProductRankingTable の行順が金額降順になる", async () => {
    renderPage({ month: "2026-05", mode: "by_product", sortBy: "amount", sortDir: "desc" });

    // データ行の出現を待つ (header row のみだと findAllByRole がすぐ返るので、商品名を anchor)
    await screen.findByText("商品B");
    const rows = screen.getAllByRole("row");
    const rowTexts = rows.map((r) => r.textContent ?? "");
    const idxA = rowTexts.findIndex((t) => t.includes("商品A"));
    const idxB = rowTexts.findIndex((t) => t.includes("商品B"));
    const idxC = rowTexts.findIndex((t) => t.includes("商品C"));
    // amount desc 順 = B (5000) → C (3000) → A (1000)
    expect(idxB).toBeGreaterThan(-1);
    expect(idxC).toBeGreaterThan(-1);
    expect(idxA).toBeGreaterThan(-1);
    expect(idxB).toBeLessThan(idxC);
    expect(idxC).toBeLessThan(idxA);
  });

  it("REQ-502: 別列を初回 click → onSearchChange updater が { sortBy, sortDir: 'asc' } を返す", async () => {
    const onSearchChange = vi.fn();
    renderPage({ month: "2026-05", mode: "by_product" }, onSearchChange);

    const amountHeader = await screen.findByRole("button", { name: /金額/ });
    fireEvent.click(amountHeader);

    expect(onSearchChange).toHaveBeenCalledTimes(1);
    const updater = onSearchChange.mock.calls[0]?.[0] as (
      prev: MonthlySalesSearch,
    ) => MonthlySalesSearch;
    expect(updater({ month: "2026-05", mode: "by_product" })).toEqual({
      month: "2026-05",
      mode: "by_product",
      sortBy: "amount",
      sortDir: "asc",
    });
  });

  it("REQ-502: 同列再 click (asc → desc) で sortDir が toggle される", async () => {
    const onSearchChange = vi.fn();
    renderPage(
      { month: "2026-05", mode: "by_product", sortBy: "amount", sortDir: "asc" },
      onSearchChange,
    );

    const amountHeader = await screen.findByRole("button", { name: /金額/ });
    fireEvent.click(amountHeader);

    expect(onSearchChange).toHaveBeenCalledTimes(1);
    const updater = onSearchChange.mock.calls[0]?.[0] as (
      prev: MonthlySalesSearch,
    ) => MonthlySalesSearch;
    expect(
      updater({ month: "2026-05", mode: "by_product", sortBy: "amount", sortDir: "asc" }),
    ).toEqual({
      month: "2026-05",
      mode: "by_product",
      sortBy: "amount",
      sortDir: "desc",
    });
  });
});

// describe-error-adoption packet（2026-08-04）AC2 / B2 是正: query error 時に
// InvokeError のデバッグ文字列（`[commands: ...]`）が表示に漏れず、describeError 出力が
// 表示されることを assert する（UI-ERR-D2）。
describe("MonthlySalesPage describeError adoption (B2, UI-ERR-D2)", () => {
  it("shows describeError output on query error without leaking the InvokeError debug message", async () => {
    mockGetMonthlySales.mockResolvedValue({
      status: "error",
      error: {
        kind: "internal",
        message: "月次売上の取得に失敗しました",
        field: null,
        error_id: "E-20260501-090000-syn2",
        stocktake_recovery: null,
      },
    });

    renderPage({ month: "2026-05", mode: "by_product" });

    // useMonthlySalesReport の query は production 設計で retry: 1 を明示（QueryClient
    // default を上書き）するため、失敗確定まで retry delay 約 1s を要する
    // （ProductListPage.test.tsx の同型 departmentsQuery 実測に倣い timeout を延長）。
    expect(
      await screen.findByText(
        "月次売上の取得に失敗しました（エラーID: E-20260501-090000-syn2）。詳細は診断ログに記録されています。",
        {},
        { timeout: 4000 },
      ),
    ).toBeInTheDocument();
    expect(screen.queryByText(/\[commands:/)).not.toBeInTheDocument();
  });
});

describe("MonthlySalesPage native input tokens（Lane 5 SC4j）", () => {
  it("SC4j: 月inputがbg-control-surfaceでbg-backgroundを持たない", async () => {
    renderPage({ month: "2026-05", mode: "by_product" });

    const monthInput = await screen.findByLabelText("月を選択");
    expect(monthInput).toHaveClass("bg-control-surface");
    expect(monthInput).not.toHaveClass("bg-background");
  });
});

it("SC22 / REQ-502: empty official department rows retain their plain paragraph", async () => {
  mockGetMonthlySales.mockResolvedValue({
    status: "ok",
    data: { ...buildReport(), official_department_totals: [] },
  });
  renderPage({ month: "2026-05", mode: "by_product" });
  const empty = await screen.findByText("公式部門集計の行はありません。");
  expect(empty.tagName).toBe("P");
  expect(empty.closest('[data-slot="alert"]')).toBeNull();
});
