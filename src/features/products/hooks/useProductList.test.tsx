// src/features/products/hooks/useProductList.test.tsx
//
// UI-01a-D1/D3/D7: 商品一覧 hook の command payload と部門候補 source。

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { commands } from "@/lib/bindings";
import { makeMockDepartment, makeMockProductWithRelations } from "../lib/test-fixtures";
import { useProductList } from "./useProductList";

vi.mock("@/lib/bindings", () => ({
  commands: {
    searchProducts: vi.fn(),
    listDepartments: vi.fn(),
  },
}));

const mockSearchProducts = vi.mocked(commands.searchProducts);
const mockListDepartments = vi.mocked(commands.listDepartments);

function makeWrapper() {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={qc}>{children}</QueryClientProvider>;
  };
}

beforeEach(() => {
  mockSearchProducts.mockReset();
  mockListDepartments.mockReset();
});

describe("useProductList (UI-01a)", () => {
  it("UI-01a-D1/D3: initial render searches active products with safe defaults", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-001" })],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });
    mockListDepartments.mockResolvedValue({ status: "ok", data: [makeMockDepartment()] });

    const { result } = renderHook(() => useProductList({ search: {} }), {
      wrapper: makeWrapper(),
    });

    await waitFor(() => {
      expect(result.current.productsQuery.isSuccess).toBe(true);
    });
    expect(mockSearchProducts).toHaveBeenCalledWith({
      keyword: null,
      department_id: null,
      is_discontinued: false,
      sort_key: "ProductCode",
      sort_order: "Asc",
      page: 1,
      per_page: 100,
    });
  });

  it("UI-01a-D7: department options come from listDepartments, not current search page", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          makeMockProductWithRelations({
            product_code: "P-001",
            department_id: 1,
            department_name: "検索結果部門",
          }),
        ],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });
    mockListDepartments.mockResolvedValue({
      status: "ok",
      data: [
        makeMockDepartment({ id: 1, name: "毛糸" }),
        makeMockDepartment({ id: 2, name: "布" }),
        makeMockDepartment({ id: 21, name: "その他" }),
      ],
    });

    const { result } = renderHook(() => useProductList({ search: {} }), {
      wrapper: makeWrapper(),
    });

    await waitFor(() => {
      expect(result.current.departmentOptions).toEqual([
        { id: 1, name: "毛糸" },
        { id: 2, name: "布" },
        { id: 21, name: "その他" },
      ]);
    });
    expect(mockListDepartments).toHaveBeenCalledTimes(1);
  });
});
