// src/features/stock-transfer/lib/transfer-request.ts
//
// UI-16 詰め替え・切り分けの明細の操作と、保存の前の検証・DTO の組み立て（純関数）。
// 設計: docs/function-design/79-ui-stock-transfer.md §79.3・UI-16-D3・D4・D6

import type {
  ProductStockUnit,
  ProductWithRelations,
  TransferCreateRequest,
  TransferDirection,
} from "@/lib/bindings";
import { createPrefixedIdempotencyKey } from "@/lib/request-helpers";
import {
  ITEM_QUANTITY_FIELD,
  addToQuantityInput,
  entryQuantityStep,
  parseQuantityInput,
} from "@/lib/unit-quantity";

export { getLocalDateString } from "@/lib/request-helpers";

export interface TransferRow {
  productCode: string;
  productName: string;
  departmentName: string;
  stockUnit: ProductStockUnit;
  currentStockQuantity: number;
  /** 入力の文字列（個数は整数、`m` は m の小数 2 桁まで） */
  quantity: string;
}

export interface TransferFormValues {
  transferDate: string;
  outRows: TransferRow[];
  inRows: TransferRow[];
}

export type TransferFormErrors = Partial<{
  transferDate: string;
  items: string;
  /** key は `rowErrorKey(direction, productCode)` */
  rows: Record<string, string>;
}>;

export interface BuildTransferRequestResult {
  request: TransferCreateRequest | null;
  errors: TransferFormErrors;
  signature: string;
}

export const TRANSFER_SIDE_LABELS: Record<TransferDirection, string> = {
  out: "減らす商品",
  in: "増やす商品",
};

/** UI-16-D4 / §79.5: どちらかの表が 0 行の間の理由の文 */
export const BOTH_SIDES_REQUIRED_MESSAGE = "減らす商品と増やす商品を 1 つ以上ずつ入れてください";

export function createTransferIdempotencyKey(): string {
  return createPrefixedIdempotencyKey("transfer");
}

export function rowsOf(values: TransferFormValues, direction: TransferDirection): TransferRow[] {
  return direction === "out" ? values.outRows : values.inRows;
}

function withRows(
  values: TransferFormValues,
  direction: TransferDirection,
  rows: TransferRow[],
): TransferFormValues {
  return direction === "out" ? { ...values, outRows: rows } : { ...values, inRows: rows };
}

export function rowErrorKey(direction: TransferDirection, productCode: string): string {
  return `${direction}:${productCode}`;
}

export type AddTransferProductResult =
  | { values: TransferFormValues; blockedMessage: null }
  | { values: null; blockedMessage: string };

/**
 * UI-16-D3: 同じ表の再追加は数量を 1 単位（`m` は 1 m）足し、もう片方の表にある商品は足さずに案内を返す。
 */
export function addProductToTransfer(
  values: TransferFormValues,
  direction: TransferDirection,
  product: ProductWithRelations,
): AddTransferProductResult {
  const other: TransferDirection = direction === "out" ? "in" : "out";
  if (rowsOf(values, other).some((row) => row.productCode === product.product_code)) {
    return {
      values: null,
      blockedMessage: `この商品は「${TRANSFER_SIDE_LABELS[other]}」に入っています`,
    };
  }
  const rows = rowsOf(values, direction);
  const existing = rows.find((row) => row.productCode === product.product_code);
  if (existing === undefined) {
    const row: TransferRow = {
      productCode: product.product_code,
      productName: product.name,
      departmentName: product.department_name,
      stockUnit: product.stock_unit,
      currentStockQuantity: product.stock_quantity,
      quantity: "1",
    };
    return { values: withRows(values, direction, [...rows, row]), blockedMessage: null };
  }
  const nextRows = rows.map((row) =>
    row === existing
      ? {
          ...row,
          quantity:
            addToQuantityInput(row.quantity, row.stockUnit, entryQuantityStep(row.stockUnit)) ??
            "1",
        }
      : row,
  );
  return { values: withRows(values, direction, nextRows), blockedMessage: null };
}

export function updateTransferRowQuantity(
  values: TransferFormValues,
  direction: TransferDirection,
  productCode: string,
  quantity: string,
): TransferFormValues {
  return withRows(
    values,
    direction,
    rowsOf(values, direction).map((row) =>
      row.productCode === productCode ? { ...row, quantity } : row,
    ),
  );
}

export function removeTransferRow(
  values: TransferFormValues,
  direction: TransferDirection,
  productCode: string,
): TransferFormValues {
  return withRows(
    values,
    direction,
    rowsOf(values, direction).filter((row) => row.productCode !== productCode),
  );
}

export function hasBothSides(values: TransferFormValues): boolean {
  return values.outRows.length > 0 && values.inRows.length > 0;
}

export function buildTransferSignature(values: TransferFormValues): string {
  const normalize = (rows: TransferRow[]) =>
    rows.map((row) => ({ product_code: row.productCode, quantity: row.quantity.trim() }));
  return JSON.stringify({
    transfer_date: values.transferDate.trim(),
    out: normalize(values.outRows),
    in: normalize(values.inRows),
  });
}

/**
 * UI-16-D4: 保存の前の検証と payload。items の順は減らす明細の後に増やす明細（§79.3）。
 * 数量は商品の単位で入れ、`m` は cm の整数に直す（SPEC-UNIT-D3。廃棄と同じ純関数）。
 */
export function buildTransferRequest(
  values: TransferFormValues,
  idempotencyKey: string,
): BuildTransferRequestResult {
  const errors: TransferFormErrors = {};
  const rowErrors: Record<string, string> = {};
  const transferDate = values.transferDate.trim();

  if (transferDate === "") errors.transferDate = "詰め替え・切り分けた日は必須です";
  if (!hasBothSides(values)) errors.items = BOTH_SIDES_REQUIRED_MESSAGE;

  const sides: [TransferDirection, TransferRow[]][] = [
    ["out", values.outRows],
    ["in", values.inRows],
  ];
  const items = sides.flatMap(([direction, rows]) =>
    rows.map((row) => {
      const quantity = parseQuantityInput(row.quantity, row.stockUnit, ITEM_QUANTITY_FIELD);
      if (quantity.error !== null) {
        rowErrors[rowErrorKey(direction, row.productCode)] = quantity.error;
      }
      return { product_code: row.productCode, direction, quantity: quantity.value ?? 0 };
    }),
  );

  if (Object.keys(rowErrors).length > 0) errors.rows = rowErrors;

  const signature = buildTransferSignature(values);
  if (Object.keys(errors).length > 0) return { request: null, errors, signature };

  return {
    request: { idempotency_key: idempotencyKey, transfer_date: transferDate, items },
    errors,
    signature,
  };
}
