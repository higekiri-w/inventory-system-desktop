# Plan Packet — UI consistency batch（検索欄 live 統一 / 52 §52.3 routing 表 / 65 §65.5 JAN 行）

## Workflow State

- Phase: archive
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: a433e44
- Amendments: 0ff2d7f
- Coordinator: Claude Fable 5（main thread）
- Writer: Codex（GPT-5.6）
- Plan Reviewer: Claude Sonnet 5（independent subagent）
- Final Reviewer: Claude Sonnet 5（independent subagent）
- Reviewed Content HEAD: 324e9c7
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required
- Human Gate: none（L3 = Windows native で 3 項目〈live 検索 / IME 確定挙動 / label 撤去後の視覚整合〉PASS + Ready 承認、2026-08-04 owner。介入 3/3）

起票時の状態遷移: kickoff → spec-check → plan-draft → plan-gate を本 plan-first commit で materialize する。spec-check → plan-draft の skip 根拠 = Design Readiness が既存設計書（50 UI-01a-D9 / 59 §59.1-59.2 / 65 / 52 / 73）で実装十分と引用（設計新設なし、既存契約の適用と docs 実態同期のみ）。

## Owner Effort Budget

- 介入回数上限: 3
- 実働時間上限: 30分
- relay 往復上限: 2

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
入出庫履歴の検索欄 live 統一は URL search state（`q` param）と operator-facing UI 挙動に触れる（DEV_WORKFLOW Risk Tiers: route/search state / operator workflow は R3）。52 §52.3 と 65 §65.5 は docs-only（単独なら R0-R2 相当）だが、batch 全体の tier は最高値の R3 を採る。

## Goal

Goal Invariant:

### 最小完了条件

- 入出庫履歴（`/inventory/records`）の商品検索欄が、商品一覧・在庫照会と同じ共有 SearchBar live 型（`debounceMs=200`、Enter 即時 flush、IME composing guard、no-trim URL 結線）で動作する。
- 52 §52.3 のルーティング表・URL 設計根拠段落、および 65 §65.5 の JAN 行が、実装の実態（実 route / 5 詳細画面 + 棚卸し列 `StocktakeItemDetail` の全 6 DTO に JAN field なし）と一致する。

### 失敗定義

- 検索欄の既存機能（キーワード絞り込み、URL 復元、IME 入力、q 変更時の page reset、他 filter との併用）のいずれかが退行する。
- docs 同期が実装と新たな乖離を生む（誤った URL / route file / 項目規定を書き込む）。

### 非目的

- 取引 4 画面（入庫 / 返品交換 / 手動販売 / 廃棄）+ 棚卸しの商品追加欄・対象確認欄の live 化 / autocomplete 化。これらは UI-04-D4 / UI-05-D5 系列の明示的 commit 型設計（scan-like flow）であり、live 候補プレビュー化は別 change 候補として `Plans.md` backlog に記録済み（owner 裁定 2026-08-04）。
- 5 詳細画面への JAN 列追加（選択肢 B）。owner 裁定（2026-08-04）で選択肢 A =「§65.5 を実態同期」を採用。B は要望発生時に別 change。
- 記録ID（`type="number"`）等、自由テキスト検索欄でない入力の変更。
- 在庫少一覧（実体 = 在庫照会 `/stock`、live 型適用済み）への変更。
- raw message 直接表示の describeError 化（別 R3 として起票予定）。
- `docs/archive/` 配下の陳腐化 URL 是正（設計合意時点のスナップショットとして凍結）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。

## Scope

- `src/features/inventory-records/InventoryRecordsPage.tsx`: 商品検索欄（`records-keyword`）を自前実装（`keywordDraft` state + 自前 IME composing guard + 即時 `updateKeywordSearch`）から共有 `SearchBar`（`src/components/patterns/SearchBar.tsx`、live 型 `debounceMs=200`）へ置換。value 結線は raw `search.q ?? ""`（UI-01a-D9 同型、normalized 値を value に渡さない）。`q` 変更時の page 既定 reset（現行 `updateKeywordSearch` の `resetPage = true`）は維持。IME 意味論は SearchBar live 実態へ統一する（Plan Gate round 1 P1-2 裁定 = 案 a）: SearchBar の composing guard は Enter keydown のみで、変換中の中間文字列が debounce 経由で `q` へ一時反映されることは商品一覧・在庫照会と同一の live 型既定挙動として許容。現行の「全 onChange を composing 中ブロック」する自前 guard は統一のため意図的に廃止する。`placeholder` prop は指定せず SearchBar 既定値へ統一する（表示文言は現行「商品コード・JAN・商品名」→「商品コード・商品名・JANで検索」へ変化し、商品一覧・在庫照会と同一文言になる — round 2 P2-A 裁定）。外付け `<label htmlFor="records-keyword">商品検索</label>` は UI-01a-D9 同型に倣い撤去する（`LiveSearchBar` は `id` を受け取らず label が宙に浮くため。アクセシブルネームは SearchBar 既定 aria-label「商品検索」が維持 — round 2 P2-B 裁定）。
- `src/features/inventory-records/InventoryRecordsPage.test.tsx`: live 結線（debounce 発火 / Enter 即時 flush / 変換確定 Enter の誤発火なし / compositionend 後の最終文字列反映 / no-trim / page reset）の regression test を追加・更新。既存の「composing 中 onChange 非発火」assert は本 batch の契約変更（SearchBar live 意味論への統一）に伴う新契約への**改稿**であり、test の削除・skip ではない（PR body で明示する）。
- `docs/function-design/65-inventory-record-traceability.md`: **TRACE-D12** の正式エントリを §65.2 設計判断 table（TRACE-D1〜D11 と同構造: Spec / Decision ID / 決定 / 理由）へ追加し、§65.4.1 共通フィルタは備考で「live 型（TRACE-D12）」を参照する形に留め、§65.8.1 の該当記述を同期、§65.11 Test Focus へ TRACE-D12 の bullet を追加。§65.5 詳細項目表は「商品コード / JAN / 商品名 / 部門」の 1 行を「商品コード / 商品名 / 部門」（全列 yes 維持）と「JAN」（全列 no）の 2 行へ**分割**して実態（5 詳細画面 + 棚卸し列 `StocktakeItemDetail` の全 6 DTO に JAN field なし・5 詳細画面で JAN 非表示。商品コード / 商品名 / 部門は表示中）へ同期し、変更履歴に owner 裁定（選択肢 A、2026-08-04）を記録。
- `docs/function-design/59-ui-shared-patterns.md`: §59.1 SearchBar 採用箇所表へ入出庫履歴（live 型）を追加。
- `docs/function-design/52-ui-shared-layout.md`: §52.3 の UI-07 行（URL `/pos/csv-import` → `/csv-import`、route file `src/routes/pos/csv-import.tsx` → `src/routes/csv-import.tsx`〈layout〉+ `src/routes/csv-import/index.tsx`〈index〉、備考「URL ドメインは POS」の是正含む）/ UI-08 行（URL `/pos/plu-export` → `/products/plu-export`、route file `src/routes/pos/plu-export.tsx` → `src/routes/products/plu-export.tsx`、備考是正含む）/ UI-10 行（route file `src/routes/stocktake/index.tsx` → `src/routes/stocktake.tsx`）+ 「URL 設計の根拠」段落の `/pos/*` 列挙削除。
- `docs/function-design/73-ui-stocktake.md`: route file 表記 `src/routes/stocktake/index.tsx` → `src/routes/stocktake.tsx` の是正（52 §52.3 UI-10 行と同根 drift）。
- `docs/function-design/90-traceability.md`: TRACE-D12 追加に伴う自動再生成（`cd src-tauri && cargo run --bin generate_traceability`。AUTO-GENERATED、手動編集禁止のまま。diff は TRACE-D12 起因の delta のみであること）— **gated Amendment 1**（Writer fail-closed 起源、選択肢 A 採択）。
- `Plans.md`: active packet link の登録 + 商品追加欄 live 候補プレビュー（autocomplete）化の backlog 起票（plan-first commit）。消化項目の打ち消しは Post-Merge Closeout で実施。

## Non-scope

- 商品追加欄 / 対象確認欄（5 画面）の挙動変更（上記 非目的参照）。
- `SearchBar` 部品本体の変更（既存 2 画面で採用実績のある部品をそのまま使う。部品規約は §59.2 / catalog ⑨ 不変）。SearchBar の composing guard は Enter keydown のみ（onChange/debounce 経路に guard なし）であり、これを含めて live 型の既定意味論として採用する（P1-2 裁定 a。onChange guard を SearchBar へ追加する案 b は商品一覧・在庫照会の挙動変更と scope 拡大を招くため不採用）。
- backend（Rust / CMD / BIZ / IO）・DB・bindings の変更（本 batch は frontend 1 file + docs のみ。`bindings.ts` diff ゼロ）。
- 記録種別 / 日付 / 記録ID / 部門 / 状態 filter の挙動変更。
- `docs/archive/` 配下の記述変更。

## Acceptance Criteria

- AC1: 入出庫履歴の商品検索欄に入力すると、debounce 200ms 経過後に URL `q` param と一覧 query が更新される。Enter は debounce を待たず即時反映する — `InventoryRecordsPage.test.tsx` の live 結線 test（fake timer で 200ms 前後の発火有無を assert）。
- AC2: 変換確定 Enter（`isComposing`）で search flush が誤発火せず、変換確定後は最終文字列が `q` へ反映される。変換中の中間文字列が debounce 経由で一時反映されることは、商品一覧・在庫照会と同一の live 型既定挙動として許容（Plan Gate round 1 P1-2 裁定） — 同 test file の IME test。
- AC3: `q` は raw のまま URL に結線され、value にも raw `search.q` が渡る（trim は query 構築時の normalized 側のみ、UI-01a-D9 同型） — 前後空白入りキーワードでの no-trim assert。
- AC4: `q` 変更（クリア含む）で page が既定へ reset される（現行挙動維持） — page reset test。
- AC5: `rg -n "/pos/" docs/function-design/` が 0 hit（52 是正後、function-design 配下から陳腐化 URL が消える）。
- AC6: 52 §52.3 の URL 列・route ファイル列が `src/routeTree.gen.ts` の実 route と一致する — Plan/Final Review の実 route 突合。
- AC7: 65 §65.5 詳細項目表で「商品コード / JAN / 商品名 / 部門」行が「商品コード / 商品名 / 部門」（全列 yes 維持）と「JAN」（全列 no）の 2 行へ分割される — 5 つの `*RecordDetailItem` 型（Receiving / Return / ManualSale / Disposal / CsvImport、`src/lib/bindings.ts`）+ 棚卸し列の対応型 `StocktakeItemDetail` の型定義に JAN field 0 hit のまま + 変更履歴に owner 裁定 A（2026-08-04）の行が存在（棚卸しの record detail 画面自体は未実装だが §65.5 の列としては存在するため DTO 突合対象に含める）。
- AC8: 既存 filter（種別 / 日付 / 記録ID / 部門 / 状態）、filter-empty reset action（SPEC-UIBB-1/2）、一覧⇄詳細の条件保持（TRACE-D11）が不変 — `npm test -- InventoryRecordsPage` exit code 0（既存 test 削除・skip なし）。
- AC9: `bash scripts/local-ci.sh full` CLEAN（L1）、`bindings.ts` diff ゼロ。
- AC10: 検索欄のアクセシブルネームが「商品検索」のまま到達可能（`findByLabelText("商品検索")` green）+ placeholder が SearchBar 既定「商品コード・商品名・JANで検索」である assert、外付け `<label>` / `id="records-keyword"` の残存 0（`rg -n "records-keyword" src/` 0 hit）。

## Design Sources

- Requirements / spec: `docs/spec/requirements.md` REQ-206（一覧検索）
- Architecture: 変更なし（UI 層内の部品置換のみ）
- Function / command / DTO: `docs/function-design/65-inventory-record-traceability.md` §65.4.1 / §65.5 / §65.8.1、`docs/function-design/59-ui-shared-patterns.md` §59.1-59.2、`docs/function-design/50-ui-product-list.md` UI-01a-D9（live 型契約の原型）、`docs/function-design/52-ui-shared-layout.md` §52.3、`docs/function-design/73-ui-stocktake.md`
- DB: 変更なし
- Screen / UI: `docs/design-system/02-component-catalog.md` ⑨（SearchBar）
- Decision log / ADR: JAN 裁定（選択肢 A）は本 packet + 65 変更履歴に記録（decision-log 昇格は不要規模と判断）

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status |
|---|---|---|
| Backend function / command / repository / validation / error | 変更なし | 該当なし |
| Command / DTO / generated binding / wire shape | 変更なし（bindings diff ゼロを AC9 で確認） | 該当なし |
| DB / transaction / audit / rollback / migration | 変更なし | 該当なし |
| Screen / UI / route state / Japanese wording | 65 §65.4.1（TRACE-D12 新設）/ §65.5 / §65.8.1、59 §59.1、52 §52.3、73 | updated in this PR |
| CSV / TSV / report / import / export format | 変更なし | 該当なし |
| Durable decision / ADR | JAN 裁定 A は 65 変更履歴 + 本 packet | updated in this PR |

## Registration / Generation Obligations

REQ coverage 追加（TRACE-D12 の設計書追加）に伴い `cargo run --bin generate_traceability` で `90-traceability.md` を再生成する（gated Amendment 1 で Scope へ明示。起票時は「該当なし」と誤判定しており、Writer fail-closed の local-ci changed traceability gate が検出した）。新規 command / route / 画面 / function-design doc の追加はなし。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| REQ-206 | 65 §65.4.1 | TRACE-D12（新設） | 商品検索欄を SearchBar live 型（debounceMs=200）へ統一。理由 = 商品一覧・在庫照会との操作一貫性（UI-01a-D9 同型）+ 自前 IME guard 実装の drift 解消。rejected = 自前実装の維持（部品規約の二重管理）、commit 型化（live 型が filter 欄の全社契約） | `InventoryRecordsPage.tsx` | live 結線 / IME / no-trim / page reset test |
| REQ-206 | 59 §59.1 | （表更新のみ） | SearchBar 採用箇所表へ入出庫履歴を追加し採用実態と一致させる | `59-ui-shared-patterns.md` | doc review |
| — | 52 §52.3 | （実態同期のみ） | UI-07/UI-08/UI-10 行と URL 設計根拠段落を実 route へ同期。rejected = route 側を doc に合わせる変更（既存 URL は PR #57 batch A 等で確定済み） | `52-ui-shared-layout.md` / `73-ui-stocktake.md` | AC5 rg sweep + review |
| REQ-206 | 65 §65.5 | （owner 裁定 A、2026-08-04） | JAN を単独行へ分割し全列 no で実態同期（商品コード / 商品名 / 部門は yes 維持）。rejected = 選択肢 B（5 画面 JAN 列追加。技術コストは小さいが operator 業務上の必要性を示す記録が皆無 — 現店舗は部門キー運用） | `65-inventory-record-traceability.md` | doc review + DTO 突合 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: yes（TRACE-D12 を 65 に、採用箇所を 59 に、JAN 裁定を 65 変更履歴に明記する）
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: JAN 裁定 A → 65 変更履歴。autocomplete 化の見送りと候補記録 → `Plans.md` backlog
- Assumptions and constraints: SearchBar 部品の live 型挙動（debounce / Enter flush / Enter keydown の isComposing guard。onChange 経路に guard なし）は既存 2 画面 + 部品単体 test で固定済み。現行 `updateKeywordSearch` は trim なし・page reset ありで、live 統一後も同意味論を保持する
- Deferred design gaps, risk, and follow-up target: 商品追加欄の autocomplete 化（variant B: Enter commit 経路維持 + 入力中候補プレビュー）は backlog 起票のみ。keywordDraft 初期値が normalized.q 由来という現行の軽微 drift は SearchBar 置換（raw 結線）で同時解消
- Test Design Matrix can cite design decision IDs or source doc sections: yes（TRACE-D12 / UI-01a-D9 / §59.2 / SPEC-UICB-*）
- Absolute guarantee / escape hatch self-check completed: yes（例外なし。既存挙動の意味論変更は「即時更新 → debounce 200ms」の統一のみで、これは TRACE-D12 の意図した変更）

## Impact Review Lenses

not applicable — 本 batch は field 調査 / 実機挙動 / 外部 tool / POS 連携 / フォーマット変更を起点としない（repo 内実査と owner 裁定のみが入力）。環境・再現性 lens: 新設の環境依存なし（既存部品の適用と docs 同期のみ、toolchain / CI 変更なし）。

## Design Readiness

- Existing design docs are sufficient because: live 型契約は UI-01a-D9（50）と §59.1-59.2（59）で確立済みで、本 batch はその適用先追加。52 / 65 は記述の実態同期であり新設計なし
- Source docs updated in this PR: 65（TRACE-D12 新設 + §65.5 同期）、59（採用箇所表）、52（§52.3）、73（route file 表記）
- Design gaps intentionally deferred: 商品追加欄 autocomplete 化（backlog）、JAN 列追加 B 案（不採用、要望発生時に再考）
- Durable decisions discovered in this plan and promoted to source docs: TRACE-D12、JAN 裁定 A

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): UI 層内のみ。CMD 以下不変
- Backend function design: 変更なし
- Command / DTO / data contract: 変更なし（bindings diff ゼロ）
- Persistence / transaction / audit impact: なし
- Operator workflow / Japanese UI wording: 検索欄のアクセシブルネーム「商品検索」は不変（外付け label は撤去し SearchBar 既定 aria-label が担う）。placeholder は SearchBar 既定「商品コード・商品名・JANで検索」へ統一（現行「商品コード・JAN・商品名」から文言変化、商品一覧・在庫照会と同一）。挙動は「即時反映」から「200ms debounce + Enter 即時」へ統一される（3 画面同一体験）
- Error, empty, retry, and recovery behavior: 不変（filter-empty reset action は既存 SPEC-UIBB-1/2 のまま）
- Testability and traceability IDs: TRACE-D12 / 既存 REQ-206 token を test に付与

## Contract Probe

N/A — 未検証の外部前提なし。SearchBar は repo 内既存部品（採用実績 2 画面 + 単体 test あり）で、外部 library / OS 挙動への新規依存を持たない。

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| TRACE-D12: 商品検索欄 SearchBar live 型（debounceMs=200） | `InventoryRecordsPage.tsx` | live 結線 test（debounce 発火 / Enter 即時） | L3: 実機で入力→一覧絞り込みの目視 |
| UI-01a-D9 同型: value = raw `search.q`、no-trim | 同上 | no-trim assert | — |
| §59.2: 変換確定 Enter の誤発火なし（keydown isComposing guard）+ 確定後の最終文字列反映 | 同上（SearchBar 経由） | IME test | L3: IME 変換入力の目視（中間反映は許容） |
| 現行意味論維持: q 変更で page 既定 reset | 同上 | page reset test | — |
| TRACE-D11: 一覧⇄詳細の検索条件・page 保持 | 既存実装（不変） | 既存 test 維持 | — |
| SPEC-UIBB-1/2: filter-empty reset action の isFilterDefault 判定に q が含まれ続ける | 既存実装（不変） | 既存 test 維持 | — |
| UI-01a-D9 同型: 外付け label / `id` 撤去 + 既定 aria-label「商品検索」維持 + placeholder 既定値統一 | `InventoryRecordsPage.tsx` | AC10 assert（`findByLabelText` + placeholder + `rg "records-keyword"` 0 hit） | L3: label 撤去後の filter 行視覚整合 |
| 59 §59.1 採用箇所表 = 実採用と一致 | `59-ui-shared-patterns.md` | — | Plan/Final Review の doc 突合 |
| 52 §52.3 表 = 実 route と一致、`/pos/*` 記載 0 | `52-ui-shared-layout.md` / `73-ui-stocktake.md` | AC5 rg sweep（review 手順） | Final Review の routeTree 突合 |
| 65 §65.5 JAN 行 = 5 詳細画面 + 棚卸し列 `StocktakeItemDetail`（全 6 DTO）実態と一致 + 裁定記録 | `65-inventory-record-traceability.md` | — | Final Review の DTO 突合 |

## Test Plan

Test Design Matrix: [test-matrices/2026-08-04-ui-consistency-batch.md](test-matrices/2026-08-04-ui-consistency-batch.md)

- targeted tests: `npm test -- InventoryRecordsPage`（live 結線 / IME / no-trim / page reset / 既存 filter regression）
- negative tests: 変換確定 Enter の非 flush（`isComposing` guard）、compositionend 後の最終文字列反映、空文字クリアでの `q` param 削除 + page reset
- compatibility checks: 既存 URL（`?q=...` 付き）での復元表示、`bindings.ts` diff ゼロ
- data safety checks: 実店舗データ不使用（synthetic fixture のみ）
- main wiring/integration checks: SearchBar → URL `q` → 一覧 query の end-to-end 結線（mock でなく実 router で assert）
- L3 を含むため、Writer 完了条件に `cargo check --release` を含める（backend 変更なしのため形式確認）

## Boundary / Wire Contract

- producer: `SearchBar onSearchChange`（raw 文字列）
- consumer: TanStack Router URL search `q` → normalized（trim）→ 一覧 query の `keyword`
- wire type: URL search param `q: string | undefined`（raw、空文字は undefined へ畳む）
- internal type: `normalized.q`（trim 済み。query 構築・filter 既定判定専用）
- precision/range: 該当なし（文字列）
- round-trip path: 入力 → URL → reload / 詳細から戻る → value 復元（raw）
- invalid input: 空文字 / 空白のみ → `q` param 削除（既存挙動維持）
- compatibility: 既存共有 URL の `q` は raw / normalized どちらの由来でも同一表示に解決される

## Review Focus

- 「即時更新 → debounce 200ms」の意味論変更が TRACE-D12 の意図どおりで、それ以外の意味論（no-trim / page reset / IME / filter-empty 判定）が現行同一であること
- `keywordDraft` + 自前 IME guard の撤去に伴う regression（特に compositionend 経路）
- docs 同期 3 点（52 / 65 / 59・73）が実装・実 route・実 DTO と一字一句一致すること（是正で新 drift を作らない）
- 非目的の遵守（商品追加欄 5 箇所に触れていないこと）
- 外付け label 撤去後の filter 行グリッドの視覚整合（他 filter は label 付きのまま。崩れの有無は L3 目視で確認）
- `90-traceability.md` の再生成 diff が TRACE-D12 起因の delta のみであること（手動編集の混入なし、AUTO-GENERATED 維持）

## Spec Contract

Contract ID: SPEC-UICB

- SPEC-UICB-1: 入出庫履歴の商品検索欄は共有 SearchBar live 型（`debounceMs=200`）で URL `q` に raw 結線される（value = raw `search.q ?? ""`）。
- SPEC-UICB-2: 変換確定 Enter（`isComposing`）は search flush を誤発火しない。変換確定後は最終文字列が `q` へ反映される。変換中の中間文字列の一時反映は live 型既定挙動として許容（商品一覧・在庫照会と同一意味論）。
- SPEC-UICB-3: `q` の変更（クリア含む）は page を既定へ reset する（現行意味論維持）。
- SPEC-UICB-4: `docs/function-design/` 配下に `/pos/` 由来の陳腐化 URL 記載が存在しない（archive 除外）。52 §52.3 の URL / route file 列は実 route と一致する。
- SPEC-UICB-5: 65 §65.5 の JAN 行は 5 詳細画面 + 棚卸し列 `StocktakeItemDetail`（全 6 DTO）の実態（JAN field なし・5 詳細画面で JAN 非表示）と一致し、選択肢 B 不採用の owner 裁定（2026-08-04）が変更履歴に残る。
- SPEC-UICB-6: 検索欄は外付け label / `id` を持たず、SearchBar 既定 aria-label「商品検索」と既定 placeholder「商品コード・商品名・JANで検索」を用いる（商品一覧・在庫照会と同一）。

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-UICB-1 | SearchBar 置換 | live 結線 test（M-A1/A2） | debounce / Enter 意味論 | test 名 + L3 |
| SPEC-UICB-2 | 自前 IME guard 撤去 | IME test（M-A3） | compositionend 経路 | test 名 + L3 |
| SPEC-UICB-3 | updateKeywordSearch 維持 | page reset test（M-A4） | reset 意味論 | test 名 |
| SPEC-UICB-4 | 52 / 73 是正 | rg sweep（M-B1） | 実 route 突合 | review 記録 |
| SPEC-UICB-5 | 65 §65.5 同期 | —（doc） | DTO 突合 | review 記録 |
| SPEC-UICB-6 | 外付け label 撤去 + 既定値統一 | AC10 assert（M-A8） | アクセシブルネーム維持 | test 名 + L3 |

## Data Safety

- 実 POS / 店舗 artifact、DB file、backup、log、receipt image、secret は commit しない。
- local-only paths: `.local/ci-evidence/`（L1 証跡、commit しない）。
- synthetic-only paths: test fixture は synthetic データのみ。

## Implementation Results

入出庫履歴の商品検索欄を共有 `SearchBar` live 型へ統一し、debounce / Enter / IME / raw `q` / page reset / accessibility の regression coverage を更新した。52 / 59 / 65 / 73 の設計記述と自動生成 90 を同期し、bindings / route generation / package manifests は不変。Writer mutation は M-A1〜M-A8 / M-B1〜M-B2 の全対象で red、復元後 clean を確認し、L1 full と release profile check を完了した。Draft PR [#61](https://github.com/higekiri-w/inventory-system-public/pull/61) を作成済み。

Closeout 追記（2026-08-04、squash merge 済み）: Coordinator mutation 独立再実測（記録非参照の独立導出、隔離 worktree）で M-A1〜M-A8 / M-B1〜M-B2 全 red・survivor 0、独立 Sonnet Final Review は Ledger 全行適合で裁定後 P1/P2=0（P2-1 = 遷移 subject 非 canonical は disposition 記録、Review Response 参照）、owner L3（Windows native）3 項目 PASS。exact-HEAD の L1 / hosted / PR HEAD 三点一致は PR body を正とする。実績 = 介入 3/3・relay 2/2・Writer fail-closed 1 件（true positive、gated Amendment 1）。

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

## Review Response

- Findings Freeze: frozen after Independent Review（2026-08-04、P1/P2=0 裁定後）; post-freeze exceptions: none.
- If R3 review-only sub-agent is skipped, record an explicit line beginning with `Review-only skipped because:` and the reason.

**Plan Gate round 1**（2026-08-04、Sonnet Plan Reviewer 独立 context、P1=2 / P2=3 / P3=1）

- P1-1 accept（Coordinator 実読裏取り: 65:94-106 の行 conflate を確認）→ §65.5 は行分割指示へ Scope / AC7 是正（JAN 単独行 no、商品コード / 商品名 / 部門は yes 維持）
- P1-2 accept（実読裏取り: `SearchBar.tsx` の isComposing guard は :89 / :161 の keydown のみ）→ 裁定 = 案 a（契約側を SearchBar live 実態へ是正。AC2 / SPEC-UICB-2 / Ledger / Human Gate / Matrix M-A3 を改稿。案 b の SearchBar 改修は商品一覧・在庫照会への影響と scope 拡大のため不採用）
- P2-1 accept → TRACE-D12 は §65.2 正式エントリ + §65.4.1 備考参照
- P2-2 accept → §65.11 Test Focus bullet を Scope へ追加
- P2-3 accept → Human Gate L3 文言を failure mode 名指しへ具体化
- P3-1 accept → 52 §52.3 UI-07 / UI-08 の before→after を明示
- 是正は plan-gate 滞留中の in-place 修正（plan-approved 前のため Amendments 非該当）。round 2 を fresh 独立 context で再審査する

**Plan Gate round 2**（2026-08-04、fresh Sonnet Plan Reviewer 独立 context、round 1 是正 7 項目 = 全 OK、新規 P1=0 / P2=2 / P3=1）

- P2-A accept（Coordinator 実読裏取り: SearchBar 既定 placeholder = 「商品コード・商品名・JANで検索」`SearchBar.tsx:143`、商品一覧・在庫照会は prop 無指定で既定値依存）→ 裁定 = 既定値採用で 3 画面同一文言へ統一、invariant 側を改稿 + AC10 新設
- P2-B accept（実読裏取り: `LiveSearchBar` は `Omit<..., "id">` で `id` を受け取らず、商品一覧は外付け label なし）→ 外付け label 撤去を Scope へ明記、アクセシブルネームは既定 aria-label「商品検索」が維持
- P3 accept → Human Gate へ到達手順を追記
- round 3 を fresh 独立 context で再審査する

**Plan Gate round 3**（2026-08-04、fresh Sonnet Plan Reviewer 独立 context、round 2 是正 3 項目 = 全 OK、新規 P1=1 / P2=0 / P3=1）

- P1 accept（Coordinator 自packet 突合で confirmed。round 2 是正を AC10 / M-A8 へ反映した際、同契約の Ledger / Spec Contract / Trace Matrix への sweep を漏らした packet-correction-full-sweep の再発型）→ Ledger へ UI-01a-D9 label/placeholder サブ契約行、SPEC-UICB-6 新設、Trace Matrix 行追加、Matrix 側の参照 ID 同期
- P3 accept → AC7 の DTO 突合対象へ `StocktakeItemDetail` を追加列挙（§65.5 全 6 列と検証範囲を一致させる）
- round 4（収束確認 focused round）を fresh 独立 context で実施する

**Plan Gate round 4**（2026-08-04、fresh Sonnet Plan Reviewer 独立 context、round 3 是正 2 項目 = 全 OK、新規 P1=1 / P2=0 / P3=0、未収束判定）

- P1 accept（Coordinator 自packet 突合で confirmed。round 3 の 6 列化を AC7 のみに適用し、SPEC-UICB-5 / Ledger / M-B2 を「5」のまま残した同型 sweep 漏れの 2 度目）→ 「5 詳細画面 / 5 DTO」系表現を `rg` で全 7 出現を機械列挙し、AC7（是正済み）と Non-scope の B 案説明（5 画面が正確）を除く 5 箇所（Goal / Scope / Ledger / SPEC-UICB-5 / M-B2）を「5 詳細画面 + 棚卸し列 StocktakeItemDetail（全 6 DTO）」へ一括改稿
- round 5（収束確認 focused round）を fresh 独立 context で実施する

**Plan Gate round 5**（2026-08-04、fresh Sonnet Plan Reviewer 独立 context）

- round 4 是正 5 箇所 = 全 OK（bindings.ts 実測で全 6 DTO と記述の一致確認）、残存「5」系 hit はすべて意図的記述と判定
- 新規 P1=0 / P2=0 / P3=0 — **収束**。Plan Gate rally は 5 round 単調収束（P1+P2: 5→2→1→1→0）で終了、owner plan 承認待ちへ

**gated Amendment 1**（2026-08-04、Writer fail-closed 起源、停止 HEAD `c819148`）

- 事象: TRACE-D12 の 65 doc 追加により `90-traceability.md`（AUTO-GENERATED）の再生成義務が発生し、`local-ci.sh changed` の traceability gate が drift を検出。Scope 未列挙のため Writer が正しく fail-closed 停止（true positive）
- 裁定: 選択肢 A 採択 = `90-traceability.md` の自動再生成を Scope へ追加（B = TRACE-D12 撤回は Plan Gate 5 round で確定した明示契約の放棄のため不採用）。Registration / Generation Obligations の起票時「該当なし」誤判定も併せて是正
- Review Focus へ「再生成 diff が TRACE-D12 起因 delta のみ」を追加し、Final Review の検分対象とする

**Independent Review**（2026-08-04、content candidate `324e9c7`）

- Coordinator mutation 独立再実測（隔離 worktree、Writer 記録非参照の独立導出）: M-A1〜M-A8 / M-B1〜M-B2 = **red 10/10、survivor 0**。M-A7 は最終 candidate の q 単独 fixture 化により一発 red を独立確認（Writer 自己申告の初回 survivor は補強済みで再現せず）。各 mutant 復元後の clean 確認済み
- Final Review 一次（独立 Sonnet）: Contract Coverage Ledger 10 行全行適合（M-A1/A5/A7 は Reviewer 側でも独立注入で red 確認）、AC1〜AC10 全て実測 green、Scope 境界遵守（SearchBar 本体・5 取引画面・backend diff 0）、90-traceability delta は REQ-206 単一セルのみ、state commit の hunk は allowlist 内。**P1=0 / P2=1 / P3=0**
- P2-1 accept（Coordinator 実見で confirmed）: local-verified 遷移 commit `6e04284` の subject が canonical `docs(plans): state-only遷移 <from>-><to>` 形式でなく、STATECAP 計数から不可視。**disposition = 履歴書き換えなし**（rename の force-push は Draft PR HEAD 変更と L1 再取得の手戻り、canonical 化は cap 4 本目化のリスク。実害〈cap 超過〉なし）。以降の遷移 commit は canonical subject で作成し、本遷移（local-verified → independent-review → human-confirm）から適用。Writer 向けの再発防止は Post-Merge Closeout で発注書 template への追記を判断
- 上記により findings 裁定後 P1/P2=0。本 commit で `local-verified → independent-review → human-confirm` を materialize、`Reviewed Content HEAD = 324e9c7` を設定（Freeze は節冒頭の Findings Freeze 行を更新）

**Human Gate**（2026-08-04）

- owner L3（Windows native）: live 検索 / IME 変換確定挙動 / label 撤去後の視覚整合の 3 項目 **PASS**、Ready 承認。介入実績 3/3（plan 承認 / L3 / Ready）・relay 2/2。`human-confirm → ready-hosted-final` を本 state-only commit で materialize し、以降 Draft のまま exact-HEAD L1 full → PR body 更新 → Ready の順で処理する

**Writer local verification**（2026-08-04）

- clean content candidate の L1 full、release profile check、target plan check、生成 drift 0 を確認し、candidate SHA と L1 evidence 所在を Draft PR #61 body に記録した。
- M-A1〜M-A8 / M-B1〜M-B2 を commit 後の clean tree から個別注入し、各対応 test / review CLI の red と復元後 clean を確認した。M-A7 の初回 survivor は複合 fixture が q 判定欠落を隠していたため、q 単独 fixture へ狭めて感度を補強した。
- 必要 evidence が揃ったため `implementing → local-verified` を本 validation / dashboard content commit に同乗させる。独立 Final Review、Windows native L3、Ready は未解決 Human Gate のまま維持する。
