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
  type EjImportResult,
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
      commitRegisterSdBulkImport: vi.fn(),
      importEjFiles: vi.fn(),
    },
  };
});

type CommitResponse = Awaited<ReturnType<typeof commands.commitDailyReportImport>>;
type RollbackResponse = Awaited<ReturnType<typeof commands.rollbackDailyReportImport>>;
type EjResponse = Awaited<ReturnType<typeof commands.importEjFiles>>;

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

  it("REQ-401 / UI-07-D19 / BIZ-08-D4: 日報取込みタブの上の記録への導線は、D4 の拒否の error の状態でも出ている", async () => {
    const ENTRY = "取り込んだ日報の記録を見る（取り消すときはここから）";
    // 37 §37.3 手順 8 の固定の文
    const D4_TEXT =
      "同じ精算の日報が、別のファイルからすでに取り込まれています。二重に数えないため、取り込みません。取り込み直すときは、前の取込みを取り消してから読み直してください。";
    const user = userEvent.setup();
    renderPage();
    expect(await screen.findByRole("link", { name: ENTRY })).toHaveAttribute(
      "href",
      "/csv-import/daily-reports",
    );

    vi.mocked(commands.parseAndValidateDailyReport).mockResolvedValueOnce({
      status: "error",
      error: {
        kind: "import_error",
        message: D4_TEXT,
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    await user.click(await screen.findByRole("button", { name: "ファイルを選んで取り込む" }));
    await user.click(await screen.findByRole("button", { name: "確認する" }));
    expect(await screen.findByText(D4_TEXT)).toBeInTheDocument();
    expect(screen.getByRole("link", { name: ENTRY })).toHaveAttribute(
      "href",
      "/csv-import/daily-reports",
    );
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

  it("U2 REQ-401 / UI-07-D17 / UI-USW-D5（closure round 2 #3）: 手で選ぶ EJ の読込みと応答を待つ間は EJ のボタンと両方のタブが止まり、成功の後と失敗の後に解ける", async () => {
    const EJ_RESULT: EjImportResult = {
      inserted_count: 0,
      already_count: 0,
      conflict_count: 0,
      conflicts: [],
      held_count: 0,
      unreadable_files: ["EJ260321.TXT"],
      gaps: [],
    };
    vi.mocked(commands.scanRegisterSd).mockResolvedValueOnce({
      status: "ok",
      data: {
        scan: {
          volume_label: "E:",
          scanned_from: "2026-02-20",
          candidates: [],
          skipped_unknown_count: 0,
          tidy_failure: null,
          ej: { new_count: 0, held_count: 0, conflict_count: 0, unreadable_file_count: 1 },
        },
        scan_token: "scan-token",
      },
    });
    vi.mocked(commands.commitRegisterSdBulkImport).mockResolvedValueOnce({
      status: "ok",
      data: { items: [{ source: { type: "Ej" }, outcome: { type: "ImportedEj", ...EJ_RESULT } }] },
    });
    const user = userEvent.setup();
    renderPage();
    await user.click(await screen.findByRole("button", { name: "SD から読む" }));
    await user.click(await screen.findByRole("button", { name: "まとめて取り込む" }));
    const ejButton = () => screen.getByRole("button", { name: "EJ のファイルを選んで取り込む" });
    await screen.findByRole("list", { name: "まとめて取り込んだ結果" });
    expect(ejButton()).toBeEnabled();
    expect(dailyTab()).toBeEnabled();
    expect(z004Tab()).toBeEnabled();
    const expectLocked = async () => {
      await waitFor(() => {
        expect(ejButton()).toBeDisabled();
      });
      expect(dailyTab()).toBeDisabled();
      expect(z004Tab()).toBeDisabled();
    };
    const expectReleased = async () => {
      await waitFor(() => {
        expect(ejButton()).toBeEnabled();
      });
      expect(dailyTab()).toBeEnabled();
      expect(z004Tab()).toBeEnabled();
    };

    // 成功: 選択の確定（読込みの待ち）から応答まで止まる
    const reading = deferred<Uint8Array>();
    const importing = deferred<EjResponse>();
    vi.mocked(open).mockResolvedValueOnce(["C:\\tmp\\EJ260321_0001.TXT"]);
    vi.mocked(readFile).mockReturnValueOnce(reading.promise);
    vi.mocked(commands.importEjFiles).mockReturnValueOnce(importing.promise);
    await user.click(ejButton());
    await expectLocked();
    expect(commands.importEjFiles).not.toHaveBeenCalled();
    await user.click(z004Tab());
    expect(dailyTab()).toHaveAttribute("aria-selected", "true");

    reading.resolve(new Uint8Array([1]));
    await waitFor(() => {
      expect(commands.importEjFiles).toHaveBeenCalledTimes(1);
    });
    await expectLocked();

    importing.resolve({ status: "ok", data: EJ_RESULT });
    await expectReleased();

    // 失敗: 応答の失敗の後も解け、同じ画面で選び直せる
    const failing = deferred<EjResponse>();
    vi.mocked(open).mockResolvedValueOnce(["C:\\tmp\\EJ260321_0001.TXT"]);
    vi.mocked(commands.importEjFiles).mockReturnValueOnce(failing.promise);
    await user.click(ejButton());
    await expectLocked();
    failing.resolve({
      status: "error",
      error: { ...ROLLBACK_ERROR, message: "合成の EJ の取込みの失敗" },
    });
    expect(await screen.findByText(/合成の EJ の取込みの失敗/)).toBeInTheDocument();
    await expectReleased();
    expect(dailyTab()).toHaveAttribute("aria-selected", "true");
  });
});
