// U3・U5（79 UI-16-D3・D4、10 SPEC-UNIT-D3・「入力の上限と安全な整数の範囲」(a)）の純関数の test。
// 期待値は設計正本から写す（上限 9999999、長さは 99999.99 m、m は小数 2 桁で cm に直す）。

import { describe, expect, it } from "vitest";

import { makeMockProductWithRelations } from "@/features/products/lib/test-fixtures";
import type { ProductStockUnit } from "@/lib/bindings";
import {
  BOTH_SIDES_REQUIRED_MESSAGE,
  addProductToTransfer,
  buildTransferRequest,
  type TransferFormValues,
  type TransferRow,
} from "./transfer-request";

function row(
  productCode: string,
  quantity: string,
  stockUnit: ProductStockUnit = "pcs",
): TransferRow {
  return {
    productCode,
    productName: `${productCode} の名前`,
    departmentName: "合成",
    stockUnit,
    currentStockQuantity: 10,
    quantity,
  };
}

function form(outRows: TransferRow[], inRows: TransferRow[]): TransferFormValues {
  return { transferDate: "2026-10-27", outRows, inRows };
}

describe("buildTransferRequest（U5 / UI-16-D4）", () => {
  it("U5: m の商品の 1.5 は 150 で送り、items は減らす明細の後に増やす明細", () => {
    const built = buildTransferRequest(
      form([row("TR-M1", "1.5", "m")], [row("TR-01", "3"), row("TR-02", "2")]),
      "transfer-key",
    );
    expect(built.errors).toEqual({});
    expect(built.request).toEqual({
      idempotency_key: "transfer-key",
      transfer_date: "2026-10-27",
      items: [
        { product_code: "TR-M1", direction: "out", quantity: 150 },
        { product_code: "TR-01", direction: "in", quantity: 3 },
        { product_code: "TR-02", direction: "in", quantity: 2 },
      ],
    });
  });

  it.each([
    ["0", "pcs", "数量は1以上の整数で入力してください"],
    ["-1", "pcs", "数量は1以上の整数で入力してください"],
    ["1.5", "pcs", "数量は1以上の整数で入力してください"],
    ["abc", "pcs", "数量は1以上の整数で入力してください"],
    ["10000000", "pcs", "数量が大きすぎます"],
    ["0", "m", "数量は0.01以上で、小数は2桁までの m で入力してください"],
    ["1.234", "m", "数量は0.01以上で、小数は2桁までの m で入力してください"],
    ["100000", "m", "数量が大きすぎます"],
  ] as const)("U5: 数量 %s（%s）を保存の前に止める", (quantity, unit, message) => {
    for (const side of ["out", "in"] as const) {
      const bad = row("TR-BAD", quantity, unit);
      const values =
        side === "out" ? form([bad], [row("TR-01", "1")]) : form([row("TR-01", "1")], [bad]);
      const built = buildTransferRequest(values, "k");
      expect(built.request).toBeNull();
      expect(built.errors.rows).toEqual({ [`${side}:TR-BAD`]: message });
    }
  });

  it("U5: 上限ちょうど（9999999・99999.99 m）は通る", () => {
    const built = buildTransferRequest(
      form([row("TR-M1", "99999.99", "m")], [row("TR-01", "9999999")]),
      "k",
    );
    expect(built.request?.items.map((item) => item.quantity)).toEqual([9999999, 9999999]);
  });

  it("U4: どちらかの表が 0 行なら理由の文で止める", () => {
    expect(buildTransferRequest(form([row("TR-01", "1")], []), "k").errors.items).toBe(
      BOTH_SIDES_REQUIRED_MESSAGE,
    );
    expect(buildTransferRequest(form([], [row("TR-01", "1")]), "k").errors.items).toBe(
      "減らす商品と増やす商品を 1 つ以上ずつ入れてください",
    );
  });

  it("日付が空なら止める", () => {
    const built = buildTransferRequest(
      { ...form([row("TR-01", "1")], [row("TR-02", "1")]), transferDate: " " },
      "k",
    );
    expect(built.request).toBeNull();
    expect(built.errors.transferDate).toBe("詰め替え・切り分けた日は必須です");
  });
});

describe("addProductToTransfer（U3 / UI-16-D3）", () => {
  const product = makeMockProductWithRelations({ product_code: "TR-01", stock_unit: "pcs" });
  const lengthProduct = makeMockProductWithRelations({ product_code: "TR-M1", stock_unit: "m" });

  it("U3: 同じ表の再追加は数量 +1（m は +1 m）", () => {
    const once = addProductToTransfer(form([], []), "in", product).values;
    if (once === null) throw new Error("expected values");
    const twice = addProductToTransfer(once, "in", product).values;
    expect(twice?.inRows.map((r) => [r.productCode, r.quantity])).toEqual([["TR-01", "2"]]);

    const m = addProductToTransfer(form([row("TR-M1", "1.5", "m")], []), "out", lengthProduct);
    expect(m.values?.outRows[0]?.quantity).toBe("2.5");
  });

  it("U3: もう片方の表にある商品は足さず、どちらの表にあるかを名指す", () => {
    const blocked = addProductToTransfer(form([], [row("TR-01", "1")]), "out", product);
    expect(blocked).toEqual({
      values: null,
      blockedMessage: "この商品は「増やす商品」に入っています",
    });
    const blockedIn = addProductToTransfer(form([row("TR-01", "1")], []), "in", product);
    expect(blockedIn.blockedMessage).toBe("この商品は「減らす商品」に入っています");
  });
});
