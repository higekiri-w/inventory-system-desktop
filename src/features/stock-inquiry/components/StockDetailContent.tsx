// src/features/stock-inquiry/components/StockDetailContent.tsx
//
// 商品詳細の内側描画（在庫数 / 売価 / 原価 / 最終入庫日 / 最終販売日 + 要再確認の案内 + 遷移 CTA × 3）。
// query の isLoading / isError / data 全状態を内包する。行インライン展開（colSpan td 内）
// と list 失敗時フォールバックカード（StockDetailCard が Card で包む）で共用する。
// 設計: docs/function-design/58-ui-stock-inquiry.md §58.7 / §58.8

import type { UseQueryResult } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { History } from "lucide-react";
import { Link, useRouterState } from "@tanstack/react-router";
import type { StockDetail } from "@/lib/bindings";
import { Button } from "@/components/ui/button";
import { CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { formatCostYen } from "../lib/format-stock-display";
import { formatLastDate } from "../lib/format-last-date";
import { StockWithRecheck } from "./StockRecheckBadge";
import { StockRecheckNotice } from "./StockRecheckNotice";

export interface StockDetailContentProps {
  query: UseQueryResult<StockDetail>;
}

const priceFormatter = new Intl.NumberFormat("ja-JP", {
  style: "currency",
  currency: "JPY",
});

function DetailRow({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="flex justify-between border-b py-2 text-sm last:border-b-0">
      <span className="text-muted-foreground">{label}</span>
      <span className="font-medium tabular-nums">{value}</span>
    </div>
  );
}

function ActiveCta({ label, productCode }: { label: string; productCode: string }) {
  const returnTo = useRouterState({ select: (state) => state.location.href });
  return (
    <Button type="button" asChild variant="outline" size="sm">
      <Link to="/stock/$code/movements" params={{ code: productCode }} search={{ returnTo }}>
        <History aria-hidden="true" />
        {label}
      </Link>
    </Button>
  );
}

export function StockDetailContent({ query }: StockDetailContentProps) {
  if (query.isLoading) {
    return (
      <CardContent className="space-y-2 py-4">
        <Skeleton className="h-5 w-1/3" />
        <Skeleton className="h-5 w-full" />
        <Skeleton className="h-5 w-full" />
      </CardContent>
    );
  }

  if (query.isError) {
    return (
      <CardContent className="py-4 text-sm text-destructive">
        商品詳細の取得に失敗しました。別の商品を選び直すか、再度お試しください。
      </CardContent>
    );
  }

  if (query.data) {
    return (
      <>
        <CardHeader className="pt-4">
          <CardTitle className="text-base">
            {query.data.product.name}
            <span className="ml-2 font-mono text-sm font-medium text-muted-foreground">
              {query.data.product.product_code}
            </span>
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <DetailRow label="在庫数" value={<StockWithRecheck product={query.data.product} />} />
            <DetailRow
              label="売価"
              value={priceFormatter.format(query.data.product.selling_price)}
            />
            <DetailRow label="原価" value={formatCostYen(query.data.product.cost_price_centi)} />
            <DetailRow label="最終入庫日" value={formatLastDate(query.data.last_receiving_date)} />
            <DetailRow label="最終販売日" value={formatLastDate(query.data.last_sale_date)} />
          </div>
          <StockRecheckNotice
            productCode={query.data.product.product_code}
            stockRecheck={query.data.product.stock_recheck}
          />
          <div className="flex flex-wrap gap-2 pb-4">
            <Button type="button" asChild variant="outline" size="sm">
              <Link to="/products/$code/edit" params={{ code: query.data.product.product_code }}>
                商品修正
              </Link>
            </Button>
            <ActiveCta label="在庫変動履歴" productCode={query.data.product.product_code} />
            <Button type="button" asChild variant="outline" size="sm">
              <Link to="/inventory/receiving">入庫記録</Link>
            </Button>
          </div>
        </CardContent>
      </>
    );
  }

  return null;
}
