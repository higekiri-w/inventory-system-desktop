# Plan Packet: Windows の runner で Rust の test を流す observe の job

wave に属さない単独の lane。出所は owner の決定（repo 外の回答台帳 TD-770、2026-11-17）: 記事 <https://claude.dev/blog/how-the-claude-code-team-thinks-about-verification/> の 3 種の test を始め、改訂が要る決定は改訂し、入れて有効性を確かめる。本 lane はそのうち「新しい gate は observe（出すだけ）から入れ、誤検知を先に洗う。flaky な test を retry・skip しない」の考えで、Windows でだけ compile・実行される Rust の code を hosted で流す。

decision-log の番号は Coordinator が割り当てた（本 lane = D-157、PR #274 = D-158）。

## Workflow State

- Risk: R3
- Plan Commit: 524b57fad068437e7b98a3a1317754086f4f92db
- Amendments: none

## Risk

Risk: R3

Reason:
新しい `.github/workflows/*` と `scripts/tests/*` は classifier が full（`workflow=true` を含む）、`docs/ci.md` は `docs=true workflow=true` を出す（`printf '%s\n' <path> | bash scripts/ci/classify-changes.sh --files-from-stdin`、2026-11-17）。hosted の CI と merge の helper の前提（`mergeable_state`）に触れる自動化で、R2 にできない。

## Goal

Goal Invariant:

### 最小完了条件

- `src-tauri/**` に触れる PR ごとに、`windows-latest` で `cargo test` と `cargo clippy --release --all-targets -- -D warnings` が走り、結果（2 つの step の outcome と落ちた test の名）が run の summary と `::warning` の annotation に出る。
- 結果は読まれる: `src-tauri/**` に触れる PR では、Coordinator が Ready の承認を求める返答に Windows observe の outcome と落ちた test の名を書く（job は緑のままなので、checks の一覧の色だけに頼らない）。昇格の数え方はその記録で行う。
- cargo の step が落ちても、job は成功で終わり、`Merge gate` の合否・helper の `status` と `merge` の判定を変えない（observe）。run の進行中は helper の `merge` が `PR not merge-clean` で止まる。同じ head なら run の完了の後に打ち直す（承認の取り直しも同期も要らない）ことを docs に書く。
- 保証の例外: checkout・toolchain・cache の step の失敗と job の cancel（job の `timeout-minutes` の時間切れを含む）では job が失敗・cancel で終わり、helper の `merge` が止まる。戻し方は job の再実行で、test の retry ではない（落ちたのは test でない）ことを `docs/ci.md` に 1 文で書く。cargo の step の時間切れ（step の `timeout-minutes`）も保証の例外に置く: 公開の runner の source（`StepsRunner.cs`、closure round 2 の Codex の読み）では failure に変わって `continue-on-error` が当たる作りだが、hosted では未実測。AC6 の実測か実際の時間切れの観測で確かめたら、上の observe の側に戻す。
- 初回の結果と時間を測り、`docs/ci.md` に書く（`未実測` のままにしない）。昇格（enforce）の条件が `docs/ci.md` にある。

### 失敗定義

- 保証の例外の外で observe の job が失敗し、helper の `merge` が止まる（run の完了の後も `mergeable_state` が `clean` にならない）、または `Merge gate` が変わる。
- 失敗が PR の画面で見えない（summary・annotation に outcome と落ちた test の名が無い）、Ready の承認を求める返答に結果が書かれない、または retry・skip で失敗を消す。
- 失敗の経路（意図的に落とした回）を一度も観測せずに observe の保証を主張する。
- secret を使う・`permissions` が `contents: read` を超える・npm を使う。

### 非目的

- `Merge gate`・`scripts/pr-gate.py`・`.github/workflows/ci.yml`・`scripts/ci/check-required-jobs.sh` の変更、enforce への昇格。
- E2E（WebDriver）、frontend の build、`tauri build`（`custom-protocol` を有効にした release の build）。
- Windows で落ちた test の修正（見つかれば別の lane。本 lane は名と原因の分類を PR の comment に残すまで）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

- S1 `.github/workflows/windows-observe.yml`（新規）:
  - `name: Windows observe`。job の名は `Merge gate` にしない（ruleset が要る context と同じ名を持たせない）。
  - `on`: `pull_request`（`branches: [main]`、`types: [opened, reopened, synchronize]`、`paths: ['src-tauri/**', '.github/workflows/windows-observe.yml']`）と `workflow_dispatch`。ci.yml の trigger（`.github/workflows/ci.yml:3-7`）に合わせ、`paths` だけを足す（required でない workflow なので、docs だけの PR で cold build と helper の `merge` の止まりを起こさない）。
  - `permissions: contents: read`、`concurrency: group: windows-observe-${{ github.event.pull_request.number || github.ref }}`・`cancel-in-progress: true`（ci.yml:9-14 と同じ形で group の名だけ変える）、`env` は ci.yml:16-18 と同じ 2 行。
  - job 1 本: job の `name` も `Windows observe`（check run の名は job の名で出るので、AC4 の「`Windows observe` の check run」をこの名で引く）、`runs-on: windows-latest`、`timeout-minutes: 60`（hang で runner を 6 時間占めない）、`defaults.run.shell: bash` と `defaults.run.working-directory: src-tauri`（Windows の既定の shell は `pwsh` で、bash の書き方の step が落ちるため。`bash` は `bash --noprofile --norc -eo pipefail {0}` で走る）。steps は `actions/checkout@v6` → `dtolnay/rust-toolchain@stable`（`components: clippy`。ci.yml:92-94 と同じ形）→ `actions/cache@v5`（ci.yml の Rust の job と同じ 3 path・key・restore key。key は `runner.os` で Windows に分かれる）→ `cargo test 2>&1 | tee "$RUNNER_TEMP/cargo-test.log"`（`id: test`、`continue-on-error: true`、step の `timeout-minutes: 30`。`pipefail` で `cargo test` の exit を保つ）→ `cargo clippy --release --all-targets -- -D warnings`（`id: clippy`、`continue-on-error: true`、step の `timeout-minutes: 20`。release の構成の検査と Windows 専用の条件付きコンパイルの code の lint を兼ねる。理由と限界は S5）→ 最後の step（`if: always()`）が 2 つの `steps.<id>.outcome` と log の `^test .* FAILED$` に当たる行（落ちた test の名）を `$GITHUB_STEP_SUMMARY` に書き、`failure` の step ごとに `::warning title=Windows observe::<step> failed`、落ちた test ごとに `::warning` を出す。log が無い・一致が 0 件でも exit 0 で終わる書き方にする（report の step 自身が job を落とさない）。retry しない。step の時間の合計（30 + 20）は job の 60 より小さく、残りを checkout・toolchain・cache に当てる（値は `未実測` の初期値で、AC6 の実測で見直す）。
  - 秘密を使わない（`secrets.` も `GITHUB_TOKEN` の env も書かない）。npm・Node を使わない。
- S2 `scripts/tests/run-workflow-tests.sh:8`: YAML の parse の列に `.github/workflows/windows-observe.yml` を足す。
- S3 `scripts/tests/ci-workflow.test.sh`: observe の契約を確かめる関数を 1 つ足し、mutant で拒むことを確かめる（Test Plan）。
- S4 `docs/ci.md`: `## Windows Observe` 節を足す（目的、observe であること、`Merge gate` の外である根拠、run の進行中は helper の `merge` が止まり、同じ head なら完了の後に打ち直す〈承認の取り直しも同期も要らない〉こと、保証の例外と戻し方〈job の再実行は test の retry ではない〉の 1 文、「`src-tauri/**` に触れる PR では、Coordinator が Ready の承認を求める返答に Windows observe の outcome と落ちた test の名を書く。昇格の数え方はその記録で行う」の 1 文、初回の時間、昇格の条件）。Cache Policy の「Rust 3 job」の行（`docs/ci.md:88`）に、Windows の job は OS の key で別の cache を持つことを 1 句で足す。CI-PUBLIC-D1 は変えない（`windows-latest` は public の repository の standard runner、下の事実）。
- S5 `docs/decision-log.md`: 末尾に D-157（Status: owner の決定 TD-770、本 packet で起票。Context: hosted は全 job が `ubuntu-latest` で Windows 専用の条件付きコンパイルの code を compile しない、記事の observe の考え。Decision: S1 の形、observe を step の `continue-on-error` で実装する理由〈helper の `mergeable_state == 'clean'`〉、job が緑のままなので結果を summary・annotation と Ready の承認を求める返答の記録で読む形、`cargo clippy --release --all-targets -- -D warnings` を使う理由〈`src-tauri/src/lib.rs` の `#[cfg(debug_assertions)]` の code は release で外れる。Windows 専用の code は他のどこでも `-D warnings` を受けない〉と限界〈`src-tauri/Cargo.toml:25` の `tauri` は `custom-protocol` を有効にしないので、`tauri build` と同じ構成ではない。cargo 自身の lint（`cargo::unused_dependencies` など）は `-- -D warnings` の外で、summary にも出ない（未確認）〉、保証の例外〈checkout・toolchain・cache の失敗と job の cancel、cargo の step の時間切れ〉、昇格は別の R3。Alternatives: ci.yml に job を足す〈`check-required-jobs.sh` と `Merge gate` が変わる〉、job の `continue-on-error`〈check の結論が失敗のまま残るかを確かめていない〉、`cargo check --release` だけ〈Windows 専用の code の lint が無い〉、schedule だけ〈PR の L3 より前に見えない〉、paths を付けない〈docs の PR でも cold build と helper の止まり〉。Guarantee range: Windows の runner の image と店の PC は同じでない。L3 を置き換えない。Revisit: 昇格の条件を満たしたとき、誤検知が出たとき、run の進行中の止まりが merge の運用を妨げたとき）。

Non-scope: `.github/workflows/ci.yml`、`scripts/pr-gate.py`、`scripts/ci/check-required-jobs.sh`、`scripts/ci/classify-changes.sh`、`.github/merge-gate-ruleset.json`、`src-tauri/**`。

push の順（`.local/checklists/env-facts.md` の E9: sandbox の token は Workflows の権限を持たない）: 計画の commit と `plan-approve.sh` の commit は Coordinator が push する。S1〜S5 の実装の commit は `.github/workflows/**` を含むので、owner が owner の terminal で push する（Coordinator は push の command を返答に inline で渡す）。AC4 の意図的な失敗の回の一時の commit と、それを戻す commit も `.github/workflows/windows-observe.yml` に触れるので、それぞれ owner が owner の terminal で push する（順: 実装の commit → 一時の commit → 観測 → 戻す commit）。

## 確かめた事実（2026-11-17）

- helper は `ci.yml` の run だけを読む: `scripts/pr-gate.py:935` の API は `actions/workflows/ci.yml/runs`、`:939` は `run['path'] == '.github/workflows/ci.yml'`、`:942-945` はその check suite の中の `Merge gate` だけを数える。別の workflow の run は `ci()` に入らない。
- ただし helper の `merge` は `scripts/pr-gate.py:1086` で `pr['mergeable_state'] == 'clean'` を要る。GitHub の GraphQL の `MergeStateStatus`（<https://docs.github.com/en/graphql/reference/pulls>、2026-11-17 に WebFetch）は `CLEAN`「Mergeable and passing commit status.」、`UNSTABLE`「Mergeable with non-passing commit status.」。required でない check が失敗・進行中でも `clean` でなくなると読める（check run が commit status と同じに数えられるかは未確認。AC4 は完了した check run の success と同じ head の `clean` の組を観測し、進行中の値は観測しない）。発注の前提（別の workflow なら helper に触れない）はこの点で足りず、S1 は job の結論を成功に保つ（step の `continue-on-error`）。step の `continue-on-error` は「Prevents a job from failing when a step fails. Set to `true` to allow a job to pass when this step fails.」（<https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax>、同日 WebFetch）。job の `continue-on-error` の check の結論は確かめていない。
- `Merge gate` の required jobs: `scripts/ci/check-required-jobs.sh:17` の `required` の集合と `:26` の `set(needs) != set(required)` は ci.yml の `merge_gate` の `needs`（`.github/workflows/ci.yml:259`）だけを見る。別の workflow の job は `needs` に入れられず、集合は変わらない。
- classifier: `.github/workflows/*` は `scripts/ci/classify-changes.sh:55` で full（`workflow=true` を含む）。分類の key は 9 つのままで、新しい key は要らない。
- Runner と費用: <https://docs.github.com/en/actions/reference/runners/github-hosted-runners> の「Standard GitHub-hosted runners for public repositories」の表に `windows-latest` がある。billing の頁は「The use of standard GitHub-hosted runners is free:」の第 1 に「In public repositories」（同日 WebFetch）。CI-PUBLIC-D1（`docs/ci.md:45`）の内。
- Cache Policy（`docs/ci.md:83-89`）: `target/` を保存しない、key は OS と `Cargo.lock` の hash。Windows の job は `runner.os` で別の key になり、Linux の 3 job の key を共有しない。
- helper は止まり、待たない: `scripts/pr-gate.py:1086` は `require(pr['mergeable'] is True and pr['mergeable_state'] == 'clean', 'PR not merge-clean')` で、その場で止まる（poll しない）。
- shell: workflow-syntax の頁（同日 WebFetch）は `pwsh` の行に「This is the default shell used on Windows.」、`bash` の行に `bash --noprofile --norc -eo pipefail {0}` と「When specifying a bash shell on Windows, the bash shell included with Git for Windows is used.」。step の `timeout-minutes` は「The maximum number of minutes to run the step before killing the process.」。step の時間切れに `continue-on-error` が効いて job が成功で終わるかは頁に書いてなく、hosted では未実測（Goal の保証の例外）。
- release の構成: `src-tauri/src/lib.rs:263` ほかに `#[cfg(debug_assertions)]`（`rg -n debug_assertions src-tauri/src`）、`src-tauri/src/main.rs:2` は `cfg_attr(not(debug_assertions), windows_subsystem = "windows")`。`docs/backlog.md:264` は開発用の Windows の Rust 1.100.0 の `cargo check --release` が出した `unused dependency tauri-specta` を `cargo::unused_dependencies` の warning と記録し、`-D warnings` が cargo 自身の lint に掛かるかを未確認とする（S5 の限界の根拠で、clippy の理由にしない）。`src-tauri/Cargo.toml:25` は `tauri = { version = "2", features = [] }` で `custom-protocol` を有効にしない。`[features]` の表は無い（`rg -n '^\[features\]' src-tauri/Cargo.toml` が 0 件）ので、ci.yml:116 の `--all-features` を付けなくても同じ feature の集合。
- Windows 専用の条件付きコンパイルの code は `src-tauri/src` の lib.rs・`io/register_sd.rs`・`mnt/restore.rs`・`mnt/offsite.rs`・`mnt/backup.rs` にある（`rg -l 'cfg\(windows\)' src-tauri/src`）。`windows-sys` は `src-tauri/Cargo.toml:48-49` の Windows の依存。`rusqlite` は `bundled`（`src-tauri/Cargo.toml:33`）で SQLite の system の依存は要らない。
- 既存の workflow の YAML の parse は `scripts/tests/run-workflow-tests.sh:8` が file を名指しで並べる（新しい file は足さないと検査の外）。
- 未実測: Windows の cold build と `cargo test` の時間、今の test が Windows で green か。

## Acceptance Criteria

- AC1: lane の PR（S1 を含む head）で `Windows observe` の run が `windows-latest` で走り、PR の checks に出る。summary に `cargo test` と `cargo clippy --release` の outcome と落ちた test の名（無ければ無いこと）がある。
- AC2: 同じ head の `ci.yml` の run の `Merge gate` が、この workflow の有無と無関係に今の `check-required-jobs.sh` で出る（`git diff origin/main -- .github/workflows/ci.yml scripts/ci/check-required-jobs.sh scripts/pr-gate.py` が空）。
- AC3: `python3 scripts/pr-gate.py status --pr N --packet docs/plans/2026-11-17-windows-test-observe.md` の blockers に Windows の run の話が出ない（run の前と後で同じ）。Coordinator が出力を PR の comment に残す。
- AC4: 2 つの head で観測し、それぞれ PR の comment に残す。(i) 意図的な失敗の回（Test Plan の「失敗の経路」）の head: (a) summary と annotation に `cargo test` の `failure` と合成の落ちた test の名が出る、(b) 後続の clippy と report の step が走る、(c) `Windows observe` の check run の conclusion が `success`、(d) 同じ head の `ci.yml` の `Merge gate`（`.github/workflows/ci.yml:258`）が成功。この回が未実施なら AC4 は未完了。(ii) revert の後の最後の head: Ready の後・helper の `merge` の前に REST の `gh api repos/higekiri-w/inventory-system-desktop/pulls/N --jq '.mergeable_state'` を取り、同じ head の `Windows observe` の check run が completed / success であることと一緒に残す。一時の Ready と、run の進行中の値の観測は求めない。check の成功から `clean` を推論しない。(ii) が `clean` でなければ実装を止め、計画を Gated Amendment で直す。
- AC5: 初回の結果（AC1 の run の `cargo test` の log から、green か、落ちた test の名と原因の分類〈Windows の挙動・test の前提・flaky の疑い〉）を PR の comment に残す。落ちた test は本 lane で直さず、backlog か別の lane の起票の候補として報告する。
- AC6: `docs/ci.md` の `## Windows Observe` 節に AC1 の run の job の時間（GitHub の job の実行時間の値と run の URL）があり、`未実測` の語が節に無い。昇格の条件が書いてある。
- AC7: `bash scripts/tests/ci-workflow.test.sh` と `bash scripts/tests/run-workflow-tests.sh` が PASS し、Test Plan の mutant を拒む。
- AC8: `bash scripts/doc-consistency-check.sh` と `--target plan` が ERROR なし、`bash scripts/check-workflow-git.sh` が OK。`rg -n "^## D-157" docs/decision-log.md` が 1 件。

## Design Readiness

- 引用する設計正本（節まで）: `docs/ci.md` の Hosted Trigger Model・Public Standard-Runner Policy（CI-PUBLIC-D1）・Cache Policy・Required Check Impact、`docs/agent-guidance/merge-evidence.md`（MG-D2 の required jobs）、`docs/DEV_WORKFLOW.md` Implementation Rules（`:140` の Writer の `cargo check --release`）。
- 必要な設計成果物: `docs/ci.md` の節と D-157 を本 PR で直す（updated in this PR、計画の固定の後）。
- plan にしかない durable な判断の昇格先: observe の実装の形と昇格の条件 → D-157 と `docs/ci.md`。
- 前提・制約と延期した design gap: `mergeable_state` が required でない check run の進行中をどう数えるかは観測しない（AC4 は完了の後の組だけ）。Windows の image と店の PC の差は L3 が持つ。E2E と `tauri build`（`custom-protocol`）は延期。step の時間切れは保証の例外（hosted では未実測）。
- 絶対保証の例外と抜け道: 主張しない。observe は Windows の失敗を見せるだけで、merge を止めない（見落としの抜け道は、Ready の承認を求める返答に結果を書く記録で狭める）。保証の例外は checkout・toolchain・cache の失敗と job の cancel で、helper の `merge` が止まり、job の再実行で戻す。cargo の step の時間切れも、hosted で確かめるまで例外に置く。
- 計画の review の答えの要旨: round 1（`538e90ac`）は step の `continue-on-error`・`paths`・止まって打ち直す運用の形を受け、保証の穴（shell・時間切れ・report の step）、結果の読み手、失敗の経路の観測、`cargo clippy --release` への置き換えを直させた。round 2（`b2477a09`）は AC4 の 2 段の観測、clippy の理由から backlog:264 を外すこと、step の時間切れを例外へ移すことを直させた。
- 判定: ready（closure round 3 の問いは round 2 の是正の 5 点）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| D-157（Windows observe） | `docs/ci.md` Windows Observe（新規）、decision-log D-157 | S1・S4・S5 | `ci-workflow.test.sh` の observe の検査と mutant | AC1・AC4・AC5・AC6 は lane の PR で観測 |
| MG-D2（required jobs） | merge-evidence MG-D2、`check-required-jobs.sh` | 変えない | 既存の `ci-workflow.test.sh`・`check-required-jobs` の検査 | AC2 |
| helper の CI 根拠と merge-clean | `docs/ci.md` Stale Green Prevention・Final Trigger Selection | 変えない | 既存の `pr-gate.test.py` | AC3・AC4 |
| CI-PUBLIC-D1 | `docs/ci.md` Public Standard-Runner Policy | 変えない（standard runner の内） | 既存の `validate_public_actions_doc_contract` | 非対象 |
| Cache Policy | `docs/ci.md` Cache Policy | S1・S4（OS の key の 1 句） | S3 の検査（`target` を cache の path に入れない） | 非対象 |
| npm 供給網ガード（D-030） | AGENTS.md | npm を使わない | S3 の検査（`npm` の語が無い） | 非対象 |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| observe の job が merge を止めない | `cargo test`・`cargo clippy --release` の step から `continue-on-error: true` が消え、job が失敗して `mergeable_state` が `clean` でなくなる | CLI（mutant） | `ci-workflow.test.sh` の `validate_windows_observe` と mutant `windows-observe-no-continue` | step の `continue-on-error` を消す |
| 失敗を見せる | 最後の step の `if: always()` か summary・`::warning` が消え、失敗が成功に見える | CLI（mutant） | 同 mutant `windows-observe-no-report` | report の step を消す・`if: always()` を外す |
| 落ちた test の名を残す | `cargo test` の出力を `tee` で log に取らない、report が `FAILED` の行を拾わない | CLI（mutant） | 同 mutant `windows-observe-no-tee` | `| tee` を消す・report の `FAILED` の抽出を消す |
| 既定の shell と step の時間切れ | `defaults.run.shell: bash` が消え `pwsh` で bash の書き方が落ちる、cargo の step の `timeout-minutes` が消えて job の時間切れ（cancel）になる | CLI（mutant） | 同 mutant `windows-observe-no-bash`・`windows-observe-no-step-timeout` | `shell: bash` を消す・step の `timeout-minutes` を消す |
| 失敗の経路 | 初回が green で、失敗した step・check run の結論・`Merge gate` の組を一度も見ない | 手の観測（Coordinator、PR の comment） | AC4 (i) の意図的な失敗の回 | 一時の commit の head で (a)〜(d) のどれかが出ない |
| `Merge gate` の外 | job の名が `Merge gate` になる、ci.yml に Windows の job が入る | CLI（mutant） | 同 mutant `windows-observe-merge-gate-name` と ci.yml に `windows-latest` が無いこと | 名を `Merge gate` にする、ci.yml に `windows-latest` を書く |
| 最小の権限・secret なし | `permissions` が `write` を含む、`secrets.` を参照する | CLI（mutant） | 同 mutant `windows-observe-write-permission`・`windows-observe-secret` | `contents: write` か `secrets.X` を足す |
| retry・skip をしない | retry の action や `cargo test -- --skip` が入る | CLI（mutant） | 同 mutant `windows-observe-skip` | `--skip` か retry を足す |
| Cache Policy | `target` を cache の path に入れる | CLI（mutant） | 同 mutant `windows-observe-target-cache` | cache の path に `target` を足す |
| YAML の形 | 新しい file が parse されない | CLI（既存の列に足す） | `run-workflow-tests.sh:8` の parse | file を列から落とす、YAML を壊す |
| 実 runner の振舞い | Windows で走らない・`mergeable_state` が `clean` にならない | 手の観測（Coordinator、PR の comment） | AC1・AC3・AC4 (ii) | run が出ない、revert の後の最後の head で Ready の後・`merge` の前の `mergeable_state` が `clean` でない |

意図的な失敗の回: 実装の commit の run（AC1）の後、lane の PR の上に一時の commit を 1 つ積み、`windows-observe.yml` の test の step の `run` の先頭に `echo 'test observe_probe::synthetic ... FAILED' | tee "$RUNNER_TEMP/cargo-test.log"; exit 1` の 1 行を足して step を必ず落とす（後続の `cargo test` の行は残す）。観測（AC4 (i) の (a)〜(d)）の後、その commit を戻す commit（`git revert`）を積む。`workflow_dispatch` の入力にしない理由: 既存の workflow に `inputs` の書き方が無く（`rg -n 'inputs' .github/workflows` が 0 件、ci.yml:4 と npm-security-monitor.yml:13 の `workflow_dispatch` は入力なし）、試すためだけの入力を残すと常設の足場になり（D-147）、`workflow_dispatch` の run は `pull_request` の trigger と別の経路で、PR の上の失敗の道と同じでない。一時の commit の head でも `ci.yml` の `Merge gate` が通る形にする（AC4 (i) の (d)。`ci-workflow.test.sh` の observe の検査を落とさない）。

実装の順: 計画の固定の後、S1〜S5 を 1 つの commit にし、owner が push する（Scope の push の順）。意図的な失敗の回の一時の commit と戻す commit も owner が push する。run の後に AC4〜AC6 の値で `docs/ci.md` の時間を書く commit は docs だけなので Coordinator が push する。
