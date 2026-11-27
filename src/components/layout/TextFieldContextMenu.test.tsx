// UI-12 / UI-CTX-D1〜D4（UI_TECH_STACK §6.13）: 文字の欄の右クリックの自前のメニュー。
// happy-dom 21 に document.execCommand は無いので vi.fn を代入し、呼ばれた時の activeElement を控える。
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { toast } from "sonner";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";

import { TextFieldContextMenu } from "./TextFieldContextMenu";

vi.mock("sonner", () => ({ toast: { error: vi.fn() } }));
// 貼り付けは clipboard の plugin で読む（D-177）。呼ばれた時の activeElement を控え、中身は readText で差す
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  readText: vi.fn(() => {
    readCalls.push(document.activeElement);
    return readText();
  }),
}));

interface Call {
  args: unknown[];
  active: Element | null;
}

let execCalls: Call[];
let readCalls: (Element | null)[];
let readText: () => Promise<string>;
let browserReadText: ReturnType<typeof vi.spyOn>;

beforeEach(() => {
  execCalls = [];
  readCalls = [];
  readText = () => Promise.resolve("");
  vi.mocked(toast.error).mockClear();
  Object.defineProperty(document, "execCommand", {
    configurable: true,
    value: vi.fn((...args: unknown[]) => {
      execCalls.push({ args, active: document.activeElement });
      return true;
    }),
  });
});

afterEach(() => {
  delete (document as { execCommand?: unknown }).execCommand;
  vi.restoreAllMocks();
  vi.useRealTimers();
  document.head.querySelectorAll("style[data-test-animation]").forEach((style) => {
    style.remove();
  });
});

// 閉じる動き（tw-animate-css の exit）の模型: animationend が来るまで Content が残る
function withCloseAnimation() {
  const style = document.createElement("style");
  style.setAttribute("data-test-animation", "");
  style.textContent =
    '[role="menu"][data-state="open"]{animation-name:enter}[role="menu"][data-state="closed"]{animation-name:exit}';
  document.head.appendChild(style);
}

function endCloseAnimation() {
  const menu = document.querySelector('[role="menu"]');
  if (!menu) return;
  const event = new Event("animationend", { bubbles: true });
  Object.defineProperty(event, "animationName", { value: "exit" });
  act(() => {
    menu.dispatchEvent(event);
  });
}

// 開いている間 body は pointer-events: none になるので、user-event の確かめを外す。
// setup は navigator.clipboard を自分の stub に差し替えるので、その後に spy を差す（WebView2 の許可の窓を通る読みを使わないことを見る）
function user() {
  const setup = userEvent.setup({ pointerEventsCheck: 0 });
  browserReadText = vi.spyOn(navigator.clipboard, "readText");
  return setup;
}

function renderWith(ui: React.ReactNode) {
  return render(<TextFieldContextMenu>{ui}</TextFieldContextMenu>);
}

function openOn(field: HTMLElement, start?: number, end?: number) {
  act(() => {
    field.focus();
    if (start !== undefined) (field as HTMLInputElement).setSelectionRange(start, end ?? start);
  });
  return fireEvent.contextMenu(field);
}

const itemNames = () => screen.queryAllByRole("menuitem").map((item) => item.textContent);
const item = (name: string) => screen.getByRole("menuitem", { name });
const commands = (name: string) => execCalls.filter((call) => call.args[0] === name);

async function waitClosed() {
  await waitFor(() => {
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
  });
}

describe("TextFieldContextMenu (UI-12 / UI-CTX-D1〜D4)", () => {
  it("UI-CTX-D1: 欄の外と開いたメニューの上の右クリックは既定を止めて何も出さない", () => {
    renderWith(
      <>
        <h1>見出し</h1>
        <button type="button">保存</button>
        <input aria-label="商品名" defaultValue="abc" />
      </>,
    );
    expect(fireEvent.contextMenu(screen.getByRole("heading"))).toBe(false);
    expect(fireEvent.contextMenu(screen.getByRole("button"))).toBe(false);
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();

    // React の木の外の要素も止める
    const outside = document.createElement("div");
    document.body.appendChild(outside);
    expect(fireEvent.contextMenu(outside)).toBe(false);
    outside.remove();

    expect(openOn(screen.getByRole("textbox"))).toBe(false);
    expect(fireEvent.contextMenu(screen.getByRole("menu"))).toBe(false);
    expect(screen.getAllByRole("menu")).toHaveLength(1);
  });

  it("UI-CTX-D1: 文字の欄の右クリックで 4 つの項目だけが出る", async () => {
    renderWith(
      <>
        <input aria-label="一つ目" defaultValue="abc" />
        <textarea aria-label="二つ目" defaultValue="defgh" />
      </>,
    );
    const first = screen.getByRole<HTMLInputElement>("textbox", { name: "一つ目" });
    const second = screen.getByRole<HTMLTextAreaElement>("textbox", { name: "二つ目" });
    openOn(first, 1);
    expect(itemNames()).toEqual(["切り取り", "コピー", "貼り付け", "すべて選択"]);

    // 閉じて別の欄で開き直すと、その欄が操作の対象になる
    await user().keyboard("{Escape}");
    await waitClosed();
    openOn(second, 2);
    expect(itemNames()).toEqual(["切り取り", "コピー", "貼り付け", "すべて選択"]);
    await user().click(item("すべて選択"));
    await waitFor(() => {
      expect([second.selectionStart, second.selectionEnd]).toEqual([0, 5]);
    });
    expect(document.activeElement).toBe(second);
    expect([first.selectionStart, first.selectionEnd]).toEqual([1, 1]);

    // focus を一つ目の欄に置いたまま二つ目で開いても、閉じた後は二つ目に戻り一つ目は選ばれない
    act(() => {
      first.focus();
      first.setSelectionRange(1, 1);
      second.setSelectionRange(2, 2);
    });
    fireEvent.contextMenu(second);
    await user().click(item("すべて選択"));
    await waitClosed();
    await waitFor(() => {
      expect(document.activeElement).toBe(second);
    });
    expect([second.selectionStart, second.selectionEnd]).toEqual([0, 5]);
    expect([first.selectionStart, first.selectionEnd]).toEqual([1, 1]);
  });

  it("UI-CTX-D3: 閉じる動きの間に別の欄で右クリックしても控えた欄は替わらない", async () => {
    withCloseAnimation();
    renderWith(
      <>
        <input aria-label="一つ目" defaultValue="abcdef" />
        <input aria-label="二つ目" defaultValue="uvwxyz" />
      </>,
    );
    const first = screen.getByRole<HTMLInputElement>("textbox", { name: "一つ目" });
    const second = screen.getByRole<HTMLInputElement>("textbox", { name: "二つ目" });
    openOn(first, 0, 2);
    await user().click(item("切り取り"));
    expect(screen.getByRole("menu", { hidden: true })).toHaveAttribute("data-state", "closed");

    // 閉じかけのメニューは開き直さず、既定のメニューも出さない
    act(() => {
      second.setSelectionRange(1, 3);
    });
    expect(fireEvent.contextMenu(second)).toBe(false);
    expect(screen.getByRole("menu", { hidden: true })).toHaveAttribute("data-state", "closed");
    await user().keyboard("{Escape}");
    endCloseAnimation();
    await waitClosed();
    await waitFor(() => {
      expect(commands("cut")).toHaveLength(1);
    });
    expect(commands("cut")[0].active).toBe(first);
    expect(execCalls).toHaveLength(1);
  });

  it("UI-CTX-D3: Esc で閉じる動きの間に Enter を押しても何も打たない", async () => {
    withCloseAnimation();
    renderWith(<input aria-label="商品名" defaultValue="abcdef" />);
    const field = screen.getByRole<HTMLInputElement>("textbox");
    const keys = user();
    openOn(field, 1, 3);
    await keys.keyboard("{ArrowDown}");
    expect(document.activeElement).toBe(item("切り取り"));
    await keys.keyboard("{Escape}");
    expect(screen.getByRole("menu", { hidden: true })).toHaveAttribute("data-state", "closed");
    await keys.keyboard("{Enter}");
    endCloseAnimation();
    await waitClosed();
    await waitFor(() => {
      expect(document.activeElement).toBe(field);
    });
    expect(execCalls).toEqual([]);
  });

  it("UI-CTX-D3: コピーを選んで閉じる動きの間に ↑・Enter を押しても、コピーを 1 回だけ打つ", async () => {
    withCloseAnimation();
    renderWith(<input aria-label="商品名" defaultValue="abcdef" />);
    const field = screen.getByRole<HTMLInputElement>("textbox");
    const keys = user();
    openOn(field, 1, 3);
    await keys.keyboard("{ArrowDown}{ArrowDown}");
    expect(document.activeElement).toBe(item("コピー"));
    await keys.keyboard("{Enter}");
    expect(screen.getByRole("menu", { hidden: true })).toHaveAttribute("data-state", "closed");
    await keys.keyboard("{ArrowUp}");
    expect(document.activeElement).toBe(item("切り取り"));
    await keys.keyboard("{Enter}");
    endCloseAnimation();
    await waitClosed();
    await waitFor(() => {
      expect(execCalls).toHaveLength(1);
    });
    expect(execCalls[0].active).toBe(field);
    expect(execCalls[0].args).toEqual(["copy"]);
  });

  it("UI-CTX-D1: touch・pen の長押しではメニューを出さない", () => {
    vi.useFakeTimers();
    renderWith(
      <>
        <h1>見出し</h1>
        <input aria-label="商品名" defaultValue="abc" />
      </>,
    );
    for (const [target, pointerType] of [
      [screen.getByRole("heading"), "touch"],
      [screen.getByRole("textbox"), "pen"],
    ] as const) {
      fireEvent.pointerDown(target, { pointerType, clientX: 10, clientY: 10 });
      act(() => {
        vi.advanceTimersByTime(800);
      });
      expect(screen.queryByRole("menu")).not.toBeInTheDocument();
      expect(screen.queryAllByRole("menuitem")).toHaveLength(0);
    }
  });

  it("UI-CTX-D2: 選択が無いと切り取り・コピーは押せない形", async () => {
    renderWith(<input aria-label="商品名" defaultValue="abc" />);
    const field = screen.getByRole("textbox");
    openOn(field, 1);
    expect(item("切り取り")).toHaveAttribute("aria-disabled", "true");
    expect(item("コピー")).toHaveAttribute("aria-disabled", "true");
    expect(item("貼り付け")).not.toHaveAttribute("aria-disabled");
    expect(item("すべて選択")).not.toHaveAttribute("aria-disabled");

    await user().keyboard("{Escape}");
    await waitClosed();
    openOn(field, 0, 2);
    expect(item("切り取り")).not.toHaveAttribute("aria-disabled");
    expect(item("コピー")).not.toHaveAttribute("aria-disabled");
  });

  it("UI-CTX-D2: readOnly の欄は切り取り・貼り付けを押せない形", () => {
    renderWith(<input aria-label="商品コード" defaultValue="abc" readOnly />);
    openOn(screen.getByRole("textbox"), 0, 3);
    expect(item("切り取り")).toHaveAttribute("aria-disabled", "true");
    expect(item("貼り付け")).toHaveAttribute("aria-disabled", "true");
    expect(item("コピー")).not.toHaveAttribute("aria-disabled");
    expect(item("すべて選択")).not.toHaveAttribute("aria-disabled");
  });

  it("UI-CTX-D3: コピーは閉じた後に欄に focus を戻してから打つ", async () => {
    renderWith(<input aria-label="商品名" defaultValue="abcdef" />);
    const field = screen.getByRole<HTMLInputElement>("textbox");

    openOn(field, 1, 3);
    await user().click(item("コピー"));
    await waitFor(() => {
      expect(commands("copy")).toHaveLength(1);
    });
    expect(commands("copy")[0].active).toBe(field);
    expect([field.selectionStart, field.selectionEnd]).toEqual([1, 3]);

    openOn(field, 1, 3);
    await user().click(item("切り取り"));
    await waitFor(() => {
      expect(commands("cut")).toHaveLength(1);
    });
    expect(commands("cut")[0].active).toBe(field);
  });

  it("UI-CTX-D3: 貼り付けは閉じた後に欄に focus を戻してから clipboard の plugin で読み、同じ欄のときだけ insertText で入れる（待たせて別の欄へ移った後は insertText の spy が呼ばれない。読んだ文字が空なら呼ばない）", async () => {
    renderWith(
      <>
        <input aria-label="一つ目" defaultValue="abc" />
        <input aria-label="二つ目" defaultValue="xyz" />
      </>,
    );
    const first = screen.getByRole<HTMLInputElement>("textbox", { name: "一つ目" });
    const second = screen.getByRole<HTMLInputElement>("textbox", { name: "二つ目" });

    // 読めた文字は、同じ欄に insertText で入れる
    readText = () => Promise.resolve("XY");
    openOn(first, 1);
    await user().click(item("貼り付け"));
    await waitFor(() => {
      expect(commands("insertText")).toHaveLength(1);
    });
    expect(readCalls).toEqual([first]);
    expect(commands("insertText")[0]).toEqual({ args: ["insertText", false, "XY"], active: first });

    // 読みの間に別の欄へ移ると、どちらの欄にも入れない
    let resolve: ((text: string) => void) | undefined;
    readText = () => new Promise((r) => (resolve = r));
    openOn(first, 1);
    await user().click(item("貼り付け"));
    await waitFor(() => {
      expect(readCalls).toHaveLength(2);
    });
    expect(readCalls[1]).toBe(first);
    act(() => {
      second.focus();
    });
    act(() => {
      resolve?.("ZZ");
    });
    await act(() => Promise.resolve());
    expect(commands("insertText")).toHaveLength(1);
    expect([first.value, second.value]).toEqual(["abc", "xyz"]);

    // 空の文字は入れない
    readText = () => Promise.resolve("");
    openOn(first, 1);
    await user().click(item("貼り付け"));
    await waitFor(() => {
      expect(readCalls).toHaveLength(3);
    });
    await act(() => Promise.resolve());
    expect(commands("insertText")).toHaveLength(1);
    expect(browserReadText).not.toHaveBeenCalled();
  });

  it("UI-CTX-D3: 読めなかったら toast でコピーし直しを案内し欄を変えない", async () => {
    renderWith(<input aria-label="商品名" defaultValue="abc" />);
    const field = screen.getByRole<HTMLInputElement>("textbox");
    readText = () => Promise.reject(new DOMException("denied", "NotAllowedError"));
    openOn(field, 1);
    await user().click(item("貼り付け"));
    await waitFor(() => {
      expect(toast.error).toHaveBeenCalledWith(
        "貼り付ける文字がありません。貼り付けたい文字をコピーしてから、もう一度お試しください。",
        { id: "text-field-paste-error", duration: 8000 },
      );
    });
    expect(commands("insertText")).toHaveLength(0);
    expect(field.value).toBe("abc");
  });

  it("UI-CTX-D3: すべて選択で欄の全文が選ばれる", async () => {
    renderWith(<textarea aria-label="メモ" defaultValue="abcde" />);
    const field = screen.getByRole<HTMLTextAreaElement>("textbox");
    openOn(field, 2);
    await user().click(item("すべて選択"));
    await waitFor(() => {
      expect([field.selectionStart, field.selectionEnd]).toEqual([0, 5]);
    });
    expect(document.activeElement).toBe(field);
  });

  it("UI-CTX-D3: 項目・Esc・外の click で閉じると focus が欄に戻り選択が開く前のまま", async () => {
    renderWith(
      <>
        <p>本文</p>
        <input aria-label="商品名" defaultValue="abcdef" />
      </>,
    );
    const field = screen.getByRole<HTMLInputElement>("textbox");
    const kept = async () => {
      await waitClosed();
      await waitFor(() => {
        expect(document.activeElement).toBe(field);
      });
      expect([field.selectionStart, field.selectionEnd]).toEqual([1, 3]);
    };

    openOn(field, 1, 3);
    await user().click(item("コピー"));
    await kept();
    expect(commands("copy")).toHaveLength(1);

    // コピーで閉じた後に開き直して Esc: 控えた操作は空に戻っていて、もう一度打たない
    openOn(field, 1, 3);
    await user().keyboard("{Escape}");
    await kept();
    expect(commands("copy")).toHaveLength(1);

    openOn(field, 1, 3);
    await user().click(screen.getByText("本文"));
    await kept();
    expect(execCalls).toHaveLength(1);
  });

  it("UI-CTX-D4: focus の欄の contextmenu で開き矢印・Enter・Esc で操作できる", async () => {
    renderWith(<input aria-label="商品名" defaultValue="abcde" />);
    const field = screen.getByRole<HTMLInputElement>("textbox");
    // メニューキー・Shift+F10 は focus の欄に contextmenu を出す
    openOn(field, 2);
    // 選択が無いので切り取り・コピーは飛ばされ、貼り付け → すべて選択
    await user().keyboard("{ArrowDown}{ArrowDown}");
    expect(document.activeElement).toBe(item("すべて選択"));
    await user().keyboard("{Enter}");
    await waitClosed();
    await waitFor(() => {
      expect([field.selectionStart, field.selectionEnd]).toEqual([0, 5]);
    });

    openOn(field, 2);
    await user().keyboard("{Escape}");
    await waitClosed();
    await waitFor(() => {
      expect(document.activeElement).toBe(field);
    });
  });

  it("UI-CTX-D2: checkbox・disabled・日付・月・時刻の欄では何も出さない", () => {
    renderWith(
      <>
        <input aria-label="checkbox" type="checkbox" />
        <input aria-label="disabled" defaultValue="abc" disabled />
        <textarea aria-label="disabled-area" defaultValue="abc" disabled />
        <fieldset disabled>
          <input aria-label="fieldset-disabled" defaultValue="abc" />
        </fieldset>
        <input aria-label="date" type="date" />
        <input aria-label="month" type="month" />
        <input aria-label="time" type="time" />
      </>,
    );
    for (const name of [
      "checkbox",
      "disabled",
      "disabled-area",
      "fieldset-disabled",
      "date",
      "month",
      "time",
    ]) {
      expect(fireEvent.contextMenu(screen.getByLabelText(name))).toBe(false);
      expect(screen.queryByRole("menu")).not.toBeInTheDocument();
    }
  });

  it("UI-CTX-D2: IME の変換中は開かない", () => {
    renderWith(<input aria-label="商品名" defaultValue="abc" />);
    const field = screen.getByRole("textbox");
    fireEvent.compositionStart(field);
    expect(openOn(field)).toBe(false);
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
    fireEvent.compositionEnd(field);
    openOn(field);
    expect(screen.getByRole("menu")).toBeInTheDocument();
  });

  it("UI-CTX-D1: 確認の窓の中の欄で開いても窓は開いたまま", async () => {
    const onOpenChange = vi.fn();
    renderWith(
      <Dialog open onOpenChange={onOpenChange}>
        <DialogContent>
          <DialogTitle>確認</DialogTitle>
          <DialogDescription>数を入れてください。</DialogDescription>
          <input aria-label="数" defaultValue="12" />
        </DialogContent>
      </Dialog>,
    );
    const field = screen.getByRole<HTMLInputElement>("textbox", { name: "数" });
    openOn(field, 0, 2);
    expect(screen.getByRole("menu")).toBeInTheDocument();
    await user().click(item("コピー"));
    await waitClosed();
    await waitFor(() => {
      expect(document.activeElement).toBe(field);
    });

    openOn(field, 0, 2);
    await user().keyboard("{Escape}");
    await waitClosed();
    await waitFor(() => {
      expect(document.activeElement).toBe(field);
    });
    expect(screen.getByRole("dialog")).toBeInTheDocument();
    expect(onOpenChange).not.toHaveBeenCalled();
  });

  it("UI-CTX-D2: 数字の欄は選択があるときだけ切り取り・コピーを押せる形で開く", async () => {
    renderWith(<input aria-label="数量" type="number" defaultValue="123" />);
    const field = screen.getByRole<HTMLInputElement>("spinbutton");
    // 数字の欄は selectionStart が null で、選択は getSelection() で読む（P2 の実測、TD-913）
    const getSelection = vi.spyOn(window, "getSelection");

    getSelection.mockReturnValue({ toString: () => "" } as Selection);
    openOn(field);
    expect(item("切り取り")).toHaveAttribute("aria-disabled", "true");
    expect(item("コピー")).toHaveAttribute("aria-disabled", "true");
    await user().keyboard("{Escape}");
    await waitClosed();

    getSelection.mockReturnValue({ toString: () => "2" } as Selection);
    openOn(field);
    expect(item("切り取り")).not.toHaveAttribute("aria-disabled");
    expect(item("コピー")).not.toHaveAttribute("aria-disabled");
  });
});
