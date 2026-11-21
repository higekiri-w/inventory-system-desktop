import { AlertCircle, Ban, CheckCircle2, CircleDashed, ListChecks, Loader2 } from "lucide-react";
import type { ReactNode } from "react";
import { Link } from "@tanstack/react-router";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import type {
  DailyReportSdScan,
  EjImportResult,
  RegisterSdBulkItemDto,
  RegisterSdBulkResponse,
  RegisterSdEjSummary,
} from "@/lib/bindings";
import { formatMoney } from "./DailyReportRollbackConfirmDialog";

/// 55 UI-07-D17（(A) の分）: 締めの一括取込みの操作と結果。控えの画面（UI-09c）への link は (A-2) が足す。
/// 状態は色だけで示さず、日本語の label と icon を併用する

/** 押せる条件: 日報の NotImported が 1 件以上か、EJ の 4 つの数のどれかが 1 以上 */
export function canBulkImport(scan: DailyReportSdScan): boolean {
  const ej = scan.ej;
  return (
    scan.candidates.some((candidate) => candidate.status === "NotImported") ||
    (ej !== null && ej.new_count + ej.held_count + ej.conflict_count + ej.unreadable_file_count > 0)
  );
}

const HELD_TEXT = (count: number) =>
  `最後の控え ${count.toLocaleString()} 件は、レジがまだ書き足すファイルの最後のため、今回は入れていません。次に SD を読んだときにもう一度確かめます（店の操作は要りません）`;

/** 一覧の下の EJ の行（`scan.ej` が null なら出さない） */
export function SdEjSummaryLine({ summary }: { summary: RegisterSdEjSummary | null }) {
  if (summary === null) return null;
  return (
    <div className="space-y-1 text-sm" aria-label="レシートの控え（EJ）">
      <p>
        {summary.new_count > 0
          ? `レシートの控え（EJ）: 新しい控え ${summary.new_count.toLocaleString()} 件`
          : "レシートの控え（EJ）: 新しい控えはありません"}
      </p>
      {summary.unreadable_file_count > 0 && (
        <p>{`読めない EJ のファイルが ${summary.unreadable_file_count.toLocaleString()} 本あります`}</p>
      )}
      {summary.conflict_count > 0 && (
        <p>{`同じ番号で中身の違う控えが ${summary.conflict_count.toLocaleString()} 件あり、取り込みません`}</p>
      )}
      {summary.held_count > 0 && (
        <p className="text-muted-foreground">{HELD_TEXT(summary.held_count)}</p>
      )}
    </div>
  );
}

type Outcome = RegisterSdBulkItemDto["outcome"];

const OUTCOME_LABEL: Record<
  Exclude<Outcome["type"], "ImportedEj">,
  { label: string; icon: ReactNode }
> = {
  ImportedDailyReport: { label: "取り込みました", icon: <CheckCircle2 aria-hidden="true" /> },
  Skipped: { label: "取り込みませんでした", icon: <Ban aria-hidden="true" /> },
  NeedsIndividualConfirmation: {
    label: "1 件ずつ確かめてください",
    icon: <ListChecks aria-hidden="true" />,
  },
  Failed: { label: "ここで止まりました", icon: <AlertCircle aria-hidden="true" /> },
  NotAttempted: { label: "取り込んでいません", icon: <CircleDashed aria-hidden="true" /> },
};

function Label({ icon, label }: { icon: ReactNode; label: string }) {
  return (
    <span className="inline-flex items-center gap-1 font-medium [&_svg]:size-4">
      {icon}
      {label}
    </span>
  );
}

/** EJ の結果のうち欠け以外の名指し（読めない file・食い違い・保留）。件数が 0 の項目は出さない */
function EjNotes({ result }: { result: EjImportResult }) {
  const savedConflicts = result.conflicts.filter((conflict) => conflict.saved);
  const unsavedConflicts = result.conflicts.filter((conflict) => !conflict.saved);
  const more = result.conflict_count - result.conflicts.length;
  const numbers = (list: typeof result.conflicts) =>
    list.map((conflict) => conflict.record_no).join("、");
  return (
    <>
      {result.unreadable_files.map((name) => (
        <p key={name}>
          {`読めないファイル ${name}: SD を差し直してもう一度読むか、「EJ のファイルを選んで取り込む」で PC の写しを選んでください`}
        </p>
      ))}
      {savedConflicts.length > 0 && (
        <p>{`記録 ${numbers(savedConflicts)}: 前に取り込んだ控えと食い違う控えがあったため、前の控えを残しました。続くときは開発者に知らせてください`}</p>
      )}
      {unsavedConflicts.length > 0 && (
        <p>{`記録 ${numbers(unsavedConflicts)}: 同じ番号の控えが食い違っていたため、保存していません。続くときは開発者に知らせてください`}</p>
      )}
      {more > 0 && <p>{`ほか ${more.toLocaleString()} 件`}</p>}
      {/* 保留は毎日の補足の文（注意の色と icon を付けない） */}
      {result.held_count > 0 && (
        <p className="text-muted-foreground">{HELD_TEXT(result.held_count)}</p>
      )}
    </>
  );
}

function inserted(result: EjImportResult, prefix = ""): string {
  return result.inserted_count > 0
    ? `${prefix}控え ${result.inserted_count.toLocaleString()} 件を取り込みました`
    : `${prefix}新しい控えはありませんでした`;
}

function EjRow({
  result,
  manual,
  onChooseEjFiles,
}: {
  result: EjImportResult;
  manual: { result: EjImportResult | null; error: string | null; pending: boolean };
  onChooseEjFiles: () => void;
}) {
  // 欠けは保存の後の全期間の値なので、手で選んだ後はその結果の欠けが今の欠け
  const gaps = (manual.result ?? result).gaps;
  // 読めない file は一括の結果と手で選んだ結果のどちらにあっても、選び直す操作がここにしか無い
  const showChoose =
    gaps.length > 0 ||
    result.unreadable_files.length > 0 ||
    (manual.result?.unreadable_files.length ?? 0) > 0;
  return (
    <li className="space-y-1 p-3 text-sm">
      <p className="font-medium">レシートの控え（EJ）</p>
      <Label icon={<CheckCircle2 aria-hidden="true" />} label={inserted(result)} />
      {gaps.map((gap) => (
        <p key={`${gap.machine_prefix}-${gap.from_no}`}>
          {gap.from_no === gap.to_no
            ? `記録 ${gap.from_no} の控えがありません`
            : `記録 ${gap.from_no}〜${gap.to_no} の控えがありません`}
        </p>
      ))}
      <EjNotes result={result} />
      {showChoose && (
        <Button variant="outline" size="sm" onClick={onChooseEjFiles} disabled={manual.pending}>
          {manual.pending && <Loader2 className="animate-spin text-ongoing" aria-hidden="true" />}
          EJ のファイルを選んで取り込む
        </Button>
      )}
      {manual.error !== null && (
        <p className="flex items-start gap-2 font-medium text-destructive" role="alert">
          <AlertCircle className="mt-0.5 size-4 shrink-0" aria-hidden="true" />
          <span>{manual.error}</span>
        </p>
      )}
      {manual.result !== null && (
        <div className="space-y-1" role="status">
          <Label
            icon={<CheckCircle2 aria-hidden="true" />}
            label={inserted(manual.result, "選んだファイルから")}
          />
          <EjNotes result={manual.result} />
        </div>
      )}
    </li>
  );
}

function ItemRow({
  item,
  busy,
  onConfirmCandidate,
}: {
  item: RegisterSdBulkItemDto;
  busy: boolean;
  onConfirmCandidate: (candidateKey: string) => void;
}) {
  const { outcome, source } = item;
  if (outcome.type === "ImportedEj") return null;
  const status = OUTCOME_LABEL[outcome.type];
  const date = source.type === "DailyReport" ? (source.report_date ?? "日付の読めない日報") : "";
  return (
    <li className="space-y-1 p-3 text-sm">
      <div className="flex flex-wrap items-center gap-3">
        <span className="font-medium">{`日報 ${date}`}</span>
        <Label icon={status.icon} label={status.label} />
      </div>
      {outcome.type === "ImportedDailyReport" && (
        <p>{`総売上 ${formatMoney(outcome.gross_amount)} / 純売上 ${formatMoney(outcome.net_amount)}`}</p>
      )}
      {(outcome.type === "Skipped" || outcome.type === "Failed") && <p>{outcome.message}</p>}
      {outcome.type === "NeedsIndividualConfirmation" && source.type === "DailyReport" && (
        <>
          <p>
            同じ日に精算回数の分からない取込みがあるため、まとめては取り込みませんでした。「確認する」から
            1 件ずつ取り込んでください。
          </p>
          <Button
            variant="outline"
            size="sm"
            disabled={busy}
            onClick={() => {
              onConfirmCandidate(source.candidate_key);
            }}
          >
            確認する
          </Button>
        </>
      )}
    </li>
  );
}

export function SdBulkImport({
  scan,
  busy,
  importing,
  response,
  ejManual,
  onBulkImport,
  onChooseEjFiles,
  onConfirmCandidate,
}: {
  scan: DailyReportSdScan;
  /** 応答を待つ間（ボタンを押せない） */
  busy: boolean;
  /** 一括の応答を待つ間 */
  importing: boolean;
  response: RegisterSdBulkResponse | null;
  ejManual: { result: EjImportResult | null; error: string | null; pending: boolean };
  onBulkImport: () => void;
  onChooseEjFiles: () => void;
  /** 「1 件ずつ確かめてください」の行の「確認する」（保持中の scan_token で個別の preview へ） */
  onConfirmCandidate: (candidateKey: string) => void;
}) {
  const stopped = response?.items.some((item) => item.outcome.type === "Failed") ?? false;
  const importedDates =
    response?.items.flatMap((item) =>
      item.outcome.type === "ImportedDailyReport" ? [item.outcome.report_date] : [],
    ) ?? [];
  const lastImportedDate = importedDates[importedDates.length - 1];
  return (
    <div className="space-y-3" aria-label="まとめて取り込む">
      {/* 止まったときは上の文が押し直しを指すので、畳んだ scan に残りが見えなくても押せる（EJ だけの失敗等） */}
      <Button onClick={onBulkImport} disabled={busy || !(stopped || canBulkImport(scan))}>
        {importing && <Loader2 className="animate-spin text-ongoing" aria-hidden="true" />}
        {importing ? "取り込んでいます…" : "まとめて取り込む"}
      </Button>
      {response && (
        <div className="space-y-3" role="status">
          {stopped && (
            <Alert variant="destructive">
              <AlertCircle aria-hidden="true" />
              <AlertDescription>
                途中で止まりました。取り込めた分は残っています。原因を直してから、もう一度「まとめて取り込む」を押してください。
              </AlertDescription>
            </Alert>
          )}
          <ul className="divide-y rounded-md border" aria-label="まとめて取り込んだ結果">
            {response.items.map((item) =>
              item.outcome.type === "ImportedEj" ? (
                <EjRow
                  key="ej"
                  result={item.outcome}
                  manual={ejManual}
                  onChooseEjFiles={onChooseEjFiles}
                />
              ) : item.source.type === "Ej" ? (
                <li key="ej" className="space-y-1 p-3 text-sm">
                  <div className="flex flex-wrap items-center gap-3">
                    <span className="font-medium">レシートの控え（EJ）</span>
                    <Label
                      icon={OUTCOME_LABEL[item.outcome.type].icon}
                      label={OUTCOME_LABEL[item.outcome.type].label}
                    />
                  </div>
                  {item.outcome.type === "Failed" && <p>{item.outcome.message}</p>}
                </li>
              ) : (
                <ItemRow
                  key={item.source.candidate_key}
                  item={item}
                  busy={busy}
                  onConfirmCandidate={onConfirmCandidate}
                />
              ),
            )}
          </ul>
          <Button asChild variant="outline">
            <Link to="/reports/daily" search={lastImportedDate ? { date: lastImportedDate } : {}}>
              日次売上を見る
            </Link>
          </Button>
        </div>
      )}
    </div>
  );
}
