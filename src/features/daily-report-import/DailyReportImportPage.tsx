import {
  AlertCircle,
  CheckCircle2,
  CircleDashed,
  CopyCheck,
  Download,
  FileX2,
  Loader2,
  TriangleAlertIcon,
} from "lucide-react";
import type { ReactNode } from "react";
import { useEffect, useState } from "react";
import { Link } from "@tanstack/react-router";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type {
  DailyReportImportResult,
  DailyReportPreviewData,
  DailyReportSdCandidate,
  DailyReportSdCandidateStatus,
  DailyReportSdScan,
} from "@/lib/bindings";
import { describeError } from "@/lib/describe-error";
import {
  DAILY_REPORT_SERIES,
  useDailyReportImportFlow,
  type DailyReportManualFile,
  type DailyReportPendingReplacement,
  type DailyReportSeries,
} from "./hooks/useDailyReportImportFlow";
import { AdditionalImportConfirmDialog } from "../csv-import/components/AdditionalImportConfirmDialog";

export function DailyReportImportPage({
  onBlockingChange,
}: {
  /** UI-07-D15 (6)・55 §55.7: 取込み・取消の応答を待つ間を親（CsvImportPage）へ上げ、タブの切替えも止める */
  onBlockingChange?: (isBlocking: boolean) => void;
} = {}) {
  const flow = useDailyReportImportFlow();
  const { state, isBlocking } = flow;
  useEffect(() => {
    onBlockingChange?.(isBlocking);
  }, [isBlocking, onBlockingChange]);

  const content = (() => {
    switch (state.status) {
      case "idle":
      case "scanning":
      case "sd_list":
        return (
          <DailyReportSourceStep
            scanning={state.status === "scanning"}
            scan={state.status === "sd_list" ? state.scan : null}
            scanError={flow.scanError}
            manualFiles={flow.manualFiles}
            pendingReplacement={flow.pendingReplacement}
            selectionError={state.lastSelectionError}
            onScan={() => {
              flow.scanSd();
            }}
            onChooseLocation={() => void flow.chooseSdLocation()}
            onConfirmCandidate={flow.confirmSdCandidate}
            onChooseFiles={() => void flow.chooseFiles()}
            onRemoveFile={flow.removeManualFile}
            onConfirmReplacement={flow.confirmReplacement}
            onCancelReplacement={flow.cancelReplacement}
            onConfirmFiles={flow.confirmManualFiles}
          />
        );
      case "parsing":
        return <DailyReportParsingStep />;
      case "preview":
        return (
          <DailyReportPreviewStep
            preview={state.preview}
            filenames={state.filenames}
            isImporting={flow.isImporting}
            selectionError={state.lastSelectionError}
            onConfirm={flow.confirmImport}
            onChooseFiles={
              // UI-07-D15 (4): SD から来た preview の「ファイルを選び直す」は一覧へ戻る
              state.sdOrigin ? flow.backToSdList : () => void flow.chooseFiles()
            }
          />
        );
      case "importing":
        return <DailyReportImportingStep filenames={state.filenames} />;
      case "result":
        return (
          <DailyReportResultStep
            result={state.result}
            reportDate={state.reportDate}
            filenames={state.filenames}
            isRollingBack={flow.isRollingBack}
            onRollback={() => {
              flow.rollback(state.result.daily_report_import_id);
            }}
            onBackToSdList={state.sdOrigin ? flow.backToSdList : undefined}
          />
        );
      case "error":
        return (
          <Alert variant="destructive">
            <AlertTitle>処理できませんでした</AlertTitle>
            <AlertDescription className="space-y-3">
              <p>{describeError(state.error)}</p>
              <Button variant="outline" onClick={flow.dismissError}>
                戻る
              </Button>
            </AlertDescription>
          </Alert>
        );
    }
  })();

  return content;
}

function DailyReportParsingStep() {
  return (
    <div
      className="flex min-h-52 flex-col items-center justify-center gap-3 rounded-lg border p-8"
      role="status"
      aria-live="polite"
    >
      <Loader2 className="size-8 animate-spin text-ongoing" aria-hidden="true" />
      <p className="text-sm font-medium">日報ファイルを解析中…</p>
    </div>
  );
}

/** UI-07-D15 (3): 状態は色だけで示さず、日本語の label と icon を併用する */
const CANDIDATE_STATUS: Record<
  DailyReportSdCandidateStatus,
  { label: string; icon: ReactNode; tone?: "success" | "warning" | "destructive" }
> = {
  NotImported: {
    label: "取り込めます",
    icon: <Download aria-hidden="true" />,
    tone: "success",
  },
  Imported: { label: "取込み済み", icon: <CheckCircle2 aria-hidden="true" /> },
  SameSettlementImported: {
    label: "同じ精算を取込み済み",
    icon: <CopyCheck aria-hidden="true" />,
    tone: "warning",
  },
  Incomplete: {
    label: "ファイルがそろっていません",
    icon: <CircleDashed aria-hidden="true" />,
    tone: "warning",
  },
  Unreadable: {
    label: "読めません",
    icon: <FileX2 aria-hidden="true" />,
    tone: "destructive",
  },
};

/** UI-07-D15 (3) の一覧の上の文（状態の数だけで決める） */
function scanSummary(scan: DailyReportSdScan): string | null {
  const count = (status: DailyReportSdCandidateStatus) =>
    scan.candidates.filter((candidate) => candidate.status === status).length;
  if (count("NotImported") > 0) return null;
  const unusable = count("Incomplete") + count("Unreadable");
  if (unusable > 0) {
    return `取り込める精算はありません。取り込めない精算が ${String(unusable)} 件あります（「ファイルがそろっていません」「読めません」の行）。`;
  }
  return "新しい精算はありません";
}

function DailyReportSourceStep({
  scanning,
  scan,
  scanError,
  manualFiles,
  pendingReplacement,
  selectionError,
  onScan,
  onChooseLocation,
  onConfirmCandidate,
  onChooseFiles,
  onRemoveFile,
  onConfirmReplacement,
  onCancelReplacement,
  onConfirmFiles,
}: {
  scanning: boolean;
  scan: DailyReportSdScan | null;
  scanError: string | null;
  manualFiles: DailyReportManualFile[];
  pendingReplacement: DailyReportPendingReplacement | null;
  selectionError: string | null;
  onScan: () => void;
  onChooseLocation: () => void;
  onConfirmCandidate: (candidateKey: string) => void;
  onChooseFiles: () => void;
  onRemoveFile: (series: DailyReportSeries) => void;
  onConfirmReplacement: () => void;
  onCancelReplacement: () => void;
  onConfirmFiles: () => void;
}) {
  const summary = scan ? scanSummary(scan) : null;
  const complete = DAILY_REPORT_SERIES.every((series) =>
    manualFiles.some((file) => file.series === series),
  );
  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle>レジの SD から読む</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <Button onClick={onScan} disabled={scanning}>
            {scanning && <Loader2 className="animate-spin text-ongoing" aria-hidden="true" />}
            {scanning ? "SD を読んでいます…" : "SD から読む"}
          </Button>
          <SelectionErrorMessage message={scanError} />
          {scanError !== null && (
            <Button variant="outline" onClick={onChooseLocation}>
              場所を選ぶ
            </Button>
          )}
          {scan && (
            <div className="space-y-3">
              <p className="text-sm font-medium" role="status">
                読み終わりました。いつもの取込みが済んでいれば、SD
                はレジに戻してください（次の精算に要ります）。
              </p>
              {summary && <p className="text-sm">{summary}</p>}
              {scan.skipped_unknown_count > 0 && (
                <p className="text-xs text-muted-foreground">
                  読まなかったファイル {scan.skipped_unknown_count.toLocaleString()} 件
                </p>
              )}
              {scan.candidates.length > 0 && (
                <ul className="divide-y rounded-md border" aria-label="SD の精算の一覧">
                  {scan.candidates.map((candidate) => (
                    <SdCandidateRow
                      key={candidate.candidate_key}
                      candidate={candidate}
                      onConfirm={onConfirmCandidate}
                    />
                  ))}
                </ul>
              )}
            </div>
          )}
        </CardContent>
      </Card>

      <section className="space-y-3" aria-label="ファイルを選んで取り込む">
        <Button variant="link" className="h-auto px-0" onClick={onChooseFiles}>
          ファイルを選んで取り込む
        </Button>
        <SelectionErrorMessage message={selectionError} />
        {manualFiles.length > 0 && (
          <div className="space-y-3">
            <ul className="divide-y rounded-md border" aria-label="選んだファイル">
              {DAILY_REPORT_SERIES.flatMap((series) => {
                const file = manualFiles.find((item) => item.series === series);
                if (!file) return [];
                return [
                  <li key={series} className="flex items-center justify-between gap-3 p-2 text-sm">
                    <span>
                      <span className="mr-2 font-medium">{series}</span>
                      {file.filename}
                    </span>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        onRemoveFile(series);
                      }}
                    >
                      外す
                    </Button>
                  </li>,
                ];
              })}
            </ul>
            <Button onClick={onConfirmFiles} disabled={!complete}>
              確認する
            </Button>
          </div>
        )}
        <p className="text-xs text-muted-foreground">
          SD が読めないときや過去の分は、Z001 / Z002 / Z005 を 1 つずつ選んで取り込めます。
        </p>
      </section>

      <AlertDialog
        open={pendingReplacement !== null}
        onOpenChange={(open) => {
          if (!open) onCancelReplacement();
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {pendingReplacement?.incoming.series} のファイルを置き換えますか？
            </AlertDialogTitle>
            <AlertDialogDescription>
              選んであった {pendingReplacement?.previous.filename} を外して、
              {pendingReplacement?.incoming.filename} を使います。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            {/* 1 回の操作で queue を 1 つだけ進める: キャンセル・Esc は onOpenChange だけ、置き換えるは close の要求を止める */}
            <AlertDialogCancel>キャンセル</AlertDialogCancel>
            <AlertDialogAction
              onClick={(event) => {
                event.preventDefault();
                onConfirmReplacement();
              }}
            >
              置き換える
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

function SdCandidateRow({
  candidate,
  onConfirm,
}: {
  candidate: DailyReportSdCandidate;
  onConfirm: (candidateKey: string) => void;
}) {
  const status = CANDIDATE_STATUS[candidate.status];
  return (
    <li className="flex flex-wrap items-center justify-between gap-3 p-3 text-sm">
      <div className="space-y-1">
        <p className="font-medium">{candidate.report_date ?? candidate.path_date}</p>
        <p className="text-xs text-muted-foreground">{candidate.source_filenames.join(" / ")}</p>
      </div>
      <div className="flex items-center gap-2">
        <Badge variant="outline" tone={status.tone}>
          {status.icon}
          {status.label}
        </Badge>
        {candidate.status === "NotImported" && (
          <Button
            size="sm"
            onClick={() => {
              onConfirm(candidate.candidate_key);
            }}
          >
            確認する
          </Button>
        )}
      </div>
    </li>
  );
}

function DailyReportPreviewStep({
  preview,
  filenames,
  isImporting,
  selectionError,
  onConfirm,
  onChooseFiles,
}: {
  preview: DailyReportPreviewData;
  filenames: string[];
  isImporting: boolean;
  selectionError: string | null;
  onConfirm: (additionalImportConfirmed: boolean) => void;
  onChooseFiles: () => void;
}) {
  const [dialogOpen, setDialogOpen] = useState(false);
  const requiresAdditionalConfirm =
    preview.duplicate_check.status === "AdditionalImportConfirmationRequired";
  const alreadyImported = preview.duplicate_check.status === "AlreadyImported";

  return (
    <div className="space-y-4">
      {alreadyImported && (
        <Alert variant="destructive">
          <AlertTitle>この日報は取込み済みです。二重取込みはできません。</AlertTitle>
          <AlertDescription>別の日報ファイルを選び直してください。</AlertDescription>
        </Alert>
      )}

      {requiresAdditionalConfirm && (
        <Alert variant="warning">
          <TriangleAlertIcon aria-hidden="true" />
          <AlertTitle>同じ日の取込みがあります</AlertTitle>
          <AlertDescription className="text-warning-strong">
            既存分を残したまま今回分を追加します。内容を確認してください。
          </AlertDescription>
        </Alert>
      )}

      <Card>
        <CardHeader className="flex flex-row items-center justify-between gap-4">
          <CardTitle>取込み内容</CardTitle>
          <Badge
            variant="outline"
            tone={
              alreadyImported ? "destructive" : requiresAdditionalConfirm ? "warning" : "success"
            }
          >
            {/* DSR-03: 同日追加確認の主情報は上部 Alert 帯が担うため、Badge は補助的な
                状態表示に留める（gated Amendment 4、PreviewStep.tsx と対称の改名。
                本画面の Alert は元々上部専用スロットに配置済みのため移動は不要）。
                gated Amendment 5（owner L3-lite round 3 裁定③）: 黒枠（既定 outline）は
                補助状態を主警告より強く見せるため、soft の tone を使う。取込み済みは上部 Alert と
                同じ危険・失敗、同日追加確認は注意・確認（D-094、L3 round 1）。 */}
            {alreadyImported ? (
              <>
                <TriangleAlertIcon aria-hidden="true" />
                取込み済み
              </>
            ) : requiresAdditionalConfirm ? (
              <>
                <TriangleAlertIcon aria-hidden="true" />
                同日データあり
              </>
            ) : (
              <>
                <CheckCircle2 aria-hidden="true" />
                確認済み
              </>
            )}
          </Badge>
        </CardHeader>
        <CardContent className="grid gap-3 text-sm md:grid-cols-2">
          <Info label="対象日" value={preview.file_info.report_date} />
          <Info label="bundle hash" value={`${preview.file_info.bundle_hash.slice(0, 8)}...`} />
          <Info label="総売上" value={formatMoney(preview.totals.gross_amount)} />
          <Info label="純売上" value={formatMoney(preview.totals.net_amount)} />
          <div className="md:col-span-2">
            <p className="text-muted-foreground">ファイル</p>
            <p className="font-medium">{filenames.join(" / ")}</p>
          </div>
        </CardContent>
      </Card>

      {preview.warnings.length > 0 && (
        <Alert>
          <AlertTitle>確認が必要な部門があります</AlertTitle>
          <AlertDescription>
            <ul className="ml-4 list-disc">
              {preview.warnings.map((warning) => (
                <li key={`${warning.code}-${warning.message}`}>{warning.message}</li>
              ))}
            </ul>
          </AlertDescription>
        </Alert>
      )}

      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>支払集計</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-sm">
            {preview.payment_summary.map((line) => (
              <Info key={line.sort_order} label={line.label} value={formatMoney(line.amount)} />
            ))}
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>部門別集計</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-sm">
            {preview.department_summary.map((line) => (
              <Info
                key={`${String(line.sort_order)}-${line.raw_department_name}`}
                label={line.raw_department_name}
                value={formatMoney(line.amount)}
              />
            ))}
          </CardContent>
        </Card>
      </div>

      <div className="flex flex-wrap gap-2">
        <Button
          onClick={() => {
            if (requiresAdditionalConfirm) {
              setDialogOpen(true);
            } else {
              onConfirm(false);
            }
          }}
          disabled={isImporting || alreadyImported}
        >
          取り込む
        </Button>
        <Button variant="outline" onClick={onChooseFiles} disabled={isImporting}>
          ファイルを選び直す
        </Button>
      </div>
      <SelectionErrorMessage message={selectionError} />
      <AdditionalImportConfirmDialog
        open={dialogOpen}
        existingImports={preview.duplicate_check.same_date_imports.map((item) => ({
          id: item.id,
          filenames: item.source_filenames,
          amount: `総売上 ${formatMoney(item.gross_amount)} / 純売上 ${formatMoney(item.net_amount)}`,
          importedAt: item.imported_at,
        }))}
        incomingImport={{
          filenames,
          amount: `総売上 ${formatMoney(preview.totals.gross_amount)} / 純売上 ${formatMoney(preview.totals.net_amount)}`,
          importedAt: preview.preview_created_at,
        }}
        onConfirm={() => {
          setDialogOpen(false);
          onConfirm(true);
        }}
        onCancel={() => {
          setDialogOpen(false);
        }}
      />
    </div>
  );
}

function SelectionErrorMessage({ message }: { message: string | null }) {
  if (message === null) return null;
  return (
    <p className="flex items-start gap-2 text-sm font-medium text-destructive" role="alert">
      <AlertCircle className="mt-0.5 size-4 shrink-0" aria-hidden="true" />
      <span>{message}</span>
    </p>
  );
}

function DailyReportImportingStep({ filenames }: { filenames: string[] }) {
  return (
    <div className="flex min-h-52 flex-col items-center justify-center gap-3 rounded-lg border p-8">
      <Loader2 className="size-8 animate-spin text-ongoing" aria-hidden="true" />
      <p className="text-sm font-medium">日報を取り込み中…</p>
      <p className="text-xs text-muted-foreground">{filenames.join(" / ")}</p>
    </div>
  );
}

function DailyReportResultStep({
  result,
  reportDate,
  filenames,
  isRollingBack,
  onRollback,
  onBackToSdList,
}: {
  result: DailyReportImportResult;
  reportDate: string;
  filenames: string[];
  isRollingBack: boolean;
  onRollback: () => void;
  /** UI-07-D15 (6): SD から来た結果だけ */
  onBackToSdList?: () => void;
}) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>日報取込み完了</CardTitle>
        <Badge variant="outline" tone="success">
          <CheckCircle2 aria-hidden="true" />
          成功
        </Badge>
      </CardHeader>
      <CardContent className="space-y-4">
        <dl className="grid grid-cols-2 gap-x-4 gap-y-2 text-sm">
          <Info label="取込み ID" value={String(result.daily_report_import_id)} />
          <Info label="対象日" value={reportDate} />
          <Info label="総売上" value={formatMoney(result.gross_amount)} />
          <Info label="純売上" value={formatMoney(result.net_amount)} />
          <Info label="警告" value={`${result.warning_count.toLocaleString()} 件`} />
        </dl>
        <Alert>
          <AlertTitle>在庫数は変わりません</AlertTitle>
          <AlertDescription>取消しても在庫数は変わりません。</AlertDescription>
        </Alert>
        <div className="flex flex-wrap gap-2">
          <Button asChild>
            <Link to="/reports/daily" search={{ date: reportDate }}>
              日次売上を見る
            </Link>
          </Button>
          {onBackToSdList && (
            <Button variant="outline" onClick={onBackToSdList} disabled={isRollingBack}>
              SD の一覧へ戻る
            </Button>
          )}
          <AlertDialog>
            <AlertDialogTrigger asChild>
              <Button variant="outline" disabled={isRollingBack}>
                {isRollingBack && (
                  <Loader2 className="mr-2 size-4 animate-spin" aria-hidden="true" />
                )}
                {isRollingBack ? "取り消し中…" : "取り消す"}
              </Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>日報取込みを取り消しますか？</AlertDialogTitle>
                <AlertDialogDescription>
                  この取込みだけを取り消します。同じ日の他の取込みは残ります。取消しても在庫数は変わりません。
                </AlertDialogDescription>
                {/* DSR-16: 単一レコード確認の definition list は原則適合。囲み階層が
                    AlertDialogContent と二重にならないよう、独立した box border は持たせず
                    border-t の行区切りのみにする（gated Amendment 3 最小調整）。 */}
                <dl className="grid grid-cols-2 gap-x-4 gap-y-2 border-t pt-3 text-sm">
                  <div>
                    <dt className="text-muted-foreground">取込み ID</dt>
                    <dd className="font-medium">{result.daily_report_import_id}</dd>
                  </div>
                  <div>
                    <dt className="text-muted-foreground">対象日</dt>
                    <dd className="font-medium">{reportDate}</dd>
                  </div>
                  <div className="col-span-2">
                    <dt className="text-muted-foreground">ファイル名</dt>
                    <dd className="font-medium">
                      <ul className="space-y-0.5">
                        {filenames.map((name) => (
                          <li key={name} className="break-words">
                            {name}
                          </li>
                        ))}
                      </ul>
                    </dd>
                  </div>
                  <div>
                    <dt className="text-muted-foreground">総売上</dt>
                    <dd className="font-medium">{formatMoney(result.gross_amount)}</dd>
                  </div>
                  <div>
                    <dt className="text-muted-foreground">純売上</dt>
                    <dd className="font-medium">{formatMoney(result.net_amount)}</dd>
                  </div>
                </dl>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>キャンセル</AlertDialogCancel>
                <AlertDialogAction variant="destructive" onClick={onRollback}>
                  取り消す
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        </div>
      </CardContent>
    </Card>
  );
}

function Info({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <dt className="text-muted-foreground">{label}</dt>
      <dd className="font-medium">{value}</dd>
    </div>
  );
}

function formatMoney(value: number | null) {
  return value === null ? "未取得" : `¥${value.toLocaleString()}`;
}
