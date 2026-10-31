import { QueryClient, QueryClientProvider, useQuery } from "@tanstack/react-query";
import { act, render, renderHook, screen, waitFor } from "@testing-library/react";
import { useBlocker } from "@tanstack/react-router";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { toast } from "sonner";
import { commands } from "@/lib/bindings";
import type {
  DailyReportImportResult,
  DailyReportPreviewData,
  DailyReportRollbackResult,
} from "@/lib/bindings";
import { d052InvalidationOracle, expectExactInvalidations } from "@/test/invalidation-oracle";
import { queryKeys } from "@/lib/query-keys";
import { open } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import {
  DAILY_REPORT_LAST_DIR_STORAGE_KEY,
  useDailyReportImportFlow,
} from "./useDailyReportImportFlow";

vi.mock("@tanstack/react-router", () => ({
  useBlocker: vi.fn(),
}));

vi.mock("sonner", () => ({
  toast: {
    error: vi.fn(),
    success: vi.fn(),
  },
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-fs", () => ({
  readFile: vi.fn(),
}));

vi.mock("@/lib/bindings", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/bindings")>();
  return {
    ...actual,
    commands: {
      parseAndValidateDailyReport: vi.fn(),
      parseAndValidateDailyReportFromSd: vi.fn(),
      scanRegisterSd: vi.fn(),
      commitDailyReportImport: vi.fn(),
      rollbackDailyReportImport: vi.fn(),
    },
  };
});

const mockParse = vi.mocked(commands.parseAndValidateDailyReport);
const mockCommit = vi.mocked(commands.commitDailyReportImport);
const mockRollback = vi.mocked(commands.rollbackDailyReportImport);
// eslint-disable-next-line @typescript-eslint/no-deprecated -- production の現行 hook 配線を spy する。
const mockUseBlocker = vi.mocked(useBlocker);
const mockToast = vi.mocked(toast);
const mockOpen = vi.mocked(open);
const mockReadFile = vi.mocked(readFile);

function makePreview(): DailyReportPreviewData {
  return {
    file_info: {
      report_date: "2026-03-21",
      bundle_hash: "a".repeat(64),
      source_files: [
        { source: "Z001", filename: "Z001_260321.CSV", file_hash: "1".repeat(64), size_bytes: 10 },
        { source: "Z002", filename: "Z002_260321.CSV", file_hash: "2".repeat(64), size_bytes: 10 },
        { source: "Z005", filename: "Z005_260321.CSV", file_hash: "3".repeat(64), size_bytes: 10 },
      ],
    },
    totals: { gross_amount: 12000, net_amount: 11000 },
    payment_summary: [
      { payment_key: "cash", label: "現金", amount: 11000, count: 7, sort_order: 1 },
    ],
    department_summary: [
      {
        department_id: 1,
        raw_department_name: "毛糸",
        normalized_department_name: "毛糸",
        amount: 8000,
        quantity: 5,
        count: 3,
        sort_order: 1,
      },
    ],
    warnings: [],
    duplicate_check: { status: "NoDuplicate", same_date_imports: [] },
    preview_created_at: "2026-03-21T10:00:00",
  };
}

function makeResult(): DailyReportImportResult {
  return {
    daily_report_import_id: 501,
    status: "completed",
    report_date: "2026-03-21",
    gross_amount: 12000,
    net_amount: 11000,
    warning_count: 0,
  };
}

function makeRollback(): DailyReportRollbackResult {
  return {
    daily_report_import_id: 501,
    status: "rolled_back",
    rolled_back_at: "2026-03-21T10:05:00",
  };
}

const THREE_PATHS = [
  "C:\\tmp\\Z001_260321.CSV",
  "C:\\tmp\\Z002_260321.CSV",
  "C:\\tmp\\Z005_260321.CSV",
];

/** UI-07-D16: dialog で 3 つを選び、「確認する」で parse する */
async function chooseAndConfirmThree(flow: () => ReturnType<typeof useDailyReportImportFlow>) {
  mockOpen.mockResolvedValueOnce(THREE_PATHS);
  await act(async () => {
    await flow().chooseFiles();
  });
  act(() => {
    flow().confirmManualFiles();
  });
}

function makeWrapper(queryClient: QueryClient) {
  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
  };
}

function installMemoryStorage() {
  const entries = new Map<string, string>();
  const storage: Storage = {
    get length() {
      return entries.size;
    },
    clear: () => {
      entries.clear();
    },
    getItem: (key: string) => entries.get(key) ?? null,
    key: (index: number) => Array.from(entries.keys())[index] ?? null,
    removeItem: (key: string) => {
      entries.delete(key);
    },
    setItem: (key: string, value: string) => {
      entries.set(key, value);
    },
  };
  Object.defineProperty(window, "localStorage", {
    value: storage,
    configurable: true,
  });
  Object.defineProperty(globalThis, "localStorage", {
    value: storage,
    configurable: true,
  });
}

function installThrowingStorage() {
  const storage = {
    get length() {
      return 0;
    },
    clear: () => {
      throw new Error("storage denied");
    },
    getItem: () => {
      throw new Error("storage denied");
    },
    key: () => null,
    removeItem: () => {
      throw new Error("storage denied");
    },
    setItem: () => {
      throw new Error("storage denied");
    },
  } as Storage;
  Object.defineProperty(window, "localStorage", {
    value: storage,
    configurable: true,
  });
}

beforeEach(() => {
  vi.clearAllMocks();
  mockParse.mockResolvedValue({
    status: "ok",
    data: { preview_data: makePreview(), preview_token: "preview-token-req401" },
  });
  mockCommit.mockResolvedValue({ status: "ok", data: makeResult() });
  mockRollback.mockResolvedValue({ status: "ok", data: makeRollback() });
  mockOpen.mockReset();
  mockReadFile.mockReset();
  mockReadFile.mockResolvedValue(new Uint8Array([1, 2, 3]));
  installMemoryStorage();
});

describe("useDailyReportImportFlow_req401", () => {
  it.each([
    ["1 file", ["C:\\tmp\\Z001_260321.CSV"]],
    ["2 files", ["C:\\tmp\\Z001_260321.CSV", "C:\\tmp\\Z002_260321.CSV"]],
  ])(
    "REQ-401 / UI-07-D16: %s cannot be confirmed before 3 series are present",
    async (_label, paths) => {
      mockOpen.mockResolvedValueOnce(paths);
      const queryClient = new QueryClient({
        defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
      });
      const { result } = renderHook(() => useDailyReportImportFlow(), {
        wrapper: makeWrapper(queryClient),
      });

      await act(async () => {
        await result.current.chooseFiles();
      });
      act(() => {
        result.current.confirmManualFiles();
      });

      expect(result.current.manualFiles.map((file) => file.series)).toHaveLength(paths.length);
      expect(mockParse).not.toHaveBeenCalled();
      expect(result.current.state.status).toBe("idle");
    },
  );

  it("REQ-401 / UI-07-D16 (c): the same series twice in one dialog adds none and explains why", async () => {
    mockOpen.mockResolvedValueOnce([
      "C:\\tmp\\Z001_260321.CSV",
      "C:\\tmp\\Z002_260321.CSV",
      "C:\\tmp\\Z005_260321.CSV",
      "C:\\tmp\\Z005_duplicate_260321.CSV",
    ]);
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const { result } = renderHook(() => useDailyReportImportFlow(), {
      wrapper: makeWrapper(queryClient),
    });

    await act(async () => {
      await result.current.chooseFiles();
    });
    act(() => {
      result.current.confirmManualFiles();
    });

    expect(result.current.manualFiles).toEqual([]);
    expect(mockParse).not.toHaveBeenCalled();
    expect(result.current.state.lastSelectionError).toBe(
      "Z001・Z002・Z005 は 1 つずつ選んでください（同じ種類のファイルが 2 つ以上あります）。",
    );
  });

  it("REQ-401 / UI-07-D16: the hook no longer accepts path-less File input", () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const { result } = renderHook(() => useDailyReportImportFlow(), {
      wrapper: makeWrapper(queryClient),
    });
    expect("selectFiles" in result.current).toBe(false);
  });

  it("REQ-401: native dialog cancellation does not change state or parse", async () => {
    mockOpen.mockResolvedValue(null);
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const { result } = renderHook(() => useDailyReportImportFlow(), {
      wrapper: makeWrapper(queryClient),
    });

    await act(async () => {
      await result.current.chooseFiles();
    });

    expect(mockOpen).toHaveBeenCalledWith({
      multiple: true,
      defaultPath: undefined,
      filters: [{ name: "CSV", extensions: ["csv", "CSV"] }],
    });
    expect(mockReadFile).not.toHaveBeenCalled();
    expect(mockParse).not.toHaveBeenCalled();
    expect(result.current.state.status).toBe("idle");
    expect(result.current.state.lastSelectionError).toBeNull();
  });

  it("REQ-401: remembers the selected folder and reopens the dialog from it", async () => {
    mockOpen.mockResolvedValue([
      "C:\\CASIO\\XZ\\2026\\07\\Z001_260321.CSV",
      "C:\\CASIO\\XZ\\2026\\07\\Z002_260321.CSV",
      "C:\\CASIO\\XZ\\2026\\07\\Z005_260321.CSV",
    ]);
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const { result } = renderHook(() => useDailyReportImportFlow(), {
      wrapper: makeWrapper(queryClient),
    });

    await act(async () => {
      await result.current.chooseFiles();
    });

    expect(mockOpen).toHaveBeenLastCalledWith(expect.objectContaining({ defaultPath: undefined }));
    expect(window.localStorage.getItem(DAILY_REPORT_LAST_DIR_STORAGE_KEY)).toBe(
      "C:\\CASIO\\XZ\\2026\\07",
    );

    await act(async () => {
      await result.current.chooseFiles();
    });

    expect(mockOpen).toHaveBeenLastCalledWith(
      expect.objectContaining({ defaultPath: "C:\\CASIO\\XZ\\2026\\07" }),
    );
  });

  it("REQ-401: keeps the browsed folder even when fewer than 3 files are chosen", async () => {
    mockOpen.mockResolvedValue(["C:\\tmp\\Z001_260321.CSV", "C:\\tmp\\Z002_260321.CSV"]);
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const { result } = renderHook(() => useDailyReportImportFlow(), {
      wrapper: makeWrapper(queryClient),
    });

    await act(async () => {
      await result.current.chooseFiles();
    });

    expect(window.localStorage.getItem(DAILY_REPORT_LAST_DIR_STORAGE_KEY)).toBe("C:\\tmp");
    expect(result.current.manualFiles.map((file) => file.filename)).toEqual([
      "Z001_260321.CSV",
      "Z002_260321.CSV",
    ]);
    expect(mockParse).not.toHaveBeenCalled();
  });

  it("REQ-401: selection flow continues when localStorage is unavailable", async () => {
    installThrowingStorage();
    mockOpen.mockResolvedValue([
      "C:\\tmp\\Z001_260321.CSV",
      "C:\\tmp\\Z002_260321.CSV",
      "C:\\tmp\\Z005_260321.CSV",
    ]);
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const { result } = renderHook(() => useDailyReportImportFlow(), {
      wrapper: makeWrapper(queryClient),
    });

    await act(async () => {
      await result.current.chooseFiles();
    });
    act(() => {
      result.current.confirmManualFiles();
    });

    expect(result.current.state.lastSelectionError).toBeNull();
    await waitFor(() => {
      expect(mockParse).toHaveBeenCalledTimes(1);
    });
  });

  it("REQ-401: native dialog with fewer than 3 files does not parse", async () => {
    mockOpen.mockResolvedValue(["C:\\tmp\\Z001_260321.CSV", "C:\\tmp\\Z002_260321.CSV"]);
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const { result } = renderHook(() => useDailyReportImportFlow(), {
      wrapper: makeWrapper(queryClient),
    });

    await act(async () => {
      await result.current.chooseFiles();
    });
    act(() => {
      result.current.confirmManualFiles();
    });

    expect(mockParse).not.toHaveBeenCalled();
    expect(result.current.state.status).toBe("idle");
    expect(result.current.state.lastSelectionError).toBeNull();
  });

  it("REQ-401: native dialog read failure stays idle without unhandled parsing", async () => {
    mockOpen.mockResolvedValue([
      "C:\\tmp\\Z001_260321.CSV",
      "C:\\tmp\\Z002_260321.CSV",
      "C:\\tmp\\Z005_260321.CSV",
    ]);
    mockReadFile.mockRejectedValue(new Error("read denied"));
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const { result } = renderHook(() => useDailyReportImportFlow(), {
      wrapper: makeWrapper(queryClient),
    });

    await act(async () => {
      await result.current.chooseFiles();
    });

    expect(mockParse).not.toHaveBeenCalled();
    expect(result.current.manualFiles).toEqual([]);
    expect(mockToast.error).toHaveBeenCalledWith("日報ファイルの選択または読み取りに失敗しました");
    expect(result.current.state.status).toBe("idle");
    expect(result.current.state.lastSelectionError).toBe(
      "日報ファイルの選択または読み取りに失敗しました",
    );
  });

  it("REQ-401: native dialog reads 3 files and parses them", async () => {
    mockOpen.mockResolvedValue([
      "C:\\tmp\\Z001_260321.CSV",
      "C:\\tmp\\Z002_260321.CSV",
      "C:\\tmp\\Z005_260321.CSV",
    ]);
    mockReadFile
      .mockResolvedValueOnce(new Uint8Array([1]))
      .mockResolvedValueOnce(new Uint8Array([2]))
      .mockResolvedValueOnce(new Uint8Array([5]));
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const { result } = renderHook(() => useDailyReportImportFlow(), {
      wrapper: makeWrapper(queryClient),
    });

    await act(async () => {
      await result.current.chooseFiles();
    });
    act(() => {
      result.current.confirmManualFiles();
    });

    expect(mockReadFile).toHaveBeenCalledTimes(3);
    await waitFor(() => {
      expect(result.current.state.status).toBe("preview");
    });
    expect(result.current.state.lastSelectionError).toBeNull();
    expect(mockParse).toHaveBeenCalledWith([
      { filename: "Z001_260321.CSV", file_bytes: [1], source_path: "C:\\tmp\\Z001_260321.CSV" },
      { filename: "Z002_260321.CSV", file_bytes: [2], source_path: "C:\\tmp\\Z002_260321.CSV" },
      { filename: "Z005_260321.CSV", file_bytes: [5], source_path: "C:\\tmp\\Z005_260321.CSV" },
    ]);
  });

  it("REQ-401: commit invalidates daily report and sales caches without invalidating Z004 csv import lists", async () => {
    // REQ-401 / I-U8 / SPEC-SDI-D7 / D-052: commit後のdaily/monthly/list invalidation exact oracle。
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useDailyReportImportFlow(), {
      wrapper: makeWrapper(queryClient),
    });

    await chooseAndConfirmThree(() => result.current);
    await waitFor(() => {
      expect(result.current.state.status).toBe("preview");
    });

    act(() => {
      result.current.confirmImport(false);
    });
    await waitFor(() => {
      expect(result.current.state.status).toBe("result");
    });

    expect(mockCommit).toHaveBeenCalledWith("preview-token-req401", false);
    expectExactInvalidations(invalidateSpy.mock.calls, d052InvalidationOracle.dailyReportImport());
  });

  it("test_daily_report_command_req401_additional_confirmation_reaches_biz", async () => {
    // REQ-401 / I-W4 / I-U6 / SPEC-SDI-D3,D5: 日報UI trueをgenerated commandへ一回だけ渡す。
    const additional = makePreview();
    additional.duplicate_check = {
      status: "AdditionalImportConfirmationRequired",
      same_date_imports: [
        {
          id: 500,
          source_filenames: ["Z001_old.CSV", "Z002_old.CSV", "Z005_old.CSV"],
          gross_amount: 9000,
          net_amount: 8000,
          imported_at: "2026-03-21T09:00:00",
        },
      ],
    };
    mockParse.mockResolvedValueOnce({
      status: "ok",
      data: { preview_data: additional, preview_token: "additional-preview-token" },
    });
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const { result } = renderHook(() => useDailyReportImportFlow(), {
      wrapper: makeWrapper(queryClient),
    });
    await chooseAndConfirmThree(() => result.current);
    await waitFor(() => {
      expect(result.current.state.status).toBe("preview");
    });

    act(() => {
      result.current.confirmImport(true);
    });

    await waitFor(() => {
      expect(result.current.state.status).toBe("result");
    });
    expect(mockCommit).toHaveBeenCalledTimes(1);
    expect(mockCommit).toHaveBeenCalledWith("additional-preview-token", true);
  });

  it("REQ-401: rollback invalidates the same daily report and sales caches", async () => {
    // REQ-401 / I-U8 / SPEC-SDI-D7 / D-052: rollback後も同じexact invalidationでremaining aggregateをrefetchする。
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useDailyReportImportFlow(), {
      wrapper: makeWrapper(queryClient),
    });

    await chooseAndConfirmThree(() => result.current);
    await waitFor(() => {
      expect(result.current.state.status).toBe("preview");
    });
    act(() => {
      result.current.confirmImport(false);
    });
    await waitFor(() => {
      expect(result.current.state.status).toBe("result");
    });
    invalidateSpy.mockClear();

    act(() => {
      result.current.rollback(501);
    });
    await waitFor(() => {
      // T7 (UI 表示磨き batch Scope 7(a) gated Amendment): 取消完了 toast は既定 duration
      // (3000ms) より長く表示し、非IT高齢operatorが読み切れる時間を確保する（文言は既存維持）。
      expect(mockToast.success).toHaveBeenCalledWith("日報取込みを取り消しました", {
        duration: 8000,
      });
    });

    expect(mockRollback).toHaveBeenCalledWith(501);
    expectExactInvalidations(invalidateSpy.mock.calls, d052InvalidationOracle.dailyReportImport());
  });

  it("test_daily_report_rollback_req401_failure_retries_same_id_and_refetches_remaining_aggregate", async () => {
    // REQ-401 / I-U8 / I-R6 / SPEC-SDI-D7 / D-052: failure保持→同一ID retry→残存aggregate再表示。
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
    const harness: { flow: ReturnType<typeof useDailyReportImportFlow> | null } = { flow: null };
    function Harness() {
      harness.flow = useDailyReportImportFlow();
      const report = useQuery({
        queryKey: queryKeys.dailySales("2026-03-21"),
        queryFn: reportQuery,
      });
      return <output aria-label="日報公式売上合計">{report.data ?? "loading"}</output>;
    }
    render(
      <QueryClientProvider client={queryClient}>
        <Harness />
      </QueryClientProvider>,
    );
    expect(await screen.findByText("900")).toBeInTheDocument();
    await chooseAndConfirmThree(() => {
      if (!harness.flow) throw new Error("harness not rendered");
      return harness.flow;
    });
    await waitFor(() => {
      expect(harness.flow?.state.status).toBe("preview");
    });
    act(() => {
      harness.flow?.confirmImport(false);
    });
    await waitFor(() => {
      expect(harness.flow?.state.status).toBe("result");
    });
    expect(await screen.findByText("900")).toBeInTheDocument();

    act(() => {
      harness.flow?.rollback(501);
    });
    await waitFor(() => {
      expect(mockToast.error).toHaveBeenCalledWith(
        "取り消しに失敗しました。もう一度お試しください",
      );
    });
    expect(harness.flow?.state.status).toBe("result");
    expect(screen.getByText("900")).toBeInTheDocument();

    act(() => {
      harness.flow?.rollback(501);
    });
    await waitFor(() => {
      expect(harness.flow?.state.status).toBe("idle");
    });
    expect(await screen.findByText("500")).toBeInTheDocument();
    expect(reportQuery).toHaveBeenCalledTimes(3);
    expect(mockRollback).toHaveBeenNthCalledWith(1, 501);
    expect(mockRollback).toHaveBeenNthCalledWith(2, 501);
  });

  it("REQ-401: commit 中だけ useBlocker と beforeunload を有効化する", async () => {
    let resolveCommit!: (
      value: Awaited<ReturnType<typeof commands.commitDailyReportImport>>,
    ) => void;
    mockCommit.mockReturnValueOnce(
      new Promise((resolve) => {
        resolveCommit = resolve;
      }),
    );
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const { result } = renderHook(() => useDailyReportImportFlow(), {
      wrapper: makeWrapper(queryClient),
    });

    await chooseAndConfirmThree(() => result.current);
    await waitFor(() => {
      expect(result.current.state.status).toBe("preview");
    });
    act(() => {
      result.current.confirmImport(false);
    });
    await waitFor(() => {
      expect(result.current.state.status).toBe("importing");
    });

    const blockerOptions = mockUseBlocker.mock.calls[mockUseBlocker.mock.calls.length - 1]?.[0] as
      | {
          shouldBlockFn: () => boolean;
          enableBeforeUnload: boolean | (() => boolean);
        }
      | undefined;
    expect(blockerOptions?.shouldBlockFn()).toBe(true);
    expect(
      typeof blockerOptions?.enableBeforeUnload === "function"
        ? blockerOptions.enableBeforeUnload()
        : blockerOptions?.enableBeforeUnload,
    ).toBe(true);

    resolveCommit({ status: "ok", data: makeResult() });
    await waitFor(() => {
      expect(result.current.state.status).toBe("result");
    });
  });

  it("REQ-401 / UI-07-D15 (6): rollback 中も useBlocker と beforeunload で result 画面を離れない", async () => {
    let resolveRollback!: (
      value: Awaited<ReturnType<typeof commands.rollbackDailyReportImport>>,
    ) => void;
    mockRollback.mockReturnValueOnce(
      new Promise((resolve) => {
        resolveRollback = resolve;
      }),
    );
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const { result } = renderHook(() => useDailyReportImportFlow(), {
      wrapper: makeWrapper(queryClient),
    });
    const lastBlocker = () =>
      mockUseBlocker.mock.calls[mockUseBlocker.mock.calls.length - 1]?.[0] as
        | { shouldBlockFn: () => boolean; enableBeforeUnload: () => boolean }
        | undefined;

    await chooseAndConfirmThree(() => result.current);
    await waitFor(() => {
      expect(result.current.state.status).toBe("preview");
    });
    act(() => {
      result.current.confirmImport(false);
    });
    await waitFor(() => {
      expect(result.current.state.status).toBe("result");
    });
    expect(lastBlocker()?.shouldBlockFn()).toBe(false);

    act(() => {
      result.current.rollback(501);
    });
    await waitFor(() => {
      expect(result.current.isRollingBack).toBe(true);
    });
    expect(lastBlocker()?.shouldBlockFn()).toBe(true);
    expect(lastBlocker()?.enableBeforeUnload()).toBe(true);

    resolveRollback({ status: "ok", data: makeRollback() });
    await waitFor(() => {
      expect(result.current.state.status).toBe("idle");
    });
    expect(lastBlocker()?.shouldBlockFn()).toBe(false);
  });
});
