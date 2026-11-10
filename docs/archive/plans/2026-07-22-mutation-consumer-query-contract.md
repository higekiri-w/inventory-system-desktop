# Plan Packet — 監査是正 順 4: mutation→consumer query 契約と回帰テストの同時整備

## Workflow State

- Phase: archive
- 遷移記録（ready-hosted-final → merge → archive、2026-07-23、圧縮記録）: merge evidence = 三点一致（PR HEAD = final L1 SHA = hosted success headSha = `2a2f017`）+ owner 委任に基づく Coordinator squash merge（PR #21、merge commit `187d7a7`）。archive = 本 closeout commit（packet/Matrix の archive 移動 + WER + Plans.md/PROJECT_HANDOFF 同期）
- 遷移記録（append-only 圧縮記録、2026-07-23）: implementing → local-verified（evidence = 実装 commit `e72ccec` 上の `local-ci.sh full` CLEAN + M1〜M4 red 実測 + Draft PR #21 三点一致。件数・SHA 証跡は PR body / .local/ci-evidence が authority — Evidence Ownership に基づき 2 pass P2-2 で本行から件数を除去）→ local-verified → independent-review（Double Audit 1 pass = Fable inline 突合 blocker 0、2 pass = Codex 独立 mutation testing）→ **2 pass が P1×3 検出により implementing へ差し戻し**（P1/P2=0 不成立のため human-confirm 遷移は発生せず、Reviewed Content HEAD は pending へ復帰 = 2 pass P2-1。この往復は未 push の非 canonical state commit 2 件を soft reset し本 content commit へ圧縮記録 — 2 pass P2-2。gate skip なし: 両 pass とも実施済み）
- 旧遷移記録（state-only 圧縮の append-only 記載）: plan-gate → plan-approved の evidence = owner plan 承認（2026-07-23、介入 1/2、Plan Gate 収束 `48bf156` に対する承認）/ plan-approved → implementing の evidence = Coordinator による Codex 実装発注書交付（本 commit 直後）。圧縮記録は gate skip を許可しない
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: 746ce6e
- Amendments: e1ed3d4, 2fa288f, 9a4b0a2
- Coordinator: Fable（main thread）
- Writer: Codex（実装発注、レビュー前に PR 作成）
- Plan Reviewer: Plan agent self rally（独立 context、新規指摘 0 まで）→ Codex plan review。順 3 実装 follow-up で試行した逆順（Codex 先行）は「正本確定済み実装 follow-up」条件付きの手法であり、本 change は契約が未正本のためオーソドックス順に戻す
- Final Reviewer: Double Audit（1 pass = Fable inline 契約突合 / 2 pass = Codex 独立 + 実 mutation testing）。R3 だが operator-visible state lifecycle（stale 表示の解消契約）に触れるため DEV_WORKFLOW Contract Audit の recommended second pass を採用
- Reviewed Content HEAD: 1cf1bbd
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required
- Human Gate: none（plan 承認 = 介入 1/2、Ready 承認 + merge/closeout 委任 = 介入 2/2、いずれも 2026-07-23 消化）。Windows native L3 は not-required — cache invalidation は視覚意匠変更を伴わず、「mutation 後に旧値が fresh 扱いで表示されない」は vitest の invalidateSpy 検査 + 実 mutation 感度実測で完結する（DEV_WORKFLOW L3 Eligibility: 自動テストで検証可能な挙動は L3 に置かない）。roadmap 1-4 受入テスト（一気通貫台本）が実機での事後検証点を兼ねる

## Owner Effort Budget

- 介入回数上限: 2
- 実働時間上限: 30分
- relay 往復上限: 2

**超過記録**: relay 実績 3 / 予算 2（Codex plan review round 3 = round 2 P1「旧共有-SSOT 指示の残存」閉塞の第三者確認。owner が超過を承認して round 3 実施を選択、2026-07-22。超過要因 = round 1 反映を追記方式で行い旧文言の置換漏れが生じた Coordinator 側の手戻り）。

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
UI route/search state や DB スキーマは変えないが、operator workflow の中核（mutation 直後の在庫・履歴・在庫少表示の正しさ）を横断 16 mutation で変更し、UI_TECH_STACK / function-design 複数 doc の安定契約（invalidation 方針）を改訂する。stable contract の変更 + operator workflow 影響で R3。R4 要素（destructive lifecycle / migration / restore）はない。

## Goal

Goal Invariant:

### 最小完了条件

- mutation→consumer query の期待 invalidation 集合が単一 SSOT（`src/lib/invalidation-contract.ts`）として存在し、監査 findings P5-1 / P5-2 / P5-3 / P5b-1 / P5b-2 / P5b-3 の全欠落が SSOT 経由の invalidate で解消されている。
- 各 mutation のテストが「実装の invalidate 列挙の写し」ではなく、**契約表 D-052-Cn から転記した独立 test oracle**（production の invalidation-contract.ts を import しない）との**完全一致比較**（順序非依存・重複検出付き、欠落・余分・重複いずれも red）になっており（P8-2）、production contract から key を 1 つ除去する mutation で該当テストが red になることを代表経路で実測済み（oracle 独立は Codex round 1 P1-1 — production と test が SSOT を共有すると mutant の両側が同時に縮み green survivor になるため）。
- 契約の導出原則（mutation が書く table を読む query は invalidate する。除外は明示列挙 + 理由）と除外表が設計正本（UI_TECH_STACK）に定着している。

### 失敗定義

- query key の追加だけで終わり、テストが実装列挙の写しのまま残る（裁定の Goal Invariant 指定違反）。
- 契約表が frontend の現行実装から逆算されて作られ、backend 書込み集合との突合根拠を持たない（監査が指摘した「実装を写す」構造の再生産）。
- 6 findings のどれかが「画面単位の個別修正」で処理され、SSOT を経由しない invalidate 呼び出しが新規に増える。

### 非目的

- P5-4（operation-logs / integrity latest-check の literal key 直書きの factory 収容）。latest-check literal は本 change で touch しない — rally round 1 P2-1 / round 2 B の裁定で invalidate 対象から一律除外が確定しており、factory 化も P5-4 保全のため行わない（round 3 P3-E で条件文を簡略化）。
- staleTime / gcTime の値の再設計。現行値は維持し、invalidation の有無のみを契約化する。
- refetch 戦略（mount 時 refetch 等)の変更。
- backend（Rust）側の変更。書込み集合は現状を正とし、frontend の invalidation を追随させる。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

- `src/lib/invalidation-contract.ts` 新設: mutation 識別子 → 期待 invalidation 集合（queryKeys factory 参照）の SSOT 定数 + 適用 helper。
- 除外判断の基準（rally round 3 P2-D で明文化、Codex round 1 P1-2 で列粒度へ精緻化）: 導出は **table.column 粒度** — mutation が確定した table.column を query result が読む場合は invalidate する。除外は除外表 E1〜E6 のみを根拠とし、staleTime 設定値は除外根拠にしない（設定値を契約根拠にすると「staleTime を変えると契約が変わる」隠れ結合を作るため。latest-check の staleTime:0 は E1 の補強論拠）。列カテゴリ基準: **数量列**（stock_quantity）と**状態列**（plu_dirty / is_discontinued 等）を読む query は invalidate、**商品 master 表示列**（name / 部門 / 単位 / 価格）の JOIN stale は E3（一般化）で許容。**逆方向も契約違反** — invalidate 対象の prefix 配下に「書込み列を読む正当な consumer」が一つも存在しない invalidate（過剰）は導出根拠を持たず、現行実装の過剰 invalidate は契約化時に除去する（Writer fail-closed 裁定 2026-07-23: CSV commit/rollback の pluDirty 除去が初適用例）。**prefix collateral の許容（第 2 次裁定 2026-07-23）**: 正当な consumer を 1 つでも含む prefix invalidate は正当であり、同 prefix 配下の他 child query が巻き込まれる collateral は許容する（例: 入庫の inventoryRecords.root は receiving 系 list が正当 consumer、returnDetail 等の巻き込みは collateral。E2 の productForm.suppliers はこの一般原則の個別事例）。leaf 単位の厳密化は全 root の再設計を要するため本 change の scope 外。
- `src/lib/query-keys.ts`: `stockMovements` への root/prefix helper 追加（現状 helper が無く P5-2 の是正が構造的に不可能）。`productForm` への root/prefix helper 追加（一括 import が products を UPDATE し `productForm.product(id)` がそれを読むため、導出原則から必要 — 対象 id 群が bulk で特定不能なため prefix invalidate。rally round 1 P2-2 で確定）。`dailySales` への root/prefix helper 追加（現行の `["daily-sales"]` literal prefix invalidate 4 箇所 = csv-import commit/rollback 2 + daily-report-import commit/rollback 2 を型安全な factory 参照に置換 — SSOT が直接参照する key に literal を残すと D-4 直書き禁止規範と矛盾するため。rally round 2 A で確定、箇所数は round 3 P2-C で実測訂正。operation-logs 系 literal 3 key は P5-4 保全のため対象外を維持し、query-keys.ts D-4 ヘッダへ「P5-4 是正までの明示例外（operation-logs 2 key + integrity latest-check、follow-up = 監査是正 P5-4 単位）」の期限付き注記を追加する — Codex round 1 P2-3 第 2 案採用）。
- `StocktakePage` の error-path 防御 refresh（conflict / not-in-progress 分岐の計 4 箇所）を named helper（例: `refreshStocktakeStateAfterConflict`）へ集約し、AC の静的検査で helper 単位の allowlist にする（Codex round 1 P2-2）。
- 対象 16 mutation の onSuccess を SSOT 経由の invalidate へ置換（下記契約表）: 商品 create / 商品 update・廃番 toggle / 商品一括 import / 入庫 / 返品 / 手動販売 / 廃棄 / 売上 CSV commit / 売上 CSV rollback / 日報 import commit・rollback / 棚卸し開始 / 棚卸し明細個数更新 / 棚卸し確定 / 整合性補正 / 閾値保存（全成功 + 部分成功） / PLU 書出し。うち日報 import / 棚卸し開始 / 棚卸し明細個数更新は集合の増減なし（SSOT 経由化のみ — D-052-S1 の絶対保証を全 production mutation で成立させるため。後者 2 つは rally round 2 C で網羅漏れとして追加。PLU 書出しは Codex round 1 P1-2 で集合拡張へ変更）。契約行は 16、独立 success handler は 18 経路（update / toggle、日報 commit / rollback が各 1 行 2 経路 — Codex round 1 P1-3。テスト oracle は経路単位）。
- 対象 mutation のテストを D-052-Cn 独立転記 oracle との完全一致検査へ書換え（P8-2。production の invalidation-contract.ts を test が import・参照することは禁止 — Codex round 1 P1-1 / round 2 P1）。invalidation 検査が存在しない ProductFormPage / IntegrityCheckPage / ThresholdSettingsPage（部分成功分岐）へ新規追加。`useCsvImportFlow` は hook 実行 test 自体が不在（page test は hook を idle mock で全置換 = P8b-1 で確認済みの構造）のため、renderHook + QueryClient wiring test を新設する（`useDailyReportImportFlow.test.tsx` パターン踏襲。rally round 1 P1-2）。
- 設計正本の改訂: UI_TECH_STACK §2.5 invalidation 節へ導出原則 + 除外表 + SSOT 参照を正本化。function-design 51 / 55 / 56 / 57 / 58 / 60 / 61 / 62 / 63 / 64 / 66 / 67 / 69 / 73 / 75 の invalidation 記述を契約 SSOT 参照へ同期（現行列挙を残すと doc 側が新たな写しになる。56 = 日報サマリ表示 / 67 = PLU 書出し — rally round 1 P3-2。57 = 月次売上 — rally round 2 E。58 = 在庫照会は §309 で CSV invalidation の具体列挙を正本化しており本 change の直接対象 — Codex round 1 P2-4）。45-cmd-daily-report-import §45.5 の汎用 invalidation 記述も SSOT 参照へ寄せる。decision-log へ D-052（契約 SSOT 配置と除外原則）新設。

### 契約表（backend 書込み集合からの導出。証拠 = 本 packet の Design Intent Trace 及び調査記録）

書式: mutation → 追加が必要な invalidation（現行との差分)。全量は invalidation-contract.ts に実装時確定。

| mutation | backend 書込み | 現行 invalidation | 追加（差分） |
|---|---|---|---|
| 商品 create（form） | products / departments.next_seq（独自コード発番時 = product_repo.rs:279、DTO 含有だが現 UI 未利用 → E6） / inventory_movements（initial_stock>0） / stocktake_items（棚卸し中） / op_logs | productList.root のみ | lowStock, stockInquiryRoot, pluDirty, stockMovements.root（新規商品に既存 cache は無く実効 no-op だが、導出原則の一様性を優先して収載 — 条件分岐の場合分けが契約の検証性を下げるため。rally round 1 P3-1）, stocktake.itemsRoot（staleTime:0 だが除外判断基準どおり table ベースで収載 — round 3 P2-D の対称適用） |
| 商品 update・廃番 toggle（独立 success handler 2 経路 = ProductFormPage.tsx:133 / :157、契約集合は共通 — Codex round 1 P1-3。name 等 master 表示列の他 consumer JOIN stale は E3 で許容） | products（plu_dirty, is_discontinued 含む） / price_history / op_logs | productList.root のみ | productForm.product(id), lowStock, stockInquiryRoot, pluDirty, stockMovements.root（detail が products を読む） |
| 商品一括 import | create と同等（上書き時は products UPDATE） | productList.root, lowStock, stockInquiryRoot, pluDirty | stockMovements.root, stocktake.itemsRoot（staleTime:0 だが table ベースで収載 — round 3 P2-D）, productForm.root（上書き対象 id の編集 form cache が stale になるため。id 群は bulk で特定不能 → prefix invalidate。rally round 1 P2-2） |
| 入庫 | receiving_records/items, products.stock_quantity, inventory_movements, op_logs | receivings.root, inventoryRecords.root, productList.root, lowStock, stockInquiryRoot | stockMovements.root, productForm.root, stocktake.itemsRoot（stock_quantity を form 表示・棚卸し一覧 JOIN が読む — Codex round 1 P1-2 の列粒度導出） |
| 返品 | return_records/items + （未処理時）products.stock_quantity, inventory_movements, op_logs | returns.root, inventoryRecords.root +（未処理時）productList.root, lowStock, stockInquiryRoot | （未処理時）stockMovements.root, productForm.root, stocktake.itemsRoot |
| 手動販売 | manual_sales/items, sale_records, products.stock_quantity, inventory_movements, op_logs | inventoryRecords.root, productList.root, lowStock, stockInquiryRoot, dailySales(date), monthlySalesRoot | stockMovements.root, productForm.root, stocktake.itemsRoot（stock_quantity を form 表示・棚卸し一覧 JOIN が読む — Codex round 1 P1-2 の列粒度導出） |
| 廃棄 | disposal_records/items, products.stock_quantity, inventory_movements, op_logs | disposals.root, inventoryRecords.root, productList.root, lowStock, stockInquiryRoot | stockMovements.root, productForm.root, stocktake.itemsRoot（stock_quantity を form 表示・棚卸し一覧 JOIN が読む — Codex round 1 P1-2 の列粒度導出） |
| 売上 CSV commit | csv_imports, csv_import_errors（commit.rs:173、読者 = csvImportLists 配下で現行 invalidate 済み）, sale_records, products.stock_quantity（pos_stock_sync）, inventory_movements（+上書き時 void 系） | csvImportLists, ["daily-sales"], lowStock, pluDirty（**過剰 — 除去対象**）, stockInquiryRoot | productList.root, monthlySalesRoot, stockMovements.root, productForm.root, stocktake.itemsRoot。**除去: pluDirty**（CSV 経路は products.plu_dirty を書かない = commit.rs / rollback.rs / inventory_repo 全て非接触を実証。書かない column の invalidate は導出根拠なし — Writer fail-closed 裁定 2026-07-23） |
| 売上 CSV rollback | sale_records.is_voided, inventory_movements.is_voided, products.stock_quantity, csv_imports.status（rollback.rs:53、読者 = csvImports で現行 invalidate 済み） | 同上 | productList.root, monthlySalesRoot, stockMovements.root, productForm.root, stocktake.itemsRoot。**除去: pluDirty**（同上） |
| 日報 import commit/rollback | daily_report_imports + summary/payment/department lines, op_logs（sale_records / products / movements 非接触 — commit.rs:98-184 / rollback.rs:30 で確認済み） | dailyReportImportLists, ["daily-sales"], monthlySalesRoot | なし（確定。dailySales は official_daily_report として daily_report 系を読む = sales_service.rs:208-216。monthlySales も official_department_totals で daily_report 由来データを読む = sales_service.rs:246-249 → monthlySalesRoot は維持で確定。rally round 1 P2-3） |
| 棚卸し開始 | stocktakes（INSERT）, stocktake_items（生成、廃番 stock=0 自動確定含む） | stocktake.status, stocktake.itemsRoot | なし（現行維持、SSOT 経由化のみ — rally round 2 C。error 分岐の防御 refresh は契約対象外） |
| 棚卸し明細個数更新 | stocktake_items（actual_count） | stocktake.itemsRoot | なし（現行維持、SSOT 経由化のみ — rally round 2 C） |
| 棚卸し確定 | stocktake_items, stocktakes, products.stock_quantity（差異品）, inventory_movements（差異品）, op_logs + 確定後 integrity_check ログ | stocktake.status, itemsRoot, lastCompleted | productList.root, lowStock, stockInquiryRoot, stockMovements.root, productForm.root（Codex round 1 P1-2 列粒度導出。stocktake.itemsRoot は現行 invalidate 済み）。latest-check literal は対象外 — 主根拠は E1（op_logs 系 table 除外）。staleTime:0（IntegrityCheckPage.tsx:89）による実効なし判定（rally round 2 B）は補強論拠（round 3 P2-D で位置付けを整理） |
| 整合性補正 | products.stock_quantity（movement 行なし = D-051）, op_logs（integrity_fix, 同一 TX） | なし | productList.root, lowStock, stockInquiryRoot, stockMovements.root（detail の products 読み）, productForm.root, stocktake.itemsRoot（Codex round 1 P1-2 列粒度導出）。latest-check literal は対象外 — 主根拠は棚卸し確定行と同じく E1（op_logs 系 table 除外、round 3 P2-D の統一基準）。round 1 P2-1 の型不一致 no-op（fix は `integrity_fix` ログ、query は `integrity_check` フィルタ = IntegrityCheckPage.tsx:83）は E1 除外の具体例としての補強論拠（round 4 P3-1 で行間の理由粒度を統一） |
| 閾値保存（部分成功含む） | app_settings のみ（op_logs も書かない唯一の mutation） | 全成功時のみ thresholdSettings.settings, lowStock, stockInquiryRoot | 部分成功（succeededFields≥1）でも同一集合を適用（P5b-3） |
| PLU 書出し | products.plu_dirty=false + plu_exported_at, op_logs | pluDirty, productList.root | stockMovements.root, productForm.root（get_stock_detail / get_product が plu 列を読む = product_repo.rs:861 — Codex round 1 P1-2）, lowStock, stockInquiryRoot（list_low_stock_products / search 系は ProductWithRelations = Product 全列を返し plu 列を含む = product_repo.rs:897 — **2 pass P1-1 で欠落検出、gated amendment 3 で追加**） |

### 除外表（明示除外 + 理由。UI_TECH_STACK へ正本化する内容）

| 除外 | 理由 |
|---|---|
| operation_logs 系 query（operation-logs 画面の 2 literal key + integrity latest-check literal）を全 mutation で invalidate しない | ほぼ全 mutation が op_logs を書くため契約が全画面 invalidate に縮退する。操作ログは管理画面であり遷移時 fetch で十分。latest-check は staleTime:0（IntegrityCheckPage.tsx:89）のため invalidate 自体が実効なし（rally round 2 B で例外条項を削除し一律除外に単純化） |
| prefix collateral 全般 — 正当な consumer を含む prefix invalidate に巻き込まれる同 prefix 配下の child query（例: productForm.root → suppliers、inventoryRecords.root → 他 record 詳細・departments、productList.root → departments） | 書込み列を読む正当な consumer が prefix 配下に 1 つでも存在すれば prefix invalidate は正当で、collateral は許容（害は余分な refetch のみ）。leaf 単位の厳密化は全 root 再設計を要するため scope 外（rally round 2 D の productForm.suppliers 個別許容を Writer fail-closed 第 2 次裁定 2026-07-23 で一般原則へ昇格） |
| 商品 master 表示列（name / 部門 / 単位 / 価格）の JOIN stale — dailySales / monthlySales / stocktake 一覧 / 入出庫詳細 / stockMovements 等の全 consumer（Codex round 1 P1-2 で売上限定から一般化） | 集計・数量の数値は不変で JOIN 表示列のみ stale。全 consumer へ invalidate すると商品 update が事実上全 query invalidate に縮退し契約が無意味化するため、表示列 stale は staleTime 経過の自然回復を許容。数量列・状態列は本除外の対象外（列カテゴリ基準参照） |
| 閾値保存 → backupRestore.settings | 同じ app_settings を読むが backup 画面は backup 系 key しか表示せず業務影響なし |
| 新規商品 create → dailySales / monthlySales | 新規商品に売上行は存在しない |
| departments.next_seq（独自コード発番カウンタ） | 書き込むのは商品 create のみ。departments query の DTO には next_seq が含まれる（product_repo.rs:237/259 の SELECT、bindings.ts）が、現 UI はこの field を利用していないため stale の可視影響がない（Writer fail-closed 第 2 次裁定 2026-07-23 で「読む query なし」から境界を是正。UI が next_seq を利用し始める場合は本除外を再評価） |

## Non-scope

- P5-4 の literal key factory 収容（上記非目的参照。着手順序: 本 change が先、P5-4 是正は本 change の merge 後）。
- 一覧フィルタリセットボタン等の UI 変更、staleTime 再設計、refetch 戦略変更。
- backend / Rust の変更、bindings 再生成。
- E2E 追加（roadmap 1-4 で評価）。

## Acceptance Criteria

- `npx vitest run` で対象 mutation の契約遵守テストが green、かつ ProductFormPage / IntegrityCheckPage / ThresholdSettingsPage 部分成功分岐に invalidateSpy 検査が新設されている（`rg -l "invalidation-contract" src/features` が対象 feature を列挙する）。
- 契約感度の実測: M1/M2 = production の `invalidation-contract.ts` から 1 key 除去（入庫 / 整合性補正、test oracle は触らない）で独立 oracle との差分 red、M3 = 閾値部分成功への適用 guard を全成功限定へ戻して red、M4 = prefix 破壊で red（Matrix の M 行として記録 — Codex round 2 P2 で M 行と記述を一致）。
- SSOT 経由の静的 regression test（vitest 内、`src/features` と `src/lib` の全 `invalidateQueries` 呼び出しをソース走査）が、①SSOT helper 本体（invalidation-contract.ts） ②backupRestore 系（対象外 domain） ③stocktake error-path の named helper（`refreshStocktakeStateAfterConflict` 相当、helper 単位 allowlist）以外の呼び出しを 1 件でも検出したら **fail する**（rg の人手分類は fail-closed でないため静的 test へ置換 — Codex round 1 P2-2。test 名は Matrix F7 行に記載）。
- UI_TECH_STACK §2.5 に導出原則 + 除外表、decision-log に D-052、function-design 対象 doc に SSOT 参照が入り、`bash scripts/doc-consistency-check.sh` pass。
- `bash scripts/local-ci.sh full` pass（hosted final は Ready 後）。

## Design Sources

- Requirements / spec: docs/research/audit-2026-07/findings/p5-state-query.md（P5-1〜P5b-3）、p8-test-quality.md（P8-2）、adjudication.md（順 4 裁定 + Goal Invariant 指定）
- Architecture: docs/UI_TECH_STACK.md §2.5 Data Fetching（invalidation 方針の実正本 = L200-251 の invalidation pattern + 補強 6 項目。本 PR で改訂。§6 は横断 UI 要素で invalidation 無関係 — rally round 1 P1-1 で節番号誤りを是正）
- Function / command / DTO: function-design 51 / 55 / 56 / 57 / 58 / 60 / 61 / 62 / 63 / 64 / 66 / 67 / 69 / 73 / 75 + 45-cmd §45.5（invalidation 記述の同期対象は UI 15 文書 + CMD 1 文書の計 16 文書 — 列挙は Scope を正とする）
- DB: docs/DB_DESIGN.md（table→query 逆引きの根拠）、D-051（fix_integrity の書込み意味論）
- Screen / UI: 上記 function-design と同じ
- Decision log / ADR: D-051（BIZ-07-D1/D2）、D-052（本 PR 新設）

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status |
|---|---|---|
| Backend function / command / repository / validation / error | 変更なし（書込み集合は現状が正） | existing sufficient |
| Command / DTO / generated binding / wire shape | 変更なし | existing sufficient |
| DB / transaction / audit / rollback / migration | 変更なし | existing sufficient |
| Screen / UI / route state / Japanese wording | UI_TECH_STACK §2.5 + function-design UI 15 文書の invalidation 記述（列挙は Scope を正とする） | updated in this PR |
| CSV / TSV / report / import / export format | 変更なし | existing sufficient |
| Durable decision / ADR | decision-log D-052（契約 SSOT 配置 + 除外原則） | updated in this PR |

## Registration / Generation Obligations

該当なし（新規 command / doc / REQ / route / 画面なし。新設は frontend module `invalidation-contract.ts` のみで、生成系検査の対象外）。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| P5-1〜P5b-3 | UI_TECH_STACK §2.5 | D-052 | SSOT 定数方式。rejected: ①画面単位の key 追加（監査が「再発する」と明示） ②doc 側を SSOT にして表を二重管理（drift 再生産） | src/lib/invalidation-contract.ts + 16 mutation onSuccess | 各 page test の契約遵守検査 |
| P8-2 | UI_TECH_STACK §2.5 | D-052 | テスト期待は契約表 D-052-Cn から転記した独立 oracle（production SSOT 非 import、完全一致比較）。rejected: ①実装列挙の写し（現行の tautology） ②SSOT を test が import（mutant の両側が同時に縮む共有 tautology — Codex round 1 P1-1） | 各 page test | production contract のみ変更する実 mutation 感度実測（Matrix M 行） |
| P5-2 前提 | query-keys.ts ヘッダ D-4 | D-052 | stockMovements root/prefix helper 新設（既存 csvImportLists / stockInquiryRoot パターン踏襲） | src/lib/query-keys.ts | prefix 整合テスト（product/list が root 配下） |
| P5-3 | 36-biz §21 / 75-ui / D-051 | D-051 | fix_integrity は cache（stock_quantity）直接更新 = invalidation は D-051 invariant の UI 側追随 | IntegrityCheckPage.tsx | fix 成功時 invalidateSpy 検査 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: 可（監査 findings + adjudication + D-051 + 本 PR で正本化する UI_TECH_STACK §2.5 / D-052）
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: 導出原則・除外表・SSOT 配置 → UI_TECH_STACK §2.5 + D-052
- Assumptions and constraints: backend 書込み集合は現状が正（Rust 変更なし）。契約表の backend 根拠は調査記録の file:line を Ledger 作成時に転記
- Deferred design gaps, risk, and follow-up target: P5-4（literal key factory 収容）は本 change merge 後。日報 import の書込み集合は確認済み（追加なしで確定、契約表を参照）
- Test Design Matrix can cite design decision IDs or source doc sections: 可（D-052 / P5-x / P8-2）
- Absolute guarantee / escape hatch self-check completed: 「全 production invalidateQueries が SSOT 経由」の絶対保証に対し、例外 = backupRestore 系（対象外 domain）と stocktake 画面内の error-path invalidate（列 refresh 用途、契約対象外）を明示。AC の rg 検査で全ヒット分類を担保

## Impact Review Lenses

not applicable — 実地調査・実機・外部ツール・POS 連携・フォーマット変更を含まない frontend cache 契約の整備。Fact check / design decision split のみ適用: backend 書込み集合（fact、調査済み）と invalidate 粒度（design decision、除外表で明示）を分離済み。

## Design Readiness

- Existing design docs are sufficient because: 不十分 — UI_TECH_STACK §2.5 は原則のみで mutation 別契約が未定義（監査指摘の根本原因）。本 PR で正本化する
- Source docs updated in this PR: UI_TECH_STACK §2.5 / decision-log D-052 / function-design UI 15 文書 + CMD 1 文書の invalidation 記述同期（列挙は Scope を正とする）
- Design gaps intentionally deferred: P5-4、staleTime 再設計（日報 import 行は確認済み・追加なしで確定）
- Durable decisions discovered in this plan and promoted to source docs: 導出原則（書いた table を読む query は invalidate、除外は明示列挙 + 理由）

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): UI 層のみ変更。書込み集合の fact は BIZ/IO から読取り
- Backend function design: 変更なし
- Command / DTO / data contract: 変更なし
- Persistence / transaction / audit impact: 変更なし
- Operator workflow / Japanese UI wording: 文言変更なし。mutation 直後の別画面表示が最新化される（operator に見えるのは「古い数字が出なくなる」のみ）
- Error, empty, retry, and recovery behavior: onSuccess のみ変更、エラー経路は不変（stocktake の error-path invalidate は現行維持）
- Testability and traceability IDs: 契約遵守テストは P5-x / P8-2 を test 名 or comment で参照

## Contract Probe

N/A — 外部 library / OS 挙動の未検証前提なし。TanStack Query の prefix invalidation 挙動は既存 production パターン（csvImportLists / stockInquiryRoot）で実証済み。

## Contract Coverage Ledger

契約 ID は D-052 の子番号（D-052 本体は本 PR で decision-log に新設）。backend 書込み根拠: C1/C3 = product_service.rs:161-233 / 802-933（movement 挿入 196-206 / 873-887、stocktake_items 216-225）、C2 = product_service.rs:273-359、C4 = receiving.rs:135-224、C5 = returns.rs:162-260（212-237 分岐）、C6 = manual_sale.rs:202-305、C7 = disposal.rs:139-223、C8/C9 = commit.rs:92-286 / rollback.rs:15-84、C10 = daily_report_import_service/commit.rs:98-184 / rollback.rs:30、C11 = stocktake_service.rs:313-469（確定後チェック連動 455-461）、C12 = integrity_service.rs:127-213（D-051）、C13 = settings_cmd.rs:167-178 + system_repo.rs:100-108、C14 = plu_export_service.rs:185（confirm_plu_export_saved）、C15 = stocktake_service.rs:225（start_stocktake、廃番 stock=0 自動確定 257-268）、C16 = stocktake_service.rs:173-202（`update_count`、内部で repo::update_stocktake_item_count 呼出 — 関数名は round 4 P2-2 で訂正）（C14〜C16 は round 3 P2-A で追記）。

adjacent-contract sweep: UI_TECH_STACK §2.5 全項目 + 対象 function-design UI 15 文書 + CMD 1 文書の invalidation 節 + query-keys.ts ヘッダ D-4（直書き禁止）を対象済み。既存 literal key の扱いも全収載 — `["daily-sales"]` は S2 で factory 化、operation-logs 系 literal（operation-logs 画面 2 key + integrity latest-check）は P5-4 保全で非接触のまま除外表 E1 に収載（rally round 2 A で sweep 主張との不整合を是正）。

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| D-052-C1 商品 create の invalidation 集合（契約表 行 1） | ProductFormPage.tsx create onSuccess → SSOT 経由 | ProductFormPage 契約遵守 test（新設） | 非 L3（自動テスト完結） |
| D-052-C2a 商品 update の集合（行 2） | ProductFormPage.tsx update onSuccess（:133） | ProductFormPage update 実操作 test（oracle は経路単位） | 同上 |
| D-052-C2b 廃番 toggle の集合（行 2、集合は C2a と共通） | ProductFormPage.tsx toggle onSuccess（:157） | ProductFormPage 廃番化・復帰の実操作 test（**新設** — 現行 test は toggleDiscontinue を mock 宣言するのみで成功操作 test なし = Codex round 1 P1-3） | 同上 |
| D-052-C3 一括 import の集合（行 3） | useProductImportFlow.ts | ProductImportPage.test.tsx の既存 integration test 拡張（hook 単体 test は不在 — Codex round 1 P2-5 で test target を訂正） | 同上 |
| D-052-C4 入庫の集合（行 4） | ReceivingPage.tsx | ReceivingPage test 書換え + M1 | 同上 |
| D-052-C5 返品の条件付き集合（行 5、register_processed 分岐） | ReturnExchangePage.tsx | ReturnExchangePage test 書換え + F5 negative | 同上 |
| D-052-C6 手動販売の集合（行 6） | ManualSalePage.tsx | ManualSalePage test 書換え | 同上 |
| D-052-C7 廃棄の集合（行 7） | DisposalPage.tsx | DisposalPage test 書換え | 同上 |
| D-052-C8 売上 CSV commit の集合（行 8、pluDirty は過剰につき除去 — Writer fail-closed 裁定 2026-07-23） | useCsvImportFlow.ts commit | useCsvImportFlow renderHook + QueryClient wiring test（**新設** — hook 実行 test は現状不在、page test は hook を idle mock で全置換。useDailyReportImportFlow.test.tsx パターン踏襲。rally round 1 P1-2） | 同上 |
| D-052-C9 売上 CSV rollback の集合（行 9、pluDirty 除去は C8 と同一裁定） | useCsvImportFlow.ts rollback | 同上（新設 test に rollback 経路を含める） | 同上 |
| D-052-C10 日報 import の集合 = 現行維持（行 10） | useDailyReportImportFlow.ts（SSOT 経由化のみ） | useDailyReportImportFlow test（契約導出型へ） | 同上 |
| D-052-C11 棚卸し確定の集合（行 13。latest-check は round 2 B 裁定で対象外） | StocktakePage.tsx handleCompleteConfirm | StocktakePage test 拡張 | 同上 |
| D-052-C15 棚卸し開始 = 現行維持（行 11、SSOT 経由化のみ — round 2 C） | StocktakePage.tsx handleStart 成功時 | StocktakePage test（契約導出型へ） | 同上 |
| D-052-C16 棚卸し明細個数更新 = 現行維持（行 12、SSOT 経由化のみ — round 2 C） | StocktakePage.tsx onUpdated | 同上 | 同上 |
| D-052-C12 整合性補正の集合（行 14、D-051 追随。latest-check literal は P2-1 裁定で対象外） | IntegrityCheckPage.tsx handleFix | IntegrityCheckPage 契約遵守 test（新設） + M2 | 同上 |
| D-052-C13 閾値保存の集合・部分成功適用（行 15、P5b-3） | ThresholdSettingsPage.tsx saveMutation 両分岐 | 部分成功 test（新設） + M3 | 同上 |
| D-052-C14 PLU 書出しの集合（行 16、Codex round 1 P1-2 で stockMovements.root + productForm.root 拡張、**2 pass P1-1 で lowStock + stockInquiryRoot 追加 = 全 6 key**） | PluExportPage.tsx | PluExportPage test（独立 oracle 全 6 key 完全一致）+ M5/M6（新 2 key の個別除去 red 実測） | 同上 |
| D-052-E1〜E6 除外表 6 項目（E1 op_logs 系 / E2 prefix collateral 一般原則（第 2 次裁定で個別許容から昇格） / E3 商品 master 表示列 JOIN stale の一般化 / E4 backupRestore.settings / E5 新規商品売上 / E6 departments.next_seq = DTO 含有だが現 UI 未利用 — E3 一般化と E6 は Codex round 1 P1-2、E2 一般化と E6 境界是正は Writer fail-closed 第 2 次裁定） | invalidation-contract.ts 非収載 + UI_TECH_STACK §2.5 除外表 | 静的 regression test + Ledger レビュー | 非 L3 |
| D-052-S1 全 production invalidate は SSOT 経由（例外 = backupRestore 系 / stocktake error-path named helper）。**強化（2 pass P1-2/P1-3）**: oracle/test → production contract の推移的 import 到達（re-export 経由含む）を禁止、helper 第二引数は `invalidationContract.<entry>(...)` 直接呼び出しのみ許可（.concat / spread / conditional / wrapper 拒否）、computed member call も検出 | 16 mutation（18 success handler）の onSuccess 置換 + error-path helper 集約 + 静的 gate の AST/import-graph 化 | 静的 regression test（fail-closed）+ re-export / concat / computed call の fixture regression | 非 L3 |
| D-052-S2 root/prefix helper 新設 3 domain: stockMovements（product / list を prefix 配下に）+ productForm（C3 の bulk 上書きが必要とする — rally round 1 P2-2。suppliers の collateral は除外表で許容）+ dailySales（`["daily-sales"]` literal 4 箇所 = csv-import 2 + daily-report-import 2 の factory 化 — rally round 2 A、箇所数は round 3 P2-C で実測訂正） | query-keys.ts | prefix 構造検査 test（新設、3 domain） + M4 | 非 L3 |
| SPEC-INV-CONTRACT-01 テスト期待は D-052-Cn 独立転記 oracle（production SSOT 非 import、完全一致比較 — P8-2） | 対象 page test 全書換え（18 経路） | M1〜M4 の契約感度実測 | 非 L3 |

## Test Plan

Test Design Matrix: [test-matrices/2026-07-22-mutation-consumer-query-contract.md](test-matrices/2026-07-22-mutation-consumer-query-contract.md)（commit 済み）

- targeted tests: 16 mutation / 18 経路の契約遵守テスト（invalidateSpy 期待は D-052-Cn 独立転記 oracle との完全一致、production SSOT 非 import）、prefix 整合（3 domain）、閾値部分成功分岐、integrity fix 成功時、ProductFormPage create / update / 廃番 toggle（Codex round 2 P1 で toggle 経路を追記）
- negative tests: M1/M2 = production contract の key 除去、M3 = 閾値部分成功 guard 退行、M4 = prefix 破壊で red（Codex round 3 P2 で M 行定義と統一）。返品 register_processed=true で在庫系 invalidate が発火しないこと
- compatibility checks: 既存 test suite green 維持（invalidate 追加による副作用なし）、doc-consistency-check pass
- data safety checks: N/A（データ書込みなし）
- main wiring/integration checks: local-ci full + hosted final

## Boundary / Wire Contract

N/A — JSON API / CSV / DTO / bindings / DB スキーマ変更なし。query key は frontend 内部契約。

## Review Focus

- 契約表の各セルが backend 書込み集合から正しく導出されているか（frontend 現行実装からの逆算になっていないか）
- D-052-E1〜E6 の除外表の妥当性（特に op_logs 除外の境界と E3 一般化の許容範囲）
- 独立 test oracle（D-052-Cn 転記）方式の運用整合 — production SSOT を test が import しない禁止が実装で守られているか、oracle 転記 drift の残余リスク管理（Ledger row 突合 + M1/M2）が機能する構造か（Codex round 1 P1-1 / round 2 P1 で共有 SSOT 方式を全廃）
- stockMovements.root 新設が既存 key shape（product/list）を prefix 配下に収められるか

## Spec Contract

Contract ID: SPEC-INV-CONTRACT-01

- 全 production mutation の成功時 invalidation は `invalidation-contract.ts` の SSOT 集合に一致し、SSOT は「mutation が確定した table.column を query result が読むなら invalidate、除外は UI_TECH_STACK §2.5 除外表 E1〜E6 に明示」の導出原則に従う。テスト期待は契約表 D-052-Cn から転記した独立 oracle（production SSOT 非 import、完全一致比較）で、production contract の key 除去・追加・重複に感度を持つ。

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-INV-CONTRACT-01 | SSOT 新設 + 16 mutation 置換 + テスト書換え | Matrix 参照 | 契約表導出の正しさ | Matrix M 行の red 実測記録 |

## Data Safety

- 実店舗データの commit なし（テストは既存 synthetic fixture のみ）
- local-only paths: なし
- synthetic-only paths: 既存 test fixture のみ

## Implementation Results

（件数・exact-HEAD SHA は PR body が authority — D-035/D-038 Evidence Ownership）

- 全 16 mutation / 18 success handler の invalidation を `invalidation-contract.ts`（D-052 SSOT）経由へ統一し、監査 findings P5-1〜P5b-3 の欠落全件と、契約化過程で発見した過剰（CSV の pluDirty）・網羅漏れ（棚卸し開始/明細更新）・C14 拡張を解消。prefix helper 3 domain 新設、stocktake error-path の named helper 集約、literal `["daily-sales"]` の factory 化
- テストは契約表 D-052-Cn 独立転記 oracle との完全一致比較へ全面書換え。静的 gate は AST/import-graph 検査（transitive re-export 遮断・helper 引数変形拒否・computed call 検出 + survivor 3 形の fixture regression）。M1〜M8 の production-only mutation 感度を clean tree で実測（記録は PR body）
- 設計正本: UI_TECH_STACK §2.5 へ導出原則（table.column 粒度・列カテゴリ基準・過剰禁止・prefix collateral 許容）+ 除外表 E1〜E6 を正本化、decision-log D-052 新設、function-design 15 doc + 45-cmd を SSOT 参照へ同期
- PR: https://github.com/higekiri-w/inventory-system-public/pull/21（squash merge `187d7a7`、2026-07-23）

## Review Response

- Findings Freeze: **frozen after Broad Audit**（Double Audit 両 pass + 2 pass findings 全是正の closure 確認完了、2026-07-23）; post-freeze exceptions: none.

### 2 pass closure 確認（2026-07-23、Coordinator 差分確証）

- fix commit `1cf1bbd`（e72ccec からの実装差分は宣言 5 file のみ）を実読確証: C14 = production SSOT / 独立 oracle とも全 6 key 一致、静的 gate = transitive re-export 検査 + AST による helper 引数 / computed call / alias 経路検査 + survivor 3 形の fixture regression（Writer が closure review で bound / destructured / assigned alias・逆順 re-export・computed destructuring まで拡張）
- M5〜M8 all red 実測（Writer 報告、clean tree、復元済み）+ local-ci full PASS + PR #21 body 更新を確認
- 2 pass の P1×3 / P2×2 は全件閉塞。P1/P2 = 0 成立 → human-confirm 遷移（Reviewed Content HEAD = 1cf1bbd を遷移 commit で設定）
- 遷移圧縮記録（本 content commit 同乗）: implementing → local-verified（evidence = 1cf1bbd 上の local-ci full CLEAN、証跡 = PR body）→ local-verified → independent-review（evidence = Double Audit 両 pass 実施済み + 本 closure 確認）

### Codex plan review round 1（2026-07-22、P1×3 / P2×5 / P3×1 → P2-3 のみ第 2 案採用の部分 accept、他は全 accept。全 P1/P2 の cite を Coordinator が実読 CONFIRMED）

- P1-1（accept）: M1/M2 が共有 SSOT tautology で自壊 — production と test が同じ contract を読むと mutant の両側が同時に縮み green survivor になる。test oracle を契約表 D-052-Cn からの独立転記に変更し、完全一致比較（欠落・余分・重複 red）へ。M1/M2 は production contract のみ変更して oracle との差で red を実測
- P1-2（accept）: 契約表を table.column 粒度へ再導出（版名は付けない — Codex round 2 P3 の再発防止案採用）。列カテゴリ基準を新設（数量列・状態列は invalidate、master 表示列 JOIN stale は E3 一般化で許容）。在庫系 8 行に productForm.root + stocktake.itemsRoot 追加、PLU 行を集合拡張（plu_dirty / plu_exported_at を get_stock_detail が読む）、書込み列補完（departments.next_seq = E6 / csv_import_errors / csv_imports.status）
- P1-3（accept）: 契約 16 行 / 18 success handler を明記、Ledger C2 を C2a（update）/ C2b（toggle、実操作 test 新設）へ分割
- P2-1（accept）: doc-consistency ERROR 3 を是正（Matrix 参照の link 化 / Findings Freeze 位置 / Plans.md active packet link）
- P2-2（accept）: AC を静的 regression test の fail-closed gate へ変更、stocktake error-path を named helper に集約し helper 単位 allowlist
- P2-3（部分 accept = 第 2 案採用）: operation-log 3 key の factory 収容は P5-4 是正単位との二重管理になるため不採用。D-4 ヘッダへ期限付き明示例外 + follow-up 先を追記し、sweep 主張を整合させる。採否は相互修正案方式で Codex へ返す
- P2-4（accept）: 58-ui-stock-inquiry（§309 の具体列挙が本 change の直接対象）+ 45-cmd §45.5 を同期対象へ追加（15 doc）
- P2-5（accept）: C3 の test target を ProductImportPage.test.tsx 既存 integration test 拡張へ訂正（hook 単体 test は不在）
- P3-1（accept）: Review Focus の除外表参照を「D-052-E1〜E6」の可変件数非依存表記へ

### Codex plan review round 2（2026-07-22、②③④ PASS / ① FAIL。P1×1 / P2×2 / P3×1 → 全 accept）

- 確認結果: 独立 oracle の設計と M1/M2 再定義は妥当（②契約表の実体 PASS = round 1 全反例が実コード突合で閉塞確認、③P2-3 第 2 案裁定に異議なし、④doc-consistency exit 0）
- P1（accept）: oracle 独立化が「新文言の追加」に留まり、旧共有-SSOT 指示が Matrix Contracts Under Test / Ledger Trace 行 / Test Plan / Review Focus の 4 箇所に残存 — Writer がどちらでも読める矛盾。Codex 提示の統一文面で全箇所置換し、Test Plan に toggle 経路を追記
- P2-1（accept）: Matrix 個別行（整合性補正 / 閾値 / 棚卸し確定 / CSV）が旧部分集合を「SSOT 集合」と呼んでいた → D-052-Cn 全量列挙の独立 oracle 完全一致表記へ統一
- P2-2（accept）: AC の「代表 3 mutation の任意 1 key 除去」が M3（guard 退行）と不一致 → M1〜M4 の実定義どおりに書き分け
- P3（accept、再発防止案採用）: 契約表・除外表の版名を廃止（「v1」表記の一括除去、Review Response の「v2」も「列粒度へ再導出」へ）— 版番号の転記 drift を構造的に防止

### Writer fail-closed 裁定（2026-07-23、implementing 中の gated amendment）

- Writer（Codex）が実装中に C8/C9 の pluDirty() 収載と table.column 導出原則の衝突を検出し fail-closed 停止（commit / M1〜M4 / PR 未実施のまま報告 — 正しい挙動）。Coordinator 実証 = CSV commit/rollback / inventory_repo の production 経路に plu_dirty 書込みなし（sales_repo の 2 ヒットは `#[cfg(test)]` fixture）
- 裁定 = **オプション 1 採用（C8/C9 から pluDirty 除去）**。E7 で過剰を許容する案は不採用 — 除外表は「書くのに invalidate しない」の正当化であり、「書かないのに invalidate する」の正当化を混ぜると契約の判別力が壊れる。導出原則に逆方向（過剰 invalidate も契約違反）を明文化し、現行実装の過剰は契約化時に除去する規律とした
- operator 影響なし: plu_dirty が変わらない以上 PLU 通知 badge の値も変わらず、除去による可視退行はない
- Writer 検出の閾値 survivor（成功 0 件の不発火 negative test 欠如、`> 0 → >= 0` 生存）も accept、Matrix に行を追加

### Double Audit 1 pass（2026-07-23、Fable inline 契約突合、Reviewed Content HEAD = e72ccec、**blocker 0**）

- Ledger 全行を実装と再突合: C1〜C16 = SSOT ↔ 独立 oracle ↔ packet 契約表の 3 方向突合で全エントリ一致（pluDirty 除去 / 返品 registerProcessed 分岐 / toggle の C2 共通集合適用を含む）。S1 = 静的 gate 3 層（allowlist 外の直接呼び出し拒否 / 任意 key 配列の抜け道拒否 / **oracle の production 非 import を機械強制**）を実読確認。S2 = prefix helper 3 domain + D-4 期限付き例外注記（query-keys.ts:8-9）。E1〜E6 = SSOT に除外対象の混入なし、E2 一般化・E6 境界是正が UI_TECH_STACK §2.5 L237/L244 と D-052（decision-log:392-397）へ同期済み
- 18 success handler の wiring を rg 全数確認（全て `invalidateByContract(queryClient, invalidationContract.*)` 形式）
- oracle の完全一致比較実装（sort + JSON stringify）は欠落・余分・重複すべて red になる構造を確認
- 判定: P1/P2 = 0。次 = 2 pass（Codex 独立 fresh context、系統的 mutation testing + survivor 探索）
- **事後注記**: 2 pass が本 pass の見落とし 3 件（C14 の lowStock/stockInquiryRoot 欠落 = SELECT 列の突合不足、静的 gate の re-export / AST 抜け道 = mutant 実測なしの構造確認に留まった限界）を検出。1 pass 単独の blocker 0 判定は覆された — Double Audit の必要性が本 change でも実証（5 change 連続）

### Double Audit 2 pass（2026-07-23、Codex 独立 fresh context、**P1×3 / P2×2 → 全 accept、merge blocker**）

- P1-1（accept、CONFIRMED）: C14 に lowStock / stockInquiryRoot 欠落 — list_low_stock_products は ProductWithRelations（Product 全列 = plu 列含む）を返す（product_repo.rs:897、Coordinator 実読）。gated amendment 3 で契約表 / Ledger を是正、SSOT / oracle / PLU test / M5・M6 実測は Writer 差し戻し
- P1-2（accept、mutant 実測済み survivor）: 静的 gate の oracle 独立検査が transitive re-export を追跡せず、re-export 経由で oracle を production に接続 + key 除去が full green — Goal Invariant が禁止した共有 tautology の再現経路。import graph の推移的到達検査へ強化
- P1-3（accept、mutant 実測済み survivor ×2）: helper 第二引数の `.concat` 変形と computed member call（`["invalidate","Queries"].join("")`）が静的 gate を素通り。AST 検査 + fixture regression へ強化
- P2-1（accept）: Reviewed Content HEAD の設定が早い（正 = P1/P2=0 確定後の human-confirm 遷移 commit、DEV_WORKFLOW:101。順 3 で検出された同型違反の Coordinator 再発）→ state-backtrack 相当の差し戻しで pending へ復帰
- P2-2（accept）: 遷移記録の test 件数を除去（Evidence Ownership、件数は PR body のみ）、非 canonical subject の未 push state commit 2 件を soft reset し本 content commit へ圧縮記録
- survivor 判定: 許容 survivor 0（3 件すべて実バグ相当 → 全て修正対象）

### Writer fail-closed 第 2 次裁定（2026-07-23、implementing 中の gated amendment 2）

- Writer が第 1 次 amendment（e1ed3d4）の「書かない column しか読まない query への invalidate は契約違反」の字義適用が prefix 機構と衝突することを検出し再停止（入庫の inventoryRecords.root は returnDetail 等を巻き込む — 現行の正当な prefix invalidate が軒並み違反になる）。原則文の精度不足は Coordinator 側の起因
- 裁定 = Writer 推奨文面を accept: **「prefix 配下に書込み列を読む正当な consumer が 1 つでも存在すれば prefix invalidate は正当、child collateral は許容」**。過剰の定義を「正当 consumer がゼロの invalidate」に精緻化（pluDirty 除去の裁定は正当 consumer ゼロなのでこの定義でも維持される）。E2 を個別許容から一般原則へ昇格。leaf 単位の厳密化は scope 外
- E6 境界是正 accept（CONFIRMED: departments SELECT/DTO に next_seq 含有 = product_repo.rs:237/259、現 UI 未利用。「読む query なし」は不正確だった）
- traceability FAIL（新規 test file の REQ/UI ID 付与 + 再生成）は正本裁定後の Writer 残作業として想定内

### Codex plan review round 3（2026-07-22、反映確認 4 点全 PASS + 新規 P2×1 / P3×1 → 全 accept。**P1 blocker なし**）

- 反映確認: 旧共有-SSOT 指示 4 箇所の置換 / Matrix 個別行の全量化 / AC と M1〜M4 の整合 / 版名廃止 — すべて PASS（旧方式語句の rg 残存 0 件、doc-consistency exit 0 を Codex 側でも実測）
- P2（accept）: Test Plan negative tests だけ旧 M3 解釈（「契約 key 除去で red（代表 3 mutation）」）が残存 → M1/M2 = key 除去、M3 = guard 退行、M4 = prefix 破壊の実定義どおりに修正
- P3（accept）: 版名機械除去による空白 drift（「契約表 の」3 箇所 +「契約表 参照」1 箇所）を整形
- relay 超過 3/2 は Owner Effort Budget 節に記録（owner 承認済み）

**Plan Gate 収束**: self rally 5 round + Codex plan review 3 round。Codex round 3 verdict = P1 blocker なし、P2 は本 round で反映済み。

### Plan Gate rally round 1（2026-07-22、独立 Plan Reviewer、P1×2 / P2×4 / P3×2 → 全 accept）

- P1-1（accept、CONFIRMED）: invalidation 正本の節番号誤り（§6 → §2.5）。全参照を §2.5 へ是正。Coordinator の実証 = UI_TECH_STACK 見出し構造（§2.5 Data Fetching L137 / §6 横断 UI 要素 L474）+ query-keys.ts:4 ヘッダの「§2.5 補強 1」
- P1-2（accept、CONFIRMED）: useCsvImportFlow の hook 実行 test 不在（page test は idle mock 全置換 = P8b-1 と同一構造）。C8/C9 を renderHook wiring test **新設**へ訂正、Scope に明記。Owner Effort Budget は owner 介入予算であり Writer 工数ではないため据え置き（Codex 予算は無制約運用）
- P2-1（accept、CONFIRMED）: 行 12 の latest-check literal は no-op（fix は `integrity_fix` ログ、query は `integrity_check` フィルタ = IntegrityCheckPage.tsx:83）→ 削除。行 11（棚卸し確定）は `run_integrity_check` 連動のため維持
- P2-2（accept、修正方向は削除でなく補完）: 一括 import の products UPDATE を `productForm.product(id)` が読む以上、導出原則から productForm root helper 新設 + 行 3 追加が正しい帰結。Scope の dangling 文を確定文へ置換、Ledger S2 拡張
- P2-3（accept、CONFIRMED）: 日報行の条件文を確定表記へ（monthlySales は official_department_totals で daily_report を読む = sales_service.rs:246-249、monthlySalesRoot 維持）
- P2-4（accept）: AC の rg 分類に stocktake error-path を第 3 バケットとして明記
- P3-1（accept、保持 + 理由明記）: 行 1 の stockMovements.root は実効 no-op だが導出原則の一様性を優先して収載、理由を契約表に記載
- P3-2（accept）: doc 同期対象へ 56（日報サマリ表示）/ 67（PLU 書出し）を追加（13 doc）

### Plan Gate rally round 2（2026-07-22、独立 Plan Reviewer、P2×4 / P3×1 → 全 accept。round 1 反映自体の矛盾なしを確認）

- A（部分 accept）: `["daily-sales"]` literal 3 箇所は dailySales root helper 新設で factory 化（SSOT が直接参照する key の D-4 整合）。latest-check の factory 化は rebut — Non-scope の P5-4 保全を維持（B 裁定で SSOT 参照自体が消えるため実質 moot）。Ledger sweep 完全性主張の文言も是正
- B（accept、CONFIRMED）: latest-check query は staleTime:0（IntegrityCheckPage.tsx:89）のため invalidate は実効なし。round 1 P2-1 の no-op 判定基準を対称適用し、行 13（棚卸し確定）からも削除、除外表 E1 の例外条項を撤去して一律除外に単純化
- C（accept、修正方向は文言拡張でなくバケット純化 + 契約表拡張）: StocktakePage:156-157 は棚卸し開始の success-path で error-path ではない（Coordinator 実証）。棚卸し開始・明細個数更新を「現行維持・SSOT 経由化のみ」で契約表へ収載（行 11/12、C15/C16）し、バケット③を真の error-path 防御 refresh のみに純化
- D（accept）: productForm.root の suppliers collateral invalidate を除外表に許容として明記
- E（accept）: 57-ui-monthly-sales を doc 同期対象へ追加（14 doc、56 と対称の consumer doc）

### Plan Gate rally round 3（2026-07-22、独立 Plan Reviewer、P2×4 / P3×1 → 全 accept。前 round 裁定への異議なし、R3 必須節充足を確認）

- P2-A（accept）: Ledger backend 根拠列挙に C14/C15/C16 の file:line を追記（plu_export_service.rs:185 / stocktake_service.rs:225 / 173-202）
- P2-B（accept）: 除外表の round 2 D 追加分（productForm.suppliers collateral）を E2 として採番し、Ledger を E1〜E5 の 5 項目へ更新
- P2-C（accept、CONFIRMED）: `["daily-sales"]` literal は 4 箇所（csv-import 2 + daily-report-import 2、Coordinator 実測）。「3 箇所」は round 2 反映時の Coordinator 誤カウントで、Scope / Ledger S2 を訂正
- P2-D（accept、修正方向は「除外の対称適用」でなく「除外判断基準の明文化 + 収載維持」）: staleTime 値を除外根拠にすると Non-scope で凍結した staleTime 設定への隠れ結合が生まれる。除外は table ベースの E1〜E5 のみを根拠とし、latest-check の staleTime:0（round 2 B）は E1 の補強論拠へ格下げ。stocktake.itemsRoot（staleTime:0）は導出原則どおり行 1/3 に収載維持、対称適用の基準を Scope に明文化
- P3-E（accept）: Non-scope の latest-check 条件文（round 1 P2-1 で前提消滅した死文）を無条件の記述へ簡略化

### Plan Gate rally round 4（2026-07-22、独立 Plan Reviewer、P2×2 / P3×1 → 全 accept。round 1〜3 の個別裁定への異議なし）

- P2-1（accept）: round 2 C の 16 行化が未伝播だった旧数値 7 箇所（Risk / Design Intent Trace / Ledger S1 / Test Plan / Trace Matrix / Matrix 2 箇所）を「16 mutation」「契約表 16 行」に統一
- P2-2（accept、CONFIRMED）: Ledger C16 の関数名を訂正 — biz 層 pub fn は `update_count`（stocktake_service.rs:173）、`update_stocktake_item_count` は repo 層（stocktake_repo.rs:243）
- P3-1（accept）: 整合性補正行の latest-check 除外理由を round 3 P2-D の統一基準（主根拠 = E1、型不一致 no-op は補強論拠）へ揃え、行間の理由粒度差を解消

### Plan Gate rally round 5（2026-07-22、独立 Plan Reviewer、P2×1 → accept。**表記修正のみで収束**）

- P2（accept）: doc 同期対象数「11 doc」が Required Design Artifacts / Design Readiness の 2 箇所で未伝播（round 1 P3-2 / round 2 E の doc 追加の drift、round 4 P2-1 と同型）→「14 doc（列挙は Scope を正とする）」へ統一し、以後の個数 drift を正本参照で防止
- round 4 反映の検証: 16 mutation 統一 = rg 機械確認でヒット 0 / C16 関数名 = 実コード突合一致 / 除外理由粒度 = 統一済み。前 4 round の全裁定に異議なし、実体的誤導出・矛盾・抜け道の最終確認も該当なし

**rally 収束宣言**: round 5 で新規実体指摘 0（表記 1 件のみ、同 round で反映済み）。self rally は 5 round で収束。次 = Codex plan review（Plan Reviewer 欄の定義どおり、結果は上記 Codex round 1 節）。
