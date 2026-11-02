import type { DailyReportImportAction, DailyReportImportState, DailyReportSdOrigin } from "./types";

/** SD の一覧の状態（UI-07-D15 (4)・(5)） */
function sdList(origin: DailyReportSdOrigin): DailyReportImportState {
  return { status: "sd_list", scan: origin.scan, scanToken: origin.scanToken };
}

export function dailyReportImportReducer(
  state: DailyReportImportState,
  action: DailyReportImportAction,
): DailyReportImportState {
  if (action.type === "reset") return { status: "idle" };

  switch (state.status) {
    case "idle":
      if (action.type === "select_files") {
        return { status: "parsing", filenames: action.filenames };
      }
      if (action.type === "scan_started") return { status: "scanning" };
      return state;
    case "scanning":
      if (action.type === "scan_succeeded") {
        return { status: "sd_list", scan: action.scan, scanToken: action.scanToken };
      }
      if (action.type === "scan_failed") return { status: "idle" };
      return state;
    case "sd_list":
      if (action.type === "scan_started") return { status: "scanning" };
      if (action.type === "sd_candidate_selected") {
        return {
          status: "parsing",
          filenames: action.filenames,
          sdOrigin: {
            scan: state.scan,
            scanToken: state.scanToken,
            candidateKey: action.candidateKey,
          },
        };
      }
      return state;
    case "parsing":
      if (action.type === "parse_succeeded") {
        return {
          status: "preview",
          preview: action.preview,
          previewToken: action.previewToken,
          filenames: state.filenames,
          sdOrigin: state.sdOrigin,
        };
      }
      if (action.type === "parse_failed") {
        // UI-07-D15 (5): SD から来た parse の失敗は kind・message に依らず一覧へ戻す
        if (state.sdOrigin) {
          return {
            status: "error",
            error: action.error,
            recoverTo: "sd_list",
            previousState: sdList(state.sdOrigin),
          };
        }
        return {
          status: "error",
          error: action.error,
          recoverTo: "idle",
          previousState: state,
        };
      }
      return state;
    case "preview":
      if (action.type === "confirm_import") {
        return {
          status: "importing",
          preview: state.preview,
          previewToken: state.previewToken,
          additionalImportConfirmed: action.additionalImportConfirmed,
          filenames: state.filenames,
          sdOrigin: state.sdOrigin,
        };
      }
      if (action.type === "back_to_sd_list" && state.sdOrigin) return sdList(state.sdOrigin);
      return state;
    case "importing":
      if (action.type === "import_succeeded") {
        return {
          status: "result",
          result: action.result,
          reportDate: action.reportDate,
          filenames: state.filenames,
          sdOrigin: state.sdOrigin,
        };
      }
      if (action.type === "import_failed") {
        return {
          status: "error",
          error: action.error,
          recoverTo: action.recoverTo,
          previousState:
            action.recoverTo === "preview"
              ? {
                  status: "preview",
                  preview: state.preview,
                  previewToken: state.previewToken,
                  filenames: state.filenames,
                  sdOrigin: state.sdOrigin,
                }
              : state,
        };
      }
      return state;
    case "result":
      if (action.type === "rollback_succeeded") return { status: "idle" };
      // UI-07-D15 (6): 取り込んだ行だけを「取込み済み」に替えた同じ scan の一覧へ戻る（読み直さない）
      if (action.type === "back_to_sd_list" && state.sdOrigin) {
        const { scan, candidateKey } = state.sdOrigin;
        return sdList({
          ...state.sdOrigin,
          scan: {
            ...scan,
            candidates: scan.candidates.map((candidate) =>
              candidate.candidate_key === candidateKey
                ? { ...candidate, status: "Imported" }
                : candidate,
            ),
          },
        });
      }
      return state;
    case "error":
      if (action.type === "dismiss_error") {
        if (state.recoverTo === "preview" && state.previousState.status === "preview") {
          return state.previousState;
        }
        if (state.recoverTo === "sd_list" && state.previousState.status === "sd_list") {
          return state.previousState;
        }
        return { status: "idle" };
      }
      return state;
  }
}
