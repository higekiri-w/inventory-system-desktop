import type { ProductSearchQuery, SortKey } from "@/lib/bindings";
import { LIST_PER_PAGE_OPTIONS } from "@/components/patterns/list-per-page";
import { z } from "zod";
import { utf16String } from "@/lib/utf16-string";

import { PRODUCT_SORT_OPTIONS } from "./search";

export type PriceRevisionPerPage = (typeof LIST_PER_PAGE_OPTIONS)[number];
export type PriceRevisionSort = (typeof PRODUCT_SORT_OPTIONS)[number]["value"];

const sortValues = PRODUCT_SORT_OPTIONS.map(({ value }) => value) as [
  PriceRevisionSort,
  ...PriceRevisionSort[],
];

const booleanSearchParam = z
  .union([z.boolean(), z.enum(["true", "false"]).transform((value) => value === "true")])
  .optional()
  .catch(undefined);

export const priceRevisionSearchSchema = z.object({
  q: utf16String({ max: 100 }).optional().catch(undefined),
  supplier: z.coerce.number().int().positive().optional().catch(undefined),
  includeUnassigned: booleanSearchParam,
  dept: z.coerce.number().int().positive().optional().catch(undefined),
  discontinued: booleanSearchParam,
  sort: z.enum(sortValues).optional().catch(undefined),
  page: z.coerce.number().int().positive().optional().catch(undefined),
  perPage: z.coerce
    .number()
    .refine((value): value is PriceRevisionPerPage =>
      LIST_PER_PAGE_OPTIONS.includes(value as PriceRevisionPerPage),
    )
    .optional()
    .catch(undefined),
});

export type PriceRevisionSearch = z.output<typeof priceRevisionSearchSchema>;

export interface PriceRevisionSearchInput {
  q?: unknown;
  supplier?: unknown;
  includeUnassigned?: unknown;
  dept?: unknown;
  discontinued?: unknown;
  sort?: unknown;
  page?: unknown;
  perPage?: unknown;
}

export interface NormalizedPriceRevisionSearch {
  q: string | undefined;
  supplier: number | undefined;
  includeUnassigned: boolean;
  dept: number | undefined;
  discontinued: boolean;
  sort: PriceRevisionSort;
  page: number;
  perPage: PriceRevisionPerPage;
}

export interface PriceRevisionSearchPatch extends Omit<
  Partial<PriceRevisionSearch>,
  "supplier" | "dept"
> {
  supplier?: number | null;
  dept?: number | null;
}

function normalizedString(value: unknown): string | undefined {
  if (typeof value !== "string") return undefined;
  const trimmed = value.trim();
  return trimmed === "" ? undefined : trimmed;
}

function normalizedPositiveInt(value: unknown): number | undefined {
  const numeric = typeof value === "number" ? value : Number(value);
  return Number.isInteger(numeric) && numeric > 0 ? numeric : undefined;
}

function normalizedBoolean(value: unknown, fallback: boolean): boolean {
  if (typeof value === "boolean") return value;
  if (value === "true") return true;
  if (value === "false") return false;
  return fallback;
}

function normalizedPerPage(value: unknown): PriceRevisionPerPage {
  const numeric = typeof value === "number" ? value : Number(value);
  return LIST_PER_PAGE_OPTIONS.includes(numeric as PriceRevisionPerPage)
    ? (numeric as PriceRevisionPerPage)
    : 50;
}

function normalizedSort(value: unknown): PriceRevisionSort {
  return typeof value === "string" && sortValues.includes(value as PriceRevisionSort)
    ? (value as PriceRevisionSort)
    : "product_code";
}

const sortKeyMap = Object.fromEntries(
  PRODUCT_SORT_OPTIONS.map(({ value, payload }) => [value, payload]),
) as Record<PriceRevisionSort, SortKey>;

export function normalizePriceRevisionSearch(
  input: PriceRevisionSearchInput,
): NormalizedPriceRevisionSearch {
  const supplier = normalizedPositiveInt(input.supplier);
  return {
    q: normalizedString(input.q),
    supplier,
    includeUnassigned:
      supplier === undefined ? false : normalizedBoolean(input.includeUnassigned, true),
    dept: normalizedPositiveInt(input.dept),
    discontinued: normalizedBoolean(input.discontinued, false),
    sort: normalizedSort(input.sort),
    page: normalizedPositiveInt(input.page) ?? 1,
    perPage: normalizedPerPage(input.perPage),
  };
}

export function buildPriceRevisionProductSearchQuery(
  input: PriceRevisionSearchInput,
): ProductSearchQuery {
  const normalized = normalizePriceRevisionSearch(input);
  return {
    keyword: normalized.q ?? null,
    department_id: normalized.dept ?? null,
    supplier_id: normalized.supplier ?? null,
    include_unassigned: normalized.includeUnassigned,
    is_discontinued: normalized.discontinued ? null : false,
    plu: "all",
    sort_key: sortKeyMap[normalized.sort],
    sort_order: "Asc",
    page: normalized.page,
    per_page: normalized.perPage,
  };
}

export function updatePriceRevisionSearch(
  current: PriceRevisionSearch,
  patch: PriceRevisionSearchPatch,
): PriceRevisionSearch {
  const next: PriceRevisionSearch = { ...current };
  if ("q" in patch) next.q = patch.q;
  if ("supplier" in patch) {
    next.supplier = patch.supplier ?? undefined;
    next.includeUnassigned = patch.supplier == null ? undefined : true;
  }
  if ("includeUnassigned" in patch) next.includeUnassigned = patch.includeUnassigned;
  if ("dept" in patch) next.dept = patch.dept ?? undefined;
  if ("discontinued" in patch) next.discontinued = patch.discontinued;
  if ("sort" in patch) next.sort = patch.sort;
  if ("page" in patch) next.page = patch.page;
  if ("perPage" in patch) next.perPage = patch.perPage;

  const pageOnly = Object.keys(patch).length === 1 && "page" in patch;
  return pageOnly ? next : { ...next, page: 1 };
}

export function resetPriceRevisionSearch(): PriceRevisionSearch {
  return { page: 1 };
}
