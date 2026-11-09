# Plan Packet: 実コード側 stale 実装状況表記の一括是正 + 在庫照会 CTA active link 化

## Workflow State

- Phase: archive
- Risk: R2
- Execution Mode: fable-window
- Plan Commit: 79d6d60
- Amendments: none
- Coordinator: Fable (Claude Code)
- Writer: Codex
- Plan Reviewer: Sonnet subagent (independent)
- Final Reviewer: Sonnet subagent (independent)
- Reviewed Content HEAD: 5824862
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required
- Human Gate: none

## Owner Effort Budget

- 介入回数上限: 3
- 実働時間上限: 30分
- relay 往復上限: 2（発注 lane 単位で計上）
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R2

Reason:
変更の大半はコメント・文言の現況化（runtime 無変更）。唯一の挙動変更は在庫照会詳細の disabled CTA 2 個を**既存 route への Link** に置き換える点で、route 定義・search state・command wire・DB のいずれも変更しない（route 新設なし、search param 追加なし）。operator 画面の変更を含むため human visual confirmation slot を立てる。

R3 トリガー「operator workflow」への該当性判定: 操作導線が 2 本増える点で同トリガーに接するが、同型先例 `docs/archive/plans/2026-07-16-sidebar-pending-links.md`（R3）が型契約変更（`NavItem.search` 追加相当）+ 20 項目横断のロジック変更を伴ったのに対し、本 change は 1 component 内の 2 CTA に閉じ、遷移先はいずれも navigation で既に常時到達可能な既存画面（新規到達性の開放ではなく到達導線の重複追加）。型契約・横断ロジック・stable contract のいずれにも触れないため R2 に留める。

## Goal

Goal Invariant:

### 最小完了条件

- 在庫照会詳細の「商品修正」「入庫記録」が disabled CTA（誤情報 tooltip）でなく実画面へ遷移する active link になり、利用者可視の誤情報が消える。
- src / src-tauri に残る「未実装・実装予定」系の stale 表記（実査確定 14 箇所、下記 Scope）が現況の正確な記述に是正される。

### 失敗定義

- 旧文言（「Phase 3 で実装予定」等）が対象領域に 1 箇所でも残る、または CTA の遷移先が実 route に到達しない。
- コメント是正のついでに `#[allow]` 属性の削除・dead code 整理など runtime/lint 面の変更へ scope が拡大する。

### 非目的

- 表示磨き（表示 UI の見た目改善）。L3 起源の表示磨き 8 件は別 packet。
- `#[allow(dead_code)]` / `#[allow(unused_imports)]` 属性自体の削除と real dead code の精査。
- 入庫記録画面への productCode 事前入力連携（search state 契約が必要、要望発生時に別 R3）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

実査（read-only agent による class 全数 sweep、2026-08-26。pattern class = 実装予定 / 未実装 / Phase N / pending / 後続PR / 将来形、判定は言及先機能の実装有無を route / module 実在で突合）で確定した 14 箇所。

**A. 在庫照会 CTA active link 化（挙動変更 + doc 同期）**

1. `src/features/stock-inquiry/components/StockDetailContent.tsx`: 「商品修正」を `/products/$code/edit`（params: code = 対象商品コード）への Link に、「入庫記録」を `/inventory/receiving` への Link に置き換える。既存 `ActiveCta`（在庫変動履歴、outline small Button + Link 型）と同型の実装を踏襲する（汎用化かコピーかは Writer 裁量。icon の有無・種別も Writer 裁量で既存 ActiveCta の作法に合わせる — 見た目の磨きは非目的）。商品修正 link は `returnTo` search param を**渡さない**単純遷移とする（保存/キャンセル後は既定どおり `/products` へ着地。Non-scope 参照）。置換後に利用者不在となる `DisabledCta` component と L28 の「将来実装予定」コメントは撤去する（未使用関数残置は lint fail）。撤去に伴い未使用化する `Tooltip` / `TooltipContent` / `TooltipProvider` / `TooltipTrigger` の import も除去する（他使用箇所なしを実査確認済み）。
2. `src/features/stock-inquiry/components/StockDetailContent.test.tsx`: 既存 REQ-301 link test（`renderWithRouter` + `findByRole("link")` + href assert）と同型で、商品修正 link（href `/products/<code>/edit`）・入庫記録 link（href `/inventory/receiving`）の 2 test を**追加**する。既存 test は無改変。
3. `docs/function-design/58-ui-stock-inquiry.md`: (a) §58.7 の disabled CTA 契約 3 行（L504-506）を撤去し、直後の「在庫変動履歴」active link 記述と同型で「商品修正」「入庫記録」の active link 契約（遷移先 route を明記。商品修正は `returnTo` を渡さない単純遷移である旨を 1 行含める）を追記する。(b) §58.6 新規追加ファイル一覧表の `StockDetailContent.tsx` 行（L117）の責務記述「商品修正/入庫記録 disabled CTA + 在庫変動履歴 active link」を「商品修正/入庫記録/在庫変動履歴 active link」へ是正する（同表の `TruncatedResultsAlert` 行が持つ更新注記の慣例に従う）。(c) 更新履歴表へ日付 + 要旨の行を追加（PR 番号非転記、PR #6 表記規約と同じ）。

**B. コメント・文言の現況化（runtime 無変更）**

4. `src/config/navigation.ts` L55: 「Phase 3/4 以降で各画面着手時に to を実 path に + status を "active" に切り替える。」— 直前 L54「全画面は route 実装済みで active。」と矛盾する切替完了後の消し忘れ。削除する。
5. `src/features/home/components/InventoryActionGrid.tsx` L5: 「全 pending（Phase 3 UI-02〜05 まで未着手）。」→ 全項目 active の現況へ。
6. `src/features/home/components/MiscActionRow.tsx` L5: 「全 pending（Phase 4 UI-10/UI-11a/UI-11b まで未着手）。」→ 同上。
7. `src-tauri/tests/design_compliance_test.rs` の該当 6 行（L156 / L194 / L215 / L221 / L273 / L282-283。Phase 6 ブロックの 2 箇所は同一クラスタ）: 「設計書作成済み、実装は後続PR」「Phase 5/6 …実装はPR-2以降」「PR-2/PR-3で実装予定」— 対応モジュールは全て実在（同ファイル自身の map が登録済み）。実装済みの現況コメントへ是正。
8. `src-tauri/src/lib.rs` L1: 「UI層未実装のため、IO/BIZ層の一部関数・型が未使用。UI実装時に解消される」— UI 層は広範に実装済みで前提が事実誤り。`#[allow(dead_code)]` 属性は維持し、理由文言のみ現況の正確な記述へ（例: 未使用 symbol の個別精査は将来判断、の趣旨）。
9. `src-tauri/src/biz/csv_import_service/mod.rs` L17: 「CMD層が未実装のため一部シンボルは未使用」— `cmd/csv_import_cmd.rs` が全 symbol を消費済み。現況へ。
10. `src-tauri/src/biz/mod.rs` L17: 「UI層未実装のため一部はまだ未使用」— cmd 層が re-export を直接使用済み。現況へ。
11. `src-tauri/src/db/mod.rs` L32: 「UI層未実装のため一部はまだ未使用」の前提部分が失効。ただし #9/#10 と異なり、この re-export block の symbol が `crate::db::X` 経路で消費されている実証はない（実消費は `biz/mod.rs` の submodule 直接 re-export 経由）。新文言は消費実態を断定せず、「BIZ/CMD 層で使用する型の集約点として維持し、未参照 symbol の残存が `#[allow(unused_imports)]` の理由」という中立の現況記述にする。消費実態を主張する場合は rg 実測を根拠に添える。
12. `src-tauri/src/cmd/integrity_cmd.rs` L4: 「設定・ログ・バックアップは Phase 6。」— いずれも実装済み（`lib.rs` の CMD-11 コマンド一覧と同一 crate 内で矛盾）。本 module が BIZ-07 の 2 コマンドのみを扱う旨の現況記述へ。
13. `src/features/csv-import/reducer.ts` L3-4: 「Vitest 着手後に…網羅 unit test を retroactive 追加する想定」— `reducer.test.ts` 実在。現況へ（網羅数の数値主張は書かない）。
14. `src/features/home/lib/count-stock-status.ts` L5: 「Vitest 着手後に unit test 追加」— `count-stock-status.test.ts` 実在。現況へ。

**C. 検査・生成**

- `cd src-tauri && cargo run --bin generate_traceability -- --check` を実行し、drift があれば再生成を同 PR に含める（REQ token を触る test 追加の完了条件）。

表記規約（PR #6 と同一）: 新規表記は「実装済み」基調で PR 番号非転記（D-050 + rehome の番号空間重複）。dated 履歴 block は据え置き。

## Non-scope

実査で確認し、意図的に除外した同型 class（reviewed-and-excluded）:

- `src/features/daily-sales/components/ExportBar.tsx` / `monthly-sales` 側の印刷 tooltip「準備中（Phase 4 で実装予定）」: `window.print` は実際に未実装（grep 0 件）で核心の事実は正確。時期ラベルの再検討は印刷機能の要否裁定（Plans.md backlog「日報画面のExcel印刷・バインダー代替受入」）と同時に行う。
- `src-tauri/src/lib.rs` L251「Phase 2 以降で段階的に拡張する。」: 明確な事実誤りではない grey zone。据え置き。
- `src/features/home/components/ActionButton.tsx` L63「後続フェーズで着手予定」: pending 項目向けの汎用 fallback 文言で、将来 pending item 追加時のための正当な一般化。据え置き。
- `#[allow(dead_code)]` / `#[allow(unused_imports)]` 属性の削除と dead code 精査（失敗定義参照）。
- 入庫記録への productCode 事前入力（非目的参照）。
- 商品修正 CTA からの `returnTo` 継続性（保存/キャンセル後に在庫照会へ戻る挙動）: 非対応の意図的除外。`sanitizeProductListReturnTo`（UI-01b-D2、`src/features/products/lib/return-to.ts`）は `/products` 系 path のみ受理し他 path は `/products` へ fallback するため、`/stock` への戻りは現契約で表現不能。対応するなら returnTo 契約拡張（search state 契約変更）の別 R3。既存唯一の遷移元 `ProductTable.tsx` の returnTo 連携は無改変。
- L3 起源の表示磨き 8 件 batch（別 packet で消化）。

## Acceptance Criteria

検索 oracle は自己参照回避のため検索対象を `src` / `src-tauri` / `docs/function-design` の明示 path に限定する（本 packet の在る `docs/plans` を検索対象に含めない。rg の負 glob は環境既知 bug のため使わない）。

- AC-1: `rg -c "Phase 3 で実装予定" src docs/function-design` が hit 0（exit 1）。
- AC-2: `rg -c "将来実装予定" src` が hit 0。
- AC-3: `rg -c "DisabledCta" src` が hit 0（component 撤去の確認）。
- AC-4: `rg -F -c 'to="/products/$code/edit"' src/features/stock-inquiry/components/StockDetailContent.tsx` ≥ 1、かつ `rg -F -c 'to="/inventory/receiving"' src/features/stock-inquiry/components/StockDetailContent.tsx` ≥ 1。
- AC-5: `rg -c "全 pending" src` が hit 0。
- AC-6: `rg -c "各画面着手時" src/config/navigation.ts` が hit 0。
- AC-7: `rg -c "実装は後続PR|で実装予定|実装はPR-2以降" src-tauri/tests/design_compliance_test.rs` が hit 0。
- AC-8: `rg -c "CMD層が未実装|UI層未実装|UI実装時に解消" src-tauri/src` が hit 0。
- AC-9: `rg -c "Vitest 着手後に" src` が hit 0。
- AC-10: 対 oracle（新文言 presence、両導線）: `rg -F -c '/products/$code/edit' docs/function-design/58-ui-stock-inquiry.md` ≥ 1、かつ `rg -F -c '/inventory/receiving' docs/function-design/58-ui-stock-inquiry.md` ≥ 1、かつ AC-1 が同 file の旧文言 0 を保証。
- AC-11: `StockDetailContent.test.tsx` の追加 link test 2 本が green、既存 test は追加のみで無改変（`git diff` で削除・変更行 0 を確認）。
- AC-12: L1 gates PASS: frontend suite（typecheck / lint / format / test / build）+ `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`（design_compliance_test 含む）+ `bash scripts/doc-consistency-check.sh` + `cargo run --bin generate_traceability -- --check` drift 0。
- AC-13: `rg -c "設定・ログ・バックアップは Phase 6" src-tauri/src` が hit 0（Scope #12 の専用 oracle）。
- AC-14: `rg -c "disabled CTA" docs/function-design/58-ui-stock-inquiry.md` が hit 0（§58.6 L117 表 + §58.7 の両所是正の確認。現状 hit はこの 2 箇所のみで dated 履歴 block に hit なしを実査確認済み）。

## Design Sources

- Requirements / spec: `docs/spec/requirements.md` REQ-301 / REQ-302（在庫照会）
- Architecture: 変更なし（`UI -> CMD -> BIZ -> IO/MNT` 境界に触れない）
- Function / command / DTO: `docs/function-design/58-ui-stock-inquiry.md` §58.7（本 PR で改訂）
- DB: 変更なし
- Screen / UI: `docs/SCREEN_DESIGN.md`（在庫照会 / 商品修正 / 入庫記録の画面実在の裏付け、無改変）
- Decision log / ADR: 変更なし（durable な新規判断なし — CTA link 化は §58.7 の契約改訂として doc 内に記録）

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status |
|---|---|---|
| Backend function / command / repository / validation / error | なし（コメントのみ） | existing sufficient |
| Command / DTO / generated binding / wire shape | なし | existing sufficient |
| DB / transaction / audit / rollback / migration | なし | existing sufficient |
| Screen / UI / route state / Japanese wording | 58-ui §58.7 CTA 契約 | updated in this PR |
| CSV / TSV / report / import / export format | なし | existing sufficient |
| Durable decision / ADR | なし | existing sufficient |

## Registration / Generation Obligations

| 新規追加物 | 登録・生成義務 |
|---|---|
| REQ coverage 追加（test 追加） | `cargo run --bin generate_traceability -- --check` で drift 確認、drift 時は再生成を同 PR に含める（追加 test は既存 REQ-301 token の同一 file 内追加のため drift なし見込み、check で確定） |

route 新設・command 新設・doc 新設・operator 画面新設: 該当なし（既存 route への Link 追加のみ）。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| REQ-301 / REQ-302 | 58-ui §58.7 StockDetailContent | §58.7 CTA 契約改訂（本 PR） | 遷移先実装済みのため disabled 前提が失効。prefill 連携は search state 契約を要するため不採用（非目的） | `StockDetailContent.tsx` | 追加 link test 2 本 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: yes（CTA 契約は 58-ui §58.7 に改訂後の姿で残る）
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: なし
- Assumptions and constraints: 遷移先 route `/products/$code/edit` / `/inventory/receiving` の実在は routeTree + navigation.ts（全項目 active）で実査確認済み
- Deferred design gaps, risk, and follow-up target: 入庫記録 prefill（非目的、要望発生時に別 R3）
- Test Design Matrix can cite design decision IDs or source doc sections: Matrix は R2 optional 判定で省略（機械 oracle は AC-1〜AC-14）
- Absolute guarantee / escape hatch self-check completed: 該当なし（保証文言の新設なし）

## Impact Review Lenses

not applicable — 起点は repo 内実査（stale 表記の突合）であり、実地調査・実機・外部 tool・POS 連携・format 変更のいずれにも該当しない。

## Design Readiness

- Existing design docs are sufficient because: 遷移先 2 画面の契約は既存 doc（55/61 系 + SCREEN_DESIGN）で確立済み。本 PR の設計変更は 58-ui §58.7 の CTA 契約改訂のみで、同 PR 内更新で完結する
- Source docs updated in this PR: `docs/function-design/58-ui-stock-inquiry.md` §58.7
- Design gaps intentionally deferred: 入庫記録 prefill
- Durable decisions discovered in this plan and promoted to source docs: なし

Minimum design checks:

- Layer ownership: UI 層のみ（コメントは各層だが runtime 無変更）
- Backend function design: 変更なし
- Command / DTO / data contract: 変更なし
- Persistence / transaction / audit impact: なし
- Operator workflow / Japanese UI wording: CTA label「商品修正」「入庫記録」は既存文言を維持、tooltip の誤情報のみ消える
- Error, empty, retry, and recovery behavior: 変更なし（Link 遷移は既存 route の挙動）
- Testability and traceability IDs: 追加 test は REQ-301 token（既存 file 内）

## Contract Probe

N/A — 外部前提なし（遷移先 route の実在は repo 内実査で確定済み、R2）。

## Contract Coverage Ledger

R2 簡易版（改訂する §58.7 の契約行のみ）:

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| §58.7 商品修正 active link（`/products/$code/edit`） | `StockDetailContent.tsx` | 追加 link test（href assert） | 視覚確認（dev 画面） |
| §58.7 入庫記録 active link（`/inventory/receiving`） | `StockDetailContent.tsx` | 追加 link test（href assert） | 視覚確認（dev 画面） |
| §58.7 在庫変動履歴 active link（既存、無改変） | 既存 `ActiveCta` | 既存 REQ-301 test（無改変） | non-scope |

## Test Plan

Test Design Matrix は R2 optional 判定で省略（理由: oracle は AC-1〜AC-14 で機械化済み、挙動変更は link 2 本のみで RTL の href assert が mutation〈遷移先誤り・link 欠落〉を直接検出する）。

- targeted tests: 追加 link test 2 本（既存 REQ-301 test と同型、`renderWithRouter` + `findByRole("link")` + href assert）
- negative tests: 不要（disabled 状態の撤去であり、失敗経路の新設なし）
- compatibility checks: 既存 test 無改変（AC-11）、design_compliance_test はコメントのみ変更で assert 無改変
- data safety checks: N/A
- main wiring/integration checks: L1 full（AC-12）

## Boundary / Wire Contract

N/A — JSON / CSV / DTO / bindings / DB 互換のいずれにも触れない。

## Review Focus

- Scope 14 箇所の全数消化と、Non-scope（reviewed-and-excluded）の判定妥当性
- コメント現況化の新文言が**新たな事実誤りを持ち込まない**こと（是正で書く主張も実読裏取り — 特に #8〜#12 の理由文言）
- 58-ui §58.7 改訂が「在庫変動履歴」記述と同型で、旧 disabled 契約の残骸（aria-disabled 3 層パターン言及等）が残らないこと
- 既存 test 無改変（追加のみ）の diff 確認

## Spec Contract

N/A（R2。改訂契約は Contract Coverage Ledger 参照）。

## Trace Matrix

N/A（R2、Design Intent Trace 参照）。

## Data Safety

N/A — 実データ・secrets・破壊的操作なし。

## Implementation Results

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

- Codex Writer が発注 1 本・fail-closed 停止 0 で完走。Scope A/B/C を単一 content commit で実装（在庫照会 CTA 2 個の Link 化 + DisabledCta/Tooltip import 撤去 + link test 追加 + 58-ui §58.6/§58.7 同期 + コメント現況化 11 箇所 + 90-traceability 再生成）。
- AC-1〜AC-14 全 PASS（Writer 実測 + Coordinator spot 再実測 + Final Reviewer 独立再実測の三者一致）。L1 full CLEAN PASS（evidence SHA と log 位置は PR body を正とする）。
- PR: https://github.com/higekiri-w/inventory-system-desktop/pull/7

## Review Response

- Findings Freeze: frozen after Final Review; post-freeze exceptions: none.
- Final Review（独立 Sonnet、read-only）2026-08-26: Scope 突合 15/15 file・packet 外 hunk 0 / AC 14/14 独立再実測 PASS / 新文言監査 11 箇所 適合・事実誤り 0（#9/#10 の消費主張は rg 実測裏取り、#11 は中立記述の指示遵守を確認）/ 58-ui 改訂整合（aria-disabled 3 層パターン残骸 0）/ traceability drift 0 / 追加 link test の mutation 感度（link 欠落・遷移先取り違えの 2 mutant 検出可）を机上確認。P1 0 / P2 0 / P3 0。
- state-only 遷移 `implementing->local-verified->independent-review->human-confirm` の根拠: content candidate の L1 full CLEAN evidence（implementing->local-verified）/ 独立 Final Reviewer 監査完了（local-verified->independent-review）/ findings P1/P2 = 0 裁定済み + Reviewed Content HEAD 設定（independent-review->human-confirm）。
- Human Gate 消化 2026-08-26: 視覚確認 PASS（owner、Windows 実機 dev、CTA 2 導線の遷移 + 文言 + returnTo なし着地の仕様確認。介入 2/3）/ Ready 承認（介入 3/3、CI 通過後の merge と closeout 実施まで owner 委任）。
- state-only 遷移 `human-confirm->ready-hosted-final` の根拠: Human Gate 全消化 + owner Ready 承認。exact HEAD の L1 full evidence と hosted run は PR body を正とする。

### Plan Gate 記録（append-only）

- Plan Gate rally: 独立 Sonnet Plan Reviewer、3 round 収束（P1+P2: 5 → 1 → 0。round 1 = AC-13/14 欠落・58-ui §58.6 L117 表 stale 残置・R2 判定の operator workflow トリガー未反証・db/mod.rs 消費主張未実証・AC-10 片側検証、round 2 = returnTo 継続性の意図的除外化、round 3 = delta 検証で新規指摘 0）。是正 commit は plan-first `79d6d60` に続く `53d765b` / `bc3597e`（いずれも Plan Gate 前の in-place 是正、gated amendment ではない）。
- owner Plan Gate 承認 2026-08-26（介入 1/3。採否 3 点 = Scope 14 箇所 / returnTo 非対応の単純遷移 / R2 維持、すべて承認）。
- state-only 遷移 `plan-draft->plan-gate->plan-approved->implementing` の根拠: packet 完成・commit 済み（plan-draft->plan-gate）/ 独立 Plan Reviewer P1/P2 = 0 + Plan Commit 記入 + plan-first commit が全実装 commit に先行（plan-gate->plan-approved）/ 実装開始許可（plan-approved->implementing）。
