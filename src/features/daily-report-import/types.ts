import type {
  DailyReportImportResult,
  DailyReportPreviewData,
  DailyReportSdScan,
  RegisterSdBulkResponse,
} from "@/lib/bindings";
import { z } from "zod";
import type { InvokeError } from "@/lib/invoke";

/// `/csv-import/daily-reports`（日報の取込みの記録）の search。`page` は 1 以上の整数で、欠落・不正は 1 ページ目。
/// route はここから import するだけ（UI_TECH_STACK UI-STATE-D2）。設計: 55-ui-csv-import.md §55.12 UI-07-D19
export const dailyReportImportRecordsSearchSchema = z.object({
  page: z.coerce.number().int().positive().optional().catch(undefined),
});

export type DailyReportImportRecordsSearch = z.output<typeof dailyReportImportRecordsSearchSchema>;

export type DailyReportErrorRecoverTo = "idle" | "preview" | "sd_list";

/**
 * 画面が持つ scan。scan の応答の `plu_snapshot_skipped_older`（45 §45.6a 手順 2a）を同じ名前で畳む。
 * 一覧へ戻っても同じ scan の知らせは残り、読み直すと新しい応答の値になる（UI-07-D15 (8)）
 */
export type DailyReportSdScanView = DailyReportSdScan & { plu_snapshot_skipped_older: boolean };

/** SD から来たこと（UI-07-D15 (5)）。手で選んだ経路の状態は持たない */
export interface DailyReportSdOrigin {
  scan: DailyReportSdScanView;
  scanToken: string;
  /** 「確認する」を押した行の candidate_key */
  candidateKey: string;
}

export type DailyReportImportState =
  | { status: "idle" }
  | { status: "scanning" }
  | { status: "sd_list"; scan: DailyReportSdScanView; scanToken: string }
  /** 55 UI-07-D17: 「まとめて取り込む」の応答を待つ間 */
  | { status: "bulk_importing"; scan: DailyReportSdScanView; scanToken: string }
  /** 55 UI-07-D17: 一括の結果。scan は応答を畳んだ表示で、残りがあれば同じ scan_token でもう一度押せる（BIZ-08-D7 のやり直し） */
  | {
      status: "bulk_result";
      scan: DailyReportSdScanView;
      scanToken: string;
      response: RegisterSdBulkResponse;
    }
  | { status: "parsing"; filenames: string[]; sdOrigin?: DailyReportSdOrigin }
  | {
      status: "preview";
      preview: DailyReportPreviewData;
      previewToken: string;
      filenames: string[];
      sdOrigin?: DailyReportSdOrigin;
    }
  | {
      status: "importing";
      preview: DailyReportPreviewData;
      previewToken: string;
      additionalImportConfirmed: boolean;
      filenames: string[];
      sdOrigin?: DailyReportSdOrigin;
    }
  | {
      status: "result";
      result: DailyReportImportResult;
      reportDate: string;
      filenames: string[];
      sdOrigin?: DailyReportSdOrigin;
    }
  | {
      status: "error";
      error: InvokeError;
      recoverTo: DailyReportErrorRecoverTo;
      previousState: DailyReportImportState;
    };

export type DailyReportImportAction =
  | { type: "scan_started" }
  | { type: "scan_succeeded"; scan: DailyReportSdScanView; scanToken: string }
  | { type: "scan_failed" }
  | { type: "sd_candidate_selected"; candidateKey: string; filenames: string[] }
  | { type: "bulk_started" }
  | { type: "bulk_succeeded"; response: RegisterSdBulkResponse }
  | { type: "bulk_failed"; error: InvokeError }
  | { type: "back_to_sd_list" }
  | { type: "select_files"; filenames: string[] }
  | { type: "parse_succeeded"; preview: DailyReportPreviewData; previewToken: string }
  | { type: "parse_failed"; error: InvokeError }
  | { type: "confirm_import"; additionalImportConfirmed: boolean }
  | { type: "import_succeeded"; result: DailyReportImportResult; reportDate: string }
  | { type: "import_failed"; error: InvokeError; recoverTo: DailyReportErrorRecoverTo }
  | { type: "dismiss_error" }
  | { type: "rollback_succeeded" }
  | { type: "reset" };
