// src/features/stocktake/StocktakePage.test.tsx
//
// REQ-205 / UI-10 Test Design Matrix T1〜T16（UI-10-D10）+ T17（UI-10-D11）+ T18〜T20（契約監査）+ T21（UI-10-D10 差異符号、受入台本 L3 起源）

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { makeMockProductWithRelations } from "@/features/products/lib/test-fixtures";
import { commands } from "@/lib/bindings";
import type {
  CmdErrorKind,
  CountRecoveryTarget,
  Department,
  StockCountSaveResult,
  Stocktake,
  StocktakeItemDetail,
} from "@/lib/bindings";
import { scrollPageToTop } from "@/lib/page-scroll";
import { d052InvalidationOracle, expectExactInvalidations } from "@/test/invalidation-oracle";

import { StocktakePage, StocktakeProgressHeader } from "./StocktakePage";
import type { StocktakeSearch } from "./types";

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

vi.mock("@/lib/bindings", () => ({
  commands: {
    listDepartments: vi.fn(),
    getActiveStocktake: vi.fn(),
    getLastCompletedStocktake: vi.fn(),
    startStocktake: vi.fn(),
    getStocktakeItems: vi.fn(),
    findStocktakeItem: vi.fn(),
    saveStockCount: vi.fn(),
    previewStockCountPlacement: vi.fn(),
    zeroUncountedItems: vi.fn(),
    listRecheckProducts: vi.fn(),
    completeStocktake: vi.fn(),
    searchProducts: vi.fn(),
  },
}));
vi.mock("@/lib/page-scroll", () => ({ scrollPageToTop: vi.fn() }));

const mockListDepartments = vi.mocked(commands.listDepartments);
const mockSearchProducts = vi.mocked(commands.searchProducts);
const mockGetActive = vi.mocked(commands.getActiveStocktake);
const mockGetLast = vi.mocked(commands.getLastCompletedStocktake);
const mockStart = vi.mocked(commands.startStocktake);
const mockGetItems = vi.mocked(commands.getStocktakeItems);
const mockFindItem = vi.mocked(commands.findStocktakeItem);
const mockSave = vi.mocked(commands.saveStockCount);
const mockPreviewPlacement = vi.mocked(commands.previewStockCountPlacement);
const mockZero = vi.mocked(commands.zeroUncountedItems);
const mockListRecheck = vi.mocked(commands.listRecheckProducts);
const mockComplete = vi.mocked(commands.completeStocktake);
const mockScrollPageToTop = vi.mocked(scrollPageToTop);

function ok<T>(data: T) {
  return { status: "ok" as const, data };
}

function cmdError(kind: CmdErrorKind, message: string) {
  return {
    status: "error" as const,
    error: { kind, message, field: null, error_id: null, stocktake_recovery: null },
  };
}

function stocktakeItem(overrides: Partial<StocktakeItemDetail> = {}): StocktakeItemDetail {
  return { ...baseStocktakeItem(), ...overrides };
}

function activeStocktake(overrides: Partial<Stocktake> = {}): Stocktake {
  return {
    id: 77,
    started_at: "2026-10-01T09:00:00",
    completed_at: null,
    status: "in_progress",
    total_cost: null,
    ...overrides,
  };
}

function baseStocktakeItem(): StocktakeItemDetail {
  return {
    id: 501,
    stocktake_id: 77,
    product_code: "P-001",
    name: "赤い糸",
    department_name: "毛糸",
    system_stock: 10,
    actual_count: null,
    counted_at: null,
    current_stock: 10,
    is_discontinued: false,
    stock_unit: "pcs",
    count_value: null,
    count_kind: null,
    count_business_date: null,
    stock_range_low: null,
    stock_range_high: null,
    recount_reasons: [],
    difference: null,
  };
}

function listResponse(overrides: Partial<Awaited<ReturnType<typeof baseListResponse>>> = {}) {
  return ok({ ...baseListResponse(), ...overrides });
}

function baseListResponse() {
  return {
    items: [baseStocktakeItem(), stocktakeItem({ id: 502, product_code: "P-002", name: "青い糸" })],
    progress: { total_items: 2, counted_items: 1, uncounted_items: 1 },
    total_count: 2,
    page: 1,
    per_page: 200,
  };
}

function renderWithClient(
  ui: ReactNode,
  {
    onSearchChange = vi.fn(),
  }: { onSearchChange?: (updater: (prev: StocktakeSearch) => StocktakeSearch) => void } = {},
) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
  return {
    queryClient,
    invalidateSpy,
    onSearchChange,
    ...render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>),
  };
}

async function renderPage(search: StocktakeSearch = {}) {
  const onSearchChange = vi.fn();
  const utils = renderWithClient(
    <StocktakePage search={search} onSearchChange={onSearchChange} />,
    {
      onSearchChange,
    },
  );
  await screen.findByRole("heading", { name: "棚卸し" });
  return utils;
}

// 計数の日付は PC の今日（73）。Date だけを差し替え、timer は本物のまま
const TODAY = "2026-10-02";

function saveResult(overrides: Partial<StockCountSaveResult> = {}) {
  return ok({
    id: 9001,
    count_kind: "default" as const,
    stock: 8,
    stock_range_low: null,
    stock_range_high: null,
    recount_reasons: [],
    later_count_exists: false,
    ...overrides,
  });
}

beforeEach(() => {
  vi.useFakeTimers({ toFake: ["Date"] });
  vi.setSystemTime(new Date(2026, 9, 2, 10, 0, 0));
  mockListDepartments.mockReset();
  mockGetActive.mockReset();
  mockGetLast.mockReset();
  mockStart.mockReset();
  mockGetItems.mockReset();
  mockFindItem.mockReset();
  mockSave.mockReset();
  mockPreviewPlacement.mockReset();
  mockZero.mockReset();
  mockListRecheck.mockReset();
  mockListRecheck.mockResolvedValue(ok([]));
  mockComplete.mockReset();
  mockSearchProducts.mockReset();
  mockScrollPageToTop.mockReset();
  mockSearchProducts.mockResolvedValue(ok({ items: [], total_count: 0, page: 1, per_page: 10 }));

  const departments: Department[] = [
    {
      id: 1,
      name: "毛糸",
      z005_name: null,
      code_prefix: null,
      next_seq: 1,
      created_at: "2026-01-01T00:00:00",
    },
  ];
  mockListDepartments.mockResolvedValue(ok(departments));
  mockGetActive.mockResolvedValue(ok(null));
  mockGetLast.mockResolvedValue(
    ok({
      stocktake_id: 10,
      completed_at: "2026-09-30T18:00:00",
      total_cost: 2000,
    }),
  );
  mockStart.mockResolvedValue(ok({ stocktake_id: 77, item_count: 2 }));
  mockGetItems.mockResolvedValue(listResponse());
  mockFindItem.mockResolvedValue(ok(baseStocktakeItem()));
  mockSave.mockResolvedValue(saveResult());
  mockZero.mockResolvedValue(ok([]));
  mockComplete.mockResolvedValue(
    ok({
      total_cost: 2500,
      total_items: 2,
      integrity_result: {
        mismatches: [],
        mismatch_count: 0,
        checked_count: 2,
        shared_jan_conflicts: [],
      },
    }),
  );
});

afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
});

/** 42 の SaveStockCountRequest（既定の計数・今日） */
function defaultCount(countValue: number, stocktakeItemId: number | null = 501) {
  return {
    product_code: "P-001",
    count_value: countValue,
    fresh: false,
    business_date: TODAY,
    stocktake_item_id: stocktakeItemId,
  };
}

describe("StocktakePage (UI-10)", () => {
  it("T1 not-started shows start CTA and last summary; active CMD state shows counting display", async () => {
    const first = await renderPage();

    expect(await screen.findByRole("button", { name: "棚卸しを開始する" })).toBeInTheDocument();
    expect(
      await screen.findByText("前回の棚卸し（2026-09-30 18:00:00）: 仕入原価総額 ¥2,000"),
    ).toBeInTheDocument();
    first.unmount();

    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();

    expect(await screen.findByText("棚卸し中（開始日: 2026-10-01 09:00:00）")).toBeInTheDocument();
    expect(await screen.findByText("入力済み 1 / 全 2")).toBeInTheDocument();
  });

  it("T2 start CTA calls startStocktake once and enters counting screen", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValueOnce(ok(null)).mockResolvedValue(ok(activeStocktake()));
    const { invalidateSpy } = await renderPage();

    await user.click(await screen.findByRole("button", { name: "棚卸しを開始する" }));

    await waitFor(() => {
      expect(mockStart).toHaveBeenCalledTimes(1);
    });
    expect(await screen.findByText("棚卸し中（開始日: 2026-10-01 09:00:00）")).toBeInTheDocument();
    expect(mockGetItems).toHaveBeenCalledWith(77, null, null, 1, 50);
    expectExactInvalidations(invalidateSpy.mock.calls, d052InvalidationOracle.stocktakeStart());
  });

  it("T3 department filter and uncounted toggle change getStocktakeItems params", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();
    await screen.findByText("棚卸し中（開始日: 2026-10-01 09:00:00）");

    await user.click(screen.getByRole("combobox", { name: "部門" }));
    await user.click(screen.getByRole("option", { name: "毛糸" }));
    await user.click(screen.getByLabelText("未入力のみ表示"));

    await waitFor(() => {
      expect(mockGetItems).toHaveBeenLastCalledWith(77, 1, false, 1, 50);
    });
  });

  it("SC8 (AC13): changing the department filter resets page to 1 even after totalCount shrinks below the current page (existing reset handler, page-reset oracle)", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    // 開始時は総件数が多く page 3 でも範囲内、フィルタ変更後は総件数が減る想定でも
    // 既存 handler が page を 1 へリセットするため page > totalPages のまま描画されない。
    mockGetItems
      .mockResolvedValueOnce(listResponse({ total_count: 250, page: 3, per_page: 50 }))
      .mockResolvedValue(listResponse({ total_count: 1, page: 1, per_page: 50 }));
    await renderPage({ page: 3 });
    await screen.findByText("棚卸し中（開始日: 2026-10-01 09:00:00）");

    await user.click(screen.getByRole("combobox", { name: "部門" }));
    await user.click(screen.getByRole("option", { name: "毛糸" }));

    await waitFor(() => {
      expect(mockGetItems).toHaveBeenLastCalledWith(77, 1, null, 1, 50);
    });
  });

  it("T4 code/JAN resolve then quantity saves with found item id", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();
    await screen.findByText("棚卸し中（開始日: 2026-10-01 09:00:00）");

    await user.type(screen.getByLabelText("商品を検索・スキャン"), "4900000000001");
    await user.click(screen.getByRole("button", { name: "対象を確認" }));
    await user.clear(await screen.findByLabelText("実際の数"));
    await user.type(screen.getByLabelText("実際の数"), "8");
    await user.click(screen.getByRole("button", { name: "数を保存" }));

    expect(mockFindItem).toHaveBeenCalledWith(77, "4900000000001");
    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledTimes(1);
    });
    expect(mockSave).toHaveBeenCalledWith(defaultCount(8));
  });

  it("DSR-17 T15: StocktakeCountEntry mount focus prevents native scroll", async () => {
    const focus = vi.spyOn(HTMLInputElement.prototype, "focus");
    mockGetActive.mockResolvedValue(ok(activeStocktake()));

    await renderPage();
    const codeInput = await screen.findByLabelText("商品を検索・スキャン");
    const callIndex = focus.mock.contexts.findIndex((context) => context === codeInput);

    expect(callIndex).toBeGreaterThanOrEqual(0);
    expect(focus.mock.calls[callIndex]).toEqual([{ preventScroll: true }]);
  });

  it("T17 focus moves code→quantity→code across scan cycle for continuous HID scanning (UI-10-D11)", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();
    await screen.findByText("棚卸し中（開始日: 2026-10-01 09:00:00）");

    await waitFor(() => {
      expect(screen.getByLabelText("商品を検索・スキャン")).toHaveFocus();
    });

    await user.type(screen.getByLabelText("商品を検索・スキャン"), "4900000000001{Enter}");
    await screen.findByLabelText("実際の数");

    await waitFor(() => {
      expect(screen.getByLabelText("実際の数")).toHaveFocus();
    });

    await user.clear(screen.getByLabelText("実際の数"));
    await user.type(screen.getByLabelText("実際の数"), "8{Enter}");

    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledWith(defaultCount(8));
    });
    await waitFor(() => {
      expect(screen.getByLabelText("商品を検索・スキャン")).toHaveFocus();
    });
  });

  it("T5 target none shows recovery text and does not update count", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockFindItem.mockResolvedValueOnce(ok(null));
    await renderPage();
    await screen.findByText("棚卸し中（開始日: 2026-10-01 09:00:00）");

    await user.type(screen.getByLabelText("商品を検索・スキャン"), "NOPE");
    await user.click(screen.getByRole("button", { name: "対象を確認" }));

    expect(
      await screen.findByText(
        "この商品は棚卸しの対象にありません。商品コードまたはJANを確認してください。新しく登録した商品は自動で追加されます",
      ),
    ).toBeInTheDocument();
    expect(mockSave).not.toHaveBeenCalled();
  });

  it("T6 counted item can be overwritten without confirmation", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    // 明細の実数がある商品は効く計数も持つ（35 の読取り）。欄の文は「入れ直す」と同じ条件で出る
    mockFindItem.mockResolvedValueOnce(
      ok(
        stocktakeItem({
          actual_count: 5,
          count_value: 5,
          count_kind: "default",
          count_business_date: "2026-10-01",
        }),
      ),
    );
    await renderPage();
    await screen.findByText("棚卸し中（開始日: 2026-10-01 09:00:00）");

    await user.type(screen.getByLabelText("商品を検索・スキャン"), "P-001");
    await user.click(screen.getByRole("button", { name: "対象を確認" }));

    expect(
      await screen.findByText(
        "数を直すときは「入れ直す」を押してください。同じ日付で「数を保存」から違う数を入れると、数え直しが必要になります",
      ),
    ).toBeInTheDocument();
    await user.clear(screen.getByLabelText("実際の数"));
    await user.type(screen.getByLabelText("実際の数"), "6");
    await user.click(screen.getByRole("button", { name: "数を保存" }));

    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledWith(defaultCount(6));
    });
  });

  it("T7 progress display matches progress payload", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockGetItems.mockResolvedValueOnce(
      listResponse({ progress: { total_items: 10, counted_items: 7, uncounted_items: 3 } }),
    );
    await renderPage();

    expect(await screen.findByText("入力済み 7 / 全 10")).toBeInTheDocument();
    expect(screen.getByRole("progressbar", { name: "棚卸し進捗" })).toHaveAttribute(
      "aria-valuenow",
      "70",
    );
  });

  it("SF1 (T16) list shows the effective count's business date in the last-count column, blank for uncounted items (UI-10-D10)", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    // 行は BIZ の出す状態（count_business_date と difference は同じ効く計数から。35「読取り」）
    mockGetItems.mockResolvedValueOnce(
      listResponse({
        items: [
          stocktakeItem({
            id: 601,
            product_code: "P-901",
            name: "差異あり商品",
            system_stock: 10,
            current_stock: 12,
            actual_count: 9,
            counted_at: "2026-10-01T09:05:00",
            count_business_date: "2026-09-28",
            // 差異は BIZ の値（35「読取り」）。値の期待（+3）は変えない
            difference: 3,
          }),
          stocktakeItem({
            id: 602,
            product_code: "P-902",
            name: "未入力商品",
            current_stock: 5,
            actual_count: null,
            counted_at: null,
          }),
          stocktakeItem({
            id: 603,
            product_code: "P-903",
            name: "前に数えた未入力商品",
            actual_count: null,
            counted_at: null,
            count_business_date: "2026-09-20",
            difference: -1,
          }),
          stocktakeItem({
            id: 604,
            product_code: "P-904",
            name: "ゼロの商品",
            actual_count: 0,
            counted_at: "2026-09-27T17:00:00",
            count_business_date: "2026-09-27",
            difference: -2,
          }),
          stocktakeItem({
            id: 605,
            product_code: "P-905",
            name: "業務日の無い商品",
            actual_count: 4,
            counted_at: "2026-10-02T10:00:00",
            count_business_date: null,
          }),
        ],
      }),
    );
    await renderPage();

    const table = await screen.findByRole("table");
    const rowOf = (name: string) => {
      const row = within(table).getByText(name).closest("tr");
      if (row === null) throw new Error(`row not found: ${name}`);
      return row;
    };
    const lastCountCell = (row: HTMLElement) => {
      const cells = within(row).getAllByRole("cell");
      return cells[cells.length - 1];
    };

    const row = rowOf("差異あり商品");
    // 表示される在庫値は差異の計算根拠と同じ current_stock（12）であり、
    // system_stock（10）ではないことを検証する（Codex レビュー P2 是正）。
    expect(within(row).getByText("12")).toBeInTheDocument();
    expect(within(row).queryByText("10")).not.toBeInTheDocument();
    expect(within(row).getByText("+3")).toBeInTheDocument();
    expect(lastCountCell(row)).toHaveTextContent(/^2026-09-28$/);
    expect(within(row).queryByText(/2026-10-01/)).not.toBeInTheDocument();

    // 計数の無い未入力の行: 差異と最終カウントが「—」
    const uncountedRow = rowOf("未入力商品");
    expect(within(uncountedRow).getAllByText("—")).toHaveLength(2);
    expect(lastCountCell(uncountedRow)).toHaveTextContent(/^—$/);

    // 前の計数を持つ未入力の行: 最終カウントは「—」（差異は BIZ の値）
    const previouslyCountedRow = rowOf("前に数えた未入力商品");
    expect(lastCountCell(previouslyCountedRow)).toHaveTextContent(/^—$/);
    expect(within(previouslyCountedRow).queryByText(/2026-09-20/)).not.toBeInTheDocument();

    // actual_count 0 は計数
    expect(lastCountCell(rowOf("ゼロの商品"))).toHaveTextContent(/^2026-09-27$/);

    // 業務日が null なら保存の時刻へ戻さない
    const noDateRow = rowOf("業務日の無い商品");
    expect(lastCountCell(noDateRow)).toHaveTextContent(/^—$/);
    expect(within(noDateRow).queryByText(/2026-10-02/)).not.toBeInTheDocument();
  });

  it("T8 negative actual_count shows FieldError and is not sent", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();
    await screen.findByText("棚卸し中（開始日: 2026-10-01 09:00:00）");

    await user.type(screen.getByLabelText("商品を検索・スキャン"), "P-001");
    await user.click(screen.getByRole("button", { name: "対象を確認" }));
    await user.clear(await screen.findByLabelText("実際の数"));
    await user.type(screen.getByLabelText("実際の数"), "-1");
    await user.click(screen.getByRole("button", { name: "数を保存" }));

    expect(await screen.findByText("0以上の数値を入力してください")).toBeInTheDocument();
    expect(mockSave).not.toHaveBeenCalled();
  });

  it("T-C5-1 empty quantity + Enter shows FieldError and does not call saveStockCount", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();
    await screen.findByText("棚卸し中（開始日: 2026-10-01 09:00:00）");

    await user.type(screen.getByLabelText("商品を検索・スキャン"), "P-001{Enter}");
    const quantityInput = await screen.findByLabelText("実際の数");
    await user.clear(quantityInput);
    await user.type(quantityInput, "{Enter}");

    expect(await screen.findByText("数量を入力してください")).toBeInTheDocument();
    expect(mockSave).not.toHaveBeenCalled();
  });

  it("T-C5-2 empty quantity + save click shows FieldError and does not call saveStockCount", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();
    await screen.findByText("棚卸し中（開始日: 2026-10-01 09:00:00）");

    await user.type(screen.getByLabelText("商品を検索・スキャン"), "P-001{Enter}");
    await user.clear(await screen.findByLabelText("実際の数"));
    await user.click(screen.getByRole("button", { name: "数を保存" }));

    expect(await screen.findByText("数量を入力してください")).toBeInTheDocument();
    expect(mockSave).not.toHaveBeenCalled();
  });

  it("T-C5-3 whitespace-only quantity + Enter shows FieldError and does not call saveStockCount", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();
    await screen.findByText("棚卸し中（開始日: 2026-10-01 09:00:00）");

    await user.type(screen.getByLabelText("商品を検索・スキャン"), "P-001{Enter}");
    const quantityInput = await screen.findByLabelText("実際の数");
    await user.clear(quantityInput);
    await user.type(quantityInput, "  {Enter}");

    expect(await screen.findByText("数量を入力してください")).toBeInTheDocument();
    expect(mockSave).not.toHaveBeenCalled();
  });

  it('T-C5-4 explicit "0" + Enter calls saveStockCount with 0 and shows no FieldError', async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();
    await screen.findByText("棚卸し中（開始日: 2026-10-01 09:00:00）");

    await user.type(screen.getByLabelText("商品を検索・スキャン"), "P-001{Enter}");
    const quantityInput = await screen.findByLabelText("実際の数");
    await user.clear(quantityInput);
    await user.type(quantityInput, "0{Enter}");

    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledWith(defaultCount(0));
    });
    expect(screen.queryByText("数量を入力してください")).not.toBeInTheDocument();
  });

  it("T11 result shows total_cost, adjusted_items, and last comparison", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();

    await user.click(await screen.findByRole("button", { name: "棚卸しを確定する" }));
    await user.click(screen.getByRole("button", { name: "確定する" }));

    expect(await screen.findByRole("heading", { name: "棚卸し結果" })).toBeInTheDocument();
    expect(screen.getByText("仕入原価総額")).toBeInTheDocument();
    expect(screen.getByText("¥2,500")).toBeInTheDocument();
    expect(screen.getByText("前回の棚卸し（2026-09-30 18:00:00）")).toBeInTheDocument();
    expect(screen.getByText("仕入原価総額 ¥2,000")).toBeInTheDocument();
  });

  it("T19 result page keeps pre-complete last-stocktake snapshot after lastCompleted invalidation (Codex contract audit P2)", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    // invalidate 前は前回棚卸し、invalidate 後は「今確定した棚卸し自身」が返ってくる状況を再現する
    mockGetLast.mockResolvedValueOnce(
      ok({ stocktake_id: 10, completed_at: "2026-09-30T18:00:00", total_cost: 2000 }),
    );
    mockGetLast.mockResolvedValue(
      ok({ stocktake_id: 77, completed_at: "2026-10-08T10:00:00", total_cost: 2500 }),
    );
    await renderPage();

    await user.click(await screen.findByRole("button", { name: "棚卸しを確定する" }));
    await user.click(screen.getByRole("button", { name: "確定する" }));

    expect(await screen.findByRole("heading", { name: "棚卸し結果" })).toBeInTheDocument();
    // 確定直前にスナップショットした前回棚卸し（2026-09-30）を表示し続け、
    // invalidate 後の再取得値（今確定した棚卸し自身、2026-10-08）には差し替わらない
    expect(screen.getByText("前回の棚卸し（2026-09-30 18:00:00）")).toBeInTheDocument();
    expect(screen.getByText("仕入原価総額 ¥2,000")).toBeInTheDocument();
    await waitFor(() => {
      expect(mockGetLast).toHaveBeenCalledTimes(2);
    });
    expect(screen.queryByText("前回の棚卸し（2026-10-08 10:00:00）")).not.toBeInTheDocument();
  });

  it("T12 result shows integrity fallback when integrity_result is null", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockComplete.mockResolvedValueOnce(
      ok({ total_cost: 2500, total_items: 2, integrity_result: null }),
    );
    await renderPage();

    await user.click(await screen.findByRole("button", { name: "棚卸しを確定する" }));
    await user.click(screen.getByRole("button", { name: "確定する" }));

    expect(await screen.findByText("整合性チェックは実行できませんでした")).toBeInTheDocument();
  });

  it("T13 stocktake error kinds recover with operator-facing messages", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValueOnce(ok(null)).mockResolvedValueOnce(ok(activeStocktake()));
    mockStart.mockResolvedValueOnce(cmdError("stocktake_in_progress", "進行中の棚卸しがあります"));
    await renderPage();

    await user.click(await screen.findByRole("button", { name: "棚卸しを開始する" }));
    expect(await screen.findByText("棚卸し中（開始日: 2026-10-01 09:00:00）")).toBeInTheDocument();
    expect(mockGetActive).toHaveBeenCalledTimes(2);

    mockGetActive.mockResolvedValue(ok(null));
    mockSave.mockResolvedValueOnce(
      cmdError("stocktake_not_in_progress", "この棚卸しは既に完了しています"),
    );
    await user.type(screen.getByLabelText("商品を検索・スキャン"), "P-001");
    await user.click(screen.getByRole("button", { name: "対象を確認" }));
    await user.clear(await screen.findByLabelText("実際の数"));
    await user.type(screen.getByLabelText("実際の数"), "8");
    await user.click(screen.getByRole("button", { name: "数を保存" }));

    expect(await screen.findByText("この棚卸しは既に完了しています")).toBeInTheDocument();
    // stocktake_not_in_progress は状態 query を invalidate/再取得し、not_started 表示へ切り替わる（73 §73.9）
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "棚卸しを開始する" })).toBeInTheDocument();
    });
  });

  it("T18 complete_stocktake stocktake_not_in_progress recovers with operator-facing message (73 §73.9 contract audit)", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockGetItems.mockResolvedValueOnce(
      listResponse({ progress: { total_items: 2, counted_items: 2, uncounted_items: 0 } }),
    );
    mockComplete.mockResolvedValueOnce(
      cmdError("stocktake_not_in_progress", "他端末での並行操作による完了済みメッセージ"),
    );
    await renderPage();

    await user.click(await screen.findByRole("button", { name: "棚卸しを確定する" }));
    mockGetActive.mockResolvedValue(ok(null));
    await user.click(await screen.findByRole("button", { name: "確定する" }));

    expect(await screen.findByText("この棚卸しは既に完了しています")).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "棚卸しを開始する" })).toBeInTheDocument();
    });
  });

  it("T21 product name search fallback resolves item when a single candidate matches (owner L3 finding)", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockFindItem.mockResolvedValueOnce(ok(null)).mockResolvedValueOnce(ok(baseStocktakeItem()));
    mockSearchProducts.mockResolvedValueOnce(
      ok({
        items: [makeMockProductWithRelations({ product_code: "P-001", name: "新商品テスト" })],
        total_count: 1,
        page: 1,
        per_page: 10,
      }),
    );
    await renderPage();
    await screen.findByText("棚卸し中（開始日: 2026-10-01 09:00:00）");

    await user.type(screen.getByLabelText("商品を検索・スキャン"), "新商品テスト");
    await user.click(screen.getByRole("button", { name: "対象を確認" }));

    expect(mockFindItem).toHaveBeenNthCalledWith(1, 77, "新商品テスト");
    expect(mockFindItem).toHaveBeenNthCalledWith(2, 77, "P-001");
    await waitFor(() => {
      expect(screen.getByLabelText("実際の数")).toHaveFocus();
    });
  });

  it("T22 product name search fallback shows candidate list when multiple products match", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockFindItem.mockResolvedValueOnce(ok(null)).mockResolvedValueOnce(ok(baseStocktakeItem()));
    mockSearchProducts.mockResolvedValueOnce(
      ok({
        items: [
          makeMockProductWithRelations({ product_code: "P-001", name: "赤い糸セット" }),
          makeMockProductWithRelations({ product_code: "P-002", name: "青い糸セット" }),
        ],
        total_count: 2,
        page: 1,
        per_page: 10,
      }),
    );
    await renderPage();
    await screen.findByText("棚卸し中（開始日: 2026-10-01 09:00:00）");

    await user.type(screen.getByLabelText("商品を検索・スキャン"), "糸セット");
    await user.click(screen.getByRole("button", { name: "対象を確認" }));

    expect(await screen.findByText("候補から商品を選んでください")).toBeInTheDocument();
    expect(screen.getByText("赤い糸セット")).toBeInTheDocument();
    expect(screen.getByText("青い糸セット")).toBeInTheDocument();

    const targetRow = screen.getByRole("row", { name: /赤い糸セット/ });
    await user.click(within(targetRow).getByRole("button", { name: "選択" }));

    expect(mockFindItem).toHaveBeenNthCalledWith(2, 77, "P-001");
    await waitFor(() => {
      expect(screen.getByLabelText("実際の数")).toHaveFocus();
    });
    expect(screen.queryByText("候補から商品を選んでください")).not.toBeInTheDocument();
  });

  it("T23 IME composition Enter does not trigger search/save (Codex review P2)", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();
    await screen.findByText("棚卸し中（開始日: 2026-10-01 09:00:00）");

    const codeInput = screen.getByLabelText("商品を検索・スキャン");
    fireEvent.change(codeInput, { target: { value: "しんしょうひん" } });
    fireEvent.keyDown(codeInput, { key: "Enter", isComposing: true });

    expect(mockFindItem).not.toHaveBeenCalled();

    fireEvent.change(codeInput, { target: { value: "P-001" } });
    fireEvent.keyDown(codeInput, { key: "Enter" });

    const quantityInput = await screen.findByLabelText("実際の数");
    fireEvent.change(quantityInput, { target: { value: "8" } });
    fireEvent.keyDown(quantityInput, { key: "Enter", isComposing: true });

    expect(mockSave).not.toHaveBeenCalled();
  });

  it("T14 complete pending shows spinner text and disables operations", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockComplete.mockImplementationOnce(() => new Promise(() => undefined));
    await renderPage();

    await user.click(await screen.findByRole("button", { name: "棚卸しを確定する" }));
    await user.click(screen.getByRole("button", { name: "確定する" }));

    expect(await screen.findByText("確定しています")).toBeInTheDocument();
    expect(screen.getByLabelText("商品を検索・スキャン")).toBeDisabled();
    expect(screen.getByRole("button", { name: "棚卸しを確定する" })).toBeDisabled();
  });

  it("T15 update success invalidates list so auto-added item appears without dedicated notification", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    const { invalidateSpy } = await renderPage();
    await screen.findByText("棚卸し中（開始日: 2026-10-01 09:00:00）");

    await user.type(screen.getByLabelText("商品を検索・スキャン"), "P-001");
    await user.click(screen.getByRole("button", { name: "対象を確認" }));
    await user.clear(await screen.findByLabelText("実際の数"));
    await user.type(screen.getByLabelText("実際の数"), "8");
    await user.click(screen.getByRole("button", { name: "数を保存" }));

    await waitFor(() => {
      expectExactInvalidations(
        invalidateSpy.mock.calls,
        d052InvalidationOracle.stocktakeCountUpdate(),
      );
    });
    expect(screen.queryByText("新しい商品を追加しました")).not.toBeInTheDocument();
    expect(within(screen.getByRole("table")).getByText("青い糸")).toBeInTheDocument();
  });

  it("SC7 (UI-10-D13): counted list shows 廃番 badge with border-border only for discontinued items", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockGetItems.mockResolvedValue(
      listResponse({
        items: [
          stocktakeItem({
            id: 601,
            product_code: "D-101",
            name: "廃番の毛糸",
            is_discontinued: true,
          }),
          stocktakeItem({
            id: 602,
            product_code: "N-101",
            name: "通常の毛糸",
            is_discontinued: false,
          }),
        ],
      }),
    );
    await renderPage();

    const discontinuedRow = await screen.findByRole("row", { name: /廃番の毛糸/ });
    const badge = within(discontinuedRow).getByText("廃番");
    expect(badge).toHaveClass("border-border");
    expect(badge.querySelector("svg")).toBeNull();

    const normalRow = screen.getByRole("row", { name: /通常の毛糸/ });
    expect(within(normalRow).queryByText("廃番")).not.toBeInTheDocument();
  });

  it("SC9 (UI-10-D13): candidate row badge is also corrected to border-border (no border-border-strong left in file)", async () => {
    const user = userEvent.setup();
    const discontinued = makeMockProductWithRelations({
      product_code: "D-201",
      name: "廃番候補の糸",
      is_discontinued: true,
    });
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockFindItem.mockResolvedValueOnce(ok(null));
    mockSearchProducts.mockResolvedValueOnce(
      ok({
        items: [discontinued, makeMockProductWithRelations({ product_code: "P-202" })],
        total_count: 2,
        page: 1,
        per_page: 10,
      }),
    );
    await renderPage();
    await user.type(await screen.findByLabelText("商品を検索・スキャン"), "廃番{Enter}");

    const row = await screen.findByRole("row", { name: /廃番候補の糸/ });
    const badge = within(row).getByText("廃番");
    expect(badge).toHaveClass("border-border");
  });

  it("SC1: counting screen has exactly one primary-styled button while a selected item and a candidate table coexist", async () => {
    const user = userEvent.setup();
    const candidates = [
      makeMockProductWithRelations({ product_code: "P-101", name: "候補の糸A" }),
      makeMockProductWithRelations({ product_code: "P-102", name: "候補の糸B" }),
    ];
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockFindItem
      .mockResolvedValueOnce(ok(null))
      .mockResolvedValueOnce(ok(stocktakeItem({ product_code: "P-101", name: "候補の糸A" })))
      .mockResolvedValueOnce(ok(null));
    mockSearchProducts
      .mockResolvedValueOnce(ok({ items: candidates, total_count: 2, page: 1, per_page: 10 }))
      .mockResolvedValueOnce(ok({ items: candidates, total_count: 2, page: 1, per_page: 10 }));
    await renderPage();

    const codeInput = await screen.findByLabelText("商品を検索・スキャン");
    await user.type(codeInput, "候補{Enter}");
    await screen.findByText("候補から商品を選んでください");
    expect(
      screen
        .getAllByRole("button")
        .filter((button) => button.className.includes("bg-primary text-primary-foreground")),
    ).toHaveLength(0);

    await user.click(within(screen.getByRole("row", { name: /候補の糸A/ })).getByRole("button"));
    expect(await screen.findByText("候補の糸A")).toBeInTheDocument();
    await user.clear(codeInput);
    await user.type(codeInput, "別候補{Enter}");
    await screen.findByText("候補から商品を選んでください");

    const primaryButtons = screen
      .getAllByRole("button")
      .filter((button) => button.className.includes("bg-primary text-primary-foreground"));
    expect(primaryButtons).toHaveLength(1);
    expect(primaryButtons[0]).toHaveAccessibleName("数を保存");
  });

  it("SC2: switching target via candidate selection clears previous FieldError", async () => {
    const user = userEvent.setup();
    const nextCandidates = [
      makeMockProductWithRelations({ product_code: "P-201", name: "切替先商品A" }),
      makeMockProductWithRelations({ product_code: "P-202", name: "切替先商品B" }),
    ];
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockFindItem
      .mockResolvedValueOnce(ok(baseStocktakeItem()))
      .mockResolvedValueOnce(ok(stocktakeItem({ product_code: "P-201", name: "切替先商品A" })));
    mockSearchProducts.mockResolvedValueOnce(
      ok({ items: nextCandidates, total_count: 2, page: 1, per_page: 10 }),
    );
    await renderPage();

    const codeInput = await screen.findByLabelText("商品を検索・スキャン");
    await user.type(codeInput, "P-001{Enter}");
    const quantityInput = await screen.findByLabelText("実際の数");
    await user.clear(quantityInput);
    await user.type(quantityInput, "-1{Enter}");
    expect(await screen.findByText("0以上の数値を入力してください")).toBeInTheDocument();

    await user.clear(codeInput);
    await user.type(codeInput, "切替先");
    await user.click(
      within(await screen.findByRole("listbox")).getByRole("option", { name: /切替先商品A/ }),
    );

    expect(await screen.findByText("切替先商品A")).toBeInTheDocument();
    expect(screen.queryByText("0以上の数値を入力してください")).not.toBeInTheDocument();
  });

  it("SC3: FieldError slot renders at fixed height with and without an error", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();
    await user.type(await screen.findByLabelText("商品を検索・スキャン"), "P-001{Enter}");

    const quantityInput = await screen.findByLabelText("実際の数");
    const slot = quantityInput.parentElement?.querySelector(".min-h-5");
    expect(slot).toBeInTheDocument();
    expect(slot).toHaveAttribute("aria-live", "polite");

    await user.clear(quantityInput);
    await user.type(quantityInput, "-1{Enter}");
    expect(await screen.findByRole("alert")).toHaveTextContent("0以上の数値を入力してください");
    expect(quantityInput.parentElement?.querySelector(".min-h-5")).toBe(slot);
  });

  it("SC3b: save button top edge stays aligned with the quantity input's top edge regardless of FieldError", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();
    await user.type(await screen.findByLabelText("商品を検索・スキャン"), "P-001{Enter}");

    const saveButton = await screen.findByRole("button", { name: "数を保存" });
    const buttonColumn = saveButton.closest(".flex");
    // 案(a) 採用: button column に min-h-5 相当の予約 slot（Label 高さ分の spacer を含む）を持たせ、
    // items-end ではなく items-start へ切り替えて入力欄の上端と揃える。
    expect(buttonColumn).toHaveClass("items-start");
    expect(buttonColumn?.querySelector(".min-h-5")).toBeInTheDocument();
    // mutation X3b-iii 是正: spacer 撤去（items-start と min-h-5 slot だけ残す）を kill するため、
    // ボタン直前に invisible + aria-hidden の spacer が実在することも独立に assert する。
    const spacer = buttonColumn?.firstElementChild;
    expect(spacer).toHaveAttribute("aria-hidden", "true");
    expect(spacer).toHaveClass("invisible");
    expect(spacer?.nextElementSibling).toBe(saveButton);

    // FieldError の出入りでこの構造が崩れないことも確認する。
    const quantityInput = await screen.findByLabelText("実際の数");
    await user.clear(quantityInput);
    await user.type(quantityInput, "-1{Enter}");
    expect(await screen.findByRole("alert")).toBeInTheDocument();
    const saveButtonAfterError = await screen.findByRole("button", { name: "数を保存" });
    const buttonColumnAfterError = saveButtonAfterError.closest(".flex");
    expect(buttonColumnAfterError).toHaveClass("items-start");
    expect(buttonColumnAfterError?.querySelector(".min-h-5")).toBeInTheDocument();
    const spacerAfterError = buttonColumnAfterError?.firstElementChild;
    expect(spacerAfterError).toHaveAttribute("aria-hidden", "true");
    expect(spacerAfterError).toHaveClass("invisible");
    expect(spacerAfterError?.nextElementSibling).toBe(saveButtonAfterError);
  });

  it("SC4: target-not-found message renders as status info, not an alert", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockFindItem.mockResolvedValueOnce(ok(null));
    await renderPage();
    await user.type(await screen.findByLabelText("商品を検索・スキャン"), "NOPE{Enter}");

    const status = await screen.findByRole("status");
    expect(status).toHaveTextContent("この商品は棚卸しの対象にありません");
    expect(status).toHaveClass("text-muted-foreground");
    expect(status).not.toHaveClass("text-destructive");
    expect(status.querySelector('svg[aria-hidden="true"]')).toBeInTheDocument();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("SC5: uncounted badge tone switches between warning and success", () => {
    const { rerender } = render(
      <StocktakeProgressHeader
        startedAt="2026-10-01T09:00:00"
        progress={{ total_items: 1, counted_items: 0, uncounted_items: 1 }}
      />,
    );
    const warningBadge = screen.getByText("未入力 1").closest('[data-slot="badge"]');
    expect(warningBadge).toHaveAttribute("data-tone", "warning");
    expect(warningBadge).toHaveClass(
      "border-warning-border",
      "bg-warning-soft",
      "text-warning-strong",
    );
    expect(warningBadge?.querySelector('svg[aria-hidden="true"]')).toBeInTheDocument();

    rerender(
      <StocktakeProgressHeader
        startedAt="2026-10-01T09:00:00"
        progress={{ total_items: 1, counted_items: 1, uncounted_items: 0 }}
      />,
    );
    const successBadge = screen.getByText("未入力 0").closest('[data-slot="badge"]');
    // SC7 / DSR-22: 完了をsoft toneとiconで伝え、旧直塗りは残さない。
    expect(successBadge).toHaveAttribute("data-variant", "outline");
    expect(successBadge).toHaveAttribute("data-tone", "success");
    expect(successBadge).toHaveClass(
      "border-success-border",
      "bg-success-soft",
      "text-success-strong",
    );
    expect(successBadge).not.toHaveClass("bg-success");
    expect(successBadge).not.toHaveClass("text-primary-foreground");
    expect(successBadge?.querySelector('svg[aria-hidden="true"]')).toBeInTheDocument();
  });

  it("SC6: product-name search fallback query includes discontinued products and marks them", async () => {
    const user = userEvent.setup();
    const discontinued = makeMockProductWithRelations({
      product_code: "D-001",
      name: "廃番の糸",
      is_discontinued: true,
    });
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockFindItem.mockResolvedValueOnce(ok(null));
    mockSearchProducts.mockResolvedValueOnce(
      ok({
        items: [discontinued, makeMockProductWithRelations({ product_code: "P-002" })],
        total_count: 2,
        page: 1,
        per_page: 10,
      }),
    );
    await renderPage();
    await user.type(await screen.findByLabelText("商品を検索・スキャン"), "廃番{Enter}");

    expect(mockSearchProducts).toHaveBeenCalledWith({
      department_id: null,
      is_discontinued: null,
      sort_key: "ProductCode",
      sort_order: "Asc",
      page: 1,
      per_page: 10,
      keyword: "廃番",
    });
    const row = await screen.findByRole("row", { name: /廃番の糸/ });
    expect(within(row).getByText("廃番")).toBeInTheDocument();
  });

  it("SC8a: default per_page in the initial items query is 50", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();
    await waitFor(() => {
      expect(mockGetItems).toHaveBeenCalledWith(77, null, null, 1, 50);
    });
  });

  it("SC8b: changing per_page resets page to 1", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage({ page: 3 });
    await user.click(await screen.findByRole("combobox", { name: "表示件数" }));
    await user.click(screen.getByRole("option", { name: "100 件" }));

    await waitFor(() => {
      expect(mockGetItems).toHaveBeenLastCalledWith(77, null, null, 1, 100);
    });
  });

  it("SC8c': list pagination uses the canonical ProductPagination component", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    // AC7（round 1/3 是正）: Pagination は totalPages<=1 で null を返す（S2）ため、
    // ボタンの存在を確認するには total_count を per_page 超へ上げて totalPages>1 にする。
    mockGetItems.mockResolvedValue(listResponse({ total_count: 250 }));
    await renderPage();

    expect(await screen.findByRole("button", { name: "前のページ" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "次のページ" })).toBeInTheDocument();
    await user.click(screen.getByRole("combobox", { name: "表示件数" }));
    expect(screen.getByRole("option", { name: "50 件" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "100 件" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "200 件" })).toBeInTheDocument();
  });

  it("SC10 (S4a rewrite): filter row lists department filter, then uncounted-only checkbox, then per-page select in that DOM order", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();

    const departmentTrigger = await screen.findByRole("combobox", { name: "部門" });
    const perPageTrigger = screen.getByRole("combobox", { name: "表示件数" });
    const uncountedCheckbox = screen.getByRole("checkbox", { name: "未入力のみ表示" });
    // SPEC-FILTER-LABEL-RT-1 D-RT7 GA3: Checkbox は入力と同じ高さの箱の中で縦中央に置く。
    expect(screen.getByLabelText("未入力のみ表示")).toBe(uncountedCheckbox);
    const checkboxLabel = uncountedCheckbox.closest("label");
    expect(checkboxLabel).toHaveClass("h-9", "items-center", "self-end");
    expect(checkboxLabel).toHaveAttribute("for", uncountedCheckbox.id);

    // S4a（round 1 是正）: 部門 → 未入力のみ表示 → 表示件数 の順で DOM 上に並ぶことを
    // compareDocumentPosition で assert する（表示件数 Select を枠内の最後尾へ移動）。
    expect(
      departmentTrigger.compareDocumentPosition(uncountedCheckbox) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(
      uncountedCheckbox.compareDocumentPosition(perPageTrigger) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    // Codex review 5129977808 P2 是正: 文書順だけでは表示件数ブロックが filter-row の
    // 外へ出ても検出できない。filter-row root への containment + lastElementChild を
    // 追加で assert する。
    const filterRow = perPageTrigger.closest(".flex.flex-wrap.items-end.gap-3");
    expect(filterRow).not.toBeNull();
    expect(filterRow).toContainElement(perPageTrigger);
    expect(filterRow?.lastElementChild).toContainElement(perPageTrigger);
  });

  it("SC4a: filter row root has rounded-lg border bg-card p-4 (old borderless frame removed)", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockGetItems.mockResolvedValue(listResponse());
    const { container } = await renderPage();
    await screen.findByRole("combobox", { name: "部門" });
    expect(container.querySelector(".rounded-lg.border.bg-card.p-4")).not.toBeNull();
    // 旧 frame（枠なし、`flex flex-wrap items-center gap-4` のみ）が残っていないこと。
    const oldFrame = Array.from(container.querySelectorAll("div")).find(
      (el) => el.className === "flex flex-wrap items-center gap-4",
    );
    expect(oldFrame).toBeUndefined();
  });

  it("SC3a: renders the top PaginationSummary above the results table when totalCount > 0", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockGetItems.mockResolvedValue(listResponse({ total_count: 2, per_page: 200 }));
    await renderPage();
    expect(
      await screen.findByText("全 2 件のうち 1〜2 件を表示（1 / 1 ページ）"),
    ).toBeInTheDocument();
  });

  it("SC6: the fieldset wrapping the bottom Pagination is absent when totalPages<=1", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockGetItems.mockResolvedValue(listResponse({ total_count: 2, per_page: 200 }));
    const { container } = await renderPage();
    await screen.findByText("全 2 件のうち 1〜2 件を表示（1 / 1 ページ）");
    // 「カウント入力」の fieldset（disabled 制御、常設）以外に一覧側の fieldset が
    // 残っていないこと（単一ページで空 <fieldset> を残さない、S5）。
    expect(container.querySelectorAll("fieldset")).toHaveLength(1);
    expect(screen.queryByRole("button", { name: "前のページ" })).not.toBeInTheDocument();
  });

  it("SC11 (Codex review 5129977808 P1 是正): saving the last uncounted item on the last page keeps the previous-page recovery control", async () => {
    const user = userEvent.setup();
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    // 未入力のみ表示・51 件・50 件表示の 2 ページ目（残り 1 件）を初期表示。
    mockGetItems.mockResolvedValueOnce(
      listResponse({
        items: [stocktakeItem({ id: 999, product_code: "P-051", name: "最後の糸" })],
        progress: { total_items: 51, counted_items: 0, uncounted_items: 51 },
        total_count: 51,
        page: 2,
        per_page: 50,
      }),
    );
    // 保存 → invalidate 後の再取得: 未入力が 50 件に減り、2 ページ目は空になる
    // （totalPages は 1 に縮むが page は 2 のまま残る）。
    mockGetItems.mockResolvedValueOnce(
      listResponse({
        items: [],
        progress: { total_items: 51, counted_items: 1, uncounted_items: 50 },
        total_count: 50,
        page: 2,
        per_page: 50,
      }),
    );
    mockFindItem.mockResolvedValueOnce(
      ok(stocktakeItem({ id: 999, product_code: "P-051", name: "最後の糸" })),
    );
    await renderPage({ page: 2, counted_only: false });
    await screen.findByText("棚卸し中（開始日: 2026-10-01 09:00:00）");

    await user.type(screen.getByLabelText("商品を検索・スキャン"), "P-051");
    await user.click(screen.getByRole("button", { name: "対象を確認" }));
    await user.clear(await screen.findByLabelText("実際の数"));
    await user.type(screen.getByLabelText("実際の数"), "10");
    await user.click(screen.getByRole("button", { name: "数を保存" }));

    await waitFor(() => {
      expect(mockGetItems).toHaveBeenLastCalledWith(77, null, false, 2, 50);
    });
    expect(await screen.findByRole("button", { name: "前のページ" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "前のページ" })).toBeEnabled();
  });

  it("SC4a: filter row root has rounded-lg border bg-card p-4 (old borderless frame removed)", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockGetItems.mockResolvedValue(listResponse());
    const { container } = await renderPage();
    await screen.findByRole("combobox", { name: "部門" });
    expect(container.querySelector(".rounded-lg.border.bg-card.p-4")).not.toBeNull();
    // 旧 frame（枠なし、`flex flex-wrap items-center gap-4` のみ）が残っていないこと。
    const oldFrame = Array.from(container.querySelectorAll("div")).find(
      (el) => el.className === "flex flex-wrap items-center gap-4",
    );
    expect(oldFrame).toBeUndefined();
  });

  it("SC3a: renders the top PaginationSummary above the results table when totalCount > 0", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockGetItems.mockResolvedValue(listResponse({ total_count: 2, per_page: 200 }));
    await renderPage();
    expect(
      await screen.findByText("全 2 件のうち 1〜2 件を表示（1 / 1 ページ）"),
    ).toBeInTheDocument();
  });

  it("SC6: the fieldset wrapping the bottom Pagination is absent when totalPages<=1", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockGetItems.mockResolvedValue(listResponse({ total_count: 2, per_page: 200 }));
    const { container } = await renderPage();
    await screen.findByText("全 2 件のうち 1〜2 件を表示（1 / 1 ページ）");
    // 「カウント入力」の fieldset（disabled 制御、常設）以外に一覧側の fieldset が
    // 残っていないこと（単一ページで空 <fieldset> を残さない、S5）。
    expect(container.querySelectorAll("fieldset")).toHaveLength(1);
    expect(screen.queryByRole("button", { name: "前のページ" })).not.toBeInTheDocument();
  });
});

describe("StocktakePage SPEC-UIBB-1/2（filter-empty reset action、73 §73.6）", () => {
  function emptyListResponse() {
    return listResponse({
      items: [],
      progress: { total_items: 0, counted_items: 0, uncounted_items: 0 },
      total_count: 0,
    });
  }

  it("SPEC-UIBB-1 絞り込み該当なしで解除ボタンを表示する", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockGetItems.mockResolvedValue(emptyListResponse());
    await renderPage({ dept: 1 });
    expect(await screen.findByRole("button", { name: "絞り込みを解除" })).toBeInTheDocument();
  });

  it("SPEC-UIBB-1 既定条件の0件では解除ボタンを出さない", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockGetItems.mockResolvedValue(emptyListResponse());
    await renderPage({});
    expect(await screen.findByText("この条件に一致する商品がありません")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "絞り込みを解除" })).not.toBeInTheDocument();
  });

  it("SPEC-UIBB-2 解除で部門フィルタ・未入力のみ・pageが既定値に戻る", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockGetItems.mockResolvedValue(emptyListResponse());
    const { onSearchChange } = await renderPage({ dept: 1, counted_only: false, page: 3 });

    const resetButton = await screen.findByRole("button", { name: "絞り込みを解除" });
    await userEvent.setup().click(resetButton);

    const mockedOnSearchChange = vi.mocked(onSearchChange);
    const updater = mockedOnSearchChange.mock.calls[
      mockedOnSearchChange.mock.calls.length - 1
    ]?.[0] as (prev: StocktakeSearch) => StocktakeSearch;
    const result = updater({ dept: 1, counted_only: false, page: 3 });
    expect(result.dept).toBeUndefined();
    expect(result.counted_only).toBeUndefined();
    expect(result.page).toBeUndefined();
  });
});

describe("StocktakePage perPage scroll（UI-10）", () => {
  it("SC9a: 表示件数変更で画面を先頭へ戻す", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage({ page: 3 });

    const user = userEvent.setup();
    await user.click(await screen.findByRole("combobox", { name: "表示件数" }));
    await user.click(screen.getByRole("option", { name: "100 件" }));

    expect(mockScrollPageToTop).toHaveBeenCalledTimes(1);
  });
});

it.each([
  [2, "+2", "text-success-strong"],
  [-3, "-3", "text-destructive-strong"],
  [0, "0", "text-muted-foreground"],
  [null, "—", null],
] as const)(
  "SC9 / REQ-205: difference %s retains sign and tone; uncounted stays neutral",
  async (difference, label, color) => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockGetItems.mockResolvedValue(
      listResponse({
        items: [
          stocktakeItem({
            name: "SC9商品",
            current_stock: 10,
            actual_count: difference === null ? null : 10 - difference,
            counted_at: difference === null ? null : "2026-10-01T09:05:00",
            difference,
          }),
        ],
        progress: {
          total_items: 1,
          counted_items: difference === null ? 0 : 1,
          uncounted_items: difference === null ? 1 : 0,
        },
        total_count: 1,
      }),
    );
    if (difference === null) {
      // SC9 / S3: 未入力は確定結果へ進めず、「—」と無着色を維持する。
      await renderPage();
      const row = (await screen.findByText("SC9商品")).closest("tr");
      if (!row) throw new Error("uncounted row missing");
      const cell = within(row).getAllByRole("cell")[5];
      expect(cell).toHaveTextContent(/^—$/);
      expect(cell).toHaveClass("text-right");
      expect(cell).not.toHaveClass("text-muted-foreground");
      expect(cell).not.toHaveClass("text-success-strong");
      expect(cell).not.toHaveClass("text-destructive-strong");
      return;
    }
    await renderPage();
    const activeRow = (await screen.findByText("SC9商品")).closest("tr");
    if (!activeRow) throw new Error("active row missing");
    expect(within(activeRow).getByText(label).closest("td")).toHaveClass(color);
  },
);

it.each([0, 1])(
  "SPEC-FILTER-LABEL-RT-1 D-RT6: 進捗は見出し行・説明行・barの順（未入力=%s）",
  (uncounted) => {
    render(
      <StocktakeProgressHeader
        startedAt="2026-10-01T09:00:00"
        progress={{ total_items: 2, counted_items: 2 - uncounted, uncounted_items: uncounted }}
      />,
    );
    const heading = screen.getByRole("heading", { level: 2 });
    const row = heading.parentElement;
    const description = screen.getByText(`入力済み ${String(2 - uncounted)} / 全 2`);
    expect(heading).toHaveClass("min-w-0", "flex-1");
    expect(row).toHaveClass("flex", "flex-wrap", "items-start", "justify-between", "gap-3");
    expect(description.tagName).toBe("P");
    expect(row?.nextElementSibling).toBe(description);
    expect(row?.parentElement).toHaveClass("space-y-1");
    expect(screen.getByText(`未入力 ${String(uncounted)}`)).toHaveClass("shrink-0");
    expect(
      screen
        .getByText(`未入力 ${String(uncounted)}`)
        .querySelector(uncounted ? "svg.lucide-triangle-alert" : "svg.lucide-circle-check"),
    ).toBeInTheDocument();
    const progress = screen.getByRole("progressbar", { name: "棚卸し進捗" });
    expect(progress.parentElement).toHaveClass("space-y-2");
    expect(progress.previousElementSibling).toBe(row?.parentElement);
  },
);

// ---- D-118 b1 の計数・要再確認・完了（73 の D-118 の節。Matrix の U1〜U9・U15・U16） ----

function recheckTarget(overrides: Partial<CountRecoveryTarget> = {}): CountRecoveryTarget {
  return {
    product_code: "P-003",
    product_name: "黄色い糸",
    stock_unit: "pcs",
    stock: 3,
    stock_range_low: 3,
    stock_range_high: 6,
    stocktake_item_id: 503,
    stocktake_id: 77,
    action: "active_count",
    block_reason: null,
    recount_reasons: ["undetermined_settlement"],
    ...overrides,
  };
}

function placement(overrides: Partial<Awaited<ReturnType<typeof placementData>>> = {}) {
  return ok({ ...placementData(), ...overrides });
}

function placementData() {
  return {
    later_count_exists: false,
    product_name: "黄色い糸",
    stock_unit: "pcs" as const,
    stock: 3,
    stock_range_low: 3 as number | null,
    stock_range_high: 6 as number | null,
    recount_reasons: ["undetermined_settlement"] as CountRecoveryTarget["recount_reasons"],
    count_value: 4 as number | null,
    count_kind: "default" as "default" | "fresh" | null,
    count_business_date: "2026-10-01" as string | null,
  };
}

async function selectItemByCode(code = "P-001") {
  const user = userEvent.setup();
  await user.type(await screen.findByLabelText("商品を検索・スキャン"), `${code}{Enter}`);
  await screen.findByLabelText(/^実際の数/);
  return user;
}

function renderWithReturn(search: StocktakeSearch, onReturnTo = vi.fn()) {
  const onSearchChange = vi.fn();
  const utils = renderWithClient(
    <StocktakePage search={search} onSearchChange={onSearchChange} onReturnTo={onReturnTo} />,
    { onSearchChange },
  );
  return { ...utils, onReturnTo };
}

describe("StocktakePage D-118 b1（73）", () => {
  it("U1 default count saves with today's date and returns focus", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();
    const user = await selectItemByCode();

    expect(screen.getByText(`保存の日付: ${TODAY}（今日）`)).toBeInTheDocument();
    expect(screen.getByLabelText(/^実際の数/)).toHaveValue("");
    await user.type(screen.getByLabelText(/^実際の数/), "8{Enter}");

    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledWith(defaultCount(8));
    });
    expect(await screen.findByText(`赤い糸 を 8 個（${TODAY}）で保存しました`)).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getByLabelText("商品を検索・スキャン")).toHaveFocus();
    });
  });

  it("U1 save result with recheck reasons tells the operator in text", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockSave.mockResolvedValueOnce(
      saveResult({
        stock_range_low: 7,
        stock_range_high: 8,
        recount_reasons: ["same_date_movement"],
      }),
    );
    await renderPage();
    const user = await selectItemByCode();
    await user.type(screen.getByLabelText(/^実際の数/), "8{Enter}");
    expect(
      await screen.findByText(
        `赤い糸 を 8 個（${TODAY}）で保存しました。要再確認: 数えた日に売り買い（入出庫）がありました`,
      ),
    ).toBeInTheDocument();
  });

  it("U1 pending count save locks switching the product until the response (73, Final Review F1)", async () => {
    let resolveSave: (value: ReturnType<typeof saveResult>) => void = () => undefined;
    mockSave.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolveSave = resolve;
        }),
    );
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();
    const user = await selectItemByCode();

    await user.type(screen.getByLabelText(/^実際の数/), "8{Enter}");
    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledWith(defaultCount(8));
    });
    expect(screen.getByLabelText("商品を検索・スキャン")).toBeDisabled();
    expect(screen.getByRole("button", { name: "対象を確認" })).toBeDisabled();

    resolveSave(saveResult());
    expect(await screen.findByText(`赤い糸 を 8 個（${TODAY}）で保存しました`)).toBeInTheDocument();
    expect(screen.getByLabelText("商品を検索・スキャン")).toBeEnabled();
  });

  it("U1 late save response for product A keeps product B's field (73, Final Review F1)", async () => {
    let resolveSave: (value: ReturnType<typeof saveResult>) => void = () => undefined;
    let resolveFind: (value: ReturnType<typeof ok<StocktakeItemDetail>>) => void = () => undefined;
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();
    const user = await selectItemByCode();
    await user.type(screen.getByLabelText(/^実際の数/), "8");

    // B の検索の応答を保留したまま A の数を保存し、A の保存の応答も保留する
    mockFindItem.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolveFind = resolve;
        }),
    );
    mockSave.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolveSave = resolve;
        }),
    );
    await user.clear(screen.getByLabelText("商品を検索・スキャン"));
    await user.type(screen.getByLabelText("商品を検索・スキャン"), "P-003{Enter}");
    await user.click(screen.getByRole("button", { name: "数を保存" }));
    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledWith(defaultCount(8));
    });
    resolveFind(ok(stocktakeItem({ id: 503, product_code: "P-003", name: "緑の糸" })));
    await screen.findByText("緑の糸");

    resolveSave(saveResult());
    expect(await screen.findByText(`赤い糸 を 8 個（${TODAY}）で保存しました`)).toBeInTheDocument();
    // A の応答は B の欄を消さない
    await user.type(await screen.findByLabelText(/^実際の数/), "7{Enter}");
    await waitFor(() => {
      expect(mockSave).toHaveBeenLastCalledWith({
        ...defaultCount(7, 503),
        product_code: "P-003",
      });
    });
  });

  it("U9 late count_now save for product A keeps product B's field after the route changes (73, Final Review G1)", async () => {
    let resolveSave: (value: ReturnType<typeof saveResult>) => void = () => undefined;
    mockGetActive.mockResolvedValue(ok(null));
    mockPreviewPlacement.mockImplementation((productCode) =>
      Promise.resolve(
        placement({
          product_name: productCode === "P-005" ? "連動なしの糸" : "緑の糸",
          count_value: null,
          count_kind: null,
          count_business_date: null,
        }),
      ),
    );
    mockSave.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolveSave = resolve;
        }),
    );
    const { onReturnTo, onSearchChange, queryClient, rerender } = renderWithReturn({
      product: "P-005",
      returnTo: "/products/P-005/edit",
    });
    const user = userEvent.setup();
    await screen.findByText("連動なしの糸");
    await user.type(screen.getByLabelText(/^実際の数/), "9");
    await user.click(screen.getByRole("button", { name: "今の数を保存" }));
    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledTimes(1);
    });

    // A の保存の応答の保留中に route の product が B に変わる
    rerender(
      <QueryClientProvider client={queryClient}>
        <StocktakePage
          search={{ product: "P-006" }}
          onSearchChange={onSearchChange}
          onReturnTo={onReturnTo}
        />
      </QueryClientProvider>,
    );
    await screen.findByText("緑の糸");
    resolveSave(saveResult());

    // A の遅い成功は B の欄を閉じず、A の編集画面へも戻らない
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "今の数を保存" })).toBeEnabled();
    });
    expect(screen.getByText("緑の糸")).toBeInTheDocument();
    expect(onReturnTo).not.toHaveBeenCalled();
    await user.type(screen.getByLabelText(/^実際の数/), "7");
    await user.click(screen.getByRole("button", { name: "今の数を保存" }));
    await waitFor(() => {
      expect(mockSave).toHaveBeenLastCalledWith({
        product_code: "P-006",
        count_value: 7,
        fresh: true,
        business_date: TODAY,
        stocktake_item_id: null,
      });
    });
  });

  it("U9 count_now save for product A closes A's field after the route's product B fails to load (73, Final Review N1)", async () => {
    let resolveSave: (value: ReturnType<typeof saveResult>) => void = () => undefined;
    mockGetActive.mockResolvedValue(ok(null));
    mockPreviewPlacement.mockImplementation((productCode) =>
      Promise.resolve(
        productCode === "P-005"
          ? placement({
              product_name: "連動なしの糸",
              count_value: null,
              count_kind: null,
              count_business_date: null,
            })
          : cmdError("not_found", "商品が見つかりません"),
      ),
    );
    mockSave.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolveSave = resolve;
        }),
    );
    const { onReturnTo, onSearchChange, queryClient, rerender } = renderWithReturn({
      product: "P-005",
      returnTo: "/products/P-005/edit",
    });
    const user = userEvent.setup();
    await screen.findByText("連動なしの糸");
    await user.type(screen.getByLabelText(/^実際の数/), "9");
    await user.click(screen.getByRole("button", { name: "今の数を保存" }));
    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledTimes(1);
    });

    // A の保存の応答の保留中に route の product が B に変わり、B の読込みが失敗する
    rerender(
      <QueryClientProvider client={queryClient}>
        <StocktakePage
          search={{ product: "P-006" }}
          onSearchChange={onSearchChange}
          onReturnTo={onReturnTo}
        />
      </QueryClientProvider>,
    );
    expect(await screen.findByText("商品が見つかりません")).toBeInTheDocument();
    expect(screen.getByText("連動なしの糸")).toBeInTheDocument();
    resolveSave(saveResult());

    // 欄は A のままなので、A の保存の成功で欄が閉じ、A の編集画面へ戻る
    await waitFor(() => {
      expect(onReturnTo).toHaveBeenCalledWith("/products/P-005/edit");
    });
    expect(screen.queryByText("連動なしの糸")).not.toBeInTheDocument();
  });

  it("U9 count_now save for product A closes A's field after a recount of C supersedes route B and fails to load (73, Final Review N1c)", async () => {
    mockGetActive.mockResolvedValue(ok(null));
    mockListRecheck.mockResolvedValue(
      ok([
        recheckTarget({
          product_code: "P-007",
          product_name: "生成りの紐",
          stocktake_item_id: null,
          action: "count_now",
        }),
      ]),
    );
    mockPreviewPlacement.mockImplementation((productCode) =>
      productCode === "P-005"
        ? Promise.resolve(
            placement({
              product_name: "連動なしの糸",
              count_value: null,
              count_kind: null,
              count_business_date: null,
            }),
          )
        : productCode === "P-006"
          ? new Promise(() => undefined)
          : Promise.resolve(cmdError("not_found", "商品が見つかりません")),
    );
    const { onSearchChange, queryClient, rerender } = renderWithReturn({
      product: "P-005",
      returnTo: "/products/P-005/edit",
    });
    const user = userEvent.setup();
    await screen.findByText("連動なしの糸");

    // route の product が B に変わり、B の読込みの決着の前に「入れ直す」C を押す。C の読込みは失敗する
    rerender(
      <QueryClientProvider client={queryClient}>
        <StocktakePage search={{ product: "P-006" }} onSearchChange={onSearchChange} />
      </QueryClientProvider>,
    );
    await waitFor(() => {
      expect(mockPreviewPlacement).toHaveBeenCalledWith("P-006", expect.anything());
    });
    const list = await screen.findByRole("region", { name: "数え直しが必要な商品" });
    await user.click(
      within(within(list).getByRole("row", { name: /生成りの紐/ })).getByRole("button", {
        name: "入れ直す",
      }),
    );
    expect(await screen.findByText("商品が見つかりません")).toBeInTheDocument();
    expect(screen.getByText("連動なしの糸")).toBeInTheDocument();

    // 欄は A のままなので、A の保存の成功で欄が閉じる
    await user.type(screen.getByLabelText(/^実際の数/), "9");
    await user.click(screen.getByRole("button", { name: "今の数を保存" }));
    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(screen.queryByText("連動なしの糸")).not.toBeInTheDocument();
    });
  });

  // A の保存の応答の保留中に route を B へ変え、B の読込みを保留したまま A の保存を成功させる
  async function saveAThenSettleB(settleB: "fail" | "succeed") {
    let resolveSave: (value: ReturnType<typeof saveResult>) => void = () => undefined;
    let settlePreviewB: () => void = () => undefined;
    mockGetActive.mockResolvedValue(ok(null));
    mockPreviewPlacement.mockImplementation((productCode) =>
      productCode === "P-005"
        ? Promise.resolve(
            placement({
              product_name: "連動なしの糸",
              count_value: null,
              count_kind: null,
              count_business_date: null,
            }),
          )
        : new Promise((resolve) => {
            settlePreviewB = () => {
              resolve(
                settleB === "fail"
                  ? cmdError("not_found", "商品が見つかりません")
                  : placement({
                      product_name: "緑の糸",
                      count_value: null,
                      count_kind: null,
                      count_business_date: null,
                    }),
              );
            };
          }),
    );
    mockSave.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolveSave = resolve;
        }),
    );
    const { onReturnTo, onSearchChange, queryClient, rerender } = renderWithReturn({
      product: "P-005",
      returnTo: "/products/P-005/edit",
    });
    const user = userEvent.setup();
    await screen.findByText("連動なしの糸");
    await user.type(screen.getByLabelText(/^実際の数/), "9");
    await user.click(screen.getByRole("button", { name: "今の数を保存" }));
    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledTimes(1);
    });
    rerender(
      <QueryClientProvider client={queryClient}>
        <StocktakePage
          search={{ product: "P-006" }}
          onSearchChange={onSearchChange}
          onReturnTo={onReturnTo}
        />
      </QueryClientProvider>,
    );
    await waitFor(() => {
      expect(mockPreviewPlacement).toHaveBeenCalledWith("P-006", expect.anything());
    });
    resolveSave(saveResult());
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "今の数を保存" })).toBeEnabled();
    });
    await waitFor(() => {
      expect(queryClient.isFetching()).toBe(0);
    });
    // B の読込みの決着の前は欄が A のまま残る
    expect(screen.getByText("連動なしの糸")).toBeInTheDocument();
    expect(onReturnTo).not.toHaveBeenCalled();
    settlePreviewB();
    return { onReturnTo, user };
  }

  it("U9 count_now save for product A closes A's field when it succeeds before the route's product B fails to load (73, Final Review N1b)", async () => {
    const { onReturnTo } = await saveAThenSettleB("fail");

    // B の読込みが失敗して欄が A のまま決まった時点で、A の保存の成功の後始末をする
    expect(await screen.findByText("商品が見つかりません")).toBeInTheDocument();
    await waitFor(() => {
      expect(onReturnTo).toHaveBeenCalledWith("/products/P-005/edit");
    });
    expect(screen.queryByText("連動なしの糸")).not.toBeInTheDocument();
  });

  it("U9 count_now save for product A keeps product B's field when it succeeds before B loads (73, Final Review N1b)", async () => {
    const { onReturnTo, user } = await saveAThenSettleB("succeed");

    // B の欄が開いたなら、A の先に届いた成功は B の欄を閉じない
    expect(await screen.findByText("緑の糸")).toBeInTheDocument();
    expect(screen.queryByText("連動なしの糸")).not.toBeInTheDocument();
    expect(onReturnTo).not.toHaveBeenCalled();
    await user.type(screen.getByLabelText(/^実際の数/), "7");
    await user.click(screen.getByRole("button", { name: "今の数を保存" }));
    await waitFor(() => {
      expect(mockSave).toHaveBeenLastCalledWith({
        product_code: "P-006",
        count_value: 7,
        fresh: true,
        business_date: TODAY,
        stocktake_item_id: null,
      });
    });
    expect(onReturnTo).not.toHaveBeenCalled();
  });

  // route の product が外れる経路（73 の D-118 の節「今の数を入れる」）。P-005 = A、P-006 = B
  function renderCountNowRoute(previewB: Promise<ReturnType<typeof placement>>) {
    mockGetActive.mockResolvedValue(ok(null));
    mockPreviewPlacement.mockImplementation((productCode) =>
      productCode === "P-005"
        ? Promise.resolve(
            placement({
              product_name: "連動なしの糸",
              count_value: null,
              count_kind: null,
              count_business_date: null,
            }),
          )
        : previewB,
    );
    const utils = renderWithReturn({ product: "P-005", returnTo: "/products/P-005/edit" });
    const setRoute = (search: StocktakeSearch) => {
      utils.rerender(
        <QueryClientProvider client={utils.queryClient}>
          <StocktakePage
            search={search}
            onSearchChange={utils.onSearchChange}
            onReturnTo={utils.onReturnTo}
          />
        </QueryClientProvider>,
      );
    };
    return { ...utils, setRoute };
  }

  function placementB() {
    return placement({
      product_name: "緑の糸",
      count_value: null,
      count_kind: null,
      count_business_date: null,
    });
  }

  function holdSave() {
    const held: { resolve: (value: ReturnType<typeof saveResult>) => void } = {
      resolve: () => undefined,
    };
    mockSave.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          held.resolve = resolve;
        }),
    );
    return held;
  }

  // 応答を解いた後の microtask と invalidate の await を流し切る
  async function flushSaveFollowUp(invalidateSpy: { mock: { calls: unknown[] } }) {
    await waitFor(() => {
      expect(invalidateSpy.mock.calls.length).toBeGreaterThan(0);
    });
    await act(async () => {
      for (let i = 0; i < 5; i += 1) await new Promise((resolve) => setTimeout(resolve, 0));
    });
  }

  it("SF4 count_now field for A closes on save after the route's product is removed while B loads (73)", async () => {
    let resolvePreviewB: () => void = () => undefined;
    const { onReturnTo, setRoute } = renderCountNowRoute(
      new Promise((resolve) => {
        resolvePreviewB = () => {
          resolve(placementB());
        };
      }),
    );
    const user = userEvent.setup();
    await screen.findByText("連動なしの糸");

    setRoute({ product: "P-006" });
    await waitFor(() => {
      expect(mockPreviewPlacement).toHaveBeenCalledWith("P-006", expect.anything());
    });
    setRoute({});
    expect(screen.queryByText("操作できませんでした")).not.toBeInTheDocument();
    expect(screen.getByText("連動なしの糸")).toBeInTheDocument();

    await user.type(screen.getByLabelText(/^実際の数/), "9");
    await user.click(screen.getByRole("button", { name: "今の数を保存" }));
    await waitFor(() => {
      expect(screen.queryByRole("heading", { name: "今の数を入れる" })).not.toBeInTheDocument();
    });
    expect(onReturnTo).not.toHaveBeenCalled();

    // 途中の B の読込みの応答は捨てる
    resolvePreviewB();
    await act(async () => {
      for (let i = 0; i < 5; i += 1) await new Promise((resolve) => setTimeout(resolve, 0));
    });
    expect(screen.queryByText("緑の糸")).not.toBeInTheDocument();
    expect(screen.queryByText("操作できませんでした")).not.toBeInTheDocument();
  });

  it.each([
    ["a: removed before the response"],
    ["b: removed after the pending close"],
    ["c: removed without another product"],
    ["d: page unmounts"],
  ] as const)(
    "SF5 count_now save for A started with returnTo does not return after the route's product is removed or the page unmounts (%s) (73)",
    async (label) => {
      const save = holdSave();
      const { onReturnTo, onSearchChange, invalidateSpy, queryClient, setRoute, unmount } =
        renderCountNowRoute(new Promise(() => undefined));
      const user = userEvent.setup();
      await screen.findByText("連動なしの糸");
      await user.type(screen.getByLabelText(/^実際の数/), "9");
      await user.click(screen.getByRole("button", { name: "今の数を保存" }));
      await waitFor(() => {
        expect(mockSave).toHaveBeenCalledTimes(1);
      });

      if (label.startsWith("d")) {
        // サイドバーで別の画面へ移った後に保存の成功が届く
        const searchCalls = vi.mocked(onSearchChange).mock.calls.length;
        unmount();
        save.resolve(saveResult());
        await flushSaveFollowUp(invalidateSpy);
        expect(onReturnTo).not.toHaveBeenCalled();
        expect(onSearchChange).toHaveBeenCalledTimes(searchCalls);
        return;
      }

      if (!label.startsWith("c")) {
        setRoute({ product: "P-006" });
        await waitFor(() => {
          expect(mockPreviewPlacement).toHaveBeenCalledWith("P-006", expect.anything());
        });
      }
      if (label.startsWith("b")) {
        save.resolve(saveResult());
        await waitFor(() => {
          expect(screen.getByRole("button", { name: "今の数を保存" })).toBeEnabled();
        });
        await waitFor(() => {
          expect(queryClient.isFetching()).toBe(0);
        });
        await flushSaveFollowUp(invalidateSpy);
        // A の欄は B の読込みの決着を待つ閉じ待ちで残る
        expect(screen.getByText("連動なしの糸")).toBeInTheDocument();
        setRoute({});
      } else {
        setRoute({});
        // 外れた時点では欄を閉じない（未保存の入力を失わない）
        expect(screen.getByText("連動なしの糸")).toBeInTheDocument();
        expect(screen.queryByText("操作できませんでした")).not.toBeInTheDocument();
        save.resolve(saveResult());
      }

      await waitFor(() => {
        expect(screen.queryByRole("heading", { name: "今の数を入れる" })).not.toBeInTheDocument();
      });
      expect(onReturnTo).not.toHaveBeenCalled();
      expect(screen.queryByText("操作できませんでした")).not.toBeInTheDocument();
    },
  );

  it("SF6 pending count_now save for A keeps B's field after B loads and the route's product is removed (73)", async () => {
    const save = holdSave();
    const { onReturnTo, invalidateSpy, setRoute } = renderCountNowRoute(
      Promise.resolve(placementB()),
    );
    const user = userEvent.setup();
    await screen.findByText("連動なしの糸");
    await user.type(screen.getByLabelText(/^実際の数/), "9");
    await user.click(screen.getByRole("button", { name: "今の数を保存" }));
    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledTimes(1);
    });

    setRoute({ product: "P-006" });
    await screen.findByText("緑の糸");
    setRoute({});
    save.resolve(saveResult());
    await flushSaveFollowUp(invalidateSpy);

    expect(screen.getByRole("heading", { name: "今の数を入れる" })).toBeInTheDocument();
    expect(screen.getByText("緑の糸")).toBeInTheDocument();
    expect(onReturnTo).not.toHaveBeenCalled();
  });

  it("U8 confirm is locked while a count save is pending (73, Final Review G2)", async () => {
    let resolveSave: (value: ReturnType<typeof saveResult>) => void = () => undefined;
    mockSave.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolveSave = resolve;
        }),
    );
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();
    const user = await selectItemByCode();
    await user.type(screen.getByLabelText(/^実際の数/), "8{Enter}");
    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledTimes(1);
    });
    expect(screen.getByRole("button", { name: "棚卸しを確定する" })).toBeDisabled();

    resolveSave(saveResult());
    expect(await screen.findByText(`赤い糸 を 8 個（${TODAY}）で保存しました`)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "棚卸しを確定する" })).toBeEnabled();
  });

  it("U7 zero actions run once while pending and lock the other saves (73, Final Review G3)", async () => {
    let resolveZero: (value: ReturnType<typeof ok<[]>>) => void = () => undefined;
    const pendingZero = () =>
      new Promise<ReturnType<typeof ok<[]>>>((resolve) => {
        resolveZero = resolve;
      });
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockComplete.mockResolvedValueOnce({
      status: "error",
      error: {
        kind: "stocktake_guard",
        message: "棚卸しを完了できません",
        field: null,
        error_id: null,
        stocktake_recovery: {
          code: "completion_blocked",
          targets: [
            recheckTarget({
              product_code: "P-002",
              product_name: "青い糸",
              block_reason: "uncounted",
              stock_range_low: null,
              stock_range_high: null,
              recount_reasons: [],
            }),
            recheckTarget({
              product_code: "P-005",
              product_name: "白い糸",
              block_reason: "uncounted",
              stock_range_low: null,
              stock_range_high: null,
              recount_reasons: [],
            }),
          ],
          settlement_pending: null,
        },
      },
    });
    await renderPage();
    const user = userEvent.setup();
    await user.click(await screen.findByRole("button", { name: "棚卸しを確定する" }));
    await user.click(screen.getByRole("button", { name: "確定する" }));
    const panel = await screen.findByRole("region", { name: "棚卸しを確定できない理由" });
    const group = within(panel).getByRole("group", { name: "まだ数えていない" });

    // 「0 にする」の保留中は同じ button も他の保存・確定も押せない
    mockZero.mockImplementationOnce(pendingZero);
    const blueRow = within(group).getByText("青い糸").closest("li");
    if (blueRow === null) throw new Error("row missing");
    const zeroOne = within(blueRow).getByRole("button", { name: "0 にする" });
    await user.click(zeroOne);
    await waitFor(() => {
      expect(mockZero).toHaveBeenCalledTimes(1);
    });
    expect(zeroOne).toBeDisabled();
    expect(within(group).getByRole("button", { name: "残りをまとめて 0 にする" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "棚卸しを確定する" })).toBeDisabled();
    expect(screen.getByLabelText("商品を検索・スキャン")).toBeDisabled();
    await user.click(zeroOne);
    expect(mockZero).toHaveBeenCalledTimes(1);
    resolveZero(ok([]));
    await waitFor(() => {
      expect(within(group).queryByText("青い糸")).not.toBeInTheDocument();
    });

    // 「残りをまとめて 0 にする」も保留中は開き直せない
    mockZero.mockImplementationOnce(pendingZero);
    const zeroAll = within(group).getByRole("button", { name: "残りをまとめて 0 にする" });
    await user.click(zeroAll);
    await user.click(
      within(screen.getByRole("alertdialog")).getByRole("button", { name: "0 にする" }),
    );
    await waitFor(() => {
      expect(mockZero).toHaveBeenCalledTimes(2);
    });
    expect(zeroAll).toBeDisabled();
    expect(screen.getByRole("button", { name: "棚卸しを確定する" })).toBeDisabled();
    resolveZero(ok([]));
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "棚卸しを確定する" })).toBeEnabled();
    });
    expect(mockZero).toHaveBeenCalledTimes(2);
  });

  it("U5 confirming today's date in the date-change dialog saves once (73, Final Review G3)", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();
    let user = await selectItemByCode();
    await user.type(screen.getByLabelText(/^実際の数/), "8{Enter}");
    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledTimes(1);
    });
    vi.setSystemTime(new Date(2026, 9, 3, 0, 5, 0));
    user = await selectItemByCode();
    await user.type(screen.getByLabelText(/^実際の数/), "6{Enter}");
    let resolveSave: (value: ReturnType<typeof saveResult>) => void = () => undefined;
    mockSave.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolveSave = resolve;
        }),
    );
    await user.dblClick(await screen.findByRole("button", { name: "今日の分として保存する" }));
    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledTimes(2);
    });
    expect(screen.getByRole("button", { name: "数を保存" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "棚卸しを確定する" })).toBeDisabled();
    resolveSave(saveResult());
    expect(
      await screen.findByText("赤い糸 を 6 個（2026-10-03）で保存しました"),
    ).toBeInTheDocument();
    expect(mockSave).toHaveBeenCalledTimes(2);
  });

  it("U1 late search response does not replace the later search's field (73, Final Review G4)", async () => {
    let resolveFind: (value: ReturnType<typeof ok<StocktakeItemDetail>>) => void = () => undefined;
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockFindItem
      .mockImplementationOnce(
        () =>
          new Promise((resolve) => {
            resolveFind = resolve;
          }),
      )
      .mockResolvedValueOnce(ok(stocktakeItem({ id: 502, product_code: "P-002", name: "青い糸" })));
    await renderPage();
    const user = userEvent.setup();
    const code = await screen.findByLabelText("商品を検索・スキャン");
    await user.type(code, "P-001{Enter}");
    await user.clear(code);
    await user.type(code, "P-002{Enter}");
    expect(await screen.findByText(/^P-002 \//)).toBeInTheDocument();

    resolveFind(ok(baseStocktakeItem()));
    await waitFor(() => {
      expect(mockFindItem).toHaveBeenCalledTimes(2);
    });
    // 遅い P-001 の応答は P-002 の欄を置き換えない
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(screen.getByText(/^P-002 \//)).toBeInTheDocument();
    expect(screen.queryByText(/^P-001 \//)).not.toBeInTheDocument();
    await user.type(screen.getByLabelText(/^実際の数/), "7{Enter}");
    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledWith({
        ...defaultCount(7, 502),
        product_code: "P-002",
      });
    });
  });

  it("U6 late recount load does not replace the later recount's field (73, Final Review G4)", async () => {
    let resolvePreview: (value: ReturnType<typeof placement>) => void = () => undefined;
    mockGetActive.mockResolvedValue(ok(null));
    mockListRecheck.mockResolvedValue(
      ok([
        recheckTarget({ stocktake_item_id: null, action: "count_now" }),
        recheckTarget({
          product_code: "P-004",
          product_name: "生成りの紐",
          stocktake_item_id: null,
          action: "count_now",
        }),
      ]),
    );
    mockPreviewPlacement
      .mockImplementationOnce(
        () =>
          new Promise((resolve) => {
            resolvePreview = resolve;
          }),
      )
      .mockResolvedValueOnce(placement({ product_name: "生成りの紐" }));
    await renderPage();
    const list = await screen.findByRole("region", { name: "数え直しが必要な商品" });
    const user = userEvent.setup();
    await user.click(
      within(within(list).getByRole("row", { name: /黄色い糸/ })).getByRole("button", {
        name: "入れ直す",
      }),
    );
    await user.click(
      within(within(list).getByRole("row", { name: /生成りの紐/ })).getByRole("button", {
        name: "入れ直す",
      }),
    );
    const panel = (await screen.findByRole("heading", { name: "今の数を入れる" })).closest(
      "section",
    );
    if (panel === null) throw new Error("panel missing");
    expect(within(panel).getByText("生成りの紐")).toBeInTheDocument();

    resolvePreview(placement());
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(within(panel).getByText("生成りの紐")).toBeInTheDocument();
    expect(within(panel).queryByText("黄色い糸")).not.toBeInTheDocument();
  });

  it("U2 late back-date preview does not replace the later date's warning (73, Final Review G4)", async () => {
    let resolvePreview: (value: ReturnType<typeof placement>) => void = () => undefined;
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockPreviewPlacement
      .mockImplementationOnce(
        () =>
          new Promise((resolve) => {
            resolvePreview = resolve;
          }),
      )
      .mockResolvedValueOnce(placement({ later_count_exists: false }));
    await renderPage();
    const user = await selectItemByCode();
    await user.click(screen.getByRole("button", { name: "前の日の分を入れる" }));
    fireEvent.change(screen.getByLabelText("数えた日"), { target: { value: "2026-09-30" } });
    fireEvent.change(screen.getByLabelText("数えた日"), { target: { value: "2026-09-29" } });
    await waitFor(() => {
      expect(mockPreviewPlacement).toHaveBeenCalledWith("P-001", "2026-09-29");
    });

    resolvePreview(placement({ later_count_exists: true }));
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(screen.queryByText("後の数が既にあり、そちらが効きます")).not.toBeInTheDocument();
  });

  it("U2 back-dated count warns when a later count exists", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockPreviewPlacement.mockResolvedValue(placement({ later_count_exists: true }));
    await renderPage();
    const user = await selectItemByCode();

    await user.click(screen.getByRole("button", { name: "前の日の分を入れる" }));
    const date = screen.getByLabelText("数えた日");
    expect(date).toHaveAttribute("type", "date");
    expect(date).toHaveAttribute("max", TODAY);
    expect(screen.queryByRole("button", { name: "入れ直す" })).not.toBeInTheDocument();
    fireEvent.change(date, { target: { value: "2026-09-30" } });

    await waitFor(() => {
      expect(mockPreviewPlacement).toHaveBeenCalledWith("P-001", "2026-09-30");
    });
    expect(await screen.findByText("後の数が既にあり、そちらが効きます")).toBeInTheDocument();
    expect(mockSave).not.toHaveBeenCalled();
    await user.type(screen.getByLabelText(/^実際の数/), "10");
    await user.click(screen.getByRole("button", { name: "数を保存" }));
    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledWith({ ...defaultCount(10), business_date: "2026-09-30" });
    });
  });

  it("U2 back-dated count without a later count shows no warning and a future date is refused", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockPreviewPlacement.mockResolvedValue(placement({ later_count_exists: false }));
    await renderPage();
    const user = await selectItemByCode();
    await user.click(screen.getByRole("button", { name: "前の日の分を入れる" }));
    fireEvent.change(screen.getByLabelText("数えた日"), { target: { value: "2026-09-29" } });
    await waitFor(() => {
      expect(mockPreviewPlacement).toHaveBeenCalledWith("P-001", "2026-09-29");
    });
    expect(screen.queryByText("後の数が既にあり、そちらが効きます")).not.toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("数えた日"), { target: { value: "2026-10-05" } });
    await user.type(screen.getByLabelText(/^実際の数/), "3");
    await user.click(screen.getByRole("button", { name: "数を保存" }));
    expect(await screen.findByText("未来の日付には保存できません")).toBeInTheDocument();
    expect(mockSave).not.toHaveBeenCalled();
  });

  it("U3 recount sends fresh true with today only", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockFindItem.mockResolvedValueOnce(ok(baseStocktakeItem())).mockResolvedValueOnce(
      ok(
        stocktakeItem({
          count_value: 5,
          count_kind: "default",
          count_business_date: "2026-10-01",
          actual_count: 5,
        }),
      ),
    );
    await renderPage();
    // 前に保存した数が無いときは「入れ直す」を出さない
    const user = await selectItemByCode();
    expect(screen.queryByRole("button", { name: "入れ直す" })).not.toBeInTheDocument();

    await user.clear(screen.getByLabelText("商品を検索・スキャン"));
    await user.type(screen.getByLabelText("商品を検索・スキャン"), "P-001{Enter}");
    expect(await screen.findByText("前に保存した数: 5 個（2026-10-01）")).toBeInTheDocument();
    await user.type(screen.getByLabelText(/^実際の数/), "4");
    await user.click(screen.getByRole("button", { name: "入れ直す" }));

    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledWith({ ...defaultCount(4), fresh: true });
    });
  });

  it("U4 length product count converts meters to cm", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockFindItem.mockResolvedValue(ok(stocktakeItem({ stock_unit: "m", current_stock: 500 })));
    await renderPage();
    const user = await selectItemByCode();
    const input = screen.getByLabelText("実際の数（m、小数 2 桁まで）");
    expect(input).toHaveAttribute("inputmode", "decimal");

    for (const [text, error] of [
      ["1.234", "数量は0以上で、小数は2桁までの m で入力してください"],
      ["-1", "数量は0以上で、小数は2桁までの m で入力してください"],
      ["10000000", "数が大きすぎます"],
    ] as const) {
      await user.clear(input);
      await user.type(input, `${text}{Enter}`);
      expect(await screen.findByText(error)).toBeInTheDocument();
    }
    expect(mockSave).not.toHaveBeenCalled();

    await user.clear(input);
    await user.type(input, "0.29{Enter}");
    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledWith(defaultCount(29));
    });
  });

  it("U5 date change since last save asks before saving", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();
    let user = await selectItemByCode();
    await user.type(screen.getByLabelText(/^実際の数/), "8{Enter}");
    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledTimes(1);
    });

    // 画面を開いたまま PC の日付が変わる
    vi.setSystemTime(new Date(2026, 9, 3, 0, 5, 0));
    user = await selectItemByCode();
    await user.type(screen.getByLabelText(/^実際の数/), "6{Enter}");
    expect(
      await screen.findByText(
        "日付が変わりました（2026-10-02 → 2026-10-03）。今日の分として保存しますか",
      ),
    ).toBeInTheDocument();
    expect(mockSave).toHaveBeenCalledTimes(1);

    await user.click(screen.getByRole("button", { name: "キャンセル" }));
    expect(mockSave).toHaveBeenCalledTimes(1);
    await user.click(screen.getByRole("button", { name: "数を保存" }));
    await user.click(await screen.findByRole("button", { name: "今日の分として保存する" }));
    await waitFor(() => {
      expect(mockSave).toHaveBeenLastCalledWith({
        ...defaultCount(6),
        business_date: "2026-10-03",
      });
    });
  });

  it("U6 recheck list shows range and recount resolves the row", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockListRecheck
      .mockResolvedValueOnce(
        ok([
          recheckTarget(),
          recheckTarget({
            product_code: "P-004",
            product_name: "生成りの紐",
            stock_unit: "m",
            stock: 450,
            stock_range_low: 450,
            stock_range_high: 520,
            stocktake_item_id: null,
            action: "count_now",
            recount_reasons: ["conflicting_counts"],
          }),
        ]),
      )
      .mockResolvedValue(
        ok([
          recheckTarget({
            product_code: "P-004",
            product_name: "生成りの紐",
            stock_unit: "m",
            stock: 450,
            stock_range_low: 450,
            stock_range_high: 520,
            stocktake_item_id: null,
          }),
        ]),
      );
    mockPreviewPlacement.mockResolvedValue(placement());
    const { invalidateSpy } = await renderPage();

    const list = await screen.findByRole("region", { name: "数え直しが必要な商品" });
    const row = within(list).getByRole("row", { name: /黄色い糸/ });
    expect(within(row).getByText("3〜6 個")).toBeInTheDocument();
    expect(within(row).getByText("要再確認")).toBeInTheDocument();
    expect(within(row).getByText("数えた日に売り買い（入出庫）がありました")).toBeInTheDocument();
    const lengthRow = within(list).getByRole("row", { name: /生成りの紐/ });
    expect(within(lengthRow).getByText("4.5〜5.2 m")).toBeInTheDocument();
    expect(within(lengthRow).getByText("同じ日に違う数が保存されています")).toBeInTheDocument();

    const user = userEvent.setup();
    await user.click(within(row).getByRole("button", { name: "入れ直す" }));
    await waitFor(() => {
      expect(mockPreviewPlacement).toHaveBeenCalledWith("P-003", TODAY);
    });
    const panel = (await screen.findByRole("heading", { name: "今の数を入れる" })).closest(
      "section",
    );
    if (panel === null) throw new Error("panel missing");
    expect(within(panel).getByText("黄色い糸")).toBeInTheDocument();
    expect(within(panel).getByText("前に保存した数: 4 個（2026-10-01）")).toBeInTheDocument();
    invalidateSpy.mockClear();
    await user.type(within(panel).getByLabelText(/^実際の数/), "5");
    await user.click(within(panel).getByRole("button", { name: "今の数を保存" }));

    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledWith({
        product_code: "P-003",
        count_value: 5,
        fresh: true,
        business_date: TODAY,
        stocktake_item_id: 503,
      });
    });
    await waitFor(() => {
      expect(within(list).queryByRole("row", { name: /黄色い糸/ })).not.toBeInTheDocument();
    });
    expectExactInvalidations(
      invalidateSpy.mock.calls,
      d052InvalidationOracle.stocktakeCountUpdate(),
    );
    expect(screen.queryByRole("heading", { name: "今の数を入れる" })).not.toBeInTheDocument();
  });

  it("U7 completion blocked lists reasons, settlement hint and zero actions", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    const blockTarget = (
      code: string,
      name: string,
      reason: "recheck" | "uncounted" | "settlement_pending",
    ) =>
      recheckTarget({
        product_code: code,
        product_name: name,
        block_reason: reason,
        stock_range_low: reason === "recheck" ? 3 : null,
        stock_range_high: reason === "recheck" ? 6 : null,
        recount_reasons: reason === "recheck" ? ["same_date_movement"] : [],
      });
    mockComplete.mockResolvedValueOnce({
      status: "error",
      error: {
        kind: "stocktake_guard",
        message: "棚卸しを完了できません",
        field: null,
        error_id: null,
        stocktake_recovery: {
          code: "completion_blocked",
          targets: [
            blockTarget("P-001", "赤い糸", "recheck"),
            blockTarget("P-002", "青い糸", "uncounted"),
            blockTarget("P-005", "白い糸", "uncounted"),
            blockTarget("P-003", "黄色い糸", "settlement_pending"),
          ],
          settlement_pending: {
            missing: [
              { from: 103, to: 103 },
              { from: 105, to: 107 },
            ],
            on_or_after: "2026-10-01",
            before: "2026-09-28",
          },
        },
      },
    });
    await renderPage();
    const user = userEvent.setup();

    await user.click(await screen.findByRole("button", { name: "棚卸しを確定する" }));
    const dialog = screen.getByRole("alertdialog");
    expect(within(dialog).getByRole("heading", { name: "棚卸しの確定" })).toBeInTheDocument();
    const warning = within(dialog)
      .getByText("確定すると取り消せません")
      .closest('[data-slot="alert"]');
    expect(warning).toHaveAttribute("data-variant", "warning");
    expect(
      within(dialog).getByText(
        "今日の最後の精算（締め）を取り込みましたか。取り込む前に完了すると、その精算の売上は棚卸しに入りません",
      ),
    ).toBeInTheDocument();
    expect(within(dialog).getByRole("button", { name: "確定する" })).toHaveAttribute(
      "data-variant",
      "destructive",
    );
    await user.click(within(dialog).getByRole("button", { name: "確定する" }));
    await waitFor(() => {
      expect(mockComplete).toHaveBeenCalledWith(77);
    });

    const panel = await screen.findByRole("region", { name: "棚卸しを確定できない理由" });
    const recheckGroup = within(panel).getByRole("group", { name: "数え直しが必要" });
    expect(within(recheckGroup).getByText("赤い糸")).toBeInTheDocument();
    expect(within(recheckGroup).getByText("3〜6 個")).toBeInTheDocument();
    expect(within(recheckGroup).queryByText("青い糸")).not.toBeInTheDocument();
    const uncountedGroup = within(panel).getByRole("group", { name: "まだ数えていない" });
    expect(within(uncountedGroup).getByText("青い糸")).toBeInTheDocument();
    expect(within(uncountedGroup).getByText("白い糸")).toBeInTheDocument();
    expect(within(uncountedGroup).queryByText("赤い糸")).not.toBeInTheDocument();
    const pendingGroup = within(panel).getByRole("group", { name: "取り込んでいない精算がある" });
    expect(within(pendingGroup).getByText("黄色い糸")).toBeInTheDocument();
    for (const line of [
      "精算 103 を取り込んでください",
      "精算 105〜107 を取り込んでください",
      "精算日が 2026-10-01 以降の精算を取り込んでください",
      "精算日が 2026-09-28 より前の精算を 1 本取り込んでください",
    ]) {
      expect(within(pendingGroup).getByText(line)).toBeInTheDocument();
    }

    // 「0 にする」は選んだ商品、日付は既定で今日
    const blueRow = within(uncountedGroup).getByText("青い糸").closest("li");
    if (blueRow === null) throw new Error("row missing");
    await user.click(within(blueRow).getByRole("button", { name: "0 にする" }));
    await waitFor(() => {
      expect(mockZero).toHaveBeenCalledWith(77, ["P-002"], TODAY);
    });
    await waitFor(() => {
      expect(within(uncountedGroup).queryByText("青い糸")).not.toBeInTheDocument();
    });

    // 「残りをまとめて 0 にする」は確認を挟み、日付を選べる
    fireEvent.change(within(uncountedGroup).getByLabelText("0 にする日付"), {
      target: { value: "2026-10-01" },
    });
    await user.click(
      within(uncountedGroup).getByRole("button", { name: "残りをまとめて 0 にする" }),
    );
    expect(
      await screen.findByText("まだ数えていない 1件を、2026-10-01 の数 0 として保存します。"),
    ).toBeInTheDocument();
    expect(mockZero).toHaveBeenCalledTimes(1);
    await user.click(
      within(screen.getByRole("alertdialog")).getByRole("button", { name: "0 にする" }),
    );
    await waitFor(() => {
      expect(mockZero).toHaveBeenLastCalledWith(77, ["P-005"], "2026-10-01");
    });
    // 理由ごとの一覧は残る（精算の理由は取込みで解ける）
    expect(
      within(panel).getByRole("group", { name: "取り込んでいない精算がある" }),
    ).toBeInTheDocument();
  });

  it("U7 unknown recovery code is not treated as completion", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockComplete.mockResolvedValueOnce({
      status: "error",
      error: {
        kind: "stocktake_guard",
        message: "棚卸しを完了できません（合成の未知の code）",
        field: null,
        error_id: null,
        stocktake_recovery: {
          code: "future_code",
          targets: [],
          settlement_pending: null,
        } as unknown as NonNullable<CmdErrorShape["stocktake_recovery"]>,
      },
    });
    await renderPage();
    const user = userEvent.setup();
    await user.click(await screen.findByRole("button", { name: "棚卸しを確定する" }));
    await user.click(screen.getByRole("button", { name: "確定する" }));
    expect(
      await screen.findByText("棚卸しを完了できません（合成の未知の code）"),
    ).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "棚卸し結果" })).not.toBeInTheDocument();
    expect(
      screen.queryByRole("region", { name: "棚卸しを確定できない理由" }),
    ).not.toBeInTheDocument();
  });

  it("U8 completion result shows valuation without adjustments", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();
    const user = userEvent.setup();
    await user.click(await screen.findByRole("button", { name: "棚卸しを確定する" }));
    await user.click(screen.getByRole("button", { name: "確定する" }));

    expect(await screen.findByRole("heading", { name: "棚卸し結果" })).toBeInTheDocument();
    expect(screen.getByText("¥2,500")).toBeInTheDocument();
    expect(screen.getByText("明細の数: 2件")).toBeInTheDocument();
    expect(
      screen.getByText("確定では在庫は動きません（数を保存したときに在庫へ入っています）"),
    ).toBeInTheDocument();
    expect(screen.queryByText("差異のあった商品")).not.toBeInTheDocument();
  });

  it.each([
    ["no active stocktake", null, null],
    ["active stocktake with the item", activeStocktake(), 505],
  ] as const)("U9 count_now entry saves fresh and returns (%s)", async (_label, active, itemId) => {
    mockGetActive.mockResolvedValue(ok(active));
    mockFindItem.mockResolvedValue(
      ok(stocktakeItem({ id: 505, product_code: "P-005", name: "連動なしの糸" })),
    );
    mockPreviewPlacement.mockResolvedValue(
      placement({
        product_name: "連動なしの糸",
        stock: 10,
        stock_range_low: null,
        stock_range_high: null,
        recount_reasons: [],
        count_value: null,
        count_kind: null,
        count_business_date: null,
      }),
    );
    const { onReturnTo } = renderWithReturn({ product: "P-005", returnTo: "/products/P-005/edit" });

    const panel = (await screen.findByRole("heading", { name: "今の数を入れる" })).closest(
      "section",
    );
    if (panel === null) throw new Error("panel missing");
    expect(await within(panel).findByText("連動なしの糸")).toBeInTheDocument();
    expect(within(panel).getByText(/現在在庫 10 個/)).toBeInTheDocument();
    expect(mockPreviewPlacement).toHaveBeenCalledWith("P-005", TODAY);
    const user = userEvent.setup();
    await user.type(within(panel).getByLabelText(/^実際の数/), "9");
    await user.click(within(panel).getByRole("button", { name: "今の数を保存" }));

    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledWith({
        product_code: "P-005",
        count_value: 9,
        fresh: true,
        business_date: TODAY,
        stocktake_item_id: itemId,
      });
    });
    await waitFor(() => {
      expect(onReturnTo).toHaveBeenCalledWith("/products/P-005/edit");
    });
  });

  it("U9 count_now save failure does not return", async () => {
    mockGetActive.mockResolvedValue(ok(null));
    mockPreviewPlacement.mockResolvedValue(placement({ product_name: "連動なしの糸" }));
    mockSave.mockResolvedValueOnce(cmdError("validation", "数が大きすぎます"));
    const { onReturnTo } = renderWithReturn({ product: "P-005", returnTo: "/products/P-005/edit" });
    const user = userEvent.setup();
    await user.type(await screen.findByLabelText(/^実際の数/), "9");
    await user.click(screen.getByRole("button", { name: "今の数を保存" }));
    expect(await screen.findByText("数が大きすぎます")).toBeInTheDocument();
    expect(onReturnTo).not.toHaveBeenCalled();
  });

  it.each([["/settings"], ["/products/P-999/edit"], [undefined]] as const)(
    "U9 returnTo outside the product's edit form stays on stocktake (%s)",
    async (returnTo) => {
      mockGetActive.mockResolvedValue(ok(null));
      mockPreviewPlacement.mockResolvedValue(placement({ product_name: "連動なしの糸" }));
      const { onReturnTo, onSearchChange } = renderWithReturn({ product: "P-005", returnTo });
      const user = userEvent.setup();
      await user.type(await screen.findByLabelText(/^実際の数/), "9");
      await user.click(screen.getByRole("button", { name: "今の数を保存" }));
      await waitFor(() => {
        expect(mockSave).toHaveBeenCalledTimes(1);
      });
      await waitFor(() => {
        expect(screen.queryByRole("heading", { name: "今の数を入れる" })).not.toBeInTheDocument();
      });
      expect(onReturnTo).not.toHaveBeenCalled();
      const updater = vi.mocked(onSearchChange).mock.calls[
        vi.mocked(onSearchChange).mock.calls.length - 1
      ]?.[0] as (prev: StocktakeSearch) => StocktakeSearch;
      expect(updater({ product: "P-005", returnTo, dept: 1 })).toEqual({
        product: undefined,
        returnTo: undefined,
        dept: 1,
      });
    },
  );

  it("U15 count save and zero use the stocktakeCountUpdate entry", async () => {
    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    mockComplete.mockResolvedValueOnce({
      status: "error",
      error: {
        kind: "stocktake_guard",
        message: "棚卸しを完了できません",
        field: null,
        error_id: null,
        stocktake_recovery: {
          code: "completion_blocked",
          targets: [
            recheckTarget({
              product_code: "P-002",
              product_name: "青い糸",
              block_reason: "uncounted",
              stock_range_low: null,
              stock_range_high: null,
              recount_reasons: [],
            }),
          ],
          settlement_pending: null,
        },
      },
    });
    const { invalidateSpy } = await renderPage();
    const user = await selectItemByCode();
    invalidateSpy.mockClear();
    await user.type(screen.getByLabelText(/^実際の数/), "8{Enter}");
    await waitFor(() => {
      expect(mockSave).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expectExactInvalidations(
        invalidateSpy.mock.calls,
        d052InvalidationOracle.stocktakeCountUpdate(),
      );
    });

    await user.click(screen.getByRole("button", { name: "棚卸しを確定する" }));
    await user.click(screen.getByRole("button", { name: "確定する" }));
    const panel = await screen.findByRole("region", { name: "棚卸しを確定できない理由" });
    invalidateSpy.mockClear();
    await user.click(within(panel).getByRole("button", { name: "0 にする" }));
    await waitFor(() => {
      expect(mockZero).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expectExactInvalidations(
        invalidateSpy.mock.calls,
        d052InvalidationOracle.stocktakeCountUpdate(),
      );
    });
  });

  it("U16 stocktake and z004 writes are enabled", async () => {
    const first = await renderPage();
    expect(await screen.findByRole("button", { name: "棚卸しを開始する" })).toBeEnabled();
    expect(screen.queryByText(/停止中/)).not.toBeInTheDocument();
    first.unmount();

    mockGetActive.mockResolvedValue(ok(activeStocktake()));
    await renderPage();
    await screen.findByText("棚卸し中（開始日: 2026-10-01 09:00:00）");
    expect(screen.queryByText(/停止中/)).not.toBeInTheDocument();
    expect(screen.getByLabelText("商品を検索・スキャン")).toBeEnabled();
    expect(screen.getByRole("button", { name: "対象を確認" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "棚卸しを確定する" })).toBeEnabled();
    const user = await selectItemByCode();
    expect(screen.getByRole("button", { name: "数を保存" })).toBeEnabled();
    await user.click(screen.getByRole("button", { name: "棚卸しを確定する" }));
    expect(screen.getByRole("alertdialog")).toBeInTheDocument();
  });
});

type CmdErrorShape = import("@/lib/bindings").CmdError;
