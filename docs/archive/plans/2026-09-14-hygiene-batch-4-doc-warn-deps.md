# Plan Packet: 衛生 batch 4（doc WARN / mockup-g / npm 名指し更新）

2026-09-14 発注に基づく起草。起点は `04644c80da187df6f9168f34fb6a6f0cd1b19a22`（origin/main）。2026-09-13 の旧発注は未実行のまま本版で置換する。実装は別 run とし、Coordinator 裁定と独立 Plan Review 通過後に発注する。

## Workflow State

- Evidence Mode: github
- Phase: archive
- Risk: R2
- Execution Mode: fable-window
- Plan Commit: be43418e13a2a382f79fd6d1f94667a7624b0b2b
- Amendments: 039c225b363cc4484f16d62144e73d5c673da3e7, 2b807fb9b0e733bcf0f5b473d289dca3692ffa5b
- Coordinator: Fable
- Writer: Codex
- Plan Reviewer: Sonnet（round 1、P1/P2 = 0）
- Final Reviewer: Sonnet + Opus（pending、独立 fresh context）
- Final Review Minimum: 2
- Human Gate: ready,merge

L3 不要: 利用者可視の変更はなく、参照 mockup の適用対象・CSS 宣言・描画を保持する。Fable 指揮の分業 lane であり D-087 の一貫担当例外は適用しない。
GitHub mode の有効化根拠は [PR #59 の専用記録](https://github.com/kosei-w90607/inventory-system-desktop/pull/59#issuecomment-5664736990)（`inventory-merge-activation-v1`、`inventory-main-merge` active、`Merge gate` strict）。本 lane は有効化後最初の R2+ PR として workflow effectiveness dogfood を担う。

## Owner Effort Budget

- 介入回数上限: 3
- 実働時間上限: 30分
- relay 往復上限: 2
- Plan Review round 天井: 3

DEV_WORKFLOW の既定値。承認依頼は「この change での介入 N 回目 / 予算 3 回」と完了する利用者価値を併記し、Coordinator が decision point 単位で計上する。Ready と merge は独立した判断。超過見込みでは任意の証拠作業を減らし、無断で予算を広げない。

## Subagent Budget

R2 の既定は同時 0〜1、depth 1、全 lane 合算同時上限 4、one-writer。今回の起草 run は単一 thread・直列、子 agent 0。後続の Plan Review と Final Review は Coordinator が別 run へ発注し、Final の独立 2 パスも同時上限を守る。返却は file:line 付き約20項目以内の要約。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R2

checker 本体に触るが、変更は既存 WARN 経路の拡張子除外 heuristic 1 行で、exit code と gate の合否契約を変更しない。lockfile / manifest を含むため旧 lane の R1 docs-only は採らない。checker の検出力低下を独立 fresh context の Final Review 2 パスで確認する。代替の R3 full（Spec Contract / Trace Matrix / Test Matrix）は契約非変更の規模に対して過剰なため採らない。

## Goal

Goal Invariant:

### 最小完了条件

- doc-consistency WARN 0 件、npm 由来の open alert 0 件、mockup-g の `:has(` 0 件。既存 runtime と描画を保持したまま衛生項目を解消する。
- smol-toml の第一候補は override。owner が dismiss 案を選ぶ場合だけ、npm high 残存を理由付きで受容し、open alert 0 件を dismiss 記録で満たす条件へ AC3 とともに差し替える。上流待ち案では最小条件を満たしたと主張せず、Coordinator が範囲と Goal を再裁定する。

### 失敗定義

- runtime 契約、gate の exit code、mockup の描画を変える、または `.npmrc` / `src-tauri/Cargo.lock` に diff を出す。
- 一般の未定義カラムの検出を失う、名指し以外の依存更新を混ぜる（名指し package の subtree で npm が再解決する semver 内 transitive は除く。Gated Amendment 2）、再走査前の GitHub 状態を解消済みと扱う。

### 非目的

- Non-scope の領域は本 lane の最適化対象にしない。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。

## 起票時実測

2026-09-14、起点 worktree で `git rev-parse --short HEAD` → `04644c80`。`bash scripts/doc-consistency-check.sh` → exit 0、`結果: WARN 5 件（ERROR なし）`、M3 は 3 箇所。WARN 内訳はページング 1（2 file）、カラム 3、M3 1（3 箇所）。
`gh api --paginate 'repos/kosei-w90607/inventory-system-desktop/dependabot/alerts?state=open'` → 下表の 6 件。lockfile の packages を Python 標準 JSON parser で確認し、js-yaml 4.3.1、smol-toml 1.7.0、vitest / mocker 4.1.5 と親依存の range / exact pin を照合した。

| alert | severity | package / 現行 → fix | manifest / 経路 |
|---|---|---|---|
| 7 | high | js-yaml 4.3.1 → 4.3.2 | package-lock.json、eslint 9.39.4 → @eslint/eslintrc 3.3.5 の `^4.1.1` |
| 6 | high | smol-toml 1.7.0 → 1.7.1 | package-lock.json、markdownlint-cli2 0.23.2 が `1.7.0` exact pin |
| 5 / 3 | medium | vitest / @vitest/mocker 4.1.5 → 4.1.11 | package.json / package-lock.json、vitest は direct devDep exact pin |
| 2 | low | rand 0.7.3 → 0.8.6 | src-tauri/Cargo.lock、phf_generator ← selectors build-dep ← kuchikiki ← tauri-utils |
| 1 | medium | glib 0.18.5 → 0.20.0 | src-tauri/Cargo.lock、gtk 0.18 ← tao / wry / tauri、Linux target |

Coordinator の発注時測定: `npm audit --audit-level=high` → vulnerabilities 5（high 3、moderate 2）。publish 日は js-yaml 4.3.2 = 2026-08-26、smol-toml 1.7.1 = 2026-07-26、vitest 4.1.11 = 2026-08-18、いずれも7日超。起草者による audit / publish 日の再測定は未実測（起草 run は install 不要）。実装時に `npm view js-yaml time --json` / `npm view smol-toml time --json` / `npm view vitest time --json` と audit で再照合し、前提が変われば範囲を拡張せず Coordinator へ返す。

## Scope

### S1a ページング記述

案: `74-ui-operation-logs.md` §74.10 に「per_page の上限は 200（PAGINATION_MAX_PER_PAGE、IO 層 system_repo で clamp、43-cmd-settings-log.md §43.5）」を1行追記。`40-cmd-product.md:279` は「ページング引数を持たない」に言い換える。`constants.rs:6` / `system_repo.rs:124` / `stocktake_repo.rs:499` と §43.5 の既存契約に合わせるだけ。
代替は checker の否定文脈除外だが、文脈依存の判定を増やすため採らない。runtime 上限追加も不要。

### S1b route path のカラム誤読

案: `scripts/doc-consistency-check.sh` の col_part 抽出直後に `case "$col_part" in tsx|ts|rs|md|sh|json|css|html|js|toml|yml|yaml|py|lock) continue ;; esac` を1行追加。`[a-z_]+\.[a-z_]+` が table 名 suppliers と拡張子 tsx を抽出し、実在 table suppliers の column tsx と誤読する経路を除く。対象は `52-ui-shared-layout.md:88` / `78-ui-supplier-management.md:19,42`。
代替の route path 表記変更は正確さを落とし、取り消し線化は意味を変えるため採らない。拡張子名と同名の不正カラムも除外される heuristic の限界は保持し、他 check は変えない。
既存 `scripts/tests/doc-consistency-plan-packet.test.sh` に独立 tmpdir の実 checker full 実行 fixture を追加する。同 file は既存 workflow suite に登録済みで、別 file と runner 登録を増やさず到達性を保持できるため。既存 plan-mode fixture を変更せず、合成 DB 設計に suppliers を定義し、route path（`src/routes/settings/` + `suppliers` + `.tsx`） では対象 WARN なし、table 名 `suppliers` + `.` + 未定義 column 名 `hygiene_missing_column` では対象 WARN ありを検証する。両ケースは ERROR なし / exit 0 も確認し、fixture 設定不良による失敗を検出成功に数えない。

### S1c M3 の文言

案: `docs/backlog.md:155` の owner 所感を「owner 自身も判断を保留」へ、同 `:231` の節参照を「docs/ARCHITECTURE.md §4」へ、`docs/quality/review-checklist.md:91` を「A/B の未決状態」へ言い換える。引用の改変ではなく、実在節と DSR-24 の意味を保持する。
代替の M3 除外追加・取り消し線化は検出力や意味を変えるため採らない。起草 run ではこれらの是正は行わない。

### S2 mockup-g の selector

案: `docs/design-system/reference/mockup-g-filter-toolbar.html:86` の h1 を持つ current パネルの section-heading と heading-copy に `legacy-page-heading` を追加し、64-67行の4 selector の `:has(h1)` を `.legacy-page-heading` に置換する。既存 class、CSS 宣言、対象要素を保持し、h2 の current パネルへ広げない。詳細度は `:has(h1)` の (0,2,1) から class の (0,3,0) へ上がるが、同要素・同 property に競合する宣言は無く（`:59-69` 実測: `.section-heading` の align-items は (0,1,0)、`.heading-copy` は min-width、`.btn` は flex-shrink を持たず、`.heading-copy>p` は font-size / color で別 property）、計算値と描画は不変（Gated Amendment 1）。
代替の current 全体指定は h2 にも作用し、現状維持は互換性依存を残すため採らない。他 mockup は編集しない。

### S3 npm 名指し更新と Cargo 記録

案: D-029 / D-030 の常設ガード下で `npm update js-yaml --ignore-scripts`、`npm install vitest@4.1.11 --save-dev --save-exact --ignore-scripts` を直列実行。js-yaml 5.2.2 の既存別経路は不変とする。
smol-toml は candidate (1): package.json に `"overrides": { "smol-toml": "1.7.1" }` を追加し `npm update smol-toml --ignore-scripts` で lockfile を解決する。npm native の patch override を Plan / PR 本文で名指しし、撤去条件を backlog の「記録目的」へ1行残す。新規 runtime 依存ではなく D-030 の運用内であり、decision-log は変更しない。
代替 (2): owner が `tolerable_risk` で alert #6 を dismiss。dev-only lint tool が repo 内 TOML 設定を parse する経路のみ、root に markdownlint TOML 設定はないという理由文を用意する。high が残るため第一候補にはしない。
代替 (3): markdownlint-cli2 の上流 release 待ちで lane から外す。目標の達成を先送りするため第一候補にはしない。
単純な smol-toml 更新は親の exact pin に阻まれ、audit の markdownlint-cli2 0.21.0 downgrade、vitest 5 major 更新は不要な互換性リスクのため採らない。
Cargo 2 件は bump せず、経路・dismiss 理由候補を backlog「記録目的」へ1行記録する。dismiss 実行は owner のみで、起草者・実装 Writer は実行しない。

## Non-scope

- runtime、DB、DTO、生成物、eslint 設定、per_page clamp の追加・変更。
- Cargo 依存 bump、`.npmrc`、`min-release-age-exclude[]`、名指しでない npm update / upgrade、npm audit fix。
- mockup-g 以外の design-system docs、doc-consistency の他 check の挙動変更。
- 起草 run の install・実装・PR 作成・Ready・merge・alert dismiss。

## Acceptance Criteria

- AC1: `bash scripts/doc-consistency-check.sh` が exit 0、WARN / ERROR とも0。実際の成功行は `結果: 全チェック通過`（checker :2078-2084）。発注の `結果: WARN 0 件（ERROR なし）` はゼロ件の意味として保持するが、その文字列は現行実装では出ない。出力形式変更はしない。
- AC2: `rg -c ':has\(' docs/design-system/reference/mockup-g-filter-toolbar.html` がマッチ0（出力なし、exit 1）。`git diff --unified=0 origin/main..HEAD -- docs/design-system/reference/mockup-g-filter-toolbar.html` は起点64-67行の selector 置換と86行の class 追加だけ。CSS 宣言およびそれ以外のHTMLは byte 不変。
- AC3: `npm audit --audit-level=high` exit 0。`gh api --paginate 'repos/kosei-w90607/inventory-system-desktop/dependabot/alerts?state=open' --jq '[.[] | select(.dependency.manifest_path == "package-lock.json" or .dependency.manifest_path == "package.json")] | length'` が0。push 後の GitHub 再走査には時差があり、default branch の検出が更新されるまでは pending と記録する。branch push だけで必ず消えるとは仮定せず、merge 後の再走査が必要なら Coordinator が closeout の確認対象として保持し、実装完了時点の未達を隠さない。
- AC3 差替え候補（owner が S3 (2) を選んだ場合のみ）: `npm audit --json` の残存 high は smol-toml / markdownlint-cli2 経路だけを許容し、`npm audit --audit-level=high` の非0を記録する。上記 open alert 件数0に加え `gh api repos/kosei-w90607/inventory-system-desktop/dependabot/alerts/6` の state=dismissed / dismissed_reason=tolerable_risk / dismissed_comment に理由文ありで代える。Goal の例外も同時に裁定し、未承認のまま差し替えない。
- AC4: `git diff --stat origin/main..HEAD -- package-lock.json package.json` と `git diff origin/main..HEAD -- package-lock.json package.json` で js-yaml / smol-toml / vitest / @vitest/* / tinyrainbow（vitest 名指し更新で npm が同 subtree を再解決した transitive。実測 3.1.0 → 3.1.1、vitest 4.1.5 / 4.1.11 とも range `^3.1.0` 内、publish 2026-07-28 で cooldown 7 日超。Gated Amendment 2）と overrides 追加（root vitest pin 同期を含む）だけ。stat 単独では package 範囲を証明できないため JSON 差分の package key / version / resolved / integrity / dependency metadata も確認する。`git diff --exit-code origin/main..HEAD -- .npmrc src-tauri/Cargo.lock` exit 0。他 package 更新が必要になったら Coordinator へ返す。
- AC5: `npm run lint` / `npm run typecheck` / `npm run format:check` / `node_modules/.bin/vitest run` がすべて exit 0（vitest 4.1.11 の全件）。
- AC6: `bash scripts/tests/doc-consistency-plan-packet.test.sh` exit 0。S1b の route path 正例 / 未定義カラム負例の両方が PASS。現行 checker では正例が失敗し、修正後は両例が成功することを確認する。
- AC7: `bash scripts/local-ci.sh full` が `RESULT=PASS`。1 worktree 1 run は同時走行禁止であり回数上限ではない。FAIL → 是正 → 再実行に再許可は不要。
- AC8: `bash scripts/check-workflow-git.sh` PASS、exit 0。

## Design Sources

- Function: `docs/function-design/74-ui-operation-logs.md` §74.10、`40-cmd-product.md` の ProductBulkFilter、`43-cmd-settings-log.md` §43.5。
- Route / DB: `52-ui-shared-layout.md`、`78-ui-supplier-management.md`、`docs/DB_DESIGN.md` と docs/db-design の suppliers 定義。
- Architecture / UI: `docs/ARCHITECTURE.md` §4、`docs/design-system/01-decision-rules.md` DSR-24、mockup-g。
- Decisions: `docs/decision-log.md` D-029 / D-030、CLAUDE.md npm供給網ガード。
- Workflow: `docs/DEV_WORKFLOW.md`、`docs/AGENT_OPERATING_MANUAL.md` §3.2、`docs/agent-guidance/merge-evidence.md`。

## Required Design Artifacts

既存正本で十分。新規設計成果物なし。74 / 40 は runtime 実態の説明同期、レビュー文言と参照 mockup は既存意味を保持する。

## Registration / Generation Obligations

新 packet は同じ plan-first commit の Plans.md「次の行動」に登録し backlog の lane 末尾へ pointer を追加する。S1b は既存 suite 登録済みテストに追加するため新規 runner 登録不要。REQ token、command、route、bindings の変更・生成義務なし。

## Design Intent Trace

R2 のため独立した Trace Matrix は作らない。S1a → §74.10 / §43.5 → AC1、S1b → 既存 DB カラム検出 → AC6、S1c → DSR-24 / §4 → AC1、S2 → mockup-g → AC2、S3 → D-029 / D-030 → AC3〜5。

## Design Intent Audit

runtime / DB / wire / operator 操作の新判断なし。override は npm native の一時運用で、撤去条件を実装時に backlog へ残す。S1b は既知の heuristic の限界を明示し、実在 table の一般的な不正カラム検出を負例で維持する。

## Impact Review Lenses

環境・再現性: 既存 npm / Node pin と常設ガードを維持。外部 alert 再走査は local audit と分離して記録する。他の業務・adapter・会計・データ lifecycle lens は対象外（runtime / 実データ非接触）。

## Design Readiness

Design Phase 判定: source design の契約変更なし（74 / 40 は実態への追記・言い換えのみで design 判断を含まない）。既存正本で十分。decision-log は変更しない。実装開始は override 候補の Coordinator 裁定と Plan Gate 通過後に限る。

## Contract Probe

R2 のため必須 probe は非該当。起票時の checker / lockfile / API 観測は「起票時実測」に記録。publish 日と解決後依存範囲は実装時の AC3 / AC4 で確認する。

## Contract Coverage Ledger

R2・runtime 契約変更なしのため非該当。AC1〜8 で検証対象を閉じる。

## Test Plan

Test Matrix は R2 かつ AC 全件が機械 oracle のため省略する。
対象検証は AC1〜6、統合 gate は AC7 / AC8。新規負例は S1b の synthetic fixture のみで実データを使わない。S2 は厳密差分で宣言・適用対象不変を確認し L3 を追加しない。実装時は `npm ci --ignore-scripts` と `npm run generate:routes` で既存の検証前提を整え、生成物に差分があれば範囲外として報告する。
起草 run の gate は doc-consistency（PK1〜PK4、M3増加なし）、check-workflow-git（PK5）、3 file 限定の差分確認。local full は実装 run の義務。

## Boundary / Wire Contract

- producer: Writer の名指し npm 更新と package.json の exact override。
- consumer: npm resolver / npm ci、既存 lint / Vitest runner。
- wire type / internal type: package.json と package-lock.json の既存 JSON schema、文字列の exact version / range。
- precision/range: vitest 4.1.11、smol-toml override 1.7.1、js-yaml は eslint 系既存 range 内 4.3.2。lockfileVersion と既存他依存 pin を保持。
- round-trip path: manifest → npm resolve → lockfile → npm ci → AC5 / AC7。
- invalid input: cooldown違反・解決不能・範囲外差分は失敗として止める。guard解除や downgrade で回避しない。
- compatibility: runtime wire / DB / Cargo / .npmrc 不変。override は上流 pin 解消時に撤去する。

## Review Focus

S1b が WARN だけを変え、一般の不正カラム検出と exit code を保持するか。S2 の計算値と適用対象が同じか（詳細度の変化は Gated Amendment 1 で契約化済み。競合宣言が無いことを `:59-69` で再確認する）。npm 差分が名指し範囲に閉じるか。smol-toml 候補と open alert 再走査の未達を合格扱いしていないか。Final Review は Workflow State の独立2パスとし自己裁定しない。

## Spec Contract

R2 のため独立した Spec Contract は非該当。既存契約の保持は Scope / AC を参照。

## Trace Matrix

R2 のため省略。Scope の各項と AC の対応を維持する。

## Data Safety

実 DB / backup / 店舗情報に触れない。S1b fixture はテスト所有 tmpdir 内だけに生成・cleanup し tracked fixture を追加しない。Cargo / .npmrc の差分なしを AC4 で確認する。

## Writer Instructions

起草 run の編集は本 packet、docs/Plans.md の active bullet 1行、docs/backlog.md の lane 末尾 pointer 1文のみ。plan-first commit と push 後に停止し、PR を作らず worktree を外す。
実装 run は packet / Plans.md を編集せず、次の確定 file 一覧だけを対象とする（smol-toml 選択が変わる場合は Coordinator が先に契約を裁定する）。

- `docs/function-design/74-ui-operation-logs.md`
- `docs/function-design/40-cmd-product.md`
- `docs/backlog.md`
- `docs/quality/review-checklist.md`
- `docs/design-system/reference/mockup-g-filter-toolbar.html`
- `scripts/doc-consistency-check.sh`
- `scripts/tests/doc-consistency-plan-packet.test.sh`
- `package.json`
- `package-lock.json`

明示 path で git add。範囲外 diff は push 前に停止して報告する。実装では小さな既存 pattern / npm native を使い新規抽象化を足さない。S1b runnable check を先に失敗させてから heuristic を修正する。
実装後は Draft PR と helper の専用記録 / CI で evidence を管理し、tracked state-only を追加しない。owner の Ready / merge は別途必要。dogfood では commands 数 / 手戻り / owner 介入 / 不要な再検証の有無を PR に記録し、workflow effectiveness の所見を closeout に引き継ぐ。

## Implementation Results

[PR #61](https://github.com/higekiri-w/inventory-system-desktop/pull/61) で実装し squash merge 済み（2026-09-15）。

- S1a: `74-ui-operation-logs.md` §74.10 に per_page 上限の記述を追記し、`40-cmd-product.md:279` をページング引数を持たない表現へ言い換えた。
- S1b: `scripts/doc-consistency-check.sh` の col_part 抽出直後に拡張子除外の `case` を追加し、route path のカラム誤読 WARN を解消した。`scripts/tests/doc-consistency-plan-packet.test.sh` に route path 正例 / 未定義カラム負例の fixture を追加した。
- S1c: `docs/backlog.md` / `docs/quality/review-checklist.md` の M3 該当文言を実在節参照へ言い換えた。
- S2: `docs/design-system/reference/mockup-g-filter-toolbar.html` の `:has(h1)` selector 4 箇所を `.legacy-page-heading` class へ置換した（Gated Amendment 1、計算値・描画は不変）。
- S3: `npm update js-yaml` / `npm install vitest@4.1.11 --save-exact` を名指し実行し、smol-toml は package.json `overrides` 1.7.1 で解決した（Gated Amendment 2、vitest subtree の semver 内 transitive `tinyrainbow` 再解決を許容）。

Final Review は Sonnet（pass、findings 0）+ Opus（pass、P3 4 件）の独立 2 パスで完了し、要約は PR #61 の comment（Sonnet: `#issuecomment-5667210488` / Opus: `#issuecomment-5667210788`）に記録済み。専用 record `inventory-workflow-v1` は review outcome pass、broad audits 2 件。Dependabot は merge 後の default branch 再走査で open alert 0 件（2026-09-15 確認）。hosted CI `Merge gate` は該当 HEAD で成功した。

## Review Response

- Findings Freeze: frozen after Final Review 2 passes (2026-09-15); post-freeze exceptions: none.

- 2026-09-15 Plan Review round 1（Sonnet、独立 fresh context、対象 `be43418e`）: P1/P2 = 0、P3 1 件。P3 = checker が hosted CI の docs job 経由で `Merge gate` の直接依存であるため、Risk 節に「Final Review Minimum 2 は workflow gate 該当を仮定した保守的選択」と明記する提案。Coordinator disposition = 記録のみ（Risk 節の根拠は WARN 経路のみで exit code 不変を実測済み、Final Review 2 パスで同観点を確認する）。plan-draft → plan-gate → plan-approved → implementing を本 commit で実体化し、Plan Commit を `be43418e13a2a382f79fd6d1f94667a7624b0b2b` に設定。

- 2026-09-15 Gated Amendment 1（Coordinator）: 実装 run（発注書 47 改訂 2）が S2 の「specificity 保持」と class 置換の両立不能（`:has(h1)` = (0,2,1) / class = (0,3,0)）で停止。Coordinator が mockup-g `:59-69` を実測し、競合宣言が無く計算値不変であることを根拠に、S2 と Review Focus の契約を「計算値・描画不変」へ訂正。選択肢 `:where()` による詳細度維持は保守者に説明が要る selector になるため採らない。AC2 は不変。
- 2026-09-15 Gated Amendment 2（Coordinator）: 実装 run 3（発注書 47 改訂 3）が S3 で `npm install vitest@4.1.11 --save-exact` により transitive `tinyrainbow` 3.1.0 → 3.1.1 が再解決され、AC4 の許可 list 外として正しく停止（依存差分は退避、S1a / S1b / S1c / S2 の 3 commit は保持）。Coordinator が range と publish 日を実測し、AC4 と Goal 失敗定義に「名指し package の subtree で npm が再解決する semver 内 transitive」を明示許可。他の非許可 package が動いた場合の停止条件は不変。
- 2026-09-15 Post-Merge Closeout（Coordinator）: Final Review 2 パス（Sonnet pass、findings 0 / Opus pass、P3 4 件）の完了を受け、Opus P3 の disposition を記録する。P3-1（packet `Implementation Results` が「未実装」のまま）と P3-2（`Plans.md` 本文の陳腐化）は本 closeout で解消した。P3-3（`40-cmd-product.md:279` の言い換えで field 名の明示が落ちた。per_page check の否定文脈検出を直す際に元の精度へ戻す）と P3-4（`ponytail:` comment を test 側だけでなく checker の `case` 行の直上にも置く）は backlog「記録目的」へ記録し、行動を待たない。relay 往復は 5 / 上限 2 を超過した。原因は Coordinator の発注品質 3 回（実装 run の発注書に packet 編集を含めた run 分離契約との衝突 / `:has(h1)` の詳細度の誤記 / AC4 で名指し package の subtree transitive 再解決を未考慮）で、Writer は毎回 fail-closed で正しく停止した。

## 後続

- override 撤去条件: markdownlint-cli2 が smol-toml ≥1.7.1 を pin する版を出したら override を外し、名指し通常更新と audit を再確認する。実装時に backlog「記録目的」へ1行残す。
- Cargo dismiss 理由候補（実行は owner）: rand = tolerable_risk「tauri-utils の build 時 hash 生成経路のみで runtime 露出なし」、glib = not_used「Linux 用 gtk 経路で Windows 配布物に含まれない」。実装時に backlog「記録目的」へ両件を1行で残す。
- per_page 上限 check の否定文脈検出は本 lane で直さない。
