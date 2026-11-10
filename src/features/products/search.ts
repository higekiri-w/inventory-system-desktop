// src/features/products/search.ts
//
// UI-01a-D1〜D4: URL search params と ProductSearchQuery の変換を一箇所に集約する。

import type {
  PluMigrationFilter,
  ProductBulkFilter,
  ProductSearchQuery,
  SortKey,
  SortOrder,
} from "@/lib/bindings";
import { LIST_PER_PAGE_OPTIONS } from "@/components/patterns/list-per-page";
import { z } from "zod";
import { utf16String } from "@/lib/utf16-string";

export const PRODUCT_DISCONTINUED_OPTIONS = [
  { value: "active", label: "表示中", payload: false },
  { value: "all", label: "すべて", payload: null },
  { value: "discontinued", label: "廃番のみ", payload: true },
] as const;
export const PRODUCT_PLU_OPTIONS = [
  { value: "all", label: "すべて", payload: "all" },
  { value: "target", label: "対象", payload: "target" },
  { value: "pending", label: "未反映", payload: "pending" },
  { value: "synced", label: "反映済み", payload: "synced" },
  { value: "excluded", label: "対象外", payload: "excluded" },
] as const satisfies readonly { value: string; label: string; payload: PluMigrationFilter }[];
export const PRODUCT_SORT_OPTIONS = [
  { value: "product_code", label: "商品コード", payload: "ProductCode" },
  { value: "name", label: "商品名", payload: "Name" },
  { value: "stock_quantity", label: "在庫数", payload: "StockQuantity" },
  { value: "selling_price", label: "売価", payload: "SellingPrice" },
] as const satisfies readonly { value: string; label: string; payload: SortKey }[];
export const PRODUCT_SORT_DIRECTION_OPTIONS = [
  { value: "asc", label: "昇順", payload: "Asc" },
  { value: "desc", label: "降順", payload: "Desc" },
] as const satisfies readonly { value: string; label: string; payload: SortOrder }[];
export type ProductDiscontinuedMode = (typeof PRODUCT_DISCONTINUED_OPTIONS)[number]["value"];
export type ProductPluMode = (typeof PRODUCT_PLU_OPTIONS)[number]["value"];
export type ProductSortParam = (typeof PRODUCT_SORT_OPTIONS)[number]["value"];
export type ProductSortDirParam = (typeof PRODUCT_SORT_DIRECTION_OPTIONS)[number]["value"];
export type ProductPerPage = (typeof LIST_PER_PAGE_OPTIONS)[number];

function descriptorValues<
  const T extends readonly [{ readonly value: string }, ...{ readonly value: string }[]],
>(descriptors: T): { [K in keyof T]: T[K]["value"] } {
  return descriptors.map(({ value }) => value) as { [K in keyof T]: T[K]["value"] };
}

const PRODUCT_DISCONTINUED_VALUES = descriptorValues(PRODUCT_DISCONTINUED_OPTIONS);
const PRODUCT_PLU_VALUES = descriptorValues(PRODUCT_PLU_OPTIONS);
const PRODUCT_SORT_VALUES = descriptorValues(PRODUCT_SORT_OPTIONS);
const PRODUCT_SORT_DIRECTION_VALUES = descriptorValues(PRODUCT_SORT_DIRECTION_OPTIONS);

export const productListSearchSchema = z.object({
  q: utf16String({ max: 100 }).optional().catch(undefined),
  dept: z.coerce.number().int().positive().optional().catch(undefined),
  discontinued: z.enum(PRODUCT_DISCONTINUED_VALUES).optional().catch(undefined),
  plu: z.enum(PRODUCT_PLU_VALUES).optional().catch(undefined),
  sort: z.enum(PRODUCT_SORT_VALUES).optional().catch(undefined),
  dir: z.enum(PRODUCT_SORT_DIRECTION_VALUES).optional().catch(undefined),
  page: z.coerce.number().int().positive().optional().catch(undefined),
  perPage: z.coerce
    .number()
    .refine((value): value is ProductPerPage =>
      LIST_PER_PAGE_OPTIONS.includes(value as ProductPerPage),
    )
    .optional()
    .catch(undefined),
});

export type ProductListSearch = z.output<typeof productListSearchSchema>;

export interface ProductListSearchInput {
  q?: unknown;
  dept?: unknown;
  discontinued?: unknown;
  plu?: unknown;
  sort?: unknown;
  dir?: unknown;
  page?: unknown;
  perPage?: unknown;
}

export interface ProductListSearchPatch extends Partial<Omit<ProductListSearch, "dept">> {
  dept?: number | null;
}

export interface NormalizedProductListSearch {
  q: string | undefined;
  dept: number | undefined;
  discontinued: ProductDiscontinuedMode;
  plu: ProductPluMode;
  sort: ProductSortParam;
  dir: ProductSortDirParam;
  page: number;
  perPage: ProductPerPage;
}

const sortKeyMap = Object.fromEntries(
  PRODUCT_SORT_OPTIONS.map(({ value, payload }) => [value, payload]),
) as Record<ProductSortParam, SortKey>;

const sortOrderMap = Object.fromEntries(
  PRODUCT_SORT_DIRECTION_OPTIONS.map(({ value, payload }) => [value, payload]),
) as Record<ProductSortDirParam, SortOrder>;
const discontinuedMap = Object.fromEntries(
  PRODUCT_DISCONTINUED_OPTIONS.map(({ value, payload }) => [value, payload]),
) as Record<ProductDiscontinuedMode, boolean | null>;
const pluMap = Object.fromEntries(
  PRODUCT_PLU_OPTIONS.map(({ value, payload }) => [value, payload]),
) as Record<ProductPluMode, PluMigrationFilter>;

function normalizeString(value: unknown): string | undefined {
  if (typeof value !== "string") return undefined;
  const trimmed = value.trim();
  return trimmed === "" ? undefined : trimmed;
}

function normalizePositiveInt(value: unknown, fallback: number): number {
  const numberValue = typeof value === "number" ? value : Number(value);
  return Number.isInteger(numberValue) && numberValue >= 1 ? numberValue : fallback;
}

function normalizeDepartment(value: unknown): number | undefined {
  const numberValue = typeof value === "number" ? value : Number(value);
  return Number.isInteger(numberValue) && numberValue >= 1 ? numberValue : undefined;
}

function normalizePerPage(value: unknown): ProductPerPage {
  const numberValue = typeof value === "number" ? value : Number(value);
  return LIST_PER_PAGE_OPTIONS.includes(numberValue as ProductPerPage)
    ? (numberValue as ProductPerPage)
    : 100; // D-6: 商品一覧は 1 件探索が主動線のため既定 100（URL 明示値は優先維持）
}

function normalizeEnum<T extends string>(value: unknown, allowed: readonly T[], fallback: T): T {
  return typeof value === "string" && allowed.includes(value as T) ? (value as T) : fallback;
}

export function normalizeProductListSearch(
  input: ProductListSearchInput,
): NormalizedProductListSearch {
  return {
    q: normalizeString(input.q),
    dept: normalizeDepartment(input.dept),
    discontinued: normalizeEnum(input.discontinued, PRODUCT_DISCONTINUED_VALUES, "active"),
    plu: normalizeEnum(input.plu, PRODUCT_PLU_VALUES, "all"),
    sort: normalizeEnum(input.sort, PRODUCT_SORT_VALUES, "product_code"),
    dir: normalizeEnum(input.dir, PRODUCT_SORT_DIRECTION_VALUES, "asc"),
    page: normalizePositiveInt(input.page, 1),
    perPage: normalizePerPage(input.perPage),
  };
}

export function buildProductSearchQuery(search: ProductListSearchInput): ProductSearchQuery {
  const normalized = normalizeProductListSearch(search);

  return {
    keyword: normalized.q ?? null,
    department_id: normalized.dept ?? null,
    is_discontinued: discontinuedMap[normalized.discontinued],
    plu: pluMap[normalized.plu],
    sort_key: sortKeyMap[normalized.sort],
    sort_order: sortOrderMap[normalized.dir],
    page: normalized.page,
    per_page: normalized.perPage,
  };
}

export function buildProductBulkFilter(search: ProductListSearchInput): ProductBulkFilter {
  const normalized = normalizeProductListSearch(search);
  return {
    keyword: normalized.q ?? null,
    department_id: normalized.dept ?? null,
    is_discontinued: discontinuedMap[normalized.discontinued],
    plu: pluMap[normalized.plu],
  };
}

export function updateProductListSearch(
  current: ProductListSearch,
  patch: ProductListSearchPatch,
): ProductListSearch {
  const next: ProductListSearch = { ...current };
  if ("q" in patch) next.q = patch.q;
  if ("dept" in patch) next.dept = patch.dept ?? undefined;
  if ("discontinued" in patch) next.discontinued = patch.discontinued;
  if ("plu" in patch) next.plu = patch.plu;
  if ("sort" in patch) next.sort = patch.sort;
  if ("dir" in patch) next.dir = patch.dir;
  if ("page" in patch) next.page = patch.page;
  if ("perPage" in patch) next.perPage = patch.perPage;

  const pageOnlyChange = Object.keys(patch).length === 1 && "page" in patch;
  return pageOnlyChange ? next : { ...next, page: 1 };
}
