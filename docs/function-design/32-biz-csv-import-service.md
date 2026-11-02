## 15. BIZ-03: Z004商品別CSV取込みパイプライン

### 計数の置き場と在庫の求め直しの契約（D-118。b1 で実装済み）

[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)（SPEC-STK-CNT-D1〜D4・D7・D11）の BIZ-03 の側。b1（[Plan Packet](../archive/plans/2026-10-22-stocktake-count-runtime-b1.md)）で §15.3〜§15.5 の parse / commit / rollback を本節の契約に置き換えた。日報の束・売上の集計日・既存の active hash の拒否は変えない。旧 ADR（時点証拠）の資料の受領（`pos_import_sources`）・前後の分類（Before / After / Unknown）・全数読取り・要再確認 flag の表と 5 つの理由・売上と返品の相殺の判定・EJ の証拠の interface・準備照会（`get_pos_stock_readiness` と issue）・共有 JAN の file 全体の保留・取消の補償と保留・外部 probe 表は持たない（置き換えた。D-109 で `#[cfg(test)]` に置いた時点証拠の code は b1 で外した）。

#### 精算の識別と並び（SPEC-STK-CNT-D4）

`parse_and_validate` は IO-02 の精算の識別メタ（[23](23-io-z004-parser.md) の `settlement_metadata` の machine_no・settlement_no・精算日）を受け、次の順に検査する。どれかに当たれば preview を拒否し（既存の `ImportError`、下の message。順 4 だけは `BizError::SettlementConflict { message, csv_import_id }` で、文が名指す取込みID を構造でも持つ〈§15.10〉）、確定の TX でも同じ検査をやり直す（preview の後に別の取込みが確定した場合を拾う）。

| 順 | 条件 | message |
|---|---|---|
| 1 | machine_no か settlement_no が無い・settlement_no が 10 進の整数でない（従来の形の file を含む）。10 進の整数は ASCII の数字だけで、値が 0 以上 `9007199254740991` 以下（JavaScript の安全な整数の範囲。精算回数と知らせの区間の端を wire の number で正確に運ぶため。2026-10-22、b1 の起票で決めた） | 精算回数の無い file は取り込めません。レジの SD の商品別の売上（Z004）を選んでください |
| 2 | Z004 の取込み（取り消したもの〈rolled_back〉を含む）があり、その machine_no がこの file と違う | machine_no の違う active な取込みがある: レジの番号が前の精算と合いません。レジの入替・初期化を確かめてください。取り消した取込みとだけ違う: 取り消した取込みとレジの番号が合いません（取込みID: {id}、精算 {n}、取込み日: {imported_at}、レジ: {machine_no}。この file のレジ: {この file の machine_no}）。どちらが店のレジの資料かを確かめてください |
| 3 | 同じ hash の active な取込みがある | （今の重複の拒否のまま） |
| 4 | 同じ machine_no・settlement_no の active な取込みがあり、hash が違う | 同じ精算と思われる別の資料が既に取り込まれています（精算 {n}、取込みID: {id}）。{id} は [24](24-io-csv-import-repo.md) の同じ精算の active な取込みの一覧の先頭（id の昇順）。CMD は kind `import_error` のまま `CmdError.related_csv_import_id` に同じ id を入れ（[41](41-cmd-pos.md) §17.4）、画面はそこから CSV取込みの詳細へ進める（[55](55-ui-csv-import.md) UI-07-D18） |
| 5 | active な取込みのうち、settlement_no がこの file より小さいのに精算日がこの file より後のもの、または settlement_no が大きいのに精算日が前のものがある | レジの日付か番号が前の精算と合いません。レジの時計か、レジの入替・初期化を確かめてください |

- 順 3〜5 と下の知らせは、取り消した取込み（rolled_back）を比べる先にしない（誤った版を取り消して正しい版を取り込める。SPEC-STK-CNT-D7）。順 2 だけは取り消した取込みとも比べる（b1 の Plan Review round 3 の是正）: machine_no は版でなくレジを示し、誤った版も同じレジの資料なので D7 の取り込み直しを妨げない。active だけと比べると、全ての取込みを取り消した後に別のレジの file が通る（店のレジは 1 台の前提。ADR D1・D4。fresh な計数と完了の判定は、保存・完了の後の取込みを判定の n に数えないので〈ADR D3・D6〉通っても「k ≤ n」で黙って計数に入りはしないが、照合は残す。owner の決定〈repo 外の回答台帳 TD-388〉）。比べる先は [24](24-io-csv-import-repo.md) の「Z004 の取込みの machine_no の照合先」が返す 1 件（active を優先する）で、それが取り消した取込みのとき（machine_no の違う active な取込みが無いとき）は、拒否の文でその取込みを名指す（最初に別のレジの file を取り込んで取り消した DB では、店のレジの file がずっと順 2 で拒まれ、原因を名指さないと利用者に理由が分からない。D-127。照合は残し、照合の基準を入れ替える操作は持たない。2026-10-23、owner の決定〈repo 外の回答台帳 TD-388〉、b1 の Plan Review closure の是正）。順 5 は取り消した誤った版の精算日と比べると正しい精算を拒みうるので active だけと比べる。全てを取り消した後のレジの番号の戻りは順 5 が止めないが、fresh な計数の判定の n と完了の時点の精算の n は、保存・完了の後の取込みと取り消した取込みを数えないので（取込みのカーソル。ADR D3・D6、b1 の Plan Review closure〈Codex〉の是正）、戻った番号の精算は日付の判定に倒れる。
- 同じ精算日の別の精算（settlement_no が違う）は、利用者の追加確認なしに preview から確定へ進める（D-071 の Z004 の側の置換え。`additional_import_confirmed` は Z004 では使わない）。
- 前の「精算日が未来の日付です」の拒否は外し、下の知らせ (b) に変える（レジと PC の日付のずれで取り込めなくなるのを避け、知らせて取り込む。ADR D4）。
- 知らせ（拒否しない。preview と結果に出す）: (a) 欠け: active な取込みと、この file を合わせた settlement_no の最小から最大の間に、取り込まれていない番号があれば、連続した欠けを 1 つの区間にまとめ、「精算 {n}〜{m} が取り込まれていません」（区間が 1 つの番号なら「精算 {n} が取り込まれていません」。区間ごと、昇順。欠けた精算の売上は在庫に入っていない）。知らせの数は区間の数で上限があり、欠けた番号を 1 つずつ並べない（壊れた大きな精算回数を受けても preview が止まらない）。(b) 精算日が PC の今日より後なら「レジと PC の日付がずれている可能性があります」。
- 理由: 精算回数は 4 年分の資料で減少・重複が無く、欠けは SD の本数の差と一致した（owner TD-261 の Q3 = a）。精算日と番号の食い違いは reset・レジの入替・時計の戻りのどれでも起き、精算の窓（ADR D3）が 1 つに決まらないので取り込まない（レジの日時は検知と窓の日付にだけ使い、PC の時計と比べない）。

#### 確定（commit）

- 確定の TX は、上の検査・JAN の候補・連動の設定を取り直し、売上（`sale_records`）と、在庫連動の商品の movement（`business_date = settlement_date`、`phase = 1`、`settlement_no`）を書き、取り込み済みの精算の一覧が変わるので、計数を持つ全ての商品と、その取込みの売上か movement を持つ全ての商品（在庫連動の on / off を問わない）を [31](31-biz-inventory-service.md) の `recompute_stock` で求め直す（ADR の横断の表と D2 の原理）。売上と在庫を動かす行の対応は今のまま（計数に入っているかで在庫の適用を分けない。入っているかは求め直しが ADR D3 の窓の判定表で決める）。売上の各行に、その商品の取込みのときの `pos_stock_sync` を `stock_linked` として写す（pos-tables。共有 JAN の行と商品の無い行は 0）。
- 結果（`ImportResult`）に、求め直しの後に要再確認になった商品の一覧（`recheck_targets`: この TX で求め直した商品のうち、求め直しの後に `stock_recheck=1` の商品。[40](40-cmd-product.md) の CountRecoveryTarget。`recount_reasons` 付き）と、上の知らせを足す。要再確認は取込みを止めない。
- 同じ JAN を複数の商品が持つ行（SPEC-STK-CNT-D4、owner の決定 TD-296）: 店内コードの lane ができるまで在庫連動の外に置き、売上を今の規則（先頭の商品）で記録し、どの候補の movement も作らない（候補に在庫連動の商品があっても同じ。行エラーにしない）。
- 数量・金額とも 0 の行（日計 0）は今のまま売上も movement も作らない（相殺の判定に使わないので保持しない）。数量 0 で金額が 0 でない行の売上の記録は今のまま。
- 正常な行が全て 0 の file（その精算で PLU の売上が無かった）は、今の空の対象の拒否をやめ、売上と movement を作らずに精算の識別メタだけを持つ取込みとして確定する（ADR D4。精算の並び・D3 の窓・欠けの判定に入る。取り込まないと、その精算が欠けとして残り、計数の窓が決まらない）。
- preview の引き当てと確定の取り直しの食い違い（D-122 からの申し送り。2026-10-22、b1 の起票で決めた）: 確定の TX で JAN の候補を取り直した結果、preview の行（`MatchedRow`）の商品と違う商品に当たる行、または preview で引き当てた商品が無くなった行が 1 つでもあれば、何も書かずに `ImportError`（「商品の登録が読み込みの後に変わりました。ファイルを選び直してください」）を返す。理由: 利用者が preview で確かめた引き当てと違う商品へ黙って売上と在庫を入れない（D-127）。知らせて確定する案は、確かめていない引き当てで在庫を動かす。preview の後に変わった在庫連動の設定は、確定の TX の値を使う（`stock_linked` は確定の時点の `pos_stock_sync`）。
- preview の cache の契約（上の食い違いの拒否と、確定の TX の順 1〜5 の取り直しに要る。b1 の Plan Review round 1 の是正、2026-10-22）: BIZ-03-D1 の「commit 最小内部契約」を次の形に改める（§15.2 の型の節と `src-tauri/tests/import_internal_contract_test.rs` の固定は b1 の実装で同じ形にする）。(1) `MatchedRow` は `line_no`・`normalized_jan`（IO-02 が正規化した 13 桁の JAN）・`product_code`（preview で引き当てた先頭の商品）・`shared_jan: bool`（preview の時点でその JAN を持つ商品が 2 つ以上）・`quantity: i64`（上の数量の換算の後の在庫の数量）・`amount` を持つ。`pos_stock_sync` は持たない（確定の TX で取り直す）。`normalized_jan` は表示のためでなく確定の判定のために持つので、「表示専用の JAN を cache へ複製しない」の理由に当たらない（D-122 からの申し送りの照合には、preview の時点の元の JAN が要る。商品コードと今の JAN からは、元の JAN の引当先が変わったことを求められない）。(2) `CachedPreview` は `created_at`・`matched_rows`・`error_rows`・`preview_data`・`settlement`（`machine_no`・`settlement_no`・`settlement_date`。parse の時点で順 1 を通った精算の識別メタ）を持つ。`active_same_date_import_ids` は外す（同じ日の追加の確認を Z004 で使わない）。(3) `CommitRequest` は `cached_data` と `today`（CMD が渡す PC の今日。知らせのため）を持ち、`additional_import_confirmed` は外す。(4) 確定の TX の照合: `settlement` で順 1〜5 を取り直す（file を読み直さない）。`matched_rows` の各行について `normalized_jan` で今の候補を引き、候補が無い、先頭の商品が `product_code` と違う、候補が 2 つ以上かどうかが `shared_jan` と違う、のどれかなら上の食い違いとして拒否する。`error_rows` のうち `unmatched_product` の行の JAN に今は候補がある場合も同じく拒否する（preview で「未登録」と確かめた行が、確定で黙って別の扱いにならない）。CMD は parse の結果から cache を作り、確定で token から復元して渡す（[41](41-cmd-pos.md)。cache の持ち主は今のまま CMD）。
- 数量（SPEC-UNIT-D8 の BIZ の換算のうち整数の数量の分。2026-10-22、b1 の起票で決めた）: 行の数量（IO-02 の整数）は、商品を引いた後に在庫の数量へ換える。個数の単位はそのまま、長さの単位（`m`）は × 100 の cm（レジの数量 1 = 1 m、SPEC-UNIT-D4 の不変条件）。売上の `sale_records.quantity` も同じ在庫の数量で書く（日次・月次の集計の `length_cm` と手動販売の売上と同じ単位）。× 100 の後の絶対値が 1 明細の数量の上限（[共通規則](10-common-rules.md) の「入力の上限と安全な整数の範囲」の `9999999`）を越える行は、preview の行の error（今の数量の上限の行の error と同じ文）にする。小数の数量（`quantity_hundredths`、整数でない個数の行の扱い）は [backlog](../backlog.md) の「単位の拡張の残りの申し送り」の P で、それまで今の parser の行の error のまま（黙って入らない）。
- 知らせの wire: 欠けの区間の端（`from`・`to`）は上の表の順 1 の範囲の精算回数なので、wire の number で正確に運ぶ（文字列にしない）。

#### 取消（rollback、SPEC-STK-CNT-D7）

- 対象の取込みの売上と movement を `is_voided=1` にし（今のまま）、取り込み済みの精算の一覧が変わるので、計数を持つ全ての商品と、その取込みの売上か movement を持つ全ての商品（在庫連動の on / off を問わない）を [31](31-biz-inventory-service.md) の `recompute_stock` で求め直す（ADR の横断の表と D2 の原理）。補償の movement は作らない。計数は消さない。結果に、求め直しの後に要再確認になった商品の一覧を足す。
- 取り込み直し（ADR D7）: 取り消した精算と同じ精算回数の file は新しい取込みとして確定する。取り込み直した取込みは、それより前に保存した fresh な計数にとって保存の後の取込みなので、判定の n の算出に入らない（ADR D3）。取り込み直した精算 k は D3 の表のとおり、k が判定の n 以下なら計数に入り（例: 同じ日の精算 99・100 の後の fresh な計数で 99 を取り消して取り込み直すと、n は 100 のままで 99 は入り、要再確認にならない）、それ以外を日付で判定する。要再確認になるのは、取り込み直した精算が判定の n だった（取り消すと n が下がり、k が n より大きくなる）場合で、精算日が計数の日なら、その精算に行のある商品は確定の結果の `recheck_targets` に出て、入れ直すと解ける（2026-10-23、owner の決定〈repo 外の回答台帳 TD-400〉）。

#### 停止の解除

停止 ADR の 5 つの入口（Z004 の確定と取消を含む）は、本節の確定と取消を入れた b1 で一度に外した（SPEC-STK-CNT-D12・SPEC-STOP-D6）。

#### SD の一括取込み（D-121、proposed・未実装）

SD の締めの一括取込み（[37 §37.10](37-biz-daily-report-import-service.md)、BIZ-08-D7）は、SD から読んだ Z004 を精算回数の昇順に、本節の preview の検査と確定をそのまま呼んで取り込む（runtime は backlog の「Z004 を SD の一括取込みに入れる runtime（B）」。本節の確定を入れる配布の単位の後）。上の表の拒否は、その file を飛ばして続ける。SD から読んだ file だけ、確定の TX の前に IO-10 の写しを書く（BIZ-08-D5 の引継ぎ。書けなければ確定しない）。ファイルを選ぶ経路（Z004 タブ）は写さず、ほかは変わらない。

> **2026-08-01 evidence sync**: 本書は既存の Z004-only product-sales import pipeline の実装契約を記録する。current operation の日報主入力 `Z001`/`Z002`/`Z005` は [37-biz-daily-report-import-service.md](37-biz-daily-report-import-service.md) のBIZ-08で扱う。集計日報データは `daily_report_imports` / `daily_report_*_lines` に保存し、item-level `sale_records` / `inventory_movements` へ擬似展開しない。Z004側はsale_records作成、`pos_stock_sync`在庫増減、重複・rollbackまで実装済み。2026-07-06店舗採取layout AはIO-02が二形状対応済み（SPEC-Z4A-D1〜D7）。field readinessは実データend-to-end再検証（CSV-09/10）を待つ。

### 15.1 モジュール構成

```
src-tauri/src/
  biz/
    mod.rs                   -- pub mod csv_import_service を追加
    product_service.rs       -- 既存（BIZ-01）
    inventory_service/       -- 既存（BIZ-02、ディレクトリモジュール）
    csv_import_service/      -- CSV取込みの業務ロジック（本セクション。mod.rs の型、parse.rs・commit.rs、rollback.rs〈取消と取消の前の読取り。§15.5・§15.6b〉）
```

---

### 15.2 型定義

#### CsvParseAndValidateRequest構造体

- file_bytes: Vec\<u8\>（Z004ファイルの生バイト列）
- filename: String（ファイル名。csv_imports.filename に記録する表示用）

#### ParseValidateResult構造体

- preview_data: PreviewData
- preview_token: String（UUID v4。CMD層がキャッシュキーとして使用）
- matched_rows: Vec\<MatchedRow\>（CMD層がキャッシュに保存する。フロントエンドには返さない）
- error_rows: Vec\<ErrorRow\>（同上）

#### PreviewData構造体

- file_info: FileInfo
- matched_summary: MatchedSummary
- error_summary: ErrorSummary
- settlement_gaps: Vec\<SettlementGap\>（知らせ (a): 取り込まれていない精算回数の区間、昇順。取込みを止めない）
- settlement_date_after_today: bool（知らせ (b): 精算日が PC の今日より後）
- preview_created_at: String（YYYY-MM-DDTHH:MM:SS）

※ preview_token は ParseValidateResult のトップレベルにのみ保持する。PreviewData はフロントエンドに表示するデータのみを含む。フロントエンドは ParseValidateResult.preview_token を commit 時に送り返す。

#### FileInfo構造体

- filename: String
- settlement_date: String（YYYY-MM-DD）
- file_hash: String（SHA-256 hex、小文字64文字）

#### MatchedSummary構造体

- count: usize（紐付け成功件数）
- total_amount: i64（matched_rows の amount 合計）
- warnings: Vec\<String\>（グループコード商品の紐付け警告等）

#### ErrorSummary構造体

- count: usize（エラー行の総数）
- items: Vec\<ErrorRow\>（最大100件。UI表示の上限。選び方は§15.3 手順 5c: error_type が invalid_jan の行を先に選び、残りの枠を他の行で埋め、line_no の昇順で並べる。100件を超えた分は count にだけ数える）

#### SettlementGap構造体

- from: i64・to: i64（欠けた精算回数の区間の端。1 つの番号なら from = to。上の表の順 1 の範囲なので wire の number で正確に運ぶ）

※ 同一 file_hash の active な取込みは parse_and_validate 内で即 `BizError::ImportError` を返す。同じ精算日の別の精算は追加の確認なしに確定へ進む（ADR D4。D-071 の Z004 の側の置換え）。

#### MatchedRow構造体（サーバ側キャッシュに保持、フロントエンドには送らない）

- line_no: usize（Z004の行番号。1始まり）
- normalized_jan: String（IO-02 が正規化した 13 桁の JAN。確定の TX で今の候補を引き直す）
- product_code: String（preview で引き当てた先頭の商品コード）
- shared_jan: bool（preview の時点でその JAN を持つ商品が 2 つ以上）
- quantity: i64（在庫の数量に換えた後の値。長さの単位は × 100 の cm。正=販売、負=返品）
- amount: i32

#### ErrorRow構造体

- line_no: usize
- normalized_jan: Option\<String\>（JAN正規化前にエラーならNone）
- name: String
- raw_quantity: String
- raw_amount: String
- error_type: String（"unmatched_product" / "invalid_format" / "invalid_jan" / "invalid_number"）
- error_message: String（利用者向け日本語メッセージ）

**enum 契約化（D-061）**: `error_type` の4値は IO の `ParseErrorType` 3 variant + BIZ 生成 `unmatched_product` を合成した `CsvImportErrorType` generated enum で表す。IO→wire 変換は明示 match とし、値・利用者向け文言は不変。

#### CommitRequest構造体

- cached_data: CachedPreview（CMD層がキャッシュから復元したデータ。matched_rows, error_rows, preview_data, settlement を含む）
- today: String（CMD が渡す PC の今日。YYYY-MM-DD。知らせのため）

**BIZ-03-D1（commit最小内部契約）**:

- `MatchedRow` はcommitが在庫・売上・監査行の生成と確定の TX の照合に使う `line_no` / `normalized_jan` / `product_code` / `shared_jan` / `quantity` / `amount` だけを保持する（上の「計数の置き場と在庫の求め直しの契約」の preview の cache の契約。`pos_stock_sync` は確定の TX で取り直す）。商品名を30分cacheへ複製しない。
- `CommitRequest` はCMDがtokenで復元済みの `cached_data` と、CMD が渡す `today` だけを受け取る。tokenのUUID検証・cache対応確認・TTL・成功時削除はCMD-07の責務であり、BIZ requestへ再格納しない。

#### ImportResult構造体

- csv_import_id: i64
- status: CsvImportStatus（"completed" / "completed_partial"。rollback 後の "rolled_back" と同じ generated enum。値・分岐は不変）
- total_items: i64
- total_amount: i64
- skipped_count: i64
- settlement_gaps: Vec\<SettlementGap\>・settlement_date_after_today: bool（preview と同じ知らせ）
- recheck_targets: Vec\<CountRecoveryTarget\>（この TX で求め直した商品のうち要再確認の商品。[40](40-cmd-product.md)）

#### RollbackResult構造体

- success: bool
- voided_sale_count: u64
- voided_movement_count: usize
- recheck_targets: Vec\<CountRecoveryTarget\>（補償の movement は作らないので補正の一覧は持たない。ADR D7）

#### CachedPreview構造体（サーバ側メモリキャッシュ）

- created_at: std::time::Instant（キャッシュ作成時刻。有効期限判定に使用）
- matched_rows: Vec\<MatchedRow\>
- error_rows: Vec\<ErrorRow\>
- preview_data: PreviewData（フロントエンドに返した内容のコピー）
- settlement: CachedSettlement（精算の識別メタ。`machine_no`・`settlement_no`・`settlement_date`。parse の時点で順 1 を通った値。確定の TX の順 1〜5 の取り直しに使う）

---

### 15.3 parse_and_validate（Stage 1+2+3）

**関数要求**: Z004ファイルを解析し、マスタ照合後のプレビューデータを返す。業務テーブルへの書き込みなし（parse失敗時のoperation_log記録は例外）。preview_token を生成して返す（キャッシュ保存はCMD層の責務）

**シグネチャ**:
```
fn parse_and_validate(
    conn: &DbConnection,
    req: CsvParseAndValidateRequest,
) -> Result<ParseValidateResult, BizError>
```

**前提条件**: 業務テーブル（csv_imports, sale_records等）への書き込みなし。ただし**parse失敗時のoperation_log記録は例外**（system_repo::insert_operation_logを呼ぶ。ログ記録失敗は警告のみで処理続行）。conn は &DbConnection（operation_logの書き込みはautocommit）。キャッシュはBIZ層で保持しない（ロック区間最小化のためCMD層がAppStateで管理）

**処理ステップ**:

1. **サイズガード**
   - req.file_bytes.len() > constants::CSV_IMPORT_FILE_SIZE_LIMIT → BizError::ImportError("ファイルサイズが上限（20MB）を超えています")
2. **Stage 1: Parse**（IO-02 委譲）
   - io::z004_parser::parse_z004(&req.file_bytes) を呼び出し
   - Err(Z004ParseError) → system_repo::insert_operation_log(conn, &NewOperationLog { operation_type: "csv_import_parse_failed", summary: エラーメッセージ, detail_json: None }) → BizError::ImportError に変換
     - DecodeFailed → "Z004ファイルの解析に失敗しました: CP932デコードエラー"
     - NoDataLines → "Z004ファイルの解析に失敗しました: データ行がありません"
     - NoSettlementDate → "Z004ファイルの解析に失敗しました: 精算日を抽出できません"
   - Ok(parse_result) → parsed_rows が 0件かつ parse_errors が非空 → system_repo::insert_operation_log(conn, &NewOperationLog { operation_type: "csv_import_parse_failed", summary: "有効なデータがありません", detail_json: None }) → BizError::ImportError("有効なデータがありません")
3. **行数ガード**
   - parse_result.total_data_lines > constants::CSV_IMPORT_LINE_LIMIT → BizError::ImportError("データ行数が上限（10,000行）を超えています")
4. **Stage 2: Validate**
   a. 空レコード除外（エラーにもカウントしない）
      - quantity == 0 かつ amount == 0（PLU登録あるが当日販売なしのスロット）
      - ※ コードが全桁ゼロか13桁JANにならない行は、IO-02 parse_data_line が個数・金額とも0なら読み飛ばし（Ok(None)）、どちらかが0でなければ invalid_jan の行エラーにする（[23 §13.4.2](23-io-z004-parser.md) SPEC-Z4A-D8、D-103）。parsed_rows には13桁JANの行だけが来るため、BIZ-03では全桁ゼロの再判定は不要
   b. 実データ行のマスタ照合: 各行について
      - product_repo::find_by_jan_code(conn, &normalized_jan) を呼び出し
      - ヒット1件 → MatchedRow { line_no, normalized_jan, product_code, shared_jan: false, quantity（在庫の数量に換えた後。上の「数量」）, amount }
      - ヒット0件 → ErrorRow { line_no, normalized_jan: Some(normalized_jan), name, raw_quantity: quantity.to_string(), raw_amount: amount.to_string(), error_type: "unmatched_product", error_message: "JAN {jan} に該当する商品がありません" }
        - （proposed・未実装、JAN の別名の runtime の lane、D-122）ヒット0件の行だけ、ErrorRow にする前に `product_repo::find_jan_alias(conn, &normalized_jan)` で前の JAN の別名（[master-tables](../db-design/master-tables.md) §1a、[30](30-biz-product-service.md) BIZ-01-D9）を引く。別名があればその商品をヒット1件と同じ形の MatchedRow にする（`pos_stock_sync` はその商品の値。warning は足さない）。無ければ上の ErrorRow のまま。不変条件（どの商品も持たない JAN だけが別名）により、ヒット1件・複数件の行に別名は無いので、引く順で結果は変わらない
      - ヒット複数件 → ORDER BY product_code ASC で先頭を採用し `shared_jan: true`（確定で在庫連動の外。上の「確定」）。warnings に "JAN {jan} は複数商品に紐付いています（{code} を使用）" を追加
      - 引き当ての規則（上の 3 つと別名）は 1 つの関数に置き、preview と、D-118 の確定の TX での JAN の候補の取り直し（上の「確定（commit）」）の両方がそれを使う（preview の後に別名ができた・消えた場合も、確定はその時点の DB で引き当てる）。結果は 0 件 / 1 件（別名を含む）/ 複数（先頭と件数）を区別して返す（preview は複数の行に上の warning を足し、確定は複数の行を上の「確定（commit）」の共有 JAN の行として扱う）。引き当ては取込みの時点の DB を使い、売れた日の商品の JAN の履歴は持たない（取り消して取り込み直すと、その時点の別名と JAN で引き当て直す）
   c. parse_result.parse_errors を ErrorRow にマージ
      - ParseErrorType → error_type 文字列変換: InvalidFormat → "invalid_format", InvalidJan → "invalid_jan", InvalidNumber → "invalid_number"。商品コードの無い枠の売上（PLU の登録を消した枠、コードを持たない通常 PLU。SPEC-Z4A-D8）も IO-02 の InvalidJan として届き、"invalid_jan" の ErrorRow になる。取込みは completed_partial になり、文言で見分ける（新しい種別を足さない理由は 23 §13.4.2）
      - Option→String 変換規約: ParseError.raw_name/raw_quantity/raw_amount が None の場合、空文字列に変換する。None は error_type が invalid_format の場合にのみ発生する（フィールド分割前のエラーで値が取得できなかった）。Z004のフィールドは実運用で空文字にならないため、空文字はパース前エラーと判別可能。DB保存（csv_import_errors）は空文字のまま、UI表示時に error_type が invalid_format なら「(不明)」等に置換する
   d. 正常な行が全て 0 の file（matched_rows も error_rows も空）は拒まず、精算の識別メタだけを持つ取込みとして preview へ進む（上の「確定」）
5. **Stage 3: Preview**
   a. file_hash 重複チェック:
      - sales_repo::find_blocking_import_by_file_hash(conn, &parse_result.file_hash) → Some → BizError::ImportError("このファイルは取込み済みです（取込みID: {id}、取込み日: {imported_at}）")
   b. 精算の識別と並び（上の表の順 1・2・4・5。順 3 は 5a）と知らせ（欠けの区間、精算日が今日より後）。今日は CMD が渡す PC の今日。精算の識別メタを `CachedPreview.settlement` に保持する
   c. PreviewData 構築
      - matched_summary: count = matched_rows.len(), total_amount = matched_rows.iter().map(|r| r.amount as i64).sum(), warnings
      - error_summary: count = error_rows.len()（全件）。items は最大100件で、error_rows のうち error_type が invalid_jan の行を line_no の昇順で先に選び、残りの枠を他の行から line_no の昇順で埋める。選んだ items は line_no の昇順で並べる。error_rows 自体（commit・csv_import_errors へ渡す全件）の内容と順は変えない
      - 理由: 商品コードの無い枠の売上（SPEC-Z4A-D8、D-103）と売上のある非JANの行は、商品を登録しても在庫に入らず、Z004 の commit の停止中（SPEC-STOP-D1）は取込み記録の詳細にも届かないため、プレビューのエラー詳細が唯一の知らせ先になる。error_rows は 4b の unmatched_product の行が先に並ぶため、先頭から切ると未登録 JAN の行が100件を超えたときにこれらの行が items から落ちる。種別だけで選び、文言では判別しない
6. **preview_token 生成**: UUID v4
7. ParseValidateResult { preview_data, preview_token, matched_rows, error_rows } を返す
   - CMD層がこの戻り値を受け取り、preview_tokenをキーとしてAppState.preview_cacheに保存する（17.5節参照）

**エラーハンドリング**:
- IO-02 parse失敗 → BizError::ImportError（operation_log記録後に変換）
- 全行パースエラー → BizError::ImportError("有効なデータがありません")
- 空レコード除外後0件 → BizError::ImportError("取込み対象のデータがありません")
- サイズ上限超過 → BizError::ImportError
- 行数上限超過（total_data_lines > constants::CSV_IMPORT_LINE_LIMIT）→ BizError::ImportError
- file_hash 重複ブロック → BizError::ImportError
- DB読み取り失敗（マスタ照合等）→ BizError::DatabaseError(DbError)

**入力例**:
```
req: CsvParseAndValidateRequest {
    file_bytes: [CP932バイト列...],
    filename: "Z004_260321",
}
```

**出力例**:
```
Ok(ParseValidateResult {
    preview_data: PreviewData {
        file_info: FileInfo {
            filename: "Z004_260321",
            settlement_date: "2026-03-21",
            file_hash: "a1b2c3d4e5f6...",
        },
        matched_summary: MatchedSummary {
            count: 45,
            total_amount: 28500,
            warnings: [],
        },
        error_summary: ErrorSummary {
            count: 3,
            items: [
                ErrorRow { line_no: 12, normalized_jan: Some("4973167064078"), name: "ﾎﾞﾀﾝ ﾊﾞﾗ", ..., error_type: "unmatched_product", error_message: "JAN 4973167064078 に該当する商品がありません" },
                ...
            ],
        },
        settlement_gaps: [],
        settlement_date_after_today: false,
        preview_created_at: "2026-03-21T19:30:00",
    },
    preview_token: "550e8400-e29b-41d4-a716-446655440000",
})
```

---

### 15.4 commit_csv_import（Stage 4）

**関数要求**: プレビュー済みデータをDBに書き込む。CMD層から受け取ったキャッシュデータを使いTX内で一括実行する

**シグネチャ**:
```
fn commit_csv_import(
    conn: &mut DbConnection,
    req: CommitRequest,
) -> Result<ImportResult, BizError>
```

注: キャッシュ復元・有効期限チェック・トークン削除はCMD層の責務（41-cmd-pos.md 17.5節参照）。BIZ層はCMD層が復元済みのデータ（CommitRequest.cached_data）を受け取る。

**処理ステップ**（`csv_import_service/commit.rs`。上の「確定（commit）」の契約の手順）:

1. **TX開始**（conn.transaction()。RAII Drop で自動 ROLLBACK）
2. **順 1〜5 の取り直し**: cache の `settlement`（精算の識別メタ）で上の表の検査をやり直す（file を読み直さない。順 3 は file_hash の TX 内再チェックで TOCTOU を防ぐ）。当たれば何も書かずに `ImportError`
3. **JAN の候補と連動の設定の取り直し**: cache の `normalized_jan` で候補を引き直し、候補が無い・先頭の商品が違う・共有 JAN かどうかが違う・未登録だった行に候補ができた、のどれかなら何も書かずに `ImportError`（「商品の登録が読み込みの後に変わりました。ファイルを選び直してください」）。在庫連動は行ごとに取り直す（共有 JAN の行は在庫連動の外）
4. **csv_imports の INSERT**（精算の鍵 `machine_no`・`settlement_no` 付き。集計値は仮の 0）
5. **売上と movement**: 各行の `sale_records` を `stock_linked`（取込みのときの連動）付きで書く。在庫連動で数量 ≠ 0 の行は movement（`movement_type = sale_auto`、`quantity = -row.quantity`〈INV-1〉、`business_date = settlement_date`、`settlement_no`、仮の `stock_after` = 今の写し）を `insert_movement` で書く（行ごとに `apply_stock_change` を呼ばない。[31](31-biz-inventory-service.md)）
6. **error_rows の記録**（非空のとき `insert_csv_import_errors`）
7. **集計値の確定**（`update_csv_import_totals`。error_rows があれば completed_partial）
8. **求め直し**: 計数を持つ全ての商品とこの取込みの売上か movement を持つ全ての商品を `recompute_stock` で 1 回ずつ求め直し、手順 5 の movement の `stock_after` を求め直しの後の写しの値で `update_movement_stock_after` に書く。在庫の上限の外は TX 全体が戻る
9. **結果の材料**: 求め直した商品のうち要再確認の商品を `recheck_targets` に、欠けの区間を `settlement_gaps` に
10. **COMMIT**
11. **TX外: 操作ログ記録**（operation_type `csv_import`。summary と detail_json は今のまま。失敗は警告のみ）
12. ImportResult を返す。CMD層は成功のときだけ preview_cache から token を削除する（[41](41-cmd-pos.md)）

**TX境界**: 手順 1〜10 が 1 TX。操作ログ（手順 11）は TX 外。

**TX失敗時**: TX ROLLBACK → csv_imports レコードなし。TX 外で `csv_import_failed` の操作ログを記録する。CMD層は Err のときキャッシュを削除しない（同じ token で確定し直しても同じ拒否になる。ファイルを選び直す）。

**符号変換ルール（INV-1準拠）**:
- row.quantity（売上帳票視点）: +3 = 3個販売、-1 = 1個返品
- inventory_quantity（在庫視点）: -3 = 在庫3個減、+1 = 在庫1個増
- 変換式: inventory_quantity = -row.quantity（常に符号反転）

**入力例**:
```
req: CommitRequest {
    cached_data: CachedPreview { matched_rows: [...], error_rows: [...], preview_data: PreviewData { ... }, settlement: CachedSettlement { ... }, .. },
    today: "2026-03-21",
}
```

**出力例**:
```
Ok(ImportResult {
    csv_import_id: 1,
    status: "completed_partial",
    total_items: 45,
    total_amount: 28500,
    skipped_count: 3,
    settlement_gaps: [],
    settlement_date_after_today: false,
    recheck_targets: [],
})
```

---

### 15.5 rollback_csv_import

**関数要求**: 指定 csv_import IDだけを論理無効化し、記録から写しを求め直す（補償の movement を作らない。ADR D7）。冪等（既に rolled_back なら何もせず成功を返す）。同一settlement_dateの他importは残す。

**シグネチャ**:
```
fn rollback_csv_import(
    conn: &mut DbConnection,
    csv_import_id: i64,
) -> Result<RollbackResult, BizError>
```

**処理ステップ**:

1. **対象確認**
   - sales_repo::find_csv_import_by_id(conn, csv_import_id) を呼び出し
   - None → BizError::NotFound("CSV取込み記録が見つかりません: ID {csv_import_id}")
   - Some(import) で import.status == "rolled_back" → RollbackResult { success: true, voided_sale_count: 0, voided_movement_count: 0, recheck_targets: [] } を即リターン（冪等）
2. **TX開始**（conn.transaction()）
3. **無効化**: `void_sale_records_by_import`・`void_movements_by_reference(&tx, "csv_import", csv_import_id)`、`update_csv_import_status(&tx, csv_import_id, "rolled_back")`（指定 ID だけ。同日の他の取込みは変えない）
4. **求め直し**: 取り込み済みの精算の一覧が変わるので、計数を持つ全ての商品とこの取込みの売上か movement を持つ全ての商品（無効の行を含む）を `recompute_stock` で求め直す。計数は消さない
5. 求め直した商品のうち要再確認の商品を `recheck_targets` に
6. **COMMIT**
7. **TX外: 操作ログ記録**（operation_type `csv_rollback`、detail_json は `csv_import_id`・`voided_sale_count`・`voided_movement_count`）
8. RollbackResult { success: true, voided_sale_count, voided_movement_count, recheck_targets } を返す

**設計判断 — 補償の movement を作らない**: 取消は元の movement を `is_voided=1` にし、写しは記録から求め直す（ADR D7）。apply_stock_change は movement を足す関数のため使わない。

**入力例**:
```
csv_import_id: 5
```

**出力例**:
```
Ok(RollbackResult {
    success: true,
    voided_sale_count: 45,
    voided_movement_count: 42,  // 在庫連動でない3件は movement がないため45-3=42
    recheck_targets: [],
})
```

---

### 15.6 list_csv_imports

**関数要求**: csv_imports 一覧をimport単位で返す。CMD 経由のラッパー（repo 直呼び防止）。同日複数importをcollapseせず、repoの `settlement_date DESC, imported_at DESC, id DESC` 順を維持する。

**シグネチャ**:
```
fn list_csv_imports(
    conn: &DbConnection,
    page: u32,
    per_page: u32,
) -> Result<PaginatedResult<CsvImport>, BizError>
```

**処理ステップ**:
1. page < 1 || per_page < 1 || per_page > 100 → BizError::ValidationFailed("ページパラメータが不正です")
2. sales_repo::list_csv_imports(conn, page, per_page) を呼び出し
3. DbError → BizError::DatabaseError に変換して返す

---

### 15.6a get_csv_import_record

**関数要求**: CSV取込み記録詳細を wire DTO として返す。[31-biz-inventory-service.md](31-biz-inventory-service.md) §12.6a 業務記録詳細 read 関数と同じ read-only パターンで、movements への source link 補完と NotFound 変換を BIZ が担う。CSV取込み詳細画面（`/csv-import/records/$importId`、[65-inventory-record-traceability.md](65-inventory-record-traceability.md) §65.3 / §65.5）用

**シグネチャ**:
```
fn get_csv_import_record(
    conn: &DbConnection,
    import_id: i64,
) -> Result<CsvImportRecordDetail, BizError>
```

**CsvImportRecordDetail構造体**（wire DTO。CSV import family の他 wire 型と同じく BIZ-03 が所有する）:
- id: i64, filename: String, settlement_date: String, total_items: i64, total_amount: i64, skipped_count: i64, status: CsvImportStatus, imported_at: String
- items: Vec\<CsvImportRecordDetailItem\>（IO 型を再利用、[24-io-csv-import-repo.md](24-io-csv-import-repo.md) §14.13a）
- error_rows: Vec\<ErrorRow\>（§15.2 既存 wire 型を再利用し、取込みプレビューのエラー表示と同一意味論にする）
- movements: Vec\<MovementRecord\>（source 補完済み）

file_hash は重複取込み判定の内部値であり operator 向け情報ではないため wire DTO に載せない。

**処理ステップ**:
1. sales_repo::get_csv_import_record_detail(conn, import_id) を呼ぶ。IO 層の NotFound は「CSV取込み記録が見つかりません」を含む BizError::NotFound、その他の IO エラーは BizError::DatabaseError に変換する
2. sales_repo::list_csv_import_error_rows(conn, import_id) を呼ぶ
3. エラー行を ErrorRow へ写像する（line_no=source_line_no、name=raw_name、error_type は raw TEXT から `CsvImportErrorType` へ変換。DB CHECK により4値保証のため想定外値は BizError::DatabaseError で fail-fast）
4. movements の各行に `biz::inventory_service` の `resolve_movement_source` と同一実装で source(label, route) を補完する（label/route 規則の独自複製を作らず、共有関数を再利用する）。共有のため `inventory_service/mod.rs` へ `pub(crate) use list::resolve_movement_source;` の re-export を 1 行追加する（`mod list` は private のため sibling module から現状 unreachable。同 file の `pub(crate) use common::apply_stock_change` と同型の既存慣習）
5. CsvImportRecordDetail を構成して返す

---

### 15.6b preview_rollback_csv_import（取消の前の読取り、BIZ-03-D2）

取消（上の「計数の置き場と在庫の求め直しの契約」の取消、SPEC-STK-CNT-D7）と同じ本体を使う。取込みの結果の画面と CSV取込みの詳細の取消の確認（[55](55-ui-csv-import.md) UI-07-D18、[65](65-inventory-record-traceability.md) §65.5a）が、取り消す前に何が起きるかを名指すために読む。

**関数要求**: 指定した取込みを取り消した後に、どの商品が要再確認か（取り消す前から要再確認の商品を含む）・取り込み直す精算の番号・記録の詳細の差異・ロス原価・参考の差が変わることがある完了した棚卸しを返す。何も書かない（売上・movement・写し・取込みの状態・操作ログ・preview の cache のどれも変えない）。

**シグネチャ**:
```
pub fn preview_rollback_csv_import(
    conn: &mut DbConnection,
    csv_import_id: i64,
) -> Result<RollbackPreview, BizError>
```

**RollbackPreview 構造体**（wire DTO。CSV import family の他 wire 型と同じく BIZ-03 が所有する）:
- csv_import_id: i64
- settlement_no: i64（取り込み直す精算の番号。この取込みの精算回数）
- recheck_targets: Vec\<CountRecoveryTarget\>（取り消した後に要再確認である商品。取消の結果の `RollbackResult.recheck_targets` と同じ作り方で、求め直した商品〈計数を持つ全ての商品とこの取込みの商品〉のうち要再確認のものなので、取り消す前から要再確認の商品も入る。[40](40-cmd-product.md) の CountRecoveryTarget）
- affected_stocktakes: Vec\<AffectedStocktake\>（この取込みの精算が完了の後の比較の範囲に入る完了した棚卸し。範囲の判定による候補で、取り消すと記録の詳細の明細の差異・ロス原価〈完了の時点の精算の n 以下の精算から求める〉と参考の差が変わることがある。範囲〈精算回数 ≤ 完了の時点の精算の n、または精算日 < 完了日〉は差異・ロス原価が使う精算を含むので、名指す集合を広げなくてよい。どの数が実際に変わるかは他の精算の判定〈ADR D3 の「D に精算があるか」「最初の 1 本」〉でも決まるので求めず、確認の文は変わりうると言うだけにする。`AffectedStocktake { stocktake_id: i64, completed_at: String }`、完了の日時の昇順）

**処理ステップ**:
1. sales_repo::find_csv_import_by_id。None → BizError::NotFound（「CSV取込み記録が見つかりません: ID {csv_import_id}」）。rolled_back → BizError::ValidationFailed（「この取込みは取り消し済みです」）
2. 精算回数: [24](24-io-csv-import-repo.md) の「active な Z004 の精算の一覧」から import ID の一致する行を引く
3. 記録の詳細の差異・ロス原価・参考の差が変わることがある完了した棚卸し: 完了した棚卸しのうち、この取込みの精算が [35](35-biz-stocktake-service.md) の「完了の後の比較の範囲」（完了の時点のカーソルと完了日で決める範囲）に入るもの。範囲の規則は BIZ-06 の共有の関数 `completed_stocktakes_comparing_settlement`（記録の詳細の `ledger_at_cutoff` と同じ判定の関数を使う）で判定し、BIZ-03 に写さない
4. TX を開き、取消の本体と同じ関数 `apply_rollback`（`&Transaction` を受ける `pub(crate)`。売上と movement の無効化、取込みの状態、ADR D2 の求め直す商品の `recompute_stock`、要再確認の商品の一覧。取消〈§15.5〉と preview がこれを呼ぶ）を当てて recheck_targets を得る
5. TX を明示の rollback で戻す（drop の振舞いに頼らない）。戻せなければ BizError::DatabaseError
6. RollbackPreview を返す（操作ログを書かない）

**設計判断 — 取消を同じ TX で当てて戻す（BIZ-03-D2）**: 取り消した後の要再確認は、精算の一覧が変わったことで ADR D3 の判定の n・「D に精算があるか」・「最初の 1 本」が変わる商品に出る（取消の前に要再確認でなかった商品にも出る）。取消と同じ関数を TX の中で当てれば、preview と取消は同じ規則で同じ商品を出す。棄却: (a) 求め直しに「この取込みを除く」引数を通して書かずに求める（`biz::stock_ledger` の全ての経路に引数が増え、取消の規則を 2 か所に持つ）。(b) 取消の結果だけで知らせる（取り消す前に名指せない。取り消してから要再確認に気づく形になる）。制約: preview は SQLite の書込みの TX を開くが、CMD の 1 本の接続（`AppState.db` の Mutex）で他の command と順に走り、戻すまで他の書込みと交わらない。preview と取消の間に記録が変われば、取消の結果（`RollbackResult.recheck_targets`）が正で、画面は結果の一覧を出し直す。見直しの契機: preview の待ち時間が利用者に見えるほど長くなったとき（求め直しは計数を持つ全ての商品に当たるので、商品と計数が増えたとき）。

---

### 15.7 非目的

このモジュールが**やらないこと**を明示する。責務境界の誤解を防ぐため。

| やらないこと | 理由 | 責務を持つモジュール |
|------------|------|-----------------|
| 物理DELETE | 全 void 処理は is_voided=1。hard delete 禁止 | — |
| IO-04 フォーマット処理（PLU TSV生成） | BIZ-04 の責務 | BIZ-04 |
| INV-5 idempotency_key パターン | BIZ-03 は file_hash 自然冪等性を使用（INV-6） | BIZ-02 の各 create 関数 |
| フロントエンド状態管理 | preview_data の表示制御、ボタン disabled 等 | UI-07 |
| Z004 ファイルの解析 | 純関数として IO-02 に分離済み | IO-02 z004_parser |
| file_hash の算出 | IO-02 が parse_z004 内で実行 | IO-02 z004_parser |
| CSV フィールドのパース | IO-02 の責務 | IO-02 z004_parser |

---

### 15.8 対応不変条件

| 不変条件 | 本モジュールでの対応 |
|---------|-----------------|
| INV-1: quantity 符号変換 | commit_csv_import の手順 5 で movement の quantity = -row.quantity。売上帳票視点→在庫視点の符号反転を BIZ 層で実施 |
| INV-2: stock_after 算出 | commit は movement を仮の stock_after で書き、求め直しの後の写しの値で update_movement_stock_after。rollback は movement を足さず求め直す |
| INV-3: 負在庫ポリシー | 写しは記録から求め直し、負の在庫を受ける（上限の外だけ TX 全体を戻す） |
| INV-4: is_voided 使用範囲 | rollback_csv_import で void_sale_records_by_import, void_movements_by_reference を呼び出し。is_voided=1 を設定するのはロールバック時のみ |
| INV-6: file_hash 自然冪等性 | parse_and_validate のステップ5a で find_blocking_import_by_file_hash。commit_csv_import の手順 2 で TX 内再チェック（TOCTOU 防止） |
| INV-7: csv_import 参照の movements は sale_auto 限定 | commit_csv_import の手順 5 で MovementType::SaleAuto 固定。void_movements_by_reference は reference_type のみで絞り込み（movement_type 条件は冗長） |
| INV-8: products 物理 DELETE 禁止 | 本モジュールは products を UPDATE のみ（stock_quantity）。DELETE 操作なし |

---

### 15.9 preview キャッシュ管理

**所有責務**: CMD層（AppState.preview_cache: Mutex\<HashMap\<String, CachedPreview\>\>）が所有。BIZ層はキャッシュを直接操作しない（ロック区間最小化のため。41-cmd-pos.md 17.3節参照）。

**ライフサイクル**（CMD層が管理）:
- **作成**: CMD層が parse_and_validate の戻り値を受け取り cache.insert
- **消費**: CMD層が commit_csv_import の Ok 受信後に cache.remove
- **有効期限**: CMD層が commit 前に created_at.elapsed() > 30分 をチェック → 期限切れなら cache.remove してCmdError返却
- **アプリ再起動**: 全キャッシュ消失（永続化しない。Preview → Commit は1セッション内で完結する前提）

**設計判断**:
- HashMap の容量: 1人運用のため同時 preview 数は最大1件。HashMap 容量は問題にならない
- **上限**: エントリ数上限10件（constants::PREVIEW_CACHE_LIMIT）。insert 前に len >= 10 なら最も古いエントリ（created_at が最小）を1件削除してから insert する（FIFO）。削除されたトークンで commit が呼ばれた場合、CMD層が「プレビューが見つかりません」CmdError を返す
- 永続化しない理由: Preview データには matched_rows（商品在庫のスナップショットを含む）があり、時間が経つと実際の在庫と乖離する。再起動後は再度 parse_and_validate を実行するのが正しい
- 30分の根拠: 利用者がプレビュー確認→取り込み判断に十分な時間。長すぎると在庫の乖離リスクが高まる
- **commit失敗時**: CMD層は通常の失敗ではキャッシュを削除せず、同一tokenで再試行可能にする。ただしactive same-date snapshot不一致は同じpreviewでは解消できないため、CMD層がtokenを削除し、新しいpreviewを要求する
- **ライフサイクル規約まとめ**: (1) ワンタイム: commit 成功時に即削除、同一トークンでの再 commit は不可 (2) 揮発: アプリ再起動で全消失、再起動後のトークンは必ず無効 (3) トークン不存在（削除済み/再起動後/追い出し）: CmdError kind="import_error" message="プレビューが見つかりません。再度ファイルを選択してください" (4) 期限切れ: CmdError kind="import_error" message="プレビューの有効期限が切れました（30分）。再度ファイルを選択してください" → UIは「ファイル選択」画面に戻す導線を表示
- **architecture/biz-task-specs.md との差異**: 関数設計でpreview_token方式に変更した。理由: Preview結果をフロントエンドに渡して再送信させると、データ量が大きく改ざんリスクもある。preview_token方式ではサーバ側キャッシュからmatched_rowsを復元するため安全。architecture/biz-task-specs.md と整合済み（本PR内で更新）

**キャッシュの所有権**: Tauri の State\<Mutex\<HashMap\<...\>\>\> で管理。CMD層がlock→操作→unlockを行い、BIZ層にはキャッシュを渡さない（ロック区間最小化）。

---

### 15.10 BizError 追加バリアント

BIZ-03 で新たに使用する BizError バリアントの確認:

```
enum BizError {
    ValidationFailed(String),   // 既存
    NotFound(String),           // 既存 — rollback 時の csv_import 不存在
    DuplicateProductCode(String), // 既存（本モジュールでは不使用）
    DatabaseError(DbError),     // 既存
    ImportError(String),        // 既存 — parse失敗、同一hashブロック、精算の識別と並びの拒否（順 4 を除く）、引き当ての食い違い等
    SettlementConflict { message: String, csv_import_id: i64 }, // 順 4 の拒否（文が名指す取込みID を構造でも持つ。CMD は kind import_error）
    IdempotencyConflict(String), // 既存（本モジュールでは不使用。INV-6方式）
}
```

**ImportError の使い分け（BIZ層）**:
- ファイル形式の問題（parse 失敗、サイズ超過）→ ImportError
- 重複ブロック（file_hash 一致で取込み済み）→ ImportError
- 精算の識別と並びの拒否（上の表の順 1〜3・5）、preview の引き当てと確定の取り直しの食い違い → ImportError（何も書かない）
- 順 4（同じ精算の別の版が active）→ SettlementConflict（何も書かない。名指した取込みの詳細へ画面が link を出すため、取込みID を構造で持つ。順 2・3 の文の取込みID は文だけ）
- マスタ未登録（error_rows で表現）→ エラーではなくプレビューの一部として返す
- ※ キャッシュ問題（token不正、有効期限切れ、不存在）はCMD層の責務。BIZ層には到達しない

---

### 更新履歴

| 日付 | PR | 内容 |
|------|-----|------|
| 2026-04-11 | PR #14 | 初版作成（BIZ-03 CSV取込みパイプライン: parse_and_validate / commit_csv_import / rollback_csv_import / list_csv_imports） |
| 2026-04-11 | PR #14 | 設計書定義に対し実装欠落していた sales_repo::find_csv_import_by_id を併せて実装。rollback_csv_import のステップ1（対象確認）が当初は別アプローチで検討されたが、設計書に明記された find_csv_import_by_id を採用 |
| 2026-04-13 | PR #22 | preview キャッシュ管理の所有責務を明確化（BIZ層→CMD層）。ロック区間最小化のため CachedPreview を CMD層 AppState に保持し、commit時に BIZ層へ渡す |
| 2026-08-03 | CSV取込み詳細 Design Phase | §15.6a get_csv_import_record を追加（CSV取込み詳細画面用 wire DTO、31 §12.6a read-only パターン、ErrorRow 再利用、resolve_movement_source 共有） |
| 2026-08-16 | PR #79 | SPEC-SDI-D1〜D8: 同日別hashを追加取込みとし、全件summary、追加確認flag、TX内snapshot再検証、insert-only commit、per-import rollbackを正本化。 |
| 2026-09-24 | ㉘ runtime ①（本 PR） | §15.0 現行buildの一時停止（SPEC-STOP-D1〜D3、BIZ-03 停止文言の正本）を追加。§15.4 / §15.5 は旧本体の記述と明記 |
| 2026-10-17 | JAN の別名の design lane（D-122） | §15.3 手順 4 b に、ヒット0件の行で前の JAN の別名を引く段と、引き当ての規則を preview と確定の取り直しで共有する文を足した（proposed・未実装） |
| 2026-10-23 | stocktake-count-runtime-b1（D-118 の b1） | 冒頭の D-118 の節を現行に。§15.0 の停止の節を外し、§15.2 の型（知らせ・`recheck_targets`）と §15.3〜§15.5 の手順を精算の識別・確定の取り直し・求め直しの新しい経路に置き換えた（[Plan Packet](../archive/plans/2026-10-22-stocktake-count-runtime-b1.md)） |
| 2026-10-25 | z004-rollback-entry（PR #197） | §15.6b（BIZ-03-D2）を現行に（取消の本体を `apply_rollback` に切り出し、取消と preview が呼ぶ。完了した棚卸しの判定は BIZ-06 の共有の関数）。冒頭の表の順 4 の文に取込みID を足し、`BizError::SettlementConflict` で返す（§15.10）（[Plan Packet](../plans/2026-10-23-z004-rollback-entry.md)） |
