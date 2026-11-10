# Plan Packet: 棚卸しの P1（STK-1 / STK-2）を直す ㉘ ② 受領・判定・保存の基盤

wave に属さない単独の lane。owner の lane 選択（2026-10-06）で起票した。並走は `agent/z001-display`（PR #145、Final Review 中）、本日起票の `agent/sd-direct-read`・`agent/stocktake-p1`（本 lane）・`agent/plu-clear`、`agent/npm-audit-1006`（npm の開発用依存）。

本 lane は [停止 ADR](../../adr/2026-09-23-legacy-stocktake-z004-write-stop.md) が定める ㉘ の runtime の列（① 停止と再現 fixture〈PR #95 で完了〉→ ② 受領・判定・保存の基盤 → ③ 計数と補正 → ④ 取込み・取消・回復 → ⑤ 一括切替、出典は `docs/archive/plans/2026-09-23-legacy-stocktake-z004-write-stop.md:3`）の ② だけを扱う（Wave Operation の 1 是正単位 = 1 packet）。③〜⑤ と、ADR が名指しする design lane「実測と POS 系列の対応を取得・保存する」は `docs/backlog.md` の entry が持つ。lane の選び方は [decision-log](../../decision-log.md) の D-109。

## Workflow State

- Phase: archive
- Risk: R3
- Plan Commit: df62d13b5814fc9afd218621c373d8331b5c6ee5
- Amendments: none
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（subagent_type: writer）
- Plan Reviewer: fresh Opus 5.5 + Codex（model は発注時に決める）
- Final Reviewer: Fable 5.1（Claude 側、R3）+ Codex（GPT-6.1 Sol 既定）。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Review Minimum: 1
- Human Gate: ready,merge
- Branch: agent/stocktake-p1

遷移の記録（append-only）:

1. kickoff → spec-check（2026-10-06、起草役）: 依頼を ㉘ ② に絞り、Risk を R3 と記録した（下の Risk）。
2. spec-check → design（2026-10-06、起草役）: 設計正本（ADR D1〜D4 / D8、35 / 32 / 20 / 21 / 23 / 24 の proposed 節、db-design の tracking / pos / master の proposed 節）は ② の中身を定めているが、② の未配線の入れ方（registry に登録しない schema 関数、test だけが呼ぶことの機械検査、旧 writer と並ぶ新関数）と、旧 import から受領記録を backfill するかが正本に無い。
3. design → plan-draft（2026-10-06、起草役、`02379046`）: 同じ plan-first の commit で D-109 を足し、`docs/db-design/pos-tables.md` の proposed 節に backfill しないことを 1 文足した。② の実装に残る設計の問いは無い。owner の判断事項（下の「owner の判断事項」Q1〜Q3）は ② の Scope を変えないが、Q2 は ADR の Status の扱い、Q3 は Matrix の ⑤ の行の期待を決めるので、Plan Gate の前に owner へ諮る。Phase は plan-draft で止める。
4. plan-draft（owner の決定の反映、2026-10-06、起草役、本 commit）: owner の決定 Q2 = (A)・Q3 = (i)（下の「owner の判断事項」）を packet・Matrix の G2b・D-109 に反映した。Q1 は本 lane が依らない design lane の判断として残す。
5. plan-draft → plan-gate（2026-10-06、起草役、本 commit）: packet と Test Design Matrix は plan-first commit `02379046` と本 commit で揃い、本 lane に未決は無い。Plan Commit は Plan Reviewer が P1/P2 = 0 と報告するまで pending。
6. plan-gate → plan-approved（2026-10-07、Coordinator）: Plan Review round 3（上限、`fe691b4b`）で 2 本の独立 reviewer がともに P1 / P2 = 0（Review Response）。Plan Commit は P3 を直した `df62d13b`。
7. plan-approved → implementing（2026-10-07、Coordinator）: AC2・AC11 の baseline を発注の直前に逐語で再実行し一致（`rg -c 'version: [0-9]+,'` = 7、`#[ignore` = 2、診断 2 本 FAIL で `XFA_TEMPORAL_FAIL` と `XFA_LATE_IMPORT_FAIL`）。Writer へ発注する。

## Owner Effort Budget

- 介入回数上限: 6（既定）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 6 | 2（2026-10-06: Q2 ADR の Status、Q3 STK-2 の green の定義） | 2（Ready、merge） | 2 | 6 = 2 + 2 + 2 |

Q1（design lane の着手条件 (4)）は design lane の判断で、本 change の介入に数えない（design lane の起票時に owner へ諮る）。manual（L3）は無い。

## Risk

Risk: R3

Reason:
DB の schema（新しい列・表・CHECK・旧明細の分類）、POS CSV（Z004 の parser の出力型に精算の識別メタを足す）、在庫の数量と版の不可分更新、取込みの前後の判定を作る（`docs/DEV_WORKFLOW.md` Risk Tiers の R3 の行）。R4 に当たらない: 新しい schema 関数は migration の registry に登録せず、通常起動の DB を変えない（D-109 (2)）。既存の行・売上・在庫を削除・書換えする処理は無く、test の DB だけに適用する。この変更で required gate の green / red は変わらない（診断 2 本は `#[ignore]` のまま、新しい test は通常の `cargo test` に加わる）。Scope の path は `scripts/ci/classify-changes.sh:55-65` の workflow の一覧に当たらない（`src-tauri/**` と `docs/**` の製品の文書だけ）ので Final Review Minimum は 1。

## Goal

Goal Invariant:

### 最小完了条件

- 時点証拠の新しい schema・受領・精算の同一性の検査・前後の分類が、試験 DB の上で ADR どおりに動くことを自動 test が示し、③（計数と補正）と ④（取込み・取消・回復）がこの基盤の上に作れる状態で main に入る。
- 通常の起動・画面・command の振舞いは ① の停止のまま変わらない（通常の起動で DB の schema が変わらず、旧本体と停止の入口はそのまま）。

### 失敗定義

- 通常の起動の DB に新しい列・表が入る、または旧 writer が新しい schema の上で動く中間版が生まれる（D-109 (2)、`docs/db-design/tracking-system-tables.md` の「移行と保存TX」の 1 項目め）。
- 受領済みでない資料・計数を始めた後に受領した資料を「実測前」と分類する、legacy の実測を未実測へ読み替える、識別メタの欠けた同日の追加を通す（ADR D2〜D4）。
- 新しい関数・型が production から呼べてしまう（D-109 (3) の `#[cfg(test)]` の隔離が効かない）。
- 既存 test を削除・skip・弱体化する。診断 2 本を ignore を外すだけで green にする。

### 非目的

- STK-1 / STK-2 を ② で解消すること（解消は ③〜⑤。Matrix の「STK-1 / STK-2 の green の条件」の行）。
- 停止の解除・migration の registry 登録・旧 writer の切替・command / bindings / 画面（⑤）。
- 計数 context・確定補正・独立再実測（③）、取込みへの判定の配線・要再確認 flag の作成と解消・取消補償・準備照会・在庫連動の有効化の拒否（④）。
- 時刻・番号印字・EJ による前後判定、精算系列の比較方法（design lane「実測と POS 系列の対応を取得・保存する」）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

本 lane は data の契約（schema・受領・分類）を作るが、通常の起動へ配線しない。operator の操作は ① の停止のまま変わらない。通常運用の列は「店が営業し、締めで Z004 を取り込み、棚卸しで数える」で、その達成は ⑤ と design lane の後。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 店: アプリ未導入（本番 DB なし、`docs/project-memory.md` の初導入の行） | いつもどおり営業・精算・Excel | 本 lane の影響なし | ⑤ と design lane の後に導入 | なし |
| 開発・demo DB（schema v7） | アプリを起動する | 起動時の migration は v7 まで。新しい列・表は入らない（D-109 (2)） | ⑤ で registry に登録 | なし（AC2 の test で固定） |
| 同 DB | 棚卸し画面・Z004 タブを開く | ① の停止の案内のまま（`docs/adr/2026-09-23-legacy-stocktake-z004-write-stop.md` SPEC-STOP-D4） | ⑤ で解除 | なし |
| 同 DB | Z004 を選んでプレビューする | 今と同じプレビュー。精算の識別メタは parser が抽出するが、画面には出さない | ④ で受領と分類へ配線 | P4（layout A のメタ行） |
| 開発者・reviewer の試験 DB | test の helper で v7 の上に時点証拠 schema を当てる | 旧明細が uncounted / auto_filled / legacy に分かれ、受領・同一性の拒否・前後の分類を test から呼べる | ③ / ④ がこの helper の上に作る | P2・P3 |
| アプリ再起動・翌日 | 同じ操作 | 状態を持たないので同じ結果 | ⑤ | なし |

この packet を完了できること（③ / ④ の基盤が test で確かめられた形で main にある）と、通常運用を達成できること（棚卸しと Z004 の取込みを再開し、日次で在庫を自動で合わせる）は別である。前者だけが本 lane の完了条件。

## Scope

予定 file（本 lane が書く全件。生成物を含む）:

- S1（新規）`src-tauri/src/db/schema_time_evidence.rs`: 時点証拠 schema を当てる関数 `apply_time_evidence_schema(conn: &Connection, version: i64) -> Result<(), DbError>`（`MigrationKind::Custom` と同じ signature、`src-tauri/src/db/migration.rs:27`）。中身は `docs/db-design/master-tables.md`・`tracking-system-tables.md`・`pos-tables.md`・`transaction-tables.md` の proposed 節: `products.stock_revision`（NOT NULL DEFAULT 0、非負 INTEGER の CHECK）と `pos_sync_disabled_revision`、`stocktakes.reconciliation_version`（DEFAULT なし。`stocktakes` も table 再構築で作り直し、旧 header は 0 を明示して埋める。DEFAULT 0 だと ⑤ まで残る旧 writer の `INSERT INTO stocktakes (started_at, status)`〈`src-tauri/src/db/stocktake_repo.rs:188`〉が新 schema で黙って版 0 を作る。`tracking-system-tables.md:43` の「恒久DEFAULTを付けない」と同じ理由）、`stocktake_items` の `observation_kind`（DEFAULT なし、table 再構築で既存行を分類して埋める。再構築で `idx_stocktake_items_stocktake_product`〈`src-tauri/src/db/schema_v1.rs:260`〉を作り直し、`tracking-system-tables.md:28` の新しい index 3 本〈item の (product_code, observation_revision)、recount の (stocktake_item_id, observation_revision)、flag の csv_import_id〉を足す）と証拠の列、新表 `stocktake_recounts`・`stocktake_recount_flags`・`pos_import_sources`、`csv_imports.source_id`（旧 import は NULL のまま。backfill しない、D-109 (4)）、`inventory_movements.stocktake_adjustment_kind` / `stocktake_recount_id`、`app_settings` の `stocktake_legacy_movement_ceiling`。1 つの TX で当て、失敗は全部戻す。TX の閉じ方は `migration_tx::commit_transaction` / `rollback_after_error` を使い（`src-tauri/src/db/schema_v7.rs:103`・`:108` と同じ形）、再構築のために `PRAGMA foreign_keys` を OFF にするなら TX の外で前の値を保存し、`migration_tx::restore_foreign_keys`（`src-tauri/src/db/migration_tx.rs:136`）で戻す（`schema_v2.rs:18-27` の手順と同じ形）。`stock_revision` の DEFAULT 0 は `master-tables.md:7` の明記どおり残す。`schema_versions` には呼出し側が渡す `version` を記録する。`migrations()` には登録しない（⑤）。file 全体が `#[cfg(test)]` の module（下の mod.rs の行）。
- S1（変更）`src-tauri/src/db/mod.rs`: `#[cfg(test)] mod schema_time_evidence;` を足す（`schema_v1`〜`v7` と同じ private mod、`src-tauri/src/db/mod.rs:22-28`。`#[cfg(test)] pub(crate) mod test_support;` の `:30-31` と同じ形）。
- S1（変更）`src-tauri/src/db/test_support.rs`: 時点証拠 schema を当てた試験 DB を作る helper（`setup_test_db` の後に `apply_time_evidence_schema(conn, migration::app_max_version() + 1)` を呼ぶ）。旧明細の各形（AC3）と旧 import を v7 の上で作ってから当てる fixture の helper もここに置く（形は `src-tauri/src/biz/csv_import_service/tests/legacy_stop_tests.rs:303` の F3 と同じ）。旧明細と旧 import は生の SQL で作り、`db/test_support.rs` から旧 BIZ（`legacy_*`）を呼ばない。
- S2（変更）`src-tauri/src/db/inventory_repo.rs`: 既存の `#[cfg(test)] mod tests`（`inventory_repo.rs:348`）の前に `#[cfg(test)] pub(crate) mod time_evidence { … }` を 1 群で置き、その中に新関数 `update_stock_quantity_with_revision(conn, product_code, new_quantity) -> Result<bool, DbError>`（数量と `stock_revision` の checked 増分を 1 つの UPDATE で。`docs/function-design/21-io-inventory-repo.md` の proposed 節）と `bump_stock_revision(conn, product_code) -> Result<i64, DbError>`。既存の `update_stock_quantity`（`inventory_repo.rs:151`）とその caller 4 か所（`inventory_service/common.rs:64`、`csv_import_service/commit.rs:274`、`stocktake_service.rs:571`、`integrity_service.rs:173`）は変えない（⑤ で置き換える）。
- S2（変更）`src-tauri/src/db/sales_repo.rs`: 既存の `#[cfg(test)]` の module（`sales_repo.rs:1410`）の前の `#[cfg(test)] pub(crate) mod time_evidence { … }` に、`pos_import_sources` の受領の upsert（hash が既にあれば最初の ID と時刻を返す）、最大 source ID（空なら 0）、精算の同一性の照合候補（同じ帳票種別で machine_no / settlement_no が一致する全受領 source、未取込み・取消済みを含む）、active import の識別メタの取得（`csv_imports` を起点に source なし・メタ欠けの行も残す）、同一性の拒否の初回記録と照会（`docs/function-design/24-io-csv-import-repo.md` の proposed 節の表の 1〜5 行）。
- S2（変更）`src-tauri/src/db/stocktake_repo.rs`: 既存の `#[cfg(test)]` の module（`stocktake_repo.rs:662`）の前の `#[cfg(test)] pub(crate) mod time_evidence { … }` に、商品の最新の有効な観測の読取り（kind・所属・`source_cursor`・`observation_revision`。`docs/function-design/20-io-product-repo.md` の proposed 節「有効観測の列挙」）。選び方: measured の明細（active / 完了済み）と `stocktake_recounts` の行を合わせ、`observation_revision` が最大の 1 件を返す（明細と recount の別や時刻の順では選ばない）。版を持つ観測が 1 つも無く legacy の明細があれば LegacyObserved。legacy は migration だけが作り版を持たないので、版を持つ観測があればそれが legacy より新しい。auto_filled と uncounted は観測にしない（前の観測を隠さない）。measured の行で `source_cursor` か `observation_revision` が NULL なら `DbError::QueryFailed`（データ破損）にする。kind と証拠の列の組の CHECK は本 lane では付けない（tracking の正本を触らないため。③ への申し送り）。
- S3（変更）`src-tauri/src/io/z004_parser.rs`: `ParseResult` に `settlement_metadata: Option<SettlementMetadata>`（machine_no / report_kind / settlement_no / settled_at の各 `Option<String>`、番号は文字列のまま）を足し、layout A のメタ行から抽出する（`docs/function-design/23-io-z004-parser.md:5`、メタ行の並びは同 `:130`）。machine_no は「マシンNo.」行、settlement_no は「精算回数」行、report_kind は「ファイル」行の値（前後の空白を除くだけ。「モード」行は抽出しない）、settled_at は抽出済みの精算日と「時刻」行の `HH:MM` を `YYYY-MM-DDTHH:MM` に結合する（分精度。秒を補わない。時刻行が無い・解釈できなければ `None`。日付だけから時刻を補わない、23:7）。従来 shape は各項目 `None`。「ファイル」行が無い・空のときは、machine_no・settlement_no の行があっても両方を `None` で返す（report_kind の無い番号だけの組を作らない。同一性の検査がこの形をメタ不足として拒否する、S4 (b)）。S3 は production の parser に入るので `#[cfg(test)]` にしない。`SettlementMetadata` は `ParseResult` と同じ `pub struct`、抽出の関数は private の `fn`（Contract Probe P5）。`ParseResult` を作る所は `z004_parser.rs:210` だけ、読む所は `src-tauri/src/biz/csv_import_service/parse.rs:34` だけ（`rg -n 'ParseResult \{' src-tauri/src` と `rg -l 'parse_z004\(' src-tauri` で確認）。struct literal で `ParseResult` を作る test は無いので、既存の test の呼び方は変わらない。
- S4（新規）`src-tauri/src/biz/csv_import_service/time_evidence.rs`（file 全体が `#[cfg(test)]` の module）: (a) 資料の受領（構文と種別を検証した後の短い独立 TX、ADR D2）、(b) 精算の同一性の検査と拒否の記録（同一精算の別 hash と、同じ精算日の active import があるときの識別メタの欠け。拒否は業務 write の前、拒否の証拠は独立 TX、ADR D3、`docs/function-design/32-biz-csv-import-service.md` の proposed 節「一つの分類関数とcommit」の後半 3 段落）。rusqlite の `Transaction` は `Deref` だけで `DerefMut` を持たない（rusqlite 0.31.0 の `transaction.rs:232`）ので、caller の TX の中から `&mut DbConnection` を受ける関数は呼べない（`E0596`、Contract Probe P6）。そこで 3 つに分ける: core の `inspect_settlement_identity(conn: &Connection, …)` は書込みの無い判定で、`&tx` でも TX の無い test からも呼べる。証拠の `record_identity_rejection_evidence(conn: &mut DbConnection, …)` は自分で TX を開き、`UPDATE pos_import_sources SET identity_rejection_code=?2, identity_rejected_at=?3 WHERE id=?1 AND identity_rejection_code IS NULL` の 1 文で 2 列を一緒に書き（`pos-tables.md:12` の「両方NULLまたは両方必須」）、`rows_affected == 1` を返して commit する。wrapper の `check_settlement_identity(conn: &mut DbConnection, …)` は業務 TX → core → `Some(code)` なら `tx.rollback()?` → 証拠 → `Err(IdentityGuardError::Rejected(code))`。rollback の失敗は証拠を書かずに `IdentityGuardError::Db`（32:42 の「保存失敗はDB error」と同じ出口）。`?` の早期 return は drop の既定の rollback で戻る。④ の caller は、自分の hash の再検査の後に caller の TX の中で core を呼び、`Some` なら同じ 3 行（rollback → 証拠 → Err）にする。`&mut DbConnection` の排他借用が 2 つの TX をまたぐことが 32:42 の「DB lockを保持」の実体（接続は 1 本）。core の規則: 同日 active の集合から取込み対象の source を除く述語は `source_id IS NULL OR source_id <> ?target`（`<>` だけだと source の無い旧 import の行が落ち、比較先を 0 件として通す）。衝突の検査は report_kind・machine_no・settlement_no の 3 つが全て Some のときだけ（NULL どうしは衝突にしない）。メタ不足の検査は正本どおり machine_no・settlement_no だけを見る（観測した実 file のメタは 6 行だが〈`23-io-z004-parser.md:130`〉、parser は可変のメタを受理し各項目を独立に抽出するので、S3 が「report_kind が None なら machine_no・settlement_no も None で返す」を強制する。これで report_kind の無い file はメタ不足の検査に当たり、衝突の検査をすり抜けない）。両方に当たるときは `IdentityConflict` を先に返す（32:40-44 の記述の順）。対象の source を `get_pos_import_source` で読めなければ `BizError::DatabaseError(DbError::NotFound)`、(c) 前後の分類（同節の 1〜5。`pos_stock_sync=false` は売上のみ、NeverObserved は After、LegacyObserved は Unknown、`source.id <= source_cursor` は Before、それ以外は Unknown。Unknown の flag の理由は数量が 0 でない行 = `sale_order_unknown`、数量 0・金額が 0 でない行 = `offset_lines_present`、LegacyObserved = `legacy_basis`、数量・金額とも 0 の行 = 「相殺の判定が要る」という印だけ返す〈判定は ④〉。共有 JAN の行は全在庫連動候補が Before のときだけ commit 可、他は file 全体を held）、(d) 在庫判定の行の集合（正常 JAN の全行を 0/0 行も含めて全候補と `pos_stock_sync` 付きで集める。売上の行の集合〈`parse.rs:94` の 0/0 除外〉は変えない、`23-io-z004-parser.md:9`。正常 JAN で商品の候補が 0 件の行は在庫判定の行に入れない〈分類の入力にしない。未登録の扱いは既存の preview の規則のまま〉）。時刻・精算日時を入力にしない。
- S4（変更）`src-tauri/src/biz/csv_import_service/mod.rs`: `#[cfg(test)] mod time_evidence;` を足す。
- S5（test）上の各 file の `#[cfg(test)]` の test と、`src-tauri/src/biz/csv_import_service/tests/` に `time_evidence_tests.rs` を新設して `tests/mod.rs` に登録する（Matrix の T 行）。既存 test は変えない。
- S6（生成物）`docs/function-design/90-traceability.md`: 新しい test の REQ 参照で `cargo run --bin generate_traceability` が再生成する（手で編集しない。他 lane と衝突したら取り込んだ後に再生成、D-098）。
- S7（docs、本 commit）`docs/decision-log.md` の末尾に D-109、`docs/db-design/pos-tables.md` の proposed 節に backfill しない 1 文、`docs/backlog.md` の本 lane の entry（STK-1 / STK-2、design lane の着手条件の状態、診断 2 本、0/0 行、③〜⑤ の申し送り）。

未配線の機械検査（D-109 (3)）: S1・S2・S4 の新しい関数・型・module は ② 〜 ④ の間 `#[cfg(test)]` の中に置き、⑤ で一括に外す。各 repo file では新しいものを `#[cfg(test)] pub(crate) mod time_evidence { … }` の 1 群にまとめ、⑤ の差分を数行にする。production の code から呼ぶと型の解決で compile error（`E0425` / `E0432` / `E0433`）になり、lint の設定に依らない（Contract Probe P1）。CI の command と lint の設定は変えない（`.github/workflows/ci.yml:116` の `cargo clippy --all-targets --all-features -- -D warnings` は lib の test target も lint する）。根だけに `#[cfg_attr(not(test), expect(dead_code))]` を置く形は採らない: 根の下の新しい repo 関数を production から直接呼ぶ変更を止められず、`src-tauri/src/lib.rs:2-14` が `biz`・`db`・`io` に module 単位の `#[allow(dead_code)]` を持つので未使用の `pub(crate)` も error にならない（P1 の A0〜A2）。新しい関数は層を問わず `pub(crate)` にする（`src-tauri/tests/design_compliance_test.rs:477-503`・`:726-737` が設計書の code block に無い `pub fn` を panic にする。`#[cfg(test)]` の module は走査から外れる〈同 `:489-493`〉が、⑤ で cfg を外した後も `pub(crate)` のまま通るようにする）。S3 は production の parser に入るので cfg(test) にせず、`ParseResult` の field と `SettlementMetadata`（`pub struct`）だけを足し、抽出の関数は private の `fn` にする。`settlement_metadata` を読むのは cfg(test) の S4 だけで、production では読まれないが `io` の module 単位の allow で警告にならない。

S2 / S4 の内部 API（全て `pub(crate)`。型は同じ cfg(test) の module に置く。`DbError` / `BizError` は既存の型）:

| 配置 | 名前 | 署名 → 返却 | cfg(test) |
|---|---|---|---|
| `db/schema_time_evidence.rs` | `apply_time_evidence_schema` | `(conn: &Connection, version: i64) -> Result<(), DbError>` | ○（module ごと） |
| `db/inventory_repo.rs` `time_evidence` | `update_stock_quantity_with_revision` | `(conn: &Connection, product_code: &str, new_quantity: i64) -> Result<bool, DbError>`（対象なしは false。版が i64 上限なら書かずに `DbError::QueryFailed("stock_revision が上限に達しています")`。SQLite の CHECK の失敗も `DbError::QueryFailed` になる〈`src-tauri/src/db/mod.rs:138-158` の変換〉ので、T9 は固定の文で 2 つを区別する） | ○ |
| 同 | `bump_stock_revision` | `(conn: &Connection, product_code: &str) -> Result<i64, DbError>`（増分後の版） | ○ |
| `db/sales_repo.rs` `time_evidence` | `upsert_pos_import_source` | `(conn: &Connection, new: &NewPosImportSource) -> Result<PosImportSource, DbError>`（同じ hash は既存の行をそのまま返す） | ○ |
| 同 | `max_pos_import_source_id` | `(conn: &Connection) -> Result<i64, DbError>`（空は 0） | ○ |
| 同 | `get_pos_import_source` | `(conn: &Connection, id: i64) -> Result<Option<PosImportSource>, DbError>` | ○ |
| 同 | `find_settlement_identity_candidates` | `(conn: &Connection, report_kind: &str, machine_no: &str, settlement_no: &str, exclude_source_id: i64) -> Result<Vec<PosImportSource>, DbError>`（未取込み・取消済みを含む全受領） | ○ |
| 同 | `list_active_import_identities` | `(conn: &Connection, settlement_date: &str) -> Result<Vec<ActiveImportIdentity>, DbError>`（`csv_imports` の completed / completed_partial を起点に LEFT JOIN。guard の入力の精算日は NOT NULL なので引数は必須） | ○ |
| 同 | `record_identity_rejection` | `(conn: &Connection, source_id: i64, code: IdentityRejectionCode, rejected_at: &str) -> Result<bool, DbError>`（上の 1 文の UPDATE。初回の保存なら true、既にあれば書かずに false） | ○ |
| `db/stocktake_repo.rs` `time_evidence` | `find_latest_effective_observation` | `(conn: &Connection, product_code: &str) -> Result<Option<EffectiveObservation>, DbError>`（観測なしは None） | ○ |
| `biz/csv_import_service/time_evidence.rs` | `receive_source` | `(conn: &mut DbConnection, parsed: &ParseResult, received_at: &str) -> Result<i64, BizError>`（source ID。短い独立 TX） | ○（module ごと） |
| 同 | `inspect_settlement_identity` | `(conn: &Connection, source_id: i64) -> Result<Option<IdentityRejectionCode>, BizError>`（書込みなし。`&tx` でも呼べる。④ の commit TX はこれを呼ぶ） | ○ |
| 同 | `record_identity_rejection_evidence` | `(conn: &mut DbConnection, source_id: i64, code: IdentityRejectionCode, rejected_at: &str) -> Result<bool, BizError>`（自分で TX を開き `record_identity_rejection` を呼んで commit） | ○ |
| 同 | `check_settlement_identity` | `(conn: &mut DbConnection, source_id: i64, rejected_at: &str) -> Result<(), IdentityGuardError>`（業務 TX → core → rollback → 証拠の wrapper。`IdentityGuardError { Rejected(IdentityRejectionCode), Db(BizError) }` は wrapper 専用。wire の `source_identity_conflict` への変換は ④） | ○ |
| 同 | `collect_stock_evidence_rows` | `(conn: &Connection, parsed: &ParseResult) -> Result<Vec<StockEvidenceRow>, BizError>` | ○ |
| 同 | `classify_stock_rows` | `(conn: &Connection, source_id: i64, rows: &[StockEvidenceRow]) -> Result<StockClassification, BizError>`（時刻・精算日時を引数に持たない） | ○ |

型: `NewPosImportSource { file_hash, received_at, settlement_date: String, machine_no, report_kind, settlement_no, settled_at: Option<String> }`、`PosImportSource { id: i64` + 同じ項目 + `identity_rejection_code: Option<IdentityRejectionCode>, identity_rejected_at: Option<String> }`、`IdentityRejectionCode { IdentityConflict, MissingIdentity }`、`ActiveImportIdentity { csv_import_id: i64, settlement_date: String, source_id: Option<i64>, machine_no, report_kind, settlement_no: Option<String> }`、`EffectiveObservation { owner: ActiveItem(i64) | CompletedItem(i64) | Recount(i64), kind: ObservationKind }`、`ObservationKind { Measured { source_cursor: i64, observation_revision: i64 } | Legacy }`（Measured で cursor が無い形を型で作れない）、`StockEvidenceRow { line_no: usize, normalized_jan: String, quantity: i32, amount: i32, candidates: Vec<(String, bool)> }`（product_code と `pos_stock_sync`）、`StockClassification { Committable(Vec<RowDecision>) | Held { line_nos: Vec<usize> } }`、`RowDecision { line_no, product_code, effect: SalesOnly | Before | After | Unknown(UnknownReason) }`、`UnknownReason { SaleOrderUnknown, OffsetLinesPresent, LegacyBasis, OffsetCheckRequired }`（最後は 0/0 行の「相殺の判定が要る」印で、flag の理由ではない）。

型・関数の形を変える所の全件（kickoff の最終項目）: 形が変わる既存の型は `z004_parser::ParseResult` だけ（作る所 `z004_parser.rs:210`、読む所 `parse.rs:34`、struct literal の test なし）。既存の関数の signature は変えない。新しい関数は cfg(test) の中にあり、呼出し元は test だけ。wire（command・DTO・bindings）は変えない。

他 lane と重なる file: `src-tauri/src/io/z004_parser.rs` と `docs/function-design/23-io-z004-parser.md`（本 lane は 23 を編集しない）は `agent/plu-clear` が触る可能性がある（未確認）。本 lane は `ParseResult` に field を 1 つ足し、メタ行の抽出の関数を足すだけで、データ行の分類（`parse_data_line`・`normalize_jan`、SPEC-Z4A-D8）には触らない。`docs/decision-log.md`（末尾に D-109 だけ）・`docs/backlog.md`（本 lane の entry だけ）・`docs/function-design/90-traceability.md`（生成物）は全 lane と共有。#145 が触る `24-io-csv-import-repo.md`・`34-biz-sales-service.md`・`56-ui-daily-sales.md` は本 lane が引用するだけで編集しない（`sales_repo.rs` は S2 で触るが、#145 の `OfficialDailyReportRow` 周辺〈#145 の merge 後の `sales_repo.rs:939`・`:1036-1258`〉には触れず、新しい関数は既存の test の module〈`:1410`〉の前の cfg(test) の 1 群に足す）。

| 共有 file | 本 lane の範囲 | 他 lane の範囲 |
|---|---|---|
| `src-tauri/src/db/sales_repo.rs` | `pos_import_sources` と active import の識別メタの新関数（`#[cfg(test)] pub(crate) mod time_evidence`、`:1410` の前に追加） | #145: 日報の公式の行の aggregate |
| `src-tauri/src/io/z004_parser.rs` | `ParseResult` の field とメタ行の抽出 | `agent/plu-clear`（触るなら）: データ行の分類 |
| `docs/decision-log.md` | 末尾の D-109 | `agent/npm-audit-1006` = D-108、C（`agent/plu-clear`）= D-110、A（`agent/sd-direct-read`）= D-111 |
| `docs/backlog.md` | STK-1 / STK-2、design lane の項、診断 2 本、0/0 行、③〜⑤ | 各 lane の項 |

## Non-scope

- `docs/Plans.md`（D-097）、`docs/project-memory.md`（SD の行は lane A、本 lane は触らない）、ADR 2 本の本文と Status（owner 決定 Q2 = (A): 時点証拠 ADR は proposed のまま進め、⑤ で停止 ADR を superseded にするのと同時に accepted にする）、`docs/function-design/` の 20 / 21 / 23 / 24 / 32 / 35 の proposed 節（② の範囲では既存で足りる。Design Readiness）。
- 通常起動の migration の registry 登録、`update_stock_quantity` と `ProductUpdates.stock_quantity`（`product_repo.rs:166`・`:1040`）の置換、旧 writer の kind 対応、`legacy_*` と停止の撤去、command / bindings / UI（⑤）。
- 計数 context・保存・確定・独立再実測・flag の解消（③）、受領と分類の preview / commit への配線・flag の作成・相殺の判定・取消補償・legacy の取消の保留・準備照会・`ej_unverified`・在庫連動の有効化の拒否（④）。
- EJ の日次取込みと相殺の判定、精算系列・番号 reset・同一性の比較方法の見直し（design lane / EJ の日次取込みの lane）。
- 実機・実データでの確認（本 lane は合成データだけ）。

## Acceptance Criteria

- AC1（schema の内容）: `apply_time_evidence_schema` を当てた試験 DB で、master / tracking / pos / transaction の proposed 節の列・表・CHECK・UNIQUE・FK がある（`PRAGMA table_info` / `sqlite_master` を test が照合する）。`stocktake_items.observation_kind` を省いた INSERT と `stocktakes.reconciliation_version` を省いた INSERT が失敗する（2 列とも DEFAULT なし）。再構築で作り直した index と新しい index がある。`products.stock_revision` が i64 上限から増やせず、REAL にならない（P3）。test 名は Matrix T1〜T3。
- AC2（未配線）: 通常の `migrate()` の後の DB に新しい表・列が無く、`schema_versions` の最大が `app_max_version()` と同じ（test は版の literal を持たない。起票時の baseline の記録: `rg -c 'version: [0-9]+,' src-tauri/src/db/migration.rs` が `7`、`def86e19` で実行）。T4。
- AC3（旧明細の分類と保全）: v7 の上で作った旧明細（両 NULL / 0・0・時刻 NULL / 数量と時刻あり / 矛盾形〈数量 NULL・時刻あり〉/ 数量あり・時刻 NULL で 0・0 でない 3 形〈actual 3・system 5、actual 0・system 5、actual 3・system 3〉）が uncounted / auto_filled / legacy / legacy / legacy になり、旧 header は `reconciliation_version=0`、`stocktake_legacy_movement_ceiling` が当てる前の movement 最大 ID（空なら 0）、在庫・movement・売上・取込みの行が当てる前後で一致する。途中で失敗させると新しい列・表・key・`schema_versions` の行が残らない。T5〜T7。
- AC4（数量と版）: `update_stock_quantity_with_revision` が同じ数量でも版を 1 進め、対象なしは false で版を変えず、上限では数量も版も書かない。`bump_stock_revision` は増分後の版を返す。呼出し側の TX を rollback すると両方戻る。T8〜T10。
- AC5（受領）: 同じ hash の受領は最初の ID と時刻を返し、別 hash は新しい ID。業務 TX を rollback しても受領は残る。最大 source ID は空で 0。T11・T12（`src-tauri/src/biz/csv_import_service/tests/time_evidence_tests.rs`）。
- AC6（精算の同一性）: 判定は書込みの無い core と、自分で TX を開く証拠の関数に分け、caller の TX の中で core を呼んで拒否すると caller の書込みが戻り証拠だけが残る（T14b）。 同じ帳票種別で machine_no / settlement_no が一致する別 hash は、未取込み・取消済みの source が相手でも業務 write の前に拒否し（`IdentityGuardError::Rejected(IdentityConflict)`）、拒否の初回の code と時刻を独立 TX で残す（2 回目で上書きしない）。NULL どうし・帳票種別の違いは衝突にしない。同じ精算日の active import があり、取込み対象か比較先（複数なら全件）のどれかで machine_no / settlement_no が欠けると拒否する（取込み対象の両方欠け・片方欠け、比較先の片方欠け、比較先の source なし、2 件目だけの欠け）。同日の active が無いとき、または両方のメタが揃い settlement_no が違うときはこの追加の拒否はしない。T13〜T16。
- AC7（前後の分類）: 非連動 = 売上のみ、NeverObserved = After、LegacyObserved = Unknown（理由 `legacy_basis`）、`source.id <= source_cursor` = Before、`source.id > source_cursor` = Unknown（数量 ≠ 0 は `sale_order_unknown`、数量 0・金額 ≠ 0 は `offset_lines_present`、0/0 は相殺の判定が要る印）。最新の有効な観測は measured の明細と recount を合わせた `observation_revision` の最大で選び、版を持つ観測があれば legacy より優先し、auto_filled はそれより前の有効な観測を隠さない。分類の入力に時刻・精算日時が無い（関数の signature）。共有 JAN の行は全在庫連動候補が Before のときだけ commit 可、1 つでも Before でなければ（NeverObserved の After を含む）file 全体が held。T17〜T22。
- AC8（在庫判定の行）: 正常 JAN の 0/0 行が在庫判定の行の集合に全候補付きで入り、売上の行の集合（`parse_and_validate` の `matched_rows`）には入らない。既存の `test_parse_and_validate_req401_empty_records_excluded` は変えずに PASS。T23。
- AC9（Z004 の識別メタ）: 合成の layout A で machine_no（「マシンNo.」）/ settlement_no（「精算回数」）/ report_kind（「ファイル」）を文字列のまま（先頭の 0 を保って）、settled_at を `YYYY-MM-DDTHH:MM` で返し、従来 shape は各項目 `None`。既存の z004 parser の test は変えずに PASS。T24・T25。
- AC10（未配線の機械検査）: `cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings` が exit 0。Writer が一時の変更で production の関数から S2 の新関数 1 つ（例 `sales_repo::time_evidence::upsert_pos_import_source`）を呼ぶと `cargo build` が `E0425` / `E0433` で失敗することを 1 回確かめ、戻して `git diff --exit-code` が exit 0（PR body に command と結果を書く。commit しない）。T26。
- AC11（既存の保護）: `rg -c '#\[ignore' src-tauri/src/biz/csv_import_service/tests/cross_feature_tests.rs` が `2`（baseline `2`）。`cd src-tauri && cargo test --offline --lib cross_feature_tests -- --ignored --nocapture` は今と同じく 2 本 FAIL し、出力に `XFA_TEMPORAL_FAIL` と `XFA_LATE_IMPORT_FAIL` が出る（baseline: 起票時に同 command で exit 101、2 failed）。ignored を含めない `cargo test` は全 PASS。`legacy_stop_tests.rs` は変えずに PASS。
- AC12（docs と生成物）: `bash scripts/doc-consistency-check.sh --target plan` と `bash scripts/doc-consistency-check.sh` が ERROR 0。traceability を再生成し、生成物の検査（L1 full の traceability）が通る。

## Design Readiness

- 引用する設計正本（節まで）: ADR `docs/adr/2026-09-18-stocktake-time-evidence.md` の SPEC-STK-TIME-D1（版の規則、:46-54）・D2（:62-68）・D3（:72-86）・D4（判定表 :96-102、判定不能の理由 :111-116、共有 JAN :140-141）・D8（:261-324）と「㉘ への引継ぎ」（:388-414）。停止 ADR SPEC-STOP-D3・D6。`docs/function-design/32-biz-csv-import-service.md` の proposed 節「一つの分類関数とcommit」（:27-52）。`35-biz-stocktake-service.md` の proposed 節（:3-48）。`20-io-product-repo.md`・`21-io-inventory-repo.md`・`23-io-z004-parser.md`・`24-io-csv-import-repo.md` の proposed 節。`docs/db-design/master-tables.md`・`tracking-system-tables.md`（「移行と保存TX」を含む）・`pos-tables.md`・`transaction-tables.md` の proposed 節。
- 必要な設計成果物: BIZ / IO / repo の振舞い = existing sufficient（上の proposed 節）。DB = existing sufficient + updated in this PR（`pos-tables.md` に backfill しない 1 文）。durable な判断 = updated in this PR（D-109: lane の選び方、未配線の入れ方、機械検査、backfill しない）。command / DTO / UI = 該当なし（⑤）。
- plan にしかない durable な判断の昇格先: D-109。
- 前提・制約と、延期した design gap の follow-up: 相殺の判定（0/0 行）・EJ の interface・準備照会・在庫連動の有効化の拒否は ④、計数と補正は ③、切替は ⑤（`docs/backlog.md` の ③〜⑤ の申し送り）。精算系列・同一性の比較方法は design lane が見直し得るが、ADR D3 の保守的な拒否のまま作り、見直しは追加の変更になる（ADR :20、D-109 の Revisit）。
- 絶対保証の自己点検: 「通常の起動で schema が変わらない」は registry に登録しないことと AC2 の test で守る。例外は test の helper だけ。「production から新関数を呼べない」は `#[cfg(test)]` の隔離で、呼べば compile error（AC10、P1 の B1〜B2）。例外: cfg(test) の中の code が `test_support` や dev-dependencies を参照しても ② では検出できない（P1 の B3。⑤ で cfg を外すときの compile で露見する）ので Review Focus で確かめる。
- 判定: ready（② の実装に設計の問いは無い。owner の決定 Q2・Q3〈2026-10-06〉は ② の Scope を変えず、Q1 は本 lane が依らない）。

## Registration / Generation Obligations

| 変更対象 | 本 lane の義務 |
|---|---|
| Tauri command | 該当なし（⑤） |
| function-design doc 新設 | 該当なし |
| source / workflow doc 新設・改名・削除 | 該当なし |
| REQ / coverage の追加 | 新しい test の REQ-205 / REQ-401 の参照で `cargo run --bin generate_traceability` を実行し `docs/function-design/90-traceability.md` を再生成（S6） |
| route / operator 画面 | 該当なし |
| Rust の module 登録（本 lane 固有） | `db/mod.rs` の `#[cfg(test)] mod schema_time_evidence;`、`csv_import_service/mod.rs` の `#[cfg(test)] mod time_evidence;`、`csv_import_service/tests/mod.rs` の `mod time_evidence_tests;`、各 repo file の `#[cfg(test)] pub(crate) mod time_evidence`。`src-tauri/tests/design_compliance_test.rs` の対応表と設計書の code block は ⑤（cfg を外すとき）。migration の registry（`migration.rs` の `migrations()`）には登録しない（D-109 (2)） |

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | 置き換えうる adapter と core の契約は | ARCHITECTURE、23、32 | 当たる。Z004 のメタ行の読み方は IO（CASIO 固有）、同一性・前後の判断は BIZ。IO は番号を解釈せず文字列で渡す（23:5） | なし |
| Fact check / design decision split | 観測した事実とアプリの決定は | project-memory、backlog、ADR | 当たる。Z004 のメタ行の並び（23:130、2026-08-17 実ファイル抽出）は観測済みの事実。精算系列の意味（backlog:26、project-memory:239）は design lane の入力で、本 lane は判断に使わない | design lane（backlog） |
| Lifecycle / retry | 受領・拒否・失敗・再試行は | 32、pos-tables | 当たる。受領は業務 TX と独立で rollback でも残る、同 hash は最初の ID、拒否の証拠は初回だけ（AC5・AC6） | なし |
| Operator workflow | 実際の操作列は | Ordinary Operation | 操作は変わらない（① の停止のまま） | ⑤ |
| Replacement path | 外部が変わったら何を替えるか | 23、D-023 | Z004 のメタ行の抽出だけが CASIO 固有 | なし |
| Data safety / evidence | 実データを commit しないか | Data Safety | 当たる。合成 fixture だけ | なし |
| Reporting / accounting semantics | 売上・在庫・棚卸しを混ぜていないか | 32、tracking | 当たる。在庫判定の行の集合と売上の行の集合を分ける（AC8）。0/0 行は売上も movement も作らない | ④ |
| Manual verification | 自動 test で示せないものは | Test Plan | 無い（通常起動へ配線しない）。L3 は ⑤ | ⑤ |
| 環境・再現性 | 新しい環境依存は | Contract Probe | SQLite の版（bundled、`Cargo.toml:32` の rusqlite 0.31）に依る ALTER・overflow の挙動を test で固定（P2・P3） | なし |

## Boundary / Wire Contract

- producer: IO-02 `parse_z004`（`settlement_metadata`）、BIZ の受領・同一性・分類（crate 内部）、DB の schema 関数（test の helper だけが呼ぶ）。
- consumer: 本 lane の test。③ / ④ の BIZ。wire（command・DTO・bindings）は無い。
- wire type: 該当なし。
- internal type: `SettlementMetadata { machine_no, report_kind, settlement_no, settled_at: Option<String> }`、分類の結果（Before / After / Unknown と理由、売上のみ、held）、source ID と cursor は `i64`（0 = 空集合）。
- precision/range: 版・cursor・ID は非負 `i64`、増分は checked。番号は文字列（先頭の 0 を保つ）。
- round-trip path: 該当なし（DB と BIZ の内部）。
- invalid input: 識別メタの欠け = 同日の active があれば拒否、無ければ通す（AC6）。版の上限 = 書かずにエラー（AC4）。
- compatibility: 通常起動の DB は registry の最大の版のまま（起票時 v7）。`ParseResult` への field の追加は既存の呼出しを壊さない。

## Test Plan

[Test Design Matrix](test-matrices/2026-10-06-stocktake-p1.md)。

- targeted tests: `cd src-tauri && cargo test --offline --lib time_evidence`、`... --lib z004_parser`、`... --lib inventory_repo`、`... --lib sales_repo`、`... --lib stocktake_repo`、`... --lib db::`。
- negative tests: 版の上限、kind を省いた INSERT、同一精算の別 hash、識別メタの欠け 3 形、migration の途中の失敗、共有 JAN の held。
- compatibility checks: 通常の `migrate()` の後に新しい表・列が無い（AC2）。既存の z004 parser・`parse_and_validate`・`legacy_stop_tests`・cross feature の通常 test が変えずに PASS（AC8・AC9・AC11）。
- data safety checks: 合成 fixture だけ（Data Safety）。
- main wiring/integration checks: 本 lane は配線しないことが契約（AC2・AC10）。

## Review Focus

- 通常の起動の DB を変えないこと（registry に登録していない、`test_support` 以外から schema 関数を呼んでいない）と、S1・S2・S4 の新しい関数・型・module が全て `#[cfg(test)]` の中にあり `pub(crate)` であること（P1）。
- cfg(test) の中の production 予定の code（S1・S2・S4 の本体）が `test_support`・dev-dependencies（`tempfile` 等）を参照しないこと（⑤ で cfg を外すときの compile で露見するが、② では機械検査が無い。P1 の B3）。
- 分類の関数が時刻・精算日時を入力に持たず、`source.id <= source_cursor` だけで Before を決めること。LegacyObserved を未実測に落とさないこと。auto_filled が前の観測を隠さないこと。
- 受領と拒否の証拠が業務 TX の rollback で消えないこと。同一性の照合が未取込み・取消済みを含むこと、識別メタの欠けを候補 0 件へ落とさないこと。
- `observation_kind` に恒久の DEFAULT を付けていないこと（table 再構築）。旧明細の分類が `tracking-system-tables.md` の表どおりで、現在の廃番 flag から逆算していないこと。
- 0/0 行が在庫判定の行に入り、売上の行に入らないこと。
- 普通の一日の操作列で、③ / ④ がこの基盤の上に Matrix の「STK-1 / STK-2 の green の条件」を満たせるか（本 lane の関数の形がそれを妨げないか）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| SPEC-STK-TIME-D1（版の不可分な増分、checked、同量でも進める） | ADR :46-54、21 proposed、master-tables proposed | S2 `update_stock_quantity_with_revision` / `bump_stock_revision` | T8〜T10 | 既存 writer の切替は ⑤ |
| SPEC-STK-TIME-D2（受領は hash で一意、最初の ID、業務と独立） | ADR :62-68、pos-tables proposed、24 proposed 1〜2 行 | S2 sales_repo、S4 (a) | T11・T12 | preview への配線は ④ |
| SPEC-STK-TIME-D3（同一精算の別 hash の拒否、識別メタの欠けの拒否、拒否の証拠） | ADR :72-86、32 proposed :40-46、24 proposed 3〜5 行 | S2 sales_repo、S4 (b) | T13〜T16 | commit TX での再検査の配線は ④、準備照会の `settlement_missing` は ④ |
| SPEC-STK-TIME-D4（判定表、理由、共有 JAN の held） | ADR :96-102 / :111-116 / :140-141、32 proposed :29-35 | S4 (c)、S2 stocktake_repo | T17〜T22 | 相殺の判定（0/0）・flag の保存は ④ |
| SPEC-STK-TIME-D4 / IO-02（0/0 行を在庫判定に残す） | 23:9、32 proposed :23、ADR :106 | S4 (d) | T23 | CachedPreview への配線は ④ |
| SPEC-STK-TIME-D3 / IO-02（識別メタの抽出） | 23:5・:130 | S3 | T24・T25 | 画面に出さない |
| SPEC-STK-TIME-D8（schema・旧明細の分類・legacy 上限・失敗で全部戻す・恒久 DEFAULT なし） | ADR :261-324、tracking / pos / master / transaction の proposed 節 | S1 | T1〜T3・T5〜T7 | registry 登録は ⑤ |
| SPEC-STOP-D6 / D-109 (2)（⑤ まで通常起動へ入れない） | 停止 ADR D6、tracking「移行と保存TX」 | S1（registry に登録しない） | T4 | ⑤ |
| D-109 (3)（test だけが呼ぶことの機械検査） | D-109 | S1・S2・S4 を `#[cfg(test)]` の中に置く、全て `pub(crate)` | T26（clippy と一時の compile error の確認） | ⑤ で cfg を外す |
| D-109 (4)（旧 import から受領を backfill しない） | pos-tables proposed（本 PR の 1 文）、32 proposed :15 | S1 | T7 | なし |
| 隣接: 既存の診断と停止 | 停止 ADR D3・D5、cross-feature-verification | 変えない | AC11 | 正規回帰への移行は ⑤ |

adjacent-contract sweep: `update_stock_quantity` の既存 caller 4 か所（Scope S2）と `ProductUpdates.stock_quantity`（`product_repo.rs:166`・`:1040`）は ⑤ の置換対象で本 lane は触らない（除外を Non-scope に明記）。`pos_stock_sync` の共通書込み経路と `pos_sync_disabled_revision` の記録（20 proposed）は ④ / ⑤（列だけ S1 で作る）。

## Contract Probe

- P1 未配線の検査の形: `$TMPDIR` の最小 crate（rustc 1.94.1、実 crate と同じく module に `#[allow(dead_code)]` を付けた `pub mod db` と `mod biz`）で `cargo clippy --offline --all-targets -- -D warnings`。A 系 = 根だけに `#[cfg_attr(not(test), expect(dead_code))]`、新しい repo 関数は `pub(crate)`: A0 そのまま -> exit 0（未使用の `pub(crate)` も module の allow で報告なし）。A1 production の関数から根の下の repo 関数を直接呼ぶ -> exit 0（検出しない）。A2 production から根を呼ぶ -> `this lint expectation is unfulfilled` で exit 101（根だけ検出）。B 系 = 新しいものを `#[cfg(test)] pub(crate) mod time_evidence { … }` に置く: B0 そのまま -> exit 0（test は通る）。B1 production から `db::time_evidence::…` を呼ぶ -> `E0433` で exit 101。同じ module の cfg(test) の関数を名前で呼ぶ形は `E0425`。B2 `use crate::db::time_evidence::…` -> `E0432` で exit 101。B3 cfg(test) の関数が cfg(test) の `test_support` を呼ぶ -> clippy・`cargo test` とも exit 0（検出しない、Review Focus）。よって B の形を採る（AC10）。
- P2 SQLite は既存行のある表に NOT NULL で DEFAULT の無い列を ALTER で足せない: Python の sqlite3（SQLite 3.51.3）で行のある表に `ALTER TABLE t ADD COLUMN k TEXT NOT NULL` -> `Cannot add a NOT NULL column with default value NULL`。空の表では成功する（bundled の SQLite 3.45.0〈`Cargo.lock` の libsqlite3-sys 0.28.0〉で Plan Review の Codex が実測）。`observation_kind` は table 再構築で埋める（tracking「移行と保存TX」の 2 項目め）。空の試験 DB では ALTER の書き方の誤りが出ないので、旧明細の行がある T5〜T7 が pin する。
- P3 整数の上限を越える加算: 同じ環境で CHECK の無い INTEGER 列に `v=v+1`（v = i64 上限）-> `9.223372036854776e+18`（REAL）に化ける。`CHECK(typeof(v)='integer' AND v>=0)` があれば `CHECK constraint failed`。版の増分は Rust の checked 演算で上限を先に検査し、CHECK を併せて置く（T2・T9）。
- P4 Z004 layout A のメタ行: 既存の観測（`23-io-z004-parser.md:130`、2026-08-17 の実ファイルの機械抽出: マシンNo. / ファイル / モード / 精算回数 / 日付 / 時刻）。本 lane は合成のメタ行で test し、実ファイルは使わない。新しい外部の probe は要らない。
- P5 Z004 layout A のメタの値の形（local の持ち帰りデータを読むだけ、値と件数は tracked に写さない）: 手元の layout A の Z004 全本で「ファイル」行の値は 1 種類で帳票名を含まず、「モード」行も全本で同じ 1 つの値、時刻は全本 `HH:MM`（`.local/field-data/recount/mc-z/shape_z.out` の S1 と、2026-10-06 の起草役の数え直し）。よって report_kind を「ファイル」行にしても同じ精算の Z004 どうしで値が揃い、D3 の衝突の検査を素通りしない。「モード」は精算と点検の区別に使われうる値だが全本同じで、抽出しない。settled_at の `YYYY-MM-DDTHH:MM` は時刻行の形に合う。
- P6 caller の TX の中から `&mut DbConnection` を受ける関数を呼べるか: `$TMPDIR/idprobe` の最小 crate（rusqlite 0.31.0、相談役が組み、起草役が再実行）。`bad` = caller の `Transaction` から `check(&mut tx)` を呼ぶ形 -> `cargo build --offline` が `E0596`（`cannot borrow data in dereference of Transaction<'_> as mutable`）で exit 101。`good` = 書込みの無い core（`&Connection`）・自分で TX を開く証拠（`&mut Connection`、`UPDATE … WHERE … IS NULL` の 1 文）・wrapper に分け、caller の TX で 1 行書いた後に core が拒否 → `tx.rollback()` → 証拠 → Err、の形 -> `cargo run --offline` が exit 0（caller の行が残らず証拠が 1 行、2 回目は上書きなし、通る場合は同じ TX の書込みが commit される）。S4 (b) の 3 分割と T14b の根拠。
- Plan Gate の前に要る probe: なし。

## Data Safety

- 実 POS の Z004 / EJ、店の DB、backup、log を commit しない。fixture は合成（JAN は `29000000…` の店内コード形、金額・名称は合成）。
- local-only: `.local/` の持ち帰りデータの集計は読むだけで、件数・実値を tracked に写さない。
- synthetic-only: 本 lane の全 test。

## owner の判断事項

owner の決定（2026-10-06、確定）。本 lane の Scope は変わらない。

- Q1 本 lane の外: design lane「実測と POS 系列の対応を取得・保存する」の着手条件 (4)（棚卸し → アプリ終了 → OS 再起動 → 翌日の遅延取込み）を外すか。owner は今は決めていない。本 lane は依らない。design lane の起票時に owner へ諮る（`docs/backlog.md` の同項）。
- Q2 決定 (A)（owner 2026-10-06）: 時点証拠 ADR（`docs/adr/2026-09-18-stocktake-time-evidence.md`、Status proposed）は ② 〜 ④ の間 proposed のまま進め、⑤ で停止 ADR を superseded にするのと同時に accepted にする。本 lane は ADR を編集しない（D-109 の Compatibility）。
- Q3 決定 (i)（owner 2026-10-06）: STK-2 の診断（`XFA_LATE_IMPORT_FAIL`）は、⑤ の時点で「計数を始めた後に受領した資料は、要再確認のあと数え直しで正しい数（例の 8）へ戻る」を green とする。数え直しなしでの一致は求めない（Matrix の G2b）。

## Implementation Results

- 結果: 時点証拠の schema（試験 DB の helper でだけ当てる）・資料の受領・精算の同一性の検査（書込みの無い判定の core・自分で TX を開く証拠・その 2 つを使う wrapper）・前後の分類・在庫判定の行の集合（0/0 行を残す）を、通常の起動へ配線せずに追加した。新しい関数・型・module は `#[cfg(test)]` の中にあり、production から呼ぶと compile error になることを一時の変更で確かめた（AC10）。Z004 の parser は layout A のメタ行から識別メタ（machine_no・report_kind・settlement_no・settled_at）を文字列のまま抽出し、「ファイル」行が無ければ番号も None にする（settled_at は返す）。通常の起動の DB schema と既存の関数の signature は変えていない。
- 既存の保護: 診断 2 本（`XFA_TEMPORAL_FAIL` / `XFA_LATE_IMPORT_FAIL`）は `#[ignore]` のまま FAIL で、green にするのは ③〜⑤（Matrix の「STK-1 / STK-2 の green の条件」）。既存 test は変えていない。
- review・CI・merge（closeout、2026-10-07）: Final Review broad は互いに独立の 2 本（Fable 5.1 = approve、Codex GPT-6.1 Sol = reject〈P2 1〉）で、是正の後の closure（Fable 5.1）が approve。Ready の後の hosted run の `Merge gate` は pass で、owner の承認（2026-10-07）で helper 経由の squash merge。AC1〜AC12 と mutant の実測値は PR の body が持つ。PR: [#148](https://github.com/higekiri-w/inventory-system-desktop/pull/148)
- packet との食い違い: なし。

## Review Response

round 1（`f8af8681`）: Claude 側 fresh Opus 5.5 = reject（P1 0 / P2 4 / P3 5）、Codex GPT-6.1 Sol（発注 229）= reject（P1 1 / P2 1 / P3 2）。相談役 Fable 5.1 が是正案の反例を探し、P1 の直し方 4 案を最小 crate で比べた。裁定: P1（根だけの expect で根の下の新関数の production 呼出しを止められない。Opus P2-1 と同じ根）accept、②〜④ の新しい関数・型・module を `#[cfg(test)]` で隔離し ⑤ で外す形に D-109 (3) と AC10 / T26 を改めた（production からの呼出しは compile error）。P2 4 件 accept、T5・T13・T15・T16・T19・T20・T22 に相談役が見つけた残りの mutant を落とす形を足した。P3 全件 accept。

round 2（`8c6a0385`）: Claude 側 fresh Opus 5.5 = approve（P3 6）、Codex GPT-6.1 Sol（発注 231）= reject（P2 1）。相談役 Fable 5.1 が是正案の反例を探し、最小 crate で確かめた。裁定: P2（同一性の検査が自分で業務 TX を開き caller の TX の中から呼べない〈E0596〉。Opus P3-1 と同じ根）accept、書込みの無い判定の core・自分で TX を開く証拠の関数・その 2 つを使う wrapper に分け、④ は caller の TX の中で core を呼ぶ形を S4 (b) に書いた（T14b）。P3 6 件 accept（比較先の自己除外の述語、衝突は 3 つの識別が揃うときだけ、拒否 code の優先、T7 の COMMIT での注入、index の再作成と照合、Measured の cursor を型で必須に、`reconciliation_version` は DEFAULT 無しで `stocktakes` も再構築、生の SQL の fixture、T9 の error の名指し）。相談役の追加の反例 3 件 accept。

round 3（上限、`fe691b4b`）: Claude 側 fresh Opus 5.5 = approve（P3 3）、Codex GPT-6.1 Sol（発注 233）= approve（P3 1）。P1 / P2 = 0 で Plan Gate を通過。P3 は Coordinator が直した: report_kind が無く番号だけがある file（parser は可変のメタを受理し各項目を独立に抽出する、Codex の実測）が衝突の検査とメタ不足の検査の両方をすり抜ける筋を、S3 が「report_kind が None なら番号も None」を強制する形で塞いだ（T24 に 1 形、2 本の P3 の共通の根）／T1 に再構築後の FK と `foreign_key_check` の 0 行を足した／S4 (d) に候補 0 件の行を在庫判定の行に入れないことを足した。

Closeout（2026-10-07）: Final Review broad（互いに独立の 2 本）は Fable 5.1 = approve（P3 4）、Codex GPT-6.1 Sol = reject（P2 1 = T20 で分類する source の ID が実在せず、時刻を入力にする誤実装を検出できない。P3 1）。相談役が追加の穴（同一性の照合の SQL が取込み対象より先に受領した source だけと照合する誤りを T13 が検出しない）を見つけた。是正は test（T20 の source を実在させ時刻を前・後・無しに動かす、T13 に先に受領した source を後から検査する形）と、parser が「ファイル」行の無いときも settled_at を保持すること、`receive_source` の doc comment。closure（Fable 5.1）は approve（前回の 5 件 closed、新しい P1 / P2 = 0、P3 1）。helper に broad 2 本（pending・fail）と closure（pass）を record 済み。Ready の後の hosted run の `Merge gate` は pass で、owner の承認で helper 経由の squash merge（2026-10-07）。Final の P3 は Matrix の T20・T13 の行に反映し、`receive_source` の doc comment が行番号で指している件と kind と証拠の列の組の CHECK は backlog の ③〜⑤ の申し送りへ送った。
