import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { commands } from "@/lib/bindings";
import type { DisposalRecordDetail } from "@/lib/bindings";
import { renderWithRouter } from "@/test/render-with-router";
import { DisposalRecordDetailPage } from "./DisposalRecordDetailPage";

vi.mock("@/lib/bindings", () => ({
  commands: {
    getDisposalRecord: vi.fn(),
  },
}));

const mockGetDisposalRecord = vi.mocked(commands.getDisposalRecord);

function renderWithClient(ui: ReactNode) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  return renderWithRouter(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>);
}

function makeDetail(overrides: Partial<DisposalRecordDetail> = {}): DisposalRecordDetail {
  return {
    id: 7,
    disposal_date: "2026-06-27",
    status: "active",
    created_at: "2026-06-27T10:30:00",
    total_loss_cost: 480,
    items: [
      {
        id: 1,
        product_code: "DP-001",
        product_name: "破損ボタン",
        department_name: "ボタン",
        stock_unit: "pcs",
        disposal_type: "damage",
        quantity: 2,
        cost_price_centi: 12000,
        reason: "袋破れ",
        line_loss_cost_centi: "24000",
      },
      {
        id: 2,
        product_code: "DP-002",
        product_name: "期限切れ接着剤",
        department_name: "手芸等材料",
        stock_unit: "pcs",
        disposal_type: "disposal",
        quantity: 3,
        cost_price_centi: 8000,
        reason: "期限切れ",
        line_loss_cost_centi: "24000",
      },
    ],
    movements: [
      {
        id: 11,
        product_code: "DP-001",
        movement_type: "disposal",
        quantity: -2,
        stock_after: 8,
        reference_type: "disposal_record",
        reference_id: 7,
        source: { label: "廃棄・破損 #7", route: "/inventory/disposal/records/7" },
        note: "袋破れ",
        created_at: "2026-06-27T10:31:00",
      },
    ],
    ...overrides,
  };
}

beforeEach(() => {
  mockGetDisposalRecord.mockReset();
});

describe("DisposalRecordDetailPage (REQ-204 / REQ-206)", () => {
  it("REQ-204: 廃棄・破損詳細に明細、ロス原価合計、関連movementを表示する", async () => {
    mockGetDisposalRecord.mockResolvedValue({ status: "ok", data: makeDetail() });

    renderWithClient(<DisposalRecordDetailPage recordId={7} />);

    expect(await screen.findByRole("heading", { name: "廃棄・破損 #7" })).toBeInTheDocument();
    expect(screen.getByText("有効")).toBeInTheDocument();
    expect(screen.getByText("¥480")).toBeInTheDocument();
    expect(screen.getByText("破損ボタン")).toBeInTheDocument();
    expect(screen.getAllByText("袋破れ").length).toBeGreaterThan(0);
    expect(screen.getByText("期限切れ接着剤")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "DP-001 の在庫変動履歴" })).toHaveAttribute(
      "href",
      "/stock/DP-001/movements",
    );
    expect(screen.getByText("-2")).toBeInTheDocument();
    expect(screen.getByText("減少")).toBeInTheDocument();
  });

  it("REQ-206: 存在しない廃棄・破損詳細は戻り導線付きで表示する", async () => {
    mockGetDisposalRecord.mockResolvedValue({
      status: "error",
      error: {
        kind: "not_found",
        message: "廃棄・破損記録が見つかりません",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });

    renderWithClient(<DisposalRecordDetailPage recordId={404} />);

    expect(await screen.findByText("廃棄・破損記録が見つかりません")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "前の画面へ戻る" })).toHaveAttribute(
      "href",
      "/inventory/records",
    );
  });

  it.each([
    [
      "/stock/DP-001/movements?type=disposal&page=2",
      "/stock/DP-001/movements?type=disposal&page=2",
    ],
    ["https://example.invalid/escape", "/inventory/records"],
    ["//example.invalid/escape", "/inventory/records"],
  ])(
    "REQ-207 / T11 DSR-18: DisposalRecordDetailPage の returnTo %s を安全に %s へ正規化する",
    async (returnTo, expected) => {
      mockGetDisposalRecord.mockResolvedValue({ status: "ok", data: makeDetail() });

      renderWithClient(<DisposalRecordDetailPage recordId={7} returnTo={returnTo} />);

      expect(await screen.findByRole("heading", { name: "廃棄・破損 #7" })).toBeInTheDocument();
      expect(screen.getByRole("link", { name: "前の画面へ戻る" })).toHaveAttribute(
        "href",
        expected,
      );
    },
  );
});

it.each([
  ["pcs", "1,234 個"],
  ["m", "12.34 m"],
] as const)("⑮ SC19: 廃棄詳細の %s を単位付き表示する", async (unit, expected) => {
  const detail = makeDetail();
  detail.items = [{ ...detail.items[0], stock_unit: unit, quantity: 1234 }];
  mockGetDisposalRecord.mockResolvedValue({ status: "ok", data: detail });
  renderWithClient(<DisposalRecordDetailPage recordId={7} />);
  expect(await screen.findByText(expected)).toBeInTheDocument();
});

// REQ-204 / SPEC-UNIT-D3 の表の 1・6、SPEC-UNIT-D6: 廃棄の記録詳細は行の単位で数量と在庫変動を出し、
// 行のロス原価は BIZ が返した 1/100 円の文字列を出す
it("REQ-204: m の商品の廃棄は 0.5 m・166.50 円、合計 ¥167、関連する在庫変動は -0.5 m・25.7 m", async () => {
  const detail = makeDetail();
  const [item] = detail.items;
  const [movement] = detail.movements;
  detail.total_loss_cost = 167;
  detail.items = [
    {
      ...item,
      product_code: "DP-M",
      stock_unit: "m",
      quantity: 50,
      cost_price_centi: 33300,
      line_loss_cost_centi: "16650",
    },
  ];
  detail.movements = [{ ...movement, product_code: "DP-M", quantity: -50, stock_after: 2570 }];
  mockGetDisposalRecord.mockResolvedValue({ status: "ok", data: detail });
  renderWithClient(<DisposalRecordDetailPage recordId={7} />);

  expect(await screen.findByText("0.5 m")).toBeInTheDocument();
  expect(screen.getByText("¥333")).toBeInTheDocument();
  expect(screen.getByText("166.50 円")).toBeInTheDocument();
  expect(screen.getByText("¥167")).toBeInTheDocument();
  expect(screen.getByText("-0.5 m")).toBeInTheDocument();
  expect(screen.getByText("25.7 m")).toBeInTheDocument();
});
