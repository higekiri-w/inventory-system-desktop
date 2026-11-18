import { readFileSync } from "node:fs";

import { fireEvent, render, renderHook, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { UnsavedChangesDialog } from "@/components/patterns/UnsavedChangesDialog";
import { consumeMainNavScroll, markMainNavScroll } from "@/lib/main-nav-scroll";
import { getWindowCloseGuardKinds } from "@/lib/window-close-guard";
import { useUnsavedChangesWarning } from "./useUnsavedChangesWarning";

interface TestBlockerOptions {
  shouldBlockFn: (args: unknown) => boolean;
  withResolver: boolean;
  enableBeforeUnload: boolean | (() => boolean);
}

interface TestBlockerResolver {
  status: "idle" | "blocked";
  current?: unknown;
  next?: unknown;
  action?: unknown;
  proceed?: () => void;
  reset?: () => void;
}

const mockUseBlocker = vi.hoisted(() =>
  vi.fn<(options: TestBlockerOptions) => TestBlockerResolver>(),
);

vi.mock("@tanstack/react-router", () => ({ useBlocker: mockUseBlocker }));

const reset = vi.fn();
const proceed = vi.fn();

function idleResolver() {
  return {
    status: "idle" as const,
    current: undefined,
    next: undefined,
    action: undefined,
    proceed: undefined,
    reset: undefined,
  };
}

function blockedResolver() {
  return {
    status: "blocked" as const,
    current: {} as never,
    next: {} as never,
    action: "PUSH" as never,
    proceed,
    reset,
  };
}

function Harness({ isDirty }: { isDirty: boolean }) {
  const warning = useUnsavedChangesWarning(isDirty);
  return (
    <>
      <input aria-label="編集中の値" defaultValue="保持される入力" />
      <UnsavedChangesDialog warning={warning} />
    </>
  );
}

beforeEach(() => {
  consumeMainNavScroll();
  mockUseBlocker.mockReturnValue(idleResolver());
  reset.mockReset();
  proceed.mockReset();
});

describe("useUnsavedChangesWarning (UI-12/UI-USW-D1/D2 / SPEC-UISN-2/3)", () => {
  it("UI-USW-D1 T1/T5: isDirty=false は遷移と beforeunload を block しない", () => {
    renderHook(() => useUnsavedChangesWarning(false));

    const options = mockUseBlocker.mock.calls[mockUseBlocker.mock.calls.length - 1][0];
    expect(options.withResolver).toBe(true);
    expect(options.shouldBlockFn({} as never)).toBe(false);
    expect(typeof options.enableBeforeUnload).toBe("function");
    expect((options.enableBeforeUnload as () => boolean)()).toBe(false);
  });

  it("UI-USW-D1/D2 T2: isDirty=true は遷移を block して破棄確認を表示する", async () => {
    mockUseBlocker.mockReturnValue(blockedResolver());
    render(<Harness isDirty />);

    const options = mockUseBlocker.mock.calls[mockUseBlocker.mock.calls.length - 1][0];
    expect(options.shouldBlockFn({} as never)).toBe(true);
    expect((options.enableBeforeUnload as () => boolean)()).toBe(true);
    expect(await screen.findByText("編集内容が保存されていません")).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "編集を続ける" })).toHaveFocus();
    });
  });

  it("UI-USW-D2 T3: 編集を続けると reset して入力を保持する", async () => {
    const user = userEvent.setup();
    mockUseBlocker.mockReturnValue(blockedResolver());
    render(<Harness isDirty />);

    await user.click(await screen.findByRole("button", { name: "編集を続ける" }));

    expect(reset).toHaveBeenCalledTimes(1);
    expect(proceed).not.toHaveBeenCalled();
    expect(screen.getByLabelText("編集中の値")).toHaveValue("保持される入力");
  });

  it("DSR-17: 主ナビ遷移を取り消すと pending marker を破棄する", async () => {
    const user = userEvent.setup();
    mockUseBlocker.mockReturnValue(blockedResolver());
    markMainNavScroll("/products");
    render(<Harness isDirty />);

    await user.click(await screen.findByRole("button", { name: "編集を続ける" }));

    expect(reset).toHaveBeenCalledTimes(1);
    expect(consumeMainNavScroll()).toBeUndefined();
  });

  it("UI-USW-D2 T4: 破棄して移動すると proceed する", async () => {
    const user = userEvent.setup();
    mockUseBlocker.mockReturnValue(blockedResolver());
    render(<Harness isDirty />);

    await user.click(await screen.findByRole("button", { name: "破棄して移動" }));

    expect(proceed).toHaveBeenCalledTimes(1);
    expect(reset).not.toHaveBeenCalled();
  });

  it("DSR-20 D-E T10: Escape と外側 pointer-down では閉じず callback も発火しない", async () => {
    mockUseBlocker.mockReturnValue(blockedResolver());
    render(<Harness isDirty />);

    const dialog = await screen.findByRole("alertdialog", {
      name: "編集内容が保存されていません",
    });
    const overlay = document.querySelector('[data-slot="alert-dialog-overlay"]');
    if (overlay === null) throw new Error("alert-dialog-overlay not found");

    fireEvent.keyDown(dialog, { key: "Escape", code: "Escape" });
    expect(dialog).toBeInTheDocument();
    expect(reset).not.toHaveBeenCalled();
    expect(proceed).not.toHaveBeenCalled();

    fireEvent.pointerDown(overlay);
    expect(dialog).toBeInTheDocument();
    expect(reset).not.toHaveBeenCalled();
    expect(proceed).not.toHaveBeenCalled();
  });

  // sibling test「DSR-20 D-E T10: Escape と外側 pointer-down では閉じず callback も発火しない」は Escape/pointer-down で閉じないことを観測するが、AlertDialog は
  // open prop のみで制御され onOpenChange は未配線のため、preventDefault() を削除しても
  // その sibling test の結果は変わらない（実測済み）。preventDefault() 欠落を kill するのは本 test の
  // source assertion のみ（formatter 非依存、comment 挿入や型注釈変更では FAIL し得る）。
  // sibling test の挙動 oracle 強化（preventDefault 除去まで検出させる改修）は本 PR の scope 外。
  it("DSR-20 D-E T10: Escape の preventDefault を明示 prop として保持する", () => {
    const source = readFileSync("src/components/patterns/UnsavedChangesDialog.tsx", "utf8");
    const normalized = source.replace(/\s+/g, " ");

    expect(normalized).toContain("onEscapeKeyDown={(event) => { event.preventDefault();");
  });

  it("T12 UI-USW-D1/D5: isDirty の間だけ native の window の close の止めに dirty を入れる", () => {
    // useBlocker の mock の値（idle）に依らず、渡した isDirty で決まる
    const { rerender, unmount } = renderHook(({ isDirty }) => useUnsavedChangesWarning(isDirty), {
      initialProps: { isDirty: false },
    });
    expect(getWindowCloseGuardKinds()).toEqual([]);

    rerender({ isDirty: true });
    expect(getWindowCloseGuardKinds()).toEqual(["dirty"]);

    rerender({ isDirty: false });
    expect(getWindowCloseGuardKinds()).toEqual([]);

    rerender({ isDirty: true });
    unmount();
    expect(getWindowCloseGuardKinds()).toEqual([]);
  });
});
