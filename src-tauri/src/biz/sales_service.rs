//! BIZ-05: 売上集計ロジック
//!
//! 日次・月次の売上データを集計し、レポート画面とCSVエクスポートに必要なデータを提供する。
//! is_voided=0 のレコードのみ対象。読み取り専用、TX不要。
//!
//! docs/function-design/34-biz-sales-service.md に基づく実装。

use crate::biz::unit_amount::{format_length_m, stock_unit_kind, StockUnitKind};
use crate::biz::BizError;
use crate::db::product_repo::ProductStockUnit;
use crate::db::sales_repo;
use crate::db::DbConnection;
use crate::io::daily_report_parser::{quantity_hundredths_to_units, DailyReportSourceKind};
use crate::io::report_csv_exporter;
use chrono::NaiveDate;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// 型定義
// ---------------------------------------------------------------------------

/// 集計モード
///
/// response serialize 時に snake_case 出力（"by_product" / "by_department"）。
/// request 引数は `get_monthly_sales(mode: String)` で受け取り CMD 層で変換する（H-1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum SalesMode {
    ByProduct,
    ByDepartment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum DailySaleSource {
    Auto,
    Manual,
}

/// 日次売上アイテム（BIZ公開型。DB型からマッピング）
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct DailySaleItem {
    pub product_code: String,
    pub name: String,
    pub department_name: String,
    pub department_id: i64,
    /// 商品の単位。`quantity` は在庫の数量（長さは cm）で、表示は単位で決める（SPEC-UNIT-D3）
    pub stock_unit: ProductStockUnit,
    pub quantity: i64,
    pub amount: i64,
    pub source: DailySaleSource,
}

/// 日次売上レポート
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct DailySalesReport {
    pub date: String,
    pub items: Vec<DailySaleItem>,
    pub department_subtotals: Vec<DeptSubtotal>,
    pub grand_total: GrandTotal,
    pub official_daily_report: Option<OfficialDailyReportSummary>,
}

/// 部門小計
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct DeptSubtotal {
    pub department_id: i64,
    pub department_name: String,
    /// 個数の種類の数量の和
    pub count_points: i64,
    /// 長さの種類の数量の和（cm）
    pub length_cm: i64,
    pub amount: i64,
}

/// 総合計
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct GrandTotal {
    /// 個数の種類の数量の和
    pub count_points: i64,
    /// 長さの種類の数量の和（cm）
    pub length_cm: i64,
    pub amount: i64,
}

/// レジ日報由来の公式日次サマリ
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct OfficialDailyReportSummary {
    pub source_import_count: i64,
    pub report_date: String,
    pub gross_amount: Option<i64>,
    pub net_amount: Option<i64>,
    pub payment_lines: Vec<OfficialDailyPaymentLine>,
    pub department_lines: Vec<OfficialDailyDepartmentLine>,
    pub warnings: Vec<String>,
    pub summary_imports: Vec<OfficialDailySummaryImport>,
}

/// レジ日報由来の日計（Z001）の1回分の取込み
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct OfficialDailySummaryImport {
    pub daily_report_import_id: i64,
    // daily_report_imports.imported_at（秒までの日時）
    pub imported_at: String,
    pub lines: Vec<OfficialDailySummaryLine>,
}

/// レジ日報由来の日計（Z001）の行
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct OfficialDailySummaryLine {
    pub label: String,
    // 単位の数（DB の 100 倍の整数を IO-07-D2 の変換で戻した値）
    pub quantity: Option<f64>,
    pub count: Option<i64>,
    pub amount: Option<i64>,
}

/// レジ日報由来の支払集計行
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct OfficialDailyPaymentLine {
    pub payment_key: String,
    pub label: String,
    pub amount: Option<i64>,
    pub count: Option<i64>,
}

/// レジ日報由来の部門別集計行
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct OfficialDailyDepartmentLine {
    pub department_id: Option<i64>,
    pub raw_department_name: String,
    pub normalized_department_name: Option<String>,
    pub amount: i64,
    // 単位の数（DB の 100 倍の整数を IO-07-D2 の変換で戻した値）
    pub quantity: Option<f64>,
    pub count: Option<i64>,
}

/// 月次売上レポート
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct MonthlySalesReport {
    pub month: String,
    pub mode: SalesMode,
    pub items: Vec<MonthlySaleItem>,
    pub prev_month_comparison: Option<Vec<MonthlySaleItem>>,
    pub official_department_totals: Option<Vec<OfficialMonthlyDepartmentTotal>>,
    /// completed の日報取込みのある日（YYYY-MM-DD、昇順、重複なし）。空 ⇔ official_department_totals が None（34 §19.4 手順 6）
    pub official_report_dates: Vec<String>,
}

/// 月次売上集計アイテム（ランキング付き）
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct MonthlySaleItem {
    pub key: String,
    pub label: String,
    /// 個数の種類の数量の和（商品別の行は、長さの商品なら 0）
    pub count_points: i64,
    /// 長さの種類の数量の和（cm。商品別の行は、個数の商品なら 0）
    pub length_cm: i64,
    /// 商品別の行だけ値を持つ（相殺して 0 になった行も種類を失わない）。部門別は null
    pub stock_unit: Option<ProductStockUnit>,
    pub amount: i64,
    pub ranking: u32,
}

/// レジ日報由来の月次公式部門集計行
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct OfficialMonthlyDepartmentTotal {
    pub department_id: Option<i64>,
    pub label: String,
    pub amount: i64,
    // 単位の数（月の SUM を 100 倍の整数で足してから戻した値）
    pub quantity: Option<f64>,
    pub count: Option<i64>,
}

/// 日報の紙の内容（34 §19.3a、SALES-SHEET-D1〜D6）
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct DailyReportSheet {
    pub date: String,
    /// その日の完了した日報の取込みの数。0 なら行は空
    pub import_count: i64,
    pub summary_rows: Vec<DailyReportSheetRow>,
    pub payment_rows: Vec<DailyReportSheetRow>,
    pub department_rows: Vec<DailyReportSheetRow>,
    /// SALES-SHEET-D2 の 2・3 の帳票（Z001 → Z002 → Z005 の順）
    pub label_changed_sources: Vec<DailyReportSourceKind>,
    /// SALES-SHEET-D2 の 3 の帳票（足さずに取込みごとに載せた）
    pub unsummed_sources: Vec<DailyReportSourceKind>,
    /// SALES-SHEET-D4: その日に売れた Z004 由来の行
    pub plu_rows: Vec<DailySaleItem>,
    pub plu_import: DailyReportSheetPluImport,
}

/// 日報の紙の 1 行
#[derive(Debug, PartialEq, serde::Serialize, specta::Type)]
pub struct DailyReportSheetRow {
    pub position: i64,
    /// 前後の空白を除いた名称（Z005 はレジの名称）。空のこともある
    pub label: String,
    // 単位の数（小数 2 桁まで）
    pub quantity: Option<f64>,
    pub count: Option<i64>,
    pub amount: Option<i64>,
    /// SALES-SHEET-D2 の 3 の帳票の行だけ、取込みの並びの中の位置（1 から）
    pub import_seq: Option<i64>,
}

/// 日報の紙の PLU の部分の取込みの数（SALES-SHEET-D4）
#[derive(Debug, PartialEq, serde::Serialize, specta::Type)]
pub struct DailyReportSheetPluImport {
    pub import_count: i64,
    pub skipped_count: i64,
}

/// レポート種別（CSVエクスポート用）
///
/// `serde::Deserialize` + `specta::Type` + `#[serde(rename_all = "snake_case")]` で
/// frontend bindings から `"daily" | "monthly_by_product" | "monthly_by_department"` literal union として
/// 受け取り、CMD 層で文字列 validation なく直接 enum として deserialize される（H-5 / Q-6 A 案）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum SalesReportType {
    /// 日次（target: YYYY-MM-DD）
    Daily,
    /// 月次・商品別（target: YYYY-MM）
    MonthlyByProduct,
    /// 月次・部門別（target: YYYY-MM）
    MonthlyByDepartment,
}

/// CSVエクスポート結果
#[derive(Debug)]
pub struct SalesCsvExportResult {
    /// UTF-8 BOM付きCSVバイト列
    pub csv_bytes: Vec<u8>,
    /// レコード件数
    pub count: usize,
    /// 推奨ファイル名
    pub suggested_filename: String,
}

// ---------------------------------------------------------------------------
// 公開関数
// ---------------------------------------------------------------------------

/// 指定日の売上データを商品別に集計し、部門小計と総合計を含むレポートを返す
///
/// 34-biz-sales-service.md セクション19.3
pub fn get_daily_sales(conn: &DbConnection, date: &str) -> Result<DailySalesReport, BizError> {
    // Step 1: 日付バリデーション
    crate::biz::inventory_service::validate_business_date(date)?;

    // Step 2: 商品別売上取得 + DB型 -> BIZ型マッピング
    let items = map_daily_sale_rows(sales_repo::get_daily_sales_records(conn, date)?)?;

    // Step 3: 部門小計の計算（BTreeMap で部門ID昇順を保証）
    // 数量は種類ごとの 2 本に分け、和は checked（SPEC-UNIT-D3 の集計の規則、X9）
    let mut dept_map: BTreeMap<i64, (String, SalesTotals)> = BTreeMap::new();
    let mut grand = SalesTotals::default();
    for item in &items {
        let entry = dept_map
            .entry(item.department_id)
            .or_insert_with(|| (item.department_name.clone(), SalesTotals::default()));
        entry.1.add(item.stock_unit, item.quantity, item.amount)?;
        grand.add(item.stock_unit, item.quantity, item.amount)?;
    }
    let department_subtotals: Vec<DeptSubtotal> = dept_map
        .into_iter()
        .map(|(id, (name, totals))| DeptSubtotal {
            department_id: id,
            department_name: name,
            count_points: totals.count_points,
            length_cm: totals.length_cm,
            amount: totals.amount,
        })
        .collect();

    // Step 4: 総合計
    let grand_total = GrandTotal {
        count_points: grand.count_points,
        length_cm: grand.length_cm,
        amount: grand.amount,
    };

    let official_daily_report = sales_repo::get_completed_daily_report_aggregate(conn, date)?
        .map(map_official_daily_report)
        .transpose()?;

    Ok(DailySalesReport {
        date: date.to_string(),
        items,
        department_subtotals,
        grand_total,
        official_daily_report,
    })
}

/// 指定日のレジの日報を、印刷する 1 日 1 枚の紙の内容にして返す。読み取りだけで保存しない
///
/// 34-biz-sales-service.md セクション19.3a（SALES-SHEET-D1〜D6）
pub fn get_daily_report_sheet(
    conn: &DbConnection,
    date: &str,
) -> Result<DailyReportSheet, BizError> {
    // SALES-SHEET-D5: 日付 → 読取り（3 つ）→ 足し算の順で、どれかで止まれば紙を返さない
    crate::biz::inventory_service::validate_business_date(date)?;
    let lines = sales_repo::get_daily_report_sheet_lines(conn, date)?;
    let records = sales_repo::get_daily_sales_records(conn, date)?;
    let (z004_import_count, skipped_count) = sales_repo::get_active_z004_import_totals(conn, date)?;

    let mut label_changed_sources = Vec::new();
    let mut unsummed_sources = Vec::new();
    let mut sheet_rows = |source: DailyReportSourceKind,
                          rows: Vec<sales_repo::DailyReportSheetLineRow>|
     -> Result<Vec<DailyReportSheetRow>, BizError> {
        let input: Vec<SheetRuleLine> = rows
            .into_iter()
            .enumerate()
            .map(|(index, row)| SheetRuleLine {
                import_id: row.daily_report_import_id,
                // 紙の名称は前後の空白を除いた値なので、同値の決め手は読取りの並びで足りる
                tie: index as i64,
                position: row.sort_order,
                label: row.label,
                quantity_hundredths: row.quantity_hundredths,
                count: row.count,
                amount: row.amount,
            })
            .collect();
        let (case, rows) = apply_sheet_rule(&input, &lines.import_ids)?;
        if case != SheetRuleCase::Summed {
            label_changed_sources.push(source);
        }
        if case == SheetRuleCase::Unsummed {
            unsummed_sources.push(source);
        }
        Ok(rows
            .into_iter()
            // SALES-SHEET-D3: 名称が空で値が全部 0 か NULL の行は載せない
            .filter(|row| {
                !(row.label.is_empty()
                    && [row.quantity_hundredths, row.count, row.amount]
                        .iter()
                        .all(|value| value.unwrap_or(0) == 0))
            })
            .map(|row| DailyReportSheetRow {
                position: row.position,
                label: row.label,
                quantity: row.quantity_hundredths.map(quantity_hundredths_to_units),
                count: row.count,
                amount: row.amount,
                import_seq: row.import_seq,
            })
            .collect())
    };
    let summary_rows = sheet_rows(DailyReportSourceKind::Z001, lines.summary_lines)?;
    let payment_rows = sheet_rows(DailyReportSourceKind::Z002, lines.payment_lines)?;
    let department_rows = sheet_rows(DailyReportSourceKind::Z005, lines.department_lines)?;

    // SALES-SHEET-D4: Z004 由来（auto）で数量か金額が 0 でない行だけ。手動販売は載せない
    let plu_rows = map_daily_sale_rows(records)?
        .into_iter()
        .filter(|item| {
            item.source == DailySaleSource::Auto && (item.quantity != 0 || item.amount != 0)
        })
        .collect();

    Ok(DailyReportSheet {
        date: date.to_string(),
        import_count: lines.import_ids.len() as i64,
        summary_rows,
        payment_rows,
        department_rows,
        label_changed_sources,
        unsummed_sources,
        plu_rows,
        plu_import: DailyReportSheetPluImport {
            import_count: z004_import_count,
            skipped_count,
        },
    })
}

/// 指定月の売上データを商品別または部門別に集計し、ランキングと前月比較を含むレポートを返す
///
/// 34-biz-sales-service.md セクション19.4
pub fn get_monthly_sales(
    conn: &DbConnection,
    month: &str,
    mode: SalesMode,
) -> Result<MonthlySalesReport, BizError> {
    // Step 1: 月バリデーション
    let (year, month_num) = validate_month(month)?;

    // Step 2: 対象月の日付範囲
    let (date_from, date_to) = month_date_range(year, month_num);

    // Step 3-4: 集計 + ランキング
    let items = fetch_and_rank(conn, &date_from, &date_to, &mode)?;

    // Step 5: 前月比較
    let (prev_year, prev_month) = if month_num == 1 {
        (year - 1, 12)
    } else {
        (year, month_num - 1)
    };
    let (prev_from, prev_to) = month_date_range(prev_year, prev_month);
    let prev_items = fetch_and_rank(conn, &prev_from, &prev_to, &mode)?;
    let prev_month_comparison = Some(prev_items);
    let official_department_totals = sales_repo::get_monthly_official_department_totals(
        conn, &date_from, &date_to,
    )?
    .map(|rows| {
        rows.into_iter()
            .map(|row| OfficialMonthlyDepartmentTotal {
                department_id: row.department_id,
                label: row.label,
                amount: row.amount,
                quantity: row.quantity_hundredths.map(quantity_hundredths_to_units),
                count: row.count,
            })
            .collect()
    });
    let official_report_dates =
        sales_repo::get_monthly_official_report_dates(conn, &date_from, &date_to)?;

    Ok(MonthlySalesReport {
        month: month.to_string(),
        mode,
        items,
        prev_month_comparison,
        official_department_totals,
        official_report_dates,
    })
}

/// 指定日または指定月の売上データをCSVバイト列としてエクスポートする
///
/// 34-biz-sales-service.md §19.5 export_sales_csv
pub fn export_sales_csv(
    conn: &DbConnection,
    report_type: &SalesReportType,
    target: &str,
) -> Result<SalesCsvExportResult, BizError> {
    let (headers, rows, count, suggested_filename) = match report_type {
        SalesReportType::Daily => {
            let report = get_daily_sales(conn, target)?;
            let headers = vec![
                "商品コード".to_string(),
                "商品名".to_string(),
                "部門".to_string(),
                "数量".to_string(),
                "金額".to_string(),
                "記録元".to_string(),
            ];
            let rows: Vec<Vec<String>> = report
                .items
                .iter()
                .map(|item| {
                    vec![
                        item.product_code.clone(),
                        item.name.clone(),
                        item.department_name.clone(),
                        csv_item_quantity(item.stock_unit, item.quantity),
                        item.amount.to_string(),
                        translate_source(&item.source).to_string(),
                    ]
                })
                .collect();
            let count = report.items.len();
            let filename = format!("sales_daily_{}.csv", target);
            (headers, rows, count, filename)
        }
        SalesReportType::MonthlyByProduct => {
            let report = get_monthly_sales(conn, target, SalesMode::ByProduct)?;
            let headers = vec![
                "ランク".to_string(),
                "商品コード".to_string(),
                "商品名".to_string(),
                "数量".to_string(),
                "金額".to_string(),
            ];
            let rows: Vec<Vec<String>> = report
                .items
                .iter()
                .map(|item| {
                    vec![
                        item.ranking.to_string(),
                        item.key.clone(),
                        item.label.clone(),
                        // 商品別の行は種類のどちらか一方だけが値を持つ
                        if item.length_cm != 0 {
                            csv_length_m(item.length_cm)
                        } else {
                            item.count_points.to_string()
                        },
                        item.amount.to_string(),
                    ]
                })
                .collect();
            let count = report.items.len();
            let filename = format!("sales_monthly_product_{}.csv", target);
            (headers, rows, count, filename)
        }
        SalesReportType::MonthlyByDepartment => {
            let report = get_monthly_sales(conn, target, SalesMode::ByDepartment)?;
            let headers = vec![
                "ランク".to_string(),
                "部門名".to_string(),
                "数量".to_string(),
                "金額".to_string(),
            ];
            let rows: Vec<Vec<String>> = report
                .items
                .iter()
                .map(|item| {
                    vec![
                        item.ranking.to_string(),
                        item.label.clone(),
                        csv_total_quantity(item.count_points, item.length_cm),
                        item.amount.to_string(),
                    ]
                })
                .collect();
            let count = report.items.len();
            let filename = format!("sales_monthly_dept_{}.csv", target);
            (headers, rows, count, filename)
        }
    };

    let csv_bytes = report_csv_exporter::export_csv(&headers, &rows);

    Ok(SalesCsvExportResult {
        csv_bytes,
        count,
        suggested_filename,
    })
}

// ---------------------------------------------------------------------------
// 内部関数
// ---------------------------------------------------------------------------

/// 数量の種類ごとの和と金額の和（SPEC-UNIT-D3 の集計の規則）。長さを点に足さない
#[derive(Debug, Default, Clone, Copy)]
struct SalesTotals {
    count_points: i64,
    length_cm: i64,
    amount: i64,
}

impl SalesTotals {
    /// 単位の種類で数量を振り分けて足す。i64 の溢れは `ValidationFailed`（wrap した値を返さない）
    fn add(&mut self, unit: ProductStockUnit, quantity: i64, amount: i64) -> Result<(), BizError> {
        let overflow = || {
            BizError::ValidationFailed("売上集計の計算でオーバーフローが発生しました".to_string())
        };
        let slot = match stock_unit_kind(unit) {
            StockUnitKind::Count => &mut self.count_points,
            StockUnitKind::Length => &mut self.length_cm,
        };
        *slot = slot.checked_add(quantity).ok_or_else(overflow)?;
        self.amount = self.amount.checked_add(amount).ok_or_else(overflow)?;
        Ok(())
    }
}

/// CSV の長さの数量: 単位の語と桁区切りを付けない m の数（130 → "1.3"、123456 → "1234.56"）
fn csv_length_m(quantity_cm: i64) -> String {
    format_length_m(quantity_cm)
        .trim_end_matches(" m")
        .replace(',', "")
}

/// CSV の商品の行の `数量`: 長さの商品は m の数、個数の商品は今と同じ整数（34 §19.5）
fn csv_item_quantity(unit: ProductStockUnit, quantity: i64) -> String {
    match stock_unit_kind(unit) {
        StockUnitKind::Count => quantity.to_string(),
        StockUnitKind::Length => csv_length_m(quantity),
    }
}

/// CSV の集計の行（月次の部門別）の `数量`: 長さが 0 なら今と同じ整数、そうでなければ画面と同じ文字列
fn csv_total_quantity(count_points: i64, length_cm: i64) -> String {
    match (count_points, length_cm) {
        (count, 0) => count.to_string(),
        (0, length) => format_length_m(length),
        (count, length) => format!("{} 点・{}", group_thousands(count), format_length_m(length)),
    }
}

/// 整数に 3 桁区切りを付ける（符号は先頭、区切りは絶対値の桁。X9 の `N 点` の `N`）
fn group_thousands(value: i64) -> String {
    let digits = value.unsigned_abs().to_string();
    let mut out = String::from(if value < 0 { "-" } else { "" });
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// 売上記録元の日本語変換（CSV出力用）
fn translate_source(source: &DailySaleSource) -> &str {
    match source {
        DailySaleSource::Auto => "POS",
        DailySaleSource::Manual => "手動",
    }
}

/// YYYY-MM 形式の月をバリデーションし、(year, month) を返す
fn validate_month(month: &str) -> Result<(i32, u32), BizError> {
    if month.len() != 7 || month.chars().nth(4) != Some('-') {
        return Err(BizError::ValidationFailed(
            "月の形式が不正です（YYYY-MM）".to_string(),
        ));
    }
    let year: i32 = month[..4]
        .parse()
        .map_err(|_| BizError::ValidationFailed("月の形式が不正です（YYYY-MM）".to_string()))?;
    let month_num: u32 = month[5..7]
        .parse()
        .map_err(|_| BizError::ValidationFailed("月の形式が不正です（YYYY-MM）".to_string()))?;

    if !(1..=12).contains(&month_num) {
        return Err(BizError::ValidationFailed("存在しない月です".to_string()));
    }

    Ok((year, month_num))
}

/// 指定年月の日付範囲（YYYY-MM-DD, YYYY-MM-DD）を返す
fn month_date_range(year: i32, month: u32) -> (String, String) {
    let first_day =
        NaiveDate::from_ymd_opt(year, month, 1).expect("month_date_range: invalid year/month");

    // 月末日: 翌月1日の前日
    let last_day = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - chrono::Duration::days(1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap() - chrono::Duration::days(1)
    };

    (
        first_day.format("%Y-%m-%d").to_string(),
        last_day.format("%Y-%m-%d").to_string(),
    )
}

/// 商品別売上の DB 行を BIZ 型へ写す（§19.3 手順 2。日報の紙の PLU の部分も同じ写像）
fn map_daily_sale_rows(
    rows: Vec<sales_repo::DailySaleRow>,
) -> Result<Vec<DailySaleItem>, BizError> {
    rows.into_iter()
        .map(|r| {
            Ok(DailySaleItem {
                product_code: r.product_code,
                name: r.name,
                department_name: r.department_name,
                department_id: r.department_id,
                stock_unit: r.stock_unit,
                quantity: r.quantity,
                amount: r.amount,
                source: match r.source.as_str() {
                    "auto" => DailySaleSource::Auto,
                    "manual" => DailySaleSource::Manual,
                    other => {
                        return Err(BizError::DatabaseError(crate::db::DbError::QueryFailed(
                            format!("unknown daily sale source: {other}"),
                        )))
                    }
                },
            })
        })
        .collect()
}

/// 34 §19.3 手順 3 の支払集計の warning（D2 の 2 と 3。3 の日は 3 の文だけ）
const PAYMENT_LABEL_CHANGED_WARNING: &str = "支払集計（Z002）は、この日の精算の間で行の名称か並びが変わっています。同じ名称の行が 2 行あれば、別々に足しています";
const PAYMENT_UNSUMMED_WARNING: &str = "支払集計（Z002）は、この日の精算の間で行の名称か並びが変わり、1 回の精算の中に同じ名称の行が 2 行以上あるため、足さずに取込みの古い順に並べています。紙（日報を印刷）で取込みごとに見られます";

fn map_official_daily_report(
    row: sales_repo::OfficialDailyReportRow,
) -> Result<OfficialDailyReportSummary, BizError> {
    let unmatched_count = row
        .department_lines
        .iter()
        .filter(|line| line.department_id.is_none())
        .count();
    let mut warnings = if unmatched_count == 0 {
        Vec::new()
    } else {
        vec![format!(
            "部門マスタと対応していない部門が {} 件あります（部門名のまま表示しています）",
            unmatched_count
        )]
    };

    // 34 §19.3 手順 3: 支払は紙と同じ SALES-SHEET-D2 の関数で足す。代表は最小 sort_order・最小の行 id
    let import_ids: Vec<i64> = row
        .summary_imports
        .iter()
        .map(|import| import.daily_report_import_id)
        .collect();
    let payment_input: Vec<SheetRuleLine> = row
        .payment_lines
        .iter()
        .map(|line| SheetRuleLine {
            import_id: line.daily_report_import_id,
            tie: line.id,
            position: line.sort_order,
            label: line.label.clone(),
            quantity_hundredths: None,
            count: line.count,
            amount: line.amount,
        })
        .collect();
    let (payment_case, payment_rows) = apply_sheet_rule(&payment_input, &import_ids)?;
    match payment_case {
        SheetRuleCase::Summed => {}
        SheetRuleCase::LabelChanged => warnings.push(PAYMENT_LABEL_CHANGED_WARNING.to_string()),
        SheetRuleCase::Unsummed => warnings.push(PAYMENT_UNSUMMED_WARNING.to_string()),
    }
    let payment_lines = payment_rows
        .into_iter()
        .map(|summed| {
            let representative = &row.payment_lines[summed.representative];
            OfficialDailyPaymentLine {
                payment_key: representative.payment_key.clone(),
                label: representative.label.clone(),
                amount: summed.amount,
                count: summed.count,
            }
        })
        .collect();

    Ok(OfficialDailyReportSummary {
        source_import_count: row.source_import_count,
        report_date: row.report_date,
        gross_amount: row.gross_amount,
        net_amount: row.net_amount,
        payment_lines,
        department_lines: row
            .department_lines
            .into_iter()
            .map(|line| OfficialDailyDepartmentLine {
                department_id: line.department_id,
                raw_department_name: line.raw_department_name,
                normalized_department_name: line.normalized_department_name,
                amount: line.amount,
                quantity: line.quantity_hundredths.map(quantity_hundredths_to_units),
                count: line.count,
            })
            .collect(),
        warnings,
        summary_imports: row
            .summary_imports
            .into_iter()
            .map(|import| OfficialDailySummaryImport {
                daily_report_import_id: import.daily_report_import_id,
                imported_at: import.imported_at,
                lines: import
                    .lines
                    .into_iter()
                    .map(|line| OfficialDailySummaryLine {
                        label: line.label,
                        quantity: line.quantity_hundredths.map(quantity_hundredths_to_units),
                        count: line.count,
                        amount: line.amount,
                    })
                    .collect(),
            })
            .collect(),
    })
}

// ---------------------------------------------------------------------------
// SALES-SHEET-D2（位置と名称の組で足す。紙と画面の支払集計で 1 つ、D-142 の (4)）
// ---------------------------------------------------------------------------

/// SALES-SHEET-D2 の 1〜3 の場合
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SheetRuleCase {
    /// 1: 取込みの間で組の集合が一致する
    Summed,
    /// 2: 一致せず、どの取込みにも同じ名称（空でない）の行が 2 行以上無い
    LabelChanged,
    /// 3: 一致せず、どれかの取込みに同じ名称（空でない）の行が 2 行以上ある。足さない
    Unsummed,
}

/// D2 の入力の 1 行（1 つの帳票の行）
struct SheetRuleLine {
    import_id: i64,
    /// 同じ位置の代表を決める同値の決め手（画面の支払は行の id）
    tie: i64,
    position: i64,
    /// 保存された名称（前後の空白は D2 の関数が除いて見分ける）
    label: String,
    quantity_hundredths: Option<i64>,
    count: Option<i64>,
    amount: Option<i64>,
}

/// D2 の出力の 1 行
#[derive(Debug)]
struct SheetRuleRow {
    /// 代表の入力の行の位置（入力の slice の index。最小の位置・最小の `tie`）
    representative: usize,
    position: i64,
    /// 前後の空白を除いた名称
    label: String,
    quantity_hundredths: Option<i64>,
    count: Option<i64>,
    amount: Option<i64>,
    /// 3 の行だけ、取込みの `import_ids` の中の位置（1 から）
    import_seq: Option<i64>,
}

/// SALES-SHEET-D2 を 1 つの帳票の行に当てる。`import_ids` はその日の完了した取込みの古い順
/// （`imported_at ASC, id ASC`）。`import_ids` に無い取込みの行は読まない（24 §14.21a 手順 2）
fn apply_sheet_rule(
    lines: &[SheetRuleLine],
    import_ids: &[i64],
) -> Result<(SheetRuleCase, Vec<SheetRuleRow>), BizError> {
    let import_index = |import_id: i64| import_ids.iter().position(|id| *id == import_id);
    let lines: Vec<(usize, usize, &SheetRuleLine)> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| import_index(line.import_id).map(|seq| (index, seq, line)))
        .collect();

    // 取込みごとの組の集合と、同じ名称（空でない）の行の有無
    let mut pair_sets: Vec<std::collections::BTreeSet<(i64, &str)>> =
        vec![std::collections::BTreeSet::new(); import_ids.len()];
    let mut duplicate_label = false;
    let mut labels_seen: Vec<std::collections::BTreeSet<&str>> =
        vec![std::collections::BTreeSet::new(); import_ids.len()];
    for (_, seq, line) in &lines {
        let label = line.label.trim();
        pair_sets[*seq].insert((line.position, label));
        if !label.is_empty() && !labels_seen[*seq].insert(label) {
            duplicate_label = true;
        }
    }
    let sets_match = pair_sets.windows(2).all(|pair| pair[0] == pair[1]);
    let case = match (sets_match, duplicate_label) {
        (true, _) => SheetRuleCase::Summed,
        (false, false) => SheetRuleCase::LabelChanged,
        (false, true) => SheetRuleCase::Unsummed,
    };

    if case == SheetRuleCase::Unsummed {
        // 3: 取込みの古い順、各取込みの中は位置の順。値は変えない
        let mut rows: Vec<(usize, i64, i64, SheetRuleRow)> = lines
            .iter()
            .map(|(index, seq, line)| {
                (
                    *seq,
                    line.position,
                    line.tie,
                    SheetRuleRow {
                        representative: *index,
                        position: line.position,
                        label: line.label.trim().to_string(),
                        quantity_hundredths: line.quantity_hundredths,
                        count: line.count,
                        amount: line.amount,
                        import_seq: Some(*seq as i64 + 1),
                    },
                )
            })
            .collect();
        rows.sort_by_key(|row| (row.0, row.1, row.2));
        return Ok((case, rows.into_iter().map(|row| row.3).collect()));
    }

    // 1・2: 位置と名称の組で足す
    struct Group<'a> {
        first_seen: usize,
        members: Vec<(usize, &'a SheetRuleLine)>,
    }
    let mut groups: BTreeMap<(i64, &str), Group> = BTreeMap::new();
    for (index, seq, line) in &lines {
        let group = groups
            .entry((line.position, line.label.trim()))
            .or_insert(Group {
                first_seen: *seq,
                members: Vec::new(),
            });
        group.first_seen = group.first_seen.min(*seq);
        group.members.push((*index, line));
    }
    let mut rows = Vec::with_capacity(groups.len());
    for ((position, label), mut group) in groups {
        let representative = group
            .members
            .iter()
            .min_by_key(|(_, line)| (line.position, line.tie))
            .map(|(index, _)| *index)
            .unwrap_or_default();
        // 足す順は取込みの id の昇順（34 SALES-SHEET-D2 の「足す値」の (b)）
        group.members.sort_by_key(|(_, line)| line.import_id);
        let sum = |value: fn(&SheetRuleLine) -> Option<i64>| {
            sum_sheet_values(group.members.iter().map(|(_, line)| value(line)))
        };
        rows.push((
            (position, group.first_seen),
            SheetRuleRow {
                representative,
                position,
                label: label.to_string(),
                quantity_hundredths: sum(|line| line.quantity_hundredths)?,
                count: sum(|line| line.count)?,
                amount: sum(|line| line.amount)?,
                import_seq: None,
            },
        ));
    }
    // 位置の昇順、同じ位置は名称が最初に現れた取込みの古い順
    rows.sort_by_key(|row| row.0);
    Ok((case, rows.into_iter().map(|row| row.1).collect()))
}

/// D2 の足す値: NULL が 1 つでもあれば NULL（溢れより先）、無ければ左から checked_add し溢れは
/// `ValidationFailed`（34 SALES-SHEET-D2 の (a)・(b)）
fn sum_sheet_values(
    values: impl Iterator<Item = Option<i64>> + Clone,
) -> Result<Option<i64>, BizError> {
    if values.clone().any(|value| value.is_none()) {
        return Ok(None);
    }
    values
        .flatten()
        .try_fold(0_i64, |sum, value| sum.checked_add(value))
        .map(Some)
        .ok_or_else(|| {
            BizError::ValidationFailed("売上集計の計算でオーバーフローが発生しました".to_string())
        })
}

/// モード別集計を取得し、ランキングを付与する
fn fetch_and_rank(
    conn: &DbConnection,
    date_from: &str,
    date_to: &str,
    mode: &SalesMode,
) -> Result<Vec<MonthlySaleItem>, BizError> {
    let mut items: Vec<MonthlySaleItem> = match mode {
        SalesMode::ByProduct => {
            let rows = sales_repo::get_monthly_sales_by_product(conn, date_from, date_to)?;
            rows.into_iter()
                .map(|r| {
                    let mut totals = SalesTotals::default();
                    totals.add(r.stock_unit, r.quantity, r.amount)?;
                    Ok(MonthlySaleItem {
                        key: r.product_code,
                        label: r.name,
                        count_points: totals.count_points,
                        length_cm: totals.length_cm,
                        stock_unit: Some(r.stock_unit),
                        amount: totals.amount,
                        ranking: 0,
                    })
                })
                .collect::<Result<_, BizError>>()?
        }
        SalesMode::ByDepartment => {
            // repo は部門と単位ごとの行を返す。部門ごとに種類で分けて足す（金額も）
            let rows = sales_repo::get_monthly_sales_by_department(conn, date_from, date_to)?;
            let mut dept_map: BTreeMap<i64, (String, SalesTotals)> = BTreeMap::new();
            for r in rows {
                dept_map
                    .entry(r.department_id)
                    .or_insert_with(|| (r.department_name.clone(), SalesTotals::default()))
                    .1
                    .add(r.stock_unit, r.quantity, r.amount)?;
            }
            dept_map
                .into_iter()
                .map(|(id, (name, totals))| MonthlySaleItem {
                    key: id.to_string(),
                    label: name,
                    count_points: totals.count_points,
                    length_cm: totals.length_cm,
                    stock_unit: None,
                    amount: totals.amount,
                    ranking: 0,
                })
                .collect()
        }
    };

    // amount 降順ソート（DB側で既にソート済みだが、明示的に保証）
    // amount 降順、同額時は key 昇順でタイブレーク（安定した順序を保証）
    items.sort_by(|a, b| b.amount.cmp(&a.amount).then_with(|| a.key.cmp(&b.key)));

    // ランキング付与（1始まり、row number方式）
    for (i, item) in items.iter_mut().enumerate() {
        item.ranking = (i + 1) as u32;
    }

    Ok(items)
}

// ===========================================================================
// テスト
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_database;
    use crate::db::product_repo::{self, NewProduct};
    use crate::db::sales_repo::{
        NewDailyReportDepartmentLine, NewDailyReportImport, NewDailyReportPaymentLine,
        NewDailyReportSummaryLine, NewSaleRecord,
    };

    fn setup_test_db() -> (tempfile::TempDir, crate::db::DbConnection) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();
        (dir, conn)
    }

    fn seed_product(conn: &DbConnection, code: &str, dept_id: i64) {
        let product = NewProduct {
            product_code: code.to_string(),
            jan_code: None,
            name: format!("商品{}", code),
            department_id: dept_id,
            supplier_id: None,
            selling_price: 500,
            cost_price_centi: 300,
            tax_rate: "10".to_string(),
            maker_code: None,
            stock_quantity: 0,
            stock_unit: "pcs".to_string(),
            is_discontinued: false,
            plu_dirty: true,
            plu_exported_at: None,
            plu_target: true,
            pos_stock_sync: true,
        };
        product_repo::insert_product(conn, &product).unwrap();
    }

    fn seed_sale(conn: &DbConnection, code: &str, date: &str, qty: i64, amt: i64, source: &str) {
        let record = NewSaleRecord {
            csv_import_id: None,
            product_code: code.to_string(),
            sale_date: date.to_string(),
            quantity: qty,
            amount: amt,
            source: source.to_string(),
            source_line_no: None,
            reason: None,
            note: None,
            stock_linked: false,
        };
        sales_repo::insert_sale_record(conn, &record).unwrap();
    }

    fn seed_daily_report(conn: &DbConnection, report_date: &str, bundle_hash: &str) -> i64 {
        sales_repo::insert_daily_report_import(
            conn,
            &NewDailyReportImport {
                report_date: report_date.to_string(),
                source_adapter: "casio_sr_s4000".to_string(),
                bundle_hash: bundle_hash.to_string(),
                source_files_json: "[]".to_string(),
                gross_amount: Some(12000),
                net_amount: Some(11000),
                status: "completed".to_string(),
                note: None,
                settlement_no: None,
            },
        )
        .unwrap()
    }

    fn seed_daily_report_lines(conn: &DbConnection, import_id: i64, department_id: Option<i64>) {
        sales_repo::insert_daily_report_payment_lines(
            conn,
            &[NewDailyReportPaymentLine {
                daily_report_import_id: import_id,
                source_file: "Z002".to_string(),
                payment_key: "cash".to_string(),
                label: "現金".to_string(),
                amount: Some(11000),
                count: Some(7),
                sort_order: 1,
            }],
        )
        .unwrap();
        sales_repo::insert_daily_report_department_lines(
            conn,
            &[NewDailyReportDepartmentLine {
                daily_report_import_id: import_id,
                source_file: "Z005".to_string(),
                department_id,
                raw_department_name: "その他小物".to_string(),
                normalized_department_name: Some("その他小物".to_string()),
                amount: 11000,
                quantity_hundredths: Some(700),
                count: Some(3),
                sort_order: 1,
            }],
        )
        .unwrap();
    }

    // -------------------------------------------------------------------
    // get_daily_sales
    // -------------------------------------------------------------------

    #[test]
    fn test_daily_sales_req501_normal() {
        // REQ-501: 日次売上（日付別商品売上集計）
        // 19.3: 正常取得 + subtotal + grand_total
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "P001", 1);
        seed_product(&conn, "P002", 3);
        seed_sale(&conn, "P001", "2026-03-21", 2, 1000, "auto");
        seed_sale(&conn, "P002", "2026-03-21", 1, 500, "manual");

        let report = get_daily_sales(&conn, "2026-03-21").unwrap();
        assert_eq!(report.items.len(), 2);
        assert_eq!(report.department_subtotals.len(), 2);
        assert_eq!(report.grand_total.count_points, 3);
        assert_eq!(report.grand_total.length_cm, 0);
        assert_eq!(report.grand_total.amount, 1500);
    }

    #[test]
    fn test_daily_sales_req501_invalid_date_format() {
        // REQ-501: 日次売上（日付別商品売上集計）
        // 19.3: 不正形式
        let (_dir, conn) = setup_test_db();
        let err = get_daily_sales(&conn, "2026/03/21").unwrap_err();
        assert!(matches!(err, BizError::ValidationFailed(_)));
    }

    #[test]
    fn test_daily_sales_req501_invalid_date_value() {
        // REQ-501: 日次売上（日付別商品売上集計）
        // 19.3: 存在しない日付
        let (_dir, conn) = setup_test_db();
        let err = get_daily_sales(&conn, "2026-02-30").unwrap_err();
        assert!(matches!(err, BizError::ValidationFailed(_)));
    }

    #[test]
    fn test_daily_sales_req501_empty_date() {
        // REQ-501: 日次売上（日付別商品売上集計）
        // 19.3: データなし -> 空レポート
        let (_dir, conn) = setup_test_db();
        let report = get_daily_sales(&conn, "2026-03-21").unwrap();
        assert!(report.items.is_empty());
        assert!(report.department_subtotals.is_empty());
        assert_eq!(report.grand_total.count_points, 0);
        assert_eq!(report.grand_total.length_cm, 0);
        assert_eq!(report.grand_total.amount, 0);
        assert!(report.official_daily_report.is_none());
    }

    #[test]
    fn test_daily_sales_req501_dept_subtotals() {
        // REQ-501: 日次売上（日付別商品売上集計）
        // 19.3: 複数部門の小計
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "P001", 1);
        seed_product(&conn, "P002", 1);
        seed_product(&conn, "P003", 3);
        seed_sale(&conn, "P001", "2026-03-21", 2, 1000, "auto");
        seed_sale(&conn, "P002", "2026-03-21", 3, 1500, "auto");
        seed_sale(&conn, "P003", "2026-03-21", 1, 800, "auto");

        let report = get_daily_sales(&conn, "2026-03-21").unwrap();
        assert_eq!(report.department_subtotals.len(), 2);

        // 部門ID=1: P001+P002 = qty5, amt2500
        let dept1 = &report.department_subtotals[0];
        assert_eq!(dept1.department_id, 1);
        assert_eq!(dept1.count_points, 5);
        assert_eq!(dept1.length_cm, 0);
        assert_eq!(dept1.amount, 2500);

        // 部門ID=3: P003 = qty1, amt800
        let dept3 = &report.department_subtotals[1];
        assert_eq!(dept3.department_id, 3);
        assert_eq!(dept3.count_points, 1);
        assert_eq!(dept3.length_cm, 0);
        assert_eq!(dept3.amount, 800);
    }

    #[test]
    fn test_get_daily_sales_includes_official_report_req501() {
        // REQ-501: 日次売上公式日報表示
        // 19.3: daily_report_* 保存済み行をofficial_daily_reportとして返す
        let (_dir, conn) = setup_test_db();
        let import_id = seed_daily_report(&conn, "2026-03-21", "official-daily");
        seed_daily_report_lines(&conn, import_id, Some(1));

        let report = get_daily_sales(&conn, "2026-03-21").unwrap();
        let official = report.official_daily_report.unwrap();
        assert_eq!(official.source_import_count, 1);
        assert_eq!(official.report_date, "2026-03-21");
        assert_eq!(official.gross_amount, Some(12000));
        assert_eq!(official.net_amount, Some(11000));
        assert_eq!(official.payment_lines[0].payment_key, "cash");
        assert_eq!(official.payment_lines[0].amount, Some(11000));
        assert_eq!(official.department_lines[0].amount, 11000);
    }

    #[test]
    fn test_get_daily_sales_no_fake_items_req501() {
        // REQ-501: 日次売上公式日報表示
        // 19.3: 日報取込みだけでは商品別明細・商品別小計・総合計を水増ししない
        let (_dir, conn) = setup_test_db();
        let import_id = seed_daily_report(&conn, "2026-03-21", "official-no-items");
        seed_daily_report_lines(&conn, import_id, Some(1));

        let report = get_daily_sales(&conn, "2026-03-21").unwrap();
        assert!(report.official_daily_report.is_some());
        assert!(report.items.is_empty());
        assert!(report.department_subtotals.is_empty());
        assert_eq!(report.grand_total.count_points, 0);
        assert_eq!(report.grand_total.length_cm, 0);
        assert_eq!(report.grand_total.amount, 0);
    }

    #[test]
    fn test_get_daily_sales_without_official_report_req501() {
        // REQ-501: 日次売上公式日報表示
        // 19.3: 日報未取込みでも既存商品別売上は取得できる
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "P001", 1);
        seed_sale(&conn, "P001", "2026-03-21", 2, 1000, "auto");

        let report = get_daily_sales(&conn, "2026-03-21").unwrap();
        assert!(report.official_daily_report.is_none());
        assert_eq!(report.items.len(), 1);
        assert_eq!(report.grand_total.amount, 1000);
    }

    #[test]
    fn test_get_daily_sales_warnings_unmatched_department_req501() {
        // REQ-501 / I-R3 / SPEC-SDI-D6: 同identityの未対応部門は集約後group数で単一warning。
        let (_dir, conn) = setup_test_db();
        let import_id = seed_daily_report(&conn, "2026-03-21", "official-warning");
        seed_daily_report_lines(&conn, import_id, None);
        let sibling_id = seed_daily_report(&conn, "2026-03-21", "official-warning-sibling");
        seed_daily_report_lines(&conn, sibling_id, None);

        let report = get_daily_sales(&conn, "2026-03-21").unwrap();
        let official = report.official_daily_report.unwrap();
        assert_eq!(official.source_import_count, 2);
        assert_eq!(official.department_lines.len(), 1);
        assert_eq!(official.department_lines[0].amount, 22000);
        assert_eq!(official.warnings.len(), 1);
        assert_eq!(
            official.warnings[0],
            "部門マスタと対応していない部門が 1 件あります（部門名のまま表示しています）"
        );
    }

    // -------------------------------------------------------------------
    // get_monthly_sales
    // -------------------------------------------------------------------

    #[test]
    fn test_monthly_sales_req502_by_product_normal() {
        // REQ-502: 月次売上（月次商品別/部門別集計）
        // 19.4: 商品別 + ランキング
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "P001", 1);
        seed_product(&conn, "P002", 1);
        seed_sale(&conn, "P001", "2026-03-10", 5, 2500, "auto");
        seed_sale(&conn, "P002", "2026-03-15", 1, 800, "auto");

        let report = get_monthly_sales(&conn, "2026-03", SalesMode::ByProduct).unwrap();
        assert_eq!(report.items.len(), 2);
        assert_eq!(report.items[0].key, "P001");
        assert_eq!(report.items[0].ranking, 1);
        assert_eq!(report.items[1].key, "P002");
        assert_eq!(report.items[1].ranking, 2);
    }

    #[test]
    fn test_monthly_sales_req502_by_department_normal() {
        // REQ-502: 月次売上（月次商品別/部門別集計）
        // 19.4: 部門別
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "P001", 1);
        seed_product(&conn, "P002", 3);
        seed_sale(&conn, "P001", "2026-03-10", 2, 1000, "auto");
        seed_sale(&conn, "P002", "2026-03-15", 5, 3000, "auto");

        let report = get_monthly_sales(&conn, "2026-03", SalesMode::ByDepartment).unwrap();
        assert_eq!(report.items.len(), 2);
        assert_eq!(report.items[0].key, "3"); // 毛糸 3000 > その他小物 1000
        assert_eq!(report.items[0].ranking, 1);
    }

    #[test]
    fn test_monthly_sales_req502_invalid_month_format() {
        // REQ-502: 月次売上（月次商品別/部門別集計）
        // 19.4: 不正形式
        let (_dir, conn) = setup_test_db();
        let err = get_monthly_sales(&conn, "2026/03", SalesMode::ByProduct).unwrap_err();
        assert!(matches!(err, BizError::ValidationFailed(_)));
    }

    #[test]
    fn test_monthly_sales_req502_invalid_month_13() {
        // REQ-502: 月次売上（月次商品別/部門別集計）
        // 19.4: 月=13
        let (_dir, conn) = setup_test_db();
        let err = get_monthly_sales(&conn, "2026-13", SalesMode::ByProduct).unwrap_err();
        assert!(matches!(err, BizError::ValidationFailed(_)));
    }

    #[test]
    fn test_monthly_sales_req502_ranking_order() {
        // REQ-502: 月次売上（月次商品別/部門別集計）
        // 19.4: amount降順 ranking=1,2,3
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "A", 1);
        seed_product(&conn, "B", 1);
        seed_product(&conn, "C", 1);
        seed_sale(&conn, "A", "2026-03-10", 1, 100, "auto");
        seed_sale(&conn, "B", "2026-03-10", 1, 300, "auto");
        seed_sale(&conn, "C", "2026-03-10", 1, 200, "auto");

        let report = get_monthly_sales(&conn, "2026-03", SalesMode::ByProduct).unwrap();
        assert_eq!(report.items[0].key, "B"); // 300
        assert_eq!(report.items[0].ranking, 1);
        assert_eq!(report.items[1].key, "C"); // 200
        assert_eq!(report.items[1].ranking, 2);
        assert_eq!(report.items[2].key, "A"); // 100
        assert_eq!(report.items[2].ranking, 3);
    }

    #[test]
    fn test_monthly_sales_req502_prev_month_comparison() {
        // REQ-502: 月次売上（月次商品別/部門別集計）
        // 19.4: 前月データあり -> Some(非空)
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "P001", 1);
        seed_sale(&conn, "P001", "2026-02-15", 2, 1000, "auto");
        seed_sale(&conn, "P001", "2026-03-15", 3, 1500, "auto");

        let report = get_monthly_sales(&conn, "2026-03", SalesMode::ByProduct).unwrap();
        assert_eq!(report.items.len(), 1);
        let prev = report.prev_month_comparison.as_ref().unwrap();
        assert_eq!(prev.len(), 1);
        assert_eq!(prev[0].amount, 1000);
    }

    #[test]
    fn test_monthly_sales_req502_prev_month_none() {
        // REQ-502: 月次売上（月次商品別/部門別集計）
        // 19.4: 前月データなし -> Some(空Vec)
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "P001", 1);
        seed_sale(&conn, "P001", "2026-03-15", 3, 1500, "auto");

        let report = get_monthly_sales(&conn, "2026-03", SalesMode::ByProduct).unwrap();
        let prev = report.prev_month_comparison.as_ref().unwrap();
        assert!(prev.is_empty());
        assert!(report.official_department_totals.is_none());
        assert!(report.official_report_dates.is_empty());
    }

    #[test]
    fn test_monthly_sales_req502_year_boundary() {
        // REQ-502: 月次売上（月次商品別/部門別集計）
        // 19.4: 2026-01 -> 前月 2025-12
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "P001", 1);
        seed_sale(&conn, "P001", "2025-12-20", 2, 1000, "auto");
        seed_sale(&conn, "P001", "2026-01-10", 3, 1500, "auto");

        let report = get_monthly_sales(&conn, "2026-01", SalesMode::ByProduct).unwrap();
        assert_eq!(report.items.len(), 1);
        assert_eq!(report.items[0].amount, 1500);

        let prev = report.prev_month_comparison.as_ref().unwrap();
        assert_eq!(prev.len(), 1);
        assert_eq!(prev[0].amount, 1000, "2025-12の前月データ");
    }

    #[test]
    fn test_monthly_sales_req502_with_returns() {
        // REQ-502: 月次売上（月次商品別/部門別集計）
        // 19.4: 返品（負数量）の集計
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "P001", 1);
        seed_sale(&conn, "P001", "2026-03-10", 5, 2500, "auto");
        seed_sale(&conn, "P001", "2026-03-15", -1, -500, "auto");

        let report = get_monthly_sales(&conn, "2026-03", SalesMode::ByProduct).unwrap();
        assert_eq!(report.items.len(), 1);
        assert_eq!(report.items[0].count_points, 4, "5 + (-1) = 4");
        assert_eq!(report.items[0].length_cm, 0);
        assert_eq!(report.items[0].stock_unit, Some(ProductStockUnit::Pcs));
        assert_eq!(report.items[0].amount, 2000, "2500 + (-500) = 2000");
    }

    #[test]
    fn test_get_monthly_sales_official_department_totals_req502() {
        // REQ-502: 月次売上公式部門集計
        // 19.4: 日報部門行はofficial_department_totalsで返す
        let (_dir, conn) = setup_test_db();
        let import_id = seed_daily_report(&conn, "2026-03-21", "official-monthly");
        seed_daily_report_lines(&conn, import_id, Some(1));

        let report = get_monthly_sales(&conn, "2026-03", SalesMode::ByProduct).unwrap();
        let official = report.official_department_totals.unwrap();
        assert_eq!(official.len(), 1);
        assert_eq!(official[0].department_id, Some(1));
        assert_eq!(official[0].label, "その他小物");
        assert_eq!(official[0].amount, 11000);
    }

    #[test]
    fn test_get_monthly_sales_req502_report_dates_follow_official_totals() {
        // REQ-502 / 19.4 手順 6 の不変条件: 日が空 ⇔ 公式部門集計が None（取消の前後）
        let (_dir, conn) = setup_test_db();
        let import_id = seed_daily_report(&conn, "2026-03-21", "dates-follow");
        seed_daily_report_lines(&conn, import_id, Some(1));

        let before = get_monthly_sales(&conn, "2026-03", SalesMode::ByProduct).unwrap();
        assert_eq!(before.official_report_dates, ["2026-03-21"]);
        assert!(before.official_department_totals.is_some());

        sales_repo::rollback_daily_report_import(&conn, import_id, "2026-03-22T10:00:00").unwrap();

        let after = get_monthly_sales(&conn, "2026-03", SalesMode::ByProduct).unwrap();
        assert!(after.official_report_dates.is_empty());
        assert!(after.official_department_totals.is_none());
    }

    #[test]
    fn test_get_monthly_sales_req502_report_dates_exclude_sale_record_only_days() {
        // REQ-502 / UI-09b-D8: Z004・手動販売だけの日は取り込んだ日に入らない
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "P001", 1);
        seed_sale(&conn, "P001", "2026-03-10", 1, 500, "auto");
        let import_id = seed_daily_report(&conn, "2026-03-23", "dates-sale-only");
        seed_daily_report_lines(&conn, import_id, Some(1));

        let report = get_monthly_sales(&conn, "2026-03", SalesMode::ByProduct).unwrap();
        assert_eq!(report.official_report_dates, ["2026-03-23"]);
    }

    #[test]
    fn test_get_monthly_sales_req502_report_dates_same_for_both_modes() {
        // REQ-502 / 19.4: 取り込んだ日は mode に依らない
        let (_dir, conn) = setup_test_db();
        let import_id = seed_daily_report(&conn, "2026-03-05", "dates-modes");
        seed_daily_report_lines(&conn, import_id, Some(1));

        let by_product = get_monthly_sales(&conn, "2026-03", SalesMode::ByProduct).unwrap();
        let by_department = get_monthly_sales(&conn, "2026-03", SalesMode::ByDepartment).unwrap();
        assert_eq!(by_product.official_report_dates, ["2026-03-05"]);
        assert_eq!(by_department.official_report_dates, ["2026-03-05"]);
    }

    #[test]
    fn test_get_monthly_sales_req502_report_dates_with_empty_department_lines() {
        // REQ-502 / 19.4 の抜け道: 部門の行が 0 の親の日は日に入り、集計は Some(空)
        let (_dir, conn) = setup_test_db();
        seed_daily_report(&conn, "2026-03-07", "dates-no-lines");

        let report = get_monthly_sales(&conn, "2026-03", SalesMode::ByProduct).unwrap();
        assert_eq!(report.official_report_dates, ["2026-03-07"]);
        assert_eq!(
            report.official_department_totals.map(|rows| rows.len()),
            Some(0)
        );
    }

    #[test]
    fn test_get_monthly_sales_no_fake_ranking_req502() {
        // REQ-502: 月次売上公式部門集計
        // 19.4: 日報部門行だけでは商品ランキング/商品別由来部門集計を水増ししない
        let (_dir, conn) = setup_test_db();
        let import_id = seed_daily_report(&conn, "2026-03-21", "official-no-ranking");
        seed_daily_report_lines(&conn, import_id, Some(1));

        let by_product = get_monthly_sales(&conn, "2026-03", SalesMode::ByProduct).unwrap();
        assert!(by_product.official_department_totals.is_some());
        assert!(by_product.items.is_empty());

        let by_department = get_monthly_sales(&conn, "2026-03", SalesMode::ByDepartment).unwrap();
        assert!(by_department.official_department_totals.is_some());
        assert!(by_department.items.is_empty());
    }

    // -------------------------------------------------------------------
    // export_sales_csv
    // -------------------------------------------------------------------

    #[test]
    fn test_export_sales_csv_req501_daily_normal() {
        // REQ-501: 日次売上CSVエクスポート — 正常系
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "P001", 1);
        seed_product(&conn, "P002", 3);
        seed_sale(&conn, "P001", "2026-03-21", 2, 1000, "auto");
        seed_sale(&conn, "P002", "2026-03-21", 1, 500, "manual");

        let result = export_sales_csv(&conn, &SalesReportType::Daily, "2026-03-21").unwrap();

        // 件数
        assert_eq!(result.count, 2);
        // ファイル名
        assert_eq!(result.suggested_filename, "sales_daily_2026-03-21.csv");
        // UTF-8 BOM
        assert_eq!(&result.csv_bytes[..3], &[0xEF, 0xBB, 0xBF]);
        // CSVの中身を検証
        let csv_str = String::from_utf8(result.csv_bytes[3..].to_vec()).unwrap();
        assert!(csv_str.starts_with("商品コード,商品名,部門,数量,金額,記録元\r\n"));
        assert!(csv_str.contains("P001"));
        assert!(csv_str.contains("P002"));
    }

    #[test]
    fn test_export_sales_csv_req501_daily_empty() {
        // REQ-501: 日次 — 売上0件 → ヘッダのみCSV
        let (_dir, conn) = setup_test_db();

        let result = export_sales_csv(&conn, &SalesReportType::Daily, "2026-03-21").unwrap();
        assert_eq!(result.count, 0);

        let csv_str = String::from_utf8(result.csv_bytes[3..].to_vec()).unwrap();
        assert!(csv_str.starts_with("商品コード,商品名,部門,数量,金額,記録元\r\n"));
        // ヘッダ行のみ（データ行なし）
        assert_eq!(csv_str.lines().count(), 1);
    }

    #[test]
    fn test_export_sales_csv_req501_daily_source() {
        // REQ-501: source翻訳 — "auto"→"POS"、"manual"→"手動"
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "P001", 1);
        seed_product(&conn, "P002", 1);
        seed_sale(&conn, "P001", "2026-03-21", 1, 500, "auto");
        seed_sale(&conn, "P002", "2026-03-21", 1, 300, "manual");

        let result = export_sales_csv(&conn, &SalesReportType::Daily, "2026-03-21").unwrap();
        let csv_str = String::from_utf8(result.csv_bytes[3..].to_vec()).unwrap();
        assert!(csv_str.contains("POS"), "auto → POS");
        assert!(csv_str.contains("手動"), "manual → 手動");
        assert!(!csv_str.contains(",auto"), "auto が生のまま残っていない");
        assert!(
            !csv_str.contains(",manual"),
            "manual が生のまま残っていない"
        );
    }

    #[test]
    fn test_export_sales_csv_req502_monthly_product() {
        // REQ-502: 月次商品別CSVエクスポート
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "P001", 1);
        seed_product(&conn, "P002", 1);
        seed_sale(&conn, "P001", "2026-03-10", 5, 2500, "auto");
        seed_sale(&conn, "P002", "2026-03-15", 1, 800, "auto");

        let result =
            export_sales_csv(&conn, &SalesReportType::MonthlyByProduct, "2026-03").unwrap();
        assert_eq!(result.count, 2);
        assert_eq!(
            result.suggested_filename,
            "sales_monthly_product_2026-03.csv"
        );

        let csv_str = String::from_utf8(result.csv_bytes[3..].to_vec()).unwrap();
        assert!(csv_str.starts_with("ランク,商品コード,商品名,数量,金額\r\n"));
        // ランク1がP001（金額2500）
        assert!(csv_str.contains("1,P001"));
    }

    #[test]
    fn test_export_sales_csv_req502_monthly_dept() {
        // REQ-502: 月次部門別CSVエクスポート
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "P001", 1);
        seed_product(&conn, "P002", 3);
        seed_sale(&conn, "P001", "2026-03-10", 2, 1000, "auto");
        seed_sale(&conn, "P002", "2026-03-15", 5, 3000, "auto");

        let result =
            export_sales_csv(&conn, &SalesReportType::MonthlyByDepartment, "2026-03").unwrap();
        assert_eq!(result.count, 2);
        assert_eq!(result.suggested_filename, "sales_monthly_dept_2026-03.csv");

        let csv_str = String::from_utf8(result.csv_bytes[3..].to_vec()).unwrap();
        assert!(csv_str.starts_with("ランク,部門名,数量,金額\r\n"));
    }

    #[test]
    fn test_export_sales_csv_req501_invalid_date() {
        // REQ-501: 不正日付 → ValidationFailed
        let (_dir, conn) = setup_test_db();
        let err = export_sales_csv(&conn, &SalesReportType::Daily, "2026/03/21").unwrap_err();
        assert!(matches!(err, BizError::ValidationFailed(_)));
    }

    #[test]
    fn test_get_daily_sales_req501_official_quantity_in_units() {
        // REQ-501 / §19.2: DB の 100 倍の整数を wire の単位の数（f64）に戻す
        let (_dir, conn) = setup_test_db();
        let import_id = seed_daily_report(&conn, "2026-03-21", "official-units");
        let line = |department_id: Option<i64>, name: &str, quantity_hundredths, sort_order| {
            NewDailyReportDepartmentLine {
                daily_report_import_id: import_id,
                source_file: "Z005".to_string(),
                department_id,
                raw_department_name: name.to_string(),
                normalized_department_name: Some(name.to_string()),
                amount: 100,
                quantity_hundredths,
                count: None,
                sort_order,
            }
        };
        sales_repo::insert_daily_report_department_lines(
            &conn,
            &[
                line(Some(1), "合成部門A", Some(130), 1),
                line(Some(2), "合成部門B", Some(400), 2),
                line(None, "合成部門C", None, 3),
            ],
        )
        .unwrap();

        let official = get_daily_sales(&conn, "2026-03-21")
            .unwrap()
            .official_daily_report
            .unwrap();
        let quantities: Vec<Option<f64>> = official
            .department_lines
            .iter()
            .map(|line| line.quantity)
            .collect();
        assert_eq!(quantities, vec![Some(1.3), Some(4.0), None]);
    }

    #[test]
    fn test_get_monthly_sales_req502_official_quantity_in_units() {
        // REQ-502 / §19.2: BIZ-08 で取り込んだ小数の個数が月次の wire で単位の数になる
        use crate::biz::daily_report_import_service::{
            commit_daily_report_import, parse_and_validate_daily_report, DailyReportInputFile,
        };
        fn file(name: &str, date: &str, rows: &str) -> DailyReportInputFile {
            let text = format!(
                "\"精算回数    \",\"0001\",\"\",\"\"\r\n\"日付        \",\"{date}\",\"\",\"\"\r\n\r\n\"レコード    \",\"キャラクター\",\"個数/件数   \",\"金額        \"\r\n{rows}"
            );
            let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode(&text);
            DailyReportInputFile {
                filename: name.to_string(),
                bytes: encoded.to_vec(),
                sd_relative_path: None,
                source_path: None,
            }
        }
        let (_dir, mut conn) = setup_test_db();
        let mut import_day = |date: &str, quantity: &str| {
            let parsed = parse_and_validate_daily_report(
                &conn,
                vec![
                    file(
                        "Z001_x.CSV",
                        date,
                        &format!("\"0001\",\"総売\",\"{quantity}\",\"1000\"\r\n\"0002\",\"純売\",\"1\",\"1000\"\r\n"),
                    ),
                    file("Z002_x.CSV", date, "\"0001\",\"現金\",\"1\",\"1000\"\r\n"),
                    file(
                        "Z005_x.CSV",
                        date,
                        &format!("\"0001\",\"その他小物\",\"{quantity}\",\"1000\"\r\n"),
                    ),
                ],
            )
            .unwrap();
            commit_daily_report_import(&mut conn, parsed.cached_preview, false, _dir.path())
                .unwrap();
        };

        import_day("2026-03-20", "1.3");
        import_day("2026-03-21", "1.2");

        let official = get_monthly_sales(&conn, "2026-03", SalesMode::ByDepartment)
            .unwrap()
            .official_department_totals
            .unwrap();
        assert_eq!(official.len(), 1);
        assert_eq!(official[0].label, "その他小物");
        assert_eq!(official[0].quantity, Some(2.5));
        let daily = get_daily_sales(&conn, "2026-03-20")
            .unwrap()
            .official_daily_report
            .unwrap();
        assert_eq!(daily.department_lines[0].quantity, Some(1.3));
    }

    // -------------------------------------------------------------------
    // 日計（Z001）の取込みごとの行（34 §19.2 / §19.3、D-096）
    // -------------------------------------------------------------------

    // (ラベル, 個数, 件数, 金額)
    type SummaryValues<'a, Q> = (&'a str, Option<Q>, Option<i64>, Option<i64>);

    fn seed_summary_lines(conn: &DbConnection, import_id: i64, lines: &[SummaryValues<'_, i64>]) {
        let rows: Vec<NewDailyReportSummaryLine> = lines
            .iter()
            .enumerate()
            .map(
                |(index, (label, quantity_hundredths, count, amount))| NewDailyReportSummaryLine {
                    daily_report_import_id: import_id,
                    source_file: "Z001".to_string(),
                    line_key: format!("summary_{index}"),
                    label: label.to_string(),
                    amount: *amount,
                    quantity_hundredths: *quantity_hundredths,
                    count: *count,
                    sort_order: index as i64 + 1,
                },
            )
            .collect();
        sales_repo::insert_daily_report_summary_lines(conn, &rows).unwrap();
    }

    fn summary_values(import: &OfficialDailySummaryImport) -> Vec<SummaryValues<'_, f64>> {
        import
            .lines
            .iter()
            .map(|line| (line.label.as_str(), line.quantity, line.count, line.amount))
            .collect()
    }

    #[test]
    fn test_get_daily_sales_req501_summary_imports_in_units_and_order() {
        // REQ-501 / 34 §19.2・§19.3 / D-104: 取込みごとの行を並びと値を変えずに写し、個数だけ単位の数にする
        let (_dir, conn) = setup_test_db();
        let first = seed_daily_report(&conn, "2026-03-21", "units-a");
        let second = seed_daily_report(&conn, "2026-03-21", "units-b");
        for (id, at) in [
            (first, "2026-03-21T19:00:00"),
            (second, "2026-03-21T18:05:30"),
        ] {
            conn.execute(
                "UPDATE daily_report_imports SET imported_at = ?1 WHERE id = ?2",
                rusqlite::params![at, id],
            )
            .unwrap();
        }
        seed_summary_lines(
            &conn,
            first,
            &[
                ("総売", Some(1250), None, Some(15800)),
                ("純売", None, Some(14), Some(14600)),
                ("部門11", None, None, None),
            ],
        );
        seed_summary_lines(&conn, second, &[("総売", Some(100), None, Some(500))]);

        let official = get_daily_sales(&conn, "2026-03-21")
            .unwrap()
            .official_daily_report
            .unwrap();
        assert_eq!(official.source_import_count, 2);
        assert_eq!(official.summary_imports.len(), 2);
        assert_eq!(official.summary_imports[0].daily_report_import_id, second);
        assert_eq!(
            official.summary_imports[0].imported_at,
            "2026-03-21T18:05:30"
        );
        assert_eq!(
            summary_values(&official.summary_imports[0]),
            vec![("総売", Some(1.0), None, Some(500))]
        );
        assert_eq!(official.summary_imports[1].daily_report_import_id, first);
        assert_eq!(
            official.summary_imports[1].imported_at,
            "2026-03-21T19:00:00"
        );
        assert_eq!(
            summary_values(&official.summary_imports[1]),
            vec![
                ("総売", Some(12.5), None, Some(15800)),
                ("純売", None, Some(14), Some(14600)),
                ("部門11", None, None, None),
            ]
        );
    }

    #[test]
    fn test_export_sales_csv_req501_daily_ignores_summary_imports() {
        // REQ-501 / 34 §19.5: 日次 CSV は商品別明細だけで、Z001 の行を出さない
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "P001", 1);
        seed_product(&conn, "P002", 3);
        seed_sale(&conn, "P001", "2026-03-21", 2, 1000, "auto");
        seed_sale(&conn, "P002", "2026-03-21", 1, 500, "manual");
        let import_id = seed_daily_report(&conn, "2026-03-21", "csv-z001");
        seed_summary_lines(
            &conn,
            import_id,
            &[
                ("総売", Some(1250), None, Some(15800)),
                ("純売", None, Some(14), Some(14600)),
                ("合成Z001行", None, Some(7), Some(98765)),
            ],
        );

        let result = export_sales_csv(&conn, &SalesReportType::Daily, "2026-03-21").unwrap();
        assert_eq!(result.count, 2);
        let csv_str = String::from_utf8(result.csv_bytes[3..].to_vec()).unwrap();
        let lines: Vec<&str> = csv_str.lines().collect();
        assert_eq!(lines[0], "商品コード,商品名,部門,数量,金額,記録元");
        assert_eq!(lines.len(), 3);
        assert!(lines[1].starts_with("P001,"));
        assert!(lines[2].starts_with("P002,"));
        assert!(!csv_str.contains("合成Z001行"));
        assert!(!csv_str.contains("98765"));
    }

    #[test]
    fn test_get_daily_sales_req501_summary_imports_from_fixture_bundles() {
        // REQ-501 / D-096 / L3 fixture: 実 parser → 取込み → 日次の読み出しで Z001 の全行が取込みごとに出る
        use crate::biz::daily_report_import_service::{
            commit_daily_report_import, parse_and_validate_daily_report, DailyReportInputFile,
        };
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/fixtures/daily-report-z001");
        let (_dir, mut conn) = setup_test_db();
        for (suffix, additional) in [
            ("260323_0001", false),
            ("260324_0001", false),
            ("260324_0002", true),
        ] {
            let files = ["Z001", "Z002", "Z005"]
                .into_iter()
                .map(|kind| {
                    let filename = format!("{kind}_{suffix}.CSV");
                    DailyReportInputFile {
                        bytes: std::fs::read(dir.join(&filename)).unwrap(),
                        filename,
                        sd_relative_path: None,
                        source_path: None,
                    }
                })
                .collect();
            let parsed = parse_and_validate_daily_report(&conn, files).unwrap();
            commit_daily_report_import(&mut conn, parsed.cached_preview, additional, _dir.path())
                .unwrap();
        }
        let labels = [
            "総売",
            "純売",
            "部門01",
            "部門02",
            "部門03",
            "部門04",
            "部門05",
            "部門06",
            "部門07",
            "部門08",
            "部門09",
            "部門10",
            "現金在高",
            "券在高",
            "信用在高",
            "税対象額",
            "税額",
            "非課税",
            "高額券枚数",
            "丸め",
            "取引中止",
            "戻モード",
            "電卓",
            "領収書",
            "部門11",
            "部門12",
            "部門13",
            "部門14",
        ];
        let find = |import: &OfficialDailySummaryImport, label: &str| {
            let line = import
                .lines
                .iter()
                .find(|line| line.label == label)
                .unwrap();
            (line.quantity, line.count, line.amount)
        };

        let day_a = get_daily_sales(&conn, "2026-03-23")
            .unwrap()
            .official_daily_report
            .unwrap();
        assert_eq!(day_a.summary_imports.len(), 1);
        let a = &day_a.summary_imports[0];
        let a_labels: Vec<&str> = a.lines.iter().map(|line| line.label.as_str()).collect();
        assert_eq!(a_labels, labels);
        assert_eq!(find(a, "総売"), (Some(12.5), None, Some(15800)));
        assert_eq!(find(a, "税対象額").1, None);
        assert_eq!(find(a, "領収書").2, None);
        assert_eq!(find(a, "戻モード"), (None, Some(1), Some(-1200)));

        let day_b = get_daily_sales(&conn, "2026-03-24")
            .unwrap()
            .official_daily_report
            .unwrap();
        assert_eq!(day_b.source_import_count, 2);
        assert_eq!(day_b.summary_imports.len(), 2);
        let (b1, b2) = (&day_b.summary_imports[0], &day_b.summary_imports[1]);
        assert!(b1.daily_report_import_id < b2.daily_report_import_id);
        for import in [b1, b2] {
            let import_labels: Vec<&str> = import
                .lines
                .iter()
                .map(|line| line.label.as_str())
                .collect();
            assert_eq!(import_labels, labels);
        }
        assert_eq!(find(b1, "部門06"), (None, Some(-1), Some(-1200)));
        assert_eq!(find(b1, "総売").2, Some(7300));
        assert_eq!(find(b2, "総売").2, Some(500));
    }

    // -------------------------------------------------------------------
    // 単位の拡張の後の数量（34 冒頭、SPEC-UNIT-D3 の集計の規則、X9）
    // -------------------------------------------------------------------

    fn seed_product_unit(conn: &DbConnection, code: &str, dept_id: i64, unit: &str) {
        seed_product(conn, code, dept_id);
        conn.execute(
            "UPDATE products SET stock_unit = ?1 WHERE product_code = ?2",
            rusqlite::params![unit, code],
        )
        .unwrap();
    }

    fn csv_lines(conn: &DbConnection, report_type: SalesReportType, target: &str) -> Vec<String> {
        let result = export_sales_csv(conn, &report_type, target).unwrap();
        String::from_utf8(result.csv_bytes[3..].to_vec())
            .unwrap()
            .lines()
            .skip(1)
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn test_daily_sales_req501_totals_split_by_unit_kind() {
        // REQ-501 / SPEC-UNIT-D3（TD-206/207）: 個数は点、長さは cm の 2 本に分け、長さを点に足さない
        let (_dir, conn) = setup_test_db();
        seed_product_unit(&conn, "K-PCS", 1, "pcs");
        seed_product_unit(&conn, "K-SHEET", 1, "sheet");
        seed_product_unit(&conn, "K-M", 1, "m");
        seed_product_unit(&conn, "K-CM", 1, "m");
        seed_sale(&conn, "K-PCS", "2026-03-21", 3, 2100, "auto");
        seed_sale(&conn, "K-SHEET", "2026-03-21", 2, 400, "auto");
        seed_sale(&conn, "K-M", "2026-03-21", 130, 910, "manual");
        seed_sale(&conn, "K-CM", "2026-03-21", 50, 350, "manual");
        // 個数の単位だけの日、長さだけの日
        seed_product_unit(&conn, "K-BALL", 1, "ball");
        seed_sale(&conn, "K-PCS", "2026-03-22", 3, 2100, "auto");
        seed_sale(&conn, "K-BALL", "2026-03-22", 2, 600, "auto");
        seed_sale(&conn, "K-M", "2026-03-23", 130, 910, "manual");

        let report = get_daily_sales(&conn, "2026-03-21").unwrap();
        let units: Vec<(&str, ProductStockUnit, i64)> = report
            .items
            .iter()
            .map(|item| (item.product_code.as_str(), item.stock_unit, item.quantity))
            .collect();
        assert_eq!(
            units,
            vec![
                ("K-CM", ProductStockUnit::M, 50),
                ("K-M", ProductStockUnit::M, 130),
                ("K-PCS", ProductStockUnit::Pcs, 3),
                ("K-SHEET", ProductStockUnit::Sheet, 2),
            ]
        );
        let subtotal = &report.department_subtotals[0];
        assert_eq!((subtotal.count_points, subtotal.length_cm), (5, 180));
        assert_eq!(subtotal.amount, 3760);
        assert_eq!(
            (
                report.grand_total.count_points,
                report.grand_total.length_cm
            ),
            (5, 180)
        );

        let report = get_daily_sales(&conn, "2026-03-22").unwrap();
        assert_eq!(
            (
                report.grand_total.count_points,
                report.grand_total.length_cm
            ),
            (5, 0)
        );
        let report = get_daily_sales(&conn, "2026-03-23").unwrap();
        assert_eq!(
            (
                report.grand_total.count_points,
                report.grand_total.length_cm
            ),
            (0, 130)
        );
        let subtotal = &report.department_subtotals[0];
        assert_eq!((subtotal.count_points, subtotal.length_cm), (0, 130));
    }

    #[test]
    fn test_monthly_sales_req502_totals_split_by_unit_kind() {
        // REQ-502 / SPEC-UNIT-D3・D11: 商品別の行は種類のどちらか一方と stock_unit、部門別は 2 本で stock_unit は null
        let (_dir, conn) = setup_test_db();
        seed_product_unit(&conn, "N-M", 1, "m");
        seed_product_unit(&conn, "N-PCS", 1, "pcs");
        seed_product_unit(&conn, "N-M-ZERO", 3, "m");
        seed_product_unit(&conn, "N-PCS-ZERO", 3, "pcs");
        seed_sale(&conn, "N-M", "2026-03-05", 130, 910, "manual");
        seed_sale(&conn, "N-M", "2026-03-20", 120, 840, "manual");
        seed_sale(&conn, "N-PCS", "2026-03-10", 5, 1000, "auto");
        // 販売と返品が相殺して 0 になった行も種類を失わない
        seed_sale(&conn, "N-M-ZERO", "2026-03-10", 130, 910, "manual");
        seed_sale(&conn, "N-M-ZERO", "2026-03-11", -130, -910, "manual");
        seed_sale(&conn, "N-PCS-ZERO", "2026-03-10", 3, 300, "auto");
        seed_sale(&conn, "N-PCS-ZERO", "2026-03-11", -3, -300, "auto");

        let by_product = get_monthly_sales(&conn, "2026-03", SalesMode::ByProduct).unwrap();
        let rows: Vec<(&str, i64, i64, Option<ProductStockUnit>)> = by_product
            .items
            .iter()
            .map(|item| {
                (
                    item.key.as_str(),
                    item.count_points,
                    item.length_cm,
                    item.stock_unit,
                )
            })
            .collect();
        assert_eq!(
            rows,
            vec![
                ("N-M", 0, 250, Some(ProductStockUnit::M)),
                ("N-PCS", 5, 0, Some(ProductStockUnit::Pcs)),
                ("N-M-ZERO", 0, 0, Some(ProductStockUnit::M)),
                ("N-PCS-ZERO", 0, 0, Some(ProductStockUnit::Pcs)),
            ]
        );

        // 部門別: repo は部門と単位ごとの 2 行、BIZ は部門ごとの 1 行にまとめて金額の降順で ranking を振る
        let repo_rows =
            sales_repo::get_monthly_sales_by_department(&conn, "2026-03-01", "2026-03-31").unwrap();
        assert_eq!(repo_rows.iter().filter(|r| r.department_id == 1).count(), 2);
        let by_department = get_monthly_sales(&conn, "2026-03", SalesMode::ByDepartment).unwrap();
        let rows: Vec<_> = by_department
            .items
            .iter()
            .map(|item| {
                (
                    item.key.as_str(),
                    item.count_points,
                    item.length_cm,
                    item.amount,
                    item.stock_unit,
                    item.ranking,
                )
            })
            .collect();
        assert_eq!(
            rows,
            vec![("1", 5, 250, 2750, None, 1), ("3", 0, 0, 0, None, 2)]
        );
    }

    #[test]
    fn test_sales_req501_req502_boundary_quantities_are_not_rejected() {
        // REQ-501 / REQ-502 / X9: 上限の端の数量の集計は範囲の検査で拒まず、正確に出る
        let (_dir, conn) = setup_test_db();
        seed_product_unit(&conn, "B-A", 1, "m");
        seed_product_unit(&conn, "B-B", 1, "m");
        seed_sale(&conn, "B-A", "2026-03-21", 9_999_999, 100, "manual");
        seed_sale(&conn, "B-A", "2026-03-21", 9_999_999, 100, "manual");
        seed_sale(&conn, "B-B", "2026-03-21", -9_999_999, -100, "manual");

        let report = get_daily_sales(&conn, "2026-03-21").unwrap();
        let quantities: Vec<i64> = report.items.iter().map(|item| item.quantity).collect();
        assert_eq!(quantities, vec![19_999_998, -9_999_999]);
        let subtotal = &report.department_subtotals[0];
        assert_eq!((subtotal.count_points, subtotal.length_cm), (0, 9_999_999));
        assert_eq!(
            (
                report.grand_total.count_points,
                report.grand_total.length_cm
            ),
            (0, 9_999_999)
        );

        let by_product = get_monthly_sales(&conn, "2026-03", SalesMode::ByProduct).unwrap();
        let lengths: Vec<(&str, i64)> = by_product
            .items
            .iter()
            .map(|item| (item.key.as_str(), item.length_cm))
            .collect();
        assert_eq!(lengths, vec![("B-A", 19_999_998), ("B-B", -9_999_999)]);
        let by_department = get_monthly_sales(&conn, "2026-03", SalesMode::ByDepartment).unwrap();
        assert_eq!(
            (
                by_department.items[0].count_points,
                by_department.items[0].length_cm
            ),
            (0, 9_999_999)
        );
    }

    #[test]
    fn test_daily_sales_req501_total_overflow_is_validation_failed() {
        // REQ-501 / X9: BIZ の和は checked。i64 の溢れは wrap した値を返さず ValidationFailed
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "OV-1", 1);
        seed_product(&conn, "OV-2", 1);
        seed_sale(&conn, "OV-1", "2026-03-21", i64::MAX, 100, "auto");
        seed_sale(&conn, "OV-2", "2026-03-21", 1, 100, "auto");

        let rows = sales_repo::get_daily_sales_records(&conn, "2026-03-21").unwrap();
        assert_eq!(rows.len(), 2, "repo は商品ごとの 2 行を返す");
        let result = get_daily_sales(&conn, "2026-03-21");
        assert!(
            matches!(result, Err(BizError::ValidationFailed(ref m)) if m.contains("オーバーフロー")),
            "{result:?}"
        );
    }

    #[test]
    fn test_export_sales_csv_req501_req502_quantity_in_display_unit() {
        // REQ-501 / REQ-502 / 34 §19.5（X9）: 商品の行の長さは m の数（単位の語・桁区切りなし）、
        // 月次の部門別は長さが 0 なら今と同じ整数、そうでなければ画面と同じ文字列
        let (_dir, conn) = setup_test_db();
        seed_product_unit(&conn, "C1-M", 1, "m");
        seed_product_unit(&conn, "C2-CM", 1, "m");
        seed_product_unit(&conn, "C3-M", 1, "m");
        seed_product_unit(&conn, "C4-M", 1, "m");
        seed_product_unit(&conn, "C5-M", 1, "m");
        seed_product_unit(&conn, "C6-PCS", 1, "pcs");
        seed_sale(&conn, "C1-M", "2026-03-21", 130, 910, "manual");
        seed_sale(&conn, "C2-CM", "2026-03-21", 50, 350, "manual");
        seed_sale(&conn, "C3-M", "2026-03-21", 123_456, 800, "manual");
        seed_sale(&conn, "C4-M", "2026-03-21", 200, 700, "manual");
        seed_sale(&conn, "C5-M", "2026-03-21", -130, -910, "manual");
        seed_sale(&conn, "C6-PCS", "2026-03-21", 3, 2100, "auto");

        assert_eq!(
            csv_lines(&conn, SalesReportType::Daily, "2026-03-21"),
            vec![
                "C1-M,商品C1-M,その他小物,1.3,910,手動",
                "C2-CM,商品C2-CM,その他小物,0.5,350,手動",
                "C3-M,商品C3-M,その他小物,1234.56,800,手動",
                "C4-M,商品C4-M,その他小物,2,700,手動",
                "C5-M,商品C5-M,その他小物,-1.3,-910,手動",
                "C6-PCS,商品C6-PCS,その他小物,3,2100,POS",
            ]
        );
        let monthly = csv_lines(&conn, SalesReportType::MonthlyByProduct, "2026-03");
        assert!(
            monthly.contains(&"1,C6-PCS,商品C6-PCS,3,2100".to_string()),
            "{monthly:?}"
        );
        assert!(
            monthly.contains(&"2,C1-M,商品C1-M,1.3,910".to_string()),
            "{monthly:?}"
        );
        assert!(
            monthly.contains(&"6,C5-M,商品C5-M,-1.3,-910".to_string()),
            "{monthly:?}"
        );

        // 月次の部門別: 4 つの部門（個数と長さ・個数だけ・長さだけが相殺して 0・長さだけ）
        let (_dir, conn) = setup_test_db();
        seed_product_unit(&conn, "D1-PCS", 1, "pcs");
        seed_product_unit(&conn, "D1-SHEET", 1, "sheet");
        seed_product_unit(&conn, "D1-M", 1, "m");
        seed_product_unit(&conn, "D1-CM", 1, "m");
        seed_product_unit(&conn, "D2-PCS", 2, "pcs");
        seed_product_unit(&conn, "D3-M", 3, "m");
        seed_product_unit(&conn, "D4-M", 4, "m");
        seed_sale(&conn, "D1-PCS", "2026-03-21", 3, 4000, "auto");
        seed_sale(&conn, "D1-SHEET", "2026-03-21", 2, 0, "auto");
        seed_sale(&conn, "D1-M", "2026-03-21", 130, 0, "manual");
        seed_sale(&conn, "D1-CM", "2026-03-21", 50, 0, "manual");
        seed_sale(&conn, "D2-PCS", "2026-03-21", 5, 3000, "auto");
        seed_sale(&conn, "D3-M", "2026-03-21", 130, 2000, "manual");
        seed_sale(&conn, "D3-M", "2026-03-22", -130, 0, "manual");
        seed_sale(&conn, "D4-M", "2026-03-21", 130, 1000, "manual");
        let lines = csv_lines(&conn, SalesReportType::MonthlyByDepartment, "2026-03");
        let quantities: Vec<&str> = lines
            .iter()
            .map(|line| line.split(',').nth(2).unwrap())
            .collect();
        assert_eq!(quantities, vec!["5 点・1.8 m", "5", "0", "1.3 m"]);
    }

    #[test]
    fn test_export_sales_csv_req502_total_quantity_groups_thousands() {
        // REQ-502 / SPEC-UNIT-D3 の表の 5（X9）: 集計の行の `N 点` の `N` は 3 桁区切り（画面と同じ golden）。
        // `length_cm` が 0 の部門は区切りも語も付けない整数、`count_points` が 0 の部門は m の形
        for (count, length, expected) in [
            (5, 180, "5 点・1.8 m"),
            (999, 180, "999 点・1.8 m"),
            (1000, 180, "1,000 点・1.8 m"),
            (1234, 180, "1,234 点・1.8 m"),
            (-1234, 180, "-1,234 点・1.8 m"),
            (1_234_567, 123_456, "1,234,567 点・1,234.56 m"),
            (1234, 0, "1234"),
            (-1234, 0, "-1234"),
            (0, 0, "0"),
            (0, 123_456, "1,234.56 m"),
        ] {
            assert_eq!(csv_total_quantity(count, length), expected);
        }

        // CSV の行: `,` を含む field は exporter が `"…"` で括る
        let (_dir, conn) = setup_test_db();
        seed_product_unit(&conn, "G1-PCS", 1, "pcs");
        seed_product_unit(&conn, "G1-M", 1, "m");
        seed_product_unit(&conn, "G2-PCS", 2, "pcs");
        seed_sale(&conn, "G1-PCS", "2026-03-21", 1234, 5000, "auto");
        seed_sale(&conn, "G1-M", "2026-03-21", 180, 0, "manual");
        seed_sale(&conn, "G2-PCS", "2026-03-21", 1234, 3000, "auto");
        let lines = csv_lines(&conn, SalesReportType::MonthlyByDepartment, "2026-03");
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert!(lines[0].contains(",\"1,234 点・1.8 m\","), "{lines:?}");
        assert!(lines[1].contains(",1234,"), "{lines:?}");
    }

    // -------------------------------------------------------------------
    // SALES-SHEET-D2 の関数（34 §19.3a。期待は 34 の例と Matrix から転記）
    // -------------------------------------------------------------------

    const OVERFLOW_MESSAGE: &str = "売上集計の計算でオーバーフローが発生しました";
    const LABEL_CHANGED_WARNING: &str = "支払集計（Z002）は、この日の精算の間で行の名称か並びが変わっています。同じ名称の行が 2 行あれば、別々に足しています";
    const UNSUMMED_WARNING: &str = "支払集計（Z002）は、この日の精算の間で行の名称か並びが変わり、1 回の精算の中に同じ名称の行が 2 行以上あるため、足さずに取込みの古い順に並べています。紙（日報を印刷）で取込みごとに見られます";

    fn rule_line(import_id: i64, position: i64, label: &str, amount: Option<i64>) -> SheetRuleLine {
        SheetRuleLine {
            import_id,
            tie: import_id * 1000 + position,
            position,
            label: label.to_string(),
            quantity_hundredths: None,
            count: Some(1),
            amount,
        }
    }

    /// (位置, 名称, 金額, import_seq) の並びで見る
    fn rule_view(rows: &[SheetRuleRow]) -> Vec<(i64, &str, Option<i64>, Option<i64>)> {
        rows.iter()
            .map(|row| (row.position, row.label.as_str(), row.amount, row.import_seq))
            .collect()
    }

    #[test]
    fn test_sheet_rows_keep_same_position_different_label_separate() {
        // REQ-501 / SALES-SHEET-D2 の 2: 同じ位置で名称が違う行は別の行
        let (case, rows) = apply_sheet_rule(
            &[
                rule_line(1, 4, "項目A", Some(10)),
                rule_line(2, 4, "項目B", Some(20)),
            ],
            &[1, 2],
        )
        .unwrap();
        assert_eq!(case, SheetRuleCase::LabelChanged);
        assert_eq!(
            rule_view(&rows),
            [(4, "項目A", Some(10), None), (4, "項目B", Some(20), None)]
        );
    }

    #[test]
    fn test_sheet_rows_split_by_position_when_rows_shift() {
        // REQ-501 / SALES-SHEET-D2 の 2: 行が 1 つずれた日は同じ名称が位置ごとの行
        let (case, rows) = apply_sheet_rule(
            &[
                rule_line(1, 15, "項目A", Some(10)),
                rule_line(1, 16, "項目B", Some(20)),
                rule_line(2, 15, "項目X", Some(5)),
                rule_line(2, 16, "項目A", Some(30)),
                rule_line(2, 17, "項目B", Some(40)),
            ],
            &[1, 2],
        )
        .unwrap();
        assert_eq!(case, SheetRuleCase::LabelChanged);
        assert_eq!(
            rule_view(&rows),
            [
                (15, "項目A", Some(10), None),
                (15, "項目X", Some(5), None),
                (16, "項目B", Some(20), None),
                (16, "項目A", Some(30), None),
                (17, "項目B", Some(40), None),
            ]
        );
    }

    #[test]
    fn test_sheet_rows_unsummed_when_rows_shift_with_duplicate_label() {
        // REQ-501 / SALES-SHEET-D2 の 3: 行がずれ 1 本の中に同じ名称があれば足さない（34 の例）
        let (case, rows) = apply_sheet_rule(
            &[
                rule_line(1, 1, "項目A", Some(10)),
                rule_line(1, 2, "項目A", Some(20)),
                rule_line(2, 1, "項目X", Some(5)),
                rule_line(2, 2, "項目A", Some(30)),
                rule_line(2, 3, "項目A", Some(40)),
            ],
            &[1, 2],
        )
        .unwrap();
        assert_eq!(case, SheetRuleCase::Unsummed);
        assert_eq!(
            rule_view(&rows),
            [
                (1, "項目A", Some(10), Some(1)),
                (2, "項目A", Some(20), Some(1)),
                (1, "項目X", Some(5), Some(2)),
                (2, "項目A", Some(30), Some(2)),
                (3, "項目A", Some(40), Some(2)),
            ]
        );
    }

    #[test]
    fn test_sheet_rows_import_seq_follows_import_ids() {
        // REQ-501 / SALES-SHEET-D2 の 3: import_seq は import_ids の位置（行の無い取込みも番号を使う）
        let (case, rows) = apply_sheet_rule(
            &[
                rule_line(11, 1, "項目A", Some(10)),
                rule_line(11, 2, "項目A", Some(20)),
                rule_line(13, 1, "項目X", Some(5)),
                rule_line(13, 2, "項目A", Some(30)),
                rule_line(13, 3, "項目A", Some(40)),
            ],
            &[11, 12, 13],
        )
        .unwrap();
        assert_eq!(case, SheetRuleCase::Unsummed);
        let seqs: Vec<Option<i64>> = rows.iter().map(|row| row.import_seq).collect();
        assert_eq!(seqs, [Some(1), Some(1), Some(3), Some(3), Some(3)]);
    }

    #[test]
    fn test_sheet_rows_blank_labels_do_not_trigger_unsummed() {
        // REQ-501 / SALES-SHEET-D2: 名称が空の同じ行は 3 の判定に数えない
        let (case, rows) = apply_sheet_rule(
            &[
                rule_line(1, 1, "項目A", Some(10)),
                rule_line(1, 2, "", Some(0)),
                rule_line(1, 3, "", Some(0)),
                rule_line(2, 1, "項目X", Some(5)),
                rule_line(2, 2, "項目A", Some(30)),
                rule_line(2, 3, "", Some(0)),
                rule_line(2, 4, "", Some(0)),
            ],
            &[1, 2],
        )
        .unwrap();
        assert_eq!(case, SheetRuleCase::LabelChanged);
        let view = rule_view(&rows);
        for expected in [
            (1, "項目A", Some(10), None),
            (1, "項目X", Some(5), None),
            (2, "項目A", Some(30), None),
        ] {
            assert!(view.contains(&expected), "{expected:?} in {view:?}");
        }
    }

    #[test]
    fn test_sheet_rows_no_notice_when_label_sets_match() {
        // REQ-501 / SALES-SHEET-D2 の 1: 組の集合が同じなら組で足す
        let (case, rows) = apply_sheet_rule(
            &[
                rule_line(1, 1, "現金", Some(100)),
                rule_line(1, 2, "項目B", Some(30)),
                rule_line(2, 1, "現金", Some(50)),
                rule_line(2, 2, "項目B", Some(7)),
            ],
            &[1, 2],
        )
        .unwrap();
        assert_eq!(case, SheetRuleCase::Summed);
        assert_eq!(
            rule_view(&rows),
            [(1, "現金", Some(150), None), (2, "項目B", Some(37), None)]
        );
    }

    #[test]
    fn test_sheet_rows_do_not_merge_same_label_at_different_positions() {
        // REQ-501 / SALES-SHEET-D2: 名称だけでは足さない（位置 7 と 33 の同じ名称）
        let (case, rows) = apply_sheet_rule(
            &[
                rule_line(1, 7, "名称", Some(10)),
                rule_line(1, 33, "名称", Some(20)),
                rule_line(2, 7, "名称", Some(10)),
                rule_line(2, 33, "名称", Some(20)),
            ],
            &[1, 2],
        )
        .unwrap();
        assert_eq!(case, SheetRuleCase::Summed);
        assert_eq!(
            rule_view(&rows),
            [(7, "名称", Some(20), None), (33, "名称", Some(40), None)]
        );
    }

    #[test]
    fn test_sheet_rows_propagate_null() {
        // REQ-501 / SALES-SHEET-D2 の足す値 (a): NULL のある組は NULL
        let (_, rows) = apply_sheet_rule(
            &[
                rule_line(1, 1, "現金", None),
                rule_line(2, 1, "現金", Some(20)),
            ],
            &[1, 2],
        )
        .unwrap();
        assert_eq!(rule_view(&rows), [(1, "現金", None, None)]);
    }

    #[test]
    fn test_sheet_rows_sum_quantity_in_hundredths() {
        // REQ-501 / SALES-SHEET-D2・IO-07-D2: 個数は 100 倍の整数で足す（1.3 + 2 → 3.3）
        let line = |import_id, quantity| SheetRuleLine {
            quantity_hundredths: Some(quantity),
            ..rule_line(import_id, 1, "総売", Some(1))
        };
        let (_, rows) = apply_sheet_rule(&[line(1, 130), line(2, 200)], &[1, 2]).unwrap();
        assert_eq!(rows[0].quantity_hundredths, Some(330));
        assert_eq!(quantity_hundredths_to_units(330), 3.3);
    }

    #[test]
    fn test_sheet_rows_order_by_position_then_first_seen_import() {
        // REQ-501 / SALES-SHEET-D2 の並び: 位置の昇順、同じ位置は最初に現れた取込みの古い順
        let (_, rows) = apply_sheet_rule(
            &[
                rule_line(1, 2, "項目B", Some(1)),
                rule_line(2, 2, "項目A", Some(2)),
                rule_line(2, 1, "項目C", Some(3)),
            ],
            &[1, 2],
        )
        .unwrap();
        assert_eq!(
            rule_view(&rows),
            [
                (1, "項目C", Some(3), None),
                (2, "項目B", Some(1), None),
                (2, "項目A", Some(2), None),
            ]
        );
    }

    #[test]
    fn test_sheet_rows_overflow_is_validation_failed() {
        // REQ-501 / SALES-SHEET-D2・D5: 足す値の溢れは ValidationFailed
        let result = apply_sheet_rule(
            &[
                rule_line(1, 1, "現金", Some(i64::MAX)),
                rule_line(2, 1, "現金", Some(1)),
            ],
            &[1, 2],
        );
        assert!(
            matches!(result, Err(BizError::ValidationFailed(message)) if message == OVERFLOW_MESSAGE)
        );
    }

    #[test]
    fn test_sheet_rows_add_in_import_id_order() {
        // REQ-501 / SALES-SHEET-D2 の (b): 足す順は取込みの id の昇順（import_ids の並びでない）
        let result = apply_sheet_rule(
            &[
                rule_line(13, 1, "現金", Some(-i64::MAX)),
                rule_line(12, 1, "現金", Some(1)),
                rule_line(11, 1, "現金", Some(i64::MAX)),
            ],
            &[13, 12, 11],
        );
        assert!(
            matches!(result, Err(BizError::ValidationFailed(message)) if message == OVERFLOW_MESSAGE)
        );
    }

    #[test]
    fn test_sheet_rows_null_wins_over_overflow() {
        // REQ-501 / SALES-SHEET-D2 の (a): NULL が溢れより先。件数は足す
        let (_, rows) = apply_sheet_rule(
            &[
                rule_line(1, 1, "現金", Some(i64::MAX)),
                rule_line(2, 1, "現金", Some(1)),
                rule_line(3, 1, "現金", None),
            ],
            &[1, 2, 3],
        )
        .unwrap();
        assert_eq!(rows[0].amount, None);
        assert_eq!(rows[0].count, Some(3));
    }

    // -------------------------------------------------------------------
    // get_daily_report_sheet と画面の支払集計（実 DB）
    // -------------------------------------------------------------------

    type SheetLine<'a> = (i64, &'a str, Option<i64>, Option<i64>, Option<i64>);

    /// 日報の取込みを 1 つ入れる。行は (位置, 名称, 個数の 100 倍, 件数, 金額)
    fn seed_sheet_import(
        conn: &DbConnection,
        date: &str,
        hash: &str,
        z001: &[SheetLine],
        z002: &[SheetLine],
        z005: &[SheetLine],
    ) -> i64 {
        let import_id = seed_daily_report(conn, date, hash);
        sales_repo::insert_daily_report_summary_lines(
            conn,
            &z001
                .iter()
                .map(
                    |(position, label, quantity, count, amount)| NewDailyReportSummaryLine {
                        daily_report_import_id: import_id,
                        source_file: "Z001".to_string(),
                        line_key: format!("summary_{position}"),
                        label: label.to_string(),
                        amount: *amount,
                        quantity_hundredths: *quantity,
                        count: *count,
                        sort_order: *position,
                    },
                )
                .collect::<Vec<_>>(),
        )
        .unwrap();
        sales_repo::insert_daily_report_payment_lines(
            conn,
            &z002
                .iter()
                .map(
                    |(position, label, _, count, amount)| NewDailyReportPaymentLine {
                        daily_report_import_id: import_id,
                        source_file: "Z002".to_string(),
                        payment_key: if label.trim() == "現金" {
                            "cash".to_string()
                        } else {
                            format!("payment_{position}")
                        },
                        label: label.to_string(),
                        amount: *amount,
                        count: *count,
                        sort_order: *position,
                    },
                )
                .collect::<Vec<_>>(),
        )
        .unwrap();
        sales_repo::insert_daily_report_department_lines(
            conn,
            &z005
                .iter()
                .map(
                    |(position, label, quantity, count, amount)| NewDailyReportDepartmentLine {
                        daily_report_import_id: import_id,
                        source_file: "Z005".to_string(),
                        department_id: Some(1),
                        raw_department_name: label.to_string(),
                        normalized_department_name: Some(label.to_string()),
                        amount: amount.unwrap_or(0),
                        quantity_hundredths: *quantity,
                        count: *count,
                        sort_order: *position,
                    },
                )
                .collect::<Vec<_>>(),
        )
        .unwrap();
        import_id
    }

    fn set_imported_at(conn: &DbConnection, import_id: i64, imported_at: &str) {
        conn.execute(
            "UPDATE daily_report_imports SET imported_at = ?1 WHERE id = ?2",
            rusqlite::params![imported_at, import_id],
        )
        .unwrap();
    }

    fn sheet_view(rows: &[DailyReportSheetRow]) -> Vec<(i64, &str, Option<i64>, Option<i64>)> {
        rows.iter()
            .map(|row| (row.position, row.label.as_str(), row.amount, row.import_seq))
            .collect()
    }

    fn payment_view(
        official: &OfficialDailyReportSummary,
    ) -> Vec<(&str, &str, Option<i64>, Option<i64>)> {
        official
            .payment_lines
            .iter()
            .map(|line| {
                (
                    line.payment_key.as_str(),
                    line.label.as_str(),
                    line.amount,
                    line.count,
                )
            })
            .collect()
    }

    fn table_counts(conn: &DbConnection) -> Vec<i64> {
        [
            "daily_report_imports",
            "daily_report_summary_lines",
            "daily_report_payment_lines",
            "daily_report_department_lines",
            "sale_records",
            "csv_imports",
            "operation_logs",
        ]
        .iter()
        .map(|table| {
            conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap()
        })
        .collect()
    }

    /// 34 SALES-SHEET-D2 の 3 の例の Z002 の 2 取込み
    const SHIFTED_Z002_FIRST: [SheetLine<'static>; 2] = [
        (1, "項目A", None, Some(1), Some(10)),
        (2, "項目A", None, Some(1), Some(20)),
    ];
    const SHIFTED_Z002_SECOND: [SheetLine<'static>; 3] = [
        (1, "項目X", None, Some(1), Some(5)),
        (2, "項目A", None, Some(1), Some(30)),
        (3, "項目A", None, Some(1), Some(40)),
    ];

    #[test]
    fn test_daily_report_sheet_sums_all_completed_imports_of_the_day() {
        // REQ-501 / SALES-SHEET-D1・D6: 同じ日の 2 取込みを 1 枚に足す。読むだけで何も書かない
        let (_dir, conn) = setup_test_db();
        let total = |amount| [(1, "総売", Some(100), None, Some(amount))];
        seed_sheet_import(&conn, "2026-03-21", "sheet-a", &total(1000), &[], &[]);
        seed_sheet_import(&conn, "2026-03-21", "sheet-b", &total(500), &[], &[]);
        let before = table_counts(&conn);

        let sheet = get_daily_report_sheet(&conn, "2026-03-21").unwrap();
        assert_eq!(sheet.import_count, 2);
        assert_eq!(
            sheet_view(&sheet.summary_rows),
            [(1, "総売", Some(1500), None)]
        );
        assert_eq!(table_counts(&conn), before);
    }

    #[test]
    fn test_daily_report_sheet_excludes_rolled_back_import() {
        // REQ-501 / SALES-SHEET-D1・D6: 取り消した取込みの値は入らない
        let (_dir, conn) = setup_test_db();
        let total = |amount| [(1, "総売", Some(100), None, Some(amount))];
        seed_sheet_import(&conn, "2026-03-21", "rb-a", &total(1000), &[], &[]);
        let rolled = seed_sheet_import(&conn, "2026-03-21", "rb-b", &total(500), &[], &[]);
        sales_repo::rollback_daily_report_import(&conn, rolled, "2026-03-22T10:00:00").unwrap();

        let sheet = get_daily_report_sheet(&conn, "2026-03-21").unwrap();
        assert_eq!(sheet.import_count, 1);
        assert_eq!(
            sheet_view(&sheet.summary_rows),
            [(1, "総売", Some(1000), None)]
        );
    }

    #[test]
    fn test_daily_report_sheet_unsummed_only_for_shifted_source() {
        // REQ-501 / SALES-SHEET-D2: 帳票ごとに場合を決める（Z002 だけ 3、Z001 は足す）
        let (_dir, conn) = setup_test_db();
        let z001 = [(1, "総売", Some(100), None, Some(100))];
        seed_sheet_import(
            &conn,
            "2026-03-21",
            "shift-a",
            &z001,
            &SHIFTED_Z002_FIRST,
            &[],
        );
        seed_sheet_import(
            &conn,
            "2026-03-21",
            "shift-b",
            &z001,
            &SHIFTED_Z002_SECOND,
            &[],
        );

        let sheet = get_daily_report_sheet(&conn, "2026-03-21").unwrap();
        assert_eq!(sheet.unsummed_sources, [DailyReportSourceKind::Z002]);
        assert_eq!(sheet.label_changed_sources, [DailyReportSourceKind::Z002]);
        assert_eq!(
            sheet_view(&sheet.summary_rows),
            [(1, "総売", Some(200), None)]
        );
        assert_eq!(
            sheet_view(&sheet.payment_rows),
            [
                (1, "項目A", Some(10), Some(1)),
                (2, "項目A", Some(20), Some(1)),
                (1, "項目X", Some(5), Some(2)),
                (2, "項目A", Some(30), Some(2)),
                (3, "項目A", Some(40), Some(2)),
            ]
        );
    }

    #[test]
    fn test_sheet_rows_drop_blank_label_zero_rows_only() {
        // REQ-501 / SALES-SHEET-D3: 名称が空で値が 0 の行は載せず、名称が空で値のある行は載せる
        let (_dir, conn) = setup_test_db();
        seed_sheet_import(
            &conn,
            "2026-03-21",
            "blank",
            &[],
            &[
                (1, "現金", None, Some(1), Some(10)),
                (2, "", None, Some(0), Some(0)),
                (3, "", None, Some(1), Some(5)),
            ],
            &[],
        );

        let sheet = get_daily_report_sheet(&conn, "2026-03-21").unwrap();
        assert_eq!(
            sheet_view(&sheet.payment_rows),
            [(1, "現金", Some(10), None), (3, "", Some(5), None)]
        );
    }

    #[test]
    fn test_daily_report_sheet_plu_rows_are_auto_nonzero_only() {
        // REQ-501 / SALES-SHEET-D4: PLU は auto で数量か金額が 0 でない行だけ（手動販売・0 の行を除く）
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "P001", 1);
        seed_product(&conn, "P002", 1);
        seed_product(&conn, "P003", 1);
        seed_sale(&conn, "P001", "2026-03-21", 2, 1000, "auto");
        seed_sale(&conn, "P002", "2026-03-21", 1, 500, "manual");
        seed_sale(&conn, "P003", "2026-03-21", 1, 300, "auto");
        seed_sale(&conn, "P003", "2026-03-21", -1, -300, "auto");

        let sheet = get_daily_report_sheet(&conn, "2026-03-21").unwrap();
        let codes: Vec<(&str, DailySaleSource)> = sheet
            .plu_rows
            .iter()
            .map(|item| (item.product_code.as_str(), item.source))
            .collect();
        assert_eq!(codes, [("P001", DailySaleSource::Auto)]);
    }

    #[test]
    fn test_daily_report_sheet_plu_import_counts() {
        // REQ-501 / SALES-SHEET-D4・24 §14.21b: Z004 の active な取込みの数と skipped_count の和
        let (_dir, conn) = setup_test_db();
        let sheet = get_daily_report_sheet(&conn, "2026-03-21").unwrap();
        assert_eq!(
            sheet.plu_import,
            DailyReportSheetPluImport {
                import_count: 0,
                skipped_count: 0
            }
        );
        for (settlement_no, status, skipped) in [
            (1, "completed_partial", 2),
            (2, "rolled_back", 7),
            (3, "completed", 0),
        ] {
            sales_repo::insert_csv_import(
                &conn,
                &sales_repo::NewCsvImport {
                    filename: "Z004.CSV".to_string(),
                    settlement_date: "2026-03-21".to_string(),
                    file_hash: format!("z004-{settlement_no}"),
                    total_items: 1,
                    total_amount: 1,
                    skipped_count: skipped,
                    status: status.to_string(),
                    machine_no: "0001".to_string(),
                    settlement_no,
                },
            )
            .unwrap();
        }
        let sheet = get_daily_report_sheet(&conn, "2026-03-21").unwrap();
        assert_eq!(
            sheet.plu_import,
            DailyReportSheetPluImport {
                import_count: 2,
                skipped_count: 2
            }
        );
    }

    #[test]
    fn test_daily_report_sheet_empty_day_returns_zero_imports() {
        // REQ-501 / SALES-SHEET-D5: 取込みの無い日は error にせず import_count 0 と空の行
        let (_dir, conn) = setup_test_db();
        let sheet = get_daily_report_sheet(&conn, "2026-03-21").unwrap();
        assert_eq!(sheet.import_count, 0);
        assert!(sheet.summary_rows.is_empty());
        assert!(sheet.payment_rows.is_empty());
        assert!(sheet.department_rows.is_empty());
        assert!(sheet.label_changed_sources.is_empty());
        assert!(sheet.plu_rows.is_empty());
    }

    #[test]
    fn test_daily_report_sheet_rejects_invalid_date() {
        // REQ-501 / SALES-SHEET-D5: §19.3 と同じ文言
        let (_dir, conn) = setup_test_db();
        assert!(matches!(
            get_daily_report_sheet(&conn, "2026-02-30"),
            Err(BizError::ValidationFailed(message)) if message == "存在しない日付です"
        ));
        assert!(matches!(
            get_daily_report_sheet(&conn, "2026/03/21"),
            Err(BizError::ValidationFailed(message)) if message == "日付の形式が不正です（YYYY-MM-DD）"
        ));
    }

    #[test]
    fn test_daily_sales_payment_unchanged_when_labels_stable() {
        // REQ-501 / 34 §19.3 手順 3: 互換の範囲の日は今と同じ行と値、warning なし
        let (_dir, conn) = setup_test_db();
        let z002 = [
            (1, "現金", None, Some(2), Some(100)),
            (2, "項目B", None, Some(1), Some(30)),
        ];
        seed_sheet_import(&conn, "2026-03-21", "stable-a", &[], &z002, &[]);
        seed_sheet_import(&conn, "2026-03-21", "stable-b", &[], &z002, &[]);

        let official = get_daily_sales(&conn, "2026-03-21")
            .unwrap()
            .official_daily_report
            .unwrap();
        assert_eq!(
            payment_view(&official),
            [
                ("cash", "現金", Some(200), Some(4)),
                ("payment_2", "項目B", Some(60), Some(2)),
            ]
        );
        assert!(official.warnings.is_empty(), "{:?}", official.warnings);
    }

    #[test]
    fn test_daily_sales_payment_representative_in_range_is_first_row() {
        // REQ-501 / 34 §19.3 手順 3: 代表は最小 sort_order・最小の行 id（名称は保存値のまま）
        let (_dir, conn) = setup_test_db();
        seed_sheet_import(
            &conn,
            "2026-03-21",
            "rep-a",
            &[],
            &[(1, "現金 ", None, Some(1), Some(10))],
            &[],
        );
        seed_sheet_import(
            &conn,
            "2026-03-21",
            "rep-b",
            &[],
            &[(1, "現金", None, Some(1), Some(20))],
            &[],
        );

        let official = get_daily_sales(&conn, "2026-03-21")
            .unwrap()
            .official_daily_report
            .unwrap();
        assert_eq!(
            payment_view(&official),
            [("cash", "現金 ", Some(30), Some(2))]
        );
        assert!(official.warnings.is_empty(), "{:?}", official.warnings);
    }

    #[test]
    fn test_daily_sales_payment_splits_duplicate_payment_key_by_position() {
        // REQ-501 / 34 §19.3 手順 3: 1 取込みの中の同じ payment_key の 2 行は位置ごとに分かれる
        let (_dir, conn) = setup_test_db();
        let z002 = [
            (1, "現金", None, Some(1), Some(10)),
            (2, "現金", None, Some(1), Some(20)),
        ];
        seed_sheet_import(&conn, "2026-03-21", "dup-a", &[], &z002, &[]);
        seed_sheet_import(&conn, "2026-03-21", "dup-b", &[], &z002, &[]);

        let official = get_daily_sales(&conn, "2026-03-21")
            .unwrap()
            .official_daily_report
            .unwrap();
        assert_eq!(
            payment_view(&official),
            [
                ("cash", "現金", Some(20), Some(2)),
                ("cash", "現金", Some(40), Some(2))
            ]
        );
    }

    #[test]
    fn test_daily_sales_payment_merges_pair_with_mixed_stored_payment_key() {
        // REQ-501 / 34 §19.3 手順 3: 同じ組で保存済みの payment_key が違っても組で足す（代表は取込み 1）
        let (_dir, conn) = setup_test_db();
        let first = seed_daily_report(&conn, "2026-03-21", "mixed-a");
        let second = seed_daily_report(&conn, "2026-03-21", "mixed-b");
        for (import_id, key, amount) in [(first, "credit", 10), (second, "payment_1", 20)] {
            sales_repo::insert_daily_report_payment_lines(
                &conn,
                &[NewDailyReportPaymentLine {
                    daily_report_import_id: import_id,
                    source_file: "Z002".to_string(),
                    payment_key: key.to_string(),
                    label: "項目C".to_string(),
                    amount: Some(amount),
                    count: Some(1),
                    sort_order: 1,
                }],
            )
            .unwrap();
        }

        let official = get_daily_sales(&conn, "2026-03-21")
            .unwrap()
            .official_daily_report
            .unwrap();
        assert_eq!(
            payment_view(&official),
            [("credit", "項目C", Some(30), Some(2))]
        );
    }

    #[test]
    fn test_daily_sales_payment_unsummed_matches_sheet_rule() {
        // REQ-501 / 34 §19.3 手順 3・SALES-SHEET-D2: 紙と同じ規則で 3、取込みの古い順の 5 行と 3 の文だけ
        let (_dir, conn) = setup_test_db();
        let z001 = [(1, "総売", Some(100), None, Some(100))];
        seed_sheet_import(
            &conn,
            "2026-03-21",
            "shift-a",
            &z001,
            &SHIFTED_Z002_FIRST,
            &[],
        );
        seed_sheet_import(
            &conn,
            "2026-03-21",
            "shift-b",
            &z001,
            &SHIFTED_Z002_SECOND,
            &[],
        );

        let official = get_daily_sales(&conn, "2026-03-21")
            .unwrap()
            .official_daily_report
            .unwrap();
        let amounts: Vec<(&str, Option<i64>)> = official
            .payment_lines
            .iter()
            .map(|line| (line.label.as_str(), line.amount))
            .collect();
        assert_eq!(
            amounts,
            [
                ("項目A", Some(10)),
                ("項目A", Some(20)),
                ("項目X", Some(5)),
                ("項目A", Some(30)),
                ("項目A", Some(40)),
            ]
        );
        assert_eq!(official.warnings, [UNSUMMED_WARNING]);
    }

    #[test]
    fn test_daily_sales_payment_label_changed_warns() {
        // REQ-501 / 34 §19.3 手順 3: 2 の日は 2 行と 2 の文だけ
        let (_dir, conn) = setup_test_db();
        seed_sheet_import(
            &conn,
            "2026-03-21",
            "lc-a",
            &[],
            &[(1, "項目A", None, Some(1), Some(10))],
            &[],
        );
        seed_sheet_import(
            &conn,
            "2026-03-21",
            "lc-b",
            &[],
            &[(1, "項目B", None, Some(1), Some(20))],
            &[],
        );

        let official = get_daily_sales(&conn, "2026-03-21")
            .unwrap()
            .official_daily_report
            .unwrap();
        assert_eq!(
            payment_view(&official),
            [
                ("payment_1", "項目A", Some(10), Some(1)),
                ("payment_1", "項目B", Some(20), Some(1)),
            ]
        );
        assert_eq!(official.warnings, [LABEL_CHANGED_WARNING]);
    }

    #[test]
    fn test_daily_sales_payment_overflow_and_null_order() {
        // REQ-501 / 34 §19.3 手順 3: 足す順は取込みの id の昇順（imported_at の逆でも）、NULL が溢れより先
        let (_dir, conn) = setup_test_db();
        let seed_three = |date: &str, amounts: [Option<i64>; 3]| {
            let ids: Vec<i64> = amounts
                .iter()
                .enumerate()
                .map(|(index, amount)| {
                    seed_sheet_import(
                        &conn,
                        date,
                        &format!("{date}-{index}"),
                        &[],
                        &[(1, "現金", None, Some(1), *amount)],
                        &[],
                    )
                })
                .collect();
            // 取込みの時刻を id の逆にする
            for (index, id) in ids.iter().enumerate() {
                set_imported_at(&conn, *id, &format!("{date}T2{}:00:00", 2 - index));
            }
        };
        seed_three("2026-03-21", [Some(i64::MAX), Some(1), Some(-i64::MAX)]);
        assert!(matches!(
            get_daily_sales(&conn, "2026-03-21"),
            Err(BizError::ValidationFailed(message)) if message == OVERFLOW_MESSAGE
        ));

        seed_three("2026-03-22", [Some(i64::MAX), Some(1), None]);
        let official = get_daily_sales(&conn, "2026-03-22")
            .unwrap()
            .official_daily_report
            .unwrap();
        assert_eq!(payment_view(&official), [("cash", "現金", None, Some(3))]);
    }

    #[test]
    fn test_payment_and_sheet_share_one_d2_function() {
        // REQ-501 / 34 §19.3 手順 3・D-142 の (4): D2 の実装は 1 つ（画面の支払と紙が同じ関数を呼ぶ）
        let service = include_str!("sales_service.rs");
        let production = service.split("#[cfg(test)]").next().unwrap();
        assert_eq!(production.matches("fn apply_sheet_rule(").count(), 1);
        assert!(production.matches("apply_sheet_rule(").count() >= 3);
        let map_fn = &production[production.find("fn map_official_daily_report(").unwrap()..];
        assert!(map_fn[..map_fn.find("\n}\n").unwrap()].contains("apply_sheet_rule("));
        let sheet_fn = &production[production.find("fn get_daily_report_sheet(").unwrap()..];
        assert!(sheet_fn[..sheet_fn.find("\n}\n").unwrap()].contains("apply_sheet_rule("));
        let repo = include_str!("../db/sales_repo.rs");
        assert!(!repo.contains("PaymentGroup"));
    }

    #[test]
    fn test_daily_sales_and_sheet_return_same_large_values() {
        // REQ-501 / 10 (e)・D-142: 上限の中の大きな値は画面も紙も範囲の検査をせずに同じ値を返す
        let (_dir, conn) = setup_test_db();
        for hash in ["large-a", "large-b"] {
            let id = seed_sheet_import(
                &conn,
                "2026-03-21",
                hash,
                &[(1, "総売", Some(100), None, Some(9999999999))],
                &[],
                &[],
            );
            conn.execute(
                "UPDATE daily_report_imports SET gross_amount = 9999999999 WHERE id = ?1",
                rusqlite::params![id],
            )
            .unwrap();
        }
        let official = get_daily_sales(&conn, "2026-03-21")
            .unwrap()
            .official_daily_report
            .unwrap();
        assert_eq!(official.gross_amount, Some(19999999998));
        let sheet = get_daily_report_sheet(&conn, "2026-03-21").unwrap();
        assert_eq!(sheet.summary_rows[0].amount, Some(19999999998));
    }

    // 既存の IO の test（sales_repo の支払の集約）から移した期待（Matrix の「既存の test の移し方」）。
    // fixture は IO の test と同じ。範囲の外の 2 本は **正本の変更で期待が変わる**（34 §19.3 手順 3
    // 「範囲の外の入力は新しい規則の値」）。

    fn insert_payment(
        conn: &DbConnection,
        import_id: i64,
        sort_order: i64,
        label: &str,
        key: &str,
        amount: Option<i64>,
        count: i64,
    ) {
        sales_repo::insert_daily_report_payment_lines(
            conn,
            &[NewDailyReportPaymentLine {
                daily_report_import_id: import_id,
                source_file: "Z002".to_string(),
                payment_key: key.to_string(),
                label: label.to_string(),
                amount,
                count: Some(count),
                sort_order,
            }],
        )
        .unwrap();
    }

    #[test]
    fn test_daily_sales_payment_out_of_range_all_active_parents_req501() {
        // REQ-501 / 34 §19.3 手順 3: 旧 test_completed_daily_report_aggregate_req501_all_active_parents の支払。
        // 並びと値は今と同じ、2 の warning が 1 件増える（正本の変更で期待が変わる）
        let (_dir, conn) = setup_test_db();
        let old_id = seed_daily_report(&conn, "2026-03-21", "latest-old");
        insert_payment(&conn, old_id, 1, "旧現金", "old", Some(1), 1);
        let latest_id = seed_daily_report(&conn, "2026-03-21", "latest-new");
        insert_payment(&conn, latest_id, 2, "クレジット", "credit", Some(2000), 2);
        insert_payment(&conn, latest_id, 1, "現金", "cash", Some(9000), 5);

        let official = get_daily_sales(&conn, "2026-03-21")
            .unwrap()
            .official_daily_report
            .unwrap();
        assert_eq!(
            payment_view(&official),
            [
                ("old", "旧現金", Some(1), Some(1)),
                ("cash", "現金", Some(9000), Some(5)),
                ("credit", "クレジット", Some(2000), Some(2)),
            ]
        );
        assert_eq!(official.warnings, [LABEL_CHANGED_WARNING]);
    }

    #[test]
    fn test_daily_sales_payment_out_of_range_null_fixture_req501() {
        // REQ-501 / 34 §19.3 手順 3: 旧 ..._propagates_parent_and_line_nulls の支払。
        // 旧い期待の 1 行 (現金新, NULL, 3) は成り立たず、D2 の 2 の 2 行と 2 の warning（正本の変更で期待が変わる）
        let (_dir, conn) = setup_test_db();
        let first = seed_daily_report(&conn, "2026-03-21", "null-a");
        let second = seed_daily_report(&conn, "2026-03-21", "null-b");
        insert_payment(&conn, first, 2, "現金旧", "cash", None, 1);
        insert_payment(&conn, second, 1, "現金新", "cash", Some(200), 2);

        let official = get_daily_sales(&conn, "2026-03-21")
            .unwrap()
            .official_daily_report
            .unwrap();
        assert_eq!(
            payment_view(&official),
            [
                ("cash", "現金新", Some(200), Some(2)),
                ("cash", "現金旧", None, Some(1)),
            ]
        );
        assert_eq!(official.warnings, [LABEL_CHANGED_WARNING]);
    }

    #[test]
    fn test_daily_sales_payment_out_of_range_identities_req501() {
        // REQ-501 / 34 §19.3 手順 3: 旧 ..._groups_identities_deterministically の支払。
        // 旧い期待の 1 行 (cash, 現金 新, 300, 3) は成り立たず、2 行と、部門の warning の後に 2 の文（正本の変更で期待が変わる）
        let (_dir, conn) = setup_test_db();
        let first = seed_daily_report(&conn, "2026-03-21", "groups-a");
        let second = seed_daily_report(&conn, "2026-03-21", "groups-b");
        insert_payment(&conn, first, 4, "現金 旧", "cash", Some(100), 1);
        insert_payment(&conn, second, 1, "現金 新", "cash", Some(200), 2);
        sales_repo::insert_daily_report_department_lines(
            &conn,
            &[
                NewDailyReportDepartmentLine {
                    daily_report_import_id: first,
                    source_file: "Z005".into(),
                    department_id: None,
                    raw_department_name: "生地 旧".into(),
                    normalized_department_name: Some("生地".into()),
                    amount: 100,
                    quantity_hundredths: Some(100),
                    count: Some(1),
                    sort_order: 4,
                },
                NewDailyReportDepartmentLine {
                    daily_report_import_id: second,
                    source_file: "Z005".into(),
                    department_id: None,
                    raw_department_name: "毛糸".into(),
                    normalized_department_name: None,
                    amount: 300,
                    quantity_hundredths: Some(300),
                    count: Some(3),
                    sort_order: 2,
                },
            ],
        )
        .unwrap();

        let official = get_daily_sales(&conn, "2026-03-21")
            .unwrap()
            .official_daily_report
            .unwrap();
        assert_eq!(
            payment_view(&official),
            [
                ("cash", "現金 新", Some(200), Some(2)),
                ("cash", "現金 旧", Some(100), Some(1)),
            ]
        );
        assert_eq!(
            official.warnings,
            [
                "部門マスタと対応していない部門が 2 件あります（部門名のまま表示しています）",
                LABEL_CHANGED_WARNING,
            ]
        );
    }
}
