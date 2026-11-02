import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useBlocker } from "@tanstack/react-router";
import { useCallback, useReducer, useState } from "react";
import { toast } from "sonner";
import { commands, CSV_IMPORT_FILE_SIZE_LIMIT } from "@/lib/bindings";
import type { DailyReportPreviewResponse, DailyReportSourceFileRequest } from "@/lib/bindings";
import { describeError } from "@/lib/describe-error";
import { extractFilename } from "@/lib/extractFilename";
import { invalidateByContract, invalidationContract } from "@/lib/invalidation-contract";
import { CMD_ERROR_KIND, InvokeError, isInvokeError, unwrapResult } from "@/lib/invoke";
import { open } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import { dailyReportImportReducer } from "../reducer";
import type { DailyReportErrorRecoverTo, DailyReportImportState } from "../types";

const INITIAL_STATE: DailyReportImportState = { status: "idle" };

/** UI-07-D16 の仮の系列（名前だけで示す。最終判定は IO-07） */
export const DAILY_REPORT_SERIES = ["Z001", "Z002", "Z005"] as const;
export type DailyReportSeries = (typeof DAILY_REPORT_SERIES)[number];

/** UI-07-D16 の選択の一覧の 1 行（idle の間だけ hook が持つ） */
export interface DailyReportManualFile {
  series: DailyReportSeries;
  filename: string;
  path: string;
  bytes: Uint8Array;
}

/** UI-07-D16 (d) の置き換えの確認を待つ file */
export interface DailyReportPendingReplacement {
  previous: DailyReportManualFile;
  incoming: DailyReportManualFile;
}

const UNKNOWN_SERIES_MESSAGE = "Z001・Z002・Z005 のどれかの名前のファイルを選んでください。";
const DUPLICATE_SERIES_MESSAGE =
  "Z001・Z002・Z005 は 1 つずつ選んでください（同じ種類のファイルが 2 つ以上あります）。";
const READ_FAILED_MESSAGE = "日報ファイルの選択または読み取りに失敗しました";
const TOO_LARGE_MESSAGE = "ファイルサイズが上限(20MB)を超えています";

function ensureInvokeError(error: unknown, cmd: string): InvokeError {
  if (isInvokeError(error)) return error;
  return new InvokeError(
    {
      kind: CMD_ERROR_KIND.INTERNAL,
      message: error instanceof Error ? error.message : String(error),
      field: null,
      error_id: null,
      stocktake_recovery: null,
    },
    { source: "commands", cmd },
  );
}

function decideRecoverTo(error: InvokeError): DailyReportErrorRecoverTo {
  if (error.cmdError.kind === CMD_ERROR_KIND.IMPORT_ERROR) return "idle";
  return "preview";
}

export const DAILY_REPORT_LAST_DIR_STORAGE_KEY = "inventory:daily-report-import:last-dir:v1";

function dirnameFromPath(path: string): string | null {
  const index = Math.max(path.lastIndexOf("\\"), path.lastIndexOf("/"));
  if (index <= 0) return null;
  return path.slice(0, index);
}

function readLastSelectedDir(): string | null {
  try {
    // localStorage can be unavailable in restricted WebView contexts.
    return window.localStorage.getItem(DAILY_REPORT_LAST_DIR_STORAGE_KEY);
  } catch {
    return null;
  }
}

function saveLastSelectedDir(path: string) {
  const dir = dirnameFromPath(path);
  if (dir === null) return;
  try {
    window.localStorage.setItem(DAILY_REPORT_LAST_DIR_STORAGE_KEY, dir);
  } catch {
    // 記憶できなくても選択フロー本体は継続する。
  }
}

function seriesOf(filename: string): DailyReportSeries | null {
  const upper = filename.toUpperCase();
  return DAILY_REPORT_SERIES.find((series) => upper.includes(series)) ?? null;
}

export function useDailyReportImportFlow() {
  const [state, dispatch] = useReducer(dailyReportImportReducer, INITIAL_STATE);
  const [lastSelectionError, setLastSelectionError] = useState<string | null>(null);
  const [scanError, setScanError] = useState<string | null>(null);
  const [manualFiles, setManualFiles] = useState<DailyReportManualFile[]>([]);
  const [pendingReplacements, setPendingReplacements] = useState<DailyReportPendingReplacement[]>(
    [],
  );
  const queryClient = useQueryClient();

  const onParsed = (data: DailyReportPreviewResponse) => {
    dispatch({
      type: "parse_succeeded",
      preview: data.preview_data,
      previewToken: data.preview_token,
    });
  };

  const parseMutation = useMutation({
    mutationFn: (files: DailyReportSourceFileRequest[]) =>
      unwrapResult(commands.parseAndValidateDailyReport(files), {
        source: "commands",
        cmd: "parse_and_validate_daily_report",
      }),
    retry: 0,
    onSuccess: onParsed,
    onError: (error: unknown) => {
      dispatch({
        type: "parse_failed",
        error: ensureInvokeError(error, "parse_and_validate_daily_report"),
      });
    },
  });

  const parseFromSdMutation = useMutation({
    mutationFn: (args: { scanToken: string; candidateKey: string }) =>
      unwrapResult(commands.parseAndValidateDailyReportFromSd(args.scanToken, args.candidateKey), {
        source: "commands",
        cmd: "parse_and_validate_daily_report_from_sd",
      }),
    retry: 0,
    onSuccess: onParsed,
    onError: (error: unknown) => {
      // 戻り先（sd_list）は reducer が状態で決める（kind・message で分岐しない）
      dispatch({
        type: "parse_failed",
        error: ensureInvokeError(error, "parse_and_validate_daily_report_from_sd"),
      });
    },
  });

  const scanMutation = useMutation({
    mutationFn: (selectedPath: string | null) =>
      unwrapResult(commands.scanRegisterSd(selectedPath), {
        source: "commands",
        cmd: "scan_register_sd",
      }),
    retry: 0,
    onSuccess: (data) => {
      dispatch({ type: "scan_succeeded", scan: data.scan, scanToken: data.scan_token });
    },
    onError: (error: unknown) => {
      // BIZ-08 §37.9 の固定の文を「SD から読む」の直下の 1 スロットに出す
      setScanError(describeError(ensureInvokeError(error, "scan_register_sd")));
      dispatch({ type: "scan_failed" });
    },
  });

  const commitMutation = useMutation({
    mutationFn: (args: {
      previewToken: string;
      additionalImportConfirmed: boolean;
      reportDate: string;
    }) =>
      unwrapResult(
        commands.commitDailyReportImport(args.previewToken, args.additionalImportConfirmed),
        {
          source: "commands",
          cmd: "commit_daily_report_import",
        },
      ),
    retry: 0,
    onSuccess: async (result, vars) => {
      dispatch({ type: "import_succeeded", result, reportDate: vars.reportDate });
      await invalidateByContract(queryClient, invalidationContract.dailyReportImport());
    },
    onError: (error: unknown) => {
      const e = ensureInvokeError(error, "commit_daily_report_import");
      dispatch({ type: "import_failed", error: e, recoverTo: decideRecoverTo(e) });
    },
  });

  const rollbackMutation = useMutation({
    mutationFn: (dailyReportImportId: number) =>
      unwrapResult(commands.rollbackDailyReportImport(dailyReportImportId), {
        source: "commands",
        cmd: "rollback_daily_report_import",
      }),
    retry: 0,
    onSuccess: async () => {
      dispatch({ type: "rollback_succeeded" });
      // Scope 7(a) UI 表示磨き batch: 取消は破壊的操作の完了通知であり、既定 3 秒
      // (RootLayout.tsx の Toaster duration) では非IT高齢operatorが読み切れない
      // おそれがあるため、この通知だけ個別に長く表示する（Toaster 全体設定は不変）。
      toast.success("日報取込みを取り消しました", { duration: 8000 });
      await invalidateByContract(queryClient, invalidationContract.dailyReportImport());
    },
    onError: () => {
      toast.error("取り消しに失敗しました。もう一度お試しください");
    },
  });

  // 取込み中と取消の応答を待つ間は画面を離れない（§55.6 の rollback の行「result 画面は維持」）
  const blocking = state.status === "importing" || rollbackMutation.isPending;
  useBlocker({
    shouldBlockFn: () => blocking,
    enableBeforeUnload: () => blocking,
  });

  const scanSd = useCallback(
    (selectedPath: string | null = null) => {
      setScanError(null);
      // UI-07-D16 (a): 選択の一覧は idle の間だけ持つ
      setManualFiles([]);
      setPendingReplacements([]);
      dispatch({ type: "scan_started" });
      scanMutation.mutate(selectedPath);
    },
    [scanMutation],
  );

  /** UI-07-D15 (1): 自動で見つからないときに folder を選んで読む */
  const chooseSdLocation = useCallback(async () => {
    const selected = await open({ directory: true });
    if (typeof selected !== "string") return;
    scanSd(selected);
  }, [scanSd]);

  const confirmSdCandidate = useCallback(
    (candidateKey: string) => {
      if (state.status !== "sd_list") return;
      const candidate = state.scan.candidates.find((item) => item.candidate_key === candidateKey);
      // 手で選んだときの文を SD から来た preview に持ち越さない
      setLastSelectionError(null);
      dispatch({
        type: "sd_candidate_selected",
        candidateKey,
        filenames: candidate?.source_filenames ?? [],
      });
      parseFromSdMutation.mutate({ scanToken: state.scanToken, candidateKey });
    },
    [state, parseFromSdMutation],
  );

  const backToSdList = useCallback(() => {
    dispatch({ type: "back_to_sd_list" });
  }, []);

  /** 一覧に足す（同じ系列は置き換えの確認へ。UI-07-D16 (b)・(d)） */
  const addManualFiles = useCallback(
    (incoming: DailyReportManualFile[], current: DailyReportManualFile[]) => {
      const next = [...current];
      const replacements: DailyReportPendingReplacement[] = [];
      for (const file of incoming) {
        const previous = next.find((item) => item.series === file.series);
        if (previous) {
          replacements.push({ previous, incoming: file });
        } else {
          next.push(file);
        }
      }
      setManualFiles(next);
      setPendingReplacements(replacements);
    },
    [],
  );

  /** UI-07-D16 (a): native file dialog で選んだ file を一覧に足す（preview・一覧からは idle の空の一覧へ） */
  const chooseFiles = useCallback(async () => {
    setLastSelectionError(null);
    try {
      const selected = await open({
        multiple: true,
        defaultPath: readLastSelectedDir() ?? undefined,
        filters: [{ name: "CSV", extensions: ["csv", "CSV"] }],
      });
      if (selected === null) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      if (paths.length === 0) return;
      saveLastSelectedDir(paths[0]);
      // (a) preview から 1 つ以上選んだら、足せる file が 0 本でも idle の空の一覧へ戻す（検査より前）
      if (state.status === "preview") dispatch({ type: "reset" });

      const named = paths.map((path) => ({
        path,
        filename: extractFilename(path),
        series: seriesOf(extractFilename(path)),
      }));
      const seriesInDialog = named.flatMap((item) => (item.series ? [item.series] : []));
      // (c) 1 回の dialog で同じ系列が 2 つ以上なら、その回の file をどれも足さない
      if (new Set(seriesInDialog).size !== seriesInDialog.length) {
        setLastSelectionError(DUPLICATE_SERIES_MESSAGE);
        return;
      }
      const files: DailyReportManualFile[] = [];
      for (const item of named) {
        if (!item.series) continue;
        const bytes = await readFile(item.path);
        // (f) 20MB を超える file は足さない
        if (bytes.byteLength > CSV_IMPORT_FILE_SIZE_LIMIT) {
          setLastSelectionError(TOO_LARGE_MESSAGE);
          toast.error(TOO_LARGE_MESSAGE);
          return;
        }
        files.push({ series: item.series, filename: item.filename, path: item.path, bytes });
      }
      if (files.length < named.length) setLastSelectionError(UNKNOWN_SERIES_MESSAGE);
      if (files.length === 0) return;
      // SD の一覧から足せたときは idle の空の一覧に足す（一覧は idle の間だけ持つので、ここでは空。足せないときは一覧を残す）
      if (state.status !== "idle" && state.status !== "preview") dispatch({ type: "reset" });
      addManualFiles(files, manualFiles);
    } catch {
      setLastSelectionError(READ_FAILED_MESSAGE);
      toast.error(READ_FAILED_MESSAGE);
    }
  }, [state.status, manualFiles, addManualFiles]);

  const removeManualFile = useCallback((series: DailyReportSeries) => {
    setManualFiles((files) => files.filter((file) => file.series !== series));
  }, []);

  /** UI-07-D16 (d) の「置き換える」 */
  const confirmReplacement = useCallback(() => {
    if (pendingReplacements.length === 0) return;
    const [first, ...rest] = pendingReplacements;
    setManualFiles((files) =>
      files.map((file) => (file.series === first.incoming.series ? first.incoming : file)),
    );
    setPendingReplacements(rest);
  }, [pendingReplacements]);

  /** UI-07-D16 (d) の「キャンセル」（一覧を変えない） */
  const cancelReplacement = useCallback(() => {
    setPendingReplacements((pending) => pending.slice(1));
  }, []);

  /** UI-07-D16 (e): 3 系列がそろったら 3 つの path・bytes で §45.3 を呼ぶ */
  const confirmManualFiles = useCallback(() => {
    const ordered = DAILY_REPORT_SERIES.map((series) =>
      manualFiles.find((file) => file.series === series),
    );
    if (state.status !== "idle" || ordered.some((file) => file === undefined)) return;
    const files = ordered.filter((file): file is DailyReportManualFile => file !== undefined);
    setLastSelectionError(null);
    setManualFiles([]);
    setPendingReplacements([]);
    dispatch({ type: "select_files", filenames: files.map((file) => file.filename) });
    parseMutation.mutate(
      files.map((file) => ({
        filename: file.filename,
        file_bytes: Array.from(file.bytes),
        source_path: file.path,
      })),
    );
  }, [state.status, manualFiles, parseMutation]);

  const confirmImport = useCallback(
    (additionalImportConfirmed: boolean) => {
      if (state.status !== "preview") return;
      dispatch({ type: "confirm_import", additionalImportConfirmed });
      commitMutation.mutate({
        previewToken: state.previewToken,
        additionalImportConfirmed,
        reportDate: state.preview.file_info.report_date,
      });
    },
    [state, commitMutation],
  );

  const rollback = useCallback(
    (dailyReportImportId: number) => {
      if (state.status !== "result") return;
      rollbackMutation.mutate(dailyReportImportId);
    },
    [state, rollbackMutation],
  );

  const dismissError = useCallback(() => {
    dispatch({ type: "dismiss_error" });
  }, []);

  return {
    state: { ...state, lastSelectionError },
    scanError,
    manualFiles,
    pendingReplacement: pendingReplacements.length > 0 ? pendingReplacements[0] : null,
    scanSd,
    chooseSdLocation,
    confirmSdCandidate,
    backToSdList,
    chooseFiles,
    removeManualFile,
    confirmReplacement,
    cancelReplacement,
    confirmManualFiles,
    confirmImport,
    rollback,
    dismissError,
    reset: () => {
      dispatch({ type: "reset" });
    },
    isParsing: parseMutation.isPending || parseFromSdMutation.isPending,
    isImporting: commitMutation.isPending,
    isRollingBack: rollbackMutation.isPending,
    /** 取込み中か取消の応答を待つ間（useBlocker と同じ条件。親のタブの切替えも止める、55 §55.7） */
    isBlocking: blocking,
  };
}
