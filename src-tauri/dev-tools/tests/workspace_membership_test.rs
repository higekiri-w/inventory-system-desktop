//! D-137 (4): workspace の `default-members` に tauri の crate（`"."`）があることを dev-tools の側で固定する。
//!
//! `"."` が外れると `cd src-tauri && cargo test` は dev-tools だけを回し、tauri の crate の test
//! （D-137 の他の test を含む）は回らないので、この崩れは dev-tools の側の test でしか止められない。
//! 外部の crate を使わない（std だけ）。

#[test]
fn default_members_keep_tauri_crate_d137() {
    let toml = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../Cargo.toml"))
        .expect("src-tauri/Cargo.toml を読めない");
    let default_members = toml
        .lines()
        .find(|l| l.trim_start().starts_with("default-members"))
        .unwrap_or("");
    assert!(
        default_members.contains("\".\""),
        "D-137 (4): src-tauri/Cargo.toml の default-members に tauri の crate（\".\"）が要る。外すと `cargo test` が tauri の crate の test を回さず（exit 0 のまま黙る）、`cargo run`（`npm run tauri dev`）も main を起動しない。今の行: {default_members:?}"
    );
}
