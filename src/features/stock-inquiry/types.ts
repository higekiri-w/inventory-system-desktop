// src/features/stock-inquiry/types.ts
//
// UI-06a 在庫照会の feature 内型定義。
// 設計: docs/function-design/58-ui-stock-inquiry.md §58.2 / §58.5

import type { ProductWithRelations } from "@/lib/bindings";
import { z } from "zod";
import { utf16String } from "@/lib/utf16-string";

/** 在庫状態表示用。閾値判定は持たず、source + stock_quantity から派生する。 */
export type StockStatus = "ok" | "low" | "stockout";

export const LOW_STOCK_FILTER = "low_stock" as const;
export const STOCK_FILTER_DESCRIPTORS = [
  { value: "all", label: "すべて" },
  { value: "stockout", label: "在庫切れ" },
  { value: LOW_STOCK_FILTER, label: "在庫少" },
] as const;

/** 状態チップのフィルタ値（URL state `status`）。 */
export type ListChipFilter = (typeof STOCK_FILTER_DESCRIPTORS)[number]["value"];

function descriptorValues<
  const T extends readonly [{ readonly value: string }, ...{ readonly value: string }[]],
>(descriptors: T): { [K in keyof T]: T[K]["value"] } {
  return descriptors.map(({ value }) => value) as { [K in keyof T]: T[K]["value"] };
}

const STOCK_FILTER_VALUES = descriptorValues(STOCK_FILTER_DESCRIPTORS);

export const stockInquirySearchSchema = z.object({
  q: utf16String({ min: 1, max: 100 }).optional().catch(undefined),
  dept: z.coerce.number().int().positive().optional().catch(undefined),
  status: z.enum(STOCK_FILTER_VALUES).optional().catch(undefined),
  // UI-06a-D1（2026-08-03 batch B）: page search param。50 §50.4 と同型
  // （number >= 1、既定は呼び出し側で `page ?? 1`、invalid は catch で吸収）。
  page: z.coerce.number().int().positive().optional().catch(undefined),
  selected: utf16String({ min: 1, max: 100 }).optional().catch(undefined),
});

/**
 * list query の戻り値正規化型。
 *
 * `search_products` は `PaginatedResult<T>`、`list_low_stock` は `T[]` で形状が
 * 異なるため、hook 内でこの型に正規化する（§58.5 Round 6 P2(a)）。
 * 自動展開 / EmptySearchPlaceholder 判定 / Pagination は常に
 * `items` / `totalCount` を参照する（生 DTO 直接参照禁止、type narrowing 維持）。
 *
 * UI-06a-D1（2026-08-03 batch B）: `truncated` field は撤去済み。pagination 導入により
 * 全件へページ送りで到達できるため、打ち切り告知フラグを持つ理由がなくなった。
 */
export interface StockInquiryListResult {
  items: ProductWithRelations[];
  /** source="search" 時のみ数値、source="low_stock" 時 null。 */
  totalCount: number | null;
  source: "search" | "low_stock";
}

/**
 * 部門フィルタの選択肢。`patterns/DepartmentFilter.tsx` を唯一の定義とし、本 module 内では
 * 使用しないため直接 re-export とする（59 §59.3、SPEC-UIBB-6）。
 */
export type { DepartmentOption } from "@/components/patterns/DepartmentFilter";

/** URL search params（zod 4 validateSearch で検証）。 */
export type StockInquirySearch = z.output<typeof stockInquirySearchSchema>;
