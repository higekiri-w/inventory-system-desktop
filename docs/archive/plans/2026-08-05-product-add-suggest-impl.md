# Plan Packet: 商品追加欄 live 候補プレビュー（ProductAddSuggest）実装

design 正本 = `docs/design-system/02-component-catalog.md` ⑮（SPEC-SUGGEST-D1〜D11、凍結。D11 は gated Amendment 3 で追加）+ 画面別 5 D-ID。design-first の経緯・裁定は [archived design packet](2026-08-04-product-add-suggest-design.md)（PR #64）を参照。

## Workflow State

- Phase: archive
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: ef10439
- Amendments: 284f664, 00719e1, 0d1ea42
- Coordinator: Fable 5（main thread）
- Writer: Codex（発注書駆動、public-writer clone。発注書は Plan Gate closure 後に提示）
- Plan Reviewer: Sonnet 5 独立 subagent（rally、3 round 天井。D-062: Writer = Codex と別 vendor 要件を充足）
- Final Reviewer: Sonnet 5 独立 subagent（fresh context、Plan Reviewer とは別 context）
- Reviewed Content HEAD: 9d31540
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required（R3 frontend change。Ready event で hosted CI が auto run される通常経路）
- Human Gate: 解消済み（2026-08-09）— 初回 L3 で IME 全角数字 blocker 検出 → gated Amendment 3 是正 → 再 L3 で残存した間欠文字欠落をスキャナ側確定設定で解消し、最終 L3 全項目 PASS + owner Ready 承認（介入 5/5）。L3 詳細・スキャナ確定設定の正本 = PR body Human Gate 節（設定は機器側にのみ残るため PR body に記録）

## Owner Effort Budget

- 介入回数上限: 5（当初 3〈plan 承認 / L3 実施 / Ready 承認〉。gated Amendment 3 で 3→5 へ調整 — recorded reason: 初回 L3 で runtime blocker〈日本語 IME による全角数字化 + Enter の変換確定消費〉を検出し、Amendment 承認 + 再 L3 の decision point 追加が不可避のため。D-038 の recorded-reason 調整）
- 実働時間上限: 45分（当初 30 分 + 再 L3 1 回分）
- relay 往復上限: 3（当初 2 = Codex 発注 1 + 予備 1 は消費済み。Amendment 3 是正実装の Codex 発注 1 を追加）

承認依頼フォーマット: `この change での介入 N 回目 / 予算 5 回` + 利用者可視の完了 1 文。

STATECAP 予算 3 本設計（state-only 遷移 commit）: ① `plan-gate -> plan-approved -> implementing`（発注直前に一括実体化）② `independent-review -> human-confirm` ③ `human-confirm -> ready-hosted-final`。その他の遷移は content commit 同乗。各 forward materialize 直後に `bash scripts/check-workflow-git.sh` を実行する。（gated Amendment 3 追記）backtrack commit は cap 対象外。Amendment 3 後の再走行 forward 遷移（implementing -> local-verified -> independent-review -> human-confirm）は content commit 同乗で実体化し、state-only 残枠 1 本（Amendment 3 時点で forward 2/3・post-implementation 1/2）は ③ に温存する。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

（Codex への実装発注は通常の発注書 relay 方式。§5.5 order branch 分離は使わない）

## Risk

Risk: R3

Reason:
operator の主要入力動線（スキャン一挙動追加フロー）に隣接する 5 画面横断の UI 実装。挙動退行が起きると全取引記録の入力速度に直結する。設計契約（catalog ⑮）は凍結済みで、本 PR はその実装凍結義務の履行。

## Goal

Goal Invariant:

### 最小完了条件

- 取引 4 画面（入庫 61 / 手動販売 62 / 返品・交換 63 / 廃棄・破損 64）+ 棚卸し（73）の商品追加欄で、入力中に live 候補プレビュー（catalog ⑮ SPEC-SUGGEST-D1〜D11 準拠）が表示され、既存 Enter commit（スキャン一挙動追加）経路が不変のまま動作する。日本語 IME 有効状態のスキャナ入力（全角数字化 + Enter の変換確定消費）でも一スキャン一追加が成立する（D11 = gated Amendment 3）。

### 失敗定義

- 既存 5 画面 test（`ReceivingPage.test.tsx` / `ManualSalePage.test.tsx` / `ReturnExchangePage.test.tsx` / `DisposalPage.test.tsx` / `StocktakePage.test.tsx`、T17/T23 含む）のいずれかが変更される、または red になる。（限定例外 = gated Amendment 1、owner 承認 2026-08-05: `StocktakePage.test.tsx` T3〈L227〉の `getByRole("combobox")` 単数 query は D6 の商品入力 combobox 追加で複数要素エラーになるため、名前付き query への特定化のみ許容する。T3 の assert 内容・他の全 test は不変のまま）
- SPEC-SUGGEST-D1〜D11 のいずれかに反する実装（自動 active 化、旧リスト click 許可、focus 奪取、新規 npm 依存等）。

### 非目的

- 既存 5 画面の commit 経路コード（`handleProductSearch` / `resolveItem` / `addProduct` / `selectCandidate` / focus 復帰 / IME guard）のリファクタ・共通化・移動（D9）。（限定例外 = gated Amendment 3、owner 承認 2026-08-09: 各画面の検索/解決関数〈`handleProductSearch` / `resolveItem`〉へ optional 明示引数 1 個の追加のみ許容する。省略時挙動は従来の state 参照と完全同一、本体ロジック・既存呼出し経路・既存 test file は不変。リファクタ・共通化・移動は引き続き禁止 = D11 (e)）
- SearchBar 系 live 型 3 画面（商品一覧 / 在庫照会 / 入出庫履歴）の変更。
- Enter commit 後の既存「複数件候補テーブル」の変更・キーボード操作追加。
- `search_products` backend / bindings / DB の変更。
- 新規 npm 依存の追加（D9 で不採用契約済み）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。

## Scope

- `src/components/patterns/ProductAddSuggest.tsx` 新設（component。named function export、patterns 配下の既存慣行に従う）
- `src/components/patterns/useProductAddSuggest.ts` 新設（hook。debounce 200ms + sequence token + isLocked()/invalidateAndClose() API。component と同 file に置くかは Writer 裁量、配置は patterns 配下固定）
- `src/components/patterns/ProductAddSuggest.test.tsx` 新設（S 系 = D1〜D11 契約 test）
- 5 画面への配線: `ReceivingPage.tsx` / `ManualSalePage.tsx` / `ReturnExchangePage.tsx` / `DisposalPage.tsx` / `StocktakePage.tsx`（既存 commit 経路コードは不変、suggest 層の追加配線のみ）
- 配線方式（5 画面共通、rally round 1 P2-5 裁定）: 既存 input 要素と既存 onKeyDown / onChange handler 関数の本体は変更せず、ProductAddSuggest が input を包んで event を合成する — onKeyDown は suggest 層が先に判定し（isComposing は suggest 処理をせず既存 handler へ即委譲 = D5、active あり Enter は候補確定 = D2、↓/↑/Esc は suggest 操作 = D6）、それ以外（active なし Enter 含む）は既存 handler をそのまま呼ぶ。onChange は既存 state 更新を呼んだうえで suggest 層へ入力値を通知する（D2 の同期解除 + close はこの通知内で行う）。既存 handler 関数本体への変更は D1 違反として fail-closed 停止の対象
- `invalidateAndClose()` の呼出しスコープ（rally round 1 P2-3 裁定）: 5 画面すべての保存 event handler 内で同期呼出しに統一する。disposal は UI-05-D15 lock ref 更新と同一 event 内（catalog ⑮ D10 特記どおり）、他 4 画面も reactive 伝播（isLocked() の render 反映）に依存せず明示呼出しとし、D4「lock 成立」時点の同期 close を render サイクル非依存に満たす。これは catalog ⑮ D10 の凍結文と矛盾しない厳格側への一意化であり、契約変更ではない
- 配線 test は新規 file に隔離: 各画面 `*.suggest.test.tsx`（W 系。既存 test file への追記は凍結義務と衝突するため禁止）
- 新規 test file への UI-NN / D-ID token 付与 + `cd src-tauri && cargo run --bin generate_traceability -- --check` の差分なし確認（差分が出る場合は再生成を同 PR に同梱）
- `docs/design-system/02-component-catalog.md` ⑮ canonical 注記の実在同期（「実装 PR で新設予定。file 未作成の現時点では本節が設計正本」→ 実 file path へ。契約 D1〜D10 本文は不変）
- （gated Amendment 3）全角数字正規化 util 新設（patterns 配下、named export。値全体が `[0-9０-９]+` に一致する場合のみ U+FF10〜U+FF19 → ASCII 数字の文字写像を行い、それ以外は入力をそのまま返す。NFKC・trim 変更・記号/かな変換は行わない）
- （gated Amendment 3）ProductAddSuggest に `onCompositionEnd` 合成 handler + `onComposedDigitsCommit?: (normalized: string) => void` prop を新設（契約 = catalog ⑮ SPEC-SUGGEST-D11。isLocked() true 中は不発火、一度だけ commit の one-shot guard 込み）
- （gated Amendment 3）5 画面の配線: `onComposedDigitsCommit` を各画面の既存検索/解決関数へ接続し、正規化済み文字列を optional 明示引数で渡す（state 経由の暗黙参照は stale closure になるため禁止 = D11 (e)）
- （gated Amendment 3）Matrix へ S22〜S27 / W13〜W17 / X22〜X26 を追加（本 Amendment commit で Coordinator が追記済み。Writer は Matrix を正として実装する）
- （gated Amendment 3）function-design 5 doc（61/62/63/64/73）の living 参照範囲を SPEC-SUGGEST-D1〜D11 へ同期（Decision History の日付行は歴史記録として不変。Non-scope の function-design 本文変更禁止に対する Amendment 判断）
- `docs/Plans.md`: backlog 行へ着手注記 + 「次の行動」active packet link（PK4）
- 本 packet + Test Design Matrix

## Non-scope

- 既存 5 画面 test file の一切の変更（凍結義務）
- `src-tauri/` 配下・bindings・DB・wire の一切
- 61/62/63/64/73 の function-design doc 本文変更（設計契約は PR #64 で確定済み。実装 status 注記が必要になった場合のみ Amendment で判断）
- 複数件候補テーブル・SearchBar 系の変更
- daily-report 系画面への展開

## Acceptance Criteria

- AC1: `ProductAddSuggest` component + `useProductAddSuggest` hook が `src/components/patterns/` に存在し、5 画面すべてに配線されている（`rg -l "ProductAddSuggest" src/features` が 5 画面の Page file を返す）
- AC2: 新規 S 系 / W 系 test が Matrix の全行を cover し `npm test` green
- AC3: 既存 5 画面 test file のうち `StocktakePage.test.tsx` を除く 4 file は diff 0（`git diff main --name-only` に現れない）。`StocktakePage.test.tsx` の diff は T3 の query 特定化のみ（`git diff main -- src/features/stocktake/StocktakePage.test.tsx` で確認し、変更が単数 combobox query の名前付き化に限られ assert 内容不変であることを PR body に記録 = gated Amendment 1）。全既存 test green
- AC4: `bash scripts/local-ci.sh full` PASS / CLEAN（Writer 完了条件に含む）
- AC5: Matrix X1〜X21 の各 mutation 注入で `npm test` が red になることを、Writer 自己実測 + Coordinator 独立再実測（clean tree、commit 後）の双方で確認する（X21 は gated Amendment 2 起源のため Coordinator 実測のみで可）
- AC6: catalog ⑮ canonical 注記が実 file path を指す（`rg -c "file 未作成" docs/design-system/02-component-catalog.md` = 0）
- AC7: Plans.md「次の行動」が本 packet へ link し PK4 を充足する
- AC8: 新規 test file が UI-NN / D-ID token を含み `cd src-tauri && cargo run --bin generate_traceability -- --check` 差分なし
- AC9: `package.json` / `package-lock.json` の diff なし（npm 依存追加ゼロ = D9）
- AC10: `cargo check --release` PASS（L3 native build 前提の Writer 完了条件、CI gate ではない）
- AC11: L3 再実施（スキャナ supported sequence 互換性確認 + UX 確認 + gated Amendment 3 の IME ON/OFF 実測項目表）の結果が PR body Human Gate に記録され `gh pr view --json body` で確認可能
- AC12（gated Amendment 3）: D11 契約 test（S22〜S27 / W13〜W17）green + mutation X22〜X26 kill（AC5 と同じ Writer 実測 + Coordinator clean tree 独立再実測の双方）

## Design Sources

- Requirements / spec: REQ-201〜204（取引 4 画面）、棚卸し（73）
- Architecture: `docs/ARCHITECTURE.md`（UI 層のみ。CMD/BIZ/IO 不変）
- Function / command / DTO: `docs/function-design/20-io-product-repo.md`（search_products 契約、流用・不変）
- Screen / UI: `61-ui-receiving.md` UI-02-D14 / `62-ui-manual-sale.md` UI-04-D16 / `63-ui-return-exchange.md` UI-03-D21 / `64-ui-disposal.md` UI-05-D16（+ UI-05-D15 lock ref）/ `73-ui-stocktake.md` UI-10-D12（+ UI-10-D2 / D11）
- Design system: `docs/design-system/02-component-catalog.md` ⑮（SPEC-SUGGEST-D1〜D11 = 凍結正本）、`docs/UI_TECH_STACK.md` §5.3 / §5.4
- Decision log / ADR: D-030（npm 依存不採用の背景）、D-031（pagination clamp、参照のみ）

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status |
|---|---|---|
| Backend function / command / repository / validation / error | 変更なし（search_products 流用） | existing sufficient |
| Command / DTO / generated binding / wire shape | 変更なし | existing sufficient |
| DB / transaction / audit / rollback / migration | 変更なし | existing sufficient |
| Screen / UI / route state / Japanese wording | catalog ⑮ + 画面別 5 D-ID（PR #64 で正本化済み） | existing sufficient |
| CSV / TSV / report / import / export format | 変更なし | existing sufficient |
| Durable decision / ADR | catalog ⑮ が durable 正本。本 PR は canonical 注記の実在同期のみ | updated in this PR |

## Registration / Generation Obligations

- Tauri command / route / operator 画面 / function-design doc の新設: 該当なし
- REQ coverage: 該当あり — 新規 test file（S 系 / W 系）に UI-NN / D-ID token を付与する。traceability T4 検査は「REQ/UI token なし FE test file 数」の両方向 baseline 検査のため、token なしの新規 test file は ERROR になる。`cd src-tauri && cargo run --bin generate_traceability -- --check` で差分なしを確認し、差分が出る場合は再生成を同 PR に同梱する（PR #61 gated Amendment 1 の教訓を Scope に前積み）
- component 新設の catalog 登録: 登録済み（⑮ が既存）。canonical 注記の実在同期のみ Scope に含む

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| REQ-201 | 61 §61.5 | UI-02-D14 | variant B（設計裁定済み、PR #64） | ReceivingPage 配線 | W1 / W8 |
| REQ-203 | 62 §62.5 | UI-04-D16 | 同上 | ManualSalePage 配線 | W2 / W8 |
| REQ-202 | 63 §63.5 | UI-03-D21 | 同上（addProduct の direction 引数を既存どおり維持） | ReturnExchangePage 配線 | W3 / W8 |
| REQ-204 | 64 §64.5 | UI-05-D16 | 同上 + UI-05-D15 lock ref 整合 | DisposalPage 配線 | W4 / W6 |
| 棚卸し | 73 §73.5 | UI-10-D12 | 確定は UI-10-D2 既存経路（findMutation → selectItem）、D11 focus 契約同一発火 | StocktakePage 配線 | W5 / W7 |
| 横断 | catalog ⑮ | SPEC-SUGGEST-D1〜D11 | 凍結正本の履行（D11 = gated Amendment 3） | ProductAddSuggest + hook + 正規化 util | S1〜S27 / X1〜X26 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history: catalog ⑮ に契約・採用背景・除外 manifest が正本化済み（PR #64）
- Plan-only durable decisions found and promoted: なし（本 packet は実装 scope のみ。durable 判断は全て catalog ⑮ 側に既存）
- Assumptions and constraints: 実装実態の実査（2026-08-05 Explore + Coordinator rg 裏取り）= 5 画面とも `commands.searchProducts` 直接 await（TanStack Query 非経由）/ lock 実名 = `isFormLocked`（receiving L179 / manual-sale L199 / return-exchange L250、派生変数）・`isFormLockedRef`（disposal L127、UI-05-D15 ref）・`isCompleting`（stocktake L131）/ T17 = `StocktakePage.test.tsx` L255（UI-10-D11 focus 遷移）・T23 = 同 L649（IME）。`rg -n` 出力で確認済み
- Deferred design gaps: stocktake のカウント入力欄 `disabled` prop と suggest 層 lock source（isCompleting）の関係は D4/D10 契約どおり isCompleting を単一 source とする。`disabled` prop は既存 commit 経路の所掌で不変。実装時に疑義が出れば Writer は fail-closed 停止
- Test Design Matrix can cite design decision IDs: S/W/X 系が各 D-ID / SPEC-SUGGEST-Dn を引用
- Absolute guarantee / escape hatch self-check: 「既存 commit 経路不変」は既存 5 test file の diff 0（AC3）+ W8 smoke で担保。配線 test を新規 file に隔離することで凍結と test 追加の衝突を構造的に回避

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | UI 層内で完結。CMD/BIZ/IO 不変 | not applicable |
| Fact check / design decision split | 実装実態（handler 名 / lock 実名 / test 番号）は 2026-08-05 実査 + rg 裏取り済み（Design Intent Audit 参照） | 本 packet |
| Lifecycle / retry | suggest fetch の open/close/active/stale 破棄 lifecycle を State Lifecycle Matrix で cover | Matrix |
| Operator workflow | スキャン一挙動フロー不変が Goal Invariant。候補プレビューは目視補助のみ | L3 |
| Replacement path | 追加のみ。既存候補テーブル残置 | Non-scope |
| Data safety / evidence | 実データ不使用（synthetic mock のみ） | Data Safety |
| Reporting / accounting semantics | not applicable（表示層のみ） | — |
| Manual verification | スキャナ実測 = supported sequence〈バーコード文字列 + Enter〉適合の互換性確認 + UX 確認（debounce 下の候補非発火）。安全性は D2/D4 software contract 側で担保（PR #64 裁定） | Ledger L3 行 |
| 環境・再現性 | 新規環境依存なし（npm 依存追加ゼロ = D9）。L3 は既存 Windows native build 手順のみ | AC9 / AC10 |

## Design Readiness

- Existing design docs are sufficient because: 実装契約は catalog ⑮ SPEC-SUGGEST-D1〜D11 + 画面別 5 D-ID で完結しており、未解決の設計問題はない（D1〜D10 は PR #64 で Plan Gate 7 round + Final Review 2 round を経て凍結、D11 は初回 L3 実機所見起源の gated Amendment 3 で追加）
- Source docs updated in this PR: catalog ⑮ canonical 注記の実在同期のみ（契約本文不変）
- Design gaps intentionally deferred: `find_stocktake_item` None 時の無言 no-op は D8 で既知 pre-existing gap として継承
- Layer ownership: UI のみ
- Backend function design: 不変
- Command / DTO / data contract: 不変（`ProductSearchQuery` 流用、per_page 5 は UI 側定数）
- Persistence / transaction / audit impact: なし
- Operator workflow / Japanese UI wording: 候補行 = 商品コード + 商品名 + 部門名、footer「ほか N 件（候補未選択で Enter: 従来の検索）」（catalog ⑮ 確定済み文言）
- Error, empty, retry, and recovery behavior: fetch 失敗 silent close、0 件非表示（D1/D3）
- Testability and traceability IDs: SPEC-SUGGEST-Dn + 画面別 D-ID + REQ-201〜204

## Contract Probe

- N/A: 未検証外部前提なし。searchProducts は既存流用（wire 不変）、npm 依存追加ゼロ、素の絶対配置要素 + 既存 React のみ（design packet の probe 裁定を継承。スキャナ物理挙動は L3 の互換性確認であり Plan Gate 前 probe の対象外 — 安全性は D2/D4 の timing 非依存 software contract で成立）。

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| SPEC-SUGGEST-D1（二層不干渉・silent 縮退） | ProductAddSuggest + 5 画面配線 | S17 / W8 / X1・X20 | 視認（L3 UX） |
| SPEC-SUGGEST-D2（Enter 分岐・active 生成/解除・onChange 同期解除 + close） | useProductAddSuggest | S4〜S8 / X1・X2・X15 | — |
| SPEC-SUGGEST-D3（debounce 200ms・1 文字・per_page 5・footer・0 件非表示） | useProductAddSuggest | S1〜S3 / X7・X16・X18 | L3 UX（debounce 下の候補非発火） |
| SPEC-SUGGEST-D4（sequence token・close 時 cancel・in-flight 不採用・lock source 3 分類・外部 clear 経路） | useProductAddSuggest + 各画面配線 | S9〜S11・S19・S21 / X3〜X5・X21 | — |
| SPEC-SUGGEST-D5（IME: suggest キー処理全域 guard、onChange 側 guard なし） | ProductAddSuggest onKeyDown / onChange | S12・S20 / X6・X14 | — |
| SPEC-SUGGEST-D6（a11y 構造・focus input 保持・click/hover/footer 契約） | ProductAddSuggest | S13〜S15・S3 / X7・X8・X11・X12・X19 | — |
| SPEC-SUGGEST-D7（候補行 3 項目・確定は既存同一 handler） | ProductAddSuggest + 5 画面配線 | S16 / W1〜W5 / X10・X17 | — |
| SPEC-SUGGEST-D8（棚卸し: searchProducts fetch + find_stocktake_item 確定 + D11 focus 同一発火） | StocktakePage 配線 | W5 / W7 | — |
| SPEC-SUGGEST-D9（patterns 配下 1 箇所実装・npm 依存ゼロ・commit 経路コード不変） | 実装形態全体 | AC1 / AC9 / AC3 | — |
| SPEC-SUGGEST-D10（isLocked() / invalidateAndClose() API・lock 単一 source・5 画面保存 event 同期呼出し） | useProductAddSuggest API + 5 画面配線 | S18・S19 / W6・W9〜W12（5 画面各 1、stocktake は確定 event）/ X9・X13 | — |
| SPEC-SUGGEST-D11（IME 全角数字正規化 + compositionend commit、gated Amendment 3） | 正規化 util + ProductAddSuggest onCompositionEnd + onComposedDigitsCommit 5 画面配線 | S22〜S27 / W13〜W17 / X22〜X26 | L3（IME ON 実機イベント順 + 一スキャン一追加） |
| UI-02-D14 / UI-04-D16 / UI-03-D21 / UI-05-D16 / UI-10-D12（画面別適用） | 各 Page 配線 | W1〜W5 | 視認 |
| UI-05-D15（disposal lock ref 整合、同 event 内 invalidateAndClose） | DisposalPage 配線 | W6 | — |
| UI-10-D2（候補確定は find_stocktake_item 経由、searchProducts 結果を直接 selectItem しない） | StocktakePage 配線 | W5 | — |
| UI-10-D11（focus 遷移契約の候補確定経由同一発火） | StocktakePage 配線 | W7 | — |
| 既存 commit 経路 test 不変（T17 = `StocktakePage.test.tsx` L255 / T23 = 同 L649 含む既存 5 test file。T3 query 特定化のみ gated Amendment 1 例外） | 凍結義務 | AC3（4 file diff 0 + T3 限定 diff 検分 + green） | 凍結義務 |
| スキャナ実測（supported sequence〈バーコード文字列 + Enter〉適合の互換性確認 + UX 確認 + gated Amendment 3 の IME ON/OFF 項目表〈IME ON 1 スキャン 1 追加・連続 2 スキャン独立 commit・IME ON 日本語手入力の composition 非破壊・IME OFF 退行なし・WebView2 実イベント順記録・JAN 専用欄無変更確認〉） | — | — | L3 行（PR body Human Gate） |
| catalog ⑮ canonical 注記の実在同期 | docs | AC6 | — |

## Test Plan

Test Design Matrix: `docs/archive/plans/test-matrices/2026-08-05-product-add-suggest-impl.md`

- targeted tests: S1〜S21（component/hook 単体、synthetic mock）、W1〜W12（画面配線、新規 file 隔離）
- Double Audit（rally round 2 P3-3 裁定で opt-in）: 本 change は operator-visible state lifecycle（suggest リスト / active / lock 連動）に触れる R3 のため、DEV_WORKFLOW Contract Audit の recommended second pass に opt-in する。Final Review = 1 pass（Sonnet fresh context、Ledger 突合 + mutation 一次）+ 2 pass（別の Sonnet fresh context による Contract Audit lane 独立再走）。Findings Freeze は両 pass 完了後に発効する
- negative tests: X1〜X21 mutation 注入（Writer 実測 + Coordinator clean tree 独立再実測。X21 は Amendment 2 起源で Coordinator 実測のみ）
- gated Amendment 3 追加分: S22〜S27（D11 契約単体）+ W13〜W17（5 画面 onComposedDigitsCommit 配線、正規化済み引数値で検索実行を assert）+ X22〜X26。実 IME のイベント順 race は RTL harness で再現不能のため、二重発火 guard の実機挙動検証は L3 所掌と Matrix に明記する
- compatibility checks: 既存 5 test file diff 0 + 全 green（AC3）、既存 D-ID 文言不変
- data safety checks: synthetic 商品データのみ、実店舗データ不使用
- main wiring/integration checks: AC1（5 画面配線 rg）、AC7（PK4）
- Writer 完了条件: `bash scripts/local-ci.sh full` PASS/CLEAN（AC4）+ `cargo check --release`（AC10、L3 native build 前提）

## Boundary / Wire Contract

- producer: `commands.searchProducts`（既存、変更なし）
- consumer: `useProductAddSuggest`（新設）
- wire type: `ProductSearchQuery` / `PaginatedResult<ProductWithRelations>`（不変）
- internal type: 候補行は product_code / name / department_name の 3 field のみ参照（D7）
- precision/range: per_page 5（UI 定数、D-031 clamp 200 の範囲内）
- round-trip path: なし（表示のみ）
- invalid input: 空文字は fetch 不発火（D3）、composing 中の候補更新は許容（D5）
- compatibility: 既存 5 画面の searchProducts 呼び出し形（commit 経路）は不変

## Review Focus

- D2 の実装が「自動 active 化ゼロ」「onChange 同期解除 + close」を本当に満たすか（表示直後 / 差し替え直後 / スキャナ高速入力の 3 局面）
- D4 sequence token の二重一致（token + 検索語）と close 系 event 全列挙（Enter commit / clear / 候補確定 / lock 成立 / Esc / blur / Tab / unmount）の網羅
- 5 画面配線の lock source が契約の 3 分類（isFormLocked 派生 ×3 / isFormLockedRef / isCompleting）と一致するか
- W 系 test が既存 handler（addProduct / findMutation→selectItem）を実際に通ることを assert しているか（mock 貫通の tautology でないか）
- 既存 5 test file の diff 0 が保たれているか（凍結義務）
- S 系 oracle が production 定数（debounce 値等）を import せず独立転記になっているか

## Spec Contract

Contract ID: SPEC-SUGGEST（正本 = catalog ⑮、本 packet は転記しない — 凍結済み契約の履行 PR であり、契約本文の重複保持は drift 源になるため参照のみとする）

- SPEC-SUGGEST-D1〜D11: `docs/design-system/02-component-catalog.md` ⑮「契約（SPEC-SUGGEST-D1〜D11、凍結正本…）」節を唯一の正とする
- 実装が契約と矛盾する実態を発見した場合、Writer は契約を再解釈せず fail-closed 停止し Coordinator へ報告する

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-SUGGEST-D1 | component + 配線 | S17 / W8 / X1・X20 | 不干渉・silent 縮退 | npm test |
| SPEC-SUGGEST-D2 | hook keyboard/active | S4〜S8 / X1・X2・X15 | active lifecycle + Enter 分岐反転感度 | npm test |
| SPEC-SUGGEST-D3 | hook fetch | S1〜S3 / X7・X16・X18 | 発火条件・footer・threshold 感度 | npm test |
| SPEC-SUGGEST-D4 | hook sequence/cancel | S9〜S11・S19・S21 / X3〜X5・X21 | 破棄条件網羅 + 外部 clear 経路 | npm test |
| SPEC-SUGGEST-D5 | component IME | S12・S20 / X6・X14 | isComposing 全域 guard + onChange 非 guard | npm test |
| SPEC-SUGGEST-D6 | component a11y/pointer | S13〜S15・S3 / X7・X8・X11・X12・X19 | 構造・click/hover・footer | npm test |
| SPEC-SUGGEST-D7 | 確定 semantics | S16 / W1〜W5 / X10・X17 | 同一 handler 委譲・field 省略感度 | npm test |
| SPEC-SUGGEST-D8 | StocktakePage 配線 | W5 / W7 | find_stocktake_item 経由 | npm test |
| SPEC-SUGGEST-D9 | 実装形態 | AC1 / AC9 / AC3 | 依存ゼロ・コード不変 | git diff / rg |
| SPEC-SUGGEST-D10 | lock API | S18・S19 / W6・W9〜W12 / X9・X13 | lock 単一 source + 5 画面保存 event 同期呼出し | npm test |
| SPEC-SUGGEST-D11 | 正規化 util + compositionend 配線 | S22〜S27 / W13〜W17 / X22〜X26 | 一度だけ commit・stale closure 禁止・過剰正規化なし | npm test + L3 |
| 画面別 D-ID ×5 | 各 Page 配線 | W1〜W5 | 5 画面横並び | npm test + L3 視認 |
| 凍結義務 | — | AC3 | diff 0 | git diff --name-only |

## Data Safety

- 実店舗データ・実商品名・実 JAN は test / 例示に使わない（synthetic のみ）
- local-only paths: なし
- committed 対象: src/ + docs/ のみ

## Implementation Results

- 共通 `ProductAddSuggest` / `useProductAddSuggest` を新設し、取引 4 画面 + 棚卸しへ既存 handler / lock source を保ったまま配線した。
- S1〜S20 / W1〜W12 と mutation X1〜X20 を実走し、初回 survivor で判明した検索語・composing change・silent error の test oracle を補強した。補助 review-only で検出した保存 event 同期呼出しの tautology も、公開 controller API の呼出し・command 前順序を弁別する oracle へ是正した。
- gated Amendment 1 は `StocktakePage.test.tsx` T3 の部門 combobox query 名前付き化 1 行だけを適用し、assert 内容と他の既存画面 test を維持した。
- Draft PR: [#65](https://github.com/higekiri-w/inventory-system-public/pull/65)

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

## Review Response

- Plan Gate rally round 1（Sonnet 5 独立 subagent、2026-08-05）: P1×1 / P2×3 / P3×1、全 accept — (P1) `invalidateAndClose()` を弁別検証する mutation 欠落 → X13 新設 + Ledger/Trace D10 行更新、(P2) D5「onChange 側 guard なし」の独立 assert 欠落 → S20 + X14 新設、(P2) `invalidateAndClose()` 呼出しスコープの解釈余地 → 5 画面保存 event 同期呼出しへ一意化（Scope 節に裁定記録、契約変更なし）+ W9 新設、(P2) suggest 層と既存 input の合成方式未定義 → Scope 節へ handler 合成パターンを明文化、(P3) UI-10-D2 の Ledger 独立行欠落 → 追加。是正 commit `a659bb8`。
- Plan Gate rally round 2（Sonnet 5 独立 subagent、fresh context、2026-08-05）: P1×1 / P3×3、全 accept — (P1) D10「5 画面統一」裁定の test 網羅が 2/5 画面で D7（W1〜W5）と非対称 → W10（ManualSale）/ W11（ReturnExchange）/ W12（Stocktake 確定 event = completeMutation、update_count 単位でない旨明記）新設 + W9 の失敗条件を Receiving 単体へ修正 + X13 red 対象を W9〜W12 へ拡張 + Ledger/Trace D10 行更新、(P3) traceability コマンドの `cd src-tauri &&` 欠落 → Scope/AC8 是正、(P3) Contract Audit recommended second pass の判断未記載 → Double Audit opt-in を Test Plan に明記、(P3) department_name 欠落 case の test id 不在 → S16 に null variant case（oracle は既存候補テーブル実表記の独立転記）を義務化。是正 commit は本 round 是正後の content commit を参照。

- Plan Gate rally round 3（Sonnet 5 独立 subagent、fresh context、2026-08-05）: P1×1 / P3×1、全 accept — (P1) Matrix 自身の Mutation-style Adequacy Questions が要求する threshold 変更 / key branch 逆転 / 出力 field 省略の 3 カテゴリが X 表に未操作化 → X15（Enter 分岐反転 → S6）/ X16（debounce 500ms 化 → S1）/ X17（department_name 省略 → S16）新設、(P3) S2/S13/S17 の同種 gap → 記録残しではなく X18〜X20 として追加（Codex 実測予算は制約なしのため全量注入を採用）。AC5 を X1〜X20 へ拡張、Ledger / Trace Matrix の該当行同期。是正 commit は本 round 是正後の content commit を参照。round 4 は新規探索なしの closure 確認限定として実施し、P1/P2=0 verdict を plan-approved の evidence とする。
- Plan Gate rally round 4 = closure 確認限定（Sonnet 5 独立 subagent、fresh context、2026-08-05）: round 3 是正の完全反映（X15〜X20 = Matrix「必須 mutation 注入」表 X15〜X20 行に実在、AC5 / Ledger / Trace 同期済み、旧範囲表記の残存は `rg -n "X1〜X12|X1〜X14" docs/plans/` hit なし = exit 1 を Coordinator・reviewer 双方が独立実測）と、新設 X15〜X20 の弁別性（各注入と対象 S test assertion の一対一対応）を実読確認。verdict「P1/P2=0、Plan Gate closure 可」。参考記録の既存軽微不整合（Trace Matrix D6 行の S3/X7 欠落、round 3 起因でない）は closure 直後の同 commit で是正済み。
- 遷移記録（recording compression）: 本 state-only commit は `plan-draft -> plan-gate -> plan-approved -> implementing` の隣接 forward 遷移を一括実体化する。中間遷移の evidence = plan-gate: packet + Test Design Matrix が plan-first commit `ef10439` で committed 済み / plan-approved: 独立 Plan Reviewer（Sonnet 5、Writer = Codex と別 vendor = D-062 充足）rally round 4 verdict「P1/P2=0、Plan Gate closure 可」（上記 rally 記録）+ owner plan 承認 2026-08-05（介入 1/3、承認記録は会話、PR body へ転記予定）/ implementing: plan-first commit `ef10439` が全実装 commit に先行することは本時点で実装 commit ゼロにより自明（PK5 ancestry は pre-merge gate で機械検査）。
- gated Amendment 1（2026-08-05、owner 承認 = 介入 2/3）: Codex Writer の fail-closed 停止（true positive）— SPEC-SUGGEST-D6 の商品入力常時 `role="combobox"` と、凍結対象 `StocktakePage.test.tsx` T3（L221、L227 の `screen.getByRole("combobox")` 単数 query = 既存部門フィルターの radix Select trigger を取得）が衝突し、D6 どおりの配線で T3 が複数要素エラー red になる。Coordinator 実読検証: T3 L221/L227 実在、他 4 画面 test への同型 `ByRole("combobox")` query は `rg -n 'ByRole\("combobox"' src/features/{receiving,manual-sale,return-exchange,disposal}/*.test.tsx` hit 0（例外は T3 の 1 test に限定可能）。裁定 = T3 の名前付き query への特定化のみ許容（assert 意図〈filter 変更で getStocktakeItems params が変わる〉は不変）。対案の D6 改訂は凍結済み a11y 契約の後退のため不採用。Goal 失敗定義 / AC3 / Ledger 凍結行を本 Amendment で同期。
- Writer 補助 review-only（Codex、Final Reviewer ではない）: 保存 event 後の listbox 消失だけでは reactive lock close と明示 `invalidateAndClose()` を弁別できない P2 を検出。5 画面の新規 W test で公開 controller API の同期呼出しと保存 command より前の順序を直接観測し、各 Page 呼出し削除 mutation の個別 red を確認。focused closure は P1/P2 なし。
- Final Review Double Audit（2026-08-05、対象 = PR #65 HEAD 5d3a575、Sonnet 5 fresh context ×2 並列・相互非参照）: 1 pass（Ledger 全行 re-verification + AC 実走）= P2×1 / P3×2、Ledger 17 行適合（negative-space 1 件除く）・AC1〜AC10 充足確認。2 pass（source docs 直読の Contract Audit lane: negative-space / Adjacent Pattern 実照合 / State Lifecycle / anti-tautology 構造検査）= P2×2。統合裁定: (F-A、両 pass 一致 = P2) `StocktakePage.suggest.test.tsx` の REQ-205 token 欠落 + 90-traceability REQ-205 行未登録（T4 検査は UI token で通過する仕様の隙間）→ accept、token 1 行追加 + 再生成で REQ-205 行へ反映（`cargo run --bin generate_traceability -- --check` OK 再確認）。(F-B、2 pass = P2) 外部 clear 経路（既存 commit 経路の `setSearchText("")` = onChange 非経由の value prop 変化、`useProductAddSuggest.ts` の value 監視 effect）が S 系で未検証 → accept、S21 新設（open リスト同期 close + timer cancel。in-flight 不採用は S10 既検証のため重複追加せず）+ X21 新設。(F-C1、1 pass = P3) act() 警告 → 単独・6 file 一括の双方で再実行し警告 0 件を実測、再現せず・再発時是正の disposition。(F-C2、1 pass = P3) S16 の null variant が否定形 assert → oracle は既存候補テーブル表記（React の null 非レンダリング）の正確な転記であり「部門なし」等の誤表示は検出可能、積極 assert 化は DOM 構造依存を増やすため見送り。是正実施 = Coordinator 直接（relay 予算 2/2 消費済みのため。3 file 軽微修正 + 再生成。writer 自己承認回避のため closure round は独立 fresh context で是正を検証する）。本記録 + S21/X21/REQ-205 是正 = gated Amendment 2。
- Findings Freeze 起点の確認: 本 packet の Final Review は Double Audit opt-in（Test Plan 参照）のため、Freeze は両 pass 完了後に発効する。closure round（是正検証）完了時に Freeze 発効を宣言する。
- Coordinator mutation 独立再実測（2026-08-05、clean tree HEAD 5a093d0、Writer 記録非参照の独立導出）: Matrix X1〜X21 の全注入を実装 2 file（useProductAddSuggest.ts / ProductAddSuggest.tsx）+ ReceivingPage.tsx 配線の実読から導出し、apply → `npx vitest run` → git checkout 復元で全件実測。結果 = kill 21/21、survivor 0（実行 script = scratchpad の mutation_rerun.py、各注入は count=1 検証付き）。復元後 tree clean・対象 test 全 green を確認。
- Final Review closure round（Sonnet 5 独立 fresh context、2026-08-05）: F-A/F-B の是正実在と構造的検証性、Coordinator 是正 diff（5d3a575..5a093d0）が裁定記録の範囲内で production code・既存 test 変更なしであること、vitest 21/21 green、traceability check OK、Amendments 行の append-only 性を独立検分。verdict「closure P1/P2=0、Findings Freeze 可」。
- 遷移記録（recording compression）: 本 state-only commit は `implementing -> local-verified -> independent-review -> human-confirm` の隣接 forward 遷移を一括実体化する（gated Amendment 2 の content commit により Phase は実質 implementing へ戻っていた）。中間遷移の evidence = local-verified: L1 full RESULT=PASS / END_TREE_STATE=CLEAN / MERGE_EVIDENCE_VALID=true（END_HEAD_SHA = 5a093d0、evidence log は .local/ci-evidence/。self-test fixture envelope 混在のため末尾 envelope を正とする）/ independent-review: Double Audit 両 pass + 統合裁定（上記）+ Coordinator mutation 独立再実測 21/21 + closure round verdict「closure P1/P2=0、Findings Freeze 可」/ human-confirm: Findings Freeze 発効 + Reviewed Content HEAD = 5a093d0 設定（本 commit）。残 Human Gate = L3（スキャナ supported sequence 互換性確認 + UX 確認）+ Ready 承認（介入 3/3 の 1 decision point に束ねる）。
- gated Amendment 3（2026-08-09、owner 承認 = 介入 4/5）: 初回 L3（Windows native、BC-BR1000U-W）で runtime blocker を検出 — 日本語 IME 有効時、HID スキャナのキー入力が IME に変換されて全角数字（`９７８４…`）の composition となり、末尾 Enter が変換確定に消費されて検索/追加が発火せず、次スキャンが連結される（HID がキーボード優先権を奪ったのではなく IME がキー入力を変換する機序を owner がメモ帳比較で実証）。裁定（会話 2026-08-09）: (1) backtrack = DEV_WORKFLOW の最早影響 phase 原則により `human-confirm -> implementing`（本 Amendment 直後の state-backtrack commit で実体化。Reviewed Content HEAD は再走の human-confirm 遷移 commit で再設定する）。(2) scope = 本 PR は 5 兼用欄のみ（A 案）。JAN 専用欄の共通正規化・保存 validation・`suggestPluTarget` の全角 13 桁 false 問題・フロント/BIZ 判定重複の整理は別 R3 change として Plans.md backlog へ起票。(3) 正規化契約 = catalog ⑮ SPEC-SUGGEST-D11 として設計正本へ追記（D1〜D10 本文不変）。値全体一致時のみ全角→半角写像、composition 中不加工、NFKC/連結分割/チェックディジット修正/スキャナ識別なし、paste 経由は known limitation。(4) commit 接続契約 = onCompositionEnd で数字のみなら正規化値 onChange 1 回 + 新設 prop `onComposedDigitsCommit(normalized)` 1 回。合成 Enter イベント偽造は不採用。画面側 commit 関数は正規化済み文字列を optional 明示引数で受ける（stale closure 禁止）。selectSuggestion 非経由・one-shot 二重発火 guard・isLocked() 尊重（詳細 = D11 (a)〜(e)）。(5) 予算 = 介入上限 3→5・実働 30→45 分・relay 2→3 の recorded-reason 調整（D-038。Owner Effort Budget 節に記録）。(6) 本 finding を post-freeze exception として登録し、旧「not yet frozen」重複行（未更新残存）は本 Amendment で削除。Matrix S22〜S27 / W13〜W17 / X22〜X26 は本 Amendment commit で Coordinator が追記。是正実装は Codex 発注（relay 3/3）、再走は implementing から遷移表どおり（independent-review で D11 追加分を含めて再審査）。
- Coordinator mutation 独立再実測（2026-08-09、clean tree HEAD 9d31540、Writer 記録非参照の独立導出）: X22〜X26 を実装実読（normalizeComposedDigits.ts / ProductAddSuggest.tsx onCompositionEnd / ReceivingPage 配線）から導出し、各注入 count 検証付きで apply → `npx vitest run`（対象 file）→ git checkout 復元で全件実測。結果 = kill 5/5、survivor 0（X22→S22 他 red / X23→S22・S25 red / X24→S24 単独 red / X25→S23 単独 red / X26→W13 単独 red）。事前 baseline 2 file 31 test green・復元後 tree clean を確認。変更 scope 検分 = 実装 commit 13 file が期待集合と一致、凍結 5 file の対 main diff は T3 限定例外のみ。
- Final Review 再走（Sonnet 5 独立 fresh context・worktree 隔離、2026-08-09、対象 = 9d31540 + 0d1ea42）: D11 (a)〜(e) 適合・D1〜D10 regression なし・Matrix S22〜S27/W13〜W17 一対一対応・anti-tautology（W13〜W15/W17 の引数値/旧値の二重 assert、W16 の isLocked gate 弁別を実読で確認）・oracle 独立性（S27 の util 直 import は単体 test として正当、数字列は synthetic のみ）・既存 test 不変・packet/Ledger/AC12 整合を静的実読で照合（実行検証は Coordinator 再実測側が担保する分担を明記）。verdict「P1/P2=0 で closure 可」。P3×2 の Coordinator 裁定: (P3-1) onCompositionEnd 内の `invalidateAndClose()` 呼出しが契約文未記載 → accept、catalog D11 (b) へ明文化一文を本 commit で追記（実装は正当、契約文の追記のみ）。(P3-2) 合成 ChangeEvent のオブジェクトスプレッド型キャスト → reviewer 判定どおり動作正当・修正不要、将来の React 更新時の注意点として本記録に残す。
- 遷移記録（recording compression、本 content commit に同乗）: `implementing -> local-verified -> independent-review -> human-confirm` の隣接 forward 遷移を一括実体化する（STATECAP 残枠温存のため state-only commit を使わない = Amendment 3 の予算追記どおり）。evidence = local-verified: Writer L1 full PASS/CLEAN（発注報告）+ Coordinator 独立 L1 full 再実行 RESULT=PASS（2026-08-09、HEAD 9d31540）/ independent-review: 上記 Final Review 再走 verdict + Coordinator mutation 独立再実測 kill 5/5 / human-confirm: Findings Freeze 再発効 + Reviewed Content HEAD = 9d31540 設定（本 commit）。残 Human Gate = 再 L3（Ledger L3 行の IME ON/OFF 項目表 6 点）+ Ready 承認（介入 5/5 の 1 decision point）。
- L3 closure（2026-08-09）: 再 L3 は Amendment 3 の基本動作（IME ON 一スキャン一追加・日本語手入力非破壊・IME OFF 退行なし・JAN 専用欄無変更・WebView2 実イベント順）を PASS としたうえで、間欠的な文字欠落（composition 確定前の上流欠落、D11 regression ではない）を検出し一旦 HOLD。切り分け（観測負荷除去）と Coordinator のスキャナ設定マニュアル実読調査（キャラクタ間遅延 default 1ms / Alt+Number の IME 迂回）に基づく owner の段階実測で、スキャナ側確定設定により欠落の再現なしを確認し closure。確定設定・実測数の正本は PR body（evidence ownership により本 packet へは転記しない）。是正はアプリ変更なし = D11 は防御層として残置、スキャナ設定と defense in depth を構成。予算実績 = 介入 5/5（recorded-reason 調整後の上限内）・relay 3/3・STATECAP forward 3/3。
- Findings Freeze: **re-frozen after Amendment 3 re-walk closure（2026-08-09）**; post-freeze exceptions: gated Amendment 3（2026-08-09、初回 L3 runtime blocker〈IME 全角数字化 + Enter 消費〉→ 再走 closure で解消済み。L3 の文字欠落は環境側是正で解消、post-freeze の実装変更なし）
- 遷移記録（recording compression、本 closeout commit に同乗）: `ready-hosted-final -> merge -> archive`。evidence = hosted final: CI run `31279446658` success（exact-HEAD 0b0179b）/ merge: owner による squash merge `bfce76a`（2026-08-09）/ archive: 本 commit で packet + Matrix を docs/archive/plans/ へ移動、Plans.md 完了記録同期。
