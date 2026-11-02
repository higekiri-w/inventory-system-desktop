# タスク仕様（CMD層）

## 計数の置き場と在庫の求め直しの契約（D-118。b1 で実装済み。(a) は command を足さない）

SPEC-STK-CNT-D4〜D6（[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)）。(a)（ADR SPEC-STK-CNT-D12）は command の名前と引数を変えず、CMD-11 の `run_integrity_check`・`fix_integrity` の戻り値の型（[36](../function-design/36-biz-integrity-check.md)）だけが変わった。以下は b1（[Plan Packet](../archive/plans/2026-10-22-stocktake-count-runtime-b1.md)）で入った。CMD-10 は [42](../function-design/42-cmd-sales-stocktake.md) の `save_stock_count` / `preview_stock_count_placement` / `zero_uncounted_items` / `list_recheck_products` を足し、無検査の数の上書きの command の公開登録を外した。CMD-07 は [41](../function-design/41-cmd-pos.md) の知らせと要再確認の一覧を透過し、Z004 では追加確認の入力を使わない。CMD は PC の今日を BIZ へ渡し、業務条件を置かない。共通の回復 payload は [40](../function-design/40-cmd-product.md)。tauri / specta 属性・collect_commands・bindings・全 caller の同期は b1 で同じ PR に入れた。旧 ADR（時点証拠）の token・context の保管・DB 世代・準備照会は置き換えた。

> **親文書**: [ARCHITECTURE.md](../ARCHITECTURE.md)

CMD層は薄いラッパーのため、各コマンドの仕様は「どのBIZ関数を呼ぶか」と「入出力の型」に限定する。

---

### CMD-01: 商品コマンド群

| コマンド名 | 入力 | 呼び出すBIZ | 出力 |
|-----------|------|-----------|------|
| create_product | ProductCreateRequest（name, department_id, selling_price, cost_price_centi, tax_rate, stock_unit, initial_stock, jan_code?, maker_code?, supplier_id?, pos_stock_sync） | BIZ-01 商品新規登録 | ProductCreateResult（product_code, success） |
| update_product | ProductUpdateRequest（product_code, 変更フィールド群） | BIZ-01 商品修正 | Result（success, warnings[]） |
| toggle_discontinue | product_code | BIZ-01 廃番切替 | Result（success, new_status） |
| search_products | SearchQuery（keyword, department_id?, is_discontinued?, sort, page, per_page） | BIZ-01 商品検索 | ProductList（items[], total_count） |
| get_product | product_code | BIZ-01 商品取得 | Product（全フィールド＋部門名＋取引先名） |
| preview_import | FileBytes | BIZ-01 一括インポート前半 | ImportPreview（valid_rows[], error_rows[], duplicate_rows[]） |
| commit_import | ImportCommitRequest（valid_rows[], overwrite_codes[], old_jan_aliases[]〈{product_code, old_jan}、proposed・未実装、D-122〉） | BIZ-01 一括インポート後半 | ImportResult（created, updated, skipped, errors）。D-122: 上書きを選んだ別の行が前の JAN を持つと validation error（field `old_jan_aliases`） |
| revise_product_price | PriceRevisionInput（product_code, new_selling_price, new_cost_price_centi, assign_supplier_id?） | BIZ-01 行単位価格改定 | PriceRevisionResult（product_code, changed, plu_dirty_set, supplier_assigned） |
| create_supplier | name | BIZ-01 取引先（メーカー/ブランド）追加 | Supplier（同名は既存行） |
| list_price_history | product_code, limit | BIZ-01 価格履歴参照 | PriceHistoryEntry[]（DESC、既定10・上限100） |

> **runtime の状態**: 上は D-117 の後の形（採用済み）。今の build は、`search_products`（`src-tauri/src/cmd/product_cmd.rs`）の入力 `ProductSearchQuery`（`src-tauri/src/db/product_repo.rs`）が PLU の絞り込み `plu`（`PluMigrationFilter`）を持ち、絞り込みに一致する全件の PLU 対象を一括で変える `bulk_set_plu_target`（`src-tauri/src/cmd/product_cmd.rs`。入力は `ProductBulkFilter` と `plu_target`、[40](../function-design/40-cmd-product.md) CMD-01-D4。`src-tauri/src/lib.rs` の `collect_commands` に登録）がある。外すのは PLU の書出しの見直しの runtime（[33](../function-design/33-biz-plu-export-service.md) §16 の冒頭の「runtime の状態」の注記）。

### CMD-02: 入庫コマンド群

| コマンド名 | 入力 | 呼び出すBIZ | 出力 |
|-----------|------|-----------|------|
| create_receiving | ReceivingCreateRequest（supplier_id?, receiving_date, note?, items[]） | BIZ-02 入庫記録 | ReceivingCreateResult（record_id, created, idempotent_replay, stock_warnings[], cost_diffs[]） |
| list_receivings | ListQuery（page, per_page, date_from?, date_to?） | BIZ-02 入庫一覧 | ReceivingList（items[], total_count） |

### CMD-03: 返品・交換コマンド群

| コマンド名 | 入力 | 呼び出すBIZ | 出力 |
|-----------|------|-----------|------|
| create_return | ReturnCreateRequest（return_type, return_date, register_processed, receipt_image?, note?, items[]） | BIZ-02 返品記録 | Result（record_id） |
| list_returns | ListQuery | BIZ-02 返品・交換一覧 | ReturnList |

### CMD-04: 手動販売出庫コマンド群

| コマンド名 | 入力 | 呼び出すBIZ | 出力 |
|-----------|------|-----------|------|
| create_manual_sale | ManualSaleCreateRequest（sale_date, reason, note?, items[]） | BIZ-02 手動販売出庫 | Result（sale_id, plu_warnings[]） |

### CMD-05: 廃棄・破損コマンド群

| コマンド名 | 入力 | 呼び出すBIZ | 出力 |
|-----------|------|-----------|------|
| create_disposal | DisposalCreateRequest（disposal_date, items[]） | BIZ-02 廃棄記録 | Result（record_id） |
| list_disposals | ListQuery | BIZ-02 廃棄・破損一覧 | DisposalList |

### CMD-06: 在庫照会コマンド群

| コマンド名 | 入力 | 呼び出すBIZ | 出力 |
|-----------|------|-----------|------|
| get_stock_detail | product_code | BIZ-01 在庫詳細 | StockDetail（stock_quantity, selling_price, cost_price_centi, last_receiving_date, last_sale_date） |
| list_low_stock | LowStockQuery（include_discontinued?） | BIZ-01 在庫少一覧 | LowStockList（items[]） |
| list_movements | MovementQuery（product_code, date_from?, date_to?〈業務日〉, movement_type?, page, per_page） | BIZ-02 在庫変動履歴（movement と計数の合成） | PaginatedResult<StockHistoryItem>（items[]〈movement か計数の行〉, total_count, page, per_page。[44](../function-design/44-cmd-inventory.md)） |

### CMD-07: Z004商品別CSV取込みコマンド群

| コマンド名 | 入力 | 呼び出すBIZ | 出力 |
|-----------|------|-----------|------|
| parse_and_validate_csv | FileBytes, filename | BIZ-03 Stage1+2 | ParseValidateResult（preview_data, preview_token） |
| commit_csv_import | CommitRequest（preview_token, additional_import_confirmed） | BIZ-03 Stage4 | ImportResult（csv_import_id, status, total_items, skipped_count） |
| rollback_csv_import | csv_import_id | BIZ-03 ロールバック | RollbackResult（success, voided_sale_count, stock_corrections） |
| preview_rollback_csv_import | csv_import_id | BIZ-03 取消の前の読取り（取消を TX で当てて戻す。何も書かない） | RollbackPreview（csv_import_id, settlement_no, recheck_targets, affected_stocktakes） |
| list_csv_imports | ListQuery（page, per_page） | BIZ-03経由 | PaginatedResult\<CsvImport\> |

### CMD-12: 日報取込みコマンド群

| コマンド名 | 入力 | 呼び出すBIZ | 出力 |
|-----------|------|-----------|------|
| scan_register_sd | selected_path?（None = 自動で探す） | BIZ-08 §37.9 | RegisterSdScanResponse（scan, scan_token）。読んだ bytes は AppState の scan cache（30 分）に置く（CMD-12-D1、D-111） |
| parse_and_validate_daily_report_from_sd | scan_token, candidate_key | BIZ-08 Stage1+2 | DailyReportPreviewResponse（preview_data, preview_token） |
| parse_and_validate_daily_report | DailyReportSourceFile[]（filename, file_bytes） | BIZ-08 Stage1+2 | DailyReportPreviewResponse（preview_data, preview_token） |
| commit_daily_report_import | CommitDailyReportRequest（preview_token, additional_import_confirmed） | BIZ-08 Stage4 | DailyReportImportResult（daily_report_import_id, status, report_date, warning_count） |
| rollback_daily_report_import | daily_report_import_id | BIZ-08 ロールバック | DailyReportRollbackResult（success, status） |
| list_daily_report_imports | ListQuery（page, per_page, date_from?, date_to?） | BIZ-08経由 | PaginatedResult\<DailyReportImport\> |

**CMD-12の責務境界**:
- ファイルバイト列をBIZ-08へ中継する。Z001/Z002/Z005のsource判定や日報バリデーションはCMDで行わない。SD の探し方・候補の規則・状態はBIZ-08が持ち、CMDは scan の snapshot を保持して渡すだけ（IO-09 を直接呼ばない）。
- preview_token のUUID形式チェックとファイルサイズ上限の早期チェックだけを防御的入力チェックとして許可する。
- BIZ-08の error は `CmdError.kind = "import_error"` または既存 `validation` / `not_found` / `internal` に変換する。
- CMD-07（Z004商品別CSV）とCMD-12（日報）はpreview cacheの保管場所を共有してよいが、cache valueの型は分ける。

### CMD-08: PLU書出しコマンド群

| コマンド名 | 入力 | 呼び出すBIZ | 出力 |
|-----------|------|-----------|------|
| get_plu_slot_summary | なし | BIZ-04 | PluRegisterSnapshotSummary（snapshot_at。占有の照合は CMD-12 scan_register_sd が BIZ-04 を呼ぶ、D-117） |
| prepare_plu_export | ExportMode（'full' / 'diff'） | BIZ-04 | PluExportPreparedResult（tsv_output, count, target_product_codes, prepared_rows[memory_no], excluded〈no_free_slot含む〉） |
| confirm_plu_export_saved | product_codes[], prepared_rows[memory_no] | BIZ-04 | PluExportConfirmResult（updated_count, confirmed_at） |
| list_plu_dirty | なし | BIZ-04経由 | Vec\<ProductResponse\>（plu_target=1 かつ plu_dirty=1 の商品一覧。D-028） |

> **runtime の状態**: 上は D-117 の後の形（採用済み）。今の build は、`get_plu_slot_summary` が件数の要約（`src-tauri/src/biz/plu_export_service.rs` の `PluRegisterSnapshotSummary`。`free_count`・`release_pending_count` ほか）を返し、`prepare_plu_export` の結果（同じ file の `PluExportPreparedResult` と `src-tauri/src/cmd/plu_export_cmd.rs` の `PluExportPrepareResponse`）が `over_limit_warning` を持ち、利用者が選ぶ Z004 で占有を照合する `import_plu_register_snapshot`（`src-tauri/src/cmd/plu_export_cmd.rs`。`src-tauri/src/lib.rs` の `collect_commands` に登録）がある。CMD-12 の `scan_register_sd` はまだ無い。撤去と SD の読取りでの照合への切替えは PLU の書出しの見直しの runtime（[33](../function-design/33-biz-plu-export-service.md) §16 の冒頭の「runtime の状態」の注記）。

### CMD-09: 売上集計コマンド群

| コマンド名 | 入力 | 呼び出すBIZ | 出力 |
|-----------|------|-----------|------|
| get_daily_sales | date（YYYY-MM-DD） | BIZ-05 日次 | DailySalesReport（items[], dept_subtotals[], grand_total） |
| get_monthly_sales | month（YYYY-MM）, mode（'by_product'/'by_department'） | BIZ-05 月次 | MonthlySalesReport（items[]{ranking埋込}, prev_month_comparison） |
| export_sales_csv | ReportParams | BIZ-05 + IO-05 | FileBytes |

### CMD-10: 棚卸しコマンド群

| コマンド名 | 入力 | 呼び出すBIZ | 出力 |
|-----------|------|-----------|------|
| start_stocktake | なし | BIZ-06 開始 | Result（stocktake_id） |
| get_stocktake_items | StocktakeQuery（stocktake_id, department_id?, counted_only?, page） | BIZ-06 棚卸しアイテム一覧 | StocktakeItemList（items[], progress{counted, total}） |
| save_stock_count | SaveStockCountRequest（product_code, count_value, fresh, business_date, stocktake_item_id?） | BIZ-06 計数の保存 | StockCountSaveResult（D-118 の b1。42） |
| preview_stock_count_placement | product_code, business_date | BIZ-06 後入力の確認と計数の欄の文脈 | CountPlacementPreview |
| zero_uncounted_items | stocktake_id, product_codes, business_date | BIZ-06 0 にする | StockCountSaveResult[] |
| list_recheck_products | なし | BIZ-06 要再確認の一覧 | CountRecoveryTarget[] |
| complete_stocktake | stocktake_id | BIZ-06 確定 | StocktakeResult（total_cost, total_items, integrity_result）。拒否は `stocktake_guard`（completion_blocked） |

### CMD-11: 設定・ログコマンド群

| コマンド名 | 入力 | 呼び出すBIZ/MNT/IO | 出力 |
|-----------|------|-----------|------|
| get_settings | なし | BIZ-09 | AppSettings（全設定値） |
| update_setting | key, value | BIZ-09 | Result（success） |
| list_logs | LogQuery（page, per_page, operation_type?, start_date?, end_date?） | BIZ-09（日付validation所有） | LogList（items[], total_count） |
| list_log_operation_types | なし | BIZ-09 | operation_type一覧（distinct） |
| create_backup | なし | MNT-01 | Result（backup_path） |
| list_backups | なし | MNT-01 | BackupList（items[]{filename, created_at, size}） |
| get_effective_backup_dir | なし | MNT-01（backup::resolve_backup_dir） | 実効バックアップ保存先（文字列） |
| restore_backup | backup_path | MNT-01 | Result（success） |
| run_integrity_check | なし | BIZ-07 | IntegrityResult（mismatches[]） |
| fix_integrity | product_codes[] | BIZ-07 補正 | Result（fixed_count） |
| check_auto_backup | なし | MNT-01（backup::check_auto_backup） | Result（bool） |
| prepare_offsite_medium | selected_path | MNT-01（offsite::prepare_offsite_medium、D-114） | OffsiteMediumView |
| check_offsite_backup | なし | MNT-01（offsite::check_offsite_backup、D-114） | OffsiteCheckResult |
| get_offsite_backup_status | なし | MNT-01（offsite::offsite_status、D-114） | OffsiteBackupStatus |
| inspect_backup | backup_path | MNT-01（backup::inspect_backup、D-114） | BackupInspection |
| reset_offsite_state | なし | MNT-01（backup::create_backup の後に offsite::reset_offsite_state、D-114） | OffsiteCheckResult |
| stop_offsite_backup | なし | MNT-01（offsite::stop_offsite_backup、D-114） | null |
| save_receipt_image | SaveImageRequest | BIZ-02（拡張子validation所有、BIZがIO-06 image_managerを呼ぶ） | Result（relative_path） |

CMD-11 の層経路は D-060 で正本化した（`ARCHITECTURE.md`「レイヤー間の呼び出し原則」参照）: 設定・操作ログ系は BIZ-09（`biz::system_service`）経由の標準経路、backup/restore 系は DB 接続所有権の交換を要する保守 orchestration として CMD → MNT-01 の正規経路（復旧規則の正本は function-design 71 §71.7）、領収書画像は base64 decode（CMD の wire 型変換）+ BIZ-02 の拡張子 validation。CMD から IO 層への直接呼び出しは禁止で、`src-tauri/tests/architecture_test.rs` の layer 依存 test が機械検査する（旧 allowlist の settings_cmd 例外 2 entry は削除済み）。検査対象は `use crate::db` / `use crate::io` の直接 import 行であり、re-export 経由の間接依存は対象外（検出強化は backlog）。
biz/mnt からの直接 `pub use` / `pub(crate) use crate::db::*` / `crate::io::*` 再公開は D-083 の allow list（`src-tauri/tests/architecture_test.rs`）で無審査の新規追加を禁止する。alias / type alias を介した再公開は対象外のまま（検出強化は backlog）。

### 更新履歴

| 日付 | PR | 内容 |
|---|---|---|
| 2026-08-16 | PR #79 | SPEC-SDI-D3: 両取込みcommandの確認flagを `additional_import_confirmed` に統一。 |
