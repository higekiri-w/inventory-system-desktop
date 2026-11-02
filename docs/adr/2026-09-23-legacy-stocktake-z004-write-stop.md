# 旧棚卸しとZ004業務commit・取消を一時停止する

## Status

superseded — 2026-10-23、D-118 の runtime の b1（[Plan Packet](../archive/plans/2026-10-22-stocktake-count-runtime-b1.md)）で 5 つの入口を一度に外し（SPEC-STOP-D6）、旧本体と停止の文言・定数・画面の案内を撤去した。置き換えた先は [棚卸しの数の置き場と在庫の求め直しの ADR](2026-10-16-stocktake-count-placement.md)（SPEC-STK-CNT-D12 の b1）。以下は accepted の時点の記録。

accepted（当時） — 2026-09-24、㉘ runtime ①（packet `docs/archive/plans/2026-09-23-legacy-stocktake-z004-write-stop.md`）。owner 決定 2026-09-22（㉘ runtime の最初の lane = 既存の危険な操作の停止と再現 fixture）。

対象は REQ-205 / REQ-401。停止は現行 build に固定し、⑤（一括切替）で本 ADR を superseded にして解除する。

改訂（2026-10-16、[decision-log](../decision-log.md) D-118）: 時点証拠 ADR は [棚卸しの数の置き場と在庫の求め直しの ADR](2026-10-16-stocktake-count-placement.md) に置き換わった。本 ADR の「⑤」は、置き換えた ADR の runtime が、棚卸しの計数・完了と Z004 の確定・取消を新しい経路で置き換えるのと同じ配布の単位を指す（同 ADR の SPEC-STK-CNT-D12 の (b)。停止に触れない (a) の単位は先に入れてよく、停止を外さない）。停止の範囲・停止 error・解除を部分的に行わないこと（SPEC-STOP-D6）は変えない。

## Context

現行の棚卸し確定は、保存済みの実カウントへ在庫を上書きする。カウント後の入出庫があると、その増減を打ち消す（[STK-1](../research/2026-09-16-diagram-audit.md#stk-1-カウント後の入出庫を棚卸し確定が打ち消す)）。Z004 の業務 commit は、棚卸しに含まれた過去の販売を後から再び減算する（[STK-2](../research/2026-09-16-diagram-audit.md#stk-2-棚卸し確定後に届く過去販売を二重に減算する)）。取消は、その二重減算を戻す経路の一部である。どちらも合成 DB で再現済みである（[XFA-T1 / XFA-T2](../diagrams/cross-feature-verification.md#実行結果)）。

正しい計数・完了・取込み・取消は[棚卸しの数の置き場と在庫の求め直しの ADR](2026-10-16-stocktake-count-placement.md)（2026-10-16 の改訂。それより前は[時点証拠 ADR](2026-09-18-stocktake-time-evidence.md) の ② 〜 ④）で実装し、⑤ で一括して切り替える。それまでの間、現行の入口を使うと在庫・売上を誤らせる。店は未導入で本番 DB が無いため、停止が効くのは開発・demo・試験 DB である。

## Decision

### SPEC-STOP-D1: 5 つの BIZ 入口が最初の文で停止する

BIZ の公開関数 `start_stocktake` / `update_count` / `complete_stocktake`（BIZ-06）と `commit_csv_import` / `rollback_csv_import`（BIZ-03）は、関数の最初の文で停止 error を返す。DB を読まず、TX を開かず、operation log を書かない。条件による例外（force_fill なし、同日 import なし、rolled_back 済みの冪等 Ok 等）を持たない。引数の検査より停止が先で、どの入力でも同じ停止 error になる。

### SPEC-STOP-D2: 停止 error は既存の kind で返す

BIZ-06 は `BizError::ValidationFailed(<BIZ-06 停止文言>)`、BIZ-03 は `BizError::ImportError(<BIZ-03 停止文言>)` を返す。CMD の既存変換で kind = `validation` / `import_error`、message = 停止文言、field = null、error_id = null になる。`CmdErrorKind` は 12 値のまま（D-061）。command の名前・引数・戻り型・登録・`src/lib/bindings.ts` は変えない。CMD `commit_csv_import` は停止 error で preview token を消さない（成功時と snapshot 不一致時だけ消す既存の分岐へ到達しない）。

停止文言の正本は [35 §20.0](../function-design/35-biz-stocktake-service.md#200-現行buildの一時停止)（BIZ-06）と [32 §15.0](../function-design/32-biz-csv-import-service.md#150-現行buildの一時停止)（BIZ-03）。

### SPEC-STOP-D3: 旧本体は crate 内部に残し、test だけが呼ぶ

現行の処理本体は `legacy_start_stocktake` / `legacy_update_count` / `legacy_complete_stocktake` / `legacy_commit_csv_import` / `legacy_rollback_csv_import`（`pub(crate)`）として残す。呼出し元は `#[cfg(test)]` の test・診断・fixture だけとする。非 test build の dead_code 警告は旧本体に `#[cfg_attr(not(test), expect(dead_code))]` を置いて抑える。production から旧本体を呼ぶ変更が入ると期待が満たされず、CI と local-ci の必須 gate `cargo clippy --all-targets --all-features -- -D warnings` が `unfulfilled_lint_expectations` で失敗する（通常の `cargo build` は warning に留まる）。これで呼出し元が test だけであることを機械検査する。既存の BIZ test と `#[ignore]` 診断 2 本は旧本体を呼び、既知の不具合を再現し続ける。新処理へ移すときの回帰の対照に使い（診断 2 本は新しい経路で PASS する回帰 test に変える）、⑤ で旧本体を撤去する。

### SPEC-STOP-D4: 画面は案内を出し、書く入口を無効にする

停止状態は画面ごとの定数 1 つが所有し、画面 component はそれを既定値にする prop で受ける（既存の flow test は停止 off を渡して維持する）。停止中は warning の Alert（icon + 見出し + 本文）を出し、書く操作の入口を無効化する。閲覧・絞り込み・プレビュー・他タブは無効化しない。画面の定数は安全の根拠にしない（backend が D1 で独立に拒否する）。案内の文言は [73 §73.15](../function-design/73-ui-stocktake.md#7315-現行buildの一時停止)（棚卸し画面）と [55 §55.0](../function-design/55-ui-csv-import.md#現行buildの一時停止)（Z004 タブ）を正本とする。

### SPEC-STOP-D5: 再現 fixture は入力と停止の検査だけを固定する

合成データの builder F1〜F7（STK-1、STK-2、旧形式の明細と取込み記録、ゼロ行、共有 JAN、逆順、欠落）を `src-tauri/src/biz/csv_import_service/tests/legacy_stop_tests.rs` に置く。① では各 fixture 状態の前提値と、5 入口の停止・DB 全 table の不変だけを assert する。後続 lane の期待結果は packet の Test Design Matrix に記録し、実行しない。判定不能の扱いに依存する期待結果は ADR 修正 lane の版に従う。

### SPEC-STOP-D6: 解除は ⑤ だけで行う

停止は ⑤（migration 登録・共通 writer・新 command / bindings / 画面の一括切替。2026-10-16 の改訂からは、置き換えた ADR の SPEC-STK-CNT-D12 の (b) の配布の単位）でだけ外す。部分的な解除（例: 開始だけ許可、同日 import がなければ commit を許可）をしない。解除時に本 ADR を superseded にして記録する。

## Rejected Options

- Option: 確定式だけを部分修正する（例: `現在庫 + N − system_stock`）。
  - Why rejected: 開始から実測までの移動を誤算する。時点証拠 ADR の計数窓・受領順の判定を欠いたまま別の誤りを作る。
- Option: 条件付きで旧処理を許可する（force_fill なし、同日 import なし等）。
  - Why rejected: 例外が増え、停止 lane が本実装に膨らむ。条件の網羅を証明できない。
- Option: `pos_stock_sync` を自動で off にして Z004 の売上だけを通す。
  - Why rejected: 連動設定の意味が変わり、⑤ の移行で元の設定を復元できなくなる。
- Option: 新しい `CmdErrorKind`（例 `suspended`）を足す。
  - Why rejected: D-061 の 12 値凍結を解き、bindings・invoke・41 / 42 / 40 / 30 を動かすが、⑤ で消える一時状態である。UI は入口を先に無効化し kind で分岐しない。
- Option: 旧本体を削除する、または `#[cfg(test)]` へ移す。
  - Why rejected: 削除は既存 BIZ test と診断を消し、③ / ④ の移行先の回帰を失う。`#[cfg(test)]` 化は旧本体だけが使う helper を非 test build で dead_code にし、clippy `-D warnings` が落ちる。
- Option: 画面の非表示だけで止める。
  - Why rejected: 古い画面・直接の command 呼出しで旧処理が動く。

## Consequences

- 開発・demo DB の旧方式の進行中棚卸しは ⑤ まで閲覧だけできる（完了・中止できない）。商品の新規登録・一括 import は停止対象外で、進行中の旧棚卸しへ未計数の明細を追加し続ける。
- 停止中は Z004 由来の商品別売上が増えず、棚卸しの評価額も増えない。日報の公式集計、入庫・手動販売・返品・廃棄は従来どおり。
- CMD `commit_csv_import` の snapshot 不一致時の token 削除、成功時だけの token 削除、追加確認 token での commit 成功は停止中は到達不能で、⑤ で新 commit 経路へ再接続するまで CMD 層の test がない。
- ホームの「前日分が未取込みです」警告は、過去の Z004 取込みを持つ開発 DB で出続ける（本番 DB は取込み 0 件で始まり出ない）。

## Evidence

- STK-1 / STK-2 の再現: `diagnostic_cross_feature_req205_count_then_movement`（`XFA_TEMPORAL_FAIL`）、`diagnostic_cross_feature_req205_req401_late_import`（`XFA_LATE_IMPORT_FAIL`）。停止後も旧本体で FAIL し続ける。
- 停止の検査: `legacy_stop_tests.rs`（BIZ、F1〜F7 × 5 入口、DB 全 table 比較）、`stocktake_cmd` / `csv_import_cmd` の test（production command、wire 完全一致）、`StocktakePage.test.tsx` / `CsvImportPage.test.tsx` / `PreviewStep.test.tsx`（案内と無効化）。
- 店は未導入で本番 DB と本番取込み履歴が無い（owner 2026-09-19）。

## Revisit Trigger

⑤（migration・新 writer・新 command・新 UI の一括切替。2026-10-16 の改訂からは、[棚卸しの数の置き場と在庫の求め直しの ADR](2026-10-16-stocktake-count-placement.md) の runtime のうち SPEC-STK-CNT-D12 の (b) の配布の単位）の実装 lane。解除時に本 ADR を superseded にし、旧本体 `legacy_*` と停止の定数・案内を撤去する。
