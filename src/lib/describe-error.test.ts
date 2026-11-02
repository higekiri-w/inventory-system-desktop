import { describe, expect, it } from "vitest";

import type { CmdError, CmdErrorKind } from "./bindings";
import { describeError } from "./describe-error";
import { InvokeError } from "./invoke";

function invokeError(cmdError: CmdError): InvokeError {
  return new InvokeError(cmdError, { source: "commands", cmd: "synthetic_command" });
}

describe("describeError (REQ-700 / UI-ERR-D1)", () => {
  it("shows the internal message, independently transcribed error ID, and diagnostic guidance", () => {
    const error = invokeError({
      kind: "internal",
      message: "バックアップ一覧の取得でエラーが発生しました",
      field: null,
      error_id: "E-20260726-153021-a1b2",
      stocktake_recovery: null,
    });

    expect(describeError(error)).toBe(
      "バックアップ一覧の取得でエラーが発生しました（エラーID: E-20260726-153021-a1b2）。詳細は診断ログに記録されています。",
    );
  });

  it("omits only the ID clause for an older internal payload without error_id", () => {
    const error = invokeError({
      kind: "internal",
      message: "バックアップ一覧の取得でエラーが発生しました",
      field: null,
      error_id: null,
      stocktake_recovery: null,
    });

    expect(describeError(error)).toBe(
      "バックアップ一覧の取得でエラーが発生しました。詳細は診断ログに記録されています。",
    );
  });

  it.each(["validation", "export_error"] as const satisfies readonly CmdErrorKind[])(
    "passes through the message for %s",
    (kind) => {
      const error = invokeError({
        kind,
        message: "独立転記した利用者向けメッセージ",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      });

      expect(describeError(error)).toBe("独立転記した利用者向けメッセージ");
    },
  );

  it("W7 / REQ-205: stocktake_guard shows the BIZ message and does not hide the refusal (40)", () => {
    // 未知の code でも message を出し、空や成功の文にしない（回復の分岐は画面が型で行う）
    const recoveryCodes = ["completion_blocked", "unknown_future_code"] as const;
    for (const code of recoveryCodes) {
      const error = invokeError({
        kind: "stocktake_guard",
        message: "棚卸しを完了できません。理由ごとの一覧を確かめてください",
        field: null,
        error_id: null,
        stocktake_recovery: {
          code,
          targets: [],
          settlement_pending: null,
        },
      } as unknown as CmdError);

      expect(describeError(error)).toBe("棚卸しを完了できません。理由ごとの一覧を確かめてください");
    }
  });

  it("passes through the message for a legacy unknown kind payload", () => {
    const error = invokeError({
      kind: "unknown_kind",
      message: "独立転記した利用者向けメッセージ",
      field: null,
      error_id: null,
      stocktake_recovery: null,
    } as unknown as CmdError);

    expect(describeError(error)).toBe("独立転記した利用者向けメッセージ");
  });

  it("uses the caller fallback for a non-command error", () => {
    expect(describeError(new Error("raw browser detail"), "処理を完了できませんでした")).toBe(
      "処理を完了できませんでした",
    );
  });
});
