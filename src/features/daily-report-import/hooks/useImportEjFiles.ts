import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useBlocker } from "@tanstack/react-router";
import { useCallback, useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import { commands, CSV_IMPORT_FILE_SIZE_LIMIT } from "@/lib/bindings";
import type { EjFileRequest, EjImportResult } from "@/lib/bindings";
import { describeError } from "@/lib/describe-error";
import { extractFilename } from "@/lib/extractFilename";
import { invalidateByContract, invalidationContract } from "@/lib/invalidation-contract";
import { unwrapResult } from "@/lib/invoke";
import { useWindowCloseGuard } from "@/lib/window-close-guard";
import { open } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";

const EJ_READ_FAILED_MESSAGE = "EJ のファイルの選択または読み取りに失敗しました";
const TOO_LARGE_MESSAGE = "ファイルサイズが上限(20MB)を超えています";

/**
 * 手で選ぶ EJ の取込み（CMD-12 importEjFiles）。55 UI-07-D17 の一括の結果と UI-09c-D5 の控えの画面が共有する。
 * native の file dialog・20MB の検査・読込みの失敗の文・応答を待つ間の止め（useBlocker と close の `ej-import`、
 * UI-USW-D5・55 §55.7）を持ち、成功で D-052 C27 を当てる。最後に成功した結果は再試行が失敗しても `reset()` まで保つ
 */
export function useImportEjFiles(options: { onSuccess?: (result: EjImportResult) => void } = {}) {
  const { onSuccess } = options;
  const queryClient = useQueryClient();
  const [result, setResult] = useState<EjImportResult | null>(null);
  const [selectionError, setSelectionError] = useState<string | null>(null);
  const mutation = useMutation({
    mutationFn: (files: EjFileRequest[]) =>
      unwrapResult(commands.importEjFiles(files), {
        source: "commands",
        cmd: "import_ej_files",
      }),
    retry: 0,
    onSuccess: async (data) => {
      setResult(data);
      onSuccess?.(data);
      // data の無い query の走っている読取り（取込みの前に読んだもの）に無効化が相乗りしないよう、先に止める
      await Promise.all(
        invalidationContract.ejImport().map((queryKey) => queryClient.cancelQueries({ queryKey })),
      );
      await invalidateByContract(queryClient, invalidationContract.ejImport());
    },
  });
  // 選択の確定（readFile の待ち）から command の終わりまでの 1 つの待ち（UI-USW-D5 の止め。解くのは choose の finally だけ）
  const [pending, setPending] = useState(false);
  const mountedRef = useRef(true);
  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
    };
  }, []);

  useBlocker({
    shouldBlockFn: () => pending,
    enableBeforeUnload: () => pending,
  });
  useWindowCloseGuard("ej-import", pending);

  const { mutateAsync, reset: resetMutation } = mutation;

  const choose = useCallback(async () => {
    setSelectionError(null);
    let paths: string[];
    try {
      const selected = await open({
        multiple: true,
        filters: [{ name: "EJ", extensions: ["txt", "TXT"] }],
      });
      if (selected === null) return;
      paths = Array.isArray(selected) ? selected : [selected];
    } catch {
      setSelectionError(EJ_READ_FAILED_MESSAGE);
      toast.error(EJ_READ_FAILED_MESSAGE);
      return;
    }
    if (paths.length === 0) return;
    setPending(true);
    try {
      const files: EjFileRequest[] = [];
      for (const path of paths) {
        const bytes = await readFile(path);
        if (bytes.byteLength > CSV_IMPORT_FILE_SIZE_LIMIT) {
          setSelectionError(TOO_LARGE_MESSAGE);
          toast.error(TOO_LARGE_MESSAGE);
          return;
        }
        files.push({ filename: extractFilename(path), file_bytes: Array.from(bytes) });
      }
      // 読む間に画面を離れていたら取り込まない（結果を見る画面が無い）
      if (!mountedRef.current) return;
      // command の失敗は mutation の error が持つ（読込みの失敗の文にしない）
      await mutateAsync(files).catch(() => undefined);
    } catch {
      setSelectionError(EJ_READ_FAILED_MESSAGE);
      toast.error(EJ_READ_FAILED_MESSAGE);
    } finally {
      setPending(false);
    }
  }, [mutateAsync]);

  /** 結果と error を消す（日報の flow の新しい scan・一括のやり直し） */
  const reset = useCallback(() => {
    setResult(null);
    setSelectionError(null);
    resetMutation();
  }, [resetMutation]);

  return {
    result,
    // unwrapResult が失敗を InvokeError にそろえるので、そのまま共通の文にする（UI-ERR-D1）
    error: selectionError ?? (mutation.error ? describeError(mutation.error) : null),
    pending,
    choose,
    reset,
  };
}
