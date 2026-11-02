# Test Design Matrix: 開発の手続きの作り直し（D-145）

Plan Packet: [../2026-10-28-process-rebuild.md](../2026-10-28-process-rebuild.md)

helper・検査・CI の振舞いを変える行（H1〜H9・H11〜H14）を、合成の git repo・fake gh・合成の packet で確かめる。既存の拒否の case（古い head・broad の不足・base の版の helper・R4 の承認を使い回さない・UI merge の道）は消さず、Phase・移送の前提で書かれた case は新しい契約に合わせて書き換える（書き換えた case と理由は PR body に挙げる。【安全: テストを弱めない】）。

## Risk

Risk: R3

## Contracts Under Test

- D-145 (2) Plan Commit の固定を script で書く（H1）
- D-145 (3) Phase を外し、Plan Commit の有無で計画の前後を決め、計画の承認の前の実装を機械が止める（H2）
- D-145 (4) 新形式の packet で Final Review Minimum と Human Gate を Risk・classifier・`## L3` 節から導き、旧形式は書いた値を使い、書いた r4・manual を落とさない（H3）
- D-145 (5) packet を archive へ移さない。検査の対象を active な packet（起点の tree に無い dated packet）に限り、helper・PK1・PK4・PK5・H2 で定義を 1 つにする（H2・H4）
- D-145 (6) L3 の使い回し C と docs だけの lane の commit の機械の closure（H5・H6）
- D-145 (7) GA の後、Scope・AC が同じなら closure（H7）
- D-145 (8) R2 は packet なしの道（H8）
- D-145 (9) 同じ head の CI を 1 回にする（H9）
- D-145 (10) reviewer の報告から reviewed head を読む（H11）、生きた docs に `path:行` を新しく足さない（H13）、packet の最小形（H14）
- 隣接（変えない）: helper は PR の base の版だけが判定する（D-099）、古い head・base の record を拒む、broad の本数の不足を拒む、R4 の承認を使い回さない、PK5 の ancestry と不変、不可逆 finding の 4 項目（文だけ）

## Failure Modes

- 計画の承認の前に code を書いた branch を、Phase の行が無くなったことで機械が通す
- 旧い packet（Phase・担当 4 行・Owner Effort Budget を持つ）を新しい helper・checker が拒み、走っている lane が止まる
- 導いた Minimum が旧い packet の明示値より小さくなり、R3 の Final が 1 本で通る
- `## L3` 節の有無の判定が外れ、manual の gate が黙って消える
- docs だけに見える lane の commit（rename・未知の path・生成物）で L3 か review を使い回す
- GA で Scope・AC が変わったのに fresh broad を求めない
- R2 の packet なしの道で、workflow・rust の変更が R2 として通る
- Draft の run の `Merge gate` が別の head の証拠として受理される
- 報告の reviewed head と capture の head が違うのに record が書かれる
- PK1・PK4 の対象を差分で絞るとき、差分が取れない場合に検査が 0 件になる（fail-open）

## Test Matrix

既存 test の引用は `rg -n '<test 名>' scripts/tests/` で在ることを確かめてから書く（実装の段で Writer が確かめ、無ければ新しく作る）。test 名は実装で決め、下の ID を test の docstring か comment に付ける。

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| PA1 `plan-approve.sh` が `Plan Commit: pending` を報告の SHA に置き換え、packet だけの commit を 1 本作る | 手写しの誤記 | CLI（合成 repo） | `scripts/tests/plan-approve.test.sh` PA1 | 置き換えが別の行に当たる、commit が packet 以外を含む |
| PA2 SHA が branch の祖先でない・40 桁の小文字 hex でない・既に SHA が入っている → exit 非 0、何も書かない | 承認していない版を固定、Plan Commit の書き換え（PK5 違反） | CLI | 同 PA2 | 祖先の検査か不変の検査が抜ける |
| PA3 `--amend SHA` は `Amendments` に足す（`none` → SHA、既存の列の後ろ）。重複・Plan Commit の子孫でない SHA は拒む | Amendments の順の崩れ | CLI | 同 PA3 | 列の前に足す、重複を通す |
| PA4 (a) lane の commit（`git log --first-parent --no-merges --name-only <SHA>..HEAD`）に packet 以外の path があれば exit 非 0、何も書かない。(b) `<SHA>` の版と HEAD の版の packet の本文が、Workflow State の `Plan Commit`・`Amendments` の行を除いて byte で違えば exit 非 0（負例: `<SHA>` の後に同じ packet の `## Scope` に 1 項足して commit → 拒む）。(c) 対象の packet に未 stage の変更（作業 tree だけで `## Scope` に 1 項足した）か stage 済みの変更があれば、何も書かずに exit 非 0（負例を 2 つ）。差分が空か、packet の承認の記録の行だけで、作業 tree と index が HEAD と同じなら通す | reviewer が読んでいない実装・計画の変更を積んでから承認を書く（Codex r2 #1・Opus r2 F4 の反例） | CLI | 同 PA4 | 差分の検査が path だけで、packet の本文を比べない |
| PA5 `<SHA>` の後に `origin/main` の単段の merge（#217 の同期だけの条件を満たす）を挟んでも通る。間の merge が条件を満たさない（merge の中の編集・main の外の branch）なら拒む | 計画の承認の前の同期で承認の道が閉じる／同期に紛れた変更を通す | CLI（合成 repo） | 同 PA5 | 二点の diff で main 側の path を拾う、merge を無条件に通す |
| PH1 Phase 行の無い新しい packet: `Plan Commit: pending` → helper `status` は `Plan Gate incomplete`、`capture` は拒む | 承認の前に capture・record できる | unit（`parse_packet`・`snapshot`） | `scripts/tests/pr-gate.test.py` PH1 | `phase == implementing` の代わりの判定が pending を通す |
| PH2 Phase 行の無い packet で Plan Commit が SHA → `status` が blocker の評価に進む | 承認済みの lane が止まる | unit | 同 PH2 | SHA の packet を計画中と扱う |
| PH3 旧形式の packet（Workflow State に Phase の行がある。Coordinator・Writer・Plan Reviewer・Final Reviewer・Branch・Owner Effort Budget を持つ）を受け、Phase の値を評価しない（8 値の外でも拒まない）。Plan Commit・Amendments・Risk の形の検査は今のまま。Phase の行の有無で形式を見分ける | 走っている lane（#216 ほか）が止まる | unit | 同 PH3 | 旧い行を必須か拒否にする |
| PH4 active な packet（起点の tree に無い dated packet）のどれかが `Plan Commit: pending` で、起点からの差分に classifier の `rust`・`frontend`・`generated`・`env`・`unknown` が出るか、`scripts/**`・`.github/**` の path があれば ERROR | 計画の承認の前の実装を通す（K11） | CLI（`check-workflow-git.sh`、合成 repo） | `scripts/tests/workflow-git-checks.test.sh` PH4 | 先走りの検査が無い、code の種類の一部を見落とす |
| PH5 同じ条件で差分が docs・packet だけなら通す（plan-first の commit は止めない） | 起票の commit が止まる | CLI | 同 PH5 | docs を code と数える |
| PH7 形式は、起点..HEAD の範囲で packet を最初に足した commit の親の tree の `scripts/plan-approve.sh` の有無で決まる。負例: 新形式なのに `Phase` の行がある packet（file 名の日付を古くしたものを含む）、旧形式なのに `Phase` の行が無い packet、新形式で Risk R2 の packet、昔 `docs/plans/` にあった path（archive へ移した packet と同じ path）を新しい lane で足し直して `Phase` の行を持たせたもの（昔の add を拾わず新形式と判定して拒む）、add の commit に親が無い（root・`--depth 1` の clone の境）か shallow の repository（ERROR）。正例: `plan-approve.sh` の無い親から起点の後に足した `Phase` のある packet（merge の前に足した lane）は旧形式として受ける | 古い日付・`Phase` の行・path の足し直し・shallow で旧形式の `Final Review Minimum: 1` を使い TD-484 を逃れる／merge の前に起票した lane が merge の後に止まる | CLI・unit（合成 repo） | `doc-consistency-plan-packet.test.sh`・`pr-gate.test.py` PH7 | 範囲を全履歴にする、親の無い commit を旧形式に落とす、日付や行だけで決める |
| PH8 active な packet の `Plan Commit` が SHA（承認済み）なら、code の差分があっても PH4 の ERROR を出さない | 承認の後の実装が止まる（PH4 の判定の反転を検出する組） | CLI | `workflow-git-checks.test.sh` PH8 | pending の判定を外す・反転する |
| PH9 起点: `WORKFLOW_BASE_SHA` が空か HEAD と同じ（`workflow_dispatch` の run）なら `origin/main` と HEAD の merge-base を使い、active な packet を正しく数える。それも HEAD と同じ（main の checkout・最初の commit の前の新しい branch）なら active 0 件として通す（今の `check-workflow-git.sh` と同じ）。起点を解決できないときは ERROR（merge-base が一意でないときは PH6） | dispatch の run で起点 = HEAD になり active が 0 件で検査が素通りする／main の checkout で full の検査が ERROR になる | CLI | `workflow-git-checks.test.sh` PH9 | 起点の落とし方が無い、merge-base = HEAD を ERROR にする |
| PH10 rename と切り直し: 旧形式の lane の packet を本 lane の merge の後の branch で rename する、または新しい main へ cherry-pick で切り直すと、rename の後の path の最初の add で新形式と判定され、`Phase` の行で拒まれる（`--follow` で辿らない）。拒む理由に「旧形式の packet は rename・切り直しをしない」を出す | 規則を知らずに rename した lane が理由の分からない ERROR で止まる | CLI（合成 repo） | `doc-consistency-plan-packet.test.sh` PH10 | 理由を出さない、rename を辿って黙って旧形式にする |
| PH11（Final Review C1）承認の後に計画の節（Scope・AC・Contract Ledger〈旧形式は Spec Contract・Trace Matrix〉、R4 は Data Safety と Recovery・Rollback の節）が承認の最新の snapshot（Plan Commit、Amendments があれば最後）と違えば、helper（status・capture・ready・record・merge の共通の `requirements`）と `check-workflow-git.sh` が拒み、理由と `plan-approve.sh <packet> --amend` を名指す。旧形式で Phase が kickoff〜plan-gate の packet は SHA があっても承認の前 | 承認の後に Scope を広げ、Amendments を登録せず追補の計画の review なしに Ready・merge へ進む | CLI（fake gh・合成 repo） | `pr-gate.test.py` `test_plan_change_after_approval_rejected`・`test_old_pre_gate_phase_is_not_approved`、`workflow-git-checks.test.sh` の C1 | 計画の節の照合が無い、照合が helper か checker の片方だけ、旧形式の pre-gate の Phase を承認済みと扱う |
| PH6 起点が解決できない・merge-base が一意でない → active な packet を決められないとして ERROR で止め、理由を出す（全 packet の検査に落とさない） | fail-open／merge 済みの packet で必ず落ちる | CLI | 同 PH6 | 起点が取れないときに 0 件で通す、全 packet を見て merge 済みの packet の PK5 で落ちる |
| MN1 新形式の packet（Phase の行が無い）: R3 = 2、R4 = 2 と r4、classifier `workflow=true` = 2（packet に Minimum を書かない） | 新形式の R3 が 1 本で通る（TD-484） | unit（`requirements`） | `pr-gate.test.py` MN1 | 導出の表を誤る |
| MN2 旧形式の packet は書いた `Final Review Minimum` をそのまま使う（R3 で 1 なら 1、同期の後も 2 に上げない。R4・`workflow=true` で 2 を求める今の検査は残す）。新形式で `Final Review Minimum` の行があれば導いた値との大きい方（新形式の R3 に 1 と書いても 2） | 走っている lane の本数が同期の後に上がる／新形式で書いた 1 で下げる | unit | 同 MN2 | 旧形式に導出を当てる、新形式で min を取る |
| MN3 新形式は `## L3` 節があれば manual、Risk R4 なら r4。どちらの形式でも Workflow State の `Human Gate` に `manual`・`r4` があれば足す（和集合） | manual・r4 が黙って消える | unit | 同 MN3 | 節の検出か書いた行の読取りが抜ける |
| MN4 承認の後の条件の照合（`approved packet condition changed`）: 新形式は Risk・`## L3` 節の有無・（あれば）`Final Review Minimum`・`Human Gate` の行で、旧形式は今の 3 field で照合する。承認の後に `Human Gate` から r4 を外すと止まる | 承認の後に L3 や r4 を外して gate を消す | unit | 同 MN4 | 照合の対象から L3・書いた行を落とす |
| MN5 Risk R3 で `Human Gate: ready,merge,r4`（旧形式と、新形式で行を書いたもの）→ r4 を要り、record の r4 が pass でなければ `r4 incomplete` | 明示の r4 承認が導出で消える（Codex #1 の反例） | unit | 同 MN5 | r4 を Risk だけから導く |
| AR1 R0/R1 の道で packet が `docs/plans/` から `docs/archive/plans/` へ移っても `Phase: archive` を求めない（#215 の型を通す）。packet の削除だけ・archive の外への移動は今のまま拒む | 移送の PR が止まる、active な packet を黙って消す | unit | 同 AR1 | 削除の拒否まで消す |
| AR2 PK1・PK4 の対象は active な packet（起点の tree に無い dated packet。`check-workflow-git.sh` と同じ起点の取り方）。起点が取れなければ ERROR | merge 済みの packet の旧い形が他の PR を止める／起点が取れず検査 0 件 | CLI（`doc-consistency-check.sh`） | `scripts/tests/doc-consistency-plan-packet.test.sh` AR2 | 定義が PK5・helper と違う、fail-open |
| AR3 PK4 の `docs/Plans.md` の `## 次の行動` の pointer の検査を外す。`--target plan PATH` の明示の検査は今のまま | Plans の節を消すと全 PR が止まる | CLI | 同 AR3 | pointer の検査が残る |
| AR4 helper: merge 済みの packet（base にある dated packet）の typo を直すだけの PR は `--risk R0` の道で通る。同じ PR が active な packet を足していれば `--packet` を要る（今のまま） | merge 済みの packet を直す道が無い（Opus F1 の反例） | unit（`requirements`） | `pr-gate.test.py` AR4 | active の判定が「差分が触る packet」のまま |
| AR5 `check-workflow-git.sh`: merge 済みの packet を 1 行直す差分で PK5 と H2 が対象にしない（squash の Plan Commit が祖先でなくても通る）。merge 済みの packet の Workflow State の行を変える差分は ERROR。merge 済みの packet の `docs/archive/plans/` への移動（`docs/plans/` 側の削除と archive 側の追加、#215 の型）は通る。active な packet は今と同じに PK5 を見る | merge 済みの packet の編集で PK5 が落ちる／merge の時点の記録の書換え／#215 型の移動が行の変更と判定されて止まる | CLI（合成 repo で squash を再現） | `workflow-git-checks.test.sh` AR5 | 対象の定義が古い、記録の保護が無い、移動を行の変更と数える |
| AR6 link の検査: merge 済みの packet の link 切れは WARN、active な packet と生きた docs の link 切れは今のまま ERROR | 文書の改名のたびに merge 済みの packet の修正を強いる／生きた link 切れを見落とす | CLI | `doc-consistency-plan-packet.test.sh` AR6 | WARN の範囲が広すぎる・狭すぎる |
| C1 L3 の後の lane の commit が classifier で docs だけ（`workflow=false`）で `## L3` 節が byte で同じ → manual を使い回す（#217 の A・B と同じ record の道） | 締めの docs の commit で L3 が無効（K13） | unit（合成 repo） | `pr-gate.test.py` C1 | C の分岐が無い |
| C2 同じで `## L3` 節が変わる → 使い回さず、理由に「L3 節が変わった」を出す | 別の基準の PASS を使い回す（反例 9） | unit | 同 C2 | 節の比較が抜ける |
| C3 lane の commit に runtime・生成物・`unknown`・`workflow=true` の path、または通常の path から docs への rename → 使い回さない（`--no-renames`） | runtime の変更の後の L3 を使い回す | unit | 同 C3 | classifier の flag の一部を見ない、rename を docs と数える |
| C4 R4 は C でも使い回さない | R4 の承認の使い回し | unit | 同 C4 | R4 を引き継ぐ |
| C5 旧形式の manual の packet で `## L3` が無いとき: `## Test Plan` 節の全体と、packet が link する Test Design Matrix の file の全体を比べる。(1) L3 の合格基準だけを `## Test Plan` で変えた docs の commit では使い回さない（#210 の型）。(2) packet は変えず Matrix の `Manual` 節の基準だけを変えた docs の commit でも使い回さない（#211 の型。例「9 pt 以上」→「10 pt 以上」）。(3) どちらも同じなら使い回す。(4) `## L3` も `## Test Plan` も無い、Matrix の link が 0 本か 2 本以上、link 先の file が無いときは使い回さない | 旧形式で変わった基準に対する PASS を流用する（Codex r1 #2・r2 #2 の反例） | unit（合成 repo） | 同 C5 | 無い節どうしを同じと扱う、Matrix を比べない |
| RC1 `--review-stage docs`: 版の間の lane の commit が C1 と同じ条件で、`--evidence` に P3 だけと裁定した pointer があれば review の closure を機械で書く | P3 の docs の是正で review を取り直す（TD-476 (2)） | unit | 同 RC1 | 分岐が無い |
| RC1b 比較元・比較先とも manual なし（`## L3` の無い新形式の R3。FMT1 の「R3・manual なし」の正例）で、計画の節が同じ docs の P3 の typo の commit → closure を書く。版の間で manual の要否が変わる（`## L3` を足した・消した）→ 拒む | manual なしの R3 で機械の closure の道が無い（Codex r3 #2）／manual の要否の変更を見落とす | unit | `pr-gate.test.py` RC1b | L3 の比較を無条件に求める、要否の変更を見ない |
| RC2 同じ道で `workflow=true` の docs（`docs/agent-guidance/**`・`docs/DEV_WORKFLOW.md` ほか gate の文書）・計画の節（packet の「計画の節」の定義: `## Scope`・`## Acceptance Criteria`・`## Contract Ledger`。旧形式で Ledger が無ければ `## Spec Contract`・`## Trace Matrix`）の変更・`--evidence` の欠け → 拒む | gate の文や計画を人の review なしに変える | unit | 同 RC2 | workflow の docs を通す、計画の節の比較が抜ける |
| RC3 RC1 の道で、旧形式の packet の `## Test Plan`（`## L3` が無いとき）か、link する Matrix の file が変わる → 拒む | 旧形式の L3 の基準を人の review なしに変える | unit | 同 RC3 | L3 の比較対象を RC に当てない |
| RC4 RC1 の道で Risk が R4 の packet → 拒む（`## Recovery` だけを変えた P3 の docs の commit でも） | R4 の復旧の文を人の review なしに変える（Opus r2 F6） | unit | 同 RC4 | H6 が R4 を受ける |
| GA1 GA の後、broad の版と今の版で計画の節（`## Scope`・`## Acceptance Criteria`・`## Contract Ledger`。旧形式で Ledger が無ければ `## Spec Contract`・`## Trace Matrix`）が byte で同じ → closure を受ける | 同じ範囲で fresh broad を取り直す | unit（`validate_review`） | 同 GA1 | 比較が無い |
| GA2 Scope か AC が変わる、節が読めない → `FRESH_BROAD`（今のまま） | 範囲が変わったのに closure で通す | unit | 同 GA2 | 節が読めないときに closure を受ける（fail-open） |
| GA3 `## Contract Ledger` だけが変わる（Scope・AC は同じ）→ `FRESH_BROAD` | 足した契約が broad の Contract Audit を通らずに入る（Opus r1 F4 の反例） | unit | 同 GA3 | H7 が Ledger を比べない |
| GA4 R4 の packet で `## Recovery` か `## Data Safety` だけが変わる → `FRESH_BROAD` | R4 の復旧の変更が closure だけで入る（Opus r2 F6） | unit | 同 GA4 | R4 の計画の節に Recovery・Data Safety が無い |
| R2N1 `--risk R2`（packet なし）は Minimum 1、`plan_commit` null で、`--manual` を要る | R2 の道が無い | unit | 同 R2N1 | `--risk` が R0/R1 だけ |
| R2N2 packet なしの R2 で classifier が `workflow=true` か `unknown=true` → 拒み、理由に「R3 の packet で起票する」を出す。`rust=true`・`frontend=true` だけなら受ける。diff が active な packet を足していれば拒む（`--packet` の道へ） | gate・未知の変更が R2 で通る／R2 の行き先が 2 つある | unit | 同 R2N2 | flag の検査が抜ける、rust を拒む |
| R2N3 R2 の画面の修正で `--manual required` なら、目視の manual の record が pass になるまで status が `manual incomplete` | 画面の目視を飛ばして merge | unit | 同 R2N3 | R2 で manual を not-required に固定する |
| R2N4 packet なしの道の全部（`--risk R0`・`R1`・`R2`）で、差分に migration（`src-tauri/src/db/migration.rs`・`src-tauri/src/db/migration_tx.rs`・`src-tauri/src/db/schema_v*.rs`）か `package.json`・`package-lock.json`・`.npmrc`、classifier の `unknown=true`、または workflow の実行 code（`workflow=true` かつ `rust=true`）があれば拒み、理由にその path と「R3 の packet で起票する」を出す。`--risk R2` では `workflow=true`（docs だけでも）も拒む。正例: `workflow=true` の docs だけの typo（`AGENTS.md`・`docs/templates/plan-packet.md`・PR template）は `--risk R0`・`R1` で受ける | migration・依存・未知の変更が計画の review なしに入る／R0・R1 の軽い道へ逃げる／gate の文書の typo まで R3 に回して重くする（Opus closure N1） | unit | 同 R2N4 | path の検査が R2 だけ、R0/R1 で workflow の docs を拒む |
| R2N5 packet なしの道で PR body の Risk の行（`Risk: R<0|1|2>（目視: 要|不要）`）の Risk か目視の要否が `--risk`・`--manual` と食い違う、行が無い・2 行ある・書式が違う → 拒む。一致すれば受ける | body が「目視: 要」なのに `--manual not-required` で目視なしに通る（Opus r3 F5） | unit | 同 R2N5 | body を読まない |
| CI1 Draft の run の aggregate の名が `Merge gate`（`.github/workflows/ci.yml` の `merge_gate` の name 式）、`ready_for_review` で run を作らない | 同じ head の CI が 2 回（TD-481） | CLI（`scripts/tests/ci-workflow.test.sh`） | `ci-workflow.test.sh` CI1 | name 式か trigger が残る |
| CI2 helper `ci()` は同じ head の最新の成功の run（`pull_request` の Draft を含む）で `Merge gate` が 1 つなら受ける。別の head・失敗・`Merge gate` の欠け・別の app は今のまま拒む | 古い head の CI を受ける（K5） | unit | `pr-gate.test.py` CI2 | head の照合か app の照合が抜ける |
| CI4 `.github/workflows/ci.yml` の docs job の `doc-consistency-check.sh` の step が `WORKFLOW_BASE_SHA` を env に持つ | hosted で起点が無く H13 が WARN に落ち、H2・H4 が止まる | CLI | `ci-workflow.test.sh` CI4 | env の行が無い |
| CI3 Ready の後の push を pre-push が拒む挙動は今のまま | Ready の後の未検証の head | CLI | `scripts/tests/pre-push.test.sh` の既存の Ready push の case | 既存の case を消す |
| RP1 `record --report FILE` は FILE の `REVIEWED_HEAD=<40hex>` の 1 行を `--reviewed-head` に使い、capture の head と違えば何も書かない | 報告と違う版の record | unit | `pr-gate.test.py` RP1 | 照合が無い |
| RP2 行が無い・2 行ある・形が違う・`--reviewed-head` と両方 → 入力の誤り（exit 2） | 曖昧な報告を受ける | unit | 同 RP2 | 先頭の 1 行だけ読む |
| FL1 起点からの差分で生きた docs（`docs/**`、`AGENTS.md`・`CLAUDE.md`。下の FL3 の対象外を除く）に `<path>.<md|rs|ts|tsx|py|sh>:<数>` を足すと ERROR。既にある行は ERROR にしない | 腐る引用を新しく作る（D-125） | CLI | `doc-consistency-plan-packet.test.sh` か新しい test の FL1 | 既存の行まで ERROR、足した行を見落とす |
| FL2 差分が取れない → 全 live docs を対象にせず WARN と理由（既存 231 か所を一斉に ERROR にしない）。pre-push と hosted は起点を渡す | 起点の無い local の実行で全件 ERROR | CLI | 同 FL2 | fail の向きを誤る |
| FL3 D-125 (2)・(3) の適用除外は ERROR にしない: `docs/backlog.md` の行で同じ行に commit（7〜40 桁の hex）を添えた `path:行`、`docs/research/**`・`docs/archive/**`・`docs/plans/**`・`docs/decision-log.md` への追加。commit の無い backlog の行は ERROR | 規則が許す時点の記録を拒む（Codex #4 の反例）／除外が広すぎる | CLI | 同 FL3 | 除外が無い、commit の検出が無い |
| PK1N 新形式（Phase の行が無い）: 全部に Risk・Goal・Scope・Acceptance Criteria、R3+ に Design Readiness・Contract Ledger・Test Plan、R4 に Recovery・Data Safety を求める。新形式の R3 で Data Safety が無くても通り、R4 で無ければ ERROR、R3 で Contract Ledger が無ければ ERROR。旧形式（Phase の行がある）は今の必須の節（Owner Effort Budget・Contract Probe・Data Safety・Test Design Matrix の link）のまま | 新しい packet を拒む、R3 の計画の芯が抜ける、走っている lane が同期の後に止まる | CLI | `doc-consistency-plan-packet.test.sh` PK1N | 必須の節の表を誤る、形式の見分けを誤る |
| FMT1 packet の「新しい packet の書式」の正例を埋めた合成の新形式の packet（R3・manual なし、R3・`## L3` あり、R4）と、起票の時点の open の lane の旧形式の packet を、PK1（`--target plan`）・PK4・PK5（`check-workflow-git.sh`）・helper の `parse_packet` と `requirements` に通し、全部が受理し、Minimum・manual・r4 が H3 の期待どおり | 読み手ごとに書式の読み方が違う（Codex #3） | integration（合成の repo） | AC8 の手順（PR body に command と出力） | どれかの読み手が別の書式を前提にする |
| KEEP 既存の拒否の case が全部残り通る: base と違う helper、古い head/base の record、broad の不足、Plan 契約の変更で fresh broad、R4 を使い回さない、UI merge の道の拒否、PK5 の ancestry と不変 | 安全の境界が黙って弱まる | 回帰 | `pr-gate.test.py`・`workflow-git-checks.test.sh`・`merge-gate.test.sh` の既存の case | 既存の拒否の case を消す・skip する |

## State Lifecycle Matrix

workflow の状態（計画・review・L3・CI の証拠）の寿命を並べる。UI・DB の状態は変えない。

| State / subject | Initial | Pending | Success | Invalidate | Refetch | Revisit | Restart | Failure | Retry | Evidence |
|---|---|---|---|---|---|---|---|---|---|---|
| 計画の固定（Plan Commit） | packet 起票、`pending` | Plan Review 中 | `plan-approve.sh` が SHA を書く | GA で Amendments に足す | helper が head の packet を読み直す | 再開は open の PR → packet | — | 祖先でない SHA を拒む | 正しい SHA で再実行 | PA1〜PA5・PH1・PH2 |
| 先走りの検査 | pending の packet | branch に code の差分 | 承認の後は検査しない | — | push ごと | — | — | ERROR で push を止める | plan-approve の後に push | PH4〜PH6 |
| Final Review の本数 | Risk と classifier から導く | broad の record | Minimum を満たす | Scope・AC の変更（GA2）で fresh broad | status ごとに導き直す | — | — | 不足で blocker | broad を足す | MN1・MN2・GA1・GA2 |
| manual（L3） | `## L3` 節の有無 | L3 を頼む | PASS の record | runtime の commit・L3 節の変更 | status が確かめ直す | 同期 A/B（#217）・C | — | FAIL は直した項目を取り直す | 取り直しの record | MN3・C1〜C4 |
| review の closure | broad の後 | 是正 | closure の record | lane の runtime の commit | status | docs だけの commit は RC1 | — | RC2 で拒む | 独立の closure | RC1・RC2・GA1 |
| CI の証拠 | push | Draft の run | 同じ head の `Merge gate` success | head が動く | helper `ci()` | — | — | 失敗・欠けで blocker | rerun・dispatch | CI1〜CI3 |

## Adjacent Pattern Audit

| Source pattern / contract | Repository sites inspected | Ported sites | Explicit exclusions and reason | Test / evidence |
|---|---|---|---|---|
| Phase の判定（`phase == implementing`・Phase の 8 値） | `rg -n "phase|Phase" scripts/pr-gate.py scripts/check-workflow-git.sh scripts/doc-consistency-check.sh scripts/tests/` を実装の段で全部挙げる（起票の時点で `scripts/pr-gate.py` の `parse_packet`・`snapshot`・`nonci`・`status`、`check-workflow-git.sh` の Phase の検査を確かめた） | 全部を Plan Commit の有無に置き換える | 旧い packet の Phase 行は読み捨てる | PH1〜PH3 |
| active な packet の定義（起点の tree に無い dated packet） | `scripts/check-workflow-git.sh` の起点の取り方、`pr-gate.py` `requirements` の `active`・`present`・`leaving`、`doc-consistency-check.sh` の `iter_plan_packet_targets` と link の検査 | helper・PK1・PK4・PK5・H2・FL1 を同じ定義に | `--target plan PATH` の明示の対象（今のまま、名指した file を見る） | AR2・AR4〜AR6・PH6・FL2 |
| classifier の呼び方 | `pr-gate.py` `requirements` の `classify-changes.sh --files-from-stdin`、#217 が足す code に触れない条件の関数 | C・RC・R2N・PH4 で同じ関数を使う | `check-workflow-git.sh`（push 前）は branch の版の classifier を使う（base の版を読む API が無い。helper は base の版のまま、D-099） | C3・R2N2・PH4 |
| `Draft (no merge evidence)` の名を前提にする所 | `rg -n "Draft \(no merge evidence\)|ready_for_review" .github scripts docs` を実装の段で全部挙げる（起票の時点で `.github/workflows/ci.yml` の `merge_gate`・trigger、`docs/ci.md`、`docs/DEV_WORKFLOW.md` の Draft PR Checkpoint を確かめた） | 全部 | archive は遡らない | CI1 |
| `Owner Effort Budget`・`介入 N 回目`・`Post-Merge Closeout`・`直近の完了`・`Wave Registry` の語 | 起票の時点の `rg -lF` の結果（packet の Scope の「消す語」の表） | 生きた文書の全部 | `docs/archive/**`・`docs/decision-log.md` の過去の D の本文・`docs/research/**`（時点の調べ） | AC7 |

## Negative Paths

- missing input: `plan-approve.sh` に SHA が無い（PA2）、`--report` の file が無い・行が無い（RP2）、`--review-stage docs` に `--evidence` が無い（RC2）
- invalid input: 短い SHA・大文字の SHA（PA2）、Phase が 8 値の外の旧い packet（PH3 は受ける。評価しない）
- duplicate/ambiguous input: `REVIEWED_HEAD=` が 2 行（RP2）、Amendments の重複（PA3）、merge-base が一意でない（PH6）
- unknown reference: 祖先でない SHA（PA2）、未知の path（C3・R2N2・PH4）
- dependency missing: 起点の ref が無い（PH6・AR2・FL2）、GitHub の API の失敗（helper の今の exit 2 のまま）
- permission/write failure: `plan-approve.sh` の commit の失敗で packet の変更を残さない（PA1 の後片付け）
- dry-run side effect: helper `status` は何も書かない（今のまま）

## Boundary Checks

- threshold: Minimum の 1 と 2（MN1・MN2）、round の上限 3（文だけ）
- null/default: `Amendments: none`（PA3）、packet なしの R2 の `plan_commit` null（R2N1）
- empty/non-empty: 差分が 0 path（PH5 は通す）、`## L3` 節が空（節があれば manual。空の節は PK1N で ERROR にする）
- min/max: なし
- status/policy enum: Risk R0〜R4、`--review-stage` の choices に `docs`
- wire type: RecordV1 は変えない（`version` 1。closure の形は #217 の `{head, base, audit}` と `{head, base, sync_from}` のまま、RC1 は `sync_from` の形を使う）
- internal type: packet の field の dict
- producer/consumer: `plan-approve.sh` → packet → `check-workflow-git.sh`・helper
- round-trip token: Plan Commit の SHA（PA1 → PH2 → PK5）
- precision/range: SHA は 40 桁の小文字 hex
- cross-language parse: bash の checker と python の helper が同じ packet を読む（PK4 は helper の `parse_packet` を呼ぶ今の形を保つ、D-107）

## Compatibility Checks

- old schema/input: 走っている lane の旧い packet（PH3・MN2・MN3・MN4）、#215 の移送の PR（AR1）、#217 の record の形（RC1）
- new schema/input: 最小形の packet（PK1N・PH1・PH2）
- optional field behavior: 旧形式の packet の担当 4 行・Branch は在っても無くても受ける（helper も PK も評価しない）。旧形式の `## Owner Effort Budget` は今までどおり PK1 の必須で、欠ければ ERROR（既存の拒否の test を弱めない）

## Data Safety Checks

- source-derived data: 使わない
- generated outputs: 合成の repo・fake gh の state は tmpdir
- secrets: 使わない（`gh` の token は fake）
- local-only files: `.local/pr-gate/` の capture（今のまま ignored）
- synthetic sample boundaries: packet・record・PR の JSON は全部合成

## Main Wiring / Integration Checks

- helper connected to main path: fake gh の process の class で `status`・`capture`・`record --report`・`record --review-stage docs` が argv から実装に届く
- output reaches manifest/report: record の comment の PATCH の body
- effective config reaches runtime: pre-push と hosted の docs job が `check-workflow-git.sh` の PH4 と doc check の FL1 を起点つきで呼ぶ（`scripts/pre-push.sh`・`.github/workflows/ci.yml` の docs job の step）
- CLI arg reaches implementation: `plan-approve.sh --amend`、`record --report`、`--risk R2`

## Mutation-style Adequacy Questions

- If a key branch is inverted: PH4 の「pending なら止める」の pending の判定を反転すると PH4 と PH8 が red（PH5 は code の差分が無いので、反転しても green のまま。検出の期待は PH4・PH8 に限る）
- If a threshold comparison changes: MN2 の max を min にすると MN2 が red
- If a guard is removed: C2 の L3 節の比較、RC2 の workflow の docs の拒否、GA2 の読めない節の扱い、AR1 の削除の拒否を外すと各 test が red
- If an output field is omitted: RP1 で reviewed head を落とすと record の照合で red
- If output order changes: PA3 の Amendments の順
- If the approval check is removed: `plan_drift` が常に None を返すと PH11 の helper と checker の test が red
- 実注入（Contract Audit の Mutation / anti-tautology）: 実装の段で Writer が上の 6 つを production 側に注入し、red を報告に残す

## Residual Test Gaps

- 文書の書き換え（AGENTS・DEV_WORKFLOW・MANUAL・template）が読み手の振舞いを変えるかは test できない。Plan Review と Final の 2 本が読む
- 「packet の節を発注の固定の問いに移しても同じだけ拾うか」は測れない（v2 §10.1 の 3）。作り直しの後の 3 lane で、問いが拾った finding の数を handoff に残す
