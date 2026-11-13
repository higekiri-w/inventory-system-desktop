import { describe, expect, it } from "vitest";

import type { ReturnExchangeFormValues } from "../types";
import {
  buildReturnExchangeRequest,
  buildReturnExchangeSignature,
} from "./return-exchange-request";

function createValues(overrides: Partial<ReturnExchangeFormValues> = {}): ReturnExchangeFormValues {
  return {
    returnDate: "2026-06-26",
    returnType: "return",
    registerProcessed: true,
    note: "",
    rows: [
      {
        productCode: "RT-001",
        productName: "返品商品",
        departmentName: "布",
        stockUnit: "pcs",
        currentStockQuantity: 5,
        direction: "in",
        quantity: "2",
      },
    ],
    ...overrides,
  };
}

describe("return exchange request builder (UI-03 / REQ-202)", () => {
  it("REQ-202 builds the generated createReturn request shape", () => {
    const built = buildReturnExchangeRequest(createValues({ note: "レシートあり" }), "ret-key-1", {
      receiptImagePath: "images/receipts/test.jpg",
    });

    expect(built.errors).toEqual({});
    expect(built.request).toEqual({
      idempotency_key: "ret-key-1",
      return_type: "return",
      return_date: "2026-06-26",
      register_processed: true,
      receipt_image_path: "images/receipts/test.jpg",
      note: "レシートあり",
      items: [{ product_code: "RT-001", direction: "in", quantity: 2 }],
    });
  });

  it("REQ-202 / UI-03-D23 (U1): レジでの戻し方が未選択なら field error にして request を作らない", () => {
    const built = buildReturnExchangeRequest(
      createValues({ registerProcessed: null }),
      "ret-key-1",
      { receiptImagePath: null },
    );

    expect(built.request).toBeNull();
    expect(built.errors).toEqual({ registerProcessed: "レジでの戻し方を選んでください" });
  });

  it.each([true, false])(
    "REQ-202 / UI-03-D23 (U3): 選んだ戻し方 %s をそのまま register_processed にする",
    (registerProcessed) => {
      const built = buildReturnExchangeRequest(createValues({ registerProcessed }), "ret-key-1", {
        receiptImagePath: null,
      });

      expect(built.errors).toEqual({});
      expect(built.request?.register_processed).toBe(registerProcessed);
    },
  );

  it("REQ-202 blocks return with out direction before submit", () => {
    const built = buildReturnExchangeRequest(
      createValues({
        rows: [
          {
            productCode: "RT-001",
            productName: "返品商品",
            departmentName: "布",
            stockUnit: "pcs",
            currentStockQuantity: 5,
            direction: "out",
            quantity: "1",
          },
        ],
      }),
      "ret-key-1",
      { receiptImagePath: null },
    );

    expect(built.request).toBeNull();
    expect(built.errors.items).toBe("返品では渡し明細を指定できません");
  });

  it("REQ-202 blocks exchange missing either in or out rows", () => {
    const onlyIn = buildReturnExchangeRequest(
      createValues({ returnType: "exchange" }),
      "ret-key-1",
      { receiptImagePath: null },
    );
    const onlyOut = buildReturnExchangeRequest(
      createValues({
        returnType: "exchange",
        rows: [
          {
            productCode: "RT-002",
            productName: "交換商品",
            departmentName: "布",
            stockUnit: "pcs",
            currentStockQuantity: 5,
            direction: "out",
            quantity: "1",
          },
        ],
      }),
      "ret-key-2",
      { receiptImagePath: null },
    );

    expect(onlyIn.request).toBeNull();
    expect(onlyOut.request).toBeNull();
    expect(onlyIn.errors.items).toBe("交換では戻り明細と渡し明細がそれぞれ必要です");
    expect(onlyOut.errors.items).toBe("交換では戻り明細と渡し明細がそれぞれ必要です");
  });

  it("REQ-202 includes note and image path in the UI signature", () => {
    const before = buildReturnExchangeSignature(createValues({ note: "" }), null);
    const afterNote = buildReturnExchangeSignature(createValues({ note: "変更" }), null);
    const afterImage = buildReturnExchangeSignature(
      createValues({ note: "" }),
      "images/receipts/test.jpg",
    );

    expect(afterNote).not.toBe(before);
    expect(afterImage).not.toBe(before);
  });
});

describe("return exchange request の単位と上限（REQ-202 / UI-03-D12 / SPEC-UNIT-D3、X1）", () => {
  const build = (stockUnit: "m" | "pcs", quantity: string) => {
    const base = createValues().rows[0];
    return buildReturnExchangeRequest(
      createValues({ rows: [{ ...base, stockUnit, quantity }] }),
      "ret-key-u",
      { receiptImagePath: null },
    );
  };

  it("REQ-202: m の商品は m で受けて cm の整数で送る（0.3 → 30）", () => {
    expect(build("m", "0.3").request?.items).toEqual([
      { product_code: "RT-001", direction: "in", quantity: 30 },
    ]);
  });

  it("REQ-202: 数量は上限の端まで送れ、1 つ外は入力の error", () => {
    expect(build("pcs", "9999999").request?.items[0]?.quantity).toBe(9999999);
    expect(build("m", "99999.99").request?.items[0]?.quantity).toBe(9999999);
    expect(build("pcs", "10000000").errors.rows?.["RT-001:in"]).toBe("数量が大きすぎます");
    expect(build("m", "100000").errors.rows?.["RT-001:in"]).toBe("数量が大きすぎます");
    expect(build("m", "1.234").errors.rows?.["RT-001:in"]).toBe(
      "数量は0.01以上で、小数は2桁までの m で入力してください",
    );
    expect(build("pcs", "0").errors.rows?.["RT-001:in"]).toBe(
      "数量は1以上の整数で入力してください",
    );
  });
});
