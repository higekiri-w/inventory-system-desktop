// src/features/csv-import/components/ResultStep.tsx
//
// Step 3/3 完了: 4 サマリ + status badge + 「売上レポートを見る」(/reports/daily 遷移、UI-09a 着手済) / 「取り消す」(rollback) / 「ホームに戻る」(navigate)。
// 「取り消す」の確認は CSV取込みの詳細と同じ部品（55 UI-07-D18）。
// 設計: docs/function-design/55-ui-csv-import.md §55.1 / §55.4 step 15-19 / §55.6 rollback spinner / §55.11

import { Link, useNavigate } from "@tanstack/react-router";
import { AlertTriangle, CheckCircle2 } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { RecheckTargetList } from "@/features/stocktake/components/RecheckTargetList";
import type { CountRecoveryTarget, ImportResult, RollbackResult } from "@/lib/bindings";
import { RollbackConfirmDialog } from "./RollbackConfirmDialog";

/** 55: 要再確認の一覧の各行から 73 の「今の数を入れる」の入口（棚卸しの画面）へ進む */
function RecheckTargets({ targets }: { targets: readonly CountRecoveryTarget[] }) {
  return (
    <RecheckTargetList
      title="数え直しが必要な商品"
      targets={targets}
      renderAction={(target) => (
        <Button variant="outline" size="sm" asChild>
          <Link to="/stocktake" search={{ product: target.product_code }}>
            棚卸しの画面で入れ直す
          </Link>
        </Button>
      )}
    />
  );
}

/** 55 の「取消の結果」: 取消の完了と、要再確認になった商品の一覧（補償や保留の step は無い） */
export function RollbackResultCard({ result }: { result: RollbackResult }) {
  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <CheckCircle2 aria-hidden="true" className="size-5" />
            取込みを取り消しました
          </CardTitle>
        </CardHeader>
        <CardContent>
          <dl className="grid grid-cols-2 gap-x-4 gap-y-2 text-sm">
            <dt className="text-muted-foreground">取り消した売上</dt>
            <dd className="font-medium">{result.voided_sale_count.toLocaleString()} 件</dd>
            <dt className="text-muted-foreground">取り消した在庫の変動</dt>
            <dd className="font-medium">{result.voided_movement_count.toLocaleString()} 件</dd>
          </dl>
        </CardContent>
      </Card>
      <RecheckTargets targets={result.recheck_targets} />
    </div>
  );
}

export interface ResultStepProps {
  result: ImportResult;
  settlementDate: string;
  filename: string;
  onRollback: () => void;
  isRollingBack: boolean;
}

export function ResultStep({
  result,
  settlementDate,
  filename,
  onRollback,
  isRollingBack,
}: ResultStepProps) {
  const navigate = useNavigate();
  // 部分成功は確認が残る注意、全件成功は完了報告。
  const isPartial = result.status === "completed_partial";

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <CardTitle>取込み完了</CardTitle>
          <Badge variant="outline" tone={isPartial ? "warning" : "success"}>
            {isPartial ? <AlertTriangle aria-hidden="true" /> : <CheckCircle2 aria-hidden="true" />}
            {isPartial ? "部分成功" : "成功"}
          </Badge>
        </CardHeader>
        <CardContent>
          <dl className="grid grid-cols-2 gap-x-4 gap-y-2 text-sm">
            <dt className="text-muted-foreground">取込み ID</dt>
            <dd className="font-medium">{result.csv_import_id}</dd>
            <dt className="text-muted-foreground">精算日</dt>
            <dd className="font-medium">{settlementDate}</dd>
            <dt className="text-muted-foreground">取込み件数</dt>
            <dd className="font-medium">{result.total_items.toLocaleString()} 件</dd>
            <dt className="text-muted-foreground">合計金額</dt>
            <dd className="font-medium">¥{result.total_amount.toLocaleString()}</dd>
            <dt className="text-muted-foreground">スキップ件数</dt>
            <dd className="font-medium">{result.skipped_count.toLocaleString()} 件</dd>
          </dl>
        </CardContent>
      </Card>

      <RecheckTargets targets={result.recheck_targets} />

      <div className="flex flex-wrap gap-2">
        {/* UI-09a 着手済 (Phase 2 8-3、PR #65 Round 1 P1 fix): settlementDate を
            URL state で渡すことで、当日 !== settlementDate のケース (例: 前日分 CSV を
            翌日取り込み) でも取込み済データを直接表示。invalidation は useCsvImportFlow
            の commit/rollback success で queryKeys.dailySalesRoot() prefix 実施済。 */}
        <Button
          variant="outline"
          onClick={() => {
            void navigate({ to: "/reports/daily", search: { date: settlementDate } });
          }}
        >
          売上レポートを見る
        </Button>

        <RollbackConfirmDialog
          csvImportId={result.csv_import_id}
          settlementDate={settlementDate}
          filename={filename}
          totalItems={result.total_items}
          totalAmount={result.total_amount}
          triggerLabel="取り消す"
          onConfirm={onRollback}
          isRollingBack={isRollingBack}
        />

        <Button
          onClick={() => {
            void navigate({ to: "/" });
          }}
        >
          ホームに戻る
        </Button>
      </div>
    </div>
  );
}
