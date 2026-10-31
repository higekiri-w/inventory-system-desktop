import type {
  DailyReportImportResult,
  DailyReportPreviewData,
  DailyReportSdScan,
} from "@/lib/bindings";
import type { InvokeError } from "@/lib/invoke";

export type DailyReportErrorRecoverTo = "idle" | "preview" | "sd_list";

/** SD から来たこと（UI-07-D15 (5)）。手で選んだ経路の状態は持たない */
export interface DailyReportSdOrigin {
  scan: DailyReportSdScan;
  scanToken: string;
  /** 「確認する」を押した行の candidate_key */
  candidateKey: string;
}

export type DailyReportImportState =
  | { status: "idle" }
  | { status: "scanning" }
  | { status: "sd_list"; scan: DailyReportSdScan; scanToken: string }
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
  | { type: "scan_succeeded"; scan: DailyReportSdScan; scanToken: string }
  | { type: "scan_failed" }
  | { type: "sd_candidate_selected"; candidateKey: string; filenames: string[] }
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
