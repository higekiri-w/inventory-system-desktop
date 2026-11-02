// src/features/stock-inquiry/lib/test-fixtures.ts
//
// Vitest 用 factory（DRY）。DTO（ProductWithRelations）+ 詳細（StockDetail）を生成。
// 設計: docs/function-design/58-ui-stock-inquiry.md §58.6

import type { Department, ProductWithRelations, StockDetail } from "@/lib/bindings";

/** ProductWithRelations の factory。Product フィールドは flatten でトップレベル。 */
export function makeMockProductWithRelations(
  overrides: Partial<ProductWithRelations> = {},
): ProductWithRelations {
  return {
    product_code: "P-0001",
    jan_code: "4901234567890",
    name: "テスト商品",
    department_id: 1,
    supplier_id: null,
    selling_price: 500,
    cost_price_centi: 30000,
    tax_rate: "10",
    maker_code: null,
    stock_quantity: 10,
    stock_unit: "pcs",
    is_discontinued: false,
    plu_dirty: false,
    plu_exported_at: null,
    plu_target: false,
    pos_stock_sync: true,
    created_at: "2026-01-01T10:00:00",
    updated_at: "2026-01-01T10:00:00",
    department_name: "毛糸",
    supplier_name: null,
    plu_memory_no: null,
    stock_recheck: false,
    stock_range_low: null,
    stock_range_high: null,
    ...overrides,
  };
}

/** StockDetail の factory。product は named field（flatten ではない）。 */
export function makeMockStockDetail(overrides: Partial<StockDetail> = {}): StockDetail {
  return {
    product: makeMockProductWithRelations(),
    last_receiving_date: "2026-03-20",
    last_sale_date: "2026-03-22",
    ...overrides,
  };
}

/** Department の factory（UI-06a-D2、部門候補 listDepartments 化の test 用）。 */
export function makeMockDepartment(overrides: Partial<Department> = {}): Department {
  return {
    id: 1,
    name: "毛糸",
    z005_name: null,
    code_prefix: "Y",
    next_seq: 1,
    created_at: "2026-01-01T10:00:00",
    ...overrides,
  };
}
