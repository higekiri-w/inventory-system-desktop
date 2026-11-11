// src/features/backup-restore/offsite-status-text.test.ts
//
// UI-11b-D14・UI-00-D12: PC の外の控えの状態の文と原因の文（設計の文の完全一致）。
// 設計: docs/function-design/68-ui-backup-restore.md UI-11b-D14 / 53-ui-home.md UI-00-D12
// mock の日数・札は設計の例と違う値（5 日・控え 2）を使い、表示が status から来たことを確かめる。

import { describe, expect, it } from "vitest";

import type { OffsiteBackupStatus, OffsiteFailureKind } from "@/lib/bindings";
import {
  offsiteAttachedText,
  offsiteCauseText,
  offsiteFailureText,
  offsiteStateLine,
} from "./offsite-status-text";

function status(overrides: Partial<OffsiteBackupStatus> = {}): OffsiteBackupStatus {
  return {
    prepared: true,
    last_success_at: "2026-10-28 09:01:00",
    last_success_label: "控え 2",
    // 日付（10月28日）と食い違う日数: UI は計算せず status の値を出す
    days_since_last_success: 5,
    stale: true,
    attached: [],
    last_failure_kind: null,
    local_backup_old: false,
    ...overrides,
  };
}

const never = { last_success_at: null, last_success_label: null, days_since_last_success: null };
const attached = [{ label: "控え 2", drive_root: "E:\\", newest_copy: null }];

describe("offsiteStateLine (REQ-901 / UI-11b-D14)", () => {
  it("REQ-901 / UI-11b-D14: 用意の前・写せた後・まだ写せていない", () => {
    expect(offsiteStateLine(status({ prepared: false, stale: false, ...never }))).toBe(
      "PC の外の控えは使っていません（任意。使わなくてもアプリは今までどおり動きます）。使うときは USB メモリを差して「この USB メモリを控えの保存先にする」を押してください。",
    );
    expect(offsiteStateLine(status({ stale: false }))).toBe(
      "最後に PC の外へ写した控え: 10月28日 09:01（控え 2、確かめ済み）",
    );
    expect(offsiteStateLine(status(never))).toBe("まだ PC の外へ写せていません");
  });
});

describe("offsiteCauseText (REQ-901 / UI-11b-D14・UI-00-D12)", () => {
  it("REQ-901 / UI-11b-D14: stale でなければ原因の文を出さない", () => {
    expect(offsiteCauseText(status({ stale: false, last_failure_kind: "io" }), "card")).toBeNull();
    expect(offsiteCauseText(status({ prepared: false, stale: false }), "home")).toBeNull();
  });

  it("REQ-901 / UI-11b-D14: (1) 記録を読めない", () => {
    const unreadable = status({ ...never, last_failure_kind: "state_unreadable" });
    expect(offsiteCauseText(unreadable, "card")).toBe(
      "PC の外の控えの記録を読めません。USB メモリを差して「記録を作り直す」を押すか、使わないなら「PC の外の控えをやめる」を押してください。",
    );
    expect(offsiteCauseText(unreadable, "home")).toBe(
      "PC の外の控えの記録を読めません。USB メモリを差してバックアップ画面の「記録を作り直す」を押すか、使わないならバックアップ画面の「PC の外の控えをやめる」を押してください。",
    );
  });

  it("REQ-901 / UI-11b-D14: (1) は (2) より先（記録を読めず、PC の中の backup も古い）", () => {
    // 読めない記録でも MNT は local_backup_old を返す（71 §71.11.5）ので、この組は実際に起きる
    const both = status({
      ...never,
      last_failure_kind: "state_unreadable",
      local_backup_old: true,
    });
    expect(offsiteCauseText(both, "card")).toBe(
      "PC の外の控えの記録を読めません。USB メモリを差して「記録を作り直す」を押すか、使わないなら「PC の外の控えをやめる」を押してください。",
    );
    expect(offsiteCauseText(both, "home")).toBe(
      "PC の外の控えの記録を読めません。USB メモリを差してバックアップ画面の「記録を作り直す」を押すか、使わないならバックアップ画面の「PC の外の控えをやめる」を押してください。",
    );
  });

  it("REQ-901 / UI-11b-D14: (2) PC の中の backup が古い（(3) より先）", () => {
    const old = status({ local_backup_old: true, last_failure_kind: "source_unverified" });
    expect(offsiteCauseText(old, "card")).toBe(
      "PC の外の控えが 5 日前のままです。PC の中の新しいバックアップが作られていません。「今すぐバックアップを作成」を押すか、自動バックアップと保存先の設定を確かめてください。",
    );
    expect(offsiteCauseText(status({ ...never, local_backup_old: true }), "home")).toBe(
      "PC の外の控えがまだ写せていません。PC の中に新しいバックアップがありません。バックアップ画面の「今すぐバックアップを作成」を押すか、自動バックアップと保存先の設定を確かめてください。",
    );
  });

  it("REQ-901 / UI-11b-D14: (3) 写す元を確かめられない", () => {
    const unverified = status({ last_failure_kind: "source_unverified" });
    expect(offsiteCauseText(unverified, "card")).toBe(
      "PC の中の最新のバックアップを確かめられないため、PC の外へ写していません。「今すぐバックアップを作成」を押してください。",
    );
    expect(offsiteCauseText(unverified, "home")).toBe(
      "PC の中の最新のバックアップを確かめられないため、PC の外へ写していません。バックアップ画面の「今すぐバックアップを作成」を押してください。",
    );
  });

  it("REQ-901 / UI-11b-D14: (4) 空き・読み戻し・保存先が同じ folder（USB の確認の文を出さない）", () => {
    expect(offsiteCauseText(status({ last_failure_kind: "storage_full", attached }), "card")).toBe(
      "PC の外の控えが 5 日写せていません。USB メモリの空きが足りません",
    );
    expect(
      offsiteCauseText(status({ ...never, last_failure_kind: "verify_mismatch" }), "card"),
    ).toBe("PC の外の控えがまだ写せていません。PC の外の控えを写せませんでした");
    const sameFolder = status({ last_failure_kind: "backup_dir_is_medium_folder", attached });
    expect(offsiteCauseText(sameFolder, "card")).toBe(
      "PC の外の控えが 5 日写せていません。PC の中の保存先が、USB メモリの InventoryBackup フォルダと同じです。「保存先を選ぶ」で PC の中の別のフォルダを選び直してください",
    );
    expect(offsiteCauseText(sameFolder, "home")).toBe(
      "PC の外の控えが 5 日写せていません。PC の中の保存先が、USB メモリの InventoryBackup フォルダと同じです。バックアップ画面の「保存先を選ぶ」で PC の中の別のフォルダを選び直してください",
    );
  });

  it("REQ-901 / UI-11b-D14: (5) 差してあるかで 2 つの文（札を名指さない）", () => {
    expect(offsiteCauseText(status({ last_failure_kind: "io" }), "card")).toBe(
      "PC の外の控えが 5 日写せていません。USB メモリが差してあるか確かめてください。",
    );
    expect(offsiteCauseText(status(never), "home")).toBe(
      "PC の外の控えがまだ写せていません。USB メモリが差してあるか確かめてください。",
    );
    const inserted = status({ last_failure_kind: "io", attached });
    expect(offsiteCauseText(inserted, "card")).toBe(
      "PC の外の控えが 5 日写せていません。USB メモリは差してありますが、写せません。いったん抜いて差し直し、直らなければ別の USB メモリを「この USB メモリを控えの保存先にする」で用意してください。",
    );
    expect(offsiteCauseText(status({ ...never, attached }), "home")).toBe(
      "PC の外の控えがまだ写せていません。USB メモリは差してありますが、写せません。いったん抜いて差し直し、直らなければ別の USB メモリをバックアップ画面の「この USB メモリを控えの保存先にする」で用意してください。",
    );
    expect(offsiteCauseText(inserted, "card")).not.toContain("控え 2");
  });
});

describe("副情報 (REQ-901 / UI-11b-D14)", () => {
  it("REQ-901 / UI-11b-D14: 差してある媒体と、last_failure_kind の固定の文", () => {
    expect(offsiteAttachedText(status({ attached }))).toBe("差してある控え: 控え 2");
    expect(offsiteAttachedText(status())).toBe("用意した USB メモリが差さっていません");
    // 記録を読めないときは状態と照合できないので、差してある・差さっていないのどちらも言わない
    expect(offsiteAttachedText(status({ last_failure_kind: "state_unreadable" }))).toBeNull();
    const texts: [OffsiteFailureKind, string][] = [
      ["storage_full", "USB メモリの空きが足りません"],
      ["io", "PC の外の控えを写せませんでした"],
      ["verify_mismatch", "PC の外の控えを写せませんでした"],
      [
        "state_unreadable",
        "PC の外の控えの記録を読めませんでした。「記録を作り直す」を押してください",
      ],
      [
        "source_unverified",
        "PC の中の最新のバックアップを確かめられなかったため、PC の外へ写していません",
      ],
      [
        "backup_dir_is_medium_folder",
        "PC の中の保存先が、USB メモリの InventoryBackup フォルダと同じです。「保存先を選ぶ」で PC の中の別のフォルダを選び直してください",
      ],
    ];
    for (const [kind, text] of texts) {
      expect(offsiteFailureText(kind)).toBe(text);
    }
  });
});
