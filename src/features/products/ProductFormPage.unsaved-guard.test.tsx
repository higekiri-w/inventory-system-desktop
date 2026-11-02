import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { cloneElement, type ReactElement } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { commands } from "@/lib/bindings";
import { makeMockDepartment, makeMockProductWithRelations } from "./lib/test-fixtures";
import { ProductFormPage } from "./ProductFormPage";

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

vi.mock("@tanstack/react-router", () => ({ useBlocker: mockUseBlocker }));
vi.mock("@/lib/bindings", () => ({
  commands: {
    listDepartments: vi.fn(),
    listSuppliers: vi.fn(),
    getProduct: vi.fn(),
    createProduct: vi.fn(),
    updateProduct: vi.fn(),
    toggleDiscontinue: vi.fn(),
  },
}));
vi.mock("sonner", () => ({ toast: { success: vi.fn(), error: vi.fn(), dismiss: vi.fn() } }));

const mockListDepartments = vi.mocked(commands.listDepartments);
const mockListSuppliers = vi.mocked(commands.listSuppliers);
const mockGetProduct = vi.mocked(commands.getProduct);
const mockCreateProduct = vi.mocked(commands.createProduct);
const mockUpdateProduct = vi.mocked(commands.updateProduct);
let blockerStatus: "idle" | "blocked" = "idle";

function shouldBlockCurrentNavigation(): boolean {
  const calls = mockUseBlocker.mock.calls;
  return calls[calls.length - 1][0].shouldBlockFn();
}

function attemptNavigation() {
  if (shouldBlockCurrentNavigation()) blockerStatus = "blocked";
}

function renderWithClient(ui: ReactElement) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  const renderPage = () => (
    <QueryClientProvider client={queryClient}>{cloneElement(ui)}</QueryClientProvider>
  );
  const view = render(renderPage());
  return {
    ...view,
    rerenderPage: () => {
      view.rerender(renderPage());
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
  mockListDepartments.mockResolvedValue({
    status: "ok",
    data: [makeMockDepartment({ id: 1, code_prefix: "T" })],
  });
  mockListSuppliers.mockResolvedValue({ status: "ok", data: [] });
  mockCreateProduct.mockResolvedValue({
    status: "ok",
    data: { product_code: "T-0001", warnings: [] },
  });
  mockUpdateProduct.mockResolvedValue({ status: "ok", data: { warnings: [] } });
});

describe("ProductFormPage unsaved guard (UI-USW-D1/D3 / SPEC-UISN-2/3)", () => {
  it("UI-01b/UI-USW-D1 T9: 入力でdirtyになり、保存成功時はnavigate前にpristineへ戻る", async () => {
    const user = userEvent.setup();
    let blockedAtNavigate: boolean | undefined;
    const { rerenderPage } = renderWithClient(
      <ProductFormPage
        mode="create"
        onNavigateToList={() => {
          blockedAtNavigate = shouldBlockCurrentNavigation();
        }}
      />,
    );

    const name = await screen.findByLabelText(/^商品名/);
    expect(shouldBlockCurrentNavigation()).toBe(false);
    await user.type(name, "ガード確認商品");
    expect(shouldBlockCurrentNavigation()).toBe(true);

    await user.click(screen.getByLabelText(/^部門/));
    await user.click(await screen.findByRole("option", { name: "毛糸（独自コード可）" }));
    await user.clear(screen.getByLabelText(/^売価/));
    await user.type(screen.getByLabelText(/^売価/), "500");
    await user.clear(screen.getByLabelText(/^原価/));
    await user.type(screen.getByLabelText(/^原価/), "300");
    attemptNavigation();
    rerenderPage();
    expect(await screen.findByText("編集内容が保存されていません")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "編集を続ける" }));
    rerenderPage();
    await user.click(screen.getByRole("button", { name: "登録する" }));

    await waitFor(() => {
      expect(mockCreateProduct).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(screen.queryByText("編集内容が保存されていません")).not.toBeInTheDocument();
    });
    expect(blockedAtNavigate).toBe(false);
  });

  it("UI-01b T9: edit読込み値をbaselineにし、変更時だけblockする", async () => {
    const user = userEvent.setup();
    let blockedAtNavigate: boolean | undefined;
    mockGetProduct.mockResolvedValue({
      status: "ok",
      data: makeMockProductWithRelations({ product_code: "P-001", name: "変更前" }),
    });
    const { rerenderPage } = renderWithClient(
      <ProductFormPage
        mode="edit"
        productCode="P-001"
        onNavigateToList={() => {
          blockedAtNavigate = shouldBlockCurrentNavigation();
        }}
      />,
    );

    const name = await screen.findByLabelText(/^商品名/);
    expect(name).toHaveValue("変更前");
    expect(shouldBlockCurrentNavigation()).toBe(false);

    await user.clear(name);
    await user.type(name, "変更後");
    expect(shouldBlockCurrentNavigation()).toBe(true);
    attemptNavigation();
    rerenderPage();
    expect(await screen.findByText("編集内容が保存されていません")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "編集を続ける" }));
    rerenderPage();
    await user.click(screen.getByRole("button", { name: "保存する" }));

    await waitFor(() => {
      expect(mockUpdateProduct).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(screen.queryByText("編集内容が保存されていません")).not.toBeInTheDocument();
    });
    expect(blockedAtNavigate).toBe(false);
  });

  it("U12 recovery link passes through the unsaved-changes guard", async () => {
    // 51・UI-USW-D1: 導線は route の遷移で離脱の確認を通る（baseline を揃えて guard を外さない）
    const user = userEvent.setup();
    const blockedAtLink: boolean[] = [];
    mockGetProduct.mockResolvedValue({
      status: "ok",
      data: makeMockProductWithRelations({ product_code: "P-005", pos_stock_sync: false }),
    });
    mockUpdateProduct.mockResolvedValue({
      status: "error",
      error: {
        kind: "stocktake_guard",
        message: "連動しない間の売上があります。今の数を入れてから在庫連動にしてください",
        field: null,
        error_id: null,
        stocktake_recovery: {
          code: "recount_required",
          targets: [
            {
              product_code: "P-005",
              product_name: "連動なしの糸",
              stock_unit: "pcs",
              stock: 7,
              stock_range_low: null,
              stock_range_high: null,
              stocktake_item_id: null,
              stocktake_id: null,
              action: "count_now",
              block_reason: null,
              recount_reasons: [],
            },
          ],
          settlement_pending: null,
        },
      },
    });
    const { rerenderPage } = renderWithClient(
      <ProductFormPage
        mode="edit"
        productCode="P-005"
        onNavigateToList={vi.fn()}
        onNavigateToStocktake={() => {
          blockedAtLink.push(shouldBlockCurrentNavigation());
          attemptNavigation();
        }}
      />,
    );

    await user.click(await screen.findByLabelText("POS販売で在庫を減らす"));
    await user.click(screen.getByRole("button", { name: "保存する" }));
    await user.click(await screen.findByRole("button", { name: "今の数を入れる" }));
    rerenderPage();

    expect(blockedAtLink).toEqual([true]);
    expect(await screen.findByText("編集内容が保存されていません")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "編集を続ける" }));
    rerenderPage();
    await waitFor(() => {
      expect(screen.queryByText("編集内容が保存されていません")).not.toBeInTheDocument();
    });
    expect(screen.getByLabelText("POS販売で在庫を減らす")).toBeChecked();

    await user.click(screen.getByRole("button", { name: "今の数を入れる" }));
    rerenderPage();
    expect(blockedAtLink).toEqual([true, true]);
    expect(await screen.findByText("編集内容が保存されていません")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "破棄して移動" }));
    const proceed = mockUseBlocker.mock.results[mockUseBlocker.mock.results.length - 1]?.value as {
      proceed?: () => void;
    };
    expect(proceed.proceed).toHaveBeenCalledTimes(1);
  });
});
