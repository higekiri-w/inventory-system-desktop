use crate::biz::csv_import_service::test_support::*;
use crate::biz::csv_import_service::*;
use crate::biz::BizError;
use crate::db::sales_repo::{self, NewCsvImport};
use crate::db::test_support::setup_test_db;

// --- parse_and_validate テスト ---

#[test]
fn test_parse_and_validate_req401_size_limit() {
    // REQ-401: CSV取込み
    let (_dir, conn) = setup_test_db();
    let big_bytes = vec![0u8; 20 * 1024 * 1024 + 1];
    let result = parse::parse_and_validate(
        &conn,
        CsvParseAndValidateRequest {
            file_bytes: big_bytes,
            filename: "too_big.csv".to_string(),
            today: TEST_TODAY.to_string(),
        },
    );
    assert!(result.is_err());
    match result.unwrap_err() {
        BizError::ImportError(msg) => assert!(msg.contains("20MB")),
        e => panic!("Expected ImportError, got {:?}", e),
    }
}

#[test]
fn test_parse_and_validate_req401_decode_error() {
    // REQ-401: CSV取込み
    let (_dir, conn) = setup_test_db();
    // 不正なバイト列（CP932としてデコード不能）
    let invalid_bytes = vec![0xFF, 0xFE, 0x00, 0x01, 0x80, 0x00];
    let result = parse::parse_and_validate(
        &conn,
        CsvParseAndValidateRequest {
            file_bytes: invalid_bytes,
            filename: "invalid.csv".to_string(),
            today: TEST_TODAY.to_string(),
        },
    );
    assert!(result.is_err());
    match result.unwrap_err() {
        BizError::ImportError(msg) => assert!(msg.contains("CP932")),
        e => panic!("Expected ImportError, got {:?}", e),
    }
}

#[test]
fn test_parse_and_validate_req401_no_data_lines() {
    // REQ-401: CSV取込み
    let (_dir, conn) = setup_test_db();
    // 1行のみ（ヘッダ行すらない）
    let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode("\"精算日\"");
    let result = parse::parse_and_validate(
        &conn,
        CsvParseAndValidateRequest {
            file_bytes: encoded.into_owned(),
            filename: "short.csv".to_string(),
            today: TEST_TODAY.to_string(),
        },
    );
    assert!(result.is_err());
    match result.unwrap_err() {
        BizError::ImportError(msg) => assert!(msg.contains("データ行がありません")),
        e => panic!("Expected ImportError, got {:?}", e),
    }
}

#[test]
fn test_parse_and_validate_req401_no_settlement_date() {
    // REQ-401: CSV取込み
    let (_dir, conn) = setup_test_db();
    // 日付なしの2行
    let text = "\"no date here\",\"\",\"\",\"\",\"\"\r\n\"header\",\"\",\"\",\"\",\"\"";
    let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode(text);
    let result = parse::parse_and_validate(
        &conn,
        CsvParseAndValidateRequest {
            file_bytes: encoded.into_owned(),
            filename: "nodate.csv".to_string(),
            today: TEST_TODAY.to_string(),
        },
    );
    assert!(result.is_err());
    match result.unwrap_err() {
        BizError::ImportError(msg) => assert!(msg.contains("精算日")),
        e => panic!("Expected ImportError, got {:?}", e),
    }
}

#[test]
fn test_parse_and_validate_req401_line_limit() {
    // REQ-401: CSV取込み
    let (_dir, conn) = setup_test_db();
    // 10001行のデータ行を生成
    let mut data_lines = Vec::new();
    for _ in 0..10001 {
        data_lines.push(("4912345678901", "テスト", 1, 100));
    }
    let refs = data_lines.to_vec();
    let bytes = make_z004_bytes(1, "2026-03-21", &refs);
    let result = parse::parse_and_validate(
        &conn,
        CsvParseAndValidateRequest {
            file_bytes: bytes,
            filename: "huge.csv".to_string(),
            today: TEST_TODAY.to_string(),
        },
    );
    assert!(result.is_err());
    match result.unwrap_err() {
        BizError::ImportError(msg) => assert!(msg.contains("10,000")),
        e => panic!("Expected ImportError, got {:?}", e),
    }
}

#[test]
fn test_parse_and_validate_req401_empty_records_excluded() {
    // REQ-401: CSV取込み
    let (_dir, conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    // quantity=0, amount=0 の行 + 正常行
    let bytes = make_z004_bytes(
        1,
        "2026-03-21",
        &[
            ("4912345678901", "正常商品", 3, 900),
            ("4912345678902", "空スロット", 0, 0),
        ],
    );
    let result = parse::parse_and_validate(
        &conn,
        CsvParseAndValidateRequest {
            file_bytes: bytes,
            filename: "Z004_260321".to_string(),
            today: TEST_TODAY.to_string(),
        },
    )
    .unwrap();
    // 空スロットは除外され、正常行1件のみマッチ
    assert_eq!(result.matched_rows.len(), 1);
    assert_eq!(result.matched_rows[0].product_code, "TEST-001");
    // 空スロットはエラーにもカウントされない（unmatchedにもならない）
    // 4912345678902 はDB未登録だが qty=0,amt=0 なので除外される
}

#[test]
fn test_parse_and_validate_req401_normal_matching() {
    // REQ-401: CSV取込み
    let (_dir, conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    create_test_product_with_jan(&conn, "TEST-002", "4912345678902", 5, false);
    let bytes = make_z004_bytes(
        1,
        "2026-03-21",
        &[
            ("4912345678901", "商品A", 3, 900),
            ("4912345678902", "商品B", 1, 200),
        ],
    );
    let result = parse::parse_and_validate(
        &conn,
        CsvParseAndValidateRequest {
            file_bytes: bytes,
            filename: "Z004_260321".to_string(),
            today: TEST_TODAY.to_string(),
        },
    )
    .unwrap();
    assert_eq!(result.matched_rows.len(), 2);
    assert_eq!(result.matched_rows[0].product_code, "TEST-001");
    assert_eq!(result.matched_rows[0].quantity, 3);
    assert_eq!(result.matched_rows[1].product_code, "TEST-002");
    assert_eq!(result.preview_data.matched_summary.count, 2);
    assert_eq!(result.preview_data.matched_summary.total_amount, 1100);
    assert_eq!(result.preview_data.error_summary.count, 0);
    assert!(!result.preview_token.is_empty());
}

#[test]
fn test_parse_and_validate_req401_unmatched_product() {
    // REQ-401: CSV取込み
    let (_dir, conn) = setup_test_db();
    // DBに商品を登録しない
    let bytes = make_z004_bytes(1, "2026-03-21", &[("4912345678901", "未登録商品", 2, 500)]);
    let result = parse::parse_and_validate(
        &conn,
        CsvParseAndValidateRequest {
            file_bytes: bytes,
            filename: "Z004_260321".to_string(),
            today: TEST_TODAY.to_string(),
        },
    )
    .unwrap();
    assert_eq!(result.matched_rows.len(), 0);
    assert_eq!(result.error_rows.len(), 1);
    assert_eq!(
        result.error_rows[0].error_type,
        CsvImportErrorType::UnmatchedProduct
    );
    assert!(result.error_rows[0].error_message.contains("4912345678901"));
}

#[test]
fn test_parse_and_validate_req401_multiple_jan_hits() {
    // REQ-401: CSV取込み
    let (_dir, conn) = setup_test_db();
    // 同じJANで2商品（グループコード）
    create_test_product_with_jan(&conn, "FS-0001", "4912345678901", 10, true);
    create_test_product_with_jan(&conn, "FS-0002", "4912345678901", 5, true);
    let bytes = make_z004_bytes(
        1,
        "2026-03-21",
        &[("4912345678901", "グループ商品", 1, 300)],
    );
    let result = parse::parse_and_validate(
        &conn,
        CsvParseAndValidateRequest {
            file_bytes: bytes,
            filename: "Z004_260321".to_string(),
            today: TEST_TODAY.to_string(),
        },
    )
    .unwrap();
    // ORDER BY product_code ASC で先頭を採用
    assert_eq!(result.matched_rows.len(), 1);
    assert_eq!(result.matched_rows[0].product_code, "FS-0001");
    // 警告がwarningsに追加される
    assert_eq!(result.preview_data.matched_summary.warnings.len(), 1);
    assert!(result.preview_data.matched_summary.warnings[0].contains("複数商品"));
}

#[test]
fn test_parse_and_validate_req401_parse_errors_merged() {
    // REQ-401: CSV取込み
    let (_dir, conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    // フィールド数不足の行を含むZ004
    let bytes = z004_text_bytes(&[
        "\"1\",\"4912345678901\",\"正常行\",\"3\",\"900\"".to_string(),
        "\"2\",\"bad_line\"".to_string(),
    ]);
    let result = parse::parse_and_validate(
        &conn,
        CsvParseAndValidateRequest {
            file_bytes: bytes,
            filename: "Z004_260321".to_string(),
            today: TEST_TODAY.to_string(),
        },
    )
    .unwrap();
    assert_eq!(result.matched_rows.len(), 1);
    // parse_errors がマージされてerror_rowsに含まれる
    assert!(!result.error_rows.is_empty());
    let format_error = result
        .error_rows
        .iter()
        .find(|e| e.error_type == CsvImportErrorType::InvalidFormat);
    assert!(format_error.is_some());
}

#[test]
fn test_parse_and_validate_req401_no_valid_data() {
    // REQ-401 / 32「確定」・packet S5・b1 Matrix Z9（期待を変えた。b1 Matrix T2）: 正常な行が全て 0 の
    // file は拒まず、売上の行も行の error も無い preview になる（精算の識別メタだけを持つ取込みとして確定できる）
    let (_dir, conn) = setup_test_db();
    // 全行が空レコード（qty=0, amt=0）
    let bytes = make_z004_bytes(
        1,
        "2026-03-21",
        &[
            ("4912345678901", "空1", 0, 0),
            ("4912345678902", "空2", 0, 0),
        ],
    );
    let result = parse::parse_and_validate(
        &conn,
        CsvParseAndValidateRequest {
            file_bytes: bytes,
            filename: "Z004_260321".to_string(),
            today: TEST_TODAY.to_string(),
        },
    );
    let result = result.expect("全て 0 の file は拒まない");
    assert!(result.matched_rows.is_empty());
    assert!(result.error_rows.is_empty());
    assert_eq!(result.preview_data.matched_summary.count, 0);
    assert_eq!(result.preview_data.matched_summary.total_amount, 0);
    assert_eq!(result.preview_data.error_summary.count, 0);
}

#[test]
fn test_parse_and_validate_req401_file_hash_blocking() {
    // REQ-401 / I-B1 / SPEC-SDI-D1: active同一hashはblockし、rolled_back後は再取込み可能。
    let (_dir, conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    let bytes = make_z004_bytes(1, "2026-03-21", &[("4912345678901", "商品A", 3, 900)]);

    // まず1回パース → file_hash を取得して直接DBに登録
    let first_result = parse_and_build_cache(&conn, bytes.clone(), "Z004_260321");
    let file_hash = &first_result.preview_data.file_info.file_hash;
    let blocking_id = sales_repo::insert_csv_import(
        &conn,
        &NewCsvImport {
            filename: "Z004_260321".to_string(),
            settlement_date: "2026-03-21".to_string(),
            file_hash: file_hash.clone(),
            total_items: 1,
            total_amount: 900,
            skipped_count: 0,
            status: "completed".to_string(),
            machine_no: "0001".to_string(),
            settlement_no: 1,
        },
    )
    .unwrap();

    // 同じファイルで再パース → ブロック
    let result = parse::parse_and_validate(
        &conn,
        CsvParseAndValidateRequest {
            file_bytes: bytes,
            filename: "Z004_260321".to_string(),
            today: TEST_TODAY.to_string(),
        },
    );
    assert!(result.is_err());
    match result.unwrap_err() {
        BizError::ImportError(msg) => assert!(msg.contains("取込み済み")),
        e => panic!("Expected ImportError, got {:?}", e),
    }

    sales_repo::update_csv_import_status(&conn, blocking_id, "rolled_back").unwrap();
    let retry = parse::parse_and_validate(
        &conn,
        CsvParseAndValidateRequest {
            file_bytes: make_z004_bytes(1, "2026-03-21", &[("4912345678901", "商品A", 3, 900)]),
            filename: "Z004_260321".to_string(),
            today: TEST_TODAY.to_string(),
        },
    );
    assert!(retry.is_ok());
}

#[test]
fn test_parse_and_validate_req401_negative_quantity_return() {
    // REQ-401: CSV取込み
    // 返品行（quantity < 0）が正常にmatched_rowsに入ることを確認
    let (_dir, conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    let bytes = make_z004_bytes(1, "2026-03-21", &[("4912345678901", "返品商品", -1, -500)]);
    let result = parse::parse_and_validate(
        &conn,
        CsvParseAndValidateRequest {
            file_bytes: bytes,
            filename: "Z004_260321".to_string(),
            today: TEST_TODAY.to_string(),
        },
    )
    .unwrap();
    assert_eq!(result.matched_rows.len(), 1);
    assert_eq!(result.matched_rows[0].quantity, -1);
    assert_eq!(result.matched_rows[0].amount, -500);
    assert_eq!(result.preview_data.matched_summary.total_amount, -500);
}

#[test]
fn test_parse_and_validate_req401_error_summary_truncation() {
    // REQ-401: CSV取込み
    // error_rows > 100件のとき、error_summary.items が100件に截断されることを確認
    let (_dir, conn) = setup_test_db();
    // 110件の未登録JAN行を生成
    let mut data_lines = Vec::new();
    for i in 0..110 {
        let jan = format!("490000000{:04}", i);
        data_lines.push((jan, format!("未登録商品{}", i), 1i32, 100i32));
    }
    let refs: Vec<(&str, &str, i32, i32)> = data_lines
        .iter()
        .map(|(j, n, q, a)| (j.as_str(), n.as_str(), *q, *a))
        .collect();
    let bytes = make_z004_bytes(1, "2026-03-21", &refs);
    let result = parse::parse_and_validate(
        &conn,
        CsvParseAndValidateRequest {
            file_bytes: bytes,
            filename: "Z004_260321".to_string(),
            today: TEST_TODAY.to_string(),
        },
    )
    .unwrap();
    // 全行がunmatched → error_rows = 110件
    assert_eq!(result.error_rows.len(), 110);
    // error_summary.count は全件数
    assert_eq!(result.preview_data.error_summary.count, 110);
    // error_summary.items は先頭100件に截断
    assert_eq!(result.preview_data.error_summary.items.len(), 100);
}

/// inline の Z004（layout A、精算 1、2026-03-21）。カンマ付きの値を書くため文字列で受ける
fn z004_text_bytes(data_lines: &[String]) -> Vec<u8> {
    layout_a_bytes("0001", "0001", "2026-03-21", data_lines)
}

fn parse_bytes(conn: &crate::db::DbConnection, bytes: Vec<u8>) -> ParseValidateResult {
    parse::parse_and_validate(
        conn,
        CsvParseAndValidateRequest {
            file_bytes: bytes,
            filename: "Z004_260321".to_string(),
            today: TEST_TODAY.to_string(),
        },
    )
    .unwrap()
}

#[test]
fn test_parse_and_validate_req401_zero_code_sales_reported() {
    // REQ-401 / SPEC-Z4A-D8 / 32 §15.3 4c: 商品コードの無い枠の売上は invalid_jan の ErrorRow（D-103）
    let (_dir, conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    let bytes = z004_text_bytes(&[
        "\"1\",\"4912345678901\",\"正常行\",\"3\",\"900\"".to_string(),
        "\"2\",\"00000000000000\",\"\",\"2\",\"800\"".to_string(),
        "\"3\",\"00000000000000\",\"\",\"1,000\",\"1,234\"".to_string(),
    ]);
    let result = parse_bytes(&conn, bytes);

    assert_eq!(result.matched_rows.len(), 1);
    assert_eq!(result.error_rows.len(), 2);
    for e in &result.error_rows {
        assert_eq!(e.error_type, CsvImportErrorType::InvalidJan);
        assert_eq!(e.normalized_jan, None);
        assert!(e.error_message.contains("商品コードの無い枠"));
    }
    let raws: Vec<(&str, &str)> = result
        .error_rows
        .iter()
        .map(|e| (e.raw_quantity.as_str(), e.raw_amount.as_str()))
        .collect();
    assert_eq!(raws, vec![("2", "800"), ("1,000", "1,234")]);
}

#[test]
fn test_parse_and_validate_req401_comma_amount_matched() {
    // REQ-401 / SPEC-Z4A-D7: カンマ付きの金額が BIZ-03 のマスタ照合へ届く
    let (_dir, conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    let bytes = z004_text_bytes(&["\"1\",\"4912345678901\",\"商品A\",\"1\",\"1,234\"".to_string()]);
    let result = parse_bytes(&conn, bytes);

    assert!(result.error_rows.is_empty());
    assert_eq!(result.matched_rows.len(), 1);
    assert_eq!(result.matched_rows[0].amount, 1234);
    assert_eq!(result.preview_data.matched_summary.total_amount, 1234);
}

#[test]
fn test_parse_and_validate_req401_non_jan_zero_rows_no_error() {
    // REQ-401 / SPEC-Z4A-D8 / SPEC-Z4A-D5: 8桁独自コード + EEEEEE の0/0の枠は error_rows に入らない
    let (_dir, conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    let bytes = make_z004_bytes(
        1,
        "2026-03-21",
        &[
            ("4912345678901", "正常商品", 3, 900),
            ("12345678EEEEEE", "独自1", 0, 0),
            ("12345678EEEEEE", "独自2", 0, 0),
        ],
    );
    let result = parse_bytes(&conn, bytes);

    assert_eq!(result.matched_rows.len(), 1);
    assert!(result.error_rows.is_empty());
}

#[test]
fn test_parse_and_validate_req401_invalid_jan_rows_kept_in_preview_cap() {
    // REQ-401 / 32 §15.2 ErrorSummary・§15.3 手順 5c: items は invalid_jan を先に選び line_no の昇順
    let (_dir, conn) = setup_test_db();
    let mut lines = vec!["\"1\",\"4900000000000\",\"小数\",\"1\",\"1.5\"".to_string()];
    lines.extend((0..100).map(|i| {
        format!(
            "\"{}\",\"490000001{i:04}\",\"未登録{i}\",\"1\",\"100\"",
            i + 2
        )
    }));
    lines.push("\"102\",\"00000000000000\",\"\",\"1\",\"500\"".to_string());
    let result = parse_bytes(&conn, z004_text_bytes(&lines));

    let summary = &result.preview_data.error_summary;
    assert_eq!(summary.count, 102);
    assert_eq!(summary.items.len(), 100);
    assert!(summary
        .items
        .iter()
        .any(|e| e.error_type == CsvImportErrorType::InvalidJan
            && e.error_message.contains("商品コードの無い枠")));
    assert!(summary
        .items
        .iter()
        .any(|e| e.error_type == CsvImportErrorType::InvalidNumber
            && e.line_no == LAYOUT_A_FIRST_DATA_LINE));
    assert!(summary
        .items
        .windows(2)
        .all(|w| w[0].line_no < w[1].line_no));
}

#[test]
fn test_parse_and_validate_req401_invalid_settlement_date() {
    // REQ-401: CSV取込み
    let (_dir, conn) = setup_test_db();
    // 不正日付（2026-99-99）を含むZ004
    let text = "\"精算日\",\"2026-99-99\",\"\",\"\",\"\"\r\n\
         \"No.\",\"コード\",\"名\",\"個\",\"額\"\r\n\
         \"1\",\"4912345678901\",\"商品\",\"1\",\"100\""
        .to_string();
    let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode(&text);
    let result = parse::parse_and_validate(
        &conn,
        CsvParseAndValidateRequest {
            file_bytes: encoded.into_owned(),
            filename: "Z004_bad_date".to_string(),
            today: TEST_TODAY.to_string(),
        },
    );
    assert!(result.is_err());
    match result.unwrap_err() {
        BizError::ImportError(msg) => assert!(msg.contains("不正な日付")),
        e => panic!("Expected ImportError, got {:?}", e),
    }
}

#[test]
fn test_parse_and_validate_req401_future_settlement_date() {
    // REQ-401: CSV取込み。32 の知らせ (b): 精算日が今日より後は拒否せず知らせる（期待を変える。Matrix T2）
    let (_dir, conn) = setup_test_db();
    let bytes = make_z004_bytes(1, "2099-12-31", &[("4912345678901", "商品", 1, 100)]);
    let result = parse::parse_and_validate(
        &conn,
        CsvParseAndValidateRequest {
            file_bytes: bytes,
            filename: "Z004_future".to_string(),
            today: "2099-12-30".to_string(),
        },
    )
    .unwrap();
    assert!(result.preview_data.settlement_date_after_today);
}

// --- 精算の識別・並び・欠け（ADR SPEC-STK-CNT-D4・D7、32 の表の順 1〜5 と知らせ） ---

const LINE_A: [(&str, &str, i32, i32); 1] = [("4912345678901", "商品A", 1, 100)];

fn preview(
    conn: &crate::db::DbConnection,
    bytes: Vec<u8>,
    today: &str,
) -> Result<ParseValidateResult, BizError> {
    parse::parse_and_validate(
        conn,
        CsvParseAndValidateRequest {
            file_bytes: bytes,
            filename: "Z004.CSV".to_string(),
            today: today.to_string(),
        },
    )
}

fn import_error(result: Result<ParseValidateResult, BizError>) -> String {
    match result {
        Err(BizError::ImportError(message)) => message,
        other => panic!(
            "ImportError を期待しました: {:?}",
            other.map(|r| r.settlement)
        ),
    }
}

/// 順 2〜5 の比べる先にならない取込み（取消済み、レジ 0001）を直に入れ、取込み ID を返す。
/// 順 4 が名指す取込みの ID を 1 以外にするための fixture（Matrix RB7。commit_tests も使う）
pub(super) fn insert_rolled_back_import(conn: &crate::db::DbConnection, settlement_no: i64) -> i64 {
    sales_repo::insert_csv_import(
        conn,
        &NewCsvImport {
            filename: format!("rolled-back-{settlement_no}"),
            settlement_date: "2026-01-01".into(),
            file_hash: format!("rolled-back-hash-{settlement_no}"),
            total_items: 0,
            total_amount: 0,
            skipped_count: 0,
            status: "rolled_back".into(),
            machine_no: "0001".into(),
            settlement_no,
        },
    )
    .unwrap()
}

fn csv_import_count(conn: &crate::db::DbConnection) -> i64 {
    conn.query_row("SELECT COUNT(*) FROM csv_imports", [], |row| row.get(0))
        .unwrap()
}

const ORDER1_REJECT: &str =
    "精算回数の無い file は取り込めません。レジの SD の商品別の売上（Z004）を選んでください";
const ORDER2_ACTIVE_REJECT: &str =
    "レジの番号が前の精算と合いません。レジの入替・初期化を確かめてください";
const ORDER5_REJECT: &str =
    "レジの日付か番号が前の精算と合いません。レジの時計か、レジの入替・初期化を確かめてください";

#[test]
fn test_parse_req401_rejects_missing_or_out_of_range_settlement_no() {
    // REQ-401 / ADR D4 / 32 の表の順 1（Matrix Z1）: メタの無い file・数字でない・範囲の外は preview も確定も拒否
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    for bytes in [
        make_conventional_z004_bytes("2026-03-21", &LINE_A),
        make_z004_bytes_raw("0001", "12A", "2026-03-21", &LINE_A),
        make_z004_bytes_raw("0001", "-1", "2026-03-21", &LINE_A),
        make_z004_bytes_raw("0001", "9007199254740992", "2026-03-21", &LINE_A),
        make_z004_bytes_raw("", "0001", "2026-03-21", &LINE_A),
    ] {
        assert_eq!(
            import_error(preview(&conn, bytes, TEST_TODAY)),
            ORDER1_REJECT
        );
    }
    // 境: 9007199254740991 は受ける
    let max = preview(
        &conn,
        make_z004_bytes_raw("0001", "9007199254740991", "2026-03-21", &LINE_A),
        TEST_TODAY,
    )
    .unwrap();
    assert_eq!(max.settlement.settlement_no, 9_007_199_254_740_991);
    // 確定も cache の精算の識別メタで順 1 を取り直す
    let mut cached = build_cached(max);
    cached.settlement.settlement_no = 9_007_199_254_740_992;
    let result = commit_csv_import(
        &mut conn,
        CommitRequest {
            cached_data: cached,
            today: TEST_TODAY.to_string(),
        },
    );
    assert!(matches!(result, Err(BizError::ImportError(ref m)) if m == ORDER1_REJECT));
    assert_eq!(csv_import_count(&conn), 0);
}

#[test]
fn test_parse_req401_rejects_other_machine_no() {
    // REQ-401 / ADR D4 / 32 の表の順 2（Matrix Z2）: active な取込みと machine_no が違えば今の文で拒み、
    // 取り消した取込みを名指さない（照合先は active を優先する）
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    let rolled = commit_bytes(&mut conn, make_z004_bytes(1, "2026-03-20", &LINE_A), "a").unwrap();
    rollback_csv_import(&mut conn, rolled.csv_import_id).unwrap();
    commit_bytes(&mut conn, make_z004_bytes(2, "2026-03-21", &LINE_A), "b").unwrap();
    let message = import_error(preview(
        &conn,
        make_z004_bytes_for_machine("0002", 3, "2026-03-22", &LINE_A),
        TEST_TODAY,
    ));
    assert_eq!(message, ORDER2_ACTIVE_REJECT);
    // 同じレジの次の精算は通る
    assert!(preview(&conn, make_z004_bytes(3, "2026-03-22", &LINE_A), TEST_TODAY).is_ok());
}

#[test]
fn test_parse_req401_rejects_other_version_of_same_settlement_and_allows_after_rollback() {
    // REQ-401 / ADR D4・D7 / 32 の表の順 4（Matrix Z4）: 同じ精算の active な取込みで hash が違えば拒み、
    // 取り消した取込みとは比べない（取り込み直せる）
    // 名指す取込みの ID を 1 以外にする（Matrix RB7。取消済みの取込みは比べる先にならない）
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    insert_rolled_back_import(&conn, 4);
    let first = commit_bytes(&mut conn, make_z004_bytes(5, "2026-03-21", &LINE_A), "a").unwrap();
    assert_ne!(first.csv_import_id, 1);
    let other = make_z004_bytes(5, "2026-03-21", &[("4912345678901", "商品A", 2, 200)]);
    let expected = format!(
        "同じ精算と思われる別の資料が既に取り込まれています（精算 5、取込みID: {}）",
        first.csv_import_id
    );
    match preview(&conn, other.clone(), TEST_TODAY) {
        Err(BizError::SettlementConflict {
            message,
            csv_import_id,
        }) => {
            assert_eq!(message, expected);
            assert_eq!(csv_import_id, first.csv_import_id);
        }
        other => panic!(
            "SettlementConflict を期待しました: {:?}",
            other.map(|r| r.settlement)
        ),
    }
    rollback_csv_import(&mut conn, first.csv_import_id).unwrap();
    commit_bytes(&mut conn, other, "b").unwrap();
    let p = crate::db::product_repo::find_by_product_code(&conn, "TEST-001")
        .unwrap()
        .unwrap();
    assert_eq!(p.product.stock_quantity, 8);
}

#[test]
fn test_parse_req401_rejects_settlement_order_mismatch_both_directions() {
    // REQ-401 / ADR D4 / 32 の表の順 5（Matrix Z5）: 精算回数と精算日の順の食い違いは両方向で拒む
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    commit_bytes(&mut conn, make_z004_bytes(10, "2026-03-10", &LINE_A), "a").unwrap();
    for bytes in [
        make_z004_bytes(11, "2026-03-09", &LINE_A),
        make_z004_bytes(9, "2026-03-11", &LINE_A),
    ] {
        assert_eq!(
            import_error(preview(&conn, bytes, TEST_TODAY)),
            ORDER5_REJECT
        );
    }
    // 同じ精算日の次の精算・前の日の前の精算は通る
    assert!(preview(
        &conn,
        make_z004_bytes(11, "2026-03-10", &LINE_A),
        TEST_TODAY
    )
    .is_ok());
    assert!(preview(&conn, make_z004_bytes(9, "2026-03-09", &LINE_A), TEST_TODAY).is_ok());
}

#[test]
fn test_parse_req401_settlement_gap_notices_and_future_date() {
    // REQ-401 / ADR D4 / 32 の知らせ（Matrix Z6）: 欠けは連続の区間にまとめ昇順（1 つなら from = to）、
    // 区間の数は欠けた番号の数に依らない。精算日が今日より後は知らせで拒否しない。preview は書かない
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    commit_bytes(&mut conn, make_z004_bytes(1, "2026-03-01", &LINE_A), "a").unwrap();
    commit_bytes(&mut conn, make_z004_bytes(3, "2026-03-03", &LINE_A), "b").unwrap();
    let before = csv_import_count(&conn);
    let gaps = |result: &ParseValidateResult| {
        result
            .preview_data
            .settlement_gaps
            .iter()
            .map(|g| (g.from, g.to))
            .collect::<Vec<_>>()
    };
    let six = preview(
        &conn,
        make_z004_bytes(6, "2026-03-06", &LINE_A),
        "2026-03-06",
    )
    .unwrap();
    assert_eq!(gaps(&six), vec![(2, 2), (4, 5)]);
    assert!(!six.preview_data.settlement_date_after_today);
    let huge = preview(
        &conn,
        make_z004_bytes(9_007_199_254_740_991, "2026-03-10", &LINE_A),
        "2026-03-09",
    )
    .unwrap();
    assert_eq!(gaps(&huge), vec![(2, 2), (4, 9_007_199_254_740_990)]);
    assert!(huge.preview_data.settlement_date_after_today);
    assert_eq!(csv_import_count(&conn), before);
}

#[test]
fn test_parse_req401_length_unit_quantity_converted_to_cm() {
    // REQ-401 / SPEC-UNIT-D4・D8 / 32 の数量（Matrix Z12）: 長さの商品の整数の数量は × 100 の cm で
    // 売上と movement に入る。× 100 の後の絶対値が 9999999 を越える行は preview の行の error。小数は今の行の error
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "LEN-001", "4912345678911", 1000, true);
    conn.execute(
        "UPDATE products SET stock_unit = 'm' WHERE product_code = 'LEN-001'",
        [],
    )
    .unwrap();
    let bytes = make_z004_bytes(
        1,
        "2026-03-21",
        &[
            ("4912345678911", "生地", 99_999, 100),
            ("4912345678911", "生地", 100_000, 100),
            ("4912345678911", "生地", -100_000, -100),
        ],
    );
    let result = preview(&conn, bytes, TEST_TODAY).unwrap();
    assert_eq!(result.matched_rows.len(), 1);
    assert_eq!(result.matched_rows[0].quantity, 9_999_900);
    assert_eq!(result.error_rows.len(), 2);
    assert!(result
        .error_rows
        .iter()
        .all(|e| e.error_type == CsvImportErrorType::InvalidNumber));

    let decimal = z004_text_bytes(&[
        "\"1\",\"4912345678911\",\"生地\",\"1.5\",\"150\"".to_string(),
        "\"2\",\"4912345678911\",\"生地\",\"2\",\"200\"".to_string(),
    ]);
    let decimal = preview(&conn, decimal, TEST_TODAY).unwrap();
    assert_eq!(decimal.matched_rows.len(), 1);
    assert_eq!(decimal.matched_rows[0].quantity, 200);
    assert_eq!(decimal.error_rows.len(), 1);

    commit_bytes(
        &mut conn,
        make_z004_bytes(2, "2026-03-22", &[("4912345678911", "生地", 3, 900)]),
        "len",
    )
    .unwrap();
    let (sale, movement): (i64, i64) = conn
        .query_row(
            "SELECT s.quantity, m.quantity FROM sale_records s
             JOIN inventory_movements m ON m.reference_type = 'csv_import'
                 AND m.reference_id = s.csv_import_id AND m.product_code = s.product_code
             WHERE s.product_code = 'LEN-001'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!((sale, movement), (300, -300));
    let p = crate::db::product_repo::find_by_product_code(&conn, "LEN-001")
        .unwrap()
        .unwrap();
    assert_eq!(p.product.stock_quantity, 700);
}

#[test]
fn test_parse_req401_rejects_other_machine_after_all_imports_rolled_back() {
    // REQ-401 / ADR D4・D7 / 32 の表の順 2（Matrix Z22。round 3・closure の是正）: 取消で active な取込みが
    // 0 件になった後も、取り消した取込みと machine_no を比べ、拒否の文でその取込みを名指す。
    // 同じレジの同じ精算の別の版は取消の後に取り込める（D7）
    let today = "2026-05-10";
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    // レジ B の file の preview は取込みの無い DB で作っておく（確定の TX の順 2 を見る）
    let other_register = build_cached(
        preview(
            &conn,
            make_z004_bytes_for_machine("0002", 1, "2026-05-11", &LINE_A),
            today,
        )
        .unwrap(),
    );
    let hundred =
        commit_bytes(&mut conn, make_z004_bytes(100, "2026-05-09", &LINE_A), "a").unwrap();
    crate::biz::stocktake_service::save_stock_count(
        &mut conn,
        &crate::biz::stocktake_service::SaveStockCountRequest {
            product_code: "TEST-001".to_string(),
            count_value: 5,
            fresh: true,
            business_date: today.to_string(),
            stocktake_item_id: None,
        },
        today,
    )
    .unwrap();
    rollback_csv_import(&mut conn, hundred.csv_import_id).unwrap();
    let imported_at = sales_repo::find_csv_import_by_id(&conn, hundred.csv_import_id)
        .unwrap()
        .unwrap()
        .imported_at;
    let named = format!(
        "取り消した取込みとレジの番号が合いません（取込みID: {}、精算 100、取込み日: {imported_at}、レジ: 0001。この file のレジ: 0002）。どちらが店のレジの資料かを確かめてください",
        hundred.csv_import_id
    );
    let message = import_error(preview(
        &conn,
        make_z004_bytes_for_machine("0002", 1, "2026-05-11", &LINE_A),
        today,
    ));
    assert_eq!(message, named);
    assert_ne!(message, ORDER2_ACTIVE_REJECT);
    let before = csv_import_count(&conn);
    let result = commit_csv_import(
        &mut conn,
        CommitRequest {
            cached_data: other_register,
            today: today.to_string(),
        },
    );
    assert!(matches!(result, Err(BizError::ImportError(ref m)) if *m == named));
    assert_eq!(csv_import_count(&conn), before);
    assert_eq!(stock_copy(&conn, "TEST-001"), (5, false, None));
    // 対照: 同じレジの精算 100 の別の版は順 2 を通り、取消の後に取り込める
    commit_bytes(
        &mut conn,
        make_z004_bytes(100, "2026-05-09", &[("4912345678901", "商品A", 2, 200)]),
        "b",
    )
    .unwrap();
}
