// src/features/inventory-records/RecordCancel.tsx
//
// REQ-208: 入出庫の記録の詳細からの取消の部品（取消の button と確認の dialog、状態の badge、取消済み・結果未確認の表示、
// 詳細の読取りの error）。
// 入庫・返品交換・手動販売・廃棄・振替の 5 つの詳細が、同じ部品と hook（useRecordCancel.ts）を使う。
// 設計: docs/function-design/65-inventory-record-traceability.md §65.5b（D-163）

import type { UseQueryResult } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ArrowLeft, Loader2 } from "lucide-react";
import { useId } from "react";

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
} from "@/components/ui/alert-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { PageHeader } from "@/components/patterns/PageHeader";
import { formatLengthOrPlain } from "@/features/stock-inquiry/lib/format-stock-display";
import { formatDateTime } from "@/lib/date-time";
import { describeError } from "@/lib/describe-error";
import { formatRecordStatus } from "./types";
import type { CancelableRecordDetail, RecordCancel } from "./useRecordCancel";

/** 取り消した記録の「関連する在庫変動」の空の文（無効にした movement は出さない。§65.8.2） */
export const CANCELED_MOVEMENTS_EMPTY_DESCRIPTION =
  "取り消したので、この記録は在庫を動かしていません";

const UNCONFIRMED_MESSAGE =
  "取り消せたか確かめられませんでした。「状態をもう一度確かめる」を押してください";

/** 「状態をもう一度確かめる」（確認の dialog と詳細の画面の結果未確認の知らせが、同じ操作を出す） */
function RecheckButton({ cancel }: { cancel: RecordCancel }) {
  return (
    <Button type="button" variant="outline" disabled={cancel.isChecking} onClick={cancel.recheck}>
      {cancel.isChecking ? <Loader2 className="size-4 animate-spin" aria-hidden="true" /> : null}
      状態をもう一度確かめる
    </Button>
  );
}

/** 詳細の「状態」の badge。取消が済んだかを確かめられていない間は、cache の active を「有効」と言い切らない */
export function RecordStatusBadge({ cancel, status }: { cancel: RecordCancel; status: string }) {
  return (
    <Badge variant="outline">
      {cancel.isUnconfirmed && status === "active" ? "結果未確認" : formatRecordStatus(status)}
    </Badge>
  );
}

export interface RecordCancelButtonProps {
  cancel: RecordCancel;
  status: string;
  /** その記録の業務日付（入庫日・返品日・販売日・廃棄日・振替の日付）。入れ直しの文が名指す */
  businessDate: string;
}

/** 有効な記録の header の操作「この記録を取り消す」と、確認の dialog。取消済みには出さない */
export function RecordCancelButton({ cancel, status, businessDate }: RecordCancelButtonProps) {
  const reasonId = useId();
  // 取消済みと確かめた後は、詳細の cache の状態によらず出さない（確かめた結果は詳細の読取りと別に持つ）
  if (status !== "active" || cancel.isCanceled) return null;
  return (
    <>
      <Button
        type="button"
        variant="outline"
        className="ml-2"
        onClick={() => {
          cancel.setOpen(true);
        }}
      >
        この記録を取り消す
      </Button>
      <AlertDialog open={cancel.open} onOpenChange={cancel.setOpen}>
        <AlertDialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-2xl">
          <AlertDialogHeader>
            <AlertDialogTitle>この記録を取り消しますか？</AlertDialogTitle>
            <AlertDialogDescription>
              この記録で動いた在庫を元に戻します。
              {cancel.recordType === "manual_sale"
                ? `${businessDate} の日次売上と月次の売上から外れます。`
                : null}
            </AlertDialogDescription>
          </AlertDialogHeader>

          <div className="space-y-3 text-sm">
            <p>
              {`記録は消さずに「取消済み」で残ります。取消を元に戻す操作はありません。打ち間違いを直すときは、取り消した後に作成の画面で入れ直してください。日付の打ち間違いでなければ、日付は ${businessDate} を選んでください（作成の画面の日付は今日が既定です）`}
            </p>
            <p>
              完了した棚卸しの評価額は変わりません（棚卸しの記録の詳細の差異・ロス原価・参考の差は変わることがあります）
            </p>
            <div className="space-y-2">
              <Label htmlFor={reasonId}>取り消す理由（必須）</Label>
              <Input
                id={reasonId}
                value={cancel.reason}
                disabled={cancel.isPending || cancel.isChecking}
                onChange={(event) => {
                  cancel.setReason(event.target.value);
                }}
              />
            </div>
            {cancel.isUnconfirmed ? (
              <div className="space-y-2">
                <p role="alert" className="text-destructive">
                  {UNCONFIRMED_MESSAGE}
                </p>
                <RecheckButton cancel={cancel} />
              </div>
            ) : null}
          </div>

          <AlertDialogFooter>
            <AlertDialogCancel disabled={cancel.isPending || cancel.isChecking}>
              キャンセル
            </AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              disabled={!cancel.canSubmit}
              onClick={(event) => {
                // 応答の知らせ（失敗・結果不明）を同じ dialog で受けるので、押しただけでは閉じない
                event.preventDefault();
                cancel.submit();
              }}
            >
              {cancel.isPending ? (
                <Loader2 className="size-4 animate-spin" aria-hidden="true" />
              ) : null}
              {cancel.isPending ? "取り消し中…" : "取り消す"}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}

export interface RecordCanceledAlertProps {
  cancel: RecordCancel;
  /** 読めている詳細。取消の日時・理由は取消済みの詳細から出す（読めていなければ、取消済みの知らせだけ） */
  detail?: CancelableRecordDetail;
}

/**
 * 取消済みの記録の header の下の表示（取消の日時・理由）と、取消で在庫が負になった商品の知らせ。
 * 取消が済んだかを確かめられていない記録には、dialog を開く前から結果未確認の知らせと「状態をもう一度確かめる」を出す
 */
export function RecordCanceledAlert({ cancel, detail }: RecordCanceledAlertProps) {
  const canceledDetail = detail?.status === "canceled" ? detail : undefined;
  if (canceledDetail === undefined && !cancel.isCanceled) {
    return cancel.isUnconfirmed ? (
      <Alert variant="warning">
        <AlertTitle>取消の結果未確認</AlertTitle>
        <AlertDescription>
          <p>{UNCONFIRMED_MESSAGE}</p>
          <RecheckButton cancel={cancel} />
        </AlertDescription>
      </Alert>
    ) : null;
  }
  // 応答を失った後に読んだ今の在庫: 0 未満だけ知らせる（作成の画面と同じ文。長さの商品は m）
  const negativeStocks = cancel.stockChecks.flatMap(({ productCode, product }) =>
    product !== undefined && product.stock_quantity < 0
      ? [
          `${productCode}: 在庫がマイナスになりました（${formatLengthOrPlain(product.stock_quantity, product.stock_unit)}）`,
        ]
      : [],
  );
  const warnings = [...cancel.stockWarnings, ...negativeStocks];
  const failedChecks = cancel.stockChecks.filter((check) => check.isError);
  const isReadingStock = cancel.stockChecks.some(
    (check) => check.product === undefined && !check.isError,
  );
  // 成功の直後は cache の状態だけが取消済み: 日時・理由は再取得した詳細が持つまで出さない（空の値を見せない）
  const canceledAt = canceledDetail?.canceled_at ?? null;
  return (
    <>
      <Alert>
        <AlertTitle>この記録は取消済みです</AlertTitle>
        {canceledAt === null ? null : (
          <AlertDescription>
            <dl className="grid gap-x-4 gap-y-1 sm:grid-cols-[auto_1fr]">
              <dt className="text-muted-foreground">取消の日時</dt>
              <dd className="font-mono tabular-nums">{formatDateTime(canceledAt)}</dd>
              <dt className="text-muted-foreground">取り消した理由</dt>
              <dd className="break-words">{canceledDetail?.cancel_reason ?? "—"}</dd>
            </dl>
          </AlertDescription>
        )}
      </Alert>
      {warnings.length > 0 || failedChecks.length > 0 || isReadingStock ? (
        <Alert variant="warning">
          <AlertTitle>取り消した後の在庫を確かめてください</AlertTitle>
          <AlertDescription>
            <ul className="list-disc space-y-1 pl-5">
              {warnings.map((warning) => (
                <li key={warning}>{warning}</li>
              ))}
              {failedChecks.map(({ productCode, retry }) => (
                <li key={productCode}>
                  {`${productCode}: 在庫を確かめられませんでした`}
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="ml-2"
                    onClick={retry}
                  >
                    {`${productCode} の在庫をもう一度確かめる`}
                  </Button>
                </li>
              ))}
            </ul>
            {isReadingStock ? <p role="status">取り消した後の在庫を確かめています…</p> : null}
          </AlertDescription>
        </Alert>
      ) : null}
    </>
  );
}

export interface RecordDetailErrorProps {
  /** 詳細を読めないときの page の題（例: 「入庫記録詳細」） */
  title: string;
  cancel: RecordCancel;
  query: Pick<UseQueryResult, "error" | "isFetching" | "refetch">;
  backLinkProps: { to: string; search: Record<string, unknown> };
}

/**
 * 5 つの詳細の読取りの error の表示。取消済みと確かめた後の再取得の失敗では、取消済みの知らせと
 * 負の在庫の警告を残し、詳細だけを読み直せる（取消の前の詳細は出さない）。page root の PageShell は各 page が持つ
 */
export function RecordDetailError({ title, cancel, query, backLinkProps }: RecordDetailErrorProps) {
  return (
    <>
      <PageHeader title={title} />
      <RecordCanceledAlert cancel={cancel} />
      <Alert variant="destructive">
        <AlertTitle>{describeError(query.error)}</AlertTitle>
        <AlertDescription>
          {cancel.isCanceled
            ? "詳細を読み直せませんでした。「詳細を読み直す」を押してください"
            : "記録IDを確認するか、入出庫履歴から開き直してください。"}
        </AlertDescription>
      </Alert>
      {cancel.isCanceled ? (
        <Button
          type="button"
          variant="outline"
          disabled={query.isFetching}
          onClick={() => void query.refetch()}
        >
          {query.isFetching ? <Loader2 className="size-4 animate-spin" aria-hidden="true" /> : null}
          詳細を読み直す
        </Button>
      ) : null}
      <Button asChild variant="outline">
        <Link {...backLinkProps}>
          <ArrowLeft aria-hidden="true" />
          前の画面へ戻る
        </Link>
      </Button>
    </>
  );
}
