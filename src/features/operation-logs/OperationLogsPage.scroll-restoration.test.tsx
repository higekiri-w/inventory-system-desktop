// src/features/operation-logs/OperationLogsPage.scroll-restoration.test.tsx
//
// Gated Amendment 3 (A3-b): AC16 / SC10a の決定論的再現 test。
// 本番と同じ createAppRouter（src/lib/app-router.ts）+ 実 routeTree を mount し、
// OperationLogsPage.test.tsx の renderWithRouter（catch-all root route）ではなく
// 実際の TanStack Router scroll restoration（内蔵 onRendered + app-router.ts の独自
// onRendered）の相互作用を経由させる。是正前は main.scrollTop が 640（page 1 の旧位置）
// に固定され、是正後は 0 になることを oracle とする（機序は plan の Gated Amendment 3 節）。

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { createAppRouter } from "@/lib/app-router";
import { commands, type LogQuery, type OperationLog } from "@/lib/bindings";

vi.mock("@/lib/bindings", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/bindings")>();
  return {
    ...actual,
    commands: { listLogs: vi.fn(), listLogOperationTypes: vi.fn() },
  };
});
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ setTitle: vi.fn().mockResolvedValue(undefined) }),
}));
vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => ({ setZoom: vi.fn().mockResolvedValue(undefined) }),
}));
vi.mock("@tanstack/react-router-devtools", () => ({
  TanStackRouterDevtools: () => null,
}));

const listLogs = vi.mocked(commands.listLogs);
const listLogOperationTypes = vi.mocked(commands.listLogOperationTypes);

const MAIN_SELECTOR = '[data-scroll-restoration-id="main"]';
// 120 件固定母数。page/per_page の組合せごとに数学的に一貫した slice を返す
// (setPerPage → update() の 2 段階更新の間に生じる過渡的な page/per_page 組合せの
// query も含め、呼出し回数に依存せず handle する)。
const TOTAL_COUNT = 120;

function log(id: number): OperationLog {
  return {
    id,
    operation_type: "backup_create",
    summary: `合成ログ ${String(id)}`,
    detail_json: null,
    created_at: "2026-07-10T12:34:56",
  };
}

function respondFor(query: LogQuery) {
  const page = query.page;
  const perPage = query.per_page;
  const startId = (page - 1) * perPage + 1;
  const count = Math.max(0, Math.min(perPage, TOTAL_COUNT - (page - 1) * perPage));
  return {
    status: "ok" as const,
    data: {
      items: Array.from({ length: count }, (_, i) => log(startId + i)),
      total_count: TOTAL_COUNT,
      page,
      per_page: perPage,
    },
  };
}

describe("AC16 / SC10a: OperationLogsPage perPage 変更後の scroll 位置（Gated Amendment 3、UI-11c）", () => {
  beforeEach(() => {
    listLogs.mockReset();
    listLogOperationTypes.mockReset();
    listLogOperationTypes.mockResolvedValue({ status: "ok", data: [] });
    listLogs.mockImplementation((query) => Promise.resolve(respondFor(query)));
  });

  afterEach(() => {
    // 先に React の tree を外す（RootLayout が body の直下へ portal で描く日報の印刷の案内を
    // React の外から消すと、後の unmount が removeChild で落ちる）
    cleanup();
    document.body.replaceChildren();
  });

  it("SC10a: page 1(640)→次のページ→page 2(900)→表示件数100 で main.scrollTop が 0 になる", async () => {
    window.history.pushState(null, "", "/settings/logs?page=1");
    const router = createAppRouter({
      history: createMemoryHistory({ initialEntries: ["/settings/logs?page=1"] }),
    });
    // scrollPageToTop() の flag は window.location.pathname に束縛される（Final Review round 3
    // P2）。実 browser history と異なり createMemoryHistory は window.location を更新しないため、
    // pathname 判定が実挙動を再現できるよう onBeforeLoad（render commit より前）で pushState を
    // 明示的に同期する（router 挙動には無関係、test 側の副作用のみ）。
    router.subscribe("onBeforeLoad", () => {
      window.history.pushState(null, "", router.latestLocation.href);
    });
    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    render(
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    );
    await waitFor(() => {
      expect(router.stores.resolvedLocation.get()?.href).toBe("/settings/logs?page=1");
    });

    const main = document.querySelector<HTMLElement>(MAIN_SELECTOR);
    if (main === null) throw new Error("RootLayout main scroll container is required");
    // happy-dom の smooth scroll 模擬は実 browser の割込み挙動を再現しないため、非同期の
    // smooth 挙動だけを避け top/left は同期的に反映する（round 2 是正: forced-top 経路が
    // main.scrollTop = 0 から main.scrollTo({ top: 0, left: 0 }) へ変わったため、完全な
    // no-op のままだと flag 適用そのものを検出できない）。
    vi.spyOn(main, "scrollTo").mockImplementation((options) => {
      const opts = options as ScrollToOptions;
      if (typeof opts.top === "number") main.scrollTop = opts.top;
      if (typeof opts.left === "number") main.scrollLeft = opts.left;
    });

    await screen.findByText("合成ログ 1");

    const user = userEvent.setup();

    main.scrollTop = 640;
    main.dispatchEvent(new Event("scroll", { bubbles: true }));

    await user.click(screen.getByRole("button", { name: "次のページ" }));
    await screen.findByText("合成ログ 51");

    main.scrollTop = 900;
    main.dispatchEvent(new Event("scroll", { bubbles: true }));

    await user.click(screen.getByRole("combobox", { name: "表示件数" }));
    await user.click(screen.getByRole("option", { name: "100 件" }));

    await screen.findByText("合成ログ 1");

    expect(main.scrollTop).toBe(0);
  });
});
