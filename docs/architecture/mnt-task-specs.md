# タスク仕様（MNT層）

## 計数の置き場と在庫の求め直しの契約（D-118。(a) と b1 の migration は実装済み）

SPEC-STK-CNT-D9・D10（[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)）。新しい業務判断を MNT へ移さない。(a)（ADR SPEC-STK-CNT-D12）の migration は v9 で実装済み（`src-tauri/src/db/schema_v9.rs`）。b1 の列（`csv_imports`・`sale_records`・`stock_counts.seen_import_id`・棚卸しの完了）を足す migration は v11 で実装済み（`src-tauri/src/db/schema_v11.rs`、[22 §20](../function-design/22-mnt-migration.md)）。

- MNT-03 は [DB の新契約](../DB_DESIGN.md) の列と表を 1 TX で足し、既存の movement の並びの鍵を埋めて写しを求め直す。棚卸しか Z004 の取込みを持つ DB では何も変えずに失敗する（開発・試験 DB はリリースの前まで作り直してよい。D-123。手順は [22 §19](../function-design/22-mnt-migration.md)）。旧行を推定の位置で新しい軸へ移さない。
- MNT-01 の backup / 復旧の規則は変えない。計数の context を持たないので、DB 接続の交換で失効させるものは無い（[43](../function-design/43-cmd-settings-log.md)）。

旧 ADR（時点証拠）の分類・legacy の上限・context の失効は置き換えた。

> **親文書**: [ARCHITECTURE.md](../ARCHITECTURE.md)
> **入力ドキュメント**: `docs/spec/requirements.md`、`docs/spec/requirements-coverage.md`、DB_DESIGN.md（テーブル定義書）

---

### MNT-01: バックアップ・リストア

**タスク要求**: SQLiteデータベースファイルのバックアップ作成と復元を行う

**理由**: データ消失・破損時の復旧手段。利用者が安心してシステムを使うための基盤

**【データ構造】**

入力（バックアップ）: なし（app_settingsからbackup_path, backup_retention_daysを読む）
入力（リストア）: バックアップファイルパス

出力: バックアップファイルパス / リストア結果

**【処理構造】**

**バックアップ:**
1. app_settingsからbackup_pathを取得
2. SQLiteのバックアップAPIまたはファイルコピーでDBファイルを複製
3. ファイル名: inventory_backup_{YYYYMMDD_HHMMSS}.db
4. backup_retention_daysより古いバックアップファイルを削除
5. operation_logsに記録（operation_type='backup_create'）

**リストア:**
1. バックアップファイルの存在確認
2. 現在のDBを一時退避（リストア失敗時の保険）
3. バックアップファイルを現在のDBファイルに上書きコピー
4. DB再接続 + PRAGMA再設定
5. スキーママイグレーション実行（古いバックアップからの復元時にスキーマが古い可能性）
6. operation_logsに記録（operation_type='backup_restore'）

**PC の外の控え（D-114）:**
1. 利用者が USB メモリを保存先として用意する（目印を置き、DB の外の状態の file に登録する）
2. 用意した媒体が見えている間、確かめ済みの最新のバックアップを写し、読み戻して SHA-256 で照合してから正式名にする
3. 最後に PC の外へ写せた日時を状態の file に残し、3 日以上前なら「控えが古い」として画面とホームで知らせる（原因が PC の中の backup の古さなら、その直し方を出す）。状態の file が読めなければ、画面の操作で退避して作り直す。PC の外の控えは任意で、画面の操作でやめられる（状態の file を退避名へ移す）
4. 復元の前に控えを読取り専用で確かめ、新しすぎる版・壊れた控えを確認の手順の前に止める
（正本は [71 §71.11〜§71.13](../function-design/71-mnt-backup.md)）

**【制御構造】**
- 自動バックアップ: app_settings.backup_enabled=1かつbackup_timeに達したら実行。アプリ起動中のみ
- 手動バックアップ: 設定画面のボタンから即時実行
- リストア中は全操作をブロック（DB接続を一度切断するため）

---

### MNT-02: 操作ログ管理

**タスク要求**: システムの主要操作を記録し、閲覧と自動削除を提供する

**理由**: トラブル時の追跡用。「いつ何をしたか」を後から確認できる

**【データ構造】**

入力（記録）: operation_type, summary, detail_json（任意）
入力（閲覧）: ページ番号、1ページあたり件数、operation_typeフィルタ（任意）
入力（削除）: なし（app_settings.log_retention_daysを使用）

出力（閲覧）: [{id, operation_type, summary, detail_json, created_at}], 総件数

**【処理構造】**

**記録:**
1. operation_logsにINSERT（created_at=現在日時）
2. operation_typeは命名規約に従う（DB_DESIGN.md参照）

**閲覧:**
1. operation_logsからcreated_at DESCで取得（ページング: LIMIT/OFFSET）
2. operation_typeフィルタがあればWHERE句に追加

**自動削除:**
1. アプリ起動時にapp_settings.log_last_cleanup_dateを確認
2. 今日と同じ → スキップ（1日1回のみ）
3. 今日と異なる → log_retention_days日前より古いレコードをDELETE
4. 削除件数 > 0 → operation_logsに記録（operation_type='log_cleanup'）
5. log_last_cleanup_dateを今日に更新

**【制御構造】**
- 記録はBIZ層の各処理から呼ばれる。トランザクション内で呼ばれた場合はそのトランザクションに含まれる
- 自動削除はアプリ起動時のみ。バックグラウンドタイマーは不要（1人運用デスクトップアプリ）

---

### MNT-03: スキーママイグレーション

**タスク要求**: DBスキーマのバージョン管理と、アプリ更新時の自動マイグレーションを行う

**理由**: アプリのアップデートでテーブル構造が変わった場合、利用者のDBを自動的に新しいスキーマに移行する

**【データ構造】**

内部テーブル: schema_versions（version INTEGER PRIMARY KEY, applied_at TEXT）

マイグレーションスクリプト: バージョン番号順のSQL文リスト（コード内に埋め込み）

**【処理構造】**

1. schema_versionsテーブルが存在しなければ作成（初回起動）
2. 現在の最大バージョンを取得
3. コード内のマイグレーションリストと比較
4. 未適用のマイグレーションを順番に実行
   - 各マイグレーション: BEGIN → ALTER TABLE等のSQL実行 → schema_versionsにINSERT → COMMIT
   - 失敗時: ROLLBACK → エラー表示（「データベースの更新に失敗しました。サポートに連絡してください」）
5. 初回起動時（schema_versions空）: CREATE TABLE文を全て実行して初期スキーマを構築

**【制御構造】**
- アプリ起動時にIO-01のDB接続初期化から呼ばれる
- マイグレーション実行中は他の操作はまだ始まっていない（起動シーケンスの最初）

---

### MNT-04: アプリケーション診断ログ

**タスク要求**: アプリケーションの全Rust層（CMD/BIZ/IO/MNT）の処理状況をファイルに記録し、障害発生時にバグの原因を特定できるようにする

**理由**: operation_logs（MNT-02）は「利用者が何をしたか」を記録する業務監査ログであり、「なぜ失敗したか」の情報がない。デスクトップアプリはサーバーと違い開発者がリアルタイムで確認できないため、エラー箇所・入力データ・モジュール名を記録するファイルベースの診断ログが必要。対応要求: REQ-700（Issue #24）

**【データ構造】**

技術構成:
- tracing: ログマクロ（error!, warn!, info!, debug!）の提供
- tracing-subscriber: ログフォーマット制御（タイムスタンプ、モジュール名、レベル）
- tracing-appender: ファイル出力、日次ローテーション

ログレベル定義:

| レベル | 記録する場面 | 通常運用 |
|---|---|---|
| ERROR | 処理が失敗して結果を返せなかった（CMD層のエラー境界で1回記録） | 記録する |
| WARN | 処理は続行したが異常な状態（在庫マイナス、棚卸し中の操作等。BIZ層で記録） | 記録する |
| INFO | 主要な処理の開始/完了（DB初期化完了、CSV取込み完了、バックアップ完了等） | 記録する |
| DEBUG | 内部の判断分岐、SQLクエリ詳細、IO層のエラー詳細 | 開発時・トラブル調査時のみ |

ログエントリに含める情報:
- タイムスタンプ（ISO 8601、ローカル時刻）
- ログレベル
- モジュール名（例: biz::csv_import_service::commit）
- メッセージ（処理中のデータ識別子を含む: product_code, csv_import_id等）

ファイル管理:
- 保存先: アプリデータフォルダ/logs/ディレクトリ
- ファイル名: app.YYYY-MM-DD（tracing-appenderの命名規則に準拠）
- 保持期間: 30日

**【処理構造】**

**ログ基盤初期化（アプリ起動シーケンスの最初）:**
1. アプリデータフォルダ/logs/ディレクトリの存在確認（なければ作成）
2. tracing-appenderで日次ローテーションのファイルアペンダーを作成
3. tracing-subscriberでフォーマッター設定（タイムスタンプ + レベル + モジュール名）
4. デフォルトログレベルをINFOに設定
5. グローバルサブスクライバーとして登録
6. 初期化失敗時 → stderrにフォールバック出力してアプリ起動を続行

**古ログファイルの自動削除（アプリ起動時）:**
1. logs/ディレクトリ内のファイル一覧を取得
2. ファイル名からYYYY-MM-DD部分を抽出
3. 30日超過のファイルを削除
4. 削除対象がなければ何もしない

**層別ログ出力ルール（重複防止）:**
- CMD層: エラー境界としてERRORを出力。BizError/CmdErrorをフロントエンドに返す際に1回だけ記録。モジュール名、エラーvariant、処理中のデータ識別子を含める
- BIZ層: 業務判断に関わる異常をWARNで出力（在庫マイナス警告等）。主要処理の完了をINFOで出力（CSV取込み完了、棚卸し確定等）
- IO層: SQLクエリ失敗等の詳細をDEBUGで出力。エラー自体は上位層に伝搬するのみ
- MNT層: バックアップ・マイグレーション等の完了をINFOで、失敗をERRORで出力

**既存コードへの適用:**
- 既存のeprintln!("警告: 操作ログ記録に失敗しました: {}", e)をtracing::warn!に置換
- CMD層の各コマンドでResult::Errを返す箇所にtracing::error!を追加

**【制御構造】**
- 全初期化処理はTauri 2.0のsetup hook内で実行する。setup hookはウィンドウ作成前に実行されるため、ログ初期化→DB初期化の順序が保証される
- 起動シーケンスの順序（setup hook内）: app_data_dir取得 → MNT-04（ログ初期化）→ IO-01（DB初期化、app_data_dir配下）→ MNT-03（マイグレーション）→ State管理
- app_data_dirは `app.path().app_data_dir()` で取得。tauri.conf.jsonのidentifierから自動解決（Linux: `~/.local/share/com.inventory-system.desktop/`）
- ログ初期化自体が失敗した場合はstderrに出力してアプリ起動を続行する（ログが書けなくてもアプリは動く）
- DB初期化失敗はsetup hookからエラーを返し、アプリ起動を中断する（データなしでは業務不可のため）
- operation_logs（MNT-02）とは独立して共存する。診断ログはファイル、操作ログはSQLite。両者は異なる目的・読者・保持期間を持つ
- 古ログファイルの削除タイミングはアプリ起動時。MNT-02のlog_cleanupと同じパターン（1日1回チェック）だが、こちらはファイルシステムのファイル日付で判定するためapp_settingsへの記録は不要
