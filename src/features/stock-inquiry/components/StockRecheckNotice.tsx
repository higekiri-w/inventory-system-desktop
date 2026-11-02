// src/features/stock-inquiry/components/StockRecheckNotice.tsx
//
// D-118 b2: 要再確認の商品の直し方の案内（在庫照会の詳細と在庫変動の見出し。58・66 の D-118 の注記）。
// 案内を出すかは写しの stock_recheck で決め、要再確認の一覧（b1 の listRecheckProducts）は理由を補うだけ
// （読込み中・失敗・当該の target が無いときは理由を出さず、案内と link は出す）。

import { Link } from "@tanstack/react-router";
import { TriangleAlertIcon } from "lucide-react";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { useRecheckProducts } from "@/features/stocktake/hooks/useRecheckProducts";
import { formatRecountReasons } from "@/features/stocktake/lib/stocktake-formatters";

export interface StockRecheckNoticeProps {
  productCode: string;
  stockRecheck: boolean;
}

export function StockRecheckNotice({ productCode, stockRecheck }: StockRecheckNoticeProps) {
  // 要再確認でない商品は一覧を読まない
  return stockRecheck ? <RecheckNotice productCode={productCode} /> : null;
}

function RecheckNotice({ productCode }: { productCode: string }) {
  const recheckQuery = useRecheckProducts();
  // 理由は取得が成功し取得中でないときだけ（data は成功か、再取得の失敗の後の cache にだけある。
  // 失敗・取得中に cache の古い理由を出さない）
  const fresh = !recheckQuery.isError && !recheckQuery.isFetching;
  const target = fresh
    ? recheckQuery.data?.find((item) => item.product_code === productCode)
    : undefined;
  const reasons = target ? formatRecountReasons(target.recount_reasons) : [];
  return (
    <Alert variant="warning">
      <TriangleAlertIcon aria-hidden="true" />
      <AlertTitle>数え直しが必要です</AlertTitle>
      <AlertDescription>
        {reasons.map((reason) => (
          <p key={reason}>{reason}</p>
        ))}
        <p>棚卸しの画面で、棚の数を数えて入れ直してください。</p>
        <Link to="/stocktake" search={{ product: productCode }} className="underline">
          棚卸しの画面で入れ直す
        </Link>
      </AlertDescription>
    </Alert>
  );
}
