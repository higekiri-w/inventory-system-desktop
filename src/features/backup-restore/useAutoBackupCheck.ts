// src/features/backup-restore/useAutoBackupCheck.ts
//
// UI-11b-D13: 設定時刻の自動バックアップの確認（71 §71.8 のフロントエンドタイマー）。
// 共通レイアウト（UI-12 RootLayout）が 1 回 mount し、画面に依らず 60 秒ごとに確認する。
// 停止の 2 つの理由・世代番号・実行中の回は module scope に持つ（UI-11b-D11 と同じ in-memory の形。
// reload / アプリ再起動で初期値に戻る）。止まっているかは毎回 2 つの理由から導き、別の flag に写さない。

import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";
import { toast } from "sonner";

import { commands } from "@/lib/bindings";
import { unwrapResult } from "@/lib/invoke";
import { queryKeys } from "@/lib/query-keys";

const AUTO_BACKUP_CHECK_INTERVAL_MS = 60_000;

// 選択の理由（UI-11b-D15: 控えを選んで確かめ・詳細・最終確認を見ている間）。resume だけが外す。
let selectionHold = false;
// 復元の理由（実行中の復元の数と fatal）。復元の開始と結果だけが変える。fatal は再起動まで外れない。
let restoringCount = 0;
let fatalHold = false;
// 停止のたびに進む。呼ぶ時点と結果が届いた時点で違えば、その結果を捨てる。
let generation = 0;
// 実行中の回（hook が unmount されても、回が終わるまで残る）。
let inFlight: Promise<void> | null = null;

function isHeld(): boolean {
  return selectionHold || restoringCount > 0 || fatalHold;
}

/** 選択の理由を立てて世代番号を進める。停止の前に始まった確認の結果は捨てる。 */
export function suspendAutoBackupCheck(): void {
  selectionHold = true;
  generation += 1;
}

/**
 * UI-11b-D15: 控えを選んだ時点で呼ぶ。選択の理由を立て、実行中の回の完了を待つ。
 * 実行中の回が無ければすぐ解決する（hook が unmount されていても、回が残れば待つ）。
 */
export async function suspendAutoBackupCheckAndWait(): Promise<void> {
  suspendAutoBackupCheck();
  if (inFlight !== null) await inFlight;
}

/** 画面の unmount と復元の card の「閉じる」が呼ぶ。選択の理由だけを外す（復元の理由は残す）。 */
export function resumeAutoBackupCheck(): void {
  selectionHold = false;
}

/** 復元を始める前に呼ぶ。実行中の復元を数え、その画面の選択の理由を復元の理由へ移す。 */
export function holdAutoBackupCheckForRestore(): void {
  generation += 1;
  restoringCount += 1;
  selectionHold = false;
}

/** 復元の結果の後に 1 回だけ呼ぶ。復元の理由だけを変える（世代番号は戻さない）。 */
export function releaseAutoBackupCheckAfterRestore(fatal: boolean): void {
  restoringCount = Math.max(0, restoringCount - 1);
  if (fatal) fatalHold = true;
}

/** test だけが使う。module の停止の理由と実行中の回を初期値へ戻す（世代番号は戻さない）。 */
export function resetAutoBackupCheckForTests(): void {
  selectionHold = false;
  restoringCount = 0;
  fatalHold = false;
  inFlight = null;
}

export function useAutoBackupCheck(): void {
  const queryClient = useQueryClient();
  const failing = useRef(false);

  useEffect(() => {
    const interval = window.setInterval(() => {
      if (isHeld() || inFlight !== null) return;
      const startedGeneration = generation;
      const round = (async () => {
        try {
          const created = await unwrapResult(commands.checkAutoBackup(), {
            source: "commands",
            cmd: "check_auto_backup",
          });
          if (generation !== startedGeneration) return;
          failing.current = false;
          if (created) {
            void queryClient.invalidateQueries({ queryKey: queryKeys.backupRestore.list() });
            toast.success("自動バックアップを作成しました");
          }
        } catch {
          if (generation !== startedGeneration) return;
          // owner 決定 2026-09-25 = (a): 失敗の toast は連続失敗の最初の 1 回だけ
          if (!failing.current) {
            toast.error("自動バックアップ確認に失敗しました", { id: "backup-auto-check-error" });
          }
          failing.current = true;
        }
      })();
      inFlight = round;
      void round.finally(() => {
        // 捨てた結果でも guard を解く。test の reset の後に始まった回を消さない
        if (inFlight === round) inFlight = null;
      });
    }, AUTO_BACKUP_CHECK_INTERVAL_MS);
    return () => {
      window.clearInterval(interval);
    };
  }, [queryClient]);
}
