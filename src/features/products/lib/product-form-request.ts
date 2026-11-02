// src/features/products/lib/product-form-request.ts
//
// UI-01b-D4/D5/D6: form state から generated command payload を作る。

import type {
  Department,
  ProductCreateRequest,
  ProductStockUnit,
  ProductTaxRate,
  ProductUpdateRequest_Deserialize,
  ProductWithRelations,
} from "@/lib/bindings";
import { formatStockOrRange } from "@/features/stocktake/lib/stocktake-formatters";
import {
  formatCostInput,
  isLengthUnit,
  MAX_SELLING_PRICE,
  MAX_STOCK_QUANTITY,
  parseCostInput,
  parsePriceInput,
  parseQuantityInput,
  type QuantityField,
} from "@/lib/unit-quantity";
import { normalizeJanCodeCandidate, validateJanCode } from "./jan-code";

export type { ProductStockUnit, ProductTaxRate };

export interface ProductFormValues {
  janCode: string;
  name: string;
  departmentId: number | null;
  sellingPrice: string;
  costPrice: string;
  taxRate: ProductTaxRate;
  stockUnit: ProductStockUnit;
  initialStock: string;
  makerCode: string;
  supplierId: number | null;
  posStockSync: boolean;
  pluTarget: boolean;
}

export interface ProductFormBuildResult<T> {
  request: T | null;
  errors: Partial<Record<keyof ProductFormValues, string>>;
}

export const createProductFormDefaults: ProductFormValues = {
  janCode: "",
  name: "",
  departmentId: null,
  sellingPrice: "0",
  costPrice: "0",
  taxRate: "10",
  stockUnit: "pcs",
  initialStock: "0",
  makerCode: "",
  supplierId: null,
  posStockSync: true,
  pluTarget: false,
};

// 商品登録の初期在庫の欄（0 を受ける。`m` の商品は m で受けて cm にする。X3、SPEC-UNIT-D3）
const INITIAL_STOCK_FIELD: QuantityField = {
  min: 0,
  max: MAX_STOCK_QUANTITY,
  tooLarge: "在庫数が大きすぎます",
  integerError: "初期在庫は0以上の整数で入力してください",
};

type FieldResult = { value: number; error: null } | { value: null; error: string };

// 売価の欄（税込の円の整数、0 以上で上限まで。X5）
function readSellingPrice(text: string): FieldResult {
  const parsed = parsePriceInput(text, MAX_SELLING_PRICE);
  if (parsed === null) return { value: null, error: "売価は0以上の整数で入力してください" };
  if (parsed === "too_large") return { value: null, error: "売価が大きすぎます" };
  return { value: parsed, error: null };
}

// 原価の欄（税抜の円の小数 2 桁まで → 1/100 円。SPEC-UNIT-D5・X6）
function readCost(text: string): FieldResult {
  const parsed = parseCostInput(text);
  if (parsed === null)
    return { value: null, error: "原価は0以上で、小数は2桁までで入力してください" };
  if (parsed === "too_large") return { value: null, error: "原価が大きすぎます" };
  return { value: parsed, error: null };
}

function trimToNullable(value: string): string | null {
  const trimmed = value.trim();
  return trimmed === "" ? null : trimmed;
}

export function productToFormValues(product: ProductWithRelations): ProductFormValues {
  return {
    janCode: product.jan_code ?? "",
    name: product.name,
    departmentId: product.department_id,
    sellingPrice: String(product.selling_price),
    costPrice: formatCostInput(product.cost_price_centi),
    taxRate: product.tax_rate === "8" || product.tax_rate === "0" ? product.tax_rate : "10",
    stockUnit: product.stock_unit,
    // 修正の画面では read-only の「現在庫」として単位つきで出す（検証も送信もしない）。
    // 要再確認なら範囲（b1 の formatStockOrRange。51 の「在庫数の表示」）
    initialStock: formatStockOrRange({
      stock: product.stock_quantity,
      stock_range_low: product.stock_range_low,
      stock_range_high: product.stock_range_high,
      stock_unit: product.stock_unit,
    }),
    makerCode: product.maker_code ?? "",
    supplierId: product.supplier_id,
    posStockSync: product.pos_stock_sync,
    pluTarget: product.plu_target,
  };
}

export function buildCreateProductRequest(
  values: ProductFormValues,
  departments: Department[],
): ProductFormBuildResult<ProductCreateRequest> {
  const errors: ProductFormBuildResult<ProductCreateRequest>["errors"] = {};
  const sellingPrice = readSellingPrice(values.sellingPrice);
  const costPrice = readCost(values.costPrice);
  const initialStock = parseQuantityInput(
    values.initialStock,
    values.stockUnit,
    INITIAL_STOCK_FIELD,
  );
  const selectedDepartment = departments.find(
    (department) => department.id === values.departmentId,
  );
  const janCode = trimToNullable(normalizeJanCodeCandidate(values.janCode));

  if (values.name.trim() === "") errors.name = "商品名を入力してください";
  if (values.departmentId === null || selectedDepartment === undefined) {
    errors.departmentId = "部門を選択してください";
  }
  if (sellingPrice.error !== null) errors.sellingPrice = sellingPrice.error;
  if (costPrice.error !== null) errors.costPrice = costPrice.error;
  if (initialStock.error !== null) errors.initialStock = initialStock.error;
  if (janCode === null && selectedDepartment?.code_prefix === null) {
    errors.janCode = "JANコードを入力するか、独自コード発番対象の部門を選択してください";
  }
  if (janCode !== null) {
    const janCodeError = validateJanCode(janCode);
    if (janCodeError !== null) errors.janCode = janCodeError;
  }

  if (
    Object.keys(errors).length > 0 ||
    sellingPrice.value === null ||
    costPrice.value === null ||
    initialStock.value === null ||
    values.departmentId === null
  ) {
    return { request: null, errors };
  }
  const departmentId = values.departmentId;

  return {
    request: {
      jan_code: janCode,
      name: values.name.trim(),
      department_id: departmentId,
      selling_price: sellingPrice.value,
      cost_price_centi: costPrice.value,
      tax_rate: values.taxRate,
      stock_unit: values.stockUnit,
      initial_stock: initialStock.value,
      maker_code: trimToNullable(values.makerCode),
      supplier_id: values.supplierId,
      pos_stock_sync: values.posStockSync,
      plu_target: values.pluTarget,
      // UI-01b-D26: true は JAN を他の商品が使っているときのダイアログからの送り直しだけ
      shared_jan_custom_code: false,
    },
    errors,
  };
}

export function buildUpdateProductRequest(
  values: ProductFormValues,
  original: ProductWithRelations,
): ProductFormBuildResult<ProductUpdateRequest_Deserialize> {
  const errors: ProductFormBuildResult<ProductUpdateRequest_Deserialize>["errors"] = {};
  const sellingPrice = readSellingPrice(values.sellingPrice);
  const costPrice = readCost(values.costPrice);
  const request: ProductUpdateRequest_Deserialize = {};

  if (values.name.trim() === "") errors.name = "商品名を入力してください";
  if (values.departmentId === null) errors.departmentId = "部門を選択してください";
  if (sellingPrice.error !== null) errors.sellingPrice = sellingPrice.error;
  if (costPrice.error !== null) errors.costPrice = costPrice.error;

  if (Object.keys(errors).length > 0 || sellingPrice.value === null || costPrice.value === null) {
    return { request: null, errors };
  }

  if (values.name.trim() !== original.name) request.name = values.name.trim();
  if (values.departmentId !== original.department_id) request.department_id = values.departmentId;
  if (values.supplierId !== original.supplier_id) request.supplier_id = values.supplierId;
  if (sellingPrice.value !== original.selling_price) request.selling_price = sellingPrice.value;
  if (costPrice.value !== original.cost_price_centi) request.cost_price_centi = costPrice.value;
  if (values.taxRate !== original.tax_rate) request.tax_rate = values.taxRate;

  const makerCode = trimToNullable(values.makerCode);
  if (makerCode !== original.maker_code) request.maker_code = makerCode;
  if (values.posStockSync !== original.pos_stock_sync) {
    request.pos_stock_sync = values.posStockSync;
  }
  if (values.pluTarget !== original.plu_target) {
    request.plu_target = values.pluTarget;
  }
  // UI-01b-D23: 送る判定は登録の単位で行う（登録の単位が長さの商品は送らない）
  if (!isLengthUnit(original.stock_unit) && values.stockUnit !== original.stock_unit) {
    request.stock_unit = values.stockUnit;
  }

  return { request, errors };
}
