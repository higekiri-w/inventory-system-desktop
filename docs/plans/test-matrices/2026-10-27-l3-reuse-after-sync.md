# Test Design Matrix: 同期の後の L3 と closure の使い回し

Plan Packet: `docs/plans/2026-10-27-l3-reuse-after-sync.md`。契約の正本は `docs/agent-guidance/merge-evidence.md` の MG-D13 と `## base同期だけでheadが変わる場合`、決定は D-144。

## Risk

Risk: R3

## Contracts Under Test

- MG-D13 同期だけの条件（packet の H2）
- MG-D13 code に触れない条件（H3）
- manual の A・B・止める形（H4・H6）
- 同期の closure（H5）
- RecordV1 の Closure と manual の形（H1）と確かめ直し（H7）
- R4 を使い回さない（H4）

## Failure Modes

- lane の手（同期の後の commit、merge の中の編集、衝突の手での解消、main の外の branch の取込み）が入った版で使い回す。
- 生成物の例外が生成物の外へ広がる、または生成物の衝突を受けられず #197 の型で外れる。
- code に触れる main 側の変更（`src-tauri/**`・`src/lib/bindings.ts`・依存・CI の実行 code・未知の path）を A で通す。
- 使い回せないときに manual の pass を黙って失う。
- `sync_from` の closure が broad の必要数・Plan 契約の変更の検査を飛ばす。
- 今の形の record を拒む（互換の破れ）。
- R4 を引き継ぐ。

## Test Matrix

合成の git repo は `GitReuse` の今の作り方（tmpdir、`git init -b main`、`g.git` を tmpdir へ向ける patch）を使う。終了値を受け取る git の呼び方（`merge-tree --write-tree`・`merge-base --is-ancestor`）も `git()` と同じ入口（`command`）を通すので、test はその入口を patch し、本 clone の git を読まない。生成物の衝突は `docs/function-design/90-traceability.md` の同じ行を両側で変え、merge の commit で別の内容に書き直して作る。classifier は repo の `scripts/ci/classify-changes.sh` を tmpdir に写して current main 側の版として読ませる（`Gate.contents` を patch。H3 だけが使う）。生成物の集合は helper の定数（`src/lib/bindings.ts`・`docs/function-design/90-traceability.md`）で、classifier から得ない。oracle は merge-evidence の MG-D13 の文から取り、helper の出力から取らない。

| ID | Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|---|
| G1 | 同期だけの条件 | 1 回の自動の merge を拒む | unit（合成 repo） | `test_sync_only_single_auto_merge` | tree の比較を byte の差分の比較のままにする、または 1 回の merge を拒む |
| G2 | 同上 | 2 回の同期を拒む（#209 の型） | unit | `test_sync_only_two_merges` | first parent を 1 段しか辿らない（今の `head^1 == source`） |
| G3 | 同上 | 共有の file の周りの行がずれた同期を拒む | unit | `test_sync_only_shared_file_context_shift` | 差分の byte・patch の比較に戻す（main 側が lane の hunk の隣の行を変える） |
| G4 | 同上 | 生成物だけの衝突を再生成した merge を拒む（#197 の型） | unit | `test_sync_only_generated_conflict_regenerated` | 衝突を全部拒む、生成物の集合を読まない、または `merge-tree --write-tree` の exit 1（衝突あり）を判定不能（`command failed`）にする |
| G5 | 同上 | 同期の merge の後に生成物だけを再生成した 1 親の commit が head の版を拒む（record と status の両方） | integration（合成 repo と `RecordLifecycle`・`nonci`） | `test_sync_only_generated_regen_commit` | 1 親の commit を全部拒む、または head 自身の second parent を求める（末尾が 1 親の head で record の A が止まる、status が blocker を出す） |
| G6 | 同上 | 同期の後の lane の commit を受ける | unit | `test_sync_only_rejects_lane_commit` | 1 親の commit を path を見ずに受ける。理由に commit の SHA が無い |
| G7 | 同上 | 生成物の外の衝突を手で解いた merge を受ける | unit | `test_sync_only_rejects_manual_conflict_resolution` | 衝突の path を生成物で絞らない |
| G8 | 同上 | 衝突の無い merge の中の lane の編集（evil merge）を受ける | unit | `test_sync_only_rejects_edit_inside_merge` | tree を `merge-tree` と比べず、2 親であることだけを見る。理由に path が無い |
| G9 | 同上 | main の外の branch を second parent に持つ merge を受ける | unit | `test_sync_only_rejects_non_main_second_parent` | second parent が base の祖先かを見ない（lane の編集を別 branch から取り込める）。`merge-base --is-ancestor` の exit 1（祖先でない）を判定不能にし、理由に merge の SHA が無い |
| G10 | 同上 | first parent の列の最後の merge の second parent が base でない版を受ける | unit | `test_sync_only_rejects_stale_last_parent` | 列の最後の merge の second parent と base を照らさない（その後ろに再生成の 1 親の commit がある組も置く） |
| G11 | 同上 | merge-base が一意でない merge を受ける | unit | `test_sync_only_rejects_ambiguous_merge_base` | `merge-base --all` の行数を見ない |
| G12 | 同上 | 生成物の例外で生成物の外の path の違いを受ける。通常の path から生成物への rename（merge の中と 1 親の commit の両方）を受ける | unit | `test_sync_only_generated_exception_is_exact` | 生成物の集合を prefix・basename で当てる（`docs/function-design/90-traceability.md.bak`・`src/lib/bindings.tsx` を生成物と誤る）。H2 の `git diff --name-only` に `--no-renames` を付けない（`public/help.html` → `docs/function-design/90-traceability.md` の rename が生成物 1 path に見える） |
| G13 | 同上 | `docs/decision-log.md` の末尾の追記どうしの衝突を union の結果で解いた merge を拒む、または union と違う形（片側を落とす・手で並べ替える）で解いた merge を受ける | unit（合成 repo） | `test_sync_only_decision_log_union_conflict` | `docs/decision-log.md` の衝突を全部拒む、または全部受ける（`git merge-file -p --union --object-id` の出力と byte で比べない）。理由に merge の SHA と path が無い |
| C1 | code に触れない条件 | docs だけの main 側を code と誤る | unit | `test_code_free_docs_only` | `docs` か `traceability` の flag で code と判定する |
| C2 | 同上 | `src-tauri/**` を通す | unit | `test_code_free_rejects_rust` | `rust` を見ない。理由に path が無い |
| C3 | 同上 | `src/lib/bindings.ts`・`package.json` を通す | unit | `test_code_free_rejects_frontend_and_generated_binding` | `frontend` を見ない、または `generated` の flag で生成物を docs 扱いにする |
| C4 | 同上 | 未知の path・`scripts/pr-gate.py` を通す | unit | `test_code_free_rejects_unknown_and_control` | `unknown` を見ない、full に送る path を code から外す |
| C5 | 同上 | rename の旧 path（`src/x.ts` → `docs/x.md`）を見落とす | unit | `test_code_free_rename_both_sides` | `--no-renames` を使わず新しい path だけを渡す |
| C6 | 同上 | 同期の列の途中で main が code を変え、次の同期の前に戻した列（main が `src/app.ts` を変える → 同期 → main が戻し docs も変える → 同期）を A で通す | integration（合成 repo と `RecordLifecycle`） | `test_code_free_union_over_sync_merges` | 起点と最後の base の差分だけを classifier に渡す（`src/app.ts` が和集合に入らず、manual の A と `--review-stage sync` が通る） |
| R1 | manual の A | docs だけの同期で manual を引き継がない | integration（`RecordLifecycle`） | `test_auto_carry_on_sync_record` | 版が変わった分岐で manual を常に pending に戻す |
| R2 | 同上 | 引き継いだ manual の `source_head` と evidence が元と違う | integration | `test_auto_carry_keeps_source_and_evidence` | `source_head` に旧 head（使い回しの版）を書く、evidence を差し替える |
| R3 | 止める形 | code に触れる同期（同期だけの条件は満たす）で closure の record が manual を黙って失う | integration | `test_code_sync_stops_non_manual_record` | 止めずに pending へ戻す。出力に code の path と、`--outcome pass --evidence` を含む B の command と `--outcome pending` の command の両方が無い |
| R4 | 同上 | lane の commit のある版で止まらない・理由が無い・通らない B を案内する | integration | `test_lane_commit_stops_with_reason` | 理由に commit の SHA が無い。何かを書いてから止まる（PATCH が呼ばれる）。出力に `--reuse-approval` の B の command が出る（同期だけの条件で外れたら `--outcome pending` だけを出す）。code に触れない条件を評価する（同期だけの条件を満たさないときに H3 の git を呼ぶ） |
| R5 | manual の B | code に触れる同期で B を受けない、または承認なしで受ける | integration | `test_b_reuse_with_approval` | `--reuse-approval` を見ずに H3 を当てる、または承認の無い B を受ける |
| R6 | A の後の A | 2 回の docs だけの同期で 2 回目を拒む | integration | `test_auto_carry_chain` | 旧 manual が使い回しなら拒む（今の `'source_head' not in old['manual']`） |
| R7 | B の後の A | B の後の docs だけの同期で承認を失う、または拒む | integration | `test_b_then_docs_sync_keeps_approval` | `reuse_approval` を落とす、起点を `source_head` にして H3 で拒む |
| R8 | B の source | `--reuse-from` が旧の `source_head` と違っても受ける | integration | `test_b_source_must_match_original` | `--reuse-from` を旧 head とだけ照らす |
| R9 | A の明示 | `--reuse-from` だけ（承認なし）で code に触れる同期を受ける | integration | `test_explicit_a_requires_code_free` | 承認が無いときに H3 を当てない |
| R10 | R4 | 同期の後に R4 の pass を引き継ぐ | integration | `test_r4_never_carried` | 版が変わった分岐で r4 も引き継ぐ |
| R11 | 明示の A の承認の引継ぎ | B → docs だけの同期 → `--reuse-from` だけの明示の A の版で、record は通るのに status が止まる | integration（`RecordLifecycle`・`nonci`） | `test_b_then_explicit_a_keeps_approval` | 明示の A で旧 manual の `reuse_approval` を落とす（`nonci` が `source_head` から H3 を当て直し、前の同期の code の取込みで blocker を出す） |
| R12 | 止める形 | head が同じで base だけが変わった版の manual 以外の record が manual を黙って pending に戻す | integration | `test_base_only_change_stops_non_manual_record` | head が同じときに pending へ戻す除外を残す。何かを書いてから止まる。理由に「base が進んだ。先に同期する」が無い |
| R13 | 判定を省く出口 | `--kind manual` で `--reuse-from` の無い record（`--outcome pending` で L3 をやり直す）が使い回しの判定で止まる | integration | `test_manual_without_reuse_skips_reuse_check` | `--reuse-from` の無い manual の record にも H2・H3 を当てる（旧 head の object が無い checkout で `command failed` の exit 2 になり、L3 をやり直せない） |
| S1 | 同期の closure | `--review-stage sync` が `sync_from` の closure を書かない | integration（fake gh の process） | `test_sync_stage_records_machine_closure` | argparse の choices に `sync` が無い。PATCH の body に `sync_from` が無い |
| S2 | 同上 | code に触れる同期で記録する | integration | `test_sync_stage_rejects_code_sync` | H3 を当てない。理由と「監査の closure を要る」が出ない |
| S3 | 同上 | 旧 review が pass でないのに記録する | integration | `test_sync_stage_needs_passed_review` | 旧 review の outcome を見ない |
| S4 | 同上 | 版が変わらない record で記録する（新しい head で先に他の kind を記録した後を含む） | integration | `test_sync_stage_needs_changed_head` | `current` の分岐を見ない。止める理由に「新しい head での最初の record でだけ使える」順序が無い |
| S5 | 同上 | 余計な引数（`--pass-model` ほか）を黙って受ける | CLI | `test_sync_stage_rejects_audit_arguments` | 入力の誤り（exit 2）にしない |
| S6 | 同上 | Plan 契約の変わった版で broad を引き継ぐ | integration | `test_sync_stage_requires_same_plan_contract` | broad の plan_commit・amendments を照らさない |
| S7 | 同上 | 起点を誤る（`sync_from` の連鎖で最後の監査の版を失う） | integration | `test_sync_stage_chain_keeps_audited_origin` | 2 回目の `sync_from` に 1 回目の head を書く |
| S8 | 同上 | 今の head に本文のある PR review が投稿済みなのに sync の closure を書く（D-107 の取りこぼし） | integration（fake gh の process） | `test_sync_stage_stops_on_posted_review` | PR review を数えない、または 1 page だけ数える。止める出力に review の id・投稿時刻・`--review-stage closure` の手順が無い。PATCH が呼ばれる |
| V1 | 互換 | 今の B の record（`source_head` と `reuse_approval`）を拒む | unit | `test_validate_legacy_reuse_record` | `validate_record` の manual の形を A の形だけにする |
| V2 | 互換 | 今の監査の closure を拒む | unit | 既存の `Records` の closure の case（`rg -n "closure=dict\(head=H" scripts/tests/pr-gate.test.py` で確かめた所） | closure の形の分岐で `audit` を落とす |
| V3 | 形 | `reuse_approval` だけの manual を受ける | unit | `Records` の mutant（`manual.source_head` だけの mutant を置き換える） | `reuse_approval` を `source_head` と組で求めない |
| V4 | 形 | closure の 2 つの形の混在（`audit` と `sync_from`）・どちらでもない形、broad が null の `sync_from` の closure を受ける | unit | `test_validate_closure_shape_exclusive` | `keys` の optional に両方を入れる。`sync_from` の形で broad の非 null の検査を外す |
| V5 | 確かめ直し | status が `source_head` の後の lane の commit を見逃す | integration（`nonci`） | `test_nonci_rechecks_manual_chain` | `nonci` が `head^1` だけを見る、または検査を外す |
| V6 | 確かめ直し | status が `sync_from` の closure の code の取込みを見逃す | integration（`nonci`） | `test_nonci_rechecks_sync_closure` | `sync_from` の closure に H3 を当て直さない |

## State Lifecycle Matrix

| State / subject | Initial | Pending | Success | Invalidate | Refetch | Revisit | Restart | Failure | Retry | Evidence |
|---|---|---|---|---|---|---|---|---|---|---|
| manual（record） | 旧 head で pass | 新しい head の capture | A で引き継ぐ（R1）/ B で使い回す（R5） | lane の commit・code の取込み・base だけの変化で外れる（R3・R4・R12） | capture の後の server の変化で fresh capture（既存） | B の後の docs の同期（R7・R11） | helper の再実行は record を読み直す（既存） | 何も書かずに止まる（R3・R4） | `--outcome pending` で外して L3 をやり直す（R3 の出力） | R1〜R13 |
| review の closure | 旧 head で pass | 新しい head の capture | `sync_from` の closure（S1） | code の取込みで外れる（S2） | 同上 | 2 回目の同期（S7） | 同上 | 止まる（S2〜S6・S8） | 監査の closure を記録する（既存） | S1〜S8 |
| status の確かめ直し | record が現在の head | — | 通る（V5・V6 の正の組） | 起点の後の lane の commit で blocker（V5） | status は毎回 git を読む | — | — | blocker を出す | — | V5・V6 |

capture / server の race、stale head/base、broad / closure、R4 の扱いは既存の test（`test_source_record_missing_changed_or_not_pass` の `mutate_capture`、`test_changed_plan_requires_new_broad` ほか）を変えずに保つ。

## Adjacent Pattern Audit

| Source pattern / contract | Repository sites inspected | Ported sites | Explicit exclusions and reason | Test / evidence |
|---|---|---|---|---|
| current main 側の classifier を `--files-from-stdin` で呼ぶ（`Gate.requirements`） | `rg -n "classify-changes" scripts/pr-gate.py`（1 か所） | H3 が同じ関数を使う | — | C1〜C5 |
| `merge.directoryRenames=false` の同期（DEV_WORKFLOW Wave Operation・Stacked train、merge-evidence の同節） | `rg -n "directoryRenames" docs scripts --glob '!docs/archive/**'` | H2 の `merge-tree` | — | G1〜G4 |
| 生成物の path の集合（hosted の drift 検査が当たる path） | `scripts/ci/classify-changes.sh` の `generated=true` の case、`.github/workflows/ci.yml` の `rust_drift` の job の drift の step | H2（helper の定数の 2 path） | `src/routeTree.gen.ts` は tracked でなく drift 検査の外なので集合に入れない。classifier の case を解析して集合を得ない | G4・G5・G12 |

## Negative Paths

- missing input: `--review-stage sync` で旧 record が無い（S3 の旧 record 無しの組）。
- invalid input: `--review-stage sync` と監査の引数（S5）。`--reuse-approval` だけ（入力の誤り、R5 の負の組）。
- duplicate/ambiguous input: merge-base が一意でない（G11）。closure の 2 つの形の混在（V4）。
- unknown reference: `--reuse-from` が旧の source と違う（R8）。first parent を辿って起点に着かない（G6 の変形）。
- dependency missing: `git merge-tree` が失敗（終了値 0/1 は判定の結果、それ以外は exit 2 で止まり満たした扱いにしない。G の各 test の `command failed` の組を 1 つ置く）。
- permission/write failure: 止める形は PATCH の前（R4 で PATCH が呼ばれないこと）。
- dry-run side effect: status（`nonci`）は書かない（V5・V6 で comment の API が呼ばれない）。

## Boundary Checks

- threshold: 同期の回数に上限なし（G2 は 2 回）。
- null/default: closure が null の旧 record の起点は broad の head（S7 の初回）。
- empty/non-empty: main 側の path が空（起点と base が同じ）は列の最後の merge の second parent が base でも、列に merge があるときだけ。
- min/max: —
- status/policy enum: outcome は pass だけを引き継ぐ（R1 の負の組で pending・fail を拒む）。
- wire type: closure の 2 つの形（V4）。
- internal type: —
- producer/consumer: `record` が書き `nonci` が読む（V5・V6）。
- round-trip token: `record read-back mismatch`（既存）。
- precision/range: SHA は 40 桁（既存の `sha`）。
- cross-language parse: —

## Compatibility Checks

- old schema/input: V1・V2。
- new schema/input: S1・R1・R5。
- output order: —
- optional field behavior: `reuse_approval` は B の後の A で残る（R7）。

## Data Safety Checks

- source-derived data: 無い（合成の repo）。
- generated outputs: 合成の生成物の file は tmpdir の中だけ。
- secrets: 無い。
- local-only files: capture は tmpdir の `.local/pr-gate`（既存の `exercise`）。
- synthetic sample boundaries: 実の PR の SHA は Contract Probe の証跡だけで、test の fixture に使わない。

## Main Wiring / Integration Checks

- helper connected to main path: `record` の版が変わった分岐（R1）と `nonci`（V5）。
- output reaches manifest/report: PATCH の body（S1）。
- effective config reaches runtime: `merge.directoryRenames=false` が `merge-tree` に渡る（G の合成で directory rename を起こす組を 1 つ置く）。
- CLI arg reaches implementation: `--review-stage sync`（S1・S5）、`--reuse-from` だけ（R9）。

## Mutation-style Adequacy Questions

実装の run は次の mutant を production 側に注入し、該当の test が red になることを確かめて PR に書く: (1) H2 の tree の比較を外す → G8、(2) second parent の祖先の検査を外す → G9（途中の main の外の merge と最後の正常な同期を分けた列にし、列の最後の merge の second parent と base の照合が mutant を覆わないようにする）、(3) H3 の判定で `unknown=true` の分類結果を code-free と誤って受理する（`unknown` が真なら他の flag を見ずに満たす）→ C4 の未知の path 単独の組（実の classifier は未知の path で `rust`・`frontend`・`env`・`unknown` を全部 true にするので、`unknown` の検査を外すだけの mutant は他の 3 flag に覆われて red にならない）、(4) H4 の止めを pending へ戻すに変える → R3、(5) H5 の H3 を外す → S2（manual 不要の fixture を使い、H4 の manual の止めが mutant を覆わないようにする）、(6) H7 の manual の確かめ直しを外す → V5、(7) 生成物の集合の照合を prefix にする → G12、(8) H2 の `--no-renames` を外す → G12、(9) H6 の `reuse_approval` の引継ぎを外す → R11、(10) H2 の `docs/decision-log.md` の union の照合を外して衝突を全部受ける → G13、(11) H5 の投稿済みの review の検査を外す → S8、(12) H3 の各 merge の path の和集合を起点と最後の base の差分に戻す → C6。

## Residual Test Gaps

- GitHub の Update branch の merge の実物は試さない（合成の repo は `git merge` だけ。違えば外れる側なので安全）。
- 実の PR の record での end-to-end（Ready・merge まで）は本 lane の PR 自身では起きない（本 lane は manual を持たない）。本 lane の merge の後に L3 を pass で記録した manual の lane の、最初の同期で Coordinator が確かめる。#197・#209 は実例にしない（旧 helper で manual が pending に戻っていて A も B も当たらず、本 lane の merge の後に同期してその head で L3 をやり直す。owner の決定 TD-504、packet の `## 後続` の移行の扱い）。
