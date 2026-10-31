// crate 内の未使用シンボルは個別精査を将来判断とし、モジュール単位の許可を維持する
#[allow(dead_code)]
mod biz;
mod cmd;
#[allow(dead_code)]
mod constants;
// pub: dev bin (src/bin/seed_demo_data.rs) から init_database / DbError を使用する
#[allow(dead_code)]
pub mod db;
#[cfg(test)]
mod finite_enum_contract_tests;
#[allow(dead_code)]
mod io;
#[allow(dead_code)]
mod mnt;
#[cfg(test)]
mod test_tracing;

// pub: dev tooling 専用のデモデータ seed ロジック。src/bin/seed_demo_data.rs と
// tests/seed_test.rs から参照される
#[allow(dead_code)]
pub mod seed_demo;

use cmd::AppState;
use mnt::diagnostic_log::DiagnosticLogConfig;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::Manager;

#[derive(Debug)]
enum StartupDatabaseError {
    RestoreReconcile(String),
    LegacyMigration(String),
    DatabaseInit(String),
    SchemaNewerThanApp(String),
}

impl std::fmt::Display for StartupDatabaseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RestoreReconcile(message)
            | Self::LegacyMigration(message)
            | Self::DatabaseInit(message)
            | Self::SchemaNewerThanApp(message) => formatter.write_str(message),
        }
    }
}

impl StartupDatabaseError {
    fn operator_message(&self) -> Option<String> {
        match self {
            Self::RestoreReconcile(details) => Some(format!(
                "前回の復元状態を安全に確定できませんでした。データ保護のため起動を中止します。\nアプリを再起動してください。繰り返し失敗する場合は管理者へ連絡してください。\n{details}"
            )),
            Self::LegacyMigration(details) => Some(format!(
                "旧データは無事です。アプリを再起動すると移行を再試行します。\n繰り返し失敗する場合は管理者へ連絡してください。\n{details}"
            )),
            Self::DatabaseInit(details) => Some(format!(
                "データベースの準備に失敗したため、起動を中止しました。アプリを再起動してもう一度お試しください。繰り返し失敗する場合は診断ログ（アプリのデータフォルダ内）を添えて管理者へ連絡してください。\n{details}"
            )),
            // MNT-03-D11: 再起動では直らないため DatabaseInit と文言を分ける
            Self::SchemaNewerThanApp(details) => Some(format!(
                "このデータは、より新しい版のアプリで使われています。この版のアプリで書き込むとデータを傷めるおそれがあるため、起動を中止しました（データは変更していません）。新しい版のアプリを入れ直してから起動してください。わからない場合は管理者へ連絡してください。\n{details}"
            )),
        }
    }
}

#[derive(Clone, Copy)]
enum StartupFailureKind {
    AppDataDir,
    CreateAppDataDir,
    Run,
}

impl StartupFailureKind {
    fn fixed_message(self) -> &'static str {
        match self {
            Self::AppDataDir => "アプリのデータ保存場所を確認できなかったため、起動を中止しました。パソコンを再起動してもう一度お試しください。繰り返し失敗する場合は管理者へ連絡してください。",
            Self::CreateAppDataDir => "アプリのデータ保存場所を作成できなかったため、起動を中止しました。ディスクの空き容量を確認し、パソコンを再起動してもう一度お試しください。繰り返し失敗する場合は管理者へ連絡してください。",
            Self::Run => "アプリを起動できませんでした。アプリを再起動してもう一度お試しください。繰り返し失敗する場合は管理者へ連絡してください。",
        }
    }
}

fn startup_failure_message(kind: StartupFailureKind, details: &impl std::fmt::Display) -> String {
    format!("{}\n{details}", kind.fixed_message())
}

fn run_startup_step<T, E, O, S>(
    kind: StartupFailureKind,
    operation: O,
    show_fatal: S,
) -> Result<T, E>
where
    E: std::fmt::Display,
    O: FnOnce() -> Result<T, E>,
    S: FnOnce(&str),
{
    operation().inspect_err(|error| {
        show_fatal(&startup_failure_message(kind, error));
    })
}

fn handle_run_failure<R>(
    error: impl std::fmt::Display,
    show_fatal: impl FnOnce(&str),
    exit: impl FnOnce(i32) -> R,
) {
    show_fatal(&startup_failure_message(StartupFailureKind::Run, &error));
    exit(1);
}

fn prepare_database(app_data: &std::path::Path) -> Result<db::DbConnection, StartupDatabaseError> {
    prepare_database_with(
        app_data,
        || {
            std::env::current_dir()
                .map_err(|error| format!("作業フォルダを確認できません: {error}"))
        },
        db::migrate_legacy_db,
    )
}

fn run_startup_auto_backup(conn: &db::DbConnection, app_data: &std::path::Path) {
    match mnt::backup::resolve_backup_dir(conn, app_data) {
        Ok(backup_dir) => {
            if let Err(e) = mnt::backup::check_auto_backup(conn, &backup_dir) {
                tracing::warn!(error = %e, "自動バックアップチェックに失敗");
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "バックアップ保存先の設定読取に失敗（自動バックアップをスキップ）");
        }
    }
}

fn prepare_database_with<C, F>(
    app_data: &std::path::Path,
    cwd: C,
    migrate: F,
) -> Result<db::DbConnection, StartupDatabaseError>
where
    C: FnOnce() -> Result<std::path::PathBuf, String>,
    F: FnOnce(&std::path::Path, &std::path::Path) -> Result<bool, std::io::Error>,
{
    prepare_database_with_init(app_data, cwd, migrate, db::init_database)
}

fn prepare_database_with_init<C, F, I>(
    app_data: &std::path::Path,
    cwd: C,
    migrate: F,
    init: I,
) -> Result<db::DbConnection, StartupDatabaseError>
where
    C: FnOnce() -> Result<std::path::PathBuf, String>,
    F: FnOnce(&std::path::Path, &std::path::Path) -> Result<bool, std::io::Error>,
    I: FnOnce(&str) -> Result<db::DbConnection, db::DbError>,
{
    let db_path = app_data.join("inventory.db");
    mnt::backup::reconcile_restore(&db_path)
        .map_err(|error| StartupDatabaseError::RestoreReconcile(error.to_string()))?;

    let new_db_exists = db_path.try_exists().map_err(|error| {
        StartupDatabaseError::LegacyMigration(format!("新DBの存在確認に失敗しました: {error}"))
    })?;
    if !new_db_exists {
        let cwd = cwd().map_err(StartupDatabaseError::LegacyMigration)?;
        match migrate(&cwd, app_data) {
            Ok(true) => tracing::info!(
                old_dir = %cwd.display(),
                new_dir = %app_data.display(),
                "既存DBを安全な単一snapshotへ移行しました（旧ファイルは保持）"
            ),
            Ok(false) => {}
            Err(error) => {
                return Err(StartupDatabaseError::LegacyMigration(format!(
                    "旧DBの移行に失敗しました: {error}"
                )))
            }
        }
    }

    let db_path_text = db_path.to_str().ok_or_else(|| {
        StartupDatabaseError::DatabaseInit("DBパスの文字列変換に失敗".to_string())
    })?;
    let conn = init(db_path_text).map_err(|error| match error {
        db::DbError::SchemaNewerThanApp { .. } => {
            StartupDatabaseError::SchemaNewerThanApp(error.to_string())
        }
        _ => StartupDatabaseError::DatabaseInit(error.to_string()),
    })?;
    if let Err(error) = mnt::backup::complete_reconciled_restore(&conn, &db_path) {
        // committed snapshot は利用可能。ログ補完/manifest cleanup は次回起動で再試行する。
        tracing::warn!(%error, "復元後処理の補完を次回起動へ持ち越し");
    }
    Ok(conn)
}

fn record_pre_window_fatal(message: &str) {
    tracing::error!(message, "pre-window initialization failed");
}

fn show_pre_window_fatal(message: &str) {
    record_pre_window_fatal(message);
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

        let title: Vec<u16> = std::ffi::OsStr::new("Inventory startup error")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let body: Vec<u16> = std::ffi::OsStr::new(message)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let dialog = std::thread::spawn(move || unsafe {
            MessageBoxW(
                std::ptr::null_mut(),
                body.as_ptr(),
                title.as_ptr(),
                MB_OK | MB_ICONERROR,
            )
        });
        if dialog.join().is_err() {
            tracing::error!("pre-window error dialog thread panicked");
        }
    }
    #[cfg(not(windows))]
    eprintln!("起動できませんでした: {message}");
}

/// tauri-specta で TS 型 bindings を `src/lib/bindings.ts` に生成（ADR-002 / 2026-04-20 採用）
///
/// `#[specta::specta]` を付けた command の Rust 型を TypeScript 型定義 + command wrapper に
/// 変換して出力する。`src/lib/bindings.ts` は **git tracked (vendor-in 運用、PR commit に同梱)**、
/// debug build 時と `bin/generate_bindings` 経由で自動再生成される。
///
/// - debug build: `run()` の先頭から呼ばれて毎起動時に最新化
/// - release build: 呼ばれない（bindings.ts は開発時のみ必要）
/// - `.gitignore` の `# src/lib/bindings.ts` の行で vendor-in に切替済（旧: gitignore 対象 / 現: PR に含めて push）
///
/// 現時点で specta 適用済み:
/// - CMD-01: create_product / update_product / toggle_discontinue / search_products / list_departments / list_suppliers / get_product / preview_import / commit_import
/// - CMD-02: create_receiving / list_receivings
/// - CMD-03: create_return / list_returns
/// - CMD-04: create_manual_sale
/// - CMD-05: create_disposal / list_disposals
/// - CMD-05b: list_inventory_records / get_disposal_record
/// - CMD-06: get_stock_detail / list_low_stock / list_movements
/// - CMD-07: parse_and_validate_csv / commit_csv_import / rollback_csv_import / list_csv_imports
/// - CMD-12: parse_and_validate_daily_report / commit_daily_report_import / rollback_daily_report_import / list_daily_report_imports
/// - CMD-08: prepare_plu_export / confirm_plu_export_saved / list_plu_dirty
/// - CMD-09: get_daily_sales / get_monthly_sales / export_sales_csv
/// - CMD-10: get_active_stocktake / start_stocktake / get_stocktake_items / find_stocktake_item / get_last_completed_stocktake / get_stocktake_record / save_stock_count / preview_stock_count_placement / zero_uncounted_items / list_recheck_products / complete_stocktake
/// - CMD-11: get_settings / update_setting / list_logs / create_backup / check_auto_backup / list_backups / restore_backup / save_receipt_image
///
/// Phase 2 以降で段階的に拡張する。
#[cfg(debug_assertions)]
pub fn export_specta_bindings() -> Result<(), String> {
    let bindings_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("src")
        .join("lib")
        .join("bindings.ts");
    export_bindings_to(&bindings_path)
}

// test から tempdir の path を渡して同じ経路（export → 整形 → 定数追記 → 置換）を通す。
#[cfg(debug_assertions)]
fn export_bindings_to(bindings_path: &std::path::Path) -> Result<(), String> {
    use tauri_specta::{collect_commands, Builder};

    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        // CMD-01: 商品管理
        cmd::product_cmd::create_product,
        cmd::product_cmd::update_product,
        cmd::product_cmd::revise_product_price,
        cmd::product_cmd::toggle_discontinue,
        cmd::product_cmd::search_products,
        cmd::product_cmd::bulk_set_plu_target,
        cmd::product_cmd::list_departments,
        cmd::product_cmd::list_suppliers,
        cmd::product_cmd::create_supplier,
        cmd::product_cmd::rename_supplier,
        cmd::product_cmd::merge_suppliers,
        cmd::product_cmd::list_suppliers_with_usage,
        cmd::product_cmd::list_price_history,
        cmd::product_cmd::get_product,
        cmd::product_cmd::preview_import,
        cmd::product_cmd::commit_import,
        // CMD-02: 入庫
        cmd::receiving_cmd::create_receiving,
        cmd::receiving_cmd::list_receivings,
        cmd::receiving_cmd::get_receiving_record,
        // CMD-03: 返品・交換
        cmd::return_cmd::create_return,
        cmd::return_cmd::list_returns,
        cmd::return_cmd::get_return_record,
        // CMD-04: 手動販売出庫
        cmd::manual_sale_cmd::create_manual_sale,
        cmd::manual_sale_cmd::get_manual_sale_record,
        // CMD-05: 廃棄・破損
        cmd::disposal_cmd::create_disposal,
        cmd::disposal_cmd::list_disposals,
        cmd::disposal_cmd::list_inventory_records,
        cmd::disposal_cmd::get_disposal_record,
        // CMD-06: 在庫照会
        cmd::inventory_cmd::get_stock_detail,
        cmd::inventory_cmd::list_low_stock,
        cmd::inventory_cmd::list_movements,
        // CMD-07: CSV取込み
        cmd::csv_import_cmd::parse_and_validate_csv,
        cmd::csv_import_cmd::commit_csv_import,
        cmd::csv_import_cmd::rollback_csv_import,
        cmd::csv_import_cmd::preview_rollback_csv_import,
        cmd::csv_import_cmd::list_csv_imports,
        cmd::csv_import_cmd::get_csv_import_record,
        // CMD-12: 日報取込み
        cmd::daily_report_import_cmd::parse_and_validate_daily_report,
        cmd::daily_report_import_cmd::commit_daily_report_import,
        cmd::daily_report_import_cmd::rollback_daily_report_import,
        cmd::daily_report_import_cmd::list_daily_report_imports,
        cmd::daily_report_import_cmd::scan_register_sd,
        cmd::daily_report_import_cmd::parse_and_validate_daily_report_from_sd,
        // CMD-08: PLU書出し
        cmd::plu_export_cmd::prepare_plu_export,
        cmd::plu_export_cmd::confirm_plu_export_saved,
        cmd::plu_export_cmd::import_plu_register_snapshot,
        cmd::plu_export_cmd::get_plu_slot_summary,
        cmd::plu_export_cmd::list_plu_dirty,
        // CMD-09: 売上集計
        cmd::sales_cmd::get_daily_sales,
        cmd::sales_cmd::get_monthly_sales,
        cmd::sales_cmd::export_sales_csv,
        // CMD-10: 棚卸し
        cmd::stocktake_cmd::get_active_stocktake,
        cmd::stocktake_cmd::start_stocktake,
        cmd::stocktake_cmd::get_stocktake_items,
        cmd::stocktake_cmd::find_stocktake_item,
        cmd::stocktake_cmd::get_last_completed_stocktake,
        cmd::stocktake_cmd::get_stocktake_record,
        cmd::stocktake_cmd::save_stock_count,
        cmd::stocktake_cmd::preview_stock_count_placement,
        cmd::stocktake_cmd::zero_uncounted_items,
        cmd::stocktake_cmd::list_recheck_products,
        cmd::stocktake_cmd::complete_stocktake,
        cmd::integrity_cmd::run_integrity_check,
        cmd::integrity_cmd::fix_integrity,
        // CMD-11: 設定・ログ・バックアップ・画像
        cmd::settings_cmd::get_settings,
        cmd::settings_cmd::update_setting,
        cmd::settings_cmd::list_logs,
        cmd::settings_cmd::list_log_operation_types,
        cmd::settings_cmd::create_backup,
        cmd::settings_cmd::check_auto_backup,
        cmd::settings_cmd::list_backups,
        cmd::settings_cmd::get_effective_backup_dir,
        cmd::settings_cmd::restore_backup,
        cmd::settings_cmd::save_receipt_image,
    ]);

    // 一時 file（同じ directory 内）へ生成・整形・定数追記まで終えてから置換する。
    // 途中で失敗しても既存の bindings_path には触れない。固定名だと tauri dev の
    // 自動生成と CLI 実行が並走したとき互いの一時 file を truncate / rename / remove
    // し合い、両方 exit 0 のまま壊れた bindings.ts が残り得る（Final Review finding #1）。
    // process id + counter で候補名を作るだけでは足りない: pid は PID namespace 間で
    // 一意ではなく、同じ directory を共有する別 namespace の process が同じ候補を
    // 選び得る（Final Review pass A finding #1）。`create_new` で候補を排他的に確保し、
    // 既に他者が確保済みなら次の候補へ進む。
    let temp_path = acquire_temp_path(bindings_path)?;

    let result = (|| -> Result<(), String> {
        builder
            .export(specta_typescript::Typescript::default(), &temp_path)
            .map_err(|e| {
                format!(
                    "TS bindings の export に失敗しました ({}): {e}",
                    bindings_path.display()
                )
            })?;
        normalize_generated_bindings(&temp_path).map_err(|e| {
            format!(
                "TS bindings の整形に失敗しました ({}): {e}",
                bindings_path.display()
            )
        })?;
        append_generated_constants(&temp_path).map_err(|e| {
            format!(
                "TS bindings の定数追記に失敗しました ({}): {e}",
                bindings_path.display()
            )
        })?;
        std::fs::rename(&temp_path, bindings_path).map_err(|e| {
            format!(
                "TS bindings の置換に失敗しました ({} -> {}): {e}",
                temp_path.display(),
                bindings_path.display()
            )
        })
    })();

    if result.is_err() {
        // best-effort: 置換前の一時 file を残さない
        let _ = std::fs::remove_file(&temp_path);
    }

    result
}

// bindings_path と同じ directory 内の一時 file 候補 path を作る。末尾は必ず `.tmp`
// （"一時 file が directory に残っていない" 判定の目印）。pid と counter を引数に
// 取ることで、test が特定の候補を確実に言い当てられる（production は
// `acquire_temp_path` から実 process id と 0 起点で呼ぶ）。
#[cfg(debug_assertions)]
fn temp_candidate_path(
    bindings_path: &std::path::Path,
    pid: u32,
    counter: u64,
) -> std::path::PathBuf {
    let file_name = bindings_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    bindings_path.with_file_name(format!("{file_name}.{pid}.{counter}.tmp"))
}

// 一時 file を排他的に確保する。候補は `temp_candidate_path` で作り、`create_new` で
// 最初に確保できた path だけを自分の一時 file として使う。他者が既に確保済みの候補は
// 消さず次へ進む。無限 retry にはせず、上限に達したら export 段の失敗として Err にする。
#[cfg(debug_assertions)]
fn acquire_temp_path(bindings_path: &std::path::Path) -> Result<std::path::PathBuf, String> {
    const MAX_ATTEMPTS: u64 = 32;

    for counter in 0..MAX_ATTEMPTS {
        let candidate = temp_candidate_path(bindings_path, std::process::id(), counter);
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(_) => return Ok(candidate),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => {
                return Err(format!(
                    "TS bindings の export に失敗しました ({}): {e}",
                    bindings_path.display()
                ))
            }
        }
    }
    Err(format!(
        "TS bindings の export に失敗しました ({}): 一時 file の候補が {MAX_ATTEMPTS} 回とも使用中でした",
        bindings_path.display()
    ))
}

#[cfg(debug_assertions)]
fn normalize_generated_bindings(path: &std::path::Path) -> std::io::Result<()> {
    let original = std::fs::read_to_string(path)?;
    let mut lines = original
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect::<Vec<_>>();

    while matches!(lines.last(), Some(line) if line.is_empty()) {
        lines.pop();
    }

    let mut normalized = lines.join("\n");
    normalized.push('\n');

    if normalized != original {
        std::fs::write(path, normalized)?;
    }

    Ok(())
}

#[cfg(debug_assertions)]
fn append_generated_constants(path: &std::path::Path) -> std::io::Result<()> {
    const EXPORT_PREFIX: &str = "export const CSV_IMPORT_FILE_SIZE_LIMIT";
    let original = std::fs::read_to_string(path)?;
    let mut lines = original
        .lines()
        .filter(|line| !line.trim_start().starts_with(EXPORT_PREFIX))
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    while matches!(lines.last(), Some(line) if line.is_empty()) {
        lines.pop();
    }
    lines.push(String::new());
    lines.push(format!(
        "{EXPORT_PREFIX}: number = {};",
        format_typescript_integer(constants::CSV_IMPORT_FILE_SIZE_LIMIT)
    ));
    let mut generated = lines.join("\n");
    generated.push('\n');
    std::fs::write(path, generated)
}

#[cfg(debug_assertions)]
fn format_typescript_integer(value: usize) -> String {
    let mut reversed = String::new();
    for (index, character) in value.to_string().chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            reversed.push('_');
        }
        reversed.push(character);
    }
    reversed.chars().rev().collect()
}

#[cfg(all(test, debug_assertions))]
mod bindings_generation_tests {
    use super::{
        append_generated_constants, export_bindings_to, normalize_generated_bindings,
        temp_candidate_path,
    };

    // 一時 file の名前は呼出しごとに変わるため、固定名の不在ではなく `.tmp` で終わる
    // entry が directory に無いことで「一時 file が残らない」を確認する。
    fn dir_has_tmp_entry(dir: &std::path::Path) -> bool {
        std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .any(|entry| entry.file_name().to_string_lossy().ends_with(".tmp"))
    }

    // WF 系 meta-test のため `test_` prefix なし命名（pre-push step ④ の REQ 番号必須対象外）
    #[test]
    fn normalize_generated_bindings_trims_trailing_whitespace() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bindings.ts");
        std::fs::write(&path, "type A = {\n\tfield: string,\t\n}\n\n").unwrap();

        normalize_generated_bindings(&path).unwrap();

        let normalized = std::fs::read_to_string(path).unwrap();
        assert_eq!(normalized, "type A = {\n\tfield: string,\n}\n");
    }

    #[test]
    fn test_bindings_req104_appends_file_size_limit_idempotently() {
        // REQ-104 / D-054: specta export 後の手書き定数 export は再実行しても重複しない。
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bindings.ts");
        std::fs::write(&path, "export type Existing = string;\n").unwrap();

        append_generated_constants(&path).unwrap();
        append_generated_constants(&path).unwrap();

        let generated = std::fs::read_to_string(path).unwrap();
        assert_eq!(
            generated
                .matches("export const CSV_IMPORT_FILE_SIZE_LIMIT")
                .count(),
            1
        );
        assert!(
            generated.ends_with("export const CSV_IMPORT_FILE_SIZE_LIMIT: number = 20_971_520;\n")
        );
    }

    // T1: 成功経路。生成・整形・定数追記まで終えた内容が置換された出力先に残り、一時 file は残らない。
    #[test]
    fn export_bindings_to_succeeds_and_leaves_no_temp_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bindings.ts");

        export_bindings_to(&path).unwrap();

        let generated = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            generated
                .matches("export const CSV_IMPORT_FILE_SIZE_LIMIT")
                .count(),
            1
        );
        assert!(!generated.lines().any(|line| line != line.trim_end()));
        assert!(!dir_has_tmp_entry(dir.path()));
    }

    // T2: 一時 file の確保（export 段）の失敗。親 directory を作れない（同名の file が
    // 塞ぐ）と Err になり、message は既存の export 失敗と同じ形で出力先 path を含む。
    // specta の export_to は通常 create_dir_all で親を自動生成するため単に親 directory
    // が無いだけでは失敗しないが、`acquire_temp_path` の `create_new` は親が directory
    // でない時点で失敗するため、この条件では specta の export より前に一時 file の
    // 確保で落ちる（Final Review pass A finding #1）。
    #[test]
    fn export_bindings_to_fails_when_parent_directory_cannot_be_created() {
        let dir = tempfile::tempdir().unwrap();
        let blocker = dir.path().join("blocker");
        std::fs::write(&blocker, "not a directory").unwrap();
        let path = blocker.join("bindings.ts");

        let message = export_bindings_to(&path).unwrap_err();

        assert!(message.contains(&path.display().to_string()));
    }

    // T3: 置換段の失敗。出力先が directory だと rename が失敗し、その directory と
    // 中身（既存内容）はそのまま残り、一時 file も残らない。
    #[test]
    fn export_bindings_to_fails_when_target_is_a_directory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bindings.ts");
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("marker.txt"), "keep").unwrap();

        let result = export_bindings_to(&path);

        assert!(result.is_err());
        assert!(path.is_dir());
        assert!(path.join("marker.txt").exists());
        assert!(!dir_has_tmp_entry(dir.path()));
    }

    // T4: 既存 file の保護。出力先の directory を書込み不可にして一時 file を確保できなく
    // すると（Final Review pass A finding #1 以降、一時 file の確保 = export 段で落ちる）
    // Err になり、出力先の既存内容は byte 単位で変わらない（一時 file 名が呼出しごとに
    // 一意になったため、名前を事前に知って directory で塞ぐ方法は使えない）。directory の
    // 書込み権限を mode で落とす条件は unix 限定（Windows では T3 が既存内容の保護を部分的
    // に見る）。root や権限を無視する filesystem では 0o555 でも書けてしまい export が
    // 正当に成功し得るため、production 呼出しの前に probe file で書込み可否を確かめ、
    // 書けてしまう環境ではこの test を対象外として skip する（Final Review pass A #2 /
    // pass B P3-3）。
    #[cfg(unix)]
    #[test]
    fn export_bindings_to_preserves_existing_content_when_directory_is_read_only() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bindings.ts");
        let existing = "export type Existing = string;\n";
        std::fs::write(&path, existing).unwrap();

        let original_mode = std::fs::metadata(dir.path()).unwrap().permissions().mode();
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555)).unwrap();

        // production 呼出しの前に probe file で書込み可否を確かめる。書けてしまう環境
        // では export の成功可否を skip 判定に使わない（mutant を隠さないため）。
        let probe = dir.path().join("write-probe");
        let write_is_blocked = std::fs::write(&probe, "probe").is_err();
        if !write_is_blocked {
            let _ = std::fs::remove_file(&probe);
        }

        let result = write_is_blocked.then(|| export_bindings_to(&path));

        // tempdir の削除が失敗しないよう、assert より先に必ず権限を戻す。
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(original_mode))
            .unwrap();

        let Some(result) = result else {
            // この環境では directory の書込み拒否が成立しない（root 実行など）ため対象外。
            return;
        };

        assert!(result.is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), existing);
    }

    // Final Review pass A finding #1: process id は PID namespace 間で一意ではないため、
    // 別 namespace の process が同じ候補（pid + counter）を選び得る。最初の候補
    // （counter=0）を他者が既に確保している状態を作り、生成は Ok で完成物を公開し、
    // 先に在った file には触れない（内容も存在も変わらない）ことを固定する。
    #[test]
    fn export_bindings_to_skips_a_temp_candidate_already_claimed_by_someone_else() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bindings.ts");

        let claimed_by_someone_else = temp_candidate_path(&path, std::process::id(), 0);
        std::fs::write(&claimed_by_someone_else, "not mine").unwrap();

        let result = export_bindings_to(&path);

        assert!(result.is_ok(), "{result:?}");
        assert_eq!(
            std::fs::read_to_string(&claimed_by_someone_else).unwrap(),
            "not mine"
        );

        let generated = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            generated
                .matches("export const CSV_IMPORT_FILE_SIZE_LIMIT")
                .count(),
            1
        );
    }

    // Final Review finding #1: 固定名の一時 file だと並走する呼出しが互いの一時 file を
    // truncate / rename / remove し合い、両方 exit 0 のまま壊れた bindings.ts が
    // 残り得た。同じ出力先へ複数 thread から同時に呼び出し、全部 Ok・出力が単独実行と
    // 一致・一時 file が残らないことを固定する回帰 test（8 thread、race を検出するのに
    // 十分な数で数秒以内に終わる）。
    #[test]
    fn export_bindings_to_is_safe_under_concurrent_calls_to_the_same_target() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bindings.ts");

        let results: Vec<Result<(), String>> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..8)
                .map(|_| scope.spawn(|| export_bindings_to(&path)))
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });
        assert!(results.iter().all(|r| r.is_ok()), "{results:?}");

        let generated = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            generated
                .matches("export const CSV_IMPORT_FILE_SIZE_LIMIT")
                .count(),
            1
        );

        let baseline_dir = tempfile::tempdir().unwrap();
        let baseline_path = baseline_dir.path().join("bindings.ts");
        export_bindings_to(&baseline_path).unwrap();
        let baseline = std::fs::read_to_string(&baseline_path).unwrap();
        assert_eq!(generated, baseline);

        assert!(!dir_has_tmp_entry(dir.path()));
    }

    // T5: 整形段の失敗。存在しない path は Err。
    #[test]
    fn normalize_generated_bindings_fails_for_missing_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing").join("bindings.ts");

        assert!(normalize_generated_bindings(&path).is_err());
    }

    // T6: 定数追記段の失敗。存在しない path は Err。
    #[test]
    fn append_generated_constants_fails_for_missing_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing").join("bindings.ts");

        assert!(append_generated_constants(&path).is_err());
    }

    #[test]
    fn test_startup_spec_sfv_t1_app_data_dir_failure_is_operator_visible() {
        // SPEC-SFV-D1/D2 / Matrix T1: oracle は production 定数を共有せず独立転記する。
        let shown = std::cell::RefCell::new(Vec::new());
        let result: Result<std::path::PathBuf, std::io::Error> = super::run_startup_step(
            super::StartupFailureKind::AppDataDir,
            || Err(std::io::Error::other("synthetic app-data detail")),
            |message| shown.borrow_mut().push(message.to_string()),
        );

        assert!(result.is_err());
        let messages = shown.borrow();
        assert_eq!(messages.len(), 1);
        let expected_fixed = "アプリのデータ保存場所を確認できなかったため、起動を中止しました。パソコンを再起動してもう一度お試しください。繰り返し失敗する場合は管理者へ連絡してください。";
        assert_eq!(
            messages[0].strip_suffix("\nsynthetic app-data detail"),
            Some(expected_fixed)
        );
        assert!(messages[0].contains("synthetic app-data detail"));
        assert!(!messages[0].contains("診断ログ"));

        let source = include_str!("lib.rs");
        assert!(source.contains(
            "let app_data = run_startup_step(\n                StartupFailureKind::AppDataDir,\n                || app.path().app_data_dir(),\n                show_pre_window_fatal,\n            )?;"
        ));
    }

    #[test]
    fn test_startup_spec_sfv_t2_create_app_data_dir_failure_is_operator_visible() {
        // SPEC-SFV-D1/D2 / Matrix T2: T1 と異なる固定文言を独立転記で固定する。
        let shown = std::cell::RefCell::new(Vec::new());
        let result: Result<(), std::io::Error> = super::run_startup_step(
            super::StartupFailureKind::CreateAppDataDir,
            || Err(std::io::Error::other("synthetic create-dir detail")),
            |message| shown.borrow_mut().push(message.to_string()),
        );

        assert!(result.is_err());
        let messages = shown.borrow();
        assert_eq!(messages.len(), 1);
        let expected_fixed = "アプリのデータ保存場所を作成できなかったため、起動を中止しました。ディスクの空き容量を確認し、パソコンを再起動してもう一度お試しください。繰り返し失敗する場合は管理者へ連絡してください。";
        let t1_fixed = "アプリのデータ保存場所を確認できなかったため、起動を中止しました。パソコンを再起動してもう一度お試しください。繰り返し失敗する場合は管理者へ連絡してください。";
        assert_eq!(
            messages[0].strip_suffix("\nsynthetic create-dir detail"),
            Some(expected_fixed)
        );
        assert!(messages[0].contains("synthetic create-dir detail"));
        assert!(!messages[0].contains("診断ログ"));
        assert_ne!(expected_fixed, t1_fixed);

        let source = include_str!("lib.rs");
        assert!(source.contains(
            "run_startup_step(\n                StartupFailureKind::CreateAppDataDir,\n                || std::fs::create_dir_all(&app_data),\n                show_pre_window_fatal,\n            )?;"
        ));
    }

    #[test]
    fn test_startup_spec_sfv_t3_database_init_failure_has_operator_message() {
        // SPEC-SFV-D3 / Matrix T3: Option signature を維持し、固定部を独立転記する。
        let message =
            super::StartupDatabaseError::DatabaseInit("synthetic database detail".to_string())
                .operator_message()
                .expect("DatabaseInit must be operator-visible");
        let expected_fixed = "データベースの準備に失敗したため、起動を中止しました。アプリを再起動してもう一度お試しください。繰り返し失敗する場合は診断ログ（アプリのデータフォルダ内）を添えて管理者へ連絡してください。";
        assert_eq!(
            message.strip_suffix("\nsynthetic database detail"),
            Some(expected_fixed)
        );
        assert!(message.contains("synthetic database detail"));
        assert!(message.contains("診断ログ（アプリのデータフォルダ内）"));

        let source = include_str!("lib.rs");
        let branch_start = source
            .rfind("let conn = match prepare_database(&app_data)")
            .expect("production DatabaseInit branch must exist");
        let branch_end = source[branch_start..]
            .find("// 5. 操作ログ自動削除")
            .expect("production DatabaseInit branch must end before step 5")
            + branch_start;
        let branch = &source[branch_start..branch_end];
        assert!(branch.contains("if let Some(message) = error.operator_message()"));
        assert!(branch.contains("show_pre_window_fatal(&message);"));
    }

    #[test]
    fn test_startup_req903_d11_newer_schema_stops_startup_with_own_message() {
        // REQ-903 / MNT-03-D11 / Matrix T9
        let app_data = tempfile::tempdir().unwrap();
        let db_path = app_data.path().join("inventory.db");
        let newer = crate::db::test_support::write_newer_schema_db(&db_path);
        let app_max = newer - 1;

        // (a) 実 DB: DatabaseInit でなく固有の区分で止まる
        let error = super::prepare_database(app_data.path()).unwrap_err();
        assert!(matches!(
            error,
            super::StartupDatabaseError::SchemaNewerThanApp(_)
        ));

        // (b) 固定部（packet Scope S3 の文を独立転記）+ "\n" + 版の detail
        let message = error
            .operator_message()
            .expect("SchemaNewerThanApp must be operator-visible");
        let expected_fixed = "このデータは、より新しい版のアプリで使われています。この版のアプリで書き込むとデータを傷めるおそれがあるため、起動を中止しました（データは変更していません）。新しい版のアプリを入れ直してから起動してください。わからない場合は管理者へ連絡してください。";
        let expected_detail = format!(
            "データの版がこのアプリより新しいため開けません（データの版: {newer}、このアプリが扱える版: {app_max}）"
        );
        assert_eq!(message, format!("{expected_fixed}\n{expected_detail}"));

        // (c) 他の init 失敗は従来どおり DatabaseInit
        let other = super::prepare_database_with_init(
            app_data.path(),
            || Ok(app_data.path().to_path_buf()),
            |_old, _new| Ok(false),
            |_path| {
                Err(crate::db::DbError::ConnectionFailed(
                    "injected connection failure".to_string(),
                ))
            },
        )
        .unwrap_err();
        assert!(matches!(
            other,
            super::StartupDatabaseError::DatabaseInit(_)
        ));
    }

    #[test]
    fn test_startup_spec_sfv_t4_run_failure_calls_fatal_reporter() {
        // SPEC-SFV-D1 / Matrix T4: process exit を起こさず reporter 配線と文言を検証する。
        let shown = std::cell::RefCell::new(Vec::new());
        let exit_codes = std::cell::RefCell::new(Vec::new());
        super::handle_run_failure(
            "synthetic run detail",
            |message| shown.borrow_mut().push(message.to_string()),
            |code| exit_codes.borrow_mut().push(code),
        );

        let messages = shown.borrow();
        assert_eq!(messages.len(), 1);
        let expected_fixed = "アプリを起動できませんでした。アプリを再起動してもう一度お試しください。繰り返し失敗する場合は管理者へ連絡してください。";
        assert_eq!(
            messages[0].strip_suffix("\nsynthetic run detail"),
            Some(expected_fixed)
        );
        assert!(messages[0].contains("synthetic run detail"));
        assert_eq!(*exit_codes.borrow(), vec![1]);

        let source = include_str!("lib.rs");
        assert!(source.contains(
            ".run(tauri::generate_context!())\n        .unwrap_or_else(|error| {\n            handle_run_failure(error, show_pre_window_fatal, std::process::exit);\n        });"
        ));
        assert!(!source.contains(".expect(\"error while running tauri application\")"));
    }

    #[test]
    fn test_startup_spec_sfv_t5_success_does_not_call_fatal_reporter() {
        // SPEC-SFV-D1 / Matrix T5: 正常起動側では fatal 文言生成・表示を呼ばない。
        let fatal_called = std::cell::Cell::new(false);
        let result: Result<&str, std::io::Error> = super::run_startup_step(
            super::StartupFailureKind::AppDataDir,
            || Ok("ready"),
            |_| fatal_called.set(true),
        );

        assert_eq!(result.unwrap(), "ready");
        assert!(!fatal_called.get());
    }

    #[test]
    fn test_startup_req901_d2_resolve_error_warns_and_skips_auto_backup() {
        // REQ-901 / MNT-01-D2 / Matrix C5
        let app_data = tempfile::tempdir().unwrap();
        let db_path = app_data.path().join("inventory.db");
        let conn = crate::db::init_database(db_path.to_str().unwrap()).unwrap();
        crate::db::system_repo::upsert_setting(&conn, "backup_enabled", "1").unwrap();
        let _failure = crate::mnt::backup::fail_setting_read("backup_path");

        let (_, logs) =
            crate::test_tracing::capture(|| super::run_startup_auto_backup(&conn, app_data.path()));

        assert!(logs.contains("WARN") && logs.contains("自動バックアップをスキップ"));
        assert!(
            !app_data.path().join("backups").exists(),
            "resolve失敗時はcheck_auto_backupを呼んではならない"
        );
    }

    #[test]
    fn test_startup_req903_m8_migration_error_stops_before_database_creation() {
        // REQ-903 / Matrix M5, M8
        let app_data = tempfile::tempdir().unwrap();
        let cwd = tempfile::tempdir().unwrap();
        let db_path = app_data.path().join("inventory.db");
        let result = super::prepare_database_with(
            app_data.path(),
            || Ok(cwd.path().to_path_buf()),
            |_old, _new| Err(std::io::Error::other("injected migration failure")),
        );
        let error = result.unwrap_err();
        assert!(matches!(
            &error,
            super::StartupDatabaseError::LegacyMigration(_)
        ));
        assert!(
            !db_path.exists(),
            "init_database must not run after migration failure"
        );

        let cwd_error = super::prepare_database_with(
            app_data.path(),
            || Err("injected CWD failure".to_string()),
            |_old, _new| Ok(false),
        );
        assert!(cwd_error.is_err());
        assert!(!db_path.exists());

        let message = error.operator_message().unwrap();
        assert!(message.contains("旧データは無事です"));
        assert!(message.contains("再起動すると移行を再試行"));
        assert!(message.contains("管理者へ連絡"));
        let (_, logs) = crate::test_tracing::capture(|| super::record_pre_window_fatal(&message));
        assert!(logs.contains("ERROR"));
        assert!(logs.contains("pre-window initialization failed"));
    }

    #[test]
    fn test_startup_req901_b6_fail_closed_reconcile_is_operator_visible_before_init() {
        // REQ-901 / Matrix B6: R3/R5/R7 は実 startup dispatcher で可視化され、DB初期化へ進まない。
        for branch in ["r3", "r5", "r7"] {
            let app_data = tempfile::tempdir().unwrap();
            let db_path = app_data.path().join("inventory.db");
            let manifest =
                std::path::PathBuf::from(format!("{}.restore_manifest", db_path.display()));
            let main_backup =
                std::path::PathBuf::from(format!("{}.restore_backup", db_path.display()));
            std::fs::write(&main_backup, b"must-survive").unwrap();
            match branch {
                "r3" => {
                    std::fs::write(
                        &manifest,
                        br#"{"attempt_id":"r3","original_files":["main"],"phase":"active"}"#,
                    )
                    .unwrap();
                    std::fs::write(
                        format!("{}-wal.restore_backup", db_path.display()),
                        b"unexpected",
                    )
                    .unwrap();
                }
                "r5" => {}
                "r7" => std::fs::write(&manifest, b"not-json").unwrap(),
                _ => unreachable!(),
            }

            let init_called = std::cell::Cell::new(false);
            let result = super::prepare_database_with_init(
                app_data.path(),
                || panic!("CWD resolution must not run after reconcile failure"),
                |_old, _new| panic!("migration must not run after reconcile failure"),
                |_path| {
                    init_called.set(true);
                    panic!("DB init must not run after reconcile failure")
                },
            );
            let error = result.unwrap_err();
            assert!(matches!(
                error,
                super::StartupDatabaseError::RestoreReconcile(_)
            ));
            let message = error
                .operator_message()
                .expect("RestoreReconcile must be operator-visible");
            assert!(message.contains("データ保護のため起動を中止"));
            assert!(message.contains("アプリを再起動してください"));
            assert!(message.contains("管理者へ連絡してください"));
            assert!(!init_called.get(), "branch={branch}");
            assert_eq!(std::fs::read(&main_backup).unwrap(), b"must-survive");
        }
    }

    #[test]
    fn test_startup_req903_m7_existing_new_db_does_not_resolve_cwd() {
        // REQ-903 / Matrix M7: 新DB既存判定は CWD 非依存で先行する。
        let app_data = tempfile::tempdir().unwrap();
        let db_path = app_data.path().join("inventory.db");
        drop(crate::db::init_database(db_path.to_str().unwrap()).unwrap());

        let conn = super::prepare_database_with(
            app_data.path(),
            || Err("CWD must not be resolved".to_string()),
            |_old, _new| panic!("migration must be skipped when the new DB exists"),
        )
        .unwrap();
        drop(conn);
    }

    #[test]
    fn test_startup_req901_b9_temp_artifacts_route_through_reconcile() {
        // REQ-901 / Matrix B9: temp-only と canonical+temp は実 startup dispatcher で T0 を通る。
        let app_data = tempfile::tempdir().unwrap();
        let db_path = app_data.path().join("inventory.db");
        let temp_path =
            std::path::PathBuf::from(format!("{}.restore_manifest.tmp", db_path.display()));
        std::fs::write(&temp_path, b"temp-only").unwrap();
        drop(
            super::prepare_database_with(
                app_data.path(),
                || Ok(app_data.path().to_path_buf()),
                |_old, _new| Ok(false),
            )
            .unwrap(),
        );
        assert!(!temp_path.exists());

        let conn = crate::db::open_existing_database(db_path.to_str().unwrap()).unwrap();
        conn.execute(
            "INSERT INTO suppliers (name, created_at) VALUES ('old', '2026-07-18T00:00:00')",
            [],
        )
        .unwrap();
        drop(conn);
        let backup_path = std::path::PathBuf::from(format!("{}.restore_backup", db_path.display()));
        let manifest_path =
            std::path::PathBuf::from(format!("{}.restore_manifest", db_path.display()));
        std::fs::rename(&db_path, &backup_path).unwrap();
        std::fs::write(
            &manifest_path,
            br#"{"attempt_id":"startup-t0","original_files":["main"],"phase":"active"}"#,
        )
        .unwrap();
        std::fs::write(&temp_path, b"stale-commit-temp").unwrap();

        let reopened = super::prepare_database_with(
            app_data.path(),
            || Err("CWD must not be needed after reconcile restores main".to_string()),
            |_old, _new| panic!("migration must not run after reconcile restores main"),
        )
        .unwrap();
        let names: Vec<String> = reopened
            .prepare("SELECT name FROM suppliers ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(names, vec!["old"]);
        assert!(!temp_path.exists() && !manifest_path.exists() && !backup_path.exists());
    }

    #[test]
    fn test_startup_req901_b9_committed_marker_survives_init_failure_then_converges() {
        // REQ-901 / Matrix B9: committed reconcile 後の init_database 失敗でも marker を再々起動へ残す。
        let app_data = tempfile::tempdir().unwrap();
        let db_path = app_data.path().join("inventory.db");
        drop(crate::db::init_database(db_path.to_str().unwrap()).unwrap());
        let manifest_path =
            std::path::PathBuf::from(format!("{}.restore_manifest", db_path.display()));
        std::fs::write(
            &manifest_path,
            br#"{"attempt_id":"init-retry","original_files":["main"],"phase":"committed"}"#,
        )
        .unwrap();

        let failed = super::prepare_database_with_init(
            app_data.path(),
            || Err("CWD must not run".to_string()),
            |_old, _new| panic!("migration must not run"),
            |_path| {
                Err(crate::db::DbError::ConnectionFailed(
                    "injected init failure".into(),
                ))
            },
        );
        assert!(matches!(
            failed,
            Err(super::StartupDatabaseError::DatabaseInit(_))
        ));
        assert!(manifest_path.exists());

        let conn = super::prepare_database_with(
            app_data.path(),
            || Err("CWD must not run".to_string()),
            |_old, _new| panic!("migration must not run"),
        )
        .unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM operation_logs
                 WHERE operation_type = 'backup_restore'
                   AND json_extract(detail_json, '$.attempt_id') = 'init-retry'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
        assert!(!manifest_path.exists());
    }

    // -----------------------------------------------------------------------
    // 22 §16 の作り直しの手順（vB、MNT-03-D14）の起動の test（Matrix R1〜R4）
    // -----------------------------------------------------------------------

    const VB_STOP: &str = "商品か原価の記録がある DB はこの版へ移せません。原価を 1/100 円で持つ版は空の DB から始めます。開発・試験 DB は作り直してください（22 §16 の作り直しの手順）: products";

    /// `path` に版 `version`（vB の前）まで当てた DB を作り、商品を 1 行入れる
    fn write_db_with_product(path: &std::path::Path, version: i64) {
        let conn = rusqlite::Connection::open(path).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        crate::db::migration::migrate_up_to(&conn, version).unwrap();
        crate::db::insert_pre_vb_product(&conn, "R-OLD");
    }

    /// 作り直しの手順 3: folder を消さずに移し、元の場所に空の folder を作り直す（起動の create_dir_all と同じ）
    fn move_away(from: &std::path::Path, to: &std::path::Path) {
        std::fs::rename(from, to).unwrap();
        assert!(!from.exists());
    }

    fn version_and_products(conn: &crate::db::DbConnection) -> (i64, i64) {
        conn.query_row(
            "SELECT (SELECT MAX(version) FROM schema_versions), (SELECT COUNT(*) FROM products)",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap()
    }

    fn file_hashes(dir: &std::path::Path) -> Vec<(String, String)> {
        use sha2::{Digest, Sha256};
        let mut out: Vec<(String, String)> = std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| {
                let path = entry.unwrap().path();
                let name = path.file_name().unwrap().to_string_lossy().into_owned();
                let hash = format!("{:x}", Sha256::digest(std::fs::read(&path).unwrap()));
                (name, hash)
            })
            .collect();
        out.sort();
        out
    }

    #[test]
    fn test_startup_req903_rebuild_moves_app_data_and_legacy_db_then_empty_vb() {
        // REQ-903 / 22 §16 作り直しの手順 (i) / Matrix R1: {app_data} を丸ごとと作業フォルダの旧 DB を移すと、
        // 取込みは起きず空の DB で vB まで当たる
        let root = tempfile::tempdir().unwrap();
        let app_data = root.path().join("app_data");
        let cwd = root.path().join("cwd");
        let moved = root.path().join("moved");
        for dir in [&app_data, &cwd, &moved] {
            std::fs::create_dir_all(dir).unwrap();
        }
        write_db_with_product(&app_data.join("inventory.db"), crate::db::VB_VERSION - 1);
        write_db_with_product(&cwd.join("inventory.db"), crate::db::VB_VERSION - 1);

        move_away(&app_data, &moved.join("app_data"));
        std::fs::create_dir_all(moved.join("cwd")).unwrap();
        move_away(
            &cwd.join("inventory.db"),
            &moved.join("cwd").join("inventory.db"),
        );
        std::fs::create_dir_all(&app_data).unwrap();
        let moved_before = file_hashes(&moved.join("cwd"));

        let conn = super::prepare_database_with(
            &app_data,
            || Ok(cwd.clone()),
            crate::db::migrate_legacy_db,
        )
        .unwrap();
        // 最新の版まで当たる（版の定数でなく app_max_version と比べる。b1 Matrix T1）
        assert_eq!(
            version_and_products(&conn),
            (crate::db::migration::app_max_version(), 0)
        );
        assert_eq!(
            file_hashes(&moved.join("cwd")),
            moved_before,
            "移した旧 DB は読まない"
        );
    }

    #[test]
    fn test_startup_req903_rebuild_with_committed_manifest_writes_no_restore_log() {
        // REQ-903 / 22 §16 作り直しの手順 (ii) / Matrix R2: 確定済みの manifest ごと {app_data} を移すと、
        // 新しい DB の操作ログに旧 DB の復元の記録は書かれない
        let root = tempfile::tempdir().unwrap();
        let app_data = root.path().join("app_data");
        let cwd = root.path().join("cwd");
        std::fs::create_dir_all(&app_data).unwrap();
        std::fs::create_dir_all(&cwd).unwrap();
        let db_path = app_data.join("inventory.db");
        write_db_with_product(&db_path, crate::db::VB_VERSION - 1);
        std::fs::write(
            format!("{}.restore_manifest", db_path.display()),
            br#"{"attempt_id":"r2-old","original_files":["main"],"phase":"committed"}"#,
        )
        .unwrap();

        move_away(&app_data, &root.path().join("moved-app_data"));
        std::fs::create_dir_all(&app_data).unwrap();

        let conn = super::prepare_database_with(
            &app_data,
            || Ok(cwd.clone()),
            crate::db::migrate_legacy_db,
        )
        .unwrap();
        // 最新の版まで当たる（版の定数でなく app_max_version と比べる。b1 Matrix T1）
        assert_eq!(
            version_and_products(&conn),
            (crate::db::migration::app_max_version(), 0)
        );
        let restore_logs: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM operation_logs WHERE operation_type = 'backup_restore'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(restore_logs, 0);
        assert!(root
            .path()
            .join("moved-app_data")
            .join("inventory.db.restore_manifest")
            .exists());
    }

    #[test]
    fn test_startup_req903_vb_stop_after_earlier_commit_backup_path_readable_from_copy() {
        // REQ-903 / 22 §16 作り直しの手順 (iii)・手順 1 (b) / Matrix R3: v8・v9 が COMMIT した後に vB で止まった DB は、
        // 写しを読み取りだけで開けば backup_path が取れ、元の file は変わらない
        let root = tempfile::tempdir().unwrap();
        let app_data = root.path().join("app_data");
        std::fs::create_dir_all(&app_data).unwrap();
        let db_path = app_data.join("inventory.db");
        write_db_with_product(&db_path, 7);
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            conn.execute(
                "UPDATE app_settings SET value = 'D:\\合成の保存先' WHERE key = 'backup_path'",
                [],
            )
            .unwrap();
        }

        let error = super::prepare_database_with(
            &app_data,
            || panic!("新 DB があるので作業フォルダは見ない"),
            |_old, _new| panic!("新 DB があるので取込みはしない"),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            super::StartupDatabaseError::DatabaseInit(_)
        ));
        let message = error.operator_message().unwrap();
        assert!(message.contains(VB_STOP), "{message}");

        let original = file_hashes(&app_data);
        let copy_dir = root.path().join("copy");
        std::fs::create_dir_all(&copy_dir).unwrap();
        for (name, _) in &original {
            std::fs::copy(app_data.join(name), copy_dir.join(name)).unwrap();
        }
        let copy = rusqlite::Connection::open_with_flags(
            copy_dir.join("inventory.db"),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        let backup_path: String = copy
            .query_row(
                "SELECT value FROM app_settings WHERE key='backup_path'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(backup_path, "D:\\合成の保存先");
        let version: i64 = copy
            .query_row("SELECT MAX(version) FROM schema_versions", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(version, crate::db::VB_VERSION - 1, "v8・v9 は COMMIT 済み");
        drop(copy);
        assert_eq!(
            file_hashes(&app_data),
            original,
            "元の DB の file は変わらない"
        );
    }

    #[test]
    fn test_startup_req903_unmoved_legacy_db_is_imported_and_vb_stops() {
        // REQ-903 / 22 §16 作り直しの手順の対照 / Matrix R4: 作業フォルダの旧 DB を移し忘れると取り込まれて vB で止まる
        let root = tempfile::tempdir().unwrap();
        let app_data = root.path().join("app_data");
        let cwd = root.path().join("cwd");
        std::fs::create_dir_all(&app_data).unwrap();
        std::fs::create_dir_all(&cwd).unwrap();
        write_db_with_product(&cwd.join("inventory.db"), crate::db::VB_VERSION - 1);

        let error = super::prepare_database_with(
            &app_data,
            || Ok(cwd.clone()),
            crate::db::migrate_legacy_db,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            super::StartupDatabaseError::DatabaseInit(_)
        ));
        assert!(error.operator_message().unwrap().contains(VB_STOP));
        let imported = rusqlite::Connection::open_with_flags(
            app_data.join("inventory.db"),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        assert_eq!(
            version_and_products(&imported),
            (crate::db::VB_VERSION - 1, 1),
            "旧 DB が取り込まれている"
        );
    }

    #[test]
    fn test_startup_req901_b10_guard_registration_precedes_mutation_setup() {
        // REQ-901 / Matrix B10: source-order regression guard.
        let source = include_str!("lib.rs");
        let guard = source.find("tauri_plugin_single_instance::init").unwrap();
        let setup = source.find(".setup(|app|").unwrap();
        assert!(
            guard < setup,
            "single-instance guard must precede mutation setup"
        );
    }

    #[test]
    fn test_startup_req901_b11_guard_failure_prevents_mutation_setup() {
        // REQ-901 / Matrix B11: injected guard plugin setup failure is observable and fail-closed.
        use std::sync::atomic::{AtomicBool, Ordering};
        static MUTATION_REACHED: AtomicBool = AtomicBool::new(false);
        MUTATION_REACHED.store(false, Ordering::SeqCst);
        let failing_guard =
            tauri::plugin::Builder::<tauri::test::MockRuntime>::new("single-instance-guard")
                .setup(|_, _| Err("injected single-instance initialization failure".into()))
                .build();
        let result = tauri::test::mock_builder()
            .plugin(failing_guard)
            .setup(|_| {
                MUTATION_REACHED.store(true, Ordering::SeqCst);
                Ok(())
            })
            .build(tauri::test::mock_context(tauri::test::noop_assets()));
        assert!(result.is_err());
        assert!(!MUTATION_REACHED.load(Ordering::SeqCst));
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    if let Err(e) = export_specta_bindings() {
        eprintln!("警告: {e}");
    }

    let builder = tauri::Builder::default();
    #[cfg(debug_assertions)]
    let builder = builder.plugin(
        tauri::plugin::Builder::<tauri::Wry>::new("startup-run-failure-simulation")
            .setup(|_, _| {
                if matches!(
                    std::env::var("INVENTORY_SIMULATE_RUN_FAILURE").as_deref(),
                    Ok("1")
                ) {
                    return Err("debug startup run failure simulation".into());
                }
                Ok(())
            })
            .build(),
    );

    builder
        // mutation を行う setup より先に single-instance guard を確立する。
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
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

            // 1. 診断ログ初期化（setup 内の最初。失敗してもアプリ起動は続行）
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

            // 3-4. reconcile → legacy migration → DB初期化。いずれかの失敗は fail-closed。
            let conn = match prepare_database(&app_data) {
                Ok(conn) => conn,
                Err(error) => {
                    if let Some(message) = error.operator_message() {
                        show_pre_window_fatal(&message);
                    } else {
                        // Defensive fallback: keep fail-closed visibility if a future variant omits
                        // its operator message while the Option-returning contract is retained.
                        tracing::error!(%error, "database initialization failed");
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

            // 7. State管理（setup 内で manage）
            app.manage(AppState {
                db: Mutex::new(conn),
                preview_cache: Mutex::new(HashMap::new()),
                daily_report_preview_cache: Mutex::new(HashMap::new()),
                register_sd_scan_cache: Mutex::new(HashMap::new()),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // CMD-01: 商品管理
            cmd::product_cmd::create_product,
            cmd::product_cmd::update_product,
            cmd::product_cmd::revise_product_price,
            cmd::product_cmd::toggle_discontinue,
            cmd::product_cmd::search_products,
            cmd::product_cmd::bulk_set_plu_target,
            cmd::product_cmd::list_departments,
            cmd::product_cmd::list_suppliers,
            cmd::product_cmd::create_supplier,
            cmd::product_cmd::rename_supplier,
            cmd::product_cmd::merge_suppliers,
            cmd::product_cmd::list_suppliers_with_usage,
            cmd::product_cmd::list_price_history,
            cmd::product_cmd::get_product,
            cmd::product_cmd::preview_import,
            cmd::product_cmd::commit_import,
            // CMD-02: 入庫
            cmd::receiving_cmd::create_receiving,
            cmd::receiving_cmd::list_receivings,
            cmd::receiving_cmd::get_receiving_record,
            // CMD-03: 返品・交換
            cmd::return_cmd::create_return,
            cmd::return_cmd::list_returns,
            cmd::return_cmd::get_return_record,
            // CMD-04: 手動販売出庫
            cmd::manual_sale_cmd::create_manual_sale,
            cmd::manual_sale_cmd::get_manual_sale_record,
            // CMD-05: 廃棄・破損
            cmd::disposal_cmd::create_disposal,
            cmd::disposal_cmd::list_disposals,
            cmd::disposal_cmd::list_inventory_records,
            cmd::disposal_cmd::get_disposal_record,
            // CMD-06: 在庫照会
            cmd::inventory_cmd::get_stock_detail,
            cmd::inventory_cmd::list_low_stock,
            cmd::inventory_cmd::list_movements,
            // CMD-07: CSV取込み
            cmd::csv_import_cmd::parse_and_validate_csv,
            cmd::csv_import_cmd::commit_csv_import,
            cmd::csv_import_cmd::rollback_csv_import,
            cmd::csv_import_cmd::preview_rollback_csv_import,
            cmd::csv_import_cmd::list_csv_imports,
            cmd::csv_import_cmd::get_csv_import_record,
            // CMD-12: 日報取込み
            cmd::daily_report_import_cmd::parse_and_validate_daily_report,
            cmd::daily_report_import_cmd::commit_daily_report_import,
            cmd::daily_report_import_cmd::rollback_daily_report_import,
            cmd::daily_report_import_cmd::list_daily_report_imports,
            cmd::daily_report_import_cmd::scan_register_sd,
            cmd::daily_report_import_cmd::parse_and_validate_daily_report_from_sd,
            // CMD-08: PLU書出し
            cmd::plu_export_cmd::prepare_plu_export,
            cmd::plu_export_cmd::confirm_plu_export_saved,
            cmd::plu_export_cmd::import_plu_register_snapshot,
            cmd::plu_export_cmd::get_plu_slot_summary,
            cmd::plu_export_cmd::list_plu_dirty,
            // CMD-09: 売上集計
            cmd::sales_cmd::get_daily_sales,
            cmd::sales_cmd::get_monthly_sales,
            cmd::sales_cmd::export_sales_csv,
            // CMD-10: 棚卸し
            cmd::stocktake_cmd::get_active_stocktake,
            cmd::stocktake_cmd::start_stocktake,
            cmd::stocktake_cmd::get_stocktake_items,
            cmd::stocktake_cmd::find_stocktake_item,
            cmd::stocktake_cmd::get_last_completed_stocktake,
            cmd::stocktake_cmd::get_stocktake_record,
            cmd::stocktake_cmd::save_stock_count,
            cmd::stocktake_cmd::preview_stock_count_placement,
            cmd::stocktake_cmd::zero_uncounted_items,
            cmd::stocktake_cmd::list_recheck_products,
            cmd::stocktake_cmd::complete_stocktake,
            // CMD-11 部分: 整合性チェック
            cmd::integrity_cmd::run_integrity_check,
            cmd::integrity_cmd::fix_integrity,
            // CMD-11 残り: 設定・ログ・バックアップ・画像
            cmd::settings_cmd::get_settings,
            cmd::settings_cmd::update_setting,
            cmd::settings_cmd::list_logs,
            cmd::settings_cmd::list_log_operation_types,
            cmd::settings_cmd::create_backup,
            cmd::settings_cmd::check_auto_backup,
            cmd::settings_cmd::list_backups,
            cmd::settings_cmd::get_effective_backup_dir,
            cmd::settings_cmd::restore_backup,
            cmd::settings_cmd::save_receipt_image,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| {
            handle_run_failure(error, show_pre_window_fatal, std::process::exit);
        });
}
