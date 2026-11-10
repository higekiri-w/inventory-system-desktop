import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";
import { utf16String } from "@/lib/utf16-string";

import { StocktakeRecordDetailPage } from "@/features/inventory-records/StocktakeRecordDetailPage";

const searchSchema = z.object({
  returnTo: utf16String({ max: 500 }).optional().catch(undefined),
});

export const Route = createFileRoute("/stocktake/records/$stocktakeId")({
  validateSearch: searchSchema,
  component: RouteComponent,
});

function RouteComponent() {
  const { stocktakeId } = Route.useParams();
  const search = Route.useSearch();
  return <StocktakeRecordDetailPage stocktakeId={Number(stocktakeId)} returnTo={search.returnTo} />;
}
