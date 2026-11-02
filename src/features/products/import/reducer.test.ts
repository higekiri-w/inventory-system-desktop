import { describe, expect, it } from "vitest";
import { InvokeError, CMD_ERROR_KIND } from "@/lib/invoke";
import { productImportReducer, PRODUCT_IMPORT_INITIAL_STATE } from "./reducer";
import type { ProductImportState } from "./types";
import type { ImportPreview, ImportRow } from "@/lib/bindings";

const importRow: ImportRow = {
  line_no: 2,
  product_code: "P-001",
  name: "はさみ",
  department_id: 1,
  selling_price: 500,
  cost_price_centi: 300,
  tax_rate: "10",
  stock_unit: null,
  initial_stock: null,
  jan_code: null,
  maker_code: null,
  supplier_id: null,
  pos_stock_sync: null,
};

const preview: ImportPreview = {
  valid_rows: [importRow],
  duplicate_rows: [
    {
      line_no: 3,
      import_row: { ...importRow, product_code: "P-002", name: "布" },
      existing_product_code: "P-002",
    },
  ],
  error_rows: [],
};

function invokeError(message = "失敗しました") {
  return new InvokeError(
    {
      kind: CMD_ERROR_KIND.INTERNAL,
      message,
      field: null,
      error_id: null,
      stocktake_recovery: null,
    },
    { source: "commands", cmd: "commit_import" },
  );
}

describe("productImportReducer (UI-01c / REQ-104)", () => {
  it("preview 成功時に overwrite selection を空で開始する", () => {
    const previewing = productImportReducer(PRODUCT_IMPORT_INITIAL_STATE, {
      type: "select_file",
      filename: "products.csv",
    });

    const next = productImportReducer(previewing, { type: "preview_succeeded", preview });

    expect(next).toMatchObject({
      status: "preview",
      filename: "products.csv",
      overwriteCodes: [],
    });
  });

  it("UI-01c-D7: 重複行の上書き選択を product_code 単位で保持する", () => {
    const state: ProductImportState = {
      status: "preview",
      filename: "products.csv",
      preview,
      overwriteCodes: [],
    };

    const checked = productImportReducer(state, {
      type: "toggle_overwrite",
      productCode: "P-002",
      checked: true,
    });
    const unchecked = productImportReducer(checked, {
      type: "toggle_overwrite",
      productCode: "P-002",
      checked: false,
    });

    expect(checked.status === "preview" ? checked.overwriteCodes : []).toEqual(["P-002"]);
    expect(unchecked.status === "preview" ? unchecked.overwriteCodes : []).toEqual([]);
  });

  it("UI-01c-D5: commit 失敗後は preview に戻れる previousState を保持する", () => {
    const committing: ProductImportState = {
      status: "committing",
      filename: "products.csv",
      preview,
      overwriteCodes: ["P-002"],
      targetRows: preview.valid_rows,
    };

    const errorState = productImportReducer(committing, {
      type: "commit_failed",
      error: invokeError(),
      recoverTo: "preview",
    });
    const recovered = productImportReducer(errorState, { type: "dismiss_error" });

    expect(errorState.status).toBe("error");
    expect(recovered).toMatchObject({
      status: "preview",
      filename: "products.csv",
      overwriteCodes: ["P-002"],
    });
  });
});
