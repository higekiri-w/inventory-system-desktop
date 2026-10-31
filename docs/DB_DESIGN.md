# 在庫管理システム テーブル定義書

## 計数の置き場と在庫の求め直しの契約（D-118。(a) と b1 で実装済み）

[棚卸しの数の置き場と在庫の求め直しの ADR](adr/2026-10-16-stocktake-count-placement.md)（SPEC-STK-CNT-D1・D2・D9・D10）の保存の契約を詳細化した。(a)（ADR SPEC-STK-CNT-D12）の並びの鍵・計数の表・写しの列は migration v9（[22 §19](function-design/22-mnt-migration.md)）で入った。b1 の列（`csv_imports.machine_no`・`settlement_no`、`sale_records.stock_linked`、`stock_counts.seen_import_id`、`stocktake_items.valuation_quantity`、`stocktakes.cutoff_import_id`・`cutoff_count_id`・`cutoff_movement_id`）と Z004 と phase を結ぶ CHECK は migration v11（[22 §20](function-design/22-mnt-migration.md)）で入った。以下のテーブル一覧・インデックス方針と ER 図はその表と列を載せている（列と制約の正本は下の各 db-design の同名の節）。

- 写しの列（`stock_recheck`・範囲）と在庫連動・共有 JAN: [master](db-design/master-tables.md)。入出庫の TX との接続: [transaction](db-design/transaction-tables.md)。
- Z004 の取込みの machine_no・settlement_no: [POS](db-design/pos-tables.md)。
- 並びの鍵（`business_date`・`phase`・`settlement_no`）・計数の表（`stock_counts`）・求め直し・棚卸し・移行: [tracking](db-design/tracking-system-tables.md)。

EJ の記録の文を持つ表（`ej_records`、proposed・未実装、D-121）: [POS](db-design/pos-tables.md) の「SD の一括取込みと EJ の控えの契約」。

`products.stock_quantity` は記録から求め直した写しで、書くのは BIZ の求め直しの関数だけ（D-051 の原則、D-118）。migration は棚卸しか Z004 の取込みを持つ DB では止まり（開発・試験 DB はリリースの前まで作り直してよい。D-123）、持たない DB では列と表を足して既存の movement の鍵を埋め、写しを求め直す（(a) の migration v9 で実装済み。SQL・CHECK / FK / index・故障注入の試験は 22 §19 のテスト）。

> **現行スキーマの入口**: [ER図](inventory_system_erd.html)、[業務データフロー](diagrams/current-system.md)。適用順序は `src-tauri/src/db/migration.rs` と `schema_v*.rs` を照合する。
> **図と本文の位置付け**: 現行構造と完成形の将来設計を区別する。[図面監査](research/2026-09-16-diagram-audit.md) に、実装と設計の不一致・再検討事項を記録する。

---

## テーブル一覧

| # | テーブル名 | 種別 | 対応REQ | 役割 |
|---|-----------|------|---------|------|
| 1 | products | マスタ | REQ-101〜104 | 商品マスタ。システムの中心 |
| 1a | product_jan_aliases | マスタ | REQ-104, 401 | 前の JAN の別名（JAN を直した商品の前の JAN の売上を同じ商品へたどる。proposed・未実装、D-122） |
| 2 | departments | マスタ | REQ-103, 402 | 部門マスタ。Z005対応＋独自コード発番 |
| 3 | suppliers | マスタ | REQ-201 | 取引先マスタ。サジェスト入力の元データ |
| 4 | receiving_records | トランザクション | REQ-201 | 入庫記録ヘッダ |
| 5 | receiving_items | トランザクション | REQ-201 | 入庫記録明細（商品ごと） |
| 6 | return_records | トランザクション | REQ-202 | 返品・交換記録ヘッダ |
| 7 | return_items | トランザクション | REQ-202 | 返品・交換記録明細（商品ごと） |
| 8 | manual_sales | トランザクション | REQ-203 | 手動販売出庫ヘッダ |
| 9 | manual_sale_items | トランザクション | REQ-203 | 手動販売出庫明細（商品ごと） |
| 10 | disposal_records | トランザクション | REQ-204 | 廃棄・破損記録ヘッダ |
| 11 | disposal_items | トランザクション | REQ-204 | 廃棄・破損記録明細（商品ごと） |
| 12 | csv_imports | POS連携 | REQ-401 | CSV取込み履歴 |
| 12a | csv_import_errors | POS連携 | REQ-401 | CSV取込みエラー・スキップ行 |
| 12b | daily_report_imports | POS連携 | REQ-401 | 日報取込み履歴（Z001/Z002/Z005の束） |
| 12c | daily_report_summary_lines | POS連携 | REQ-401, 501, 502 | 日計サマリ行（Z001由来） |
| 12d | daily_report_payment_lines | POS連携 | REQ-401, 501, 502 | 取引キー・支払集計行（Z002由来） |
| 12e | daily_report_department_lines | POS連携 | REQ-401, 501, 502 | 部門別売上集計行（Z005由来） |
| 13 | sale_records | POS連携 | REQ-401, 501, 502 | 売上レコード（自動＋手動の統合） |
| 13a | plu_slots | POS連携 | REQ-402, 907 | スキャニング PLU のレジ占有・永続予約 |
| 14 | inventory_movements | 在庫追跡 | REQ-303 | 在庫変動履歴（全ての在庫増減の記録） |
| 15 | price_history | 在庫追跡 | REQ-102 | 価格変更履歴 |
| 16 | stocktakes | 棚卸し | REQ-205 | 棚卸しヘッダ |
| 17 | stocktake_items | 棚卸し | REQ-205 | 棚卸し明細（商品ごとの実カウント） |
| 17a | stock_counts | 棚卸し | REQ-205 | 計数の置き場（日付と精算の並びの位置を持つ append-only の計数。migration v9、D-118 の (a)。保存する経路と取込みのカーソルの列 `seen_import_id` は b1〈v11〉） |
| 18 | operation_logs | システム | QR-06 | 操作ログ |
| 19 | app_settings | システム | QR系 | アプリ設定（キー・バリューストア） |

---

## テーブル定義（詳細）

各テーブルのカラム定義・設計意図・業務シナリオは以下のサブドキュメントに記述:

| グループ | ファイル | テーブル |
|---------|---------|---------|
| マスタ | [db-design/master-tables.md](db-design/master-tables.md) | products, product_jan_aliases（proposed、D-122）, departments, suppliers |
| PLU slot | [db-design/plu-tables.md](db-design/plu-tables.md) | plu_slots、PLU snapshot 用 app_settings key |
| トランザクション | [db-design/transaction-tables.md](db-design/transaction-tables.md) | receiving_records/items, return_records/items, manual_sales/items, disposal_records/items |
| POS連携 | [db-design/pos-tables.md](db-design/pos-tables.md) | csv_imports, csv_import_errors, daily_report_imports, daily_report_summary_lines, daily_report_payment_lines, daily_report_department_lines, B-1/B-2パース仕様, sale_records |
| 在庫追跡・棚卸し・システム | [db-design/tracking-system-tables.md](db-design/tracking-system-tables.md) | inventory_movements, price_history, stocktakes/items, stock_counts, operation_logs, app_settings |

---

## 設計方針メモ

### SQLite固有の考慮事項
- **外部キー制約**: SQLiteはデフォルトで外部キーが無効。`PRAGMA foreign_keys = ON;` を接続時に必ず実行する
- **日時の格納**: TEXT型でISO 8601形式（例: 2026-03-21T19:45:00）。SQLiteにはDATETIME型がないが、TEXT型でISO 8601ならソート・比較が正しく動く
- **BOOLEAN型**: SQLiteにはBOOLEAN型がないため、INTEGER型で0/1を使用
- **トランザクション**: 入庫・CSV取込み等の「ヘッダ＋明細＋在庫更新＋変動履歴」はBEGIN〜COMMITで一括コミット。途中で失敗したらROLLBACK
- **マイグレーション**: バージョン管理テーブル（schema_versions）で適用履歴を保持し、アプリ起動時に未適用migrationを順に実行する。業務テーブル一覧とは別の管理表で、[ER図](inventory_system_erd.html) には含める

### ポリモーフィック関連の整合性
- inventory_movementsのreference_type + reference_idはアプリケーション側で整合性を担保
- reference_typeの許容値はCHECK制約で固定（指摘#11対応）
- 参照先のテーブルとIDが正しいかはビジネスロジック層でチェック
- 不整合が起きた場合は操作ログに記録して警告

### POS日報と商品別売上のデータ境界（D-025 / 2026-06-30）
- `daily_report_imports` と配下の `daily_report_*_lines` は、アプリがレジの SD から直接読む（D-111。予備は利用者が選ぶファイル）`Z001` / `Z002` / `Z005` の日報集計を、アプリ内部の日報モデルに変換して保存する。別layoutの受理はadapter互換性であり、通常のoperator手順は増やさない。
- `sale_records` は、Z004または手動販売出庫から得られる商品別売上の正本である。Z001/Z002/Z005は商品コード・JAN単位の明細ではないため、`sale_records` や `inventory_movements` へ擬似展開しない。
- 日報取込みの rollback は日報取込みレコードを `rolled_back` にする論理取消に限定する。在庫変動を作らないため、在庫補正や movement void は発生しない。
- レジ依存のファイル名、列名、文字コード、改行、PCツール手順は POS adapter の責務とし、DBには app core が使う report_date / section / label / amount / quantity_hundredths（個数の 100 倍の整数）/ count / department mapping だけを保存する。

### 日時書式規約（指摘#18対応）
- `*_date` カラム: YYYY-MM-DD（日付のみ。例: sale_date, receiving_date, disposal_date）
- `*_at` カラム: YYYY-MM-DDTHH:MM:SS（日時。例: created_at, updated_at, imported_at）
- タイムゾーン: ローカル（JST）で統一。UTCにしない（1店舗のローカルアプリのため）
- settlement_dateはYYYY-MM-DD（精算日は日付単位）

### D-6: 入出庫記録・在庫変動追跡の完成形（2026-06-27 追加）

商業用の在庫管理として、入庫・返品交換・手動販売出庫・廃棄破損・CSV取込み・棚卸し補正は、作成後に一覧・詳細・在庫変動履歴から追跡できる必要がある。完成形の DB 方針は [function-design/65-inventory-record-traceability.md](function-design/65-inventory-record-traceability.md) §65.6 を正とする。

- 業務記録ヘッダは `active` / `canceled` / `corrected` の状態を持つ方向で設計する。
- 取消は物理削除せず、取消理由と取消日時を残す。
- 訂正は元記録を取消/訂正済みにし、新しい記録を作る。
- 在庫戻しは既存 movement を消さず、逆方向 movement を追加する。
- `operation_logs` は監査・保守ログであり、業務記録や在庫変動履歴の正本にはしない。

### CHECK制約方針（指摘#17対応）
以下の列挙値カラムにCHECK制約を設定する。有限集合の IPC 露出値は D-061 / D-064 により Rust の generated enum を SSOT とする literal union 契約であり、本 CHECK 制約は**同一の値集合を防御として二重化したまま維持する**（DB 層は TEXT のまま schema 不変。DB CHECK と wire enum の対応関係の正本は decision-log D-061 / D-064）:
- products.tax_rate: CHECK(tax_rate IN ('10','8','0'))
- products.stock_unit: CHECK(stock_unit IN ('pcs','sheet','hon','bag','box','kumi','set','ball','cho','m'))（D-113・D-128。vB〈v10〉で巻・cm を外した。[master-tables](db-design/master-tables.md) の「単位と原価の精度の契約」）
- sale_records.source: CHECK(source IN ('auto','manual'))
- inventory_movements.movement_type: CHECK(movement_type IN ('sale_auto','sale_manual','receiving','return','disposal','stocktake'))
- inventory_movements.reference_type: CHECK(reference_type IN ('csv_import','manual_sale','receiving_record','return_record','disposal_record','stocktake') OR reference_type IS NULL)
- return_records.return_type: CHECK(return_type IN ('return','exchange'))
- return_items.direction: CHECK(direction IN ('in','out'))
- disposal_items.disposal_type: CHECK(disposal_type IN ('disposal','damage','other'))
- csv_imports.status: CHECK(status IN ('completed','completed_partial','rolled_back'))
- csv_import_errors.error_type: CHECK(error_type IN ('unmatched_product','invalid_format','invalid_jan','invalid_number'))
- daily_report_imports.status: CHECK(status IN ('completed','rolled_back'))
- daily_report_imports.source_adapter: CHECK(source_adapter IN ('casio_sr_s4000'))
- daily_report_summary_lines.source_file: CHECK(source_file IN ('Z001'))
- daily_report_payment_lines.source_file: CHECK(source_file IN ('Z002'))
- daily_report_department_lines.source_file: CHECK(source_file IN ('Z005'))
- stocktakes.status: CHECK(status IN ('in_progress','completed'))
- manual_sales.reason: CHECK(reason IN ('plu_unregistered','other'))

### インデックス方針（指摘#16対応）
jan_codeとfile_hash以外に、以下のインデックスを初期設定する:
- sale_records(sale_date) — 日次売上一覧の検索用
- sale_records(product_code, sale_date) — 商品別売上集計用
- sale_records(csv_import_id) — ロールバック時の関連レコード特定用
- daily_report_imports(report_date) — 日報取込みの対象日検索用
- daily_report_imports(bundle_hash) — 同一Z001/Z002/Z005束の重複検知用
- daily_report_summary_lines(daily_report_import_id) — 日報表示用
- daily_report_payment_lines(daily_report_import_id) — 日報表示用
- daily_report_department_lines(daily_report_import_id, department_id) — 部門別日報表示と部門マスタ突合用
- inventory_movements(product_code, created_at) — 在庫変動履歴の商品別時系列表示用
- inventory_movements(reference_type, reference_id) — 元操作からの逆引き用
- inventory_movements(product_code, business_date, phase, settlement_no) — `idx_inventory_movements_ledger`。記録から写しを求め直す並びの鍵（migration v9、D-118 の (a)）
- stock_counts(product_code, id) — `idx_stock_counts_product_id`。商品ごとの効く計数を選ぶ（migration v9、D-118 の (a)）
- stocktake_items(stocktake_id, product_code) — 棚卸し中の商品検索用
- products(department_id) — 部門別商品一覧用
- products(is_discontinued) — 廃番フィルタリング用

### stock_quantity整合性チェック（指摘#12対応）
- **前提**: 全商品の在庫はシステム導入時の初期投入（REQ-104一括インポートまたはREQ-101個別登録）から履歴が始まる。初期投入時にinventory_movementsにmovement_type='receiving'で初期在庫レコードを作成し、全商品の履歴起点を保証する
- **pos_stock_sync=0の商品**: CSV取込みで在庫を動かさない商品は、inventory_movementsにsale_autoの行が作られない。手動販売出庫や棚卸し補正でのみ在庫が変動する。整合性チェックはこれらの商品も含めて行う
- **チェックタイミング**: CSV取込み完了時、棚卸し確定時、設定画面からの手動実行
- **チェック方法**: 各商品について、記録（inventory_movements と stock_counts）から求め直した値と、products の写しの 4 項目（`stock_quantity`・`stock_recheck`・`stock_range_low`・`stock_range_high`）を突合する（[36](function-design/36-biz-integrity-check.md)。計数の無い商品では求め直しの値は `is_voided=0` の movement の和と同じ）
- **差異発見時**: operation_logsに記録。画面に「在庫数に不整合があります」と警告表示。差異のある商品一覧を表示
- **復旧方針**: 整合性の画面からの補正は、利用者に差異を提示し、保存している値を記録から求め直した値に合わせて補正する旨を確認する（文言正本は function-design/75-ui-integrity-check.md）。確定した場合のみ、写しを求め直しの結果（`recompute_stock`）へ直接更新する。inventory_movements への行追加はしない（挿入すると記録自体が変わり再チェックが収束しないため）。補正は同一TX内の integrity_fix 操作ログへ before/after 付きで必須記録する（BIZ-07-D2/D3、D-051、D-118）。例外は移行（migration v9 の手順 8、[22 §19](function-design/22-mnt-migration.md)）で、写しと記録の和の違う商品を同じ TX で直し、`trigger: migration` の integrity_fix 操作ログに残して、起動の後に整合性の画面で知らせる

### ファイルパス保存方針（指摘#19対応）
- receipt_image_path: アプリ管理下の相対パスで保存（例: images/receipts/2026-03-21_001.jpg）
- バックアップパス（app_settings.backup_path）: 利用者が指定する絶対パス。デフォルトはアプリインストールフォルダ配下
- 画像ファイルの実体はアプリのデータフォルダ内に保存。DBにはパスのみ記録

### operation_type命名規約（指摘#14対応）
操作種別は `エンティティ_動詞` の形式で統一する:
- product_create, product_update, product_discontinue
- csv_import, csv_rollback
- daily_report_import, daily_report_rollback
- receiving_create
- return_create
- manual_sale_create
- disposal_create
- stocktake_start, stocktake_complete
- plu_export
- backup_create, backup_restore
- setting_update
- log_cleanup

### D-1: 一括インポートのエンコーディング対応（2026-03-29 確定）
- **文字コード判定**: BOM（0xEF 0xBB 0xBF）があればUTF-8、なければCP932としてデコード
- **BOMなしUTF-8は非対応**: 日本語Excelから「CSV（コンマ区切り）」で保存するとCP932、「CSV UTF-8」で保存するとBOM付きUTF-8になるため、BOMなしUTF-8が出るケースはない。仕様として非対応を明記
- **フォーマット検証**: デコード後に必須ヘッダ列の検証（列名・列数の一致チェック）と区切り文字の確認をセットで行う
- **プレビュー必須**: 取込み前に先頭数行を必ずプレビュー画面で表示。CP932デコードが成功しても文字化けしている場合は利用者が目視で気づける
- **デコード失敗時**: 「ファイルの文字コードが判別できません。Excelで保存し直してください」とエラー表示

### D-2: PLU書出し→レジ反映の検知手段（2026-03-29 確定 / 2026-08-18 D-072 更新 / 2026-10-16 D-117 更新）
- **plu_dirty / plu_exported_at の更新タイミング**: PLUファイル生成だけでは更新しない。UI-08で保存先を選び、PLUファイル保存後に利用者が「この書出しを未反映から外す」と明示確認した時点で、生成時の対象商品だけを `plu_dirty=0` にし、`plu_exported_at` に現在日時を記録する。
- **plu_dirty の意味の限定（D-028）**: `plu_dirty` は `plu_target=1`（スキャニングPLU書出し対象）の商品についてのみ「レジ未反映」を意味する。`plu_target=0` の商品は `plu_dirty` の値にかかわらず PLU書出し抽出と UI-00 PLU未反映通知の対象外（抽出・通知クエリが `plu_target=1` 条件を持つ）。詳細は [db-design/master-tables.md](db-design/master-tables.md) plu_target 設計意図と decision-log D-028 を参照。
- **永続 slot と再書出し（D-072 / SPEC-PLS-D1、D5）**: memory No. は `plu_slots` に JAN 単位で永続割当し、書出しごとに再採番しない。Diff / Full はどちらも CV17 へ投入できる。Full は対象の商品の app 管理 slot 全体、Diff は未反映対象を出力する。投入失敗時は Diff / Full の書出し直しで回復する（前に保存した file は取り込まない。保存の後の照合で枠が変わりうる。D-117）。外部登録と free slot は出力しない。アプリは枠を消さず（clear 行を書かない）、別の JAN に使い回さない（D-117 (1) の採用済みの方針。今の build は Clear 行を書き〈`PLU_CLEAR_ROW_ENABLED = true`〉、撤去は PLU の書出しの見直しの runtime。[33 §16](function-design/33-biz-plu-export-service.md) の冒頭の runtime の注記）。
- **レジ反映の確認**: アプリはその場では確かめられない。次の締めの SD の読取りで Z004 の全スロットを照合すると（D-117 の採用済みの方針。今の build は PLU 書出しの画面で利用者が選ぶ Z004 で照合する）、レジに入ったコードと名称は分かる（`reserved → active`）が、単価・課税方式・部門リンクの反映は Z004 に無い（D-117 (7)、D-011 の前提の訂正）。`plu_exported_at` は「アプリ側でPLUファイルを保存済みにした日時」であり、PCツール受理やレジ反映の証明ではない。
- **画面上の注意表示**: UI-08の完了画面に「アプリで確認できるのはPLUファイル保存まで。PCツールへの取込み、SDカード書出し、レジ読込みは手動確認が必要」と常時表示する。

### D-3: CSV取込み中の排他制御（2026-03-29 確定）
- **UIロック範囲**: Preview開始〜Commit完了/キャンセルまで。取込みボタンをdisabled、画面内にimport_in_progress状態を持つ
- **SQLite設定**: WALモードで読み取りをブロックしない。BUSY_TIMEOUT=5000msで書き込み競合時にリトライ
- **BUSY TIMEOUT失敗時**: 「別の処理が実行中です。しばらく待ってからもう一度お試しください」と利用者に案内
- **「戻る→再実行」事故防止**: Preview画面からブラウザバックで戻った場合もimport_in_progress状態を維持。明示的な「キャンセル」操作でのみロック解除

### D-4: 在庫少閾値の初期値（2026-03-29 確定）
- **初期値**: stock_low_threshold=3（一般商品: 3個以下）、stock_low_threshold_fabric=500（生地: 500cm=5m以下）
- **適用ルール**: 長さの単位（`m`）の商品には stock_low_threshold_fabric、個数の単位（ほかの 9 種）の商品には stock_low_threshold を適用する（D-113、[共通規則](function-design/10-common-rules.md) SPEC-UNIT-D10）
- **設定画面**: 利用者が変更可能。入力バリデーションで0以下の値は拒否（最小値=1）
- **商品個別閾値**: 初期バージョンでは全商品一律。将来拡張でproductsにcustom_low_thresholdカラムを追加する余地を残す

### D-5: 操作ログ保持期間（2026-03-29 確定、2026-10-23 に decision-log D-136 で改定）
- **保持期間**: 3000 日（帳簿書類の 7 年〈申告の期限の翌日から〉に足りる長さ。app_settings の log_retention_days。2026-03-29 の確定は 365 日）。保持日数を確定できないとき（数でない・0）は消さない（[72](function-design/72-mnt-log-manager.md) MNT-02-D1）
- **自動削除タイミング**: アプリ起動時に1日1回だけ実行。前回チェック日をapp_settings（log_last_cleanup_date）に記録し、同日中は再実行しない
- **削除の記録**: 削除件数をoperation_logs（operation_type='log_cleanup'）に1件記録。例: 「42件の操作ログを削除しました（2025-03-29以前）」
- **アーカイブ**: 不要。保持日数を過ぎたログは DB から消える。保持期間を過ぎた後の復元は保証しない（残る backup〈保持は別の設定、既定 3 日〉に残るかは、その backup の作成の時点による）

### 技術的認知負荷の軽減方針
- 俺（Claude）は設計判断の根拠と困りそうなケースを毎回セットで出す
- マスターは業務シナリオでぶつけて「実態と合ってるか」を判断する
- 技術的妥当性の検証責任は俺が持つ
- 実装フェーズではテストコードが機械的に検証する
