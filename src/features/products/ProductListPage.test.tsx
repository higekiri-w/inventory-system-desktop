// src/features/products/ProductListPage.test.tsx
//
// UI-01a: 商品検索・一覧 page integration。

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState, type ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { commands } from "@/lib/bindings";
import { scrollPageToTop } from "@/lib/page-scroll";
import { toast } from "sonner";
import { d052InvalidationOracle, expectExactInvalidations } from "@/test/invalidation-oracle";
import { makeMockDepartment, makeMockProductWithRelations } from "./lib/test-fixtures";
import { ProductListPage } from "./ProductListPage";
import type { ProductListSearch } from "./search";

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
    const resolvedTo = params?.code !== undefined ? to.replace("$code", params.code) : to;
    const query =
      search?.returnTo !== undefined ? `?returnTo=${encodeURIComponent(search.returnTo)}` : "";
    return <a href={`${resolvedTo}${query}`}>{children}</a>;
  },
}));

vi.mock("@/lib/bindings", () => ({
  commands: {
    searchProducts: vi.fn(),
    listDepartments: vi.fn(),
    listSuppliers: vi.fn(),
    bulkSetPluTarget: vi.fn(),
  },
}));
vi.mock("@/lib/page-scroll", () => ({ scrollPageToTop: vi.fn() }));
vi.mock("sonner", () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const mockSearchProducts = vi.mocked(commands.searchProducts);
const mockListDepartments = vi.mocked(commands.listDepartments);
const mockBulkSetPluTarget = vi.mocked(commands.bulkSetPluTarget);
const mockScrollPageToTop = vi.mocked(scrollPageToTop);
const mockToastSuccess = vi.mocked(toast.success);

function renderWithClient(ui: ReactNode) {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  return render(<QueryClientProvider client={qc}>{ui}</QueryClientProvider>);
}

beforeEach(() => {
  mockSearchProducts.mockReset();
  mockListDepartments.mockReset();
  mockBulkSetPluTarget.mockReset();
  mockScrollPageToTop.mockReset();
  mockToastSuccess.mockReset();
});

describe("ProductListPage (UI-01a)", () => {
  it("REQ-907 B-V2: restores the PLU filter selection and sends the normalized search payload", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "PLU-FILTER" })],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });
    mockListDepartments.mockResolvedValue({ status: "ok", data: [] });

    renderWithClient(
      <ProductListPage search={{ discontinued: "all", plu: "pending" }} onSearchChange={vi.fn()} />,
    );

    await screen.findByText("PLU-FILTER");
    // SPEC-FILTER-LABEL-RT-1 D-RT2: toolbar の3群に可視ラベルが結線される。
    for (const name of ["廃番表示", "PLU表示", "並び順"]) {
      const group = screen.getByRole("group", { name });
      const label = screen.getByText(name);
      expect(label).toBeVisible();
      expect(group).toHaveAttribute("aria-labelledby", label.id);
    }
    // SC15 / AC-L3-3: 上置き Label の検索欄と隣接 control の下辺を揃える。
    const toolbar = screen.getByText("商品を検索", { selector: "label" }).parentElement
      ?.parentElement;
    expect(toolbar).toHaveClass("items-end");
    expect(toolbar).not.toHaveClass("items-center");
    expect(screen.getByRole("button", { name: "未反映" })).toHaveAttribute("aria-pressed", "true");
    expect(mockSearchProducts).toHaveBeenCalledWith({
      keyword: null,
      department_id: null,
      is_discontinued: null,
      plu: "pending",
      sort_key: "ProductCode",
      sort_order: "Asc",
      page: 1,
      per_page: 100,
    });
  });

  it("REQ-907 B-V3: confirms filter-wide bulk target, toasts counts, and invalidates C19", async () => {
    const user = userEvent.setup();
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "PLU-BULK" })],
        total_count: 37,
        page: 1,
        per_page: 50,
      },
    });
    mockListDepartments.mockResolvedValue({ status: "ok", data: [makeMockDepartment({ id: 2 })] });
    mockBulkSetPluTarget.mockResolvedValue({
      status: "ok",
      data: {
        matched_count: 37,
        updated_count: 31,
        invalid_jan_skipped_count: 4,
        discontinued_skipped_count: 2,
      },
    });
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
    });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    render(
      <QueryClientProvider client={queryClient}>
        <ProductListPage
          search={{ q: "  糸  ", dept: 2, discontinued: "all", plu: "pending" }}
          onSearchChange={vi.fn()}
        />
      </QueryClientProvider>,
    );
    await screen.findByText("PLU-BULK");
    await user.click(screen.getByRole("button", { name: "PLU 対象にする" }));
    expect(screen.getByText("絞り込みに一致する商品をPLU対象にしますか")).toBeInTheDocument();
    expect(screen.getByText(/現在の絞り込み条件に一致する 37 件が対象です/)).toBeInTheDocument();
    expect(screen.getByText(/現在の絞り込みに一致しない商品は変更されません/)).toBeInTheDocument();
    expect(screen.getByText(/PLU 書出しと PC ツールの取込みが別途必要です/)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "キャンセル" }));
    expect(mockBulkSetPluTarget).not.toHaveBeenCalled();

    await user.click(screen.getByRole("button", { name: "PLU 対象にする" }));
    const dialog = screen.getByRole("alertdialog");
    expect(within(dialog).getByRole("button", { name: "PLU 対象にする" })).toHaveAttribute(
      "data-variant",
      "default",
    );
    await user.click(within(dialog).getByRole("button", { name: "PLU 対象にする" }));
    await waitFor(() => {
      expect(mockBulkSetPluTarget).toHaveBeenCalledWith(
        { keyword: "糸", department_id: 2, is_discontinued: null, plu: "pending" },
        true,
      );
    });
    expect(mockToastSuccess).toHaveBeenCalledWith(
      "31 件を更新しました（JAN 不備 4 件 / 廃番 2 件は対象外）",
    );
    expectExactInvalidations(invalidateSpy.mock.calls, d052InvalidationOracle.pluBulkTarget());
  });

  it("REQ-907 B-V3: sends false for exclusion and shows a destructive failure alert", async () => {
    const user = userEvent.setup();
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: { items: [makeMockProductWithRelations()], total_count: 1, page: 1, per_page: 50 },
    });
    mockListDepartments.mockResolvedValue({ status: "ok", data: [] });
    mockBulkSetPluTarget.mockResolvedValue({
      status: "error",
      error: {
        kind: "internal",
        message: "synthetic failure",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    renderWithClient(<ProductListPage search={{ plu: "synced" }} onSearchChange={vi.fn()} />);
    await screen.findByText("P-0001");
    await user.click(screen.getByRole("button", { name: "PLU 対象から外す" }));
    const dialog = screen.getByRole("alertdialog");
    expect(
      within(dialog).getByText("絞り込みに一致する商品をPLU対象から外しますか"),
    ).toBeInTheDocument();
    await user.click(within(dialog).getByRole("button", { name: "PLU 対象から外す" }));
    await waitFor(() => {
      expect(mockBulkSetPluTarget).toHaveBeenCalledWith(
        { keyword: null, department_id: null, is_discontinued: false, plu: "synced" },
        false,
      );
    });
    expect(await screen.findByText("PLU対象の一括更新に失敗しました")).toBeInTheDocument();
  });

  it("renders active product list with department master options", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-001", name: "はさみ" })],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });
    mockListDepartments.mockResolvedValue({
      status: "ok",
      data: [
        makeMockDepartment({ id: 1, name: "毛糸" }),
        makeMockDepartment({ id: 2, name: "布" }),
      ],
    });

    renderWithClient(<ProductListPage search={{}} onSearchChange={vi.fn()} />);

    expect(screen.getByRole("heading", { name: "商品検索・一覧" })).toBeInTheDocument();
    expect(await screen.findByText("P-001")).toBeInTheDocument();
    expect(screen.getByText("はさみ")).toBeInTheDocument();
    // UI-01a-D9（2026-08-03 gated amendment）: live 型化により明示 id 契約（旧 PR #98 Codex R2 P2）は
    // 廃止。現在の live 型は可視 Label「商品を検索」を htmlFor で結線し、aria-label は持たない。
    expect(screen.getByLabelText("商品を検索")).toHaveAttribute("type", "search");
    expect(
      Array.from(
        screen.getByRole("group", { name: "廃番表示" }).querySelectorAll("button"),
        (button) => button.textContent,
      ),
    ).toEqual(["表示中", "すべて", "廃番のみ"]);
    expect(screen.getByRole("link", { name: "商品登録" })).toHaveAttribute(
      "href",
      "/products/new?returnTo=%2Fproducts%3Fdiscontinued%3Dactive%26plu%3Dall%26sort%3Dproduct_code%26dir%3Dasc%26page%3D1%26perPage%3D100",
    );
    expect(screen.getByRole("link", { name: "修正" })).toHaveAttribute(
      "href",
      "/products/P-001/edit?returnTo=%2Fproducts%3Fdiscontinued%3Dactive%26plu%3Dall%26sort%3Dproduct_code%26dir%3Dasc%26page%3D1%26perPage%3D100",
    );
    await waitFor(() => {
      expect(mockListDepartments).toHaveBeenCalledTimes(1);
    });
  });

  it("keeps controls visible when product query fails", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "error",
      error: {
        kind: "internal",
        message: "取得に失敗しました",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    mockListDepartments.mockResolvedValue({ status: "ok", data: [makeMockDepartment()] });

    renderWithClient(<ProductListPage search={{ q: "はさみ" }} onSearchChange={vi.fn()} />);

    expect(screen.getByLabelText("商品を検索")).toBeInTheDocument();
    await waitFor(
      () => {
        expect(screen.getByText("商品一覧の取得に失敗しました")).toBeInTheDocument();
      },
      { timeout: 5000 },
    );
  });

  // B0 characterization: 空結果の EmptyState DOM 固定（意図的差分③）
  // bare div → EmptyState 標準 UI に置換。title(h3) + description の 2 要素に分割される。
  it("B0-products-empty: products query が items 空を返したとき EmptyState の title と description が表示される", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [],
        total_count: 0,
        page: 1,
        per_page: 50,
      },
    });
    mockListDepartments.mockResolvedValue({ status: "ok", data: [] });

    renderWithClient(<ProductListPage search={{}} onSearchChange={vi.fn()} />);

    expect(
      await screen.findByRole("heading", { name: "該当する商品がありません" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText("検索条件を変更するか、新しい商品を登録してください"),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "商品を登録する" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "PLU 対象にする" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "PLU 対象から外す" })).toBeDisabled();
  });

  it("shows department loading failure without breaking product search controls", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-002", name: "布地" })],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });
    mockListDepartments.mockResolvedValue({
      status: "error",
      error: {
        kind: "internal",
        message: "部門取得に失敗しました",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });

    renderWithClient(<ProductListPage search={{}} onSearchChange={vi.fn()} />);

    expect(screen.getByLabelText("商品を検索")).toBeInTheDocument();
    expect(await screen.findByText("P-002")).toBeInTheDocument();
    // departmentsQuery は production 設計で retry: 1 を持つ（QueryClient default を上書き）ため、
    // 失敗確定まで retry delay 約 1s を要する。既定 timeout 1000ms は並列負荷で同着 flake する
    // （batch B L1 で実測）ので延長する。
    expect(
      await screen.findByText("部門一覧の取得に失敗しました", {}, { timeout: 4000 }),
    ).toBeInTheDocument();
  });

  it("REQ-105 UI-01a uses the maker-code placeholder and shows the cost header", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "COST-001" })],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });
    mockListDepartments.mockResolvedValue({ status: "ok", data: [] });
    renderWithClient(<ProductListPage search={{}} onSearchChange={vi.fn()} />);
    await screen.findByText("COST-001");
    expect(screen.getByLabelText("商品を検索")).toHaveAttribute(
      "placeholder",
      "商品コード・商品名・JAN・メーカー品番で検索",
    );
    expect(screen.getByRole("columnheader", { name: "原価" })).toBeInTheDocument();
  });
});

describe("ProductListPage SPEC-UIBB-1/2（filter-empty reset action、既存「商品を登録する」と共存）", () => {
  beforeEach(() => {
    mockListDepartments.mockResolvedValue({
      status: "ok",
      data: [
        makeMockDepartment({ id: 1, name: "毛糸" }),
        makeMockDepartment({ id: 2, name: "布" }),
      ],
    });
  });

  it("SPEC-UIBB-1 絞り込み該当なしで解除ボタンを表示する", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });
    renderWithClient(
      <ProductListPage search={{ q: "該当なし", dept: 1 }} onSearchChange={vi.fn()} />,
    );
    expect(await screen.findByRole("button", { name: "絞り込みを解除" })).toBeInTheDocument();
  });

  it("SPEC-UIBB-1 既定条件の0件では解除ボタンを出さない（登録のみ表示）", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });
    renderWithClient(<ProductListPage search={{}} onSearchChange={vi.fn()} />);
    expect(await screen.findByRole("link", { name: "商品を登録する" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "絞り込みを解除" })).not.toBeInTheDocument();
  });

  it("SPEC-UIBB-1 既定0件は登録のみ・非既定0件は登録と解除の2ボタン", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });
    const defaultRender = renderWithClient(
      <ProductListPage search={{}} onSearchChange={vi.fn()} />,
    );
    expect(await screen.findByRole("link", { name: "商品を登録する" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "絞り込みを解除" })).not.toBeInTheDocument();
    defaultRender.unmount();

    renderWithClient(<ProductListPage search={{ q: "毛糸" }} onSearchChange={vi.fn()} />);
    expect(await screen.findByRole("link", { name: "商品を登録する" })).toBeInTheDocument();
    expect(await screen.findByRole("button", { name: "絞り込みを解除" })).toBeInTheDocument();
  });

  it("SPEC-UIBB-2 解除で全条件が既定値に戻り、sort/dir/perPageは変更しない", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });
    const onSearchChange = vi.fn();
    renderWithClient(
      <ProductListPage
        search={{
          q: "毛糸",
          dept: 2,
          discontinued: "discontinued",
          page: 3,
          sort: "name",
          dir: "desc",
          perPage: 100,
        }}
        onSearchChange={onSearchChange}
      />,
    );
    const resetButton = await screen.findByRole("button", { name: "絞り込みを解除" });
    await userEvent.setup().click(resetButton);

    const updater = onSearchChange.mock.calls[onSearchChange.mock.calls.length - 1]?.[0] as (
      prev: ProductListSearch,
    ) => ProductListSearch;
    const result = updater({
      q: "毛糸",
      dept: 2,
      discontinued: "discontinued",
      page: 3,
      sort: "name",
      dir: "desc",
      perPage: 100,
    });
    expect(result.q).toBeUndefined();
    expect(result.dept).toBeUndefined();
    expect(result.discontinued).toBeUndefined();
    expect(result.page).toBe(1);
    // sort / dir / perPage は絞り込み条件ではないため reset で変更しない
    expect(result.sort).toBe("name");
    expect(result.dir).toBe("desc");
    expect(result.perPage).toBe(100);
  });
});

describe("ProductListPage SPEC-UIBB-10/11（live 型検索 + 複数ボタン中央揃え、UI-01a-D9）", () => {
  beforeEach(() => {
    mockListDepartments.mockResolvedValue({
      status: "ok",
      data: [makeMockDepartment({ id: 1, name: "毛糸" })],
    });
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-001", name: "はさみ" })],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });
  });

  it("SPEC-UIBB-10 検索入力が200msデバウンスでqに反映される", async () => {
    const onSearchChange = vi.fn();
    renderWithClient(<ProductListPage search={{ page: 2 }} onSearchChange={onSearchChange} />);
    const input = screen.getByLabelText("商品を検索");
    await userEvent.setup().type(input, "はさみ");
    await waitFor(
      () => {
        const call = onSearchChange.mock.calls.find((c) => {
          const updater = c[0] as (prev: ProductListSearch) => ProductListSearch;
          return updater({ page: 2 }).q === "はさみ";
        });
        expect(call).toBeDefined();
      },
      { timeout: 3000 },
    );
    const call = onSearchChange.mock.calls.find((c) => {
      const updater = c[0] as (prev: ProductListSearch) => ProductListSearch;
      return updater({ page: 2 }).q === "はさみ";
    });
    const updater = call?.[0] as (prev: ProductListSearch) => ProductListSearch;
    // q 変更（page 以外の変更）は pageOnlyChange 機構で page=1 に戻る
    expect(updater({ page: 2 }).page).toBe(1);
  });

  it("SPEC-UIBB-10 可視Label「商品を検索」を表示し、検索ボタンとaria-labelは持たない", () => {
    renderWithClient(<ProductListPage search={{}} onSearchChange={vi.fn()} />);
    const input = screen.getByLabelText("商品を検索");
    expect(input).toHaveAttribute("type", "search");
    expect(screen.queryByRole("button", { name: "検索" })).not.toBeInTheDocument();
    expect(screen.queryByText("検索", { selector: "label" })).not.toBeInTheDocument();
    expect(screen.getByText("商品を検索", { selector: "label" })).toHaveAttribute("for", input.id);
    expect(input).not.toHaveAttribute("aria-label");
  });

  it("SPEC-UIBB-10 Enterで即時flushしIME変換確定Enterでは発火しない", () => {
    const onSearchChange = vi.fn();
    renderWithClient(<ProductListPage search={{}} onSearchChange={onSearchChange} />);
    const input = screen.getByLabelText("商品を検索");

    // IME 変換確定の Enter は発火しない
    fireEvent.change(input, { target: { value: "はさみ" } });
    const composingEnter = new KeyboardEvent("keydown", {
      key: "Enter",
      bubbles: true,
      cancelable: true,
    });
    Object.defineProperty(composingEnter, "isComposing", { value: true, configurable: true });
    input.dispatchEvent(composingEnter);
    expect(onSearchChange).not.toHaveBeenCalled();

    // 通常の Enter は debounce を待たず即時 flush
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onSearchChange).toHaveBeenCalled();
    const updater = onSearchChange.mock.calls[0]?.[0] as (
      prev: ProductListSearch,
    ) => ProductListSearch;
    expect(updater({}).q).toBe("はさみ");
  });

  it("SPEC-UIBB-10 クリアでqが外れpageが既定に戻る", async () => {
    const onSearchChange = vi.fn();
    renderWithClient(
      <ProductListPage search={{ q: "はさみ", page: 3 }} onSearchChange={onSearchChange} />,
    );
    const input = screen.getByLabelText("商品を検索");
    await userEvent.setup().clear(input);
    await waitFor(
      () => {
        const call = onSearchChange.mock.calls.find((c) => {
          const updater = c[0] as (prev: ProductListSearch) => ProductListSearch;
          return updater({ q: "はさみ", page: 3 }).q === undefined;
        });
        expect(call).toBeDefined();
      },
      { timeout: 3000 },
    );
    const call = onSearchChange.mock.calls.find((c) => {
      const updater = c[0] as (prev: ProductListSearch) => ProductListSearch;
      return updater({ q: "はさみ", page: 3 }).q === undefined;
    });
    const updater = call?.[0] as (prev: ProductListSearch) => ProductListSearch;
    expect(updater({ q: "はさみ", page: 3 }).page).toBe(1);
    expect(input).toHaveValue("");
  });

  it("SPEC-UIBB-10 空白込み入力が表示保持されCMDのkeywordだけがtrimされる", async () => {
    // controlled harness: updater の結果を search prop に反映して再描画する
    function Harness() {
      const [search, setSearch] = useState<ProductListSearch>({});
      return (
        <ProductListPage
          search={search}
          onSearchChange={(updater) => {
            setSearch((prev) => updater(prev));
          }}
        />
      );
    }
    renderWithClient(<Harness />);
    const input = screen.getByLabelText("商品を検索");
    fireEvent.change(input, { target: { value: "  はさみ  " } });
    fireEvent.keyDown(input, { key: "Enter" });
    // 再描画後も入力表示は空白込みのまま（normalizedSearch.q 結線なら trim 済みに書き戻される）
    await waitFor(() => {
      expect(input).toHaveValue("  はさみ  ");
    });
    // CMD payload の keyword だけが trim される
    await waitFor(() => {
      const keywords = mockSearchProducts.mock.calls.map((c) => c[0].keyword);
      expect(keywords).toContain("はさみ");
    });
    expect(mockSearchProducts.mock.calls.map((c) => c[0].keyword)).not.toContain("  はさみ  ");
  });

  it("SPEC-UIBB-11 空状態の2ボタンは中央揃えで登録が先", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });
    renderWithClient(<ProductListPage search={{ q: "存在しない" }} onSearchChange={vi.fn()} />);
    const resetButton = await screen.findByRole("button", { name: "絞り込みを解除" });
    const wrapper = resetButton.parentElement;
    if (wrapper === null) {
      throw new Error("reset button has no parent wrapper");
    }
    expect(wrapper).toHaveClass("justify-center");
    const registerLink = within(wrapper).getByRole("link", {
      name: "商品を登録する",
    });
    // 「商品を登録する」が先、「絞り込みを解除」が後（DOM 順序）
    expect(
      registerLink.compareDocumentPosition(resetButton) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
  });
});

describe("ProductListPage S5 pilot（ListShell 採用、D-6）", () => {
  beforeEach(() => {
    mockListDepartments.mockResolvedValue({ status: "ok", data: [] });
  });

  it("SC5a: URL 指定なしは per_page 100、?perPage=50 指定時は per_page 50", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-100" })],
        total_count: 1,
        page: 1,
        per_page: 100,
      },
    });
    renderWithClient(<ProductListPage search={{}} onSearchChange={vi.fn()} />);
    await screen.findByText("P-100");
    expect(mockSearchProducts).toHaveBeenLastCalledWith(expect.objectContaining({ per_page: 100 }));

    mockSearchProducts.mockClear();
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-050" })],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });
    renderWithClient(<ProductListPage search={{ perPage: 50 }} onSearchChange={vi.fn()} />);
    await screen.findByText("P-050");
    expect(mockSearchProducts).toHaveBeenLastCalledWith(expect.objectContaining({ per_page: 50 }));
  });

  it("SC5b: toolbar 枠・上部件数・下部 pager・ListSkeleton・0 件時の非描画", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-200" })],
        total_count: 150,
        page: 1,
        per_page: 100,
      },
    });
    const onSearchChange = vi.fn();
    const { container } = renderWithClient(
      <ProductListPage search={{}} onSearchChange={onSearchChange} />,
    );
    await screen.findByText("P-200");

    // toolbar 枠（rounded-lg border bg-card p-4）
    expect(container.querySelector(".rounded-lg.border.bg-card.p-4")).not.toBeNull();
    // 上部件数（PaginationSummary、S2 で text-sm text-muted-foreground tabular-nums を
    // 1 要素に同時に持つ。下部 Pagination は text-sm/text-muted-foreground と
    // tabular-nums を別要素に分離するため、3 class 同時一致は上部のみ）
    const topSummaries = container.querySelectorAll(
      "div.text-sm.text-muted-foreground.tabular-nums",
    );
    expect(topSummaries).toHaveLength(1);
    const topSummary = topSummaries.item(0);
    expect(topSummary).not.toBeNull();
    expect(topSummary).toHaveTextContent("全 150 件のうち 1〜100 件を表示（1 / 2 ページ）");
    // 下部 pager が search state を更新する
    await userEvent.setup().click(screen.getByRole("button", { name: "次のページ" }));
    const updater = onSearchChange.mock.calls[onSearchChange.mock.calls.length - 1]?.[0] as (
      prev: ProductListSearch,
    ) => ProductListSearch;
    expect(updater({}).page).toBe(2);
  });

  it("SC9d: rendered ListShell root carries identityColumns=2 class tokens (商品一覧のみ活性化)", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-210" })],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });
    const { container } = renderWithClient(
      <ProductListPage search={{}} onSearchChange={vi.fn()} />,
    );
    await screen.findByText("P-210");
    // ListShell root は stickyHeader により list-shell-sticky を持つため一意に特定できる。
    const rootClass = container.querySelector(".list-shell-sticky")?.className ?? "";
    const tokens = rootClass.split(/\s+/);
    expect(tokens).toContain("[&_thead_th:nth-child(1)]:sticky");
    expect(tokens).toContain("[&_thead_th:nth-child(2)]:left-[9rem]");
    expect(tokens).toContain("[&_tbody_td:nth-child(1)]:bg-background");
  });

  it("GA3b-1: PageShell root carries none of flex/h-full/min-h-0/flex-col (Lane 4 Gated Amendment 3, GA1a の逆転)", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-GA3B1" })],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });
    const { container } = renderWithClient(
      <ProductListPage search={{}} onSearchChange={vi.fn()} />,
    );
    await screen.findByText("P-GA3B1");
    const rootTokens = (container.firstElementChild?.className ?? "").split(/\s+/);
    expect(rootTokens).not.toContain("flex");
    expect(rootTokens).not.toContain("h-full");
    expect(rootTokens).not.toContain("min-h-0");
    expect(rootTokens).not.toContain("flex-col");
    expect(rootTokens).not.toContain("overflow-hidden");
  });

  it("SC5b: isLoading 中は自前 Skeleton ではなく ListSkeleton を描画する", () => {
    mockSearchProducts.mockReturnValue(new Promise(() => undefined));
    renderWithClient(<ProductListPage search={{}} onSearchChange={vi.fn()} />);
    expect(screen.getByLabelText("一覧を読み込み中")).toBeInTheDocument();
  });

  it("SC5b: 0 件時は上部件数・下部 pager のいずれも描画しない", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 100 },
    });
    const { container } = renderWithClient(
      <ProductListPage search={{}} onSearchChange={vi.fn()} />,
    );
    await screen.findByRole("heading", { name: "該当する商品がありません" });
    expect(container.querySelector("div.text-base")).toBeNull();
    expect(screen.queryByRole("button", { name: "次のページ" })).toBeNull();
  });

  it("SC5c: returnTo は既定 100 / Select 変更後 200 の perPage を保持する", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-RT" })],
        total_count: 1,
        page: 1,
        per_page: 200,
      },
    });
    renderWithClient(<ProductListPage search={{ perPage: 200 }} onSearchChange={vi.fn()} />);
    await screen.findByText("P-RT");
    expect(screen.getByRole("link", { name: "商品登録" })).toHaveAttribute(
      "href",
      expect.stringContaining("perPage%3D200"),
    );
  });
});

describe("ProductListPage S10 PLU 一括 caption（Gated Amendment 2 / owner L3 FAIL-2）", () => {
  beforeEach(() => {
    mockListDepartments.mockResolvedValue({ status: "ok", data: [] });
  });

  it("SC7: caption「PLU 一括操作」+ 作用範囲・確認予告文が toolbarSecondary にあり、2 button が同じ group に属する", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-CAP" })],
        total_count: 1,
        page: 1,
        per_page: 100,
      },
    });
    renderWithClient(<ProductListPage search={{}} onSearchChange={vi.fn()} />);
    await screen.findByText("P-CAP");

    const captionLabel = screen.getByText("PLU 一括操作");
    expect(captionLabel).toHaveClass("text-sm", "font-medium", "text-foreground");
    const captionNote = screen.getByText(
      "絞り込みに一致する 1 件すべてが対象です。他のページの商品も含みます。押すと確認画面が開きます。",
    );
    expect(captionNote).toHaveClass("text-muted-foreground");

    const addButton = screen.getByRole("button", { name: "PLU 対象にする" });
    const removeButton = screen.getByRole("button", { name: "PLU 対象から外す" });
    const addGroup = addButton.closest('[role="group"]');
    const removeGroup = removeButton.closest('[role="group"]');
    expect(addGroup).not.toBeNull();
    expect(addGroup).toBe(removeGroup);
    expect(addGroup).toHaveAttribute("aria-labelledby");
  });
});

describe("ProductListPage perPage scroll（UI-01a）", () => {
  it("SC9a: 表示件数変更で画面を先頭へ戻す", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-SCROLL" })],
        total_count: 150,
        page: 2,
        per_page: 100,
      },
    });
    mockListDepartments.mockResolvedValue({ status: "ok", data: [] });
    renderWithClient(
      <ProductListPage search={{ page: 2, perPage: 100 }} onSearchChange={vi.fn()} />,
    );
    await screen.findByText("P-SCROLL");

    const user = userEvent.setup();
    await user.click(screen.getByRole("combobox", { name: "表示件数" }));
    await user.click(screen.getByRole("option", { name: "50 件" }));

    expect(mockScrollPageToTop).toHaveBeenCalledTimes(1);
  });
});

describe("ProductListPage SC11: PLU 一括 caption 独立行 2 段 + 実件数 3 分岐 + dialog title 同期（Gated Amendment 3 S14）", () => {
  beforeEach(() => {
    mockListDepartments.mockResolvedValue({ status: "ok", data: [] });
  });

  function captionBlock(description: HTMLElement): string[] {
    return (description.parentElement?.className ?? "").split(/\s+/).filter(Boolean);
  }

  it("(a) total_count > 0: 実件数を含む 3 文 + describedby + basis-full の独立行", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-CAP-A" })],
        total_count: 1234,
        page: 1,
        per_page: 100,
      },
    });
    renderWithClient(<ProductListPage search={{}} onSearchChange={vi.fn()} />);

    const description = await screen.findByText(
      "絞り込みに一致する 1,234 件すべてが対象です。他のページの商品も含みます。押すと確認画面が開きます。",
    );
    expect(description).toHaveAttribute("id", "plu-bulk-description");

    const group = description.parentElement?.querySelector('[role="group"]') ?? null;
    expect(group).not.toBeNull();
    expect(group).toHaveAttribute("aria-describedby", "plu-bulk-description");
    expect(group).toHaveAttribute("aria-labelledby", "plu-bulk-caption");

    const tokens = captionBlock(description);
    expect(tokens).toContain("basis-full");
    expect(tokens).toContain("items-start");
    expect(tokens).not.toContain("ml-auto");
    expect(tokens).not.toContain("items-end");
  });

  it("(b) total_count === 0: 実行できない理由文", async () => {
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 100 },
    });
    renderWithClient(<ProductListPage search={{}} onSearchChange={vi.fn()} />);

    const description = await screen.findByText("絞り込みに一致する商品がないため実行できません。");
    expect(description).toHaveAttribute("id", "plu-bulk-description");
    const group = description.parentElement?.querySelector('[role="group"]') ?? null;
    expect(group).not.toBeNull();
    expect(group).toHaveAttribute("aria-describedby", "plu-bulk-description");
  });

  it("(c) データ未取得（読込中）: 読込中の文言", () => {
    mockSearchProducts.mockReturnValue(new Promise(() => undefined));
    renderWithClient(<ProductListPage search={{}} onSearchChange={vi.fn()} />);

    const description = screen.getByText(
      "件数を読み込んでいます。読み込みが終わると操作できます。",
    );
    expect(description).toHaveAttribute("id", "plu-bulk-description");
    const group = description.parentElement?.querySelector('[role="group"]') ?? null;
    expect(group).not.toBeNull();
    expect(group).toHaveAttribute("aria-describedby", "plu-bulk-description");
  });

  it("dialog title は「絞り込みに一致する商品を…」で caption の作用範囲と矛盾しない", async () => {
    const user = userEvent.setup();
    mockSearchProducts.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-CAP-T" })],
        total_count: 1,
        page: 1,
        per_page: 100,
      },
    });
    renderWithClient(<ProductListPage search={{}} onSearchChange={vi.fn()} />);
    await screen.findByText("P-CAP-T");

    await user.click(screen.getByRole("button", { name: "PLU 対象にする" }));
    expect(screen.getByText("絞り込みに一致する商品をPLU対象にしますか")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "キャンセル" }));

    await user.click(screen.getByRole("button", { name: "PLU 対象から外す" }));
    expect(screen.getByText("絞り込みに一致する商品をPLU対象から外しますか")).toBeInTheDocument();
  });
});
