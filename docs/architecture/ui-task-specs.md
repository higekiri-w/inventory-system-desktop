# タスク仕様（UI層）

## 計数の置き場と在庫の求め直しの契約（D-118。(a)・b1・b2 は実装済み）

SPEC-STK-CNT-D3〜D6（[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)）。(a)（ADR SPEC-STK-CNT-D12）で入ったのは、整合性検証（[75](../function-design/75-ui-integrity-check.md)）と操作ログ（[74](../function-design/74-ui-operation-logs.md) UI-11c-D14）の表示。b1（[Plan Packet](../archive/plans/2026-10-22-stocktake-count-runtime-b1.md)）で 73・55・51 の在庫連動の拒否・60・65 の棚卸しの記録の詳細・75 の共有 JAN の一覧が入った。50・51・58・66 の範囲の表示と要再確認の案内、65・66 の計数の行は b2（[Plan Packet](../plans/2026-10-23-stocktake-count-runtime-b2.md)）で入った。新方式の操作の契約は [73](../function-design/73-ui-stocktake.md)（検索 → 数を入れて即保存、日付の表示と前の日の分、要再確認の一覧と入れ直し、完了の拒否の一覧と「0 にする」）、[55](../function-design/55-ui-csv-import.md)（Z004 の preview の拒否と知らせ、取込みの結果の要再確認の一覧）、[65](../function-design/65-inventory-record-traceability.md) / [66](../function-design/66-ui-stock-movements.md)（計数の行を履歴に混ぜる、今の数を入れ直す）を正とする。[51](../function-design/51-ui-product-form.md) / [58](../function-design/58-ui-stock-inquiry.md) は要再確認の在庫を範囲で出し、51 / [60](../function-design/60-ui-product-import.md) は共有 JAN と在庫連動を始めるときの拒否を示す。

未保存の入力だけを画面の状態に置き、保存済みの計数と写しは DB / query から取り直す。要再確認の理由を UI で推測せず、BIZ が返す型付きの理由と次の操作を示す。日本語と非色シグナルで表示する。既存の route / returnTo、IME、focus とページの器を保つ。b1 の Windows L3 は b1 の packet の台本で行う（b2 の分は未実施）。旧 ADR（時点証拠）の begin・保留・準備照会・非連動化の未調整の案内は置き換えた。

> **親文書**: [ARCHITECTURE.md](../ARCHITECTURE.md)

UI層の仕様は [画面設計書](../SCREEN_DESIGN.md) と各function-designを正本とする。現行の到達・戻り経路は [画面遷移図](../diagrams/current-system.md)、`screen_mockups.html` は初期提案の歴史資料。ここでは各UIタスクの「状態管理」「CMD呼び出しパターン」「利用者操作フロー」を記述する。

---

### UI-12: 共通レイアウト

**タスク要求**: 全画面で共有するサイドバー＋メインの2カラム構成を提供する

**【状態管理】**
- 現在のアクティブ画面（サイドバーのハイライト制御、TanStack Router の `<Link activeProps>` で表現）
- グローバル通知（操作成功/警告/エラーのトースト表示、Sonner）
- ウィンドウタイトル（ルート遷移に追従して `在庫管理システム - <画面名>` 形式で更新）

**【構成】**
- サイドバー: 画面遷移ナビゲーション。**4 エリア分類（使用頻度基準）**
  - 毎日の業務: ホーム / 売上データ取込み / 日次売上 / 在庫照会 / 月次売上
  - 商品管理: 商品検索・一覧 / 商品登録 / 一括インポート / PLU書出し / 一括価格改定
  - 入出庫: 入庫記録 / 返品・交換 / 手動販売出庫 / 廃棄・破損 / 入出庫履歴 / 在庫少一覧 / 棚卸し
  - D-138（proposed・未実装）の後: 毎日の業務の「在庫照会」を「商品と在庫」（`/products`）にし、商品管理から「商品検索・一覧」を外し、入出庫の「在庫少一覧」を「在庫切れ・在庫少」にする（[50](../function-design/50-ui-product-list.md) UI-01a-D14）
  - システム管理: バックアップ・復元 / 操作ログ / 在庫少の基準 / 整合性検証 / 取引先管理
- 項目の実装上の正本は `src/config/navigation.ts`。表示数を別の固定値として管理しない。
- エリア識別はアイコン + 区切り線で行う（色分けは廃止、[../design-system/00-foundations.md](../design-system/00-foundations.md)「4色エリアモデルの扱い」準拠）
- メインエリア: 選択された画面コンポーネントを Outlet で描画
- 通知エリア: Sonner Toaster で右下表示（3 秒で自動消去）

**【設計判断の出典】**
- 4 エリア確定 + 色分け廃止 + 各画面のエリア配置: [docs/plans/ui-12-design-agreement.md](../archive/plans/2026-04-21-ui-12-design-agreement.md) §1.1 / §1.2 / §7
- ウィンドウタイトル機構: 同 §5.6
- 関数設計（コンポーネント分割・型定義・処理フロー）: [function-design/52-ui-shared-layout.md](../function-design/52-ui-shared-layout.md)

### UI-shortcuts: ショートカット一覧ダイアログ

**タスク要求**: グローバル Ctrl+/ 押下でショートカット一覧ダイアログを開閉できる、純フロントエンドのオーバーレイ機能を提供する

**【状態管理】**
- ダイアログの開閉状態（`useState<boolean>` を `useShortcutsDialog` 内に閉じ込め、RootLayout から Dialog の props として渡す）
- 登録済みショートカット定義（`src/features/shortcuts/data.ts` の `SHORTCUTS` 定数が SSOT。本 PR では `global.show-shortcuts` 1 件のみ、各画面 PR でこの配列に追記して拡張）

**【CMD呼び出し】**
- 該当なし（純フロントエンド、IO / CMD / BIZ 層触らない、Tauri ネイティブ API も使わない）

**【利用者操作フロー】**
1. Ctrl+/ 押下 → ダイアログ開く（再押下でトグル閉じる）
2. Esc / オーバーレイクリック / 右上 × → 閉じる（Radix Dialog 標準）
3. Tab / Shift+Tab → Dialog 内 focus 順次 / 逆順移動（Radix focus trap、Dialog 外に脱出しない）
4. ダイアログ内に「グローバル」「このページ」の 2 section、各 section は `<kbd>` 要素 + 説明テキストの table 形式

**【除外条件】**
- IME composition 中の keydown は除外（`event.isComposing === true || event.keyCode === 229`）。日本語入力中（商品名 / 取引先名 / 検索ボックス等）の誤発火防止
- input / textarea / contenteditable focus 中の Ctrl+/ は除外（フォーム入力中の誤発火防止）
- `event.preventDefault()` を呼びブラウザ既定キーバインド（Firefox の Quick Find 等）と衝突抑止

**【設計判断の出典】**
- スコープ確定（Ctrl+/ のみ、Ctrl+K / Alt+←/→ は Backlog）: [docs/archive/plans/2026-05-12-phase-2-ui-shortcuts.md](../archive/plans/2026-05-12-phase-2-ui-shortcuts.md) §Scope 確定
- ショートカット仕様 SSOT: [UI_TECH_STACK.md §5.2](../UI_TECH_STACK.md)
- 関数設計（コンポーネント分割・型定義・キーボード処理仕様）: [function-design/54-ui-shortcuts.md](../function-design/54-ui-shortcuts.md)

### UI-00: ホーム画面

**タスク要求**: アプリ起動時およびメイン画面として、その日の状況把握と毎日機能への入口を提供する

**【状態管理】**
- 昨日の売上サマリ（金額、点数、取引件数）
- 在庫切れ件数・在庫少件数（廃番除外）
- PLU未反映件数（plu_dirty=1 の商品数）
- 最終CSV取込み日（前日分が未取込みの場合は警告表示）

**【CMD呼び出し】**
- 画面表示時 → CMD-09 get_daily_sales（昨日）+ CMD-06 list_low_stock（在庫少） + CMD-08 list_plu_dirty（PLU通知） + CMD-07 list_csv_imports（最終取込み日）
- 大ボタン押下 → 各機能画面へ遷移

**【利用者操作フロー】**
1. 起動/ホーム復帰時にサマリ表示
2. PLU未反映件数が1以上なら黄色の通知バーを最上部に表示
3. 大ボタンで毎日機能（CSV取込み/売上レポート/在庫照会/商品管理）へ遷移（モックアップ準拠、月次売上はサイドバー経由、商品管理は Phase 3 UI-01a まで pending disabled）。D-138 の後は在庫照会・商品管理の 2 枚を「商品と在庫」「在庫切れ・在庫少」にする（[53](../function-design/53-ui-home.md) UI-00-D13）
4. 中段に入出庫4機能、下段にたまに使う機能（棚卸し/バックアップ/設定）

**【制御構造】**
- サマリ取得失敗時は各カードに「取得失敗」表示、他カードは動作継続（部分障害許容）
- PLU通知バーはクリックで UI-08 PLU書出し画面へ遷移

### UI-01a: 商品検索・一覧

**タスク要求**: 商品の検索・絞込み・並替え・一覧表示（PLU の移行状態の列・絞り込み・一括更新は D-117 で撤去）。D-138 で UI-06a と 1 つの画面「商品と在庫」（`/products`）にする（[50](../function-design/50-ui-product-list.md) UI-01a-D14〜D19、proposed・未実装）

**【状態管理】**
- 検索条件（keyword, department_id, is_discontinued, sort_key, sort_order）
- ページング状態（current_page, per_page, total_count）
- 商品一覧データ

**【CMD呼び出し】**
- 画面表示時 / 検索実行時 / ページ遷移時 → CMD-01 search_products
- 商品行クリック → CMD-01 get_product → UI-01bへ遷移

**【利用者操作フロー】**
1. 検索バーにキーワード入力（商品名/商品コード/JANコード/メーカー品番、SPEC-PRV-D2）
2. 部門フィルタ、廃番表示ON/OFF
3. 一覧表示（商品コード/商品名/売価/原価/在庫数/部門/廃番状態）。原価は売価の右隣（UI-01a-D13）
4. 行クリックで商品編集画面へ遷移
5. 「新規登録」ボタンでUI-01bへ遷移（新規モード）

> **runtime の状態**: 上は D-117 の後の形（採用済み）。今の build は、一覧に「PLU」の列（`src/features/products/components/ProductTable.tsx` の `ProductTable`。対象外 / 未反映 / 反映済み の Badge）を持ち、`src/features/products/ProductListPage.tsx` に PLU の絞り込み（URL の `plu`）と、絞り込みに一致する全件の一括更新（`commands.bulkSetPluTarget`、CMD-01 の `bulk_set_plu_target`）がある。外すのは PLU の書出しの見直しの runtime（[33](../function-design/33-biz-plu-export-service.md) §16 の冒頭の「runtime の状態」の注記）。

### UI-01b: 商品登録・修正

**タスク要求**: 商品情報の入力・保存

**【状態管理】**
- モード（'create' / 'edit'）
- フォームデータ（全フィールド）
- バリデーションエラー
- 部門リスト、取引先リスト（サジェスト用）

**【CMD呼び出し】**
- 画面表示時（editモード）→ CMD-01 get_product
- 部門リスト取得 → CMD-01 list_departments
- 取引先リスト取得 → CMD-01 list_suppliers
- 新しい取引先を追加 → CMD-01 create_supplier
- edit mode の価格履歴取得 → CMD-01 list_price_history
- 保存ボタン → CMD-01 create_product または update_product
- 廃番ボタン → CMD-01 toggle_discontinue

**【利用者操作フロー】**
1. 新規: JANコード入力、またはJANなしで独自コード発番対象部門を選択。JANなし + code_prefixなし部門は保存前に止める
2. 必須項目入力（名前、売価、原価、税率、部門）
3. 任意項目入力（取引先、メーカー品番、初期在庫、pos_stock_sync）
4. pos_stock_sync: 長さの単位（stock_unit='m'）なら初期値OFFを提案、利用者が変更可能
5. 保存 → バリデーション → 成功トースト → 一覧に戻る
6. edit は memory No. を表示しない（D-117）。廃番解除で plu_target を自動復帰しない
7. edit は第 5 セクション「価格履歴」で直近 10 件を表示し、「すべて表示」で 100 件を取得する。create では表示しない

> **runtime の状態**: 上は D-117 の後の形（採用済み）。今の build は、edit で「レジメモリNo.」（`plu_memory_no`）を表示する（`src/features/products/ProductFormPage.tsx`）。外すのは PLU の書出しの見直しの runtime（[33](../function-design/33-biz-plu-export-service.md) §16 の冒頭の「runtime の状態」の注記）。

### UI-14: 一括価格改定

**タスク要求**: REQ-105 / REQ-106。値上げリストを見ながら商品を絞り込み、売価・原価を行単位で改定する

**【状態管理】**
- URL state（取引先、取引先未設定を含む、部門、keyword、廃番を含む、sort、page、perPage）
- 商品一覧と各行の新売価 / 新原価（案）入力、行確定状態
- 取引先 complete master data と inline 追加状態

**【CMD呼び出し】**
- 一覧取得 → CMD-01 search_products（keyword は maker_code を含む）
- 取引先候補 / 追加 → CMD-01 list_suppliers / create_supplier
- 行確定 → CMD-01 revise_product_price
- 最近改定の導出 → CMD-01 list_price_history

**【利用者操作フロー】**
1. 取引先・部門・keyword・廃番で絞り込む。取引先指定時は「取引先未設定の商品も含める」を既定 on とし、在庫ゼロ商品も対象にする
2. 現売価をリストの旧売価と突合し、新売価を手入力する
3. 現掛率と整数除算で導出した新原価（案）を確認・修正する
4. 1 行ずつ確定し、supplier_id が未設定なら、既定 off の同時紐付け toggle を on にしたときだけ選択中の取引先で補完する（D-088、詳細は 77 SPEC-PRV-D6）
5. 本日の price_history がある行は icon + 日本語 `最近改定` で示す。再読込で確定前入力が失われる旨を常時明記する

詳細契約は [77-ui-bulk-price-revision.md](../function-design/77-ui-bulk-price-revision.md) を正とする。

### UI-01c: 商品一括インポート

**タスク要求**: CSVファイルからの商品一括登録

**【状態管理】**
- インポート段階（'select_file' / 'preview' / 'result'）
- プレビューデータ（valid_rows, error_rows, duplicate_rows）
- 重複行の上書き/スキップ選択状態

**【CMD呼び出し】**
- ファイル選択後 → CMD-01 preview_import
- 「取り込む」ボタン → CMD-01 commit_import

**【利用者操作フロー】**
1. CSVファイルを選択
2. プレビュー表示（先頭数行＋エラー行＋重複行）。任意列 `PLU対象` と 13 桁でない JAN（空か 8 桁）の warning / 対象外補正を text で示し、色だけに依存しない
3. 重複行ごとに「上書き」「スキップ」を選択
4. 「取り込む」で確定 → 結果サマリ表示

### UI-02〜05: 入出庫系画面群

4画面とも共通パターン: 「ヘッダ入力→明細行追加→保存」。

**【共通状態管理パターン】**
- ヘッダデータ（日付、取引先等）
- 明細行リスト（商品選択済み/未選択の行）
- 商品検索ポップアップの表示状態

**【共通CMD呼び出しパターン】**
- 商品選択時 → CMD-01 search_products（ポップアップ内検索）
- 保存ボタン → CMD-02〜05 の各createコマンド

**画面固有の差分:**

| 画面 | ヘッダ固有項目 | 明細固有項目 | 特殊UI |
|------|-------------|-----------|--------|
| UI-02 入庫 | 取引先（サジェスト選択） | 数量, 入庫時原価 | なし |
| UI-03 返品 | 種別（返品/交換）、レジ戻し済み？ | 方向（in/out）、数量 | レシート画像添付 |
| UI-04 手動販売 | 理由選択 | 数量, 金額 | PLU登録済み警告表示 |
| UI-05 廃棄 | なし | 種別、数量、原価、理由 | なし |

### UI-06a/06b/06c: 在庫照会系画面群

**UI-06a 商品別在庫照会:**
- D-138 で UI-01a と 1 つの画面「商品と在庫」（`/products`）にし、`/stock` は寄せる（proposed・未実装。在庫の状態・在庫切れ・在庫少の一覧・行の詳細の契約は [58](../function-design/58-ui-stock-inquiry.md)）
- CMD-06 get_stock_detail で商品詳細取得
- 商品検索→選択→在庫数・売価・原価・最終入庫日・最終販売日を表示

**UI-06b 在庫少一覧:**
- 独立画面ではなく、UI-06a `/stock` への `status=low_stock` deep-link（D-047、2026-07-16、詳細は [52-ui-shared-layout.md](../function-design/52-ui-shared-layout.md) UI-12-D1）。D-138 の後は「在庫切れ・在庫少」として `/products?status=low_stock` を開き、在庫切れと在庫少を 1 つの一覧に出す
- CMD-06 list_low_stock で閾値以下の商品を一覧取得し、廃番を除いて出す（D-138 の後は「すべて」の状態の欄にも同じ集合を廃番を含めて使う、58 契約 H・廃番除外。今の build は UI-06a 内のフィルタ contract で `list_low_stock(false)` 固定）

**UI-06c 在庫変動履歴:**
- CMD-06 list_movements で商品別の変動一覧取得
- 期間・種別の絞込みフィルタ

### UI-07: 売上データ取込み画面

**【状態管理】**
- 取込み種別（'daily_report' / 'product_sales'）。current operation の既定は 'daily_report'
- 日報取込み段階（状態の一覧は [55](../function-design/55-ui-csv-import.md) UI-07-D15 (5) が正本。SD から読む `scanning`・`sd_list` を含む）。Z001/Z002/Z005の3ファイルbundleを扱う
- 日報プレビューデータ（report_date, source_files, totals, payment_summary, department_summary, warnings, duplicate_check）
- Z004商品別CSV取込み段階（既存実装の 'idle' / 'parsing' / 'preview' / 'importing' / 'result' / 'error'）。PLU確認後の別トラックとして扱う
- インポート実行中フラグ（排他制御用、D-3対応）— Phase 2 8-2 で `useBlocker` 常時 block + 状態バナーに昇格（確認ダイアログ廃止、page unmount 後の state 喪失問題回避）

**【CMD呼び出し】**
- 日報ファイル選択後 → CMD-12 parse_and_validate_daily_report
- 日報「取り込む」ボタン → CMD-12 commit_daily_report_import
- 日報同日別bundleの追加確認時 → 両タブ共通の確認ダイアログ後にcommit（additional_import_confirmed=true）
- 日報完了取消し → CMD-12 rollback_daily_report_import
- Z004商品別CSV選択後 → CMD-07 parse_and_validate_csv（既存トラック）
- Z004商品別CSV「取り込む」ボタン → CMD-07 commit_csv_import（既存トラック）
- Z004商品別CSV完了取消し（取込みの結果の画面の「取り消す」と CSV取込みの詳細の「この取込みを取り消す」の 2 つの入口）→ 共通の確認が CMD-07 preview_rollback_csv_import で取り消した後のようすを読み、確定で CMD-07 rollback_csv_import（55 UI-07-D18）

**【利用者操作フロー】**
1. 既定タブ「日報取込み」で「SD から読む」を押し（CMD-12 scan_register_sd）、読み終えたら SD をレジへ戻す案内と精算の一覧を見て、「取り込めます」の精算を選ぶ（CMD-12 parse_and_validate_daily_report_from_sd、D-111、UI-07-D15）。SD が読めない・過去の分は予備の「ファイルを選んで取り込む」で Z001/Z002/Z005 を 1 つずつ選ぶ（UI-07-D16。dialog の path を `source_path` で送り、SD の上の file なら BIZ-08 が SD の入力として扱う）
2. プレビュー表示（対象日、読み込む3ファイル、総売上/純売上、支払集計、部門別集計、警告）
3. 部門未対応warningは取り込めるが、商品別・在庫引落しには使われないことを表示する
4. 重複チェック結果（同一bundle取込み済み→ブロック、同じ精算を別のファイルから取込み済み→ブロック〈BIZ-08-D4〉、同日別bundle→既存全件summaryを示す追加確認）
5. 「取り込む」で確定 → 実行中はボタンdisabled → 結果サマリ
6. PLU登録後の商品別売上・`pos_stock_sync`在庫増減は「商品別CSV取込み（Z004）」トラックを使う。店舗採取layout Aの選択はIO-02対応後に可能になる

**【制御構造】**
- Preview開始〜Commit完了/キャンセルまでimport_in_progress=true。この間「取り込む」ボタンdisabled
- importing 中の他画面遷移: `useBlocker` で常時 block（確認ダイアログなし）+ 画面上に状態バナー「取込み完了まで他画面に移れません」表示。完了/エラー/idle で unblock
- 状態管理: useReducer + discriminated union（Zustand 不採用、Phase 2 8-2 確定）。IPC channel 不採用（indeterminate spinner + 状態文言、commit 単一 TX のため partial progress は誤認源）
- 日報取込みは sale_records / inventory_movements を作らない。Z004商品別CSV取込みだけが商品別売上を作り、`pos_stock_sync=true`の商品にinventory_movementsを作る（店舗採取layout AはIO-02が二形状対応済み）
- 取消確認は対象import IDを保持し、「この取込みだけを取り消します。同じ日の他の取込みは残ります。」と示す。成功後は同日の残存aggregateと履歴を再取得する

**【設計判断の出典】**
- 状態管理・IPC channel 不採用判定、6 variant 詳細化、useBlocker 常時 block 設計: [docs/archive/plans/2026-05-13-phase-2-ui-07.md](../archive/plans/2026-05-13-phase-2-ui-07.md) §2 確定済の前提
- 関数設計（コンポーネント分割・型定義・reducer 遷移表・状態遷移図）: [function-design/55-ui-csv-import.md](../function-design/55-ui-csv-import.md)
- REQ-401再設計: 2026-06-30 field-check と D-022/D-023/D-025 により、current operation の主動線は Z001/Z002/Z005 日報取込み、Z004はPLU後の商品別トラックとして分離

### UI-08: PLU書出し画面

**【状態管理】**
- 書出しモード（'diff' / 'full'）。既定は 'diff'
- レジ登録状況（未照合 / 最終照合日時。照合は売上データ取込みの SD の読取りで自動、D-117）
- 差分対象商品（CMD-08 list_plu_dirty）
- 書出し準備結果（bytes_base64, suggested_filename, target_product_codes, prepared_rows[memory_no], count, excluded）
- 保存状態（'idle' / 'preparing' / 'save_dialog' / 'saved' / 'confirming_exported' / 'confirmed' / 'error'）

**【CMD呼び出し】**
- 画面表示時 → CMD-08 list_plu_dirty（差分対象の一覧）
- 画面表示時 → CMD-08 get_plu_slot_summary
- 「差分を書き出す」/「全件を書き出す」→ CMD-08 prepare_plu_export
- TSV保存後、利用者が「この書出しを未反映から外す」を明示実行 → CMD-08 confirm_plu_export_saved

**【利用者操作フロー】**
1. 未照合なら売上データ取込みで SD を一度読む導線を先に出す。最終照合日時は above the fold に 1 行で表示する
2. Diff / Full を選び、対象件数、レジの設定を SD に書いてから CV17 で読む案内（Diff / Full とも、UI-08-D5）、保存後に必要なPCツール/SDカード/レジ操作を確認する
3. `prepare_plu_export` でTSVを生成し、Tauri native save dialog で保存先を選ぶ
4. 保存失敗またはキャンセル時は `plu_dirty` を変更せず、再保存・再生成できる
5. 保存成功後、画面上に「PCツールへ投入できない場合はこのまま再書出しできます」と表示する
6. 利用者が保存したTSVをPCツールへ投入する対象として扱うことを確認したら、「この書出しを未反映から外す」で `confirm_plu_export_saved` を実行する
7. 完了画面では更新件数、保存ファイル名、確認日時、未反映解除はアプリ側状態でありレジ反映の証明ではないことを表示する

**【制御構造】**
- TSV生成と未反映解除は分離する。生成成功だけでは `plu_dirty` を落とさない
- `target_product_codes` は prepare 結果から保持し、confirm ではその exact set だけを送る
- prepare は slot 予約を永続化し、再 prepare でも同じ memory No. を使う。confirm は reserved→active を exact memory No. で確定する
- Diff / Full とも CV17 へ投入できる。失敗時はどちらかを書き出し直す（前に保存した file は取り込まない。保存の後の照合で枠が変わりうる）

> **runtime の状態**: 上は D-117 の後の形（採用済み）。今の build の `src/features/plu-export/PluExportPage.tsx` は、利用者が `FilePicker`（`@/components/FilePicker`）で選んだ Z004 を `commands.importPluRegisterSnapshot`（CMD-08 の `import_plu_register_snapshot`）へ渡して占有を照合し、`get_plu_slot_summary` の件数の要約（`free_count`・`release_pending_count` ほか）を出し、`prepare_plu_export` の `over_limit_warning` を `overLimitWarning` に持つ。売上データ取込みの SD の読取りでの自動の照合はまだ無い（CMD-12 の `scan_register_sd` が無い）。操作フロー 1 の最終照合日時と操作フロー 2 の UI-08-D5 の SD の設定書込みの案内もまだ無い（今の画面は Z004 の「最終読込み日時」を出す）。撤去と SD の読取りでの照合への切替えは PLU の書出しの見直しの runtime（[33](../function-design/33-biz-plu-export-service.md) §16 の冒頭の「runtime の状態」の注記）。

### UI-09a/09b: 売上レポート画面群

**UI-09a 日次売上:**
- 日付ナビ（前日 button + `<input type="date">` + 翌日 button、デフォルトは当日、SP-501-02）→ CMD-09 get_daily_sales
- 商品別一覧（6 列 = 商品コード / 商品名 / 部門 / 数量 / 単価 / 金額）+ 部門小計行（grey 帯）+ 総合計
- サマリ 4 カード: 売上合計 / 販売点数 / 売上明細数（`items.length` + 自動・手動内訳、Tooltip「売上レコード行数ベース。レシート単位の取引件数は後続仕様で定義。」）/ 前日比（前日 useQuery、前日 0 円 = 比較不可、当日 fail = 画面全体 Alert）
- 単価列: `Math.round(Math.abs(amount) / Math.abs(quantity))` の派生値（実績単価、quantity=0 は null = 「—」placeholder + ソート時末尾配置）。商品マスタ販売単価ではなく売上記録から算出
- URL state: `?date=YYYY-MM-DD&dept=N&sortBy=...&sortDir=...`（TanStack Router validateSearch + zod 4 直接渡し、不正値は `.optional().catch(undefined)` で undefined fallback）
- 部門フィルタ Select（単一選択、`items` から派生）+ 列ヘッダ click でソート（5 列: product_code / name / quantity / unit_price / amount、部門列は BIZ-05 順固定）
- CSV出力ボタン → CMD-09 export_sales_csv（base64 → Blob ダウンロード + `setTimeout(100)` で `revokeObjectURL` + Sonner 成功/失敗トースト id-based dedup）
- 印刷ボタン: `日報を印刷`。その日のレジの日報を 1 日 1 枚の紙に印刷する（押したときに読み直す。押せない日はボタンの下に理由とリンク・再試行。[56](../function-design/56-ui-daily-sales.md) の UI-09a-D17〜D19、D-120）
- TabsHeader: UI-09b PR #66 で `src/components/sales/TabsHeader.tsx` に共通化、router-driven `<Link>` で日次 (`/reports/daily`) / 月次 (`/reports/monthly`) 切替（disabled 状態廃止）
- 主動線: UI-07 ResultStep「売上レポートを見る」CTA → `navigate({ to: "/reports/daily", search: { date: settlementDate } })`（settlementDate を URL state で渡し、当日 fallback されず取込み済日付のデータを直接表示）、UI-07 commitMutation / rollbackMutation success で `["daily-sales"]` prefix invalidate（UI-09a + UI-00 ホーム両方 refetch）

**UI-09b 月次売上:**
- 月ナビ（前月 button + `<input type="month">` + 翌月 button、デフォルトは当月、SP-502-01）→ CMD-09 get_monthly_sales（1 useQuery、`prev_month_comparison` field を派生表示）
- サマリ 4 カード: 月間売上合計 / 月間販売点数 / 期間表示「YYYY/MM/DD-MM/DD」固定文言（Q-1、営業日数 BIZ 拡張は Backlog） / 前月比（`prev_month_comparison` 派生、空配列 = 「比較不可」表示）
- 部門別テーブル 4 列（部門 / 売上 / 構成比 + `<Progress>` バー / 前月比 + 色分け）、構成比は `compute-composition.ts` 派生、前月比色分け閾値 ±1.0% + prev_amount <= 0 ガード「—」（Q-7、Z004 返品超過月対策）。商品数列は `MonthlySaleItem` DTO に `product_count` field 不在のため非対応（Q-4、Plans.md Backlog 参照）
- 商品ランキング上位 10（SP-502-03、`pick-top-ranking.ts` 派生）+ 1 位黄色バッジ強調（`item.ranking === 1` 追従、BIZ row_number 同順位なし前提）
- モード切替: `?mode=by_product|by_department` URL state（zod 4 enum、validateSearch fallback）、部門フィルタは MonthlySaleItem DTO に部門情報不在のため非対応（Q-4 BIZ 拡張は Backlog、SP-502-05）
- CSV 出力ボタン → CMD-09 export_sales_csv（`useExportFile({ reportType: "monthly_by_product" | "monthly_by_department" })`、Sonner id `export-monthly_by_product-success` / `export-monthly_by_department-success` 等 reportType ラベル付き）
- 印刷ボタン: 仕様未定のため Phase 2 では disabled（aria-disabled + Tooltip + cursor-not-allowed）
- 失敗 4 状態: API fail = 画面全体 Alert / items 空 = 「当月データなし」/ prev_month_comparison 空 = サマリ「比較不可」+ 各行「—」/ 前月だけ fail = 構造的不可能（1 useQuery 設計、Q-5）

**UI-09c レシートの控え（proposed・未実装、D-121）:**
- 決定は [SCREEN_DESIGN](../SCREEN_DESIGN.md) の「レシートの控え画面」（UI-09c-D1〜D5）。BIZ は BIZ-08 §37.11、CMD は CMD-12 §45.6c。function-design の文書は runtime の lane が作る
- 日付と番号で EJ の記録を探し、控えの文を等幅で見せる。読むだけ。欠けを知らせ、EJ のファイルを手で選んで埋められる

### UI-10: 棚卸し画面

**【状態管理】**
- 棚卸し状態（'none' / 'in_progress' / 'completed'）
- 絞込み条件（department_id, counted_only）
- カウント入力中の商品

**【CMD呼び出し】**
- 「棚卸し開始」→ CMD-10 start_stocktake
- 一覧取得 → CMD-10 get_stocktake_items
- カウント入力 → CMD-10 update_count（1件ずつ即保存）
- 「確定」→ CMD-10 complete_stocktake

### UI-11a: 閾値設定画面

**タスク要求**: 在庫少閾値などの運用パラメータを利用者が変更できるようにする

**【状態管理】**
- 現在の設定値（stock_low_threshold / stock_low_threshold_fabric / その他）
- 編集中の新値、未保存フラグ

**【CMD呼び出し】**
- 画面表示時 → CMD-11 get_settings
- 変更保存 → CMD-11 update_setting

**【利用者操作フロー】**
1. 各閾値を数値入力（最小値1、0以下は拒否）
2. 「保存」ボタンで確定、成功トースト
3. 変更は即時反映（次回の在庫少一覧から適用）

### UI-11b: バックアップ画面

**タスク要求**: 手動バックアップ、自動バックアップ設定、過去バックアップからのリストアを提供する

**【状態管理】**
- 既存バックアップ一覧（ファイル名、日時、サイズ）
- 自動バックアップ設定（有効/無効、時刻、保持日数）
- 復元中フラグ

**【CMD呼び出し】**
- 画面表示時 → CMD-11 list_backups + CMD-11 get_settings（バックアップ関連）
- 手動バックアップ → CMD-11 create_backup
- 自動設定変更 → CMD-11 update_setting
- リストア → CMD-11 restore_backup（確認ダイアログ付き）

**【利用者操作フロー】**
1. バックアップ一覧を日時降順で表示
2. 「今すぐバックアップ」ボタンで即時実行
3. リストア時は「現在のデータが上書きされます」確認ダイアログ必須

**【制御構造】**
- リストア実行中は全操作をブロック（画面全体オーバーレイ）
- リストア成功後はアプリ再起動を促すダイアログ表示

### UI-11c: 操作ログ画面

**タスク要求**: operation_logs の閲覧・絞込みを提供する

**【状態管理】**
- 期間フィルタ（開始日・終了日）
- 種別フィルタ（operation_type の選択）
- 種別候補一覧（保持中ログ全体の distinct operation_type）
- ページング状態（current_page, per_page, total_count）
- 取得済みログデータ

**【CMD呼び出し】**
- 画面表示時 / フィルタ変更時 / ページ遷移時 → CMD-11 list_logs
- 画面表示時 → CMD-11 list_log_operation_types（種別フィルタ候補、保持中ログ全体のdistinct）

**【利用者操作フロー】**
1. デフォルトは直近30日・全種別・1ページ目
2. 期間ピッカーと種別ドロップダウンで絞込み
3. 各ログ行の明示的な「詳細を表示」buttonで詳細（payload）を展開表示（JSON整形）。展開中はvisible label / accessible nameを「詳細を閉じる」に変え、native buttonのEnter / Spaceで操作する。行全体clickは使わず、一度に展開する行は1件だけとする。関連記録リンクを押しても展開toggleは起こさない。

詳細な route/URL state、期間 predicate、operation_type registry、detail_json 安全設計、関連記録リンク contract は [function-design/74-ui-operation-logs.md](../function-design/74-ui-operation-logs.md) を正とする。

### UI-13: 在庫整合性検証画面

**タスク要求**: REQ-904 stock_quantity 整合性チェック。BIZ-07 の実行画面を提供する

REQ-403 / SP-403 の POS 部門別売上照合は別の deferred 要求であり、UI-13 に含めない。将来設計では POS 側とシステム側の同日部門集計の数量・金額差と原因調査材料を扱い、自動修正は非スコープとする。

**【状態管理】**
- 実行状態（'idle' / 'running' / 'completed'）
- 検証結果（差異なし / 差異あり商品リスト）
- 補正対象の選択状態（各差異行で「補正する」チェックボックス）

**【CMD呼び出し】**
- 「整合性チェック実行」ボタン → CMD-11 run_integrity_check
- 差異検出時、選択行で「補正確定」→ CMD-11 fix_integrity（写しを記録から求め直した値へ直接更新。inventory_movements への行追加なし、同一TX内 integrity_fix 操作ログ必須 — BIZ-07-D2/D3）

**【利用者操作フロー】**
1. 画面表示時は idle 状態。「整合性チェック実行」ボタンのみ表示
2. 実行中はプログレスバーと処理中メッセージ
3. 差異なし → 緑色の成功メッセージ、「直近の確認日時」を記録
4. 差異あり → 差異商品一覧（商品コード/商品名/違う項目/保存している値/記録から求め直した値）
5. 利用者が補正する商品を選択し「補正を確定」ボタン（文言正本は 75-ui-integrity-check.md 文言表）
6. 確認ダイアログ（在庫数を記録から求め直した値に合わせて補正し、操作ログに記録する旨。文言正本は 75-ui-integrity-check.md 文言表）
7. 確定 → 写しを記録から求め直した値へ直接更新（inventory_movements への行追加なし）+ 同一TX内 integrity_fix 操作ログ必須記録（BIZ-07-D2/D3、D-051）

**【制御構造】**
- チェック実行は重い処理のため、UIは running 状態で他操作を受け付けない（画面内オーバーレイ）
- 差異一覧はページング（100件ごと）
- 補正は個別行選択可能（全差異を一括補正させない。1件ずつ確認させる）

### 更新履歴

| 日付 | PR | 内容 |
|---|---|---|
| 2026-08-16 | PR #79 | SPEC-SDI-D5/D7: 両売上取込みタブの追加確認とper-import取消契約を正本化。 |
