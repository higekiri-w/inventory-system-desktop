import { describe, expect, it } from "vitest";

import { InvokeError, unwrapResult } from "./invoke";

describe("unwrapResult", () => {
  it("normalizes a raw string rejection to an internal InvokeError (REQ-700)", async () => {
    // Tauri IPC can reject with an untyped string before typedError returns a Result.
    // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors
    const rejection = unwrapResult(Promise.reject("invalid enum payload"), {
      source: "commands",
      cmd: "finite_enum_probe",
    });

    await expect(rejection).rejects.toMatchObject({
      name: "InvokeError",
      cmdError: {
        kind: "internal",
        message: "invalid enum payload",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    } satisfies Partial<InvokeError>);
  });
});
