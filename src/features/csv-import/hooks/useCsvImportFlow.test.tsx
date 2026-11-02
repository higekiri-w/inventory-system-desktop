import { QueryClient, QueryClientProvider, useQuery } from "@tanstack/react-query";
import { act, render, renderHook, screen, waitFor } from "@testing-library/react";
import { useBlocker } from "@tanstack/react-router";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { toast } from "sonner";

import {
  commands,
  CSV_IMPORT_FILE_SIZE_LIMIT,
  type ImportResult,
  type PreviewData,
  type RollbackResult,
} from "@/lib/bindings";
import { queryKeys } from "@/lib/query-keys";
import { d052InvalidationOracle, expectExactInvalidations } from "@/test/invalidation-oracle";
import { useCsvImportFlow } from "./useCsvImportFlow";

vi.mock("@tanstack/react-router", () => ({ useBlocker: vi.fn() }));
vi.mock("sonner", () => ({ toast: { error: vi.fn(), success: vi.fn() } }));
vi.mock("@/lib/bindings", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/bindings")>();
  return {
    ...actual,
    commands: {
      parseAndValidateCsv: vi.fn(),
      commitCsvImport: vi.fn(),
      rollbackCsvImport: vi.fn(),
    },
  };
});

const mockParse = vi.mocked(commands.parseAndValidateCsv);
const mockCommit = vi.mocked(commands.commitCsvImport);
const mockRollback = vi.mocked(commands.rollbackCsvImport);
// eslint-disable-next-line @typescript-eslint/no-deprecated -- production の現行 hook 配線を spy する。
const mockUseBlocker = vi.mocked(useBlocker);
const mockToast = vi.mocked(toast);

function makePreview(): PreviewData {
  return {
    file_info: {
      filename: "sales.csv",
      settlement_date: "2026-07-23",
      file_hash: "a".repeat(64),
    },
    matched_summary: { count: 1, total_amount: 500, warnings: [] },
    error_summary: { count: 0, items: [] },
    settlement_gaps: [],
    settlement_date_after_today: false,
    preview_created_at: "2026-07-23T10:00:00",
  };
}

function makeResult(): ImportResult {
  return {
    csv_import_id: 401,
    status: "completed",
    total_items: 1,
    total_amount: 500,
    skipped_count: 0,
    settlement_gaps: [],
    settlement_date_after_today: false,
    recheck_targets: [],
  };
}

function makeRollback(): RollbackResult {
  return { success: true, voided_sale_count: 1, voided_movement_count: 1, recheck_targets: [] };
}

interface BlockerOptions {
  shouldBlockFn: () => boolean;
  enableBeforeUnload?: boolean | (() => boolean);
}

/// 最後の render の始めの useBlocker の呼出し数。renderHook の callback で毎 render 書き換える
const blockerMark = { calls: 0 };

function renderFlow(queryClient: QueryClient) {
  return renderHook(
    () => {
      blockerMark.calls = mockUseBlocker.mock.calls.length;
      return useCsvImportFlow();
    },
    { wrapper: makeWrapper(queryClient) },
  );
}

/// 最後の render が登録した全ての blocker（importing と取消の止め）。どれかが true なら止まる
/// （@tanstack/history は blockers を順に見て 1 つでも true なら止める）
function lastRenderBlocks(): { route: boolean; beforeUnload: boolean } {
  const options = mockUseBlocker.mock.calls
    .slice(blockerMark.calls)
    .map((call) => call[0] as unknown as BlockerOptions);
  return {
    route: options.some((o) => o.shouldBlockFn()),
    beforeUnload: options.some((o) =>
      typeof o.enableBeforeUnload === "function"
        ? o.enableBeforeUnload()
        : o.enableBeforeUnload === true,
    ),
  };
}

function makeWrapper(queryClient: QueryClient) {
  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
  };
}

async function reachResult(result: {
  current: ReturnType<typeof useCsvImportFlow>;
}): Promise<void> {
  act(() => {
    result.current.selectFile({
      bytes: new Uint8Array([1, 2, 3]),
      filename: "sales.csv",
      size: 3,
    });
  });
  await waitFor(() => {
    expect(result.current.state.status).toBe("preview");
  });
  act(() => {
    result.current.confirmImport();
  });
  await waitFor(() => {
    expect(result.current.state.status).toBe("result");
  });
}

beforeEach(() => {
  vi.clearAllMocks();
  mockParse.mockResolvedValue({
    status: "ok",
    data: { preview_data: makePreview(), preview_token: "preview-token" },
  });
  mockCommit.mockResolvedValue({ status: "ok", data: makeResult() });
  mockRollback.mockResolvedValue({ status: "ok", data: makeRollback() });
});

describe("useCsvImportFlow UI-07 D-052-C8/C9", () => {
  it("REQ-401: 上限ちょうどは parse し、上限 +1 は command 前に拒否する", async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const { result } = renderHook(() => useCsvImportFlow(), {
      wrapper: makeWrapper(queryClient),
    });

    act(() => {
      result.current.selectFile({
        bytes: new Uint8Array([1]),
        filename: "boundary.csv",
        size: CSV_IMPORT_FILE_SIZE_LIMIT,
      });
    });
    await waitFor(() => {
      expect(mockParse).toHaveBeenCalledWith([1], "boundary.csv");
    });

    mockParse.mockClear();
    act(() => {
      result.current.selectFile({
        bytes: new Uint8Array([1]),
        filename: "oversize.csv",
        size: CSV_IMPORT_FILE_SIZE_LIMIT + 1,
      });
    });
    expect(mockParse).not.toHaveBeenCalled();
    expect(mockToast.error).toHaveBeenCalledWith("ファイルサイズが上限(20MB)を超えています");
  });

  it("REQ-401: parse failure は idle へ復帰可能な error state にする", async () => {
    mockParse.mockResolvedValueOnce({
      status: "error",
      error: {
        kind: "validation",
        message: "合成CSVを解析できません",
        field: "file",
        error_id: null,
        stocktake_recovery: null,
      },
    });
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const { result } = renderHook(() => useCsvImportFlow(), {
      wrapper: makeWrapper(queryClient),
    });

    act(() => {
      result.current.selectFile({
        bytes: new Uint8Array([0]),
        filename: "invalid.csv",
        size: 1,
      });
    });

    await waitFor(() => {
      expect(result.current.state).toMatchObject({ status: "error", recoverTo: "idle" });
    });
    act(() => {
      result.current.dismissError();
    });
    expect(result.current.state.status).toBe("idle");
  });

  it("REQ-401: import_error commit failure recovers to idle", async () => {
    mockCommit.mockResolvedValueOnce({
      status: "error",
      error: {
        kind: "import_error",
        message: "合成済み preview が見つかりません",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const { result } = renderHook(() => useCsvImportFlow(), {
      wrapper: makeWrapper(queryClient),
    });

    act(() => {
      result.current.selectFile({
        bytes: new Uint8Array([1]),
        filename: "sales.csv",
        size: 1,
      });
    });
    await waitFor(() => {
      expect(result.current.state.status).toBe("preview");
    });
    act(() => {
      result.current.confirmImport();
    });
    await waitFor(() => {
      expect(result.current.state.status).toBe("error");
    });
    expect(result.current.state).toMatchObject({ status: "error", recoverTo: "idle" });

    act(() => {
      result.current.dismissError();
    });
    expect(result.current.state.status).toBe("idle");
  });

  it("REQ-401: internal commit failure shows the error state and recovers to the same preview", async () => {
    // 55 §55.5 / §55.9: 取込み（commit）の失敗は kind を問わず ErrorState を出す。
    // import_error 以外は recoverTo="preview" で、戻ると失敗前と同じプレビューから再試行できる。
    mockCommit.mockResolvedValueOnce({
      status: "error",
      error: {
        kind: "internal",
        message: "合成の DB ロック競合",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const { result } = renderHook(() => useCsvImportFlow(), {
      wrapper: makeWrapper(queryClient),
    });

    act(() => {
      result.current.selectFile({
        bytes: new Uint8Array([1]),
        filename: "sales.csv",
        size: 1,
      });
    });
    await waitFor(() => {
      expect(result.current.state.status).toBe("preview");
    });
    const previewBeforeFailure = result.current.state;
    act(() => {
      result.current.confirmImport();
    });
    await waitFor(() => {
      expect(result.current.state.status).toBe("error");
    });
    expect(result.current.state).toMatchObject({ status: "error", recoverTo: "preview" });

    act(() => {
      result.current.dismissError();
    });
    expect(result.current.state).toEqual(previewBeforeFailure);
  });

  it("commit invalidates the exact independent oracle set", async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useCsvImportFlow(), {
      wrapper: makeWrapper(queryClient),
    });

    await reachResult(result);

    expect(mockCommit).toHaveBeenCalledWith("preview-token");
    expectExactInvalidations(invalidateSpy.mock.calls, d052InvalidationOracle.csvImportCommit());
  });

  it("rollback invalidates the same exact independent oracle set", async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useCsvImportFlow(), {
      wrapper: makeWrapper(queryClient),
    });
    await reachResult(result);
    invalidateSpy.mockClear();

    act(() => {
      result.current.rollback(401);
    });
    await waitFor(() => {
      expect(result.current.state.status).toBe("idle");
    });

    expect(mockRollback).toHaveBeenCalledWith(401);
    // 55 の「取消の結果」: 取消の結果（要再確認の一覧）を画面へ渡す
    expect(result.current.rollbackResult).toEqual(makeRollback());
    expectExactInvalidations(invalidateSpy.mock.calls, d052InvalidationOracle.csvImportRollback());
    // T7 (UI 表示磨き batch Scope 7(a) gated Amendment): 取消完了 toast は既定 duration (3000ms)
    // より長く表示し、非IT高齢operatorが読み切れる時間を確保する（文言は既存維持）。
    expect(toast.success).toHaveBeenCalledWith("取込みを取り消しました", { duration: 8000 });
  });

  it("REQ-401: commit 中だけ useBlocker と beforeunload を有効化する", async () => {
    let resolveCommit!: (value: Awaited<ReturnType<typeof commands.commitCsvImport>>) => void;
    mockCommit.mockReturnValueOnce(
      new Promise((resolve) => {
        resolveCommit = resolve;
      }),
    );
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const { result } = renderFlow(queryClient);

    act(() => {
      result.current.selectFile({
        bytes: new Uint8Array([1]),
        filename: "sales.csv",
        size: 1,
      });
    });
    await waitFor(() => {
      expect(result.current.state.status).toBe("preview");
    });
    act(() => {
      result.current.confirmImport();
    });
    await waitFor(() => {
      expect(result.current.state.status).toBe("importing");
    });

    // Matrix T7: 最後の render の全ての blocker を読む（取消の止めが後に呼ばれても importing の止めを見る）
    expect(lastRenderBlocks()).toEqual({ route: true, beforeUnload: true });

    resolveCommit({ status: "ok", data: makeResult() });
    await waitFor(() => {
      expect(result.current.state.status).toBe("result");
    });
  });

  it("REQ-401 / UI-07-D18: 取消の応答を待つ間は useBlocker と beforeunload で画面を離れず、失敗すれば離れられる", async () => {
    // 55 §55.7・§55.11（Matrix U12）: 取消の止めは共有の hook（useRollbackCsvImport）の 1 か所
    let resolveRollback!: (value: Awaited<ReturnType<typeof commands.rollbackCsvImport>>) => void;
    mockRollback.mockReturnValueOnce(
      new Promise((resolve) => {
        resolveRollback = resolve;
      }),
    );
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const { result } = renderFlow(queryClient);
    await reachResult(result);
    expect(lastRenderBlocks()).toEqual({ route: false, beforeUnload: false });

    act(() => {
      result.current.rollback(401);
    });
    await waitFor(() => {
      expect(result.current.isRollingBack).toBe(true);
    });
    expect(lastRenderBlocks()).toEqual({ route: true, beforeUnload: true });

    resolveRollback({
      status: "error",
      error: {
        kind: "internal",
        message: "合成の取消の失敗",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    await waitFor(() => {
      expect(mockToast.error).toHaveBeenCalledWith(
        "取り消しに失敗しました。もう一度お試しください",
      );
    });
    await waitFor(() => {
      expect(result.current.isRollingBack).toBe(false);
    });
    expect(result.current.state.status).toBe("result");
    expect(lastRenderBlocks()).toEqual({ route: false, beforeUnload: false });
  });

  it("test_import_rollback_req401_failure_retries_same_id_success_refetches_remaining_aggregate", async () => {
    // REQ-401 / I-U8 / I-R6 / SPEC-SDI-D7 / D-052: failureはresult保持、same ID retry成功でremaining aggregateを再表示する。
    mockRollback.mockResolvedValueOnce({
      status: "error",
      error: {
        kind: "internal",
        message: "合成 rollback failure",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const reportQuery = vi
      .fn()
      .mockResolvedValueOnce(900)
      .mockResolvedValueOnce(900)
      .mockResolvedValue(500);
    const harness: { flow: ReturnType<typeof useCsvImportFlow> | null } = { flow: null };
    function Harness() {
      harness.flow = useCsvImportFlow();
      const report = useQuery({
        queryKey: queryKeys.dailySales("2026-07-23"),
        queryFn: reportQuery,
      });
      return <output aria-label="日次売上合計">{report.data ?? "loading"}</output>;
    }
    render(
      <QueryClientProvider client={queryClient}>
        <Harness />
      </QueryClientProvider>,
    );
    expect(await screen.findByText("900")).toBeInTheDocument();
    act(() => {
      harness.flow?.selectFile({
        bytes: new Uint8Array([1, 2, 3]),
        filename: "sales.csv",
        size: 3,
      });
    });
    await waitFor(() => {
      expect(harness.flow?.state.status).toBe("preview");
    });
    act(() => {
      harness.flow?.confirmImport();
    });
    await waitFor(() => {
      expect(harness.flow?.state.status).toBe("result");
    });
    expect(await screen.findByText("900")).toBeInTheDocument();

    act(() => {
      harness.flow?.rollback(401);
    });

    await waitFor(() => {
      expect(mockToast.error).toHaveBeenCalledWith(
        "取り消しに失敗しました。もう一度お試しください",
      );
    });
    expect(harness.flow?.state.status).toBe("result");
    expect(screen.getByText("900")).toBeInTheDocument();

    act(() => {
      harness.flow?.rollback(401);
    });
    await waitFor(() => {
      expect(harness.flow?.state.status).toBe("idle");
    });
    expect(await screen.findByText("500")).toBeInTheDocument();
    expect(reportQuery).toHaveBeenCalledTimes(3);
    expect(mockRollback).toHaveBeenNthCalledWith(1, 401);
    expect(mockRollback).toHaveBeenNthCalledWith(2, 401);
  });
});
