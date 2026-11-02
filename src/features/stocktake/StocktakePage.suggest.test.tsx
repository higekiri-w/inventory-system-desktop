// REQ-205 / UI-10-D2/D11/D12 / SPEC-SUGGEST-D8,D10: ProductAddSuggest 配線 W5/W7/W8/W12。

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { makeMockProductWithRelations } from "@/features/products/lib/test-fixtures";
import { commands, type StocktakeItemDetail } from "@/lib/bindings";
import { StocktakePage } from "./StocktakePage";

vi.mock("@/lib/bindings", () => ({
  commands: {
    listDepartments: vi.fn(),
    getActiveStocktake: vi.fn(),
    getLastCompletedStocktake: vi.fn(),
    startStocktake: vi.fn(),
    getStocktakeItems: vi.fn(),
    findStocktakeItem: vi.fn(),
    saveStockCount: vi.fn(),
    completeStocktake: vi.fn(),
    searchProducts: vi.fn(),
    listRecheckProducts: vi.fn(),
  },
}));

const mockSearchProducts = vi.mocked(commands.searchProducts);
const mockFindItem = vi.mocked(commands.findStocktakeItem);
const mockComplete = vi.mocked(commands.completeStocktake);
const mockInvalidateAndClose = vi.hoisted(() => vi.fn());

vi.mock("@/components/patterns/useProductAddSuggest", async (importOriginal) => {
  const actual =
    await importOriginal<typeof import("@/components/patterns/useProductAddSuggest")>();
  return {
    ...actual,
    useProductAddSuggest: (...args: Parameters<typeof actual.useProductAddSuggest>) => {
      const controller = actual.useProductAddSuggest(...args);
      return {
        ...controller,
        invalidateAndClose: () => {
          mockInvalidateAndClose();
          controller.invalidateAndClose();
        },
      };
    },
  };
});

function ok<T>(data: T) {
  return { status: "ok" as const, data };
}
function item(overrides: Partial<StocktakeItemDetail> = {}): StocktakeItemDetail {
  return {
    id: 501,
    stocktake_id: 77,
    product_code: "ST-001",
    name: "棚卸し商品",
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
    ...overrides,
  };
}
function renderPage() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={client}>
      <StocktakePage search={{}} onSearchChange={vi.fn()} />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  vi.mocked(commands.listDepartments).mockResolvedValue(ok([]));
  vi.mocked(commands.getActiveStocktake).mockResolvedValue(
    ok({
      id: 77,
      started_at: "2026-10-01T09:00:00",
      completed_at: null,
      status: "in_progress",
      total_cost: null,
    }),
  );
  vi.mocked(commands.getLastCompletedStocktake).mockResolvedValue(ok(null));
  vi.mocked(commands.getStocktakeItems).mockResolvedValue(
    ok({
      items: [item()],
      progress: { total_items: 1, counted_items: 0, uncounted_items: 1 },
      total_count: 1,
      page: 1,
      per_page: 200,
    }),
  );
  vi.mocked(commands.listRecheckProducts).mockResolvedValue(ok([]));
  vi.mocked(commands.saveStockCount).mockResolvedValue(
    ok({
      id: 1,
      count_kind: "default",
      stock: 10,
      stock_range_low: null,
      stock_range_high: null,
      recount_reasons: [],
      later_count_exists: false,
    }),
  );
  mockSearchProducts.mockReset();
  mockFindItem.mockReset();
  mockComplete.mockReset();
  mockInvalidateAndClose.mockReset();
});

describe("StocktakePage ProductAddSuggest (UI-10-D12)", () => {
  it("W5/W7: live候補確定をfind_stocktake_item経由で解決し数量欄へfocusする", async () => {
    mockSearchProducts.mockResolvedValue(
      ok({
        items: [
          makeMockProductWithRelations({ product_code: "ST-L1", name: "棚卸し候補A" }),
          makeMockProductWithRelations({ product_code: "ST-L2", name: "棚卸し候補B" }),
        ],
        total_count: 2,
        page: 1,
        per_page: 5,
      }),
    );
    mockFindItem.mockResolvedValue(ok(item({ product_code: "ST-L1", name: "棚卸し候補A" })));
    renderPage();
    fireEvent.change(await screen.findByLabelText("商品を検索・スキャン"), {
      target: { value: "棚卸し候補" },
    });
    const listbox = await screen.findByRole("listbox");
    fireEvent.click(within(listbox).getByRole("option", { name: /ST-L1.*棚卸し候補A/ }));

    await waitFor(() => {
      expect(mockFindItem).toHaveBeenCalledWith(77, "ST-L1");
    });
    await waitFor(() => {
      expect(screen.getByLabelText("実際の数")).toHaveFocus();
    });
  });

  it("W8: activeなしEnterは既存resolveItem経路を実行する", async () => {
    const user = userEvent.setup();
    mockFindItem.mockResolvedValue(ok(item({ product_code: "ST-E1", name: "Enter棚卸し商品" })));
    mockSearchProducts.mockResolvedValue(ok({ items: [], total_count: 0, page: 1, per_page: 5 }));
    renderPage();
    await user.type(await screen.findByLabelText("商品を検索・スキャン"), "ST-E1{Enter}");
    await waitFor(() => {
      expect(mockFindItem).toHaveBeenCalledWith(77, "ST-E1");
    });
    expect(await screen.findByText("Enter棚卸し商品")).toBeInTheDocument();
  });

  it("W12: 棚卸し確定eventで表示中候補を同期closeする", async () => {
    mockSearchProducts.mockResolvedValue(
      ok({
        items: [
          makeMockProductWithRelations({ product_code: "ST-P1", name: "確定前候補A" }),
          makeMockProductWithRelations({ product_code: "ST-P2", name: "確定前候補B" }),
        ],
        total_count: 2,
        page: 1,
        per_page: 5,
      }),
    );
    renderPage();
    fireEvent.change(await screen.findByLabelText("商品を検索・スキャン"), {
      target: { value: "確定前" },
    });
    await screen.findByRole("listbox");
    mockComplete.mockReturnValue(
      new Promise(() => {
        // 確定中 lock を観測するため、意図的に未解決のままにする。
      }),
    );
    fireEvent.click(screen.getByRole("button", { name: "棚卸しを確定する" }));
    fireEvent.click(await screen.findByRole("button", { name: "確定する" }));
    expect(mockInvalidateAndClose).toHaveBeenCalledOnce();
    await waitFor(() => {
      expect(mockComplete).toHaveBeenCalledOnce();
    });
    expect(mockInvalidateAndClose.mock.invocationCallOrder[0]).toBeLessThan(
      mockComplete.mock.invocationCallOrder[0],
    );
    await waitFor(() => {
      expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
    });
  });

  it("W17: compositionendの正規化済み引数を既存resolveItem経路へ渡す", async () => {
    mockFindItem.mockResolvedValue(ok(item({ product_code: "ST-C1", name: "合成入力棚卸し商品" })));
    renderPage();

    fireEvent.compositionEnd(await screen.findByLabelText("商品を検索・スキャン"), {
      target: { value: "６７８９０" },
    });

    await waitFor(() => {
      expect(mockFindItem).toHaveBeenCalledWith(77, "67890");
    });
    expect(mockFindItem).not.toHaveBeenCalledWith(77, "６７８９０");
  });
});
