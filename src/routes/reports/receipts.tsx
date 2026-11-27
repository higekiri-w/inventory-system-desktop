// src/routes/reports/receipts.tsx
//
// UI-09c レシートの控え画面のファイルベースルート。search の schema は feature の types が持つ（daily.tsx と同じ形）。
// 設計: docs/SCREEN_DESIGN.md UI-09c-D1、docs/function-design/80-ui-ej-receipts.md §80.2

import { createFileRoute } from "@tanstack/react-router";
import { EjReceiptsPage } from "@/features/ej-receipts/EjReceiptsPage";
import { ejReceiptsSearchSchema, type EjReceiptsSearch } from "@/features/ej-receipts/types";

export const Route = createFileRoute("/reports/receipts")({
  validateSearch: ejReceiptsSearchSchema,
  component: RouteComponent,
});

function RouteComponent() {
  const search = Route.useSearch();
  const navigate = Route.useNavigate();

  const handleSearchChange = (next: EjReceiptsSearch, options: { replace?: boolean } = {}) => {
    void navigate({ search: next, replace: options.replace });
  };

  return <EjReceiptsPage search={search} onSearchChange={handleSearchChange} />;
}
