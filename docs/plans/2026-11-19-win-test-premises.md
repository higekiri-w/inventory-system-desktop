# Plan Packet: Windows で落ちる lib の test 60 本の前提を直す

wave に属さない単独の lane。出所は owner の決定（repo 外の回答台帳 TD-825 の A、2026-11-19。関門の作業の合間に）と [backlog](../backlog.md) の「Windows で落ちる lib の test の前提を直す lane」（群 A〜D）。事実は起点 `91fec088`（PR #287 を含む）の Windows observe の run <https://github.com/higekiri-w/inventory-system-desktop/actions/runs/44523246270>（lib の unittests 1559 passed・60 failed・4 ignored）。経緯: PR #286 の run（`44504745664`）と owner の Windows（#286 の comment `7008849050`）では 53 本。PR #286 の Final の P2（B12 の原因が確かめられていないのに分類を閉じて読める）と P3（`docs/backlog.md:55` の行）をこの lane で直す（Coordinator の裁定）。

decision-log の番号は D-161（D-160 は PR #292 が使った。`git show origin/main:docs/decision-log.md | grep -o '^## D-[0-9]*' | tail -1` → `## D-160`、2026-11-20）。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R3

Reason:
`.gitattributes` を足して Windows の新しい clone の改行を変え（classifier は unknown の path として全部の分類を `true` にする: `printf '%s\n' .gitattributes | bash scripts/ci/classify-changes.sh --files-from-stdin`、2026-11-19）、PC の外の控えの媒体の root を決める関数（`mnt/offsite.rs`）の Windows の分岐を Linux と共通にする。R4 に当てない理由: 消す file の組・上書きしない公開・復元の経路を変えず、root の判定は本番で今と同じ（返す root の綴りは列の `X:\`。Contract Ledger）。データの安全の境界に触れるので R2 にしない。

## Goal

Goal Invariant:

### 最小完了条件

- lane の PR の最後の head の Windows observe の run で、tauri の crate の 8 本の test の実行 file が全部 `test result: ok`（60 本が通り、新しく落ちる test が無い）。
- 群ごとに、製品が Windows で契約どおりに振る舞う根拠が PR の comment にある。B12 は Windows で実際に返る error（variant・失敗した操作・OS の error）を run の log で確かめた結果で直し方を決める。
- Linux の test・clippy・fmt は今のまま通り、直した test が Linux で今の契約を弱めていない（Test Plan の mutation）。
- `docs/ci.md` と `docs/backlog.md` の Windows の失敗の文が確かめた事実になる。

### 失敗定義

- 60 本のどれかを skip・`#[cfg(not(windows))]`・期待の弱めで消す（契約が Windows に当たらないと正本で言える場合を除く。根拠は PR に）。
- tracked の fixture（CP932・CRLF の合成 CSV）の bytes が変わる。
- Windows の本番で、取外し可能でない drive の folder を用意できる、または今用意できる USB メモリを用意できなくなる。
- B12 を推定のまま閉じる。

### 非目的

- Windows observe の昇格（enforce）、`.github/workflows/**` と `scripts/**` の変更（classifier に `.gitattributes` を足すことを含む）、製品の振舞いの変更、report の検査の深さの残り 3 つ（backlog）、`i/mixed` の 4 本の正規化（`--renormalize` は流さない）。

## 確かめた事実（2026-11-19、`91fec088` と run の log）

- log の落ちた名: `grep -E ' test .* \.\.\. FAILED' <log> | grep -v warning | awk '{print $3}' | sort -u | wc -l` → `60`（A 7・B 48〈`mnt::offsite` 46・`cmd::settings_cmd` 2〉・C 2・D 3）。log は job `135557034512`（#286 は `135493302020` で 53）。`comm -13` で増えた 7 本は群 B で、どれも `Fx::prepare`（`offsite.rs:1393`）の `unwrap` が `NotRemovable`: `mnt::offsite::tests::` の `test_check_offsite_req901_d12_{only_future_dated_is_no_local_backup,retention_ignores_future_dated_copies,skips_future_dated_source}`・`test_offsite_req901_d12_{clock_ahead_then_fixed_copies_and_clears_stale,clock_set_back_copies_backdated_backup}`・`test_offsite_status_req901_d12_{future_record_is_treated_as_absent,local_backup_old_ignores_future_dated}`。
- A（7 本）: どれも自分の source（`src-tauri/src/` の `*.rs`）を読み `\n` を含む文字列で探す。repo に `.gitattributes` が無い。`git ls-files --eol` の `i/crlf` の 29 は全部 `tests/fixtures/` の合成の CSV（`.CSV` 23・`.csv` 6。tracked の CSV はこれだけ）で、CP932・CRLF が契約（各 README）。`i/mixed` の 4 本は test に関係しない。
- 改行の probe（Linux の `core.autocrlf=true` で Windows を代用。実の Windows は未実測）: bare の clone に属性の commit を足し、(a) 前の commit の `git clone -c core.autocrlf=true -b pre` で `git pull --ff-only` の後の `git status --porcelain`、新しい `git clone -c core.autocrlf=true` で (b) `i/crlf` の各 file の `cmp <f> <(git cat-file blob :<f>)`、(c) `git ls-files --eol '*.rs'`、(d) `src-tauri/Cargo.toml`・`src/lib/bindings.ts` を index の LF で書き直した後の `git status --porcelain`。

| 属性 | (a) | (b) | (c) | (d) |
|---|---|---|---|---|
| なし | 0 | 29/29 | 121 `w/crlf` | ` M` 2 本（11-14 の観測と同じ） |
| S1（退けた案 (d)・(e) とその理由は D-161 の Alternatives） | 0 | 29/29 | 121 `w/lf` | 0 |

  S1 の後: `git ls-files --eol tests/fixtures | grep -c 'i/crlf.*attr/-text'` → `29`、CSV の 2 行を消して `git add --renormalize tests/fixtures` → 29 本が変更。前の clone は pull の後も `lib.rs` が `w/crlf`、(d) で 2 本が ` M`。`tauri dev` が書き直す file は 11-14 の観測の 2 本で、他は未確認。
- B（48 本）: `prepare_with`（`offsite.rs:344-360`）は `drive_root` の root が列に `same_root` で在るかで `NotRemovable` を決める。Windows の `drive_root`（`:983-995`）は注入の `roots` を使わず prefix と root を返すので、test が注入した `\\?\C:\…\E`（`Fx::new`、`:1368`。CMD は `settings_cmd.rs:991-995`）が `\\?\C:\` になり `NotRemovable`。Windows 以外（`:997-1005`）は注入の列のうち選んだ path を含む最も長いもの。本番の列は `removable_drive_roots()`（`register_sd.rs:158-185`、`DRIVE_REMOVABLE` の `X:\`）。71 §71.11.3 は production と test が同じ内部関数を通す。`component_eq`（`offsite.rs:960-980`、Windows は ASCII の大文字小文字を区別しない）は MNT-01-D11（`:952`）が使う。
- C（2 本）: Windows の `pin_to_volume`（`backup.rs:477`、71 §71.4 (2)）は `\\?\Volume{GUID}\…` を返す（log の `left`）。`test_pin_to_volume_req901_d10_normalizes_before_pinning`（`backup.rs:2202-2216`）は `actual` を drive 文字の形で期待し、`test_create_backup_req901_metadata_error_propagates_without_success_log`（`:1691-1710`）は注入先（`:81-85`）を drive 文字の形で作るので当たらない。
- D（3 本）: `test_inspect_backup_req901_d9_uri_escapes_path_chars`（`backup.rs:2374-2396`）は `\` を含む 1 つの file 名を作る（Windows は区切り。`os error 3`）。`test_req401_resolve_accepts_drive_casio_and_model_folder`（`register_sd.rs:676-692`）は小文字の folder の綴りを期待するが、`child_dir`（`:575-585`）は Windows では定数の綴り `CASIO\SR500_550_4000` を返す（同じ folder。IO-09-D1 は大文字小文字を区別しない）。B12 `test_restore_req901_b12_partial_rollback_failure_reconciles_remaining_wal`（`restore.rs:1439-1471`）は WAL を残すために別の接続 `reader` の読みの transaction を開いたまま復元し、`:1462` の `Unrecoverable` の assert が落ちた（log は variant を出さない）。推定（未実測）: Windows では開いたままの file の改名が共有違反で失敗し、main の退避（`:487-506`）が `Recovered` で返る。

## Scope

- S1 `.gitattributes`（新規、3 行）: `* text=auto eol=lf`、`*.csv -text`、`*.CSV -text`。新しい clone の text は LF を既定とし、CSV は bytes のまま、既存の混在改行の 4 本（`i/mixed`）は正規化しない（`--renormalize` しない）。A の 7 本は変えない。
- S2 `src-tauri/src/mnt/offsite.rs`: `drive_root` と `same_root` の cfg の 4 つ（`:983-1015`）を消し、1 つの `fn removable_root(selected: &Path, roots: &[PathBuf]) -> Option<PathBuf>` にする: `roots` のうち、component の列が `selected` の先頭と `component_eq` で等しい最も長いもの。`prepare_with` の手順 1・2（`:351-360`）は `removable_root(..).ok_or(OffsiteError::NotRemovable)?`。Linux は今の `starts_with` と同じ。test を 2 つ足す: (a) 選んだ path の ASCII の大文字小文字が root と違う（Windows は用意できる・Linux は `NotRemovable`、`cfg!(windows)` で分ける。root の兄弟の `E2` は両 OS で `NotRemovable`）、(b) `#[cfg(windows)]` の純関数の test で本番の形の path（列 `[D:\, E:\]` で `E:\` → `E:\`、`e:\backup` → `E:\`、`C:\Users\x` → None、`\\?\E:\x` → None）。`cd src-tauri && cargo run --bin generate_traceability` で `docs/function-design/90-traceability.md` を作り直す（`offsite.rs` 54 → 56 件、Codex の実測）。
- S3 B12（`restore.rs:1439-1471`）: (a) まず受け取った `error` を `eprintln!("{error:?}")` で出し、`:1462` の assert に `"{error:?}"` を付け、その前で `Recovered` なら `reader` を閉じて既存の DB を `db::open_existing_database`（no-create）で開き、`supplier_names` と `assert_no_restore_artifacts` の結果を出して push し、log で variant・message（失敗した操作と OS の error、`:263-265`）・元の行を読む。(b) その結果で直す（下の「B12 の決め方」）。
- S4 C: `backup.rs:2211` の `actual` の期待を、Windows では prefix が `\\?\Volume{` で、prefix の後の component が `root.join("a/c/d")` の prefix の後と等しい形にする（`:2210` は今のまま）。`:1697-1701` の注入先を `pin_to_volume(&backup_dir).unwrap().actual.join(name_at(now))` にする（Linux は今と同じ path。71 §71.10 の表の実体の path）。
- S5 D の 2 本: `backup.rs:2380-2384` は Windows では土台を `std::fs::canonicalize(dir.path())`（実際に開く path に `?` が入る）にし、`\` を区切りとして folder `d#1`・`50% done` の下の `x.db` を作る（純関数の assert `:2392-2395` は今のまま）。`register_sd.rs:691` は Linux は今の綴りの一致のまま、Windows は `fs::canonicalize` の両辺の一致にする。
- S6 正本: 71 `:650-651`（§71.11.2 手順 1・2）は root の選び方だけを差し替える: 手順 1 の前半を「`selected` を、見えている取外し可能な drive の root〈§71.11.3 の列〉のうち `selected` を含むものにする（component ごとに比べ、Windows は ASCII の大文字小文字を区別しない）」、手順 2 を「無ければ `OffsiteError::NotRemovable`」にする。手順 1 の後半（以後の読み書きは `pin_to_volume` で固定した実体を通す）は残す。`docs/decision-log.md` に D-161（下の Design Readiness）。
- S7 docs: `docs/ci.md:105` の「（どれも test の前提。…）」を確かめた事実（群ごとの原因、B12 の error、直した PR）にする。`docs/backlog.md:55`（PR #188 の申し送り。D-159 と PR #286 で追える）を消し、`:56-60` の項目を外し、1 項目を足す: `test_restore_req901_b1_evacuation_failure_preserves_real_wal_snapshot` も開いた接続の下の改名に頼り、B12 の推定が当たれば Windows で空振りする。もう 1 項目を足す: owner の Windows の clone の手順の安全（`docs/DEV_SETUP_CHECKLIST.md:274` の `git restore .`、`docs/msi-distribution-runbook.md:57` の `git restore package-lock.json`、同期の手順 `docs/DEV_SETUP_CHECKLIST.md:294` の `git reset --hard` は、`assume-unchanged` の編集や意図した未 stage の編集を消しうる。S1 の後に新しい clone へ移るときの置き場と、固定の path の手順との関係も決める。出所: この lane の計画の review round 1〜3、owner の決定 TD-837）。この lane はこれらの手順書を変えない（S1 の後も前の clone では今の文が当たる）。

B12 の決め方（S3 (b)）:

1. `Recovered` で、失敗した操作が退避の改名、OS の error が `os error 32`（共有違反）か `5`（拒否）で、S3 (a) の no-create の open が `old-main`・`old-wal` の 2 行と遺物なし: 製品は MNT-01-D4 の安全側。test の前提（Linux は開いた file を改名できる）を外す: `reader` を消し、`conn` に `set_db_config(DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE, true)`（rusqlite 0.31 の `config.rs:29`・`:119`）で close の後も WAL を残し、両 OS で B12 の経路（WAL の巻き戻しの失敗 → `Unrecoverable` → reconcile で `Recovered`）を今の assert のまま通す。Linux の mutation が red にならない、または Windows で `Unrecoverable` に届かないときは 2 へ。
2. 1 に当たらない・届かない場合で、製品が MNT-01-D4 の安全側（`Recovered` で元の DB が戻る、または `Unrecoverable` で reconcile が収束する）: 根拠を PR の comment に書き、Gated Amendment で直し方を足す（期待を OS で分けない。Windows でも WAL の巻き戻しの失敗 → `Unrecoverable` → reconcile で `Recovered` の経路を今の assert で通す）。
3. 契約と違う（`Ok`、`Recovered` なのに元の DB が開けない・行が欠ける・遺物が残る、reconcile が収束しない）: 製品の欠陥として止め、owner に諮る。この lane で直さない。

Non-scope: `src-tauri/Cargo.toml`・`Cargo.lock`、60 本の外の test（実装の順 1 の扱いを除く）、`src-tauri/src/io/register_sd.rs` の本体（`child_dir` の綴りは契約の外）、`Plans.md`、clone の手順書（`docs/DEV_SETUP_CHECKLIST.md`・`docs/msi-distribution-runbook.md`。S7 の backlog の項目、TD-837）。

実装の順（push は Coordinator）: 1. 計画の固定の後、S1・S2・S3 (a)・S4・S5 の commit を push し、run を読む（59 本が通り B12 の message が出る見込み）。60 本の外か新しく落ちた test（B の 2 段目の assert 等）は B12 の決め方 2・3 と同じに扱う（Gated Amendment か owner）。2. S3 (b) を push し、run で 60 本を確かめる。3. S6・S7 を push する。

## Acceptance Criteria

- AC1: 実装の順の 3 の head の Windows observe の run で、8 本の実行 file が全部 `test result: ok`、`FAILED` の行が 0（run の URL と数を PR の comment に）。
- AC2: 実装の順の 1 の run の B12 の message（variant・操作・OS の error）と元の行と、決め方のどの枝を採ったかを PR の comment に残し、`docs/ci.md` の文がそれと合う。
- AC3: Linux で `cd src-tauri && cargo test`・`cargo clippy --all-targets --all-features -- -D warnings`・`cargo fmt --check` が PASS（hosted の `Merge gate`）。`git diff origin/main --stat -- tests/fixtures` が空、`git ls-files --eol tests/fixtures | grep -c 'i/crlf.*attr/-text'` が `29`。
- AC4: Test Plan の Linux の mutation を Writer が 1 つずつ入れ、`cd src-tauri && cargo test <Test Name>` の出力が `FAILED` になることを確かめて戻し、mutation ごとの結果を PR の comment に残す。
- AC5: `rg -n 'fn drive_root|fn same_root' src-tauri/src/mnt/offsite.rs` が 0 件、`rg -n 'Windows で落ちる lib の test の前提を直す lane' docs` が本 packet だけ。`cd src-tauri && cargo run --bin generate_traceability -- --check` が ERROR 0。`bash scripts/doc-consistency-check.sh` と `--target plan` が ERROR なし、`bash scripts/check-workflow-git.sh` が OK。

## Design Readiness

- 引用する設計正本: 71 §71.4 (2)・§71.10 の表・§71.11.2・§71.11.3・§71.12、MNT-01-D4（§71.7）・D6・D8・D9・D10、29 IO-09-D1・D3、IO-11-D3、D-157（Windows observe）、D-159、D-147。
- 必要な設計成果物: S6 の 71 の文と D-161。DB・command・DTO・画面は変わらない。
- D-161 の要旨（S6 で書く）: Decision: repo の text の file は新しい clone で LF を既定とする（`* text=auto eol=lf`。既存の混在改行の 4 本は正規化しない）、CSV は bytes のまま（`*.csv`・`*.CSV` に `-text`。29 本の CP932・CRLF の fixture）。前の clone は改行が今のまま残る（手順は backlog の別の項目）。Why: Windows の autocrlf で source を読む A の 7 本が落ち、`tauri dev` の後に「変更あり」が出る。Alternatives: (a) test で `\r\n` を正規化（同じ形の test が増えるたびに落ち、`tauri dev` の件が残る）; (b) `core.autocrlf=false`（人の設定に頼る）; (c) `* -text`（diff と merge が崩れる）; (d) `tests/fixtures/** -text`（前の clone の pull で 5 本が ` M`）; (e) `*.rs` と名指しの file だけを LF（書き直す file・読む source が増えるたびに名指しを足す）。Revisit: CRLF の要る file（`.bat` 等）を足すとき。
- 前提・延期した gap: `removable_drive_roots`（変えない）と本物の USB メモリの用意は自動の test の外（D-114 の L3 で確かめ済み）。Windows の mutation は流さない（S2 (b) は Windows observe の結果だけで見る）。
- 絶対保証の例外: 主張しない。S2 は列の root の component の一致だけを見て、drive の種類は `removable_drive_roots` が持つ。
- 判定: ready（計画の review の前）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| MNT-01-D8 用意の root（取外し可能でない root を拒む） | 71 §71.11.2 手順 1・2、§71.11.3 | S2・S6 | 46 本と CMD の 2 本（Windows observe）、`test_prepare_offsite_req901_d8_rejects_non_removable`、S2 (a)・(b) | 本番の列は変えない。判定は同じ（返す root の綴りは列の `X:\`） |
| MNT-01-D10・D11（SD・同じ folder） | 71 §71.11.2 手順 3・3c | 変えない | `..._d10_prepare_and_copy_refuse_register_sd_root` ほか（Windows で初めて通る） | — |
| 71 §71.4 (2)・MNT-01-D6 | 71 §71.4、§71.10 の表 | S4 | C の 2 本 | 製品は変えない |
| MNT-01-D9（URI の符号化） | 71 §71.12 手順 1 | S5 | `test_inspect_backup_req901_d9_uri_escapes_path_chars` | 製品は変えない |
| IO-09-D1（SD の root） | 29 §29.7.4 手順 3 | S5 | `test_req401_resolve_accepts_drive_casio_and_model_folder` | 返す綴りは契約の外 |
| MNT-01-D4（B12） | 71 §71.7 手順 8e、MNT-01-D4 | S3 | B12 の test | 決め方の 3 は owner |
| source を読む test（SPEC-SFV・D-142・IO-09-D3・IO-11-D3・MNT-01-D7） | 各正本 | S1 | A の 7 本 | test は変えない |
| D-161（改行） | decision-log D-161（新規） | S1・S6・S7 | A の 7 本（Windows observe）、AC3 の fixture | 前の clone の手順は変えない（S7 の backlog の項目） |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| Windows の checkout が LF | `.gitattributes` が無い・`eol=lf` が無い | Rust（Windows observe） | A の 7 本 | S1 の 1 行目を消す（Windows で red） |
| fixture の bytes を変えない | fixture が text として正規化される | CLI | AC3 | CSV の 2 行を消して throwaway の clone で `git add --renormalize tests/fixtures`（29 本が変更、probe） |
| 選んだ path を含む root だけを用意する | 列の外の path を通す・root の下の folder を root にしない | Rust（mutation、Linux） | `..._rejects_non_removable`、`..._d10_prepare_and_copy_refuse_register_sd_root` | 常に `Some(selected)` を返す（前者が red）。一致の root でなく `selected` を返す（後者が red） |
| 一致は component ごと | 文字列の前方一致で `E2` が `E` の下になる | Rust（mutation、Linux） | S2 (a) の兄弟の case | component の比べを `to_string_lossy().starts_with` にする |
| Windows は ASCII の大文字小文字を区別しない | Windows の比べが `==` | Rust（Windows observe） | S2 (a)・(b) | Windows の `component_eq` を `a == b` にする（Windows で red。流さない） |
| Windows の `pin_to_volume` は Volume GUID | 期待を drive 文字の形に戻す | Rust | C の 2 本 | `:1701` の注入先を `canonicalize` の形に戻す（Windows で red） |
| metadata の失敗を成功にしない | 注入が当たらず `Ok` | Rust（mutation、Linux） | `test_create_backup_req901_metadata_error_propagates_without_success_log` | 注入先を別の名前にする（`Ok` で red） |
| B12 の経路が両 OS で通る | WAL が残らず巻き戻しの失敗に届かない | Rust（mutation、Linux） | B12 の test | 枝 1 の後で `RollbackWalAfterMain` を `None` にする・`NO_CKPT_ON_CLOSE` の行を消す（どちらも `unwrap_err` か `Unrecoverable` の assert が red） |
| URI の符号化 | Windows の path で `#`・`%` を符号化しない | Rust | D の uri の test | `sqlite_uri_path` の `%23` の置換を消す（両 OS で red） |
| SD の root を見つける | 別の folder を返す | Rust（mutation、Linux） | D の resolve の test | `child_dir` が `parent` を返す（red） |
| REQ の追跡表 | S2 の test の追加で件数がずれる | CLI | AC5 の `--check` | 再生成を忘れる（`ERROR [T1]`） |
