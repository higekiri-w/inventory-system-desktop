// UI-12 / UI-USW-D5 の 1・3・8: native の window の close の止めの集合と判定（store だけ。Tauri を引かない）
import { render, renderHook } from "@testing-library/react";
import { StrictMode } from "react";
import { afterEach, describe, expect, it } from "vitest";

import {
  closeWindowCloseDialog,
  getWindowCloseDialogKind,
  getWindowCloseGuardKinds,
  hasBusyWindowCloseGuard,
  registerWindowCloseDialog,
  requestWindowClose,
  useWindowCloseGuard,
  type WindowCloseGuardKind,
} from "./window-close-guard";

function Guard({ kind, active }: { kind: WindowCloseGuardKind; active: boolean }) {
  useWindowCloseGuard(kind, active);
  return null;
}

let unregister: (() => void) | undefined;

/** 確認の窓を出す者がいる状態にする（component の代わりに store へ直に登録する） */
function mountPresenter() {
  unregister = registerWindowCloseDialog();
}

afterEach(() => {
  unregister?.();
  unregister = undefined;
  closeWindowCloseDialog();
});

describe("window-close-guard (UI-12/UI-USW-D5)", () => {
  it("UI-USW-D5 の 3: 出す者がいて集合が空なら止めず、確認の窓も開かない", () => {
    mountPresenter();

    expect(requestWindowClose()).toBe(false);
    expect(getWindowCloseDialogKind()).toBeNull();
  });

  it("UI-USW-D5 の 3: 出す者がいなければ、集合に止めがあっても止めない（F6）", () => {
    renderHook(() => {
      useWindowCloseGuard("dirty", true);
    });

    expect(getWindowCloseGuardKinds()).toEqual(["dirty"]);
    expect(requestWindowClose()).toBe(false);
    expect(getWindowCloseDialogKind()).toBeNull();
  });

  it("UI-USW-D5 の 3: active の間だけ集合に入り、未保存だけなら未保存の窓を開いて止める", () => {
    mountPresenter();
    const { rerender } = renderHook(
      ({ active }) => {
        useWindowCloseGuard("dirty", active);
      },
      { initialProps: { active: false } },
    );
    expect(getWindowCloseGuardKinds()).toEqual([]);

    rerender({ active: true });
    expect(getWindowCloseGuardKinds()).toEqual(["dirty"]);
    expect(requestWindowClose()).toBe(true);
    expect(getWindowCloseDialogKind()).toBe("dirty");

    // 「編集を続ける」: 確認の窓だけを閉じ、集合は残る（次の × でまた止まる）
    closeWindowCloseDialog();
    expect(getWindowCloseDialogKind()).toBeNull();
    expect(getWindowCloseGuardKinds()).toEqual(["dirty"]);

    rerender({ active: false });
    expect(getWindowCloseGuardKinds()).toEqual([]);
    expect(requestWindowClose()).toBe(false);
  });

  it("UI-USW-D5 の 3: 未保存と進行中が同時なら進行中の窓、進行中が複数なら先に入った方", () => {
    mountPresenter();
    render(
      <>
        <Guard kind="dirty" active />
        <Guard kind="daily-report-rollback" active />
        <Guard kind="z004-import" active />
      </>,
    );

    expect(requestWindowClose()).toBe(true);
    expect(getWindowCloseDialogKind()).toBe("daily-report-rollback");
  });

  it("UI-USW-D5 の 3: 確認の窓が開いている間の 2 回目の × は、止めるだけで窓を差し替えない（F5）", () => {
    mountPresenter();
    const { rerender } = render(
      <>
        <Guard kind="dirty" active />
        <Guard kind="z004-import" active={false} />
      </>,
    );
    expect(requestWindowClose()).toBe(true);
    expect(getWindowCloseDialogKind()).toBe("dirty");

    rerender(
      <>
        <Guard kind="dirty" active />
        <Guard kind="z004-import" active />
      </>,
    );
    expect(requestWindowClose()).toBe(true);
    expect(getWindowCloseDialogKind()).toBe("dirty");
  });

  it("UI-USW-D5 の 4: 開いている窓の種類が集合から外れると、確認の窓は自動で閉じる", () => {
    mountPresenter();
    const { rerender } = render(
      <>
        <Guard kind="z004-rollback" active />
        <Guard kind="dirty" active />
      </>,
    );
    expect(requestWindowClose()).toBe(true);
    expect(getWindowCloseDialogKind()).toBe("z004-rollback");

    // 別の種類が外れても、開いている窓はそのまま
    rerender(
      <>
        <Guard kind="z004-rollback" active />
        <Guard kind="dirty" active={false} />
      </>,
    );
    expect(getWindowCloseDialogKind()).toBe("z004-rollback");

    rerender(
      <>
        <Guard kind="z004-rollback" active={false} />
        <Guard kind="dirty" active={false} />
      </>,
    );
    expect(getWindowCloseDialogKind()).toBeNull();
    expect(requestWindowClose()).toBe(false);
  });

  it("T8 UI-USW-D5 の 1: StrictMode でも出し入れが対になり、同じ種類の止めが互いを消さない（F4）", () => {
    mountPresenter();
    const { rerender, unmount } = render(
      <StrictMode>
        <Guard kind="dirty" active />
        <Guard kind="dirty" active />
      </StrictMode>,
    );
    // 二重の effect でも、hook の instance ごとに 1 つずつ
    expect(getWindowCloseGuardKinds()).toEqual(["dirty", "dirty"]);

    rerender(
      <StrictMode>
        <Guard kind="dirty" active={false} />
        <Guard kind="dirty" active />
      </StrictMode>,
    );
    expect(getWindowCloseGuardKinds()).toEqual(["dirty"]);
    expect(requestWindowClose()).toBe(true);
    closeWindowCloseDialog();

    unmount();
    expect(getWindowCloseGuardKinds()).toEqual([]);
    expect(requestWindowClose()).toBe(false);
  });

  it("UI-USW-D5 の 8 / UI-11b-D13: 進行中の getter は進行中の 7 つの種類だけを数え、未保存は数えない", () => {
    const { rerender } = render(<Guard kind="dirty" active />);
    expect(hasBusyWindowCloseGuard()).toBe(false);

    for (const kind of [
      "z004-import",
      "z004-rollback",
      "daily-report-import",
      "daily-report-rollback",
      "sd-bulk-import",
      "ej-import",
      "inventory-record-cancel",
    ] as const) {
      rerender(
        <>
          <Guard kind="dirty" active />
          <Guard key={kind} kind={kind} active />
        </>,
      );
      expect(hasBusyWindowCloseGuard(), kind).toBe(true);
    }

    rerender(<Guard kind="dirty" active />);
    expect(hasBusyWindowCloseGuard()).toBe(false);
  });
});
