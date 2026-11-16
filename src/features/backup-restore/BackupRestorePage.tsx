import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import {
  AlertTriangle,
  CheckCircle2,
  DatabaseBackup,
  FolderOpen,
  Loader2,
  RotateCcw,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { PageHeader } from "@/components/patterns/PageHeader";
import { PageShell } from "@/components/patterns/PageShell";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  commands,
  type AppSetting,
  type BackupInfo,
  type BackupInspection,
  type BackupResult,
  type CmdErrorKind,
} from "@/lib/bindings";
import { describeError } from "@/lib/describe-error";
import { isInvokeError, unwrapResult } from "@/lib/invoke";
import { queryKeys } from "@/lib/query-keys";
import {
  clearRestoreSuccessPending,
  setRestoreSuccessPending,
} from "@/lib/restore-success-notification";
import {
  holdAutoBackupCheckForRestore,
  releaseAutoBackupCheckAfterRestore,
  resumeAutoBackupCheck,
  suspendAutoBackupCheckAndWait,
  trackOffsiteOperation,
} from "./useAutoBackupCheck";
import { open } from "@tauri-apps/plugin-dialog";
import { formatBackupDate } from "./offsite-status-text";
import { OffsiteBackupPanel, type OffsiteAction, type OffsiteMessage } from "./OffsiteBackupPanel";
import { useOffsiteBackupStatus } from "./useOffsiteBackupStatus";

const BACKUP_SETTING_KEYS = new Set([
  "backup_enabled",
  "backup_time",
  "backup_path",
  "backup_retention_days",
]);

function settingValue(settings: AppSetting[] | undefined, key: string, fallback: string): string {
  return settings?.find((setting) => setting.key === key)?.value ?? fallback;
}

function formatBackupSize(sizeBytes: number): string {
  return `${(sizeBytes / 1_000_000).toFixed(1)} MB`;
}

function restoreErrorKind(error: unknown): CmdErrorKind | null {
  return isInvokeError(error) ? error.cmdError.kind : null;
}

function restoreErrorId(error: unknown): string | null {
  return isInvokeError(error) ? (error.cmdError.error_id ?? null) : null;
}

// UI-11b-D15: 止める文言（§68.9 の文の完全一致）
const BLOCKED_NEWER =
  "この控えは、より新しい版のアプリで作られています。この版のアプリでは戻せません。新しい版のアプリを入れてから戻してください（今のデータは変わっていません）。";
const BLOCKED_CORRUPT = "この控えは壊れているため戻せません。別の控えを選んでください。";
const BLOCKED_UNREADABLE = "この控えを読めませんでした。別の控えを選んでください。";

// §68.7 の restore_inspecting / restore_blocked / restore_detail
type RestoreCheck =
  | { status: "inspecting" }
  | { status: "blocked"; message: string }
  | { status: "restorable"; inspection: BackupInspection };

function checkFromInspection(inspection: BackupInspection): RestoreCheck {
  if (inspection.newer_than_app) return { status: "blocked", message: BLOCKED_NEWER };
  if (!inspection.quick_check_ok) return { status: "blocked", message: BLOCKED_CORRUPT };
  return { status: "restorable", inspection };
}

// 復元の対象。一覧の行（BackupInfo）と「控えを選んで確かめる」で選んだ file（UI-11b-D15）。
// file の作成日時・サイズは確かめ（inspectBackup）が返すまで分からない（§68.9: 取れなければ file 名を出す）
interface RestoreTarget {
  file_path: string;
  file_name: string;
  created_at: string | null;
  size_bytes: number | null;
}

function targetFromBackup(backup: BackupInfo): RestoreTarget {
  return {
    file_path: backup.file_path,
    file_name: backup.file_name,
    created_at: backup.created_at,
    size_bytes: backup.size_bytes,
  };
}

function targetFromFile(path: string): RestoreTarget {
  const file_name = path.split(/[\\/]/).pop() ?? path;
  return { file_path: path, file_name, created_at: null, size_bytes: null };
}

/** 詳細の見出し・最終確認の日時。作成日時が取れない file は日時を作り出さず file 名（§68.9） */
function restoreTargetLabel(target: RestoreTarget | null): string {
  if (target === null) return "";
  return target.created_at !== null ? formatBackupDate(target.created_at) : target.file_name;
}

/** §68.7: 復元の選択中（詳細・最終確認を含む）と busy / fatal の間は、用意・作り直し・やめるを止める */
function offsiteOperationsLocked(
  controlsDisabled: boolean,
  selected: RestoreTarget | null,
): boolean {
  return controlsDisabled || selected !== null;
}

function RestoreTargetDetails({ target }: { target: RestoreTarget }) {
  return (
    <>
      {target.size_bytes !== null ? (
        <p className="text-sm text-muted-foreground">
          サイズ: {formatBackupSize(target.size_bytes)}
        </p>
      ) : null}
      <p className="text-sm break-all text-muted-foreground">{target.file_path}</p>
    </>
  );
}

/** §68.11 の backup_path 保存失敗: 設定の card の中に文（現在の保存先の表示は保つ） */
function PathUpdateError({ message }: { message: string | null }) {
  if (message === null) return null;
  return (
    <Alert variant="destructive">
      <AlertDescription>
        <p>{message}</p>
      </AlertDescription>
    </Alert>
  );
}

interface RestoreState {
  selected: RestoreTarget | null;
  check: RestoreCheck | null;
  preBackupFailed: boolean;
  breakGlassAccepted: boolean;
  confirmOpen: boolean;
}

const initialRestoreState: RestoreState = {
  selected: null,
  check: null,
  preBackupFailed: false,
  breakGlassAccepted: false,
  confirmOpen: false,
};

/**
 * UI-11b-D14・§68.4: 「PC の外の控え」の card の 3 つの操作（§68.7 の offsite_preparing / offsite_resetting /
 * offsite_stopping）。成功・失敗とも status を取り直し（作り直しは一覧も）、card は取り直した status で決める（§68.10）。
 * 実行中かは画面の state（action）で持つが、離脱で消えるので、command の promise は module に登録する（§68.7）
 */
function useOffsiteActions() {
  const queryClient = useQueryClient();
  const [action, setAction] = useState<OffsiteAction | null>(null);
  const [message, setMessage] = useState<OffsiteMessage | null>(null);

  async function run(kind: OffsiteAction, operation: () => Promise<string>, alsoList: boolean) {
    setAction(kind);
    setMessage(null);
    try {
      setMessage({ tone: "success", text: await trackOffsiteOperation(operation()) });
    } catch (error) {
      setMessage({ tone: "error", text: describeError(error) });
    } finally {
      await queryClient.invalidateQueries({ queryKey: queryKeys.backupRestore.offsiteStatus() });
      if (alsoList) {
        await queryClient.invalidateQueries({ queryKey: queryKeys.backupRestore.list() });
      }
      setAction(null);
    }
  }

  async function prepare() {
    const selected = await open({ directory: true, multiple: false });
    if (!selected || Array.isArray(selected)) return;
    await run(
      "preparing",
      async () => {
        const medium = await unwrapResult(
          commands.prepareOffsiteMedium({ selected_path: selected }),
          { source: "commands", cmd: "prepare_offsite_medium" },
        );
        return `${medium.label} を PC の外の控えの保存先にしました`;
      },
      false,
    );
  }

  // CMD が先に backup を作るので、失敗でも一覧を取り直す（§68.4 手順 4）
  async function reset() {
    await run(
      "resetting",
      async () => {
        await unwrapResult(commands.resetOffsiteState(), {
          source: "commands",
          cmd: "reset_offsite_state",
        });
        return "PC の外の控えの記録を作り直しました";
      },
      true,
    );
  }

  async function stop() {
    await run(
      "stopping",
      async () => {
        await unwrapResult(commands.stopOffsiteBackup(), {
          source: "commands",
          cmd: "stop_offsite_backup",
        });
        return "PC の外の控えをやめました";
      },
      false,
    );
  }

  return { action, message, prepare, reset, stop };
}

export function BackupRestorePage() {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const [restoreState, setRestoreState] = useState<RestoreState>(initialRestoreState);
  const [manualBackupResult, setManualBackupResult] = useState<BackupResult | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [restoreFailureErrorId, setRestoreFailureErrorId] = useState<string | null>(null);
  const [isCreatingBackup, setIsCreatingBackup] = useState(false);
  const [isUpdatingSetting, setIsUpdatingSetting] = useState(false);
  const [isRunningPreBackup, setIsRunningPreBackup] = useState(false);
  const [isRestoring, setIsRestoring] = useState(false);
  const offsite = useOffsiteActions();
  // §68.4 の backup_path 変更の手順 3（path_update_error）。設定の card の中に出す
  const [pathError, setPathError] = useState<string | null>(null);
  const [fatalRestoreKind, setFatalRestoreKind] = useState<Extract<
    CmdErrorKind,
    "restore_failed_unrecoverable" | "restore_durability_unknown"
  > | null>(null);

  const settingsQuery = useQuery({
    queryKey: queryKeys.backupRestore.settings(),
    queryFn: () =>
      unwrapResult(commands.getSettings(), { source: "commands", cmd: "get_settings" }),
  });

  const backupsQuery = useQuery({
    queryKey: queryKeys.backupRestore.list(),
    queryFn: () =>
      unwrapResult(commands.listBackups(), { source: "commands", cmd: "list_backups" }),
  });

  const offsiteStatusQuery = useOffsiteBackupStatus();

  const effectiveBackupDirQuery = useQuery({
    queryKey: queryKeys.backupRestore.effectiveDir(),
    queryFn: () =>
      unwrapResult(commands.getEffectiveBackupDir(), {
        source: "commands",
        cmd: "get_effective_backup_dir",
      }),
  });

  const settings = settingsQuery.data;
  const backupEnabled = settingValue(settings, "backup_enabled", "1") !== "0";
  const backupTime = settingValue(settings, "backup_time", "23:00");
  const backupPath = settingValue(settings, "backup_path", "");
  const backupRetentionDays = settingValue(settings, "backup_retention_days", "3");
  const backups = useMemo(() => backupsQuery.data ?? [], [backupsQuery.data]);
  const selectedBackupLabel = restoreTargetLabel(restoreState.selected);
  const isFatal = fatalRestoreKind !== null;
  const isInspecting = restoreState.check?.status === "inspecting";
  // §68.7: 用意・作り直し・やめるの実行中も busy（一覧の行と「控えを選んで確かめる」も止める）
  const isBusy =
    isCreatingBackup ||
    isUpdatingSetting ||
    isRunningPreBackup ||
    isRestoring ||
    isInspecting ||
    offsite.action !== null;
  const controlsDisabled = isFatal || isBusy;
  // §68.7: 復元の選択中（詳細・最終確認を含む）も用意・作り直し・やめるを止める（確認の本体が選んだ控えを掃除しない）
  const offsiteOperationsDisabled = offsiteOperationsLocked(
    controlsDisabled,
    restoreState.selected,
  );

  const backupRows = useMemo(
    () =>
      backups.map((backup, index) => ({
        backup,
        displayDate: formatBackupDate(backup.created_at),
        displaySize: formatBackupSize(backup.size_bytes),
        isLatest: index === 0,
      })),
    [backups],
  );

  // UI-11b-D15: 画面を離れたら選択の理由を外す（復元の理由は module に残る）
  useEffect(
    () => () => {
      resumeAutoBackupCheck();
    },
    [],
  );

  async function refetchBackupState() {
    await Promise.all([settingsQuery.refetch(), backupsQuery.refetch()]);
  }

  async function updateBackupSetting(key: string, value: string) {
    if (!BACKUP_SETTING_KEYS.has(key)) return;
    setIsUpdatingSetting(true);
    setErrorMessage(null);
    setRestoreFailureErrorId(null);
    setStatusMessage(null);
    try {
      await unwrapResult(commands.updateSetting({ key, value }), {
        source: "commands",
        cmd: "update_setting",
      });
      await queryClient.invalidateQueries({ queryKey: queryKeys.backupRestore.settings() });
      if (key === "backup_path") {
        await queryClient.invalidateQueries({ queryKey: queryKeys.backupRestore.list() });
        await queryClient.invalidateQueries({ queryKey: queryKeys.backupRestore.effectiveDir() });
      }
      setStatusMessage("設定を保存しました");
      toast.success("バックアップ設定を保存しました");
    } catch (error) {
      setErrorMessage(describeError(error));
      toast.error("バックアップ設定を保存できませんでした");
    } finally {
      setIsUpdatingSetting(false);
    }
  }

  async function handleChooseBackupPath() {
    const selected = await open({ directory: true, multiple: false });
    if (!selected || Array.isArray(selected)) return;
    setPathError(null);
    // §68.4 の backup_path 変更の手順 3: 保存の前に確かめ、Err・例外は保存しない（71 MNT-01-D11）
    setIsUpdatingSetting(true);
    try {
      await unwrapResult(commands.checkBackupDirChoice({ selected_path: selected }), {
        source: "commands",
        cmd: "check_backup_dir_choice",
      });
    } catch (error) {
      setPathError(describeError(error));
      return;
    } finally {
      setIsUpdatingSetting(false);
    }
    await updateBackupSetting("backup_path", selected);
  }

  async function handleChooseBackupFile() {
    if (controlsDisabled) return;
    const selected = await open({
      directory: false,
      multiple: false,
      filters: [{ name: "バックアップ", extensions: ["db"] }],
    });
    // cancel は何もしない（止めない）
    if (!selected || Array.isArray(selected)) return;
    await selectRestoreBackup(targetFromFile(selected));
  }

  async function handleManualBackup() {
    setIsCreatingBackup(true);
    setManualBackupResult(null);
    setErrorMessage(null);
    setRestoreFailureErrorId(null);
    setStatusMessage(null);
    try {
      const result = await unwrapResult(commands.createBackup(), {
        source: "commands",
        cmd: "create_backup",
      });
      setManualBackupResult(result);
      await queryClient.invalidateQueries({ queryKey: queryKeys.backupRestore.list() });
      setStatusMessage("バックアップを作成しました");
      toast.success("バックアップを作成しました");
    } catch (error) {
      setErrorMessage(describeError(error));
      toast.error("バックアップを作成できませんでした");
      // §68.4 手順 4: 公開の後の失敗では確かめ済みの正式名が残りうるので、失敗でも取り直す
      await queryClient.invalidateQueries({ queryKey: queryKeys.backupRestore.list() });
    } finally {
      setIsCreatingBackup(false);
    }
  }

  async function selectRestoreBackup(backup: RestoreTarget) {
    setErrorMessage(null);
    setRestoreFailureErrorId(null);
    setStatusMessage(null);
    setRestoreState({ ...initialRestoreState, selected: backup, check: { status: "inspecting" } });
    // UI-11b-D15: 確かめの前に共通の確認を止め、実行中の回を待つ（待つ間に届いた結果は捨てる）
    await suspendAutoBackupCheckAndWait();
    let check: RestoreCheck;
    try {
      const inspection = await unwrapResult(
        commands.inspectBackup({ backup_path: backup.file_path }),
        { source: "commands", cmd: "inspect_backup" },
      );
      check = checkFromInspection(inspection);
      // 選んだ file の作成日時・サイズは確かめの結果から（取れなければ null のまま、§68.9）
      const known: RestoreTarget = {
        ...backup,
        created_at: backup.created_at ?? inspection.created_at,
        size_bytes: backup.size_bytes ?? inspection.size_bytes,
      };
      setRestoreState((current) =>
        current.selected === backup ? { ...current, selected: known, check } : current,
      );
      return;
    } catch {
      // CmdError（読めない控え）も IPC の例外も同じ固定の文。CmdError の message は画面に出さない
      check = { status: "blocked", message: BLOCKED_UNREADABLE };
    }
    setRestoreState((current) => (current.selected === backup ? { ...current, check } : current));
  }

  function handleCloseRestore() {
    setRestoreState(initialRestoreState);
    setErrorMessage(null);
    setRestoreFailureErrorId(null);
    resumeAutoBackupCheck();
    // 停止の待ちで捨てた確認の `true` が作った backup を一覧に出す（§68.10）
    void queryClient.invalidateQueries({ queryKey: queryKeys.backupRestore.list() });
  }

  async function handleStartRestoreConfirmation() {
    if (!restoreState.selected) return;
    setIsRunningPreBackup(true);
    setErrorMessage(null);
    setRestoreFailureErrorId(null);
    setStatusMessage(null);
    try {
      await unwrapResult(commands.createBackup(), {
        source: "commands",
        cmd: "create_backup",
      });
      await queryClient.invalidateQueries({ queryKey: queryKeys.backupRestore.list() });
      setRestoreState((current) => ({ ...current, confirmOpen: true }));
    } catch (error) {
      setErrorMessage(describeError(error));
      setRestoreState((current) => ({
        ...current,
        preBackupFailed: true,
        breakGlassAccepted: false,
        confirmOpen: false,
      }));
    } finally {
      setIsRunningPreBackup(false);
    }
  }

  async function handleRestore() {
    if (!restoreState.selected) return;
    setIsRestoring(true);
    setErrorMessage(null);
    setRestoreFailureErrorId(null);
    setStatusMessage(null);
    setRestoreState((current) => ({ ...current, confirmOpen: false }));
    // UI-11b-D13: 復元の理由を立て、この画面の選択の理由を復元の理由へ移す
    holdAutoBackupCheckForRestore();
    let fatal = false;
    // §68.7: CMD の Ok で成功を確定する。その後の画面の更新の失敗を復元の失敗と取り違えない
    let restored = false;
    try {
      await unwrapResult(commands.restoreBackup({ backup_path: restoreState.selected.file_path }), {
        source: "commands",
        cmd: "restore_backup",
      });
      restored = true;
      queryClient.clear();
      // UI-11b-D11: 遷移先ホームで success Alert を表示する in-memory one-shot flag
      setRestoreSuccessPending();
      await navigate({ to: "/" });
    } catch (error) {
      if (restored) {
        // 復元は成功している（成功の後の clear・flag・navigate の失敗）。失敗の表示・state にせず、
        // 次回到達での誤表示を防ぐため flag を消し、home Alert を出せない代わりに成功の toast を出す
        // （無通知への劣化を防ぐ fallback。primary path の navigate 成功時は home Alert に一本化）
        clearRestoreSuccessPending();
        console.warn("復元成功後の画面の更新に失敗しました:", error);
        toast.success("バックアップから復元しました");
        return;
      }
      const kind = restoreErrorKind(error);
      setRestoreFailureErrorId(restoreErrorId(error));
      if (kind === "restore_failed_unrecoverable" || kind === "restore_durability_unknown") {
        // UI-11b-D13: fatal 2 種は再起動まで確認を止めたままにする
        fatal = true;
        setFatalRestoreKind(kind);
        setErrorMessage(null);
      } else {
        // §68.7: restore_failed_recovered・IPC の例外の後は未選択へ。再試行は選び直して確かめから
        setRestoreState(initialRestoreState);
        setErrorMessage(
          "バックアップの復元に失敗しました。現在のデータには戻しています。もう一度お試しください。",
        );
        await refetchBackupState();
      }
    } finally {
      // UI-11b-D13: 結果の後に 1 回だけ。成功の後の処理が throw して catch へ落ちても 2 回にしない。
      // resume は呼ばない（離れた画面の handler が再訪した画面の選択の理由を外さない）
      releaseAutoBackupCheckAfterRestore(fatal);
      setIsRestoring(false);
    }
  }

  const loadError = settingsQuery.isError || backupsQuery.isError;

  return (
    <PageShell>
      <PageHeader
        title="バックアップ・復元"
        description="アプリのデータ全体をまとめて保存し、必要なときに元に戻すためのページです。自動バックアップの時刻を設定したり、今すぐ手動でバックアップを作成したり、保存先を選んだりできます。過去のバックアップから復元すると現在の記録は元に戻せませんが、復元の前には自動で今の状態のバックアップが作られます。"
      />

      {fatalRestoreKind ? (
        <Alert variant="destructive">
          <AlertTitle>
            {fatalRestoreKind === "restore_durability_unknown"
              ? "復元結果を確認できませんでした"
              : "再起動が必要です"}
          </AlertTitle>
          <AlertDescription className="space-y-2">
            <p>
              {fatalRestoreKind === "restore_durability_unknown"
                ? "復元が完了したか確定できませんでした。"
                : "バックアップの復元に失敗し、DB接続の復旧もできませんでした。"}
            </p>
            <p className="font-medium">アプリを閉じて、もう一度開いてください</p>
            {restoreFailureErrorId ? <p>（エラーID: {restoreFailureErrorId}）</p> : null}
          </AlertDescription>
        </Alert>
      ) : null}

      {statusMessage ? (
        <Alert className="border-success bg-success-soft text-success-strong">
          <CheckCircle2 />
          <AlertTitle>{statusMessage}</AlertTitle>
          <AlertDescription className="space-y-1">
            {manualBackupResult ? (
              <>
                <p>
                  {manualBackupResult.file_name}（{formatBackupSize(manualBackupResult.size_bytes)}
                  ）
                </p>
                <p className="text-sm break-all">保存先: {manualBackupResult.file_path}</p>
              </>
            ) : (
              <p>最新の状態を画面に反映しました。</p>
            )}
          </AlertDescription>
        </Alert>
      ) : null}

      {errorMessage ? (
        <Alert variant="destructive">
          <AlertTitle>
            {restoreState.preBackupFailed
              ? "復元前のバックアップを作成できませんでした"
              : "操作に失敗しました"}
          </AlertTitle>
          <AlertDescription className="space-y-2">
            <p>{errorMessage}</p>
            {restoreFailureErrorId ? <p>（エラーID: {restoreFailureErrorId}）</p> : null}
          </AlertDescription>
        </Alert>
      ) : null}

      {loadError ? (
        <Alert variant="destructive">
          <AlertTitle>バックアップ情報を取得できませんでした</AlertTitle>
          <AlertDescription className="space-y-3">
            <p>設定とバックアップ一覧をもう一度読み込んでください。</p>
            <Button type="button" variant="outline" onClick={() => void refetchBackupState()}>
              <RotateCcw />
              再読込
            </Button>
          </AlertDescription>
        </Alert>
      ) : null}

      <fieldset disabled={controlsDisabled} className="space-y-6 disabled:opacity-70">
        <section className="grid gap-4 lg:grid-cols-[minmax(0,420px)_minmax(0,1fr)]">
          <Card>
            <CardHeader>
              <CardTitle>バックアップ設定</CardTitle>
            </CardHeader>
            <CardContent className="space-y-5">
              <div className="flex items-start gap-3">
                <Checkbox
                  id="backup-enabled"
                  checked={backupEnabled}
                  onCheckedChange={(checked) =>
                    void updateBackupSetting("backup_enabled", checked === true ? "1" : "0")
                  }
                />
                <div className="grid gap-1">
                  <Label htmlFor="backup-enabled">自動バックアップを使う</Label>
                  <p className="text-sm text-muted-foreground">
                    指定時刻を過ぎたら 60 秒ごとの確認でバックアップを作成します。
                  </p>
                </div>
              </div>

              <div className="grid gap-2">
                <Label htmlFor="backup-time">自動バックアップ時刻</Label>
                <Input
                  id="backup-time"
                  type="time"
                  defaultValue={backupTime}
                  onBlur={(event) =>
                    void updateBackupSetting("backup_time", event.currentTarget.value)
                  }
                />
              </div>

              <div className="grid gap-2">
                <Label htmlFor="backup-retention-days">保存日数</Label>
                <Input
                  id="backup-retention-days"
                  type="number"
                  min={1}
                  defaultValue={backupRetentionDays}
                  onBlur={(event) =>
                    void updateBackupSetting("backup_retention_days", event.currentTarget.value)
                  }
                />
              </div>

              <Separator />

              <div className="space-y-2">
                {effectiveBackupDirQuery.isSuccess ? (
                  <>
                    <p className="text-sm font-medium">現在の保存先</p>
                    <p className="rounded-md border bg-muted px-3 py-2 text-sm break-all text-muted-foreground">
                      {effectiveBackupDirQuery.data}
                    </p>
                    {backupPath ? null : (
                      <p className="text-xs text-muted-foreground">
                        保存先が未設定のためアプリ既定フォルダに保存されます
                      </p>
                    )}
                  </>
                ) : null}
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => void handleChooseBackupPath()}
                >
                  <FolderOpen />
                  保存先を選ぶ
                </Button>
                <PathUpdateError message={pathError} />
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>手動バックアップ</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <p className="text-sm text-muted-foreground">
                今のデータを控えとして保存します。復元前にも自動で同じバックアップを作成します。
              </p>
              <Button type="button" onClick={() => void handleManualBackup()}>
                {isCreatingBackup ? <Loader2 className="animate-spin" /> : <DatabaseBackup />}
                今すぐバックアップを作成
              </Button>
            </CardContent>
          </Card>
        </section>

        <OffsiteBackupPanel
          status={offsiteStatusQuery.data}
          isError={offsiteStatusQuery.isError}
          onRetry={() => void offsiteStatusQuery.refetch()}
          operationsDisabled={offsiteOperationsDisabled}
          fileEntryDisabled={controlsDisabled}
          busy={offsite.action}
          message={offsite.message}
          onPrepare={() => {
            if (!offsiteOperationsDisabled) void offsite.prepare();
          }}
          onReset={() => {
            if (!offsiteOperationsDisabled) void offsite.reset();
          }}
          onStop={() => {
            if (!offsiteOperationsDisabled) void offsite.stop();
          }}
          onChooseFile={() => void handleChooseBackupFile()}
        />

        <section className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(320px,420px)]">
          <Card>
            <CardHeader>
              <CardTitle>バックアップ一覧</CardTitle>
            </CardHeader>
            <CardContent>
              {settingsQuery.isLoading || backupsQuery.isLoading ? (
                <p className="text-sm text-muted-foreground">読込み中です。</p>
              ) : null}

              {backupsQuery.isSuccess && backupRows.length === 0 ? (
                <div className="space-y-2 rounded-md border border-dashed p-4">
                  <p className="font-medium">まだバックアップはありません</p>
                  <p className="text-sm text-muted-foreground">
                    復元に使う控えを作るには、手動バックアップを実行してください。
                  </p>
                </div>
              ) : null}

              {backupRows.length > 0 ? (
                <div className="overflow-x-auto">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>日時</TableHead>
                        <TableHead>サイズ</TableHead>
                        <TableHead>ファイル名</TableHead>
                        <TableHead className="text-right">操作</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {backupRows.map(({ backup, displayDate, displaySize, isLatest }) => (
                        <TableRow key={backup.file_path}>
                          <TableCell>
                            <div className="flex flex-wrap items-center gap-2">
                              <span className="font-medium">{displayDate}</span>
                              {isLatest ? (
                                <Badge className="border-warning bg-warning-emphasis text-primary-foreground">
                                  最新
                                </Badge>
                              ) : null}
                            </div>
                          </TableCell>
                          <TableCell className="tabular-nums">{displaySize}</TableCell>
                          <TableCell className="max-w-[18rem] text-sm break-all text-muted-foreground">
                            {backup.file_name}
                          </TableCell>
                          <TableCell className="text-right">
                            <Button
                              type="button"
                              variant="outline"
                              size="sm"
                              onClick={() => {
                                void selectRestoreBackup(targetFromBackup(backup));
                              }}
                              disabled={controlsDisabled}
                            >
                              この控えに戻す
                            </Button>
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </div>
              ) : null}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>復元</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              {restoreState.selected ? (
                <>
                  <div className="space-y-2">
                    <p className="text-lg font-semibold">{selectedBackupLabel} の控え</p>
                    <RestoreTargetDetails target={restoreState.selected} />
                  </div>
                  {restoreState.check?.status === "inspecting" ? (
                    <p className="flex items-center gap-2 text-sm" role="status">
                      <Loader2 className="animate-spin" aria-hidden="true" />
                      控えを確かめています
                    </p>
                  ) : null}
                  {restoreState.check?.status === "blocked" ? (
                    <Alert variant="destructive">
                      <AlertTitle>復元できません</AlertTitle>
                      <AlertDescription>
                        <p>{restoreState.check.message}</p>
                      </AlertDescription>
                    </Alert>
                  ) : null}
                  {restoreState.check?.status === "restorable" ? (
                    <>
                      <div className="space-y-1 rounded-md border p-3">
                        <p className="flex items-center gap-2 font-medium">
                          <CheckCircle2 className="size-4 text-success" aria-hidden="true" />
                          この控えは戻せます
                        </p>
                        {restoreState.check.inspection.product_count !== null ? (
                          <p className="text-sm">
                            商品の数: {restoreState.check.inspection.product_count} 件
                          </p>
                        ) : null}
                        {restoreState.check.inspection.last_operation_at !== null ? (
                          <p className="text-sm">
                            最後の記録:{" "}
                            {formatBackupDate(restoreState.check.inspection.last_operation_at)}
                          </p>
                        ) : null}
                      </div>
                      <Alert variant="warning">
                        <AlertTriangle />
                        <AlertTitle>復元すると今の記録は戻せません</AlertTitle>
                        <AlertDescription>
                          この時点の状態に戻ります。この控えより後に記録した内容は消えます
                        </AlertDescription>
                      </Alert>
                      {restoreState.preBackupFailed ? (
                        <div className="space-y-3 rounded-md border p-3">
                          <div className="flex items-start gap-3">
                            <Checkbox
                              id="break-glass-restore"
                              checked={restoreState.breakGlassAccepted}
                              onCheckedChange={(checked) => {
                                setRestoreState((current) => ({
                                  ...current,
                                  breakGlassAccepted: checked === true,
                                }));
                              }}
                            />
                            <Label htmlFor="break-glass-restore">
                              今の状態は保存できませんが、復元を続けます
                            </Label>
                          </div>
                          <Button
                            type="button"
                            onClick={() => {
                              setRestoreState((current) => ({ ...current, confirmOpen: true }));
                            }}
                            disabled={!restoreState.breakGlassAccepted || controlsDisabled}
                          >
                            最終確認へ進む
                          </Button>
                        </div>
                      ) : (
                        <Button
                          type="button"
                          onClick={() => void handleStartRestoreConfirmation()}
                          disabled={controlsDisabled}
                        >
                          {isRunningPreBackup ? <Loader2 className="animate-spin" /> : null}
                          復元の確認へ進む
                        </Button>
                      )}
                    </>
                  ) : null}
                  {isRestoring ? null : (
                    <Button
                      type="button"
                      variant="outline"
                      onClick={handleCloseRestore}
                      disabled={controlsDisabled}
                    >
                      閉じる
                    </Button>
                  )}
                </>
              ) : (
                <p className="text-sm text-muted-foreground">
                  一覧から戻したい控えを選ぶと、内容と確認手順を表示します。
                </p>
              )}
            </CardContent>
          </Card>
        </section>
      </fieldset>

      <RestoreConfirmDialog
        open={restoreState.confirmOpen}
        label={selectedBackupLabel}
        onOpenChange={(open) => {
          setRestoreState((current) => ({ ...current, confirmOpen: open }));
        }}
        onRestore={() => void handleRestore()}
      />
    </PageShell>
  );
}

interface RestoreConfirmDialogProps {
  open: boolean;
  label: string;
  onOpenChange: (open: boolean) => void;
  onRestore: () => void;
}

function RestoreConfirmDialog({ open, label, onOpenChange, onRestore }: RestoreConfirmDialogProps) {
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>元に戻せません</AlertDialogTitle>
          <AlertDialogDescription className="space-y-2">
            <span className="block">
              {label} の控えに戻します。この控えより後に記録した内容は消えます。
            </span>
            <span className="block">復元後、画面はホームに戻ります。</span>
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>キャンセル</AlertDialogCancel>
          <AlertDialogAction
            variant="destructive"
            onClick={(event) => {
              event.preventDefault();
              onRestore();
            }}
          >
            {label} の控えに戻す
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
