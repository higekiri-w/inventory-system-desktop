# Plan Packet: ハーネスの残りの小口を整理する（R2）

wave に属さない単独の lane。2026-09-30 起草。出典は [backlog](../../backlog.md) の「ハーネスの残りの小口の整理」（`docs/backlog.md:87`、ハーネス刷新 PR4・PR5 の closeout で挙がった 4 件）。owner 決定 2026-09-30: 4 件を R2 の 1 本にする（規則の意味を変えるので、Risk Tiers `docs/DEV_WORKFLOW.md:48` の R2「Local developer workflow … docs change」に当たり、backlog の「R0/R1 で 1 本」は起票時の見積もり違い）。同日の owner 回答: Windows の Codex Desktop はこの repo では多分使わない（個人の別 repo では使う）。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

実装後の状態はPR native state / 専用record / CIが所有し、trackedに書かない。

- Phase: archive
- Risk: R2
- Plan Commit: 842f7ab8790887808e8e3aede3f995ed1050d9f0
- Amendments: none
- Coordinator: Opus 5.5（Claude Code main session、effort high）
- Writer: Opus 5.5 subagent（`subagent_type: writer`）
- Plan Reviewer: fresh Opus 5.5（`subagent_type: reviewer`）+ Codex GPT-6.1 Sol / high
- Final Reviewer: fresh Opus 5.5 + Codex GPT-6.1 Sol / high
- Final Review Minimum: 2
- Human Gate: ready,merge
- Branch: agent/harness-small-cleanup

座組は `docs/AGENT_OPERATING_MANUAL.md` `## 座組`（確認日 2026-09-30）に照合した。Final Reviewer の Claude 側は R2 で design lane でないため fresh Opus 5.5。Final Review Minimum 2 は、Scope の `AGENTS.md`・`docs/AGENT_OPERATING_MANUAL.md`・`docs/DEV_WORKFLOW.md`・`docs/agent-guidance/*` が `scripts/ci/classify-changes.sh:59` の policy docs（workflow=true）に当たるため（`docs/DEV_WORKFLOW.md:84`）。manual なし: 製品 runtime・画面・配布物への変化がない文書変更で、Windows native の確認対象がない。

遷移記録（append-only）:

- kickoff → spec-check（2026-09-30、本 commit）: 対象を S1〜S4 に限定し、Risk を R2 と記録した。
- spec-check → plan-draft（2026-09-30、本 commit）: 唯一の許された skip。設計正本が workflow 文書自身（`AGENTS.md`・`docs/DEV_WORKFLOW.md`・`docs/AGENT_OPERATING_MANUAL.md`・`docs/agent-guidance/README.md`）で、その改訂が本 lane の Scope そのもの。Design Readiness を参照。
- plan-draft → plan-gate（2026-09-30、本 commit）: packet を plan-first commit で確定。R2 のため Test Design Matrix は作らない（下の Test Plan）。fresh Opus 5.5 と Codex の Plan Review へ。
- plan-gate → design（2026-09-30、round 1 の後の是正）: round 1 の Codex #2（P2）が「本 lane は既存の正本に無い規則（`Plans.md` に残す件数・entry の形・移送先、新しい §5.4、profiles・evals の撤去）を決めるので、`docs/DEV_WORKFLOW.md:96` の skip の条件〈既存の設計正本で十分〉に当たらず、`:131` の表は Plan より前に decision-log か ADR を求める」と示し、上の spec-check → plan-draft の skip を取り消して design に戻る。設計を無効にする指摘は最初に影響を受ける phase に戻り表をたどり直す（`docs/DEV_WORKFLOW.md:104`）。
- design → plan-draft → plan-gate（2026-09-30、round 1 の後）: 設計の出力が同じ plan-first の変更の中にある（`docs/DEV_WORKFLOW.md:97`）= 本 commit の `docs/decision-log.md` D-101 と、本 packet の設計判断 D1〜D5。未解決の設計の問いは無い。packet と D-101 を同じ commit に置き、1 つの commit で隣接する遷移を記録する（`docs/DEV_WORKFLOW.md:103`）。round 1 の他の是正は Review Response 参照。
- plan-gate → plan-approved（2026-10-01、Coordinator、本 commit）: 上限の Plan Review round 3（対象 `a8e3bb17`）の後、round 天井の disposition「同型指摘の一括是正」（`842f7ab8`）を Coordinator が現物で確かめた（doc check full・`--target plan`・PK5・`scripts/tests/reading-order-drift.test.sh` が exit 0）。一括是正の独立の再確認は Final Review が兼ねる。owner の plan-approved の判断（2026-10-01）。Plan Commit = `842f7ab8`。
- plan-approved → implementing（2026-10-01、Coordinator、state-only、本 commit）: Writer（Opus 5.5 subagent、`subagent_type: writer`）へ実装を発注する。発注の直前に AC-S1-1〜AC-S4-4・AC-C-4 の baseline を逐語で再実行し、上の baseline と一致した。

## Owner Effort Budget

- 介入回数上限: 6（既定 6）
- 実働時間上限: 30 分（既定 30 分）
- Plan Review round 天井: 3（既定 3）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 6 | 3（2026-09-30: #5 の回答 1、組み方の判断 1。2026-10-01: plan-approved 1〈予備から〉） | 2（Ready 1、merge 1） | 1 | 6 = 3 + 2 + 1 |

relay（Coordinator が Codex を起動する往復）は上限を置かない（D-098）。
既定値・数え方・上限に届くときの扱いは `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R2

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers を当てた。変更は毎 session 読まれる運用文書（`AGENTS.md`・`docs/Plans.md`）、reviewer 発注の規則（MANUAL §5.4）、参照されない補助文書（agent-guidance の profiles・evals）の改訂と削除で、runtime 契約・DB・CSV/TSV・command・route/search・画面の振舞いを変えない。R3 の「merge gate changes」に当たらない理由: helper（`scripts/pr-gate.py`）・classifier（`scripts/ci/classify-changes.sh`）・`.github/workflows/ci.yml`・pre-push・checker（`scripts/doc-consistency-check.sh`）・`scripts/tests/*` はいずれも触らず、required gate の green / red は変わらない（`.codex/bin/*-safe-*.sh` と `scripts/tests/codex-safe-wrappers.test.sh` を残す裁定は下の Scope S2）。R2 の「Local developer workflow … docs change that affects maintainability」に当たる。R0/R1 でない理由: S1〜S3 は規則の意味（closeout の書き方、Workspace Access の適用条件、reviewer 発注の形）を変える。tie-breaker（`docs/DEV_WORKFLOW.md:52`「If uncertain between R2 and R3, choose R3 when the change touches a stable contract, output schema, data safety boundary, command wire shape, UI route/search behavior, or workflow gate」）に当てると、本 lane は merge gate・helper・classifier・hosted CI・PK の合否を変えず（AC-S2-2 が `.codex`・wrapper test・`ci.yml`・classifier の差分 0 を固定し、helper・checker は Non-scope）、S3 は発注書の書き方の規則で、Plan Review の判定の義務（3 値の答え、2 つの問いを分ける）は既存の `docs/DEV_WORKFLOW.md:270-271` にあるものを写すだけ（新 §5.4 の 3 値の問いは Plan Review の発注に限り、Final Review の発注に広げない）なので、workflow gate に触れない。「workflow gate」の 2 つの読み方: `docs/DEV_WORKFLOW.md:84` の括弧（workflow gate changes = classifier `workflow=true`）は Final Review Minimum を 2 にする trigger として当て、上の tie-breaker（`:52`）の workflow gate は merge gate・helper・classifier・hosted CI・PK の合否の意味で読む。本 lane は前者に当たり（Workflow State の Final Review Minimum 2 の理由）、後者に当たらない。前例の R3（PR #113〈`docs/Plans.md:37`、D-092〉の独立性と Final Review の合否の規則、D-090〈`docs/decision-log.md:738`〉の Plan Review の問いの追加）は review の義務・判定そのものを変えた点で本 lane と異なる。owner の R2 の決定（2026-09-30、冒頭）は claim として扱い、根拠は上の当てはめ。

## Goal

Goal Invariant:

### 最小完了条件

- 毎 session 読まれる文書が今の運用と合い、短くなる: `AGENTS.md` の Workspace Access は wrapper を使う条件（Windows の Codex Desktop から動かすとき）を明示し、`docs/Plans.md` の `## 直近の完了` は「1 行 + link」の entry が上限件数以内で、main の SHA を含まない。
- reviewer 発注の規則が今の発注の形と逆向きでない: `docs/AGENT_OPERATING_MANUAL.md` §5.4 が「読むもの・読まないもの・判定の問い・確かめる command を具体に書く」側に立ち、「観点 list・必読順・検証 command の指定は書かない」の文が無い。
- 参照されない `docs/agent-guidance/profiles/` と `evals/` が無く、README・shared・model-notes に切れた参照が残らない。

### 失敗定義

- `docs/Plans.md` を刈った結果、完了の記録（PR・archive の packet・closeout 時の申し送り）が repo のどこからも辿れなくなる。
- Workspace Access の文を変えた結果、`.env*`・鍵を読まない規則（`AGENTS.md` Safety）や Windows の Codex Desktop 向けの wrapper・`.codex/rules/default.rules`・test が壊れる、または CI の `Run Codex safe wrapper regression tests`（`.github/workflows/ci.yml:299-300`）が red になる。
- §5.4 を書き換えた結果、read-only の宣言・報告フォーマット・subagent 生成上限（現行の 5 点のうち残すべき 3 点）が消え、reviewer が tracked file を編集する、または結論を誘導される。
- `docs/agent-guidance/` の削除で `bash scripts/doc-consistency-check.sh` の link 検査が red になる。

### 非目的

- wrapper の script・test・`ci.yml`・classifier・helper・checker の変更（CI 制御の変更は R3 になり、owner 裁定で本 lane から外した）。
- `docs/AGENT_OPERATING_MANUAL.md` の全面的な縮約と節番号の振り直し（`docs/backlog.md:66` の別項目）。
- `docs/Plans.md` の `## 次の行動`・`## ブロッカー`・`## 製品の未決判断` の内容の見直し（本 lane は `## 直近の完了` と Wave Registry の完了済み wave だけを扱う）。
- Codex 側の発注の雛形（`.local/`、tracked 外）の改訂。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

workflow の変更なので、変更後の文書を使う通常の場面を操作列で書く。発注 → 停止と訂正 → review → owner 判断の通常列はそのまま使う。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| lane の PR が merge 済みで、closeout の R0 PR を書く Coordinator / subagent | `docs/DEV_WORKFLOW.md` Post-Merge Closeout の改訂後の行に従い、`## 直近の完了` の先頭に 1 行（日付・PR link・1 文・archive の packet への link、SHA なし）を足し、上限を超えた古い行を `docs/archive/harness-context/2026-09-30-Plans-completed.md` の `## 直近の完了` 節の先頭へ移す（長文が同 file に既にある短い行は移さずに消す） | 次の session が `docs/Plans.md` を読むとき、直近の完了が上限件数以内の短い行で見え、詳細は link 先（PR・archive の packet・移送先の archive）で読める | `bash scripts/doc-consistency-check.sh` が exit 0（link 実在と PK4 の pointer 行）、`## 直近の完了` の行数が上限以内 | 移送先の file は本 lane で作る。closeout の雛形（`.local/`）は merge 後に Coordinator が直す（Non-scope） |
| Coordinator が read-only の reviewer（Plan Review / Final Review）への発注書を書く | 改訂後の MANUAL §5.4 に従い、goal・scope 境界・read-only 宣言・報告フォーマット・subagent 生成上限に加えて、読むもの・読まないもの・判定の問い（Plan Review の発注では Ordinary Operation の 3 値を冒頭で答える、「目的を達成できるか」と「危険な結果を出さないか」を分けて問う）・確かめる command を具体に書く。packet・SHA の無い発注（Explorer 等）では goal に対する問いと対象範囲・読まないものを書き、packet・AC・SHA の項は当たるときだけ書く。期待する結論・finding・検証の順序と、同じ round の他の reviewer の結果は書かない。closure・是正の確認の発注では前回の findings と修正差分から始める | reviewer が対象と問いを 1 通りに受け取り、冒頭の 1 行で操作列の成立を答え、findings を claims として返す。Coordinator は `docs/DEV_WORKFLOW.md` Review Rules に従って裁定する | reviewer の報告が §5.4 の報告フォーマット（Verdict・件数・file:line）を満たす | §5.4 の改訂は今の発注の形（2026-09-27 以降の Codex review の発注）を規則にするもので、新しい発注の形を導入しない |
| 新しい session が `AGENTS.md` の Session Start から入り、Workspace Access を読む | WSL の作業 checkout で通常の読書・検索（Claude Code の Read / `rg`、Codex の `-s danger-full-access --ignore-rules` 起動）を行う。Windows の Codex Desktop から動かすときだけ `.codex/bin/*-safe-*.sh` と `.codex/README.md` の許可コマンドに従う | wrapper を使わない通常の session が、規則に反しているかの迷いなく読書できる。Windows の Codex Desktop を使う場面では従来どおり wrapper の経路が残る | `AGENTS.md` の Workspace Access が「Windows の Codex Desktop から動かすとき」の条件を先頭に持つ | Windows の Codex Desktop をこの repo で使わない（owner 2026-09-30「多分使わない」）は前提であり、使う場面が戻っても wrapper・test・rules は残っているので壊れない |

## Scope

各項目に出典と、main `58d9c051`（本 packet の起草時）での確認結果を付ける。

### S1: `docs/Plans.md` の `## 直近の完了` を刈り、closeout の書き方を変える（backlog (1)）

現状（確認済み）:

- `docs/Plans.md` は 57,760 bytes（`wc -c docs/Plans.md`）。`## 直近の完了` は `:28`〜`:101`（`## ブロッカー` が `:102`）で 52,056 bytes、entry は 34 件（AC-S1-1 の baseline）、最古は PR #52（`:85`。merge 日の記載が無く、archive の packet の path から 2026-09-13）、最新は 2026-09-30 の PR #130（entry ごとの最初の日付の測定: `sed -n '/^## 直近の完了/,/^### Wave Registry/p' docs/Plans.md | grep -E '^- \*\*' | awk 'match($0,/20[0-9][0-9]-[0-9][0-9]-[0-9][0-9]/){print substr($0,RSTART,RLENGTH)}' | sort | sed -n '1p;$p'` → `2026-09-13` / `2026-09-30`、main `58d9c051`。entry は全て `- **` で始まり、34 件全てに日付がある。`:73` の PR #71 の entry 内にある 8 月の日付は所感の起源で、entry の日付ではない）。entry は長文で main の SHA を含む（例 `:30` の「main `2ba91dea`」。8 桁の SHA を持つ行は `## 直近の完了`〜`## ブロッカー` で 32、AC-S1-2）。
- `### Wave Registry`（`:87`〜`:101`、3,216 bytes）は wave 10・11・12 の完了済みの記録で、`:89` の形式の行が「完了済み wave の記録だけを置く。進行中の lane は `docs/plans/` の packet が持つ（D-097）。これより前の完了済み wave の記録は archive に移送済み」と定める。lane 行は squash の SHA を含む。
- `docs/DEV_WORKFLOW.md:32`（Artifact Map）は Plans.md を「Current phase, active work, blockers, next actions only」、`:361`（Done Definition）は「reflects the live state, not the full PR history」、`AGENTS.md:33`（Working Rules）は「履歴の全文は dashboard に戻さない」と定め、`docs/DEV_WORKFLOW.md:112`（Evidence Ownership、D-038）は exact-HEAD の SHA を `Plans.md` に転記しないと定める（2026-07-12 以降の記述に適用。現在の entry は全て 2026-09-13 以降）。書き足す規則は `docs/DEV_WORKFLOW.md:329`「Update `Plans.md` so it reflects current live state, completed work, archived evidence, and next action」の 1 行だけで、entry の形と上限が無い。
- 完了履歴の既存の archive は `docs/archive/harness-context/2026-09-14-Plans.md`（126,060 bytes、`docs/Plans.md:3` が指す「移送前の記録」の snapshot）。`docs/Plans.md:114`・`:120`、`docs/backlog.md:5` からも参照される。
- PK4 は `## 次の行動` に `docs/plans/` を指す pointer の行があることだけを見る（`scripts/doc-consistency-check.sh:1305-1321`）。`docs/Plans.md:13` にその行がある（確認済み）。helper は `Plans.md` を読まない（D-097）。
- 既知の残り: 完了済み wave（10・11・12）を全て移すと `### Wave Registry` は見出しと `:89` の形式の行だけになる。消すと AC-S1-1 の範囲の終端（`/^### Wave Registry/`）が変わるので本 lane では残し、見出しと形式の行の扱いは closeout で backlog に積む（Plan Review round 3 の後の振り分け、Review Response 参照）。

設計判断 D1（刈った entry の行き先）: **非破壊に archive へ移す**（Coordinator の推奨を採る）。新しい file `docs/archive/harness-context/2026-09-30-Plans-completed.md` を作り、刈った entry と Wave Registry の完了済み wave（10・11・12）の行とその下の lane の行（字下げを保つ）を、本文を変えずに移す（相対 link だけ移送先に合わせて補正。`2026-09-14-Plans.md` と同じ方式。補正は link の path の接頭だけ: `archive/` 始まりは `archive/` を `../` に置き換え、それ以外の docs 起点の相対 path は `../../` を前置し、同じ dir の file も `../harness-context/` の形で書く。AC-S1-5 の正規化が `../` と `archive/` の差だけを吸収するため）。以後の closeout も同じ file の `## 直近の完了` 節の先頭へ移す（1 file を rolling で使う）。長文が同 file に既にある短い行（本 lane で書き換えた 10 件）は、上限を超えたとき移さずに消す（同じ entry の重複を作らない）。棄却案: (a) 消す（PR と archive の packet が持つ）— entry には closeout 時の申し送り（follow-up の置き場、closeout で決めた事項、Codex の Final Review の扱い）が含まれ、古い packet の Review Response に無いものがある。34 件を 1 件ずつ照合して消してよいか決める手間が、移す手間より大きく、消すと戻せない。(b) 既存の `2026-09-14-Plans.md` へ追記 — その file は「移送前の記録」の snapshot で構造が異なり（`## 残作業分類`・`## Backlog（未了）` 等）、追記すると snapshot の意味が崩れる。

設計判断 D2（残す件数の規則）: **直近 10 件**（固定の N）。理由: wave 13・14 以降は wave の境界を Wave Registry に置かず（D-097、`docs/Plans.md:9-16` の「次の行動」に置く）、「1 wave」は機械で数えられない。10 件は今の closeout の頻度（2026-09-25〜30 の 6 日で 17 件。`sed -n '/^## 直近の完了/,/^### Wave Registry/p' docs/Plans.md | grep '^- ' | grep -cE '2026-09-(25|26|27|28|29|30)'` → `17`、main `58d9c051`）で 3〜4 日分の視界に当たり、`grep -c` で検査できる。棄却案: 1 wave（境界が無い）、5 件（並走 6 本の wave 14 が同日に閉じると 1 wave 分を切る）。

設計判断 D3（entry の 1 行の形）: 先頭から順に、日付（YYYY-MM-DD）、PR 番号の link（GitHub の PR url）、題名（Risk、必要なら wave・lane）、結果の 1 文、archive の packet への相対 link（R3/R4 で Matrix があれば Matrix の link を続ける）を 1 行に置く。packet の無い R0/R1 の merge は PR link だけ（archive の packet の link は書かない）。main の SHA・squash の SHA・test 本数・介入回数は書かない（D-038）。残す 10 件も本 lane でこの形に書き換え、元の長文は D1 の archive に移す（つまり 34 件全ての長文が archive に入り、`Plans.md` には 10 件の短い行が残る）。

設計判断 D4（`docs/DEV_WORKFLOW.md:329` の書き換え）: 1 行を「`Plans.md` の `## 直近の完了` の先頭に 1 行（日付・PR link・1 文・archive の packet への link。packet の無い R0/R1 の merge は PR link だけ。SHA・test 本数を書かない）を足し、10 件を超えた古い行を `docs/archive/harness-context/2026-09-30-Plans-completed.md` の `## 直近の完了` 節の先頭へ本文を変えずに移す（相対 link の接頭だけ D1 の規則で補正する）。長文が同 file に既にある短い行は移さずに消す。`## 次の行動`・`## ブロッカー`・`## 製品の未決判断` を現在の状態に同期する」の趣旨に置き換える（英語の行なので英語で書く）。`docs/DEV_WORKFLOW.md:361-363`（Done Definition）と `:32` は既に整合し、変えない。`docs/AGENT_OPERATING_MANUAL.md` §5.3（Plans.md cleanup prompt、`:129-138`）の 1「完了項目を archive へ移す」は整合し、移送先の file 名を足すだけにする。

file と変更目的:

- `docs/Plans.md`: `## 直近の完了` を 10 件の短い行にし、`### Wave Registry` は見出し `### Wave Registry` と `:89` の形式の行を残して完了済み wave（10・11・12）の行とその下の lane の行を移す。`:3` の「完了履歴は archive」の link と `:89` の形式の行の archive の link（どちらも `2026-09-14-Plans.md` を指す）に新 file を足す。
- `docs/archive/harness-context/2026-09-30-Plans-completed.md`（新規）: `## 直近の完了` より前に HTML comment で「移送前の記録。現在の状態は docs/Plans.md」の注記と、以後の closeout が `## 直近の完了` 節の先頭へ足す（link の接頭は D1 の規則で補正する）規則の 1 行を置く。本文は `## 直近の完了`（移した entry）と `### Wave Registry`（完了済み wave 10〜12 の行とその下の lane の行。字下げを保つ。形式の行は置かない）の 2 節（AC-S1-5 の command がこの 2 節を切り出す）。移した entry の相対 link を D1 の接頭で補正する。
- `docs/DEV_WORKFLOW.md:329`: D4（packet の無い R0/R1 の merge は PR link だけ、の句を含む）。
- `docs/AGENT_OPERATING_MANUAL.md:129-138`（§5.3）: 移送先の file 名を 1 の行に足す。

### S2: `AGENTS.md` の Workspace Access を「Windows の Codex Desktop から動かすとき」に限る（backlog (2)）

現状（確認済み）:

- `AGENTS.md:48-55`（`## Workspace Access`）。`:50` が「WSL の作業 checkout を使用する。Windows からの実行、許可コマンド、起動設定は `.codex/README.md`」、`:52` が repo 内の読書・検索を `.codex/bin/read-safe-file.sh` / `search-safe-files.sh` / `list-safe-files.sh` に寄せる規則、`:53` が repo 外の Skill の読み方、`:54` が Windows から raw `wsl.exe ... bash -lc ...`・`cat` / `sed` / `rg` / `find` を広く allow しない規則。
- wrapper を使う規則の出典は D-049（`docs/decision-log.md:367-373`）: Windows の Codex Desktop の execpolicy が public clone の path の wrapper だけを auto-allow するため。`.codex/README.md:171-204` が Windows からの `wsl.exe ... --exec .codex/bin/*-safe-*.sh` の形と ask/deny の方針を持つ。
- 今の Codex は WSL から `-s danger-full-access --ignore-rules` で起動され rules を読まない（`docs/backlog.md:160` の起動行）。発注書 331 本（2026-09-30 時点の local の数、Coordinator 実測）で wrapper を使うものは 0。owner 2026-09-30: Windows の Codex Desktop はこの repo では多分使わない。
- tracked で wrapper を名指しする箇所（`rg -n 'read-safe-file|search-safe-files|list-safe-files|Workspace Access' --glob '!docs/archive/**'`）: `AGENTS.md:48`・`:52`、`scripts/tests/classify-changes.test.sh:101`（classifier の test の入力、変えない）、`docs/backlog.md:87`（本 lane の出典）、`docs/decision-log.md:370-378`（D-049 / D-050 の履歴、変えない）、`docs/research/audit-2026-07/adjudication.md:44`（履歴、変えない）。`.agents/**`・`.claude/**`・`CLAUDE.md`・`docs/TOOLING_SKILL_COMMANDS.md`・`docs/DEV_SETUP_CHECKLIST.md` に hit は無い。
- `.codex/bin/*-safe-*.sh` は `scripts/ci/classify-changes.sh:55` の実行制御（full）、`scripts/tests/codex-safe-wrappers.test.sh` は `.github/workflows/ci.yml:299-300` の step で走る（baseline: `bash scripts/tests/codex-safe-wrappers.test.sh` exit 0、`PASS: codex-safe-wrappers (T1-T15, HC-D8)`）。

裁定（Coordinator 2026-09-30、そのまま使う）: wrapper の script と test は残す。消すと `ci.yml` と classifier に触れ、CI 制御の変更で R3 になる。`AGENTS.md` の Workspace Access だけを「Windows の Codex Desktop から動かすとき」に限る。

変更の要点: `## Workspace Access` を、(1) WSL の作業 checkout を使い、別 worktree では cwd を確かめる（全 session 共通）、(2) Windows の Codex Desktop から動かすときだけ、`.codex/README.md` の許可コマンドに従い、repo 内の読書・検索を `.codex/bin/read-safe-file.sh` / `search-safe-files.sh` / `list-safe-files.sh` で行い、raw `wsl.exe ... bash -lc ...`・`cat` / `sed` / `rg` / `find` を広く allow せず、PowerShell の相対 path や UNC アクセスに依存しない、(3) repo 外の Skill は指定された `SKILL.md` と必要な参照だけを直接読む（全 session 共通、wrapper に外部 path を渡さない文は (2) へ）、の 3 点に組み替える。`.env*`・鍵・`auth.json` を読まない規則は `## Safety` にあり、変えない。D-049 のうち今も有効な wrapper の repo root 解決と safe read の境界は「Windows の Codex Desktop の経路」の決定として有効なまま、適用条件を AGENTS の文に書くだけで改訂しない（execpolicy の 2 mirror の維持は PR #97 で撤去済み〈D-092 の Compatibility、`docs/decision-log.md:764`〉で、policy は `.codex/rules/default.rules` だけ〈`scripts/tests/codex-safe-wrappers.test.sh:333` T11〉。D-101 の Compatibility に記録済み）。

file と変更目的:

- `AGENTS.md:48-55`: 上の組み替え。
- PR #81（Draft、`chore/repo-path-lowercase`）が触る `.codex/README.md`・`.codex/rules/default.rules`・`CLAUDE.md`・`docs/DEV_SETUP_CHECKLIST.md`・`docs/TOOLING_SKILL_COMMANDS.md`・`docs/project-memory.md`・`docs/plu-export-and-real-csv-verification.md`・`scripts/tests/codex-safe-wrappers.test.sh` は Scope に入れない（README の `/home/kosei/Projects/` の path は #81 が直す）。

### S3: `docs/AGENT_OPERATING_MANUAL.md` §5.4 を今の発注の形に合わせる（backlog (3)）

現状（確認済み）:

- `docs/AGENT_OPERATING_MANUAL.md:140-150`（§5.4「低制約発注書 profile（read-only の Reviewer / Explorer 向け）」）。5 点（goal・scope 境界・read-only 宣言・報告フォーマット・subagent 生成上限）のみで構成し、「過程指示・検証手順の指定を書かない（手順を縛るほど性能が落ちる世代特性への対応…）」「観点 list・必読順・検証 command の指定は書かない」と定める。Contract Audit / Final Review 役への発注では実施項目を scope 境界に列挙する例外がある。
- 今の reviewer 発注の実際の形（Coordinator の雛形、tracked 外、2026-09-27 以降の Plan Review / Final Review broad の発注）は次を具体に書く: 前提（隔離 worktree・SHA の一致で止まる条件・子 agent 禁止・tracked file の編集と git / PR 操作の禁止・実店舗の実値を書かない）、**読むもの**（AGENTS の節、DEV_WORKFLOW の要る節、対象 packet を `## Review Response` の前まで、設計正本の触る節と隣接節、Scope が触る現物）、**読まないもの**（他の reviewer の結果、PR body・comment）、注意（owner 決定の原文は claim として扱う）、**判定の問い**（冒頭 1 行で Ordinary Operation の操作列の成立を 3 値で答える。そのうえで A「正常な条件で目的を達成できるか」・B「危険な結果を出さないか」・C「現物と合っているか」を別々に問う。確かめる command〈AC の command を含む〉を具体に書き、doc check は `docs/quality/review-checklist.md:7` の段階別〈PR 前は手元の既定 mode、PR 後は hosted の docs job の結果を読み、手元は hosted の赤の再現と `--target plan` だけ〉に従う。旧前提の語を `rg` で全列挙する）、深度（effort と実読の範囲。「検証の手順・順序は任せる」）、severity の決め方、報告様式（冒頭 1 行・合否と件数・findings 表・AC 実測表・確認した対象と確認できなかった対象・監査した commit と `git status --porcelain`）。
- owner 2026-09-27「Sol は指示の質に結果が寄る」で具体化が方針。`docs/DEV_WORKFLOW.md:270-271`（Review Rules の Plan Review の発注の項、D-090）も「対象 packet の適用版・Goal・`Ordinary Operation`・Contract Probe・対象差分を参照させ」「操作列が目的を達成できるかを 3 値で答え」「『危険な結果を出さないか』と『正常な条件で目的を達成できるか』は別々に問う」と具体化の側にある。
- §5.4 を参照する他の箇所（`rg -n '5\.4|低制約' docs AGENTS.md CLAUDE.md .agents .claude --glob '!docs/archive/**'`。無関係な `§6.5.4`・`§5.4 フォーカス管理`・`65.4`・PDF の `5.4` を除く）: `docs/AGENT_OPERATING_MANUAL.md:154`（§5.6 の冒頭「§5.4 の read-only Reviewer / Explorer 専用の低制約 profile ではなく」）、`docs/backlog.md:87`（本 lane の出典）・`:160`（runbook 候補、「§5.4 / §5.6 は発注書の中身 profile のみ」）、`docs/decision-log.md:430-456`・`:521-522`・`:647`・`:767`（D-056 / D-058 / D-062 / D-081 / D-092 の履歴、変えない。D-056 は D-092 で全面 superseded 済み〈`docs/decision-log.md:764`〉で、今の §5.4 の内容は D-092 が「維持」した側にあるため、本 lane は D-056 に行を足さず D-101 の Replaces で D-092 の維持した内容を名指す）、`docs/research/2026-07-28-codex-sol-week-playbook.md:14`（履歴、変えない）。`docs/templates/subagent-review-packet.md` は §5.4 を参照せず、`## Role`・`## Target`（読ませないもの）・`## Contract Audit`・`## Output` の構成で既に「読むもの・読ませないもの・出力契約」を持ち、変えない。`.claude/agents/reviewer.md` は §5.4 を参照しない。

変更の要点（新しい §5.4）: 見出しは「5.4 read-only の Reviewer / Explorer への発注書」の趣旨に改める。見出しの語は変え、節番号は変えない（`docs/decision-log.md:767` の D-092 が「MANUAL の見出しと節番号は変えず」と定めるが、その文は PR2 の範囲〈D-092 の D3 / D4〉で、本 lane の見出しの語の変更は D-101 が記録する）。

- 必ず書くこと: (1) goal（何を判定・報告してほしいか）、(2) 対象と読む範囲（packet は `## Review Response` の前まで。設計正本の触る節と隣接節。Scope が触る現物）と**読まないもの**（**同じ round の**他の reviewer の結果の置き場、PR body・comment。範囲は `docs/AGENT_OPERATING_MANUAL.md:36` の独立性の項と同じ）、(3) read-only 宣言（tracked file の編集・git / PR 操作の禁止。Final Review の review comment 投稿は例外として明記）と subagent 生成上限（既定 0）、(4) 判定の問い（Plan Review の発注〈`docs/DEV_WORKFLOW.md:270-271` の対象〉では冒頭 1 行で Ordinary Operation の操作列の成立を `成立 / 具体的な反例あり / 外部前提が未確認` で答える。Final Review の発注で 3 値を必須にしない。「正常な条件で目的を達成できるか」と「危険な結果を出さないか」を別々に問う）と**確かめる command**（AC の command を含めて具体に書く。ただし doc check は `docs/quality/review-checklist.md:7` の段階別〈PR 前は手元の既定 mode、PR 後は hosted の docs job の結果を読み、手元は hosted の赤の再現と `--target plan` だけ〉に従う）、(5) 報告フォーマット（合否と P1/P2/P3 の件数・findings は file:line と具体的な反例と最小の修正案一案・AC 実測表・確認した対象と確認できなかった対象・全文 dump 禁止）、(6) 停止する条件（対象 SHA / PR head の不一致だけ。途中で許可を求めて止まらない）。packet・SHA の無い発注（例: Explorer の証跡収集、`docs/AGENT_OPERATING_MANUAL.md:21`）は、goal に対する問いと対象範囲・読まないものを書き、(2)(4)(6) の packet・AC・SHA の項は当たるときだけ書く。(7) closure・是正の確認の発注では前回の findings と修正差分から始める（`docs/templates/subagent-review-packet.md:19` の括弧「closure・是正の発注は前回 findings から始めてよい」と `:29`「closureは前回指摘と修正差分、影響する契約から確認する」）。
- 書かないこと: 期待する結論や finding（「○○が問題のはず」）、finding の件数の目標、検証の手順と順序（「まず X を確認してから Y」。対象と問いは書くが、順序は reviewer に任せる）、同じ round の他の reviewer の結果、Writer 向けの手順（§5.6）。
- 残すこと: findings は claims として Coordinator が裁定する（`docs/DEV_WORKFLOW.md` Review Rules）。Contract Audit / Final Review 役への発注で「Contract Audit」の実施項目を対象として列挙する文（`docs/AGENT_OPERATING_MANUAL.md:150` の「Contract Audit / Final Review 役への発注では…過程指示ではない」まで）。
- 消すこと: 「過程指示・検証手順の指定を書かない（手順を縛るほど性能が落ちる世代特性への対応…）」（`:142`）、「観点 list・必読順・検証 command の指定は書かない」（`:150`）、その根拠の括弧書き、および `:150` の末尾の「検証の手順・順序・command は引き続き指定しない。」の 1 文（残す Contract Audit の文からこの 1 文を切り離して消す）。

file と変更目的:

- `docs/AGENT_OPERATING_MANUAL.md:140-150`（§5.4）: 上の書き換え。`:154`（§5.6 冒頭）の「低制約 profile」の語を新しい見出しに合わせる。
- `docs/backlog.md:160` の「§5.4 / §5.6 は発注書の中身 profile のみで起動機構が未文書化」は、§5.4 の改訂後も真（起動機構は書かない）なので変えない。

### S4: `docs/agent-guidance/profiles/` と `evals/` を消す（backlog (4)）

現状（確認済み）:

- `docs/agent-guidance/profiles/{frontier,balanced,high-throughput}.md`（651 / 545 / 514 bytes）と `evals/{decision-gate-fixture,context-routing-fixture}.md`（3,791 / 3,040 bytes）。D-057（2026-07-28、GPT-5.6 family 向け）で置き、D-082（2026-09-05）が「profile 分離、slot-neutral 方針、人格を tracked に置かない方針は維持」とした。
- repo 内の参照（`rg -n 'profiles|evals' --glob '!docs/archive/**'`。`docs/project-memory.md:40` の「daily report profiles」は無関係）: `docs/agent-guidance/README.md:6`（profile の一覧、「未指定は frontier」）・`:16`（2 つの fixture で比較する）、`docs/backlog.md:66`（MANUAL 縮約の項の「profiles の README への統合」）・`:87`（本 lane の出典）。`docs/agent-guidance/shared.md` は「ここやprofileに別の契約を複製しない」「profileは作業の分解や応答密度の補助であり…」の 2 文、`docs/agent-guidance/model-notes.md` は「共通契約と用途別profileを使う」の 1 文で profile の語を使う（AC-S4-2 の baseline: README 2、shared 2、model-notes 1）。
- script・test・workflow は file を名指ししない（`rg -n 'agent-guidance/(profiles|evals)' scripts .github` は hit 0、exit 1）。`scripts/tests/classify-changes.test.sh:47` は `docs/agent-guidance/shared.md` を classifier の入力にするだけで、profiles・evals を使わない。`docs/agent-guidance/merge-evidence.md` に profiles・evals の語は無い。
- Coordinator の発注書・雛形は profile を指定しない（2026-09-30 時点、local の 331 本）。
- 既知の残り: `scripts/tests/reading-order-drift.test.sh:61-63` のコメント（`:63`「the routing fixture separately checks what models actually do with them」）が削除する `evals/context-routing-fixture.md` を指す。`scripts/tests` は Non-scope（classifier の workflow 分類）なので本 lane では直さず、closeout で backlog に積む。

設計判断 D5: **消す**（Coordinator の推奨を採る）。理由: 発注書が profile を選ばず、fixture は D-057 の起草時（2026-07-28）の 1 回の比較に使われ、以後の model 更改（D-082、HC-D1/D7）で再実行されていない。README の一覧と shared・model-notes の profile の文は、存在しない選択肢を毎回読ませる。D-057 の部分改訂として D-101 に記録する。棄却案: (a) 残して README の参照だけ消す（誰も参照しない file を tracked に残す理由が無く、classifier の policy docs に数えられ続ける）、(b) `docs/archive/` へ移す（履歴は git と D-057 / D-082 の本文が持つ。archive にも参照が無く、移す価値が無い）、(c) README へ統合する（`docs/backlog.md:66` の案。統合しても選ぶ場面が無い）。

file と変更目的:

- `docs/agent-guidance/profiles/*.md`・`docs/agent-guidance/evals/*.md`: `git rm`（5 file）。
- `docs/agent-guidance/README.md:6`・`:16`: profile の行と fixture の文を消す。「Model updates」の「モデル更新時は…観測された不足にだけ補助を追加する」は残す。
- `docs/agent-guidance/shared.md`: profile に触れる 2 文を、profile の語を使わない形に直す（「ここに別の契約を複製しない」「読書量を減らす場合も…」の趣旨は残す）。
- `docs/agent-guidance/model-notes.md`: 「共通契約と用途別profileを使う」を「共通契約を使う」にする。
- `docs/decision-log.md`: D-101 は本 packet の plan 側の commit で記録済み（下の「D-101」）。D-057 の `Superseded in part by: D-101` の 1 行も同じ commit（D-056 は D-092 で全面 superseded 済みなので足さない）。実装の PR で D-101 を変えるのは、実装の結果で文が変わるときだけ。

### D-101（本 commit で decision-log に記録済み）

D-101 の本文は `docs/decision-log.md` の末尾（D-100 の後。`rg -n '^## D-101' docs/decision-log.md`）。設計判断 D1〜D5 の昇格先で、Replaces (partial) は D-057（profile 分離と `frontier` 既定）・D-092 が維持した MANUAL §5.4 の内容（低制約 profile）・`docs/DEV_WORKFLOW.md:329` の 1 行、Compatibility は D-049 のうち有効な部分（wrapper の repo root 解決と safe read の境界。execpolicy の mirror は PR #97 で撤去済み）と、変えない test・CI・template。正本（`AGENTS.md`・`docs/DEV_WORKFLOW.md`・`docs/AGENT_OPERATING_MANUAL.md`・`docs/agent-guidance/`）への昇格は S1〜S4（実装の PR）。

## Non-scope

- wrapper の script（`.codex/bin/*-safe-*.sh`）・test（`scripts/tests/codex-safe-wrappers.test.sh`）・`.github/workflows/ci.yml`・`scripts/ci/classify-changes.sh`・helper・checker の変更（Coordinator の裁定 2026-09-30。CI 制御の変更は R3）。
- PR #81（Draft、`chore/repo-path-lowercase`）が触る file（`.codex/README.md`・`.codex/rules/default.rules`・`CLAUDE.md`・`docs/DEV_SETUP_CHECKLIST.md`・`docs/TOOLING_SKILL_COMMANDS.md`・`docs/project-memory.md`・`docs/plu-export-and-real-csv-verification.md`・`scripts/tests/codex-safe-wrappers.test.sh` ほか）。`.codex/README.md` の `/home/kosei/Projects/` の path は #81 が直す。
- Coordinator の closeout の雛形（`.local/`、tracked 外）の `Plans.md` の entry の書き方: merge 後に Coordinator が直す。
- `docs/backlog.md` の他の項目（`:66` の MANUAL 全面縮約と「profiles の README への統合」の語、`:160` の runbook）。`:87` の完了の反映と `:66` の「profiles の README への統合」の語の削除は本 lane の closeout で行う。
- `docs/Plans.md` の `## 次の行動`・`## ブロッカー`・`## 製品の未決判断`・`## 参照` の内容（`:3` の link の追加だけ S1 に含む）。
- `docs/templates/subagent-review-packet.md`・`.claude/agents/reviewer.md` の改訂（S3 の現状のとおり整合しており、変える理由が無い）。
- D-049 の改訂（有効な部分〈wrapper の repo root 解決と safe read の境界〉は適用条件を AGENTS の文に書くだけで有効のまま。execpolicy の mirror は PR #97 で撤去済みで、本 lane は触れない）。

## Acceptance Criteria

baseline は main `58d9c051` の worktree で AC の command を逐語で実行した出力（2026-09-30）。実装完了時の期待値と区別する。

- AC-S1-1（entry の上限）: `sed -n '/^## 直近の完了/,/^### Wave Registry/p' docs/Plans.md | grep -c '^- '` → baseline `34`、完了時 `10` 以下で `1` 以上。
- AC-S1-2（SHA の転記なし）: `sed -n '/^## 直近の完了/,/^## ブロッカー/p' docs/Plans.md | grep -cE '`[0-9a-f]{8}`'` → baseline `32`、完了時 `0`（Wave Registry の完了済み wave の行と squash の SHA を持つ lane の行も移すため）。
- AC-S1-3（file size、参考値）: `wc -c docs/Plans.md` → baseline `57760 docs/Plans.md`、完了時の値は `未実測`（実装の PR body に実測を書く。数の目標は AC-S1-1 が持つ）。
- AC-S1-4（closeout の規則）: `rg -n 'Update `Plans.md` so it reflects current live state' docs/DEV_WORKFLOW.md` → baseline `329:- Update ...`（1 行）、完了時 `0` 行。`rg -n '2026-09-30-Plans-completed.md' docs/DEV_WORKFLOW.md docs/AGENT_OPERATING_MANUAL.md docs/Plans.md` → baseline `0` 行、完了時 各 file に `1` 行以上。
- AC-S1-5（非破壊の移送）: `test -e docs/archive/harness-context/2026-09-30-Plans-completed.md; echo $?` → baseline `1`、完了時 `0`。本文の同一性は件数でなく diff で検査する: `diff <(git show origin/main:docs/Plans.md | sed -n '/^## 直近の完了/,/^## ブロッカー/p' | grep -E '^[[:space:]]*- ' | grep -vE '^- 形式:' | sed -E 's#\]\((\.\./)+#](#g; s#\]\(archive/#](#g') <(sed -n '/^## 直近の完了/,$p' docs/archive/harness-context/2026-09-30-Plans-completed.md | grep -E '^[[:space:]]*- ' | grep -vE '^- 形式:' | sed -E 's#\]\((\.\./)+#](#g; s#\]\(archive/#](#g'); echo $?` → 完了時 `0`（base の `## 直近の完了` の entry 34 行 + Wave Registry の完了済み wave の行 3 行 + その下の字下げの lane の行 8 行 = 45 行〈形式の行を除く。`git show origin/main:docs/Plans.md | sed -n '/^## 直近の完了/,/^## ブロッカー/p' | grep -E '^[[:space:]]*- ' | grep -vE '^- 形式:' | wc -l` → `45`、main `58d9c051`〉が、移送 file の同じ 2 節の行と、相対 link の `../`・`archive/` の接頭の差だけを正規化して一致する。字下げは比較に含む。移送 file が無い baseline では左右が一致せず非 0）。検出力の probe（2026-09-30、`$TMPDIR` の copy で実施、tracked に残さない。command は移送 file の path だけ copy に置き換えて逐語で実行）: base の 45 行を D1 の接頭で補正した正しい移送の copy → exit `0`（diff 出力なし）。同じ copy から字下げの lane の行 8 本を消した copy → exit `1`（`36,38d35`・`40,42d36`・`44,45d37` の 3 hunk）。同じ copy の entry 2 件目を 1 件目の複製に置き換えた copy → exit `1`（`2c2` の 1 hunk）。移した entry の PR link と archive の packet の link が `bash scripts/doc-consistency-check.sh` の link 実在検査を通る。
- AC-S2-1（適用条件の明示）: `sed -n '/^## Workspace Access/,/^## Memory Model/p' AGENTS.md | rg -c 'Codex Desktop'` → baseline `0`、完了時 `1` 以上。同じ範囲で `rg -c 'read-safe-file'` → baseline `1`、完了時 `1`（wrapper の経路は Windows の Codex Desktop の条件の下に残る）。
- AC-S2-2（wrapper・CI 制御に触らない）: `git diff --stat origin/main -- .codex scripts/tests/codex-safe-wrappers.test.sh .github/workflows/ci.yml scripts/ci/classify-changes.sh | wc -l` → baseline `0`、完了時 `0`。`bash scripts/tests/codex-safe-wrappers.test.sh` → baseline exit 0（`PASS: codex-safe-wrappers (T1-T15, HC-D8)`）、完了時 exit 0。
- AC-S2-3（Safety 以降は不変）: `diff <(git show origin/main:AGENTS.md | sed -n '/^## Safety/,$p') <(sed -n '/^## Safety/,$p' AGENTS.md); echo $?` → baseline `0`、完了時 `0`（`## Safety` と `## npm 供給網ガード` に変更行が無い）。
- AC-S3-1（逆向きの文の削除）: `rg -c '観点 list・必読順・検証 command の指定は書かない|手順を縛るほど性能が落ちる世代特性|command は引き続き指定しない' docs/AGENT_OPERATING_MANUAL.md` → baseline `2`（`:142` と `:150`。`:150` は「観点 list…」と「command は引き続き指定しない」の 2 句を同じ行に持つので行数は 2 のまま）、完了時 `0`（rg は hit 0 で exit 1 になるので `|| echo 0`）。
- AC-S3-2（具体に書く側の要点）: 4 語を別々に検査する。`sed -n '/^### 5.4 /,/^### 5.6 /p' docs/AGENT_OPERATING_MANUAL.md | rg -c '読むもの'` → baseline `0`（rg exit 1）、完了時 `1` 以上。同じ前処理で `rg -c '読まないもの'` → baseline `0`、完了時 `1` 以上。同じ前処理で `rg -c '判定の問い'` → baseline `0`、完了時 `1` 以上。同じ前処理で `rg -c '確かめる command'` → baseline `0`、完了時 `1` 以上。検出力の probe（2026-09-30、`$TMPDIR` の copy、tracked に残さない）: §5.4 の見出しの直後に 4 語を 1 行ずつ足した copy で 4 つとも `1`、そこから `確かめる command` の行を消した copy で `読むもの` `1`・`読まないもの` `1`・`判定の問い` `1`・`確かめる command` `0`（その検査だけ落ちる）。残すべき 3 点は見出しの行を除いた §5.4 の本文に対して別々に検査する: `sed -n '/^### 5.4 /,/^### 5.6 /p' docs/AGENT_OPERATING_MANUAL.md | grep -v '^### ' | rg -c 'read-only 宣言'` → baseline `1`、完了時 `1` 以上。同じ前処理で `rg -c '報告フォーマット'` → baseline `1`、完了時 `1` 以上。同じ前処理で `rg -c 'subagent 生成上限'` → baseline `1`、完了時 `1` 以上（3 つとも hit 0 なら rg exit 1）。検出力の probe（2026-09-30、`$TMPDIR` の copy、tracked に残さない）: `subagent 生成上限` を含む行を消した copy で `read-only 宣言` `1`・`報告フォーマット` `1`・`subagent 生成上限` `0`（3 つ目だけ落ちる）。
- AC-S3-3（参照の整合）: `rg -n '低制約' docs/AGENT_OPERATING_MANUAL.md` → baseline `2` 行（`:140`・`:154`）、完了時 `0` 行（見出しと §5.6 の参照を新しい語に合わせる）。`rg -n '5\.4' docs/AGENT_OPERATING_MANUAL.md` → 完了時 `:154` 相当の参照が新しい見出しへの参照として残る（切れた節番号を作らない）。
- AC-S4-1（削除）: `test ! -e docs/agent-guidance/profiles && test ! -e docs/agent-guidance/evals; echo $?` → baseline `1`、完了時 `0`。
- AC-S4-2（切れた参照なし）: `rg -c 'profile|evals/' docs/agent-guidance/README.md docs/agent-guidance/shared.md docs/agent-guidance/model-notes.md` → baseline `README.md:2` / `shared.md:2` / `model-notes.md:1`、完了時 hit 0（rg exit 1）。
- AC-S4-3（script・workflow の名指しなし）: `rg -n 'agent-guidance/(profiles|evals)' scripts .github docs --glob '!docs/archive/**' --glob '!docs/backlog.md' --glob '!docs/plans/**'` → baseline hit 0（exit 1）、完了時 hit 0（`docs/decision-log.md` の D-101 は path でなく語で書く）。
- AC-S4-4（決定の記録）: `rg -c '^## D-101' docs/decision-log.md` → baseline `1`（round 1 の後の design の段階で本 commit に記録済み。起票時は `0`）、完了時 `1`。
- AC-C-1（docs gate）: `bash scripts/doc-consistency-check.sh` → 完了時 exit 0。active packet がある間は `bash scripts/doc-consistency-check.sh --target plan` も exit 0。
- AC-C-2（PK5）: `bash scripts/check-workflow-git.sh` → 完了時 exit 0。
- AC-C-3（classifier の分類、参考）: 実装の PR の hosted CI で docs job が走り、`Merge gate` が成功する（Ready 後の run）。
- AC-C-4（reading-order drift）: `bash scripts/tests/reading-order-drift.test.sh` → baseline exit 0（`PASS: reading-order-drift`）、完了時 exit 0（新 §5.4 の「読むもの」に入口文書を列挙すると `DRIFT_PATTERN`〈`scripts/tests/reading-order-drift.test.sh:21`。`AGENTS` → `DEV_WORKFLOW` → `Plans` → `project-memory` の 4 語がこの順に、各 80 文字以内で近接して現れると一致。対象は tracked の `*.md` から `AGENTS.md`・`docs/archive/**`・`docs/decision-log.md`・`docs/plans/**` を除いたもので、`docs/AGENT_OPERATING_MANUAL.md` は対象〉に当たりうる）。

## Design Readiness

設計の完了条件は `docs/DEV_WORKFLOW.md` Design Phase Rules の Design completion criteria。

- 引用する設計正本（節まで）: `AGENTS.md` `## Workspace Access`・`## Working Rules`・`## Safety`、`docs/DEV_WORKFLOW.md` `## Artifact Map`・`## Workflow State`（Evidence Ownership）・`## Review Rules`・`## Post-Merge Closeout`・`## Done Definition`、`docs/AGENT_OPERATING_MANUAL.md` `## 座組`・§3 の独立性の項・§5.3・§5.4・§5.6、`docs/agent-guidance/README.md`・`shared.md`・`model-notes.md`、`docs/decision-log.md` D-038・D-049・D-057・D-082・D-090・D-092・D-097・D-098。これらは今の運用（長文の entry、wrapper を全 session の規則にする Workspace Access、「観点 list・必読順・検証 command の指定は書かない」§5.4、profile の一覧）を書いていて、本 lane が決める規則（`Plans.md` に残す件数・entry の形・移送先、Workspace Access の適用条件、新しい §5.4 の必ず書くこと・書かないこと、profiles・evals の撤去）は曖昧なく説明していない。
- 足りない（spec-check の結果）: 上の理由で `docs/DEV_WORKFLOW.md:96` の skip の条件〈Design Readiness cites existing design docs as sufficient〉は成り立たず、design を通す（round 1 の Codex #2、遷移記録）。
- 必要な設計成果物（Design artifact selection の当たる行だけ）: 「Durable cross-cutting choice, workflow gate change, architecture tradeoff, or decision likely to be revisited」（`docs/DEV_WORKFLOW.md:131`）→ `docs/decision-log.md` D-101（updated in this PR、design の段階の plan 側の commit。D-057 の `Superseded in part by: D-101` の行も同じ commit）。設計の出力 = D-101 と本 packet の設計判断 D1〜D5。設計正本（`AGENTS.md`・`docs/DEV_WORKFLOW.md`・`docs/AGENT_OPERATING_MANUAL.md`・`docs/agent-guidance/`）への昇格は S1〜S4（実装の PR）: これらは運用の振舞いを書く文書で、実装と同じ PR で変えないと文書と事実が食い違う（先例 `docs/archive/plans/2026-09-30-ci-dedup.md` の Design Readiness「design の出力は packet と decision-log、正本への昇格は実装」）。ADR = not needed（workflow 文書の改訂で、製品の設計判断でない）。BIZ / DTO / DB / 画面 / CSV の行は当たらない（製品コードを変えない）。
- plan にしかない durable な判断の昇格先: S1 の D1〜D4 → D-101（本 commit）と、実装の PR で `docs/DEV_WORKFLOW.md` Post-Merge Closeout・MANUAL §5.3。S2 の適用条件 → D-101 の Decision と Compatibility（本 commit）と、実装の PR で `AGENTS.md`。S3 の要点 → D-101（本 commit）と、実装の PR で MANUAL §5.4。S4 の D5 → D-101（本 commit、D-057 の部分改訂）。実装の結果で文が変わるときだけ D-101 を直す。
- 前提・制約と、延期した design gap の follow-up: Windows の Codex Desktop をこの repo で使わない（owner 2026-09-30「多分」）は前提で、戻す経路（wrapper・test・rules）は残す。closeout の雛形（`.local/`）の同期は merge 後の Coordinator。`docs/backlog.md:66` の MANUAL 縮約は別 lane。
- 絶対保証（cannot happen / always happens）の例外と escape hatch の自己点検: 「`Plans.md` に SHA を書かない」は D-038 の既存規則で、例外は無い。「10 件以下」は closeout の手順で守り、機械の検査は AC-S1-1 の command（checker には足さない。足すなら別 lane で PK を変える）。
- 判定（ready / not ready）と理由: ready。設計の出力（D-101 と D1〜D5）が本 commit にあり、未解決の設計の問いは無い。実装者は本 packet 無しでも、D-101 と（実装後は）`AGENTS.md`・`docs/DEV_WORKFLOW.md`・MANUAL §5.3 / §5.4・`docs/agent-guidance/README.md` から「何を変えるか・なぜ・何を退けたか」を読める。Windows native・実機の検証は不要。

## Registration / Generation Obligations

| 変更対象 | 登録・生成義務 |
|---|---|
| source / workflow doc 新設・改名・削除 | S1: `docs/archive/harness-context/2026-09-30-Plans-completed.md` の新設 → `docs/Plans.md:3` の「完了履歴は archive」の link に足す（`docs/PROJECT_HANDOFF.md` は `2026-09-14-Plans.md` を参照せず〈`rg -n 'harness-context' docs/PROJECT_HANDOFF.md` は `:22`・`:32` の 2 hit で、どちらも `2026-09-14-PROJECT_HANDOFF.md`〉、更新不要）。S4: `docs/agent-guidance/profiles/*`・`evals/*` の削除 → `docs/agent-guidance/README.md` の一覧から外す（AC-S4-2）。link 実在は `bash scripts/doc-consistency-check.sh` で確かめる |

Tauri command・function-design doc・REQ / coverage・route・operator 画面の行は該当なし（製品の code・設計書・生成物に触れない）。

## Impact Review Lenses

not applicable。現地調査・実機確認・外部 tool・POS 連携・CSV/TSV/report の形式・operator の操作の発見・設計前提の変更のいずれからも出発しない、workflow 文書の改訂と削除。各 lens に当たる finding は無い。

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | Which concepts belong to replaceable external adapters, and which concepts are stable app-core contracts? | Architecture, function design, decision-log, Plan Packet | 不該当（製品の境界に触れない） | なし |
| Fact check / design decision split | Which claims are observed facts from hardware/tool/files, and which are app decisions that need source-doc promotion? | Investigation doc, source design docs, decision-log | 不該当（事実は Scope の file:line と AC の baseline、判断は D-101 に置く） | なし |
| Lifecycle / retry | What happens before, during, after, and after failure for import/export, duplicate input, rollback, retry, cancellation, and re-run? | Function design, DB design, UI design, Test Matrix | 不該当（runtime に触れない） | なし |
| Operator workflow | What does the operator do in the real sequence across app, external tool, media, print/export, backup, and recovery? | Screen/UI design, function design, Plan Packet manual checks | 不該当（operator の操作に触れない） | なし |
| Replacement path | If the external system changes, which files/modules/docs are replaced and which app-core contracts remain stable? | Architecture, function design, decision-log | 不該当 | なし |
| Data safety / evidence | How can the claim be supported by anonymized shape/count/hash/procedure evidence without committing real store data? | Plan Packet Data Safety, investigation doc, review evidence | 不該当（店のデータを扱わない） | なし |
| Reporting / accounting semantics | Are totals, summaries, item records, returns, corrections, and inventory movements modeled separately enough to avoid false business meaning? | DB design, function design, report design, Test Matrix | 不該当 | なし |
| Manual verification | Which assertions cannot be proven by automated tests and require Windows native L3, external tool import, or real-device confirmation? | Plan Packet, Test Matrix, PR body | 不該当（L3 なし。AC は command で確かめる） | なし |
| 環境・再現性 | 新設の環境依存（toolchain / CI runner / OS 差異等）を repo-pinned config で強制するか、明示的に defer するか | repo-pinned config, Plan Packet | 不該当（新設の環境依存なし） | なし |

## Boundary / Wire Contract

not applicable。JSON API・browser state・CSV・config・manifest・cache schema・Tauri command DTO・生成 bindings・report 出力・DB 互換のいずれにも触れない。

- producer: なし
- consumer: なし
- wire type: なし
- internal type: なし
- precision/range: なし
- round-trip path: なし
- invalid input: なし
- compatibility: なし

## Test Plan

R2 のため Test Design Matrix は作らない: 変更は文書の書き換えと削除で、mutation を注入して red を確かめる対象の code・test が無く、AC の command（削除検査と存在検査）が検出力を持つ。Human Gate に manual を含まない。

- targeted tests: AC-S1-1〜S4-4 の command を実装後に逐語で実行し、baseline と完了時の値を PR body に表で書く。`bash scripts/doc-consistency-check.sh`（full。link 実在・PK1〜PK4・pointer 行）と `--target plan`。
- negative tests: AC-S3-1（旧い文の削除検査。現物の旧表現「観点 list・必読順・検証 command の指定は書かない」「command は引き続き指定しない」で一致し、完了時に不一致）、AC-S1-2（SHA の削除検査）、AC-S4-2（切れた参照の検査）。検出力は AC-S1-5（lane の行 8 本の欠落で非 0、複製 1 件で非 0）と AC-S3-2（4 語の 1 つ・3 点の 1 つを消すとその検査だけ 0）の probe で確かめた（各 AC の baseline）。
- compatibility checks: AC-S2-2（wrapper・test・`ci.yml`・classifier に差分が無い、wrapper test PASS）、AC-S2-3（`## Safety` に変更行が無い）、AC-S3-3（§5.4 への参照が切れない）、`bash scripts/check-workflow-git.sh`（PK5）。
- reading-order drift: 新 §5.4 の「読むもの」に入口文書（`AGENTS.md`・`docs/DEV_WORKFLOW.md`・`Plans.md`・`docs/project-memory.md`）を列挙すると `scripts/tests/reading-order-drift.test.sh` の `DRIFT_PATTERN` に当たりうるので、実装後にこの test を回す（AC-C-4）。
- data safety checks: 店のデータ・secret に触れない。移送する entry は既に公開 repository にある文で、新しい情報を足さない。
- main wiring/integration checks: 実装の PR の hosted CI（docs job、Ready 後の `Merge gate`）。

## Review Focus

- Ordinary Operation の 3 行が、変更後の文書だけを読んで 1 通りに実行できるか（特に closeout の行: 移送先・上限・行の形）。
- S1 の D1（非破壊の移送）と D2（10 件）が Goal Invariant の「短くなる」と「失敗定義」の「辿れなくなる」を同時に満たすか。棄却案の理由が現物と合うか。
- S2 の組み替えで、`## Safety` の規則と D-049 の Windows の経路が失われていないか。「Windows の Codex Desktop から動かすとき」の条件が、WSL からの Codex（`--ignore-rules`）と Claude Code の通常の session を縛らない書き方か。
- S3 の新しい §5.4 が、`docs/DEV_WORKFLOW.md:270-271`（Review Rules の Plan Review の発注の項）と `docs/templates/subagent-review-packet.md` と `docs/AGENT_OPERATING_MANUAL.md:36`（同じ round の他の reviewer の結果、closure は前回 findings から）と矛盾しないか。「書かないこと」に結論の誘導・期待する finding・検証の順序が含まれ、「必ず書くこと」に read-only 宣言・報告フォーマット・subagent 生成上限が残っているか。
- S4 の削除で切れる参照が AC-S4-2・S4-3 の検索式で全列挙されているか（`docs/decision-log.md` の D-057 / D-082 の本文は履歴として残す）。
- D-101（`docs/decision-log.md`、本 commit）が D-057・D-092 が維持した §5.4 の内容・D-049 の何を置き換え、何を残すかを正しく書いているか（D-056 は D-092 で全面 superseded 済み。D-049 の execpolicy の mirror は PR #97 で撤去済み）。
- AC の baseline が現物と合うか（確かめる command は AC の command。doc check は `docs/quality/review-checklist.md:7` の段階別で、PR 前の Plan Review は手元の既定 mode と `--target plan`）。R2・Final Review Minimum 2・Human Gate `ready,merge` の前提。

## Contract Ledger

R2 のため書かない（template の「ここから Data Safety までが R3/R4 の部分」）。

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| N/A | R2（Contract Ledger は R3/R4 の必須節） | — | — | 非対象 |

## Contract Probe

N/A。外部 library・OS・hardware の未確認の前提に依存しない（文書の改訂と削除）。

## Data Safety

R2 のため必須でない。店のデータ・secret・`.env*`・`auth.json` に触れず、移送する entry は既に tracked の文。

## Implementation Results

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

PR: [#132](https://github.com/higekiri-w/inventory-system-desktop/pull/132)（Writer = Opus 5.5 subagent。helper 経由の squash merge、2026-10-01）。AC の実測表と検証の結果は PR body。

- S1: `docs/Plans.md` の `## 直近の完了` の長文 entry 全件と Wave Registry の完了済み wave 10〜12 の行（lane の行を含む）を、`git show origin/main:docs/Plans.md` から機械的に切り出して `docs/archive/harness-context/2026-09-30-Plans-completed.md` へ移した（相対 link の接頭だけ D1 の規則で補正、AC-S1-5 の diff が一致）。`Plans.md` には直近 10 件を D3 の形の 1 行で置き、Wave Registry は見出しと形式の行だけにした。`docs/DEV_WORKFLOW.md` Post-Merge Closeout の行を D4 の英文に置き換え、MANUAL §5.3 の 1 に移送先を足した。
- S2: `AGENTS.md` の Workspace Access を、全 session 共通（WSL の checkout と cwd の確認、repo 外の Skill の読み方）と、Windows の Codex Desktop から動かすときだけの項（`.codex/README.md`・wrapper の読書・検索・allow しない command・PowerShell / UNC）に組み替えた。`## Safety` 以降は不変。
- S3: MANUAL §5.4 を「read-only の Reviewer / Explorer への発注書」にし、必ず書くこと 8 項目・packet の無い発注と closure の扱い・残す Contract Audit の文・書かないことを置いた。逆向きの 3 句を消し、§5.6 冒頭の参照を新しい見出しに合わせた。AC-S3-2 の検出力の確認で、2 語が同じ行にあると 1 語の欠落を見分けられなかったため、read-only 宣言と subagent 生成上限、判定の問いと確かめる command を別の項目に分けた。
- S4: `docs/agent-guidance/profiles/`・`evals/` の 5 file を `git rm` し、README の profile の行と fixture の文、shared の profile の 2 文、model-notes の「用途別profile」を外した。
- D-101 は実装の結果で文が変わらず、変更しなかった。
- 見送り（packet の既知の残り、closeout で backlog へ）: `scripts/tests/reading-order-drift.test.sh:61-63` のコメントが削除した routing fixture を指す点、Wave Registry の見出しと形式の行の扱い。
- 実装の要旨（closeout、2026-10-01）: `docs/Plans.md` の `## 直近の完了` は 1 行 + link の短い行を直近 10 件だけ置く形になり、長文と完了済み wave の記録は完了履歴の file へ移った。closeout は短い行を先頭に 1 行足し、10 を超えた古い行を完了履歴へ移す（長文が既にある行は消す）書き方に変わった（D-101）。`AGENTS.md` の Workspace Access は全 session 共通の項と Windows の Codex Desktop から動かすときだけの項に分かれ、MANUAL §5.4 は read-only の Reviewer / Explorer への発注書に必ず書くことの規則になった。どこからも参照されない GPT-5.6 向けの profiles・evals は削除した。
- review・CI・merge: Plan Review は 3 round の天井で、round 3 の後の一括是正を Coordinator が確かめ owner が plan-approved。Final Review broad は fresh Opus 5.5・Codex GPT-6.1 Sol とも approve で、P3 4 件のうち 3 件を是正し 1 件を見送り、closure は GPT-6.1 Sol が approve。Draft の run は各 head で success、Ready の後の run の `Merge gate` と `CLEAN` で helper 経由で merge した（owner の Ready・merge、2026-10-01）。manual は not-required。

## Review Response

Fill after review.
先行 round の結果・評価（判定・件数・採否・reviewer の意見）はこの節にだけ書く。前半の節と遷移記録には「round N の是正（Review Response 参照）」だけを書く（独立 review は `## Review Response` より前だけを読む）。

- Plan Review round 1（2026-09-30、対象 `a210940f`、互いに独立の 2 本）: fresh Opus 5.5 = reject（P2 2 / P3 5）、Codex GPT-6.1 Sol = reject（P2 5 / P3 1）。Coordinator が現物で裏取りして全件を採用し、起草役（Fable 5.1 subagent、発注 180）が是正した。Codex #2 で design を通し直し、Phase は plan-gate → design → plan-draft → plan-gate（遷移記録 `:30-31`）。Owner Effort Budget は変えない（本 round で owner の判断を消費していない）。finding ごとの採否と是正の所在（行番号は本 commit）:
  - Codex #2（P2、Design Phase の skip の条件を満たしていない。本 lane は既存の正本に無い規則を決めるので `docs/DEV_WORKFLOW.md:96` の skip に当たらず、`:131` の表は Plan より前に decision-log か ADR を求める）: 採用。遷移記録に `plan-gate → design` と `design → plan-draft → plan-gate` を append `:30-31`（既存の spec-check → plan-draft の行は履歴として残す）、`docs/decision-log.md` 末尾に D-101 を本 commit で記録（`:844`）、Design Readiness を「足りない（spec-check の結果）」「必要な設計成果物（`:131` の行）= D-101 と D1〜D5、正本への昇格は S1〜S4」「判定 ready」の形に書き直した `:219-225`（先例 `docs/archive/plans/2026-09-30-ci-dedup.md:30-31`）。
  - Opus F1・Codex #3（P2、AC-S1-5 の件数比較〈34〉は entry の入替えや複製を検出しない）: 採用。AC-S1-5 `:200` を base の `## 直近の完了` と Wave Registry の完了済み wave の行と、移送 file の同じ 2 節の本文の diff（`../`・`archive/` の接頭だけ `sed` で正規化）が 0 になる command にし、`$TMPDIR` の probe の結果（正しい移送 exit 0、複製 1 件で exit 1）を baseline に書いた。D1 `:104` に link 補正の接頭の規則、移送 file の 2 節の構造を `:115`。
  - Opus F2（P2、§5.4 の「検証の手順・順序・command は引き続き指定しない。」が「残すこと」の Contract Audit の文に抱き合わせで残る）: 採用。S3 の消すこと `:152` にこの 1 文（`docs/AGENT_OPERATING_MANUAL.md:150` 末尾）を足し、残すこと `:151` を Contract Audit の文の範囲に限った。AC-S3-1 `:204` の regex に `command は引き続き指定しない` を足して baseline を測り直した（`2`。同じ行 `:150` に 2 句があるので行数は変わらない）。
  - Codex #1（P2、「読まないもの」の他の reviewer の結果の禁止が closure・是正の確認まで縛り、`docs/AGENT_OPERATING_MANUAL.md:36` と template の「前回 findings から始めてよい」に合わない）: 採用。必ず書くこと (2) `:149` を「同じ round の」他の reviewer の結果に限り、(7) closure・是正の確認は前回の findings と修正差分から始める（`docs/templates/subagent-review-packet.md:19`・`:29`）を足した。書かないこと `:150`、Ordinary Operation の reviewer の行 `:87`、Review Focus `:279`、D-101 の Decision (3)。
  - Codex #5（P2、D-101 の Compatibility の D-049 に「execpolicy の mirror」を含めていたが、mirror の維持は PR #97 で撤去済み〈D-092 `docs/decision-log.md:765`〉で、`.codex/rules/default.rules` だけが policy〈wrapper test T11、`scripts/tests/codex-safe-wrappers.test.sh:333`〉）: 採用。D-101 の Compatibility を「有効なのは wrapper の repo root 解決と safe read の境界だけ」の形で decision-log に書き、packet の S2 の裁定 `:131`、失敗定義 `:66`、Non-scope `:190`、Review Focus `:281` を同じ形にした。
  - Codex #4（P2、AC-S3-2 の 3 点の検査が `read-only|subagent|報告フォーマット` の 1 本で、1 つ消えても通る。見出しの行も数える）: 採用。AC-S3-2 `:205` を見出しの行を除いた本文に対する `read-only 宣言`・`報告フォーマット`・`subagent 生成上限` の別々の `rg -c`（baseline 各 `1`）にし、`subagent 生成上限` の行を消した probe で 3 つ目だけ `0` になる結果を書いた。
  - Opus F3〜F7・Codex #6（P3、事実と指示の小さな直し）: 採用。(a) `:766` は D-092（D-093 ではない）`:145`、見出しの語は変え節番号は変えない `:147`、Design Readiness の引用 `:219`。(b) 行番号: Risk Tiers の R2 `:48` `:3`、Final Review Minimum `:84` `:23`、Plan Review の発注の項 `:270-271` `:144`・`:279`。(c) 数: 2026-09-25〜30 の entry は 17 件（12 件ではない）`:106`、`docs/PROJECT_HANDOFF.md` の `harness-context` は 2 hit `:231`。(d) D-056・D-057 に `Superseded in part by: D-101` の 1 行（`docs/decision-log.md:436`・`:447`）。(e) 長文が archive に既にある短い行は上限を超えたとき移さずに消す `:86`・`:104`・`:110`。(f) `### Wave Registry` の見出しと形式の行を残し、`docs/Plans.md:89` の形式の行の archive の link に新 file を足す `:114`。
  - packet の「D-101 の文案の要点」は decision-log の参照に縮め `:178`、AC-S4-4 `:210` の baseline を `1` に直した（完了時も `1`）。S4 の file の行 `:176`。
- Plan Review round 2（2026-09-30、対象 `06d1755e`〈round 1 の是正 commit〉、互いに独立の 2 本）: fresh Opus 5.5 = reject（P2 1 / P3 5）、Codex GPT-6.1 Sol = reject（P2 2 / P3 0）。Coordinator が現物で裏取りし、Opus #5 を除いて採用、起草役（Fable 5.1 subagent、発注 182）が是正した。Phase は plan-gate のまま（遷移記録に行を足さない）。Owner Effort Budget は変えない。round 3 が Plan Review の天井（`docs/DEV_WORKFLOW.md` Review Rules の round 数 3）なので、同じ型の残りも本 commit で一括して直した。finding ごとの採否と是正の所在（行番号は本 commit）:
  - Opus #1・Codex #1（P2、AC-S1-5 の両辺の `grep -E '^- '` が Wave Registry の字下げの lane の行 8 本を比べず、lane の行の欠落を検出しない）: 採用。AC-S1-5 `:201` の両辺を `grep -E '^[[:space:]]*- '` にし、比較の行数を 45（entry 34 + wave 3 + lane 8。測定 command と出力を併記）に測り直した。probe を回し直し、正しい移送 exit 0、lane の行 8 本を消した移送 exit 1（3 hunk）、entry 1 件の複製 exit 1（`2c2`）を baseline に書いた。移送の対象を「完了済み wave の行とその下の lane の行（字下げを保つ）」と D1 `:104`・S1 の file の行 `:114`・`:115`・AC-S1-2 `:198`・D-101 の Decision (1)（`docs/decision-log.md:846`）に書いた。
  - Codex #2（P2、§5.4 の「必ず書くこと」(4) の「AC の command は実際に回して baseline と比べる」が、D-100 の D4〈PR の後の reviewer は hosted の docs job の結果を読み、既定 mode の再実行を外す〉と `docs/quality/review-checklist.md:7` の段階別の doc check に食い違う）: 採用。(4) `:149` を「確かめる command を具体に書く（AC の command を含む）。ただし doc check は `docs/quality/review-checklist.md:7` の段階別に従う」の形にし、同じ趣旨の S3 の現状 `:143`・Review Focus `:283`・D-101 の Decision (3)（`docs/decision-log.md:846`）を同じ形に揃えた。
  - Opus #4（P3、Goal と Ordinary Operation が挙げる「確かめる command」が「必ず書くこと」に無く、AC-S3-2 の 4 語の OR 検査は 1 語が欠けても通る）: 採用。上の (4) に「確かめる command」を明記し、AC-S3-2 `:206` を語ごとの 4 つの `rg -c`（各 baseline `0`、完了時 `1` 以上）に分け、4 語を足した copy から `確かめる command` の行を消した probe でその検査だけ `0` になる結果を書いた。Test Plan `:270`。
  - Opus #2（P3、D-056 は D-092 で全面 superseded 済み〈`docs/decision-log.md:764`〉で、本 lane が D-056 に足した `Superseded in part by: D-101` の行は系譜を二重にする）: 採用。D-056 の当該行を消し（D-057 の行は残す）、D-101 の Replaces（`docs/decision-log.md:849`）の D-056 の項を「D-092 が維持した MANUAL §5.4 の内容（低制約 profile）」に置き換えた。packet の対応箇所 `:145`・`:177`・`:181`・`:220`・`:222`・`:282` を同じ形にした。行の削除で `docs/decision-log.md` の `:436` 以降の行番号が 1 減るので、前半の参照（`:52`・`:131`・`:145`・`:147`）を現物で測り直した。
  - Opus #3（P3、Risk の Reason に `docs/DEV_WORKFLOW.md:51` の tie-breaker への当てはめが無く、前例の R3〈PR #113、D-090〉との違いが書かれていない）: 採用。Reason `:52` に tie-breaker の引用と当てはめ（merge gate・helper・classifier・hosted CI・PK の合否を変えない、S3 は発注書の書き方の規則で Plan Review の判定の義務は `:270-271` の既存を写すだけ）、前例との違い（review の義務・判定そのものを変えた）を足した。owner の R2 の決定は claim のまま。
  - Opus #6（P3、`scripts/tests/reading-order-drift.test.sh:61-63` のコメントが削除する `evals/context-routing-fixture.md` を指す）: 採用。S4 の現状 `:167` に既知の残りとして 1 行（`scripts/tests` は Non-scope なので本 lane では直さず、closeout で backlog に積む）。
  - Codex の未確認の注記 + Coordinator の確認（最古の entry の日付「2026-08-31 の PR #52」は、`docs/Plans.md:85` の PR #52 の entry が 2026-09-13 と読める）: 採用。entry ごとの最初の日付を 34 件測り直し（command と出力を `:98` に併記）、最古は PR #52（archive の packet の path から 2026-09-13。`:73` の 2026-08-31 は PR #71 の entry 内の所感の起源）、現在の entry は全て 2026-09-13 以降 `:100` に直した。
  - Opus の備考（D4 の「本文を変えずに移す」に link の接頭の補正が伴うこと、「先頭」が `## 直近の完了` 節の先頭であることが読み取れない）: 採用。D4 `:110`・D1 `:104`・S1 の file の行 `:115`・Ordinary Operation の closeout の行 `:86`・D-101 の Decision (1)（`docs/decision-log.md:846`）に「`## 直近の完了` 節の先頭」と「相対 link の接頭は D1 の規則で補正」を明記した。
  - Opus #5（P3、§5.4 (2) の「packet は `## Review Response` の前まで」を同じ round の結果の段落だけに狭める）: 見送り。今の reviewer 発注の雛形は Plan Review・Final Review とも packet の `## Review Response` 以降を読ませない形で運用しており、S3 はその実務に §5.4 を合わせるのが目的で、closure・是正の確認は (7) の例外が持つため、狭めるのは本 lane の目的の外。
- Plan Review round 3（2026-09-30、対象 `a8e3bb17`〈round 2 の是正 commit〉、互いに独立の 2 本。Plan Review の round 天井）: fresh Opus 5.5 = reject（P2 1 / P3 6）、Codex GPT-6.1 Sol = approve（P 0）。Coordinator が現物で裏取りし、`docs/DEV_WORKFLOW.md` Review Rules の round 天井の項に従って「同型指摘の一括是正 / backlog 化」に振り分け、起草役（Fable 5.1 subagent、発注 184）が是正した。round 天井に届いたので次の round は開かず、独立の再確認は Final Review が兼ねる。Phase は plan-gate のまま（遷移記録に行を足さない）。Owner Effort Budget は変えない。finding ごとの採否と是正の所在（行番号は本 commit）:
  - Opus #1（P2、新 §5.4 は read-only の Reviewer と Explorer の両方に当たるが、「必ず書くこと」の (2)(4)(6) が packet と SHA を前提にし、packet の無い Explorer の証跡収集の発注〈`docs/AGENT_OPERATING_MANUAL.md:21`〉では 1 通りに書けない）: 一括是正。(4) の 3 値の問いを Plan Review の発注（`docs/DEV_WORKFLOW.md:270-271` の対象）に限り、Final Review の発注で 3 値を必須にしない文と、packet・SHA の無い発注は goal に対する問いと対象範囲・読まないものを書き (2)(4)(6) の packet・AC・SHA の項は当たるときだけ書く文を `:150` に足した。Ordinary Operation の reviewer の行 `:87`、Risk の「写すだけ」の括弧 `:52`、D-101 の Decision (3)（`docs/decision-log.md:847`）を同じ形にした。
  - Opus #2（P3、Risk の tie-breaker の「workflow gate」と Final Review Minimum の根拠 `docs/DEV_WORKFLOW.md:84` の「workflow gate changes」の読み方が書かれていない）: 一括是正。Risk `:52` に、`:84` の括弧（classifier `workflow=true`）は Final Review Minimum 2 の trigger、tie-breaker（`:52`）の workflow gate は merge gate・helper・classifier・hosted CI・PK の合否の意味で、本 lane は前者に当たり後者に当たらない、の 1 文を足した。
  - Opus #3（P3、D-092 に D-101 への back-reference が無い）: 一括是正。D-092 の末尾に `Superseded in part by: D-101`（§5.4 の低制約 profile の維持の部分）の 1 行（`docs/decision-log.md:769`。D-057 `:446` と同じ形）。
  - Opus #4（P3、S2 の要点 (2) の wrapper が glob で、AC-S2-1 の `read-safe-file` と一致しない）: 一括是正。`:132` を 3 つの script 名で書いた。
  - Opus #6（P3、packet の無い R0/R1 の merge の entry に archive の packet の link が無い）: 一括是正。D3 `:109`・D4 `:111`・S1 の file の行 `:117` に「PR link だけ（archive の packet の link は書かない）」。
  - Opus #7（P3、引用の行番号）: 一括是正。tie-breaker を `docs/DEV_WORKFLOW.md:52`（`:52`）、D-049 の範囲を `docs/decision-log.md:367-373`（`:125`）に実読で直した。
  - Opus の残存リスク（新 §5.4 の「読むもの」に入口文書を列挙すると `scripts/tests/reading-order-drift.test.sh` の `DRIFT_PATTERN` に当たりうる）: 一括是正。Test Plan `:274` に 1 行、AC-C-4 `:216` にこの test の exit 0（DRIFT_PATTERN の条件を括弧で）。
  - Opus #5（P3、完了済み wave を全て移すと `### Wave Registry` は見出しと形式の行だけになる）: backlog。消すと AC-S1-1 の範囲の終端が変わるので本 commit では入れず、S1 の既知の残り `:103` に 1 行、closeout で backlog に積む。

- Final Review broad（2026-10-01、対象 `89c58c2e`、互いに独立の 2 本、Final Review Minimum 2）: fresh Opus 5.5 = approve（P1 / P2 0、P3 2）、Codex GPT-6.1 Sol = approve（P1 / P2 0、P3 2。PR #132 の review）。両本とも round 3 の後の一括是正（`a8e3bb17..842f7ab8`）を独立に確かめ、正本との矛盾は無かった。2 本を pending で record してから是正した（Coordinator、finding は現物で確認）。
  - Opus #1（P3、`docs/DEV_WORKFLOW.md:329` の「exceeds 10 lines」が Wave Registry の行を数えるかで 1 通りに読めない）: 是正。「`### Wave Registry` より上の entry の行が 10 を超えたら」に直した（AC-S1-1 の数え方と一致）。
  - Opus #2（P3、`.codex/README.md` への起動設定の案内が Desktop の条件の下だけに移り、WSL の Codex CLI の起動設定〈`.codex/README.md:35`〉を AGENTS から辿れない）: 是正。`AGENTS.md` の Workspace Access の冒頭に「Codex の起動設定は `.codex/README.md`」を戻し、Desktop の項は「Windows 側の実行と許可コマンド」に絞った。Safety 以降は不変（AC-S2-3）。
  - Codex #1（P3、`docs/Plans.md` の短い行に D3 の求める R3/R4 の Matrix の link が無い）: 是正。archive に Matrix がある R3/R4 の entry に Matrix の link を足した（R2 の小口のまとめは D3 の対象外）。`docs/DEV_WORKFLOW.md:329` にも Matrix の link を書き足した。
  - Codex #2（P3、AC-S3-2 の「読まないもの」の単独検査は、語が §5.4 の別の行にもあるので必須の行を消しても落ちない）: 見送り。直すには packet の AC を変える Gated Amendment が要る。実装の §5.4 は必須項目がそろうことを両 reviewer が確かめており、AC は merge 後に archive される計画の検査で以後の運用に残らないため、費用に見合わない。

- Closeout（2026-10-01）: 是正（上限の数え方、AGENTS の Codex の起動設定の案内、R3/R4 の短い行の Matrix の link）の後の closure は Codex GPT-6.1 Sol = approve（新しい指摘 0）。record は broad の 2 本を pending、closure を pass で記録した。Ready の後の run `36758395876` は `Merge gate` success、helper の blockers は空、`mergeStateStatus` は `CLEAN`。Draft の run は各 head で success。見送った Codex #2（AC-S3-2 の単独検査の検出力）は archive される計画の検査で運用に残らないため積まない。既知の残り 2 点（Plan Review round 3 の Opus #5 の `### Wave Registry` の見出しと形式の行、S4 の `scripts/tests/reading-order-drift.test.sh:61-63` のコメント）は `docs/backlog.md` の保留へ置いた。本 closeout は D-101 の書き方で `docs/Plans.md` に短い行を足した最初の closeout。
