import type { ReturnCreateRequest } from "@/lib/bindings";
import { createPrefixedIdempotencyKey } from "@/lib/request-helpers";
import { ITEM_QUANTITY_FIELD, parseQuantityInput } from "@/lib/unit-quantity";
import type {
  ReturnExchangeFormErrors,
  ReturnExchangeFormValues,
  ReturnExchangeRow,
} from "../types";

export { getLocalDateString } from "@/lib/request-helpers";

export interface BuildReturnExchangeRequestResult {
  request: ReturnCreateRequest | null;
  errors: ReturnExchangeFormErrors;
  signature: string;
}

export interface ReceiptPathInput {
  receiptImagePath: string | null;
}

export function createReturnExchangeIdempotencyKey(): string {
  return createPrefixedIdempotencyKey("return");
}

function rowKey(row: Pick<ReturnExchangeRow, "productCode" | "direction">): string {
  return `${row.productCode}:${row.direction}`;
}

function normalizeRows(rows: ReturnExchangeRow[]) {
  return rows.map((row) => ({
    product_code: row.productCode,
    direction: row.direction,
    quantity: row.quantity.trim(),
  }));
}

export function buildReturnExchangeSignature(
  values: ReturnExchangeFormValues,
  receiptImagePath: string | null,
): string {
  return JSON.stringify({
    return_date: values.returnDate.trim(),
    return_type: values.returnType,
    register_processed: values.registerProcessed,
    receipt_image_path: receiptImagePath,
    note: values.note.trim(),
    items: normalizeRows(values.rows),
  });
}

export function buildReturnExchangeRequest(
  values: ReturnExchangeFormValues,
  idempotencyKey: string,
  receipt: ReceiptPathInput,
): BuildReturnExchangeRequestResult {
  const errors: ReturnExchangeFormErrors = {};
  const rowErrors: Record<string, string> = {};
  const returnDate = values.returnDate.trim();
  const note = values.note.trim();

  if (returnDate === "") errors.returnDate = "返品日は必須です";
  // UI-03-D23: レジでの戻し方は既定を置かず、選ぶまで request を作らない
  const registerProcessed = values.registerProcessed;
  if (registerProcessed === null) errors.registerProcessed = "レジでの戻し方を選んでください";
  if (values.rows.length === 0) errors.items = "明細が1件以上必要です";

  const hasIn = values.rows.some((row) => row.direction === "in");
  const hasOut = values.rows.some((row) => row.direction === "out");
  if (values.returnType === "return" && hasOut) {
    errors.items = "返品では渡し明細を指定できません";
  }
  if (values.returnType === "exchange" && (!hasIn || !hasOut)) {
    errors.items = "交換では戻り明細と渡し明細がそれぞれ必要です";
  }

  const items = values.rows.map((row) => {
    const quantity = parseQuantityInput(row.quantity, row.stockUnit, ITEM_QUANTITY_FIELD);
    if (quantity.error !== null) rowErrors[rowKey(row)] = quantity.error;
    return {
      product_code: row.productCode,
      direction: row.direction,
      quantity: quantity.value ?? 0,
    };
  });

  if (Object.keys(rowErrors).length > 0) errors.rows = rowErrors;

  const signature = buildReturnExchangeSignature(values, receipt.receiptImagePath);
  if (Object.keys(errors).length > 0 || registerProcessed === null) {
    return { request: null, errors, signature };
  }

  return {
    request: {
      idempotency_key: idempotencyKey,
      return_type: values.returnType,
      return_date: returnDate,
      register_processed: registerProcessed,
      receipt_image_path: receipt.receiptImagePath,
      note: note === "" ? null : note,
      items,
    },
    errors,
    signature,
  };
}
