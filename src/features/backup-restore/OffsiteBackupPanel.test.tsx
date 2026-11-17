// src/features/backup-restore/OffsiteBackupPanel.test.tsx
//
// UI-11b-D14: 「PC の外の控え」の card の表示（presentational）。
// 設計: docs/function-design/68-ui-backup-restore.md UI-11b-D14 / §68.4 / §68.11

import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import type { OffsiteBackupStatus } from "@/lib/bindings";
import { OffsiteBackupPanel, type OffsiteBackupPanelProps } from "./OffsiteBackupPanel";

function status(overrides: Partial<OffsiteBackupStatus> = {}): OffsiteBackupStatus {
  return {
    prepared: true,
    last_success_at: "2026-10-28 09:01:00",
    last_success_label: "控え 2",
    days_since_last_success: 0,
    stale: false,
    attached: [],
    last_failure_kind: null,
    local_backup_old: false,
    ...overrides,
  };
}

function renderPanel(overrides: Partial<OffsiteBackupPanelProps> = {}) {
  const props: OffsiteBackupPanelProps = {
    status: status(),
    isError: false,
    onRetry: vi.fn(),
    operationsDisabled: false,
    fileEntryDisabled: false,
    busy: null,
    message: null,
    onPrepare: vi.fn(),
    onReset: vi.fn(),
    onStop: vi.fn(),
    onChooseFile: vi.fn(),
    ...overrides,
  };
  render(<OffsiteBackupPanel {...props} />);
  return props;
}

describe("OffsiteBackupPanel (REQ-901 / UI-11b-D14)", () => {
  it("REQ-901 / UI-11b-D14: 用意の前は「PC の外の控えをやめる」が無く、2 つの button がある", () => {
    renderPanel({
      status: status({
        prepared: false,
        last_success_at: null,
        last_success_label: null,
        days_since_last_success: null,
      }),
    });
    expect(
      screen.getByText(
        "PC の外の控えは使っていません（任意。使わなくてもアプリは今までどおり動きます）。使うときは USB メモリを差して「この USB メモリを控えの保存先にする」を押してください。",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "この USB メモリを控えの保存先にする" }),
    ).toBeEnabled();
    expect(screen.getByRole("button", { name: "控えを選んで確かめる" })).toBeEnabled();
    expect(screen.queryByRole("button", { name: "PC の外の控えをやめる" })).not.toBeInTheDocument();
    expect(screen.queryByText("用意した USB メモリが差さっていません")).not.toBeInTheDocument();
  });

  it("REQ-901 / UI-11b-D14: 用意の後は状態の 1 行・差してある媒体の副情報・「PC の外の控えをやめる」", () => {
    renderPanel({
      status: status({ attached: [{ label: "控え 2", drive_root: "E:\\", newest_copy: null }] }),
    });
    expect(
      screen.getByText("最後に PC の外へ写した控え: 10月28日 09:01（控え 2、確かめ済み）"),
    ).toBeInTheDocument();
    expect(screen.getByText("差してある控え: 控え 2")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "PC の外の控えをやめる" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "記録を作り直す" })).not.toBeInTheDocument();
  });

  it("REQ-901 / UI-11b-D14: 差さっていない媒体と last_failure_kind の固定の文を副情報に出す", () => {
    renderPanel({ status: status({ last_failure_kind: "storage_full" }) });
    expect(screen.getByText("用意した USB メモリが差さっていません")).toBeInTheDocument();
    expect(screen.getByText("USB メモリの空きが足りません")).toBeInTheDocument();
  });

  it("REQ-901 / UI-11b-D14: 記録を読めないときは原因の文と「記録を作り直す」「PC の外の控えをやめる」", () => {
    renderPanel({
      status: status({
        last_success_at: null,
        last_success_label: null,
        days_since_last_success: null,
        stale: true,
        last_failure_kind: "state_unreadable",
      }),
    });
    const cause = screen.getByText(
      "PC の外の控えの記録を読めません。USB メモリを差して「記録を作り直す」を押すか、使わないなら「PC の外の控えをやめる」を押してください。",
    );
    expect(cause.closest('[data-slot="alert"]')).toHaveAttribute("data-variant", "warning");
    expect(screen.getByRole("button", { name: "記録を作り直す" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "PC の外の控えをやめる" })).toBeInTheDocument();
  });

  it("REQ-901 / UI-11b-D14: 記録を読めないときは差してある媒体の行を出さない（状態と照合できない）", () => {
    renderPanel({
      status: status({
        last_success_at: null,
        last_success_label: null,
        days_since_last_success: null,
        stale: true,
        last_failure_kind: "state_unreadable",
      }),
    });
    expect(screen.queryByText("用意した USB メモリが差さっていません")).not.toBeInTheDocument();
    expect(screen.queryByText(/^差してある控え/)).not.toBeInTheDocument();
    // last_failure_kind の固定の文は今までどおり出す
    expect(
      screen.getByText("PC の外の控えの記録を読めませんでした。「記録を作り直す」を押してください"),
    ).toBeInTheDocument();
  });

  it("REQ-901 / UI-11b-D14: 取得失敗は card を残して文と再読込を出す", () => {
    const props = renderPanel({ status: undefined, isError: true });
    expect(screen.getByText("PC の外の控え")).toBeInTheDocument();
    expect(screen.getByText("PC の外の控えの状態を取得できませんでした")).toBeInTheDocument();
    screen.getByRole("button", { name: "再読込" }).click();
    expect(props.onRetry).toHaveBeenCalledTimes(1);
    expect(screen.getByRole("button", { name: "控えを選んで確かめる" })).toBeInTheDocument();
  });

  it("REQ-901 / §68.7: 3 つの操作と file の入口の disabled を別に受ける", () => {
    renderPanel({
      status: status({ stale: true, last_failure_kind: "state_unreadable" }),
      operationsDisabled: true,
      fileEntryDisabled: false,
    });
    for (const name of [
      "この USB メモリを控えの保存先にする",
      "記録を作り直す",
      "PC の外の控えをやめる",
    ]) {
      expect(screen.getByRole("button", { name })).toBeDisabled();
    }
    expect(screen.getByRole("button", { name: "控えを選んで確かめる" })).toBeEnabled();
  });
});
