// src/features/backup-restore/BackupRestorePage.flow.test.tsx
//
// UI-11b-D11 (batch A packet, SPEC-UIPOLA-D1): 復元成功通知の producer
// (BackupRestorePage) → consumer (ホーム画面) を実 Router + memory history で結線する
// 統合テスト。Matrix C5-C8 に対応。in-memory one-shot flag の寿命契約
// (mount 時取り込み・mount 中表示維持・非表示は unmount 後の再訪/reload/失敗経路のみ) を検証する。

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, createRouter, RouterProvider } from "@tanstack/react-router";
import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { StrictMode } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { routeTree } from "@/routeTree.gen";
import { commands } from "@/lib/bindings";
import type {
  AppSetting,
  BackupInfo,
  BackupInspection,
  CmdErrorKind,
  CsvImport,
  DailySalesReport,
  PaginatedResult,
  ProductResponse,
  ProductWithRelations,
} from "@/lib/bindings";
import { open } from "@tauri-apps/plugin-dialog";
import { clearRestoreSuccessPending } from "@/lib/restore-success-notification";
import { resetAutoBackupCheckForTests } from "./useAutoBackupCheck";
import { queryKeys } from "@/lib/query-keys";

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ setTitle: vi.fn().mockResolvedValue(undefined) }),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

vi.mock("@/lib/bindings", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/bindings")>();
  return {
    ...actual,
    commands: {
      // backup-restore (producer)
      getSettings: vi.fn(),
      updateSetting: vi.fn(),
      createBackup: vi.fn(),
      checkAutoBackup: vi.fn(),
      listBackups: vi.fn(),
      getEffectiveBackupDir: vi.fn(),
      restoreBackup: vi.fn(),
      inspectBackup: vi.fn(),
      // home (consumer)
      getDailySales: vi.fn(),
      listLowStock: vi.fn(),
      listPluDirty: vi.fn(),
      listCsvImports: vi.fn(),
    },
  };
});

const mockGetSettings = vi.mocked(commands.getSettings);
const mockCreateBackup = vi.mocked(commands.createBackup);
const mockCheckAutoBackup = vi.mocked(commands.checkAutoBackup);
const mockListBackups = vi.mocked(commands.listBackups);
const mockGetEffectiveBackupDir = vi.mocked(commands.getEffectiveBackupDir);
const mockRestoreBackup = vi.mocked(commands.restoreBackup);
const mockInspectBackup = vi.mocked(commands.inspectBackup);
const mockOpen = vi.mocked(open);

const mockGetDailySales = vi.mocked(commands.getDailySales);
const mockListLowStock = vi.mocked(commands.listLowStock);
const mockListPluDirty = vi.mocked(commands.listPluDirty);
const mockListCsvImports = vi.mocked(commands.listCsvImports);

function ok<T>(data: T) {
  return { status: "ok" as const, data };
}

function cmdError(message: string, kind: CmdErrorKind = "internal") {
  return {
    status: "error" as const,
    error: { kind, message, field: null, error_id: null, stocktake_recovery: null },
  };
}

const BACKUP_DATE_LABEL = "7月3日 21:00";

function makeBackupInfo(): BackupInfo {
  return {
    file_name: "inventory_backup_20260703_210000.db",
    file_path: "/tmp/backups/inventory_backup_20260703_210000.db",
    size_bytes: 12_400_000,
    created_at: "2026-07-03 21:00:00",
  };
}

function makeInspection(overrides: Partial<BackupInspection> = {}): BackupInspection {
  return {
    file_name: "inventory_backup_20260703_210000.db",
    created_at: "2026-07-03 21:00:00",
    size_bytes: 12_400_000,
    schema_version: 9,
    app_max_version: 9,
    newer_than_app: false,
    quick_check_ok: true,
    product_count: 37,
    last_operation_at: "2026-07-03T20:47:00",
    ...overrides,
  };
}

function makeAppSettings(): AppSetting[] {
  return [
    { key: "backup_enabled", value: "1", updated_at: "2026-07-06T00:00:00" },
    { key: "backup_time", value: "23:00", updated_at: "2026-07-06T00:00:00" },
    { key: "backup_path", value: "/tmp/backups", updated_at: "2026-07-06T00:00:00" },
    { key: "backup_retention_days", value: "3", updated_at: "2026-07-06T00:00:00" },
  ];
}

function makeDailySales(): DailySalesReport {
  return {
    date: "2026-07-28",
    items: [],
    department_subtotals: [],
    grand_total: { count_points: 0, length_cm: 0, amount: 0 },
    official_daily_report: null,
  };
}

function makeProduct(productCode: string, stockQuantity: number): ProductWithRelations {
  return {
    product_code: productCode,
    jan_code: null,
    name: `synthetic-${productCode}`,
    department_id: 1,
    supplier_id: null,
    selling_price: 100,
    cost_price_centi: 50,
    tax_rate: "10",
    maker_code: null,
    stock_quantity: stockQuantity,
    stock_unit: "pcs",
    is_discontinued: false,
    plu_dirty: false,
    plu_exported_at: null,
    plu_target: false,
    pos_stock_sync: true,
    created_at: "2026-01-01T00:00:00",
    updated_at: "2026-01-01T00:00:00",
    department_name: "synthetic-department",
    supplier_name: null,
    plu_memory_no: null,
    stock_recheck: false,
    stock_range_low: null,
    stock_range_high: null,
  };
}

function makePluDirty(): ProductResponse {
  return {
    product_code: "PLU-1",
    jan_code: null,
    name: "synthetic-PLU-1",
    department_id: 1,
    selling_price: 100,
    cost_price_centi: 50,
    stock_unit: "pcs",
    stock_quantity: 1,
    plu_dirty: true,
    plu_exported_at: null,
  };
}

function csvPage(items: CsvImport[]): PaginatedResult<CsvImport> {
  return { items, total_count: items.length, page: 1, per_page: 1 };
}

function installDefaultCommands() {
  mockGetSettings.mockResolvedValue(ok(makeAppSettings()));
  mockListBackups.mockResolvedValue(ok([makeBackupInfo()]));
  mockCreateBackup.mockResolvedValue(
    ok({
      file_name: "inventory_backup_20260706_100000.db",
      file_path: "/tmp/backups/inventory_backup_20260706_100000.db",
      size_bytes: 10_000_000,
    }),
  );
  mockCheckAutoBackup.mockResolvedValue(ok(false));
  mockInspectBackup.mockResolvedValue(ok(makeInspection()));
  mockGetEffectiveBackupDir.mockResolvedValue(ok("/tmp/backups"));
  mockOpen.mockResolvedValue(null);

  mockGetDailySales.mockResolvedValue(ok(makeDailySales()));
  mockListLowStock.mockResolvedValue(ok([makeProduct("OUT", 0)]));
  mockListPluDirty.mockResolvedValue(ok([makePluDirty()]));
  mockListCsvImports.mockResolvedValue(ok(csvPage([])));
}

function renderApp(initialPath: string) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  const router = createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: [initialPath] }),
  });
  const utils = render(
    <StrictMode>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </StrictMode>,
  );
  return { router, queryClient, ...utils };
}

async function driveToRestoreSuccess(user: ReturnType<typeof userEvent.setup>) {
  await screen.findByRole("heading", { name: "バックアップ・復元" });
  const backupDate = await screen.findByText(BACKUP_DATE_LABEL);
  const row = backupDate.closest("tr");
  expect(row).not.toBeNull();
  if (!row) throw new Error("backup row not found");
  await user.click(within(row).getByRole("button", { name: "この控えに戻す" }));
  await user.click(await screen.findByRole("button", { name: "復元の確認へ進む" }));
  await screen.findByRole("alertdialog", { name: "元に戻せません" });
  await user.click(screen.getByRole("button", { name: `${BACKUP_DATE_LABEL} の控えに戻す` }));
}

beforeEach(() => {
  vi.clearAllMocks();
  clearRestoreSuccessPending();
  resetAutoBackupCheckForTests();
  installDefaultCommands();
});

afterEach(() => {
  vi.useRealTimers();
});

describe("自動バックアップの確認 (REQ-901 / UI-11b-D13)", () => {
  it("REQ-901 / UI-11b-D13: バックアップ画面を開いていても確認は 1 分に 1 回", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    renderApp("/settings/backup");
    await screen.findByRole("heading", { name: "バックアップ・復元" });

    await vi.advanceTimersByTimeAsync(60_000);
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(1);

    await vi.advanceTimersByTimeAsync(60_000);
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(2);
  });
});

describe("復元成功 Alert 統合テスト (UI-11b-D11, Matrix C5/C6)", () => {
  it("test_ui11b_d11_restore_success_shows_alert_exactly_once_under_strictmode", async () => {
    const user = userEvent.setup();
    mockRestoreBackup.mockResolvedValue(ok(null));

    renderApp("/settings/backup");
    await driveToRestoreSuccess(user);

    await waitFor(() => {
      expect(screen.getByText(/^昨日の売上 \(/)).toBeInTheDocument();
    });
    expect(screen.getAllByText("バックアップから復元しました")).toHaveLength(1);
  });

  it("test_ui11b_d11_alert_survives_same_mount_rerender", async () => {
    const user = userEvent.setup();
    mockRestoreBackup.mockResolvedValue(ok(null));

    const { queryClient } = renderApp("/settings/backup");
    await driveToRestoreSuccess(user);
    await screen.findByText("バックアップから復元しました");

    // ホーム到達後の query 更新 (他 query 更新を模した re-render) を発生させる。
    mockListLowStock.mockResolvedValue(ok([makeProduct("OUT", 0), makeProduct("LOW", 3)]));
    await act(async () => {
      await queryClient.invalidateQueries();
    });

    await waitFor(() => {
      expect(screen.getAllByText("バックアップから復元しました")).toHaveLength(1);
    });
  });

  it("test_ui11b_d11_alert_hidden_after_unmount_revisit", async () => {
    const user = userEvent.setup();
    mockRestoreBackup.mockResolvedValue(ok(null));

    const { router } = renderApp("/settings/backup");
    await driveToRestoreSuccess(user);
    await screen.findByText("バックアップから復元しました");

    // unmount 後の再訪: 別画面へ遷移してからホームへ戻る。
    await act(async () => {
      await router.navigate({ to: "/settings/backup" });
    });
    await screen.findByRole("heading", { name: "バックアップ・復元" });

    await act(async () => {
      await router.navigate({ to: "/" });
    });

    await waitFor(() => {
      expect(screen.getByText(/^昨日の売上 \(/)).toBeInTheDocument();
    });
    expect(screen.queryByText("バックアップから復元しました")).not.toBeInTheDocument();
  });

  it("test_ui11b_d11_alert_hidden_after_store_reset_reload_substitute", async () => {
    const user = userEvent.setup();
    mockRestoreBackup.mockResolvedValue(ok(null));

    const first = renderApp("/settings/backup");
    await driveToRestoreSuccess(user);
    await screen.findByText("バックアップから復元しました");

    // reload / アプリ再起動の代替 (Matrix Residual Test Gaps): 旧 DOM を破棄し
    // module-scope flag を明示 reset した状態で新規マウントする (reload 相当)。
    first.unmount();
    clearRestoreSuccessPending();

    const { router: router2 } = renderApp("/");
    await waitFor(() => {
      expect(screen.getAllByText(/昨日の売上/)).not.toHaveLength(0);
    });
    expect(router2.state.location.pathname).toBe("/");
    expect(screen.queryByText("バックアップから復元しました")).not.toBeInTheDocument();
  });
});

describe("復元成功 Alert negative パス (UI-11b-D11, Matrix C7/C8)", () => {
  it("test_ui11b_d11_no_flag_direct_home_visit_shows_no_alert", async () => {
    renderApp("/");

    await waitFor(() => {
      expect(screen.getByText(/^昨日の売上 \(/)).toBeInTheDocument();
    });
    expect(screen.queryByText("バックアップから復元しました")).not.toBeInTheDocument();
  });

  it("test_ui11b_d11_restore_failure_then_normal_home_navigation_shows_no_alert", async () => {
    const user = userEvent.setup();
    mockRestoreBackup.mockResolvedValueOnce(
      cmdError("復元対象のファイルを読み込めませんでした", "restore_failed_recovered"),
    );

    const { router } = renderApp("/settings/backup");
    await screen.findByRole("heading", { name: "バックアップ・復元" });
    const backupDate = await screen.findByText(BACKUP_DATE_LABEL);
    const row = backupDate.closest("tr");
    expect(row).not.toBeNull();
    if (!row) throw new Error("backup row not found");
    await user.click(within(row).getByRole("button", { name: "この控えに戻す" }));
    await user.click(await screen.findByRole("button", { name: "復元の確認へ進む" }));
    await screen.findByRole("alertdialog", { name: "元に戻せません" });
    await user.click(screen.getByRole("button", { name: `${BACKUP_DATE_LABEL} の控えに戻す` }));

    // 復元失敗表示のまま (flag 非生成、ホームへは遷移しない)。
    expect(
      await screen.findByText(
        "バックアップの復元に失敗しました。現在のデータには戻しています。もう一度お試しください。",
      ),
    ).toBeInTheDocument();

    // operator が通常操作でホームへ遷移しても Alert は出ない。
    await act(async () => {
      await router.navigate({ to: "/" });
    });

    await waitFor(() => {
      expect(screen.getByText(/^昨日の売上 \(/)).toBeInTheDocument();
    });
    expect(screen.queryByText("バックアップから復元しました")).not.toBeInTheDocument();
  });
});

// ---- UI-11b-D15・D13: 復元の前の控えの確かめと共通の確認の停止（PC の外の控えの runtime (a)） ----

type RestoreResult = Awaited<ReturnType<typeof commands.restoreBackup>>;

const BLOCKED_NEWER =
  "この控えは、より新しい版のアプリで作られています。この版のアプリでは戻せません。新しい版のアプリを入れてから戻してください（今のデータは変わっていません）。";
const BLOCKED_CORRUPT = "この控えは壊れているため戻せません。別の控えを選んでください。";
const BLOCKED_UNREADABLE = "この控えを読めませんでした。別の控えを選んでください。";
const NOTHING_SELECTED = "一覧から戻したい控えを選ぶと、内容と確認手順を表示します。";
const BACKUP_B_LABEL = "7月2日 09:00";

function makeBackupB(): BackupInfo {
  return {
    file_name: "inventory_backup_20260702_090000.db",
    file_path: "/tmp/backups/inventory_backup_20260702_090000.db",
    size_bytes: 11_000_000,
    created_at: "2026-07-02 09:00:00",
  };
}

function deferred<T>() {
  let resolve: (value: T) => void = () => undefined;
  const promise = new Promise<T>((settle) => {
    resolve = settle;
  });
  return { promise, resolve };
}

function setupFakeTimers() {
  vi.useFakeTimers({ shouldAdvanceTime: true });
  return userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
}

async function selectRow(user: ReturnType<typeof userEvent.setup>, label: string) {
  const date = await screen.findByText(label);
  const row = date.closest("tr");
  if (!row) throw new Error("backup row not found");
  await user.click(within(row).getByRole("button", { name: "この控えに戻す" }));
}

async function startRestore(user: ReturnType<typeof userEvent.setup>, label: string) {
  await user.click(await screen.findByRole("button", { name: "復元の確認へ進む" }));
  await screen.findByRole("alertdialog", { name: "元に戻せません" });
  await user.click(screen.getByRole("button", { name: `${label} の控えに戻す` }));
}

async function leavePage(router: ReturnType<typeof renderApp>["router"]) {
  await act(async () => {
    await router.navigate({ to: "/" });
  });
  await screen.findByText(/^昨日の売上 \(/);
}

async function revisitPage(router: ReturnType<typeof renderApp>["router"]) {
  await act(async () => {
    await router.navigate({ to: "/settings/backup" });
  });
  await screen.findByRole("heading", { name: "バックアップ・復元" });
}

async function expectNoCheckWhile(ms: number) {
  const before = mockCheckAutoBackup.mock.calls.length;
  await vi.advanceTimersByTimeAsync(ms);
  expect(mockCheckAutoBackup).toHaveBeenCalledTimes(before);
}

async function expectCheckResumes() {
  const before = mockCheckAutoBackup.mock.calls.length;
  await vi.advanceTimersByTimeAsync(60_000);
  expect(mockCheckAutoBackup).toHaveBeenCalledTimes(before + 1);
}

function listInvalidateCount(spy: { mock: { calls: unknown[][] } }): number {
  return spy.mock.calls.filter(
    ([filters]) =>
      JSON.stringify((filters as { queryKey?: unknown } | undefined)?.queryKey) ===
      JSON.stringify(queryKeys.backupRestore.list()),
  ).length;
}

describe("止めた控え (REQ-901 / UI-11b-D15)", () => {
  it.each([
    [
      "新しすぎる版",
      () =>
        mockInspectBackup.mockResolvedValue(
          ok(
            makeInspection({
              newer_than_app: true,
              schema_version: 10,
              product_count: null,
              last_operation_at: null,
            }),
          ),
        ),
      BLOCKED_NEWER,
    ],
    [
      "壊れた控え",
      () =>
        mockInspectBackup.mockResolvedValue(
          ok(
            makeInspection({ quick_check_ok: false, product_count: null, last_operation_at: null }),
          ),
        ),
      BLOCKED_CORRUPT,
    ],
    [
      "読めない控え（Err）",
      () =>
        mockInspectBackup.mockResolvedValue(
          cmdError("この控えを読めませんでした。別の控えを選んでください。", "validation"),
        ),
      BLOCKED_UNREADABLE,
    ],
    [
      "読めない控え（IPC の例外）",
      () => mockInspectBackup.mockRejectedValue(new Error("synthetic ipc")),
      BLOCKED_UNREADABLE,
    ],
  ] as const)(
    "REQ-901 / UI-11b-D15: %s は固有の文言で止め、事前バックアップと復元の button が無く、「閉じる」で未選択へ戻る",
    async (_label, arrange, message) => {
      const user = userEvent.setup();
      arrange();
      renderApp("/settings/backup");

      await selectRow(user, BACKUP_DATE_LABEL);
      expect(await screen.findByText(message)).toBeInTheDocument();
      expect(screen.queryByText("この控えは戻せます")).not.toBeInTheDocument();
      expect(screen.queryByRole("button", { name: "復元の確認へ進む" })).not.toBeInTheDocument();
      expect(screen.queryByRole("button", { name: "一覧へ戻る" })).not.toBeInTheDocument();
      expect(mockCreateBackup).not.toHaveBeenCalled();

      await user.click(screen.getByRole("button", { name: "閉じる" }));
      expect(screen.getByText(NOTHING_SELECTED)).toBeInTheDocument();
      expect(screen.queryByText(message)).not.toBeInTheDocument();
      expect(mockCreateBackup).not.toHaveBeenCalled();
    },
  );
});

describe("共通の確認の停止と再開 (REQ-901 / UI-11b-D15・D13)", () => {
  it("REQ-901 / UI-11b-D15: 詳細・最終確認・「キャンセル」の間は止め、「閉じる」で再開して一覧を invalidate する", async () => {
    const user = setupFakeTimers();
    const { queryClient } = renderApp("/settings/backup");
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await selectRow(user, BACKUP_DATE_LABEL);
    await screen.findByText("この控えは戻せます");
    await expectNoCheckWhile(120_000);

    await user.click(screen.getByRole("button", { name: "復元の確認へ進む" }));
    await screen.findByRole("alertdialog", { name: "元に戻せません" });
    await expectNoCheckWhile(60_000);

    await user.click(screen.getByRole("button", { name: "キャンセル" }));
    await waitFor(() => {
      expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    });
    expect(screen.getByText("この控えは戻せます")).toBeInTheDocument();
    await expectNoCheckWhile(60_000);

    const listInvalidatesBefore = listInvalidateCount(invalidateSpy);
    await user.click(screen.getByRole("button", { name: "閉じる" }));
    expect(screen.getByText(NOTHING_SELECTED)).toBeInTheDocument();
    expect(listInvalidateCount(invalidateSpy)).toBe(listInvalidatesBefore + 1);
    await expectCheckResumes();
  });

  it("REQ-901 / UI-11b-D15: restore_blocked の「閉じる」で再開する", async () => {
    const user = setupFakeTimers();
    mockInspectBackup.mockResolvedValue(
      ok(makeInspection({ quick_check_ok: false, product_count: null })),
    );
    renderApp("/settings/backup");

    await selectRow(user, BACKUP_DATE_LABEL);
    await screen.findByText(BLOCKED_CORRUPT);
    await expectNoCheckWhile(120_000);

    await user.click(screen.getByRole("button", { name: "閉じる" }));
    await expectCheckResumes();
  });

  it("REQ-901 / UI-11b-D15: pre_backup_failed の「閉じる」で再開し、事前バックアップの失敗の表示を残さない", async () => {
    const user = setupFakeTimers();
    mockCreateBackup.mockResolvedValueOnce(cmdError("synthetic pre-backup failure", "validation"));
    renderApp("/settings/backup");

    await selectRow(user, BACKUP_DATE_LABEL);
    await user.click(await screen.findByRole("button", { name: "復元の確認へ進む" }));
    expect(
      await screen.findByText("復元前のバックアップを作成できませんでした"),
    ).toBeInTheDocument();
    await expectNoCheckWhile(60_000);

    await user.click(screen.getByRole("button", { name: "閉じる" }));
    expect(screen.queryByText("synthetic pre-backup failure")).not.toBeInTheDocument();
    expect(screen.queryByText("操作に失敗しました")).not.toBeInTheDocument();
    expect(
      screen.queryByText("復元前のバックアップを作成できませんでした"),
    ).not.toBeInTheDocument();
    await expectCheckResumes();
  });

  it("REQ-901 / UI-11b-D15: 控えを選んだまま画面を離れると再開する", async () => {
    const user = setupFakeTimers();
    const { router } = renderApp("/settings/backup");

    await selectRow(user, BACKUP_DATE_LABEL);
    await screen.findByText("この控えは戻せます");
    await expectNoCheckWhile(60_000);

    await leavePage(router);
    await expectCheckResumes();
  });

  it.each(["restore_failed_unrecoverable", "restore_durability_unknown"] as const)(
    "REQ-901 / UI-11b-D13: %s の後は離脱 → 再訪 → 確かめ →「閉じる」→ 再離脱でも再開しない",
    async (kind) => {
      const user = setupFakeTimers();
      mockRestoreBackup.mockResolvedValueOnce(cmdError("synthetic fatal", kind));
      const { router } = renderApp("/settings/backup");

      await selectRow(user, BACKUP_DATE_LABEL);
      await startRestore(user, BACKUP_DATE_LABEL);
      await screen.findByText("アプリを閉じて、もう一度開いてください");
      await expectNoCheckWhile(60_000);

      await leavePage(router);
      await expectNoCheckWhile(60_000);
      await revisitPage(router);
      await selectRow(user, BACKUP_DATE_LABEL);
      await screen.findByText("この控えは戻せます");
      await user.click(screen.getByRole("button", { name: "閉じる" }));
      await expectNoCheckWhile(60_000);
      await leavePage(router);
      await expectNoCheckWhile(120_000);
    },
  );

  it.each([
    ["成功", ok(null) as RestoreResult, true],
    [
      "restore_failed_recovered",
      cmdError("synthetic recovered", "restore_failed_recovered") as RestoreResult,
      true,
    ],
    [
      "restore_failed_unrecoverable",
      cmdError("synthetic fatal", "restore_failed_unrecoverable") as RestoreResult,
      false,
    ],
  ] as const)(
    "REQ-901 / UI-11b-D13: 復元の実行中に離脱 → 再訪 → 再離脱しても再開せず、結果（%s）で決まる",
    async (_label, result, resumes) => {
      const user = setupFakeTimers();
      const pendingRestore = deferred<RestoreResult>();
      mockRestoreBackup.mockReturnValueOnce(pendingRestore.promise);
      const { router } = renderApp("/settings/backup");
      try {
        await selectRow(user, BACKUP_DATE_LABEL);
        await startRestore(user, BACKUP_DATE_LABEL);
        await waitFor(() => {
          expect(mockRestoreBackup).toHaveBeenCalledTimes(1);
        });

        await leavePage(router);
        await expectNoCheckWhile(60_000);
        await revisitPage(router);
        await leavePage(router);
        await expectNoCheckWhile(120_000);

        pendingRestore.resolve(result);
        await vi.advanceTimersByTimeAsync(0);
        if (resumes) {
          await expectCheckResumes();
        } else {
          await expectNoCheckWhile(120_000);
        }
      } finally {
        pendingRestore.resolve(ok(null));
        await vi.advanceTimersByTimeAsync(0);
      }
    },
  );

  it.each([
    ["restore_detail", false],
    ["restore_blocked", true],
  ] as const)(
    "REQ-901 / UI-11b-D13・D15: 前の復元の結果は、再訪した画面で選んだ控え B（%s）の停止を外さない",
    async (_label, blockB) => {
      const user = setupFakeTimers();
      const backupB = makeBackupB();
      mockListBackups.mockResolvedValue(ok([makeBackupInfo(), backupB]));
      mockInspectBackup.mockImplementation(({ backup_path }) =>
        Promise.resolve(
          ok(
            backup_path === backupB.file_path && blockB
              ? makeInspection({ newer_than_app: true, product_count: null })
              : makeInspection(),
          ),
        ),
      );
      const pendingRestore = deferred<RestoreResult>();
      mockRestoreBackup.mockReturnValueOnce(pendingRestore.promise);
      const { router } = renderApp("/settings/backup");
      try {
        await selectRow(user, BACKUP_DATE_LABEL);
        await startRestore(user, BACKUP_DATE_LABEL);
        await waitFor(() => {
          expect(mockRestoreBackup).toHaveBeenCalledTimes(1);
        });
        await leavePage(router);
        await revisitPage(router);

        await selectRow(user, BACKUP_B_LABEL);
        await screen.findByText(blockB ? BLOCKED_NEWER : "この控えは戻せます");

        pendingRestore.resolve(cmdError("synthetic recovered", "restore_failed_recovered"));
        await vi.advanceTimersByTimeAsync(0);
        await expectNoCheckWhile(120_000);

        if (!blockB) {
          await user.click(screen.getByRole("button", { name: "復元の確認へ進む" }));
          await screen.findByRole("alertdialog", { name: "元に戻せません" });
          await user.click(screen.getByRole("button", { name: "キャンセル" }));
          await waitFor(() => {
            expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
          });
          await expectNoCheckWhile(60_000);
        }

        await user.click(screen.getByRole("button", { name: "閉じる" }));
        await expectCheckResumes();
      } finally {
        pendingRestore.resolve(ok(null));
        await vi.advanceTimersByTimeAsync(0);
      }
    },
  );
});

describe("手動バックアップの失敗の後の取り直し (REQ-901 / §68.4 手順 4)", () => {
  it("REQ-901 / §68.4: 手動 backup の失敗でも一覧を invalidate して取り直し、失敗の文を出す", async () => {
    const user = userEvent.setup();
    const message =
      "この保存先は PC の中の folder として確かめられません。PC の中の folder を選び直してください";
    mockCreateBackup.mockResolvedValueOnce(cmdError(message, "validation"));
    const { queryClient } = renderApp("/settings/backup");
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    await screen.findByText(BACKUP_DATE_LABEL);
    const listCallsBefore = mockListBackups.mock.calls.length;

    await user.click(screen.getByRole("button", { name: "今すぐバックアップを作成" }));

    expect(await screen.findByText(message)).toBeInTheDocument();
    expect(listInvalidateCount(invalidateSpy)).toBe(1);
    await waitFor(() => {
      expect(mockListBackups.mock.calls.length).toBeGreaterThan(listCallsBefore);
    });
  });
});
