// src/features/ej-receipts/EjReceiptsPage.tsx
//
// UI-09c レシートの控え画面。EJ の記録を日付と番号で探し、控えの文を記録のまま出す（読むだけ）。
// route 結線は src/routes/reports/receipts.tsx。search + onSearchChange を props で受けて testable に保つ。
// 設計: docs/SCREEN_DESIGN.md UI-09c-D1〜D5、docs/function-design/80-ui-ej-receipts.md

import { AlertTriangle, Loader2 } from "lucide-react";
import { useEffect, useState } from "react";
import type { KeyboardEvent } from "react";
import { Link } from "@tanstack/react-router";
import { toast } from "sonner";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { PageHeader } from "@/components/patterns/PageHeader";
import { PageShell } from "@/components/patterns/PageShell";
import { normalizeComposedDigits } from "@/components/patterns/normalizeComposedDigits";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { EjNotes, inserted } from "@/features/daily-report-import/components/SdBulkImport";
import { useImportEjFiles } from "@/features/daily-report-import/hooks/useImportEjFiles";
import type { EjImportResult, EjNumberGap, EjRecordDay, EjRecordSummary } from "@/lib/bindings";
import { describeError } from "@/lib/describe-error";
import { EjDateNavigator } from "./components/EjDateNavigator";
import { useEjRecordDay, useEjRecordDetail, useEjRecordsByNo } from "./hooks/useEjRecords";
import { EJ_RECEIPT_FONT_FAMILY, ejKindLabel, ejTime, formatEjAmount } from "./lib/receipt-format";
import type { EjReceiptsSearch } from "./types";

export interface EjReceiptsPageProps {
  search: EjReceiptsSearch;
  onSearchChange: (next: EjReceiptsSearch, options?: { replace?: boolean }) => void;
}

const EMPTY_TEXT = "レシートの控えはまだありません。売上データ取込みで SD を読むと、ここに出ます。";

function gapText(gap: EjNumberGap): string {
  return gap.from_no === gap.to_no
    ? `記録 ${gap.from_no} の控えがありません。`
    : `記録 ${gap.from_no}〜${gap.to_no} の控えがありません。`;
}

function RetryAlert({
  title,
  error,
  onRetry,
}: {
  title: string;
  error: unknown;
  onRetry: () => void;
}) {
  return (
    <Alert variant="destructive">
      <AlertTitle>{title}</AlertTitle>
      <AlertDescription className="flex items-center justify-between gap-2">
        <span>{describeError(error)}</span>
        <Button type="button" variant="outline" size="sm" onClick={onRetry}>
          再試行
        </Button>
      </AlertDescription>
    </Alert>
  );
}

/** UI-09c-D5: 手で選ぶ EJ の結果（読めない file・食い違いの番号を名指す。UI-07-D17 と同じ文、TD-310） */
function ManualEjResult({
  result,
  error,
}: {
  result: EjImportResult | null;
  error: string | null;
}) {
  return (
    <>
      {error !== null && (
        <p className="text-sm font-medium text-destructive" role="alert">
          {error}
        </p>
      )}
      {result !== null && (
        <div className="space-y-1 text-sm" role="status" aria-label="選んだファイルの取込みの結果">
          <EjNotes result={result} />
        </div>
      )}
    </>
  );
}

/** UI-09c-D2: その日の控えの表。行は上下と Enter でも選べる（UI-09c の利用者配慮） */
function RecordTable({
  records,
  selectedId,
  onSelect,
}: {
  records: EjRecordSummary[];
  selectedId: number | null;
  onSelect: (id: number) => void;
}) {
  // roving tabindex: 選んだ行（一覧に無ければ先頭の行）だけを Tab で止める
  const focusableId = records.some((r) => r.id === selectedId) ? selectedId : records[0].id;
  const handleKeyDown = (event: KeyboardEvent<HTMLTableRowElement>, index: number) => {
    const step = event.key === "ArrowDown" ? 1 : event.key === "ArrowUp" ? -1 : 0;
    if (step !== 0) {
      const nextIndex = index + step;
      if (nextIndex < 0 || nextIndex >= records.length) return;
      event.preventDefault();
      onSelect(records[nextIndex].id);
      const sibling =
        step === 1
          ? event.currentTarget.nextElementSibling
          : event.currentTarget.previousElementSibling;
      if (sibling instanceof HTMLElement) sibling.focus();
    } else if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onSelect(records[index].id);
    }
  };
  return (
    <Table role="grid" aria-label="控えの一覧">
      <TableHeader>
        <TableRow>
          <TableHead>時刻</TableHead>
          <TableHead>番号</TableHead>
          <TableHead>種類</TableHead>
          <TableHead className="text-right">合計</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {records.map((record, index) => (
          <TableRow
            key={record.id}
            aria-selected={record.id === selectedId}
            data-state={record.id === selectedId ? "selected" : undefined}
            tabIndex={record.id === focusableId ? 0 : -1}
            className="cursor-pointer focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
            onClick={() => {
              onSelect(record.id);
            }}
            onKeyDown={(event) => {
              handleKeyDown(event, index);
            }}
          >
            <TableCell>{ejTime(record.printed_at)}</TableCell>
            <TableCell className="tabular-nums">{record.record_no}</TableCell>
            <TableCell>{ejKindLabel(record.kind, record.mode)}</TableCell>
            <TableCell className="text-right tabular-nums">
              {formatEjAmount(record.total_amount)}
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}

/** UI-09c-D4: 控えの文は記録のまま（行末の空白を保ち、折り返さず横に scroll）。trim・tab の展開をしない */
function ReceiptBody({ id }: { id: number }) {
  const detail = useEjRecordDetail(id);
  if (detail.isPending) return <p role="status">控えの文を読み込んでいます…</p>;
  if (detail.isError) {
    return (
      <RetryAlert
        title="控えの文を読めませんでした"
        error={detail.error}
        onRetry={() => {
          void detail.refetch();
        }}
      />
    );
  }
  const { summary, body_text } = detail.data;
  return (
    <section className="space-y-2" aria-label="控えの文">
      <h2 className="text-sm font-medium">{`記録 ${summary.record_no}（${summary.printed_at}）`}</h2>
      <pre
        className="overflow-x-auto rounded-md border bg-card p-3 text-base leading-snug"
        style={{ whiteSpace: "pre", fontFamily: EJ_RECEIPT_FONT_FAMILY }}
      >
        {body_text}
      </pre>
    </section>
  );
}

export function EjReceiptsPage({ search, onSearchChange }: EjReceiptsPageProps) {
  const dayQuery = useEjRecordDay(search.date ?? null);
  // 選んだ行は画面の状態（URL に持たない。80 §80.3）。日と組で持ち、表示中の日と同じときだけ使う
  // （サイドバー・戻る・前の表の行の click など moveTo を通らない日の移動でも外れる）
  const [selected, setSelected] = useState<{ id: number; date: string } | null>(null);
  const selectedId =
    selected !== null && dayQuery.data?.date === selected.date ? selected.id : null;
  const [numberInput, setNumberInput] = useState("");
  const [submittedNo, setSubmittedNo] = useState<string | null>(search.no ?? null);
  const byNo = useEjRecordsByNo(submittedNo);
  const ejImport = useImportEjFiles({
    onSuccess: (result) => {
      toast.success(inserted(result, "選んだファイルから"));
    },
  });

  // `?no=` で来たら（同じ画面の link からも）その番号を探す
  useEffect(() => {
    if (search.no !== undefined) setSubmittedNo(search.no);
  }, [search.no]);

  // UI-09c-D3: 1 件ならその日に移って行を選ぶ。`?no=` は search を { date, no: undefined } に replace で替える
  const fromLink = search.no !== undefined;
  // 件数は今回の検索の応答で判定する（C27 で無効にした cache・取り直し中の値で移らない）
  const numberResult =
    submittedNo !== null && byNo.isSuccess && !byNo.isFetching ? byNo.data : undefined;
  const numberError = submittedNo !== null && byNo.isError && !byNo.isFetching;
  // 手で選ぶ EJ の取込みの待ちの間は route の移動が止まるので、待ちが終わってから移る（検索を消さない）
  const importPending = ejImport.pending;
  useEffect(() => {
    if (numberResult?.length !== 1 || importPending) return;
    const [found] = numberResult;
    setSelected({ id: found.id, date: found.date });
    setSubmittedNo(null);
    onSearchChange({ date: found.date, no: undefined }, { replace: fromLink });
  }, [numberResult, onSearchChange, fromLink, importPending]);

  const moveTo = (date: string, id: number | null = null) => {
    setSelected(id === null ? null : { id, date });
    onSearchChange({ date, no: undefined });
  };

  const manualResult = <ManualEjResult result={ejImport.result} error={ejImport.error} />;

  const renderDay = (day: EjRecordDay) => {
    if (day.date === null) {
      return (
        <div className="space-y-2">
          <p>{EMPTY_TEXT}</p>
          <Link
            to="/csv-import"
            search={{ tab: "daily-report" }}
            className="font-medium text-primary underline underline-offset-4"
          >
            売上データ取込みへ
          </Link>
          {manualResult}
        </div>
      );
    }
    return (
      <>
        <div className="flex flex-wrap items-start justify-between gap-4">
          <EjDateNavigator
            date={day.date}
            prevDate={day.prev_date}
            nextDate={day.next_date}
            onChange={(date) => {
              setSubmittedNo(null);
              moveTo(date);
            }}
          />
          <div className="space-y-1">
            <Label htmlFor="ej-record-no">番号で探す</Label>
            <Input
              id="ej-record-no"
              className="w-40"
              value={numberInput}
              placeholder="例: 102"
              onChange={(event) => {
                setNumberInput(event.target.value);
              }}
              onKeyDown={(event) => {
                // IME の変換の確定の Enter は送らない。全角の数字は半角にしてから送る（BIZ は ASCII だけを受ける）
                if (event.key !== "Enter" || event.nativeEvent.isComposing) return;
                event.preventDefault();
                const no = normalizeComposedDigits(numberInput.trim());
                // 同じ番号の Enter は取り直す（一時の読取りの失敗から同じ入力でやり直せる）
                if (no === submittedNo) void byNo.refetch();
                else setSubmittedNo(no);
              }}
            />
            {numberError && (
              <p className="text-sm font-medium text-destructive" role="alert">
                {describeError(byNo.error)}
              </p>
            )}
            {numberResult?.length === 0 && (
              <p className="text-sm" role="status">
                その番号の控えはありません
              </p>
            )}
            {numberResult !== undefined && numberResult.length > 1 && (
              <div className="space-y-1 text-sm" role="status">
                <p>{`同じ番号の控えが ${String(numberResult.length)} 件あります`}</p>
                <ul className="flex flex-wrap gap-2">
                  {numberResult.map((ref) => (
                    <li key={ref.id}>
                      <Button
                        type="button"
                        variant="link"
                        size="sm"
                        className="h-auto p-0"
                        onClick={() => {
                          moveTo(ref.date, ref.id);
                        }}
                      >
                        {ref.date}
                      </Button>
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        </div>

        {day.gaps.length > 0 && (
          <Alert variant="warning">
            <AlertTriangle aria-hidden="true" />
            <AlertTitle>控えが足りない所があります</AlertTitle>
            <AlertDescription>
              {day.gaps.map((gap) => (
                <p key={`${gap.machine_prefix}-${gap.from_no}`}>{gapText(gap)}</p>
              ))}
            </AlertDescription>
          </Alert>
        )}
        {manualResult}

        {day.truncated && (
          <p className="text-sm" role="status">
            この日の控えが多いため、先頭 1000 件だけを出しています
          </p>
        )}

        <div className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
          {day.records.length === 0 ? (
            <p>この日の控えはありません</p>
          ) : (
            <RecordTable
              records={day.records}
              selectedId={selectedId}
              onSelect={(id) => {
                setSelected(day.date === null ? null : { id, date: day.date });
              }}
            />
          )}
          {selectedId !== null && <ReceiptBody id={selectedId} />}
        </div>
      </>
    );
  };

  return (
    <PageShell>
      <PageHeader
        title="レシートの控え"
        description="レジの電子ジャーナル（EJ）の記録を、レシートの控えとして見ます。"
      />

      {dayQuery.isPending ? (
        <p role="status">読み込んでいます…</p>
      ) : dayQuery.isError ? (
        <RetryAlert
          title="レシートの控えを読めませんでした"
          error={dayQuery.error}
          onRetry={() => {
            void dayQuery.refetch();
          }}
        />
      ) : (
        renderDay(dayQuery.data)
      )}

      {/* UI-09c-D5: 補助のリンク（記録の無い日・空の DB でも常に置く） */}
      <div className="space-y-2 border-t pt-3">
        {/* 日の一覧が読込み中・失敗の間も取込みは押せるので、結果と error はリンクの上に出す */}
        {!dayQuery.isSuccess && manualResult}
        <Button
          type="button"
          variant="link"
          size="sm"
          className="h-auto p-0"
          disabled={ejImport.pending}
          onClick={() => {
            void ejImport.choose();
          }}
        >
          {ejImport.pending && <Loader2 className="animate-spin text-ongoing" aria-hidden="true" />}
          EJ のファイルを選んで取り込む
        </Button>
      </div>
    </PageShell>
  );
}
