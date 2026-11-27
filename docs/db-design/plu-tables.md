# テーブル定義（PLU スロット永続割当）

> **親文書**: [DB_DESIGN.md](../DB_DESIGN.md)
> **設計判断**: [decision-log.md D-072](../decision-log.md#d-072-plu-slot-永続割当と段階導入2026-08-18)、[D-117](../decision-log.md)

---

## 25. plu_slots

### 役割

SR-S4000 のスキャニング PLU 領域について、レジで観測した占有とアプリが管理する JAN 単位の予約をメモリ No. ごとに永続化する。アプリ管理スロットはアプリ、既存登録はレジを authority とし、空き判定には Z004 全スロットダンプの照合を必須とする（**BIZ-04-D3 / SPEC-PLS-D1、D2**）。照合は毎日の締めの SD の読取りで自動で行う（[33 §16.3](../function-design/33-biz-plu-export-service.md)、D-117 (3)）。アプリは枠を消さず、ある JAN の枠を別の JAN に使い回さない（D-117 (1)）。

### カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---|---|---|---|
| memory_no | INTEGER | PRIMARY KEY, CHECK(217 <= memory_no AND memory_no <= 5000) | レジのスキャニング PLU メモリ No. |
| scanning_code | TEXT | NULLABLE | app 管理時は 13 桁 JAN、既存登録はレジで観測したコード、free は NULL |
| status | TEXT | NOT NULL, CHECK(status IN ('free','external','reserved','active')) | スロット状態 |
| reserved_at | TEXT | NULLABLE | 予約日時 |
| activated_at | TEXT | NULLABLE | slot が active になった日時。アプリ側の記録で、書くのは reserved の保存済み確認（confirm）、照合での観測による採用（reserved → active、free → active）、prepare での external の採用。実レジへの反映完了は証明しない（UI-08-D2、D-011 / D-023） |
| updated_at | TEXT | NOT NULL | 最終更新日時 |

`scanning_code` には `status IN ('reserved','active')` の行だけを対象とする partial UNIQUE index を置く。同じ JAN をアプリ管理の複数スロットへ割り当てる状態は DB が拒否する。`external` は対象外とし、同じコードの `external` が複数行あること、`external` と `reserved` / `active` が同じコードを持つこと（レジがアプリの JAN を別の枠にも持つ観測。[33 §16.3](../function-design/33-biz-plu-export-service.md)）を許す。

### migration（D-117）

migration v5 は table と partial UNIQUE index を作り、memory No. 217〜5000 の **4,784 行**を `status='free'`, `scanning_code=NULL` で事前投入した（**MNT-03-D9 / SPEC-PLS-D1**）。D-117 の runtime（PLU の書出しの見直し）が、次の migration を足した（版と形は [22 §24](../function-design/22-mnt-migration.md) の vP、MNT-03-D25）。

1. status の CHECK を上の 4 値にする（SQLite は CHECK の変更に表の作り直しが要る）。作り直しで `released_at` を外す。1 つの TX で全行を新しい表へ写し、同じ TX で、作り直しの前の行と memory No. ごとに全部の列を比べ、違えば何も変えずに止める。
2. 既存の `release_pending` の行を `external` にする（`scanning_code`・`reserved_at`・`activated_at` は変えない。対応づけは写しの `INSERT … SELECT` の `CASE` で行う。旧い表を `UPDATE` すると、旧い partial UNIQUE が `external` を含むので、同じ JAN の `release_pending` と `external` / `active` の組で UNIQUE 違反になる）。この行はレジに登録が残っていてアプリが書かない枠で、その JAN を再び対象にすると prepare の step 5 が同じ memory No. を採用する。
3. partial UNIQUE index を `status IN ('reserved','active')` で作り直す。
4. `app_settings` の key `plu_register_snapshot_summary`（もう書かない・読まない）と `plu_register_snapshot_at` を消す。照合日時を消すのは、前の版の DB の照合日時で書出しの gate（`register_snapshot_required`）を通さないため（前の版の app が復元を終えた DB・復元の途中で止まった DB を含む。[33 §16.8](../function-design/33-biz-plu-export-service.md)、owner の決定 2026-11-26、repo 外の回答台帳 TD-912）。版を上げた DB は、書出しの前に SD を 1 回読む。

本番の DB は無い（店は未導入）ので、対象は開発・demo・試験の DB と owner の PC の DB である。数量・在庫・売上・商品は変えない。

### products との結合

`plu_slots.scanning_code = products.jan_code` を JAN 単位の結合キーとする。product_code は slot identity に使わない。同一 JAN の商品群は 1 スロットを共有する。前の JAN の別名（[master-tables](master-tables.md) §1a、D-122）は結合キーに入れない（別名の JAN の枠は「対象の商品を持たない JAN の枠」のまま。枠の遷移は別名を読まず書かない）。商品一覧・商品詳細は memory No. を返さない（D-117 (5)。UI-01b-D19 の撤去）。

### 状態遷移

| 現在状態 | 契機 | 次状態 | 必須処理 |
|---|---|---|---|
| free | prepare | reserved | 最小空き番号、reserved_at |
| free | 照合: レジ有・対象の JAN 一致（その JAN に reserved / active なし） | active | 採用、activated_at。同一 JAN の観測が複数なら最小番号だけ active、残り external（観測コードを保持） |
| free | 照合: レジ有・reserved / active を持つ JAN と同一コード | external | 観測コードを保持。アプリの枠は変えない（`duplicate_on_register`、33 §16.3） |
| free | 照合: レジ有・その他 | external | 観測コードを保持 |
| free | 照合: レジ空 | free | no-op |
| external | prepare: 同一コードの JAN が対象かつ reserved / active なし | active | 採用（同一コードの external が複数なら最小番号）、activated_at。新規 free 予約はしない（レジ既存登録の app 管理への移行） |
| external | 照合: レジ空 | free | scanning_code を NULL |
| external | 照合: レジ有・同一コード | external | 維持 |
| external | 照合: レジ有・別コード | external | 観測コードを更新 |
| reserved | confirm / 照合: レジ有・同一コード | active | activated_at |
| reserved | 照合: レジ空 | reserved | 未書込み予約を維持 |
| reserved | 照合: レジ有・別コード | external | 予約破棄、reservation_dropped、その JAN は次の prepare で新しい空き枠を予約 |
| reserved | 保存失敗・キャンセル後の再 prepare | reserved | 同じ番号を維持 |
| reserved / active | 商品の対象外化・廃番化・JAN の変更 | 変えない | 対象の商品を持たない JAN の枠として残し、書き出さない（33 §16.6） |
| active | 照合: レジ空 | active | missing 報告、その JAN の対象の商品を dirty |
| active | 照合: レジ有・同一コード | active | 維持 |
| active | 照合: レジ有・別コード（その JAN に対象の商品がある） | active | conflict 報告、その JAN の対象の商品を dirty |
| active | 照合: レジ有・別コード（その JAN に対象の商品が無い） | external | 観測コードを保持（33 §16.3） |

別コード衝突、同一 JAN 重複、要修正商品の扱いを含む照合表の正本は [33-biz-plu-export-service.md](../function-design/33-biz-plu-export-service.md) §16.3 とする（**BIZ-04-D3 / SPEC-PLS-D2〜D4**）。

### db::plu_slot_repo の責務

- memory No. 順の全 slot 読取り、JAN / status による検索
- 最小 `free` slot の予約、sticky allocation の再取得
- スナップショット照合、prepare、confirm を呼出側の同一 transaction 内で行う CRUD
- `products.jan_code` との結合
- status CHECK、範囲 CHECK、partial UNIQUE 違反を `DbError` に変換し、部分更新を残さない

## app_settings の PLU スナップショットキー

| key | value | 説明 |
|---|---|---|
| plu_register_snapshot_at | ISO 8601 timestamp | 最後に Z004 全スロット占有を照合した日時 |
| plu_register_snapshot_settlement_no | 10 進の整数の文字列 | 最後に照合した Z004 の精算回数。これより小さい精算回数の Z004 は照合しない（古い SD。33 §16.3 手順 2）。照合の TX で更新する |

一度も照合していない状態では prepare を `register_snapshot_required` で拒否する。照合は毎日の締めの SD の読取りで行い、店主が別に読む操作は無い（**BIZ-04-D3 / SPEC-PLS-D2、D-117 (3)**）。占有の件数は app_settings に置かず、照合の operation_logs にだけ残す。

## 集計境界

`plu_slots` と上記 app_settings は PLU 登録の制御情報であり、売上、在庫、会計の集計には加算・減算・仕訳として参加しない。Z001 / Z002 / Z005 の公式日報集計と Z004 の商品別売上・在庫変動は従来どおり別 series で処理し、PLU slot 状態を集計根拠にしない（**BIZ-04-D7 / SPEC-PLS-D8、D-025**）。
