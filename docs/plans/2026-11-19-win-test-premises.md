# Plan Packet: Windows で落ちる lib の test 53 本の前提を直す

wave に属さない単独の lane。出所は owner の決定（repo 外の回答台帳 TD-825 の A、2026-11-19。PR #286 の merge の後、関門の作業の合間に）と [backlog](../backlog.md) の「Windows で落ちる lib の test の前提を直す lane」（群 A〜D）。事実は PR #286 の Windows observe の run <https://github.com/higekiri-w/inventory-system-desktop/actions/runs/44504745664>（lib の unittests 1559 passed・53 failed・4 ignored）と、owner の Windows の `cargo test --no-fail-fast`（同じ 53 本、PR #286 の comment <https://github.com/higekiri-w/inventory-system-desktop/pull/286#issuecomment-7008849050>）。PR #286 の Final の P2（B12 の原因が確かめられていないのに分類を閉じて読める）と P3（`docs/backlog.md:55` の行）をこの lane で直す（Coordinator の裁定）。

decision-log の番号は仮に D-160 と書く（`grep -on "^## D-[0-9]*" docs/decision-log.md | tail -1` → `1452:## D-159`、2026-11-19）。Coordinator が並走の lane と突き合わせて割り当てる。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R3

Reason:
`.gitattributes` を足して全 file の Windows の checkout の改行を変え（classifier は unknown の path として全部の分類を出す: `printf '%s\n' .gitattributes | bash scripts/ci/classify-changes.sh --files-from-stdin` → `rust=true rust_drift=true frontend=true docs=true env=true generated=true traceability=true workflow=true unknown=true`、2026-11-19）、PC の外の控えの用意で媒体の root を決める関数（`src-tauri/src/mnt/offsite.rs`）の Windows の分岐を Linux と共通の 1 つにする。R4 に当てない理由: 消す file の組・上書きしない公開・復元の経路を変えず、root の判定は本番で今と同じ結果を返す（Contract Ledger）。データの安全の境界に触れるので R2 にしない。R4 に当たるかを計画の review に問う。

## Goal

Goal Invariant:

### 最小完了条件

- lane の PR の最後の head の Windows observe の run で、tauri の crate の 8 本の test の実行 file が全部 `test result: ok`（53 本が通り、新しく落ちる test が無い）。
- 群ごとに、製品の code が Windows で契約どおりに振る舞う根拠が PR の comment にある。B12 は Windows で実際に返る error（variant・失敗した操作・OS の error）を run の log で確かめた結果で直し方を決める。
- Linux の test・clippy・fmt は今のまま通り、直した test が Linux で今の契約を弱めていない（Test Plan の mutation）。
- `docs/ci.md` と `docs/backlog.md` の Windows の 53 本の文が確かめた事実を書く。

### 失敗定義

- 53 本のどれかを skip・`#[cfg(not(windows))]`・期待の弱めで消す（その test の契約が Windows に当たらないと正本で言える場合を除き、根拠を PR に書く）。
- tracked の fixture（CP932・CRLF の合成 CSV）の bytes が変わる。
- Windows の本番で、取外し可能でない drive の folder を用意できる、または今用意できる USB メモリを用意できなくなる。
- B12 を推定のまま閉じる。

### 非目的

- Windows observe の昇格（enforce）、`.github/workflows/**` と `scripts/**` の変更（classifier に `.gitattributes` を足すことを含む）、製品の振舞いの変更、Windows observe の report の検査の深さの残り 3 つ（backlog の別の項目）、4 つの `i/mixed` の file の改行の正規化（`git add --renormalize` は流さない）。

## 確かめた事実（2026-11-19、`91fec088` と run の log）

- log の落ちた名: `grep -E ' test .* \.\.\. FAILED$' <log> | grep -v warning | awk '{print $3}' | sort -u | wc -l` → `53`（`mnt::offsite` 39、`cmd::settings_cmd` 2、`mnt::backup` 3、`mnt::restore` 2、`io::register_sd` 2、`io::register_sd_move` 1、`biz::sales_service` 1、`bindings_generation_tests` 3）。log は `gh api repos/higekiri-w/inventory-system-desktop/actions/jobs/135493302020/logs`。
- A（7 本）: どれも自分の source を `include_str!`・`read_to_string` で読み `\n` を含む文字列で探す（`src-tauri/src/lib.rs:781-784`・`:812`・`:910`、`src-tauri/src/biz/sales_service.rs:3103`、`src-tauri/src/io/register_sd.rs:1468-1472`、`src-tauri/src/io/register_sd_move.rs:1038-1042`、`src-tauri/src/mnt/restore.rs:2178`）。repo に `.gitattributes` が無い。`git ls-files --eol` の index の改行: `i/lf` 1561・`i/crlf` 29・`i/mixed` 4・`i/-text` 22・`i/none` 5・空 18。`i/crlf` の 29 は全部 `tests/fixtures/{daily-report,daily-report-sheet,daily-report-z001,z004}/` の合成の CSV で、CP932・CRLF であることが契約（各 README、`tests/fixtures/z004/README.md:155`。`daily_report_parser.rs:837`・`sales_service.rs:1845` の test が読む）。`i/mixed` は `.agents/skills/inventory-code-review/references/{review-checklist,source-map}.md`・`tasks/{lessons,todo}.md`（test に関係しない）。
- A の probe（Linux で `core.autocrlf=true` の clone を Windows の代わりにした。実の Windows は未実測）: 属性なしは `src-tauri/src/lib.rs` が `w/crlf`。S1 の 2 行を commit した版の clone は `*.rs`・`*.sh` が `w/lf`、29 の fixture が `i/crlf w/crlf attr/-text`、`git status` が空。属性の前に作った clone は pull の後も `w/crlf` のまま（status は空）で、`git read-tree HEAD` と `git checkout-index -f -a` で `w/lf` になった。
- B（41 本）: `prepare_with`（`offsite.rs:344-360`）は `drive_root(selected, &env.roots)` の root が `env.roots` に `same_root` で在るかで `NotRemovable` を決める。Windows の `drive_root`（`:983-995`）は注入の `roots` を使わず prefix と root の component を返すので、test が注入した `\\?\C:\…\E`（`Fx::new` の `canonicalize`、`:1368`。CMD の test は `settings_cmd.rs:991-995`）が `\\?\C:\` になり、全部 `NotRemovable`。Windows 以外の `drive_root`（`:997-1005`）は注入の列のうち選んだ path を含む最も長いもの。本番の列は `removable_drive_roots()`（`register_sd.rs:158-185`、`GetDriveTypeW` が `DRIVE_REMOVABLE` の `X:\`）。71 §71.11.3（`docs/function-design/71-mnt-backup.md:673`）は「production と test は同じ内部関数を通し、test だけの判定を作らない」。`same_components`・`component_eq`（`offsite.rs:960-980`、Windows は ASCII の大文字小文字を区別しない）は在り、MNT-01-D11 の判定（`:952`）が使う。
- C（2 本）: Windows の `pin_to_volume`（`backup.rs:477`、71 §71.4 (2)、`71-mnt-backup.md:97`）は `\\?\Volume{GUID}\…` を返す（log: `left: "\\?\Volume{e871…}\Users\runneradmin\…\a\c\d"`）。`test_pin_to_volume_req901_d10_normalizes_before_pinning`（`backup.rs:2202-2216`）は `actual` を drive 文字の形で期待し、`test_create_backup_req901_metadata_error_propagates_without_success_log`（`:1691-1710`）は注入先 `fail_metadata`（`:81-85`、path の文字列の一致）を `canonicalize` の drive 文字の形で作るので当たらない。
- D（3 本）: `test_inspect_backup_req901_d9_uri_escapes_path_chars`（`backup.rs:2374-2396`）は `\` を含む 1 つの file 名を作る（Windows は区切りで、`rename` が `os error 3`）。`test_req401_resolve_accepts_drive_casio_and_model_folder`（`register_sd.rs:676-692`）は小文字の folder を作り同じ綴りを期待するが、`child_dir`（`:575-585`）は先に `parent.join(name)` を `is_dir` で見るので、大文字小文字を区別しない Windows では定数の綴り `CASIO\SR500_550_4000` を返す（同じ folder。IO-09-D1 の照合は大文字小文字を区別しない）。B12 `test_restore_req901_b12_partial_rollback_failure_reconciles_remaining_wal`（`restore.rs:1439-1471`）は WAL を残すために別の接続 `reader` の読みの transaction を開いたまま `restore_backup_with_ops` を呼び、`:1462` の `matches!(error, RestoreError::Unrecoverable(_))` が落ちた。log は返った variant を出さない。推定（未実測）: Windows では開いたままの file の改名が共有違反で失敗し、main の退避（`:487-506`）が `rollback_after_failure` の `Recovered` で返る。

## Scope

- S1 `.gitattributes`（新規、2 行）: `* text=auto eol=lf` と `tests/fixtures/** -text`。text の file はどの OS の checkout でも LF、fixture は bytes のまま（改行の変換も `--renormalize` もしない）。A の 7 本の test は変えない。
- S2 `src-tauri/src/mnt/offsite.rs`: `drive_root` と `same_root` の cfg の 4 つ（`:983-1015`）を消し、1 つの `fn removable_root(selected: &Path, roots: &[PathBuf]) -> Option<PathBuf>` にする: `roots` のうち、component の列が `selected` の component の列の先頭と `component_eq` で 1 つずつ等しいもの（最も長いもの）。`prepare_with` の手順 1・2（`:351-360`）は `removable_root(..).ok_or(OffsiteError::NotRemovable)?`。Linux は今の `starts_with`（component ごとの `==`）と同じ。test を 2 つ足す: (a) 選んだ path の ASCII の大文字小文字が root と違う（Windows は用意できる・Linux は `NotRemovable`。期待を `cfg!(windows)` で分ける。列の root の兄弟の `E2` を選ぶと両 OS で `NotRemovable`）、(b) `#[cfg(windows)]` の純関数の test で本番の形の path（`e:\backup` と列 `[D:\, E:\]` → `E:\`、`C:\Users\x` → None、`\\?\E:\x` → None〈今の `same_root` も一致しない〉）。
- S3 B12（`restore.rs:1439-1471`）: (a) まず `:1462` の assert に `"{error:?}"` を付けて push し、Windows observe の log で variant と message（`io_message` が失敗した操作と OS の error を含む、`:263-265`）を読む。(b) その結果で直す（下の「B12 の決め方」）。
- S4 C: `backup.rs:2211` の `actual` の期待を、Windows では prefix が `\\?\Volume{` で始まり、prefix の後の component の列が `root.join("a/c/d")` の prefix の後と等しい形にする（`normalized` の期待 `:2210` は両 OS で今のまま）。`:1697-1701` の注入先を `pin_to_volume(&backup_dir).unwrap().actual.join(name_at(now))` にする（Linux は今と同じ path。71 §71.10 の表〈`71-mnt-backup.md:556`〉の「公開の後の正式名〈実体の path〉」のとおり）。
- S5 D の 2 本: `backup.rs:2380-2384` は Windows では `\` を区切りとして folder `d#1`・`50% done` の下の `x.db` を作る（`?` は Windows の名前に使えない。純関数の assert `:2392-2395` は両 OS で今のまま）。`register_sd.rs:691` は Linux は今の綴りの一致のまま、Windows は `fs::canonicalize` の両辺の一致にする。
- S6 正本: `docs/function-design/71-mnt-backup.md:650-651`（§71.11.2 手順 1・2）を S2 の規則の文（「見えている取外し可能な drive の root〈§71.11.3 の列〉のうち `selected` を含むもの。component ごとに比べ、Windows は ASCII の大文字小文字を区別しない。無ければ `NotRemovable`」）に置き換え、`:673` に Windows でも同じ関数を通すことを 1 句足す。`docs/decision-log.md` に D-160（下の Design Readiness）。
- S7 docs: `docs/ci.md:105` の「（どれも test の前提。…）」を確かめた事実（群ごとの原因、B12 の実際の error、直した PR）に置き換える。`docs/backlog.md:55`（PR #188 の申し送りの行。完了は D-159 と PR #286 から追える）を消し、`:56-60` の項目を外す。`docs/DEV_SETUP_CHECKLIST.md:274`（`tauri dev` の後の autocrlf の「変更あり」）を、S1 の後の既存の Windows の clone の 1 回だけの操作（`git status --porcelain` が空のときだけ `git read-tree HEAD` と `git checkout-index -f -a`。local の変更を上書きする）に置き換える。`docs/msi-distribution-runbook.md:57` の autocrlf の但し書き（S1 の後は改行だけの差が出ない）を消す。

B12 の決め方（S3 (b)）:

1. `Recovered` で、失敗した操作が退避の改名（`main の退避に失敗`・`wal の退避に失敗` 等）、OS の error が共有違反（`os error 32`）か拒否（`os error 5`）: 製品は MNT-01-D4 の安全側（元の DB 一式を元の名前へ戻した）。test の前提（Linux は開いた file を改名できる）を外す: `reader` を消し、`conn` に `set_db_config(DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE, true)`（rusqlite 0.31 の `config.rs:29`・`:119`）を掛けて close の後も WAL を残し、両 OS で B12 の経路（main の復帰の後の WAL の巻き戻しの失敗 → `Unrecoverable` → reconcile で `Recovered`）を通す。Linux で WAL が残らず B12 の経路に届かない（Test Plan の mutation が red にならない）ときは、`reader` を残して Windows の期待を「`Recovered` で DB を開き直すと `old-main`・`old-wal`、復元の遺物なし」に分ける。
2. それ以外の variant・操作で、製品が MNT-01-D4 の安全側（`Recovered` で元の DB が戻る、または `Unrecoverable` で reconcile が収束する）: 根拠を PR の comment に書き、Gated Amendment で直し方を足す。
3. 契約と違う（`Ok`、`Recovered` なのに元の DB が開けない・行が欠ける、reconcile が収束しない）: 製品の欠陥として止め、owner に諮る。この lane で直さない。

Non-scope: `src-tauri/Cargo.toml`・`Cargo.lock`、53 本の外の test、`src-tauri/src/io/register_sd.rs` の本体（`child_dir` の綴りは契約の外）、`Plans.md`。

実装の順（`.github/**` に触れないので push は Coordinator ができる）: 1. 計画の固定の後、S1・S2・S3 (a)・S4・S5 の commit を push し、run を読む（52 本が通り B12 の message が出る見込み）。2. S3 (b) を push し、run で 53 本を確かめる。3. S6・S7 を push する。

## Acceptance Criteria

- AC1: 実装の順の 3 の head の Windows observe の run で、8 本の実行 file が全部 `test result: ok`、`FAILED` の行が 0（Coordinator が run の URL と数を PR の comment に残す）。
- AC2: 実装の順の 1 の run の B12 の message（variant・操作・OS の error）と、決め方のどの枝を採ったかを PR の comment に残し、`docs/ci.md` の文がそれと合う。
- AC3: Linux で `cd src-tauri && cargo test`・`cargo clippy --all-targets --all-features -- -D warnings`・`cargo fmt --check` が PASS（hosted の `Merge gate`）。`git diff origin/main --stat -- tests/fixtures` が空、`git ls-files --eol tests/fixtures | grep -c 'i/crlf.*attr/-text'` が `29`。
- AC4: Test Plan の Linux の mutation を Writer が 1 つずつ入れ、`cd src-tauri && cargo test <Test Name>` の出力が `FAILED` になることを確かめて戻し、mutation ごとの結果を PR の comment に残す。
- AC5: `rg -n 'fn drive_root|fn same_root' src-tauri/src/mnt/offsite.rs` が 0 件、`rg -n 'Windows で落ちる lib の test の前提を直す lane' docs` が本 packet だけ。`bash scripts/doc-consistency-check.sh` と `--target plan` が ERROR なし、`bash scripts/check-workflow-git.sh` が OK。

## Design Readiness

- 引用する設計正本: 71 §71.4 (2)（`pin_to_volume`）・§71.10 の表・§71.11.2・§71.11.3・§71.12、MNT-01-D4（§71.7）・D6・D8・D9・D10、29 IO-09-D1・D3、IO-11-D3、D-157（Windows observe）、D-159、D-147。
- 必要な設計成果物: S6 の 71 の文と D-160。DB・command・DTO・画面は変わらない。
- D-160 の要旨（S6 で書く）: Decision: repo の text の file はどの OS の checkout でも LF（`* text=auto eol=lf`）、`tests/fixtures/**` は bytes のまま（`-text`）。Why: Windows の autocrlf の checkout で source を読む test が落ち（53 本の A）、`tauri dev` の書き直しが「変更あり」に出る。fixture は CP932・CRLF が契約。Alternatives: (a) 7 本の test で `\r\n` を正規化する（同じ形の test が増えるたびに Windows で落ち、`tauri dev` の件が残る）; (b) runner と owner の clone の `core.autocrlf=false`（`.github/**` と人の設定に頼り、clone ごとに要る）; (c) `* -text`（binary の扱いで diff と merge が崩れる）。Revisit: CRLF を要る text の file（`.bat`・`.cmd` 等）を足すとき。
- 前提・延期した gap: `removable_drive_roots`（変えない）と本物の USB メモリの用意は自動の test の外（D-114 の lane の L3 で確かめ済み）。Windows の mutation は流さない（S2 (b) の Windows の test は Windows observe の結果だけで見る）。
- 絶対保証の例外: 主張しない。S2 の root の判定は列の root の component の一致だけを見て、drive の種類は `removable_drive_roots` が持つ。
- 判定: ready（計画の review の前）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| MNT-01-D8 用意の root（取外し可能でない root を拒む） | 71 §71.11.2 手順 1・2、§71.11.3 | S2・S6 | 39 本と CMD の 2 本（Windows observe）、`test_prepare_offsite_req901_d8_rejects_non_removable`、S2 (a)・(b) | 本番の列は変えない（`removable_drive_roots`）。本番の結果は同じ: 取外し可能な `X:\` の下を選べば `X:\`、それ以外は `NotRemovable` |
| MNT-01-D10・D11（SD・同じ folder） | 71 §71.11.2 手順 3・3c | 変えない | `test_offsite_req901_d10_prepare_and_copy_refuse_register_sd_root` ほか（Windows で初めて通る） | — |
| 71 §71.4 (2)・MNT-01-D6 | 71 §71.4、§71.10 の表 | S4 | C の 2 本 | 製品は変えない |
| MNT-01-D9（URI の符号化） | 71 §71.12 手順 1 | S5 | `test_inspect_backup_req901_d9_uri_escapes_path_chars` | 製品は変えない |
| IO-09-D1（SD の root） | 29 §29.7.4 手順 3 | S5 | `test_req401_resolve_accepts_drive_casio_and_model_folder` | 返す綴りは契約の外 |
| MNT-01-D4（B12） | 71 §71.7 手順 8e、MNT-01-D4 | S3 | B12 の test | 決め方の 3 は owner |
| source を読む test（SPEC-SFV・D-142・IO-09-D3・IO-11-D3・MNT-01-D7） | 各 test の正本 | S1 | A の 7 本 | test は変えない |
| D-160（改行） | decision-log D-160（新規） | S1・S6・S7 | A の 7 本（Windows observe）、AC3 の fixture | 既存の clone は S7 の操作 |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| Windows の checkout が LF | `.gitattributes` が無い・`eol=lf` が無い | Rust（Windows observe） | A の 7 本 | S1 の 1 行目を消す（Windows で 7 本が red。Linux では見えない） |
| fixture の bytes を変えない | fixture が text として正規化される | CLI | AC3 | 2 行目を消して throwaway の clone で `git add --renormalize tests/fixtures` を流す（29 本が変更に出る。Writer が手元で確かめる） |
| 選んだ path を含む root だけを用意する | 列の外の path を通す・root の下の folder を root にしない | Rust（mutation、Linux） | `..._rejects_non_removable`、`..._d10_prepare_and_copy_refuse_register_sd_root`（`sd.join("x")` を選ぶ） | `removable_root` が常に `Some(selected)` を返す（前者が red）。一致の root でなく `selected` を返す（後者の `sd_folder` が red） |
| 一致は component ごと | 文字列の前方一致で `E2` が `E` の下になる | Rust（mutation、Linux） | S2 (a) の兄弟の case | component の比べを `to_string_lossy().starts_with` にする |
| Windows は ASCII の大文字小文字を区別しない | Windows の比べが `==` | Rust（Windows observe） | S2 (a)・(b) | Windows の `component_eq` を `a == b` にする（Windows で red。流さない、Design Readiness） |
| Windows の `pin_to_volume` は Volume GUID | 期待を drive 文字の形に戻す | Rust | C の 2 本 | `:1701` の注入先を `canonicalize` の形に戻す（Windows で red。Linux は同じ path で通る） |
| metadata の失敗を成功にしない | 注入が当たらず `Ok` | Rust（mutation、Linux） | `test_create_backup_req901_metadata_error_propagates_without_success_log` | 注入先を別の名前にする（`Ok` で red） |
| B12 の経路が両 OS で通る | WAL が残らず巻き戻しの失敗に届かない | Rust（mutation、Linux） | B12 の test | S3 (b) の 1 の後で `RollbackWalAfterMain` を `None` にする（`Unrecoverable` の assert が red）。`NO_CKPT_ON_CLOSE` の行を消す（WAL が残らず red） |
| URI の符号化 | Windows の path で `#`・`%` を符号化しない | Rust | D の uri の test | `sqlite_uri_path` の `%23` の置換を消す（両 OS で red） |
| SD の root を見つける | 別の folder を返す | Rust（mutation、Linux） | D の resolve の test | `child_dir` が `parent` を返す（red） |
