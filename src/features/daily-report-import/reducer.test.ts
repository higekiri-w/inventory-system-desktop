import { describe, expect, it } from "vitest";
import type {
  DailyReportImportResult,
  DailyReportPreviewData,
  DailyReportSdScan,
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
  candidates: [
    {
      candidate_key: "key-a",
      path_date: "2026-03-21",
      report_date: "2026-03-21",
      source_filenames: ["Z001_21 .CSV", "Z002_21 .CSV", "Z005_21 .CSV"],
      status: "NotImported",
    },
    {
      candidate_key: "key-b",
      path_date: "2026-03-21",
      report_date: "2026-03-21",
      source_filenames: ["Z001_21A.CSV", "Z002_21A.CSV", "Z005_21A.CSV"],
      status: "NotImported",
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
});
