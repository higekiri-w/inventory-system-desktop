// src/routes/inventory/return.records.$recordId.tsx
//
// REQ-202/REQ-206 返品・交換記録詳細 route。

import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";
import { utf16String } from "@/lib/utf16-string";

import { ReturnRecordDetailPage } from "@/features/inventory-records/ReturnRecordDetailPage";

const searchSchema = z.object({
  returnTo: utf16String({ max: 500 }).optional().catch(undefined),
});

export const Route = createFileRoute("/inventory/return/records/$recordId")({
  validateSearch: searchSchema,
  component: RouteComponent,
});

function RouteComponent() {
  const { recordId } = Route.useParams();
  const search = Route.useSearch();
  return <ReturnRecordDetailPage recordId={Number(recordId)} returnTo={search.returnTo} />;
}
