import { useMutation } from "@tanstack/react-query";

import { commands } from "@/lib/bindings";
import { unwrapResult } from "@/lib/invoke";

export interface CompleteStocktakeVariables {
  stocktakeId: number;
}

export function useCompleteStocktake() {
  return useMutation({
    mutationFn: ({ stocktakeId }: CompleteStocktakeVariables) =>
      unwrapResult(commands.completeStocktake(stocktakeId), {
        source: "commands",
        cmd: "complete_stocktake",
      }),
  });
}
