// src/features/products/lib/product-form-request.test.ts

import { describe, expect, it } from "vitest";

import { makeMockDepartment, makeMockProductWithRelations } from "./test-fixtures";
import {
  buildCreateProductRequest,
  buildUpdateProductRequest,
  createProductFormDefaults,
  productToFormValues,
  type ProductFormValues,
} from "./product-form-request";

const prefixDepartment = makeMockDepartment({ id: 1, name: "毛糸", code_prefix: "Y" });
const noPrefixDepartment = makeMockDepartment({ id: 2, name: "通常部門", code_prefix: null });

function validValues(overrides: Partial<ProductFormValues> = {}): ProductFormValues {
  return {
    ...createProductFormDefaults,
    janCode: "2000000000138",
    name: "テスト商品",
    departmentId: 1,
    sellingPrice: "500",
    costPrice: "300",
    initialStock: "10",
    ...overrides,
  };
}

describe("buildCreateProductRequest (UI-01b-D4/D6)", () => {
  it("builds JAN product request", () => {
    const result = buildCreateProductRequest(validValues(), [prefixDepartment]);

    expect(result.errors).toEqual({});
    expect(result.request).toMatchObject({
      jan_code: "2000000000138",
      department_id: 1,
      stock_unit: "pcs",
      initial_stock: 10,
      pos_stock_sync: true,
      plu_target: false,
      // REQ-101 / UI-01b-D26: 1 回目の登録は独自コードの道を選ばない
      shared_jan_custom_code: false,
    });
  });

  it("allows JAN blank only when selected department has code_prefix", () => {
    expect(
      buildCreateProductRequest(validValues({ janCode: "", departmentId: 1 }), [
        prefixDepartment,
        noPrefixDepartment,
      ]).request,
    ).toMatchObject({ jan_code: null });

    const result = buildCreateProductRequest(validValues({ janCode: "", departmentId: 2 }), [
      prefixDepartment,
      noPrefixDepartment,
    ]);
    expect(result.request).toBeNull();
    expect(result.errors.janCode).toContain("独自コード発番対象");
  });

  it("preserves m POS sync override in payload", () => {
    const result = buildCreateProductRequest(validValues({ stockUnit: "m", posStockSync: true }), [
      prefixDepartment,
    ]);

    expect(result.request).toMatchObject({ stock_unit: "m", pos_stock_sync: true });
  });

  it("REQ-402 sends PLU target in create payload", () => {
    const result = buildCreateProductRequest(validValues({ pluTarget: true }), [prefixDepartment]);

    expect(result.request).toMatchObject({ plu_target: true });
  });

  it("REQ-101 saves trimmed normalized JAN wire value", () => {
    const result = buildCreateProductRequest(
      validValues({ janCode: " ４９０１２３４５６７８８７ " }),
      [prefixDepartment],
    );

    expect(result.errors).toEqual({});
    expect(result.request).toMatchObject({ jan_code: "4901234567887" });
  });

  it.each(["4901234567887", "96385074", "49123456"])(
    "REQ-101 accepts a valid JAN-8 or JAN-13 wire value: %s",
    (janCode) => {
      const result = buildCreateProductRequest(validValues({ janCode }), [prefixDepartment]);

      expect(result.errors).toEqual({});
      expect(result.request).toMatchObject({ jan_code: janCode });
    },
  );

  it.each(["123456789012", "490123456788A"])(
    "REQ-101 returns exact JAN length error: %s",
    (janCode) => {
      const result = buildCreateProductRequest(validValues({ janCode }), [prefixDepartment]);

      expect(result.request).toBeNull();
      expect(result.errors.janCode).toBe("JANコードは13桁または8桁で入力してください");
    },
  );

  it.each(["4901234567890", "49123457"])(
    "REQ-101 returns exact JAN check digit error: %s",
    (janCode) => {
      const result = buildCreateProductRequest(validValues({ janCode }), [prefixDepartment]);

      expect(result.request).toBeNull();
      expect(result.errors.janCode).toBe(
        "JANコードのチェックディジットが一致しません。入力値を確認してください",
      );
    },
  );
});

describe("buildUpdateProductRequest (UI-01b-D5)", () => {
  it("UI-01b PRODUCT-PATCH-D1 omits every unchanged field", () => {
    const original = makeMockProductWithRelations({
      supplier_id: 3,
      maker_code: "A-1",
    });

    expect(buildUpdateProductRequest(productToFormValues(original), original).request).toEqual({});
  });

  it("sends only supported changed fields and never sends read-only fields", () => {
    const original = makeMockProductWithRelations({
      product_code: "P-001",
      jan_code: "4901234567890",
      stock_quantity: 20,
      stock_unit: "m",
      supplier_id: 3,
      maker_code: "A-1",
    });
    const values = {
      ...productToFormValues(original),
      name: "更新後商品",
      supplierId: null,
      makerCode: "",
      initialStock: "999",
      stockUnit: "pcs" as const,
      janCode: "DIFFERENT",
    };

    const result = buildUpdateProductRequest(values, original);

    expect(result.request).toEqual({
      name: "更新後商品",
      supplier_id: null,
      maker_code: null,
    });
    expect(result.request).not.toHaveProperty("product_code");
    expect(result.request).not.toHaveProperty("jan_code");
    expect(result.request).not.toHaveProperty("stock_quantity");
    expect(result.request).not.toHaveProperty("stock_unit");
  });

  it("UI-01b-D23: 個数の単位の商品は単位を変えたときだけ stock_unit を送る", () => {
    const original = makeMockProductWithRelations({ stock_unit: "pcs" });
    const changed = buildUpdateProductRequest(
      { ...productToFormValues(original), stockUnit: "ball" },
      original,
    );
    expect(changed.request).toEqual({ stock_unit: "ball" });

    const unchanged = buildUpdateProductRequest(productToFormValues(original), original);
    expect(unchanged.request).not.toHaveProperty("stock_unit");
  });

  it("UI-01b PRODUCT-PATCH-D1 sends ordinary and clearable values without unrelated fields", () => {
    const original = makeMockProductWithRelations({
      name: "変更前",
      department_id: 1,
      supplier_id: null,
      selling_price: 500,
      cost_price_centi: 30000,
      tax_rate: "10",
      maker_code: null,
      pos_stock_sync: true,
      plu_target: false,
    });

    const result = buildUpdateProductRequest(
      {
        ...productToFormValues(original),
        name: "変更後",
        departmentId: 4,
        supplierId: 9,
        sellingPrice: "1200",
        costPrice: "700",
        taxRate: "8",
        makerCode: "M-9",
        posStockSync: false,
        pluTarget: true,
      },
      original,
    );

    expect(result.request).toEqual({
      name: "変更後",
      department_id: 4,
      supplier_id: 9,
      selling_price: 1200,
      cost_price_centi: 70000,
      tax_rate: "8",
      maker_code: "M-9",
      pos_stock_sync: false,
      plu_target: true,
    });
  });

  it("REQ-402 sends PLU target only when changed", () => {
    const original = makeMockProductWithRelations({ plu_target: false });

    expect(buildUpdateProductRequest(productToFormValues(original), original).request).toEqual({});
    expect(
      buildUpdateProductRequest({ ...productToFormValues(original), pluTarget: true }, original)
        .request,
    ).toEqual({ plu_target: true });
  });
});

// UI-01b-D22 / SPEC-UNIT-D3・D5（X3・X5・X6）: 10 個の単位、m の初期在庫、売価・原価・初期在庫の上限
describe("product form request の単位と上限（UI-01b-D22 / REQ-101）", () => {
  const create = (overrides: Partial<ProductFormValues>) =>
    buildCreateProductRequest(validValues(overrides), [prefixDepartment]);

  it("UI-01b-D22: m の初期在庫は m で受けて cm にする（既定の 0 も登録できる）", () => {
    expect(create({ stockUnit: "m", initialStock: "0" }).request).toMatchObject({
      stock_unit: "m",
      initial_stock: 0,
    });
    expect(create({ stockUnit: "m", initialStock: "25" }).request?.initial_stock).toBe(2500);
    expect(create({ stockUnit: "m", initialStock: "26.15" }).request?.initial_stock).toBe(2615);
    expect(create({ stockUnit: "m", initialStock: "0.005" }).errors.initialStock).toBe(
      "数量は0以上で、小数は2桁までの m で入力してください",
    );
    // 個数の単位は今の整数の規則のまま
    expect(create({ stockUnit: "ball", initialStock: "3" }).request).toMatchObject({
      stock_unit: "ball",
      initial_stock: 3,
    });
    expect(create({ stockUnit: "ball", initialStock: "25.5" }).errors.initialStock).toBe(
      "初期在庫は0以上の整数で入力してください",
    );
  });

  it("UI-01b-D22: 売価・原価・初期在庫は上限の端まで受け、1 つ外は欄の error で request を作らない", () => {
    expect(
      create({ sellingPrice: "999999", costPrice: "999999.99", initialStock: "999999999" }).request,
    ).toMatchObject({
      selling_price: 999999,
      cost_price_centi: 99999999,
      initial_stock: 999999999,
    });
    expect(create({ costPrice: "83.33" }).request?.cost_price_centi).toBe(8333);
    expect(create({ costPrice: "83.333" }).errors.costPrice).toBe(
      "原価は0以上で、小数は2桁までで入力してください",
    );
    expect(create({ sellingPrice: "83.33" }).errors.sellingPrice).toBe(
      "売価は0以上の整数で入力してください",
    );
    expect(create({ stockUnit: "m", initialStock: "9999999.99" }).request?.initial_stock).toBe(
      999999999,
    );
    const over = create({
      sellingPrice: "1000000",
      costPrice: "1000000",
      initialStock: "1000000000",
    });
    expect(over.request).toBeNull();
    expect(over.errors).toMatchObject({
      sellingPrice: "売価が大きすぎます",
      costPrice: "原価が大きすぎます",
      initialStock: "在庫数が大きすぎます",
    });
    expect(create({ stockUnit: "m", initialStock: "10000000" }).errors.initialStock).toBe(
      "在庫数が大きすぎます",
    );
  });

  it("UI-01b-D22: 修正の画面は 10 個の単位をそのまま出し、現在庫を単位つきで出す", () => {
    for (const unit of [
      "pcs",
      "sheet",
      "hon",
      "bag",
      "box",
      "kumi",
      "set",
      "ball",
      "cho",
      "m",
    ] as const) {
      expect(
        productToFormValues(makeMockProductWithRelations({ stock_unit: unit })).stockUnit,
      ).toBe(unit);
    }
    const stock = (stock_unit: "m" | "pcs", stock_quantity: number) =>
      productToFormValues(makeMockProductWithRelations({ stock_unit, stock_quantity }))
        .initialStock;
    expect(stock("m", 2750)).toBe("27.5 m");
    expect(stock("pcs", 20)).toBe("20 個");
    expect(stock("m", 0)).toBe("0 m");
    expect(stock("pcs", 0)).toBe("0 個");
    expect(stock("m", -30)).toBe("-0.3 m");
  });

  it("UI-01b-D22: 修正では現在庫を検証も送信もせず、売価・原価には上限を掛ける", () => {
    const original = makeMockProductWithRelations({ stock_unit: "m", stock_quantity: 2750 });
    const unchanged = buildUpdateProductRequest(productToFormValues(original), original);
    expect(unchanged.errors).toEqual({});
    expect(unchanged.request).toEqual({});
    const over = buildUpdateProductRequest(
      { ...productToFormValues(original), sellingPrice: "1000000", costPrice: "1000000" },
      original,
    );
    expect(over.request).toBeNull();
    expect(over.errors).toEqual({
      sellingPrice: "売価が大きすぎます",
      costPrice: "原価が大きすぎます",
    });
    expect(
      buildUpdateProductRequest(
        { ...productToFormValues(original), sellingPrice: "999999", costPrice: "999999.99" },
        original,
      ).request,
    ).toEqual({ selling_price: 999999, cost_price_centi: 99999999 });
    // 修正の画面の原価の初期値は小数 2 桁までの円（8333 → 83.33。SPEC-UNIT-D5）
    expect(
      productToFormValues(makeMockProductWithRelations({ cost_price_centi: 8333 })).costPrice,
    ).toBe("83.33");
  });
});

describe("productToFormValues の現在庫（D-118 b2、51）", () => {
  it("U4 productToFormValues formats recheck range for read-only stock", () => {
    // REQ-102 / SPEC-STK-CNT-D3・SPEC-UNIT-D3（Matrix U4）: 要再確認なら範囲、長さは m、そうでなければ 1 つの値。
    // 範囲の文字は送信しない（未変更の update request は空）
    const recheck = makeMockProductWithRelations({
      stock_quantity: 9,
      stock_recheck: true,
      stock_range_low: 7,
      stock_range_high: 12,
    });
    expect(productToFormValues(recheck).initialStock).toBe("7〜12 個");
    expect(buildUpdateProductRequest(productToFormValues(recheck), recheck).request).toEqual({});
    const length = makeMockProductWithRelations({
      stock_quantity: 450,
      stock_unit: "m",
      stock_recheck: true,
      stock_range_low: 400,
      stock_range_high: 450,
    });
    expect(productToFormValues(length).initialStock).toBe("4〜4.5 m");
    const plain = makeMockProductWithRelations({ stock_quantity: 3 });
    expect(productToFormValues(plain).initialStock).toBe("3 個");
  });
});
