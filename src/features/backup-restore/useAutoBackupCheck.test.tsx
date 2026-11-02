// src/features/backup-restore/useAutoBackupCheck.test.tsx
//
// UI-11b-D13: 共通レイアウトが mount する自動バックアップの確認 hook。
// 設計: docs/function-design/68-ui-backup-restore.md UI-11b-D13 / 71 §71.8

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook } from "@testing-library/react";
import { StrictMode, type ReactNode } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { toast } from "sonner";
import { commands } from "@/lib/bindings";
import { queryKeys } from "@/lib/query-keys";
import {
  holdAutoBackupCheckForRestore,
  releaseAutoBackupCheckAfterRestore,
  resetAutoBackupCheckForTests,
  resumeAutoBackupCheck,
  suspendAutoBackupCheck,
  suspendAutoBackupCheckAndWait,
  useAutoBackupCheck,
} from "./useAutoBackupCheck";

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

vi.mock("@/lib/bindings", () => ({
  commands: { checkAutoBackup: vi.fn(), checkOffsiteBackup: vi.fn() },
}));

type CheckResult = Awaited<ReturnType<typeof commands.checkAutoBackup>>;
type OffsiteResult = Awaited<ReturnType<typeof commands.checkOffsiteBackup>>;
type OffsiteKind =
  | { kind: "not_prepared" }
  | { kind: "no_local_backup" }
  | { kind: "medium_missing" }
  | { kind: "up_to_date" }
  | { kind: "copied"; file_name: string; labels: string[] };

const mockCheckAutoBackup = vi.mocked(commands.checkAutoBackup);
const mockCheckOffsite = vi.mocked(commands.checkOffsiteBackup);
const mockToastSuccess = vi.mocked(toast.success);
const mockToastError = vi.mocked(toast.error);

function ok(data: boolean): CheckResult {
  return { status: "ok", data };
}

function failure(): CheckResult {
  return {
    status: "error",
    error: {
      kind: "internal",
      message: "保存先に書けません",
      field: null,
      error_id: null,
      stocktake_recovery: null,
    },
  };
}

function offsiteOk(data: OffsiteKind = { kind: "not_prepared" }): OffsiteResult {
  return { status: "ok", data };
}

function offsiteFailure(): OffsiteResult {
  return {
    status: "error",
    error: {
      kind: "internal",
      message: "PC の外の控えを写せませんでした",
      field: null,
      error_id: "e-1",
      stocktake_recovery: null,
    },
  };
}

function invalidationsOf(spy: { mock: { calls: unknown[][] } }, key: readonly string[]): number {
  return spy.mock.calls.filter(
    ([filters]) =>
      JSON.stringify((filters as { queryKey?: unknown } | undefined)?.queryKey) ===
      JSON.stringify(key),
  ).length;
}

function renderCheck({ strict = false } = {}) {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
  const wrapper = ({ children }: { children: ReactNode }) => {
    const tree = <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
    return strict ? <StrictMode>{tree}</StrictMode> : tree;
  };
  return {
    invalidateSpy,
    ...renderHook(
      () => {
        useAutoBackupCheck();
      },
      { wrapper },
    ),
  };
}

beforeEach(() => {
  vi.useFakeTimers();
  resetAutoBackupCheckForTests();
  mockCheckAutoBackup.mockReset();
  mockCheckAutoBackup.mockResolvedValue(ok(false));
  mockCheckOffsite.mockReset();
  mockCheckOffsite.mockResolvedValue(offsiteOk());
  mockToastSuccess.mockReset();
  mockToastError.mockReset();
});

afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
});

describe("useAutoBackupCheck (REQ-901 / UI-11b-D13)", () => {
  it("REQ-901 / UI-11b-D13: 60 秒ごとに確認し、mount の瞬間には呼ばない", async () => {
    renderCheck();
    expect(mockCheckAutoBackup).not.toHaveBeenCalled();

    await vi.advanceTimersByTimeAsync(60_000);
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(1);

    await vi.advanceTimersByTimeAsync(60_000);
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(2);
  });

  it("REQ-901 / UI-11b-D13: `true` で一覧を invalidate し成功 toast、`false` では何もしない", async () => {
    mockCheckAutoBackup.mockResolvedValueOnce(ok(true));
    const { invalidateSpy } = renderCheck();

    await vi.advanceTimersByTimeAsync(60_000);
    expect(invalidationsOf(invalidateSpy, queryKeys.backupRestore.list())).toBe(1);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: queryKeys.backupRestore.list() });
    expect(mockToastSuccess).toHaveBeenCalledTimes(1);
    expect(mockToastSuccess).toHaveBeenCalledWith("自動バックアップを作成しました");

    // 次の回は false
    await vi.advanceTimersByTimeAsync(60_000);
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(2);
    expect(invalidationsOf(invalidateSpy, queryKeys.backupRestore.list())).toBe(1);
    expect(mockToastSuccess).toHaveBeenCalledTimes(1);
    expect(mockToastError).not.toHaveBeenCalled();
  });

  it("REQ-901 / UI-11b-D13: 失敗の toast は連続失敗の最初の 1 回だけ", async () => {
    mockCheckAutoBackup
      .mockResolvedValueOnce(failure())
      .mockResolvedValueOnce(failure())
      .mockResolvedValueOnce(failure())
      .mockResolvedValueOnce(ok(false))
      .mockResolvedValueOnce(failure());
    renderCheck();

    await vi.advanceTimersByTimeAsync(180_000);
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(3);
    expect(mockToastError).toHaveBeenCalledTimes(1);
    expect(mockToastError).toHaveBeenCalledWith("自動バックアップ確認に失敗しました", {
      id: "backup-auto-check-error",
    });

    await vi.advanceTimersByTimeAsync(60_000); // false で連続失敗を解く
    expect(mockToastError).toHaveBeenCalledTimes(1);

    await vi.advanceTimersByTimeAsync(60_000); // 解いた後の最初の失敗
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(5);
    expect(mockToastError).toHaveBeenCalledTimes(2);
  });

  it("REQ-901 / UI-11b-D13: `suspendAutoBackupCheck()` の後は呼ばず、`resumeAutoBackupCheck()` の後は再び呼ぶ", async () => {
    renderCheck();

    await vi.advanceTimersByTimeAsync(60_000);
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(1);

    suspendAutoBackupCheck();
    await vi.advanceTimersByTimeAsync(120_000);
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(1);

    resumeAutoBackupCheck();
    await vi.advanceTimersByTimeAsync(60_000);
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(2);
  });

  it("REQ-901 / UI-11b-D13: unmount で止まり、StrictMode の二重 mount でも 1 本", async () => {
    const { unmount } = renderCheck({ strict: true });

    await vi.advanceTimersByTimeAsync(60_000);
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(1);

    unmount();
    await vi.advanceTimersByTimeAsync(60_000);
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(1);
  });

  it("REQ-901 / UI-11b-D13: 前の確認が解決するまで次を呼ばない", async () => {
    let settle: (value: CheckResult) => void = () => undefined;
    mockCheckAutoBackup.mockReturnValueOnce(
      new Promise<CheckResult>((resolve) => {
        settle = resolve;
      }),
    );
    renderCheck();

    try {
      await vi.advanceTimersByTimeAsync(60_000);
      expect(mockCheckAutoBackup).toHaveBeenCalledTimes(1);

      await vi.advanceTimersByTimeAsync(120_000);
      expect(mockCheckAutoBackup).toHaveBeenCalledTimes(1);

      settle(ok(false));
      await vi.advanceTimersByTimeAsync(60_000);
      expect(mockCheckAutoBackup).toHaveBeenCalledTimes(2);
    } finally {
      settle(ok(false));
    }
  });
});

function pendingCheck() {
  let settle: (value: CheckResult) => void = () => undefined;
  const promise = new Promise<CheckResult>((resolve) => {
    settle = resolve;
  });
  mockCheckAutoBackup.mockReturnValueOnce(promise);
  return {
    settle: (value: CheckResult) => {
      settle(value);
    },
  };
}

function trackSettled(promise: Promise<void>) {
  const state = { settled: false };
  void promise.then(() => {
    state.settled = true;
  });
  return state;
}

describe("停止の待ちと 2 つの停止の理由 (REQ-901 / UI-11b-D13・D15)", () => {
  it("REQ-901 / UI-11b-D15: `suspendAutoBackupCheckAndWait()` は実行中の回の解決まで待ち、待つ間に届いた `true` を捨てる", async () => {
    const check = pendingCheck();
    const { invalidateSpy } = renderCheck();
    try {
      await vi.advanceTimersByTimeAsync(60_000);
      expect(mockCheckAutoBackup).toHaveBeenCalledTimes(1);

      const waiting = trackSettled(suspendAutoBackupCheckAndWait());
      await vi.advanceTimersByTimeAsync(0);
      expect(waiting.settled).toBe(false);

      check.settle(ok(true));
      await vi.advanceTimersByTimeAsync(0);
      expect(waiting.settled).toBe(true);
      expect(mockToastSuccess).not.toHaveBeenCalled();
      expect(invalidationsOf(invalidateSpy, queryKeys.backupRestore.list())).toBe(0);
    } finally {
      check.settle(ok(false));
    }
  });

  it("REQ-901 / UI-11b-D15: 実行中の回が無いとき・hook が一度も mount されていないときはすぐ解決する", async () => {
    const unmounted = trackSettled(suspendAutoBackupCheckAndWait());
    await vi.advanceTimersByTimeAsync(0);
    expect(unmounted.settled).toBe(true);

    resetAutoBackupCheckForTests();
    renderCheck();
    const idle = trackSettled(suspendAutoBackupCheckAndWait());
    await vi.advanceTimersByTimeAsync(0);
    expect(idle.settled).toBe(true);
    expect(mockCheckAutoBackup).not.toHaveBeenCalled();
  });

  it("REQ-901 / UI-11b-D15: 確かめの開始の後に hook を unmount しても、実行中の回が終わるまで解決しない", async () => {
    const check = pendingCheck();
    const { unmount } = renderCheck();
    try {
      await vi.advanceTimersByTimeAsync(60_000);
      expect(mockCheckAutoBackup).toHaveBeenCalledTimes(1);
      unmount();

      const waiting = trackSettled(suspendAutoBackupCheckAndWait());
      await vi.advanceTimersByTimeAsync(60_000);
      expect(waiting.settled).toBe(false);

      check.settle(ok(false));
      await vi.advanceTimersByTimeAsync(0);
      expect(waiting.settled).toBe(true);
    } finally {
      check.settle(ok(false));
    }
  });

  it("REQ-901 / UI-11b-D13: 停止 → 再開の後に届いた停止の前の `true` は捨て、guard は解けて次の tick で呼ぶ", async () => {
    const check = pendingCheck();
    const { invalidateSpy } = renderCheck();
    try {
      await vi.advanceTimersByTimeAsync(60_000);
      suspendAutoBackupCheck();
      resumeAutoBackupCheck();

      check.settle(ok(true));
      await vi.advanceTimersByTimeAsync(0);
      expect(mockToastSuccess).not.toHaveBeenCalled();
      expect(invalidationsOf(invalidateSpy, queryKeys.backupRestore.list())).toBe(0);

      await vi.advanceTimersByTimeAsync(60_000);
      expect(mockCheckAutoBackup).toHaveBeenCalledTimes(2);
    } finally {
      check.settle(ok(false));
    }
  });

  it("REQ-901 / UI-11b-D13: 停止の前に始まった確認が停止中に失敗しても toast・連続失敗の状態を変えず、再開後の最初の失敗は通知する", async () => {
    const check = pendingCheck();
    renderCheck();
    try {
      await vi.advanceTimersByTimeAsync(60_000);
      expect(mockCheckAutoBackup).toHaveBeenCalledTimes(1);
      suspendAutoBackupCheck();

      check.settle(failure());
      await vi.advanceTimersByTimeAsync(0);
      expect(mockToastError).not.toHaveBeenCalled();

      resumeAutoBackupCheck();
      mockCheckAutoBackup.mockResolvedValueOnce(failure());
      await vi.advanceTimersByTimeAsync(60_000);
      expect(mockCheckAutoBackup).toHaveBeenCalledTimes(2);
      // 古い失敗が連続失敗の状態を立てていれば、ここで通知されない
      expect(mockToastError).toHaveBeenCalledTimes(1);
      expect(mockToastError).toHaveBeenCalledWith("自動バックアップ確認に失敗しました", {
        id: "backup-auto-check-error",
      });
    } finally {
      check.settle(ok(false));
    }
  });

  it("REQ-901 / UI-11b-D13: 復元の理由は resume で外れず release(false) で外れ、fatal は何でも外れない", async () => {
    renderCheck();

    holdAutoBackupCheckForRestore();
    resumeAutoBackupCheck();
    await vi.advanceTimersByTimeAsync(120_000);
    expect(mockCheckAutoBackup).not.toHaveBeenCalled();

    releaseAutoBackupCheckAfterRestore(false);
    await vi.advanceTimersByTimeAsync(60_000);
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(1);

    holdAutoBackupCheckForRestore();
    releaseAutoBackupCheckAfterRestore(true);
    resumeAutoBackupCheck();
    await vi.advanceTimersByTimeAsync(120_000);
    holdAutoBackupCheckForRestore();
    releaseAutoBackupCheckAfterRestore(false);
    await vi.advanceTimersByTimeAsync(120_000);
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(1);
  });

  it("REQ-901 / UI-11b-D13: hold は選択の理由を復元の理由へ移す", async () => {
    renderCheck();

    suspendAutoBackupCheck();
    holdAutoBackupCheckForRestore();
    releaseAutoBackupCheckAfterRestore(false);
    await vi.advanceTimersByTimeAsync(60_000);
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(1);
  });

  it("REQ-901 / UI-11b-D13: 前の復元の結果は再訪した画面の選択の理由を外さない", async () => {
    renderCheck();

    holdAutoBackupCheckForRestore();
    resumeAutoBackupCheck(); // 復元の実行中に画面を離れる
    suspendAutoBackupCheck(); // 再訪した画面で別の控えを選ぶ
    releaseAutoBackupCheckAfterRestore(false); // 前の復元が fatal でない結果で終わる
    await vi.advanceTimersByTimeAsync(120_000);
    expect(mockCheckAutoBackup).not.toHaveBeenCalled();

    resumeAutoBackupCheck();
    await vi.advanceTimersByTimeAsync(60_000);
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(1);
  });

  it("REQ-901 / UI-11b-D13: 実行中の復元を数え、先に終わった復元の結果が後の復元の停止を外さない", async () => {
    renderCheck();

    holdAutoBackupCheckForRestore();
    holdAutoBackupCheckForRestore();
    releaseAutoBackupCheckAfterRestore(false);
    await vi.advanceTimersByTimeAsync(120_000);
    expect(mockCheckAutoBackup).not.toHaveBeenCalled();

    releaseAutoBackupCheckAfterRestore(false);
    await vi.advanceTimersByTimeAsync(60_000);
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(1);
  });
});

function pendingOffsite() {
  let settle: (value: OffsiteResult) => void = () => undefined;
  const promise = new Promise<OffsiteResult>((resolve) => {
    settle = resolve;
  });
  mockCheckOffsite.mockReturnValueOnce(promise);
  return {
    settle: (value: OffsiteResult) => {
      settle(value);
    },
  };
}

const offsiteKey = queryKeys.backupRestore.offsiteStatus();

describe("PC の外の控えの確認 (REQ-901 / UI-11b-D16)", () => {
  it("REQ-901 / UI-11b-D16: mount で 1 回だけ checkOffsiteBackup を呼び（checkAutoBackup は呼ばない）、StrictMode でも 1 回", async () => {
    renderCheck({ strict: true });
    await vi.advanceTimersByTimeAsync(0);
    expect(mockCheckOffsite).toHaveBeenCalledTimes(1);
    expect(mockCheckAutoBackup).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(30_000);
    expect(mockCheckOffsite).toHaveBeenCalledTimes(1);
  });

  it("REQ-901 / UI-11b-D16: 同じ回の中で checkAutoBackup の結果の後（成功・失敗とも）に呼ぶ", async () => {
    renderCheck();
    await vi.advanceTimersByTimeAsync(0);
    mockCheckOffsite.mockClear();

    const auto = pendingCheck();
    await vi.advanceTimersByTimeAsync(60_000);
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(1);
    expect(mockCheckOffsite).not.toHaveBeenCalled();
    auto.settle(ok(true));
    await vi.advanceTimersByTimeAsync(0);
    expect(mockCheckOffsite).toHaveBeenCalledTimes(1);

    const failed = pendingCheck();
    await vi.advanceTimersByTimeAsync(60_000);
    expect(mockCheckOffsite).toHaveBeenCalledTimes(1);
    failed.settle(failure());
    await vi.advanceTimersByTimeAsync(0);
    expect(mockCheckOffsite).toHaveBeenCalledTimes(2);
  });

  it("REQ-901 / UI-11b-D16: 停止中は mount も interval も呼ばない", async () => {
    suspendAutoBackupCheck();
    renderCheck();
    await vi.advanceTimersByTimeAsync(180_000);
    expect(mockCheckOffsite).not.toHaveBeenCalled();
    expect(mockCheckAutoBackup).not.toHaveBeenCalled();

    resumeAutoBackupCheck();
    await vi.advanceTimersByTimeAsync(60_000);
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(1);
    expect(mockCheckOffsite).toHaveBeenCalledTimes(1);
  });

  it("REQ-901 / UI-11b-D16: PC の中の確認の待機中に止めると、その結果の後に offsite を開始しない", async () => {
    renderCheck();
    await vi.advanceTimersByTimeAsync(0);
    mockCheckOffsite.mockClear();
    const auto = pendingCheck();
    await vi.advanceTimersByTimeAsync(60_000);
    const waiting = trackSettled(suspendAutoBackupCheckAndWait());
    auto.settle(ok(false));
    await vi.advanceTimersByTimeAsync(0);
    expect(waiting.settled).toBe(true);
    expect(mockCheckOffsite).not.toHaveBeenCalled();
  });

  it("REQ-901 / UI-11b-D15・D16: 開始済みの offsite（mount の 1 回）の完了まで待ち、その結果を捨てる", async () => {
    const offsite = pendingOffsite();
    const { invalidateSpy } = renderCheck();
    try {
      await vi.advanceTimersByTimeAsync(0);
      expect(mockCheckOffsite).toHaveBeenCalledTimes(1);
      const waiting = trackSettled(suspendAutoBackupCheckAndWait());
      await vi.advanceTimersByTimeAsync(120_000);
      expect(waiting.settled).toBe(false);
      expect(mockCheckAutoBackup).not.toHaveBeenCalled();

      offsite.settle(offsiteFailure());
      await vi.advanceTimersByTimeAsync(0);
      expect(waiting.settled).toBe(true);
      expect(invalidationsOf(invalidateSpy, offsiteKey)).toBe(0);
      expect(mockToastError).not.toHaveBeenCalled();
    } finally {
      offsite.settle(offsiteOk());
    }
  });

  it("REQ-901 / UI-11b-D15・D16: interval の回の offsite の完了まで待つ", async () => {
    renderCheck();
    await vi.advanceTimersByTimeAsync(0);
    const offsite = pendingOffsite();
    try {
      await vi.advanceTimersByTimeAsync(60_000);
      expect(mockCheckOffsite).toHaveBeenCalledTimes(2);
      const waiting = trackSettled(suspendAutoBackupCheckAndWait());
      await vi.advanceTimersByTimeAsync(0);
      expect(waiting.settled).toBe(false);
      offsite.settle(offsiteOk({ kind: "up_to_date" }));
      await vi.advanceTimersByTimeAsync(0);
      expect(waiting.settled).toBe(true);
    } finally {
      offsite.settle(offsiteOk());
    }
  });

  it("REQ-901 / UI-11b-D16: 失敗の toast は連続の最初の 1 回だけ、medium_missing では出さず、失敗でない結果で連続を解く", async () => {
    mockCheckOffsite
      .mockResolvedValueOnce(offsiteOk({ kind: "medium_missing" }))
      .mockResolvedValueOnce(offsiteFailure())
      .mockResolvedValueOnce(offsiteFailure())
      .mockResolvedValueOnce(offsiteOk({ kind: "up_to_date" }))
      .mockResolvedValueOnce(offsiteFailure());
    renderCheck();
    await vi.advanceTimersByTimeAsync(0);
    expect(mockToastError).not.toHaveBeenCalled();

    await vi.advanceTimersByTimeAsync(120_000);
    expect(mockCheckOffsite).toHaveBeenCalledTimes(3);
    expect(mockToastError).toHaveBeenCalledTimes(1);
    expect(mockToastError).toHaveBeenCalledWith("PC の外の控えを写せませんでした", {
      id: "backup-offsite-error",
    });

    await vi.advanceTimersByTimeAsync(120_000);
    expect(mockCheckOffsite).toHaveBeenCalledTimes(5);
    expect(mockToastError).toHaveBeenCalledTimes(2);
    expect(mockToastSuccess).not.toHaveBeenCalled();
  });

  it("REQ-901 / UI-11b-D16: 結果の種類に依らず status を invalidate する（6 種）", async () => {
    const results: OffsiteResult[] = [
      offsiteOk({ kind: "not_prepared" }),
      offsiteOk({ kind: "no_local_backup" }),
      offsiteOk({ kind: "medium_missing" }),
      offsiteOk({ kind: "up_to_date" }),
      offsiteOk({
        kind: "copied",
        file_name: "inventory_backup_20991231_235959.db",
        labels: ["控え 2"],
      }),
      offsiteFailure(),
    ];
    for (const result of results) mockCheckOffsite.mockResolvedValueOnce(result);
    const { invalidateSpy } = renderCheck();
    await vi.advanceTimersByTimeAsync(0);
    expect(invalidationsOf(invalidateSpy, offsiteKey)).toBe(1);
    for (let round = 2; round <= results.length; round += 1) {
      await vi.advanceTimersByTimeAsync(60_000);
      expect(invalidationsOf(invalidateSpy, offsiteKey)).toBe(round);
    }
  });
});
