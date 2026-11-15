# Plan Packet: 同期で docs だけが衝突した lane の L3 の使い回しを直す

wave に属さない単独の lane。merge の helper（`scripts/pr-gate.py`）の同期だけの条件と manual の使い回し B、その test、正本の文（`docs/agent-guidance/merge-evidence.md` の「base同期だけでheadが変わる場合」、`docs/DEV_WORKFLOW.md` の L3 の表、`docs/decision-log.md`）を変える。製品の runtime（`src/**` の非 test・`src-tauri/**`）・CI の job・classifier は変えない。

出所: owner の決定 TD-704（repo 外の回答台帳、2026-11-14）「同期で docs だけが衝突した lane が L3 を使い回せずにやり直しになるのは理屈に合わないので、規則を直す。今後も必ず同じ状況になる」。owner は PR #123（2026-09-29）で manual の再利用の条件を緩めず、D-144（2026-10-27）で生成物と `docs/decision-log.md` の union だけを受けた。TD-704 はそれより後の決定で、残っていた backlog の観測の項目（「衝突を解いた版の manual の再利用（観測）」）に owner が答えたもの。手で解いてよい path の範囲は owner の決定 TD-706（同じ台帳、2026-11-14。下の「Open Questions」の決定の記録）で (b) に決まり、残りの問いは TD-709〜TD-713（同じ台帳、2026-11-14、owner の「全部推奨」）で決まり、計画の review round 1 の後に test の file の根拠を TD-714（同じ台帳、2026-11-15）で直し、受ける違いの範囲を TD-715（同じ台帳、2026-11-15）で衝突していない手の編集まで広げた。起点の事実は下の「調べ」。

decision-log の番号: 本 lane は D-154 を予約する。2026-11-14 に `git fetch origin` の後で引き直した: `git grep -n "^## D-15" origin/main -- docs/decision-log.md` の最後は D-152（`a90b0d6`）、`git grep -n -E "D-15[3-9]" <open の 7 本の origin/agent/* branch> -- docs/decision-log.md docs/plans` の当たりは PR #250（`agent/window-close-guard`）の D-153 だけで、D-154 以降は無い。

安全の境界: この変更は `AGENTS.md`「継ぎ足さない」と `docs/DEV_WORKFLOW.md` Workflow State（D-098）が挙げる安全の境界のうち「古い証拠の拒否」を緩める向きに変える。改訂は通常の packet・PR の経路で行い（本 packet）、owner の決定の日付は TD-704・TD-706・TD-709〜TD-713（2026-11-14）と TD-714・TD-715（2026-11-15）。何を新たに受け、何を今までどおり拒むかを下の「境界ごとの受けると拒む」に書く。

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

- L3 の PASS の後に main と同期し、その merge の結果が自動の merge（`git -c merge.directoryRenames=false merge-tree --write-tree FIRST SECOND`）と違う lane が、次の 2 つを満たすとき、L3 をやり直さずに、同期だけを見る closure の 2 本（座組表の Final Reviewer の Claude 側と Codex）の判定と owner の 1 行の承認で、manual を使い回せる（今の B と同じ `record --kind manual --outcome pass --evidence <旧 evidence> --reuse-from SOURCE --reuse-approval POINTER`）。違う path は 2 種で、`merge-tree --name-only` が衝突と出した path の手の解消（以下「衝突した path」）と、衝突していない path の自動の merge との違い（以下「衝突していない手の編集」。TD-715 で受ける）。生成物と、結果が union と一致する `docs/decision-log.md` は今のとおり数えない。(1) 違う path が 2 種とも全部「手で解いてよい path」（下の「境界ごとの受けると拒む」。TD-706 の (b): docs〈L3 の比較対象と計画の節に触れない所〉と frontend の test の file〈`src/test/**`・`src/**/*.test.ts`・`src/**/*.test.tsx`。`src/routes/**` の中は basename が `-` で始まるものだけ〉。Rust の test は TD-709 で含めない）。(2) 手で解いた path に active な packet と `l3_basis` が link する Matrix が無く、packet のある lane では L3 の比較対象が `source_head` と今の head で byte で同じ。closure の判定の項目: 衝突した path では手の解消が両側の変更を落としていないこと。衝突していない手の編集は、同期で取り込んだ main 側の変更に合わせる編集（辻褄合わせ）で、承認済みの Scope の中なら合格。lane の新しい変更（Scope の外、または同期と関係のない変更）なら closure は不合格とし、merge から外して lane の 1 親の commit に分け、通常の経路（C・監査の closure・Gated Amendment・L3 のやり直しのいずれか）に回す（round 2 #1。機械は目的を判定せず、この基準は closure が持つ）。合格の編集も L3 の項目に影響しないこと。frontend の test の file では test を弱めていないことと、Tailwind の scanner を経た生成 CSS への影響が L3 の項目の合否を変えないこと（TD-714・D-146）。
- helper は受ける側でも、出力と B の案内で「衝突として手で解いた path」と「衝突していない手の編集」を分けて名指す（closure の 2 本が何を見るかを取り違えないため）。使い回せないとき、helper は今と同じく manual 以外の record を書かずに止め、外れた理由を種類で分けて名指す（集合の外の違いの path〈衝突した path と衝突していない手の編集を分けて〉・手で解いた packet と Matrix の path・L3 の比較対象の変化・code に触れる main 側の path）。次の一手は、(1)(2) のどちらかで外れたら L3 のやり直しだけ、満たせば B の完全な command か L3 のやり直し。
- helper の `status`・`ready`・`merge` は、`source_head` から今の head まで同じ条件を確かめ直し、手で解いた path があるのに `reuse_approval` の無い manual と、`sync_from` の形の closure を拒む。

### 失敗定義

- 集合の外の違い（衝突した path でも衝突していない手の編集でも）を、手で解いた path として受けてしまう。
- 衝突していない手の編集を、helper の出力と B の案内で衝突の解消と区別せずに出し、closure が「何のための編集か」を見ずに通す形になる。または、lane の新しい変更（承認済みの Scope の外、または同期と関係のない変更。例: 同期と無関係な `docs/function-design/99-unreviewed.md` を merge に足す〈Codex round 2 の隔離試作〉）を merge に混ぜて、B で L3 を使い回せる基準になる（closure の合否の基準が無い）。
- code（classifier の `rust`・`frontend`・`env`・`unknown`。frontend の test の file の `frontend` を除く。`src-tauri/**` の Rust の test〈`#[cfg(test)]` の module を含む本体の file と分かれた dir〉は code のまま）・生成物の手書き（生成物は今のとおり再生成）・gate の文書（`workflow=true`）・active な packet と Matrix・packet の L3 の比較対象（`l3_basis`）・計画の節（`plan_sections`）に触れる手の解消を受けてしまう。
- main から自動の merge で取り込んだ Matrix の変化（手で解いていない path）で L3 の比較対象が変わったのに、B の record（`record --reuse-from`）が書かれる。
- 手で解いた同期で、manual を人の closure と owner の承認なしに引き継ぐ（自動の A）、または review の closure を helper の記録（`--review-stage sync`・`docs`）で代える。偽の衝突の解消（片側を落とす）を機械が通す経路ができる。
- `scripts/plan-approve.sh` が手で解いた merge を「main の同期」として受けるようになる（計画の review の後に reviewer の読まない編集が merge に紛れる）。
- R4 の `r4` の承認を使い回す、または TD-713 の 1 回の承認に `r4` を束ねる。
- 旧 manual が前の B の `reuse_approval` を持つとき、その承認を新しく手で解いた同期の承認として使う。

### 非目的

- lane の 1 親の commit の扱い（C）を広げること（`traceability` の docs・R4 を C に入れない）。B が受ける衝突していない手の編集は同期の辻褄合わせに限り、lane の新しい変更は merge から外して 1 親の commit に分けて今の C・監査の closure・Gated Amendment・L3 のやり直しの条件で裁くので、C の受ける範囲は変わらない（round 2 #1。同期と関係のない lane の新しい docs の変更を L3 のやり直しなしに受けることは C の拡張で、本 lane の外。owner に別に確かめる）。
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

`src/test/**` と `src/**/*.test.ts(x)` は app の JS の runtime module に入らない: `package.json` の `build` は `vite build`（entry から import を辿る）で、`grep -rlE` で test の file でも `src/test/` でもない `src/**` の file から `src/test/…` か `*.test` を import する所は 0 件（Codex round 1 でも runtime の import と `import.meta.glob` は 0 件）。ただし app の build に全く入らないわけではない: `vite.config.ts:12` の Tailwind plugin の scanner は `src/styles/globals.css` に test の source の除外が無く、test の file の class を拾って生成 CSS に入れうる（Codex round 1 F2 の隔離実験: lock と同じ Tailwind 4.2.2 で、import されない `src/test/example.test.tsx` に `w-[173px]` を足すと生成 CSS に `width: 173px` が入った）。そこで根拠を「JS の runtime module に入らない」とし、CSS への影響は B の closure 2 本が見る（TD-714、D-146 の観点）。`tsconfig.json` の `include` は `src` と `src/test` を含むので、`tsc --noEmit`（typecheck）と vitest には入り、hosted の frontend の job が新しい head で確かめる。

Rust の test（2026-11-14、本 run で実測）: `rg -l "#\[cfg\(test\)\]" src-tauri/src --type rust --stats` は 102 file 中 69 file（127 箇所）で、多くは本体の file の中の module。ほかに分かれた dir が 3 つある: `src-tauri/tests`（`architecture_test.rs`・`design_compliance_test.rs` ほか）、`src-tauri/dev-tools/tests`、`src-tauri/src/biz/csv_import_service/tests`（`mod.rs:15` の `mod tests;` で本体の crate の module に入る）。TD-706 は Rust の test を全部入れない。分かれた dir の file を path で受けることはできるが、TD-709（2026-11-14）で決定どおり外したままにし、本 lane の集合に入れない（必要になれば D-154 の Revisit で見直す）。

route の生成器（2026-11-15、round 1 の是正で実測）: `vite.config.ts:12` の `tanstackRouter()` は `tsr.config.json`（`target`・`autoCodeSplitting` だけ）で `routeFileIgnorePrefix` を指定せず、既定の `-` で始まる file だけを route tree から外す。`src/routes/**` の今の test の file は `src/routes/-stocktake-route.test.tsx` と `src/routes/settings/-logs.test.ts` の 2 つで、どちらも `-` で始まる。`-` で始まらない `src/routes/**/*.test.tsx` は route として生成され build に入る（Opus round 1 P3-3）ので、集合に入れない。

今の規則で 4 本は全部、同期だけの条件の merge の要件（merge-evidence の「同期だけの条件」の (a)）を満たさない（生成物と union の外の衝突）。B も同期だけの条件を要るので使えず、L3 のやり直しになる。C は 1 親の lane の commit だけを許し、merge の中の違いには効かない。

## 案の比較

| 案 | 中身 | 採否と理由 |
|---|---|---|
| 1（採る） | 同期だけの条件の merge の判定（`sync_merge`）に、使い回しの経路だけが頼む `resolve=True` を足す。自動の merge と違う path のうち「手で解いてよい path」の集合の中のものを、衝突した path（衝突の解消）と衝突していない手の編集に分けて、拒まず「手で解いた path」として返す（TD-715）。集合の外の違いは今のとおり拒む。手で解いた path が 1 つでもあれば、取り込んだ main 側が code に触れるかに関わらず A にせず B に回す（closure 2 本と owner の承認）。review の closure も監査の closure を要る | 今の B の道（`--reuse-approval`・closure 2 本・owner の 1 行）をそのまま使い、RecordV1 も記録の手順も変えない。機械が判定できるのは「どの path を手で解いたか」までで、両側を保ったかは人が見る、という線を 1 か所で引ける。4 本はどれも main 側の code を取り込むので、今でも B の closure と owner の承認が要る形で、手の解消をその closure の範囲に足すだけになる |
| 1-(a)（見送り、TD-706） | 案 1 の手で解いてよい path を docs だけにし、frontend の test の file を入れない | owner が (b) を選んだ（TD-706）。(a) では #228（`DailyReportImportPage.sd.test.tsx`）と #216（`src/test/page-root-pageshell-sweep.test.ts`）が L3 のやり直しのままで、test の file は app の JS の runtime module に入らず hosted の typecheck・test が新しい head で確かめるので、docs と分ける理由が弱い。test を消す・skip する・弱める解消と、Tailwind の scanner を経た CSS への影響は closure の判定の項目で見る（TD-714） |
| 8（見送り、TD-714） | Tailwind の `@source not` で test の file を CSS の scanner から外し、test の file を app の build から切り離す | app の build の設定（`src/styles/globals.css`）を変え、生成 CSS の差の確かめが要って lane の範囲が helper と規則の文の外へ広がる。CSS への影響は B の closure 2 本が見れば足りる（TD-714） |
| 2 | C（lane の docs だけの commit）を merge の中の docs の違いにも広げ、helper が機械で受ける | 片側を落とす解消を機械は見分けられず（失敗定義の 2 つ目）、人の closure が入らない。C は `traceability` の docs（function-design）を受けないので、#236・#228 の衝突に効かない |
| 3 | `docs/decision-log.md` の union の解き方を `docs/backlog.md`・`docs/Plans.md` 等にも広げる | #227 の測定で、衝突は同じ項目の書き換えどうしで、union は旧い行と新しい行を両方残す（誤った結果を機械が受ける）。末尾の追記どうしでない衝突には使えない |
| 4 | 手の解消の結果に両側の追加の行が全部残るかを機械で確かめ、満たせば A | 同じ行を両側が書き換えた正しい解消（第 3 の行）は両側の行を含まないので、正しい解消を拒み、行を残しただけの誤った解消を受ける。正規化した差分の同値を根拠にしない MG-D13 の方針にも反する |
| 5 | closure を 1 本（Claude 側だけ）にする | B と別の本数の経路ができ、helper の分岐と規則の文が増える。失敗定義の中心（片側を落とす）は reviewer の見落としが要で、独立の 2 本の B と同じ強さを保つ |
| 6 | lane を新しい main へ cherry-pick・rebase で切り直す | head の SHA と Plan Commit の祖先関係が変わり、旧形式の packet は新形式と判定される（DEV_WORKFLOW Wave Operation）。L3 の版も変わるので使い回しにならない |
| 7 | 今のまま L3 をやり直す | owner の決定 TD-704 で採らない |

## 境界ごとの受けると拒む

言葉: 「衝突した path」は、各 sync merge の `git -c merge.directoryRenames=false merge-tree --write-tree --name-only -z FIRST SECOND` の出力の tree の後ろに並ぶ path（今の `sync_merge` の `fields[1:]`）。「違う path」は、自動の merge の tree と merge の commit の `git diff --name-only --no-renames` の path と衝突した path の和（今の `paths`）。「衝突していない手の編集」は、違う path のうち衝突した path でないもの（自動の merge は衝突なく済んだのに、merge の commit の結果がそれと違う path）。「手で解いた path」は、違う path のうち、生成物でも、union と一致する `docs/decision-log.md` でもなく、「手で解いてよい path」に当たるもので、衝突した path と衝突していない手の編集の 2 種を分けて持つ（TD-715 で両方を受ける）。

| 境界 | 新たに受ける | 今までどおり拒む |
|---|---|---|
| 同期だけの条件の merge（`sync_merge`、`resolve=True` の時だけ） | 手で解いてよい path の、衝突した path の手の解消と、衝突していない手の編集（TD-715）。helper は 2 種を分けて「手で解いた path」として返す。helper が受けるのは path の集合の判定までで、使い回しの合否は B の closure が決める: 衝突していない手の編集は同期で取り込んだ main 側の変更に合わせる編集（辻褄合わせ）で承認済みの Scope の中なら合格、lane の新しい変更（Scope の外、または同期と関係のない変更）なら不合格で、merge から外して lane の 1 親の commit に分け、通常の経路（C・監査の closure・Gated Amendment・L3 のやり直しのいずれか）に回す（round 2 #1。機械の分岐は足さない） | 集合の外の違い（衝突した path でも衝突していない手の編集でも。code・Rust の test・`workflow=true` の docs・`env`・`generated` の手書き・未知の path・pattern に当たらない `src/**`）。second parent が main の外・merge-base が一意でない・最後の merge が今の base でない、の条件はそのまま。rename は `--no-renames` で両側の path を見て、片側でも集合の外なら拒む（2 種の区分は path ごとに付ける） |
| 手で解いてよい path の集合（TD-706 の (b)・TD-709・TD-712） | current main 側の classifier で `rust`・`env`・`generated`・`workflow`・`unknown` が全部 false の path のうち、(1) docs: `docs` が true で `frontend` が false（`traceability`・`rust_drift` は true でよい。hosted の drift の検査が新しい head で走る）、または (2) frontend の test の file: helper の定数の pattern（`src/test/**`・`src/**/*.test.ts`・`src/**/*.test.tsx`。`src/routes/**` の中は basename が `-` で始まるものだけ）に当たる path。pattern は `frontend`・`traceability`・`rust_drift` を外すためだけに使い、ほかの flag の検査を飛ばさない（例 `src/test/.env.test` は `env` で拒む）。test の file は JS の runtime module に入らないが、Tailwind の scanner を経て生成 CSS に入りうるので、CSS への影響は B の closure 2 本が見る（TD-714。機械では判定しない） | pattern の近い外れ（`src/testing/**`・`src/lib/test.ts`・`*.test.ts.bak`・`*.test.js`・`src/routes/foo.test.tsx`）。Rust の test（`src-tauri/**` の全部。本体の file の `#[cfg(test)]` の module も、`src-tauri/tests/**` 等の分かれた dir も。TD-709）。gate の文書（TD-712） |
| 生成物 | — | 変えない（衝突してよく、手で解かず再生成。手で解いた path に数えない。D-098） |
| decision-log | `docs/decision-log.md` の結果が union とも自動の merge とも違うとき、拒まず手で解いた path として B に回す（衝突していれば衝突した path、衝突していなければ衝突していない手の編集） | 結果が union と一致すれば今のとおり機械で受け、手で解いた path に数えない |
| active な packet・Matrix（D-145 (6)） | — | packet のある lane で、手で解いた path に `--packet` の path があれば止める（Workflow State などの L3 の比較対象の外の変化を塞ぐ）。`## L3` の無い旧形式の packet（`l3_basis` が Test Plan の Matrix の link を読むのはこの形だけ。`## L3` のある packet と新しい形の packet は Matrix を読まない）では、`source_head` と今の head のどちらかの版の `l3_basis` が link する Matrix の path があっても止める。`carry_manual`・`record --reuse-from`・`nonci` の共通の確かめ |
| L3 の比較対象（`l3_basis`） | — | packet のある lane で、手で解いた path か lane の docs だけの commit があれば、`source_head` と今の head の L3 の比較対象が byte で同じことを求める（今の `l3_change`）。手で解いていない Matrix を main から自動の merge で取り込んだ変化もここで止める。`carry_manual`・`record --reuse-from`・`nonci` の 3 か所で同じ条件。packet の無い lane（TD-711）は `l3_change` を呼ばない |
| 計画の節（`plan_sections`） | — | 今のとおり、承認の最新の snapshot と byte で同じでなければ `plan_drift` が status・Ready・merge で止める（本 lane で新しい検査を足さない） |
| code | — | 衝突した path の code の手の解消は同期だけの条件を満たさず、L3 のやり直しだけを案内する（今のとおり）。取り込んだ main 側の code は今の code に触れない条件と B のまま |
| manual の自動の引継ぎ A | — | 手で解いた path があれば、main 側が code に触れなくても、旧 manual が前の B の `reuse_approval` を持っていても A にしない。`carry_manual` は B の完全な command と L3 のやり直しの command を示して何も書かずに止まる |
| B の record（`record --reuse-from`） | `--reuse-approval` をこの record で渡したとき、手で解いた path を受ける（R3・R4・packet の無い R0〜R2） | `--reuse-approval` を渡さない record（R0/R1 の明示の A）は、旧 manual の `reuse_approval` を引き継げる場合でも、手で解いた path があれば拒む。判定はこの record の CLI の引数で行い、結果に入る approval（引き継いだものを含む）で行わない |
| review の closure | — | `sync_closure` は `resolve=False` のまま呼ぶので、手で解いた同期は今の文（自動の merge と違う）で拒まれ、監査の closure を案内する（B の 2 本が兼ねる） |
| status・Ready・merge の確かめ直し（`nonci`） | `reuse_approval` のある manual は、手で解いた path があっても通す（同期の連鎖で後の A が引き継いだ `reuse_approval` を含む） | 手で解いた path があるのに `reuse_approval` の無い manual。`sync_from` の形の closure は `resolve=False` で確かめ直すので、手で解いた merge を含めば拒む |
| `scripts/plan-approve.sh` | — | `sync_merge` を既定（`resolve=False`）で呼び、手で解いた merge も、衝突していない path の編集を含む merge も今のとおり「not a sync of main」で拒む |
| R4（TD-710） | 手で解いた同期での B の manual の使い回し（今の B が R4 の code に触れる同期に効くのと同じ） | `r4` の承認は使い回さず、TD-713 の 1 回の承認にも束ねない（新しい head での明示の `record --kind r4`）。C（`docs_route`）は今のとおり R4 に使わない |
| packet の無い R0〜R2（TD-711） | 手で解いた同期での B の manual の使い回し。closure の 2 本の証跡を `reuse_approval` の pointer の先に置く（review の要らない R0/R1 の manual の lane）か、review の closure の evidence に並べる（R2） | `l3_change` と packet・Matrix の確かめは packet が無いので行わない（L3 の基準は PR body にあり、tree の手の解消で変わらない。今の B と同じ）。`--reuse-from` だけの明示の A は手で解いた同期で拒む |
| Ready の承認（TD-713） | 手で解いた同期の Ready の承認を、B の owner の 1 行の承認と同じ 1 回に束ねる | helper は手で解いた同期を「同期だけ」と判定しない（`--review-stage sync` が通らない）ので、AGENTS.md の「同期だけと判定した同期を経た head」に入らず、承認は取り直す。AGENTS.md は変えない |

境界の 4 観点（round 1 の是正、TD-715 の差し替え、round 2 の是正〈#1 の closure の基準・#2 の順序・#7 の表し方〉で一巡した）:

- 数の範囲: 手で解いた path は 2 種とも 0 個以上。両方 0 個なら判定・文・記録は今と同じ（新しい文を出さない）。同期の merge は 1 回以上で、`sync_merges` は起点から head までの全部の merge の手で解いた path の和を `self.resolved`（種と path の組の集合。Scope の表し方）に集める。同じ path が merge によって別の種になれば両方の組が入り、両方の行に出る。1 つの merge でもどちらかの種が 1 個以上あれば A にしない。数に上限は置かず、文は種類ごとに全部の path を並べる（今の文と同じく sorted、`, ` 区切り）。closure の基準（round 2 #1）は衝突していない手の編集の path ごとに当て、1 つでも lane の新しい変更なら closure は不合格（その path を merge から外せば残りを B で裁き直す）。
- 0 の表示: 手で解いた path が無い同期の止める文と B の案内は今の文のまま。B の案内と carry の止める文には、1 個以上ある種だけを「衝突として手で解いた path: …」「衝突していない手の編集: …」の別の行で出し、0 個の種の行は出さない。closure の基準の文は「衝突していない手の編集」の行にだけ添えるので、その種が 0 個なら出ない。拒むときの「集合の外の違い: …」も 2 種を分けて書き、0 個の側は出さない。`self.resolved` が空のときの真偽は「手で解いた path が無い」と一致する（class の既定 `()` も、`sync_merges` が置く空の集合も偽）。
- 失敗の時点と順序: どれも書込みの前（`record` は server への PATCH/POST の前）に止まる。`sync_merges` は head から first parent を辿り、最初に外れた merge で止まる。merge の中の順は、main の祖先 → merge-base の一意 → `merge-tree`（終了値 0/1 の外は判定不能 exit 2）→ decision-log の union（一致すれば数えない）→ 集合の外の違い（衝突した path と衝突していない手の編集をまとめて見て、外れた path を全部出す）。`sync_merges` の後は 3 つの経路とも「共通の確かめ（packet・Matrix の path → `l3_change`）→ 承認の有無 → code に触れる main 側の path」の順（round 2 #2）。承認の有無と code の段は経路ごとに: `carry_manual` は、`self.resolved` があれば B と L3 のやり直しの command を示して止まり、無くて code があれば今の B の案内で止まる。`record --reuse-from` は、`self.resolved` があって `--reuse-approval` が無ければ拒み、次に `--reuse-approval` が無くて code があれば今の文で拒む。`nonci` は、`self.resolved` があって manual に `reuse_approval` が無ければ拒み、次に `reuse_approval` が無くて code があれば今の文で拒む。前の段で止まれば後の段の理由は出さない（L3 の変更と main 側の code が同時にあれば L3 の理由だけが出る。T18）。closure の基準での不合格は人の判定で、helper の順序の外（B の record を書く前に owner の承認が出ない）。
- 種類の混在: 1 つの merge に衝突した path と衝突していない手の編集が混ざれば、両方が集合の中なら B（2 種を分けて名指す）、どちらかに集合の外があれば集合の外で止める（L3 のやり直しだけ）。衝突していない手の編集に辻褄合わせと lane の新しい変更が混ざれば、closure は不合格で、lane の新しい変更だけを 1 親の commit に分ける。手で解いた merge と自動の merge が列に混ざれば B。手で解いた path と lane の docs だけの commit（C）が混ざれば、両方の条件（C の docs だけ・手で解いた path の集合・L3 の比較）を求めて B。手で解いた path と生成物の再生成・union と一致する decision-log が混ざれば、後の 2 つは今のとおり数えない。

## Scope

- `scripts/pr-gate.py`（計画の固定の後の実装の run）:
  - `resolve` の渡し方（round 1 #6）: `sync_merge(commit, first, second, base, resolve=False)`・`sync_merges(origin, head, base, docs=False, resolve=False)`・`reusable(origin, head, base, code=True, docs=False, resolve=False)` を既定にし、`True` を渡すのは使い回しの経路の 3 か所だけ: `carry_manual`、`record --reuse-from`、`nonci` の manual の `source_head` からの確かめ直し。`plan-approve.sh` の `sync_merge`、`sync_closure`、`nonci` の `sync_from` の closure の確かめ直しは既定（`False`）のまま。呼出し側は `rg -n "sync_merges\(|reusable\(|sync_merge\(" scripts` で列挙し直す。
  - `sync_merge(..., resolve=True)`: 今の `paths`（違う path）から生成物を除き、`docs/decision-log.md` は union と一致すれば除く。残りを全部（衝突した path も衝突していない手の編集も。TD-715）手で解いてよい path の判定に回し、集合の外が 1 つでもあれば拒む（「集合の外の違い」と名指し、衝突した path と衝突していない手の編集を分けて並べる）。全部が集合の中なら、`fields[1:]` に入る path を衝突した path、入らない path を衝突していない手の編集として、種を付けて `self.resolved` に足す（戻り値の 4 つ組 `(commit, first, second, merge_base)` は変えない。`plan-approve.sh` と `code_paths` が位置で受けているため）。
  - `self.resolved` の表し方（round 2 #7）: 1 つの属性に、`('conflict', path)` と `('edit', path)` の組の集合で持つ。2 種の list の tuple（`([], [])`）にしない（tuple は中が空でも真で、`if self.resolved:` が常に真になる）。空の判定は `if self.resolved:` の真偽だけで行い、class の既定 `()` と `sync_merges` が置く `set()` はどちらも偽。文に出すときに種で分けて path を sorted に並べる。
  - 手で解いてよい path の判定（round 1 #8）: path ごとに current main 側の classifier（`self.contents('scripts/ci/classify-changes.sh', base)`）を通し、`rust`・`env`・`generated`・`workflow`・`unknown` が全部 false を求める。そのうえで、定数の pattern（例 `TEST_FILES`）に当たれば `frontend`・`traceability`・`rust_drift` を問わず受け、当たらなければ `docs` が true で `frontend` が false を求める。pattern は `src/test/**`・`src/**/*.test.ts`・`src/**/*.test.tsx` で、`src/routes/**` の中は basename が `-` で始まるものだけ。定数は `GENERATED` の近くに置き、comment に「frontend の test の file（TD-706）。classifier から解析して得ない。Rust の test は入れない（TD-709）」と書く。`ponytail:` comment に限界（「JS の runtime module に入らない」は route の生成器の `routeFileIgnorePrefix` の既定 `-` と、app の code が test の file を import しないことに依る。Tailwind の scanner は test の file の class を生成 CSS に入れうるので、CSS への影響は機械で見ず B の closure が見る。TD-714）と見直す条件（`tsr.config.json` が ignore の接頭辞・route の dir を変えたとき、app の code から test の file への import が出たとき、CSS の scanner の source の設定が変わったとき、Rust の分かれた test の dir を入れる決定が出たとき）を書く。
  - `resolve=False` の `sync_merge` は今の処理だけを通り、classifier も `self.resolved` も触らない（`plan-approve.sh:98` は `object.__new__(g.Gate)` で `__init__` を通らずに `sync_merge` を呼ぶ）。`resolved = ()` を `docs_commits = ()` の隣に class 属性の既定として置く。
  - `sync_merges`: 呼ぶたびに `self.resolved` を `set()` にしてから（`self.docs_commits` と同じ置き方）、列の全部の merge の手で解いた path の和を集める。
  - 共通の確かめ（round 1 #3・#5。`carry_manual`・`record --reuse-from`・`nonci` の 3 か所が同じ関数を呼ぶ。例 `resolved_change(source, head)`）: packet がある lane で、(i) `self.resolved` に `--packet` の path か、`source` と `head` のどちらかの版で `l3_basis` が link する Matrix の path（`## L3` の無い旧形式の packet だけが持つ）があれば理由を返す、(ii) `self.docs_commits` か `self.resolved` があれば `l3_change(source, head)` の理由を返す。packet の無い lane（TD-711）は (i)(ii) とも行わず `None`（`l3_change` を呼ばない）。今の `self.docs_commits and self.l3_change(...)` の 3 か所はこの関数に置き換える。
  - 3 か所の順序（round 2 #2）: どれも `reusable(..., code=True, resolve=True)` で code の path の list を受け取り（`sync_merges` の判定はここで済む）、その後を「共通の確かめ → 承認の有無 → code の require」の順に並べ替える。今の `record --reuse-from`（`pr-gate.py:948-952`）と `nonci`（`pr-gate.py:789-792`）は code の require が `l3_change` より先なので、code の require を後ろへ動かす。code の require は `reusable` を `code=True` で呼んでから、承認の無いときだけ当てる（今の `code=args.reuse_approval is None`・`code='reuse_approval' not in manual` の引数の分岐は、require の条件に移す）。`carry_manual` は今も共通の確かめが code の B の案内より先なので、承認の有無の段（`self.resolved` があれば B を示して止まる）をその間に入れる。各段の止め方は境界の 4 観点の「失敗の時点と順序」のとおり。
  - `carry_manual`: `resolve=True` で `reusable` を呼び、共通の確かめで止める。`self.resolved` があれば、main 側が code に触れなくても、旧 manual が `reuse_approval` を持っていても、B の完全な command と L3 のやり直しの command を示して何も書かずに止まる（理由の文に「衝突として手で解いた path: …」と「衝突していない手の編集: …」を、1 個以上ある種だけ別の行で）。R4（TD-710）でも同じく B を示し、`r4` は今のとおり引き継がない。
  - `record --reuse-from`: `resolve=True` で `reusable` を呼び、共通の確かめで止める（Codex round 1 F3 の反例: 別の docs を手で解き、Matrix の変更は main から自動の merge で取り込む → 書込みの前に拒む）。次に、`self.resolved` があるとき、`args.reuse_approval` が無ければ拒む（判定は CLI の引数で行い、結果に入る `approval`〈旧 manual から引き継ぐものを含む〉で行わない。round 1 #4。止める文に `needs --reuse-approval` の句を入れる）。最後に、`--reuse-approval` が無くて code があれば今の文で拒む。R3・R4・packet の無い R0〜R2 のどれでも同じ。
  - `nonci`: manual の `source_head` からの確かめ直しは `resolve=True` で、共通の確かめ → 手で解いた path があって `reuse_approval` が無ければ拒む → `reuse_approval` が無くて code があれば拒む、の順。`sync_from` の closure の確かめ直しは `resolve=False` のまま（手で解いた merge を含めば今の文で拒む）。
  - `sync_closure`: 変えない（`resolve=False` の既定で、手で解いた同期を今の文で拒み、監査の closure を案内する）。
  - 止める文と B の案内（round 1 #1、TD-715 で差し替え）: 拒む側は「集合の外の違い: …」（衝突した path と衝突していない手の編集を分けて。L3 のやり直しだけ）と「手で解いた packet・Matrix: …」（L3 のやり直しだけ）。受ける側は B の command と L3 のやり直しの command に「衝突として手で解いた path: …」と「衝突していない手の編集: …」を別の行で添える（closure の 2 本の発注にそのまま写せる形）。「衝突していない手の編集」の行には合否の基準を 1 文で添える（round 2 #1）: 「衝突していない手の編集は、同期で取り込んだ main 側の変更に合わせる編集（辻褄合わせ）で、承認済みの Scope の中なら合格。lane の新しい変更（Scope の外、または同期と関係のない変更）なら closure は不合格とし、merge から外して lane の 1 親の commit に分け、通常の経路（C・監査の closure・Gated Amendment・L3 のやり直しのいずれか）に回す」。helper はこの基準で分岐しない（文を出すだけ）。
- `scripts/tests/pr-gate.test.py`: 下の Test Plan の test を `SyncOnly`・`SyncRecords`・`PacketReuse` に足す。今の `test_sync_only_rejects_manual_conflict_resolution`・`test_sync_only_rejects_edit_inside_merge`・`test_sync_only_decision_log_union_conflict` の拒む向きは既定（`resolve=False`）の判定として残す（弱めない）。
- `scripts/tests/plan-approve.test.sh`（round 1 #7）: PA5 に 2 組を足す。(a) 手で解いてよい種類の path（例 `docs/x.md`）を lane と main の両方が変えて衝突させ、手で解いた merge、(b) 衝突していない `docs/x.md` を merge の中で編集した merge。どちらも「not a sync of main」で拒まれる。今の `hidden.txt` の組は classifier で `unknown` になり、新しい判定でも拒まれるので既定の厳しさを見分けない（Opus round 1 P3-2）。(a) が `resolve` の既定を見分ける。
- `docs/agent-guidance/merge-evidence.md`: MG-D13 の行、「同期だけの条件」の (a) の違ってよい path の文（使い回しの経路では、手で解いてよい path の中なら衝突した path も衝突していない手の編集も違ってよい。TD-715）、「manualの使い回し」の B（判定の項目に「衝突した path では手の解消が両側の変更を落としていない。衝突していない手の編集は、同期で取り込んだ main 側の変更に合わせる編集（辻褄合わせ）で、承認済みの Scope の中なら合格。lane の新しい変更（Scope の外、または同期と関係のない変更）なら closure は不合格とし、merge から外して lane の 1 親の commit に分け、通常の経路（C・監査の closure・Gated Amendment・L3 のやり直しのいずれか）に回す〈TD-715・round 2 #1〉。合格の編集も L3 の項目に影響しない。test の file では test を消す・skip する・弱めることをしていない。test の file の class が Tailwind の scanner を経て生成 CSS に入る変化が L3 の項目の合否を変えない〈D-146 の観点〉」を足す。TD-714。round 1 #11 の 1 文「TD-713 で束ねるのは B の承認と Ready の承認で、`r4` は束ねない〈新しい head での明示の `record --kind r4`〉」を足す）・使い回せないときの次の一手（種類ごと）、「reviewのclosure」、「Helperの境界」の `--reuse-from` の文、「使い回しの元は…確かめ直し」の段落（`resolve` の経路と共通の確かめ）。L3 の使い回し C の節は変えない。長い段落は段落ごと書き直す（継ぎ足さない）。
- `docs/DEV_WORKFLOW.md`（round 1 #9）:
  - L3 の表: B の行の後ろに「同期の merge で衝突の解消か手の編集をした（手で解いてよい path の中だけ）（B）| 同期の closure 2 本と owner の 1 行の承認で使い回す | Final Reviewer 2 本・owner」の行を足し、「取り直す」の行を「lane の commit（C の外）・同期の merge の集合の外の違い（runtime・Rust の test・gate の文書・未知の path）・手で解いた packet と Matrix・生成物の手書き、`## L3` 節の変更」に書き直す。
  - Stacked train の 3 つ目の箇条（:157「衝突の解消が実装の file に及んだら、独立の Final Reviewer が差分を確かめ直す（docs だけの解消は差分の確認だけ）」）を、「同期の merge で衝突を解くか手で編集した版は、path に関わらず監査の closure を要る（helper の記録の closure にならない）。manual のある lane は merge-evidence の B（D-154）で L3 を使い回すか取り直す」の形に書き直す。
  - Wave Operation の :149「衝突の解消が内容を変えたら通常の再検証と現在版の closure を行う」は今も正しいので残す。
- `docs/decision-log.md`: 末尾に D-154（TD-704・TD-706・TD-709〜TD-715、案の比較〈(a) と Tailwind の `@source not` を見送った理由を含む〉、境界〈手で解いてよい path の中なら衝突の解消も衝突していない手の編集も受け、2 種を分けて名指して closure が見る〈TD-715〉、衝突していない手の編集の合否の基準〈同期で取り込んだ main 側の変更に合わせる編集（辻褄合わせ）で承認済みの Scope の中なら合格、lane の新しい変更（Scope の外、または同期と関係のない変更）なら closure は不合格とし、merge から外して lane の 1 親の commit に分け、通常の経路（C・監査の closure・Gated Amendment・L3 のやり直しのいずれか）に回す。C は広げない。round 2 #1〉、集合の外の違いは拒む、共通の確かめ → 承認の有無 → code の順〈round 2 #2〉、R4 に開き `r4` は取り直して束ねない、packet の無い R0〜R2 に開く、gate の文書は集合の外、Ready の承認は B の承認と 1 回に束ねる〉、Guarantee range: 衝突の手の解消の正しさと、衝突していない手の編集の目的と L3 の項目への無影響は closure の判断で機械の保証でない〈TD-715〉。frontend の test の file は JS の runtime module に入らないが、Tailwind の scanner を経て生成 CSS に入りうる。その影響は B の closure 2 本の判断（D-146）で、機械の保証でない〈TD-714〉、Revisit: app の code が test の file を import したとき・route の生成器の ignore の接頭辞が変わったとき・CSS の scanner の source の設定が変わったとき・closure が test の file 由来の CSS の影響を見落とした事例が出たとき〈`@source not` で test の file を外す案を見直す。今は app の build の設定を変えて lane の範囲が広がるので見送る〉・Rust の分かれた test の dir〈TD-709〉や gate の文書〈TD-712〉を入れる必要が出たとき）。D-144 に `Superseded in part by: D-154` の行を足し、対象を名指す（round 2 #6。D-144 の実際の文で引く）: (1) の 2 句「自動の 3-way merge（`git -c merge.directoryRenames=false merge-tree --write-tree` の結果と tree が同じ。…）」と「生成物（…）だけは自動の merge と違ってよく、衝突してよい」（D-154 で、使い回しの経路では手で解いてよい path の中の衝突の解消と衝突していない手の編集も違ってよい。2 句は `plan-approve.sh`・`sync_closure`・`sync_from` の確かめ直しの既定〈`resolve=False`〉では今のまま）と、Compatibility の「ただし先に merge する PR と生成物・decision-log の外で衝突する見込みがある PR は、その merge と同期の後に L3 を頼む（DEV_WORKFLOW Wave Operation の L3 の箇条に 1 文で残す）」（その 1 文は D-145 (6) の「merge の順に縛らない」〈DEV_WORKFLOW:165〉で既に DEV_WORKFLOW から消えている。D-154 で衝突が集合の中なら L3 の後に同期してよい）。
- `docs/backlog.md`（round 1 #9）: 項目「衝突を解いた版の manual の再利用（観測）」を実装の merge で済みにし、将来形の文「残る観測は、それ以外の共有の file の衝突で manual のやり直しが何回起きたか…owner が改めて決める」を消す（計画の commit は起票の 1 文だけで、round 2 #6 でその 1 文の根拠を TD-714 の「JS の runtime module に入らない」に直した）。
- 本 packet（本 commit）。

Non-scope: `AGENTS.md`（Ready の承認を保つ判定は今のとおり helper の `--review-stage sync` か R0/R1 の明示の A で、手で解いた同期は承認を取り直す。B の承認と同じ 1 回に束ねる。TD-713）、`scripts/ci/classify-changes.sh`、`.github/**`、RecordV1、`scripts/plan-approve.sh`（呼出しは既定のまま。test だけ足す）。

## Acceptance Criteria

- AC1: `python3 scripts/tests/pr-gate.test.py` が全 PASS で、Test Plan の受ける組・拒む組の test（T1〜T18・T1b・T4b）が全部ある。
- AC2: `bash scripts/tests/run-workflow-tests.sh` が全 PASS（`plan-approve.test.sh` の PA5 の今の 3 組と、足した 2 組〈衝突した `docs/x.md` の手の解消・衝突していない `docs/x.md` の merge の中の編集〉がどれも「not a sync of main」で拒まれる）。
- AC3: Test Plan の mutation（M1〜M12）を 1 つずつ実装に入れると、`python3 scripts/tests/pr-gate.test.py`（M2 は加えて `bash scripts/tests/plan-approve.test.sh`）で名指した test が FAIL（exit 非 0）になり、戻すと全 PASS に戻ることを実装の run の報告に書く（D-098 (9)）。M2 では、PA5 の足した (a) の組で `plan-approve.sh` が packet を書かずに exit 非 0 で止まることを出力で示す（`plan-approve.sh` は `object.__new__(g.Gate)` で `endpoint` を持たないので、既定を True にすると classifier の読取りで例外になりうる。round 2 #3）。既定を True にすると例外でなく判定として手の解消を受けてしまうことの証明は、classifier の読取りを合成 base の実 file に patch した unit の T5 の red に置く。通常の `plan-approve.sh` は変えない。
- AC4: `bash scripts/doc-consistency-check.sh` と `--target plan docs/plans/2026-11-14-sync-docs-conflict.md` が ERROR なし、`bash scripts/check-workflow-git.sh` が exit 0。
- AC5: merge-evidence・DEV_WORKFLOW（L3 の表・Stacked train）・decision-log（D-154、D-144 の Superseded in part の対象〈(1) の 2 句と Compatibility の 1 文〉）・backlog の文が helper の振舞いと同じ（手で解いてよい path の中なら衝突の解消も衝突していない手の編集も受け 2 種を分けて名指す・手で解いてよい path の集合・A にしない・監査の closure・共通の確かめ → 承認の有無 → code の順・確かめ直し・plan-approve の既定・`r4` を束ねない）で、衝突していない手の編集の合否の基準（辻褄合わせで Scope の中なら合格、lane の新しい変更なら不合格で 1 親の commit に分けて通常の経路へ。round 2 #1）が Goal・merge-evidence の B・D-154・helper の B の案内の 4 か所で同じ文であり、L3 の使い回し C の節は変わらない（`git diff` で C の節に差分が無い）。
- AC6: 合成の repo の test で、#236 の形（docs と function-design の衝突を手で解き、main 側が code を含む）・#216 の形（`src/test/*.test.ts` の衝突）・#228 の形（`src/**/*.sd.test.tsx` と function-design の衝突）が B の command で manual を使い回せ、`--reuse-approval` の無い record・`--review-stage sync` は何も書かずに止まる。同じ衝突の解消に、衝突していない docs の追加の編集を 1 つ混ぜると、B で使い回せ、helper の出力と B の案内が「衝突として手で解いた path」と「衝突していない手の編集」を別の行で名指す（TD-715）。衝突していない集合の外の編集（例 `src/lib/x.ts`）を 1 つ混ぜると、どの経路でも「集合の外の違い」で止まり、L3 のやり直しだけを案内する。R4 の形では使い回した後も `r4` は pending（TD-710）。packet の無い R2 と R0/R1 の manual の lane でも B で使い回せ、`l3_change` を呼ばずに通り、R0/R1 の `--reuse-from` だけの明示の A は止まる（TD-711）。`workflow=true` の path・`src-tauri/**` の Rust の test・`src/routes/foo.test.tsx`・`src/test/.env.test` の手の解消は同期だけの条件を満たさない（TD-712・TD-709・round 1 #8）。
- AC7: 別の docs を手で解き、packet の link する Matrix の変更は main から自動の merge で取り込んだ同期で、`record --kind manual --outcome pass --reuse-from SOURCE --reuse-approval POINTER` が書込みの前に L3 の比較対象の変化で止まり、server の comment が変わらない（Codex round 1 F3。T15）。
- AC8: 同期の連鎖で、手で解いた同期の B の後の普通の同期（code に触れない）は A で manual を引き継ぎ、`status` の確かめ直し（`nonci`）が通る。手で解いた同期が 2 回続くと、2 回目は旧 manual の `reuse_approval` があっても A にならず、新しい `--reuse-approval` の B を要る（T17）。

## Design Readiness

- 引用する設計正本（節まで）: `docs/agent-guidance/merge-evidence.md` の MG-D13・「base同期だけでheadが変わる場合」（同期だけの条件、codeに触れない条件、manualの使い回し A・B、reviewのclosure）・「lane の docs だけの commit（C）と GA の後の closure」・「Helperの境界」、`docs/DEV_WORKFLOW.md` の L3 の表と Workflow State（D-098 の安全の境界）、`docs/decision-log.md` D-098・D-144・D-145 (6)(7)・D-146、`AGENTS.md` Working Rules の「継ぎ足さない」「最小の実装（D-147）」と Decision and Approval Boundaries の Ready の承認の範囲。
- 必要な設計成果物: 正本の文の改訂は updated in this PR（計画の固定の後の実装の run で、helper と test と同じ PR に入れる）。plan-first の正本の先行の改訂は要らない（規則の文と helper を同時に変え、食い違う期間を作らない。D-145 の TD-502 と同じ理由）。
- plan にしかない durable な判断の昇格先: 案の比較・境界の表 → D-154 と merge-evidence の該当の文。調べの command と出力は本 packet に残す。
- 前提・制約と延期した design gap: 手の解消が両側を保つかは機械で判定しない（案 4）。closure の 2 本と owner の判断に置き、D-154 の Guarantee range に書く。classifier が将来 docs の分類を変えれば集合も変わる（current main 側の classifier を使う今の方針どおり）。
- 絶対保証の例外と抜け道: 「code に触れる手の解消を受けない」は classifier の分類と (b) の定数の pattern に依る。classifier が未知の path を full にするので、未登録の path は受けない側に倒れる。pattern は `frontend`・`traceability`・`rust_drift` だけを外し、`env`・`generated`・`workflow`・`unknown`・`rust` は pattern に当たっても拒む。`src/routes/**` の test の file は route の生成器の既定の ignore の接頭辞 `-` に依る（`tsr.config.json` が変われば崩れる。D-154 の Revisit と `ponytail:` comment）。衝突した path と衝突していない手の編集の区分は `merge-tree --name-only` の衝突の列に依る（区分は名指しと closure の見る所に使い、受けるかどうかは区分でなく集合で決まる）。`merge-tree` の終了値 0/1 の外は判定不能として受けない側に倒れる。frontend の test の file の「JS の runtime module に入らない」は今の `vite build` と import の実測に依り、app の code が test の file を import するようになれば崩れる（D-154 の Revisit と helper の `ponytail:` comment に置く）。生成 CSS には Tailwind の scanner を経て入りうるので、「test の file の解消は app の build を変えない」とは保証しない。CSS への影響は B の closure 2 本が見る（TD-714、D-154 の Guarantee range）。
- 判定: ready（Open Questions は全部決まった: Q1 は TD-706・TD-709、Q2〜Q5 は TD-710〜TD-713。未決の問いは残らない）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| MG-D13 | merge-evidence 設計判断の表・「同期だけの条件」（TD-706 の (b) の集合の中の衝突の解消と衝突していない手の編集。TD-715） | `sync_merge(resolve=True)`・`sync_merges`・`TEST_FILES`（TD-709・TD-712・TD-715・round 1 #8） | T1・T1b・T2・T3・T4・T4b・T14・T16 | 非対象（helper） |
| MG-D13 既定 | merge-evidence「同期だけの条件」・「Helperの境界」 | `resolve=False` の既定と渡す 3 か所（round 1 #6） | T5、今の `SyncOnly` の test | 非対象 |
| MG-D13 B | merge-evidence「manualの使い回し」(B 承認) | `carry_manual`・`record --reuse-from`（TD-710・TD-711・round 1 #4） | T6・T7・T10・T13・T17 | 非対象 |
| MG-D13 B と `r4` | merge-evidence「manualの使い回し」の B の 1 文（TD-713 で束ねるのは B と Ready、`r4` は束ねない。round 1 #11） | 文だけ（helper は今のとおり `r4` を引き継がない） | T10 | 非対象 |
| MG-D13 共通の確かめ | merge-evidence「使い回しの元は…確かめ直し」の段落・「L3 の使い回し C」 | `resolved_change`（`carry_manual`・`record --reuse-from`・`nonci`。round 1 #3・#5）と 3 か所の順序（共通の確かめ → 承認の有無 → code。round 2 #2） | T11・T15・T13・T18 | 非対象 |
| MG-D13 B の基準 | merge-evidence「manualの使い回し」(B 承認) の判定の項目（衝突していない手の編集の合否の基準。round 2 #1） | 止める文と B の案内の「衝突していない手の編集」の行に基準の文を添える（分岐は足さない） | T12（文の有無）。合否は closure の判断（Test Plan の機械の test を置かない契約） | 非対象 |
| MG-D13 closure | merge-evidence「reviewのclosure」 | `sync_closure`（変えない。`resolve=False`） | T8 | 非対象 |
| MG-D13 確かめ直し | merge-evidence「使い回しの元は…確かめ直し」の段落 | `nonci` | T9・T17 | 非対象 |
| MG-D13 止める文 | merge-evidence「manualの使い回し」の使い回せないときの次の一手 | 止める文と B の案内（round 1 #1） | T12 | 非対象 |
| D-145 (6) C | merge-evidence「L3 の使い回し C」 | 変えない（`l3_change` を共通の確かめから呼ぶ） | T11、`scripts/tests/pr-gate.test.py` の今の C1〜C5・RC1〜RC4 の test（PK3 は `scripts/tests` を探さないので名を token にしない） | 非対象 |
| D-145 (7) 計画の節 | merge-evidence「docs の closure」・DEV_WORKFLOW Workflow State | 変えない（`plan_drift`） | 今の test | 非対象 |
| D-144 | decision-log D-144 (1)〜(7)・Compatibility | Superseded in part の行（(1) の 2 句と Compatibility の 1 文。round 1 #9・round 2 #6） | — | 非対象 |
| D-146 | decision-log D-146（B の「影響なし」） | 変えない（B の判定の項目に「両側を保つ」を足す） | — | 非対象 |
| D-098 | DEV_WORKFLOW Workflow State の安全の境界（古い証拠の拒否） | 境界の表の「拒む」列を helper の test で固定 | T1〜T17・T1b・T4b | 非対象 |
| PA5 | `scripts/plan-approve.sh` (a) の merge の判定 | 変えない（`sync_merge` の既定）。test に 2 組を足す（round 1 #7） | `plan-approve.test.sh` PA5 | 非対象 |
| DEV-L3 | DEV_WORKFLOW L3 の表 | 手で解いた同期の B の行と「取り直す」の行（round 1 #9） | doc-consistency | 非対象 |
| DEV-STACK | DEV_WORKFLOW Stacked train の衝突の解消の箇条 | 文の書き直し（round 1 #9） | doc-consistency | 非対象 |
| D-154 | decision-log（新規） | 本 lane の決定 | — | 非対象 |

## Test Plan

合成の repo の入力は、特に書かない限り main 側が code を含まない（code の拒否が新しい判定を隠さないため。Codex round 1 E）。main 側が code を含む組は B の案内を見る test と、順序を固定する T18 にだけ置く。拒む向きの test は `GateError` が出たことだけでなく、狙った段の止める文の固有の句を assert する（別の段の理由で止まって green になることを防ぐ。round 2 #5。例: 承認の無い B の record は `needs --reuse-approval`、packet・Matrix の path は「手で解いた packet・Matrix」、`l3_change` はその理由の句、集合の外は「集合の外の違い」）。

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| MG-D13 | 衝突した docs の手の解消を、使い回しの経路で拒む | unit（合成 repo） | T1 `test_resolve_conflicted_docs_returns_resolved` | `resolve=True` でも衝突した集合の中の path を拒む |
| MG-D13 TD-715 | 衝突していない docs の編集（単独、または衝突した docs の解消と同じ merge に混ぜたもの）を拒む／衝突していない集合の外の編集（`src/lib/x.ts`・未知の path）を受ける／受けた 2 種を同じ列に入れる | unit | T1b `test_resolve_unconflicted_edits_in_set_only` | 衝突していない手の編集を集合の判定に回さず今のとおり拒む、衝突していない path を集合の判定から外して受ける（M8）、または区分を付けずに 1 つの列にする |
| MG-D13 | 衝突した function-design（`traceability`・`rust_drift`）の手の解消を拒む | unit | T2 `test_resolve_traceability_docs` | 集合を C の `NON_DOCS` で作る |
| MG-D13 | 衝突した code・未知の path の手の解消を受ける | unit | T3 `test_resolve_rejects_code_unknown` | 集合の判定を外す（M1） |
| MG-D13 TD-706 round 1 #8 | frontend の test の file（`src/test/**`・`src/**/*.test.ts(x)`・`src/routes/-x.test.tsx`）の手の解消を拒む／近い外れ（`src/testing/x.ts`・`src/lib/test.ts`・`src/a.test.ts.bak`・`src/a.test.js`・`src/routes/foo.test.tsx`）や `env` の path（`src/test/.env.test`）を受ける | unit | T4 `test_resolve_frontend_test_files_exact` | pattern を `src/test` の前方一致や `test` の部分一致にする（M6）、pattern に当たった path の `env` 等の検査を飛ばす、`src/routes` の `-` の条件を外す |
| MG-D13 TD-709 | Rust の test（`src-tauri/src/**/*.rs` の `#[cfg(test)]` の中だけの違い、`src-tauri/tests/*.rs`）の手の解消を受ける | unit | T4b `test_resolve_rejects_rust_tests` | pattern に `src-tauri/**/tests/**` や `*_test.rs` を足す、または集合の判定を path の末尾の `test` で行う |
| MG-D13 既定 round 1 #6 | 既定の判定（`sync_merge`・`sync_merges`・`reusable`）が手の解消を受ける | unit・CLI | T5 `test_sync_merge_default_stays_strict`（classifier の読取りを合成 base の実 file に patch し、既定の判定が例外でなく判定として手の解消を拒むことを assert する。round 2 #3）と `plan-approve.test.sh` PA5 の足した (a)（packet を書かずに exit 非 0 で止まる） | `resolve` の既定を True にする（M2。T5 は判定として受けて red、PA5 の (a) は止める文の欠けで red） |
| MG-D13 A round 1 #4 | 手で解いた同期で manual を自動で引き継ぐ（旧 manual に `reuse_approval` が無い組と有る組） | unit | T6 `test_resolved_sync_never_auto_carries` | `carry_manual` が `self.resolved` を見ない（M3）、旧 manual の `reuse_approval` の有無で A を許す（M9） |
| MG-D13 B round 1 #4 | B の command で使い回せない／`--reuse-approval` を渡さない record で受ける（旧 manual が `reuse_approval` を持つ組を含む） | unit | T7 `test_resolved_sync_reuses_with_approval_only` | `record --reuse-from` が判定を結果の `approval`（引き継いだものを含む）で行う（M9）、承認の有無を見ない |
| closure | `--review-stage sync`・`docs` が手で解いた同期で機械の closure を書く | unit | T8 `test_resolved_sync_rejects_machine_closure` | `sync_closure` に `resolve=True` を渡す（M4） |
| 確かめ直し | status が承認の無い manual・`sync_from` の closure を通す | unit | T9 `test_nonci_rechecks_resolved_sync` | `nonci` の `reuse_approval` の確かめを外す（M5）、`sync_from` の確かめ直しに `resolve=True` を渡す |
| R4 TD-710・round 1 #11 | R4 の lane の手で解いた同期で B を拒む／`r4` を使い回す | unit | T10 `test_resolved_sync_r4_reuses_manual_keeps_fresh_r4` | `carry_manual`・`record --reuse-from` が R4 で手の解消を拒む、または `r4` を新しい head の record で pass のまま残す |
| 共通の確かめ round 1 #5 | 手で解いた path（衝突した path と、TD-715 で受けるようになった衝突していない手の編集の両方の組）に active な packet か `l3_basis` の Matrix があるのに使い回す（L3 の比較対象と計画の節は byte で同じになる入力。`l3_change` では止まらない）。Matrix の組は `## L3` の無い旧形式の packet（C5 と同じ形）で Matrix を link する（`## L3` のある packet では `l3_basis` が Matrix を読まない。round 2 #5） | unit | T11 `test_resolved_sync_packet_or_matrix_path_stops` | 共通の確かめの (i) を外す（M10） |
| 止める文と B の案内 TD-715 | 拒む文（集合の外の違い〈2 種を分けて〉・packet と Matrix）と、受ける側の B の案内（衝突として手で解いた path・衝突していない手の編集を別の行で）で種類を分けて名指さない、0 個の種の行や、手で解いた path が無い同期で新しい文を出す | unit | T12 `test_resolved_sync_names_paths_and_next_step` | 理由の文・B の案内から path の列挙か種類の名を落とす、2 種を 1 つの行にまとめる、0 個の種の行を出す、「衝突していない手の編集」の行から合否の基準の文を落とす（round 2 #1） |
| R0〜R2 TD-711 | packet の無い R2 と R0/R1 の manual の lane の手で解いた同期で、B を拒む・packet を読みに行って止まる／`--reuse-from` だけの明示の A を受ける | unit | T13 `test_resolved_sync_without_packet_reuses_with_approval` | 共通の確かめを packet の有無で分けずに呼ぶ（M7）、または明示の A で `self.resolved` を見ない |
| gate の文書 TD-712 | `workflow=true` の path（`docs/DEV_WORKFLOW.md`・`docs/agent-guidance/merge-evidence.md`）の衝突の手の解消を集合の中で受ける | unit | T14 `test_resolve_rejects_gate_docs` | 集合の判定から `workflow` を外す |
| 共通の確かめ round 1 #3 | 別の docs を手で解き、Matrix の変更は main から自動の merge で取り込んだ同期で、B の record が書かれる。packet は `## L3` の無い旧形式（C5 と同じ形）で Matrix を link する（round 2 #5） | unit | T15 `test_reuse_from_stops_on_auto_merged_matrix_change` | `record --reuse-from` の共通の確かめから `l3_change` を外す（M11） |
| decision-log round 1 #10・TD-715 | 衝突した `docs/decision-log.md` の union と違う解消、または衝突していない `docs/decision-log.md` の手の編集を拒む（2 種の区分を取り違える）／union と一致する解消を手で解いた path に数える | unit | T16 `test_resolve_decision_log_union_mismatch_goes_to_b` | union と違う結果を今のとおり拒む、または union と一致する結果を `self.resolved` に入れる |
| 連鎖 round 1 #10 | 手で解いた同期の B の後の普通の同期で A と `nonci` が通らない／手で解いた同期が 2 回続くと 2 回目が旧い承認で通る | unit | T17 `test_resolved_sync_chain` | `carry_manual` の起点を旧 record の head でなく `source_head` にする（前の B で済んだ手の解消が残り、後の普通の同期の A が止まる）、`nonci` が A で引き継いだ `reuse_approval` を認めない、または 2 回目の手で解いた同期を旧い承認で通す（M9） |
| 順序 round 2 #2 | L3 の変更（旧形式の packet が link する Matrix の main からの自動の取込み）と main 側の code が同時にある手で解いた同期で、code の理由が先に出る、または server に書く | unit | T18 `test_resolved_sync_l3_reason_precedes_code` | `carry_manual`・`record --reuse-from`（`--reuse-approval` の有る組と無い組）・`nonci` のどれかで code の require を共通の確かめより先に置く（M12）。どの組も `l3_change` の理由の句が出て code の句が出ず、mock server の PATCH/POST が 0 回であることを assert する |

mutation（実装の run で 1 つずつ注入し、red を確かめて戻す。AC3）: M1 集合の判定を外す → T3。M2 `resolve` の既定を True → T5 と PA5 の (a)。M3 `carry_manual` で `self.resolved` を見ない → T6。M4 `sync_closure` に `resolve=True` → T8。M5 `nonci` の `reuse_approval` の確かめを外す → T9。M6 test の file の pattern を `'test' in path` の部分一致にする → T4 と T4b。M7 共通の確かめの packet の有無の分岐を外す → T13。M8 衝突していない path を集合の判定から外して受ける（集合の外の衝突していない編集を通す）→ T1b。M9 承認の判定を結果の `approval`・旧 manual の `reuse_approval` で行う → T6 と T7。M10 共通の確かめの packet・Matrix の path の確かめを外す → T11。M11 `record --reuse-from` の `l3_change` を外す → T15。M12 `record --reuse-from` と `nonci` の code の require を共通の確かめの前に戻す（今の順）→ T18。

機械の test を置かない契約（helper では判定せず、B の closure 2 本の判定の項目で見る。実装の PR では、merge-evidence の B の判定の項目にこれらの文があることを doc の差分で確かめる。AC5）:

- 衝突していない手の編集の合否（TD-715・round 2 #1）: closure の発注は、helper が「衝突していない手の編集」と名指した path ごとに次を問う。(1) 同期で取り込んだ main 側の変更に合わせる編集（辻褄合わせ）か。(2) 承認済みの Scope の中か。(3) どの main 側の変更（merge の second parent の側の commit と path）に合わせたか。3 つとも答えられて (1)(2) が真なら合格。lane の新しい変更（Scope の外、または同期と関係のない変更）なら不合格とし、merge から外して lane の 1 親の commit に分け、通常の経路（C・監査の closure・Gated Amendment・L3 のやり直しのいずれか）に回すよう返す。
- frontend の test の file の生成 CSS への影響（TD-714・round 2 #4）: closure の発注は、D-146 に照らして次の 3 点を問う。(1) 手で解いた test の file で、class の literal の文字列が増えたか減ったか。(2) 当たるなら、app の code がその class を動的な組立て（文字列の連結・template literal 等）で使うか。(3) 当たるなら、それが L3 の項目の操作の合否や業務の結果を変えるか。画面の表層（表示の値・文言・配置）だけの変化なら影響なし（D-146）。merge-evidence の B の「L3 の項目の合否を変えない」（Scope）と同じ線で、「画面の見えが変わるか」を問わない。

L3: 置かない。operator の画面・app の build を変えず、helper と規則の文だけを変える。振舞いは合成の git の履歴の上の helper の test（上の表）で観測でき、Windows/Tauri native でしか観測できない項目が無い（DEV_WORKFLOW L3 Eligibility の (1) を満たさない）。

## 並走する lane への効き方

Coordinator の決定: 本 lane の merge まで #236・#227・#228・#216 の同期と merge を止める（helper の物差しを変えるため）。#194・#250・#251 は今の helper で進む。

TD-706 の (b) で書く。見送った (a) なら #228・#216 は test の file が集合の外で L3 のやり直しだった（#236・#227 は (a) でも同じ）。

| PR | 手で解く path（生成物を除く） | (b) での効き方 | 補足 |
|---|---|---|---|
| #236（R3） | docs 5 path（function-design の 24・57 を含む） | B で使い回せる | main 側が code を含むので、今の規則でも衝突が無ければ B だった。手で解いた 5 path を closure の範囲に足す。`docs/daily-monthly-acceptance-runbook.md` が L3 の手順に関わるなら、closure の「L3 の項目に影響なし」の判定で見る。packet の L3 の比較対象が手の解消で変わらないことも helper が確かめる |
| #227（R3） | `docs/backlog.md` | B で使い回せる（`90-traceability.md` は再生成） | — |
| #228（R4） | `docs/backlog.md`・24・37・45・55・`src/features/daily-report-import/DailyReportImportPage.sd.test.tsx` | B で使い回せる（TD-710。`*.test.tsx` が集合の中）。`r4` は新しい head で承認を取り直す | test の file の解消は closure が「test を消す・skip する・弱めていない」も見る |
| #216（R4） | `src/test/page-root-pageshell-sweep.test.ts` | #228 と同じ（TD-710。`src/test/**` が集合の中）。`r4` は取り直す | 同上 |

4 本はどれも手で解いた同期になるので、Ready の承認は B の owner の 1 行の承認と同じ 1 回で取り直す（TD-713）。

本 lane の merge の後: 止めた 4 本が同期すると、取り込む main 側に `scripts/pr-gate.py`（classifier は full）が入るので、どの lane も code に触れる同期（B）になる。4 本はもともと #235 の code を取り込むので B で、増える手は無い。#194・#250・#251 が本 lane の merge の後に同期すると、同じ理由で A でなく B になる（L3 を済ませた lane は closure 2 本と owner の承認が要る）。L3 を済ませた lane は本 lane の merge の前に merge するほうが手が少ない。本 lane の merge の後、同期の前の lane の helper は D-099 で base の版と違うと止まり、base の版の command を示す（今の動き）。

## Open Questions

### 決定の記録

- Q1 → TD-706（repo 外の回答台帳、2026-11-14）: owner は「両方推奨」と答え、Coordinator の推奨の (b) に決まった。手で解いてよい path（L3 を使い回せる同期の merge の違いの範囲。round 1 #1 で衝突した path に限ったが、TD-715 で差し替え、集合の中なら衝突していない手の編集も含める）は、docs（L3 の比較対象と計画の節に触れない所）と、JS の runtime module に入らない frontend の test の file（`src/**/*.test.ts(x)`・`src/test/**`。`src/routes/**` の中は basename が `-` で始まるものだけ。round 1 #8）。Rust の test は含めない。(a) は「案の比較」の 1-(a) に見送った案として残す。
- Q1 の根拠の訂正 → TD-714（repo 外の回答台帳、2026-11-15、owner の「a でいいかな」= Coordinator の推奨の (a)）: TD-706 の (b) の範囲のまま、根拠「test の file は app の build に入らない」を「JS の runtime module に入らない（Tailwind の scanner は test の file の class を拾い生成 CSS に入りうる。Codex round 1 F2 の隔離実験）」に直す。CSS への影響は B の closure 2 本が見る（D-146 の観点）と契約にする。Tailwind の `@source not` で test の file を外す案は、app の build の設定を変えて lane の範囲が広がるので見送る（D-154 の Revisit）。
- 衝突していない手の編集 → TD-715（repo 外の回答台帳、2026-11-15、owner の決定は Coordinator の推奨の (a)。owner の所感の原文は台帳にあり、本 packet には ID で引く）: 計画の review round 1 の裁定 #1（衝突した path に限る）を差し替える。手で解いてよい path の集合（docs と TD-706・TD-714 の test の file）の中なら、衝突の解消だけでなく、衝突していない path の自動の merge との違い（手の編集）も受けて B に回す。closure 2 本と owner の 1 行の承認は今のまま。集合の外の違い、packet・Matrix・L3 の比較対象・計画の節、code・生成物・decision-log の扱いは変えない（round 1 の #3〜#11 の是正は全部残す）。helper の出力と B の案内は 2 種を分けて名指し、closure は衝突していない手の編集が何のためか、L3 の項目に影響しないかを見る。
- Q1 の Rust の範囲 → TD-709（2026-11-14）: Rust の test の大半は本体の file の中（`src-tauri/src` の 102 file 中 69 file の `#[cfg(test)]`）で、分かれた dir（`src-tauri/tests`・`src-tauri/dev-tools/tests`・`src-tauri/src/biz/csv_import_service/tests`）もあるが、TD-709 で全部外したままにする（分かれた dir も入れない）。必要になれば D-154 の Revisit で見直す。

以下は owner の「全部推奨」（Coordinator の推奨のとおり）で決まった。

- Q2 → TD-710（2026-11-14）: R4 の lane（#228・#216）に B のこの道を開く。今の B は R4 の code に触れる同期にも効き、`r4` の承認は新しい head で取り直す。
- Q3 → TD-711（2026-11-14）: packet の無い R0〜R2 の lane にも開く。今の B と同じく closure の 2 本の証跡を `reuse_approval` の pointer の先（R0/R1）か review の closure の evidence（R2）に置き、L3 の基準は PR body にあって tree の手の解消で変わらないので `l3_basis` の比較をしない。
- Q4 → TD-712（2026-11-14）: gate の文書（classifier の `workflow=true`）の手の解消は集合の外に置く。規則の文の手の解消は lane の規則の review の対象で、機械の分岐を増やさない。必要が出たら D-154 の Revisit で見直す。
- Q5 → TD-713（2026-11-14）: 手で解いた同期は helper が「同期だけ」と判定しないので、AGENTS.md の Ready の承認は取り直しになる。B の owner の 1 行の承認と同じ 1 回に束ね、AGENTS.md は変えない。

未決の問いは残らない。

## 計画の review に問う点

closure round 2（発注 901、`e3c20d09`、Codex・Opus とも request changes〈P1 0 / P2 1〉）の答え。裁定は PR #257 の comment「計画の review の closure round 2（発注 901、`e3c20d09`）の報告と裁定」（Coordinator、2026-11-15）。round 1 の裁定 #2〜#11 と TD-715 で差し替えた #1 は、2 本とも今の版で成り立つと判定した（round 1 の表は PR #257 の round 1 の裁定の comment にある）。

| # | 裁定の要旨 | 直した所 |
|---|---|---|
| 1 | 衝突していない手の編集を lane の新しい変更と判定したときの不合格の基準が無い（同期と無関係な docs を merge に混ぜると B で通る）。機械の分岐を足さず、辻褄合わせで承認済みの Scope の中なら合格、lane の新しい変更なら closure は不合格で、merge から外して lane の 1 親の commit に分け、通常の経路へ | Goal の最小完了条件の closure の判定の項目、失敗定義の 2 つ目、非目的の C の行（C を広げない関係と、owner に別に確かめる件）、境界の表の `sync_merge` の行、境界の 4 観点の全部、Scope の止める文と B の案内・merge-evidence の B の判定の項目・D-154 の境界、AC5、Ledger の「MG-D13 B の基準」、T12、Test Plan の機械の test を置かない契約（closure の発注で問う 3 つの文） |
| 2 | 失敗の順序が `record --reuse-from`・`nonci` の今の順（code の require が先）と合わない。3 か所とも「共通の確かめ → 承認の有無 → code」に並べ替え、L3 の変更と main 側の code が同時にある入力で L3 の理由が先に出て書込みが無いことを test で固定 | Scope の「3 か所の順序」と `record --reuse-from`・`nonci` の行、境界の 4 観点の失敗の時点と順序、Ledger の共通の確かめ、Test Plan の前文、T18・M12、AC1・AC3、D-154 の境界 |
| 3 | AC3 の M2 は PA5 で `AttributeError`（`object.__new__` の Gate に `endpoint` が無い）になり、「例外での停止でない」を満たせない。PA5 (a) は packet を書かずに exit 非 0 で止まることを求め、例外でないことの証明は classifier を patch した unit（T5）に置く。通常の `plan-approve.sh` は変えない | AC3、T5 の行 |
| 4 | TD-714 の closure の問い（「画面の見えを変えるか」）が D-146 と食い違う。Scope の「L3 の項目の合否を変えない」に揃え 3 点の問いに | Test Plan の機械の test を置かない契約の 2 つ目の項目 |
| 5 | T11・T15 の Matrix の組は `## L3` の無い旧形式の packet でないと組めない。止める文の固有の句を assert させる | T11・T15 の行、境界の表の packet・Matrix の行、Scope の共通の確かめの (i)、Test Plan の前文 |
| 6 | `docs/backlog.md` の本 lane の 1 文の「build に入らない」、Scope の D-144 (1) の引用が D-144 の文と合わない | `docs/backlog.md` の 1 句（TD-714 の根拠に）、Scope の backlog の行、Scope の decision-log の D-144 の Superseded in part の対象（(1) の実際の 2 句と Compatibility の 1 文）、Ledger の D-144、AC5 |
| 7 | `self.resolved` を 2 種で持つとき、class の既定 `()` と実体 `([], [])` で `if self.resolved:` が常に真になる罠 | Scope の「`self.resolved` の表し方」（種と path の組の集合を 1 つの属性に持つ）と `sync_merges` の行、境界の 4 観点の数の範囲と 0 の表示 |

是正で足した契約の文（#1 の基準・#2 の順序・#7 の表し方）は、境界の表の後ろの「境界の 4 観点」（数の範囲・0 の表示・失敗の時点と順序・種類の混在）で一巡した。

closure round 2 の裁定の注（owner に別に確かめる件、出所: round 2 の Opus の P2）: TD-715 の owner の所感のうち、同期と関係のない lane の新しい docs の変更を R4・packet の無い lane・`traceability` の docs で L3 のやり直しなしに受けることは C の拡張で、本 lane の非目的の外。本 lane は #1 の形で進める。

closure round 3（2 本）で見てほしい所:

- #1: 衝突していない手の編集の合否の基準が、Goal・失敗定義・境界の表・merge-evidence の B・D-154・helper の B の案内・Test Plan の closure の問いで同じ文になっているか。基準が機械の分岐を足さずに、Codex round 2 の反例（同期と無関係な `docs/function-design/99-unreviewed.md` を merge に足す）を closure の不合格として塞ぐか。不合格の後の「1 親の commit に分けて通常の経路へ」が、C を広げない（非目的）と両立するか。
- #2: 3 か所の「共通の確かめ → 承認の有無 → code」の並べ替えが、今の `reusable` の `code=` の引数の分岐を require の条件へ移す形で、今の拒む向き（`--reuse-from` だけの明示の A で code を拒む、`reuse_approval` の無い manual で code を拒む）を弱めないか。T18 と M12 が、`record --reuse-from` と `nonci` の順を今の順に戻す mutation を red にする組か。
- #3: AC3 の M2 の PA5 の求め（packet を書かずに exit 非 0）と T5 の patch した unit で、既定を True にする mutation が判定として手の解消を受けることを示せるか。
- #4: 生成 CSS の 3 点の問いが D-146（表層だけなら影響なし）と食い違わず、操作の合否・業務の結果を変える class の変化を拾えるか。
- #5: T11・T15 の旧形式の packet の組と、止める文の固有の句の assert が、別の段の理由で green になる経路を塞ぐか。
- #6: D-144 の Superseded in part の対象（(1) の 2 句と Compatibility の 1 文）が D-144 の実際の文と一致し、既定の `resolve=False` の経路では 2 句が今のまま効くことと矛盾しないか。Compatibility の 1 文を対象に残したこと（round 1 #9）が round 2 #6 の「その 2 句にする」の読みと合うか。
- #7: `self.resolved` を種と path の組の集合にする表し方が、同じ path が merge によって別の種になる場合と、`plan-approve.sh` の `object.__new__` の Gate（class の既定 `()`）の両方で正しいか。
- この変更で不要になる記述・条件が残っていないか（round 1 の表を PR の comment に任せて packet から外したこと、D-144 の対象、backlog の 1 句）。
