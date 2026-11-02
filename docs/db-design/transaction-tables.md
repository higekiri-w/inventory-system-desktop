# テーブル定義（トランザクション）

## 計数の置き場と在庫の求め直しの契約（D-118。(a) と b1 で実装済み）

[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)（SPEC-STK-CNT-D1・D2）との接続。以下の入庫・返品交換・手動販売・廃棄の header / item の列と符号規約は変えない。(a)（ADR SPEC-STK-CNT-D12）で入ったのは全部の項目（2 つ目の窓の判定は求め直しの側にあり、計数を保存する経路は b1 で入った）。

- 在庫を動かす TX は、header / item・inventory_movements（並びの鍵 `business_date` = header の日付〈`receiving_date`・`return_date`・`sale_date`・`disposal_date`〉、`phase` = 0）を書き、同じ TX で触れた商品を求め直して写し（`products.stock_quantity` ほか）を書く。どれかの失敗で TX 全体を戻す。
- header の日付は業務の日付 D で、計数に入っているかは ADR D3 の窓の判定表で決まる（既定の計数と同じ日付なら決まらない、fresh な計数より先に保存したものは入っている、など）。日付を計数の日付より前にした入力は、計数のある商品の在庫に足されない（数えた数に入っているとみなす。ADR の Consequences）。
- レジ処理済みの返品など在庫を動かさない経路は movement を作らない（今のまま）。冪等な要求の再送は新しい movement を作らない（今のまま）。

## 金額の丸めと、原価の 1/100 円

入庫の原価小計・合計と廃棄のロス原価は、明細の `quantity` と原価と商品の単位の価格の基準数量から BIZ が求める（[共通規則](../function-design/10-common-rules.md) SPEC-UNIT-D6）。列は足さず、行の金額を保存しない。`receiving_items` と `disposal_items` の原価は `cost_price_centi`（1/100 円。vB が改名した。既存の値は変換せず、行の無い DB にだけ当たる〈22 §16、D-119〉。[master-tables](master-tables.md) の「単位と原価の精度の契約」、`決定（owner、D-113 J2）`）。

---

> **親文書**: [DB_DESIGN.md](../DB_DESIGN.md)

---

## 4-5. receiving_records + receiving_items（入庫記録）

### 役割
商品が届いたとき（仕入入庫）の記録。ヘッダに日付・取引先、明細に商品ごとの数量・原価。

### receiving_records カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | 入庫記録ID |
| supplier_id | INTEGER | FK → suppliers.id, NULLABLE | 取引先 |
| receiving_date | TEXT | NOT NULL | 入庫日 |
| note | TEXT | NULLABLE | 備考 |
| idempotency_key | TEXT | NOT NULL, 非空CHECK, UNIQUE INDEX | 再送を識別するキー（migration v2） |
| request_fingerprint | TEXT | NOT NULL, 非空CHECK | 同じキーで異なる内容が送られた場合の検出用（migration v2） |
| created_at | TEXT | NOT NULL | 作成日時 |

### receiving_items カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | 明細ID |
| receiving_record_id | INTEGER | FK → receiving_records.id, NOT NULL | 親ヘッダ |
| product_code | TEXT | FK → products.product_code, NOT NULL | 商品コード |
| quantity | INTEGER | NOT NULL | 入庫数量（個 or cm） |
| cost_price_centi | INTEGER | NOT NULL | 入庫時点の原価（1/100 円。単価変動に対応） |

### 設計意図
- **ヘッダ＋明細に分けた理由**: 1回の入庫で複数商品を記録する（伝票1枚=ヘッダ1件、商品行=明細N件）。画面ラフの入庫リストがこの構造
- **cost_price_centiを明細に持つ理由**: 入庫時点の原価を記録。商品マスタの原価は後から変わる可能性があるが、入庫時にいくらで仕入れたかは変わらない事実
- `receiving_records.supplier_id` は取引先統合（SPEC-SUP-D4）で残す側の取引先へ付け替わり得る。統合は同一実体の重複解消であり、入庫日・明細・原価の事実は変更しない

### 入出庫記録追跡の完成形（2026-06-27 追加）

現行の冪等性列は入庫・返品交換・手動販売・廃棄破損の各ヘッダに実装済み。追加DDL・既存行のbackfillは [migration v2](../function-design/22-mnt-migration.md#9-mnt-03-追加-migration-v2冪等性カラム)、同一キーの再送と内容不一致の判定は [入出庫BIZ](../function-design/31-biz-inventory-service.md) を参照する。以下の取消・訂正用フィールドは将来設計であり、現行ERには含めない。

入庫、返品・交換、手動販売、廃棄・破損の各ヘッダは、完成形では後から一覧・詳細で追える業務記録として扱う。詳細は [65-inventory-record-traceability.md](../function-design/65-inventory-record-traceability.md) §65.6 を正とする。

- 各ヘッダは `status`（active / canceled / corrected）、取消日時、取消理由、訂正元/訂正先を持つ方向で migration を設計する。
- 取消・訂正は物理削除ではなく、ヘッダ状態変更 + 逆方向 `inventory_movements` 追加で表現する。
- 各業務記録詳細は関連 `inventory_movements` を表示し、商品別在庫変動履歴からも元記録へ戻れるようにする。

### 業務シナリオ例
```
取引先Aから毛糸3種類が届いた:
  receiving_records: id=1, supplier_id=1, receiving_date="2026-03-21"
  receiving_items: 
    id=1, receiving_record_id=1, product_code="4976383262108", quantity=12, cost_price_centi=11100
    id=2, receiving_record_id=1, product_code="4976383262207", quantity=8, cost_price_centi=11100
    id=3, receiving_record_id=1, product_code="4979738052116", quantity=10, cost_price_centi=22200

→ 同時に products.stock_quantity をそれぞれ +12, +8, +10
→ 同時に inventory_movements に3行挿入（movement_type="receiving"）
```

---

## 6-7. return_records + return_items（返品・交換記録）

### 役割
返品・交換の帳面記録。レシート画像の添付先。レジ戻し済みかどうかのフラグで在庫処理を分岐。

### return_records カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | 返品記録ID |
| return_type | TEXT | NOT NULL | 種別。'return'（返品）/ 'exchange'（交換） |
| return_date | TEXT | NOT NULL | 返品日 |
| register_processed | BOOLEAN | NOT NULL, DEFAULT 1 | レジ戻し処理済みか。1=済み（在庫はCSVで自動反映）、0=未処理（システムで在庫増減） |
| receipt_image_path | TEXT | NULLABLE | レシート画像のファイルパス |
| note | TEXT | NULLABLE | 備考 |
| idempotency_key | TEXT | NOT NULL, 非空CHECK, UNIQUE INDEX | 再送を識別するキー（migration v2） |
| request_fingerprint | TEXT | NOT NULL, 非空CHECK | 同じキーでの内容不一致を検出（migration v2） |
| created_at | TEXT | NOT NULL | 作成日時 |

### return_items カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | 明細ID |
| return_record_id | INTEGER | FK → return_records.id, NOT NULL | 親ヘッダ |
| product_code | TEXT | FK → products.product_code, NOT NULL | 商品コード |
| direction | TEXT | NOT NULL | 'in'（戻り=在庫増）/ 'out'（渡し=在庫減） |
| quantity | INTEGER | NOT NULL | 数量 |

### 設計意図
- **register_processedフラグの理由（Z004実機検証結果に基づく）**: レジの「戻」モードで処理した返品はZ004にマイナス値で出る。CSV取込みで在庫が自動反映されるから、システム側で在庫を動かすと二重計上になる。このフラグで分岐
- **directionの理由**: 交換は「戻り商品（in）」と「渡し商品（out）」の2種類がある。返品はinのみ。1つのreturn_recordに対してin/outの明細が混在する

### 業務シナリオ例
```
ケース1: 通常の交換（レジで戻し処理済み）
  return_records: id=1, return_type="exchange", register_processed=1
  return_items:
    id=1, return_record_id=1, product_code="4976383262108", direction="in", quantity=1  （戻り）
    id=2, return_record_id=1, product_code="4976383262207", direction="out", quantity=1 （渡し）
  → register_processed=1 なので在庫は動かさない。帳面記録のみ
  → CSV取込み時にZ004のマイナス値で在庫が自動反映される

ケース2: 色交換（レジ打ちしない）
  return_records: id=2, return_type="exchange", register_processed=0
  return_items:
    id=3, return_record_id=2, product_code="HZ-0012", direction="in", quantity=1  （戻り）
    id=4, return_record_id=2, product_code="HZ-0013", direction="out", quantity=1 （渡し）
  → register_processed=0 なのでシステムで在庫増減
  → products.stock_quantity: HZ-0012は+1、HZ-0013は-1
  → inventory_movementsに2行挿入
```

---

## 8-9. manual_sales + manual_sale_items（手動販売出庫）

### 役割
CSV取込みで拾えない販売を補完記録する。使い道は PLU 未登録の新商品の販売と、[10](../function-design/10-common-rules.md) SPEC-UNIT-D4 の店に聞いてから決める項の既定 (a) の間の長さの商品の売った分の記録。

### 【重要】利用者ヒアリング結果（2026-03-28）
利用者に「レジを打たずにお金だけもらって商品を渡すことはあるか」を確認した結果：
- 家の商品は必ずレジを打つ → レジ外販売は無い
- 2階の預かり品はレジ打たないが月末に2階に売上を渡す別会計 → システム対象外
- レジ故障時も直ったらレジ打ちする → 最終的にZ004に載る

**結論: PLU全商品登録後、レジを通さない販売は実質ゼロ。手動販売出庫を使うのは「PLU登録が間に合わなかった新商品が売れた場合」のみ。**

今は [10](../function-design/10-common-rules.md) SPEC-UNIT-D4 の店に聞いてから決める項の既定 (a)（D-119 の U5 = c）で、長さの商品の売った分も手動販売で記録する。

### manual_sales カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | 手動販売ID |
| sale_date | TEXT | NOT NULL | 販売日 |
| reason | TEXT | NOT NULL | 理由。'plu_unregistered'（PLU未登録の新商品）/ 'other'（その他） |
| note | TEXT | NULLABLE | 備考（自由記述） |
| idempotency_key | TEXT | NOT NULL, 非空CHECK, UNIQUE INDEX | 再送を識別するキー（migration v2） |
| request_fingerprint | TEXT | NOT NULL, 非空CHECK | 同じキーでの内容不一致を検出（migration v2） |
| created_at | TEXT | NOT NULL | 作成日時 |

### manual_sale_items カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | 明細ID |
| manual_sale_id | INTEGER | FK → manual_sales.id, NOT NULL | 親ヘッダ |
| product_code | TEXT | FK → products.product_code, NOT NULL | 商品コード |
| quantity | INTEGER | NOT NULL | 数量（個 or cm） |
| amount | INTEGER | NOT NULL | 金額（円） |

### 設計意図
- **sale_recordsとは別テーブルにした理由**: sale_recordsはCSV取込み経由の自動売上が大量に入る。手動販売は数が少なく、理由（reason）や備考が必要。テーブルを分けた方が各テーブルの責務が明確
- **amountを明細に持つ理由**: 手動販売時は商品マスタの売価と違う金額で売ることがある（値引き等）。実際の販売金額を記録する
- **reasonを簡素化した理由**: 利用者ヒアリング（2026-03-28）の結果、レジ外販売は実質ゼロで、当時の使うケースは「PLU未登録新商品」だった。'off_register'や'fabric_cut'は実態に合わないため削除。今の手動販売の使い道は、PLU 未登録の新商品と、[10](../function-design/10-common-rules.md) SPEC-UNIT-D4 の店に聞いてから決める項の既定 (a)（長さの商品は在庫連動の自動提案が `false` で、売った分を手動販売で記録する。店主の答えが無い間の形、D-119 の U5 = c）。reason の値（`plu_unregistered`・`other`）と既定（[62](../function-design/62-ui-manual-sale.md) UI-04-D3）はこのまま

### 二重減算問題と対策（2026-03-28 分析・確定）
**問題**: PLU登録済み商品を手動販売出庫で記録すると、CSV取込みでも在庫が減り、手動でも減る＝二重減算
**発生条件**: PLU登録済み商品を手動販売出庫の対象にした場合のみ
**対策（案1＋案3の組み合わせ）**:
- **画面の用途限定**: 説明バー「PLU登録が間に合わなかった新商品のみ。レジで打てる商品はCSV取込みで自動記録されます」
- **PLU登録済み商品の警告**: 手動販売出庫で商品を選んだとき、その商品がPLU登録済み（plu_dirty=0）なら「この商品はレジで打てます。手動販売出庫すると在庫が二重に減る可能性があります」と警告表示
- **警告は無視可能**: 利用者が「わかった上で入力したい」場合（例: 何らかの特殊事情）は警告を閉じて続行できる

---

## 10-11. disposal_records + disposal_items（廃棄・破損記録）

### 役割
販売できなくなった商品の記録。在庫から差し引き、ロス実績として記録する。

### disposal_records カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | 廃棄記録ID |
| disposal_date | TEXT | NOT NULL | 廃棄日 |
| idempotency_key | TEXT | NOT NULL, 非空CHECK, UNIQUE INDEX | 再送を識別するキー（migration v2） |
| request_fingerprint | TEXT | NOT NULL, 非空CHECK | 同じキーでの内容不一致を検出（migration v2） |
| created_at | TEXT | NOT NULL | 作成日時 |

### disposal_items カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | 明細ID |
| disposal_record_id | INTEGER | FK → disposal_records.id, NOT NULL | 親ヘッダ |
| product_code | TEXT | FK → products.product_code, NOT NULL | 商品コード |
| disposal_type | TEXT | NOT NULL | 種別。'disposal'（廃棄）/ 'damage'（破損）/ 'other' |
| quantity | INTEGER | NOT NULL | 数量 |
| cost_price_centi | INTEGER | NOT NULL | 原価（1/100 円。ロス金額の計算用） |
| reason | TEXT | NOT NULL | 理由（例: 袋破れ、色焼け） |

### 設計意図
- **cost_price_centiを明細に持つ理由**: ロス原価の合計を画面で表示するため（棚卸し時のロス把握）。商品マスタの原価は変わる可能性があるが、廃棄時点の原価を記録しておく
- **disposal_typeを明細に持つ理由**: 1回の記録で「破損1件＋廃棄2件」のように種別が混在できる
