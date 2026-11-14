// UI-00 / UI-00-D11: HomePage から実 useHomeSummary までのmain wiring contract。

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { toast } from "sonner";

import { commands } from "@/lib/bindings";
import { scrollPageToTop } from "@/lib/page-scroll";
import { consumeRestoreSuccessPending } from "@/lib/restore-success-notification";
import type {
  CmdError,
  CsvImport,
  DailySalesReport,
  OffsiteBackupStatus,
  PaginatedResult,
  ProductResponse,
  ProductWithRelations,
} from "@/lib/bindings";
import {
  useAutoBackupCheck,
  resetAutoBackupCheckForTests,
} from "@/features/backup-restore/useAutoBackupCheck";
import { renderWithRouter } from "@/test/render-with-router";
import { HomePage } from "./HomePage";

vi.mock("./components/QuickActionGrid", () => ({ QuickActionGrid: () => null }));
vi.mock("./components/InventoryActionGrid", () => ({ InventoryActionGrid: () => null }));
vi.mock("./components/MiscActionRow", () => ({ MiscActionRow: () => null }));
vi.mock("./components/PluNotificationBar", () => ({ PluNotificationBar: () => null }));

vi.mock("sonner", () => ({
  toast: {
    error: vi.fn(),
  },
}));

vi.mock("@/lib/bindings", () => ({
  commands: {
    getDailySales: vi.fn(),
    listLowStock: vi.fn(),
    listPluDirty: vi.fn(),
    listCsvImports: vi.fn(),
    getOffsiteBackupStatus: vi.fn(),
    checkAutoBackup: vi.fn(),
    checkOffsiteBackup: vi.fn(),
  },
}));

vi.mock("@/lib/page-scroll", () => ({ scrollPageToTop: vi.fn() }));
vi.mock("@/lib/restore-success-notification", () => ({
  consumeRestoreSuccessPending: vi.fn(),
}));

const mockGetDailySales = vi.mocked(commands.getDailySales);
const mockListLowStock = vi.mocked(commands.listLowStock);
const mockListPluDirty = vi.mocked(commands.listPluDirty);
const mockListCsvImports = vi.mocked(commands.listCsvImports);
const mockGetOffsiteStatus = vi.mocked(commands.getOffsiteBackupStatus);
const mockCheckAutoBackup = vi.mocked(commands.checkAutoBackup);
const mockCheckOffsite = vi.mocked(commands.checkOffsiteBackup);
const mockToastError = vi.mocked(toast.error);
const mockScrollPageToTop = vi.mocked(scrollPageToTop);
const mockConsumeRestoreSuccessPending = vi.mocked(consumeRestoreSuccessPending);

const syntheticError: CmdError = {
  kind: "internal",
  message: "synthetic query failure",
  field: null,
  error_id: "E-SYNTHETIC",
  stocktake_recovery: null,
};

function makeDailySales(): DailySalesReport {
  return {
    date: "2026-07-28",
    items: [],
    department_subtotals: [],
    grand_total: { count_points: 4, length_cm: 0, amount: 12000 },
    official_daily_report: null,
  };
}

function makeProduct(productCode: string, stockQuantity: number): ProductWithRelations {
  return {
    product_code: productCode,
    jan_code: null,
    name: `synthetic-${productCode}`,
    department_id: 1,
    supplier_id: null,
    selling_price: 100,
    cost_price_centi: 50,
    tax_rate: "10",
    maker_code: null,
    stock_quantity: stockQuantity,
    stock_unit: "pcs",
    is_discontinued: false,
    plu_dirty: false,
    plu_exported_at: null,
    plu_target: false,
    pos_stock_sync: true,
    created_at: "2026-01-01T00:00:00",
    updated_at: "2026-01-01T00:00:00",
    department_name: "synthetic-department",
    supplier_name: null,
    plu_memory_no: null,
    stock_recheck: false,
    stock_range_low: null,
    stock_range_high: null,
  };
}

function makePluDirty(): ProductResponse {
  return {
    product_code: "PLU-1",
    jan_code: null,
    name: "synthetic-PLU-1",
    department_id: 1,
    selling_price: 100,
    cost_price_centi: 50,
    stock_unit: "pcs",
    stock_quantity: 1,
    plu_dirty: true,
    plu_exported_at: null,
  };
}

function makeCsvImport(settlementDate: string): CsvImport {
  return {
    id: 1,
    filename: "synthetic.csv",
    settlement_date: settlementDate,
    file_hash: "b".repeat(64),
    total_items: 1,
    total_amount: 100,
    skipped_count: 0,
    status: "completed",
    imported_at: "2026-01-01T00:00:00",
  };
}

function csvPage(items: CsvImport[]): PaginatedResult<CsvImport> {
  return { items, total_count: items.length, page: 1, per_page: 1 };
}

function installSuccessfulCommands() {
  mockGetDailySales.mockResolvedValue({ status: "ok", data: makeDailySales() });
  mockListLowStock.mockResolvedValue({
    status: "ok",
    data: [makeProduct("OUT", 0), makeProduct("LOW", 3)],
  });
  mockListPluDirty.mockResolvedValue({ status: "ok", data: [makePluDirty()] });
  mockListCsvImports.mockResolvedValue({
    status: "ok",
    data: csvPage([makeCsvImport("2000-01-01")]),
  });
  mockGetOffsiteStatus.mockResolvedValue({
    status: "ok",
    data: makeOffsiteStatus({ prepared: false, stale: false }),
  });
  mockCheckAutoBackup.mockResolvedValue({ status: "ok", data: false });
  mockCheckOffsite.mockResolvedValue({ status: "ok", data: { kind: "not_prepared" } });
}

// 原因の行は prepared = true・stale = true を前提にする。日数・札は設計の例と違う値（5 日・控え 2）
function makeOffsiteStatus(overrides: Partial<OffsiteBackupStatus> = {}): OffsiteBackupStatus {
  return {
    prepared: true,
    last_success_at: "2026-10-28 09:01:00",
    last_success_label: "控え 2",
    days_since_last_success: 5,
    stale: true,
    attached: [],
    last_failure_kind: null,
    local_backup_old: false,
    ...overrides,
  };
}

function renderPage({ withAutoCheck = false } = {}) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  // ホームの知らせの link（Link）に router context が要る
  return renderWithRouter(
    <QueryClientProvider client={queryClient}>
      {withAutoCheck ? <HomeWithAutoCheck /> : <HomePage />}
    </QueryClientProvider>,
  );
}

// UI-11b-D16: 共通レイアウトの確認 hook を実物で mount する（status の invalidate → 再取得）
function HomeWithAutoCheck() {
  useAutoBackupCheck();
  return <HomePage />;
}

afterEach(() => {
  vi.useRealTimers();
});

beforeEach(() => {
  vi.clearAllMocks();
  resetAutoBackupCheckForTests();
  mockConsumeRestoreSuccessPending.mockReturnValue(false);
  installSuccessfulCommands();
});

describe("HomePage UI-00 orchestration wiring", () => {
  it("UI-11b-D12 Matrix T4: 復元成功 flag ありの mount でページ先頭へ one-shot scroll する", async () => {
    mockConsumeRestoreSuccessPending.mockReturnValue(true);

    renderPage();

    expect(await screen.findByText("バックアップから復元しました")).toBeInTheDocument();
    expect(mockScrollPageToTop).toHaveBeenCalledTimes(1);
  });

  it("UI-11b-D12 Matrix T5: 復元成功 flag なしの通常 mount では scroll しない", async () => {
    renderPage();

    expect(await screen.findByText(/昨日の売上/)).toBeInTheDocument();
    expect(mockScrollPageToTop).not.toHaveBeenCalled();
  });

  it("renders the stale-import warning and healthy summary from real query results", async () => {
    renderPage();

    expect(await screen.findByText("前日分が未取込みです")).toBeInTheDocument();
    // SC21: 既存のdestructive警告へ非色シグナルを追加する。
    const missingImport = screen.getByText("前日分が未取込みです").closest('[data-slot="alert"]');
    expect(missingImport).toHaveAttribute("data-variant", "destructive");
    expect(missingImport?.querySelector('svg[aria-hidden="true"]')).toBeInTheDocument();
    expect(screen.getByText("最後の取込み精算日: 2000-01-01")).toBeInTheDocument();
    expect(screen.getByText(/昨日の売上/)).toBeInTheDocument();
    expect(screen.getAllByText("1 件")).toHaveLength(2);
    expect(screen.getByText(/12[,，]?000/)).toBeInTheDocument();
  });

  it("reports a PLU failure without hiding healthy summary results", async () => {
    mockListPluDirty.mockResolvedValue({ status: "error", error: syntheticError });
    renderPage();

    await waitFor(() => {
      expect(mockToastError).toHaveBeenCalledWith("PLU 通知の取得に失敗しました", {
        id: "plu-dirty-error",
      });
    });
    expect(mockToastError).not.toHaveBeenCalledWith(
      "取込み履歴の取得に失敗しました",
      expect.anything(),
    );
    expect(await screen.findByText(/昨日の売上/)).toBeInTheDocument();
    expect(screen.getByText("在庫切れ")).toBeInTheDocument();
  });

  it("reports an import-history failure independently and keeps healthy summaries", async () => {
    mockListCsvImports.mockResolvedValue({ status: "error", error: syntheticError });
    renderPage();

    await waitFor(() => {
      expect(mockToastError).toHaveBeenCalledWith("取込み履歴の取得に失敗しました", {
        id: "csv-imports-error",
      });
    });
    expect(mockToastError).not.toHaveBeenCalledWith(
      "PLU 通知の取得に失敗しました",
      expect.anything(),
    );
    expect(screen.queryByText("前日分が未取込みです")).not.toBeInTheDocument();
    expect(await screen.findByText(/昨日の売上/)).toBeInTheDocument();
    expect(screen.getByText("在庫少")).toBeInTheDocument();
  });
});

describe("PC の外の控えの知らせ (REQ-901 / UI-00-D12)", () => {
  function stale(overrides: Partial<OffsiteBackupStatus>) {
    mockGetOffsiteStatus.mockResolvedValue({ status: "ok", data: makeOffsiteStatus(overrides) });
  }

  it("REQ-901 / UI-00-D12: 用意の前・やめた後・stale でないときは知らせない", async () => {
    for (const overrides of [
      { prepared: false, stale: false },
      { stale: false },
    ] as Partial<OffsiteBackupStatus>[]) {
      stale(overrides);
      const view = renderPage();
      expect(await screen.findByText(/昨日の売上/)).toBeInTheDocument();
      await waitFor(() => {
        expect(mockGetOffsiteStatus).toHaveBeenCalled();
      });
      expect(screen.queryByText(/PC の外の控え/)).not.toBeInTheDocument();
      view.unmount();
    }
  });

  it("REQ-901 / UI-00-D12: stale なら日数の文（status の日数）と「バックアップ画面へ」の link", async () => {
    stale({});
    renderPage();
    const text = await screen.findByText(
      "PC の外の控えが 5 日写せていません。USB メモリが差してあるか確かめてください。",
    );
    expect(text.closest('[data-slot="alert"]')).toHaveAttribute("data-variant", "warning");
    expect(screen.getByRole("link", { name: "バックアップ画面へ" })).toHaveAttribute(
      "href",
      "/settings/backup",
    );
  });

  it.each([
    [
      "写せていない",
      { last_success_at: null, last_success_label: null, days_since_last_success: null },
      "PC の外の控えがまだ写せていません。USB メモリが差してあるか確かめてください。",
    ],
    [
      "記録を読めない",
      { last_failure_kind: "state_unreadable" },
      "PC の外の控えの記録を読めません。USB メモリを差してバックアップ画面の「記録を作り直す」を押すか、使わないならバックアップ画面の「PC の外の控えをやめる」を押してください。",
    ],
    [
      "PC の中の backup が古い",
      { local_backup_old: true },
      "PC の外の控えが 5 日前のままです。PC の中の新しいバックアップが作られていません。バックアップ画面の「今すぐバックアップを作成」を押すか、自動バックアップと保存先の設定を確かめてください。",
    ],
    [
      "写す元を確かめられない",
      { last_failure_kind: "source_unverified" },
      "PC の中の最新のバックアップを確かめられないため、PC の外へ写していません。バックアップ画面の「今すぐバックアップを作成」を押してください。",
    ],
    [
      "空きなし",
      { last_failure_kind: "storage_full" },
      "PC の外の控えが 5 日写せていません。USB メモリの空きが足りません",
    ],
    [
      "保存先が同じ folder",
      {
        last_failure_kind: "backup_dir_is_medium_folder",
        attached: [{ label: "控え 2", drive_root: "E:\\", newest_copy: null }],
      },
      "PC の外の控えが 5 日写せていません。PC の中の保存先が、USB メモリの InventoryBackup フォルダと同じです。バックアップ画面の「保存先を選ぶ」で PC の中の別のフォルダを選び直してください",
    ],
    [
      "io・差してない",
      { last_failure_kind: "io" },
      "PC の外の控えが 5 日写せていません。USB メモリが差してあるか確かめてください。",
    ],
    [
      "io・差してある",
      {
        last_failure_kind: "io",
        attached: [{ label: "控え 2", drive_root: "E:\\", newest_copy: null }],
      },
      "PC の外の控えが 5 日写せていません。USB メモリは差してありますが、写せません。いったん抜いて差し直し、直らなければ別の USB メモリをバックアップ画面の「この USB メモリを控えの保存先にする」で用意してください。",
    ],
  ] as [string, Partial<OffsiteBackupStatus>, string][])(
    "REQ-901 / UI-00-D12: %s の原因の文",
    async (_label, overrides, text) => {
      stale(overrides);
      renderPage();
      expect(await screen.findByText(text)).toBeInTheDocument();
    },
  );

  it("REQ-901 / UI-00-D12: status の取得失敗は知らせなしで toast", async () => {
    mockGetOffsiteStatus.mockResolvedValue({ status: "error", error: syntheticError });
    renderPage();
    await waitFor(() => {
      expect(mockToastError).toHaveBeenCalledWith("PC の外の控えの状態を取得できませんでした", {
        id: "offsite-status-error",
      });
    });
    expect(screen.queryByRole("link", { name: "バックアップ画面へ" })).not.toBeInTheDocument();
    expect(await screen.findByText(/昨日の売上/)).toBeInTheDocument();
  });

  it("REQ-901 / UI-00-D12: 取得が成功の後に失敗へ変わったら、前の status の知らせを出し続けない", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    stale({});
    const offsite = deferredOffsite();
    mockCheckOffsite.mockReturnValueOnce(offsite.promise);
    renderPage({ withAutoCheck: true });
    const text = "PC の外の控えが 5 日写せていません。USB メモリが差してあるか確かめてください。";
    expect(await screen.findByText(text)).toBeInTheDocument();

    // 確認の結果の invalidate で取り直した status が失敗する（query は前の data を持ったまま isError）
    mockGetOffsiteStatus.mockResolvedValue({ status: "error", error: syntheticError });
    offsite.resolve({ status: "ok", data: { kind: "up_to_date" } });
    await waitFor(() => {
      expect(mockToastError).toHaveBeenCalledWith("PC の外の控えの状態を取得できませんでした", {
        id: "offsite-status-error",
      });
    });
    expect(screen.queryByText(text)).not.toBeInTheDocument();
    expect(screen.queryByRole("link", { name: "バックアップ画面へ" })).not.toBeInTheDocument();
  });

  it.each([
    [
      "5 日前",
      {},
      "PC の外の控えが 5 日写せていません。USB メモリが差してあるか確かめてください。",
    ],
    [
      "未コピー",
      { last_success_at: null, last_success_label: null, days_since_last_success: null },
      "PC の外の控えがまだ写せていません。USB メモリが差してあるか確かめてください。",
    ],
  ] as [string, Partial<OffsiteBackupStatus>, string][])(
    "REQ-901 / UI-00-D12・UI-11b-D16: 新しい写し（copied）の後の再取得で知らせが消える（%s から）",
    async (_label, overrides, text) => {
      vi.useFakeTimers({ shouldAdvanceTime: true });
      stale(overrides);
      const offsite = deferredOffsite();
      mockCheckOffsite.mockReturnValueOnce(offsite.promise);
      renderPage({ withAutoCheck: true });
      expect(await screen.findByText(text)).toBeInTheDocument();

      mockGetOffsiteStatus.mockResolvedValue({
        status: "ok",
        data: makeOffsiteStatus({
          last_success_at: "2026-11-02 09:00:00",
          days_since_last_success: 0,
          stale: false,
        }),
      });
      offsite.resolve({
        status: "ok",
        data: {
          kind: "copied",
          file_name: "inventory_backup_20261102_080000.db",
          labels: ["控え 2"],
        },
      });
      await waitFor(() => {
        expect(screen.queryByText(text)).not.toBeInTheDocument();
      });
      expect(screen.queryByRole("link", { name: "バックアップ画面へ" })).not.toBeInTheDocument();
    },
  );

  it.each([
    ["up_to_date", { status: "ok", data: { kind: "up_to_date" } }],
    ["失敗", { status: "error", error: syntheticError }],
  ] as const)(
    "REQ-901 / UI-00-D12・UI-11b-D16: %s の後に status が stale のままなら知らせが残る",
    async (_label, result) => {
      vi.useFakeTimers({ shouldAdvanceTime: true });
      stale({});
      const offsite = deferredOffsite();
      mockCheckOffsite.mockReturnValueOnce(offsite.promise);
      renderPage({ withAutoCheck: true });
      const text = "PC の外の控えが 5 日写せていません。USB メモリが差してあるか確かめてください。";
      expect(await screen.findByText(text)).toBeInTheDocument();
      const callsBefore = mockGetOffsiteStatus.mock.calls.length;
      offsite.resolve(result as Awaited<ReturnType<typeof commands.checkOffsiteBackup>>);
      await waitFor(() => {
        expect(mockGetOffsiteStatus.mock.calls.length).toBeGreaterThan(callsBefore);
      });
      expect(screen.getByText(text)).toBeInTheDocument();
    },
  );
});

function deferredOffsite() {
  type Result = Awaited<ReturnType<typeof commands.checkOffsiteBackup>>;
  let resolve: (value: Result) => void = () => undefined;
  const promise = new Promise<Result>((settle) => {
    resolve = settle;
  });
  return {
    promise,
    resolve: (value: Result) => {
      resolve(value);
    },
  };
}
