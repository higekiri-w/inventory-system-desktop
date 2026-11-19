//! D-159: tauri-build の Windows の resource（Common Controls v6 の manifest を含む）は
//! `rustc-link-arg-bins` で bin にだけ入り、lib の unittests・integration test の実行 file は
//! manifest が無いまま Windows で `STATUS_ENTRYPOINT_NOT_FOUND`（`TaskDialogIndirect`）で起動しない。
//! Windows の MSVC では tauri の既定の manifest を外し、`src-tauri/dev-tools/build.rs` と同じ 2 つの
//! link の引数を `rustc-link-arg` で出して、MSVC の linker に crate の全 link の対象へ manifest を
//! 埋めさせる（既定の manifest を残すと manifest が 2 つになり link できない）。
//! 2 つの build script の行の一致は `tests/bundle_identity_test.rs` の
//! `windows_manifest_link_args_d159` が確かめる。
//!
//! ponytail: Windows の GNU の target では lib の test の実行 file に manifest が入らないまま
//! （GNU の linker は `/MANIFEST…` を受けず、開発用の Windows と CI は MSVC）。
//! GNU で lib の test を回す必要が出たら `windres` の resource か `rustc-link-arg` の linker 別の形を足す。

use tauri_build::{Attributes, WindowsAttributes};

fn main() {
    // build script の `cfg!` は host を見るので、target は Cargo の環境変数で読む。
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let windows_msvc = target_os == "windows" && target_env == "msvc";

    let windows_attributes = if windows_msvc {
        WindowsAttributes::new_without_app_manifest()
    } else {
        WindowsAttributes::new()
    };
    if let Err(error) =
        tauri_build::try_build(Attributes::new().windows_attributes(windows_attributes))
    {
        println!("{error:#}");
        std::process::exit(1);
    }

    if windows_msvc {
        println!("cargo::rustc-link-arg=/MANIFEST:EMBED");
        println!(
            "cargo::rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
        );
    }
}
