// src/lib/invoke.ts
//
// tauri-specta 経路（src/lib/bindings.ts の commands.*）向けエラー helper。
// ADR-004 §2.1。Phase 2 closeout で fallback 経路は撤去済み。

import type { CmdError, CmdErrorKind } from "./bindings";

type CmdErrorKindMap = {
  [Kind in CmdErrorKind as Uppercase<Kind>]: Kind;
};

export const CMD_ERROR_KIND = {
  VALIDATION: "validation",
  NOT_FOUND: "not_found",
  DUPLICATE: "duplicate",
  INTERNAL: "internal",
  IMPORT_ERROR: "import_error",
  EXPORT_ERROR: "export_error",
  IDEMPOTENCY_CONFLICT: "idempotency_conflict",
  STOCKTAKE_IN_PROGRESS: "stocktake_in_progress",
  STOCKTAKE_NOT_IN_PROGRESS: "stocktake_not_in_progress",
  STOCKTAKE_GUARD: "stocktake_guard",
  RESTORE_FAILED_RECOVERED: "restore_failed_recovered",
  RESTORE_FAILED_UNRECOVERABLE: "restore_failed_unrecoverable",
  RESTORE_DURABILITY_UNKNOWN: "restore_durability_unknown",
} as const satisfies CmdErrorKindMap;

export type { CmdErrorKind } from "./bindings";

export type InvokeSource = "commands";

export interface InvokeErrorContext {
  source: InvokeSource;
  cmd: string;
}

/**
 * Error サブクラス。eslint `only-throw-error` を満たしつつ、
 * 呼び出し元は `isInvokeError(e)` で CmdError を取り出せる。
 */
export class InvokeError extends Error {
  readonly cmdError: CmdError;
  readonly context: InvokeErrorContext;

  constructor(cmdError: CmdError, context: InvokeErrorContext) {
    super(`[${context.source}:${context.cmd}] ${cmdError.kind}: ${cmdError.message}`);
    this.name = "InvokeError";
    this.cmdError = cmdError;
    this.context = context;
  }
}

export function isInvokeError(err: unknown): err is InvokeError {
  return err instanceof InvokeError;
}

/**
 * 純粋に CmdError 形状（`{ kind: string, message: string }`）かを判定する。
 *
 * `InvokeError`（Error サブクラスで `cmdError` を内包する）は **ここでは true を返さない**。
 * InvokeError を CmdError に展開したい場合は `isInvokeError` + `.cmdError` もしくは
 * `toCmdError` の正規化経由で扱う。両者を 1 関数で判定すると呼び出し側が `error.kind`
 * 直参照で Error サブクラス側の未定義アクセスを踏みやすいため分離（Codex PR #48 P2-1 対応）。
 */
export function isCmdError(err: unknown): err is CmdError {
  return (
    typeof err === "object" &&
    err !== null &&
    !(err instanceof Error) &&
    "kind" in err &&
    typeof (err as { kind: unknown }).kind === "string" &&
    "message" in err &&
    typeof (err as { message: unknown }).message === "string"
  );
}

/**
 * 任意の値を CmdError に正規化する。
 * - 既に CmdError 形状ならそのまま返す（InvokeError なら .cmdError を取り出す）
 * - Error インスタンスは message を拾って internal として包む
 * - その他は String(err) を message として internal
 */
export function toCmdError(err: unknown): CmdError {
  if (err instanceof InvokeError) return err.cmdError;
  if (isCmdError(err)) return err;
  return {
    kind: CMD_ERROR_KIND.INTERNAL,
    message: err instanceof Error ? err.message : String(err),
    field: null,
    error_id: null,
    stocktake_recovery: null,
  };
}

/**
 * tauri-specta の typedError wrapper が返す Result 型を unwrap する。
 *
 * bindings.ts の commands.xxx(...) は `{ status: "ok", data: T } | { status: "error", error: E }`
 * 形式（tauri-specta v2 の typedError ランタイム）。このヘルパで ok 時は data を返し、
 * error 時は CmdError を包んだ InvokeError として throw する。
 *
 * 併せて wrapper 経由で再 throw された Error（Rust panic 等）は catch で InvokeError に正規化する。
 */
export async function unwrapResult<T>(
  resultPromise: Promise<{ status: "ok"; data: T } | { status: "error"; error: CmdError }>,
  ctx: InvokeErrorContext,
): Promise<T> {
  let result: { status: "ok"; data: T } | { status: "error"; error: CmdError };
  try {
    result = await resultPromise;
  } catch (e) {
    throw new InvokeError(toCmdError(e), ctx);
  }
  if (result.status === "error") {
    throw new InvokeError(toCmdError(result.error), ctx);
  }
  return result.data;
}
