# Plan Packet: UI 安全網 batch（Error Boundary + unsaved changes ガード）

## Workflow State

- Phase: archive
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: bfc770c
- Amendments: 7067123 96cae97
- Coordinator: Claude (Fable 5)
- Writer: Codex（owner relay。D-062 (c) 適合形〈Codex Writer + Sonnet Plan/Final Reviewer〉の 2 回目 dogfood、PR #58 closeout の次 dogfood 指定に従う）
- Plan Reviewer: Claude (Sonnet 5 subagent、独立 context)
- Final Reviewer: Claude (Sonnet 5 subagent、fresh context)
- Reviewed Content HEAD: 63cf1c6
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required
- Human Gate: merge のみ（plan 承認 2026-08-03 / L3 Windows native 目視 PASS + Ready 承認 2026-08-04 は消化済み）

遷移記録（append-only、初回分）: 本 packet を追加する content commit で `kickoff -> spec-check -> design -> plan-draft -> plan-gate` を材料化する。evidence = task scoped + Risk R3 判定を本 packet に記録（kickoff→spec-check）、設計正本の不足を識別 — Error Boundary 戦略は DEV_SETUP_CHECKLIST 7-8a と 52 §52.7 が UI_TECH_STACK §6.10 を指すが §6.10 本文が不在（番号予約のみ）、unsaved changes ガードは DEV_SETUP_CHECKLIST 7-8c の 1 行（`useUnsavedChangesWarning` + isDirty 連動）のみで正本節が不在、61/62/63/64 の「商品登録へ」導線注意書きが hook 前提でないアドホック文言のまま、69 §69.13 に 7-8c backlog 注記が残存（spec-check→design）、design 出力を同一 plan-first change 内で source docs へ反映 — UI_TECH_STACK §6.10（Error Boundary 戦略）/ §6.11（未保存編集の離脱ガード）新設、52 §52.7 行更新、61 §61.5 / 62 / 63 / 64 の該当行置換、69 §69.13 行更新（design→plan-draft）、packet + Test Design Matrix 完成・commit（plan-draft→plan-gate）。

2026-08-03 Sonnet Plan Review round 1 = FAIL（P1=3 / P2=0 / P3=2、全件 accept、裁定詳細は Review Response 参照）。P1-1（Contract Probe CP1/CP2 が pending のまま plan-gate 確定 — DEV_WORKFLOW「before Plan Gate」要件の自己解釈による緩和）/ P1-2（isDirty 解除タイミングの設計契約欠落 — 51 の create/update `onSuccess` は baseline 未 reset のまま navigate するため、規定なしでは保存成功パスが毎回誤発火する）/ P1-3（分類表に 65 系 6 page が未列挙 + sweep が全数性を機械検証しない）は design 出力の改訂を要するため、最早影響 phase = design へ backtrack する（`plan-gate -> design`）。

2026-08-03 round 1 是正 content commit で `design -> plan-draft -> plan-gate` を再材料化する。evidence = CP1/CP2 probe を throwaway harness で実施し PASS を Contract Probe 節へ記録（`@tanstack/react-router` 1.168.23 実測、P1-1 是正 — 以後の probe は plan-gate 遷移前実施を厳守）、§6.11 UI-USW-D1 へ「保存成功時の誤発火防止 MUST」（`onSuccess` 内 navigate 前の baseline 同期 / result panel 型の `!isFormLocked` 明示式）を追記し分類表 51/61〜64/69 行を補強（P1-2 是正）、65 系 6 page の除外行追加 + 60 表記是正 + T17 の全数性 sweep 化 + X9 追加（P1-3 是正）、P3 2 件反映（Final Review の 2 回目 Contract Audit pass 明記 / 60 component 名）、Plan Commit 欄へ bfc770c 転記（design→plan-draft）、`doc-consistency-check --target plan` 通過・commit（plan-draft→plan-gate）。

2026-08-03 Sonnet Plan Review round 2 = conditional pass（P1=0 / P2=1 / P3=1、全件 accept、round 1 是正の独立再検証 = (a)(b)(c) 全て実質完了、裁定詳細は Review Response 参照）。P2-1（IntegrityCheckPage が catch-all 除外行の (e) 理由と食い違う — `selectedCodes` checkbox 蓄積 + `result` state 実在）は分類軸 (b) の精緻化と明示行追加という design 出力の改訂を要するため、最早影響 phase = design へ backtrack する。**process 記録**: round 1 是正（b1c917e）は design 出力の改訂を伴ったにもかかわらず `state-backtrack` state-only commit を作らず content commit の narrative のみで backtrack を主張した — DEV_WORKFLOW backtrack 契約からの逸脱（round 2 P3-1 起源）。round 1 の遷移記録の backtrack 主張は「plan-gate に留まったままの in-place 是正だった」と読み替える（append-only のため原文は残す）。round 2 是正からは正規機構を踏む: `state-backtrack plan-gate->design`（4b10d55、state-only）→ 本 content commit で再前進。

2026-08-03 round 2 是正 content commit で `design -> plan-draft -> plan-gate` を再材料化する。evidence = §6.11 UI-USW-D3 除外軸 (b) を「file・DB 等からの再実行で再導出可能（値の記入を伴わない選択 state を含む）」へ精緻化し、分類表へ IntegrityCheckPage 明示行（除外 (b)）を追加、catch-all 行を component 名個別列挙へ改め 75 を分離（P2-1 是正、design→plan-draft）、Matrix T17 へ「除外側個別列挙・自動除外型実装の禁止」を明記（residual risk 採用）、`doc-consistency-check --target plan` 通過・commit（plan-draft→plan-gate）。

2026-08-03 Sonnet Plan Review round 3（closure、fresh 独立 context）= PASS（P1/P2/P3 = 0、round 2 closure 判定 (a)(b)(c) 全 closed、新規指摘なし）。分類表 26 page 全数一致・backtrack 機構の DEV_WORKFLOW 適合・§6.11 精緻化の実装整合を独立再検証済み。plan-gate 収束（rally 実績 = round 1 → round 2 → round 3 の単調収束）。owner plan 承認待ち（plan-gate → plan-approved は owner 承認 evidence を伴う state-only commit で材料化する）。

2026-08-03 owner plan 承認（介入 1/3、承認取得の interactive 記録あり。承認文言 = 「承認する」）。本 content commit で `plan-gate -> plan-approved -> implementing` を材料化する。evidence = Plan Review 3 round 収束・P1/P2=0（plan-gate→plan-approved、上記 round 3 記録が pre-existing evidence）、Codex Writer 発注書の交付準備完了 + Plan Commit 記入済み bfc770c（plan-approved→implementing）。発注書は owner relay（外部端末、cwd = public-writer clone 固定）で交付する。

2026-08-03 Codex Writer fail-closed 停止（true positive、relay 1/4 消化）: packet 分類表 64 行の「入力差分に画像選択済みを含む」が実装（`DisposalPage.tsx` の form state は disposalDate / rows 系のみ、画像 state なし）および 64 Non-scope「画像添付」と矛盾 — Coordinator の design 出力の事実誤り（63 返品・交換の `receipt: ReceiptImageState` との対称性を誤仮定。Plan Review 3 round も未検出、Writer の実装時実査が捕捉）。最早影響 phase = design へ backtrack する（`state-backtrack implementing->design` = 3fa958c、state-only）。

2026-08-03 Amendment 1 content commit で `design -> plan-draft -> plan-gate` を再材料化する。evidence = 64 §表示/操作の該当行から「（画像選択を含む）」を削除し Non-scope と整合化、packet 分類表 64 行を「result panel 型（明細 or 入力〈廃棄日等〉差分）・画像機能なし」へ是正（63 行は `receipt` state 実在確認済みのため画像維持、design→plan-draft）、packet 内「画像」全 sweep で追随漏れなしを確認・commit（plan-draft→plan-gate）。amendment review（Sonnet 独立 context、focused）を経て plan-approved 以降を再材料化する。

2026-08-03 本 content commit で `plan-gate -> plan-approved -> implementing` を再材料化する。evidence = Amendment 1 review PASS + 機構是正完了 8dd533d（plan-gate 通過）、原 owner plan 承認（介入 1/3）が有効 — Amendment 1 は Goal / Scope / AC / 適用 6 画面構成を変えない事実是正であることを amendment review が byte-identical 検証で確認済み、Plan Commit bfc770c 維持（plan-approved→implementing）。Codex Writer へ再開発注（前提是正の追記発注、relay 2/4 消化見込み）。

2026-08-03 本 content commit で `implementing -> local-verified -> independent-review` を材料化する。evidence = Writer 実装完了報告 + 全 gate green + L1 full exact-HEAD CLEAN（定量記録は PR #60 body を正、Final Review が evidence log を実確認 — implementing→local-verified）、Coordinator mutation 独立再実測 X1〜X9 全 red + Final Review P1/P2=0（Review Response 参照 — local-verified→independent-review）。Final Review P3×2 は本 commit で是正済み。次 = `independent-review -> human-confirm` の state-only 遷移（Reviewed Content HEAD 設定を含む）→ owner L3（Windows native 目視）+ Ready 承認（介入 2/3）。

2026-08-04 owner L3 PASS + Ready 承認（介入 2/3、Review Response 参照）。本 content commit で L3 記録と backlog 起票を確定し、続く state-only 遷移で `human-confirm -> ready-hosted-final` を材料化する。以後の段取り = 最終 tracked HEAD で L1 full exact-HEAD CLEAN を再取得（packet 記録 commit が実装 HEAD 63cf1c6 より後のため）→ PR body へ最終 evidence 記載（tracked 外）→ PR Ready 化 → hosted CI 三点一致 → owner merge（介入 3/3）。

2026-08-04 最終 tracked HEAD で L1 full exact-HEAD CLEAN（MERGE_EVIDENCE_VALID=true、記録は PR body 正）→ PR Ready 化 → hosted CI run success で三点一致成立 → owner merge 承認（介入 3/3）→ PR #60 squash merge `42095f5`。本 archive 化 commit で `ready-hosted-final -> merge -> archive` を材料化する。evidence = merge 実確認（state MERGED / mergeCommit 42095f5、pre-existing）+ 本 commit での archive 移動と Plans.md 完了記録。

## Owner Effort Budget

- 介入回数上限: 3（plan 承認 / L3 目視 + Ready 承認 / merge）
- 実働時間上限: 30分
- relay 往復上限: 4（batch A 実績 6/4・batch B 実績 10/4 を踏まえた調整値。Codex Writer 発注 + Plan Review 複数 round + Final Review を見込む。超過が見えた時点で Coordinator が停止し owner の事前明示承認を得る）

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
全画面共通の route 遷移挙動（navigation blocking）と operator workflow（破棄確認ダイアログ / crash 回復導線）に触れる横断 UI 基盤変更で、DEV_WORKFLOW Risk Tier の「UI route/search behavior」「operator workflow」に該当する。wire / DTO / DB は不変。

## Goal

Goal Invariant: operator が (1) 画面描画中の未捕捉例外で白画面に遭遇せず、日本語の説明と回復導線を得られる、(2) 手入力した未保存の編集内容を、確認なしの画面遷移で失わない。

### 最小完了条件

- render 例外発生時に日本語 fallback 画面（再試行 + ホームへ戻る導線付き）が表示される
- 適用対象画面で isDirty=true の in-app 遷移時に破棄確認ダイアログが出て、「編集を続ける」で入力が保持される

### 失敗定義

- crash が白画面のままになる経路が残る
- ガードが isDirty=false や保存成功後の遷移で誤発火し、operator の通常操作を阻害する
- 既存の importing ガード（55 §55.7）や EmptyState / Alert 2 系統（02 ⑥）の挙動が変わる

### 非目的

- Storybook 配線（npm 依存追加を伴う開発 infra、別判断）
- error telemetry / 外部送信
- react-hook-form 等 form library の導入（UI_TECH_STACK §2.7 方針維持）
- Tauri native window close（×ボタン）の破棄防止（UI-USW-D4 で非保証を明文化、別 backlog）
- backend / Tauri command / DTO / DB 変更

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。

## Scope

1. **Error Boundary 2 層**（UI-EB-D1〜D3、UI_TECH_STACK §6.10 新設）: router `defaultErrorComponent`（RootLayout 保持で `<main>` 領域に fallback）+ root route `errorComponent`（RootLayout 自体の render 例外向け全画面 fallback）+ 共通 fallback component（日本語見出し / データ非消失の説明 / 再試行 / ホームへ戻る / 技術詳細折りたたみ）。npm 依存追加なし。
2. **`useUnsavedChangesWarning(isDirty: boolean)` hook**（UI-USW-D1、§6.11 新設）: TanStack Router `useBlocker`（`shouldBlockFn` + `withResolver`）+ `enableBeforeUnload` 連動。共通破棄確認 AlertDialog（UI-USW-D2）を hook とセットで提供。
3. **適用画面の全数分類と配線**(UI-USW-D3、下記分類表): 適用 6 画面（51 / 61 / 62 / 63 / 64 / 69）へ isDirty 計算 + hook 配線。
4. **再導入防止 sweep test**: `useBlocker` 直接使用を 3 hook（csv-import / daily-report-import / useUnsavedChangesWarning）に限定 + 分類表の適用画面全てに hook 配線があること + `src/features/**/*Page.tsx` 実在集合と分類（適用 + 除外）の全数一致を検証（batch B `describe-error-no-local-duplicates` 同型 + 全数性 diff）。
5. **docs sync**: §6.10 / §6.11 新設、52 §52.7 行更新、61/62/63/64 文言置換、69 §69.13 行更新（以上は本 plan-first change で反映済み）。DEV_SETUP_CHECKLIST 7-8a / 7-8c の消化更新、適用各 function-design への適用 1 行追記と更新履歴、UI_TECH_STACK §7.4 更新履歴は実装 commit で行う。

### 適用画面 全数分類表

分類軸（UI-USW-D3）: 適用対象 = 「利用者の手入力で構築され、保存前に画面遷移で失われると再入力が必要な蓄積編集 state を持つ画面」。除外軸 = (a) 進行中処理ガード既設（55 §55.7 所有、相互排他）、(b) file・DB 等からの再実行で再導出可能な導出 state（値の記入を伴わない選択 state を含む — round 2 P2 で精緻化）、(c) 行単位の即時 DB 保存で蓄積未保存が生じない、(d) 外部ツールまたぎの専用復帰設計が所有、(e) 編集 state なし（表示 / URL search state / 即実行操作のみ）。

| 画面（design doc） | component | 分類 | isDirty 定義 / 除外理由 |
|---|---|---|---|
| 商品登録・編集（51） | `ProductFormPage` | 適用 | values が初期値と不一致（新規 = 空 form 初期値、編集 = 読込み値）。**現行実装は create/update の `onSuccess` が baseline 未 reset のまま `onNavigateToList` を呼ぶため、UI-USW-D1 MUST（navigate 前の baseline 同期）の実装追加が必須**（round 1 P1-2） |
| 入庫（61） | `ReceivingPage` | 適用 | result panel 型: `isDirty = (明細 ≥1 行 or ヘッダ入力差分) && !isFormLocked`。保存成功後（result panel、`isFormLocked`）は非 block で、result panel 内の遷移導線も block しない |
| 手動販売（62） | `ManualSalePage` | 適用 | 61 と同じ result panel 型（`!isFormLocked` を明示的に含める。result panel 内「詳細を見る」「日次売上へ」導線は非 block） |
| 返品・交換（63） | `ReturnExchangePage` | 適用 | 61 と同じ result panel 型 + 入力差分に画像選択済みを含む |
| 廃棄・破損（64） | `DisposalPage` | 適用 | 61 と同じ result panel 型（明細 ≥1 行 or 入力〈廃棄日等〉が初期値と差分）。**画像機能は存在しない**（64 Non-scope「画像添付」どおり — Amendment 1 で「画像選択済みを含む」の事実誤りを是正、Writer fail-closed 起源） |
| 閾値設定（69） | `ThresholdSettingsPage` | 適用 | 既存 isDirty（values ≠ savedValues の descriptor 比較）を hook へ接続。`onSuccess` の `setSavedValues(submittedValues)` が UI-USW-D1 MUST の既存正例 |
| 棚卸し（73） | `StocktakePage` | 除外 (c) | カウントは `useUpdateCount`（「数を保存」）で行単位の即時 DB 保存（`StocktakePage.tsx` `saveCount`）。蓄積未保存 state がなく、未保存は単一入力欄のみで再入力コスト極小 |
| 在庫変動履歴 + 記録詳細 5 画面（65） | `InventoryRecordsPage`、`CsvImportRecordDetailPage` / `DisposalRecordDetailPage` / `ManualSaleRecordDetailPage` / `ReceivingRecordDetailPage` / `ReturnRecordDetailPage` | 除外 (e) | 照会・表示のみ。`InventoryRecordsPage` の検索フィルタ draft は URL search state 相当で編集 state なし、詳細 5 画面は入力なし（round 1 P1-3 で追補） |
| CSV 取込み（55）/ 日報取込み | 2 flow | 除外 (a)(b) | importing 中は §55.7 既設 `useBlocker` ガードが所有（確認ダイアログなし常時 block の既存設計判断）。preview は file 再選択で再導出可能 |
| 商品CSV取込み（60） | `ProductImportPage`（子: `ProductImportPreview`） | 除外 (b) | preview は file 再選択で再導出可能 |
| PLU 書出し（67） | `PluExportPage` | 除外 (d) | §6.5「外部ツールをまたぐ未完了状態」の localStorage 復帰設計が所有 |
| バックアップ・復元（68） | `BackupRestorePage` | 除外 (e) | 即実行操作系。backup_path 選択は保存操作単位で完結、蓄積編集 state なし |
| 整合性チェック（75） | `IntegrityCheckPage` | 除外 (b) | `result` は同一 DB 状態からの `runIntegrityCheck` 再実行で再導出可能、`selectedCodes`（補正対象 checkbox）は値の記入を伴わない選択 state で再実行後の再選択で再現可能。補正実行自体は確認ダイアログ既設の即実行操作（round 2 P2 で catch-all から明示行へ昇格） |
| 一覧・照会・表示系（50/53/54/56/57/58/66/74。54 は page component なし） | `HomePage` / `DailySalesPage` / `MonthlySalesPage` / `OperationLogsPage` / `ProductListPage` / `StockInquiryPage` / `StockMovementsPage` | 除外 (e) | URL search state / 表示のみで編集 state なし |

全数性の検証: sweep test（T17）が `src/features/**/*Page.tsx` の実在集合と本分類表の記載（適用 manifest + 除外 list）の diff を機械検証する — 新規 page 追加時に未分類のままだと test が red になる（round 1 P1-3 の是正で plan 段階の目視から機械検証へ昇格）。Writer は実装時に分類表との不一致を発見したら fail-closed で Coordinator へ報告する。

## Non-scope

- CSV / 日報 / 商品CSV取込みの preview 段階 discard 防止（分類軸 (a)(b)）
- PLU 書出し（67）の未完了状態管理（§6.5 既存設計が所有）
- Tauri native window close・OS 強制終了・reload での破棄防止の保証（`enableBeforeUnload` はベストエフォート連動、UI-USW-D4）
- beforeunload ダイアログのカスタム文言（platform 制約で不可）
- 既存 importing ガード（§55.7）の挙動変更

## Acceptance Criteria

- AC1: 新設 hook / fallback の unit・integration test（Matrix T1〜T8）が `npm test` で green
- AC2: 適用 6 画面の配線 test（T9〜T14、各画面 dirty→block + 保存成功後の非 block。`src/features/<feature>/` 配下の `*unsaved-guard*.test.tsx` 命名）を含む `npm test` が exit 0
- AC3: sweep test（`unsaved-changes-guard-sweep.test.ts`）が「useBlocker 直接使用 3 hook 限定」+「分類表適用画面の hook 配線一致」+「`src/features/**/*Page.tsx` 実在集合と分類（適用 + 除外）の全数一致」を検証し green
- AC4: 既存 importing ガードの回帰（T15。引用する既存 test 名は `rg -l "useBlocker" src` で実在確認してから Matrix に転記）を含む既存 suite 全体の `npm test` が exit 0
- AC5: `bash scripts/local-ci.sh full` が exact-HEAD で CLEAN（L1）
- AC6: `bash scripts/doc-consistency-check.sh --target plan` PASS
- AC7: L3 = crash fallback 画面と破棄確認ダイアログの Windows native 目視 PASS を PR body の L3 節に記録する。前提 = Writer 完了時の `cargo check --release` が exit 0（Test Plan 参照）。synthetic crash 再現手順（dev 専用 throw の一時 patch diff）を Ready 依頼と同時に提示し、tracked には残さない

## Design Sources

- Requirements / spec: `docs/DEV_SETUP_CHECKLIST.md` 12.3（7-8a / 7-8c）
- Architecture: `docs/architecture/ui-task-specs.md`
- Function / command / DTO: なし（frontend-only）
- DB: なし
- Screen / UI: `docs/UI_TECH_STACK.md` §6.10 / §6.11（本 change 新設）・§2.7・§5.5.1、`docs/function-design/` 52 / 51 / 61 / 62 / 63 / 64 / 65 / 69 / 73 / 55、`docs/design-system/01-decision-rules.md`（DSR-03 / DSR-07）、`docs/design-system/02-component-catalog.md`（⑥）
- Decision log / ADR: D-053（describeError / error_id 境界 — 名称類似の別概念）、D-062（編成契約）

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status |
|---|---|---|
| Backend function / command / repository / validation / error | なし | N/A（frontend-only） |
| Command / DTO / generated binding / wire shape | なし | N/A（bindings diff ゼロが AC 相当の前提） |
| DB / transaction / audit / rollback / migration | なし | N/A |
| Screen / UI / route state / Japanese wording | UI_TECH_STACK §6.10 / §6.11、52 §52.7、61 / 62 / 63 / 64 該当行、69 §69.13 | updated in this PR（plan-first で反映済み）。51 / 61〜64 / 69 への適用 1 行追記・各更新履歴・UI_TECH_STACK §7.4・DEV_SETUP_CHECKLIST 7-8a/7-8c 消化は updated in this PR（実装 commit） |
| CSV / TSV / report / import / export format | なし | N/A |
| Durable decision / ADR | §6.10 / §6.11 を正本とし新規 decision-log ID は起票しない（既存 D-053 / D-062 参照で足りる） | existing sufficient |

## Registration / Generation Obligations

- FE test 新設 → test 名 or 参照コメントに REQ-NNN / UI-NN token を含める（WF-TRACE T4 baseline 両方向）。
- 他は該当なし: route 新設なし / Tauri command なし / function-design doc 新設なし（UI_TECH_STACK 節新設は design_compliance map 対象外）/ navigation.ts 変更なし / §5.5 consultation relay 不使用。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| 7-8a | UI_TECH_STACK §6.10 | UI-EB-D1 | 2 層構成（defaultErrorComponent = layout 保持 / root errorComponent = 全画面）。却下: `react-error-boundary` 依存追加（npm 供給網ガードと Router 標準機構で不要）、全画面単層（sidebar 消失で回復導線が乏しくなる） | `src/main.tsx`、`src/routes/__root.tsx`、共通 fallback component | T6 / T7 / T8、X4 / X5 |
| 7-8a | UI_TECH_STACK §6.10 | UI-EB-D2 | crash 画面構成（日本語見出し / データ非消失説明 / 再試行 + ホームへ戻る / 技術詳細折りたたみ、icon + 文言併用 = §5.5.1） | 共通 fallback component | T6 / T8、X6 / X8 |
| 7-8a | UI_TECH_STACK §6.10 | UI-EB-D3 | describeError（UI-ERR-D1 / D-053）は CmdError 専用で render 例外は対象外。query isError 経路（02 ⑥）は不変で Error Boundary は最終防衛層のみ担う | fallback component（describeError 非経由） | T6、回帰 suite |
| 7-8c | UI_TECH_STACK §6.11 | UI-USW-D1 | hook API は `isDirty: boolean` 受け取りの薄い形。却下: form library 依存の dirty 追跡（§2.7 方針違反）、画面ごとの useBlocker 直書き（sweep で禁止） | `src/hooks/useUnsavedChangesWarning.ts` | T1〜T5、X1 / X3 |
| 7-8c | UI_TECH_STACK §6.11 | UI-USW-D2 | 破棄確認 AlertDialog（「編集を続ける」既定 / 「破棄して移動」destructive）。DSR-07 適合 = 未保存内容の破棄は復元不能な不可逆操作 | 共通 dialog component | T2〜T4、X2 |
| 7-8c | UI_TECH_STACK §6.11 | UI-USW-D1 MUST / UI-USW-D3 | 保存成功時の baseline 同期（誤発火防止 MUST、round 1 P1-2）+ 適用範囲の分類軸（蓄積編集 state 基準 + 除外軸 a〜e）。importing ガードと相互排他 | 適用 6 画面 + sweep test | T9〜T14 / T16 / T17、X7 / X9 |
| 7-8c | UI_TECH_STACK §6.11 | UI-USW-D4 | native window close は非保証と明文化（beforeunload 連動はベストエフォート）。却下: onCloseRequested 実装（検証コスト大、別 backlog） | §6.11 記載のみ | — |

## Design Intent Audit

- Source docs can answer what/why: §6.10 / §6.11 新設 + 52/61/62/63/64/69 の同期で、chat 履歴・packet 非依存で参照可能。
- Plan-only durable decisions promoted: Error Boundary 戦略と離脱ガード契約を §6.10 / §6.11 へ昇格（packet は分類表の作業用写しのみ保持）。
- Assumptions and constraints: TanStack Router `^1.168` の `defaultErrorComponent` 捕捉境界と `useBlocker withResolver` 挙動（CP1 / CP2 で plan-approved 前に実証）。
- Deferred design gaps: native close ガード（UI-USW-D4 backlog）、beforeunload の WebView2 実機挙動（L3 で観察のみ、保証にしない）。
- Test Design Matrix cites decision IDs: yes（Matrix 参照）。
- Escape hatch self-check: useBlocker 直接使用の例外は既存 2 hook + 新設 1 hook のみで sweep が機械強制。§55.7 importing ガードとの相互排他を UI-USW-D3 に明記。

## Impact Review Lenses

not applicable（設計 backlog〈Phase 1 起源の意図的 deferral〉消化起点で、フィールド調査・実機・POS 由来の新事実なし）。ただし:

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| 環境・再現性 | beforeunload × Tauri WebView2 native close の挙動は未検証前提として UI-USW-D4 で明示 defer（repo-pinned 強制はしない） | §6.11 UI-USW-D4、backlog（native close ガード） |

## Design Readiness

- Existing design docs are sufficient because: 戦略・契約の正本（§6.10 / §6.11）を本 plan-first change で新設し、隣接 doc の stale 文言（52 / 61 / 62 / 63 / 64 / 69）を同時に同期済み。
- Source docs updated in this PR: 上記 Required Design Artifacts 表参照。
- Design gaps intentionally deferred: native close ガード、error telemetry。
- Durable decisions promoted: UI-EB-D1〜D3 / UI-USW-D1〜D4。

Minimum design checks:

- Layer ownership: UI 層のみ（CMD/BIZ/IO/MNT 不変）。
- Backend function design: 変更なし。
- Command / DTO / data contract: 変更なし（bindings diff ゼロ）。
- Persistence / transaction / audit impact: なし。
- Operator workflow / Japanese UI wording: crash fallback 文言・破棄確認文言を §6.10 / §6.11 に規定、L3 で目視。
- Error, empty, retry, recovery: fallback の再試行 / ホームへ戻る導線（UI-EB-D2）、既存 02 ⑥ 2 系統は不変（UI-EB-D3）。
- Testability / traceability: SPEC-UISN-1〜4 + Matrix T/X 行 + REQ/UI token。

## Contract Probe

前提 = 未検証の外部ライブラリ挙動 2 点。**plan-approved 遷移前に最小実験を完了し結果を本節へ追記する**（fail した場合は design へ backtrack）。実験は throwaway test file で行い commit しない。

- CP1: 子 route component の render throw が router `defaultErrorComponent` で捕捉され、RootLayout（sidebar）を保持したまま `<main>` 内に fallback が render される（root へ escalate しない）: 最小 vitest harness（memory history + throw する route、throwaway file で実施後削除） -> **PASS（2026-08-03 実施）**。sidebar マーカー保持を確認、root `errorComponent` 併設時も子 route throw は defaultErrorComponent に捕捉（`@tanstack/react-router` 1.168.23 / vitest 実行）
- CP2: `useBlocker({ shouldBlockFn, withResolver: true })` が block 時に resolver 状態を返し、proceed() で遷移続行 / reset() で遷移中止できる（`@tanstack/react-router` 実 version）: 最小 vitest harness -> **PASS（2026-08-03 実施）**。`status: 'blocked'` → `proceed()` で遷移続行 / `reset()` で中止 + `status: 'idle'` 復帰、`shouldBlockFn` false 時は非 block を確認。resolver 実 shape = `action / current / next / proceed / reset / status`
- CP3: beforeunload が Tauri native close で発火するか: 実証しない（UI-USW-D4 で非保証を明文化）-> N/A

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| UI-EB-D1（2 層捕捉・layout 保持） | main.tsx / __root.tsx | T6 / T7（X4 / X5） | L3 目視 |
| UI-EB-D2（crash 画面構成・回復導線） | 共通 fallback component | T6 / T8（X6 / X8） | L3 目視 |
| UI-EB-D3（describeError 境界不変） | fallback は describeError 非経由 | T6 + 既存 describeError suite 不変 | non-scope（挙動不変） |
| UI-USW-D1（hook API / beforeunload 連動） | useUnsavedChangesWarning.ts | T1〜T5（X1 / X3） | — |
| UI-USW-D2（破棄確認 dialog・DSR-07 適合） | 共通 dialog | T2〜T4（X2） | L3 目視 |
| UI-USW-D3（分類軸・6 画面適用・相互排他・全数一致） | 適用 6 画面 + sweep | T9〜T14 / T16 / T17（X7 / X9） | — |
| UI-USW-D1 MUST（保存成功時の baseline 同期 = 誤発火防止） | 適用 6 画面の `onSuccess` / result panel lock | T9〜T14 の保存成功後非 block case | — |
| UI-USW-D4（native close 非保証） | §6.11 記載 | —（文書契約） | non-scope 明記 |
| 55 §55.7 importing ガード不変 | 変更なし | T15（既存 test 回帰） | — |
| 02 ⑥ EmptyState / Alert 2 系統不変 | 変更なし | 既存 suite 回帰 | — |
| UI-ERR-D1 / D-053（describeError 所有） | 変更なし | 既存 suite 回帰 | — |
| 52 §52.7 行（7-8a 解消） | 52 更新（済み） | doc-consistency（AC6） | — |
| 61/62/63/64 該当行・69 §69.13（文言置換） | 各 doc 更新（済み） | doc-consistency（AC6） | — |
| DEV_SETUP_CHECKLIST 7-8a / 7-8c 消化 | 実装 commit | doc-consistency（AC6） | — |
| §5.5.1（色だけに依存しない） | fallback / dialog は icon + 文言 | T6（見出し文言 assert） | L3 目視 |

隣接契約 sweep: §6.4（describeError 変換表）/ §6.1（通知の使い分け）/ 02 ⑭ FilePicker / §55.7 を実査し、上記の不変行以外に本 Scope が行使できる契約はないことを確認した（catalog への破棄確認 dialog 新規登録は §6.11 を正本とするため不要 — AlertDialog は既存部品の適用であり新パターン登録対象ではない。Plan Review で妥当性を確認する）。

## Test Plan

Test Design Matrix: `docs/archive/plans/test-matrices/2026-08-03-ui-safety-net-batch.md`（archive 化に伴い path 更新）

- targeted tests: T1〜T14（hook / fallback / 6 画面配線）
- negative tests: 誤発火防止（isDirty=false、保存成功後遷移の非 block — UI-USW-D1 MUST の baseline 同期を含む）
- compatibility checks: T15（importing ガード回帰）+ 既存 suite 全 green
- data safety checks: synthetic 入力のみ、実店舗データ不使用
- main wiring/integration checks: T16/T17 sweep（useBlocker 直接使用禁止 + 配線 manifest 一致 + `*Page.tsx` 全数分類一致）
- L3 を含むため Writer 完了条件に `cargo check --release` を含める（CI gate ではない）
- Final Review では 2 回目の Contract Audit pass を実施し、State Lifecycle Matrix の 3 系統（isDirty / blocker resolver / ErrorFallback）の遷移再検証を明示対象とする（DEV_WORKFLOW の operator-visible state lifecycle recommend を採用 — round 1 P3-1）

## Boundary / Wire Contract

- producer / consumer / wire type / internal type: 変更なし（IPC / DTO / bindings diff ゼロ）
- route/search: URL schema・search param 不変。追加されるのは遷移 block の resolver 状態のみ（URL に載らない）
- invalid input: isDirty は boolean のみ受理（型で強制）
- compatibility: beforeunload は platform 既定ダイアログ（カスタム文言なし）

## Review Focus

- 分類表の全数性（`src/features/` page component 列挙との突合、除外理由の妥当性 — 特に 73 棚卸し除外と 60 preview 除外）
- 誤発火経路の網羅（保存成功後 navigate、result panel、リセット操作、「商品登録へ」導線）
- CP1 / CP2 probe 結果と設計前提の一致（plan-approved 前提条件）
- oracle 独立性（文言 literal は component から import しない独立転記、anchor 一意性を rg -c で確認）
- 既存 useBlocker 2 hook の test への影響ゼロ

## Spec Contract

Contract ID: SPEC-UISN

- SPEC-UISN-1: render 例外発生時、operator は白画面ではなく日本語 fallback（再試行 + ホームへ戻る導線付き）を得る。
- SPEC-UISN-2: 適用 6 画面で isDirty=true の in-app 遷移は破棄確認を経由し、「編集を続ける」選択で入力が保持される。
- SPEC-UISN-3: isDirty=false および保存成功後の遷移は block されない（誤発火ゼロ）。
- SPEC-UISN-4: 55 §55.7 importing ガード・02 ⑥ 2 系統・UI-ERR-D1 describeError 境界は不変。

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-UISN-1 | Scope 1 | T6 / T7 / T8（X4〜X6 / X8） | fallback 表示・回復導線 | `npm test` + L3 目視 |
| SPEC-UISN-2 | Scope 2 / 3 | T2〜T4 / T9〜T14（X1 / X2 / X7） | block + 入力保持 | `npm test` + L3 目視 |
| SPEC-UISN-3 | Scope 2 / 3 | T1 + 各画面保存後非 block test | 誤発火防止 | `npm test` |
| SPEC-UISN-4 | Non-scope 遵守 | T15 + 既存 suite | 回帰 | `npm test` / L1 full |

## Data Safety

- 実店舗データは不要。test は synthetic form 入力のみ。
- crash 再現の dev 専用 throw を tracked に残さない（L3 手順は Ready 依頼への一時 patch diff 添付で行い commit しない）。
- `.local/ci-evidence/` は commit しない。

## Implementation Results

実装は Draft PR #60（https://github.com/higekiri-w/inventory-system-public/pull/60、Writer = Codex）。Error Boundary 2 層（`main.tsx` defaultErrorComponent + `__root.tsx` errorComponent）+ 共通 `RouteErrorFallback`、`useUnsavedChangesWarning` + `UnsavedChangesDialog`、適用 6 画面配線（ProductFormPage は `flushSync` による navigate 前 baseline 同期 = UI-USW-D1 MUST の実装形）、全 26 page 明示分類の sweep test（適用 6 + 除外 20 の個別列挙）、実装段 docs 同期（DEV_SETUP_CHECKLIST 7-8a/7-8c 消化ほか）を含む。生成物（bindings / routeTree / traceability）・依存の diff ゼロ。定量 evidence（test 件数 / exact-HEAD SHA / gate 記録）は D-035/D-038 に従い PR body を正とする。

## Review Response

2026-08-03 Plan Review round 1（Sonnet 5、独立 context、対象 = bfc770c）: P1=3 / P2=0 / P3=2、Coordinator 裁定 = 全件 accept（各 P1 は引用 file:line を Coordinator が実読裏取り — DEV_WORKFLOW L63 原文 / `ProductFormPage.tsx` onSuccess の baseline 未 reset / 65 系 6 page 実在）。

- P1-1（probe 時機違反）: accept。CP1/CP2 を実施し PASS を記録（是正 commit）。probe 自体は両方 PASS のため設計変更なし。
- P1-2（isDirty 解除契約欠落）: accept。§6.11 UI-USW-D1 へ MUST 追記 + 分類表補強。
- P1-3（分類表 65 系未列挙 / 全数性の機械検証欠如）: accept。65 行追加 + T17 全数性 sweep 化 + X9。
- P3-1（Contract Audit の追加 pass）: accept。Test Plan に Final Review での実施を明記（DEV_WORKFLOW の operator-visible state lifecycle recommend 採用）。
- P3-2（60 component 名表記）: accept。`ProductImportPage`（子: `ProductImportPreview`）へ是正。

2026-08-03 Plan Review round 2（Sonnet 5、fresh 独立 context、対象 = bfc770c + b1c917e）: P1=0 / P2=1 / P3=1、round 1 是正の独立再検証 (a)(b)(c) = 全て実質完了。Coordinator 裁定 = 全件 accept（P2 は `IntegrityCheckPage.tsx` の state 実在を実読裏取り、P3 は DEV_WORKFLOW backtrack 契約原文 + batch B の先例〈backtrack narrative + 実 state-backtrack commit の組〉との突合で確認）。

- P2-1（IntegrityCheckPage 分類）: accept、Coordinator 裁定 = **除外 (b)**。`result` は同一 DB からの再実行で再導出可能、`selectedCodes` は値の記入を伴わない選択 state。除外軸 (b) を精緻化し明示行を追加、catch-all 行は component 名個別列挙へ改めた。
- P3-1（遷移記録の backtrack 主張と実 phase 遷移の乖離）: accept、ただし修正方向は reviewer 案（主張削除）ではなく batch B 先例に合わせた正規機構の履行 — round 1 分は process 逸脱として遷移記録に記録し、round 2 是正から `state-backtrack` state-only commit（4b10d55）を経由。
- residual risk 採用: T17 sweep の除外側個別列挙（自動除外型実装の禁止）を Matrix に明記。61〜64 の保存成功後に stale 警告文が残る既存 quirk は本 change の scope 外の pre-existing として記録のみ（ガードの発火判定に影響なし。是正するなら別 change）。

2026-08-03 Plan Review round 3（Sonnet 5、fresh 独立 context、対象 = bfc770c〜e781f9a 累積）: P1/P2/P3 = 0、round 2 closure (a)(b)(c) 全 closed。独立検証 = 分類表と `src/features/**/*Page.tsx` 実在 26 file の 1:1 全数一致、4b10d55 の hunk レベル state-only 適合、機械 gate（check-workflow-git / doc-consistency-check --target plan）の独立再実行 PASS。**Plan Gate 収束、P1/P2 = 0**。

2026-08-03 Amendment 1 focused review（Sonnet 5、独立 context、対象 = 3fa958c / 7067123 / 96cae97）: 事実是正の内容 = 妥当（DisposalPage 画像 state 不在 / ReturnExchangePage `receipt` 実在 / 64 Non-scope 整合を独立裏取り、「画像」残存 sweep = クリーン、Goal/Scope/AC/適用 6 画面 = 64 行文言以外 byte-identical で owner 再承認トリガーなし）。機構 P1×2 = accept: (1) Amendment commit が遷移記録の主張どおりに Phase を復帰させていない、(2) Amendments field 未記録。是正 = 本 commit で Phase を plan-gate へ復帰 + `Amendments: 7067123 96cae97` を記録し、`design -> plan-draft -> plan-gate` の再材料化を完成させる（evidence = Amendment 1 内容の review PASS）。

2026-08-03 Coordinator mutation 独立再実測（実装後、clean tree、Writer 記録非参照の独立導出）: X1〜X9 全 red・survivor 0。X8 anchor 一意性（`rg -c`、production/test の衝突なし）も独立確認。

2026-08-03 Final Review（Sonnet 5、fresh 独立 context、対象 = 実装 commit + PR #60 body）: P1/P2 = 0、P3 = 2、Coordinator 裁定 = 全件 accept・即時是正（本 commit: Implementation Results の D-038 準拠 backfill / Matrix T15 の実名転記）。Contract Coverage Ledger 全行適合（UI-USW-D1 MUST の navigate 前 baseline 同期を実コード確認）、negative space クリーン、State Lifecycle 3 系統の追加 audit pass PASS（round 1 P3-1 採用分）、X2/X9 の抜き打ち再注入 red、sweep manifest と実在 26 page の 1:1 一致、PR body freshness 整合、L1 evidence（exact-HEAD CLEAN / MERGE_EVIDENCE_VALID）実 log 確認。

- 観察記録（pre-existing、scope 外）: `useDailyReportImportFlow` の useBlocker regression coverage が csv-import 側より薄い（mock のみで shouldBlockFn/enableBeforeUnload の assert なし）。必要なら別 change で補強。
2026-08-04 owner L3（Windows native 目視、介入 2/3 = L3 + Ready 承認）: child fallback PASS / root fallback PASS / 未保存変更 6 画面 PASS / 保存成功後・result panel 非 block PASS。観察事項 = 一部検索欄が live 検索型へ未統一（UI-01a-D9 は商品一覧のみ適用済み）— 本 change の安全網 scope 外で L3 結果を妨げない follow-up 候補として Plans.md backlog へ起票。

- Findings Freeze: frozen after Final Review（2026-08-03）; post-freeze exceptions: none.
