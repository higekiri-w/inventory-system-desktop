// src/features/inventory-records/TransferRecordDetailPage.tsx
//
// UI-16b 詰め替え・切り分けの記録の詳細（読取り専用）。
// 設計: docs/function-design/79-ui-stock-transfer.md §79.8、65 TRACE-D14

import { ArrowLeft, PackageSearch } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { EmptyState } from "@/components/patterns/EmptyState";
import { PageHeader } from "@/components/patterns/PageHeader";
import { PageShell } from "@/components/patterns/PageShell";
import { formatStockDisplay } from "@/features/stock-inquiry/lib/format-stock-display";
import { MovementTable, stockUnitMap } from "@/features/stock-movements/components/MovementTable";
import { commands, type TransferDirection, type TransferRecordDetailItem } from "@/lib/bindings";
import { formatDateTime } from "@/lib/date-time";
import { describeError } from "@/lib/describe-error";
import { unwrapResult } from "@/lib/invoke";
import { queryKeys } from "@/lib/query-keys";
import { returnToLinkProps } from "@/lib/return-to";
import { formatRecordStatus } from "./types";

export interface TransferRecordDetailPageProps {
  recordId: number;
  returnTo?: string;
}

const SIDE_HEADINGS: Record<TransferDirection, string> = {
  out: "減らした商品",
  in: "増やした商品",
};

/** §79.8: 取消・訂正の代わりの案内（D-143 (5)） */
export const TRANSFER_CORRECTION_GUIDE =
  "間違えたときは、この記録と同じ日付・同じ数量で、減らす商品と増やす商品を入れ替えて、もう一度詰め替え・切り分けを記録してください";

function SideTable({
  direction,
  items,
}: {
  direction: TransferDirection;
  items: TransferRecordDetailItem[];
}) {
  return (
    <section className="space-y-3 rounded-md border p-4">
      <h2 className="text-xl font-semibold">{SIDE_HEADINGS[direction]}</h2>
      <Table aria-label={SIDE_HEADINGS[direction]}>
        <TableHeader>
          <TableRow>
            <TableHead>商品コード</TableHead>
            <TableHead>商品名</TableHead>
            <TableHead>部門</TableHead>
            <TableHead className="text-right">数量</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {items.map((item) => (
            <TableRow key={item.id}>
              <TableCell className="font-mono font-medium">{item.product_code}</TableCell>
              <TableCell className="min-w-[12rem] whitespace-normal">{item.product_name}</TableCell>
              <TableCell>{item.department_name}</TableCell>
              <TableCell className="text-right tabular-nums">
                {formatStockDisplay(item.quantity, item.stock_unit)}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </section>
  );
}

export function TransferRecordDetailPage({ recordId, returnTo }: TransferRecordDetailPageProps) {
  const backLinkProps = returnToLinkProps(returnTo) ?? { to: "/inventory/records", search: {} };
  const detailQuery = useQuery({
    queryKey: queryKeys.inventoryRecords.transferDetail(recordId),
    queryFn: () =>
      unwrapResult(commands.getTransferRecord(recordId), {
        source: "commands",
        cmd: "get_transfer_record",
      }),
    staleTime: 30_000,
    gcTime: 5 * 60_000,
    retry: 0,
  });

  if (detailQuery.isLoading) {
    return (
      <PageShell>
        <Skeleton className="h-8 w-64" />
        <Skeleton className="h-32 w-full" />
        <Skeleton className="h-48 w-full" />
      </PageShell>
    );
  }

  if (detailQuery.isError) {
    return (
      <PageShell>
        <PageHeader title="詰め替え・切り分け詳細" />
        <Alert variant="destructive">
          <AlertTitle>{describeError(detailQuery.error)}</AlertTitle>
          <AlertDescription>
            記録IDを確認するか、入出庫履歴から開き直してください。
          </AlertDescription>
        </Alert>
        <Button asChild variant="outline">
          <Link {...backLinkProps}>
            <ArrowLeft aria-hidden="true" />
            前の画面へ戻る
          </Link>
        </Button>
      </PageShell>
    );
  }

  const detail = detailQuery.data;
  if (!detail) return null;

  return (
    <PageShell>
      <PageHeader
        title={`詰め替え・切り分け #${String(detail.id)}`}
        actions={
          <Button asChild variant="outline">
            <Link {...backLinkProps}>
              <ArrowLeft aria-hidden="true" />
              前の画面へ戻る
            </Link>
          </Button>
        }
      />

      <section className="rounded-md border p-4">
        <div className="grid gap-3 text-sm sm:grid-cols-4">
          <div>
            <span className="text-muted-foreground">記録ID</span>
            <div className="font-medium">{detail.id}</div>
          </div>
          <div>
            <span className="text-muted-foreground">詰め替え・切り分けた日</span>
            <div className="font-medium">{detail.transfer_date}</div>
          </div>
          <div>
            <span className="text-muted-foreground">記録日時</span>
            <div className="font-mono font-medium tabular-nums">
              {formatDateTime(detail.created_at)}
            </div>
          </div>
          <div>
            <span className="text-muted-foreground">状態</span>
            <div>
              <Badge variant="outline">{formatRecordStatus(detail.status)}</Badge>
            </div>
          </div>
        </div>
      </section>

      <Alert>
        <AlertTitle>取消はできません</AlertTitle>
        <AlertDescription>{TRANSFER_CORRECTION_GUIDE}</AlertDescription>
      </Alert>

      {(["out", "in"] as const).map((direction) => (
        <SideTable
          key={direction}
          direction={direction}
          items={detail.items.filter((item) => item.direction === direction)}
        />
      ))}

      <section className="space-y-3 rounded-md border p-4">
        <h2 className="text-xl font-semibold">関連する在庫変動</h2>
        {detail.movements.length === 0 ? (
          <EmptyState
            icon={PackageSearch}
            title="関連する在庫変動がありません"
            description="この記録に紐づく在庫変動は見つかりません"
          />
        ) : (
          <MovementTable
            movements={detail.movements}
            stockUnitByProductCode={stockUnitMap(detail.items)}
          />
        )}
      </section>
    </PageShell>
  );
}
