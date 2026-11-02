import { createMemoryHistory, createRouter, RouterProvider } from "@tanstack/react-router";
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ setTitle: vi.fn().mockResolvedValue(undefined) }),
}));

vi.mock("@/features/backup-restore/useAutoBackupCheck", () => ({
  useAutoBackupCheck: () => undefined,
  resumeAutoBackupCheck: vi.fn(),
  suspendAutoBackupCheckAndWait: vi.fn().mockResolvedValue(undefined),
  holdAutoBackupCheckForRestore: vi.fn(),
  releaseAutoBackupCheckAfterRestore: vi.fn(),
}));

vi.mock("@/features/receiving/ReceivingPage", () => ({
  ReceivingPage: () => <div>入庫記録ページ marker</div>,
}));

vi.mock("@/features/return-exchange/ReturnExchangePage", () => ({
  ReturnExchangePage: () => <div>返品・交換ページ marker</div>,
}));

vi.mock("@/features/manual-sale/ManualSalePage", () => ({
  ManualSalePage: () => <div>手動販売出庫ページ marker</div>,
}));

vi.mock("./ReceivingRecordDetailPage", () => ({
  ReceivingRecordDetailPage: ({ recordId }: { recordId: number }) => (
    <div>入庫記録詳細 marker #{recordId}</div>
  ),
}));

vi.mock("./ReturnRecordDetailPage", () => ({
  ReturnRecordDetailPage: ({ recordId }: { recordId: number }) => (
    <div>返品・交換詳細 marker #{recordId}</div>
  ),
}));

vi.mock("./ManualSaleRecordDetailPage", () => ({
  ManualSaleRecordDetailPage: ({ recordId }: { recordId: number }) => (
    <div>手動販売出庫詳細 marker #{recordId}</div>
  ),
}));

import { routeTree } from "@/routeTree.gen";

function renderRoute(initialPath: string) {
  const router = createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: [initialPath] }),
  });

  return render(<RouterProvider router={router} />);
}

describe("other inventory record detail routes (REQ-201 / REQ-202 / REQ-203 / REQ-206)", () => {
  it.each([
    ["/inventory/receiving/records/12", "入庫記録詳細 marker #12", "入庫記録ページ marker"],
    ["/inventory/return/records/22", "返品・交換詳細 marker #22", "返品・交換ページ marker"],
    [
      "/inventory/manual-sale/records/32",
      "手動販売出庫詳細 marker #32",
      "手動販売出庫ページ marker",
    ],
  ])(
    "renders the detail route for %s instead of the entry page",
    async (path, detailText, pageText) => {
      renderRoute(path);

      expect(await screen.findByText(detailText)).toBeInTheDocument();
      expect(screen.queryByText(pageText)).not.toBeInTheDocument();
    },
  );
});
