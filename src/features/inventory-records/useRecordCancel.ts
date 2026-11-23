// src/features/inventory-records/useRecordCancel.ts
//
// REQ-208: 入出庫の記録（入庫・返品交換・手動販売・廃棄・振替）の詳細からの取消の hook。
// 5 つの詳細が同じ hook と部品（RecordCancel.tsx）を使う。
// 設計: docs/function-design/65-inventory-record-traceability.md §65.5b（D-163）

import { useMutation, useQueries, useQueryClient, type QueryKey } from "@tanstack/react-query";
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
  // 「状態をもう一度確かめる」が使う、取消の error と取消の前の関連 movement の商品を持つ
  const [unconfirmed, setUnconfirmed] = useState<{ error: unknown; codes: string[] } | null>(null);
  const [isChecking, setIsChecking] = useState(false);
  // 成功の応答の負の在庫の文
  const [stockWarnings, setStockWarnings] = useState<string[]>([]);
  // 応答を失った後に取消済みと分かったとき、今の在庫を読む商品（取消の前の関連 movement の商品）
  const [stockCheckCodes, setStockCheckCodes] = useState<string[]>([]);

  const applyCanceled = async () => {
    setOpen(false);
    setReason("");
    setUnconfirmed(null);
    await invalidateByContract(queryClient, invalidationContract.inventoryRecordCancel());
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
      await applyCanceled();
      // 成功の応答の stock_warnings は無いので、再取得の後に今の在庫を読む
      setStockCheckCodes(codes);
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
    onSuccess: async (result) => {
      setStockWarnings(result.stock_warnings);
      // 取消は破壊的操作の完了通知なので、既定より長く出す（useRollbackCsvImport と同じ表示時間）
      toast.success("記録を取り消しました", { duration: 8000 });
      await applyCanceled();
    },
    onError: (error, variables) => recheck(error, variables.codes),
  });

  // 応答を待つ間は route の移動・リロード等の page の離脱・native の window の close（UI-USW-D5）を止める。
  // pending は onSuccess / onError の後に外れるので、成功・失敗・結果不明の知らせの後に解ける
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
    // 読み直しの間は前の値（取消の前の在庫かもしれない）を出さない
    product: query.isFetching ? undefined : query.data?.product,
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
    stockWarnings,
    stockChecks,
  };
}

export type RecordCancel = ReturnType<typeof useRecordCancel>;
