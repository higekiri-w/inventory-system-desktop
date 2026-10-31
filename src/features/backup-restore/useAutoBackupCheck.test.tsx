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
  commands: { checkAutoBackup: vi.fn() },
}));

type CheckResult = Awaited<ReturnType<typeof commands.checkAutoBackup>>;

const mockCheckAutoBackup = vi.mocked(commands.checkAutoBackup);
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
    expect(invalidateSpy).toHaveBeenCalledTimes(1);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: queryKeys.backupRestore.list() });
    expect(mockToastSuccess).toHaveBeenCalledTimes(1);
    expect(mockToastSuccess).toHaveBeenCalledWith("自動バックアップを作成しました");

    // 次の回は false
    await vi.advanceTimersByTimeAsync(60_000);
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(2);
    expect(invalidateSpy).toHaveBeenCalledTimes(1);
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
      expect(invalidateSpy).not.toHaveBeenCalled();
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
      expect(invalidateSpy).not.toHaveBeenCalled();

      await vi.advanceTimersByTimeAsync(60_000);
      expect(mockCheckAutoBackup).toHaveBeenCalledTimes(2);
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
