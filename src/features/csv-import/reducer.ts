// src/features/csv-import/reducer.ts
//
// 6 variant × 9 action の純関数 reducer。副作用ゼロ。unit test 実装済み。
// 設計: docs/function-design/55-ui-csv-import.md §55.2 reducer 遷移表 + §55.8 状態遷移図

import type { CsvImportAction, CsvImportState } from "./types";

/// CSV 取込みフローの純関数 reducer。
/// invalid 遷移は state 据え置き (§55.2)。
export function csvImportReducer(state: CsvImportState, action: CsvImportAction): CsvImportState {
  // 任意 state からの reset (例外フロー、navigation 抜け道用、§55.2 reducer 遷移表)
  if (action.type === "reset") {
    return { status: "idle" };
  }

  switch (state.status) {
    case "idle":
      if (action.type === "select_file") {
        return { status: "parsing", filename: action.filename };
      }
      return state;

    case "parsing":
      if (action.type === "parse_succeeded") {
        return {
          status: "preview",
          preview: action.preview,
          previewToken: action.previewToken,
          filename: state.filename,
        };
      }
      if (action.type === "parse_failed") {
        // parsing 失敗は無条件で idle 復帰 (filename / token も破棄)
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
        // preview snapshot を importing state に持ち越す。import_failed(recoverTo="preview")
        // 時の復帰経路で preview variant を再構築するために必要 (§55.2 previousState 保持の根拠)
        return {
          status: "importing",
          preview: state.preview,
          previewToken: state.previewToken,
          filename: state.filename,
        };
      }
      if (action.type === "select_file") {
        // 「ファイルを選び直す」CTA。旧 preview を破棄して再 parse
        return { status: "parsing", filename: action.filename };
      }
      return state;

    case "importing":
      if (action.type === "import_succeeded") {
        return {
          status: "result",
          result: action.result,
          settlementDate: action.settlementDate,
          filename: state.filename,
        };
      }
      if (action.type === "import_failed") {
        // recoverTo は呼び出し側 (useCsvImportFlow) が kind から決定して action に詰める。
        // recoverTo === "preview" の時は preview variant を再構築して previousState に詰める
        // (importing variant の preview snapshot から復元、§55.2 dismiss_error 経路の型安全要件)
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
                  filename: state.filename,
                }
              : state,
        };
      }
      return state;

    case "result":
      if (action.type === "rollback_succeeded") {
        return { status: "idle" };
      }
      return state;

    case "error":
      if (action.type === "dismiss_error") {
        // recoverTo === "preview" + 元 state が preview variant の場合のみ復元 (型安全)。
        // それ以外は idle にフォールバック (§55.2)。
        if (state.recoverTo === "preview" && state.previousState.status === "preview") {
          return state.previousState;
        }
        return { status: "idle" };
      }
      return state;
  }
}
