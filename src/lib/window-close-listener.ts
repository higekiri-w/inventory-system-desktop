// src/lib/window-close-listener.ts
//
// UI-USW-D5 の 2・3: native の window の close（×・Alt+F4）の listener を、窓に 1 つだけ登録する。
// listener を 1 つでも置くと Rust が close を止め、閉じるのは常に JS の destroy の仕事になる。
// そのため、止めの仕組みが壊れたときは「止めずに閉じる」側に倒す（fail-open、UI-USW-D5 の 5）。
// 設計: docs/UI_TECH_STACK.md §6.11 UI-USW-D5

import { getCurrentWindow, type Window as TauriWindow } from "@tauri-apps/api/window";

import { requestWindowClose } from "@/lib/window-close-guard";

type CloseWindow = Pick<TauriWindow, "onCloseRequested" | "destroy">;

// この webview で登録を試み、失敗で終わっていない印。page の読み直しでは残り、アプリを起動し直すと消える。
const MARK_KEY = "window-close-guard:listener";

let installed = false;

/** 読めないときは印が無いものとして扱う（最初の page の失敗でアプリが勝手に閉じないことを優先する） */
function readMark(): boolean {
  try {
    return window.sessionStorage.getItem(MARK_KEY) !== null;
  } catch {
    return false;
  }
}

/** 書けない・消せないときは、その例外を無視して読めた値で続ける */
function writeMark(present: boolean): void {
  try {
    if (present) window.sessionStorage.setItem(MARK_KEY, "1");
    else window.sessionStorage.removeItem(MARK_KEY);
  } catch {
    // 無視する（UI-USW-D5 の 2）
  }
}

/** UI-USW-D5 の 3: 止めるときだけ preventDefault する。止めないときは onCloseRequested の包みが destroy で閉じる */
function handleCloseRequested(event: { preventDefault: () => void }): void {
  try {
    if (requestWindowClose()) event.preventDefault();
  } catch (error) {
    // (5) 判定の例外は止めない。ここで投げると包みが destroy に届かず、閉じられない窓になる
    console.warn("window の close の判定に失敗しました。止めずに閉じます:", error);
  }
}

async function register(getWindow: () => CloseWindow): Promise<void> {
  let hadMark = false;
  try {
    hadMark = readMark();
    writeMark(true);
    // 窓はここで取る（Tauri の外では getCurrentWindow() が同期で投げるので、既定の引数では呼ばない）
    const unlisten = await getWindow().onCloseRequested(handleCloseRequested);
    // 開発の HMR でこの module が差し替わるときは、古い listener を外して最初の page の扱いへ戻す
    import.meta.hot?.dispose(() => {
      unlisten();
      writeMark(false);
    });
  } catch (error) {
    console.warn("window の close の listener を登録できませんでした:", error);
    if (!hadMark) {
      // 起動して最初の page: 前の page の listener は無く、Rust は close を止めない。× はそのまま閉じる
      writeMark(false);
      return;
    }
    // 読み直しの後: 前の page の listener が Rust に残って close を止め続けるので、アプリが自分で閉じる
    try {
      await getWindow().destroy();
    } catch (destroyError) {
      console.warn("window を閉じられませんでした:", destroyError);
    }
  }
}

/** `src/main.tsx` が render の前に 1 回だけ呼ぶ。2 回目の呼出しは無視する。登録の失敗で app の起動を止めない */
export function installWindowCloseGuard(getWindow: () => CloseWindow = getCurrentWindow): void {
  if (installed) return;
  installed = true;
  void register(getWindow);
}
