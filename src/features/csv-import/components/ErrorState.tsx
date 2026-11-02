// src/features/csv-import/components/ErrorState.tsx
//
// CsvImportState.error variant の表示。CmdError.kind 別固定文言 + recoverTo に応じた復帰ボタンラベル。
// 設計: docs/function-design/55-ui-csv-import.md §55.5 CmdError kind 別表示マトリクス

import { Link } from "@tanstack/react-router";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { describeError } from "@/lib/describe-error";
import { CMD_ERROR_KIND, type CmdErrorKind, type InvokeError } from "@/lib/invoke";
import type { ErrorRecoverTo } from "../types";

export interface ErrorStateProps {
  error: InvokeError;
  recoverTo: ErrorRecoverTo;
  onDismiss: () => void;
}

/// kind 別固定文言 (§55.5):
/// - import_error → 「プレビューが利用できません」
/// - validation → 「入力に問題があります」
/// - その他 (internal / not_found 等) → 「エラーが発生しました」
function titleForKind(kind: CmdErrorKind): string {
  switch (kind) {
    case CMD_ERROR_KIND.IMPORT_ERROR:
      return "プレビューが利用できません";
    case CMD_ERROR_KIND.VALIDATION:
      return "入力に問題があります";
    default:
      return "エラーが発生しました";
  }
}

export function ErrorState({ error, recoverTo, onDismiss }: ErrorStateProps) {
  const title = titleForKind(error.cmdError.kind);
  const buttonLabel = recoverTo === "preview" ? "プレビューに戻る" : "最初に戻る";
  // 同じ精算の別の版の拒否（32 の順 4）が名指す取込み。出すかと ID は field だけで決める（文から読まない。55 §55.11）
  const relatedImportId = error.cmdError.related_csv_import_id;

  return (
    <div className="space-y-4">
      <Alert variant="destructive">
        <AlertTitle>{title}</AlertTitle>
        <AlertDescription>
          <p>{describeError(error)}</p>
          {relatedImportId != null ? (
            <p>
              <Link
                to="/csv-import/records/$importId"
                params={{ importId: String(relatedImportId) }}
                search={{ returnTo: "/csv-import?tab=z004" }}
                className="font-medium underline underline-offset-4"
              >
                取込みID {relatedImportId} の記録を見る（取り消すときはここから）
              </Link>
            </p>
          ) : null}
        </AlertDescription>
      </Alert>
      <Button onClick={onDismiss}>{buttonLabel}</Button>
    </div>
  );
}
