# Test Design Matrix: 新規登録で、入れた JAN をほかの商品が使っているとき独自コードで登録できる道

Plan Packet: `docs/plans/2026-10-27-jan-dup-custom-code.md`。oracle は設計正本（30 §4.2 BIZ-01-D11・手順 3a/3a2/3b、40 の変換、51 UI-01b-D26、UI_TECH_STACK 冒頭の段落）から独立に写す。production の文字列・集合を test に import しない。

## Risk

Risk: R3

## Contracts Under Test

- BIZ-01-D11 (1): 新規登録で入れた JAN を、ほかの商品が JAN か商品コードとして持てば `JanInUse` で止め、message が使っている全ての商品を商品コードの昇順で名指す（印は「商品コードだけ同じ」「POS販売で在庫を減らす」「廃番」をこの順に「・」でつないで 1 つの括弧に入れ、当たらない商品には括弧を付けない。商品フォームの画面の語で書き、「在庫連動」を使わない。round 1 の是正）。何も書かない。
- TD-338: JAN をどの商品も使っていなければ JAN を商品コードにする（今の形）。
- BIZ-01-D11 (2): `shared_jan_custom_code = true` で独自コードを振り JAN を持たせ、同じ JAN の在庫連動の商品を同じ TX で在庫連動なしにし、求め直し、`product_update` のログ（detail `{"pos_stock_sync":{"old":true,"new":false}}`）と warnings（「同じ JAN の商品 {商品コード}「{商品名}」の「POS販売で在庫を減らす」を「減らさない」にしました」）で名指す。既存の商品の商品コードは変えない（TD-352 からの例外）。
- 30 §4.2 手順 3b: 変更の後の状態で D4 を検査、field `pos_stock_sync`。
- 30 §4.2 エラーハンドリング: 3a2 より後の失敗で、外した在庫連動・next_seq・商品が全て戻る。
- 40: `JanInUse` → kind `duplicate`・field `jan_code`・message はそのまま。
- 51 UI-01b-D26: kind `duplicate` かつ field `jan_code` でダイアログ。送り直しは `shared_jan_custom_code: true`・`pos_stock_sync: false`、ほかの field は同じ。成功で toast（warnings の行）と遷移、invalidation は `productCreateSharedJan`。戻るで送らず入力を保つ。接頭辞の無い部門で主の button が無い。送り直しの失敗はダイアログを閉じて保存の error。
- UI_TECH_STACK 冒頭の段落: `productCreateSharedJan` = `productCreate` の 7 key + `productForm.root()` + `stocktake.recheckList()`。`productCreate` は変えない。

## Failure Modes

- 商品コードだけを見て、ほかの商品の `jan_code` にある JAN を黙って商品コードにする（旧い形）。
- JAN を見て、商品コードが J で JAN の違う商品を見落とし、INSERT の一意制約で `DuplicateProductCode` になる（名指しが無い）。
- 名指しの並びが不定（DB の返す順）、印が抜ける。
- 3a2 で群を外し忘れて D4 の拒否になる、または商品コードだけが J の商品（群でない）まで外す、在庫連動でない商品にもログを書く。
- 3a2 の外しを別の TX で行い、後の失敗で在庫連動だけが外れて残る。next_seq だけが進んで残る。
- 3a2 で外した商品を求め直さず、写し（在庫・要再確認・範囲）が古いまま残る。
- 文に画面に無い「在庫連動」を使う、商品コードだけが同じ商品も外れると読める。
- CMD が field を付けず、UI がダイアログを開けない。CMD が通常の重複（`DuplicateProductCode`）にも field `jan_code` を付け、UI が通常の重複でダイアログを開く。UI が message を解析する、field の無い `duplicate`（同時の登録）でもダイアログを開く。
- 送り直しが checkbox の値（true）を送り D4 で拒まれる。1 回目と違う値（正規化前の JAN 等）を送る。
- 送り直しの成功で `productCreate` だけを invalidate し、外した商品の修正の画面と要再確認の一覧が古いまま。
- 戻るで送られる、入力・dirty が消える。

## Test Matrix

- Before citing an existing test as regression coverage, use `rg` or an equivalent repository search to verify that the cited test exists.（下の既存の test は 2026-10-27 に `rg -n "fn <名前>" src-tauri/src` と `rg -n "<it の名前>" src` で確かめた）
- helper と mock の実装を読み、実際に通る境界と置換される境界を確認して Test Type / coverage を選ぶ。Rust の B 行は `setup_test_db` の実 SQLite と実 `create_product`（mock なし）。TS の U 行は `commands.*` を mock した `ProductFormPage` の render で、BIZ を通らない（BIZ の契約は B 行が持つ）。
- oracle の anchor（message・文・toast）は設計正本の文字どおりの literal を test に写す。商品の名前・商品コードは test の既定（`default_create_request` の名前等）と区別できる値にする。
- 結果を空にする注入で、期待が空集合の case だけが kill を主張していないかを確かめ、各組合せに非空の期待を 1 件置く（B4 は外す商品が 1 件・外さない商品が 1 件、B7 は空）。

| ID | Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|---|
| B1 | BIZ-01-D11 (1) JAN で名指す・並び・印 | ほかの商品の `jan_code` を見ない / 並び不定 / 印の欠け | unit（実 DB） | 新設（REQ-101。例 `test_create_product_req101_rejects_jan_in_use_naming_all_products`）: IO で直に 4 商品を置く（D4 の違反の組も fixture として置ける。`seed_jan_product` の形）: 商品コード J・`jan_code` = 別の値 K・在庫連動の X、`jan_code` = J の `HZ-0003`（在庫連動なし・廃番）・`HZ-0002`（在庫連動・廃番）・`HZ-0001`（在庫連動なし）をこの順に INSERT。J で create（flag false）→ `Err(JanInUse(m))`、m = `この JAN（J）はほかの商品が使っています: J「X の名前」（商品コードだけ同じ・POS販売で在庫を減らす）、HZ-0001「…」、HZ-0002「…」（POS販売で在庫を減らす・廃番）、HZ-0003「…」（廃番）`（完全一致。J は数字なので商品コードの昇順で先頭）。m に `在庫連動` が無い。商品の数・部門の next_seq・操作ログの数が変わらない | 3a が `find_by_product_code` だけを見る（M1）、2 つの検索の結果を連結の順のまま並べる（M8: `jan_code` の側を先に連結すると J が末尾になる）、印を付けない・組合せの印や順を誤る・「商品コードだけ同じ」を JAN の一致した商品にも付ける |
| B2 | 同上、商品コードの側・群の外 | 商品コードが J で JAN の違う商品を見落とす / 3a2 で外す | unit | 新設（例 `test_create_product_req101_jan_in_use_by_product_code_only`）: 商品コード J・`jan_code` = 別の値・在庫連動の商品 X。J で create（flag false）→ `JanInUse`、X を印「（商品コードだけ同じ・POS販売で在庫を減らす）」付きで名指す。続けて flag true → Ok、X の在庫連動は true のまま、warnings は空 | 3a が `jan_code` だけを見る、3a2 が商品コードでも外す |
| B3 | TD-338 | JAN が空いていても独自コードにする | unit | 既存 `test_create_product_req101_jan`、`test_create_product_req101_accepts_valid_jan_8_and_jan_13`（`result.product_code == jan_code`） | 3a が常に `JanInUse` を返すか独自コードにする |
| B4 | BIZ-01-D11 (2) 群を外す・ログ・warnings・求め直し | 外し忘れ / 群でない商品・連動でない商品へのログ / warnings の欠け | unit | 新設（例 `test_create_product_req101_shared_jan_custom_code_unlinks_group`）: 商品コード J・`jan_code` J・在庫連動の A と、`jan_code` J・在庫連動なしの C。まず flag false で create → `JanInUse` の message に J が 1 回だけ出て、A の印は「（POS販売で在庫を減らす）」（JAN も J なので「商品コードだけ同じ」を付けない。2 つの検索の両方に当たる A を重ねない）。次に flag true・`pos_stock_sync` false・接頭辞のある部門で create → Ok。`product_code` は接頭辞つきで J でない、新しい商品の `jan_code` = J・在庫連動 false。A は false、C は false のまま。操作ログの `product_update` は A の 1 行だけで detail が `{"pos_stock_sync":{"old":true,"new":false}}`、summary `商品を更新しました: J`。warnings = [`同じ JAN の商品 J「A の名前」の「POS販売で在庫を減らす」を「減らさない」にしました`]。A の `stock_quantity` が create の前と同じ。A の `product_code` は J のまま（TD-352 からの例外。再採番しない） | 3a2 が外さない（M2。D4 で Err）、在庫連動でない C にもログを書く（M3）、warnings を返さない |
| B5 | 手順 3b と失敗で戻す | 新しい商品を在庫連動で送っても通る / 外しが残る | unit | 新設（例 `test_create_product_req101_shared_jan_custom_code_rejects_linked_new_and_rolls_back`）: B4 と同じ A で、flag true・`pos_stock_sync` true → `ValidationFailedAt { field: "pos_stock_sync", message: "同じ JAN の商品（J）があるため、在庫連動にできません" }`。A は在庫連動 true のまま、部門の next_seq・商品の数・操作ログの数が変わらない | 3b を外す、3a2 の外しを別の TX にする（M9） |
| B6 | 接頭辞の無い部門で止まり戻す | 発番の前に外しが残る | unit | 新設（例 `test_create_product_req101_shared_jan_custom_code_requires_code_prefix`）: B4 の A、接頭辞の無い部門で flag true → `ValidationFailed("この部門は独自コード発番に対応していません")`、A は true のまま | 3a2 が発番の前に外し、TX の外で書く |
| B7 | 外す商品 0 | 群が無いと失敗にする / 空の warnings に行を足す | unit | 新設（例 `test_create_product_req101_shared_jan_custom_code_without_group`）: J をどの商品も使っていない DB で flag true → Ok、独自コード、`jan_code` = J、warnings = []、`product_update` のログ 0 | 群が無いと Err にする、warnings に空の行 |
| B8 | 3a2 より後の失敗で全て戻す | INSERT の後の失敗で外しが残る | unit | 新設（例 `test_create_product_req101_shared_jan_custom_code_rolls_back_on_late_failure`）: 既存の failpoint `CREATE_PRODUCT_AFTER_INSERT` を立て、B4 の A で flag true → Err。A は true のまま、next_seq・商品の数・ログの数が変わらない | 3a2 の外しを先に commit する（M9） |
| B9 | jan None で flag は意味を持たない | flag で振舞いが変わる | unit | 新設（例 `test_create_product_req101_shared_jan_custom_code_ignored_without_jan`）: jan None・flag true → 今どおり独自コード、warnings = [] | flag true で jan None を拒む・別の経路にする |
| B10 | BIZ-01-D11 (2) 外した商品の求め直し（round 1 の是正、Codex F6） | 3a2 が `recompute_stock` を呼ばず、外した商品の写しが古いまま | unit（実 DB） | 新設（例 `test_create_product_req101_shared_jan_custom_code_recomputes_unlinked`。既存 `test_update_product_req102_toggle_stock_link_recomputes` の「写しをわざと食い違わせて観測する」形）: 商品コード J・`jan_code` J・在庫連動の A を `seed_jan_product` で置き、初期在庫の movement（`receiving`、数量 5）を IO で直に足して記録からの期待値を 5・要再確認なし・範囲なしにする。次に写しを SQL で直に `stock_quantity = 77`・`stock_recheck = 1`・`stock_range_low = 70`・`stock_range_high = 80` に違えておく。flag true・`pos_stock_sync` false・接頭辞のある部門で create → Ok。A の写しが `stock_quantity = 5`・`stock_recheck = 0`・`stock_range_low`・`stock_range_high` が NULL（記録からの期待値。test に literal で書き、production の関数で求めない） | 3a2 の `recompute_stock` の呼出しを省く（M10）、求め直しを新しい商品だけにする |
| B-old1 | 旧い契約の置換え | — | unit | 直す `test_create_product_req101_duplicate_jan`（2 つ目の create の期待を `DuplicateProductCode` から `JanInUse` へ。理由: BIZ-01-D11） | — |
| B-old2 | 旧い契約の置換え | — | unit | 直す `test_create_product_req101_rejects_shared_jan_with_stock_linked`（1 つ目・3 つ目の create は 3a で `JanInUse` になる。D4 の拒否の case は B5 へ移し、field `jan_code` の assert を外す。理由: BIZ-01-D11 で JAN の側から D4 に届く経路が無くなる） | — |
| C1 | 40 の変換 | field が付かない / message を作り直す | unit | 新設（REQ-905。例 `test_cmd_error_req905_from_jan_in_use`）: `BizError::JanInUse("X")` → `kind == Duplicate`、`field == Some("jan_code")`、`message == "X"`。既存 `test_cmd_error_req905_from_duplicate`（`src-tauri/src/cmd/mod.rs` の 309 行、2026-10-29 に `rg -n 'fn test_cmd_error_req905_from_duplicate'` で確かめた。今は kind と message しか見ない）に `assert!(cmd_err.field.is_none())` を足す（round 1 の是正、Codex F1。U6 は mock の `field:null` なので producer の誤変換を見ない。Codex が隔離の copy で、assertion を足すと誤変換で red、戻すと green を確かめた） | `JanInUse` の field を `None` にする（M4）、message に接頭辞を足す、`DuplicateProductCode` の field を `Some("jan_code")` にする（M11。通常の重複でダイアログが開く回帰） |
| U1 | wire の既定 | 1 回目に flag を送らない・true を送る | unit（TS） | `product-form-request.test.ts` の `builds JAN product request` に `shared_jan_custom_code: false` を足す | `buildCreateProductRequest` が field を省く・true にする |
| U2 | UI-01b-D26 開く | ダイアログが開かない / message を解析 | component | 新設（`ProductFormPage.test.tsx`、REQ-101 / UI-01b-D26）: `createProduct` が `{kind:"duplicate", field:"jan_code", message:"この JAN（…）はほかの商品が使っています: P-901「赤」（POS販売で在庫を減らす）"}` を返す → `alertdialog` に title `この JAN はほかの商品が使っています`・その message・2 段目と 3 段目の文（51 の文字どおり）・button `独自コードで登録する`・`戻る`。`onNavigateToList` は呼ばれない、入力は残る。多数の商品の case（Final Review の是正、Codex F1）: 合成の 20 件を名指す message で、`alertdialog` が `flex max-h-[90vh] flex-col`、文の 3 段の箱が `min-h-0 overflow-y-auto`、その親の header が縮められる flex（`min-h-0`）、title と 2 つの button は箱の外。箱は role `region`・名前 `使っている商品と変わる所` で、初めの focus の `戻る` から Shift+Tab で箱、Tab で `戻る` → `独自コードで登録する` → 箱と回る（closure の是正、Codex P2。happy-dom は layout を測らないので上限と構造と focus の順だけ） | onError が field を見ない（M5 は U6 で）、ダイアログを作らない、高さの上限・scroll を外す・button を scroll の箱へ入れる（M12）、箱を Tab の循環から外す（M14） |
| U3 | 送り直し・toast・invalidation | 値が違う / checkbox の値を送る / 古い entry | component | 新設: U2 の後、在庫連動の checkbox は on のまま `独自コードで登録する` → 2 回目の `createProduct` の引数が 1 回目と `shared_jan_custom_code: true`・`pos_stock_sync: false` 以外で一致。resolve `{product_code:"HZ-0012", warnings:["同じ JAN の商品 P-901「赤」の「POS販売で在庫を減らす」を「減らさない」にしました"]}` → toast の title `商品「…」を登録しました（商品コード: HZ-0012）`、説明にその warnings の行、`id: "product-save-success"`。`expectExactInvalidations(..., d052InvalidationOracle.productCreateSharedJan())`。`onNavigateToList` が呼ばれ、呼ばれた時点で離脱の確認に渡る dirty が `false`（UI-USW-D1 の成功時、遷移の前の baseline の同期。Final Review の是正、Codex F2）。warnings が空の case は説明を出さない | `pos_stock_sync` を form の値で送る（M6）、`productCreate` で invalidate（M7）、warnings を出さない、成功時の `setSavedValues` を消す（M13） |
| U4 | 戻る | 送る / 入力が消える | component | 新設: U2 の後 `戻る` → `createProduct` は 1 回だけ、ダイアログが閉じ、入力値が残り、`一覧へ戻る` で離脱の確認が出る（dirty のまま） | 戻るで送り直す、values を reset する |
| U5 | 接頭辞の無い部門 | 主の button が出る | component | 新設: 部門の `code_prefix` が null の部門を選んだ状態で U2 → `独自コードで登録する` が無く、`部門「{部門名}」は独自コードを振れません。独自コードを振れる部門を選び直してから、もう一度「登録する」を押してください。` | 部門を見ずに主の button を出す |
| U6 | field の無い duplicate | 全ての duplicate でダイアログ | component | 新設: `{kind:"duplicate", field:null, message:"この商品コードは既に使用されています: X"}` → ダイアログは開かず、`保存できませんでした` の alert に message | kind だけで開く（M5） |
| U7 | 送り直しの失敗 | ダイアログが残る / error が出ない | component | 新設: U2 の後、2 回目の `createProduct` が `{kind:"validation", field:null, message:"この部門の独自コードの番号を振れません"}` → ダイアログが閉じ、`保存できませんでした` の alert にその message、遷移しない | 失敗でダイアログを閉じない、saveError に入れない |
| I1 | D-052 の entry | entry が無い / `productCreate` を変える | unit（TS） | `invalidation-contract.meta.test.ts` の key の一覧に `productCreateSharedJan`、oracle（`src/test/invalidation-oracle.ts`、UI_TECH_STACK の段落から写す）と完全一致。`productCreate` の比較は今のまま | entry の key を足し忘れる、`productCreate` に recheckList を足す |

## State Lifecycle Matrix

| State / subject | Initial | Pending | Success | Invalidate | Refetch | Revisit | Restart | Failure | Retry | Evidence |
|---|---|---|---|---|---|---|---|---|---|---|
| 商品フォームの create（ダイアログ） | 入力済みの form、ダイアログは閉 | 1 回目の `createProduct`（保存の button は pending で押せない） | 1 回目が成功（JAN が空いている）→ 今の toast と遷移 | `productCreate`（今のまま） | 一覧が再取得 | — | — | 1 回目が `duplicate`+`jan_code` → ダイアログ開。ほかの error → 今の alert | `登録する` をもう一度押す → 同じ知らせ | U2・U6、既存の create の test |
| ダイアログの送り直し | ダイアログ開 | 2 回目の `createProduct`（主の button は pending で押せない） | toast（warnings）と遷移、baseline を同期 | `productCreateSharedJan` | 一覧・在庫照会・要再確認の一覧・外した商品の修正の画面が再取得 | 外した商品の修正の画面を開くと在庫連動が外れている | DB は commit 済み（再起動しても同じ） | ダイアログを閉じ alert、入力は残る | `登録する` から再び（同じ知らせ） | U3・U7、L3-2 |
| 戻る | ダイアログ開 | — | ダイアログ閉、dirty のまま | なし | なし | — | — | — | `登録する` | U4 |
| 群の商品の在庫連動（DB） | A は在庫連動 | 送り直しの TX の中 | A は false、ログ 1 行、写しは同じ値 | — | — | — | — | TX が戻り A は true | — | B4・B5・B8・B10 |

## Adjacent Pattern Audit

| Source pattern / contract | Repository sites inspected | Ported sites | Explicit exclusions and reason | Test / evidence |
|---|---|---|---|---|
| error を kind（と code / field）で分け、message を解析しない | `ProductFormPage.tsx` の `updateMutation.onError`（`stocktake_guard` + `recount_required`）、`createMutation.onError`（今は describeError だけ） | `createMutation.onError` に kind `duplicate` + field `jan_code` | `toggleMutation.onError`（duplicate を返さない） | U2・U6 |
| 確認ダイアログ（shadcn AlertDialog、DSR-07） | `src/features/products/components/DiscontinueConfirmDialog.tsx`、`PluBulkTargetConfirmDialog.tsx` | `SharedJanCustomCodeDialog.tsx`（同じ部品・pending の扱い） | — | U2〜U5、L3-1 |
| 在庫連動の true → false（update の形: `update_product`・`recompute_stock`・BIZ-01-D7 のログ） | `update_product`（30 §4.4、`product_service.rs` の「連動の切替の TX はその商品を求め直す」）、`commit_import` の上書きの行（在庫連動を書く） | create の手順 3a2 | `commit_import` は本 lane で変えない（関門 8・非目的） | B4 |
| 成功の toast と invalidation（UI-01b-D14・D15、UI-USW-D1 の baseline の同期） | `createMutation.onSuccess`、`updateMutation.onSuccess` | 送り直しの成功 | — | U3 |
| D-052 の entry と独立の oracle | `src/lib/invalidation-contract.ts`、`src/test/invalidation-oracle.ts`、`invalidation-contract.meta.test.ts` | `productCreateSharedJan` | — | I1 |

## Negative Paths

- missing input: jan None で flag true（B9）。
- invalid input: flag true で `pos_stock_sync` true（B5）。JAN の形の誤りは BIZ-01-D1 が TX の外で先に拒む（既存 `test_create_product_req101_rejects_invalid_jan_wire_with_exact_message`）。
- duplicate/ambiguous input: JAN がほかの商品の `jan_code` に 2 件（B1）、商品コードにだけ（B2）。
- unknown reference: 部門・取引先が無い → 今の検証（TX の外）。
- dependency missing: 接頭辞の無い部門（B6・U5）。
- permission/write failure: INSERT の後の失敗（B8 の failpoint）。
- dry-run side effect: 1 回目（flag false）の知らせで何も書かない（B1 の数の不変）。

## Boundary Checks

- threshold: 外す商品 0 / 1 / 群に連動でない商品が混ざる（B7・B4）。
- null/default: `jan_code` None、`field` null の duplicate（B9・U6）。
- empty/non-empty: warnings 空 / 1 件（U3 の 2 case）。
- min/max: 独自コードの番号の上限は §4.3 の既存の test（3a2 でも同じ関数）。
- status/policy enum: CmdErrorKind の値は変えない（`finite_enum_contract_tests.rs` の既存の検査）。
- wire type: `shared_jan_custom_code: boolean`（AC5）。
- internal type: `bool`、`BizError::JanInUse(String)`。
- producer/consumer: 1 回目の producer（`buildCreateProductRequest`）と 2 回目（送り直し）が同じ値（U3）。
- round-trip token: 無し（送り直しは state に残した 1 回目の request を使う）。
- precision/range: 無し。
- cross-language parse: field `jan_code` の文字列が Rust（`"jan_code".to_string()`）と TS（`=== "jan_code"`）で同じ（C1・U2）。

## Compatibility Checks

- old schema/input: DB の列・制約は変えない。JAN が空いているときの request（flag false）は今と同じ結果（B3）。
- new schema/input: `ProductCreateRequest` に必須の bool（frontend と backend は同じ build）。
- output order: 名指しは商品コードの昇順（B1）。warnings も商品コードの昇順。
- optional field behavior: `ProductCreateResult.warnings` は今も空の配列を返す経路を保つ（B3・B7）。

## Data Safety Checks

- source-derived data: 無し（店の JAN・商品名を使わない）。
- generated outputs: `src/lib/bindings.ts`・`docs/function-design/90-traceability.md`（generator で再生成）。
- secrets: 無し。
- local-only files: L3 の開発・試験 DB。
- synthetic sample boundaries: test の JAN は `jan(n)` と BIZ-01-D1 の golden。

## Main Wiring / Integration Checks

- helper connected to main path: `buildCreateProductRequest` の値が 1 回目と送り直しの両方に使われる（U3）。
- output reaches manifest/report: 操作ログの `product_update`（B4）、toast の warnings（U3）。
- effective config reaches runtime: 無し。
- CLI arg reaches implementation: command `create_product` は BIZ を素通しで呼ぶ（`product_cmd.rs`、変えない）。

## Mutation-style Adequacy Questions

| ID | 注入（production だけを変える） | red になる test |
|---|---|---|
| M1 | 3a で `find_by_jan_code` を呼ばず `find_by_product_code` だけを見る | B1 |
| M2 | 3a2 の外しの loop を消す | B4（D4 で Err） |
| M3 | 3a2 で在庫連動でない商品にも update とログを書く | B4（ログの数） |
| M4 | CMD の `JanInUse` の field を `None` にする | C1 |
| M5 | UI の onError で field を見ず kind `duplicate` だけでダイアログを開く | U6 |
| M6 | 送り直しの `pos_stock_sync` を form の値にする | U3 |
| M7 | 送り直しの成功の invalidation を `productCreate` にする | U3 |
| M8 | 名指しの並びを、`find_by_jan_code` の結果の後に `find_by_product_code` の結果を連結した順のまま（sort しない）にする | B1（商品コードの側だけの X〈商品コード J〉が末尾になる） |
| M9 | 3a2 の外しを TX の外の接続で書く（または外しの後に commit する） | B5・B8 |
| M10 | 3a2 で外した商品の `recompute_stock` の呼出しを省く（round 1 の是正、Codex F6） | B10（写しが 77・要再確認・範囲のまま） |
| M11 | CMD の `DuplicateProductCode` の field を `Some("jan_code")` にする（round 1 の是正、Codex F1） | C1（既存 `test_cmd_error_req905_from_duplicate` の `field.is_none()`） |
| M12 | `SharedJanCustomCodeDialog` の高さの上限・文の箱の scroll を外す（Final Review の是正、Codex F1） | U2 の多数の商品の case（是正の前の形で red を確かめた） |
| M13 | create の成功の `flushSync(() => setSavedValues(...))` を消す（Final Review の是正、Codex F2） | U3（遷移の時点の dirty が true になる） |
| M14 | 本文の region の `tabIndex` を -1 にする（Final の closure の是正、Codex P2） | U2 の多数の商品の case（Shift+Tab で箱に focus が入らない） |

- If a mock value is changed so it differs from the design-doc expected value: U2・U3 の message・warnings は mock の値をそのまま出すことを確かめ、文（2 段目・3 段目・接頭辞の無い部門）は 51 の文字どおりの literal で確かめる（mock から来ない）。
- If invalidate/refetch changes the value before versus after the operation: U3 の `expectExactInvalidations` が entry の集合を確かめる（M7）。
- If a key branch is inverted: M5（field の分岐）、M6。
- If a threshold comparison changes: 該当なし（数の閾値を足さない）。
- If a guard is removed: 3b（B5）、接頭辞（B6）。
- If an output field is omitted: warnings（B4・U3）、field（C1）。
- If output order changes: M8。
- If dry-run performs a side effect: B1 の数の不変（1 回目の知らせで書かない）。
- If a JSON number crosses JavaScript safe integer range: 該当なし（新しい数を wire に足さない）。
- If a state token is round-tripped through browser/client code: 該当なし（送り直しは state の request を使う）。

## Residual Test Gaps

- 共有 JAN の拒否（field `pos_stock_sync`）を商品フォームがどこに出すかの test は今も無い（51 冒頭の項は「該当の欄の近く」、今の実装は form の上の alert）。本 lane は文と出し方を変えないので足さない（既存の gap。必要なら別の小口）。
- Windows の WebView2 でのダイアログの見え方・Esc・Tab は L3-1〜L3-3（自動 test では jsdom の振舞いだけ）。
- ダイアログの高さの上限が実際に効き、本文が overflow したときに箱の中を scroll して全文と 2 つの button に届くことの見え方は、どの自動 test も測らない（happy-dom は layout を測らない。U2 は class・構造・focus の順だけ）。L3-1 は window の最小の高さ（Tauri の `minHeight`）のため overflow を作れるとは限らないので、L3-1 は本文の region に Tab で入れて focus の ring が見えることと矢印の key で動くことまでを持ち、overflow したときの見え方は本 gap として残す（Final Review とその closure の是正、Codex F1・P2）。
