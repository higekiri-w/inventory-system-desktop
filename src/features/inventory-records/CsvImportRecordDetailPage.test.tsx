import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, createRouter, RouterProvider } from "@tanstack/react-router";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { toast } from "sonner";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { commands } from "@/lib/bindings";
import type {
  CmdError,
  CountRecoveryTarget,
  CsvImportRecordDetail,
  MovementRecord,
} from "@/lib/bindings";
import { routeTree } from "@/routeTree.gen";
import { d052InvalidationOracle, expectExactInvalidations } from "@/test/invalidation-oracle";
import { renderWithRouter } from "@/test/render-with-router";
import { CsvImportRecordDetailPage } from "./CsvImportRecordDetailPage";

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

vi.mock("@/features/daily-report-import/DailyReportImportPage", () => ({
  DailyReportImportPage: () => <div>日報取込み初期画面</div>,
}));

vi.mock("@/features/csv-import/hooks/useCsvImportFlow", () => ({
  useCsvImportFlow: () => ({
    state: { status: "idle" },
    selectFile: vi.fn(),
    confirmImport: vi.fn(),
    rollback: vi.fn(),
    dismissError: vi.fn(),
    isParsing: false,
    isImporting: false,
    isRollingBack: false,
  }),
}));

vi.mock("@/lib/bindings", () => ({
  commands: {
    getCsvImportRecord: vi.fn(),
    previewRollbackCsvImport: vi.fn(),
    rollbackCsvImport: vi.fn(),
  },
}));

vi.mock("sonner", async (importOriginal) => {
  const actual = await importOriginal<typeof import("sonner")>();
  return { ...actual, toast: { success: vi.fn(), error: vi.fn() } };
});

const mockGetCsvImportRecord = vi.mocked(commands.getCsvImportRecord);
const mockPreviewRollback = vi.mocked(commands.previewRollbackCsvImport);
const mockRollback = vi.mocked(commands.rollbackCsvImport);
const mockToast = vi.mocked(toast);

function makeMovement(): MovementRecord {
  return {
    id: 501,
    product_code: "Z004",
    movement_type: "sale_auto",
    quantity: -2,
    stock_after: 8,
    reference_type: "csv_import",
    reference_id: 41,
    source: { label: "CSV取込み #41", route: "/csv-import/records/41" },
    note: null,
    created_at: "2026-08-03T10:11:12",
  };
}

function makeDetail(overrides: Partial<CsvImportRecordDetail> = {}): CsvImportRecordDetail {
  return {
    id: 41,
    filename: "synthetic-z004.csv",
    settlement_date: "2026-08-02",
    total_items: 1,
    total_amount: 880,
    skipped_count: 0,
    status: "completed",
    imported_at: "2026-08-03T10:11:12",
    items: [
      {
        id: 601,
        product_code: "Z004",
        product_name: "合成テスト商品",
        department_name: "テスト部門",
        stock_unit: "pcs",
        quantity: 2,
        amount: 880,
        is_voided: false,
      },
    ],
    error_rows: [],
    movements: [makeMovement()],
    ...overrides,
  };
}

function makeQueryClient() {
  return new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
}

function renderWithClient(ui: ReactNode, initialPath = "/") {
  return renderWithRouter(
    <QueryClientProvider client={makeQueryClient()}>{ui}</QueryClientProvider>,
    initialPath,
  );
}

beforeEach(() => {
  mockGetCsvImportRecord.mockReset();
  mockPreviewRollback.mockReset();
  mockRollback.mockReset();
  mockToast.success.mockReset();
  mockToast.error.mockReset();
  mockPreviewRollback.mockResolvedValue({
    status: "ok",
    data: { csv_import_id: 41, settlement_no: 101, recheck_targets: [], affected_stocktakes: [] },
  });
});

function recheckTarget(): CountRecoveryTarget {
  return {
    product_code: "Z004",
    product_name: "合成テスト商品",
    stock_unit: "pcs",
    stock: 9,
    stock_range_low: 9,
    stock_range_high: 10,
    stocktake_item_id: null,
    stocktake_id: null,
    action: "count_now",
    block_reason: null,
    recount_reasons: ["undetermined_settlement"],
  };
}

/** 詳細の「この取込みを取り消す」から確認を開き、読取りの後に「取り消す」を押す */
async function confirmRollback(user: ReturnType<typeof userEvent.setup>) {
  await user.click(await screen.findByRole("button", { name: "この取込みを取り消す" }));
  const dialog = screen.getByRole("alertdialog");
  expect(await within(dialog).findByText("精算 101")).toBeInTheDocument();
  await user.click(within(dialog).getByRole("button", { name: "取り消す" }));
}

describe("CsvImportRecordDetailPage (REQ-206 / REQ-207)", () => {
  it("REQ-401: /csv-import 直接進入で既存の取込み画面を描画する", async () => {
    const router = createRouter({
      routeTree,
      history: createMemoryHistory({ initialEntries: ["/csv-import"] }),
    });
    render(
      <QueryClientProvider client={makeQueryClient()}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    );

    expect(await screen.findByRole("heading", { name: "売上データ取込み" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "日報取込み" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(screen.getByRole("tab", { name: "商品別CSV取込み（Z004）" })).toBeInTheDocument();
  });

  it("REQ-206: completed のヘッダ・明細・movement・商品リンクを表示する", async () => {
    mockGetCsvImportRecord.mockResolvedValue({ status: "ok", data: makeDetail() });

    renderWithClient(<CsvImportRecordDetailPage importId={41} />);

    expect(await screen.findByRole("heading", { name: "CSV取込み #41" })).toBeInTheDocument();
    expect(screen.getByText("synthetic-z004.csv")).toBeInTheDocument();
    expect(screen.getByText("2026-08-02")).toBeInTheDocument();
    expect(screen.getByText("成功")).toBeInTheDocument();
    expect(screen.getAllByText("¥880")).toHaveLength(2);
    expect(screen.getAllByText("2026-08-03 10:11:12")).toHaveLength(2);

    const itemRow = screen.getByRole("row", { name: /Z004 合成テスト商品/ });
    expect(within(itemRow).getByText("テスト部門")).toBeInTheDocument();
    expect(within(itemRow).getByText("2 個")).toBeInTheDocument();
    expect(within(itemRow).getByText("有効")).toBeInTheDocument();
    expect(within(itemRow).getByRole("link", { name: "Z004 の在庫変動履歴" })).toHaveAttribute(
      "href",
      "/stock/Z004/movements",
    );
    expect(screen.getByRole("link", { name: "CSV取込み #41" })).toHaveAttribute(
      "href",
      "/csv-import/records/41",
    );
    expect(screen.getByText("取込みエラーはありません")).toBeInTheDocument();
  });

  it("REQ-206: completed_partial を正規化しエラー行を表示する", async () => {
    mockGetCsvImportRecord.mockResolvedValue({
      status: "ok",
      data: makeDetail({
        status: "completed_partial",
        skipped_count: 1,
        error_rows: [
          {
            line_no: 7,
            normalized_jan: "4900000000004",
            name: "合成エラー商品",
            raw_quantity: "abc",
            raw_amount: "440",
            error_type: "invalid_number",
            error_message: "数量を数値として解釈できません",
          },
        ],
      }),
    });

    renderWithClient(<CsvImportRecordDetailPage importId={41} />);

    expect(await screen.findByText("部分成功")).toBeInTheDocument();
    const errorRow = screen.getByRole("row", { name: /合成エラー商品/ });
    expect(within(errorRow).getByText("7")).toBeInTheDocument();
    expect(within(errorRow).getByText("4900000000004")).toBeInTheDocument();
    expect(within(errorRow).getByText("数値エラー")).toBeInTheDocument();
    expect(within(errorRow).getByText("数量を数値として解釈できません")).toBeInTheDocument();
  });

  it("REQ-207: movement の元記録をクリックすると詳細 route を描画する", async () => {
    mockGetCsvImportRecord.mockResolvedValue({ status: "ok", data: makeDetail() });
    const user = userEvent.setup();
    const router = createRouter({
      routeTree,
      history: createMemoryHistory({ initialEntries: ["/stock/Z004/movements"] }),
    });
    render(
      <QueryClientProvider client={makeQueryClient()}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    );

    await user.click(await screen.findByRole("link", { name: "CSV取込み #41" }));

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/csv-import/records/41");
    });
    await waitFor(() => {
      expect(mockGetCsvImportRecord).toHaveBeenCalledWith(41);
    });
    expect(await screen.findByRole("heading", { name: "CSV取込み #41" })).toBeInTheDocument();
  });

  it("REQ-206: NotFound を利用者向け日本語で表示する", async () => {
    mockGetCsvImportRecord.mockResolvedValue({
      status: "error",
      error: {
        kind: "not_found",
        message: "CSV取込み記録が見つかりません: 404",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });

    renderWithClient(<CsvImportRecordDetailPage importId={404} />);

    expect(await screen.findByText("CSV取込み記録が見つかりません: 404")).toBeInTheDocument();
    expect(
      screen.getByText("記録IDを確認するか、在庫変動履歴から開き直してください。"),
    ).toBeInTheDocument();
  });

  it("REQ-206: rolled_back は取消済み明細と movement 0 件を正常表示する", async () => {
    mockGetCsvImportRecord.mockResolvedValue({
      status: "ok",
      data: makeDetail({
        status: "rolled_back",
        items: makeDetail().items.map((item) => ({ ...item, is_voided: true })),
        movements: [],
      }),
    });

    renderWithClient(<CsvImportRecordDetailPage importId={41} />);

    expect(await screen.findByText("この取込みは取消済みです")).toBeInTheDocument();
    expect(screen.getByText("取消済み")).toBeInTheDocument();
    expect(screen.getByText("明細取消済み")).toBeInTheDocument();
    expect(screen.getByText("合成テスト商品")).toBeInTheDocument();
    expect(screen.getByText("関連する在庫変動がありません")).toBeInTheDocument();
  });

  it.each([
    ["/stock/Z004/movements?type=sale_pos&page=2", "/stock/Z004/movements?type=sale_pos&page=2"],
    ["https://example.invalid/escape", "/inventory/records"],
    ["//example.invalid/escape", "/inventory/records"],
  ])("REQ-207: returnTo %s の戻り先を安全に %s へ正規化する", async (returnTo, expected) => {
    mockGetCsvImportRecord.mockResolvedValue({ status: "ok", data: makeDetail() });

    renderWithClient(<CsvImportRecordDetailPage importId={41} returnTo={returnTo} />);

    expect(await screen.findByRole("link", { name: "前の画面へ戻る" })).toHaveAttribute(
      "href",
      expected,
    );
  });
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
  [
    "rolled_back",
    "取消済み",
    "destructive",
    ["border-destructive-border", "bg-destructive-soft", "text-destructive-strong"],
  ],
] as const)(
  "SC8 / REQ-401: %s preserves the state meaning",
  async (status, label, tone, classes) => {
    const detail = makeDetail({ status });
    detail.items[0].is_voided = status === "rolled_back";
    mockGetCsvImportRecord.mockResolvedValue({ status: "ok", data: detail });
    renderWithClient(<CsvImportRecordDetailPage importId={41} />);
    const badge = (await screen.findByText(label)).closest('[data-slot="badge"]');
    expect(badge).toHaveAttribute("data-variant", "outline");
    expect(badge).toHaveAttribute("data-tone", tone);
    expect(badge).toHaveClass(...classes);
    expect(badge?.querySelector('svg[aria-hidden="true"]')).toBeInTheDocument();
    if (status === "rolled_back") {
      const itemBadge = screen.getByText("明細取消済み").closest('[data-slot="badge"]');
      expect(itemBadge).toHaveAttribute("data-tone", "destructive");
      expect(itemBadge).toHaveClass(
        "border-destructive-border",
        "bg-destructive-soft",
        "text-destructive-strong",
      );
      expect(itemBadge?.querySelector('svg[aria-hidden="true"]')).toBeInTheDocument();
    } else {
      expect(screen.getByText("有効")).not.toHaveAttribute("data-tone");
    }
  },
);

it.each([
  ["pcs", "1,234 個"],
  ["m", "12.34 m"],
] as const)("⑮ SC19: CSV詳細の %s を単位付き表示する", async (unit, expected) => {
  const detail = makeDetail();
  detail.items = [{ ...detail.items[0], stock_unit: unit, quantity: 1234 }];
  mockGetCsvImportRecord.mockResolvedValue({ status: "ok", data: detail });
  renderWithClient(<CsvImportRecordDetailPage importId={41} />);
  expect(await screen.findByText(expected)).toBeInTheDocument();
});

// REQ-401 / UI-06c-D5 / SPEC-UNIT-D3 の表の 6: CSV 取込みの記録詳細の関連する在庫変動も明細の単位で出す
it("REQ-401: m の商品の取込みは 1.3 m、関連する在庫変動は -1.3 m・26.2 m", async () => {
  const detail = makeDetail();
  const [item] = detail.items;
  detail.items = [{ ...item, product_code: "CSV-M", stock_unit: "m", quantity: 130 }];
  detail.movements = [
    { ...makeMovement(), product_code: "CSV-M", quantity: -130, stock_after: 2620 },
  ];
  mockGetCsvImportRecord.mockResolvedValue({ status: "ok", data: detail });
  renderWithClient(<CsvImportRecordDetailPage importId={41} />);

  expect(await screen.findByText("1.3 m")).toBeInTheDocument();
  expect(screen.getByText("-1.3 m")).toBeInTheDocument();
  expect(screen.getByText("26.2 m")).toBeInTheDocument();
});

describe("CsvImportRecordDetailPage の取消（65 §65.5a TRACE-D13）", () => {
  it("REQ-207: 成功・部分成功の詳細に「この取込みを取り消す」を出し、取消済みには出さない", async () => {
    for (const status of ["completed", "completed_partial"] as const) {
      mockGetCsvImportRecord.mockResolvedValue({ status: "ok", data: makeDetail({ status }) });
      const { unmount } = renderWithClient(<CsvImportRecordDetailPage importId={41} />);
      expect(
        await screen.findByRole("button", { name: "この取込みを取り消す" }),
      ).toBeInTheDocument();
      // header の操作に置き、明細の行には取消を置かない
      const itemRow = screen.getByRole("row", { name: /Z004 合成テスト商品/ });
      expect(within(itemRow).queryByRole("button")).toBeNull();
      unmount();
    }
    mockGetCsvImportRecord.mockResolvedValue({
      status: "ok",
      data: makeDetail({ status: "rolled_back", movements: [] }),
    });
    renderWithClient(<CsvImportRecordDetailPage importId={41} />);
    expect(await screen.findByText("この取込みは取消済みです")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "この取込みを取り消す" })).toBeNull();
  });

  it("REQ-401: 詳細から取り消すと同じ ID で取消し、C9 で再取得し、取消の結果と Z004 の導線を出す", async () => {
    const user = userEvent.setup();
    mockGetCsvImportRecord
      .mockResolvedValueOnce({ status: "ok", data: makeDetail() })
      .mockResolvedValue({
        status: "ok",
        data: makeDetail({ status: "rolled_back", movements: [] }),
      });
    mockRollback.mockResolvedValue({
      status: "ok",
      data: {
        success: true,
        voided_sale_count: 1,
        voided_movement_count: 1,
        recheck_targets: [recheckTarget()],
      },
    });
    const queryClient = makeQueryClient();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    renderWithRouter(
      <QueryClientProvider client={queryClient}>
        <CsvImportRecordDetailPage importId={41} />
      </QueryClientProvider>,
    );

    await confirmRollback(user);

    expect(await screen.findByRole("region", { name: "数え直しが必要な商品" })).toBeInTheDocument();
    expect(mockRollback).toHaveBeenCalledTimes(1);
    expect(mockRollback).toHaveBeenCalledWith(41);
    expect(mockToast.success).toHaveBeenCalledWith("取込みを取り消しました", { duration: 8000 });
    await waitFor(() => {
      expectExactInvalidations(
        invalidateSpy.mock.calls,
        d052InvalidationOracle.csvImportRollback(),
      );
    });
    expect(screen.getByText("取込みを取り消しました")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "商品別CSV取込み（Z004）を開く" })).toHaveAttribute(
      "href",
      "/csv-import?tab=z004",
    );
    expect(await screen.findByText("この取込みは取消済みです")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "この取込みを取り消す" })).toBeNull();
  });

  it("REQ-401: 取消の応答を待つ間は詳細の画面から移れず、失敗すれば移れる", async () => {
    // 55 §55.7・§55.11、65 §65.5a（Matrix U13）: 止めは共有の hook にあり、詳細の入口にも効く。
    // memory history の止めは push と replace だけに効くので、link の押下で試す
    const user = userEvent.setup();
    mockGetCsvImportRecord.mockResolvedValue({ status: "ok", data: makeDetail() });
    let resolveRollback!: (value: Awaited<ReturnType<typeof commands.rollbackCsvImport>>) => void;
    mockRollback.mockReturnValueOnce(
      new Promise((resolve) => {
        resolveRollback = resolve;
      }),
    );
    const router = createRouter({
      routeTree,
      history: createMemoryHistory({
        initialEntries: ["/csv-import/records/41?returnTo=%2Fcsv-import"],
      }),
    });
    render(
      <QueryClientProvider client={makeQueryClient()}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    );

    await confirmRollback(user);
    expect(await screen.findByRole("button", { name: /取り消し中…/ })).toBeDisabled();
    await user.click(screen.getByRole("link", { name: "前の画面へ戻る" }));
    expect(router.state.location.pathname).toBe("/csv-import/records/41");
    expect(screen.getByRole("heading", { name: "CSV取込み #41" })).toBeInTheDocument();

    resolveRollback({
      status: "error",
      error: {
        kind: "internal",
        message: "合成の取消の失敗",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    await waitFor(() => {
      expect(mockToast.error).toHaveBeenCalledWith(
        "取り消しに失敗しました。もう一度お試しください",
      );
    });
    expect(await screen.findByRole("button", { name: "この取込みを取り消す" })).toBeEnabled();
    await user.click(screen.getByRole("link", { name: "前の画面へ戻る" }));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/csv-import");
    });
  });

  it("REQ-401: 取り消した後に別の取込みの詳細へ移ると、前の取込みの取消の結果を出さない", async () => {
    // 65 §65.5a（Matrix U15）: 同じ route の param が変わると、詳細の state を取込みID で作り直す
    const user = userEvent.setup();
    let rolledBack = false;
    mockGetCsvImportRecord.mockImplementation((id: number) =>
      Promise.resolve({
        status: "ok",
        data: makeDetail({
          id,
          ...(id === 41 && rolledBack ? { status: "rolled_back", movements: [] } : {}),
        }),
      }),
    );
    mockRollback.mockImplementation(() => {
      rolledBack = true;
      return Promise.resolve({
        status: "ok",
        data: {
          success: true,
          voided_sale_count: 1,
          voided_movement_count: 1,
          recheck_targets: [recheckTarget()],
        },
      });
    });
    const router = createRouter({
      routeTree,
      history: createMemoryHistory({ initialEntries: ["/csv-import/records/41"] }),
    });
    render(
      <QueryClientProvider client={makeQueryClient()}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    );

    await confirmRollback(user);
    expect(await screen.findByRole("region", { name: "数え直しが必要な商品" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "商品別CSV取込み（Z004）を開く" })).toBeInTheDocument();
    expect(await screen.findByText("この取込みは取消済みです")).toBeInTheDocument();

    await router.navigate({ to: "/csv-import/records/$importId", params: { importId: "42" } });

    expect(await screen.findByRole("heading", { name: "CSV取込み #42" })).toBeInTheDocument();
    expect(screen.queryByText("取込みを取り消しました")).toBeNull();
    expect(screen.queryByRole("region", { name: "数え直しが必要な商品" })).toBeNull();
    expect(screen.queryByRole("link", { name: "商品別CSV取込み（Z004）を開く" })).toBeNull();
    expect(screen.getByRole("button", { name: "この取込みを取り消す" })).toBeEnabled();
  });

  it("REQ-401: 取消の失敗は Toast で、同じ ID のまま再試行できる", async () => {
    const user = userEvent.setup();
    mockGetCsvImportRecord.mockResolvedValue({ status: "ok", data: makeDetail() });
    const failure: CmdError = {
      kind: "internal",
      message: "合成の取消の失敗",
      field: null,
      error_id: null,
      stocktake_recovery: null,
    };
    mockRollback.mockResolvedValueOnce({ status: "error", error: failure });
    renderWithClient(<CsvImportRecordDetailPage importId={41} />);

    await confirmRollback(user);

    await waitFor(() => {
      expect(mockToast.error).toHaveBeenCalledWith(
        "取り消しに失敗しました。もう一度お試しください",
      );
    });
    expect(screen.queryByText("この取込みは取消済みです")).toBeNull();
    expect(screen.queryByText("取込みを取り消しました")).toBeNull();

    mockRollback.mockResolvedValueOnce({
      status: "ok",
      data: { success: true, voided_sale_count: 1, voided_movement_count: 1, recheck_targets: [] },
    });
    await confirmRollback(user);
    await waitFor(() => {
      expect(mockRollback).toHaveBeenCalledTimes(2);
    });
    expect(mockRollback).toHaveBeenNthCalledWith(1, 41);
    expect(mockRollback).toHaveBeenNthCalledWith(2, 41);
  });
});
