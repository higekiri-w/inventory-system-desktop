//! D-132: MSI の製品名・upgrade code・`identifier`・`bundle.targets` を固定する。
//!
//! 期待値は D-132 の値を直接書く（設定から読んだ値どうしを比べない）。

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
