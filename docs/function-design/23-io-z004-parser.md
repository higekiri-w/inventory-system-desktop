## 13. IO-02: Z004パーサー

### 精算の識別メタ（D-109 で実装、D-118 で用途を改めた）

`parse_z004` は純関数・DB 非依存のまま、`ParseResult` に `settlement_metadata: Option<SettlementMetadata>`（machine_no / report_kind / settlement_no / settled_at、各 Option、番号は文字列）を返す（D-109 の ② で実装済み）。settled_at は POS が印字した精算日時で、取引の時刻やアプリの受領の時刻ではない。parser は精算の系列・同一性・時計の信用を認定しない。

[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)（SPEC-STK-CNT-D4、D-118）からの用途: BIZ-03 は machine_no と settlement_no（10 進の整数にした値）と精算日で、Z004 を識別し、並べ、欠けを知らせる。machine_no か settlement_no を読めない file（従来の形）は BIZ-03 が取り込まない（[32](32-biz-csv-import-service.md) の新契約。parser は今どおり受理して None を返し、拒否は BIZ の責務）。精算日時は検知（精算日と精算回数の順の食い違い）と表示にだけ使い、前後の判定に使わない。

- 数量・金額とも 0 の正常 JAN の行を BIZ の判定まで保持する旧 ADR（時点証拠）の要求は外した（売上と返品の相殺の判定を持たない。SPEC-STK-CNT-D11）。parser の出力の形は変えない。
- 処理は既存のサイズ外ガード・strict decode・改行の正規化・layout / header の検出・日付 / JAN / 符号の規則を維持する。PLU の占有の読取り mode も用途を維持する（D-117）。

### 13.1 モジュール構成

```
src-tauri/src/
  io/
    mod.rs            -- pub mod z004_parser
    z004_parser.rs    -- Z004ファイルパーサー（純関数、DB非依存）
```

IO層の新ディレクトリ。DB操作を伴わない純粋なファイルフォーマット変換を配置する。
`db/` とは異なり `DbConnection` を一切受け取らない。

---

### 13.2 型定義

#### ParseResult構造体

Z004ファイルのパース成功時の結果。行単位エラーがあっても返る（致命的エラーでなければ）。

- settlement_date: String（YYYY-MM-DD。従来shapeは1行目、layout Aはヘッダより前のメタ行群から抽出した精算日）
- parsed_rows: Vec\<ParsedRow\>（正常にパースできたデータ行）
- parse_errors: Vec\<ParseError\>（行単位のパースエラー）
- total_data_lines: usize（改行正規化後、検出したヘッダ行より後の非空行でフィールド分割を試みた行の総数。Ok(Some)=正常行、Ok(None)=売上の無い枠（SPEC-Z4A-D8）、Err=エラー行すべてカウント。空文字列のみの行は除外。従来shapeではヘッダが2行目なので旧定義と実質同値。SPEC-Z4A-D4）
- file_hash: String（SHA-256、生バイト列から算出、hex小文字64文字。INV-6準拠）
- settlement_metadata: Option\<SettlementMetadata\>（精算の識別メタ。layout A のメタ行から読む。従来shapeは None。冒頭の節）

#### SettlementMetadata構造体

精算の識別メタ。番号は意味を確かめるまで文字列のまま（先頭の 0 を保つ）。parser は精算の系列・同一性・時計の信用を認定しない。

- machine_no: Option\<String\>
- report_kind: Option\<String\>
- settlement_no: Option\<String\>
- settled_at: Option\<String\>（POS が印字した精算日時）

#### ParsedRow構造体

正常にパースできた1データ行。

- line_no: usize（ファイル内の行番号。1始まり）
- normalized_jan: String（正規化後のJANコード。13桁かJAN-8の8桁。形は§13.5）
- name: String（Z004上の商品名。半角カナ等そのまま）
- quantity: i32（数量。マイナスあり＝返品）
- amount: i32（金額。マイナスあり＝返品）

#### ParseError構造体

行単位のパースエラー。他の行の処理は継続する。

- line_no: usize（エラーが発生した行番号。1始まり）
- error_type: ParseErrorType
- error_message: String（日本語、利用者向けメッセージ）
- raw_name: Option\<String\>（パース途中で取得できた商品名。フィールド分割前のエラーではNone）
- raw_quantity: Option\<String\>（パース途中で取得できた数量生値）
- raw_amount: Option\<String\>（パース途中で取得できた金額生値）

#### ParseErrorType列挙型

```
enum ParseErrorType {
    InvalidFormat,   // フィールド数不正等の構造エラー
    InvalidJan,      // JANコード正規化失敗
    InvalidNumber,   // 数量・金額の数値変換失敗
}
```

#### Z004ParseError列挙型（致命的エラー）

ファイル全体の処理を中断する致命的エラー。Result::Errとして返す。

```
enum Z004ParseError {
    DecodeFailed(String),       // CP932デコード失敗
    NoDataLines(String),        // 2行未満（ヘッダ行すらない）
    NoSettlementDate(String),   // ヘッダまたは精算日を抽出不能
    ImportError(String),        // 全スロット占有読取り mode の構造が契約を満たさない（§13.3.1。parse_z004 は返さない）
}
```

**設計意図**: 行単位エラー（ParseError）と致命的エラー（Z004ParseError）を分離する。行単位エラーは「他の行は処理できた」を意味し、致命的エラーは「ファイル自体が不正」を意味する。BIZ-03はこの区別を使って、operation_logsへの記録パターンを分岐する。

---

### 13.3 parse_z004（公開関数）

**関数要求**: Z004ファイルの生バイト列を受け取り、構造化データに変換する。純粋関数。DB非依存。副作用なし

**シグネチャ**:
```
fn parse_z004(raw_bytes: &[u8]) -> Result<ParseResult, Z004ParseError>
```

**処理ステップ**:
1. file_hash算出: SHA-256(raw_bytes) → hex小文字64文字
2. CP932 strictデコード
   - 失敗 → Err(Z004ParseError::DecodeFailed("CP932デコードに失敗しました。ファイル形式を確認してください"))
3. 改行正規化: `\u{0085}`（NEL）/ `\r\n` / `\n` / `\r` → `\n` に統一
   - 順序: `\r\n` → `\n` を先に処理し、その後 `\r` → `\n`（順序を逆にすると `\r\n` が `\n\n` になる）
   - `\u{0085}` → `\n` は独立して処理可能
4. `\n` で行分割。空行は除去しない（行番号を保持するため）
5. 2行未満 → Err(Z004ParseError::NoDataLines("データ行がありません。ファイル形式を確認してください"))
6. shape判定とヘッダ位置・settlement_dateの確定（SPEC-Z4A-D1〜D3）
   - 従来shape: 1行目に `YYYY-MM-DD` があり、かつ2行目が中間強度検査（CSV分割後が5フィールドかつ第2フィールドに「コード」または半角カナ「ｺｰﾄﾞ」を含む。第5フィールドのラベル照合はしない）を満たす場合。settlement_dateは1行目の一致を現行どおり採用する。実物の Z004 は layout A で（2026-07-06 の店の採取）、従来shapeの file は精算回数・マシンNo. を持たないので BIZ-03 が取り込まない（[32](32-biz-csv-import-service.md) の新契約の表の 1）。parser のこの分岐を外すのは code の別の lane
   - layout A: 従来shapeの二重条件を満たさない場合、先頭20行以内から最初のヘッダ行を走査する。ヘッダ未検出 → Err(Z004ParseError::NoSettlementDate("ヘッダ行を検出できません。ファイル形式を確認してください"))
   - layout Aヘッダ検査: CSV分割後が5フィールドで、第2フィールドに「コード」または半角カナ「ｺｰﾄﾞ」、第5フィールドに「金額」を含むこと（実ファイルのヘッダ第2フィールドは半角カナ「ｽｷｬﾆﾝｸﾞｺｰﾄﾞ」、2026-08-17 実ファイル機械抽出）。フィールド数だけでなく位置アンカー付きラベル照合を重ね、5フィールドの非ヘッダ行を排除する
   - layout Aの精算日は、ヘッダより前のメタ行群で第1フィールドに「日付」を含む行を優先する。該当行に日付がない場合のみ、メタ行群の先頭から最初の日付パターンへfallbackする
   - layout Aの日付は `YYYY-MM-DD` / `YYYY-M-D` / `YYYY/M/D`（dash / slash 両区切りとも月日1〜2桁）を受理し、`YYYY-MM-DD`へゼロ埋め正規化する。日付未検出 → Err(Z004ParseError::NoSettlementDate("精算日を抽出できません。ファイル形式を確認してください"))
   - 二形状のどちらでもない入力は上記致命的エラーで安全停止し、部分結果を返さない
7. 検出したヘッダ行をスキップ
8. ヘッダ行より後: 各行について
   - 空行（trimして空文字列）→ スキップ（エラーにもカウントしない）
   - parse_data_line(line, line_no) を呼び出し
     - Ok(Some(row)) → parsed_rowsに追加
     - Ok(None) → 売上の無い枠（SPEC-Z4A-D8）。スキップ（エラーにもカウントしない）
     - Err(error) → parse_errorsに追加
9. ParseResult { settlement_date, parsed_rows, parse_errors, total_data_lines, file_hash } を返す

**入力例**:
```
raw_bytes: CP932エンコードされたZ004ファイル
  layout A:
  1〜6行目: マシンNo. / ファイル / モード / 精算回数 / 日付 / 時刻のメタ行（2列、CP932 12byte固定幅のラベルpadding。2026-08-17 実ファイル機械抽出）
  7行目: 空行（区切り）
  8行目: "レコード    ","ｽｷｬﾆﾝｸﾞｺｰﾄﾞ ","キャラクター","個数        ","金額        "
  9行目以降: 5フィールドの全スロットダンプ

  従来shape（parser は読むが BIZ-03 が拒む）:
  1行目: "精算日報 2026-03-21 ..."
  2行目: "No,コード,名称,個数,金額"
  3行目: "1","4976383262108","ﾊﾏﾅｶ ｱﾐｱﾐ極太",3,1782
  4行目: "2","00000000000000","",0,0
  ...
```

**出力例**:
```
Ok(ParseResult {
    settlement_date: "2026-03-21",
    parsed_rows: [
        ParsedRow { line_no: 3, normalized_jan: "4976383262108", name: "ﾊﾏﾅｶ ｱﾐｱﾐ極太", quantity: 3, amount: 1782 },
        ...
    ],
    parse_errors: [],
    total_data_lines: 150,
    file_hash: "a1b2c3d4e5f6..."
})
```

**エラーハンドリング**:
- CP932デコード失敗 → Z004ParseError::DecodeFailed（即リターン）
- 2行未満 → Z004ParseError::NoDataLines（即リターン）
- 先頭20行以内にヘッダ未検出 → Z004ParseError::NoSettlementDate（文言: `ヘッダ行を検出できません。ファイル形式を確認してください`、即リターン）
- メタ行群から日付抽出不能 → Z004ParseError::NoSettlementDate（文言: `精算日を抽出できません。ファイル形式を確認してください`、即リターン）
- 行単位エラーはparse_errorsに蓄積し、他の行の処理は継続

**二形状 amendment（SPEC-Z4A-D1〜D6）**:

| ID | 契約 |
|---|---|
| SPEC-Z4A-D1 | 従来shapeとlayout Aを受理し、従来shapeは1行目日付 + 2行目中間強度検査（5フィールド + 第2フィールドに「コード」/半角「ｺｰﾄﾞ」含有、第5フィールドのラベル照合なし）の二重条件で判定する（従来shapeは BIZ-03 が拒む。§13.3 手順 6）。二形状外は致命的エラーでfail-closedする |
| SPEC-Z4A-D2 | layout Aのヘッダは5フィールド + 第2「コード」（半角カナ「ｺｰﾄﾞ」を含む）/第5「金額」の位置アンカーで検査し、先頭20行以内だけを走査する |
| SPEC-Z4A-D3 | layout Aの精算日は「日付」ラベル行優先 + 最初の日付パターンfallbackとし、`YYYY-MM-DD` / `YYYY-M-D` / `YYYY/M/D`（月日1〜2桁）をゼロ埋め正規化する |
| SPEC-Z4A-D4 | `ParseResult`系の型と意味論を変えず、line_noは物理行番号、total_data_linesはヘッダ後の試行行数とする |
| SPEC-Z4A-D5 | `E`は14桁固定幅の右パディング。13桁JAN + Eと8桁 + EEEEEEは除いてJANに正規化する（8桁はJAN-8。§13.5、D-166で改訂） |
| SPEC-Z4A-D6 | layout Aの全スロットダンプを既存BIZ-03上限内で受理し、売上の無い枠（SPEC-Z4A-D8）をskipする。コード全桁0でも個数か金額が0でない行はskipしない（D-103で改訂） |
| SPEC-Z4A-D7 | 個数・金額は3桁区切りのカンマ付きの整数も受理する（§13.4.1、D-103） |
| SPEC-Z4A-D8 | 行の分類は個数・金額を先に読み、売上の有無で決める。コード全桁0で売上のある行（PLUの登録を消した枠、コードを持たない通常PLU）はInvalidJanの行エラーにし、黙って捨てない（§13.4.2、D-103） |

---

### 13.3.1 IO-02 全スロット占有読取り mode（IO-02-D1 / SPEC-PLS-D2）

売上取込み用 `parse_z004` とは別に `parse_plu_register_snapshot(raw_bytes)` を公開する。この mode は layout A の preamble / ヘッダ検査と CP932 decode を再利用するが、売上列・精算日・JAN 妥当性を評価せず、ヘッダ後の **5,000 行**すべてを `PluRegisterSlot { memory_no, raw_code }` として返す。

```rust
pub fn parse_plu_register_snapshot(raw_bytes: &[u8]) -> Result<Vec<PluRegisterSlot>, Z004ParseError>
```

- memory_no は第1 field を整数として読み、重複・欠落・範囲外を許さない。
- 14 桁コードが全ゼロなら `raw_code=None` とし、行自体は skip しない。
- 13 桁数字 + 右端 `E` は padding を 1 文字だけ除いて 13 桁へ正規化する。
- **8 桁コード + `E`×6 は 14 桁 raw のまま返す**。JAN error にせず、BIZ-04 が `external` として扱える観測値を保存する。
- 上記以外の非空コードは trim や JAN 補正をせず raw のまま返し、authority 判定を BIZ-04 に委ねる。
- データ行数が **5,000** でない、memory No. が欠落・重複する、または layout A header を検出できない場合は `ImportError` として fail-closed し、部分結果を返さない。

この mode は occupancy snapshot 専用であり、`ParsedRow`、`sale_records`、`inventory_movements`、日報集計へ入力しない（SPEC-PLS-D8）。

入力は毎日の締めの SD の読取り（BIZ-08 §37.9）が読んだ窓の中の Z004 で、BIZ-04 が精算の識別メタの `settlement_no` が最大の 1 本を選んで渡す（[33 §16.3](33-biz-plu-export-service.md)、D-117 (3)。未実装: PLU の書出しの見直しの runtime。今は店主が FilePicker で選んだ Z004 を読む）。店主が FilePicker で選ぶ経路は撤去する（runtime の lane）。mode の形は変えない。

---

### 13.4 parse_data_line（内部関数）

**関数要求**: Z004の1データ行をパースし、ParsedRowに変換する。売上の無い枠（SPEC-Z4A-D8）はOk(None)で返す

**シグネチャ**:
```
fn parse_data_line(line: &str, line_no: usize) -> Result<Option<ParsedRow>, ParseError>
```

**処理ステップ**:
1. ダブルクォート囲みCSV分割 → 5フィールド取得
   - フィールド: record_no, scanning_code_raw, name_raw, quantity_raw, amount_raw
   - フィールド数 ≠ 5 → Err(ParseError { line_no, error_type: InvalidFormat, error_message: "行{line_no}: フィールド数が不正です（期待: 5, 実際: {n}）" })
2. quantity_raw → 整数の読み取り（§13.4.1）
   - 失敗 → Err(ParseError { line_no, error_type: InvalidNumber, error_message: "行{line_no}: 数量が数値ではありません: '{raw}'" })
3. amount_raw → 整数の読み取り（§13.4.1）
   - 失敗 → Err(ParseError { line_no, error_type: InvalidNumber, error_message: "行{line_no}: 金額が数値ではありません: '{raw}'" })
4. 売上の有無: has_sales = quantity ≠ 0 または amount ≠ 0
5. scanning_code_raw → normalize_jan(scanning_code_raw, line_no) 呼び出し（分類は§13.4.2）
   - Ok(None)（全桁ゼロ・8桁の全部0 + EEEEEE）かつ has_sales = false → Ok(None)（売上の無い枠。エラーにもカウントしない）
   - Ok(None)（全桁ゼロ・8桁の全部0 + EEEEEE）かつ has_sales = true → Err(ParseError { line_no, error_type: InvalidJan, error_message: "行{line_no}: 商品コードの無い枠（メモリNo.{record_no}）に売上があります。PLU の登録を消した枠の売上などで、在庫には反映されません" })。record_no は第1フィールドを trim した値
   - Err(msg) かつ has_sales = false → Ok(None)（売上の無い枠）
   - Err(msg) かつ has_sales = true → Err(ParseError { line_no, error_type: InvalidJan, error_message: msg })
   - Ok(Some(normalized_jan)) → 次ステップへ（個数・金額とも0の行も返す。除外はBIZ-03 Stage 2）
6. Ok(Some(ParsedRow { line_no, normalized_jan, name: name_raw.to_string(), quantity, amount }))

どのErrでも raw_name / raw_quantity / raw_amount には分割後のフィールド値をそのまま入れる（従来どおり）。

#### 13.4.1 個数・金額の整数の読み取り（SPEC-Z4A-D7）

前後の空白を trim した値が次のどちらかに全体で一致するときだけ受理し、カンマを除いて i32 にする。カンマなしの値も式で判定し、`str::parse` だけに任せない（`+5` を拒む）。それ以外は InvalidNumber（例: 空、小数、区切りの位置が違うカンマ、先頭グループが0で始まるカンマ付きの値 `0,123`、符号 `+`、i32の範囲外）。

- `-?[0-9]+`（カンマなし）
- `-?[1-9][0-9]{0,2}(,[0-9]{3})+`（3桁区切り。先頭グループは0で始まらない1〜3桁。例: `1,234`、`-12,345`）

根拠: 実データの Z004 の金額欄に、1,000 以上の値が3桁区切りのカンマ付きで出る形がある（持ち帰りデータの手元集計 2026-10-04。件数は数え直し前のため書かない）。数量欄のカンマは観測されていないが、同じ帳票で1,000以上の数量だけが別の書式になる根拠も無いため、同じ規則を当てる（数量にカンマが出なければ、この分岐は使われないだけ）。

却下案:
- カンマ・円記号・空白をすべて除いてから読む緩い除去: `1,23` のような区切りの崩れた値を黙って別の数として読む。本書の目的（読めない値を黙って誤らない）に反する。日報（IO-07）も同じ規則で読む（[D-158](../decision-log.md)。検査は `src-tauri/src/io/mod.rs` の 1 か所）
- カンマ付きの値を InvalidNumber のまま残す（従来）: 1,000 円以上の PLU の売上が在庫に反映されない
- 小数の受理: 単位の拡張（D-113。backlog「単位の拡張の残りの申し送り」の P）で数量の型ごと決める。本書では InvalidNumber のまま

単位の拡張の後（proposed・未実装、Z004・EJ の lane、D-113）: 個数（第 4 欄）は、上の 2 つの形の整数部に任意の `.` + ASCII 数字 1〜2 桁を続けた形を受け、小数 2 桁までの 100 倍の整数（`quantity_hundredths: i64`、`ParsedRow.quantity` を置き換える）にする（[共通規則](10-common-rules.md) SPEC-UNIT-D8。浮動小数を通さない）。金額（第 5 欄）は今の整数の規則のまま。IO は商品を引かず、在庫の数量への換算（個数の商品で 100 で割り切れない値は在庫に効かせない、長さの商品は 100 倍の整数がそのまま cm）は BIZ-03 が行う。型の変更は [backlog](../backlog.md) の「単位の拡張の残りの申し送り」の P が BIZ-03 の換算（[32](32-biz-csv-import-service.md) の「数量」）と一緒に行い、それまで上の規則が実装の正本である。

#### 13.4.2 売上の有無による行の分類（SPEC-Z4A-D8、D-103）

| コード欄 | 個数・金額とも0 | 個数か金額が0でない |
|---|---|---|
| JAN（§13.5 の `Ok(Some)`。13桁 + 右パディング1文字、8桁 + `EEEEEE` を含む） | Ok(Some)。BIZ-03 が空レコード（quantity=0 かつ amount=0）として除外する（D-118 で相殺の判定は持たない。[32](32-biz-csv-import-service.md) の冒頭の D-118 の節） | Ok(Some) |
| 全桁0（未登録の枠、PLU の登録を消した枠、コードを持たない通常PLU。8桁の全部0 + `EEEEEE` を含む） | Ok(None) | InvalidJan（商品コードの無い枠の売上） |
| JANにならない（§13.5 の `Err`） | Ok(None) | InvalidJan（normalize_jan の文言） |

根拠: PLU の登録を消した（clear した）枠は、名前が空・コード全桁0のまま、精算の前の数量と金額を持って次の Z004 に残る（2026-09-28 の検証、持ち帰りデータの手元集計 2026-09-29）。従来の「全桁0 = 空スロット」は、この売上を数量・金額を見ずに捨て、在庫にもエラーにも出さなかった。owner 決定 TD-105（2026-10-04）で、エラーとして知らせ、元の商品へ戻すことはしない。JANにならないコードの枠（D-103 の時点では 8桁 + EEEEEE の枠。D-166 から JAN-8 として受ける）は 2026-07-06 以降の Z004 に毎回あり（持ち帰りデータの手元集計 2026-10-04）、従来は売上が無くても毎回 InvalidJan になり、取込みは毎回 completed_partial になっていた。売上の無い枠を毎回のエラーに出すと、本当に知らせたい行（上の表の右列）が同じ一覧に埋もれる。

知らせ方: 既存の行エラーの経路をそのまま使う。BIZ-03 が `invalid_jan` の ErrorRow にし、取込みは completed_partial になり、`csv_import_errors` に残る。プレビューのエラー詳細と取込み記録の詳細の「メッセージ」「内容」列に上の文言が出る。プレビューのエラー詳細（最大100件）では invalid_jan の行が他の種別の行より先に選ばれ、未登録 JAN の行が多くても落ちない（[32 §15.3 手順 5c](32-biz-csv-import-service.md)）。normalized_jan は None（画面は「(不明)」「—」）、raw_quantity / raw_amount は元の文字列。

却下案:
- 新しい error 種別（例 `cleared_slot`）: `csv_import_errors.error_type` の CHECK の migration、`CsvImportErrorType` の generated enum と bindings、画面の label 2 か所が要る。見分けは文言で足り、実データでもまれ。後続（例: ADR D5 の合計の照合）が種別で区別する必要が出たら足す
- memory No. と PLU 枠の割当ての履歴から元の商品へ戻す: owner 決定 TD-105 で採らない
- 名前が空であることも条件にする: コードを持たない通常PLU（名前あり）の売上も同じく在庫に反映されないので、同じく知らせる。名前の有無に依らない
- コードの判定を先に、数量の判定を後にする（従来の順）: 売上の有無で分類できない

Revisit: 通常PLU（番号打ちの PLU）を店が日常に使い始め、毎回の取込みにこのエラーが出るようになったとき。後続の lane が「商品コードの無い枠の売上」を種別で数える必要が出たとき。

**CSVフィールド分割の仕様**:
- ダブルクォート囲み: フィールド値がダブルクォートで囲まれている場合は除去する
- ダブルクォート内のカンマ: フィールド区切りとして扱わない
- ダブルクォートのエスケープ: `""` → `"`
- 囲みなしフィールドも許容（Z004の実データで混在する可能性を考慮）

---

### 13.5 normalize_jan（内部関数）

**関数要求**: Z004のスキャニングコードをJANコード（13桁か、JAN-8の8桁）に正規化する。全桁ゼロはOk(None)で返す（売上の有無による分類は呼出し側の§13.4.2）

**シグネチャ**:
```
fn normalize_jan(raw: &str, line_no: usize) -> Result<Option<String>, String>
```

**処理ステップ**:
1. 前後空白をtrim
2. 全桁ゼロ判定: 全文字が '0' → Ok(None)
   - 桁数を問わない: 13桁ゼロ `0000000000000`・14桁ゼロ `00000000000000` のほか、他の桁数の全ゼロ（例 `0`、`00000000`）もOk(None)。空文字（trim後）はこの判定に当たらず、手順6のErr
3. 末尾の大文字 `EEEEEE` を除いた値（無ければそのまま）が ASCII 数字8桁 → 8桁が全部0なら Ok(None)、それ以外は Ok(Some(8桁))（JAN-8。D-166）。8桁の詰めは大文字 `EEEEEE` だけで、他の英字・小文字の `e`・`E` が6個でない詰めは手順6のErr
4. 14桁かつ末尾がASCIIアルファベット（a-z, A-Z）→ 14桁固定幅の右パディング1文字を除去して13桁化
5. 結果が13桁かつ全文字が数字 → Ok(Some(normalized))
6. それ以外 → Err("行{line_no}: JANコード '{raw}' を正規化できません")

**正規化ルール**:
- 先頭ゼロは保持する（JANコードの正当な構成要素）
- チェックデジット検証はしない（レジ出力をそのまま信頼。DB_DESIGN.md準拠）
- 8桁（素の8桁か8桁 + 大文字 `EEEEEE`）と13桁（素の13桁か13桁 + 右パディング1文字）以外の長さは不正（Err）
- 14桁で末尾が数字の場合は不正（Err）。末尾アルファベットは識別子ではなく、コード欄を14桁固定幅にする右パディングとして除去する（SPEC-Z4A-D5）

**入力→出力例**:

| 入力 | 出力 | 説明 |
|------|------|------|
| `"4976383262108"` | Ok(Some("4976383262108")) | 13桁そのまま |
| `"4976383262108E"` | Ok(Some("4976383262108")) | 14桁末尾E除去 |
| `"00000000000000"` | Ok(None) | 14桁全ゼロ（売上の有無は§13.4.2） |
| `"0000000000000"` | Ok(None) | 13桁全ゼロ（売上の有無は§13.4.2） |
| `"12345678EEEEEE"` | Ok(Some("12345678")) | 8桁 + EEEEEE（JAN-8） |
| `"12345678"` | Ok(Some("12345678")) | 8桁そのまま（JAN-8） |
| `"00000000EEEEEE"` | Ok(None) | 8桁全ゼロ + EEEEEE（売上の有無は§13.4.2） |
| `"12345678ABCDEF"` | Err(...) | EEEEEE 以外の詰め＝不正 |
| `"497638326210"` | Err(...) | 12桁＝桁数不足 |
| `"49763832621089"` | Err(...) | 14桁末尾数字＝不正 |
| `"ABCDEFGHIJKLM"` | Err(...) | 数字以外＝不正 |

---

### 13.6 入力境界仕様

| 項目 | 仕様 | 根拠 |
|------|------|------|
| エンコーディング | CP932 strict（デコード失敗は致命的エラー） | カシオSR-S4000のZ004出力形式 |
| 区切り文字 | カンマ、ダブルクォート囲み、5フィールド固定 | Z004実機検証結果 |
| 改行 | \u{0085} (NEL) / \r\n / \n / \r を正規化 | CP932デコード後に正規化。生バイトでの分割は誤判定リスクあり。持ち帰った Z 帳票と SD の file は CRLF だけで NEL の行終端の実例は無く（2026-10 の見直しの事実の一覧）、正規化は防御として残す |
| 空行 | スキップ（エラーにもカウントしない） | ファイル末尾等の余白行 |
| 制御文字 | 改行正規化後は特別な処理なし | |
| 金額・数量 | i32整数のみ。3桁区切りのカンマは受理（§13.4.1）。浮動小数は不正（InvalidNumber） | 実データの金額欄に3桁区切りの形がある。小数は Z001 / Z005 / EJ に出ており、Z004 の数量の小数は backlog「単位の拡張の残りの申し送り」の P で扱う |
| 入力上限 | 10,000行 / 20MB | IO-02では検査しない。BIZ-03でガードチェック |
| layout A行数 | 全5,000スロット程度を受理 | 既存BIZ-03上限内。parser側に新規サイズガードは設けない（SPEC-Z4A-D6） |
| 返品値 | quantity < 0, amount < 0 を許容 | Z004のレジ戻しはマイナス値で出力される |

---

### 13.7 エラーハンドリングまとめ

| エラー | 型 | 影響範囲 | 後続処理 |
|--------|---|---------|---------|
| CP932デコード失敗 | Z004ParseError::DecodeFailed | ファイル全体 | 即リターン（Result::Err） |
| 2行未満 | Z004ParseError::NoDataLines | ファイル全体 | 即リターン（Result::Err） |
| ヘッダ未検出（先頭20行以内） | Z004ParseError::NoSettlementDate | ファイル全体 | 原因別文言で即リターン（Result::Err） |
| 精算日抽出不能 | Z004ParseError::NoSettlementDate | ファイル全体 | 原因別文言で即リターン（Result::Err） |
| フィールド数不正 | ParseError (InvalidFormat) | 1行のみ | parse_errorsに追加、他の行は処理継続 |
| JAN正規化失敗（売上のある行だけ）・商品コードの無い枠の売上 | ParseError (InvalidJan) | 1行のみ | parse_errorsに追加、他の行は処理継続（§13.4.2） |
| 数値変換失敗 | ParseError (InvalidNumber) | 1行のみ | parse_errorsに追加、他の行は処理継続 |

**致命的エラー（Z004ParseError）と行単位エラー（ParseError）の使い分け**:
- Z004ParseError: ファイル自体が処理不能。parsed_rowsを構築する前に判明するエラー。BIZ-03はoperation_logsに `csv_import_parse_failed` として記録
- ParseError: 特定の行が不正だが、他の行は正常にパースできた。BIZ-03はcsv_import_errorsテーブルに記録

---

### 13.8 非目的

このモジュールが**やらないこと**を明示する。責務境界の誤解を防ぐため。

| やらないこと | 理由 | 責務を持つモジュール |
|------------|------|-----------------|
| DB操作 | 純関数モジュール。DbConnectionを受け取らない | IO-01（db/） |
| マスタ照合（JAN→product_code紐付け） | 業務ロジック | BIZ-03 Stage 2 Validate |
| 重複チェック（file_hash照合） | 業務ロジック | BIZ-03 Stage 3 Preview |
| 符号変換（売上帳票視点→在庫視点） | INV-1の在庫視点変換 | BIZ-03 Stage 4 Commit |
| 正常JANの空レコード除外（quantity=0 and amount=0） | 業務ルール判定（売上も movement も作らない行を決めるのは BIZ。D-118 で相殺の判定は持たない） | BIZ-03 Stage 2 Validate |
| 入力サイズ上限チェック（10,000行/20MB） | 上流のガードチェック | BIZ-03（parse_z004呼び出し前） |

---

### 13.9 対応不変条件

| 不変条件 | 本モジュールでの対応 |
|---------|-----------------|
| INV-6: file_hashの算出 | parse_z004のステップ1でSHA-256(raw_bytes)を算出。hex小文字64文字。デコード前の生バイト列から計算（改行コード差異も区別される） |
| INV-1: quantity符号規約 | 本モジュールは関知しない。Z004の生値（マイナスあり）をそのまま返す。在庫視点への変換はBIZ-03の責務 |

---

### 13.10 依存ライブラリ

| クレート | 用途 | 備考 |
|---------|------|------|
| sha2 | file_hashの算出（SHA-256） | Cargo.toml に追加済み |
| encoding_rs | CP932（Shift_JIS）デコード | 新規追加が必要。`SHIFT_JIS` デコーダを使用 |
| regex | 従来shapeの日付抽出、layout Aの日付2形式抽出・正規化 | 追加済み |

**encoding_rsの選定理由**: Rustの標準ライブラリにはCP932デコードがない。encoding_rsはWHATWG Encoding Standardの実装であり、`SHIFT_JIS` ラベルでCP932互換のデコードが可能。strictモード（`decode_without_bom_handling` + エラーチェック）で使用する。
