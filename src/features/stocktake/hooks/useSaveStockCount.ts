import { useMutation } from "@tanstack/react-query";

import { commands, type SaveStockCountRequest } from "@/lib/bindings";
import { unwrapResult } from "@/lib/invoke";

/** 計数の保存（既定の計数・fresh な計数。42 の `save_stock_count`） */
export function useSaveStockCount() {
  return useMutation({
    mutationFn: (request: SaveStockCountRequest) =>
      unwrapResult(commands.saveStockCount(request), {
        source: "commands",
        cmd: "save_stock_count",
      }),
  });
}
