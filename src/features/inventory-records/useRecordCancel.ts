// src/features/inventory-records/useRecordCancel.ts
//
// REQ-208: 入出庫の記録（入庫・返品交換・手動販売・廃棄・振替）の詳細からの取消の hook。
// 5 つの詳細が同じ hook と部品（RecordCancel.tsx）を使う。
// 設計: docs/function-design/65-inventory-record-traceability.md §65.5b（D-163）

import {
  skipToken,
  useMutation,
  useQueries,
  useQuery,
  useQueryClient,
  type QueryKey,
} from "@tanstack/react-query";
import { useBlocker } from "@tanstack/react-router";
import { useState } from "react";
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
  // 記録の種類と ID ごとに query の cache に持つ: 画面を離れても詳細の cache に取消の前の active が残るので、
  // 同じ詳細を開き直しても「状態をもう一度確かめる」で active を確かめるまで「取り消す」を押させない
  // （置き場が詳細の cache と同じ QueryClient なら、片方だけが残ることがない）。
  // ponytail: 印を消すのはこの hook の取り直しだけ。開き直した詳細の query が先に取消済みを読んだときは、
  // 取消の button が出ないので印は残り、今の在庫も読まない（初めから取消済みの記録を開いたのと同じ表示）。
  // その経路でも負の在庫を知らせる要求が出たら、開いたときに印があれば取り直す
  const unconfirmedKey = queryKeys.recordCancelUnconfirmed(recordType, recordId);
  const unconfirmed =
    useQuery<Unconfirmed | null>({
      queryKey: unconfirmedKey,
      queryFn: skipToken,
      gcTime: Number.POSITIVE_INFINITY,
    }).data ?? null;
  const setUnconfirmed = (next: Unconfirmed | null) => {
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

  // 失敗・結果不明のたびに同じ詳細の状態を取り直し、取り直した状態で分ける。
  // 詳細の query を通さずに読む（読取りの失敗で画面の全体を error の表示に替えず、dialog と理由を残す）
  const recheck = async (error: unknown, codes: string[]) => {
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
      applyCanceled({ stockWarnings: result.stock_warnings, stockCheckCodes: [] });
    },
    onError: (error, variables) => recheck(error, variables.codes),
  });

  // 応答を待つ間は route の移動・リロード等の page の離脱・native の window の close（UI-USW-D5）を止める。
  // pending は onSuccess / onError の後に外れるので、成功の Toast・失敗と結果不明の取り直しの後に解ける
  useBlocker({
    shouldBlockFn: () => mutation.isPending,
    enableBeforeUnload: () => mutation.isPending,
  });
  useWindowCloseGuard("inventory-record-cancel", mutation.isPending);

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

  const isBusy = mutation.isPending || isChecking;
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
    /** 取消が済んだかを確かめられていない（「取り消す」は押せず、「状態をもう一度確かめる」を出す） */
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
