import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  Outlet,
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  defaultParseSearch,
} from "@tanstack/react-router";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState, type ReactNode } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { commands } from "@/lib/bindings";
import type { InventoryRecordSummary } from "@/lib/bindings";
import { scrollPageToTop } from "@/lib/page-scroll";
import { renderWithRouter } from "@/test/render-with-router";
import { InventoryRecordsPage } from "./InventoryRecordsPage";
import {
  formatRecordStatus,
  inventoryRecordsSearchSchema,
  type InventoryRecordsSearch,
} from "./types";

vi.mock("@/lib/bindings", () => ({
  commands: {
    listDepartments: vi.fn(),
    listInventoryRecords: vi.fn(),
  },
}));
vi.mock("@/lib/page-scroll", () => ({ scrollPageToTop: vi.fn() }));

const mockListDepartments = vi.mocked(commands.listDepartments);
const mockListInventoryRecords = vi.mocked(commands.listInventoryRecords);
const mockScrollPageToTop = vi.mocked(scrollPageToTop);

function renderWithClient(ui: ReactNode, initialPath?: string) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  return renderWithRouter(
    <QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>,
    initialPath,
  );
}

function renderWithDetailRoutes() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  const rootRoute = createRootRoute({ component: Outlet });
  const recordsRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/",
    component: () => <InventoryRecordsPage search={{}} onSearchChange={vi.fn()} />,
  });
  const csvDetailRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/csv-import/records/$importId",
    component: () => <div>CSV取込み詳細画面</div>,
  });
  const stocktakeDetailRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/stocktake/records/$stocktakeId",
    component: () => <div>棚卸し詳細画面</div>,
  });
  const router = createRouter({
    routeTree: rootRoute.addChildren([recordsRoute, csvDetailRoute, stocktakeDetailRoute]),
    history: createMemoryHistory({ initialEntries: ["/"] }),
  });
  return {
    router,
    ...render(
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    ),
  };
}

function makeRecord(overrides: Partial<InventoryRecordSummary> = {}): InventoryRecordSummary {
  return {
    record_type: "disposal_record",
    record_id: 7,
    business_date: "2026-06-27",
    representative_item: "ボタン #02",
    item_count: 2,
    status: "active",
    created_at: "2026-06-27T10:30:00",
    detail_route: "/inventory/disposal/records/7",
    ...overrides,
  };
}

beforeEach(() => {
  mockListDepartments.mockReset();
  mockListInventoryRecords.mockReset();
  mockScrollPageToTop.mockReset();
  mockListDepartments.mockResolvedValue({
    status: "ok",
    data: [
      {
        id: 2,
        name: "ボタン",
        z005_name: "ボタン",
        code_prefix: "BT",
        next_seq: 1,
        created_at: "2026-06-27T10:00:00",
      },
    ],
  });
});

afterEach(() => {
  vi.useRealTimers();
});

describe("InventoryRecordsPage (REQ-206)", () => {
  it("REQ-206: 記録種別フィルターで7種の業務記録を選べる（U14: 廃棄・破損の次に詰め替え・切り分け）", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });

    renderWithClient(<InventoryRecordsPage search={{}} onSearchChange={vi.fn()} />);

    const user = userEvent.setup();
    const recordType = await screen.findByLabelText("記録種別");
    expect(recordType).toHaveAttribute("data-slot", "select-trigger");
    expect(recordType.tagName).toBe("BUTTON");
    await user.click(recordType);
    expect((await screen.findAllByRole("option")).map((option) => option.textContent)).toEqual([
      "すべて",
      "入庫",
      "返品・交換",
      "手動販売出庫",
      "廃棄・破損",
      "詰め替え・切り分け",
      "CSV取込み",
      "棚卸し",
    ]);
    await user.click(screen.getByRole("option", { name: "すべて" }));

    const status = screen.getByLabelText("状態");
    expect(status).toHaveAttribute("data-slot", "select-trigger");
    expect(status.tagName).toBe("BUTTON");
    await user.click(status);
    expect((await screen.findAllByRole("option")).map((option) => option.textContent)).toEqual([
      "すべて",
      "有効",
      "取消済み",
      "進行中",
    ]);
  });

  it("REQ-206 / 65 §65.8.1: 商品・部門filterの母集団差注記を常時表示する", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });

    renderWithClient(<InventoryRecordsPage search={{}} onSearchChange={vi.fn()} />);

    expect(
      await screen.findByText(
        "商品・部門での絞り込みは、CSV取込みでは取込み明細、棚卸しでは差異のあった商品が対象です。",
      ),
    ).toBeInTheDocument();
  });

  it("REQ-206 / SPEC-DISP-B2-1 D-B7 / 65 §65.8.1: 進行中棚卸しは代表商品を-で表示する", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          makeRecord({
            record_type: "stocktake",
            record_id: 51,
            representative_item: "算定前商品",
            item_count: 9,
            status: "in_progress",
            detail_route: "/stocktake/records/51",
          }),
        ],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });

    renderWithClient(<InventoryRecordsPage search={{}} onSearchChange={vi.fn()} />);

    // ⑮ SC6/SC7: 表示列から外したIDは詳細リンクで記録を特定する。
    const link = await screen.findByRole("link", { name: "詳細を見る" });
    expect(link.getAttribute("href")).toContain("/stocktake/records/51");
    const row = link.closest("tr");
    expect(row).not.toBeNull();
    expect(within(row as HTMLElement).getAllByText("-")).toHaveLength(1);
    expect(within(row as HTMLElement).getByText("進行中")).toBeInTheDocument();
    expect(within(row as HTMLElement).queryByText("算定前商品")).not.toBeInTheDocument();
    expect(within(row as HTMLElement).queryByText("9")).not.toBeInTheDocument();
  });

  it("REQ-206 / 65 §65.8.1: 完了棚卸しの差異0件は代表商品に差異なしを表示する", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          makeRecord({
            record_type: "stocktake",
            record_id: 52,
            representative_item: "明細なし",
            item_count: 0,
            status: "active",
            detail_route: "/stocktake/records/52",
          }),
        ],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });

    renderWithClient(<InventoryRecordsPage search={{}} onSearchChange={vi.fn()} />);

    // ⑮ SC6/SC7: 表示列から外したIDは詳細リンクで記録を特定する。
    const link = await screen.findByRole("link", { name: "詳細を見る" });
    expect(link.getAttribute("href")).toContain("/stocktake/records/52");
    const row = link.closest("tr");
    expect(row).not.toBeNull();
    expect(within(row as HTMLElement).getByText("差異なし")).toBeInTheDocument();
    expect(within(row as HTMLElement).queryByText("0")).not.toBeInTheDocument();
    expect(within(row as HTMLElement).queryByText("明細なし")).not.toBeInTheDocument();
  });

  it("REQ-206: formatRecordStatusは4値options経由で進行中を表示する", () => {
    expect(formatRecordStatus("in_progress")).toBe("進行中");
    expect(formatRecordStatus("active")).toBe("有効");
    expect(formatRecordStatus("canceled")).toBe("取消済み");
  });

  it.each([
    {
      recordType: "csv_import",
      recordId: 41,
      detailRoute: "/csv-import/records/41",
      expectedPath: "/csv-import/records/41",
      renderedHeading: "CSV取込み詳細画面",
    },
    {
      recordType: "stocktake",
      recordId: 51,
      detailRoute: "/stocktake/records/51",
      expectedPath: "/stocktake/records/51",
      renderedHeading: "棚卸し詳細画面",
    },
  ])(
    "REQ-207: $recordType 行の詳細ボタンでSPA詳細画面へ遷移する",
    async ({ recordType, recordId, detailRoute, expectedPath, renderedHeading }) => {
      mockListInventoryRecords.mockResolvedValue({
        status: "ok",
        data: {
          items: [
            makeRecord({
              record_type: recordType,
              record_id: recordId,
              detail_route: detailRoute,
            }),
          ],
          total_count: 1,
          page: 1,
          per_page: 50,
        },
      });
      const user = userEvent.setup();
      const { router } = renderWithDetailRoutes();

      await user.click(await screen.findByRole("link", { name: "詳細を見る" }));

      await waitFor(() => {
        expect(router.state.location.pathname).toBe(expectedPath);
      });
      expect(await screen.findByText(renderedHeading)).toBeInTheDocument();
    },
  );

  it("REQ-206: search stateからlistInventoryRecords queryを作り廃棄詳細へリンクする", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: { items: [makeRecord()], total_count: 1, page: 2, per_page: 50 },
    });
    const user = userEvent.setup();
    // S5: returnTo は router の現在地 href（useRouterState）から取るため、直接 props で渡す
    // search state と同じ URL に router を初期化する（DSR-18、本物の送信元 URL を模す）。
    const initialSearchParams = new URLSearchParams({
      recordType: "disposal_record",
      dateFrom: "2026-06-01",
      dateTo: "2026-06-30",
      q: "ボタン",
      recordId: "7",
      departmentId: "2",
      status: "active",
      page: "2",
    });
    const initialPath = `/inventory/records?${initialSearchParams.toString()}`;

    const { router } = renderWithClient(
      <InventoryRecordsPage
        search={{
          recordType: "disposal_record",
          dateFrom: "2026-06-01",
          dateTo: "2026-06-30",
          q: "ボタン",
          recordId: 7,
          departmentId: 2,
          status: "active",
          page: 2,
        }}
        onSearchChange={vi.fn()}
      />,
      initialPath,
    );

    await waitFor(() => {
      expect(mockListInventoryRecords).toHaveBeenCalledWith({
        record_type: "disposal_record",
        date_from: "2026-06-01",
        date_to: "2026-06-30",
        record_id: 7,
        product_keyword: "ボタン",
        department_id: 2,
        status: "active",
        page: 2,
        per_page: 50,
      });
    });
    expect(await screen.findByText("入出庫履歴")).toBeInTheDocument();
    const departmentTrigger = await screen.findByLabelText("部門");
    expect(departmentTrigger).toHaveAttribute("data-slot", "select-trigger");
    expect(departmentTrigger.tagName).toBe("BUTTON");
    expect(departmentTrigger).toHaveTextContent("ボタン");
    expect(screen.getByLabelText("記録ID")).toHaveValue(7);
    const statusTrigger = screen.getByLabelText("状態");
    expect(statusTrigger).toHaveAttribute("data-slot", "select-trigger");
    expect(statusTrigger.tagName).toBe("BUTTON");
    expect(statusTrigger).toHaveTextContent("有効");
    expect(screen.getAllByText("廃棄・破損").length).toBeGreaterThan(0);
    expect(screen.getByText("ボタン #02")).toBeInTheDocument();
    const detailLink = screen.getByRole("link", { name: "詳細を見る" });
    expect(detailLink).toHaveAttribute(
      "href",
      "/inventory/disposal/records/7?returnTo=%2Finventory%2Frecords%3FrecordType%3Ddisposal_record%26dateFrom%3D2026-06-01%26dateTo%3D2026-06-30%26q%3D%25E3%2583%259C%25E3%2582%25BF%25E3%2583%25B3%26recordId%3D7%26departmentId%3D2%26status%3Dactive%26page%3D2",
    );

    // C3/C4: href assertion だけでなく click による実 SPA 遷移を証明する
    // (X3 mutation: static/runtime 代表を生 <a> に戻した場合に red 化する)。
    await user.click(detailLink);
    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/inventory/disposal/records/7");
    });
    expect(router.state.location.search).toEqual({
      returnTo:
        "/inventory/records?recordType=disposal_record&dateFrom=2026-06-01&dateTo=2026-06-30&q=%E3%83%9C%E3%82%BF%E3%83%B3&recordId=7&departmentId=2&status=active&page=2",
    });
  });

  it("T6 REQ-207 / DSR-18: 数字だけの検索語が詳細 link の returnTo で string のまま往復する", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: { items: [makeRecord()], total_count: 1, page: 1, per_page: 50 },
    });
    // router 自身の navigate（SearchBar → onSearchChange）が作る href を模す。
    // defaultStringifySearch({ q: "2099000000019" }) は数字だけの string を JSON-quote
    // して保存するため、URL は q=%222099000000019%22 になる（起票時実測 Probe 2）。
    const initialPath = "/inventory/records?q=%222099000000019%22";

    renderWithClient(
      <InventoryRecordsPage search={{ q: "2099000000019" }} onSearchChange={vi.fn()} />,
      initialPath,
    );

    const detailLink = await screen.findByRole("link", { name: "詳細を見る" });
    const href = detailLink.getAttribute("href") ?? "";
    const outerSearch = defaultParseSearch(new URL(href, "http://inventory.local").search) as {
      returnTo?: string;
    };
    const innerSearch = defaultParseSearch(
      new URL(outerSearch.returnTo ?? "", "http://inventory.local").search,
    );
    expect(inventoryRecordsSearchSchema.parse(innerSearch).q).toBe("2099000000019");
  });

  it("REQ-206: filter変更時はpageを1に戻す", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 3, per_page: 50 },
    });
    const user = userEvent.setup();
    const onSearchChange = vi.fn();

    renderWithClient(
      <InventoryRecordsPage
        search={{ recordType: "disposal_record", page: 3 }}
        onSearchChange={onSearchChange}
      />,
    );

    const recordTypeTrigger = await screen.findByLabelText("記録種別");
    expect(recordTypeTrigger).toHaveAttribute("data-slot", "select-trigger");
    expect(recordTypeTrigger.tagName).toBe("BUTTON");
    await user.click(recordTypeTrigger);
    await user.click(await screen.findByRole("option", { name: "すべて" }));

    const lastCall = onSearchChange.mock.calls[onSearchChange.mock.calls.length - 1] as [
      (prev: { recordType?: string; page?: number }) => { recordType?: string; page?: number },
    ];
    const updater = lastCall[0];
    expect(updater({ recordType: "disposal_record", page: 3 })).toEqual({
      recordType: "all",
      page: 1,
    });
  });

  it("⑧SC4c: 状態selectで「すべて」を選ぶとall sentinelへ戻りstatusフィルタが外れる", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 3, per_page: 50 },
    });
    const user = userEvent.setup();
    const onSearchChange = vi.fn();

    renderWithClient(
      <InventoryRecordsPage
        search={{ status: "active", page: 3 }}
        onSearchChange={onSearchChange}
      />,
    );

    const statusTrigger = await screen.findByLabelText("状態");
    expect(statusTrigger).toHaveAttribute("data-slot", "select-trigger");
    expect(statusTrigger.tagName).toBe("BUTTON");
    await user.click(statusTrigger);
    await user.click(await screen.findByRole("option", { name: "すべて" }));

    const lastStatusCall = onSearchChange.mock.calls[onSearchChange.mock.calls.length - 1] as [
      (prev: { status?: string; page?: number }) => { status?: string; page?: number },
    ];
    expect(lastStatusCall[0]({ status: "active", page: 3 })).toEqual({
      status: "all",
      page: 1,
    });
  });

  it("⑧SC4c-2: 状態selectでall以外を選ぶとsearch stateがその値へ更新される（Codex round3 P2-R2-1、取消済み/進行中の内部値を含む）", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });
    const user = userEvent.setup();
    const onSearchChange = vi.fn();

    renderWithClient(<InventoryRecordsPage search={{}} onSearchChange={onSearchChange} />);

    const statusTrigger = await screen.findByLabelText("状態");

    const STATUS_OPTIONS: [label: string, value: string][] = [
      ["有効", "active"],
      ["取消済み", "canceled"],
      ["進行中", "in_progress"],
    ];
    for (const [label, expectedValue] of STATUS_OPTIONS) {
      await user.click(statusTrigger);
      await user.click(await screen.findByRole("option", { name: label }));
      const lastCall = onSearchChange.mock.calls[onSearchChange.mock.calls.length - 1] as [
        (prev: { status?: string; page?: number }) => { status?: string; page?: number },
      ];
      expect(lastCall[0]({})).toEqual({
        status: expectedValue,
        page: 1,
      });
    }
  });

  it("⑧SC4a: 記録種別selectでall以外を選ぶとsearch stateがその値へ更新される", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });
    const user = userEvent.setup();
    const onSearchChange = vi.fn();

    renderWithClient(<InventoryRecordsPage search={{}} onSearchChange={onSearchChange} />);

    const recordTypeTrigger = await screen.findByLabelText("記録種別");
    await user.click(recordTypeTrigger);
    await user.click(await screen.findByRole("option", { name: "廃棄・破損" }));

    const lastCall = onSearchChange.mock.calls[onSearchChange.mock.calls.length - 1] as [
      (prev: { recordType?: string; page?: number }) => { recordType?: string; page?: number },
    ];
    expect(lastCall[0]({})).toEqual({
      recordType: "disposal_record",
      page: 1,
    });

    await user.click(recordTypeTrigger);
    await user.click(await screen.findByRole("option", { name: "入庫" }));
    const lastCallReceiving = onSearchChange.mock.calls[onSearchChange.mock.calls.length - 1] as [
      (prev: { recordType?: string; page?: number }) => { recordType?: string; page?: number },
    ];
    expect(lastCallReceiving[0]({})).toEqual({
      recordType: "receiving_record",
      page: 1,
    });

    // P2-R2-1: 残る option も内部値を独立リテラルで検査する（round 1 は具体例2件で打ち切っていた）。
    const REMAINING_RECORD_TYPE_OPTIONS: [label: string, value: string][] = [
      ["返品・交換", "return_record"],
      ["手動販売出庫", "manual_sale"],
      ["CSV取込み", "csv_import"],
      ["棚卸し", "stocktake"],
    ];
    for (const [label, expectedValue] of REMAINING_RECORD_TYPE_OPTIONS) {
      await user.click(recordTypeTrigger);
      await user.click(await screen.findByRole("option", { name: label }));
      const lastCallRemaining = onSearchChange.mock.calls[onSearchChange.mock.calls.length - 1] as [
        (prev: { recordType?: string; page?: number }) => { recordType?: string; page?: number },
      ];
      expect(lastCallRemaining[0]({})).toEqual({
        recordType: expectedValue,
        page: 1,
      });
    }
  });

  it("⑧SC4b: 部門selectで実在部門を選ぶとtrigger表示が部門名になりdepartmentIdがnumberになる（round-trip、L8-D5）", async () => {
    function Harness() {
      const [search, setSearch] = useState<InventoryRecordsSearch>({});
      return <InventoryRecordsPage search={search} onSearchChange={setSearch} />;
    }
    const user = userEvent.setup();

    renderWithClient(<Harness />);

    const departmentTrigger = await screen.findByLabelText("部門");
    expect(departmentTrigger).toHaveTextContent("すべて");
    await waitFor(() => {
      expect(departmentTrigger).not.toBeDisabled();
    });
    await user.click(departmentTrigger);
    await user.click(await screen.findByRole("option", { name: "ボタン" }));

    await waitFor(() => {
      expect(departmentTrigger).toHaveTextContent("ボタン");
    });
    await waitFor(() => {
      expect(mockListInventoryRecords).toHaveBeenLastCalledWith(
        expect.objectContaining({ department_id: 2 }),
      );
    });

    await user.click(departmentTrigger);
    await user.click(await screen.findByRole("option", { name: "すべて" }));
    await waitFor(() => {
      expect(departmentTrigger).toHaveTextContent("すべて");
    });
    await waitFor(() => {
      expect(mockListInventoryRecords).toHaveBeenLastCalledWith(
        expect.objectContaining({ department_id: null }),
      );
    });
  });

  it("SC4b: 表示件数を200へ変更するとpage 1・per_page 200で再取得する", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: { items: [makeRecord()], total_count: 1, page: 1, per_page: 200 },
    });

    function Harness() {
      const [search, setSearch] = useState<InventoryRecordsSearch>({ page: 3 });
      return <InventoryRecordsPage search={search} onSearchChange={setSearch} />;
    }

    renderWithClient(<Harness />);
    await screen.findByRole("link", { name: "詳細を見る" });
    const user = userEvent.setup();
    await user.click(screen.getByRole("combobox", { name: "表示件数" }));
    await user.click(await screen.findByRole("option", { name: "200 件" }));

    await waitFor(() => {
      expect(mockListInventoryRecords).toHaveBeenLastCalledWith(
        expect.objectContaining({ page: 1, per_page: 200 }),
      );
    });
  });

  it("REQ-206 / TRACE-D12: 商品検索を200ms後に反映しpageを1へ戻す", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 2, per_page: 50 },
    });
    const onSearchChange = vi.fn();

    renderWithClient(
      <InventoryRecordsPage
        search={{ recordType: "disposal_record", page: 2 }}
        onSearchChange={onSearchChange}
      />,
    );

    const keywordInput = await screen.findByLabelText("商品を検索");
    vi.useFakeTimers();
    fireEvent.change(keywordInput, { target: { value: "ボタン" } });

    expect(onSearchChange).not.toHaveBeenCalled();
    await act(async () => {
      await vi.advanceTimersByTimeAsync(199);
    });
    expect(onSearchChange).not.toHaveBeenCalled();
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1);
    });

    expect(onSearchChange).toHaveBeenCalledTimes(1);
    const updater = onSearchChange.mock.calls[0]?.[0] as (prev: {
      recordType?: string;
      q?: string;
      page?: number;
    }) => { recordType?: string; q?: string; page?: number };
    expect(updater({ recordType: "disposal_record", page: 2 })).toEqual({
      recordType: "disposal_record",
      q: "ボタン",
      page: 1,
    });
  });

  it("REQ-206 / TRACE-D12: Enterはdebounceを待たず商品検索を即時反映する", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 2, per_page: 50 },
    });
    const onSearchChange = vi.fn();

    renderWithClient(
      <InventoryRecordsPage
        search={{ recordType: "disposal_record", page: 2 }}
        onSearchChange={onSearchChange}
      />,
    );

    const keywordInput = await screen.findByLabelText("商品を検索");
    fireEvent.change(keywordInput, { target: { value: "ボタン" } });
    expect(onSearchChange).not.toHaveBeenCalled();

    fireEvent.keyDown(keywordInput, { key: "Enter" });

    expect(onSearchChange).toHaveBeenCalledTimes(1);
    const updater = onSearchChange.mock.calls[0]?.[0] as (prev: {
      recordType?: string;
      q?: string;
      page?: number;
    }) => { recordType?: string; q?: string; page?: number };
    expect(updater({ recordType: "disposal_record", page: 2 })).toEqual({
      recordType: "disposal_record",
      q: "ボタン",
      page: 1,
    });
  });

  it("REQ-206 / TRACE-D12: 商品検索のクリアでqを外しpageを1へ戻す", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 3, per_page: 50 },
    });
    const onSearchChange = vi.fn();

    renderWithClient(
      <InventoryRecordsPage search={{ q: "ボタン", page: 3 }} onSearchChange={onSearchChange} />,
    );

    const keywordInput = await screen.findByLabelText("商品を検索");
    vi.useFakeTimers();
    fireEvent.change(keywordInput, { target: { value: "" } });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(200);
    });

    const updater = onSearchChange.mock.calls[0]?.[0] as (
      prev: InventoryRecordsSearch,
    ) => InventoryRecordsSearch;
    expect(updater({ q: "ボタン", page: 3 })).toEqual({ q: undefined, page: 1 });
  });

  it("REQ-206 / TRACE-D12: IME変換確定Enterはflushせず確定後の最終文字列を反映する", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 2, per_page: 50 },
    });
    const onSearchChange = vi.fn();

    renderWithClient(
      <InventoryRecordsPage
        search={{ recordType: "disposal_record", page: 2 }}
        onSearchChange={onSearchChange}
      />,
    );

    const keywordInput = await screen.findByLabelText("商品を検索");
    vi.useFakeTimers();
    fireEvent.compositionStart(keywordInput);
    fireEvent.change(keywordInput, { target: { value: "ボタ" } });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(200);
    });

    expect(onSearchChange).toHaveBeenCalledTimes(1);
    const intermediateUpdater = onSearchChange.mock.calls[0]?.[0] as (prev: {
      recordType?: string;
      q?: string;
      page?: number;
    }) => { recordType?: string; q?: string; page?: number };
    expect(intermediateUpdater({ recordType: "disposal_record", page: 2 }).q).toBe("ボタ");

    fireEvent.change(keywordInput, { target: { value: "ボタン" } });
    const composingEnter = new KeyboardEvent("keydown", {
      key: "Enter",
      bubbles: true,
      cancelable: true,
    });
    Object.defineProperty(composingEnter, "isComposing", { value: true, configurable: true });
    keywordInput.dispatchEvent(composingEnter);

    expect(onSearchChange).toHaveBeenCalledTimes(1);
    fireEvent.compositionEnd(keywordInput);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(199);
    });
    expect(onSearchChange).toHaveBeenCalledTimes(1);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1);
    });

    expect(onSearchChange).toHaveBeenCalledTimes(2);
    const finalUpdater = onSearchChange.mock.calls[1]?.[0] as (prev: {
      recordType?: string;
      q?: string;
      page?: number;
    }) => { recordType?: string; q?: string; page?: number };
    expect(finalUpdater({ recordType: "disposal_record", page: 2 })).toEqual({
      recordType: "disposal_record",
      q: "ボタン",
      page: 1,
    });
  });

  it("REQ-206 / TRACE-D12: raw qの前後空白を入力表示に保持しqueryだけtrimする", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 2, per_page: 50 },
    });

    function Harness() {
      const [search, setSearch] = useState<InventoryRecordsSearch>({ page: 2 });
      return <InventoryRecordsPage search={search} onSearchChange={setSearch} />;
    }

    renderWithClient(<Harness />);
    const keywordInput = await screen.findByLabelText("商品を検索");
    fireEvent.change(keywordInput, { target: { value: "  ボタン  " } });
    fireEvent.keyDown(keywordInput, { key: "Enter" });

    await waitFor(() => {
      expect(keywordInput).toHaveValue("  ボタン  ");
    });
    await waitFor(() => {
      expect(mockListInventoryRecords.mock.calls.map(([query]) => query.product_keyword)).toContain(
        "ボタン",
      );
    });
    expect(
      mockListInventoryRecords.mock.calls.map(([query]) => query.product_keyword),
    ).not.toContain("  ボタン  ");
  });

  it("REQ-206 / SPEC-UICB-6: live型既定の可視Label「商品を検索」・placeholderを使い、検索ボタンとaria-labelは持たない", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });

    renderWithClient(<InventoryRecordsPage search={{}} onSearchChange={vi.fn()} />);

    const keywordInput = await screen.findByLabelText("商品を検索");
    expect(keywordInput).toHaveAttribute("type", "search");
    expect(keywordInput).toHaveAttribute("placeholder", "商品コード・商品名・JANで検索");
    expect(screen.getByText("商品を検索", { selector: "label" })).toBeInTheDocument();
    expect(keywordInput).toHaveAttribute("id");
    expect(keywordInput).not.toHaveAttribute("aria-label");
    expect(screen.queryByRole("button", { name: "検索" })).not.toBeInTheDocument();
  });
});

describe("InventoryRecordsPage Lane 4 S1c/S3c: frame color + top summary", () => {
  it("SC4c: filter section root has rounded-lg border bg-card p-4", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });
    const { container } = renderWithClient(
      <InventoryRecordsPage search={{}} onSearchChange={vi.fn()} />,
    );
    await screen.findByLabelText("記録種別");
    const box = container.querySelector(".rounded-lg.border.bg-card.p-4");
    expect(box).not.toBeNull();
    // 旧 frame class（`space-y-3 rounded-md border p-4`、bg-card なし）が残っていないこと。
    expect(container.querySelector(".space-y-3.rounded-md.border.p-4")).toBeNull();
  });

  it("SC3c: renders the top PaginationSummary above the table when total_count > 0", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          makeRecord({
            record_type: "stocktake",
            record_id: 51,
            representative_item: "算定前商品",
            item_count: 9,
            status: "in_progress",
            detail_route: "/stocktake/records/51",
          }),
        ],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });
    renderWithClient(<InventoryRecordsPage search={{}} onSearchChange={vi.fn()} />);
    expect(
      await screen.findByText("全 1 件のうち 1〜1 件を表示（1 / 1 ページ）"),
    ).toBeInTheDocument();
  });
});

describe("InventoryRecordsPage SPEC-UIBB-1/2（filter-empty reset action、65 §65.8.1）", () => {
  it("SPEC-UIBB-1 絞り込み該当なしで解除ボタンを表示する", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });
    renderWithClient(<InventoryRecordsPage search={{ q: "該当なし" }} onSearchChange={vi.fn()} />);
    expect(await screen.findByRole("button", { name: "絞り込みを解除" })).toBeInTheDocument();
  });

  it("SPEC-UIBB-1 既定条件の0件では解除ボタンを出さない", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });
    renderWithClient(<InventoryRecordsPage search={{}} onSearchChange={vi.fn()} />);
    expect(await screen.findByText("入出庫履歴がありません")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "絞り込みを解除" })).not.toBeInTheDocument();
  });

  it("SPEC-UIBB-2 解除で全検索条件とpageが既定値に戻る", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 3, per_page: 50 },
    });
    const onSearchChange = vi.fn();
    renderWithClient(
      <InventoryRecordsPage
        search={{
          recordType: "disposal_record",
          dateFrom: "2026-07-01",
          dateTo: "2026-07-31",
          q: "ボタン",
          recordId: 7,
          departmentId: 2,
          status: "active",
          page: 3,
        }}
        onSearchChange={onSearchChange}
      />,
    );
    const resetButton = await screen.findByRole("button", { name: "絞り込みを解除" });
    await userEvent.setup().click(resetButton);

    const updater = onSearchChange.mock.calls[onSearchChange.mock.calls.length - 1]?.[0] as (
      prev: Record<string, unknown>,
    ) => Record<string, unknown>;
    const result = updater({
      recordType: "disposal_record",
      dateFrom: "2026-07-01",
      dateTo: "2026-07-31",
      q: "ボタン",
      recordId: 7,
      departmentId: 2,
      status: "active",
      page: 3,
    });
    expect(result).toEqual({
      recordType: undefined,
      dateFrom: undefined,
      dateTo: undefined,
      q: undefined,
      recordId: undefined,
      departmentId: undefined,
      status: undefined,
      page: undefined,
    });
  });
});

describe("InventoryRecordsPage perPage scroll（REQ-206 / TRACE-D1）", () => {
  it("SC9a: 表示件数変更で画面を先頭へ戻す", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: { items: [makeRecord()], total_count: 120, page: 2, per_page: 50 },
    });
    renderWithClient(<InventoryRecordsPage search={{ page: 2 }} onSearchChange={vi.fn()} />);
    await screen.findByRole("link", { name: "詳細を見る" });

    const user = userEvent.setup();
    await user.click(screen.getByRole("combobox", { name: "表示件数" }));
    await user.click(screen.getByRole("option", { name: "100 件" }));

    expect(mockScrollPageToTop).toHaveBeenCalledTimes(1);
  });
});

describe("InventoryRecordsPage native input tokens（Lane 5 SC4a）", () => {
  it("SC4a: 記録種別/開始日/終了日/記録ID/部門/状態の6箇所すべてがbg-control-surfaceでbg-backgroundを持たない", async () => {
    mockListInventoryRecords.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 50 },
    });

    renderWithClient(<InventoryRecordsPage search={{}} onSearchChange={vi.fn()} />);

    const fields = [
      await screen.findByLabelText("記録種別"),
      screen.getByLabelText("開始日"),
      screen.getByLabelText("終了日"),
      screen.getByLabelText("記録ID"),
      screen.getByLabelText("部門"),
      screen.getByLabelText("状態"),
    ];

    for (const field of fields) {
      expect(field).toHaveClass("bg-control-surface");
      expect(field).not.toHaveClass("bg-background");
    }
  });
});

it("⑮ SC6 / SPEC-DISP-B2-1 D-B7: 一覧の見出しとデータ行は記録ID・明細数を除く6列", async () => {
  mockListInventoryRecords.mockResolvedValue({
    status: "ok",
    data: { items: [makeRecord()], total_count: 1, page: 1, per_page: 50 },
  });
  renderWithClient(<InventoryRecordsPage search={{}} onSearchChange={vi.fn()} />);
  await screen.findByText("ボタン #02");
  expect(screen.getAllByRole("columnheader").map((cell) => cell.textContent)).toEqual([
    "種別",
    "業務日付",
    "代表商品",
    "状態",
    "記録日時",
    "操作",
  ]);
  const row = screen.getByRole("link", { name: "詳細を見る" }).closest("tr");
  if (row === null) throw new Error("expected table structure");
  expect(within(row).getAllByRole("cell")).toHaveLength(6);
  expect(within(row).queryByText("#7")).not.toBeInTheDocument();
});
