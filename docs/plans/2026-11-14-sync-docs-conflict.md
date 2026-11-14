# Plan Packet: 同期で docs だけが衝突した lane の L3 の使い回しを直す

wave に属さない単独の lane。merge の helper（`scripts/pr-gate.py`）の同期だけの条件と manual の使い回し B、その test、正本の文（`docs/agent-guidance/merge-evidence.md` の「base同期だけでheadが変わる場合」、`docs/DEV_WORKFLOW.md` の L3 の表、`docs/decision-log.md`）を変える。製品の runtime（`src/**` の非 test・`src-tauri/**`）・CI の job・classifier は変えない。

出所: owner の決定 TD-704（repo 外の回答台帳、2026-11-14）「同期で docs だけが衝突した lane が L3 を使い回せずにやり直しになるのは理屈に合わないので、規則を直す。今後も必ず同じ状況になる」。owner は PR #123（2026-09-29）で manual の再利用の条件を緩めず、D-144（2026-10-27）で生成物と `docs/decision-log.md` の union だけを受けた。TD-704 はそれより後の決定で、残っていた backlog の観測の項目（「衝突を解いた版の manual の再利用（観測）」）に owner が答えたもの。手で解いてよい path の範囲は owner の決定 TD-706（同じ台帳、2026-11-14。下の「Open Questions」の決定の記録）で (b) に決まった。起点の事実は下の「調べ」。

decision-log の番号: 本 lane は D-154 を予約する。2026-11-14 に `git fetch origin` の後で引き直した: `git grep -n "^## D-15" origin/main -- docs/decision-log.md` の最後は D-152（`a90b0d6`）、`git grep -n -E "D-15[3-9]" <open の 7 本の origin/agent/* branch> -- docs/decision-log.md docs/plans` の当たりは PR #250（`agent/window-close-guard`）の D-153 だけで、D-154 以降は無い。

安全の境界: この変更は `AGENTS.md`「継ぎ足さない」と `docs/DEV_WORKFLOW.md` Workflow State（D-098）が挙げる安全の境界のうち「古い証拠の拒否」を緩める向きに変える。改訂は通常の packet・PR の経路で行い（本 packet）、owner の決定の日付は TD-704 と TD-706（どちらも 2026-11-14）。何を新たに受け、何を今までどおり拒むかを下の「境界ごとの受けると拒む」に書く。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R3

Reason:
merge の関門（helper の同期だけの条件・manual の使い回し・closure）と gate の文書（`docs/agent-guidance/merge-evidence.md`・`docs/DEV_WORKFLOW.md`）を変える。classifier が `scripts/pr-gate.py` を full、gate の文書を `workflow=true` にするので R2 にできず、Final は 2 本（Double Audit、Codex を含む。座組表）。R4 にしない理由: 破壊的な操作・実データ・secret・不可逆な git の操作を伴わず、RecordV1 の wire の形も GitHub への書込みの経路も変えない。

## Goal

Goal Invariant:

### 最小完了条件

- L3 の PASS の後に main と同期し、その merge が生成物と `docs/decision-log.md` の union の外で、許す path の集合（下の「境界ごとの受けると拒む」の「手で解いてよい path」。TD-706 の (b): docs〈L3 の比較対象と計画の節に触れない所〉と、build に入らない frontend の test の file〈`src/test/**`・`src/**/*.test.ts`・`src/**/*.test.tsx`〉。Rust の test は TD-706 で含めない）の中だけで衝突を手で解いた（または自動の merge と違う）lane が、L3 をやり直さずに、同期だけを見る closure の 2 本（座組表の Final Reviewer の Claude 側と Codex）の「手の解消が両側の変更を落としていない・L3 の項目に影響なし」の判定と owner の 1 行の承認で、manual を使い回せる（今の B と同じ `record --kind manual --outcome pass --evidence <旧 evidence> --reuse-from SOURCE --reuse-approval POINTER`）。
- 使い回せないとき、helper は今と同じく manual 以外の record を書かずに止め、手で解いた path と、外れた理由（許す集合の外の path・L3 の比較対象の変化）と次の一手を名指す。
- helper の `status`・`ready`・`merge` は、`source_head` から今の head まで同じ条件を確かめ直し、手で解いた path があるのに `reuse_approval` の無い manual と、`sync_from` の形の closure を拒む。

### 失敗定義

- code（classifier の `rust`・`frontend`・`env`・`unknown`。(b) の frontend の test の file を除く。`src-tauri/**` の Rust の test〈`#[cfg(test)]` の module を含む本体の file〉は code のまま）・生成物の手書き（生成物は今のとおり再生成）・gate の文書（`workflow=true`）・packet の L3 の比較対象（`l3_basis`）・計画の節（`plan_sections`）に触れる手の解消を受けてしまう。
- 手で解いた同期で、manual を人の closure と owner の承認なしに引き継ぐ（自動の A）、または review の closure を helper の記録（`--review-stage sync`・`docs`）で代える。偽の衝突の解消（片側を落とす）を機械が通す経路ができる。
- `scripts/plan-approve.sh` が手で解いた merge を「main の同期」として受けるようになる（計画の review の後に reviewer の読まない編集が merge に紛れる）。
- R4 の `r4` の承認を使い回す。

### 非目的

- lane の 1 親の commit の扱い（C）を広げること（`traceability` の docs・R4 を C に入れない）。
- 手の解消が両側を保つことを機械で証明すること（下の案 4。closure の判断に置く）。
- 生成物・`docs/decision-log.md` の union の規則を変えること（union の結果は今のとおり機械で受ける）。
- `docs/backlog.md` 等に union を広げること（下の案 3）。
- classifier・CI の job・RecordV1 の変更。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 調べ（起票の時点、2026-11-14）

測定は `git fetch origin` の後の `origin/main`（`a90b0d6`、PR #256 の merge の後。発注の時点の `580f7fe5` より 1 本進んだ）で、`git merge-tree --write-tree --name-only origin/main origin/agent/<branch>`。

| PR | branch | Risk（packet） | 衝突の path（生成物を含む） | 取り込む main 側の code（merge-base → main） |
|---|---|---|---|---|
| #236 | `agent/monthly-imported-days` | R3、`## L3` あり | `docs/Plans.md`・`docs/backlog.md`・`docs/daily-monthly-acceptance-runbook.md`・`docs/function-design/24-io-csv-import-repo.md`・`docs/function-design/57-ui-monthly-sales.md` | あり（#235 の `src-tauri/**`・`src/**` ほか。merge-base `19ec25e`） |
| #227 | `agent/return-register-processed` | R3、`## L3` あり | `docs/backlog.md`・`docs/function-design/90-traceability.md`（生成物。発注の時点の測定から増えた） | あり（merge-base `24e8909`） |
| #228 | `agent/xz-bkup-move` | R4、`## L3` あり | `docs/backlog.md`・24・37・45・55・`90-traceability.md`（生成物）・`src/features/daily-report-import/DailyReportImportPage.sd.test.tsx` | あり（merge-base `24e8909`） |
| #216 | `agent/stock-transfer` | R4、`## L3` あり | `src/test/page-root-pageshell-sweep.test.ts` | あり（merge-base `24e8909`、`package.json` も） |
| #194 | `agent/backup-offsite-runtime-b` | — | 無し（exit 0） | 本 lane の対象外 |

衝突の path の classifier の出力（`printf '%s\n' <path> | bash scripts/ci/classify-changes.sh --files-from-stdin` の `=true`）: `docs/Plans.md`・`docs/backlog.md`・`docs/daily-monthly-acceptance-runbook.md`・`docs/decision-log.md` は `docs` だけ。function-design の 24・37・57 は `rust_drift`・`docs`・`traceability`。2 つの test の file は `rust_drift`・`frontend`・`traceability`。どれも `workflow`・`rust`・`env`・`generated`・`unknown` は false。

#227 の `docs/backlog.md` の衝突（`git show <merge-tree の tree>:docs/backlog.md` の衝突の印）は、両側が同じ項目「返品・交換の `register_processed` の既定…」の行を書き換えたもので、末尾の追記どうしではない。`git merge-file --union` は旧い行と新しい行の両方を残し、項目が 2 つになる（下の案 3 を見送る根拠）。

`src/test/**` と `src/**/*.test.ts(x)` は app の build に入らない: `package.json` の `build` は `vite build`（entry から import を辿る）で、`grep -rlE` で test の file でも `src/test/` でもない `src/**` の file から `src/test/…` か `*.test` を import する所は 0 件。`tsconfig.json` の `include` は `src` と `src/test` を含むので、`tsc --noEmit`（typecheck）と vitest には入り、hosted の frontend の job が新しい head で確かめる。

Rust の test（2026-11-14、本 run で実測）: `rg -l "#\[cfg\(test\)\]" src-tauri/src --type rust --stats` は 102 file 中 69 file（127 箇所）で、多くは本体の file の中の module。ほかに分かれた dir が 3 つある: `src-tauri/tests`（`architecture_test.rs`・`design_compliance_test.rs` ほか）、`src-tauri/dev-tools/tests`、`src-tauri/src/biz/csv_import_service/tests`（`mod.rs:15` の `mod tests;` で本体の crate の module に入る）。TD-706 は Rust の test を全部入れない。分かれた dir の file を path で受けることはできるが、決定の範囲の外に置き、本 lane の集合に入れない（入れるなら別の決定。下の「計画の review に問う点」）。

今の規則で 4 本は全部、同期だけの条件の merge の要件（merge-evidence の「同期だけの条件」の (a)）を満たさない（生成物と union の外の衝突）。B も同期だけの条件を要るので使えず、L3 のやり直しになる。C は 1 親の lane の commit だけを許し、merge の中の違いには効かない。

## 案の比較

| 案 | 中身 | 採否と理由 |
|---|---|---|
| 1（採る） | 同期だけの条件の merge の判定（`sync_merge`）に「手で解いてよい path」の集合を足し、その集合の中の違い（衝突の解消と、自動の merge と違う編集）を拒まず「手で解いた path」として返す。手で解いた path が 1 つでもあれば、取り込んだ main 側が code に触れるかに関わらず A にせず B に回す（closure 2 本と owner の承認）。review の closure も監査の closure を要る | 今の B の道（`--reuse-approval`・closure 2 本・owner の 1 行）をそのまま使い、RecordV1 も記録の手順も変えない。機械が判定できるのは「どの path を手で解いたか」までで、両側を保ったかは人が見る、という線を 1 か所で引ける。4 本はどれも main 側の code を取り込むので、今でも B の closure と owner の承認が要る形で、手の解消をその closure の範囲に足すだけになる |
| 1-(a)（見送り、TD-706） | 案 1 の手で解いてよい path を docs だけにし、frontend の test の file を入れない | owner が (b) を選んだ（TD-706）。(a) では #228（`DailyReportImportPage.sd.test.tsx`）と #216（`src/test/page-root-pageshell-sweep.test.ts`）が L3 のやり直しのままで、test の file は app の build に入らず hosted の typecheck・test が新しい head で確かめるので、docs と分ける理由が弱い。test を消す・skip する・弱める解消は closure の判定の項目で見る |
| 2 | C（lane の docs だけの commit）を merge の中の docs の違いにも広げ、helper が機械で受ける | 片側を落とす解消を機械は見分けられず（失敗定義の 2 つ目）、人の closure が入らない。C は `traceability` の docs（function-design）を受けないので、#236・#228 の衝突に効かない |
| 3 | `docs/decision-log.md` の union の解き方を `docs/backlog.md`・`docs/Plans.md` 等にも広げる | #227 の測定で、衝突は同じ項目の書き換えどうしで、union は旧い行と新しい行を両方残す（誤った結果を機械が受ける）。末尾の追記どうしでない衝突には使えない |
| 4 | 手の解消の結果に両側の追加の行が全部残るかを機械で確かめ、満たせば A | 同じ行を両側が書き換えた正しい解消（第 3 の行）は両側の行を含まないので、正しい解消を拒み、行を残しただけの誤った解消を受ける。正規化した差分の同値を根拠にしない MG-D13 の方針にも反する |
| 5 | closure を 1 本（Claude 側だけ）にする | B と別の本数の経路ができ、helper の分岐と規則の文が増える。失敗定義の中心（片側を落とす）は reviewer の見落としが要で、独立の 2 本の B と同じ強さを保つ |
| 6 | lane を新しい main へ cherry-pick・rebase で切り直す | head の SHA と Plan Commit の祖先関係が変わり、旧形式の packet は新形式と判定される（DEV_WORKFLOW Wave Operation）。L3 の版も変わるので使い回しにならない |
| 7 | 今のまま L3 をやり直す | owner の決定 TD-704 で採らない |

## 境界ごとの受けると拒む

| 境界 | 新たに受ける | 今までどおり拒む |
|---|---|---|
| 同期だけの条件の merge（`sync_merge`） | 手で解いてよい path の集合の中の、自動の merge との違い（衝突の解消・解消でない編集）。helper は列挙して「手で解いた path」として返す | 集合の外の違い（code・`workflow=true` の docs・未知の path・Rust の test・pattern に当たらない `src/**`）。second parent が main の外・merge-base が一意でない・最後の merge が今の base でない、の条件はそのまま |
| 手で解いてよい path の集合（TD-706 の (b)） | 次の 2 種の和。(1) docs: classifier の `docs` が true で、`rust`・`frontend`・`env`・`generated`・`workflow`・`unknown` が全部 false の path（`traceability`・`rust_drift` は受ける。hosted の drift の検査が新しい head で走る）。`docs/decision-log.md` の union と違う結果もここに入る（union の結果は今のとおり手の解消に数えない）。(2) build に入らない frontend の test の file: helper の定数の pattern `src/test/**`・`src/**/*.test.ts`・`src/**/*.test.tsx` に当たる path（classifier から解析して得ない。`GENERATED` と同じ扱い。classifier はこれらを `frontend` にするので (1) の判定より先に当てる） | 生成物の手書き（今のとおり違ってよいが、再生成の結果を hosted の drift 検査が確かめる。手で解いた path に数えない）。(2) の pattern の近い外れ（`src/testing/**`・`src/lib/test.ts`・`*.test.ts.bak`・`*.test.js` 等）。Rust の test（`src-tauri/**` の全部。本体の file の中の `#[cfg(test)]` の module も、`src-tauri/tests/**` 等の分かれた dir も。TD-706） |
| L3 の比較対象（`l3_basis`） | — | 手で解いた path があれば、packet のある lane で `source_head` と今の head の L3 の比較対象が byte で同じことを求める（今の C の `l3_change` と同じ関数）。違う・比べられないなら使い回さない。active な packet の path と、`l3_basis` が link する Matrix の path を手で解いた path に含む merge も拒む |
| 計画の節（`plan_sections`） | — | 今のとおり、承認の最新の snapshot と byte で同じでなければ `plan_drift` が status・Ready・merge で止める（本 lane で新しい検査を足さない） |
| code | — | 手で解いた path に code を含む merge は同期だけの条件を満たさず、L3 のやり直しだけを案内する（今のとおり）。取り込んだ main 側の code は今の code に触れない条件と B のまま |
| 生成物 | — | 変えない（再生成。D-098） |
| decision-log | union と違う手の解消を、拒まず「手で解いた path」として B に回す | union の結果を機械で受ける今の規則は変えない |
| manual の自動の引継ぎ A | — | 手で解いた path があれば、main 側が code に触れなくても A にしない。`carry_manual` は B の完全な command（手で解いた path を名指す）と L3 のやり直しの command を示して何も書かずに止まる |
| review の closure | — | 手で解いた path があれば `--review-stage sync`・`--review-stage docs` を拒み、監査の closure を案内する（B の 2 本が兼ねる） |
| status・Ready・merge の確かめ直し | `reuse_approval` のある manual は、手で解いた path があっても通す | 手で解いた path があるのに `reuse_approval` の無い manual、`sync_from` の形の closure を拒む |
| `scripts/plan-approve.sh` | — | 手で解いた merge を今のとおり「not a sync of main」で拒む（`sync_merge` の既定は今の厳しい判定のまま、helper の使い回しの経路だけが新しい判定を頼む） |
| R4 | （Q2 の答えが「入れる」なら）B の manual の使い回し | `r4` の承認は今のとおり使い回さない（新しい head での明示の承認） |
| packet の無い R0〜R2 | （Q3 の答えが「入れる」なら）今の B と同じく、closure の 2 本の証跡を `reuse_approval` の pointer の先に置いて使い回す | `l3_basis` の比較は packet が無いので行わない（今の B と同じ） |

## Scope

- `scripts/pr-gate.py`（計画の固定の後の実装の run）:
  - `sync_merge(commit, first, second, base, resolve=False)`: `resolve=False`（既定、`plan-approve.sh` の呼出し）は今のまま。`resolve=True` のときは、自動の merge と違う path（今の `lane`）のうち手で解いてよい path の集合の中のものを拒まずに返し、集合の外が 1 つでもあれば今の文で拒む。`docs/decision-log.md` は union と一致すれば今のとおり除き、一致しなければ集合の判定に回す。集合の判定は、まず定数の pattern（frontend の test の file）、外れた path を current main 側の classifier（`self.contents('scripts/ci/classify-changes.sh', base)`）に path ごとに通す（今の `code_paths` と同じ形）。
  - `sync_merges`: 手で解いた path を `self.resolved`（`self.docs_commits` と同じ置き方）に集める。呼出し側は `reusable`・`carry_manual`・`sync_closure`・`nonci`・`record --reuse-from` の 5 か所（`rg -n "sync_merges|reusable\(|sync_merge\(" scripts` で列挙し直す）。
  - `carry_manual`: `self.resolved` があれば B の command と L3 のやり直しの command を示して止める（理由の文に手で解いた path を並べる）。L3 の比較は `self.docs_commits or self.resolved` のとき行う。
  - `record --reuse-from`: `--reuse-approval` があれば手で解いた path を受け、無ければ（明示の A）拒む。
  - `sync_closure`: `self.resolved` があれば拒み、監査の closure を案内する。
  - `nonci`: manual の `source_head` から確かめ直すとき、手で解いた path があって `reuse_approval` が無ければ拒む。`sync_from` の closure は手で解いた path があれば拒む。
  - pattern の定数（例 `TEST_FILES`）を `GENERATED` の近くに置き、comment に「build に入らない frontend の test の file（TD-706）。classifier から解析して得ない。Rust の test は入れない」と書く。`ponytail:` comment に限界（app の code が test の file を import すると「build に入らない」が崩れる）と見直す条件（その import が出たとき、または Rust の分かれた test の dir を入れる決定が出たとき）を書く。
- `scripts/tests/pr-gate.test.py`: 下の Test Plan の test を `SyncOnly`・`SyncRecords`・`PacketReuse` に足す。今の `test_sync_only_rejects_manual_conflict_resolution`・`test_sync_only_rejects_edit_inside_merge`・`test_sync_only_decision_log_union_conflict` の拒む向きは既定（`resolve=False`）の判定として残す（弱めない）。
- `scripts/tests/plan-approve.test.sh`: 変えない（PA5 の「edit in merge」が今のとおり拒まれることが、既定の判定を変えていない根拠）。
- `docs/agent-guidance/merge-evidence.md`: MG-D13 の行、「同期だけの条件」の (a) の違ってよい path の文、「manualの使い回し」の B（判定の項目に「手の解消が両側の変更を落としていない。test の file では test を消す・skip する・弱めることをしていない」を足す）・使い回せないときの次の一手、「reviewのclosure」、「Helperの境界」の `--reuse-from` の文。L3 の使い回し C の節は変えない。
- `docs/DEV_WORKFLOW.md`: L3 の表の B の行に「同期で生成物と decision-log の union の外の衝突を手で解いた（手で解いてよい path の中だけ）」を足し、「取り直す」の行と食い違わないようにする。Wave Operation の「衝突の解消が内容を変えたら通常の再検証と現在版の closure を行う」は今も正しいので残す。
- `docs/decision-log.md`: 末尾に D-154（TD-704・TD-706、案の比較〈(a) を見送った理由を含む〉、境界、Guarantee range: 手の解消の正しさは closure の判断で機械の保証でない）。D-144 に `Superseded in part by: D-154`（(1) の「生成物の外の衝突は満たさない」の部分）の行を足す。
- `docs/backlog.md`: 項目「衝突を解いた版の manual の再利用（観測）」を実装の merge で済みにする（本 commit は起票の 1 文だけ）。
- 本 packet（本 commit）。

Non-scope: `AGENTS.md`（Ready の承認を保つ判定は今のとおり helper の `--review-stage sync` か R0/R1 の明示の A で、手で解いた同期は承認を取り直す。B の承認と同じ 1 回に束ねる。Q5）、`scripts/ci/classify-changes.sh`、`.github/**`、RecordV1、`scripts/plan-approve.sh`。

## Acceptance Criteria

- AC1: `python3 scripts/tests/pr-gate.test.py` が全 PASS で、Test Plan の受ける組・拒む組の test が全部ある。
- AC2: `bash scripts/tests/run-workflow-tests.sh` が全 PASS（`plan-approve.test.sh` の PA5 を含む。既定の判定を変えていない）。
- AC3: Test Plan の mutation（M1〜M6）を実装に入れると、`python3 scripts/tests/pr-gate.test.py` で名指した test が FAIL（exit 非 0）になり、戻すと全 PASS に戻ることを実装の run の報告に書く（D-098 (9)）。
- AC4: `bash scripts/doc-consistency-check.sh` と `--target plan docs/plans/2026-11-14-sync-docs-conflict.md` が ERROR なし、`bash scripts/check-workflow-git.sh` が exit 0。
- AC5: merge-evidence・DEV_WORKFLOW・decision-log の文が helper の振舞いと同じ（手で解いてよい path の集合、A にしない、監査の closure、確かめ直し、plan-approve の既定）で、L3 の使い回し C の節は変わらない（`git diff` で C の節に差分が無い）。
- AC6: 合成の repo の test で、#236 の形（docs と function-design の衝突を手で解き、main 側が code を含む）と #216 の形（`src/test/*.test.ts` の衝突）と #228 の形（`src/**/*.sd.test.tsx` と function-design の衝突）が、B の command で manual を使い回せ、`--reuse-approval` の無い record・`--review-stage sync` は何も書かずに止まる。同じ合成の repo で `src-tauri/src/**/*.rs`（`#[cfg(test)]` の module の中だけの違い）と `src-tauri/tests/*.rs` の手の解消は同期だけの条件を満たさず、L3 のやり直しだけを案内する。

## Design Readiness

- 引用する設計正本（節まで）: `docs/agent-guidance/merge-evidence.md` の MG-D13・「base同期だけでheadが変わる場合」（同期だけの条件、codeに触れない条件、manualの使い回し A・B、reviewのclosure）・「lane の docs だけの commit（C）と GA の後の closure」・「Helperの境界」、`docs/DEV_WORKFLOW.md` の L3 の表と Workflow State（D-098 の安全の境界）、`docs/decision-log.md` D-098・D-144・D-145 (6)(7)・D-146、`AGENTS.md` Working Rules の「継ぎ足さない」「最小の実装（D-147）」と Decision and Approval Boundaries の Ready の承認の範囲。
- 必要な設計成果物: 正本の文の改訂は updated in this PR（計画の固定の後の実装の run で、helper と test と同じ PR に入れる）。plan-first の正本の先行の改訂は要らない（規則の文と helper を同時に変え、食い違う期間を作らない。D-145 の TD-502 と同じ理由）。
- plan にしかない durable な判断の昇格先: 案の比較・境界の表 → D-154 と merge-evidence の該当の文。調べの command と出力は本 packet に残す。
- 前提・制約と延期した design gap: 手の解消が両側を保つかは機械で判定しない（案 4）。closure の 2 本と owner の判断に置き、D-154 の Guarantee range に書く。classifier が将来 docs の分類を変えれば集合も変わる（current main 側の classifier を使う今の方針どおり）。
- 絶対保証の例外と抜け道: 「code に触れる手の解消を受けない」は classifier の分類と (b) の定数の pattern に依る。classifier が未知の path を full にするので、未登録の path は受けない側に倒れる。frontend の test の file の「build に入らない」は今の `vite build` と import の実測に依り、app の code が test の file を import するようになれば崩れる（D-154 の Revisit と helper の `ponytail:` comment に置く）。
- 判定: ready（Q1 は TD-706 で (b) に決まった。残る Q2〜Q5 は R4・R0〜R2・gate の文書・Ready の承認の分岐で、どの答えでも Scope の形〈helper の関数・test・正本の場所〉は変わらない）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| MG-D13 | merge-evidence 設計判断の表・「同期だけの条件」（TD-706 の (b) の集合） | `sync_merge(resolve=True)`・`sync_merges`・`TEST_FILES` | T1〜T5・T4b | 非対象（helper） |
| MG-D13 B | merge-evidence「manualの使い回し」(B 承認) | `carry_manual`・`record --reuse-from` | T6・T7・T10 | 非対象 |
| MG-D13 closure | merge-evidence「reviewのclosure」 | `sync_closure` | T8 | 非対象 |
| MG-D13 確かめ直し | merge-evidence「使い回しの元は…確かめ直し」の段落 | `nonci` | T9 | 非対象 |
| D-145 (6) C | merge-evidence「L3 の使い回し C」 | 変えない（`l3_change` を手で解いた path にも使う） | T11、`scripts/tests/pr-gate.test.py` の今の C1〜C5・RC1〜RC4 の test（PK3 は `scripts/tests` を探さないので名を token にしない） | 非対象 |
| D-145 (7) 計画の節 | merge-evidence「docs の closure」・DEV_WORKFLOW Workflow State | 変えない（`plan_drift`） | 今の test | 非対象 |
| D-144 | decision-log D-144 (1)〜(6) | Superseded in part の行 | — | 非対象 |
| D-146 | decision-log D-146（B の「影響なし」） | 変えない（B の判定の項目に「両側を保つ」を足す） | — | 非対象 |
| D-098 | DEV_WORKFLOW Workflow State の安全の境界（古い証拠の拒否） | 境界の表の「拒む」列を helper の test で固定 | T1〜T12 | 非対象 |
| PA5 | `scripts/plan-approve.sh` (a) の merge の判定 | 変えない（`sync_merge` の既定） | `plan-approve.test.sh` PA5 | 非対象 |
| DEV-L3 | DEV_WORKFLOW L3 の表 | B の行の文 | doc-consistency | 非対象 |
| D-154 | decision-log（新規） | 本 lane の決定 | — | 非対象 |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| MG-D13 | docs の衝突を手で解いた merge を拒む（使い回しの経路） | unit（合成 repo） | T1 `test_resolve_docs_conflict_returns_resolved` | `resolve=True` でも集合の中の違いを拒む |
| MG-D13 | function-design（`traceability`）の手の解消を拒む | unit | T2 `test_resolve_traceability_docs` | 集合を C の `NON_DOCS` で作る |
| MG-D13 | code・`workflow=true` の docs・未知の path の手の解消を受ける | unit | T3 `test_resolve_rejects_code_gate_docs_unknown` | 集合の判定を外す（M1） |
| MG-D13 (b) TD-706 | frontend の test の file（`src/test/**`・`src/**/*.test.ts(x)`）の手の解消を拒む／pattern の近い外れ（`src/testing/x.ts`・`src/lib/test.ts`・`src/a.test.ts.bak`・`src/a.test.js`）を受ける | unit | T4 `test_resolve_frontend_test_files_exact` | pattern を `src/test` の前方一致や `test` の部分一致にする、または pattern を外す |
| MG-D13 TD-706 Rust | Rust の test（`src-tauri/src/**/*.rs` の `#[cfg(test)]` の中だけの違い、`src-tauri/tests/*.rs`）の手の解消を受ける | unit | T4b `test_resolve_rejects_rust_tests` | pattern に `src-tauri/**/tests/**` や `*_test.rs` を足す、または集合の判定を path の末尾の `test` で行う |
| PA5 | 既定の判定が手の解消を受ける | unit・CLI | T5 `test_sync_merge_default_stays_strict` と `plan-approve.test.sh` PA5 | `resolve` の既定を True にする（M2） |
| MG-D13 A | 手で解いた同期で manual を自動で引き継ぐ | unit | T6 `test_resolved_sync_never_auto_carries` | `carry_manual` が `self.resolved` を見ない（M3） |
| MG-D13 B | B の command で使い回せない／`--reuse-approval` なしの明示の A で受ける | unit | T7 `test_resolved_sync_reuses_with_approval_only` | `record --reuse-from` が承認の有無を見ない |
| closure | `--review-stage sync`・`docs` が手の解消の同期で機械の closure を書く | unit | T8 `test_resolved_sync_rejects_machine_closure` | `sync_closure` が `self.resolved` を見ない（M4） |
| 確かめ直し | status が承認の無い manual・`sync_from` の closure を通す | unit | T9 `test_nonci_rechecks_resolved_sync` | `nonci` の確かめ直しを外す（M5） |
| R4 | `r4` を使い回す／（Q2 が入れるなら）R4 の B を拒む | unit | T10 `test_resolved_sync_r4_keeps_fresh_r4` | `r4` を新しい head の record で pass のまま残す |
| L3 の比較対象 | 手の解消が packet の L3 節・link する Matrix を変えたのに使い回す | unit | T11 `test_resolved_sync_l3_basis_change_stops` | 手で解いた path で `l3_change` を呼ばない |
| 止まる文 | 止まるときに手で解いた path と次の一手を名指さない | unit | T12 `test_resolved_sync_names_paths_and_next_step` | 理由の文から path の列挙を落とす |

mutation（実装の run で注入し、red を確かめて戻す。AC3）: M1 集合の判定を外す → T3 が red。M2 `sync_merge` の `resolve` の既定を True → T5 と PA5 が red。M3 `carry_manual` で `self.resolved` を見ない → T6 が red。M4 `sync_closure` で `self.resolved` を見ない → T8 が red。M5 `nonci` の `reuse_approval` の確かめを外す → T9 が red。M6 frontend の test の file の pattern を `'test' in path` の部分一致にする → T4 と T4b が red。

L3: 置かない。operator の画面・app の build を変えず、helper と規則の文だけを変える。振舞いは合成の git の履歴の上の helper の test（上の表）で観測でき、Windows/Tauri native でしか観測できない項目が無い（DEV_WORKFLOW L3 Eligibility の (1) を満たさない）。

## 並走する lane への効き方

Coordinator の決定: 本 lane の merge まで #236・#227・#228・#216 の同期と merge を止める（helper の物差しを変えるため）。#194・#250・#251 は今の helper で進む。

TD-706 の (b) で書く。見送った (a) なら #228・#216 は test の file が集合の外で L3 のやり直しだった（#236・#227 は (a) でも同じ）。

| PR | 手で解く path（生成物を除く） | (b) での効き方 | 補足 |
|---|---|---|---|
| #236（R3） | docs 5 path（function-design の 24・57 を含む） | B で使い回せる | main 側が code を含むので、今の規則でも衝突が無ければ B だった。手で解いた 5 path を closure の範囲に足す。`docs/daily-monthly-acceptance-runbook.md` が L3 の手順に関わるなら、closure の「L3 の項目に影響なし」の判定で見る。packet の L3 の比較対象が手の解消で変わらないことも helper が確かめる |
| #227（R3） | `docs/backlog.md` | B で使い回せる（`90-traceability.md` は再生成） | — |
| #228（R4） | `docs/backlog.md`・24・37・45・55・`src/features/daily-report-import/DailyReportImportPage.sd.test.tsx` | Q2 で R4 を開くなら B で使い回せる（`*.test.tsx` が集合の中）。`r4` は新しい head で承認を取り直す | Q2 で R4 を開かないなら L3 のやり直し。test の file の解消は closure が「test を消す・skip する・弱めていない」も見る |
| #216（R4） | `src/test/page-root-pageshell-sweep.test.ts` | #228 と同じ（`src/test/**` が集合の中） | 同上 |

本 lane の merge の後: 止めた 4 本が同期すると、取り込む main 側に `scripts/pr-gate.py`（classifier は full）が入るので、どの lane も code に触れる同期（B）になる。4 本はもともと #235 の code を取り込むので B で、増える手は無い。#194・#250・#251 が本 lane の merge の後に同期すると、同じ理由で A でなく B になる（L3 を済ませた lane は closure 2 本と owner の承認が要る）。L3 を済ませた lane は本 lane の merge の前に merge するほうが手が少ない。本 lane の merge の後、同期の前の lane の helper は D-099 で base の版と違うと止まり、base の版の command を示す（今の動き）。

## Open Questions

### 決定の記録

- Q1 → TD-706（repo 外の回答台帳、2026-11-14）: owner は「両方推奨」と答え、Coordinator の推奨の (b) に決まった。手で解いてよい path（L3 を使い回せる衝突・自動の merge との違いの範囲）は、docs（L3 の比較対象と計画の節に触れない所）と、build に入らない frontend の test の file（`src/**/*.test.ts(x)`・`src/test/**`）。Rust の test は本体の file の中にあって分けられないので含めない（本 run の実測では分かれた dir もあるが、決定の範囲の外に置く。上の「調べ」）。(a) は「案の比較」の 1-(a) に見送った案として残す。

### 未決（owner に諮る）

- Q2: R4 の lane（#228・#216）に B のこの道を開くか。推奨は開く（candidate）: 今の B は R4 にも効き（code に触れる同期）、`r4` の承認は今のとおり新しい head で取り直す。開かないなら (b) に決まっても #228・#216 は L3 のやり直し。
- Q3: packet の無い R0〜R2 の lane にも開くか。推奨は開く（candidate）: 今の B と同じく closure の 2 本の証跡を `reuse_approval` の pointer の先に置き、L3 の基準は PR body にあって tree の手の解消で変わらない。
- Q4: gate の文書（classifier の `workflow=true`）の手の解消を集合の外に置くか。推奨は外に置く（candidate。規則の文の手の解消は lane の規則の review の対象で、L3 と無関係でも機械の分岐を増やさないため）。必要が出たら D-154 の Revisit で見直す。
- Q5: 手で解いた同期は helper が「同期だけ」と判定しないので、AGENTS.md の Ready の承認は取り直しになる。B の owner の 1 行の承認と同じ 1 回に束ね、AGENTS.md は変えない、でよいか。推奨はこのまま（candidate）。

## 計画の review に問う点

- 境界の表の「拒む」列が、D-098 の古い証拠の拒否を本 lane が緩める範囲（手で解いた path を B に回す所）の外で弱めていないか。特に `plan-approve.sh` の既定の判定・`sync_from` の確かめ直し・`r4`。
- 手で解いてよい path の集合を classifier の flag で決めるとき、`traceability`・`rust_drift` を受けてよいか（function-design は設計正本で、手の解消が契約の意味を変え得る。B の closure で見る前提で足りるか）。
- 手で解いた path に active な packet と link する Matrix を含む merge を拒む規則と、`l3_change` の比較が重複していないか（どちらか一方で足りるなら消す）。
- この変更で不要になる記述・条件は無いか（D-144 の「生成物の外の衝突は満たさない」の文、backlog の観測の項目、DEV_WORKFLOW の L3 の表の「取り直す」の行の読み）。
- Test Plan の T1〜T12・T4b と M1〜M6 で、受ける組と拒む組が足りるか（自動の merge と違う「解消でない編集」を集合の中で受けることの是非を含む）。
- TD-706 の frontend の test の file の pattern（`src/test/**`・`src/**/*.test.ts`・`src/**/*.test.tsx`）が、今の tree の test の file（`*.sd.test.tsx` を含む）を漏れなく当て、build に入る file を当てないか。`src/test/**` に vitest の setup 等の test でない file があっても build に入らないか。
- TD-706 の Rust の除外の理由（「本体の file の中にあって分けられない」）は、本 run の実測で `src-tauri/tests/**` 等の分かれた dir があり、全部には当たらない。決定の範囲どおり Rust を全部外す本 packet の形でよいか（入れる向きの見直しは owner の別の決定。判断の材料として Coordinator に返す）。
