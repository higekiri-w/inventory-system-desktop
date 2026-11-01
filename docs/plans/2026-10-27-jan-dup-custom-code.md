# Plan Packet: 新規登録で、入れた JAN をほかの商品が使っているとき独自コードで登録できる道

wave に属さない単独の lane（2026-10-27 に同時に起票した 4 本の 1 本。同じ日の関門 8 の lane〈branch `agent/product-csv-preview-check`〉も 30 と商品の BIZ を触る）。`docs/backlog.md` の項目「新規登録で、入れた JAN が既に他の商品にあるとき独自コードで登録できる道」（owner の決定、repo 外の回答台帳 TD-349）を起票する。本 commit は Plan Packet と Test Design Matrix の起票と、plan-first の正本の修正だけで、実装は plan-approved の後の別の run。branch は `agent/jan-dup-custom-code`（main `50f97a76` から）。新しい決定は D-141（予約の番号）。

規模と手法（起票の最初に数えた）: 実装で触る code は Rust 3 file（`src-tauri/src/biz/product_service.rs` の `create_product` と test、`src-tauri/src/biz/mod.rs` の `BizError`、`src-tauri/src/cmd/mod.rs` の変換と test）、TS 8 file（`ProductFormPage.tsx`・新しい `components/SharedJanCustomCodeDialog.tsx`・`lib/product-form-request.ts`・`src/lib/invalidation-contract.ts`・`src/test/invalidation-oracle.ts` と test 3 file）、生成物 2（`src/lib/bindings.ts`・`docs/function-design/90-traceability.md`）。正本は plan-first で 7 file を直した（下の「本 commit で変えた正本」。round 1 の是正で `docs/db-design/master-tables.md` を足して 8 file）。command の数・route・migration・IO の関数は 0。wire は `ProductCreateRequest` に bool を 1 つ足すだけ（`CmdErrorKind` の値は変えない）。手法: 1 lane のまま（BIZ の知らせと送り直しの flag、CMD の変換、画面のダイアログは 1 つの契約〈D-141〉の両端で、分けるとどちらかが使い手の無い契約か、送る先の無い画面になる）。

## Workflow State

- Phase: implementing
- Risk: R3
- Plan Commit: 0925c9ecd712b51f3dea5a28009ebbc01ee8ecbb
- Amendments: none
- Coordinator: Opus 5.5 main session
- Writer: 起票は Opus 5.5 subagent（本 run の起草役）。実装の Writer は座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）の Writer 行から Coordinator が発注時に決める
- Plan Reviewer: fresh Opus 5.5 + Codex（座組表どおり）
- Final Reviewer: Fable 5（Claude 側。座組表の「R3 以上と design lane で Fable 5」、当面 Fable 5）+ Codex（座組表どおり）
- Final Review Minimum: 1
- Human Gate: ready,merge,manual
- Branch: agent/jan-dup-custom-code

遷移の記録:

1. kickoff → spec-check（2026-10-27、起草役）: 発注書の範囲（D4 との関係、画面の振舞い、商品 CSV の経路との一貫性、並走との重なり）を範囲にし、Risk を R3 と記録した（下の Risk）。
2. spec-check → design（2026-10-27、起草役）: 30 §4.2 の手順 3a（JAN を商品コードにし、商品コードの重複で `DuplicateProductCode`）と 51 の商品フォームには、JAN をほかの商品が使っているときの知らせと独自コードの道が無い。30・40・51・SCREEN_DESIGN・UI_TECH_STACK・decision-log を直す必要がある。
3. design → plan-draft（2026-10-27、起草役、同じ commit）: 設計の出力（30 BIZ-01-D11、51 UI-01b-D26、40 の変換、UI_TECH_STACK の invalidation の entry、D-141）は本 commit に置いた。未解決の設計の問いは無い（owner の判断事項 2 問は owner の方針〈repo 外の回答台帳 TD-401〉で推奨を既定にした。下の「owner の判断事項」）。
4. plan-draft のまま是正（round 1、2026-10-29、Coordinator の裁定）: 計画の review round 1（fresh Opus 5.5・Codex、どちらも request changes）の findings を全件受理した Coordinator の裁定どおりに、packet・Matrix・設計正本（30・51・SCREEN_DESIGN・master-tables・decision-log の D-141・backlog）を直した。判定・件数・採否は下の Review Response。Phase と Plan Commit は変えない。
5. plan-draft → plan-approved（2026-10-29、Coordinator）: 計画の review の closure（round 2、`0925c9ec`）は fresh Opus 5.5・Codex とも approve（P1/P2 0、P3 だけ。PR #212 の comment「計画の review の closure（round 2）」）。`scripts/plan-approve.sh` で Plan Commit を `0925c9ec` に固定した（`3c1974af`）。
6. plan-approved → implementing（2026-10-29、Coordinator）: 実装の Writer（Opus 5.5 subagent、発注 678）が `948bb671`〜`2e9f5b76` で実装した。旧形式の packet は Phase が承認の前の値の間は承認の前と扱われる（`docs/DEV_WORKFLOW.md` の「承認の後の計画の変更」）ので、Phase の行を本 commit で進めた（`plan-approve.sh` は Phase を書かない）。D-052 の道標の行は、固定の Scope の文（D-141 が accepted になったら足す）に合わせて実装の commit から戻した。

plan-gate へ進める前に Coordinator がすること: (1) main（`ee5b221a`、PR #202 の merge を含む。起票の起点 `50f97a76` の後に入った）を単段で取り込む（`git -c merge.directoryRenames=false merge origin/main`）。予想する衝突は `docs/decision-log.md` の末尾（#202 の D-138 と本 lane の D-141。番号の順に両方を残す）だけ。51・SCREEN_DESIGN・backlog・UI_TECH_STACK の #202 の hunk は本 lane の hunk と別の行（下の節の所有表）。(2) 関門 8 の lane の予定 path と節を取り直し、下の所有表を直す（同時の起票なので本 run からは見えない）。(3) 30 の局所の決定の ID: 関門 8 の lane（PR #209）が `BIZ-01-D10` を使うので、本 lane は `BIZ-01-D11` にした（2026-10-27、Coordinator。済み）。

## Owner Effort Budget

- 介入回数上限: 6（既定）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 6 | 0 | 5（owner の判断事項 Q1〜Q3 を見る〈3 問を 1 回で見せるが decision point は 3 なので 3〉、L3 1、Ready と merge 1〈train 委任なら Ready・merge で 1〉） | 1 | 6 = 0 + 5 + 1 |

実働の見込み: L3 は約 5 分（下の Test Plan の L3。fixture は要らない）。Q1〜Q3 を見るのは約 4 分（見込み、未実測）。

承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の R3「Tauri command DTO ... operator workflow ... changes」に当たる: `ProductCreateRequest`（command `create_product` の入力の DTO）に field を足し、bindings を再生成する。商品の登録の operator workflow（保存の後に確認のダイアログを挟む）を変える。登録の TX で、ほかの商品の在庫連動（`products.pos_stock_sync`）を true → false に変える（ADR SPEC-STK-CNT-D4・D8 の守りの内）。R4 にしない理由: 削除・backup・restore・migration・店の実データに触れない。在庫連動を外すのは記録も確認も要らない変更（30 の冒頭の節の「在庫連動を始める」の項）で、戻せる（群の JAN を直せば在庫連動にできる）。R3 の要件: Contract Ledger・Data Safety・Contract Probe・Test Design Matrix・Design Readiness を置いた。Final Review Minimum は 1、Human Gate は `ready,merge,manual`（画面の変更なので manual。`docs/DEV_WORKFLOW.md` Human Visual Confirmation）。

required gate: 本 commit は docs だけで、どの required gate の green / red も変えない見込み（docs の job と aggregate だけが走る。下の AC の実測）。実装の PR は Rust と TS を変えるので、changed 分類で rust・frontend の job も走る（gate の定義は変えない）。

## Goal

Goal Invariant:

### 最小完了条件

- 商品フォームの新規登録で、ほかの商品が使っている JAN（同じ JAN の商品か、商品コードがその JAN の商品）を入れて「登録する」を押すと、その画面で、使っている商品を全て名指したダイアログが開く。
- ダイアログで「独自コードで登録する」を押すと、その場で新しい商品が独自コードで登録され、入れた JAN を持つ。同じ JAN の在庫連動の商品は同じ TX で在庫連動なしになり、外した商品が成功の知らせと操作ログに名指しで残る。
- JAN がほかの商品に無いときは、今どおり JAN が商品コードになる（TD-338）。

### 失敗定義

- JAN をほかの商品が使っているのに、黙って JAN を商品コードにした商品ができる、または黙って独自コードで登録される。
- 登録の後に、在庫連動の商品と同じ JAN を持つ別の商品がある（ADR SPEC-STK-CNT-D4 の不変条件が破れる）。
- 登録が途中で失敗したのに、ほかの商品の在庫連動か部門の `next_seq` が変わったまま残る。
- 利用者が「戻る」を押したのに送られる、または入力が消える。
- 2 色目を入れるのに、登録の画面を離れる（商品 CSV・既存の商品の修正へ行く）必要がある。

### 非目的

- 商品 CSV の取込みの経路（30 §4.8・§4.9、60）を変える（関門 8 の lane の範囲。共有 JAN の行の error は今どおり）。
- D4 の不変条件・Z004 の売上の割り当て（群の先頭の商品）・店内コードを変える。
- 商品フォームの edit で JAN を変えられるようにする（UI-01b-D5 のまま）。
- 同じ JAN の群の売価・税率の違いを登録の画面で知らせる（Q2 の推奨。PLU の画面の要修正のまま）。
- 既存の商品を開く導線（商品コードの link）をダイアログに置く（文で案内する）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 布の商品 A（JAN J、在庫連動）が登録済み | 商品登録で 2 色目の名前・独自コードの部門・価格を入れ、JAN 欄に J を読ませて「登録する」 | ダイアログ「この JAN はほかの商品が使っています」が開き、A を「（POS販売で在庫を減らす）」付きで名指し、独自コードの道と、この商品と JAN が同じ商品の「POS販売で在庫を減らす」が「減らさない」になることを示す。入力は残る | 「独自コードで登録する」を押す | なし |
| 上のダイアログ | 「独自コードで登録する」 | 2 色目が独自コード（例 `HZ-0012`）で登録され、JAN J を持つ。toast に商品コードと「同じ JAN の商品 A「…」の「POS販売で在庫を減らす」を「減らさない」にしました」。一覧へ戻り、A の修正の画面を開くと「POS販売で在庫を減らす」が「減らさない」になっている。A の商品コードは J のまま（TD-352 からの例外、Q3） | — | なし |
| A と 2 色目（どちらも在庫連動なし）が J を持つ | 3 色目を同じ手順で登録 | ダイアログが A と 2 色目を名指す。登録すると 3 色目が独自コードで入り、外す商品は無い（toast は商品コードだけ） | — | なし |
| 選んだ部門に独自コードの接頭辞が無い | J を入れて「登録する」 | ダイアログは主の button を出さず「部門「…」は独自コードを振れません。独自コードを振れる部門を選び直してから、もう一度「登録する」を押してください。」 | 「戻る」→ 部門を選び直して「登録する」→ 上の 1 行目（主の button 付きのダイアログ）→ 2 行目 | なし |
| 廃番にした同じ品を、知らずに登録し直そうとした | J を入れて「登録する」 | ダイアログが A を「（廃番）」付きで名指し、同じ品なら商品一覧で JAN を検索してその商品を直す（「表示に戻す」）よう示す | 「戻る」→ 商品一覧で J を検索 → A を開いて「表示に戻す」 | なし |
| J をどの商品も使っていない | J を入れて「登録する」 | 今どおり商品コード J で登録される（ダイアログは出ない） | — | なし |
| 2 色目の登録の後、レジで J が売れた | 翌日 Z004 を取り込む | 売上は今どおり群の先頭の商品（商品コードの昇順）に記録し、どの商品の在庫も動かさない（ADR D4）。在庫は棚卸しと手入力で合わせる | — | 今の規則（`docs/db-design/pos-tables.md` の Z004 のマスタ照合、backlog の DATA-2） |

この文書を完了できること（起票）と、通常運用を達成できること（実装と L3 の後）は別。本 commit の後も、2 色目は実装の merge まで商品 CSV でしか入れられない。

## Scope

決めたこと（発注書の「決めること」）:

- D4 との関係: D4 の不変条件は変えない（新しい商品も既存の商品も、同じ JAN の群の中では在庫連動にできない）。独自コードの道では、新しい商品を在庫連動なしで登録し、同じ JAN の在庫連動の商品を同じ TX で在庫連動なしにする（ダイアログで名指して同意を得てから。Q1 の採用案〈推奨〉を既定にした）。登録の画面での知らせ方は確認のダイアログ（51 UI-01b-D26）。
- 画面の振舞い: 重複を知らせたダイアログで、その場で独自コードの登録へ進める（D-127 の「その場で直せる」）。部門に接頭辞が無いときは原因（部門）と直し方を名指す。同じ品を登録し直す誤りには、直し方（商品一覧で JAN を検索して直す）を名指す。
- 商品 CSV の取込みの経路との一貫性: どちらの経路も同じ不変条件（D4）で、同じ JAN を各色に持たせる形（TD-352）。ただし商品フォームの道では、先に JAN を商品コードとして登録した 1 色目は商品コードを変えられないので J のまま群に残る（TD-352 の「全色を独自コード」からの例外。30 BIZ-01-D11 の例外の項・D-141 の Guarantee range、Q3。既存の商品の再採番は足さない）。CSV は商品コードを file が持ち、在庫連動にできない行は今どおり行の error で名指す（file を直す経路なので、ほかの商品の在庫連動を自動で外さない）。CSV の経路は変えない（非目的）。
- 新しい決定: D-141（本 commit で decision-log に置いた）。

本 commit で変えた正本（plan-first）:

- `docs/function-design/30-biz-product-service.md`: §4.2 の `ProductCreateRequest`（`shared_jan_custom_code: bool`）・`ProductCreateResult` の `warnings` の意味・手順 3a・3a2・3b・9・エラーハンドリング、設計判断 BIZ-01-D11 の新設、§4.10 の `JanInUse`。冒頭の「計数の置き場と在庫の求め直しの契約」の共有 JAN の項の field を `pos_stock_sync` だけにした（JAN の側の field `jan_code` で返す経路が無くなるため）。
- `docs/function-design/40-cmd-product.md`: `BizError::JanInUse` の変換（kind `duplicate`・field `jan_code`）、冒頭の StocktakeRecovery の項の field を `pos_stock_sync` だけにした。
- `docs/function-design/51-ui-product-form.md`: UI-01b-D26 の新設、冒頭の共有 JAN の項、UI-01b-D25 の理由の「商品コードの重複で止まる」、§7.2・§7.5 Create 5・§7.6・§7.8・§7.9。
- `docs/SCREEN_DESIGN.md`: 商品登録・修正画面のレイアウト判断に 1 項。
- `docs/UI_TECH_STACK.md`: 冒頭の段落（写しと要再確認の一覧の invalidation）に `productCreateSharedJan` の entry。
- `docs/decision-log.md`: D-141（末尾）。Compatibility に D-052 の C23 = `productCreateSharedJan` と件数（mutation entry 22 → 23）を置いた（round 1 の是正。D-052 は merge 済みで、decision-log の冒頭の規則〈本文は append-only、後から足せるのは Status と Superseded の行だけ〉により D-052 の Contract 行と Decision の件数は書き換えない。件数の正本の側は実装で直す。下の実装の Scope）。
- `docs/backlog.md`: 項目「新規登録で、入れた JAN が既に他の商品にあるとき独自コードで登録できる道」に起票の行。round 1 の是正で、保留の見た目・UX に候補「商品フォームの共有 JAN の拒否の文を画面の語にするか」（出所: PR #212 の計画の review round 1、Opus の E の気づき）。
- `docs/db-design/master-tables.md`（round 1 の是正）: 困りそうなケースの「CSV取込み時にJANコードで複数商品がヒットする場合」に、TD-352 からの例外（1 色目の商品コードが JAN のまま残る）の 1 文。
- round 1 の是正で、上の 30・51・SCREEN_DESIGN の文も直した: `JanInUse` の message の印と step 3a2 の warnings を商品フォームの画面の語（欄の名「POS販売で在庫を減らす」と値「減らす」「減らさない」）にし、印「商品コードだけ同じ」を足した（30 BIZ-01-D11 の message の形・文の語の項、51 UI-01b-D26、SCREEN_DESIGN の 1 項）。BIZ-01-D11 に TD-352 からの例外の項を足し、「TD-352 と同じ形」を弱めた。51 UI-01b-D25 の理由を初回の知らせと確認の後の登録に分けた。51 の冒頭の `recount_required` の項（#210）は正本として変えていない。

実装（plan-approved の後）:

- `src-tauri/src/biz/mod.rs`: `BizError::JanInUse(String)` と `Display`（「JAN 使用中: {message}」の形。ほかの variant と同じ並び）。
- `src-tauri/src/biz/product_service.rs`:
  - `ProductCreateRequest` に `pub shared_jan_custom_code: bool`（doc comment に BIZ-01-D11）。
  - `create_product` の手順 3（30 §4.2 の 3a・3a2・3b）: 3a は `product_repo::find_by_jan_code` と `find_by_product_code` で使っている商品を集め（商品コードの昇順、同じ商品は 1 つ）、message を BIZ-01-D11 の形で作って `JanInUse` を返す。3a2 は `generate_custom_code` の後、`find_by_jan_code` の在庫連動の商品ごとに `product_repo::update_product`（`pos_stock_sync: Some(false)`）・`recompute_stock`・`system_repo::insert_operation_log`（`product_update`、BIZ-01-D7 の detail を `serde_json` で組む）を行い warnings を足す。手順 9 で warnings を返す。
  - 手順 3b の field の選び分け（今の `if req.pos_stock_sync { "pos_stock_sync" } else { "jan_code" }`）を `pos_stock_sync` だけにする（下の「この変更で不要になるもの」）。
  - test（`mod tests`、REQ-101 を付ける。名前は実装で決める）: Matrix の B1〜B10 を足す。直す test（理由は旧い契約〈JAN を商品コードにして共有 JAN の組を作る〉が BIZ-01-D11 で置き換わったため）: `test_create_product_req101_duplicate_jan`（期待を `DuplicateProductCode` から `JanInUse` へ）、`test_create_product_req101_rejects_shared_jan_with_stock_linked`（1 つ目と 3 つ目の create は `JanInUse` になるので、D4 の拒否の case を 3a2 の経路〈Matrix の B5〉へ移す。field `jan_code` の assert は外す）。ほかに `cargo test` で red になる create の test があれば、旧い契約に依るものだけを同じ理由で直し、PR に列挙する（D4 のほかの経路の test は変えない）。
  - `ProductCreateRequest` の literal: `rg -n 'ProductCreateRequest \{' src-tauri` の 11 行（struct の定義 1、`default_create_request` の signature の行 1 とその本体 1、test の literal 8。2026-10-27 に確かめた）のうち、`..default_create_request()` を使わない所に `shared_jan_custom_code: false` を足す。
- `src-tauri/src/cmd/mod.rs`: `From<BizError> for CmdError` に `JanInUse(message)` → `{ kind: Duplicate, message, field: Some("jan_code") }`。test（REQ-905 の命名に合わせる）に変換の 1 本（Matrix の C1）。既存の `test_cmd_error_req905_from_duplicate`（`rg -n 'fn test_cmd_error_req905_from_duplicate' src-tauri/src/cmd/mod.rs` → 309 行、2026-10-29。今は kind と message しか見ない）に `assert!(cmd_err.field.is_none())` を足す（UI は field `jan_code` の有無で 2 つを分けるので、`DuplicateProductCode` の側の field が `None` であることも producer の契約。test を強める変更で、弱めない）。
- 生成物: `src/lib/bindings.ts`（`cd src-tauri && cargo run --bin generate_bindings`。差分は `ProductCreateRequest` の `shared_jan_custom_code: boolean` の 1 行の見込み）、`docs/function-design/90-traceability.md`（`cd src-tauri && cargo run --bin generate_traceability`。手で直さない）。
- `src/features/products/lib/product-form-request.ts`: `buildCreateProductRequest` の request に `shared_jan_custom_code: false`。
- `src/features/products/components/SharedJanCustomCodeDialog.tsx`（新設）: shadcn AlertDialog（`DiscontinueConfirmDialog.tsx` と同じ部品）。props は open・BIZ の message・選んでいる部門（名前と `code_prefix` の有無）・pending・`onConfirm`・`onCancel`。文と button は 51 UI-01b-D26 の文字どおり。
- `src/features/products/ProductFormPage.tsx`: `createMutation` の `onError` で `isInvokeError(error) && error.cmdError.kind === "duplicate" && error.cmdError.field === "jan_code"` のときダイアログを開く（message を解析しない。recount_required の分け方と同じ）。送り直しは別の mutation（または同じ mutation の引数）で `shared_jan_custom_code: true`・`pos_stock_sync: false`、成功の toast の説明に `result.warnings`、invalidation は `invalidationContract.productCreateSharedJan()`。送り直しの失敗はダイアログを閉じて `setSaveError(describeError(error))`。
- `src/lib/invalidation-contract.ts`: entry `productCreateSharedJan`（`productCreate` の 7 key + `queryKeys.productForm.root()` + `queryKeys.stocktake.recheckList()`）。`src/test/invalidation-oracle.ts` に同じ entry を UI_TECH_STACK の段落から独立に写し、`src/lib/invalidation-contract.meta.test.ts` の key の一覧と entry の比較に足す。D-052 の C23（D-141 の Compatibility）として件数を直す: `invalidation-contract.ts` の doc comment「D-052-C1〜C22」→「C1〜C23」、meta.test の it の名前「all 22 mutation entries」→「all 23」と `toHaveLength(22)` → `toHaveLength(23)`（REQ-907 B-I1。数える対象が 1 つ増えるので直すのが正しく、test を弱める変更でない）、`docs/UI_TECH_STACK.md` の D-052 の件数の行（今は「22 mutation entry / 25 success handler 呼出し（2026-08-25 実測）」「D-052-C1〜C22」）を実装の後の実測（entry は meta.test の 23、handler は `rg -c "invalidateByContract\(" src/features --glob '!**/*.test.*'` の合計。2026-10-29 の起点で既に 26 で、行の 25 と食い違っている。送り直しの成功の handler が 1 つ増える見込み、未実測）と日付で書き直し、C23 を足す。D-141 が accepted になったら D-052 に `- Superseded in part by: D-141（件数と C23）` の行を足す（decision-log の冒頭の規則で足せる行。D-141 が proposed の間は足さない）。
- TS の test: `ProductFormPage.test.tsx`（Matrix の U2〜U7）、`product-form-request.test.ts`（U1）。
- 新しい表示文言と test の matcher の衝突: `rg -F '<語>' src --glob '*.test.*'` を `独自コードで登録`・`ほかの商品が使っています`・`在庫連動を外しました`・`在庫連動なしになります`・`JAN を検索`・`独自コードを振れません` で実行し、全て 0 件（2026-10-27。うち `在庫連動を外しました`・`在庫連動なしになります` は起票の時の語で、round 1 の是正で下の画面の語に置き換えた）。`戻る` は exact の `getByRole("button", { name: "戻る" })` が `ProductImportPage.test.tsx` にあるが、商品フォームの test は `一覧へ戻る` の exact だけで、同じ画面に `戻る` の button は無い（衝突しない）。round 1 の是正で文を画面の語にした後の語（2026-10-29）: `商品コードだけ同じ`・`にしました`・`JAN が同じ商品` は 0 件。`POS販売で在庫を減らす` は 4 file 10 行にあるが、全て `getByLabelText`／`findByLabelText`（label の要素だけを見る）か、`recount_required` の案内の `name: "「POS販売で在庫を減らす」にできません"`（role の名前の完全一致）で、ダイアログの文の段落に当たらない。`減らさない` は `ProductFormPage.test.tsx` の `recount_required` の案内の文の完全一致 1 行だけ（衝突しない）。

型・JSON・関数の形を変える所（作る所・読む所・test の fixture・test の呼び方、`rg` で全件。2026-10-27）:

| 対象 | 作る所 | 読む所 | test の fixture / 呼び方 |
|---|---|---|---|
| `ProductCreateRequest.shared_jan_custom_code` | `src/features/products/lib/product-form-request.ts`（`buildCreateProductRequest`）、`ProductFormPage.tsx`（送り直し） | `src-tauri/src/biz/product_service.rs` の `create_product`（`src-tauri/src/cmd/product_cmd.rs` の `create_product` は素通し） | Rust: `rg -n 'ProductCreateRequest \{' src-tauri` の 11 行（Scope の内訳）。TS: `product-form-request.test.ts`（`toMatchObject` なので既存は壊れない）、`ProductFormPage.test.tsx` の `mockCreateProduct` の `toHaveBeenCalledWith(expect.objectContaining(...))` |
| `ProductCreateResult.warnings`（形は変えず、中身を使い始める） | `create_product` の手順 9 | `ProductFormPage.tsx` の成功の toast（今は読んでいない） | `ProductFormPage.test.tsx` の `mockCreateProduct` の resolve 値（今は `warnings: []`） |
| `BizError::JanInUse` | `create_product` の手順 3a | `src-tauri/src/cmd/mod.rs` の変換、`biz/mod.rs` の `Display` | `cmd/mod.rs` の test、`product_service.rs` の test |
| `CmdError`（kind `duplicate`・field `jan_code` の組を新しく使う。型は変えない） | `cmd/mod.rs` | `ProductFormPage.tsx` の `onError` | `ProductFormPage.test.tsx` の reject 値 |
| `invalidationContract.productCreateSharedJan` | `src/lib/invalidation-contract.ts` | `ProductFormPage.tsx` | `src/test/invalidation-oracle.ts`、`invalidation-contract.meta.test.ts` |

他の lane との所有（`docs/DEV_WORKFLOW.md` Wave Operation。open の PR #172・#190・#193・#194・#196・#197・#201・#204・#207 の差分〈`gh pr diff N --name-only`〉と #193 の packet の「予定 path の一覧」（62）を、本 lane の予定 path（22）と `LC_ALL=C comm -12` で突き合わせた。2026-10-27。#202 は 2026-10-26 に merge 済みで、起点 `50f97a76` から `ee5b221a` までの差分〈25 path〉と比べた）:

| file | 節 | 所有 |
|---|---|---|
| `docs/function-design/30-biz-product-service.md` | §4.2 の `ProductCreateRequest`・`ProductCreateResult`・手順 3〜3b・9・エラーハンドリング・BIZ-01-D11、§4.10 の enum の 1 行、冒頭の共有 JAN の項の field の句 | 本 lane。関門 8 の lane は §4.8・§4.9 と §4.2 の BIZ-01-D1 の段落（手順 1 と 2 の間。発注 597 の読むもの）。重なりは plan-gate の前に Coordinator が取り直す |
| `src-tauri/src/biz/product_service.rs` | `ProductCreateRequest`・`create_product`・create の test（`test_create_product_req101_*` の並び） | 本 lane。関門 8 の lane は `preview_import`・`commit_import` とその test。共有の `shared_jan_conflict` は本 lane も関門 8 も変えない見込み |
| `src-tauri/src/biz/mod.rs` | `BizError` の variant と `Display` の arm | #172・#197 も variant を足す（同じ enum の末尾近く）。先に merge した方の版を起点に後の方が同期で並べる（本 lane は後） |
| `src-tauri/src/cmd/mod.rs` | `From<BizError> for CmdError` の arm と test | #172・#197 も arm を足す。同上 |
| `src/lib/bindings.ts` | 生成物（`ProductCreateRequest`） | #172・#197・#193 も再生成する。衝突は手で解かず、取り込んだ後に再生成する（D-098） |
| `docs/function-design/90-traceability.md` | 生成物 | 同上 |
| `docs/function-design/40-cmd-product.md` | BizError → CmdError の変換の一覧の `DuplicateProductCode` の次の 1 行、冒頭の StocktakeRecovery の項の field の句 | #197 は CmdError の struct と変換の一覧の別の行（`@@ -47`・`@@ -66`）。行が隣り合えば同期で両方を残す。#193（packet）は 40 の商品の結果の列の節 |
| `docs/function-design/51-ui-product-form.md` | 冒頭の共有 JAN の項、UI-01b-D25 の理由の句、UI-01b-D26 の行（D25 の次）、§7.2・§7.5 Create 5・§7.6・§7.8・§7.9 | #202（merge 済み）は 16 行目・25 行目（UI-01b-D2）。#193（packet）は冒頭の在庫数の表示の項（b2）。別の行 |
| `docs/SCREEN_DESIGN.md` | 商品登録・修正画面のレイアウト判断の 1 項（create mode の商品コードの項の次） | #202（merge 済み）は同じ節の 349 行目（main の版）の hunk で、本 lane の行と 3 行離れる。#172・#197 は別の画面の節。#193（packet）は在庫の表示の節 |
| `docs/UI_TECH_STACK.md` | 冒頭の段落（写しと要再確認の一覧の invalidation）の 1 文。実装で D-052 の件数の行（§2.5） | #202（merge 済み）は 235・456 行目。ほかの open な PR の差分に無い |
| `docs/db-design/master-tables.md` | 困りそうなケースの共有 JAN の項の 1 文（round 1 の是正） | 本 lane（`gh pr list --state open --json number,files` で open な PR の差分に無い。2026-10-29） |
| `src/features/products/ProductFormPage.tsx`・`ProductFormPage.test.tsx`・`lib/product-form-request.ts`・`lib/product-form-request.test.ts` | create の mutation・ダイアログの state・`buildCreateProductRequest` と create の test | #193（b2、実装中）は edit の在庫数の表示と `productToFormValues`。#193 を先に merge し、本 lane が同期で当て直す |
| `docs/decision-log.md` | D-141（末尾） | #194・#201 も末尾に足す。#202（merge 済み）は D-138。末尾への追記は merge の順で両方を残す |
| `docs/backlog.md` | 項目「新規登録で、入れた JAN が既に…」とその下の起票の行 | #172・#190・#196・#201 は別の項目。#202（merge 済み）は直前の項目の下に 5 行を足した（間に別の項目が 1 つある） |
| `src/lib/invalidation-contract.ts`・`src/test/invalidation-oracle.ts`・`src/lib/invalidation-contract.meta.test.ts`・`SharedJanCustomCodeDialog.tsx`・本 packet・Matrix | 全体 | 本 lane（他の PR の差分と #193 の予定 path に無い） |

merge の順（既定案）: #193（b2、導入の前の関門 2、実装中）→ #197・#172（BizError の variant を先に足す）→ 本 lane。関門 8 の lane とは、先に merge した方の版を起点に後の方が同期する（30 と `product_service.rs` は別の節・関数）。L3 は本 lane が train の先頭に来て `origin/main` を取り込んだ head で頼む（`docs/DEV_WORKFLOW.md` Wave Operation、D-124）。

この変更で不要になる記述・条件・code（D-127 の問い）:

- 30 §4.2 のエラーハンドリングの「JAN の重複で起きる」（`DuplicateProductCode` は INSERT の一意制約に当たったときだけになる）。本 commit で直した。
- 共有 JAN の拒否の field `jan_code`（30 の冒頭の共有 JAN の項、40 の冒頭、51 の冒頭）。新規登録で JAN をほかの商品が使っていれば手順 3a が先に止め、update は在庫連動を始める保存だけを検査するので、field `jan_code` で返す経路が無くなる。本 commit で記述を直し、実装で `create_product` の field の選び分けの `else { "jan_code" }` を消す（Q1 が (c) なら残す。下の Q1）。
- 51 UI-01b-D25 の「同じ JAN の create は商品コードの重複で止まる」。本 commit で直した。
- 2 色目を商品 CSV でしか入れられない運用（backlog の項目の前提）。実装の merge の後の closeout で項目を閉じる。
- 残すもの: `DuplicateProductCode`（INSERT の一意制約と 30 §4.3 の手順 7 の安全の検査が使う）、`create_product` の手順 3b（3a2 の経路で新しい商品を在庫連動にして送ったときの守り。UI は送らないが、command は UI の外からも呼べる）。

## owner の判断事項（推奨を既定にした所）

owner の方針（repo 外の回答台帳 TD-401）で、owner に質問せずに推奨を既定にして書き進めた。答えが推奨と違えば、plan-gate の前なら packet を直し、後なら Gated Amendment にする。

選択肢の記号は D-141 の Alternatives と 30 BIZ-01-D11 の却下の記号に揃える（round 1 の是正。D-141・30・backlog の 3 か所が同じ記号を使っていたので、packet の側を直した）。Q2・Q3 の選択肢は D-141 の Alternatives に無いので、記号でなく名前で呼ぶ。

- Q1（D4 との関係。正本〈ADR SPEC-STK-CNT-D4〉は「新しい商品も既存の商品も群の中では在庫連動にできない」までを決め、登録が既存の商品の在庫連動を変えてよいかは決めていない）: 2 色目を独自コードで登録するとき、既存の同じ JAN の在庫連動の商品をどうするか。
  - 採用案〔推奨・既定。D-141 の Decision (2)〕ダイアログで、この商品と JAN が同じ商品の「POS販売で在庫を減らす」が「減らさない」になることを名指し、「独自コードで登録する」で登録と同じ TX で外す（操作ログと toast で名指す）。理由: 登録の画面を離れずに済む（D-127）。true → false は記録も確認も要らない変更。
  - (c) 外さずに拒む。ダイアログで在庫連動の商品を名指し、その商品の修正で在庫連動を外してから登録し直すよう案内する。
  - (d) D4 を緩め、既存の商品の在庫連動を残す（ADR の改訂が要り、店内コードの lane の Revisit。レジの売上で既存の商品の在庫だけが減り、黙ってずれる）。推奨しない。
- Q2（同じ JAN の群で売価・税率が違うと、PLU の書出しで群が要修正になる〈33 の同一 JAN の判定〉）: 登録の画面で知らせるか。
  - 知らせない〔推奨・既定〕: 今どおり PLU の画面の要修正で理由つきで出る。理由: 判定を 33 と登録の 2 か所に持つと食い違う。
  - 知らせる: 登録の後の toast で知らせる（`warnings` に 1 行）。
- Q3（TD-352 からの例外。round 1 の是正で足した）: 群の 1 色目を先に商品フォームで登録すると商品コードが JAN（J）になり（TD-338）、2 色目以降を本 lane の道で独自コードで加えても、1 色目は商品コードを変えられないので J のまま群に残る。TD-352 の「群は全色を独自コード」からの例外を受け入れるか。
  - 受け入れる〔推奨・既定〕: 1 色目は J のまま。理由: 商品コードは主キーで修正の画面でも読取専用（UI-01b-D5）。Z004 の照合は `jan_code` だけで行い売上は群の先頭に記録し在庫を動かさない（ADR D4）ので、商品コードが J のままでも在庫は黙ってずれない。既存の商品の再採番は本 lane に足さない。
  - 受け入れない: 1 色目も独自コードにする道（商品コードの振り直し。主キーの全参照の書換えで、SP-102-04 の「変更不可」の改訂が要る）を別の lane で起票する。本 lane はそれまで今の形で進めるか、1 色目が J の群への追加を止めるかを owner が決める。

出所: Q1 は発注書の「決めること」（D4 との関係）。Q2 は起草役（AI）が起票で見つけた候補。Q3 は AI の review（PR #212 の計画の review round 1、Opus の P2・Codex の F2）から出た問い。Q2・Q3 は owner の確認の前に Human Gate の必須にしない（AGENTS.md の Working Rules）。

答えで変わる所（推奨と違う答えのとき。round 1 の是正で `rg` で取り直した。2026-10-29）:

| 答え | 正本 | packet | Matrix |
|---|---|---|---|
| Q1 (c) 外さずに拒む | 30: 冒頭の共有 JAN の項の field の句（「field `jan_code` で返す経路は無い」）、`ProductCreateRequest.shared_jan_custom_code` の意味、`ProductCreateResult.warnings` の意味、手順 3a2（外さない）・3b（field `jan_code` で拒む）・9・エラーハンドリングの 3b の行、BIZ-01-D11 の本文・文の語の項（warnings の文）・在庫連動を外す理由・却下 (c)。40: 冒頭の StocktakeRecovery の項の field の句、`JanInUse` の変換の行の説明。51: 冒頭の共有 JAN の項（field と UI-01b-D26 の括弧）、UI-01b-D26（在庫連動の商品があるとき主の button を出さず直し方を出す、2 段目の文、送り直し、warnings の toast）、§7.5 Create 5（`pos_stock_sync: false`）・§7.6・§7.9。SCREEN_DESIGN の商品登録・修正画面の 1 項（「減らさない」になる）。UI_TECH_STACK の冒頭の段落の `productCreateSharedJan` の文（要らない。`productCreate` を使う）。decision-log D-141 の Decision (2)・Why・Compatibility の C23 | Goal の最小完了条件の 2 つ目、Ordinary Operation の 1・2 行目、Scope の決めたこと（D4 との関係）、実装の Scope の手順 3a2・3b の field の選び分け・invalidation の entry と C23 の件数・meta.test の件数、この変更で不要になるもの（field `jan_code` を残す）、型・JSON の表の `warnings`・`invalidationContract.productCreateSharedJan` の行、Contract Audit、Contract Ledger の BIZ-01-D11 (2)・手順 3b・失敗で全てを戻す・UI-01b-D26・D-052 の invalidation の行、AC7（`productCreateSharedJan` の数）、Boundary / Wire Contract の round-trip と invalid input、L3-2 | B4・B5・B8・B10・B-old2・U3・U7・I1、M2・M3・M7・M9・M10、State Lifecycle の「ダイアログの送り直し」「群の商品の在庫連動（DB）」の行 |
| Q1 (d) D4 を緩める | ADR SPEC-STK-CNT-D4 の改訂（本 lane の外、店内コードの lane の Revisit）。上の Q1 (c) の正本の全部に加えて、30 冒頭の共有 JAN の項の不変条件 | packet の作り直し（Goal の失敗定義の 2 つ目が変わる） | 作り直し |
| Q2 知らせる | 30: `ProductCreateResult.warnings` の意味（「step 3a2 で…それ以外は空」）、手順 3a2 に 33 の同一 JAN の判定を足す、BIZ-01-D11 の文の語の項（warnings の文）。decision-log D-141 の Guarantee range（「登録の画面では知らせない」）。51 UI-01b-D26 の toast の説明 | Goal の非目的の 4 つ目（売価・税率の違いを知らせる）、Design Readiness の前提・延期した gap、Contract Ledger の「33 の同一 JAN の PLU の判定」の行（非対象 → 対象）、Contract Audit の種類の混在 | B 行と U3 に売価・税率の違う case を足す、Mutation に判定を省く注入を足す |
| Q3 受け入れない・今の形のまま（振り直しの lane を別に起票し、本 lane はそれまで今の形で進める） | 30 BIZ-01-D11 の TD-352 からの例外の項、decision-log D-141 の Context・Why・Guarantee range・Revisit、master-tables の困りそうなケースの例外の文。振り直しの lane を起票するなら backlog に項目 | Scope の決めたこと（商品 CSV の経路との一貫性）、Ordinary Operation の 2 行目（A の商品コードは J のまま）、Impact Review Lenses の Fact check | 振り直しの lane が持つ（本 lane の Matrix は変えない見込み） |
| Q3 受け入れない・1 色目が J の群への追加を止める（振り直しの lane まで、商品コードが J の商品がある群に商品フォームで色を加えない） | 上の行の正本の全部に加えて、30 §4.2 の手順 3a（商品コードが J の商品があるときの知らせの扱い）・3a2（商品コードが J の商品がある群では独自コードで登録しない）、51 UI-01b-D26 の主の button（商品コードが J の商品があるときは `独自コードで登録する` を出さない）と直し方の文（商品 CSV か振り直しの lane へ案内する） | 上の行の packet の全部に加えて、Ordinary Operation の 1・2 行目（1 色目が J の群ではダイアログに主の button が無く、2 色目を登録できない） | B4（1 色目が J の群で flag true が拒まれる case に変わる）・U2（主の button の有無）・U3（送り直しの case を 1 色目が J でない群に替える） |

## Non-scope

- 上の Goal の「非目的」の全部。
- `docs/Plans.md`（D-097。lane の行を足さない）。
- migration・IO の関数（`product_repo` の既存の `find_by_jan_code`・`find_by_product_code`・`update_product` を使う）・新しい command・route。
- 操作ログの画面の表示（74。`product_update` の detail は BIZ-01-D7 の形のまま）。

## Acceptance Criteria

- AC1: `bash scripts/doc-consistency-check.sh` が exit 0（ERROR なし）。
- AC2: `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-10-27-jan-dup-custom-code.md` が exit 0。
- AC3（正本の旧い文）: `rg -n "JAN の重複で起きる|field .pos_stock_sync. か .jan_code|在庫連動を自動で off にして再送しない|同じ JAN の create は商品コードの重複で止まる" docs/function-design docs/SCREEN_DESIGN.md docs/decision-log.md` が exit 1（0 件）。baseline（起点 `50f97a76`）: `git grep -n -E '<同じ式>' 50f97a76 -- docs/function-design docs/SCREEN_DESIGN.md docs/decision-log.md` で 5 行（30 の 8・123 行、40 の 8 行、51 の 7・31 行）。
- AC4（BIZ の field）: `rg -c "pub shared_jan_custom_code: bool" src-tauri/src/biz/product_service.rs` が `1`。baseline: exit 1。
- AC5（生成物）: `rg -c "shared_jan_custom_code: boolean" src/lib/bindings.ts` が `1`。`cd src-tauri && cargo run --bin generate_bindings` の後の `git diff --exit-code src/lib/bindings.ts` が exit 0（commit 済みの版と一致）。baseline: exit 1。
- AC6（CMD）: `rg -c "BizError::JanInUse" src-tauri/src/cmd/mod.rs` が 1 以上。baseline: exit 1。
- AC7（UI と invalidation）: `rg -c "SharedJanCustomCodeDialog" src/features/products/ProductFormPage.tsx` が 1 以上、`rg -c "productCreateSharedJan" src/lib/invalidation-contract.ts src/test/invalidation-oracle.ts` が各 `1`。baseline: どちらも exit 1。`rg -c "toHaveLength\(23\)" src/lib/invalidation-contract.meta.test.ts` が `1`（D-052 の C23。round 1 の是正で足した。baseline: `toHaveLength(22)` が 1 行〈84 行目、2026-10-29〉）。
- AC8（Rust の test）: `cd src-tauri && cargo test --lib product_service` と `cd src-tauri && cargo test --lib cmd::` が全 PASS で、Matrix の B1〜B10・C1 が含まれる（名前と本数は PR の runner の出力で示す）。baseline: 未実測（本 run は cargo を回さない。発注書の条件）。
- AC9（frontend）: `npm run typecheck && npm run lint && npm run format:check && npm test && npm run build` が exit 0 で、Matrix の U1〜U7・I1 が含まれる。baseline: 未実測（本 run は npm を回さない）。
- AC10（mutation）: Matrix の Mutation-style の M1〜M11 を production の file（`src-tauri/src/biz/product_service.rs`・`src-tauri/src/cmd/mod.rs`・`src/features/products/ProductFormPage.tsx`）に 1 つずつ注入し、挙げた test が `cargo test --lib product_service` / `cargo test --lib cmd::` / `npx vitest run src/features/products` で FAIL（exit 非 0）になる（実注入、戻した後に PASS。結果は PR に書く）。
- AC11（生成物）: `cd src-tauri && cargo run --bin generate_traceability -- --check` が exit 0。
- AC12: `cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings` が exit 0。
- AC13: `bash scripts/check-workflow-git.sh` が exit 0。
- AC14（Writer の完了条件、L3 の前）: `cd src-tauri && cargo check --release` が exit 0（`docs/DEV_WORKFLOW.md` Implementation Rules）。
- AC15（L3）: 下の Test Plan の L3-1〜L3-3 が owner の `PASS`（`/products/new` で観測。helper の manual の record に残す）。

実測（本 commit の前、2026-10-27。逐語で実行した。AC4〜AC15 の完了時は実装の後）:

```text
AC3  baseline: git grep -n -E '<AC3 の式>' 50f97a76 -- docs/function-design docs/SCREEN_DESIGN.md docs/decision-log.md → 5 行 / 本 commit の正本の修正の後: rg exit=1
AC4  rg -c "pub shared_jan_custom_code: bool" src-tauri/src/biz/product_service.rs → exit=1
AC5  rg -c "shared_jan_custom_code: boolean" src/lib/bindings.ts → exit=1
AC6  rg -c "BizError::JanInUse" src-tauri/src/cmd/mod.rs → exit=1
AC7  rg -c "SharedJanCustomCodeDialog" src/features/products/ProductFormPage.tsx → exit=1 / rg -c "productCreateSharedJan" src/lib/invalidation-contract.ts src/test/invalidation-oracle.ts → exit=1
```

AC1・AC2・AC13 の結果と `bash scripts/ci/classify-changes.sh --base 50f97a76 --head HEAD` の出力は、本 commit の後に測って PR の本文に書く（packet に SHA と数を写さない。D-038）。

## Design Readiness

- 引く設計正本（節まで）: 30 §4.2（手順 1〜9、BIZ-01-D1）・§4.3（BIZ-01-D6）・§4.4（BIZ-01-D7、在庫連動の切替の求め直し）・§4.10・冒頭の「計数の置き場と在庫の求め直しの契約」（共有 JAN、在庫連動を始める〈D8〉）、ADR SPEC-STK-CNT-D2 の表の「在庫連動の切替」の行・D4（共有 JAN の段）・D8、40 の BizError → CmdError の変換・CmdErrorKind の enum 契約（D-061）、51 冒頭の D-118 の節・UI-01b-D4・D5・D6・D13・D14・D15・D16〜D18・D25・§7.5・§7.6、UI_TECH_STACK の冒頭の段落と D-052（§2.5）、master-tables の `products.jan_code`（UNIQUE を付けない理由、グループコード）、D-028（JAN を使っていない行）、33 の同一 JAN の判定（PLU）、repo 外の回答台帳 TD-338・TD-349・TD-352・TD-401。
- 必要な設計成果物（Design artifact selection）: 「New or changed BIZ ... error ...」→ 30（updated in this PR、BIZ-01-D11）、「New or changed Tauri command, DTO, generated binding ...」→ 30・40 と下の Boundary / Wire Contract（updated in this PR）、「New or changed operator workflow ... Japanese wording」→ SCREEN_DESIGN・51（updated in this PR、UI-01b-D26）、UI_TECH_STACK（updated in this PR: invalidation の entry。部品は既存の AlertDialog で、技術の選定は existing sufficient）、「Durable cross-cutting choice」→ decision-log D-141（updated in this PR）。「New or changed table, column ...」は当たらない（DB の列・制約は変えない。master-tables は existing sufficient: `jan_code` は UNIQUE を付けず、共有 JAN を DB に置ける）。
- plan にしかない durable な判断の昇格先: 無い（全て 30 BIZ-01-D11・51 UI-01b-D26・D-141 に置いた）。owner の判断事項の選択肢は D-141 の Alternatives にも置いた。
- 前提・制約と延期した gap: Q1〜Q3 は推奨を既定にした（上）。同じ JAN の群の売価・税率の違いは PLU の画面で知らせる（Q2 の「知らせない」、延期しても安全な理由: PLU の書出しは群を要修正にして出さず、黙って誤った売価をレジに入れない）。
- 絶対保証の自己点検: 「登録の後に D4 が破れない」は、create の手順 3b が変更の後の状態で検査するので、3a2 の外し忘れや UI の外からの呼出し（在庫連動のまま flag を送る）でも拒む。「失敗したら何も変わらない」は、3a2 の外しと next_seq の引上げを同じ TX に置き、RAII で戻す（Matrix の B5・B6・B8）。「JAN をほかの商品が使っていれば必ず知らせる」は、商品コードと JAN の 2 つを見るが、JAN の別名（D-122、`product_jan_aliases`）は見ない（別名は `jan_code` でなく、51 UI-01b-D25 のまま。D-141 の Compatibility）。同時の登録は単一の利用者・単一の接続（`AppState` の mutex）で起きない見込みで、起きても INSERT の一意制約が `DuplicateProductCode` で止める。
- 判定: ready（Q1〜Q3 は推奨を既定にし、答えで変わる所を owner の判断事項の表に名指した）。

## Contract Audit（新しく足す契約の文の境界の一巡）

| 観点 | 契約の文（30 BIZ-01-D11・手順 3a/3a2/3b、51 UI-01b-D26） | 確かめたこと |
|---|---|---|
| 安全な整数の範囲 | 「独自コードを振る（§4.3）」 | 新しい数を足さない。独自コードの番号の i64 の上限は §4.3 の手順 3c・3d の今の検査（`この部門の独自コードの番号を振れません`）が 3a2 でも効き、TX を戻す。wire の新しい field は bool で、JS の数の範囲に触れない |
| 0 の表示 | 「外す商品は 0、warnings は空」「件数で省かない」 | JAN をほかの商品が使っていない（0 件）なら `JanInUse` を返さず JAN を商品コードにする（3a）。3a2 で外す商品が 0 なら warnings は空で、toast は説明を出さない（空の行を出さない。Matrix の B7・U3 の空の case）。名指しの一覧が 0 件で出ることは無い（0 件なら知らせない） |
| 失敗の時点と順序 | 「何も書かない」「3a2 で外した在庫連動と next_seq の引上げも戻る」 | 順: 入力の検証（TX の外、JAN の形 BIZ-01-D1）→ TX → 3a の知らせ（何も書く前）→ 3a2 の発番・外し・求め直し・ログ → 3b の D4 → INSERT → 初期在庫 → 棚卸し → ログ → COMMIT。3a2 より後の失敗（3b・INSERT・初期在庫・DB）は全て RAII で戻る（Matrix の B5・B8）。接頭辞の無い部門は §4.3 の手順 2 で発番の前に止まる（Matrix の B6）。UI はダイアログで先に止めるが、BIZ でも止まる。送り直しの失敗はダイアログを閉じ、今の保存の error の出し方で出す（U7） |
| 単位や種類の混在 | 「JAN が同じ商品か、商品コードがその JAN の商品」「印（商品コードだけ同じ・POS販売で在庫を減らす・廃番）」 | 商品コードが J で JAN が別の値の商品（商品 CSV で入りうる）は、知らせの一覧に入るが、JAN が違うので 3a2 で外さない（D4 の群でない。Matrix の B2）。廃番の商品も群に入り、D4 が数えるので外す（`find_by_jan_code` は廃番を含む）。一覧の印は 3 種（商品コードだけ同じ・POS販売で在庫を減らす・廃番）を決まった順に「・」でつなぎ、当たらない商品には括弧を付けない（組合せは 1 つの形で書く。round 1 の是正で「商品コードだけ同じ」を足し、外れない商品を区別した）。新しい商品の数量単位・売価が群と違っても登録は止めない（PLU の要修正は Q2） |

round 1 の是正で足した契約の文（30 BIZ-01-D11 の印「商品コードだけ同じ」・文の語・TD-352 からの例外、51 UI-01b-D26 の 2 段目・D25 の理由、Boundary の invalid input の限定、D-141 の C23）の一巡:

- 安全な整数の範囲: 新しい数は件数の 22 → 23 だけで、test の literal と doc の数（JS の数の範囲に触れない）。印と文は文字列だけ。
- 0 の表示: 印が 1 つも当たらない商品は括弧を付けない（空の「（）」を出さない）。商品コードだけが同じ商品が 0 なら 2 段目の「「商品コードだけ同じ」の商品は変わりません」は当てはまる商品の無い文になるが、固定の文で害は無い（1 段目に印が無いことで読める）。外す商品が 0 なら warnings は空で toast の説明を出さない（今のまま）。
- 失敗の時点と順序: 足した文はどれも知らせと結果の文で、失敗の順序（上の表）を変えない。TD-352 の例外は何も書かない（再採番しない）。C23 の件数は実装の PR で code と同時に直す（片方だけ直すと meta.test が red）。
- 単位や種類の混在: 商品コードと JAN の両方が J の商品（B4 の A）は JAN が J なので「商品コードだけ同じ」を付けず、外す。商品コードが J で JAN が別の値の商品（B2 の X）だけに付け、外さない。印の順（商品コードだけ同じ → POS販売で在庫を減らす → 廃番）は 1 つに決め、B1 の完全一致で確かめる。

## Registration / Generation Obligations

| 変更対象 | 義務 | 本 lane |
|---|---|---|
| Tauri command（frontend から呼ぶ） | specta 登録・属性・`generate_bindings` | 新しい command は無い。既存の `create_product` の DTO を変えるので `cd src-tauri && cargo run --bin generate_bindings` で `src/lib/bindings.ts` を再生成する（AC5） |
| REQ / coverage の追加・変更 | `generate_traceability` で `docs/function-design/90-traceability.md` を再生成 | REQ-101・REQ-905 の test を足すので再生成する（AC11） |

ほかの行（function-design doc 新設、source / workflow doc 新設、route、operator 画面）は該当なし（ダイアログは既存の画面の中の部品で、route と navigation を変えない）。

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | 外部の adapter と app-core の境 | — | 不該当: レジの file・adapter に触れない（Z004 の割り当ては今のまま） | なし |
| Fact check / design decision split | 観測した事実とアプリの判断の分け | D-141 | 当たる: 同じ JAN で色の違う品がある（店の事実、TD-334・TD-344）、群は全色を独自コード（owner の決定 TD-352）。既存の在庫連動を外す形はアプリの判断（Q1 の推奨）。1 色目の商品コードが JAN のまま残るのは TD-352 からの例外で、AI の review の問い（Q3） | Q1・Q3 |
| Lifecycle / retry | 前・中・後・失敗の後 | Contract Audit、Matrix の State Lifecycle | 当たる: 知らせ → 戻る / 送り直し → 成功 / 失敗。失敗は全て TX で戻る。もう一度「登録する」を押せば同じ知らせが出る | なし |
| Operator workflow | 実際の順序 | Ordinary Operation | 当たる: 2 色目の登録・廃番の品の登録し直し・接頭辞の無い部門 | L3 |
| Replacement path | 外部の系が変わったとき | — | 不該当: 店内コードの lane で D4 が変われば D-141 の Revisit | D-141 の Revisit |
| Data safety / evidence | 店のデータを repo に入れない | Data Safety | 当たる: test と L3 は合成の JAN だけ | なし |
| Reporting / accounting semantics | 集計の意味 | ADR D4 | 当たる（変えない）: 群の売上は先頭の商品に記録し在庫を動かさない今の規則のまま。在庫連動を外した商品は以後のレジの売上で在庫が減らない（ダイアログが示す） | なし |
| Manual verification | 自動 test で示せない主張 | Test Plan の L3 | 当たる: Windows の WebView2 でのダイアログの見え方・日本語・Tab と Esc・toast の説明の行 | L3-1〜L3-3 |
| 環境・再現性 | 新しい環境依存 | — | 不該当: 依存を足さない（AlertDialog は既存） | なし |

## Boundary / Wire Contract

- producer: frontend `buildCreateProductRequest`（`shared_jan_custom_code: false`）と `ProductFormPage` の送り直し（`true`）。error は BIZ `create_product` の手順 3a → CMD の変換。
- consumer: BIZ `create_product`。error は `ProductFormPage` の `createMutation.onError`。
- wire type: `ProductCreateRequest.shared_jan_custom_code: boolean`（必須。省くと command の deserialize が失敗する）。error は既存の `CmdError { kind: "duplicate", message: string, field: "jan_code" }`。成功は既存の `ProductCreateResult.warnings: string[]`。
- internal type: Rust `bool`、`BizError::JanInUse(String)`。
- precision/range: bool と文字列だけ。
- round-trip path: 1 回目の request（false）→ `JanInUse` → ダイアログ → 同じ値の request（true、`pos_stock_sync` だけ false）→ `ProductCreateResult`。2 回目の request は 1 回目と `shared_jan_custom_code`・`pos_stock_sync` 以外の field が同じ（Matrix の U3）。
- invalid input: `shared_jan_custom_code: true` で `jan_code: null` は意味を持たず今どおり発番。`true` で `pos_stock_sync: true` は、同じ JAN の別の商品がある状態で送ると手順 3b が field `pos_stock_sync` で拒む（同じ JAN の別の商品が無ければ拒まず、在庫連動の新しい商品を独自コードで登録する。Matrix の B5・B7）。
- compatibility: 古い frontend（field を送らない）は無い（frontend と backend は同じ build）。`DuplicateProductCode` の wire（field なし）は変えない。`CmdErrorKind` の値の数（13）は変えない。

## Test Plan

Test Design Matrix: `docs/plans/test-matrices/2026-10-27-jan-dup-custom-code.md`。

- targeted tests: `cd src-tauri && cargo test --lib product_service`、`cd src-tauri && cargo test --lib cmd::`、`npx vitest run src/features/products src/lib/invalidation-contract.meta.test.ts`。
- negative tests: Matrix の B5・B6・B8・U4・U6・U7。
- compatibility checks: JAN がほかの商品に無いときの既存の test（`test_create_product_req101_jan`・`test_create_product_req101_accepts_valid_jan_8_and_jan_13`）が変わらず PASS。`DuplicateProductCode` の変換の test（`test_cmd_error_req905_from_duplicate`）が変わらず PASS。
- data safety checks: test の JAN は合成（既存の test の `jan(n)` の helper、30 BIZ-01-D1 の golden の `4901234567887`）。
- main wiring/integration checks: `ProductFormPage.test.tsx` が `commands.createProduct` の mock の 2 回の呼出しと `invalidateByContract` の引数を確かめる（U3）。
- L3（Human Gate `manual`。Writer は AC14 の `cargo check --release` を済ませてから頼む）。到達経路は全て商品一覧 → 「新規登録」。入力物は無い（L3-1 の最初に owner がフォームで 1 商品を登録する）。依存する既知 backlog は無い。owner の操作の回数を減らす工夫: 3 項目を 1 回の起動で続けて行い（合計約 5 分）、同じ便に商品フォームを触る他の lane（#193 b2）の L3 があれば、先に merge する順を守ったうえで 1 回の依頼に並べる（束ねられる条件は `docs/DEV_WORKFLOW.md` Wave Operation の L3 の束ね方）。
  - L3-1（画面: 商品登録 `/products/new`）: 合成の JAN `4901234567887`・独自コードの部門で 1 商品を登録し、もう一度「新規登録」で別の名前・同じ JAN・独自コードの部門を入れて「登録する」。合格: ダイアログが開き、1 つ目の商品の商品コードと名前と「（POS販売で在庫を減らす）」が読め、2 段目の「この商品と、JAN が同じ商品は全て「POS販売で在庫を減らす」が「減らさない」になります」を含む文の 3 段と 2 つの button が日本語で読め、どの欄が変わるかが分かる。Tab で button を移れ、Esc で閉じて入力が残る。ダイアログの本文（使っている商品と 3 段の文）の箱に Tab（と Shift+Tab）で入れて focus の ring が見え、矢印の key で動き（本文が箱に収まって overflow が無ければ動かないのは期待どおり）、2 つの button にも Tab で移れる（Final Review とその closure の是正、Codex F1。overflow したときの見え方は window の最小の高さで保証できないので、自動 test の構造と Matrix の Residual Test Gaps に置く）。
  - L3-2（同じ画面）: もう一度「登録する」→「独自コードで登録する」。合格: toast に独自コード（接頭辞つき）と「同じ JAN の商品 4901234567887「…」の「POS販売で在庫を減らす」を「減らさない」にしました」が出て一覧へ戻る。一覧に 2 商品があり、1 つ目の修正の画面で「POS販売で在庫を減らす」が「減らさない」になっている。
  - L3-3（同じ画面）: 独自コードの接頭辞が無い部門を選び、同じ JAN で「登録する」。合格: ダイアログに「独自コードで登録する」が無く、部門の名前を挙げた文が出る。「戻る」で入力が残る。

## Review Focus

- Q1〜Q3 の推奨を既定にした所が、答えで変わる所として全て名指されているか（owner の判断事項の表、Scope の「この変更で不要になるもの」、D-141、Matrix）。
- 手順 3a の知らせが、JAN と商品コードの 2 つを見て、0 件のときに今の形（TD-338）を変えないか。
- 手順 3a2 の外しが D4 の群（同じ `jan_code`）だけで、商品コードだけが J の商品を外さないか。求め直しと BIZ-01-D7 のログが update と同じ形か。
- 失敗の順序（Contract Audit）と、全ての失敗で外しと next_seq が戻ること。
- ダイアログが message を解析せず kind と field で開き、field の無い `duplicate` を今の alert のまま出すか。送り直しが `pos_stock_sync: false` を必ず送るか。
- invalidation の entry が D-052 の導出（写した列を読む consumer）に合い、過剰でないか（`productCreate` を変えない）。
- この変更で不要になる記述・条件・code が他に無いか。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| BIZ-01-D11 (1) JAN をほかの商品が使っていれば `JanInUse` で全てを名指す（JAN と商品コードの 2 つ、昇順、印） | 30 §4.2 手順 3a・BIZ-01-D11 | `create_product` の手順 3a | B1・B2 | L3-1 |
| TD-338（JAN がほかの商品に無ければ JAN を商品コード） | 30 §4.2 手順 3a | 同上（今の形を保つ） | 既存の `test_create_product_req101_jan`、B3 | — |
| BIZ-01-D11 (2) `shared_jan_custom_code` で独自コード + JAN、群の在庫連動を外す、求め直し、BIZ-01-D7 のログ、warnings | 30 §4.2 手順 3a2・9、BIZ-01-D7、ADR SPEC-STK-CNT-D2 の表の「在庫連動の切替」 | 手順 3a2 | B4・B7・B9・B10（求め直し） | L3-2 |
| BIZ-01-D11 の文の語（message の印と warnings を商品フォームの画面の語で書く、印「商品コードだけ同じ」） | 30 BIZ-01-D11 の message の形・文の語の項、51 UI-01b-D26 | 手順 3a・3a2 の文 | B1・B2・B4、U2・U3 | L3-1・L3-2 |
| TD-352 からの例外（1 色目の商品コードは J のまま、再採番しない） | 30 BIZ-01-D11 の例外の項、D-141 Guarantee range、master-tables | 変えない（再採番の処理を足さない） | B4（A の `product_code` が J のまま） | 非対象（owner の確認は Q3） |
| 共有 JAN の不変条件（ADR SPEC-STK-CNT-D4）を変更の後の状態で検査、field `pos_stock_sync` | 30 冒頭の共有 JAN の項、§4.2 手順 3b | 手順 3b（field の選び分けを外す） | B5 | — |
| 失敗で全てを戻す（外し・next_seq・商品） | 30 §4.2 エラーハンドリング | TX の RAII | B5・B6・B8 | — |
| BIZ-01-D6 の発番（接頭辞の無い部門で止まる、番号の上限） | 30 §4.3 | 既存の `generate_custom_code` を 3a2 から呼ぶ | B6、既存の §4.3 の test | — |
| BIZ-01-D1 の JAN の形の検証（TX の外） | 30 §4.2 BIZ-01-D1 | 変えない（3a2 の経路でも先に効く） | 既存の `test_create_product_req101_rejects_invalid_jan_wire_with_exact_message` | 関門 8 の lane が BIZ-01-D1 の文を直す（重なりは Coordinator が取り直す） |
| `JanInUse` → kind `duplicate`・field `jan_code`、`DuplicateProductCode` は field なしのまま、CmdErrorKind 13 個 | 40 BizError → CmdError、D-061 | `cmd/mod.rs` | C1、既存の `test_cmd_error_req905_from_duplicate`（field `None` の assertion を足す） | — |
| `ProductCreateRequest.shared_jan_custom_code`（wire、必須の bool） | 30 §4.2、本 packet Boundary / Wire Contract | Rust の struct、bindings、`buildCreateProductRequest` | U1、AC5 | — |
| UI-01b-D26 ダイアログ（kind と field で開く、文、button、送り直し、warnings の toast、接頭辞の無い部門、戻る、失敗） | 51 UI-01b-D26・§7.5 Create 5・§7.6 | `ProductFormPage.tsx`・`SharedJanCustomCodeDialog.tsx` | U2〜U7 | L3-1〜L3-3 |
| 51 冒頭の共有 JAN の項（field `pos_stock_sync`、黙って off にして再送しない） | 51 冒頭の D-118 の節 | 今の出し方を保つ（本 lane は文と出し方を変えない） | 無し（`rg -n "在庫連動にできません" src/features/products/ProductFormPage.test.tsx` は recount_required の 1 件だけで、共有 JAN の拒否の表示の test は今も無い。2026-10-27。Matrix の Residual Test Gaps） | — |
| UI-01b-D14 保存成功 toast（商品コード、navigate より前） | 51 UI-01b-D14 | 送り直しの成功 | U3 | L3-2 |
| UI-01b-D15 / D-052 の invalidation（`productCreateSharedJan` = `productCreate` + `productForm.root()` + `stocktake.recheckList()`、`productCreate` は変えない） | UI_TECH_STACK 冒頭の段落・§2.5 | `invalidation-contract.ts` | I1、U3 | — |
| D-052 の C23 と件数（mutation entry 22 → 23、success handler は実装の後に実測） | D-141 Compatibility（D-052 の本文は append-only で書き換えない）、UI_TECH_STACK §2.5 の件数の行 | `invalidation-contract.ts` の doc comment、`invalidation-contract.meta.test.ts` の件数、UI_TECH_STACK の件数の行 | I1（`toHaveLength(23)`） | — |
| UI-USW-D1 離脱の確認（成功で baseline を同期、戻るで dirty を保つ） | 51 §7.6 | 今の `flushSync` の同期を送り直しの成功でも行う | U3・U4 | — |
| UI-01b-D16〜D18（JAN の正規化・検証・PLU の提案）は送り直しでも同じ値 | 51 UI-01b-D16〜D18 | 送り直しは 1 回目と同じ組み立ての値を使う | U3（field の一致） | — |
| UI-01b-D6（在庫連動の提案）・D13（確認ダイアログの先例） | 51 | ダイアログの送り直しは checkbox の値によらず false、checkbox の表示は変えない | U3 | — |
| UI-01b-D25（JAN の別名は知らせに当たらない） | 51 UI-01b-D25、D-141 Compatibility | 変えない（`product_jan_aliases` を読まない） | 非対象（別名の runtime は未実装、D-122） | — |
| 商品 CSV の取込みの共有 JAN の行の error | 30 §4.8・§4.9、60 | 変えない | 既存の `test_import_req104_shared_jan_row_error_both_directions_and_commit_recheck` | 非対象（関門 8 の lane） |
| D8 在庫連動を始める検査（false → true） | 30 冒頭の「在庫連動を始める」 | 変えない（3a2 は true → false だけ） | 既存の update の test | — |
| 33 の同一 JAN の PLU の判定（売価・税率の違いは要修正） | 33、D-141 Guarantee range | 変えない（Q2 の「知らせない」） | 非対象 | — |
| §4.2 手順 6 棚卸し中の明細の追加 | 30 §4.2 | 変えない（新しい商品だけ） | 既存の create の test | — |

隣接する契約の sweep（30 §4.2〜§4.4・§4.10、40 の変換の節、51 冒頭と §7.1〜§7.8、UI_TECH_STACK 冒頭）で、上の行に無いものは Scope が触らない（§4.4.1 価格改定・§4.5 廃番・§4.7 取引先・§4.9.1・§4.9.2 前の JAN の別名）。

## Contract Probe

- N/A: 外部の library・OS・hardware の未確認の前提に依らない。`bool` の field が tauri-specta で必須の `boolean` になることは、同じ struct の `pos_stock_sync: bool` → `pos_stock_sync: boolean`（`src/lib/bindings.ts` の `ProductCreateRequest`、2026-10-27 に読んだ）で現物にある。

## Data Safety

- commit しないもの: 店の JAN・商品名・DB・backup・log。test と L3 は合成の JAN（`4901234567887` は 30 BIZ-01-D1 の golden の valid な EAN-13、既存の test の `jan(n)`）。
- local だけの path: L3 の開発・試験 DB（owner の PC）。
- 合成だけの path: Rust の test の一時 DB（`setup_test_db`）、TS の test の mock。

## Implementation Results

exact-HEAD の SHA と test の本数は写さない（Evidence Ownership、D-038）。正本は PR #212 の本文・comment と helper の record。

- 実装（発注 678）: `948bb671`〜`6faff008`。BIZ の `JanInUse` と手順 3a・3a2（同じ JAN の群の在庫連動を外し、求め直し、操作ログ、warnings を同じ TX で）、CMD の kind `duplicate`・field `jan_code` の変換、wire の `shared_jan_custom_code`、商品フォームの `SharedJanCustomCodeDialog` と送り直し、D-052 の entry `productCreateSharedJan`（C23）、bindings と traceability の再生成。Matrix の B1〜B10・C1・U1〜U7・I1。M1〜M11 が assertion で red になることは Final Review の Codex も実測した（PR #212 の comment）。
- Final Review（2 本、https://github.com/kosei-w90607/inventory-system-desktop/pull/212#issuecomment-6543042258）: Codex は request changes（P2 2・P3 1）、Fable 5 は approve（P3 2）。裁定は全件受理。
- 是正（発注 685）: ダイアログの高さを viewport の中に収め、文だけを scroll し、title と button を scroll の外に固定した（F1。U2 の多数の商品の case）。Final の closure（発注 689、Codex P2）を受け、本文の箱を名前の付いた scroll の region（`tabIndex={0}`）にし、Tab と Shift+Tab で入れることを U2 で確かめ、L3-1 を本文の region の focus と矢印の key の観測に書き直した。U3 に遷移の時点の dirty が false の assert を足した（F2。M13）。設計正本の「未実装」の印を「D-141 proposed・実装済み（PR #212、L3 待ち）」にし、本節と Review Response を埋めた（F3 と Fable 5 の P3）。
- 残り: L3-1〜L3-3（AC15、Human Gate `manual`）と closure の review。D-141 の Status は owner の Q1〜Q3 の確認まで proposed のまま。

## Review Response

### Final Review（2026-10-31、REVIEWED_HEAD `3442e865`）

- Codex（gpt-6.1-sol、high）: request changes。P1 0 / P2 2 / P3 1。
- Fable 5: approve。P1 0 / P2 0 / P3 2。
- Coordinator の裁定: 全件受理（PR #212 の comment）。是正は発注 685。

| finding | 採否 | 是正 |
|---|---|---|
| Codex F1（P2）ダイアログに高さの上限と scroll が無く、商品が多い・window が低いと説明と button に届かない | 受理 | `SharedJanCustomCodeDialog.tsx`: AlertDialogContent を `flex max-h-[90vh] flex-col`、header を縮められる flex にし、文の 3 段の箱だけを `overflow-y-auto`。title と button は scroll の外（DSR-24 の picker dialog の先例）。共通の AlertDialog は変えない。Matrix U2 に合成 20 件の case と M12、51 UI-01b-D26 に形を足した。closure（発注 689、Codex P2: 箱が `tabIndex -1` で Radix の FocusScope の Tab の循環に入らず、Chromium の自動の Tab の対象化も保証にならない）を受け、箱を `role="region"`・`aria-label="使っている商品と変わる所"`・`tabIndex={0}` にした（`jsx-a11y/no-noninteractive-tabindex` はこの 1 か所だけ理由付きの disable）。U2 に Tab・Shift+Tab で region に入る assert と M14、L3-1 を region の focus・矢印の key・button への Tab に書き直した |
| Codex F2（P2）U3 が成功時の遷移の前の baseline の同期（UI-USW-D1）を確かめない | 受理 | U3 で `onNavigateToList` の呼ばれた時点の dirty を記録し `false` を assert。create の成功の `setSavedValues` を消す mutation で、足す前は green・足した後は red、戻して green を確かめた（Matrix M13） |
| Codex F3（P3）・Fable 5 P3 1 本目 実装済みなのに「未実装」の印が残る | 受理 | 51 UI-01b-D26 と §7.2 の file tree、SCREEN_DESIGN、UI_TECH_STACK の冒頭の段落、master-tables の例外の項を「D-141 proposed・実装済み（PR #212、L3 待ち）」にした。D-141 の Status と D-052 の道標は変えない。51 の変更履歴に 1 行 |
| Fable 5 P3 2 本目 Implementation Results・Review Response が空欄 | 受理 | 本節と上の Implementation Results を埋めた（SHA・件数は写さない） |

### Plan Review round 1（2026-10-29、REVIEWED_HEAD `010a4ac8`）

- fresh Opus 5.5: request changes。P1 0 / P2 2 / P3 3。
- Codex: request changes。P1 0 / P2 2 / P3 4。
- Coordinator の裁定: 全件受理（下の採否）。是正は plan-draft のまま（遷移の記録の 4）。

| finding | 採否 | 是正 |
|---|---|---|
| Opus P2-1 ダイアログ・message の印・warnings・L3 が画面に無い「在庫連動」を使う（51 の冒頭の `recount_required` の項〈#210〉に反する） | 受理 | 30 BIZ-01-D11 の message の印と文の語の項、手順 3a2 の warnings、51 UI-01b-D26 の 1 段目・2 段目、SCREEN_DESIGN の 1 項、Ordinary Operation の 1・2 行目、L3-1・L3-2 を欄の名「POS販売で在庫を減らす」と値「減らす」「減らさない」で書き直した。51 の冒頭の `recount_required` の項は正本として変えない。冒頭の共有 JAN の項の BIZ の message（main に前からある食い違い）は backlog の候補にした（Opus の E の気づき） |
| Opus P2-2・Codex F2 「TD-352 と同じ形」が成り立たない（1 色目の商品コードが J のまま残る） | 受理 | 30 BIZ-01-D11 の本文と却下 (a)、D-141 の Context・Why・Guarantee range・Revisit、master-tables の困りそうなケース、Scope の決めたことで「同じ JAN を各色に持たせる形」に弱め、TD-352 の例外を明記した。再採番は足さない。owner の判断事項に Q3 |
| Codex F1 通常の重複の field の不変を検証できない | 受理 | Matrix C1 に既存の `test_cmd_error_req905_from_duplicate` へ `field.is_none()` の assertion、M11（`DuplicateProductCode` の field を `Some("jan_code")` にする）。実装の Scope の `cmd/mod.rs` |
| Opus P3 D-052 の C23 と件数 | 受理 | D-052 は merge 済みで本文を書き換えない（decision-log の冒頭の規則）ので、D-141 の Compatibility に C23 と件数を置き、実装の Scope に `invalidation-contract.ts` の doc comment・meta.test の件数（22 → 23）・UI_TECH_STACK の件数の行の実測、D-141 が accepted になったときの D-052 の Superseded in part の行を足した。Ledger・AC7 |
| Opus P3・Codex F3 Q1 の記号の食い違いと答えで変わる所の漏れ | 受理 | 記号を D-141 の Alternatives に揃え（packet の (b)・(c) → (c)・(d)）、Q1〜Q3 の答えで変わる所を owner の判断事項の表にした |
| Opus P3 商品コードだけが同じ商品が外れると読める | 受理 | 印「商品コードだけ同じ」を足し、2 段目を「JAN が同じ商品は…」「「商品コードだけ同じ」の商品は変わりません」にした（30・51・Matrix B1・B2・U2） |
| Codex F4 `pos_stock_sync: true` の拒否の条件が広い | 受理 | Boundary / Wire Contract の invalid input を「同じ JAN の別の商品がある状態で送ると拒む」に限った |
| Codex F5 UI-01b-D25 の理由に旧い経路の限定が残る | 受理 | 51 UI-01b-D25 の理由を初回の知らせと確認の後の登録に分けて書き直した（D-122 の別名を消す規則と、create で別名に固有の警告を出さない契約は保つ） |
| Codex F6 B4 では求め直しを証明できない | 受理 | Matrix に B10（写しをわざと記録からの期待値と違えておき、数量・要再確認・範囲が求め直される）と M10（3a2 の `recompute_stock` の呼出しを省く） |
