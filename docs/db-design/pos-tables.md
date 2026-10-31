# テーブル定義（POS連携）

## 計数の置き場と在庫の求め直しの契約（D-118。(a) と b1 で実装済み）

[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)（SPEC-STK-CNT-D4・D9）の POS の側。既存の csv_imports の status の集合、sale_records と日報の分離は変えない。旧 ADR（時点証拠）の `pos_import_sources`（資料の受領・同一性の拒否の証拠・`first_scan_id`）と `pos_source_scans` は作らない（D-109 で `#[cfg(test)]` に置いた schema は b1 で外した）。(a)（ADR SPEC-STK-CNT-D12）で入ったのは最後の項目の移行の止まりで、表の列とほかの項目は b1（[Plan Packet](../archive/plans/2026-10-22-stocktake-count-runtime-b1.md)、migration v11）で入った。

| 保存先 | 足す列 | 制約と意味 |
|---|---|---|
| csv_imports | machine_no TEXT | NOT NULL（新しい取込み）。IO-02 が Z004 のメタから読んだマシンNo.（leading zero を保つ文字列） |
| csv_imports | settlement_no INTEGER | NOT NULL（新しい取込み）。IO-02 が読んだ精算回数を 10 進の整数にした値（0 以上） |
| sale_records | stock_linked INTEGER | NOT NULL、0/1 の CHECK。確定の TX で、その行の商品の取込みのときの `pos_stock_sync` を写す（共有 JAN の行と商品の無い行は 0）。後から書き換えない。ADR D3 の理由 1 と D8 の在庫連動を始める検査が、今の設定でなくこの値で判定する |

- 精算回数・マシンNo. を読めない Z004（従来の形）は取り込まない（preview で拒む。[32](../function-design/32-biz-csv-import-service.md) の新契約）。
- 同じ machine_no・settlement_no の active な取込み（completed / completed_partial）は 1 つだけ（BIZ が preview と確定の TX で検査する。取り消した取込みは比べない）。DB の UNIQUE は張らない（取消の後の取り込み直しで同じ番号の行が複数になる）。
- 精算日と精算回数の順の食い違いと欠けた精算回数の知らせは active な取込みの `settlement_date`・`settlement_no` から、別の machine_no は取り消したものを含む全ての Z004 の取込みの `machine_no` から、BIZ が判定する（新しい表を持たない。[32](../function-design/32-biz-csv-import-service.md) の表の順 2）。
- 確定で作る Z004 の movement は `business_date = settlement_date`、`phase = 1`、`settlement_no` を持つ（[tracking](tracking-system-tables.md) の同名の契約）。
- 取込みID（`csv_imports.id`、AUTOINCREMENT）は使い回さない（取消は status を変えるだけで行を消さない）。fresh な計数と棚卸しの完了は、保存・完了の時点の `MAX(csv_imports.id)` を取込みのカーソル（tracking の `stock_counts.seen_import_id`・`stocktakes.cutoff_import_id`）に残し、精算回数ではなく取込みID で「保存・完了の前に取り込んだ精算か」を決める（ADR D3・D6。精算回数は取消の後に別の取込みが再び使う。b1 の Plan Review closure〈Codex〉の是正）。
- 移行: 既存の取込みがある DB では runtime の migration は止まる（(a) の migration v9 で実装済み。tracking の「移行」。開発・試験 DB はリリースの前まで作り直してよい。D-123）。`csv_imports` の 2 列と `sale_records.stock_linked` は b1 の migration v11 が足した（[22 §20](../function-design/22-mnt-migration.md)、ADR SPEC-STK-CNT-D12）。

> **親文書**: [DB_DESIGN.md](../DB_DESIGN.md)

---

## 12. csv_imports（CSV取込み履歴）

### 役割
Z004ファイルの取込み履歴。重複取込み防止、ロールバック管理の基盤。

### カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | 取込みID |
| filename | TEXT | NOT NULL | ファイル名（例: Z004_260321） |
| settlement_date | TEXT | NOT NULL | 精算日（YYYY-MM-DD） |
| file_hash | TEXT | NOT NULL, INDEX | ファイルのSHA-256ハッシュ。重複チェックはアプリ側で status IN ('completed','completed_partial') と組み合わせて判定 |
| total_items | INTEGER | NOT NULL | 取込み件数（正常に紐付けできた商品数） |
| total_amount | INTEGER | NOT NULL | 売上合計金額（正常取込み分のみ） |
| skipped_count | INTEGER | NOT NULL, DEFAULT 0 | スキップされた行数（マスタ未登録＋行エラー） |
| status | TEXT | NOT NULL, CHECK(status IN ('completed','completed_partial','rolled_back')) | 状態 |
| imported_at | TEXT | NOT NULL | 取込み日時（YYYY-MM-DDTHH:MM:SS） |
| machine_no | TEXT | CHECK(machine_no IS NOT NULL AND machine_no <> '') | マシンNo.（leading zero を保つ文字列。v11、D-118 の b1。冒頭の D-118 の節） |
| settlement_no | INTEGER | CHECK(settlement_no IS NOT NULL AND settlement_no >= 0) | 精算回数（v11、D-118 の b1） |

### statusの値

| 値 | 意味 | 発生条件 |
|---|------|---------|
| completed | 全行正常取込み | error_rows=0 |
| completed_partial | 一部スキップあり | error_rows>0 |
| rolled_back | 取込み取消済み | 利用者が巻き戻し操作 |

**Parse失敗時の扱い（2026-03-29 確定）**: Parse段階（Stage 1）で失敗した場合、csv_importsにレコードを作成しない。Parse失敗はファイル形式の問題であり、取込み処理自体が開始されていないため。失敗の記録はoperation_logs（operation_type='csv_import_parse_failed'）に残す。同じファイルの再取込みはfile_hashで制御しないため、ファイルを修正して再試行可能。

### 設計意図
- **file_hashの理由（SP-401-12）**: 同じファイルを誤って2回取り込むと在庫が二重に減る。ハッシュ値で同一ファイルを検知してブロック
- **statusの3値設計（B-4対応、2026-03-29確定）**: Parse失敗時はcsv_importsにレコードを作成しない方針のためfailedは不要。completed_partialは「部分成功」を正式に認める仕組み
- **skipped_countの理由**: CSV取込み完了画面で「3件の商品がスキップされました」と表示するため。詳細はcsv_import_errorsテーブルに記録
- **total_items / total_amountの理由**: CSV取込み画面の完了サマリに表示する値。正常取込み分のみをカウント

### 困りそうなケースと対応方針（2026-03-28 確定）

**ケース1: 同じファイルを2回取り込もうとした**
- file_hashが一致 ＋ statusが'completed'または'completed_partial' → ブロック（「取込み済みです」）
- file_hashが一致 ＋ statusが'rolled_back' → 許可（ロールバック後の再取込み）
- 重複チェックSQL: `WHERE file_hash = ? AND status IN ('completed','completed_partial')`

**ケース2: 同じ精算日の別ファイル（精算前後に2回CSV出力した等）**
- settlement_dateが同じ ＋ 別のfile_hash ＋ 既存がcompleted/completed_partial → 既存分を残す追加取込みとして確認する
- 確認時は同日のactive importを全件表示し、承認後は新規分だけINSERTする
- byte違いで同内容の復旧用ファイル等を選んでいないかをoperatorが確認し、キャンセル時は何もしない
- D-118 の runtime からは、Z004 のケース2・3は精算回数で決める（上の「計数の置き場と在庫の求め直しの契約」）: 同じ machine_no・settlement_no の active があれば別 hash でも拒否し、別の精算回数なら確認なしに追加する。利用者確認は日報の側（§12b 以降）だけに残る（D-071 の Superseded in part の行）

**ケース3: ファイル名が同じだが中身が違う（同じ日に2回精算した等）**
- file_hashが異なるためケース2として処理される

**ケース4: completed_partialの取込みを完全にしたい（スキップ商品を後から登録した）**
- 現在のcsv_importをロールバック→同じファイルで再取込み。今度はmatchedになる
- csv_import_errorsの情報から「何を登録すればいいか」が追える

**file_hashのUNIQUE制約の修正**: 当初UNIQUE制約を付けていたが、ロールバック後の再取込みで同じハッシュのレコードが2件できる可能性がある（rolled_back + completed）。UNIQUE制約を外し、アプリ側でチェックに変更する

`settlement_date` は集計・表示のgroup keyでありuniqueness keyではない。active identityは `file_hash + active status` で判定する。訂正は対象import IDのrollbackと再取込みを明示した2操作で行う。

---

## 12a. csv_import_errors（CSV取込みエラー・スキップ行）

### 役割
CSV取込み時にスキップされた行（マスタ未登録、フォーマットエラー等）を保存する。利用者が「何を直せばいいか」を追跡するためのテーブル。

### カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | ID |
| csv_import_id | INTEGER | FK → csv_imports.id, NOT NULL | 親の取込みレコード |
| source_line_no | INTEGER | NOT NULL | Z004のレコード番号 |
| normalized_jan | TEXT | NULLABLE | 正規化後のJANコード（正規化前にエラーならNULL） |
| raw_name | TEXT | NOT NULL | Z004上の商品名（そのまま保存） |
| raw_quantity | TEXT | NOT NULL | Z004上の個数（文字列。値の出どころは設計意図の「raw_quantity / raw_amountの値」） |
| raw_amount | TEXT | NOT NULL | Z004上の金額（同上） |
| error_type | TEXT | NOT NULL, CHECK(error_type IN ('unmatched_product','invalid_format','invalid_jan','invalid_number')) | エラー種別 |
| error_message | TEXT | NOT NULL | エラー詳細（例: 「JAN 4973167064078 に該当する商品がありません」） |
| created_at | TEXT | NOT NULL | 作成日時（YYYY-MM-DDTHH:MM:SS） |

### error_typeの値

| 値 | 意味 | 発生段階 |
|---|------|---------|
| unmatched_product | マスタ未登録商品 | Validate |
| invalid_format | フィールド数不正等 | Parse |
| invalid_jan | JAN正規化不能（売上のある行だけ）、または商品コードの無い枠（PLU の登録を消した枠、コードを持たない通常 PLU）の売上（D-103） | Parse |
| invalid_number | 個数・金額が数値でない | Parse |

### 設計意図
- **raw_quantity / raw_amountをTEXTで持つ理由**: 数値変換に失敗した行も保存するため。INTEGERだと変換失敗時に保存できない
- **raw_quantity / raw_amountの値**: IO-02 の ParseError 由来の行（invalid_format / invalid_jan / invalid_number）は、取得できた field の元の文字列（数値変換前。カンマ付きのまま）。field が 5 未満の invalid_format で取得できなかった field は `""`。unmatched_product など BIZ-03 が整数から作る行は整数の文字列表現（金額 `1,234` の行は `"1234"`。[32 §15.3](../function-design/32-biz-csv-import-service.md) 手順 4b / 4c、D-103）
- **normalized_janがNULLABLEな理由**: JAN正規化自体が失敗した場合（invalid_jan）は正規化後の値がない
- **error_messageの理由**: 利用者向けの日本語メッセージをそのまま保存。画面表示時に再生成しなくて済む

---

## 12b. daily_report_imports（日報取込み履歴）

### 役割
Z001/Z002/Z005 の1営業日分ファイル束を1つの日報取込みとして管理する。日報取込みは売上集計の正本だが、商品別明細ではないため `sale_records` とは分ける。

### カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | 日報取込みID |
| report_date | TEXT | NOT NULL | 対象営業日（YYYY-MM-DD） |
| source_adapter | TEXT | NOT NULL, CHECK(source_adapter IN ('casio_sr_s4000')) | 取込み元adapter。外部レジ差し替え時の境界 |
| bundle_hash | TEXT | NOT NULL, INDEX | Z001/Z002/Z005の生バイトhashを安定順で束ねたSHA-256。重複取込み判定に使う |
| settlement_no | INTEGER | NULLABLE | 束の精算回数（3 本とも読めてそろうときだけ。IO-07-D5）。同じ精算を別の bytes で二重に取り込まない照合に使う（BIZ-08-D4、D-111。列を足す migration は [22 §17](../function-design/22-mnt-migration.md#17-mnt-03-追加-日報の束の精算回数の列の-migrationd-111)。既存行は NULL） |
| source_files_json | TEXT | NOT NULL | ファイル名、個別hash、サイズ、adapter内source名（Z001/Z002/Z005）のJSON。SD の file（SD の候補からの束と、手で選んだ SD 上の file）は、各要素に SD の相対 path（`sd_relative_path`）と PC 側の写しの app data からの相対 path（`copy_path`、BIZ-08-D5）も持つ（無い要素も読む）。BIZ-08 の内部の serialize 用の型で書き、wire 型 `DailyReportSourceFileInfo` に field を足さない（37 §37.4 手順 6）。実CSV本文は保存しない（写しは DB の外の file） |
| gross_amount | INTEGER | NULLABLE | Z001/Z005から導出できる総売上額。未確定・欠損時はNULL |
| net_amount | INTEGER | NULLABLE | 返品・値引等を反映した日報上の純売上額。未確定・欠損時はNULL |
| status | TEXT | NOT NULL, CHECK(status IN ('completed','rolled_back')) | 状態 |
| imported_at | TEXT | NOT NULL | 取込み日時（YYYY-MM-DDTHH:MM:SS） |
| rolled_back_at | TEXT | NULLABLE | 取消日時（YYYY-MM-DDTHH:MM:SS） |
| note | TEXT | NULLABLE | operator向け補足、adapter警告要約 |

### 設計意図
- **bundle_hashの理由**: Z001/Z002/Z005は3ファイルで1日報を構成するため、個別ファイルではなく束単位で同一取込みを検知する。
- **source_files_jsonの理由**: 外部adapterの証跡（ファイル名、個別hash、サイズ、source名）を残すが、実店舗のCSV本文や金額明細をそのまま保存しない。検索・集計に使う値は下位行テーブルへ正規化する。
- **statusの2値設計**: Parse/Validate失敗時は取込みレコードを作らない。成功後の取消は物理削除せず `rolled_back` にする。

### 冪等性・同日追加方針
- `bundle_hash` が一致し `status='completed'` の取込みがある場合はブロックする。
- `report_date` が一致し別 `bundle_hash` の `completed` がある場合は、既存全件を提示して追加確認を要求する。承認後も既存parentを変更せず、新規取込みだけを作る。
- `report_date` と `settlement_no` が一致し別 `bundle_hash` の `completed` がある場合はブロックする（追加確認で通さない。BIZ-08-D4）。`settlement_no` が NULL の行・束は照合しない。SD から取り込む束は `settlement_no` を持つ束だけなので（BIZ-08-D6）、SD の経路の行は NULL にならない。CV17 が SD の原本を移して複製するだけで bytes を作り直さないことは未確認の前提（29 §29.7.2 の SD-23）なので、bytes の一致だけに頼らず精算回数でも照らして、同じ精算を二重に数えないための照合。
- `rolled_back` の同一bundleは再取込み可能。
- `report_date` はgroup keyであってuniqueness keyではない。訂正は対象parent IDのrollbackと再取込みを明示した2操作で行う。

---

## 12c. daily_report_summary_lines（日計サマリ行）

### 役割
Z001由来の日計サマリを、アプリ内部の表示・照合に使える行データとして保存する。外部ファイルの列名をそのまま業務ルールにしないため、adapterが `line_key` / `label` / 数値項目へ正規化する。

### カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | 行ID |
| daily_report_import_id | INTEGER | FK → daily_report_imports.id, NOT NULL | 親の日報取込み |
| source_file | TEXT | NOT NULL, CHECK(source_file IN ('Z001')) | adapter内source名 |
| line_key | TEXT | NOT NULL | adapterが正規化した行キー（例: gross_sales, net_sales, tax_10） |
| label | TEXT | NOT NULL | operator表示用ラベル |
| amount | INTEGER | NULLABLE | 金額。該当しない行はNULL |
| quantity_hundredths | INTEGER | NULLABLE | 個数の100倍の整数（`1.3` 個 → 130。小数 2 桁まで、IO-07-D2）。総売の行だけが持ち、該当しない行はNULL。migration v7 で `quantity` から改名し既存値を100倍した |
| count | INTEGER | NULLABLE | 件数（整数）。該当しない行はNULL |
| sort_order | INTEGER | NOT NULL | 表示順 |

### 設計意図
- Z001の4列データや事前行はCASIO adapterの事実であり、app coreは日報サマリ行として扱う。
- `amount` / `quantity_hundredths` / `count` をNULL許容にすることで、レジ側の行意味が金額・数量・件数のどれかに偏っても同じ行モデルで保持できる。
- **個数を100倍の整数で持つ理由（D-104）**: 部門キーで小数の数量（レジは小数 2 桁まで）を打った日は総売の個数が小数になる。整数のままだとその日の日報の取込みが丸ごと失敗していた。REAL は月次の `SUM` で誤差が積もり、TEXT は `SUM` できないので、固定小数の整数にする。列名に `_hundredths` を付け、単位の数として読む古い SQL が黙って100倍の値を出さないよう、改名で読み手を壊して直させる。wire へは BIZ が単位の数（`f64`）に戻して出す。

---

## 12d. daily_report_payment_lines（取引キー・支払集計行）

### 役割
Z002由来の取引キー・支払・現金等の集計行を保存する。日報の支払内訳やレジ締め確認に使うが、商品別売上や在庫変動には使わない。

### カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | 行ID |
| daily_report_import_id | INTEGER | FK → daily_report_imports.id, NOT NULL | 親の日報取込み |
| source_file | TEXT | NOT NULL, CHECK(source_file IN ('Z002')) | adapter内source名 |
| payment_key | TEXT | NOT NULL | adapterが正規化した支払・取引キー |
| label | TEXT | NOT NULL | operator表示用ラベル |
| amount | INTEGER | NULLABLE | 金額 |
| count | INTEGER | NULLABLE | 件数 |
| sort_order | INTEGER | NOT NULL | 表示順 |

### 設計意図
- 改行・文字コード処理はIO parserで閉じる。持ち帰った Z 帳票と SD の file は CRLF だけで、NEL / raw 0x85 の行終端の実例は無い（2026-10 の見直しの事実の一覧。0x85 は CP932 の 2 バイト文字の中にだけ現れる）。正規化は防御として残す。
- 支払・取引キーはreporting/accounting意味を持つが、在庫引落しの根拠にはならない。
- **同じ日の取込みをまたいで足す鍵（proposed・未実装、D-120）**: `payment_key` は保存を続けるが、足す鍵には使わない。日次売上の支払集計と日報の紙は、位置（`sort_order`）と名称（`label` の前後の空白を除いた文字列）の組で見分けて足す（行がずれ、1 本の中に同じ名称が 2 行ある日は足さない。規則の正本は [34](../function-design/34-biz-sales-service.md) SALES-SHEET-D2、画面の側は [34](../function-design/34-biz-sales-service.md) §19.3 手順 3。IO の [24](../function-design/24-io-csv-import-repo.md) §14.21 手順 3 は行を足さずに返す）。並び順から作る `payment_N` で足すと、精算の間で名称が変わった日に別の項目を 1 行に足す。§12c の Z001 と §12e の Z005 の行も、日報の紙では同じ組で足す（日次売上の画面の Z001 は取込みごと〈D-096〉、Z005 は部門ごと〈24 §14.21 手順 4〉のまま）。列・制約・migration は変えない。

---

## 12e. daily_report_department_lines（部門別売上集計行）

### 役割
Z005由来の部門別売上を保存する。日次・月次レポートの部門合計や、REQ-403の将来照合候補として使う。

### カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | 行ID |
| daily_report_import_id | INTEGER | FK → daily_report_imports.id, NOT NULL | 親の日報取込み |
| source_file | TEXT | NOT NULL, CHECK(source_file IN ('Z005')) | adapter内source名 |
| department_id | INTEGER | FK → departments.id, NULLABLE | app部門マスタに対応できた場合のID |
| raw_department_name | TEXT | NOT NULL | Z005上の部門名または部門表示 |
| normalized_department_name | TEXT | NULLABLE | adapterが正規化した部門名 |
| amount | INTEGER | NOT NULL | 部門別売上金額 |
| quantity_hundredths | INTEGER | NULLABLE | 部門別個数の100倍の整数（小数 2 桁まで、IO-07-D2）。取得できない場合はNULL。migration v7 で `quantity` から改名し既存値を100倍した |
| count | INTEGER | NULLABLE | 部門別件数（整数）。取得できない場合はNULL |
| sort_order | INTEGER | NOT NULL | 表示順 |

### 設計意図
- `department_id` はNULL許容にする。レジ側部門名とアプリ部門マスタが一致しない日でも日報は取り込めるようにし、未対応は警告として表示する。
- 部門別日報は公式日報集計の根拠だが、商品別数量・在庫減算を復元できない。`sale_records` への展開は禁止する。
- `quantity_hundredths` は §12c と同じ100倍の整数（D-104）。月次の部門集計（`SUM`）も100倍の整数のまま足し、BIZ-05 が wire で単位の数に戻す。部門売りの小数は在庫に効かないので、単位の換算（cm 等）はしない。

---

## SD の一括取込みと EJ の控えの契約（proposed・未実装、D-121）

EJ の記録の文をレシートの控えとして持つ表（BIZ-08-D9、[37 §37.11](../function-design/37-biz-daily-report-import-service.md)）。migration は後続の runtime の lane（backlog の「SD の一括取込みと EJ の控えの runtime（A）」）が、その時点の版の番号で足す。`sale_records`・`inventory_movements`・日報の表と結ばない（EJ は在庫と売上の集計に使わない。ADR SPEC-STK-CNT-D11）。原価の列を持たないので、原価の lane の vB の起点の検査（[22 §16](../function-design/22-mnt-migration.md)）の対象の表にならない。DB の 1 file に入るので backup・restore の対象になる。

### ej_records カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | 記録の ID |
| machine_prefix | TEXT | NOT NULL, CHECK(length(machine_prefix) = 4) | 記録の見出しの 4 桁の欄（IO-08 の `number_prefix`。Z004 の machine_no と同じ値を観測） |
| record_no | TEXT | NOT NULL, CHECK(length(record_no) = 6) | 記録の 6 桁の番号（IO-08 の `number`。先頭 0 を保つ）。UNIQUE(machine_prefix, record_no) |
| printed_at | TEXT | NOT NULL, INDEX | 記録の日時（`YYYY-MM-DD HH:MM`、レジの時計のまま） |
| mode | TEXT | NOT NULL, CHECK(mode IN ('normal','return','training','settlement','inspection','program','program1','program3','off','unrecognized')) | 見出しのモード欄（IO-08 の `EjMode`） |
| kind | TEXT | NOT NULL, CHECK(kind IN ('sale','cancelled','cash_movement','receipt','settlement','inspection','settings','training','unclassified')) | 記録の種類（IO-08 の `EjRecordKind`） |
| settlement_report | TEXT | NULLABLE, CHECK(settlement_report IN ('daily','plu','attendance')) | 精算票の帳票（kind = settlement だけ） |
| settlement_completed | INTEGER | NULLABLE, CHECK(settlement_completed IN (0,1)) | 精算票の終わりの印字の有無（kind = settlement だけ） |
| total_amount | INTEGER | NULLABLE | 取引の合計（円）。kind = sale で明細を復元できた記録だけ（37 §37.11 手順 2） |
| body_text | TEXT | NOT NULL | 見出し 2 行と本文の行を decode したまま "\n" でつないだ文（レシートの控え） |
| imported_at | TEXT | NOT NULL | 保存の日時（YYYY-MM-DDTHH:MM:SS） |

### 設計意図
- 鍵は記録の番号で、file の hash・名前ではない（`XZ` 直下の EJ は伸び、同じ記録が `XZ` と `XZ_BKUP` に入る。IO-08.10）。
- 明細を表に分けない。控えを見るには文で足り、復元できない記録（小数の数量・未知の行）も見える。`total_amount` は一覧で探す手がかりだけ。
- insert-only。UPDATE・DELETE しない（同じ番号で文の違う記録は上書きせず知らせる。BIZ-08-D9）。取消の操作は持たない。
- 店の実データ（品名・金額）を持つ表なので、tracked の fixture は合成の EJ だけにする。

---

## B-2: Z001/Z002/Z005日報取込み仕様（2026-06-30 設計）

### 処理パイプライン: Parse bundle → Validate bundle → Preview → Commit

**入力単位**:
- 1営業日の日報取込みは `Z001`、`Z002`、`Z005` の3ファイルを必須束として扱う。
- adapterはファイル名・内容からsourceを判定し、欠損、重複、未知sourceをPreview前にエラーにする。
- CV17 1.1.1のZ001/Z002/Z005は、SD の `XZ_BKUP` とツール内部ディレクトリ常在ファイル（同じ bytes。取込み前の `XZ` の原本も同じ bytes であることは未確認の前提。29 §29.7.2 の SD-23）の layout A（7行プリアンブル、1行ヘッダ、4列データ行）と、エクスポート機能出力の layout B（先頭メタフィールド、ヘッダ、4列反復の連結）を両方サポートする。adapterはどちらも `record_code, label, quantity_or_count, amount` 系の4列行へ正規化してから内部行へ変換する。
- 日付はsource/layoutにより `YYYY/M/D` または `YYYY-MM-DD` で出力されるため、IO-07で `YYYY-MM-DD` に正規化してからBIZ-08へ渡す。
- 入力の標準はアプリがレジの SD から直接読む束（IO-09、BIZ-08 §37.9、D-111）。予備は利用者が選ぶ 3 ファイル（PC の `EcrDatas` の写しを含む。`XZ_BKUP` と同じ bytes）。
- Excel帳票はsource of truthでも日別archiveでもない。店は今、SDからCV17へ取り込んだZ001/Z002/Z005を同じExcelファイル群へ毎日ほぼそのまま貼り付けて上書きするため、日別履歴は印刷・バインダーだけに残る。layout A/Bの受理はadapter互換性であってoperatorの選択肢ではない。sanitized版のExcelは数値突合に使わず、列構成・ラベル・行の並びの参照に限定する。
- PCツール上には `Z006`（グループ）、`Z009`（時間帯別）、`Z011`（担当者）も存在するが、個人店の初期日報に必要な業務用途が未確認のため、初期DBモデルには保存しない。必要性が確認された場合は adapter 入力とDB保存先を別設計で追加する。

**Stage 1: Parse bundle（IO-07に委譲）**
1. 各ファイルを生バイトで受け取り、CP932 strict decodeする。
2. 改行はCP932デコード後に `\u0085` / CRLF / LF / CR を正規化する。
3. adapter内source（Z001/Z002/Z005）ごとにプリアンブル、ヘッダ、4列データ行を解釈し、`DailyReportParseResult` を返す。
4. parse失敗時は `daily_report_imports` を作らず、operation_logsに `daily_report_parse_failed` を記録する。

**Stage 2: Validate bundle**
1. 3ファイルの対象日が一致し、精算回数を読めたファイルどうしの精算回数が一致することを確認する（精算回数の比較は IO-07 が Stage 1 で行う。IO-07-D3。不一致は `settlement_mismatch` で commit 不可）。
2. Z005の部門名を `departments.name` に照合する。未一致はエラーではなく警告にし、`department_id=NULL` でpreview可能にする。
3. 必須集計行（総売上または純売上など、adapterが定義する最低限のサマリ）が欠ける場合はcommit不可エラーにする。
4. `bundle_hash` でactive同一bundleを判定し、`report_date` でactive同日parent全件の追加確認snapshotを作る。

**Stage 3: Preview**
- 表示内容: 対象日、読み込む3ファイル、総売上/純売上、支払集計、部門別集計、部門未対応警告、同一bundleブロックまたは同日別bundle追加確認。
- 利用者は「取り込む」「ファイルを選び直す」「追加確認」を選べる。同日別bundleでは既存active parentを全件表示する。
- previewは30分有効のtokenでCMD層cacheに保持する。Z004のpreview cacheと同じAppStateを使ってよいが、型は分ける。

**Stage 4: Commit**
BEGIN TRANSACTION内で:
1. 同一active `bundle_hash` を再検査する。
2. 同一 `report_date` のactive parent ID一覧を再取得し、preview snapshotと一致することを確認する。不一致なら副作用なしで中断する。
3. 既存parentを変更せず `daily_report_imports` に1行INSERTする。
4. Z001由来行を `daily_report_summary_lines` にINSERTする。
5. Z002由来行を `daily_report_payment_lines` にINSERTする。
6. Z005由来行を `daily_report_department_lines` にINSERTする。
7. COMMIT。

COMMIT後に:
7. operation_logsに `daily_report_import` を記録する。
   - operation_logs記録失敗は日報取込み自体をROLLBACKせず、診断ログまたは後続確認対象として扱う。

**Rollback**
1. `daily_report_imports.id` で対象を取得する。
2. BEGIN TRANSACTION内で `status='rolled_back'` と `rolled_back_at` を更新する。
3. COMMIT。
4. operation_logsに `daily_report_rollback` を記録する。
   - operation_logs記録失敗はrollback済み状態を戻さず、診断ログまたは後続確認対象として扱う。
5. `sale_records` / `inventory_movements` / `products.stock_quantity` は変更しない。
6. 更新条件は指定IDだけとし、同一 `report_date` の他parentは変更しない。

### Z004との関係
- Z004はPLU登録後の商品別売上として既存B-1仕様を維持し、BIZ-03 commitは`pos_stock_sync=true`の商品に在庫変動を作成する。店舗採取layout AのIO-02二形状対応（SPEC-Z4A-D1〜D7）はDB契約を変更しない。
- Z001/Z002/Z005日報取込みの完了は、Z004取込み完了を意味しない。
- 商品別ランキング、商品別在庫自動引落し、返品のSKU単位反映はZ004または手動販売出庫が根拠になる。

---

## B-1: Z004パース仕様（2026-03-29 確定）

### 処理パイプライン: Parse → Validate → Preview → Commit

### Stage 1: Parse（バイトレベル → 構造化データ）

**ファイル読込**:
- 生バイトで読み込む
- CP932でstrictデコード（デコード失敗は即エラー → 取込み中断、operation_logsに記録）
- file_hashはデコード前の生バイトからSHA-256で計算

**改行処理**:
- CP932デコード後に改行を正規化する
- 区切り候補: `\u0085`（NEL）/ `\r\n` / `\n` / `\r`
- 理由: 生バイト0x85で切るとCP932のマルチバイト途中の誤判定リスクがある

**行構造**:
- 1行目: メタ情報（日付抽出元）
- 2行目: カラムヘッダ（読み飛ばし）
- 3行目以降: データ行
- 2行未満なら形式エラー → 取込み中断

**ヘッダ日付抽出**:
- 1行目からYYYY-MM-DDパターンを抽出 → settlement_dateに保存
- 抽出できなければ形式エラー → 取込み中断

**データ行パース**:
- ダブルクォート囲み、カンマ区切りの5フィールド固定
- フィールド: record_no / scanning_code_raw / name_raw / quantity_raw / amount_raw
- フィールド数不足/過剰 → 行単位エラー（error_type='invalid_format'）、他の行は処理継続

**JAN正規化**:
- scanning_code_rawが14桁かつ末尾がアルファベット（E等）→ 末尾を除去して13桁化
- 個数・金額を先に読み、売上の有無で行を分ける（[23 §13.4.2](../function-design/23-io-z004-parser.md) SPEC-Z4A-D8、D-103）
- コード全桁0（`00000000000000` / `0000000000000`）で個数・金額とも0 → 売上の無い枠（除外、エラーにもカウントしない）
- コード全桁0で個数か金額が0でない（PLU の登録を消した枠、コードを持たない通常 PLU）→ 行エラー（error_type='invalid_jan'、商品コードの無い枠の売上）。黙って除外しない
- 13桁JANにならないコード（8桁独自コード + `EEEEEE` 等）→ 個数・金額とも0なら除外、売上があれば行エラー（error_type='invalid_jan'）

**数値変換**:
- quantity_raw / amount_raw を整数にパース。3桁区切りのカンマ（例 `1,234`）は受理する（23 §13.4.1 SPEC-Z4A-D7）
- パース失敗 → 行エラー（error_type='invalid_number'）

### Stage 2: Validate（構造化データ → 処理対象の選別）

**空レコード除外**（エラーにもカウントしない）:
- quantity = 0 かつ amount = 0 → 除外（PLU登録あるが当日販売なし。コード全桁0の行は Stage 1 で分類済みで、ここには来ない）

**実データ判定**:
- normalized_janが有効（非ゼロ）かつ（quantity ≠ 0 or amount ≠ 0）

**マスタ照合**:
- products.jan_codeで検索（product_codeへのfallbackはしない）
- ヒット1件 → matched（紐付け成功）
- ヒット0件 → 前の JAN の別名（[master-tables](master-tables.md) §1a、D-122、proposed・未実装）があればその商品で matched、無ければ unmatched（error_type='unmatched_product' → csv_import_errorsに記録）
- ヒット複数件 → `ORDER BY product_code ASC`で先頭に紐付け（常に同じ商品に紐付く決定的ルール。先頭へ割り当てる限界は [backlog](../backlog.md) の DATA-2。共有 JAN の行を在庫連動の外〈売上だけ記録〉にする形は D-118 の Compatibility が決めた後の形〈(b) は proposed・未実装〉で、今の build は先頭へ紐付ける）

**返品の扱い**:
- quantity < 0 または amount < 0 → 返品として許可（ここで弾かない）

**pos_stock_sync判定**:
- 紐付いた商品のpos_stock_sync=0 → 売上記録は作るが在庫減算しないフラグを立てる

**Validateの結果分類**:
```
validate_result
├── matched_rows[]        → 正常に紐付けできた行
├── error_rows[]          → マスタ未登録 + 行パースエラー
└── excluded_rows_count   → 空レコード + 販売なし（カウント不要）
```

### Stage 3: Preview（利用者に確認を求める）

**画面表示内容**:
- ファイル名、精算日
- 冪等性チェック結果（同一hashブロック/同日別hash追加確認/問題なし。D-118 の runtime からは、同一hashブロック/同じ精算の別版の拒否/精算回数の無い file の拒否/問題なし）
- 「○○件の売上を取り込みます（合計 ¥○○○）」
- error_rowsがある場合: 「以下の○件はスキップされます」（JAN・商品名・個数・金額・理由を一覧表示）

**利用者のアクション**:
- 「取り込む」→ Stage 4に進む（error_rowsはスキップ）
- 「キャンセル」→ 何もしない
- error_rowsを見て「商品登録忘れてた」→ キャンセル→商品登録→再取込み

### Stage 4: Commit（トランザクション内で一括書込み）

下の Commit と Rollback は、現行 build では停止中の旧本体の記述（[停止 ADR](../adr/2026-09-23-legacy-stocktake-z004-write-stop.md)、[32](../function-design/32-biz-csv-import-service.md) §15.0）。

BEGIN TRANSACTION内で:
1. 同一active `file_hash` を再検査する。
2. 同一 `settlement_date` のactive import ID一覧を再取得し、preview snapshotと一致することを確認する。不一致なら副作用なしで中断する。
3. 既存importを変更せずcsv_importsにレコード作成（statusは仮値。ステップ6で確定）
4. matched_rowsの各行について:
   - sale_recordsにINSERT（source='auto', source_line_no=レコード番号）
   - pos_stock_sync=1の商品 → inventory_movementsにINSERTし、その商品の写しを記録から求め直す（`recompute_stock`）
   - pos_stock_sync=0の商品 → sale_recordsのみ（在庫は動かさない）
5. error_rowsがある場合 → csv_import_errorsテーブルにINSERT
6. csv_imports.status / total_items / total_amount / skipped_countを確定:
   - error_rows=0 → status='completed'
   - error_rows>0 → status='completed_partial'
7. COMMIT
8. operation_logsに取込みログを記録

**Rollback**:
1. `csv_imports.id` で対象を特定する。settlement_dateだけで対象を選ばない。
2. BEGIN TRANSACTION内で対象ID由来の `sale_records` / `inventory_movements` だけをvoidし、無効にした movement の商品ごとに `recompute_stock` で写しを求め直して（補償の movement は作らない）、対象 `csv_imports.status='rolled_back'` にする。
3. 同一settlement_dateの他のactive import、sale_records、inventory_movementsは変更しない。すでにrolled_backの対象IDは冪等成功とする。
4. COMMIT後に対象IDを含む `csv_rollback` operation logをbest-effortで記録する。

---

## 13. sale_records（売上レコード）

### 役割
商品別の売上記録。CSV取込み（自動）と手動販売出庫の両方がここに入る。売上レポート（REQ-501/502）のデータソース。

### カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | 売上ID |
| csv_import_id | INTEGER | FK → csv_imports.id, NULLABLE | CSV取込み元。NULLなら手動 |
| product_code | TEXT | FK → products.product_code, NOT NULL | 商品コード |
| sale_date | TEXT | NOT NULL | 販売日（YYYY-MM-DD） |
| quantity | INTEGER | NOT NULL | 数量。マイナスなら返品（売上帳票視点） |
| amount | INTEGER | NOT NULL | 金額。マイナスなら返品（売上帳票視点） |
| source | TEXT | NOT NULL, CHECK(source IN ('auto','manual')) | 記録元 |
| source_line_no | INTEGER | NULLABLE | Z004のレコード番号。将来の原票追跡用 |
| reason | TEXT | NULLABLE | 手動の場合の理由 |
| note | TEXT | NULLABLE | 備考 |
| is_voided | BOOLEAN | NOT NULL, DEFAULT 0 | 論理無効化フラグ。ロールバック時に1 |
| created_at | TEXT | NOT NULL | 作成日時（YYYY-MM-DDTHH:MM:SS） |
| stock_linked | INTEGER | NOT NULL, DEFAULT 0, CHECK(stock_linked IN (0,1)) | 取込みのときの在庫連動（v11、D-118 の b1。Z004 の確定だけが行の `pos_stock_sync` を書き、手動の売上ほかは 0） |

### 設計意図
- **自動と手動を同じテーブルに統合した理由**: 売上レポート（日次・月次）では自動売上と手動売上をまとめて表示する。別テーブルだと毎回UNIONが必要になり、集計クエリが複雑になる。sourceカラムで区別すれば1テーブルで完結
- **quantityがマイナスの理由（Z004実機検証結果）**: Z004のレジ戻し処理はマイナス値で出力される。そのまま取り込む。「5個売れて1個返品」の日は、quantity=5の行とquantity=-1の行の2行ができるか、ネットでquantity=4の1行になるかはZ004の出力形式次第。実機検証結果ではPLU別のネット値（販売-戻し）で出力されるため、1行にまとまる
- **csv_import_idの理由**: ロールバック時にどの売上レコードを無効化するか特定するため

### 業務シナリオ例
```
CSV取込み: 2026/03/21の精算データ
  sale_records:
    id=1, csv_import_id=1, product_code="4976383262108", sale_date="2026-03-21",
    quantity=3, amount=1782, source="auto"
    （3個×594円。返品なし）

  sale_records:
    id=2, csv_import_id=1, product_code="4973167902615", sale_date="2026-03-21",
    quantity=-1, amount=-385, source="auto"
    （返品1個。Z004でマイナスが出た場合。ネット値なら売上と相殺された値で1行）

手動販売出庫:
  sale_records:
    id=3, csv_import_id=NULL, product_code="HZ-0099", sale_date="2026-03-21",
    quantity=1, amount=880, source="manual", reason="plu_unregistered"
    （PLU登録前の新商品ヘアゴムが1個売れた）
```

---

### 更新履歴

| 日付 | PR | 内容 |
|---|---|---|
| 2026-08-16 | PR #79 | D-071 / SPEC-SDI-D1〜D8: business dateをgroup keyとし、両pipelineのinsert-only TX、active snapshot再検証、per-import rollbackを正本化。 |
| 2026-10-06 | sd-direct-read（design、D-111） | `daily_report_imports.settlement_no` と同じ精算の二重取込みの拒否（BIZ-08-D4）。入力の標準をレジの SD の直読みにした。 |
| 2026-10-17 | sd-direct-read-runtime（起票） | `settlement_no` の列を足す migration を [22 §17](../function-design/22-mnt-migration.md#17-mnt-03-追加-日報の束の精算回数の列の-migrationd-111) に置いた。 |
| 2026-10-17 | sd-bulk-import-design（design、D-121） | 「SD の一括取込みと EJ の控えの契約」（`ej_records`）を proposed で足した。 |
