import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, createRouter, RouterProvider } from "@tanstack/react-router";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { commands, type ReceivingRecordDetail } from "@/lib/bindings";
import {
  makeMockProductWithRelations,
  makeMockStockDetail,
} from "@/features/stock-inquiry/lib/test-fixtures";
import { routeTree } from "@/routeTree.gen";

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ setTitle: vi.fn().mockResolvedValue(undefined) }),
}));

vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => ({ setZoom: vi.fn().mockResolvedValue(undefined) }),
}));

vi.mock("@/lib/bindings", () => ({
  commands: {
    listLogs: vi.fn(),
    listLogOperationTypes: vi.fn(),
    getReceivingRecord: vi.fn(),
    listInventoryRecords: vi.fn(),
    listDepartments: vi.fn(),
    searchProducts: vi.fn(),
    listLowStock: vi.fn(),
    getStockDetail: vi.fn(),
    listMovements: vi.fn(),
    getDisposalRecord: vi.fn(),
  },
}));

const listLogs = vi.mocked(commands.listLogs);
const listLogOperationTypes = vi.mocked(commands.listLogOperationTypes);
const getReceivingRecord = vi.mocked(commands.getReceivingRecord);
const listInventoryRecords = vi.mocked(commands.listInventoryRecords);
const listDepartments = vi.mocked(commands.listDepartments);
const searchProducts = vi.mocked(commands.searchProducts);
const listLowStock = vi.mocked(commands.listLowStock);
const getStockDetail = vi.mocked(commands.getStockDetail);
const listMovements = vi.mocked(commands.listMovements);
const getDisposalRecord = vi.mocked(commands.getDisposalRecord);

function receivingDetail(): ReceivingRecordDetail {
  return {
    id: 12,
    receiving_date: "2026-07-15",
    supplier_id: null,
    supplier_name: null,
    note: "synthetic",
    status: "active",
    created_at: "2026-07-15T10:00:00",
    total_cost: 120,
    items: [
      {
        id: 1,
        product_code: "SYN-001",
        product_name: "合成テスト商品",
        department_name: "テスト部門",
        stock_unit: "pcs",
        quantity: 1,
        cost_price_centi: 120,
        line_cost_centi: "12000",
      },
    ],
    movements: [],
  };
}

beforeEach(() => {
  listLogs.mockReset();
  listLogOperationTypes.mockReset();
  getReceivingRecord.mockReset();
  listInventoryRecords.mockReset();
  listDepartments.mockReset();
  searchProducts.mockReset();
  listLowStock.mockReset();
  getStockDetail.mockReset();
  listMovements.mockReset();
  getDisposalRecord.mockReset();
  listLogOperationTypes.mockResolvedValue({ status: "ok", data: ["backup_create"] });
  listLogs.mockResolvedValue({
    status: "ok",
    data: {
      items: [
        {
          id: 1,
          operation_type: "backup_create",
          summary: "合成調査ログ",
          detail_json: '{"record_type":"receiving_record","record_id":12}',
          created_at: "2026-07-15T10:00:00",
        },
      ],
      total_count: 61,
      page: 4,
      per_page: 20,
    },
  });
  getReceivingRecord.mockResolvedValue({ status: "ok", data: receivingDetail() });
  listDepartments.mockResolvedValue({ status: "ok", data: [] });
  listInventoryRecords.mockResolvedValue({
    status: "ok",
    data: {
      items: [
        {
          record_type: "receiving_record",
          record_id: 12,
          business_date: "2026-07-15",
          representative_item: "合成テスト商品",
          item_count: 1,
          status: "active",
          created_at: "2026-07-15T10:00:00",
          detail_route: "/inventory/receiving/records/12",
        },
      ],
      total_count: 1,
      page: 1,
      per_page: 50,
    },
  });
});

describe("REQ-207 / UI-11c-D16 / DSR-18 returnTo route flow", () => {
  it("T9 REQ-207 / DSR-15: 解析不能になる returnTo でも業務記録詳細が描画され、前の画面へ戻る が既定 hub を指す", async () => {
    const history = createMemoryHistory({
      initialEntries: ["/inventory/receiving/records/12?returnTo=%2Fa%2F..%2F%2F%5B"],
    });
    const router = createRouter({ routeTree, history });
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
    });
    render(
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    );

    expect(await screen.findByRole("heading", { name: "入庫記録 #12" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "前の画面へ戻る" })).toHaveAttribute(
      "href",
      "/inventory/records",
    );
  });

  it("T10 TRACE-D11: pushes from filtered logs to detail and back to the exact source href", async () => {
    const sourceHref =
      "/settings/logs?start_date=2026-07-01&end_date=2026-07-31&operation_type=backup_create&page=4";
    const history = createMemoryHistory({ initialEntries: [sourceHref] });
    const router = createRouter({ routeTree, history });
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
    });
    render(
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    );
    const user = userEvent.setup();

    await user.click(await screen.findByRole("button", { name: "詳細を表示" }));
    await user.click(screen.getByRole("link", { name: "関連記録を見る" }));

    await waitFor(() => {
      expect(router.state.location.href).toBe(
        "/inventory/receiving/records/12?returnTo=%2Fsettings%2Flogs%3Fstart_date%3D2026-07-01%26end_date%3D2026-07-31%26operation_type%3Dbackup_create%26page%3D4",
      );
    });
    expect(history.length).toBe(2);
    expect(await screen.findByRole("heading", { name: "入庫記録 #12" })).toBeInTheDocument();

    await user.click(screen.getByRole("link", { name: "前の画面へ戻る" }));

    await waitFor(() => {
      expect(router.state.location.href).toBe(sourceHref);
    });
    expect(history.length).toBe(3);
    expect(await screen.findByText("合成調査ログ")).toBeInTheDocument();
  });

  it("T8 REQ-207 / REQ-303: 入出庫履歴（数字だけの検索語）→ 詳細 → 前の画面へ戻る で検索語が残り、戻った先の location.href が出発時の href と文字列一致する", async () => {
    // S5: SearchBar → navigate({search}) が作る href を模す。数字だけの q は JSON-quote
    // されるため url は q=%222099000000019%22 になる（起票時実測 Probe 2）。
    const sourceHref = "/inventory/records?q=%222099000000019%22";
    const history = createMemoryHistory({ initialEntries: [sourceHref] });
    const router = createRouter({ routeTree, history });
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
    });
    render(
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    );
    const user = userEvent.setup();

    await user.click(await screen.findByRole("link", { name: "詳細を見る" }));

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/inventory/receiving/records/12");
    });
    expect(await screen.findByRole("heading", { name: "入庫記録 #12" })).toBeInTheDocument();

    await user.click(screen.getByRole("link", { name: "前の画面へ戻る" }));

    await waitFor(() => {
      expect(router.state.location.href).toBe(sourceHref);
    });
    expect(router.state.location.search).toEqual({ q: "2099000000019" });
    expect(await screen.findByText("合成テスト商品")).toBeInTheDocument();
  });
  it("T7 SPEC-NAV-RETURN-TYPES C5 REQ-207 / REQ-303: 在庫照会（数字だけの商品コード）→ 在庫変動履歴 → 業務記録詳細 → 前の画面へ戻る → 在庫照会へ戻る で各段の href が行きと文字列一致する", async () => {
    const code = "2099000000019";
    const product = makeMockProductWithRelations({
      product_code: code,
      name: "合成数字コード商品",
    });
    searchProducts.mockResolvedValue({
      status: "ok",
      data: { items: [product], total_count: 1, page: 1, per_page: 50 },
    });
    listLowStock.mockResolvedValue({ status: "ok", data: [] });
    getStockDetail.mockResolvedValue({ status: "ok", data: makeMockStockDetail({ product }) });
    listMovements.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          {
            kind: "movement",
            business_date: "2026-06-27",
            settlement_no: null,
            saved_at: "2026-06-27T10:11:12",
            count: null,
            movement: {
              id: 10,
              product_code: code,
              movement_type: "disposal",
              quantity: -1,
              stock_after: 9,
              reference_type: "disposal_record",
              reference_id: 7,
              source: { label: "廃棄・破損 #7", route: "/inventory/disposal/records/7" },
              note: "synthetic",
              created_at: "2026-06-27T10:11:12",
            },
          },
        ],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });
    getDisposalRecord.mockResolvedValue({
      status: "ok",
      data: {
        id: 7,
        disposal_date: "2026-06-27",
        status: "active",
        created_at: "2026-06-27T10:30:00",
        total_loss_cost: 300,
        items: [
          {
            id: 1,
            product_code: code,
            product_name: "合成数字コード商品",
            department_name: "毛糸",
            stock_unit: "pcs",
            disposal_type: "damage",
            quantity: 1,
            cost_price_centi: 300,
            reason: "synthetic",
            line_loss_cost_centi: "30000",
          },
        ],
        movements: [],
      },
    });

    // router の直列化と同じ形（数字だけの q / selected は JSON-quote される）。
    const stockHref = `/stock?q=%22${code}%22&selected=%22${code}%22`;
    const history = createMemoryHistory({ initialEntries: [stockHref] });
    const router = createRouter({ routeTree, history });
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
    });
    render(
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    );
    const user = userEvent.setup();

    await user.click(await screen.findByRole("link", { name: "在庫変動履歴" }));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe(`/stock/${code}/movements`);
    });
    const movementsHref = router.state.location.href;

    await user.click(await screen.findByRole("link", { name: "廃棄・破損 #7" }));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/inventory/disposal/records/7");
    });
    expect(await screen.findByRole("heading", { name: "廃棄・破損 #7" })).toBeInTheDocument();

    await user.click(screen.getByRole("link", { name: "前の画面へ戻る" }));
    await waitFor(() => {
      expect(router.state.location.href).toBe(movementsHref);
    });

    await user.click(await screen.findByRole("link", { name: "在庫照会へ戻る" }));
    await waitFor(() => {
      expect(router.state.location.href).toBe(stockHref);
    });
    expect(router.state.location.search).toEqual({ q: code, selected: code });
  });
});
