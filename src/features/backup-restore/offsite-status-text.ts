// src/features/backup-restore/offsite-status-text.ts
//
// UI-11b-D14（バックアップ画面の「PC の外の控え」の card）と UI-00-D12（ホームの知らせ）が出す
// 状態の文と原因の文を、status から選ぶ 1 か所。日数は計算せず status の値を出す（古さの判定は MNT-01）。
// 設計: docs/function-design/68-ui-backup-restore.md UI-11b-D14 / 53-ui-home.md UI-00-D12

import type { OffsiteBackupStatus, OffsiteFailureKind } from "@/lib/bindings";

/** `YYYY-MM-DD HH:MM:SS` を `7月3日 21:00` の和式日時にする（解析できなければそのまま） */
export function formatBackupDate(value: string): string {
  const date = new Date(value.includes("T") ? value : value.replace(" ", "T"));
  if (Number.isNaN(date.getTime())) return value;
  const month = String(date.getMonth() + 1);
  const day = String(date.getDate());
  const hours = String(date.getHours()).padStart(2, "0");
  const minutes = String(date.getMinutes()).padStart(2, "0");
  return `${month}月${day}日 ${hours}:${minutes}`;
}

/** card はそのまま、ホームは button の名前の前に「バックアップ画面の」を付ける */
export type OffsiteTextPlace = "card" | "home";

// 43 §43.8.2〜§43.8.7 のエラーの写像の表の固定の文（card の副情報と原因の文の (4)）
const SAME_FOLDER = (place: OffsiteTextPlace) =>
  `PC の中の保存先が、USB メモリの InventoryBackup フォルダと同じです。${button(place, "保存先を選ぶ")}で PC の中の別のフォルダを選び直してください`;

const FAILURE_TEXT: Record<OffsiteFailureKind, string> = {
  register_sd: "PC の外の控えを写せませんでした",
  not_removable: "USB メモリを選んでください（この場所は PC の外の控えに使えません）",
  storage_full: "USB メモリの空きが足りません",
  verify_mismatch: "PC の外の控えを写せませんでした",
  io: "PC の外の控えを写せませんでした",
  state_unreadable: "PC の外の控えの記録を読めませんでした。「記録を作り直す」を押してください",
  source_unverified: "PC の中の最新のバックアップを確かめられなかったため、PC の外へ写していません",
  backup_dir_is_medium_folder: SAME_FOLDER("card"),
};

function button(place: OffsiteTextPlace, name: string): string {
  return place === "home" ? `バックアップ画面の「${name}」` : `「${name}」`;
}

/** `last_failure_kind` の固定の文（card の副情報） */
export function offsiteFailureText(kind: OffsiteFailureKind): string {
  return FAILURE_TEXT[kind];
}

/** 状態の 1 行（UI-11b-D14 の主情報） */
export function offsiteStateLine(status: OffsiteBackupStatus): string {
  if (!status.prepared) {
    return "PC の外の控えは使っていません（任意。使わなくてもアプリは今までどおり動きます）。使うときは USB メモリを差して「この USB メモリを控えの保存先にする」を押してください。";
  }
  if (status.last_success_at === null) return "まだ PC の外へ写せていません";
  const label = status.last_success_label ? `${status.last_success_label}、` : "";
  return `最後に PC の外へ写した控え: ${formatBackupDate(status.last_success_at)}（${label}確かめ済み）`;
}

/** 差してある媒体の副情報 */
export function offsiteAttachedText(status: OffsiteBackupStatus): string {
  if (status.attached.length === 0) return "用意した USB メモリが差さっていません";
  return `差してある控え: ${status.attached.map((medium) => medium.label).join("、")}`;
}

/** `stale` のときだけ、原因の文を (1)〜(5) の順で 1 つ選ぶ（UI-11b-D14、UI-00-D12） */
export function offsiteCauseText(
  status: OffsiteBackupStatus,
  place: OffsiteTextPlace,
): string | null {
  if (!status.stale) return null;
  const days = status.last_success_at === null ? null : status.days_since_last_success;
  const notCopied =
    days === null
      ? "PC の外の控えがまだ写せていません。"
      : `PC の外の控えが ${String(days)} 日写せていません。`;
  const kind = status.last_failure_kind;
  // (1) 記録を読めない
  if (kind === "state_unreadable") {
    return `PC の外の控えの記録を読めません。USB メモリを差して${button(place, "記録を作り直す")}を押すか、使わないなら${button(place, "PC の外の控えをやめる")}を押してください。`;
  }
  // (2) PC の中の backup が古い（USB を差し直しても直らない）
  if (status.local_backup_old) {
    const head =
      days === null
        ? "PC の外の控えがまだ写せていません。PC の中に新しいバックアップがありません。"
        : `PC の外の控えが ${String(days)} 日前のままです。PC の中の新しいバックアップが作られていません。`;
    return `${head}${button(place, "今すぐバックアップを作成")}を押すか、自動バックアップと保存先の設定を確かめてください。`;
  }
  // (3) 写す元を確かめられない
  if (kind === "source_unverified") {
    return `PC の中の最新のバックアップを確かめられないため、PC の外へ写していません。${button(place, "今すぐバックアップを作成")}を押してください。`;
  }
  // (4) USB の差し直しでは直らない失敗: 43 の固定の文を続ける
  if (kind === "backup_dir_is_medium_folder") return `${notCopied}${SAME_FOLDER(place)}`;
  if (kind === "storage_full" || kind === "verify_mismatch") {
    return `${notCopied}${FAILURE_TEXT[kind]}`;
  }
  // (5) それ以外: 用意済みの媒体が見えているかで分ける（失敗した媒体と同じとは限らないので札を名指さない）
  if (status.attached.length === 0) {
    return `${notCopied}USB メモリが差してあるか確かめてください。`;
  }
  return `${notCopied}USB メモリは差してありますが、写せません。いったん抜いて差し直し、直らなければ別の USB メモリを${button(place, "この USB メモリを控えの保存先にする")}で用意してください。`;
}
