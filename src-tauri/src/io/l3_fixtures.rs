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

/// SD の一括取込みと EJ の取込み（(A-1)。packet の S9 と `## L3`）の入力物。
/// 合成の SD の folder `sd-a`・`sd-b`・`sd-c` と手で選ぶ EJ を L3 の日 T から作る。日報は tracked の
/// `tests/fixtures/daily-report/` の 3 本の日付と精算回数の行を置き換え、EJ は IO-08 の合成の組立てで作る（TD-296）。
/// 読み戻しに BIZ-08 の scan と一括取込みを使うので、層の検査（test だけの cross-layer）のため cfg(test) の module に置く
#[cfg(test)]
mod sd_bulk {
    use crate::biz::daily_report_import_service::{
        commit_register_sd_bulk_import, import_ej_records, scan_register_sd_daily_reports,
        DailyReportSdCandidateStatus, DailyReportSdScanResult, DailyReportSdSelection,
        EjSourceFile, RegisterSdBulkOutcome, RegisterSdEjSummary,
    };
    use crate::db::test_support::setup_test_db;
    use crate::db::DbConnection;
    use crate::io::ej_parser::synthetic::{file as ej_bytes, header, item, totals, NAME_A, NAME_B};
    use crate::io::REGISTER_SD_FOLDER;
    use chrono::{Days, NaiveDate};
    use std::path::{Path, PathBuf};

    /// 日報の束（T からの日数・精算回数・名前の接尾字）
    const SD_A_BUNDLES: [(i64, &str, char); 3] =
        [(-1, "0001", ' '), (0, "0002", ' '), (0, "0003", 'A')];
    const SD_C_BUNDLE: (i64, &str, char) = (1, "0004", ' ');

    fn day_at(day: NaiveDate, offset: i64) -> NaiveDate {
        if offset < 0 {
            day - Days::new(offset.unsigned_abs())
        } else {
            day + Days::new(offset as u64)
        }
    }

    /// tracked の 3 本の対象日と精算回数の行を置き換えた bytes（Z001・Z002・Z005 の順）
    fn bundle(date: NaiveDate, settlement: &str) -> Vec<(&'static str, Vec<u8>)> {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/daily-report");
        ["Z001", "Z002", "Z005"]
            .into_iter()
            .map(|kind| {
                let bytes = std::fs::read(base.join(format!("{kind}_260321.CSV"))).unwrap();
                let (text, _, errors) = encoding_rs::SHIFT_JIS.decode(&bytes);
                assert!(!errors);
                let settlement_line = "\"精算回数    \",\"0001\"";
                assert_eq!(text.matches("2026-03-21").count(), 1, "{kind}");
                assert_eq!(text.matches(settlement_line).count(), 1, "{kind}");
                let text = text
                    .replace("2026-03-21", &date.format("%Y-%m-%d").to_string())
                    .replace(
                        settlement_line,
                        &format!("\"精算回数    \",\"{settlement}\""),
                    );
                (kind, encoding_rs::SHIFT_JIS.encode(&text).0.into_owned())
            })
            .collect()
    }

    /// 合成の取引の記録（番号は 6 桁、時刻は day の 10:00 + 分）
    fn ej_records(day: NaiveDate, numbers: std::ops::RangeInclusive<u32>) -> Vec<String> {
        numbers
            .map(|number| {
                let at = format!("{} 10:{:02}", day.format("%Y-%m-%d"), number % 60);
                [
                    header("", &at, &format!("{number:06}")),
                    vec![item(NAME_A, 120), item(NAME_B, 380)],
                    totals(2, 500),
                ]
                .concat()
            })
            .collect::<Vec<_>>()
            .concat()
    }

    fn ej_name(day: NaiveDate, seq: Option<u16>) -> String {
        match seq {
            Some(seq) => format!("EJ{}_{seq:04}.TXT", day.format("%y%m%d")),
            None => format!("EJ{}.TXT", day.format("%y%m%d")),
        }
    }

    fn put(root: &Path, relative: &str, bytes: &[u8]) {
        let path = relative
            .split('\\')
            .fold(root.to_path_buf(), |path, part| path.join(part));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }

    fn put_bundle(root: &Path, day: NaiveDate, (offset, settlement, suffix): (i64, &str, char)) {
        let date = day_at(day, offset);
        for (kind, bytes) in bundle(date, settlement) {
            let relative = format!(
                "XZ\\{}\\{kind}_{}{suffix}.CSV",
                date.format("%Y\\%m"),
                date.format("%d")
            );
            put(root, &relative, &bytes);
        }
    }

    fn sd_root(base: &Path) -> PathBuf {
        REGISTER_SD_FOLDER
            .iter()
            .fold(base.to_path_buf(), |path, part| path.join(part))
    }

    /// 書く 4 つの出力先（`out` の下）
    const OUTPUTS: [&str; 4] = ["sd-a", "sd-b", "sd-c", "ej-manual"];

    /// `out` の下に sd-a・sd-b・sd-c と ej-manual を書き、手で選ぶ EJ の path を返す。
    /// 出力先が 1 つでも既にあれば何も書かずに止める（違う T の古い file が残り、L3 の数が変わるため）
    fn write_sd_bulk_files(out: &Path, day: NaiveDate) -> PathBuf {
        let existing: Vec<&str> = OUTPUTS
            .into_iter()
            .filter(|name| out.join(name).exists())
            .collect();
        assert!(
            existing.is_empty(),
            "{} に前の入力物 {} があるので作りませんでした。空の folder を INVENTORY_L3_OUT に指定してください",
            out.display(),
            existing.join("・")
        );
        let before = day_at(day, -1);
        let next = day_at(day, 1);
        // sd-a: T−1 と T（精算 2 回）の束、`XZ` の伸びる EJ（最後が保留）、`XZ_BKUP` の EJ（000104〜000105 が欠け）、LF の読めない EJ
        let sd_a = sd_root(&out.join("sd-a"));
        std::fs::create_dir_all(sd_a.join("XZ_BKUP")).unwrap();
        for bundle in SD_A_BUNDLES {
            put_bundle(&sd_a, day, bundle);
        }
        put(
            &sd_a,
            &format!("XZ\\{}", ej_name(day, None)),
            &ej_bytes(&ej_records(day, 106..=108)),
        );
        put(
            &sd_a,
            &format!("XZ_BKUP\\{}", ej_name(before, Some(1))),
            &ej_bytes(&ej_records(before, 101..=103)),
        );
        // CR を全部外して LF だけにする（CP932 の 2 byte 目に CR は無い）
        let lf: Vec<u8> = ej_bytes(&ej_records(before, 104..=105))
            .into_iter()
            .filter(|byte| *byte != b'\r')
            .collect();
        put(
            &sd_a,
            &format!("XZ_BKUP\\{}", ej_name(before, Some(2))),
            &lf,
        );
        // 手で選ぶ EJ: 欠けの 000104〜000105（CRLF。名前は連番つきなので最後まで入る）
        let manual = out.join("ej-manual").join(ej_name(before, Some(2)));
        std::fs::create_dir_all(manual.parent().unwrap()).unwrap();
        std::fs::write(&manual, ej_bytes(&ej_records(before, 104..=105))).unwrap();
        // sd-b: 同じ束と `XZ_BKUP` の EJ だけ
        let sd_b = sd_root(&out.join("sd-b"));
        std::fs::create_dir_all(sd_b.join("XZ_BKUP")).unwrap();
        for bundle in SD_A_BUNDLES {
            put_bundle(&sd_b, day, bundle);
        }
        put(
            &sd_b,
            &format!("XZ_BKUP\\{}", ej_name(before, Some(1))),
            &ej_bytes(&ej_records(before, 101..=103)),
        );
        // sd-c: T+1 の束と、T+1 の記録まで伸びた `XZ` の EJ
        let sd_c = sd_root(&out.join("sd-c"));
        std::fs::create_dir_all(sd_c.join("XZ_BKUP")).unwrap();
        put_bundle(&sd_c, day, SD_C_BUNDLE);
        let grown = [ej_records(day, 106..=108), ej_records(next, 109..=110)].concat();
        put(
            &sd_c,
            &format!("XZ\\{}", ej_name(day, None)),
            &ej_bytes(&grown),
        );
        manual
    }

    fn scan(
        conn: &DbConnection,
        out: &Path,
        folder: &str,
        today: NaiveDate,
    ) -> DailyReportSdScanResult {
        scan_register_sd_daily_reports(
            conn,
            DailyReportSdSelection::Selected(out.join(folder)),
            today,
            None,
        )
        .unwrap()
    }

    fn statuses(result: &DailyReportSdScanResult) -> Vec<DailyReportSdCandidateStatus> {
        result
            .scan
            .candidates
            .iter()
            .map(|candidate| candidate.status)
            .collect()
    }

    /// 候補の中身の日付（並べ替え済み。日付の置換を確かめる）
    fn report_dates(result: &DailyReportSdScanResult) -> Vec<String> {
        let mut dates: Vec<String> = result
            .scan
            .candidates
            .iter()
            .map(|candidate| candidate.report_date.clone().unwrap_or_default())
            .collect();
        dates.sort();
        dates
    }

    fn summary(
        new_count: i64,
        held_count: i64,
        unreadable_file_count: i64,
    ) -> Option<RegisterSdEjSummary> {
        Some(RegisterSdEjSummary {
            new_count,
            held_count,
            conflict_count: 0,
            unreadable_file_count,
        })
    }

    /// `## L3` の表の順（L3-1 → L3-3 → L3-1b → L3-2 の 2 回）に読み戻し、表の数と同じことを確かめる
    fn assert_l3_flow(out: &Path, manual: &Path, day: NaiveDate) {
        use DailyReportSdCandidateStatus::{Imported, NotImported};
        let today = day_at(day, 2);
        let (_dir, mut conn) = setup_test_db();
        let app = out.join("app");
        // L3-1
        let a = scan(&conn, out, "sd-a", today);
        assert_eq!(statuses(&a), [NotImported; 3]);
        let text = |offset| day_at(day, offset).format("%Y-%m-%d").to_string();
        assert_eq!(report_dates(&a), [text(-1), text(0), text(0)]);
        assert_eq!(a.scan.ej, summary(5, 1, 1));
        let bulk = commit_register_sd_bulk_import(&mut conn, &a.snapshot, &app).unwrap();
        assert_eq!(bulk.items.len(), 4);
        assert!(bulk.items[..3]
            .iter()
            .all(|item| matches!(item.outcome, RegisterSdBulkOutcome::ImportedDailyReport(_))));
        let RegisterSdBulkOutcome::ImportedEj(ej) = &bulk.items[3].outcome else {
            panic!("{:?}", bulk.items[3].outcome);
        };
        assert_eq!((ej.inserted_count, ej.held_count), (5, 1));
        assert_eq!(ej.unreadable_files, [ej_name(day_at(day, -1), Some(2))]);
        let gaps: Vec<(&str, &str)> = ej
            .gaps
            .iter()
            .map(|gap| (gap.from_no.as_str(), gap.to_no.as_str()))
            .collect();
        assert_eq!(gaps, [("000104", "000105")]);
        // L3-3
        let manual_file = EjSourceFile {
            filename: manual.file_name().unwrap().to_string_lossy().into_owned(),
            sd_relative_path: None,
            bytes: std::fs::read(manual).unwrap(),
        };
        let filled = import_ej_records(&mut conn, vec![manual_file], &app).unwrap();
        assert_eq!((filled.inserted_count, filled.gaps.len()), (2, 0));
        // L3-1b: 押せない（日報は全部取込み済み、EJ の 4 つの数が 0）
        let b = scan(&conn, out, "sd-b", today);
        assert_eq!(statuses(&b), [Imported; 3]);
        assert_eq!(b.scan.ej, summary(0, 0, 0));
        // L3-2 の 1 回目: T の最後が入り、保留は T+1 の最後の 1 件
        let c = scan(&conn, out, "sd-c", today);
        assert_eq!(statuses(&c), [NotImported]);
        assert_eq!(report_dates(&c), [text(1)]);
        assert_eq!(c.scan.ej, summary(2, 1, 0));
        let bulk = commit_register_sd_bulk_import(&mut conn, &c.snapshot, &app).unwrap();
        assert!(matches!(
            bulk.items[0].outcome,
            RegisterSdBulkOutcome::ImportedDailyReport(_)
        ));
        // L3-2 の 2 回目: 保留だけ
        let again = scan(&conn, out, "sd-c", today);
        assert_eq!(statuses(&again), [Imported]);
        assert_eq!(again.scan.ej, summary(0, 1, 0));
    }

    #[test]
    fn test_l3_fixtures_req401_sd_bulk_inputs_read_back() {
        // REQ-401 / packet S9（Test Plan の L3 の入力物）: 固定の T で作り、scan と一括取込みで読み戻すと `## L3` の数
        let day = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let manual = write_sd_bulk_files(dir.path(), day);
        assert_l3_flow(dir.path(), &manual, day);
    }

    #[test]
    fn test_l3_fixtures_req401_sd_bulk_refuses_existing_outputs() {
        // REQ-401 / packet S9（Final round 1 #9）: 同じ出力先に作り直すと古い T の file が残るので、書く前に止めて理由を出す
        let dir = tempfile::tempdir().unwrap();
        let day = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        write_sd_bulk_files(dir.path(), day);
        let next = day + Days::new(1);
        let error = std::panic::catch_unwind(|| write_sd_bulk_files(dir.path(), next)).unwrap_err();
        let message = error.downcast_ref::<String>().unwrap();
        assert!(
            message.contains("前の入力物 sd-a・sd-b・sd-c・ej-manual があるので作りませんでした"),
            "{message}"
        );
        // 何も書いていない（次の T の `XZ` の EJ が無い）
        let sd_a = sd_root(&dir.path().join("sd-a"));
        assert!(!sd_a.join("XZ").join(ej_name(next, None)).exists());
        assert!(sd_a.join("XZ").join(ej_name(day, None)).exists());
    }

    #[test]
    #[ignore = "INVENTORY_L3_DAY（YYYY-MM-DD）と INVENTORY_L3_OUT（出力の folder）を与えて手元で実行する"]
    fn write_sd_bulk_l3_fixtures() {
        // (A-1) の L3 の前に Coordinator が走らせる。T+1 の束を未来の日付にしないため、T は L3 の日の前日より前にする
        let day = std::env::var("INVENTORY_L3_DAY").expect("INVENTORY_L3_DAY が未設定です");
        let day = NaiveDate::parse_from_str(&day, "%Y-%m-%d")
            .ok()
            .filter(|parsed| parsed.format("%Y-%m-%d").to_string() == day)
            .expect("INVENTORY_L3_DAY は YYYY-MM-DD で与えてください");
        let out = std::env::var_os("INVENTORY_L3_OUT").expect("INVENTORY_L3_OUT が未設定です");
        let out = Path::new(&out);
        let manual = write_sd_bulk_files(out, day);
        // 読み戻しは別の一時 folder の写しで行い、出力の folder に app の写しを残さない
        let check = tempfile::tempdir().unwrap();
        let check_manual = write_sd_bulk_files(check.path(), day);
        assert_l3_flow(check.path(), &check_manual, day);
        println!(
            "(A-1) の L3 の入力物を {} に書きました（T = {day}。sd-a・sd-b・sd-c と、手で選ぶ EJ {}）",
            out.display(),
            manual.display()
        );
    }
}

/// PLU の書出しの見直しの runtime の L3 の入力物（packet `2026-11-24-plu-redesign-runtime` の S10、`## L3`）。
/// 全枠の Z004（5,000 行、CP932・CRLF）だけを持つ合成の SD の folder を 2 つ作る:
/// `sd-a` は読取りの窓の日数ぶんの bytes の違う Z004、`sd-b` はどの Z004 も精算回数が `sd-a` の最大より小さい。
/// 読み戻しに BIZ-08 の scan と BIZ-04 の照合を使うので cfg(test) の module に置く（`sd_bulk` と同じ理由）
#[cfg(test)]
mod plu_snapshot {
    use super::{cp932, jan, settlement_date};
    use crate::biz::daily_report_import_service::{
        scan_register_sd_daily_reports, DailyReportSdSelection,
    };
    use crate::biz::plu_export_service::{
        refresh_plu_register_snapshot_from_sd, PluRegisterSnapshotRefresh,
    };
    use crate::db::test_support::setup_test_db;
    use crate::db::DbConnection;
    use crate::io::REGISTER_SD_FOLDER;
    use chrono::NaiveDate;
    use std::path::Path;

    /// 読取りの窓の日数（37 §37.9 手順 2 の 30 日）
    const WINDOW_DAYS: i64 = 30;
    /// `sd-a` の精算回数は 101〜130、`sd-b` は 1〜3（どれも `sd-a` の最大より小さい）
    const SD_A_FIRST_SETTLEMENT: i64 = 101;
    const SD_B_DAYS: i64 = 3;
    const OUTPUTS: [&str; 2] = ["sd-a", "sd-b"];

    /// 全枠の Z004。217〜219 番だけがレジの既存の登録（合成の JAN）で、ほかは空
    fn full_slot_z004(settlement_no: i64, date: NaiveDate) -> Vec<u8> {
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
        for memory_no in 1..=5_000_usize {
            let (code, name) = match memory_no {
                217..=219 => (format!("{}E", jan(memory_no - 216)), "合成の既存登録"),
                _ => ("00000000000000".to_string(), ""),
            };
            lines.push(format!(
                "\"{memory_no:04}\",\"{code}\",\"{name}\",\"0\",\"0\""
            ));
        }
        cp932(&lines)
    }

    /// `out/{folder}` の SD の `XZ` に、`day` から `days` 日ぶん遡った各日の Z004 を置く（精算回数は古い日から `first`〜）
    fn write_sd(out: &Path, folder: &str, day: NaiveDate, days: i64, first: i64) {
        let root = REGISTER_SD_FOLDER
            .iter()
            .fold(out.join(folder), |path, part| path.join(part));
        for index in 0..days {
            let date = settlement_date(day, index + 1 - days);
            let dir = root
                .join("XZ")
                .join(date.format("%Y").to_string())
                .join(date.format("%m").to_string());
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                dir.join(format!("Z004_{} .CSV", date.format("%d"))),
                full_slot_z004(first + index, date),
            )
            .unwrap();
        }
    }

    /// 出力先が既にあれば何も書かずに止める（違う T の古い file が残るため）
    fn write_plu_snapshot_files(out: &Path, day: NaiveDate) {
        let existing: Vec<&str> = OUTPUTS
            .into_iter()
            .filter(|name| out.join(name).exists())
            .collect();
        assert!(
            existing.is_empty(),
            "{} に前の入力物 {} があるので作りませんでした。空の folder を INVENTORY_L3_OUT に指定してください",
            out.display(),
            existing.join("・")
        );
        write_sd(out, "sd-a", day, WINDOW_DAYS, SD_A_FIRST_SETTLEMENT);
        write_sd(out, "sd-b", day, SD_B_DAYS, 1);
    }

    fn refresh(
        conn: &mut DbConnection,
        out: &Path,
        folder: &str,
        today: NaiveDate,
    ) -> (usize, PluRegisterSnapshotRefresh) {
        let scanned = scan_register_sd_daily_reports(
            conn,
            DailyReportSdSelection::Selected(out.join(folder)),
            today,
            None,
        )
        .unwrap();
        let z004s = scanned.snapshot.plu_register_z004s;
        let refresh = refresh_plu_register_snapshot_from_sd(conn, &z004s).unwrap();
        (z004s.len(), refresh)
    }

    /// `## L3` の L3-1 → L3-2 の順に読み戻す
    fn assert_l3_flow(out: &Path, day: NaiveDate) {
        let (_dir, mut conn) = setup_test_db();
        let (count, first) = refresh(&mut conn, out, "sd-a", day);
        assert_eq!(
            count as i64, WINDOW_DAYS,
            "bytes の違う Z004 が窓の日数ぶん"
        );
        assert!(matches!(first, PluRegisterSnapshotRefresh::Refreshed(_)));
        let setting = |key: &str| crate::db::system_repo::get_setting(&conn, key).unwrap();
        assert_eq!(
            setting("plu_register_snapshot_settlement_no"),
            Some((SD_A_FIRST_SETTLEMENT + WINDOW_DAYS - 1).to_string())
        );
        let external: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM plu_slots WHERE status = 'external'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(external, 3);
        let (count, second) = refresh(&mut conn, out, "sd-b", day);
        assert_eq!(count as i64, SD_B_DAYS);
        assert!(matches!(second, PluRegisterSnapshotRefresh::SkippedOlder));
    }

    #[test]
    fn test_l3_fixtures_req907_plu_snapshot_inputs_read_back() {
        // REQ-907 / packet S10: 固定の T で作り、scan と照合で読み戻すと sd-a は照合し、sd-b は古い SD として照合しない
        let day = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let dir = tempfile::tempdir().unwrap();
        write_plu_snapshot_files(dir.path(), day);
        assert_l3_flow(dir.path(), day);
    }

    #[test]
    #[ignore = "INVENTORY_L3_DAY（YYYY-MM-DD）と INVENTORY_L3_OUT（出力の folder）を与えて手元で実行する"]
    fn write_plu_snapshot_l3_fixtures() {
        // L3 の前に Coordinator が走らせる。T は L3 の日（Z004 の日付を未来にしない）
        let day = std::env::var("INVENTORY_L3_DAY").expect("INVENTORY_L3_DAY が未設定です");
        let day = NaiveDate::parse_from_str(&day, "%Y-%m-%d")
            .ok()
            .filter(|parsed| parsed.format("%Y-%m-%d").to_string() == day)
            .expect("INVENTORY_L3_DAY は YYYY-MM-DD で与えてください");
        let out = std::env::var_os("INVENTORY_L3_OUT").expect("INVENTORY_L3_OUT が未設定です");
        let out = Path::new(&out);
        write_plu_snapshot_files(out, day);
        assert_l3_flow(out, day);
        println!(
            "PLU の runtime の L3 の入力物を {} に書きました（T = {day}。sd-a・sd-b）",
            out.display()
        );
    }
}
