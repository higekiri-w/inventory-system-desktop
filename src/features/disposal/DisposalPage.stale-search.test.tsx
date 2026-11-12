import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { makeMockProductWithRelations } from "@/features/products/lib/test-fixtures";
import { commands } from "@/lib/bindings";
import type { ProductWithRelations } from "@/lib/bindings";
import { DisposalPage } from "./DisposalPage";

// 関門 13 の lane（UI-05-D15）: 保存の開始・リセット・「続けて」の前に始めた直接検索の応答は捨て、
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
    select({ location: { href: "/inventory/disposal" } }),
}));

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn(), dismiss: vi.fn() },
}));

vi.mock("@/lib/bindings", () => ({
  commands: {
    listDisposals: vi.fn(),
    searchProducts: vi.fn(),
    createDisposal: vi.fn(),
  },
}));

type SearchResponse = Awaited<ReturnType<typeof commands.searchProducts>>;
type CreateResponse = Awaited<ReturnType<typeof commands.createDisposal>>;

const mockSearchProducts = vi.mocked(commands.searchProducts);
const mockCreate = vi.mocked(commands.createDisposal);
const mockListDisposals = vi.mocked(commands.listDisposals);

const SEARCH_LABEL = "廃棄・破損商品検索";
const SAVE_LABEL = "廃棄・破損を保存";
const RESULT_TITLE = "廃棄・破損を保存しました";
const CONTINUE_LABEL = "続けて廃棄・破損";
const FIRST_CODE = "DP-001";
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

const firstProduct = makeMockProductWithRelations({
  product_code: "DP-001",
  name: "廃棄確認 商品",
  cost_price_centi: 12000,
  stock_quantity: 4,
});
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
      <DisposalPage />
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
  mockListDisposals.mockResolvedValue({
    status: "ok",
    data: { items: [], total_count: 0, page: 1, per_page: 10 },
  });
});

describe("DisposalPage stale direct search (UI-05-D15 / REQ-204)", () => {
  it("REQ-204 UI-05-D15 S1 late search success after save success keeps the result panel and allows no second create", async () => {
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

  it("REQ-204 UI-05-D15 S2 late search success while saving is not added", async () => {
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

  it("REQ-204 UI-05-D15 S3 late search success after continue is not added and not in the next payload", async () => {
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

  it("REQ-204 UI-05-D15 S4 late search success after reset is not added", async () => {
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

  it("REQ-204 UI-05-D15 S5 late search failure after reset shows no message", async () => {
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

  it("REQ-204 UI-05-D15 S6 search started before a failed save is discarded and a new search is added", async () => {
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

  it("REQ-204 UI-05-D15 S7 late multi-candidate response after reset shows nothing", async () => {
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

  it("REQ-204 UI-05-D15 S7 late no-match response after reset shows nothing", async () => {
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

  it("REQ-204 UI-05-D15 S8 a search started after reset is added even if the older one resolves later", async () => {
    const user = userEvent.setup();
    renderPage();
    await addFirstProduct(user);
    const late = holdSearch("LATE-001");
    await searchFor(user, "LATE-001");

    await user.click(screen.getByRole("button", { name: "リセット" }));
    // catalog ⑮ D11: 数字確定（compositionend）の経路で新しい検索を始める。
    const fresh = holdSearch("45678");
    fireEvent.compositionEnd(screen.getByLabelText(SEARCH_LABEL), {
      target: { value: "４５６７８" },
    });
    await waitFor(() => {
      expect(mockSearchProducts).toHaveBeenCalledWith(
        expect.objectContaining({ keyword: "45678" }),
      );
    });
    fresh.resolve(
      searchResult([makeMockProductWithRelations({ product_code: "NEW-001", name: NEW_NAME })]),
    );
    expect(await screen.findByText(NEW_NAME)).toBeInTheDocument();
    late.resolve(lateOk);
    await settle();

    expect(screen.queryByText(LATE_NAME)).toBeNull();
    expect(screen.getByText(NEW_NAME)).toBeInTheDocument();
  });
});
