# Plan Packet: ㉕ 廃棄・破損の保存結果に「詳細を見る」+ `returnTo` を追加（UI-05-D17 改訂、R3）

2026-09-16 起草。owner 2026-09-11「やったほうがいい」で格上げ（起源: PR #23 owner L3 所感 2026-08-31「廃棄のみ保存結果に詳細 link なし」）。出典は Backlog「やると決めたもの（順番未定）」の「廃棄・破損の保存結果に『詳細を見る』+ `returnTo` を追加するか」。wave 11 の lane 2 で、lane 1 ㉔（`docs/plans/2026-09-16-home-mockup-c-runtime.md`、ホーム）と file footprint が互いに素。file:line は origin/main `9d6799ff` で実測（Coordinator 2026-09-16）。実装は別 run（Codex）とし、独立 Plan Review 通過後に発注する。Test Design Matrix: `docs/plans/test-matrices/2026-09-16-disposal-result-detail-link.md`。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

- Evidence Mode: github
- Phase: archive
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: fe8ff219bc8d46af64d9d9d74c461c28347c0549
- Amendments: 4c7b32f8dd72b8ed96f1f70bd62e02390827d01d, d4c5fb947ebfc90c6a08d1c098def0fb5c11a530, c44a1f3ae4dae03924a2e95716b373667c6baa3a
- Coordinator: Fable 5.1
- Writer: Sonnet（GA3。Codex relay 3/3 到達後、subagent、worktree 分離、Plan Reviewer とは別 fresh context）
- Plan Reviewer: Sonnet（独立 fresh context）
- Final Reviewer: Sonnet + Opus（独立 fresh context）
- Final Review Minimum: 2
- Human Gate: ready,merge,manual

manual = owner Windows native L3 1 往復（廃棄・破損を 1 件保存 → 保存結果の「詳細を見る」→ 詳細の「前の画面へ戻る」で廃棄・破損画面へ戻る。目視と PASS/FAIL のみ。fixture 不要 = demo seed の任意の商品で廃棄を 1 件作れる）。

遷移記録（append-only）:
- kickoff → spec-check → design → plan-draft → plan-gate（本 commit）: Risk R3（route/search state の producer を 1 site 追加。R2/R3 で迷う場合は R3 の規則）。Design Phase = UI-05-D17 の改訂を design 判断 D-D1 として 64 の決定表で先行し（実装 run で S3 として更新）、DSR-18 / 65 TRACE-D11（「保存結果」を producer に含む横断契約）は不変。Test Design Matrix を同 commit で置く。
- plan-gate → plan-approved → implementing（本 commit、state-only）: Plan Review round 1（Sonnet、P2 1 / P3 1）→ in-place 是正 `fe8ff219`（64 の節番号 §64.8 → §64.9、T8 の行番号）→ round 2 closure（Sonnet、新規 P1/P2 なし、`rg '64\.8'` = 0 / `':335'` = 0 を確認）= 通過可。Plan Commit = `fe8ff219`（plan-first `31d9643d` → 是正を含む確定版）。実装は Codex 発注書 59 で本 commit を HEAD_SHA として開始する。

## Owner Effort Budget

- 介入回数上限: 3
- 実働時間上限: 20分
- relay 往復上限: 3（既定 2 から改訂、Gated Amendment 2。理由: run 1 = AC1 の count 誤記、run 2 = Test Plan / Matrix の「実 router context」誤記で、いずれも Coordinator 起因の fail-closed 停止で Writer の作業は 0。3 往復目は発注書 59 改訂 3 の実装 run。owner 承認 = 起動）
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
業務記録詳細 route へ `returnTo`（search state）を送る producer を 1 site（廃棄・破損の保存結果 panel）追加する。DSR-18 の送信契約は既存で、consumer（`DisposalRecordDetailPage` の `returnTo` 正規化 = `src/lib/return-to.ts` 共通 helper + `/inventory/records` fallback）は不変。route 定義・DTO・BIZ は不変。UI route/search behavior に触れるため Risk Tiers の規則で R3 とし、Test Design Matrix / Spec Contract / Trace Matrix / Data Safety を置く。R4 要素（データ lifecycle・実データ）はない。

## Goal

Goal Invariant: 廃棄・破損を保存した直後に、その記録の詳細へ「詳細を見る」で移動し、詳細の「前の画面へ戻る」で廃棄・破損画面（search state 込み）へ戻れる。入庫（UI-02-D16）・返品・交換（UI-03-D22）の保存結果と同じ導線になる。

### 最小完了条件

- (1) 保存結果 panel（`DisposalPage.tsx:333-345`）に「詳細を見る」link（`to="/inventory/disposal/records/$recordId"`、`params={{ recordId: String(result.record_id) }}`、`search={{ returnTo }}`）が出る。並び順 = 続けて廃棄・破損 / 詳細を見る / 在庫照会へ戻る（入庫 `ReceivingPage.tsx:351-369` と同順）。
- (2) 64 の UI-05-D17 行・§64.5 bullet・§64.9 Test Focus bullet が「保存結果と recent list」の producer 契約に改訂され、変更履歴に 1 行。
- (3) test T8（`DisposalPage.test.tsx:334`）が「link あり + href に `returnTo`」へ反転して PASS。

### 失敗定義

detail route / `src/lib/return-to.ts` / `DisposalRecordDetailPage` の変更、recent list（`DisposalPage.tsx:708-716`）の変更、他画面（入庫・返品交換・手動販売）の変更、DSR-18 本文（`01-decision-rules.md`）の変更、`idempotent_replay` 時に link を出さない分岐の追加（再送結果でも `record_id` は同じ記録を指すため link を出す）。

### 非目的

保存結果 panel が詳細往復で消える問題（DSR-19 系、Backlog 別項）、手動販売の `sale_id` 条件付き link の変更、保存結果 panel の他の項目・文言。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 起票時実測（origin/main `9d6799ff`、Coordinator 計測 2026-09-16）

- `src/features/disposal/DisposalPage.tsx`: `:134` `const returnTo = useRouterState({ select: (state) => state.location.href })`（既存、recent list 用）/ `:333-345` 保存結果 panel の button 群 = 「続けて廃棄・破損」+「在庫照会へ戻る」の 2 つ / `:708-716` recent list の「詳細を見る」（`to="/inventory/disposal/records/$recordId"` + `search={{ returnTo }}`、`Eye` icon）/ `rg -c '詳細を見る'` = 1、`rg -c 'search=\{\{ returnTo \}\}'` = 1。`Eye` / `Link` は import 済み（`:6` `:8`）
- `src/features/receiving/ReceivingPage.tsx:351-369`: 保存結果の button 順 = 続けて入庫 / 詳細を見る（`Eye`、`variant="outline"`、`asChild` Link + `search={{ returnTo }}`）/ 在庫照会へ戻る。本 lane の見本
- `src/features/disposal/DisposalPage.test.tsx`: 対象 test file 全 PASS（本数は PR body / CI 出力を正とする。GA1〜GA3 で本数の書き写し誤りが 3 回発生したため固定値として記録しない。closeout の是正、Final Review pass B P3-3）。`:334` T8「saved disposal result does not add a detail link」= `queryByRole("link", { name: "詳細を見る" })` が不在を固定（record_id 41）/ `:462-466` recent list の href `/inventory/disposal/records/12?returnTo=%2Finventory%2Fdisposal` を固定 / `rg -c '詳細を見る'` = 3
- consumer: `src/features/inventory-records/DisposalRecordDetailPage.test.tsx:129` 「REQ-207 / T11 DSR-18: DisposalRecordDetailPage の returnTo %s を安全に %s へ正規化する」（欠落・不正・外部 URL の fallback を既に固定。本 lane 非接触）
- route: `src/routes/inventory/disposal/records/$recordId.tsx` 実在（非接触）
- docs: `64-ui-disposal.md:37` UI-05-D17 行「保存結果には詳細 link がないため、この送信契約の producer に含めない」/ `:120` §64.5 bullet「『詳細を見る』は UI-05-D17 に従って…」（recent list の文脈）/ `:165` §64.9 Test Focus「保存結果には詳細 link を追加しない」/ `:171` 変更履歴（2026-08-30、非遡及で残す）。`rg -c '保存結果には詳細 link がない|保存結果には詳細 link を追加しない|保存結果は詳細 link なし'` = 3（`:37` `:165` `:171`）/ `rg -c '保存結果と recent list'` = 0
- `65-inventory-record-traceability.md:43` TRACE-D11 は「一覧、作業画面の recent list、保存結果、操作ログ」を producer に含む横断契約（不変）。`61-ui-receiving.md:36` UI-02-D16「保存結果と recent list の『詳細を見る』は…送る」= 改訂文の同型
- `ui-task-specs.md:197` UI-05 行に導線の記述なし（不変）

## 設計判断（Coordinator adjudication、Plan Review で覆せる）

- **D-D1 UI-05-D17 を「保存結果と recent list」へ改訂**: 決定列を「保存結果と recent list の『詳細を見る』は、search state を含む現在の `/inventory/disposal` URL を `returnTo` として廃棄・破損詳細へ送る（DSR-18）」に、理由列を「詳細確認後に廃棄・破損作業画面へ戻れるようにする。保存直後の確認導線を入庫・返品・交換と対称にする（owner L3 所感 2026-08-31 / 2026-09-11）。廃棄だけ保存結果に link を出さない業務理由は source docs に無い。送信・検証・fallback の横断規範は DSR-18 を正とする」に改める。捨てた案: 現行契約の維持（非対称のまま）/ 保存結果を詳細画面へ自動遷移（DSR-19 の result panel 方針と衝突）
- **D-D2 実装は入庫の保存結果 link のコピー**: `ReceivingPage.tsx:355-363` と同じ `Button asChild variant="outline"` + `Link`（`Eye` icon）。`returnTo` は `:134` の既存値を再利用し、新しい state を作らない。`idempotent_replay` でも link を出す（`record_id` は既存記録を指す）
- **D-D3 decision-log は追加しない**: 契約の正本は 64 の決定表（UI-05-D17）で、横断規範 DSR-18 / TRACE-D11 は既に「保存結果」を producer に含む。durable な新判断ではなく既存規範への適合

## Scope

- **S1 `src/features/disposal/DisposalPage.tsx`**: `:333-345` の button 群に「詳細を見る」を追加（D-D2、順序 = 続けて廃棄・破損 / 詳細を見る / 在庫照会へ戻る）
- **S2 `src/features/disposal/DisposalPage.test.tsx`**: `:334` T8 を「T8 UI-05-D17: saved disposal result links to the detail with returnTo」へ反転し、`getByRole("link", { name: "詳細を見る" })` の `href` = `/inventory/disposal/records/41?returnTo=%2Finventory%2Fdisposal` を固定（recent list の `:462` と同型。recent list を同時に描画する test では link が 2 本になり得るため、保存結果 panel 内に scope して取る）
- **S3 `docs/function-design/64-ui-disposal.md`**: `:37` UI-05-D17 行（D-D1）/ `:120` §64.5 bullet を「保存結果と recent list の『詳細を見る』は UI-05-D17 に従って…」へ / `:165` §64.9 Test Focus bullet を「保存結果と recent list の『詳細を見る』が現在の廃棄・破損画面 URL を `returnTo` として送る」へ / 変更履歴に「2026-09-16 | ㉕ | UI-05-D17 を改訂し、保存結果にも詳細 link（`returnTo` 送信）を追加。入庫・返品・交換と対称化」
- **S4（Coordinator、plan-first commit）**: Plans.md / backlog.md の登録、Test Design Matrix

## Non-scope

- `src/routes/**`、`src/lib/return-to.ts`、`src/features/inventory-records/**`
- `src/features/receiving/**`、`src/features/return-exchange/**`、`src/features/manual-sale/**`
- `docs/design-system/01-decision-rules.md`（DSR-18）、`65-inventory-record-traceability.md`（TRACE-D11 は既に保存結果を含む）
- `DisposalPage.tsx` の recent list / form / mutation / invalidation
- `src-tauri/**`

## Acceptance Criteria

rg oracle は出力空 = 0 件。baseline は起票時実測（origin/main `9d6799ff`）。

- **AC1** `rg -c '詳細を見る' src/features/disposal/DisposalPage.tsx` = 2（baseline 1）/ `rg -c 'search=\{\{ returnTo \}\}' 同` = 2（baseline 1）/ `rg -c 'useRouterState' 同` = 2（baseline 2 = `:6` import + `:134` 呼び出し。**GA1 で 1 → 2 に訂正**、新 state なし）
- **AC2** `rg -c 'does not add a detail link' src/features/disposal/DisposalPage.test.tsx` = 0（baseline 1）/ `rg -c 'records/41\?returnTo=%2Finventory%2Fdisposal' 同` = 1（baseline 0）/ `rg -c '詳細を見る' 同` ≥ 3（baseline 3）
- **AC3** `rg -c '保存結果には詳細 link がない|保存結果には詳細 link を追加しない' docs/function-design/64-ui-disposal.md` = 0（baseline 2 = `:37` `:165`。`:171` の変更履歴「保存結果は詳細 link なしの現行契約を維持」は非遡及で残す）/ `rg -c '保存結果と recent list' 同` ≥ 2（baseline 0）/ `rg -c 'UI-05-D17 を改訂' 同` = 1（baseline 0）
- **AC4**（負の oracle）`git diff --name-only origin/main..HEAD -- src/routes src/lib src/features/inventory-records src/features/receiving src/features/return-exchange src/features/manual-sale src-tauri docs/design-system docs/function-design/65-inventory-record-traceability.md | wc -l` = 0
- **AC5** mutant: (1) S1 の link から `search={{ returnTo }}` を外す → S2 の href assert が FAIL（`?returnTo=` 不在）。(2) S1 の link を消す → S2 が FAIL。(3) `params` の `record_id` を `item.id` 等の別値にすると `/records/41` 不一致で FAIL。3 本とも実装後に kill を実測して報告する
- **AC6** 対象 test file（`DisposalPage`）全 PASS（本数は PR body / CI 出力を正とする。closeout の是正、Final Review pass B P3-3）、`npm run typecheck` / `lint` / `format:check` PASS、最終 `bash scripts/local-ci.sh full` PASS。反転前の T8 は link 追加で必ず FAIL するため、修正前後の red / green を報告する
- **AC7** `bash scripts/doc-consistency-check.sh --target plan` ERROR 0 / `bash scripts/check-workflow-git.sh` PASS
- **AC-L3-1** 廃棄・破損（Windows native）: 商品を 1 件追加して保存 → 保存結果に「詳細を見る」が出る → click で廃棄・破損詳細 → 「前の画面へ戻る」で `/inventory/disposal` に戻る（1 往復）。PASS/FAIL のみ

## Design Sources

List the source design docs this plan relies on. Plan Packets are not durable design source of truth.

- Requirements / spec: REQ-204（廃棄・破損）、REQ-206（記録の追跡・戻り導線）
- Architecture: 該当なし
- Function / command / DTO: 64 §64.1 決定表 UI-05-D11（保存結果の内容）/ UI-05-D17（本 lane で改訂）、§64.5、§64.9 Test Focus。61 §UI-02-D16（同型の先例）。65 TRACE-D11（横断契約、不変）
- DB: 該当なし
- Screen / UI: `docs/design-system/01-decision-rules.md` DSR-18（送信・検証・fallback の規範、不変）、DSR-19（保存成功の feedback、result panel 方針）
- Decision log / ADR: なし（D-D3）

## Required Design Artifacts

Use `docs/DEV_WORKFLOW.md` Design artifact selection to decide what must exist before implementation.

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | なし | not applicable |
| Command / DTO / generated binding / wire shape | なし | not applicable |
| DB / transaction / audit / rollback / migration | なし | not applicable |
| Screen / UI / route state / Japanese wording | 64 §64.1 UI-05-D17 / §64.5 / §64.9 | updated in this PR（S3） |
| CSV / TSV / report / import / export format | なし | not applicable |
| Durable decision / ADR | DSR-18 / TRACE-D11（既存、保存結果を producer に含む） | existing sufficient |

## Registration / Generation Obligations

該当なし（command / route / doc 新設なし、REQ 追加なし、bindings 非接触、`generate:routes` 不要〈route file 非接触〉）。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| REQ-204 / REQ-206 | 64 §64.1 UI-05-D17 / 65 TRACE-D11 / DSR-18 | D-D1 | 保存結果も producer に含め、入庫・返品交換と対称化。捨てた案: 非対称の維持 / 自動遷移 | S1 / S3 | T8' |
| REQ-206 | 61 UI-02-D16（先例） | D-D2 | 入庫の保存結果 link と同じ実装、新 state なし、replay でも link | S1 | T8' / 既存 recent list test |
| — | DSR-18 / TRACE-D11 | D-D3 | decision-log 追加なし（既存規範への適合） | — | — |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: yes（64 UI-05-D17 の改訂文と理由列に owner 所感の起源と対称化の理由を置く）
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: D-D1 → 64 決定表（S3）
- Assumptions and constraints: consumer の `returnTo` 正規化（`return-to.ts` + `DisposalRecordDetailPage.test.tsx:129`）は既存 test で fallback が固定済み。本 lane は producer 追加のみ
- Deferred design gaps, risk, and follow-up target: 保存結果 panel が詳細往復で消える（DSR-19 系、Backlog 別項）は本 lane で扱わない。詳細から戻ると form は初期状態になる（入庫・返品交換と同じ既知挙動）
- Test Design Matrix can cite design decision IDs or source doc sections: yes（Matrix 参照）
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: consumer / route / 他画面の不変を AC4（負の oracle）で機械検査。`returnTo` は URL 由来だが送信側は現在 href をそのまま渡し、検証は consumer 側（不変）

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | not applicable | — |
| Fact check / design decision split | 事実 = 起票時実測（button 群、recent list の link 形、T8、64 の 3 箇所）。判断 = D-D1〜D-D3 | 本 packet |
| Lifecycle / retry | result panel の表示中だけ link が出る。`resetForm` で `result = null` → link 消滅。`idempotent_replay` でも出る | Matrix State Lifecycle |
| Operator workflow | 保存直後に記録を確認して作業画面へ戻れる（入庫・返品交換と同じ手順） | AC-L3-1 |
| Replacement path | not applicable | — |
| Data safety / evidence | 表示のみ。L3 は開発 DB の synthetic 廃棄 1 件 | Data Safety |
| Reporting / accounting semantics | not applicable | — |
| Manual verification | 1 往復の目視 | AC-L3-1 |
| 環境・再現性 | URL: `/inventory/disposal/records/:id?returnTo=...` は既存 route と既存 search param | — |

## Design Readiness

State whether the design is ready for implementation.

- Existing design docs are sufficient because: DSR-18 と 65 TRACE-D11 が「保存結果」を producer に含む横断契約を既に定め、61 UI-02-D16 に同型の先例がある。64 UI-05-D17 だけが「保存結果は producer 外」と例外を固定しており、その 1 行の改訂（D-D1）が本 lane の design 出力
- Source docs updated in this PR: 64 §64.1 / §64.5 / §64.9 / 変更履歴（S3。実装 run で更新。本 plan-first commit は packet / Matrix / Plans.md / backlog.md のみ）
- Design gaps intentionally deferred: result panel が詳細往復で消える件（Backlog）
- Durable decisions discovered in this plan and promoted to source docs: D-D1（64）

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): UI のみ（link 1 本）
- Backend function design: 非接触
- Command / DTO / data contract: 非接触
- Persistence / transaction / audit impact: なし
- Operator workflow / Japanese UI wording: 「詳細を見る」（既存 label と同一）
- Error, empty, retry, and recovery behavior: `returnTo` 欠落・不正時の fallback は consumer 既存（不変）
- Testability and traceability IDs: REQ-204 / REQ-206 / UI-05-D17、REQ 新設なし

## Contract Probe

N/A: 外部前提なし（TanStack Router の `Link` + `search` は recent list の同 route で実証済み、R3 だが library 挙動の新規依存はない）。

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| UI-05-D17（改訂: 保存結果の `returnTo` 送信） | S1 / S3 | T8'（S2） | AC-L3-1 |
| UI-05-D17（recent list、不変） | 非接触 | 既存 `:462-466` | — |
| DSR-18 consumer（正規化 / fallback、不変） | 非接触 | 既存 `DisposalRecordDetailPage.test.tsx:129` | — |
| 他画面の producer 不変 | 非接触 | AC4（負の oracle） | — |

## Test Plan

For R3/R4, include or link a Test Design Matrix.

Matrix: `docs/plans/test-matrices/2026-09-16-disposal-result-detail-link.md`。

- targeted tests: S2 を実装と同 commit で反転（AC2 / AC6、red → green）
- negative tests: mutant 3 本（AC5）。`returnTo` 欠落・不正の fallback は consumer 既存 test（本 lane 非接触）
- compatibility checks: recent list の href 固定 test（`:462-466`）が無変更 PASS、AC4
- data safety checks: Data Safety 参照
- main wiring/integration checks: `DisposalPage.test.tsx` は `@tanstack/react-router` を mock（`:22-45`。`Link` は `to` / `params` / `search` から `<a href>` を組み立て、`useRouterState` は `/inventory/disposal` を返す）した unit test で、`renderWithClient` は QueryClientProvider のみ。href の oracle は recent list の既存 test（`:462`）と同じ mock 上で成立する。実 router の配線（route 解決 + 詳細の「前の画面へ戻る」）は AC-L3-1 が担う（**GA2 で「実 router context」の誤記を訂正**）

## Boundary / Wire Contract

browser state（`returnTo` search param）を扱うため記入する。

- producer: `DisposalPage` 保存結果 panel の `Link`（本 lane で追加）。既存 producer = 同 page の recent list
- consumer: `/inventory/disposal/records/$recordId` route → `DisposalRecordDetailPage`（`return-to.ts` 共通 helper、不変）
- wire type: URL search param `returnTo`（string、URL-encoded href）
- internal type: `string`（`useRouterState` の `location.href`）
- precision/range: not applicable
- round-trip path: `/inventory/disposal`（search state 込み）→ `/inventory/disposal/records/:id?returnTo=<encoded>` → 「前の画面へ戻る」→ `returnTo` の URL
- invalid input: 欠落・`/` 始まりでない・`//` 始まりは consumer が `/inventory/records` へ fallback（既存、不変）
- compatibility: additive（既存 deep link と recent list の挙動は不変）

## Review Focus

- S2 の link 取得が保存結果 panel に scope されているか（同 test で recent list を描画すると「詳細を見る」が 2 本になる）。`idempotent_replay` 時に link を出す判断（D-D2）が 64 UI-05-D11 の「`idempotent_replay` の有無を表示」と矛盾しないか。64 の 3 箇所 + 変更履歴の同期漏れ（AC3）

## Spec Contract

Required for R3/R4.

Contract ID: SPEC-UI05-D17-R1

- 廃棄・破損の保存結果 panel の「詳細を見る」は、search state を含む現在の `/inventory/disposal` URL を `returnTo` として `/inventory/disposal/records/$recordId` へ送る（Test: T8'）
- recent list の「詳細を見る」の送信契約は不変（Test: 既存 `DisposalPage.test.tsx:462-466`）
- consumer の `returnTo` 正規化と `/inventory/records` fallback は不変（Test: 既存 `DisposalRecordDetailPage.test.tsx:129`、evidence: AC4）
- `idempotent_replay` の保存結果でも同じ link を出す（Test: T8' の派生 1 本 or review/evidence: 実装 diff で分岐なしを確認）

## Trace Matrix

Required for R3/R4.

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-UI05-D17-R1（保存結果 producer） | S1 / S3 | T8' | link の scope、順序 | AC1 / AC2 / AC3 |
| SPEC-UI05-D17-R1（recent list 不変） | — | `:462-466` 既存 | 無変更 | AC1（count 2 のうち 1 は既存） |
| SPEC-UI05-D17-R1（consumer 不変） | — | `DisposalRecordDetailPage.test.tsx:129` 既存 | 非接触 | AC4 |
| SPEC-UI05-D17-R1（replay でも link） | S1 | T8' 派生 or diff review | 分岐なし | Matrix |

## Data Safety

Required for R3/R4.

- 実店舗データ・実商品名・実 JAN を test / packet / PR に含めない（test は既存 synthetic fixture `record_id: 41` 等）
- L3 は開発 DB（demo seed）で synthetic 廃棄 1 件。本番 DB / backup を使わない
- local-only / synthetic-only path の追加なし

## Implementation Results

[PR #71](https://github.com/higekiri-w/inventory-system-desktop/pull/71) で実装し squash merge 済み（`949a5c4c`、2026-09-16）。Writer = Sonnet subagent（Codex 発注書 59 は run 1〜3 とも packet 記述誤りで実装前に fail-closed、GA1〜GA3）。実装 `e307b53a`、docs `0c49b5af`。Final Review = Sonnet pass A（Contract Audit、finding 0）/ Opus pass B（P3 3 → closeout で是正）。owner L3 round 1 PASS（Coordinator session で報告、helper manual record）。介入 1 / 予算 3、Codex relay 3 / 上限 3

## Review Response

- Findings Freeze: not yet frozen; post-freeze exceptions: none.

### Plan Review round 1（2026-09-16、plan-gate、Sonnet、裁定 Coordinator）

- P2（packet が 64 の `:165` を「§64.8」と記載、実際は §64.9 Test Focus。§64.8 は Non-scope / Follow-up `:140-149`）= accept → 起票時実測 / 最小完了条件 (2) / S3 / Design Sources / Required Design Artifacts / Design Readiness の「§64.8」を「§64.9」へ一括是正（本 commit）。line 番号 `:165` と AC3 の rg oracle は正しく、実装対象は不変
- P3（T8 の `it(` 開始行は `:334`、packet は `:335`）= accept → packet / Matrix の `:335` を `:334` へ
- reviewer 確認済み: D-D1〜D-D3 の整合、Risk R3 判定、Adjacent Pattern Audit の網羅（保存結果 + 業務記録 detail route を持つ画面は 4 つのみ）、T8 単体では recent list が空 mock のため link は 1 本（S2 の scope 方針は安全側）
- 判定: 通過可（P1/P2 = 0 は本是正の closure 確認後）。round 2 = closure

### Plan Review round 2（closure、Sonnet）

- round 1 の P2 / P3 = closed（packet / Matrix で `64.8` 0 件、`:335` 0 件、round 1 節の記録が指摘と一致）。契約整合に影響する変更なし（diff は節番号・行番号のラベル訂正と round 1 節の追記のみ）。新規 P1/P2 なし。**Plan Gate 通過可**

### Gated Amendment 1（2026-09-16、Codex 発注書 59 run 1 の fail-closed 停止）

- run 1（HEAD `0ee89c14`）: Writer は AC1 の `rg -c 'useRouterState' src/features/disposal/DisposalPage.tsx` を実行し、実測 2（`:6` import + `:134` 呼び出し）が packet の期待値 1 と不一致のため実装前に停止（正しい挙動）。file 編集なし、worktree 除去済み
- 原因: Coordinator の起票時実測の誤り（`rg -n 'returnTo'` の目視で呼び出し行だけを数え、`rg -c` を実行していなかった）
- 是正: AC1 の期待値と baseline を 2 へ訂正（本 commit）。Scope / 設計判断 / 他の AC は不変
- Owner Effort Budget: relay 1/2 を消費（Coordinator 起因）。発注書 59 改訂 2 は本 GA の登録 commit を HEAD_SHA にする
- 教訓: AC の rg oracle は起票時に同じ command を実行して数値を写す（目視の行数で代用しない）

### Gated Amendment 2（2026-09-16、Codex 発注書 59 run 2 の fail-closed 停止）

- run 2（HEAD `0e0e8df8`）: Writer は packet Test Plan `:216` と Matrix T8' 行の「実 router context（`renderWithClient`）」が現物（`DisposalPage.test.tsx:22-45` で `@tanstack/react-router` を mock、`renderWithClient` は QueryClientProvider のみ）と不一致のため実装前に停止（正しい挙動）。環境準備（`npm ci` / routes 生成差分なし）まで実施、file 編集なし、worktree 除去済み
- 原因: Coordinator の未検証 claim（test の render helper 名から実 router と推定し、mock 節を読んでいなかった）
- 是正: Test Plan と Matrix の検証方式を「既存 router mock による unit 検証 + 実 router 配線は L3」へ訂正（本 commit）。href の oracle（FM1〜FM3 の検出力）は mock の `Link` が `search` から query を組み立てるため不変。Scope / AC / 設計判断は不変
- Owner Effort Budget: relay 上限 2 → 3（Coordinator 起因 2 回、Writer の作業 0）。発注書 59 改訂 3 は本 GA の登録 commit を HEAD_SHA にする。owner 承認 = 起動
- 教訓: packet に書く test の前提（render helper / mock 境界）は test file の `vi.mock` 節を読んで写す

### Gated Amendment 3（2026-09-16、Codex 発注書 59 run 3 の AC6 停止）

- run 3（HEAD `0c256b31`）: Writer は T8 反転の red（T8 のみ FAIL、24 PASS）まで確認したが、AC6 の「23 本」が vitest 実測 25 本（`it` 23 + `it.each` 2 ケース）と不一致のため原状復帰して停止（正しい挙動）。commit 0、worktree 除去済み
- 原因: Coordinator が `rg -c '^\s*it\('` の値を写し、`it.each`（`:712`）のケース数を数えていなかった
- 是正: 起票時実測と AC6 を 25 本へ訂正（本 commit）。Scope / 設計判断 / 他の AC は不変
- Owner Effort Budget: Codex relay 3/3 到達（3 回とも Coordinator 起因、Writer の作業 0）。残作業（S1〜S3 の実装 + 検証 + Draft PR）は ㉓ GA2 と同じく Coordinator 側の Sonnet subagent run で行い、Writer field を Sonnet に改める。Codex は Final Review の是正が必要になった場合に owner 承認で再依頼
- 教訓: test 本数は `vitest run <file>` の出力から写す（`it(` の rg count は `it.each` を数えない）

### Final Review round 1（2026-09-16、head `0c49b5af`）

- pass A（Sonnet）: AC oracle 再実測一致、mutant 1 本を実注入して kill を確認、finding 0（[comment](https://github.com/higekiri-w/inventory-system-desktop/pull/71#issuecomment-5694394631)）
- pass B（Opus）: P3 3 件 = (1) 64 `:31` `:64` の導線列挙に「詳細を見る」未追加、(2) `:120` の二重記述（「保存結果と最近の廃棄・破損一覧の」が `:119` と重複）、(3) packet の test 本数転記（Evidence Ownership、本数を固定値で書かず PR body / CI 出力を正とすべき）→ いずれも accept、本 closeout で是正（[comment](https://github.com/higekiri-w/inventory-system-desktop/pull/71#issuecomment-5694443932)）
- 裁定: pass B の指摘 3 件を全件 accept。64 の UI-05-D11 決定列 / state 表に「詳細を見る」を追加、§64.5 の二重記述を整理、本 packet の AC6 と起票時実測の test 本数を「対象 test file 全 PASS（本数は PR body / CI 出力を正とする）」へ改めた（いずれも本 closeout commit）。review = pass、manual = pass（owner L3 round 1）
