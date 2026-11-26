// src/features/products/ProductListPage.tsx
//
// UI-01a 商品検索・一覧 page。

import { PackagePlus, PackageSearch } from "lucide-react";
import { Link } from "@tanstack/react-router";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { PageHeader } from "@/components/patterns/PageHeader";
import { SegmentedControl } from "@/components/ui/segmented-control";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { EmptyState } from "@/components/patterns/EmptyState";
import { SearchBar } from "@/components/patterns/SearchBar";
import { DepartmentFilter } from "@/components/patterns/DepartmentFilter";
import { ListShell } from "@/components/patterns/ListShell";
import { PageShell } from "@/components/patterns/PageShell";
import { LIST_PER_PAGE_OPTIONS } from "@/components/patterns/list-per-page";
import { ProductTable } from "./components/ProductTable";
import { useProductList } from "./hooks/useProductList";
import { buildProductListReturnTo } from "./lib/return-to";
import {
  PRODUCT_DISCONTINUED_OPTIONS,
  PRODUCT_SORT_DIRECTION_OPTIONS,
  PRODUCT_SORT_OPTIONS,
  updateProductListSearch,
  type ProductListSearch,
} from "./search";
import { scrollPageToTop } from "@/lib/page-scroll";

export interface ProductListPageProps {
  search: ProductListSearch;
  onSearchChange: (updater: (prev: ProductListSearch) => ProductListSearch) => void;
}

export function ProductListPage({ search, onSearchChange }: ProductListPageProps) {
  const { productsQuery, departmentsQuery, departmentOptions, normalizedSearch } = useProductList({
    search,
  });

  const updateSearch = (patch: Parameters<typeof updateProductListSearch>[1]) => {
    onSearchChange((prev) => updateProductListSearch(prev, patch));
  };
  const returnTo = buildProductListReturnTo(normalizedSearch);
  // filter-empty reset action（catalog ⑥、SPEC-UIBB-1/2）: q / dept / discontinued が既定値以外か。
  // sort / dir / perPage は結果集合を狭めないため対象外（分類軸どおり）。
  const isFilterDefault =
    normalizedSearch.q === undefined &&
    normalizedSearch.dept === undefined &&
    normalizedSearch.discontinued === "active";
  const totalCount = productsQuery.data?.total_count ?? 0;
  // items-end で上置き Label の検索欄と隣接 control の下辺を揃える。
  const toolbar = (
    <div className="flex flex-wrap items-end gap-3">
      {/* live 型（UI-01a-D9、owner L3 2026-08-03）。controlled value は raw search.q — trim 済みの
          normalizedSearch.q を結線すると live 反映のたびに trim 済み値が書き戻され「trim なし」契約が破れる。
          trim は CMD query 変換（buildProductSearchQuery）でのみ行う。page reset は updateSearch の
          pageOnlyChange 機構が担う。 */}
      <SearchBar
        value={search.q ?? ""}
        placeholder="商品コード・商品名・JAN・メーカー品番で検索"
        debounceMs={200}
        onSearchChange={(value) => {
          updateSearch({ q: value === "" ? undefined : value });
        }}
      />
      <DepartmentFilter
        options={departmentOptions}
        selected={normalizedSearch.dept ?? null}
        disabled={departmentsQuery.isLoading}
        onChange={(dept) => {
          updateSearch({ dept });
        }}
        allLabel="すべての部門"
        idPrefix="product-dept-filter"
      />
      {departmentsQuery.isError ? (
        <p className="text-sm text-destructive" role="alert">
          部門一覧の取得に失敗しました
        </p>
      ) : null}
      <SegmentedControl
        showLabel
        ariaLabel="廃番表示"
        value={normalizedSearch.discontinued}
        options={PRODUCT_DISCONTINUED_OPTIONS}
        onValueChange={(value) => {
          updateSearch({ discontinued: value });
        }}
      />
    </div>
  );

  const toolbarSecondary = (
    <div className="flex flex-wrap items-end gap-3">
      <div className="grid gap-1">
        <label className="text-sm text-muted-foreground" htmlFor="product-sort">
          並び替え
        </label>
        <Select
          value={normalizedSearch.sort}
          onValueChange={(value) => {
            const sort = PRODUCT_SORT_OPTIONS.find((option) => option.value === value)?.value;
            if (sort !== undefined) updateSearch({ sort });
          }}
        >
          <SelectTrigger id="product-sort" className="w-[10rem]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {PRODUCT_SORT_OPTIONS.map((option) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      <SegmentedControl
        showLabel
        ariaLabel="並び順"
        value={normalizedSearch.dir}
        options={PRODUCT_SORT_DIRECTION_OPTIONS}
        onValueChange={(value) => {
          updateSearch({ dir: value });
        }}
      />
      <div className="grid gap-1">
        <label className="text-sm text-muted-foreground" htmlFor="product-per-page">
          表示件数
        </label>
        <Select
          value={String(normalizedSearch.perPage)}
          onValueChange={(value) => {
            const perPage = LIST_PER_PAGE_OPTIONS.find((option) => String(option) === value);
            if (perPage !== undefined) {
              updateSearch({ perPage });
              scrollPageToTop();
            }
          }}
        >
          <SelectTrigger id="product-per-page" className="w-[7rem]">
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
  );

  return (
    // Lane 4 Gated Amendment 3 GA3b（2026-09-07、GA4b で高さ方針を viewport 基準へ改訂）:
    // 商品一覧のみ、表自身を縦横 scroll 箱にする data grid 型。旧 Gated Amendment 1 の
    // PageShell への `flex h-full min-h-0 flex-col` は撤去した——箱が
    // `max-h-[calc(100vh-6.75rem)]` の viewport 基準 max-h で自立し内容超過時のみ箱内で
    // 縦 scroll するため、page root からの高さ継承は不要（<main> は toolbar 分だけ
    // page scroll を担う）。
    <PageShell>
      <PageHeader
        title="商品検索・一覧"
        actions={
          <Button type="button" asChild>
            <Link to="/products/new" search={{ returnTo }}>
              <PackagePlus aria-hidden="true" />
              商品登録
            </Link>
          </Button>
        }
      />

      <ListShell
        toolbar={toolbar}
        toolbarSecondary={toolbarSecondary}
        pagination={{
          page: productsQuery.data?.page ?? normalizedSearch.page,
          perPage: productsQuery.data?.per_page ?? normalizedSearch.perPage,
          totalCount,
          onPageChange: (page) => {
            updateSearch({ page });
          },
        }}
        topSummary
        stickyHeader
        identityColumns={2}
        isLoading={productsQuery.isLoading}
      >
        {productsQuery.isError ? (
          <Alert variant="destructive">
            <AlertTitle>商品一覧の取得に失敗しました</AlertTitle>
            <AlertDescription>
              検索条件を変えるか、しばらくしてからもう一度お試しください。
            </AlertDescription>
          </Alert>
        ) : productsQuery.data?.items.length === 0 ? (
          // 意図的差分③: bare div → EmptyState 標準 UI（catalog ⑥）
          // filter-empty reset action（catalog ⑥、SPEC-UIBB-1/2）: 既存「商品を登録する」action は
          // 常設のまま維持し、絞り込みが非既定（q / dept / discontinued のいずれか）のときだけ
          // reset ボタンを横並びで併置する（既存 action が先、reset ボタンが後）。
          // sort / dir / perPage は結果集合を狭めないため reset 対象外（変更しない）。
          <EmptyState
            icon={PackageSearch}
            title="該当する商品がありません"
            description="検索条件を変更するか、新しい商品を登録してください"
            action={
              // 複数ボタンは中央揃え（catalog ⑥、owner L3 2026-08-03 是正、SPEC-UIBB-11）
              <div className="flex flex-wrap items-center justify-center gap-2">
                <Button type="button" asChild variant="outline">
                  <Link to="/products/new" search={{ returnTo }}>
                    商品を登録する
                  </Link>
                </Button>
                {!isFilterDefault && (
                  <Button
                    type="button"
                    variant="outline"
                    onClick={() => {
                      updateSearch({
                        q: undefined,
                        dept: undefined,
                        discontinued: undefined,
                        page: undefined,
                      });
                    }}
                  >
                    絞り込みを解除
                  </Button>
                )}
              </div>
            }
          />
        ) : productsQuery.data ? (
          <ProductTable items={productsQuery.data.items} returnTo={returnTo} />
        ) : null}
      </ListShell>
    </PageShell>
  );
}
