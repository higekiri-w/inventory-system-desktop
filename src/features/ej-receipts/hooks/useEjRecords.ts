// src/features/ej-receipts/hooks/useEjRecords.ts
//
// UI-09c の 3 つの読取り（日の一覧・番号で探す・控えの文）。key は queryKeys.ejRecords の下（D-052 C26・C27 が root で無効にする）。
// 設計: docs/function-design/80-ui-ej-receipts.md §80.5、45 §45.6c

import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { commands } from "@/lib/bindings";
import { unwrapResult } from "@/lib/invoke";
import { queryKeys } from "@/lib/query-keys";

/** `null` は記録のある最も新しい日。日を移る間は前の日を出したまま（日の移動と番号の欄を unmount しない） */
export function useEjRecordDay(date: string | null) {
  return useQuery({
    queryKey: queryKeys.ejRecords.day(date),
    queryFn: () =>
      unwrapResult(commands.listEjRecords(date), { source: "commands", cmd: "list_ej_records" }),
    placeholderData: keepPreviousData,
  });
}

/** `null` の間は探さない */
export function useEjRecordsByNo(recordNo: string | null) {
  return useQuery({
    queryKey: queryKeys.ejRecords.byNo(recordNo ?? ""),
    queryFn: () =>
      unwrapResult(commands.findEjRecordsByNo(recordNo ?? ""), {
        source: "commands",
        cmd: "find_ej_records_by_no",
      }),
    enabled: recordNo !== null,
  });
}

/** 選んだ控えの文は ID で読む（番号で見つけた控えが一覧の 1,000 件の外でも出る） */
export function useEjRecordDetail(id: number | null) {
  return useQuery({
    queryKey: queryKeys.ejRecords.detail(id ?? 0),
    queryFn: () =>
      unwrapResult(commands.getEjRecord(id ?? 0), { source: "commands", cmd: "get_ej_record" }),
    enabled: id !== null,
  });
}
