# Plan Packet: Windows の runner で Rust の test を流す observe の job

wave に属さない単独の lane。出所は owner の決定（repo 外の回答台帳 TD-770、2026-11-17）: 記事 <https://claude.dev/blog/how-the-claude-code-team-thinks-about-verification/> の 3 種の test を始め、改訂が要る決定は改訂し、入れて有効性を確かめる。本 lane はそのうち「新しい gate は observe（出すだけ）から入れ、誤検知を先に洗う。flaky な test を retry・skip しない」の考えで、Windows でだけ compile・実行される Rust の code を hosted で流す。

decision-log の番号: 本 lane は D-157 を予約する。`docs/decision-log.md` の見出しの最後は `## D-156`（:1381）と `## D-155`（:1394）。手元の `origin/main` と `origin/agent/*` の 49 本の ref を `git grep -l -E 'D-15[7-9]' <refs> -- docs` で引くと、当たりは `docs/plans/2026-11-17-b-scope-sync-hands.md` の「D-155〜D-159 は 0 件」の文だけ（ref は本 run で fetch していない）。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R3

Reason:
新しい `.github/workflows/*` と `scripts/tests/*` は classifier が full（`workflow=true` を含む）、`docs/ci.md` は `docs=true workflow=true` を出す（`printf '%s\n' <path> | bash scripts/ci/classify-changes.sh --files-from-stdin`、2026-11-17）。hosted の CI と merge の helper の前提（`mergeable_state`）に触れる自動化で、R2 にできない。

## Goal

Goal Invariant:

### 最小完了条件

- `src-tauri/**` に触れる PR ごとに、`windows-latest` で `cargo test` と `cargo check --release` が走り、結果（成否と落ちた test の名）が PR の checks の画面から読める。
- その job は失敗しても `Merge gate` の合否・helper の `status` と `merge` の判定を変えない（observe）。走っている間だけ helper の `merge` が待つことを docs に書く。
- 初回の結果と時間を測り、`docs/ci.md` に書く（`未実測` のままにしない）。昇格（enforce）の条件が `docs/ci.md` にある。

### 失敗定義

- observe の job の失敗で helper の `merge` が止まる（`mergeable_state` が `clean` にならない）、または `Merge gate` が変わる。
- 失敗が PR の画面で見えない（成功に見えて中身が落ちている）、または retry・skip で失敗を消す。
- secret を使う・`permissions` が `contents: read` を超える・npm を使う。

### 非目的

- `Merge gate`・`scripts/pr-gate.py`・`.github/workflows/ci.yml`・`scripts/ci/check-required-jobs.sh` の変更、enforce への昇格。
- E2E（WebDriver）、Windows での clippy、frontend の build。
- Windows で落ちた test の修正（見つかれば別の lane。本 lane は名と原因の分類を PR の comment に残すまで）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

- S1 `.github/workflows/windows-observe.yml`（新規）:
  - `name: Windows observe`。job の名は `Merge gate` にしない（ruleset が要る context と同じ名を持たせない）。
  - `on`: `pull_request`（`branches: [main]`、`types: [opened, reopened, synchronize]`、`paths: ['src-tauri/**', '.github/workflows/windows-observe.yml']`）と `workflow_dispatch`。ci.yml の trigger（`.github/workflows/ci.yml:3-7`）に合わせ、`paths` だけを足す（required でない workflow なので、docs だけの PR で cold build と helper の待ちを起こさない）。
  - `permissions: contents: read`、`concurrency: group: windows-observe-${{ github.event.pull_request.number || github.ref }}`・`cancel-in-progress: true`（ci.yml:9-14 と同じ形で group の名だけ変える）、`env` は ci.yml:16-18 と同じ 2 行。
  - job 1 本: `runs-on: windows-latest`、`timeout-minutes: 60`（hang で runner を 6 時間占めない）、`defaults.run.working-directory: src-tauri`。steps は `actions/checkout@v6` → `dtolnay/rust-toolchain@stable` → `actions/cache@v5`（ci.yml の Rust の job と同じ 3 path・key・restore key。key は `runner.os` で Windows に分かれる）→ `cargo test`（`id: test`、`continue-on-error: true`）→ `cargo check --release`（`id: release`、`continue-on-error: true`。`src-tauri/src/main.rs:2` の `windows_subsystem` は Windows の release でだけ効く）→ 最後の step（`if: always()`）が 2 つの `steps.<id>.outcome` を `$GITHUB_STEP_SUMMARY` に表で書き、`failure` があれば `::warning title=Windows observe::<step> failed` を出す。retry しない。
  - 秘密を使わない（`secrets.` も `GITHUB_TOKEN` の env も書かない）。npm・Node を使わない。
- S2 `scripts/tests/run-workflow-tests.sh:8`: YAML の parse の列に `.github/workflows/windows-observe.yml` を足す。
- S3 `scripts/tests/ci-workflow.test.sh`: observe の契約を確かめる関数を 1 つ足し、mutant で拒むことを確かめる（Test Plan）。
- S4 `docs/ci.md`: `## Windows Observe` 節を足す（目的、observe であること、`Merge gate` の外である根拠、走っている間 helper の `merge` が待つこと、初回の時間、昇格の条件）。Cache Policy の「Rust 3 job」の行（`docs/ci.md:88`）に、Windows の job は OS の key で別の cache を持つことを 1 句で足す。CI-PUBLIC-D1 は変えない（`windows-latest` は public の repository の standard runner、下の事実）。
- S5 `docs/decision-log.md`: 末尾に D-157（Status: owner の決定 TD-770、本 packet で起票。Context: hosted は全 job が `ubuntu-latest` で `cfg(windows)` の 29 か所を compile しない、記事の observe の考え。Decision: S1 の形、observe を step の `continue-on-error` で実装する理由〈helper の `mergeable_state == 'clean'`〉、昇格は別の R3。Alternatives: ci.yml に job を足す〈`check-required-jobs.sh` と `Merge gate` が変わる〉、job の `continue-on-error`〈check の結論が失敗のまま残るかを確かめていない〉、schedule だけ〈PR の L3 より前に見えない〉、paths を付けない〈docs の PR でも cold build と待ち〉。Guarantee range: Windows の runner の image と店の PC は同じでない。L3 を置き換えない。Revisit: 昇格の条件を満たしたとき、誤検知が出たとき、待ちが merge の運用を妨げたとき）。

Non-scope: `.github/workflows/ci.yml`、`scripts/pr-gate.py`、`scripts/ci/check-required-jobs.sh`、`scripts/ci/classify-changes.sh`、`.github/merge-gate-ruleset.json`、`src-tauri/**`。

push の順（`.local/checklists/env-facts.md` の E9: sandbox の token は Workflows の権限を持たない）: 計画の commit と `plan-approve.sh` の commit は Coordinator が push する。S1〜S5 の実装の commit は `.github/workflows/**` を含むので、owner が owner の terminal で push する（Coordinator は push の command を返答に inline で渡す）。

## 確かめた事実（2026-11-17）

- helper は `ci.yml` の run だけを読む: `scripts/pr-gate.py:935` の API は `actions/workflows/ci.yml/runs`、`:939` は `run['path'] == '.github/workflows/ci.yml'`、`:942-945` はその check suite の中の `Merge gate` だけを数える。別の workflow の run は `ci()` に入らない。
- ただし helper の `merge` は `scripts/pr-gate.py:1086` で `pr['mergeable_state'] == 'clean'` を要る。GitHub の GraphQL の `MergeStateStatus`（<https://docs.github.com/en/graphql/reference/pulls>、2026-11-17 に WebFetch）は `CLEAN`「Mergeable and passing commit status.」、`UNSTABLE`「Mergeable with non-passing commit status.」。required でない check が失敗・進行中でも `clean` でなくなると読める（check run が commit status と同じに数えられるかは未確認で、AC4 で lane の PR で観測する）。発注の前提（別の workflow なら helper に触れない）はこの点で足りず、S1 は job の結論を成功に保つ（step の `continue-on-error`）。step の `continue-on-error` は「Prevents a job from failing when a step fails. Set to `true` to allow a job to pass when this step fails.」（<https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax>、同日 WebFetch）。job の `continue-on-error` の check の結論は確かめていない。
- `Merge gate` の required jobs: `scripts/ci/check-required-jobs.sh:17` の `required` の集合と `:26` の `set(needs) != set(required)` は ci.yml の `merge_gate` の `needs`（`.github/workflows/ci.yml:259`）だけを見る。別の workflow の job は `needs` に入れられず、集合は変わらない。
- classifier: `.github/workflows/*` は `scripts/ci/classify-changes.sh:55` で full（`workflow=true` を含む）。分類の key は 9 つのままで、新しい key は要らない。
- Runner と費用: <https://docs.github.com/en/actions/reference/runners/github-hosted-runners> の「Standard GitHub-hosted runners for public repositories」の表に `windows-latest` がある。billing の頁は「The use of standard GitHub-hosted runners is free:」の第 1 に「In public repositories」（同日 WebFetch）。CI-PUBLIC-D1（`docs/ci.md:45`）の内。
- Cache Policy（`docs/ci.md:83-89`）: `target/` を保存しない、key は OS と `Cargo.lock` の hash。Windows の job は `runner.os` で別の key になり、Linux の 3 job の key を共有しない。
- `cfg(windows)` は `src-tauri/src` に 29 か所（`grep -rn -c` で lib.rs 2・register_sd.rs 6・restore.rs 9・offsite.rs 8・backup.rs 4）、`windows-sys` は `src-tauri/Cargo.toml:48-49` の Windows の依存。`rusqlite` は `bundled`（`src-tauri/Cargo.toml:33`）で SQLite の system の依存は要らない。
- 既存の workflow の YAML の parse は `scripts/tests/run-workflow-tests.sh:8` が file を名指しで並べる（新しい file は足さないと検査の外）。
- 未実測: Windows の cold build と `cargo test` の時間、今の test が Windows で green か。

## Acceptance Criteria

- AC1: lane の PR（S1 を含む head）で `Windows observe` の run が `windows-latest` で走り、PR の checks に出る。summary に `cargo test` と `cargo check --release` の outcome の表がある。
- AC2: 同じ head の `ci.yml` の run の `Merge gate` が、この workflow の有無と無関係に今の `check-required-jobs.sh` で出る（`git diff origin/main -- .github/workflows/ci.yml scripts/ci/check-required-jobs.sh scripts/pr-gate.py` が空）。
- AC3: `python3 scripts/pr-gate.py status --pr N --packet docs/plans/2026-11-17-windows-test-observe.md` の blockers に Windows の run の話が出ない（run の前と後で同じ）。Coordinator が出力を PR の comment に残す。
- AC4: Windows の run の進行中と完了の後に `gh api repos/higekiri-w/inventory-system-desktop/pulls/N --jq '.mergeable_state'` を引き、値を PR の comment に残す。完了の後（test が落ちていても）が `clean`（Draft の間は `draft` が返るなら、Ready の後の値で見る）。`clean` にならなければ実装を止め、計画を Gated Amendment で直す。
- AC5: 初回の結果（AC1 の run の `cargo test` の log から、green か、落ちた test の名と原因の分類〈Windows の挙動・test の前提・flaky の疑い〉）を PR の comment に残す。落ちた test は本 lane で直さず、backlog か別の lane の起票の候補として報告する。
- AC6: `docs/ci.md` の `## Windows Observe` 節に AC1 の run の job の時間（GitHub の job の実行時間の値と run の URL）があり、`未実測` の語が節に無い。昇格の条件が書いてある。
- AC7: `bash scripts/tests/ci-workflow.test.sh` と `bash scripts/tests/run-workflow-tests.sh` が PASS し、Test Plan の mutant を拒む。
- AC8: `bash scripts/doc-consistency-check.sh` と `--target plan` が ERROR なし、`bash scripts/check-workflow-git.sh` が OK。`rg -n "^## D-157" docs/decision-log.md` が 1 件。

## Design Readiness

- 引用する設計正本（節まで）: `docs/ci.md` の Hosted Trigger Model・Public Standard-Runner Policy（CI-PUBLIC-D1）・Cache Policy・Required Check Impact、`docs/agent-guidance/merge-evidence.md`（MG-D2 の required jobs）、`docs/DEV_WORKFLOW.md` Implementation Rules（`cargo check --release`）。
- 必要な設計成果物: `docs/ci.md` の節と D-157 を本 PR で直す（updated in this PR、計画の固定の後）。
- plan にしかない durable な判断の昇格先: observe の実装の形と昇格の条件 → D-157 と `docs/ci.md`。
- 前提・制約と延期した design gap: `mergeable_state` が required でない check run の失敗・進行中をどう数えるかは AC4 で観測する。Windows の image と店の PC の差は L3 が持つ。clippy と E2E は延期。
- 絶対保証の例外と抜け道: 主張しない。observe は Windows の失敗を見せるだけで、merge を止めない（見落とせば merge される）。
- 計画の review に問う点:
  1. observe を step の `continue-on-error` と summary・warning で実装する形は、失敗を見せることと helper の `merge` を止めないことを両方満たすか。job の `continue-on-error` の方が見えやすく、`mergeable_state` も `clean` に保つなら、そちらを採るべきか。
  2. `paths` の filter（`src-tauri/**` と自身）は Goal に足りるか（`Cargo.lock` は `src-tauri` の下）。ci.yml が path filter を廃した理由（required の `Merge gate` を必ず出す）は required でないこの workflow に当たらない、という読みは正しいか。
  3. 進行中の Windows の run で helper の `merge` が `PR not merge-clean` で止まる待ちを受けてよいか（受けないなら trigger を変える）。
  4. 昇格の条件の案: Windows で落ちた test を別の lane で直した後、`src-tauri/**` に触れる PR の連続 5 run で誤検知 0（candidate。数は owner の判断）。昇格は ci.yml・`check-required-jobs.sh`・`Merge gate` を変える別の R3。
  5. `cargo check --release` を入れる理由（`main.rs:2` の Windows の release だけの属性）は足りるか。clippy を外す判断は正しいか。
  6. この変更で不要になる記述・条件は無いか（例: `docs/DEV_WORKFLOW.md:140` の Writer の `cargo check --release` は、observe の段では残す）。
- 判定: ready（問い 1〜3 の答えで S1 の trigger と実装の形が変わりうる）。

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
| observe の job が merge を止めない | `cargo test`・`cargo check --release` の step から `continue-on-error: true` が消え、job が失敗して `mergeable_state` が `clean` でなくなる | CLI（mutant） | `ci-workflow.test.sh` の `validate_windows_observe` と mutant `windows-observe-no-continue` | step の `continue-on-error` を消す |
| 失敗を見せる | 最後の step の `if: always()` か summary・`::warning` が消え、失敗が成功に見える | CLI（mutant） | 同 mutant `windows-observe-no-report` | report の step を消す・`if: always()` を外す |
| `Merge gate` の外 | job の名が `Merge gate` になる、ci.yml に Windows の job が入る | CLI（mutant） | 同 mutant `windows-observe-merge-gate-name` と ci.yml に `windows-latest` が無いこと | 名を `Merge gate` にする、ci.yml に `windows-latest` を書く |
| 最小の権限・secret なし | `permissions` が `write` を含む、`secrets.` を参照する | CLI（mutant） | 同 mutant `windows-observe-write-permission`・`windows-observe-secret` | `contents: write` か `secrets.X` を足す |
| retry・skip をしない | retry の action や `cargo test -- --skip` が入る | CLI（mutant） | 同 mutant `windows-observe-skip` | `--skip` か retry を足す |
| Cache Policy | `target` を cache の path に入れる | CLI（mutant） | 同 mutant `windows-observe-target-cache` | cache の path に `target` を足す |
| YAML の形 | 新しい file が parse されない | CLI（既存の列に足す） | `run-workflow-tests.sh:8` の parse | file を列から落とす、YAML を壊す |
| 実 runner の振舞い | Windows で走らない・`mergeable_state` が `clean` にならない | 手の観測（Coordinator、PR の comment） | AC1・AC3・AC4 | run が出ない、完了の後も `clean` でない |

実装の順: 計画の固定の後、S1〜S5 を 1 つの commit にし、owner が push する（Scope の push の順）。run の後に AC4〜AC6 の値で `docs/ci.md` の時間を書く commit は docs だけなので Coordinator が push する。
