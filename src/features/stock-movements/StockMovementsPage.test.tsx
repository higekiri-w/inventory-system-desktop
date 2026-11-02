import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { defaultParseSearch } from "@tanstack/react-router";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useState, type ReactNode } from "react";

import { commands } from "@/lib/bindings";
import type { MovementRecord, StockDetail, StockHistoryItem } from "@/lib/bindings";
import { scrollPageToTop } from "@/lib/page-scroll";
import { stockInquirySearchSchema } from "@/features/stock-inquiry/types";
import { renderWithRouter } from "@/test/render-with-router";
import { StockMovementsPage } from "./StockMovementsPage";
import type { StockMovementsSearch } from "./types";
import { normalizeStockMovementsSearch, stockMovementsSearchSchema } from "./types";

vi.mock("@/lib/bindings", () => ({
  commands: {
    getStockDetail: vi.fn(),
    listMovements: vi.fn(),
    listRecheckProducts: vi.fn(),
  },
}));
vi.mock("@/lib/page-scroll", () => ({ scrollPageToTop: vi.fn() }));

const mockGetStockDetail = vi.mocked(commands.getStockDetail);
const mockListMovements = vi.mocked(commands.listMovements);
const mockListRecheckProducts = vi.mocked(commands.listRecheckProducts);
const mockScrollPageToTop = vi.mocked(scrollPageToTop);

function renderWithClient(ui: ReactNode) {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  return renderWithRouter(<QueryClientProvider client={qc}>{ui}</QueryClientProvider>);
}

function makeStockDetail(overrides: Partial<StockDetail> = {}): StockDetail {
  return {
    product: {
      product_code: "BT0002",
      jan_code: "4901234567890",
      name: "ボタン #02",
      department_id: 1,
      supplier_id: null,
      selling_price: 120,
      cost_price_centi: 80,
      tax_rate: "10",
      maker_code: null,
      stock_quantity: 3,
      stock_unit: "pcs",
      is_discontinued: false,
      plu_dirty: false,
      plu_exported_at: null,
      plu_target: false,
      pos_stock_sync: true,
      created_at: "2026-01-01T10:00:00",
      updated_at: "2026-01-01T10:00:00",
      department_name: "ボタン",
      supplier_name: null,
      plu_memory_no: null,
      stock_recheck: false,
      stock_range_low: null,
      stock_range_high: null,
    },
    last_receiving_date: "2026-06-01",
    last_sale_date: "2026-06-20",
    ...overrides,
  };
}

function makeMovement(overrides: Partial<MovementRecord> = {}): MovementRecord {
  return {
    id: 10,
    product_code: "BT0002",
    movement_type: "disposal",
    quantity: -1,
    stock_after: 3,
    reference_type: "disposal_record",
    reference_id: 7,
    source: { label: "廃棄・破損 #7", route: "/inventory/disposal/records/7" },
    note: "破損",
    created_at: "2026-06-27T10:11:12",
    ...overrides,
  };
}

/// T2: listMovements の行は StockHistoryItem（movement の行。D-118 b2）
function movementItem(movement: MovementRecord): StockHistoryItem {
  return {
    kind: "movement",
    business_date: movement.created_at.slice(0, 10),
    settlement_no: null,
    saved_at: movement.created_at,
    movement,
    count: null,
  };
}

beforeEach(() => {
  mockGetStockDetail.mockReset();
  mockListMovements.mockReset();
  mockListRecheckProducts.mockReset();
  mockScrollPageToTop.mockReset();
});

describe("StockMovementsPage (UI-06c)", () => {
  it("REQ-303: URL searchからMovementQueryを作りlistMovementsを呼ぶ", async () => {
    mockGetStockDetail.mockResolvedValue({ status: "ok", data: makeStockDetail() });
    mockListMovements.mockResolvedValue({
      status: "ok",
      data: { items: [movementItem(makeMovement())], total_count: 1, page: 2, per_page: 50 },
    });

    renderWithClient(
      <StockMovementsPage
        productCode="BT0002"
        search={{
          dateFrom: "2026-06-01",
          dateTo: "2026-06-30",
          type: "disposal",
          page: 2,
        }}
        onSearchChange={vi.fn()}
      />,
    );

    await waitFor(() => {
      expect(mockListMovements).toHaveBeenCalledWith({
        product_code: "BT0002",
        date_from: "2026-06-01",
        date_to: "2026-06-30",
        movement_type: "disposal",
        page: 2,
        per_page: 50,
      });
    });
    expect(await screen.findByText("ボタン #02")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "廃棄・破損 #7" })).toHaveAttribute(
      "href",
      "/inventory/disposal/records/7?returnTo=%2Fstock%2FBT0002%2Fmovements%3FdateFrom%3D2026-06-01%26dateTo%3D2026-06-30%26type%3Ddisposal%26page%3D2",
    );
  });

  it("REQ-303: 商品情報取得失敗でもmovement listを表示する", async () => {
    mockGetStockDetail.mockResolvedValue({
      status: "error",
      error: {
        kind: "not_found",
        message: "商品が見つかりません",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    mockListMovements.mockResolvedValue({
      status: "ok",
      data: { items: [movementItem(makeMovement())], total_count: 1, page: 1, per_page: 50 },
    });

    renderWithClient(
      <StockMovementsPage productCode="BT0002" search={{}} onSearchChange={vi.fn()} />,
    );

    await waitFor(
      () => {
        expect(screen.getByText("商品情報の取得に失敗しました")).toBeInTheDocument();
      },
      { timeout: 5000 },
    );
    expect(screen.getByText("廃棄・破損 #7")).toBeInTheDocument();
  });

  it("REQ-303: filter変更時はpageを1に戻す（sentinel round-trip、P2-1）", async () => {
    mockGetStockDetail.mockResolvedValue({ status: "ok", data: makeStockDetail() });
    mockListMovements.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });
    const user = userEvent.setup();

    function Harness() {
      const [search, setSearch] = useState<StockMovementsSearch>({ type: "all", page: 3 });
      return <StockMovementsPage productCode="BT0002" search={search} onSearchChange={setSearch} />;
    }

    renderWithClient(<Harness />);

    const movementType = await screen.findByLabelText("種別");
    expect(movementType).toHaveAttribute("data-slot", "select-trigger");
    expect(movementType.tagName).toBe("BUTTON");
    await user.click(movementType);
    expect((await screen.findAllByRole("option")).map((option) => option.textContent)).toEqual([
      "すべて",
      "入庫",
      "返品・交換",
      "POS売上",
      "手動販売",
      "廃棄・破損",
      "棚卸し",
    ]);
    await user.click(screen.getByRole("option", { name: "入庫" }));

    await waitFor(() => {
      expect(mockListMovements).toHaveBeenLastCalledWith(
        expect.objectContaining({ movement_type: "receiving", page: 1 }),
      );
    });

    await user.click(movementType);
    await user.click(await screen.findByRole("option", { name: "すべて" }));

    await waitFor(() => {
      expect(mockListMovements).toHaveBeenLastCalledWith(
        expect.objectContaining({ movement_type: null, page: 1 }),
      );
    });

    await user.click(movementType);
    await user.click(await screen.findByRole("option", { name: "手動販売" }));

    await waitFor(() => {
      expect(mockListMovements).toHaveBeenLastCalledWith(
        expect.objectContaining({ movement_type: "sale_manual", page: 1 }),
      );
    });

    // P2-R2-1: 残る option も内部値を独立リテラルで検査する（round 1 は具体例3件で打ち切っていた）。
    const REMAINING_MOVEMENT_TYPE_OPTIONS: [label: string, value: string][] = [
      ["返品・交換", "return"],
      ["POS売上", "sale_auto"],
      ["廃棄・破損", "disposal"],
      ["棚卸し", "stocktake"],
    ];
    for (const [label, expectedValue] of REMAINING_MOVEMENT_TYPE_OPTIONS) {
      await user.click(movementType);
      await user.click(await screen.findByRole("option", { name: label }));

      await waitFor(() => {
        expect(mockListMovements).toHaveBeenLastCalledWith(
          expect.objectContaining({ movement_type: expectedValue, page: 1 }),
        );
      });
    }
  });

  it("SC4c: 表示件数を200へ変更するとpage 1・per_page 200で再取得する", async () => {
    mockGetStockDetail.mockResolvedValue({ status: "ok", data: makeStockDetail() });
    mockListMovements.mockResolvedValue({
      status: "ok",
      data: { items: [movementItem(makeMovement())], total_count: 1, page: 1, per_page: 200 },
    });

    function Harness() {
      const [search, setSearch] = useState<StockMovementsSearch>({ page: 3 });
      return <StockMovementsPage productCode="BT0002" search={search} onSearchChange={setSearch} />;
    }

    renderWithClient(<Harness />);
    await screen.findByText("廃棄・破損 #7");
    const user = userEvent.setup();
    await user.click(screen.getByRole("combobox", { name: "表示件数" }));
    await user.click(await screen.findByRole("option", { name: "200 件" }));

    await waitFor(() => {
      expect(mockListMovements).toHaveBeenLastCalledWith(
        expect.objectContaining({ page: 1, per_page: 200 }),
      );
    });
  });
});

describe("StockMovementsPage Lane 4 S1d/S3d: frame color + top summary", () => {
  it("SC4d: filter section root has rounded-lg border bg-card p-4", async () => {
    mockGetStockDetail.mockResolvedValue({ status: "ok", data: makeStockDetail() });
    mockListMovements.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });
    const { container } = renderWithClient(
      <StockMovementsPage productCode="BT0002" search={{}} onSearchChange={vi.fn()} />,
    );
    await waitFor(() => {
      expect(mockListMovements).toHaveBeenCalled();
    });
    // 商品情報 card（src/features/stock-movements/StockMovementsPage.tsx の section、既存 bg-card、Non-scope）も同じ組合せ class を持つため
    // querySelector 単独では最初の一致（商品情報 card）を拾ってしまう
    // （filter frame を戻しても検知できない）。開始日 label の祖先を frame として
    // 特定し、frame class の出現数（商品情報 card + filter frame の 2 個）を検査する。
    const filterFrame = screen.getByText("開始日").closest(".rounded-lg.border.bg-card.p-4");
    expect(filterFrame).not.toBeNull();
    expect(container.querySelectorAll(".rounded-lg.border.bg-card.p-4")).toHaveLength(2);
  });

  it("SC3d: renders the top PaginationSummary above the table when total_count > 0", async () => {
    mockGetStockDetail.mockResolvedValue({ status: "ok", data: makeStockDetail() });
    mockListMovements.mockResolvedValue({
      status: "ok",
      data: { items: [movementItem(makeMovement())], total_count: 1, page: 1, per_page: 50 },
    });
    renderWithClient(
      <StockMovementsPage productCode="BT0002" search={{}} onSearchChange={vi.fn()} />,
    );
    expect(
      await screen.findByText("全 1 件のうち 1〜1 件を表示（1 / 1 ページ）"),
    ).toBeInTheDocument();
  });
});

describe("StockMovementsPage SPEC-UIBB-1/2（filter-empty reset action、66 §66.6）", () => {
  it("SPEC-UIBB-1 絞り込み該当なしで解除ボタンを表示する", async () => {
    mockGetStockDetail.mockResolvedValue({ status: "ok", data: makeStockDetail() });
    mockListMovements.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });
    renderWithClient(
      <StockMovementsPage
        productCode="BT0002"
        search={{ type: "disposal" }}
        onSearchChange={vi.fn()}
      />,
    );
    expect(await screen.findByRole("button", { name: "絞り込みを解除" })).toBeInTheDocument();
  });

  it("SPEC-UIBB-1 既定条件の0件では解除ボタンを出さない", async () => {
    mockGetStockDetail.mockResolvedValue({ status: "ok", data: makeStockDetail() });
    mockListMovements.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });
    renderWithClient(
      <StockMovementsPage productCode="BT0002" search={{}} onSearchChange={vi.fn()} />,
    );
    expect(await screen.findByText("在庫変動履歴がありません")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "絞り込みを解除" })).not.toBeInTheDocument();
  });

  it("SPEC-UIBB-2 解除でdateFrom・dateTo・type・pageが既定値に戻る", async () => {
    mockGetStockDetail.mockResolvedValue({ status: "ok", data: makeStockDetail() });
    mockListMovements.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 3, per_page: 50 },
    });
    const onSearchChange = vi.fn();
    renderWithClient(
      <StockMovementsPage
        productCode="BT0002"
        search={{ dateFrom: "2026-06-01", dateTo: "2026-06-30", type: "disposal", page: 3 }}
        onSearchChange={onSearchChange}
      />,
    );
    const resetButton = await screen.findByRole("button", { name: "絞り込みを解除" });
    await userEvent.setup().click(resetButton);

    const updater = onSearchChange.mock.calls[onSearchChange.mock.calls.length - 1]?.[0] as (
      prev: Record<string, unknown>,
    ) => Record<string, unknown>;
    const result = updater({
      dateFrom: "2026-06-01",
      dateTo: "2026-06-30",
      type: "disposal",
      page: 3,
    });
    expect(result).toEqual({
      dateFrom: undefined,
      dateTo: undefined,
      type: undefined,
      page: undefined,
    });
  });
});

describe("StockMovementsPage perPage scroll（UI-06c）", () => {
  it("SC9a: 表示件数変更で画面を先頭へ戻す", async () => {
    mockGetStockDetail.mockResolvedValue({ status: "ok", data: makeStockDetail() });
    mockListMovements.mockResolvedValue({
      status: "ok",
      data: { items: [movementItem(makeMovement())], total_count: 120, page: 2, per_page: 50 },
    });
    renderWithClient(
      <StockMovementsPage productCode="BT0002" search={{ page: 2 }} onSearchChange={vi.fn()} />,
    );
    await screen.findByText("廃棄・破損 #7");

    const user = userEvent.setup();
    await user.click(screen.getByRole("combobox", { name: "表示件数" }));
    await user.click(screen.getByRole("option", { name: "100 件" }));

    expect(mockScrollPageToTop).toHaveBeenCalledTimes(1);
  });
});

describe("StockMovementsPage native input tokens（Lane 5 SC4h）", () => {
  it("SC4h: 開始日/終了日/種別の3箇所すべてがbg-control-surfaceでbg-backgroundを持たない", async () => {
    mockGetStockDetail.mockResolvedValue({ status: "ok", data: makeStockDetail() });
    mockListMovements.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });
    renderWithClient(
      <StockMovementsPage productCode="BT0002" search={{}} onSearchChange={vi.fn()} />,
    );

    const fields = [
      await screen.findByLabelText("開始日"),
      screen.getByLabelText("終了日"),
      screen.getByLabelText("種別"),
    ];
    for (const field of fields) {
      expect(field).toHaveClass("bg-control-surface");
      expect(field).not.toHaveClass("bg-background");
    }
  });
});

describe("StockMovementsPage SPEC-UI06C-D9-R1（在庫照会への戻り導線）", () => {
  it.each([
    ["/stock?q=BT&status=all&selected=BT0002", "/stock?q=BT&status=all&selected=BT0002"],
    [undefined, "/stock?q=BT0002&selected=BT0002"],
    ["https://example.invalid/escape", "/stock?q=BT0002&selected=BT0002"],
    ["//example.invalid/escape", "/stock?q=BT0002&selected=BT0002"],
    // T4: app 内だが /stock でない pathname は pin 不一致として fallback へ。
    ["/inventory/records?page=2", "/stock?q=BT0002&selected=BT0002"],
    ["/stocktake?page=2", "/stock?q=BT0002&selected=BT0002"],
    ["/stock/BT0002/movements", "/stock?q=BT0002&selected=BT0002"],
  ])(
    "REQ-303 / UI-06c-D9: 在庫照会へ戻る の returnTo %s を %s へ正規化する",
    async (returnTo, expected) => {
      mockGetStockDetail.mockResolvedValue({ status: "ok", data: makeStockDetail() });
      mockListMovements.mockResolvedValue({
        status: "ok",
        data: { items: [], total_count: 0, page: 1, per_page: 50 },
      });
      renderWithClient(
        <StockMovementsPage
          productCode="BT0002"
          search={returnTo === undefined ? {} : { returnTo }}
          onSearchChange={vi.fn()}
        />,
      );
      expect(await screen.findByRole("link", { name: "在庫照会へ戻る" })).toHaveAttribute(
        "href",
        expected,
      );
    },
  );

  it("REQ-303 / UI-06c-D9 (GA5): 数字だけの商品コードでも fallback の q / selected が string で往復する", async () => {
    mockGetStockDetail.mockResolvedValue({ status: "ok", data: makeStockDetail() });
    mockListMovements.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });
    renderWithClient(
      <StockMovementsPage productCode="2099000000019" search={{}} onSearchChange={vi.fn()} />,
    );

    const link = await screen.findByRole("link", { name: "在庫照会へ戻る" });
    const url = new URL(link.getAttribute("href") ?? "", "http://x");
    // T2-num: route が実際に使う schema まで通し、number 化で落ちないことを検証する。
    const parsed = stockInquirySearchSchema.parse(defaultParseSearch(url.search));
    expect(parsed.q).toBe("2099000000019");
    expect(parsed.selected).toBe("2099000000019");
  });

  it("REQ-207 / UI-06c-D9: 元記録 link の returnTo に在庫変動履歴の returnTo を入れ子で含める", async () => {
    mockGetStockDetail.mockResolvedValue({ status: "ok", data: makeStockDetail() });
    mockListMovements.mockResolvedValue({
      status: "ok",
      data: { items: [movementItem(makeMovement())], total_count: 1, page: 2, per_page: 50 },
    });
    renderWithClient(
      <StockMovementsPage
        productCode="BT0002"
        search={{
          dateFrom: "2026-06-01",
          dateTo: "2026-06-30",
          type: "disposal",
          page: 2,
          returnTo: "/stock?q=BT&selected=BT0002",
        }}
        onSearchChange={vi.fn()}
      />,
    );
    expect(await screen.findByRole("link", { name: "廃棄・破損 #7" })).toHaveAttribute(
      "href",
      expect.stringContaining("%26returnTo%3D"),
    );
  });

  it.each(["filter 変更", "reset"])("UI-06c-D9: %s 後も returnTo が残る", async (action) => {
    mockGetStockDetail.mockResolvedValue({ status: "ok", data: makeStockDetail() });
    mockListMovements.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 3, per_page: 50 },
    });
    const prev = {
      dateFrom: "2026-06-01",
      dateTo: "2026-06-30",
      type: "disposal" as const,
      page: 3,
      returnTo: "/stock?q=BT&selected=BT0002",
    };
    const onSearchChange = vi.fn();
    renderWithClient(
      <StockMovementsPage productCode="BT0002" search={prev} onSearchChange={onSearchChange} />,
    );
    const user = userEvent.setup();
    if (action === "filter 変更") {
      await user.click(await screen.findByLabelText("種別"));
      await user.click(await screen.findByRole("option", { name: "入庫" }));
    } else {
      await user.click(await screen.findByRole("button", { name: "絞り込みを解除" }));
    }
    const updater = onSearchChange.mock.calls[onSearchChange.mock.calls.length - 1]?.[0] as (
      prev: StockMovementsSearch,
    ) => StockMovementsSearch;
    expect(updater(prev)).toEqual(
      action === "filter 変更"
        ? { ...prev, type: "receiving", page: 1 }
        : {
            dateFrom: undefined,
            dateTo: undefined,
            type: undefined,
            page: undefined,
            returnTo: prev.returnTo,
          },
    );
  });
});

describe("StockMovementsPage SPEC-NAV-RETURN-TYPES-2026-09-25 C4（detailReturnTo の直列化）", () => {
  // 「廃棄・破損 #7」の href から外側の returnTo（= detailReturnTo）を取り出す。
  async function renderDetailReturnTo(search: StockMovementsSearch): Promise<string> {
    mockGetStockDetail.mockResolvedValue({ status: "ok", data: makeStockDetail() });
    mockListMovements.mockResolvedValue({
      status: "ok",
      data: { items: [movementItem(makeMovement())], total_count: 1, page: 1, per_page: 50 },
    });
    const { unmount } = renderWithClient(
      <StockMovementsPage productCode="BT0002" search={search} onSearchChange={vi.fn()} />,
    );
    const href = (await screen.findByRole("link", { name: "廃棄・破損 #7" })).getAttribute("href");
    unmount();
    const outer = new URL(href ?? "", "http://inventory.local");
    return new URLSearchParams(outer.search).get("returnTo") ?? "";
  }

  // 受け側（在庫変動履歴 route の validateSearch と同じ schema）で search に戻す。
  function parseMovementsSearch(query: string): StockMovementsSearch {
    return stockMovementsSearchSchema.parse(defaultParseSearch(query));
  }

  async function renderBackToStockHref(search: StockMovementsSearch): Promise<string | null> {
    mockGetStockDetail.mockResolvedValue({ status: "ok", data: makeStockDetail() });
    mockListMovements.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });
    const { unmount } = renderWithClient(
      <StockMovementsPage productCode="BT0002" search={search} onSearchChange={vi.fn()} />,
    );
    const href = (await screen.findByRole("link", { name: "在庫照会へ戻る" })).getAttribute("href");
    unmount();
    return href;
  }

  // (b) の "123" は / 始まりでない不正な戻り先で、正規の戻り先（/ 始まり）では新旧の直列化に
  // 差が出ない。JSON として読める string を引用符付きで運べること（直列化の可逆性。将来 string
  // の param を足したときに number へ化ける罠）を検査するための入力。
  it.each<[string, StockMovementsSearch]>([
    [
      "(a) 期間・種別・page・数字だけの商品コードの入れ子 returnTo",
      {
        dateFrom: "2026-06-01",
        dateTo: "2026-06-30",
        type: "disposal",
        page: 2,
        returnTo: "/stock?q=%222099000000019%22&selected=%222099000000019%22",
      },
    ],
    ["(b) JSON として読める不正な returnTo", { returnTo: "123" }],
    ["(c) 既定値だけ", {}],
  ])(
    "REQ-303 / UI-06c-D9 (SPEC-NAV-RETURN-TYPES C4): 元記録 link の detailReturnTo は受け側の schema で元の search に戻る %s",
    async (_label, search) => {
      const detailReturnTo = await renderDetailReturnTo(search);
      const inner = new URL(detailReturnTo, "http://inventory.local");
      expect(inner.pathname).toBe("/stock/BT0002/movements");
      const parsed = parseMovementsSearch(inner.search);
      expect(normalizeStockMovementsSearch(parsed)).toEqual(normalizeStockMovementsSearch(search));
      expect(parsed.returnTo).toBe(search.returnTo);
      if (Object.keys(search).length === 0) {
        expect(detailReturnTo).toBe("/stock/BT0002/movements");
      }
    },
  );

  // (d) / 始まりでない不正値は新旧とも不正な戻り先で、在庫照会へ戻る先は returnTo なしと同じ
  // 既定の戻り先に着地する（詳細 link の href は新旧で変わってよい。正規の戻り先では差が出ない）。
  it.each([
    ["123", "?returnTo=123"],
    ["true", "?returnTo=true"],
    ["null", "?returnTo=null"],
    ["[]", "?returnTo=%5B%5D"],
  ])(
    "REQ-303 / UI-06c-D9 (SPEC-NAV-RETURN-TYPES C4): / 始まりでない不正値 %s でも在庫照会へ戻る先が既定の戻り先で新旧同じ",
    async (returnTo, legacyQuery) => {
      const detailReturnTo = await renderDetailReturnTo({ returnTo });
      const current = parseMovementsSearch(
        new URL(detailReturnTo, "http://inventory.local").search,
      );
      const legacy = parseMovementsSearch(legacyQuery);
      const expected = await renderBackToStockHref({});
      expect(expected).toBe("/stock?q=BT0002&selected=BT0002");
      expect(await renderBackToStockHref(current)).toBe(expected);
      expect(await renderBackToStockHref(legacy)).toBe(expected);
    },
  );
});

// REQ-303 / UI-06c-D5 / SPEC-UNIT-D3 の表の 6: 読んだ商品 1 つから単位の対応を作り、表が m で出す
it("UI-06c-D5: m の商品の在庫変動は増減 +2.5 m・変動後在庫 27.5 m、現在庫 27.5 m で出す", async () => {
  const detail = makeStockDetail();
  mockGetStockDetail.mockResolvedValue({
    status: "ok",
    data: { ...detail, product: { ...detail.product, stock_unit: "m", stock_quantity: 2750 } },
  });
  mockListMovements.mockResolvedValue({
    status: "ok",
    data: {
      items: [movementItem(makeMovement({ quantity: 250, stock_after: 2750 }))],
      total_count: 1,
      page: 1,
      per_page: 50,
    },
  });
  renderWithClient(
    <StockMovementsPage productCode="BT0002" search={{}} onSearchChange={vi.fn()} />,
  );

  expect(await screen.findByText("+2.5 m")).toBeInTheDocument();
  // 現在庫（商品の欄）と変動後在庫（表）の 2 か所
  expect(screen.getAllByText("27.5 m")).toHaveLength(2);
});

describe("StockMovementsPage の見出しと履歴（D-118 b2、66）", () => {
  it("U5 header current stock shows range and recheck badge in meters", async () => {
    // REQ-303 / SPEC-STK-CNT-D3・SPEC-UNIT-D3（Matrix U5）: 見出しの「現在庫」に範囲（長さは m）と badge、その下に案内
    const detail = makeStockDetail();
    mockGetStockDetail.mockResolvedValue({
      status: "ok",
      data: {
        ...detail,
        product: {
          ...detail.product,
          stock_unit: "m",
          stock_quantity: 450,
          stock_recheck: true,
          stock_range_low: 400,
          stock_range_high: 450,
        },
      },
    });
    mockListMovements.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });
    mockListRecheckProducts.mockResolvedValue({ status: "ok", data: [] });

    renderWithClient(
      <StockMovementsPage productCode="BT0002" search={{}} onSearchChange={vi.fn()} />,
    );

    const header = (await screen.findByText("現在庫")).parentElement;
    if (header === null) throw new Error("header not found");
    expect(header).toHaveTextContent("4〜4.5 m");
    expect(header).toHaveTextContent("要再確認");
    expect(header).not.toHaveTextContent("400");
    expect(await screen.findByText("数え直しが必要です")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "棚卸しの画面で入れ直す" })).toBeInTheDocument();
  });

  it("U6 count row keeps meters from its stock_unit when product detail fails", async () => {
    // REQ-303 / UI-06c-D3（Matrix U6）: 商品情報の取得だけが失敗しても、計数の行は行の単位で「4.5 m」
    mockGetStockDetail.mockResolvedValue({
      status: "error",
      error: {
        kind: "internal",
        message: "取得に失敗しました",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    mockListMovements.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          {
            kind: "count",
            business_date: "2026-06-27",
            settlement_no: null,
            saved_at: "2026-06-27T10:11:12",
            movement: null,
            count: {
              id: 1,
              count_value: 450,
              stock_unit: "m",
              count_kind: "default",
              stocktake_id: null,
              source: null,
            },
          },
        ],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });

    renderWithClient(
      <StockMovementsPage productCode="BT0002" search={{}} onSearchChange={vi.fn()} />,
    );

    // 商品情報の取得は hook の retry 1 の後に失敗になる（今の部分障害の test と同じ待ち方）
    await waitFor(
      () => {
        expect(screen.getByText("商品情報の取得に失敗しました")).toBeInTheDocument();
      },
      { timeout: 5000 },
    );
    expect(screen.getByText("数を確認（4.5 m、2026-06-27）")).toBeInTheDocument();
  });

  it("U9 history rows render in server order", async () => {
    // REQ-303（Matrix U9）: 画面は BIZ の順のまま描く（業務日でも保存の時刻でも並べ直さない）
    mockGetStockDetail.mockResolvedValue({ status: "ok", data: makeStockDetail() });
    const row = (id: number, businessDate: string, savedAt: string): StockHistoryItem => ({
      ...movementItem(makeMovement({ id, note: `行 ${String(id)}`, created_at: savedAt })),
      business_date: businessDate,
      saved_at: savedAt,
    });
    mockListMovements.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          row(1, "2026-06-02", "2026-06-10T09:00:00"),
          row(2, "2026-06-03", "2026-06-12T09:00:00"),
          row(3, "2026-06-01", "2026-06-11T09:00:00"),
        ],
        total_count: 3,
        page: 1,
        per_page: 50,
      },
    });

    renderWithClient(
      <StockMovementsPage productCode="BT0002" search={{}} onSearchChange={vi.fn()} />,
    );

    await screen.findByText("行 1");
    const notes = screen.getAllByText(/^行 \d$/).map((element) => element.textContent);
    expect(notes).toEqual(["行 1", "行 2", "行 3"]);
  });
});
