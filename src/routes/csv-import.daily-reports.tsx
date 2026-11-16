// src/routes/csv-import.daily-reports.tsx
//
// UI-07-D19「日報の取込みの記録」route。search `page` の schema は feature の types が持つ（UI-STATE-D2）。
// 設計: docs/function-design/55-ui-csv-import.md §55.12

import { createFileRoute } from "@tanstack/react-router";

import { DailyReportImportRecordsPage } from "@/features/daily-report-import/DailyReportImportRecordsPage";
import { dailyReportImportRecordsSearchSchema } from "@/features/daily-report-import/types";

export const Route = createFileRoute("/csv-import/daily-reports")({
  validateSearch: dailyReportImportRecordsSearchSchema,
  component: RouteComponent,
});

function RouteComponent() {
  const { page } = Route.useSearch();
  const navigate = Route.useNavigate();
  return (
    <DailyReportImportRecordsPage
      page={page ?? 1}
      onPageChange={(next) => {
        void navigate({ search: { page: next } });
      }}
    />
  );
}
