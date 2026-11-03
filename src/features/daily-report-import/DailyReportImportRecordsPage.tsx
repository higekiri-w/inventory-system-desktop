// src/features/daily-report-import/DailyReportImportRecordsPage.tsx
//
// UI-07-D19「日報の取込みの記録」: 取込みの結果の画面を離れた後に、取込みを行から取り消す。
// 日報は在庫を動かさず入出庫の記録でないので、入出庫履歴（65）でなく売上データ取込みの家族に置く。
// 設計: docs/function-design/55-ui-csv-import.md §55.12 / UI-07-D14 / §55.9 / §55.7

import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { CheckCircle2, Loader2, Undo2 } from "lucide-react";
import { PageHeader } from "@/components/patterns/PageHeader";
import { PageShell } from "@/components/patterns/PageShell";
import { Pagination } from "@/components/patterns/Pagination";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { commands, type DailyReportImportRow } from "@/lib/bindings";
import { formatDateTime } from "@/lib/date-time";
import { describeError } from "@/lib/describe-error";
import { unwrapResult } from "@/lib/invoke";
import { queryKeys } from "@/lib/query-keys";
import {
  DailyReportRollbackConfirmDialog,
  formatMoney,
} from "./components/DailyReportRollbackConfirmDialog";
import { useRollbackDailyReportImport } from "./hooks/useRollbackDailyReportImport";

// ponytail: 新しい順のページ送りだけで、対象日の絞込みは持たない（O5）。
// 1 ページ目に目当ての日が無いという所感が出たら、search に対象日を足し CMD の date_from・date_to を渡す。
const DAILY_REPORT_IMPORTS_PER_PAGE = 20;

export interface DailyReportImportRecordsPageProps {
  /** search `page`（route が 1 以上に正規化して渡す） */
  page: number;
  onPageChange: (page: number) => void;
}

export function DailyReportImportRecordsPage({
  page,
  onPageChange,
}: DailyReportImportRecordsPageProps) {
  const listQuery = useQuery({
    queryKey: queryKeys.dailyReportImports(page, DAILY_REPORT_IMPORTS_PER_PAGE),
    queryFn: () =>
      unwrapResult(
        commands.listDailyReportImports(page, DAILY_REPORT_IMPORTS_PER_PAGE, null, null),
        { source: "commands", cmd: "list_daily_report_imports" },
      ),
    retry: 0,
  });
  // 成功の後の再取得（D-052 C10）で行が「取消済み」に替わる。失敗なら行と ID はそのまま（§55.9）
  const rollback = useRollbackDailyReportImport();
  const pendingId = rollback.isPending ? rollback.variables : undefined;

  return (
    <PageShell>
      <PageHeader
        title="日報の取込みの記録"
        description="誤って取り込んだ日報の取込みを取り消せます。取り消しても在庫数は変わりません。取り込み直すときは、売上データ取込みの日報取込みで読み直してください。"
        actions={
          <Button asChild variant="outline">
            <Link to="/csv-import">売上データ取込みへ戻る</Link>
          </Button>
        }
      />

      {listQuery.isPending ? (
        <p role="status" className="flex items-center gap-2 text-sm">
          <Loader2 className="size-4 animate-spin text-ongoing" aria-hidden="true" />
          日報の取込みの記録を読み込んでいます…
        </p>
      ) : listQuery.isError ? (
        <Alert variant="destructive">
          <AlertTitle>日報の取込みの記録を読み込めませんでした</AlertTitle>
          <AlertDescription className="space-y-3">
            <p>{describeError(listQuery.error)}</p>
            <Button
              variant="outline"
              disabled={listQuery.isFetching}
              onClick={() => {
                void listQuery.refetch();
              }}
            >
              再試行
            </Button>
          </AlertDescription>
        </Alert>
      ) : listQuery.data.total_count === 0 ? (
        <p className="text-sm">日報の取込みの記録はまだありません。</p>
      ) : listQuery.data.items.length === 0 ? (
        // search の page が最後のページを越えた。記録が無いと読ませない
        <div className="space-y-3">
          <p className="text-sm">このページには記録がありません。</p>
          <Button
            variant="outline"
            onClick={() => {
              onPageChange(1);
            }}
          >
            1 ページ目を開く
          </Button>
        </div>
      ) : (
        <div className="space-y-3">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>対象日</TableHead>
                <TableHead>精算回数</TableHead>
                <TableHead>取込みID</TableHead>
                <TableHead>ファイル名</TableHead>
                <TableHead className="text-right">総売上</TableHead>
                <TableHead className="text-right">純売上</TableHead>
                <TableHead>取込み日時</TableHead>
                <TableHead>状態</TableHead>
                <TableHead className="text-right">操作</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {listQuery.data.items.map((row) => (
                <DailyReportImportRecordRow
                  key={row.id}
                  row={row}
                  isRollingBack={pendingId === row.id}
                  disabled={rollback.isPending}
                  onRollback={() => {
                    // 行の取込みID そのものを渡す（日付から ID を推し量らない。UI-07-D14）
                    rollback.mutate(row.id);
                  }}
                />
              ))}
            </TableBody>
          </Table>
          <Pagination
            page={listQuery.data.page}
            perPage={listQuery.data.per_page}
            totalCount={listQuery.data.total_count}
            onPageChange={onPageChange}
          />
        </div>
      )}
    </PageShell>
  );
}

function DailyReportImportRecordRow({
  row,
  isRollingBack,
  disabled,
  onRollback,
}: {
  row: DailyReportImportRow;
  isRollingBack: boolean;
  disabled: boolean;
  onRollback: () => void;
}) {
  const completed = row.status === "completed";
  return (
    <TableRow>
      <TableCell>{row.report_date}</TableCell>
      {/* 数をそのまま（桁区切りなし・先頭の 0 なし）。null は「記録なし」（TD-590） */}
      <TableCell>{row.settlement_no === null ? "記録なし" : String(row.settlement_no)}</TableCell>
      <TableCell className="tabular-nums">{row.id}</TableCell>
      <TableCell className="whitespace-normal">
        <ul className="space-y-0.5">
          {row.source_filenames.map((name) => (
            <li key={name} className="break-words">
              {name}
            </li>
          ))}
        </ul>
      </TableCell>
      <TableCell className="text-right tabular-nums">{formatMoney(row.gross_amount)}</TableCell>
      <TableCell className="text-right tabular-nums">{formatMoney(row.net_amount)}</TableCell>
      <TableCell className="font-mono tabular-nums">{formatDateTime(row.imported_at)}</TableCell>
      <TableCell>
        {/* 状態は色だけで示さず、label と icon を併用する */}
        {completed ? (
          <Badge variant="outline" tone="success">
            <CheckCircle2 aria-hidden="true" />
            取込み済み
          </Badge>
        ) : (
          <Badge variant="outline">
            <Undo2 aria-hidden="true" />
            取消済み
          </Badge>
        )}
      </TableCell>
      <TableCell className="text-right">
        {completed && (
          <DailyReportRollbackConfirmDialog
            id={row.id}
            reportDate={row.report_date}
            filenames={row.source_filenames}
            grossAmount={row.gross_amount}
            netAmount={row.net_amount}
            onConfirm={onRollback}
            isRollingBack={isRollingBack}
            disabled={disabled}
            size="sm"
          />
        )}
      </TableCell>
    </TableRow>
  );
}
