import { useQuery } from "@tanstack/react-query";

import { commands } from "@/lib/bindings";
import { unwrapResult } from "@/lib/invoke";
import { queryKeys } from "@/lib/query-keys";

/** 要再確認の一覧（73 の棚卸しの画面と、58・66 の要再確認の案内が読む。写しを書く entry が invalidate する） */
export function useRecheckProducts() {
  return useQuery({
    queryKey: queryKeys.stocktake.recheckList(),
    queryFn: () =>
      unwrapResult(commands.listRecheckProducts(), {
        source: "commands",
        cmd: "list_recheck_products",
      }),
    staleTime: 0,
  });
}
