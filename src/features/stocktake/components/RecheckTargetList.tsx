// src/features/stocktake/components/RecheckTargetList.tsx
//
// 要再確認の商品の一覧（73 の棚卸しの画面の上部、55 の取込みと取消の結果）。
// 各行は商品・範囲（単位どおり）・理由。状態は文字と icon で示し、色だけにしない。
// `renderAction` を渡さないとき（取消の確認の中。55 UI-07-D18）は「操作」列と入れ直しの文を出さない。

import { AlertTriangle } from "lucide-react";
import type { ReactNode } from "react";

import { Badge } from "@/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { CountRecoveryTarget } from "@/lib/bindings";

import { formatRecountReasons, formatStockOrRange } from "../lib/stocktake-formatters";

export interface RecheckTargetListProps {
  title: string;
  targets: readonly CountRecoveryTarget[];
  renderAction?: (target: CountRecoveryTarget) => ReactNode;
}

export function RecheckTargetList({ title, targets, renderAction }: RecheckTargetListProps) {
  if (targets.length === 0) return null;
  return (
    <section aria-label={title} className="space-y-2 rounded-md border border-warning-border p-4">
      <h2 className="flex items-center gap-1.5 text-lg font-semibold">
        <AlertTriangle aria-hidden="true" className="size-5" />
        {title}
      </h2>
      {renderAction ? <p className="text-sm">棚の数を確かめて入れ直してください。</p> : null}
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>商品</TableHead>
            <TableHead className="text-right">在庫</TableHead>
            <TableHead>理由</TableHead>
            {renderAction ? <TableHead className="text-right">操作</TableHead> : null}
          </TableRow>
        </TableHeader>
        <TableBody>
          {targets.map((target) => (
            <TableRow key={target.product_code}>
              <TableCell>
                <span className="font-medium">{target.product_name}</span>
                <span className="ml-2 font-mono text-sm text-muted-foreground">
                  {target.product_code}
                </span>
              </TableCell>
              <TableCell className="text-right tabular-nums">
                {formatStockOrRange(target)}
                {target.stock_range_low !== null ? (
                  <Badge variant="outline" tone="warning" className="ml-2">
                    <AlertTriangle aria-hidden="true" />
                    要再確認
                  </Badge>
                ) : null}
              </TableCell>
              <TableCell className="whitespace-normal">
                {formatRecountReasons(target.recount_reasons).join("。")}
              </TableCell>
              {renderAction ? (
                <TableCell className="text-right">{renderAction(target)}</TableCell>
              ) : null}
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </section>
  );
}
