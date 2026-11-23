// UI-12 / UI-USW-D5 の 3・4: native の window の close を止めたときの確認の窓。
// 実の listener（installWindowCloseGuard）を Tauri の window の mock に登録し、× は登録された handler を呼んで表す。
import {
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  Link,
  Outlet,
  RouterProvider,
} from "@tanstack/react-router";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { toast } from "sonner";
import { beforeAll, beforeEach, describe, expect, it, vi } from "vitest";

import { UnsavedChangesDialog } from "@/components/patterns/UnsavedChangesDialog";
import { useUnsavedChangesWarning } from "@/hooks/useUnsavedChangesWarning";
import {
  getWindowCloseDialogKind,
  getWindowCloseGuardKinds,
  useWindowCloseGuard,
  type WindowCloseGuardKind,
} from "@/lib/window-close-guard";
import { installWindowCloseGuard } from "@/lib/window-close-listener";
import { WindowCloseDialog } from "./WindowCloseDialog";

type CloseHandler = (event: { preventDefault: () => void }) => void | Promise<void>;

const tauriWindow = vi.hoisted(() => ({
  handlers: [] as CloseHandler[],
  destroy: vi.fn<() => Promise<void>>(),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    onCloseRequested: (handler: CloseHandler) => {
      tauriWindow.handlers.push(handler);
      return Promise.resolve(() => undefined);
    },
    destroy: tauriWindow.destroy,
  }),
}));
vi.mock("sonner", () => ({ toast: { error: vi.fn() } }));

const BUSY_COPY = [
  {
    kind: "z004-import",
    title: "取込みの途中です",
    description:
      "取込みが終わるまでお待ちください。終わると閉じられます。いま閉じると、取り込めたかどうかをこの画面で確かめられなくなります。閉じたときは、次にアプリを開いてから「売上データ取込み」の「商品別CSV取込み（Z004）」タブの「取り込んだ記録を見る」で確かめてください。記録に無ければ、もう一度取り込んでください。",
  },
  {
    kind: "z004-rollback",
    title: "取消の途中です",
    description:
      "取消が終わるまでお待ちください。終わると閉じられます。いま閉じると、取り消せたかどうかをこの画面で確かめられなくなります。閉じたときは、次にアプリを開いてから「売上データ取込み」の「商品別CSV取込み（Z004）」タブの「取り込んだ記録を見る」で、取消済みかを確かめてください。取消済みでなければ、もう一度取り消してください。",
  },
  {
    kind: "daily-report-import",
    title: "取込みの途中です",
    description:
      "取込みが終わるまでお待ちください。終わると閉じられます。いま閉じると、取り込めたかどうかをこの画面で確かめられなくなります。閉じたときは、次にアプリを開いてから「売上データ取込み」の「日報取込み」タブの「取り込んだ日報の記録を見る」で確かめてください。記録に無ければ、もう一度取り込んでください。",
  },
  {
    kind: "daily-report-rollback",
    title: "取消の途中です",
    description:
      "取消が終わるまでお待ちください。終わると閉じられます。いま閉じると、取り消せたかどうかをこの画面で確かめられなくなります。閉じたときは、次にアプリを開いてから「売上データ取込み」の「日報取込み」タブの「取り込んだ日報の記録を見る」で、取消済みかを確かめてください。取消済みでなければ、もう一度取り消してください。",
  },
  {
    kind: "sd-bulk-import",
    title: "取込みの途中です",
    description:
      "取込みが終わるまでお待ちください。終わると閉じられます。いま閉じると、どこまで取り込めたかをこの画面で確かめられなくなります。日報は 1 つずつ確定するので、一部だけが入っていることがあります。閉じたときは、次にアプリを開いてから「売上データ取込み」の「日報取込み」タブで同じ SD を読み直し、「まとめて取り込む」が押せればもう一度押してください。取り込み済みの日報と控えは二重に入りません。",
  },
  {
    kind: "ej-import",
    title: "取込みの途中です",
    description:
      "取込みが終わるまでお待ちください。終わると閉じられます。いま閉じると、取り込めたかどうかをこの画面で確かめられなくなります。閉じたときは、次にアプリを開いてから「売上データ取込み」の「日報取込み」タブで同じ SD を読み直し、「まとめて取り込む」が押せれば押してください。結果に欠けか読めないファイルの文が残っていれば、「EJ のファイルを選んで取り込む」で同じファイルを選び直してください。取り込み済みの控えは二重に入りません。",
  },
  {
    // 取消の応答を待つ間にも、確かめの読取りを待つ間（取消済みと確かめた後のこともある）にも出る:
    // 取消が途中とも終わったとも言わない
    kind: "inventory-record-cancel",
    title: "この画面を閉じますか",
    description:
      "閉じた後は、次にアプリを開いてから「入出庫履歴」でこの記録を開き、取消済みかを確かめてください。取消済みでなければ、もう一度取り消してください。",
  },
] as const satisfies readonly { kind: WindowCloseGuardKind; title: string; description: string }[];

const DIRTY_TITLE = "編集内容が保存されていません";
const DIRTY_DESCRIPTION = "このまま閉じると、入力した内容は破棄されます。";

function Guard({ kind, active = true }: { kind: WindowCloseGuardKind; active?: boolean }) {
  useWindowCloseGuard(kind, active);
  return null;
}

/** × / Alt+F4: Tauri が登録済みの handler を呼ぶ。止めたかは preventDefault の呼出しで見る */
function pressClose() {
  const event = { preventDefault: vi.fn() };
  act(() => {
    void tauriWindow.handlers[0](event);
  });
  return event;
}

function overlay(): Element {
  const overlays = document.querySelectorAll('[data-slot="alert-dialog-overlay"]');
  if (overlays.length === 0) throw new Error("alert-dialog-overlay not found");
  return overlays[overlays.length - 1];
}

beforeAll(async () => {
  installWindowCloseGuard();
  await waitFor(() => {
    expect(tauriWindow.handlers).toHaveLength(1);
  });
});

beforeEach(() => {
  tauriWindow.destroy.mockResolvedValue(undefined);
});

describe("WindowCloseDialog (UI-12/UI-USW-D5)", () => {
  it("T2 UI-USW-D5/D4: 確認の窓を mount していても、集合が空なら止めず何も出さない", () => {
    render(
      <>
        <Guard kind="dirty" active={false} />
        <WindowCloseDialog />
      </>,
    );

    const event = pressClose();

    expect(event.preventDefault).not.toHaveBeenCalled();
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    expect(tauriWindow.destroy).not.toHaveBeenCalled();
  });

  it("T3 UI-USW-D5/D2: 未保存なら止めて破棄の確認を出し、既定 focus は「編集を続ける」", async () => {
    render(
      <>
        <Guard kind="dirty" />
        <WindowCloseDialog />
      </>,
    );

    const event = pressClose();

    expect(event.preventDefault).toHaveBeenCalledTimes(1);
    const dialog = screen.getByRole("alertdialog", { name: DIRTY_TITLE });
    expect(dialog).toHaveAccessibleDescription(DIRTY_DESCRIPTION);
    await waitFor(() => {
      expect(within(dialog).getByRole("button", { name: "編集を続ける" })).toHaveFocus();
    });
    expect(within(dialog).getByRole("button", { name: "破棄して閉じる" })).toBeInTheDocument();
    expect(tauriWindow.destroy).not.toHaveBeenCalled();
  });

  it("T3 UI-USW-D5: 「編集を続ける」は確認の窓だけを閉じ、止めは集合に残る", async () => {
    const user = userEvent.setup();
    render(
      <>
        <Guard kind="dirty" />
        <input aria-label="編集中の値" defaultValue="保持される入力" />
        <WindowCloseDialog />
      </>,
    );
    pressClose();

    await user.click(screen.getByRole("button", { name: "編集を続ける" }));

    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    expect(tauriWindow.destroy).not.toHaveBeenCalled();
    expect(getWindowCloseGuardKinds()).toEqual(["dirty"]);
    expect(screen.getByLabelText("編集中の値")).toHaveValue("保持される入力");
    // もう一度 × を押すと同じ窓が出る
    expect(pressClose().preventDefault).toHaveBeenCalledTimes(1);
    expect(screen.getByRole("alertdialog", { name: DIRTY_TITLE })).toBeInTheDocument();
  });

  it("T3 UI-USW-D5: 「破棄して閉じる」は destroy を 1 回呼ぶ", async () => {
    const user = userEvent.setup();
    render(
      <>
        <Guard kind="dirty" />
        <WindowCloseDialog />
      </>,
    );
    pressClose();

    await user.click(screen.getByRole("button", { name: "破棄して閉じる" }));

    expect(tauriWindow.destroy).toHaveBeenCalledTimes(1);
    expect(toast.error).not.toHaveBeenCalled();
  });

  it.each(BUSY_COPY)(
    "T4 UI-USW-D5 / 55 §55.7: $kind は止めて種類どおりの窓を出し、既定 focus は「待つ」",
    async ({ kind, title, description }) => {
      render(
        <>
          <Guard kind="dirty" />
          <Guard kind={kind} />
          <WindowCloseDialog />
        </>,
      );

      const event = pressClose();

      // 未保存と進行中が同時なら進行中の窓
      expect(event.preventDefault).toHaveBeenCalledTimes(1);
      const dialog = screen.getByRole("alertdialog", { name: title });
      expect(dialog).toHaveAccessibleDescription(description);
      // TD-839: 閉じた後の案内で取込みの完了を言い切らない
      expect(dialog).not.toHaveTextContent("取り込めています");
      await waitFor(() => {
        expect(within(dialog).getByRole("button", { name: "待つ" })).toHaveFocus();
      });
      expect(within(dialog).getByRole("button", { name: "閉じる" })).toBeInTheDocument();
      expect(tauriWindow.destroy).not.toHaveBeenCalled();
    },
  );

  it("T4 UI-USW-D5: 進行中の窓は「待つ」で閉じ、destroy を呼ばない", async () => {
    const user = userEvent.setup();
    render(
      <>
        <Guard kind="z004-import" />
        <WindowCloseDialog />
      </>,
    );
    pressClose();

    await user.click(screen.getByRole("button", { name: "待つ" }));

    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    expect(tauriWindow.destroy).not.toHaveBeenCalled();
    expect(getWindowCloseGuardKinds()).toEqual(["z004-import"]);
  });

  it("T4 UI-USW-D5 / DSR-20: 進行中の窓の Esc は「待つ」と同じ（確認の窓だけを閉じる）", () => {
    render(
      <>
        <Guard kind="daily-report-import" />
        <WindowCloseDialog />
      </>,
    );
    pressClose();
    const dialog = screen.getByRole("alertdialog", { name: "取込みの途中です" });

    fireEvent.keyDown(dialog, { key: "Escape", code: "Escape" });

    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    expect(getWindowCloseDialogKind()).toBeNull();
    expect(tauriWindow.destroy).not.toHaveBeenCalled();
  });

  it("T4 UI-USW-D5 / DSR-20: 進行中の窓は外側の click では閉じない", () => {
    render(
      <>
        <Guard kind="z004-rollback" />
        <WindowCloseDialog />
      </>,
    );
    pressClose();
    const dialog = screen.getByRole("alertdialog", { name: "取消の途中です" });

    fireEvent.pointerDown(overlay());
    fireEvent.click(overlay());

    // 閉じる経路は onOpenChange だけなので、窓の状態が変わらないこと = onOpenChange が呼ばれていないこと
    expect(dialog).toBeInTheDocument();
    expect(getWindowCloseDialogKind()).toBe("z004-rollback");
    expect(tauriWindow.destroy).not.toHaveBeenCalled();
  });

  it("T4 UI-USW-D5: 進行中の種類が集合から外れると窓は自動で閉じ、その後の × は止めない", () => {
    const { rerender } = render(
      <>
        <Guard kind="daily-report-rollback" />
        <WindowCloseDialog />
      </>,
    );
    pressClose();
    expect(screen.getByRole("alertdialog", { name: "取消の途中です" })).toBeInTheDocument();

    rerender(
      <>
        <Guard kind="daily-report-rollback" active={false} />
        <WindowCloseDialog />
      </>,
    );

    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    expect(pressClose().preventDefault).not.toHaveBeenCalled();
    expect(tauriWindow.destroy).not.toHaveBeenCalled();
  });

  it("T4 UI-USW-D5: 進行中の窓の「閉じる」は destroy を 1 回呼ぶ", async () => {
    const user = userEvent.setup();
    render(
      <>
        <Guard kind="z004-import" />
        <WindowCloseDialog />
      </>,
    );
    pressClose();

    await user.click(screen.getByRole("button", { name: "閉じる" }));

    expect(tauriWindow.destroy).toHaveBeenCalledTimes(1);
    expect(toast.error).not.toHaveBeenCalled();
  });

  it("T5 UI-USW-D2 / DSR-20: 未保存の窓は Esc と外側の click で閉じない", () => {
    render(
      <>
        <Guard kind="dirty" />
        <WindowCloseDialog />
      </>,
    );
    pressClose();
    const dialog = screen.getByRole("alertdialog", { name: DIRTY_TITLE });

    // onOpenChange は「編集を続ける」と同じ処理へ配線してあるので、Esc の preventDefault を外すと窓が閉じる
    fireEvent.keyDown(dialog, { key: "Escape", code: "Escape" });
    expect(dialog).toBeInTheDocument();
    expect(getWindowCloseDialogKind()).toBe("dirty");

    fireEvent.pointerDown(overlay());
    fireEvent.click(overlay());
    expect(dialog).toBeInTheDocument();
    expect(getWindowCloseDialogKind()).toBe("dirty");
    expect(tauriWindow.destroy).not.toHaveBeenCalled();
  });

  it("T6 UI-USW-D5 / UI-EB-D1: 確認の窓が unmount されていれば、止めがあっても止めない（F6）", () => {
    const { rerender } = render(
      <>
        <Guard kind="z004-import" />
        <WindowCloseDialog />
      </>,
    );
    rerender(<Guard kind="z004-import" />);
    expect(getWindowCloseGuardKinds()).toEqual(["z004-import"]);

    const event = pressClose();

    expect(event.preventDefault).not.toHaveBeenCalled();
    expect(getWindowCloseDialogKind()).toBeNull();
  });

  it("T7 UI-USW-D5: 確認の窓が出ている間の 2 回目の × は止めるだけで、窓を重ねず差し替えない（F5）", () => {
    const { rerender } = render(
      <>
        <Guard kind="dirty" />
        <Guard kind="z004-import" active={false} />
        <WindowCloseDialog />
      </>,
    );
    pressClose();
    rerender(
      <>
        <Guard kind="dirty" />
        <Guard kind="z004-import" />
        <WindowCloseDialog />
      </>,
    );

    const second = pressClose();

    expect(second.preventDefault).toHaveBeenCalledTimes(1);
    expect(screen.getAllByRole("alertdialog")).toHaveLength(1);
    expect(screen.getByRole("alertdialog", { name: DIRTY_TITLE })).toBeInTheDocument();
    expect(getWindowCloseDialogKind()).toBe("dirty");
    expect(tauriWindow.destroy).not.toHaveBeenCalled();
  });

  it.each([
    { kind: "dirty", close: "破棄して閉じる" },
    { kind: "z004-import", close: "閉じる" },
  ] as const)(
    "T13 UI-USW-D5: 「$close」の destroy が拒まれたら、確認の窓を閉じて直し方を Toast で出す",
    async ({ kind, close }) => {
      const user = userEvent.setup();
      tauriWindow.destroy.mockRejectedValue(new Error("合成の destroy の失敗"));
      render(
        <>
          <Guard kind={kind} />
          <WindowCloseDialog />
        </>,
      );
      pressClose();

      await user.click(screen.getByRole("button", { name: close }));

      await waitFor(() => {
        expect(toast.error).toHaveBeenCalledWith(
          "閉じられませんでした。もう一度 × を押してください。",
        );
      });
      expect(toast.error).toHaveBeenCalledTimes(1);
      expect(tauriWindow.destroy).toHaveBeenCalledTimes(1);
      expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
      // 止めは残っているので、もう一度 × を押すと同じ窓が出る
      expect(pressClose().preventDefault).toHaveBeenCalledTimes(1);
      expect(screen.getByRole("alertdialog")).toBeInTheDocument();
    },
  );

  it("T14 UI-USW-D5/D2: route の確認の窓の上に close の窓が出て、「編集を続ける」は close の窓だけを閉じる", async () => {
    const user = userEvent.setup();
    function DirtyPage() {
      const warning = useUnsavedChangesWarning(true);
      return (
        <>
          <Link to={"/next" as never}>移動する</Link>
          <UnsavedChangesDialog warning={warning} />
        </>
      );
    }
    const rootRoute = createRootRoute({
      component: () => (
        <>
          <Outlet />
          <WindowCloseDialog />
        </>
      ),
    });
    const router = createRouter({
      routeTree: rootRoute.addChildren([
        createRoute({ getParentRoute: () => rootRoute, path: "/", component: DirtyPage }),
        createRoute({
          getParentRoute: () => rootRoute,
          path: "/next",
          component: () => <h1>移動先</h1>,
        }),
      ]),
      history: createMemoryHistory({ initialEntries: ["/"] }),
    });
    render(<RouterProvider router={router} />);
    await user.click(await screen.findByRole("link", { name: "移動する" }));
    const routeDialog = await screen.findByRole("alertdialog", { name: DIRTY_TITLE });
    expect(routeDialog).toHaveAccessibleDescription(
      "このまま移動すると、入力した内容は破棄されます。",
    );

    // route の窓が出ている間の × も止め、close の窓を後ろ（上）に出す
    expect(pressClose().preventDefault).toHaveBeenCalledTimes(1);
    const dialogs = screen.getAllByRole("alertdialog", { hidden: true });
    expect(dialogs).toHaveLength(2);
    expect(dialogs[0]).toBe(routeDialog);
    expect(dialogs[1]).toHaveAccessibleDescription(DIRTY_DESCRIPTION);

    await user.click(within(dialogs[1]).getByRole("button", { name: "編集を続ける" }));

    // close の窓だけが消え、route の窓は保留のまま残る（proceed も reset もされていない）
    await waitFor(() => {
      expect(screen.getAllByRole("alertdialog", { hidden: true })).toHaveLength(1);
    });
    expect(routeDialog).toBeInTheDocument();
    expect(router.state.location.pathname).toBe("/");
    expect(tauriWindow.destroy).not.toHaveBeenCalled();

    pressClose();
    const reopened = screen.getAllByRole("alertdialog", { hidden: true });
    await user.click(within(reopened[1]).getByRole("button", { name: "破棄して閉じる" }));

    expect(tauriWindow.destroy).toHaveBeenCalledTimes(1);
    expect(routeDialog).toBeInTheDocument();
    expect(router.state.location.pathname).toBe("/");
  });
});
