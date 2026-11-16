// src/components/sales/TabsHeader.test.tsx
//
// PR-1 (B1 nav fix): activeOptions.includeSearch:false により search params 付き URL でも
// 売上タブ (日次/月次) の active が維持されることを data-status 属性で検証する。
// 設計: docs/plans/2026-05-22-tone-and-nav-fix.md PR-1
//
// 回帰検出力のため initialEntries に search params を必ず載せる (R2-1):
// search なし URL では includeSearch true/false どちらでも日次 Link が active になり、
// 修正前コード (includeSearch:true デフォルト) でも pass して回帰を検出できない。
// active は DOM クラス文字列でなく data-status="active" 属性で検証する (クラス hardcode は脆い)。

import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import {
  createRootRoute,
  createRoute,
  createRouter,
  createMemoryHistory,
  RouterProvider,
} from "@tanstack/react-router";

import { TabsHeader } from "./TabsHeader";

function renderAt(initialPath: string, month?: string) {
  const rootRoute = createRootRoute({ component: () => <TabsHeader month={month} /> });
  const dailyRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/reports/daily",
    component: () => null,
  });
  const monthlyRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/reports/monthly",
    component: () => null,
  });
  const routeTree = rootRoute.addChildren([dailyRoute, monthlyRoute]);
  const router = createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: [initialPath] }),
  });
  return render(<RouterProvider router={router} />);
}

describe("TabsHeader (PR-1 B1: search params 付き URL の active 維持)", () => {
  it("REQ-501: /reports/daily?date=... でも日次タブが active を維持し月次は非 active", async () => {
    renderAt("/reports/daily?date=2026-03-22&sortBy=quantity&sortDir=desc");

    const dailyLink = await screen.findByRole("link", { name: "日次" });
    const monthlyLink = screen.getByRole("link", { name: "月次" });

    expect(dailyLink).toHaveAttribute("data-status", "active");
    expect(monthlyLink).not.toHaveAttribute("data-status", "active");
  });

  it("REQ-502: /reports/monthly?month=... でも月次タブが active を維持し日次は非 active", async () => {
    renderAt("/reports/monthly?month=2026-05&mode=by_product");

    const monthlyLink = await screen.findByRole("link", { name: "月次" });
    const dailyLink = screen.getByRole("link", { name: "日次" });

    expect(monthlyLink).toHaveAttribute("data-status", "active");
    expect(dailyLink).not.toHaveAttribute("data-status", "active");
  });
});

describe("TabsHeader (REQ-502 UI-09b-D11: 日次から月次への戻り)", () => {
  it("T1: month を渡すと月次の link が その月を開き、渡さないと今の href のまま", async () => {
    const { unmount } = renderAt("/reports/daily?date=2026-03-23", "2026-03");
    expect(await screen.findByRole("link", { name: "月次" })).toHaveAttribute(
      "href",
      "/reports/monthly?month=2026-03",
    );
    expect(screen.getByRole("link", { name: "日次" })).toHaveAttribute("href", "/reports/daily");
    unmount();

    renderAt("/reports/daily?date=2026-03-23");
    expect(await screen.findByRole("link", { name: "月次" })).toHaveAttribute(
      "href",
      "/reports/monthly",
    );
    expect(screen.getByRole("link", { name: "日次" })).toHaveAttribute("href", "/reports/daily");
  });
});
