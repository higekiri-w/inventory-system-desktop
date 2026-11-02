import { describe, expect, it } from "vitest";

import { d052InvalidationOracle } from "@/test/invalidation-oracle";

import { invalidationContract } from "./invalidation-contract";

describe("UI-07 D-052 invalidation SSOT shape", () => {
  it("REQ-907 B-I1: defines all 23 mutation entries exactly once against the independent oracle", () => {
    expect(Object.keys(invalidationContract).sort()).toEqual(
      [
        "productCreate",
        "productCreateSharedJan",
        "productUpdate",
        "productImport",
        "receiving",
        "returnExchange",
        "manualSale",
        "disposal",
        "csvImportCommit",
        "csvImportRollback",
        "dailyReportImport",
        "stocktakeStart",
        "stocktakeCountUpdate",
        "stocktakeComplete",
        "integrityFix",
        "thresholdSave",
        "pluExportConfirm",
        "pluRegisterSnapshot",
        "pluExportPrepare",
        "pluBulkTarget",
        "productPriceRevise",
        "supplierRename",
        "supplierMerge",
      ].sort(),
    );
    const actualEntries = [
      invalidationContract.productCreate(),
      invalidationContract.productCreateSharedJan(),
      invalidationContract.productUpdate("P-001"),
      invalidationContract.productImport(),
      invalidationContract.receiving(),
      invalidationContract.returnExchange(false),
      invalidationContract.manualSale("2026-07-23"),
      invalidationContract.disposal(),
      invalidationContract.csvImportCommit(),
      invalidationContract.csvImportRollback(),
      invalidationContract.dailyReportImport(),
      invalidationContract.stocktakeStart(),
      invalidationContract.stocktakeCountUpdate(),
      invalidationContract.stocktakeComplete(),
      invalidationContract.integrityFix(),
      invalidationContract.thresholdSave(),
      invalidationContract.pluExportConfirm(),
      invalidationContract.pluRegisterSnapshot(),
      invalidationContract.pluExportPrepare(),
      invalidationContract.pluBulkTarget(),
      invalidationContract.productPriceRevise("P-001"),
      invalidationContract.supplierRename(),
      invalidationContract.supplierMerge(),
    ];
    const expectedEntries = [
      d052InvalidationOracle.productCreate(),
      d052InvalidationOracle.productCreateSharedJan(),
      d052InvalidationOracle.productUpdate("P-001"),
      d052InvalidationOracle.productImport(),
      d052InvalidationOracle.receiving(),
      d052InvalidationOracle.returnExchange(false),
      d052InvalidationOracle.manualSale("2026-07-23"),
      d052InvalidationOracle.disposal(),
      d052InvalidationOracle.csvImportCommit(),
      d052InvalidationOracle.csvImportRollback(),
      d052InvalidationOracle.dailyReportImport(),
      d052InvalidationOracle.stocktakeStart(),
      d052InvalidationOracle.stocktakeCountUpdate(),
      d052InvalidationOracle.stocktakeComplete(),
      d052InvalidationOracle.integrityFix(),
      d052InvalidationOracle.thresholdSave(),
      d052InvalidationOracle.pluExportConfirm(),
      d052InvalidationOracle.pluRegisterSnapshot(),
      d052InvalidationOracle.pluExportPrepare(),
      d052InvalidationOracle.pluBulkTarget(),
      d052InvalidationOracle.productPriceRevise("P-001"),
      d052InvalidationOracle.supplierRename(),
      d052InvalidationOracle.supplierMerge(),
    ];

    expect(actualEntries).toHaveLength(23);
    expect(actualEntries.map((entry) => entry.map((key) => JSON.stringify(key)).sort())).toEqual(
      expectedEntries.map((entry) => entry.map((key) => JSON.stringify(key)).sort()),
    );
    for (const entry of actualEntries) {
      expect(entry.length).toBeGreaterThan(0);
      expect(new Set(entry.map((key) => JSON.stringify(key))).size).toBe(entry.length);
    }
  });
});
