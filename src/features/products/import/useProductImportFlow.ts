import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useCallback, useMemo, useReducer } from "react";
import { toast } from "sonner";
import type { PickedFile } from "@/components/FilePicker";
import {
  commands,
  CSV_IMPORT_FILE_SIZE_LIMIT,
  type ImportPreview,
  type ImportRow,
} from "@/lib/bindings";
import { invalidateByContract, invalidationContract } from "@/lib/invalidation-contract";
import { CMD_ERROR_KIND, InvokeError, isInvokeError, unwrapResult } from "@/lib/invoke";
import { PRODUCT_IMPORT_INITIAL_STATE, productImportReducer } from "./reducer";
import type { ProductImportRecoverTo, ProductImportState } from "./types";

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

function decideRecoverTo(error: InvokeError): ProductImportRecoverTo {
  if (error.cmdError.kind === CMD_ERROR_KIND.IMPORT_ERROR) return "idle";
  return "preview";
}

export function buildProductImportTargetRows(
  preview: ImportPreview,
  overwriteCodes: readonly string[],
): ImportRow[] {
  const overwriteSet = new Set(overwriteCodes);
  const overwriteRows = preview.duplicate_rows
    .filter((row) => overwriteSet.has(row.import_row.product_code))
    .map((row) => row.import_row);
  return [...preview.valid_rows, ...overwriteRows];
}

export interface UseProductImportFlowResult {
  state: ProductImportState;
  targetRows: ImportRow[];
  selectFile: (file: PickedFile) => void;
  toggleOverwrite: (productCode: string, checked: boolean) => void;
  confirmImport: () => void;
  dismissError: () => void;
  reset: () => void;
  isPreviewing: boolean;
  isCommitting: boolean;
}

export function useProductImportFlow(): UseProductImportFlowResult {
  const [state, dispatch] = useReducer(productImportReducer, PRODUCT_IMPORT_INITIAL_STATE);
  const queryClient = useQueryClient();

  const previewMutation = useMutation({
    mutationFn: (args: { fileBytes: number[] }) =>
      unwrapResult(commands.previewImport(args.fileBytes), {
        source: "commands",
        cmd: "preview_import",
      }),
    retry: 0,
    onSuccess: (preview) => {
      dispatch({ type: "preview_succeeded", preview });
    },
    onError: (error: unknown) => {
      dispatch({ type: "preview_failed", error: ensureInvokeError(error, "preview_import") });
    },
  });

  const commitMutation = useMutation({
    mutationFn: (args: { targetRows: ImportRow[]; overwriteCodes: string[] }) =>
      unwrapResult(commands.commitImport(args.targetRows, args.overwriteCodes), {
        source: "commands",
        cmd: "commit_import",
      }),
    retry: 0,
    onSuccess: async (result) => {
      dispatch({ type: "commit_succeeded", result });
      toast.success("商品マスタをインポートしました", { id: "product-import-success" });
      await invalidateByContract(queryClient, invalidationContract.productImport());
    },
    onError: (error: unknown) => {
      const invokeError = ensureInvokeError(error, "commit_import");
      dispatch({
        type: "commit_failed",
        error: invokeError,
        recoverTo: decideRecoverTo(invokeError),
      });
    },
  });

  const targetRows = useMemo(() => {
    if (state.status !== "preview") return [];
    return buildProductImportTargetRows(state.preview, state.overwriteCodes);
  }, [state]);

  const selectFile = useCallback(
    (file: PickedFile) => {
      if (file.size > CSV_IMPORT_FILE_SIZE_LIMIT) {
        toast.error("ファイルサイズが上限(20MB)を超えています");
        return;
      }
      const fileBytes = Array.from(file.bytes);
      dispatch({ type: "select_file", filename: file.filename });
      previewMutation.mutate({ fileBytes });
    },
    [previewMutation],
  );

  const toggleOverwrite = useCallback((productCode: string, checked: boolean) => {
    dispatch({ type: "toggle_overwrite", productCode, checked });
  }, []);

  const confirmImport = useCallback(() => {
    if (state.status !== "preview") return;
    const rows = buildProductImportTargetRows(state.preview, state.overwriteCodes);
    if (rows.length === 0) return;
    dispatch({ type: "commit_requested", targetRows: rows });
    commitMutation.mutate({ targetRows: rows, overwriteCodes: state.overwriteCodes });
  }, [state, commitMutation]);

  const dismissError = useCallback(() => {
    dispatch({ type: "dismiss_error" });
  }, []);

  const reset = useCallback(() => {
    dispatch({ type: "reset" });
  }, []);

  return {
    state,
    targetRows,
    selectFile,
    toggleOverwrite,
    confirmImport,
    dismissError,
    reset,
    isPreviewing: previewMutation.isPending,
    isCommitting: commitMutation.isPending,
  };
}
