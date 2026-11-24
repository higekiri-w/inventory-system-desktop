import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, createRouter, RouterProvider } from "@tanstack/react-router";
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { routeTree } from "@/routeTree.gen";

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ setTitle: vi.fn().mockResolvedValue(undefined) }),
}));

vi.mock("@/features/ej-receipts/EjReceiptsPage", () => ({
  EjReceiptsPage: ({ search }: { search: object }) => (
    <div>
      <h1>控えの画面</h1>
      <output>{JSON.stringify(search)}</output>
    </div>
  ),
}));

async function searchOf(url: string): Promise<string> {
  const router = createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: [url] }),
  });
  const view = render(
    <QueryClientProvider client={new QueryClient()}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  await screen.findByRole("heading", { name: "控えの画面" });
  const text = screen.getByRole("status").textContent ?? "";
  view.unmount();
  return text;
}

describe("UI-09c-D1 /reports/receipts search REQ-908", () => {
  it.each([
    ["/reports/receipts?date=2026-13-01", "{}"],
    ["/reports/receipts?date=2026-02-30", "{}"],
    ["/reports/receipts?date=2025-02-29", "{}"],
    ["/reports/receipts?date=abc", "{}"],
    // Router が数に読む値は捨てる（番号の URL はアプリの中の link が文字列で作る）
    ["/reports/receipts?no=12", "{}"],
    ["/reports/receipts?no=1e2", "{}"],
    ["/reports/receipts?no=100000", "{}"],
    ["/reports/receipts?no=%221234567%22", "{}"],
    ["/reports/receipts?no=a", "{}"],
    // closure P3: date があれば番号を捨ててもその日（date が無いときだけ最新の日）
    ["/reports/receipts?date=2026-11-20&no=12", '{"date":"2026-11-20"}'],
    ["/reports/receipts?date=2024-02-29", '{"date":"2024-02-29"}'],
    ["/reports/receipts?no=000123", '{"no":"000123"}'],
    // link の `search={{ no: "100000" }}` が作る形
    ["/reports/receipts?no=%22100000%22", '{"no":"100000"}'],
  ])("U1: %s → %s", async (url, expected) => {
    expect(await searchOf(url)).toBe(expected);
  });
});
