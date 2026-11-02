// src/routes/products/$code.edit.tsx

import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";

import { ProductFormPage } from "@/features/products/ProductFormPage";
import { parseProductListSearchFromReturnTo } from "@/features/products/lib/return-to";

const searchSchema = z.object({
  returnTo: z.string().optional().catch(undefined),
});

export const Route = createFileRoute("/products/$code/edit")({
  validateSearch: searchSchema,
  component: RouteComponent,
});

function RouteComponent() {
  const { code } = Route.useParams();
  const search = Route.useSearch();
  const navigate = Route.useNavigate();

  return (
    <ProductFormPage
      mode="edit"
      productCode={code}
      returnTo={search.returnTo}
      onNavigateToList={(returnTo) => {
        void navigate({ to: "/products", search: parseProductListSearchFromReturnTo(returnTo) });
      }}
      onNavigateToStocktake={(stocktakeSearch) => {
        void navigate({ to: "/stocktake", search: stocktakeSearch });
      }}
    />
  );
}
