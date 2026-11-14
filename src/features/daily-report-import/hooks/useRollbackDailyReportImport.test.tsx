// REQ-401 / UI-07-D19 / 55 §55.7・§55.9: 日報の取消の共有の hook（入力はすべて合成）
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useBlocker } from "@tanstack/react-router";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { toast } from "sonner";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { commands, type CmdError, type DailyReportRollbackResult } from "@/lib/bindings";
import { d052InvalidationOracle, expectExactInvalidations } from "@/test/invalidation-oracle";
import { useRollbackDailyReportImport } from "./useRollbackDailyReportImport";

vi.mock("@tanstack/react-router", () => ({ useBlocker: vi.fn() }));
vi.mock("sonner", () => ({ toast: { error: vi.fn(), success: vi.fn() } }));
vi.mock("@/lib/bindings", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/bindings")>();
  return { ...actual, commands: { rollbackDailyReportImport: vi.fn() } };
});

type RollbackResponse = Awaited<ReturnType<typeof commands.rollbackDailyReportImport>>;
interface Blocker {
  shouldBlockFn: () => boolean;
  enableBeforeUnload: () => boolean;
}

const mockRollback = vi.mocked(commands.rollbackDailyReportImport);
// eslint-disable-next-line @typescript-eslint/no-deprecated -- production の現行 hook 配線を spy する。
const mockUseBlocker = vi.mocked(useBlocker);
const lastBlocker = () =>
  mockUseBlocker.mock.calls[mockUseBlocker.mock.calls.length - 1]?.[0] as unknown as Blocker;

const OK: DailyReportRollbackResult = {
  daily_report_import_id: 7,
  status: "rolled_back",
  rolled_back_at: "2026-03-21T11:00:00",
};
const FAILED: CmdError = {
  kind: "internal",
  message: "合成の取消の失敗",
  field: null,
  error_id: null,
  stocktake_recovery: null,
};

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((r) => {
    resolve = r;
  });
  return { promise, resolve };
}

function setup(onSuccess = vi.fn()) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  const hook = renderHook(() => useRollbackDailyReportImport({ onSuccess }), { wrapper });
  return { ...hook, invalidateSpy, onSuccess };
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("useRollbackDailyReportImport_req401", () => {
  it("REQ-401 / §55.7: 取消の応答を待つ間だけ離脱を止め、成功の後は Toast・C10 の再取得・onSuccess の後に解く", async () => {
    const { result, invalidateSpy, onSuccess } = setup();
    expect(lastBlocker().shouldBlockFn()).toBe(false);
    expect(lastBlocker().enableBeforeUnload()).toBe(false);

    const pending = deferred<RollbackResponse>();
    mockRollback.mockReturnValueOnce(pending.promise);
    act(() => {
      result.current.mutate(7);
    });
    await waitFor(() => {
      expect(result.current.isPending).toBe(true);
    });
    expect(lastBlocker().shouldBlockFn()).toBe(true);
    expect(lastBlocker().enableBeforeUnload()).toBe(true);
    expect(mockRollback).toHaveBeenCalledWith(7);

    pending.resolve({ status: "ok", data: OK });
    await waitFor(() => {
      expect(result.current.isPending).toBe(false);
    });
    expect(lastBlocker().shouldBlockFn()).toBe(false);
    expect(lastBlocker().enableBeforeUnload()).toBe(false);
    expect(onSuccess).toHaveBeenCalledWith(OK);
    expect(toast.success).toHaveBeenCalledWith("日報取込みを取り消しました", { duration: 8000 });
    expectExactInvalidations(invalidateSpy.mock.calls, d052InvalidationOracle.dailyReportImport());
  });

  it("REQ-401 / §55.9: 失敗は Toast だけで、止めを解き onSuccess を呼ばず再取得もしない", async () => {
    const { result, invalidateSpy, onSuccess } = setup();
    const pending = deferred<RollbackResponse>();
    mockRollback.mockReturnValueOnce(pending.promise);
    act(() => {
      result.current.mutate(7);
    });
    await waitFor(() => {
      expect(lastBlocker().shouldBlockFn()).toBe(true);
    });

    pending.resolve({ status: "error", error: FAILED });
    await waitFor(() => {
      expect(result.current.isPending).toBe(false);
    });
    expect(lastBlocker().shouldBlockFn()).toBe(false);
    expect(lastBlocker().enableBeforeUnload()).toBe(false);
    expect(toast.error).toHaveBeenCalledWith("取り消しに失敗しました。もう一度お試しください");
    expect(onSuccess).not.toHaveBeenCalled();
    expect(invalidateSpy).not.toHaveBeenCalled();
  });
});
