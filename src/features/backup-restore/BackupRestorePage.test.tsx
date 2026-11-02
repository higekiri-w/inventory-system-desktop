import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { toast } from "sonner";
import { commands } from "@/lib/bindings";
import type { BackupInspection, CmdErrorKind } from "@/lib/bindings";
import { queryKeys } from "@/lib/query-keys";
import { open } from "@tauri-apps/plugin-dialog";
import {
  clearRestoreSuccessPending,
  consumeRestoreSuccessPending,
} from "@/lib/restore-success-notification";
import { BackupRestorePage } from "./BackupRestorePage";
import {
  holdAutoBackupCheckForRestore,
  resetAutoBackupCheckForTests,
  useAutoBackupCheck,
} from "./useAutoBackupCheck";

const mockNavigate = vi.fn();

vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => mockNavigate,
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

vi.mock("@/lib/bindings", () => ({
  commands: {
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
  },
}));

const mockGetSettings = vi.mocked(commands.getSettings);
const mockUpdateSetting = vi.mocked(commands.updateSetting);
const mockCreateBackup = vi.mocked(commands.createBackup);
const mockCheckAutoBackup = vi.mocked(commands.checkAutoBackup);
const mockListBackups = vi.mocked(commands.listBackups);
const mockGetEffectiveBackupDir = vi.mocked(commands.getEffectiveBackupDir);
const mockRestoreBackup = vi.mocked(commands.restoreBackup);
const mockInspectBackup = vi.mocked(commands.inspectBackup);
const mockCheckOffsite = vi.mocked(commands.checkOffsiteBackup);
const mockGetOffsiteStatus = vi.mocked(commands.getOffsiteBackupStatus);
const mockCheckBackupDirChoice = vi.mocked(commands.checkBackupDirChoice);
const mockOpen = vi.mocked(open);
const mockToastSuccess = vi.mocked(toast.success);

function ok<T>(data: T) {
  return { status: "ok" as const, data };
}

function cmdError(message: string, kind: CmdErrorKind = "internal", errorId: string | null = null) {
  return {
    status: "error" as const,
    error: { kind, message, field: null, error_id: errorId, stocktake_recovery: null },
  };
}

function renderWithClient(ui: ReactNode) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  const clearSpy = vi.spyOn(queryClient, "clear");
  return {
    queryClient,
    clearSpy,
    ...render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>),
  };
}

// UI-11b-D13: 確認の timer は共通レイアウトの hook が持つ。page 単体では timer が無く
// 停止の assertion が自明に通るため、hook を mount する wrapper と page を一緒に描く。
function PageWithAutoBackupCheck() {
  useAutoBackupCheck();
  return <BackupRestorePage />;
}

type CheckResult = Awaited<ReturnType<typeof commands.checkAutoBackup>>;

function deferred<T>() {
  let resolve: (value: T) => void = () => undefined;
  const promise = new Promise<T>((settle) => {
    resolve = settle;
  });
  return { promise, resolve };
}

function listInvalidateCount(spy: { mock: { calls: unknown[][] } }): number {
  return spy.mock.calls.filter(
    ([filters]) =>
      JSON.stringify((filters as { queryKey?: unknown } | undefined)?.queryKey) ===
      JSON.stringify(queryKeys.backupRestore.list()),
  ).length;
}

// 設計の例（§68.9）と無関係の値にし、表示が mock から来たことを確かめる
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

function mockDefaultCommands() {
  mockGetSettings.mockResolvedValue(
    ok([
      { key: "backup_enabled", value: "1", updated_at: "2026-07-06T00:00:00" },
      { key: "backup_time", value: "23:00", updated_at: "2026-07-06T00:00:00" },
      { key: "backup_path", value: "/tmp/backups", updated_at: "2026-07-06T00:00:00" },
      { key: "backup_retention_days", value: "3", updated_at: "2026-07-06T00:00:00" },
    ]),
  );
  mockListBackups.mockResolvedValue(
    ok([
      {
        file_name: "inventory_backup_20260703_210000.db",
        file_path: "/tmp/backups/inventory_backup_20260703_210000.db",
        size_bytes: 12_400_000,
        created_at: "2026-07-03 21:00:00",
      },
    ]),
  );
  mockCreateBackup.mockResolvedValue(
    ok({
      file_name: "inventory_backup_20260706_100000.db",
      file_path: "/tmp/backups/inventory_backup_20260706_100000.db",
      size_bytes: 10_000_000,
    }),
  );
  mockUpdateSetting.mockResolvedValue(ok(null));
  mockCheckAutoBackup.mockResolvedValue(ok(false));
  mockGetEffectiveBackupDir.mockResolvedValue(ok("/tmp/backups"));
  mockRestoreBackup.mockResolvedValue(ok(null));
  mockInspectBackup.mockResolvedValue(ok(makeInspection()));
  mockOpen.mockResolvedValue(null);
  mockCheckOffsite.mockResolvedValue(ok({ kind: "not_prepared" as const }));
  mockGetOffsiteStatus.mockResolvedValue(
    ok({
      prepared: false,
      last_success_at: null,
      last_success_label: null,
      days_since_last_success: null,
      stale: false,
      attached: [],
      last_failure_kind: null,
      local_backup_old: false,
    }),
  );
  mockCheckBackupDirChoice.mockResolvedValue(ok(null));
}

async function openRestoreDetail(user: ReturnType<typeof userEvent.setup>) {
  await screen.findByRole("heading", { name: "バックアップ・復元" });
  const backupDate = await screen.findByText("7月3日 21:00");
  const row = backupDate.closest("tr");
  expect(row).not.toBeNull();
  if (!row) throw new Error("backup row not found");
  await user.click(within(row).getByRole("button", { name: "この控えに戻す" }));
  // UI-11b-D15: 詳細は控えの確かめの解決の後に出る
  // SC20 / DSR-08: 既存文言・roleを保ってwarning variantへ移行する。
  expect(
    (await screen.findByText("復元すると今の記録は戻せません")).closest('[data-slot="alert"]'),
  ).toHaveAttribute("data-variant", "warning");
  expect(
    screen.getByText("復元すると今の記録は戻せません").closest('[data-slot="alert"]'),
  ).toHaveAttribute("role", "alert");
  expect(
    screen.getByText("この時点の状態に戻ります。この控えより後に記録した内容は消えます"),
  ).toBeInTheDocument();
}

async function startRestoreConfirmation(user: ReturnType<typeof userEvent.setup>) {
  await openRestoreDetail(user);
  await user.click(screen.getByRole("button", { name: "復元の確認へ進む" }));
}

beforeEach(() => {
  vi.useRealTimers();
  mockNavigate.mockReset();
  mockGetSettings.mockReset();
  mockUpdateSetting.mockReset();
  mockCreateBackup.mockReset();
  mockCheckAutoBackup.mockReset();
  mockListBackups.mockReset();
  mockGetEffectiveBackupDir.mockReset();
  mockRestoreBackup.mockReset();
  mockOpen.mockReset();
  mockToastSuccess.mockReset();
  mockInspectBackup.mockReset();
  mockDefaultCommands();
  clearRestoreSuccessPending();
  resetAutoBackupCheckForTests();
});

afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
});

describe("DSR-20 D-D", () => {
  it("T9: 復元確認の中止 button はキャンセル文言を使う", async () => {
    const user = userEvent.setup();
    renderWithClient(<BackupRestorePage />);

    await startRestoreConfirmation(user);
    const dialog = screen.getByRole("alertdialog");
    expect(within(dialog).getByRole("button", { name: "キャンセル" })).toBeInTheDocument();
    expect(within(dialog).queryByRole("button", { name: "やめる" })).not.toBeInTheDocument();
  });
});

describe("BackupRestorePage (UI-11b / QR-05 / REQ-901)", () => {
  it("QR-05 REQ-901 shows break-glass only when pre-restore backup fails", async () => {
    const user = userEvent.setup();
    mockCreateBackup.mockResolvedValueOnce(cmdError("保存先にバックアップを作成できませんでした"));

    renderWithClient(<BackupRestorePage />);
    await startRestoreConfirmation(user);

    expect(mockRestoreBackup).not.toHaveBeenCalled();
    expect(screen.getByText("復元前のバックアップを作成できませんでした")).toBeInTheDocument();

    const proceed = screen.getByRole("button", { name: "最終確認へ進む" });
    expect(proceed).toBeDisabled();

    await user.click(
      screen.getByRole("checkbox", { name: "今の状態は保存できませんが、復元を続けます" }),
    );
    expect(proceed).toBeEnabled();

    await user.click(proceed);
    expect(screen.getByRole("alertdialog", { name: "元に戻せません" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "7月3日 21:00 の控えに戻す" })).toBeEnabled();
  });

  it("QR-05 REQ-901 treats recovered restore failure as retryable", async () => {
    const user = userEvent.setup();
    mockRestoreBackup.mockResolvedValueOnce(cmdError("復元対象のファイルを読み込めませんでした"));

    renderWithClient(<BackupRestorePage />);
    await startRestoreConfirmation(user);
    await user.click(screen.getByRole("button", { name: "7月3日 21:00 の控えに戻す" }));

    expect(
      await screen.findByText(
        "バックアップの復元に失敗しました。現在のデータには戻しています。もう一度お試しください。",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "今すぐバックアップを作成" })).toBeEnabled();
    expect(screen.queryByText("アプリを閉じて、もう一度開いてください")).not.toBeInTheDocument();
  });

  it("QR-05 REQ-901 branches restore failures by CmdError kind", async () => {
    const user = userEvent.setup();
    mockRestoreBackup.mockResolvedValueOnce({
      status: "error",
      error: {
        kind: "restore_failed_unrecoverable",
        message: "message text is intentionally unrelated",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });

    renderWithClient(<BackupRestorePage />);
    await startRestoreConfirmation(user);
    await user.click(screen.getByRole("button", { name: "7月3日 21:00 の控えに戻す" }));

    expect(await screen.findByText("再起動が必要です")).toBeInTheDocument();
  });

  it("QR-05 REQ-901 shows non-assertive guidance for durability unknown", async () => {
    const user = userEvent.setup();
    mockRestoreBackup.mockResolvedValueOnce({
      status: "error",
      error: {
        kind: "restore_durability_unknown",
        message: "durability state unknown",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });

    renderWithClient(<BackupRestorePage />);
    await startRestoreConfirmation(user);
    await user.click(screen.getByRole("button", { name: "7月3日 21:00 の控えに戻す" }));

    expect(await screen.findByText("復元結果を確認できませんでした")).toBeInTheDocument();
    expect(screen.getByText("復元が完了したか確定できませんでした。")).toBeInTheDocument();
    expect(screen.queryByText(/現在のデータには戻しています/)).not.toBeInTheDocument();
  });

  it.each([
    [
      "restore_failed_recovered",
      "バックアップの復元に失敗しました。現在のデータには戻しています。もう一度お試しください。",
    ],
    ["restore_failed_unrecoverable", "再起動が必要です"],
    ["restore_durability_unknown", "復元結果を確認できませんでした"],
  ] as const)(
    "REQ-700 68 §68.7 shows error_id for %s as a separate element",
    async (kind, text) => {
      const user = userEvent.setup();
      mockRestoreBackup.mockResolvedValueOnce(
        cmdError("synthetic restore detail", kind, "E-20260726-153021-a1b2"),
      );

      renderWithClient(<BackupRestorePage />);
      await startRestoreConfirmation(user);
      await user.click(screen.getByRole("button", { name: "7月3日 21:00 の控えに戻す" }));

      expect(await screen.findByText(text)).toBeInTheDocument();
      expect(screen.getByText("（エラーID: E-20260726-153021-a1b2）")).toBeInTheDocument();
      if (kind === "restore_failed_recovered") {
        // §68.7: restore_failed_recovered の後は復元の card が未選択（再試行は選び直して確かめから）
        expect(
          screen.getByText("一覧から戻したい控えを選ぶと、内容と確認手順を表示します。"),
        ).toBeInTheDocument();
        expect(screen.queryByRole("button", { name: "復元の確認へ進む" })).not.toBeInTheDocument();
      }
    },
  );

  it("QR-05 REQ-901 shows restart guidance and disables operations on double failure", async () => {
    const user = userEvent.setup();
    mockRestoreBackup.mockResolvedValueOnce(
      cmdError(
        "バックアップの復元に失敗し、DB接続の復旧もできませんでした。アプリを再起動してください",
        "restore_failed_unrecoverable",
      ),
    );

    renderWithClient(<BackupRestorePage />);
    await startRestoreConfirmation(user);
    await user.click(screen.getByRole("button", { name: "7月3日 21:00 の控えに戻す" }));

    expect(await screen.findByText("再起動が必要です")).toBeInTheDocument();
    expect(screen.getByText("アプリを閉じて、もう一度開いてください")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "今すぐバックアップを作成" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "保存先を選ぶ" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "この控えに戻す" })).toBeDisabled();
  });

  // SPEC-COLOR-EMPHASIS-RT-1 / D-CE2（main wiring）: 常に危険・失敗のまま残る Alert に、
  // 部品が描く三角 icon が 1 つだけ出る（site の明示の icon と二重にならない）。
  it("D-CE2: the fatal restore alert is the soft destructive alert with exactly one icon", async () => {
    const user = userEvent.setup();
    mockRestoreBackup.mockResolvedValueOnce(
      cmdError("synthetic fatal restore failure", "restore_failed_unrecoverable"),
    );

    renderWithClient(<BackupRestorePage />);
    await startRestoreConfirmation(user);
    await user.click(screen.getByRole("button", { name: "7月3日 21:00 の控えに戻す" }));

    const alert = (await screen.findByText("再起動が必要です")).closest('[data-slot="alert"]');
    expect(alert).toHaveAttribute("data-variant", "destructive");
    expect(alert).toHaveClass("bg-destructive-soft", "border-destructive");
    expect(alert?.querySelectorAll("svg")).toHaveLength(1);
  });

  it.each([
    ["restore_failed_unrecoverable", "再起動が必要です"],
    ["restore_durability_unknown", "復元結果を確認できませんでした"],
  ] as const)(
    "QR-05 REQ-901 stops auto backup checks after double failure (%s)",
    async (kind, title) => {
      vi.useFakeTimers({ shouldAdvanceTime: true });
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      mockCheckAutoBackup.mockResolvedValue(ok(false));
      mockRestoreBackup.mockResolvedValueOnce(cmdError("synthetic fatal restore failure", kind));

      renderWithClient(<PageWithAutoBackupCheck />);
      await startRestoreConfirmation(user);
      await user.click(screen.getByRole("button", { name: "7月3日 21:00 の控えに戻す" }));

      expect(await screen.findByText(title)).toBeInTheDocument();
      const callsAtFatal = mockCheckAutoBackup.mock.calls.length;

      await vi.advanceTimersByTimeAsync(60_000);

      expect(mockCheckAutoBackup).toHaveBeenCalledTimes(callsAtFatal);
    },
  );

  it.each([
    ["成功", () => mockRestoreBackup.mockResolvedValueOnce(ok(null))],
    [
      "restore_failed_recovered",
      () =>
        mockRestoreBackup.mockResolvedValueOnce(
          cmdError("synthetic recovered failure", "restore_failed_recovered"),
        ),
    ],
    ["IPC の例外", () => mockRestoreBackup.mockRejectedValueOnce(new Error("synthetic ipc"))],
  ] as const)(
    "REQ-901 / UI-11b-D13: fatal でない結果の後は再開する（%s）",
    async (_label, arrangeRestore) => {
      vi.useFakeTimers({ shouldAdvanceTime: true });
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      arrangeRestore();

      renderWithClient(<PageWithAutoBackupCheck />);
      await startRestoreConfirmation(user);
      await user.click(screen.getByRole("button", { name: "7月3日 21:00 の控えに戻す" }));
      await waitFor(() => {
        expect(mockRestoreBackup).toHaveBeenCalledTimes(1);
      });
      await waitFor(() => {
        expect(screen.getByRole("button", { name: "この控えに戻す" })).toBeEnabled();
      });
      const callsAfterRestore = mockCheckAutoBackup.mock.calls.length;

      await vi.advanceTimersByTimeAsync(60_000);

      expect(mockCheckAutoBackup).toHaveBeenCalledTimes(callsAfterRestore + 1);
    },
  );

  it("QR-05 REQ-901 clears query cache and navigates home after restore success", async () => {
    const user = userEvent.setup();
    const { clearSpy } = renderWithClient(<BackupRestorePage />);

    await startRestoreConfirmation(user);
    await user.click(screen.getByRole("button", { name: "7月3日 21:00 の控えに戻す" }));

    await waitFor(() => {
      expect(clearSpy).toHaveBeenCalled();
    });
    expect(mockNavigate).toHaveBeenCalledWith({ to: "/" });
    // primary path (navigate 成功) は home Alert に一本化し、toast との二重通知にしない。
    expect(mockToastSuccess).not.toHaveBeenCalled();
  });

  it("test_ui11b_d11_sets_restore_success_pending_flag_before_navigate", async () => {
    const user = userEvent.setup();
    renderWithClient(<BackupRestorePage />);

    await startRestoreConfirmation(user);
    await user.click(screen.getByRole("button", { name: "7月3日 21:00 の控えに戻す" }));

    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith({ to: "/" });
    });
    // producer が flag を set 済みであることを consumer 相当の consume で確認する
    // (このテストの consume 自体が flag を消費するため、本 assertion が最終確認)。
    expect(consumeRestoreSuccessPending()).toBe(true);
  });

  it("test_ui11b_d11_clears_restore_success_pending_flag_when_navigate_rejects", async () => {
    const user = userEvent.setup();
    mockNavigate.mockRejectedValueOnce(new Error("synthetic navigate rejection"));
    renderWithClient(<BackupRestorePage />);

    await startRestoreConfirmation(user);
    await user.click(screen.getByRole("button", { name: "7月3日 21:00 の控えに戻す" }));

    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith({ to: "/" });
    });
    // navigate reject 後は次回到達での誤表示を防ぐため flag が消去されている。
    expect(consumeRestoreSuccessPending()).toBe(false);
    // home Alert を出せない reject fallback として、成功 feedback の toast が発火する
    // （無通知への劣化を防ぐ。coordinator 裁定 2026-08-03、要裁定 #2 是正）。
    expect(mockToastSuccess).toHaveBeenCalledWith("バックアップから復元しました");
  });

  it("QR-05 REQ-901 shows created backup file path after manual backup", async () => {
    const user = userEvent.setup();

    renderWithClient(<BackupRestorePage />);
    await screen.findByRole("heading", { name: "バックアップ・復元" });

    await user.click(screen.getByRole("button", { name: "今すぐバックアップを作成" }));

    expect(await screen.findByText("バックアップを作成しました")).toBeInTheDocument();
    expect(
      screen.getByText("保存先: /tmp/backups/inventory_backup_20260706_100000.db"),
    ).toBeInTheDocument();
  });

  it("QR-05 REQ-901 updates backup_path only from directory picker", async () => {
    const user = userEvent.setup();
    mockOpen.mockResolvedValueOnce(null).mockResolvedValueOnce("/tmp/new-backups");

    renderWithClient(<BackupRestorePage />);
    await screen.findByRole("heading", { name: "バックアップ・復元" });

    expect(screen.queryByRole("textbox", { name: "現在の保存先" })).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "保存先を選ぶ" }));
    expect(mockUpdateSetting).not.toHaveBeenCalled();

    await user.click(screen.getByRole("button", { name: "保存先を選ぶ" }));
    await waitFor(() => {
      expect(mockUpdateSetting).toHaveBeenCalledWith({
        key: "backup_path",
        value: "/tmp/new-backups",
      });
    });
    expect(mockOpen).toHaveBeenCalledWith({ directory: true, multiple: false });
  });

  it("QR-05 REQ-901 shows effective backup dir returned by backend", async () => {
    renderWithClient(<BackupRestorePage />);
    await screen.findByRole("heading", { name: "バックアップ・復元" });

    expect(await screen.findByText("/tmp/backups")).toBeInTheDocument();
  });

  it("QR-05 REQ-901 shows fallback note when backup_path is unset", async () => {
    mockGetSettings.mockResolvedValue(
      ok([
        { key: "backup_enabled", value: "1", updated_at: "2026-07-06T00:00:00" },
        { key: "backup_time", value: "23:00", updated_at: "2026-07-06T00:00:00" },
        { key: "backup_path", value: "", updated_at: "2026-07-06T00:00:00" },
        { key: "backup_retention_days", value: "3", updated_at: "2026-07-06T00:00:00" },
      ]),
    );
    mockGetEffectiveBackupDir.mockResolvedValue(ok("/home/user/.local/share/app/backups"));

    renderWithClient(<BackupRestorePage />);
    await screen.findByRole("heading", { name: "バックアップ・復元" });

    expect(await screen.findByText("/home/user/.local/share/app/backups")).toBeInTheDocument();
    expect(
      screen.getByText("保存先が未設定のためアプリ既定フォルダに保存されます"),
    ).toBeInTheDocument();
  });

  it("QR-05 REQ-901 hides effective backup dir row when the fetch fails", async () => {
    mockGetEffectiveBackupDir.mockResolvedValue(cmdError("実効保存先を取得できませんでした"));

    renderWithClient(<BackupRestorePage />);
    await screen.findByRole("heading", { name: "バックアップ・復元" });
    await screen.findByRole("button", { name: "保存先を選ぶ" });

    expect(screen.queryByText("現在の保存先")).not.toBeInTheDocument();
    expect(
      screen.queryByText("保存先が未設定のためアプリ既定フォルダに保存されます"),
    ).not.toBeInTheDocument();
    // 他機能（手動バックアップ）は取得失敗の影響を受けない
    expect(screen.getByRole("button", { name: "今すぐバックアップを作成" })).toBeEnabled();
  });
});

describe("復元の前の控えの確かめ (REQ-901 / UI-11b-D15)", () => {
  it("REQ-901 / UI-11b-D15: 戻せる控えの詳細に確かめの結果を出し、同じ path を確かめと復元に渡す", async () => {
    const user = userEvent.setup();
    renderWithClient(<BackupRestorePage />);

    await startRestoreConfirmation(user);
    expect(screen.getByText("この控えは戻せます")).toBeInTheDocument();
    expect(screen.getByText("商品の数: 37 件")).toBeInTheDocument();
    expect(screen.getByText("最後の記録: 7月3日 20:47")).toBeInTheDocument();
    expect(mockInspectBackup).toHaveBeenCalledWith({
      backup_path: "/tmp/backups/inventory_backup_20260703_210000.db",
    });

    await user.click(screen.getByRole("button", { name: "7月3日 21:00 の控えに戻す" }));
    await waitFor(() => {
      expect(mockRestoreBackup).toHaveBeenCalledWith({
        backup_path: "/tmp/backups/inventory_backup_20260703_210000.db",
      });
    });
  });

  it("REQ-901 / UI-11b-D15: 商品の数 0 は「0 件」と出し、取れない値の行は出さない", async () => {
    const user = userEvent.setup();
    mockInspectBackup.mockResolvedValue(
      ok(makeInspection({ product_count: 0, last_operation_at: null })),
    );
    renderWithClient(<BackupRestorePage />);

    await openRestoreDetail(user);
    expect(screen.getByText("この控えは戻せます")).toBeInTheDocument();
    expect(screen.getByText("商品の数: 0 件")).toBeInTheDocument();
    expect(screen.queryByText(/^最後の記録:/)).not.toBeInTheDocument();
  });

  it("REQ-901 / UI-11b-D15: 商品の数が取れない控えは商品の数の行を出さない", async () => {
    const user = userEvent.setup();
    mockInspectBackup.mockResolvedValue(ok(makeInspection({ product_count: null })));
    renderWithClient(<BackupRestorePage />);

    await openRestoreDetail(user);
    expect(screen.queryByText(/^商品の数:/)).not.toBeInTheDocument();
    expect(screen.getByText("最後の記録: 7月3日 20:47")).toBeInTheDocument();
  });

  it("REQ-901 / UI-11b-D13 / §68.7: 成功の後の処理が throw しても release は 1 回（別の復元の停止を外さない）で、失敗と表示しない", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
    const { clearSpy } = renderWithClient(<PageWithAutoBackupCheck />);
    clearSpy.mockImplementation(() => {
      throw new Error("synthetic clear failure");
    });

    await startRestoreConfirmation(user);
    // 前の画面で始めた別の復元がまだ実行中（復元の理由を 1 つ持つ）
    holdAutoBackupCheckForRestore();
    await user.click(screen.getByRole("button", { name: "7月3日 21:00 の控えに戻す" }));
    await waitFor(() => {
      expect(clearSpy).toHaveBeenCalled();
    });
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "この控えに戻す" })).toBeEnabled();
    });
    const callsAfterRestore = mockCheckAutoBackup.mock.calls.length;
    // §68.7: CMD の Ok の後の失敗は復元の失敗・元に戻した、と表示しない。成功の feedback を出す
    expect(screen.queryByText(/バックアップの復元に失敗しました/)).not.toBeInTheDocument();
    expect(mockToastSuccess).toHaveBeenCalledWith("バックアップから復元しました");
    expect(consumeRestoreSuccessPending()).toBe(false);

    await vi.advanceTimersByTimeAsync(120_000);

    // release が 1 回だけ: 2 回なら別の復元の理由まで外れて確認が走る
    expect(mockCheckAutoBackup).toHaveBeenCalledTimes(callsAfterRestore);
  });

  it("REQ-901 / UI-11b-D15: 確かめの間（確認の待ちと inspectBackup の解決まで）は文言を出し、操作を押せない", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
    const pendingCheck = deferred<CheckResult>();
    const pendingInspect = deferred<Awaited<ReturnType<typeof commands.inspectBackup>>>();
    mockCheckAutoBackup.mockReturnValueOnce(pendingCheck.promise);
    mockInspectBackup.mockReturnValueOnce(pendingInspect.promise);

    const { queryClient } = renderWithClient(<PageWithAutoBackupCheck />);
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    try {
      await screen.findByText("7月3日 21:00");
      await vi.advanceTimersByTimeAsync(60_000);
      expect(mockCheckAutoBackup).toHaveBeenCalledTimes(1);

      await user.click(screen.getByRole("button", { name: "この控えに戻す" }));
      expect(screen.getByText("控えを確かめています")).toBeInTheDocument();
      expect(mockInspectBackup).not.toHaveBeenCalled();
      expect(screen.getByRole("button", { name: "閉じる" })).toBeDisabled();
      expect(screen.getByRole("button", { name: "今すぐバックアップを作成" })).toBeDisabled();
      expect(screen.getByRole("button", { name: "この控えに戻す" })).toBeDisabled();

      const listInvalidatesBefore = listInvalidateCount(invalidateSpy);
      pendingCheck.resolve(ok(true));
      await waitFor(() => {
        expect(mockInspectBackup).toHaveBeenCalledTimes(1);
      });
      expect(mockToastSuccess).not.toHaveBeenCalled();
      expect(listInvalidateCount(invalidateSpy)).toBe(listInvalidatesBefore);

      // inspectBackup の未解決の間も続く
      expect(screen.getByText("控えを確かめています")).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "閉じる" })).toBeDisabled();
      expect(screen.getByRole("button", { name: "今すぐバックアップを作成" })).toBeDisabled();
      await user.click(screen.getByRole("button", { name: "今すぐバックアップを作成" }));
      expect(mockCreateBackup).not.toHaveBeenCalled();
      expect(mockInspectBackup).toHaveBeenCalledTimes(1);

      pendingInspect.resolve(ok(makeInspection()));
      expect(await screen.findByText("この控えは戻せます")).toBeInTheDocument();
      expect(screen.queryByText("控えを確かめています")).not.toBeInTheDocument();
      expect(screen.getByRole("button", { name: "閉じる" })).toBeEnabled();
    } finally {
      pendingCheck.resolve(ok(false));
      pendingInspect.resolve(ok(makeInspection()));
      await vi.advanceTimersByTimeAsync(0);
    }
  });
});

it("SC3 / QR-05 / D-CE3: latest backup is an emphasis badge filled with warning-emphasis", async () => {
  renderWithClient(<BackupRestorePage />);
  const latest = await screen.findByText("最新");
  expect(latest).toHaveClass("border-warning", "bg-warning-emphasis", "text-primary-foreground");
  expect(latest).not.toHaveClass("bg-primary");
  expect(latest).not.toHaveAttribute("data-tone");
});

it("⑮ SC4: 確定済みの操作説明を全文表示する", () => {
  renderWithClient(<BackupRestorePage />);
  expect(
    screen.getByText(
      "アプリのデータ全体をまとめて保存し、必要なときに元に戻すためのページです。自動バックアップの時刻を設定したり、今すぐ手動でバックアップを作成したり、保存先を選んだりできます。過去のバックアップから復元すると現在の記録は元に戻せませんが、復元の前には自動で今の状態のバックアップが作られます。",
    ),
  ).toBeInTheDocument();
});

describe("控えを選んで確かめる の file 名の表示 (REQ-901 / UI-11b-D15・§68.9)", () => {
  it("REQ-901 / §68.9: 作成日時が取れない file は日時を作り出さず file 名を出す", async () => {
    const user = userEvent.setup();
    mockDefaultCommands();
    mockOpen.mockResolvedValueOnce("/media/usb/inventory.db");
    mockInspectBackup.mockResolvedValue(
      ok(makeInspection({ file_name: "inventory.db", created_at: null })),
    );
    renderWithClient(<BackupRestorePage />);

    await user.click(await screen.findByRole("button", { name: "控えを選んで確かめる" }));
    expect(await screen.findByText("inventory.db の控え")).toBeInTheDocument();
    expect(mockInspectBackup).toHaveBeenCalledWith({ backup_path: "/media/usb/inventory.db" });

    await user.click(screen.getByRole("button", { name: "復元の確認へ進む" }));
    const dialog = await screen.findByRole("alertdialog", { name: "元に戻せません" });
    expect(
      within(dialog).getByText(
        "inventory.db の控えに戻します。この控えより後に記録した内容は消えます。",
      ),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByRole("button", { name: "inventory.db の控えに戻す" }),
    ).toBeInTheDocument();
    expect(document.body.textContent).not.toMatch(/Invalid Date|1月1日 0:00|1月1日 00:00/);
  });
});
