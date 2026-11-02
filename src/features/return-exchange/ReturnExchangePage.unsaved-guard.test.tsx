import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { makeMockProductWithRelations } from "@/features/products/lib/test-fixtures";
import { commands } from "@/lib/bindings";
import { ReturnExchangePage } from "./ReturnExchangePage";

interface TestBlockerOptions {
  shouldBlockFn: () => boolean;
}

const mockUseBlocker = vi.hoisted(() =>
  vi.fn((options: TestBlockerOptions) =>
    options.shouldBlockFn()
      ? { status: "blocked" as const, reset: vi.fn(), proceed: vi.fn() }
      : { status: "idle" as const },
  ),
);

vi.mock("@tanstack/react-router", () => ({
  Link: ({ to, children }: { to: string; children: ReactNode }) => <a href={to}>{children}</a>,
  useBlocker: mockUseBlocker,
  useRouterState: ({ select }: { select: (state: { location: { href: string } }) => unknown }) =>
    select({ location: { href: "/inventory/return" } }),
}));
vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn(), dismiss: vi.fn() },
}));
vi.mock("@/lib/bindings", () => ({
  commands: {
    listReturns: vi.fn(),
    searchProducts: vi.fn(),
    createReturn: vi.fn(),
    saveReceiptImage: vi.fn(),
  },
}));

const mockListReturns = vi.mocked(commands.listReturns);
const mockSearchProducts = vi.mocked(commands.searchProducts);
const mockCreateReturn = vi.mocked(commands.createReturn);
const mockSaveReceiptImage = vi.mocked(commands.saveReceiptImage);
let blockerStatus: "idle" | "blocked" = "idle";

function shouldBlockCurrentNavigation(): boolean {
  const calls = mockUseBlocker.mock.calls;
  return calls[calls.length - 1][0].shouldBlockFn();
}

function renderPage() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  const page = () => (
    <QueryClientProvider client={queryClient}>
      <ReturnExchangePage />
    </QueryClientProvider>
  );
  const view = render(page());
  return {
    ...view,
    rerenderPage: () => {
      view.rerender(page());
    },
  };
}

beforeEach(() => {
  blockerStatus = "idle";
  mockUseBlocker.mockImplementation(() =>
    blockerStatus === "blocked"
      ? {
          status: "blocked" as const,
          reset: vi.fn(() => {
            blockerStatus = "idle";
          }),
          proceed: vi.fn(),
        }
      : { status: "idle" as const },
  );
  vi.stubGlobal("scrollTo", vi.fn());
  vi.spyOn(URL, "createObjectURL").mockReturnValue("blob:receipt-preview");
  mockListReturns.mockResolvedValue({
    status: "ok",
    data: { items: [], total_count: 0, page: 1, per_page: 10 },
  });
  mockSearchProducts.mockResolvedValue({
    status: "ok",
    data: {
      items: [makeMockProductWithRelations({ product_code: "RT-001", name: "返品商品" })],
      total_count: 1,
      page: 1,
      per_page: 10,
    },
  });
  mockSaveReceiptImage.mockResolvedValue({
    status: "ok",
    data: { relative_path: "images/receipts/receipt.png" },
  });
  mockCreateReturn.mockResolvedValue({
    status: "ok",
    data: {
      record_id: 1,
      created: true,
      idempotent_replay: false,
      stock_warnings: [],
      stock_applied_product_codes: ["RT-001"],
    },
  });
});

describe("ReturnExchangePage unsaved guard (UI-USW-D1/D3 / SPEC-UISN-2)", () => {
  it("UI-03 T12: 画像選択もdirtyに含み、保存結果では非blockになる", async () => {
    const user = userEvent.setup();
    const { rerenderPage } = renderPage();

    const search = await screen.findByLabelText("返品・交換商品検索");
    const note = screen.getByLabelText("備考");
    expect(shouldBlockCurrentNavigation()).toBe(false);

    fireEvent.drop(screen.getByTestId("file-picker-dropzone"), {
      dataTransfer: { files: [new File(["receipt"], "receipt.png", { type: "image/png" })] },
    });
    await screen.findByText("receipt.png");
    expect(shouldBlockCurrentNavigation()).toBe(true);
    await user.click(screen.getByRole("button", { name: "レシート画像を削除" }));
    expect(shouldBlockCurrentNavigation()).toBe(false);

    await user.type(note, "交換理由確認済み");
    expect(shouldBlockCurrentNavigation()).toBe(true);
    await user.clear(note);
    expect(shouldBlockCurrentNavigation()).toBe(false);

    await user.type(search, "RT-001{enter}");
    await screen.findByText("RT-001");
    expect(shouldBlockCurrentNavigation()).toBe(true);
    blockerStatus = "blocked";
    rerenderPage();
    expect(await screen.findByText("編集内容が保存されていません")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "編集を続ける" }));
    rerenderPage();
    // UI-03-D23: 既定の選択が無いので、保存の前にレジでの戻し方を選ぶ
    await user.click(
      screen.getByLabelText(
        "バーコードを読んでいない（部門キー・差額だけの戻し・レジを通していない）",
      ),
    );
    await user.click(screen.getByRole("button", { name: "返品・交換を保存" }));

    await waitFor(() => {
      expect(mockCreateReturn).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(screen.queryByText("編集内容が保存されていません")).not.toBeInTheDocument();
    });
    expect(shouldBlockCurrentNavigation()).toBe(false);
  });
});
