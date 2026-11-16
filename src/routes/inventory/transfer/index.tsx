// src/routes/inventory/transfer/index.tsx
//
// UI-16 詰め替え・切り分け入力画面の index route。
// 設計: docs/function-design/79-ui-stock-transfer.md §79.2

import { createFileRoute } from "@tanstack/react-router";

import { StockTransferPage } from "@/features/stock-transfer/StockTransferPage";

export const Route = createFileRoute("/inventory/transfer/")({
  component: StockTransferPage,
});
