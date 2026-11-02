// src/features/csv-import/reducer.test.ts
//
// csvImportReducer 純関数の unit test。
// 6 state variant × 9 action = 54 組合せ網羅 (describe.each table-driven)。
// 設計: docs/function-design/55-ui-csv-import.md §55.2 reducer 遷移表 + §55.8 状態遷移図
// Phase 1 7-7a Vitest 初期化、option A 純関数 only test の 1 file (54 ケース)

import { describe, it, expect } from "vitest";
import type { ImportResult, PreviewData } from "@/lib/bindings";
import type { InvokeError } from "@/lib/invoke";
import { csvImportReducer } from "./reducer";
import type { CsvImportAction, CsvImportState } from "./types";

/// mock data factories (state / action variant 構築用、中身は遷移検証に必要な最小限)。
const mockPreview = {} as unknown as PreviewData;
const mockResult = {} as unknown as ImportResult;
const mockError = {} as unknown as InvokeError;

/// 6 state variant の代表 mock state。dismiss_error の preview 復帰経路を検証するため
/// error variant は recoverTo="preview" + previousState=preview の組合せを採用。
const initialStates: Record<CsvImportState["status"], CsvImportState> = {
  idle: { status: "idle" },
  parsing: { status: "parsing", filename: "test.csv" },
  preview: {
    status: "preview",
    preview: mockPreview,
    previewToken: "tok-init",
    filename: "test.csv",
  },
  importing: {
    status: "importing",
    preview: mockPreview,
    previewToken: "tok-init",
    filename: "test.csv",
  },
  result: {
    status: "result",
    result: mockResult,
    settlementDate: "2026-05-17",
    filename: "test.csv",
  },
  error: {
    status: "error",
    error: mockError,
    recoverTo: "preview",
    previousState: {
      status: "preview",
      preview: mockPreview,
      previewToken: "tok-prev",
      filename: "test.csv",
    },
  },
};

/// 9 action variant の代表 mock action。
const actions: Record<CsvImportAction["type"], CsvImportAction> = {
  select_file: { type: "select_file", filename: "new.csv" },
  parse_succeeded: { type: "parse_succeeded", preview: mockPreview, previewToken: "tok-new" },
  parse_failed: { type: "parse_failed", error: mockError },
  confirm_import: { type: "confirm_import" },
  import_succeeded: { type: "import_succeeded", result: mockResult, settlementDate: "2026-05-17" },
  import_failed: { type: "import_failed", error: mockError, recoverTo: "preview" },
  dismiss_error: { type: "dismiss_error" },
  rollback_succeeded: { type: "rollback_succeeded" },
  reset: { type: "reset" },
};

/// 6 state × 9 action = 54 組合せの期待 state.status 表。
/// 設計: 55-ui-csv-import.md §55.2 reducer 遷移表
/// invalid 遷移は state 据え置き (期待 = 初期 state.status)。
/// reset は全 state から idle (action.type === "reset" を最優先処理)。
/// dismiss_error は recoverTo="preview" + previousState.status === "preview" の場合 preview 復帰、
/// それ以外は idle fallback (本 fixture では error state を上記組合せに固定したため preview 復帰経路)。
const transitionTable: [
  CsvImportState["status"],
  CsvImportAction["type"],
  CsvImportState["status"],
][] = [
  // idle (action 9 件) — select_file のみ valid、他は state 据え置き、reset は idle 継続
  ["idle", "select_file", "parsing"],
  ["idle", "parse_succeeded", "idle"],
  ["idle", "parse_failed", "idle"],
  ["idle", "confirm_import", "idle"],
  ["idle", "import_succeeded", "idle"],
  ["idle", "import_failed", "idle"],
  ["idle", "dismiss_error", "idle"],
  ["idle", "rollback_succeeded", "idle"],
  ["idle", "reset", "idle"],

  // parsing (action 9 件) — parse_succeeded → preview, parse_failed → error
  ["parsing", "select_file", "parsing"],
  ["parsing", "parse_succeeded", "preview"],
  ["parsing", "parse_failed", "error"],
  ["parsing", "confirm_import", "parsing"],
  ["parsing", "import_succeeded", "parsing"],
  ["parsing", "import_failed", "parsing"],
  ["parsing", "dismiss_error", "parsing"],
  ["parsing", "rollback_succeeded", "parsing"],
  ["parsing", "reset", "idle"],

  // preview (action 9 件) — confirm_import → importing, select_file → parsing
  ["preview", "select_file", "parsing"],
  ["preview", "parse_succeeded", "preview"],
  ["preview", "parse_failed", "preview"],
  ["preview", "confirm_import", "importing"],
  ["preview", "import_succeeded", "preview"],
  ["preview", "import_failed", "preview"],
  ["preview", "dismiss_error", "preview"],
  ["preview", "rollback_succeeded", "preview"],
  ["preview", "reset", "idle"],

  // importing (action 9 件) — import_succeeded → result, import_failed → error
  ["importing", "select_file", "importing"],
  ["importing", "parse_succeeded", "importing"],
  ["importing", "parse_failed", "importing"],
  ["importing", "confirm_import", "importing"],
  ["importing", "import_succeeded", "result"],
  ["importing", "import_failed", "error"],
  ["importing", "dismiss_error", "importing"],
  ["importing", "rollback_succeeded", "importing"],
  ["importing", "reset", "idle"],

  // result (action 9 件) — rollback_succeeded → idle
  ["result", "select_file", "result"],
  ["result", "parse_succeeded", "result"],
  ["result", "parse_failed", "result"],
  ["result", "confirm_import", "result"],
  ["result", "import_succeeded", "result"],
  ["result", "import_failed", "result"],
  ["result", "dismiss_error", "result"],
  ["result", "rollback_succeeded", "idle"],
  ["result", "reset", "idle"],

  // error (action 9 件) — dismiss_error → preview (本 fixture の error fixture が recoverTo="preview" + previousState.preview のため)
  ["error", "select_file", "error"],
  ["error", "parse_succeeded", "error"],
  ["error", "parse_failed", "error"],
  ["error", "confirm_import", "error"],
  ["error", "import_succeeded", "error"],
  ["error", "import_failed", "error"],
  ["error", "dismiss_error", "preview"],
  ["error", "rollback_succeeded", "error"],
  ["error", "reset", "idle"],
];

describe("csvImportReducer (6 state × 9 action = 54 組合せ網羅)", () => {
  describe.each(transitionTable)("%s + %s", (initialStatus, actionType, expectedStatus) => {
    it(`transitions to ${expectedStatus}`, () => {
      const state = initialStates[initialStatus];
      const action = actions[actionType];
      const next = csvImportReducer(state, action);
      expect(next.status).toBe(expectedStatus);
    });
  });
});

/// error variant の dismiss_error idle fallback 経路 (recoverTo !== "preview" or previousState.status !== "preview")。
/// 上表 fixture では preview 復帰のみカバー、別 fixture で idle fallback を 1 ケース補強。
describe("csvImportReducer dismiss_error idle fallback (recoverTo='idle')", () => {
  it("falls back to idle when recoverTo='idle'", () => {
    const errorState: CsvImportState = {
      status: "error",
      error: mockError,
      recoverTo: "idle",
      previousState: { status: "idle" },
    };
    const next = csvImportReducer(errorState, { type: "dismiss_error" });
    expect(next.status).toBe("idle");
  });
});

/// payload carry を verify する focused tests (Codex Round 1 P2-1 反映、PR #64)。
/// 上表 transitionTable は `next.status` のみ検証するため、preview / previewToken / filename /
/// previousState 等の payload が壊れても green になる回帰盲点を補強。
/// UI-07 PR #62 Round 1 で実際に被弾した「importing variant の preview snapshot carry」
/// 「import_failed の previousState 再構築」「dismiss_error の preview 復帰 payload」を直接検証。
describe("csvImportReducer focused payload carry (Round 1 回帰点)", () => {
  it("parse_succeeded: preview / previewToken / filename を preview state に正しく載せる", () => {
    const state: CsvImportState = { status: "parsing", filename: "test.csv" };
    const action: CsvImportAction = {
      type: "parse_succeeded",
      preview: mockPreview,
      previewToken: "tok-new",
    };
    const next = csvImportReducer(state, action);
    expect(next).toEqual({
      status: "preview",
      preview: mockPreview,
      previewToken: "tok-new",
      filename: "test.csv",
    });
    if (next.status === "preview") {
      expect(next.preview).toBe(mockPreview); // identity 保持の念押し
    }
  });

  it("confirm_import: preview snapshot + previewToken + filename を importing に carry する", () => {
    const state: CsvImportState = {
      status: "preview",
      preview: mockPreview,
      previewToken: "tok-init",
      filename: "test.csv",
    };
    const action: CsvImportAction = { type: "confirm_import" };
    const next = csvImportReducer(state, action);
    expect(next).toEqual({
      status: "importing",
      preview: mockPreview,
      previewToken: "tok-init",
      filename: "test.csv",
    });
    if (next.status === "importing") {
      expect(next.preview).toBe(mockPreview); // identity 保持
    }
  });

  it("import_failed(recoverTo='preview'): previousState を preview variant として再構築する (型安全 dismiss_error 経路)", () => {
    const state: CsvImportState = {
      status: "importing",
      preview: mockPreview,
      previewToken: "tok-init",
      filename: "test.csv",
    };
    const action: CsvImportAction = {
      type: "import_failed",
      error: mockError,
      recoverTo: "preview",
    };
    const next = csvImportReducer(state, action);
    expect(next).toEqual({
      status: "error",
      error: mockError,
      recoverTo: "preview",
      previousState: {
        status: "preview",
        preview: mockPreview,
        previewToken: "tok-init",
        filename: "test.csv",
      },
    });
    // importing → error 遷移時に previousState が importing 自体ではなく preview に再構築されている
    if (next.status === "error") {
      expect(next.previousState.status).toBe("preview");
    }
  });

  it("dismiss_error from error (recoverTo='preview' + previousState=preview): previousState identity をそのまま返す", () => {
    const previewState: CsvImportState = {
      status: "preview",
      preview: mockPreview,
      previewToken: "tok-prev",
      filename: "test.csv",
    };
    const state: CsvImportState = {
      status: "error",
      error: mockError,
      recoverTo: "preview",
      previousState: previewState,
    };
    const next = csvImportReducer(state, { type: "dismiss_error" });
    expect(next).toBe(previewState); // object identity で previousState がそのまま返ることを保証
  });
});
