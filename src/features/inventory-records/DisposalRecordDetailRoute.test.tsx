import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, createRouter, RouterProvider } from "@tanstack/react-router";
import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { routeTree } from "@/routeTree.gen";
import { commands } from "@/lib/bindings";
import type { DisposalRecordDetail } from "@/lib/bindings";

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ setTitle: vi.fn().mockResolvedValue(undefined) }),
}));

vi.mock("@/lib/bindings", () => ({
  commands: {
    getDisposalRecord: vi.fn(),
    listDisposals: vi.fn(),
  },
}));

const mockGetDisposalRecord = vi.mocked(commands.getDisposalRecord);
const mockListDisposals = vi.mocked(commands.listDisposals);

function makeDetail(): DisposalRecordDetail {
  return {
    id: 3,
    disposal_date: "2026-06-27",
    status: "active",
    canceled_at: null,
    cancel_reason: null,
    created_at: "2026-06-27T10:00:00",
    total_loss_cost: 8080,
    items: [
      {
        id: 1,
        product_code: "L3IR-K001",
        product_name: "L3確認 毛糸 赤",
        department_name: "毛糸",
        stock_unit: "pcs",
        disposal_type: "damage",
        quantity: 2,
        cost_price_centi: 440,
        reason: "L3確認: 袋破れ",
        line_loss_cost_centi: "88000",
      },
      {
        id: 2,
        product_code: "L3IR-N001",
        product_name: "L3確認 布 花柄",
        department_name: "布",
        stock_unit: "m",
        disposal_type: "disposal",
        quantity: 120,
        cost_price_centi: 60,
        reason: "L3確認: 汚れ",
        line_loss_cost_centi: "720000",
      },
    ],
    movements: [],
  };
}

function renderRoute(initialPath: string) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  const router = createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: [initialPath] }),
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  mockGetDisposalRecord.mockReset();
  mockListDisposals.mockReset();
  mockListDisposals.mockResolvedValue({
    status: "ok",
    data: { items: [], total_count: 0, page: 1, per_page: 10 },
  });
});

describe("DisposalRecordDetail route", () => {
  it("REQ-204 keeps the disposal entry page at /inventory/disposal after route nesting", async () => {
    renderRoute("/inventory/disposal");

    expect(await screen.findByRole("heading", { name: "廃棄・破損" })).toBeInTheDocument();
    expect(screen.getByLabelText("廃棄・破損商品検索")).toBeInTheDocument();
  });

  it("REQ-206 renders the disposal detail page at /inventory/disposal/records/$recordId", async () => {
    mockGetDisposalRecord.mockResolvedValue({ status: "ok", data: makeDetail() });

    renderRoute("/inventory/disposal/records/3");

    expect(await screen.findByRole("heading", { name: "廃棄・破損 #3" })).toBeInTheDocument();
    expect(screen.getByText("L3確認 毛糸 赤")).toBeInTheDocument();
    expect(screen.getByText("¥8,080")).toBeInTheDocument();
  });
});
