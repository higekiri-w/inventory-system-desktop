# モデルベースの横断業務検証

> **親文書**: [現行構造の図面](current-system.md)
> **検証対象**: 入庫・手動販売・返品・廃棄・商品別CSVの取込み/取消・棚卸し。実店舗データを使わず、実BIZ関数と一時SQLiteを使用する。

## 図の使い分け

| 名称 | 整理すること | 今回の用途 |
|---|---|---|
| ER図（Entity Relationship Diagram） | データの構造・キー・関係 | [現行DB](../inventory_system_erd.html) |
| シーケンス図（Sequence Diagram） | 誰がいつ何を呼ぶか | 下記のカウント・販売・取込み・確定の前後関係 |
| ステートマシン図（State Machine Diagram） | 状態と許可する遷移 | 下記の棚卸し・未取込み売上の状態 |
| アクティビティ図（Activity Diagram） | 作業の分岐・合流・担当 | 下記の検証手順。Mermaid flowchartによる簡略表現 |
| データフロー図（Data Flow Diagram / DFD） | 入力・処理・保存先・出力 | [在庫と日報の流れ](current-system.md#業務データの流れ)。現行図は厳密なDFD記法ではなく業務フローの簡略表現 |
| C4コンテキスト図 / コンテナ図 | 利用者・外部システム・アプリ・保存先の境界 | POS/PCツール/アプリの責務を見直すときの候補。今回新しい図は増やさない |

一般的な名称を使って目的を明確にし、同じ情報の重複図は増やさない。[C4公式](https://c4model.com/diagrams) も必要な粒度だけを選ぶ方針を示している。

## 検証モデルの契約

これは製品の新しい動作仕様ではなく、既存契約と業務上の期待を検査するためのモデルである。矛盾を見つけても製品コードや採用済み仕様を自動的に変更しない。

### XFA-D1: 記録上の在庫・売上モデル

- 合成商品の初期在庫から開始し、操作の意味だけで期待在庫・売上数量・売上金額を更新する。DBの現在値・movement合計・BIZの計算関数から期待値を作らない。
- 手動販売は `source=manual` の売上を作る。商品別CSVは `source=auto` の売上を作り、どちらも入力した数量・金額を期待値に用いる。
- 入庫・廃棄・返品の帳面記録は `sale_records` を作らない。POS側の返品売上はCSVの負数行として別に扱う。
- 日次売上は `(product_code, source)` ごとの数量・金額と総計を比較する。総計だけ一致して記録元を取り違える不具合も検出対象とする。
- 入庫は加算、手動販売と廃棄は減算。レジ未処理返品は加算、レジ処理済みの帳面記録は在庫不変。商品別CSVは売上を作り、`pos_stock_sync=true` の商品だけ在庫へ反映する。
- CSV取消は対象importの寄与だけを取り除く。再取消と同じ冪等キーの再送は業務状態不変。同一キーの別内容や同一hashのactiveファイルは拒否され、業務状態は変わらない。
- 操作後に、商品ごとの在庫、非voidのmovement合計、日次売上の数量・金額を独立モデルと比較する。業務上の拒否では、ヘッダ/明細/売上/movementも増えないことを比較する。失敗を説明する操作ログの追加は許容する。
- 合成商品はJANを共有せず、同じ在庫単位にする。共有JAN、単位変換、価格/原価履歴の意味はこのモデルでは判定しない。

根拠: [在庫処理 §12](../function-design/31-biz-inventory-service.md)、[商品別CSV §15](../function-design/32-biz-csv-import-service.md)、[売上集計](../function-design/34-biz-sales-service.md)。

### XFA-D2: 棚卸しの物理的な数量モデル

- 外部の観測値として「合成シナリオ上、棚に何個あるか」を持つ。手動販売・入庫等の現物移動で変化し、CSVファイルの取込み自体では現物は変化しない。
- POS販売の発生とCSVの取込みを別の操作にする。未取込みの間にDB在庫と現物数が違うことは、それだけで不具合とはしない。
- カウントはその時点の現物数を観測する。カウント後の現物移動を消さず、取込みを完了した時点の在庫が現物と一致することを業務oracleとする。
- 棚卸し確定後にカウントより前の販売データを取り込む場合も、同じ現物減少を二重に計上しないことを検査する。取込み日と発生日の一般的な解決方法は未採用であり、このモデルは具体的な順序の反例を提示するだけとする。
- 再カウント、カウント後に変動がない場合、遅れたCSVを確定前に取り込んだ場合も対照として検証する。
- `start_stocktake` は全商品の明細を作るため、診断対象の商品をカウントし、対象外の未カウント商品は `force_fill=true` で現在庫を充当する。合成の対象外商品は非負在庫とし、負在庫の0補正を診断に混ぜない。確定処理が成功した後の在庫と現物モデルの不一致だけを時点問題と判定し、ValidationFailedやfixture不備を既知問題の再現に数えない。
- 時点診断・正常対照の判定は診断対象SKUの在庫数だけを比較する。対象外SKUのforce_fill充当結果、売上、台帳はこの時点判定の検査対象外。別の通常業務モデルで行う全商品・source別売上・台帳比較と区別する。

既存の [棚卸し設計](../function-design/35-biz-stocktake-service.md) と [STK-1](../research/2026-09-16-diagram-audit.md#stk-1-カウント後の入出庫を棚卸し確定が打ち消す) は、保存済みactual_countへ在庫を合わせる実装と業務期待の食い違いを示す。本書は特定の修正方式の採用を意味しない。

### XFA-D3: 生成・実行・再現

- 既存のRust test・一時DB・CSV fixture builderを再利用する。業務処理をmockせず、BIZ→DBまで実行する。
- 通常の業務操作について、有限の操作集合から順序付きの組合せを作り、その前後に同日追加取込み・取消・再送・拒否・再接続を組み合わせる。無限の状態空間を網羅したとは主張しない。
- 最初に期待と異なった操作までのprefix、期待値、実値を失敗出力へ残す。これは最初の失敗prefixであり、全ての操作削除候補から求めた最小反例とは呼ばない。
- 接続を閉じて同じ合成DBへ再接続する検証はDB永続化の確認であり、Tauriの再起動やCMDのpreview cache失効の検証とは区別する。
- 棚卸しと商品別CSVの書く操作の停止（[停止 ADR](../adr/2026-09-23-legacy-stocktake-z004-write-stop.md)、superseded）は D-118 の b1 で外した。harness は新しい経路（計数の保存・完了・Z004 の確定と取消）を呼ぶ。STK-1・STK-2 の診断 2 本は、新しい経路で PASS する通常の回帰 test（`test_cross_feature_req205_count_then_movement_is_kept`・`test_cross_feature_req205_req401_late_import_not_double_counted`）に移した（XFA-D4 の「修正が採用された際は通常の回帰テストへ移す」）。

### XFA-D4: 通常テストと診断の判定

- 採用済み契約を守る通常ケースと、モデルが整合した誤値も検出する対照は通常の `cargo test` に含める。
- 棚卸しの既知問題とその近傍を探索する新規診断テストは、`#[ignore = "XFA: explicit diagnostic; known stocktake temporal contract conflict"]` を付けた専用テストとし、診断コマンドで必ず明示実行する。既存テストをignoreに変更しない。
- 診断関数の `diagnostic_` 接頭辞は意図的にtraceability generatorの `test_` 抽出対象から外し、既知FAILの診断をREQ coverageの保護実績に計上しない。REQ番号は追跡用に名前・コメントへ残し、実行結果は本書に別記する。
- 診断の期待値を現在の誤動作へ合わせない。違反があれば実際に非0で終了し、実装の不合格として記録する。`should_panic` で正常化したり、panicを握りつぶしてpassにしない。
- 通常suiteのPASSと診断suiteのFAILを別々に報告する。既知問題の修正が採用された際は、該当反例を通常の回帰テストへ移す。

明示実行はRust標準の [ignored test実行](https://doc.rust-lang.org/book/ch11-02-running-tests.html#ignoring-tests-unless-specifically-requested) を使う。新しいテストframeworkや合格判定を緩めるgateは追加しない。

### XFA-D5: 証拠と限界

- 結果を図の矢印とモデルの契約へ対応付ける。新しい不具合、既存STK-1の別経路、仕様上の限界、fixture/モデル側の誤りを分ける。
- 一般的な店舗運用を全て再現したとは主張しない。対象は単一プロセスのBIZ/SQLite、合成商品、整数数量、明示した操作順序である。
- GUI・native dialog・実POS・バックアップ復元・PLU状態・共有JAN・日報bundle・長期運用の全組合せは今回の実行対象外。必要な後続を結果に残す。

## シーケンス図: 発生時点と反映時点

業務時点を示す概念シーケンス。取込み・確定の矢印ではUI/CMD/BIZの経由を省略し、現物・観測・DB反映の前後関係に絞る。

```mermaid
sequenceDiagram
  participant Shelf as 現物の数量モデル
  participant POS as POSの販売イベント
  participant Count as 棚卸し
  participant DB as アプリDB
  POS->>Shelf: 販売で2個減る
  Note over POS,DB: まだCSVを取り込んでいない
  Shelf->>Count: 残数をカウント
  Count->>DB: 棚卸しを確定
  POS->>DB: 先ほどの販売CSVを後から取り込む
  Note over Shelf,DB: 同じ販売を二度減算していないか比較
```

## ステートマシン図: 棚卸しと未取込み売上

```mermaid
stateDiagram-v2
  state 棚卸し {
    [*] --> 未開始
    未開始 --> 未カウント: 開始
    未カウント --> カウント済み: 現物を数える
    カウント済み --> カウント済み: 再カウント
    カウント済み --> 完了: 確定
  }
  state 売上データ {
    [*] --> 未取込みなし
    未取込みなし --> 未取込みあり: POSで販売
    未取込みあり --> 未取込みなし: CSV取込み
  }
```

両者は独立に状態を持つ。棚卸しが「完了」でも、過去の売上が「未取込みあり」の場合を検証から落とさない。

## アクティビティ図: 操作から検証へ

```mermaid
flowchart TD
  START([開始]) --> SEED[合成商品・初期在庫を用意]
  SEED --> ACTION[操作列から次の操作を選ぶ]
  ACTION --> MODEL[業務モデルの期待値を更新]
  MODEL --> BIZ[実BIZ関数を呼び出す]
  BIZ --> CHECK{期待値と実結果が一致するか}
  CHECK -->|はい・操作が残る| ACTION
  CHECK -->|いいえ| TRACE[最初の失敗prefixと値を記録]
  CHECK -->|はい・終了| PASS[この操作列はPASS]
  TRACE --> CLASSIFY[製品・仕様・モデル・fixtureの原因を判別]
  CLASSIFY --> END([結果を報告])
  PASS --> END
```

## 実行結果

通常suiteと明示診断を別々に実行した。通常の業務モデル・正常対照はPASS、棚卸しの時間軸の診断はFAIL（終了コード101）。診断の失敗は、製品の既知問題を修正したという意味にはしない。

```bash
cd src-tauri
cargo test --offline --lib cross_feature_tests -- --nocapture
cargo test --offline --lib cross_feature_tests -- --ignored --nocapture
```

テスト本体は [cross_feature_tests.rs](../../src-tauri/src/biz/csv_import_service/tests/cross_feature_tests.rs)。通常の操作集合は `OPERATIONS` を正本とする。各順序付きペアの後に入庫とDB再接続を固定で追加し、その入庫の再送・内容不一致拒否、同日売上/返品の追加、active hash再取込み拒否、対象importの取消/再取消、取消後再取込み、再取込み後の同hash拒否・旧IDの再取消、最後のDB再接続を続ける。各操作後に独立モデルと比較する。数量/金額/商品数を任意に生成する無限探索ではない。

探索件数・所要時間は上記コマンドの `XFA_NORMAL traces=...` とRust testの `finished in ...` で測定する。測定値と出力は [PR #69のValidation](https://github.com/kosei-w90607/inventory-system-desktop/pull/69) に公開時に反映し、テスト件数の正本を文書へ複製しない。所要時間はその環境での実測であり、性能保証ではない。

| 検証 | 結果 | 根拠 |
|---|---|---|
| 通常の在庫・売上・再送・取消 | PASS | 実BIZ、実parser、一時DB、日次売上consumerをモデルと比較 |
| カウント後に変動なし | PASS | 保存済み数と現物が一致 |
| 移動後に再カウントして確定 | PASS | 最新の観測に合わせた対照 |
| カウント前のPOS販売を確定前に取り込む | PASS | 現物・保存実数・反映済み在庫が同じ時点になる対照 |
| カウント後のPOS販売を確定後に取り込む | PASS | 未取込み中の差を不具合と誤認しない対照 |
| カウント後の入庫/販売/廃棄/返品/取込み→確定 | FAIL | 下記XFA-T1。確定は成功し、その後の数量比較で失敗 |
| POS販売→カウント→確定→遅いCSV取込み | FAIL | 下記XFA-T2。取込みも成功し、その後の数量比較で失敗 |

### XFA-T1: 保存した実数が後続の現物移動を打ち消す（D-118 の b1 の前の結果）

D-118 の b1 で計数の置き場（ADR D5）と求め直しに置き換え、下の不具合は通常の回帰 test で止める。以下は置き換える前の記録。

初期在庫とカウントを10とした合成シナリオ。いずれも処理は成功し、数量の比較で失敗した。全ケースで内部整合性チェックは不整合0だった。

| カウント後の操作 | 期待する現物数 | 確定後の在庫 |
|---|---:|---:|
| 入庫 +3 | 13 | 10 |
| 手動販売 -2 | 8 | 10 |
| 廃棄 -1 | 9 | 10 |
| レジ未処理返品 +1 | 11 | 10 |
| POS販売 -2 → CSV取込み | 8 | 10 |

[STK-1](../research/2026-09-16-diagram-audit.md#stk-1-カウント後の入出庫を棚卸し確定が打ち消す) を複数の入口から再現したもの。同じ失敗原因を別々の新規不具合として水増ししない。

### XFA-T2: 棚卸しに含まれた過去の販売を遅いCSVが再度減算する（D-118 の b1 の前の結果）

D-118 の b1 で精算の窓（ADR D3）に置き換え、下の不具合は通常の回帰 test で止める。以下は置き換える前の記録。

| 順序 | 出来事 | 現物モデル | アプリ在庫 |
|---|---|---:|---:|
| 1 | 初期状態・棚卸し開始 | 10 | 10 |
| 2 | POSで2個販売、CSVは未取込み | 8 | 10 |
| 3 | 現物を数えて8を保存 | 8 | 10 |
| 4 | 棚卸し確定 | 8 | 8 |
| 5 | その販売CSVを取り込む | 8 | **6** |

```text
prefix=[Start, PosSale, Count, Complete, ImportPending]
expected physical=8, actual stock=6, internal mismatches=0
```

棚卸し確定は8へ合わせる補正を記録し、その後のCSVは通常の販売減算を記録する。現物では一度だけ起きた販売が、記録上二度在庫を減らす。[棚卸しBIZ](../../src-tauri/src/biz/stocktake_service.rs) と [CSV commit](../../src-tauri/src/biz/csv_import_service/commit.rs) を横断すると成立する。監査では [STK-2](../research/2026-09-16-diagram-audit.md#stk-2-棚卸し確定後に届く過去販売を二重に減算する) としてSTK-1と別に追跡する。確定直前の再カウントだけではSTK-2を解消しないため、それぞれに是正の合格条件を持つ。解消するには、カウントの基準時点と、取込み済み/未取込みイベントの境界を業務として定義する必要がある。今回その方式は採用していない。

### 検査モデルの対照と残る範囲

- 在庫とmovementを同時に増やす合成mutationを与えた。内部整合性チェックが不整合0でも、独立モデルは不一致を検出した。
- 売上の総計が変わらないままmanualをautoへ変更するmutationも実行し、モデルが不一致を検出した。商品×sourceの比較を総計比較へ縮退させない。
- `build_cached` の重複は既存のtest_supportへ挙動不変で移設し、既存commit/rollbackテストの成功を確認した。既存のassertや実行条件は変更していない。
- 失敗した診断は通常gateのREQ coverageへ計上していない。修正採用後に通常の回帰テストへ移すまで、上記の明示コマンドで再検査できる。
- 未検証範囲はXFA-D5を維持する。棚卸しとCSV取消の時点関係、任意期間の遅い到着、共有JAN・単位変換・現物の計数誤りは、今回の結果だけでは判断できない。
