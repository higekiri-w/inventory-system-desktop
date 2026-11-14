import { describe, expect, it } from "vitest";

import { OPERATION_TYPE_LABELS } from "./operation-type-labels";

describe("operation type labels（UI-11c）", () => {
  it("SC9b: 現行5種別のcategoryとlabelが独立転記した契約と一致する", () => {
    const expected = {
      product_price_revise: { category: "商品管理", label: "一括価格改定" },
      product_bulk_plu_target: { category: "商品管理", label: "PLU 対象一括切替" },
      plu_register_snapshot_import: { category: "PLU書出し", label: "PLU 登録状態の取込み" },
      supplier_rename: { category: "取引先管理", label: "取引先の改名" },
      supplier_merge: { category: "取引先管理", label: "取引先の統合" },
    } as const;

    for (const [operationType, contract] of Object.entries(expected)) {
      expect(OPERATION_TYPE_LABELS[operationType]).toEqual(contract);
    }
  });

  it("U15 / 74 registry: transfer_create は入出庫の「詰め替え・切り分け記録」で、廃棄・破損記録の次に並ぶ", () => {
    expect(OPERATION_TYPE_LABELS.transfer_create).toEqual({
      category: "入出庫",
      label: "詰め替え・切り分け記録",
    });
    const order = Object.keys(OPERATION_TYPE_LABELS);
    expect(order.indexOf("transfer_create")).toBe(order.indexOf("disposal_create") + 1);
  });
});
