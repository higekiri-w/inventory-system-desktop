import type { ReceivingCreateRequest } from "@/lib/bindings";
import { createPrefixedIdempotencyKey } from "@/lib/request-helpers";
import { ITEM_QUANTITY_FIELD, parseCostInput, parseQuantityInput } from "@/lib/unit-quantity";
import type { ReceivingFormErrors, ReceivingFormValues, ReceivingRow } from "../types";

export { getLocalDateString } from "@/lib/request-helpers";

export interface BuildReceivingRequestResult {
  request: ReceivingCreateRequest | null;
  errors: ReceivingFormErrors;
  signature: string;
}

export function createReceivingIdempotencyKey(): string {
  return createPrefixedIdempotencyKey("receiving");
}

function normalizeRows(rows: ReceivingRow[]) {
  return rows.map((row) => ({
    product_code: row.productCode,
    quantity: row.quantity.trim(),
    cost_price_centi: row.costPrice.trim(),
  }));
}

export function buildReceivingSignature(values: ReceivingFormValues): string {
  return JSON.stringify({
    supplier_id: values.supplierId,
    receiving_date: values.receivingDate.trim(),
    note: values.note.trim(),
    items: normalizeRows(values.rows),
  });
}

export function buildReceivingRequest(
  values: ReceivingFormValues,
  idempotencyKey: string,
): BuildReceivingRequestResult {
  const errors: ReceivingFormErrors = {};
  const rowErrors: Record<string, string> = {};
  const receivingDate = values.receivingDate.trim();
  const note = values.note.trim();

  if (receivingDate === "") {
    errors.receivingDate = "届いた日は必須です";
  }
  if (values.rows.length === 0) {
    errors.items = "明細が1件以上必要です";
  }

  const items = values.rows.map((row, index) => {
    const quantity = parseQuantityInput(row.quantity, row.stockUnit, ITEM_QUANTITY_FIELD);
    // 原価は小数 2 桁までの円 → 1/100 円（SPEC-UNIT-D5）
    const costPrice = parseCostInput(row.costPrice);
    const messages = [];
    if (quantity.error !== null) messages.push(quantity.error);
    if (costPrice === null) messages.push("原価は0以上で、小数は2桁までで入力してください");
    else if (costPrice === "too_large") messages.push("原価が大きすぎます");
    if (messages.length > 0) rowErrors[row.productCode] = messages.join(" / ");

    return {
      product_code: row.productCode,
      quantity: quantity.value ?? 0,
      cost_price_centi: typeof costPrice === "number" ? costPrice : 0,
      index,
    };
  });

  if (Object.keys(rowErrors).length > 0) errors.rows = rowErrors;

  const signature = buildReceivingSignature(values);
  if (Object.keys(errors).length > 0) {
    return { request: null, errors, signature };
  }

  return {
    request: {
      idempotency_key: idempotencyKey,
      supplier_id: values.supplierId,
      receiving_date: receivingDate,
      note: note === "" ? null : note,
      items: items.map(({ product_code, quantity, cost_price_centi }) => ({
        product_code,
        quantity,
        cost_price_centi,
      })),
    },
    errors,
    signature,
  };
}
