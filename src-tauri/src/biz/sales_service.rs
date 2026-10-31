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
use crate::io::daily_report_parser::quantity_hundredths_to_units;
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
    let db_rows = sales_repo::get_daily_sales_records(conn, date)?;
    let items: Vec<DailySaleItem> = db_rows
        .into_iter()
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
        .collect::<Result<Vec<_>, BizError>>()?;

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
        .map(map_official_daily_report);

    Ok(DailySalesReport {
        date: date.to_string(),
        items,
        department_subtotals,
        grand_total,
        official_daily_report,
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

    Ok(MonthlySalesReport {
        month: month.to_string(),
        mode,
        items,
        prev_month_comparison,
        official_department_totals,
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

fn map_official_daily_report(
    row: sales_repo::OfficialDailyReportRow,
) -> OfficialDailyReportSummary {
    let unmatched_count = row
        .department_lines
        .iter()
        .filter(|line| line.department_id.is_none())
        .count();
    let warnings = if unmatched_count == 0 {
        Vec::new()
    } else {
        vec![format!(
            "部門マスタと対応していない部門が {} 件あります（部門名のまま表示しています）",
            unmatched_count
        )]
    };

    OfficialDailyReportSummary {
        source_import_count: row.source_import_count,
        report_date: row.report_date,
        gross_amount: row.gross_amount,
        net_amount: row.net_amount,
        payment_lines: row
            .payment_lines
            .into_iter()
            .map(|line| OfficialDailyPaymentLine {
                payment_key: line.payment_key,
                label: line.label,
                amount: line.amount,
                count: line.count,
            })
            .collect(),
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
    }
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
}
