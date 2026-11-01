//! D-137 の Compatibility（Gated Amendment 1・2）: 開発用の bin を Windows の MSVC で
//! tauri の crate の bin と同じ link の設定に揃える。他の target では何も出さない。
//!
//! (i) tauri の crate の bin は `src-tauri/build.rs` の `tauri_build::build()` から
//! Common Controls v6 の依存を持つ manifest を受けるが、workspace の member の dev-tools は
//! 受けない。manifest が無いと Windows で `generate_bindings` が
//! `STATUS_ENTRYPOINT_NOT_FOUND`（`TaskDialogIndirect`）で起動しない
//! （PR #201 の L3 の 1 回目、2026-10-29）。ここでは MSVC の linker に manifest を埋めさせ、
//! tauri-build の既定の manifest と同じ Common Controls v6 の依存を 1 つ足す。
//!
//! (ii) `tauri build` は `STATIC_VCRUNTIME=true` を立て、tauri-build は空の `msvcrt.lib` の
//! 検索の path（依存する側の link にも届く）と CRT を指し直す link の引数（tauri の crate の
//! target だけ）を出す。dev-tools の bin は空の `msvcrt.lib` だけを拾って C の runtime が欠け、
//! `LNK1120` で link できなかった（PR #201 の L3 の 2 回目、2026-11-01）。dev-tools は
//! 両方を自分で出して揃える（`STATIC_VCRUNTIME` は読まず、MSVC ならいつも static の CRT）。
//!
//! (iii) 出所と license: byte 列と 11 の引数は tauri-build 2.6.3 の `src/static_vcruntime.rs`
//! （`Copyright 2019-2024 Tauri Programme within The Commons Conservancy`、`Apache-2.0 OR MIT`。
//! 元は ChrisDenton/static_vcruntime〈`MIT OR Apache-2.0 OR Zlib`〉）の写しで、MIT の条件に従って
//! その copyright と license を下に書く（tauri-build 2.6.3 の crate の `LICENSE_MIT` の本文）。
//!
//! > MIT License
//! >
//! > Copyright (c) 2017 - Present Tauri Apps Contributors
//! >
//! > Permission is hereby granted, free of charge, to any person obtaining a copy
//! > of this software and associated documentation files (the "Software"), to deal
//! > in the Software without restriction, including without limitation the rights
//! > to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! > copies of the Software, and to permit persons to whom the Software is
//! > furnished to do so, subject to the following conditions:
//! >
//! > The above copyright notice and this permission notice shall be included in all
//! > copies or substantial portions of the Software.
//! >
//! > THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! > IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! > FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! > AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! > LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! > OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! > SOFTWARE.
//!
//! (iv) tauri-build の版を上げたら、その `src/static_vcruntime.rs` の byte 列・引数・順・条件と
//! 見比べる。
//!
//! ponytail: Windows の GNU の target（`x86_64-pc-windows-gnu`）には入れない
//! （GNU の linker は `/MANIFEST…` を受けず、開発用の Windows は MSVC だけ）。
//! GNU で dev-tools の bin を回す必要が出たら `windres` の resource を足す。
//! dev-tools の bin は `STATIC_VCRUNTIME=false` で `tauri build` したときも static の CRT で
//! link する（tauri の crate の bin は動的になるが、dev-tools の bin は MSI に入らないので害は無い）。
//! 揃える必要が出たら、空の `msvcrt.lib` と 11 の引数の両方を tauri-build と同じ条件にする。

use std::{env, fs, path::Path};

fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    // build script の `cfg!` は host を見るので、target は Cargo の環境変数で読む。
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_os == "windows" && target_env == "msvc" {
        println!("cargo::rustc-link-arg=/MANIFEST:EMBED");
        println!(
            "cargo::rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
        );
        static_vcruntime();
    }
}

/// tauri-build 2.6.3 の `static_vcruntime::build` と同じ（(iii) の写し）。
fn static_vcruntime() {
    override_msvcrt_lib();

    // Rust が決め打ちしない、ぶつかる library を外す。
    println!("cargo::rustc-link-arg=/NODEFAULTLIB:libvcruntimed.lib");
    println!("cargo::rustc-link-arg=/NODEFAULTLIB:vcruntime.lib");
    println!("cargo::rustc-link-arg=/NODEFAULTLIB:vcruntimed.lib");
    println!("cargo::rustc-link-arg=/NODEFAULTLIB:libcmtd.lib");
    println!("cargo::rustc-link-arg=/NODEFAULTLIB:msvcrt.lib");
    println!("cargo::rustc-link-arg=/NODEFAULTLIB:msvcrtd.lib");
    println!("cargo::rustc-link-arg=/NODEFAULTLIB:libucrt.lib");
    println!("cargo::rustc-link-arg=/NODEFAULTLIB:libucrtd.lib");
    // 使う library を指す。
    println!("cargo::rustc-link-arg=/DEFAULTLIB:libcmt.lib");
    println!("cargo::rustc-link-arg=/DEFAULTLIB:libvcruntime.lib");
    println!("cargo::rustc-link-arg=/DEFAULTLIB:ucrt.lib");
}

/// Rust が決め打ちする `msvcrt.lib` を、ほぼ空の object の file で差し替える。
fn override_msvcrt_lib() {
    let machine: &[u8] = match env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86_64") => &[0x64, 0x86],
        Ok("x86") => &[0x4C, 0x01],
        _ => return,
    };
    let bytes: &[u8] = &[
        1, 0, 94, 3, 96, 98, 60, 0, 0, 0, 1, 0, 0, 0, 0, 0, 132, 1, 46, 100, 114, 101, 99, 116,
        118, 101, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 60, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 10, 16, 0, 46, 100, 114, 101, 99, 116, 118, 101, 0, 0, 0, 0, 1, 0, 0, 0, 3, 0, 4, 0,
        0, 0,
    ];

    // 上流の `create_new` は写さない: `OUT_DIR` は build script の走り直しでも消えないので、
    // 上書きで書く。書けなければ build を止める。
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR は cargo が build script に渡す");
    let path = Path::new(&out_dir).join("msvcrt.lib");
    fs::write(path, [machine, bytes].concat()).expect("OUT_DIR の msvcrt.lib を書けない");
    println!("cargo::rustc-link-search=native={out_dir}");
}
