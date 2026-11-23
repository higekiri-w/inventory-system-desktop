// src/lib/window-close-guard.ts
//
// UI-USW-D5: native の window の close（×・Alt+F4）の止めの集合と、確認の窓の状態。
// 止めの hook（UI-USW-D1 の hook と 55 §55.7 の進行中の処理のガードの hook）が useBlocker と同じ条件を
// useWindowCloseGuard に渡し、× を受けた listener（window-close-listener.ts）が requestWindowClose で判定する。
// 多くの画面の test は window を mock していないので、ここは @tauri-apps/api を import しない。
// 設計: docs/UI_TECH_STACK.md §6.11 UI-USW-D5 の 1・3・8

import { useEffect } from "react";

/** 進行中（取込み・取消の応答を待つ間）の 7 つと、未保存の入力の `dirty` */
export type WindowCloseGuardKind =
  | "z004-import"
  | "z004-rollback"
  | "daily-report-import"
  | "daily-report-rollback"
  | "sd-bulk-import"
  | "ej-import"
  | "inventory-record-cancel"
  | "dirty";

// ponytail: 窓は 1 つ（tauri.conf.json の windows が 1 件）なので、module の 1 つの store で持つ。
// 窓が 2 つ以上になれば、窓の label ごとに持つ。
// 鍵は hook の instance ごと（同じ種類の止めが 2 つあっても互いを消さない）。Map は入れた順を保つ。
const guards = new Map<symbol, WindowCloseGuardKind>();
// 確認の窓の状態: 閉じていれば null、開いていれば開いた時の種類
let dialogKind: WindowCloseGuardKind | null = null;
// 確認の窓を出す者（WindowCloseDialog）の mount の数
let presenters = 0;
const listeners = new Set<() => void>();

function isBusyKind(kind: WindowCloseGuardKind): boolean {
  return kind !== "dirty";
}

function setDialogKind(next: WindowCloseGuardKind | null): void {
  if (dialogKind === next) return;
  dialogKind = next;
  for (const listener of listeners) listener();
}

/** 集合にある種類（入れた順） */
export function getWindowCloseGuardKinds(): WindowCloseGuardKind[] {
  return [...guards.values()];
}

/** UI-USW-D5 の 8・UI-11b-D13: 進行中の止めが集合にあるか（useAutoBackupCheck が回のたびに読む） */
export function hasBusyWindowCloseGuard(): boolean {
  return getWindowCloseGuardKinds().some(isBusyKind);
}

/** `active` の間だけ、この hook の instance の鍵で種類を集合に入れる。useBlocker と同じ条件を渡す */
export function useWindowCloseGuard(kind: WindowCloseGuardKind, active: boolean): void {
  useEffect(() => {
    if (!active) return;
    const key = Symbol(kind);
    guards.set(key, kind);
    return () => {
      guards.delete(key);
      // 開いている確認の窓の種類が集合から無くなったら、確認の窓を閉じる（進行中が終われば、その後の × で閉じられる）
      if (dialogKind !== null && !getWindowCloseGuardKinds().includes(dialogKind)) {
        setDialogKind(null);
      }
    };
  }, [kind, active]);
}

/**
 * UI-USW-D5 の 3 の (1)〜(4): × を受けたときの判定（同期）。止めるなら true を返す。
 * (5) の「例外なら止めない」は呼ぶ側（listener の handler）が持つ。
 */
export function requestWindowClose(): boolean {
  // (1) 確認の窓を出す者がいなければ止めない（root の error の fallback の間を含む）
  if (presenters === 0) return false;
  // (2) 集合が空なら止めない
  if (guards.size === 0) return false;
  // (3) 既に開いていれば止めるだけで何も変えない（2 回目の ×）
  if (dialogKind !== null) return true;
  // (4) 進行中が 1 つでもあれば進行中（複数あれば先に入った方）、無ければ未保存の窓を開く
  setDialogKind(getWindowCloseGuardKinds().find(isBusyKind) ?? "dirty");
  return true;
}

/** 確認の窓だけを閉じる（「編集を続ける」「待つ」）。集合は変えない */
export function closeWindowCloseDialog(): void {
  setDialogKind(null);
}

/** WindowCloseDialog が mount の間だけ「出す者」として登録する。戻り値で外す */
export function registerWindowCloseDialog(): () => void {
  presenters += 1;
  return () => {
    presenters -= 1;
  };
}

export function subscribeWindowCloseDialog(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function getWindowCloseDialogKind(): WindowCloseGuardKind | null {
  return dialogKind;
}
