// src/features/products/search.test.ts
//
// UI-01a-D1〜D4/D7: URL search params -> ProductSearchQuery mapping と page reset/preserve。

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

import { LIST_PER_PAGE_OPTIONS } from "@/components/patterns/list-per-page";
import {
  PRODUCT_DISCONTINUED_OPTIONS,
  PRODUCT_PLU_OPTIONS,
  PRODUCT_SORT_DIRECTION_OPTIONS,
  PRODUCT_SORT_OPTIONS,
  buildProductSearchQuery,
  buildProductBulkFilter,
  normalizeProductListSearch,
  productListSearchSchema,
  updateProductListSearch,
} from "./search";
import { MOVEMENT_TYPE_OPTIONS, stockMovementsSearchSchema } from "../stock-movements/types";
import {
  INVENTORY_RECORD_STATUS_OPTIONS,
  INVENTORY_RECORD_TYPE_OPTIONS,
  formatRecordStatus,
  formatRecordType,
  inventoryRecordsSearchSchema,
} from "../inventory-records/types";
import {
  DAILY_SORT_DESCRIPTORS,
  DAILY_SORT_DIRECTION_OPTIONS,
  dailySalesSearchSchema,
} from "../daily-sales/types";
import {
  MONTHLY_MODE_DESCRIPTORS,
  MONTHLY_SORT_DESCRIPTORS,
  MONTHLY_SORT_DIRECTION_OPTIONS,
  monthlySalesSearchSchema,
} from "../monthly-sales/types";
import { STOCK_FILTER_DESCRIPTORS, stockInquirySearchSchema } from "../stock-inquiry/types";

describe("product list search mapping (UI-01a)", () => {
  it("UI-01a-D1/D3: defaults to active products payload", () => {
    expect(buildProductSearchQuery({})).toEqual({
      keyword: null,
      department_id: null,
      is_discontinued: false,
      plu: "all",
      sort_key: "ProductCode",
      sort_order: "Asc",
      page: 1,
      per_page: 100,
    });
  });

  it("UI-01a-D2/D3: maps URL params to generated enum payload", () => {
    expect(
      buildProductSearchQuery({
        q: "  HZ-0047  ",
        dept: 3,
        discontinued: "discontinued",
        plu: "synced",
        sort: "selling_price",
        dir: "desc",
        page: 4,
        perPage: 200,
      }),
    ).toEqual({
      keyword: "HZ-0047",
      department_id: 3,
      is_discontinued: true,
      plu: "synced",
      sort_key: "SellingPrice",
      sort_order: "Desc",
      page: 4,
      per_page: 200,
    });
  });

  it("UI-01a-D2/D4: normalizes invalid URL values before command payload", () => {
    const normalized = normalizeProductListSearch({
      q: "",
      dept: -1,
      discontinued: "unknown",
      plu: "bogus",
      sort: "bad_sort",
      dir: "sideways",
      page: 0,
      perPage: 201,
    });

    expect(normalized).toEqual({
      q: undefined,
      dept: undefined,
      discontinued: "active",
      plu: "all",
      sort: "product_code",
      dir: "asc",
      page: 1,
      perPage: 100,
    });
    expect(buildProductSearchQuery(normalized).per_page).toBe(100);
  });

  it("REQ-907 B-V2: normalizes PLU URL values and builds the unpaged bulk filter", () => {
    expect(normalizeProductListSearch({ plu: "pending", discontinued: "all" })).toMatchObject({
      plu: "pending",
      discontinued: "all",
      page: 1,
    });
    expect(buildProductSearchQuery({ plu: "bogus" })).toMatchObject({ plu: "all" });
    expect(
      buildProductBulkFilter({
        q: "  糸  ",
        dept: 2,
        discontinued: "all",
        plu: "pending",
        page: 9,
        perPage: 200,
      }),
    ).toEqual({
      keyword: "糸",
      department_id: 2,
      is_discontinued: null,
      plu: "pending",
    });
    expect(updateProductListSearch({ plu: "pending", page: 7 }, { plu: "synced" })).toEqual({
      plu: "synced",
      page: 1,
    });
  });

  it("UI-01a-D4: filter/sort/perPage changes reset page but page-only changes preserve filters", () => {
    const current = {
      q: "毛糸",
      dept: 2,
      discontinued: "all" as const,
      sort: "name" as const,
      dir: "desc" as const,
      page: 3,
      perPage: 100 as const,
    };

    expect(updateProductListSearch(current, { page: 4 })).toEqual({ ...current, page: 4 });
    expect(updateProductListSearch(current, { q: "布" })).toEqual({
      ...current,
      q: "布",
      page: 1,
    });
    expect(updateProductListSearch(current, { dept: null })).toEqual({
      ...current,
      dept: undefined,
      page: 1,
    });
    expect(updateProductListSearch(current, { perPage: 200 })).toEqual({
      ...current,
      perPage: 200,
      page: 1,
    });
  });
});

describe("feature-owned finite search schemas", () => {
  it("UI-STATE-D2: product descriptors match the independently transcribed contract", () => {
    expect(PRODUCT_DISCONTINUED_OPTIONS).toEqual([
      { value: "active", label: "表示中", payload: false },
      { value: "all", label: "すべて", payload: null },
      { value: "discontinued", label: "廃番のみ", payload: true },
    ]);
    expect(PRODUCT_PLU_OPTIONS).toEqual([
      { value: "all", label: "すべて", payload: "all" },
      { value: "target", label: "対象", payload: "target" },
      { value: "pending", label: "未反映", payload: "pending" },
      { value: "synced", label: "反映済み", payload: "synced" },
      { value: "excluded", label: "対象外", payload: "excluded" },
    ]);
    expect(PRODUCT_SORT_OPTIONS).toEqual([
      { value: "product_code", label: "商品コード", payload: "ProductCode" },
      { value: "name", label: "商品名", payload: "Name" },
      { value: "stock_quantity", label: "在庫数", payload: "StockQuantity" },
      { value: "selling_price", label: "売価", payload: "SellingPrice" },
    ]);
    expect(PRODUCT_SORT_DIRECTION_OPTIONS).toEqual([
      { value: "asc", label: "昇順", payload: "Asc" },
      { value: "desc", label: "降順", payload: "Desc" },
    ]);
    expect(LIST_PER_PAGE_OPTIONS).toEqual([50, 100, 200]);
  });

  it("UI-STATE-D2: adjacent finite descriptors match the fixed URL and UI contract", () => {
    expect(MOVEMENT_TYPE_OPTIONS).toEqual([
      { value: "all", label: "すべて" },
      { value: "receiving", label: "入庫" },
      { value: "return", label: "返品・交換" },
      { value: "sale_auto", label: "POS売上" },
      { value: "sale_manual", label: "手動販売" },
      { value: "disposal", label: "廃棄・破損" },
      { value: "transfer", label: "詰め替え・切り分け" },
      { value: "stocktake", label: "棚卸し" },
    ]);
    expect(INVENTORY_RECORD_TYPE_OPTIONS).toEqual([
      { value: "all", label: "すべて" },
      { value: "receiving_record", label: "入庫" },
      { value: "return_record", label: "返品・交換" },
      { value: "manual_sale", label: "手動販売出庫" },
      { value: "disposal_record", label: "廃棄・破損" },
      { value: "transfer_record", label: "詰め替え・切り分け" },
      { value: "csv_import", label: "CSV取込み" },
      { value: "stocktake", label: "棚卸し" },
    ]);
    expect(INVENTORY_RECORD_STATUS_OPTIONS).toEqual([
      { value: "all", label: "すべて" },
      { value: "active", label: "有効" },
      { value: "canceled", label: "取消済み" },
      { value: "in_progress", label: "進行中" },
    ]);
    expect(DAILY_SORT_DESCRIPTORS).toEqual([
      { value: "product_code", label: "商品コード", align: "left" },
      { value: "name", label: "商品名", align: "left" },
      { value: "quantity", label: "数量", align: "right" },
      { value: "unit_price", label: "単価", align: "right" },
      { value: "amount", label: "金額", align: "right" },
    ]);
    expect(DAILY_SORT_DIRECTION_OPTIONS).toEqual([
      { value: "asc", label: "昇順" },
      { value: "desc", label: "降順" },
    ]);
    expect(MONTHLY_MODE_DESCRIPTORS).toEqual([
      { value: "by_product", label: "商品別ランキング" },
      { value: "by_department", label: "部門別構成比" },
    ]);
    expect(MONTHLY_SORT_DESCRIPTORS).toEqual([
      {
        value: "name",
        productLabel: "商品名",
        departmentLabel: "部門",
        align: "left",
        modes: ["by_product", "by_department"],
      },
      {
        value: "quantity",
        productLabel: "数量",
        departmentLabel: "数量",
        align: "right",
        modes: ["by_product", "by_department"],
      },
      {
        value: "amount",
        productLabel: "金額",
        departmentLabel: "売上",
        align: "right",
        modes: ["by_product", "by_department"],
      },
      {
        value: "prev_month_diff",
        productLabel: "前月比",
        departmentLabel: "前月比",
        align: "right",
        modes: ["by_product", "by_department"],
      },
    ]);
    expect(MONTHLY_SORT_DIRECTION_OPTIONS).toEqual([
      { value: "asc", label: "昇順" },
      { value: "desc", label: "降順" },
    ]);
    expect(STOCK_FILTER_DESCRIPTORS).toEqual([
      { value: "all", label: "すべて" },
      { value: "stockout", label: "在庫切れ" },
      { value: "low_stock", label: "在庫少" },
    ]);
  });

  it.each([
    {
      name: "product PLU",
      schema: productListSearchSchema,
      key: "plu",
      values: ["all", "target", "pending", "synced", "excluded"],
    },
    {
      name: "product discontinued",
      schema: productListSearchSchema,
      key: "discontinued",
      values: ["active", "all", "discontinued"],
    },
    {
      name: "product sort",
      schema: productListSearchSchema,
      key: "sort",
      values: ["product_code", "name", "stock_quantity", "selling_price"],
    },
    {
      name: "product direction",
      schema: productListSearchSchema,
      key: "dir",
      values: ["asc", "desc"],
    },
    {
      name: "product page size",
      schema: productListSearchSchema,
      key: "perPage",
      values: [50, 100, 200],
    },
    {
      name: "movement type",
      schema: stockMovementsSearchSchema,
      key: "type",
      values: [
        "all",
        "receiving",
        "return",
        "sale_auto",
        "sale_manual",
        "disposal",
        "transfer",
        "stocktake",
      ],
    },
    {
      name: "record type",
      schema: inventoryRecordsSearchSchema,
      key: "recordType",
      values: [
        "all",
        "receiving_record",
        "return_record",
        "manual_sale",
        "disposal_record",
        "transfer_record",
      ],
    },
    {
      name: "record status",
      schema: inventoryRecordsSearchSchema,
      key: "status",
      values: ["all", "active"],
    },
    {
      name: "daily sort",
      schema: dailySalesSearchSchema,
      key: "sortBy",
      values: ["product_code", "name", "quantity", "unit_price", "amount"],
    },
    {
      name: "daily direction",
      schema: dailySalesSearchSchema,
      key: "sortDir",
      values: ["asc", "desc"],
    },
    {
      name: "monthly mode",
      schema: monthlySalesSearchSchema,
      key: "mode",
      values: ["by_product", "by_department"],
    },
    {
      name: "monthly sort",
      schema: monthlySalesSearchSchema,
      key: "sortBy",
      values: ["name", "quantity", "amount", "prev_month_diff"],
    },
    {
      name: "monthly direction",
      schema: monthlySalesSearchSchema,
      key: "sortDir",
      values: ["asc", "desc"],
    },
    {
      name: "stock status",
      schema: stockInquirySearchSchema,
      key: "status",
      values: ["all", "stockout", "low_stock"],
    },
  ])(
    "$name accepts every owner variant and catches an invalid value",
    ({ schema, key, values }) => {
      for (const value of values) {
        expect(schema.parse({ [key]: value })).toMatchObject({ [key]: value });
      }
      expect(schema.parse({ [key]: "__invalid__" })).toMatchObject({ [key]: undefined });
    },
  );

  it("inventory record formatters do not depend on the all sentinel being first", () => {
    const typeOptions = INVENTORY_RECORD_TYPE_OPTIONS as unknown as {
      value: string;
      label: string;
    }[];
    const statusOptions = INVENTORY_RECORD_STATUS_OPTIONS as unknown as {
      value: string;
      label: string;
    }[];
    const [typeSentinel] = typeOptions.splice(0, 1);
    const [statusSentinel] = statusOptions.splice(0, 1);
    typeOptions.push(typeSentinel);
    statusOptions.push(statusSentinel);

    try {
      expect(formatRecordType("receiving_record")).toBe("入庫");
      expect(formatRecordStatus("active")).toBe("有効");
      expect(formatRecordType("all")).toBe("all");
      expect(formatRecordStatus("all")).toBe("all");
    } finally {
      typeOptions.pop();
      statusOptions.pop();
      typeOptions.unshift(typeSentinel);
      statusOptions.unshift(statusSentinel);
    }
  });
});

describe("finite search source ownership", () => {
  const repoRoot = process.cwd();
  const source = (path: string) => readFileSync(join(repoRoot, path), "utf8");

  it.each([
    ["src/routes/products/index.tsx", "productListSearchSchema"],
    ["src/routes/stock/$code.movements.tsx", "stockMovementsSearchSchema"],
    ["src/routes/inventory/records.tsx", "inventoryRecordsSearchSchema"],
    ["src/routes/reports/daily.tsx", "dailySalesSearchSchema"],
    ["src/routes/reports/monthly.tsx", "monthlySalesSearchSchema"],
    ["src/routes/stock/index.tsx", "stockInquirySearchSchema"],
  ])("%s imports and wires its feature schema without a local enum", (path, schemaName) => {
    const routeSource = source(path);
    expect(routeSource).toMatch(
      new RegExp(`import\\s*\\{[\\s\\S]*?${schemaName}[\\s\\S]*?\\}\\s*from`),
    );
    expect(routeSource).toContain(`validateSearch: ${schemaName}`);
    expect(routeSource).not.toContain("z.enum(");
  });

  it.each([
    [
      "src/features/products/ProductListPage.tsx",
      "PRODUCT_DISCONTINUED_OPTIONS",
      "const discontinuedOptions",
    ],
    [
      "src/features/stock-movements/StockMovementsPage.tsx",
      "MOVEMENT_TYPE_OPTIONS",
      "const movementTypeOptions",
    ],
    [
      "src/features/inventory-records/InventoryRecordsPage.tsx",
      "INVENTORY_RECORD_TYPE_OPTIONS",
      '<option value="receiving_record"',
    ],
    [
      "src/features/daily-sales/components/ProductTable.tsx",
      "DAILY_SORT_DESCRIPTORS",
      'column="product_code"',
    ],
    [
      "src/features/monthly-sales/components/ModeTabs.tsx",
      "MONTHLY_MODE_DESCRIPTORS",
      "const modeOptions = [",
    ],
    [
      "src/features/monthly-sales/components/ProductRankingTable.tsx",
      "monthlySortDescriptorsForMode",
      'column="quantity"',
    ],
    [
      "src/features/monthly-sales/components/DepartmentTable.tsx",
      "monthlySortDescriptorsForMode",
      'column="amount"',
    ],
    [
      "src/features/stock-inquiry/components/StatusChips.tsx",
      "STOCK_FILTER_DESCRIPTORS",
      "const CHIPS",
    ],
  ])("%s renders choices from the feature owner", (path, ownerName, forbiddenFragment) => {
    const componentSource = source(path);
    expect(componentSource).toContain(ownerName);
    expect(componentSource).not.toContain(forbiddenFragment);
  });
});
