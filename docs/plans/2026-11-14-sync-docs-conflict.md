# Plan Packet: 同期で docs だけが衝突した lane の L3 の使い回しを直す

wave に属さない単独の lane。merge の helper（`scripts/pr-gate.py`）の manual（L3）の使い回しの入口と案内、その test、正本の文（`docs/agent-guidance/merge-evidence.md` の「base同期だけでheadが変わる場合」、`docs/DEV_WORKFLOW.md` の L3 の表、`docs/decision-log.md`）を変える。製品の runtime（`src/**` の非 test・`src-tauri/**`）・CI の job・classifier は変えない。

出所は owner の決定（原文は repo 外の回答台帳にあり、ここには ID で引く）:

- 直す理由: TD-704（2026-11-14）。同期で docs だけが衝突した lane が L3 のやり直しになるのは理屈に合わないので、規則を直す。
- 形: TD-723（2026-11-15。所感 TD-722 を受けた決定）。L3 の後の変更が L3 を壊さないかは closure の review が判定し、owner の 1 行の承認で使い回す。「同期の辻褄合わせか、lane の新しい変更か」の区別はしない。TD-724（2026-11-15）。L3 の後に lane に足した commit も同じ review で使い回す（別の lane にすると決めていた TD-717 は本 lane に吸収する）。
- 受ける path: TD-706（docs と frontend の test の file）、TD-709（Rust の test は入れない）、TD-712（gate の文書は入れない）、TD-714（test の file の根拠は JS の runtime module に入らないこと。生成 CSS への影響は review が見る）、TD-715（同期の merge の中の、衝突していない path の手の編集も受ける）。
- 開く lane: TD-710（R4。`r4` の承認は取り直す）、TD-711（packet の無い R0〜R2）、TD-713（Ready の承認は B の承認と同じ 1 回に束ねる）。
- B の closure の本数: TD-728（2026-11-15。出所は owner の問い）。入ってきたもので分ける。入った変更が docs と frontend の test の file だけなら closure は独立の reviewer 1 本（main 側が frontend の test の file だけの同期を含む）、それ以外は今どおり 2 本。入った変更を数える範囲は、その B の案内の範囲（起点は旧 record の head。下の「規則の形」の「B の closure の本数」）とする（Coordinator の裁定、2026-11-15）。TD-729（2026-11-15。発注 920 の報告の問いを受けた owner の決定）。R4 の lane は入ってきたものに関わらず 2 本のままで、1 本になるのは R3 以下（packet の無い R0〜R2 を含む）だけ。
- 受け入れた限界: TD-727（2026-11-15）。今の機械の使い回し A・C が docs の class 名の literal による生成 CSS の変化を見ないことは、受け入れて記録する。A・C の条件は変えない。指摘の出所は AI の review（PR #257 の書き直した計画の round 1。issuecomment-6896142698）。

この版は TD-723 で全体を書き直した。前の形への計画の review（round 1〜5）の報告と裁定は PR #257 の comment にある（issuecomment-6879653760・6880564987・6894430761・6894828848・6895153579。packet に写さない）。

decision-log の番号: 本 lane は D-154 を予約する。2026-11-15 に `git fetch origin` の後で引き直した: `git grep -n "^## D-15" origin/main -- docs/decision-log.md` の最後は D-152（`b86da99`）、`git grep -n -E "D-15[3-9]" <origin/agent/* の branch> -- docs/decision-log.md docs/plans` の当たりは本 lane と PR #250（`agent/window-close-guard`）の D-153 だけ。

安全の境界: この変更は `docs/DEV_WORKFLOW.md` Workflow State（D-098）が挙げる安全の境界のうち 2 つを改訂する。改訂は通常の packet・PR の経路で行う（本 packet）。

- 「古い証拠の拒否」を緩める向きに変える。L3 の後に lane の手（下の「規則の形」）が入った版を、機械の拒否から、独立の closure と owner の承認に置き換える（owner の決定の日付は上のとおり）。
- 「独立 review」のうち、B の closure の本数を入ってきたもので分ける（owner の決定 TD-728・TD-729、2026-11-15。出所は上の箇条）。R3 以下の lane で、その B の案内の範囲（起点は旧 record の head。下の「規則の形」の「B の closure の本数」）で入った変更が docs と frontend の test の file だけなら独立の reviewer 1 本、それ以外と R4 の lane は今の B と同じ 2 本。code を取り込んだ B を記録した後の、docs と frontend の test の file だけのやり直しも 1 本（Coordinator の裁定、2026-11-15）。今の B（code に触れる同期）のうち、R3 以下の lane で main 側の code が frontend の test の file だけの同期も、2 本から 1 本になる。Final Review の本数、review の closure の record が現在の版の独立の closure を 1 回以上求めること、reviewer が Writer・Coordinator と別の fresh context であることは変えない。

Plan Commit の固定・R4 の承認・fail-closed は変えない（集合の外・判定不能は L3 のやり直しに倒れる）。

## Workflow State

- Risk: R3
- Plan Commit: a43cb55db9442604d2ca98e6f954f1926f17e4be
- Amendments: none

## Risk

Risk: R3

Reason:
merge の関門（helper の manual の使い回しと、その確かめ直し）と gate の文書（`docs/agent-guidance/merge-evidence.md`・`docs/DEV_WORKFLOW.md`）を変える。classifier が `scripts/pr-gate.py` を full、gate の文書を `workflow=true` にするので R2 にできず、Final は 2 本（Double Audit、Codex を含む。座組表）。R4 にしない理由: 破壊的な操作・実データ・secret・不可逆な git の操作を伴わず、RecordV1 の wire の形も GitHub への書込みの経路も変えない。

## Goal

Goal Invariant:

### 最小完了条件

- L3 の PASS の後に入った変更が次の 3 種だけの lane は、L3 をやり直さずに、B の closure（座組表の Final Reviewer の Claude 側と Codex の 2 本。R3 以下の lane で、その B の案内の範囲〈起点は旧 record の head〉で入った変更が docs と frontend の test の file だけなら、Codex の 1 本。下の「B の closure の本数」、TD-728・729）の判定と owner の 1 行の承認で manual を使い回せる（今の B と同じ `record --kind manual --outcome pass --evidence <旧 evidence> --reuse-from SOURCE --reuse-approval POINTER`）。(a) main との同期で取り込む main 側の変更（code を含んでよい。今の B）。(b) 同期の merge の中の lane の手（衝突の解消と、衝突していない path の編集）で、path が全部「集合」の中。(c) lane の commit で、path が全部集合の中。集合は docs（gate の文書を除く）と frontend の test の file（下の「規則の形」）。
- 機械が引き継ぐ範囲は今のまま（A: 自動の merge と再生成だけで、code に触れない同期。C: packet のある R3 の lane の、classifier が docs だけと見る commit）。
- helper が確かめるのは入口だけ: 集合の判定と、packet のある lane で L3 の基準（`## L3` 節。旧形式で無ければ `## Test Plan` 節と link する Matrix）が L3 の版と byte で同じこと。受けるときは、手で解いた path・lane の commit の path・取り込む main 側の code を種ごとに名指し、要る closure の本数を添えて B の command を案内する。合否は判定せず、走った closure の本数も数えない（Coordinator が確かめる）。
- 合否は B の closure が決める: L3 の版から今の head までの変更を見て、(i) L3 の項目に影響しない、(ii) lane の手の普通の正しさ（Scope の「review が見るもの」）。どちらかが不合格なら使い回さない。不合格の後は、直す commit を lane に足して closure を出し直すか、L3 をやり直す。
- 使い回せないとき（集合の外・L3 の基準の変化）、helper は今と同じく manual 以外の record を書かずに止め、理由と path を名指して L3 のやり直しの command だけを案内する。
- helper の `status`・`ready`・`merge` は、manual の `source_head` から今の head まで同じ入口を確かめ直し、review が要る変更があるのに `reuse_approval` の無い manual を拒む。

### 失敗定義

- 集合の外の lane の手（app の code・Rust の test・gate の文書・`env`・未知の path）を受ける。
- packet のある lane の、lane の手（手で解いた path・lane の commit）がある版で、L3 の基準が変わったのに使い回しの record が書かれる（main から自動の merge で取り込んだ Matrix の変化を含む）。
- lane の手（手で解いた path・C に当たらない lane の commit）がある版で、manual を closure と owner の承認なしに引き継ぐ（旧 manual の `reuse_approval` を新しい手の承認として使う場合を含む）。または review の closure を helper の記録（`--review-stage sync`・`docs`）で代える。
- R4 の lane の B、または取り込む main 側に集合の外の path（app の code・Rust・`env`・gate の文書・未知の path）がある B を、closure 1 本で使い回す。または helper の案内がその版で 1 本と言う。
- helper が合否の基準を自分の文で言い直し、正本より狭いか広い基準が closure の発注に写る。
- 不合格の後の一手として、push 済みの履歴の書換え（force push）を求める。
- `scripts/plan-approve.sh` が手で解いた merge を「main の同期」として受ける（計画の review の後に reviewer の読まない編集が merge に紛れる）。
- R4 の `r4` の承認を使い回す、または B の承認に束ねる。
- 今 review なしで通る経路（A・C・docs の closure）に、新しく review か承認を求める。

### 非目的

- 機械が引き継ぐ範囲（A・C）と docs の closure（`--review-stage docs`）を広げる・狭めること。
- 手で解いた path の正しさと、L3 の項目への影響を機械で判定すること。
- 集合を広げること（Rust の test〈TD-709〉、gate の文書〈TD-712〉）。Tailwind の scanner から test の file を外すこと（TD-714）。
- 生成物・`docs/decision-log.md` の union の規則、classifier・CI の job・RecordV1 を変えること。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 調べ（2026-11-15 に測り直した）

`git fetch origin` の後の `origin/main`（`b86da99`）で測った。L3 の版は各 PR の helper の record（GitHub の API の read。manual は 5 本とも pass で `source_head` 無し）の head、Risk は record の `r4`（pending = R4、not-required = R3）、衝突は `git -c merge.directoryRenames=false merge-tree --write-tree --name-only origin/main <PR の head>`。

| PR | Risk | L3 の版 | 今の main との衝突（生成物を含む） | 新しい形での扱い |
|---|---|---|---|---|
| #236 | R3 | `f921644`（PR の head と同じ） | `docs/Plans.md`・`docs/backlog.md`・`docs/daily-monthly-acceptance-runbook.md`・function-design の 24・57・`90-traceability.md`（生成物） | B（手で解いた path は docs の 5 つ。生成物は再生成） |
| #227 | R3 | `c3c9af2`（同上） | `docs/backlog.md`・`90-traceability.md`（生成物） | B |
| #228 | R4 | `eedca58`（同上） | `docs/backlog.md`・function-design の 24・37・45・55・`90-traceability.md`（生成物）・`src/features/daily-report-import/DailyReportImportPage.sd.test.tsx` | B。`r4` は取り直す |
| #216 | R4 | `2ce6966`（同上） | `src/test/page-root-pageshell-sweep.test.ts` | B。`r4` は取り直す |
| #194 | R4 | `31e00a3`（PR の head `e433617` は `fc06c46` を取り込んだ merge で、自動の merge との違いは 0 path） | 無し（exit 0） | 今の B のまま（code に触れる同期） |

- 5 本とも取り込む main 側は code を含む（L3 の版と main の merge-base〈#236 は `19ec25e`、ほかの 4 本は `24e8909`〉から `origin/main` までの `git diff --name-only --no-renames` を `bash scripts/ci/classify-changes.sh --files-from-stdin` に渡すと `rust=true`・`frontend=true`）。今の規則では #236・#227・#228・#216 は生成物と union の外で衝突するので同期だけの条件を満たさず、B も使えずに L3 のやり直しになる。
- classifier の出力（`printf '%s\n' <path> | bash scripts/ci/classify-changes.sh --files-from-stdin` の `=true`）: `docs/backlog.md`・`docs/decision-log.md`・`docs/plans/2026-11-14-sync-docs-conflict.md` は `docs` だけ。`docs/function-design/24-io-csv-import-repo.md` は `rust_drift`・`docs`・`traceability`。`src/test/a.test.ts` と `src/features/x/Y.sd.test.tsx` は `rust_drift`・`frontend`・`traceability`。どれも `rust`・`env`・`generated`・`workflow`・`unknown` は false。classifier が当てる path は必ずどれかの flag が true で、`NON_DOCS` が全部 false の path は `docs` が true（C の受ける path は集合に含まれる）。
- frontend の test の file は app の JS の runtime module に入らない: `package.json` の `build` は `vite build`（entry から import を辿る）で、test の file でも `src/test/` でもない `src/**` から test の file を import する所と `import.meta.glob` は 0 件（`rg -l -e "from ['\"][^'\"]*\.test['\"]" -e "from ['\"][^'\"]*/test/[^'\"]*['\"]" -e "import\.meta\.glob" src --glob '!**/*.test.ts' --glob '!**/*.test.tsx' --glob '!src/test/**' | wc -l` が 0）。集合の pattern に当たる tracked の file は 202（`git ls-files 'src/test/**' 'src/**/*.test.ts' 'src/**/*.test.tsx' | sort -u | wc -l`）。`src` の直下（`**` が 0 段）の test の file は今 0 件（`git ls-files src | rg -c '^src/[^/]+\.test\.tsx?$'` が当たり無し。2026-11-15、`c9b90c0`）。
- route の生成器: `tsr.config.json` と `vite.config.ts` は `routeFileIgnorePrefix` を指定せず（grep 0 件）、既定の `-` で始まる file だけを route tree から外す。`src/routes/**` の今の test の file は `src/routes/-stocktake-route.test.tsx` と `src/routes/settings/-logs.test.ts` の 2 つで、どちらも `-` で始まる。`-` で始まらない `src/routes/**/*.test.tsx` は route として生成され build に入るので、集合に入れない。
- 生成 CSS: `src/styles/globals.css` の `@import "tailwindcss"` は `source(…)` も `@source not` も持たず、Tailwind の scanner は app が import しない file の class の literal も拾う。import されない `src/test/example.test.tsx` に `w-[173px]` を足すと生成 CSS に `width: 173px` が入った（round 1 の Codex の隔離実験、Tailwind 4.2.2。TD-714 の根拠）。本 run の隔離実験（lock と同じ `@tailwindcss/oxide` 4.2.2 の `Scanner` に、`@tailwindcss/vite` が既定で渡す `**/*` の source を与えた）では、`docs/x.md` の `w-[174px]` も `src/test/a.test.tsx` の `w-[173px]` も候補に入った。docs の literal も生成 CSS まで届く: 隔離した copy で `docs/review257-css-probe.md` に `w-[173829px]` を足すと生成 CSS に `width: 173829px` が入り、同じ入力を今の helper の A・C に渡すと、どちらも `reuse_approval` なしで manual を使い回した（PR #257 の書き直した計画の round 1 の Codex の隔離実験、`c9b90c0`、Tailwind 4.2.2 の scanner と compiler。発注 920 の run では流し直していない）。review が見るもの (i) は test の file に限らず lane の手の全部で literal を見る。
- 機械の A・C と生成 CSS（TD-727 の受け入れた限界の記録）: A・C は docs の literal による生成 CSS の変化を見ない。docs の literal が足す rule が画面を変えるのは、その class が実際の描画で使われるとき（app の組立て・連結・実行時の data・依存の markup を含む）。次の検索範囲では、その該当を確認しなかった。網羅的な無影響の保証ではない（式が拾わない組立て方、実行時の data から来る class 名、下の 2 つのほかの依存は見ていない）。測ったのは 2026-11-16、`3a362da`（`src` は `origin/main` の `b86da99` と差分なし）で、`rg` はどれも test の file を除く `src`（`src --glob '!**/*.test.ts' --glob '!**/*.test.tsx' --glob '!src/test/**'`）。式と依存の実測は round 2 の fresh Opus の review のもので、発注 922 の run で流し直した。
  - 接頭辞を問わない組立て（`rg -n -e '[A-Za-z0-9\]]-\$\{'`）: 当たりは 15 件で、class 名は無い（`data-testid` 2、toast の `id` 4 とその comment 1、`id` 3、`htmlFor` 2、`aria-controls` 1、listbox の option の id 1、期間の文言 1）。
  - 連結の 2 式（``rg -n -e "['\"`][A-Za-z0-9:\[\]-]*-['\"`]\s*\+"`` と ``rg -n -e "\+\s*['\"`]-"``）: どちらも 0 件（exit 1）。
  - class の直接の書込み（`rg -n -e "\.className =|setAttribute\(['\"]class|classList\."`）: 0 件（exit 1）。
  - 依存が出す class（本体の checkout の `node_modules` の read。版は lock と同じ）: lucide-react 1.8.0 は `lucide`・`lucide-<icon 名>` と利用側が渡す `className`。sonner 2.0.7 は `sonner-loading-wrapper`・`sonner-loading-bar`・`sonner-spinner`・`sonner-loader` と利用側が渡す `className`・`classNames`。Tailwind の utility 名は無い。

本 lane の merge の後に同期する lane は、取り込む main 側に `scripts/pr-gate.py`（classifier は full）が入るので、A でなく B になる（上の 5 本はもともと code を取り込むので増える手は無い）。L3 を済ませたほかの lane は本 lane の merge の前に merge するほうが手が少ない。同期の前の lane の helper は D-099 で base の版と違うと止まり、base の版の command を示す（今の動き）。

## 規則の形

言葉:

- 「L3 の版」: manual の pass を確かめた版（record の manual の `source_head`、無ければその record の head）。
- 「手で解いた path」: 同期の merge の commit が、自動の merge（`git -c merge.directoryRenames=false merge-tree --write-tree FIRST SECOND`）の結果と違う path と、自動の merge が衝突と出した path（今の `sync_merge` の `paths`）。生成物と、結果が union と一致する `docs/decision-log.md` は今のとおり数えない。衝突の解消と、衝突していない path の編集を分けない（TD-715・TD-723）。
- 「lane の commit」: first parent の列の 1 親の commit で、生成物の外の path を変えるもの。「lane の手」は、手で解いた path と lane の commit。
- 「集合」: current main 側の classifier で `rust`・`env`・`generated`・`workflow`・`unknown` が全部 false の path のうち、(1) docs: `docs` が true で `frontend` が false（`traceability`・`rust_drift` は true でよい。hosted の drift の検査が新しい head で走る）、または (2) frontend の test の file: helper の定数の pattern（`src/test/**`・`src/**/*.test.ts`・`src/**/*.test.tsx`。`**` は 0 段以上の dir で、`src/a.test.ts` も当たる〈classifier の `src/*.test.ts`・`src/*.test.tsx` も star が slash を跨ぎ、`src` の下のどの深さでも test の file として当てる〉。`src/routes/**` の中は basename が `-` で始まるものだけ）に当たる path。pattern は `frontend`・`traceability`・`rust_drift` を外すためだけに使い、ほかの flag の検査を飛ばさない（例 `src/test/.env.test` は `env` で外れる）。rename は `--no-renames` で両側の path を見て、片側でも外なら集合の外。
- 「承認済みの Scope」: packet の `## Scope` 節。packet の無い R0〜R2 の lane は PR body の 5 行の「範囲」の行。
- 「review が要る変更」: 手で解いた path、C に当たらない lane の commit、取り込む main 側の code。
- 「B の closure の本数」（TD-728・729）: 2 本（座組表の Final Reviewer の Claude 側と Codex。今の B と同じ）。次の両方を満たすときだけ 1 本（Codex。Writer・Coordinator と別の fresh context の独立の reviewer）。(1) lane の Risk が R3 以下（packet の無い R0〜R2 を含む。R4 の lane は入った変更に関わらず 2 本）。(2) その B の案内の範囲（起点は今の B と同じ旧 record の head）で取り込む main 側の path が、`docs/function-design/90-traceability.md`（生成物の docs。今の「code に触れない条件」でも code でない）を除いて全部集合の中（main 側の path が無い列を含む）。lane の手は B に入る時点で全部集合の中なので、本数は Risk と main 側の path だけで決まる。(2) は本数のためだけの判定で、今の「code に触れない条件」（`code_paths`・`CODE_FLAGS`）と A の条件は変えない。main 側の frontend の test の file は集合の中（1 本の側）、app の code・Rust・`src/lib/bindings.ts`・`env`・gate の文書（`workflow=true`）・未知の path は集合の外（2 本の側）。

誰が判定するか。L3 の版から今の head までの first parent の列を helper が辿り、行ごとに当てる。「L3 をやり直す」の列に 1 つでも当たれば L3 のやり直し、当たらずに「review」の列に 1 つでも当たれば B、全部が「機械」の列なら A。

| L3 の後に入った変更 | 機械が引き継ぐ（今のまま） | review で使い回す（B） | L3 をやり直す |
|---|---|---|---|
| 同期で取り込む main 側の変更 | code に触れない（A） | code に触れる（今の B） | — |
| 同期の merge の中の lane の手 | 生成物の再生成、union と一致する `docs/decision-log.md`（数えない） | 手で解いた path が全部集合の中 | 集合の外が 1 つでもある |
| lane の commit | 生成物だけの再生成。packet のある R3 の lane で classifier が docs だけと見る commit（C） | C に当たらず、path が全部集合の中（function-design の docs、test の file、R4 の lane と packet の無い lane の docs） | 集合の外が 1 つでもある |
| L3 の基準（packet の `## L3` 節。旧形式で無ければ `## Test Plan` 節と link する Matrix） | — | — | lane の手がある版で、L3 の版と byte で違う（packet の無い lane は比べない。基準は PR body にある） |

B は、B の closure の合格（本数は上の「B の closure の本数」、見るものは Scope の「review が見るもの」）と owner の 1 行の承認で、今の B の command を記録する。command は R3・R4・packet の無い R0〜R2 のどれでも同じ（Risk で違うのは本数だけ）。

C を B に置き換えない: C は review なしで通る経路（D-145 (6)。P3 だけの docs の是正）で、B に寄せると今より手が増える。A と同じ理由（機械が判定でき、review が要らない）で機械の列に残し、C に当たらない lane の commit だけを B に入れる。

active な packet と Matrix を lane の手が変えても、path では拒まない（今の C と同じ扱い）。L3 の基準は上の表の比較、計画の節は `plan_drift`、Workflow State は `gate_conditions` と PK5 が今のとおり止める。それ以外の節の変更は B の closure が見る。

変えない経路:

- review の closure の機械の記録（`--review-stage sync`・`docs`）は lane の手を今のとおり受けない。手で解いた同期と C に当たらない lane の commit は監査の closure を要り、B の closure がそれを兼ねる（1 本のときも、review の closure の record が求める独立の closure 1 回以上を満たす）。
- `scripts/plan-approve.sh` は今の判定のまま、手で解いた merge を「not a sync of main」で拒む。
- `sync_from` の形の closure の確かめ直し、`r4` を引き継がないこと、`--reuse-from` の無い manual の record（L3 のやり直し・新しい L3）が判定を省くこと。
- `AGENTS.md` の Ready の承認: helper が「同期だけ」と判定した同期だけが承認を保つ。lane の手がある版は承認を取り直し、B の承認と同じ 1 回に束ねてよい（TD-713）。

境界の 4 観点:

- 数の範囲: 手で解いた path・lane の commit の path・取り込む main 側の code は、どれも 0 個以上で上限なし（文には種ごとに全部を sorted、`, ` 区切りで並べる）。同期の merge は 0 回以上（merge の無い列は、今の C と同じく起点の版が今の base を含むことを求める）。手で解いた path は列の全部の merge の和。B の closure の本数の分かれ目は、R3 以下の lane で、取り込む main 側の集合の外の path の 0 個（1 本）と 1 個（2 本）の間。
- 0 の表示: 0 個の種の行は出さない。手で解いた path も C に当たらない lane の commit も無いとき（code だけ、または 3 種とも無い）は、判定・記録は今と同じで、新しい行を出さない。code だけの文は、本数の語だけを「B の closure の本数」の値から出し、2 本のときは今の文と同じ。
- 失敗の時点と順序: どれも書込みの前（`record` は server への PATCH/POST の前）に止まる。順は「列の判定（head から first parent を辿り、最初に外れた commit で止まる。merge の中は main の祖先 → merge-base の一意 → `merge-tree`〈終了値 0/1 の外は判定不能 exit 2〉→ 集合の外）→ L3 の比較 → 承認の有無」。前の段で止まれば後の段の理由は出さない（L3 の基準の変化と main 側の code が同時にあれば L3 の理由だけが出る）。closure の合否は人の判定で、この順序の外にある。
- 種類の混在: review が要る変更の 3 種が混ざれば、B の案内に種ごとの行を並べる。集合の外が 1 つでもあれば、ほかが集合の中でも L3 のやり直しだけ。C の commit と review が要る変更が混ざれば B（closure は全部を見る）。手で解いた merge と自動の merge が列に混ざれば B。closure の不合格の後に足した直す commit は lane の commit として同じ規則に入り、前の merge の手で解いた path と並べて名指す。

## Scope

- **review が見るもの**（B の合否と不合格の後の一手。定義はこの項目だけが持つ。実装では merge-evidence の「manualの使い回し」の B がこの文を持ち、D-154 は要旨と参照、helper は節を名指すだけで言い直さない）:
  - 見る範囲: L3 の版から今の head までの変更の全部。helper が名指す 3 種を含み、名指しに限らない（C で機械が受ける commit と、自動の merge で取り込んだ main 側の docs も範囲の中）。前に B で使い回した範囲は、その時の closure の証跡を踏まえてよい（今の closure の契約と同じ）。
  - (i) L3 の項目に影響しない。意味は D-146（各項目の合否と、確かめた操作・業務の結果を変えない。画面の表層だけの変化は数えない）。取り込む main 側の code と、lane の手の class の literal が Tailwind の scanner を経て生成 CSS に入る変化を含む（literal が増えたか減ったか → 生成された class が実際の描画で使われるか〈app の組立て・連結・実行時の data・依存の markup を含む〉→ 当たるなら操作の合否・業務の結果を変えるか、の順に見る。TD-714）。
  - (ii) lane の手の普通の正しさ。手で解いた path は両側（lane 側と main 側）の変更を落とさない。test を消す・skip する・弱めることをしない。承認済みの Scope の中にある（Scope の外の変更は今の経路〈Gated Amendment で計画を変える〉で扱い、それが通るまで (ii) は不合格）。
  - (i)(ii) のどちらかが不合格なら使い回さない。不合格の後は、直す commit を lane に足す（集合の中なら helper が新しい head で同じ規則の B を案内するので、closure を出し直す。集合の外に触れる直しは L3 のやり直し）か、L3 をやり直す（`record --kind manual --outcome pending --evidence POINTER`）。push 済みの履歴は書き換えない。
  - 本数: 上の「B の closure の本数」（2 本。R3 以下の lane で、その B の案内の範囲〈起点は旧 record の head〉で入った変更が docs と frontend の test の file だけなら独立の reviewer 1 本。TD-728・729）。1 本のときの reviewer は Codex（座組表の Final Reviewer の行: Codex の本務は合否判定で、取り直しでも外さない）。要る本数の closure が合格したことは、Coordinator が owner に B の承認を求める前に確かめる（helper は数えない）。
  - 承認と証跡: owner の 1 行の承認の pointer を `--reuse-approval` に置く。R2〜R4 は B の closure の証跡（2 本なら両方）を review の closure の evidence に並べ、review の要らない R0/R1 の manual の lane は承認の pointer の先に置く（今の B と同じ）。Ready の承認は同じ 1 回に束ねてよい（TD-713）。`r4` は束ねず、新しい head で `record --kind r4` を取り直す（TD-710）。
- `scripts/pr-gate.py`（計画の固定の後の実装の run。名は例で、振舞いが契約）:
  - 入口の切替: `sync_merge`・`sync_merges`・`reusable` に、lane の手を集合で受ける引数（例 `hands=False`）を足す。`True` を渡すのは使い回しの経路の 3 か所だけ（`carry_manual`、`record --reuse-from`、`nonci` の manual の `source_head` からの確かめ直し）。`plan-approve.sh` の `sync_merge`、`sync_closure`、`nonci` の `sync_from` の確かめ直しは既定のまま。呼出し側は `rg -n "sync_merges\(|reusable\(|sync_merge\(" scripts` で列挙し直す。
  - 既定（`False`）は今の処理だけを通り、classifier も新しい属性も触らない（`plan-approve.sh` は `object.__new__(g.Gate)` で `__init__` を通らずに `sync_merge` を呼ぶ）。
  - `sync_merge(..., hands=True)`: 今の `paths` から生成物を除き、`docs/decision-log.md` は union と一致すれば除く（一致しなければ今のように拒まず、ほかの path と同じに扱う）。残りに集合の外が 1 つでもあれば「集合の外」と path を名指して拒む。全部が集合の中なら `self.resolved`（path の集合）に足す。戻り値の 4 つ組 `(commit, first, second, merge_base)` は変えない（`plan-approve.sh` と `code_paths` が位置で受けている）。
  - `sync_merges(..., hands=True)`: 1 親の commit の生成物の外の path を、(1) C の条件（今の `docs` と `NON_DOCS`）に当たれば今のとおり `self.docs_commits` へ、(2) 当たらず全部が集合の中なら `self.lane`（path の集合）へ足し、(3) 集合の外が 1 つでもあれば「集合の外」と path を名指して拒む。merge の無い列は、今の C と同じく起点の版が今の base を含むことを求める。呼ぶたびに `self.resolved`・`self.lane` を空にしてから集める（`self.docs_commits` と同じ置き方）。
  - 属性の持ち方: `resolved = ()`・`lane = ()` を class の既定として `docs_commits = ()` の隣に置き、足すときは in-place でなく置き換える（例 `self.resolved = {*self.resolved, *paths}`）。`sync_merges` を通さずに `sync_merge` を直接呼ぶ gate では class の既定 `()` のままで、`()` に `add` は無い。空の判定は真偽だけで行う（`()` も空の集合も偽）。
  - 集合の判定: path ごとに current main 側の classifier（`self.contents('scripts/ci/classify-changes.sh', base)`）を通し、上の「集合」の定義で決める。frontend の test の file の pattern は定数（例 `TEST_FILES`）として `GENERATED` の近くに置き、classifier から解析して得ない。`ponytail:` comment に限界（「JS の runtime module に入らない」は route の生成器の `routeFileIgnorePrefix` の既定 `-` と、app の code が test の file を import しないことに依る。生成 CSS への影響は機械で見ず review が見る。TD-714）と見直す条件（`tsr.config.json` が ignore の接頭辞・route の dir を変えたとき、app の code から test の file への import が出たとき、CSS の scanner の source の設定が変わったとき、Rust の分かれた test の dir を入れる決定が出たとき）を書く。
  - L3 の比較（3 か所が同じ 1 つの関数を呼ぶ）: packet のある lane で、`self.docs_commits`・`self.resolved`・`self.lane` のどれかがあれば、`l3_change(source, head)` の理由で止める。packet の無い lane（TD-711）は呼ばない。今の `self.docs_commits and self.l3_change(...)` の 3 か所を置き換える。
  - 3 か所の順序: 列の判定（`reusable(..., code=True, hands=True)` で code の path も受け取る）→ L3 の比較 → 承認の有無。承認の有無は 1 段で、review が要る変更（`self.resolved`・`self.lane`・code）があるのに承認が無ければ止める。今の `record --reuse-from` と `nonci` は code の require が `l3_change` より先なので後ろへ動かし、`code=args.reuse_approval is None`・`code='reuse_approval' not in manual` の引数の分岐は require の条件に移す。
  - `carry_manual`: review が要る変更があれば、旧 manual が `reuse_approval` を持っていても引き継がず、B の完全な command と L3 のやり直しの command を示して何も書かずに止まる。無ければ今のとおり引き継ぐ（A。旧 manual の `reuse_approval` はそのまま残す）。
  - `record --reuse-from`: 承認は、この record の CLI の `--reuse-approval` の有無で判定する（結果に入る approval〈旧 manual から引き継ぐものを含む〉で判定しない）。`self.resolved` か `self.lane` があって引数が無ければ、`needs --reuse-approval` の句を持つ文で拒む。
  - `nonci`: manual の `source_head` からの確かめ直しで、review が要る変更があるのに manual に `reuse_approval` が無ければ拒む。
  - 止める文と B の案内: 拒む側は「集合の外」と path（L3 のやり直しの command だけを案内する）。受ける側（`carry_manual` が B の command と L3 のやり直しの command を示して止まる文）には、1 個以上ある種だけを「手で解いた path: …」「lane の commit の path: …」「取り込む main 側の code: …」の別の行で添え、最後に基準を言い直さずに、要る本数と正本を名指す 1 行（「closure N 本が見るものと合否: `docs/agent-guidance/merge-evidence.md` の「manualの使い回し」の B」。N は「B の closure の本数」の値）を置く。helper の文で本数を言う所は、この 1 行と、今の code だけの文の 2 か所（`carry_manual` の B の案内と `record --reuse-from` の拒む文の "two sync closures"）で、どれも同じ 1 つの値から出す。lane の手が無く本数が 2 のときの文は今の文と同じ。
  - 本数の判定（TD-728・729。本数のためだけ）: 使い回しの経路の列の判定（`reusable(..., hands=True)`）が、同じ merge の列で取り込む main 側の path（集め方は `code_paths` と同じ。各 merge の 2 親の merge-base から second parent まで、`--no-renames` の両側の和）から `docs/function-design/90-traceability.md` を除き、残りを lane の手と同じ集合の判定に通す。全部が集合の中で、lane の Risk（呼出し側が今持っている requirements の `risk`）が R4 でなければ 1、それ以外は 2。値は `self.resolved`・`self.lane` と同じ置き方の属性に置き、class の既定は 2（例 `closures = 2`。使い回しの経路を通らない gate と、判定の前に止まった gate は 2 本の側）。`code_paths`・`CODE_FLAGS`・A の条件・`reusable` の戻り値は変えない。
  - 本数の確かめ: helper は走った closure の本数を数えず、判定もしない（今の B の 2 本も機械で数えていない）。今の record の形では数えられない: review の closure は audit を 1 つだけ持ち（`closure: {head, base, audit}`。2 本のときも 1 回の record で、証跡の pointer を `--evidence` に並べる）、`--pr-reviews` は今の head の本文のある PR review を読み切ったことの照合（D-107）で closure の本数ではなく、review の要らない R0/R1 の manual の lane は review の closure を持たない。数えるには RecordV1 を変えることになる（非目的）。helper に足すのは、上の「本数の判定」と、その値を文に出す所だけ（record の field を足さない）。要る本数の closure が合格したことは Coordinator の確かめに置く（「review が見るもの」の「本数」）。
- `scripts/tests/pr-gate.test.py`: 下の Test Plan の test を足す。今の test のうち期待が変わるのは、使い回しの経路で lane の commit を止めていた 4 つ: `test_lane_commit_stops_with_reason`（packet の無い lane の `docs/lane.md` の commit。TD-724・TD-711 で B の案内に変わる）、`test_c3_runtime_unknown_workflow_and_rename_stop`（止める入力は同じで、理由の句が「集合の外」に変わる）、`test_c4_r4_never_takes_docs_route`（R4 の lane の docs の commit。機械で引き継がないことは同じで、次の一手が L3 のやり直しだけから B の案内に変わる。TD-710・TD-724）、`test_nonci_rechecks_manual_chain`（`reuse_approval` の無い manual の後の `docs/lane.md` の commit。status が拒むことは同じで、理由が lane の commit の名指しから承認が無いことに変わる）。隔離した copy（`git archive HEAD`）に入口の試作を当てて `python3 -I scripts/tests/pr-gate.test.py` を回すと、今の 167 test のうち落ちたのはこの 4 つだけだった（subTest を含めて failures=7。2026-11-15、本 run）。変える理由は owner の決定（2026-11-15）で、集合の外の lane の commit が L3 のやり直しだけで止まること（`--reuse-approval` を案内しない、`code_paths` を呼ばない）は同じ test に入力を変えて残す。`SyncOnly` の拒む向きの test（`test_sync_only_rejects_lane_commit`・`test_sync_only_rejects_manual_conflict_resolution`・`test_sync_only_rejects_edit_inside_merge`・`test_sync_only_decision_log_union_conflict` ほか）は既定の判定として変えない。
- `scripts/tests/plan-approve.test.sh`: PA5 に拒む 2 組を足す。(a) 集合の中の path（例 `docs/x.md`）を lane と main の両方が変えて衝突させ、手で解いた merge、(b) 衝突していない `docs/x.md` を merge の中で編集した merge。どちらも「not a sync of main」で書込みの前に拒まれる。今の 3 組（正常な同期 1 組の受理、merge の中の編集と main の外の branch の merge の拒否 2 組）の assertion は変えない。今の `hidden.txt` の組は classifier で `unknown` になり、引数の既定を見分けない。
- `docs/agent-guidance/merge-evidence.md`: MG-D13 の行、「同期だけの条件」（使い回しの経路では lane の手を集合で受けること）、「manualの使い回し」（A の条件、B の入口の 3 種、B の closure の本数と、数えるのは Coordinator であること、上の「review が見るもの」の全文、使い回せないときの次の一手）、「reviewのclosure」（B の closure が監査の closure を兼ねる文から本数の「2本」を外す）、「使い回しの元は…確かめ直し」の段落、「Helperの境界」の `--reuse-from` の文。「L3 の使い回し C」の節は変えない。節の見出し「base同期だけでheadが変わる場合」は `AGENTS.md` などから節名で引かれているので変えず、lane の commit も扱うことは前文の 1 文で言う。長い段落は段落ごと書き直す（継ぎ足さない）。
- `docs/DEV_WORKFLOW.md`:
  - L3 の表: 上の「誰が判定するか」の表に合わせて書き直す（A と C の行は今のまま。B の行を「取り込んだ main 側の code、同期の merge の中の手の解消・編集、C に当たらない lane の commit で、lane の手が docs と frontend の test の file の中」、その「L3」の列を「closure（2 本。R3 以下の lane で、その B の案内の範囲〈起点は旧 record の head。merge-evidence の B〉で入った変更が docs と frontend の test の file だけなら 1 本）と owner の 1 行の承認で使い回す」、「誰が判定」の列を「Final Reviewer（本数は左の列）・owner」にし、「取り直す」の行を「lane の手が集合の外〈runtime・Rust の test・gate の文書・未知の path〉、`## L3` 節の変更」にする）。
  - Stacked train の 3 つ目の箇条（「衝突の解消が実装の file に及んだら…（docs だけの解消は差分の確認だけ）」）を、「同期の merge で衝突を解くか手で編集した版は、path に関わらず監査の closure を要る（helper の記録の closure にならない）。manual のある lane は merge-evidence の B で L3 を使い回すか取り直す」の形に書き直す。Wave Operation の「衝突の解消が内容を変えたら通常の再検証と現在版の closure を行う」は今も正しいので残す。
- `docs/decision-log.md`:
  - 末尾に D-154。出所（上の TD の ID）。Decision（上の「規則の形」の要旨。基準の文は merge-evidence の 1 か所が持ち、D-154 は要旨と参照〈D-146 と同じ置き方〉）。Alternatives（今のまま L3 をやり直す〈TD-704 で採らない〉、lane の手を機械だけで受ける〈片側を落とした解消を機械は見分けられない〉、手の編集の目的〈同期の辻褄合わせか〉を closure に判定させる〈TD-723 でやめた〉、C を B に置き換える〈今より手が増える〉、B の closure を入った変更に関わらず 2 本にする〈TD-728 で採らない〉、R4 の lane も 1 本にする〈TD-729 で採らない〉）。Guarantee range（lane の手の正しさと L3 の項目への無影響は B の closure と owner の判断で、機械の保証でない。B の closure の本数は helper が案内に出すだけで、走った本数は Coordinator が確かめる〈今の B の 2 本と同じく機械で数えない〉。R3 以下の lane で、その B の案内の範囲で入った変更が docs と frontend の test の file だけの B は、独立の reviewer 1 本の判断に依る〈TD-728・729、2026-11-15〉。lane の手の frontend の test の file と docs の literal が生成 CSS に入る影響も同じ。機械の A・C は docs の literal による生成 CSS の変化を見ない: 今からある限界で、TD-727〈2026-11-15〉で受け入れた。調べの「機械の A・C と生成 CSS」に記載した検索範囲〈接頭辞を問わない組立て、連結の 2 式、class の直接の書込み、test を除く `src`。依存の lucide-react 1.8.0・sonner 2.0.7 が出す class の実測〉では、docs の literal から生成された class が描画で使われる該当を確認しなかった。網羅的な無影響の保証ではない。D-154 には式・結果・日付・head と、指摘の出所〈AI の review、PR #257 の round 1・round 2〉を書く。closure の報告の SHA と B の record の head の一致は今の B と同じで、manual の record は照合を持たず、R2〜R4 は後の review の closure の record が照合する）。Compatibility（D-144 の (1)・(4)・(6) のうち、使い回しの経路が受ける範囲を置き換える。D-145 (6) の C と docs の closure、D-146 の「影響なし」の意味は変えない。D-144 (4)・(5) の B の closure の本数〈2 本〉は、R3 以下の lane で、その B の案内の範囲で入った変更が docs と frontend の test の file だけのとき 1 本に置き換える〈D-144 の B のうち、main 側の code が frontend の test の file だけの同期が当たる。D-144 の `Superseded in part by: D-154` の行が覆う〉。D-146 の Guarantee range・Compatibility が書く 2 本は D-144 の当時の本数の引用で、D-146 の Decision は変えず、D-146 には行を足さない）。Revisit（B で使い回した L3 の項目が merge の後に FAIL したとき、1 本の closure で使い回した版で見落としが見つかったとき、closure が片側を落とした解消か生成 CSS の影響を見落としたとき、app の code が test の file を import したとき、app が class 名を組み立てて使うようになったとき、依存が utility 名の class を出すようになったとき、route の生成器の ignore の接頭辞か CSS の scanner の source の設定が変わったとき、Rust の分かれた test の dir か gate の文書を入れる必要が出たとき）。
  - D-144 に足すのは `Superseded in part by: D-154` の 1 行だけ（D-144 の本文は書き換えない）。D-144 の Compatibility の L3 の順の 1 文は D-124 の残りで、D-124 の `Superseded in part by: D-145` の行が記録済みなので、D-154 の対象でない。
- `docs/backlog.md`: 項目「衝突を解いた版の manual の再利用（観測）」を実装の merge で済みにし、将来形の文「残る観測は…owner が改めて決める」を消す（計画の commit は起票の 1 文だけ）。
- 本 packet（本 commit）。

Non-scope: `AGENTS.md`、`scripts/ci/classify-changes.sh`、`.github/**`、RecordV1、`scripts/plan-approve.sh`（呼出しは既定のまま。test だけ足す）、`sync_closure`。

## Acceptance Criteria

- AC1: `python3 scripts/tests/pr-gate.test.py` が全 PASS で、Test Plan の T1〜T16 が全部ある。
- AC2: `bash scripts/tests/run-workflow-tests.sh` が全 PASS。`plan-approve.test.sh` の PA5 は、既存の正常な同期 1 組が受理を保ち（今の受理の assertion `PA5 sync` を変えない）、既存の拒否 2 組（merge の中の編集・main の外の branch の merge）と足した拒否 2 組が「not a sync of main」で書込みの前に拒まれる。
- AC3: Test Plan の mutation（M1〜M13）を 1 つずつ実装に入れると、名指した test が FAIL（exit 非 0）になり、戻すと全 PASS に戻ることを実装の run の報告に書く（DEV_WORKFLOW Contract Audit の Mutation / anti-tautology）。M2 では、PA5 の足した (a) の組で `plan-approve.sh` が packet を書かずに exit 非 0 で止まることを出力で示す（`plan-approve.sh` の gate は `endpoint` を持たないので、既定を True にすると classifier の読取りで例外になりうる）。既定を True にすると例外でなく判定として手の解消を受けてしまうことの証明は、classifier の読取りを合成 base の実 file に patch した T4 の red に置く。
- AC4: `bash scripts/doc-consistency-check.sh` と `--target plan docs/plans/2026-11-14-sync-docs-conflict.md` が ERROR なし、`bash scripts/check-workflow-git.sh` が exit 0。
- AC5: merge-evidence・DEV_WORKFLOW（L3 の表・Stacked train）・decision-log（D-154、D-144 の `Superseded in part by: D-154` の 1 行）・backlog の文が helper の振舞いと同じ（「誰が判定するか」の表、集合、変えない経路、順序）。「review が見るもの」の文は merge-evidence の「manualの使い回し」の B の 1 か所だけにあり、D-154 は要旨と参照、helper の案内は本数と節を名指す 1 行で、基準を言い直した文がほかに無い（`rg -n "両側.*落とさ|L3 の項目に影響" docs/agent-guidance docs/DEV_WORKFLOW.md docs/decision-log.md scripts` の当たりを列挙して見比べる）。「L3 の使い回し C」の節は変わらない（`git diff` で C の節に差分が無い）。
- AC6: 合成の repo の test で、次の形が B の command で manual を使い回せ、`--reuse-approval` の無い record と `--review-stage sync` は何も書かずに止まる: #236 の形（docs と function-design の衝突を手で解き、main 側が code を含む。案内は手で解いた path と code を別の行で名指す）、#216 の形（`src/test/*.test.ts` の衝突）、#228 の形（`src/**/*.sd.test.tsx` と function-design の衝突。R4 で、使い回した後も `r4` は pending）、L3 の後の lane の commit（R3 の lane の function-design の docs、R4 の lane の docs、packet の無い R0/R1 の lane の docs。R0/R1 の `--reuse-from` だけの明示の A は止まる）。packet のある R3 の lane の docs だけの commit は今のとおり承認なしで引き継がれる（C）。集合の外（`src/lib/x.ts`・`workflow=true` の docs・`src-tauri/**` の Rust の test・`src/routes/foo.test.tsx`・`src/test/.env.test`）は、merge の中でも lane の commit でも「集合の外」で止まり、L3 のやり直しだけを案内する。
- AC7: L3 の基準が変わった版（別の docs を手で解き、旧形式の packet が link する Matrix の変更は main から自動の merge で取り込んだ同期）で、`record --kind manual --outcome pass --reuse-from SOURCE --reuse-approval POINTER` が書込みの前に L3 の理由で止まり、server の comment が変わらない。
- AC8: 連鎖で、B の後の code に触れない同期は A で manual を引き継ぎ、`status` の確かめ直しが通る。B の後に新しい lane の手が入ると、旧 manual の `reuse_approval` があっても A にならず、新しい `--reuse-approval` の B を要る。
- AC9: Test Plan の T16 が PASS で、helper の文の本数が、R3 以下の lane で取り込む main 側の path が（`docs/function-design/90-traceability.md` を除いて）全部集合の中なら 1 本、集合の外が 1 つでもあれば 2 本、R4 の lane は入った変更に関わらず 2 本（main 側が frontend の test の file だけの組は、R3 以下で 1 本、R4 で 2 本）。merge-evidence の「manualの使い回し」の B・DEV_WORKFLOW の L3 の表・D-154 が同じ分かれ目と TD-728・TD-729 を書き、走った closure の本数は helper が数えず Coordinator が確かめると書いてある（`rg -n "TD-728|TD-729" docs/agent-guidance/merge-evidence.md docs/DEV_WORKFLOW.md docs/decision-log.md` の当たりを列挙して見比べる）。

## Design Readiness

- 引用する設計正本（節まで）: `docs/agent-guidance/merge-evidence.md` の MG-D13・「base同期だけでheadが変わる場合」（同期だけの条件、codeに触れない条件、manualの使い回し A・B、reviewのclosure）・「lane の docs だけの commit（C）と GA の後の closure」・「Helperの境界」、`docs/DEV_WORKFLOW.md` の L3 の表・Stacked train・Workflow State（D-098 の安全の境界）・Risk Tiers と Draft・Ready・merge（PR body の 5 行）、`docs/decision-log.md` D-098・D-124・D-144・D-145 (6)(7)・D-146、`docs/AGENT_OPERATING_MANUAL.md` の座組（Final Reviewer の行と独立性）、`AGENTS.md` Working Rules の「継ぎ足さない」「最小の実装（D-147）」と Decision and Approval Boundaries の Ready の承認の範囲。
- 必要な設計成果物: 正本の文の改訂は updated in this PR（計画の固定の後の実装の run で、helper と test と同じ PR に入れる。規則の文と helper を同時に変え、食い違う期間を作らない）。
- plan にしかない durable な判断の昇格先: 「規則の形」と「review が見るもの」→ merge-evidence の該当の文と D-154。調べの command と出力は本 packet に残す。
- 前提・制約と延期した design gap: lane の手の正しさと L3 の項目への影響は機械で判定しない（B の closure と owner の判断。D-154 の Guarantee range）。B の closure が要る本数だけ走ったことも機械で確かめず、Coordinator が確かめる（今の B と同じ。TD-728 の 1 本は独立の reviewer 1 本の判断に依る）。lane の手の docs の literal による生成 CSS の変化は、review が見るもの (i) で見る。機械の A・C は今のままで、docs の literal による生成 CSS の変化を見ない。今からある限界で、TD-727（2026-11-15）で受け入れた（指摘の出所は AI の review、PR #257 の round 1・round 2）。調べの「機械の A・C と生成 CSS」に記載した検索範囲では、docs の literal から生成された class が描画で使われる該当を確認しなかった（網羅的な無影響の保証ではない）。見直す条件は D-154 の Revisit に置く。classifier が将来 docs の分類を変えれば集合も変わる（current main 側の classifier を使う今の方針どおり）。
- 絶対保証の例外と抜け道: 「code に触れる lane の手を受けない」は classifier の分類と定数の pattern に依る。classifier が未知の path を full にするので、未登録の path は受けない側に倒れる。pattern は `frontend`・`traceability`・`rust_drift` だけを外す。`src/routes/**` の test の file は route の生成器の既定の ignore の接頭辞に依り、frontend の test の file の「JS の runtime module に入らない」は今の `vite build` と import の実測に依る（どちらも崩れる条件を D-154 の Revisit と helper の `ponytail:` comment に置く）。`merge-tree` の終了値 0/1 の外は判定不能として受けない側に倒れる。
- 判定: ready（未決の問いは無い。owner の決定は前文の ID）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| MG-D13 入口 | merge-evidence 設計判断の表・「同期だけの条件」 | `sync_merge`・`sync_merges` の `hands=True` と集合の判定、`TEST_FILES` | T1・T2・T3・T14 | 非対象（helper） |
| MG-D13 既定 | merge-evidence「同期だけの条件」・「Helperの境界」 | 既定 `False` と渡す 3 か所、属性の持ち方 | T4、今の `SyncOnly` の test、`plan-approve.test.sh` PA5 | 非対象 |
| MG-D13 A | merge-evidence「manualの使い回し」(A 自動) | `carry_manual`（review が要る変更があれば引き継がない） | T5・T15 | 非対象 |
| MG-D13 B | merge-evidence「manualの使い回し」(B 承認) | `record --reuse-from`（CLI の引数で判定）、R4・packet の無い lane | T6・T9・T11・T15 | 非対象 |
| MG-D13 B の基準 | merge-evidence「manualの使い回し」(B 承認) の「review が見るもの」 | 文だけ（helper は節を名指す 1 行。分岐は足さない） | T12（名指す 1 行の有無）。合否は closure の判断（機械の test を置かない契約） | 非対象 |
| MG-D13 B の本数 | merge-evidence「manualの使い回し」(B 承認) の本数の文（TD-728・729） | 本数の判定（main 側の path の集合の判定と Risk）と、その値を出す helper の文（走った本数は数えない） | T16（helper の文の本数）・T12（2 本の code だけの文が今のまま）。走った closure の本数は Coordinator の確かめ（機械の test を置かない契約） | 非対象 |
| MG-D13 L3 の比較 | merge-evidence「使い回しの元は…確かめ直し」の段落・「L3 の使い回し C」 | 3 か所の同じ関数と順序 | T10・T11・T13 | 非対象 |
| MG-D13 closure | merge-evidence「reviewのclosure」 | `sync_closure`（変えない） | T7 | 非対象 |
| MG-D13 確かめ直し | merge-evidence「使い回しの元は…確かめ直し」の段落 | `nonci` | T8・T15 | 非対象 |
| MG-D13 止める文 | merge-evidence「manualの使い回し」の使い回せないときの次の一手 | 止める文と B の案内 | T12 | 非対象 |
| D-145 (6) C | merge-evidence「L3 の使い回し C」 | 変えない（`docs_commits` と `l3_change`） | T3、`scripts/tests/pr-gate.test.py` の今の C1・C2・C5・RC1〜RC4 の test（PK3 は `scripts/tests` を探さないので名を token にしない） | 非対象 |
| D-145 (7) 計画の節 | merge-evidence「docs の closure」・DEV_WORKFLOW Workflow State | 変えない（`plan_drift`・`gate_conditions`） | 今の test | 非対象 |
| D-144 | decision-log D-144 (1)〜(7) | `Superseded in part by: D-154` の 1 行 | — | 非対象 |
| D-146 | decision-log D-146（B の「影響なし」） | 変えない（「review が見るもの」の (i) が引く） | — | 非対象 |
| D-098 | DEV_WORKFLOW Workflow State の安全の境界（古い証拠の拒否、独立 review のうち B の closure の本数） | 「誰が判定するか」の表の「L3 をやり直す」の列と、案内の本数の分かれ目を helper の test で固定 | T1〜T4・T10・T13・T16 | 非対象 |
| PA5 | `scripts/plan-approve.sh` (a) の merge の判定 | 変えない。test に拒否の 2 組を足す（今の受理 1 組と拒否 2 組は保つ） | `plan-approve.test.sh` PA5 | 非対象 |
| DEV-L3 | DEV_WORKFLOW L3 の表 | 表の書き直し | doc-consistency | 非対象 |
| DEV-STACK | DEV_WORKFLOW Stacked train の衝突の解消の箇条 | 文の書き直し | doc-consistency | 非対象 |
| D-154 | decision-log（新規） | 本 lane の決定 | — | 非対象 |

## Test Plan

合成の repo の入力は、特に書かない限り main 側が code を含まない（code の拒否が新しい判定を隠さないため）。拒む向きの test は `GateError` が出たことだけでなく、狙った段の止める文の固有の句を assert する（別の段の理由で止まって green になることを防ぐ。例: 集合の外は「集合の外」、承認の無い B の record は `needs --reuse-approval`、L3 の比較はその理由の句）。

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| MG-D13 入口 | 集合の中の手で解いた path（衝突した docs・function-design、衝突していない docs の編集）を使い回しの経路で拒む／集合の外（`src/lib/x.ts`・未知の path・`workflow=true` の docs）を受ける | unit（合成 repo） | T1 `test_hands_resolved_paths_in_set_only` | 集合を C の `NON_DOCS` で作る、衝突していない編集を拒む、集合の判定を外す（M1） |
| MG-D13 入口 TD-706・709 | frontend の test の file（`src/test/**`・`src/**/*.test.ts(x)`・`**` が 0 段の `src/a.test.ts`・`src/routes/-x.test.tsx`）を拒む／近い外れ（`src/testing/x.ts`・`src/lib/test.ts`・`src/a.test.ts.bak`・`src/a.test.js`・`src/routes/foo.test.tsx`）、`env` の path（`src/test/.env.test`）、Rust の test（`src-tauri/tests/*.rs`・`src-tauri/src/**/*.rs`）を受ける | unit | T2 `test_hands_set_frontend_test_files_exact` | pattern を `src/test` の前方一致や `test` の部分一致にする（M6）、`**` を 1 段以上の dir に限る（pattern を Python の `fnmatch` にそのまま渡すと `src/a.test.ts` が外れる）、pattern に当たった path の `env` 等の検査を飛ばす、`src/routes` の `-` の条件を外す |
| MG-D13 入口 TD-724 | lane の commit: packet のある R3 の docs だけの commit（C）が承認なしで引き継がれない／C に当たらない集合の中の commit（function-design の docs・test の file・R4 の lane の docs）が承認なしで引き継がれる、または L3 のやり直しだけで止まる／集合の外の commit（`src/lib/x.ts`）を受ける、または B を案内する | unit | T3 `test_hands_lane_commit_routes` | C に当たらない commit を機械で引き継ぐ（M10）、lane の commit を集合の判定に回さない（M8）、C の commit に承認を求める |
| MG-D13 既定 | 既定の判定（`sync_merge`・`sync_merges`・`reusable`）が lane の手を受ける／`sync_merges` を通さずに `sync_merge` を直接呼ぶ gate で `hands=True` が `AttributeError` になる | unit・CLI | T4 `test_sync_merge_default_stays_strict`（classifier の読取りを合成 base の実 file に patch し、`sync_merges` を通さない gate で `sync_merge` を直接呼ぶ。既定が判定〈`GateError`〉として拒むことと、`hands=True` で例外なく 4 つ組が返り `self.resolved` に path が入ることを assert する）と `plan-approve.test.sh` PA5（足した拒否 2 組。今の正常な同期 1 組の受理と拒否 2 組は変えない） | 既定を True にする（M2）、`self.resolved` に in-place で足す |
| MG-D13 A | lane の手がある版で manual を自動で引き継ぐ（旧 manual に `reuse_approval` が無い組と有る組） | unit | T5 `test_hands_never_auto_carry` | `carry_manual` が `self.resolved`・`self.lane` を見ない（M3）、旧 manual の `reuse_approval` で A を許す（M9） |
| MG-D13 B | B の command で使い回せない／`--reuse-approval` を渡さない record で受ける（旧 manual が `reuse_approval` を持つ組を含む） | unit | T6 `test_hands_reuse_needs_cli_approval` | 承認の判定を結果の approval で行う（M9）、承認の有無を見ない |
| closure | `--review-stage sync`・`docs` が、手で解いた同期か C に当たらない lane の commit のある版で機械の closure を書く | unit | T7 `test_hands_reject_machine_closure` | `sync_closure` に `hands=True` を渡す（M4） |
| 確かめ直し | status が、lane の手があるのに `reuse_approval` の無い manual、または lane の手を含む `sync_from` の closure を通す | unit | T8 `test_nonci_rechecks_hands` | `nonci` の承認の確かめを外す（M5）、`sync_from` の確かめ直しに `hands=True` を渡す |
| R4 TD-710 | R4 の lane の lane の手で B を拒む／`r4` を使い回す | unit | T9 `test_hands_r4_reuses_manual_keeps_fresh_r4` | R4 で lane の手を拒む、または `r4` を新しい head の record で pass のまま残す |
| L3 の比較 | L3 の基準が変わったのに B の record が書かれる: (a) 別の docs を手で解き、Matrix の変更は main から自動の merge で取り込んだ同期（`## L3` の無い旧形式の packet が Matrix を link。今の C5 と同じ形）、(b) function-design の docs の lane の commit と `## L3` 節の変更 | unit | T10 `test_hands_l3_basis_change_stops_before_write` | `record --reuse-from` の L3 の比較を外す（M11）、比較を `self.docs_commits` だけで起こす |
| packet なし TD-711 | packet の無い R2 と R0/R1 の manual の lane の lane の手で、B を拒む・packet を読みに行って止まる／`--reuse-from` だけの明示の A を受ける | unit | T11 `test_hands_without_packet_reuses_with_approval` | L3 の比較を packet の有無で分けずに呼ぶ（M7）、明示の A で lane の手を見ない |
| 止める文と B の案内 | 種を分けて名指さない、0 個の種の行を出す、lane の手が無く本数が 2 の同期（main 側に app の code がある）で今の文を変える、正本を名指す 1 行が無い | unit | T12 `test_hands_guidance_names_kinds_and_points_to_rule` | 3 種を 1 つの行にまとめる、path の列挙を落とす、0 個の種の行を出す、本数が 2 の code だけの組の文を変える、名指す 1 行を落とす、拒む文から「集合の外」か path を落とす |
| 順序 | L3 の基準の変化・lane の手・main 側の code が同時にある版で、承認か code の理由が先に出る、または server に書く | unit | T13 `test_hands_l3_reason_precedes_approval` | `carry_manual`・`record --reuse-from`（`--reuse-approval` の有る組と無い組）・`nonci` のどれかで承認の有無の段を L3 の比較より先に置く（M12）。どの組も L3 の理由の句が出て承認と code の句が出ず、mock server の PATCH/POST が 0 回であることを assert する |
| decision-log | `docs/decision-log.md` の union と違う結果を使い回しの経路で拒む／union と一致する結果を手で解いた path に数える | unit | T14 `test_hands_decision_log_union_mismatch_goes_to_review` | union と違う結果を今のとおり拒む、または union と一致する結果を `self.resolved` に入れる |
| 連鎖 | B の後の code に触れない同期で A と `nonci` が通らない／B の後の新しい lane の手が旧い承認で通る／手で解いた merge の後に足した直す commit が別の規則になる | unit | T15 `test_hands_chain` | `carry_manual` の起点を旧 record の head でなく `source_head` にする、`nonci` が A で引き継いだ `reuse_approval` を認めない、新しい lane の手を旧い承認で通す（M9）、直す commit の後の案内が手で解いた path と lane の commit の path の両方を名指さない |
| MG-D13 B の本数 TD-728・729 | R3 以下の lane で入った変更が docs と frontend の test の file だけの文が 2 本と言う／main 側に集合の外の path がある版か R4 の lane の文が 1 本と言う。組（特に書かない限り R3 の lane）: (a) 手で解いた docs だけ（main 側は docs だけ）→ 1 本、(b) (a) に main 側の `src/lib/x.ts` を 1 つ足す → 2 本、(c) (a) の main 側を frontend の test の file（`src/test/a.test.ts`）と `docs/function-design/90-traceability.md` にする → 1 本、(d) (a) の main 側に gate の文書（`docs/DEV_WORKFLOW.md`）を足す組と `src/lib/bindings.ts` を足す組 → どちらも 2 本、(e) merge の無い、C に当たらない lane の commit（function-design の docs）だけ → 1 本、(f) code のある B を記録した後の、C に当たらない lane の commit だけ → 1 本（起点は旧 record の head）、(g) (a) と同じ入力で R4 の lane → 2 本（TD-729）、(h) (a) と同じ入力で packet の無い R0/R1 の manual の lane → 1 本、(i) lane の手が無く main 側が frontend の test の file だけの同期（今の B）→ `carry_manual` の案内と `record --reuse-from` の拒む文が 1 本 | unit | T16 `test_hands_guidance_closure_count_follows_main_paths` | 本数を 1 本に固定する（M13）か 2 本に固定する、本数の判定に `code_paths`（`CODE_FLAGS`）を使う（(c)・(i) が 2 本、(d) の gate の文書が 1 本になる）、R4 を見ない（(g)）、`90-traceability.md` を集合の外に数える（(c)）、`src/lib/bindings.ts` を生成物として除く（(d)）、起点を `source_head` にして (f) を 2 本にする、本数を言う所ごとに別の値を使う（(i)） |

mutation（実装の run で 1 つずつ注入し、red を確かめて戻す。AC3）: M1 merge の集合の判定を外す → T1。M2 既定を True → T4 と PA5 の (a)。M3 `carry_manual` で lane の手を見ない → T5。M4 `sync_closure` に `hands=True` → T7。M5 `nonci` の承認の確かめを外す → T8。M6 test の file の pattern を `'test' in path` の部分一致にする → T2。M7 L3 の比較の packet の有無の分岐を外す → T11。M8 lane の commit を集合の判定に回さず受ける → T3。M9 承認の判定を結果の approval・旧 manual の `reuse_approval` で行う → T5・T6・T15。M10 C に当たらない集合の中の lane の commit を `self.docs_commits` に入れる → T3。M11 `record --reuse-from` の L3 の比較を外す → T10。M12 承認の有無の段を L3 の比較の前に置く → T13。M13 本数の判定を外して 1 本に固定する（main 側に集合の外の path があっても、R4 の lane でも 1 本と言う、危険な向き）→ T16（(b)・(d)・(g) がどれも red）。

機械の test を置かない契約: 「review が見るもの」の (i)(ii) の合否は B の closure の判断で、helper は判定しない。要る本数の closure が走って合格したことは Coordinator の確かめで、helper は数えない（T16 が固定するのは helper の文の本数だけ）。実装の PR では、merge-evidence の B にその文があり、ほかに言い直しが無いことを doc の差分で確かめる（AC5）。

L3: 置かない。operator の画面・app の build を変えず、helper と規則の文だけを変える。振舞いは合成の git の履歴の上の helper の test で観測でき、Windows/Tauri native でしか観測できない項目が無い（DEV_WORKFLOW L3 Eligibility の (1) を満たさない）。
