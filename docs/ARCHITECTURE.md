# 在庫管理システム アーキテクチャ設計書

## 計数の置き場と在庫の求め直しの契約（D-118。(a)・b1・b2 は実装済み）

[棚卸しの数の置き場と在庫の求め直しの ADR](adr/2026-10-16-stocktake-count-placement.md)（SPEC-STK-CNT-D1〜D12）の層ごとの責務。(a)（ADR SPEC-STK-CNT-D12）で入ったのは、BIZ の並びの鍵と求め直しの関数・整合性の突合、IO の並びの鍵付きの movement と写しの列の書込み、MNT の migration v9、UI の整合性検証（75）と操作ログ（74）の表示。b1（[Plan Packet](archive/plans/2026-10-22-stocktake-count-runtime-b1.md)）で、計数の保存・完了・Z004 の識別と確定と取消・要再確認の一覧・共有 JAN と在庫連動を始める検査・MNT の migration v11 が入り、停止の入口を外した。b2（[Plan Packet](plans/2026-10-23-stocktake-count-runtime-b2.md)）で、商品の結果の写しの 3 列と商品の画面の範囲の表示、在庫変動の履歴の計数の行（BIZ が movement と計数を並びの鍵の順に合成する）が入った。旧 ADR（時点証拠）の計数 context・受領・前後の分類・全数読取り・EJ の完全性の責務は置き換えた。

| 所有層 | 新契約の責務 / 詳細 |
|---|---|
| UI | 商品ごとの即保存（日付 D を常に出す、前の日の分の選択、日付の変わり目の確認）、要再確認の一覧と入れ直し、範囲の表示、完了の拒否の一覧と「0 にする」。[UI task](architecture/ui-task-specs.md)、51/55/58/65/66/73 |
| CMD | 生成 wire・error の変換、PC の今日を BIZ へ渡す。業務条件は置かない。[CMD task](architecture/cmd-task-specs.md)、40/41/42 |
| BIZ | 並びの鍵、求め直しの関数（写しを書く唯一の経路）、在庫変動の履歴の合成（鍵の順・絞り込み・ページ）、要再確認の判定、Z004 の識別・順・欠けの知らせ、完了の条件と評価額の非遡及。[BIZ task](architecture/biz-task-specs.md)、30/31/32/35/36 |
| IO | 精算の識別メタを返す parser、並びの鍵付きの movement と計数の保存、写しの列の書込みと読取り、在庫変動の履歴の並べない読取り。[IO task](architecture/io-task-specs.md)、20/21/23/24 |
| MNT | 列と表を足す migration（(a) の v9、b1 の v11。棚卸しか Z004 の取込みを持つ DB では止まる。開発・試験 DB はリリースの前まで作り直してよい。D-123）。[MNT task](architecture/mnt-task-specs.md) |

レジ固有の形式は adapter、精算の並びと要再確認の判定は BIZ が所有する。時刻で前後を判定しない。EJ は在庫に使わない。

> **現行構造の図**: [層・画面遷移・業務データフロー](diagrams/current-system.md)、[ER図](inventory_system_erd.html)。実装の照合結果と設計上の懸念は [図面監査](research/2026-09-16-diagram-audit.md) を参照。
> **入力ドキュメント**: `docs/spec/requirements.md`（REQ inventory）、`docs/spec/requirements-coverage.md`（公開要求 coverage）、DB_DESIGN.md（テーブル定義書）、SCREEN_DESIGN.md（画面設計書）、screen_mockups.html（モックアップ）

---

## 1. 設計方針

### 本書の位置付け
「要求を仕様化する技術」（清水吉男）のUSDM手法に基づき、要求仕様書（What: システムの外部振る舞い）と実装（コード）の間に置く「タスク設計」レイヤー。システム内部の大きな処理単位（タスク）の分割と、各タスクの振る舞いを定義する。

### 階層構造
```
要求（ヒアリングシート）
  ↓
要求仕様（公開複合正本）← `docs/spec/requirements.md` + `docs/spec/requirements-coverage.md` + linked design docs
  ↓
★ タスク設計（本書）← ARCHITECTURE.md
  ↓
関数設計（詳細設計）← FUNCTION_DESIGN.md / function-design + 実装の型定義・テストを照合
  ↓
実装（Tauri + React + SQLite）
```

### レイヤー分割方針
本書のアーキテクチャ分析テンプレートに基づき、5層に分割する。

| レイヤー | 技術 | 責務 |
|---------|------|------|
| UI層 | React + TypeScript | 表示/UI処理。画面描画、利用者操作の受付、入力バリデーション |
| CMD層 | Tauri Commands（Rust） | フロントエンド↔バックエンドの接点。入口として薄く保つ。業務ルールは持たない |
| BIZ層 | Rust | メイン処理。ビジネスロジック、業務ルールの実行、トランザクション制御 |
| IO層 | Rust + SQLite | 入出力処理。DB操作、ファイルI/O、外部フォーマット変換 |
| MNT層 | Rust | 保守/診断処理。バックアップ、操作ログ、定期メンテナンス |

### CMD層の責務ルール
CMD層は「画面からバックエンドへ渡す入口」であり、以下のみを行う:
- リクエストの型変換（フロントエンドのJSON → Rustの構造体）。有限文字列から内部 enum への変換失敗はこの境界で拒否してよいが、変換後の値に対する業務条件の判定はBIZ層が所有する
- BIZ層の呼び出し
- レスポンスの型変換（Rustの構造体 → フロントエンドのJSON）
- **業務ルール、業務バリデーション、DB操作は一切行わない**

同じ業務条件をCMD層とBIZ層の双方で判定してはならない。CMD層で返す
`CmdError` の `kind` / `message` / `field` は、BIZ層のエラーを変換して保持する
（internal / restore_*系の `error_id` のみCMD境界で付与する。40-cmd-product.md 5.3 CMD-ERR-D1）。
この単一所有ルールを **ARCH-VAL-D1** とする。

wire型変換の失敗と、source designが明示するresource-safetyの早期拒否
（例: `41-cmd-pos.md` のfile size上限）は業務validationではない。これらはCMD境界で
拒否してよいが、BIZの業務contractを置き換えたり、異なる条件・文言を定義してはならない。

### レイヤー間の呼び出し原則
- UI → CMD → BIZ → IO の一方向のみ。上位層が下位層を呼ぶ
- **UIがIOを直接呼ぶことは禁止**。一括インポートもUI→CMD→BIZ-01→IO-03の順で呼ぶ
- **CMD が IO 層を直接呼ぶことは禁止**（BIZ 経由のみ。`src-tauri/tests/architecture_test.rs` の layer 依存 test が機械検査する。D-060）
- **CMD → MNT の直接呼び出しは、DB 接続所有権の交換を要する保守 orchestration（MNT-01 バックアップ/復元系）に限り正規経路とする**。`AppState.db` Mutex は CMD 層の所有物であり、接続の取り出し・差し替えは CMD にしか実行できないことが根拠。復旧規則・分類の正本は `function-design/71-mnt-backup.md` §71.7 が所有し、CMD は同 § の呼び出しパターンを実行するのみで独自の規則を定義しない（D-060）
- BIZ層同士の呼び出しは許可（例: BIZ-01の商品登録がBIZ-06の棚卸しアイテム追加を呼ぶ）
- IO層同士の呼び出しは許可（例: IO-01のリポジトリをIO-02のパーサーが使うことはないが、MNT-03がIO-01を呼ぶ）

### POS Adapter Boundary（レジ換装境界）

POS連携は、レジ固有の adapter とアプリ内共通モデルの境界を明確に分ける。これは `D-023` の設計判断であり、将来レジが変わったときに商品マスタ、在庫管理、売上レポート、取込み履歴を作り直さず、レジ依存の入出力だけを差し替えるための境界である。

#### レジ adapter が所有するもの

- 外部ファイル形式: 文字コード、改行、列名、ヘッダ/メタ行、ファイル名、帳票番号、PLU登録ファイル形式。
- レジ/PCツール手順: SDカード、PCツール、バックアップ、取込み/書込み、失敗時の再実行手順。
- レジ制約: PLU上限、部門上限、商品名バイト数、税区分ラベル、スキャニングコード規則。
- 外部データからアプリ内モデルへの変換: 日報サマリ、決済/取引キー集計、部門別売上、商品別売上、PLU書出し結果。
- 実機確認: 受理可否、出力形状、匿名化した証跡、操作手順差分。

#### アプリ core が所有するもの

- 商品マスタ、部門、取引先、価格履歴、`plu_dirty` / `plu_exported_at`。
- 在庫変動、業務記録、売上/日報/レポートの内部モデル。
- 取込み lifecycle: Parse -> Validate -> Preview -> Commit、重複判定、rollback、operation_logs。
- operator-facing な業務概念、画面文言、エラー/リカバリ、データ安全境界。

#### CASIO SR-S4000 adapter の現在の分割

| Adapter area | CASIO-specific source | App-internal target |
|---|---|---|
| PLU export | CV17 1.1.1 PLUファイル `.txt` / PCツール / SR-S4000 | 商品マスタから生成したPLU登録候補と書出し結果 |
| Daily report import | `Z001` / `Z002` / `Z005` | 日報サマリ、決済/取引キー集計、部門別売上 |
| Product-sales import | `Z004` after PLU registration | 商品別売上、`pos_stock_sync`に基づく在庫増減・rollback（実装済み。店舗採取layout Aは二形状対応済み） |
| Optional report tabs | `Z006` / `Z009` / `Z011` | 初期非スコープ。グループ、時間帯別、担当者別の具体的な個人店ニーズが出た時だけ追加 |
| Operation procedure | SDカード、PCツール、バックアップ、実機確認 | operator workflow、証跡、安全手順 |
| Register SD source | SD の `CASIO\SR500_550_4000\XZ` / `XZ_BKUP` の配置と名前（IO-09） | 取込みの候補の束（BIZ-08 §37.9）。毎日の売上はアプリが SD を直接読む（D-111）。CV17 を PLU の書込みだけに使うのは取込み済みを `XZ_BKUP` へ移すアプリの操作（後続の lane）の後で、それまでは店の CV17 の取込みも続く（D-111 の運用の制約） |

`Z001` / `Z002` / `Z004` / `Z005` / `Z006` / `Z009` / `Z011`、CV17、SR-S4000 は CASIO adapter details として扱う。BIZ/CMD/UI の安定 contract へ直接漏らす場合は、該当 source design doc に理由、代替案、将来レジ変更時の影響を記録する。

#### 事実確認と設計判断の分離

- 事実確認: 実機/PCツールが受理・出力する形式、文字コード、改行、列、件数、バージョン、運用手順、匿名化証跡。
- 設計判断: アプリ内モデル、DB保存先、在庫引落し条件、rollback semantics、画面導線、レジ変更時に維持する共通 contract。

フィールド調査結果は adapter facts として記録し、アプリ core の contract へ昇格する場合は Design Phase で判断する。

### IO-01（SQLiteデータアクセス層）の実装分割方針
設計上は1タスクだが、実装段階では以下のリポジトリに分割する:
- product_repository（商品マスタ、部門、取引先、価格履歴）
- inventory_repository（在庫変動、入庫、返品、廃棄、手動販売）
- sales_repository（売上レコード、Z004 CSV取込み、CSV取込みエラー、日報取込み、日報サマリ/支払/部門別行）
- stocktake_repository（棚卸しヘッダ、棚卸し明細）
- system_repository（操作ログ、設定）

---

## 2. タスク一覧

### UI層（React + TypeScript）

| ID | タスク名 | 対応REQ | 対応画面 | 責務 |
|----|---------|---------|---------|------|
| UI-00 | ホーム画面 | REQ-301/302, SP-102-07 | ホーム | ダッシュボード式の入口。サマリカード（昨日の売上、在庫切れ件数、在庫少件数）、PLU未反映通知バー、毎日機能への大ボタン |
| UI-01a | 商品検索・一覧 | REQ-103 | 商品マスタ一覧 | 商品の検索・絞込み・並替え・廃番視覚区別。D-138 で UI-06a と 1 つの画面「商品と在庫」（`/products`）にする（proposed・未実装） |
| UI-01b | 商品登録・編集 | REQ-101, 102 | 商品登録、商品編集 | 商品情報の入力フォーム、JANスキャン、独自コード発番、売価変更、廃番切替 |
| UI-01c | 商品一括インポート | REQ-104 | 一括インポート | CSVアップロード、プレビュー表示、エラー表示、上書き/スキップ選択 |
| UI-02 | 入庫記録画面 | REQ-201 | 入庫記録 | 複数商品の一括入庫入力、取引先選択、数量入力 |
| UI-03 | 返品・交換記録画面 | REQ-202 | 返品・交換 | レジ戻し済みフラグ、戻り/渡し明細、レシート画像添付 |
| UI-04 | 手動販売出庫画面 | REQ-203 | 手動販売出庫 | PLU未登録商品の販売記録、PLU登録済み警告表示 |
| UI-05 | 廃棄・破損記録画面 | REQ-204 | 廃棄・破損 | 種別選択、理由入力、ロス原価記録 |
| UI-02b / UI-03b / UI-04b / UI-05b / UI-07b / UI-10b | 入出庫履歴・記録詳細 | REQ-206, 207 | 入出庫履歴、各業務記録詳細 | 横断検索、入庫・返品交換・手動販売・廃棄破損・CSV取込み・棚卸しの追跡。取消・訂正は後続 |
| UI-06a | 商品別在庫照会 | REQ-301 | 在庫照会 | 商品検索→在庫数・売価・原価・最終入庫日・最終販売日。D-138 で UI-01a と 1 つの画面「商品と在庫」にする（proposed・未実装） |
| UI-06b | 在庫少一覧 | REQ-302 | 在庫照会（deep-link、独立画面なし） | UI-06a `/stock` の `status=low_stock` フィルタへのサイドバー deep-link（D-047）。閾値以下の商品一覧、廃番除外は UI-06a の既存 `list_low_stock` contract を再利用。D-138 の後は「在庫切れ・在庫少」として `/products?status=low_stock` を開く（proposed・未実装） |
| UI-06c | 在庫変動履歴 | REQ-303 | 変動履歴 | 商品別の全変動を時系列表示、種別・期間絞込み |
| UI-07 | 売上データ取込み画面 | REQ-401 | 売上データ取込み | current operation ではZ001/Z002/Z005日報取込みを主動線にする。既存Z004商品別取込みはPLU確認後の別トラックとして扱う |
| UI-08 | PLU書出し画面 | REQ-402 | レジ登録データ作成 | 全件/差分選択、書出し実行、plu_dirty商品一覧 |
| UI-09a | 日次売上一覧 | REQ-501 | 日次売上 | 日付指定、商品別一覧、部門小計、記録元区別 |
| UI-09b | 月次売上集計 | REQ-502 | 月次売上 | 月指定、商品別/部門別、ランキング、前月比較 |
| UI-10 | 棚卸し画面 | REQ-205 | 棚卸し | 開始/中断再開、部門絞込み、カウント入力、差異表示、確定 |
| UI-11a | 閾値設定画面 | QR系 | 設定（閾値） | 在庫少閾値・生地用閾値等の運用パラメータ変更 |
| UI-11b | バックアップ画面 | QR-05 | 設定（バックアップ） | 手動バックアップ実行、自動バックアップ設定、リストア |
| UI-11c | 操作ログ画面 | QR-06 | 設定（ログ） | 操作ログの閲覧・期間/種別フィルタ・ページング |
| UI-12 | 共通レイアウト | - | 全画面共通 | サイドバー＋メイン2カラム構成、ナビゲーション |
| UI-13 | 在庫整合性検証画面 | REQ-904 | 在庫整合性検証 | stock_quantity整合性チェック実行、差異商品一覧表示、在庫数補正の確定確認（movement 追加なしの直接更新、BIZ-07-D2） |
| UI-14 | 一括価格改定 | REQ-105, 106 | 一括価格改定 | 取引先・部門・商品検索、行ごとの売価・原価改定。取引先紐付けは明示選択 |
| UI-15 | 取引先管理 | REQ-106, 107 | 取引先管理 | 取引先の追加・改名・重複統合 |

REQ-403（POS 部門別売上照合）は UI-13 / REQ-904 の在庫整合性とは別契約であり、画面 task 未割当の deferred 要求とする。部門集計の数量・金額差と原因調査材料を示すが、自動修正は行わない。

### CMD層（Tauriコマンド）

| ID | タスク名 | 呼び出すBIZ | 責務（薄いラッパー） |
|----|---------|-----------|------|
| CMD-01 | 商品コマンド群 | BIZ-01 | 商品CRUD、検索、一括インポートの入口 |
| CMD-02 | 入庫コマンド群 | BIZ-02 | 入庫記録の作成・一覧取得の入口 |
| CMD-03 | 返品・交換コマンド群 | BIZ-02 | 返品・交換記録の作成・一覧取得の入口 |
| CMD-04 | 手動販売出庫コマンド群 | BIZ-02 | 手動販売出庫の作成の入口 |
| CMD-05 | 廃棄・破損コマンド群 | BIZ-02 | 廃棄・破損記録の作成・一覧取得の入口 |
| CMD-06 | 在庫照会コマンド群 | BIZ-01, BIZ-02 | 在庫照会・変動履歴の取得の入口 |
| CMD-07 | Z004商品別CSV取込みコマンド群 | BIZ-03 | parse/validate/commit/rollbackの入口。商品別売上と`pos_stock_sync`在庫増減は実装済みだが、現行 build では業務 commit・取消を停止している（[停止 ADR](adr/2026-09-23-legacy-stocktake-z004-write-stop.md)。新しい取込みの形は D-118 の (b)）。店舗採取layout AはIO-02が二形状対応済み |
| CMD-08 | PLU書出しコマンド群 | BIZ-04 | PLUファイル生成の入口 |
| CMD-09 | 売上集計コマンド群 | BIZ-05 | 日次・月次集計データ取得の入口 |
| CMD-10 | 棚卸しコマンド群 | BIZ-06 | 棚卸し操作の入口 |
| CMD-11 | 設定・ログコマンド群 | BIZ-09, BIZ-02, MNT-01 | 設定CRUD、ログ取得、バックアップ/復元、領収書画像保存の入口（D-060） |
| CMD-12 | 日報取込みコマンド群 | BIZ-08 | Z001/Z002/Z005 bundle preview/commit/rollback/listの入口 |

### BIZ層（ビジネスロジック）

| ID | タスク名 | 対応REQ | 責務 |
|----|---------|---------|------|
| BIZ-01 | 商品管理ロジック | REQ-101〜104 | 商品登録・修正・廃番・独自コード発番・売価変更（price_history記録）・一括インポート（エンコーディング判定＋プレビュー）・PLU dirty管理 |
| BIZ-02 | 在庫変動ロジック | REQ-201〜204, 303 | 入庫・出庫・返品・廃棄の共通在庫処理。inventory_movements記録と、記録からの写しの求め直し（`recompute_stock`）を同じ TX で実行。返品のregister_processedフラグ分岐。手動販売出庫のPLU登録済み警告判定 |
| BIZ-03 | Z004商品別CSV取込みパイプライン | REQ-401 | 4段階処理（Parse→Validate→Preview→Commit）。商品別売上、在庫引落し、重複チェック、ロールバック、csv_import_errors記録。現行 build では業務 commit・取消を停止している（停止 ADR。新しい形は D-118 の (b)） |
| BIZ-04 | PLU書出しロジック | REQ-402 | 商品マスタ→PLU登録候補生成。plu_target=1商品の抽出（三分バケット: 対象/対象外/要修正）、全件/差分モード、同一JAN dedup、adapter提供のレジ制約警告、PLUファイル生成とapp-side書出し済み確認を分離したplu_dirty/plu_exported_at更新制御（D-028） |
| BIZ-05 | 売上集計ロジック | REQ-501, 502 | 日次・月次の売上集計。商品別売上は sale_records、公式日報集計は daily_report_* を参照し、両者の意味を分けて返す |
| BIZ-06 | 棚卸しロジック | REQ-205 | 棚卸し開始（全商品のstocktake_items生成）・カウント入力・確定（total_cost計算、valuation_cost_price_centi記録、差異補正のinventory_movements記録）。現行 build では開始・カウント入力・確定を停止している（停止 ADR。確定の新しい形は D-118 の (b)）。進行中は1件まで制約。新規商品登録時の自動追加 |
| BIZ-07 | 整合性チェックロジック | QR系 | 写しの 4 項目（`stock_quantity`・要再確認の印・範囲）と記録から求め直した値（BIZ-02 の `derive_stock`）の突合。補正は `recompute_stock` で写しを書く（D-118 の (a)）。差異発見時の利用者確認フロー |
| BIZ-08 | 日報取込みロジック | REQ-401 | Z001/Z002/Z005 bundleのParse→Validate→Preview→Commit、日報テーブル保存、論理rollback。sale_records/inventory_movementsへ擬似展開しない |
| BIZ-09 | システム設定・操作ログロジック | REQ-902, 905 | 設定一覧取得・設定upsert・操作ログ検索（日付validation所有）・operation_type一覧。system_repoを呼ぶBIZ境界（D-060。仕様詳細は38-biz-system-service.md） |

### IO層（ファイルI/O・DB）

| ID | タスク名 | 責務 |
|----|---------|------|
| IO-01 | SQLiteデータアクセス層 | 全テーブルのCRUD操作。PRAGMA foreign_keys=ON、WAL モード設定、BUSY_TIMEOUT=5000ms。実装時はリポジトリパターンで分割（product/inventory/sales/stocktake/system） |
| IO-02 | Z004パーサー | CP932デコード、改行正規化（\u0085/\r\n/\n/\r）、メタ行日付抽出、5フィールドCSVパース、JAN正規化（末尾E除去→13桁化）。純粋なファイルフォーマット処理、業務ロジックなし |
| IO-03 | 商品マスタCSVインポーター | エンコーディング自動判定（BOM→UTF-8/それ以外→CP932）、ヘッダ検証、プレビューデータ生成。業務ロジック（重複チェック等）はBIZ-01側 |
| IO-04 | PLUフォーマッター | CASIO SR-S4000 / CV17 1.1.1 adapter のPLUファイル生成。2026-07-03 field gate により `.txt`、11列、13桁JAN必須、通常PLU216枠使用時217始まり、PCツールSD書込みからSR-S4000設定読みまでの反映手順を確認済み |
| IO-05 | レポートCSVエクスポーター | 売上一覧等のCSV出力。UTF-8 BOM付き（Excel対応） |
| IO-06 | 画像ファイル管理 | レシート画像の保存（アプリ管理下の相対パス）、パス管理 |
| IO-07 | POS日報bundleパーサー | CASIO SR-S4000 adapter のZ001/Z002/Z005パース。CP932/NEL対応、3ファイル束のsource判定、日報サマリ/支払/部門別行への正規化。業務ロジックなし |
| IO-08 | EJパーサー | CASIO SR-S4000 adapter の電子ジャーナル（EJ）1 file を記録の列へ構造復元。24バイト固定幅・CP932・CRLF、本文による記録の種類の判定と位置による行分類、記録内照合がそろう取引だけ明細と値引きを復元し、それ以外は記録単位で復元不能として返す。番号の連続・区間の完全性・商品同定・時刻は判断しない。業務ロジックなし |
| IO-09 | レジの SD の列挙と読取り | レジの SD の root を探し（取外し可能な drive の `CASIO\SR500_550_4000`）、`XZ` / `XZ_BKUP` の Z・EJ の名前を列挙し、指定 file を読取り専用で読んで形を測る。SD に書かない。業務ロジックなし（D-111） |
| IO-10 | SD から読んだ原本の写しの保存 | 取り込む束の file を、アプリのデータ folder の `pos-sources/casio-sr-s4000/sd/` へ SD の相対 path を保った名前で、上書きせずに書く（一時 file から rename）。SD には書かない。業務ロジックなし（D-111） |

### MNT層（保守/診断）

| ID | タスク名 | 責務 |
|----|---------|------|
| MNT-01 | バックアップ・リストア | SQLiteファイルのコピーバックアップ。自動（設定時刻）/手動。保持日数管理。リストア（バックアップファイルからDB復元）。PC の外の控え（USB メモリへ自動で写して照合）と復元の前の控えの確かめ（D-114） |
| MNT-02 | 操作ログ管理 | operation_logsへの記録（operation_type命名規約に従う）。閲覧画面用のページング取得。保持日数（既定 3000 日、帳簿書類の 7 年に足りる長さ。D-136）を超えたものの自動削除（アプリ起動時1日1回） |
| MNT-03 | スキーママイグレーション | schema_versionテーブルでバージョン管理。アプリ起動時にスキーマバージョン確認→必要ならALTER TABLE実行 |
| MNT-04 | アプリケーション診断ログ | tracing基盤による全層横断のファイルログ。エラー箇所・入力データ・モジュール名を記録。日次ローテーション、30日保持。REQ-700対応 |

---

## 3. タスク間の依存関係

### 実装順序（依存関係に基づく）

以下は初期の段階的な開発順序であり、現在の実行時呼出し図や未了作業の一覧ではない。段階の行の tag（`v0.7.0-ui-foundation`〜`v1.0.0`）は計画の時点の名で、作った tag は履歴境界の前の private archive の記録（public repo に tag は無い。[引き継ぎ案内](PROJECT_HANDOFF.md) の履歴境界の節）。v1.0 の release は自動在庫連動の完成まで待つ（D-070）。実行時の関係は [現行構造図](diagrams/current-system.md)、作業の現在地は helper の status と git log、導入の前の関門は [Plans](Plans.md) を参照する（D-145）。

```
第1段階: 基盤
  IO-01（DB層）→ MNT-03（マイグレーション）→ UI-12（共通レイアウト）

第2段階: 商品マスタ
  BIZ-01 → CMD-01 → UI-01a, UI-01b
  （商品の登録・検索・編集ができる状態）

第3段階: 入出庫＋在庫照会
  BIZ-02 → CMD-02〜05 → UI-02〜05
  CMD-06 → UI-06a, UI-06b（UI-06a 内 deep-link、D-047）, UI-06c
  （在庫が動く状態。手動運用で使い始められる）

第4段階: POS連携
  IO-02 → BIZ-03 → CMD-07 → UI-07（Z004商品別CSV取込み）
  IO-07 → BIZ-08 → CMD-12 → UI-07（日報取込み）
  IO-09 → BIZ-08 → CMD-12 → UI-07（SD から読む日報の候補、D-111）
  IO-10 ← BIZ-08 commit（取り込む束の原本の写し、D-111）
  IO-04 → BIZ-04 → CMD-08 → UI-08（PLU書出し）
  （レジ連携が動く。日次運用が回る状態）

第5段階: レポート＋棚卸し＋一括インポート
  BIZ-05 → CMD-09 → UI-09a, UI-09b（売上レポート）
  BIZ-06 → CMD-10 → UI-10（棚卸し）
  IO-03 → BIZ-01 → CMD-01 → UI-01c（一括インポート）
  BIZ-07（整合性チェック）

第5.5段階: 診断ログ基盤（第6段階の前提）
  MNT-04（アプリケーション診断ログ）
  （全層横断のtracing基盤。第6段階以降の実装で最初から活用する）

第6段階: 保守＋仕上げ
  MNT-01（バックアップ）
  MNT-02（操作ログ管理。アプリ起動時cleanupのみ、CMD-11とは非連結）
  CMD-11 → UI-11a/UI-11b/UI-11c（設定・バックアップ・ログ）
  IO-05（CSVエクスポート）
  IO-06（画像管理）

第7段階: UI基盤構築（Phase 1）
  技術スタック導入（Tailwind CSS 4 + shadcn/ui + TanStack + Zustand + feature-local controlled state + Zod）
  UI-12（共通レイアウト）→ ルーティング → invokeラッパ + TanStack Query基盤
  横断UI要素テンプレート（Toast/Dialog/空状態/エラー状態）
  （v0.7.0-ui-foundation タグ）

第8段階: 毎日使う5画面（Phase 2、モックアップ完成済み）
  UI-00（ホーム）+ UI-07（CSV取込み）+ UI-09a（日次売上）
  UI-06a（在庫照会）+ UI-09b（月次売上）
  ▼ 利用者デモ（必須 gate）— 5画面の操作フロー合意
  （v0.8.0-ui-daily タグ）

第9段階: 商品管理＋入出庫画面（Phase 3）
  UI-01a/b/c（商品管理）
  UI-02〜05（入出庫系4画面）
  （v0.9.0-ui-product-inventory タグ）

第10段階: 在庫特殊＋システム管理（Phase 4）
  UI-06c（変動履歴）+ UI-08（PLU書出し）※UI-06b 在庫少一覧は D-047 により UI-06a deep-link で完了、独立画面なし
  UI-10（棚卸し、中断再開UIが特殊）
  UI-11a/11b/11c（設定・バックアップ・ログ）
  UI-13（整合性検証）
  （v1.0.0 タグ — 全画面実装完了）
```

### 第2段階終了時点で利用者に見せられるもの
- 商品マスタの登録・検索・編集ができる
- 利用者が「使い勝手」を早期にフィードバックできる

### 第3段階終了時点で業務運用を開始できるもの
- 手動での入庫・在庫照会が可能
- CSV取込みなしでも在庫管理の基本操作ができる

### 第4段階終了時点で日次運用が回る
- レジ精算→adapter経由の売上/日報取込み→アプリ内レポート/在庫反映フローが動く
- PLU書出しで adapter 経由のレジ登録ファイルを生成できる

---

## 4. 未確定事項

- **Q40（障害時の対応）**: 画面ごとの CmdError と retry（UI-13 ほか）と共通の Error Boundary は実装済み。包括的な障害時対応の方針の残りは [backlog](backlog.md) の Q40 の項目が追う

---

## 5-9. タスク仕様（詳細）

各層のタスク仕様は以下のサブドキュメントに記述:

| レイヤー | ファイル | タスク |
|---------|---------|--------|
| BIZ層 | [architecture/biz-task-specs.md](architecture/biz-task-specs.md) | BIZ-01〜BIZ-09 |
| IO層 | [architecture/io-task-specs.md](architecture/io-task-specs.md) | IO-01〜IO-10 |
| MNT層 | [architecture/mnt-task-specs.md](architecture/mnt-task-specs.md) | MNT-01〜MNT-04 |
| CMD層 | [architecture/cmd-task-specs.md](architecture/cmd-task-specs.md) | CMD-01〜CMD-12 |
| UI層 | [architecture/ui-task-specs.md](architecture/ui-task-specs.md) | UI-00、UI-01a〜UI-14 |

---

## 10. 設計判断ログ

| 日付 | 判断 | 理由 |
|------|------|------|
| 2026-03-31 | 5層分割（UI/CMD/BIZ/IO/MNT）を採用 | 本書のアーキテクチャ分析テンプレート（表示/メイン/保守/入出力）に基づく。CMDを独立層にしたのはTauriのコマンド境界がフロントエンド↔バックエンドの明確な接点になるため |
| 2026-03-31 | CMD層は薄いラッパーに限定 | 業務ルールをCMD層に書くとフロントエンドとの結合が強くなる。テスト容易性のためにBIZ層に寄せる |
| 2026-03-31 | IO-01は設計上1タスク、実装時はリポジトリ分割 | 設計文書の粒度としては1タスクが適切。実装時にproduct/inventory/sales/stocktake/systemに分割する方針を明記 |
| 2026-03-31 | 実装順序は6段階 | 依存関係に基づく段階的デリバリー。第2段階で利用者フィードバック取得、第4段階で日次運用開始 |
| 2026-03-31 | タスク仕様を3視点（データ/処理/制御）で記述 | 「要求を仕様化する技術」の推奨に従い、実装者が迷わないレベルまでタスク内部を言語化 |
| 2026-03-31 | UI層はCMD呼び出しパターン＋状態管理に限定 | 画面レイアウトはSCREEN_DESIGN.md＋モックアップに記述済み。重複を避ける |
| 2026-03-31 | UIからIOの直接呼び出し禁止を明文化 | レイヤー原則の遵守。一括インポートもUI→CMD→BIZ→IOの順 |
| 2026-03-31 | pos_stock_syncは明示フラグ扱い | stock_unit='cm'だけで自動決定すると将来の柔軟性が下がる。UIで初期値提案＋変更可能にする |
| 2026-04-13 | 診断ログ（MNT-04）をoperation_logs（MNT-02）と独立して導入 | operation_logsは業務監査ログ（利用者向け、SQLite）。診断ログは障害追跡（開発者向け、ファイル）。目的・読者・保持期間が異なるため別系統で管理 |
| 2026-04-13 | 診断ログを第5.5段階として第6段階の前に導入 | 全層横断のインフラであり、第6段階のMNT-01等で最初から活用するため先に基盤を整える |
| 2026-04-13 | ERRORログはCMD層で1回のみ記録する方針 | エラーはIO→BIZ→CMDと伝搬するため、各層でERRORを出すと同じエラーが3回記録される。CMD層のエラー境界で1回だけ記録し、IO層の詳細はDEBUGレベルで追跡可能にする |
| 2026-04-13 | ログ初期化をアプリ起動シーケンスの最初に配置 | DB初期化やマイグレーションの失敗もログに残すため。ログ初期化自体の失敗はstderrフォールバックで対応 |
| 2026-04-13 | 全初期化処理をTauri setup hook内に移動 | Tauri公式推奨: "Don't write code before Tauri starts, write it in the setup hook instead!"。setup hookはウィンドウ作成前に実行される。app.path().app_data_dir()でパス解決でき、dirsクレートの追加が不要。DB初期化の相対パス("inventory.db")もapp_data_dir配下の絶対パスに修正 |
| 2026-04-16 | UI技術スタック選定（React 19 + Tauri 2 + shadcn/ui + TanStack + Tailwind stone系）+ ウォーム系パレット採用 | refactoring-ui §4 の「Pure grays look lifeless」原則と手芸店業務の4根拠（環境一致・利用者距離感・目の疲労軽減・差別化）。デザイン哲学は核心4本柱（refactoring-ui / ux-principles / GOV.UK / IBM Carbon）+ 補助3原則（Polaris / Atlassian / Fluent 2）+ japanese-webdesign 観点借用で構成。詳細は [UI_TECH_STACK.md](UI_TECH_STACK.md) を参照 |
| 2026-08-30 | CMD-11 の MNT-02 依存記述を実査是正（docs 整合性衛生 batch、本 PR） | `docs/architecture/cmd-task-specs.md` CMD-11 mapping と `rg -n "mnt" src-tauri/src/cmd/` 実査により、CMD-11 配下いずれの command も MNT-02（操作ログ管理）を直接・間接（BIZ-09 経由含む）に消費していないと確認。L153 依存列から MNT-02 を削除し、L228 flow 行を実態（MNT-02 は起動時 cleanup のみで CMD-11 と非連結）へ是正 |
