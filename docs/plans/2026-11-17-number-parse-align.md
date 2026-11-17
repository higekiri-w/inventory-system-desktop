# Plan Packet: 日報（Z001・Z002・Z005）の数の読取りを Z004 と同じ厳しさにそろえる

wave に属さない単独の lane。出所は owner の決定 2026-11-17（repo 外の回答台帳 TD-775、原文「そろえるべきだと思う」）と、differential test を始める owner の決定（TD-770。本 lane が最初の当て所）。調べの起点は相談役（Fable）の code の読み（2026-11-17、実行は未確認）で、下の「調べ」で現物を読み直した。計画の review round 1（PR #274 の comment issuecomment-6945631254 の「裁定（Coordinator）」の 8 項目）で計画を直した。本 commit は Plan Packet だけで、`src-tauri/**`・`src/**` と設計正本（29・23・37・45・55・pos-tables・decision-log ほか）は計画の固定の後の実装の PR で変える（Design Readiness）。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R3

Reason:
POS CSV（日報 Z001・Z002・Z005）の受ける値の契約を変える（今まで受けていた区切りの崩れた値・`¥`・数の中の空白・`+` を `invalid_number` にする）。日報の取込み（BIZ-08）の利用者向けの文を 1 つ変え（S4）、SD 直読みの一覧の候補の DTO に field を 1 つ足して「読めません」の行に理由を出す（S5）。DEV_WORKFLOW Risk Tiers の R3「POS CSV」「Tauri command の DTO」「状態を持つ画面」「operator の業務の流れ」。R4 にしない理由: DB の schema・migration・書込み・取込み済みの行・backup に触れない（読取りの段で止まる値が増えるだけで、取込み済みのデータは読み直さない。下の「調べ」の F6）。取消の後に同じ原本を読み直せなくなる互換の限界（F6）はデータを消さない（取り消した行は `rolled_back` のまま残る）。

## Goal

Goal Invariant:

### 最小完了条件

- 日報の数の列（Z001 の第 3・4 列、Z002 の第 3・4 列、Z005 の第 3・4 列）は、Z004 と同じ 1 つの整数の形の規則（D-103 の (1)、23 §13.4.1: `-?[0-9]+` か `-?[1-9][0-9]{0,2}(,[0-9]{3})+`）で読む。前後の空白は呼ぶ側が値の全体に 1 回だけ trim し、規則の検査は trim しない。区切りの崩れた値（`0,123`・`1,2,3`・`1,23`・`1234,567`・`,123`・`123,`・`1,,234`）、`¥`・`￥` 付き、数の中の空白（`1 234`、個数の列の `1 .5`・`1\t.5`・`1　.5`・`1,234 .5`）、`+` 付きは `invalid_number` になり、別の数として読まれない。
- 日報だけの必要は「承認済みの差」として残る: (A1) 空の値は今どおり列ごとに `None`（Z005 の金額は必須で `invalid_number`）、(A2) 上限は今どおり IO-07-D6（金額・件数の列は絶対値 `9999999999`、個数の列は絶対値 `9999999.99`〈100 倍で `999999999`、`constants.rs:75` の `MAX_DAILY_REPORT_QUANTITY_HUNDREDTHS`〉。越えれば `number_too_large`。Z004 は `i32` なので、`2147483648` は Z004 が拒み日報が受け、`10000000` は Z004 が受け日報の個数の列が拒む、の両向きの差がある）、(A3) 個数の列（Z001 の総売の行の第 3 列、Z005 の第 3 列）は整数部に上の規則を当てた上で `.` + 1〜2 桁の小数を受ける（IO-07-D2。`1,234.5` → 123450）。
- 2 つの parser の数の読取りに同じ入力の表を通す差分の test が、行ごとの 3 つ（Z004・日報の整数・日報の個数）の正確な期待と違えば fail する（表に無い差、承認済みの差の消失、差の向きや error の種類の変化の全部）。
- 手で選ぶ経路（「ファイルを選んで取り込む」）で数の列が読めないとき、利用者向けの文がどの帳票（= どの file。束は帳票ごとに 1 本）のどの列が読めないかを名指し、この日の日報は読めないので管理者に知らせるよう伝えて止める（S4。値・file 名・行番号は出さない。BIZ-08-D1）。
- SD 直読みの一覧で、数の列の誤り（`invalid_number`・`number_too_large`）で「読めません」になった行は、行の下に帳票・列を名指して管理者に知らせるよう伝える文を出す（S5。値・file 名・行番号は出さない。D-127）。ほかの理由の「読めません」（大きすぎる file・形の外れ・精算回数の無い束・別の精算の混在）は今のまま文を出さない。
- 計画の固定の前の Contract Probe（AC9。手順と結果は PR #274 の comment）で、手元の実データの日報に新しく拒む値が 0。

### 失敗定義

- 区切りの崩れた値・`¥`・数の中の空白・`+` を、日報のどれかの数の列が数として受ける。
- 正しい 3 桁区切り（`1,234`・`-12,345`・`1,000,000`）、カンマなしの整数、前後の空白、個数の列の `1,234.5`・`1.3`・`-2` のどれかを、今と違う値で読むか拒む。
- A1〜A3 のどれかが変わる（空の列が error になる、上限の扱いが変わる、件数の列が小数を受ける）。
- 差分の test の承認済みの差の行に、理由・正本の ID の無い行が足される。
- `settlement_mismatch`・`number_too_large` の文の優先（BIZ-08-D2・D2a）が変わる。
- `number_too_large`・`invalid_number` 以外の `error_message` が利用者へ渡る（手で選ぶ経路の文と SD の一覧の文のどちらでも）。
- SD の一覧の文、手で選ぶ経路の文のどちらかに、値・file 名・行番号が出る。
- Contract Probe で新しく拒む値が 1 つでもあるのに計画を固定する。

### 非目的

- `split_csv_fields` の 3 つの写しの統合（別の lane）。
- EJ（IO-08）の金額 token の規則（29-io-ej-parser の「金額 token」）。EJ も桁区切りの位置を確かめない（下の「調べ」の F7）が本 lane では変えず、D-158 の Guarantee range に「EJ は対象外」と書き、backlog に調べの候補を 1 行足す（S6）。
- Z004 の数の規則の変更（23 §13.4.1 の受ける形・i32・小数を拒むことは変えない。共有の関数へ移すだけ）、Z004 と日報のほかの列の規則。
- 旧い緩い規則で受けた束を取り消した後の読み直しを通すこと（旧い緩い受理は戻さない。F6 の互換の限界）。
- SD の一覧の、数の誤り以外の「読めません」の理由（大きすぎる file・形・精算回数・別の精算の混在）を出すこと。
- UI の新しい画面、DB の変更。

## 調べ（2026-11-17、main `ddfd174`。code の読みと、tracked の合成 fixture への 1 回の計数。round 1 の是正で worktree `e8b40dec` を読み直した）

- F1 日報の数の読取り: `parse_optional_i64`（`src-tauri/src/io/daily_report_parser.rs:538`）と `parse_optional_hundredths`（:562）は先に `clean_number`（:620、`value.trim().replace([',', '￥', '¥', ' '], "")`）を通す。`parse_optional_i64` は `+`/`-` を 1 つ外して ASCII 数字だけを確かめ（:545〜:548）、`parse_optional_hundredths` は `-` だけを外す（:572）。`parse_required_i64`（:596）は空を `Invalid` にする。上限は `parse_optional_i64` が `MAX_DAILY_REPORT_INTEGER`（`constants.rs:72`、`9_999_999_999`。:555）、`parse_optional_hundredths` が `MAX_DAILY_REPORT_QUANTITY_HUNDREDTHS`（`constants.rs:75`、`999_999_999`。:590）で見る。呼出しは Z001 の :291・:293・:307、Z002 の :443・:455、Z005 の :490・:502 の 7 か所だけ。
- F2 Z004 の数の読取り: `parse_z004_int`（`src-tauri/src/io/z004_parser.rs:408`）は trim した値が `^-?(?:[0-9]+|[1-9][0-9]{0,2}(?:,[0-9]{3})+)$` に全体で一致するときだけ、カンマを除いて `i32` に読む（:409〜:417）。呼出しは :363（数量）・:372（金額）の 2 か所。test は `test_parse_z004_req401_comma_grouped_numbers`（:769）と `test_parse_z004_req401_malformed_comma_invalid_number`（:788、`1,23`・`12,3456`・`1,,234`・`,123`・`1,234,`・`0,123`・`1.5`・`+5`・`2,147,483,648` を拒む）。
- F3 正本の文: 29 IO-07-D2（`docs/function-design/29-io-daily-report-parser.md:87`）は「3桁区切りのカンマと空白は既存の `clean_number` で除いた後」と書く。D-103（`docs/decision-log.md:895`）の Alternatives（:900）と 23 §13.4.1 の却下案（`docs/function-design/23-io-z004-parser.md:235`）は、日報の `clean_number` と同じ緩い除去を「区切りの崩れた値を黙って別の数として読む」として Z004 で不採用にし、23 は「日報側の規則は IO-07 が持ち、本書は変えない」と書く。`docs/plu-export-and-real-csv-verification.md:120` は「`daily_report_parser`（IO-07）はクォート対応分割・カンマ/¥除去・NEL 正規化を実装済み」と書き、S3 の後に古くなる。
- F4 実データの形の記録: 2026-07-06 の実 Z001/Z002/Z004/Z005 の匿名化解析（`docs/plu-export-and-real-csv-verification.md:118`）は「全フィールドがダブルクォート囲み。Z001/Z002 の金額にはクォート内の千位区切りカンマ（`"##,###"`）が実在する」と書き、`¥`・数の中の空白・`+` の観測を書かない。同 :120 の「カンマ/¥除去を実装済み」は実装の記述で、`¥` の観測ではない。29 §29.4 の shape の表（:173〜:190）にも `¥`・空白の形は無い。観測の不在だけでは受けなくする根拠に足りない（round 1 の指摘 3）ので、手元の実データに当てた Contract Probe（AC9）で、新しく拒む値は 0 件だった。
- F5 tracked の合成 fixture（`tests/fixtures/daily-report/`・`daily-report-z001/`・`daily-report-sheet/` の Z001・Z002・Z005 の 21 本）: ヘッダの後の第 3・4 列の空でない値を、D-103 の (1) の整数部に任意の `.` + 1〜2 桁を足した式で数えた。測定の command: `python3 -I - tests/fixtures/daily-report*/Z00[125]*.CSV`（CP932 で decode、`csv.reader`、ヘッダ `レコード` の後の 4 列の行の第 3・4 列）。出力: `values 556 with_comma 0 rejected_by_strict 0`。→ 新しい規則で既存の fixture の bundle の test の値は変わらない見込み（カンマ付きの値は fixture に無いので、正しい 3 桁区切りの受理は T2 で別に確かめる）。daily の unit test の値で崩れた区切り・`¥`・`+`・数の中の空白を使うものは無い（`rg '[0-9],[0-9]|¥|￥|"\+[0-9]'` で `daily_report_parser.rs:1062` の `"1,234.5"` だけ。新しい規則でも受ける）。
- F6 取込み済みのデータへの影響: `parse_daily_report_bundle` の呼出しは `biz/daily_report_import_service/parse.rs:69`（手で選ぶ経路と SD の候補の preview）と `scan.rs:251`（SD の一覧）の 2 か所だけで、どちらも取込みの前の読取り。取込み済みの行（`daily_report_imports` と子の行）は数を保存した後で、読み直す経路は無い（取消は import の id で、bytes を読まない）。影響は 2 つ: (i) 崩れた値を含む束を前に取り込んでいた場合、SD の一覧ではその束が「取込み済み」から「読めません」（S5 で帳票・列を名指す文つき）に変わる（`scan.rs:259〜:262` で `parse_errors` があれば bundle_hash を作る前に `Unreadable`）。(ii) 互換の限界: その束を取り消すと、同じ原本は新しい規則で拒まれ、読み直せない。`docs/db-design/pos-tables.md:155`（§12b）の「`rolled_back` の同一bundleは再取込み可能」は重複の照合の規則で、読取りで止まる束には当たらない。取り消した行は `rolled_back` のまま残り、データは消えない。取消を戻す操作は無い（§12b :149、取消は status の更新だけ）。店の手で直す道は無い: SD の `XZ_BKUP` と PC の `EcrDatas` の写しは同じ bytes（29 §29.7.2 の SD-22、`docs/function-design/29-io-daily-report-parser.md:244`）で同じ parser が拒み、Z の file は精算のその時に SD へ書き出される結果でもう一度書き出せない見込み（repo 外の回答台帳 TD-781、未確認）。そこで手で選ぶ経路と SD の一覧の文（S4・S5）は帳票と列を名指したうえで、この日の日報は読めないので管理者に知らせるよう伝えて止める。旧い緩い受理は戻さない（D-127 の「黙って通さない」と TD-775）。TD-781 は未確認だが、4 年分の手元の実データで新しく拒む値は 0 件（AC9）なので、この止まり方を受容する。
- F6a SD 直読みと予備の経路: 2 つの経路は同じ `parse_daily_report_bundle` を通るので、数の規則は 1 か所の置き換えで両方にそろう。崩れた値の束は、SD の一覧では「読めません」（37 §37.9 手順 5〈:436〉「利用者向けには出さない」。`scan.rs:259〜:262` が `parse_errors` を `log_parse_diagnostics` に渡した後 `fallback(Unreadable)` に畳み、wire の `DailyReportSdCandidate`〈`biz/daily_report_import_service/mod.rs:184〜:192`、`src/lib/bindings.ts:691〜:699`〉に理由の field は無い。画面の `SdCandidateRow`〈`src/features/daily-report-import/DailyReportImportPage.tsx:342`〉は `CANDIDATE_STATUS`〈:149〜:174〉の label と icon だけを出す）、手で選ぶ経路では今は汎用の文「日報ファイルの解析に失敗しました」（`parse.rs:82`、BIZ-08-D1。`test_daily_report_req401_parse_error_logs_parse_failed`〈`biz/daily_report_import_service/tests.rs:147`、入力は Z001 の個数の `not-a-number` で `invalid_number`〉が固定）で、帳票も列も名指さない。帳票と列を名指すのは `number_too_large` だけ（BIZ-08-D2a、`parse.rs:357` の `number_too_large_message`）。IO の `invalid_number` の文（`daily_report_parser.rs:601` の `number_error`、`{帳票}の{列}列を変換できません`）は `number_too_large` の文と同じく帳票と列の語だけで作られ、値・file 名・行番号を含まない。
- F7 EJ: `amount_token`（`src-tauri/src/io/ej_parser.rs:1356`）は桁区切りを「数字の後ならどこでも」受け（:1398）、`1,2,3` を 123、`0,123` を 123 と読む（29-io-ej-parser「金額 token」の文のとおり）。EJ は印字の text で、全角の数字・`，`・`￥` の観測された形を持ち、読めない行は記録を `Unknown` にする別の契約なので、本 lane に入れない（非目的。backlog の調べの候補、S6）。

### 差分の test（T1）の表: 入力ごとの 3 つの正確な期待

`v` は `Ok(Some(v))`（Z004 は `Some(v)`）、`空` は `Ok(None)`、`inv` は `Invalid`（Z004 は `None`）、`big` は `TooLarge`。「今」は code の読み（F1・F2）、「後」は T1 の期待。Z004 は今と後で同じ。差の列が空の行は、3 つの結果を個数の ÷100 でそろえると等しい行。

| 入力（trim の前） | Z004 | 日報の整数 今 | 日報の個数 今 | 日報の整数 後 | 日報の個数 後 | 承認済みの差 |
|---|---|---|---|---|---|---|
| `1234` / ` 1234 ` | 1234 | 1234 | 123400 | 1234 | 123400 | |
| `1,234` / `-12,345` / `1,000,000` | v | v | v×100 | v | v×100 | |
| `0123` / `-0` | 123 / 0 | 123 / 0 | 12300 / 0 | 123 / 0 | 12300 / 0 | |
| `0,123` | inv | 123 | 12300 | inv | inv | |
| `1,2,3` / `1,23` / `1,,234` | inv | 123 / 123 / 1234 | ×100 | inv | inv | |
| `1234,567` / `12,3456` | inv | 1234567 / 123456 | ×100 | inv | inv | |
| `,123` / `123,` / `1,234,` | inv | 123 / 123 / 1234 | ×100 | inv | inv | |
| `¥1,234` / `￥1234` | inv | 1234 | 123400 | inv | inv | |
| `1 234`（数の中の ASCII 空白） | inv | 1234 | 123400 | inv | inv | |
| `1　234`（全角空白） | inv | inv | inv | inv | inv | |
| `+5` | inv | 5 | inv | inv | inv | |
| `１２３`（全角数字） | inv | inv | inv | inv | inv | |
| `1.5` / `1,234.5` | inv | inv | 150 / 123450 | inv | 150 / 123450 | A3 |
| `1 .5` / `1,234 .5`（小数点の前の ASCII 空白） | inv | inv | 150 / 123450 | inv | inv | |
| `1\t.5` / `1　.5`（小数点の前の tab・全角空白） | inv | inv | inv | inv | inv | |
| `1.234` / `.5` / `1.` | inv | inv | inv | inv | inv | |
| `9999999.99` | inv | inv | 999999999 | inv | 999999999 | A3（個数の上限の端） |
| 空・空白だけ | inv | 空 | 空 | 空 | 空 | A1（Z005 の金額は `parse_required_i64` で inv） |
| `-` | inv | inv | inv | inv | inv | |
| `10000000` | 10000000 | 10000000 | big | 10000000 | big | A2（逆向き: Z004 が受け、個数の列が拒む） |
| `2147483648` | inv（i32 の外） | 2147483648 | big | 2147483648 | big | A2 |
| `10000000000` | inv | big | big | big | big | A2 |
| `99999999999999999999` | inv | big | big | big | big | A2 |
| `9a` / `abc` | inv | inv | inv | inv | inv | |

確かめ: 個数の上限は `constants.rs:75`（`999_999_999`）と 29 IO-07-D6（`docs/function-design/29-io-daily-report-parser.md:101`、「個数の列は絶対値 `999999999`（`9999999.99`）以下」）。`1 .5` が今 150 になるのは `clean_number`（:620）が ASCII 空白を除くため、`1\t.5`・`1　.5` が今 `inv` なのは `clean_number` が tab・全角空白を除かず、`:577` の数字の検査で外れるため。

## 案の比較

- (a) 採る: D-103 の (1) の整数の形の検査を `src-tauri/src/io/mod.rs` の 1 つの `pub(crate)` の関数に移し、Z004 と日報の 2 関数がそれを通る。`io/mod.rs` は IO の module どうしが共有する唯一の定義を置く既存の場所（`REGISTER_SD_FOLDER`、`io/mod.rs:17`）。新しい file・依存を足さない（`regex` と `LazyLock` は `z004_parser.rs` が既に使う）。
- (b) 不採用: 日報が `z004_parser::parse_z004_int` を呼ぶ。`i32` で日報の金額（上限 `9999999999`）が入らず、形の誤りと上限の越え（IO-07-D6 の `invalid_number` と `number_too_large`）を分けられない。
- (c) 不採用: 日報に同じ正規表現を写す。2 つの写しがずれる（今の食い違いの原因と同じ形）。
- (d) 不採用: 共有の関数を `z004_parser.rs` に置いて日報から呼ぶ。大きさは (a) と同じだが、日報の規則が Z004 の module に依る形になり、読み手が「Z004 だけの規則」と読む。
- SD の一覧の理由（S5）: (e) 採る: 候補の DTO に `unreadable_reason: Option<String>` を足し、BIZ が S4 と同じ関数で列の文を作り、画面はその文をそのまま出す（今の `ImportError` の message をそのまま出す経路と同じ形。画面は文字列で分岐しない）。(f) 不採用: `DailyReportSdCandidateStatus` に数の誤りの値を足す（帳票・列の組の数だけ値か、別の構造化した field が要り、画面が文を組む。文の正本が BIZ と画面の 2 か所になる）。

## Scope

本 commit（plan-first）:

- 本 packet `docs/plans/2026-11-17-number-parse-align.md` だけ。

実装（計画の固定の後）:

- S1 `src-tauri/src/io/mod.rs`: `pub(crate) fn strip_thousands_grouping(value: &str) -> Option<String>` を足す。受けた値を trim せずに `^-?(?:[0-9]+|[1-9][0-9]{0,2}(?:,[0-9]{3})+)$` に全体で当て、一致すればカンマを除いた文字列（符号を含む）を、外れれば `None` を返す（D-103 の (1)、23 §13.4.1）。trim は呼ぶ側が値の全体に 1 回だけ行う（S2・S3。整数部だけを渡す S3 の個数の列で、数の中の空白を受けないため）。正規表現は `LazyLock` で 1 回だけ作る（`z004_parser.rs:409` の今の形）。差分の test（T1）をこの file の `#[cfg(test)] mod tests` に置く。
- S2 `src-tauri/src/io/z004_parser.rs`: `parse_z004_int` の本体を `crate::io::strip_thousands_grouping(raw.trim())?.parse().ok()` にし、自前の正規表現を消す。受ける形・`i32`・error の文は変えない。T1 から呼ぶため `pub(super)` にする。
- S3 `src-tauri/src/io/daily_report_parser.rs`:
  - `clean_number` を消す。
  - `parse_optional_i64`: 値を 1 回 trim し、空なら `Ok(None)`（A1）。trim した値を `strip_thousands_grouping` に渡し、`None` なら `Err(NumberError::Invalid)`。その後の `parse::<i64>` の桁あふれを `TooLarge` にする分岐と上限の検査（IO-07-D6）は今のまま（形を先に見るので、`test_parse_daily_report_req401_overflow_digits_with_bad_tail_is_invalid_number` の順序の保証も今のまま）。`+`/`-` を外して ASCII 数字を確かめる今の検査（:545〜:548）は共有の関数が持つので消す。
  - `parse_optional_hundredths`: 値を 1 回 trim し、空なら `Ok(None)`。trim した値を `split_once('.')` で整数部と小数部に分け、整数部を trim せずに `strip_thousands_grouping` に通す（`None` なら `Invalid`。`1 .5`・`1\t.5`・`1　.5`・`1,234 .5` の整数部は空白で終わるので `Invalid`）。小数部が 1〜2 桁の ASCII 数字でなければ `Invalid`（今の規則）。100 倍・上限の計算は今のまま。
  - 2 つの関数を T1 から呼ぶため `pub(super)` にする（`NumberError` も同じ）。
  - T2（下の Test Plan）を `mod tests` に足す。
- S4 `src-tauri/src/biz/daily_report_import_service/parse.rs`: `number_too_large_message`（:357）を、列の文（`{列の文}`。BIZ-08-D2a のつなぎ方）だけを返す `pub(super) fn number_column_text(errors) -> Option<String>` に置き換える。`number_too_large` が 1 件でもあればその誤りの `error_message` だけで、無ければ `invalid_number` の誤りの `error_message` だけで作る（並びと重複の除き方は今の BIZ-08-D2a のまま）。手で選ぶ経路の文は `settlement_mismatch` → `number_column_text` が `Some`（`number_too_large` か `invalid_number` がある）なら `number_column_text` + `この日の日報は読めません。管理者に知らせてください。`（例 `{帳票}の{列}列を変換できません。この日の日報は読めません。管理者に知らせてください。`。BIZ-08-D2a の文の優先と列の文のつなぎ方は変えず、`number_too_large` の文の後半も同じ文に置き換える。言い方は app の既存の文〈`src-tauri/src/db/schema_v13.rs:16` の「管理者に知らせてください」〉に合わせる。F6 のとおり、数が大きすぎる束も含めて店の手で直す道が無い。Coordinator の裁定 2026-11-17）→ 汎用の文、の順で選ぶ。BIZ は `number_too_large` と `invalid_number` 以外の `error_message` を利用者へ渡さない（`else` の枝は汎用の文だけ）。`test_daily_report_req401_parse_error_logs_parse_failed`（tests.rs:147）は入力を数でない誤りに変える: Z001 の 9 行目を 3 列のデータ行にした束（`invalid_format`、`日報CSVのデータ行が4列ではありません`。`daily_report_parser.rs:351〜:358`）。利用者向けの文と `operation_logs.summary` の汎用の文の期待と、診断の WARN の assert（`source_file`・`filename`・`line_no=Some(9)`・`error_type`・`error_message`）は残し、`error_type`・`error_message` の期待の値だけを入力に合わせる（S4 の後は `invalid_number` の入力で汎用の文が出なくなり、汎用の文の経路の test が無くなるため。round 1 の指摘 6）。T3・T4 を足す。
- S5 SD の一覧の理由（`invalid_number`・`number_too_large` の束だけ）:
  - `src-tauri/src/biz/daily_report_import_service/mod.rs`: `DailyReportSdCandidate`（:184）に `pub unreadable_reason: Option<String>` を足す（doc comment: 数の列の誤りで `Unreadable` の束だけ `Some`。値・file 名・行番号を含まない）。
  - `src-tauri/src/biz/daily_report_import_service/scan.rs`: `fallback`（:207〜:220）の `DailyReportSdCandidate` と、読めた束の `DailyReportSdCandidate`（:295）は `unreadable_reason: None`。`parse_errors` の枝（:259〜:262）だけ、`settlement_mismatch` が無ければ `number_column_text(&parsed.parse_errors)` に S4 と同じ文 `この日の日報は読めません。管理者に知らせてください。` を足したものを `Some` にする（`settlement_mismatch` があるか、`number_column_text` が `None` なら `None`。今の `Unreadable` の他の理由〈:235 の大きすぎる file、:242 の形、:285 の精算回数の無い束〉と `log_parse_diagnostics` は変えない）。T5 を `tests.rs` に足す。
  - `src/lib/bindings.ts`: `cd src-tauri && cargo run --bin generate_bindings` で再生成（`DailyReportSdCandidate` に `unreadable_reason: string | null` が出る）。手で直さない。command の登録は変えない（型は既に `DailyReportSdScan` から引かれる。45 §45.8〈`docs/function-design/45-cmd-daily-report-import.md:250`〉）。
  - `src/features/daily-report-import/DailyReportImportPage.tsx`: `SdCandidateRow`（:342）で、`candidate.unreadable_reason !== null` のとき行の下にその文を出す（`sameSettlement` の案内〈:379〜:391〉と同じ置き方。文字列で分岐しない）。`DailyReportImportPage.sd.test.tsx` の `candidate()`（:53〜:66）と `src/features/daily-report-import/reducer.test.ts` の 2 つの候補の object literal（:102・:110）に `unreadable_reason: null` を足し（型に合わせる fixture の直しで、期待は変えない）、T6 を足す。
- S6 設計正本（実装の PR の中で、code と同じ PR）:
  - `docs/function-design/29-io-daily-report-parser.md`: IO-07-D2 の「（3桁区切りのカンマと空白は既存の `clean_number` で除いた後）」を「整数部は Z004 と同じ整数の形の規則（23 §13.4.1、D-103 の (1)、D-158。29 の中では 23 への link にする。前後の空白は値の全体に 1 回だけ除き、整数部の中の空白は受けない）」に置き換える。IO-07 の数の読取りの規則（金額・件数の列も同じ整数の形の規則。区切りの崩れた値・`¥`・数の中の空白・`+` は `invalid_number`。承認済みの差 A1〜A3）を IO-07-D2 の直前か IO-07-D6 の並びに置く（言葉を足す前に、今の IO-07-D2・D6 の文の置き換えで足りるかを先に見る）。F6 の互換の限界（旧い規則で受けた束を取り消すと同じ原本は読み直せない。写しは同じ bytes〈SD-22〉で店の手で直す道は無く、文は帳票・列を名指して管理者に知らせるよう伝えて止める）を同じ所に 1 文。§29.5 の `invalid_number` の行の発生条件に「整数の形（D-158）でない」を入れる。更新履歴に 1 行。
  - `docs/function-design/23-io-z004-parser.md` §13.4.1（:235）: 却下案の文を、緩い除去の方式そのものの説明（「カンマ・円記号・空白をすべて除いてから読む緩い除去」）に置き換え、削った後の `clean_number` を今の関数として指さない。却下の理由の文（`1,23` のような区切りの崩れた値を黙って別の数として読む）は残す。「日報側の規則は IO-07 が持ち、本書は変えない」を「日報（IO-07）も同じ規則で読む（D-158。検査は `io/mod.rs` の 1 か所）」に置き換える。
  - `docs/function-design/37-biz-daily-report-import-service.md`: BIZ-08-D2a（:225）の文の後半（`この日報ファイルは取り込めません。` から後）と例を `この日の日報は読めません。管理者に知らせてください。` に置き換え、`number_too_large` が無く `invalid_number` があるときの文（同じつなぎ方の列の文〈IO-07 の `…列を変換できません`〉+ 同じ後半）を足し、直し方を名指す理由の文を「写しは同じ bytes〈SD-22〉で店の手で直す道が無い」に置き換え、「BIZ は `number_too_large` 以外の `error_message` を利用者へ渡さない」を `invalid_number` を含む形に直す。§37.7 の表の `number_too_large` の行の対処を「管理者に知らせる〈写しは同じ bytes で、店の手で直す道は無い〉」に置き換え、同じ対処の `invalid_number` の行を足す（2 つを 1 行にまとめてよい）。§37.9 手順 5（:436）の「利用者向けには出さない」を「数の列の誤り（`invalid_number`・`number_too_large`、`settlement_mismatch` の無い束）は `unreadable_reason` に BIZ-08-D2a の列の文と `この日の日報は読めません。管理者に知らせてください。` を入れる。ほかの parse の誤りは出さない」に置き換え、§37.9 の `DailyReportSdCandidate`（:406〜:413）に `unreadable_reason` の行を足す。更新履歴に 1 行。
  - `docs/function-design/45-cmd-daily-report-import.md`: :156 と :250 の `DailyReportSdCandidate` の field の記述に `unreadable_reason`（37 §37.9。CMD は素通し）を足す。
  - `docs/function-design/55-ui-csv-import.md` UI-07-D15 (3)（:55）: 「読めません」の行は、候補の `unreadable_reason` があれば行の下にその文を出す（文字列で分岐しない）を足す。
  - `docs/db-design/pos-tables.md` §12b（:155）: 「`rolled_back` の同一bundleは再取込み可能」に「（重複の照合では止めない。読取りの規則〈29 IO-07、D-158〉で読めない束は読み直せない）」を足す。
  - `docs/plu-export-and-real-csv-verification.md:120`: 「カンマ/¥除去」を「3 桁区切りのカンマ付きの整数の読取り（Z004 と同じ規則、D-158）」に置き換える。
  - `docs/decision-log.md`: D-158 を足す（Decision: D-103 の (1) を日報の数の列へ広げ、trim は値の全体に 1 回、承認済みの差 A1〜A3 を残す。手で選ぶ経路の `invalid_number` の文と、SD の一覧の `unreadable_reason`。Why: TD-775、区切りの崩れた値を黙って別の数として読まない、D-127。Alternatives: 上の案 (b)〜(d)・(f)、`¥`・空白を受け続ける〈Contract Probe で新しく拒む値が 0〉。Guarantee range: F6 の SD の一覧の表示の変化と、取消の後の読み直しの互換の限界。EJ は対象外。Compatibility: DB・Z004 の受ける形は変えない。DTO は `DailyReportSdCandidate` に `Option` の field を 1 つ足すだけ（bindings は `string | null`））。D-103 は今も有効なので `Superseded in part` の行は足さない（D-103 の (1) を置き換えず、範囲を広げる。decision-log の冒頭の規則〈:12〉の「一部が置き換わったとき」に当たらない）。
  - `docs/backlog.md` の「保留」（:161）: EJ の `amount_token`（`ej_parser.rs:1356`）の桁区切りの緩さを調べる候補を 1 行（出所: PR #274 の計画の review round 1、AI の review 由来で owner の確認の前に Human Gate の必須にしない）。
  - `docs/function-design/90-traceability.md`: `cd src-tauri && cargo run --bin generate_traceability` で再生成（REQ-401 の test が増える）。手で直さない。

Non-scope: `split_csv_fields`、EJ（`ej_parser.rs`）、Z004 の受ける形と `ParsedRow` の型、`DailyReportSdCandidateStatus` の値、command の登録、DB の schema・migration、取込み済みのデータ、tracked の fixture（F5 で値が変わらない）。

他の lane との重なり: 本起票では `gh pr list` を引けていない（sandbox の network の制限）。実装の発注の前に Coordinator が open の PR の `gh pr diff N --name-only` と `@@` の行で、Scope の file の hunk の重なりを確かめる。

## Acceptance Criteria

- AC1（差分の test、red から）: 実装の手順は (i) S2・S3 の `pub(super)` だけを入れ、T1 を足して `cd src-tauri && cargo test --lib io::tests` が red（上の T1 の表の「今」と「後」が違う行が、期待と違う行として 1 回の実行で全部出る）、(ii) S1〜S3 の置き換えで T1 が green。(i) の red の出力（期待と違う行の一覧）を実装の報告と PR に貼る。これが T1 の表の「今」の列の実測になる。
- AC2（日報の経路）: T2 が green（3 帳票の全部の数の列で、崩れた区切り・`¥`・数の中の空白・`+` が `invalid_number`、正しい 3 桁区切りと `1,234.5` の個数を今の値で読む）。
- AC3（利用者向けの文）: T3〜T6 が green。`settlement_mismatch` と `number_too_large` の既存の test は green。`number_too_large` の文の後半の変更（S4）に合わせて、`test_daily_report_req401_over_limit_names_column_without_raw_value`（`OVER_LIMIT_Z001_Z005_MESSAGE`、tests.rs:2457）・`test_daily_report_req401_settlement_mismatch_wins_over_over_limit`（tests.rs:2548）・`src/features/daily-report-import/DailyReportImportPage.flow.test.tsx:143` の期待の文だけを 37 BIZ-08-D2a の新しい文に直す（assertion の種類と強さは変えない）。ほかの `settlement_mismatch` と `number_too_large` の test は変えない。直した `test_daily_report_req401_parse_error_logs_parse_failed` が汎用の文で green。
- AC4（今の挙動）: `cd src-tauri && cargo test` と `npm test` が全部 green。S4 の 1 本の入力の直しと、AC3 の 3 つの期待の文の直しと、S5 の `candidate()` と `reducer.test.ts` の候補の field の足し（既存の assertion は変えない）のほかに、既存の test を変えない・skip しない・弱めない（Z004 の `test_parse_z004_req401_*`、日報の `test_parse_daily_report_req401_*`、`DailyReportImportPage.sd.test.tsx` の既存の期待を含む）。
- AC5（mutation、Test Plan）: 実装の後の版に M1〜M10 を 1 つずつ当てて、Rust の mutant は `cd src-tauri && cargo test --lib`、M9 は `npm test -- src/features/daily-report-import` を回し、Test Plan の「Would fail if...」の列の test が red。結果を実装の報告と PR に書き、mutant は戻す。
- AC6（gate）: `cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings`、`cd src-tauri && cargo run --bin generate_traceability -- --check`、`npm run typecheck && npm run lint && npm run format:check` が exit 0。
- AC7（docs）: S6 の正本の直しが Scope のとおり。`bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-11-17-number-parse-align.md` が ERROR なし。
- AC8（範囲）: `git diff origin/main --name-only` が Scope の file だけ。`cd src-tauri && cargo run --bin generate_bindings` の後の `src/lib/bindings.ts` の差分が `DailyReportSdCandidate` の `unreadable_reason` の 1 field（と生成の comment）だけ。
- AC9（Contract Probe）: 計画の固定の前に Coordinator が手元の実データ（レジの SD の写しの Z001・Z002・Z005）に今の規則と新しい規則を当て（`,`・`¥` 等を消すと空になる非空の値も今受ける値に数える）、帳票・列ごとの件数と新しく拒んだ値の位置だけを PR #274 の comment に置く（値・file 名は残さない。repo に入れない。DEV_WORKFLOW の Plan Packet Rules のとおり packet の節にしない）。新しく拒む値が 0。観測: `python3 -I .local/field-data/recount/probe-274-number-format.py <SD の写しの root>` の `new_reject_total 0`（結果は PR #274 の comment issuecomment-6946069570・issuecomment-6946307519）。

## Design Readiness

- 引用する設計正本（節まで）: `docs/function-design/29-io-daily-report-parser.md` IO-07-D2・IO-07-D6・§29.4・§29.5、`docs/function-design/23-io-z004-parser.md` §13.4.1（SPEC-Z4A-D7）、`docs/function-design/37-biz-daily-report-import-service.md` BIZ-08-D1・D2・D2a・§37.7・§37.9 手順 5、`docs/function-design/45-cmd-daily-report-import.md` §45.6a・§45.8、`docs/function-design/55-ui-csv-import.md` UI-07-D15、`docs/db-design/pos-tables.md` §12b、`docs/decision-log.md` D-103・D-104・D-142、`docs/spec/requirements.md` REQ-401、AGENTS.md の「使い手に押しつけない（D-127）」。
- 必要な設計成果物: updated in the implementation PR（S6）。本 commit に正本の直しを置かないのは、S4・S5 の文の採否・`¥` を受けない判断・A1〜A3 の表を計画の review と Contract Probe で確かめてから正本に書くため（2026-11-13-z001-row-guard と同じ置き方）。plan にしかない durable な判断（1 つの整数の形の規則・trim の置き場・承認済みの差・S4 と S5 の文・取消の後の互換の限界）は実装の PR で 29・23・37・45・55・pos-tables・D-158 へ昇格する。
- 前提・制約と延期した design gap: (i) `¥`・空白・`+` を受けない根拠は、F4 の観測の不在に Contract Probe（AC9）の実測を足したもの。(ii) S4・S5 の文は行番号を出さない（BIZ-08-D1 と IO-07-D6 の「値・file 名・行番号を入れない」）。(iii) S4・S5 の文は直し方を名指さず、帳票・列を名指して「この日の日報は読めません。管理者に知らせてください。」で止める（F6。写しは同じ bytes〈SD-22〉、もう一度書き出せない見込み〈TD-781、未確認〉。受容の根拠は AC9 の 0 件）。
- 絶対保証の例外と抜け道: 「区切りの崩れた値を数として読まない」は日報の 3 帳票の 7 か所の呼出しが全部 S3 の 2 関数を通ること（F1 の呼出しの一覧）と T2 で見る。EJ は抜け道として残る（F7、非目的、D-158 の Guarantee range）。
- 計画の review の固定の問いへの先回りの答え:
  - (1) 普通の一日: 店主が SD から読む・手で選ぶ、どちらでもレジの出した日報（正しい 3 桁区切りの金額）は今と同じ値で取り込まれる（F4・F5、Contract Probe、T2 の正しい区切りの行、既存の fixture の bundle の test）。SD の一覧の行の見た目は、数の誤りの「読めません」の行のほかは変わらない。
  - (2) 登録と生成: command・route は変えない。`DailyReportSdCandidate` に field を足すので `src/lib/bindings.ts` を再生成する（S5、AC8）。REQ-401 の test が増え、`90-traceability.md` を再生成する。画面の到達は既存の SD の一覧で、T6 が行に文が出ることを見る。
  - (3) 外の前提: 外の library・機器を足さない。`regex` は入れてある依存。レジの日報の数の形は F4 の記録と Contract Probe に依る。
  - (4) データの安全: test は合成の data だけ。実 POS file・実 DB を読まない・commit しない（Contract Probe は Coordinator が手元で行い、件数だけを PR の comment に書く）。取込み済みのデータは読み直さない（F6）。
  - (5) 前提の出所: lane と「そろえる」は owner の決定（TD-775）、differential test は TD-770。案 (a)・(e) の選択・A1〜A3 の線引き・S4 と S5 の文は起票の Writer（AI）の推奨で、owner の決定ではない（S4 は D-127 と発注の「どの file・何行目・何列」から。行番号は BIZ-08-D1 と食い違うので出さない案にした）。S5 と取消の後の互換の限界は計画の review round 1（Codex・Opus、AI の review）の指摘を Coordinator が受理したもの。S4・S5 の「管理者に知らせてください」で止める文は closure round 2 の裁定（PR #274 の comment issuecomment-6947161892）で、`number_too_large` の後半を同じ文にそろえたのは Coordinator の裁定（2026-11-17）。店の事実は TD-781（未確認）。
- 判定: ready（Contract Probe は済み、新しく拒む 0〈AC9、PR #274 の comment〉。計画の review の問いは下の節）。

この変更で不要になるもの（AGENTS.md「継ぎ足さない」）: `clean_number` と、`parse_optional_i64` の自前の符号と数字の検査（S3）、`parse_z004_int` の自前の正規表現（S2）、`number_too_large_message`（S4 の `number_column_text` に置き換え）、BIZ-08-D2a の文の後半と §37.7 の `number_too_large` の対処（S6 で管理者に知らせる文に置き換え）、29 IO-07-D2 の `clean_number` の括弧書き、23 §13.4.1 の「日報側の規則は IO-07 が持ち、本書は変えない」と `clean_number` を指す却下案の文、37 §37.9 手順 5 の「利用者向けには出さない」、`plu-export-and-real-csv-verification.md:120` の「カンマ/¥除去」（S6）。足すのは共有の 1 関数、DTO の 1 field、画面の 1 つの段落と test だけ。

## Contract Ledger

隣の契約の sweep: 日報の 7 か所の数の読取り（F1）、IO-07-D2（個数の小数）、IO-07-D6（上限と誤りの種類の分け方）、§29.5 の error の表、BIZ-08-D1・D2・D2a（利用者向けの文の選び方）、§37.9 手順 5（SD の一覧の `Unreadable` と理由）、45 §45.6a・§45.8（SD の候補の wire と bindings）、55 UI-07-D15 (3)（一覧の行）、pos-tables §12b（取消の後の再取込み）、23 §13.4.1（Z004 の受ける形）。

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| D-158（新）/ D-103 (1) 日報の数の列も同じ整数の形の規則、trim は値の全体に 1 回 | 29 IO-07-D2・§29.5、23 §13.4.1 | S1・S3 | T1、T2 | 非対象（下の「L3 を置かない理由」） |
| SPEC-Z4A-D7 Z004 の受ける形は変えない | 23 §13.4.1 | S2（共有の関数へ移すだけ） | `test_parse_z004_req401_comma_grouped_numbers`・`test_parse_z004_req401_malformed_comma_invalid_number`（変えない）、T1 | 非対象 |
| IO-07-D2 個数の列の小数 2 桁（A3） | 29 IO-07-D2 | S3（整数部だけを共有の関数へ） | `test_parse_daily_report_req401_decimal_quantity_hundredths`・`..._decimal_quantity_rejects_invalid_shapes`・`..._count_column_stays_integer`（変えない）、T1 の A3 の行 | 非対象 |
| IO-07-D6 上限と `invalid_number`/`number_too_large` の分け方（A2、両向きの差） | 29 IO-07-D6 | S3（形の検査の後に今の桁あふれと上限の検査） | `..._accepts_limit_values`・`..._rejects_over_limit_with_column`・`..._overflow_digits_with_bad_tail_is_invalid_number`（変えない）、T1 の A2 の行 | 非対象 |
| 29 §29.4 空の列は `None`、Z005 の金額は必須（A1） | 29 §29.4 の Z001・Z002・Z005 の行 | S3（空の判定を共有の関数より前に置く） | T1 の A1 の行、既存の日報の test | 非対象 |
| BIZ-08-D2a 数の列の誤りは帳票と列を名指し、管理者に知らせるよう伝える（`invalid_number` へ広げ、`number_too_large` の後半も同じ文） | 37 BIZ-08-D1・D2a・§37.7 | S4 | T3、`test_daily_report_req401_over_limit_names_column_without_raw_value`・`test_daily_report_req401_settlement_mismatch_wins_over_over_limit`（期待の文の後半だけを直す。AC3） | 非対象（文の中身は BIZ の test が見る。画面は message をそのまま出す既存の経路で、`src/features/daily-report-import/DailyReportImportPage.flow.test.tsx:143` が `number_too_large` の文で同じ経路を固定。期待の文だけを直す。AC3） |
| BIZ-08-D1 `number_too_large`・`invalid_number` 以外の `error_message` を利用者へ渡さない | 37 BIZ-08-D1 | S4（`else` の枝は汎用の文） | `test_daily_report_req401_parse_error_logs_parse_failed`（入力を `invalid_format` に直し、汎用の文と WARN の assert を残す）、M7 | 非対象 |
| BIZ-08-D2 `settlement_mismatch` の文が最優先 | 37 BIZ-08-D2 | S4 で順を変えない | 既存の `settlement_mismatch` の test（変えない） | 非対象 |
| §37.9 手順 5 SD の一覧は parse の誤りで `Unreadable`、数の列の誤りは `unreadable_reason` で帳票・列を名指し、管理者に知らせるよう伝える | 37 §37.9、45 §45.6a・§45.8、55 UI-07-D15 (3) | S5（DTO・scan・bindings の再生成・`SdCandidateRow`） | T5、T6、既存の scan の test（変えない）、M8、M9 | 非対象（文の中身は T5、行に出ることは T6） |
| 取消の後の互換の限界（旧い規則で受けた束は取消の後に読み直せない。店の手で直す道は無く、文は帳票・列を名指して管理者に知らせるよう伝える） | 29 IO-07（S6 の 1 文）、pos-tables §12b、D-158 Guarantee range | S6（正本の文だけ。旧い緩い受理は戻さない） | T4 | 非対象 |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| D-158 / D-103 (1): Z004 と日報の数の読取りは、行ごとの正確な期待のとおりで、差は承認済みの差 A1〜A3 だけ | 片方だけ緩い・違う値で読む（今の `clean_number`）、承認済みの差が消える・向きや error の種類が変わる | differential（unit、`io/mod.rs` の `mod tests`。`parse_z004_int`・`parse_optional_i64`・`parse_optional_hundredths` に同じ入力の表を通す） | T1 `test_number_parse_req401_daily_matches_z004_except_approved_differences`: 入力は上の「T1 の表」の全行（`1 .5`・`1\t.5`・`1　.5`・`1,234 .5` を含む）。行ごとに 3 つの正確な期待（`Ok(値)`・`Ok(None)`・`Invalid`・`TooLarge`。Z004 の `None` は `Invalid`）を持ち、3 つの結果のどれか 1 つでも期待と違えば fail（表に無い差・承認済みの差の消失・向きや error の種類の変化の全部）。加えて、個数の ÷100 でそろえた 3 つが違う行は承認済みの差の欄（A1/A2/A3・理由の 1 行・正本の ID）を必須にし、欄のある行の 3 つが等しければ fail。期待と違う行を全部集めてから 1 回で fail する（AC1 の red で今の違いの一覧が 1 回の実行で出る）。comment に `REQ-401 / D-158 / D-103 (1)` | 日報のどちらかの関数が `clean_number` に戻る・`+` を受ける・`¥` を外す・整数部を trim する。Z004 側だけが変わる。上限の値が変わる（`10000000` の個数が受かる等）。承認済みの差の欄に理由の無い行が足される |
| D-158: 日報の 3 帳票の全部の数の列が規則を通る | 呼出しの 1 か所が共有の関数を通らない | unit（`daily_report_parser.rs` の `mod tests`、`parse_daily_report_bundle` 経由） | T2 `test_parse_daily_report_req401_rejects_malformed_grouping_in_every_number_column`: Z001（総売の個数・ほかの行の件数・金額）、Z002（件数・金額）、Z005（個数・金額）の各列に `0,123`・`1,2,3`・`¥1,234`・`1 234`・`+5` を、個数の列には `1 .5`・`1\t.5`・`1　.5`・`1,234 .5` も 1 つずつ入れて、その帳票・その行に `invalid_number`。同じ列に `1,234`・`-12,345`（個数の列は `1,234.5` も）を入れて今の値で読む | 7 か所の呼出しのどれかが別の読取りを使う。正しい 3 桁区切りを拒む。個数の列の整数部が trim される |
| BIZ-08-D2a（広げた形）: `invalid_number` の束は帳票と列を名指し、値・行番号を出さない | 汎用の文のまま（今）、値や行番号が漏れる、`number_too_large` の優先が崩れる | unit（`daily_report_import_service/tests.rs`） | T3 `test_daily_report_req401_invalid_number_names_column_without_raw_value`: (i) Z002 の金額に `1,23` の束で、message と `operation_logs.summary` が 37 の文（`Z002の金額列を変換できません。この日の日報は読めません。管理者に知らせてください。`。期待の文は 37 から転記し production の定数を import しない）で、`1,23`・`9` を含まない。(ii) 同じ束に `number_too_large` の列を足すと、列の文が `number_too_large` の列だけで後半が同じ文になる（例 Z001 の金額に `10000000000` を足して `Z001の金額列の値が大きすぎます。この日の日報は読めません。管理者に知らせてください。`。37 から転記）。入力は `invalid_number` だけを見る（汎用の文の経路は直した tests.rs:147 が見る） | S4 の分岐が無い・順が逆・`error_message` 以外の値を渡す |
| 取消の後の互換の限界 | 旧い規則で受けた束の取消の後に、読み直しが通る（緩い受理が戻る）か、取消が失敗する | integration（`daily_report_import_service/tests.rs`、合成の DB） | T4 `test_daily_report_req401_rolled_back_lenient_import_rereads_as_invalid_number`: 合成の旧取込み（Z002 の金額に `1,23` の束の `bundle_hash`〈`build_bundle_hash` に `parse_daily_report_bundle` の `source_files` を渡した値〉で `sales_repo::insert_daily_report_import` が書いた `completed` の行。旧い規則の commit を再現する代わり）を置き、`list_daily_report_imports` がその行を `completed`（記録の一覧の「取込み済み」）で返す → `rollback_daily_report_import` が成功し行が `rolled_back` → 同じ束の `parse_and_validate_daily_report` が T3 (i) の文の `ImportError` で止まる（`AlreadyImported`・preview にならない） | 読み直しが preview まで通る、取消が数の規則で失敗する、取消の後の文が汎用の文になる |
| §37.9 手順 5 / D-127: SD の一覧の数の誤りの「読めません」は帳票・列を名指し、管理者に知らせるよう伝える | 理由が出ない（今）、値・file 名・行番号が漏れる、対象外（数でない誤り・`settlement_mismatch` のある束・ほかの理由の `Unreadable`）に文が付く | unit（`daily_report_import_service/tests.rs`、`scan_with_roots` の合成の SD） | T5 `test_req401_scan_number_error_unreadable_names_column`: Z002 の金額に `1,23` の束が `Unreadable` で `unreadable_reason` が 37 §37.9 の文（`Z002の金額列を変換できません。この日の日報は読めません。管理者に知らせてください。`。37 から転記）で `1,23` と file 名を含まない。同じ scan の次の 3 つの束は `Unreadable` で `unreadable_reason` が `None`: BOM 付きの束（`test_req401_scan_shape_violation_is_unreadable` と同じ作り）、(a) `invalid_format` だけの束（Z001 の 9 行目を 3 列のデータ行にした、S4 で直す tests.rs:147 と同じ作り）、(b) `settlement_mismatch` と `invalid_number`（Z002 の金額に `1,23`）が一緒の束 | S5 の枝が無い・対象外（数でない誤り・`settlement_mismatch` のある束・ほかの理由）にも文を付ける・`error_message` 以外を渡す |
| 55 UI-07-D15 (3): 「読めません」の行に理由の文が出る | 画面が field を出さない | frontend（`DailyReportImportPage.sd.test.tsx`） | T6 `REQ-401 / UI-07-D15 (3): an unreadable row with a reason shows the reason text, one without shows none`: `unreadable_reason` に 37 §37.9 の形の文（`Z002の金額列を変換できません。この日の日報は読めません。管理者に知らせてください。`）のある `Unreadable` の行と `null` の行を並べ、前の行の中にだけその文が出る（`row()` の helper で行を引く） | `SdCandidateRow` が文を出さない・ほかの行に出す |
| 同上（mutation） | 日報の整数の列が緩む | mutation | M1: `parse_optional_i64` の共有の関数の呼出しを `value.trim().replace([',', '￥', '¥', ' '], "")` に戻す | T1・T2 が red |
| 同上（mutation） | 日報の個数の列が緩む | mutation | M2: `parse_optional_hundredths` の整数部の検査を同じ置換に戻す | T1・T2 が red |
| D-103 (1) 先頭のグループは 0 で始まらない | 共有の関数が緩む | mutation | M3: 共有の正規表現の `[1-9][0-9]{0,2}` を `[0-9]{1,3}` に | `test_parse_z004_req401_malformed_comma_invalid_number`（`0,123`）・T1（`0,123` の行の 3 つの期待）・T2（`0,123`）が red |
| D-103 (1) `+` を拒む | 共有の関数が `+` を受ける | mutation | M4: 共有の正規表現の `-?` を `[+-]?` に | `test_parse_z004_req401_malformed_comma_invalid_number`（`+5`）・T1・T2 が red |
| BIZ-08-D2 / D2a の順 | `invalid_number` の文が `number_too_large` より先に選ばれる | mutation | M5: `number_column_text` の `number_too_large` の優先を外し、`invalid_number` を先に選ぶ | T3 の (ii) が red（既存の `test_daily_report_req401_over_limit_names_column_without_raw_value` は `invalid_number` を含まないので red にならない） |
| A1 空の列は `None` | 空の判定が共有の関数の後に回り、空が `invalid_number` になる | mutation | M6: `parse_optional_i64` の空の早期 return を消す | T1 の A1 の行が red（既存の test のどれが空の列で red になるかは実装の段で数えて報告に書く。fixture の Z001 には空の値の列がある） |
| BIZ-08-D1 汎用の文 | 数でない誤りの `error_message` が利用者へ渡る | mutation | M7: S4 の `else` の枝が汎用の文の代わりに最初の誤りの `error_message` を渡す | 直した `test_daily_report_req401_parse_error_logs_parse_failed` が red |
| §37.9 手順 5 の理由 | scan が理由を落とす | mutation | M8: S5 の `parse_errors` の枝の `unreadable_reason` を `None` にする | T5 が red |
| §37.9 手順 5 の対象外 | scan が `settlement_mismatch` のある束に理由を付ける | mutation | M10: S5 の `parse_errors` の枝の `settlement_mismatch` の除外を外す | T5 の (b) が red |
| UI-07-D15 (3) の理由の表示 | 画面が理由を出さない | mutation | M9: `SdCandidateRow` の `unreadable_reason` の段落を消す | T6 が red |
| 今の挙動（受ける値と値） | 置き換えで既存の値・error が変わる | unit・integration（既存） | `cargo test` と `npm test` 全体（AC4） | 既存の Z004・日報・BIZ-08・SD の一覧の test の値と error が変わる |

状態の遷移: 読取りの段のほかは、T4 の「取込み済み（`completed`） → 取消（`rolled_back`） → 読み直しで拒む」の 1 本。新しい DB の書込み・再試行・再起動の遷移は持たない。negative は T1 の崩れた行、T2 の各列、T3 の (i)、T4 の読み直し、T5 の BOM の束。

## L3 を置かない理由

`## L3` 節は置かない（manual なし）。利用者に見える変化は (i) 崩れた値の file を読んだとき手で選ぶ経路の文が汎用の文から帳票と列を名指す文に変わること、(ii) SD の一覧でその束が「読めません」になり、行の下に帳票・列を名指す文が出ることだけで、どちらもレジの出す正しい日報（F4、Contract Probe）では起きない。文の中身は T3・T5 が BIZ の境界で見て、画面は (i) が `ImportError` の message をそのまま出す既存の経路（`number_too_large` の文で `DailyReportImportPage.flow.test.tsx` が固定）、(ii) が T6 の jsdom の test で見るので、DEV_WORKFLOW L3 の L3 Eligibility の (1)（Windows/Tauri native でしか観測できない）を満たさない。

## 計画の review に問う点

1. S4（`invalid_number` の文で帳票と列を名指す）の文と、行番号を出さない線引き（BIZ-08-D1・IO-07-D6）でよいか。
2. S5 を `invalid_number`・`number_too_large` に限り、`settlement_mismatch` の束とほかの理由の `Unreadable` に文を出さない線引きでよいか。
3. T4 の合成の旧取込みの作り方（`insert_daily_report_import` で `completed` の行を置く）でよいか。
4. L3 を置かないことでよいか（上の節）。
5. D-158 を新しい D にし、D-103 に `Superseded in part` を足さない扱いでよいか。
