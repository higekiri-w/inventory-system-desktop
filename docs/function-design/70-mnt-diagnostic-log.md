## MNT-04: アプリケーション診断ログ

### 70.1 モジュール構成

```
src-tauri/src/
  mnt/
    mod.rs               -- pub mod diagnostic_log を追加
    diagnostic_log.rs    -- ログ基盤初期化、古ログ削除（本セクション）
    backup.rs            -- 既存（MNT-01、stub）
    log_manager.rs       -- 既存（MNT-02、stub）
    migration.rs         -- 既存（MNT-03）
  lib.rs                 -- run()内の起動シーケンスにログ初期化を追加
```

---

### 70.2 依存クレート

```toml
# Cargo.toml [dependencies] に追加
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "local-time"] }
tracing-appender = "0.2"
time = { version = "0.3", features = ["formatting", "local-offset"] }
```

- tracing: ログマクロ（error!, warn!, info!, debug!）
- tracing-subscriber: フォーマッター、フィルタ、レイヤー合成
- tracing-appender: 日次ファイルローテーション
- time: ローカルタイムスタンプのフォーマット（tracing-subscriberのlocal-time機能が依存）

---

### 70.3 型定義

#### DiagnosticLogConfig構造体

```
struct DiagnosticLogConfig {
    log_dir: PathBuf,          // ログファイルの保存ディレクトリ
    retention_days: u32,       // 保持日数（デフォルト: 30）
    file_prefix: String,       // ファイル名プレフィックス（デフォルト: "app"）
}
```

#### DiagnosticLogError列挙型

```
enum DiagnosticLogError {
    DirectoryCreationFailed(String),  // logs/ディレクトリの作成失敗
    SubscriberInitFailed(String),     // tracing-subscriberの初期化失敗
}
```

この型は BizError や DbError とは独立。ログ初期化はDB接続より前に実行されるため、既存のエラー型階層には組み込まない。

---

### 70.4 init_diagnostics

**関数要求**: tracing基盤を初期化し、ファイルへのログ出力を開始する。アプリ起動シーケンスの最初に呼ばれる

**シグネチャ**:
```
fn init_diagnostics(config: DiagnosticLogConfig) -> Result<(), DiagnosticLogError>
```

**処理ステップ**:
1. `config.log_dir` ディレクトリの存在確認。存在しなければ `std::fs::create_dir_all` で作成
   - 作成失敗 → `DiagnosticLogError::DirectoryCreationFailed` を返す
2. `tracing_appender::rolling::daily(config.log_dir, config.file_prefix)` で日次ローテーションアペンダーを作成
   - ファイル名形式: `{prefix}.YYYY-MM-DD`（tracing-appenderのデフォルト動作）
3. `tracing_subscriber::fmt()` でフォーマッターを構築:
   - `.with_writer(file_appender)` — ファイルに出力
   - `.with_ansi(false)` — ANSIカラーコード無効（ファイル出力のため）
   - `.with_target(true)` — モジュール名を出力（例: `biz::csv_import_service::commit`）
   - `.with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339())` — ISO 8601ローカル時刻
   - `.with_env_filter("inventory_system_tauri_scaffold_lib=info")` — デフォルトINFO。クレート名でフィルタし、外部依存のログを抑制
4. `.try_init()` でグローバルサブスクライバーとして登録
   - 失敗 → `DiagnosticLogError::SubscriberInitFailed` を返す
5. `tracing::info!("診断ログ初期化完了")` を出力して正常終了

**エラーハンドリング**:
- この関数が返すエラーは呼び出し元（lib.rs の run()）でキャッチし、stderrにフォールバック出力してアプリ起動を続行する
- ログ初期化失敗はアプリの致命的エラーにしない（ログが書けなくてもアプリは動くべき）

---

### 70.5 cleanup_old_log_files

**関数要求**: 指定日数を超過した古いログファイルを削除する。アプリ起動時に呼ばれる

**シグネチャ**:
```
fn cleanup_old_log_files(config: &DiagnosticLogConfig) -> Result<u32, std::io::Error>
```

戻り値: 削除したファイル数

**処理ステップ**:
1. `config.log_dir` ディレクトリ内のファイル一覧を `std::fs::read_dir` で取得
   - ディレクトリが存在しない → Ok(0) を返す（初回起動時はログディレクトリがまだない可能性）
   - NotFound 以外の失敗 → `std::io::Error` を返す。`Path::exists()` の
     catch-all で permission / metadata error を不存在へ変換しない
2. 各ファイルについて:
   a. directory entry の取得失敗 → entry error と directory context を
      `tracing::warn!` で記録し、次の entry へ進む
   b. ファイル名を Unicode 文字列へ変換できない → path と変換失敗を
      `tracing::warn!` で記録し、当該 entry をスキップ
   c. ファイル名が `{prefix}.YYYY-MM-DD` パターンに一致するか確認（正規表現: `^{prefix}\.\d{4}-\d{2}-\d{2}$`）
   d. パターン不一致 → スキップ（関係ないファイルを消さない）
   e. ファイル名からYYYY-MM-DD部分を抽出し `chrono::NaiveDate` にパース
   f. owned prefix / shape に一致した値の calendar parse 失敗 →
      filename と error を `tracing::warn!` で記録し、当該 entry をスキップ
   g. `chrono::Local::now().date_naive() - file_date > config.retention_days` → 削除対象
   h. `std::fs::remove_file` で削除
   i. 削除失敗 → `tracing::warn!` で警告を出すがカウントしない。次のファイルに進む
3. 削除したファイル数を返す

**エラーハンドリング**:
- `read_dir` 自体の失敗は NotFound だけ `Ok(0)`、権限問題等は
  `std::io::Error` を返す。呼び出し元で `tracing::warn!` して起動を続行
- 個々の entry 取得、filename 変換、owned filename の日付 parse、
  ファイル削除失敗は警告のみ。後続 entry の処理を中断しない
- ログ初期化（70.4）の後に呼ばれるため、tracing::warn! が使える

**MNT-04-D1: top-level 判定 error は返し、個別 entry failure は記録して継続**

- 決定: NotFound は初回起動の正常な空状態とする。NotFound 以外の top-level
  `read_dir` failure は cleanup 全体の判定を不能にするため上位へ返す。
  iterator 内の個別 entry、filename 変換、owned filename の calendar parse、
  remove failure は他 entry の削除可否を無効にしないため、path / error / contextを
  WARNに残して当該 entryだけskipする
- Why: top-level errorを空dirへ変えると保持期限超過logが残り続けても無観測になる。
  一方、個別entryで全体をabortすると、読める期限超過logまで掃除できず可用性を落とす
- Rejected alternatives: `Path::exists()==false`でcatch-all return /
  `Err(_) => continue`のsilent skip / 個別entry errorでcleanup全体abort
- 見直し契機: diagnostic log filesystem adapterを専用serviceへ置換するとき

**entry failure 注入境界（監査順7 Plan Review P2-1）**:

production と test は次の同一 generic helper を通す。production 専用の別 loop、
test 専用 cleanup implementation、`#[cfg(test)]` だけの entry 処理関数を作らない。

```rust
fn cleanup_log_entries<I>(
    config: &DiagnosticLogConfig,
    today: chrono::NaiveDate,
    entries: I,
) -> u32
where
    I: IntoIterator<Item = std::io::Result<PathBuf>>;
```

production は `read_dir` の各 `DirEntry` を
`entry.map(|entry| entry.path())` で上記 item 型へ変換して渡す。test は同じ helper
へ `Err(io::Error)` と実 tempdir の `PathBuf` を並べ、WARN と後続削除を検証する。
実装レビューでは public `cleanup_old_log_files` と failure-injection test の双方が
この helper を呼び、entry 分類・warn・remove の production-only / test-only
分岐が存在しないことを明示確認する。

---

### 70.6 lib.rs 起動シーケンスの変更

**関数要求**: run() を Tauri setup hook 方式に書き換え、ログ初期化→DB初期化→State管理を setup 内で順序立てて実行する

**変更前の起動順序**:
```
pub fn run() {
    // DB初期化がBuilder外にある（Tauri非推奨パターン）
    let db_path = "inventory.db";  // 相対パス（CWD依存で不安定）
    let conn = db::init_database(db_path).expect("DB初期化に失敗しました");

    let state = AppState { db: Mutex::new(conn), preview_cache: Mutex::new(HashMap::new()) };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(...)
        .run(...)
        .expect("error while running tauri application");
}
```

**変更後の起動順序（setup hook方式、実装同期版）**:
```
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // app_data_dir: tauri.conf.json の identifier から自動解決
            // Linux: ~/.local/share/com.inventory-system.desktop/
            // Windows: C:\Users\{user}\AppData\Roaming\com.inventory-system.desktop\
            let app_data = run_startup_step(
                StartupFailureKind::AppDataDir,
                || app.path().app_data_dir(),
                show_pre_window_fatal,
            )?;
            run_startup_step(
                StartupFailureKind::CreateAppDataDir,
                || std::fs::create_dir_all(&app_data),
                show_pre_window_fatal,
            )?;

            // 1. 診断ログ初期化（setup内の最初。失敗してもアプリ起動は続行）
            let log_config = DiagnosticLogConfig {
                log_dir: app_data.join("logs"),
                retention_days: 30,
                file_prefix: "app".to_string(),
            };
            if let Err(e) = mnt::diagnostic_log::init_diagnostics(&log_config) {
                eprintln!("警告: 診断ログの初期化に失敗しました: {}", e);
            }

            // 2. 古ログファイル削除（ログ初期化後に実行）
            if let Err(e) = mnt::diagnostic_log::cleanup_old_log_files(&log_config) {
                tracing::warn!(error = %e, "古いログファイルの削除に失敗");
            }

            // 3-4. reconcile → legacy移行 → DB初期化。いずれかの失敗も fail-closed（利用者向け固定文言を表示して起動中止）
            let conn = match prepare_database(&app_data) {
                Ok(conn) => conn,
                Err(error) => {
                    if let Some(message) = error.operator_message() {
                        show_pre_window_fatal(&message);
                    }
                    return Err(error.to_string().into());
                }
            };

            // 5. 操作ログ自動削除（DB初期化後に実行。失敗してもアプリ起動は続行）
            if let Err(e) = mnt::log_manager::cleanup_old_logs(&conn) {
                tracing::warn!(error = %e, "操作ログの自動削除に失敗");
            }

            // 6. 自動バックアップチェック（起動時。失敗してもアプリ起動は続行）
            run_startup_auto_backup(&conn, &app_data);

            // 7. State管理（setup内でmanage）
            app.manage(AppState {
                db: Mutex::new(conn),
                preview_cache: Mutex::new(HashMap::new()),
                daily_report_preview_cache: Mutex::new(HashMap::new()),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![...])
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| {
            handle_run_failure(error, show_pre_window_fatal, std::process::exit);
        });
}
```

`app_data_dir` 取得・作成の失敗、および DB 初期化系の失敗はいずれも `.expect()` によるパニックではなく `run_startup_step` / `handle_run_failure` を介した fail-closed 経路（利用者向け固定文言表示 + プロセス終了）に統一されている（`src-tauri/src/lib.rs` `run_startup_step` 定義）。操作ログ自動削除（MNT-02）と自動バックアップチェック（MNT-01）は DB 初期化後・State 管理前に実行され、いずれも失敗してもアプリ起動は継続する。

**app_data_dirの取得方針（2026-04-13 設計変更）**:
- Tauri 2.0 の setup hook 内で `app.path().app_data_dir()` を使用する
- setup hook はウィンドウ作成前に実行される（Tauri公式: "Runs before the main loop, so no windows are yet created"）
- `dirs` クレートの追加は不要。Tauri のパス解決を使うことでパスの一貫性を保つ
- `tauri.conf.json` の `identifier: "com.inventory-system.desktop"` からパスが自動解決される
  - Linux: `~/.local/share/com.inventory-system.desktop/`
  - Windows: `C:\Users\{user}\AppData\Roaming\com.inventory-system.desktop\`

**DB初期化パスの修正（同時実施）**:
- 変更前: `"inventory.db"`（相対パス。CWD依存で不安定）
- 変更後: `app_data.join("inventory.db")`（app_data_dir配下の絶対パス）
- 理由: 相対パスだとアプリの起動元ディレクトリによってDBファイルの場所がブレる。app_data_dir配下に固定することでデータの永続化先が確定する

**setup hookエラー時の挙動**:
- setup が `Err` を返すと `tauri::Builder::run()` がエラーで終了する
- DB初期化失敗（`init_database` のエラー）は `?` で伝搬し、アプリ起動を中断する（データなしでは業務不可のため）
- ログ初期化失敗は `if let Err` でキャッチして続行する（ログなしでもアプリは動くべき）

**テストへの影響**:
- 既存テストはDB初期化パスを `tempfile::tempdir()` 経由で渡しており、setup hook を経由しない。テストコードへの影響はない
- init_diagnostics / cleanup_old_log_files はユニットテストで独立テスト可能

---

### 70.7 既存コードへのtracing埋め込み方針

#### 70.7.1 CMD層: ERRORログの追加

**対象**: `From<BizError> for CmdError` の変換処理（`cmd/mod.rs`）

**方針**: BizError → CmdError 変換時に、全variantで `tracing::error!` を出力する。これによりCMD層を通る全エラーが自動的に1回だけ記録される。

**変更箇所**: `cmd/mod.rs` の `impl From<BizError> for CmdError`

```
impl From<BizError> for CmdError {
    fn from(err: BizError) -> Self {
        // 全variantでERRORログを出力（エラー境界での1回記録）
        tracing::error!(error = %err, "CMD層エラー");
        match err {
            // ... 既存の変換ロジック（変更なし）
        }
    }
}
```

**補足**:
- `%err` は `Display` トレイトを使用。BizError に `Display` 実装がなければ追加する（`{:?}` で代替可）
- CMD層の各コマンド関数内で個別に `tracing::error!` を書く必要はない。`From` 変換に集約する
- `CmdError::internal()` で直接CmdErrorを生成するケース（DB接続取得失敗等）は、そのコマンド関数内で `tracing::error!` を出力する

#### 70.7.2 BIZ層: eprintln! の tracing::warn! 置換

**対象**: 既存の `eprintln!("警告: 操作ログ記録に失敗しました: {}", e)` を `tracing::warn!` に置換

**置換箇所**（12箇所）:
- `biz/product_service.rs`（1箇所）
- `biz/csv_import_service/parse.rs`（1箇所）
- `biz/csv_import_service/commit.rs`（2箇所）
- `biz/csv_import_service/rollback.rs`（1箇所）
- `biz/plu_export_service.rs`（1箇所）
- `biz/integrity_service.rs`（2箇所）
- `biz/stocktake_service.rs`（3箇所）
- `biz/inventory_service/receiving.rs`（確認要）

**置換パターン**:
```
// Before
eprintln!("警告: 操作ログ記録に失敗しました: {}", e);

// After
tracing::warn!(error = %e, "操作ログ記録に失敗");
```

#### 70.7.3 BIZ層: INFO/WARNログの追加

**INFOログ追加箇所**（主要処理の完了記録）:
- `biz/product_service.rs`: create_product完了、commit_import完了
- `biz/csv_import_service/commit.rs`: CSV取込み完了（件数・金額含む）
- `biz/csv_import_service/rollback.rs`: ロールバック完了
- `biz/plu_export_service.rs`: PLU書出し済み確認（件数含む）
- `biz/stocktake_service.rs`: 棚卸し開始、棚卸し確定
- `biz/integrity_service.rs`: 整合性チェック完了（不整合件数含む）

**WARNログ追加箇所**（異常だが続行する場面）:
- `biz/inventory_service/common.rs`: apply_stock_change で stock_after < 0（在庫マイナス警告）
- `biz/csv_import_service/commit.rs`: completed_partial（一部スキップあり）

**INFOログのフォーマット例**:
```
tracing::info!(
    product_code = %product_code,
    "商品登録完了"
);

tracing::info!(
    csv_import_id = import_id,
    total_items = total_items,
    total_amount = total_amount,
    "CSV取込み完了"
);
```

**WARNログのフォーマット例**:
```
tracing::warn!(
    product_code = %product_code,
    stock_after = stock_after,
    "在庫マイナス警告"
);
```

#### 70.7.4 MNT層: INFOログの追加

**追加箇所**:
- `db/mod.rs`: init_database完了（schema_version、テーブル数を含む）
- `mnt/migration.rs`: マイグレーション実行（バージョン番号を含む）

**フォーマット例**:
```
tracing::info!(
    schema_version = current_version,
    "DB初期化完了"
);

tracing::info!(
    version = version,
    "マイグレーション適用: v{}",
    version
);
```

---

### 70.8 テスト方針

#### ユニットテスト

**cleanup_old_log_files のテスト**:
- `test_cleanup_req700_deletes_old_files`: 31日前のファイルを作成し、cleanup後に削除されていること
- `test_cleanup_req700_keeps_recent_files`: 29日前のファイルが削除されないこと
- `test_cleanup_req700_ignores_non_matching_files`: プレフィックス不一致のファイルがスキップされること
- `test_cleanup_req700_empty_directory`: 空ディレクトリでOk(0)が返ること
- `test_cleanup_req700_nonexistent_directory`: 存在しないディレクトリでOk(0)が返ること
- `test_cleanup_req700_distinguishes_not_found_from_read_dir_error`: NotFoundだけOk(0)、
  NotFound以外はErr
- `test_cleanup_req700_entry_error_warns_and_continues`: entry errorをwarnし、後続削除を継続
- `test_cleanup_req700_invalid_calendar_date_warns_and_continues`: owned filenameの
  calendar parse errorをwarnし、後続削除を継続
- `test_cleanup_req700_non_unicode_filename_warns_and_continues`: non-Unicode filenameを
  warnし、後続削除を継続（対応platformのみ）
- `test_cleanup_req700_delete_failure_warns_and_continues`: remove failureをwarnし、後続削除を継続

**init_diagnostics のテスト**:
- `test_init_creates_log_directory`: 存在しないディレクトリが作成されること
- `test_init_writes_log_file`: 初期化後にtracing::info!を呼び、ログファイルが生成されること

※ `tracing::subscriber::set_global_default` はプロセスで1回しか呼べないため、init_diagnosticsの統合テストは `#[cfg(test)]` 内で `tracing_subscriber::fmt().with_writer(...)` のテスト用サブスクライバーを使うか、各テストをバイナリ分離（`tests/` ディレクトリの統合テスト）で実行する

#### 手動検証

- アプリ起動後に `app_data/logs/` ディレクトリにログファイルが存在すること
- エラー操作（存在しない商品コードの検索等）でERRORレベルが記録されること
- 在庫マイナス操作でWARNレベルが記録されること

---

### 70.9 ログ出力フォーマット例

```
2026-04-13T14:30:00.123+09:00  INFO inventory_system_tauri_scaffold_lib::mnt::diagnostic_log: 診断ログ初期化完了
2026-04-13T14:30:00.456+09:00  INFO inventory_system_tauri_scaffold_lib::db: DB初期化完了 schema_version=2
2026-04-13T14:32:01.789+09:00  INFO inventory_system_tauri_scaffold_lib::biz::csv_import_service::commit: CSV取込み完了 csv_import_id=15 total_items=127 total_amount=85430
2026-04-13T14:32:01.790+09:00  WARN inventory_system_tauri_scaffold_lib::biz::inventory_service::common: 在庫マイナス警告 product_code="NU-0023" stock_after=-2
2026-04-13T14:33:00.100+09:00 ERROR inventory_system_tauri_scaffold_lib::cmd: CMD層エラー error=DatabaseError(QueryFailed("FOREIGN KEY constraint failed"))
```
