import { QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRouter,
  Link,
  Outlet,
  RouterProvider,
} from "@tanstack/react-router";
import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { commands } from "@/lib/bindings";
import type { EjImportResult, EjRecordDay, EjRecordRef, EjRecordSummary } from "@/lib/bindings";
import { createAppQueryClient } from "@/lib/query-client";
import { queryKeys } from "@/lib/query-keys";
import { routeTree } from "@/routeTree.gen";

// 本物の画面のまま、日報の画面へ移る link だけを持つ枠にする
vi.mock("@/components/layout/RootLayout", () => ({
  RootLayout: () => (
    <>
      <nav>
        <Link to="/csv-import" search={{ tab: "daily-report" }}>
          日報へ
        </Link>
      </nav>
      <Outlet />
    </>
  ),
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
vi.mock("@tauri-apps/plugin-fs", () => ({ readFile: vi.fn() }));
vi.mock("sonner", () => ({ toast: { success: vi.fn(), error: vi.fn() } }));
vi.mock("@/lib/bindings", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/bindings")>();
  return {
    ...actual,
    commands: {
      ...actual.commands,
      listEjRecords: vi.fn(),
      findEjRecordsByNo: vi.fn(),
      getEjRecord: vi.fn(),
      scanRegisterSd: vi.fn(),
      commitRegisterSdBulkImport: vi.fn(),
    },
  };
});

const ok = <T,>(data: T) => ({ status: "ok" as const, data });
const row: EjRecordSummary = {
  id: 1,
  machine_prefix: "0001",
  record_no: "000102",
  printed_at: "2026-11-19 10:00",
  kind: "sale",
  mode: "normal",
  total_amount: 1,
};
const theDay: EjRecordDay = {
  date: "2026-11-19",
  prev_date: null,
  next_date: null,
  records: [row],
  gaps: [],
  truncated: false,
};
const imported: EjImportResult = {
  inserted_count: 1,
  already_count: 0,
  conflict_count: 0,
  conflicts: [],
  held_count: 0,
  unreadable_files: [],
  gaps: [],
};

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(commands.listEjRecords).mockResolvedValue(ok(theDay));
  vi.mocked(commands.getEjRecord).mockResolvedValue(ok({ summary: row, body_text: "body id1" }));
  vi.mocked(commands.scanRegisterSd).mockResolvedValue(
    ok({
      scan_token: "scan-1",
      scan: {
        volume_label: "E:",
        scanned_from: "2026-11-19",
        candidates: [],
        skipped_unknown_count: 0,
        tidy_failure: null,
        ej: { new_count: 1, held_count: 0, conflict_count: 0, unreadable_file_count: 0 },
      },
    }),
  );
});

describe("UI-09c-D3 番号で探す × 55 UI-07-D17 の一括（C26） REQ-908", () => {
  it.each([
    ["C26 の前", false],
    ["C26 の後", true],
  ])(
    "U6: 控えの画面で走っていた番号の読取りが取込み前の 0 件を %s に返しても、一括の成功の後に戻って同じ番号を探すと読み直して控えを開く（実の Router・本番の QueryClient）",
    async (_label, lateReply) => {
      let resolvePreImport!: (
        value: Awaited<ReturnType<typeof commands.findEjRecordsByNo>>,
      ) => void;
      const find = vi.mocked(commands.findEjRecordsByNo);
      find
        .mockReturnValueOnce(
          new Promise((resolve) => {
            resolvePreImport = resolve;
          }),
        )
        .mockResolvedValue(ok<EjRecordRef[]>([{ id: 1, date: "2026-11-19" }]));
      let resolveBulk!: (
        value: Awaited<ReturnType<typeof commands.commitRegisterSdBulkImport>>,
      ) => void;
      vi.mocked(commands.commitRegisterSdBulkImport).mockReturnValueOnce(
        new Promise((resolve) => {
          resolveBulk = resolve;
        }),
      );
      const queryClient = createAppQueryClient();
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ["/reports/receipts"] }),
      });
      render(
        <QueryClientProvider client={queryClient}>
          <RouterProvider router={router} />
        </QueryClientProvider>,
      );
      const user = userEvent.setup();
      await screen.findByRole("grid");
      await user.type(screen.getByLabelText("番号で探す"), "102{Enter}");
      await waitFor(() => {
        expect(find).toHaveBeenCalledTimes(1);
      });

      // 応答を待ったまま日報の画面へ移り、一括の取込みを始める
      await user.click(screen.getByRole("link", { name: "日報へ" }));
      await user.click(await screen.findByRole("button", { name: "SD から読む" }));
      await user.click(await screen.findByRole("button", { name: "まとめて取り込む" }));
      await waitFor(() => {
        expect(commands.commitRegisterSdBulkImport).toHaveBeenCalledTimes(1);
      });
      expect(queryClient.getQueryState(queryKeys.ejRecords.byNo("102"))?.fetchStatus).toBe(
        "fetching",
      );

      if (!lateReply) {
        act(() => {
          resolvePreImport(ok<EjRecordRef[]>([]));
        });
      }
      act(() => {
        resolveBulk(
          ok({ items: [{ source: { type: "Ej" }, outcome: { type: "ImportedEj", ...imported } }] }),
        );
      });
      await screen.findByRole("link", { name: "レシートの控えを見る" });
      if (lateReply) {
        act(() => {
          resolvePreImport(ok<EjRecordRef[]>([]));
        });
        await waitFor(() => {
          expect(queryClient.getQueryState(queryKeys.ejRecords.byNo("102"))?.fetchStatus).toBe(
            "idle",
          );
        });
      }

      await user.click(screen.getByRole("link", { name: "レシートの控えを見る" }));
      await screen.findByRole("grid");
      await user.type(screen.getByLabelText("番号で探す"), "102{Enter}");
      expect(await screen.findByText("body id1")).toBeInTheDocument();
      expect(screen.queryByText("その番号の控えはありません")).not.toBeInTheDocument();
      expect(find).toHaveBeenCalledTimes(2);
    },
  );
});
