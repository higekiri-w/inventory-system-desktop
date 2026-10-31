# REQ インベントリ

> **親文書**: [README.md](README.md)（spec index）
> **出典**: owner が repo 外で保管する要求原典の REQ 17 本 + 開発拡張 ID 8 本（コードコメント / Design Phase 由来）+ Design Phase 補足 ID 7 本 + 原典 SP 復活対応 ID 1 本。公開要求の全 ID 対応と状態は [requirements-coverage.md](requirements-coverage.md) を正とする
> **役割**: `generate_traceability` bin（`cargo run --bin generate_traceability -- --check`）の T2 phantom REQ 判定 / T3 テスト 0 本 WARN の判定基盤。本表にない REQ ID をテストが使うと CI / pre-push が ERROR になる
> **coverage**: `required` はテスト 0 本なら T3 WARN、`deferred` は未実装要求として T3 WARN 対象外。実装着手時に `required` へ戻す。
> **更新規約**: owner 保管原典の変更を採用するときは、本表、[requirements-coverage.md](requirements-coverage.md)、対応 design、traceability を同じ変更で同期する。原典の文面や機微な業務事実は公開 repository へ転記しない

## REQ 一覧（33 行）

| REQ ID | 名称 | 対応タスク | 出典 | coverage |
|---|---|---|---|---|
| REQ-101 | 商品を新規登録できること | UI-01b, BIZ-01 | 要求仕様書 v2.1 | required |
| REQ-102 | 商品情報を修正できること | UI-01b（価格履歴セクション）, BIZ-01 | 要求仕様書 v2.1 | required |
| REQ-103 | 商品を検索・一覧表示できること | UI-01a, BIZ-01 | 要求仕様書 v2.1 | required |
| REQ-104 | 商品マスタをCSVから一括インポートできること（任意列 `PLU対象` は 1 / 0 / 空欄） | UI-01c, BIZ-01 | 要求仕様書 v2.1 + D-072 | required |
| REQ-105 | 取引先（メーカー）・部門で商品を絞り込み、売価・原価を一括で改定できること | UI-14, BIZ-01 | 要求仕様書 v2.1（SP-102-08） | required |
| REQ-106 | 取引先（メーカー/ブランド）を商品登録・価格改定・取引先管理画面から追加登録できること | UI-01b, UI-14, UI-15, BIZ-01 | Design Phase 補足 2026-08-22 / owner 裁定 2026-08-25 | required |
| REQ-107 | 取引先（メーカー/ブランド）の名称を変更し、重複した取引先を統合できること | UI-15, BIZ-01 | Design Phase 補足 2026-08-25 | required |
| REQ-201 | 仕入れによる入庫を記録できること | UI-02, BIZ-02 | 要求仕様書 v2.1 | required |
| REQ-202 | 顧客返品・交換による入庫を記録できること | UI-03, BIZ-02 | 要求仕様書 v2.1 | required |
| REQ-203 | CSV取込みで記録されない販売出庫を手動で記録できること | UI-04, BIZ-02 | 要求仕様書 v2.1 | required |
| REQ-204 | 廃棄・破損による出庫を記録できること | UI-05, BIZ-02 | 要求仕様書 v2.1 | required |
| REQ-205 | 棚卸しによる在庫数の補正ができること | UI-10, BIZ-06 | 要求仕様書 v2.1 | required |
| REQ-206 | 入出庫系の業務記録を後から一覧・詳細で追跡できること | UI-02b, UI-03b, UI-04b, UI-05b, UI-07b, UI-10b | Design Phase 補足 2026-06-27 | required |
| REQ-207 | 在庫変動履歴から元業務記録へ相互参照できること | UI-06c, BIZ-02 | Design Phase 補足 2026-06-27 | required |
| REQ-208 | 入出庫系の業務記録を物理削除せず取消・訂正できること | UI-02b, UI-03b, UI-04b, UI-05b, BIZ-02 | Design Phase 補足 2026-06-27 | deferred |
| REQ-209 | 仕入入庫の明細原価が商品マスタの原価と異なる場合に検知し、マスタ原価の更新を提案できること | UI-02, BIZ-02 | Design Phase 補足 2026-08-22 | required |
| REQ-210 | 仕入れた品を別の商品に詰め替え・切り分けた在庫の動きを記録できること | UI-16, UI-16b, BIZ-02 | Design Phase 補足 2026-10-27（D-143） | required |
| REQ-301 | 商品別の在庫数を照会できること | UI-00, UI-06a | 要求仕様書 v2.1 | required |
| REQ-302 | 在庫切れ・在庫少の商品を一覧表示できること | UI-00, UI-06b | 要求仕様書 v2.1 | required |
| REQ-303 | 商品ごとの在庫変動履歴を時系列で参照できること | UI-06c, BIZ-02 | 要求仕様書 v2.1 | required |
| REQ-401 | POS売上データの取込み（current SALES は Z001/Z002/Z005 日報取込み。既存Z004は商品別取込み） | UI-07, IO-07, BIZ-08, CMD-12, BIZ-03 | 要求仕様書 v2.1 + D-022 + D-025 | required |
| REQ-402 | 商品マスタからPLU登録用データを生成し、永続 memory No. でレジに書き出せること | UI-08, BIZ-04 | 要求仕様書 v2.1 + D-072 | required |
| REQ-403 | POS CSVの部門別集計とシステムの売上記録を突合し、整合性を検証できること | 未割当（UI-13/REQ-904とは別） | owner 保管原典 | deferred |
| REQ-501 | 日次の売上一覧をJANコード・商品名・個数・金額付きで表示できること | UI-09a, BIZ-05 | 要求仕様書 v2.1 | required |
| REQ-502 | 月次の売上集計・傾向を表示できること | UI-09b, BIZ-05 | 要求仕様書 v2.1 | required |
| REQ-700 | 診断ログ | MNT-04 | 開発拡張 | required |
| REQ-901 | バックアップ | MNT-01, UI-11b | 開発拡張 | required |
| REQ-902 | ログ管理（操作ログ記録/一覧/自動削除） | MNT-02 | 開発拡張 | required |
| REQ-903 | マイグレーション/DB基盤（初期化/スキーマ更新） | MNT-03, IO-01 | 開発拡張 | required |
| REQ-904 | 整合性チェック（在庫数突合/修復） | BIZ-07, UI-13 | 開発拡張 | required |
| REQ-905 | 設定管理（設定CRUD/エラー変換） | CMD-11, UI-11a | 開発拡張 | required |
| REQ-906 | 画像管理（レシート画像保存） | IO-06 | 開発拡張 | required |
| REQ-907 | PLU slot のレジ占有を毎日の SD の読取りで照合し、JAN ごとの永続割当を安全に管理できること | BIZ-04, BIZ-01, BIZ-08, UI-08 | 開発拡張（D-072、D-117） | required |

## 補足

- 開発拡張 ID（REQ-700, REQ-901〜907）は owner 保管原典の ID 集合には含まれない。REQ-907 は D-072 の design-first 契約として追加し、D-117 で解放・bulk onboarding・段階移行状態を外した（runtime は後続の lane）。
- Design Phase 補足 ID（REQ-106 / REQ-107 / REQ-206〜210）は既存要求の実装・聞き取りで露出した横断要件である。
- REQ-107 は取引先管理の実装により `coverage=required` とする。
- REQ-208 は実装着手まで `coverage=deferred` とする。
- REQ-106 / REQ-209 は価格改定支援の設計契約（SPEC-PRV-D11）として `coverage=required` とする。
- REQ-403 は POS 部門別売上照合として `coverage=deferred` とし、`--check` の `[T3]` WARN 対象外にする。UI-13 / REQ-904 の在庫整合性とは別契約であり、専用 task の Design Phase と実装着手時に `required` へ戻す。
- SP-NNN / QR-NN / `UI-NNx-Dn`（設計決定 ID）は本表の対象外（traceability v1 の Non-scope）。
