// src/features/csv-import/types.ts
//
// UI-07 CSV取込み画面の state machine 型定義 (6 variant discriminated union + 9 action)。
// 設計: docs/function-design/55-ui-csv-import.md §55.2

import { z } from "zod";
import type { ImportResult, PreviewData } from "@/lib/bindings";
import type { InvokeError } from "@/lib/invoke";

/// `/csv-import` のタブ（初期選択だけを決める search `tab`）の値の集合。route と画面はここから導く。
/// 設計: 55-ui-csv-import.md §55.11 / UI_TECH_STACK.md UI-STATE-D2
export const CSV_IMPORT_TAB_VALUES = ["daily-report", "z004"] as const;

export type CsvImportTab = (typeof CSV_IMPORT_TAB_VALUES)[number];

/// 欠落・ほかの値は既定（日報取込み）に落とす
export const csvImportSearchSchema = z.object({
  tab: z.enum(CSV_IMPORT_TAB_VALUES).optional().catch(undefined),
});

export type CsvImportSearch = z.output<typeof csvImportSearchSchema>;

/// エラー解消時の遷移先。
/// - "idle": キャッシュ期限切れ / プレビュー消失等の import_error 系。最初に戻る
/// - "preview": validation / 一時的失敗。プレビューに戻って再 commit 可能
export type ErrorRecoverTo = "idle" | "preview";

/// 6 variant discriminated union。CMD-07 のフローに対応した直線状態機械。
/// 設計: 55-ui-csv-import.md §55.2 CsvImportState 6 variant 定義
export type CsvImportState =
  | { status: "idle" }
  | { status: "parsing"; filename: string }
  | {
      status: "preview";
      preview: PreviewData;
      previewToken: string;
      filename: string;
    }
  | {
      status: "importing";
      preview: PreviewData;
      previewToken: string;
      filename: string;
    }
  | { status: "result"; result: ImportResult; settlementDate: string; filename: string }
  | {
      status: "error";
      error: InvokeError;
      recoverTo: ErrorRecoverTo;
      previousState: CsvImportState;
    };

/// 9 variant の reducer action。`rollback_failed` は別 action にせず、
/// rollback useMutation の onError で Sonner トースト + state 据え置きで処理 (§55.2)。
export type CsvImportAction =
  | { type: "select_file"; filename: string }
  | { type: "parse_succeeded"; preview: PreviewData; previewToken: string }
  | { type: "parse_failed"; error: InvokeError }
  | { type: "confirm_import" }
  | { type: "import_succeeded"; result: ImportResult; settlementDate: string }
  | { type: "import_failed"; error: InvokeError; recoverTo: ErrorRecoverTo }
  | { type: "dismiss_error" }
  | { type: "rollback_succeeded" }
  | { type: "reset" };
