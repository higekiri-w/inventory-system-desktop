// U1 / C-NAV（79 UI-16-D1・§79.7、REQ-210）: 実 router の上で、サイドバーの入出庫の「廃棄・破損」の次に
// 「詰め替え・切り分け」があり、/inventory/transfer でその項目だけが active になる。

import {
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { render, screen, within } from "@testing-library/react";
import { expect, it } from "vitest";

import { SidebarArea } from "@/components/layout/SidebarArea";
import { navigation } from "@/config/navigation";

it("test_navigation_req210_ui16_sidebar_active_at_inventory_transfer", async () => {
  const inventoryArea = navigation.find((area) => area.id === "inventory");
  if (inventoryArea === undefined) throw new Error("inventory area is required");
  const rootRoute = createRootRoute({ component: () => <SidebarArea area={inventoryArea} /> });
  const transferRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/inventory/transfer",
    component: () => null,
  });
  const router = createRouter({
    routeTree: rootRoute.addChildren([transferRoute]),
    history: createMemoryHistory({ initialEntries: ["/inventory/transfer"] }),
  });
  render(<RouterProvider router={router} />);

  const link = await screen.findByRole("link", { name: "詰め替え・切り分け" });
  expect(link).toHaveAttribute("href", "/inventory/transfer");
  expect(link).toHaveAttribute("aria-current", "page");
  expect(screen.getByRole("link", { name: "廃棄・破損" })).not.toHaveAttribute("aria-current");

  const list = link.closest("ul");
  if (list === null) throw new Error("expected a list of sidebar links");
  const labels = within(list)
    .getAllByRole("link")
    .map((item) => item.textContent);
  expect(labels.indexOf("詰め替え・切り分け")).toBe(labels.indexOf("廃棄・破損") + 1);
});
