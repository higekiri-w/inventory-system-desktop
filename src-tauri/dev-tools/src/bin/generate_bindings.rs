//! TS bindings 生成専用バイナリ（開発ツール）
//!
//! `cargo run --bin generate_bindings` で `<project-root>/src/lib/bindings.ts` を生成する。
//! GUI を立ち上げずに bindings.ts だけを更新したい場合に使用する。
//!
//! 出力先は `CARGO_MANIFEST_DIR`（src-tauri/）基準の絶対パスで解決される
//! （`lib.rs::export_specta_bindings`）。実行時の cwd に依存しないため、
//! project root / src-tauri / IDE task runner いずれからでも動く。
//!
//! 通常の tauri dev 起動時は `lib.rs::run()` 冒頭で `export_specta_bindings()`
//! が debug build 時に自動呼び出しされるため、このバイナリは必要に応じて実行する。
//!
//! 生成のいずれかの段（specta export / 整形 / 定数追記 / 置換）が失敗した場合は
//! 完了 message を出さず、stderr に失敗した段と対象 path を出して非 0 で終了する。

#[cfg(debug_assertions)]
fn main() {
    if let Err(e) = inventory_system_tauri_scaffold_lib::export_specta_bindings() {
        eprintln!("{e}");
        std::process::exit(1);
    }
    println!("TS bindings exported to src/lib/bindings.ts");
}

// release build では export_specta_bindings が cfg(debug_assertions) で存在しないため、
// 本バイナリも同じ条件で no-op にする（bindings 生成は開発時専用の workflow）。
#[cfg(not(debug_assertions))]
fn main() {
    eprintln!("generate_bindings is a debug-build tool; run with a debug profile");
    std::process::exit(1);
}
