// REQ-401 / UI-07-D15 (6)・§55.7・§55.9: 日報取込みタブが取込み・取消の応答を待つ間は、タブの切替えも止める。
// タブの切替えは router の遷移でなく useBlocker が効かず、TabsContent が日報の flow を unmount するので、
// 親の CsvImportPage と実の日報の flow を通して確かめる（入力はすべて合成）
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  commands,
  type CmdError,
  type DailyReportImportResult,
  type DailyReportPreviewData,
  type DailyReportRollbackResult,
} from "@/lib/bindings";
import { open } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import { renderWithRouter } from "@/test/render-with-router";
import { CsvImportPage } from "./CsvImportPage";
import { useCsvImportFlow } from "./hooks/useCsvImportFlow";

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ setTitle: vi.fn().mockResolvedValue(undefined) }),
}));

vi.mock("sonner", () => ({
  toast: { error: vi.fn(), success: vi.fn() },
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
vi.mock("@tauri-apps/plugin-fs", () => ({ readFile: vi.fn() }));

vi.mock("./hooks/useCsvImportFlow", () => ({ useCsvImportFlow: vi.fn() }));

vi.mock("@/lib/bindings", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/bindings")>();
  return {
    ...actual,
    commands: {
      scanRegisterSd: vi.fn(),
      parseAndValidateDailyReportFromSd: vi.fn(),
      parseAndValidateDailyReport: vi.fn(),
      commitDailyReportImport: vi.fn(),
      rollbackDailyReportImport: vi.fn(),
    },
  };
});

type CommitResponse = Awaited<ReturnType<typeof commands.commitDailyReportImport>>;
type RollbackResponse = Awaited<ReturnType<typeof commands.rollbackDailyReportImport>>;

const mockCommit = vi.mocked(commands.commitDailyReportImport);
const mockRollback = vi.mocked(commands.rollbackDailyReportImport);

const IMPORT_ID = 501;

function makePreview(): DailyReportPreviewData {
  return {
    file_info: {
      report_date: "2026-03-21",
      bundle_hash: "a".repeat(64),
      source_files: [
        { source: "Z001", filename: "Z001_260321.CSV", file_hash: "1".repeat(64), size_bytes: 10 },
        { source: "Z002", filename: "Z002_260321.CSV", file_hash: "2".repeat(64), size_bytes: 10 },
        { source: "Z005", filename: "Z005_260321.CSV", file_hash: "3".repeat(64), size_bytes: 10 },
      ],
    },
    totals: { gross_amount: 12000, net_amount: 11000 },
    payment_summary: [
      { payment_key: "cash", label: "現金", amount: 11000, count: 7, sort_order: 1 },
    ],
    department_summary: [],
    warnings: [],
    duplicate_check: { status: "NoDuplicate", same_date_imports: [] },
    preview_created_at: "2026-03-21T10:00:00",
  };
}

const RESULT: DailyReportImportResult = {
  daily_report_import_id: IMPORT_ID,
  status: "completed",
  report_date: "2026-03-21",
  gross_amount: 12000,
  net_amount: 11000,
  warning_count: 0,
};

const ROLLBACK_ERROR: CmdError = {
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

const dailyTab = () => screen.getByRole("tab", { name: "日報取込み" });
const z004Tab = () => screen.getByRole("tab", { name: "商品別CSV取込み（Z004）" });

function renderPage() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return renderWithRouter(
    <QueryClientProvider client={queryClient}>
      <CsvImportPage />
    </QueryClientProvider>,
  );
}

/** 予備の経路で 3 本を選び、preview まで進める */
async function goToPreview(user: ReturnType<typeof userEvent.setup>) {
  await user.click(await screen.findByRole("button", { name: "ファイルを選んで取り込む" }));
  await user.click(await screen.findByRole("button", { name: "確認する" }));
  await screen.findByText("取込み内容");
}

/** preview から取り込み、結果の画面まで進める */
async function goToResult(user: ReturnType<typeof userEvent.setup>) {
  mockCommit.mockResolvedValueOnce({ status: "ok", data: RESULT });
  await goToPreview(user);
  await user.click(screen.getByRole("button", { name: "取り込む" }));
  await screen.findByText("日報取込み完了");
}

async function confirmRollback(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByRole("button", { name: "取り消す" }));
  const dialog = await screen.findByRole("alertdialog");
  await user.click(within(dialog).getByRole("button", { name: "取り消す" }));
}

beforeEach(() => {
  vi.clearAllMocks();
  // Z004 の flow は待っていない idle（本 test は日報の側の止めだけを見る）
  vi.mocked(useCsvImportFlow).mockImplementation(() => ({
    state: { status: "idle" },
    selectFile: vi.fn(),
    confirmImport: vi.fn(),
    rollback: vi.fn(),
    rollbackResult: null,
    dismissError: vi.fn(),
    reset: vi.fn(),
    isParsing: false,
    isImporting: false,
    isRollingBack: false,
  }));
  vi.mocked(open).mockResolvedValue([
    "C:\\tmp\\Z001_260321.CSV",
    "C:\\tmp\\Z002_260321.CSV",
    "C:\\tmp\\Z005_260321.CSV",
  ]);
  vi.mocked(readFile).mockResolvedValue(new Uint8Array([1]));
  vi.mocked(commands.parseAndValidateDailyReport).mockResolvedValue({
    status: "ok",
    data: { preview_data: makePreview(), preview_token: "preview-token-tabs" },
  });
});

describe("CsvImportPage daily report tabs_req401", () => {
  it("REQ-401 / UI-07-D15 (6): 日報の取消の応答を待つ間はどちらのタブにも切り替えられない", async () => {
    const user = userEvent.setup();
    renderPage();
    await goToResult(user);
    expect(dailyTab()).toBeEnabled();
    expect(z004Tab()).toBeEnabled();

    const pending = deferred<RollbackResponse>();
    mockRollback.mockReturnValueOnce(pending.promise);
    await confirmRollback(user);

    await waitFor(() => {
      expect(z004Tab()).toBeDisabled();
    });
    expect(dailyTab()).toBeDisabled();
    await user.click(z004Tab());
    expect(dailyTab()).toHaveAttribute("aria-selected", "true");
    expect(screen.getByText("日報取込み完了")).toBeInTheDocument();

    const ok: DailyReportRollbackResult = {
      daily_report_import_id: IMPORT_ID,
      status: "rolled_back",
      rolled_back_at: "2026-03-21T11:00:00",
    };
    pending.resolve({ status: "ok", data: ok });
    // 応答の後に解く（成功は idle へ戻る）
    await waitFor(() => {
      expect(z004Tab()).toBeEnabled();
    });
    expect(dailyTab()).toBeEnabled();
  });

  it("REQ-401 / UI-07-D15 (6): 日報の取込みの応答を待つ間もどちらのタブにも切り替えられない", async () => {
    const user = userEvent.setup();
    renderPage();
    await goToPreview(user);

    const pending = deferred<CommitResponse>();
    mockCommit.mockReturnValueOnce(pending.promise);
    await user.click(screen.getByRole("button", { name: "取り込む" }));
    await screen.findByText("日報を取り込み中…");

    await waitFor(() => {
      expect(z004Tab()).toBeDisabled();
    });
    expect(dailyTab()).toBeDisabled();
    await user.click(z004Tab());
    expect(dailyTab()).toHaveAttribute("aria-selected", "true");
    expect(screen.getByText("日報を取り込み中…")).toBeInTheDocument();

    pending.resolve({ status: "ok", data: RESULT });
    await screen.findByText("日報取込み完了");
    expect(z004Tab()).toBeEnabled();
    expect(dailyTab()).toBeEnabled();
  });

  it("REQ-401 / §55.9: 取消の途中でタブを押しても、失敗した後は result のまま同じ取込み ID で再試行できる", async () => {
    const user = userEvent.setup();
    renderPage();
    await goToResult(user);

    const pending = deferred<RollbackResponse>();
    mockRollback.mockReturnValueOnce(pending.promise);
    await confirmRollback(user);
    await screen.findByRole("button", { name: "取り消し中…" });
    // 待つ間に Z004 タブを押す（止まっていなければ日報の flow が unmount される）
    await user.click(z004Tab());

    pending.resolve({ status: "error", error: ROLLBACK_ERROR });
    await waitFor(() => {
      expect(dailyTab()).toBeEnabled();
    });
    await user.click(dailyTab());

    expect(await screen.findByText("日報取込み完了")).toBeInTheDocument();
    expect(screen.getByText(String(IMPORT_ID))).toBeInTheDocument();
    mockRollback.mockResolvedValueOnce({ status: "error", error: ROLLBACK_ERROR });
    await confirmRollback(user);
    await waitFor(() => {
      expect(mockRollback).toHaveBeenCalledTimes(2);
    });
    expect(mockRollback).toHaveBeenNthCalledWith(1, IMPORT_ID);
    expect(mockRollback).toHaveBeenNthCalledWith(2, IMPORT_ID);
  });
});
