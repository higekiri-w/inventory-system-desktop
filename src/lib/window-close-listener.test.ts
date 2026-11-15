// UI-12 / UI-USW-D5 の 2・3・5: native の window の close の listener の登録と、登録の失敗の扱い。
// page の読み直しは module の読み直し（vi.resetModules()）で表す（sessionStorage は読み直しでは残る）。
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

type CloseHandler = (event: { preventDefault: () => void }) => void | Promise<void>;

function fakeWindow() {
  const handlers: CloseHandler[] = [];
  const unlisten = vi.fn();
  const win = {
    onCloseRequested: vi.fn((handler: CloseHandler) => {
      handlers.push(handler);
      return Promise.resolve(unlisten);
    }),
    destroy: vi.fn(() => Promise.resolve()),
  };
  return { win, handlers };
}

/** 新しい page の module を読む（module の flag は初期値、sessionStorage はそのまま） */
async function loadPage() {
  vi.resetModules();
  return await import("./window-close-listener");
}

/** 登録の Promise とその後の destroy の Promise を流す */
async function settle() {
  await new Promise((resolve) => setTimeout(resolve, 0));
}

let warn: ReturnType<typeof vi.spyOn>;

beforeEach(() => {
  window.sessionStorage.clear();
  warn = vi.spyOn(console, "warn").mockImplementation(() => undefined);
});

afterEach(() => {
  vi.doUnmock("@/lib/window-close-guard");
  vi.unstubAllGlobals();
});

describe("installWindowCloseGuard (UI-12/UI-USW-D5)", () => {
  it("T1 UI-USW-D5 の 2: 2 回呼んでも onCloseRequested の登録は 1 回", async () => {
    const { installWindowCloseGuard } = await loadPage();
    const { win } = fakeWindow();

    installWindowCloseGuard(() => win);
    installWindowCloseGuard(() => win);
    await settle();

    expect(win.onCloseRequested).toHaveBeenCalledTimes(1);
    expect(win.destroy).not.toHaveBeenCalled();
    expect(warn).not.toHaveBeenCalled();
  });

  it("UI-USW-D5 の 3: handler は判定が止めると返したときだけ preventDefault する", async () => {
    const requestWindowClose = vi.fn(() => false);
    vi.doMock("@/lib/window-close-guard", () => ({ requestWindowClose }));
    const { installWindowCloseGuard } = await loadPage();
    const { win, handlers } = fakeWindow();
    installWindowCloseGuard(() => win);
    await settle();

    const passed = { preventDefault: vi.fn() };
    await handlers[0](passed);
    expect(passed.preventDefault).not.toHaveBeenCalled();

    requestWindowClose.mockReturnValue(true);
    const stopped = { preventDefault: vi.fn() };
    await handlers[0](stopped);
    expect(stopped.preventDefault).toHaveBeenCalledTimes(1);
  });

  it("T9 UI-USW-D5 の 3 (5): 判定が例外を投げても handler は投げず、preventDefault しない（F2）", async () => {
    vi.doMock("@/lib/window-close-guard", () => ({
      requestWindowClose: () => {
        throw new Error("合成の判定の失敗");
      },
    }));
    const { installWindowCloseGuard } = await loadPage();
    const { win, handlers } = fakeWindow();
    installWindowCloseGuard(() => win);
    await settle();

    const event = { preventDefault: vi.fn() };
    // 投げると onCloseRequested の包みが destroy に届かず、閉じられない窓になる
    await expect(Promise.resolve().then(() => handlers[0](event))).resolves.toBeUndefined();
    expect(event.preventDefault).not.toHaveBeenCalled();
    expect(warn).toHaveBeenCalledTimes(1);
  });

  it("T11 (a) 最初の page で登録が reject: 投げず warn を出し、destroy せず、印を消す（F1）", async () => {
    const { installWindowCloseGuard } = await loadPage();
    const { win } = fakeWindow();
    win.onCloseRequested.mockImplementationOnce(() =>
      Promise.reject(new Error("合成の登録の失敗")),
    );

    expect(() => {
      installWindowCloseGuard(() => win);
    }).not.toThrow();
    await settle();

    expect(warn).toHaveBeenCalledTimes(1);
    expect(win.destroy).not.toHaveBeenCalled();
    expect(window.sessionStorage.length).toBe(0);
  });

  it("T11 (b) 最初の page で窓の取得が同期で投げる: 投げず warn を出し、destroy せず、印を消す（F1）", async () => {
    const { installWindowCloseGuard } = await loadPage();
    const getWindow = vi.fn(() => {
      throw new TypeError("合成: Tauri の外");
    });

    expect(() => {
      installWindowCloseGuard(getWindow);
    }).not.toThrow();
    await settle();

    expect(getWindow).toHaveBeenCalledTimes(1);
    expect(warn).toHaveBeenCalledTimes(1);
    expect(window.sessionStorage.length).toBe(0);
  });

  it("T11 (c) 読み直しの後の登録が reject: destroy で自分から閉じる（F7）", async () => {
    const first = fakeWindow();
    (await loadPage()).installWindowCloseGuard(() => first.win);
    await settle();
    expect(window.sessionStorage.length).toBe(1);

    const { installWindowCloseGuard } = await loadPage();
    const second = fakeWindow();
    second.win.onCloseRequested.mockImplementationOnce(() =>
      Promise.reject(new Error("合成の登録の失敗")),
    );
    installWindowCloseGuard(() => second.win);
    await settle();

    expect(second.win.destroy).toHaveBeenCalledTimes(1);
    expect(first.win.destroy).not.toHaveBeenCalled();
    expect(warn).toHaveBeenCalledTimes(1);
  });

  it("T11 (c) 前の page が登録の結果の前に読み直された後の、窓の取得の同期の失敗でも destroy を試す", async () => {
    const first = fakeWindow();
    first.win.onCloseRequested.mockReturnValueOnce(new Promise(() => undefined));
    (await loadPage()).installWindowCloseGuard(() => first.win);
    await settle();

    const { installWindowCloseGuard } = await loadPage();
    const second = fakeWindow();
    const getWindow = vi
      .fn(() => second.win)
      .mockImplementationOnce(() => {
        throw new TypeError("合成: 窓を取れない");
      });
    installWindowCloseGuard(getWindow);
    await settle();

    expect(second.win.onCloseRequested).not.toHaveBeenCalled();
    expect(second.win.destroy).toHaveBeenCalledTimes(1);
  });

  it("T11 (d) 読み直しの後の登録が reject し destroy も reject: 投げず warn だけ", async () => {
    (await loadPage()).installWindowCloseGuard(() => fakeWindow().win);
    await settle();

    const { installWindowCloseGuard } = await loadPage();
    const { win } = fakeWindow();
    win.onCloseRequested.mockImplementationOnce(() =>
      Promise.reject(new Error("合成の登録の失敗")),
    );
    win.destroy.mockImplementationOnce(() => Promise.reject(new Error("合成の destroy の失敗")));
    const unhandled = vi.fn();
    process.on("unhandledRejection", unhandled);
    try {
      expect(() => {
        installWindowCloseGuard(() => win);
      }).not.toThrow();
      await settle();
    } finally {
      process.off("unhandledRejection", unhandled);
    }

    expect(unhandled).not.toHaveBeenCalled();
    expect(win.destroy).toHaveBeenCalledTimes(1);
    expect(warn).toHaveBeenCalledTimes(2);
  });

  it("T11 (e) 読み直しの後の登録が成功: destroy しない", async () => {
    (await loadPage()).installWindowCloseGuard(() => fakeWindow().win);
    await settle();

    const { installWindowCloseGuard } = await loadPage();
    const { win } = fakeWindow();
    installWindowCloseGuard(() => win);
    await settle();

    expect(win.onCloseRequested).toHaveBeenCalledTimes(1);
    expect(win.destroy).not.toHaveBeenCalled();
    expect(warn).not.toHaveBeenCalled();
    expect(window.sessionStorage.length).toBe(1);
  });

  it("T11 (f) 印の読みが投げる + 登録が reject: 印が無いものとして扱い destroy しない", async () => {
    const { installWindowCloseGuard } = await loadPage();
    const { win } = fakeWindow();
    win.onCloseRequested.mockImplementationOnce(() =>
      Promise.reject(new Error("合成の登録の失敗")),
    );
    // 前の page の印が有っても、読めなければ最初の page として扱う
    vi.stubGlobal("sessionStorage", {
      getItem: () => {
        throw new Error("合成: sessionStorage を読めない");
      },
      setItem: () => undefined,
      removeItem: () => undefined,
    });
    expect(() => {
      installWindowCloseGuard(() => win);
    }).not.toThrow();
    await settle();

    expect(win.onCloseRequested).toHaveBeenCalledTimes(1);
    expect(win.destroy).not.toHaveBeenCalled();
  });

  it.each([
    { mark: "有る", stored: "1", destroyCalls: 1 },
    { mark: "無い", stored: null, destroyCalls: 0 },
  ])(
    "T11 (g) 印は読めた（$mark）が書込み・消去が投げる + 登録が reject: 投げずに登録を試し、destroy は $destroyCalls 回",
    async ({ stored, destroyCalls }) => {
      const { installWindowCloseGuard } = await loadPage();
      const { win } = fakeWindow();
      win.onCloseRequested.mockImplementationOnce(() =>
        Promise.reject(new Error("合成の登録の失敗")),
      );
      const setItem = vi.fn(() => {
        throw new Error("合成: sessionStorage に書けない");
      });
      vi.stubGlobal("sessionStorage", {
        getItem: () => stored,
        setItem,
        removeItem: () => {
          throw new Error("合成: sessionStorage から消せない");
        },
      });
      expect(() => {
        installWindowCloseGuard(() => win);
      }).not.toThrow();
      await settle();

      expect(setItem).toHaveBeenCalledTimes(1);
      expect(win.onCloseRequested).toHaveBeenCalledTimes(1);
      expect(win.destroy).toHaveBeenCalledTimes(destroyCalls);
    },
  );
});
