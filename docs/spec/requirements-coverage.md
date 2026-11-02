# 公開要求 coverage ledger

## 目的

本書は、owner が repo 外で保管する要求原典の ID 集合と、公開 repository 内の定義先を対応付ける正本である。原典の文面や機微な業務事実は転記せず、製品判断として必要な公開要約だけを保持する。

要求の公開正本は次の複合構成とする。

1. [requirements.md](requirements.md): REQ インベントリと traceability policy
2. 本 ledger: 原典 ID 全件の公開要約、定義先、状態、差分理由
3. `docs/ARCHITECTURE.md`、`docs/SCREEN_DESIGN.md`、`docs/function-design/`、`docs/architecture/`: 詳細契約と設計判断

## 状態

- `current`: 公開定義が現行契約を表す。実装・テスト coverage は traceability 文書で別に確認する。
- `partial`: 一部は現行契約だが、残りは後続または非スコープである。
- `deferred`: 要求は保持するが、現行の実装契約としては未着手である。
- `superseded`: 原典の前提を後発の検証・設計判断が置換した。現行契約はリンク先を正とする。

## Coverage ledger

| ID | 公開要約 | 現行の定義先 | 状態 | 差分・後続理由 |
|---|---|---|---|---|
| REQ-101 | 商品を新規登録できる | `spec/requirements.md`; `function-design/30-biz-product-service.md`; `function-design/51-ui-product-form.md` | current | — |
| REQ-102 | 商品情報を修正し、変更履歴と連携状態を保てる | `spec/requirements.md`; `function-design/30-biz-product-service.md`; `function-design/51-ui-product-form.md` | current | — |
| REQ-103 | 商品を検索・一覧表示できる | `spec/requirements.md`; `function-design/30-biz-product-service.md`; `function-design/50-ui-product-list.md` | current | — |
| REQ-104 | 商品マスタを CSV から一括取込でき、任意列 `PLU対象` を preview / 適用できる | `spec/requirements.md`; `function-design/26-io-product-csv-importer.md`; `function-design/30-biz-product-service.md`; `function-design/60-ui-product-import.md` | current | — |
| REQ-201 | 仕入入庫を在庫変動と業務記録として保存できる | `spec/requirements.md`; `function-design/31-biz-inventory-service.md`; `function-design/61-ui-receiving.md` | current | — |
| REQ-202 | 返品・交換と在庫への反映有無を記録できる | `spec/requirements.md`; `function-design/31-biz-inventory-service.md`; `function-design/63-ui-return-exchange.md`; `function-design/65-inventory-record-traceability.md` | current | — |
| REQ-203 | 自動取込で扱わない販売出庫を手動記録できる | `spec/requirements.md`; `function-design/31-biz-inventory-service.md`; `function-design/62-ui-manual-sale.md` | current | — |
| REQ-204 | 廃棄・破損等の出庫を理由付きで記録できる | `spec/requirements.md`; `function-design/31-biz-inventory-service.md`; `function-design/64-ui-disposal.md`; `function-design/65-inventory-record-traceability.md` | current | — |
| REQ-205 | 棚卸しを中断・再開し、実数との差異を在庫へ反映できる | `spec/requirements.md`; `function-design/35-biz-stocktake-service.md`; `function-design/73-ui-stocktake.md` | current | — |
| REQ-301 | 商品別の現在庫と判断に必要な情報を照会できる | `spec/requirements.md`; `function-design/58-ui-stock-inquiry.md` | current | — |
| REQ-302 | 在庫切れ・在庫少の商品を一覧できる | `spec/requirements.md`; `function-design/53-ui-home.md`; `function-design/58-ui-stock-inquiry.md` | current | — |
| REQ-303 | 商品別の在庫変動を時系列で追跡できる | `spec/requirements.md`; `function-design/65-inventory-record-traceability.md`; `function-design/66-ui-stock-movements.md` | current | — |
| REQ-401 | POS 由来データを検証・取込し、現行運用の日報情報を保存できる | `spec/requirements.md`; `function-design/29-io-daily-report-parser.md`; `function-design/32-biz-csv-import-service.md`; `function-design/37-biz-daily-report-import-service.md`; `function-design/55-ui-csv-import.md` | superseded | 単一形式前提は後発の現場確認と D-022/D-025 により二つの取込 track へ置換した |
| REQ-402 | 商品マスタから対応 POS 向け PLU データを永続 memory No. で生成・保存できる | `spec/requirements.md`; `function-design/25-io-plu-formatter.md`; `function-design/33-biz-plu-export-service.md`; `function-design/67-ui-plu-export.md` | superseded | 原典の形式・容量前提を adapter 契約と D-072 の slot 永続割当へ置換した（メモリNo. 採番規則は REQ-907 で永続割当へ置換）。Diff / Full とも投入可 |
| REQ-907 | レジ占有を毎日の SD の読取りで照合し、PLU slot の JAN ごとの永続割当を安全に管理できる | `spec/requirements.md`; `db-design/plu-tables.md`; `function-design/33-biz-plu-export-service.md`; `function-design/30-biz-product-service.md`; `function-design/37-biz-daily-report-import-service.md`; `function-design/67-ui-plu-export.md` | current | D-117 で解放・bulk onboarding・段階移行状態を外した（runtime は後続の lane） |
| REQ-403 | POS 側とシステム側の部門別売上集計を照合できる | `spec/requirements.md`; `architecture/biz-task-specs.md`; `db-design/pos-tables.md`; `db-design/master-tables.md` | deferred | UI-13 の在庫整合性（REQ-904）とは別責務で task 未割当。自動修正は行わない |
| REQ-501 | 日次売上を商品・部門の観点で確認できる | `spec/requirements.md`; `function-design/27-io-report-csv-exporter.md`; `function-design/34-biz-sales-service.md`; `function-design/56-ui-daily-sales.md` | current | — |
| REQ-502 | 月次売上を商品・部門・比較の観点で集計できる | `spec/requirements.md`; `function-design/34-biz-sales-service.md`; `function-design/57-ui-monthly-sales.md` | current | — |
| SP-101 | 商品コード、名称、分類、価格、在庫単位等を検証して商品を登録する | `function-design/30-biz-product-service.md`; `function-design/51-ui-product-form.md` | current | — |
| SP-102 | 商品情報の変更、廃番、売価履歴、PLU 再出力要否を一貫して扱う | `function-design/30-biz-product-service.md`; `function-design/51-ui-product-form.md` | current | — |
| SP-102-08 | 取引先（メーカー）・部門で商品を絞り込み、売価・原価を一括で改定する（REQ-105） | `spec/requirements.md`; `function-design/30-biz-product-service.md`; `function-design/77-ui-bulk-price-revision.md` | current | 2026-08-21 原典全数突合で dropped-silently と判明し復活 |
| SP-103 | コード・名称・分類・状態で商品を検索し、業務単位で数量を表示する | `function-design/30-biz-product-service.md`; `function-design/58-ui-stock-inquiry.md` | current | — |
| SP-103-04 | 商品一覧の基本列で原価を確認できる（REQ-103） | `function-design/50-ui-product-list.md` | current | UI-01a-D13 で原価列を基本列に追加 |
| SP-104 | 一括取込前に行検証と重複時の選択を行い、正常な対象だけを確定して結果を示す | `function-design/60-ui-product-import.md` | partial | テンプレート配布は現行非スコープ |
| SP-201 | 複数商品の仕入入庫を数量・原価・任意の取引先とともに記録する | `function-design/31-biz-inventory-service.md`; `function-design/61-ui-receiving.md` | current | — |
| SP-202 | 返品・交換の増減方向と POS 側処理有無を記録し、二重反映を避けて証跡を追跡可能にする | `function-design/31-biz-inventory-service.md`; `function-design/63-ui-return-exchange.md`; `function-design/65-inventory-record-traceability.md` | partial | 印刷は後続 |
| SP-203 | 自動取込対象外の販売を理由・数量とともに記録し、在庫を減算する | `function-design/31-biz-inventory-service.md`; `function-design/62-ui-manual-sale.md`; `function-design/65-inventory-record-traceability.md` | current | — |
| SP-204 | 廃棄・破損等を商品・数量・業務日・種別・理由とともに記録し、在庫とロス記録へ反映する | `function-design/31-biz-inventory-service.md`; `function-design/64-ui-disposal.md`; `function-design/65-inventory-record-traceability.md` | partial | 集計と追加証跡は後続 |
| SP-205 | 棚卸しの開始、中断再開、絞込み、実数入力、差異確認、確定を安全に扱う | `function-design/35-biz-stocktake-service.md`; `function-design/73-ui-stocktake.md` | current | — |
| SP-301 | 商品の在庫、価格、直近の入出庫情報を検索結果から確認する | `function-design/58-ui-stock-inquiry.md` | current | — |
| SP-302 | 在庫ゼロまたは設定閾値以下の商品を、廃番状態を考慮して示す | `function-design/53-ui-home.md`; `function-design/58-ui-stock-inquiry.md` | current | — |
| SP-303 | 在庫変動を日時・種別・増減・変動後在庫・元記録とともに時系列表示し、期間・種別で絞り込む | `function-design/66-ui-stock-movements.md`; `function-design/65-inventory-record-traceability.md` | current | — |
| SP-401 | POS 由来ファイルを parse、validate、preview、commit の境界で扱い、重複や失敗を安全に回復する | `function-design/29-io-daily-report-parser.md`; `function-design/32-biz-csv-import-service.md`; `function-design/37-biz-daily-report-import-service.md`; `function-design/55-ui-csv-import.md` | superseded | 現行運用は日報 bundle と商品別 track を分離する |
| SP-402 | PLU 候補を全件・差分で生成し、不適合と上限を検査し、保存確認後だけ出力状態を更新する | `function-design/25-io-plu-formatter.md`; `function-design/33-biz-plu-export-service.md`; `function-design/67-ui-plu-export.md` | superseded | 原典の形式・上限前提は現行 adapter 契約に置換済み |
| SP-403 | POS 側とシステム側の同日部門集計を照合し、数量・金額差と原因調査材料を示す | 本 ledger; `architecture/biz-task-specs.md`; `db-design/pos-tables.md`; `db-design/master-tables.md` | deferred | task 未割当。REQ-904 の在庫整合性と混同せず、自動修正は非スコープ |
| SP-501 | 日付を指定して日次売上と商品・部門の内訳を確認し、外部利用可能な形へ出力する | `function-design/27-io-report-csv-exporter.md`; `function-design/34-biz-sales-service.md`; `function-design/56-ui-daily-sales.md` | partial | 日次表示と CSV 出力は現行。印刷は後続設計で扱う |
| SP-502 | 月を指定して商品・部門別集計、順位、比較を確認する | `function-design/34-biz-sales-service.md`; `function-design/57-ui-monthly-sales.md` | partial | 商品・部門別集計、順位、比較、CSV 出力は現行。一部の絞込みと印刷は後続設計で扱う |
| QR-01 | 金額を小数で扱える | 本 ledger; `docs/DB_DESIGN.md`; `function-design/30-biz-product-service.md`; `function-design/34-biz-sales-service.md` | deferred | 現行は整数円契約であり、そのまま対応済みとは扱わない |
| QR-02 | 開発者不在でも単独運用でき、必要な手順を確認できる | `docs/PROJECT_HANDOFF.md`; `docs/DEV_SETUP_CHECKLIST.md`; `function-design/68-ui-backup-restore.md` | current | — |
| QR-03 | 税率変更に追従できる | `function-design/30-biz-product-service.md`; `function-design/34-biz-sales-service.md` | partial | 現行は固定候補を扱う。任意の制度変更への追従設計は後続 |
| QR-04 | 複数世代のバックアップを保持する | `function-design/71-mnt-backup.md`; `function-design/68-ui-backup-restore.md` | current | 現行 retention 契約が要求を包含する |
| QR-05 | バックアップの自動実行、状態確認、復元を行える | `function-design/71-mnt-backup.md`; `function-design/68-ui-backup-restore.md` | current | — |
| QR-06 | 主要操作を記録し、検索・閲覧・保持・外部出力できる | `function-design/72-mnt-log-manager.md`; `function-design/74-ui-operation-logs.md`; `function-design/65-inventory-record-traceability.md` | partial | 記録・閲覧と、保持日数（既定は 72 MNT-02-D1）を過ぎた記録の削除は現行。外部出力は後続、旧 archive 前提は owner decision により保持日数の cleanup へ supersede |

## 完全性契約

- owner 保管原典の distinct ID は REQ 17、SP 19、QR 6 の計 42 件である。
- ledger の ID は重複不可で、上記 42 件との差集合は常に空でなければならない。
- `partial` / `deferred` / `superseded` を `current` に変更するときは、対応する source design と traceability evidence を同じ変更で更新する。
- 原典の具体的な業務値・発言・所在・ファイル名・保管 path は本書へ転記しない。

## Semantic audit protocol

構造検査だけでは意味のずれを検出できないため、原典を参照できる独立 reviewer が次を全 42 行で確認する。

1. ID と公開要約が原典の業務目的を保持し、具体的な業務値・発言を持ち込んでいない。
2. 定義先が実在する source design で、公開要約と同じ責務を定義している。placeholder や自己参照だけの行は失敗とする。
3. `current` は公開定義が現行契約を表す。未対応部分があれば `partial` または `deferred` に下げる。
4. `partial` / `deferred` / `superseded` は差分・後続理由が source design と一致する。
5. 似た名前の別契約を混同していない。特に REQ-403 / SP-403 の POS 部門別売上照合と、REQ-904 / UI-13 の在庫整合性を別々に追跡する。

review evidence は `ID / status / pass-fail` と finding だけを記録し、原典の文面、実 path、検査 log は tracked artifact へ転載しない。42 行すべての pass が揃わなければ本 ledger は公開候補にできない。
