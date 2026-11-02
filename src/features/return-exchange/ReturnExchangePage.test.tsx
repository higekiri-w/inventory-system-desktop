import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import dateTimeSource from "./ReturnExchangePage.tsx?raw";

import { makeMockProductWithRelations } from "@/features/products/lib/test-fixtures";
import { commands } from "@/lib/bindings";
import { d052InvalidationOracle, expectExactInvalidations } from "@/test/invalidation-oracle";
import { ReturnExchangePage } from "./ReturnExchangePage";

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
const mockScrollTo = vi.fn();

const registerProcessedStockDescription =
  "この保存では在庫数を変更しません。日次CSV取込みで返品分の在庫が反映されます。";
const registerUnprocessedStockDescription =
  "この保存で在庫数を反映します。日次CSVに同じ返品を重ねて取込まない運用です。";

function createDeferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((promiseResolve) => {
    resolve = promiseResolve;
  });
  return { promise, resolve };
}

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
  mockListReturns.mockResolvedValue({
    status: "ok",
    data: { items: [], total_count: 0, page: 1, per_page: 10 },
  });
}

async function addSingleProduct(user: ReturnType<typeof userEvent.setup>) {
  mockSearchProducts.mockResolvedValue({
    status: "ok",
    data: {
      items: [makeMockProductWithRelations({ product_code: "RT-001", name: "返品商品" })],
      total_count: 1,
      page: 1,
      per_page: 10,
    },
  });

  await user.type(await screen.findByLabelText("返品・交換商品検索"), "RT-001{enter}");
  expect(await screen.findByText("RT-001")).toBeInTheDocument();
}

function dropReceipt(file: File) {
  fireEvent.drop(screen.getByTestId("file-picker-dropzone"), {
    dataTransfer: { files: [file] },
  });
}

beforeEach(() => {
  mockScrollTo.mockReset();
  vi.stubGlobal("scrollTo", mockScrollTo);
  vi.spyOn(URL, "createObjectURL").mockReturnValue("blob:receipt-preview");
  mockListReturns.mockReset();
  mockSearchProducts.mockReset();
  mockCreateReturn.mockReset();
  mockSaveReceiptImage.mockReset();
  mockDefaultQueries();
});

describe("ReturnExchangePage (UI-03 / REQ-202)", () => {
  it("shows register processed explanation as text and badge", async () => {
    const user = userEvent.setup();
    renderWithClient(<ReturnExchangePage />);

    expect(await screen.findByText("CSV取込みで反映")).toBeInTheDocument();
    expect(screen.getAllByText(registerProcessedStockDescription).length).toBeGreaterThan(0);
    expect(screen.getByLabelText("レジ戻し済み")).toBeChecked();
    await user.click(screen.getByLabelText("レジ未処理"));

    expect(screen.getByLabelText("レジ未処理")).toBeChecked();
    expect(screen.getByText("この保存で反映")).toBeInTheDocument();
    expect(screen.getAllByText(registerUnprocessedStockDescription).length).toBeGreaterThan(0);
  });

  it("successful submit invalidates returns and stock keys only when register_processed=false", async () => {
    const user = userEvent.setup();
    mockCreateReturn.mockResolvedValue({
      status: "ok",
      data: { record_id: 30, created: true, idempotent_replay: false, stock_warnings: [] },
    });

    const { queryClient } = renderWithClient(<ReturnExchangePage />);
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    await addSingleProduct(user);
    await user.click(screen.getByLabelText("レジ未処理"));
    await user.click(screen.getByRole("button", { name: "返品・交換を保存" }));

    await waitFor(() => {
      expect(mockScrollTo).toHaveBeenCalledWith({ top: 0, left: 0, behavior: "smooth" });
    });
    expect(await screen.findByText("返品・交換を保存しました")).toBeInTheDocument();
    expect(screen.getAllByText(registerUnprocessedStockDescription).length).toBeGreaterThan(0);
    expect(screen.getByRole("link", { name: "詳細を見る" })).toHaveAttribute(
      "href",
      "/inventory/return/records/30?returnTo=%2Finventory%2Freturn",
    );
    await waitFor(() => {
      expectExactInvalidations(
        invalidateSpy.mock.calls,
        d052InvalidationOracle.returnExchange(false),
      );
    });
  });

  // T4 (UI-03-D22): recent list returnTo contract.
  it("REQ-202/REQ-206: recent list exposes all-history and detail links", async () => {
    mockListReturns.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          {
            id: 30,
            return_date: "2026-06-27",
            return_type: "return",
            register_processed: false,
            note: "袋破れ",
            created_at: "2026-06-27T10:00:00",
          },
        ],
        total_count: 1,
        page: 1,
        per_page: 5,
      },
    });

    renderWithClient(<ReturnExchangePage />);

    expect(await screen.findByText("袋破れ")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "すべての履歴を見る" })).toHaveAttribute(
      "href",
      "/inventory/records?recordType=return_record",
    );
    expect(screen.getByRole("link", { name: "詳細を見る" })).toHaveAttribute(
      "href",
      "/inventory/return/records/30?returnTo=%2Finventory%2Freturn",
    );
  });

  it("REQ-202/UI-03-D19: note is multiline and visible in the saved result", async () => {
    const user = userEvent.setup();
    mockCreateReturn.mockResolvedValue({
      status: "ok",
      data: { record_id: 36, created: true, idempotent_replay: false, stock_warnings: [] },
    });

    renderWithClient(<ReturnExchangePage />);
    const noteField = await screen.findByLabelText("備考");
    expect(noteField.tagName).toBe("TEXTAREA");
    await user.type(noteField, "サイズ交換のため確認済み");
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "返品・交換を保存" }));

    await waitFor(() => {
      expect(mockCreateReturn).toHaveBeenCalled();
    });
    expect(mockCreateReturn.mock.calls[0][0].note).toBe("サイズ交換のため確認済み");
    const resultRegion = await screen.findByRole("region", { name: "保存結果" });
    expect(within(resultRegion).getByText("備考")).toBeInTheDocument();
    expect(within(resultRegion).getByText("サイズ交換のため確認済み")).toBeInTheDocument();
  });

  it("REQ-202/REQ-206/UI-03-D19: recent list shows note text and fallback", async () => {
    mockListReturns.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          {
            id: 30,
            return_date: "2026-06-27",
            return_type: "return",
            register_processed: false,
            note: "袋破れ",
            created_at: "2026-06-27T10:00:00",
          },
          {
            id: 31,
            return_date: "2026-06-28",
            return_type: "exchange",
            register_processed: true,
            note: null,
            created_at: "2026-06-28T11:00:00",
          },
        ],
        total_count: 2,
        page: 1,
        per_page: 10,
      },
    });

    renderWithClient(<ReturnExchangePage />);

    const recentRegion = await screen.findByRole("region", { name: "直近の返品・交換" });
    expect(await within(recentRegion).findByText("袋破れ")).toBeInTheDocument();
    // ⑰ SC6 / UIDISP-D6: 記録日時セルの書体契約。
    expect(within(recentRegion).getByRole("cell", { name: "2026-06-27 10:00:00" })).toHaveClass(
      "font-mono",
      "tabular-nums",
    );
    expect(within(recentRegion).getByText("—")).toBeInTheDocument();
    expect(within(recentRegion).queryByText("備考なし")).not.toBeInTheDocument();
  });

  it("successful register-processed submit invalidates returns without stock keys", async () => {
    const user = userEvent.setup();
    mockCreateReturn.mockResolvedValue({
      status: "ok",
      data: { record_id: 32, created: true, idempotent_replay: false, stock_warnings: [] },
    });

    const { queryClient } = renderWithClient(<ReturnExchangePage />);
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "返品・交換を保存" }));

    expect(await screen.findByText("返品・交換を保存しました")).toBeInTheDocument();
    const resultRegion = screen.getByRole("region", { name: "保存結果" });
    expect(within(resultRegion).getByText("—")).toBeInTheDocument();
    expect(within(resultRegion).queryByText("備考なし")).not.toBeInTheDocument();
    expect(screen.getAllByText(registerProcessedStockDescription).length).toBeGreaterThan(0);
    await waitFor(() => {
      expectExactInvalidations(
        invalidateSpy.mock.calls,
        d052InvalidationOracle.returnExchange(true),
      );
    });
  });

  it("can add the same product as both return-in and exchange-out rows", async () => {
    const user = userEvent.setup();
    renderWithClient(<ReturnExchangePage />);
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "RT-001", name: "返品商品" })],
        total_count: 1,
        page: 1,
        per_page: 10,
      },
    });

    await user.click(screen.getByLabelText("種別"));
    await user.click(await screen.findByRole("option", { name: "交換" }));
    await user.type(await screen.findByLabelText("返品・交換商品検索"), "RT-001{enter}");
    await user.click(screen.getByLabelText("追加方向"));
    await user.click(await screen.findByRole("option", { name: "渡し" }));
    await user.type(screen.getByLabelText("返品・交換商品検索"), "RT-001{enter}");

    expect(screen.getAllByText("RT-001")).toHaveLength(2);
    expect(screen.getAllByLabelText("RT-001 の数量")).toHaveLength(2);
  });

  it("shows receipt preview and rotates idempotency key after removing an image following failure", async () => {
    const user = userEvent.setup();
    mockSaveReceiptImage.mockResolvedValue({
      status: "ok",
      data: { relative_path: "images/receipts/receipt.png" },
    });
    mockCreateReturn
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
        data: { record_id: 33, created: true, idempotent_replay: false, stock_warnings: [] },
      });

    renderWithClient(<ReturnExchangePage />);
    await addSingleProduct(user);
    const file = new File(["receipt"], "receipt.png", { type: "image/png" });
    dropReceipt(file);

    expect(await screen.findByAltText("選択したレシート画像")).toHaveAttribute(
      "src",
      "blob:receipt-preview",
    );
    await user.click(screen.getByRole("button", { name: "返品・交換を保存" }));
    expect(
      await screen.findByText("一時的なエラー。詳細は診断ログに記録されています。"),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(mockScrollTo).toHaveBeenCalledWith({ top: 0, left: 0, behavior: "smooth" });
    });
    const firstKey = mockCreateReturn.mock.calls[0][0].idempotency_key;

    await user.click(screen.getByRole("button", { name: "レシート画像を削除" }));
    expect(screen.queryByText("receipt.png")).not.toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "返品・交換を保存" }));

    await waitFor(() => {
      expect(mockCreateReturn).toHaveBeenCalledTimes(2);
    });
    expect(mockCreateReturn.mock.calls[1][0].idempotency_key).not.toBe(firstKey);
    expect(mockCreateReturn.mock.calls[1][0].receipt_image_path).toBeNull();
  });

  it("validates rows before saving a receipt image", async () => {
    const user = userEvent.setup();
    mockSaveReceiptImage.mockResolvedValue({
      status: "ok",
      data: { relative_path: "images/receipts/invalid.png" },
    });

    renderWithClient(<ReturnExchangePage />);
    await addSingleProduct(user);
    await user.click(screen.getByLabelText("種別"));
    await user.click(await screen.findByRole("option", { name: "交換" }));
    const file = new File(["receipt"], "invalid.png", { type: "image/png" });
    dropReceipt(file);
    expect(await screen.findByText("invalid.png")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "返品・交換を保存" }));

    expect(
      await screen.findByText("交換では戻り明細と渡し明細がそれぞれ必要です"),
    ).toBeInTheDocument();
    expect(mockSaveReceiptImage).not.toHaveBeenCalled();
    expect(mockCreateReturn).not.toHaveBeenCalled();
  });

  it("rotates idempotency key when adding an image after a create failure", async () => {
    const user = userEvent.setup();
    mockSaveReceiptImage.mockResolvedValue({
      status: "ok",
      data: { relative_path: "images/receipts/added-after-failure.png" },
    });
    mockCreateReturn
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
        data: { record_id: 34, created: true, idempotent_replay: false, stock_warnings: [] },
      });

    renderWithClient(<ReturnExchangePage />);
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "返品・交換を保存" }));
    expect(
      await screen.findByText("一時的なエラー。詳細は診断ログに記録されています。"),
    ).toBeInTheDocument();
    const firstKey = mockCreateReturn.mock.calls[0][0].idempotency_key;

    const file = new File(["receipt"], "added-after-failure.png", { type: "image/png" });
    dropReceipt(file);
    expect(await screen.findByText("added-after-failure.png")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "返品・交換を保存" }));

    await waitFor(() => {
      expect(mockCreateReturn).toHaveBeenCalledTimes(2);
    });
    expect(mockCreateReturn.mock.calls[1][0].idempotency_key).not.toBe(firstKey);
    expect(mockCreateReturn.mock.calls[1][0].receipt_image_path).toBe(
      "images/receipts/added-after-failure.png",
    );
  });

  it("hides product registration recovery link while a save is pending", async () => {
    const user = userEvent.setup();
    const deferred = createDeferred<Awaited<ReturnType<typeof commands.createReturn>>>();
    mockCreateReturn.mockReturnValue(deferred.promise);

    renderWithClient(<ReturnExchangePage />);
    await addSingleProduct(user);
    mockSearchProducts.mockResolvedValueOnce({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 10 },
    });
    await user.type(screen.getByLabelText("返品・交換商品検索"), "NO-HIT{enter}");

    expect(await screen.findByText("該当する商品がありません")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "商品登録へ進む" })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "返品・交換を保存" }));

    expect(await screen.findByRole("button", { name: "保存中..." })).toBeDisabled();
    expect(screen.queryByRole("link", { name: "商品登録へ進む" })).not.toBeInTheDocument();

    deferred.resolve({
      status: "error",
      error: {
        kind: "internal",
        message: "一時的なエラー",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    expect(
      await screen.findByText("一時的なエラー。詳細は診断ログに記録されています。"),
    ).toBeInTheDocument();
  });

  it("retry after create failure reuses saved receipt path without saving the same image again", async () => {
    const user = userEvent.setup();
    mockSaveReceiptImage.mockResolvedValue({
      status: "ok",
      data: { relative_path: "images/receipts/receipt.png" },
    });
    mockCreateReturn
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
        data: { record_id: 31, created: true, idempotent_replay: false, stock_warnings: [] },
      });

    renderWithClient(<ReturnExchangePage />);
    await addSingleProduct(user);
    const file = new File(["receipt"], "receipt.png", { type: "image/png" });
    dropReceipt(file);
    expect(await screen.findByText("receipt.png")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "返品・交換を保存" }));
    expect(
      await screen.findByText("一時的なエラー。詳細は診断ログに記録されています。"),
    ).toBeInTheDocument();
    const firstKey = mockCreateReturn.mock.calls[0][0].idempotency_key;
    await user.click(screen.getByRole("button", { name: "返品・交換を保存" }));

    await waitFor(() => {
      expect(mockCreateReturn).toHaveBeenCalledTimes(2);
    });
    expect(mockSaveReceiptImage).toHaveBeenCalledTimes(1);
    expect(mockCreateReturn.mock.calls[1][0].idempotency_key).toBe(firstKey);
    expect(mockCreateReturn.mock.calls[1][0]).toMatchObject({
      receipt_image_path: "images/receipts/receipt.png",
    });
  });

  it("rotates idempotency key when editing note after a create failure", async () => {
    const user = userEvent.setup();
    mockCreateReturn
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
        data: { record_id: 35, created: true, idempotent_replay: false, stock_warnings: [] },
      });

    renderWithClient(<ReturnExchangePage />);
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "返品・交換を保存" }));
    expect(
      await screen.findByText("一時的なエラー。詳細は診断ログに記録されています。"),
    ).toBeInTheDocument();
    const firstKey = mockCreateReturn.mock.calls[0][0].idempotency_key;

    await user.type(screen.getByLabelText("備考"), "備考を追記");
    await user.click(screen.getByRole("button", { name: "返品・交換を保存" }));

    await waitFor(() => {
      expect(mockCreateReturn).toHaveBeenCalledTimes(2);
    });
    expect(mockCreateReturn.mock.calls[1][0].idempotency_key).not.toBe(firstKey);
    expect(mockCreateReturn.mock.calls[1][0].note).toBe("備考を追記");
  });

  it("keeps return rows fixed to return-in direction", async () => {
    const user = userEvent.setup();
    renderWithClient(<ReturnExchangePage />);
    await addSingleProduct(user);

    // L8-D6: disabled な trigger は開けないため選択肢構成を happy-dom で検証できない
    // （選択肢は「戻り」「渡し」を常時2件描画、到達不能性は disabled に一本化、L3-only）。
    // ここでは disabled 状態そのものだけを assert する。
    expect(screen.getByLabelText("追加方向")).toBeDisabled();
    expect(screen.getByLabelText("RT-001 の方向")).toBeDisabled();
  });

  it("SC10a/SC10b: 種別・追加方向selectはSelect combobox（data-slot=select-trigger）である", async () => {
    const user = userEvent.setup();
    renderWithClient(<ReturnExchangePage />);

    const typeTrigger = screen.getByLabelText("種別");
    expect(typeTrigger).toHaveAttribute("data-slot", "select-trigger");
    expect(typeTrigger.tagName).toBe("BUTTON");

    await user.click(typeTrigger);
    await user.click(await screen.findByRole("option", { name: "交換" }));

    const directionTrigger = screen.getByLabelText("追加方向");
    expect(directionTrigger).toHaveAttribute("data-slot", "select-trigger");
    expect(directionTrigger.tagName).toBe("BUTTON");
    expect(directionTrigger).not.toBeDisabled();

    await user.click(directionTrigger);
    expect(await screen.findByRole("option", { name: "戻り" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "渡し" })).toBeInTheDocument();
  });

  it("SC10c: per-row の方向selectは他行の表示値へ波及しない（L8-D3）", async () => {
    const user = userEvent.setup();
    renderWithClient(<ReturnExchangePage />);

    await user.click(screen.getByLabelText("種別"));
    await user.click(await screen.findByRole("option", { name: "交換" }));
    await addSingleProduct(user);
    mockSearchProducts.mockResolvedValueOnce({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "RT-002", name: "返品商品2" })],
        total_count: 1,
        page: 1,
        per_page: 10,
      },
    });
    await user.type(screen.getByLabelText("返品・交換商品検索"), "RT-002{enter}");
    expect(await screen.findByLabelText("RT-002 の方向")).toBeInTheDocument();

    expect(screen.getByLabelText("RT-001 の方向")).toHaveTextContent("戻り（在庫+）");
    expect(screen.getByLabelText("RT-002 の方向")).toHaveTextContent("戻り（在庫+）");

    // rowKey に direction が含まれるため選択後は行が再マウントされる
    // （key 変化）— DOM 参照を使い回さず選択後に再取得する。
    await user.click(screen.getByLabelText("RT-001 の方向"));
    await user.click(await screen.findByRole("option", { name: "渡し（在庫-）" }));

    expect(screen.getByLabelText("RT-001 の方向")).toHaveTextContent("渡し（在庫-）");
    expect(screen.getByLabelText("RT-002 の方向")).toHaveTextContent("戻り（在庫+）");

    // row 2 自身の select を直接操作しても row 2 だけが変わること
    // （"常に rows[0] を書き換える" mutant を kill する）。
    await user.click(screen.getByLabelText("RT-002 の方向"));
    await user.click(await screen.findByRole("option", { name: "渡し（在庫-）" }));

    expect(screen.getByLabelText("RT-002 の方向")).toHaveTextContent("渡し（在庫-）");
    expect(screen.getByLabelText("RT-001 の方向")).toHaveTextContent("渡し（在庫-）");
  });

  it("Codex round3 P2-class: 種別selectの初期値（返品）はreturn_typeとしてそのまま送信される", async () => {
    const user = userEvent.setup();
    mockCreateReturn.mockResolvedValue({
      status: "ok",
      data: { record_id: 60, created: true, idempotent_replay: false, stock_warnings: [] },
    });

    renderWithClient(<ReturnExchangePage />);
    await addSingleProduct(user);
    await user.click(screen.getByRole("button", { name: "返品・交換を保存" }));
    await waitFor(() => {
      expect(mockCreateReturn).toHaveBeenCalledTimes(1);
    });
    expect(mockCreateReturn.mock.calls[0][0].return_type).toBe("return");
    expect(mockCreateReturn.mock.calls[0][0].items[0]?.direction).toBe("in");
  });

  it("Codex round3 P2-class: 種別selectで交換→返品へ明示的に戻すとreturn_typeがreturnとして送信される（初期値頼みではない検査）", async () => {
    const user = userEvent.setup();
    mockCreateReturn.mockResolvedValue({
      status: "ok",
      data: { record_id: 63, created: true, idempotent_replay: false, stock_warnings: [] },
    });

    renderWithClient(<ReturnExchangePage />);
    await user.click(screen.getByLabelText("種別"));
    await user.click(await screen.findByRole("option", { name: "交換" }));
    await addSingleProduct(user);

    // 交換 → 返品へ明示的に戻す（渡し行が無いため hasOut ガードに抵触しない）。
    await user.click(screen.getByLabelText("種別"));
    await user.click(await screen.findByRole("option", { name: "返品" }));
    await user.click(screen.getByRole("button", { name: "返品・交換を保存" }));

    await waitFor(() => {
      expect(mockCreateReturn).toHaveBeenCalledTimes(1);
    });
    expect(mockCreateReturn.mock.calls[0][0].return_type).toBe("return");
  });

  it("Codex round3 P2-class: 種別selectで交換へ切り替えるとreturn_typeがexchangeとして、追加方向の戻り・渡しがどちらもitems.directionとして送信される", async () => {
    const user = userEvent.setup();
    mockCreateReturn.mockResolvedValue({
      status: "ok",
      data: { record_id: 61, created: true, idempotent_replay: false, stock_warnings: [] },
    });

    renderWithClient(<ReturnExchangePage />);
    await user.click(screen.getByLabelText("種別"));
    await user.click(await screen.findByRole("option", { name: "交換" }));

    await user.click(screen.getByLabelText("追加方向"));
    await user.click(await screen.findByRole("option", { name: "戻り" }));
    await addSingleProduct(user);

    mockSearchProducts.mockResolvedValueOnce({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "RT-002", name: "返品商品2" })],
        total_count: 1,
        page: 1,
        per_page: 10,
      },
    });
    await user.click(screen.getByLabelText("追加方向"));
    await user.click(await screen.findByRole("option", { name: "渡し" }));
    await user.type(screen.getByLabelText("返品・交換商品検索"), "RT-002{enter}");
    expect(await screen.findByText("RT-002")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "返品・交換を保存" }));
    await waitFor(() => {
      expect(mockCreateReturn).toHaveBeenCalledTimes(1);
    });
    const request = mockCreateReturn.mock.calls[0][0];
    expect(request.return_type).toBe("exchange");
    expect(request.items.find((item) => item.product_code === "RT-001")?.direction).toBe("in");
    expect(request.items.find((item) => item.product_code === "RT-002")?.direction).toBe("out");
  });

  it("Codex round4 P2-class: 追加方向selectで渡しへ切り替えてから戻りへ戻すとRT-001のdirectionがinとして送信される（初期値頼みではない検査）", async () => {
    const user = userEvent.setup();
    mockCreateReturn.mockResolvedValue({
      status: "ok",
      data: { record_id: 64, created: true, idempotent_replay: false, stock_warnings: [] },
    });

    renderWithClient(<ReturnExchangePage />);
    await user.click(screen.getByLabelText("種別"));
    await user.click(await screen.findByRole("option", { name: "交換" }));

    // 初期値(戻り)のままでは Radix が onValueChange を発火しないため、
    // 一度「渡し」へ切り替えてから「戻り」へ戻す実操作を経由させてから RT-001 を追加する。
    await user.click(screen.getByLabelText("追加方向"));
    await user.click(await screen.findByRole("option", { name: "渡し" }));
    await user.click(screen.getByLabelText("追加方向"));
    await user.click(await screen.findByRole("option", { name: "戻り" }));
    await addSingleProduct(user);

    // 交換は戻り明細と渡し明細の両方が必要なため RT-002 を渡しで追加する。
    mockSearchProducts.mockResolvedValueOnce({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "RT-002", name: "返品商品2" })],
        total_count: 1,
        page: 1,
        per_page: 10,
      },
    });
    await user.click(screen.getByLabelText("追加方向"));
    await user.click(await screen.findByRole("option", { name: "渡し" }));
    await user.type(screen.getByLabelText("返品・交換商品検索"), "RT-002{enter}");
    expect(await screen.findByText("RT-002")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "返品・交換を保存" }));
    await waitFor(() => {
      expect(mockCreateReturn).toHaveBeenCalledTimes(1);
    });
    const request = mockCreateReturn.mock.calls[0][0];
    expect(request.items.find((item) => item.product_code === "RT-001")?.direction).toBe("in");
  });

  it("Codex round3 P2-class: per-row の方向selectの戻り・渡しがどちらもitems.directionとしてそのまま送信される", async () => {
    const user = userEvent.setup();
    mockCreateReturn.mockResolvedValue({
      status: "ok",
      data: { record_id: 62, created: true, idempotent_replay: false, stock_warnings: [] },
    });

    renderWithClient(<ReturnExchangePage />);
    await user.click(screen.getByLabelText("種別"));
    await user.click(await screen.findByRole("option", { name: "交換" }));
    await addSingleProduct(user);

    mockSearchProducts.mockResolvedValueOnce({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "RT-002", name: "返品商品2" })],
        total_count: 1,
        page: 1,
        per_page: 10,
      },
    });
    await user.type(screen.getByLabelText("返品・交換商品検索"), "RT-002{enter}");
    expect(await screen.findByLabelText("RT-002 の方向")).toBeInTheDocument();

    // RT-001: 「渡し（在庫-）」を明示選択（最終状態 out）。
    await user.click(screen.getByLabelText("RT-001 の方向"));
    await user.click(await screen.findByRole("option", { name: "渡し（在庫-）" }));

    // RT-002: 「渡し（在庫-）」→「戻り（在庫+）」の順に明示選択（最終状態 in、両 option を経由）。
    await user.click(screen.getByLabelText("RT-002 の方向"));
    await user.click(await screen.findByRole("option", { name: "渡し（在庫-）" }));
    await user.click(screen.getByLabelText("RT-002 の方向"));
    await user.click(await screen.findByRole("option", { name: "戻り（在庫+）" }));

    await user.click(screen.getByRole("button", { name: "返品・交換を保存" }));
    await waitFor(() => {
      expect(mockCreateReturn).toHaveBeenCalledTimes(1);
    });
    const items = mockCreateReturn.mock.calls[0][0].items;
    expect(items.find((item) => item.product_code === "RT-001")?.direction).toBe("out");
    expect(items.find((item) => item.product_code === "RT-002")?.direction).toBe("in");
  });
});

describe("ReturnExchangePage native input tokens（Lane 5 SC4d）", () => {
  it("SC4d: 種別/備考/追加方向/方向の4箇所すべてがbg-control-surfaceでbg-backgroundを持たない、registerOptionClassラベルは不変", async () => {
    const user = userEvent.setup();
    renderWithClient(<ReturnExchangePage />);
    await addSingleProduct(user);

    const fields = [
      screen.getByLabelText("種別"),
      screen.getByLabelText("備考"),
      screen.getByLabelText("追加方向"),
      screen.getByLabelText("RT-001 の方向"),
    ];
    for (const field of fields) {
      expect(field).toHaveClass("bg-control-surface");
      expect(field).not.toHaveClass("bg-background");
    }

    // registerOptionClass のラジオ選択肢ラベルは対象外、bg-background のまま不変。
    const registerLabel = screen.getByLabelText("レジ未処理").closest("label");
    expect(registerLabel).not.toBeNull();
    expect(registerLabel).toHaveClass("bg-background");
  });
});

// SPEC-COLOR-EMPHASIS-RT-1 / D-CE17 / S21: 登録方法の選択カードは選ぶと枠が操作の色で focus の前後に
// 変わらないため、focus は不透明な ring。radio の点は操作の色（accent-primary）。
it("D-CE17 / S21: register option cards use the opaque focus ring and the operation-colored radio", () => {
  renderWithClient(<ReturnExchangePage />);
  const processed = screen.getByLabelText("レジ戻し済み");
  const unprocessed = screen.getByLabelText("レジ未処理");
  expect(processed).toBeChecked();
  for (const radio of [processed, unprocessed]) {
    expect(radio).toHaveClass("accent-primary");
    const card = radio.closest("label");
    expect(card).toHaveClass("focus-within:ring-ring");
    expect(card).not.toHaveClass("focus-within:ring-ring/50");
  }
  expect(processed.closest("label")).toHaveClass("border-primary", "bg-primary/5");
});

it("⑮ SC3/SC14: 副題と直近件数の説明を表示する", () => {
  renderWithClient(<ReturnExchangePage />);
  expect(
    screen.getByText("レジ戻し済みなら帳面記録だけ、未処理ならこの保存で在庫を反映します"),
  ).toBeInTheDocument();
  expect(screen.getByText("直近 10 件の返品・交換を新しい順に表示します。")).toBeInTheDocument();
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
    renderWithClient(<ReturnExchangePage />);
    await user.type(await screen.findByLabelText("返品・交換商品検索"), "単位{enter}");
    const candidate = (await screen.findByText("UNIT-001")).closest("tr");
    if (candidate === null) throw new Error("expected table structure");
    expect(within(candidate).getByText(display)).toBeInTheDocument();
    await user.click(within(candidate).getByRole("button", { name: "戻りに追加" }));
    const inputRow = (await screen.findByLabelText("UNIT-001 の数量")).closest("tr");
    if (inputRow === null) throw new Error("expected table structure");
    // SC19 Amendment 3: 入力表の列順と、数量・単位が同じ cell にある契約を固定する。
    const inputTable = inputRow.closest("table");
    if (inputTable === null) throw new Error("expected table structure");
    expect(
      within(inputTable)
        .getAllByRole("columnheader")
        .map((cell) => cell.textContent),
    ).toEqual(["商品コード", "商品名", "部門", "現在庫", "方向", "数量", "操作"]);
    const cells = within(inputRow).getAllByRole("cell");
    expect(cells[3].textContent).toBe(display);
    expect(
      within(cells[5]).getByRole("spinbutton", { name: "UNIT-001 の数量" }),
    ).toBeInTheDocument();
    expect(within(cells[5]).getByText(unitLabel)).toBeInTheDocument();
  },
);

it("⑰ SC6 / UIDISP-D6: 共有 formatDateTime を import しローカル定義を持たない", () => {
  expect(dateTimeSource).toMatch(
    /import\s*\{[^}]*\bformatDateTime\b[^}]*\}\s*from\s*"@\/lib\/date-time"/,
  );
  expect(dateTimeSource).not.toMatch(/function\s+(?:formatDateTime|formatCheckedAt)\s*\(/);
  expect(dateTimeSource).not.toContain("formatCheckedAt");
});

// REQ-202 / UI-03-D12 / SPEC-UNIT-D3: 数量欄の属性は m の商品の行だけ小数 2 桁の入力にする
it.each([
  ["m", { inputmode: "decimal", min: "0.01", step: "0.01" }],
  ["pcs", { inputmode: "numeric", min: "1", step: null }],
] as const)("UI-03-D12: %s の商品の数量欄の属性", async (unit, expected) => {
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
  renderWithClient(<ReturnExchangePage />);
  await user.type(await screen.findByLabelText("返品・交換商品検索"), "属性{enter}");
  const candidate = (await screen.findByText("ATTR-001")).closest("tr");
  if (candidate === null) throw new Error("expected table structure");
  await user.click(within(candidate).getByRole("button", { name: "戻りに追加" }));
  const input = await screen.findByLabelText("ATTR-001 の数量");
  expect(input).toHaveValue(1);
  expect(input).toHaveAttribute("type", "number");
  expect(input.getAttribute("inputmode")).toBe(expected.inputmode);
  expect(input.getAttribute("min")).toBe(expected.min);
  expect(input.getAttribute("step")).toBe(expected.step);
});
