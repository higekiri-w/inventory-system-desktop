## 4. BIZ-01: 商品管理ロジック

### 計数の置き場と在庫の求め直しの契約（D-118。(a) と b1 で実装済み）

[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)（SPEC-STK-CNT-D2・D4・D8）の BIZ-01 の側。旧 ADR（時点証拠）の版・非連動化の記録・`ej_unverified` による有効化の拒否と既定値 false・本番前 preflight は持たない（置き換えた）。(a)（ADR SPEC-STK-CNT-D12）で入ったのは 1 つ目の項目（`ProductUpdates.stock_quantity` を外し、初期在庫の movement〈業務日は登録した TX の PC の今日〉と `recompute_stock`）。共有 JAN と在庫連動を始める検査の項目は b1（[Plan Packet](../archive/plans/2026-10-22-stocktake-count-runtime-b1.md)）で入った。

- 汎用 `ProductUpdates` の `stock_quantity` 欄は外した（[20](20-io-product-repo.md)。production で `Some` を渡す呼出しは無かった）。create と商品一括 import の新規行は、初期数量を初期在庫の movement（`business_date` = 登録日、`phase=0`）として同じ TX で INSERT し、[31](31-biz-inventory-service.md) の `recompute_stock` で写しを書く。商品一括 import の既存商品の上書きは在庫を変えない（今のまま）。
- 共有 JAN（SPEC-STK-CNT-D4）: 不変条件は 1 つ。変更の後に、在庫連動の商品と同じ JAN を持つ別の商品があってはならない。create・update（在庫連動の変更。update は JAN を変えない: `ProductUpdateRequest` に `jan_code` は無く、商品フォームの edit の JAN 欄は読取専用〈BIZ-01-D1 の「edit は jan_code 変更不可」、[51](51-ui-product-form.md) UI-01b-D5〉。update の検査は在庫連動を始める〈false → true〉保存だけで、ほかの欄だけの保存は組を新しく作らないので検査しない〈移行の前からある組の商品の修正も通る。組は [36](36-biz-integrity-check.md) の整合性の画面で知らせる〉）・商品一括 import（新規の行と、JAN と在庫連動を書く上書きの行。JAN が変わる経路は上書きの行だけ）の TX の中で、変更の後の状態でこれを検査し、どの経路でも、どちらの商品を変える場合でも（例: 一括 import の上書きで在庫連動の商品 A の JAN を非連動の C と同じ値に変える、一括 import の上書きで非連動の C の JAN を A と同じ値に変える、商品の修正か一括 import の上書きで C を在庫連動にする）拒否する（JAN の経路の書き分けは b1 の Plan Review round 1 の是正、2026-10-22）（既存の `validation` kind、field `pos_stock_sync`〈create は §4.2 の step 3b、update は §4.4 の在庫連動を始める保存。JAN が他の商品にある新規登録は step 3a の `JanInUse` が先に止めるので、field `jan_code` で返す経路は無い。BIZ-01-D11〉、message「同じ JAN の商品（{同じ JAN を持つほかの商品コード}）があるため、在庫連動にできません」。`ValidationFailedAt`。回復型〈`stocktake_guard`〉には載せない）。商品一括 import では当該行だけを error 行にし、他の正常行は続ける。preview だけの検証で確定を通さない。
- 商品一括 import の判定の状態と確定の拒否（b1 の Plan Review round 2 の是正、2026-10-22。下の在庫連動を始める検査も同じ）: preview は、新規の行（valid_rows）を CSV の順に、DB の今の状態に先の新規の行の変更を当てた状態で判定し（同じ CSV の新規の行どうしで、在庫連動の商品と同じ JAN の別の商品ができる組は、後の行が error 行）、重複の行（上書きの候補）は、DB の今の状態に全ての新規の行の変更を当てた状態で判定する（確定は新規の行の後に上書きの行を処理する。[60](60-ui-product-import.md) の送る行の並び。ほかの重複の行は上書きするかが preview の時点で分からないので当てない。§4.9.2 の preview の判定と同じ）。確定は送られた行の順に、TX の中のその時点の状態（先に処理した行の変更を含む）で同じ検査をし、当たる行が 1 つでもあれば（preview の後に DB が変わったか、上書きを選んだ行どうしが当たった場合だけ起きる）、何も書かずに `BizError::ValidationFailed`（field なし）で確定全体を戻す。message は最初に当たった行を名指す: 「商品 {商品コード} の行: {preview の行の error と同じ文}。読み込みの後に登録か売上が変わったか、上書きを選んだほかの行と重なったため、取込みを止めました（どの行も取り込んでいません）。その行をスキップにするか、ファイルを選び直してもう一度読み込んでください」。形は §4.9 の単位の再検証（BIZ-01-D8）と §4.9.2 の J3・J4・J7 と同じ（行を名指して確定全体を戻し、preview へ導く）で、結果の型 `ImportResult`（TS 名 `ProductImportResult`）と画面の結果は変えない。当たった行だけを飛ばして他を確定する形は採らない（結果の型に行の拒否の一覧を足し、結果の画面を作り直すことになり、確定の後に利用者が何を取り込まなかったかを別の画面で読み直すことになる）。
- 在庫連動を始める（SPEC-STK-CNT-D8）: 既存の商品の false → true は、連動なしで取り込んだ Z004 の売上（`sale_records`、`is_voided=0`、`stock_linked=0`、個数 ≠ 0 か金額 ≠ 0）が全て効く計数に確実に入っている（ADR D3 の判定表で「入る」）ときだけ許す。「入っていない」か「決まらない」売上があるとき、効く計数が衝突しているとき、効く計数が無く連動なしの売上が 1 件でもあるときは `recount_required` で拒否し、message「連動しない間の売上があります。今の数を入れてから在庫連動にしてください」と回復先（[40](40-cmd-product.md) の回復型。進行中の棚卸しにその商品の明細があれば action `active_count`、なければ `count_now`）を返す。効く計数が無い商品は、連動なしの売上が 1 件でもあれば拒否する。在庫連動を始める保存が共有 JAN の不変条件と `recount_required` の両方に当たるときは、共有 JAN の拒否を先に返す（数えても直らない拒否を先に名指す。商品の修正と一括 import の行の両方。b1 の Plan Review round 2 の是正）。true → false は記録も確認も要らない。`pos_stock_sync` の既定値（新規の商品・商品一括 import の省略時の true、DB の `DEFAULT 1`）は変えない。

既存の価格改定・supplier・PLU の枠（D-117 で解放しない。[33 §16.6](33-biz-plu-export-service.md)）・初期在庫の意味は不変。拒否理由と回復先は[共通 error の新契約](40-cmd-product.md)で型付きに渡し、DB / 数量範囲外は従来の失敗経路で TX 全体を戻す。

### 単位と原価の精度の契約（D-113、BIZ-01-D8）

[共通規則](10-common-rules.md) SPEC-UNIT-D1・D5・D10・D11 の BIZ-01 の側。単位・原価の 1/100 円・上限・商品 CSV の単位と初期在庫と原価は現行の契約。

- `stock_unit` は 10 個の code の `ProductStockUnit`（`ball` を含む。`決定（owner、D-113 J1）`）。create の検証は wire の enum のまま。update と商品 CSV の上書きは個数の単位どうしの変更だけを許す（下の 2 つの段）。`roll`・`cm` は開発・試験 DB の作り直しの起点（vB）で一覧から外した（D-119・D-128。共通規則 SPEC-UNIT-D1）。
- （D-119。owner の決定 U2 = b）update と商品 CSV の上書きで、個数の種類の単位どうしの変更だけを許す（共通規則 SPEC-UNIT-D2）。`ProductUpdateRequest` に通常 field の `stock_unit: Option<ProductStockUnit>`（omitted / null は更新しない）を足し、`ProductUpdates.stock_unit` で UPDATE する。検査は §4.4 の手順 2 で（手順 2 の今の検査〈商品名・売価・原価・部門・取引先〉の後）、値を持ち今の単位と違うとき、今の単位と新しい単位がどちらも `stock_unit_kind` = `Count` でなければ `ValidationFailed("在庫単位は個数の単位どうしでだけ変えられます")`。値が今の単位と同じなら、省くのは `stock_unit` の SET と detail_json の key `stock_unit` だけ（通常の UPDATE とログ行は §4.4 の手順 5・6 の今の契約のまま。BIZ-01-D7 の「変更した」の規則）。在庫・明細・価格は変えない（個数の単位の価格の基準数量はどれも 1）。単位の変更は `plu_dirty` の理由にならない（PLU の file は単位を持たない。§4.4 の手順 4 の売価の変更・手順 4b の対象化だけが立てる）。操作ログは BIZ-01-D7 の規則で key `stock_unit` に `{"old": code, "new": code}` を書く（商品 CSV の上書きの経路も同じ形、下の上書きの段）。
- 原価は `cost_price_centi`（1/100 円、`決定（owner、D-113 J2）`。原価の lane、D-128）。create・update・revise_product_price・CSV 取込みの検証は「0 以上、`99999999`（`999999.99` 円）以下」（[共通規則](10-common-rules.md) の「入力の上限と安全な整数の範囲」）。上限を超えたら `ValidationFailed("原価が大きすぎます")`（CSV は preview の行の error）。price_history の原価も 1/100 円。原価を書く操作ログは 2 つで全部: BIZ-01-D7 の商品修正のログと、`revise_product_price` の `product_price_revise` のログ（一括の価格改定と入庫の原価差分のダイアログ〈UI-02-D15〉から書く）。どちらも detail_json の原価の key は `cost_price_centi`、値は 1/100 円の整数（同じ key に円と 1/100 円が混ざると、後から区別できない。ログは書き換えない契約）。原価の 1/100 円は開発・試験 DB を作り直した起点で入れる（D-119、22 §16 の vB）ので、key `cost_price`（円）の記録はその DB に無く、読む側（[74](74-ui-operation-logs.md) §74.8）は `cost_price_centi` だけを知ればよい。
- 売価・原価・初期在庫の上限（共通規則の同じ節）: create・update・revise_product_price・CSV 取込みの売価は 0 以上 `999999`（越えれば `売価が大きすぎます`）、原価は 1/100 円で 0 以上 `99999999`（`MAX_COST_PRICE_CENTI`。越えれば `原価が大きすぎます`）、create と CSV 取込みの初期在庫は 0 以上 `999999999`（越えれば `在庫数が大きすぎます`。`m` の行は cm にした後の値で比べる）。CSV は preview の行の error にし、commit は wire の行を信頼せず TX の中で同じ上限を確かめて、越えれば `ValidationFailed` で TX 全体を戻す。初期在庫の負は、create が `初期在庫は0以上で入力してください`、商品 CSV（preview の行の error と commit）が `初期在庫の値が不正です: '{値}'`。
- 一括インポート（§4.8・§4.9）の任意列 `在庫単位` は、前後の空白を除いた値が 10 個の code か表示の語（`個` `枚` `本` `袋` `箱` `組` `セット` `玉` `丁` `m`）なら code に直す。空は `None`（INSERT で `pcs`、今どおり）。それ以外は preview の行の error `在庫単位が正しくありません: {値}`（commit も新規の行の単位を 10 個の code と確かめる）。vB で外した `cm`・`巻`・`roll` の行の error には、preview と commit の両方で `retired_stock_unit_hint` の直し方を添える（共通規則 SPEC-UNIT-D10）。
- 上書き（§4.9 の `overwrite_codes`）の単位（D-119）: 正規化した `在庫単位` が既存の商品の単位と違う行は、今の単位と CSV の単位がどちらも個数の種類なら単位を `ProductUpdates.stock_unit` で上書きする。種類をまたぐか、どちらかが長さの単位なら行の error `在庫単位は個数の単位どうしでだけ変えられます（登録: {登録の語}、CSV: {値}）`。commit は wire の行を信頼せず、TX の中で同じ規則で再検証し、外れれば `ValidationFailed` で TX 全体を戻す（文の `{値}` は wire の行の code）。wire の行の単位が 10 個の code に無ければ、新規の行と同じ `在庫単位が正しくありません: {code}` で TX 全体を戻す（preview は code に正規化した行だけを返すので、preview を通らない wire にだけ当たる）。空は今の単位のまま。種類をまたいで上書きできると、`m` の商品を `ball` で上書きしたときに在庫の 100 が 100 cm から 100 玉に、過去の入庫の明細（今の単位を JOIN して金額を求める）の意味も変わる。
  - `plu_dirty`: 上書きの行が、上書きで書く列（§4.9 の手順 2 a. の UPDATE が書く JAN・商品名・部門・売価・原価・税率と、CSV が値を持つときだけ書く取引先・メーカー品番・在庫連動・PLU対象）のうち `在庫単位` の他は全て既存の商品と同じ（単位だけの変更）なら `plu_dirty` を変えない（単位の変更は `plu_dirty` の理由にならない、上の update の段と同じ）。それ以外の上書きの行は §4.9 の手順 2 d. のとおり `plu_dirty=1`。
  - 操作ログ: 単位を変えた商品ごとに、同じ TX の中で `product_update` のログを 1 行書き、summary は商品修正と同じ `商品を更新しました: {商品コード}`、detail_json は BIZ-01-D7 の形で key `stock_unit` だけを持つ（`{"stock_unit":{"old":code,"new":code}}`。同じ行で他の列が変わっても、上書きの経路は単位の key だけを書く。他の列の前後は今どおり記録しない）。§4.9 の手順 4 の件数の要約のログ（TX の外）は今のまま。理由: 商品修正の経路と同じ種類・同じ形なので操作ログ画面の読み方が 1 つで済み、TX の中なので単位を変えた commit にだけ必ず残り、商品ごとに元の単位を追える（件数の要約に前後の一覧を入れる案は TX の外で、書けなかったときに元の単位を失う）。
- `初期在庫` は、同じ行の正規化した `在庫単位` が `m` なら m の数として SPEC-UNIT-D3 の入力の規則で cm にする（`25` → 2500、`26.15` → 2615、`25.155` は行の error `初期在庫の値が不正です: '25.155'`）。それ以外の単位と空は今の整数の parse のまま。`在庫単位` が行の error の行は `初期在庫` を読まない（単位が決まらない）。Coordinator が店の事実（台帳 L-214）から決めた（2026-10-08）。`原価` の列は小数 2 桁までの円を `parse_hundredths`（`初期在庫` の m と同じ関数）で 1/100 円にする（SPEC-UNIT-D5・D10。行の error の分け方は D10）。`売価` は整数のまま。
- 在庫計数 Excel の売価・原価は税抜で、アプリの売価は税込（台帳 TD-166）。Excel から作る CSV の換算は初期投入の作業の範囲で、本契約は CSV の値をそのまま読む。

### 4.1 モジュール構成

```
src-tauri/src/
  biz/
    mod.rs
    product_service.rs  -- 商品管理の業務ロジック
```

### 4.2 create_product

**関数要求**: 商品を新規登録する。独自コード発番、バリデーション、棚卸し中の自動追加を含む

**シグネチャ**:
```
fn create_product(conn: &mut DbConnection, req: ProductCreateRequest) -> Result<ProductCreateResult, BizError>
```

**ProductCreateRequest構造体**:
- jan_code: Option<String>（JANスキャン入力。Noneなら独自コード発番）
- name: String
- department_id: i64
- selling_price: i64
- cost_price_centi: i64
- tax_rate: ProductTaxRate（"10" / "8" / "0"）
- stock_unit: ProductStockUnit（10 個の code。共通規則 SPEC-UNIT-D1）
- initial_stock: i64（0なら在庫なし）
- maker_code: Option<String>
- supplier_id: Option<i64>
- pos_stock_sync: bool
- plu_target: bool（D-028: スキャニングPLU書出し対象フラグ。UI-01b が jan_code の 13 桁数字判定から初期値を提案し、利用者が変更した確定値を渡す。pos_stock_sync と同じ「UI で初期値提案 + 変更可」パターン）
- shared_jan_custom_code: bool（BIZ-01-D11。true は「JAN を持ったまま独自コードで登録し、同じ JAN の商品の群に加える」。UI-01b は step 3a の `JanInUse` を受けた確認のダイアログからだけ true を送る〈[51](51-ui-product-form.md) UI-01b-D26〉。jan_code が None なら意味を持たない〈今どおり発番〉）

**ProductCreateResult構造体**:
- product_code: String（発番された場合は独自コード）
- warnings: Vec<String>（step 3a2 で在庫連動を外した商品ごとの知らせ。それ以外は空）

**処理ステップ**:
1. **バリデーション**
   a. name が空文字 → BizError::ValidationFailed("商品名は必須です")
   b. selling_price < 0 → BizError::ValidationFailed("売価は0以上で入力してください")
   c. cost_price_centi < 0 → BizError::ValidationFailed("原価は0以上で入力してください")
   d. tax_rate / stock_unit は wire 境界で `ProductTaxRate` / `ProductStockUnit` として型検査済み
   e. department_idの存在チェック → product_repo::find_department_by_id()。None → BizError::ValidationFailed("指定された部門が存在しません")
   f. supplier_idがSomeの場合 → 存在チェック。None → BizError::ValidationFailed("指定された取引先が存在しません")
   g. jan_codeがSomeの場合 → BIZ-01-D1 の JAN 形式検証（ASCII 数字 8/13 桁 + チェックディジット。違反は ValidationFailed）

**enum 契約化（D-061 / D-064、二層化）**: wire 経路（CMD 境界の request/response）は tax_rate/stock_unit を generated enum で型検査する。一方、file 由来の `ImportRow.tax_rate: String` / `stock_unit: Option<String>` は維持する。file 経路の値域 guard は BIZ が持つ: tax_rate と stock_unit（preview が code か表示の語を code に正規化し、missing / 空文字は `None` のまま保持して INSERT 時に `"pcs"`、一覧に無い値は行の error。BIZ-01-D8）。

**設計判断（BIZ-01-D1、JAN 形式 validation、2026-08-11。適用の経路は 2026-10-27 に BIZ-01-D10 で改めた）**: step 1g として jan_code が Some の場合の形式検証を追加する。適用は手入力 create 経路と商品 CSV の取込み（preview_import の行の検証と commit_import の TX を開く前の再検証。BIZ-01-D10、D-140。判定は下の core validator を共有し、文は経路ごとに持つ）。Z004 の取込み（商品を作らない）と既存 DB 行は対象外。edit は jan_code 変更不可（product_repo 契約）のため対象外。検証対象は受領済みの正規化後 wire 値であり、trim と全角→半角正規化は 51 UI-01b-D17（frontend）が実行済み、BIZ は再正規化しない。検証内容 = ASCII 数字 8 桁または 13 桁、かつモジュラス 10 チェックディジット整合。違反は `ValidationFailed`（文言は 51 UI-01b-D17 の 2 文言と完全一致）。チェックディジット判定は BIZ 所有の core validator（新設 module）が行い、CASIO adapter（io/plu_formatter）の CV17 出力検証関数は共有しない — adapter 詳細を core 契約へ昇格しない（[ARCHITECTURE.md POS Adapter Boundary](../ARCHITECTURE.md) / D-023）。EAN-8 は 7 桁データ部の先頭桁（idx 0）に重み 3 を割り当てる交互配分（idx 偶数 = 3 / 奇数 = 1）で、EAN-13（idx 偶数 = 1 / 奇数 = 3）とは重み配分の偶奇が逆である（既存 adapter 関数のコピー実装禁止）。core validator（BIZ）/ frontend validator（51 の `jan-code.ts`）/ adapter validator（io、不変）は同一 GS1 標準の意図的独立実装とし、golden 独立転記 oracle は 2 profile で拘束する: EAN-13 は三側共通（valid `4901234567887` / invalid `4901234567890`、偶奇反転 mutation の kill case = `4901234567887`）、EAN-8（GS1 公表例 `96385074` / synthetic `49123456`、kill case = `49123456`）は core/frontend の二側のみ（adapter は EAN-13 専用のため置かない）。ISBN-10 は許容しない（本は 13 桁 JAN で登録 = owner 裁定 2026-08-11）。

**設計判断（BIZ-01-D2、plu_target 初期提案判定の二重実装契約、2026-08-11）**: `should_default_plu_target`（一覧 inline 導出と CSV import の初期 plu_target 導出が使用）と frontend `suggestPluTarget`（51 UI-01b-D18）は「ASCII 数字 13 桁のみ true」の同一意味論を持つ意図的二重実装である。BIZ 側の入力 domain は正規化後 wire 値であり trim を行わない（現実装どおり）。実装統合（wire 越え SSOT 化）は行わず、両側に同一ケース表の独立転記 oracle drift-guard test を置いて意味論 drift を機械検出する。

2. **トランザクション開始（rusqlite::Transaction RAII）**
   - `conn.transaction()` で開始。Drop 時に自動 ROLLBACK
   - 実装時の差分: 元の設計ではステップ3だったが、generate_custom_code 内の increment_next_seq が DB を更新するため TX 内に含める必要あり

3. **product_codeの決定**（TX内）
   a. jan_codeがSome で shared_jan_custom_code = false（BIZ-01-D11）:
      - product_repo::find_by_jan_code(J) と product_repo::find_by_product_code(J) で、JAN が J の商品と商品コードが J の商品を集める（J は jan_code の値。商品コードの昇順、同じ商品は 1 つ）
      - 1 件でもあれば → BizError::JanInUse(message)（message は BIZ-01-D11 の形で全ての商品を名指す）。何も書かない
      - 無ければ product_code = jan_code.clone()（JAN が他の商品に無いときは JAN を商品コードにする今の形、repo 外の回答台帳 TD-338）
   a2. jan_codeがSome で shared_jan_custom_code = true（BIZ-01-D11）:
      - generate_custom_code(&tx, department_id) を呼ぶ（後述）→ product_code を取得。jan_code は J のまま持つ
      - product_repo::find_by_jan_code(J) の商品のうち pos_stock_sync = true の各商品（商品コードの昇順。廃番の商品を含む）を、product_repo::update_product（`ProductUpdates { pos_stock_sync: Some(false), .. }`）で在庫連動なしにし、商品ごとに `recompute_stock`（[31](31-biz-inventory-service.md)。在庫連動の切替の TX はその商品を求め直す、ADR SPEC-STK-CNT-D2 の表の「在庫連動の切替」の行。§4.4 の update と同じ）と操作ログ（operation_type = "product_update"、summary = "商品を更新しました: {商品コード}"、detail_json = `{"pos_stock_sync":{"old":true,"new":false}}`。BIZ-01-D7 の形）を書き、warnings に「同じ JAN の商品 {商品コード}「{商品名}」の「POS販売で在庫を減らす」を「減らさない」にしました」を足す（文は商品フォームの画面の語。BIZ-01-D11 の message の形の項）
      - JAN が J の商品が無くても（読んだ後に変わった等）失敗にせず、独自コードで登録する（外す商品は 0、warnings は空）
   b. jan_codeがNone:
      - generate_custom_code(&tx, department_id) を呼ぶ（後述）→ product_codeを取得
      - jan_code = None のまま

3b. **共有 JAN の不変条件**（冒頭の「計数の置き場と在庫の求め直しの契約」の共有 JAN の項、ADR SPEC-STK-CNT-D4）: step 3 の後の状態（step 3a2 で外した後）で検査し、当たれば BizError::ValidationFailedAt（message「同じ JAN の商品（{同じ JAN を持つほかの商品コード}）があるため、在庫連動にできません」、field `pos_stock_sync`）。step 3a で JAN が他の商品に無いことを確かめ、step 3a2 で同じ JAN の商品を在庫連動なしにするので、当たるのは step 3a2 の経路で新しい商品を在庫連動にして送ったときだけ（UI-01b は送らない）

4. **productsにINSERT**
   - NewProduct構造体を構築してproduct_repo::insert_product()を呼ぶ
   - plu_dirty = true, plu_exported_at = None
   - pos_stock_sync = req.pos_stock_sync
   - plu_target = req.plu_target（D-028。plu_target=0 の商品は plu_dirty=true のままでも抽出・通知クエリの plu_target 条件で対象外になるため、plu_dirty の値は分岐しない）

5. **初期在庫の記録**（initial_stock > 0 の場合のみ）
   - inventory_repo::insert_movement() を呼ぶ
   - movement_type = "receiving", quantity = initial_stock, stock_after = initial_stock
   - reference_type = None, reference_id = None（初期投入は特定の入庫記録に紐付かない）
   - note = "初期在庫投入"

6. **棚卸し中チェック**
   - stocktake_repo::find_active_stocktake() を呼ぶ
   - Some(stocktake) → stocktake_repo::insert_stocktake_item() を呼ぶ
     - stocktake_id = stocktake.id, product_code, system_stock = initial_stock, actual_count = None

7. **操作ログ記録**
   - system_repo::insert_operation_log() を呼ぶ
   - operation_type = "product_create"
   - summary = "商品を登録しました: {name} ({product_code})"
   - detail_json = None

8. **COMMIT**

9. **結果返却**
   - ProductCreateResult { product_code, warnings }（warnings は step 3a2 で足したもの。それ以外の経路は空）

**エラーハンドリング**:
- バリデーション失敗 → BizError::ValidationFailed(message)。トランザクション開始前なのでROLLBACK不要
- JAN を他の商品が使っている（step 3a）→ BizError::JanInUse(message)。TX内で、まだ何も書いていない → RAII自動ROLLBACK
- 共有 JAN の不変条件（step 3b）→ BizError::ValidationFailedAt。step 3a2 で外した在庫連動と next_seq の引上げも RAII自動ROLLBACK で戻る
- 重複コード → BizError::DuplicateProductCode(code)。TX内で発生 → RAII自動ROLLBACK。JAN が他の商品にあるときは step 3a が先に `JanInUse` を返すので、起きるのは INSERT の一意制約に当たったとき（step 3a の後に同じ商品コードが入った等）だけ。独自コードは §4.3 が既存の番号を飛ばすため（BIZ-01-D6）、既存の商品に当たって止まることはない
- DB操作失敗（INSERT等）→ RAII自動ROLLBACK → BizError::DatabaseError(DbError)
- 棚卸しアイテム追加失敗 → RAII自動ROLLBACK → BizError::DatabaseError(DbError)

**設計判断（BIZ-01-D11、JAN が他の商品にあるときは知らせ、独自コードで群に加える道を出す、2026-10-27、D-141）**: 新規登録で入れた JAN を他の商品が使っているとき（JAN が同じ商品か、商品コードがその JAN の商品）、黙って JAN を商品コードにせず、`JanInUse` で使っている商品を名指して止める。利用者が独自コードで登録すると選んだら（`shared_jan_custom_code = true`）、独自コードを振り、JAN を持たせ、同じ JAN の在庫連動の商品を同じ TX で在庫連動なしにする（repo 外の回答台帳 TD-349。後から加える色に独自コードを振り、同じ JAN を各色に持たせる形。TD-352 の「群は全色を独自コード」とは下の例外の項だけ違う）。
- message の形: 「この JAN（{J}）はほかの商品が使っています: {商品コード}「{商品名}」、…」。各商品の後ろに、次の印を 1 つの括弧に「・」でつないで付ける（当たらない商品には括弧を付けない）: JAN が J でなく商品コードだけが J なら「商品コードだけ同じ」、在庫連動なら「POS販売で在庫を減らす」、廃番なら「廃番」（この順。例「（商品コードだけ同じ・POS販売で在庫を減らす）」「（POS販売で在庫を減らす・廃番）」）。並びは商品コードの昇順。件数で省かない（群は色の数で、画面のダイアログが全てを出す）。CMD は kind `duplicate`・field `jan_code` で返す（[40](40-cmd-product.md)）。
- 文の語: `JanInUse` の message と step 3a2 の warnings は、商品フォームの画面の語（在庫の節の欄の名「POS販売で在庫を減らす」と値「減らす」「減らさない」、[51](51-ui-product-form.md) の冒頭の `recount_required` の項）で書き、画面に無い「在庫連動」を使わない。読むのは商品フォームだけ（`rg -n 'createProduct' src --glob '!src/lib/bindings.ts' --glob '!*.test.*'` が `ProductFormPage.tsx` と `product-form-request.ts`〈request の組み立て〉だけ、`create_product` の BIZ の呼出し側は `cmd/product_cmd.rs` だけ。2026-10-29）なので、BIZ の文を画面の語にしても他の画面へ波及しない。冒頭の共有 JAN の項の message（「在庫連動にできません」。商品フォームと商品一括 import の両方が読む）は本決定で変えない（[backlog](../backlog.md) の候補）。
- TD-352 からの例外: 群の 1 色目を先に商品フォームで登録すると、JAN がほかの商品に無いので商品コードは JAN（J）になる（TD-338）。2 色目以降を本決定の道で加えても、1 色目の商品コードは J のまま残る（商品コードは主キーで、修正の画面でも読取専用〈[51](51-ui-product-form.md) UI-01b-D5〉。変えると全参照の書換えになる〈[master-tables](../db-design/master-tables.md) の困りそうなケースの SP-102-04 と同じ理由〉）。このため群の中で 1 色目だけが独自コードでなく、TD-352 の「全色を独自コード」の例外になる。既存の商品を独自コードへ振り直す処理は足さない。害: Z004 の照合は `jan_code` だけで行い、売上は群の先頭（商品コードの昇順）に記録し在庫を動かさない（[pos-tables](../db-design/pos-tables.md) の Z004 のマスタ照合、ADR D4）ので、商品コードが J のままでも在庫は黙ってずれない。例外を受け入れるかは owner の確認を待つ（packet の Q3。AI の review〈PR #212 の計画の review round 1〉から出た問い）。
- 在庫連動を外す理由: 同じ JAN の商品はレジの売上でどれが売れたか見分けられず、ADR SPEC-STK-CNT-D4 の不変条件（在庫連動の商品と同じ JAN を持つ別の商品があってはならない）で、新しい商品も既存の商品も在庫連動にできない。外さずに拒むと、利用者は登録の画面を離れて既存の商品の修正で在庫連動を外し、入力をやり直すことになる（D-127 の「その場で直せる」に反する）。true → false は記録も確認も要らない変更（冒頭の節の「在庫連動を始める」の項）なので、登録の確認のダイアログで名指して同意を得たうえで、登録と同じ TX で外す。外した商品は operation log と warnings で名指す。
- 却下 (a) 今の形（JAN を商品コードにし、JAN の商品コードがあるときだけ `DuplicateProductCode`）: 2 色目は商品 CSV でしか入れられない（TD-349 の起点）。JAN が別の商品コードの商品にだけあるときは、知らせも同意も無く、JAN を商品コードにした商品が群に加わる（後から加える色に独自コードを振る形にならない）。
- 却下 (b) 知らせずに自動で独自コードにする: 同じ品を 2 つ登録する誤り（廃番にした商品の登録し直し等）に気づけない。
- 却下 (c) 既存の在庫連動の商品は外さずに拒み、その商品の修正へ案内する: 上の在庫連動を外す理由。owner の判断を待つ選択肢として残す（[decision-log](../decision-log.md) D-141 の Alternatives）。
- 却下 (d) D4 を緩め、既存の商品の在庫連動を残す: レジの売上で既存の商品の在庫だけが減り、別の色が売れても減る（黙ってずれる）。D4 の改訂は店内コードの lane の Revisit（ADR）で、本決定の範囲の外。
- Revisit: 店内コードの lane で共有 JAN の群の扱い（ADR の Revisit）が変わったとき。商品フォームの edit で JAN を変えられるようにするとき（同じ知らせを edit にも置く）。

---

### 4.3 generate_custom_code

**関数要求**: 部門の接頭辞と連番から独自コードを生成する。トランザクション内で呼ばれることを前提とする

**シグネチャ**:
```
fn generate_custom_code(conn: &rusqlite::Transaction<'_>, department_id: i64) -> Result<String, BizError>
// 注: 連番の更新（increment_next_seq）を呼び出し元の TX に含めるため、引数は借りた transaction に型で限る
//     （31 §12.2 の apply_stock_change と同じ形。通常の接続を渡すとコンパイルが止まる）
```

**処理ステップ**:
1. product_repo::find_department_by_id(department_id) → department
2. department.code_prefix が None → BizError::ValidationFailed("この部門は独自コード発番に対応していません")
3. 既存の番号の最大を求める（BIZ-01-D6）
   a. product_repo::list_product_codes_by_prefix(conn, "{code_prefix}-") → codes（部門を問わず全商品から。比較は大文字・小文字を区別する）
   b. 各 code の `{code_prefix}-` より後ろが 1 文字以上の ASCII 数字だけで、i64 として読めるものを番号として数える。それ以外（数字以外を含む・空・i64 を超える桁）は数えない（step 6 の形の文字列にならないため、当たらない）
   c. floor = 数えた番号の最大.checked_add(1)（数えた番号が無ければ 1）。None（最大が i64::MAX）→ BizError::ValidationFailed("この部門の独自コードの番号を振れません")
   d. cand = max(department.next_seq, floor)。cand.checked_add(1) が None（cand が i64::MAX）→ 同じ BizError::ValidationFailed。step 5 の SQL `next_seq + 1` は i64 を超えると SQLite が next_seq を REAL にするため、ここで止める。step 4・5 の next_seq の SQL 更新の前に返すので、商品・next_seq を変えない
4. product_repo::raise_next_seq(conn, department_id, floor)（next_seq が floor より小さいときだけ floor へ上げる。この後の next_seq は cand）
5. product_repo::increment_next_seq(conn, department_id) → seq_num（= cand）
6. code = format!("{}-{:04}", department.code_prefix, seq_num)（例: "HZ-0047"。9999 を超えたら 5 桁以上で続ける。`HZ-9999` の次は "HZ-10000"。owner 決定 2026-10-04 TD-109、master-tables C-1）
7. product_repo::find_by_product_code(conn, &code) で重複チェック
   - Some → BizError::DuplicateProductCode(code)（step 3〜5 により起きないが、安全のため残す）
8. code を返す

step 3〜5 の読み書きは呼び出し元の TX の中で行う。登録が後で失敗すれば next_seq の引上げも戻る（§4.2 のエラーハンドリング）。

**設計判断（BIZ-01-D6、既存の独自コードを飛ばして発番する、2026-10-04、D-106）**: 発番は、同じ接頭辞の既存の独自コードの番号の最大 + 1 と next_seq の大きい方を使う。商品 CSV の取込み（§4.9）は商品コードを CSV のまま入れ、部門の next_seq を進めない。このため `{接頭辞}-NNNN` の形のコードを取り込んだ部門では、旧 step 5 の重複チェックが毎回 `DuplicateProductCode` を返し、同じ TX の rollback で next_seq も戻るので、画面からの独自コードの登録が止まり続けた（Contract Probe、`docs/archive/plans/2026-10-04-custom-code-seq.md`）。取り込んだ番号の間の抜けた番号は埋めず、最大の次から振る（owner 決定 2026-10-04 TD-108: `HZ-0001` と `HZ-0005` なら `HZ-0006`。使わなくなった番号を別の商品に回さない）。番号の読み取り（どの文字列を番号と数えるか）は業務の規則なので BIZ に置き、IO は接頭辞が一致するコードの一覧と next_seq の引上げだけを持つ。対象は全商品で、部門を問わない（商品コードは部門をまたいで一意。別の部門の ID で取り込んだ `HZ-...` も飛ばす）。
- 却下 (a) 空き番号まで 1 つずつ進める（next_seq から順に、使われていない最初の番号を使う）: 新しい repo 関数が要らず最も小さいが、取り込んだ番号の抜けを埋め、owner 決定 TD-108（使わなくなった番号を別の商品に回さない）に反する。補足（推測）: 抜けた番号が店の古い紙の記録に残っていれば、その番号が別の商品を指すようになる。
- 却下 (b) 商品 CSV の取込みで、取り込んだコードの番号に合わせて部門の next_seq を進める: 取込みの経路に発番の規則（コードの形の読み取り）を持ち込む。CSV の部門 ID とコードの接頭辞が違う行（別の部門の ID で `HZ-...`）を扱えず、発番と取込みの 2 か所が next_seq を書くことになり、今後ほかの経路でコードが入るたびに同じ対応が要る。
- 却下 (c) 失敗の文言を変えて、利用者に直してもらう: next_seq を直す画面が無く（`docs/UI_TECH_STACK.md` E6）、利用者が抜け出せない。
- 却下 (d) next_seq の列をやめ、毎回最大値だけから振る: migration が要る（`departments` の再作成か列の放置）。商品は削除しない（廃番は flag）ため結果は本設計と同じで、得るものが無い。
- 却下 (e) 最大値を SQL（`CAST(substr(...))` と `GLOB`）で求める: 番号の読み取りの規則が IO の SQL に入り、i64 を超える桁や空の扱いが SQL の型変換に依る。
- Revisit: 商品の削除（物理削除）を入れるとき（削除された番号の再利用の扱い）。独自コードの形（`{接頭辞}-{4 桁以上}`）を変えるとき。商品が増えて、接頭辞の一覧の読み取りが登録の待ち時間として見えるとき。

---

### 4.4 update_product

**関数要求**: 商品情報を更新する。売価/原価が変わった場合はprice_historyに記録し、売価変更時はplu_dirty=1にする。原価のみの変更では plu_dirty を立てない

**シグネチャ**:
```
fn update_product(conn: &mut DbConnection, product_code: &str, req: &ProductUpdateRequest) -> Result<ProductUpdateResult, BizError>
```

**ProductUpdateRequest構造体**: 全フィールドがOption型
- name, department_id, selling_price, cost_price_centi, tax_rate, maker_code, supplier_id, pos_stock_sync, plu_target
- stock_unit（通常 field。BIZ-01-D8 の個数の単位どうしの検査）

**部分更新wire契約（PRODUCT-PATCH-D1 / REQ-102）**:

| field群 | JSON omitted | JSON null | JSON value |
|---|---|---|---|
| 通常field: `name`, `department_id`, `selling_price`, `cost_price_centi`, `tax_rate`, `pos_stock_sync`, `plu_target`, `stock_unit` | 更新しない | 更新しない | 値を設定する |
| clear可能field: `supplier_id`, `maker_code` | 更新しない | 値をclearする | 値を設定する |

Rustでは通常fieldを `Option<T>`、clear可能fieldを `Option<Option<T>>` とし、後者はmissingとnullを区別するcustom deserializerを維持する。`ProductUpdateRequest` 全体にserde defaultを明示して、tauri-spectaがcommand入力用のgenerated `ProductUpdateRequest_Deserialize`を全property optionalとして生成できるようにする。`ProductUpdateRequest_Serialize` はcommand入力契約ではなく、frontendは `_Deserialize` を直接使う。

**処理ステップ**:
1. product_repo::find_by_product_code(product_code) → existing
   - None → BizError::NotFound("商品が見つかりません")
2. バリデーション（Someのフィールドのみ。create_productと同じルール）
   - stock_unit は上の検査の後に、今の単位との個数の単位どうしの変更だけを許す（BIZ-01-D8）
3. **トランザクション開始（BEGIN）**
4. **価格変更チェック**
   - selling_priceまたはcost_price_centiがSomeで、既存値と異なる場合:
     - product_repo::insert_price_history() を呼ぶ
     - old_selling = existing.selling_price, new_selling = req.selling_price.unwrap_or(existing.selling_price)
     - old_cost_centi, new_cost_centiも同様
   - selling_priceが変わった場合 → updatesにplu_dirty=trueを追加
4b. **plu_target / JAN 遷移チェック**（BIZ-01-D3 / SPEC-PLS-D4、D-117 (1)）
   - plu_target が Some で既存値 0 → 1 に変わる場合 → updatesにplu_dirty=trueを追加（レジ登録が新たに必要になったため）
   - 1 → 0 の場合は PLU の枠を変えない（BIZ-04 を呼ばない。枠は残り、書出しに出なくなる。[33 §16.6](33-biz-plu-export-service.md)）
   - CSV 上書き等の許可された経路で jan_code が変わる場合も、旧 JAN の枠を変えない。変更後の plu_target が 1 なら updatesにplu_dirty=trueを追加（新しい JAN のレジ登録が要るため）
5. **productsをUPDATE**
   - ProductUpdatesを構築してproduct_repo::update_product()を呼ぶ
6. **操作ログ記録**
   - operation_type = "product_update"
   - detail_json = 変更した field ごとの変更前後（BIZ-01-D7）。変更した field が 1 つも無ければ null（ログの行は書く）
7. **COMMIT**

**エラーハンドリング**:
- 商品が見つからない → BizError::NotFound
- バリデーション失敗 → BizError::ValidationFailed
- DB操作失敗 → ROLLBACK → BizError::DatabaseError

**設計判断（BIZ-01-D7、商品修正の操作ログの detail_json、2026-10-06）**: detail_json は JSON object で、変更した field ごとに key を 1 つ持ち、値は `{"old": 変更前, "new": 変更後}` とする。key は ProductUpdateRequest の field 名（`name` / `department_id` / `supplier_id` / `selling_price` / `cost_price_centi` / `tax_rate` / `maker_code` / `pos_stock_sync` / `plu_target` / `stock_unit`）。「変更した」は、request の field が値を持ち（clear 可能 field の JSON null を含む。PRODUCT-PATCH-D1）、更新前の商品の値と違うこと。値を持っても更新前と同じ field は書かない。値の型: `name` は文字列、`maker_code` は文字列か null（clear）、`department_id`・`selling_price`・`cost_price_centi` は整数、`supplier_id` は整数か null（clear）、`tax_rate` は DB と同じ文字列（`"10"` / `"8"` / `"0"`）、`pos_stock_sync`・`plu_target` は真偽値、`stock_unit` は code の文字列（`"pcs"` 等）。部門・取引先は products 表が持つ ID のまま書き、名前を引かない。JSON は serde_json で組み立て、文字列を手で連結しない（商品名の `"` や `\` で壊れた JSON を作らない）。key の順序は契約にしない。読む側は操作ログ画面（74 §74.8 の既知 key 要約と技術情報の raw JSON）。この形より前に書かれた記録（売価・原価の片方だけが変わっても両方を持つ）は書き換えない。
- 却下 (a) 売価・原価のどちらかが変わったら両方を書く今の形を残し、他の field だけを足す: field ごとに規則が違い、読む側が key の有無で「変えた field」を判断できない。
- 却下 (b) request が値を持つ field を、変更の有無に関わらず全部書く: 変えていない値が並び、何を変えたかが読み取りにくい（owner の L3 所感 2026-09-10「更新したのに詳細情報なし」の起点は、何を変えたかが残らないこと）。
- 却下 (c) 変更後の値だけを書く: step 6 の「変更前後」と `db-design/tracking-system-tables.md` §18 の「商品修正なら変更前後のフィールド名・値」に反する。
- Revisit: ProductUpdateRequest に field を足すとき（同じ規則で detail に足す）。操作ログ画面に商品修正専用の表示（名前の解決や「旧 → 新」の整形）を作るとき。

---

### 4.4.1 revise_product_price（SPEC-PRV-D5 / REQ-105）

**関数要求**: UI-14 の行単位確定と UI-02 の入庫原価差分承諾から、売価・原価だけを安全に改定する

```rust
fn revise_product_price(
    conn: &mut DbConnection,
    input: PriceRevisionInput,
) -> Result<PriceRevisionResult, BizError>
```

- `PriceRevisionInput { product_code, new_selling_price: i64, new_cost_price_centi: i64, assign_supplier_id: Option<i64> }`
- `PriceRevisionResult { product_code, changed: bool, plu_dirty_set: bool, supplier_assigned: bool }`

**処理ステップ**:

1. 商品を取得し、不存在は `BizError::NotFound` とする。`new_selling_price`（円）/ `new_cost_price_centi`（1/100 円）は 0 以上の整数だけを受け付ける
2. 売価・原価とも現値と同じなら価格改定は no-op（`changed=false`）。price_history と `product_price_revise` operation log は書かない
3. 変更がある場合は 1 transaction で、価格改定用の部分更新、price_history の old/new 4 値（`old_selling` / `new_selling` / `old_cost_centi` / `new_cost_centi`）、`system_repo::insert_operation_log(operation_type = "product_price_revise")` を書く
4. `plu_dirty` は売価変更時のみ 1 にする。**原価のみの変更では plu_dirty を立てない**（PLU TSV に原価列はない）
5. `assign_supplier_id` が Some の場合も、**supplier_id が NULL のときだけ**設定する。既存の非 NULL 値は上書きしない（SPEC-PRV-D6）。結果の `supplier_assigned` はこの成否を表す
6. 途中失敗は products / price_history / operation_logs の変更をまとめて rollback する

既存 `update_product` の全項目 request を UI-14 から流用せず、この関数が意図した価格 field と漸進的な supplier 紐付けだけを所有する。

---

### 4.5 toggle_discontinue

**関数要求**: 商品の廃番フラグを反転する（BIZ-01-D3 / SPEC-PLS-D4）

**シグネチャ**:
```
fn toggle_discontinue(conn: &mut DbConnection, product_code: &str) -> Result<bool, BizError>
```

**処理ステップ**:
1. product_repo::find_by_product_code(product_code) → existing
   - None → BizError::NotFound
2. new_status = !existing.is_discontinued
3. **トランザクション開始**
4. 廃番化（new_status=true）は `plu_target=0` も同時更新する。PLU の枠は変えない（D-117 (1)）。廃番解除は `plu_target` を自動で 1 に戻さない
5. product_repo::update_product() を呼ぶ。廃番解除時は `plu_dirty` / `plu_target` の既存値を尊重する
6. system_repo::insert_operation_log(operation_type="product_discontinue")
7. **COMMIT**
8. new_statusを返す（trueなら廃番になった、falseなら復帰した）

---

### 4.6 search_products（BIZ層のラッパー）

**関数要求**: 検索条件をIO層に渡して商品一覧を取得する。BIZ層では追加の業務ロジックなし

**シグネチャ**:
```
fn search_products(conn: &DbConnection, query: ProductSearchQuery) -> Result<PaginatedResult<ProductWithRelations>, BizError>
```

**処理ステップ**:
1. product_repo::search_products(conn, &query) を呼ぶ
2. 結果をそのまま返す（DbError → BizError::DatabaseErrorに変換）

---

### 4.7 list_departments（BIZ層のラッパー）

**関数要求**: UI の部門選択候補として、departments 初期データを全件取得する。商品検索結果から候補を派生しない。

**シグネチャ**:
```
fn list_departments(conn: &DbConnection) -> Result<Vec<Department>, BizError>
```

**処理ステップ**:
1. product_repo::list_departments(conn) を呼ぶ
2. 結果をそのまま返す（DbError → BizError::DatabaseErrorに変換）

**設計判断**: UI-01a 商品検索・一覧の部門フィルタは全 21 部門を候補として表示する必要がある。`search_products` の現在ページ `items` から部門候補を作ると、検索条件・ページング・廃番状態で候補が欠けるため採用しない。

---

### 4.7.1 list_suppliers（BIZ層のラッパー）

**関数要求**: UI-01b の取引先選択候補として、取引先マスタ全件を取得する。取引先は任意項目だが、選択候補は current product / current page から派生しない。

**シグネチャ**:
```
fn list_suppliers(conn: &DbConnection) -> Result<Vec<product_repo::Supplier>, BizError>
```

**処理ステップ**:
1. product_repo::list_suppliers(conn) を呼ぶ
2. 結果をそのまま返す（DbError → BizError::DatabaseErrorに変換）

**設計判断（SPEC-PRV-D6）**: 初回 UI-01b で保留した inline 新規取引先作成は、価格改定支援の Design Phase で `create_supplier` として消化する。候補一覧は引き続き complete master data を正とする。

### 4.7.2 create_supplier（SPEC-PRV-D6 / REQ-106）

```rust
fn create_supplier(
    conn: &DbConnection,
    name: String,
) -> Result<product_repo::Supplier, BizError>
```

1. `name` を trim し、空文字は `BizError::ValidationFailed` とする
2. `product_repo::find_or_create_supplier` を呼ぶ
3. 同名が既に存在する場合は新規行を作らず既存行を返す

### 4.7.3 list_price_history（SPEC-PRV-D9 / REQ-102）

```rust
fn list_price_history(
    conn: &DbConnection,
    product_code: String,
    limit: u32,
) -> Result<Vec<product_repo::PriceHistoryEntry>, BizError>
```

- `changed_at DESC, id DESC` で返す
- limit は既定 10・上限 100 とし、超過は 100 に丸める
- `product_code` 不存在時は読取り専用契約として空配列を返す

### 4.7.4 rename_supplier（SPEC-SUP-D3 / REQ-107）

```rust
fn rename_supplier(
    conn: &mut DbConnection,
    supplier_id: i64,
    name: String,
) -> Result<product_repo::Supplier, BizError>
```

1. `supplier_id` の取引先を取得し、不存在は `BizError::NotFound` とする
2. `name` を trim し、空文字は `BizError::ValidationFailed` とする
3. trim 後の name が現在名と同じなら既存 `Supplier` を返し、DB 更新と操作ログ記録は行わない
4. 他の取引先が同じ name を使っている場合は `BizError::ValidationFailed` とし、UI が「統合」を案内できる validation message を返す
5. 1 transaction を開始し、`product_repo::rename_supplier` で name と `updated_at` を現在日時へ更新する
6. 同じ transaction 内で `insert_operation_log(operation_type = "supplier_rename")` を呼び、取引先 ID と変更前後の名称を記録する
7. 全処理成功で commit し、更新後の `Supplier` を返す。途中失敗は rollback する

### 4.7.5 merge_suppliers（SPEC-SUP-D4 / REQ-107）

```rust
struct SupplierMergeResult {
    products_updated: i64,
    receiving_records_updated: i64,
}

fn merge_suppliers(
    conn: &mut DbConnection,
    source_id: i64,
    target_id: i64,
) -> Result<SupplierMergeResult, BizError>
```

1. `source_id == target_id` は `BizError::ValidationFailed` とする
2. source（消す側）と target（残す側）をそれぞれ取得し、どちらかが不存在なら `BizError::NotFound` とする
3. 1 transaction を開始する
4. `product_repo::merge_suppliers` を呼び、products と receiving_records の `supplier_id` を target へ付け替えてから source 行を DELETE する
5. 同じ transaction 内で `insert_operation_log(operation_type = "supplier_merge")` を呼び、source / target の ID・名称と `products_updated` / `receiving_records_updated` を記録する
6. 全処理成功で commit し、`SupplierMergeResult` を返す。途中失敗は 2 UPDATE・DELETE・操作ログをまとめて rollback する

統合は同一実体の重複解消であり、単独削除の代替ではない。成功後の再実行は source 不在として not-found を返し、黙って成功扱いにはしない。

### 4.7.6 list_suppliers_with_usage（SPEC-SUP-D8 / REQ-107）

```rust
struct SupplierWithUsage {
    id: i64,
    name: String,
    product_count: i64,
    receiving_record_count: i64,
}

fn list_suppliers_with_usage(
    conn: &DbConnection,
) -> Result<Vec<SupplierWithUsage>, BizError>
```

1. `product_repo::list_suppliers_with_usage` を呼ぶ
2. IO 内部行を BIZ 所有の `SupplierWithUsage` に変換する
3. name 昇順と 0 件取引先の包含を維持して返す

---

### 4.8 一括インポート: preview_import

**関数要求**: 商品マスタCSVファイルをパースし、各行を正常/エラー/重複に分類したプレビューデータを返す。DBへの書き込みなし

**シグネチャ**:
```
fn preview_import(conn: &DbConnection, file_bytes: &[u8]) -> Result<ImportPreview, BizError>
```

**ImportPreview構造体**:
```
struct ImportPreview {
    valid_rows: Vec<ImportRow>,       // 正常行（新規登録可能）
    error_rows: Vec<ImportErrorRow>,  // バリデーションエラー行
    duplicate_rows: Vec<ImportDuplicateRow>, // product_code が既存の行
}
```

**ImportRow構造体**:
```
struct ImportRow {
    line_no: usize,
    product_code: String,
    name: String,
    department_id: i64,
    selling_price: i64,
    cost_price_centi: i64,
    tax_rate: String,
    stock_unit: Option<String>,     // 省略時 "pcs"
    initial_stock: Option<i64>,     // 省略時 0
    jan_code: Option<String>,
    maker_code: Option<String>,
    supplier_id: Option<i64>,
    pos_stock_sync: Option<bool>,   // 省略時 true
    plu_target: Option<bool>,       // 任意列「PLU対象」。空欄は None。`1` + JAN 不備は Some(false) へ正規化
    warnings: Vec<String>,          // preview warning（`1` + JAN 不備 等）。通常は空。wire では #[serde(default)]（PR #86 gated amendment 1）
}
```

**ImportDuplicateRow構造体**:
```
struct ImportDuplicateRow {
    line_no: usize,
    import_row: ImportRow,
    existing_product_code: String,
    old_jan_question: Option<String>, // proposed・未実装（JAN の別名の runtime の lane、D-122）。前の JAN の問いを出す行だけ Some(前の JAN)。§4.9.2
}
```

**OldJanAliasAnswer構造体**（proposed・未実装、D-122。`commit_import` の入力。Deserialize）:
```
struct OldJanAliasAnswer {
    product_code: String, // 問いに「ある」と答えた上書きの行の商品コード
    old_jan: String,      // その行の `old_jan_question` の値（問いで見せた前の JAN）をそのまま返す
}
```

**処理ステップ**:

1. **入力バリデーション**
   - file_bytes が空 → BizError::ValidationFailed("ファイルが空です")
   - この空入力条件はBIZ-01だけが所有する（**BIZ-01-VAL-D1**）

2. **IO-03 呼出し**
   - io::product_csv_importer::parse_product_csv(file_bytes) → ImportParseResult
   - Err → BizError::ImportError(メッセージ)

3. **ヘッダ検証**（必須列の確認）
   - 必須列: "商品コード", "商品名", "部門ID", "売価", "原価", "税率"
   - 不足 → BizError::ImportError("必須列が不足しています: {不足列名}")
   - 任意列 `PLU対象` は `1` / `0` / 空欄だけを受理する。それ以外は行 error とし、IO-03 は raw 値の parse だけを担う（BIZ-01-D4 / SPEC-PLS-D6）

4. **各行のバリデーション**
   - 商品コード: 空でないこと。空でなければ 100 文字（UTF-16 code unit）以内であること。超過は「商品コードは100文字以内で入力してください」（BIZ-01-D5）
   - 商品名: 空でないこと
   - 部門ID: 整数変換可能、departments に存在すること
   - 売価: 0以上の整数。原価: 小数 2 桁までの円（SPEC-UNIT-D10 の `原価` の行の error の分け方）
   - 税率: '10', '8', '0' のいずれか
   - 商品コードと JANコードの形（BIZ-01-D10）: 指数の形の `商品コード`・`JANコード`、13 桁の数字の `商品コード` で `JANコード` が空の行、`JANコード` が BIZ-01-D1 の判定（ASCII 数字 8 桁か 13 桁で、チェックディジットが合う）を通らない行は行の error。重複の分類（手順 5）より前なので、上書きの候補の行にも当たる
   - `PLU対象=1` かつ JAN が 13 桁数字でない（空か 8 桁。形の誤った JAN は上の BIZ-01-D10 の行の error が先に止める）場合は preview warning を付けて `plu_target=0` に正規化する
   - バリデーション失敗 → error_rows に追加

5. **重複チェック**
   - product_code で product_repo::find_by_product_code(conn, code) を検索
   - 既存あり → duplicate_rows に追加
   - 既存なし → valid_rows に追加
   - （proposed・未実装、JAN の別名の runtime の lane、D-122）全行の分類の後に、duplicate_rows の各行の `old_jan_question` を §4.9.2 の preview の判定で決め、JAN が別の商品の別名に当たる行（valid_rows と duplicate_rows）に §4.9.2 の warning を足す

5. **結果返却**
   - ImportPreview { valid_rows, error_rows, duplicate_rows }

**エラーハンドリング**:
- IO-03 パース失敗 → BizError::ImportError(メッセージ)
- ヘッダ不足 → BizError::ImportError(メッセージ)
- DB読み取り失敗 → BizError::DatabaseError(DbError)

---

### 4.9 一括インポート: commit_import

**関数要求**: プレビュー済みの正常行と上書き対象行をDBに一括登録する

**シグネチャ**:
```
fn commit_import(
    conn: &mut DbConnection,
    valid_rows: Vec<ImportRow>,
    overwrite_codes: Vec<String>,
    old_jan_aliases: Vec<OldJanAliasAnswer>, // proposed・未実装（JAN の別名の runtime の lane、D-122）。前の JAN の問いに「ある」と答えた上書きの行の商品コードと、問いで見せた前の JAN の組。空 = 全て「ない」。§4.9.2
) -> Result<ImportResult, BizError>
```

**ImportResult構造体**:
```
struct ImportResult {
    created_count: usize,
    updated_count: usize,
    skipped_count: usize,
}
```

**処理ステップ**:

0. **商品コードの上限と、商品コード・JANコードの形の再検証**（BIZ-01-D5・BIZ-01-D10）
   - TX を開く前に valid_rows の全行を §4.8 step 4 と同じ判定で調べる（wire から来る行を信頼しない）
   - 1 行でも 100 文字（UTF-16 code unit）を超えれば BizError::ValidationFailed("商品コードは100文字以内で入力してください") を返し、DB を変えない（正常な行も登録しない）
   - 1 行でも BIZ-01-D10 の形に当たれば、valid_rows の順で最初に当たった行の、preview の行の error と同じ最初の文で BizError::ValidationFailed を返し、DB を変えない（正常な行も登録しない）。判定は行の値だけで DB を読まないので、preview を通った行では起きない（preview を通らない wire にだけ当たる）

0b. （proposed・未実装、D-122）**前の JAN の答えの形の検査**: TX を開く前に §4.9.2 の判定表の「TX を開く前」の行（J1・J2）を確かめ、満たさなければその行の error を返し、DB を変えない

1. **TX開始**（conn.transaction()）

2. **各行を処理**（TX内で直接 repo 関数を呼ぶ。create_product / update_product の BIZ 関数は内部でTXを開始するため、ネストTX回避のために呼ばない）
   - valid_rows の各行について:
   a. overwrite_codes に product_code が含まれる → product_repo::update_product で直接更新
   b. overwrite_codes に含まれない → product_repo::insert_product で直接INSERT
   c. 新規登録の場合:
      - 4.3 create_product と同等の処理をインライン実行（独自コード発番なし。CSVに product_code が指定済み）
      - 部門の next_seq は進めない。CSV が `{接頭辞}-NNNN` の形のコードを入れても、画面からの次の発番は §4.3 が既存の番号を飛ばす（BIZ-01-D6）
      - `PLU対象` が `1` / `0` ならその値を適用する。ただし `1` で JAN 不備なら preview warning 済みの `0` を使う。列なし / 空欄は `is_discontinued=0 かつ jan_code が 13 桁数字なら 1、それ以外 0` の既存導出規則を使う
      - initial_stock > 0 → inventory_repo::insert_movement に receiving として記録
      - 進行中の棚卸し → stocktake_repo::insert_stocktake_item に自動追加
   d. 上書き更新の場合: `PLU対象` 列に `1` / `0` がある行だけ plu_target を更新する。列なし / 空欄は既存値を保持する。1→0 と JAN 変更は §4.4 の 4b と同じく PLU の枠を変えない（上書きは今どおり plu_target の値によらず plu_dirty=1。JAN の変更もこれで未反映になる。ただし単位だけを変える行は plu_dirty を変えず、単位を変えた商品ごとに TX の中で `product_update` のログを書く。BIZ-01-D8 の上書きの段、D-119）
   e. （proposed・未実装、D-122）前の JAN の別名: 手順 1 の直後（行を処理する前）に §4.9.2 の判定表の「行を処理する前」の行（J3〜J5）を、全行を処理した後（COMMIT の前）に「全行を処理した後」の行（J6・J7）を、表の順に確かめる。全て満たせば組ごとに `product_repo::insert_jan_alias(old_jan, product_code)` で別名を書く。満たさない行があれば、その行の error（型・field・message は判定表）を返し、TX 全体を戻す。JAN を持った商品の別名は、DB の trigger が手順 a〜c の INSERT / UPDATE の中で消す（BIZ は消さない）

3. **COMMIT**（tx.commit()）

4. **TX外: 操作ログ記録**
   - operation_type: "product_import"
   - summary: "商品一括インポート: 新規{created}件、更新{updated}件"

5. **結果返却**
   - ImportResult { created_count, updated_count, skipped_count }

**エラーハンドリング**:
- TX内でのDB操作失敗 → BizError::DatabaseError（TX自動ロールバック）
- 個別行のバリデーションエラー → preview_import で事前に検出済みのため通常到達しない。商品コードの上限と、商品コード・JANコードの形は step 0 で再検証する（BIZ-01-D5・BIZ-01-D10）
- （D-118 の b1）共有 JAN の不変条件か在庫連動を始める検査に当たる行（preview の後に DB が変わったか、上書きを選んだ行どうしが当たった場合）→ 上の「計数の置き場と在庫の求め直しの契約」の節の形で `BizError::ValidationFailed`（field なし、最初に当たった行を名指す）、TX 全体を戻す
- （proposed・未実装、D-122）前の JAN の答えの形・別名の判定を満たさない → §4.9.2 の判定表の行の error（step 0b の行は DB を変えず、step 2 e. の行は TX 全体を戻す）

**設計判断（BIZ-01-D5、商品コードの長さ上限、2026-09-25）**: 商品コードの上限は 100 文字（owner 決定 2026-09-25）。在庫照会の route search `selected` / `q` の zod `.max(100)` と揃え、「登録できるのに照会で選べない」食い違いを消す。数え方は zod `.max()` と同じ UTF-16 code unit（Rust は `encode_utf16().count()`、定数 `constants::PRODUCT_CODE_MAX_LEN`）。文字数（`chars().count()`）では BMP 外の文字で照会より多く通し、byte 数（`len()`）ではかなで照会より少なく通すため採らない。判定は BIZ に置き、任意の文字列が入る唯一の経路である商品 CSV の取込み（preview_import の行検証と commit_import の TX 前の再検証）で閉じる。手入力の create_product は JAN（BIZ-01-D1）か §4.3 の発番で上限に届かず、update_product は product_code を変えないため判定を置かない。DB CHECK は置かない: `products.product_code` を外部キーで参照する定義が 8 あり、SQLite は CHECK の追加に `products` の再作成が要る一方、上の BIZ の判定で入口が閉じるため。

**設計判断（BIZ-01-D10、商品 CSV の商品コードと JANコードの形、2026-10-27、D-140）**: 商品 CSV の取込みは、黙って入るとレジの売上（Z004）に引き当たらないか、誤ったコードのまま変えられない（SP-102-04）形の行を、preview の行の error で名指して止める（owner の決定、repo 外の回答台帳 TD-421・TD-426）。判定は行の値（IO-03 が前後の空白を除いた後）だけで行い、DB を読まない。preview（§4.8 手順 4。重複の分類より前なので新規の行と上書きの候補の行の両方）と commit（§4.9 手順 0。TX を開く前、wire の行を信頼しない）で同じ判定を 1 か所の関数で使う。

| # | 当たる行 | 行の error の文（`{値}` は CSV の値） |
|---|---|---|
| F1 | `商品コード` か `JANコード` が指数の形（ASCII 数字が 1 つ以上、任意で `.` と ASCII 数字 1 つ以上、`E` か `e`、`+`、ASCII 数字 1 つ以上。例 `4.90123E+12`・`5E+12`）。列ごとに判定し、両方が当たれば 2 つ出す | `{列}が指数の形です: '{値}'。Excel が桁を落として書いた値です。元の Excel でその列を文字列の値にして（TEXT の列）保存し直してください。セルを選んだときの上の欄（数式バー）に E+ が出るなら、元の資料から全部の桁を入れ直してください`（`{列}` は `商品コード` か `JANコード`。直し方は初期投入の手順書の流れ 1 の手順 4 の `TEXT` の列と同じで、普通は列ごとに 1 回で直り、使い手に 1 行ずつ打ち直させない。D-127） |
| F2 | `商品コード` が ASCII 数字ちょうど 13 桁で、`JANコード` が空（列が無い場合を含む） | `商品コードが13桁の数字でJANコードが空です: '{値}'。JANを商品コードにした商品は、JANコードにも同じ値を入れてください` |
| F3 | `JANコード` が空でなく F1 に当たらず、BIZ-01-D1 の core validator（`biz::jan_code::validate`）が長さか文字で外す | `JANコードが8桁か13桁の数字ではありません: '{値}'。先頭の0が落ちていないかを見て、商品のバーコードの数字を元の資料から文字列で入れ直してください` |
| F4 | `JANコード` が空でなく F1 に当たらず、同じ validator がチェックディジットで外す | `JANコードのチェックディジットが合いません: '{値}'。商品のバーコードの数字を元の資料から入れ直してください` |

- 行の errors の中の順: 形の検査の文は全部、`JANコード` を読んだ直後の 1 か所に、上の表の順（F1 の `商品コード`、F1 の `JANコード`、F2、F3、F4）で入れる（F1 の `JANコード` と F3・F4 は排他、F2 と F1 の `JANコード` も排他）。行の error は今と同じく `ImportErrorRow.errors` の文字列で、wire の型は変えない。
- 通るもの（今のまま）: `JANコード` の空（JAN を持たない商品）、独自コードの行（`JANコード` が空でも）、13 桁の数字の `商品コード` に `JANコード` があり F3・F4 を通る行（`商品コード` と同じ値でなくても通る）、先頭 2 の店内コードと共有 JAN（グループコード）でチェックディジットが合う値（GS1 の同じモジュラス 10。実物のバーコードは合う）、12 桁・14 桁の数字の `商品コード` で `JANコード` が空の行（F2 は 13 桁だけ）。12 桁を通す理由: 値だけで「JAN を商品コードにした行」と言えるのは 13 桁ちょうどだけで、12 桁の数字は先頭の 0 が落ちた 13 桁の JAN とほかの数字のコードを値で分けられず、店の今の独自コードに数字だけのものがあるかは確かめていない（8 桁と同じ前提。Revisit）。この形は、当面は初期投入の手順書の流れ 1 の手順 4 の不一致の確かめ（数字だけの `商品コード` が `JANコード` と同じでない行が 0）が拾う（止めるかは下の Revisit で決める）。
- 文を手入力の create（51 UI-01b-D17 と完全一致の 2 文）と分ける理由: CSV の行は Excel で直すので、原因の見当（先頭の 0、Excel の桁落ち）と直す場所（元の資料、文字列で入れる）を名指す。判定（validator）は共有し、文だけを経路が持つ。
- 上書きの行で `商品コード` が 13 桁の数字の商品の JAN を消す行（`JANコード` が空か列が無い）は F2 で止まる（その JAN を商品コードにした商品から JAN を消すと Z004 が引き当たらない）。13 桁の数字でない商品の JAN を消す上書き（意図した契約。上の BIZ-01-D8 の上書きの段の `plu_dirty` の項、§4.9.2 の「JAN を空にする変更」）は止めない。
- 却下: 形の誤りを warning にして取り込む（黙って入る形が残り、初期投入で見落とす。TD-421 の答え b は止める）。行を補う（`JANコード` を `商品コード` から補う、指数の形を数に戻す。指数の形は桁が既に失われ、補った値は誤る。13 桁の独自コードを JAN と見なす誤りも作る）。DB の CHECK（既存の DB 行の互換と `products` の作り直し。master-tables の「jan_code の形式 validation を DB に置かない理由」）。frontend で検査する（preview の行の分類は BIZ が持つ。UI に判定を写さない）。
- 範囲: 本判定より前に入った DB の行（開発・試験 DB）は直さず、その商品を上書きする CSV の行だけが当たる（その行の JAN を直して取り込む）。Z004 の取込みは商品を作らないので対象外のまま。
- Revisit: 8 桁の JAN を商品コードにする形（初期投入の手順書の J4 の a）を採るとき（8 桁の数字の `商品コード` で `JANコード` が空の行も F2 に足すか）。手入力の create は 8 桁の JAN もそのまま商品コードにする（§4.2 の手順 3 の product_code の決定。`test_create_product_req101_accepts_valid_jan_8_and_jan_13` が `96385074`・`49123456` で assert する）ので、J4 の答えと関係なく導入の後に 8 桁の JAN の商品コードの商品ができ、その商品を CSV で上書きして `JANコード` を空にすると JAN が黙って消える（Z004 が引き当たらない。13 桁と違い F2 は止めない）。8 桁の JAN をレジがどう Z004 に書くかが分かったとき（13 桁で `00000` から始まる `JANコード` を止めるか）。店の独自コードに数字だけのものが無いと分かったとき（12 桁の数字の `商品コード` で `JANコード` が空の行〈先頭の 0 が落ちた 13 桁の JAN を商品コードにした行〉も止めるか）。

---

### 4.9.1 bulk_set_plu_target（BIZ-01-D4。撤去の対象、D-117 (2)）

商品一覧の絞り込みに一致する全件を「PLU 対象にする / 外す」段階導入の道具は撤去する。どの商品をレジで売るかは、商品ごとの `plu_target`（UI-01b）と商品 CSV の任意列 `PLU対象`（§4.8、UI-01c）で決める。runtime の lane が code・command・bindings・UI・test と一緒に消す。設計照合 test（`src-tauri/tests/design_compliance_test.rs`）が code の pub fn と突き合わせるので、撤去までは signature だけを残す。

```rust
fn bulk_set_plu_target(
    conn: &mut DbConnection,
    filter: ProductBulkFilter,
    plu_target: bool,
) -> Result<BulkPluTargetResult, BizError>
```

### 4.9.2 前の JAN の別名（BIZ-01-D9、proposed・未実装、D-122）

**関数要求**: 商品の JAN を直すときに「前の番号のバーコードが付いた現物は店にありますか」を 1 回聞き、「ある」と答えた商品だけ、前の JAN をその商品の別名として残す（owner の決定、repo 外の回答台帳 TD-284・TD-286）。「ない」（既定）なら別名を作らず、前の JAN で売れた分は売上の取込みの `unmatched_product` の行エラーで知らせる（[32](32-biz-csv-import-service.md) §15.3 手順 4 b。黙って誤った商品へ付けない）。別名の表と不変条件は [master-tables](../db-design/master-tables.md) §1a。

**別名を作る条件**（3 つ全てを満たす商品だけ、前の JAN を別名にする）:

1. 直す前の JAN（`A`）が空でない。
2. 直す前に `A` を持つ商品がその商品 1 つだけ（`find_by_jan_code(A)` が 1 件）。共有 JAN（同じ JAN を複数の商品が持つ）の商品は、たどる先が 1 つに決まらないので聞かない（共有 JAN は店内コードまで在庫連動の外。owner の決定 TD-296 の 6b、[32](32-biz-csv-import-service.md) の D-118 の確定の段）。
3. 直した後に `A` を持つ商品が無い（同じ取込みの他の行が `A` を持つなら別名を作らない。その行の商品が `A` で引き当たる）。

問いを出すかと別名が成り立つかは分けて決める: 問いは下の「preview の判定」が上書きの選択に依らない部分だけで出し（広めに出る）、別名が成り立つかは下の「commit の判定」が実際の選択で決める。JAN を空にする変更（`A` → なし）も「直す」に入る。PLU の枠の有無・状態（`reserved` / `active` / `external` / 無し）は条件にしない（D-122 (2)。レジがアプリの JAN を別の枠にも持つ観測〈`external`〉があり、アプリはレジの枠を全部は知らない）。

**経路**: 今 JAN を変える経路は商品 CSV の上書き（§4.9）だけ（商品フォームの edit は JAN を読取専用にする。UI-01b-D11）。JAN を変える経路を新しく作るときは、同じ問いの条件・答えの運び方・commit の判定をその経路に入れ、入れるまでは別名を作らない（答えの無い経路の既定は「ない」）。商品が JAN を持つ全ての経路（商品フォームの新規〈§4.2〉、商品 CSV の新規と上書き、将来の経路）で、その JAN の別名は DB の trigger が消す（BIZ は消さない。master-tables §1a）。

**preview の判定**（§4.8 手順 5）: duplicate_rows の行 `r`（既存の商品 `P`）について、`P.jan_code = Some(A)`、`r.import_row.jan_code ≠ Some(A)`、DB で `A` を持つ商品が `P` だけ、新規の行（valid_rows）が `A` を持たない、の全てを満たせば `old_jan_question = Some(A)`、それ以外は `None`。上書きの選択に依らない条件だけで決める（ほかの重複行は、上書きするかが preview の時点で分からないので数えない。問いは広めに出る。ほかの重複行 `Q` の新しい JAN が `A` でも問いは出し、`Q` も上書きにして「ある」と答えた場合は commit の判定表の J6 が扱う）。**ほかの**重複行の上書きの選択から UI が問いの出し入れを判定し直す形は、判定の規則を UI に写すので採らない（判定は BIZ の 1 か所。その行自身の上書きの選択で問いを見せるかは [60](60-ui-product-import.md) UI-01c-D17）。

**preview の warning**（§4.8 手順 5）: 行の JAN が別名の表にあり、その別名の商品が行の商品と違うとき、`ImportRow.warnings` に「JAN {A} は商品 {別名の商品コード} の前の番号として売上をたどっています。取り込むと、この番号の売上はこの行の商品へ入ります」を足す（取込みは拒まない。DB の trigger が別名を消す）。行の商品が別名の商品と同じ（前の JAN へ戻した）ときは出さない。下の J5 はこの warning を利用者に見せた証拠として扱い、画面が 50 行の表示の制限と別に全件を見せる（[60](60-ui-product-import.md) UI-01c-D17）。

**答えの運び方**: UI-01c は `old_jan_question` が `Some` の重複行に問いを出し（既定「ない」）、送る集合を次の 1 式で作って `commit_import` の `old_jan_aliases` に載せる（[60](60-ui-product-import.md) UI-01c-D17 も同じ式、[42](42-cmd-sales-stocktake.md) §22.6）: `{ (r.import_row.product_code, r.old_jan_question) | r ∈ duplicate_rows, r.old_jan_question が Some, r に「上書き」を選んだ, r の答えが「ある」 }`。`old_jan` は問いで見せた値をそのまま返す（UI は書き換えない）。空の一覧は全て「ない」。

**commit の判定**（§4.9 手順 0b・2 e.。判定・error・回復の置き場はこの表だけで、§4.9・[42](42-cmd-sales-stocktake.md) §22.6・[60](60-ui-product-import.md) UI-01c-D17 はこの表を引く）: 下の行を上から順に確かめ、最初に満たさない行の error を返す。組 `(P, A)` は `old_jan_aliases` の 1 つの組（`P` = `product_code`、`A` = `old_jan`）で、組と行は送られた順に見る。J1・J2 は TX を開く前で DB を変えない。J3〜J7 は TX の中で、満たさなければ TX 全体を戻す（preview の後に DB が変わった場合。答えを黙って捨てず、問いで見せていない番号を別名にせず、別名の warning を見せずに別名を消さない〈TD-310 の原則〉）。名指す `{Q}`・`{商品コード}` が複数あるときは valid_rows の順で最初の行の商品。全ての行を満たせば組ごとに `insert_jan_alias(A, P)`。

| # | 時点 | 条件 | 満たさないときの error（型・field・message） | 回復（[60](60-ui-product-import.md) UI-01c-D17） |
|---|---|---|---|---|
| J1 | TX を開く前（手順 0b） | 各組の `product_code` が `overwrite_codes` にあり、`old_jan` が空でない | `BizError::ValidationFailed`（field なし）「前の番号の答えは、上書きする行にだけ付けられます」 | UI が上の「答えの運び方」の 1 式で送る限り起きない。起きたら preview し直す |
| J2 | TX を開く前（手順 0b） | 同じ `product_code` に違う `old_jan` の組が無い（同じ組の重複は 1 つとして扱う） | `BizError::ValidationFailed`（field なし）「前の番号の答えが同じ商品に 2 つあります」 | J1 と同じ |
| J3 | 行を処理する前（手順 2 e.） | 組ごとに、商品 `P` があり、`P.jan_code = A`（問いで見せた前の JAN と DB の今の JAN が同じ） | `BizError::ValidationFailed`（field なし）「商品 {P} の JAN が確認の後に変わりました。もう一度プレビューしてください」 | ファイルを選び直して preview し直す（preview が今の DB で問いを出し直す） |
| J4 | 行を処理する前（手順 2 e.） | 組ごとに、`A` を持つ商品が `P` だけ | `BizError::ValidationFailed`（field なし）「前の番号 {A} を商品 {P} のほかにも持つ商品があります（共有になりました）。もう一度プレビューしてください」 | J3 と同じ（共有になった `A` には問いは出なくなり、`A` の売上は [32](32-biz-csv-import-service.md) §15.3 手順 4 b の既存の共有 / 1 件ヒットの規則に従う） |
| J5 | 行を処理する前（手順 2 e.） | 答えの有無に依らず、取込みの行（valid_rows の新規の行と上書きを選んだ重複行）ごとに、行の JAN `A` が別名の表にあり、その別名の商品 `X` が行の商品と違うなら、行の `warnings` に、`A` と今の `X` で上の「preview の warning」の文を作った文字列と一致する要素がある（preview の後に別の取込みで別名ができた、または別名の商品が変わった行を見つける） | `BizError::ValidationFailed`（field なし）「前の番号 {A} が確認の後に商品 {X} の別名になりました。もう一度プレビューしてください」 | J3 と同じ（preview し直すと warning が出て、その画面で取り込むかを決められる） |
| J6 | 全行を処理した後（手順 2 e.、COMMIT の前） | 組ごとに、上書きを選んだ別の重複行 `Q` が `A` を持たない（実際の `overwrite_codes` で判定する条件 3） | `BizError::ValidationFailedAt { field: "old_jan_aliases", message }`、message「商品 {Q} の行が前の番号 {A} を使うので、商品 {P} の問いは『ない』にしてください」 | UI が preview・答え・上書きの選択を保ったまま message を出し、`P` の答えを「ない」に直すか `Q` をスキップすれば、preview をやり直さずにもう一度確定できる（field `old_jan_aliases` はこの行だけで、増える wire はこの field の値だけ） |
| J7 | 全行を処理した後（手順 2 e.、COMMIT の前） | 組ごとに、J6 のほかに `A` を持つ商品が無い（`P` の行が `A` を直していない、新規の行が `A` を持つ。preview の後に変わった場合だけ起きる） | `BizError::ValidationFailed`（field なし）「前の番号 {A} を取込みの後も持つ商品があります（{商品コード}）。もう一度プレビューしてください」 | J3 と同じ |

UI は回復を field の値で分ける（field `old_jan_aliases` = J6、field なし = ほかの行）。CMD は既存の変換で型と field をそのまま写す（[42](42-cmd-sales-stocktake.md) §22.3・§22.6）。

**そのほかの限界**（受け入れ済み。D-122 の Guarantee range）: 先に別の経路（別の CSV の取込み）で前の JAN を直して「ない」と答えた後に、「ある」にする手段は無い（問いは JAN を直すときの 1 回だけ。別名を消す画面を持たないことと対）。

**共有 JAN との関係**: 別名は D-118 の共有 JAN の不変条件（変更の後に在庫連動の商品と同じ JAN を持つ別の商品があってはならない）の「同じ JAN を持つ別の商品」に数えない。不変条件（どの商品も持たない JAN だけが別名）により、別名で引き当てる JAN を商品が持つことは無いので、引き当てで別名と商品が競らない。

**理由と却下**（詳細は decision-log D-122）: 問いを JAN を変える経路の 1 か所に置き、消すのは DB の trigger に任せる。却下: 前の JAN の枠（`plu_slots`）の列に別名を持たせる（D-117 の round 1 の形。枠の状態の CHECK と照合の遷移に継ぎ目ができる）、JAN を書く経路ごとに別名を消す（経路の挙げ漏れで古い別名が生き返る）、`product_repo` の INSERT / UPDATE の関数の中で消す（`products.jan_code` を別の SQL で書く将来の経路・migration が漏れる）、答えの合わない commit で別名を書かずに続ける（「ある」の答えが黙って消え、その番号の売上が行エラーになる理由が分からない）。

**Revisit**: 商品フォームの edit で JAN を変えられるようにするとき（同じ問いを足す）。別名の一覧・削除の画面が要る事情が出たとき（誤って「ある」と答えた別名を消すには、今はその JAN を持つ商品を登録するか開発者が消す）。売上の引き当て以外（商品検索・棚卸し・入庫のバーコードの読取り）でも前の JAN を引きたい事情が出たとき。

### 4.10 BizError列挙型（Phase 5 拡張）

```
enum BizError {
    ValidationFailed(String),
    ValidationFailedAt { message: String, field: String },
    NotFound(String),
    DuplicateProductCode(String),
    JanInUse(String),                 // BIZ-01-D11: 新規登録で入れた JAN を他の商品が使っている（message は BIZ が作る）
    DatabaseError(DbError),
    ImportError(String),
    IdempotencyConflict(String),
    StocktakeInProgress(String),      // Phase 5 追加: BIZ-06（進行中の棚卸しが存在）
    StocktakeNotInProgress(String),   // Phase 5 追加: BIZ-06（棚卸しが完了済み）
}
```

`ValidationFailedAt` は、業務 validation をBIZ層に一本化したまま
`CmdError.field` を保持する必要がある場合に使う。CMD変換後は
`CmdError { kind: "validation", message, field: Some(field) }` となる。
field を持たない既存 validation は `ValidationFailed(String)` を維持する。

**設計判断 — UncountedItemsExist を設けない理由**: BIZ-06 の complete_stocktake は未入力商品がある場合に `ValidationFailed` を返す（メッセージに件数を含める）。専用バリアントは不要。CMD層は `ValidationFailed` のメッセージ内容でUI表示を分岐できる。
