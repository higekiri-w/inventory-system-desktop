import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { DailyReportImportResult, DailyReportPreviewData } from "@/lib/bindings";
import { renderWithRouter } from "@/test/render-with-router";
import { DailyReportImportPage } from "./DailyReportImportPage";
import { useDailyReportImportFlow } from "./hooks/useDailyReportImportFlow";
import type { DailyReportImportState } from "./types";

vi.mock("./hooks/useDailyReportImportFlow", () => ({
  useDailyReportImportFlow: vi.fn(),
}));

const mockUseFlow = vi.mocked(useDailyReportImportFlow);
const chooseFiles = vi.fn();
const backToSdList = vi.fn();
const confirmImport = vi.fn();
const rollback = vi.fn();
const dismissError = vi.fn();
const reset = vi.fn();

function makePreview(
  status: DailyReportPreviewData["duplicate_check"]["status"],
): DailyReportPreviewData {
  return {
    file_info: {
      report_date: "2026-03-21",
      bundle_hash: "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
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
    department_summary: [
      {
        department_id: null,
        raw_department_name: "未対応部門",
        normalized_department_name: null,
        amount: 8000,
        quantity: 5,
        count: 3,
        sort_order: 1,
      },
    ],
    warnings: [
      {
        code: "unmatched_department",
        message: "未対応部門は部門マスタにありません",
        source_file: "Z005",
        line_no: 12,
      },
    ],
    duplicate_check: {
      status,
      same_date_imports:
        status === "AdditionalImportConfirmationRequired"
          ? [
              {
                id: 100,
                source_filenames: ["Z001_old.CSV", "Z002_old.CSV", "Z005_old.CSV"],
                gross_amount: 9000,
                net_amount: 8000,
                imported_at: "2026-03-21T09:00:00",
              },
              {
                id: 99,
                source_filenames: ["Z001_older.CSV", "Z002_older.CSV", "Z005_older.CSV"],
                gross_amount: null,
                net_amount: 7000,
                imported_at: "2026-03-21T08:00:00",
              },
            ]
          : [],
    },
    preview_created_at: "2026-03-21T10:00:00",
  };
}

function makeResult(): DailyReportImportResult {
  return {
    daily_report_import_id: 501,
    status: "completed",
    report_date: "2026-03-21",
    gross_amount: 12000,
    net_amount: 11000,
    warning_count: 1,
  };
}

function setFlow(state: DailyReportImportState, isImporting = false) {
  mockUseFlow.mockReturnValue({
    state: { ...state, lastSelectionError: null },
    scanError: null,
    manualFiles: [],
    pendingReplacement: null,
    scanSd: vi.fn(),
    chooseSdLocation: vi.fn(),
    confirmSdCandidate: vi.fn(),
    backToSdList,
    chooseFiles,
    removeManualFile: vi.fn(),
    confirmReplacement: vi.fn(),
    cancelReplacement: vi.fn(),
    confirmManualFiles: vi.fn(),
    confirmImport,
    rollback,
    dismissError,
    reset,
    isParsing: false,
    isImporting,
    isRollingBack: false,
    isBlocking: false,
  });
}

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  document.body.style.pointerEvents = "";
});

describe("DailyReportImportPage_req401", () => {
  // SPEC-COLOR-EMPHASIS-RT-1 / D-CE5: 待ちの spinner は進行中の色（操作の色ではない）。
  it("REQ-401 / D-CE5: the parsing spinner uses the ongoing color", async () => {
    setFlow({ status: "parsing", filenames: ["Z001_260321.CSV"] });

    renderWithRouter(<DailyReportImportPage />);

    const status = await screen.findByRole("status");
    expect(status).toHaveTextContent("日報ファイルを解析中…");
    const spinner = status.querySelector("svg");
    expect(spinner).toHaveClass("animate-spin", "text-ongoing");
    expect(spinner).not.toHaveClass("text-primary");
  });

  it("REQ-401: preview shows target date, totals, payment, department, and unmatched warning", async () => {
    setFlow({
      status: "preview",
      preview: makePreview("NoDuplicate"),
      previewToken: "preview-token-req401",
      filenames: ["Z001_260321.CSV", "Z002_260321.CSV", "Z005_260321.CSV"],
    });

    renderWithRouter(<DailyReportImportPage />);

    expect(await screen.findByText("取込み内容")).toBeInTheDocument();
    expect(screen.getByText("2026-03-21")).toBeInTheDocument();
    expect(screen.getByText("¥12,000")).toBeInTheDocument();
    expect(screen.getAllByText("¥11,000").length).toBeGreaterThan(0);
    expect(screen.getByText("現金")).toBeInTheDocument();
    expect(screen.getByText("未対応部門")).toBeInTheDocument();
    expect(screen.getByText("未対応部門は部門マスタにありません")).toBeInTheDocument();
  });

  it("REQ-401 / I-U3..I-U6 / SPEC-SDI-D5: additional preview opens exact shared confirmation", async () => {
    // REQ-401 / I-U3 / I-U4 / I-U5 / I-U6 / SPEC-SDI-D5: adjacent exact UI、全bundle、cancel、true一回送信を固定する。
    const user = userEvent.setup();
    setFlow({
      status: "preview",
      preview: makePreview("AdditionalImportConfirmationRequired"),
      previewToken: "preview-token-req401",
      filenames: ["Z001_260321.CSV", "Z002_260321.CSV", "Z005_260321.CSV"],
    });

    renderWithRouter(<DailyReportImportPage />);

    const importButton = await screen.findByRole("button", { name: "取り込む" });
    // Badge は主情報を上部 Alert（本画面は元々上部専用スロットに配置済み）に譲り、
    // 補助的な状態表示へ改名（gated Amendment 4、PreviewStep.tsx と対称）。
    const badge = screen.getByText("同日データあり");
    expect(badge).toBeInTheDocument();
    expect(screen.queryByText("追加確認")).not.toBeInTheDocument();
    // gated Amendment 5（owner L3-lite round 3 裁定③）: 黒枠でなく soft warning token
    // （PreviewStep.tsx と対称）。
    expect(badge.closest('[data-slot="badge"]')).toHaveClass(
      "border-warning-border",
      "bg-warning-soft",
      "text-warning-strong",
    );
    await user.click(importButton);
    expect(screen.getByText("同じ日のデータを追加で取り込みますか？")).toBeInTheDocument();
    expect(screen.queryByRole("checkbox")).not.toBeInTheDocument();
    expect(
      screen.getByText(
        "この操作は既存の取込みを置き換えません。対象日の売上に今回分を追加します。復旧用に書き出した同内容のファイルを選んでいないか確認してください。",
      ),
    ).toBeInTheDocument();
    const dialog = screen.getByRole("alertdialog");
    expect(dialog).toHaveTextContent("既存分（2回）");
    // gated Amendment 5（owner L3-lite round 3 裁定①）: 共通 alert-dialog.tsx の
    // data-[size=default]:sm:max-w-lg に詳細度で勝つ data-size 修飾子付きの幅指定
    // （AdditionalImportConfirmDialog.tsx は両 tab 共有のため PreviewStep.test.tsx と重複
    // assert だが、日報 tab 経由でも同じ dialog が適用されることを固定する）。
    expect(dialog).toHaveClass("data-[size=default]:sm:max-w-3xl");
    // UI-07-D13 の規定項目（import ID / filename(s) / 金額 / 取込み日時）が省略なく
    // 全件表示されることを確認する（T3 項目完全性 oracle、gated Amendment 3/4 でも不変）。
    // DSR-16 構造 assert: 既存分・今回分とも同一の列を揃えた表（比較目的の structured
    // list）で render される（gated Amendment 4: 今回分の definition list 分離を廃止）。
    const table = within(dialog).getByRole("table");
    expect(within(table).getByRole("columnheader", { name: "取込み ID" })).toBeInTheDocument();
    expect(within(table).getByRole("columnheader", { name: "ファイル名" })).toBeInTheDocument();
    expect(within(table).getByRole("columnheader", { name: "合計金額" })).toBeInTheDocument();
    expect(within(table).getByRole("columnheader", { name: "取込み日時" })).toBeInTheDocument();
    // ヘッダ行 + 既存分2件 + 今回分1行 = 4行。per-card の反復ではなく1つの表に全件が並ぶ。
    expect(within(table).getAllByRole("row")).toHaveLength(4);
    expect(within(table).getByText("100")).toBeInTheDocument();
    // 複数 filenames は " / " 連結でなく個別行表示（実装後レビュー round 1 是正 3）。
    expect(within(table).getByText("Z001_old.CSV")).toBeInTheDocument();
    expect(within(table).getByText("Z002_old.CSV")).toBeInTheDocument();
    expect(within(table).getByText("Z005_old.CSV")).toBeInTheDocument();
    expect(table).toHaveTextContent("総売上 ¥9,000 / 純売上 ¥8,000");
    // 取込み日時は人間向け表示（既存 formatDateTime 慣行: ISOの"T"を空白へ）。
    expect(table).toHaveTextContent("2026-03-21 09:00:00");
    expect(within(table).getByText("99")).toBeInTheDocument();
    expect(within(table).getByText("Z001_older.CSV")).toBeInTheDocument();
    expect(within(table).getByText("Z002_older.CSV")).toBeInTheDocument();
    expect(within(table).getByText("Z005_older.CSV")).toBeInTheDocument();
    expect(table).toHaveTextContent("総売上 未取得 / 純売上 ¥7,000");
    // 今回分は同一 table の最終行（ID 列に「今回」Badge、既存分と同列位置）。
    expect(within(table).getByText("今回")).toBeInTheDocument();
    expect(within(table).getByText("Z001_260321.CSV")).toBeInTheDocument();
    expect(within(table).getByText("Z002_260321.CSV")).toBeInTheDocument();
    expect(within(table).getByText("Z005_260321.CSV")).toBeInTheDocument();
    expect(table).toHaveTextContent("総売上 ¥12,000 / 純売上 ¥11,000");
    expect(table).toHaveTextContent("2026-03-21 10:00:00");
    await user.click(screen.getByRole("button", { name: "キャンセル" }));
    expect(confirmImport).not.toHaveBeenCalled();
    await user.click(importButton);
    await user.click(screen.getByRole("button", { name: "追加で取り込む" }));
    expect(confirmImport).toHaveBeenCalledWith(true);
  });

  it("REQ-401: already imported preview shows a page-level alert and opens native reselect dialog", async () => {
    const user = userEvent.setup();
    setFlow({
      status: "preview",
      preview: makePreview("AlreadyImported"),
      previewToken: "preview-token-req401",
      filenames: ["Z001_260321.CSV", "Z002_260321.CSV", "Z005_260321.CSV"],
    });

    renderWithRouter(<DailyReportImportPage />);

    expect(
      await screen.findByText("この日報は取込み済みです。二重取込みはできません。"),
    ).toBeInTheDocument();
    expect(screen.getByText("別の日報ファイルを選び直してください。")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "ファイルを選び直す" }));
    expect(chooseFiles).toHaveBeenCalledTimes(1);
    expect(backToSdList).not.toHaveBeenCalled();
    expect(confirmImport).not.toHaveBeenCalled();
    expect(screen.queryByLabelText("Z001 Z002 Z005 ファイルを選び直す")).not.toBeInTheDocument();
  });

  it("REQ-401 / I-U6 / SPEC-SDI-D5: importing disables daily additional confirmation", async () => {
    // REQ-401 / I-U6 / SPEC-SDI-D5: importing中はdialog/commandを開始できない。
    const user = userEvent.setup();
    setFlow(
      {
        status: "preview",
        preview: makePreview("AdditionalImportConfirmationRequired"),
        previewToken: "preview-token-req401",
        filenames: ["Z001_260321.CSV", "Z002_260321.CSV", "Z005_260321.CSV"],
      },
      true,
    );
    renderWithRouter(<DailyReportImportPage />);

    const importButton = await screen.findByRole("button", { name: "取り込む" });
    expect(importButton).toBeDisabled();
    await user.click(importButton);
    expect(confirmImport).not.toHaveBeenCalled();
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
  });

  it("REQ-401 / I-U3 / SPEC-SDI-D5: additional required preview shows exact alert", async () => {
    // REQ-401 / I-U3 / SPEC-SDI-D5: 日報tabもshared Alert文言と非色依存statusを使う。
    setFlow({
      status: "preview",
      preview: makePreview("AdditionalImportConfirmationRequired"),
      previewToken: "preview-token-req401",
      filenames: ["Z001_260321.CSV", "Z002_260321.CSV", "Z005_260321.CSV"],
    });

    renderWithRouter(<DailyReportImportPage />);

    expect(await screen.findByText("同じ日の取込みがあります")).toBeInTheDocument();
    expect(
      screen.getByText("既存分を残したまま今回分を追加します。内容を確認してください。"),
    ).toBeInTheDocument();
  });

  it("REQ-401 / I-U7 / SPEC-SDI-D7: result rollback identifies the selected import", async () => {
    // REQ-401 / I-U7 / SPEC-SDI-D7: exact ID/date/files/amountとsibling残存を明示する。
    const user = userEvent.setup();
    setFlow({
      status: "result",
      result: makeResult(),
      reportDate: "2026-03-21",
      filenames: ["Z001_260321.CSV", "Z002_260321.CSV", "Z005_260321.CSV"],
    });

    renderWithRouter(<DailyReportImportPage />);

    expect(await screen.findByText("日報取込み完了")).toBeInTheDocument();
    expect(screen.getByText("在庫数は変わりません")).toBeInTheDocument();
    expect(screen.getByText("取消しても在庫数は変わりません。")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "日次売上を見る" })).toHaveAttribute(
      "href",
      "/reports/daily?date=2026-03-21",
    );

    await user.click(screen.getByRole("button", { name: "取り消す" }));
    const rollbackDialog = screen.getByRole("alertdialog");
    expect(rollbackDialog).toBeInTheDocument();
    expect(screen.getByText("日報取込みを取り消しますか？")).toBeInTheDocument();
    expect(rollbackDialog).toHaveTextContent(
      "この取込みだけを取り消します。同じ日の他の取込みは残ります。取消しても在庫数は変わりません。",
    );
    // UI-07-D14 の規定項目（Scope 2 構造化: ID / 対象日 / source filenames / 総売上 / 純売上）が
    // ラベル付きで全て残っていることを確認する（T2）。source filenames は 3 件 fixture で
    // 全件を個別 assert する（先頭 1 件化 mutation の検出 — gated Amendment 2、
    // Codex cross review 実測 survivor の是正）。
    expect(within(rollbackDialog).getByText("取込み ID")).toBeInTheDocument();
    expect(within(rollbackDialog).getByText("501")).toBeInTheDocument();
    expect(within(rollbackDialog).getByText("対象日")).toBeInTheDocument();
    expect(within(rollbackDialog).getByText("ファイル名")).toBeInTheDocument();
    expect(within(rollbackDialog).getByText("Z001_260321.CSV")).toBeInTheDocument();
    expect(within(rollbackDialog).getByText("Z002_260321.CSV")).toBeInTheDocument();
    expect(within(rollbackDialog).getByText("Z005_260321.CSV")).toBeInTheDocument();
    expect(within(rollbackDialog).getByText("総売上")).toBeInTheDocument();
    expect(within(rollbackDialog).getByRole("button", { name: "取り消す" })).toHaveAttribute(
      "data-variant",
      "destructive",
    );
    expect(rollbackDialog).toHaveTextContent("¥12,000");
    expect(within(rollbackDialog).getByText("純売上")).toBeInTheDocument();
    expect(rollbackDialog).toHaveTextContent("¥11,000");

    await user.click(screen.getByRole("button", { name: "キャンセル" }));
    expect(rollback).not.toHaveBeenCalled();
    await waitFor(() => {
      expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    });
  });

  it("REQ-401: result rollback confirm calls rollback only from the dialog action", async () => {
    const user = userEvent.setup();
    setFlow({
      status: "result",
      result: makeResult(),
      reportDate: "2026-03-21",
      filenames: ["Z001_260321.CSV", "Z002_260321.CSV", "Z005_260321.CSV"],
    });

    renderWithRouter(<DailyReportImportPage />);
    await user.click(await screen.findByRole("button", { name: "取り消す" }));
    await waitFor(() => {
      expect(screen.getByRole("alertdialog")).toBeInTheDocument();
    });
    const dialog = screen.getByRole("alertdialog");
    const action = dialog.querySelector('[data-slot="alert-dialog-action"]');
    expect(action).not.toBeNull();
    fireEvent.click(action as HTMLElement);
    expect(rollback).toHaveBeenCalledWith(501);
  });

  it("test_daily_report_result_cta_daily_sales_date_req501", async () => {
    const user = userEvent.setup();
    setFlow({
      status: "result",
      result: makeResult(),
      reportDate: "2026-03-21",
      filenames: ["Z001_260321.CSV", "Z002_260321.CSV", "Z005_260321.CSV"],
    });

    const { router } = renderWithRouter(<DailyReportImportPage />);

    const dailySalesLink = await screen.findByRole("link", { name: "日次売上を見る" });
    expect(dailySalesLink).toHaveAttribute("href", "/reports/daily?date=2026-03-21");

    // C3/C4: href assertion だけでなく click による実 SPA 遷移を証明する
    // (X3 mutation: static 代表を生 <a> に戻した場合に red 化する)。
    await user.click(dailySalesLink);
    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/reports/daily");
    });
    expect(router.state.location.search).toEqual({ date: "2026-03-21" });
  });

  it("T1 (UI 表示磨き batch Scope 1): 完了画面の action ボタン群が単一の flex 行 container 配下に render される", async () => {
    setFlow({
      status: "result",
      result: makeResult(),
      reportDate: "2026-03-21",
      filenames: ["Z001_260321.CSV", "Z002_260321.CSV", "Z005_260321.CSV"],
    });

    renderWithRouter(<DailyReportImportPage />);

    const dailySalesLink = await screen.findByRole("link", { name: "日次売上を見る" });
    const rollbackButton = screen.getByRole("button", { name: "取り消す" });
    // Z004 側 src/features/csv-import/components/ResultStep.tsx の action の container と同型の `flex flex-wrap gap-2` container 配下に
    // 両方の action が並ぶことを確認する（間隔調整ではなく構造化、Plan Review round 1 P3-3）。
    const buttonRow = dailySalesLink.closest("div");
    expect(buttonRow).not.toBeNull();
    expect(buttonRow).toHaveClass("flex", "flex-wrap", "gap-2");
    expect(buttonRow).toContainElement(rollbackButton);
  });
});

it.each([
  // D-094（L3 round 1 で試しを採る）: 取込み済みは上部 Alert と同じ危険・失敗。同日追加確認は注意・確認のまま。
  [
    "AlreadyImported",
    "取込み済み",
    "destructive",
    ["border-destructive-border", "bg-destructive-soft", "text-destructive-strong"],
  ],
  [
    "AdditionalImportConfirmationRequired",
    "同日データあり",
    "warning",
    ["border-warning-border", "bg-warning-soft", "text-warning-strong"],
  ],
  [
    "NoDuplicate",
    "確認済み",
    "success",
    ["border-success-border", "bg-success-soft", "text-success-strong"],
  ],
] as const)(
  "SC6 / REQ-401: %s has exactly one matching tone",
  async (status, label, tone, classes) => {
    setFlow({
      status: "preview",
      preview: makePreview(status),
      previewToken: "preview-token-req401",
      filenames: ["Z001.csv", "Z002.csv", "Z005.csv"],
    });
    renderWithRouter(<DailyReportImportPage />);
    const badge = (await screen.findByText(label)).closest('[data-slot="badge"]');
    expect(badge).toHaveAttribute("data-variant", "outline");
    expect(badge).toHaveAttribute("data-tone", tone);
    expect(badge).toHaveClass(...classes);
    for (const expected of classes)
      expect(badge?.className.split(/\s+/).filter((token) => token === expected)).toHaveLength(1);
    expect(badge?.querySelector('svg[aria-hidden="true"]')).toBeInTheDocument();
    // 他の tone の地を持たない（取込み済みが注意・確認の地へ戻る、同日追加確認が危険・失敗へ寄る退行を拒む）。
    for (const other of ["bg-warning-soft", "bg-destructive-soft", "bg-success-soft"])
      if (!(classes as readonly string[]).includes(other)) expect(badge).not.toHaveClass(other);
    if (status === "AlreadyImported") {
      const alert = screen
        .getByText("この日報は取込み済みです。二重取込みはできません。")
        .closest('[data-slot="alert"]');
      expect(alert).toHaveAttribute("data-variant", "destructive");
    }
    if (status === "AdditionalImportConfirmationRequired") {
      const alert = screen.getByText("同じ日の取込みがあります").closest('[data-slot="alert"]');
      expect(alert).toHaveAttribute("data-variant", "warning");
      expect(alert).toHaveAttribute("role", "alert");
      // DSR-08 / AC-L3-12: warning の Alert は部品が icon を描かないため、画面が三角 icon を置く。
      expect(alert?.querySelectorAll("svg")).toHaveLength(1);
      expect(alert?.querySelector("svg")).toHaveAttribute("aria-hidden", "true");
    }
  },
);

it("SC29 / REQ-401: successful daily import is a success status badge", async () => {
  setFlow({
    status: "result",
    result: makeResult(),
    reportDate: "2026-03-21",
    filenames: ["Z001.csv", "Z002.csv", "Z005.csv"],
  });
  renderWithRouter(<DailyReportImportPage />);
  const badge = (await screen.findByText("成功")).closest('[data-slot="badge"]');
  expect(badge).toHaveAttribute("data-variant", "outline");
  expect(badge).toHaveAttribute("data-tone", "success");
  expect(badge).toHaveClass("border-success-border", "bg-success-soft", "text-success-strong");
  expect(badge?.querySelector('svg[aria-hidden="true"]')).toBeInTheDocument();
});
