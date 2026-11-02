import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { makeMockProductWithRelations } from "@/features/products/lib/test-fixtures";
import { commands } from "@/lib/bindings";
import { d052InvalidationOracle, expectExactInvalidations } from "@/test/invalidation-oracle";
import { ManualSalePage } from "./ManualSalePage";

vi.mock("@/hooks/useUnsavedChangesWarning", () => ({
  useUnsavedChangesWarning: () => ({
    isBlocked: false,
    continueEditing: vi.fn(),
    discardAndProceed: vi.fn(),
  }),
}));

const mockNavigate = vi.fn();

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
  useNavigate: () => mockNavigate,
  useRouterState: ({ select }: { select: (state: { location: { href: string } }) => unknown }) =>
    select({ location: { href: "/inventory/manual-sale" } }),
}));

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn(), dismiss: vi.fn() },
}));

vi.mock("@/lib/bindings", () => ({
  commands: {
    searchProducts: vi.fn(),
    createManualSale: vi.fn(),
    listInventoryRecords: vi.fn(),
  },
}));

const mockSearchProducts = vi.mocked(commands.searchProducts);
const mockCreateManualSale = vi.mocked(commands.createManualSale);
const mockListInventoryRecords = vi.mocked(commands.listInventoryRecords);
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
      items: [
        makeMockProductWithRelations({
          product_code: "MS-001",
          name: "L3確認 新商品",
          selling_price: 120,
          stock_quantity: 3,
        }),
      ],
      total_count: 1,
      page: 1,
      per_page: 10,
    },
  });

  await user.type(await screen.findByLabelText("手動販売商品検索"), "MS-001{enter}");
  expect(await screen.findByText("MS-001")).toBeInTheDocument();
}

beforeEach(() => {
  mockScrollTo.mockReset();
  vi.stubGlobal("scrollTo", mockScrollTo);
  mockNavigate.mockReset();
  mockSearchProducts.mockReset();
  mockCreateManualSale.mockReset();
  mockListInventoryRecords.mockResolvedValue({
    status: "ok",
    data: {
      items: [],
      total_count: 0,
      page: 1,
      per_page: 5,
    },
  });
});

describe("ManualSalePage (UI-04 / REQ-203)", () => {
  // T6 (UI-04-D17): recent list returnTo contract.
  it("REQ-203/REQ-206: recent list exposes all-history and detail links", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          {
            record_type: "manual_sale",
            record_id: 2,
            business_date: "2026-06-28",
            representative_item: "ボタン #04",
            item_count: 1,
            status: "active",
            created_at: "2026-06-28T02:59:48",
            detail_route: "/inventory/manual-sale/records/2",
          },
        ],
        total_count: 1,
        page: 1,
        per_page: 5,
      },
    });

    renderWithClient(<ManualSalePage />);

    expect(await screen.findByText("直近の手動販売出庫")).toBeInTheDocument();
    // ⑮ SC8/SC11/SC17/SC18: 種別固定のID列・備考なし・一重枠・中立Badge。
    const status = await screen.findByText("有効");
    expect(status).toHaveAttribute("data-slot", "badge");
    expect(status).toHaveClass("text-foreground");
    expect(status).toHaveAttribute("data-variant", "outline");
    expect(status).not.toHaveAttribute("data-tone");
    const table = status.closest("table");
    if (table === null) throw new Error("expected table structure");
    expect(
      within(table)
        .getAllByRole("columnheader")
        .map((cell) => cell.textContent),
    ).toEqual(["販売日", "記録ID", "代表商品", "明細数", "状態", "記録日時", "操作"]);
    expect(table.parentElement).not.toHaveClass("border");
    expect(table.parentElement?.parentElement?.tagName).toBe("SECTION");
    expect(mockListInventoryRecords).toHaveBeenCalledWith({
      record_type: "manual_sale",
      date_from: null,
      date_to: null,
      record_id: null,
      product_keyword: null,
      department_id: null,
      status: null,
      page: 1,
      per_page: 5,
    });
    expect(await screen.findByText("ボタン #04")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "すべての履歴を見る" })).toHaveAttribute(
      "href",
      "/inventory/records?recordType=manual_sale",
    );
    expect(screen.getByRole("link", { name: "詳細を見る" })).toHaveAttribute(
      "href",
      "/inventory/manual-sale/records/2?returnTo=%2Finventory%2Fmanual-sale",
    );
  });

  it("REQ-203/REQ-206: recent list shows empty state without blocking form", async () => {
    renderWithClient(<ManualSalePage />);

    expect(await screen.findByText("直近の手動販売出庫はありません")).toBeInTheDocument();
    expect(screen.getByLabelText("手動販売商品検索")).toBeEnabled();
  });

  it("REQ-203/REQ-206: recent list error stays section-local", async () => {
    mockListInventoryRecords.mockRejectedValue(new Error("recent failed"));

    renderWithClient(<ManualSalePage />);

    expect(await screen.findByText("直近の手動販売出庫を取得できませんでした")).toBeInTheDocument();
    expect(screen.getByLabelText("手動販売商品検索")).toBeEnabled();
  });

  it("REQ-203 adds a single matching product and restores focus", async () => {
    const user = userEvent.setup();
    renderWithClient(<ManualSalePage />);

    await addSingleProduct(user);

    expect(screen.getByText("L3確認 新商品")).toBeInTheDocument();
    expect(screen.getByLabelText("MS-001 の数量")).toHaveValue(1);
    await waitFor(() => {
      expect(screen.getByLabelText("手動販売商品検索")).toHaveFocus();
    });
  });

  it("SC2: 理由selectはSelect comboboxで選択が保存 payload の reason を更新する", async () => {
    const user = userEvent.setup();
    mockCreateManualSale.mockResolvedValueOnce({
      status: "ok",
      data: {
        sale_id: 1,
        created: true,
        idempotent_replay: false,
        plu_warnings: [],
        stock_warnings: [],
        needs_confirmation: false,
        confirmation_token: null,
      },
    });
    renderWithClient(<ManualSalePage />);
    await addSingleProduct(user);

    const reasonTrigger = screen.getByLabelText("理由");
    expect(reasonTrigger).toHaveAttribute("data-slot", "select-trigger");
    expect(reasonTrigger.tagName).toBe("BUTTON");
    expect(reasonTrigger).toHaveTextContent("PLU未登録商品の販売");

    await user.click(reasonTrigger);
    await user.click(await screen.findByRole("option", { name: "その他" }));
    expect(reasonTrigger).toHaveTextContent("その他");

    await user.click(screen.getByRole("button", { name: "手動販売を保存" }));

    await waitFor(() => {
      expect(mockCreateManualSale).toHaveBeenCalledWith(
        expect.objectContaining({ reason: "other" }),
      );
    });
  });

  it("Codex round3 P2-class: 理由selectの初期値（PLU未登録）はreasonとしてそのまま送信される（otherとの内部値独立検査）", async () => {
    const user = userEvent.setup();
    mockCreateManualSale.mockResolvedValueOnce({
      status: "ok",
      data: {
        sale_id: 1,
        created: true,
        idempotent_replay: false,
        plu_warnings: [],
        stock_warnings: [],
        needs_confirmation: false,
        confirmation_token: null,
      },
    });
    renderWithClient(<ManualSalePage />);
    await addSingleProduct(user);

    await user.click(screen.getByRole("button", { name: "手動販売を保存" }));

    await waitFor(() => {
      expect(mockCreateManualSale).toHaveBeenCalledWith(
        expect.objectContaining({ reason: "plu_unregistered" }),
      );
    });
  });

  it("Codex round4 P2-class: 理由selectでその他へ切り替えてからPLU未登録へ戻すとreasonがplu_unregisteredとして送信される（初期値頼みではない検査）", async () => {
    const user = userEvent.setup();
    mockCreateManualSale.mockResolvedValueOnce({
      status: "ok",
      data: {
        sale_id: 1,
        created: true,
        idempotent_replay: false,
        plu_warnings: [],
        stock_warnings: [],
        needs_confirmation: false,
        confirmation_token: null,
      },
    });
    renderWithClient(<ManualSalePage />);
    await addSingleProduct(user);

    const reasonTrigger = screen.getByLabelText("理由");
    await user.click(reasonTrigger);
    await user.click(await screen.findByRole("option", { name: "その他" }));
    await user.click(screen.getByLabelText("理由"));
    await user.click(await screen.findByRole("option", { name: "PLU未登録商品の販売" }));

    await user.click(screen.getByRole("button", { name: "手動販売を保存" }));

    await waitFor(() => {
      expect(mockCreateManualSale).toHaveBeenCalledWith(
        expect.objectContaining({ reason: "plu_unregistered" }),
      );
    });
  });

  it("REQ-203 requires explicit selection when multiple products match", async () => {
    const user = userEvent.setup();
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          makeMockProductWithRelations({ product_code: "MS-001", name: "布A" }),
          makeMockProductWithRelations({ product_code: "MS-002", name: "布B" }),
        ],
        total_count: 2,
        page: 1,
        per_page: 10,
      },
    });

    renderWithClient(<ManualSalePage />);
    await user.type(await screen.findByLabelText("手動販売商品検索"), "布{enter}");

    expect(await screen.findByText("候補から手動販売する商品を選んでください")).toBeInTheDocument();
    expect(screen.queryByLabelText("MS-001 の数量")).not.toBeInTheDocument();

    await user.click(screen.getAllByRole("button", { name: "手動販売に追加" })[1]);

    expect(await screen.findByLabelText("MS-002 の数量")).toBeInTheDocument();
  });

  it("REQ-203 shows product registration guidance for no match and unsaved rows", async () => {
    const user = userEvent.setup();
    renderWithClient(<ManualSalePage />);
    await addSingleProduct(user);
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 10 },
    });

    await user.type(screen.getByLabelText("手動販売商品検索"), "missing{enter}");

    expect(await screen.findByText("該当する商品がありません")).toBeInTheDocument();
    expect(
      screen.getByText("未登録商品の場合は、商品マスタに登録してから手動販売へ戻って追加します。"),
    ).toBeInTheDocument();
    const unsaved = screen.getByText(
      "未保存の手動販売内容があります。商品登録へ進むとこの画面の入力は残りません。",
    );
    // D-CE10: 失敗ではない知らせは注意・確認の文字（危険の赤にしない）。
    expect(unsaved).toHaveClass("text-warning-emphasis");
    expect(unsaved).not.toHaveClass("text-destructive");
    expect(screen.getByRole("link", { name: "商品登録へ進む" })).toHaveAttribute(
      "href",
      "/products/new",
    );
  });

  it("REQ-203 merges duplicate product codes into one row", async () => {
    const user = userEvent.setup();
    renderWithClient(<ManualSalePage />);

    await addSingleProduct(user);
    await user.type(screen.getByLabelText("手動販売商品検索"), "MS-001{enter}");

    expect(screen.getAllByText("MS-001")).toHaveLength(1);
    expect(screen.getByLabelText("MS-001 の数量")).toHaveValue(2);
    expect(screen.getByLabelText("MS-001 の販売金額")).toHaveValue(240);
  });

  it("REQ-203 blocks invalid quantity and amount before submit", async () => {
    const user = userEvent.setup();
    renderWithClient(<ManualSalePage />);
    await addSingleProduct(user);

    fireEvent.change(screen.getByLabelText("MS-001 の数量"), { target: { value: "1.5" } });
    fireEvent.change(screen.getByLabelText("MS-001 の販売金額"), { target: { value: "-1" } });
    await user.click(screen.getByRole("button", { name: "手動販売を保存" }));

    expect(
      await screen.findByText(
        "MS-001: 数量は1以上の整数で入力してください / 販売金額は0以上の整数で入力してください",
      ),
    ).toBeInTheDocument();
    expect(mockScrollTo).not.toHaveBeenCalled();
    expect(mockCreateManualSale).not.toHaveBeenCalled();
  });

  it("REQ-203 scrolls to the top when manual sale create fails", async () => {
    const user = userEvent.setup();
    mockCreateManualSale.mockResolvedValueOnce({
      status: "error",
      error: {
        kind: "internal",
        message: "一時的なエラー",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });

    renderWithClient(<ManualSalePage />);
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "手動販売を保存" }));

    expect(
      await screen.findByText("一時的なエラー。詳細は診断ログに記録されています。"),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(mockScrollTo).toHaveBeenCalledWith({ top: 0, left: 0, behavior: "smooth" });
    });
  });

  it("REQ-203 clears row validation errors when an invalid row is removed and re-added", async () => {
    const user = userEvent.setup();
    renderWithClient(<ManualSalePage />);
    await addSingleProduct(user);

    fireEvent.change(screen.getByLabelText("MS-001 の数量"), { target: { value: "0" } });
    await user.click(screen.getByRole("button", { name: "手動販売を保存" }));

    expect(
      await screen.findByText("MS-001: 数量は1以上の整数で入力してください"),
    ).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "MS-001 を削除" }));
    await user.type(screen.getByLabelText("手動販売商品検索"), "MS-001{enter}");

    expect(
      screen.queryByText("MS-001: 数量は1以上の整数で入力してください"),
    ).not.toBeInTheDocument();
    expect(screen.getByLabelText("MS-001 の数量")).toHaveAttribute("aria-invalid", "false");
  });

  it("REQ-203 keeps validation errors for unchanged rows when another row changes", async () => {
    const user = userEvent.setup();
    mockSearchProducts
      .mockResolvedValueOnce({
        status: "ok",
        data: {
          items: [makeMockProductWithRelations({ product_code: "MS-001", name: "商品A" })],
          total_count: 1,
          page: 1,
          per_page: 10,
        },
      })
      .mockResolvedValueOnce({
        status: "ok",
        data: {
          items: [makeMockProductWithRelations({ product_code: "MS-002", name: "商品B" })],
          total_count: 1,
          page: 1,
          per_page: 10,
        },
      });

    renderWithClient(<ManualSalePage />);
    await user.type(await screen.findByLabelText("手動販売商品検索"), "MS-001{enter}");
    await user.type(screen.getByLabelText("手動販売商品検索"), "MS-002{enter}");
    fireEvent.change(screen.getByLabelText("MS-001 の数量"), { target: { value: "0" } });
    fireEvent.change(screen.getByLabelText("MS-002 の数量"), { target: { value: "0" } });
    await user.click(screen.getByRole("button", { name: "手動販売を保存" }));

    expect(
      await screen.findByText("MS-001: 数量は1以上の整数で入力してください"),
    ).toBeInTheDocument();
    expect(screen.getByText("MS-002: 数量は1以上の整数で入力してください")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "MS-001 を削除" }));

    expect(
      screen.queryByText("MS-001: 数量は1以上の整数で入力してください"),
    ).not.toBeInTheDocument();
    expect(screen.getByText("MS-002: 数量は1以上の整数で入力してください")).toBeInTheDocument();
  });

  it("REQ-203 shows PLU confirmation without result and resubmits with the same key and token", async () => {
    const user = userEvent.setup();
    mockCreateManualSale
      .mockResolvedValueOnce({
        status: "ok",
        data: {
          sale_id: null,
          created: false,
          idempotent_replay: false,
          plu_warnings: ["MS-001: この商品はレジで打てます（PLU登録済み）"],
          stock_warnings: [],
          needs_confirmation: true,
          confirmation_token: "confirm-token-1",
        },
      })
      .mockResolvedValueOnce({
        status: "ok",
        data: {
          sale_id: 31,
          created: true,
          idempotent_replay: false,
          plu_warnings: ["MS-001: この商品はレジで打てます（PLU登録済み）"],
          stock_warnings: [],
          needs_confirmation: false,
          confirmation_token: null,
        },
      });

    const { queryClient } = renderWithClient(<ManualSalePage />);
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "手動販売を保存" }));

    expect(await screen.findByText("PLU登録済みの商品があります")).toBeInTheDocument();
    // 警告明細は AlertDescription（data-slot="alert-description"）内に描画されること。
    // Alert 直下の生要素は grid の幅0px列に落ちて縦一文字表示になる（2026-07-07 実機報告の退行防止）
    const warningItem = screen.getByText("MS-001: この商品はレジで打てます（PLU登録済み）");
    expect(warningItem.closest('[data-slot="alert-description"]')).not.toBeNull();
    await waitFor(() => {
      expect(mockScrollTo).toHaveBeenCalledWith({ top: 0, left: 0, behavior: "smooth" });
    });
    expect(screen.queryByText("手動販売を保存しました")).not.toBeInTheDocument();
    expect(invalidateSpy).not.toHaveBeenCalled();
    const firstKey = mockCreateManualSale.mock.calls[0][0].idempotency_key;

    await user.click(screen.getByRole("button", { name: "確認して保存" }));

    await waitFor(() => {
      expect(mockCreateManualSale).toHaveBeenCalledTimes(2);
    });
    expect(mockCreateManualSale.mock.calls[1][0]).toEqual(
      expect.objectContaining({
        idempotency_key: firstKey,
        confirmation_token: "confirm-token-1",
      }),
    );
    expect(await screen.findByText("手動販売を保存しました")).toBeInTheDocument();
  });

  it("REQ-203 clears PLU confirmation after editing sale contents", async () => {
    const user = userEvent.setup();
    mockCreateManualSale.mockResolvedValue({
      status: "ok",
      data: {
        sale_id: null,
        created: false,
        idempotent_replay: false,
        plu_warnings: ["MS-001: この商品はレジで打てます（PLU登録済み）"],
        stock_warnings: [],
        needs_confirmation: true,
        confirmation_token: "confirm-token-1",
      },
    });

    renderWithClient(<ManualSalePage />);
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "手動販売を保存" }));
    expect(await screen.findByText("PLU登録済みの商品があります")).toBeInTheDocument();

    await user.type(screen.getByLabelText("備考"), "確認後に修正");

    expect(screen.queryByText("PLU登録済みの商品があります")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "手動販売を保存" })).toBeInTheDocument();
  });

  it("T7 UI-04-D17: null sale id keeps the detail link hidden", async () => {
    const user = userEvent.setup();
    mockCreateManualSale.mockResolvedValue({
      status: "ok",
      data: {
        sale_id: null,
        created: false,
        idempotent_replay: false,
        plu_warnings: ["MS-001: この商品はレジで打てます（PLU登録済み）"],
        stock_warnings: [],
        needs_confirmation: true,
        confirmation_token: "confirm-token-1",
      },
    });

    renderWithClient(<ManualSalePage />);
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "手動販売を保存" }));

    expect(await screen.findByText("PLU登録済みの商品があります")).toBeInTheDocument();
    expect(screen.queryByRole("link", { name: "詳細を見る" })).not.toBeInTheDocument();
  });

  it("REQ-203 displays manual sale result and invalidates inventory and sales queries", async () => {
    const user = userEvent.setup();
    mockCreateManualSale.mockResolvedValue({
      status: "ok",
      data: {
        sale_id: 41,
        created: true,
        idempotent_replay: true,
        plu_warnings: [],
        stock_warnings: ["MS-001: 在庫がマイナスになりました（-1）"],
        needs_confirmation: false,
        confirmation_token: null,
      },
    });

    const { queryClient } = renderWithClient(<ManualSalePage />);
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "手動販売を保存" }));
    const saleDate = mockCreateManualSale.mock.calls[0][0].sale_date;

    await waitFor(() => {
      expect(mockScrollTo).toHaveBeenCalledWith({ top: 0, left: 0, behavior: "smooth" });
    });
    expect(await screen.findByText("手動販売を保存しました")).toBeInTheDocument();
    expect(screen.getByText("41")).toBeInTheDocument();
    expect(screen.getByText("再送結果")).toBeInTheDocument();
    expect(screen.getByText("MS-001: 在庫がマイナスになりました（-1）")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "詳細を見る" })).toHaveAttribute(
      "href",
      "/inventory/manual-sale/records/41?returnTo=%2Finventory%2Fmanual-sale",
    );
    expect(screen.getByRole("button", { name: "手動販売を保存" })).toBeDisabled();
    await waitFor(() => {
      expectExactInvalidations(
        invalidateSpy.mock.calls,
        d052InvalidationOracle.manualSale(saleDate),
      );
    });

    await user.click(screen.getByRole("button", { name: "日次売上へ" }));
    expect(mockNavigate).toHaveBeenCalledWith({
      to: "/reports/daily",
      search: { date: saleDate },
    });
  });

  it("REQ-203 disables return and editing while saving", async () => {
    const user = userEvent.setup();
    const deferred = createDeferred<{
      status: "ok";
      data: {
        sale_id: number;
        created: boolean;
        idempotent_replay: boolean;
        plu_warnings: string[];
        stock_warnings: string[];
        needs_confirmation: boolean;
        confirmation_token: string | null;
      };
    }>();
    mockCreateManualSale.mockReturnValue(deferred.promise);

    renderWithClient(<ManualSalePage />);
    await addSingleProduct(user);
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 10 },
    });
    await user.type(screen.getByLabelText("手動販売商品検索"), "missing{enter}");
    expect(await screen.findByRole("link", { name: "商品登録へ進む" })).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "手動販売を保存" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "保存中..." })).toBeDisabled();
      expect(screen.getByRole("button", { name: "リセット" })).toBeDisabled();
      expect(screen.getByLabelText("MS-001 の数量")).toBeDisabled();
      expect(screen.getByLabelText("手動販売商品検索")).toBeDisabled();
      expect(screen.queryByRole("link", { name: "在庫照会へ戻る" })).not.toBeInTheDocument();
      expect(screen.queryByRole("link", { name: "商品登録へ進む" })).not.toBeInTheDocument();
    });

    deferred.resolve({
      status: "ok",
      data: {
        sale_id: 51,
        created: true,
        idempotent_replay: false,
        plu_warnings: [],
        stock_warnings: [],
        needs_confirmation: false,
        confirmation_token: null,
      },
    });
  });
});

describe("ManualSalePage native input tokens（Lane 5 SC4g）", () => {
  it("SC4g: 理由selectがbg-control-surfaceでbg-backgroundを持たない", async () => {
    renderWithClient(<ManualSalePage />);

    const reason = await screen.findByLabelText("理由");
    expect(reason).toHaveClass("bg-control-surface");
    expect(reason).not.toHaveClass("bg-background");
  });
});

it("⑮ SC3/SC14: 副題と直近件数の説明を表示する", () => {
  renderWithClient(<ManualSalePage />);
  expect(
    screen.getByText("レジCSVに入らない販売を手入力し、在庫と売上へ反映します"),
  ).toBeInTheDocument();
  expect(screen.getByText("直近 5 件の手動販売出庫を新しい順に表示します。")).toBeInTheDocument();
});

it.each([
  ["pcs", "1,234 個", "個"],
  ["m", "12.34 m", "m"],
] as const)(
  "⑮ SC19: %s は現在庫を単位付きで表示し、数量と単位を同じ cell に添える",
  async (unit, display, unitLabel) => {
    const user = userEvent.setup();
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          makeMockProductWithRelations({
            product_code: "UNIT-001",
            name: "単位確認A",
            stock_quantity: 1234,
            stock_unit: unit,
          }),
          makeMockProductWithRelations({ product_code: "UNIT-002", name: "単位確認B" }),
        ],
        total_count: 2,
        page: 1,
        per_page: 10,
      },
    });
    renderWithClient(<ManualSalePage />);
    await user.type(await screen.findByLabelText("手動販売商品検索"), "単位{enter}");
    const candidate = (await screen.findByText("UNIT-001")).closest("tr");
    if (candidate === null) throw new Error("expected table structure");
    expect(within(candidate).getByText(display)).toBeInTheDocument();
    await user.click(within(candidate).getByRole("button", { name: "手動販売に追加" }));
    const inputRow = (await screen.findByLabelText("UNIT-001 の数量")).closest("tr");
    if (inputRow === null) throw new Error("expected table structure");
    // SC19 Amendment 3: 入力表の列順と、数量・単位が同じ cell にある契約を固定する。
    const inputTable = inputRow.closest("table");
    if (inputTable === null) throw new Error("expected table structure");
    expect(
      within(inputTable)
        .getAllByRole("columnheader")
        .map((cell) => cell.textContent),
    ).toEqual(["商品コード", "商品名", "部門", "現在庫", "数量", "販売金額", "操作"]);
    const cells = within(inputRow).getAllByRole("cell");
    expect(cells[3].textContent).toBe(display);
    expect(
      within(cells[4]).getByRole("spinbutton", { name: "UNIT-001 の数量" }),
    ).toBeInTheDocument();
    expect(within(cells[4]).getByText(unitLabel)).toBeInTheDocument();
    expect(
      within(cells[5]).getByRole("spinbutton", { name: "UNIT-001 の販売金額" }),
    ).toBeInTheDocument();
    expect(within(cells[5]).getByText("円")).toBeInTheDocument();
  },
);

// REQ-203 / UI-04-D18 / SPEC-UNIT-D7（X11）: 数量の初期値は基準数量ぶん、長さの商品は未編集の間だけ金額が数量に追従する
describe("手動販売の数量と金額（UI-04-D18）", () => {
  async function addProduct(unit: "m" | "pcs") {
    const user = userEvent.setup();
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          makeMockProductWithRelations({
            product_code: "AMT-001",
            name: "金額確認A",
            selling_price: 700,
            stock_unit: unit,
          }),
          makeMockProductWithRelations({ product_code: "AMT-002", name: "金額確認B" }),
        ],
        total_count: 2,
        page: 1,
        per_page: 10,
      },
    });
    renderWithClient(<ManualSalePage />);
    await user.type(await screen.findByLabelText("手動販売商品検索"), "金額{enter}");
    const candidate = (await screen.findByText("AMT-001")).closest("tr");
    if (candidate === null) throw new Error("expected table structure");
    await user.click(within(candidate).getByRole("button", { name: "手動販売に追加" }));
    return {
      user,
      quantity: await screen.findByLabelText("AMT-001 の数量"),
      amount: screen.getByLabelText("AMT-001 の販売金額"),
    };
  }

  it.each([
    ["m", 1, { inputmode: "decimal", min: "0.01", step: "0.01" }],
    ["pcs", 1, { inputmode: "numeric", min: "1", step: null }],
  ] as const)(
    "UI-04-D18: %s の商品を追加すると数量 %s・金額 700 で、数量欄の属性は単位で決まる",
    async (unit, initialQuantity, expected) => {
      const { quantity, amount } = await addProduct(unit);
      expect(quantity).toHaveValue(initialQuantity);
      expect(amount).toHaveValue(700);
      expect(quantity.getAttribute("inputmode")).toBe(expected.inputmode);
      expect(quantity.getAttribute("min")).toBe(expected.min);
      expect(quantity.getAttribute("step")).toBe(expected.step);
    },
  );

  it("UI-04-D18: m の商品は数量 1.3 で金額 910、数量が空の間は金額欄も空、金額を直した後は追従しない", async () => {
    const { user, quantity, amount } = await addProduct("m");
    fireEvent.change(quantity, { target: { value: "1.3" } });
    expect(amount).toHaveValue(910);
    fireEvent.change(quantity, { target: { value: "" } });
    expect(amount).toHaveValue(null);
    fireEvent.change(quantity, { target: { value: "1.3" } });
    expect(amount).toHaveValue(910);
    // 同じ商品の再追加: 数量に 1 m を足し、未編集の金額は式で求め直す
    // （追加すると候補の一覧は消えるので、検索し直して同じ商品を追加する）
    await user.type(screen.getByLabelText("手動販売商品検索"), "金額{enter}");
    const [addAgain] = await screen.findAllByRole("button", { name: "手動販売に追加" });
    await user.click(addAgain);
    expect(quantity).toHaveValue(2.3);
    expect(amount).toHaveValue(1610);
    // 金額を直した後は数量を変えても金額を変えない
    fireEvent.change(amount, { target: { value: "800" } });
    fireEvent.change(quantity, { target: { value: "2" } });
    expect(amount).toHaveValue(800);
  });

  it("UI-04-D18: 個数の商品は数量を変えても金額を求め直さない", async () => {
    const { quantity, amount } = await addProduct("pcs");
    fireEvent.change(quantity, { target: { value: "3" } });
    expect(amount).toHaveValue(700);
  });
});
