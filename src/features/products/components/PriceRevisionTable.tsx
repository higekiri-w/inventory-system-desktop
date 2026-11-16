import { useState } from "react";
import { AlertCircle, Check, Clock3, Loader2 } from "lucide-react";
import { toast } from "sonner";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { PriceRevisionInput } from "@/lib/bindings";
import { formatCostYen } from "@/features/stock-inquiry/lib/format-stock-display";
import { formatCostInput, MAX_SELLING_PRICE, parseCostInput } from "@/lib/unit-quantity";
import type { PriceRevisionRow } from "../hooks/usePriceRevisionList";
import { useReviseProductPrice } from "../hooks/useReviseProductPrice";
import { deriveProposedCost, formatMarkupRate, isRevisedToday } from "../lib/price-revision-math";

function yen(value: number): string {
  return `¥${value.toLocaleString("ja-JP")}`;
}

function parsePrice(value: string): number | null {
  if (value.trim() === "") return null;
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed >= 0 ? parsed : null;
}

function PriceRevisionRowView({
  row,
  selectedSupplierId,
  assignSupplier,
  todayYmd,
}: {
  row: PriceRevisionRow;
  selectedSupplierId: number | undefined;
  assignSupplier: boolean;
  todayYmd: string;
}) {
  const product = row.product;
  const [selling, setSelling] = useState("");
  const [cost, setCost] = useState("");
  const [costTouched, setCostTouched] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const sellingPrice = parsePrice(selling);
  // 原価は小数 2 桁までの円 → 1/100 円（SPEC-UNIT-D5）
  const parsedCost = parseCostInput(cost);
  const costPrice = typeof parsedCost === "number" ? parsedCost : null;
  // 上限を越えた欄は error を出し、直すまで確定できない（自動で入る案も同じ。X5・X6）
  const sellingTooLarge = sellingPrice !== null && sellingPrice > MAX_SELLING_PRICE;
  const costTooLarge = parsedCost === "too_large";
  const sellingInvalid = selling !== "" && sellingPrice === null;
  const costInvalid = cost !== "" && parsedCost === null;

  const mutation = useReviseProductPrice({
    onSuccess: () => {
      toast.success(`${product.name}の価格を改定しました`, {
        duration: 5000,
        id: `price-revision-success-${product.product_code}`,
      });
      setSelling("");
      setCost("");
      setCostTouched(false);
      setError(null);
    },
    onError: (caught) => {
      setError(caught.message);
    },
  });

  const payload = (): PriceRevisionInput | null => {
    if (sellingPrice === null || costPrice === null || sellingTooLarge || costTooLarge) return null;
    return {
      product_code: product.product_code,
      new_selling_price: sellingPrice,
      new_cost_price_centi: costPrice,
      assign_supplier_id:
        selectedSupplierId !== undefined && assignSupplier ? selectedSupplierId : null,
    };
  };
  const submit = () => {
    const input = payload();
    if (input !== null) {
      setError(null);
      mutation.mutate(input);
    }
  };

  const recentlyRevised = isRevisedToday(row.latestChangedAt, todayYmd);
  const editing = selling !== "" || cost !== "";

  return (
    <TableRow data-testid={`price-row-${product.product_code}`}>
      <TableCell className="font-mono font-medium">{product.product_code}</TableCell>
      <TableCell className="font-mono">{product.jan_code ?? "—"}</TableCell>
      <TableCell>{product.maker_code ?? "—"}</TableCell>
      <TableCell className="min-w-48">
        <div className="flex items-center gap-2">
          <span>{product.name}</span>
          {recentlyRevised ? (
            <Badge variant="secondary">
              <Clock3 aria-hidden="true" />
              最近改定
            </Badge>
          ) : null}
          {editing && !mutation.isPending && error === null ? (
            <Badge variant="outline">入力中</Badge>
          ) : null}
        </div>
      </TableCell>
      <TableCell className="text-right tabular-nums">{yen(product.selling_price)}</TableCell>
      <TableCell className="text-right tabular-nums">
        {formatCostYen(product.cost_price_centi)}
      </TableCell>
      <TableCell className="text-right tabular-nums">
        {formatMarkupRate(product.cost_price_centi, product.selling_price)}
      </TableCell>
      <TableCell className="min-w-32 align-top">
        <Input
          type="number"
          min={0}
          step={1}
          aria-label={`${product.product_code} 新売価`}
          value={selling}
          disabled={mutation.isPending}
          aria-invalid={sellingInvalid || sellingTooLarge}
          onChange={(event) => {
            const value = event.target.value;
            setSelling(value);
            setError(null);
            const parsed = parsePrice(value);
            if (parsed !== null && !costTouched) {
              // 新原価（案）は入力の欄の初期値の文字列（9100 → `91`。SPEC-UNIT-D5）
              setCost(
                formatCostInput(
                  deriveProposedCost(parsed, product.cost_price_centi, product.selling_price),
                ),
              );
            } else if (value === "" && !costTouched) {
              setCost("");
            }
          }}
        />
        {sellingInvalid ? (
          <p className="mt-1 text-xs text-destructive">0以上の整数で入力してください</p>
        ) : null}
        {sellingTooLarge ? (
          <p className="mt-1 text-xs text-destructive">売価が大きすぎます</p>
        ) : null}
      </TableCell>
      <TableCell className="min-w-32 align-top">
        <Input
          type="number"
          inputMode="decimal"
          min={0}
          step="0.01"
          aria-label={`${product.product_code} 新原価（税抜、案）`}
          value={cost}
          disabled={mutation.isPending}
          aria-invalid={costInvalid || costTooLarge}
          onChange={(event) => {
            setCost(event.target.value);
            setCostTouched(true);
            setError(null);
          }}
        />
        {costInvalid ? (
          <p className="mt-1 text-xs text-destructive">
            原価は0以上で、小数は2桁までで入力してください
          </p>
        ) : null}
        {costTooLarge ? <p className="mt-1 text-xs text-destructive">原価が大きすぎます</p> : null}
      </TableCell>
      <TableCell className="min-w-44 align-top">
        <Button
          type="button"
          size="sm"
          disabled={payload() === null || mutation.isPending}
          aria-label={
            mutation.isPending
              ? `${product.product_code} を確定中`
              : `${product.product_code} を確定`
          }
          onClick={submit}
        >
          {mutation.isPending ? (
            <Loader2 aria-hidden="true" className="animate-spin" />
          ) : (
            <Check aria-hidden="true" />
          )}
          {mutation.isPending ? "確定中" : "確定"}
        </Button>
        {error !== null ? (
          <div className="mt-2 space-y-1 text-xs text-destructive" role="alert">
            <p className="flex items-center gap-1">
              <AlertCircle aria-hidden="true" />
              確定できませんでした
            </p>
            <p>{error}</p>
            <Button
              type="button"
              size="sm"
              variant="outline"
              aria-label={`${product.product_code} を再試行`}
              onClick={submit}
            >
              再試行
            </Button>
          </div>
        ) : null}
      </TableCell>
    </TableRow>
  );
}

export function PriceRevisionTable({
  rows,
  selectedSupplierId,
  assignSupplier,
}: {
  rows: PriceRevisionRow[];
  selectedSupplierId: number | undefined;
  assignSupplier: boolean;
}) {
  const todayYmd = new Date().toLocaleDateString("sv-SE");
  return (
    <Table className="min-w-[1280px]">
      <TableHeader>
        <TableRow>
          {[
            "商品コード",
            "JAN",
            "メーカー品番",
            "商品名",
            "現売価",
            "現原価",
            "現掛率",
            "新売価",
            "新原価（税抜、案）",
            "確定",
          ].map((label) => (
            <TableHead key={label}>{label}</TableHead>
          ))}
        </TableRow>
      </TableHeader>
      <TableBody>
        {rows.map((row) => (
          <PriceRevisionRowView
            key={row.product.product_code}
            row={row}
            selectedSupplierId={selectedSupplierId}
            assignSupplier={assignSupplier}
            todayYmd={todayYmd}
          />
        ))}
      </TableBody>
    </Table>
  );
}
