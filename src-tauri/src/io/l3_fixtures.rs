//! b1 の Windows L3 の入力物の生成（packet の Test Plan の「L3 の fixture の表」。合成の値だけ。TD-296）
//!
//! 精算日を L3 の日 T からの相対で決めるので file は tracked にせず、ここで T から作る。
//! 商品一括 import の CSV 2 つと layout A の Z004 7 つ（CP932・CRLF、マシンNo. `0001`）。
//! 通常の test は固定の T で作って読み戻し（CI で生成の誤りを止める）、手元で走らせる ignore の test は
//! env の `INVENTORY_L3_DAY`（`YYYY-MM-DD`）と `INVENTORY_L3_OUT`（出力の folder）で手元で作る。

use crate::io::{product_csv_importer, z004_parser};
use chrono::{Days, NaiveDate};
use std::path::Path;

/// L3 の商品（表の P1〜P6）。部門は作り直した空の DB にある初期の部門 `1`
struct L3Product {
    code: &'static str,
    name: &'static str,
    unit: &'static str,
    linked: bool,
    initial_stock: &'static str,
    selling_price: i64,
    cost: &'static str,
}

const PRODUCTS: [L3Product; 6] = [
    L3Product {
        code: "L3-P1",
        name: "合成P1",
        unit: "pcs",
        linked: true,
        initial_stock: "10",
        selling_price: 1000,
        cost: "500.00",
    },
    L3Product {
        code: "L3-P2",
        name: "合成P2",
        unit: "pcs",
        linked: true,
        initial_stock: "10",
        selling_price: 1000,
        cost: "500.00",
    },
    L3Product {
        code: "L3-P3",
        name: "合成P3",
        unit: "pcs",
        linked: true,
        initial_stock: "10",
        selling_price: 1000,
        cost: "500.00",
    },
    L3Product {
        code: "L3-P4",
        name: "合成P4",
        unit: "m",
        linked: true,
        initial_stock: "5",
        selling_price: 300,
        cost: "150.00",
    },
    L3Product {
        code: "L3-P5",
        name: "合成P5",
        unit: "pcs",
        linked: false,
        initial_stock: "10",
        selling_price: 1000,
        cost: "500.00",
    },
    L3Product {
        code: "L3-P6",
        name: "合成P6",
        unit: "pcs",
        linked: true,
        initial_stock: "10",
        selling_price: 1000,
        cost: "500.00",
    },
];

/// Z004 の file（名前・精算回数・T からの日数・行〈商品の番号 1〜6、数量、金額〉）
type Settlement = (&'static str, i64, i64, &'static [(usize, i32, i32)]);

const SETTLEMENTS: [Settlement; 7] = [
    ("z004-0101.csv", 101, -3, &[(3, 1, 1000), (5, 1, 1000)]),
    ("z004-0102.csv", 102, -2, &[(1, 1, 1000)]),
    ("z004-0102-other.csv", 102, -2, &[(1, 2, 2000)]),
    ("z004-0099.csv", 99, 0, &[(2, 1, 1000)]),
    // 正常な行が全て 0（売上も movement も作らない精算）
    ("z004-0104.csv", 104, 0, &[(1, 0, 0)]),
    ("z004-0200.csv", 200, 365, &[(2, 1, 1000)]),
    ("z004-0103.csv", 103, -1, &[(3, 1, 1000)]),
];

const PRODUCT_HEADER: &str =
    "商品コード,商品名,部門ID,売価,原価,税率,JANコード,在庫単位,初期在庫,POS在庫連動";

/// 合成の 13 桁の JAN（`29` で始まる店内の範囲、check digit が合う）
fn jan(index: usize) -> String {
    let body = format!("29000000000{index}");
    let sum: u32 = body
        .bytes()
        .enumerate()
        .map(|(i, b)| u32::from(b - b'0') * if i % 2 == 0 { 1 } else { 3 })
        .sum();
    format!("{body}{}", (10 - sum % 10) % 10)
}

fn cp932(lines: &[String]) -> Vec<u8> {
    let text = lines.join("\r\n") + "\r\n";
    let (encoded, _, had_errors) = encoding_rs::SHIFT_JIS.encode(&text);
    assert!(!had_errors, "CP932 で書けない文字があります");
    encoded.into_owned()
}

fn product_line(product: &L3Product, jan_code: &str) -> String {
    format!(
        "{},{},1,{},{},10,{},{},{},{}",
        product.code,
        product.name,
        product.selling_price,
        product.cost,
        jan_code,
        product.unit,
        product.initial_stock,
        u8::from(product.linked)
    )
}

fn settlement_date(day: NaiveDate, offset: i64) -> NaiveDate {
    if offset >= 0 {
        day.checked_add_days(Days::new(offset.unsigned_abs()))
    } else {
        day.checked_sub_days(Days::new(offset.unsigned_abs()))
    }
    .expect("精算日が日付の範囲の外です")
}

fn z004(settlement_no: i64, date: NaiveDate, rows: &[(usize, i32, i32)]) -> Vec<u8> {
    let mut lines = vec![
        "\"マシンNo.   \",\"0001\"".to_string(),
        "\"ファイル    \",\"Z004\"".to_string(),
        format!("\"精算回数    \",\"{settlement_no:04}\""),
        format!("\"日付        \",\"{}\"", date.format("%Y-%m-%d")),
        "\"時刻        \",\"20:00\"".to_string(),
        String::new(),
        "\"レコード    \",\"ｽｷｬﾆﾝｸﾞｺｰﾄﾞ \",\"キャラクター\",\"個数        \",\"金額        \""
            .to_string(),
    ];
    for (line, (index, quantity, amount)) in rows.iter().enumerate() {
        lines.push(format!(
            "\"{}\",\"{}\",\"{}\",\"{quantity}\",\"{amount}\"",
            line + 1,
            jan(*index),
            PRODUCTS[index - 1].name
        ));
    }
    cp932(&lines)
}

/// T から 9 file（名前と中身）を作る
fn l3_files(day: NaiveDate) -> Vec<(&'static str, Vec<u8>)> {
    let mut new_lines = vec![PRODUCT_HEADER.to_string()];
    new_lines.extend(
        PRODUCTS
            .iter()
            .enumerate()
            .map(|(i, product)| product_line(product, &jan(i + 1))),
    );
    // P1 の上書きの行（JAN を P2 の JAN に。L3-12 の共有 JAN の拒否）
    let conflict_lines = vec![
        PRODUCT_HEADER.to_string(),
        product_line(&PRODUCTS[0], &jan(2)),
    ];
    let mut files = vec![
        ("products-new.csv", cp932(&new_lines)),
        ("products-jan-conflict.csv", cp932(&conflict_lines)),
    ];
    files.extend(
        SETTLEMENTS.iter().map(|(name, no, offset, rows)| {
            (*name, z004(*no, settlement_date(day, *offset), rows))
        }),
    );
    files
}

fn write_l3_files(dir: &Path, day: NaiveDate) {
    std::fs::create_dir_all(dir).expect("出力の folder を作れません");
    for (name, bytes) in l3_files(day) {
        std::fs::write(dir.join(name), bytes).expect("file を書けません");
    }
}

/// 書いた file を parser で読み戻し、表の精算回数・精算日・行・商品と同じことを確かめる
fn assert_parse_back(dir: &Path, day: NaiveDate) {
    let read = |name: &str| std::fs::read(dir.join(name)).expect("file を読めません");
    for (name, no, offset, rows) in SETTLEMENTS {
        let result = z004_parser::parse_z004(&read(name)).expect(name);
        let metadata = result.settlement_metadata.as_ref().expect(name);
        assert_eq!(metadata.machine_no.as_deref(), Some("0001"), "{name}");
        assert_eq!(
            metadata.settlement_no.as_deref(),
            Some(format!("{no:04}").as_str()),
            "{name}"
        );
        assert_eq!(
            result.settlement_date,
            settlement_date(day, offset).format("%Y-%m-%d").to_string(),
            "{name}"
        );
        assert!(result.parse_errors.is_empty(), "{name}");
        let parsed: Vec<(String, i32, i32)> = result
            .parsed_rows
            .iter()
            .map(|row| (row.normalized_jan.clone(), row.quantity, row.amount))
            .collect();
        let expected: Vec<(String, i32, i32)> = rows
            .iter()
            .map(|(index, quantity, amount)| (jan(*index), *quantity, *amount))
            .collect();
        assert_eq!(parsed, expected, "{name}");
    }

    let field = |row: &product_csv_importer::ParsedRow, key: &str| {
        row.fields.get(key).cloned().unwrap_or_default()
    };
    let new = product_csv_importer::parse_product_csv(&read("products-new.csv")).unwrap();
    assert!(new.parse_errors.is_empty());
    assert_eq!(new.rows.len(), PRODUCTS.len());
    for (i, (row, product)) in new.rows.iter().zip(PRODUCTS.iter()).enumerate() {
        assert_eq!(field(row, "商品コード"), product.code);
        assert_eq!(field(row, "部門ID"), "1");
        assert_eq!(field(row, "JANコード"), jan(i + 1));
        assert_eq!(field(row, "在庫単位"), product.unit);
        assert_eq!(field(row, "初期在庫"), product.initial_stock);
        assert_eq!(field(row, "売価"), product.selling_price.to_string());
        assert_eq!(field(row, "原価"), product.cost);
        assert_eq!(
            field(row, "POS在庫連動"),
            u8::from(product.linked).to_string()
        );
    }
    let conflict =
        product_csv_importer::parse_product_csv(&read("products-jan-conflict.csv")).unwrap();
    assert_eq!(conflict.rows.len(), 1);
    assert_eq!(field(&conflict.rows[0], "商品コード"), "L3-P1");
    assert_eq!(field(&conflict.rows[0], "JANコード"), jan(2));
}

#[test]
fn test_l3_fixtures_req401_req104_generated_files_parse_back() {
    // REQ-401 / REQ-104 / b1 Matrix F1（AC12）: 固定の T で 9 file を作り、parser で読み戻すと表と同じ
    let day = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let dir = tempfile::tempdir().unwrap();
    write_l3_files(dir.path(), day);
    let mut names: Vec<String> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names.len(), 9);
    assert_parse_back(dir.path(), day);
    // JAN の check digit が合う（13 桁全体の重み 1・3 の和が 10 の倍数。生成の式と独立に確かめる）
    for index in 1..=PRODUCTS.len() {
        let code = jan(index);
        let sum: u32 = code
            .bytes()
            .rev()
            .enumerate()
            .map(|(i, b)| u32::from(b - b'0') * if i % 2 == 0 { 1 } else { 3 })
            .sum();
        assert_eq!((code.len(), sum % 10), (13, 0), "{code}");
    }
}

#[test]
#[ignore = "INVENTORY_L3_DAY（YYYY-MM-DD）と INVENTORY_L3_OUT（出力の folder）を与えて手元で実行する"]
fn write_b1_l3_fixtures() {
    // b1 の L3 の日に Coordinator が走らせる（packet の S9 と Test Plan の L3）
    let day = std::env::var("INVENTORY_L3_DAY").expect("INVENTORY_L3_DAY が未設定です");
    let day = NaiveDate::parse_from_str(&day, "%Y-%m-%d")
        .ok()
        .filter(|parsed| parsed.format("%Y-%m-%d").to_string() == day)
        .expect("INVENTORY_L3_DAY は YYYY-MM-DD で与えてください");
    let out = std::env::var_os("INVENTORY_L3_OUT").expect("INVENTORY_L3_OUT が未設定です");
    let dir = Path::new(&out);
    write_l3_files(dir, day);
    assert_parse_back(dir, day);
    println!(
        "b1 の L3 の file を {} に書きました（T = {day}）",
        dir.display()
    );
}
