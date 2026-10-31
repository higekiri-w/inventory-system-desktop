// src/features/daily-sales/components/ExportBar.test.tsx
//
// UI-09a-D17 日報を印刷: 4 つの状態の文・リンク・再試行と、押したときの順（読み直し → 描き直し → 印刷）。
// 期待の文字列は 56 UI-09a-D17 の表から転記する（production の定数を import しない）。

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { commands } from "@/lib/bindings";
import type { DailyReportSheet } from "@/lib/bindings";
import { renderWithRouter } from "@/test/render-with-router";

import { useDailyReportSheet } from "../hooks/useDailyReportSheet";
import { ExportBar } from "./ExportBar";

vi.mock("@/lib/bindings", () => ({
  commands: {
    getDailyReportSheet: vi.fn(),
  },
}));

const mockGetSheet = vi.mocked(commands.getDailyReportSheet);

const NOT_IMPORTED = "この日のレジ日報は未取込みのため、印刷できません。";
const LABEL_CHANGED_LINE =
  "支払集計（Z002）：この日の精算の間で、帳票の行の名称か並びが変わっています。同じ名称の行が 2 行あれば、別々に足しています。";
const UNSUMMED_LINE =
  "支払集計（Z002）：この日の精算の間で、帳票の行の名称か並びが変わり、1 回の精算の中に同じ名称の行が 2 行以上あります。別の項目を足さないよう、この帳票は足さずに取込みごとに載せています。";

function buildSheet(overrides: Partial<DailyReportSheet> = {}): DailyReportSheet {
  return {
    date: "2026-03-21",
    import_count: 1,
    summary_rows: [
      { position: 1, label: "総売", quantity: null, count: null, amount: 100, import_seq: null },
    ],
    payment_rows: [],
    department_rows: [],
    label_changed_sources: [],
    unsummed_sources: [],
    plu_rows: [],
    plu_import: { import_count: 0, skipped_count: 0 },
    ...overrides,
  };
}

function ok(data: DailyReportSheet) {
  return { status: "ok" as const, data };
}

const failure = {
  status: "error" as const,
  error: {
    kind: "internal" as const,
    message: "日報の紙の内容を取得できませんでした",
    field: null,
    error_id: null,
    stocktake_recovery: null,
  },
};

function Harness() {
  const sheetQuery = useDailyReportSheet("2026-03-21");
  return (
    <ExportBar
      onExportCsv={vi.fn()}
      isExporting={false}
      date="2026-03-21"
      sheetQuery={sheetQuery}
    />
  );
}

function renderBar() {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  return renderWithRouter(
    <QueryClientProvider client={qc}>
      <Harness />
    </QueryClientProvider>,
  );
}

async function printButton() {
  return screen.findByRole("button", { name: "日報を印刷" });
}

// jsdom の環境に window.print が無いので、test ごとに mock を置く
let printSpy = vi.fn<() => void>();

beforeEach(() => {
  vi.clearAllMocks();
  printSpy = vi.fn<() => void>();
  Object.defineProperty(window, "print", { value: printSpy, configurable: true, writable: true });
});

afterEach(() => {
  vi.useRealTimers();
});

describe("ExportBar UI-09a-D17", () => {
  it("ExportBar disables print with reason when no imports (UI-09a)", async () => {
    mockGetSheet.mockResolvedValue(ok(buildSheet({ import_count: 0, summary_rows: [] })));
    renderBar();

    expect(await screen.findByText(NOT_IMPORTED)).toBeInTheDocument();
    const button = await printButton();
    expect(button).toHaveAttribute("aria-disabled", "true");
    fireEvent.click(button);
    expect(mockGetSheet).toHaveBeenCalledTimes(1);
    expect(printSpy).not.toHaveBeenCalled();
    expect(screen.getByRole("link", { name: "売上データ取込みを開く" })).toHaveAttribute(
      "href",
      "/csv-import",
    );
  });

  it("print is skipped when refetch fails or no imports (UI-09a)", async () => {
    // 読み直しの失敗: 印刷せず CmdError の文と再試行。再試行で次の応答（1 件）なら押せる
    mockGetSheet.mockResolvedValueOnce(ok(buildSheet())).mockResolvedValueOnce(failure);
    renderBar();
    const button = await printButton();
    await waitFor(() => {
      expect(button).toHaveAttribute("aria-disabled", "false");
    });
    fireEvent.click(button);
    expect(await screen.findByText(/日報の紙の内容を取得できませんでした/)).toBeInTheDocument();
    expect(printSpy).not.toHaveBeenCalled();
    expect(button).toHaveAttribute("aria-disabled", "true");

    mockGetSheet.mockResolvedValueOnce(ok(buildSheet()));
    fireEvent.click(screen.getByRole("button", { name: "再試行" }));
    await waitFor(() => {
      expect(button).toHaveAttribute("aria-disabled", "false");
    });
    expect(mockGetSheet).toHaveBeenCalledTimes(3);

    // 読み直しで 0 件: 印刷せず未取込みの文
    mockGetSheet.mockResolvedValueOnce(ok(buildSheet({ import_count: 0, summary_rows: [] })));
    fireEvent.click(button);
    expect(await screen.findByText(NOT_IMPORTED)).toBeInTheDocument();
    expect(printSpy).not.toHaveBeenCalled();
  });

  it("print waits refetch and prints new values (UI-09a)", async () => {
    vi.useFakeTimers({ toFake: ["Date"] });
    vi.setSystemTime(new Date(2026, 2, 21, 18, 5, 30));
    let resolveRefetch: (value: ReturnType<typeof ok>) => void = () => undefined;
    mockGetSheet.mockResolvedValueOnce(ok(buildSheet())).mockReturnValueOnce(
      new Promise((resolve) => {
        resolveRefetch = resolve;
      }),
    );
    let printedText = "";
    printSpy.mockImplementation(() => {
      printedText = screen.getByTestId("daily-report-sheet").textContent ?? "";
    });
    renderBar();
    const button = await printButton();
    await waitFor(() => {
      expect(button).toHaveAttribute("aria-disabled", "false");
    });

    fireEvent.click(button);
    // (a) 読み直しの解決の前には印刷しない
    await waitFor(() => {
      expect(mockGetSheet).toHaveBeenCalledTimes(2);
    });
    expect(printSpy).not.toHaveBeenCalled();

    const updated = buildSheet();
    updated.summary_rows[0].amount = 600;
    resolveRefetch(ok(updated));
    await waitFor(() => {
      expect(printSpy).toHaveBeenCalledTimes(1);
    });
    // (b) 印刷の時点の紙は読み直した値、(c) 印刷の時刻は押した時刻
    expect(printedText).toContain("600");
    expect(printedText).not.toContain("100");
    expect(printedText).toContain("印刷 2026-03-21 18:05");
  });

  it("ExportBar shows skipped PLU rows with link (UI-09a)", async () => {
    mockGetSheet.mockResolvedValue(
      ok(buildSheet({ plu_import: { import_count: 1, skipped_count: 3 } })),
    );
    renderBar();

    expect(await screen.findByText("数えていない PLU の行が 3 行あります")).toBeInTheDocument();
    const link = screen.getByRole("link", { name: "取込みの記録を開く" });
    const url = new URL(link.getAttribute("href") ?? "", "http://localhost");
    expect(url.pathname).toBe("/inventory/records");
    expect(url.searchParams.get("recordType")).toBe("csv_import");
    expect(url.searchParams.get("status")).toBe("active");
    expect(url.searchParams.get("dateFrom")).toBe("2026-03-21");
    expect(url.searchParams.get("dateTo")).toBe("2026-03-21");
    expect(await printButton()).toHaveAttribute("aria-disabled", "false");
  });

  it("ExportBar shows label change notices per source (UI-09a)", async () => {
    mockGetSheet.mockResolvedValueOnce(ok(buildSheet({ label_changed_sources: ["Z002"] })));
    const first = renderBar();
    expect(await screen.findByText(LABEL_CHANGED_LINE)).toBeInTheDocument();
    expect(screen.getAllByText(/^支払集計（Z002）：/)).toHaveLength(1);
    first.unmount();

    mockGetSheet.mockResolvedValueOnce(
      ok(buildSheet({ label_changed_sources: ["Z002"], unsummed_sources: ["Z002"] })),
    );
    const second = renderBar();
    expect(await screen.findByText(UNSUMMED_LINE)).toBeInTheDocument();
    expect(screen.queryByText(LABEL_CHANGED_LINE)).not.toBeInTheDocument();
    second.unmount();

    mockGetSheet.mockResolvedValueOnce(ok(buildSheet()));
    const third = renderBar();
    const button = await printButton();
    await waitFor(() => {
      expect(button).toHaveAttribute("aria-disabled", "false");
    });
    expect(within(third.container).queryByText(/：/)).not.toBeInTheDocument();
  });
});
