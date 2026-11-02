import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, createRouter, RouterProvider } from "@tanstack/react-router";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { commands, type CmdError } from "@/lib/bindings";
import { Route as CsvImportIndexRoute } from "@/routes/csv-import/index";
import { routeTree } from "@/routeTree.gen";
import { renderWithRouter } from "@/test/render-with-router";
import { CsvImportPage } from "./CsvImportPage";
import { useCsvImportFlow } from "./hooks/useCsvImportFlow";
import { csvImportSearchSchema } from "./types";

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ setTitle: vi.fn().mockResolvedValue(undefined) }),
}));

vi.mock("@/features/daily-report-import/DailyReportImportPage", () => ({
  DailyReportImportPage: () => <div>daily report req401 content</div>,
}));

vi.mock("./hooks/useCsvImportFlow", () => ({ useCsvImportFlow: vi.fn() }));

vi.mock("@/lib/bindings", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/bindings")>();
  return { ...actual, commands: { parseAndValidateCsv: vi.fn() } };
});

const mockParse = vi.mocked(commands.parseAndValidateCsv);

/** 実の route tree で `path` を開く（route の validateSearch を通る） */
function renderRoute(path: string) {
  const router = createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: [path] }),
  });
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
}

type Flow = ReturnType<typeof useCsvImportFlow>;

function flow(overrides: Partial<Flow> = {}): Flow {
  return {
    state: { status: "idle" },
    selectFile: vi.fn(),
    confirmImport: vi.fn(),
    rollback: vi.fn(),
    rollbackResult: null,
    dismissError: vi.fn(),
    isParsing: false,
    isImporting: false,
    isRollingBack: false,
    ...overrides,
  } as Flow;
}

beforeEach(() => {
  vi.mocked(useCsvImportFlow).mockReset();
  vi.mocked(useCsvImportFlow).mockImplementation(() => flow());
});

describe("CsvImportPage_req401", () => {
  it("REQ-401: opens as sales import with daily report default tab and Z004 tab label", () => {
    render(<CsvImportPage />);

    expect(screen.getByRole("heading", { name: "売上データ取込み" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "日報取込み" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(screen.getByRole("tab", { name: "商品別CSV取込み（Z004）" })).toBeInTheDocument();
    expect(screen.getByText("daily report req401 content")).toBeInTheDocument();
  });

  it("U16 stocktake and z004 writes are enabled", async () => {
    // 停止の案内が無く、Z004 の「取り込む」が押せる（SPEC-STOP-D6 で 5 つの入口を一度に外した）
    const user = userEvent.setup();
    const confirmImport = vi.fn();
    const previewFlow = flow({
      state: {
        status: "preview",
        preview: {
          file_info: {
            filename: "Z004_0001.CSV",
            settlement_date: "2026-03-21",
            file_hash: "b".repeat(64),
          },
          matched_summary: { count: 1, total_amount: 500, warnings: [] },
          error_summary: { count: 0, items: [] },
          settlement_gaps: [],
          settlement_date_after_today: false,
          preview_created_at: "2026-03-21T10:00:00",
        },
        previewToken: "token-1",
        filename: "Z004_0001.CSV",
      },
      confirmImport,
    });
    vi.mocked(useCsvImportFlow).mockImplementation(() => previewFlow);
    renderWithRouter(<CsvImportPage />);

    await user.click(await screen.findByRole("tab", { name: "商品別CSV取込み（Z004）" }));

    expect(screen.queryByText(/停止中/)).not.toBeInTheDocument();
    expect(screen.getByText("紐付け結果")).toBeInTheDocument();
    const importButton = screen.getByRole("button", { name: "取り込む" });
    expect(importButton).toBeEnabled();
    await user.click(importButton);
    expect(confirmImport).toHaveBeenCalledTimes(1);
  });

  it("REQ-401: tab=z004 で Z004 タブを開き、不正な tab は既定、取り込んだ記録の link は種別 CSV取込み", async () => {
    // 55 UI-07-D18・UI-STATE-D2（Matrix U8）: route は feature の schema をそのまま validateSearch に渡す
    expect(CsvImportIndexRoute.options.validateSearch).toBe(csvImportSearchSchema);

    const { unmount } = renderRoute("/csv-import?tab=z004");
    expect(await screen.findByRole("tab", { name: "商品別CSV取込み（Z004）" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(
      screen.getByRole("link", { name: "取り込んだ記録を見る（取り消すときはここから）" }),
    ).toHaveAttribute("href", "/inventory/records?recordType=csv_import");
    unmount();

    renderRoute("/csv-import?tab=zzz");
    expect(await screen.findByRole("tab", { name: "日報取込み" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(screen.getByText("daily report req401 content")).toBeInTheDocument();
  });

  it("REQ-401: 取消の応答を待つ間はタブを切り替えられない", async () => {
    // 55 §55.7（Matrix U14）: タブの切替えは router の遷移でなく blocker が効かない。
    // Radix の TabsContent は選ばれていないタブを unmount し、Z004 の flow ごと消えるので disabled で止める
    const user = userEvent.setup();
    const resultState = {
      status: "result" as const,
      result: {
        csv_import_id: 401,
        status: "completed" as const,
        total_items: 1,
        total_amount: 500,
        skipped_count: 0,
        settlement_gaps: [],
        settlement_date_after_today: false,
        recheck_targets: [],
      },
      settlementDate: "2026-03-21",
      filename: "Z004_0001.CSV",
    };
    const z004Tab = () => screen.getByRole("tab", { name: "商品別CSV取込み（Z004）" });
    const dailyTab = () => screen.getByRole("tab", { name: "日報取込み" });

    vi.mocked(useCsvImportFlow).mockImplementation(() =>
      flow({ state: resultState, isRollingBack: true }),
    );
    const { unmount } = renderRoute("/csv-import?tab=z004");
    await waitFor(() => {
      expect(dailyTab()).toBeDisabled();
    });
    expect(z004Tab()).toBeDisabled();
    await user.click(dailyTab());
    expect(z004Tab()).toHaveAttribute("aria-selected", "true");
    expect(screen.queryByText("daily report req401 content")).toBeNull();
    unmount();

    vi.mocked(useCsvImportFlow).mockImplementation(() =>
      flow({ state: resultState, isRollingBack: false }),
    );
    renderRoute("/csv-import?tab=z004");
    expect(await screen.findByRole("tab", { name: "商品別CSV取込み（Z004）" })).toBeEnabled();
    expect(dailyTab()).toBeEnabled();
    await user.click(dailyTab());
    expect(dailyTab()).toHaveAttribute("aria-selected", "true");
    expect(screen.getByText("daily report req401 content")).toBeInTheDocument();
  });

  it("REQ-401: 同じ精算の別の版の拒否の Alert から、名指した取込みの詳細へ進める", async () => {
    // 55 UI-07-D18 の順 4 の拒否からの導線（Matrix U11）: link を出すかと ID は related_csv_import_id だけで決め、
    // 文から読まない。parseAndValidateCsv の拒否が実の flow を通って ErrorState に出る
    const actual = await vi.importActual<typeof import("./hooks/useCsvImportFlow")>(
      "./hooks/useCsvImportFlow",
    );
    vi.mocked(useCsvImportFlow).mockImplementation(actual.useCsvImportFlow);
    const order4 = (id: number) =>
      `同じ精算と思われる別の資料が既に取り込まれています（精算 101、取込みID: ${String(id)}）`;
    const order3 = "このファイルは取込み済みです（取込みID: 5、取込み日: 2026-10-01T10:00:00）";
    const cases: {
      message: string;
      related: number | null | undefined;
      expected: number | null;
      absent?: number;
    }[] = [
      { message: order4(7), related: 7, expected: 7 },
      { message: order4(9), related: 7, expected: 7, absent: 9 },
      { message: order4(7), related: null, expected: null },
      { message: order3, related: undefined, expected: null },
      { message: order3, related: 7, expected: 7, absent: 5 },
      { message: "合成の拒否の文", related: 7, expected: 7 },
    ];

    for (const testCase of cases) {
      const error: CmdError = {
        kind: "import_error",
        message: testCase.message,
        field: null,
        error_id: null,
        stocktake_recovery: null,
      };
      if (testCase.related !== undefined) error.related_csv_import_id = testCase.related;
      mockParse.mockResolvedValueOnce({ status: "error", error });
      const { unmount } = renderRoute("/csv-import?tab=z004");
      const zone = await screen.findByTestId("file-picker-dropzone");
      fireEvent.drop(zone, {
        dataTransfer: { files: [new File([new Uint8Array([1, 2, 3])], "z004-0101.csv")] },
      });
      const alert = await screen.findByRole("alert");
      await waitFor(() => {
        expect(within(alert).getByText("プレビューが利用できません")).toBeInTheDocument();
      });
      expect(within(alert).getByText(testCase.message)).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "最初に戻る" })).toBeInTheDocument();
      const recordLinks = screen
        .queryAllByRole("link")
        .filter((link) => link.getAttribute("href")?.startsWith("/csv-import/records/"));
      if (testCase.expected === null) {
        expect(within(alert).queryByRole("link")).toBeNull();
        expect(recordLinks).toHaveLength(0);
      } else {
        const link = within(alert).getByRole("link", {
          name: `取込みID ${String(testCase.expected)} の記録を見る（取り消すときはここから）`,
        });
        const href = new URL(link.getAttribute("href") ?? "", "http://localhost");
        expect(href.pathname).toBe(`/csv-import/records/${String(testCase.expected)}`);
        expect(href.searchParams.get("returnTo")).toBe("/csv-import?tab=z004");
        if (testCase.absent !== undefined) {
          expect(
            recordLinks.some((l) =>
              l.getAttribute("href")?.startsWith(`/csv-import/records/${String(testCase.absent)}`),
            ),
          ).toBe(false);
        }
      }
      unmount();
    }
  });
});
