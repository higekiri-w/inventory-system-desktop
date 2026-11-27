import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { commands } from "@/lib/bindings";
import { makeMockProductWithRelations } from "../lib/test-fixtures";
import { usePriceRevisionList } from "./usePriceRevisionList";

vi.mock("@/lib/bindings", () => ({
  commands: {
    searchProducts: vi.fn(),
    listSuppliers: vi.fn(),
    listDepartments: vi.fn(),
    listPriceHistory: vi.fn(),
  },
}));

const mockSearchProducts = vi.mocked(commands.searchProducts);
const mockListSuppliers = vi.mocked(commands.listSuppliers);
const mockListDepartments = vi.mocked(commands.listDepartments);
const mockListPriceHistory = vi.mocked(commands.listPriceHistory);

function makeWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
  };
}

function arrangeProducts(items = [makeMockProductWithRelations({ product_code: "P-001" })]) {
  mockSearchProducts.mockResolvedValue({
    status: "ok",
    data: { items, total_count: items.length, page: 1, per_page: 50 },
  });
  mockListSuppliers.mockResolvedValue({ status: "ok", data: [] });
  mockListDepartments.mockResolvedValue({ status: "ok", data: [] });
  mockListPriceHistory.mockResolvedValue({ status: "ok", data: [] });
}

beforeEach(() => {
  vi.clearAllMocks();
  arrangeProducts();
});

describe("usePriceRevisionList UI-14 / REQ-105", () => {
  it("取引先指定時は supplier_id と include_unassigned=true を searchProducts に渡す", async () => {
    const { result } = renderHook(() => usePriceRevisionList({ search: { supplier: 7 } }), {
      wrapper: makeWrapper(),
    });
    await waitFor(() => {
      expect(result.current.productsQuery.isSuccess).toBe(true);
    });
    expect(mockSearchProducts).toHaveBeenCalledWith(
      expect.objectContaining({ supplier_id: 7, include_unassigned: true }),
    );
  });

  it("includeUnassigned=false のとき include_unassigned=false を渡す", async () => {
    const { result } = renderHook(
      () =>
        usePriceRevisionList({
          search: { supplier: 7, includeUnassigned: false },
        }),
      { wrapper: makeWrapper() },
    );
    await waitFor(() => {
      expect(result.current.productsQuery.isSuccess).toBe(true);
    });
    expect(mockSearchProducts).toHaveBeenCalledWith(
      expect.objectContaining({ supplier_id: 7, include_unassigned: false }),
    );
  });

  it("取引先未指定なら supplier_id null と include_unassigned false を渡し URL の includeUnassigned は無視する", async () => {
    const { result } = renderHook(
      () => usePriceRevisionList({ search: { includeUnassigned: true } }),
      { wrapper: makeWrapper() },
    );
    await waitFor(() => {
      expect(result.current.productsQuery.isSuccess).toBe(true);
    });
    expect(mockSearchProducts).toHaveBeenCalledWith(
      expect.objectContaining({ supplier_id: null, include_unassigned: false }),
    );
  });

  it("在庫数 0 の商品も一覧に含まれ query に在庫条件を載せない", async () => {
    arrangeProducts([makeMockProductWithRelations({ product_code: "ZERO", stock_quantity: 0 })]);
    const { result } = renderHook(() => usePriceRevisionList({ search: {} }), {
      wrapper: makeWrapper(),
    });
    await waitFor(() => {
      expect(result.current.rows).toHaveLength(1);
    });
    expect(result.current.rows[0]?.product.product_code).toBe("ZERO");
    expect(Object.keys(mockSearchProducts.mock.calls[0]?.[0] ?? {}).sort()).toEqual(
      [
        "department_id",
        "include_unassigned",
        "is_discontinued",
        "keyword",
        "page",
        "per_page",
        "sort_key",
        "sort_order",
        "supplier_id",
      ].sort(),
    );
  });

  it("page 内の各行について listPriceHistory(code, 1) を呼び changed_at を行に結び付ける", async () => {
    arrangeProducts([
      makeMockProductWithRelations({ product_code: "P-001" }),
      makeMockProductWithRelations({ product_code: "P-002" }),
      makeMockProductWithRelations({ product_code: "P-003" }),
    ]);
    const changedAtByCode: Record<string, string> = {
      "P-001": "2026-08-23T09:00:00",
      "P-002": "2026-08-22T10:00:00",
      "P-003": "2026-08-21T11:00:00",
    };
    mockListPriceHistory.mockImplementation((code) =>
      Promise.resolve({
        status: "ok",
        data: [
          {
            id: 1,
            old_selling_price: 100,
            new_selling_price: 110,
            old_cost_price_centi: 50,
            new_cost_price_centi: 55,
            changed_at: changedAtByCode[code] ?? "unexpected",
          },
        ],
      }),
    );

    const { result } = renderHook(() => usePriceRevisionList({ search: {} }), {
      wrapper: makeWrapper(),
    });
    await waitFor(() => {
      expect(result.current.rows.map((row) => row.latestChangedAt)).toEqual([
        "2026-08-23T09:00:00",
        "2026-08-22T10:00:00",
        "2026-08-21T11:00:00",
      ]);
    });
    expect(mockListPriceHistory.mock.calls).toEqual([
      ["P-001", 1],
      ["P-002", 1],
      ["P-003", 1],
    ]);
  });
});
