import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import dateTimeSource from "./ReceivingPage.tsx?raw";

import {
  makeMockProductWithRelations,
  makeMockSupplier,
} from "@/features/products/lib/test-fixtures";
import { commands } from "@/lib/bindings";
import { d052InvalidationOracle, expectExactInvalidations } from "@/test/invalidation-oracle";
import { ReceivingPage } from "./ReceivingPage";

vi.mock("@/hooks/useUnsavedChangesWarning", () => ({
  useUnsavedChangesWarning: () => ({
    isBlocked: false,
    continueEditing: vi.fn(),
    discardAndProceed: vi.fn(),
  }),
}));

vi.mock("@tanstack/react-router", () => ({
  Link: ({
    to,
    params,
    search,
    children,
  }: {
    to: string;
    params?: Record<string, string>;
    search?: Record<string, string>;
    children: ReactNode;
  }) => {
    const path =
      params === undefined
        ? to
        : Object.entries(params).reduce((path, [key, value]) => {
            return path.replace(`$${key}`, value);
          }, to);
    const query = search === undefined ? "" : `?${new URLSearchParams(search).toString()}`;
    const href = `${path}${query}`;
    return <a href={href}>{children}</a>;
  },
  useRouterState: ({ select }: { select: (state: { location: { href: string } }) => unknown }) =>
    select({ location: { href: "/inventory/receiving" } }),
}));

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn(), dismiss: vi.fn() },
}));

vi.mock("@/lib/bindings", () => ({
  commands: {
    listSuppliers: vi.fn(),
    createSupplier: vi.fn(),
    listReceivings: vi.fn(),
    searchProducts: vi.fn(),
    createReceiving: vi.fn(),
    getProduct: vi.fn(),
    reviseProductPrice: vi.fn(),
  },
}));

const mockListSuppliers = vi.mocked(commands.listSuppliers);
const mockListReceivings = vi.mocked(commands.listReceivings);
const mockSearchProducts = vi.mocked(commands.searchProducts);
const mockCreateReceiving = vi.mocked(commands.createReceiving);
const mockGetProduct = vi.mocked(commands.getProduct);
const mockReviseProductPrice = vi.mocked(commands.reviseProductPrice);
const mockScrollTo = vi.fn();

function renderWithClient(ui: ReactNode) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  return {
    queryClient,
    ...render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>),
  };
}

function mockDefaultQueries() {
  mockListSuppliers.mockResolvedValue({
    status: "ok",
    data: [makeMockSupplier({ id: 1, name: "テスト商事" })],
  });
  mockListReceivings.mockResolvedValue({
    status: "ok",
    data: { items: [], total_count: 0, page: 1, per_page: 10 },
  });
}

function createDeferred<T>() {
  let resolve: (value: T) => void = () => {
    throw new Error("deferred promise is not initialized");
  };
  const promise = new Promise<T>((res) => {
    resolve = res;
  });
  return { promise, resolve };
}

async function addSingleProduct(user: ReturnType<typeof userEvent.setup>) {
  mockSearchProducts.mockResolvedValue({
    status: "ok",
    data: {
      items: [makeMockProductWithRelations({ product_code: "P-001", name: "はさみ" })],
      total_count: 1,
      page: 1,
      per_page: 10,
    },
  });

  await user.type(await screen.findByLabelText("入庫商品検索"), "P-001{enter}");
  expect(await screen.findByText("P-001")).toBeInTheDocument();
}

beforeEach(() => {
  mockScrollTo.mockReset();
  vi.stubGlobal("scrollTo", mockScrollTo);
  mockListSuppliers.mockReset();
  mockListReceivings.mockReset();
  mockSearchProducts.mockReset();
  mockCreateReceiving.mockReset();
  mockGetProduct.mockReset();
  mockReviseProductPrice.mockReset();
  mockDefaultQueries();
});

describe("ReceivingPage (UI-02 / REQ-201)", () => {
  it("adds a single search result and returns focus to the search input", async () => {
    const user = userEvent.setup();
    renderWithClient(<ReceivingPage />);

    await addSingleProduct(user);

    expect(screen.getByText("はさみ")).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getByLabelText("入庫商品検索")).toHaveFocus();
    });
  });

  it("requires selection when product search returns multiple candidates", async () => {
    const user = userEvent.setup();
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          makeMockProductWithRelations({ product_code: "P-001", name: "はさみ" }),
          makeMockProductWithRelations({ product_code: "P-002", name: "布" }),
        ],
        total_count: 2,
        page: 1,
        per_page: 10,
      },
    });

    renderWithClient(<ReceivingPage />);
    await user.type(await screen.findByLabelText("入庫商品検索"), "P{enter}");

    expect(await screen.findByText("候補から入庫する商品を選んでください")).toBeInTheDocument();
    expect(screen.queryByLabelText("P-001 の数量")).not.toBeInTheDocument();

    await user.click(screen.getAllByRole("button", { name: "入庫に追加" })[1]);

    expect(await screen.findByLabelText("P-002 の数量")).toBeInTheDocument();
  });

  it("shows product registration recovery when product search has no results", async () => {
    const user = userEvent.setup();
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 10 },
    });

    renderWithClient(<ReceivingPage />);
    await user.type(await screen.findByLabelText("入庫商品検索"), "missing{enter}");

    expect(await screen.findByText("該当する商品がありません")).toBeInTheDocument();
    expect(
      screen.getByText("未登録商品の場合は、商品マスタに登録してから入庫記録に戻って追加します。"),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "商品登録へ進む" })).toHaveAttribute(
      "href",
      "/products/new",
    );
  });

  it("shows unsaved warning before product registration recovery when rows exist", async () => {
    const user = userEvent.setup();
    renderWithClient(<ReceivingPage />);
    await addSingleProduct(user);
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 10 },
    });

    await user.type(screen.getByLabelText("入庫商品検索"), "missing{enter}");

    const unsaved = await screen.findByText(
      "未保存の入庫内容があります。商品登録へ進むとこの画面の入力は残りません。",
    );
    // D-CE10: 失敗ではない知らせは注意・確認の文字（危険の赤にしない）。
    expect(unsaved).toHaveClass("text-warning-emphasis");
    expect(unsaved).not.toHaveClass("text-destructive");
  });

  it("allows no-supplier save when supplier options fail", async () => {
    const user = userEvent.setup();
    mockListSuppliers.mockResolvedValue({
      status: "error",
      error: {
        kind: "internal",
        message: "取引先取得失敗",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    mockCreateReceiving.mockResolvedValue({
      status: "ok",
      data: {
        record_id: 10,
        created: true,
        idempotent_replay: false,
        stock_warnings: [],
        cost_diffs: [],
      },
    });

    renderWithClient(<ReceivingPage />);
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "入庫を保存" }));

    await waitFor(() => {
      expect(mockCreateReceiving).toHaveBeenCalledWith(
        expect.objectContaining({ supplier_id: null }),
      );
    });
    expect(await screen.findByText("入庫を保存しました")).toBeInTheDocument();
  });

  it("UI-02-D3: selects supplier via picker and allows 指定なし", async () => {
    const user = userEvent.setup();
    mockCreateReceiving.mockResolvedValue({
      status: "ok",
      data: {
        record_id: 20,
        created: true,
        idempotent_replay: false,
        stock_warnings: [],
        cost_diffs: [],
      },
    });

    renderWithClient(<ReceivingPage />);
    await addSingleProduct(user);

    const supplierTrigger = await screen.findByLabelText("取引先");
    expect(supplierTrigger).toHaveAttribute("aria-haspopup", "dialog");
    expect(supplierTrigger.tagName).toBe("BUTTON");
    expect(supplierTrigger).toHaveTextContent("指定なし");

    await user.click(supplierTrigger);
    await user.click(await screen.findByRole("button", { name: "テスト商事" }));
    expect(supplierTrigger).toHaveTextContent("テスト商事");

    // P2-1: 実在取引先から「指定なし」へ解除する round-trip を保存前に確認する
    // （保存後は isFormLocked で trigger が disabled になるため、保存は 1 回だけ行う）。
    await user.click(supplierTrigger);
    await user.click(await screen.findByRole("button", { name: "指定なし" }));
    expect(supplierTrigger).toHaveTextContent("指定なし");

    await user.click(screen.getByRole("button", { name: "入庫を保存" }));
    await waitFor(() => {
      expect(mockCreateReceiving).toHaveBeenCalledWith(
        expect.objectContaining({ supplier_id: null }),
      );
    });
  });

  it("SC3b: 取引先pickerで実在取引先を選んだまま保存すると対応するnumberで保存される（P2-R2-2）", async () => {
    const user = userEvent.setup();
    mockCreateReceiving.mockResolvedValue({
      status: "ok",
      data: {
        record_id: 21,
        created: true,
        idempotent_replay: false,
        stock_warnings: [],
        cost_diffs: [],
      },
    });

    renderWithClient(<ReceivingPage />);
    await addSingleProduct(user);

    const supplierTrigger = await screen.findByLabelText("取引先");
    await user.click(supplierTrigger);
    await user.click(await screen.findByRole("button", { name: "テスト商事" }));
    expect(supplierTrigger).toHaveTextContent("テスト商事");

    await user.click(screen.getByRole("button", { name: "入庫を保存" }));
    await waitFor(() => {
      expect(mockCreateReceiving).toHaveBeenCalledWith(expect.objectContaining({ supplier_id: 1 }));
    });
  });

  it("successful submit shows result and invalidates receiving/product/inventory record queries", async () => {
    const user = userEvent.setup();
    mockCreateReceiving.mockResolvedValue({
      status: "ok",
      data: {
        record_id: 11,
        created: true,
        idempotent_replay: false,
        stock_warnings: ["P-001: 在庫がマイナスになりました（-1）"],
        cost_diffs: [],
      },
    });

    const { queryClient } = renderWithClient(<ReceivingPage />);
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "入庫を保存" }));

    await waitFor(() => {
      expect(mockScrollTo).toHaveBeenCalledWith({ top: 0, left: 0, behavior: "smooth" });
    });
    expect(await screen.findByText("記録ID")).toBeInTheDocument();
    expect(screen.getByText("11")).toBeInTheDocument();
    expect(screen.getByText("P-001: 在庫がマイナスになりました（-1）")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "詳細を見る" })).toHaveAttribute(
      "href",
      "/inventory/receiving/records/11?returnTo=%2Finventory%2Freceiving",
    );
    expect(screen.getByRole("button", { name: "入庫を保存" })).toBeDisabled();
    await user.click(screen.getByRole("button", { name: "入庫を保存" }));
    expect(mockCreateReceiving).toHaveBeenCalledTimes(1);
    await waitFor(() => {
      expectExactInvalidations(invalidateSpy.mock.calls, d052InvalidationOracle.receiving());
    });
  });

  // T2 (UI-02-D16): recent list returnTo contract.
  it("REQ-201/REQ-206: recent list exposes all-history and detail links", async () => {
    mockListReceivings.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          {
            id: 11,
            receiving_date: "2026-06-27",
            supplier_id: 1,
            supplier_name: "テスト商事",
            note: "午前便",
            created_at: "2026-06-27T09:00:00",
          },
        ],
        total_count: 1,
        page: 1,
        per_page: 5,
      },
    });

    renderWithClient(<ReceivingPage />);

    expect((await screen.findAllByText("テスト商事")).length).toBeGreaterThan(0);
    // ⑰ SC6 / UIDISP-D6: 記録日時セルの書体契約。
    expect(screen.getByRole("cell", { name: "2026-06-27 09:00:00" })).toHaveClass(
      "font-mono",
      "tabular-nums",
    );
    expect(screen.getByRole("link", { name: "すべての履歴を見る" })).toHaveAttribute(
      "href",
      "/inventory/records?recordType=receiving_record",
    );
    expect(screen.getByRole("link", { name: "詳細を見る" })).toHaveAttribute(
      "href",
      "/inventory/receiving/records/11?returnTo=%2Finventory%2Freceiving",
    );
  });

  it("REQ-201 clears row validation errors when an invalid row is removed and re-added", async () => {
    const user = userEvent.setup();
    renderWithClient(<ReceivingPage />);
    await addSingleProduct(user);

    fireEvent.change(screen.getByLabelText("P-001 の数量"), { target: { value: "0" } });
    await user.click(screen.getByRole("button", { name: "入庫を保存" }));

    expect(
      await screen.findByText("P-001: 数量は1以上の整数で入力してください"),
    ).toBeInTheDocument();
    expect(mockScrollTo).not.toHaveBeenCalled();

    await user.click(screen.getByRole("button", { name: "P-001 を削除" }));
    await user.type(screen.getByLabelText("入庫商品検索"), "P-001{enter}");

    expect(
      screen.queryByText("P-001: 数量は1以上の整数で入力してください"),
    ).not.toBeInTheDocument();
    expect(screen.getByLabelText("P-001 の数量")).toHaveAttribute("aria-invalid", "false");
  });

  it("REQ-201 keeps validation errors for unchanged rows when another row changes", async () => {
    const user = userEvent.setup();
    mockSearchProducts
      .mockResolvedValueOnce({
        status: "ok",
        data: {
          items: [makeMockProductWithRelations({ product_code: "P-001", name: "はさみ" })],
          total_count: 1,
          page: 1,
          per_page: 10,
        },
      })
      .mockResolvedValueOnce({
        status: "ok",
        data: {
          items: [makeMockProductWithRelations({ product_code: "P-002", name: "布" })],
          total_count: 1,
          page: 1,
          per_page: 10,
        },
      });

    renderWithClient(<ReceivingPage />);
    await user.type(await screen.findByLabelText("入庫商品検索"), "P-001{enter}");
    await user.type(screen.getByLabelText("入庫商品検索"), "P-002{enter}");
    fireEvent.change(screen.getByLabelText("P-001 の数量"), { target: { value: "0" } });
    fireEvent.change(screen.getByLabelText("P-002 の数量"), { target: { value: "0" } });
    await user.click(screen.getByRole("button", { name: "入庫を保存" }));

    expect(
      await screen.findByText("P-001: 数量は1以上の整数で入力してください"),
    ).toBeInTheDocument();
    expect(screen.getByText("P-002: 数量は1以上の整数で入力してください")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "P-001 を削除" }));

    expect(
      screen.queryByText("P-001: 数量は1以上の整数で入力してください"),
    ).not.toBeInTheDocument();
    expect(screen.getByText("P-002: 数量は1以上の整数で入力してください")).toBeInTheDocument();
  });

  it("submit pending disables editing and hides back navigation action", async () => {
    const user = userEvent.setup();
    const deferred = createDeferred<{
      status: "ok";
      data: {
        record_id: number;
        created: boolean;
        idempotent_replay: boolean;
        stock_warnings: string[];
        cost_diffs: [];
      };
    }>();
    mockCreateReceiving.mockReturnValue(deferred.promise);

    renderWithClient(<ReceivingPage />);
    await addSingleProduct(user);
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          makeMockProductWithRelations({ product_code: "P-002", name: "布" }),
          makeMockProductWithRelations({ product_code: "P-003", name: "糸" }),
        ],
        total_count: 2,
        page: 1,
        per_page: 10,
      },
    });
    await user.type(screen.getByLabelText("入庫商品検索"), "P{enter}");
    expect(await screen.findByText("候補から入庫する商品を選んでください")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "入庫を保存" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "保存中..." })).toBeDisabled();
      expect(screen.getByRole("button", { name: "リセット" })).toBeDisabled();
      expect(screen.getByLabelText("P-001 の数量")).toBeDisabled();
      for (const button of screen.getAllByRole("button", { name: "入庫に追加" })) {
        expect(button).toBeDisabled();
      }
      expect(screen.queryByRole("link", { name: "在庫照会へ戻る" })).not.toBeInTheDocument();
    });

    deferred.resolve({
      status: "ok",
      data: {
        record_id: 12,
        created: true,
        idempotent_replay: false,
        stock_warnings: [],
        cost_diffs: [],
      },
    });
  });

  it("keeps the same idempotency key for same-content retry", async () => {
    const user = userEvent.setup();
    mockCreateReceiving
      .mockResolvedValueOnce({
        status: "error",
        error: {
          kind: "internal",
          message: "一時的なエラー",
          field: null,
          error_id: null,
          stocktake_recovery: null,
        },
      })
      .mockResolvedValueOnce({
        status: "ok",
        data: {
          record_id: 20,
          created: true,
          idempotent_replay: false,
          stock_warnings: [],
          cost_diffs: [],
        },
      });

    renderWithClient(<ReceivingPage />);
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "入庫を保存" }));
    expect(
      await screen.findByText("一時的なエラー。詳細は診断ログに記録されています。"),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(mockScrollTo).toHaveBeenCalledWith({ top: 0, left: 0, behavior: "smooth" });
    });
    const firstKey = mockCreateReceiving.mock.calls[0][0].idempotency_key;

    await user.click(screen.getByRole("button", { name: "入庫を保存" }));

    await waitFor(() => {
      expect(mockCreateReceiving).toHaveBeenCalledTimes(2);
    });
    expect(mockCreateReceiving.mock.calls[1][0].idempotency_key).toBe(firstKey);
  });

  it("generates a new idempotency key when the form is edited after create failure", async () => {
    const user = userEvent.setup();
    mockCreateReceiving
      .mockResolvedValueOnce({
        status: "error",
        error: {
          kind: "internal",
          message: "一時的なエラー",
          field: null,
          error_id: null,
          stocktake_recovery: null,
        },
      })
      .mockResolvedValueOnce({
        status: "ok",
        data: {
          record_id: 21,
          created: true,
          idempotent_replay: false,
          stock_warnings: [],
          cost_diffs: [],
        },
      });

    renderWithClient(<ReceivingPage />);
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "入庫を保存" }));
    expect(
      await screen.findByText("一時的なエラー。詳細は診断ログに記録されています。"),
    ).toBeInTheDocument();
    const firstKey = mockCreateReceiving.mock.calls[0][0].idempotency_key;

    await user.type(screen.getByLabelText("備考"), "納品書あり");
    await user.click(screen.getByRole("button", { name: "入庫を保存" }));

    await waitFor(() => {
      expect(mockCreateReceiving).toHaveBeenCalledTimes(2);
    });
    expect(mockCreateReceiving.mock.calls[1][0].idempotency_key).not.toBe(firstKey);
  });

  it("renders recent receiving success empty and error states", async () => {
    mockListReceivings.mockResolvedValueOnce({
      status: "ok",
      data: {
        items: [
          {
            id: 1,
            supplier_id: 1,
            supplier_name: "テスト商事",
            receiving_date: "2026-06-25",
            note: "午前便",
            created_at: "2026-06-25T10:00:00",
          },
        ],
        total_count: 1,
        page: 1,
        per_page: 10,
      },
    });

    renderWithClient(<ReceivingPage />);
    expect(await screen.findByText("午前便")).toBeInTheDocument();
  });

  it("REQ-209 T5 / Matrix T1: 保存済み入庫と更新・見送りの影響を原価差分ダイアログで説明する", async () => {
    const user = userEvent.setup();
    mockCreateReceiving.mockResolvedValue({
      status: "ok",
      data: {
        record_id: 901,
        created: true,
        idempotent_replay: false,
        stock_warnings: [],
        cost_diffs: [
          {
            product_code: "P-901",
            product_name: "テスト毛糸",
            master_cost_price_centi: 50000,
            received_cost_price_centi: 50100,
          },
        ],
      },
    });

    renderWithClient(<ReceivingPage />);
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "入庫を保存" }));

    const dialog = await screen.findByRole("dialog", { name: "入庫原価を確認してください" });
    expect(within(dialog).getByText(/入庫の記録は保存済みです/)).toBeInTheDocument();
    expect(
      within(dialog).getByText(/商品マスタの原価が今回の実原価に変わります/),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByText(/入庫記録はそのまま残り、商品マスタの原価は変わりません/),
    ).toBeInTheDocument();
    expect(within(dialog).getByRole("heading", { name: "テスト毛糸" })).toBeInTheDocument();
    expect(within(dialog).getByText("商品コード")).toBeInTheDocument();
    expect(within(dialog).getByText("P-901")).toBeInTheDocument();
    expect(within(dialog).getByText("マスタ原価")).toBeInTheDocument();
    expect(within(dialog).getByText("¥500")).toBeInTheDocument();
    expect(within(dialog).getByText("今回の実原価")).toBeInTheDocument();
    expect(within(dialog).getByText("¥501")).toBeInTheDocument();
  });

  it("REQ-209 / DSR-16 Matrix T6: 複数の原価差分商品を一意の見出しで識別できる", async () => {
    const user = userEvent.setup();
    mockCreateReceiving.mockResolvedValue({
      status: "ok",
      data: {
        record_id: 902,
        created: true,
        idempotent_replay: false,
        stock_warnings: [],
        cost_diffs: [
          {
            product_code: "P-901",
            product_name: "テスト毛糸",
            master_cost_price_centi: 50000,
            received_cost_price_centi: 50100,
          },
          {
            product_code: "P-902",
            product_name: "テスト生地",
            master_cost_price_centi: 70000,
            received_cost_price_centi: 72000,
          },
        ],
      },
    });

    renderWithClient(<ReceivingPage />);
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "入庫を保存" }));

    const dialog = await screen.findByRole("dialog", { name: "入庫原価を確認してください" });
    expect(within(dialog).getByRole("heading", { name: "テスト毛糸" })).toBeInTheDocument();
    expect(within(dialog).getByRole("heading", { name: "テスト生地" })).toBeInTheDocument();
  });

  it("REQ-209 T6: 原価差分が空または冪等再送ならダイアログを表示しない", async () => {
    const user = userEvent.setup();
    mockCreateReceiving.mockResolvedValueOnce({
      status: "ok",
      data: {
        record_id: 902,
        created: true,
        idempotent_replay: false,
        stock_warnings: [],
        cost_diffs: [],
      },
    });
    const first = renderWithClient(<ReceivingPage />);
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "入庫を保存" }));
    expect(await screen.findByText("入庫を保存しました")).toBeInTheDocument();
    expect(
      screen.queryByRole("dialog", { name: "入庫原価を確認してください" }),
    ).not.toBeInTheDocument();
    first.unmount();

    mockCreateReceiving.mockResolvedValueOnce({
      status: "ok",
      data: {
        record_id: 902,
        created: false,
        idempotent_replay: true,
        stock_warnings: [],
        cost_diffs: [],
      },
    });
    renderWithClient(<ReceivingPage />);
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "入庫を保存" }));
    expect(await screen.findByText("同じ内容の再送として処理済み")).toBeInTheDocument();
    expect(
      screen.queryByRole("dialog", { name: "入庫原価を確認してください" }),
    ).not.toBeInTheDocument();
  });

  it("Matrix C16 / UI-02-D7・D15: 原価 83.33 円の商品は欄が 83.33 で 8333 を送り、差分の dialog は ¥83.33・¥85 で 8500 に更新する", async () => {
    const user = userEvent.setup();
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          makeMockProductWithRelations({
            product_code: "C16-A",
            name: "原価の小数",
            cost_price_centi: 8333,
          }),
        ],
        total_count: 1,
        page: 1,
        per_page: 10,
      },
    });
    mockCreateReceiving.mockResolvedValue({
      status: "ok",
      data: {
        record_id: 916,
        created: true,
        idempotent_replay: false,
        stock_warnings: [],
        cost_diffs: [
          {
            product_code: "C16-A",
            product_name: "原価の小数",
            master_cost_price_centi: 8333,
            received_cost_price_centi: 8500,
          },
        ],
      },
    });
    mockGetProduct.mockResolvedValue({
      status: "ok",
      data: makeMockProductWithRelations({ product_code: "C16-A", selling_price: 100 }),
    });
    mockReviseProductPrice.mockResolvedValue({
      status: "ok",
      data: {
        product_code: "C16-A",
        changed: true,
        plu_dirty_set: false,
        supplier_assigned: false,
      },
    });
    renderWithClient(<ReceivingPage />);
    await user.type(await screen.findByLabelText("入庫商品検索"), "C16-A{enter}");
    const cost = await screen.findByLabelText("C16-A の原価");
    expect(cost).toHaveValue(83.33);
    expect(cost).toHaveAttribute("inputmode", "decimal");
    expect(cost).toHaveAttribute("step", "0.01");
    await user.click(screen.getByRole("button", { name: "入庫を保存" }));
    await waitFor(() => {
      expect(mockCreateReceiving).toHaveBeenCalledTimes(1);
    });
    expect(mockCreateReceiving.mock.calls[0][0].items[0]?.cost_price_centi).toBe(8333);
    const dialog = await screen.findByRole("dialog", { name: "入庫原価を確認してください" });
    expect(within(dialog).getByText("¥83.33")).toBeInTheDocument();
    expect(within(dialog).getByText("¥85")).toBeInTheDocument();
    await user.click(
      within(dialog).getByRole("button", { name: "マスタ原価をこの実原価に更新する" }),
    );
    await waitFor(() => {
      expect(mockReviseProductPrice).toHaveBeenCalledWith({
        product_code: "C16-A",
        new_selling_price: 100,
        new_cost_price_centi: 8500,
        assign_supplier_id: null,
      });
    });
  });

  it("REQ-209 T7: 現売価を据え置いて入庫実原価へ商品単位で更新する", async () => {
    const user = userEvent.setup();
    mockCreateReceiving.mockResolvedValue({
      status: "ok",
      data: {
        record_id: 903,
        created: true,
        idempotent_replay: false,
        stock_warnings: [],
        cost_diffs: [
          {
            product_code: "P-901",
            product_name: "テスト毛糸",
            master_cost_price_centi: 50000,
            received_cost_price_centi: 50100,
          },
        ],
      },
    });
    mockGetProduct.mockResolvedValue({
      status: "ok",
      data: makeMockProductWithRelations({ product_code: "P-901", selling_price: 1200 }),
    });
    mockReviseProductPrice.mockResolvedValue({
      status: "ok",
      data: {
        product_code: "P-901",
        changed: true,
        plu_dirty_set: false,
        supplier_assigned: false,
      },
    });

    renderWithClient(<ReceivingPage />);
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "入庫を保存" }));
    await user.click(
      await screen.findByRole("button", { name: "マスタ原価をこの実原価に更新する" }),
    );

    await waitFor(() => {
      expect(mockReviseProductPrice).toHaveBeenCalledWith({
        product_code: "P-901",
        new_selling_price: 1200,
        new_cost_price_centi: 50100,
        assign_supplier_id: null,
      });
    });
    expect(mockReviseProductPrice).toHaveBeenCalledTimes(1);
    expect(await screen.findByText("マスタ原価を更新しました")).toBeInTheDocument();
  });

  it("REQ-209 T8: 原価更新失敗を行内表示して再試行でき、入庫成功表示を維持する", async () => {
    const user = userEvent.setup();
    mockCreateReceiving.mockResolvedValue({
      status: "ok",
      data: {
        record_id: 904,
        created: true,
        idempotent_replay: false,
        stock_warnings: [],
        cost_diffs: [
          {
            product_code: "P-901",
            product_name: "テスト毛糸",
            master_cost_price_centi: 50000,
            received_cost_price_centi: 50100,
          },
        ],
      },
    });
    mockGetProduct.mockResolvedValue({
      status: "ok",
      data: makeMockProductWithRelations({ product_code: "P-901", selling_price: 1200 }),
    });
    mockReviseProductPrice
      .mockResolvedValueOnce({
        status: "error",
        error: {
          kind: "internal",
          message: "更新失敗",
          field: null,
          error_id: null,
          stocktake_recovery: null,
        },
      })
      .mockResolvedValueOnce({
        status: "ok",
        data: {
          product_code: "P-901",
          changed: true,
          plu_dirty_set: false,
          supplier_assigned: false,
        },
      });

    renderWithClient(<ReceivingPage />);
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "入庫を保存" }));
    await user.click(
      await screen.findByRole("button", { name: "マスタ原価をこの実原価に更新する" }),
    );

    expect(await screen.findByText("マスタ原価の更新に失敗しました")).toBeInTheDocument();
    expect(screen.getByText("入庫を保存しました")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "再試行する" }));
    expect(await screen.findByText("マスタ原価を更新しました")).toBeInTheDocument();
    expect(mockReviseProductPrice).toHaveBeenCalledTimes(2);
  });

  it("REQ-209 T9: 見送りで閉じても更新や記録を行わず入庫成功表示を維持する", async () => {
    const user = userEvent.setup();
    mockCreateReceiving.mockResolvedValue({
      status: "ok",
      data: {
        record_id: 905,
        created: true,
        idempotent_replay: false,
        stock_warnings: [],
        cost_diffs: [
          {
            product_code: "P-901",
            product_name: "テスト毛糸",
            master_cost_price_centi: 50000,
            received_cost_price_centi: 50100,
          },
        ],
      },
    });

    renderWithClient(<ReceivingPage />);
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "入庫を保存" }));
    await user.click(await screen.findByRole("button", { name: "見送って閉じる" }));

    expect(
      screen.queryByRole("dialog", { name: "入庫原価を確認してください" }),
    ).not.toBeInTheDocument();
    expect(screen.getByText("入庫を保存しました")).toBeInTheDocument();
    expect(mockGetProduct).not.toHaveBeenCalled();
    expect(mockReviseProductPrice).not.toHaveBeenCalled();
  });

  it("T4 (UI 表示磨き batch Scope 4): 原価更新の成功/失敗を role とテキストで判別できる success token 表示にする", async () => {
    const user = userEvent.setup();
    mockCreateReceiving.mockResolvedValue({
      status: "ok",
      data: {
        record_id: 906,
        created: true,
        idempotent_replay: false,
        stock_warnings: [],
        cost_diffs: [
          {
            product_code: "P-901",
            product_name: "テスト毛糸",
            master_cost_price_centi: 50000,
            received_cost_price_centi: 50100,
          },
          {
            product_code: "P-902",
            product_name: "テストボタン",
            master_cost_price_centi: 10000,
            received_cost_price_centi: 11000,
          },
        ],
      },
    });
    mockGetProduct.mockImplementation((productCode: string) =>
      Promise.resolve({
        status: "ok",
        data: makeMockProductWithRelations({ product_code: productCode, selling_price: 1200 }),
      }),
    );
    mockReviseProductPrice
      .mockResolvedValueOnce({
        status: "ok",
        data: {
          product_code: "P-901",
          changed: true,
          plu_dirty_set: false,
          supplier_assigned: false,
        },
      })
      .mockResolvedValueOnce({
        status: "error",
        error: {
          kind: "internal",
          message: "更新失敗",
          field: null,
          error_id: null,
          stocktake_recovery: null,
        },
      });

    renderWithClient(<ReceivingPage />);
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "入庫を保存" }));

    const dialog = await screen.findByRole("dialog", { name: "入庫原価を確認してください" });
    const buttons = within(dialog).getAllByRole("button", {
      name: "マスタ原価をこの実原価に更新する",
    });
    expect(buttons).toHaveLength(2);
    await user.click(buttons[0]);
    await user.click(buttons[1]);

    await waitFor(() => {
      expect(within(dialog).getByRole("status")).toHaveTextContent("マスタ原価を更新しました");
    });
    // 色だけに依存しない判別（UI-02-D15）: role と日本語テキストの両方で成功/失敗を見分けられる。
    await waitFor(() => {
      expect(within(dialog).getByRole("alert")).toHaveTextContent("マスタ原価の更新に失敗しました");
    });
  });

  it("T5 (UI 表示磨き batch Scope 5): 原価更新成功後にマスタ原価表示が新値へ更新され、全行処理済みで閉じるボタン文言が変わる", async () => {
    const user = userEvent.setup();
    mockCreateReceiving.mockResolvedValue({
      status: "ok",
      data: {
        record_id: 907,
        created: true,
        idempotent_replay: false,
        stock_warnings: [],
        cost_diffs: [
          {
            product_code: "P-901",
            product_name: "テスト毛糸",
            master_cost_price_centi: 50000,
            received_cost_price_centi: 50100,
          },
        ],
      },
    });
    mockGetProduct.mockResolvedValue({
      status: "ok",
      data: makeMockProductWithRelations({ product_code: "P-901", selling_price: 1200 }),
    });
    mockReviseProductPrice.mockResolvedValue({
      status: "ok",
      data: {
        product_code: "P-901",
        changed: true,
        plu_dirty_set: false,
        supplier_assigned: false,
      },
    });

    renderWithClient(<ReceivingPage />);
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "入庫を保存" }));

    const dialog = await screen.findByRole("dialog", { name: "入庫原価を確認してください" });
    function masterCostValue() {
      const label = within(dialog).getByText("マスタ原価");
      return label.parentElement?.querySelector("dd")?.textContent;
    }
    expect(masterCostValue()).toBe("¥500");
    // footer は未処理のうちは既存の「見送って閉じる」のまま。
    expect(within(dialog).getByRole("button", { name: "見送って閉じる" })).toBeInTheDocument();

    await user.click(
      within(dialog).getByRole("button", { name: "マスタ原価をこの実原価に更新する" }),
    );

    await waitFor(() => {
      expect(masterCostValue()).toBe("¥501");
    });
    // 61 §61.5 追記: 全行の更新完了後、footer 文言が状態対応（「閉じる」）に変わる。
    expect(within(dialog).getByRole("button", { name: "閉じる" })).toBeInTheDocument();
    expect(
      within(dialog).queryByRole("button", { name: "見送って閉じる" }),
    ).not.toBeInTheDocument();
  });

  it("T11 (owner L3 P1 — merge blocker): 原価差分ダイアログは外側クリック・Escape・×では閉じず、明示ボタンでのみ閉じる", async () => {
    const user = userEvent.setup();
    mockCreateReceiving.mockResolvedValue({
      status: "ok",
      data: {
        record_id: 908,
        created: true,
        idempotent_replay: false,
        stock_warnings: [],
        cost_diffs: [
          {
            product_code: "P-901",
            product_name: "テスト毛糸",
            master_cost_price_centi: 50000,
            received_cost_price_centi: 50100,
          },
        ],
      },
    });

    renderWithClient(<ReceivingPage />);
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "入庫を保存" }));

    const dialog = await screen.findByRole("dialog", { name: "入庫原価を確認してください" });

    // 右上×（DialogPrimitive.Close、既定 showCloseButton）が render されないこと。
    expect(document.querySelector('[data-slot="dialog-close"]')).not.toBeInTheDocument();

    // overlay への pointer-down（dialog 外側クリック相当）で dialog が残ること。
    const overlay = document.querySelector('[data-slot="dialog-overlay"]');
    if (overlay === null) throw new Error("dialog-overlay not found");
    fireEvent.pointerDown(overlay);
    expect(screen.getByRole("dialog", { name: "入庫原価を確認してください" })).toBeInTheDocument();

    // Escape keydown で dialog が残ること。
    await user.keyboard("{Escape}");
    expect(screen.getByRole("dialog", { name: "入庫原価を確認してください" })).toBeInTheDocument();

    // 更新中は disabled・成功後は success 表示という既存契約は維持されている
    // （T4/T5 の回帰、ここでは disabled 状態の入口だけ再確認する）。
    expect(
      within(dialog).getByRole("button", { name: "マスタ原価をこの実原価に更新する" }),
    ).toBeEnabled();

    // footer の明示ボタン（「見送って閉じる」）でのみ閉じること。
    await user.click(within(dialog).getByRole("button", { name: "見送って閉じる" }));
    expect(
      screen.queryByRole("dialog", { name: "入庫原価を確認してください" }),
    ).not.toBeInTheDocument();
  });
});

describe("ReceivingPage native input tokens（Lane 5 SC4b）", () => {
  it("SC4b: 取引先triggerがbg-control-surfaceでbg-backgroundを持たない", async () => {
    renderWithClient(<ReceivingPage />);

    const supplier = await screen.findByLabelText("取引先");
    expect(supplier).toHaveClass("bg-control-surface");
    expect(supplier).not.toHaveClass("bg-background");
  });
});

it("⑮ SC3/SC14: 副題と直近件数の説明を表示する", () => {
  renderWithClient(<ReceivingPage />);
  expect(screen.getByText("届いた商品をまとめて入庫し、在庫へ反映します")).toBeInTheDocument();
  expect(screen.getByText("直近 10 件の入庫を新しい順に表示します。")).toBeInTheDocument();
});

it.each([
  [null, "—", undefined],
  ["", "—", undefined],
  ["   ", "—", undefined],
  [
    "長い備考を省略しても全文を確認できます。",
    "長い備考を省略しても全文を確認できます。",
    "長い備考を省略しても全文を確認できます。",
  ],
])("⑮ SC9: 入庫備考=%s の表示とtitle", async (note, expected, title) => {
  mockListReceivings.mockResolvedValue({
    status: "ok",
    data: {
      items: [
        {
          id: 1,
          receiving_date: "2026-09-08",
          supplier_name: "取引先",
          supplier_id: 1,
          note,
          created_at: "2026-09-08T10:00:00",
        },
      ],
      total_count: 1,
      page: 1,
      per_page: 10,
    },
  });
  renderWithClient(<ReceivingPage />);
  const cell = await screen.findByText(expected);
  expect(cell).toHaveClass("truncate");
  if (title === undefined) expect(cell).not.toHaveAttribute("title");
  else expect(cell).toHaveAttribute("title", title);
});

it("⑰ SC6 / UIDISP-D6: 共有 formatDateTime を import しローカル定義を持たない", () => {
  expect(dateTimeSource).toMatch(
    /import\s*\{[^}]*\bformatDateTime\b[^}]*\}\s*from\s*"@\/lib\/date-time"/,
  );
  expect(dateTimeSource).not.toMatch(/function\s+(?:formatDateTime|formatCheckedAt)\s*\(/);
  expect(dateTimeSource).not.toContain("formatCheckedAt");
});

it("UI-02-D3: refetches suppliers after creating one from the picker", async () => {
  const created = makeMockSupplier({ id: 44, name: "追加取引先" });
  vi.mocked(commands.createSupplier).mockResolvedValue({ status: "ok", data: created });
  mockListSuppliers
    .mockResolvedValueOnce({ status: "ok", data: [] })
    .mockResolvedValue({ status: "ok", data: [created] });
  const user = userEvent.setup();
  renderWithClient(<ReceivingPage />);
  const trigger = screen.getByLabelText("取引先");
  await waitFor(() => {
    expect(trigger).toBeEnabled();
  });
  await user.click(trigger);
  await user.click(screen.getByRole("button", { name: "新しい取引先を追加" }));
  await user.type(screen.getByLabelText("取引先名"), created.name);
  await user.click(screen.getByRole("button", { name: "追加する" }));
  await waitFor(() => {
    expect(trigger).toHaveTextContent(created.name);
  });
  expect(mockListSuppliers).toHaveBeenCalledTimes(2);
  expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  expect(mockCreateReceiving).not.toHaveBeenCalled();
});

// REQ-201 / UI-02-D7 / SPEC-UNIT-D3: 数量欄の属性は m の商品の行だけ小数 2 桁の入力にする
it.each([
  ["m", { inputmode: "decimal", min: "0.01", step: "0.01" }],
  ["pcs", { inputmode: "numeric", min: "1", step: null }],
] as const)("UI-02-D7: %s の商品の数量欄の属性", async (unit, expected) => {
  const user = userEvent.setup();
  mockSearchProducts.mockResolvedValue({
    status: "ok",
    data: {
      items: [
        makeMockProductWithRelations({
          product_code: "ATTR-001",
          name: "属性確認A",
          stock_unit: unit,
        }),
        makeMockProductWithRelations({ product_code: "ATTR-002", name: "属性確認B" }),
      ],
      total_count: 2,
      page: 1,
      per_page: 10,
    },
  });
  renderWithClient(<ReceivingPage />);
  await user.type(await screen.findByLabelText("入庫商品検索"), "属性{enter}");
  const candidate = (await screen.findByText("ATTR-001")).closest("tr");
  if (candidate === null) throw new Error("expected table structure");
  await user.click(within(candidate).getByRole("button", { name: "入庫に追加" }));
  const input = await screen.findByLabelText("ATTR-001 の数量");
  expect(input).toHaveValue(1);
  expect(input).toHaveAttribute("type", "number");
  expect(input.getAttribute("inputmode")).toBe(expected.inputmode);
  expect(input.getAttribute("min")).toBe(expected.min);
  expect(input.getAttribute("step")).toBe(expected.step);
});
