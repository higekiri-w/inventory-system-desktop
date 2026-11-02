# Plan Packet: 同期の後の L3 と closure の使い回し（merge-evidence MG-D13・D-144）

wave に属さない単独の lane。helper（`scripts/pr-gate.py`）の manual の使い回しと base 同期の closure の受理条件、その test、正本の文（merge-evidence・DEV_WORKFLOW・ci.md・decision-log）を変える workflow の lane。製品の runtime・CI の job・classifier は変えない。

出所: owner の決定（2026-10-27、repo 外の回答台帳 TD-474〈A と B の両方〉、TD-476〈docs だけの同期の closure を同じ条件で helper の記録に代える〉）。所感 TD-475（小さな見直しで重い手続きに縛られる今の体制はおかしい）を受け、範囲を MG-D13 とその helper・test・正本の文に絞る。手続きそのものの軽量化は本 lane の外。

規模と手法: 起票（本 commit）は packet・Matrix と正本の文（plan-first。下の「本 commit で変えた正本」）。helper と test の実装は plan-approved の後の別の run。並走の workflow の lane #204（`agent/docs-file-line-check`）と直列。順は Coordinator が決める。

## Workflow State

- Phase: implementing
- Risk: R3
- Plan Commit: fabc9d67e355b63664f997269a4bad5e234732fe
- Amendments: none
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（起票は発注 638。実装は plan-approved の後の別の run）
- Plan Reviewer: fresh Opus 5.5（fork でない）+ Codex。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Reviewer: Claude 側 1 本 + Codex。座組表どおり（R3 なので Claude 側は Fable 5〈当面、TD-410〉）
- Final Review Minimum: 2
- Human Gate: ready,merge
- Branch: agent/l3-reuse-after-sync

manual なし: helper と test と docs の変更で、Windows native で確かめる物が無い。Final Review Minimum 2: `scripts/pr-gate.py` と `docs/agent-guidance/**` を変え classifier が `workflow=true` を出す workflow gate の変更（`docs/DEV_WORKFLOW.md` Workflow State の `Final Review Minimum`）。Double Audit（Contract Audit を 2 回）。

遷移の記録:

1. kickoff → spec-check（2026-10-27、起草役）: 範囲を TD-474・TD-476 の A・B・同期の closure と、それに合わせる正本の文にし、Risk を R3 と記録した（下の Risk）。
2. spec-check → design（2026-10-27、起草役、同じ commit）: merge-evidence の「base同期だけでheadが変わる場合」の再利用の条件を置き換える設計が要る（下の Design Readiness）。
3. design → plan-draft（2026-10-27、起草役、同じ commit）: 設計の成果を正本へ置いた（merge-evidence MG-D13 と同節、D-144）。未解決の設計の問いは無い。
4. plan-draft → plan-gate（2026-10-28、Coordinator）: packet と Matrix は起票の commit `4d057c65` で commit 済み。`doc-consistency-check.sh --target plan` は全チェック通過。
5. plan-gate のまま是正（2026-10-28、round 1 の reject を受けて）: round 1 の是正（Review Response 参照）。
6. plan-gate のまま是正（2026-10-28、round 2 の reject を受けて）: round 2 の是正（Review Response 参照）。
7. plan-gate のまま是正（2026-10-28、round 3〈上限〉の reject を受けて）: round 3 の是正（Review Response 参照）。
8. plan-gate のまま是正（2026-10-28、closure の P2 を受けて）: 是正の確かめ（closure）の是正（Review Response 参照）。
9. plan-gate → plan-approved → implementing（2026-10-28、Coordinator）: Plan Review は round 1〜3（上限）と、上限の後の是正の確かめ（owner の決定 TD-503・TD-504）で、独立の reviewer の P1/P2 = 0（Claude 側 fresh Opus 5.5 の closure が `ee0386b0` で approve、Codex 側 Sol の closure が `fabc9d67` で approve）。Plan Commit = `fabc9d67`（計画の最後の commit で、すべての実装の commit より前）。

## Owner Effort Budget

- 介入回数上限: 6（既定）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 6 | 2（TD-474・TD-476） | 2（Ready 1・merge 1） | 2（Ready の後の base 同期の再 Ready 1・Plan Review の裁定が owner に上がる 1） | 6 = 2 + 2 + 2 |

承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
merge の関門（helper の manual と review の closure の受理条件）を変える。`docs/DEV_WORKFLOW.md` Risk Tiers の R3 の「merge gate changes」。D-098 が安全の境界に挙げる「古い証拠の拒否」を、PR 差分の byte 同一から同期だけの条件の機械の確かめへ置き換える改訂で、AGENTS.md Working Rules の「安全の境界の改訂は通常の packet・PR の経路で行う」に従い、owner の決定（2026-10-27、TD-474・TD-476）で起票する。R4 にしない理由: 破壊的な操作・実データ・secret・不可逆な git 操作を伴わない。helper は read-only の git（`rev-parse`・`merge-base`・`merge-tree --write-tree`・`diff`）だけを足し、`merge-tree --write-tree` は object を書くが ref と作業 tree を変えない。

required gate の green / red: この変更で `Merge gate`（hosted の required check）の green / red は変わらない。変わるのは CI の外の helper の manual と review の closure の受理条件だけ（classifier・`.github/workflows/**`・`scripts/ci/**` を変えない）。

## Goal

Goal Invariant:

### 最小完了条件

- L3 の PASS の後に base 同期だけで head が変わった PR で、lane の差分が同じ（同期だけの条件）なら、共有の file の index の行や周りの行がずれても、同期が何回でも、owner が L3 をやり直さずに merge まで進める。取り込んだ main 側が code に触れなければ helper が自動で（A）、触れれば同期の closure 2 本と owner の 1 行の承認で（B）。
- 同期だけで code に触れない版では、独立 review の closure を回さずに helper の 1 回の record（`--review-stage sync`）で review の要件が満ち、同じ record で manual も引き継がれる。
- 使い回せないとき、helper が何も書かずに止まり、外れた理由（lane の commit・自動の merge と違う path・衝突の path・code に触れる main 側の path）と次の一手を名指す。

### 失敗定義

- lane の手（同期の merge の中の編集・衝突の手での解消・同期の後の lane の commit・main の外の branch の取込み）が入った版で、manual か closure を使い回す。
- code に触れる同期で、owner の承認なしに manual を、独立の closure なしに review を通す。
- R4 の承認を使い回す。
- 使い回せないときに manual の pass を黙って pending へ戻し、B の道を失わせる（今の closure を先に記録したときの挙動）。
- 今の正当な record（`source_head` と `reuse_approval` の組、監査の closure）を新しい helper が拒む。

### 非目的

- 手続きそのものの軽量化（closure の本数・Double Audit・Plan Review の形。Coordinator が別に案を作る）。
- classifier・CI の job・`Merge gate`・GitHub の ruleset の変更。
- 同期の自動化（helper が merge する等）、GitHub の Update branch を使える形にすること。
- stacked train の base の付け替えの後の使い回し（`docs/DEV_WORKFLOW.md` Stacked train のまま）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

workflow の変更なので、L3 の後の base 同期の通常列を置く。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| Draft、broad と manual（L3）が pass の record。先行 PR が docs だけを変えて merge | Coordinator が `git -c merge.directoryRenames=false merge origin/main` で同期 → push → `capture` → `record --kind review --review-stage sync --outcome pass` | 1 回の record で closure（`sync_from`）と manual（`source_head`）が新しい head に付く。owner の操作は無い | status の blocker が CI だけ | P1・P4（docs だけの同期は A） |
| 同上。先行 PR が code を変えて merge | 同期 → push → closure 2 本（座組表の Claude 側と Codex）に L3 の項目への影響も問う → owner に 1 行の承認を諮る → `capture` → `record --kind manual --outcome pass --evidence <旧 manual の evidence> --reuse-from SOURCE --reuse-approval POINTER` → `capture` → `record --kind review --review-stage closure`（2 本の証跡を evidence に並べる） | owner は L3 をやり直さず、承認の 1 行だけ | closure が P1/P2 0 で、L3 の項目に影響なし | P2・P3（#197・#209 の型は B。ただし #197・#209 の今の record は旧 helper で closure を先に記録したため manual が pending に戻っていて、A も B も「旧 record の manual が pass」を要るので当たらない。本 lane の merge の後に同期し、その head で L3 をやり直す〈owner の決定 TD-504〉。下の `## 後続` の移行の扱い） |
| 同上。B の後にさらに docs だけの同期 | 同期 → push → `capture` → `record --kind review --review-stage sync --outcome pass` | 起点は旧 record の head（B で承認した版）なので A で引き継ぐ | 同上 | — |
| 同期の後に lane の commit がある、または衝突を手で解いた（生成物と `docs/decision-log.md` の union の結果を除く） | `capture` → manual 以外の record | helper が何も書かずに止まり、外れた commit・path と次の一手を出す。同期だけの条件（H2）で外れたので B は案内せず、`record --kind manual --outcome pending --evidence POINTER` だけを出す | L3 をやり直す | — |

## Scope

新しく書く引用は D-125・`docs/DOC_STYLE_GUIDE.md` §1 の形（path と symbol・節名・決定 ID。行番号で引かない）。owner の決定は「repo 外の回答台帳 TD-nnn」で引く。

### 予定の file

| file | 変更 | 段 |
|---|---|---|
| `docs/agent-guidance/merge-evidence.md` | 設計判断の表に MG-D13。RecordV1 の Closure に `{head, base, sync_from}` の形と説明の 1 文。「状態と非CI記録」の closure 数の段落・manual の段落・capture の段落の該当の文。「base同期だけでheadが変わる場合」の再利用の 3 段落を、同期だけの条件・code に触れない条件・manual の使い回し・review の closure・確かめ直しの 5 段落に置き換え。「Helperの境界」の CLI の文と pending へ戻す文。「実行手順」の順序の文 | 本 commit |
| `docs/DEV_WORKFLOW.md` | Workflow State の箇条の `MG-D6〜D9` の引用、Wave Operation の前文の `MG-D6` の引用を MG-D13 に。Ready の箇条の「L3 を済ませた PR を後ろへ回す順にしない…」の括弧を削除。L3（manual）の箇条（先頭で頼む既定・束ねる条件・`comm -12` の手順・防ぐ失敗の文）を 1 文の新しい規則に置き換え | 本 commit |
| `docs/ci.md` | `## Stale Green Prevention` の `MG-D6/D7` の引用を MG-D13 に | 本 commit |
| `docs/decision-log.md` | D-144 を末尾に足す。D-124 の末尾に `Superseded in part by: D-144` の行 | 本 commit |
| `docs/backlog.md` | workflow の「衝突を解いた版の manual の再利用（観測）」の項目を D-144 の後の残り（生成物と decision-log の union の外の衝突）に合わせる | 本 lane の round 2 の是正 |
| `scripts/pr-gate.py` | 下の H1〜H7 | 実装 |
| `scripts/tests/pr-gate.test.py` | 下の Matrix の行 | 実装 |
| `docs/plans/2026-10-27-l3-reuse-after-sync.md`・`docs/plans/test-matrices/2026-10-27-l3-reuse-after-sync.md` | 本 packet と Matrix | 本 commit |

`docs/AGENT_OPERATING_MANUAL.md` の座組表は変えない。Final Reviewer の行の「closure（base 同期を含む）の Claude 側の既定は Fable 5」は closure を誰が回すかの値で、いつ要るかは merge-evidence が持つ（MG-D13 の文が要らない場合を持ち、座組表に条件を継ぎ足さない）。`docs/Plans.md` に lane の行を足さない（D-097）。

### helper の変更（`scripts/pr-gate.py`）

- H1 `validate_record`: closure は `{head, base, audit}` か `{head, base, sync_from}` のどちらか 1 つの形（両方・どちらも無しは `invalid record fields`）。`sync_from` の形には「独立の run」の検査を当てない。`sync_from` の形でも broad が null なら拒む（今の監査の closure と同じく broad を要る）。manual は `source_head` だけ（A）か `source_head` と `reuse_approval` の組（B）を受け、`reuse_approval` だけは拒む。`source_head` があれば outcome は pass（今のまま）。
- H2 同期だけの条件の関数（`reuse_geometry` を置き換える。名は実装で決める）: 引数は起点の SHA・head・base。head から first parent を辿って起点に着くまでの各 commit が (a) 2 親の merge で、second parent が base の祖先（`git merge-base --is-ancestor`）、2 親の `merge-base --all` が 1 行、tree が `git -c merge.directoryRenames=false merge-tree --write-tree FIRST SECOND` の tree と同じか、違う path（`git diff --name-only --no-renames`）と衝突の path が全部生成物、または (b) 1 親で、親との差分の path（`git diff --name-only --no-renames`）が全部生成物。列の中で最後の merge の second parent が base で、その後ろ（head まで）は (b) だけ（head 自身が 1 親の再生成の commit でもよい。列に merge が無い、つまり head が起点と同じか (b) だけなら満たさない）。path の列挙は全部 `--no-renames` に固定し、通常の path から生成物への rename は通常の path の削除として拒む。`merge-tree --write-tree` と `merge-base --is-ancestor` の終了値 0/1 は判定の結果（衝突の有無・祖先か否か）として扱い、それ以外の終了値は判定不能（`command failed`）とする。今の `git()` は 0 以外を全部 `command failed` にするので、この 2 つは終了値を受け取る呼び方にする（G4 の衝突と G9 の理由に SHA が出るのはこれに依る）。終了値を受け取る呼び方も `git()` と同じ 1 つの入口（`command`）を通し、test はその入口を patch する（合成の repo へ向ける patch が新しい呼び方にも効く）。`docs/decision-log.md` の衝突と違いは、merge の結果のその blob が `git merge-file -p --union --object-id FIRST:docs/decision-log.md MERGE_BASE:docs/decision-log.md SECOND:docs/decision-log.md` の出力と byte で一致するときだけ、生成物と同じく受ける（並走する lane の末尾への D の追記どうしの衝突。手で解いた結果・片側を落とした結果は一致しないので外れる。`.gitattributes` の `merge=union` は同期の `git merge` が lane の作業 tree の属性を読むので、属性を足した最初の同期で効かない。下の Contract Probe）。生成物の集合は helper の定数で、hosted の drift 検査が当たる 2 path（`src/lib/bindings.ts`・`docs/function-design/90-traceability.md`）に固定する（comment で `scripts/ci/classify-changes.sh` の `generated` の case と `.github/workflows/ci.yml` の `rust_drift` の job を引く）。classifier の case を解析して得ない。`src/routeTree.gen.ts` は tracked でなく drift 検査の外なので入れない。満たさないときは理由（commit の SHA と path）を持つ `GateError`。
- H3 code に触れない条件の関数: H2 が辿った同期の列の各 merge（(a) の 2 親の merge。merge-base は H2 で一意を確かめ済み）について `git diff --name-only --no-renames <merge-base(FIRST, SECOND)> SECOND` の path を集め、その和集合を、`requirements` が使うのと同じ current main 側の classifier の呼び方（`--files-from-stdin`）に渡し、`rust`・`frontend`・`env`・`unknown` が全部 false。満たさないときは、classifier が code に分ける path を名指す（path ごとに classifier に渡して決める）。`requirements` の classifier の呼出しは 1 つの関数にまとめて両方から使う。起点と最後の base の差分だけを分類しない（列の途中で main が code を変えて後で戻すと、最後の差分に code が残らず、L3 の版と違う code を A で通す。round 3）。H4・H5・H6・H7 は H2 の列を受け取るこの 1 つの関数を使う。
- H4 `record` の版が変わった分岐（`current` が偽）: 旧 record の manual が pass で manual が必須なら、起点を旧 record の head として H2 と H3 を当てる。満たせば manual を `{outcome: pass, evidence: 旧 evidence, source_head: 旧 manual の source_head か旧 head, reuse_approval: 旧にあればそのまま}` で引き継ぐ。H3 は H2 を満たしたときだけ評価する。`--kind manual` で `--reuse-from` が無いとき（L3 をやり直す `--outcome pending` や新しい L3 の記録）は H4 の判定を省く（旧 head の object が無い checkout でも L3 をやり直す出口を塞がない）。満たさず、今回の `--kind` が manual でなければ、何も書かずに exit 1 で止まり、理由と次の一手を出す。判定不能（git の失敗の exit 2。local に base や旧 head の object が無いとき等）では次の一手に `git fetch origin` を添える。次の一手は理由で分ける: H2 で外れたら `record --kind manual --outcome pending --evidence POINTER` だけ（B も H2 を要るので案内しない）。H2 を満たし H3 だけで外れたら、B の完全な command `record --kind manual --outcome pass --evidence <旧 manual の evidence> --reuse-from <旧 manual の source_head か旧 head> --reuse-approval POINTER` と要るもの（同期の closure 2 本と owner の 1 行の承認）、または `--outcome pending` の command の両方。R4 は今のまま pending。head が旧 record の head と同じで base だけが変わったときも同じに止める（H2 は満たさない）。理由は「base が進んだ。先に同期する」で、次の一手は同期（`git -c merge.directoryRenames=false merge origin/main`）の後の record。
- H5 `--review-stage sync`（argparse の choices に足す）: `--kind review --outcome pass` だけを受け、`--pass-model`・`--run-ref`・`--evidence`・`--reviewed-head`・`--pr-reviews` を受けたら入力の誤り（exit 2）。版が変わったときだけ（`current` が真なら止める）。旧 record の review が pass で、broad の plan_commit と amendments が今の packet と同じ。起点は旧 record の closure が監査の形ならその head、`sync_from` の形ならその `sync_from`、closure が無ければ broad の head。H2 と H3 を満たせば `review = {outcome: pass, broad: 旧 broad, closure: {head, base, sync_from: 起点}}`。満たさなければ何も書かずに止まり、理由と「監査の closure を要る」を出す。今の head に本文のある PR review が投稿済みなら（GitHub の PR review を全 page 数える。`--pr-reviews` の検査と同じ数え方）、各 review の id と投稿時刻と通常の closure の記録手順（`--review-stage closure` と `--pr-reviews N`）を示して書込みの前に止まる（D-107 の取りこぼしの防止を sync の経路でも保つ）。`--review-stage sync` は新しい head での最初の record でだけ使える（先に他の kind を記録すると `current` が真になる）。`current` が真で止めるときの理由の文にこの順序を書く（自動化はしない）。
- H6 manual の明示の使い回し（`--reuse-from`。`--reuse-approval` があれば B、無ければ A を明示で求める形で、review の要らない R0/R1 の manual の lane の同期で使う）: `--reuse-from` は旧 record の manual の `source_head`（あれば）か旧 head と一致、evidence は旧 manual と一致、旧 manual は pass。起点を旧 record の head として H2 を当て、H3 は `--reuse-approval` が無いときだけ当てる。書く manual は `source_head` と `reuse_approval`（B なら `--reuse-approval` の値、`--reuse-approval` が無ければ旧 manual の `reuse_approval` を H4 と同じく引き継ぐ。旧に無ければ置かない）。引き継いだ `reuse_approval` があれば H7 は H3 を当て直さないので、B → docs だけの同期 → 明示の A の版も status を通る。R0/R1 の lane は review の closure を持たないので、B の closure 2 本の証跡は `reuse_approval` の pointer の先（owner の承認と同じ場所）に置く。今の「`--reuse-from` と `--reuse-approval` は両方か両方無し」の入力の検査は「`--reuse-approval` は `--reuse-from` と組でだけ」に変える。旧 manual が使い回しでも受ける（今の「使い回しの使い回しは拒む」を外す）。
- H7 `nonci`（status・ready・merge）: manual に `source_head` があれば H2 を `source_head` から現在の head へ当て直し、`reuse_approval` が無ければ H3 も当てる。closure が `sync_from` の形なら H2 と H3 を `sync_from` から当て直す。`validate_review` は `sync_from` の形の closure を現在の head/base の closure として受ける（broad の必要数と Plan 契約の検査は今のまま）。

呼び出し側・隣接の確認（`rg -n "closure|source_head|reuse_|review_stage|nonci\(|validate_review\(|reuse_geometry" scripts/pr-gate.py` と test 側の同様の `rg` で確かめた）:

- 作る所: `Gate.record`（closure の dict を作る review の分岐、manual の分岐、版が変わった分岐の record の初期値）。
- 読む所: `validate_record`、`Gate.validate_review`、`Gate.nonci`、`Gate.record` の closure の分岐（旧 broad の検査）。
- test の fixture と呼び方: `scripts/tests/pr-gate.test.py` の `good_record`・`args`（`reuse_from`・`reuse_approval` を持つ）、`Records` の mutant（`manual.source_head` だけの record を拒む mutant は H1 で A の形として受けるので、`reuse_approval` だけの mutant に置き換える）、`RecordLifecycle.exercise`（`reuse_geometry` を patch する。H2 の関数名に合わせる）と `test_reuse_then_closure_order`・`test_source_record_missing_changed_or_not_pass`・`test_closure_first_loses_old_manual_and_cannot_restore_cache`（H4 で「closure を先に記録すると止まる」に変わる）、`GitReuse.test_exact_merge_and_changed_patch`（H2 の合成の repo の test に置き換える）、fake gh の process の class の closure の記録（`review('closure','closure',…)` の呼び方。closure の監査の形は変えないのでそのまま通る）。
- `scripts/tests/merge-gate.test.sh` は helper を呼ばない（`rg -n 'pr-gate|manual' scripts/tests/merge-gate.test.sh` が 0 件）。`scripts/doc-consistency-check.sh` の PK4 と `scripts/tests/doc-consistency-plan-packet.test.sh` は helper の `parse_packet`・`workflow_fields` だけを使い、本 lane はそれらを変えない。

### 他の lane との所有

起票の時点（起点 `bbea3314`）の open の PR 12 本の変更 file（`gh pr diff <N> --name-only`）と本 lane の予定 path を突き合わせた。重なるのは次の 3 つ。

| file | 相手 | 相手の所有 | 本 lane の所有 |
|---|---|---|---|
| `docs/agent-guidance/merge-evidence.md` | #204 | `## 実行手順` の R2+ の例の段落の review packet の引用 | 設計判断の表の MG-D13 の行、`## 状態と非CI記録` の該当の文、`## base同期だけでheadが変わる場合`、`## Helperの境界` の該当の文、`## 実行手順` の「Double Auditは」の段落の順序の文 |
| `docs/ci.md` | #204 | `### Final Trigger Selection` と `## Local Commands` | `## Stale Green Prevention` の 1 文 |
| `docs/backlog.md` | #172・#190・#201・#211・#212・#215 | それぞれの項目（どれも「衝突を解いた版の manual の再利用（観測）」の行を変えない。2026-10-28 に `gh pr diff <N>` で確かめた） | workflow の「衝突を解いた版の manual の再利用（観測）」の 1 項目 |
| `docs/decision-log.md` | #194・#201・#209・#211・#212・#216 | 末尾への D の追記（D-139〜D-143 ほか） | 末尾の D-144 と D-124 の末尾の 1 行。末尾の追記は merge 順で両方を残す |

`scripts/pr-gate.py`・`scripts/tests/pr-gate.test.py`・`docs/DEV_WORKFLOW.md` はどの open の PR も触らない。helper・検査・gate を変える #204 とは直列（順は Coordinator が決める）。

### この変更で不要になる記述・条件（D-127）

- merge-evidence の PR 差分の byte 比較の条件（`git diff --binary --full-index …` の比較）、単一の merge の条件、「再利用した manual の再利用は拒む」、manual → fresh capture → closure の記録の順序（A では順序が要らず、B は「新しい head での最初の record」の 1 文に縮む）。
- DEV_WORKFLOW Wave Operation の L3 を train の先頭で頼む既定、束ねる条件と `comm -12` の手順、L3 済みの PR を後ろへ回さない句、共有の path を closeout へ寄せる句（D-124 の (1)〜(4)。L3 の後の同期で L3 が無効にならないので、既定としては要らない）。ただし先に merge する PR と生成物・`docs/decision-log.md` の union の外で衝突する見込みがある PR は同期で L3 が外れるので、DEV_WORKFLOW の L3 の箇条に「その merge と同期の後に L3 を頼む」1 文を範囲を衝突へ狭めて残す（round 2 の是正）。
- DEV_WORKFLOW・ci.md の `MG-D6〜D9`・`MG-D6`・`MG-D6/D7` の引用（再利用の規則はどの MG-D の行にも無く、引用がずれていた。MG-D13 に集めた）。
- helper の `reuse_geometry` と、`nonci` の「`head^1` が source_head・2 親・`head^2` が base」の 3 つの検査（H2・H7 に置き換わる）。版が変わった record で head が同じで base だけが変わったときに manual を黙って pending に戻す挙動（H4 の止める形に通す）。
- 残すもの: R4 の承認を使い回さない文（今も正しい）。同期の後の CI。B と code に触れる同期の監査の closure。

## Non-scope

- 座組表（`docs/AGENT_OPERATING_MANUAL.md` `## 座組`）の値と closure の本数。
- classifier（`scripts/ci/classify-changes.sh`）の分類の変更。tests だけの main 側の変更（`src/**/*.test.ts*`）も classifier が `frontend=true` に分けるので code として扱う（下の Contract Audit の既知の限界）。
- 同期だけの条件を CI の job にすること。
- Coordinator の運用の checklist（repo の外）の書換え（Coordinator が本 lane の merge の後に合わせる）。
- 他の active な packet の D-124 の引用（`docs/plans/2026-10-23-stocktake-count-runtime-b2.md` の 1 文。その lane の closeout で archive へ移り、遡って直さない）。

## Acceptance Criteria

- AC1（同期だけの条件）: `PYTHONDONTWRITEBYTECODE=1 python3 scripts/tests/pr-gate.test.py` が exit 0 で、Matrix の G1〜G13 の test が合成の git repo で、(a) 2 回の同期の自動の merge・生成物だけが衝突し再生成した merge・生成物だけの再生成の commit・`docs/decision-log.md` の末尾の追記の衝突を union の結果で解いた merge を受け、(b) 同期の後の lane の commit・生成物の外の衝突を手で解いた merge・自動の merge と違う tree（merge の中の編集）・main の外の branch を second parent に持つ merge・列の最後の merge の second parent が base でない版・merge-base が一意でない merge・通常の path から生成物への rename・`docs/decision-log.md` を union と違う形で解いた merge を拒み、拒む理由に commit の SHA と path を含む。baseline（起点 `bbea3314`）: 同じ command が exit 0、末尾 `OK`（今の test は byte 同一の 1 回の merge だけを受ける）。
- AC2（code に触れない条件）: 同じ command で Matrix の C1〜C6 が、main 側が docs・packet・`docs/function-design/90-traceability.md` だけなら満たし、`src-tauri/**`・`src/lib/bindings.ts`・`package.json`・`scripts/pr-gate.py`・未知の path のどれかがあれば満たさず、その path を名指す。
- AC3（A と B と止める形）: 同じ command で Matrix の R1〜R13 が、docs だけの同期で manual を引き継ぎ（どの kind の record でも）、code に触れる同期と head が同じで base だけが変わった版で manual 以外の record を何も書かずに止めて理由と次の一手（code に触れるだけなら `--outcome pass --evidence` を含む B の完全な command と `--outcome pending` の 2 つの command、同期だけの条件で外れたら `--outcome pending` だけ）を出し、`--reuse-from` の無い manual の record では判定を省き、B を受け、B の後の docs だけの同期で自動と明示の A を受け（明示の A も `reuse_approval` を引き継ぎ status が通る）、R4 を引き継がないことを確かめる。
- AC4（同期の closure）: 同じ command で Matrix の S1〜S8 が、`--review-stage sync` で `sync_from` の closure を書き、code に触れる同期・旧 review が pass でない・版が変わらない・余計な引数・Plan 契約の変更・今の head に投稿済みの本文のある review を拒むことを確かめる。
- AC5（記録の形と確かめ直し）: 同じ command で Matrix の V1〜V6 が、今の形の record（`source_head` と `reuse_approval` の組・監査の closure）を新しい `validate_record` が受け、`reuse_approval` だけ・closure の 2 つの形の混在を拒み、`nonci` が `source_head`・`sync_from` から確かめ直すことを確かめる。
- AC6（文書）: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-10-27-l3-reuse-after-sync.md` が ERROR 0。`rg -n 'byte比較で同一|MG-D6〜D9|MG-D6/D7|MG-D6の単一merge|先頭で頼むのが既定' docs --glob '!docs/archive/**' --glob '!docs/decision-log.md' --glob '!docs/plans/**'` が 0 件（本 commit で満たす）。
- AC7（workflow の回帰）: `bash scripts/check-workflow-git.sh` が exit 0、Draft の hosted の `Draft (no merge evidence)` が緑（classifier は `workflow=true` と `pr-gate.py` で full を出す）。

## Design Readiness

- 引用する設計正本（節まで）: `docs/agent-guidance/merge-evidence.md` の `## 設計判断`（MG-D13）・`## 状態と非CI記録`（RecordV1）・`## base同期だけでheadが変わる場合`・`## Helperの境界`・`## 実行手順`。`docs/DEV_WORKFLOW.md` の `## Workflow State`・`## Wave Operation`。`docs/decision-log.md` の D-098・D-124・D-144。
- 必要な設計成果物: workflow gate change → `docs/decision-log.md`（D-144）と merge-evidence の MG-D13 の節 = updated in this PR（本 commit）。
- plan にしかない durable な判断の昇格先: 同期だけの条件・code に触れない条件・A・B・同期の closure・止める形は全部 merge-evidence と D-144 に置いた。packet にだけある判断は helper の関数の分け方（H1〜H7）で、実装の段取り。
- 前提・制約と延期した design gap: classifier の分類に頼る（tests だけの main 側の変更も code として B に回る）。GitHub の Update branch の merge が `merge-tree` と違えば外れる（安全側）。どちらも D-144 の Guarantee range に書いた。
- 絶対保証の自己点検: 「lane の手が入った版で使い回さない」の例外と escape hatch は、(1) 生成物の違い（hosted の drift 検査が source との一致を強制。`Merge gate` が要る）、(2) `merge-tree` の rename 検出の差（`merge.directoryRenames=false` を同期と同じにする。違えば外れる側）、(3) `docs/decision-log.md` の union の結果（docs の 1 file に限る。union は両側の行を残すだけで、lane の行を足し引きできない。外れた結果は手の解消として拒む）、(4) record を helper の外で手で書く経路（今と同じ信頼の形。owner の account だけが書け、`nonci` が確かめ直す）。
- 判定: ready。

## Registration / Generation Obligations

該当なし（Tauri command・function-design・REQ・route・画面を足さない。`docs/agent-guidance/merge-evidence.md` は既存の文書で、目次・索引の登録は変わらない）。

## Impact Review Lenses

not applicable: 実機・外部の tool・POS・CSV・operator の発見から始まる作業でない。owner の決定（TD-474・TD-476）と PR #197・#209 の git の履歴から始まり、git の挙動は下の Contract Probe で確かめた。

## Boundary / Wire Contract

- producer: `scripts/pr-gate.py` の `record`（専用 PR comment の RecordV1 の JSON）。
- consumer: 同じ helper の `server_record`・`validate_record`・`nonci`・`record`（PR の base の版の helper だけが判定する。D-099）。
- wire type: `review.closure` は `{head, base, audit}` か `{head, base, sync_from}`。`manual` は `source_head?` と `reuse_approval?`（`reuse_approval` は `source_head` があるときだけ）。`version` は 1 のまま。
- internal type: 同じ dict。
- precision/range: SHA は 40 桁の小文字 hex（`sha`）。pointer は空でない 1 行（`pointer`）。
- round-trip path: `record` が書いた JSON を `server_record` が `decode_json`（重複 key を拒む）→ `validate_record` で読み戻し、`record read-back mismatch` で照合する（今のまま）。
- invalid input: closure の 2 つの形の混在・どちらでもない形、`reuse_approval` だけの manual は `invalid record fields` / `incomplete manual reuse`（exit 2 / 1 は実装で今の分け方に合わせる）。
- compatibility: 今の record（監査の closure、`source_head` と `reuse_approval` の組）はそのまま受ける。新しい形の record を古い helper（本 lane の merge の前の main の版）は `invalid record fields` で拒むが、helper は PR の base の版だけが動き（D-099）、本 lane の merge の後の base でしか新しい形を書かないので混ざらない。

## Test Plan

Test Design Matrix: [test-matrices/2026-10-27-l3-reuse-after-sync.md](test-matrices/2026-10-27-l3-reuse-after-sync.md)

- targeted tests: `PYTHONDONTWRITEBYTECODE=1 python3 scripts/tests/pr-gate.test.py`（Matrix の全行）。
- negative tests: Matrix の G6〜G13・C2〜C6・R3〜R5・R12・S2〜S8・V3〜V4。
- compatibility checks: V1・V2（今の形の record）。
- data safety checks: 合成の git repo を tmpdir に作り、実の PR・store のデータを使わない。fake gh は今の process の fake を使う。
- main wiring/integration checks: fake gh の process の class で `record --review-stage sync` が argv から実装に届き、comment の PATCH の body に `sync_from` が載ること（S1 の integration の行）。
- workflow: `bash scripts/check-workflow-git.sh`、`bash scripts/doc-consistency-check.sh`、`--target plan`。Draft の hosted（full）。

## Review Focus

- 同期だけの条件が lane の手を全部拒むか（Matrix の G の反例。特に main の外の branch を second parent に持つ merge と、生成物の例外の範囲）。
- code に触れない条件の path の集合が classifier と食い違わないか（`unknown` の扱い、rename の両側）。
- H4 の止める形が今の記録の順序（B・closure）と矛盾せず、B の道を失わせないか。
- `sync_from` の closure が独立 review の要件（broad の必要数・Plan 契約の変更で fresh broad）を緩めていないか。
- この変更で不要になる記述・条件は他に無いか（D-127）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| MG-D13 同期だけの条件 | merge-evidence `## base同期だけでheadが変わる場合` | H2 | G1〜G13 | 非対象 |
| MG-D13 code に触れない条件 | 同上 | H3 | C1〜C6 | 非対象 |
| MG-D13 manual の A | 同上、`## Helperの境界` | H4 | R1・R2・R6・R7 | 非対象 |
| MG-D13 manual の B | 同上 | H6 | R5・R8・R9・R11 | 非対象 |
| MG-D13 止める形（D-127） | 同上 | H4・H5 | R3・R4・R12・R13・S2 | 非対象 |
| MG-D13 同期の closure | 同上、`## 状態と非CI記録` | H5 | S1〜S8 | 非対象 |
| RecordV1 の Closure と manual の形 | `## 状態と非CI記録` | H1 | V1〜V4 | 非対象 |
| 確かめ直し | `## base同期だけでheadが変わる場合` の最後の段落 | H7 | V5・V6 | 非対象 |
| R4 を使い回さない | 同上 | H4（R4 は pending のまま） | R10 | 非対象 |
| closure の数と broad の保持（隣接） | `## 状態と非CI記録` | 変えない（H7 の `validate_review` は broad の検査を保つ） | `scripts/tests/pr-gate.test.py` の `RecordLifecycle` の Plan 契約の変更と broad の欠けの既存の case と S6 | 非対象 |
| helper は base の版だけが判定する（隣接、D-099） | `## CIとmain保護` | 変えない | 既存 | 非対象 |
| capture の一致と記録の前後の照合（隣接） | `## 状態と非CI記録` | 変えない | `RecordLifecycle` の capture の変更の既存の case | 非対象 |
| L3 を頼む時点（D-124 の置換え） | DEV_WORKFLOW `## Wave Operation` | 文だけ（本 commit） | AC6 | 非対象 |

## Contract Audit（起票の時点の一巡）

新しく足す契約の文（MG-D13 の 5 段落、RecordV1 の Closure の形、Helper の境界の CLI の文、DEV_WORKFLOW の L3 の箇条、D-144）を境界の 4 観点で一巡した。

- 安全な整数の範囲: 新しい数は無い（同期の回数に上限を置かず、first parent を起点まで辿る。起点に着かずに root か 1 親の lane の commit に当たれば止まるので、辿る長さは履歴の長さで抑えられる）。→ 不足なし。
- 0 の表示: main 側の変更の path が 0 件（起点の後に main が進まない同期は無い。列の最後の merge の second parent が base で、merge の無い列〈起点と同じ head を含む〉は満たさない）。code の path が 0 件なら満たす。生成物の違いが 0 件の merge は tree の一致と同じ扱い。→ 正本の不足は「head が起点と同じ」の扱いで、H2 に「満たさない」と書いた（版が変わらない record は今の `current` の分岐で、使い回しを当てない）。
- 失敗の時点と順序: H4 の止めは「何も書かずに」で、comment の PATCH の前（今の `validate_record` と `same_pr` の照合の前）に置く。B は新しい head での最初の record、A は順序が要らない。H5 は版が変わった最初の record でだけ（正本と止める理由の文に書く）。head が同じで base だけが変わった版も H4 の止める形に通す（失敗定義 4 と合わせる）。`merge-tree --write-tree` と `merge-base --is-ancestor` の終了値 0/1 は判定の結果で、それ以外の終了値の git の失敗（`command failed`）は exit 2 で止まり、満たした扱いにしない（判定不能は満たさない、MG-D13）。→ 正本の不足は「判定不能」で、merge-evidence の同期だけの条件の段落に書いた。
- 単位や種類の混在: 起点が 3 種（manual は旧 record の head・確かめ直しは `source_head`、closure は最後の監査の版）。B の承認の後の A は、記録の時点では旧 record の head から当て、確かめ直しでは `reuse_approval` があれば code に触れない条件を当てない（承認が code の取込みを覆っている）。closure の `sync_from` は最後の監査の版で、`sync_from` の連鎖でも同じ値を引き継ぐ。→ 正本の不足は確かめ直しの起点で、merge-evidence の最後の段落に書いた。

既知の限界（受け入れる）: classifier の `frontend=true` に tests だけの main 側の変更も入るので、それは B に回る。`.github/workflows/**` など CI の実行 code も code として B に回る。

## Contract Probe

- `git merge-tree --write-tree` が使えること: `git --version` → `git version 2.56.0`（helper は owner の WSL の git で動く）。
- PR #209 の 2 回の同期の merge は自動の merge を再現する: `git -c merge.directoryRenames=false merge-tree --write-tree a4a110e1^1 a4a110e1^2` → `ea11e797…`、`git rev-parse a4a110e1^{tree}` → `ea11e797…`（一致）。`c4737fa2` も `041e4d48…` で一致（full SHA は `git rev-parse` で引ける）。
- PR #197 の同期の merge は生成物だけが違う: `git -c merge.directoryRenames=false merge-tree --write-tree 3c7e1413^1 3c7e1413^2` は exit 1、衝突は `docs/function-design/90-traceability.md` だけ。`git diff --name-only <merge-tree の tree> 3c7e1413^{tree}` → `docs/function-design/90-traceability.md` だけ。
- 今の byte 比較は #197 で外れる: `git diff --binary --full-index --no-ext-diff --no-textconv 3573dc67...256c904e -- | sha256sum` → `dae6b528…`、`… a2365759...3c7e1413 -- | sha256sum` → `b5476485…`（違う）。
- code に触れない条件の分け方: #214 の同期の main 側（`git diff --name-only --no-renames a2365759 bbea3314 --` の 7 path、docs だけ）を `bash scripts/ci/classify-changes.sh --files-from-stdin` に渡すと `rust=false`・`frontend=false`・`env=false`・`unknown=false`（A）。#193 の同期の main 側（`3573dc67` から `a2365759`）は `src-tauri/**`・`src/**`・`src/lib/bindings.ts` を含む（B）。
- `src/routeTree.gen.ts` は tracked でなく（`.gitignore` にある）drift 検査の外なので、生成物の集合は `src/lib/bindings.ts` と `docs/function-design/90-traceability.md` の 2 path に固定する（H2 の定数）。どちらも hosted の `rust_drift` の job が再生成と差分で確かめる（`.github/workflows/ci.yml` の `bindings.ts drift check` と traceability の step）。
- 本 lane と #209 の decision-log の衝突（round 2）: `git -c merge.directoryRenames=false merge-tree --write-tree --name-only a4a110e1 ba57ca91` → exit 1、衝突は `docs/decision-log.md` だけ（#209 は D-140、本 lane は D-144 を同じ末尾に足す。merge-base は `bbea3314` の 1 つ）。#197 は衝突しない: 同じ command の `3c7e1413 ba57ca91` → exit 0。
- `.gitattributes` の `docs/decision-log.md merge=union` は最初の同期で効かない（round 2、`$TMPDIR` の使い捨ての `git clone --shared` で測り、ref は本 clone に残していない）: 本 lane の tree に `.gitattributes` を足した tree を `git commit-tree -p bbea3314` で合成した commit M（squash の merge の後の main を想定、`1e0c8ace…`）に対し、(1) `merge-tree --write-tree --name-only a4a110e1 M` → exit 1、`docs/decision-log.md` が衝突。(2) `a4a110e1` を checkout した作業 tree で `git -c merge.directoryRenames=false merge M` → exit 1、同じ衝突（属性は lane の作業 tree から読まれ、`a4a110e1` には `.gitattributes` が無い）。(3) 作業 tree に M の `.gitattributes` を置くか `-c attr.tree=M` を付けると `merge-tree` は exit 0（tree `4f489fd3…`）。属性は lane が取り込んだ後の同期でしか効かないので (a) を採らず、H2 の union の受理（(b)）にした。
- union の受理の値: 同じ M で `git merge-file -p --union --object-id a4a110e1:docs/decision-log.md bbea3314:docs/decision-log.md M:docs/decision-log.md` の出力の sha256 の先頭 `811e372d4c1004e1` は、(3) の union の merge の `docs/decision-log.md` の blob の sha256 の先頭と一致。出力に衝突の marker は無く、D-140 と D-144 の見出しが両方残る。衝突した tree と union の tree の違いは `git diff --name-only --no-renames` で `docs/decision-log.md` だけ。
- `.gitattributes` は classifier で `unknown`（full）に分かれる（`printf '.gitattributes\n' | bash scripts/ci/classify-changes.sh --files-from-stdin` → 9 flag が全部 true）。(a) を採らないので Scope に入れない。

## Data Safety

- commit しないもの: 実の PR の record の中身・owner の承認の原文・店のデータ。
- local だけ: `.local/pr-gate/` の capture（今のまま ignored）。
- 合成だけ: test の git repo（tmpdir）と fake gh の state。

## 後続

- Coordinator の運用の checklist（repo の外）の base 同期と Ready・merge の手順を MG-D13 に合わせる（Coordinator）。
- 移行の扱い（round 3、closure の是正で改めた）: 旧 helper で manual が pending に戻った PR（#197・#209。`gh api` の record で manual が `pending`・evidence 0 件）は、本 lane の merge の後に同期し、その head で L3 をやり直す（owner の決定 TD-504）。pending の manual に旧 L3 の evidence を `--reuse-from` なしで pass と記録し直すことはしない（使い回しの検査と L3 の版〈`source_head`〉の保持を迂回し、merge-evidence の「pending を CLI の pass 宣言で補完しない」と衝突する）。本 lane の helper の変更（`scripts/pr-gate.py`・`scripts/tests/pr-gate.test.py`）を取り込む最初の同期は classifier で code に分かれる同期なので、旧 pass が残っていても使い回しはどのみち B になる。helper は変えない。

## Implementation Results

実装の commit: `b3e53359`（`scripts/pr-gate.py`・`scripts/tests/pr-gate.test.py`）。起点 `3ac6fc22`。

| H | 実装（`scripts/pr-gate.py`） | 変えた呼び出し側 |
|---|---|---|
| H1 | `validate_record`: closure は `{head, base, audit}` か `{head, base, sync_from}` の一方（`keys` で形を 1 つに固定。両方・どちらも無しは `invalid record fields`）。どちらの形も broad を要る（`closure needs the broad it continues`）。manual は `source_head` だけ（A）か `source_head` と `reuse_approval`（B）、`reuse_approval` だけは `incomplete manual reuse` | — |
| H2 | `Gate.sync_merges(origin, head, base)`（first parent を起点まで辿る）と `Gate.sync_merge`（2 親の merge の検査。`merge-base --is-ancestor`・`merge-base --all`・`git -c merge.directoryRenames=false merge-tree --write-tree --name-only -z`・`docs/decision-log.md` の `merge-file -p --union --object-id` の byte 照合）。生成物の集合は定数 `GENERATED`（2 path）、union の path は `UNION`。path の列挙は `changed()`（`git diff --name-only --no-renames -z`）。終了値 0/1 を受ける呼び方は `command(..., status=True)` / `git(..., status=True)` の 1 つの入口 | `reuse_geometry` を削除 |
| H3 | `Gate.code_paths(merges, base)`（各 merge の `merge-base..second` の path の和集合を current main の classifier に渡し、`CODE_FLAGS` の 4 flag で判定、code の path を 1 つずつ名指す）。classifier の呼出しは module の `classify(classifier, paths)` にまとめ、`Gate.requirements` も使う。H2 と H3 の入口は `Gate.reusable(origin, head, base, code=True)`（判定不能の exit 2 に `git fetch origin` を添える） | `Gate.requirements` |
| H4 | `Gate.carry_manual(old, snap)`。`Gate.record` の版が変わった分岐で、旧 manual が pass・manual が必須・`--kind` が manual でないときに呼ぶ。head が同じで base だけが変わった版は「base が進んだ。先に同期する」で止める。H2 で外れたら `--outcome pending` の command だけ、H3 だけで外れたら B の完全な command（`Gate.cli`）と `--outcome pending` の command を出す。どちらも書込みの前 | `Gate.record` |
| H5 | `Gate.sync_closure(...)`、argparse の `--review-stage` に `sync`。監査の引数と pass 以外の outcome は exit 2。`current` が真なら順序の文で止める。起点は監査の closure の head・`sync_from`・broad の head。投稿済みの本文のある PR review は `Gate.posted_reviews`（全 page、`--pr-reviews` の検査と共有）で数えて止める | `Gate.record` の review の分岐、`main` |
| H6 | `Gate.record` の manual の分岐。`--reuse-approval` は `--reuse-from` と組でだけ（exit 2）。`--reuse-from` は旧 manual の `source_head` か旧 head と一致、evidence 一致、旧 manual が使い回しでも受ける。H3 は `--reuse-approval` が無いときだけ。`reuse_approval` は無ければ旧 manual から引き継ぐ | `Gate.record` |
| H7 | `Gate.nonci`: manual の `source_head` から H2（`reuse_approval` が無ければ H3 も）、closure の `sync_from` から H2 と H3 を当て直す。`validate_review` は変えない（`sync_from` の形も head/base で照合される） | `Gate.nonci` |

test（`scripts/tests/pr-gate.test.py`）: 78 → 123（`PYTHONDONTWRITEBYTECODE=1 python3 scripts/tests/pr-gate.test.py` → `Ran 123 tests` `OK`）。合成の repo は `Synth`（tmpdir、`git init -b main`、helper の `git` を `-C tmpdir` へ向ける patch、classifier は repo の `scripts/ci/classify-changes.sh` を `Gate.contents` の patch で渡す）。

- 足した test: `SyncOnly` 14（G1〜G4・G6〜G13 と G5 の unit、`merge.directoryRenames=false` の組 `test_sync_only_uses_directory_renames_false`）、`CodeFree` 5（C1〜C5）、`SyncRecords` 22（R1〜R13、S2〜S4・S6・S7、V5・V6、G5 の record と status、C6）、`Records` 2（V1 `test_validate_legacy_reuse_record`、V4 `test_validate_closure_shape_exclusive`）、`ReviewedHead` 3（fake gh の process で S1・S5・S8）。V2 は既存の `Records.test_closure_required_for_head_or_base_change`。
- 書き換えた既存の test: `Records.test_wire_fail_closed` の `manual.source_head` だけの mutant を `reuse_approval` だけの mutant に置換（V3。`source_head` だけは H1 で A の形）。`RecordLifecycle.exercise` は `reuse_geometry` の patch を `sync_merges`・`code_paths` の stub に替え、head/base/req・承認・stage・PR review・合成の repo を受けるようにした。`test_closure_first_loses_old_manual_and_cannot_restore_cache` を `test_closure_first_carries_or_stops_and_keeps_b` に置換（H4 で closure を先に記録しても A は引き継ぎ、code の同期は書かずに止まって B が残る）。`GitReuse.test_exact_merge_and_changed_patch` は `SyncOnly` に置換。`test_reuse_then_closure_order`・`test_source_record_missing_changed_or_not_pass` は変えずに通る。comment の `# MG-D6 / GA1` は MG-D13 の文に書き換えた。

mutant（clean な tree に 1 つずつ注入、対応の test が red、`git checkout` の後に `git diff --exit-code` で復元を確認）: Mutation-style Adequacy Questions の (1)〜(12) は全部 red。追加の 7 つ（G2 の 1 段だけ辿る・G4 の merge-tree の exit 1 を失敗扱い・R6 の使い回しの使い回しを拒む・R12 の base だけの変化を pending に戻す・R13 の `--reuse-from` の無い manual に H4 を当てる・S7 の `sync_from` に旧 head を書く・V4 の closure の両形を optional）も全部 red。

検査: `PYTHONDONTWRITEBYTECODE=1 bash scripts/tests/run-workflow-tests.sh` exit 0。`bash scripts/doc-consistency-check.sh` exit 0（WARN 13、ERROR なし）。`--target plan docs/plans/2026-10-27-l3-reuse-after-sync.md` 全チェック通過。`bash scripts/check-workflow-git.sh` exit 0。AC6 の `rg` 0 件（exit 1）。

## Review Response

### Plan Review round 1（起点 `bc50e4e90a4038ff11a0f601567571a3d502da38`、判定 reject）

Codex と fresh Opus 5.5 の findings を Coordinator が裁定し、下の表の全件を受理した（受理しなかったもの: なし。各 finding の重大度は表の列）。

| ID | 出所 | 重大度 | 処置 | 直した節 |
|---|---|---|---|---|
| C1 | Codex #1 | P1 | H2 の path の列挙（tree の比較と 1 親の commit）を全部 `--no-renames` に固定し、通常の path → 生成物への rename を拒む | packet H2・AC1、merge-evidence「同期だけの条件」、D-144 Decision (1)、Matrix G12 |
| C2 | Codex #2・Opus F1 | P1/P2 | 「head の second parent が base」を「列の中で最後の merge の second parent が base、その後ろは (b) だけ」に統一 | packet H2・AC1・Contract Audit（0 の表示）、merge-evidence「同期だけの条件」、D-144 Decision (1)、Matrix G5・G10・Boundary Checks |
| C3 | Codex #3・Opus F2 | P2 | 明示の A（`--reuse-from` だけ）は旧 manual の `reuse_approval` を引き継ぐ | packet H6・AC3、merge-evidence「manualの使い回し」・「Helperの境界」、Matrix R11 |
| C4 | Opus F3 | P2 | head が同じで base だけが変わった版の pending へ戻す除外を消し、H4 の止める形（理由「base が進んだ。先に同期する」）に通す | packet H4・Contract Audit（失敗の時点と順序）・不要になる記述・後続、merge-evidence「manualの使い回し」・「Helperの境界」、D-144 Decision (6)、Matrix R12 |
| C5 | Opus F5 | P3 | `merge-tree --write-tree` と `merge-base --is-ancestor` の終了値 0/1 は結果、それ以外は判定不能 | packet H2・Contract Audit、merge-evidence「同期だけの条件」、Matrix G4・G9・Negative Paths |
| C6 | Opus F6 | P3 | 生成物の集合を drift 検査の当たる 2 path の helper の定数に固定し、`src/routeTree.gen.ts` を外す | packet H2・Contract Probe、merge-evidence「同期だけの条件」、D-144 Decision (1)、Matrix の前文・Adjacent Pattern Audit |
| C7 | Opus F7 | P3 | B の command を `--outcome pass --evidence <旧 evidence>` を含む完全な形にし、R0/R1 の lane の B の closure の証跡を `reuse_approval` の pointer の先に置く | packet Ordinary Operation・H4・H6、merge-evidence「manualの使い回し」・「Helperの境界」 |
| C8 | Opus F8 | P3 | D-144 の Why の「docs の相互作用は doc の検査が見る」を直し、Guarantee range に受け入れた限界（TD-476）を書く | D-144 Why・Guarantee range |
| C9 | Opus F4（最小の形） | P3 | `--review-stage sync` は新しい head での最初の record でだけ使える順序を正本と止める理由の文に書く（自動化しない） | packet H5・Contract Audit、merge-evidence「reviewのclosure」・「Helperの境界」、Matrix S4 |
| C10 | Codex の補足 | — | mutant の注記: G9 は途中の main の外の merge と最後の正常な同期を分け、S2 は manual 不要の fixture を使う | Matrix Mutation-style Adequacy Questions |

### Plan Review round 2（起点 `ba57ca91af3bfa256a564819848d6208512b5711`、判定 reject）

Codex と fresh Opus 5.5 の findings を Coordinator が裁定し、下の表の全件を受理した（受理しなかったもの: なし。各 finding の重大度は表の列）。

| ID | 出所 | 重大度 | 処置 | 直した節 |
|---|---|---|---|---|
| D1 | Opus F1 | P1 | 本 lane と #209 の decision-log の末尾の追記の衝突。`.gitattributes` の `merge=union`（(a)）を実測し、最初の同期の `git merge` と `merge-tree` で効かないので採らず、H2 に「`docs/decision-log.md` の衝突は `git merge-file --union` の出力と byte で一致するときだけ受ける」（(b)）を足した | packet H2・AC1・Design Readiness（絶対保証の自己点検）・Contract Probe、merge-evidence「同期だけの条件」、D-144 Decision (1)、Matrix G13 |
| D2 | Opus F2 | P2 | DEV_WORKFLOW の L3 の箇条に、生成物と decision-log の union の外で衝突する見込みのある PR は先の merge と同期の後に L3 を頼む 1 文を残し、backlog の観測の項目を更新 | DEV_WORKFLOW Wave Operation の L3 の箇条、packet の不要になる記述・Scope の予定の file と所有、D-144 Compatibility、`docs/backlog.md` |
| D3 | Opus F3 | P2 | 使い回せないときの案内を理由で分ける（H2 で外れたら pending だけ、H3 だけなら B と pending） | packet H4・Ordinary Operation の 4 行目・AC3、merge-evidence「manualの使い回し」、Matrix R3・R4 |
| D4 | Codex #1 | P2 | `--review-stage sync` でも今の head に本文のある review が投稿済みなら、id と時刻と通常の closure の手順を示して書込みの前に止める | packet H5・AC4、merge-evidence「reviewのclosure」・「Helperの境界」、Matrix S8 |
| D5 | Codex #2 | P2 | mutant (3) を「`unknown=true` の分類結果を code-free と誤って受理する」に置き換え | Matrix Mutation-style Adequacy Questions |
| D6 | Opus F4 | P3 | `--kind manual` で `--reuse-from` が無いときは H4 の判定を省き、H3 は H2 を満たしたときだけ評価する | packet H4・AC3、merge-evidence「manualの使い回し」、Matrix R13 |
| D7 | Opus F5 | P3 | H1 に「`sync_from` の closure で broad が null なら拒む」 | packet H1、Matrix V4 |
| D8 | Opus F6 | P3 | 終了値を受け取る git の呼び方も `git()` と同じ入口を通し、test の patch が効く形にする | packet H2、Matrix の前文 |

### Plan Review round 3（起点 `d277f2b73625964edafd7e568f4690d8976590ec`、判定 reject。round の天井）

上限の到達: Coordinator の disposition = まとめて是正（`docs/DEV_WORKFLOW.md` Review Rules）。是正の確かめは owner の判断を待つ。Codex と fresh Opus 5.5 の findings を Coordinator が裁定し、下の表の全件を受理した（受理しなかったもの: なし）。

| ID | 出所 | 重大度 | 処置 | 直した節 |
|---|---|---|---|---|
| E1 | Codex #1 | P1 | H3 を、起点と最後の base の差分から、同期の列の各 merge の `merge-base(FIRST, SECOND)` → SECOND の変更の path の和集合へ置き換え（途中で main が code を変えて戻す列を A で通さない）。H4〜H7 は同じ関数を使う | packet H3・AC2、merge-evidence「codeに触れない条件」、D-144 Decision (2)、Matrix C6・mutant (12) |
| E2 | Opus #1 | P2 | 旧 helper で manual が pending に戻った PR（#197・#209）の移行の扱い（owner の判断で今の head に manual の pass を記録するか L3 をやり直す。helper は変えない） | packet Ordinary Operation の行 2・`## 後続`、Matrix Residual Test Gaps |
| E3 | Opus #2 | P3 | decision-log の衝突の解き方の command を正本に 1 句 | merge-evidence「同期だけの条件」、DEV_WORKFLOW Wave Operation の L3 の箇条 |
| E4 | Opus #3 | P3 | 明示の A（`--reuse-from` だけ）は R0/R1 の lane で使うと正本に 1 文 | merge-evidence `## Helperの境界` |
| E5 | Opus #4 | P3 | 判定不能の次の一手に `git fetch origin`。status は ref と作業 tree を変えない（`merge-tree --write-tree` は object だけを書く） | packet H4、merge-evidence「同期だけの条件」・`## Helperの境界` |

### 是正の確かめ（closure）の表（起点 `ee0386b014a98918105d8480471709afe29f4db0`）

Codex は reject（round 3 の P1 は計画上 closed、是正で足した移行の手順に新しい P2）、Opus は N1・N2（P3）。owner の決定（repo 外の回答台帳 TD-504）で移行の手順を L3 のやり直しに改めた。

| ID | 出所 | 重大度 | 処置 | 直した節 |
|---|---|---|---|---|
| F1 | Codex #1 | P2 | 移行の「今の head に manual の pass を `--reuse-from` なしで記録し直す」選択肢を消し、#197・#209 は本 lane の merge の後に同期してその head で L3 をやり直す（TD-504）とした | packet Ordinary Operation の行 2 の注記・`## 後続`、Matrix Residual Test Gaps |
| N1 | Opus N1 | P3 | 本 lane の helper の変更を取り込む最初の同期は code に触れるので、使い回しはどのみち B になる旨を F1 の文に入れた | packet `## 後続` |
| N2 | Opus N2 | P3 | `source_head` の無い pass で L3 の版が消える問題は、F1 で記録し直す選択肢ごと消えた | （F1 に同じ） |
先行 round の結果・評価（判定・件数・採否・reviewer の意見）はこの節にだけ書く。前半の節と遷移記録には「round N の是正（Review Response 参照）」だけを書く（独立 review は `## Review Response` より前だけを読む）。
