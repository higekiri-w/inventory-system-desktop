# Plan Packet: 日報（Z001・Z002・Z005）の数の読取りを Z004 と同じ厳しさにそろえる

wave に属さない単独の lane。出所は owner の決定 2026-11-17（repo 外の回答台帳 TD-775、原文「そろえるべきだと思う」）と、differential test を始める owner の決定（TD-770。本 lane が最初の当て所）。調べの起点は相談役（Fable）の code の読み（2026-11-17、実行は未確認）で、下の「調べ」で現物を読み直した。本 commit は Plan Packet の起票だけで、`src-tauri/**` と設計正本（29・23・37・decision-log）は計画の固定の後の実装の PR で変える（Design Readiness）。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R3

Reason:
POS CSV（日報 Z001・Z002・Z005）の受ける値の契約を変える（今まで受けていた区切りの崩れた値・`¥`・数の中の空白・`+` を `invalid_number` にする）。日報の取込み（BIZ-08）の利用者向けの文も 1 つ変える（下の Scope の S4）。DEV_WORKFLOW Risk Tiers の R3「POS CSV」「operator の業務の流れ」。R4 にしない理由: DB の schema・migration・書込み・取込み済みの行・backup・command の DTO・bindings・画面の file に触れない（読取りの段で止まる値が増えるだけで、取込み済みのデータは読み直さない。下の「調べ」の F6）。

## Goal

Goal Invariant:

### 最小完了条件

- 日報の数の列（Z001 の第 3・4 列、Z002 の第 3・4 列、Z005 の第 3・4 列）は、Z004 と同じ 1 つの整数の形の規則（D-103 の (1)、23 §13.4.1: `-?[0-9]+` か `-?[1-9][0-9]{0,2}(,[0-9]{3})+`、前後の空白は trim）で読む。区切りの崩れた値（`0,123`・`1,2,3`・`1,23`・`1234,567`・`,123`・`123,`・`1,,234`）、`¥`・`￥` 付き、数の中の空白（`1 234`）、`+` 付きは `invalid_number` になり、別の数として読まれない。
- 日報だけの必要は「承認済みの差」として残る: (A1) 空の値は今どおり列ごとに `None`（Z005 の金額は必須で `invalid_number`）、(A2) 整数の範囲は今どおり IO-07-D6（絶対値 `9999999999`、越えれば `number_too_large`）、(A3) 個数の列（Z001 の総売の行の第 3 列、Z005 の第 3 列）は整数部に上の規則を当てた上で `.` + 1〜2 桁の小数を受ける（IO-07-D2。`1,234.5` → 123450）。
- 2 つの parser の数の読取りに同じ入力の表を通す差分の test が、承認済みの差の表（A1〜A3）に無い差で fail する。
- 手で選ぶ経路（「ファイルを選んで取り込む」）で数の列が読めないとき、利用者向けの文がどの帳票（= どの file。束は帳票ごとに 1 本）のどの列が読めないかを名指す（S4。値・file 名・行番号は出さない。BIZ-08-D1）。

### 失敗定義

- 区切りの崩れた値・`¥`・数の中の空白・`+` を、日報のどれかの数の列が数として受ける。
- 正しい 3 桁区切り（`1,234`・`-12,345`・`1,000,000`）、カンマなしの整数、前後の空白、個数の列の `1,234.5`・`1.3`・`-2` のどれかを、今と違う値で読むか拒む。
- A1〜A3 のどれかが変わる（空の列が error になる、上限の扱いが変わる、件数の列が小数を受ける）。
- 差分の test の承認済みの差の表に、理由の無い行が足される。
- `settlement_mismatch`・`number_too_large` の文の優先（BIZ-08-D2・D2a）が変わる。

### 非目的

- `split_csv_fields` の 3 つの写しの統合（別の lane）。
- EJ（IO-08）の金額 token の規則（29-io-ej-parser の「金額 token」）。EJ も桁区切りの位置を確かめない（下の「調べ」の F7）が、本 lane では変えず、計画の review に問う。
- Z004 の数の規則の変更（23 §13.4.1 の受ける形・i32・小数を拒むことは変えない。共有の関数へ移すだけ）、Z004 と日報のほかの列の規則。
- SD 直読みの一覧の「読めません」の行に理由を出すこと（UI-07-D15 の画面の変更。S4 は手で選ぶ経路の文だけ）。
- UI の新しい画面、DTO・bindings・DB の変更。

## 調べ（2026-11-17、main `ddfd174`。code の読みと、tracked の合成 fixture への 1 回の計数）

- F1 日報の数の読取り: `parse_optional_i64`（`src-tauri/src/io/daily_report_parser.rs:538`）と `parse_optional_hundredths`（:562）は先に `clean_number`（:620、`value.trim().replace([',', '￥', '¥', ' '], "")`）を通す。`parse_optional_i64` は `+`/`-` を 1 つ外して ASCII 数字だけを確かめ（:545）、`parse_optional_hundredths` は `-` だけを外す（:571）。`parse_required_i64`（:596）は空を `Invalid` にする。呼出しは Z001 の :291・:293・:307、Z002 の :443・:455、Z005 の :490・:502 の 7 か所だけ。
- F2 Z004 の数の読取り: `parse_z004_int`（`src-tauri/src/io/z004_parser.rs:408`）は trim した値が `^-?(?:[0-9]+|[1-9][0-9]{0,2}(?:,[0-9]{3})+)$` に全体で一致するときだけ、カンマを除いて `i32` に読む。呼出しは :363（数量）・:372（金額）の 2 か所。test は `test_parse_z004_req401_comma_grouped_numbers`（:769）と `test_parse_z004_req401_malformed_comma_invalid_number`（:788、`1,23`・`12,3456`・`1,,234`・`,123`・`1,234,`・`0,123`・`1.5`・`+5`・`2,147,483,648` を拒む）。
- F3 正本の文: 29 IO-07-D2（`docs/function-design/29-io-daily-report-parser.md:87`）は「3桁区切りのカンマと空白は既存の `clean_number` で除いた後」と書く。D-103（`docs/decision-log.md:895`）の Alternatives（:900）と 23 §13.4.1 の却下案（`docs/function-design/23-io-z004-parser.md:235`）は、日報の `clean_number` と同じ緩い除去を「区切りの崩れた値を黙って別の数として読む」として Z004 で不採用にし、「日報側の規則は IO-07 が持ち、本書は変えない」と書く。
- F4 実データの形の記録: 2026-07-06 の実 Z001/Z002/Z004/Z005 の匿名化解析（`docs/plu-export-and-real-csv-verification.md:118`）は「全フィールドがダブルクォート囲み。Z001/Z002 の金額にはクォート内の千位区切りカンマ（`"##,###"`）が実在する」と書き、`¥`・数の中の空白・`+` の観測を書かない。同 :120 の「カンマ/¥除去を実装済み」は実装の記述で、`¥` の観測ではない。29 §29.4 の shape の表（:173〜:190）にも `¥`・空白の形は無い。→ `¥`・`￥`・数の中の空白・`+` は受けない側に倒す（根拠は観測の不在。repo 外の詳細の要約〈`inventory-field-check/summaries/2026-07-06-z00x-shape-analysis.md`〉は本起票では読んでいない）。レジの日報の金額の形は「3 桁区切りのカンマ付き（1,000 以上）」で、正しい 3 桁区切りは今後も受ける。
- F5 tracked の合成 fixture（`tests/fixtures/daily-report/`・`daily-report-z001/`・`daily-report-sheet/` の Z001・Z002・Z005 の 21 本）: ヘッダの後の第 3・4 列の空でない値を、D-103 の (1) の整数部に任意の `.` + 1〜2 桁を足した式で数えた。測定の command: `python3 -I - tests/fixtures/daily-report*/Z00[125]*.CSV`（CP932 で decode、`csv.reader`、ヘッダ `レコード` の後の 4 列の行の第 3・4 列）。出力: `values 556 with_comma 0 rejected_by_strict 0`。→ 新しい規則で既存の fixture の bundle の test の値は変わらない見込み（カンマ付きの値は fixture に無いので、正しい 3 桁区切りの受理は T2 で別に確かめる）。daily の unit test の値で崩れた区切り・`¥`・`+`・数の中の空白を使うものは無い（`rg '[0-9],[0-9]|¥|￥|"\+[0-9]'` で `daily_report_parser.rs:1062` の `"1,234.5"` だけ。新しい規則でも受ける）。
- F6 取込み済みのデータへの影響: `parse_daily_report_bundle` の呼出しは `biz/daily_report_import_service/parse.rs:69`（手で選ぶ経路と SD の候補の preview）と `scan.rs:251`（SD の一覧）の 2 か所だけで、どちらも取込みの前の読取り。取込み済みの行（`daily_report_imports` と子の行）は数を保存した後で、読み直す経路は無い（取消は import の id で、bytes を読まない）。影響は 1 つだけ: 崩れた値を含む束を前に取り込んでいた場合、SD の一覧ではその束が「取込み済み」から「読めません」に変わる（`scan.rs:261` で `parse_errors` があれば bundle_hash を作る前に `Unreadable`）。F4 から実データでは起きない見込みで、データは変わらない。
- F6a SD 直読みと予備の経路: 2 つの経路は同じ `parse_daily_report_bundle` を通るので、数の規則は 1 か所の置き換えで両方にそろう。崩れた値の束は、SD の一覧では「読めません」（37 §37.9 手順 5。理由は出さない）、手で選ぶ経路では今は汎用の文「日報ファイルの解析に失敗しました」（`parse.rs:82`、BIZ-08-D1。`test_daily_report_req401_parse_error_logs_parse_failed`〈`biz/daily_report_import_service/tests.rs:147`〉が固定）で、帳票も列も名指さない。帳票と列を名指すのは `number_too_large` だけ（BIZ-08-D2a、`parse.rs:357` の `number_too_large_message`）。IO の `invalid_number` の文（`daily_report_parser.rs:601` の `number_error`、`{帳票}の{列}列を変換できません`）は `number_too_large` の文と同じく帳票と列の語だけで作られ、値・file 名・行番号を含まない。
- F7 EJ: `amount_token`（`src-tauri/src/io/ej_parser.rs:1356`）は桁区切りを「数字の後ならどこでも」受け（:1398）、`1,2,3` を 123、`0,123` を 123 と読む（29-io-ej-parser「金額 token」の文のとおり）。EJ は印字の text で、全角の数字・`，`・`￥` の観測された形を持ち、読めない行は記録を `Unknown` にする別の契約なので、本 lane に入れない（非目的。計画の review に問う）。

### 今の 2 つの読取りの違い（code の読み。実行は実装の段の差分の test が出す）

| 入力（trim の前） | Z004 `parse_z004_int` | 日報の整数の列 `parse_optional_i64` | 日報の個数の列 `parse_optional_hundredths` | そろえた後の日報 |
|---|---|---|---|---|
| `1234` / ` 1234 ` | 1234 | 1234 | 123400 | 同じ |
| `1,234` / `-12,345` / `1,000,000` | 受ける | 受ける | 受ける（×100） | 同じ |
| `0123` / `-0` | 123 / 0 | 123 / 0 | 12300 / 0 | 同じ（両方とも受ける。差なし） |
| `0,123` | invalid | 123 | 12300 | invalid |
| `1,2,3` / `1,23` / `1,,234` | invalid | 123 / 123 / 1234 | ×100 | invalid |
| `1234,567` / `12,3456` | invalid | 1234567 / 123456 | ×100 | invalid |
| `,123` / `123,` / `1,234,` | invalid | 123 / 123 / 1234 | ×100 | invalid |
| `¥1,234` / `￥1234` | invalid | 1234 | 123400 | invalid |
| `1 234`（数の中の ASCII 空白） | invalid | 1234 | 123400 | invalid |
| `1　234`（全角空白） | invalid | invalid | invalid | 同じ |
| `+5` | invalid | 5 | invalid | invalid（日報の中の 2 関数の食い違いも消える） |
| `１２３`（全角数字） | invalid | invalid | invalid | 同じ |
| `1.5` / `1,234.5` | invalid | invalid | 150 / 123450 | 同じ（A3） |
| `1.234` / `.5` / `1.` | invalid | invalid | invalid | 同じ |
| 空・空白だけ | invalid | `None` | `None` | 同じ（A1。Z005 の金額は `parse_required_i64` で invalid） |
| `-` | invalid | invalid | invalid | 同じ |
| `2147483648` | invalid（i32 の外） | 2147483648 | 214748364800 | 同じ（A2） |
| `10000000000` | invalid | `number_too_large` | `number_too_large` | 同じ（A2） |
| `99999999999999999999` | invalid | `number_too_large` | `number_too_large` | 同じ（A2） |
| `9a` / `abc` | invalid | invalid | invalid | 同じ |

## 案の比較

- (a) 採る: D-103 の (1) の整数の形の検査を `src-tauri/src/io/mod.rs` の 1 つの `pub(crate)` の関数に移し、Z004 と日報の 3 関数がそれを通る。`io/mod.rs` は IO の module どうしが共有する唯一の定義を置く既存の場所（`REGISTER_SD_FOLDER`、`io/mod.rs:17`）。新しい file・依存を足さない（`regex` と `LazyLock` は `z004_parser.rs` が既に使う）。
- (b) 不採用: 日報が `z004_parser::parse_z004_int` を呼ぶ。`i32` で日報の金額（上限 `9999999999`）が入らず、形の誤りと上限の越え（IO-07-D6 の `invalid_number` と `number_too_large`）を分けられない。
- (c) 不採用: 日報に同じ正規表現を写す。2 つの写しがずれる（今の食い違いの原因と同じ形）。
- (d) 不採用: 共有の関数を `z004_parser.rs` に置いて日報から呼ぶ。大きさは (a) と同じだが、日報の規則が Z004 の module に依る形になり、読み手が「Z004 だけの規則」と読む。

## Scope

本 commit（plan-first）:

- 本 packet `docs/plans/2026-11-17-number-parse-align.md` だけ。

実装（計画の固定の後）:

- S1 `src-tauri/src/io/mod.rs`: `pub(crate) fn strip_thousands_grouping(raw: &str) -> Option<String>` を足す。trim した値が `^-?(?:[0-9]+|[1-9][0-9]{0,2}(?:,[0-9]{3})+)$` に全体で一致すればカンマを除いた文字列（符号を含む）を、外れれば `None` を返す（D-103 の (1)、23 §13.4.1）。正規表現は `LazyLock` で 1 回だけ作る（`z004_parser.rs:409` の今の形）。差分の test（T1）をこの file の `#[cfg(test)] mod tests` に置く。
- S2 `src-tauri/src/io/z004_parser.rs`: `parse_z004_int` の本体を `crate::io::strip_thousands_grouping(raw)?.parse().ok()` にし、自前の正規表現を消す。受ける形・`i32`・error の文は変えない。T1 から呼ぶため `pub(super)` にする。
- S3 `src-tauri/src/io/daily_report_parser.rs`:
  - `clean_number` を消す。
  - `parse_optional_i64`: trim して空なら `Ok(None)`（A1）。`strip_thousands_grouping` が `None` なら `Err(NumberError::Invalid)`。その後の `parse::<i64>` の桁あふれを `TooLarge` にする分岐と上限の検査（IO-07-D6）は今のまま（形を先に見るので、`test_parse_daily_report_req401_overflow_digits_with_bad_tail_is_invalid_number` の順序の保証も今のまま）。`+`/`-` を外して ASCII 数字を確かめる今の検査（:545〜:548）は共有の関数が持つので消す。
  - `parse_optional_hundredths`: trim して空なら `Ok(None)`。`split_once('.')` で整数部と小数部に分け、整数部を `strip_thousands_grouping` に通す（`None` なら `Invalid`）。小数部が 1〜2 桁の ASCII 数字でなければ `Invalid`（今の規則）。100 倍・上限の計算は今のまま。
  - 2 つの関数を T1 から呼ぶため `pub(super)` にする（`NumberError` も同じ）。
  - T2（下の Test Plan）を `mod tests` に足す。
- S4 `src-tauri/src/biz/daily_report_import_service/parse.rs`: `number_too_large_message` を error の種類を引数に取る形にし（例 `number_column_message(errors, error_type)`）、`settlement_mismatch` → `number_too_large` → `invalid_number` → 汎用の文、の順で選ぶ（`number_too_large` の文の優先と形は変えない。`invalid_number` だけの束は `{帳票}の{列}列を変換できません。…この日報ファイルは取り込めません。レジの日報ファイルを選び直してください。`）。BIZ は `number_too_large` と `invalid_number` 以外の `error_message` を利用者へ渡さない。`test_daily_report_req401_parse_error_logs_parse_failed`（tests.rs:147）の利用者向けの文と `operation_logs.summary` の期待を 37 の改訂の文に直す（設計正本の変更に合わせる直しで、診断の WARN の assert は変えない）。T3 を足す。
- S5 設計正本（実装の PR の中で、code と同じ PR）:
  - `docs/function-design/29-io-daily-report-parser.md`: IO-07-D2 の「（3桁区切りのカンマと空白は既存の `clean_number` で除いた後）」を「整数部は Z004 と同じ整数の形の規則（23 §13.4.1、D-103 の (1)、D-157。29 の中では 23 への link にする）」に置き換える。IO-07 の数の読取りの規則（金額・件数の列も同じ整数の形の規則。区切りの崩れた値・`¥`・数の中の空白・`+` は `invalid_number`。承認済みの差 A1〜A3）を IO-07-D2 の直前か IO-07-D6 の並びに置く（言葉を足す前に、今の IO-07-D2・D6 の文の置き換えで足りるかを先に見る）。§29.5 の `invalid_number` の行の発生条件に「整数の形（D-157）でない」を入れる。更新履歴に 1 行。
  - `docs/function-design/23-io-z004-parser.md` §13.4.1: 却下案の「日報側の規則は IO-07 が持ち、本書は変えない」を「日報（IO-07）も同じ規則で読む（D-157。検査は `io/mod.rs` の 1 か所）」に置き換える。却下の理由の文は残す。
  - `docs/function-design/37-biz-daily-report-import-service.md`: BIZ-08-D2a に、`number_too_large` が無く `invalid_number` があるときの文（同じ形、列の文は IO-07 の `…列を変換できません`）を足し、BIZ-08-D1 の「BIZ は `number_too_large` 以外の `error_message` を利用者へ渡さない」を `invalid_number` を含む形に直す。§37.7 の表に 1 行。更新履歴に 1 行。
  - `docs/decision-log.md`: D-157 を足す（Decision: D-103 の (1) を日報の数の列へ広げ、承認済みの差 A1〜A3 を残す。手で選ぶ経路の `invalid_number` の文。Why: TD-775、区切りの崩れた値を黙って別の数として読まない。Alternatives: 上の案 (b)〜(d)、`¥`・空白を受け続ける〈F4 で観測が無い〉。Guarantee range: F6 の SD の一覧の表示の変化。Compatibility: DB・DTO・bindings・Z004 の受ける形は変えない）。D-103 は今も有効なので `Superseded in part` の行は足さない（D-103 の (1) を置き換えず、範囲を広げる。decision-log の冒頭の規則〈:12〉の「一部が置き換わったとき」に当たらない）。
  - `docs/function-design/90-traceability.md`: `cd src-tauri && cargo run --bin generate_traceability` で再生成（REQ-401 の test が増える）。手で直さない。

Non-scope: `split_csv_fields`、EJ（`ej_parser.rs`）、Z004 の受ける形と `ParsedRow` の型、SD の一覧の画面と `DailyReportSdCandidateStatus`、DTO・command・`src/lib/bindings.ts`、`src/**`、DB の schema・migration、取込み済みのデータ、tracked の fixture（F5 で値が変わらない）。

他の lane との重なり: 本起票では `gh pr list` を引けていない（sandbox の network の制限）。実装の発注の前に Coordinator が open の PR の `gh pr diff N --name-only` と `@@` の行で、`io/mod.rs`・`z004_parser.rs`・`daily_report_parser.rs`・`daily_report_import_service/parse.rs`・`tests.rs`・29・23・37・decision-log の hunk の重なりを確かめる。

## Acceptance Criteria

- AC1（差分の test、red から）: 実装の手順は (i) S2・S3 の `pub(super)` だけを入れ、T1 を足して `cd src-tauri && cargo test --lib io::tests` が red（今の違いの表の「そろえた後」が「同じ」でない行が承認済みの差の表に無い差として出る）、(ii) S1〜S3 の置き換えで T1 が green。(i) の red の出力（差の一覧）を実装の報告と PR に貼る。これが「今の違いの表」の実測になる。
- AC2（日報の経路）: T2 が green（3 帳票の全部の数の列で、崩れた区切り・`¥`・数の中の空白・`+` が `invalid_number`、正しい 3 桁区切りと `1,234.5` の個数を今の値で読む）。
- AC3（利用者向けの文）: T3 が green。`settlement_mismatch` と `number_too_large` の既存の test（`test_daily_report_req401_over_limit_names_column_without_raw_value` ほか）は変えずに green。
- AC4（今の挙動）: `cd src-tauri && cargo test` が全部 green。S4 の 1 本の期待の文の直しのほかに、既存の test を変えない・skip しない・弱めない（Z004 の `test_parse_z004_req401_*` と日報の `test_parse_daily_report_req401_*` を含む）。`cd src && npx vitest run` の対象は変えない（frontend の file に触れない）。
- AC5（mutation、Test Plan）: 実装の後の版に M1〜M6 を 1 つずつ当てて `cd src-tauri && cargo test --lib` を回し、Test Plan の「Would fail if...」の列の test が red（M3 は T1 が green のままであることも確かめる）。結果を実装の報告と PR に書き、mutant は戻す。
- AC6（gate）: `cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings`、`cd src-tauri && cargo run --bin generate_traceability -- --check` が exit 0。
- AC7（docs）: S5 の正本の直しが Scope のとおり。`bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-11-17-number-parse-align.md` が ERROR なし。
- AC8（範囲）: `git diff origin/main --name-only` が Scope の file だけ。`cd src-tauri && cargo run --bin generate_bindings` の後に `src/lib/bindings.ts` の差分なし。

## Design Readiness

- 引用する設計正本（節まで）: `docs/function-design/29-io-daily-report-parser.md` IO-07-D2・IO-07-D6・§29.4・§29.5、`docs/function-design/23-io-z004-parser.md` §13.4.1（SPEC-Z4A-D7）、`docs/function-design/37-biz-daily-report-import-service.md` BIZ-08-D1・D2・D2a・§37.7・§37.9 手順 5、`docs/decision-log.md` D-103・D-104・D-142、`docs/spec/requirements.md` REQ-401、AGENTS.md の「使い手に押しつけない（D-127）」。
- 必要な設計成果物: updated in the implementation PR（S5）。本 commit に正本の直しを置かないのは、S4 の文の採否・`¥` を受けない判断・A1〜A3 の表を計画の review で確かめてから正本に書くため（2026-11-13-z001-row-guard と同じ置き方）。plan にしかない durable な判断（1 つの整数の形の規則・承認済みの差・S4 の文）は実装の PR で 29・23・37・D-157 へ昇格する。
- 前提・制約と延期した design gap: (i) `¥`・空白・`+` を受けない根拠は F4 の観測の不在で、repo 外の詳細の要約と実 file は読んでいない（Contract Probe の候補。計画の review に問う）。(ii) SD の一覧の「読めません」に理由を出さない（非目的。D-127 の観点では gap で、backlog の候補）。(iii) S4 の文は行番号を出さない（BIZ-08-D1 と IO-07-D6 の「値・file 名・行番号を入れない」）。
- 絶対保証の例外と抜け道: 「区切りの崩れた値を数として読まない」は日報の 3 帳票の 7 か所の呼出しが全部 S3 の 2 関数を通ること（F1 の呼出しの一覧）と T2 で見る。EJ は抜け道として残る（F7、非目的）。
- 計画の review の固定の問いへの先回りの答え:
  - (1) 普通の一日: 店主が SD から読む・手で選ぶ、どちらでもレジの出した日報（正しい 3 桁区切りの金額）は今と同じ値で取り込まれる（F4・F5、T2 の正しい区切りの行、既存の fixture の bundle の test）。
  - (2) 登録と生成: command・DTO・bindings・route は変えない。REQ-401 の test が増え、`90-traceability.md` を再生成する。
  - (3) 外の前提: 外の library・機器を足さない。`regex` は入れてある依存。レジの日報の数の形は F4 の記録に依る。
  - (4) データの安全: test は合成の data だけ。実 POS file・実 DB を読まない・commit しない。取込み済みのデータは読み直さない（F6）。
  - (5) 前提の出所: lane と「そろえる」は owner の決定（TD-775）、differential test は TD-770。案 (a) の選択・A1〜A3 の線引き・S4 の文は起票の Writer（AI）の推奨で、owner の決定ではない（S4 は D-127 と発注の「どの file・何行目・何列」から。行番号は BIZ-08-D1 と食い違うので出さない案にした）。
- 判定: ready（計画の review の問いは下の節）。

この変更で不要になるもの（AGENTS.md「継ぎ足さない」）: `clean_number` と、`parse_optional_i64` の自前の符号と数字の検査（S3）、`parse_z004_int` の自前の正規表現（S2）、29 IO-07-D2 の `clean_number` の括弧書き、23 §13.4.1 の「日報側の規則は IO-07 が持ち、本書は変えない」（S5）。足すのは共有の 1 関数と test だけ。

## Contract Ledger

隣の契約の sweep: 日報の 7 か所の数の読取り（F1）、IO-07-D2（個数の小数）、IO-07-D6（上限と誤りの種類の分け方）、§29.5 の error の表、BIZ-08-D1・D2・D2a（利用者向けの文の選び方）、§37.9 手順 5（SD の一覧の `Unreadable`）、23 §13.4.1（Z004 の受ける形）。

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| D-157（新）/ D-103 (1) 日報の数の列も同じ整数の形の規則 | 29 IO-07-D2・§29.5、23 §13.4.1 | S1・S3 | T1、T2 | 非対象（下の「L3 を置かない理由」） |
| SPEC-Z4A-D7 Z004 の受ける形は変えない | 23 §13.4.1 | S2（共有の関数へ移すだけ） | `test_parse_z004_req401_comma_grouped_numbers`・`test_parse_z004_req401_malformed_comma_invalid_number`（変えない）、T1 | 非対象 |
| IO-07-D2 個数の列の小数 2 桁（A3） | 29 IO-07-D2 | S3（整数部だけを共有の関数へ） | `test_parse_daily_report_req401_decimal_quantity_hundredths`・`..._decimal_quantity_rejects_invalid_shapes`・`..._count_column_stays_integer`（変えない）、T1 の A3 の行 | 非対象 |
| IO-07-D6 上限と `invalid_number`/`number_too_large` の分け方（A2） | 29 IO-07-D6 | S3（形の検査の後に今の桁あふれと上限の検査） | `..._accepts_limit_values`・`..._rejects_over_limit_with_column`・`..._overflow_digits_with_bad_tail_is_invalid_number`（変えない）、T1 の A2 の行 | 非対象 |
| 29 §29.4 空の列は `None`、Z005 の金額は必須（A1） | 29 §29.4 の Z001・Z002・Z005 の行 | S3（空の判定を共有の関数より前に置く） | T1 の A1 の行、既存の日報の test | 非対象 |
| BIZ-08-D2a 数の列の誤りは帳票と列を名指す（`invalid_number` へ広げる） | 37 BIZ-08-D1・D2a・§37.7 | S4 | T3、`test_daily_report_req401_parse_error_logs_parse_failed`（期待の文を正本に合わせて直す）、`test_daily_report_req401_over_limit_names_column_without_raw_value`（変えない） | 非対象（文の中身は BIZ の test が見る。画面は message をそのまま出す既存の経路で、`src/features/daily-report-import/DailyReportImportPage.flow.test.tsx:143` が `number_too_large` の文で同じ経路を固定） |
| BIZ-08-D2 `settlement_mismatch` の文が最優先 | 37 BIZ-08-D2 | S4 で順を変えない | 既存の `settlement_mismatch` の test（変えない）、M5 | 非対象 |
| §37.9 手順 5 SD の一覧は parse の誤りで `Unreadable` | 37 §37.9 | 変えない（F6 の表示の変化だけ） | 既存の scan の test（変えない） | 非対象 |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| D-157 / D-103 (1): Z004 と日報の数の読取りは、承認済みの差 A1〜A3 のほかに同じ値を受け同じ値を拒む | 片方だけ緩い・違う値で読む（今の `clean_number`） | differential（unit、`io/mod.rs` の `mod tests`。`parse_z004_int`・`parse_optional_i64`・`parse_optional_hundredths` に同じ入力の表を通す） | T1 `test_number_parse_req401_daily_matches_z004_except_approved_differences`: 入力は上の「今の 2 つの読取りの違い」の表の全行（正しい区切り・崩れた区切り・符号・`¥`・空白・小数・上限・空）。Z004 の `Some(v)` と日報の整数の `Ok(Some(v))`・個数の `Ok(Some(v*100))` を同じとみなし、Z004 の `None` と日報の `Err(_)` を同じとみなす。承認済みの差は test の中の表（入力・差の種類 A1/A2/A3・理由の 1 行・正本の ID）に限り、差がその表の行と一致しなければ fail。差の表に無い差を全部集めてから 1 回で fail する（今の違いの一覧が 1 回の実行で出る）。comment に `REQ-401 / D-157 / D-103 (1)` | 日報のどちらかの関数が `clean_number` に戻る・`+` を受ける・`¥` を外す。Z004 側だけが変わる。承認済みの差の表に理由の無い行が足される（行の理由の欄を必須にする） |
| D-157: 日報の 3 帳票の全部の数の列が規則を通る | 呼出しの 1 か所が共有の関数を通らない | unit（`daily_report_parser.rs` の `mod tests`、`parse_daily_report_bundle` 経由） | T2 `test_parse_daily_report_req401_rejects_malformed_grouping_in_every_number_column`: Z001（総売の個数・ほかの行の件数・金額）、Z002（件数・金額）、Z005（個数・金額）の各列に `0,123`・`1,2,3`・`¥1,234`・`1 234`・`+5` を 1 つずつ入れて、その帳票・その行に `invalid_number`。同じ列に `1,234`・`-12,345`（個数の列は `1,234.5` も）を入れて今の値で読む | 7 か所の呼出しのどれかが別の読取りを使う。正しい 3 桁区切りを拒む |
| BIZ-08-D2a（広げた形）: `invalid_number` の束は帳票と列を名指し、値・行番号を出さない | 汎用の文のまま（今）、値や行番号が漏れる、`number_too_large` の優先が崩れる | unit（`daily_report_import_service/tests.rs`） | T3 `test_daily_report_req401_invalid_number_names_column_without_raw_value`: (i) Z002 の金額に `1,23` の束で、message と `operation_logs.summary` が 37 の文（`Z002の金額列を変換できません。この日報ファイルは取り込めません。レジの日報ファイルを選び直してください。`。期待の文は 37 から転記し production の定数を import しない）で、`1,23`・`9` を含まない。(ii) 同じ束に `number_too_large` の列を足すと `number_too_large` の文だけになる | S4 の分岐が無い・順が逆・`error_message` 以外の値を渡す |
| 同上（mutation） | 日報の整数の列が緩む | mutation | M1: `parse_optional_i64` の共有の関数の呼出しを `value.trim().replace([',', '￥', '¥', ' '], "")` に戻す | T1・T2 が red |
| 同上（mutation） | 日報の個数の列が緩む | mutation | M2: `parse_optional_hundredths` の整数部の検査を同じ置換に戻す | T1・T2 が red |
| D-103 (1) 先頭のグループは 0 で始まらない | 共有の関数が緩む（両方が同じに緩むので T1 は等しく green） | mutation | M3: 共有の正規表現の `[1-9][0-9]{0,2}` を `[0-9]{1,3}` に | `test_parse_z004_req401_malformed_comma_invalid_number`（`0,123`）と T2（`0,123`）が red。T1 は green のまま（等価の差。T1 だけでは共有の規則そのものを守れないことの確かめ） |
| D-103 (1) `+` を拒む | 共有の関数が `+` を受ける | mutation | M4: 共有の正規表現の `-?` を `[+-]?` に | `test_parse_z004_req401_malformed_comma_invalid_number`（`+5`）と T2 が red |
| BIZ-08-D2 / D2a の順 | `invalid_number` の文が `number_too_large`・`settlement_mismatch` より先に選ばれる | mutation | M5: S4 の選ぶ順の `invalid_number` を `number_too_large` より前に | T3 の (ii) と `test_daily_report_req401_over_limit_names_column_without_raw_value` が red |
| A1 空の列は `None` | 空の判定が共有の関数の後に回り、空が `invalid_number` になる | mutation | M6: `parse_optional_i64` の空の早期 return を消す | T1 の A1 の行が red（既存の test のどれが空の列で red になるかは実装の段で数えて報告に書く。fixture の Z001 には空の値の列がある） |
| 今の挙動（受ける値と値） | 置き換えで既存の値・error が変わる | unit・integration（既存） | `cargo test` 全体（AC4） | 既存の Z004・日報・BIZ-08 の test の値と error が変わる |

状態の遷移: 読取りの段だけで、DB の書込み・再試行・再起動の遷移を持たない。negative は T1 の崩れた行、T2 の各列、T3 の (i)。

## L3 を置かない理由

`## L3` 節は置かない（manual なし）。利用者に見える変化は (i) 崩れた値の file を読んだとき手で選ぶ経路の文が汎用の文から帳票と列を名指す文に変わること、(ii) SD の一覧でその束が「読めません」になることだけで、どちらもレジの出す正しい日報（F4）では起きない。文の中身は T3 が BIZ の境界で見て、画面は `ImportError` の message をそのまま出す既存の経路（`number_too_large` の文で `DailyReportImportPage.flow.test.tsx` が固定）なので、DEV_WORKFLOW L3 の L3 Eligibility の (1)（Windows/Tauri native でしか観測できない）を満たさない。合成の崩れた file を予備の経路で読ませて文を目視する L3 を置くかは計画の review に問う。

## 計画の review に問う点

1. S4（`invalid_number` の文で帳票と列を名指す）を本 lane に入れるか。発注は「どの file・何行目・何列」を名指すとし、「既存の `invalid_number` の経路と文言の形」とするが、今の経路は利用者に汎用の文しか出さない（F6a、BIZ-08-D1）。本 packet は帳票（= file）と列だけを名指し、行番号は BIZ-08-D1 と IO-07-D6 の「行番号を入れない」に合わせて出さない案にした。行番号まで要るか、S4 を別の lane にするか。
2. `¥`・`￥`・数の中の空白・`+` を受けない判断の根拠（F4 の観測の不在）で足りるか。足りなければ、実装の前に repo 外の 2026-07-06 の要約か店の PC の実 file で、日報の数の列の値を本 lane の式で数える Contract Probe（数と位置だけを記録）を置く。
3. EJ（F7）を本 lane の外に置くことでよいか（別の lane の候補として backlog に書くか）。
4. L3 を置かないことでよいか（上の節）。
5. 共有の関数の置き場（案 (a) の `io/mod.rs`）と、T1 のために 3 関数を `pub(super)` にすることでよいか。
6. D-157 を新しい D にし、D-103 に `Superseded in part` を足さない扱いでよいか。
