// src/routes/inventory/manual-sale.records.$recordId.tsx
//
// REQ-203/REQ-206 手動販売記録詳細 route。

import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";
import { utf16String } from "@/lib/utf16-string";

import { ManualSaleRecordDetailPage } from "@/features/inventory-records/ManualSaleRecordDetailPage";

const searchSchema = z.object({
  returnTo: utf16String({ max: 500 }).optional().catch(undefined),
});

export const Route = createFileRoute("/inventory/manual-sale/records/$recordId")({
  validateSearch: searchSchema,
  component: RouteComponent,
});

function RouteComponent() {
  const { recordId } = Route.useParams();
  const search = Route.useSearch();
  return <ManualSaleRecordDetailPage recordId={Number(recordId)} returnTo={search.returnTo} />;
}
