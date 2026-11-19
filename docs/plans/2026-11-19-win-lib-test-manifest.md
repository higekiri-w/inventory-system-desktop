# Plan Packet: Windows で lib の test の実行 file が起動しない件を直す

wave に属さない単独の lane。出所は owner の決定（repo 外の回答台帳 TD-800 の A、2026-11-19。急がず、MSI の L3 つきの R3）と [backlog](../backlog.md) の「Windows で lib の test の実行 file が起動しない件を直す」。Windows observe（D-157）の初回の run（PR #273）から `cargo test` が毎回 failure で出る原因。

decision-log の番号は仮に D-159 と書く（2026-11-19 の `origin/main` の最後は D-158〈`grep -on "^## D-[0-9]*" docs/decision-log.md | tail -1` → `1440:## D-158`〉）。Coordinator が並走の lane と突き合わせて割り当て、違えば計画の review の前に直す。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R3

Reason:
配る exe（MSI に入る main の実行 file）の link の形を変え、manifest の作り主が tauri-build から MSVC の linker に替わる。D-137 の先例（PR #201、dev-tools の bin の link の形）と同じく、Windows の実機でしか結果が見えない build の変更で、R2 の「gate と未知の path に触れない開発の道具」に当たらない。予定の path は classifier が `rust=true generated=true traceability=true docs=true workflow=true` を出す（`printf '%s\n' src-tauri/build.rs src-tauri/tests/bundle_identity_test.rs docs/ci.md docs/backlog.md docs/decision-log.md | bash scripts/ci/classify-changes.sh --files-from-stdin`、2026-11-19、exit 0）。

## Goal

Goal Invariant:

### 最小完了条件

- Windows の MSVC で、tauri の crate の lib の unittests・integration test・main の bin の test の実行 file が、Common Controls v6 の依存を持つ manifest つきで link され、`0xc0000139 STATUS_ENTRYPOINT_NOT_FOUND`（`TaskDialogIndirect`）で止まらずに起動する。lane の PR の Windows observe の run で `cargo test` の lib の unittests が走る（`test result:` の行が出る）。
- 配る exe は今と同じく Common Controls v6 の依存を持つ manifest を 1 つだけ持ち、MSI で入れて起動できる（L3）。
- Windows の MSVC 以外の target（Linux の CI と開発、Windows の GNU）の build は今と同じ。
- backlog の項目と `docs/ci.md` の Windows Observe の文が、直った後の事実を書く。

### 失敗定義

- lib の test の実行 file が Windows で起動しない（observe の run の log に lib の unittests の `test result:` が無い、または `0xc0000139`）。
- 配る exe が起動しない、manifest が 2 つになる（link の失敗）か Common Controls v6 の依存が無い、起動で UAC の確認が出る。
- Linux の build・test の結果が変わる。
- 新しく Windows で走った test が落ちたのを、skip・retry・弱めることで消す。

### 非目的

- 新しく Windows で走った test の失敗の修正（名と原因の分類を PR の comment と backlog に残すまで。直しは別の lane）。
- `.github/workflows/windows-observe.yml` の変更（`--no-fail-fast` を含む）と enforce への昇格（D-157 の別の R3）。
- Windows の GNU の target で lib の test を起動させること。
- 依存の追加・更新（`Cargo.toml`・`Cargo.lock` は変えない）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 確かめた事実（2026-11-19、手元の cargo registry の source と repo の現物）

- 原因: tauri-build 2.6.3（`src-tauri/Cargo.lock`）は Windows の resource（icon・版の情報・既定の manifest）を `tauri-winres` 0.3.5 に渡し（`tauri-build-2.6.3/src/lib.rs:622-676`、manifest は `:624-626` の `set_manifest`）、`embed-resource` 3.0.8（`Cargo.lock` の版。手元には 3.0.11 もあるが lock は 3.0.8）が bin のある crate で `cargo:rustc-link-arg-bins=` を出す（`embed-resource-3.0.8/src/lib.rs:443-444`）。lib の unittests と `tests/` の integration test の実行 file はこれを受けない。
- 既定の manifest（`tauri-build-2.6.3/src/windows-app-manifest.xml`）は Common Controls v6 の `dependentAssembly` の 1 つだけで、UAC・DPI の要素を持たない。
- `TaskDialogIndirect` を import するのは `tauri-plugin-dialog` 2.7.1 が有効にする `rfd` 0.16.0 の `common-controls-v6` の feature（`tauri-plugin-dialog-2.7.1/Cargo.toml:106`、`rfd-0.16.0/src/backend/win_cid/message_dialog.rs:76`・`:167`）。v6 の manifest が無い実行 file は loader が起動の前に止める。実測は lib の unittests の 1 本（PR #273 の run、下の「今 Windows で走っていない test」）。同じ lib を link する integration test と main の bin の unittests も止まる見込み（推定。fail-fast で走っておらず、AC1 で見る）。
- 外す手段: `WindowsAttributes::new_without_app_manifest()`（`tauri-build-2.6.3/src/lib.rs:267`）は `app_manifest: None` で、`set_manifest` を呼ばない（icon・版の情報は今のまま）。`Attributes::new().windows_attributes(..)`（`:365`・`:371`）と `try_build(attributes) -> Result<()>`（`:469`）は public。`build()`（`:453`）は `try_build(Attributes::default())` の失敗を出力して `exit(1)` する。
- `STATIC_VCRUNTIME` の扱い（`:708-709`）は `try_build` の中にあり、tauri の crate では今のまま tauri-build が持つ。dev-tools の `static_vcruntime()` の写しは要らない（D-137 の Gated Amendment 2 は tauri-build が tauri の crate の target にだけ link の引数を出すことの是正で、tauri の crate 自身には当たらない）。
- 先例: `src-tauri/dev-tools/build.rs` は `CARGO_CFG_TARGET_OS == "windows"` かつ `CARGO_CFG_TARGET_ENV == "msvc"` のときだけ `cargo::rustc-link-arg=/MANIFEST:EMBED` と `/MANIFESTDEPENDENCY:…Common-Controls…` を出し、PR #201 の L3 の 2・3 回目で dev-tools の bin が Windows で起動した（D-137 の Compatibility）。`rustc-link-arg` は Cargo の文書で「benchmarks, binaries, `cdylib` crates, examples, and tests」に渡る（PR #201 の Gated Amendment 1 が 2026-10-29 に読んだ）。lib の unittests は「tests」に当たり、`-bins`・`-tests` では届かない（同 amendment の Cargo の source の `LinkArgTarget::applies_to` の読み）。
- `src-tauri/Cargo.toml` の `crate-type = ["staticlib", "cdylib", "rlib"]`: `rustc-link-arg` は cdylib の DLL にも渡り、`target/` の下の DLL にも manifest が入る（DLL の既定の ID は 2。MSVC の `/MANIFEST` の文書）。DLL は MSI に入らない（D-137 の Guarantee range: MSI の実行 file は main の 1 つだけ）。害は無いと読む（未実測）。
- UAC: MSVC の `/MANIFEST` の文書（PR #201 の Gated Amendment 1 が引いた <https://learn.microsoft.com/en-us/cpp/build/reference/manifest-create-side-by-side-assembly-manifest>）は、`/MANIFESTUAC` を指定しないと `level='asInvoker'` の `trustInfo` の断片が入ると書く。配る exe の manifest は tauri の既定と違い、`trustInfo`（`asInvoker`、`uiAccess='false'`）が増える見込み。起動の権限は今と同じ（呼んだ人の権限、昇格の確認なし）。配る exe での実物は `未実測`（L3-3 で見る）。
- 二重の manifest: tauri の既定の manifest の resource（ID 1）を残したまま linker に `/MANIFEST:EMBED` で ID 1 の manifest を作らせると、同じ resource が 2 つになり link が失敗する見込み（推定、未実測。そのため `new_without_app_manifest` と組にする）。
- 今 Windows で走っていない test: `cargo test` は既定で最初に落ちた実行 file で止まる（`--no-fail-fast` の無い既定）。PR #273 の run（<https://github.com/higekiri-w/inventory-system-desktop/actions/runs/44076654226>）の log では、dev-tools の bin の unittests 3 本（`generate_bindings`・`generate_traceability`・`seed_demo_data`）と `tests\workspace_membership_test.rs` が走って `test result: ok` を出した後、tauri の crate の lib の unittests（`Running unittests src\lib.rs`）が `exit code: 0xc0000139, STATUS_ENTRYPOINT_NOT_FOUND` で止まった。その後ろの tauri の crate の main の bin の unittests（`src\main.rs`）と integration test（`src-tauri/tests/` の 6 file）は走っていない（2026-11-19 に `gh run view 44076654226 --log` の read で確かめた。出所は round 1 の Codex の報告の問い 5）。
- doctest は無い（`grep -rn '^\s*//[/!] *```' src-tauri/src` が 0 件）。
- 画面の native の dialog: frontend が使う `@tauri-apps/plugin-dialog` は `open`（`src/components/FilePicker.tsx` ほか）と `save`（`src/features/plu-export/PluExportPage.tsx`）だけで、`ask`・`confirm`・`message` は無い（`grep -rn "plugin-dialog" src`）。ファイルを選ぶ窓は v6 の依存が無くても開き区別できないので、L3 で見ない。閉じるときの確認の窓（`src/components/layout/WindowCloseDialog.tsx`、D-153）は HTML の窓で、Common Controls に依らない。Rust の `MessageBoxW`（`src-tauri/src/lib.rs` の `show_pre_window_fatal`）は起動の失敗のときだけで、故障の注入なしには出せない。
- Linux への影響: 分岐は Windows の MSVC の guard の中だけで、Linux では `try_build(Attributes::new()…)` が今の `build()` と同じ設定（`WindowsAttributes` は Windows の resource の作成〈`target_triple.contains("windows")` の中〉でしか読まれない）。

## Scope

- S1 `src-tauri/build.rs`: `tauri_build::build()` を次の形に替える（全体で 20 行ほど）。
  - `CARGO_CFG_TARGET_OS`・`CARGO_CFG_TARGET_ENV` を `target_os`・`target_env` に読み（build script の `cfg!` は host を見るため。dev-tools と同じ）、guard を 1 つの `let`（`let windows_msvc = target_os == "windows" && target_env == "msvc";`）で求めて、manifest の選択と下の 2 行の引数の両方にこの名だけを使う（`"msvc"` の literal はこの行の 1 か所）。Windows の MSVC のときだけ `WindowsAttributes::new_without_app_manifest()`、それ以外は `WindowsAttributes::new()`（Windows の GNU は tauri の既定の manifest のまま。GNU の linker は `/MANIFEST…` を受けない）を `tauri_build::Attributes::new().windows_attributes(..)` に渡して `tauri_build::try_build` を呼ぶ。
  - 失敗は握りつぶさない: `try_build` の `Err` は `{error:#}` を出して build を止める（`build()` と同じく exit 1 か panic）。`build()` が足す「`cargo update` を試す」の案内の 1 文は写さない（版の食い違いの案内で、error の本文は残る）。
  - Windows の MSVC のときだけ、dev-tools と同じ 2 行（`cargo::rustc-link-arg=/MANIFEST:EMBED` と `cargo::rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'`）を出す。`rustc-link-arg-bins`・`-tests` にしない（lib の unittests に届かない）。
  - `/MANIFESTUAC` は指定しない（linker の既定の `asInvoker`。dev-tools の bin と同じ。理由は D-159 の Alternatives）。
  - `static_vcruntime` は写さない（確かめた事実）。`cargo::rerun-if-changed=build.rs` は書かない: tauri-build が設定・icon・capabilities の `rerun-if-changed` を既に出し、build.rs 自身の変更は Cargo が build script を作り直して走らせる。
  - doc comment は理由を短く書く（D-159 を引く）。`ponytail:` comment: Windows の GNU では lib の test の実行 file に manifest が入らないまま（開発用の Windows と CI は MSVC）。GNU で lib の test を回す必要が出たら `windres` の resource か `rustc-link-arg` の linker 別の形を足す。
- S2 `src-tauri/tests/bundle_identity_test.rs`: test `windows_manifest_link_args_d159` を 1 つ足す（既存の `cargo_toml()` と同じく file を文として読む形）。どの検査も、前の空白を除いて `//` で始まる行（comment）を除いた行だけを見る。(a) `src-tauri/build.rs` の行に `new_without_app_manifest()` と `try_build` がある。(b) 2 つの build script のそれぞれで、行から `"cargo::rustc-link-arg=/MANIFEST` で始まり次の `"` で終わる文字列 literal を抜き、その集合が 2 つの build script で等しく、要素が 2 つ（`/MANIFEST:EMBED` と `/MANIFESTDEPENDENCY:…`。`-bins`・`-tests` の literal はこの接頭辞に合わないので集合が欠けて red）。(c) `src-tauri/build.rs` に `let windows_msvc = target_os == "windows" && target_env == "msvc";` の行があり、`src-tauri/build.rs` の `"msvc"` はその 1 か所だけ。message は崩れたときに Windows で起きること（lib の test が起動しない・二重の manifest で link できない）を名指す。
- S3 `src-tauri/dev-tools/build.rs` の doc comment (i) の 1 文「tauri の crate の bin は `src-tauri/build.rs` の `tauri_build::build()` から Common Controls v6 の依存を持つ manifest を受けるが、workspace の member の dev-tools は受けない」を、今の事実（tauri の crate も同じ 2 行を linker に渡す。D-159、行の一致は S2 の test が確かめる）に置き換える。code は変えない。
- S4 `docs/decision-log.md`: 末尾に D-159 を足す（下の Design Readiness の要旨）。D-137 の Compatibility の「dev-tools の bin は、tauri の crate の build script が入れる Windows の resource（…Common Controls v6 を求める manifest）を受けない」の文の manifest の部分に、D-159 で tauri の crate の manifest も linker が作るようになったことを 1 句で添える（D-137 の決定は変えない）。
- S5 `docs/ci.md` の `## Windows Observe` の「初回の結果と時間」の行: 括弧の中の backlog の項目の名（今の項目の名と既に食い違う「…と、Windows でだけ出る clippy の 3 件を直す」）と「別の lane で直すまで、どちらも毎回 failure で出る」を、直した後の事実に置き換える: clippy の 3 件は PR #281、lib の test の起動は D-159 で直した。lane の PR の run の `cargo test` の outcome と、新しく走って落ちた test があればその backlog の項目を引く。「昇格（enforce）の条件」の行の文は変えない（「見つかった Windows の失敗が直っていること」は、この lane の後は S6 の新しい項目〈あれば〉を指す）。
- S6 `docs/backlog.md`: 「Windows で lib の test の実行 file が起動しない件を直す」の項目を外し、その上の PR #188 の申し送りの末尾「tauri の crate の `cargo test --lib` の test の実行 file の件は残る（…）」を「→ 済み（日付、PR、D-159）」に書き換える。lane の PR の run で新しく走って落ちた test があれば、原因ごとに 1 つの項目を足す（名・原因の分類〈Windows の挙動・test の前提・flaky の疑い〉・出所の run の URL）。下の「L3 の自動化の E-1」の項目の「上の Windows の lib の test の直しの後」は、外した項目を指さない文（「D-159 の後」）に直す。

Non-scope: `.github/workflows/**`、`src-tauri/Cargo.toml`、`src-tauri/Cargo.lock`、`src-tauri/tauri.conf.json`、`src-tauri/dev-tools/build.rs` の code、`docs/msi-distribution-runbook.md`（今の事実の表に manifest の行は無く、書いた事実は変わらない）、`Plans.md`（関門ではない）。

実装の順（`.github/**` に触れないので push は Coordinator ができる）:

1. 計画の固定の後、S1〜S4 の commit を push する。
2. その head の lane の PR の Windows observe の run を AC1・AC2 で読む。
3. その結果で S5・S6 の commit を push する（`docs/ci.md` は classifier が `workflow=true` を出すので、L3 の後に積むと merge-evidence の L3 の使い回しに当たらない）。
4. S5・S6 を含む head で L3 を 1 回だけ行う。

## Acceptance Criteria

- AC1: 実装の順の 2 の run の `cargo test` の log で、Coordinator が tauri の crate の test の実行 file 8 本（lib の unittests `src\lib.rs`、main の bin の unittests `src\main.rs`、`tests\` の integration test 6 本）の 1 本ずつを次の 3 つに分け、run の URL とともに PR の comment に残す: 走った（`Running` の行の後、次の `Running` の行より前に `test result:` の行がある。`running 0 tests` と `test result: FAILED` も走ったに数える）/ 前の失敗で走っていない（`Running` の行が無い）/ 異常終了（`Running` の行の後に `test result:` の行が無いまま止まった。`0xc0000139` はここ）。合格: 異常終了が 0 本で、lib の unittests が走った。走っていないものが 1 本以上残ったときは L3 の用意 4 の log で同じく分け、8 本が全部走ったになったときに閉じる（Goal の最小完了条件の 1 つ目を閉じる）。
- AC2: AC1 の run（と L3 の用意 4 の log）の `cargo test` の log に `FAILED` の行があれば、名と原因の分類を PR の comment に残し、S6 の backlog の項目にする。skip・retry・assert の弱めで消さない。落ちた test の有無は merge を止めない（observe）が、Coordinator は Ready の承認を求める返答に書く（D-157 の (3)）。Windows の店の PC での業務に当たる失敗（backup・復元・SD への書込みの Windows の分岐）なら、別の lane の優先を owner に諮る。
- AC3: Linux で `cd src-tauri && cargo build` と `cargo test` と `cargo clippy --all-targets --all-features -- -D warnings` と `cargo fmt --check` が PASS（hosted の `Merge gate` と同じ）。`git diff origin/main -- src-tauri/Cargo.toml src-tauri/Cargo.lock` が空。
- AC4: `cd src-tauri && cargo test --test bundle_identity_test windows_manifest_link_args_d159` が PASS し、Test Plan の mutation で red になる（Writer が手元で 1 つずつ入れて戻し、結果を PR の comment に残す）。
- AC5: `try_build` の失敗が build script で build を止める: Linux の `src-tauri` で `TAURI_CONFIG='{"identifier": 1}' cargo build > "$TMPDIR/ac5.log" 2>&1; echo exit=$?`（型の合わない設定）が非 0 で、``grep -cF 'failed to run custom build command for `inventory-system-tauri-scaffold' "$TMPDIR/ac5.log"`` が `1`（round 1 の Opus の reviewer の実測: 正しい実装で 1 件、mutation `let _ = try_build(..)` で 0 件。mutation では `src/lib.rs` の `tauri::generate_context!()` の compile error で非 0 になるので、exit だけでは区別できない）。Writer が正しい実装と mutation の両方で流し、exit と件数を PR の comment に残す。設定の値で失敗しないときは別の壊れた設定を探し、使った値を書く。
- AC6: `cargo check --release`（`## L3` 節のある lane の Writer の完了条件）。
- AC7: `## L3` の L3-1・L3-3 が PASS（merge の条件）。
- AC8: `bash scripts/doc-consistency-check.sh` と `--target plan` が ERROR なし、`bash scripts/check-workflow-git.sh` が OK。`rg -n "^## D-159" docs/decision-log.md` が 1 件。`rg -n "lib の test の実行 file が起動しない件を直す" docs` が本 packet だけ。

## Design Readiness

- 引用する設計正本: decision-log D-137（Compatibility、Gated Amendment 1・2 の manifest と static の CRT）、D-157（Windows observe、`docs/ci.md` Windows Observe）、D-147（最小の実装）、`docs/DEV_WORKFLOW.md` Implementation Rules（`cargo check --release`）、`docs/msi-distribution-runbook.md` 流れ 1・2（MSI の作り方と入れ方）。
- 必要な設計成果物: D-159 を本 PR で足す（計画の固定の後、S4）。function-design・DB・画面の設計は変わらない（command・DTO・画面・データに触れない）。
- D-159 の要旨（S4 で書く）: Status: owner の決定 TD-800 の A、本 packet で起票。Context: tauri-build の resource は `rustc-link-arg-bins` で bin にだけ入り、v6 の manifest の無い lib の unittests の実行 file は Windows observe の run（PR #273）で `0xc0000139`（`TaskDialogIndirect`）で起動しなかった（実測はこの 1 本。integration test・main の bin の unittests はその run で走っていない）。Decision: Windows の MSVC で tauri の既定の manifest を外し、dev-tools と同じ 2 つの link の引数を `rustc-link-arg` で tauri の crate の link の対象（cdylib の DLL、lib の unittests、integration test、main の bin とその unittests）に渡す。他の target は今のまま。行の一致は S2 の test。Why: 1 つの手段で配る exe・test の実行 file・dev-tools の bin が同じ manifest になり、新しい crate・file が要らない。Alternatives: (a) tauri の既定の manifest を残して test の側にだけ足す（`rustc-link-arg-tests` は lib の unittests に届かない。`rustc-link-arg` は bin にも渡り二重の manifest になる）; (b) `/MANIFESTUAC:NO` で tauri の既定と同じ中身にする（引数が 1 つ増え、dev-tools の bin と食い違う。`asInvoker` は起動の権限を変えず、Microsoft が推す形）; (c) manifest の file を共有して `include_str!`・`include!` する（package をまたぐ path と `rerun-if-changed` が要り、行の一致は S2 の test で足りる）; (d) Windows で lib の test を回さない（D-157 の observe の目的に反し、Windows だけの code の test が走らない）; (e) `embed-resource` を build-dependency に足して `.rc` を作る（依存と file が増える）。Guarantee range: Windows の MSVC の target。GNU は今のまま lib の test が起動しない。`target/` の cdylib の DLL にも manifest が入る（配らない）。Compatibility: 配る exe の manifest に `trustInfo`（`asInvoker`）が増える（L3-3 で確かめる）。icon・版の情報・`STATIC_VCRUNTIME` は変えない。D-137 の dev-tools の 2 行と同じ。Revisit: tauri-build を上げるとき（既定の manifest の中身と `new_without_app_manifest` を見比べる）、配る exe の manifest に DPI の要素を足すとき（そのとき manifest の file を持つ形を比べる）。
- plan にしかない durable な判断の昇格先: 上の D-159。
- 前提・制約と延期した gap: 新しく Windows で走った test の失敗の修正は延期（S6 の backlog）。`cargo test` が最初に落ちた実行 file で止まるので、落ちた実行 file の後ろは observe の run では走らないまま残りうる。そのときは L3 の用意 4 で 1 回だけ `--no-fail-fast` で起動を見る（D-157 の observe の command は変えない）。二重の manifest の link の失敗と DLL の manifest の害の無さは推定。
- 絶対保証の例外と抜け道: 主張しない。Linux の S2 の test は文の形だけを見て、Windows の link の結果は見ない（AC1 と L3 が持つ）。L3-3 は exe の byte 列を UTF-8 で読んで `<assembly>…</assembly>` を数えるだけで、`RT_MANIFEST` の resource を ID・言語まで列挙しない（列挙は owner に Win32 の P/Invoke を貼らせることになる。起きうる二重〈tauri の resource と linker の manifest〉はどちらも UTF-8 の XML なので今の数え方で 2 になり、Coordinator が判定する）。
- 判定: ready（計画の review の前）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| D-159（tauri の crate の manifest を linker が作る） | decision-log D-159（新規） | S1・S3・S4 | `windows_manifest_link_args_d159` と mutation、Windows observe の `cargo test`（AC1、hosted の観測） | L3-1・L3-3（走っていない実行 file が残れば用意 4） |
| D-137（dev-tools の manifest と static の CRT） | decision-log D-137 の Compatibility | S3（comment）・S4（1 句） | 既存の `bundled_binaries_are_main_only_d137`・`default_members_and_default_run_d137`、S2 の行の一致 | 非対象（dev-tools の code は変えない） |
| D-157（Windows observe） | `docs/ci.md` Windows Observe | S5 | 変えない（workflow に触れない） | AC1・AC2 は lane の PR で観測 |
| D-132・runbook の MSI の形 | decision-log D-132、`docs/msi-distribution-runbook.md` | 変えない | 既存の `bundle_identity_test` の D-132 の test | L3-1（入れて起動） |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| lib の test の実行 file に manifest が届く | link の引数が `rustc-link-arg-bins`・`-tests` になり、lib の unittests に届かない | Rust（文の検査、mutation） | `windows_manifest_link_args_d159` | `src-tauri/build.rs` の行を `cargo::rustc-link-arg-bins=` にする |
| manifest が 1 つ | MSVC で `WindowsAttributes::new()` のままにして tauri の resource と linker の manifest が二重になる | Rust（mutation） | 同 test の (a) | `new_without_app_manifest()` を `new()` にする（名を comment に残しても red） |
| dev-tools と同じ依存 | 2 つの build script の `/MANIFESTDEPENDENCY` の文がずれる | Rust（mutation） | 同 test の (b) | どちらかの `version='6.0.0.0'` か `publicKeyToken` を 1 文字変える |
| MSVC だけに出す | guard が広がり Windows の GNU の linker に `/MANIFEST…` が渡る、または guard が消え Linux の linker にも渡る | Rust（mutation）と CLI | 同 test の (c)、AC3 の Linux の `cargo build` | guard の `&&` を `\|\|` にする（(c) が red。Linux は引数が出ず通るので CLI では見えない）。`&& target_env == "msvc"` を消す（(c) が red）。guard の `if` を外して 2 行を無条件に出す（Linux の link が `rust-lld: error: cannot open /MANIFEST:EMBED` で落ち、AC3 が red。PR #201 の probe と同じ） |
| build の失敗を握りつぶさない | `try_build` の `Err` を捨て、壊れた設定で build script が通る | CLI（negative の probe） | AC5 | `let _ = try_build(..)` にする（`failed to run custom build command for` の行が 0 件になる） |
| Linux は今と同じ | Linux の build・test・clippy が変わる | CLI | AC3（hosted の `Merge gate`） | 分岐の外で何かを出す |
| Windows で tauri の crate の test の実行 file が起動する | 起動の前に loader が止める | 手の観測（Coordinator、PR の comment） | AC1（Windows observe の run の log、残りは L3 の用意 4 の log） | 8 本のどれかが異常終了（`Running` の後に `test result:` が無い。`0xc0000139`） |
| 新しく走った test の失敗を隠さない | 落ちた test を skip・retry・弱めで消す | review と手の観測 | AC2 | 落ちた test の名が comment と backlog に無い |
| 配る exe が起動し manifest が 1 つ | MSI のアプリが起動しない、manifest が無い・2 つ | L3（Windows の実機） | L3-1・L3-3 | 下の表の合格基準を満たさない |

## L3

L3 Eligibility: (1) 配る exe の link と起動、MSI で入れた exe の manifest は Windows の実機でしか観測できない（CI は MSI を build しない。D-157 の Windows observe は `tauri build` を回さない）。(2) 新しい道具は要らない（手順書の流れ 1・2 と PowerShell）。(3) 故障の注入は無い（起動の失敗のときの `MessageBoxW` は対象にしない）。

到達経路と用意（Coordinator が「実装の順」の 3 で S5・S6 の commit を push した後の head で 1 回だけ頼み、owner に貼り付ける command を返答の本文に inline で渡す。Claude Code の sandbox から Windows の PowerShell を呼べないので、build と導入は owner が貼る）:

1. 開発用の Windows の clone（[手順書](../msi-distribution-runbook.md)の流れ 1）で、最新の `origin/main` を取り込んだ PR の head を checkout し、`git status --porcelain` が空であることを確かめ、`npm ci --ignore-scripts` の後、既にある `src-tauri\target\release\bundle\` を repo の外の日付の付いた folder へ移す（前の build の MSI で合否を決めない）。`npm run tauri build` がエラーで止まらず MSI ができることを確かめ、MSI の SHA-256 を控える。止まったら先へ進まず、build の log を控えて Coordinator に戻す（二重の manifest の推定が外れた場合を含む）。
2. 入れる前と後は、[手順書](../msi-distribution-runbook.md#確かめる項目)の「予行演習の準備と片付け」の手順に従う（MSI のアプリと開発中のアプリは同じデータの folder を使うため。手順は写さず、その節を引く）。版 A は用意 1 の MSI（PR の head）と読み、版 B は無い。準備: 「始める前に」でアプリを全部止め、2 つの folder を控えて退避する。用意 1 の MSI を流れ 2 のとおり入れ（前の版の上に入れてよい。M4）、最初の起動（L3-1）の前に作業フォルダーに `inventory.db` が無いことを確かめる。試験のデータは L3-1 の最初の起動が作る空の DB で足りる（L3-1 はホームの画面を見るだけ）。片付け: L3-1・L3-3 が終わったら（FAIL で止めたときも）、同じ節の「終わった後」の (1)〜(5) のとおり戻す。
3. 依存する既知の backlog: 無い。fixture は要らない。
4. 条件つき（AC1 で「前の失敗で走っていない」実行 file が 1 本以上残ったときだけ）: 用意 1 の clone の同じ head で、PowerShell に下の「用意 4」の code block を貼り、出来た log の file を Coordinator に戻す。走っていない実行 file が残っていなければ行わない。

用意 4（clone の root で貼る。exit が 0 でなくても、落ちた test は AC2 で扱うので log を戻す）:

```powershell
Set-Location src-tauri
cmd /c "cargo test --no-fail-fast > %TEMP%\pr286-no-fail-fast.log 2>&1"
"exit=$LASTEXITCODE log=$env:TEMP\pr286-no-fail-fast.log"
```

L3-3（PowerShell に貼る）:

```powershell
$f = @(Get-ChildItem "C:\Program Files\在庫管理システム" -File -Filter *.exe)
if ($f.Count -ne 1) { throw "exe count: $($f.Count)" }
$t = [Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes($f[0].FullName))
$m = [regex]::Matches($t, '(?s)<assembly.*?</assembly>')
"manifests=$($m.Count)"
$m | ForEach-Object Value
```

| ID | 画面・到達手順 | 観測できる合格基準 | merge の条件 |
|---|---|---|---|
| L3-1 | スタートメニューの「在庫管理システム」から起動する | UAC の確認が出ずに、アプリの窓が開いてホームの画面が出る。「エントリ ポイントが見つかりません」などの Windows のエラーの窓が出ない。見終わったらアプリを閉じる | はい |
| L3-3 | 上の「L3-3」の code block を PowerShell に貼る | `manifests=1` で、出た文に `Microsoft.Windows.Common-Controls` と `6.0.0.0` があり、`requestedExecutionLevel` の要素の `level` の値が `asInvoker`、`uiAccess` の値が `false`（値を囲む引用符は `'` でも `"` でもよい）。出た文の全体を控えて Coordinator に戻す | はい。`exe count:` の error が出た、`manifests=0`、Common-Controls が無い、`requestedExecutionLevel` が無い、`level` が `asInvoker` でない、`uiAccess` が `false` でない、のどれかなら FAIL（tauri の既定の manifest のままの旧い exe はここで FAIL）。`manifests` が 2 以上なら、出た文を Coordinator が読んで manifest かを判定する |

止め方: L3-1 が FAIL なら L3-3 を行わず、出た窓の文を控えて Coordinator に戻し、用意 2 の片付けをする。
