// src/features/products/components/ProductForm.tsx
//
// UI-01b-D4〜D8: 商品登録・修正 form。
// UI-01b-D10: 識別 / 分類と取引先 / 価格 / 在庫 の 4 セクションに分割する。
// UI-01b-D11: read-only 入力は readOnly + bg-muted で示す（長さの単位の商品の数量単位 select は disabled）。
// UI-01b-D12: 必須項目ラベルに（必須）を付ける（色符号化しない）。
// UI-01b-D13: 「廃番にする」は確認ダイアログを通す（「表示に戻す」は直接実行）。

import React, { useEffect, useState } from "react";
import { ArrowLeft, ChevronDown, Save } from "lucide-react";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { FormSection } from "@/components/patterns/FormSection";
import {
  isComposedDigitsOnly,
  normalizeComposedDigits,
} from "@/components/patterns/normalizeComposedDigits";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { commands, type Department, type Supplier } from "@/lib/bindings";
import { unwrapResult } from "@/lib/invoke";
import {
  SupplierPickerDialog,
  supplierCurrentLabel,
} from "@/features/suppliers/components/SupplierPickerDialog";
import { formatCostInput, isLengthUnit, unitCostFromPack } from "@/lib/unit-quantity";
import { suggestPluTarget } from "../lib/jan-code";
import type { ProductFormValues, ProductTaxRate } from "../lib/product-form-request";
import { DiscontinueConfirmDialog } from "./DiscontinueConfirmDialog";
import { StockRecheckBadge } from "@/features/stock-inquiry/components/StockRecheckBadge";
import { StockUnitField } from "./StockUnitField";
import { PriceHistorySection } from "./PriceHistorySection";

export interface ProductFormProps {
  mode: "create" | "edit";
  values: ProductFormValues;
  departments: Department[];
  suppliers: Supplier[];
  errors: Partial<Record<keyof ProductFormValues, string>>;
  saveError: string | null;
  supplierWarning: string | null;
  productCodeLabel?: string;
  productName?: string;
  isDiscontinued?: boolean;
  /** 修正の画面の「現在庫」の隣に「要再確認」の badge を出す（写しの stock_recheck。51） */
  stockRecheck?: boolean;
  /** 在庫の節の「数量単位」「POS販売で在庫を減らす」の行のすぐ下に出す案内（在庫連動を始める拒否。51） */
  stockSyncNotice?: React.ReactNode;
  isSaving: boolean;
  isTogglePending?: boolean;
  saveDisabled?: boolean;
  posSyncTouched: boolean;
  pluTargetTouched?: boolean;
  showPluTargetEnableNote?: boolean;
  onValuesChange: React.Dispatch<React.SetStateAction<ProductFormValues>>;
  onPosSyncTouchedChange: (touched: boolean) => void;
  onPluTargetTouchedChange?: (touched: boolean) => void;
  onSubmit: () => void;
  onCancel: () => void;
  onToggleDiscontinue?: () => void;
}

function FieldError({ message }: { message: string | undefined }) {
  return message === undefined ? null : (
    <p className="text-sm text-destructive" role="alert">
      {message}
    </p>
  );
}

export function ProductForm({
  mode,
  values,
  departments,
  suppliers,
  errors,
  saveError,
  supplierWarning,
  productCodeLabel,
  productName,
  isDiscontinued = false,
  stockRecheck = false,
  stockSyncNotice,
  isSaving,
  isTogglePending = false,
  saveDisabled = false,
  posSyncTouched,
  pluTargetTouched = false,
  showPluTargetEnableNote = false,
  onValuesChange,
  onPosSyncTouchedChange,
  onPluTargetTouchedChange,
  onSubmit,
  onCancel,
  onToggleDiscontinue,
}: ProductFormProps) {
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [supplierOptions, setSupplierOptions] = useState(suppliers);
  const [supplierPickerOpen, setSupplierPickerOpen] = useState(false);

  useEffect(() => {
    setSupplierOptions((current) => {
      const merged = [...suppliers];
      for (const supplier of current) {
        if (!merged.some((candidate) => candidate.id === supplier.id)) merged.push(supplier);
      }
      if (
        merged.length === current.length &&
        merged.every(
          (supplier, index) =>
            supplier.id === current[index]?.id && supplier.name === current[index]?.name,
        )
      ) {
        return current;
      }
      return merged;
    });
  }, [suppliers]);

  const update = <K extends keyof ProductFormValues>(key: K, value: ProductFormValues[K]) => {
    onValuesChange((prev) => ({ ...prev, [key]: value }));
  };

  // 長さの単位の価格は 1 m あたり（UI-01b-D22）
  const perMeter = isLengthUnit(values.stockUnit) ? "、1 m あたり" : "";

  const handleToggleClick = () => {
    if (isDiscontinued) {
      // 表示に戻すは直接実行（UI-01b-D13）
      onToggleDiscontinue?.();
    } else {
      setConfirmOpen(true);
    }
  };

  return (
    <form
      className="space-y-8"
      onSubmit={(event) => {
        event.preventDefault();
        onSubmit();
      }}
    >
      {saveError !== null ? (
        <Alert variant="destructive">
          <AlertTitle>保存できませんでした</AlertTitle>
          <AlertDescription>{saveError}</AlertDescription>
        </Alert>
      ) : null}
      {supplierWarning !== null ? (
        <Alert>
          <AlertTitle>取引先候補を取得できませんでした</AlertTitle>
          <AlertDescription>{supplierWarning}</AlertDescription>
        </Alert>
      ) : null}

      {mode === "edit" ? (
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant={isDiscontinued ? "secondary" : "outline"}>
            {isDiscontinued ? "廃番" : "表示中"}
          </Badge>
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={isTogglePending}
            onClick={handleToggleClick}
          >
            {isDiscontinued ? "表示に戻す" : "廃番にする"}
          </Button>
        </div>
      ) : null}

      <FormSection title="商品の識別" description="商品コードとJANコードは登録後に変更できません。">
        <div className="grid gap-4 md:grid-cols-2">
          <div className="space-y-1">
            <Label htmlFor="product-code">商品コード</Label>
            <Input
              id="product-code"
              className="bg-muted"
              value={mode === "create" ? "保存時に自動決定" : (productCodeLabel ?? "")}
              readOnly
            />
          </div>
          <div className="space-y-1">
            <Label htmlFor="jan-code">JANコード</Label>
            <Input
              id="jan-code"
              className={mode === "edit" ? "bg-muted" : undefined}
              value={values.janCode}
              readOnly={mode === "edit"}
              onChange={(event) => {
                const inputValue = event.target.value;
                const nextJanCode =
                  !(event.nativeEvent as InputEvent).isComposing && isComposedDigitsOnly(inputValue)
                    ? normalizeComposedDigits(inputValue)
                    : inputValue;
                onValuesChange((prev) => ({
                  ...prev,
                  janCode: nextJanCode,
                  pluTarget:
                    mode === "create" && !pluTargetTouched
                      ? suggestPluTarget(nextJanCode)
                      : prev.pluTarget,
                }));
              }}
              onCompositionEnd={(event) => {
                const inputValue = event.currentTarget.value;
                const nextJanCode = isComposedDigitsOnly(inputValue)
                  ? normalizeComposedDigits(inputValue)
                  : inputValue;
                onValuesChange((prev) => ({
                  ...prev,
                  janCode: nextJanCode,
                  pluTarget:
                    mode === "create" && !pluTargetTouched
                      ? suggestPluTarget(nextJanCode)
                      : prev.pluTarget,
                }));
              }}
            />
            <FieldError message={errors.janCode} />
          </div>
        </div>

        <div className="space-y-2 rounded-md border border-input p-3">
          <div className="flex items-start gap-3">
            <Checkbox
              id="plu-target"
              checked={values.pluTarget}
              onCheckedChange={(checked) => {
                onPluTargetTouchedChange?.(true);
                update("pluTarget", checked === true);
              }}
            />
            <div className="grid gap-1 leading-none">
              <Label htmlFor="plu-target">レジにバーコード登録する</Label>
              <p className="text-sm text-muted-foreground">
                スキャニングPLUに書き出す商品だけオンにします。
              </p>
            </div>
          </div>
          {showPluTargetEnableNote ? (
            <p className="text-sm text-warning-strong">
              オンにして保存すると、この商品はPLU未反映として差分書出しに表示されます。
            </p>
          ) : null}
        </div>

        <div className="grid gap-4 md:grid-cols-2">
          <div className="space-y-1">
            <Label htmlFor="product-name">商品名（必須）</Label>
            <Input
              id="product-name"
              value={values.name}
              onChange={(event) => {
                update("name", event.target.value);
              }}
            />
            <FieldError message={errors.name} />
          </div>
          <div className="space-y-1">
            <Label htmlFor="maker-code">メーカー品番</Label>
            <Input
              id="maker-code"
              value={values.makerCode}
              onChange={(event) => {
                update("makerCode", event.target.value);
              }}
            />
          </div>
        </div>
      </FormSection>

      <FormSection
        title="分類と取引先"
        description="部門は必須です。取引先は任意で、後から変更できます。"
      >
        <div className="grid gap-4 md:grid-cols-2">
          <div className="space-y-1">
            <Label htmlFor="department-id">部門（必須）</Label>
            <Select
              value={values.departmentId === null ? "" : String(values.departmentId)}
              onValueChange={(value) => {
                update("departmentId", Number(value));
              }}
            >
              <SelectTrigger id="department-id" className="w-full">
                <SelectValue placeholder="選択してください" />
              </SelectTrigger>
              <SelectContent>
                {departments.map((department) => (
                  <SelectItem key={department.id} value={String(department.id)}>
                    {department.name}
                    {department.code_prefix !== null ? "（独自コード可）" : ""}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <FieldError message={errors.departmentId} />
          </div>
          <div className="space-y-1">
            <Label id="supplier-label" htmlFor="supplier-id">
              取引先
            </Label>
            <Button
              type="button"
              variant="outline"
              id="supplier-id"
              className="w-full justify-between bg-control-surface"
              aria-haspopup="dialog"
              aria-labelledby="supplier-label supplier-id"
              disabled={supplierWarning !== null}
              onClick={() => {
                setSupplierPickerOpen(true);
              }}
            >
              <span className="truncate">
                {supplierCurrentLabel(supplierOptions, values.supplierId ?? null, "取引先なし")}
              </span>
              <ChevronDown className="size-4 text-muted-foreground opacity-50" aria-hidden="true" />
            </Button>
            <SupplierPickerDialog
              open={supplierPickerOpen}
              onOpenChange={setSupplierPickerOpen}
              suppliers={supplierOptions}
              isLoading={false}
              isError={supplierWarning !== null}
              leadingLabel="取引先なし"
              selected={values.supplierId ?? null}
              onRetry={() => {
                /* 取得失敗中は host の trigger が disabled のため到達しない。 */
              }}
              onSelect={(id) => {
                update("supplierId", id);
              }}
              onCreated={async () => {
                // QueryClient を持たない form のため、既存の local 候補一覧を再取得する。
                const refreshed = await unwrapResult(commands.listSuppliers(), {
                  source: "commands",
                  cmd: "list_suppliers",
                });
                setSupplierOptions(refreshed);
              }}
            />
          </div>
        </div>
      </FormSection>

      <FormSection
        title="価格"
        description="売価は税込の円の整数、原価は税抜の円で小数 2 桁まで入力します。長さ（m）の商品は 1 m あたりの値段です。"
      >
        <div className="grid gap-4 md:grid-cols-3">
          <div className="space-y-1">
            <Label htmlFor="selling-price">売価（必須{perMeter}）</Label>
            <Input
              id="selling-price"
              inputMode="numeric"
              value={values.sellingPrice}
              onChange={(event) => {
                update("sellingPrice", event.target.value);
              }}
            />
            <FieldError message={errors.sellingPrice} />
          </div>
          <div className="space-y-1">
            <Label htmlFor="cost-price">原価（税抜、必須{perMeter}）</Label>
            <Input
              id="cost-price"
              inputMode="decimal"
              value={values.costPrice}
              onChange={(event) => {
                update("costPrice", event.target.value);
              }}
            />
            <FieldError message={errors.costPrice} />
            {/* UI-01b-D24: 個数の単位の商品だけ。長さの単位にすると閉じて隠れ、開き直すと空から始まる */}
            {isLengthUnit(values.stockUnit) ? null : (
              <CostPackHelper
                onApply={(centi) => {
                  update("costPrice", formatCostInput(centi));
                }}
              />
            )}
          </div>
          <div className="space-y-1">
            <Label htmlFor="tax-rate">税率</Label>
            <Select
              value={values.taxRate}
              onValueChange={(value) => {
                update("taxRate", value as ProductTaxRate);
              }}
            >
              <SelectTrigger id="tax-rate" className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="10">10%</SelectItem>
                <SelectItem value="8">8%</SelectItem>
                <SelectItem value="0">0%</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
      </FormSection>

      <FormSection
        title="在庫"
        description={
          mode === "create"
            ? "初期在庫と数量単位を設定します。登録後は、数量単位を個数の単位どうし（例: 個 → 玉）でだけ変更できます。"
            : isLengthUnit(values.stockUnit)
              ? "現在庫と数量単位は登録後に変更できません。"
              : "現在庫は変更できません。数量単位は個数の単位どうしでだけ変更できます。"
        }
      >
        <div className="space-y-1 md:w-1/2">
          <div className="flex items-center gap-2">
            <Label htmlFor="initial-stock">
              {mode === "create" ? "初期在庫（必須）" : "現在庫"}
            </Label>
            {mode === "edit" && stockRecheck ? <StockRecheckBadge /> : null}
          </div>
          <Input
            id="initial-stock"
            className={mode === "edit" ? "bg-muted" : undefined}
            inputMode={mode === "create" && values.stockUnit === "m" ? "decimal" : "numeric"}
            value={values.initialStock}
            readOnly={mode === "edit"}
            onChange={(event) => {
              update("initialStock", event.target.value);
            }}
          />
          <FieldError message={errors.initialStock} />
        </div>

        <StockUnitField
          mode={mode}
          stockUnit={values.stockUnit}
          posStockSync={values.posStockSync}
          posSyncTouched={posSyncTouched}
          readOnly={mode === "edit" && isLengthUnit(values.stockUnit)}
          onStockUnitChange={(stockUnit) => {
            update("stockUnit", stockUnit);
          }}
          onPosStockSyncChange={(checked) => {
            onPosSyncTouchedChange(true);
            update("posStockSync", checked);
          }}
          onPosStockSyncSuggest={(checked) => {
            // 自動提案経路: touched は立てず値だけ更新する（UI-01b-D6）
            update("posStockSync", checked);
          }}
        />
        {stockSyncNotice}
      </FormSection>

      {mode === "edit" && productCodeLabel !== undefined ? (
        <PriceHistorySection productCode={productCodeLabel} />
      ) : null}

      <div className="flex flex-wrap justify-end gap-2 border-t pt-4">
        <Button type="button" variant="outline" onClick={onCancel}>
          <ArrowLeft aria-hidden="true" />
          一覧へ戻る
        </Button>
        <Button type="submit" disabled={isSaving || saveDisabled}>
          <Save aria-hidden="true" />
          {mode === "create" ? "登録する" : "保存する"}
        </Button>
      </div>

      <DiscontinueConfirmDialog
        open={confirmOpen}
        productName={productName ?? values.name}
        onConfirm={() => {
          setConfirmOpen(false);
          onToggleDiscontinue?.();
        }}
        onCancel={() => {
          setConfirmOpen(false);
        }}
      />
    </form>
  );
}

/** 原価の「袋の値段から出す」の補助（UI-01b-D24）。2 つの欄の値は保存せず、閉じると捨てる */
function CostPackHelper({ onApply }: { onApply: (centi: number) => void }) {
  const [open, setOpen] = useState(false);
  const [packPrice, setPackPrice] = useState("");
  const [packCount, setPackCount] = useState("");
  const [error, setError] = useState<string | null>(null);

  const toggle = () => {
    setOpen((current) => !current);
    setPackPrice("");
    setPackCount("");
    setError(null);
  };

  return (
    <div className="space-y-2 pt-1">
      <Button type="button" variant="outline" size="sm" aria-expanded={open} onClick={toggle}>
        袋の値段から出す
      </Button>
      {open ? (
        <div className="space-y-2 rounded-md border p-3">
          <div className="space-y-1">
            <Label htmlFor="cost-pack-price">袋の値段（円）</Label>
            <Input
              id="cost-pack-price"
              inputMode="decimal"
              value={packPrice}
              onChange={(event) => {
                setPackPrice(event.target.value);
              }}
            />
          </div>
          <div className="space-y-1">
            <Label htmlFor="cost-pack-count">入り数</Label>
            <Input
              id="cost-pack-count"
              inputMode="numeric"
              value={packCount}
              onChange={(event) => {
                setPackCount(event.target.value);
              }}
            />
          </div>
          <FieldError message={error ?? undefined} />
          <Button
            type="button"
            size="sm"
            onClick={() => {
              const result = unitCostFromPack(packPrice, packCount);
              setError(result.error);
              if (result.value !== null) onApply(result.value);
            }}
          >
            原価に入れる
          </Button>
        </div>
      ) : null}
    </div>
  );
}
