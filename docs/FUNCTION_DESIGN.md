# 在庫管理システム 関数設計書

## 計数の置き場と在庫の求め直しの契約（D-118。(a)・b1・b2 は実装済み）

[棚卸しの数の置き場と在庫の求め直しの ADR](adr/2026-10-16-stocktake-count-placement.md)（SPEC-STK-CNT-D1〜D12）の詳細を、各文書の同名の節へ展開した。(a) で入ったのは、写しを記録から求め直す関数と並びの鍵付きの movement（21・31〈`biz/stock_ledger.rs`〉）、整合性の突合と補正（36・75）、計数の表と写しの列の migration v9（22 §19、`db/schema_v9.rs`）。b1（[Plan Packet](archive/plans/2026-10-22-stocktake-count-runtime-b1.md)）で、計数の保存と完了（35・42・73）、Z004 の識別・確定・取消（24・32・41・55）、共有 JAN と在庫連動を始める検査（30・51・60）、整合性の共有 JAN の一覧（36・75）、migration v11（22 §20）が入った。b2（[Plan Packet](plans/2026-10-23-stocktake-count-runtime-b2.md)）で、商品の結果の写しの 3 列（20・40）、商品の画面の範囲の表示と要再確認の案内（50・51・58・66）、在庫変動の履歴の計数の行（44・65・66）が入った。旧 ADR（時点証拠、superseded）の節は置き換えた。

| 境界 | 詳細の source |
|---|---|
| 保存・写し・parser | [20](function-design/20-io-product-repo.md)、[21](function-design/21-io-inventory-repo.md)、[23](function-design/23-io-z004-parser.md)、[24](function-design/24-io-csv-import-repo.md) |
| 商品の設定・入出庫・求め直し・取込み・計数と完了・整合性 | [30](function-design/30-biz-product-service.md)、[31](function-design/31-biz-inventory-service.md)、[32](function-design/32-biz-csv-import-service.md)、[35](function-design/35-biz-stocktake-service.md)、[36](function-design/36-biz-integrity-check.md) |
| 回復型・POS・計数の wire・DB の交換 | [40](function-design/40-cmd-product.md)、[41](function-design/41-cmd-pos.md)、[42](function-design/42-cmd-sales-stocktake.md)、[43](function-design/43-cmd-settings-log.md) |
| 画面 | [51](function-design/51-ui-product-form.md)、[55](function-design/55-ui-csv-import.md)、[58](function-design/58-ui-stock-inquiry.md)、[60](function-design/60-ui-product-import.md)、[65](function-design/65-inventory-record-traceability.md)、[66](function-design/66-ui-stock-movements.md)、[73](function-design/73-ui-stocktake.md) |

永続化は [DB 設計](DB_DESIGN.md) へ集約する。

> **入力ドキュメント**: ARCHITECTURE.md（タスク仕様）、DB_DESIGN.md（テーブル定義書）
> **対象範囲**: 実装第1〜第4段階 + 第7段階 UI 基盤 (UI-12 共通レイアウト) + 第8段階 Phase 2 8-1 (UI-00 ホーム画面) + 8-2 (UI-07 売上データ取込み画面) + 8-6 (UI-shortcuts ショートカット一覧ダイアログ) + Phase 4 UI-11b バックアップ・復元 / UI-11a 閾値設定 / UI-10 棚卸し / UI-11c 操作ログ画面 Design Phase

### 現時点の対象モジュール
- IO-01: SQLiteデータアクセス層（db::init, product_repo, inventory_repo, system_repo, stocktake_repo, sales_repo）
- IO-01 追加: CSV取込みリポジトリ（csv_import_repo）
- IO-01 追加: 売上集計クエリ（sales_repo BIZ-05用）、棚卸しリポジトリ拡張（stocktake_repo BIZ-06用）、movements集計（inventory_repo BIZ-07用）
- IO-02: Z004パーサー（z004_parser）
- IO-03: 商品マスタCSVインポーター（product_csv_importer）
- IO-04: PLUフォーマッター（plu_formatter。E-4確定仕様）
- MNT-03: スキーママイグレーション（migration v1〜v4）
- BIZ-01: 商品管理ロジック（product_service）+ 一括インポート（preview_import, commit_import）
- BIZ-02: 在庫変動ロジック（inventory_service）
- BIZ-03: Z004商品別CSV取込みパイプライン（csv_import_service）
- BIZ-04: PLU書出しロジック（plu_export_service）
- BIZ-05: 売上集計ロジック（sales_service）
- BIZ-06: 棚卸しロジック（stocktake_service）
- BIZ-07: 整合性チェックロジック（integrity_service）
- BIZ-08: 日報取込みロジック（daily_report_import_service）
- CMD-01: 商品コマンド群（product_cmd）+ 一括インポート
- CMD-07: Z004商品別CSV取込みコマンド群（csv_import_cmd）
- CMD-08: PLU書出しコマンド群（plu_export_cmd）
- CMD-09: 売上集計コマンド群（sales_cmd）
- CMD-10: 棚卸しコマンド群（stocktake_cmd）
- CMD-11 部分: 整合性チェックコマンド（integrity_cmd）
- CMD-02〜05: 入出庫コマンド群（receiving_cmd, return_cmd, manual_sale_cmd, disposal_cmd）
- CMD-06: 在庫照会コマンド群（inventory_cmd）
- CMD-11 残り: 設定・ログ・バックアップ・画像コマンド群（settings_cmd）
- CMD-12: 日報取込みコマンド群（daily_report_import_cmd）
- MNT-01: バックアップ・リストア（backup）
- MNT-02: 操作ログ管理（log_manager）
- MNT-04: アプリケーション診断ログ（diagnostic_log）
- IO-05: レポートCSVエクスポーター（report_csv_exporter）
- IO-06: 画像ファイル管理（image_manager）
- IO-07: POS日報bundleパーサー（daily_report_parser）
- IO-08: EJパーサー（ej_parser）
- IO-09: レジの SD の列挙と読取り（register_sd）
- IO-10: SD から読んだ原本の写しの保存（pos_source_copy）
- UI-01a: 商品検索・一覧（ProductListPage / patterns/SearchBar（旧 ProductSearchBar、PR-B で統合）/ ProductTable / patterns/Pagination）— Design Phase 更新済み。REQ-103、URL state、既存 `commands.searchProducts` 契約、`list_departments` CMD 設計、pagination、廃番表示、HID scanner 前提は [function-design/50-ui-product-list.md](function-design/50-ui-product-list.md) を参照
- UI-01b: 商品登録・編集（ProductFormPage）— routes/ 系に更新済み。商品登録・修正 form 設計、generated command、supplier 候補、廃番確認、保存 toast は [function-design/51-ui-product-form.md](function-design/51-ui-product-form.md) を参照
- UI-01c: 商品一括インポート（ProductImportPage）— Design Phase 更新済み。REQ-104、`/products/import` route、generated `previewImport` / `commitImport` 契約、共通 FilePicker（D-054、§6.5.4）、preview / duplicate / commit flow、Windows native L3 は [function-design/60-ui-product-import.md](function-design/60-ui-product-import.md) を参照
- UI-14: 一括価格改定（PriceRevisionPage）— Design Phase 更新済み。REQ-105/106、取引先・部門・keyword 絞り込み、行単位確定、新原価案、漸進 supplier 紐付け、中断・再開は [function-design/77-ui-bulk-price-revision.md](function-design/77-ui-bulk-price-revision.md) を参照
- UI-02: 入庫記録（ReceivingPage）— Design Phase 更新済み。REQ-201、`/inventory/receiving` route、generated `createReceiving` / `listReceivings` 契約、商品追加/スキャナ相当 Enter 入力、冪等キー、recent list、Windows native L3 は [function-design/61-ui-receiving.md](function-design/61-ui-receiving.md) を参照
- UI-03: 返品・交換（ReturnExchangePage）— Design Phase 更新済み。REQ-202、`/inventory/return` route、generated `createReturn` / `listReturns` / `saveReceiptImage` 契約、レジでの戻し方の問いと明細ごとの在庫の反映（D-148）、画像添付、商品追加/スキャナ相当 Enter 入力、冪等キー、recent list、Windows native L3 は [function-design/63-ui-return-exchange.md](function-design/63-ui-return-exchange.md) を参照
- UI-04: 手動販売出庫（ManualSalePage）— Design Phase 更新済み。REQ-203、`/inventory/manual-sale` route、generated `createManualSale` 契約、商品追加/スキャナ相当 Enter 入力、PLU登録済み確認、冪等キー、日次売上「手動」Badge L3 は [function-design/62-ui-manual-sale.md](function-design/62-ui-manual-sale.md) を参照
- UI-12: 共通レイアウト（RootLayout / Sidebar / navigation 定数）+ ウィンドウタイトル動的更新機構
- UI-00: ホーム画面（HomePage / useHomeSummary / useYesterdayDate / count-stock-status）+ TanStack Query 4 useQuery 束ね（業務ロジックあり版テンプレ初適用）
- UI-shortcuts: ショートカット一覧ダイアログ（ShortcutsDialog / useShortcutsDialog / SHORTCUTS 定数）+ global Ctrl+/ keydown listener + IME / input 除外（業務ロジックあり版テンプレ、CMD 呼び出し 0 件 + state 駆動分岐パターン初適用）
- UI-07: 売上データ取込み画面（SalesImportPage / DailyReportImport flow / existing Z004 CsvImport flow）— current operation は Z001/Z002/Z005 日報取込み、既存Z004商品別CSV取込みはPLU後トラックとして分離
- UI-08: PLU書出し画面（PluExportPage）— D-072 / D-117 / REQ-907 Design Phase 更新済み。毎日の SD の読取りでの Z004 占有の照合、永続 memory No.、`prepare_plu_export` / `confirm_plu_export_saved` 二段階契約、Diff / Full の CV17 投入、Windows native L3 は [function-design/67-ui-plu-export.md](function-design/67-ui-plu-export.md) を参照
- UI-11b: バックアップ・復元画面（BackupRestorePage）— Design Phase 追加済み（実装済み）。QR-05 / REQ-901、PR #141 generated `commands.*`、復元前強制バックアップ、break-glass 例外、復元後 query cache clear、Windows native L3 は [function-design/68-ui-backup-restore.md](function-design/68-ui-backup-restore.md) を参照
- UI-11a: 閾値設定画面（ThresholdSettingsPage、operator 名称「在庫少の基準」）— Design Phase 追加済み（実装済み）。QR系 / D-4、`/settings/thresholds` route、既存 `getSettings` / `updateSetting` 契約（新規 CMD なし）、在庫少基準 2 key 所有、整数 1〜99999 検証、部分失敗表示、Windows native L3 軽量 2 項目は [function-design/69-ui-threshold-settings.md](function-design/69-ui-threshold-settings.md) を参照
- UI-10: 棚卸し画面（StocktakePage）— Design Phase 追加済み（実装済み）。REQ-205、`/stocktake` route、既存 CMD-10 4 コマンド（specta 化は実装 PR）+ 新規 `find_stocktake_item` / `get_last_completed_stocktake` 設計、検索/スキャン 1 発解決 + counted 済み上書き再入力常時許可、確定は常時確認ダイアログ（force_fill 文言分岐）+ total_cost 主役の結果画面、前回完了棚卸し比較、10-4a IPC channel 不採用確定、Windows native L3 5 項目は [function-design/73-ui-stocktake.md](function-design/73-ui-stocktake.md) を参照
- UI-11c: 操作ログ画面（OperationLogsPage）— Design Phase 追加済み（実装済み）。REQ-902 / TRACE-D3、`/settings/logs` route、既存 `listLogs` への `start_date`/`end_date` 拡張（JST 暦日 inclusive/exclusive、row/count predicate 同一性）、新規 `list_log_operation_types` + IO `find_distinct_operation_types`（保持中ログ全体の distinct 種別、現在ページ由来を禁止）、canonical operation_type 日本語ラベル registry、detail_json 既知field要約+折りたたみraw JSON+安全上限、関連業務記録リンクの明示 contract（許可リストと厳密な `record_id` 条件、入庫・返品交換・手動販売・廃棄の作成 producer は対応済み、csv_import / stocktake は別 follow-up）、範囲外page回復、empty 2系統、retry、REQ-902/905 traceability 是正、Windows native L3 8 項目は [function-design/74-ui-operation-logs.md](function-design/74-ui-operation-logs.md) を参照
- UI-13: 在庫整合性検証画面（IntegrityCheckPage）— Design Phase 追加済み（実装済み）。REQ-904、`/settings/integrity` route、idle/running/completed lifecycle、generated `runIntegrityCheck` / `fixIntegrity`、operation log由来の直近確認日時、行単位選択 + 確認dialog、100件client-side paging、fix結果・skipped警告・retry選択保持、非色状態表示、Windows native L3 3項目は [function-design/75-ui-integrity-check.md](function-design/75-ui-integrity-check.md) を参照
- UI request primitives: 入庫・返品交換・手動販売・廃棄request builderのdomain非依存primitive owner、既存wrapper互換、prefix / integer境界、timezone非依存test contractは [function-design/76-ui-request-primitives.md](function-design/76-ui-request-primitives.md) を参照
- UI-09a: 日次売上レポート画面（DailySalesPage / useDailySalesReport / useExportDailySalesCsv / calculate-unit-price / sort-items / group-items / filter-items / compute-summary / date-nav）+ TanStack Router validateSearch（zod 4 直接渡し）+ 2 useQuery 部分障害許容 + 1 useMutation Blob ダウンロード + 派生 5 純関数 + factory（業務ロジックあり版テンプレ、URL state + 2 useQuery + 派生 5 純関数 + 単価派生 + 部門小計テーブル + 主動線 CTA 配線パターン初適用）
- UI-09b: 月次売上レポート画面（MonthlySalesPage / useMonthlySalesReport / compute-summary / compute-period-label / compute-comparison / compute-composition / pick-top-ranking / sort-items / format-month-label / month-nav）+ TanStack Router validateSearch（zod 4 mode/sortBy/sortDir）+ **1 useQuery + prev_month_comparison field 派生**（UI-09a 2 useQuery 機械的横展開でなく BIZ 設計前提に従う、Q-5）+ 共通 `useExportFile({ reportType })` 経由 CSV + 派生 6 純関数 + factory 2 種類 + TabsHeader 共通化 (`src/components/sales/`、router-driven) + Progress wrapper 配置（業務ロジックあり版テンプレ、1 useQuery + 失敗 4 状態 + DTO 不在情報 UI 派生回避パターン初適用）
- 8-7 useExportFile: `src/lib/hooks/useExportFile.ts` 共通化（UI-09a useExportDailySalesCsv を wrapper 化、SalesReportType bindings import = drift 耐性、Sonner id `export-${reportType}-success/error`）
- UI-06a: 在庫照会画面（StockInquiryPage / useStockInquiry / StockDetailContent / derive-stock-state / format-stock-display / format-last-date / filter-low-stock-list）+ TanStack Router validateSearch（zod 4 q/dept/status/page/selected）+ **3 useQuery 部分障害許容**（search_products | list_low_stock + get_stock_detail + listDepartments 独立、UI-09a 横展開）+ StockInquiryListResult 正規化型（PaginatedResult vs 配列の形状不一致吸収）+ 色分け契約 H + 検索駆動表示契約 I（D-138 で撤去し、UI-01a と 1 つの画面「商品と在庫」にする。proposed・未実装） + toggle/toggle-group 新規 add（collapsible は F1 で未使用化、primitive 残置）+ CSV 取込み invalidation（業務ロジックあり版テンプレ、3 useQuery + 1 件自動展開 + selected 不在 clear + 選択行直下インライン展開 + HID スキャナ前提検索パターン初適用、初 Windows L3 デモ起因 F1/F2 修正済 Codex CLI Round 3-4）
- UI-06c: 商品別在庫変動履歴（StockMovementsPage / useStockMovements / StockHistoryTable / movement-formatters。D-118 の b2 で movement と計数の行を並びの鍵の順に混ぜ、業務日で絞り、見出しに要再確認の範囲と案内）+ TanStack Router validateSearch（zod 4 dateFrom/dateTo/type/page）+ **2 useQuery 部分障害許容**（get_stock_detail + list_movements）+ PR #112 `MovementRecord.source` contract による元業務記録リンク + 日本語 movement 種別 / 増減ラベル + Windows native L3

UI-06b は独立画面ではなく UI-06a への deep-link のため、専用の関数設計書は作らない（D-047、詳細は [function-design/58-ui-stock-inquiry.md](function-design/58-ui-stock-inquiry.md) と [function-design/52-ui-shared-layout.md](function-design/52-ui-shared-layout.md) UI-12-D1）。

---

## 1. 本書の位置付け

```
要求仕様（USDM、約130本）
  ↓
タスク設計（ARCHITECTURE.md）
  ↓
★ 関数設計（本書）← FUNCTION_DESIGN.md
  ↓
実装（コード）
```

タスク仕様が「処理の大きな流れとデータの受け渡し」を決めたのに対し、関数設計は「その中の1ステップを確実に完結させるための具体的なロジック」を決める。実装者がソースコードを書く際に迷わないレベルまで具体化する。

### 記述方針
- 各関数について「関数要求」「引数・戻り値」「処理ステップ」「エラーハンドリング」を記述
- Rustの型名・モジュール構造を意識するが、Rustの文法そのものは書かない（擬似コードレベル）
- 第1〜第2段階に必要な関数から着手し、後続段階の関数も各サブ文書に置いた

---

## 目次

### 共通ルール
- [不変条件・共通型定義](function-design/10-common-rules.md) — INV-1〜8、DbError、PaginatedResult、TX方針

### IO層（データアクセス・ファイルI/O）
- [IO-01: 商品・部門・取引先リポジトリ](function-design/20-io-product-repo.md) — product_repo, system_repo, stocktake_repo + 売上集計クエリ(BIZ-05用) + 棚卸しリポジトリ拡張(BIZ-06用)
- [IO-01 追加: 在庫変動リポジトリ](function-design/21-io-inventory-repo.md) — 入庫/返品/手動販売/廃棄repos, sales_repo + movements集計(BIZ-07用)
- [IO-01 追加: POS取込みリポジトリ](function-design/24-io-csv-import-repo.md) — csv_imports, csv_import_errors, daily_report_imports, daily_report_*_lines（BIZ-03/BIZ-08 用）
- [IO-02: Z004パーサー](function-design/23-io-z004-parser.md) — parse_z004, normalize_jan（純関数、DB非依存）
- [IO-03: 商品マスタCSVインポーター](function-design/26-io-product-csv-importer.md) — parse_product_csv（純関数、DB非依存）
- [IO-04: PLUフォーマッター](function-design/25-io-plu-formatter.md) — generate_plu_tsv（純関数、DB非依存。E-4確定仕様）
- [IO-05: レポートCSVエクスポーター](function-design/27-io-report-csv-exporter.md) — export_csv（UTF-8 BOM付き、純関数）
- [IO-06: 画像ファイル管理](function-design/28-io-image-manager.md) — save_receipt_image（レシート画像保存、相対パス管理）
- [IO-07: POS日報bundleパーサー](function-design/29-io-daily-report-parser.md) — parse_daily_report_bundle（Z001/Z002/Z005、CP932/NEL、純関数）。同じ文書の §29.7 に IO-09: レジの SD の列挙と読取り（find_register_sd_roots / resolve_register_sd_root / list_register_sd_entries / read_register_sd_file、D-111）と §29.8 に IO-10: SD から読んだ原本の写しの保存（save_pos_source_copy）
- [IO-08: EJパーサー](function-design/29-io-ej-parser.md) — parse_ej（EJ 1 file を取引単位の記録へ構造復元、24バイト固定幅・CP932・CRLF、純関数）

### MNT層（保守）
- [MNT-01: バックアップ・リストア](function-design/71-mnt-backup.md) — create_backup（VACUUM INTO）, restore_backup, check_auto_backup, list_backups, inspect_backup（D-114、実装済み）, PC の外の控え（prepare_offsite_medium / check_offsite_backup / offsite_status / reset_offsite_state / stop_offsite_backup。D-114、runtime は後続の (b)）
- [MNT-02: 操作ログ管理](function-design/72-mnt-log-manager.md) — cleanup_old_logs（起動時自動削除）
- [MNT-03: スキーママイグレーション](function-design/22-mnt-migration.md) — migration v1（初期スキーマ）〜 v4（日報取込みテーブル）
- [MNT-04: アプリケーション診断ログ](function-design/70-mnt-diagnostic-log.md) — init_diagnostics, cleanup_old_log_files, 既存コードへのtracing埋め込み方針

### BIZ層（ビジネスロジック）
- [BIZ-01: 商品管理ロジック](function-design/30-biz-product-service.md) — create/update/toggle/search/generate_code + preview_import/commit_import
- [BIZ-02: 在庫変動ロジック](function-design/31-biz-inventory-service.md) — apply_stock_change, create_receiving/return/manual_sale/disposal
- [BIZ-03: Z004商品別CSV取込みパイプライン](function-design/32-biz-csv-import-service.md) — parse_and_validate, commit_csv_import, rollback_csv_import, preview_rollback_csv_import
- [BIZ-04: PLU書出しロジック](function-design/33-biz-plu-export-service.md) — prepare_plu_export, confirm_plu_export_saved, list_plu_dirty
- [BIZ-05: 売上集計ロジック](function-design/34-biz-sales-service.md) — get_daily_sales, get_monthly_sales
- [BIZ-06: 棚卸しロジック](function-design/35-biz-stocktake-service.md) — start_stocktake, update_count, complete_stocktake
- [BIZ-07: 整合性チェックロジック](function-design/36-biz-integrity-check.md) — run_integrity_check, fix_integrity
- [BIZ-08: 日報取込みロジック](function-design/37-biz-daily-report-import-service.md) — scan_register_sd_daily_reports, parse_and_validate_daily_report, commit_daily_report_import, rollback_daily_report_import

### CMD層（Tauriコマンド）
- [CMD-01: 商品コマンド群](function-design/40-cmd-product.md) + 一括インポート
- [CMD-07/08: POS連携コマンド群](function-design/41-cmd-pos.md) — Z004商品別CSV取込み(parse/commit/rollback/list), PLU書出し(export/list_dirty)
- [CMD-09/10/11部分: 売上集計・棚卸し・整合性コマンド群](function-design/42-cmd-sales-stocktake.md) — 売上(daily/monthly), 棚卸し(start/items/count/complete), 整合性(check/fix)
- [CMD-02〜05: 入出庫コマンド群 / CMD-06: 在庫照会コマンド群](function-design/44-cmd-inventory.md) — 入庫/返品/手動販売/廃棄(create/list), 在庫照会(detail/low_stock/movements)
- [CMD-11残り: 設定・ログ・バックアップ・画像コマンド群](function-design/43-cmd-settings-log.md) — settings CRUD, list_logs（UI-11c-D2/D3 期間拡張）, list_log_operation_types（新規、UI-11c-D4）, backup/restore, save_receipt_image
- [CMD-12: 日報取込みコマンド群](function-design/45-cmd-daily-report-import.md) — Z001/Z002/Z005 daily report bundle SD scan/parse/commit/rollback/list

### UI層（React）
- [UI-12: 共通レイアウト](function-design/52-ui-shared-layout.md) — RootLayout, Sidebar, navigation 定数, ウィンドウタイトル動的更新機構（業務ロジックなし版テンプレ初導入）
- [UI-patterns: 共通 UI パターン部品](function-design/59-ui-shared-patterns.md) — PageHeader, SummaryCard, FormSection, EmptyState, SearchBar, DepartmentFilter（業務ロジックなし版テンプレ、`src/components/patterns/` の props 契約 + 採用箇所対応表。DOM 規約の正典は design-system/02-component-catalog.md）
- [UI request primitives: request builder共通基盤](function-design/76-ui-request-primitives.md) — prefixed idempotency key、local calendar date、strict safe integer parserのsingle ownerと4 feature wrapper互換
- [UI-00: ホーム画面](function-design/53-ui-home.md) — HomePage, useHomeSummary, useYesterdayDate, count-stock-status（業務ロジックあり版テンプレ初適用、4 useQuery 部分障害許容）
- [UI-shortcuts: ショートカット一覧ダイアログ](function-design/54-ui-shortcuts.md) — ShortcutsDialog, useShortcutsDialog, SHORTCUTS 定数（業務ロジックあり版テンプレ、CMD 呼び出し 0 件 + state 駆動分岐パターン初適用）
- [UI-07: 売上データ取込み画面](function-design/55-ui-csv-import.md) — SalesImportPage, DailyReportImport flow, existing Z004 CsvImport flow（業務ロジックあり版テンプレ、current operation 日報主動線 + PLU後商品別トラック）
- [UI-09a: 日次売上レポート画面](function-design/56-ui-daily-sales.md) — DailySalesPage, useDailySalesReport, useExportDailySalesCsv, calculate-unit-price（業務ロジックあり版テンプレ、URL state + 2 useQuery + 派生 5 純関数 + 単価派生 + 部門小計テーブル + 主動線 CTA 配線パターン初適用）
- [UI-09b: 月次売上レポート画面](function-design/57-ui-monthly-sales.md) — MonthlySalesPage, useMonthlySalesReport, compute-summary/period-label/comparison/composition, pick-top-ranking, sort-items, format-month-label（業務ロジックあり版テンプレ、1 useQuery + prev_month_comparison 派生 + 失敗 4 状態 + TabsHeader 共通化 + Progress wrapper + 8-7 useExportFile 共通化）
- [UI-06a: 在庫照会画面](function-design/58-ui-stock-inquiry.md) — StockInquiryPage, useStockInquiry, StockDetailContent, derive-stock-state, format-stock-display, format-last-date, filter-low-stock-list（業務ロジックあり版テンプレ、3 useQuery 部分障害許容 + StockInquiryListResult 正規化 + 色分け契約 H + 検索駆動表示契約 I（D-138 で撤去し、UI-01a と 1 つの画面「商品と在庫」にする。proposed・未実装） + 1 件自動展開 + selected 不在 clear + 選択行直下インライン展開 + フォールバックカード + toggle/toggle-group 新規 add（collapsible 未使用化）+ CSV 取込み invalidation）
- [UI-06c: 商品別在庫変動履歴](function-design/66-ui-stock-movements.md) — StockMovementsPage, useStockMovements, StockHistoryTable（業務ロジックあり版テンプレ、2 useQuery 部分障害許容 + `listMovements` の履歴〈movement と計数〉filter/pagination + 元記録リンク + URL state + Windows native L3）
- [UI-01a: 商品検索・一覧](function-design/50-ui-product-list.md) — ProductListPage, patterns/SearchBar（旧 ProductSearchBar、PR-B で統合）, ProductTable, patterns/Pagination（Design Phase 更新済み、REQ-103、URL state、既存 `commands.searchProducts` 契約、`list_departments` CMD 設計、pagination、廃番表示、HID scanner 前提）
- [UI-01b: 商品登録・編集](function-design/51-ui-product-form.md) — ProductFormPage / ProductForm / StockUnitField / DiscontinueConfirmDialog（routes/ 系に更新済み、generated command、supplier 候補、廃番確認、保存 toast）
- [UI-01c: 商品一括インポート](function-design/60-ui-product-import.md) — ProductImportPage, product-import reducer, preview / duplicate / commit flow（Design Phase 更新済み、REQ-104、generated `previewImport` / `commitImport` 契約、共通 FilePicker（D-054、§6.5.4）、上書き確認、Windows native L3）
- [UI-14: 一括価格改定](function-design/77-ui-bulk-price-revision.md) — PriceRevisionPage, PriceRevisionFilters, PriceRevisionTable（Design Phase 更新済み、REQ-105/106、行単位確定、取引先の漸進補完、price_history 導出の「最近改定」、Windows native L3 は後続実装 PR B）
- [UI-02: 入庫記録](function-design/61-ui-receiving.md) — ReceivingPage, header/item form, product search add, recent receiving list（Design Phase 更新済み、REQ-201、generated `createReceiving` / `listReceivings` 契約、HID scanner 前提の Enter 追加、冪等キー、query invalidation、Windows native L3）
- [UI-03: 返品・交換](function-design/63-ui-return-exchange.md) — ReturnExchangePage, header/item/image form, product search add, recent return list（Design Phase 更新済み、REQ-202、generated `createReturn` / `listReturns` / `saveReceiptImage` 契約、レジでの戻し方の問いと明細ごとの在庫の反映（D-148）、HID scanner 前提の Enter 追加、画像添付、冪等キー、query invalidation、Windows native L3）
- [UI-04: 手動販売出庫](function-design/62-ui-manual-sale.md) — ManualSalePage, header/item form, product search add, PLU確認, result links（Design Phase 更新済み、REQ-203、generated `createManualSale` 契約、HID scanner 前提の Enter 追加、PLU登録済み確認、冪等キー、query invalidation、日次売上「手動」Badge L3）
- [UI-05: 廃棄・破損](function-design/64-ui-disposal.md) — DisposalPage, header/item form, product search add, recent disposal list（Design Phase 更新済み、REQ-204、generated `createDisposal` / `listDisposals` 契約、明細単位の種別/理由、HID scanner 前提の Enter 追加、冪等キー、query invalidation、Windows native L3）
- [UI-08: PLU書出し](function-design/67-ui-plu-export.md) — PluExportPage, Diff/Full, native save dialog, prepare/confirm two-step command, app-side exported state, CV17 1.1.1 manual gate（Design Phase 更新済み、REQ-402）
- [UI-11b: バックアップ・復元](function-design/68-ui-backup-restore.md) — BackupRestorePage, backup settings, manual backup, backup list, restore safety flow（Design Phase 追加済み（実装済み）、QR-05 / REQ-901、復元前強制バックアップ + break-glass 例外 + cache clear + Windows native L3）
- [UI-11a: 閾値設定（在庫少の基準）](function-design/69-ui-threshold-settings.md) — ThresholdSettingsPage, useThresholdSettings, useSaveThresholds, extract-thresholds（Design Phase 追加済み（実装済み）、QR系 / D-4、既存 getSettings / updateSetting 契約、所有 2 key 限定、整数 1〜99999 検証、dirty key のみ順次保存 + 部分失敗表示、Windows native L3 軽量 2 項目）
- [UI-10: 棚卸し](function-design/73-ui-stocktake.md) — StocktakePage, useStocktakeStatus, useStocktakeItems, useSaveStockCount, useCompleteStocktake, useFindStocktakeItem, useLastCompletedStocktake（Design Phase 追加済み（実装済み）、REQ-205、検索/スキャン主動線、数の保存と「入れ直す」・要再確認の一覧・完了の拒否の一覧〈D-118 の b1〉、常時確認確定、前回 total_cost 比較、10-4a channel 不採用確定、Windows native L3）
- [UI-11c: 操作ログ画面](function-design/74-ui-operation-logs.md) — OperationLogsPage, useOperationLogs（Design Phase 追加済み（実装済み）、REQ-902 / TRACE-D3、`/settings/logs` route、`listLogs` 期間拡張 + 新規 `list_log_operation_types`、canonical operation_type registry、detail_json 既知field要約+安全上限、関連記録リンク明示 contract、範囲外page回復、empty 2系統、retry、REQ-902/905 traceability 是正、Windows native L3 8 項目）
- [UI-13: 在庫整合性検証画面](function-design/75-ui-integrity-check.md) — IntegrityCheckPage（Design Phase 追加済み（実装済み）、REQ-904、`/settings/integrity` route、idle/running/completed、generated integrity commands、直近確認日時、個別選択 + 確認dialog、100件paging、fix summary・skipped警告・retry選択保持、非色状態表示、Windows native L3 3項目）
- [入出庫記録・在庫変動追跡 完成形](function-design/65-inventory-record-traceability.md) — REQ-206/207/208、入庫/返品・交換/手動販売/廃棄・破損/CSV取込み/棚卸しの一覧・詳細、在庫変動履歴との相互リンク、取消/訂正、操作ログとの役割分担

### トレーサビリティ
- [REQ トレーサビリティマトリクス](function-design/90-traceability.md) — AUTO-GENERATED（`cd src-tauri && cargo run --bin generate_traceability` で再生成。REQ ↔ 設計書 ↔ テスト対応表、`-- --check` が CI / pre-push の drift gate）
