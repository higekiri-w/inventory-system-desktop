// REQ-203 / UI-04-D16 / SPEC-SUGGEST-D7,D10: ProductAddSuggest 配線 W2/W8/W10。

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { makeMockProductWithRelations } from "@/features/products/lib/test-fixtures";
import { commands } from "@/lib/bindings";
import { ManualSalePage } from "./ManualSalePage";

vi.mock("@/hooks/useUnsavedChangesWarning", () => ({
  useUnsavedChangesWarning: () => ({
    isBlocked: false,
    continueEditing: vi.fn(),
    discardAndProceed: vi.fn(),
  }),
}));
vi.mock("@tanstack/react-router", () => ({
  Link: ({ to, children }: { to: string; children: ReactNode }) => <a href={to}>{children}</a>,
  useNavigate: () => vi.fn(),
  useRouterState: ({ select }: { select: (state: { location: { href: string } }) => unknown }) =>
    select({ location: { href: "/inventory/manual-sale" } }),
}));
vi.mock("sonner", () => ({ toast: { success: vi.fn(), error: vi.fn(), dismiss: vi.fn() } }));
vi.mock("@/lib/bindings", () => ({
  commands: {
    searchProducts: vi.fn(),
    createManualSale: vi.fn(),
    listInventoryRecords: vi.fn(),
  },
}));

const mockSearchProducts = vi.mocked(commands.searchProducts);
const mockCreateManualSale = vi.mocked(commands.createManualSale);
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

function renderPage() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={client}>
      <ManualSalePage />
    </QueryClientProvider>,
  );
}

function result(items: ReturnType<typeof makeMockProductWithRelations>[], perPage: number) {
  return {
    status: "ok" as const,
    data: { items, total_count: items.length, page: 1, per_page: perPage },
  };
}

beforeEach(() => {
  vi.mocked(commands.listInventoryRecords).mockResolvedValue({
    status: "ok",
    data: { items: [], total_count: 0, page: 1, per_page: 5 },
  });
  mockSearchProducts.mockReset();
  mockCreateManualSale.mockReset();
  mockInvalidateAndClose.mockReset();
});

describe("ManualSalePage ProductAddSuggest (UI-04-D16)", () => {
  it("W2: live候補確定を既存addProductへ委譲して手動販売明細を追加する", async () => {
    mockSearchProducts.mockResolvedValue(
      result(
        [
          makeMockProductWithRelations({ product_code: "MS-L1", name: "販売候補A" }),
          makeMockProductWithRelations({ product_code: "MS-L2", name: "販売候補B" }),
        ],
        5,
      ),
    );
    renderPage();
    fireEvent.change(await screen.findByLabelText("手動販売商品検索"), {
      target: { value: "販売候補" },
    });
    const listbox = await screen.findByRole("listbox");
    fireEvent.click(within(listbox).getByRole("option", { name: /MS-L1.*販売候補A/ }));
    expect(await screen.findByText("販売候補A")).toBeInTheDocument();
    expect(screen.getByLabelText("MS-L1 の数量")).toHaveValue(1);
  });

  it("W8: activeなしEnterは既存commit検索を実行する", async () => {
    const user = userEvent.setup();
    mockSearchProducts.mockResolvedValue(
      result([makeMockProductWithRelations({ product_code: "MS-E1", name: "Enter販売商品" })], 10),
    );
    renderPage();
    await user.type(await screen.findByLabelText("手動販売商品検索"), "MS-E1{Enter}");
    expect(await screen.findByText("Enter販売商品")).toBeInTheDocument();
    expect(mockSearchProducts).toHaveBeenCalledWith(
      expect.objectContaining({ keyword: "MS-E1", per_page: 10 }),
    );
  });

  it("W10: 手動販売保存eventで表示中候補を同期closeする", async () => {
    const user = userEvent.setup();
    mockSearchProducts.mockResolvedValueOnce(
      result([makeMockProductWithRelations({ product_code: "MS-S1", name: "保存対象" })], 10),
    );
    renderPage();
    await user.type(await screen.findByLabelText("手動販売商品検索"), "MS-S1{Enter}");
    await screen.findByText("保存対象");
    mockSearchProducts.mockResolvedValueOnce(
      result(
        [
          makeMockProductWithRelations({ product_code: "MS-P1", name: "保存前候補A" }),
          makeMockProductWithRelations({ product_code: "MS-P2", name: "保存前候補B" }),
        ],
        5,
      ),
    );
    fireEvent.change(screen.getByLabelText("手動販売商品検索"), { target: { value: "保存前" } });
    await screen.findByRole("listbox");
    mockCreateManualSale.mockReturnValue(
      new Promise(() => {
        // 保存中 lock を観測するため、意図的に未解決のままにする。
      }),
    );
    fireEvent.click(screen.getByRole("button", { name: "手動販売を保存" }));
    expect(mockInvalidateAndClose).toHaveBeenCalledOnce();
    await waitFor(() => {
      expect(mockCreateManualSale).toHaveBeenCalledOnce();
    });
    expect(mockInvalidateAndClose.mock.invocationCallOrder[0]).toBeLessThan(
      mockCreateManualSale.mock.invocationCallOrder[0],
    );
    await waitFor(() => {
      expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
    });
  });

  it("W14: compositionendの正規化済み引数で既存検索を実行する", async () => {
    mockSearchProducts.mockResolvedValue(
      result([makeMockProductWithRelations({ product_code: "MS-C1", name: "合成入力商品" })], 10),
    );
    renderPage();

    fireEvent.compositionEnd(await screen.findByLabelText("手動販売商品検索"), {
      target: { value: "２３４５６" },
    });

    await waitFor(() => {
      expect(mockSearchProducts).toHaveBeenCalledWith(
        expect.objectContaining({ keyword: "23456", per_page: 10 }),
      );
    });
    expect(mockSearchProducts).not.toHaveBeenCalledWith(
      expect.objectContaining({ keyword: "２３４５６", per_page: 10 }),
    );
  });
  it("UI-04-D19 W16: 保存eventでlockした後のcompositionendは検索を始めない", async () => {
    const user = userEvent.setup();
    mockSearchProducts.mockResolvedValueOnce(
      result([makeMockProductWithRelations({ product_code: "MS-K1", name: "保存対象" })], 10),
    );
    renderPage();
    await user.type(await screen.findByLabelText("手動販売商品検索"), "MS-K1{Enter}");
    await screen.findByText("保存対象");

    mockCreateManualSale.mockReturnValue(
      new Promise(() => {
        // lock 中の compositionend を観測するため、意図的に未解決のままにする。
      }),
    );
    fireEvent.click(screen.getByRole("button", { name: "手動販売を保存" }));
    await waitFor(() => {
      expect(mockCreateManualSale).toHaveBeenCalledOnce();
    });
    mockSearchProducts.mockClear();

    const lockedInput = screen.getByLabelText("手動販売商品検索");
    fireEvent.compositionEnd(lockedInput, {
      target: { value: "５６７８９" },
    });

    expect(lockedInput).toHaveValue("５６７８９");
    expect(mockSearchProducts).not.toHaveBeenCalled();
  });
});
