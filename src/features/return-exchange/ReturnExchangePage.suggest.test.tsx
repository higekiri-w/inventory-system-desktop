// REQ-202 / UI-03-D21 / SPEC-SUGGEST-D7,D10: ProductAddSuggest 配線 W3/W8/W11。

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { makeMockProductWithRelations } from "@/features/products/lib/test-fixtures";
import { commands } from "@/lib/bindings";
import { ReturnExchangePage } from "./ReturnExchangePage";

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
    select({ location: { href: "/inventory/return" } }),
}));
vi.mock("sonner", () => ({ toast: { success: vi.fn(), error: vi.fn(), dismiss: vi.fn() } }));
vi.mock("@/lib/bindings", () => ({
  commands: {
    listReturns: vi.fn(),
    searchProducts: vi.fn(),
    createReturn: vi.fn(),
    saveReceiptImage: vi.fn(),
  },
}));

const mockSearchProducts = vi.mocked(commands.searchProducts);
const mockCreateReturn = vi.mocked(commands.createReturn);
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
      <ReturnExchangePage />
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
  vi.stubGlobal("scrollTo", vi.fn());
  vi.spyOn(URL, "createObjectURL").mockReturnValue("blob:test");
  vi.mocked(commands.listReturns).mockResolvedValue({
    status: "ok",
    data: { items: [], total_count: 0, page: 1, per_page: 10 },
  });
  mockSearchProducts.mockReset();
  mockCreateReturn.mockReset();
  mockInvalidateAndClose.mockReset();
});

describe("ReturnExchangePage ProductAddSuggest (UI-03-D21)", () => {
  it("W3: live候補確定をeffectiveAddDirection付き既存addProductへ委譲する", async () => {
    mockSearchProducts.mockResolvedValue(
      result(
        [
          makeMockProductWithRelations({ product_code: "RT-L1", name: "交換候補A" }),
          makeMockProductWithRelations({ product_code: "RT-L2", name: "交換候補B" }),
        ],
        5,
      ),
    );
    const user = userEvent.setup();
    renderPage();
    await screen.findByLabelText("返品・交換商品検索");
    await user.click(screen.getByLabelText("種別"));
    await user.click(await screen.findByRole("option", { name: "交換" }));
    await user.click(screen.getByLabelText("追加方向"));
    await user.click(await screen.findByRole("option", { name: "渡し" }));
    fireEvent.change(screen.getByLabelText("返品・交換商品検索"), {
      target: { value: "交換候補" },
    });
    const listbox = await screen.findByRole("listbox");
    fireEvent.click(within(listbox).getByRole("option", { name: /RT-L1.*交換候補A/ }));
    expect(await screen.findByText("交換候補A")).toBeInTheDocument();
    const directionTrigger = screen.getByLabelText("RT-L1 の方向");
    expect(directionTrigger).toHaveAttribute("data-slot", "select-trigger");
    expect(directionTrigger.tagName).toBe("BUTTON");
    expect(directionTrigger).toHaveTextContent("渡し（在庫-）");
  });

  it("W8: activeなしEnterは既存commit検索を実行する", async () => {
    const user = userEvent.setup();
    mockSearchProducts.mockResolvedValue(
      result([makeMockProductWithRelations({ product_code: "RT-E1", name: "Enter返品商品" })], 10),
    );
    renderPage();
    await user.type(await screen.findByLabelText("返品・交換商品検索"), "RT-E1{Enter}");
    expect(await screen.findByText("Enter返品商品")).toBeInTheDocument();
    expect(mockSearchProducts).toHaveBeenCalledWith(
      expect.objectContaining({ keyword: "RT-E1", per_page: 10 }),
    );
  });

  it("W11: 返品保存eventで表示中候補を同期closeする", async () => {
    const user = userEvent.setup();
    mockSearchProducts.mockResolvedValueOnce(
      result([makeMockProductWithRelations({ product_code: "RT-S1", name: "保存対象" })], 10),
    );
    renderPage();
    await user.type(await screen.findByLabelText("返品・交換商品検索"), "RT-S1{Enter}");
    await screen.findByText("保存対象");
    // UI-03-D23: 既定の選択が無いので、保存の前にレジでの戻し方を選ぶ
    await user.click(
      screen.getByLabelText(
        "バーコードを読んでいない（部門キー・差額だけの戻し・レジを通していない）",
      ),
    );
    mockSearchProducts.mockResolvedValueOnce(
      result(
        [
          makeMockProductWithRelations({ product_code: "RT-P1", name: "保存前候補A" }),
          makeMockProductWithRelations({ product_code: "RT-P2", name: "保存前候補B" }),
        ],
        5,
      ),
    );
    fireEvent.change(screen.getByLabelText("返品・交換商品検索"), { target: { value: "保存前" } });
    await screen.findByRole("listbox");
    mockCreateReturn.mockReturnValue(
      new Promise(() => {
        // 保存中 lock を観測するため、意図的に未解決のままにする。
      }),
    );
    fireEvent.click(screen.getByRole("button", { name: "返品・交換を保存" }));
    expect(mockInvalidateAndClose).toHaveBeenCalledOnce();
    await waitFor(() => {
      expect(mockCreateReturn).toHaveBeenCalledOnce();
    });
    expect(mockInvalidateAndClose.mock.invocationCallOrder[0]).toBeLessThan(
      mockCreateReturn.mock.invocationCallOrder[0],
    );
    await waitFor(() => {
      expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
    });
  });

  it("W15: compositionendの正規化済み引数で既存検索を実行する", async () => {
    mockSearchProducts.mockResolvedValue(
      result([makeMockProductWithRelations({ product_code: "RT-C1", name: "合成入力商品" })], 10),
    );
    renderPage();

    fireEvent.compositionEnd(await screen.findByLabelText("返品・交換商品検索"), {
      target: { value: "３４５６７" },
    });

    await waitFor(() => {
      expect(mockSearchProducts).toHaveBeenCalledWith(
        expect.objectContaining({ keyword: "34567", per_page: 10 }),
      );
    });
    expect(mockSearchProducts).not.toHaveBeenCalledWith(
      expect.objectContaining({ keyword: "３４５６７", per_page: 10 }),
    );
  });
});
