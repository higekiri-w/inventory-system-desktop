# CI

CIとmerge evidenceの正本。設計理由とwire契約は [merge-evidence.md](agent-guidance/merge-evidence.md)（MG-D1〜D14）。

## 現行契約

GitHubはmainへのPRとCIを強制し、helperはreview/manual/R4を確認する。直接UI merge・確認を飛ばす直接gh mergeは禁止し、GitHub UIがCI成功だけでmerge可能と示し得る残存リスクを受容した契約であって、非CI結果をserverが強制するとは主張しない。

## Verification Ladder

| 層 | 目的 | 証跡 |
|---|---|---|
| L0 pre-push | push増分のfeedback、Ready push拒否 | `.local/quality-check.log` |
| L1 local full（任意） | hosted と同じ gate 一式を手元で回す道具。merge の条件にしない | `.local/ci-evidence/` の log |
| L2 hosted | push ごとの run（Draft を含む）が必要 job の実成功を `Merge gate` へ集約する。Ready で回し直さず、同じ head の run を merge の根拠にする（D-145 (9)） | GitHub の対象 workflow / check |

hostedを最終CI根拠にし、一律のlocal fullやPR本文へのSHA転記をmergeの条件にしない。localで観測した失敗、Windows L3・manual/R4の保護は維持する。

## Hosted Trigger Model

`pull_request`のopened/reopened/synchronize（base=main）と`workflow_dispatch`だけを使う。Ready にしたときは回し直さない（同じ head の CI を 1 回にする。TD-481、D-145 (9)）。pushやbody編集では起動しない。全docsをevent対象とし、path filterと本文skip tokenは廃止した。Draft でも changes と changed 分類の job と aggregate が走り、aggregate は常に `Merge gate` を出す。`pull_request` の event では Draft と Ready で選ぶ job が同じ（`draft` を読むのは消した name 式だけだった）。

aggregate の if は `always()` だけ。changes成功条件をifへ追加すると分類失敗でaggregateがskipされるため禁止。`check-required-jobs.sh`が分類key/値と全jobを検査し、選択されたjobはsuccessだけ、非対象jobはsuccess/skippedを受理する。failure/cancelled/欠落/未知値は拒否する。CI tokenはcontents:read。

### Final Trigger Selection

CI-TRIGGER-D1: 同じHEADへ予防的なdispatchを重ねない。dispatchは常にfull。同じ head の CI は 1 回（D-145 (9)）。

final run = 同じ head の最新の run（`pull_request`〈Draft を含む〉か `workflow_dispatch`）が completed / success で、その check suite に GitHub Actions の `Merge gate` が success でちょうど 1 つある run（`scripts/pr-gate.py` の `Gate.ci`）。

| HEADの状態 | 選ぶtrigger | dispatch前の確認 |
|---|---|---|
| docsを含む全PR | push（opened・synchronize）、再開はreopened。Ready では回し直さない | dispatch しない |
| required final の自動 run または explicit dispatch が作成されない、失敗、または cancel | 原因是正後のrecovery dispatch | 同一 HEAD に、helper が受理する final run も、進行中の run も無いこと |
| 同一 HEAD に helper が受理する final run が既にある | 既存runを使う | Ready再操作もdispatchも不要 |

base付替え等でrunが無い場合も、同一 HEAD に final run が無いこと、または run の失敗/cancel を確認してからrecoveryを選ぶ。

## Risk Routing

一般docsはdocs＋Merge gate。policy docsはdocs＋workflow回帰＋Merge gate。実行制御、tests、未知pathは全gate。Rust/frontend/env/generated/traceabilityの既存分類は保持する。RiskはCI範囲を縮めず、owner/modelが意味から判断する。R3/R4 の計画の固定、独立review、必要manual/R4はCI成功で代替しない。

## Public Standard-Runner Policy

CI-PUBLIC-D1: この repository は public で、現行 workflow は standard GitHub-hosted runner のみを使う。[GitHub Actions billing](https://docs.github.com/en/billing/concepts/product-billing/github-actions) と [job execution time](https://docs.github.com/en/actions/how-tos/monitor-workflows/view-job-execution-time) の公式 contract 上、この組み合わせでは Actions の billable execution minutes は発生しないため、private repository 時代の月間 minute 使用率や reset 日を hosted gate の判断条件にしない。larger runner を将来導入する場合は public repository でも課金対象になるため、別の R3 change で費用・security boundary・routing を再設計する。

free minutes と無関係に、runner / Actions service の障害、concurrency、cache、重複実行には運用コストがある。CI-TRIGGER-D1 の 1 HEAD 1 final、Draft 中の hosted と L0 での事前検証、失敗原因を直してからの recovery を維持する。

GitHub Actions が利用不能なら merge を停止し、許可済みの local 作業と証跡を保存する。

## Local Commands

```bash
bash scripts/local-ci.sh changed
bash scripts/local-ci.sh full  # 任意
bash scripts/tests/run-workflow-tests.sh
```

changedはorigin/main（なければmain）とのmerge-baseからPR全差分を分類する。push増分とは異なる。fullはdocs、実履歴PK5、shell syntax、workflow YAML、workflow回帰、Rust fmt/clippy/tests、bindings、traceability、frontend install/routes/typecheck/lint/format/tests/build、envを保持する。npm auditは従来どおりwarn-only。fullは`node_modules`がsymlinkならgateの前に失敗する（`npm ci`がsymlink先を空にするため）。changedは検査しない。

local fullとhosted workflow jobは同じ`run-workflow-tests.sh`を呼び、検証一覧を複製しない。hosted docs jobは全経路でfetch-depth: 0、実PR head/baseを用いてPK5・先走りの検査を実行し、doc check にも `WORKFLOW_BASE_SHA`（PR の base）を渡す（active な packet と新しい path:行 の起点、D-145 (5)(10)）。workflow job も fetch-depth: 0（suite の実 repo の self-pass が起点を要る）。浅い履歴や解決不能baseは成功にしない。Node pinとRuby/ripgrep/Pythonを用意する。

local evidenceは開始/終了のHEADとtree状態を保存し、gate中のHEAD更新・CLEANからDIRTYへの変更を失敗にする。DIRTYは診断用。localでの失敗を無視しない。

## Classifier Contract

`classify-changes.sh`の既存9key（rust/rust_drift/frontend/docs/env/generated/traceability/workflow/unknown）を共有する。workflow=trueはworkflow回帰の意味だけで、consumerがRust/frontendへ再昇格させない。詳細path表はMG-D4。rename/copyの両pathと削除を分類し、未知path/比較不能はfull fallback。

## Pre-push Contract

pre-pushは実際のpush先remote_refのReady状態を確認し、Ready pushと照会失敗を拒否する。修正はDraftへ戻す。push先のbranchのPRがopenに無くmergedにあればpushを拒否し、mainから新しいbranchを案内する。pushするfileに未stageの変更があればWARNを出してpushを続ける。Rust、docs、env、traceability、frontend routes/typecheck/lintとPK5・先走りの検査を維持する。修正後は対象検証・改版record・hosted finalを使う。

Rustまたはtraceability分類では`cargo run --bin generate_traceability -- --check`を1回実行し、T1/T2/T4のERRORをpush拒否、T3を従来どおりWARNとして扱う。全Rust関数名へ`_reqNNN`だけを要求する重複検査は使わない。REQ対象の名前規約と、技術/workflowテストの適用SPEC・設計IDはreviewで確認し、機械検査の成功を全テストの仕様対応の証明にしない。

緊急bypassは既存の固定token（owner-approved/tooling-unavailable/incident-response）でhookに記録する。raw no-verifyは使わない。これはGitHub rulesetのbypassやhelperを省略する許可ではない。

## Stale Green Prevention

正規経路は`python3 scripts/pr-gate.py status|capture|record|ready|merge --pr NUMBER`。helperはPRのbaseと同じ版でだけ動く（違えば`helper differs from base`でbaseの版を使うcommandを示して止まる）。R3/R4は`--packet`、packet なしの道（R0〜R2）は`--risk`と`--manual required|not-required`を明示し、PR body の Risk の行と照合する。capture/server/head/baseが変われば記録し直す。Ready/mergeはowner指示の後、helperがfreshなPR・record・rules・CIを確認する。mergeはmatch-headとstrictを使い、admin fallbackはない。

Ready後の修正はDraftへ戻し、旧greenを流用しない。base同期時のclosureとmanualの使い回しは[merge-evidence](agent-guidance/merge-evidence.md)のMG-D13、Actions停止時の扱いは[merge-evidence](agent-guidance/merge-evidence.md)「Actions停止時」に従う。

## Cache Policy

- actions/cache は `~/.cargo/registry/index/`、`~/.cargo/registry/cache/`、`~/.cargo/git/db/` の依存取得 cache だけを保存する。
- `src-tauri/target/` と `~/.cargo/bin/` は保存しない。
- key は OS + `src-tauri/Cargo.lock` hash、restore key は OS prefix を維持する。
- Rust 3 job は同じ immutable key を使う（Windows observe の job は `runner.os` の key で別の cache を持つ）。cold miss 時は first writer が保存し、他 job の同一 key save 競合は warning として受容する。job 別 key に分けて同じ依存 cache を三重保存しない。
- npm は既存 `actions/setup-node` の `cache: npm` を維持し、`node_modules/` は保存しない。
- 10GB 上限再到達時は cache usage と key 数を確認し、target 等の build output を再追加しない。

## Required Check Impact

requiredはGitHub Actions/15368由来の`Merge gate`。strict、PR必須、bypassなし、削除/force禁止を`.github/merge-gate-ruleset.json`に定義する。helperはcurrent main側のpolicyと実効rulesを照合し、PRが自分の必要reviewやpolicyを緩めない。設定はhelperから変更しない。

## Windows Observe

D-157。ci.yml の job は全部 `ubuntu-latest` で、`cfg(windows)` の code を compile しない。`.github/workflows/windows-observe.yml` は `src-tauri/**` か自身に触れる PR（と `workflow_dispatch`）で、`windows-latest` の上で `cargo test` と `cargo clippy --release --all-targets -- -D warnings` を流す。

- observe（出すだけ）: 2 つの cargo の step は `continue-on-error: true` で、job は成功で終わる。結果（2 つの step の outcome と落ちた test の名）は run の summary と `::warning` の annotation に出す。retry・skip しない。
- `Merge gate` の外: 別の workflow なので ci.yml の `merge_gate` の `needs` と `check-required-jobs.sh` の集合に入らず、helper の CI 根拠（ci.yml の run だけを読む）も変わらない。path filter はこの workflow だけに置く（required でないので、docs だけの PR で cold build と helper の止まりを起こさない）。
- 結果の読み手: `src-tauri/**` に触れる PR では、Coordinator が Ready の承認を求める返答に Windows observe の outcome と落ちた test の名を書く。昇格の数え方はその記録で行う。
- helper の `merge`: 要るのは `mergeable_state == 'clean'`。run の進行中は helper の `merge` が止まりうる（未観測）。止まったら同じ head で完了の後に打ち直す（承認の取り直しも同期も要らない）。
- 保証の例外: checkout・toolchain・cache の step の失敗と job の cancel（job の `timeout-minutes` の時間切れを含む）では job が失敗・cancel で終わり、helper の `merge` が止まる。戻し方は job の再実行で、落ちたのは test でないので test の retry ではない。cargo の step の時間切れ（step の `timeout-minutes`）も例外に置く。step の時間切れは、実際の時間切れを hosted で観測したら observe の側に戻す。
- 初回の結果と時間: 初回（2026-11-17、PR #273 の run <https://github.com/higekiri-w/inventory-system-desktop/actions/runs/44076654226>）は `cargo test`（lib の unittests の実行 file が Windows で起動しない）と `cargo clippy --release`（Windows でだけ出る 3 件）がどちらも failure で、job の実行時間は 439 秒（cold build）。clippy の 3 件は PR #281、lib の test の起動は D-159 で直した。D-159 の lane の run（2026-11-19、PR #286 の <https://github.com/higekiri-w/inventory-system-desktop/actions/runs/44504745664>）では、`cargo test` は lib の unittests が起動して 53 本が落ちた。PR #287 の後の run（<https://github.com/higekiri-w/inventory-system-desktop/actions/runs/44523246270>）では 60 本で、どれも test の前提だった: A 7 本は source を `\n` で探すのに Windows の checkout が CRLF（`.gitattributes` が無い）、B 48 本は Windows の媒体の root の選び方が test の注入した root の列を使わず `NotRemovable`、C 2 本は `pin_to_volume` が返す Volume GUID の形を test が drive 文字の形で期待・注入、D 3 本は file 名の `\`・SD の folder の綴り・B12。B12 は `Recovered("main の退避に失敗: … (os error 32)")` で、test が開いたままにした別の接続の下で main の改名が共有違反で失敗した（元の DB の 2 行が戻り、遺物なし。MNT-01-D4 の安全側。<https://github.com/higekiri-w/inventory-system-desktop/actions/runs/44562295748>）。PR #291 で直した（A は D-161、B は §71.11.2 の root の選び方を両 OS で共通にした）。job は observe で success。
- 昇格（enforce）の条件: Ready の承認を求める返答の記録で、`src-tauri/**` に触れる PR が続けて 10 本（owner の決定、2026-11-17、repo 外の回答台帳 TD-783）、誤検知（Windows の挙動でも test の前提の誤りでもない失敗、flaky の疑い）なしで通り、見つかった Windows の失敗が直っていること。昇格は別の R3 で、`Merge gate` と `check-required-jobs.sh` の変え方を計画する。

## Related Records

[DEV_WORKFLOW](DEV_WORKFLOW.md)、[merge evidence設計とrollout](agent-guidance/merge-evidence.md)、[D-033/D-063と後継判断](decision-log.md)。初回Actions Enableと旧CIの実績はarchive/過去PRの記録を参照する。
