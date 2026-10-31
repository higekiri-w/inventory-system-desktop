export interface OperationTypeLabel {
  category: string;
  label: string;
}

export const OPERATION_TYPE_LABELS: Partial<Record<string, OperationTypeLabel>> = {
  product_create: { category: "商品管理", label: "商品登録" },
  product_update: { category: "商品管理", label: "商品修正" },
  product_discontinue: { category: "商品管理", label: "廃番切替" },
  product_import: { category: "商品管理", label: "商品一括インポート" },
  product_price_revise: { category: "商品管理", label: "一括価格改定" },
  product_bulk_plu_target: { category: "商品管理", label: "PLU 対象一括切替" },
  supplier_rename: { category: "取引先管理", label: "取引先の改名" },
  supplier_merge: { category: "取引先管理", label: "取引先の統合" },
  receiving_create: { category: "入出庫", label: "入庫記録" },
  return_create: { category: "入出庫", label: "返品・交換記録" },
  manual_sale_create: { category: "入出庫", label: "手動販売出庫記録" },
  disposal_create: { category: "入出庫", label: "廃棄・破損記録" },
  transfer_create: { category: "入出庫", label: "詰め替え・切り分け記録" },
  csv_import: { category: "売上データ取込み", label: "売上データ取込み" },
  csv_import_failed: { category: "売上データ取込み", label: "売上データ取込み失敗" },
  csv_import_parse_failed: { category: "売上データ取込み", label: "売上データ解析失敗" },
  csv_rollback: { category: "売上データ取込み", label: "売上データ取込み取消" },
  daily_report_import: { category: "売上データ取込み", label: "日報取込み" },
  daily_report_import_failed: { category: "売上データ取込み", label: "日報取込み失敗" },
  daily_report_parse_failed: { category: "売上データ取込み", label: "日報解析失敗" },
  daily_report_rollback: { category: "売上データ取込み", label: "日報取込み取消" },
  stocktake_start: { category: "棚卸し", label: "棚卸し開始" },
  stocktake_complete: { category: "棚卸し", label: "棚卸し確定" },
  plu_export: { category: "PLU書出し", label: "PLU書出し" },
  plu_register_snapshot_import: { category: "PLU書出し", label: "PLU 登録状態の取込み" },
  integrity_check: { category: "整合性検証", label: "整合性チェック実行" },
  integrity_fix: { category: "整合性検証", label: "整合性補正" },
  backup_create: { category: "システム管理", label: "バックアップ作成" },
  backup_restore: { category: "システム管理", label: "バックアップ復元" },
  log_cleanup: { category: "システム管理", label: "操作ログ自動削除" },
};

/** UI-11c-D4: filter表示順はbackendのdistinct順ではなくregistry表の宣言順。 */
export const OPERATION_TYPE_ORDER = Object.keys(OPERATION_TYPE_LABELS);

export function operationTypeLabel(value: string) {
  return OPERATION_TYPE_LABELS[value]?.label ?? `その他（${value}）`;
}
