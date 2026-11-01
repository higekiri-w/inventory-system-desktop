// UI-16（79 §79.10 の UI-16-D2〜D11）の component test。期待値は 79 から写す。
// U10 の oracle は src/test/invalidation-oracle.ts の C23（79 UI-16-D10 から独立に写した列）。

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import pageSource from "./StockTransferPage.tsx?raw";

import { makeMockProductWithRelations } from "@/features/products/lib/test-fixtures";
import { commands } from "@/lib/bindings";
import type { ProductWithRelations, TransferCreateRequest } from "@/lib/bindings";
import { d052InvalidationOracle, expectExactInvalidations } from "@/test/invalidation-oracle";
import { StockTransferPage } from "./StockTransferPage";

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
    className,
    children,
  }: {
    to: string;
    params?: Record<string, string>;
    search?: Record<string, string>;
    className?: string;
    children: ReactNode;
  }) => {
    const path = params?.recordId ? to.replace("$recordId", params.recordId) : to;
    const query = search ? `?${new URLSearchParams(search).toString()}` : "";
    return (
      <a className={className} href={`${path}${query}`}>
        {children}
      </a>
    );
  },
  useRouterState: ({ select }: { select: (state: { location: { href: string } }) => unknown }) =>
    select({ location: { href: "/inventory/transfer" } }),
}));

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn(), dismiss: vi.fn() },
}));

vi.mock("@/lib/bindings", () => ({
  commands: {
    listInventoryRecords: vi.fn(),
    searchProducts: vi.fn(),
    createTransfer: vi.fn(),
  },
}));

const mockListInventoryRecords = vi.mocked(commands.listInventoryRecords);
const mockSearchProducts = vi.mocked(commands.searchProducts);
const mockCreateTransfer = vi.mocked(commands.createTransfer);
const mockScrollTo = vi.fn();

const OUT_SEARCH = "減らす商品の検索";
const IN_SEARCH = "増やす商品の検索";
const SAVE = "詰め替え・切り分けを保存";

const fabric = makeMockProductWithRelations({
  product_code: "TR-M1",
  name: "合成の反物",
  stock_unit: "m",
  stock_quantity: 1000,
});
const scrap = makeMockProductWithRelations({
  product_code: "TR-01",
  name: "合成の端切れ",
  stock_unit: "pcs",
  stock_quantity: 0,
});

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
  const promise = new Promise<T>((promiseResolve) => {
    resolve = promiseResolve;
  });
  return { promise, resolve };
}

function searchResult(items: ProductWithRelations[]) {
  return {
    status: "ok" as const,
    data: { items, total_count: items.length, page: 1, per_page: 10 },
  };
}

function okResult(recordId: number, extra: { replay?: boolean; warnings?: string[] } = {}) {
  return {
    status: "ok" as const,
    data: {
      record_id: recordId,
      created: !extra.replay,
      idempotent_replay: extra.replay ?? false,
      stock_warnings: extra.warnings ?? [],
    },
  };
}

const internalError = {
  status: "error" as const,
  error: {
    kind: "internal" as const,
    message: "一時的なエラー",
    field: null,
    error_id: null,
    stocktake_recovery: null,
  },
};

async function addTo(
  user: ReturnType<typeof userEvent.setup>,
  label: string,
  product: ProductWithRelations,
) {
  mockSearchProducts.mockResolvedValueOnce(searchResult([product]));
  await user.type(screen.getByLabelText(label), `${product.product_code}{enter}`);
}

async function fillBothSides(user: ReturnType<typeof userEvent.setup>) {
  await addTo(user, OUT_SEARCH, fabric);
  expect(await screen.findByLabelText("TR-M1 の数量")).toBeInTheDocument();
  await addTo(user, IN_SEARCH, scrap);
  expect(await screen.findByLabelText("TR-01 の数量")).toBeInTheDocument();
}

beforeEach(() => {
  mockScrollTo.mockReset();
  vi.stubGlobal("scrollTo", mockScrollTo);
  mockListInventoryRecords.mockReset();
  mockSearchProducts.mockReset();
  mockCreateTransfer.mockReset();
  mockListInventoryRecords.mockResolvedValue({
    status: "ok",
    data: { items: [], total_count: 0, page: 1, per_page: 10 },
  });
});

describe("StockTransferPage（REQ-210 / UI-16）", () => {
  it("UI-16-D1・§79.5: 題・副題・日付の欄（既定は今日）と 2 つの表の見出し", () => {
    renderWithClient(<StockTransferPage />);
    expect(
      screen.getByRole("heading", { name: "詰め替え・切り分け", level: 1 }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "仕入れた品を別の商品にしたとき、減らす商品と増やす商品の数を記録します。原価は変わりません",
      ),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("詰め替え・切り分けた日")).toHaveValue(
      new Date().toLocaleDateString("sv-SE"),
    );
    expect(screen.getByRole("heading", { name: "減らす商品（元）" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "増やす商品（先）" })).toBeInTheDocument();
  });

  it("U2 / UI-16-D2: generated binding の commands だけを使い、ad hoc invoke を呼ばない", () => {
    expect(pageSource).toContain("commands.createTransfer(");
    expect(pageSource).toContain("commands.listInventoryRecords(");
    expect(pageSource).not.toMatch(/\binvoke\s*\(/);
    expect(pageSource).not.toContain("@tauri-apps/api/core");
  });

  it("U3: 2 つの表に Enter で足し、同じ表の再追加は +1、もう片方の表の商品は足さず案内を出し、payload の direction が表に対応する", async () => {
    const user = userEvent.setup();
    mockCreateTransfer.mockResolvedValue(okResult(41));
    renderWithClient(<StockTransferPage />);

    await fillBothSides(user);
    await addTo(user, IN_SEARCH, scrap);
    await waitFor(() => {
      expect(screen.getByLabelText("TR-01 の数量")).toHaveValue(2);
    });

    await addTo(user, IN_SEARCH, fabric);
    expect(await screen.findByText("この商品は「減らす商品」に入っています")).toBeInTheDocument();
    expect(screen.getAllByLabelText("TR-M1 の数量")).toHaveLength(1);
    await addTo(user, OUT_SEARCH, scrap);
    expect(await screen.findByText("この商品は「増やす商品」に入っています")).toBeInTheDocument();
    expect(screen.getAllByLabelText("TR-01 の数量")).toHaveLength(1);

    fireEvent.change(screen.getByLabelText("TR-M1 の数量"), { target: { value: "1.5" } });
    await user.click(screen.getByRole("button", { name: SAVE }));
    await waitFor(() => {
      expect(mockCreateTransfer).toHaveBeenCalledTimes(1);
    });
    const [request] = mockCreateTransfer.mock.calls[0] as [TransferCreateRequest];
    expect(request.transfer_date).toMatch(/^\d{4}-\d{2}-\d{2}$/);
    expect(request.items).toEqual([
      { product_code: "TR-M1", direction: "out", quantity: 150 },
      { product_code: "TR-01", direction: "in", quantity: 2 },
    ]);
  });

  it("U3: 候補が複数なら選んだ表にだけ足す", async () => {
    const user = userEvent.setup();
    renderWithClient(<StockTransferPage />);
    mockSearchProducts.mockResolvedValueOnce(searchResult([fabric, scrap]));
    await user.type(screen.getByLabelText(IN_SEARCH), "合成{enter}");
    expect(await screen.findByText("候補から増やす商品を選んでください")).toBeInTheDocument();
    await user.click(screen.getAllByRole("button", { name: "増やす商品に追加" })[1]);
    const inTable = await screen.findByRole("table", { name: "増やす商品（先）" });
    expect(within(inTable).getByText("TR-01")).toBeInTheDocument();
    expect(screen.queryByRole("table", { name: "減らす商品（元）" })).not.toBeInTheDocument();
  });

  it("U4: どちらかの表が 0 行の間は保存が disabled で、理由の文が出る", async () => {
    const user = userEvent.setup();
    renderWithClient(<StockTransferPage />);
    const reason = "減らす商品と増やす商品を 1 つ以上ずつ入れてください";
    expect(screen.getByRole("button", { name: SAVE })).toBeDisabled();
    expect(screen.getByText(reason)).toBeInTheDocument();

    await addTo(user, OUT_SEARCH, fabric);
    expect(await screen.findByLabelText("TR-M1 の数量")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: SAVE })).toBeDisabled();
    expect(screen.getByText(reason)).toBeInTheDocument();

    await addTo(user, IN_SEARCH, scrap);
    await waitFor(() => {
      expect(screen.getByRole("button", { name: SAVE })).toBeEnabled();
    });
    expect(screen.queryByText(reason)).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "TR-M1 を削除" }));
    expect(screen.getByRole("button", { name: SAVE })).toBeDisabled();
    expect(screen.getByText(reason)).toBeInTheDocument();
  });

  it("U5: 数量の誤りは command の前に行の field error で止め、先頭へスクロールしない", async () => {
    const user = userEvent.setup();
    renderWithClient(<StockTransferPage />);
    await fillBothSides(user);
    fireEvent.change(screen.getByLabelText("TR-01 の数量"), { target: { value: "0" } });
    await user.click(screen.getByRole("button", { name: SAVE }));
    expect(
      await screen.findByText("TR-01: 数量は1以上の整数で入力してください"),
    ).toBeInTheDocument();
    expect(mockCreateTransfer).not.toHaveBeenCalled();
    expect(mockScrollTo).not.toHaveBeenCalled();
  });

  it("U6: 失敗の後の同内容の再試行は同じ key、編集の後・成功の後の新しい記録は新しい key", async () => {
    const user = userEvent.setup();
    mockCreateTransfer
      .mockResolvedValueOnce(internalError)
      .mockResolvedValueOnce(internalError)
      .mockResolvedValueOnce(okResult(51))
      .mockResolvedValueOnce(okResult(52));
    renderWithClient(<StockTransferPage />);
    await fillBothSides(user);

    await user.click(screen.getByRole("button", { name: SAVE }));
    expect(
      await screen.findByText("一時的なエラー。詳細は診断ログに記録されています。"),
    ).toBeInTheDocument();
    const firstKey = mockCreateTransfer.mock.calls[0][0].idempotency_key;
    expect(firstKey).toMatch(/^transfer-/);

    await user.click(screen.getByRole("button", { name: SAVE }));
    await waitFor(() => {
      expect(mockCreateTransfer).toHaveBeenCalledTimes(2);
    });
    expect(mockCreateTransfer.mock.calls[1][0].idempotency_key).toBe(firstKey);

    fireEvent.change(screen.getByLabelText("TR-01 の数量"), { target: { value: "4" } });
    await user.click(screen.getByRole("button", { name: SAVE }));
    expect(await screen.findByText(/詰め替え・切り分けを記録しました/)).toBeInTheDocument();
    const editedKey = mockCreateTransfer.mock.calls[2][0].idempotency_key;
    expect(editedKey).not.toBe(firstKey);

    await user.click(screen.getByRole("button", { name: "続けて詰め替え・切り分け" }));
    await fillBothSides(user);
    await user.click(screen.getByRole("button", { name: SAVE }));
    await waitFor(() => {
      expect(mockCreateTransfer).toHaveBeenCalledTimes(4);
    });
    expect(mockCreateTransfer.mock.calls[3][0].idempotency_key).not.toBe(editedKey);
  });

  it("U7: 保存中は日付・2 つの追加欄・表・リセットが disabled", async () => {
    const user = userEvent.setup();
    const deferred = createDeferred<Awaited<ReturnType<typeof commands.createTransfer>>>();
    mockCreateTransfer.mockReturnValue(deferred.promise);
    renderWithClient(<StockTransferPage />);
    await fillBothSides(user);

    await user.click(screen.getByRole("button", { name: SAVE }));
    expect(
      await screen.findByRole("button", { name: "詰め替え・切り分けを記録しています" }),
    ).toBeDisabled();
    expect(screen.getByLabelText(OUT_SEARCH)).toBeDisabled();
    expect(screen.getByLabelText(IN_SEARCH)).toBeDisabled();
    expect(screen.getByLabelText("詰め替え・切り分けた日")).toBeDisabled();
    expect(screen.getByLabelText("TR-M1 の数量")).toBeDisabled();
    expect(screen.getByLabelText("TR-01 の数量")).toBeDisabled();
    expect(screen.getByRole("button", { name: "リセット" })).toBeDisabled();

    deferred.resolve(okResult(42));
    expect(await screen.findByText(/詰め替え・切り分けを記録しました/)).toBeInTheDocument();
  });

  it.each([
    ["減らす商品", OUT_SEARCH],
    ["増やす商品", IN_SEARCH],
  ])("U7: %sの欄で保存 event と競合して完了した検索の結果を反映しない", async (_side, label) => {
    const user = userEvent.setup();
    const deferredSearch = createDeferred<Awaited<ReturnType<typeof commands.searchProducts>>>();
    const deferredSave = createDeferred<Awaited<ReturnType<typeof commands.createTransfer>>>();
    const late = makeMockProductWithRelations({ product_code: "TR-LATE", name: "競合の商品" });
    mockCreateTransfer.mockImplementation(() => {
      deferredSearch.resolve(searchResult([late]));
      return deferredSave.promise;
    });
    renderWithClient(<StockTransferPage />);
    await fillBothSides(user);
    mockSearchProducts.mockReturnValueOnce(deferredSearch.promise);
    await user.type(screen.getByLabelText(label), "TR-LATE{enter}");

    await user.click(screen.getByRole("button", { name: SAVE }));
    expect(
      await screen.findByRole("button", { name: "詰め替え・切り分けを記録しています" }),
    ).toBeDisabled();
    expect(screen.queryByText("TR-LATE")).not.toBeInTheDocument();

    deferredSave.resolve(okResult(44));
    expect(await screen.findByText(/詰め替え・切り分けを記録しました/)).toBeInTheDocument();
    expect(screen.queryByText("TR-LATE")).not.toBeInTheDocument();
  });

  it.each([
    ["減らす商品", "リセット", "成功", OUT_SEARCH],
    ["減らす商品", "リセット", "失敗", OUT_SEARCH],
    ["減らす商品", "続けて詰め替え・切り分け", "成功", OUT_SEARCH],
    ["減らす商品", "続けて詰め替え・切り分け", "失敗", OUT_SEARCH],
    ["増やす商品", "リセット", "成功", IN_SEARCH],
    ["増やす商品", "リセット", "失敗", IN_SEARCH],
    ["増やす商品", "続けて詰め替え・切り分け", "成功", IN_SEARCH],
    ["増やす商品", "続けて詰め替え・切り分け", "失敗", IN_SEARCH],
  ])(
    "U17 / UI-16-D7: %sの欄で「%s」の前に始まった検索が%sで完了しても新しいフォームに反映しない",
    async (_side, resetButton, outcome, label) => {
      const user = userEvent.setup();
      const deferredSearch = createDeferred<Awaited<ReturnType<typeof commands.searchProducts>>>();
      const late = makeMockProductWithRelations({
        product_code: "TR-LATE",
        name: "古い検索の商品",
      });
      mockCreateTransfer.mockResolvedValue(okResult(45));
      renderWithClient(<StockTransferPage />);
      await fillBothSides(user);
      mockSearchProducts.mockReturnValueOnce(deferredSearch.promise);
      await user.type(screen.getByLabelText(label), "TR-LATE{enter}");

      if (resetButton === "続けて詰め替え・切り分け") {
        await user.click(screen.getByRole("button", { name: SAVE }));
        expect(await screen.findByText(/詰め替え・切り分けを記録しました/)).toBeInTheDocument();
      }
      await user.click(screen.getByRole("button", { name: resetButton }));
      await waitFor(() => {
        expect(screen.queryByLabelText("TR-M1 の数量")).not.toBeInTheDocument();
      });

      await act(async () => {
        deferredSearch.resolve(outcome === "成功" ? searchResult([late]) : internalError);
        await deferredSearch.promise;
      });
      expect(screen.queryByLabelText("TR-LATE の数量")).not.toBeInTheDocument();
      expect(
        screen.queryByText("一時的なエラー。詳細は診断ログに記録されています。"),
      ).not.toBeInTheDocument();
      expect(screen.getByLabelText(label)).toHaveValue("");
      expect(screen.getByRole("button", { name: SAVE })).toBeDisabled();
    },
  );

  it("U18 / §79.6: 検索 0 件はその欄の近くに案内・商品登録への導線・未保存の入力が残らない警告を出し、既存の行を保つ", async () => {
    const user = userEvent.setup();
    renderWithClient(<StockTransferPage />);
    await addTo(user, OUT_SEARCH, fabric);
    expect(await screen.findByLabelText("TR-M1 の数量")).toBeInTheDocument();

    mockSearchProducts.mockResolvedValueOnce(searchResult([]));
    await user.type(screen.getByLabelText(IN_SEARCH), "missing{enter}");

    expect(await screen.findByText("該当する商品がありません")).toBeInTheDocument();
    expect(
      screen.getByText(
        "未登録商品の場合は、商品マスタに登録してから詰め替え・切り分けへ戻って追加します。",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "未保存の詰め替え・切り分けの内容があります。商品登録へ進むとこの画面の入力は残りません。",
      ),
    ).toHaveClass("text-warning-emphasis");
    expect(screen.getByRole("link", { name: "商品登録へ進む" })).toHaveAttribute(
      "href",
      "/products/new",
    );
    expect(screen.getByLabelText("TR-M1 の数量")).toBeInTheDocument();
  });

  it("U18 / §79.6: 検索の失敗は describeError の文をその欄の近くに出し、既存の行を保つ", async () => {
    const user = userEvent.setup();
    renderWithClient(<StockTransferPage />);
    await addTo(user, OUT_SEARCH, fabric);
    expect(await screen.findByLabelText("TR-M1 の数量")).toBeInTheDocument();

    mockSearchProducts.mockResolvedValueOnce(internalError);
    await user.type(screen.getByLabelText(IN_SEARCH), "TR-01{enter}");

    expect(
      await screen.findByText("一時的なエラー。詳細は診断ログに記録されています。"),
    ).toBeInTheDocument();
    expect(screen.queryByRole("link", { name: "商品登録へ進む" })).not.toBeInTheDocument();
    expect(screen.getByLabelText("TR-M1 の数量")).toBeInTheDocument();
  });

  it("U8・U10・U11: 保存結果に記録の番号・2 つの一覧（数量と単位）・warning・replay、C23 の無効化、returnTo 付きの詳細", async () => {
    const user = userEvent.setup();
    mockCreateTransfer.mockResolvedValue(
      okResult(41, { replay: true, warnings: ["TR-M1: 在庫がマイナスになりました"] }),
    );
    const { queryClient } = renderWithClient(<StockTransferPage />);
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    await fillBothSides(user);
    fireEvent.change(screen.getByLabelText("TR-M1 の数量"), { target: { value: "1.5" } });
    fireEvent.change(screen.getByLabelText("TR-01 の数量"), { target: { value: "3" } });
    await user.click(screen.getByRole("button", { name: SAVE }));

    const heading = await screen.findByRole("heading", {
      name: "詰め替え・切り分けを記録しました（記録 #41）",
    });
    await waitFor(() => {
      expect(mockScrollTo).toHaveBeenCalledWith({ top: 0, left: 0, behavior: "smooth" });
    });
    const section = heading.closest("section");
    if (section === null) throw new Error("保存結果 section が見つかりません");
    const result = within(section);
    expect(result.getByRole("heading", { name: "減らした商品" })).toBeInTheDocument();
    expect(result.getByRole("heading", { name: "増やした商品" })).toBeInTheDocument();
    expect(result.getByText("1.5 m")).toBeInTheDocument();
    expect(result.getByText("3 個")).toBeInTheDocument();
    expect(result.getByRole("link", { name: "TR-M1 の在庫を見る" })).toHaveAttribute(
      "href",
      "/products?q=TR-M1",
    );
    expect(result.getByRole("link", { name: "TR-01 の在庫を見る" })).toHaveAttribute(
      "href",
      "/products?q=TR-01",
    );
    expect(result.getByText("TR-M1: 在庫がマイナスになりました")).toBeInTheDocument();
    expect(result.getByText("同じ内容の再送として処理済み")).toBeInTheDocument();
    expect(result.getByRole("link", { name: "詳細を見る" })).toHaveAttribute(
      "href",
      "/inventory/transfer/records/41?returnTo=%2Finventory%2Ftransfer",
    );

    await waitFor(() => {
      expectExactInvalidations(invalidateSpy.mock.calls, d052InvalidationOracle.stockTransfer());
    });
  });

  it("U9・U11: 最近の記録を transfer_record・page 1・per_page 10 で読み、行の詳細は returnTo 付き", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          {
            record_type: "transfer_record",
            record_id: 12,
            business_date: "2026-10-27",
            representative_item: "合成の反物 ほか1件",
            item_count: 2,
            status: "active",
            created_at: "2026-10-27T10:30:00",
            detail_route: "/inventory/transfer/records/12",
          },
        ],
        total_count: 1,
        page: 1,
        per_page: 10,
      },
    });
    renderWithClient(<StockTransferPage />);

    expect(await screen.findByText("合成の反物 ほか1件")).toBeInTheDocument();
    expect(mockListInventoryRecords).toHaveBeenCalledWith(
      expect.objectContaining({ record_type: "transfer_record", page: 1, per_page: 10 }),
    );
    expect(screen.getByText("2026-10-27")).toBeInTheDocument();
    expect(screen.getByText("2026-10-27 10:30:00")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "すべての履歴を見る" })).toHaveAttribute(
      "href",
      "/inventory/records?recordType=transfer_record",
    );
    expect(screen.getByRole("link", { name: "詳細を見る" })).toHaveAttribute(
      "href",
      "/inventory/transfer/records/12?returnTo=%2Finventory%2Ftransfer",
    );
  });

  it("U9: 最近の記録の空と失敗", async () => {
    const { unmount } = renderWithClient(<StockTransferPage />);
    expect(await screen.findByText("最近の詰め替え・切り分けはありません")).toBeInTheDocument();
    unmount();

    mockListInventoryRecords.mockResolvedValue(internalError);
    renderWithClient(<StockTransferPage />);
    expect(await screen.findByText("最近の詰め替え・切り分けを取得できません")).toBeInTheDocument();
  });
});
