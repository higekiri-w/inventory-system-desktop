//! D-132: MSI の製品名・upgrade code・`identifier`・`bundle.targets` を固定する。
//! D-137: bundle（MSI・deb）に入る実行 file を main の 1 つにする crate の形
//! （`src/bin/` と main 以外の `[[bin]]` が無い、`default-members` と `default-run`）を固定する。
//!
//! 期待値は D-132・D-137 の値を直接書く（設定から読んだ値どうしを比べない）。

use serde_json::Value;

fn conf() -> Value {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tauri.conf.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("tauri.conf.json を読めない"))
        .expect("tauri.conf.json が JSON でない")
}

#[test]
fn upgrade_code_is_pinned_d132() {
    assert_eq!(
        conf()["bundle"]["windows"]["wix"]["upgradeCode"],
        "9a951e09-cc62-5404-be40-7dfe6250fee3",
        "D-132 (2): bundle.windows.wix.upgradeCode は変えてはならない。変えると次の MSI は前の版を置き換えず、別のアプリとして並んで入る"
    );
}

#[test]
fn identifier_is_fixed_d132() {
    assert_eq!(
        conf()["identifier"],
        "com.inventory-system.desktop",
        "D-132 (3): identifier は変えてはならない。データの folder の名なので、変えた版は空の DB で起動する"
    );
}

#[test]
fn bundle_targets_d132() {
    assert_eq!(
        conf()["bundle"]["targets"],
        "msi",
        "D-132 (4): bundle.targets は文字列の \"msi\"（NSIS の setup.exe を作らない）"
    );
}

#[test]
fn product_name_and_window_title_d132() {
    let c = conf();
    assert_eq!(
        c["productName"], "在庫管理システム",
        "D-132 (1): productName"
    );
    assert_eq!(
        c["app"]["windows"][0]["title"], "在庫管理システム",
        "D-132 (1): 窓の題は productName と同じ名"
    );
    assert_eq!(
        c["bundle"]["windows"]["wix"]["language"], "ja-JP",
        "D-132 (1): bundle.windows.wix.language（日本語の製品名の MSI を作る）"
    );
}

fn cargo_toml() -> String {
    std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("Cargo.toml を読めない")
}

#[test]
fn bundled_binaries_are_main_only_d137() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(
        !dir.join("src/bin").exists(),
        "D-137 (1): src-tauri/src/bin/ を置いてはならない。tauri-cli は tauri の crate の src/bin/ の bin を bundle に入れ、MSI で店の PC へ配る（seed_demo_data.exe が作業フォルダーに DB を作り、22 §12 で取り込まれる経路）。開発用の bin は src-tauri/dev-tools/src/bin/ に置く"
    );
    assert!(
        dir.join("src/main.rs").is_file(),
        "D-137 (1): main の実行 file（src/main.rs）が無い"
    );
    assert!(
        !cargo_toml().lines().any(|l| l.trim() == "[[bin]]"),
        "D-137 (1): src-tauri/Cargo.toml に [[bin]] を書いてはならない。tauri の crate の bin は MSI に入って店の PC へ配られる。開発用の bin は src-tauri/dev-tools に置く"
    );
}

#[test]
fn default_members_and_default_run_d137() {
    let toml = cargo_toml();
    let default_members = toml
        .lines()
        .find(|l| l.trim_start().starts_with("default-members"))
        .unwrap_or("");
    assert!(
        default_members.contains("\"dev-tools\""),
        "D-137 (2): [workspace] の default-members に \"dev-tools\" が要る。外すと `cargo run --bin generate_bindings` ほかの開発用の bin の呼び出しが失敗し、`cargo test`・clippy が移した bin とその test を黙って見なくなる。今の行: {default_members:?}"
    );
    assert!(
        toml.lines()
            .any(|l| l.trim() == "default-run = \"inventory-system-tauri-scaffold\""),
        "D-137 (2): default-run = \"inventory-system-tauri-scaffold\" が要る。外すと default-members の 2 つの package の bin が候補になり、`npm run tauri dev` が打つ `--bin` の無い `cargo run` が止まる"
    );
}
