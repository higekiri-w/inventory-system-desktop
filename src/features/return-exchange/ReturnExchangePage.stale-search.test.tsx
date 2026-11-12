import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { makeMockProductWithRelations } from "@/features/products/lib/test-fixtures";
import { commands } from "@/lib/bindings";
import type { ProductWithRelations } from "@/lib/bindings";
import { ReturnExchangePage } from "./ReturnExchangePage";

// 関門 13 の lane（UI-03-D24）: 保存の開始・リセット・「続けて」の前に始めた直接検索の応答は捨て、
// その後に始めた検索は採る。遅れた応答は deferred の promise で順を固定する（Test Plan の S1〜S8）。

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

type SearchResponse = Awaited<ReturnType<typeof commands.searchProducts>>;
type CreateResponse = Awaited<ReturnType<typeof commands.createReturn>>;

const mockSearchProducts = vi.mocked(commands.searchProducts);
const mockCreate = vi.mocked(commands.createReturn);
const mockListReturns = vi.mocked(commands.listReturns);
const mockSaveReceiptImage = vi.mocked(commands.saveReceiptImage);

const SEARCH_LABEL = "返品・交換商品検索";
const SAVE_LABEL = "返品・交換を保存";
const RESULT_TITLE = "返品・交換を保存しました";
const CONTINUE_LABEL = "続けて返品・交換";
const FIRST_CODE = "RT-001";
const LATE_NAME = "遅れて届いた商品";
const NEW_NAME = "新しく読んだ商品";

const saveOk: CreateResponse = {
  status: "ok",
  data: { record_id: 90, created: true, idempotent_replay: false, stock_warnings: [] },
};
const saveFailed: CreateResponse = {
  status: "error",
  error: {
    kind: "internal",
    message: "一時的なエラー",
    field: null,
    error_id: null,
    stocktake_recovery: null,
  },
};

function searchResult(items: ProductWithRelations[]): SearchResponse {
  return { status: "ok", data: { items, total_count: items.length, page: 1, per_page: 10 } };
}

const firstProduct = makeMockProductWithRelations({ product_code: "RT-001", name: "返品商品" });
const lateOk = searchResult([
  makeMockProductWithRelations({ product_code: "LATE-001", name: LATE_NAME }),
]);
const lateFailure: SearchResponse = {
  status: "error",
  error: {
    kind: "internal",
    message: "遅れた検索の失敗",
    field: null,
    error_id: null,
    stocktake_recovery: null,
  },
};

function createDeferred<T>() {
  let resolve: (value: T) => void = () => {
    throw new Error("deferred promise is not initialized");
  };
  const promise = new Promise<T>((promiseResolve) => {
    resolve = promiseResolve;
  });
  return { promise, resolve };
}

const heldSearches = new Map<string, ReturnType<typeof createDeferred<SearchResponse>>>();

function holdSearch(keyword: string) {
  const deferred = createDeferred<SearchResponse>();
  heldSearches.set(keyword, deferred);
  return deferred;
}

function renderPage() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <ReturnExchangePage />
    </QueryClientProvider>,
  );
}

async function searchFor(user: ReturnType<typeof userEvent.setup>, keyword: string) {
  const input = await screen.findByLabelText(SEARCH_LABEL);
  await user.clear(input);
  await user.type(input, `${keyword}{enter}`);
  await waitFor(() => {
    expect(mockSearchProducts).toHaveBeenCalledWith(expect.objectContaining({ keyword }));
  });
}

async function addFirstProduct(user: ReturnType<typeof userEvent.setup>) {
  await searchFor(user, FIRST_CODE);
  expect(await screen.findByText(FIRST_CODE)).toBeInTheDocument();
}

async function settle() {
  await act(async () => {
    await new Promise((resolve) => setTimeout(resolve, 30));
  });
}

function saveButton() {
  return screen.getByRole("button", { name: SAVE_LABEL });
}

beforeEach(() => {
  vi.stubGlobal("scrollTo", vi.fn());
  heldSearches.clear();
  mockSearchProducts.mockReset();
  mockSearchProducts.mockImplementation((query) => {
    const held = heldSearches.get(query.keyword ?? "");
    if (held !== undefined) return held.promise;
    return Promise.resolve(searchResult([firstProduct]));
  });
  mockCreate.mockReset();
  vi.spyOn(URL, "createObjectURL").mockReturnValue("blob:receipt-preview");
  mockSaveReceiptImage.mockReset();
  mockListReturns.mockResolvedValue({
    status: "ok",
    data: { items: [], total_count: 0, page: 1, per_page: 10 },
  });
});

describe("ReturnExchangePage stale direct search (UI-03-D24 / REQ-202)", () => {
  it("REQ-202 UI-03-D24 S1 late search success after save success keeps the result panel and allows no second create", async () => {
    const user = userEvent.setup();
    mockCreate.mockResolvedValue(saveOk);
    renderPage();
    await addFirstProduct(user);
    const late = holdSearch("LATE-001");
    await searchFor(user, "LATE-001");

    await user.click(saveButton());
    expect(await screen.findByText(RESULT_TITLE)).toBeInTheDocument();
    late.resolve(lateOk);
    await settle();

    expect(screen.queryByText(LATE_NAME)).toBeNull();
    expect(screen.getByText(RESULT_TITLE)).toBeInTheDocument();
    expect(saveButton()).toBeDisabled();
    expect(mockCreate).toHaveBeenCalledTimes(1);
  });

  it("REQ-202 UI-03-D24 S2 late search success while saving is not added", async () => {
    const user = userEvent.setup();
    const save = createDeferred<CreateResponse>();
    mockCreate.mockReturnValue(save.promise);
    renderPage();
    await addFirstProduct(user);
    const late = holdSearch("LATE-001");
    await searchFor(user, "LATE-001");

    await user.click(saveButton());
    await waitFor(() => {
      expect(mockCreate).toHaveBeenCalledTimes(1);
    });
    late.resolve(lateOk);
    await settle();
    expect(screen.queryByText(LATE_NAME)).toBeNull();

    save.resolve(saveOk);
    expect(await screen.findByText(RESULT_TITLE)).toBeInTheDocument();
    await settle();
    expect(screen.queryByText(LATE_NAME)).toBeNull();
    expect(mockCreate).toHaveBeenCalledTimes(1);
  });

  it("REQ-202 UI-03-D24 S3 late search success after continue is not added and not in the next payload", async () => {
    const user = userEvent.setup();
    mockCreate.mockResolvedValue(saveOk);
    renderPage();
    await addFirstProduct(user);
    const late = holdSearch("LATE-001");
    await searchFor(user, "LATE-001");

    await user.click(saveButton());
    expect(await screen.findByText(RESULT_TITLE)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: CONTINUE_LABEL }));
    late.resolve(lateOk);
    await settle();
    expect(screen.queryByText(LATE_NAME)).toBeNull();

    await addFirstProduct(user);
    await user.click(saveButton());
    await waitFor(() => {
      expect(mockCreate).toHaveBeenCalledTimes(2);
    });
    const secondPayload = JSON.stringify(mockCreate.mock.calls[1][0]);
    expect(secondPayload).toContain(FIRST_CODE);
    expect(secondPayload).not.toContain("LATE-001");
  });

  it("REQ-202 UI-03-D24 S4 late search success after reset is not added", async () => {
    const user = userEvent.setup();
    renderPage();
    await addFirstProduct(user);
    const late = holdSearch("LATE-001");
    await searchFor(user, "LATE-001");

    await user.click(screen.getByRole("button", { name: "リセット" }));
    expect(screen.queryByText(FIRST_CODE)).toBeNull();
    late.resolve(lateOk);
    await settle();

    expect(screen.queryByText(LATE_NAME)).toBeNull();
  });

  it("REQ-202 UI-03-D24 S5 late search failure after reset shows no message", async () => {
    const user = userEvent.setup();
    renderPage();
    await addFirstProduct(user);
    const late = holdSearch("LATE-001");
    await searchFor(user, "LATE-001");

    await user.click(screen.getByRole("button", { name: "リセット" }));
    late.resolve(lateFailure);
    await settle();

    expect(screen.queryByText(/遅れた検索の失敗/)).toBeNull();
  });

  it("REQ-202 UI-03-D24 S6 search started before a failed save is discarded and a new search is added", async () => {
    const user = userEvent.setup();
    mockCreate.mockResolvedValue(saveFailed);
    renderPage();
    await addFirstProduct(user);
    const late = holdSearch("LATE-001");
    await searchFor(user, "LATE-001");

    await user.click(saveButton());
    expect(await screen.findByText("保存に失敗しました")).toBeInTheDocument();
    late.resolve(lateOk);
    await settle();
    expect(screen.queryByText(LATE_NAME)).toBeNull();

    const fresh = holdSearch("NEW-001");
    await searchFor(user, "NEW-001");
    fresh.resolve(
      searchResult([makeMockProductWithRelations({ product_code: "NEW-001", name: NEW_NAME })]),
    );
    expect(await screen.findByText(NEW_NAME)).toBeInTheDocument();
    expect(screen.queryByText(LATE_NAME)).toBeNull();
  });

  it("REQ-202 UI-03-D24 S6 a new search is added after the save is rejected by frontend validation", async () => {
    const user = userEvent.setup();
    renderPage();
    await addFirstProduct(user);
    await user.click(screen.getByLabelText("種別"));
    await user.click(await screen.findByRole("option", { name: "交換" }));

    await user.click(saveButton());
    expect(
      await screen.findByText("交換では戻り明細と渡し明細がそれぞれ必要です"),
    ).toBeInTheDocument();

    const fresh = holdSearch("NEW-001");
    await searchFor(user, "NEW-001");
    fresh.resolve(
      searchResult([makeMockProductWithRelations({ product_code: "NEW-001", name: NEW_NAME })]),
    );
    expect(await screen.findByText(NEW_NAME)).toBeInTheDocument();
    expect(mockCreate).not.toHaveBeenCalled();
  });

  it("REQ-202 UI-03-D24 S7 late multi-candidate response after reset shows nothing", async () => {
    const user = userEvent.setup();
    renderPage();
    await addFirstProduct(user);
    const late = holdSearch("LATE-001");
    await searchFor(user, "LATE-001");

    await user.click(screen.getByRole("button", { name: "リセット" }));
    late.resolve(
      searchResult([
        makeMockProductWithRelations({ product_code: "LATE-001", name: LATE_NAME }),
        makeMockProductWithRelations({ product_code: "LATE-002", name: "遅れて届いた候補" }),
      ]),
    );
    await settle();

    expect(screen.queryByText(LATE_NAME)).toBeNull();
    expect(screen.queryByText("遅れて届いた候補")).toBeNull();
    expect(screen.queryByText(/候補から/)).toBeNull();
  });

  it("REQ-202 UI-03-D24 S7 late no-match response after reset shows nothing", async () => {
    const user = userEvent.setup();
    renderPage();
    await addFirstProduct(user);
    const late = holdSearch("LATE-001");
    await searchFor(user, "LATE-001");

    await user.click(screen.getByRole("button", { name: "リセット" }));
    late.resolve(searchResult([]));
    await settle();

    expect(screen.queryByText("該当する商品がありません")).toBeNull();
  });

  it("REQ-202 UI-03-D24 S8 a search started after reset is added even if the older one resolves later", async () => {
    const user = userEvent.setup();
    renderPage();
    await addFirstProduct(user);
    const late = holdSearch("LATE-001");
    await searchFor(user, "LATE-001");

    await user.click(screen.getByRole("button", { name: "リセット" }));
    const fresh = holdSearch("NEW-001");
    await searchFor(user, "NEW-001");
    fresh.resolve(
      searchResult([makeMockProductWithRelations({ product_code: "NEW-001", name: NEW_NAME })]),
    );
    expect(await screen.findByText(NEW_NAME)).toBeInTheDocument();
    late.resolve(lateOk);
    await settle();

    expect(screen.queryByText(LATE_NAME)).toBeNull();
    expect(screen.getByText(NEW_NAME)).toBeInTheDocument();
  });

  it("REQ-202 UI-03-D24 RX1 exchange-out search resolved after switching to return adds an in row", async () => {
    const user = userEvent.setup();
    mockCreate.mockResolvedValue(saveOk);
    renderPage();
    await user.click(screen.getByLabelText("種別"));
    await user.click(await screen.findByRole("option", { name: "交換" }));
    await user.click(screen.getByLabelText("追加方向"));
    await user.click(await screen.findByRole("option", { name: "渡し" }));
    const late = holdSearch("LATE-001");
    await searchFor(user, "LATE-001");

    await user.click(screen.getByLabelText("種別"));
    await user.click(await screen.findByRole("option", { name: "返品" }));
    late.resolve(lateOk);
    expect(await screen.findByText(LATE_NAME)).toBeInTheDocument();

    await user.click(saveButton());
    await waitFor(() => {
      expect(mockCreate).toHaveBeenCalledTimes(1);
    });
    expect(mockCreate.mock.calls[0][0].items).toEqual([
      expect.objectContaining({ product_code: "LATE-001", direction: "in" }),
    ]);
  });

  it("REQ-202 UI-03-D24 RX2 direction at Enter is used when the add direction changes before the response", async () => {
    const user = userEvent.setup();
    renderPage();
    await user.click(screen.getByLabelText("種別"));
    await user.click(await screen.findByRole("option", { name: "交換" }));
    const late = holdSearch("LATE-001");
    await searchFor(user, "LATE-001");

    await user.click(screen.getByLabelText("追加方向"));
    await user.click(await screen.findByRole("option", { name: "渡し" }));
    late.resolve(lateOk);
    expect(await screen.findByText(LATE_NAME)).toBeInTheDocument();

    expect(screen.getByLabelText("LATE-001 の方向")).toHaveTextContent("戻り（在庫+）");
  });

  it("REQ-202 UI-03-D24 RX3 late receipt image after save success keeps the result panel and allows no second create", async () => {
    const user = userEvent.setup();
    mockCreate.mockResolvedValue(saveOk);
    renderPage();
    await addFirstProduct(user);
    const bytes = createDeferred<ArrayBuffer>();
    const file = new File(["receipt"], "late-receipt.png", { type: "image/png" });
    file.arrayBuffer = () => bytes.promise;
    fireEvent.drop(screen.getByTestId("file-picker-dropzone"), {
      dataTransfer: { files: [file] },
    });

    await user.click(saveButton());
    expect(await screen.findByText(RESULT_TITLE)).toBeInTheDocument();
    bytes.resolve(new ArrayBuffer(7));
    await settle();

    expect(screen.getByText(RESULT_TITLE)).toBeInTheDocument();
    expect(saveButton()).toBeDisabled();
    expect(screen.queryByAltText("選択したレシート画像")).toBeNull();
    expect(mockCreate).toHaveBeenCalledTimes(1);
    expect(mockCreate.mock.calls[0][0].receipt_image_path).toBeNull();
    expect(mockSaveReceiptImage).not.toHaveBeenCalled();
  });
});
