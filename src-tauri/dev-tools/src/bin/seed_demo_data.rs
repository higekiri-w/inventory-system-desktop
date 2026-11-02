//! デモデータ投入専用バイナリ（開発ツール）
//!
//! Phase 2 UI-00 以降の UI 開発で手動 SQL 投入なしにダッシュボード数値が確認できるよう、
//! 商品 100 件 / 取引先 5 件 / 売上 30 日分 (300 件) / 在庫変動 400 件 を投入する。
//!
//! 実装本体は `inventory_system_tauri_scaffold_lib::seed_demo` にあり、本バイナリは
//! CLI 引数パース + init_database 呼出し + 進捗ログ出力のみ担当する。
//!
//! ## 使い方
//! ```bash
//! # 既定: ./inventory.db に追記
//! cargo run --bin seed_demo_data
//!
//! # 別 DB を指定
//! cargo run --bin seed_demo_data -- --db /tmp/seed-test.db
//!
//! # 既存データを全削除してから再投入（tty で confirm あり）
//! cargo run --bin seed_demo_data -- --reset
//! ```

use inventory_system_tauri_scaffold_lib::db::init_database;
use inventory_system_tauri_scaffold_lib::seed_demo::{run_seed, SeedError};
use std::fmt;
use std::io::{self, BufRead, IsTerminal, Write};
use std::process::ExitCode;

const DEFAULT_DB_PATH: &str = "./inventory.db";

// -------- CLI エラー型 --------

#[derive(Debug)]
enum CliError {
    Usage(String),
    Seed(SeedError),
    Io(io::Error),
    Aborted,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Usage(msg) => write!(f, "引数エラー: {}", msg),
            CliError::Seed(e) => write!(f, "seed エラー: {}", e),
            CliError::Io(e) => write!(f, "IOエラー: {}", e),
            CliError::Aborted => write!(f, "利用者により中断されました"),
        }
    }
}

impl std::error::Error for CliError {}

impl From<SeedError> for CliError {
    fn from(e: SeedError) -> Self {
        CliError::Seed(e)
    }
}

impl From<io::Error> for CliError {
    fn from(e: io::Error) -> Self {
        CliError::Io(e)
    }
}

impl From<inventory_system_tauri_scaffold_lib::db::DbError> for CliError {
    fn from(e: inventory_system_tauri_scaffold_lib::db::DbError) -> Self {
        CliError::Seed(SeedError::from(e))
    }
}

// -------- CLI --------

struct Cli {
    db_path: String,
    reset: bool,
}

fn print_usage() {
    println!("seed_demo_data — デモデータ投入ツール");
    println!();
    println!("USAGE:");
    println!("    cargo run --bin seed_demo_data [-- OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!(
        "    --db <path>    対象 DB ファイルパス (既定: {})",
        DEFAULT_DB_PATH
    );
    println!("    --reset        既存データを全削除してから再投入 (tty で確認あり)");
    println!("    --help         このヘルプを表示");
}

fn parse_args(args: &[String]) -> Result<Cli, CliError> {
    let mut db_path = DEFAULT_DB_PATH.to_string();
    let mut reset = false;
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            "--reset" => {
                reset = true;
                i += 1;
            }
            "--db" => {
                if i + 1 >= args.len() {
                    return Err(CliError::Usage("--db の後にパスが必要です".to_string()));
                }
                db_path = args[i + 1].clone();
                i += 2;
            }
            other => {
                return Err(CliError::Usage(format!("未知の引数: {}", other)));
            }
        }
    }
    Ok(Cli { db_path, reset })
}

// -------- 本体 --------

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cli = match parse_args(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[seed] {}", e);
            print_usage();
            return ExitCode::from(2);
        }
    };

    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(CliError::Aborted) => {
            eprintln!("[seed] 中断しました");
            ExitCode::from(130)
        }
        Err(e) => {
            eprintln!("[seed] 失敗: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli) -> Result<(), CliError> {
    println!("[seed] target DB: {}", cli.db_path);

    // init_database: schema 保証（空なら v1+v2 migration 適用、既存 DB にも追記可）
    let mut conn = init_database(&cli.db_path)?;

    if cli.reset && !confirm_reset()? {
        return Err(CliError::Aborted);
    }

    // reset + seed を 1 トランザクションで実行（partial failure 時は自動 rollback）
    let summary = run_seed(&mut conn, cli.reset)?;
    if cli.reset {
        println!("[seed] reset: 全テーブル DELETE → 再投入完了");
    }

    println!(
        "[seed] suppliers: {} inserted / {} skipped",
        summary.suppliers_inserted, summary.suppliers_skipped
    );
    println!(
        "[seed] products: {} inserted / {} skipped",
        summary.products_inserted, summary.products_skipped
    );
    println!(
        "[seed] inventory_movements (receiving): {} inserted / {} skipped",
        summary.receiving_movements_inserted, summary.receiving_movements_skipped
    );
    println!(
        "[seed] sale_records: {} inserted / {} skipped",
        summary.sale_records_inserted, summary.sale_records_skipped
    );
    println!(
        "[seed] inventory_movements (sale_auto): {} inserted / {} skipped",
        summary.sale_movements_inserted, summary.sale_movements_skipped
    );
    if summary.negative_stock_warnings > 0 {
        println!(
            "[seed] WARN: {} 件の sale で stock_after が負になりました (seed の乱数特性、補正なし)",
            summary.negative_stock_warnings
        );
    }
    println!("[seed] 完了");
    Ok(())
}

// -------- confirm --------

fn confirm_reset() -> Result<bool, CliError> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        eprintln!("[seed] --reset は tty からのみ実行可能です (confirm 不可のため abort)");
        return Ok(false);
    }
    print!("本当に reset しますか？ (yes/no): ");
    io::stdout().flush()?;
    let mut line = String::new();
    let stdin = io::stdin();
    stdin.lock().read_line(&mut line)?;
    Ok(line.trim() == "yes")
}
