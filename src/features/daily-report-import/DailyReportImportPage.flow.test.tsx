import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { toast } from "sonner";
import { commands, type DailyReportPreviewData } from "@/lib/bindings";
import { open } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import { DailyReportImportPage } from "./DailyReportImportPage";

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

const mockOpen = vi.mocked(open);
const mockReadFile = vi.mocked(readFile);
const mockParse = vi.mocked(commands.parseAndValidateDailyReport);
const mockToast = vi.mocked(toast);

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

function renderWithClient(ui: ReactNode) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>);
}

beforeEach(() => {
  vi.clearAllMocks();
  mockOpen.mockReset();
  mockReadFile.mockReset();
  mockReadFile
    .mockResolvedValueOnce(new Uint8Array([1]))
    .mockResolvedValueOnce(new Uint8Array([2]))
    .mockResolvedValueOnce(new Uint8Array([5]));
  mockParse.mockResolvedValue({
    status: "ok",
    data: { preview_data: makePreview(), preview_token: "preview-token-req401" },
  });
});

describe("DailyReportImportPage flow_req401", () => {
  it("REQ-401 / UI-07-D16: 2 files cannot be confirmed; the 3rd file enables parse with source paths", async () => {
    const user = userEvent.setup();
    mockOpen
      .mockResolvedValueOnce(["C:\\tmp\\Z001_260321.CSV", "C:\\tmp\\Z002_260321.CSV"])
      .mockResolvedValueOnce(["C:\\tmp\\Z005_260321.CSV"]);

    renderWithClient(<DailyReportImportPage />);

    await user.click(screen.getByRole("button", { name: "ファイルを選んで取り込む" }));

    const list = await screen.findByRole("list", { name: "選んだファイル" });
    expect(within(list).getAllByRole("listitem")).toHaveLength(2);
    expect(screen.getByRole("button", { name: "確認する" })).toBeDisabled();
    expect(mockParse).not.toHaveBeenCalled();

    await user.click(screen.getByRole("button", { name: "ファイルを選んで取り込む" }));
    await waitFor(() => {
      expect(within(list).getAllByRole("listitem")).toHaveLength(3);
    });
    const confirm = screen.getByRole("button", { name: "確認する" });
    expect(confirm).toBeEnabled();
    expect(mockParse).not.toHaveBeenCalled();
    expect(mockToast.error).not.toHaveBeenCalled();

    await user.click(confirm);

    await waitFor(() => {
      expect(screen.getByText("取込み内容")).toBeInTheDocument();
    });
    expect(mockParse).toHaveBeenCalledWith([
      { filename: "Z001_260321.CSV", file_bytes: [1], source_path: "C:\\tmp\\Z001_260321.CSV" },
      { filename: "Z002_260321.CSV", file_bytes: [2], source_path: "C:\\tmp\\Z002_260321.CSV" },
      { filename: "Z005_260321.CSV", file_bytes: [5], source_path: "C:\\tmp\\Z005_260321.CSV" },
    ]);
  });
});
