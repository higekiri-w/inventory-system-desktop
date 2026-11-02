// src/features/stock-inquiry/components/ProductListTable.test.tsx
//
// REQ-301: ProductListTable の選択行直下インライン展開（colSpan 展開行）+ detail 状態描画。
// 旧「テーブル下部固定カード」実装の混入を nextElementSibling colSpan guard で落とす（C-P2-3）。
// 設計: docs/function-design/58-ui-stock-inquiry.md §58.7 / §58.8

import { useState } from "react";
import { describe, it, expect, vi } from "vitest";
import type { UseQueryResult } from "@tanstack/react-query";
import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { StockDetail } from "@/lib/bindings";
import { renderWithRouter } from "@/test/render-with-router";
import { ProductListTable } from "./ProductListTable";
import { makeMockProductWithRelations, makeMockStockDetail } from "../lib/test-fixtures";

function makeDetailQuery(
  overrides: Partial<UseQueryResult<StockDetail>> = {},
): UseQueryResult<StockDetail> {
  return {
    isLoading: false,
    isError: false,
    isSuccess: true,
    data: makeMockStockDetail(),
    ...overrides,
  } as unknown as UseQueryResult<StockDetail>;
}

// name は department_name デフォルト（"毛糸"）と衝突しない一意名にする（getByText 複数マッチ回避）。
const items = [
  makeMockProductWithRelations({ product_code: "P-001", name: "はさみ" }),
  makeMockProductWithRelations({ product_code: "P-002", name: "ボタン" }),
];

describe("ProductListTable (REQ-301 インライン展開)", () => {
  it("REQ-302: stockout row renders 在庫切れ badge label", async () => {
    renderWithRouter(
      <ProductListTable
        items={[makeMockProductWithRelations({ product_code: "P-ZERO", stock_quantity: 0 })]}
        source="search"
        selected={null}
        detailQuery={makeDetailQuery()}
        onSelect={vi.fn()}
      />,
    );
    expect(await screen.findByText("在庫切れ")).toBeInTheDocument();
    // SC4 / DSR-22: 在庫の既存色とiconを維持してtoneを集約する。
    const badge = screen.getByText("在庫切れ");
    expect(badge).toHaveAttribute("data-tone", "destructive");
    expect(badge).toHaveClass(
      "border-destructive-border",
      "bg-destructive-soft",
      "text-destructive-strong",
    );
    expect(badge.querySelector('svg[aria-hidden="true"]')).toBeInTheDocument();
  });

  it("REQ-302: low-stock row renders 在庫少 badge label", async () => {
    renderWithRouter(
      <ProductListTable
        items={[makeMockProductWithRelations({ product_code: "P-LOW", stock_quantity: 2 })]}
        source="low_stock"
        selected={null}
        detailQuery={makeDetailQuery()}
        onSelect={vi.fn()}
      />,
    );
    expect(await screen.findByText("在庫少")).toBeInTheDocument();
    // SC4 / DSR-22: 在庫の既存色とiconを維持してtoneを集約する。
    const badge = screen.getByText("在庫少");
    expect(badge).toHaveAttribute("data-tone", "warning");
    expect(badge).toHaveClass("border-warning-border", "bg-warning-soft", "text-warning-strong");
    expect(badge.querySelector('svg[aria-hidden="true"]')).toBeInTheDocument();
  });

  it("REQ-302 / SPEC-DISP-B2-1 D-B4: search positive stock renders 在庫あり status label", async () => {
    renderWithRouter(
      <ProductListTable
        items={[makeMockProductWithRelations({ product_code: "P-OK", stock_quantity: 10 })]}
        source="search"
        selected={null}
        detailQuery={makeDetailQuery()}
        onSelect={vi.fn()}
      />,
    );
    expect(await screen.findByText("在庫あり")).toBeInTheDocument();
    // SC1 / DSR-22 / catalog ⑬: 中立は tone 対象外。期待値は catalog の中立色契約。
    const badge = screen.getByText("在庫あり");
    expect(badge).toHaveAttribute("data-variant", "outline");
    expect(badge).not.toHaveAttribute("data-tone");
    expect(badge).toHaveClass("border-stone-200", "bg-stone-50", "text-stone-600");
  });

  it("REQ-301: product code cell uses readable table text size", async () => {
    renderWithRouter(
      <ProductListTable
        items={[makeMockProductWithRelations({ product_code: "HZ-0047", stock_quantity: 10 })]}
        source="search"
        selected={null}
        detailQuery={makeDetailQuery()}
        onSelect={vi.fn()}
      />,
    );
    const cell = (await screen.findByText("HZ-0047")).closest("td");
    expect(cell?.className).toContain("text-sm");
    expect(cell?.className).not.toContain("text-xs");
  });

  it("REQ-301: detail header product code uses readable table text size", async () => {
    const { container } = renderWithRouter(
      <ProductListTable
        items={items}
        source="search"
        selected="P-001"
        detailQuery={makeDetailQuery({
          data: makeMockStockDetail({
            product: makeMockProductWithRelations({ product_code: "P-001", name: "はさみ" }),
          }),
        })}
        onSelect={vi.fn()}
      />,
    );
    await screen.findByText("最終入庫日");
    const detailCode = container.querySelector('tr[data-state="selected"] + tr span.font-mono');
    expect(detailCode?.textContent).toBe("P-001");
    expect(detailCode?.className).toContain("text-sm");
    expect(detailCode?.className).not.toContain("text-xs");
  });

  it("REQ-301: 選択行の直下に詳細をインライン展開する", async () => {
    renderWithRouter(
      <ProductListTable
        items={items}
        source="search"
        selected="P-001"
        detailQuery={makeDetailQuery({
          data: makeMockStockDetail({
            product: makeMockProductWithRelations({ product_code: "P-001", name: "はさみ" }),
          }),
        })}
        onSelect={vi.fn()}
      />,
    );
    // 展開行内に詳細が描画される（「最終入庫日」は列ヘッダと衝突しないラベル）
    expect(await screen.findByText("最終入庫日")).toBeInTheDocument();
  });

  it("REQ-301: 選択行の nextElementSibling が colSpan 展開行（td[colspan=7]、旧下部固定の混入 guard）", async () => {
    renderWithRouter(
      <ProductListTable
        items={items}
        source="search"
        selected="P-001"
        detailQuery={makeDetailQuery()}
        onSelect={vi.fn()}
      />,
    );
    const codeCell = await screen.findByText("P-001");
    const selectedRow = codeCell.closest("tr");
    expect(selectedRow).not.toBeNull();
    const expansionRow = selectedRow?.nextElementSibling;
    expect(expansionRow?.querySelector('td[colspan="7"]')).not.toBeNull();
  });

  it("REQ-301: 非選択時は展開行を描画しない", async () => {
    renderWithRouter(
      <ProductListTable
        items={items}
        source="search"
        selected={null}
        detailQuery={makeDetailQuery()}
        onSelect={vi.fn()}
      />,
    );
    await screen.findByText("はさみ");
    expect(screen.queryByText("最終入庫日")).not.toBeInTheDocument();
  });

  it("REQ-301: detail 失敗時は展開行内に inline エラー（部分障害許容、一覧は維持、§58.8）", async () => {
    renderWithRouter(
      <ProductListTable
        items={items}
        source="search"
        selected="P-001"
        detailQuery={makeDetailQuery({ isError: true, isSuccess: false, data: undefined })}
        onSelect={vi.fn()}
      />,
    );
    expect(await screen.findByText(/商品詳細の取得に失敗しました/)).toBeInTheDocument();
    // 一覧自体は維持（非選択の他商品行は残る）
    expect(screen.getByText("ボタン")).toBeInTheDocument();
  });

  it("REQ-301: 展開行 td は whitespace-normal で折り返し可（Codex Round1 P2-1、旧 nowrap 回帰 guard）", async () => {
    renderWithRouter(
      <ProductListTable
        items={items}
        source="search"
        selected="P-001"
        detailQuery={makeDetailQuery()}
        onSelect={vi.fn()}
      />,
    );
    const expansionCell = (await screen.findByText("P-001"))
      .closest("tr")
      ?.nextElementSibling?.querySelector("td");
    expect(expansionCell?.className).toContain("whitespace-normal");
  });

  it("SC2/SC10: 取引先列ヘッダが表示され、supplier_name の値/null がそれぞれ表示される（低在庫 view）", async () => {
    renderWithRouter(
      <ProductListTable
        items={[
          makeMockProductWithRelations({
            product_code: "P-SUP",
            name: "リボン",
            supplier_name: "取引先A",
          }),
          makeMockProductWithRelations({
            product_code: "P-NOSUP",
            name: "レース",
            supplier_name: null,
          }),
        ]}
        source="low_stock"
        selected={null}
        detailQuery={makeDetailQuery()}
        onSelect={vi.fn()}
      />,
    );
    expect(await screen.findByText("取引先")).toBeInTheDocument();
    expect(await screen.findByText("取引先A")).toBeInTheDocument();
    expect(await screen.findByText("—")).toBeInTheDocument();
  });

  it("SC10: 「すべて」（source=search）でも取引先列ヘッダが表示される", async () => {
    renderWithRouter(
      <ProductListTable
        items={items}
        source="search"
        selected={null}
        detailQuery={makeDetailQuery()}
        onSelect={vi.fn()}
      />,
    );
    expect(await screen.findByText("取引先")).toBeInTheDocument();
  });

  it("SC4: 選択中の行を再クリックすると onSelect が null で呼ばれる（展開行トグルクローズ）", async () => {
    const onSelect = vi.fn();
    renderWithRouter(
      <ProductListTable
        items={items}
        source="search"
        selected="P-001"
        detailQuery={makeDetailQuery()}
        onSelect={onSelect}
      />,
    );
    const row = (await screen.findByText("はさみ")).closest("tr");
    expect(row).not.toBeNull();
    await userEvent.setup().click(row as HTMLElement);
    expect(onSelect).toHaveBeenCalledWith(null);
  });

  it("SC4: 別の行をクリックすると onSelect がその行の product_code で呼ばれる（回帰）", async () => {
    const onSelect = vi.fn();
    renderWithRouter(
      <ProductListTable
        items={items}
        source="search"
        selected="P-001"
        detailQuery={makeDetailQuery()}
        onSelect={onSelect}
      />,
    );
    const row = (await screen.findByText("ボタン")).closest("tr");
    expect(row).not.toBeNull();
    await userEvent.setup().click(row as HTMLElement);
    expect(onSelect).toHaveBeenCalledWith("P-002");
  });
});

// SPEC-COLOR-EMPHASIS-RT-1 / D-CE7: 詳細を開いた行は左端のバー + 進行中の地で詳細と一体に見せる。
// table.tsx の既定は variant 付き（data-[state=selected]:bg-muted）のため、同じ variant で上書きする。
// 描画（CSS の詳細度）は jsdom で測れないため AC-L3-9。
describe("ProductListTable (REQ-301 詳細を開いた行の進行中の見た目)", () => {
  function Harness() {
    const [selected, setSelected] = useState<string | null>(null);
    return (
      <ProductListTable
        items={[
          makeMockProductWithRelations({
            product_code: "P-001",
            name: "はさみ",
            stock_quantity: 2,
          }),
          makeMockProductWithRelations({
            product_code: "P-002",
            name: "ボタン",
            stock_quantity: 10,
          }),
        ]}
        source="low_stock"
        selected={selected}
        detailQuery={makeDetailQuery()}
        onSelect={setSelected}
      />
    );
  }

  it("REQ-301 D-CE7: the open row and its detail row carry the ongoing bar and surface; closing removes them", async () => {
    const user = userEvent.setup();
    renderWithRouter(<Harness />);
    const rowA = (await screen.findByText("はさみ")).closest("tr");
    const rowB = screen.getByText("ボタン").closest("tr");
    if (!rowA || !rowB) throw new Error("rows not rendered");

    // 閉じた行は透明のバーで列をそろえる。
    expect(rowA).toHaveClass("border-l-4", "border-l-transparent");
    expect(rowA).not.toHaveClass("data-[state=selected]:bg-ongoing-soft");

    await user.click(rowA);
    expect(rowA).toHaveAttribute("data-state", "selected");
    expect(rowA).toHaveClass(
      "border-l-4",
      "border-l-ongoing",
      "data-[state=selected]:bg-ongoing-soft",
      "hover:bg-ongoing-soft",
    );
    expect(rowA).not.toHaveClass("data-[state=selected]:bg-muted");
    const detailRow = rowA.nextElementSibling;
    expect(detailRow?.querySelector('td[colspan="7"]')).not.toBeNull();
    expect(detailRow).toHaveClass(
      "border-l-4",
      "border-l-ongoing",
      "bg-ongoing-soft",
      "hover:bg-ongoing-soft",
    );
    expect(detailRow).not.toHaveClass("bg-muted");

    // 別の行を開くと元の行から外れる。
    await user.click(rowB);
    expect(rowA).not.toHaveAttribute("data-state");
    expect(rowA).toHaveClass("border-l-transparent");
    expect(rowA).not.toHaveClass("border-l-ongoing");
    expect(rowB).toHaveClass("border-l-ongoing", "data-[state=selected]:bg-ongoing-soft");

    // もう一度押すと閉じる。
    await user.click(rowB);
    expect(rowB).not.toHaveClass("border-l-ongoing");
    expect(screen.queryByText("最終入庫日")).not.toBeInTheDocument();
  });

  it("REQ-301 D-CE7 / WCAG 1.4.3: the low-stock cell turns -strong only while its row is open", async () => {
    const user = userEvent.setup();
    renderWithRouter(<Harness />);
    const rowA = (await screen.findByText("はさみ")).closest("tr");
    if (!rowA) throw new Error("row not rendered");
    const stockCell = rowA.querySelectorAll("td")[5];

    expect(stockCell).toHaveClass("text-warning-emphasis");
    expect(stockCell).not.toHaveClass("text-warning-strong");

    await user.click(rowA);
    expect(stockCell).toHaveClass("text-warning-strong", "font-medium");
    expect(stockCell).not.toHaveClass("text-warning-emphasis");

    await user.click(rowA);
    expect(stockCell).toHaveClass("text-warning-emphasis");
    expect(stockCell).not.toHaveClass("text-warning-strong");
  });
});

describe("ProductListTable の在庫の欄（D-118 b2、58）", () => {
  it("U2 recheck range does not change stock status derived from copy", async () => {
    // REQ-302 / SPEC-STK-CNT-D3（Matrix U2）: 範囲と badge は出すが、状態は写しの 1 で「在庫少」（範囲の下端 0 で在庫切れにしない）
    renderWithRouter(
      <ProductListTable
        items={[
          makeMockProductWithRelations({
            product_code: "U2-REC",
            stock_quantity: 1,
            stock_recheck: true,
            stock_range_low: 0,
            stock_range_high: 1,
          }),
          makeMockProductWithRelations({ product_code: "U2-PLN", stock_quantity: 3 }),
        ]}
        source="low_stock"
        selected={null}
        detailQuery={makeDetailQuery()}
        onSelect={vi.fn()}
      />,
    );
    expect(await screen.findByText("0〜1 個")).toBeInTheDocument();
    expect(screen.getAllByText("要再確認")).toHaveLength(1);
    expect(screen.getAllByText("在庫少")).toHaveLength(2);
    expect(screen.queryByText("在庫切れ")).not.toBeInTheDocument();
    expect(screen.getByText("3 個")).toBeInTheDocument();
  });
});
