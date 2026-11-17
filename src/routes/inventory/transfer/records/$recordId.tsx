// src/routes/inventory/transfer/records/$recordId.tsx
//
// UI-16b 詰め替え・切り分けの記録の詳細 route。
// 設計: docs/function-design/79-ui-stock-transfer.md §79.8

import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";

import { TransferRecordDetailPage } from "@/features/inventory-records/TransferRecordDetailPage";
import { utf16String } from "@/lib/utf16-string";

const searchSchema = z.object({
  returnTo: utf16String({ max: 500 }).optional().catch(undefined),
});

export const Route = createFileRoute("/inventory/transfer/records/$recordId")({
  validateSearch: searchSchema,
  component: RouteComponent,
});

function RouteComponent() {
  const { recordId } = Route.useParams();
  const search = Route.useSearch();
  return <TransferRecordDetailPage recordId={Number(recordId)} returnTo={search.returnTo} />;
}
