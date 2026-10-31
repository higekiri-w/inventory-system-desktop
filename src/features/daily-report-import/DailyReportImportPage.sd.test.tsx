// REQ-401 / UI-07-D15（D-111）: SD から読む画面。固定の文は 55 UI-07-D15・37 §37.9・45 §45.6b・§45.7 から写す
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  commands,
  type CmdError,
  type DailyReportPreviewData,
  type DailyReportSdCandidate,
  type DailyReportSdScan,
} from "@/lib/bindings";
import { open } from "@tauri-apps/plugin-dialog";
import { DailyReportImportPage } from "./DailyReportImportPage";

vi.mock("@tanstack/react-router", () => ({
  useBlocker: vi.fn(),
  Link: ({ children }: { children: ReactNode }) => <a href="/reports/daily">{children}</a>,
}));

vi.mock("sonner", () => ({
  toast: { error: vi.fn(), success: vi.fn() },
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
vi.mock("@tauri-apps/plugin-fs", () => ({ readFile: vi.fn() }));

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

const mockOpen = vi.mocked(open);
const mockScan = vi.mocked(commands.scanRegisterSd);
const mockFromSd = vi.mocked(commands.parseAndValidateDailyReportFromSd);
const mockCommit = vi.mocked(commands.commitDailyReportImport);

const READ_DONE =
  "読み終わりました。いつもの取込みが済んでいれば、SD はレジに戻してください（次の精算に要ります）。";

function candidate(
  key: string,
  status: DailyReportSdCandidate["status"],
  day = "21",
): DailyReportSdCandidate {
  return {
    candidate_key: key,
    path_date: `2026-03-${day}`,
    report_date: `2026-03-${day}`,
    source_filenames: [`Z001_${key}.CSV`, `Z002_${key}.CSV`, `Z005_${key}.CSV`],
    status,
  };
}

function makeScan(candidates: DailyReportSdCandidate[], skipped = 0): DailyReportSdScan {
  return {
    volume_label: "E:",
    scanned_from: "2026-02-20",
    candidates,
    skipped_unknown_count: skipped,
  };
}

function makePreview(): DailyReportPreviewData {
  return {
    file_info: {
      report_date: "2026-03-21",
      bundle_hash: "a".repeat(64),
      source_files: [
        { source: "Z001", filename: "Z001_A.CSV", file_hash: "1".repeat(64), size_bytes: 10 },
        { source: "Z002", filename: "Z002_A.CSV", file_hash: "2".repeat(64), size_bytes: 10 },
        { source: "Z005", filename: "Z005_A.CSV", file_hash: "3".repeat(64), size_bytes: 10 },
      ],
    },
    totals: { gross_amount: 12000, net_amount: 11000 },
    payment_summary: [],
    department_summary: [],
    warnings: [],
    duplicate_check: { status: "NoDuplicate", same_date_imports: [] },
    preview_created_at: "2026-03-21T10:00:00",
  };
}

function cmdError(kind: CmdError["kind"], message: string): { status: "error"; error: CmdError } {
  return {
    status: "error",
    error: { kind, message, field: null, error_id: null, stocktake_recovery: null },
  };
}

function renderPage() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <DailyReportImportPage />
    </QueryClientProvider>,
  );
}

async function scanWith(scan: DailyReportSdScan) {
  mockScan.mockResolvedValueOnce({ status: "ok", data: { scan, scan_token: "scan-token" } });
  const user = userEvent.setup();
  renderPage();
  await user.click(screen.getByRole("button", { name: "SD から読む" }));
  await screen.findByText(READ_DONE);
  return user;
}

function row(name: string) {
  const list = screen.getByRole("list", { name: "SD の精算の一覧" });
  const item = within(list)
    .getAllByRole("listitem")
    .find((element) => (element.textContent ?? "").includes(name));
  if (!item) throw new Error(`row not found: ${name}`);
  return item;
}

beforeEach(() => {
  vi.clearAllMocks();
  mockFromSd.mockResolvedValue({
    status: "ok",
    data: { preview_data: makePreview(), preview_token: "preview-token" },
  });
});

describe("DailyReportImportPage SD UI-07-D15 REQ-401", () => {
  it("REQ-401 / UI-07-D15 (3): five states show a Japanese label and an icon, only NotImported has 確認する", async () => {
    await scanWith(
      makeScan([
        candidate("A", "NotImported"),
        candidate("B", "Imported"),
        candidate("C", "SameSettlementImported"),
        candidate("D", "Incomplete"),
        candidate("E", "Unreadable"),
      ]),
    );
    const expected: [string, string][] = [
      ["A", "取り込めます"],
      ["B", "取込み済み"],
      ["C", "同じ精算を取込み済み"],
      ["D", "ファイルがそろっていません"],
      ["E", "読めません"],
    ];
    for (const [key, label] of expected) {
      const item = row(`Z001_${key}.CSV`);
      const badge = within(item).getByText(label);
      expect(badge.querySelector("svg")).not.toBeNull();
      expect(within(item).queryAllByRole("button", { name: "確認する" })).toHaveLength(
        key === "A" ? 1 : 0,
      );
    }
    expect(screen.queryByText("新しい精算はありません")).not.toBeInTheDocument();
  });

  it("REQ-401 / UI-07-D15 (2)(3): read-done text and the summary text follow the state counts", async () => {
    await scanWith(makeScan([candidate("A", "Incomplete"), candidate("B", "Unreadable")]));
    expect(
      screen.getByText(
        "取り込める精算はありません。取り込めない精算が 2 件あります（「ファイルがそろっていません」「読めません」の行）。",
      ),
    ).toBeInTheDocument();
    expect(screen.queryByText("新しい精算はありません")).not.toBeInTheDocument();
  });

  it.each([
    ["no candidates", makeScan([])],
    [
      "only imported",
      makeScan([candidate("A", "Imported"), candidate("B", "SameSettlementImported")]),
    ],
  ])("REQ-401 / UI-07-D15 (3)(c): %s shows 新しい精算はありません", async (_label, scan) => {
    await scanWith(scan);
    expect(screen.getByText("新しい精算はありません")).toBeInTheDocument();
    expect(screen.getByText(READ_DONE)).toBeInTheDocument();
  });

  it("REQ-401 / UI-07-D15 (3)(a): an importable row hides the summary text even with an unreadable row", async () => {
    await scanWith(makeScan([candidate("A", "NotImported"), candidate("B", "Unreadable")]));
    expect(screen.queryByText(/取り込める精算はありません/)).not.toBeInTheDocument();
    expect(screen.queryByText("新しい精算はありません")).not.toBeInTheDocument();
  });

  it.each([
    [3, true],
    [0, false],
  ])(
    "REQ-401 / UI-07-D15 (3): skipped_unknown_count %d shows the note: %s",
    async (count, shown) => {
      await scanWith(makeScan([candidate("A", "NotImported")], count));
      expect(screen.queryByText("読まなかったファイル 3 件") !== null).toBe(shown);
      expect(screen.queryByText(/読まなかったファイル/) !== null).toBe(shown);
    },
  );

  it("REQ-401 / UI-07-D15 (1): SD not found shows the fixed text and 場所を選ぶ, which scans the chosen folder", async () => {
    mockScan.mockResolvedValueOnce(
      cmdError(
        "import_error",
        "SD が見つかりません。レジの SD をこの PC に差してから、もう一度読んでください。",
      ),
    );
    const user = userEvent.setup();
    renderPage();
    expect(screen.queryByRole("button", { name: "場所を選ぶ" })).not.toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "SD から読む" }));
    expect(
      await screen.findByText(
        "SD が見つかりません。レジの SD をこの PC に差してから、もう一度読んでください。",
      ),
    ).toBeInTheDocument();

    mockOpen.mockResolvedValueOnce("D:\\SDの写し\\CASIO");
    mockScan.mockResolvedValueOnce({
      status: "ok",
      data: { scan: makeScan([candidate("A", "NotImported")]), scan_token: "scan-token" },
    });
    await user.click(screen.getByRole("button", { name: "場所を選ぶ" }));
    await screen.findByText(READ_DONE);
    expect(mockOpen).toHaveBeenCalledWith({ directory: true });
    expect(mockScan).toHaveBeenLastCalledWith("D:\\SDの写し\\CASIO");
  });

  it("REQ-401 / UI-07-D15 (4): ファイルを選び直す on an SD preview returns to the same list", async () => {
    const user = await scanWith(makeScan([candidate("A", "NotImported")]));
    await user.click(within(row("Z001_A.CSV")).getByRole("button", { name: "確認する" }));
    await screen.findByText("取込み内容");
    expect(mockFromSd).toHaveBeenCalledWith("scan-token", "A");

    await user.click(screen.getByRole("button", { name: "ファイルを選び直す" }));

    expect(await screen.findByText(READ_DONE)).toBeInTheDocument();
    expect(within(row("Z001_A.CSV")).getByText("取り込めます")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "SD から読む" })).toBeInTheDocument();
    expect(mockOpen).not.toHaveBeenCalled();
    expect(mockScan).toHaveBeenCalledTimes(1);
  });

  it.each([
    ["import_error", "SD を読んでから時間がたちました。もう一度 SD を読んでください。"],
    ["validation", "この精算は取り込めません"],
  ] as const)(
    "REQ-401 / UI-07-D15 (5): SD parse failure (%s) shows the message and 戻る returns to the list",
    async (kind, message) => {
      mockFromSd.mockResolvedValueOnce(cmdError(kind, message));
      const user = await scanWith(makeScan([candidate("A", "NotImported")]));
      await user.click(within(row("Z001_A.CSV")).getByRole("button", { name: "確認する" }));
      expect(await screen.findByText(message)).toBeInTheDocument();

      await user.click(screen.getByRole("button", { name: "戻る" }));
      expect(await screen.findByText(READ_DONE)).toBeInTheDocument();

      mockScan.mockResolvedValueOnce({
        status: "ok",
        data: { scan: makeScan([candidate("A", "NotImported")]), scan_token: "scan-token-2" },
      });
      await user.click(screen.getByRole("button", { name: "SD から読む" }));
      await waitFor(() => {
        expect(mockScan).toHaveBeenCalledTimes(2);
      });
    },
  );

  it("REQ-401 / UI-07-D15 (4) / BIZ-08-D5: after a copy failure, 戻る then ファイルを選び直す returns to the list", async () => {
    mockCommit.mockResolvedValueOnce(
      cmdError(
        "internal",
        "SD から読んだファイルの写しを PC に保存できなかったため、取り込みませんでした。PC の空き容量を確かめて、もう一度取り込んでください。",
      ),
    );
    const user = await scanWith(makeScan([candidate("A", "NotImported")]));
    await user.click(within(row("Z001_A.CSV")).getByRole("button", { name: "確認する" }));
    await screen.findByText("取込み内容");
    await user.click(screen.getByRole("button", { name: "取り込む" }));
    expect(
      await screen.findByText(
        /SD から読んだファイルの写しを PC に保存できなかったため、取り込みませんでした。/,
      ),
    ).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "戻る" }));
    await screen.findByText("取込み内容");
    await user.click(screen.getByRole("button", { name: "ファイルを選び直す" }));

    expect(await screen.findByText(READ_DONE)).toBeInTheDocument();
    expect(within(row("Z001_A.CSV")).getByText("取り込めます")).toBeInTheDocument();
    expect(mockOpen).not.toHaveBeenCalled();
  });

  it("REQ-401 / UI-07-D16 (a)(c): choosing an undeterminable file on the SD list keeps the scanned list", async () => {
    const user = await scanWith(makeScan([candidate("A", "NotImported")]));
    mockOpen.mockResolvedValueOnce(["C:\\a\\notes.CSV"]);
    await user.click(screen.getByRole("button", { name: "ファイルを選んで取り込む" }));

    expect(
      await screen.findByText("Z001・Z002・Z005 のどれかの名前のファイルを選んでください。"),
    ).toBeInTheDocument();
    expect(screen.getByText(READ_DONE)).toBeInTheDocument();
    expect(within(row("Z001_A.CSV")).getByText("取り込めます")).toBeInTheDocument();
  });

  it("REQ-401 / UI-07-D15 (5) / UI-07-D16 (c): a selection text on the SD list is not carried into the SD preview", async () => {
    const user = await scanWith(makeScan([candidate("A", "NotImported")]));
    mockOpen.mockResolvedValueOnce(["C:\\a\\notes.CSV"]);
    await user.click(screen.getByRole("button", { name: "ファイルを選んで取り込む" }));
    const unknown = "Z001・Z002・Z005 のどれかの名前のファイルを選んでください。";
    expect(await screen.findByText(unknown)).toBeInTheDocument();

    await user.click(within(row("Z001_A.CSV")).getByRole("button", { name: "確認する" }));
    await screen.findByText("取込み内容");
    expect(screen.queryByText(unknown)).not.toBeInTheDocument();
  });

  it("REQ-401 / UI-07-D15 (6): while a rollback is pending, SD の一覧へ戻る is disabled", async () => {
    mockCommit.mockResolvedValueOnce({
      status: "ok",
      data: {
        daily_report_import_id: 7,
        status: "completed",
        report_date: "2026-03-21",
        gross_amount: 12000,
        net_amount: 11000,
        warning_count: 0,
      },
    });
    vi.mocked(commands.rollbackDailyReportImport).mockReturnValueOnce(new Promise(() => undefined));
    const user = await scanWith(makeScan([candidate("A", "NotImported")]));
    await user.click(within(row("Z001_A.CSV")).getByRole("button", { name: "確認する" }));
    await screen.findByText("取込み内容");
    await user.click(screen.getByRole("button", { name: "取り込む" }));
    await screen.findByText("日報取込み完了");
    expect(screen.getByRole("button", { name: "SD の一覧へ戻る" })).toBeEnabled();

    await user.click(screen.getByRole("button", { name: "取り消す" }));
    await user.click(
      within(await screen.findByRole("alertdialog")).getByRole("button", { name: "取り消す" }),
    );

    expect(await screen.findByText("取り消し中…")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "SD の一覧へ戻る" })).toBeDisabled();
    expect(screen.getByText("日報取込み完了")).toBeInTheDocument();
  });

  it("REQ-401 / UI-07-D15 (6): after importing A, SD の一覧へ戻る shows A imported and B still importable without rescanning", async () => {
    mockCommit.mockResolvedValueOnce({
      status: "ok",
      data: {
        daily_report_import_id: 7,
        status: "completed",
        report_date: "2026-03-21",
        gross_amount: 12000,
        net_amount: 11000,
        warning_count: 0,
      },
    });
    const user = await scanWith(
      makeScan([candidate("A", "NotImported"), candidate("B", "NotImported", "20")]),
    );
    await user.click(within(row("Z001_A.CSV")).getByRole("button", { name: "確認する" }));
    await screen.findByText("取込み内容");
    await user.click(screen.getByRole("button", { name: "取り込む" }));
    await screen.findByText("日報取込み完了");

    await user.click(screen.getByRole("button", { name: "SD の一覧へ戻る" }));

    expect(await screen.findByText(READ_DONE)).toBeInTheDocument();
    const a = row("Z001_A.CSV");
    expect(within(a).getByText("取込み済み")).toBeInTheDocument();
    expect(within(a).queryByRole("button", { name: "確認する" })).not.toBeInTheDocument();
    const b = row("Z001_B.CSV");
    expect(within(b).getByText("取り込めます")).toBeInTheDocument();

    await user.click(within(b).getByRole("button", { name: "確認する" }));
    await waitFor(() => {
      expect(mockFromSd).toHaveBeenLastCalledWith("scan-token", "B");
    });
    expect(mockScan).toHaveBeenCalledTimes(1);
  });
});
