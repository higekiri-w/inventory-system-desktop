import { describe, expect, it } from "vitest";
import type {
  DailyReportImportResult,
  DailyReportPreviewData,
  DailyReportSdScan,
  RegisterSdBulkResponse,
} from "@/lib/bindings";
import type { InvokeError } from "@/lib/invoke";
import { dailyReportImportReducer } from "./reducer";
import type { DailyReportImportState, DailyReportSdOrigin } from "./types";

const preview = {} as DailyReportPreviewData;
const result = {} as DailyReportImportResult;
const error = {} as InvokeError;

describe("dailyReportImportReducer REQ-401", () => {
  it("test_daily_report_ui_req401_preview_to_importing_carries_snapshot", () => {
    const state: DailyReportImportState = {
      status: "preview",
      preview,
      previewToken: "token",
      filenames: ["Z001.csv", "Z002.csv", "Z005.csv"],
    };

    const next = dailyReportImportReducer(state, {
      type: "confirm_import",
      additionalImportConfirmed: true,
    });

    expect(next).toEqual({
      status: "importing",
      preview,
      previewToken: "token",
      additionalImportConfirmed: true,
      filenames: ["Z001.csv", "Z002.csv", "Z005.csv"],
    });
  });

  it("test_daily_report_ui_req401_import_failed_recovers_to_preview", () => {
    const state: DailyReportImportState = {
      status: "importing",
      preview,
      previewToken: "token",
      additionalImportConfirmed: false,
      filenames: ["Z001.csv", "Z002.csv", "Z005.csv"],
    };

    const failed = dailyReportImportReducer(state, {
      type: "import_failed",
      error,
      recoverTo: "preview",
    });
    const recovered = dailyReportImportReducer(failed, { type: "dismiss_error" });

    expect(recovered.status).toBe("preview");
    if (recovered.status === "preview") {
      expect(recovered.preview).toBe(preview);
      expect(recovered.previewToken).toBe("token");
    }
  });

  it("test_daily_report_ui_req401_result_rollback_returns_idle", () => {
    const state: DailyReportImportState = {
      status: "result",
      result,
      reportDate: "2026-03-21",
      filenames: ["Z001.CSV", "Z002.CSV", "Z005.CSV"],
    };
    expect(dailyReportImportReducer(state, { type: "rollback_succeeded" })).toEqual({
      status: "idle",
    });
  });

  it("test_daily_report_ui_req401_invalid_actions_keep_current_state", () => {
    const idle: DailyReportImportState = { status: "idle" };
    expect(
      dailyReportImportReducer(idle, {
        type: "confirm_import",
        additionalImportConfirmed: false,
      }),
    ).toBe(idle);

    const previewState: DailyReportImportState = {
      status: "preview",
      preview,
      previewToken: "token",
      filenames: ["Z001.csv", "Z002.csv", "Z005.csv"],
    };
    expect(
      dailyReportImportReducer(previewState, {
        type: "rollback_succeeded",
      }),
    ).toBe(previewState);
  });
});

// UI-07-D15 (4)〜(6)（D-111）: SD から来たことの持ち越しと一覧へ戻る遷移
const scan: DailyReportSdScan = {
  volume_label: "E:",
  scanned_from: "2026-02-20",
  skipped_unknown_count: 0,
  tidy_failure: null,
  ej: null,
  candidates: [
    {
      candidate_key: "key-a",
      path_date: "2026-03-21",
      report_date: "2026-03-21",
      source_filenames: ["Z001_21 .CSV", "Z002_21 .CSV", "Z005_21 .CSV"],
      settlement_no: 101,
      status: "NotImported",
      unreadable_reason: null,
    },
    {
      candidate_key: "key-b",
      path_date: "2026-03-21",
      report_date: "2026-03-21",
      source_filenames: ["Z001_21A.CSV", "Z002_21A.CSV", "Z005_21A.CSV"],
      settlement_no: 101,
      status: "NotImported",
      unreadable_reason: null,
    },
  ],
};
const origin: DailyReportSdOrigin = { scan, scanToken: "scan-token", candidateKey: "key-a" };
const sdList: DailyReportImportState = { status: "sd_list", scan, scanToken: "scan-token" };

describe("dailyReportImportReducer UI-07-D15 REQ-401", () => {
  it("REQ-401 / UI-07-D15 (5): scanning → sd_list → parsing → preview carries the SD origin", () => {
    const scanning = dailyReportImportReducer({ status: "idle" }, { type: "scan_started" });
    expect(scanning).toEqual({ status: "scanning" });
    const listed = dailyReportImportReducer(scanning, {
      type: "scan_succeeded",
      scan,
      scanToken: "scan-token",
    });
    expect(listed).toEqual(sdList);
    const parsing = dailyReportImportReducer(listed, {
      type: "sd_candidate_selected",
      candidateKey: "key-a",
      filenames: scan.candidates[0].source_filenames,
    });
    expect(parsing).toEqual({
      status: "parsing",
      filenames: scan.candidates[0].source_filenames,
      sdOrigin: origin,
    });
    const previewed = dailyReportImportReducer(parsing, {
      type: "parse_succeeded",
      preview,
      previewToken: "token",
    });
    expect(previewed).toMatchObject({ status: "preview", sdOrigin: origin });
    expect(dailyReportImportReducer(scanning, { type: "scan_failed" })).toEqual({ status: "idle" });
  });

  it("REQ-401 / UI-07-D15 (5): SD parse failure recovers to the same list; manual failure to idle", () => {
    const failed = dailyReportImportReducer(
      { status: "parsing", filenames: [], sdOrigin: origin },
      { type: "parse_failed", error },
    );
    expect(failed).toMatchObject({ status: "error", recoverTo: "sd_list", previousState: sdList });
    expect(dailyReportImportReducer(failed, { type: "dismiss_error" })).toEqual(sdList);

    const manual = dailyReportImportReducer(
      { status: "parsing", filenames: [] },
      { type: "parse_failed", error },
    );
    expect(manual).toMatchObject({ status: "error", recoverTo: "idle" });
    expect(dailyReportImportReducer(manual, { type: "dismiss_error" })).toEqual({ status: "idle" });
  });

  it("REQ-401 / UI-07-D15 (4) / BIZ-08-D5: copy failure keeps the SD origin and reselect returns to the list", () => {
    const importing = dailyReportImportReducer(
      { status: "preview", preview, previewToken: "token", filenames: [], sdOrigin: origin },
      { type: "confirm_import", additionalImportConfirmed: false },
    );
    const failed = dailyReportImportReducer(importing, {
      type: "import_failed",
      error,
      recoverTo: "preview",
    });
    const recovered = dailyReportImportReducer(failed, { type: "dismiss_error" });
    expect(recovered).toMatchObject({ status: "preview", previewToken: "token", sdOrigin: origin });
    expect(dailyReportImportReducer(recovered, { type: "back_to_sd_list" })).toEqual(sdList);

    const manualPreview: DailyReportImportState = {
      status: "preview",
      preview,
      previewToken: "token",
      filenames: [],
    };
    expect(dailyReportImportReducer(manualPreview, { type: "back_to_sd_list" })).toBe(
      manualPreview,
    );
    expect(dailyReportImportReducer(manualPreview, { type: "select_files", filenames: [] })).toBe(
      manualPreview,
    );
  });

  it("REQ-401 / UI-07-D15 (6): result returns to the same list with only the imported row replaced", () => {
    const resultState = dailyReportImportReducer(
      {
        status: "importing",
        preview,
        previewToken: "token",
        additionalImportConfirmed: false,
        filenames: [],
        sdOrigin: origin,
      },
      { type: "import_succeeded", result, reportDate: "2026-03-21" },
    );
    expect(resultState).toMatchObject({ status: "result", sdOrigin: origin });
    const back = dailyReportImportReducer(resultState, { type: "back_to_sd_list" });
    expect(back.status).toBe("sd_list");
    if (back.status === "sd_list") {
      expect(back.scanToken).toBe("scan-token");
      expect(back.scan.candidates.map((candidate) => candidate.status)).toEqual([
        "Imported",
        "NotImported",
      ]);
    }
    expect(dailyReportImportReducer(resultState, { type: "rollback_succeeded" })).toEqual({
      status: "idle",
    });
    const manualResult: DailyReportImportState = {
      status: "result",
      result,
      reportDate: "2026-03-21",
      filenames: [],
    };
    expect(dailyReportImportReducer(manualResult, { type: "back_to_sd_list" })).toBe(manualResult);
  });

  it("REQ-401 / UI-07-D17（closure round 2 #1）: bulk_succeeded folds the response into the same scan, and the next bulk accumulates", () => {
    const withEj: DailyReportSdScan = {
      ...scan,
      ej: { new_count: 4, held_count: 0, conflict_count: 0, unreadable_file_count: 1 },
    };
    const bulk = (from: DailyReportImportState, items: RegisterSdBulkResponse["items"]) =>
      dailyReportImportReducer(dailyReportImportReducer(from, { type: "bulk_started" }), {
        type: "bulk_succeeded",
        response: { items },
      });
    const daily = (key: string): RegisterSdBulkResponse["items"][number]["source"] => ({
      type: "DailyReport",
      candidate_key: key,
      report_date: "2026-03-21",
    });
    const imported = { ...result, type: "ImportedDailyReport" as const };
    const failed = { type: "Failed" as const, kind: "internal" as const, message: "合成の失敗" };

    // 1 回目: key-a だけが入り、EJ は止まった → EJ の数は scan の時のまま（何も入っていない）
    const first = bulk({ status: "sd_list", scan: withEj, scanToken: "scan-token" }, [
      { source: daily("key-a"), outcome: imported },
      { source: daily("key-b"), outcome: failed },
      { source: { type: "Ej" }, outcome: { type: "NotAttempted" } },
    ]);
    if (first.status !== "bulk_result") throw new Error("not bulk_result");
    expect(first.scanToken).toBe("scan-token");
    expect(first.scan.candidates.map((candidate) => candidate.status)).toEqual([
      "Imported",
      "NotImported",
    ]);
    expect(first.scan.ej).toEqual(withEj.ej);

    // 2 回目: key-a は「取込み済み」で飛ばされ、key-b と EJ が入る → key-a の印は残り、EJ は結果の数（新しい控えは 0）
    const second = bulk(first, [
      { source: daily("key-a"), outcome: { type: "Skipped", message: "取込み済みです" } },
      { source: daily("key-b"), outcome: imported },
      {
        source: { type: "Ej" },
        outcome: {
          type: "ImportedEj",
          inserted_count: 4,
          already_count: 0,
          conflict_count: 2,
          conflicts: [],
          held_count: 1,
          unreadable_files: ["EJ260320.TXT", "EJ260321.TXT"],
          gaps: [],
        },
      },
    ]);
    if (second.status !== "bulk_result") throw new Error("not bulk_result");
    expect(second.scan.candidates.map((candidate) => candidate.status)).toEqual([
      "Imported",
      "Imported",
    ]);
    expect(second.scan.ej).toEqual({
      new_count: 0,
      held_count: 1,
      conflict_count: 2,
      unreadable_file_count: 2,
    });
    // 個別の preview へは畳んだ scan のまま進む
    expect(
      dailyReportImportReducer(second, {
        type: "sd_candidate_selected",
        candidateKey: "key-b",
        filenames: [],
      }),
    ).toMatchObject({ status: "parsing", sdOrigin: { scan: second.scan } });
  });
});
