// src/features/inventory-records/useRecordCancel.ts
//
// REQ-208: 入出庫の記録（入庫・返品交換・手動販売・廃棄・振替）の詳細からの取消の hook。
// 5 つの詳細が同じ hook と部品（RecordCancel.tsx）を使う。
// 設計: docs/function-design/65-inventory-record-traceability.md §65.5b（D-163）

import { useMutation, useQueries, useQueryClient, type QueryKey } from "@tanstack/react-query";
import { useBlocker } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { commands, type ReferenceType } from "@/lib/bindings";
import { describeError } from "@/lib/describe-error";
import { invalidateByContract, invalidationContract } from "@/lib/invalidation-contract";
import { CMD_ERROR_KIND, isInvokeError, unwrapResult } from "@/lib/invoke";
import { queryKeys } from "@/lib/query-keys";
import { useWindowCloseGuard } from "@/lib/window-close-guard";

/** 取り消せる 5 つの記録（CSV取込みは §65.5a、棚卸しは取消を持たない） */
export type CancelableRecordType = Exclude<ReferenceType, "csv_import" | "stocktake">;

/** 5 つの詳細の DTO が共通に持つ部分 */
export interface CancelableRecordDetail {
  status: string;
  canceled_at: string | null;
  cancel_reason: string | null;
  movements: { product_code: string }[];
}

interface UseRecordCancelOptions<T extends CancelableRecordDetail> {
  recordType: CancelableRecordType;
  recordId: number;
  /** その詳細の query key（取り直した詳細をここへ書く） */
  queryKey: QueryKey;
  /** その詳細の読取り（詳細の query と同じ関数） */
  readDetail: () => Promise<T>;
}

/** 取消が済んだかを確かめられていない記録: 取消の error と、取消の前の関連 movement の商品 */
interface Unconfirmed {
  error: unknown;
  codes: string[];
}

/** 確かめた取消の結果: 成功の応答の負の在庫の文か、応答を失った後に今の在庫を読む商品 */
interface Confirmed {
  stockWarnings: string[];
  stockCheckCodes: string[];
}

const FAILED_MESSAGE = "取り消しに失敗しました。もう一度お試しください";

/** 拒否の応答（validation）で届いた文。それ以外の error は null */
function rejectionMessage(error: unknown): string | null {
  return isInvokeError(error) && error.cmdError.kind === CMD_ERROR_KIND.VALIDATION
    ? describeError(error)
    : null;
}

export function useRecordCancel<T extends CancelableRecordDetail>({
  recordType,
  recordId,
  queryKey,
  readDetail,
}: UseRecordCancelOptions<T>) {
  const queryClient = useQueryClient();
  const [open, setOpen] = useState(false);
  const [reason, setReason] = useState("");
  // 取消の失敗・結果不明の後に、状態の取り直しも失敗した（取消が済んだかを言い切れない）。
  // 印は記録の種類と ID ごとに query の cache に置く: 画面を離れても詳細の cache に取消の前の active が残るので、
  // 同じ詳細を開き直しても、確かめるまで「有効」と言い切らず、「取り消す」を押させない。
  // cache の項目は未確認の間だけ持ち（取消を試していない記録には作らず、確かめたら消す）、この画面の state はその写し。
  // ponytail: 開き直した詳細の query が新しく active を読んでも印は解かない（cache の active が取消の前の読取りか
  // 後の読取りかを見分けない）。「状態をもう一度確かめる」で解く。自動で解く要求が出たら、印に時刻を持たせて
  // 詳細の dataUpdatedAt と比べる
  const unconfirmedKey = queryKeys.recordCancelUnconfirmed(recordType, recordId);
  const [unconfirmed, setUnconfirmedState] = useState(
    () => queryClient.getQueryData<Unconfirmed>(unconfirmedKey) ?? null,
  );
  const setUnconfirmed = (next: Unconfirmed | null) => {
    setUnconfirmedState(next);
    if (next === null) {
      queryClient.removeQueries({ queryKey: unconfirmedKey, exact: true });
      return;
    }
    // 読取りではないので取り直せない: 印の項目だけは、画面を離れた後も既定の gcTime で回収させない
    queryClient.setQueryDefaults([unconfirmedKey[0]], { gcTime: Number.POSITIVE_INFINITY });
    queryClient.setQueryData(unconfirmedKey, next);
  };
  const [isChecking, setIsChecking] = useState(false);
  // 確かめた取消の結果。詳細の読取りと別に持つ（詳細の再取得が遅い・失敗しても、取消済みと負の在庫を出す）
  const [confirmed, setConfirmed] = useState<Confirmed | null>(null);
  const stockCheckCodes = confirmed?.stockCheckCodes ?? [];

  const applyCanceled = (result: Confirmed) => {
    setOpen(false);
    setReason("");
    setUnconfirmed(null);
    setConfirmed(result);
    // 再取得の完了を待たない（知らせの後は待ちの止めを解く。詳細の再取得の失敗は RecordDetailError が受ける）
    void invalidateByContract(queryClient, invalidationContract.inventoryRecordCancel());
  };

  // 未確認の記録を開き直した詳細の読取りが、先に取消済みを読んだ: 取り直しの canceled と同じに扱う。
  // 詳細の変化は page の useQuery の再描画が運ぶので、描画のたびに見る。印は cache から読む
  // （1 回目が印を消すので、StrictMode の 2 回の effect でも 2 回通さない）
  useEffect(() => {
    const marker = queryClient.getQueryData<Unconfirmed>(unconfirmedKey);
    if (marker !== undefined && queryClient.getQueryData<T>(queryKey)?.status === "canceled") {
      applyCanceled({ stockWarnings: [], stockCheckCodes: marker.codes });
    }
  });

  // 失敗・結果不明のたびに同じ詳細の状態を取り直し、取り直した状態で分ける。
  // 詳細の query を通さずに読む（読取りの失敗で画面の全体を error の表示に替えず、dialog と理由を残す）
  const recheck = async (error: unknown, codes: string[]) => {
    // 印があれば「状態をもう一度確かめる」の読取り（取消の直後の取り直しには、まだ印が無い）
    const wasUnconfirmed = queryClient.getQueryData(unconfirmedKey) !== undefined;
    let fresh: T;
    try {
      fresh = await readDetail();
    } catch {
      setUnconfirmed({ error, codes });
      return;
    }
    queryClient.setQueryData(queryKey, fresh);
    const rejection = rejectionMessage(error);
    if (fresh.status === "canceled") {
      // commit の後に応答を失った、別の画面で先に取り消した
      if (rejection !== null) toast.info(rejection);
      // 読取りの間に、開き直した詳細の読取りが先に取消済みを読んで印を解いた（上の effect）: C25 は 1 回だけ
      if (wasUnconfirmed && queryClient.getQueryData(unconfirmedKey) === undefined) return;
      // 成功の応答の stock_warnings は無いので、取消の前の関連 movement の商品の今の在庫を読む
      applyCanceled({ stockWarnings: [], stockCheckCodes: codes });
      return;
    }
    setUnconfirmed(null);
    toast.error(rejection ?? FAILED_MESSAGE);
  };

  const mutation = useMutation({
    mutationFn: (variables: { reason: string; codes: string[] }) =>
      unwrapResult(commands.cancelInventoryRecord(recordType, recordId, variables.reason), {
        source: "commands",
        cmd: "cancel_inventory_record",
      }),
    retry: 0,
    onSuccess: (result) => {
      // 取消は破壊的操作の完了通知なので、既定より長く出す（useRollbackCsvImport と同じ表示時間）
      toast.success("記録を取り消しました", { duration: 8000 });
      // 再取得を待つ間（その間に開き直したときも）、取消の前の「有効」・関連 movement・取消の button を出さない。
      // 取消の日時・理由は再取得した詳細が補う
      queryClient.setQueryData<T>(
        queryKey,
        (old) => old && { ...old, status: "canceled", movements: [] },
      );
      applyCanceled({ stockWarnings: result.stock_warnings, stockCheckCodes: [] });
    },
    onError: (error, variables) => recheck(error, variables.codes),
  });

  // 取消の応答と「状態をもう一度確かめる」の読取りを待つ間は、route の移動・リロード等の page の離脱・native の
  // window の close（UI-USW-D5）を止める（離れて開き直した画面には、その応答が届かない）。
  // pending は onSuccess / onError の後に外れるので、成功の Toast・失敗と結果不明の取り直しの後に解ける
  const isBusy = mutation.isPending || isChecking;
  useBlocker({ shouldBlockFn: () => isBusy, enableBeforeUnload: () => isBusy });
  useWindowCloseGuard("inventory-record-cancel", isBusy);

  // 応答を失った後の負の在庫の知らせ: 商品ごとに今の在庫を読む（読めなかった商品はその商品だけ読み直せる）
  const stockChecks = useQueries({
    queries: stockCheckCodes.map((code) => ({
      queryKey: queryKeys.stockInquiry.detail(code),
      queryFn: () =>
        unwrapResult(commands.getStockDetail(code), {
          source: "commands",
          cmd: "get_stock_detail",
        }),
      staleTime: 0,
      retry: 0,
    })),
  }).map((query, index) => ({
    productCode: stockCheckCodes[index] ?? "",
    // 読み直しの間と読めなかったときは、cache の前の値（取消の前の在庫かもしれない）を出さない
    product: query.isFetching || query.isError ? undefined : query.data?.product,
    isError: query.isError && !query.isFetching,
    retry: () => void query.refetch(),
  }));

  const trimmedReason = reason.trim();

  return {
    recordType,
    open,
    /** 送信中と取り直しの間は閉じない（応答の知らせを同じ dialog で受ける） */
    setOpen: (next: boolean) => {
      if (!isBusy) setOpen(next);
    },
    reason,
    setReason,
    isPending: mutation.isPending,
    isChecking,
    /** 取消が済んだかを確かめられていない（詳細は「結果未確認」、「取り消す」は押せず、「状態をもう一度確かめる」を出す） */
    isUnconfirmed: unconfirmed !== null,
    canSubmit: trimmedReason !== "" && !isBusy && unconfirmed === null,
    /** 「取り消す」: 取消を送る。取消の前の関連 movement の商品を、応答を失ったときの在庫の読取りに残す */
    submit: () => {
      if (trimmedReason === "" || isBusy || unconfirmed !== null) return;
      const codes = [
        ...new Set(
          (queryClient.getQueryData<T>(queryKey)?.movements ?? []).map(
            (movement) => movement.product_code,
          ),
        ),
      ];
      mutation.mutate({ reason: trimmedReason, codes });
    },
    /** 「状態をもう一度確かめる」: 詳細の読取りだけをやり直す（取消は送らない） */
    recheck: () => {
      if (unconfirmed === null || isBusy) return;
      setIsChecking(true);
      void recheck(unconfirmed.error, unconfirmed.codes).finally(() => {
        setIsChecking(false);
      });
    },
    /** 取消済みと確かめた（詳細の再取得が終わる前・失敗した後も、取消済みの知らせを出す） */
    isCanceled: confirmed !== null,
    stockWarnings: confirmed?.stockWarnings ?? [],
    stockChecks,
  };
}

export type RecordCancel = ReturnType<typeof useRecordCancel>;
