## 19. BIZ-05: 売上集計ロジック

> **2026-06-30 REQ-401 redesign note**: 既存BIZ-05は `sale_records` 由来の商品別売上を日次/月次レポートとして返す。current operation の公式日報入力は Z001/Z002/Z005 であり、これは `daily_report_imports` / `daily_report_*_lines` 由来の集計データである。BIZ-05は今後、公式日報集計と商品別売上明細を分けて返す。日報集計を `sale_records` に擬似展開しない。

### 数量と単位（D-113）

[共通規則](10-common-rules.md) SPEC-UNIT-D3 の「表示する所」（owner の決定 TD-203: 長さの商品の数量は m、単価は 1 m あたり）の BIZ-05 の側。
- `DailySaleItem` は `stock_unit: ProductStockUnit` を持つ（`quantity` は在庫の数量、長さは cm）。`DeptSubtotal`・`GrandTotal`・`MonthlySaleItem` は数量を `count_points: i64`（個数の種類の数量の和）と `length_cm: i64`（長さの種類の数量の和、cm）の 2 つで持つ。`MonthlySaleItem` の商品別の行はどちらか一方だけが値を持ち、`stock_unit: ProductStockUnit | null`（商品別だけ値）で数量の種類を運ぶ（販売と返品が相殺して `(0, 0)` になった長さの商品の行を `0 m` で出すため。SPEC-UNIT-D11）。集計の読取りを安全な整数の範囲で検査して拒むことはしない（[共通規則](10-common-rules.md) の「入力の上限と安全な整数の範囲」の (d): 1 明細の数量の上限 `9999999` の下では、1 つの集計が `9007199254740991` を越えるのに 900,720,016 件の明細が要る）。CSV（§19.5）は画面と同じ値から書く。
- 部門小計・総合計・月次の部門別は、数量の種類ごとに 2 本に分けて足す（SPEC-UNIT-D3 の集計の規則、owner の決定 TD-206・TD-207）。個数の種類の 9 単位は単位をまたいで `count_points` に足し、`m` は `length_cm` に足す（`pcs` 3・`sheet` 2・`m` 130・`m` 50 の日は `count_points` 5・`length_cm` 180）。種類は `biz::unit_amount::stock_unit_kind` で決める。月次の部門別は、repo が部門と商品の単位で GROUP BY した行を返し、BIZ が部門ごとに種類で分けて足す（金額も。repo は種類を知らず、`SUM(sr.quantity)` を部門だけで足さない）。和は checked で足し、i64 の溢れは `ValidationFailed`（範囲の契約ではなく溢れの扱い）。SQL の `SUM` の溢れは SQLite の error のまま。
- §19.5 の CSV の `数量` の列: 商品の行（日次・月次の商品別）は在庫の数量を表示の単位の数で書く（個数の商品は今と同じ整数、`m` の商品は単位の語と桁区切りを付けない m の数 `1.3`・`1234.56`〈SPEC-UNIT-D3〉）。集計の行（月次の部門別）は、`length_cm` が 0 の部門は整数（`count_points`。区切りも語も付けない `1234`）、0 でなければ画面と同じ文字列（`5 点・1.8 m`・`1.8 m`。`N 点` の `N` は 3 桁区切りで `1,234 点・1.8 m`）。
- レジの日報（公式日報）の数量（D-104）は変えない。
- レジの日報の値（公式日報の総売・純売・支払・部門・日計と、§19.3a の日報の紙）は、parser の上限（[29](29-io-daily-report-parser.md) IO-07-D6）で安全な整数の範囲に収め、読取りで範囲を検査しない（D-142。[共通規則](10-common-rules.md) の「入力の上限と安全な整数の範囲」の (e)）。日次売上の画面と日報の紙は同じ値を同じ書き方で出す。
- テスト: 上限の端の数量の集計が正確に出る。同じ日・同じ部門の長さの商品 A の明細 `9999999`（cm）が 2 件、B の明細 `-9999999` が 1 件 → A の行 `19999998`、B の行 `-9999999`、部門小計と総合計の `length_cm` は `9999999`、`count_points` は 0 で、`get_daily_sales` は成功する。月次の商品別・部門別も同じ形で 1 つずつ。

### 19.1 モジュール構成

```
src-tauri/src/
  biz/
    sales_service.rs  -- 日次・月次売上集計
```

### 19.2 型定義

**DailySalesReport構造体**:

```
struct DailySalesReport {
    date: String,                          // 対象日（YYYY-MM-DD）
    items: Vec<DailySaleItem>,             // 商品別売上一覧
    department_subtotals: Vec<DeptSubtotal>, // 部門小計
    grand_total: GrandTotal,               // 総合計
    official_daily_report: Option<OfficialDailyReportSummary>, // Z001/Z002/Z005日報集計。未取込みならNone
}
```

**DailySaleItem構造体**:

```
struct DailySaleItem {
    product_code: String,
    name: String,
    department_name: String,
    department_id: i64,
    stock_unit: ProductStockUnit, // 商品の単位（表示は単位で決める）
    quantity: i64,       // 売上帳票視点（+販売/-返品）。在庫の数量（長さは cm）
    amount: i64,         // 金額（円）
    source: DailySaleSource, // "auto" / "manual"
}
```

**DeptSubtotal構造体**:

```
struct DeptSubtotal {
    department_id: i64,
    department_name: String,
    count_points: i64,   // 個数の種類の数量の和
    length_cm: i64,      // 長さの種類の数量の和（cm）
    amount: i64,
}
```

**GrandTotal構造体**:

```
struct GrandTotal {
    count_points: i64,
    length_cm: i64,
    amount: i64,
}
```

**OfficialDailyReportSummary構造体（REQ-401 redesign target）**:

```
struct OfficialDailyReportSummary {
    source_import_count: i64,
    report_date: String,
    gross_amount: Option<i64>,
    net_amount: Option<i64>,
    payment_lines: Vec<OfficialDailyPaymentLine>,
    department_lines: Vec<OfficialDailyDepartmentLine>,
    warnings: Vec<String>,
    summary_imports: Vec<OfficialDailySummaryImport>, // Z001の全行を取込みごとに。imported_at ASC, id ASC。合算しない（D-096）
}
```

**OfficialDailySummaryImport構造体**:

```
struct OfficialDailySummaryImport {
    daily_report_import_id: i64,          // UIのkeyと取込みの識別
    imported_at: String,                  // YYYY-MM-DDTHH:MM:SS（daily_report_imports.imported_at）
    lines: Vec<OfficialDailySummaryLine>, // sort_order ASC, id ASC
}
```

**OfficialDailySummaryLine構造体**:

```
struct OfficialDailySummaryLine {
    label: String,          // 保存されたZ001のラベルをそのまま
    quantity: Option<f64>,  // 総売の行の個数（単位の数、小数 2 桁まで。それ以外の行はNone）
    count: Option<i64>,     // 総売以外の行の件数（総売の行はNone）
    amount: Option<i64>,
}
```

- `summary_imports` は `report_date=date AND status='completed'` の親ごとに1件で、件数は `source_import_count` と等しい。`line_key`（総売・純売以外は並び順から作る `summary_N`）は利用者に意味を持たないため返さない。
- `OfficialDailyReportSummary` の field の並びは code の struct と生成 bindings の並びと同じとする。wire の field 列は `src-tauri/tests/import_internal_contract_test.rs` の wire 契約 test が完全一致で固定し、`summary_imports` は既存 7 field（`source_import_count` 〜 `warnings`）の後ろに足す。

**OfficialDailyPaymentLine構造体**:

```
struct OfficialDailyPaymentLine {
    payment_key: String,
    label: String,
    amount: Option<i64>,
    count: Option<i64>,
}
```

**OfficialDailyDepartmentLine構造体**:

```
struct OfficialDailyDepartmentLine {
    department_id: Option<i64>,
    raw_department_name: String,
    normalized_department_name: Option<String>,
    amount: i64,
    quantity: Option<f64>,  // 単位の数（小数 2 桁まで）
    count: Option<i64>,
}
```

日報（Z001 / Z005）の個数は DB と IO の DB DTO では 100 倍の整数（`quantity_hundredths`）、wire（`OfficialDailySummaryLine` / `OfficialDailyDepartmentLine` / `OfficialMonthlyDepartmentTotal` の `quantity`）では単位の数（`f64`）とする。変換は BIZ-05 の写像（`map_official_daily_report` と月次の写像）で IO-07 の `quantity_hundredths_to_units` を呼ぶだけで、ほかの値は変えない（IO-07-D2、D-104）。TypeScript の型は `number | null` のまま変わらず、UI は今までどおり `toLocaleString("ja-JP")` で出す（`1.3` は `1.3`、整数は小数点なし。UI-09a-D16 の「数は `toLocaleString`」の前提を保つ）。件数（`count`）と金額は整数のまま。

**MonthlySalesReport構造体**:

```
struct MonthlySalesReport {
    month: String,                                  // 対象月（YYYY-MM）
    mode: SalesMode,                                // 集計モード
    items: Vec<MonthlySaleItem>,                    // 集計結果
    prev_month_comparison: Option<Vec<MonthlySaleItem>>, // 前月比較（取得できた場合）
    official_department_totals: Option<Vec<OfficialMonthlyDepartmentTotal>>, // Z005日報集計由来。日報未取込みならNone
    official_report_dates: Vec<String>,             // completed の日報取込みのある日（YYYY-MM-DD、昇順、重複なし）。日報未取込みなら空（§19.4 手順 6、57 UI-09b-D11。未実装）
}
```

**MonthlySaleItem構造体**:

```
struct MonthlySaleItem {
    key: String,            // 商品別: product_code / 部門別: department_id の文字列
    label: String,          // 商品別: 商品名 / 部門別: 部門名
    count_points: i64,      // 商品別の行は、長さの商品なら 0
    length_cm: i64,         // 商品別の行は、個数の商品なら 0
    stock_unit: Option<ProductStockUnit>, // 商品別の行だけ値。部門別は None
    amount: i64,
    ranking: u32,           // amount降順のランキング（1始まり）
}
```

**OfficialMonthlyDepartmentTotal構造体（REQ-401 redesign target）**:

```
struct OfficialMonthlyDepartmentTotal {
    department_id: Option<i64>,
    label: String,
    amount: i64,
    quantity: Option<f64>,  // 単位の数（月の SUM を 100 倍の整数で足してから戻す）
    count: Option<i64>,
}
```

**SalesMode列挙型**:

```
enum SalesMode {
    ByProduct,      // 商品別
    ByDepartment,   // 部門別
}
```

---

### 19.3 get_daily_sales

**関数要求**: 指定日の売上データを商品別に集計し、部門小計と総合計を含むレポートを返す。is_voided=0 のレコードのみ対象

**シグネチャ**:
```
fn get_daily_sales(conn: &DbConnection, date: &str) -> Result<DailySalesReport, BizError>
```

**前提条件**: conn は &DbConnection（autocommit）。読み取り専用クエリのためTX不要

**処理ステップ**:

1. **日付バリデーション**
   - date が YYYY-MM-DD 形式でない → BizError::ValidationFailed("日付の形式が不正です（YYYY-MM-DD）")
   - chrono で暦妥当性チェック（2月30日等）→ BizError::ValidationFailed("存在しない日付です")

2. **商品別売上取得と同日active import加算**
   - sales_repo::get_daily_sales_records(conn, date) → Vec<DailySaleItem>
   - SQLは `sale_date=? AND is_voided=0` の全行を対象にし、`product_code + source` でquantity / amountを合計する。Z004の同日複数import由来行をすべて含める
   - `source='auto'` と `source='manual'` は別groupとし、同一商品でも統合しない。表示名・部門はproduct_codeから決定し、結果を部門ID、商品コード、sourceの決定順で返す
   - 0件でも正常（データなしの日）

3. **公式日報集計取得（REQ-401 redesign target）**
   - sales_repo::get_completed_daily_report_aggregate(conn, date) → Option<OfficialDailyReportRow>（IO の DB DTO）を、`map_official_daily_report` で `OfficialDailyReportSummary` へ写す。warning は BIZ がここで作る
   - `report_date=date AND status='completed'` の親を全件対象とし、`source_import_count` に親件数を返す。単一parent IDはwireへ返さない（`summary_imports` の要素の取込み ID は除く、D-096）
   - 親gross/netは合計する。ただし対象親のいずれかがNULLなら集約値もNULLとし、不完全値を確定値に見せない
   - paymentは SALES-SHEET-D2 の規則（位置と名称の組で見分け、1・2 は組で足し、3 は足さない）で足す（D-120・D-142。前の実装は `payment_key` で集約し、並び順から作る `payment_N` が名称の違う行を 1 行に足しえた）。関数は `sales_service.rs` の `apply_sheet_rule` で、画面の支払集計と紙が同じものを呼ぶ。IO は支払の行を足さずに返し（[24 §14.21](24-io-csv-import-repo.md) 手順 3）、BIZ-05 が §19.3a の紙と同じ 1 つの関数で D2 を当てて足す（紙と画面で足し方の実装を 2 つ持たない。D2 の足す値の溢れは紙と同じ `BizError::ValidationFailed`「売上集計の計算でオーバーフローが発生しました」。D-142 の上限の下では届かない）。group の代表（返す `label` と `payment_key`）は group の中の最小 `sort_order`、同値なら最小の行 id の行。返す `payment_key` の意味と wire の型は変えない（同じ `payment_key` の group が 2 つ返りうる。画面は行の key を `payment_key` と配列の index から作る、56）。今と同じ行と値になるのは、その日の全取込みで「位置と前後の空白を除いた名称の組」の集合が同じで、1 取込みの中に同じ `payment_key` の行が無く、各組の保存済みの `payment_key` も全取込みで同じ入力に限る（互換の範囲の正本はこの段。D2 の 3 は組の集合が一致しない日だけなので、この範囲の日には起きない）。ただし範囲の中でも、ある組のある値に NULL があり、取込みの id の昇順で最初の NULL より前の和が溢れる入力は、今は `DbError::QueryFailed`、新しい規則では NULL になる（D2 の「足す値」の (a)）。NULL の無い値の和が溢れる入力は、今は `DbError::QueryFailed`、新しい規則では `BizError::ValidationFailed`（どちらも画面は表示を止める）。範囲の外の入力は新しい規則の値になる: 1 取込みの中に同じ `payment_key` の行が 2 つある入力は、今は 1 行に足され、新しい規則では位置ごとの 2 行に分かれる。同じ組の行の保存済みの `payment_key` が取込みで違う入力（IO-07-D4a の前の parser が `credit` とし、後の parser が `payment_N` とした、同じ位置・同じ名称の行が同じ日にある）は、今は `payment_key` ごとの 2 行に分かれ、新しい規則では組で足した 1 行になる。departmentは `department_id`、未対応行は `normalized_department_name` fallback `raw_department_name` で集約する。amountは合計し、optional quantity/countは対象行のいずれかがNULLなら集約値もNULLとする
   - labelとsortはgroup内の最小 `sort_order`、同値なら最小row IDの行を決定的な代表とする
   - 日報未取込みでも正常。`official_daily_report=None` とし、UIは「日報未取込み」と表示できる
   - SALES2-D5: 集約後の `department_id IS NULL` groupが n 件ある場合、`warnings` に「部門マスタと対応していない部門が n 件あります（部門名のまま表示しています）」を1件だけ追加する。importごとに警告を重複させない。NULL groupがなければ空配列
   - 支払に SALES-SHEET-D2 のどれを当てたか（上の段の関数の結果。1 は `Summed`、2 は `LabelChanged`、3 は `Unsummed`）で、`warnings` に次の 1 件を足す（部門の warning の後。D-120。wire の型は変えない）。2 の日（`LabelChanged`）は「支払集計（Z002）は、この日の精算の間で行の名称か並びが変わっています。同じ名称の行が 2 行あれば、別々に足しています」。3 の日（`Unsummed`）は「支払集計（Z002）は、この日の精算の間で行の名称か並びが変わり、1 回の精算の中に同じ名称の行が 2 行以上あるため、足さずに取込みの古い順に並べています。紙（日報を印刷）で取込みごとに見られます」だけ（2 の文を重ねない）。1 の日（`Summed`）は足さない
   - Z001の行は合算しない。IO が同じ親の集合について親ごとに返す取込み（`OfficialDailyReportRow.summary_imports`）を、並びと値を変えずに `OfficialDailySummaryImport` / `OfficialDailySummaryLine` へ写し、`summary_imports` に取込みの古い順で返す（D-096、[24 §14.21](24-io-csv-import-repo.md#1421-get_completed_daily_report_aggregate)）

4. **部門小計の計算**
   - items を department_id でグルーピング
   - 各グループの quantity を数量の種類ごと（count_points / length_cm）に、amount を合計 → Vec<DeptSubtotal>
   - 部門IDの昇順でソート

5. **総合計の計算**
   - 全 items の quantity を数量の種類ごとに、amount を合計 → GrandTotal

6. **結果返却**
   - DailySalesReport { date, items, department_subtotals, grand_total, official_daily_report }

**設計判断 — 日報集計と商品別明細を分ける**:
- `official_daily_report` はレジ日報の公式集計を表す。
- `items` / `department_subtotals` / `grand_total` はZ004または手動販売出庫に基づく商品別売上を表す。
- Z001/Z002/Z005は商品別明細を持たないため、`items` を水増ししない。
- UIは、日報集計と商品別明細の差を「日報集計」「商品別（PLU/Z004・手動販売）」のように日本語で分けて表示する。
- 公式日報seriesと商品別seriesは別の正本であり、互いを加算して一つの売上値にしない。
- 公式日報seriesの中でも、Z001の行は足してよいと確かめていないため取込みごとに返し、支払・部門のように合算しない（D-096）。日報の紙は §19.3a で Z001 も足す（D-120）。

**エラーハンドリング**:
- 日付形式不正 → BizError::ValidationFailed(メッセージ)
- 暦妥当性エラー → BizError::ValidationFailed(メッセージ)
- DB読み取り失敗 → BizError::DatabaseError(DbError)

**入力例**:
```
date: "2026-03-21"
```

**出力例**:
```
Ok(DailySalesReport {
    date: "2026-03-21",
    items: [
        DailySaleItem { product_code: "4976383262108", name: "ﾊﾏﾅｶ ｱﾐｱﾐ極太", department_name: "毛糸", stock_unit: "ball", quantity: 3, amount: 1782, source: "auto" },
        DailySaleItem { product_code: "HZ-0099", name: "ヘアゴムA", department_name: "ヘア雑貨", stock_unit: "pcs", quantity: 1, amount: 880, source: "manual" },
    ],
    department_subtotals: [
        DeptSubtotal { department_id: 2, department_name: "ヘア雑貨", count_points: 1, length_cm: 0, amount: 880 },
        DeptSubtotal { department_id: 3, department_name: "毛糸", count_points: 3, length_cm: 0, amount: 1782 },
    ],
    grand_total: GrandTotal { count_points: 4, length_cm: 0, amount: 2662 },
})
```

---

### 19.3a get_daily_report_sheet（日報の紙、D-120）

**関数要求**: 指定日のレジの日報を、印刷する 1 枚の紙の内容にして返す。その日の完了した日報の取込みを全部足し、PLU の売上はその日に売れた行だけを付ける。読み取りだけで、何も保存しない。

紙を作る規則（決定の理由と棄却案は [D-120](../decision-log.md)）:

- **SALES-SHEET-D1（1 日 1 枚）**: 紙はその日（`report_date`）の完了した（`status='completed'`）日報の取込みを全部足した 1 枚の内容とする。精算ごとの紙は作らない（repo 外の回答台帳 TD-280）。取り消した取込みは入らない。
- **SALES-SHEET-D2（足し方と保証の範囲）**: Z001・Z002・Z005 のそれぞれ（以下、帳票）で、行を「位置（`sort_order`）と名称（`label` の前後の空白を除いた文字列）の組」で見分け、帳票ごとに次の 3 つのどれか 1 つで載せる。日次売上の画面の支払集計（§19.3 手順 3）も Z002 にこの規則を当てる（紙と画面で足し方を変えず、BIZ-05 の同じ 1 つの関数で足す。D-142）。
  1. その日の取込みの間で組の集合が一致する帳票（取込みが 1 つの日を含む）: 組ごとに足す。
  2. 組の集合が一致せず、どの取込みにも同じ名称（空でない）の行が 2 行以上無い帳票: 組ごとに足し、`label_changed_sources` に入れる。名称の差し替えの日は同じ位置に名称ごとの行が、行の入れ替わり・行の数の違いの日は同じ名称が位置ごとの行が載る（その日の合計は人が見て足す）。
  3. 組の集合が一致せず、どれかの取込みに同じ名称（空でない）の行が 2 行以上ある帳票: 足さず、取込みごとに全部の行を値を変えずに載せ（行の `import_seq` に、その行の取込みが IO の返す取込みの id の並び〈[24 §14.21a](24-io-csv-import-repo.md) の `import_ids`。その日の完了した取込みの `imported_at ASC, id ASC`〉の何番目か〈1 から〉を入れる。その帳票の行の無い取込みも番号を 1 つ使う）、`label_changed_sources` と `unsummed_sources` の両方に入れる。行がずれた日に位置と名称の組でも別の項目を足すのを避ける（例: 取込み 1 の位置 1・2 が同じ名称 A〈10・20〉、取込み 2 で先頭に 1 行入り A が位置 2・3〈30・40〉に来ると、組で足すと位置 2 が取込み 1 の 2 行目と取込み 2 の 1 つ目の A の和 50 になる）。
  - 保証の範囲: 名称のある行で、別の項目の値を 1 行に足すことは無い。1 では、各位置の行は取込みごとに 1 つなので、組の集合が同じなら各位置の名称が全部の取込みで同じ。2 では、各取込みの中で名称が 1 行だけなので、同じ組の行は全部の取込みでその名称の 1 行。3 は足さない。名称が空の行は名称で見分けられず位置だけで見分けるので、2 で行がずれた日には別の枠の値を足しうる（同じ名称の判定に空の名称を含めない。手元の Z 帳票の 2026-09-30 版での集計で名称が空の行は Z002 の 39〜50 行目だけで、全本で値は 0。Contract Probe の P3・P4）。同じ精算を二重に取り込んだ束は二重に足す（D-120 の Guarantee range）。
  - 足す値（1・2）: 組ごと・値（個数・件数・金額）ごとに次の順で決める（紙と画面の支払集計で同じ）。(a) その組の行のどれかでその値が NULL なら、その組のその値は NULL とし、足し算をしない（未取得を 0 に見せない。溢れの検査より NULL が先で、NULL のある値は溢れの error にならない）。(b) NULL が無ければ、組の行を取込みの id（`daily_report_import_id`）の昇順に左から `checked_add` で足し、途中の和が溢れたらそこで止め、紙も画面の支払集計も `BizError::ValidationFailed` を返す（本書冒頭の「数量と単位」の溢れの扱いと同じ。範囲の契約ではない。D-142 の上限の下では届かない）。途中の溢れは足す順に依る（例: 取込みの id の順の金額が `i64::MAX`・`1`・`-i64::MAX` の 3 取込みの組は、この順では 2 つ目で溢れて error、逆の順なら `1`）。取込みの id の昇順にするのは、今の支払の集約（`sales_repo.rs` の `get_completed_daily_report_aggregate`）が group の中の行を読取りの並び（行の `sort_order ASC, id ASC`）のまま足し、§19.3 手順 3 の互換の範囲の日は 1 つの組が取込みごとに同じ位置の 1 行なので、その並びは行の id の昇順で、行の id は取込みの transaction の中で親の後に AUTOINCREMENT で振られるため取込みの id の昇順と同じになるから（取込みの古い順〈`imported_at ASC, id ASC`〉は、取込みの時刻の時計が戻った日に id の順と食い違いうるので、足す順には使わない）。個数は 100 倍の整数のまま足し、最後に IO-07 の `quantity_hundredths_to_units` で単位の数へ戻す（IO-07-D2。3 の行も同じ関数で戻す）。
  - 並び: 1・2 は位置の昇順、同じ位置に名称が 2 つ以上ある帳票は、その位置の中で名称が最初に現れた取込みの古い順。3 は取込みの古い順、各取込みの中は位置の昇順。`import_seq` は 1・2 の行では無い（`None`）。
  - 知らせ: `label_changed_sources` は 2 と 3 の帳票、`unsummed_sources` は 3 の帳票（どちらも Z001 → Z002 → Z005 の順、重複なし。取込みが 1 つの日は空）。紙と印刷ボタンの下の画面の文は UI-09a-D17、日次売上の支払集計の warning の文は §19.3 手順 3 が持ち、どちらも 3 の帳票には 3 の文だけを出す。
  - Z005 は部門マスタと照合せず、レジの名称（`raw_department_name`）のまま見分ける（部門の照合と月次の部門集計は画面の側で、変えない）。
  - 鍵を「位置と名称の組」にする理由: 別の項目を 1 行に黙って足す入力が無い。手元の Z 帳票の 2026-09-30 版での集計（2026-10-04 に数えた、repo 外。数と位置だけ）で、行の数は Z001 28・Z002 50・Z005 21 で全本同じ、Z001 の名称の並びは 1 種類で同日に 2 回以上精算した 19 日でも同じ、Z002・Z005 の変化は同じ位置での名称の差し替えが 1 日（同日 2 回以上の 19 日のうち 1 日）だけ、名称のある行で 1 本の中に同じ名称が 2 行ある例は 0（名称の空の Z002 の 39〜50 行目は位置で分かれる。Contract Probe の P3）。並び順だけの鍵（今の `payment_N`）では、差し替えの日に別の項目を 1 行に足す。名称だけの鍵では、完全一致の同じ名称が 1 本に 2 行あると分けられず、どの位置の行かの知らせも出せない。名称と出現の順の組では、差し替えた先の名称が既存の名称と同じだと黙って足す。
  - 精算をまたいで足してよい根拠: 取扱説明書で日計明細の精算は対象の集計をクリアするので、各精算の行は前の精算から後の分（29 §29.4.1 の「行の構成と性質」）。レジスターツールも、その日の精算を全部足した日報を表示している（店主の答え、TD-280・TD-305。足すことがレジスターツールの仕様かは未確認の前提）。行ごとにレジスターツールの足した値と同じになるかは未確認の前提で、紙へ切り替える前の UI-09a-D19 の (2)（店のアプリの運用の開始の後の実際の 1 日で、アプリの紙と今の Excel の紙を並べる）で確かめる（[D-142](../decision-log.md) の (5)。runtime の lane の Contract Probe の P2。D-120 の Guarantee range の「runtime の lane の L3 で確かめる」の置き場を移した）。
- **SALES-SHEET-D3（載せる行）**: 足した後の全部の組（SALES-SHEET-D2 の 3 の帳票は取込みごとの全部の行。以下この段の「組」はその行も指す）を載せ、値が 0 の行も 0 として載せる。名称が空で、足した値が全部 0 か NULL の組は載せない（Z002 の 39〜50 行目の名称の無い枠。手元の Z 帳票の 2026-09-30 版での集計（2026-10-04 に数えた、repo 外。数と位置だけ）で全本、名称が空で値は 0。今の紙の Z002 の枠もラベルのある 1〜38 行目だけ〈2026-09-29 版の店の Excel の写しでの数、`docs/project-memory.md` の締めの Excel の行の 38〉）。名称が空で値のある組は載せ、名称の表示は画面の側（UI-09a-D17）で決める。値が全部 0 の行を外す形は owner の見た目の判断で変えうる（D-120 の Revisit）。
- **SALES-SHEET-D4（PLU の売上の部分）**: 紙の PLU の部分は、その日の Z004 由来の商品別売上（`get_daily_sales` の手順 2 と同じ集約〈`product_code + source`、同日の全 Z004 取込み〉の `source='auto'` の行）のうち、数量か金額が 0 でない行を、手順 2 と同じ並びで載せる（その日の合計が数量・金額とも 0 の商品〈売って返品した等〉は載らない）。売れた商品だけが出るので、店が今している「その日に売ったものを指定して部分的に印刷する」作業が要らない（TD-279）。手動販売の行（`source='manual'`）はレジの日報ではないので載せない。あわせて、その日の active な Z004 の取込みの数と、取り込まなかった行の数（`csv_imports.skipped_count` の和。商品に当たらない行と、行の誤り〈コードや数の形の誤り等〉の両方を含む）を `plu_import` で返す（取り込んでいない日と売れなかった日を紙の上で分け、数えていない行があることを隠さない）。
- **SALES-SHEET-D5（失敗と空の日）**: 失敗の順は、日付の検査 → 読取り（日報の行・PLU の行・Z004 の取込みの数の 3 つ。`DbError`）→ 足し算（溢れ）。処理ステップもこの順にする。どれかで失敗したら紙の内容を返さない（一部だけの紙を作らない）。完了した日報の取込みが 0 件の日は error にせず、`import_count = 0` と空の行で返す（UI は印刷を止める。UI-09a-D17）。
- **SALES-SHEET-D6（保存しない）**: 紙の内容は DB に保存せず、呼ぶたびにその時点の取込みから作る。印刷した後に取込みを取り消すか追加すると、次に作る紙は変わる（紙の印刷日時で見分ける。UI-09a-D17）。

**型定義**:

```
struct DailyReportSheet {
    date: String,                                // 対象日（YYYY-MM-DD）
    import_count: i64,                           // その日の完了した日報の取込みの数（24 §14.21a の `import_ids` の長さ）。0 なら行は空
    summary_rows: Vec<DailyReportSheetRow>,      // Z001（SALES-SHEET-D2・D3）
    payment_rows: Vec<DailyReportSheetRow>,      // Z002
    department_rows: Vec<DailyReportSheetRow>,   // Z005
    label_changed_sources: Vec<DailyReportSourceKind>, // SALES-SHEET-D2 の 2・3 の帳票（組の集合が一致しない）。Z001 → Z002 → Z005 の順、重複なし
    unsummed_sources: Vec<DailyReportSourceKind>,      // SALES-SHEET-D2 の 3 の帳票（足さずに取込みごとに載せた）。同じ順、重複なし
    plu_rows: Vec<DailySaleItem>,                // SALES-SHEET-D4。§19.2 の型をそのまま使う
    plu_import: DailyReportSheetPluImport,
}

struct DailyReportSheetRow {
    position: i64,          // 帳票の行の位置（sort_order）
    label: String,          // 前後の空白を除いた名称（Z005 は raw_department_name）。空のこともある
    quantity: Option<f64>,  // 個数（単位の数、小数 2 桁まで）。Z001 の総売の行と Z005 の行だけ
    count: Option<i64>,     // 件数。Z001 の総売以外の行と Z002 の行
    amount: Option<i64>,
    import_seq: Option<i64>, // SALES-SHEET-D2 の 3 の帳票の行だけ、その行の取込みの `import_ids` の中の位置（1 から）。足した行は None
}

struct DailyReportSheetPluImport {
    import_count: i64,      // その日の active な Z004 の取込みの数
    skipped_count: i64,     // その取込みの skipped_count の和
}
```

`DailyReportSourceKind` は IO-07 の型（`Z001` / `Z002` / `Z005`、[29 §29.2](29-io-daily-report-parser.md#292-型定義)）で、wire にはすでに `DailyReportSourceFileInfo.source` として出ている。

**シグネチャ**:
```
fn get_daily_report_sheet(conn: &DbConnection, date: &str) -> Result<DailyReportSheet, BizError>
```

**前提条件**: conn は &DbConnection（autocommit）。読み取り専用のためTX不要（3 つの読取りの間に取込みが確定・取消されると、日報の部分と PLU の部分が別の時点の値になりうる。単一店舗・単一利用者で、取込みと印刷を同時に行わない。印刷の直前に読み直すので、次の印刷で揃う）。

**処理ステップ**:

1. **日付バリデーション**: §19.3 の手順 1 と同じ検査と文言。
2. **読取り**（SALES-SHEET-D5 の順）: `sales_repo::get_daily_report_sheet_lines(conn, date)`（[24 §14.21a](24-io-csv-import-repo.md)）で、その日の完了した取込みの id の並び（`import_ids`）と、Z001・Z002・Z005 の全行（取込みの古い順・位置の順）を得る。続けて `sales_repo::get_daily_sales_records(conn, date)`（§19.3 の手順 2 と同じ IO 関数）と `sales_repo::get_active_z004_import_totals(conn, date)`（[24 §14.21b](24-io-csv-import-repo.md)）を読む。どれかの失敗で止まる。
3. **足す**: 日報の取込みが 0 件なら行を空にする（SALES-SHEET-D5）。1 件以上なら帳票ごとに SALES-SHEET-D2 の 1〜3 のどれかを決めて行を作り（1・2 は組で足し、3 は取込みごとに写す）、SALES-SHEET-D3 で載せる行を選ぶ。溢れで止まる。
4. **PLU の部分**: 手順 2 の商品別売上から `source='auto'` で数量か金額が 0 でない行を残し、Z004 の取込みの数から `plu_import` を作る。
5. **結果返却**: `DailyReportSheet` を返す。

**エラーハンドリング**:
- 日付形式不正・暦妥当性エラー → `BizError::ValidationFailed`（§19.3 と同じ文言）
- 足し算の溢れ → `BizError::ValidationFailed`（本書冒頭の「数量と単位」の溢れと同じ扱い）
- DB読み取り失敗 → `BizError::DatabaseError(DbError)`

**テスト**: 同じ日の 2 取込みで、位置と名称が同じ行が足される（Z001 の総売の個数 `1.3` + `2` → `3.3`）。一方の取込みで金額が NULL の組は NULL。同じ位置で名称が違う 2 行は別の行に残り、`label_changed_sources` にその帳票が入る。行が 1 つずれた 2 取込み（片方だけ途中に 1 行多い）では、同じ名称が位置ごとに 2 行に分かれ、`label_changed_sources` にその帳票が入る。2 取込みの組の集合が同じ日は `label_changed_sources` が空で、1 本の中に同じ名称が 2 行あっても組で足す。行がずれ、片方の取込みに同じ名称が 2 行ある日（SALES-SHEET-D2 の 3 の例）は、その帳票だけ足さずに取込みごとの行（`import_seq` 1・2）で返し、`unsummed_sources` と `label_changed_sources` にその帳票が入り、ほかの帳票は足す。同じ日の 3 取込みの 2 本目にその帳票の行が無いとき、3 本目の行の `import_seq` は 3。組の集合が一致せず、同じ名称の 2 行が名称の空の行だけの日は 2 のまま足す。名称が空で値が 0 の行は載らず、名称が空で値のある行は載る。取込みが 0 件の日は `import_count = 0` で行が空。取り消した取込みの行は入らない。PLU の部分に手動販売の行と数量・金額とも 0 の行が入らない。足し算の溢れは `ValidationFailed` で、紙の内容を返さない。取込みの id の順に金額が `i64::MAX`・`1`・`-i64::MAX` の組は `ValidationFailed`（並べ替えて `1` にしない）、`i64::MAX`・`1`・NULL の組は NULL（溢れより NULL が先）。

---

### 19.4 get_monthly_sales

**関数要求**: 指定月の売上データを商品別または部門別に集計し、ランキングと前月比較を含むレポートを返す。is_voided=0 のレコードのみ対象

**シグネチャ**:
```
fn get_monthly_sales(
    conn: &DbConnection,
    month: &str,
    mode: SalesMode,
) -> Result<MonthlySalesReport, BizError>
```

**前提条件**: conn は &DbConnection（autocommit）。読み取り専用クエリのためTX不要

**処理ステップ**:

1. **月バリデーション**
   - month が YYYY-MM 形式でない → BizError::ValidationFailed("月の形式が不正です（YYYY-MM）")
   - 月の範囲チェック（01-12）→ BizError::ValidationFailed("存在しない月です")

2. **対象月の日付範囲を導出**
   - date_from = "{month}-01"
   - date_to = 月末日を計算（例: 2026-03 → "2026-03-31"）

3. **モード別集計（商品別売上明細）**
   - ByProduct:
     - sales_repo::get_monthly_sales_by_product(conn, date_from, date_to) → Vec<(product_code, name, stock_unit, quantity, amount)>
     - key = product_code, label = name。quantity は単位の種類で count_points か length_cm に入れ、stock_unit を運ぶ
   - ByDepartment:
     - sales_repo::get_monthly_sales_by_department(conn, date_from, date_to) → Vec<(department_id, department_name, stock_unit, quantity, amount)>（部門と商品の単位ごとの行）
     - 部門ごとに、数量を種類で分けて count_points / length_cm に、amount を足す。key = department_id.to_string(), label = department_name、stock_unit = None

4. **ランキング付与（商品別売上明細）**
   - amount の降順でソート
   - 1始まりの ranking を付与
   - 同額の場合は同順位（dense rank ではなく row number）

5. **前月比較（商品別売上明細）**
   - 前月を計算: YYYY-MM → 1ヶ月前（2026-01 → 2025-12 の年境界に注意）
   - 前月の同集計をステップ3-4と同じ方法で取得
   - 取得できた場合は Some(prev_items)、前月データなしは Some(空Vec)

6. **公式日報部門集計（REQ-401 redesign target）**
   - sales_repo::get_monthly_official_department_totals(conn, date_from, date_to) → Option<Vec<OfficialMonthlyDepartmentTotal>>
   - `daily_report_department_lines` を `daily_report_imports.status='completed'` かつ `report_date BETWEEN date_from AND date_to` で集計する
   - 同一日の複数completed親もすべて加算する。日次の追加取込みが月次合計へ加算されることを回帰契約とする
   - mode に関係なく `official_department_totals` として返す
   - 日報取込みが1件もない月は `None`。一部日だけ日報がある月は取得済み日だけの合計とする
   - 取り込んだ日（未実装、57 UI-09b-D10・D11）: sales_repo::get_monthly_official_report_dates(conn, date_from, date_to) → Vec<String>（[24 §14.22a](24-io-csv-import-repo.md)。completed の親の distinct report date、昇順）を `official_report_dates` として返す。parent の件数を使わない（同じ日の複数 completed 親は 1 日）。mode に関係なく返す
   - 不変条件: 同じ conn・同じ `date_from`/`date_to`・同じ `status='completed'` の条件で読むので、`official_report_dates` が空 ⇔ `official_department_totals` が `None`。CMD が DB の mutex を持つ間に 2 つを読むので、間に書込みは入らない。部門の行が 0 の親だけの日は、日には入り、部門の集計は `Some(空Vec)` になる
   - 公式日報部門集計と商品別集計は別seriesとして返し、両者を加算しない。取り込んだ日は `daily_report_imports` だけから読み、`sale_records`（Z004・手動販売）の日を入れない（57 UI-09b-D8）

7. **結果返却**
   - MonthlySalesReport { month, mode, items, prev_month_comparison, official_department_totals, official_report_dates }

**エラーハンドリング**:
- 月形式不正 → BizError::ValidationFailed(メッセージ)
- DB読み取り失敗 → BizError::DatabaseError(DbError)

**設計判断 — ranking を items に埋め込む（architecture/cmd-task-specs.md との差異）**:
- architecture/cmd-task-specs.md CMD-09 は `MonthlySalesReport(items[], rankings[], prev_month_comparison)` と ranking を独立配列で記載
- 本設計では `MonthlySaleItem.ranking` として items に埋め込む。理由: ranking は items の amount 降順と1:1対応しており、別配列にすると items とのインデックス同期が必要になる。埋め込みの方がUI側の実装が単純

**設計判断 — 取り込んだ日を月次の DTO で返す（57 UI-09b-D11、未実装）**:
- 既存の `list_daily_report_imports`（§37.6）を UI から呼んで日を作る案は採らない。取込みごとの行で取消した行も返し、completed で絞り日で distinct にする規則（UI-09b-D10）を UI に置くことになる。paging（per_page の上限 100）で月の全部を 1 回で得る保証が無い。公式部門集計と別の query・cache になり、invalidate の時差で日と集計が食い違う時間ができる
- `get_monthly_official_department_totals`（24 §14.22）とは別の IO の関数にする。既存の関数の `None` の契約と test を変えずに済み、2 つの食い違いは手順 6 の不変条件の test が見張る
- 「取り込んでいない日」（暦の補集合と今日より後の日の除外）は返さない。今日の日付を BIZ に持ち込まず、UI が `official_report_dates` と暦から作る（57 UI-09b-D11）

**設計判断 — 全件返却（ページングなし）**:
- 先決事項D-5 に基づき、初期実装では全件返却
- 4000商品の月次集計でもJSON応答は数MB（許容範囲）
- パフォーマンス問題が発生した場合は LIMIT/OFFSET を追加

**設計判断 — 前月比較の年境界処理**:
- 2026年1月 → 前月は2025年12月
- chrono の NaiveDate 演算で month - 1 を計算（年のロールオーバーを自動処理）
- テストで12月→1月の境界を明示的にカバーすること

---

### 19.5 export_sales_csv

**関数要求**: 指定日または指定月の売上データをCSVバイト列としてエクスポートする。日次・月次（商品別/部門別）の3種類のレポート形式をサポート

**型定義**:

```
enum SalesReportType {
    Daily,              // 日次（target: YYYY-MM-DD）
    MonthlyByProduct,   // 月次・商品別（target: YYYY-MM）
    MonthlyByDepartment,// 月次・部門別（target: YYYY-MM）
}

struct SalesCsvExportResult {
    csv_bytes: Vec<u8>,          // UTF-8 BOM付きCSVバイト列
    count: usize,                // レコード件数
    suggested_filename: String,  // 推奨ファイル名
}
```

**シグネチャ**:
```
fn export_sales_csv(
    conn: &DbConnection,
    report_type: &SalesReportType,
    target: &str,
) -> Result<SalesCsvExportResult, BizError>
```

**前提条件**: conn は &DbConnection（autocommit）。読み取り専用クエリのためTX不要

**処理ステップ**:

1. **report_type で分岐してデータ取得 + CSV構築**
   - Daily:
     - get_daily_sales(conn, target) を呼ぶ
     - ヘッダ: `["商品コード", "商品名", "部門", "数量", "金額", "記録元"]`
     - rows: items → [product_code, name, department_name, 数量（冒頭の「数量と単位」の商品の行）, amount, translate_source(source)]
     - suggested_filename: `sales_daily_{target}.csv`
   - MonthlyByProduct:
     - get_monthly_sales(conn, target, SalesMode::ByProduct) を呼ぶ
     - ヘッダ: `["ランク", "商品コード", "商品名", "数量", "金額"]`
     - rows: items → [ranking, key, label, 数量（同、商品の行）, amount]
     - suggested_filename: `sales_monthly_product_{target}.csv`
   - MonthlyByDepartment:
     - get_monthly_sales(conn, target, SalesMode::ByDepartment) を呼ぶ
     - ヘッダ: `["ランク", "部門名", "数量", "金額"]`
     - rows: items → [ranking, label, 数量（同、集計の行）, amount]
     - suggested_filename: `sales_monthly_dept_{target}.csv`

2. **CSV生成**
   - report_csv_exporter::export_csv(&headers, &rows) → Vec<u8>（UTF-8 BOM付き、CRLF改行）

3. **結果返却**
   - SalesCsvExportResult { csv_bytes, count: items.len(), suggested_filename }

**エラーハンドリング**:
- 日付/月形式不正 → BizError::ValidationFailed（get_daily_sales/get_monthly_sales 内でバリデーション済み）
- DB読み取り失敗 → BizError::DatabaseError(DbError)

**設計判断 — source の日本語変換**:
- CSV出力では "auto" → "POS"、"manual" → "手動" に変換。利用者向け帳票のため英語識別子のままでは不親切
- 内部ヘルパー translate_source で変換。未知の値は防御的にそのまま通す

**設計判断 — 既存関数の再利用**:
- get_daily_sales / get_monthly_sales を内部で呼び出す。日付バリデーション・DBクエリ・集計ロジックの重複を避ける
- CSV列構造のみが新規ロジック。IO-05 export_csv は純関数で既実装

---

### 19.6 非目的

このモジュールが**やらないこと**を明示する。責務境界の誤解を防ぐため。

| やらないこと | 理由 | 責務を持つモジュール |
|------------|------|-----------------|
| sale_records への書き込み | 読み取り専用モジュール | BIZ-03（CSV取込み）, BIZ-02（手動販売） |
| is_voided レコードの操作 | ロールバック処理 | BIZ-03 rollback_csv_import |
| 在庫変動の集計 | 在庫変動履歴は別ドメイン | BIZ-02 / CMD-06 list_movements |
| ページング処理 | 初期実装では全件返却（先決事項D-5） | 将来追加時はこのモジュール内 |

### 19.7 対応不変条件

| 不変条件 | 本モジュールでの対応 |
|---------|-----------------|
| INV-1: quantity符号規約 | sale_records.quantity を売上帳票視点でそのまま返す（+販売/-返品）。符号変換しない |
| INV-4: is_voided の使用範囲 | WHERE is_voided = 0 で voided レコードを除外。voided を操作する処理は持たない |

### 更新履歴

| 日付 | PR | 内容 |
|---|---|---|
| 2026-08-16 | PR #79 | SPEC-SDI-D6: 商品別 `product_code + source` 集約、全completed日報親のNULL安全な日次集約、`source_import_count`、月次additive regressionを正本化。 |
| 2026-09-27 | daily-report-z-display（design） | `OfficialDailyReportSummary.summary_imports` と `OfficialDailySummaryImport` / `OfficialDailySummaryLine` を追加。Z001の行は取込みごとに返し合算しない（D-096、[Plan Packet](../archive/plans/2026-09-27-daily-report-z-display.md)）。 |
| 2026-10-04 | daily-report-import-gaps（plan-first） | 日報の個数の wire を単位の数（`f64`）にし、DB の 100 倍の整数から BIZ-05 で戻す（IO-07-D2、D-104）。 |
| 2026-10-08 | 単位の拡張 design lane | 冒頭に単位の拡張の後の数量（proposed・未実装、D-113 / SPEC-UNIT-D3・D11、owner の決定 TD-203）を追加: 売上の wire の単位と、単位の違う商品の数量を数量の種類ごとの 2 本（点と m）に分けて出す規則（TD-206・TD-207）。 |
| 2026-10-17 | 日報の紙と意味づけ（design） | §19.3a `get_daily_report_sheet`（日報の紙、proposed・未実装）と SALES-SHEET-D1〜D6 を追加: 1 日 1 枚、Z001・Z002・Z005 を位置と名称の組で足す（行がずれ、1 本の中に同じ名称が 2 行ある帳票は足さずに取込みごと）、PLU は売れた行だけ（D-120）。§19.3 の手順 3 の支払の集約を同じ規則にし、足さない日と名称か並びの変わった日に warning を 1 件足す。足す値は NULL を溢れより先に見て、取込みの id の昇順に足す。 |
| 2026-10-06 | z001-display（runtime、起票） | §19.2 の `OfficialDailyReportSummary` の field の並びを code（`sales_service.rs` の struct）と生成 bindings に合わせた（`source_import_count` を先頭へ。PR #114 Final Review の P3、[Plan Packet](../archive/plans/2026-10-06-z001-display.md)）。 |
| 2026-10-27 | 日報の紙の runtime（起票、D-142） | 冒頭の「数量と単位」にレジの日報の値の範囲（IO-07-D6 の上限で収め、読取りで検査しない）を足した。§19.3 手順 3 の支払を、IO が足さずに返した行に BIZ-05 が紙と同じ 1 つの関数で SALES-SHEET-D2 を当てる形にし、互換の範囲の正本を 24 §14.21 手順 3 からこの段へ移した。D2 の足す値の溢れを紙と画面とも `ValidationFailed` にした（proposed・未実装）。 |
| 2026-10-29 | 日報の紙の runtime（Plan Review round 1 の是正） | §19.3a の「精算をまたいで足してよい根拠」の未確認の前提（行ごとにレジスターツールの値と同じか）を確かめる場を、runtime の lane の L3 から紙の切替えの前の UI-09a-D19 の (2) へ移した（D-142 の (5)）。 |
| 2026-10-31 | 日報の紙の runtime（実装、D-120・D-142） | §19.3 手順 3 と §19.3a の proposed の注記を外し、D2 の関数（`apply_sheet_rule`）の置き場を書いた。 |
| 2026-11-03 | 月次の画面に取り込んだ日を出す（起票、関門 12 の前提 B） | `MonthlySalesReport` に `official_report_dates` を足し（未実装）、§19.4 手順 6 の「将来coverage countを返す場合は」を 24 §14.22a で日を得る手順と、`official_department_totals` との不変条件に置き換えた。取り込んだ日を月次の DTO で返す設計判断の段を足した（57 UI-09b-D11、[packet](../plans/2026-11-03-monthly-imported-days.md)）。 |
