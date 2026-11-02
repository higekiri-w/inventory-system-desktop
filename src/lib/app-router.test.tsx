import { createMemoryHistory, RouterProvider, type ParsedLocation } from "@tanstack/react-router";
import { getElementScrollRestorationEntry, scrollRestorationCache } from "@tanstack/router-core";
import { act, cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  applyMainNavScroll,
  createAppRouter,
  getAppScrollRestorationKey,
  pruneScrollRestorationEntries,
} from "./app-router";
import { consumeMainNavScroll, markMainNavScroll } from "./main-nav-scroll";
import { scrollPageToTop } from "./page-scroll";

const MAIN_SELECTOR = '[data-scroll-restoration-id="main"]';
const PRODUCTS_LIST_SELECTOR = '[data-scroll-restoration-id="products-list"]';

type AppRouter = ReturnType<typeof createAppRouter>;

interface DeferredRouteRender {
  promise: Promise<void>;
  resolve: () => void;
}

const inventoryRecordsRouteStub = vi.hoisted(() => ({
  deferredRender: undefined as DeferredRouteRender | undefined,
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ setTitle: vi.fn().mockResolvedValue(undefined) }),
}));

vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => ({ setZoom: vi.fn().mockResolvedValue(undefined) }),
}));

vi.mock("@/features/backup-restore/useAutoBackupCheck", () => ({
  useAutoBackupCheck: () => undefined,
  suspendAutoBackupCheck: vi.fn(),
  resumeAutoBackupCheck: vi.fn(),
}));

vi.mock("@tanstack/react-router-devtools", () => ({
  TanStackRouterDevtools: () => null,
}));

vi.mock("@/features/inventory-records/InventoryRecordsPage", async () => {
  const { useEffect, useState } = await import("react");

  return {
    InventoryRecordsPage: () => {
      const deferredRender = inventoryRecordsRouteStub.deferredRender;
      const [isReady, setIsReady] = useState(deferredRender === undefined);

      useEffect(() => {
        if (deferredRender === undefined) return;

        let isMounted = true;
        void deferredRender.promise.then(() => {
          if (isMounted) setIsReady(true);
        });
        return () => {
          isMounted = false;
        };
      }, [deferredRender]);

      return (
        <div>
          {isReady ? (
            <div data-testid="inventory-records-content">inventory records route</div>
          ) : (
            <span data-testid="inventory-records-loading">loading inventory records</span>
          )}
        </div>
      );
    },
  };
});

vi.mock("@/features/inventory-records/ReceivingRecordDetailPage", () => ({
  ReceivingRecordDetailPage: () => <div>receiving record detail route</div>,
}));

vi.mock("@/features/stock-inquiry/StockInquiryPage", () => ({
  StockInquiryPage: () => <div>stock inquiry route</div>,
}));

function locationWith(href: string, historyKey: string): ParsedLocation {
  return {
    href,
    pathname: href.split("?")[0] ?? href,
    search: {},
    searchStr: href.includes("?") ? `?${href.split("?")[1]}` : "",
    state: { __TSR_key: historyKey },
    hash: "",
    maskedLocation: undefined,
    unmaskOnReload: undefined,
    publicHref: href,
  } as ParsedLocation;
}

async function navigateAndRender(router: AppRouter, to: string) {
  await act(async () => {
    await router.navigate({ href: to });
  });
  await waitFor(() => {
    expect(router.stores.resolvedLocation.get()?.href).toBe(to);
  });
}

function installMainScroller(initialTop = 0) {
  const main = document.createElement("main");
  main.setAttribute("data-scroll-restoration-id", "main");
  main.scrollTop = initialTop;
  document.body.append(main);
  return main;
}

function trackScroll(main: HTMLElement, top: number) {
  main.scrollTop = top;
  main.dispatchEvent(new Event("scroll", { bubbles: true }));
}

function delayNextInventoryRecordsRender(): DeferredRouteRender {
  let resolve: () => void = () => undefined;
  const promise = new Promise<void>((resolvePromise) => {
    resolve = resolvePromise;
  });
  const deferredRender = { promise, resolve };
  inventoryRecordsRouteStub.deferredRender = deferredRender;
  return deferredRender;
}

function installNativeScrollClamp(main: HTMLElement) {
  let actualTop = main.scrollTop;
  let actualLeft = main.scrollLeft;
  let forcedScrollHeight: number | undefined;
  let forcedScrollWidth: number | undefined;
  const requestedTops: number[] = [];
  const requestedLefts: number[] = [];

  Object.defineProperty(main, "clientHeight", {
    configurable: true,
    get: () => 500,
  });
  Object.defineProperty(main, "clientWidth", {
    configurable: true,
    get: () => 800,
  });
  Object.defineProperty(main, "scrollHeight", {
    configurable: true,
    get: () =>
      forcedScrollHeight ??
      (main.querySelector('[data-testid="inventory-records-content"]') === null ? 500 : 1_000),
  });
  // round 3 是正（Opus P2）: scrollWidth を固定値にすると横判定
  // （scrollWidth >= savedScrollLeft + clientWidth）が初回から常に満たされ、scrollLeft
  // 復元行を削除しても pass する mutant が生存する。scrollHeight と同型の条件付き getter
  // にし、content marker 不在時は clientWidth（伸びる要素がまだ無い）にする。
  Object.defineProperty(main, "scrollWidth", {
    configurable: true,
    get: () =>
      forcedScrollWidth ??
      (main.querySelector('[data-testid="inventory-records-content"]') === null
        ? main.clientWidth
        : 1_600),
  });
  Object.defineProperty(main, "scrollTop", {
    configurable: true,
    get: () => actualTop,
    set: (requestedTop: number) => {
      requestedTops.push(requestedTop);
      const maximumTop = Math.max(0, main.scrollHeight - main.clientHeight);
      actualTop = Math.min(Math.max(0, requestedTop), maximumTop);
    },
  });
  Object.defineProperty(main, "scrollLeft", {
    configurable: true,
    get: () => actualLeft,
    set: (requestedLeft: number) => {
      requestedLefts.push(requestedLeft);
      const maximumLeft = Math.max(0, main.scrollWidth - main.clientWidth);
      actualLeft = Math.min(Math.max(0, requestedLeft), maximumLeft);
    },
  });

  return {
    requestedTops,
    requestedLefts,
    setScrollHeight: (scrollHeight: number) => {
      forcedScrollHeight = scrollHeight;
    },
    setScrollWidth: (scrollWidth: number) => {
      forcedScrollWidth = scrollWidth;
    },
  };
}

async function renderAppRouterAt(initialHref: string) {
  window.history.pushState(null, "", initialHref);
  const router = createAppRouter({
    history: createMemoryHistory({ initialEntries: [initialHref] }),
  });
  // scrollPageToTop() の flag は window.location.pathname に束縛される（Final Review round 3
  // P2）。実 browser history と異なり createMemoryHistory は window.location を更新しないため、
  // pathname 判定 test が実挙動を再現できるよう onBeforeLoad（render commit より前）で
  // pushState を明示的に同期する（router 挙動には無関係、test 側の副作用のみ）。
  router.subscribe("onBeforeLoad", () => {
    window.history.pushState(null, "", router.latestLocation.href);
  });
  render(<RouterProvider router={router} />);
  await waitFor(() => {
    expect(router.stores.resolvedLocation.get()?.href).toBe(initialHref);
  });
  const main = document.querySelector<HTMLElement>(MAIN_SELECTOR);
  if (main === null) throw new Error("RootLayout main scroll container is required");
  return { router, main };
}

describe("UI-12 / DSR-17 app router configuration", () => {
  beforeEach(() => {
    consumeMainNavScroll();
    sessionStorage.clear();
    inventoryRecordsRouteStub.deferredRender = undefined;
  });

  afterEach(() => {
    // 先に React の tree を外す（RootLayout が body の直下へ portal で描く要素〈UI-09a-D18 の印刷の案内〉を
    // React の外から消すと、後の unmount が removeChild で落ちる）
    cleanup();
    document.body.replaceChildren();
  });

  it("T1: uses the complete href instead of the history-entry key", () => {
    expect(getAppScrollRestorationKey(locationWith("/stock", "entry-a"))).toBe("/stock");
    expect(getAppScrollRestorationKey(locationWith("/stock?status=low_stock", "entry-b"))).toBe(
      "/stock?status=low_stock",
    );
  });

  it("T9: wires restoration, the main selector, and the href key into the router", () => {
    const router = createAppRouter({
      history: createMemoryHistory({ initialEntries: ["/stock?status=low_stock"] }),
    });

    expect(router.options.scrollRestoration).toBe(true);
    expect(router.options.scrollToTopSelectors).toEqual([MAIN_SELECTOR, PRODUCTS_LIST_SELECTOR]);
    expect(
      router.options.getScrollRestorationKey?.(
        locationWith("/inventory/records?page=3", "unrelated-entry-key"),
      ),
    ).toBe("/inventory/records?page=3");
  });

  it("T4: consumes a matching flag and instantly scrolls main to the top", () => {
    const main = installMainScroller(240);
    const scrollTo = vi.spyOn(main, "scrollTo");
    markMainNavScroll("/products?q=thread");

    applyMainNavScroll("/products?q=thread");

    expect(scrollTo).toHaveBeenCalledOnce();
    expect(scrollTo).toHaveBeenCalledWith({ top: 0, left: 0 });
    expect(consumeMainNavScroll()).toBeUndefined();
  });

  it("T4: consumes a mismatched flag without scrolling", () => {
    const main = installMainScroller(240);
    const scrollTo = vi.spyOn(main, "scrollTo");
    markMainNavScroll("/products?q=thread");

    applyMainNavScroll("/inventory/records?page=3");

    expect(scrollTo).not.toHaveBeenCalled();
    expect(consumeMainNavScroll()).toBeUndefined();
  });

  it("T4: leaves scroll untouched when no flag is present", () => {
    const main = installMainScroller(240);
    const scrollTo = vi.spyOn(main, "scrollTo");

    applyMainNavScroll("/products?q=thread");

    expect(scrollTo).not.toHaveBeenCalled();
  });

  it("T10: restores a saved main position on a returnTo push and resets an uncached href", async () => {
    const sourceHref = "/inventory/records?recordType=receiving_record&page=2";
    const detailHref =
      "/inventory/receiving/records/12?returnTo=%2Finventory%2Frecords%3FrecordType%3Dreceiving_record%26page%3D2";
    const { router, main } = await renderAppRouterAt(sourceHref);
    installNativeScrollClamp(main);
    expect(router.isScrollRestoring).toBe(true);
    expect(router.isScrollRestorationSetup).toBe(true);

    trackScroll(main, 360);
    await navigateAndRender(router, detailHref);
    expect(main.scrollTop).toBe(0);

    window.dispatchEvent(new Event("pagehide"));
    expect(JSON.parse(sessionStorage.getItem("tsr-scroll-restoration-v1_3") ?? "{}")).toEqual(
      expect.objectContaining({
        [sourceHref]: {
          [MAIN_SELECTOR]: { scrollX: 0, scrollY: 360 },
        },
      }),
    );

    const deferredRender = delayNextInventoryRecordsRender();
    await navigateAndRender(router, sourceHref);
    expect(main.scrollTop).toBe(0);

    await act(async () => {
      deferredRender.resolve();
      await deferredRender.promise;
    });
    await waitFor(() => {
      expect(main.querySelector('[data-testid="inventory-records-content"]')).not.toBeNull();
      expect(main.scrollTop).toBe(360);
    });

    trackScroll(main, 225);
    await navigateAndRender(router, "/products/new");
    expect(main.scrollTop).toBe(0);

    window.dispatchEvent(new Event("pagehide"));
    expect(sessionStorage.getItem("tsr-scroll-restoration-v1_3")).toContain(sourceHref);
  });

  it("T5: lets a main-navigation flag override a proven cache hit", async () => {
    const sourceHref = "/inventory/records?page=4";
    const detailHref = "/inventory/receiving/records/12";
    const { router, main } = await renderAppRouterAt(sourceHref);

    trackScroll(main, 410);
    await navigateAndRender(router, detailHref);
    await navigateAndRender(router, sourceHref);
    expect(main.scrollTop).toBe(410);

    trackScroll(main, 515);
    await navigateAndRender(router, detailHref);
    markMainNavScroll(sourceHref);
    await navigateAndRender(router, sourceHref);

    expect(main.scrollTop).toBe(0);
    expect(main.scrollTop).not.toBe(515);
  });

  it("T11: consumes a stale same-href flag on the next href change without breaking return restoration", async () => {
    const sourceHref = "/inventory/records?page=5";
    const detailHref = "/inventory/receiving/records/12";
    const { router, main } = await renderAppRouterAt(sourceHref);

    trackScroll(main, 310);
    await navigateAndRender(router, detailHref);
    trackScroll(main, 190);
    await navigateAndRender(router, sourceHref);
    expect(main.scrollTop).toBe(310);

    markMainNavScroll(sourceHref);
    trackScroll(main, 470);
    await navigateAndRender(router, detailHref);
    expect(main.scrollTop).toBe(190);
    expect(consumeMainNavScroll()).toBeUndefined();

    await navigateAndRender(router, sourceHref);
    expect(main.scrollTop).toBe(470);
  });

  it("SC10b: a valid same-pathname force-scroll-top flag overrides a proven cache hit (Gated Amendment 3, A3-a)", async () => {
    // DSR-03 型（navigate を伴わない scrollPageToTop 呼出し、perPage reset の実態）を模した
    // 同一 pathname・search のみ差し替えのシナリオ（"/inventory/records" 固定、page だけが変わる）。
    const listHrefA = "/inventory/records?page=20";
    const listHrefB = "/inventory/records?page=200";
    const { router, main } = await renderAppRouterAt(listHrefA);
    // happy-dom の smooth scroll 模擬は setTimeout(0) 経由の遅延書込みで、後続 await 中に
    // main.scrollTop を非同期に上書きし得る（実 browser の割込みと異なる挙動）。scrollTo の
    // 非同期挙動だけを避け、top/left は同期的に反映する（round 2 是正: forced-top 経路が
    // main.scrollTop = 0 から main.scrollTo({ top: 0, left: 0 }) へ変わったため、完全な
    // no-op のままだと flag 適用そのものを検出できない）。
    vi.spyOn(main, "scrollTo").mockImplementation((options) => {
      const opts = options as ScrollToOptions;
      if (typeof opts.top === "number") main.scrollTop = opts.top;
      if (typeof opts.left === "number") main.scrollLeft = opts.left;
    });

    trackScroll(main, 410);
    await navigateAndRender(router, listHrefB);
    await navigateAndRender(router, listHrefA);
    expect(main.scrollTop).toBe(410);

    trackScroll(main, 515);
    await navigateAndRender(router, listHrefB);
    // listHrefB（listHrefA と同じ pathname）で flag を立てる
    scrollPageToTop();
    await navigateAndRender(router, listHrefA);

    expect(main.scrollTop).toBe(0);
    expect(main.scrollTop).not.toBe(410);
  });

  it("SC10b: an expired same-pathname force-scroll-top flag lets the normal return restoration proceed (Gated Amendment 3, A3-a)", async () => {
    const listHrefA = "/inventory/records?page=21";
    const listHrefB = "/inventory/records?page=210";
    const { router, main } = await renderAppRouterAt(listHrefA);
    // happy-dom の smooth scroll 模擬は setTimeout(0) 経由の遅延書込みで、後続 await 中に
    // main.scrollTop を非同期に上書きし得る（実 browser の割込みと異なる挙動）。scrollPageToTop()
    // が呼ぶ scrollTo は no-op にし、flag の期限判定だけを見る。
    vi.spyOn(main, "scrollTo").mockImplementation(() => undefined);

    trackScroll(main, 410);
    await navigateAndRender(router, listHrefB);
    await navigateAndRender(router, listHrefA);
    expect(main.scrollTop).toBe(410);

    trackScroll(main, 515);
    await navigateAndRender(router, listHrefB);

    // flag の期限（1000ms）を実経過時間で切らす。TanStack Router 自身も内部で Date.now() に
    // 依存するため（pending 状態のタイミング計算等）、Date.now を直接 mock せず実待機する。
    scrollPageToTop();
    await new Promise((resolve) => setTimeout(resolve, 1_050));
    await navigateAndRender(router, listHrefA);

    expect(main.scrollTop).toBe(515);
    expect(main.scrollTop).not.toBe(0);
  });

  it("SC10b: a force-scroll-top flag bound to a different pathname does not override another screen's return restoration (Final Review round 3, P2)", async () => {
    const listHref = "/inventory/records?page=22";
    const detailHref = "/inventory/receiving/records/12?case=force-scroll-top-cross-pathname";
    const { router, main } = await renderAppRouterAt(listHref);
    // happy-dom の smooth scroll 模擬は setTimeout(0) 経由の遅延書込みで、後続 await 中に
    // main.scrollTop を非同期に上書きし得る（実 browser の割込みと異なる挙動）。scrollPageToTop()
    // が呼ぶ scrollTo は no-op にし、pathname 束縛の判定だけを見る。
    vi.spyOn(main, "scrollTo").mockImplementation(() => undefined);

    trackScroll(main, 410);
    await navigateAndRender(router, detailHref);
    await navigateAndRender(router, listHref);
    expect(main.scrollTop).toBe(410);

    trackScroll(main, 515);
    await navigateAndRender(router, detailHref);
    // pathname A（detailHref）で flag を立てる。navigate を伴わない DSR-03 型呼出しの近似
    scrollPageToTop();
    // TTL 内に pathname B（listHref、別画面）へ遷移 → pathname 不一致のため flag は無視される
    await navigateAndRender(router, listHref);

    // listHref 自身の正規の戻り復元（proven cache hit = 515）が保たれる
    expect(main.scrollTop).toBe(515);
    expect(main.scrollTop).not.toBe(0);
  });

  it("T12: reapplies a clamped saved position once after delayed content expands", async () => {
    const sourceHref = "/inventory/records?recordType=receiving_record&page=12";
    const detailHref = "/inventory/receiving/records/12?case=delayed-reapply";
    const { router, main } = await renderAppRouterAt(sourceHref);
    const clamp = installNativeScrollClamp(main);

    trackScroll(main, 360);
    await navigateAndRender(router, detailHref);
    const deferredRender = delayNextInventoryRecordsRender();
    clamp.requestedTops.length = 0;

    await navigateAndRender(router, sourceHref);
    expect(main.scrollTop).toBe(0);
    expect(
      getElementScrollRestorationEntry(router, {
        id: "main",
        getKey: (location) => location.href,
      }),
    ).toEqual({ scrollX: 0, scrollY: 360 });
    expect(clamp.requestedTops.filter((top) => top === 360)).toHaveLength(1);

    await act(async () => {
      deferredRender.resolve();
      await deferredRender.promise;
    });
    await waitFor(() => {
      expect(main.scrollTop).toBe(360);
    });
    expect(clamp.requestedTops.filter((top) => top === 360)).toHaveLength(2);

    await act(async () => {
      main
        .querySelector('[data-testid="inventory-records-content"]')
        ?.append(document.createElement("div"));
      await Promise.resolve();
    });
    expect(clamp.requestedTops.filter((top) => top === 360)).toHaveLength(2);
  });

  it.each(["wheel", "pointerdown", "keydown"])(
    "T12: cancels a delayed reapply after %s user input",
    async (eventType) => {
      const sourceHref = `/inventory/records?recordType=receiving_record&cancel=${eventType}`;
      const detailHref = `/inventory/receiving/records/12?cancel=${eventType}`;
      const { router, main } = await renderAppRouterAt(sourceHref);
      const clamp = installNativeScrollClamp(main);

      trackScroll(main, 340);
      await navigateAndRender(router, detailHref);
      const deferredRender = delayNextInventoryRecordsRender();
      clamp.requestedTops.length = 0;

      await navigateAndRender(router, sourceHref);
      expect(main.scrollTop).toBe(0);
      main.dispatchEvent(new Event(eventType, { bubbles: true }));

      await act(async () => {
        deferredRender.resolve();
        await deferredRender.promise;
      });
      await waitFor(() => {
        expect(main.querySelector('[data-testid="inventory-records-content"]')).not.toBeNull();
      });
      expect(main.scrollTop).toBe(0);
      expect(clamp.requestedTops.filter((top) => top === 340)).toHaveLength(1);
    },
  );

  it("T12: cancels a delayed reapply on the next navigation", async () => {
    const sourceHref = "/inventory/records?recordType=receiving_record&cancel=navigation";
    const detailHref = "/inventory/receiving/records/12?cancel=navigation";
    const { router, main } = await renderAppRouterAt(sourceHref);
    const clamp = installNativeScrollClamp(main);

    trackScroll(main, 320);
    await navigateAndRender(router, detailHref);
    delayNextInventoryRecordsRender();
    clamp.requestedTops.length = 0;

    await navigateAndRender(router, sourceHref);
    expect(main.scrollTop).toBe(0);
    await navigateAndRender(router, "/products/new?from=delayed-restoration-test");

    clamp.setScrollHeight(1_000);
    await act(async () => {
      main.append(document.createElement("div"));
      await Promise.resolve();
    });
    expect(clamp.requestedTops.filter((top) => top === 320)).toHaveLength(1);
    expect(main.scrollTop).toBe(0);
  });

  it("T13: does not arm delayed restoration after a matching main-navigation flag", async () => {
    const sourceHref = "/inventory/records?recordType=receiving_record&page=13";
    const detailHref = "/inventory/receiving/records/12?case=main-navigation";
    const { router, main } = await renderAppRouterAt(sourceHref);
    const clamp = installNativeScrollClamp(main);

    trackScroll(main, 390);
    await navigateAndRender(router, detailHref);
    const deferredRender = delayNextInventoryRecordsRender();
    clamp.requestedTops.length = 0;
    markMainNavScroll(sourceHref);

    await navigateAndRender(router, sourceHref);
    expect(main.scrollTop).toBe(0);

    await act(async () => {
      deferredRender.resolve();
      await deferredRender.promise;
    });
    await waitFor(() => {
      expect(main.querySelector('[data-testid="inventory-records-content"]')).not.toBeNull();
    });
    expect(main.scrollTop).toBe(0);
    expect(clamp.requestedTops.filter((top) => top === 390)).toHaveLength(1);
  });

  it("GA3b-7: applyMainNavScroll resets both main and the list scroll container when both are present (Lane 4 Gated Amendment 3, GA1c の拡張)", () => {
    const main = installMainScroller(240);
    const box = document.createElement("div");
    box.setAttribute("data-list-scroll-container", "");
    box.setAttribute("data-scroll-restoration-id", "products-list");
    main.append(box);
    const boxScrollTo = vi.spyOn(box, "scrollTo").mockImplementation(() => undefined);
    const mainScrollTo = vi.spyOn(main, "scrollTo").mockImplementation(() => undefined);
    markMainNavScroll("/products?q=thread");

    applyMainNavScroll("/products?q=thread");

    expect(boxScrollTo).toHaveBeenCalledWith({ top: 0, left: 0 });
    expect(mainScrollTo).toHaveBeenCalledWith({ top: 0, left: 0 });
  });

  it("GA3b-7: applyMainNavScroll falls back to main when no list scroll container exists", () => {
    const main = installMainScroller(240);
    const mainScrollTo = vi.spyOn(main, "scrollTo").mockImplementation(() => undefined);
    markMainNavScroll("/products?q=thread");

    applyMainNavScroll("/products?q=thread");

    expect(mainScrollTo).toHaveBeenCalledWith({ top: 0, left: 0 });
  });

  it("GA1c: pruneScrollRestorationEntries keeps both main and products-list selectors across repeated sweeps (round 1 P1 failure mode — allowlist must not collapse to main-only)", () => {
    // round 1 P1: allowlist が main 単独のままだと箱がまだ DOM に無い起動直後の sweep や
    // 通常の onBeforeLoad sweep のたびに products-list の cache entry が削られてしまう。
    // 静的 2-selector 配列（round 2 是正）であれば複数回の sweep を跨いでも両方残る。
    if (scrollRestorationCache === null) throw new Error("scroll restoration cache is required");
    const stockMain = { scrollX: 1, scrollY: 111 };
    const stockList = { scrollX: 2, scrollY: 222 };
    scrollRestorationCache.set(() => ({
      "/stock": {
        '[data-scroll-restoration-id="main"]': stockMain,
        '[data-scroll-restoration-id="products-list"]': stockList,
      },
    }));

    createAppRouter(); // startup sweep（box がまだ DOM に無い状態を模す）
    pruneScrollRestorationEntries("/stock"); // 2 回目の sweep（onBeforeLoad 相当）

    expect(scrollRestorationCache.state["/stock"]['[data-scroll-restoration-id="main"]']).toEqual(
      stockMain,
    );
    expect(
      scrollRestorationCache.state["/stock"]['[data-scroll-restoration-id="products-list"]'],
    ).toEqual(stockList);
  });

  it("SP1: startup sweep prunes non-main entries across all keys", () => {
    // DSR-17 (j) / D-G: startup removes every non-main selector from every key.
    if (scrollRestorationCache === null) throw new Error("scroll restoration cache is required");
    const stockMain = { scrollX: 1, scrollY: 101 };
    const itemsMain = { scrollX: 2, scrollY: 202 };
    const positionalSelector = "div:nth-child(2) > nav:nth-child(1)";
    scrollRestorationCache.set(() => ({
      "/stock": {
        '[data-scroll-restoration-id="main"]': stockMain,
        [positionalSelector]: { scrollX: 3, scrollY: 303 },
      },
      "/items": {
        '[data-scroll-restoration-id="main"]': itemsMain,
        [positionalSelector]: { scrollX: 4, scrollY: 404 },
      },
    }));

    createAppRouter();

    expect(scrollRestorationCache.state["/stock"]['[data-scroll-restoration-id="main"]']).toEqual(
      stockMain,
    );
    expect(scrollRestorationCache.state["/items"]['[data-scroll-restoration-id="main"]']).toEqual(
      itemsMain,
    );
    expect(scrollRestorationCache.state["/stock"][positionalSelector]).toBeUndefined();
    expect(scrollRestorationCache.state["/items"][positionalSelector]).toBeUndefined();
  });

  it("SP2: onBeforeLoad prunes only the fromLocation key", async () => {
    // DSR-17 (j) / D-G: the first load has no fromLocation; later pruning is source-key only.
    if (scrollRestorationCache === null) throw new Error("scroll restoration cache is required");
    const { router } = await renderAppRouterAt("/stock");
    expect(router.stores.resolvedLocation.get()?.href).toBe("/stock");

    const stockMain = { scrollX: 5, scrollY: 505 };
    const itemsMain = { scrollX: 6, scrollY: 606 };
    const positionalSelector = "div:nth-child(2) > nav:nth-child(1)";
    const itemsPositional = { scrollX: 7, scrollY: 707 };
    scrollRestorationCache.set(() => ({
      "/stock": {
        '[data-scroll-restoration-id="main"]': stockMain,
        [positionalSelector]: { scrollX: 8, scrollY: 808 },
      },
      "/items": {
        '[data-scroll-restoration-id="main"]': itemsMain,
        [positionalSelector]: itemsPositional,
      },
    }));

    const sidebarViewport = document.createElement("div");
    sidebarViewport.style.overflow = "auto";
    document.body.append(sidebarViewport);
    sidebarViewport.scrollTop = 77;
    sidebarViewport.dispatchEvent(new Event("scroll", { bubbles: true }));

    await act(async () => {
      await router.navigate({ to: "/items" as never });
    });

    expect(scrollRestorationCache.state["/stock"]['[data-scroll-restoration-id="main"]']).toEqual(
      stockMain,
    );
    expect(Object.keys(scrollRestorationCache.state["/stock"] ?? {})).toEqual([
      '[data-scroll-restoration-id="main"]',
    ]);
    expect(scrollRestorationCache.state["/items"]['[data-scroll-restoration-id="main"]']).toEqual(
      itemsMain,
    );
    expect(scrollRestorationCache.state["/items"][positionalSelector]).toEqual(itemsPositional);
  });

  it("SP3: main entry survives prune", () => {
    // DSR-17 (j) / D-G: the main selector and its coordinates are preserved verbatim.
    if (scrollRestorationCache === null) throw new Error("scroll restoration cache is required");
    const mainEntry = { scrollX: 9, scrollY: 909 };
    scrollRestorationCache.set(() => ({
      "/a": {
        '[data-scroll-restoration-id="main"]': mainEntry,
        "div:nth-child(3) > aside:nth-child(1)": { scrollX: 10, scrollY: 1_010 },
      },
    }));

    pruneScrollRestorationEntries("/a");

    expect(scrollRestorationCache.state["/a"]['[data-scroll-restoration-id="main"]']).toEqual(
      mainEntry,
    );
  });
});
