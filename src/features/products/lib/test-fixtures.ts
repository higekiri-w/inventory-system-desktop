// src/features/products/lib/test-fixtures.ts
//
// UI-01a tests 用 DTO factory。

import type { Department, ProductWithRelations, Supplier } from "@/lib/bindings";

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

export function makeMockSupplier(overrides: Partial<Supplier> = {}): Supplier {
  return {
    id: 1,
    name: "テスト取引先",
    created_at: "2026-01-01T10:00:00",
    ...overrides,
  };
}
