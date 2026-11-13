import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { screen, within } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { commands } from "@/lib/bindings";
import type {
  ManualSaleRecordDetail,
  ReceivingRecordDetail,
  ReturnRecordDetail,
} from "@/lib/bindings";
import { renderWithRouter } from "@/test/render-with-router";
import { ManualSaleRecordDetailPage } from "./ManualSaleRecordDetailPage";
import { ReceivingRecordDetailPage } from "./ReceivingRecordDetailPage";
import { ReturnRecordDetailPage } from "./ReturnRecordDetailPage";

vi.mock("@/lib/bindings", () => ({
  commands: {
    getReceivingRecord: vi.fn(),
    getReturnRecord: vi.fn(),
    getManualSaleRecord: vi.fn(),
  },
}));

const mockGetReceivingRecord = vi.mocked(commands.getReceivingRecord);
const mockGetReturnRecord = vi.mocked(commands.getReturnRecord);
const mockGetManualSaleRecord = vi.mocked(commands.getManualSaleRecord);

function renderWithClient(ui: ReactNode) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  return renderWithRouter(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>);
}

function makeReceivingDetail(): ReceivingRecordDetail {
  return {
    id: 12,
    receiving_date: "2026-06-27",
    supplier_id: 1,
    supplier_name: "テスト商事",
    note: "午前便",
    status: "active",
    created_at: "2026-06-27T09:00:00",
    total_cost: 1221,
    items: [
      {
        id: 1,
        product_code: "RCV-001",
        product_name: "入庫テスト商品 A",
        department_name: "毛糸",
        stock_unit: "pcs",
        quantity: 2,
        cost_price_centi: 11100,
        line_cost_centi: "22200",
      },
      {
        id: 2,
        product_code: "RCV-002",
        product_name: "入庫テスト商品 B",
        department_name: "ボタン",
        stock_unit: "pcs",
        quantity: 3,
        cost_price_centi: 33300,
        line_cost_centi: "99900",
      },
    ],
    movements: [
      {
        id: 101,
        product_code: "RCV-001",
        movement_type: "receiving",
        quantity: 2,
        stock_after: 12,
        reference_type: "receiving_record",
        reference_id: 12,
        source: { label: "入庫 #12", route: "/inventory/receiving/records/12" },
        note: null,
        created_at: "2026-06-27T09:00:00",
      },
    ],
  };
}

function makeReturnDetail(): ReturnRecordDetail {
  return {
    id: 22,
    return_type: "exchange",
    return_date: "2026-06-27",
    register_processed: false,
    receipt_image_path: "receipts/20260627.png",
    note: "サイズ交換",
    status: "active",
    created_at: "2026-06-27T10:00:00",
    items: [
      {
        id: 1,
        product_code: "RTN-001",
        product_name: "返品テスト商品",
        department_name: "ボタン",
        stock_unit: "pcs",
        direction: "in",
        quantity: 1,
      },
    ],
    movements: [
      {
        id: 201,
        product_code: "RTN-001",
        movement_type: "return",
        quantity: 1,
        stock_after: 6,
        reference_type: "return_record",
        reference_id: 22,
        source: { label: "返品・交換 #22", route: "/inventory/return/records/22" },
        note: null,
        created_at: "2026-06-27T10:00:00",
      },
    ],
  };
}

function makeManualSaleDetail(): ManualSaleRecordDetail {
  return {
    id: 32,
    sale_date: "2026-06-27",
    reason: "plu_unregistered",
    note: "店頭販売",
    status: "active",
    created_at: "2026-06-27T11:00:00",
    total_amount: 980,
    items: [
      {
        id: 1,
        product_code: "MS-001",
        product_name: "手動販売テスト商品",
        department_name: "布",
        stock_unit: "pcs",
        quantity: 1,
        amount: 980,
      },
    ],
    movements: [
      {
        id: 301,
        product_code: "MS-001",
        movement_type: "sale_manual",
        quantity: -1,
        stock_after: 4,
        reference_type: "manual_sale",
        reference_id: 32,
        source: { label: "手動販売 #32", route: "/inventory/manual-sale/records/32" },
        note: null,
        created_at: "2026-06-27T11:00:00",
      },
    ],
  };
}

beforeEach(() => {
  mockGetReceivingRecord.mockReset();
  mockGetReturnRecord.mockReset();
  mockGetManualSaleRecord.mockReset();
});

describe("other inventory record detail pages (REQ-201 / REQ-202 / REQ-203 / REQ-206)", () => {
  it("REQ-201: 入庫記録詳細に明細、原価合計、関連movementを表示する", async () => {
    mockGetReceivingRecord.mockResolvedValue({ status: "ok", data: makeReceivingDetail() });

    renderWithClient(
      <ReceivingRecordDetailPage
        recordId={12}
        returnTo="/inventory/records?recordType=receiving_record"
      />,
    );

    expect(await screen.findByRole("heading", { name: "入庫記録 #12" })).toBeInTheDocument();
    expect(screen.getByText("テスト商事")).toBeInTheDocument();
    expect(screen.getByText("¥1,221")).toBeInTheDocument();
    const firstItem = screen.getByRole("row", { name: /RCV-001/ });
    expect(within(firstItem).getByText("入庫テスト商品 A")).toBeInTheDocument();
    expect(within(firstItem).getByText("¥111")).toBeInTheDocument();
    expect(within(firstItem).getByText("222.00 円")).toBeInTheDocument();
    const secondItem = screen.getByRole("row", { name: /RCV-002/ });
    expect(within(secondItem).getByText("入庫テスト商品 B")).toBeInTheDocument();
    expect(within(secondItem).getByText("¥333")).toBeInTheDocument();
    expect(within(secondItem).getByText("999.00 円")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "RCV-001 の在庫変動履歴" })).toHaveAttribute(
      "href",
      "/stock/RCV-001/movements",
    );
    expect(screen.getByRole("link", { name: "前の画面へ戻る" })).toHaveAttribute(
      "href",
      "/inventory/records?recordType=receiving_record",
    );
  });

  it.each([
    [
      "/stock/RCV-001/movements?type=receiving&page=2",
      "/stock/RCV-001/movements?type=receiving&page=2",
    ],
    ["https://example.invalid/escape", "/inventory/records"],
    ["//example.invalid/escape", "/inventory/records"],
    // T3: router 形式（defaultStringifySearch で作った値、数字だけの q を JSON-quote 済み）は
    // 描画 href が入力と文字列一致する（DSR-17 (b)）。
    [
      "/inventory/records?recordType=receiving_record&page=2&q=%222099000000019%22",
      "/inventory/records?recordType=receiving_record&page=2&q=%222099000000019%22",
    ],
  ])(
    "T11 DSR-18: ReceivingRecordDetailPage の returnTo %s を安全に %s へ正規化する",
    async (returnTo, expected) => {
      mockGetReceivingRecord.mockResolvedValue({ status: "ok", data: makeReceivingDetail() });

      renderWithClient(<ReceivingRecordDetailPage recordId={12} returnTo={returnTo} />);

      expect(await screen.findByRole("link", { name: "前の画面へ戻る" })).toHaveAttribute(
        "href",
        expected,
      );
    },
  );

  it("REQ-202: 返品・交換詳細にレジでの戻し方、レシート有無、方向付き明細を表示する", async () => {
    mockGetReturnRecord.mockResolvedValue({ status: "ok", data: makeReturnDetail() });

    renderWithClient(<ReturnRecordDetailPage recordId={22} />);

    expect(await screen.findByRole("heading", { name: "返品・交換 #22" })).toBeInTheDocument();
    expect(screen.getByText("交換")).toBeInTheDocument();
    // UI-03-D23 (U6): 記録の値は false を「バーコードなし」で出す
    expect(screen.getByText("レジでの戻し方")).toBeInTheDocument();
    expect(screen.getByText("バーコードなし")).toBeInTheDocument();
    expect(screen.getByText("レシート画像")).toBeInTheDocument();
    expect(screen.getByText("添付あり")).toBeInTheDocument();
    const noteRegion = screen.getByRole("region", { name: "備考" });
    expect(within(noteRegion).getByText("サイズ交換")).toBeInTheDocument();
    expect(screen.getByText("戻り（在庫+）")).toBeInTheDocument();
    expect(screen.getByText("返品テスト商品")).toBeInTheDocument();
  });

  it("REQ-202 / UI-03-D23 (U6): バーコードで戻した記録で在庫変動が 0 件なら、Z004 の取込みで戻ることを示す", async () => {
    const detail = makeReturnDetail();
    detail.register_processed = true;
    detail.movements = [];
    mockGetReturnRecord.mockResolvedValue({ status: "ok", data: detail });

    renderWithClient(<ReturnRecordDetailPage recordId={22} />);

    expect(await screen.findByText("バーコードで戻した")).toBeInTheDocument();
    expect(screen.getByText("関連する在庫変動がありません")).toBeInTheDocument();
    expect(
      screen.getByText(
        "レジでバーコードを読んで戻した記録で、どの商品も売上データ（Z004）の取込みで在庫に戻ります",
      ),
    ).toBeInTheDocument();
  });

  it.each([
    ["/stock/RTN-001/movements?type=return&page=2", "/stock/RTN-001/movements?type=return&page=2"],
    ["https://example.invalid/escape", "/inventory/records"],
    ["//example.invalid/escape", "/inventory/records"],
  ])(
    "T11 DSR-18: ReturnRecordDetailPage の returnTo %s を安全に %s へ正規化する",
    async (returnTo, expected) => {
      mockGetReturnRecord.mockResolvedValue({ status: "ok", data: makeReturnDetail() });

      renderWithClient(<ReturnRecordDetailPage recordId={22} returnTo={returnTo} />);

      expect(await screen.findByRole("link", { name: "前の画面へ戻る" })).toHaveAttribute(
        "href",
        expected,
      );
    },
  );

  it("REQ-202/UI-03-D19: ⑮ SC13: 返品・交換詳細は空備考を「—」で独立表示する", async () => {
    mockGetReturnRecord.mockResolvedValue({
      status: "ok",
      data: { ...makeReturnDetail(), note: null },
    });

    renderWithClient(<ReturnRecordDetailPage recordId={22} />);

    expect(await screen.findByRole("heading", { name: "返品・交換 #22" })).toBeInTheDocument();
    const noteRegion = screen.getByRole("region", { name: "備考" });
    expect(within(noteRegion).getByText("—")).toBeInTheDocument();
    expect(within(noteRegion).queryByText("備考なし")).not.toBeInTheDocument();
  });

  it("REQ-203: 手動販売詳細に販売金額合計、日次売上リンク、関連movementを表示する", async () => {
    mockGetManualSaleRecord.mockResolvedValue({ status: "ok", data: makeManualSaleDetail() });

    renderWithClient(<ManualSaleRecordDetailPage recordId={32} />);

    expect(await screen.findByRole("heading", { name: "手動販売出庫 #32" })).toBeInTheDocument();
    expect(screen.getByText("PLU未登録商品の販売")).toBeInTheDocument();
    expect(screen.getAllByText("¥980").length).toBeGreaterThan(0);
    expect(screen.getByText("手動販売テスト商品")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "日次売上で確認" })).toHaveAttribute(
      "href",
      "/reports/daily?date=2026-06-27",
    );
    expect(screen.getByText("-1")).toBeInTheDocument();
    expect(screen.getByText("減少")).toBeInTheDocument();
  });

  it.each([
    [
      "/stock/MS-001/movements?type=sale_manual&page=2",
      "/stock/MS-001/movements?type=sale_manual&page=2",
    ],
    ["https://example.invalid/escape", "/inventory/records"],
    ["//example.invalid/escape", "/inventory/records"],
  ])(
    "T11 DSR-18: ManualSaleRecordDetailPage の returnTo %s を安全に %s へ正規化する",
    async (returnTo, expected) => {
      mockGetManualSaleRecord.mockResolvedValue({ status: "ok", data: makeManualSaleDetail() });

      renderWithClient(<ManualSaleRecordDetailPage recordId={32} returnTo={returnTo} />);

      expect(await screen.findByRole("link", { name: "前の画面へ戻る" })).toHaveAttribute(
        "href",
        expected,
      );
    },
  );
});

it.each([
  ["pcs", "1,234 個"],
  ["m", "12.34 m"],
] as const)("⑮ SC19: 入庫・返品・手動販売詳細の %s を単位付き表示する", async (unit, expected) => {
  const receiving = makeReceivingDetail();
  receiving.items = [{ ...receiving.items[0], stock_unit: unit, quantity: 1234 }];
  mockGetReceivingRecord.mockResolvedValue({ status: "ok", data: receiving });
  const r = renderWithClient(<ReceivingRecordDetailPage recordId={12} />);
  expect(await screen.findByText(expected)).toBeInTheDocument();
  r.unmount();
  const returns = makeReturnDetail();
  returns.items = [{ ...returns.items[0], stock_unit: unit, quantity: 1234 }];
  mockGetReturnRecord.mockResolvedValue({ status: "ok", data: returns });
  const t = renderWithClient(<ReturnRecordDetailPage recordId={22} />);
  expect(await screen.findByText(expected)).toBeInTheDocument();
  t.unmount();
  const sales = makeManualSaleDetail();
  sales.items = [{ ...sales.items[0], stock_unit: unit, quantity: 1234 }];
  mockGetManualSaleRecord.mockResolvedValue({ status: "ok", data: sales });
  renderWithClient(<ManualSaleRecordDetailPage recordId={32} />);
  expect(await screen.findByText(expected)).toBeInTheDocument();
});

// REQ-207 / UI-06c-D5 / SPEC-UNIT-D3 の表の 1・6、SPEC-UNIT-D6: 記録詳細は行ごとの単位で数量と
// 「関連する在庫変動」を出し、行の金額は BIZ が返した 1/100 円の文字列を出す
describe("記録詳細の単位つきの数量と在庫変動（REQ-207 / UI-06c-D5）", () => {
  const movementCells = () =>
    within(
      screen.getByRole("heading", { name: "関連する在庫変動" }).closest("section") ?? document.body,
    )
      .getAllByRole("row")
      .slice(1)
      .map((row) => {
        const tds = row.querySelectorAll("td");
        return [tds[2].textContent, tds[3].textContent];
      });

  it("REQ-201: 入庫の記録詳細は m の行が 2.5 m・832.50 円、関連する在庫変動は行ごとの単位", async () => {
    const detail = makeReceivingDetail();
    const [first, second] = detail.items;
    const [movement] = detail.movements;
    detail.total_cost = 833;
    detail.items = [
      {
        ...first,
        product_code: "RCV-M",
        stock_unit: "m",
        quantity: 250,
        cost_price_centi: 33300,
        line_cost_centi: "83250",
      },
      {
        ...second,
        product_code: "RCV-PCS",
        stock_unit: "pcs",
        quantity: 3,
        cost_price_centi: 0,
        line_cost_centi: "0",
      },
    ];
    detail.movements = [
      { ...movement, id: 1, product_code: "RCV-M", quantity: 250, stock_after: 2750 },
      { ...movement, id: 2, product_code: "RCV-PCS", quantity: 3, stock_after: 13 },
    ];
    mockGetReceivingRecord.mockResolvedValue({ status: "ok", data: detail });
    renderWithClient(<ReceivingRecordDetailPage recordId={12} />);

    const row = await screen.findByRole("row", { name: /RCV-M/ });
    expect(within(row).getByText("2.5 m")).toBeInTheDocument();
    expect(within(row).getByText("¥333")).toBeInTheDocument();
    expect(within(row).getByText("832.50 円")).toBeInTheDocument();
    expect(screen.getByText("¥833")).toBeInTheDocument();
    expect(
      within(screen.getByRole("row", { name: /RCV-PCS/ })).getByText("0.00 円"),
    ).toBeInTheDocument();
    expect(movementCells()).toEqual([
      ["+2.5 m増加", "27.5 m"],
      ["+3増加", "13"],
    ]);
  });

  it("REQ-201: BIZ が行の金額を埋め忘れた空の文字列は 0.00 円ではなく —", async () => {
    const detail = makeReceivingDetail();
    const [first] = detail.items;
    detail.items = [{ ...first, line_cost_centi: "" }];
    mockGetReceivingRecord.mockResolvedValue({ status: "ok", data: detail });
    renderWithClient(<ReceivingRecordDetailPage recordId={12} />);
    const row = await screen.findByRole("row", { name: /RCV-001/ });
    expect(within(row).getByText("—")).toBeInTheDocument();
    expect(within(row).queryByText("0.00 円")).toBeNull();
  });

  it("REQ-202 / REQ-203: 返品・手動販売の記録詳細の関連する在庫変動も明細の単位で出す", async () => {
    const returns = makeReturnDetail();
    const [returnItem] = returns.items;
    const [returnMovement] = returns.movements;
    returns.items = [{ ...returnItem, product_code: "RET-M", stock_unit: "m", quantity: 30 }];
    returns.movements = [
      { ...returnMovement, product_code: "RET-M", quantity: 30, stock_after: 2600 },
    ];
    mockGetReturnRecord.mockResolvedValue({ status: "ok", data: returns });
    const rendered = renderWithClient(<ReturnRecordDetailPage recordId={22} />);
    expect(await screen.findByText("0.3 m")).toBeInTheDocument();
    expect(movementCells()).toEqual([["+0.3 m増加", "26 m"]]);
    rendered.unmount();

    const sales = makeManualSaleDetail();
    const [saleItem] = sales.items;
    const [saleMovement] = sales.movements;
    sales.items = [{ ...saleItem, product_code: "MS-M", stock_unit: "m", quantity: 130 }];
    sales.movements = [
      { ...saleMovement, product_code: "MS-M", quantity: -130, stock_after: 2620 },
    ];
    mockGetManualSaleRecord.mockResolvedValue({ status: "ok", data: sales });
    renderWithClient(<ManualSaleRecordDetailPage recordId={32} />);
    expect(await screen.findByText("1.3 m")).toBeInTheDocument();
    expect(movementCells()).toEqual([["-1.3 m減少", "26.2 m"]]);
  });
});
