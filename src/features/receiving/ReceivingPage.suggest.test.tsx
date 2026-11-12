// REQ-201 / UI-02-D14 / SPEC-SUGGEST-D7,D10: ProductAddSuggest 配線 W1/W8/W9。

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  makeMockProductWithRelations,
  makeMockSupplier,
} from "@/features/products/lib/test-fixtures";
import { commands } from "@/lib/bindings";
import { ReceivingPage } from "./ReceivingPage";

vi.mock("@/hooks/useUnsavedChangesWarning", () => ({
  useUnsavedChangesWarning: () => ({
    isBlocked: false,
    continueEditing: vi.fn(),
    discardAndProceed: vi.fn(),
  }),
}));

vi.mock("@tanstack/react-router", () => ({
  Link: ({ to, children }: { to: string; children: ReactNode }) => <a href={to}>{children}</a>,
  useRouterState: ({ select }: { select: (state: { location: { href: string } }) => unknown }) =>
    select({ location: { href: "/inventory/receiving" } }),
}));

vi.mock("sonner", () => ({ toast: { success: vi.fn(), error: vi.fn(), dismiss: vi.fn() } }));

vi.mock("@/lib/bindings", () => ({
  commands: {
    listSuppliers: vi.fn(),
    listReceivings: vi.fn(),
    searchProducts: vi.fn(),
    createReceiving: vi.fn(),
  },
}));

const mockSearchProducts = vi.mocked(commands.searchProducts);
const mockCreateReceiving = vi.mocked(commands.createReceiving);
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
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={queryClient}>
      <ReceivingPage />
    </QueryClientProvider>,
  );
}

function searchResult(items: ReturnType<typeof makeMockProductWithRelations>[], perPage: number) {
  return {
    status: "ok" as const,
    data: { items, total_count: items.length, page: 1, per_page: perPage },
  };
}

beforeEach(() => {
  vi.mocked(commands.listSuppliers).mockResolvedValue({
    status: "ok",
    data: [makeMockSupplier({ id: 1, name: "テスト取引先" })],
  });
  vi.mocked(commands.listReceivings).mockResolvedValue({
    status: "ok",
    data: { items: [], total_count: 0, page: 1, per_page: 10 },
  });
  mockSearchProducts.mockReset();
  mockCreateReceiving.mockReset();
  mockInvalidateAndClose.mockReset();
});

describe("ReceivingPage ProductAddSuggest (UI-02-D14)", () => {
  it("W1: live候補確定を既存addProductへ委譲して入庫明細を追加する", async () => {
    mockSearchProducts.mockResolvedValue(
      searchResult(
        [
          makeMockProductWithRelations({ product_code: "RC-L1", name: "入庫候補A" }),
          makeMockProductWithRelations({ product_code: "RC-L2", name: "入庫候補B" }),
        ],
        5,
      ),
    );
    renderPage();

    fireEvent.change(await screen.findByLabelText("入庫商品検索"), {
      target: { value: "入庫候補" },
    });
    const listbox = await screen.findByRole("listbox");
    fireEvent.click(within(listbox).getByRole("option", { name: /RC-L1.*入庫候補A/ }));

    expect(await screen.findByText("入庫候補A")).toBeInTheDocument();
    expect(screen.getByLabelText("RC-L1 の数量")).toHaveValue(1);
  });

  it("W8: activeなしEnterは既存commit検索を実行する", async () => {
    const user = userEvent.setup();
    mockSearchProducts.mockResolvedValue(
      searchResult(
        [makeMockProductWithRelations({ product_code: "RC-E1", name: "Enter入庫商品" })],
        10,
      ),
    );
    renderPage();

    await user.type(await screen.findByLabelText("入庫商品検索"), "RC-E1{Enter}");

    expect(await screen.findByText("Enter入庫商品")).toBeInTheDocument();
    expect(mockSearchProducts).toHaveBeenCalledWith(
      expect.objectContaining({ keyword: "RC-E1", per_page: 10 }),
    );
  });

  it("W9: 入庫保存eventで表示中候補を同期closeする", async () => {
    const user = userEvent.setup();
    mockSearchProducts.mockResolvedValueOnce(
      searchResult([makeMockProductWithRelations({ product_code: "RC-S1", name: "保存対象" })], 10),
    );
    renderPage();
    await user.type(await screen.findByLabelText("入庫商品検索"), "RC-S1{Enter}");
    await screen.findByText("保存対象");

    mockSearchProducts.mockResolvedValueOnce(
      searchResult(
        [
          makeMockProductWithRelations({ product_code: "RC-P1", name: "保存前候補A" }),
          makeMockProductWithRelations({ product_code: "RC-P2", name: "保存前候補B" }),
        ],
        5,
      ),
    );
    fireEvent.change(screen.getByLabelText("入庫商品検索"), { target: { value: "保存前" } });
    await screen.findByRole("listbox");
    mockCreateReceiving.mockReturnValue(
      new Promise(() => {
        // 保存中 lock を観測するため、意図的に未解決のままにする。
      }),
    );

    fireEvent.click(screen.getByRole("button", { name: "入庫を保存" }));

    expect(mockInvalidateAndClose).toHaveBeenCalledOnce();
    await waitFor(() => {
      expect(mockCreateReceiving).toHaveBeenCalledOnce();
    });
    expect(mockInvalidateAndClose.mock.invocationCallOrder[0]).toBeLessThan(
      mockCreateReceiving.mock.invocationCallOrder[0],
    );
    await waitFor(() => {
      expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
    });
  });

  it("W13: compositionendの正規化済み引数で既存検索を実行する", async () => {
    mockSearchProducts.mockResolvedValue(
      searchResult(
        [makeMockProductWithRelations({ product_code: "RC-C1", name: "合成入力商品" })],
        10,
      ),
    );
    renderPage();

    fireEvent.compositionEnd(await screen.findByLabelText("入庫商品検索"), {
      target: { value: "１２３４５" },
    });

    await waitFor(() => {
      expect(mockSearchProducts).toHaveBeenCalledWith(
        expect.objectContaining({ keyword: "12345", per_page: 10 }),
      );
    });
    expect(mockSearchProducts).not.toHaveBeenCalledWith(
      expect.objectContaining({ keyword: "１２３４５", per_page: 10 }),
    );
  });
  it("UI-02-D19 W16: 保存eventでlockした後のcompositionendは検索を始めない", async () => {
    const user = userEvent.setup();
    mockSearchProducts.mockResolvedValueOnce(
      searchResult([makeMockProductWithRelations({ product_code: "RC-K1", name: "保存対象" })], 10),
    );
    renderPage();
    await user.type(await screen.findByLabelText("入庫商品検索"), "RC-K1{Enter}");
    await screen.findByText("保存対象");

    mockCreateReceiving.mockReturnValue(
      new Promise(() => {
        // lock 中の compositionend を観測するため、意図的に未解決のままにする。
      }),
    );
    fireEvent.click(screen.getByRole("button", { name: "入庫を保存" }));
    await waitFor(() => {
      expect(mockCreateReceiving).toHaveBeenCalledOnce();
    });
    mockSearchProducts.mockClear();

    const lockedInput = screen.getByLabelText("入庫商品検索");
    fireEvent.compositionEnd(lockedInput, {
      target: { value: "５６７８９" },
    });

    expect(lockedInput).toHaveValue("５６７８９");
    expect(mockSearchProducts).not.toHaveBeenCalled();
  });
});
