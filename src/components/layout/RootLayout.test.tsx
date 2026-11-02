import {
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  RouterProvider,
} from "@tanstack/react-router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { resumeAutoBackupCheck } from "@/features/backup-restore/useAutoBackupCheck";
import { commands } from "@/lib/bindings";

import { RootLayout } from "./RootLayout";

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ setTitle: vi.fn().mockResolvedValue(undefined) }),
}));

vi.mock("@tanstack/react-router-devtools", () => ({
  TanStackRouterDevtools: () => null,
}));

vi.mock("./Sidebar", () => ({
  Sidebar: () => <nav aria-label="test-sidebar" />,
}));

vi.mock("@/components/ui/sonner", () => ({
  Toaster: () => null,
}));

vi.mock("@/components/ui/tooltip", () => ({
  TooltipProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}));

vi.mock("@/features/shortcuts", () => ({
  ShortcutsDialog: () => null,
  useShortcutsDialog: () => ({ open: false, setOpen: vi.fn() }),
}));

// UI-11b-D13: RootLayout は実 hook の useAutoBackupCheck を呼ぶため、hook は mock せず command だけを mock する
vi.mock("@/lib/bindings", () => ({
  commands: { checkAutoBackup: vi.fn() },
}));

const mockCheckAutoBackup = vi.mocked(commands.checkAutoBackup);

function createLayoutRouter() {
  const outerRoute = createRootRoute({ component: () => <Outlet /> });
  const layoutRoute = createRoute({
    getParentRoute: () => outerRoute,
    id: "layout",
    component: RootLayout,
  });
  const indexRoute = createRoute({
    getParentRoute: () => layoutRoute,
    path: "/",
    component: () => <p>test outlet</p>,
  });
  const otherRoute = createRoute({
    getParentRoute: () => layoutRoute,
    path: "/stock",
    component: () => <p>other outlet</p>,
  });
  return createRouter({
    routeTree: outerRoute.addChildren([layoutRoute.addChildren([indexRoute, otherRoute])]),
    history: createMemoryHistory({ initialEntries: ["/"] }),
  });
}

function renderLayout() {
  const router = createLayoutRouter();
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const view = render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return { router, ...view };
}

beforeEach(() => {
  resumeAutoBackupCheck();
  mockCheckAutoBackup.mockReset();
  mockCheckAutoBackup.mockResolvedValue({ status: "ok", data: false });
});

afterEach(() => {
  vi.useRealTimers();
});

describe("UI-12 / DSR-17 RootLayout scroll container", () => {
  it("T6: gives the persistent main element its restoration id", async () => {
    const { container } = renderLayout();

    await screen.findByText("test outlet");
    expect(container.querySelector("main")).toHaveAttribute("data-scroll-restoration-id", "main");
  });
});

describe("UI-12 / UI-09a-D18 RootLayout print notice", () => {
  it("UI-09a-D18: どの画面にも印刷のときだけ出す案内を 1 行置く", async () => {
    const { container } = renderLayout();
    await screen.findByText("test outlet");

    const notice = screen.getByText(
      "この画面の印刷には対応していません。日報は日次売上の画面の「日報を印刷」から印刷してください。",
    );
    // #root の外（body の直下）に置き、画面には出さず印刷のときだけ出す
    expect(container).not.toContainElement(notice);
    expect(notice.parentElement).toBe(document.body);
    expect(notice).toHaveClass("hidden", "print:block");
  });
});

describe("UI-12 / UI-11b-D13 RootLayout auto backup check", () => {
  it("UI-12 / UI-11b-D13: route を移っても自動バックアップの確認が続く", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    const { router } = renderLayout();
    await screen.findByText("test outlet");

    await vi.advanceTimersByTimeAsync(60_000);
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(1);

    await act(async () => {
      await router.navigate({ to: "/stock" });
    });
    await screen.findByText("other outlet");

    await vi.advanceTimersByTimeAsync(60_000);
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(2);
  });
});
