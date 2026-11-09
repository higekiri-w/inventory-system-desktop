# Plan Packet: ㉓ 一括価格改定の取引先紐付けを既定 off + 文言明示（R2）

2026-09-16 起草。owner 決定（2026-09-16、PR #63 L3 round 3 の所感「価格改定のついでに未設定だった取引先が勝手に変わる」→ (a) 既定 off + (b) 文言明示の両方）。出典は Backlog「一括価格改定の取引先紐付けを既定 off + 文言明示」（owner 決定 2026-09-16）。㉒（`docs/plans/2026-09-15-display-fixes-batch-2.md`）と file footprint が互いに素で、並走可。file:line は origin/main `eabc549c` で実測（Coordinator 2026-09-16）。実装は別 run（Codex）とし、独立 Plan Review 通過後に発注する。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

- Evidence Mode: github
- Phase: archive
- Risk: R2
- Execution Mode: fable-window
- Plan Commit: a129080ca3985723c162e5a86b476683eda61cca
- Amendments: 3aedb032f144dd76063947c6f1f1776bd8b28269, 2d22b7d4b33550d97439a79421780ae7dab96651, 68962ddb4582aaabd4668113cdf5ba17689d6aab
- Coordinator: Fable 5.1
- Writer: Codex
- Plan Reviewer: Sonnet（独立 fresh context）
- Final Reviewer: Sonnet + Opus（独立 fresh context）
- Final Review Minimum: 2
- Human Gate: ready,merge,manual

manual = owner Windows native L3 1 画面（一括価格改定: 取引先選択後の toggle が off で表示され、文言が新文になっている。目視 2 点のみ。行確定時の挙動は unit test〈AC7〉で閉じる、Plan Review round 2）。

遷移記録（append-only）:
- kickoff → spec-check → plan-draft → plan-gate（本 commit）: Risk R2、owner 決定 (a) 既定 off + (b) 文言明示を Goal Invariant へ確定。Design Readiness は既存 77 §77.2 REQ-106/SPEC-PRV-D6・§77.6・30-biz §4.4.1 手順 5（不変の前提）を十分と引用（既定値と文言の同期のみ、新 component / 新 token なし）。Test Design Matrix は R2 で AC が機械 oracle + 既存 test の反転で閉じるため付けない。
- plan-gate → plan-approved → implementing（本 commit、state-only）: Plan Review round 1（Sonnet、P1 2 / P2 1 / P3 1）→ in-place 是正 `cb8050fa`（3 本目 test の期待値反転を S2 へ、AC-L3-1 を目視 2 点へ〈L3 Eligibility〉、D-088 の書式、参照ラベル）→ round 2 closure（新規 P1 1 = Workflow State の manual 説明の伝播漏れ）→ 是正 `a129080c` → round 3 closure = 通過可（P1/P2 = 0、round 天井 3 で終了）。Plan Commit = `a129080c`（plan-first `ff0f3e25` → 補正 `99607d45` → 是正 2 本を含む確定版）。実装は Codex 発注書 57 で本 commit を HEAD_SHA として開始する。

## Owner Effort Budget

- 介入回数上限: 3
- 実働時間上限: 20分
- relay 往復上限: 3（既定 2 から改訂、Gated Amendment 1。理由: 実装 run 2 回が Coordinator の発注書誤り〈commit 構成が packet Test Plan と不一致 / HEAD 照合先を worktree と明示せず〉で実装前に fail-closed 停止し、Writer の作業は 0。3 往復目は発注書 57 改訂 3 の実装 run。owner 承認 = 起動）
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R2

Reason:
利用者可視の既定値 1 箇所（toggle の `useState` 初期値と供給者変更時の戻し先）と label 文言 1 箇所を変える UI 変更で、BIZ（30-biz §4.4.1 `revise_product_price` 手順 5「未設定の商品だけに設定し既存値を上書きしない」）・CMD・DTO（`assign_supplier_id`）は不変。`PriceRevisionTable.tsx:75` は既存の `assignSupplier` 値をそのまま `assign_supplier_id` へ渡す構造を保つため、契約変更を伴わない。取引先ピッカー（DSR-24）や「取引先未設定の商品も含める」（別 toggle、既定 on のまま）も対象外。R3 に上げる契約変更はない。

## Goal

Goal Invariant: 一括価格改定の行確定で、利用者が明示的に選ばない限り商品の取引先を変更しない。既存の「未設定の商品だけに設定し既存値を上書きしない」BIZ 規則（30-biz §4.4.1 手順 5）は不変。

### 最小完了条件

- (1) toggle の既定が off、取引先を変えても off のまま（on へ戻さない）。
- (2) label 文言が「確定した商品の取引先が未設定なら、この取引先を設定する」。
- (3) 設計書 77 の REQ-106 / SPEC-PRV-D6 行と §「絞り込みと取引先の漸進補完」の該当 bullet が新既定と文言に同期。
- (4) test が新既定を固定。
- (5) decision-log に D-088。

### 失敗定義

BIZ / CMD / DTO（`assign_supplier_id`）の変更、`取引先未設定の商品も含める`（別 toggle、既定 on のまま）への波及、取引先ピッカー（DSR-24）の変更、`PriceRevisionTable` の受け渡し変更。

### 非目的

取引先の一括紐付け機能の新設、Toast / Alert の変更。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 起票時実測（origin/main `eabc549c`、Coordinator 計測 2026-09-16）

- `rg -c 'useState\(true\)' src/features/products/PriceRevisionPage.tsx` = 1（`:32`）/ `rg -c 'setAssignSupplier\(true\)' 同` = 2（`:34` useEffect、`:37` patchSearch）
- `rg -c '未設定の商品にこの取引先を設定する' src/features/products/PriceRevisionPage.tsx` = 1 / 同 `PriceRevisionPage.test.tsx` = 7
- `docs/function-design/77-ui-bulk-price-revision.md`: `:30` REQ-106 / SPEC-PRV-D6 行「紐付け toggle は既定 on。」= 1、`:93`「`未設定の商品にこの取引先を設定する` は取引先選択中だけ表示し、既定 on とする。」= 1、`:128`（§77.8 テスト観点の SPEC-PRV-D3 bullet、別 toggle）は不変
- test: `:186`「…既定 on で supplier 変更時に on へ戻る」/ `:217` 付近「browser 履歴で supplier search が変わった場合も取引先設定 toggle を既定 on に戻す」の 2 本が既定 on を固定
- `PriceRevisionTable.tsx:75` は `assignSupplier` を受けて `assign_supplier_id` を渡す（変更なし）
- decision-log の最終は D-087（2026-09-14）

## 設計判断（Coordinator adjudication、Plan Review で覆せる）

- **D-PR1 既定 off**（owner (a)）: `useState(false)`。利用者が on にした後に取引先を変えた場合も off へ戻す（`useEffect` と `patchSearch` の `setAssignSupplier(true)` を `false` に。別の取引先へ意図が持ち越されない）。捨てた案: on のまま文言だけ（不意の紐付けが残る）/ `useEffect` の撤去（on が別取引先へ持ち越される）
- **D-PR2 文言**（owner (b)）: 「確定した商品の取引先が未設定なら、この取引先を設定する」。checkbox の `id` / `htmlFor` は不変。捨てた案: 補足文の追加（1 行で読める文にする）
- **D-PR3 設計書の改訂は 77 だけ**: REQ-106 / SPEC-PRV-D6 行の「紐付け toggle は既定 on」→「紐付け toggle は既定 off（owner 2026-09-16、D-088）」+ 理由列に「価格改定のついでに取引先が変わる副作用を避ける」を 1 句。§ bullet を新既定・新文言・「取引先を変えても off のまま」へ。更新履歴 1 行。30-biz / 40-cmd は不変（契約は UI の既定値のみ）
- **D-PR4 decision-log D-088**: 「一括価格改定の取引先紐付けは既定 off で明示的に選ぶ（2026-09-16）」。書式は直近の D-087 と同じ（`## D-088: 一括価格改定の取引先紐付けは既定 off で明示的に選ぶ（2026-09-16）` の header + `- Status:` / `- Decision:` / `- Why:` / `- Compatibility:` の 4 field。日本語 field 名は使わない。Plan Review round 1 P2 で訂正）

## Scope

- **S1 `src/features/products/PriceRevisionPage.tsx`**: `true` → `false` の 3 箇所（`:32` `useState(false)`、`:34` `useEffect` 内 `setAssignSupplier(false)`、`:37` `patchSearch` 内 `setAssignSupplier(false)`）。構造（`useEffect` / `patchSearch` の分岐）は残す = 取引先を変えたら off へ戻る。`:76` の label 文言を新文へ
- **S2 `src/features/products/PriceRevisionPage.test.tsx`**: `:186` の test を「既定 off。on にしてから取引先を変えると off に戻る」へ反転（test 名に「既定 off」を含める）。`:217` 付近「browser 履歴で supplier search が変わった場合も…既定 on に戻す」を「既定 off に戻す」へ。文言参照 7 箇所を新文へ。**Plan Review round 1 P1-1: `:275` の test「確定は該当行 1 商品だけを reviseProductPrice に送り assign_supplier_id は取引先選択 + toggle on のとき supplier_id、それ以外 null」も対象。既定 off のため mount 直後（未クリック）の確定は `assign_supplier_id: null`、checkbox クリック後の確定は `assign_supplier_id: 7` へ期待値を入れ替える（DTO 契約 `assign_supplier_id` 自体は不変、test の期待値だけの反転。失敗定義の「DTO の変更」には当たらない）。**
- **S3 `docs/function-design/77-ui-bulk-price-revision.md`**: `:30` / `:93` / 更新履歴
- **S4 `docs/decision-log.md`**: D-088 を末尾に追加
- **S5（Coordinator、plan-first commit）**: Plans.md / backlog.md の登録

## Non-scope

- `src-tauri/**`
- `PriceRevisionTable.tsx`
- `PriceRevisionFilters.tsx`（「取引先未設定の商品も含める」は既定 on のまま）
- `SupplierPickerDialog`
- 30 / 40 の設計書
- catalog
- Toast / Alert

## Acceptance Criteria

rg oracle は出力空 = 0 件。baseline は起票時実測（origin/main `eabc549c`）。

- **AC1** `rg -c 'useState\(false\)' src/features/products/PriceRevisionPage.tsx` = 1（baseline 0）/ `rg -c 'setAssignSupplier\(true\)' 同` = 0（baseline 2）/ `rg -c 'setAssignSupplier\(false\)' 同` = 2（baseline 0）
- **AC2** `rg -c '確定した商品の取引先が未設定なら、この取引先を設定する' src/features/products/PriceRevisionPage.tsx` = 1（baseline 0）/ `rg -c '未設定の商品にこの取引先を設定する' src/features/products src/features/products/components` = 0（baseline 1 + test 7）
- **AC3** `rg -c '紐付け toggle は既定 off' docs/function-design/77-ui-bulk-price-revision.md` = 1（baseline 0）/ `rg -c '既定 on' docs/function-design/77-ui-bulk-price-revision.md` = 3（baseline 5 = `:27` `:30` `:92` `:93` `:128`。残る 3 = `:27` `:92` `:128` の「取引先未設定の商品も含める」〈SPEC-PRV-D3〉の既定 on、本 lane 非対象）/ `rg -c 'D-088' docs/function-design/77-ui-bulk-price-revision.md` ≥ 1
- **AC4** `rg -c '^## D-088' docs/decision-log.md` = 1（baseline 0）
- **AC5** `PriceRevisionPage.test.tsx` の 2 本が新既定を固定（test 名に「既定 off」を含む）: `rg -c '既定 off' src/features/products/PriceRevisionPage.test.tsx` ≥ 2（baseline 0）。mutant: ~~`useState(false)` → `true` で FAIL~~（**GA2 で訂正**: mount 後の `useEffect` が `false` に戻すため初期値だけの mutant は検出できない〈Codex run 3 で実測、27 test 全 PASS〉。正しい mutant は (1) `useEffect` 内 `setAssignSupplier(false)` → `true`〈mount 後に on へ戻り `:186` の `not.toBeChecked()` が落ちる〉。~~(2) `patchSearch` 内 …~~（**GA3 で撤回**: 取引先変更は `normalizedSearch.supplier` の変化で `useEffect` も発火し `false` へ上書きするため、`patchSearch` 側だけの変異は観測不能〈検証 run で 27 PASS を実測〉。mutant は (1) の 1 本で AC5 を閉じる。**Opus P2 の訂正（PR #67 pass B comment が正本）**: `patchSearch` の戻し行は同一取引先を選び直す経路（`normalizedSearch.supplier` が変わらず `useEffect` が発火しない）で単独に効く load-bearing 行。除去しない（Final Review pass B で反証: `renderStateful({ supplier: 7 })` → toggle on → ピッカーで同じ取引先を再選択 → `:37` あり = off に戻る / 除去 = on のまま）
- **AC6**（負の oracle）`git diff --name-only origin/main..HEAD -- src-tauri src/features/products/components docs/function-design/30-biz-product-service.md docs/function-design/40-cmd-product.md | wc -l` = 0
- **AC7** 対象 test（`PriceRevisionPage`）PASS、`npm run typecheck` / `lint` / `format:check` PASS、最終 `bash scripts/local-ci.sh full` PASS（`:275` の test の期待値反転を含む。反転前の test は既定 off で必ず FAIL するため、修正前後の red / green を報告する）
- **AC8** `bash scripts/doc-consistency-check.sh --target plan` ERROR 0 / `bash scripts/check-workflow-git.sh` PASS
- **AC-L3-1** 一括価格改定（Windows native）: 取引先を選ぶと toggle が **off** で表示され、文言が「確定した商品の取引先が未設定なら、この取引先を設定する」。**Plan Review round 1 P1-2: 「off のまま確定しても取引先が空のまま」は demo seed に supplier_id NULL の商品が無く、DB の直接編集（synthetic row / UPDATE）を要するため L3 Eligibility（DEV_WORKFLOW）により L3 から外し、unit test（`:275` の mount 直後の確定 = `assign_supplier_id: null`、AC7）で閉じる。L3 は目視 2 点（toggle が off で表示、文言が新文）のみ。**

## Design Sources

List the source design docs this plan relies on. Plan Packets are not durable design source of truth.

- Requirements / spec: REQ-106（77 §77.2）
- Architecture: 該当なし
- Function / command / DTO: 77 §77.2 SPEC-PRV-D6、§77.6「絞り込みと取引先の漸進補完」（`:93`）。30-biz §4.4.1 手順 5（`assign_supplier_id` は supplier_id が NULL のときだけ設定、既存値は上書きしない。不変の前提）
- DB: 該当なし
- Screen / UI: 77 §77.6（同上）
- Decision log / ADR: `docs/decision-log.md` の書式（D-088 を追加、S4）

## Required Design Artifacts

Use `docs/DEV_WORKFLOW.md` Design artifact selection to decide what must exist before implementation.

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | なし（30-biz §4.4.1 手順 5 は不変） | existing sufficient（not applicable） |
| Command / DTO / generated binding / wire shape | なし（`assign_supplier_id` 契約は不変） | existing sufficient（not applicable） |
| DB / transaction / audit / rollback / migration | なし | not applicable |
| Screen / UI / route state / Japanese wording | 77 §77.2 REQ-106/SPEC-PRV-D6 行、§77.6 | updated in this PR（S3） |
| CSV / TSV / report / import / export format | なし | not applicable |
| Durable decision / ADR | D-088 | updated in this PR（S4） |

## Registration / Generation Obligations

該当なし（command / route / doc 新設なし、REQ 追加なし、bindings 非接触）。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| REQ-106 | 77 §77.2 SPEC-PRV-D6 / §77.6 `:93` | D-PR1 | 既定 off。捨てた案: on のまま文言だけ（不意の紐付けが残る）/ `useEffect` 撤去（on が別取引先へ持ち越される） | S1 | `PriceRevisionPage.test.tsx` |
| REQ-106 | 77 §77.6 `:93` | D-PR2 | 文言「確定した商品の取引先が未設定なら、この取引先を設定する」。捨てた案: 補足文の追加 | S1 | `PriceRevisionPage.test.tsx` |
| REQ-106 | 77 §77.2 SPEC-PRV-D6 行 / §77.6 `:93` | D-PR3 | 設計書改訂は 77 だけ、30-biz / 40-cmd は不変 | S3 | — |
| — | `docs/decision-log.md` | D-PR4 | D-088 を末尾に追加 | S4 | — |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: yes（77 の該当行と decision-log D-088 を同 commit で更新）
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: D-088（decision-log、S4）
- Assumptions and constraints: 30-biz §4.4.1 手順 5「未設定の商品だけに設定し既存値を上書きしない」は不変（起票時実測で確認、変更しない）
- Deferred design gaps, risk, and follow-up target: なし
- Test Design Matrix can cite design decision IDs or source doc sections: yes（D-PR1〜D-PR4。R2 のため専用 Test Design Matrix は付けず、Test Plan に理由を記す）
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: `assign_supplier_id` の BIZ 規則不変を AC6（負の oracle、`src-tauri` / `PriceRevisionTable` の隣接 component / 30-biz / 40-cmd の diff 0 件）で機械検査

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | not applicable | — |
| Fact check / design decision split | 事実 = 起票時実測（`useState`/`setAssignSupplier` 箇所、文言箇所、77 の既定 on 記述、decision-log 最終 D-087）。判断 = D-PR1〜D-PR4 | 本 packet |
| Lifecycle / retry | not applicable | — |
| Operator workflow | 業務: 未設定商品への紐付けが opt-in になる。初年度の漸進補完の速度は落ちるが owner 受容（PR #63 L3 round 3 所感） | AC-L3-1 |
| Replacement path | not applicable | — |
| Data safety / evidence | データ: 既存 supplier_id の上書きなし、不変（30-biz §4.4.1 手順 5） | AC6 |
| Reporting / accounting semantics | not applicable | — |
| Manual verification | 1 画面の抜き取り | AC-L3-1 |
| 環境・再現性 | 互換: URL state 不変 | — |

## Design Readiness

State whether the design is ready for implementation.

- Existing design docs are sufficient because: 変更は既定値 1 箇所（`useState` 初期値と供給者変更時の戻し先）と label 文言 1 箇所の同期で、新しい振舞いはない。BIZ の「未設定の商品だけに設定」規則（30-biz §4.4.1 手順 5）は不変で、77 §77.2 REQ-106/SPEC-PRV-D6 行と §77.6 の bullet を新既定・新文言へ揃えれば十分。新 component / 新 token なし、mockup 不要
- Source docs updated in this PR: 77（S3）、decision-log D-088（S4）（いずれも実装 run で更新。本 plan-first commit は packet / Plans.md / backlog.md のみ）
- Design gaps intentionally deferred: なし
- Durable decisions discovered in this plan and promoted to source docs: D-088（decision-log）

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): UI のみ（`useState` 初期値と label 文言）。BIZ / CMD / DTO は不変
- Backend function design: 非接触
- Command / DTO / data contract: 非接触（`assign_supplier_id` 契約は不変、AC6）
- Persistence / transaction / audit impact: なし
- Operator workflow / Japanese UI wording: label 文言 1 箇所を新文へ（AC2）
- Error, empty, retry, and recovery behavior: 不変
- Testability and traceability IDs: REQ-106 / SPEC-PRV-D6、REQ 新設なし

## Contract Probe

N/A: 外部前提なし（library / OS 挙動に依存しない。`useState` 初期値と label 文言の変更のみ、R2）。

## Contract Coverage Ledger

R2 のため任意だが、docs 同期の網羅性を独立 review で確認できるよう置く。

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| REQ-106 / SPEC-PRV-D6（既定 off、文言） | S1 / S3 | `PriceRevisionPage.test.tsx` | AC-L3-1 |
| 30-biz §4.4.1 手順 5（未設定の商品だけに設定、既存値を上書きしない） | 非接触 | AC6（負の oracle） | — |

## Test Plan

For R3/R4, include or link a Test Design Matrix.

Test Design Matrix は付けない（R2、AC が機械 oracle〈rg count〉+ 既存 test の反転で閉じるため）。

- targeted tests: S1 / S2 の test を実装と同 commit で更新（AC5）
- negative tests: mutant（GA2 → GA3: `useEffect` の戻し先 `false` → `true` の 1 本）で AC5 が FAIL することを実装時に確認
- compatibility checks: `assign_supplier_id` 契約は不変（AC6 の負の oracle）
- data safety checks: not applicable
- main wiring/integration checks: not applicable（`useEffect` / `patchSearch` の分岐構造は不変）

## Boundary / Wire Contract

not applicable（DTO / wire 非接触。`assign_supplier_id` の契約は不変、AC6）。

## Review Focus

- S2 の裁定（供給者変更時に off へ戻す）が Goal Invariant と矛盾しないか。文言が 1 行で読めるか。77 の「既定 on」の残存 1 件が別 toggle であることの明記

## Implementation Results

[PR #67](https://github.com/higekiri-w/inventory-system-desktop/pull/67) で実装し squash merge 済み（`ebbbef14`、2026-09-16）。Writer = Codex 発注書 57（run 1 / 2 は Coordinator の発注書誤りで fail-closed、run 3 で実装 `462ebc87` `612f5f9a`）、検証・Draft PR = Sonnet run（source 編集なし）。GA1〜GA3、base 同期 `54e4c14f`。Final Review round 1 = Sonnet pass（P3 1）/ Opus P2 1 → 裁定合意で pass（`#issuecomment-5684502195`）。owner L3 round 1 PASS（`#issuecomment-5684432477`）。介入 1 / 予算 3、relay 3 / 上限 3

## Review Response

- Findings Freeze: not yet frozen; post-freeze exceptions: none.

### Plan Review round 1（2026-09-16、plan-gate、Sonnet、裁定 Coordinator）

- P1-1（`PriceRevisionPage.test.tsx:275` の 3 本目 test が既定 on 前提で `assign_supplier_id: 7` / `null` を期待し、Scope S2 に無い）= accept → S2 に期待値反転（mount 直後 = null、click 後 = 7。DTO 契約は不変）を追記、AC7 に red / green の報告を追記
- P1-2（AC-L3-1 の「off のまま確定しても取引先が空のまま」は demo seed に supplier_id NULL の商品が無く fixture 準備が要る）= 裁定: DB 直接編集を要するため L3 Eligibility（DEV_WORKFLOW）により L3 から外し、unit test（`:275`、AC7）で閉じる。L3 は目視 2 点のみ
- P2（D-PR4 の decision-log 書式「背景 / 決定 / 影響 / 代替案」が repo 慣行と不一致）= accept → D-087 と同じ `Status / Decision / Why / Compatibility` に訂正
- P3（起票時実測の `:131` Review Focus という非実在ラベル）= accept → `:128`（§77.8 テスト観点 SPEC-PRV-D3 bullet）に訂正
- 是正 `cb8050fa`。round 2 = closure

### Plan Review round 2（closure、Sonnet）

- round 1 の 4 件 = closed。P1-2 の裁定は DEV_WORKFLOW L3 Eligibility 条件 (3) と Human Visual Confirmation の規則に照らして妥当と確認
- 新規 P1: Workflow State の `manual =` 説明文が旧 3 点のまま（AC-L3-1 だけ直して伝播漏れ）= accept → 目視 2 点に揃える `a129080c`
- round 3 = closure（天井）

### Plan Review round 3（closure、Sonnet）

- `manual =` 行と AC-L3-1 の一致を確認、新規 finding なし。**Plan Gate 通過可**（P1/P2 = 0）

### Gated Amendment 1（2026-09-16、Codex 発注書 57 の fail-closed 停止 ×2）

- run 1（HEAD `64c155fa`）: 発注書が「test 先行を別 commit」と指示し、packet Test Plan「S1 / S2 の test を実装と同 commit で更新」と literal 衝突 → 実装前に停止（正しい挙動）。発注書を 2 commit 構成へ改訂
- run 2（同 HEAD）: 改訂後の発注書が HEAD 照合先を worktree と明示せず、pin した cwd（本体 clone、main `f2ef9e52`）で照合して不一致停止（正しい挙動）。発注書に `cd /tmp/codex-23-impl && git rev-parse HEAD` を明示
- Owner Effort Budget: relay 2 → 3（Coordinator 起因、Writer の作業 0）。packet の Scope / AC / 設計判断は不変
- 教訓: 発注書の手順節は packet Test Plan と突き合わせる。worktree を使う run では照合 command に path を含める

### Gated Amendment 2（2026-09-16、Codex 発注書 57 run 3 の AC5 停止）

- run 3（HEAD `2e010e5e`）: Writer は S1 / S2（`462ebc87`、red 3 failed → green 27 passed）と S3 / S4（`612f5f9a`）を実装したが、AC5 の mutant「`useState(false)` → `true`」が 27 test 全 PASS で生存し、packet 条件を満たせず停止（正しい挙動）。原因は Coordinator の mutant 設計誤り: mount 後の `useEffect` が既定 off へ戻すため、初期値だけを変えても挙動が変わらない
- 是正: AC5 / Test Plan の mutant を `useEffect` / `patchSearch` の戻し先 `false` → `true` の 2 本に訂正（本 commit）。実装 commit は不変
- 残作業（検証と公開のみ、source 編集なし）: 訂正後 mutant 2 本、AC7 の lint / format:check / `local-ci.sh full`、AC8、push 済み commit の Draft PR 作成。relay 上限 3 を使い切った（3 run とも Coordinator 起因）ため、これらは Coordinator 側の Sonnet run で行う（Writer の実装成果物は変更しない。Codex は Final Review の是正が必要になった場合に owner 承認で再依頼）

### Gated Amendment 3（2026-09-16、検証 run の AC5 finding）

- 検証 run（Sonnet、head `d25443df`）: AC1〜AC4 / AC6〜AC8 PASS、`local-ci.sh full` PASS。AC5 の GA2 mutant (1) `useEffect` は 3 test FAIL で kill、(2) `patchSearch` は 27 PASS で生存
- 裁定: (2) は `useEffect` が同じ戻しを独立に行うため観測不能（mutant 設計の誤り、実装・test の欠陥ではない）。AC5 の mutant を (1) の 1 本に訂正（本 commit）。**Opus P2 の訂正（PR #67 pass B comment が正本）**: `patchSearch` の `setAssignSupplier(false)`（`PriceRevisionPage.tsx:37`）は同一取引先を選び直す経路で単独に効く load-bearing 行で、除去しない。本 lane では S1「構造は残す」のまま
- Draft PR #67 作成済み。次: base 同期（main の #64 / #66 取込み）→ Final Review（Sonnet + Opus）→ owner L3 目視 2 点 → record → Ready

### Final Review round 1（2026-09-16、head `54e4c14f`）

- pass A（Sonnet）: P3 1 = PR body の GA3 反映漏れ → body 追記で対応
- pass B（Opus）: P2 1 = GA3 記録の誤り（`patchSearch` の戻し行を除去可能と記録）→ 上記訂正（AC5 / GA3 節）。P3 = `docs/architecture/ui-task-specs.md:153` の旧「既定 on」記述 → 本 closeout（docs/design 同期 commit）で是正。P3 = `useState(false)` 単独 mutant は観測不能 → GA2 で記録済み、対応不要
- 裁定: Opus の指摘を全件 accept、実装は不変（`:37` は load-bearing 行）。**Opus の合意条件 2 点**: (1) 反証の再現手順（`renderStateful({ supplier: 7 })` → toggle on → ピッカーで同じ取引先を再選択 → `:37` あり = off / 除去 = on）を packet に記録する。(2) 「同一取引先の再選択で off へ戻ることを固定する test を 1 本」を Backlog へ起票する際、現行挙動の固定であり owner が別を望めば test ごと変える旨の但し書きを付ける
- review = pass（broad 2 audits @ `54e4c14f`、`#issuecomment-5684502195`）。manual = pass（owner L3 round 1、`#issuecomment-5684432477`）

## 後続

- Backlog「やると決めたもの（順番未定）」の同一取引先再選択 test 起票（Final Review pass B の合意条件 2、下記 3. 参照）
