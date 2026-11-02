import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";

import { StocktakePage } from "@/features/stocktake/StocktakePage";
import type { StocktakeSearch } from "@/features/stocktake/types";

const booleanSearch = z.union([
  z.boolean(),
  z.enum(["true", "false"]).transform((value) => value === "true"),
]);

const searchSchema = z.object({
  dept: z.coerce.number().int().positive().optional().catch(undefined),
  counted_only: booleanSearch.optional().catch(undefined),
  page: z.coerce.number().int().positive().optional().catch(undefined),
  product: z.string().min(1).optional().catch(undefined),
  returnTo: z.string().optional().catch(undefined),
});

export const Route = createFileRoute("/stocktake/")({
  validateSearch: searchSchema,
  component: RouteComponent,
});

function RouteComponent() {
  const search = Route.useSearch();
  const navigate = Route.useNavigate();
  const handleSearchChange = (updater: (prev: StocktakeSearch) => StocktakeSearch) => {
    void navigate({ search: (prev) => updater(prev) });
  };
  return (
    <StocktakePage
      search={search}
      onSearchChange={handleSearchChange}
      onReturnTo={(to) => {
        void navigate({ to });
      }}
    />
  );
}
