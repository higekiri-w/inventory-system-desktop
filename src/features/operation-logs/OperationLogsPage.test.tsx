import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { startTransition, Suspense, use, useState } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import dateTimeSource from "./OperationLogsPage.tsx?raw";

import { commands } from "@/lib/bindings";
import { scrollPageToTop } from "@/lib/page-scroll";
import { renderWithRouter } from "@/test/render-with-router";
import { OperationLogsPage } from "./OperationLogsPage";
import * as operationLogTypes from "./types";
import { normalizeOperationLogsSearch, type OperationLogsSearch } from "./types";

vi.mock("@/lib/bindings", () => ({
  commands: { listLogs: vi.fn(), listLogOperationTypes: vi.fn() },
}));
vi.mock("@/lib/page-scroll", () => ({ scrollPageToTop: vi.fn() }));
const listLogs = vi.mocked(commands.listLogs);
const listTypes = vi.mocked(commands.listLogOperationTypes);
const mockScrollPageToTop = vi.mocked(scrollPageToTop);

type SearchChange = (updater: (previous: OperationLogsSearch) => OperationLogsSearch) => void;

function renderPage(
  search: OperationLogsSearch = {},
  onSearchChange = vi.fn<SearchChange>(),
  initialPath = "/settings/logs",
) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  renderWithRouter(
    <QueryClientProvider client={client}>
      <OperationLogsPage search={search} onSearchChange={onSearchChange} />
    </QueryClientProvider>,
    initialPath,
  );
  return onSearchChange;
}

function renderStatefulPage(initialSearch: OperationLogsSearch = {}) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  function StatefulPage() {
    const [search, setSearch] = useState(initialSearch);
    return (
      <OperationLogsPage
        search={search}
        onSearchChange={(updater) => {
          setSearch(updater);
        }}
      />
    );
  }
  renderWithRouter(
    <QueryClientProvider client={client}>
      <StatefulPage />
    </QueryClientProvider>,
    "/settings/logs",
  );
}
const log = (
  detail_json: string | null = '{"product_code":"SYN-1","unknown_key":"<script>alert(1)</script>"}',
) => ({
  id: 1,
  operation_type: "future_type",
  summary: "合成ログ",
  detail_json,
  created_at: "2026-07-10T12:34:56",
});

beforeEach(() => {
  listLogs.mockReset();
  listTypes.mockReset();
  mockScrollPageToTop.mockReset();
  listTypes.mockResolvedValue({ status: "ok", data: ["backup_create", "future_type"] });
});

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

describe("UI-11c REQ-902", () => {
  it("⑰ SC2/SC6 / UIDISP-D2/D6: 長い JSON 要約の祖先で折り返し、日時は等幅にする", async () => {
    const values = [{ product_code: "SYN-LONG-".repeat(40) }];
    listLogs.mockResolvedValue({
      status: "ok",
      data: {
        items: [log(JSON.stringify({ mismatches: values }))],
        total_count: 1,
        page: 1,
        per_page: 20,
      },
    });
    renderPage();
    await userEvent.setup().click(await screen.findByRole("button", { name: "詳細を表示" }));
    const summary = screen.getByRole("group", { name: "ログ詳細の要約" });
    const value = within(summary).getByText(JSON.stringify(values));
    expect(value.tagName).toBe("DD");
    expect(value).toHaveClass("break-all");
    expect(value).not.toHaveClass("whitespace-pre-wrap");
    expect(value.closest("td")).toHaveAttribute("colspan", "4");
    expect(value.closest("td")).toHaveClass("whitespace-normal");
    expect(screen.getByRole("cell", { name: "2026-07-10 12:34:56" })).toHaveClass(
      "font-mono",
      "tabular-nums",
    );
  });

  it("normalizes the initial search to a JST calendar 30-day default", () => {
    expect(normalizeOperationLogsSearch({}, new Date("2026-07-11T12:00:00+09:00"))).toMatchObject({
      start_date: "2026-06-12",
      end_date: "2026-07-11",
      page: 1,
    });
    expect(
      normalizeOperationLogsSearch(
        { start_date: "2026-01-01", end_date: "2026-02-01", operation_type: "x", page: 3 },
        new Date("2026-07-11"),
      ),
    ).toEqual({ start_date: "2026-01-01", end_date: "2026-02-01", operation_type: "x", page: 3 });
  });

  it("preserves explicit one-sided and fully-cleared date URL states", () => {
    const now = new Date("2026-07-11T12:00:00+09:00");
    expect(normalizeOperationLogsSearch({ start_date: "2026-07-01" }, now)).toEqual({
      start_date: "2026-07-01",
      end_date: undefined,
      page: 1,
    });
    expect(normalizeOperationLogsSearch({ end_date: "2026-07-11" }, now)).toEqual({
      start_date: undefined,
      end_date: "2026-07-11",
      page: 1,
    });
    expect(normalizeOperationLogsSearch({ start_date: "", end_date: "" }, now)).toEqual({
      start_date: undefined,
      end_date: undefined,
      page: 1,
    });
    expect(
      normalizeOperationLogsSearch({ start_date: undefined, end_date: undefined }, now),
    ).toEqual({
      start_date: "2026-06-12",
      end_date: "2026-07-11",
      page: 1,
    });
  });

  it.each([
    [{ start_date: "2026-07-01" }, { start_date: "2026-07-01", end_date: null }],
    [{ end_date: "2026-07-11" }, { start_date: null, end_date: "2026-07-11" }],
  ] as const)(
    "sends one-sided URL bounds to CMD without restoring the missing side",
    async (search, dates) => {
      listLogs.mockResolvedValue({
        status: "ok",
        data: { items: [log()], total_count: 1, page: 1, per_page: 20 },
      });
      renderPage(search);
      await waitFor(() => {
        expect(listLogs).toHaveBeenCalledWith({
          ...dates,
          operation_type: null,
          page: 1,
          per_page: 50,
        });
      });
    },
  );

  it.each([
    [
      "開始日",
      "log-start",
      "2026-07-02",
      { start_date: "2026-07-02", end_date: "2026-07-11", operation_type: "backup_create" },
    ],
    [
      "終了日",
      "log-end",
      "2026-07-12",
      { start_date: "2026-07-01", end_date: "2026-07-12", operation_type: "backup_create" },
    ],
  ] as const)(
    "resets page and preserves other filters when %s changes",
    async (_label, id, value, expected) => {
      listLogs.mockResolvedValue({
        status: "ok",
        data: { items: [log()], total_count: 1, page: 3, per_page: 20 },
      });
      renderStatefulPage({
        start_date: "2026-07-01",
        end_date: "2026-07-11",
        operation_type: "backup_create",
        page: 3,
      });
      await waitFor(() => {
        expect(listLogs).toHaveBeenCalledWith({
          start_date: "2026-07-01",
          end_date: "2026-07-11",
          operation_type: "backup_create",
          page: 3,
          per_page: 50,
        });
      });
      fireEvent.change(document.getElementById(id) as HTMLInputElement, { target: { value } });
      await waitFor(() => {
        expect(listLogs).toHaveBeenLastCalledWith({ ...expected, page: 1, per_page: 50 });
      });
    },
  );

  it("resets page and preserves date filters when operation type changes", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [log()], total_count: 1, page: 3, per_page: 20 },
    });
    renderStatefulPage({ start_date: "2026-07-01", end_date: "2026-07-11", page: 3 });
    const user = userEvent.setup();
    const typeTrigger = await screen.findByLabelText("種別");
    await user.click(typeTrigger);
    await user.click(await screen.findByRole("option", { name: "その他（future_type）" }));
    await waitFor(() => {
      expect(listLogs).toHaveBeenLastCalledWith({
        start_date: "2026-07-01",
        end_date: "2026-07-11",
        operation_type: "future_type",
        page: 1,
        per_page: 50,
      });
    });

    await user.click(typeTrigger);
    await user.click(await screen.findByRole("option", { name: "すべて" }));
    await waitFor(() => {
      expect(listLogs).toHaveBeenLastCalledWith({
        start_date: "2026-07-01",
        end_date: "2026-07-11",
        operation_type: null,
        page: 1,
        per_page: 50,
      });
    });
  });

  it("SC4d: 表示件数を200へ変更するとpage 1・per_page 200で再取得する", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [log()], total_count: 1, page: 1, per_page: 200 },
    });
    renderStatefulPage({ start_date: "2026-07-01", end_date: "2026-07-11", page: 3 });
    await screen.findByText("合成ログ");

    const user = userEvent.setup();
    await user.click(screen.getByRole("combobox", { name: "表示件数" }));
    await user.click(await screen.findByRole("option", { name: "200 件" }));

    await waitFor(() => {
      expect(listLogs).toHaveBeenLastCalledWith(
        expect.objectContaining({ page: 1, per_page: 200 }),
      );
    });
  });

  it("keeps one-sided bounds when a date input is cleared", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [log()], total_count: 1, page: 3, per_page: 20 },
    });
    renderStatefulPage({ start_date: "2026-07-01", end_date: "2026-07-11", page: 3 });
    await screen.findByText("合成ログ");
    fireEvent.change(screen.getByLabelText("開始日"), { target: { value: "" } });
    await waitFor(() => {
      expect(listLogs).toHaveBeenLastCalledWith({
        start_date: null,
        end_date: "2026-07-11",
        operation_type: null,
        page: 1,
        per_page: 50,
      });
    });
  });

  it("keeps the start-only bound when the end date is cleared", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [log()], total_count: 1, page: 3, per_page: 20 },
    });
    renderStatefulPage({ start_date: "2026-07-01", end_date: "2026-07-11", page: 3 });
    await screen.findByText("合成ログ");
    fireEvent.change(screen.getByLabelText("終了日"), { target: { value: "" } });
    await waitFor(() => {
      expect(listLogs).toHaveBeenLastCalledWith({
        start_date: "2026-07-01",
        end_date: null,
        operation_type: null,
        page: 1,
        per_page: 50,
      });
    });
  });

  it("keeps the last valid list and expanded row while an inverted range is corrected", async () => {
    listLogs
      .mockResolvedValueOnce({
        status: "ok",
        data: { items: [log()], total_count: 45, page: 3, per_page: 20 },
      })
      .mockResolvedValueOnce({
        status: "ok",
        data: {
          items: [{ ...log(), summary: "修正後の合成ログ" }],
          total_count: 1,
          page: 1,
          per_page: 20,
        },
      });
    renderStatefulPage({ start_date: "2026-07-01", end_date: "2026-07-10", page: 3 });
    await userEvent.setup().click(await screen.findByRole("button", { name: "詳細を表示" }));
    expect(screen.getByText("商品コード")).toBeInTheDocument();
    expect(screen.getAllByText("全 45 件のうち 41〜45 件を表示（3 / 3 ページ）")).toHaveLength(2);
    expect(screen.getByRole("button", { name: "前のページ" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "次のページ" })).toBeDisabled();
    expect(listLogs).toHaveBeenCalledTimes(1);

    fireEvent.change(screen.getByLabelText("開始日"), { target: { value: "2026-07-11" } });
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "開始日は終了日と同じ日か、それより前の日付にしてください",
    );
    expect(listLogs).toHaveBeenCalledTimes(1);
    expect(screen.getAllByText("合成ログ")).toHaveLength(2);
    expect(screen.getByText("商品コード")).toBeInTheDocument();
    expect(screen.getAllByText("全 45 件のうち 41〜45 件を表示（3 / 3 ページ）")).toHaveLength(2);
    expect(screen.getByRole("button", { name: "前のページ" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "次のページ" })).toBeDisabled();

    fireEvent.change(screen.getByLabelText("開始日"), { target: { value: "2026-07-02" } });
    expect(await screen.findByText("修正後の合成ログ")).toBeInTheDocument();
    expect(listLogs).toHaveBeenCalledTimes(2);
    expect(listLogs).toHaveBeenLastCalledWith({
      start_date: "2026-07-02",
      end_date: "2026-07-10",
      operation_type: null,
      page: 1,
      per_page: 50,
    });
  });

  it("REQ-902 keeps the last committed valid search after a newer valid search commits", async () => {
    listLogs.mockImplementation((query) =>
      Promise.resolve({
        status: "ok",
        data: {
          items: [
            {
              ...log(),
              summary:
                query.start_date === "2026-07-02"
                  ? "最新のcommit済み合成ログ"
                  : "初回commit済み合成ログ",
            },
          ],
          total_count: 1,
          page: 1,
          per_page: 20,
        },
      }),
    );
    renderStatefulPage({ start_date: "2026-07-01", end_date: "2026-07-10", page: 1 });
    expect(await screen.findByText("初回commit済み合成ログ")).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("開始日"), { target: { value: "2026-07-02" } });
    expect(await screen.findByText("最新のcommit済み合成ログ")).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("開始日"), { target: { value: "2026-07-11" } });
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "開始日は終了日と同じ日か、それより前の日付にしてください",
    );
    expect(screen.getByText("最新のcommit済み合成ログ")).toBeInTheDocument();
    expect(screen.queryByText("初回commit済み合成ログ")).not.toBeInTheDocument();
    expect(listLogs).toHaveBeenCalledTimes(2);
  });

  it("REQ-902 does not expose search from a discarded valid transition render", async () => {
    const renderedStartDates: (string | undefined)[] = [];
    const neverResolves = new Promise<void>(() => undefined);
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });

    function RenderProbe({ currentSearch }: { currentSearch: OperationLogsSearch }) {
      renderedStartDates.push(currentSearch.start_date);
      return null;
    }

    function SuspendAfterPage({ active }: { active: boolean }) {
      if (active) use(neverResolves);
      return null;
    }

    function ConcurrentPage() {
      const [search, setSearch] = useState<OperationLogsSearch>({
        start_date: "2026-07-01",
        end_date: "2026-07-10",
        page: 3,
      });
      const [shouldSuspend, setShouldSuspend] = useState(false);
      return (
        <>
          <OperationLogsPage
            search={search}
            onSearchChange={(updater) => {
              setSearch(updater);
            }}
          />
          <RenderProbe currentSearch={search} />
          <button
            type="button"
            onClick={() => {
              startTransition(() => {
                setSearch({
                  start_date: "2026-07-02",
                  end_date: "2026-07-10",
                  page: 1,
                });
                setShouldSuspend(true);
              });
            }}
          >
            valid transitionを開始
          </button>
          <button
            type="button"
            onClick={() => {
              setShouldSuspend(false);
              setSearch({
                start_date: "2026-07-11",
                end_date: "2026-07-10",
                page: 1,
              });
            }}
          >
            invalid searchをcommit
          </button>
          <SuspendAfterPage active={shouldSuspend} />
        </>
      );
    }

    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [log()], total_count: 45, page: 3, per_page: 20 },
    });
    renderWithRouter(
      <QueryClientProvider client={client}>
        <Suspense>
          <ConcurrentPage />
        </Suspense>
      </QueryClientProvider>,
    );
    expect(await screen.findByText("合成ログ")).toBeInTheDocument();
    expect(screen.getAllByText("全 45 件のうち 41〜45 件を表示（3 / 3 ページ）")).toHaveLength(2);

    fireEvent.click(screen.getByRole("button", { name: "valid transitionを開始" }));
    await waitFor(() => {
      expect(renderedStartDates).toContain("2026-07-02");
    });
    fireEvent.click(screen.getByRole("button", { name: "invalid searchをcommit" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "開始日は終了日と同じ日か、それより前の日付にしてください",
    );
    expect(screen.getByText("合成ログ")).toBeInTheDocument();
    expect(screen.getAllByText("全 45 件のうち 41〜45 件を表示（3 / 3 ページ）")).toHaveLength(2);
    expect(listLogs).toHaveBeenCalledTimes(1);
  });

  it("gets type options independently and shows unknown raw fallback", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [log()], total_count: 1, page: 1, per_page: 20 },
    });
    renderPage();
    const user = userEvent.setup();
    // badge 表示の1件（select を開く前でも見える）。
    expect((await screen.findAllByText("その他（future_type）")).length).toBeGreaterThanOrEqual(1);
    // trigger を開いた状態で option としても同じ raw fallback 文言が存在すること。
    await user.click(screen.getByLabelText("種別"));
    expect(
      await screen.findByRole("option", { name: "その他（future_type）" }),
    ).toBeInTheDocument();
    expect(listTypes).toHaveBeenCalledTimes(1);
  });

  it("conveys known and unknown operation types with visible badge text, not color alone", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          { ...log(), id: 1, operation_type: "backup_create" },
          { ...log(), id: 2, operation_type: "synthetic_unregistered_type" },
        ],
        total_count: 2,
        page: 1,
        per_page: 20,
      },
    });
    renderPage();

    const knownBadge = (await screen.findAllByText("バックアップ作成")).find(
      (element) => element.dataset.slot === "badge",
    );
    expect(knownBadge).toBeVisible();
    expect(screen.getByText("その他（synthetic_unregistered_type）")).toBeVisible();
  });

  // UI-11c: Gated Amendment 2 A2-c — category and option order are independent literals.
  it("orders operation type groups and options by the canonical registry, with unknown values last", async () => {
    listTypes.mockResolvedValue({
      status: "ok",
      data: [
        "backup_create",
        "supplier_merge",
        "future_type",
        "csv_import",
        "receiving_create",
        "product_update",
        "supplier_rename",
        "product_create",
      ],
    });
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [log()], total_count: 1, page: 1, per_page: 20 },
    });
    const onSearchChange = renderPage();
    const user = userEvent.setup();
    await user.click(await screen.findByLabelText("種別"));

    const groups = await screen.findAllByRole("group");
    expect(
      groups.map((group) => group.querySelector('[data-slot="select-label"]')?.textContent),
    ).toEqual(["商品管理", "取引先管理", "入出庫", "売上データ取込み", "システム管理", "その他"]);

    expect(screen.getAllByRole("option").map((option) => option.textContent)).toEqual([
      "すべて",
      "商品登録",
      "商品修正",
      "取引先の改名",
      "取引先の統合",
      "入庫記録",
      "売上データ取込み",
      "バックアップ作成",
      "その他（future_type）",
    ]);

    // P2-5: option の group 所属・順序を group ごとに独立して検査する。
    expect(
      within(groups[0])
        .getAllByRole("option")
        .map((option) => option.textContent),
    ).toEqual(["商品登録", "商品修正"]);
    expect(
      within(groups[1])
        .getAllByRole("option")
        .map((option) => option.textContent),
    ).toEqual(["取引先の改名", "取引先の統合"]);
    expect(
      within(groups[2])
        .getAllByRole("option")
        .map((option) => option.textContent),
    ).toEqual(["入庫記録"]);
    expect(
      within(groups[3])
        .getAllByRole("option")
        .map((option) => option.textContent),
    ).toEqual(["売上データ取込み"]);
    expect(
      within(groups[4])
        .getAllByRole("option")
        .map((option) => option.textContent),
    ).toEqual(["バックアップ作成"]);
    expect(
      within(groups[5])
        .getAllByRole("option")
        .map((option) => option.textContent),
    ).toEqual(["その他（future_type）"]);

    // P2-2: 表示文言だけでなく内部値も独立リテラルで検査する。
    await user.click(screen.getByRole("option", { name: "取引先の改名" }));
    const lastCall = onSearchChange.mock.calls[onSearchChange.mock.calls.length - 1] as [
      (prev: OperationLogsSearch) => OperationLogsSearch,
    ];
    expect(lastCall[0]({})).toEqual({ operation_type: "supplier_rename", page: 1 });

    // P2-R2-1: 残る option も内部値を独立リテラルで検査する（round 1 は具体例1件で打ち切っていた）。
    const REMAINING_OPERATION_TYPE_OPTIONS: [label: string, value: string][] = [
      ["商品登録", "product_create"],
      ["商品修正", "product_update"],
      ["取引先の統合", "supplier_merge"],
      ["入庫記録", "receiving_create"],
      ["売上データ取込み", "csv_import"],
      ["バックアップ作成", "backup_create"],
      ["その他（future_type）", "future_type"],
    ];
    for (const [label, expectedValue] of REMAINING_OPERATION_TYPE_OPTIONS) {
      await user.click(await screen.findByLabelText("種別"));
      await user.click(await screen.findByRole("option", { name: label }));
      const lastCallRemaining = onSearchChange.mock.calls[onSearchChange.mock.calls.length - 1] as [
        (prev: OperationLogsSearch) => OperationLogsSearch,
      ];
      expect(lastCallRemaining[0]({})).toEqual({ operation_type: expectedValue, page: 1 });
    }
  });

  it("UI-11c SC9c: 期間・種別のnative入力欄にcontrol tokenを適用する", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [log()], total_count: 1, page: 1, per_page: 50 },
    });
    renderPage();
    await screen.findByText("合成ログ");

    for (const label of ["開始日", "終了日", "種別"]) {
      expect(screen.getByLabelText(label)).toHaveClass("border-input", "bg-control-surface");
    }
  });

  it("does not derive operation type options from the current page", async () => {
    listTypes.mockResolvedValue({ status: "ok", data: ["backup_create"] });
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [log()], total_count: 1, page: 1, per_page: 20 },
    });
    renderPage();
    expect(await screen.findByText("その他（future_type）")).toBeInTheDocument();

    // P2-6: open 状態で正例（registry 由来の option）を先に確認してから、
    // ページ由来候補（future_type）が混入していないことを同じ open 状態で assert する。
    const user = userEvent.setup();
    await user.click(await screen.findByLabelText("種別"));
    expect(await screen.findByRole("option", { name: "バックアップ作成" })).toBeInTheDocument();
    expect(screen.queryByRole("option", { name: "その他（future_type）" })).not.toBeInTheDocument();
  });

  it("keeps the log list and selected unknown type usable when the type registry fails", async () => {
    listTypes.mockResolvedValue({
      status: "error",
      error: {
        kind: "internal",
        message: "registry unavailable",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [log()], total_count: 1, page: 1, per_page: 20 },
    });
    renderPage({ operation_type: "future_type" });
    expect(await screen.findByText("合成ログ")).toBeInTheDocument();
    await userEvent.setup().click(screen.getByLabelText("種別"));
    expect(
      await screen.findByRole("option", { name: "その他（future_type）" }),
    ).toBeInTheDocument();
  });

  it("expands one row, labels known fields, and renders hostile JSON as text", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [log()], total_count: 1, page: 1, per_page: 20 },
    });
    renderPage();
    const trigger = await screen.findByRole("button", { name: "詳細を表示" });
    await userEvent.setup().click(trigger);
    expect(await screen.findByText("商品コード")).toBeInTheDocument();
    expect(screen.getByText("<script>alert(1)</script>")).toBeInTheDocument();
    expect(document.querySelector("script")).toBeNull();
    expect(screen.getByText("技術情報（JSON）")).toBeInTheDocument();
  });

  it("UI-11c-D14 / REQ-902: stock_unit の key がある補正は長さの商品を m で出し、key の無い記録済みの log は整数のまま", async () => {
    const detail = {
      fixed_count: 4,
      skipped_count: 0,
      adjustments: [
        {
          product_code: "SYN-M",
          stock_unit: "m",
          old_stock: 2750,
          new_stock: 2700,
          adjustment: -50,
        },
        {
          product_code: "SYN-CM",
          stock_unit: "m",
          old_stock: 2700,
          new_stock: 2750,
          adjustment: 50,
        },
        { product_code: "SYN-OLD", old_stock: 2750, new_stock: 2700, adjustment: -50 },
        {
          product_code: "SYN-PCS",
          stock_unit: "pcs",
          old_stock: 2750,
          new_stock: 2700,
          adjustment: -50,
        },
        {
          product_code: "SYN-BAD",
          stock_unit: "kg",
          old_stock: 2750,
          new_stock: 2700,
          adjustment: -50,
        },
      ],
    };
    listLogs.mockResolvedValue({
      status: "ok",
      data: {
        items: [{ ...log(JSON.stringify(detail)), operation_type: "integrity_fix" }],
        total_count: 1,
        page: 1,
        per_page: 20,
      },
    });
    renderPage();
    await userEvent.setup().click(await screen.findByRole("button", { name: "詳細を表示" }));

    const rows = within(screen.getByRole("region", { name: "整合性補正の内容" })).getAllByRole(
      "listitem",
    );
    expect(rows[0]).toHaveTextContent("旧在庫 27.5 m → 新在庫 27 m");
    expect(rows[0]).toHaveTextContent("差分 -0.5 m");
    expect(rows[1]).toHaveTextContent("旧在庫 27 m → 新在庫 27.5 m");
    expect(rows[1]).toHaveTextContent("差分 +0.5 m");
    for (const row of rows.slice(2)) {
      expect(row).toHaveTextContent("旧在庫 2,750 → 新在庫 2,700");
      expect(row).toHaveTextContent("差分 -50");
    }
  });

  // UI-11c-D14 / REQ-902 / Matrix U6: 記録済みの形（old_stock・new_stock・adjustment）と 36 の before/after の形
  it("integrity_fix の記録済みの形と before/after の形の両方を読む", async () => {
    const stockCopy = (quantity: number, range: [number, number] | null = null) => ({
      stock_quantity: quantity,
      stock_recheck: range !== null,
      stock_range_low: range?.[0] ?? null,
      stock_range_high: range?.[1] ?? null,
    });
    const renderDetail = async (detail: Record<string, unknown>) => {
      listLogs.mockResolvedValue({
        status: "ok",
        data: {
          items: [{ ...log(JSON.stringify(detail)), operation_type: "integrity_fix" }],
          total_count: 1,
          page: 1,
          per_page: 20,
        },
      });
      renderPage();
      await userEvent.setup().click(await screen.findByRole("button", { name: "詳細を表示" }));
    };
    const detail = {
      fixed_count: 3,
      skipped_count: 0,
      adjustments: [
        { product_code: "U6-LEGACY", old_stock: 10, new_stock: 7, adjustment: -3 },
        {
          product_code: "U6-RANGE",
          stock_unit: "pcs",
          before: stockCopy(5, [4, 6]),
          after: stockCopy(5, [3, 7]),
        },
        {
          product_code: "U6-M",
          stock_unit: "m",
          before: stockCopy(2750),
          after: stockCopy(2700, [2650, 2800]),
        },
      ],
      trigger: "migration",
    };
    await renderDetail(detail);

    const rows = within(screen.getByRole("region", { name: "整合性補正の内容" })).getAllByRole(
      "listitem",
    );
    expect(rows).toHaveLength(3);
    expect(rows[0]).toHaveTextContent("U6-LEGACY");
    expect(rows[0]).toHaveTextContent("旧在庫 10 → 新在庫 7");
    expect(rows[0]).toHaveTextContent("差分 -3");
    expect(rows[1]).toHaveTextContent("U6-RANGE");
    expect(rows[1]).toHaveTextContent("5（要再確認 4〜6） → 5（要再確認 3〜7）");
    expect(rows[2]).toHaveTextContent("27.5 m → 27 m（要再確認 26.5 m〜28 m）");
    const summary = screen.getByRole("group", { name: "ログ詳細の要約" });
    expect(within(summary).queryByText(JSON.stringify(detail.adjustments))).toBeNull();
  });

  it.each([
    [
      "recheck without range",
      { stock_quantity: 5, stock_recheck: true, stock_range_low: null, stock_range_high: null },
    ],
    [
      "range without recheck",
      { stock_quantity: 5, stock_recheck: false, stock_range_low: 4, stock_range_high: 6 },
    ],
    [
      "recheck not boolean",
      { stock_quantity: 5, stock_recheck: 1, stock_range_low: null, stock_range_high: null },
    ],
    [
      "unsafe quantity",
      {
        stock_quantity: Number.MAX_SAFE_INTEGER + 1,
        stock_recheck: false,
        stock_range_low: null,
        stock_range_high: null,
      },
    ],
    ["after missing", null],
  ])(
    "UI-11c-D14 / REQ-902 / U6: どちらの形にも合わない要素（%s）は技術情報へ回す",
    async (_name, after) => {
      const adjustments = [
        {
          product_code: "U6-BAD",
          before: {
            stock_quantity: 5,
            stock_recheck: false,
            stock_range_low: null,
            stock_range_high: null,
          },
          ...(after === null ? {} : { after }),
        },
      ];
      listLogs.mockResolvedValue({
        status: "ok",
        data: {
          items: [
            {
              ...log(JSON.stringify({ fixed_count: 1, adjustments })),
              operation_type: "integrity_fix",
            },
          ],
          total_count: 1,
          page: 1,
          per_page: 20,
        },
      });
      renderPage();
      await userEvent.setup().click(await screen.findByRole("button", { name: "詳細を表示" }));

      expect(screen.queryByRole("region", { name: "整合性補正の内容" })).toBeNull();
      const summary = screen.getByRole("group", { name: "ログ詳細の要約" });
      expect(within(summary).getByText(JSON.stringify(adjustments))).toBeVisible();
      expect(screen.getByText("技術情報（JSON）")).toBeInTheDocument();
    },
  );

  it("test_operation_logs_req902_t9_renders_integrity_adjustments_as_scoped_plain_text", async () => {
    const detail = {
      fixed_count: 2,
      skipped_count: 0,
      adjustments: [
        { product_code: "SYN-DOWN", old_stock: 10, new_stock: 7, adjustment: -3 },
        {
          product_code: "<script>synthetic</script>",
          old_stock: 3,
          new_stock: 8,
          adjustment: 5,
        },
      ],
    };
    listLogs.mockResolvedValue({
      status: "ok",
      data: {
        items: [{ ...log(JSON.stringify(detail)), operation_type: "integrity_fix" }],
        total_count: 1,
        page: 1,
        per_page: 20,
      },
    });
    renderPage();
    await userEvent.setup().click(await screen.findByRole("button", { name: "詳細を表示" }));

    const region = screen.getByRole("region", { name: "整合性補正の内容" });
    const rows = within(region).getAllByRole("listitem");
    expect(rows).toHaveLength(2);
    expect(rows[0]).toHaveTextContent("SYN-DOWN");
    expect(rows[0]).toHaveTextContent("旧在庫 10 → 新在庫 7");
    expect(rows[0]).toHaveTextContent("差分 -3");
    expect(rows[1]).toHaveTextContent("<script>synthetic</script>");
    expect(rows[1]).toHaveTextContent("旧在庫 3 → 新在庫 8");
    expect(rows[1]).toHaveTextContent("差分 +5");
    expect(document.querySelector("script")).toBeNull();
    const summary = screen.getByRole("group", { name: "ログ詳細の要約" });
    expect(within(summary).queryByText(JSON.stringify(detail.adjustments))).toBeNull();
  });

  it("test_operation_logs_req902_t10_keeps_raw_integrity_json_in_technical_details", async () => {
    const detail = {
      fixed_count: 1,
      adjustments: [{ product_code: "SYN-RAW", old_stock: 10, new_stock: 7, adjustment: -3 }],
    };
    listLogs.mockResolvedValue({
      status: "ok",
      data: {
        items: [{ ...log(JSON.stringify(detail)), operation_type: "integrity_fix" }],
        total_count: 1,
        page: 1,
        per_page: 20,
      },
    });
    renderPage();
    const user = userEvent.setup();
    await user.click(await screen.findByRole("button", { name: "詳細を表示" }));

    const technical = screen.getByText("技術情報（JSON）").closest("details");
    expect(technical).not.toBeNull();
    const technicalDetails = technical as HTMLElement;
    const summary = within(technicalDetails).getByText("技術情報（JSON）");
    const rawJson = within(technicalDetails).getByText(/"product_code": "SYN-RAW"/);
    expect(technicalDetails).not.toHaveAttribute("hidden");
    expect(summary).toBeVisible();
    expect(technicalDetails).not.toHaveAttribute("open");
    await user.click(summary);
    expect(technicalDetails).toHaveAttribute("open");
    expect(rawJson).toBeVisible();
  });

  it("test_operation_logs_req902_t11_keeps_generic_detail_for_other_operation_types", async () => {
    const adjustments = [
      { product_code: "SYN-GENERIC", old_stock: 3, new_stock: 5, adjustment: 2 },
    ];
    listLogs.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          {
            ...log(JSON.stringify({ product_code: "SYN-GENERIC", count: 3, adjustments })),
            operation_type: "csv_import",
          },
        ],
        total_count: 1,
        page: 1,
        per_page: 20,
      },
    });
    renderPage();
    await userEvent.setup().click(await screen.findByRole("button", { name: "詳細を表示" }));

    expect(screen.queryByRole("region", { name: "整合性補正の内容" })).toBeNull();
    const summary = screen.getByRole("group", { name: "ログ詳細の要約" });
    expect(within(summary).getByText("商品コード")).toBeVisible();
    expect(within(summary).getByText("SYN-GENERIC")).toBeVisible();
    expect(within(summary).getByText("件数")).toBeVisible();
    expect(within(summary).getByText("3")).toBeVisible();
    expect(within(summary).getByText("adjustments")).toBeVisible();
    expect(within(summary).getByText(JSON.stringify(adjustments))).toBeVisible();
  });

  const validAdjustment = {
    product_code: "SYN",
    old_stock: 1,
    new_stock: 2,
    adjustment: 1,
  };
  const fieldDefects = (["product_code", "old_stock", "new_stock", "adjustment"] as const).flatMap(
    (field) => {
      const wrongType = field === "product_code" ? 123 : "1";
      return [
        [
          `${field} missing`,
          {
            adjustments: [
              Object.fromEntries(Object.entries(validAdjustment).filter(([key]) => key !== field)),
            ],
          },
        ],
        [`${field} null`, { adjustments: [{ ...validAdjustment, [field]: null }] }],
        [`${field} wrong type`, { adjustments: [{ ...validAdjustment, [field]: wrongType }] }],
      ] satisfies [string, Record<string, unknown>][];
    },
  );
  const malformedAdjustmentCases: [string, Record<string, unknown>][] = [
    ["adjustments missing", { fixed_count: 1 }],
    ["adjustments empty", { adjustments: [] }],
    ["adjustments not array", { adjustments: "invalid" }],
    [
      "mixed valid and invalid",
      {
        adjustments: [
          { ...validAdjustment, product_code: "SYN-OK" },
          { product_code: "SYN-BAD", old_stock: 1, new_stock: 2 },
        ],
      },
    ],
    [
      "old_stock unsafe integer",
      {
        adjustments: [
          {
            ...validAdjustment,
            old_stock: Number.MAX_SAFE_INTEGER + 1,
          },
        ],
      },
    ],
    [
      "new_stock unsafe integer",
      {
        adjustments: [
          {
            ...validAdjustment,
            new_stock: Number.MAX_SAFE_INTEGER + 1,
          },
        ],
      },
    ],
    [
      "adjustment unsafe integer",
      {
        adjustments: [
          {
            ...validAdjustment,
            adjustment: Number.MAX_SAFE_INTEGER + 1,
          },
        ],
      },
    ],
    ...fieldDefects,
  ];

  it.each(malformedAdjustmentCases)(
    "test_operation_logs_req902_t12_degrades_malformed_adjustments_%s",
    async (_name, detail) => {
      listLogs.mockResolvedValue({
        status: "ok",
        data: {
          items: [{ ...log(JSON.stringify(detail)), operation_type: "integrity_fix" }],
          total_count: 1,
          page: 1,
          per_page: 20,
        },
      });
      renderPage();
      await userEvent.setup().click(await screen.findByRole("button", { name: "詳細を表示" }));

      expect(screen.queryByRole("region", { name: "整合性補正の内容" })).toBeNull();
      expect(screen.getByText("技術情報（JSON）")).toBeInTheDocument();
      const summary = screen.getByRole("group", { name: "ログ詳細の要約" });
      expect(summary).toBeVisible();
      if ("adjustments" in detail) {
        expect(within(summary).getByText("adjustments")).toBeVisible();
        const adjustments = detail.adjustments;
        const renderedAdjustments =
          typeof adjustments === "string" ? adjustments : JSON.stringify(adjustments);
        expect(within(summary).getByText(renderedAdjustments)).toBeVisible();
      }
    },
  );

  it.each([
    [20, null],
    [21, "他 1 件は技術情報（JSON）で確認"],
    [25, "他 5 件は技術情報（JSON）で確認"],
  ])("test_operation_logs_req902_t13_limits_adjustments_to_twenty_%s", async (count, remainder) => {
    const adjustments = Array.from({ length: count }, (_, index) => ({
      product_code: `SYN-${String(index).padStart(2, "0")}`,
      old_stock: index + 10,
      new_stock: index,
      adjustment: -10,
    }));
    listLogs.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          {
            ...log(JSON.stringify({ adjustments })),
            operation_type: "integrity_fix",
          },
        ],
        total_count: 1,
        page: 1,
        per_page: 20,
      },
    });
    renderPage();
    await userEvent.setup().click(await screen.findByRole("button", { name: "詳細を表示" }));

    const region = screen.getByRole("region", { name: "整合性補正の内容" });
    expect(within(region).getAllByRole("listitem")).toHaveLength(20);
    expect(within(region).getByText("SYN-19")).toBeVisible();
    expect(within(region).queryByText("SYN-20")).toBeNull();
    if (remainder) {
      expect(within(region).getByText(remainder)).toBeVisible();
    } else {
      expect(within(region).queryByText(/他 \d+ 件は技術情報/)).toBeNull();
    }
  });

  // SPEC-COLOR-EMPHASIS-RT-1 / D-CE7: 開いた行と詳細の行を進行中のバー + 地で一体に見せる。
  it("D-CE7: the open log row and its detail row carry the ongoing bar and surface; closing removes them", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [log()], total_count: 1, page: 1, per_page: 20 },
    });
    renderPage();
    const user = userEvent.setup();
    const trigger = await screen.findByRole("button", { name: "詳細を表示" });
    const row = trigger.closest("tr");
    if (!row) throw new Error("row not rendered");
    expect(row).toHaveClass("border-l-4", "border-l-transparent");
    expect(row).not.toHaveClass("has-aria-expanded:bg-ongoing-soft");

    await user.click(trigger);
    expect(row).toHaveClass(
      "border-l-4",
      "border-l-ongoing",
      "has-aria-expanded:bg-ongoing-soft",
      "hover:bg-ongoing-soft",
    );
    expect(row).not.toHaveClass("has-aria-expanded:bg-muted/50");
    const detailRow = document.getElementById(trigger.getAttribute("aria-controls") ?? "");
    expect(detailRow).toBe(row.nextElementSibling);
    expect(detailRow).toHaveClass(
      "border-l-4",
      "border-l-ongoing",
      "bg-ongoing-soft",
      "hover:bg-ongoing-soft",
    );

    await user.click(screen.getByRole("button", { name: "詳細を閉じる" }));
    expect(row).toHaveClass("border-l-transparent");
    expect(row).not.toHaveClass("border-l-ongoing");
    expect(row.nextElementSibling).toBeNull();
  });

  it("toggles detail exactly once through native Enter and Space keyboard paths", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [log()], total_count: 1, page: 1, per_page: 20 },
    });
    renderPage();
    const trigger = await screen.findByRole("button", { name: "詳細を表示" });
    expect(trigger).toHaveTextContent("詳細を表示");
    trigger.focus();
    const user = userEvent.setup();
    await user.keyboard("{Enter}");
    expect(trigger).toHaveAttribute("aria-expanded", "true");
    expect(trigger).toHaveAccessibleName("詳細を閉じる");
    expect(trigger).toHaveTextContent("詳細を閉じる");
    expect(screen.getByText("商品コード")).toBeInTheDocument();
    await user.keyboard("{Enter}");
    expect(trigger).toHaveAttribute("aria-expanded", "false");
    expect(trigger).toHaveAccessibleName("詳細を表示");
    expect(trigger).toHaveTextContent("詳細を表示");
    await user.keyboard(" ");
    expect(trigger).toHaveAttribute("aria-expanded", "true");
    expect(trigger).toHaveTextContent("詳細を閉じる");
  });

  it("keeps only one row expanded and does not toggle it when its related-record link is clicked", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          log('{"record_type":"receiving_record","record_id":7}'),
          { ...log('{"record_type":"return_record","record_id":8}'), id: 2 },
        ],
        total_count: 2,
        page: 1,
        per_page: 20,
      },
    });
    renderPage();
    const user = userEvent.setup();
    const triggers = await screen.findAllByRole("button", { name: "詳細を表示" });

    await user.click(triggers[0]);
    expect(triggers[0]).toHaveAttribute("aria-expanded", "true");
    fireEvent.click(screen.getByRole("link", { name: "関連記録を見る" }));
    expect(triggers[0]).toHaveAttribute("aria-expanded", "true");

    await user.click(triggers[1]);
    expect(triggers[0]).toHaveAttribute("aria-expanded", "false");
    expect(triggers[1]).toHaveAttribute("aria-expanded", "true");
    expect(screen.getAllByText("関連記録ID")).toHaveLength(1);
  });

  it("T9 UI-11c-D16: related records carry every current investigation search parameter", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: {
        items: [log('{"record_type":"receiving_record","record_id":7}')],
        total_count: 1,
        page: 4,
        per_page: 20,
      },
    });
    renderPage(
      {
        start_date: "2026-07-01",
        end_date: "2026-07-31",
        operation_type: "backup_create",
        page: 4,
      },
      vi.fn<SearchChange>(),
      "/settings/logs?start_date=2026-07-01&end_date=2026-07-31&operation_type=backup_create&page=4",
    );
    await userEvent.setup().click(await screen.findByRole("button", { name: "詳細を表示" }));

    const link = screen.getByRole("link", { name: "関連記録を見る" });
    const detailUrl = new URL(link.getAttribute("href") ?? "", "https://app.invalid");
    expect(detailUrl.pathname).toBe("/inventory/receiving/records/7");
    const returnTo = detailUrl.searchParams.get("returnTo");
    expect(returnTo).not.toBeNull();
    const returnUrl = new URL(returnTo ?? "", "https://app.invalid");
    expect(returnUrl.pathname).toBe("/settings/logs");
    expect(returnUrl.searchParams.get("start_date")).toBe("2026-07-01");
    expect(returnUrl.searchParams.get("end_date")).toBe("2026-07-31");
    expect(returnUrl.searchParams.get("operation_type")).toBe("backup_create");
    expect(returnUrl.searchParams.get("page")).toBe("4");
  });

  it("handles null and invalid detail JSON safely", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: {
        items: [log(null), { ...log("not-json"), id: 2 }],
        total_count: 2,
        page: 1,
        per_page: 20,
      },
    });
    renderPage();
    const triggers = await screen.findAllByRole("button", { name: "詳細を表示" });
    await userEvent.setup().click(triggers[0]);
    expect(screen.getByText("詳細情報はありません")).toBeInTheDocument();
    expect(screen.queryByText("技術情報（JSON）")).not.toBeInTheDocument();
    await userEvent.setup().click(triggers[1]);
    expect(screen.getByText("詳細情報を解析できませんでした")).toBeInTheDocument();
  });

  it("truncates oversized technical JSON and only links explicit typed records", async () => {
    const fields = Object.fromEntries(
      Array.from({ length: 22 }, (_, index) => [`field_${String(index)}`, "x".repeat(2500)]),
    );
    listLogs.mockResolvedValue({
      status: "ok",
      data: {
        items: [log(JSON.stringify({ ...fields, record_type: "receiving_record", record_id: 7 }))],
        total_count: 1,
        page: 1,
        per_page: 20,
      },
    });
    renderPage();
    await userEvent.setup().click(await screen.findByRole("button", { name: "詳細を表示" }));
    expect(screen.getByText(/他 4 件のフィールド/)).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "関連記録を見る" })).toHaveAttribute(
      "href",
      "/inventory/receiving/records/7?returnTo=%2Fsettings%2Flogs",
    );
    await userEvent.setup().click(screen.getByText("技術情報（JSON）"));
    expect(screen.getByText(/以降は長すぎるため省略しました/)).toBeInTheDocument();
  });

  it.each([
    ["zero", { record_type: "receiving_record", record_id: 0 }],
    ["negative", { record_type: "receiving_record", record_id: -1 }],
    ["fractional", { record_type: "receiving_record", record_id: 1.5 }],
    ["numeric string", { record_type: "receiving_record", record_id: "7" }],
    ["unsafe integer", { record_type: "receiving_record", record_id: Number.MAX_SAFE_INTEGER + 1 }],
    ["unknown record_type", { record_type: "csv_import", record_id: 7 }],
    ["missing record_type", { record_id: 7 }],
    ["missing record_id", { record_type: "receiving_record" }],
  ])("hides the related-record link for %s", async (_caseName, detail) => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [log(JSON.stringify(detail))], total_count: 1, page: 1, per_page: 20 },
    });
    renderPage();
    await userEvent.setup().click(await screen.findByRole("button", { name: "詳細を表示" }));
    expect(screen.queryByRole("link", { name: "関連記録を見る" })).not.toBeInTheDocument();
  });

  it("shows the related-record link for a positive safe integer and typed allowlist value", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: {
        items: [log('{"record_type":"receiving_record","record_id":1}')],
        total_count: 1,
        page: 1,
        per_page: 20,
      },
    });
    renderPage();
    await userEvent.setup().click(await screen.findByRole("button", { name: "詳細を表示" }));
    expect(screen.getByRole("link", { name: "関連記録を見る" })).toHaveAttribute(
      "href",
      "/inventory/receiving/records/1?returnTo=%2Fsettings%2Flogs",
    );
  });

  it("uses the 30-day empty copy for the default search", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 20 },
    });
    const today = new Date();
    const end = `${String(today.getFullYear())}-${String(today.getMonth() + 1).padStart(2, "0")}-${String(today.getDate()).padStart(2, "0")}`;
    const startDate = new Date(today);
    startDate.setDate(startDate.getDate() - 29);
    const start = `${String(startDate.getFullYear())}-${String(startDate.getMonth() + 1).padStart(2, "0")}-${String(startDate.getDate()).padStart(2, "0")}`;
    renderPage({ start_date: start, end_date: end });
    expect(await screen.findByText("この30日間の操作ログはありません")).toBeInTheDocument();
  });

  it("uses one captured today for the backend query and default empty-state decision", async () => {
    const RealDate = Date;
    const fixedNow = new RealDate("2026-07-11T23:59:59.999+09:00");
    const BoundaryDate = function (value?: string | number | Date) {
      if (arguments.length > 0) return new RealDate(value as string);
      return new RealDate(fixedNow);
    } as unknown as DateConstructor;
    Object.setPrototypeOf(BoundaryDate, RealDate);
    vi.stubGlobal("Date", BoundaryDate);
    const normalizeSpy = vi.spyOn(operationLogTypes, "normalizeOperationLogsSearch");
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 20 },
    });

    renderPage();

    await waitFor(() => {
      expect(listLogs).toHaveBeenCalledWith(
        expect.objectContaining({ start_date: "2026-06-12", end_date: "2026-07-11" }),
      );
    });
    expect(await screen.findByText("この30日間の操作ログはありません")).toBeInTheDocument();
    const [queryNormalization, defaultFilterNormalization] = normalizeSpy.mock.calls;
    expect(queryNormalization[1]).toBeInstanceOf(RealDate);
    expect(defaultFilterNormalization[1]).toBe(queryNormalization[1]);
  });

  it("uses the filtered empty copy when an operation type is selected", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 20 },
    });
    renderPage({ operation_type: "backup_create" });
    expect(await screen.findByText("該当する操作ログがありません")).toBeInTheDocument();
  });

  it("distinguishes out-of-range and recovers to page 1", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 21, page: 4, per_page: 20 },
    });
    const change = renderPage({ page: 4 });
    await userEvent.setup().click(await screen.findByRole("button", { name: "先頭ページに戻る" }));
    const updater = change.mock.calls[change.mock.calls.length - 1]?.[0];
    expect(updater({ page: 4 })).toMatchObject({ page: 1 });
  });

  it("requests the largest positive u32 page and presents out-of-range recovery", async () => {
    const maxPage = 0xffff_ffff;
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 1, page: maxPage, per_page: 20 },
    });

    renderPage({ page: maxPage });

    expect(await screen.findByRole("button", { name: "先頭ページに戻る" })).toBeInTheDocument();
    expect(listLogs).toHaveBeenCalledWith(expect.objectContaining({ page: maxPage, per_page: 50 }));
  });

  it("SPEC-UIBB-1 絞り込み該当なしで解除ボタンを表示する", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 20 },
    });
    renderPage({ operation_type: "backup_create" });
    expect(await screen.findByRole("button", { name: "絞り込みを解除" })).toBeInTheDocument();
  });

  it("SPEC-UIBB-1 既定条件の0件では解除ボタンを出さない", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 20 },
    });
    renderPage();
    expect(await screen.findByText("この30日間の操作ログはありません")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "絞り込みを解除" })).not.toBeInTheDocument();
  });

  it("SPEC-UIBB-2 解除で期間・種別・pageが既定値に戻る", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 3, per_page: 20 },
    });
    const change = renderPage({
      start_date: "2026-07-01",
      end_date: "2026-07-11",
      operation_type: "backup_create",
      page: 3,
    });
    const resetButton = await screen.findByRole("button", { name: "絞り込みを解除" });
    await userEvent.setup().click(resetButton);

    const updater = change.mock.calls[change.mock.calls.length - 1]?.[0] as (
      prev: OperationLogsSearch,
    ) => OperationLogsSearch;
    const result = updater({
      start_date: "2026-07-01",
      end_date: "2026-07-11",
      operation_type: "backup_create",
      page: 3,
    });
    expect(result.start_date).toBeUndefined();
    expect(result.end_date).toBeUndefined();
    expect(result.operation_type).toBeUndefined();
    expect(result.page).toBe(1);
  });

  it("shows an error and retries the same query", async () => {
    listLogs
      .mockResolvedValueOnce({
        status: "error",
        error: {
          kind: "internal",
          message: "synthetic failure",
          field: null,
          error_id: null,
          stocktake_recovery: null,
        },
      })
      .mockResolvedValueOnce({
        status: "ok",
        data: { items: [], total_count: 0, page: 1, per_page: 20 },
      });
    renderPage({ operation_type: "future_type" });
    await userEvent.setup().click(await screen.findByRole("button", { name: "再試行" }));
    await waitFor(() => {
      expect(listLogs).toHaveBeenCalledTimes(2);
    });
    expect(listLogs.mock.calls[1]?.[0].operation_type).toBe("future_type");
  });

  // describe-error-adoption packet（2026-08-04）AC2 / B3 是正: query error 時に InvokeError の
  // デバッグ文字列（`[commands: ...]`）が表示に漏れず、describeError 出力が表示されることを
  // assert する（UI-ERR-D2、B3 は retry: 0 のため単発 reject で即 error 状態に落ちる）。
  it("shows describeError output on query error without leaking the InvokeError debug message", async () => {
    listLogs.mockResolvedValue({
      status: "error",
      error: {
        kind: "internal",
        message: "操作ログの取得に失敗しました",
        field: null,
        error_id: "E-20260710-090000-syn3",
        stocktake_recovery: null,
      },
    });
    renderPage();
    expect(
      await screen.findByText(
        "操作ログの取得に失敗しました（エラーID: E-20260710-090000-syn3）。詳細は診断ログに記録されています。",
      ),
    ).toBeInTheDocument();
    expect(screen.queryByText(/\[commands:/)).not.toBeInTheDocument();
  });
});

describe("OperationLogsPage Lane 4 S1e/S1g/S3e: frame color, table wrapper rounded-lg, top summary", () => {
  it("SC4e: filter section root has rounded-lg border bg-card p-4", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 20 },
    });
    renderPage();
    await screen.findByLabelText("種別");
    expect(document.body.querySelector(".rounded-lg.border.bg-card.p-4")).not.toBeNull();
  });

  it("SC4g: table wrapper has overflow-x-auto rounded-lg border, no bg-card, and no rounded-md leftover", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [log()], total_count: 1, page: 1, per_page: 20 },
    });
    renderPage();
    await screen.findByText("合成ログ");
    const wrapper = document.body.querySelector(".overflow-x-auto");
    expect(wrapper).not.toBeNull();
    const tokens = (wrapper?.className ?? "").split(/\s+/);
    expect(tokens).toContain("rounded-lg");
    expect(tokens).toContain("border");
    expect(tokens).not.toContain("rounded-md");
    expect(tokens).not.toContain("bg-card");
  });

  it("SC3e: renders the top PaginationSummary above the table when items are present", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [log()], total_count: 1, page: 1, per_page: 20 },
    });
    renderPage();
    expect(
      await screen.findByText("全 1 件のうち 1〜1 件を表示（1 / 1 ページ）"),
    ).toBeInTheDocument();
  });
});

describe("OperationLogsPage perPage scroll（UI-11c）", () => {
  it("SC9a: 表示件数変更で画面を先頭へ戻す", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [log()], total_count: 120, page: 2, per_page: 50 },
    });
    renderStatefulPage({ start_date: "2026-07-01", end_date: "2026-07-11", page: 2 });
    await screen.findByText("合成ログ");

    const user = userEvent.setup();
    await user.click(screen.getByRole("combobox", { name: "表示件数" }));
    await user.click(screen.getByRole("option", { name: "100 件" }));

    expect(mockScrollPageToTop).toHaveBeenCalledTimes(1);
  });

  it("SC9a control: ページ送りでは画面を先頭へ戻さない", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [log()], total_count: 120, page: 1, per_page: 50 },
    });
    renderPage({ start_date: "2026-07-01", end_date: "2026-07-11" });
    await screen.findByText("合成ログ");

    await userEvent.setup().click(screen.getByRole("button", { name: "次のページ" }));

    expect(mockScrollPageToTop).not.toHaveBeenCalled();
  });
});

it("⑰ SC6 / UIDISP-D6: 共有 formatDateTime を import しローカル定義を持たない", () => {
  expect(dateTimeSource).toMatch(
    /import\s*\{[^}]*\bformatDateTime\b[^}]*\}\s*from\s*"@\/lib\/date-time"/,
  );
  expect(dateTimeSource).not.toMatch(/function\s+(?:formatDateTime|formatCheckedAt)\s*\(/);
  expect(dateTimeSource).not.toContain("formatCheckedAt");
});
