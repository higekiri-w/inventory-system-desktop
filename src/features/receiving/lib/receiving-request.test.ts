import { describe, expect, it } from "vitest";

import { buildReceivingRequest, buildReceivingSignature } from "./receiving-request";
import type { ReceivingFormValues } from "../types";

function makeValues(overrides: Partial<ReceivingFormValues> = {}): ReceivingFormValues {
  return {
    supplierId: null,
    receivingDate: "2026-06-25",
    note: "",
    rows: [
      {
        productCode: "P-001",
        productName: "はさみ",
        stockUnit: "pcs",
        quantity: "2",
        costPrice: "120",
      },
    ],
    ...overrides,
  };
}

describe("receiving-request (UI-02-D7/D8 / REQ-201)", () => {
  it("builds a create request with nullable supplier and note", () => {
    const result = buildReceivingRequest(makeValues(), "key-1");

    expect(result.errors).toEqual({});
    expect(result.request).toEqual({
      idempotency_key: "key-1",
      supplier_id: null,
      receiving_date: "2026-06-25",
      note: null,
      items: [{ product_code: "P-001", quantity: 2, cost_price_centi: 12000 }],
    });
  });

  it("blocks blank date, empty rows, decimal quantity, and negative cost before command", () => {
    const result = buildReceivingRequest(
      makeValues({
        receivingDate: "",
        rows: [
          {
            productCode: "P-001",
            productName: "はさみ",
            stockUnit: "pcs",
            quantity: "1.5",
            costPrice: "-1",
          },
        ],
      }),
      "key-1",
    );

    expect(result.request).toBeNull();
    expect(result.errors.receivingDate).toBe("届いた日は必須です");
    expect(result.errors.rows?.["P-001"]).toContain("数量は1以上の整数");
    expect(result.errors.rows?.["P-001"]).toContain(
      "原価は0以上で、小数は2桁までで入力してください",
    );

    const empty = buildReceivingRequest(makeValues({ rows: [] }), "key-1");
    expect(empty.request).toBeNull();
    expect(empty.errors.items).toBe("明細が1件以上必要です");
  });

  it("keeps the same signature for same-content retry and changes it after edit", () => {
    const values = makeValues();
    const sameContent = makeValues();
    const edited = makeValues({ note: "納品書あり" });

    expect(buildReceivingSignature(values)).toBe(buildReceivingSignature(sameContent));
    expect(buildReceivingSignature(values)).not.toBe(buildReceivingSignature(edited));
  });
});

describe("receiving-request の単位と上限（REQ-201 / UI-02-D7 / SPEC-UNIT-D3、X1・X6）", () => {
  const build = (stockUnit: "m" | "pcs", quantity: string, costPrice = "333") =>
    buildReceivingRequest(
      makeValues({
        rows: [{ productCode: "P-U", productName: "合成", stockUnit, quantity, costPrice }],
      }),
      "key-u",
    );

  it("REQ-201: m の商品は m で受けて cm の整数で送る（2.5 → 250、0.29 → 29）", () => {
    expect(build("m", "2.5").request?.items).toEqual([
      { product_code: "P-U", quantity: 250, cost_price_centi: 33300 },
    ]);
    expect(build("m", "0.29").request?.items[0]?.quantity).toBe(29);
  });

  it("REQ-201: 数量と原価は上限の端まで送れ、1 つ外は入力の error で request を作らない", () => {
    expect(build("pcs", "9999999", "999999.99").request?.items[0]).toEqual({
      product_code: "P-U",
      quantity: 9999999,
      cost_price_centi: 99999999,
    });
    expect(build("m", "99999.99").request?.items[0]?.quantity).toBe(9999999);
    for (const [unit, quantity] of [
      ["pcs", "10000000"],
      ["m", "100000"],
    ] as const) {
      const result = build(unit, quantity);
      expect(result.request).toBeNull();
      expect(result.errors.rows?.["P-U"]).toBe("数量が大きすぎます");
    }
    const cost = build("pcs", "1", "1000000");
    expect(cost.request).toBeNull();
    expect(cost.errors.rows?.["P-U"]).toBe("原価が大きすぎます");
  });

  it("REQ-201: m の商品の形式の error は m の文、個数の商品は今の整数の文", () => {
    for (const quantity of ["0", "1.300", "1e2", "-1", ""]) {
      expect(build("m", quantity).errors.rows?.["P-U"]).toBe(
        "数量は0.01以上で、小数は2桁までの m で入力してください",
      );
    }
    expect(build("pcs", "1.3").errors.rows?.["P-U"]).toBe("数量は1以上の整数で入力してください");
  });
});
