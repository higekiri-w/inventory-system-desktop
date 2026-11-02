// src/features/stock-movements/StockMovementsPage.tsx
//
// UI-06c 商品別在庫変動履歴 page。
// 設計: docs/function-design/66-ui-stock-movements.md

import { ArrowLeft, PackageSearch } from "lucide-react";
import { Link, defaultStringifySearch } from "@tanstack/react-router";
import { useState } from "react";

import { LIST_PER_PAGE_OPTIONS } from "@/components/patterns/list-per-page";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { EmptyState } from "@/components/patterns/EmptyState";
import { PageHeader } from "@/components/patterns/PageHeader";
import { PageShell } from "@/components/patterns/PageShell";
import { Pagination, PaginationSummary } from "@/components/patterns/Pagination";
import { StockWithRecheck } from "@/features/stock-inquiry/components/StockRecheckBadge";
import { StockRecheckNotice } from "@/features/stock-inquiry/components/StockRecheckNotice";
import { scrollPageToTop } from "@/lib/page-scroll";
import { returnToLinkProps } from "@/lib/return-to";
import type { StockMovementsSearch } from "./types";
import { MOVEMENT_TYPE_OPTIONS, normalizeStockMovementsSearch } from "./types";
import { useStockMovements } from "./hooks/useStockMovements";
import { stockUnitMap } from "./components/MovementTable";
import { StockHistoryTable } from "./components/StockHistoryTable";

export interface StockMovementsPageProps {
  productCode: string;
  search: StockMovementsSearch;
  onSearchChange: (updater: (prev: StockMovementsSearch) => StockMovementsSearch) => void;
}

export function StockMovementsPage({
  productCode,
  search,
  onSearchChange,
}: StockMovementsPageProps) {
  const normalizedSearch = normalizeStockMovementsSearch(search);
  const [perPage, setPerPage] = useState<(typeof LIST_PER_PAGE_OPTIONS)[number]>(50);
  const { productQuery, movementsQuery } = useStockMovements({
    productCode,
    search: normalizedSearch,
    perPage,
  });

  const updateSearch = (patch: Partial<StockMovementsSearch>, resetPage = false) => {
    onSearchChange((prev) => ({
      ...prev,
      ...patch,
      page: resetPage ? 1 : (patch.page ?? prev.page),
    }));
  };
  // filter-empty reset action（catalog ⑥、SPEC-UIBB-1/2）: dateFrom / dateTo / type のいずれかが
  // 既定値以外か。
  const isFilterDefault =
    normalizedSearch.dateFrom === undefined &&
    normalizedSearch.dateTo === undefined &&
    normalizedSearch.type === "all";
  const resetFilters = () => {
    onSearchChange((prev) => ({
      ...prev,
      dateFrom: undefined,
      dateTo: undefined,
      type: undefined,
      page: undefined,
    }));
  };
  // UI-06c-D9: 「在庫照会へ戻る」は /stock 以外へ着地しない（pin）。pin 不一致・欠落・不正で
  // helper が null を返したら、商品コードで在庫照会を再展開する既定 fallback へ。
  const backLinkProps = returnToLinkProps(search.returnTo, { pathname: "/stock" });
  // UI-06c-D9: 元記録 link の returnTo は、正規化した search（既定値を載せない）と入れ子の
  // returnTo を router の既定の直列化で組む（手組みでも location.href でもない理由は 66 UI-06c-D9）。
  const detailReturnTo = `/stock/${encodeURIComponent(productCode)}/movements${defaultStringifySearch(
    {
      dateFrom: normalizedSearch.dateFrom,
      dateTo: normalizedSearch.dateTo,
      type: normalizedSearch.type === "all" ? undefined : normalizedSearch.type,
      page: normalizedSearch.page > 1 ? normalizedSearch.page : undefined,
      returnTo: search.returnTo,
    },
  )}`;

  return (
    <PageShell>
      <PageHeader
        title="在庫変動履歴"
        actions={
          <Button type="button" asChild variant="outline">
            {backLinkProps ? (
              <Link {...backLinkProps}>
                <ArrowLeft aria-hidden="true" />
                在庫照会へ戻る
              </Link>
            ) : (
              <Link to="/stock" search={{ q: productCode, selected: productCode }}>
                <ArrowLeft aria-hidden="true" />
                在庫照会へ戻る
              </Link>
            )}
          </Button>
        }
      />

      <section className="rounded-lg border bg-card p-4">
        {productQuery.isLoading ? (
          <div className="space-y-2">
            <Skeleton className="h-5 w-48" />
            <Skeleton className="h-5 w-80" />
          </div>
        ) : productQuery.isError ? (
          <p className="text-sm text-destructive" role="alert">
            商品情報の取得に失敗しました
          </p>
        ) : productQuery.data ? (
          <div className="grid gap-3 sm:grid-cols-4">
            <div className="sm:col-span-2">
              <p className="text-sm text-muted-foreground">商品</p>
              <p className="font-medium">{productQuery.data.product.name}</p>
              <p className="font-mono text-sm text-muted-foreground">
                {productQuery.data.product.product_code}
              </p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">部門</p>
              <p className="font-medium">{productQuery.data.product.department_name}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">現在庫</p>
              <div className="font-medium tabular-nums">
                <StockWithRecheck product={productQuery.data.product} />
              </div>
            </div>
            <div className="sm:col-span-4">
              <StockRecheckNotice
                productCode={productQuery.data.product.product_code}
                stockRecheck={productQuery.data.product.stock_recheck}
              />
            </div>
          </div>
        ) : null}
      </section>

      <div className="flex flex-wrap items-end gap-3 rounded-lg border bg-card p-4">
        <div className="grid gap-1">
          <label className="text-sm text-muted-foreground" htmlFor="movement-date-from">
            開始日
          </label>
          <input
            id="movement-date-from"
            className="h-9 rounded-md border border-input bg-control-surface px-3 text-sm"
            type="date"
            value={normalizedSearch.dateFrom ?? ""}
            onChange={(event) => {
              updateSearch({ dateFrom: event.currentTarget.value || undefined }, true);
            }}
          />
        </div>
        <div className="grid gap-1">
          <label className="text-sm text-muted-foreground" htmlFor="movement-date-to">
            終了日
          </label>
          <input
            id="movement-date-to"
            className="h-9 rounded-md border border-input bg-control-surface px-3 text-sm"
            type="date"
            value={normalizedSearch.dateTo ?? ""}
            onChange={(event) => {
              updateSearch({ dateTo: event.currentTarget.value || undefined }, true);
            }}
          />
        </div>
        <div className="grid gap-1">
          <label className="text-sm text-muted-foreground" htmlFor="movement-type">
            種別
          </label>
          <Select
            value={normalizedSearch.type}
            onValueChange={(value) => {
              updateSearch({ type: value as StockMovementsSearch["type"] }, true);
            }}
          >
            <SelectTrigger id="movement-type">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {MOVEMENT_TYPE_OPTIONS.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="grid gap-1">
          <label className="text-sm text-muted-foreground" htmlFor="stock-movements-per-page">
            表示件数
          </label>
          <Select
            value={String(perPage)}
            onValueChange={(value) => {
              const next = LIST_PER_PAGE_OPTIONS.find((option) => String(option) === value);
              if (next === undefined) return;
              setPerPage(next);
              updateSearch({}, true);
              scrollPageToTop();
            }}
          >
            <SelectTrigger id="stock-movements-per-page" className="w-[7rem]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {LIST_PER_PAGE_OPTIONS.map((option) => (
                <SelectItem key={option} value={String(option)}>
                  {option} 件
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>

      {movementsQuery.isLoading ? (
        <div className="space-y-2">
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-10 w-full" />
        </div>
      ) : movementsQuery.isError ? (
        <Alert variant="destructive">
          <AlertTitle>在庫変動履歴の取得に失敗しました</AlertTitle>
          <AlertDescription>
            検索条件を変えるか、しばらくしてからもう一度お試しください。
          </AlertDescription>
        </Alert>
      ) : movementsQuery.data?.items.length === 0 ? (
        <EmptyState
          icon={PackageSearch}
          title="在庫変動履歴がありません"
          description="この商品には該当する在庫変動がありません"
          // 絞り込みが既定値以外のときだけ reset action を出す（catalog ⑥、SPEC-UIBB-1/2）。
          action={
            !isFilterDefault ? (
              <Button type="button" variant="outline" onClick={resetFilters}>
                絞り込みを解除
              </Button>
            ) : undefined
          }
        />
      ) : movementsQuery.data ? (
        <div className="space-y-3">
          {movementsQuery.data.total_count > 0 && (
            <PaginationSummary
              page={movementsQuery.data.page}
              perPage={movementsQuery.data.per_page}
              totalCount={movementsQuery.data.total_count}
            />
          )}
          <StockHistoryTable
            items={movementsQuery.data.items}
            stockUnitByProductCode={stockUnitMap(
              productQuery.data ? [productQuery.data.product] : [],
            )}
            returnTo={detailReturnTo}
          />
          <Pagination
            page={movementsQuery.data.page}
            perPage={movementsQuery.data.per_page}
            totalCount={movementsQuery.data.total_count}
            onPageChange={(page) => {
              updateSearch({ page });
            }}
          />
        </div>
      ) : null}
    </PageShell>
  );
}
