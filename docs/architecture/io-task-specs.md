# タスク仕様（IO層）

## 計数の置き場と在庫の求め直しの契約（D-118。(a)・b1・b2 は実装済み）

SPEC-STK-CNT-D1・D2・D4・D9（[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)）。(a)（ADR SPEC-STK-CNT-D12）で入ったのは、並びの鍵付きの movement、写しの列の書込み、求め直しの読取り（[21](../function-design/21-io-inventory-repo.md)）。計数の保存と Z004 の分は b1（[Plan Packet](../archive/plans/2026-10-22-stocktake-count-runtime-b1.md)）で入った。商品の読取りの写しの 3 列と、在庫変動の履歴の並べない 2 つの読取り（並びの鍵は履歴の行が持ち、`MovementRecord` の形は変えない）は b2（[Plan Packet](../plans/2026-10-23-stocktake-count-runtime-b2.md)）で入った。IO-01 は [20](../function-design/20-io-product-repo.md) / [21](../function-design/21-io-inventory-repo.md) / [24](../function-design/24-io-csv-import-repo.md) の新契約で、並びの鍵付きの movement と計数（`stock_counts`）を保存し、写しの列を書く（呼ぶのは BIZ の求め直しの関数だけ）。TX の開始と確定と業務の判断は BIZ が所有する。24 は Z004 の取込みに machine_no・settlement_no を保存し、active な精算の一覧を BIZ へ渡す。

IO-02 は [23](../function-design/23-io-z004-parser.md) の精算の識別メタを純粋に抽出する（D-109 で実装済み）。時計・精算の系列の信用は判断しない。既存の CP932・改行・符号・売上の無い枠の読み飛ばし（[23 §13.4.2](../function-design/23-io-z004-parser.md)、D-103）と PLU の占有の mode は保つ。(a) の migration v9（movement の並びの鍵の列・`stock_counts`・写しの列、`db/schema_v9.rs`）は入った。Z004 の取込みの記録の識別の列の SQL / schema は (b) で未実装で、[DB 契約](../DB_DESIGN.md) を runtime へ渡す。旧 ADR（時点証拠）の受領・cursor・kind・flag・版の保存は置き換えた。

> **親文書**: [ARCHITECTURE.md](../ARCHITECTURE.md)
> **入力ドキュメント**: `docs/spec/requirements.md`、`docs/spec/requirements-coverage.md`、DB_DESIGN.md（テーブル定義書）

---

### IO-01: SQLiteデータアクセス層

**タスク要求**: 全25テーブルへのCRUD操作、DB接続管理、初期設定を提供する

**理由**: DB操作をIO層に集約することで、BIZ層がSQLを直接書かなくて済む。DB固有の設定（PRAGMA、WALモード等）もここで一元管理

**【データ構造】**

- 各テーブルに対応するRust構造体（Product, Department, Supplier, ReceivingRecord, ...）
- クエリ結果をRust構造体にマッピング

**【処理構造】**

**DB接続初期化:**
1. SQLiteファイルを開く（なければ作成）
2. PRAGMA foreign_keys = ON
3. PRAGMA journal_mode = WAL
4. PRAGMA busy_timeout = 5000
5. MNT-03（マイグレーション）を呼び出してスキーマを最新化

**リポジトリパターン（実装時の分割単位）:**
- product_repository: products, departments, suppliers, price_history のCRUD
- inventory_repository: inventory_movements, receiving_records/items, return_records/items, manual_sales/items, disposal_records/items のCRUD
- sales_repository: sale_records, csv_imports, csv_import_errors のCRUD
- stocktake_repository: stocktakes, stocktake_items のCRUD
- system_repository: operation_logs, app_settings のCRUD
- plu_slot_repository: plu_slots の snapshot / reservation / confirm / release CRUD と products の JAN JOIN

**【制御構造】**
- コネクションプールは不要（1人運用デスクトップアプリ、単一接続で十分）
- トランザクション管理はBIZ層が開始/コミット/ロールバックを指示し、IO層が実行
- product_repository の keyword は商品名、product_code、jan_code、maker_codeの部分一致とし、検索一覧と filter 全件処理で共有する（SPEC-PRV-D2）
- 価格改定は products の価格・`plu_dirty`・NULL supplier_id だけを部分更新し、price_history の old/new 4 値を同じ BIZ transaction で書ける repository 関数を提供する（SPEC-PRV-D5 / D6）
- `find_or_create_supplier` は trim 後の name を使い、空文字を拒否する。`list_price_history` は `changed_at DESC, id DESC`、既定 10・上限 100、不存在 product_code は空配列とする（SPEC-PRV-D6 / D9）

---

### IO-02: Z004パーサー

**タスク要求**: Z004ファイルのバイト列を受け取り、構造化データに変換する。純粋なフォーマット変換のみ、業務ロジックなし

**理由**: カシオSR-S4000固有のCSV形式を、売上取込みと PLU 占有 snapshot の二つの明示 mode で変換する。レジ移行時にはこのモジュールだけ差し替える

**【データ構造】**

入力: 生バイト列

出力: ParseResult
- settlement_date: String（YYYY-MM-DD）
- parsed_rows: Vec<ParsedRow>（line_no, normalized_jan, name, quantity: i32, amount: i32）
- parse_errors: Vec<ParseError>（line_no, error_type, error_message）
- 占有 mode: Vec<PluRegisterSlot>（memory_no, raw_code）を全 **5,000 行**分返す

**【処理構造】**

※ 詳細はdb-design/pos-tables.mdの「B-1: Z004パース仕様 Stage 1: Parse」に記載。ここでは処理の流れのみ。

1. CP932 strictデコード（失敗 → ParseError返却、parsed_rows空）
2. 改行正規化（\u0085 / \r\n / \n / \r）
3. 1行目からYYYY-MM-DD抽出 → settlement_date（失敗 → エラー）
4. 2行目スキップ（ヘッダ）
5. 3行目以降を5フィールドCSVパース
   - フィールド数不正 → parse_errorsに追加、次の行へ
   - quantity/amountを整数パース（3桁区切りのカンマを受理）。失敗 → parse_errorsに追加
   - JAN正規化（末尾アルファベット除去→13桁化）。コードが全桁ゼロか13桁JANにならない行は、個数・金額とも0なら除外、売上があれば parse_errors に追加（[23 §13.4.2](../function-design/23-io-z004-parser.md)、D-103）
6. ParseResultを返す

**【制御構造】**
- ステートレス。入力バイト列を受け取り、結果を返すだけ
- 行単位エラーは他の行をブロックしない
- 占有 mode は layout A preamble / header 検査を再利用し、売上列・JAN 妥当性を評価しない。13桁 + E は正規化し、8桁 + E×6 は raw のまま返す。データ行数が 5,000 でなければ ImportError で全体を拒否する（IO-02-D1 / SPEC-PLS-D2）

---

### IO-03: 商品マスタCSVインポーター

**タスク要求**: 利用者が作成したCSVファイルを読み込み、構造化データに変換する

**理由**: 初期データ投入（4000商品）やマスタ更新で、Excelで作ったCSVを取り込む。エンコーディングの違いを吸収する

**【データ構造】**

入力: 生バイト列

出力: ImportParseResult
- headers: Vec<String>
- rows: Vec<HashMap<String, String>>（ヘッダ名→値のマップ）
- parse_errors: Vec<ParseError>

**【処理構造】**

1. エンコーディング判定
   - 先頭3バイトがBOM（0xEF 0xBB 0xBF）→ UTF-8としてデコード（BOMは除去）
   - それ以外 → CP932としてstrictデコード
   - デコード失敗 → エラー
2. 改行で分割（\r\n / \n / \r）
3. 1行目をヘッダとしてパース（カンマ区切り）
4. 2行目以降をデータ行としてパース
   - フィールド数がヘッダと一致しない → parse_errorsに追加
5. ImportParseResultを返す

任意列 `PLU対象` は raw string のまま保持する。`1` / `0` / 空欄の意味付けと JAN warning は BIZ-01 の責務（IO-03-D1 / SPEC-PLS-D6）。

**【制御構造】**
- ステートレス
- ヘッダ検証（必須列の確認）はBIZ-01側の責務

---

### IO-04: PLUフォーマッター

**タスク要求**: `plu_slots` 由来の memory No. を持つ product 行を、CV17 1.1.1 が受理する CP932・タブ区切り **11 列**へ変換する（IO-04-D2、D4 / SPEC-PLS-D3、D5。clear 行は D-117 で外す）。

> **runtime の状態**: 上は D-117 の後の形（採用済み）。今の build は Clear 行を書く（`src-tauri/src/constants.rs` の `PLU_CLEAR_ROW_ENABLED = true`、`io/plu_formatter.rs` の `PluExportRowKind::Clear`）。外すのは PLU の書出しの見直しの runtime（[33](../function-design/33-biz-plu-export-service.md) §16 の冒頭の「runtime の状態」の注記）。

**データ構造**: `PluExportRow` は `memory_no` を持つ。memory No. は 217〜5000 の入力値を 6 桁ゼロ埋めし、行順から生成しない。

**処理構造**:
- product 行は 13 桁 JAN、加工済み名称、単価、課税方式、固定列、部門リンクを既存 adapter 規則で出力する
- external / free slot は入力・出力しない
- header / 列順、CP932、CRLF、商品名加工、tax mapping は [25-io-plu-formatter.md](../function-design/25-io-plu-formatter.md) を正本とする

**制御構造**: IO-04 は純粋変換だけを行う。slot 予約、status 遷移、Diff / Full の行選択は BIZ-04 が所有する。

---

### IO-05: レポートCSVエクスポーター

**タスク要求**: 売上集計データをCSVファイルに変換する

**理由**: 売上レポートをExcelで開いたり、会計ソフトに取り込んだりする利用者のニーズに対応

**【データ構造】**

入力: Vec<Vec<String>>（行列データ）+ Vec<String>（ヘッダ）

出力: CSVファイルバイト列（UTF-8 BOM付き）

**【処理構造】**

1. ヘッダ行を書き込み
2. データ行を書き込み（カンマ区切り、ダブルクォート囲み）
3. UTF-8 BOM（0xEF 0xBB 0xBF）を先頭に付与
4. バイト列を返す

**【制御構造】**
- ステートレス

---

### IO-06: 画像ファイル管理

**タスク要求**: レシート画像の保存とパス管理を行う

**理由**: 返品・交換記録にレシート画像を添付する機能（REQ-202）の基盤

**【データ構造】**

入力: 画像バイト列 + ファイル名

出力: 保存先の相対パス（例: images/receipts/2026-03-21_001.jpg）

**【処理構造】**

1. 保存ディレクトリの確認（なければ作成）
2. ファイル名の生成: {日付}_{連番}.{拡張子}
3. アプリデータフォルダ/images/receipts/ に保存
4. 相対パスを返す（DBにはこの相対パスを記録）

**【制御構造】**
- ファイル名の連番は同日内でインクリメント

---

### IO-07: POS日報bundleパーサー

**タスク要求**: CASIO SR-S4000 adapter の Z001/Z002/Z005 ファイル束を受け取り、アプリ内部の日報サマリ・支払集計・部門別集計データに変換する。純粋なフォーマット変換のみ、業務ロジックなし

**理由**: current operation の日報主入力は Z004 ではなく Z001/Z002/Z005 である。レジ依存の文字コード、改行、メタ行、ファイル名、列構造を IO adapter に閉じ、BIZ/UI/DB は stable app-internal daily report model を扱えるようにする

**【データ構造】**

入力:
- Vec\<DailyReportSourceFile\>（filename, bytes）

出力:
- DailyReportParseResult
  - report_date: String（YYYY-MM-DD）
  - source_files[]: source_file（Z001/Z002/Z005）, filename, file_hash, size_bytes
  - summary_lines[]: line_key, label, amount?, quantity_hundredths?, count?, sort_order
  - payment_lines[]: payment_key, label, amount?, count?, sort_order
  - department_lines[]: raw_department_name, normalized_department_name?, amount, quantity_hundredths?, count?, sort_order
  - parse_errors[]: source_file?, filename?, line_no?, error_type, error_message（BIZ-08の開発者向けdiagnostic logで消費し、利用者向けwire/operation logへraw detailを出さない）
  - settlement_no?: 3 本とも精算回数を読め、そろうときだけの束の精算回数（IO-07-D5、BIZ-08-D4 の照合に使う）

**【処理構造】**

1. ファイル名または内容から adapter 内 source（Z001/Z002/Z005）を判定する
2. 3 source が1つずつ揃っていることを確認する。欠損・重複・未知sourceは parse_errors にする
3. 各ファイルを CP932 strict decode する
4. CP932 decode 後に改行を `\u0085` / CRLF / LF / CR で正規化する
5. sourceごとの行構造を parse する
   - Z001 → summary_lines
   - Z002 → payment_lines
   - Z005 → department_lines
6. 3 source で report_date が一致することを parse result に含める。一致しない場合は parse_errors にする。精算回数を読めたファイルが 2 本以上で値が違えば `settlement_mismatch` を parse_errors にする（IO-07-D3）
7. 生バイトから個別hashとbundle_hash素材を作る。bundle_hashの確定はBIZ-08で安定順に束ねて行う

summary/payment/departmentのsourceは格納先から一意に決まるため行ごとには重複保持しない。入力filenameはsource file metadataに加え、source判定前に失敗してmetadataへ入らないunknown fileを識別するparse error provenanceとして診断専用に保持する（IO-07-D1）。

**【制御構造】**
- ステートレス。DBを呼ばない
- CASIO 固有の表記、列位置、メタ行、改行、文字コードはこの層で吸収する
- app core が使う値は line_key / label / amount / quantity_hundredths（個数の100倍の整数、IO-07-D2）/ count / department label に正規化して返す。line_key / payment_key はラベルだけで決め、「レコード」列（行の位置）を鍵に使わない（IO-07-D4）

---

### IO-08: EJパーサー

**タスク要求**: CASIO SR-S4000 が SD に保存する電子ジャーナル（EJ）1 file の生バイト列を受け取り、記録（取引・取引中止・入金 / 出金 / 替・領収書・精算票・点検票・設定・練習）の列へ元 file の行番号つきで構造復元する。記録の種類は見出しのモード欄ではなく本文の行で決める。通常販売と返品の取引は、文法に一致し記録内の照合がそろう場合に限り明細と値引きへ復元する。純粋なフォーマット変換のみ、業務ロジックなし

**理由**: もとは EJ を PLU 販売の本番開始の前提にするため（[時点証拠 ADR SPEC-STK-TIME-D5](../adr/2026-09-18-stocktake-time-evidence.md#spec-stk-time-d5-ejの完全と商品を判定可能を分ける)、superseded）。D-118（2026-10-16）で EJ は在庫に使わず、レシートの控えの閲覧に使う（[ADR SPEC-STK-CNT-D11](../adr/2026-10-16-stocktake-count-placement.md)）。閲覧でも誤った明細を見せないため、一致しない取引を記録単位の復元不能として返す。レジ依存の固定幅・ラベル・モード欄を IO adapter に閉じる（D-023）

**【データ構造】**

入力:
- 生バイト列（CP932、CRLF、BOM なし、各行 24 バイト固定幅）。file 名・現在時刻は受け取らない

出力:
- EjParseResult
  - file_hash: String（生バイトの SHA-256、小文字 hex 64 文字）
  - leading_lines[]: 最初の記録ヘッダより前の行（line_no, text, kind）
  - records[]: header_line_no, mode（見出しのモード欄: Normal / Return / Training / Settlement / Inspection / Program / Program1 / Program3 / Off / Unrecognized）, kind（本文で決める記録の種類: Sale / Cancelled / CashMovement / Receipt / Settlement { report, completed } / Inspection / Settings / Training / Unclassified）, printed_at（分精度の文字列）, number_prefix, number（先頭 0 を保つ文字列）, body[]（line_no, text, kind）, restoration（Restored { items[], item_count, adjustments[] } / NoItems / Unresolved { reasons[] }）
  - items[]: line_no, name, quantity（戻の印の明細は -1）, unit_price?, amount（印字の符号のまま）
  - adjustments[]: line_no, kind（ItemDiscount { item_line_no } / SubtotalDiscount / MinusKey）, amount
  - diagnostics[]: line_no?, code（7 種）, scope（Line / Record / File）, message（code ごとの固定文言。行の生の文字列を含めない）

**【処理構造】**

1. 生バイトの SHA-256 を file_hash にする
2. CRLF だけで行に分割する（改行を正規化しない）
3. 各行を CP932 strict decode する。decode できない行があれば致命的エラー（幅の診断より先）
4. 2 行の記録ヘッダ（モード欄 + 日時 / 番号行）で記録を区切る。最初のヘッダより前は先頭断片
5. 記録の種類を本文の行（精算票・点検票の題、トレーニングの表示、入金 / 出金 / 替・領収書・取引中止の形）で決め、本文に題が無いときだけ練習・設定をモード欄で決める。記録の種類と記録内の位置（区切りの前 / 後）で行を分類する。24 バイトでない行、または 24 バイトの中に孤立した CR / LF を含む行と、最終改行なしは診断にし、どれにも当たらない行は Unknown として残す
6. 取引の記録は、訂正（直前の明細・値引き・支払行の取消）・値引き・折返しをまとめ、数量×単価・点数・合計（無ければ取り消されていない支払行 1 行）を記録内で照合し、そろえば明細と値引きを復元する。取引中止・入金 / 出金 / 替・領収書・精算票（次のヘッダで閉じた中断を含む。file の最後の記録で終わりの印字が無ければ復元不能）・点検票・設定・練習は明細なし
7. 一致しない記録は、その記録だけを復元不能にして診断を積む

詳細は [function-design/29-io-ej-parser.md](../function-design/29-io-ej-parser.md)（IO-08-D1〜D10）。

**【制御構造】**
- ステートレス。DBを呼ばない
- 致命的エラーは 3 種（DecodeFailed / NoRecords / Empty）で部分結果を返さない。それ以外は記録単位の復元不能 + 診断
- EOF を記録の閉じの根拠にしない。復元不能の記録から明細を読み出す API を持たない
- 番号の連続・区間の完全性・分割 file の連結・Z004 との対応・商品同定・時刻の解釈は行わない（BIZ。区間の対応づけと欠けの定義の記録は時点証拠 ADR SPEC-STK-TIME-D5a〈superseded。D-118 で在庫には使わない〉）

---

### IO-09: レジの SD の列挙と読取り

**タスク要求**: レジ（CASIO SR-S4000）の SD を取外し可能な drive から探すか利用者が選んだ folder から解決し、`CASIO\SR500_550_4000\XZ` と `XZ_BKUP` の売上の file（Z・EJ）の名前を列挙し、指定された file の生バイトを読取り専用で読んで形（CP932 strict・BOM・CRLF だけ・最終改行）を測る。手で選んだ file の path が自動で見つけた SD の root の下にあるかを path だけで決め（`locate_in_register_sd_roots`）、下に無い file がその時点で在るかを metadata で確かめる（`check_selected_file_present`）。SD へは書かない。業務ロジックなし（function-design は [29 §29.7](../function-design/29-io-daily-report-parser.md#297-io-09-レジの-sd-の列挙と読取りd-111)）

**理由**: 毎日の売上データは CV17 の取込みに依らずにアプリが SD から直接読む（D-111）。SD の配置・名前・読み方はレジ依存なので IO adapter に閉じる（D-023）。どの file を候補にするか・取込み済みかは BIZ-08（§37.9）が決める

**【データ構造】**

入力: なし（自動）または選ばれた path、窓の始まりの日付、相対 path と上限 byte 数、手で選んだ file の path と root の列

出力: RegisterSdRoot（path, volume_label）、RegisterSdListing（entries[]{relative_path, area〈Sales = XZ / Backup = XZ_BKUP〉, kind〈Z{series, day, suffix, backup_seq?} / Ej{backup_seq?} / Unknown〉, date?, size_bytes}, backup_area_present）、RegisterSdFile（relative_path, bytes, shape）、RegisterSdLocatedFile（root, relative_path）。「自動か選んだ場所か」の選択は BIZ-08 の `DailyReportSdSelection` で、IO-09 の型ではない

**【制御構造】**
- 使う file system の API を読取りの許可の列（`File::open`・`read_dir`・`metadata` 等）に限り、書く系（`OpenOptions`・`File::create`・`fs::copy`・`fs::write` 等）を使わない（IO-09-D3）
- 名前は分類と組分けにだけ使い、取込み済みの判定に使わない（IO-09-D2）
- 列挙・読取りの途中の失敗は全体の失敗にする

---

### IO-10: SD から読んだ原本の写しの保存

**タスク要求**: 取り込む束の file の生バイトを、アプリのデータ folder の `pos-sources/casio-sr-s4000/sd/{SD の相対 path}` に書く。同じ bytes があれば書かず、違う bytes があれば上書きせずに hash を付けた名前にする。保存先の directory を作ってから一時 file に書いて rename する。SD には書かない。業務ロジックなし（function-design は [29 §29.8](../function-design/29-io-daily-report-parser.md#298-io-10-sd-から読んだ原本の写しの保存d-111)）

**理由**（D-111）: (a) の段階（SD 直読みの runtime の lane）では SD は動かさず、CV17 の取込みをやめると PC 側に原本の写しが無くなるため（owner 決定 2026-10-06、D-111）。いつ書くか・失敗の扱いは BIZ-08-D5
