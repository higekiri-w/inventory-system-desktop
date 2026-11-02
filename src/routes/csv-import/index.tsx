import { createFileRoute } from "@tanstack/react-router";

import { CsvImportPage } from "@/features/csv-import/CsvImportPage";
import { csvImportSearchSchema } from "@/features/csv-import/types";

/// UI-07 CSV取込み画面の index route。search `tab` の値の集合は feature の types が持つ（UI-STATE-D2）。
/// 設計: docs/function-design/55-ui-csv-import.md §55.1 接続点 / §55.11
export const Route = createFileRoute("/csv-import/")({
  validateSearch: csvImportSearchSchema,
  component: RouteComponent,
});

function RouteComponent() {
  const { tab } = Route.useSearch();
  return <CsvImportPage tab={tab} />;
}
