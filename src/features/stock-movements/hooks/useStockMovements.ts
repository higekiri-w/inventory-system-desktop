// src/features/stock-movements/hooks/useStockMovements.ts
//
// UI-06c 商品別在庫変動履歴の 2 useQuery 部分障害許容 hook。
// 設計: docs/function-design/66-ui-stock-movements.md §66.4

import { useQuery } from "@tanstack/react-query";
import type { UseQueryResult } from "@tanstack/react-query";

import { commands } from "@/lib/bindings";
import type { PaginatedResult, StockDetail, StockHistoryItem } from "@/lib/bindings";
import { unwrapResult } from "@/lib/invoke";
import { queryKeys } from "@/lib/query-keys";
import type { NormalizedStockMovementsSearch } from "../types";

export interface UseStockMovementsArgs {
  productCode: string;
  search: NormalizedStockMovementsSearch;
  perPage: number;
}

export interface UseStockMovementsResult {
  productQuery: UseQueryResult<StockDetail>;
  movementsQuery: UseQueryResult<PaginatedResult<StockHistoryItem>>;
}

export function useStockMovements(args: UseStockMovementsArgs): UseStockMovementsResult {
  const productQuery = useQuery({
    queryKey: queryKeys.stockMovements.product(args.productCode),
    queryFn: () =>
      unwrapResult(commands.getStockDetail(args.productCode), {
        source: "commands",
        cmd: "get_stock_detail",
      }),
    staleTime: 10_000,
    gcTime: 5 * 60_000,
    retry: 1,
  });

  const movementsQuery = useQuery({
    queryKey: queryKeys.stockMovements.list(args.productCode, {
      ...args.search,
      perPage: args.perPage,
    }),
    queryFn: () =>
      unwrapResult(
        commands.listMovements({
          product_code: args.productCode,
          date_from: args.search.dateFrom ?? null,
          date_to: args.search.dateTo ?? null,
          movement_type: args.search.type === "all" ? null : args.search.type,
          page: args.search.page,
          per_page: args.perPage,
        }),
        { source: "commands", cmd: "list_movements" },
      ),
    staleTime: 10_000,
    gcTime: 5 * 60_000,
    retry: 1,
  });

  return { productQuery, movementsQuery };
}
