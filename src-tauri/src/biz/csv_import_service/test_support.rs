//! BIZ-03 テスト共有ヘルパー

use crate::biz::csv_import_service::parse::parse_and_validate;
use crate::biz::csv_import_service::{
    CachedPreview, CsvParseAndValidateRequest, ParseValidateResult,
};
use crate::db::DbConnection;
use encoding_rs::SHIFT_JIS;
use std::time::Instant;

/// JAN付き商品を作成するテストヘルパー
///
/// department_id=1（初期データ「その他小物」）を使用。
pub(crate) fn create_test_product_with_jan(
    conn: &DbConnection,
    product_code: &str,
    jan_code: &str,
    stock: i64,
    pos_stock_sync: bool,
) {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO products (product_code, jan_code, name, department_id, selling_price, cost_price_centi, tax_rate, stock_quantity, stock_unit, is_discontinued, plu_dirty, pos_stock_sync, created_at, updated_at)
         VALUES (?1, ?2, ?3, 1, 100, 50, '10', ?4, 'pcs', 0, 1, ?5, ?6, ?7)",
        rusqlite::params![product_code, jan_code, format!("テスト商品 {}", product_code), stock, pos_stock_sync, now, now],
    )
    .expect("テスト商品の作成に失敗");
    // 写しは記録から求め直すので、同じ値の初期在庫の movement も入れる（Matrix T3）
    if stock != 0 {
        crate::db::test_support::insert_initial_stock_movement(conn, product_code, stock, stock);
    }
}

/// 従来の形（精算回数の無い file）の Z004 バイト列（32 の順 1 の拒否の入力）
///
/// data_lines: (jan, name, quantity, amount) のスライス
pub(crate) fn make_conventional_z004_bytes(
    settlement_date: &str,
    data_lines: &[(&str, &str, i32, i32)],
) -> Vec<u8> {
    let mut lines = Vec::new();
    // 1行目: メタ行（日付含む）
    lines.push(format!("\"精算日\",\"{}\",\"\",\"\",\"\"", settlement_date));
    // 2行目: ヘッダ（読み飛ばし）
    lines.push("\"No.\",\"スキャニングコード\",\"商品名\",\"個数\",\"金額\"".to_string());
    // 3行目以降: データ行
    for (i, (jan, name, qty, amt)) in data_lines.iter().enumerate() {
        lines.push(format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
            i + 1,
            jan,
            name,
            qty,
            amt
        ));
    }

    let text = lines.join("\r\n");
    let (encoded, _, _) = SHIFT_JIS.encode(&text);
    encoded.into_owned()
}

/// layout A（マシンNo.・精算回数・日付のメタつき）の Z004 バイト列（レジ `machine_no`）
pub(crate) fn make_z004_bytes_for_machine(
    machine_no: &str,
    settlement_no: i64,
    settlement_date: &str,
    data_lines: &[(&str, &str, i32, i32)],
) -> Vec<u8> {
    make_z004_bytes_raw(
        machine_no,
        &format!("{settlement_no:04}"),
        settlement_date,
        data_lines,
    )
}

/// layout A の Z004 バイト列（精算回数の欄を文字列のまま書く。32 の順 1 の拒否の入力）
pub(crate) fn make_z004_bytes_raw(
    machine_no: &str,
    settlement_raw: &str,
    settlement_date: &str,
    data_lines: &[(&str, &str, i32, i32)],
) -> Vec<u8> {
    let rows: Vec<String> = data_lines
        .iter()
        .enumerate()
        .map(|(i, (jan, name, qty, amt))| {
            format!("\"{}\",\"{jan}\",\"{name}\",\"{qty}\",\"{amt}\"", i + 1)
        })
        .collect();
    layout_a_bytes(machine_no, settlement_raw, settlement_date, &rows)
}

/// `layout_a_bytes` の最初のデータ行の行番号（メタ 5 行・空行・ヘッダの後）
pub(crate) const LAYOUT_A_FIRST_DATA_LINE: usize = 8;

/// layout A の Z004 バイト列（データ行は書いた文字列のまま。カンマ付きの値・壊れた行の入力）
pub(crate) fn layout_a_bytes(
    machine_no: &str,
    settlement_raw: &str,
    settlement_date: &str,
    data_lines: &[String],
) -> Vec<u8> {
    let mut lines = vec![
        format!("\"マシンNo.   \",\"{machine_no}\""),
        "\"ファイル    \",\"Z004\"".to_string(),
        format!("\"精算回数    \",\"{settlement_raw}\""),
        format!("\"日付        \",\"{settlement_date}\""),
        "\"時刻        \",\"20:00\"".to_string(),
        String::new(),
        "\"レコード    \",\"ｽｷｬﾆﾝｸﾞｺｰﾄﾞ \",\"キャラクター\",\"個数        \",\"金額        \""
            .to_string(),
    ];
    lines.extend(data_lines.iter().cloned());
    let text = lines.join("\r\n");
    let (encoded, _, _) = SHIFT_JIS.encode(&text);
    encoded.into_owned()
}

/// テスト用Z004バイト列（layout A、マシンNo. `0001`、精算回数 `settlement_no`）
///
/// data_lines: (jan, name, quantity, amount) のスライス
pub(crate) fn make_z004_bytes(
    settlement_no: i64,
    settlement_date: &str,
    data_lines: &[(&str, &str, i32, i32)],
) -> Vec<u8> {
    make_z004_bytes_for_machine("0001", settlement_no, settlement_date, data_lines)
}

/// test の既定の今日（精算日が今日より後の知らせが出ない日）
pub(crate) const TEST_TODAY: &str = "2099-12-31";

/// parse_and_validate を呼んで結果を返す（commit/rollback テストの前段処理用）
///
/// パースが成功することを前提とする。失敗時はpanic。
pub(crate) fn parse_and_build_cache(
    conn: &DbConnection,
    file_bytes: Vec<u8>,
    filename: &str,
) -> ParseValidateResult {
    parse_and_validate(
        conn,
        CsvParseAndValidateRequest {
            file_bytes,
            filename: filename.to_string(),
            today: TEST_TODAY.to_string(),
        },
    )
    .expect("parse_and_validate がテスト中に失敗しました")
}

/// CachedPreview を構築するヘルパー
pub(crate) fn build_cached(result: ParseValidateResult) -> CachedPreview {
    CachedPreview {
        created_at: Instant::now(),
        matched_rows: result.matched_rows,
        error_rows: result.error_rows,
        preview_data: result.preview_data,
        settlement: result.settlement,
    }
}

/// preview から確定までを通す（既定の今日）
pub(crate) fn commit_bytes(
    conn: &mut DbConnection,
    file_bytes: Vec<u8>,
    filename: &str,
) -> Result<crate::biz::csv_import_service::ImportResult, crate::biz::BizError> {
    let cached = build_cached(parse_and_build_cache(conn, file_bytes, filename));
    crate::biz::csv_import_service::commit_csv_import(
        conn,
        crate::biz::csv_import_service::CommitRequest {
            cached_data: cached,
            today: TEST_TODAY.to_string(),
        },
    )
}

/// 写しの在庫・要再確認の印・範囲（(low, high)）
pub(crate) fn stock_copy(
    conn: &DbConnection,
    product_code: &str,
) -> (i64, bool, Option<(i64, i64)>) {
    conn.query_row(
        "SELECT stock_quantity, stock_recheck, stock_range_low, stock_range_high
         FROM products WHERE product_code = ?1",
        [product_code],
        |row| {
            let low: Option<i64> = row.get(2)?;
            let high: Option<i64> = row.get(3)?;
            Ok((row.get(0)?, row.get(1)?, low.zip(high)))
        },
    )
    .unwrap()
}
