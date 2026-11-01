import { expect } from "vitest";

import { queryKeys } from "@/lib/query-keys";

export type InvalidationKey = readonly unknown[];

/**
 * D-052-C1〜C22 と C23（D-141）を test 側へ独立転記した oracle。
 * production の invalidation-contract.ts を参照してはならない。
 */
export const d052InvalidationOracle = {
  productCreate: () => [
    queryKeys.productList.root(),
    queryKeys.lowStock(false),
    queryKeys.stockInquiryRoot(),
    queryKeys.pluDirty(),
    queryKeys.stockMovements.root(),
    queryKeys.stocktake.itemsRoot(),
    queryKeys.inventoryRecords.stocktakeDetailRoot(),
  ],
  // C23（D-141）: UI_TECH_STACK 冒頭の段落「商品の登録の entry に要再確認の一覧と商品フォームを足した別の entry」
  productCreateSharedJan: () => [
    queryKeys.productList.root(),
    queryKeys.lowStock(false),
    queryKeys.stockInquiryRoot(),
    queryKeys.pluDirty(),
    queryKeys.stockMovements.root(),
    queryKeys.stocktake.itemsRoot(),
    queryKeys.inventoryRecords.stocktakeDetailRoot(),
    queryKeys.productForm.root(),
    queryKeys.stocktake.recheckList(),
  ],
  productUpdate: (productCode: string) => [
    queryKeys.productList.root(),
    queryKeys.productForm.product(productCode),
    queryKeys.lowStock(false),
    queryKeys.stockInquiryRoot(),
    queryKeys.pluDirty(),
    queryKeys.pluSlotSummary(),
    queryKeys.stockMovements.root(),
    queryKeys.stocktake.itemsRoot(),
    queryKeys.stocktake.recheckList(),
  ],
  productImport: () => [
    queryKeys.productList.root(),
    queryKeys.lowStock(false),
    queryKeys.stockInquiryRoot(),
    queryKeys.pluDirty(),
    queryKeys.stockMovements.root(),
    queryKeys.stocktake.itemsRoot(),
    queryKeys.productForm.root(),
    queryKeys.inventoryRecords.stocktakeDetailRoot(),
    queryKeys.stocktake.recheckList(),
  ],
  receiving: () => [
    queryKeys.receivings.root(),
    queryKeys.inventoryRecords.root(),
    queryKeys.productList.root(),
    queryKeys.lowStock(false),
    queryKeys.stockInquiryRoot(),
    queryKeys.stockMovements.root(),
    queryKeys.productForm.root(),
    queryKeys.stocktake.itemsRoot(),
    queryKeys.stocktake.recheckList(),
  ],
  returnExchange: (registerProcessed: boolean) => [
    queryKeys.returns.root(),
    queryKeys.inventoryRecords.root(),
    ...(!registerProcessed
      ? [
          queryKeys.productList.root(),
          queryKeys.lowStock(false),
          queryKeys.stockInquiryRoot(),
          queryKeys.stockMovements.root(),
          queryKeys.productForm.root(),
          queryKeys.stocktake.itemsRoot(),
          queryKeys.stocktake.recheckList(),
        ]
      : []),
  ],
  manualSale: (saleDate: string) => [
    queryKeys.inventoryRecords.root(),
    queryKeys.productList.root(),
    queryKeys.lowStock(false),
    queryKeys.stockInquiryRoot(),
    queryKeys.dailySales(saleDate),
    queryKeys.monthlySalesRoot(),
    queryKeys.stockMovements.root(),
    queryKeys.productForm.root(),
    queryKeys.stocktake.itemsRoot(),
    queryKeys.stocktake.recheckList(),
  ],
  disposal: () => [
    queryKeys.disposals.root(),
    queryKeys.inventoryRecords.root(),
    queryKeys.productList.root(),
    queryKeys.lowStock(false),
    queryKeys.stockInquiryRoot(),
    queryKeys.stockMovements.root(),
    queryKeys.productForm.root(),
    queryKeys.stocktake.itemsRoot(),
    queryKeys.stocktake.recheckList(),
  ],
  csvImportCommit: () => [
    queryKeys.csvImportLists(),
    queryKeys.inventoryRecords.root(),
    queryKeys.dailySalesRoot(),
    queryKeys.lowStock(false),
    queryKeys.stockInquiryRoot(),
    queryKeys.productList.root(),
    queryKeys.monthlySalesRoot(),
    queryKeys.stockMovements.root(),
    queryKeys.productForm.root(),
    queryKeys.stocktake.itemsRoot(),
    queryKeys.stocktake.recheckList(),
  ],
  csvImportRollback: () => [
    queryKeys.csvImportLists(),
    queryKeys.inventoryRecords.root(),
    queryKeys.dailySalesRoot(),
    queryKeys.lowStock(false),
    queryKeys.stockInquiryRoot(),
    queryKeys.productList.root(),
    queryKeys.monthlySalesRoot(),
    queryKeys.stockMovements.root(),
    queryKeys.productForm.root(),
    queryKeys.stocktake.itemsRoot(),
    queryKeys.stocktake.recheckList(),
  ],
  dailyReportImport: () => [
    queryKeys.dailyReportImportLists(),
    queryKeys.dailySalesRoot(),
    queryKeys.monthlySalesRoot(),
  ],
  stocktakeStart: () => [
    queryKeys.stocktake.status(),
    queryKeys.stocktake.itemsRoot(),
    queryKeys.inventoryRecords.root(),
    queryKeys.stocktake.recheckList(),
  ],
  stocktakeCountUpdate: () => [
    queryKeys.stocktake.itemsRoot(),
    queryKeys.productList.root(),
    queryKeys.lowStock(false),
    queryKeys.stockInquiryRoot(),
    queryKeys.stockMovements.root(),
    queryKeys.productForm.root(),
    queryKeys.stocktake.recheckList(),
  ],
  stocktakeComplete: () => [
    queryKeys.stocktake.status(),
    queryKeys.stocktake.itemsRoot(),
    queryKeys.stocktake.lastCompleted(),
    queryKeys.productList.root(),
    queryKeys.lowStock(false),
    queryKeys.stockInquiryRoot(),
    queryKeys.stockMovements.root(),
    queryKeys.productForm.root(),
    queryKeys.inventoryRecords.root(),
    queryKeys.stocktake.recheckList(),
  ],
  integrityFix: () => [
    queryKeys.productList.root(),
    queryKeys.lowStock(false),
    queryKeys.stockInquiryRoot(),
    queryKeys.stockMovements.root(),
    queryKeys.productForm.root(),
    queryKeys.stocktake.itemsRoot(),
    queryKeys.stocktake.recheckList(),
  ],
  thresholdSave: () => [
    queryKeys.thresholdSettings.settings(),
    queryKeys.lowStock(false),
    queryKeys.stockInquiryRoot(),
  ],
  pluExportConfirm: () => [
    queryKeys.pluDirty(),
    queryKeys.pluSlotSummary(),
    queryKeys.productList.root(),
    queryKeys.stockMovements.root(),
    queryKeys.productForm.root(),
    queryKeys.lowStock(false),
    queryKeys.stockInquiryRoot(),
  ],
  pluRegisterSnapshot: () => [queryKeys.pluSlotSummary()],
  pluExportPrepare: () => [queryKeys.pluSlotSummary()],
  pluBulkTarget: () => [
    queryKeys.productList.root(),
    queryKeys.pluDirty(),
    queryKeys.productForm.root(),
    queryKeys.pluSlotSummary(),
  ],
  productPriceRevise: (productCode: string) => [
    ["product-list"],
    ["product-form", "product", { productCode }],
    ["plu-dirty"],
    ["price-revision"],
  ],
  supplierRename: () => [
    ["product-form"],
    ["price-revision"],
    ["suppliers"],
    ["product-list"],
    ["products", "low-stock", { includeDiscontinued: false }],
    ["stock-inquiry"],
    ["receivings"],
    ["inventory-records"],
  ],
  supplierMerge: () => [
    ["product-form"],
    ["price-revision"],
    ["suppliers"],
    ["product-list"],
    ["products", "low-stock", { includeDiscontinued: false }],
    ["stock-inquiry"],
    ["receivings"],
    ["inventory-records"],
  ],
} satisfies Record<string, (...args: never[]) => InvalidationKey[]>;

function stableKey(key: InvalidationKey): string {
  return JSON.stringify(key);
}

export function expectExactInvalidations(
  calls: readonly unknown[][],
  expectedKeys: readonly InvalidationKey[],
): void {
  const actualKeys = calls.map((call) => {
    const filters = call[0] as { queryKey?: InvalidationKey } | undefined;
    if (filters?.queryKey === undefined) throw new Error("queryKey のない invalidate 呼出しです");
    return filters.queryKey;
  });

  expect(actualKeys.map(stableKey).sort()).toEqual(expectedKeys.map(stableKey).sort());
}
