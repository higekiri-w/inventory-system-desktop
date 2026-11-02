// src/lib/hooks/useExportFile.test.ts
//
// UI-09a/b §57.5: useExportFile の異常系。
// describe-error-adoption packet（2026-08-04）AC3 / B4 是正: onError toast の引数が
// describeError 出力であること（InvokeError のデバッグ文字列 `[commands: ...]` が
// 利用者向け toast へ漏れないこと）を assert する。

import { createElement } from "react";
import type { ReactNode } from "react";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { toast } from "sonner";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { commands } from "@/lib/bindings";

import { useExportFile } from "./useExportFile";

vi.mock("@/lib/bindings", () => ({
  commands: {
    exportSalesCsv: vi.fn(),
  },
}));

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

const mockExportSalesCsv = vi.mocked(commands.exportSalesCsv);
const mockToastError = vi.mocked(toast.error);

// .ts（非 .tsx）ファイル内で JSX 構文を避けるため createElement を使う
// （packet Matrix が本 file を `useExportFile.test.ts` と明示命名しているため）。
function makeWrapper() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return function Wrapper({ children }: { children: ReactNode }) {
    return createElement(QueryClientProvider, { client: qc }, children);
  };
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("useExportFile (UI-09a/b §57.5, B4 describeError adoption)", () => {
  it("shows describeError output in the error toast, not the raw InvokeError debug message", async () => {
    mockExportSalesCsv.mockResolvedValue({
      status: "error",
      error: {
        kind: "internal",
        message: "CSV出力に失敗しました",
        field: null,
        error_id: "E-20260501-090000-syn4",
        stocktake_recovery: null,
      },
    });

    const { result } = renderHook(() => useExportFile(), { wrapper: makeWrapper() });
    result.current.exportFile({ reportType: "monthly_by_product", target: "2026-05" });

    await waitFor(() => {
      expect(mockToastError).toHaveBeenCalled();
    });

    const [message] = mockToastError.mock.calls[0];
    expect(message).toBe(
      "出力に失敗しました: CSV出力に失敗しました（エラーID: E-20260501-090000-syn4）。詳細は診断ログに記録されています。",
    );
    expect(message).not.toMatch(/\[commands:/);
  });

  it("passes through the message unchanged for a non-internal kind (compatibility, describeError 素通し戦略)", async () => {
    mockExportSalesCsv.mockResolvedValue({
      status: "error",
      error: {
        kind: "export_error",
        message: "対象期間のデータがありません",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });

    const { result } = renderHook(() => useExportFile(), { wrapper: makeWrapper() });
    result.current.exportFile({ reportType: "daily", target: "2026-05-01" });

    await waitFor(() => {
      expect(mockToastError).toHaveBeenCalled();
    });

    const [message] = mockToastError.mock.calls[0];
    expect(message).toBe("出力に失敗しました: 対象期間のデータがありません");
  });
});
