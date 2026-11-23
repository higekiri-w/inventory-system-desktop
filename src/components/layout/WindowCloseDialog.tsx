import { getCurrentWindow } from "@tauri-apps/api/window";
import { TriangleAlert } from "lucide-react";
import { useEffect, useSyncExternalStore } from "react";
import { toast } from "sonner";

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogMedia,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import {
  closeWindowCloseDialog,
  getWindowCloseDialogKind,
  registerWindowCloseDialog,
  subscribeWindowCloseDialog,
  type WindowCloseGuardKind,
} from "@/lib/window-close-guard";

/// UI-USW-D5 の 4: native の window の close（×・Alt+F4）を止めたときの確認の窓。RootLayout が 1 つ mount する。
/// 設計: docs/UI_TECH_STACK.md §6.11 UI-USW-D5（文言は owner の決定、repo 外の回答台帳 TD-689）

interface WindowCloseDialogCopy {
  title: string;
  description: string;
  /** 確認の窓だけを閉じる側（既定 focus） */
  stay: string;
  /** アプリの窓を閉じる側 */
  close: string;
}

const IMPORT_WAIT =
  "取込みが終わるまでお待ちください。終わると閉じられます。いま閉じると、取り込めたかどうかをこの画面で確かめられなくなります。";
const ROLLBACK_WAIT =
  "取消が終わるまでお待ちください。終わると閉じられます。いま閉じると、取り消せたかどうかをこの画面で確かめられなくなります。";
const Z004_RECORDS =
  "閉じたときは、次にアプリを開いてから「売上データ取込み」の「商品別CSV取込み（Z004）」タブの「取り込んだ記録を見る」で";
const DAILY_REPORT_RECORDS =
  "閉じたときは、次にアプリを開いてから「売上データ取込み」の「日報取込み」タブの「取り込んだ日報の記録を見る」で";
const IMPORT_CHECK = "確かめてください。記録に無ければ、もう一度取り込んでください。";
const ROLLBACK_CHECK =
  "、取消済みかを確かめてください。取消済みでなければ、もう一度取り消してください。";

const COPY: Record<WindowCloseGuardKind, WindowCloseDialogCopy> = {
  dirty: {
    title: "編集内容が保存されていません",
    description: "このまま閉じると、入力した内容は破棄されます。",
    stay: "編集を続ける",
    close: "破棄して閉じる",
  },
  "z004-import": {
    title: "取込みの途中です",
    description: IMPORT_WAIT + Z004_RECORDS + IMPORT_CHECK,
    stay: "待つ",
    close: "閉じる",
  },
  "z004-rollback": {
    title: "取消の途中です",
    description: ROLLBACK_WAIT + Z004_RECORDS + ROLLBACK_CHECK,
    stay: "待つ",
    close: "閉じる",
  },
  "daily-report-import": {
    title: "取込みの途中です",
    description: IMPORT_WAIT + DAILY_REPORT_RECORDS + IMPORT_CHECK,
    stay: "待つ",
    close: "閉じる",
  },
  "daily-report-rollback": {
    title: "取消の途中です",
    description: ROLLBACK_WAIT + DAILY_REPORT_RECORDS + ROLLBACK_CHECK,
    stay: "待つ",
    close: "閉じる",
  },
  // 55 UI-07-D17: 一括は日報ごとに確定するので、閉じた後は読み直して押し直す（取込みの完了を言い切らない。TD-839）
  "sd-bulk-import": {
    title: "取込みの途中です",
    description:
      "取込みが終わるまでお待ちください。終わると閉じられます。いま閉じると、どこまで取り込めたかをこの画面で確かめられなくなります。日報は 1 つずつ確定するので、一部だけが入っていることがあります。閉じたときは、次にアプリを開いてから「売上データ取込み」の「日報取込み」タブで同じ SD を読み直し、「まとめて取り込む」が押せればもう一度押してください。取り込み済みの日報と控えは二重に入りません。",
    stay: "待つ",
    close: "閉じる",
  },
  "ej-import": {
    title: "取込みの途中です",
    description:
      "取込みが終わるまでお待ちください。終わると閉じられます。いま閉じると、取り込めたかどうかをこの画面で確かめられなくなります。閉じたときは、次にアプリを開いてから「売上データ取込み」の「日報取込み」タブで同じ SD を読み直し、「まとめて取り込む」が押せれば押してください。結果に欠けか読めないファイルの文が残っていれば、「EJ のファイルを選んで取り込む」で同じファイルを選び直してください。取り込み済みの控えは二重に入りません。",
    stay: "待つ",
    close: "閉じる",
  },
  // 65 §65.5b: 入出庫の記録の取消。取消の応答を待つ間にも、「状態をもう一度確かめる」の読取りを待つ間
  // （取消済みと確かめた後のこともある）にも出るので、取消が途中とも終わったとも言わず、
  // 閉じた後に記録を開いて状態を確かめることだけを言う（TD-839）
  "inventory-record-cancel": {
    title: "この画面を閉じますか",
    description:
      "閉じた後は、次にアプリを開いてから「入出庫履歴」でこの記録を開き" + ROLLBACK_CHECK,
    stay: "待つ",
    close: "閉じる",
  },
};

/// 「破棄して閉じる」「閉じる」: 止めた後に閉じるのは destroy（close() は CloseRequested をもう一度出す）。
/// 拒まれたら確認の窓を閉じ、その場で直し方を出す（O5）。Tauri の外の同期の例外も同じ扱い。
async function destroyWindow(): Promise<void> {
  try {
    await getCurrentWindow().destroy();
  } catch {
    closeWindowCloseDialog();
    toast.error("閉じられませんでした。もう一度 × を押してください。");
  }
}

export function WindowCloseDialog() {
  const kind = useSyncExternalStore(subscribeWindowCloseDialog, getWindowCloseDialogKind);
  // mount の間だけ「出す者」になる。いない間の × は止めない（UI-USW-D5 の 3 の (1)）
  useEffect(() => registerWindowCloseDialog(), []);

  if (kind === null) return null;
  const copy = COPY[kind];
  return (
    // DSR-20 dismiss 本則: Esc と Cancel は onOpenChange 経由で同じ処理（確認の窓だけを閉じる）へ通す。
    // 外側クリックは Radix AlertDialog primitive 既定で非 dismiss（handler 非提供）。
    <AlertDialog
      open
      onOpenChange={(open) => {
        if (!open) closeWindowCloseDialog();
      }}
    >
      <AlertDialogContent
        onEscapeKeyDown={(event) => {
          // DSR-20 硬化条件 (b): 未保存内容を破棄するか継続するかを明示選択させる（UI-USW-D2 と同じ）。
          // 進行中の窓の Esc は「待つ」と同じなので止めない。
          if (kind === "dirty") event.preventDefault();
        }}
      >
        <AlertDialogHeader>
          <AlertDialogMedia>
            <TriangleAlert aria-hidden="true" />
          </AlertDialogMedia>
          <AlertDialogTitle>{copy.title}</AlertDialogTitle>
          <AlertDialogDescription>{copy.description}</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{copy.stay}</AlertDialogCancel>
          <AlertDialogAction
            variant="destructive"
            onClick={() => {
              void destroyWindow();
            }}
          >
            {copy.close}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
