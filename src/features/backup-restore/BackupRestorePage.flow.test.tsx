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
  OffsiteBackupStatus,
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
      checkOffsiteBackup: vi.fn(),
      getOffsiteBackupStatus: vi.fn(),
      prepareOffsiteMedium: vi.fn(),
      resetOffsiteState: vi.fn(),
      stopOffsiteBackup: vi.fn(),
      checkBackupDirChoice: vi.fn(),
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
const mockCheckOffsite = vi.mocked(commands.checkOffsiteBackup);
const mockGetOffsiteStatus = vi.mocked(commands.getOffsiteBackupStatus);
const mockPrepareOffsite = vi.mocked(commands.prepareOffsiteMedium);
const mockResetOffsite = vi.mocked(commands.resetOffsiteState);
const mockStopOffsite = vi.mocked(commands.stopOffsiteBackup);
const mockCheckBackupDirChoice = vi.mocked(commands.checkBackupDirChoice);
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

function makeOffsiteStatus(overrides: Partial<OffsiteBackupStatus> = {}): OffsiteBackupStatus {
  return {
    prepared: true,
    last_success_at: null,
    last_success_label: null,
    days_since_last_success: null,
    stale: true,
    attached: [],
    last_failure_kind: null,
    local_backup_old: false,
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
  mockCheckOffsite.mockResolvedValue(ok({ kind: "not_prepared" as const }));
  mockGetOffsiteStatus.mockResolvedValue(ok(makeOffsiteStatus({ prepared: false, stale: false })));
  mockPrepareOffsite.mockResolvedValue(
    ok({ label: "控え 2", drive_root: "E:\\", newest_copy: null }),
  );
  mockResetOffsite.mockResolvedValue(ok({ kind: "up_to_date" as const }));
  mockStopOffsite.mockResolvedValue(ok(null));
  mockCheckBackupDirChoice.mockResolvedValue(ok(null));

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

// ---- UI-11b-D14・D15・F3: PC の外の控え（runtime (b)） ----

const OFFSITE_KEY = queryKeys.backupRestore.offsiteStatus();
const UNREADABLE_CAUSE =
  "PC の外の控えの記録を読めません。USB メモリを差して「記録を作り直す」を押すか、使わないなら「PC の外の控えをやめる」を押してください。";
const BEFORE_PREPARE =
  "PC の外の控えは使っていません（任意。使わなくてもアプリは今までどおり動きます）。使うときは USB メモリを差して「この USB メモリを控えの保存先にする」を押してください。";
const NO_MEDIUM =
  "用意した USB メモリが差さっていません。USB メモリを差して「記録を作り直す」を押すか、使わないなら「PC の外の控えをやめる」を押してください";
const RESET_FAILED =
  "PC の外の控えの記録の作り直しを終えられませんでした。「記録を作り直す」がまだ出ていれば、もう一度押してください";
const STOP_FAILED =
  "PC の外の控えをやめる操作を終えられませんでした。「PC の外の控えをやめる」がまだ出ていれば、もう一度押してください";
const SAME_FOLDER =
  "PC の中の保存先が、USB メモリの InventoryBackup フォルダと同じです。「保存先を選ぶ」で PC の中の別のフォルダを選び直してください";
const FILE_PATH = "E:\\InventoryBackup\\inventory_backup_20991231_235959.db";

function keyCount(spy: { mock: { calls: unknown[][] } }, key: readonly string[]): number {
  return spy.mock.calls.filter(
    ([filters]) =>
      JSON.stringify((filters as { queryKey?: unknown } | undefined)?.queryKey) ===
      JSON.stringify(key),
  ).length;
}

function invalidations(spy: { mock: { calls: unknown[][] } }) {
  return {
    offsite: keyCount(spy, OFFSITE_KEY),
    list: keyCount(spy, queryKeys.backupRestore.list()),
  };
}

/** 共通レイアウトの mount の 1 回の確認（offsite）の結果と invalidate を済ませる */
async function settleMountCheck() {
  await waitFor(() => {
    expect(mockCheckOffsite).toHaveBeenCalled();
  });
  await act(async () => {
    await Promise.resolve();
  });
}

const unreadableStatus = () => makeOffsiteStatus({ last_failure_kind: "state_unreadable" });

// describeError の表示（internal は診断ログの案内を続ける、UI-ERR-D1）
const internalShown = (message: string) => `${message}。詳細は診断ログに記録されています。`;

describe("PC の外の控えの card (REQ-901 / UI-11b-D14・§68.4)", () => {
  it("REQ-901 / UI-11b-D14: 「記録を作り直す」で作り直し、status と一覧を取り直し、取り直した status で表示を決める", async () => {
    const user = userEvent.setup();
    mockGetOffsiteStatus.mockResolvedValue(ok(unreadableStatus()));
    const { queryClient } = renderApp("/settings/backup");
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    expect(await screen.findByText(UNREADABLE_CAUSE)).toBeInTheDocument();
    await settleMountCheck();
    const before = invalidations(invalidateSpy);

    // 取り直しの status は読める記録（(1) 以外の原因）
    mockGetOffsiteStatus.mockResolvedValue(ok(makeOffsiteStatus()));
    await user.click(screen.getByRole("button", { name: "記録を作り直す" }));

    expect(await screen.findByText("PC の外の控えの記録を作り直しました")).toBeInTheDocument();
    expect(mockResetOffsite).toHaveBeenCalledTimes(1);
    expect(invalidations(invalidateSpy)).toEqual({
      offsite: before.offsite + 1,
      list: before.list + 1,
    });
    expect(
      await screen.findByText(
        "PC の外の控えがまだ写せていません。USB メモリが差してあるか確かめてください。",
      ),
    ).toBeInTheDocument();
    expect(screen.queryByText(UNREADABLE_CAUSE)).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "記録を作り直す" })).not.toBeInTheDocument();
  });

  it.each([
    ["NoMediumAttached", cmdError(NO_MEDIUM, "validation"), NO_MEDIUM],
    ["StateResetFailed", cmdError(RESET_FAILED, "internal"), internalShown(RESET_FAILED)],
  ] as const)(
    "REQ-901 / UI-11b-D14: 作り直しの失敗（%s）でも status と一覧を取り直し、固定の文を card に出す",
    async (_label, result, message) => {
      const user = userEvent.setup();
      mockGetOffsiteStatus.mockResolvedValue(ok(unreadableStatus()));
      mockResetOffsite.mockResolvedValueOnce(result);
      const { queryClient } = renderApp("/settings/backup");
      const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
      await screen.findByText(UNREADABLE_CAUSE);
      await settleMountCheck();
      const before = invalidations(invalidateSpy);

      await user.click(screen.getByRole("button", { name: "記録を作り直す" }));

      expect(await screen.findByText(message)).toBeInTheDocument();
      expect(invalidations(invalidateSpy)).toEqual({
        offsite: before.offsite + 1,
        list: before.list + 1,
      });
      // 取り直した status がまだ読めない記録なので、原因の文と button が残り押し直せる
      expect(screen.getByText(UNREADABLE_CAUSE)).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "記録を作り直す" })).toBeEnabled();
      expect(screen.getByRole("button", { name: "PC の外の控えをやめる" })).toBeEnabled();
    },
  );

  it("REQ-901 / UI-11b-D14・§68.4 手順 5: やめるは確認をはさみ、キャンセルでは呼ばず、やめると用意の前の文", async () => {
    const user = userEvent.setup();
    mockGetOffsiteStatus.mockResolvedValue(ok(makeOffsiteStatus({ stale: false })));
    const { queryClient } = renderApp("/settings/backup");
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    await screen.findByRole("button", { name: "PC の外の控えをやめる" });
    await settleMountCheck();
    const before = invalidations(invalidateSpy);
    await user.click(screen.getByRole("button", { name: "PC の外の控えをやめる" }));
    const dialog = await screen.findByRole("alertdialog");
    expect(
      within(dialog).getByText(
        "PC の外の控えをやめますか。USB メモリへの写しとホームの知らせが止まります。USB メモリの上の控えと PC の中のバックアップは消えません。",
      ),
    ).toBeInTheDocument();
    await user.click(within(dialog).getByRole("button", { name: "キャンセル" }));
    await waitFor(() => {
      expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    });
    expect(mockStopOffsite).not.toHaveBeenCalled();

    mockGetOffsiteStatus.mockResolvedValue(
      ok(makeOffsiteStatus({ prepared: false, stale: false })),
    );
    await user.click(screen.getByRole("button", { name: "PC の外の控えをやめる" }));
    await user.click(
      within(await screen.findByRole("alertdialog")).getByRole("button", { name: "やめる" }),
    );

    expect(await screen.findByText("PC の外の控えをやめました")).toBeInTheDocument();
    expect(mockStopOffsite).toHaveBeenCalledTimes(1);
    expect(invalidations(invalidateSpy)).toEqual({
      offsite: before.offsite + 1,
      list: before.list,
    });
    expect(await screen.findByText(BEFORE_PREPARE)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "PC の外の控えをやめる" })).not.toBeInTheDocument();
  });

  it.each([
    [true, "移す前の失敗は用意の後のまま button が残る"],
    [false, "移した後の sync の失敗は用意の前の文で button が無い"],
  ] as [boolean, string][])(
    "REQ-901 / UI-11b-D14: StopFailed の後は取り直した status で決める（prepared = %s: %s）",
    async (preparedAfter) => {
      const user = userEvent.setup();
      mockGetOffsiteStatus.mockResolvedValue(ok(makeOffsiteStatus({ stale: false })));
      mockStopOffsite.mockResolvedValueOnce(cmdError(STOP_FAILED, "internal"));
      const { queryClient } = renderApp("/settings/backup");
      const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
      await screen.findByRole("button", { name: "PC の外の控えをやめる" });
      await settleMountCheck();
      const before = invalidations(invalidateSpy);
      await user.click(screen.getByRole("button", { name: "PC の外の控えをやめる" }));
      mockGetOffsiteStatus.mockResolvedValue(
        ok(makeOffsiteStatus({ prepared: preparedAfter, stale: false })),
      );
      await user.click(
        within(await screen.findByRole("alertdialog")).getByRole("button", { name: "やめる" }),
      );

      expect(await screen.findByText(internalShown(STOP_FAILED))).toBeInTheDocument();
      expect(invalidations(invalidateSpy).offsite).toBe(before.offsite + 1);
      if (preparedAfter) {
        expect(screen.getByRole("button", { name: "PC の外の控えをやめる" })).toBeInTheDocument();
      } else {
        expect(await screen.findByText(BEFORE_PREPARE)).toBeInTheDocument();
        expect(
          screen.queryByRole("button", { name: "PC の外の控えをやめる" }),
        ).not.toBeInTheDocument();
      }
    },
  );

  it("REQ-901 / UI-11b-D14・§68.4 手順 2: 用意は成功・失敗とも status を取り直し、picker の cancel では呼ばない", async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp("/settings/backup");
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    const prepare = await screen.findByRole("button", {
      name: "この USB メモリを控えの保存先にする",
    });
    await settleMountCheck();
    const before = invalidations(invalidateSpy).offsite;

    mockOpen.mockResolvedValueOnce(null);
    await user.click(prepare);
    expect(mockPrepareOffsite).not.toHaveBeenCalled();
    expect(keyCount(invalidateSpy, OFFSITE_KEY)).toBe(before);

    mockOpen.mockResolvedValueOnce("E:\\");
    await user.click(prepare);
    expect(
      await screen.findByText("控え 2 を PC の外の控えの保存先にしました"),
    ).toBeInTheDocument();
    expect(mockPrepareOffsite).toHaveBeenCalledWith({ selected_path: "E:\\" });
    expect(keyCount(invalidateSpy, OFFSITE_KEY)).toBe(before + 1);

    const refused = "USB メモリを選んでください（この場所は PC の外の控えに使えません）";
    mockOpen.mockResolvedValueOnce("C:\\");
    mockPrepareOffsite.mockResolvedValueOnce(cmdError(refused, "validation"));
    await user.click(prepare);
    expect(await screen.findByText(refused)).toBeInTheDocument();
    expect(keyCount(invalidateSpy, OFFSITE_KEY)).toBe(before + 2);
  });
});

describe("§68.7 の無効化 (REQ-901 / UI-11b-D14・D15)", () => {
  const OPERATIONS = [
    "この USB メモリを控えの保存先にする",
    "記録を作り直す",
    "PC の外の控えをやめる",
  ];

  function expectOperations(disabled: boolean, options: { hidden?: boolean } = {}) {
    for (const name of OPERATIONS) {
      const button = screen.getByRole("button", { name, ...options });
      if (disabled) expect(button).toBeDisabled();
      else expect(button).toBeEnabled();
    }
  }

  it("REQ-901 / §68.7: 復元の詳細・最終確認・確認から詳細へ戻る間は 3 つの操作を止め、閉じると戻る", async () => {
    const user = userEvent.setup();
    mockGetOffsiteStatus.mockResolvedValue(ok(unreadableStatus()));
    renderApp("/settings/backup");
    await screen.findByText(UNREADABLE_CAUSE);
    expectOperations(false);

    await selectRow(user, BACKUP_DATE_LABEL);
    await screen.findByText("この控えは戻せます");
    expectOperations(true);
    for (const name of OPERATIONS) {
      await user.click(screen.getByRole("button", { name }));
    }
    expect(mockOpen).not.toHaveBeenCalled();
    expect(mockPrepareOffsite).not.toHaveBeenCalled();
    expect(mockResetOffsite).not.toHaveBeenCalled();
    expect(mockStopOffsite).not.toHaveBeenCalled();

    await user.click(screen.getByRole("button", { name: "復元の確認へ進む" }));
    await screen.findByRole("alertdialog", { name: "元に戻せません" });
    expectOperations(true, { hidden: true });
    await user.click(screen.getByRole("button", { name: "キャンセル" }));
    await waitFor(() => {
      expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    });
    expectOperations(true);

    await user.click(screen.getByRole("button", { name: "閉じる" }));
    expectOperations(false);
  });

  it("REQ-901 / §68.7: 未選択でも busy の間（手動バックアップの実行中）は 3 つの操作と file の入口を止める", async () => {
    const user = userEvent.setup();
    mockGetOffsiteStatus.mockResolvedValue(ok(unreadableStatus()));
    const pending = deferred<Awaited<ReturnType<typeof commands.createBackup>>>();
    mockCreateBackup.mockReturnValueOnce(pending.promise);
    renderApp("/settings/backup");
    await screen.findByText(UNREADABLE_CAUSE);

    await user.click(screen.getByRole("button", { name: "今すぐバックアップを作成" }));
    expectOperations(true);
    expect(screen.getByRole("button", { name: "控えを選んで確かめる" })).toBeDisabled();

    pending.resolve(
      ok({
        file_name: "inventory_backup_20260706_100000.db",
        file_path: "/tmp/backups/inventory_backup_20260706_100000.db",
        size_bytes: 10_000_000,
      }),
    );
    await waitFor(() => {
      expectOperations(false);
    });
    expect(screen.getByRole("button", { name: "控えを選んで確かめる" })).toBeEnabled();
  });

  it("REQ-901 / §68.7（P3-a）: 用意の実行中は一覧の行と file の入口が disabled で inspectBackup を呼ばない", async () => {
    const user = userEvent.setup();
    const pending = deferred<Awaited<ReturnType<typeof commands.prepareOffsiteMedium>>>();
    mockPrepareOffsite.mockReturnValueOnce(pending.promise);
    renderApp("/settings/backup");
    const backupDate = await screen.findByText(BACKUP_DATE_LABEL);
    const row = backupDate.closest("tr");
    if (!row) throw new Error("backup row not found");

    mockOpen.mockResolvedValueOnce("E:\\");
    await user.click(screen.getByRole("button", { name: "この USB メモリを控えの保存先にする" }));
    await waitFor(() => {
      expect(mockPrepareOffsite).toHaveBeenCalledTimes(1);
    });
    const rowButton = within(row).getByRole("button", { name: "この控えに戻す" });
    const fileEntry = screen.getByRole("button", { name: "控えを選んで確かめる" });
    expect(rowButton).toBeDisabled();
    expect(fileEntry).toBeDisabled();
    await user.click(rowButton);
    await user.click(fileEntry);
    expect(mockInspectBackup).not.toHaveBeenCalled();
    expect(mockOpen).toHaveBeenCalledTimes(1);

    pending.resolve(ok({ label: "控え 2", drive_root: "E:\\", newest_copy: null }));
    await screen.findByText("控え 2 を PC の外の控えの保存先にしました");
    expect(within(row).getByRole("button", { name: "この控えに戻す" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "控えを選んで確かめる" })).toBeEnabled();
  });

  it.each([
    { operation: "用意", entry: "一覧の行" },
    { operation: "作り直し", entry: "file の入口" },
  ] as const)(
    "REQ-901 / §68.7: $operation の実行中に画面を離れて戻り、$entry で控えを選んでも、完了まで inspectBackup を呼ばない",
    async ({ operation, entry }) => {
      const user = userEvent.setup();
      const pendingPrepare = deferred<Awaited<ReturnType<typeof commands.prepareOffsiteMedium>>>();
      const pendingReset = deferred<Awaited<ReturnType<typeof commands.resetOffsiteState>>>();
      if (operation === "用意") {
        mockPrepareOffsite.mockReturnValueOnce(pendingPrepare.promise);
        mockOpen.mockResolvedValueOnce("E:\\");
      } else {
        mockGetOffsiteStatus.mockResolvedValue(ok(unreadableStatus()));
        mockResetOffsite.mockReturnValueOnce(pendingReset.promise);
      }
      const { router } = renderApp("/settings/backup");
      await screen.findByText(BACKUP_DATE_LABEL);
      await user.click(
        await screen.findByRole("button", {
          name: operation === "用意" ? "この USB メモリを控えの保存先にする" : "記録を作り直す",
        }),
      );
      await waitFor(() => {
        expect(operation === "用意" ? mockPrepareOffsite : mockResetOffsite).toHaveBeenCalledTimes(
          1,
        );
      });

      // 画面の state（offsite_preparing / offsite_resetting）は離脱で消え、再訪した画面は前の実行中を知らない
      await leavePage(router);
      await revisitPage(router);
      if (entry === "一覧の行") {
        await selectRow(user, BACKUP_DATE_LABEL);
      } else {
        mockOpen.mockResolvedValueOnce(FILE_PATH);
        await user.click(screen.getByRole("button", { name: "控えを選んで確かめる" }));
      }
      await act(async () => {
        await new Promise((resolve) => setTimeout(resolve, 0));
      });
      // 実行中の操作（媒体の掃除を含む）が終わるまで、確かめの表示のまま待つ
      expect(mockInspectBackup).not.toHaveBeenCalled();
      expect(screen.getByText("控えを確かめています")).toBeInTheDocument();

      if (operation === "用意") {
        pendingPrepare.resolve(ok({ label: "控え 2", drive_root: "E:\\", newest_copy: null }));
      } else {
        pendingReset.resolve(ok({ kind: "up_to_date" as const }));
      }
      expect(await screen.findByText("この控えは戻せます")).toBeInTheDocument();
      expect(mockInspectBackup).toHaveBeenCalledTimes(1);
    },
  );
});

describe("保存先を選ぶ (REQ-901 / UI-11b-F3・§68.4 の backup_path 変更)", () => {
  async function choosePath(user: ReturnType<typeof userEvent.setup>) {
    await user.click(await screen.findByRole("button", { name: "保存先を選ぶ" }));
  }

  it("REQ-901 / UI-11b-F3: 確かめが BackupDirIsMediumFolder なら保存せず、設定の card に固定の文を出す", async () => {
    const user = userEvent.setup();
    mockOpen.mockResolvedValueOnce("E:\\InventoryBackup");
    mockCheckBackupDirChoice.mockResolvedValueOnce(cmdError(SAME_FOLDER, "validation"));
    const { queryClient } = renderApp("/settings/backup");
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    await screen.findByText("/tmp/backups");

    await choosePath(user);

    const message = await screen.findByText(SAME_FOLDER);
    expect(message.closest('[data-slot="card"]')).toHaveTextContent("バックアップ設定");
    expect(mockCheckBackupDirChoice).toHaveBeenCalledWith({ selected_path: "E:\\InventoryBackup" });
    expect(commands.updateSetting).not.toHaveBeenCalled();
    expect(screen.getByText("/tmp/backups")).toBeInTheDocument();
    expect(keyCount(invalidateSpy, queryKeys.backupRestore.list())).toBe(0);
    expect(keyCount(invalidateSpy, queryKeys.backupRestore.effectiveDir())).toBe(0);
  });

  it("REQ-901 / UI-11b-F3: IPC の例外でも保存しない", async () => {
    const user = userEvent.setup();
    mockOpen.mockResolvedValueOnce("E:\\InventoryBackup");
    mockCheckBackupDirChoice.mockRejectedValueOnce(new Error("synthetic ipc failure"));
    renderApp("/settings/backup");
    await choosePath(user);
    await waitFor(() => {
      expect(mockCheckBackupDirChoice).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "保存先を選ぶ" })).toBeEnabled();
    });
    expect(commands.updateSetting).not.toHaveBeenCalled();
  });

  it("REQ-901 / UI-11b-F3: Ok のときだけ確かめた値と同じ値で 1 回保存し、picker の cancel ではどちらも呼ばない", async () => {
    const user = userEvent.setup();
    vi.mocked(commands.updateSetting).mockResolvedValue(ok(null));
    renderApp("/settings/backup");

    mockOpen.mockResolvedValueOnce(null);
    await choosePath(user);
    expect(mockCheckBackupDirChoice).not.toHaveBeenCalled();
    expect(commands.updateSetting).not.toHaveBeenCalled();

    mockOpen.mockResolvedValueOnce("D:\\store-backups");
    await choosePath(user);
    await waitFor(() => {
      expect(commands.updateSetting).toHaveBeenCalledTimes(1);
    });
    expect(mockCheckBackupDirChoice).toHaveBeenCalledWith({ selected_path: "D:\\store-backups" });
    expect(commands.updateSetting).toHaveBeenCalledWith({
      key: "backup_path",
      value: "D:\\store-backups",
    });
    expect(mockCheckBackupDirChoice.mock.invocationCallOrder[0]).toBeLessThan(
      vi.mocked(commands.updateSetting).mock.invocationCallOrder[0],
    );
  });
});

describe("控えを選んで確かめる (REQ-901 / UI-11b-D15 の file の入口)", () => {
  const FILE_LABEL = "12月31日 23:59";

  it("REQ-901 / UI-11b-D15: 選んだ file が同じ path で確かめ・事前バックアップ・復元へ渡る", async () => {
    const user = userEvent.setup();
    mockInspectBackup.mockResolvedValue(
      ok(
        makeInspection({
          file_name: "inventory_backup_20991231_235959.db",
          created_at: "2099-12-31 23:59:59",
        }),
      ),
    );
    mockRestoreBackup.mockResolvedValueOnce(ok(null));
    renderApp("/settings/backup");

    mockOpen.mockResolvedValueOnce(FILE_PATH);
    await user.click(await screen.findByRole("button", { name: "控えを選んで確かめる" }));
    expect(await screen.findByText("この控えは戻せます")).toBeInTheDocument();
    expect(mockOpen).toHaveBeenCalledWith(
      expect.objectContaining({
        directory: false,
        multiple: false,
        filters: [{ name: "バックアップ", extensions: ["db"] }],
      }),
    );
    expect(mockInspectBackup).toHaveBeenCalledWith({ backup_path: FILE_PATH });

    await startRestore(user, FILE_LABEL);
    await waitFor(() => {
      expect(mockRestoreBackup).toHaveBeenCalledWith({ backup_path: FILE_PATH });
    });
    expect(mockCreateBackup).toHaveBeenCalledTimes(1);
  });

  it("REQ-901 / UI-11b-D15: picker の cancel では確かめも停止もしない", async () => {
    const user = setupFakeTimers();
    renderApp("/settings/backup");
    mockOpen.mockResolvedValueOnce(null);
    await user.click(await screen.findByRole("button", { name: "控えを選んで確かめる" }));
    expect(mockInspectBackup).not.toHaveBeenCalled();
    await expectCheckResumes();
  });

  it("REQ-901 / UI-11b-D15・D16: mount の 1 回の offsite の完了を待ってから確かめ、止めている間は呼ばず、キャンセルでは再開せず、閉じると次の tick で写す", async () => {
    const user = setupFakeTimers();
    const pendingOffsite = deferred<Awaited<ReturnType<typeof commands.checkOffsiteBackup>>>();
    mockCheckOffsite.mockReturnValueOnce(pendingOffsite.promise);
    mockInspectBackup.mockResolvedValue(
      ok(
        makeInspection({
          file_name: "inventory_backup_20991231_235959.db",
          created_at: "2099-12-31 23:59:59",
        }),
      ),
    );
    renderApp("/settings/backup");
    await screen.findByRole("heading", { name: "バックアップ・復元" });
    expect(mockCheckOffsite).toHaveBeenCalledTimes(1);

    mockOpen.mockResolvedValueOnce(FILE_PATH);
    await user.click(screen.getByRole("button", { name: "控えを選んで確かめる" }));
    await vi.advanceTimersByTimeAsync(0);
    expect(mockInspectBackup).not.toHaveBeenCalled();

    pendingOffsite.resolve(ok({ kind: "up_to_date" }));
    expect(await screen.findByText("この控えは戻せます")).toBeInTheDocument();
    expect(mockInspectBackup).toHaveBeenCalledTimes(1);
    await expectNoCheckWhile(120_000);
    expect(mockCheckOffsite).toHaveBeenCalledTimes(1);

    await user.click(screen.getByRole("button", { name: "復元の確認へ進む" }));
    await screen.findByRole("alertdialog", { name: "元に戻せません" });
    await user.click(screen.getByRole("button", { name: "キャンセル" }));
    await waitFor(() => {
      expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    });
    await expectNoCheckWhile(60_000);
    expect(mockCheckOffsite).toHaveBeenCalledTimes(1);

    await user.click(screen.getByRole("button", { name: "閉じる" }));
    await expectCheckResumes();
    await waitFor(() => {
      expect(mockCheckOffsite).toHaveBeenCalledTimes(2);
    });
  });
});
