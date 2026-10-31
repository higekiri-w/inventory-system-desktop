// REQ-401 / UI-07-D16（D-111）: 予備の経路「ファイルを選んで取り込む」。固定の文は 55 UI-07-D16 から写す
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { commands, type DailyReportPreviewData } from "@/lib/bindings";
import { open } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import { DailyReportImportPage } from "./DailyReportImportPage";

vi.mock("@tanstack/react-router", () => ({
  useBlocker: vi.fn(),
  Link: ({ children }: { children: ReactNode }) => <a href="/reports/daily">{children}</a>,
}));

vi.mock("sonner", () => ({
  toast: { error: vi.fn(), success: vi.fn() },
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
vi.mock("@tauri-apps/plugin-fs", () => ({ readFile: vi.fn() }));

vi.mock("@/lib/bindings", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/bindings")>();
  return {
    ...actual,
    commands: {
      scanRegisterSd: vi.fn(),
      parseAndValidateDailyReportFromSd: vi.fn(),
      parseAndValidateDailyReport: vi.fn(),
      commitDailyReportImport: vi.fn(),
      rollbackDailyReportImport: vi.fn(),
    },
  };
});

const mockOpen = vi.mocked(open);
const mockReadFile = vi.mocked(readFile);
const mockParse = vi.mocked(commands.parseAndValidateDailyReport);
const mockCommit = vi.mocked(commands.commitDailyReportImport);

function makePreview(): DailyReportPreviewData {
  return {
    file_info: {
      report_date: "2026-03-21",
      bundle_hash: "a".repeat(64),
      source_files: [],
    },
    totals: { gross_amount: 12000, net_amount: 11000 },
    payment_summary: [],
    department_summary: [],
    warnings: [],
    duplicate_check: { status: "NoDuplicate", same_date_imports: [] },
    preview_created_at: "2026-03-21T10:00:00",
  };
}

function renderPage() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  render(
    <QueryClientProvider client={queryClient}>
      <DailyReportImportPage />
    </QueryClientProvider>,
  );
  return userEvent.setup();
}

async function choose(user: ReturnType<typeof userEvent.setup>, paths: string[] | null) {
  mockOpen.mockResolvedValueOnce(paths);
  await user.click(screen.getByRole("button", { name: "ファイルを選んで取り込む" }));
}

function listedNames(): string[] {
  const list = screen.queryByRole("list", { name: "選んだファイル" });
  if (!list) return [];
  return within(list)
    .getAllByRole("listitem")
    .map((item) => (item.textContent ?? "").replace("外す", ""));
}

beforeEach(() => {
  vi.clearAllMocks();
  mockReadFile.mockResolvedValue(new Uint8Array([7]));
  mockParse.mockResolvedValue({
    status: "ok",
    data: { preview_data: makePreview(), preview_token: "preview-token" },
  });
});

describe("DailyReportImportPage files UI-07-D16 REQ-401", () => {
  it("REQ-401 / UI-07-D16 (b)(e): files are added one by one; 確認する needs all three and 外す disables it again", async () => {
    const user = renderPage();
    await choose(user, ["C:\\a\\Z001_21 .CSV"]);
    await choose(user, ["C:\\a\\Z005_21 .CSV"]);
    await waitFor(() => {
      expect(listedNames()).toHaveLength(2);
    });
    expect(screen.getByRole("button", { name: "確認する" })).toBeDisabled();
    await choose(user, ["C:\\a\\Z002_21 .CSV"]);
    await waitFor(() => {
      expect(listedNames()).toEqual(["Z001Z001_21 .CSV", "Z002Z002_21 .CSV", "Z005Z005_21 .CSV"]);
    });
    expect(screen.getByRole("button", { name: "確認する" })).toBeEnabled();

    const list = screen.getByRole("list", { name: "選んだファイル" });
    await user.click(within(list).getAllByRole("button", { name: "外す" })[1]);
    expect(listedNames()).toEqual(["Z001Z001_21 .CSV", "Z005Z005_21 .CSV"]);
    expect(screen.getByRole("button", { name: "確認する" })).toBeDisabled();
    expect(mockParse).not.toHaveBeenCalled();
  });

  it("REQ-401 / UI-07-D16 (c): an undeterminable name and a duplicated series are refused with fixed texts", async () => {
    const user = renderPage();
    await choose(user, ["C:\\a\\notes.CSV"]);
    expect(
      await screen.findByText("Z001・Z002・Z005 のどれかの名前のファイルを選んでください。"),
    ).toBeInTheDocument();
    expect(listedNames()).toEqual([]);

    await choose(user, ["C:\\a\\Z001_a.CSV", "C:\\a\\z001_b.csv"]);
    expect(
      await screen.findByText(
        "Z001・Z002・Z005 は 1 つずつ選んでください（同じ種類のファイルが 2 つ以上あります）。",
      ),
    ).toBeInTheDocument();
    expect(listedNames()).toEqual([]);
  });

  it("REQ-401 / UI-07-D16 (d): choosing the same series asks before replacing; cancel keeps the list", async () => {
    const user = renderPage();
    await choose(user, ["C:\\a\\Z001_old.CSV"]);
    await waitFor(() => {
      expect(listedNames()).toEqual(["Z001Z001_old.CSV"]);
    });

    await choose(user, ["C:\\a\\Z001_new.CSV"]);
    const dialog = await screen.findByRole("alertdialog");
    expect(within(dialog).getByText("Z001 のファイルを置き換えますか？")).toBeInTheDocument();
    expect(dialog).toHaveTextContent(
      "選んであった Z001_old.CSV を外して、Z001_new.CSV を使います。",
    );
    await user.click(within(dialog).getByRole("button", { name: "キャンセル" }));
    await waitFor(() => {
      expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    });
    expect(listedNames()).toEqual(["Z001Z001_old.CSV"]);

    await choose(user, ["C:\\a\\Z001_new.CSV"]);
    await user.click(
      within(await screen.findByRole("alertdialog")).getByRole("button", { name: "置き換える" }),
    );
    await waitFor(() => {
      expect(listedNames()).toEqual(["Z001Z001_new.CSV"]);
    });
  });

  it.each(["置き換える", "キャンセル"])(
    "REQ-401 / UI-07-D16 (d): two replacements from one dialog are asked one by one (%s advances only one)",
    async (answer) => {
      const user = renderPage();
      await choose(user, ["C:\\a\\Z001_old.CSV", "C:\\a\\Z002_old.CSV"]);
      await waitFor(() => {
        expect(listedNames()).toEqual(["Z001Z001_old.CSV", "Z002Z002_old.CSV"]);
      });

      await choose(user, ["D:\\b\\Z001_new.CSV", "D:\\b\\Z002_new.CSV"]);
      const first = await screen.findByRole("alertdialog");
      expect(within(first).getByText("Z001 のファイルを置き換えますか？")).toBeInTheDocument();
      await user.click(within(first).getByRole("button", { name: answer }));

      const second = await screen.findByRole("alertdialog");
      expect(within(second).getByText("Z002 のファイルを置き換えますか？")).toBeInTheDocument();
      await user.click(within(second).getByRole("button", { name: "キャンセル" }));
      await waitFor(() => {
        expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
      });
      expect(listedNames()).toEqual([
        answer === "置き換える" ? "Z001Z001_new.CSV" : "Z001Z001_old.CSV",
        "Z002Z002_old.CSV",
      ]);
    },
  );

  it("REQ-401 / UI-07-D16 (a)(c): reselecting from a manual preview with no addable file returns to an empty list", async () => {
    const user = renderPage();
    await choose(user, ["C:\\a\\Z001_21 .CSV", "C:\\a\\Z002_21 .CSV", "C:\\a\\Z005_21 .CSV"]);
    await user.click(await screen.findByRole("button", { name: "確認する" }));
    await screen.findByText("取込み内容");

    mockOpen.mockResolvedValueOnce(["D:\\b\\notes.CSV"]);
    await user.click(screen.getByRole("button", { name: "ファイルを選び直す" }));

    expect(
      await screen.findByText("Z001・Z002・Z005 のどれかの名前のファイルを選んでください。"),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "ファイルを選んで取り込む" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "取り込む" })).not.toBeInTheDocument();
    expect(listedNames()).toEqual([]);
  });

  it("REQ-401 / UI-07-D16 (a): reselecting from a manual preview starts a new list with only the chosen files", async () => {
    const user = renderPage();
    await choose(user, ["C:\\a\\Z001_21 .CSV", "C:\\a\\Z002_21 .CSV", "C:\\a\\Z005_21 .CSV"]);
    await user.click(await screen.findByRole("button", { name: "確認する" }));
    await screen.findByText("取込み内容");
    expect(mockParse).toHaveBeenCalledTimes(1);

    // キャンセルなら preview のまま
    mockOpen.mockResolvedValueOnce(null);
    await user.click(screen.getByRole("button", { name: "ファイルを選び直す" }));
    expect(mockOpen).toHaveBeenCalledTimes(2);
    expect(screen.getByText("取込み内容")).toBeInTheDocument();
    expect(screen.getByText("2026-03-21")).toBeInTheDocument();

    mockOpen.mockResolvedValueOnce(["D:\\b\\Z001_22 .CSV"]);
    await user.click(screen.getByRole("button", { name: "ファイルを選び直す" }));
    await waitFor(() => {
      expect(listedNames()).toEqual(["Z001Z001_22 .CSV"]);
    });
    expect(mockParse).toHaveBeenCalledTimes(1);

    await choose(user, ["D:\\b\\Z002_22 .CSV", "D:\\b\\Z005_22 .CSV"]);
    await user.click(await screen.findByRole("button", { name: "確認する" }));
    await screen.findByText("取込み内容");
    expect(mockParse).toHaveBeenLastCalledWith([
      { filename: "Z001_22 .CSV", file_bytes: [7], source_path: "D:\\b\\Z001_22 .CSV" },
      { filename: "Z002_22 .CSV", file_bytes: [7], source_path: "D:\\b\\Z002_22 .CSV" },
      { filename: "Z005_22 .CSV", file_bytes: [7], source_path: "D:\\b\\Z005_22 .CSV" },
    ]);
  });

  it("REQ-401 / UI-07-D15 (6): a manually chosen import result has no SD の一覧へ戻る", async () => {
    mockCommit.mockResolvedValueOnce({
      status: "ok",
      data: {
        daily_report_import_id: 9,
        status: "completed",
        report_date: "2026-03-21",
        gross_amount: 12000,
        net_amount: 11000,
        warning_count: 0,
      },
    });
    const user = renderPage();
    await choose(user, ["C:\\a\\Z001_21 .CSV", "C:\\a\\Z002_21 .CSV", "C:\\a\\Z005_21 .CSV"]);
    await user.click(await screen.findByRole("button", { name: "確認する" }));
    await user.click(await screen.findByRole("button", { name: "取り込む" }));
    await screen.findByText("日報取込み完了");
    expect(screen.queryByRole("button", { name: "SD の一覧へ戻る" })).not.toBeInTheDocument();
  });
});
