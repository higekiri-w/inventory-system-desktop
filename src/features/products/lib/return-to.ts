// src/features/products/lib/return-to.ts
//
// UI-01b-D2: 保存後の戻り先は `/products` 一覧 route と search params だけ許可する。

import type { ProductListSearch } from "../search";

export function sanitizeProductListReturnTo(value: string | null | undefined): string {
  if (value === null || value === undefined || value.trim() === "") {
    return "/products";
  }

  let url: URL;
  try {
    url = new URL(value, "http://inventory.local");
  } catch {
    return "/products";
  }

  if (url.origin !== "http://inventory.local") {
    return "/products";
  }

  if (url.pathname !== "/products" && url.pathname !== "/products/") {
    return "/products";
  }

  return `/products${url.search}`;
}

export function buildProductListReturnTo(search: ProductListSearch): string {
  const params = new URLSearchParams();
  if (search.q !== undefined && search.q !== "") params.set("q", search.q);
  if (search.dept !== undefined) params.set("dept", String(search.dept));
  if (search.discontinued !== undefined) params.set("discontinued", search.discontinued);
  if (search.sort !== undefined) params.set("sort", search.sort);
  if (search.dir !== undefined) params.set("dir", search.dir);
  if (search.page !== undefined) params.set("page", String(search.page));
  if (search.perPage !== undefined) params.set("perPage", String(search.perPage));
  const query = params.toString();
  return query === "" ? "/products" : `/products?${query}`;
}

export function parseProductListSearchFromReturnTo(value: string): ProductListSearch {
  const safe = sanitizeProductListReturnTo(value);
  const url = new URL(safe, "http://inventory.local");
  const numberParam = (key: string): number | undefined => {
    const raw = url.searchParams.get(key);
    if (raw === null) return undefined;
    const parsed = Number(raw);
    return Number.isFinite(parsed) ? parsed : undefined;
  };

  return {
    q: url.searchParams.get("q") ?? undefined,
    dept: numberParam("dept"),
    discontinued: (url.searchParams.get("discontinued") ??
      undefined) as ProductListSearch["discontinued"],
    sort: (url.searchParams.get("sort") ?? undefined) as ProductListSearch["sort"],
    dir: (url.searchParams.get("dir") ?? undefined) as ProductListSearch["dir"],
    page: numberParam("page"),
    perPage: numberParam("perPage") as ProductListSearch["perPage"],
  };
}
