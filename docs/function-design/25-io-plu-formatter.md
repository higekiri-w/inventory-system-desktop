## 12. IO-04: PLUフォーマッター

> **CV17 1.1.1 adapter profile（2026-07-03 field gate反映）**: 2026-04-08 オンライン調査の CV17 Ver.2.0.1 前提（10列TSV / 1始まり / product_code fallback）は、現場 CV17 1.1.1 の `スキャニングPLU(商品)` 取込みで受理されなかった。2026-07-02 field gate では、CV17 1.1.1 export template 由来の11列ヘッダ、`.txt` 拡張子、JAN/EAN互換のスキャニングコードが必要であることを確認した。2026-07-03 field gate では、この形状の `.txt` を `CV17 TXT import -> PC tool SD settings write -> SR-S4000 設定読み -> barcode/register behavior confirmation` の流れで反映できることを確認した。PLU総枠5000は通常PLUとスキャニングPLUで共有する。SR-S4000 本体取扱説明書（2026-07-03 確認）の工場出荷時配分は通常PLU 216 + スキャニングPLU 4,784 で、スキャニングPLU は通常PLU の最大登録数 216 の次の 217 から始まる（店の通常PLU で登録があるのは試しの 2 枠だけで、開始は登録の件数では決まらない）。開始メモリNo. は CV17 の設定で変えられる（CV17 の取説）が、店は 217 のまま変えていない（owner 回答、D-117 (7)）。設定UIには広げず、コード側 profile として memory No. `217..=5000` を保持する。店が開始を変えたときは profile を見直す。以下は SR-S4000 / CV17 1.1.1 用の現行 adapter profile とする。
>
> **D-072（2026-08-18）/ D-117（2026-10-16）**: memory No. は `plu_slots` に JAN 単位で永続割当し、入力行が保持する。Diff / Full とも CV17 へ投入でき、外部登録と free slot は出力しない（IO-04-D2、D4 / SPEC-PLS-D3、D5）。clear 行（枠を消す行）は書かない（D-117 (1)。IO-04-D3 の撤去）。

### 12.1 モジュール構成

```
src-tauri/src/
  io/
    mod.rs               -- pub mod plu_formatter を追加
    plu_formatter.rs     -- PLUファイル生成（本セクション）
```

### 12.2 型定義

**PluExportRow構造体**（BIZ-04から渡される行データ。33-biz-plu-export-service.md で定義）:
- memory_no: i64（`plu_slots` 由来）
- product_code: String
- jan_code: Option\<String\>
- name: String
- selling_price: i64
- tax_rate: String（"10" / "8" / "0"）
- department_name: String

**PluFileOutput構造体**（IO-04の戻り値。旧名 PluCsvOutput。実体が CV17 向けタブ区切り `.txt` であることに合わせ実体語彙へ改名。IO-04-D1、監査是正 順8 / P7b-3）:
- bytes: Vec\<u8\>（CP932エンコード済みPLUファイルバイト列）
- suggested_filename: String（例: "PLU_20260408.txt"）
- content_type: &'static str = "text/tab-separated-values"
- encoding: &'static str = "CP932"

**PluFormatError列挙型**:
```
enum PluFormatError {
    EncodingError { product_code: String, char: char, message: String },
    // CP932エンコード不能文字（文字集合の問題）。product_code と原因文字を含む
    InvalidScanningCode { product_code: String, message: String },
    // スキャニングPLUに必要なJANコードの欠落・桁数不正・チェックディジット不正
    TaxMappingError { product_code: String, tax_rate: String, message: String },
    // 税区分マッピング不正（業務値の問題）。product_code と不正値を含む
}
```

---

### 12.3 generate_plu_tsv

**関数要求**: PluExportRow のリストをカシオレジスターツール（CV17 1.1.1）のスキャニングPLUインポート形式（タブ区切りテキスト、CP932）に変換する

**シグネチャ**:
```
fn generate_plu_tsv(rows: &[PluExportRow]) -> Result<PluFileOutput, PluFormatError>
```

**処理ステップ**:

1. **ヘッダ行の構築**（タブ区切り、11列。CV17 1.1.1 export template の列名・順序に合わせる）:
   ```
   メモリNo.\tｽｷｬﾆﾝｸﾞｺｰﾄﾞ\t名称\t単価\t課税方式\t単品売り\t負単価\t品番PLU\tゼロ単価\t入力桁制限\t部門リンク
   ```
   `メモリーNo.` / `スキャニングコード` は CV17 1.1.1 で拒否されたため使わない。`入力桁制限` は必須列として出力し、値は field gate で観測した `無し` とする。

2. **各行のデータ変換**（rowsの順序を保持）:
   a. **メモリNo.** = 入力行の `memory_no`（`plu_slots` 由来）を 6 桁ゼロ埋めする。217..5000 の範囲外は拒否し、生成順から番号を導出しない（IO-04-D2 / SPEC-PLS-D3）。
   b. **ｽｷｬﾆﾝｸﾞｺｰﾄﾞ** = jan_code（product_code へ fallback しない）
      - jan_code は13桁数字かつJAN/EAN-13チェックディジット有効であること
      - JANなし、13桁以外、チェックディジット不正は出力不可
   c. **名称** = 商品名加工パイプライン（12.4参照）
   d. **単価** = selling_price（整数文字列）
   e. **課税方式** = 税区分マッピング（12.5参照）
   f. **固定列**: 単品売り=`いいえ`、負単価=`いいえ`、品番PLU=`いいえ`、ゼロ単価=`いいえ`、入力桁制限=`無し`（IO-04-D5: `単品売り=はい` は SR-S4000 で scan-call 即時の自動会計 + レシート発行を意味し、2026-08-19 Windows native L3 で実機確認した。operator の通常会計待ちにするため `いいえ` を固定値とする。PR #85 gated amendment 4）
   g. **部門リンク** = department_name

   clear 行は出力しない（IO-04-D3 は D-117 で撤去）。`external` / `free` slot は入力行として受け付けず、IO-04 からも出力しない（IO-04-D4 / SPEC-PLS-D5）。

3. **タブ区切りPLUファイル組み立て**:
   - 各フィールドをタブ（`\t`）で結合
   - 各行をCRLF（`\r\n`）で結合
   - ヘッダ行 + データ行を連結

4. **CP932エンコード**:
   - encoding_rs の SHIFT_JIS エンコーダを使用
   - エンコード不能文字が検出された場合 → PluFormatError::EncodingError（product_code + 原因文字を含む）
   - 注: 商品名は加工パイプラインで事前にCP932安全な文字に変換済みのため、通常はここでエラーにならない。防御的チェック

5. **PluFileOutput構築**:
   - bytes = CP932エンコード済みバイト列
   - suggested_filename = `PLU_{YYYYMMDD}.txt`（現在日付。CV17 1.1.1 の import dialog で既定選択可能な拡張子）
   - content_type = "text/tab-separated-values"
   - encoding = "CP932"

**入力例**:
```
[
  PluExportRow {
    product_code: "4976383262108",
    jan_code: Some("4976383262108"),
    name: "ハマナカ アミアミ極太 col.42",
    selling_price: 648,
    tax_rate: "10",
    department_name: "毛糸",
  }
]
```

**出力例**（CP932バイト列をUTF-8表記した場合）:
```
メモリNo.\tｽｷｬﾆﾝｸﾞｺｰﾄﾞ\t名称\t単価\t課税方式\t単品売り\t負単価\t品番PLU\tゼロ単価\t入力桁制限\t部門リンク\r\n
000217\t4976383262108\tﾊﾏﾅｶ ｱﾐｱﾐ極太 c\t648\t税1(内税)\tいいえ\tいいえ\tいいえ\tいいえ\t無し\t毛糸\r\n
```

**エラーハンドリング**:
- 空リスト → Ok（ヘッダのみのPLUファイルを返す。BIZ-04側で0件チェック済みのため到達しないが防御的に許容）
- CP932エンコード不能 → PluFormatError::EncodingError("商品 {product_code} の名称にCP932非対応文字 '{char}' が含まれています")
- JANなし / 13桁以外 / チェックディジット不正 → PluFormatError::InvalidScanningCode("商品 {product_code} のJANコードはスキャニングPLU書出しに使えません")
- 税区分マッピング不正値 → 12.5のエラー参照

---

### 12.4 商品名加工パイプライン

**順序固定**（順序を変えると差分バグが発生するため厳守）:

1. **`_` → 半角スペース置換**: カシオレジスターツールでは `_` + 次文字がダブルサイズ制御パターン。全除去ではなく置換で正当な商品名を保護
2. **タブ/改行 → 半角スペース置換**: タブ区切り構造を壊す制御文字を除去
3. **全角→半角カナ変換**: 全角カタカナ（アイウ）→ 半角カタカナ（ｱｲｳ）。レジ表示は半角カナ前提
4. **CP932エンコード**: Rust String（UTF-8）→ CP932バイト列に変換
5. **16バイト切り詰め**: CP932エンコード後の**バイト数**で16バイト以内に切り詰め。マルチバイト文字の途中で切れないよう、末尾がマルチバイトの第1バイトなら1バイト戻す
6. **末尾スペースパディング**: 16バイト未満なら半角スペース（0x20）で16バイトまでパディング

**注意事項**:
- 切り詰めの基準は**バイト数**（文字数ではない）。CP932では全角文字=2バイト、半角カナ=1バイト
- 半角カナ変換前に全角で16文字以内でも、変換後のバイト数で判定する
- `_` は全ての出現箇所を置換する（先頭のみではない）

---

### 12.5 課税方式マッピング

| products.tax_rate | → PLUファイル値 | 意味 |
|-------------------|--------------|------|
| "10" | `税1(内税)` | 標準税率10%・内税 |
| "8" | `税2(内税)` | 軽減税率8%・内税 |
| "0" | `非課税` | 非課税 |

**不正値**: 上記3値以外 → PluFormatError::TaxMappingError { product_code, tax_rate, message: "税率'{tax_rate}'はPLU書出しに対応していません" }（DB CHECK制約で通常到達しないが防御的にエラー）

---

### 12.6 ステートレス設計

- DB接続不要。純関数
- 入力は PluExportRow のスライス、出力は PluFileOutput
- 日付取得（suggested_filename用）のみ外部依存（chrono::Local::now()）
- テスト安定化のため、suggested_filename 生成は内部ヘルパーに切り出し、テストでは固定日付を注入可能にする（例: `generate_plu_tsv_with_date(rows, date)` を内部関数として持ち、公開関数は Local::now() で呼ぶ）
- 1 行の全 11 列（メモリNo.〜部門リンク。行末の CRLF は含まない）の CP932 の byte 列は、crate の中の関数 `format_plu_row(row)` の 1 つの出どころから作る。file に書く行（§12.3）と、BIZ-04 の confirm の照合の値（[33 §16.5](33-biz-plu-export-service.md)。この byte 列の SHA-256）は同じ関数を使うので、名称の 16 バイトの切り詰め（§12.4）で落ちる末尾は照合の値にも入らない（D-168）

---

### 12.7 テスト方針

**ゴールデンテスト**（hex literal で期待バイト列を固定）:
- 正常系: 1商品、複数商品
- 改行コード: CRLF（`\r\n` = 0x0D 0x0A）
- CP932マルチバイト境界: 濁点（ﾞ）/半濁点（ﾟ）を含む商品名
- 16バイト切り詰め: ちょうど16バイト、15バイト+マルチバイト（切れない）、長い商品名
- `_` → 半角スペース置換
- タブ文字 → 半角スペース置換
- 全角カタカナ → 半角カナ変換

**エラーテスト**:
- `test_encode_unmappable_char_returns_error`: CP932非対応文字（例: 絵文字）→ EncodingError に product_code と原因文字が含まれること
- 税区分マッピング不正値（例: "15"）→ エラー

**境界テスト**:
- 空リスト → ヘッダのみPLUファイル
- selling_price = 0 → "0"
- selling_price = 999999 → "999999"（6桁上限）
- PLU総枠5000を通常PLUとスキャニングPLUで共有し、工場出荷時配分（通常PLU 216）の profile で memory No. = 217始まりで出力されること
- suggested_filename = `.txt`
- jan_code = None / 13桁以外 / チェックディジット不正 → InvalidScanningCode

**手動互換性 gate（UI-08 implementation PR）**:
- CV17 1.1.1 へ生成 `.txt` を投入し、11列ヘッダ、CP932、CRLF、税区分、部門リンク、16byte商品名、工場出荷時配分のスキャニングPLU memory range（217〜5000）、13桁JANコードの受理可否を確認する
- 確認証跡は実JAN・実商品名・価格を含めず、受理可否、列名差異、件数、エラー文言だけを記録する
- CV17 import成功だけではSR-S4000反映成功を意味しない。SD-card書出し / SR-S4000設定読込 / register scan-call の確認はUI-08 PRのmanual gateとして残す

---

### 12.8 対応不変条件

- **INV-8: products物理DELETE禁止** — IO-04自体はDBに触れないため直接関係しないが、入力のPluExportRowはBIZ-04経由で取得されるため間接的に保証
