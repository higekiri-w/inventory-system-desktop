// REQ-401 / UI-07-D19（55 §55.12）・UI-07-D14・§55.9・§55.7: 日報の取込みの記録と行の取消。
// 実の route tree で `/csv-import/daily-reports` を開く（route の validateSearch を通る）。入力はすべて合成
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, createRouter, RouterProvider } from "@tanstack/react-router";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { toast } from "sonner";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  commands,
  type CmdError,
  type DailyReportImportRow,
  type DailyReportRollbackResult,
  type PaginatedResult,
} from "@/lib/bindings";
import { Route as DailyReportsRoute } from "@/routes/csv-import.daily-reports";
import { routeTree } from "@/routeTree.gen";
import { d052InvalidationOracle, expectExactInvalidations } from "@/test/invalidation-oracle";
import { dailyReportImportRecordsSearchSchema } from "./types";

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ setTitle: vi.fn().mockResolvedValue(undefined) }),
}));
vi.mock("sonner", () => ({
  toast: { error: vi.fn(), success: vi.fn() },
  // root layout が描く
  Toaster: () => null,
}));
vi.mock("@/lib/bindings", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/bindings")>();
  return {
    ...actual,
    commands: { listDailyReportImports: vi.fn(), rollbackDailyReportImport: vi.fn() },
  };
});

type ListResponse = Awaited<ReturnType<typeof commands.listDailyReportImports>>;
type RollbackResponse = Awaited<ReturnType<typeof commands.rollbackDailyReportImport>>;

const mockList = vi.mocked(commands.listDailyReportImports);
const mockRollback = vi.mocked(commands.rollbackDailyReportImport);

const DESCRIPTION =
  "この取込みだけを取り消します。同じ日の他の取込みは残ります。取消しても在庫数は変わりません。";

function importRow(
  overrides: Partial<DailyReportImportRow> & { id: number },
): DailyReportImportRow {
  return {
    report_date: "2026-03-21",
    source_filenames: [
      `Z001_${String(overrides.id)}.CSV`,
      `Z002_${String(overrides.id)}.CSV`,
      `Z005_${String(overrides.id)}.CSV`,
    ],
    settlement_no: 101,
    gross_amount: 15000,
    net_amount: 14000,
    status: "completed",
    imported_at: "2026-03-21T10:00:00",
    rolled_back_at: null,
    ...overrides,
  };
}

// 同じ対象日の 2 行（精算 101・102）と、精算回数の記録の無い取消済みの行
const ROW_101 = importRow({ id: 12, settlement_no: 101, gross_amount: 15000, net_amount: 14000 });
const ROW_102 = importRow({ id: 11, settlement_no: 102, gross_amount: 9000, net_amount: 8500 });
const ROW_OLD = importRow({
  id: 10,
  report_date: "2026-03-20",
  settlement_no: null,
  status: "rolled_back",
  rolled_back_at: "2026-03-20T12:00:00",
});

function page(
  items: DailyReportImportRow[],
  totalCount = items.length,
  pageNo = 1,
): { status: "ok"; data: PaginatedResult<DailyReportImportRow> } {
  return {
    status: "ok",
    data: { items, total_count: totalCount, page: pageNo, per_page: 20 },
  };
}

const ROLLED_BACK: DailyReportRollbackResult = {
  daily_report_import_id: 11,
  status: "rolled_back",
  rolled_back_at: "2026-03-21T11:00:00",
};

const FAILED: CmdError = {
  kind: "internal",
  message: "合成の取消の失敗",
  field: null,
  error_id: null,
  stocktake_recovery: null,
};

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((r) => {
    resolve = r;
  });
  return { promise, resolve };
}

function renderRoute(path: string) {
  const router = createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: [path] }),
  });
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
  const view = render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return { router, invalidateSpy, ...view };
}

/** 行を ID のセルで探す */
function rowOf(id: number) {
  const table = screen.getByRole("table");
  const row = within(table)
    .getAllByRole("row")
    .find((element) => within(element).queryAllByRole("cell")[2]?.textContent === String(id));
  if (!row) throw new Error(`row not found: ${String(id)}`);
  return row;
}

async function openTable() {
  await screen.findByRole("table");
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("DailyReportImportRecordsPage UI-07-D19 REQ-401", () => {
  it("U1: 列・精算回数・状態の label と icon・取り消すは completed の行だけ・ファイル名は全部", async () => {
    mockList.mockResolvedValue(page([ROW_101, ROW_102, ROW_OLD]));
    renderRoute("/csv-import/daily-reports");
    await openTable();

    expect(screen.getByRole("heading", { name: "日報の取込みの記録" })).toBeInTheDocument();
    expect(
      screen.getByText(
        "誤って取り込んだ日報の取込みを取り消せます。取り消しても在庫数は変わりません。取り込み直すときは、売上データ取込みの日報取込みで読み直してください。",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "売上データ取込みへ戻る" })).toHaveAttribute(
      "href",
      "/csv-import",
    );
    const headers = within(screen.getByRole("table"))
      .getAllByRole("columnheader")
      .map((cell) => cell.textContent);
    expect(headers).toEqual([
      "対象日",
      "精算回数",
      "取込みID",
      "ファイル名",
      "総売上",
      "純売上",
      "取込み日時",
      "状態",
      "操作",
    ]);

    const cells = (id: number) =>
      within(rowOf(id))
        .getAllByRole("cell")
        .map((cell) => cell.textContent);
    expect(cells(12).slice(0, 7)).toEqual([
      "2026-03-21",
      "101",
      "12",
      "Z001_12.CSVZ002_12.CSVZ005_12.CSV",
      "¥15,000",
      "¥14,000",
      "2026-03-21 10:00:00",
    ]);
    expect(cells(11)[1]).toBe("102");
    // null は「記録なし」（空の欄・0 にしない）
    expect(cells(10)[1]).toBe("記録なし");

    for (const id of [12, 11]) {
      const label = within(rowOf(id)).getByText("取込み済み");
      expect(label.querySelector("svg")).not.toBeNull();
      expect(within(rowOf(id)).getByRole("button", { name: "取り消す" })).toBeEnabled();
    }
    const rolledBack = within(rowOf(10)).getByText("取消済み");
    expect(rolledBack.querySelector("svg")).not.toBeNull();
    expect(within(rowOf(10)).queryByRole("button", { name: "取り消す" })).not.toBeInTheDocument();
  });

  it("U2: 行の取り消すは確認を経て、その行の ID だけで 1 回呼ぶ（キャンセルでは呼ばない）", async () => {
    const user = userEvent.setup();
    mockList.mockResolvedValue(page([ROW_101, ROW_102, ROW_OLD]));
    mockRollback.mockResolvedValue({ status: "ok", data: ROLLED_BACK });
    renderRoute("/csv-import/daily-reports");
    await openTable();

    await user.click(within(rowOf(11)).getByRole("button", { name: "取り消す" }));
    let dialog = await screen.findByRole("alertdialog");
    expect(within(dialog).getByText("日報取込みを取り消しますか？")).toBeInTheDocument();
    expect(within(dialog).getByText(DESCRIPTION)).toBeInTheDocument();
    const terms = within(dialog)
      .getAllByRole("term")
      .map((term) => [term.textContent, term.nextElementSibling?.textContent]);
    expect(terms).toEqual([
      ["取込み ID", "11"],
      ["対象日", "2026-03-21"],
      ["ファイル名", "Z001_11.CSVZ002_11.CSVZ005_11.CSV"],
      ["総売上", "¥9,000"],
      ["純売上", "¥8,500"],
    ]);
    await user.click(within(dialog).getByRole("button", { name: "キャンセル" }));
    expect(mockRollback).not.toHaveBeenCalled();

    await user.click(within(rowOf(11)).getByRole("button", { name: "取り消す" }));
    dialog = await screen.findByRole("alertdialog");
    await user.click(within(dialog).getByRole("button", { name: "取り消す" }));
    await waitFor(() => {
      expect(mockRollback).toHaveBeenCalledTimes(1);
    });
    expect(mockRollback).toHaveBeenCalledWith(11);
  });

  it("U3: 成功で Toast（8 秒）・C10 の再取得と同じ集合・再取得の後に行が取消済みで取り消すが消える", async () => {
    const user = userEvent.setup();
    mockList.mockResolvedValueOnce(page([ROW_101, ROW_102, ROW_OLD]));
    mockList.mockResolvedValue(
      page([
        ROW_101,
        { ...ROW_102, status: "rolled_back", rolled_back_at: "2026-03-21T11:00:00" },
        ROW_OLD,
      ]),
    );
    mockRollback.mockResolvedValue({ status: "ok", data: ROLLED_BACK });
    const { invalidateSpy } = renderRoute("/csv-import/daily-reports");
    await openTable();

    await user.click(within(rowOf(11)).getByRole("button", { name: "取り消す" }));
    await user.click(
      within(await screen.findByRole("alertdialog")).getByRole("button", { name: "取り消す" }),
    );

    await waitFor(() => {
      expect(within(rowOf(11)).getByText("取消済み")).toBeInTheDocument();
    });
    expect(within(rowOf(11)).queryByRole("button", { name: "取り消す" })).not.toBeInTheDocument();
    expect(within(rowOf(12)).getByText("取込み済み")).toBeInTheDocument();
    expect(toast.success).toHaveBeenCalledWith("日報取込みを取り消しました", { duration: 8000 });
    expectExactInvalidations(invalidateSpy.mock.calls, d052InvalidationOracle.dailyReportImport());
  });

  it("U4: 失敗は Toast で、行は取込み済みのまま、同じ行からもう一度押すと同じ ID で呼ぶ", async () => {
    const user = userEvent.setup();
    mockList.mockResolvedValue(page([ROW_101, ROW_102]));
    mockRollback.mockResolvedValue({ status: "error", error: FAILED });
    renderRoute("/csv-import/daily-reports");
    await openTable();

    for (let attempt = 1; attempt <= 2; attempt += 1) {
      await user.click(within(rowOf(11)).getByRole("button", { name: "取り消す" }));
      await user.click(
        within(await screen.findByRole("alertdialog")).getByRole("button", { name: "取り消す" }),
      );
      await waitFor(() => {
        expect(toast.error).toHaveBeenCalledTimes(attempt);
      });
      expect(within(rowOf(11)).getByText("取込み済み")).toBeInTheDocument();
      await waitFor(() => {
        expect(within(rowOf(11)).getByRole("button", { name: "取り消す" })).toBeEnabled();
      });
    }
    expect(toast.error).toHaveBeenCalledWith("取り消しに失敗しました。もう一度お試しください");
    expect(mockRollback).toHaveBeenNthCalledWith(1, 11);
    expect(mockRollback).toHaveBeenNthCalledWith(2, 11);
  });

  it("U5: 取消の応答を待つ間は全部の行の取り消すが押せず、押した行が取り消し中…", async () => {
    const user = userEvent.setup();
    mockList.mockResolvedValue(page([ROW_101, ROW_102]));
    const pending = deferred<RollbackResponse>();
    mockRollback.mockReturnValueOnce(pending.promise);
    renderRoute("/csv-import/daily-reports");
    await openTable();

    await user.click(within(rowOf(11)).getByRole("button", { name: "取り消す" }));
    await user.click(
      within(await screen.findByRole("alertdialog")).getByRole("button", { name: "取り消す" }),
    );
    expect(await within(rowOf(11)).findByRole("button", { name: "取り消し中…" })).toBeDisabled();
    expect(within(rowOf(12)).getByRole("button", { name: "取り消す" })).toBeDisabled();

    pending.resolve({ status: "error", error: FAILED });
    await waitFor(() => {
      expect(within(rowOf(12)).getByRole("button", { name: "取り消す" })).toBeEnabled();
    });
  });

  it("U6: 読込み中・空（total_count 0 のときだけ）・失敗は CMD の message と再試行", async () => {
    const user = userEvent.setup();
    const loading = deferred<ListResponse>();
    mockList.mockReturnValueOnce(loading.promise);
    const { unmount } = renderRoute("/csv-import/daily-reports");
    expect(await screen.findByText("日報の取込みの記録を読み込んでいます…")).toBeInTheDocument();
    loading.resolve(page([], 0));
    expect(await screen.findByText("日報の取込みの記録はまだありません。")).toBeInTheDocument();
    unmount();

    mockList.mockResolvedValueOnce({
      status: "error",
      error: {
        kind: "import_error",
        message: "既存日報のファイル情報を読み取れません",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    mockList.mockResolvedValueOnce(page([ROW_101]));
    renderRoute("/csv-import/daily-reports");
    const alert = await screen.findByRole("alert");
    expect(within(alert).getByText("既存日報のファイル情報を読み取れません")).toBeInTheDocument();
    expect(screen.queryByText("日報の取込みの記録はまだありません。")).not.toBeInTheDocument();
    await user.click(within(alert).getByRole("button", { name: "再試行" }));
    await openTable();
    expect(rowOf(12)).toBeInTheDocument();
  });

  it("U7: search page の検証・20 件・ページ送り・開き直し・範囲外のページ", async () => {
    const user = userEvent.setup();
    // UI-STATE-D2: route は feature の schema をそのまま validateSearch に渡す
    expect(DailyReportsRoute.options.validateSearch).toBe(dailyReportImportRecordsSearchSchema);

    for (const path of [
      "/csv-import/daily-reports",
      "/csv-import/daily-reports?page=0",
      "/csv-import/daily-reports?page=abc",
    ]) {
      mockList.mockClear();
      mockList.mockResolvedValue(page([ROW_101], 21));
      const { unmount } = renderRoute(path);
      await openTable();
      expect(mockList).toHaveBeenCalledWith(1, 20, null, null);
      unmount();
    }

    mockList.mockClear();
    mockList.mockImplementation((pageNo) => Promise.resolve(page([ROW_101], 21, pageNo)));
    const first = renderRoute("/csv-import/daily-reports");
    await openTable();
    await user.click(screen.getByRole("button", { name: "次のページ" }));
    await waitFor(() => {
      expect(mockList).toHaveBeenLastCalledWith(2, 20, null, null);
    });
    expect(first.router.state.location.search).toEqual({ page: 2 });
    first.unmount();

    // 同じ search で開き直すと同じページ
    mockList.mockClear();
    const revisit = renderRoute("/csv-import/daily-reports?page=2");
    await openTable();
    expect(mockList).toHaveBeenCalledWith(2, 20, null, null);
    revisit.unmount();

    // 範囲外のページ: 行 0・total_count 1 以上は「記録が無い」と読ませない
    mockList.mockClear();
    mockList.mockImplementation((pageNo) =>
      Promise.resolve(pageNo === 9 ? page([], 3, 9) : page([ROW_101, ROW_102, ROW_OLD], 3, 1)),
    );
    renderRoute("/csv-import/daily-reports?page=9");
    expect(await screen.findByText("このページには記録がありません。")).toBeInTheDocument();
    expect(screen.queryByText("日報の取込みの記録はまだありません。")).not.toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "1 ページ目を開く" }));
    await openTable();
    expect(mockList).toHaveBeenLastCalledWith(1, 20, null, null);
  });
});
