# Plan Packet: UI 表示磨き batch 第 1 弾（9 件 — gated Amendment 2 で Scope 7(b) を除外）

Plans.md「次の行動」③（UI backlog の表示磨き batch）の第 1 弾。PR #80 / PR #5 / UI-15 / PR #14 の各 L3 owner 所感に起源を持つ非阻害 P3 の表示磨きを 1 PR に束ねる。本 change は新編成の初運用でもある: Writer = Claude Sonnet 5（worktree isolation）、実装後レビューに Opus 5 の修正案 claims-producer round（D-056 の確定 role のまま、§5.4 低制約 profile）+ Codex cross review を置く。

対象外の裁定（owner 相談 2026-08-29 の座組決定に基づく Coordinator 整理、Plan Gate 承認で確定）:
- 「戻り先の検索維持」— 既存 backlog「前の画面へ戻る」導線契約 design-first 候補の所管（owner 裁定要のため本 batch から除外)
- 「状態表現の統一」（hub 3 値 vs 詳細 raw 粒度）— 65 §65.6.1 L134 が「completed_partial は一覧では『有効』に含め、詳細は詳細画面が担う」と**意図的乖離を明文化済み**（PR #13/#14 で確定した設計）。磨き PR で触らず design-first 裁定候補として backlog 残置
- UI-08 prepare failure Alert — CMD/BIZ 契約拡張を要するため既存 backlog どおり別件

## Workflow State

- Phase: ready-hosted-final
- Risk: R2
- Execution Mode: fable-window
- Plan Commit: b3ca503
- Amendments: b1cd203 7be74f0 8ef3597 19f4d65（gated Amendment 2 — Scope 7(b) 除外 + T7 確定 + T2 強化、実装後レビュー round 1 裁定 / gated Amendment 3 — DSR-16 準拠再構成 + docs 5 点追加、owner L3-lite round 1 可読性 FAIL の裁定 / gated Amendment 4 — 横 overflow 是正 + DSR-03 整合 + 今回行の同一 Table 化、owner L3-lite round 2 FAIL の裁定 / gated Amendment 5 — CostDiffDialog 暗黙 dismiss 硬化 + T11 追補、owner L3 で発見の P1〈merge blocker〉裁定）。owner L3-lite round 3 裁定（dialog 幅 CSS 詳細度是正 / Z004 Alert・両 tab Badge の warning tone 統一）は packet 契約不変の実装是正のため新規 Amendment を追加しない — 「owner L3-lite round 3 と裁定」節の narrative が正本
- Coordinator: Claude Fable 5 (main session)
- Writer: Claude Sonnet 5 (subagent、worktree isolation、§5.6 従来型発注書駆動)
- Plan Reviewer: Claude Sonnet 5 (independent fresh context、Writer とは別 context)
- Final Reviewer: Claude Sonnet 5 (independent fresh context) + Opus 5 修正案 claims-producer round（§5.4 低制約、read-only）+ Codex cross review + Coordinator 裁定
  - Opus 投入の例外記録（manual §3 L36「通常レビューは既存分業を維持」に対する例外、Plan Review round 1 P2-1）: owner 明示の座組決定（2026-08-29「その座組でやってみたいね」）による pilot 投入。目的 = ①保留中の Opus×Sonnet 並走レビュー実験（Plans.md 記録）の「簡単 backlog 水準」較正 ②operator-facing UI 磨きにおける修正案 claims-producer 型の実務検証。read-only・§5.4 低制約・Writer 非割当の D-056 機構は不変。本例外の最終確定は Plan Gate 承認に含める
- Reviewed Content HEAD: aafa06a
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required
- Human Gate: none（owner Windows native L3 最終確認 全項目 PASS 2026-08-29。残る owner 操作は Ready 化と merge のみ〈merge は委任範囲〉）

Phase 遷移記録（kickoff → spec-check → plan-draft → plan-gate、本 plan-first commit に同乗）: task scope と R2 判定は本 packet に記録。spec-check では表示磨き 10 件の規定 doc 節を全数実査（Coordinator 発注の read-only 調査で file:line / 節番号 / 契約接触の有無を確認）し、既存 docs は「4 件の軽微 doc 追記（Scope 8）を Writer が同 PR で行う」前提で実装十分と判定 — 未解決の設計問題はないため spec-check → plan-draft の許可された skip を適用。編成規律の適合は Coordinator が正本実読で確認済み: D-062 (c) の別 vendor Plan Reviewer 義務は Writer が Codex の packet 限定で本 packet 非適用、manual §2 の Writer ≠ Plan/Final Reviewer + fresh context 独立は別 context 起用で充足、manual §3 L35（希少・高コスト slot を通常実装 Writer に充てない）に Sonnet は非該当、L36 により Opus 5 は read-only Reviewer 発注書ロールのまま（Writer 割り当てなし）。

## Owner Effort Budget

- 介入回数上限: 3
- 実働時間上限: 30分
- relay 往復上限: 2（Codex は cross review のみ。Sonnet Writer は subagent のため relay 不要）
- Plan Review round 天井: 3

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R2

Reason:
契約非変更の表示磨きが中心（文言・レイアウト・通知・スクロールの表示挙動）。route/search state・Tauri command DTO・bindings・DB schema・CSV format・merge gate に触れない。backend 変更なし（frontend のみ + 設計 doc の軽微追記 4 箇所）。operator-facing 画面の変更のため owner 目視を Human Gate に置く（Plans.md 注意リストの慣行）。R3 該当行（route/search state / operator workflow の実挙動変更）には至らない — filter・遷移・データ意味論は不変で、見た目と通知の磨きに限定。tie-break の反証（Plan Review round 1 P3-1。Scope 7(b) は gated Amendment 2 で除外済み — 実装後レビュー round 1 裁定①）: 除外前の 7(b) 先頭スクロールは SCREEN_DESIGN L263/283/417/419 で既に文書化済みの 4 画面パターンの 5 画面目への水平展開であり新規 workflow 意味論を持ち込まない。Scope 5(a) のマスタ原価反映は既存 mutation（`revisePrice`）へ渡した値の表示反映のみで新規 command / route を要しない。いずれも「不確実なら R3」の不確実性に該当しない。

## Goal

Goal Invariant:

### 最小完了条件

- L3 owner 所感起源の表示磨き 9 件（下記 Scope 1-7〈7(b) は gated Amendment 2 で除外〉+ doc 追記対応分）が実装され、owner 目視で「磨かれた」と確認できる。既存の画面挙動・データ表示の意味・遷移に回帰がない。

### 失敗定義

- 磨き対象の既存機能（取込み・取消・原価更新・改名・hub 検索）のいずれかに挙動回帰が出る
- summary 構造化で UI-07-D13/D14 の規定項目集合（ID / 日付 / filename / 件数・金額等）から項目が欠落する
- CostDiffDialog の成功表示が「色だけに依存」する形になる（UI-02-D15 違反）
- 追記した doc 文言と実装が不一致になる（doc-code drift の新規持ち込み）

### 非目的

- 「状態表現の統一」「戻り先の検索維持」「UI-08 Alert 改善」（冒頭の対象外裁定どおり）
- repo 全体の card-soup 監査（DSR-16 の水平展開 — design-first の後続 task）と `inventory-operator-ui` SKILL.md 更新（sandbox の `.claude/skills` write deny により Claude worker 経路不可 — Codex / owner 経路の後続 task）— gated Amendment 3
- 通知基盤（Toaster 設定）の全画面的変更 — 個別通知の改善に留める
- backend / wire / route / filter 意味論の変更（なし）
- 新規 component 依存の追加（既存 shadcn/ui + success token の範囲）

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

1. **日報完了画面の action ボタン行の構造化**: `src/features/daily-report-import/DailyReportImportPage.tsx` L305-345 の `DailyReportResultStep` — 現況は `CardContent className="space-y-4"` 配下に「日次売上を見る」Button と rollback AlertDialog が別々の block 要素として縦積み（flex 行が存在しない）。これを Z004 側 `src/features/csv-import/components/ResultStep.tsx` L67-112 の `flex flex-wrap gap-2` パターンへ**初めて構造化して統一**する（Plan Review round 1 P3-3: 「間隔調整」ではなく構造化）
2. **rollback summary のラベル付き構造化（両 tab）**: `ResultStep.tsx` L91-96 と `DailyReportImportPage.tsx` L332-337 の文字列連結 summary を、ラベル付きの構造化表示（definition list 等）+ 適切な改行へ（gated Amendment 3: 単一レコード確認 = definition list は DSR-16 に原則適合。囲み階層の点検による最小調整のみ、再構成はしない）。**UI-07-D14 の規定項目集合（Z004: ID / 精算日 / filename / 件数 / 金額、日報: ID / 対象日 / source filenames / 総売上 / 純売上）を欠落なく維持**
3. **追加確認 summary の DSR-16 準拠再構成（gated Amendment 3 で改訂）**: `AdditionalImportConfirmDialog.tsx`（両 tab 共有）— **UI-07-D13 の規定項目 + scroll 可能一覧 + 省略禁止を維持**したまま、DSR-16 の canonical 実例として再構成する: ①既存分は**列の揃った Table**（GOV.UK summary list 型）②**今回分も同一 Table の最終行**（別 tbody / footer、ID 列に「今回」Badge、淡背景 + テキストで色非依存判別 — gated Amendment 4 で definition list 分離を廃止）③取込み日時は人間向け表示 + 空白位置 2 行折返し可 ④dialog は sm:max-w-3xl + table-fixed + ファイル名 whitespace-normal break-words + min-w-0 で**最小 1024×720 に横スクロールなし**（gated Amendment 4、共通 table.tsx は変更せず利用側 override）⑤薄い border だけを唯一のグループ信号にしない ⑥同日追加 Alert を **DSR-03 の上部専用スロット**へ移動（pre-existing 違反の是正を owner 指示で bundle）+ PreviewStep の Badge を「同日データあり」+ TriangleAlert の補助状態表示へ改名
4. **CostDiffDialog の成功・エラー表示の Alert 化**: `src/features/receiving/CostDiffDialog.tsx` L113-131 — 手組み `<p>` を既存 success token パターン（`IntegrityCheckPage` の成功 Alert と同型）へ統一。**UI-02-D15（色だけに依存しない・成功/失敗を同じ行で判別）を維持**（role="status" / テキスト判別を保持）
5. **CostDiffDialog の更新成功後表示**: (a) マスタ原価表示（L104-105、prop 固定値）を更新成功後に新値へ反映 (b) footer 文言（L148-159、常時「見送って閉じる」）を全行処理済み時は「閉じる」等の状態対応文言へ
6. **UI-15 改名成功 toast**: `src/features/suppliers/components/RenameSupplierRow.tsx` L33-48 の成功時に `toast.success`（統合 `MergeSupplierDialog.tsx` L63-64 と対称の完了通知）
7. **hub 系**: (a) 取込み取消の成功通知の視認性改善 — gated Amendment 2 で確定: `useCsvImportFlow.ts` / `useDailyReportImportFlow.ts` の取消完了 `toast.success` 2 箇所の duration を個別指定 8000ms へ（Toaster 全体設定は変更しない） (b) **除外（gated Amendment 2、実装後レビュー round 1 裁定①）**: 先頭スクロールは mount 一律実装が詳細 returnTo 戻りの位置喪失を招き、rollback 起点の one-shot 伝搬は別 route 間で自然な経路がないため本 batch から revert。scroll 復元方針は画面横断の design-first 裁定候補として closeout で backlog 起票
8. **設計 doc の更新 8 箇所（Writer が同 PR で実施、referent 実査済み。旧 SCREEN_DESIGN スクロール追記は Amendment 2 で revert 済み）**: (a) `61-ui-receiving.md` §61.5 1 文 (b) `78-ui-supplier-management.md` §78.6 1 文 (c) `75-ui-integrity-check.md` PageHeader 文言明記 — 以上実施済み。gated Amendment 3 で追加: (d) `docs/design-system/01-decision-rules.md` へ **DSR-16「同型情報のグループ化と囲みの階層」新設**（判断フロー: 比較目的 = 列を揃えた表/structured list、レコード固有操作あり = 一意見出しの summary card、単一レコード確認 = key/value definition list。共通領域は意味階層ごとに 1 つ、内部反復は余白/行区切り、薄い border を唯一のグループ信号にしない。根拠 = NN/g Common Region・GOV.UK summary list） (e) `00-foundations.md` L17 の `--border` 行「4.5:1 境界可視性」誤記を実測比（対 #fafaf9 ≈ 1.20:1）へ修正し、境界の役割記述を DSR-16 と整合させる (f) `03-philosophy.md` へ理論参照 1 段落（共通領域・比較可能性） (g) `02-component-catalog.md` の確認 Dialog へ比較用 variant 追記 (h) `docs/quality/review-checklist.md` へ「同型情報の表示形式が DSR-16 の判断フローに適合するか」1 行
9. **整合性検証の名称統一**: `IntegrityCheckPage.tsx` L185-186 の PageHeader title「在庫整合性チェック」→「**在庫整合性検証**」へ（doc 正本〈75 doc 見出し「在庫整合性検証画面」/ SCREEN_DESIGN L42・62・426「在庫整合性検証」〉に一致させる。サイドバー「整合性検証」〈52 §52.3 メニュー正本表と一致〉は幅制約もあり不変。**統一先の最終確定は Plan Gate 承認に含める**）
10. FE test: 各磨き項目の RTL oracle（Test Design Matrix 参照）。既存 test の削除・無効化・skip 禁止。summary 構造化に伴う既存 assert の文言追随は意味不変の範囲で可
11. commit 分割: 磨き実装（画面群ごとに 1-2 commit）→ doc 追記 → test の順は Writer 裁量、ただし doc 追記と対応実装は同 PR 必須

## Non-scope

- 「状態表現の統一」（§65.6.1 の意図的乖離 — design-first 裁定候補として backlog 残置）
- 「戻り先の検索維持」（導線契約 design-first の所管）
- UI-08 prepare failure Alert（CMD/BIZ 契約拡張要）
- Toaster（`RootLayout.tsx` L69）の position / richColors / duration 等の全画面設定変更
- backend / bindings / route / filter / DB の変更（なし）
- UI-07-D13/D14 の規定項目集合の変更（構造化のみ、集合不変）

## Acceptance Criteria

- AC1: `npm test` green（Matrix T1-T10 の新規・更新 RTL test + 既存 suite）
- AC2: `npm run typecheck` / `lint` / `format:check` / `build` green
- AC3: `cd src-tauri && cargo test` green（backend 無変更の regression 確認）+ `git diff --exit-code src/lib/bindings.ts`（差分ゼロ）
- AC4: UI-07-D13/D14 の項目集合維持 — 構造化後の summary に規定項目の全 token が render される RTL assert（T2 / T3）
- AC5: `rg -c "在庫整合性チェック" src/ --glob '!src/lib/bindings.ts'` = 0（exit 1、旧見出しの全滅。`src/lib/bindings.ts` L290 の 1 件は `integrity_cmd.rs` L10 の Rust doc comment 由来の生成 JSDoc で operator 非可視・backend 非接触の Non-scope により許容 — Plan Review round 1 P1-1）+ `rg -c "在庫整合性検証" src/features/integrity-check/IntegrityCheckPage.tsx` ≥ 1
- AC6: doc 更新 8 箇所（Scope 8 (a)-(h)）の diff が存在し（SCREEN_DESIGN の追記なし = revert 済みを含む） `bash scripts/doc-consistency-check.sh` ERROR 0
- AC7: `bash scripts/local-ci.sh full` CLEAN（L1、exact-HEAD evidence は PR body 所管）
- AC8: Human Gate に owner 目視を含むため Writer 完了条件に `cargo check --release`（CI gate ではない）

## Design Sources

- Requirements / spec: REQ-105（原価改定系）/ REQ-107（取引先）/ REQ-206（記録追跡）— 表示磨きのため要求追加なし
- Function / command / DTO: [55-ui-csv-import.md](../../function-design/55-ui-csv-import.md) §55.2 UI-07-D13（L176-189）/ UI-07-D14（L191-195）、[61-ui-receiving.md](../../function-design/61-ui-receiving.md) §61.5 UI-02-D15（L118）、[78-ui-supplier-management.md](../../function-design/78-ui-supplier-management.md) §78.6 / §78.7（L110 完了通知）、[65-inventory-record-traceability.md](../../function-design/65-inventory-record-traceability.md) §65.6.1（L134 意図的乖離 — 非接触の根拠）、[75-ui-integrity-check.md](../../function-design/75-ui-integrity-check.md)、[52-ui-shared-layout.md](../../function-design/52-ui-shared-layout.md) §52.3
- Screen / UI: `docs/SCREEN_DESIGN.md` L263 / L283 / L417 / L419（先頭スクロール既存パターン）・L42 / L62 / L426（在庫整合性検証）、`docs/design-system/README.md`
- Decision log / ADR: D-056（Opus role — 本編成の遵守対象）、D-062（Plan Reviewer vendor 規則の適用判定）、manual §2 / §3 / §5.4 / §5.6

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status |
|---|---|---|
| Backend function / command / repository / validation / error | — | 触らない |
| Command / DTO / generated binding / wire shape | — | 触らない（AC3 で差分ゼロ確認） |
| DB / transaction / audit / rollback / migration | — | 触らない |
| Screen / UI / route state / Japanese wording | 55 §55.2 / 61 §61.5 / 78 §78.6 / 75 doc | existing sufficient + Scope 8 の軽微追記 3 箇所を updated in this PR（Writer 実施） |
| CSV / TSV / report / import / export format | — | 触らない |
| Durable decision / ADR | — | 新規 durable decision なし（名称統一は既存 doc 正本への実装追随） |

## Registration / Generation Obligations

| 新規追加物 | 登録・生成義務 | 対応 |
|---|---|---|
| Tauri command / route / 画面 / doc 新設 | — | 該当なし（既存画面の表示磨きのみ） |
| REQ coverage 追加 | — | 該当なし（新規 REQ token なし。既存 test の文言追随のみで token 集合不変。万一 Writer が REQ token 付き test を新設する場合は `generate_traceability` 再生成を完了条件に含める — §5.6） |
| Consultation Relay | — | 不使用 |

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| UI-07-D13 / D14 | 55 §55.2 | — | 項目集合は契約、フォーマットは未規定 → 構造化は純表示改善。項目削減は失敗定義 | Scope 2 / 3 | T2 / T3 |
| UI-02-D15 | 61 §61.5 | — | 色だけに依存しない契約を維持したまま success token で視覚統一（IntegrityCheckPage 既存パターン踏襲） | Scope 4 | T4 |
| マスタ原価表示の正確性 | 61 §61.5 + Scope 8(a) 追記 | — | doc は更新後反映を未規定 → 追記で契約化し drift 防止 | Scope 5(a) | T5 |
| 統合との通知対称性 | 78 §78.7 L110（統合の完了通知規定）+ Scope 8(b) 追記 | — | 改名非通知は現行 doc 適合だが、owner 所感の非対称解消には §78.6 追記が正道（無断実装は doc-code drift） | Scope 6 | T6 |
| 先頭スクロール既存パターン | SCREEN_DESIGN 既存 4 画面 | — | **gated Amendment 2 で除外** — mount 一律は詳細戻り UX を壊し event-driven の自然な伝搬経路がない。design-first 裁定候補へ | 除外（revert） | — |
| 名称の doc-code 一致 | 75 doc 見出し / SCREEN_DESIGN L42・62・426 / 52 §52.3 | — | doc 正本は「(在庫)整合性検証」で一貫、PageHeader「在庫整合性チェック」のみ逸脱 → 実装側を doc へ寄せる（doc 側を変える案は 3 doc + D1-D10 表現の広域改訂になり不採用） | Scope 9 | T10 |
| §65.6.1 意図的乖離の非接触 | 65 §65.6.1 L134 | PR #13 設計 | 状態表現統一は本 batch 非対象（Non-scope 明記） | — | 既存 test 凍結 |

## Design Intent Audit

- Source docs can answer what/why without chat history: yes — 各磨き項目の規定節・規定の粒度（項目集合 vs フォーマット）を spec-check で全数実査済み
- Plan-only durable decisions promoted: Scope 8 の 4 追記が表示挙動を doc 契約化（Writer 実施、同 PR）
- Assumptions: success token・Alert・toast の既存パターンが design-system に実在（IntegrityCheckPage / MergeSupplierDialog / ResultStep で実査済み）
- Deferred gaps: 状態表現統一・戻り先検索維持・UI-08 Alert（Non-scope、backlog 追跡）
- Test Design Matrix cites decision IDs: yes
- Absolute guarantee self-check: 「項目集合不変」は D13/D14 の規定項目列挙（55 §55.2 実文言）に依存 — Explore 実査で引用済み、Writer は referent を再確認する

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | 表示層のみ、adapter 非接触 | — |
| Fact check / design decision split | 全 10 項目の実装 file:line・規定 doc 節・契約接触の有無を read-only 調査で実査（引用は本 packet に転記）。状態表現統一の除外は 65 §65.6.1 L134 実文言に基づく | 本 packet |
| Lifecycle / retry | CostDiffDialog の更新成功 → 表示反映（T5）、toast 表示 → 消滅は既存 Toaster 挙動 | Matrix |
| Operator workflow | 表示磨きのみ、operation 手順不変。L3-lite で視認確認 | Human Gate |
| Replacement path | not applicable | — |
| Data safety / evidence | synthetic のみ | Data Safety 節 |
| Reporting / accounting semantics | 金額・件数の表示値は不変（表示形式のみ） | T2 / T3 |
| Manual verification | owner 目視 1 回（9 件まとめて） | Human Gate |
| 環境・再現性 | 新設の環境依存なし | — |

## Design Readiness

- Existing design docs are sufficient because: 磨き 9 件のうち 7 件は規定 doc がフォーマット未規定（= 純表示改善の余地）で contract 適合のまま実装可能。残り 2 件（マスタ原価反映 / 改名通知）+ 名称明記は Scope 8 の軽微追記 3 箇所で doc 契約化してから実装（同 PR、未解決の設計問題なし）
- Source docs updated in this PR: 61 §61.5 / 78 §78.6 / SCREEN_DESIGN 入出庫履歴節 / 75 doc（いずれも 1 文級、Writer 実施）
- Design gaps intentionally deferred: Non-scope 3 件
- Durable decisions discovered and promoted: なし（新規 Decision 不要）

Minimum design checks:

- Layer ownership: UI 層のみ（CMD/BIZ/IO 非接触、AC3 で機械確認）
- Backend function design: 変更なし
- Command / DTO / data contract: 変更なし（bindings 差分ゼロ）
- Persistence / transaction / audit impact: なし
- Operator workflow / Japanese wording: 追記 doc 文言と実装文言の一致を review で突合
- Error / empty / retry / recovery: CostDiffDialog の成功/失敗/部分成功の表示分岐を T4 / T5 で固定
- Testability / traceability: 既存 REQ token 集合不変

## Contract Probe

N/A — 未検証の外部前提なし。依存する前提（規定 doc の文言・既存パターンの実在・実装 anchor 行）はすべて spec-check の read-only 調査で実査済み（Impact Review Lenses Fact check 行）。

## Contract Coverage Ledger

R2 のため必須ではないが、契約接触点を明示する（磨きが契約を壊さない方向の検査が主）:

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| UI-07-D14: rollback summary の規定項目集合（両 tab） | Scope 2 構造化 | T2（全項目 token render assert） | L3 視認 |
| UI-07-D13: 追加確認の項目 + scroll 一覧 + 省略禁止 | Scope 3 構造化 | T3 | L3 視認 |
| UI-02-D15: 色だけに依存しない成功/失敗判別 | Scope 4 Alert 化 | T4（role="status" + テキスト判別維持） | L3 視認 |
| 61 §61.5 追記: 更新成功後のマスタ原価反映 | Scope 5(a) | T5（更新成功 → 新値 render） | L3 視認 |
| footer 状態対応文言 | Scope 5(b) | T5（全行処理済みで「閉じる」系） | L3 視認 |
| 78 §78.6 追記: 改名完了通知（§78.7 と対称） | Scope 6 | T6（toast.success 呼出し assert） | L3 視認 |
| 取消成功通知の視認性（gated Amendment 2 で確定: 取消完了 2 toast の duration 8000ms） | Scope 7(a) | T7（両 hook test で duration: 8000 を実 assert） | L3 視認 |
| 〈除外〉hub 先頭スクロール（gated Amendment 2） | revert（実装・T8・SCREEN_DESIGN 追記とも撤去） | — | design-first 裁定候補へ |
| ボタン行 flex パターン統一 | Scope 1 | T1（wrapper 存在 or snapshot 級 assert、形骸化しない範囲で） | L3 視認 |
| 名称 doc-code 一致（在庫整合性検証） | Scope 9 | T10 + AC5（rg 全滅検査） | L3 視認 |
| 65 §65.6.1 意図的乖離の非接触 | — | 既存 test 凍結（AC1 / AC3） | non-scope |

## Test Plan

Test Design Matrix: [test-matrices/2026-08-29-ui-polish-batch.md](test-matrices/2026-08-29-ui-polish-batch.md)（tricky R2 判定: 10 項目 × 契約維持 oracle のため添付）

- targeted tests: T1-T7・T10（RTL、項目集合維持・表示分岐・通知・名称。T8 は gated Amendment 2 で除外）
- negative tests: CostDiffDialog 失敗行の表示維持（T4）、summary 項目欠落の検出（T2/T3 の全 token assert）
- compatibility checks: 既存 suite green（AC1）、backend 無変更（AC3）
- data safety checks: synthetic のみ
- main wiring/integration checks: なし（表示層のみ）
- Human Gate に owner 目視を含むため Writer 完了条件に `cargo check --release`（AC8）

## Boundary / Wire Contract

N/A — JSON / browser state / CSV / config / DTO / bindings / DB 互換のいずれにも非接触（AC3 で bindings 差分ゼロを機械確認）。

## Review Focus

- 項目集合の欠落（D13/D14）— 構造化で 1 項目でも落ちれば P1
- doc 追記文言と実装の一致（Scope 8 の 3 箇所）
- 既存 test の意味を変える改変が「文言追随」を超えていないか
- Opus 修正案 round: UI 磨きの質（間隔・構造・視認性・アクセシビリティ）を `inventory-operator-ui` の観点で — 非 IT の高齢 operator に対する可読性優先、色のみの状態符号化禁止
- toast 追加が既存の通知パターン（文言スタイル・件数情報）と整合するか

## Spec Contract

Contract ID: SPEC-UI-POLISH-BATCH1-2026-08-29

- rollback / 追加確認 summary は UI-07-D13/D14 の規定項目集合を欠落なくラベル付き構造化で表示する
- CostDiffDialog は成功を success token の Alert 系表示 + テキストで示し、更新成功後はマスタ原価の新値と状態対応 footer 文言を表示する
- 取引先改名成功時に完了通知を出す（§78.6 追記と同文言レベル）
- 整合性検証画面の PageHeader title は「在庫整合性検証」（doc 正本一致）

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| UI-07-D13/D14 | Scope 2/3 | T2/T3 | 項目集合欠落 | Matrix |
| UI-02-D15 | Scope 4/5 | T4/T5 | 色非依存維持 | Matrix |
| §78.6 追記 | Scope 6/8(b) | T6 | doc-実装一致 | PR body |
| 〈除外〉先頭スクロール | gated Amendment 2 で revert | — | — | packet narrative |
| 名称一致 | Scope 9/8(d) | T10/AC5 | rg 全滅 | PR body |

## Data Safety

- 実データ・実 DB file は commit しない。test fixture は synthetic のみ
- L3-lite 手順（各画面の表示確認、synthetic seed は最小限）は Ready 依頼と同時に PR body へ記載
- `.local/ci-evidence/` はローカルのみ

## Implementation Results

Scope 1-7(a), 9 の表示磨きと Scope 8 の doc 追記 3 箇所を実装し、Matrix T1-T7/T10 の RTL oracle を追加した（既存 test の削除・無効化なし、文言追随のみ）。ResultStep / DailyReportResultStep の rollback 確認 summary と AdditionalImportConfirmDialog の追加確認 summary は 3 site 共通のラベル付き `<dl>` 構造化様式に統一し、UI-07-D13/D14 の規定項目集合は維持した。CostDiffDialog は IntegrityCheckPage 踏襲の success token Alert へ統一し、更新成功後のマスタ原価反映と footer 状態対応文言を追加した。RenameSupplierRow に統合と対称の完了 toast を追加、取消完了 toast は個別 duration 延長（T7 gated Amendment 確定）。IntegrityCheckPage の PageHeader title を doc 正本「在庫整合性検証」へ統一。

実装後レビュー round 1（Opus 修正案 + Codex cross、gated Amendment 2、Coordinator 全件 accept）を受けて是正: ①Scope 7(b) 先頭スクロールを完全 revert（mount effect / T8 / SCREEN_DESIGN 追記を撤去、design-first 裁定候補として backlog 起票予定）②T2 を日報 source filenames 3 件 fixture の全件個別 assert へ強化（先頭 1 件化 mutation の survivor 是正）③break-all を break-words へ置換し、複数 filenames を " / " 連結でなく個別行表示（`ExistingImportSummary.filenames` を `string[]` 化）+ 既存分「ファイル名」の col-span-2 欠落是正 ④3 site（ResultStep / DailyReportResultStep / AdditionalImportConfirmDialog）の dl gap トークンと「取込み ID」「合計金額」「ファイル名」ラベル語彙を統一。

ローカル gate（typecheck / lint / format:check / npm test / build / cargo test / cargo check --release / bindings.ts 差分ゼロ / doc-consistency-check.sh ERROR 0 / local-ci.sh full）は是正反映後も全て CLEAN。

owner L3-lite round 1（D13 項目完全性 PASS / 可読性 FAIL）を受けた gated Amendment 3 是正: ①docs 5 点を design-first で先行更新（`01-decision-rules.md` DSR-16 新設、`00-foundations.md` `--border` 誤記修正、`03-philosophy.md` 理論参照、`02-component-catalog.md` 比較用 variant、`review-checklist.md` DSR-16 確認項目）②`AdditionalImportConfirmDialog.tsx` を DSR-16 canonical 実例へ再構成（既存分 = 列を揃えた Table、今回分 = 「今回分」ラベル付き独立領域、取込み日時を人間向け表示、dialog 幅拡大）③rollback summary 2 site（ResultStep / DailyReportResultStep）は definition list のまま囲み階層の二重化だけ解消（再構成なし）④Matrix T3 を構造 assert（列揃え Table / 「今回分」ラベル領域 / 人間向け日時）で拡張、項目完全性 oracle は不変。

owner L3-lite round 2（横スクロールなしの可読性 blocker / 今回分との比較性要改善）を受けた gated Amendment 4 是正: ①`AdditionalImportConfirmDialog.tsx` の横 overflow 是正（`sm:max-w-3xl` + `table-fixed` + 列幅設計〈ID 10% / ファイル名 40% / 合計金額 25% / 取込み日時 25%〉+ ファイル名 cell `whitespace-normal break-words` + 金額・日時 cell `whitespace-normal` + `min-w-0`。共通 `table.tsx` は変更せず利用側 className で override）②今回分の definition list 独立領域を廃止し、同一 Table の最終行（別 tbody、ID 列に「今回」Badge、行に `bg-muted/50`）へ統合③`PreviewStep.tsx`（Z004）の同日追加確認 Alert を画面上部の専用スロットへ移動（DSR-03 pre-existing 違反の是正）+ Badge を「同日データあり」+ TriangleAlertIcon の補助状態表示へ改名④日報 tab 側は Alert 配置が元々正しいため変更なし、Badge のみ対称で同様に改名⑤Matrix T3 を「同一 table 内の今回行」構造 assert + 上部 Alert 配置 assert + Badge 改名 assert へ追随。

owner L3-lite round 3（「owner L3-lite round 3 と裁定」節の Coordinator 裁定どおり、packet 契約不変のため新規 Amendment なし）を受けた是正 3 点: ①`AdditionalImportConfirmDialog.tsx` の `AlertDialogContent` className を `sm:max-w-3xl` から `data-[size=default]:sm:max-w-3xl` へ（共通 `alert-dialog.tsx` L53 の `data-[size=default]:sm:max-w-lg` に対して CSS 詳細度で負けていたため。共通 primitive は不変）②`PreviewStep.tsx`（Z004）の同日追加確認 Alert を日報側と同一の `border-warning bg-warning-soft text-warning-strong` warning tone へ統一（是正前は neutral で非対称だった）③両 tab の「同日データあり」Badge を黒枠（既定 outline）から soft warning token（`border-warning-border bg-warning-soft text-warning-strong`、`StockStatusBadge.tsx` の低在庫バッジと同型）へ。既存 test へ幅 class・Alert tone class・Badge token class の render assert を最小追随。

owner L3（Windows 実機）で発見の P1（merge blocker）を受けた gated Amendment 5 是正: 入庫確定後の原価差分ダイアログが外クリック・Escape・右上×で閉じ、結果画面に再表示経路がないため、owner が再確認のため入庫を再実行し在庫二重加算が実発生（61 §61.5「商品ごとの確認を必須にする」と不整合）。①`CostDiffDialog.tsx` の `DialogContent` へ `onPointerDownOutside` / `onEscapeKeyDown` の `preventDefault` + `showCloseButton={false}` を追加し modal を硬化（共通 `dialog.tsx` は無変更、prop 実在は L45 で確認済み）。既存の `isPending` guard は維持。終了経路は footer の「見送って閉じる / 閉じる」のみ②結果画面への再表示ボタンは不採用（状態管理拡大の回避、見送り時の次回入庫再提示契約が既存 safety net）③`61-ui-receiving.md` §61.5 へ「原価差分ダイアログは明示ボタンのみで閉じる（外側クリック・Escape・× では閉じない）」を追記④Matrix T11: overlay pointer-down / Escape で dialog 残存・×ボタン不在・footer 明示ボタンで閉じるの 4 assert を実装し、是正前の実装へ一時的に戻して red 化することを確認した上で復元（mutation kill 実証）。更新中 disabled・成功後表示の既存 T4/T5/T7-T9 は無変更で green のまま。

Draft PR: https://github.com/higekiri-w/inventory-system-desktop/pull/15（是正反映後に body 更新済み）

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

## Review Response

Fill after review.
- Findings Freeze: not yet frozen; post-freeze exceptions: none.

### Plan Review rally 記録（2026-08-29、append-only）

- round 1（Claude Sonnet 5 独立 fresh context、対象 = plan-first commit `e865aec`）: P1×2（AC5 の rg 対象が生成物 bindings.ts L290〈integrity_cmd.rs L10 の Rust doc comment 由来 JSDoc〉を含み AC3 / Non-scope と自己矛盾 / T7 の「gated Amendment 不要の範囲」に規約根拠なし — Plan Gate 後の packet / Matrix 変更に materiality 閾値の第三経路は存在しない）/ P2×1（Opus round が D-056・manual §3 L36 の投入基準〈難所限定・通常レビューは既存分業維持〉に未適合 — packet の編成検証が vendor 制約のみで投入基準を未検証）/ P3×3（Risk tie-break 反証の明示不足 / SCREEN_DESIGN L34→L42 誤引用 / Scope 1 の「間隔調整」表現が実態〈flex 行の新規構造化〉を過小表現）。Coordinator が P1-1 / P3-2 を rg 実測で独立裏取り（bindings.ts:290 と integrity_cmd.rs:10 の実在、L34 = 入出庫履歴行・L42 = 在庫整合性検証行）し**全 6 件 accept**。是正 = AC5 glob 除外 + 許容差分の根拠明記、T7 の無条件 gated Amendment 化（packet / Matrix 両方 sweep）、Opus 投入の例外記録 1 行（owner 座組決定 2026-08-29 起点、並走実験の簡単水準較正目的、Plan Gate 承認で最終確定）、Risk 反証 2 文、L34→L42 訂正 2 箇所、Scope 1 表現補正

### Plan Gate 承認と implementing 遷移（2026-08-29、append-only）

- round 2 closure（別 Sonnet fresh context、対象 = 是正 commit `b3ca503`）: 6/6 CLOSED（AC5 glob 除外の実測動作確認、例外記録と manual §3 L36 / D-056 実文の literal 突合、SCREEN_DESIGN L42 実文確認を含む）、旧前提 sweep 実質 0（rally 記録内の意図的引用のみ）、delta 起因の新規 findings 0。**P1/P2 残 0**
- owner Plan Gate 承認（2026-08-29、介入 1/3）。承認に含まれる確定 2 点: ①Opus 5 投入の例外適用（manual §3 L36 に対する例外、Workflow State 記載の根拠どおり）②名称統一先 = PageHeader「在庫整合性検証」（サイドバー不変）
- plan-gate → plan-approved → implementing の materialize evidence: 上記 P1/P2=0、plan-first commit `e865aec` + rally 是正 `b3ca503` = Plan Commit が全実装 commit に先行（実装 commit 未作成）、Writer は Claude Sonnet 5 subagent（worktree isolation、発注書駆動）

### 実装後レビュー round 1 記録（Opus 修正案 + Codex cross、2026-08-29、append-only）

- Opus 5 修正案 claims-producer round（§5.4 低制約、対象 = `c88c814..c5298c2`）: P1×2（hub の mount 一律 `scrollPageToTop` が詳細 returnTo 戻りのスクロール位置を毎回失わせる — 既存 5 site は mutation handler 内の event-driven で mount 契機は本 hub のみの新規パターン / 追加確認 dialog 既存分の「ファイル名」に col-span-2 欠落 — 半幅 + break-all で Scope 3 が排除対象とした任意位置折返しを再生産）/ P2×2（T7 の gated Amendment 未記録 / 3 site の dl gap トークン・ラベル語彙の不統一）/ P3×2（duration 延長のみの視認性改善は L3 所感次第で Alert 併用検討余地 / dl が aria-describedby に含まれない既存同型制約）
- Codex cross review: P2×5 / P3×1 + mutation 抜き取り 6 件実測（**survivor 1 = 日報 D14 の filenames を先頭 1 件化しても T2 が GREEN**）。P2-1 = Opus P1-1 同型（mount scroll、T8 は mount 時呼出しのみ固定で回帰未検出）/ P2-2（T2 oracle 弱 — 上記 survivor）/ P2-3 = Opus P2-1 同型（T7 の事前 Amendment 不在）/ P2-4（PR body の L1 evidence が HEAD `c88c814` + TREE_STATE=DIRTY で `c5298c2` の clean exact-content 証跡でない + 遷移 commit subject 非 canonical — local-verified 判定の根拠喪失）/ P2-5（break-all は「任意位置折返し回避」と逆方向、4 箇所）/ P3-1（SCREEN_DESIGN 追記自身による既存例参照の行ずれ）
- Coordinator 裁定（実証: scrollPageToTop の mount effect L82-84 と col-span-2 非対称を worktree 実読で確認、L1 evidence の DIRTY は Codex 実測引用を採用）: **全件 accept**。
  1. Scope 7(b) は本 batch から**除外**（gated Amendment 2）— persistent `<main>` による stale scroll は hub 固有でなく画面横断の事象であり、mount 一律は最基本動線（一覧→詳細→戻る）を壊す。rollback 起点の one-shot 遷移情報は rollback 画面と hub が別 route のため自然な伝搬経路がなく、scroll 復元方針は design-first 裁定候補として closeout で backlog 起票。実装 revert（mount effect + T8 + SCREEN_DESIGN 追記）
  2. T7 は gated Amendment 2 で確定記録（取込み取消 2 toast の duration 8000ms）
  3. T2 を日報 filenames 全件の個別 assert へ強化（Codex mutation survivor の是正）
  4. break-all を break-words へ置換し filenames は full-width / リスト表示化（4 箇所）+ dl gap・ラベル語彙（ID / 金額系）の 3 site 統一（Opus P2-2 併合）
  5. local-verified 判定を取り下げ implementing へ backtrack。是正後の clean exact HEAD で L1 full を再取得し、canonical な state-only 遷移で再実体化
  6. Opus P3-1 は owner L3-lite 所感待ちの disposition、P3-2 はアクセシビリティ磨き候補として closeout で backlog、Codex P3-1 は 7(b) revert により行ずれが自己解消（撤回後に参照先の実位置を再確認）
- 是正のため implementing へ backtrack（本 commit）

### Final Review 記録（2026-08-29、append-only）

- Sonnet 独立 fresh context の Final Review（対象 = Reviewed Content HEAD `ca8637e`）: 実装後レビュー round 1 の全 findings **CLOSED**（7(b) revert の完全性を rg 実測 0 件で確認 / T2 強化は `filenames.slice(0,1)` 変異の実注入 → red → 復元 green で kill を実証 / break-words 置換は対象 4 箇所完全〈src/ 残存の break-all 8 件は Non-scope 画面の既存分〉/ dl gap・語彙 3 site 統一 / T7 Amendment 記録済み）。AC1-AC6 独立再実測 全 PASS（bindings 差分ゼロ・doc-consistency ERROR 0 含む）。Amendment 2 適合（旧前提の残骸は append-only narrative 内のみ）、是正 delta 全 hunk が裁定 6 点に 1 対 1 対応、`ExistingImportSummary.filenames` の string→string[] は FE 内部型で wire 非接触、state-only `ca8637e` の hunk audit 適合（canonical subject 一致）、既存 test 凍結維持（撤去は T8 のみ = 7(b) revert の正当な随伴）。**新規 P1/P2/P3 = 0**
- L1 evidence の Coordinator 独立検分: evidence log 実 envelope の END_HEAD_SHA = `ca8637e` / END_TREE_STATE=CLEAN / MERGE_EVIDENCE_VALID=true / RESULT=PASS（中間の別 SHA block は local-ci self-test の fixture envelope — 既知の型）
- local-verified → independent-review → human-confirm の materialize evidence: 上記 Final Review P1/P2 = 0。残 Human Gate = owner L3-lite 目視（9 件）+ Ready 承認 + merge

### owner L3-lite round 1 と可読性 FAIL の裁定（2026-08-29、append-only）

- owner L3-lite round 1（content = Reviewed Content HEAD `ca8637e`）: D13 項目完全性・構造化 = PASS / **可読性（磨かれて見やすい）= FAIL**（追加取込みは未実行・DB mutation なしで停止）。Goal Invariant「owner 目視で磨かれたと確認できる」の未達
- owner relay の Codex 調査を Coordinator が独立確認（全 claims 裏付けあり）: ①`00-foundations.md` L17 の `--border` #e7e5e4「4.5:1 境界可視性」は誤記 — 対 `--background` #fafaf9 の実測算出は約 1.20:1（直下の muted-foreground 行〈本物の AA 4.5:1〉からの転記ミスの形。薄い border を唯一のグループ信号とする正本根拠が不成立）②`01-decision-rules.md` は DSR-15 まででカード/表選択・囲み階層・共通領域の判断規則が不在（該当語 0 hit）③理論引用（NN/g Common Region = 共通領域は強いグループ化原理だが囲み多用は clutter / GOV.UK summary list = 小量の関連 key-value はカードでなく summary list）は確立文献の正確な要約
- Coordinator 裁定（owner 承認 2026-08-29、介入 3/3 消化 + 再 L3・Ready・merge の超過 +2 を事前承認済み）: ①可読性 FAIL は本 PR の責務として是正 — 追加取込み dialog を **DSR-16 の canonical 実例**として再構成（既存 3 件 = 列の揃った structured list / 今回分 = 「今回」ラベル付き独立領域 / 日時の人間向け表示 / dialog 幅拡大）、rollback summary 2 site は DSR-16 照合の最小調整に留める ②gated Amendment 3（Scope 2/3 改訂 + docs 5 点追加 + Matrix T3 追随）③repo 全体の card-soup 監査と `inventory-operator-ui` SKILL.md 更新は**後続 task へ分離**（後者は sandbox の `.claude/skills` write deny により Claude worker 経路では書けず Codex / owner 経路）。新 Skill 不要・memory 不要（durable 判断は repo docs 正本）の Codex 見解に同意
- 是正のため implementing へ backtrack（本 commit）

### Final Review round 2 記録（2026-08-29、append-only）

- Sonnet 独立 fresh context の delta closure（対象 = `7be74f0..efe4d24`、Reviewed Content HEAD を `efe4d24` へ更新）: Amendment 3 の Scope 3 5 要素・Scope 8 (d)-(h) 5 doc に全 hunk が 1 対 1 対応、scope 外 hunk なし。DSR-16 は既存 DSR 文体に整合、`00-foundations.md` の修正値は reviewer 独立検算 **1.2022:1** で doc 記載 ≈1.20:1 と一致。dialog は Table 化（列見出し 4 種 + 行区切り + 外周囲み 1 つ）+「今回分」独立領域 + formatDateTime + sm:max-w-2xl（既存慣行整合）。T3 は role ベースの構造 assert + 全件表示 assert で mutation 耐性維持。AC1/AC2/AC3/AC6 再実測 全 PASS。遷移 commit `efe4d24` の hunk audit 適合（非 literal subject の STATECAP 経緯は commit body + PR body で追跡可能）。既存 test 凍結維持。**新規 P1/P2/P3 = 0**
- local-verified → independent-review → human-confirm の再 materialize evidence: 上記 P1/P2 = 0。STATECAP aggregate cap 3 消化済みのため本遷移も content-riding 形（非 literal subject、経緯は本節と PR body が追跡）。残 Human Gate = owner L3-lite round 2（再目視）+ Ready 承認 + merge

### owner L3-lite round 2 と裁定（2026-08-29、append-only）

- owner L3-lite round 2（content = `efe4d24`）: D13 項目完全性 PASS / 既存分の構造化・比較性 PASS / **横スクロールなしの可読性 FAIL（blocker — 日時列と主操作ボタンが画面内で完結しない）** / 今回分との比較性 要改善。DB mutation なし。findings は owner 観察 + Codex 言語化の relay で、Coordinator が独立確認: ①DSR-03 は実在し「同日追加確認」を**上部 Alert 帯の専用スロット**へ literal 規定 — 現配置（紐付け結果・エラー詳細の後方）は pre-existing 違反（本 PR の PreviewStep 接触は filenames 追随 2 行のみと diff 実測）②`whitespace-nowrap` は共通 `src/components/ui/table.tsx` の TableHead / TableCell 既定 ③「追加確認」Badge は PreviewStep L55 に実在
- Coordinator 裁定（全件 accept、gated Amendment 4）: ①**横 overflow 是正** — dialog を sm:max-w-3xl へ + table-fixed + ファイル名 cell の whitespace-normal break-words（共通 table.tsx は変更せず利用側 className override）+ 日時の空白位置 2 行折返し + grid/container へ min-w-0。**L3 条件: 最小 1024×720 で横スクロールなし** ②**DSR-03 整合** — 同日追加 Alert を画面上部専用スロットへ移動（pre-existing 違反の是正を owner 指示で bundle） ③Badge は「同日データあり」へ改名 + TriangleAlert アイコンの補助状態表示化（主情報は上部 Alert が担う役割分担） ④**今回分を同一 Table の最終行へ**（別 tbody / footer、ID 列に「今回」Badge、淡背景 + テキストで色非依存判別。definition list 分離を廃止 — 比較対象は列を揃える DSR-16 の帰結）
- 介入実績: 5 回中 4 消化（Plan Gate / L3r1 / DSR-16 承認 / L3r2）。残り L3r3 + Ready で **+1 超過見込み**を owner へ明示
- 是正のため implementing へ backtrack（本 commit）

### round 3 delta closure と human-confirm 再遷移（2026-08-29、append-only）

- Writer round 3（`8ef3597..295bfcd`）: 横 overflow 是正（sm:max-w-3xl = 内容幅 ≈720px、table-fixed 10/40/25/25%、filename whitespace-normal break-words、日時 2 行折返し、min-w-0。金額 cell への whitespace-normal は同目的の技術補完として Writer 透明申告）+ 今回行の同一 Table 最終行化（「今回」Badge + 淡背景）+ DSR-03 整合（Z004 Alert を最上部へ、日報側は元々適合を rg 確認、Badge は両 tab「同日データあり」+ TriangleAlert へ）+ T3 追随。L1 実 envelope = HEAD `295bfcd` / TREE_STATE=CLEAN / RESULT=PASS
- delta closure = **Coordinator 直接検分**（独立 subagent round は省略。理由: R2 で独立 review は本 PR で既に 2 full round 実施済み、delta は 6 file の狭範囲、owner の時間効率要請。検分実測: 共通 table.tsx の diff 0 行 / table-fixed + w-[10%] 列幅の実在 / 「同日データあり」改名 / 今回行の同一 Table 化と設計意図 comment / Writer gate 全 green + L1 CLEAN）。P1/P2 相当の未解決 finding なし
- local-verified → independent-review → human-confirm の再 materialize（content-riding 形、STATECAP 経緯は既records）。残 = owner L3-lite round 3 + Ready + merge（介入 +1 超過の承認を L3 依頼に併記）

### owner L3-lite round 3 と裁定（2026-08-29、append-only）

- owner L3-lite round 3（content = `295bfcd`）: 上部 Alert 移動 / Badge 文言 + アイコン / 表への今回行統合 / 横スクロール消失 = **PASS**。意図した dialog 幅の適用 = **FAIL** / Z004 Alert の warning tone = 要是正。findings は owner 観察（実機写真）+ Codex 言語化の relay
- Coordinator 独立確認（全 3 点 true positive）: ①共通 `alert-dialog.tsx` L53 に `data-[size=default]:sm:max-w-lg` が実在 — 属性 selector 付きで呼出し側の素の `sm:max-w-3xl` より CSS 詳細度が強く、720px 設計が実 DOM で負けて default 幅のまま ②Z004 `PreviewStep.tsx` L58 の同日追加 Alert は neutral、日報側 L152 は `border-warning bg-warning-soft text-warning-strong` で非対称 ③warning token 群は実在
- 裁定（packet 契約不変の実装是正のため Amendment 追加なし、本 narrative が記録）: ①呼出し側を `data-[size=default]:sm:max-w-3xl` へ（共通 primitive の wide size 新設は今回不採用 — 単一利用箇所に最小変更） ②Z004 上部 Alert を日報側と同一の warning tone へ統一 ③Badge は黒枠でなく soft warning token（`border-warning-border bg-warning-soft text-warning-strong` 系）— 黒枠は補助状態を主警告より強く見せ operable にも見える情報階層の逆転のため不採用。強弱設計 = 上部 Alert（warning、主）> Badge（soft warning、補助）> 表（具体確認）
- 是正のため implementing へ backtrack（本 commit）

### round 4 delta closure と human-confirm 再遷移（2026-08-29、append-only）

- Writer round 4（`8415479..291e32d`、6 file / +56-10）: ①幅 override `data-[size=default]:sm:max-w-3xl`（AdditionalImportConfirmDialog L87、共通 primitive 無変更）②Z004 同日追加 Alert を日報側と同一の warning tone へ（PreviewStep L60）③両 tab Badge を soft warning token へ（`border-warning-border bg-warning-soft text-warning-strong`、StockStatusBadge 同型・token 実在を globals.css で確認）+ class assert test 追随。L1 実 envelope = HEAD `291e32d` / CLEAN / PASS
- delta closure = Coordinator 直接検分（3 点の class を diff 実測で確認、round 3 と同じ R2 裁量根拠）。commit subject の「gated Amendment 5」表記は informal shorthand で、tracked Amendments 行は 3 件のまま + 非追加の明記あり — 正本無矛盾を検分済み
- local-verified → independent-review → human-confirm の再 materialize（content-riding 形）。残 = owner L3-lite round 4 + Ready + merge

### owner L3 で発見の P1（CostDiffDialog 暗黙 dismiss）と裁定（2026-08-29、append-only）

- owner L3（Windows 実機）: 入庫確定後の原価差分ダイアログが外クリック・Escape・右上×で閉じ、結果画面に再表示経路がない。アプリ前面化のための余白クリックだけで dialog が消え、再確認のため入庫を再実行して記録 ID 3→4・在庫二重加算が実発生。**merge blocker**
- Coordinator 独立確認（全 true positive）: `CostDiffDialog.tsx` L76-81 の onOpenChange guard は isPending 中のみ / `ReceivingPage.tsx` の結果画面に再 open 経路なし / 61 §61.5 L35 Why「商品ごとの確認を必須にする」と暗黙 dismiss が不整合（L131 の次回入庫再提示は残るが、再確認のための入庫再実行という危険誘導が実演された）。dismissal 意味論は PR #5 由来の pre-existing で本 batch は表示のみ接触 — ただし当 dialog は Scope 4/5 の当事者であり本 PR で是正する
- 裁定（gated Amendment 5）: ①CostDiffDialog のみ modal 硬化 — DialogContent へ `onPointerDownOutside` / `onEscapeKeyDown` の preventDefault + `showCloseButton={false}`（共通 dialog.tsx は無変更、prop 実在を L45 で確認済み）。終了経路は footer の「見送って閉じる / 閉じる」のみ ②結果画面への再表示ボタンは今回**不採用**（状態管理拡大の回避、見送り時の次回入庫再提示契約が safety net） ③61 §61.5 へ「原価差分ダイアログは明示ボタンのみで閉じる（外側クリック・Escape・× では閉じない）」1 文追記 ④回帰 test T11: overlay pointer-down / Escape で dialog 残存・×ボタン不在・footer 明示ボタンで閉じる・更新中 / 成功後の既存表示契約維持
- 是正のため implementing へ backtrack（本 commit）

### round 5 delta closure と human-confirm 再遷移（2026-08-29、append-only）

- Writer round 5（`19f4d65..aafa06a`）: CostDiffDialog の modal 硬化（showCloseButton={false} + onPointerDownOutside / onEscapeKeyDown preventDefault、L89-94。共通 dialog.tsx 無変更・isPending guard 維持・終了は footer 明示ボタンのみ）+ 61 §61.5 の 1 文追記 + T11（4 assert、是正前差し戻しで red を実証後 green 復元）。既存 T4/T5/T7-T9 green 維持。L1 実 envelope = HEAD `aafa06a` / CLEAN / PASS
- delta closure = Coordinator 直接検分（dismissal 3 prop・doc 追記・T11 の data-slot assert を diff / rg 実測。R2 裁量根拠は round 3 と同）。P1/P2 相当の未解決 finding なし
- local-verified → independent-review → human-confirm の再 materialize（content-riding 形）。残 = owner L3 最終確認 + Ready + merge

### owner L3 最終確認 PASS と ready-hosted-final 遷移（2026-08-29、append-only）

- owner Windows native L3 最終確認（対象 PR HEAD `118a5a4` / 是正 content `aafa06a`）: **判定 PASS（P1/P2 残なし）** — CostDiffDialog の暗黙 dismiss 是正 5 項目（× 不在 / 外クリック残存 / Esc 残存 / footer で閉じる / 表示項目）全 PASS、追加取込みダイアログ 5 項目（幅 / 横スクロールなし比較 / warning tone Alert / soft warning Badge / 今回行の同一表最終行）全 PASS。検証データはアプリ内復元機能で baseline へ復元済み、追加取込みは未実行
- owner P3 follow-up（非阻害）: 原価差分ダイアログの選択結果説明の充実（入庫記録は保存済み / 更新の意味 / 見送りの意味の 3 点明記）— 別 change の改善候補として closeout で backlog 起票
- human-confirm → ready-hosted-final の evidence: Human Gate 全項目解決（上記 PASS）+ owner の bundled 依頼（PASS + Ready 進行）への応答として PASS 報告受領。遷移は Draft のまま本 commit で作成し、その exact HEAD で L1 full → PR body 全面 refresh → owner Ready 化 → hosted 三点一致 → merge（PR #14 と同じ委任範囲）の順
