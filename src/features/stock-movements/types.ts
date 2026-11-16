// src/features/stock-movements/types.ts
//
// UI-06c 商品別在庫変動履歴の URL search / 表示型。
// 設計: docs/function-design/66-ui-stock-movements.md §66.3

import { z } from "zod";
import { utf16String } from "@/lib/utf16-string";

export const MOVEMENT_TYPE_OPTIONS = [
  { value: "all", label: "すべて" },
  { value: "receiving", label: "入庫" },
  { value: "return", label: "返品・交換" },
  { value: "sale_auto", label: "POS売上" },
  { value: "sale_manual", label: "手動販売" },
  { value: "disposal", label: "廃棄・破損" },
  { value: "transfer", label: "詰め替え・切り分け" },
  { value: "stocktake", label: "棚卸し" },
] as const;

function descriptorValues<
  const T extends readonly [{ readonly value: string }, ...{ readonly value: string }[]],
>(descriptors: T): { [K in keyof T]: T[K]["value"] } {
  return descriptors.map(({ value }) => value) as { [K in keyof T]: T[K]["value"] };
}

export const MOVEMENT_TYPES = descriptorValues(MOVEMENT_TYPE_OPTIONS);

export type MovementTypeFilter = (typeof MOVEMENT_TYPE_OPTIONS)[number]["value"];

export const stockMovementsSearchSchema = z.object({
  dateFrom: z
    .string()
    .regex(/^\d{4}-\d{2}-\d{2}$/)
    .optional()
    .catch(undefined),
  dateTo: z
    .string()
    .regex(/^\d{4}-\d{2}-\d{2}$/)
    .optional()
    .catch(undefined),
  type: z.enum(MOVEMENT_TYPES).optional().catch(undefined),
  page: z.coerce.number().int().positive().optional().catch(undefined),
  returnTo: utf16String({ max: 500 }).optional().catch(undefined),
});

export type StockMovementsSearch = z.output<typeof stockMovementsSearchSchema>;

export interface NormalizedStockMovementsSearch {
  dateFrom?: string;
  dateTo?: string;
  type: MovementTypeFilter;
  page: number;
}

export function normalizeStockMovementsSearch(
  search: StockMovementsSearch,
): NormalizedStockMovementsSearch {
  return {
    dateFrom: search.dateFrom,
    dateTo: search.dateTo,
    type: search.type ?? "all",
    page: Math.max(1, search.page ?? 1),
  };
}
