// src/features/stock-inquiry/StockInquiryPage.test.tsx
//
// REQ-301: StockInquiryPage の結果 1 件自動展開（Q-3 補強）。
// list query 結果が 1 件のとき useEffect → onSearchChange で selected URL state を更新。
// SPEC-UIBB-1/2/4/5/8/9（2026-08-03 batch B）: filter-empty reset / pagination / 範囲外 page /
// 部門候補 listDepartments 化。
// 設計: docs/function-design/58-ui-stock-inquiry.md §58.7 / §58.9

import { describe, it, expect, vi, beforeEach } from "vitest";
import { useState, type ReactNode } from "react";
import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import { commands } from "@/lib/bindings";
import { renderWithRouter } from "@/test/render-with-router";
import type { StockInquirySearch } from "./types";
import { StockInquiryPage } from "./StockInquiryPage";
import { scrollPageToTop } from "@/lib/page-scroll";
import {
  makeMockDepartment,
  makeMockProductWithRelations,
  makeMockStockDetail,
} from "./lib/test-fixtures";

vi.mock("@/lib/bindings", () => ({
  commands: {
    searchProducts: vi.fn(),
    listLowStock: vi.fn(),
    getStockDetail: vi.fn(),
    listDepartments: vi.fn(),
  },
}));
vi.mock("@/lib/page-scroll", () => ({ scrollPageToTop: vi.fn() }));

const mockSearch = vi.mocked(commands.searchProducts);
const mockLowStock = vi.mocked(commands.listLowStock);
const mockDetail = vi.mocked(commands.getStockDetail);
const mockListDepartments = vi.mocked(commands.listDepartments);
const mockScrollPageToTop = vi.mocked(scrollPageToTop);

function renderWithClient(ui: ReactNode, qc?: QueryClient) {
  const client =
    qc ??
    new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
    });
  return renderWithRouter(<QueryClientProvider client={client}>{ui}</QueryClientProvider>);
}

beforeEach(() => {
  mockSearch.mockReset();
  mockLowStock.mockReset();
  mockDetail.mockReset();
  mockListDepartments.mockReset();
  mockScrollPageToTop.mockReset();
  mockListDepartments.mockResolvedValue({ status: "ok", data: [] });
});

// B0 characterization: 空結果の EmptyState DOM 固定（意図的差分③）
// bare div → EmptyState 標準 UI に置換。title(h3) + description の 2 要素に分割される。
// useStockInquiry を内蔵する page なので renderWithClient で render し bindings を vi.mock する
describe("StockInquiryPage (B0 empty-state characterization)", () => {
  it("B0-stock-empty: searchProducts が items 空を返したとき EmptyState の title と description が表示される", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [],
        total_count: 0,
        page: 1,
        per_page: 50,
      },
    });
    renderWithClient(
      <StockInquiryPage search={{ q: "存在しない商品", status: "all" }} onSearchChange={vi.fn()} />,
    );

    expect(
      await screen.findByRole("heading", { name: "該当する商品がありません" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText("商品コード・商品名・JANコードを変えてもう一度検索してください"),
    ).toBeInTheDocument();
  });
});

describe("StockInquiryPage (REQ-301 自動展開)", () => {
  it("REQ-302: search result stockout reaches page as 在庫切れ", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-ZERO", stock_quantity: 0 })],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });
    const onSearchChange = vi.fn();
    renderWithClient(
      <StockInquiryPage search={{ q: "P-ZERO", status: "all" }} onSearchChange={onSearchChange} />,
    );

    const codeCell = await screen.findByText("P-ZERO");
    const row = codeCell.closest("tr");
    if (row === null) {
      throw new Error("P-ZERO row not found");
    }
    expect(within(row).getByText("在庫切れ")).toBeInTheDocument();
  });

  it("REQ-302: low_stock result reaches page as 在庫少", async () => {
    mockLowStock.mockResolvedValue({
      status: "ok",
      data: [
        makeMockProductWithRelations({
          product_code: "P-LOW",
          name: "少ない商品",
          stock_quantity: 2,
        }),
      ],
    });
    const onSearchChange = vi.fn();
    renderWithClient(
      <StockInquiryPage search={{ status: "low_stock" }} onSearchChange={onSearchChange} />,
    );

    const codeCell = await screen.findByText("P-LOW");
    const row = codeCell.closest("tr");
    if (row === null) {
      throw new Error("P-LOW row not found");
    }
    expect(within(row).getByText("在庫少")).toBeInTheDocument();
    // SC3b（round 2/3 是正）: status !== "all"（low_stock）のとき totalCount が無いため
    // 上部 PaginationSummary は描画されない。
    expect(screen.queryByText(/件のうち/)).not.toBeInTheDocument();
    // Final Review round 1 P1（Opus mutation 再実測）: (totalCount ?? 0) > 0 の
    // ガードが落ちても「0 件」の誤描画自体は起きないことを独立に確認する。
    expect(screen.queryByText("0 件")).not.toBeInTheDocument();
  });

  it('SC3b: status === "all" かつ total_count 0 のとき上部 summary は描画されない', async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });
    renderWithClient(
      <StockInquiryPage search={{ q: "該当なし", status: "all" }} onSearchChange={vi.fn()} />,
    );
    await screen.findByRole("heading", { name: "該当する商品がありません" });
    expect(screen.queryByText("0 件")).not.toBeInTheDocument();
    expect(screen.queryByText(/件のうち/)).not.toBeInTheDocument();
  });

  it("SC4b: filter row root has rounded-lg border bg-card p-4 (old borderless frame removed)", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-001" })],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });
    const { container } = renderWithClient(
      <StockInquiryPage search={{ q: "はさみ", status: "all" }} onSearchChange={vi.fn()} />,
    );
    await screen.findByText("P-001");
    expect(container.querySelector(".rounded-lg.border.bg-card.p-4")).not.toBeNull();
    // SC15 / AC-L3-3: 上置き Label の検索欄と隣接 control の下辺を揃える。
    const toolbar = screen.getByText("商品を検索", { selector: "label" }).parentElement
      ?.parentElement;
    expect(toolbar).toHaveClass("items-end");
    expect(toolbar).not.toHaveClass("items-center");
    const oldFrame = Array.from(container.querySelectorAll("div")).find(
      (el) => el.className === "flex flex-wrap items-center gap-3",
    );
    expect(oldFrame).toBeUndefined();
  });

  it("REQ-301: 検索結果 1 件で onSearchChange に selected を渡し詳細を自動展開", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "SOLO-1" })],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });
    const onSearchChange = vi.fn();
    renderWithClient(
      <StockInquiryPage search={{ q: "SOLO", status: "all" }} onSearchChange={onSearchChange} />,
    );

    // list 結果 1 件で auto-expand useEffect が onSearchChange を呼ぶ
    await waitFor(() => {
      expect(onSearchChange).toHaveBeenCalled();
    });
    // updater を適用して selected が SOLO-1 になることを確認
    const updaters = onSearchChange.mock.calls.map((call) => call[0] as (p: object) => object);
    const results = updaters.map((u) => u({}));
    expect(results).toContainEqual(expect.objectContaining({ selected: "SOLO-1" }));
  });

  it("REQ-301: 一覧 query 失敗 + 詳細 query 成功でも詳細カードを独立描画（部分障害許容、§58.8）", async () => {
    // 一覧（search_products）失敗・詳細（get_stock_detail）成功の部分障害シナリオ。
    // 行インライン展開は list 成功前提のため、list 失敗時は Alert 下のフォールバックカードに
    // 詳細を独立描画する（§58.8、Codex Round 1 P2-1 の契約を構造変更後も維持）。
    mockSearch.mockResolvedValue({
      status: "error",
      error: {
        kind: "internal",
        message: "一覧の取得に失敗しました",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    mockDetail.mockResolvedValue({
      status: "ok",
      data: makeMockStockDetail({
        product: makeMockProductWithRelations({ product_code: "P-001", name: "はさみ" }),
      }),
    });
    const onSearchChange = vi.fn();
    renderWithClient(
      <StockInquiryPage
        search={{ q: "はさみ", status: "all", selected: "P-001" }}
        onSearchChange={onSearchChange}
      />,
    );

    // 一覧は取得失敗 Alert、詳細カードは独立して描画される（listQuery の retry:1 backoff を待つ）。
    await waitFor(
      () => {
        expect(screen.getByText("取得に失敗しました")).toBeInTheDocument();
        expect(screen.getByText("在庫数")).toBeInTheDocument();
      },
      { timeout: 5000 },
    );
    // 詳細カード固有の商品コード（CardTitle 内）も描画されている。
    expect(screen.getByText("P-001")).toBeInTheDocument();
  });

  it("REQ-301: list 成功 + selected 指定で選択行直下にインライン展開（§58.8）", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          makeMockProductWithRelations({ product_code: "P-001", name: "はさみ" }),
          makeMockProductWithRelations({ product_code: "P-002", name: "ボタン" }),
        ],
        total_count: 2,
        page: 1,
        per_page: 50,
      },
    });
    mockDetail.mockResolvedValue({
      status: "ok",
      data: makeMockStockDetail({
        product: makeMockProductWithRelations({ product_code: "P-001", name: "はさみ" }),
      }),
    });
    const onSearchChange = vi.fn();
    renderWithClient(
      <StockInquiryPage
        search={{ q: "P", status: "all", selected: "P-001" }}
        onSearchChange={onSearchChange}
      />,
    );
    // 詳細（最終入庫日 = 列ヘッダと衝突しないラベル）が選択行直下のインライン展開行に描画される。
    // colSpan 展開行の DOM 構造（nextElementSibling）は ProductListTable.test で検証済み。
    await waitFor(() => {
      expect(screen.getByText("最終入庫日")).toBeInTheDocument();
    });
  });

  it("REQ-301: 行クリックで selected が更新され選択行直下に展開する（統合経路、C-P2-3）", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          makeMockProductWithRelations({ product_code: "P-001", name: "はさみ" }),
          makeMockProductWithRelations({ product_code: "P-002", name: "ボタン" }),
        ],
        total_count: 2,
        page: 1,
        per_page: 50,
      },
    });
    mockDetail.mockResolvedValue({
      status: "ok",
      data: makeMockStockDetail({
        product: makeMockProductWithRelations({ product_code: "P-002", name: "ボタン" }),
      }),
    });
    // onSearchChange updater を useState に適用する stateful harness（URL state を実更新する）
    function Harness() {
      const [search, setSearch] = useState<StockInquirySearch>({ q: "P", status: "all" });
      return (
        <StockInquiryPage
          search={search}
          onSearchChange={(updater) => {
            setSearch((prev) => updater(prev));
          }}
        />
      );
    }
    const user = userEvent.setup();
    renderWithClient(<Harness />);
    // 一覧 2 件描画（自動展開なし）。クリック前は詳細なし。
    await waitFor(() => {
      expect(screen.getByText("ボタン")).toBeInTheDocument();
    });
    expect(screen.queryByText("最終入庫日")).not.toBeInTheDocument();
    // ボタン行（P-002）をクリック → selected 更新 → 直下にインライン展開
    await user.click(screen.getByText("ボタン"));
    await waitFor(() => {
      expect(screen.getByText("最終入庫日")).toBeInTheDocument();
    });
  });

  it("R2-3: 選択中行の再クリックで onSearchChange に selected: undefined を渡す（page 結線）", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          makeMockProductWithRelations({ product_code: "P-001", name: "はさみ" }),
          makeMockProductWithRelations({ product_code: "P-002", name: "ボタン" }),
        ],
        total_count: 2,
        page: 1,
        per_page: 50,
      },
    });
    mockDetail.mockResolvedValue({
      status: "ok",
      data: makeMockStockDetail({
        product: makeMockProductWithRelations({ product_code: "P-002", name: "ボタン" }),
      }),
    });
    const onSearchChange = vi.fn();
    const search: StockInquirySearch = { q: "P", status: "all", selected: "P-002" };
    const { container } = renderWithClient(
      <StockInquiryPage search={search} onSearchChange={onSearchChange} />,
    );
    await waitFor(() => {
      expect(screen.getByText("最終入庫日")).toBeInTheDocument();
    });
    // 選択中（P-002）行を再クリック → ProductListTable の onSelect(null) 経由で
    // StockInquiryPage.tsx の `code ?? undefined` 結線が selected: undefined を渡す
    // （選択行は data-state="selected"、展開行と product 名テキストが重複するため row 要素を直接特定する）
    const user = userEvent.setup();
    const selectedRow = container.querySelector('tr[data-state="selected"]');
    if (selectedRow === null) {
      throw new Error("選択中行（data-state=selected）が見つからない");
    }
    await user.click(selectedRow);
    await waitFor(() => {
      expect(onSearchChange).toHaveBeenCalled();
    });
    const updaters = onSearchChange.mock.calls.map((call) => call[0] as (p: object) => object);
    const results = updaters.map((updater) => updater(search));
    expect(results).toContainEqual(expect.objectContaining({ selected: undefined }));
  });
});

describe("StockInquiryPage SPEC-UIBB-1/2（filter-empty reset action）", () => {
  it("SPEC-UIBB-1 絞り込み該当なしで解除ボタンを表示する", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });
    renderWithClient(
      <StockInquiryPage
        search={{ q: "該当なし", dept: 1, status: "all" }}
        onSearchChange={vi.fn()}
      />,
    );
    expect(await screen.findByRole("button", { name: "絞り込みを解除" })).toBeInTheDocument();
  });

  it("SPEC-UIBB-1 既定条件の0件では解除ボタンを出さない", async () => {
    // q/dept/status すべて既定（isFilterDefault）のときは isAllEmpty ガードで
    // search_products 自体を呼ばず EmptySearchPlaceholder を表示する（契約 I）。
    // reset action は EmptyState 側の action slot にのみ存在するため、既定条件では
    // どのケースでも reset ボタンが出ないことを確認する。
    renderWithClient(<StockInquiryPage search={{}} onSearchChange={vi.fn()} />);
    expect(
      await screen.findByText("商品コード、商品名、または JAN コードで検索してください"),
    ).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "絞り込みを解除" })).not.toBeInTheDocument();
    expect(mockSearch).not.toHaveBeenCalled();
  });

  it("SPEC-UIBB-2 解除で全条件が既定値に戻る", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });
    const onSearchChange = vi.fn();
    renderWithClient(
      <StockInquiryPage
        search={{ q: "該当なし", dept: 3, status: "all", page: 2 }}
        onSearchChange={onSearchChange}
      />,
    );
    const resetButton = await screen.findByRole("button", { name: "絞り込みを解除" });
    await userEvent.setup().click(resetButton);

    const updater = onSearchChange.mock.calls[onSearchChange.mock.calls.length - 1]?.[0] as (
      prev: StockInquirySearch,
    ) => StockInquirySearch;
    const result = updater({ q: "該当なし", dept: 3, status: "all", page: 2 });
    expect(result.q).toBeUndefined();
    expect(result.dept).toBeUndefined();
    expect(result.status).toBeUndefined();
    expect(result.page).toBeUndefined();
    expect(result.selected).toBeUndefined();
  });
});

describe("StockInquiryPage SPEC-UIBB-4（pagination の条件 reset / 維持）", () => {
  it("SC4a: 表示件数を100へ変更するとpage 1・per_page 100で再取得する", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          makeMockProductWithRelations({ product_code: "P-001" }),
          makeMockProductWithRelations({ product_code: "P-002" }),
        ],
        total_count: 120,
        page: 1,
        per_page: 100,
      },
    });
    function Harness() {
      const [search, setSearch] = useState<StockInquirySearch>({ q: "P", status: "all", page: 2 });
      return <StockInquiryPage search={search} onSearchChange={setSearch} />;
    }
    renderWithClient(<Harness />);
    await screen.findByText("P-001");

    const user = userEvent.setup();
    await user.click(screen.getByRole("combobox", { name: "表示件数" }));
    await user.click(await screen.findByRole("option", { name: "100 件" }));

    await waitFor(() => {
      expect(mockSearch).toHaveBeenLastCalledWith(
        expect.objectContaining({ page: 1, per_page: 100 }),
      );
    });
  });

  it("SPEC-UIBB-4 検索条件変更でpage=1に戻る", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-001" })],
        total_count: 1,
        page: 2,
        per_page: 50,
      },
    });
    const onSearchChange = vi.fn();
    renderWithClient(
      <StockInquiryPage
        search={{ q: "毛糸", status: "all", page: 2 }}
        onSearchChange={onSearchChange}
      />,
    );
    await screen.findByText("P-001");
    const searchInput = screen.getByRole("searchbox");
    await userEvent.setup().type(searchInput, "X");
    // 単一件自動展開の selected 更新 call も混ざるため、q を変更した call を明示的に待つ
    // （debounce 200ms を待つため waitFor の既定 timeout を超える場合がある）。
    await waitFor(
      () => {
        const qChangeCall = onSearchChange.mock.calls.find((call) => {
          const updater = call[0] as (prev: StockInquirySearch) => StockInquirySearch;
          return updater({ q: "毛糸", status: "all", page: 2 }).q !== "毛糸";
        });
        expect(qChangeCall).toBeDefined();
      },
      { timeout: 3000 },
    );
    const qChangeCall = onSearchChange.mock.calls.find((call) => {
      const updater = call[0] as (prev: StockInquirySearch) => StockInquirySearch;
      return updater({ q: "毛糸", status: "all", page: 2 }).q !== "毛糸";
    });
    const updater = qChangeCall?.[0] as (prev: StockInquirySearch) => StockInquirySearch;
    const result = updater({ q: "毛糸", status: "all", page: 2 });
    expect(result.page).toBeUndefined();
  });

  // Final Review P2 是正: q 経路だけでは dept / status handler の page reset 除去 mutant を
  // 殺せない（survivor 実証済み）。3 経路それぞれの UI 操作で updater を検証する。
  it("SPEC-UIBB-4 部門変更でpage=1に戻る", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-001" })],
        total_count: 1,
        page: 2,
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
    const onSearchChange = vi.fn();
    renderWithClient(
      <StockInquiryPage
        search={{ q: "毛糸", status: "all", page: 2 }}
        onSearchChange={onSearchChange}
      />,
    );
    await screen.findByText("P-001");
    const user = userEvent.setup();
    await user.click(screen.getByRole("combobox", { name: "部門" }));
    await user.click(await screen.findByRole("option", { name: "布" }));
    const prev: StockInquirySearch = { q: "毛糸", status: "all", page: 2 };
    const deptChangeCall = onSearchChange.mock.calls.find((call) => {
      const updater = call[0] as (p: StockInquirySearch) => StockInquirySearch;
      return updater(prev).dept === 2;
    });
    expect(deptChangeCall).toBeDefined();
    const updater = deptChangeCall?.[0] as (p: StockInquirySearch) => StockInquirySearch;
    const result = updater(prev);
    expect(result.page).toBeUndefined();
    expect(result.selected).toBeUndefined();
  });

  it("SPEC-UIBB-4 状態チップ変更でpage=1に戻る", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-001" })],
        total_count: 1,
        page: 2,
        per_page: 50,
      },
    });
    mockLowStock.mockResolvedValue({ status: "ok", data: [] });
    const onSearchChange = vi.fn();
    renderWithClient(
      <StockInquiryPage
        search={{ q: "毛糸", status: "all", page: 2 }}
        onSearchChange={onSearchChange}
      />,
    );
    await screen.findByText("P-001");
    const user = userEvent.setup();
    await user.click(screen.getByRole("radio", { name: "在庫少" }));
    const prev: StockInquirySearch = { q: "毛糸", status: "all", page: 2 };
    const statusChangeCall = onSearchChange.mock.calls.find((call) => {
      const updater = call[0] as (p: StockInquirySearch) => StockInquirySearch;
      return updater(prev).status === "low_stock";
    });
    expect(statusChangeCall).toBeDefined();
    const updater = statusChangeCall?.[0] as (p: StockInquirySearch) => StockInquirySearch;
    const result = updater(prev);
    expect(result.page).toBeUndefined();
    expect(result.selected).toBeUndefined();
  });

  it("SPEC-UIBB-4 page移動で検索条件を維持する", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-001" })],
        total_count: 60,
        page: 1,
        per_page: 50,
      },
    });
    const onSearchChange = vi.fn();
    renderWithClient(
      <StockInquiryPage
        search={{ q: "毛糸", status: "all", page: 1 }}
        onSearchChange={onSearchChange}
      />,
    );
    const nextButton = await screen.findByRole("button", { name: "次のページ" });
    await userEvent.setup().click(nextButton);

    const updater = onSearchChange.mock.calls[onSearchChange.mock.calls.length - 1]?.[0] as (
      prev: StockInquirySearch,
    ) => StockInquirySearch;
    const result = updater({ q: "毛糸", status: "all", page: 1 });
    expect(result.page).toBe(2);
    expect(result.q).toBe("毛糸");
    expect(result.status).toBe("all");
  });
});

describe("StockInquiryPage SPEC-UIBB-5（51件 synthetic で page 2 到達 + truncated alert 撤去）", () => {
  it("SPEC-UIBB-5 51件でpage2に到達できる", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-051", name: "51件目" })],
        total_count: 51,
        page: 2,
        per_page: 50,
      },
    });
    renderWithClient(
      <StockInquiryPage search={{ q: "毛糸", status: "all", page: 2 }} onSearchChange={vi.fn()} />,
    );
    expect(await screen.findByText("P-051")).toBeInTheDocument();
    // S3b（round 3）: 上部 PaginationSummary + 下部 Pagination の両方に同じ件数文言が
    // 現れるため、単数一致の getByText ではなく getAllByText の件数で確認する。
    expect(screen.getAllByText("全 51 件のうち 51〜51 件を表示（2 / 2 ページ）")).toHaveLength(2);
  });
});

describe("StockInquiryPage SPEC-UIBB-8（範囲外 page 回復導線）", () => {
  it("SPEC-UIBB-8 範囲外pageで先頭ページに戻る導線を表示する", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 10, page: 5, per_page: 50 },
    });
    const onSearchChange = vi.fn();
    renderWithClient(
      <StockInquiryPage
        search={{ q: "毛糸", status: "all", page: 5 }}
        onSearchChange={onSearchChange}
      />,
    );
    expect(
      await screen.findByRole("heading", { name: "このページには表示する商品がありません" }),
    ).toBeInTheDocument();
    // 通常 EmptyState / reset action は表示されない（優先判定）
    expect(
      screen.queryByRole("heading", { name: "該当する商品がありません" }),
    ).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "絞り込みを解除" })).not.toBeInTheDocument();

    const backButton = screen.getByRole("button", { name: "先頭ページに戻る" });
    await userEvent.setup().click(backButton);
    const updater = onSearchChange.mock.calls[onSearchChange.mock.calls.length - 1]?.[0] as (
      prev: StockInquirySearch,
    ) => StockInquirySearch;
    const result = updater({ q: "毛糸", status: "all", page: 5 });
    expect(result.page).toBeUndefined();
    expect(result.q).toBe("毛糸");
    expect(result.status).toBe("all");
  });
});

describe("StockInquiryPage SPEC-UIBB-9（部門候補 loading / error 結線）", () => {
  it("SPEC-UIBB-9 候補ロード中はDepartmentFilterがdisabled", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });
    // listDepartments を pending のまま固定（resolve しない）
    mockListDepartments.mockImplementation(() => new Promise(() => undefined));
    renderWithClient(
      <StockInquiryPage search={{ q: "毛糸", status: "all" }} onSearchChange={vi.fn()} />,
    );
    await waitFor(() => {
      expect(screen.getByRole("combobox", { name: "部門" })).toBeDisabled();
    });
  });

  it("SPEC-UIBB-9 候補取得失敗でも一覧は表示され失敗文言が出る", async () => {
    const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-001" })],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });
    mockListDepartments.mockResolvedValue({
      status: "error",
      error: {
        kind: "internal",
        message: "取得失敗",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    renderWithClient(
      <StockInquiryPage search={{ q: "毛糸", status: "all" }} onSearchChange={vi.fn()} />,
      qc,
    );
    expect(await screen.findByText("P-001")).toBeInTheDocument();
    expect(await screen.findByRole("alert")).toHaveTextContent("部門候補の取得に失敗しました");
  });

  it("SPEC-UIBB-9 部門選択中も他部門へ直接切替できる", async () => {
    mockListDepartments.mockResolvedValue({
      status: "ok",
      data: [
        makeMockDepartment({ id: 1, name: "毛糸" }),
        makeMockDepartment({ id: 2, name: "布" }),
      ],
    });
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-001", department_id: 1 })],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });
    renderWithClient(
      <StockInquiryPage search={{ q: "毛糸", status: "all", dept: 1 }} onSearchChange={vi.fn()} />,
    );
    await screen.findByText("P-001");
    const user = userEvent.setup();
    await user.click(screen.getByRole("combobox", { name: "部門" }));
    // dept=1 選択中でも布（id=2）が候補に残っている（DSR-10 縮退なし）
    expect(await screen.findByRole("option", { name: "布" })).toBeInTheDocument();
  });
});

describe("StockInquiryPage perPage scroll（UI-06a）", () => {
  it("SC9a: 表示件数変更で画面を先頭へ戻す", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-SCROLL" })],
        total_count: 120,
        page: 2,
        per_page: 50,
      },
    });
    renderWithClient(
      <StockInquiryPage search={{ q: "P", status: "all", page: 2 }} onSearchChange={vi.fn()} />,
    );
    await screen.findByText("P-SCROLL");

    const user = userEvent.setup();
    await user.click(screen.getByRole("combobox", { name: "表示件数" }));
    await user.click(screen.getByRole("option", { name: "100 件" }));

    expect(mockScrollPageToTop).toHaveBeenCalledTimes(1);
  });
});

describe("StockInquiryPage SPEC-DISP-B2-1（D-B3 / D-B5）", () => {
  it("在庫照会の副題を表示する", async () => {
    renderWithClient(<StockInquiryPage search={{}} onSearchChange={vi.fn()} />);
    expect(
      await screen.findByText("商品ごとの在庫数と状態を確認し、その場で入出庫へ進みます"),
    ).toBeInTheDocument();
  });

  it("low_stock は検索・部門・状態で絞った後の3件を全3件と表示する", async () => {
    mockLowStock.mockResolvedValue({
      status: "ok",
      data: [
        ...[1, 2, 3].map((id) =>
          makeMockProductWithRelations({
            product_code: `LOW-${String(id)}`,
            name: `対象商品${String(id)}`,
            department_id: 1,
            stock_quantity: 2,
          }),
        ),
        makeMockProductWithRelations({ product_code: "OTHER-Q", name: "別商品" }),
        makeMockProductWithRelations({
          product_code: "OTHER-DEPT",
          name: "対象商品別部門",
          department_id: 2,
        }),
        makeMockProductWithRelations({
          product_code: "ZERO",
          name: "対象商品在庫切れ",
          stock_quantity: 0,
        }),
      ],
    });
    renderWithClient(
      <StockInquiryPage
        search={{ status: "low_stock", q: "対象商品", dept: 1 }}
        onSearchChange={vi.fn()}
      />,
    );
    expect(await screen.findByText("LOW-3")).toBeInTheDocument();
    expect(screen.getByText("全 3 件")).toBeInTheDocument();
    expect(screen.getByText("全 3 件")).toHaveClass("text-base", "font-semibold", "tabular-nums");
    expect(screen.queryByText(/件のうち/)).not.toBeInTheDocument();
  });

  it("all は件数のみの行を出さず現行PaginationSummaryを表示する", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-051" })],
        total_count: 51,
        page: 2,
        per_page: 50,
      },
    });
    renderWithClient(
      <StockInquiryPage search={{ q: "商品", status: "all", page: 2 }} onSearchChange={vi.fn()} />,
    );
    await screen.findByText("P-051");
    expect(screen.queryByText(/^全 \d+ 件$/)).not.toBeInTheDocument();
    expect(screen.getAllByText("全 51 件のうち 51〜51 件を表示（2 / 2 ページ）")).toHaveLength(2);
  });

  it("low_stock の0件は件数行なしでEmptyStateだけを表示する", async () => {
    mockLowStock.mockResolvedValue({ status: "ok", data: [] });
    renderWithClient(
      <StockInquiryPage search={{ status: "low_stock" }} onSearchChange={vi.fn()} />,
    );
    await screen.findByRole("heading", { name: "該当する商品がありません" });
    expect(screen.queryByText(/^全 \d+ 件$/)).not.toBeInTheDocument();
    expect(screen.queryByText(/件のうち/)).not.toBeInTheDocument();
    expect(screen.queryByRole("table")).not.toBeInTheDocument();
  });
});
