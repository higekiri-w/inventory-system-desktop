// UI-00 / UI-00-D11: Home の実 query orchestration contract。

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { commands } from "@/lib/bindings";
import type {
  CmdError,
  CsvImport,
  DailySalesReport,
  PaginatedResult,
  ProductResponse,
  ProductWithRelations,
} from "@/lib/bindings";
import { useHomeSummary } from "./useHomeSummary";

vi.mock("@/lib/bindings", () => ({
  commands: {
    getDailySales: vi.fn(),
    listLowStock: vi.fn(),
    listPluDirty: vi.fn(),
    listCsvImports: vi.fn(),
  },
}));

const mockGetDailySales = vi.mocked(commands.getDailySales);
const mockListLowStock = vi.mocked(commands.listLowStock);
const mockListPluDirty = vi.mocked(commands.listPluDirty);
const mockListCsvImports = vi.mocked(commands.listCsvImports);

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

function makePluDirty(productCode: string): ProductResponse {
  return {
    product_code: productCode,
    jan_code: null,
    name: `synthetic-${productCode}`,
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
    file_hash: "a".repeat(64),
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

function localYesterday(): string {
  const date = new Date();
  date.setDate(date.getDate() - 1);
  return date.toLocaleDateString("sv-SE");
}

function makeHarness() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  return { queryClient, wrapper };
}

function installSuccessfulCommands(settlementDate = "2000-01-01") {
  mockGetDailySales.mockResolvedValue({ status: "ok", data: makeDailySales() });
  mockListLowStock.mockResolvedValue({
    status: "ok",
    data: [makeProduct("OUT", 0), makeProduct("NEG", -1), makeProduct("LOW", 3)],
  });
  mockListPluDirty.mockResolvedValue({
    status: "ok",
    data: [makePluDirty("PLU-1"), makePluDirty("PLU-2")],
  });
  mockListCsvImports.mockResolvedValue({
    status: "ok",
    data: csvPage([makeCsvImport(settlementDate)]),
  });
}

beforeEach(() => {
  vi.clearAllMocks();
  installSuccessfulCommands();
});

afterEach(() => {
  vi.useRealTimers();
});

describe("useHomeSummary UI-00 orchestration", () => {
  it("calls four commands with exact arguments and stores distinct literal query keys", async () => {
    const yesterday = localYesterday();
    const { queryClient, wrapper } = makeHarness();
    const { result } = renderHook(() => useHomeSummary(), { wrapper });

    await waitFor(() => {
      expect(result.current.sales.isSuccess).toBe(true);
      expect(result.current.lowStock.isSuccess).toBe(true);
      expect(result.current.pluDirty.isSuccess).toBe(true);
      expect(result.current.csvImports.isSuccess).toBe(true);
    });

    expect(mockGetDailySales).toHaveBeenCalledWith(yesterday);
    expect(mockListLowStock).toHaveBeenCalledWith(false);
    expect(mockListPluDirty).toHaveBeenCalledWith();
    expect(mockListCsvImports).toHaveBeenCalledWith(1, 1);
    expect(queryClient.getQueryData(["daily-sales", "detail", { date: yesterday }])).toEqual(
      makeDailySales(),
    );
    expect(
      queryClient.getQueryData(["products", "low-stock", { includeDiscontinued: false }]),
    ).toHaveLength(3);
    expect(queryClient.getQueryData(["plu-dirty"])).toHaveLength(2);
    expect(queryClient.getQueryData(["csv-imports", "list", { page: 1, perPage: 1 }])).toEqual(
      csvPage([makeCsvImport("2000-01-01")]),
    );
  });

  it("derives stock, PLU, settlement, and strict stale-import values from command data", async () => {
    const { wrapper } = makeHarness();
    const { result } = renderHook(() => useHomeSummary(), { wrapper });

    await waitFor(() => {
      expect(result.current.csvImports.isSuccess).toBe(true);
    });

    expect(result.current.derived).toEqual({
      yesterdayLabel: localYesterday(),
      outOfStockCount: 2,
      lowStockCount: 1,
      pluDirtyCount: 2,
      lastImportSettlementDate: "2000-01-01",
      needsImportWarning: true,
    });
  });

  it.each([
    ["no import", [] as CsvImport[]],
    ["same settlement date", [makeCsvImport(localYesterday())]],
  ])("does not warn for %s", async (_caseName, imports) => {
    mockListCsvImports.mockResolvedValue({ status: "ok", data: csvPage(imports) });
    const { wrapper } = makeHarness();
    const { result } = renderHook(() => useHomeSummary(), { wrapper });

    await waitFor(() => {
      expect(result.current.csvImports.isSuccess).toBe(true);
    });

    expect(result.current.derived.needsImportWarning).toBe(false);
  });

  it("keeps three successful queries when one command returns an error", async () => {
    mockListLowStock.mockResolvedValue({ status: "error", error: syntheticError });
    const { wrapper } = makeHarness();
    const { result } = renderHook(() => useHomeSummary(), { wrapper });

    await waitFor(() => {
      expect(result.current.lowStock.isError).toBe(true);
      expect(result.current.sales.isSuccess).toBe(true);
      expect(result.current.pluDirty.isSuccess).toBe(true);
      expect(result.current.csvImports.isSuccess).toBe(true);
    });

    expect(mockGetDailySales).toHaveBeenCalledTimes(1);
    expect(mockListLowStock).toHaveBeenCalledTimes(1);
    expect(mockListPluDirty).toHaveBeenCalledTimes(1);
    expect(mockListCsvImports).toHaveBeenCalledTimes(1);
    expect(result.current.sales.data).toEqual(makeDailySales());
    expect(result.current.pluDirty.data).toHaveLength(2);
    expect(result.current.csvImports.data?.items[0]?.settlement_date).toBe("2000-01-01");
  });

  it("refetches daily sales under the new query key after a visible day rollover", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    vi.setSystemTime(new Date(2026, 6, 29, 23, 59));
    const visibility = vi.spyOn(document, "visibilityState", "get").mockReturnValue("visible");
    const { queryClient, wrapper } = makeHarness();
    renderHook(() => useHomeSummary(), { wrapper });
    await waitFor(() => {
      expect(mockGetDailySales).toHaveBeenCalledWith("2026-07-28");
    });

    vi.setSystemTime(new Date(2026, 6, 30, 0, 1));
    act(() => {
      document.dispatchEvent(new Event("visibilitychange"));
    });

    await waitFor(() => {
      expect(mockGetDailySales).toHaveBeenCalledWith("2026-07-29");
    });
    expect(queryClient.getQueryData(["daily-sales", "detail", { date: "2026-07-29" }])).toEqual(
      makeDailySales(),
    );
    visibility.mockRestore();
  });
});
