import { readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";

const SOURCE_ROOT = join(process.cwd(), "src");
const FEATURES_ROOT = join(SOURCE_ROOT, "features");

const APPLIED_PAGES = [
  { component: "DisposalPage", path: "src/features/disposal/DisposalPage.tsx" },
  { component: "ManualSalePage", path: "src/features/manual-sale/ManualSalePage.tsx" },
  { component: "ProductFormPage", path: "src/features/products/ProductFormPage.tsx" },
  { component: "ReceivingPage", path: "src/features/receiving/ReceivingPage.tsx" },
  {
    component: "ReturnExchangePage",
    path: "src/features/return-exchange/ReturnExchangePage.tsx",
  },
  {
    component: "ThresholdSettingsPage",
    path: "src/features/threshold-settings/ThresholdSettingsPage.tsx",
  },
] as const;

const EXCLUDED_PAGES = [
  { component: "BackupRestorePage", path: "src/features/backup-restore/BackupRestorePage.tsx" },
  { component: "CsvImportPage", path: "src/features/csv-import/CsvImportPage.tsx" },
  {
    component: "DailyReportImportPage",
    path: "src/features/daily-report-import/DailyReportImportPage.tsx",
  },
  { component: "DailySalesPage", path: "src/features/daily-sales/DailySalesPage.tsx" },
  { component: "HomePage", path: "src/features/home/HomePage.tsx" },
  {
    component: "IntegrityCheckPage",
    path: "src/features/integrity-check/IntegrityCheckPage.tsx",
  },
  {
    component: "CsvImportRecordDetailPage",
    path: "src/features/inventory-records/CsvImportRecordDetailPage.tsx",
  },
  {
    component: "DisposalRecordDetailPage",
    path: "src/features/inventory-records/DisposalRecordDetailPage.tsx",
  },
  {
    component: "InventoryRecordsPage",
    path: "src/features/inventory-records/InventoryRecordsPage.tsx",
  },
  {
    component: "ManualSaleRecordDetailPage",
    path: "src/features/inventory-records/ManualSaleRecordDetailPage.tsx",
  },
  {
    component: "ReceivingRecordDetailPage",
    path: "src/features/inventory-records/ReceivingRecordDetailPage.tsx",
  },
  {
    component: "ReturnRecordDetailPage",
    path: "src/features/inventory-records/ReturnRecordDetailPage.tsx",
  },
  {
    component: "StocktakeRecordDetailPage",
    path: "src/features/inventory-records/StocktakeRecordDetailPage.tsx",
  },
  { component: "MonthlySalesPage", path: "src/features/monthly-sales/MonthlySalesPage.tsx" },
  { component: "OperationLogsPage", path: "src/features/operation-logs/OperationLogsPage.tsx" },
  { component: "PluExportPage", path: "src/features/plu-export/PluExportPage.tsx" },
  { component: "PriceRevisionPage", path: "src/features/products/PriceRevisionPage.tsx" },
  { component: "ProductImportPage", path: "src/features/products/ProductImportPage.tsx" },
  { component: "ProductListPage", path: "src/features/products/ProductListPage.tsx" },
  { component: "StockInquiryPage", path: "src/features/stock-inquiry/StockInquiryPage.tsx" },
  {
    component: "StockMovementsPage",
    path: "src/features/stock-movements/StockMovementsPage.tsx",
  },
  { component: "StocktakePage", path: "src/features/stocktake/StocktakePage.tsx" },
  // prettier-ignore
  { component: "SupplierManagementPage", path: "src/features/suppliers/SupplierManagementPage.tsx" },
] as const;

function collectSourceFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    return entry.isDirectory() ? collectSourceFiles(path) : [path];
  });
}

function readRepoFile(path: string): string {
  return readFileSync(join(process.cwd(), path), "utf8");
}

describe("unsaved changes guard sweep (UI-12/UI-USW-D3 / SPEC-UISN-2)", () => {
  it("T16: useBlocker の直接使用は 55 §55.7 の進行中の処理のガードの hook と共通警告 hookだけに限定する", () => {
    const directUsers = collectSourceFiles(SOURCE_ROOT)
      .filter((path) => /\.(ts|tsx)$/.test(path) && !/\.test\.(ts|tsx)$/.test(path))
      .filter((path) => /\buseBlocker\s*\(/.test(readFileSync(path, "utf8")))
      .map((path) => relative(process.cwd(), path))
      .sort();

    expect(directUsers).toEqual([
      "src/features/csv-import/hooks/useCsvImportFlow.ts",
      "src/features/csv-import/hooks/useRollbackCsvImport.ts",
      "src/features/daily-report-import/hooks/useDailyReportImportFlow.ts",
      "src/hooks/useUnsavedChangesWarning.ts",
    ]);
  });

  it("T17: 適用6画面の配線と全Page分類を明示manifestへ完全一致させる", () => {
    for (const { component, path } of APPLIED_PAGES) {
      const source = readRepoFile(path);
      expect(source, `${component} must use the shared hook`).toMatch(
        /useUnsavedChangesWarning\s*\(/,
      );
      expect(source, `${component} must render the shared dialog`).toMatch(
        /<UnsavedChangesDialog\b/,
      );
    }

    const actualPages = collectSourceFiles(FEATURES_ROOT)
      .filter((path) => path.endsWith("Page.tsx"))
      .map((path) => relative(process.cwd(), path))
      .sort();
    const classifiedPages = [...APPLIED_PAGES, ...EXCLUDED_PAGES].map(({ path }) => path).sort();

    expect(actualPages).toEqual(classifiedPages);
  });
});
