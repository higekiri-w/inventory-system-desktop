## MNT-02: 操作ログ管理

### 72.1 モジュール構成

```
src-tauri/src/
  mnt/
    mod.rs           -- pub mod log_manager（既存宣言済み）
    log_manager.rs   -- ログクリーンアップ（本セクション）
  db/
    system_repo.rs   -- list_operation_logs, delete_old_logs を追加（§2.8拡張）
  lib.rs             -- setup hook に cleanup_old_logs 呼び出しを追加
```

---

### 72.2 依存クレート

追加なし。`chrono`（既存依存）を使用。

---

### 72.3 型定義

なし。system_repo の関数を組み合わせるオーケストレーション層。

---

### 72.4 cleanup_old_logs

**関数要求**: 操作ログの自動削除を実行する。1日1回のみ。アプリ起動時にsetup hookから呼ばれる

**シグネチャ**:
```
fn cleanup_old_logs(conn: &DbConnection) -> Result<(), DbError>
```

**処理ステップ**:
1. `system_repo::get_setting(conn, "log_last_cleanup_date")` で最終クリーンアップ日を取得
   - `None` → 初回実行。ステップ3に進む
   - `Some(date_str)` → ステップ2に進む
2. `date_str` を `chrono::NaiveDate` にパース。今日の日付と比較
   - 同日 → 何もせず `Ok(())` を返す（1日1回制限）
   - 異なる日 → ステップ3に進む
   - パース失敗 → ステップ3に進む（不正な値はリセット扱い）
3. `system_repo::get_setting(conn, "log_retention_days")` で保持日数を取得（MNT-02-D1）
   - `None`（設定の行が無い）→ 既定 3000 日を使う
   - `Some(days_str)` で 1 以上の整数（`u32`）→ その値を使う
   - `Some(days_str)` で整数として読めない、または 0 → 削除も日付の更新もせず、`tracing::warn!` を記録して `Ok(())` を返す（保持日数を確定できないので消さない）
4. `system_repo::delete_old_logs(conn, retention_days)` を実行。削除件数を受け取る
   - 今日から保持日数を引いた日が日付の範囲の外（`chrono` で引けない）なら、それより前の記録は無いので何も消さず 0 件を返す（panic しない）
5. 削除件数 > 0 の場合:
   - `system_repo::insert_operation_log` で記録:
     - `operation_type`: `"log_cleanup"`
     - `summary`: `"操作ログを{N}件削除しました（{cutoff_date}以前）"`
     - `detail_json`: `None`
6. `system_repo::upsert_setting(conn, "log_last_cleanup_date", &today_str)` で日付を更新
7. `Ok(())` を返す

**エラーハンドリング**:
- DB操作失敗 → `DbError` をそのまま返す
- 呼び出し元（lib.rs）で `tracing::warn!` して続行する（ログクリーンアップ失敗はアプリ起動を止めない）

**MNT-02-D1: 保持日数の既定は 3000 日（帳簿書類の 7 年に足りる長さ）、保持日数を確定できないときは消さない**（[decision-log](../decision-log.md) D-136）

- 決定: `log_retention_days` の既定（DB を作るときの初期データ `schema_v1.rs` と、設定の行が無いときの値）は 3000 日。設定の値が整数として読めないか 0 のときは、既定に置き換えずに削除を飛ばす。日付の範囲の外になる大きな値は、消す対象が無いとして 0 件にする。掃除の仕組み（起動時に 1 日 1 回、`created_at` が今日から保持日数を引いた日の 0 時より前の行を物理削除し、件数を `log_cleanup` で記録する）は変えない。この仕組みでは、保持日数が N のとき、ある日の記録はその日から N 日後の日まで残る
- Why: 単位の変更・整合性の補正・migration の求め直しの差・共有 JAN の違反の知らせは操作ログにだけ残り、店の伝票の保存は 7 年（[project-memory.md](../project-memory.md) の Store Premises Facts の「Slip retention duty」の行）。owner の決定（repo 外の回答台帳 TD-348）で 7 年程度にし、同じ取引の伝票が残る間は操作ログも残る長さにした（TD-418）。帳簿書類の 7 年の数え方と、それに要る日数（個人・法人）は [decision-log](../decision-log.md) D-136 の Why (1) が持ち、3000 日はそれに足りる切りのよい値。読めない値を既定へ置き換えて消すと、設定が壊れたときに残すべき記録を消す（backup の保持日数の [71](71-mnt-backup.md) MNT-01-D3 と同じ理由。消さない側の失敗は記録が溜まるだけで、次に読めたときに戻る）。0 を受けると、その日より前の全ての記録を消す
- Rejected alternatives: 消さない（掃除を外す。owner は 7 年程度と決めた。DB と backup の大きさに上限が無くなる）/ 2557 日（取引の日から 7 暦年。年の初めの取引の記録が、同じ取引の伝票より 1 年余り早く消える）/ 2555 日（365 × 7。2557 日より更に短い）/ 既存の DB の値を migration で書き換える（本番の DB は無く、開発・試験 DB はリリースの前まで作り直してよい。D-123 (1)。migration を足すと前の版のアプリがその DB を開けなくなる、22 §3.2 MNT-03-D11）/ 読めない値を既定 3000 日で掃除する（既定が設定より短いと消しすぎる）
- 大きさ: 合成の 511400 行（1 日 200 行 × 2557 日、summary は日本語 40 字、detail_json は約 300 byte）で DB の file は 210006016 byte、期間の filter の件数と先頭 50 行の取得はそれぞれ約 0.07 秒（2026-10-23、開発用の PC の WSL、sqlite3 3.51.3。手順は `docs/archive/plans/2026-10-23-oplog-retention.md` の Contract Probe）。3000 日分は行数の比で約 246389538 byte（未実測）。店の 1 日の行数は未実測
- 見直し契機: 店の 1 日の行数が分かり、DB か backup の大きさが運用に障ったとき。保持日数を画面で変えたい要望が出たとき（[69](69-ui-threshold-settings.md) UI-11a-D1 の Revisit）。店が法人と分かったとき（D-136 の Revisit）

---

### 72.5 lib.rs 起動シーケンスの変更

**追加箇所**: `src-tauri/src/lib.rs` の `run` の setup の手順 5。DB の初期化（手順 3-4 の `prepare_database`）の後、自動バックアップの確認（手順 6 の `run_startup_auto_backup`）の前（2026-10-24 に `run` の setup で確かめた）。手順 5 のコメント `// 5. 操作ログ自動削除` は、起動の test `test_startup_spec_sfv_t3_database_init_failure_has_operator_message`（同じ file の `mod bindings_generation_tests`）が DB の初期化の失敗の分岐の終わりの目印として文字列で探すので、コメントを変えるときはこの test も合わせる

```
// 5. 操作ログ自動削除（DB初期化後に実行。失敗してもアプリ起動は続行）
if let Err(e) = mnt::log_manager::cleanup_old_logs(&conn) {
    tracing::warn!(error = %e, "操作ログの自動削除に失敗");
}
```

---

### 72.6 テスト方針

| テスト名 | 検証内容 |
|---------|---------|
| `test_cleanup_old_logs_req902_first_run` | `log_last_cleanup_date`キー未存在時にクリーンアップ実行 |
| `test_cleanup_old_logs_req902_same_day_skip` | 同日2回目の呼び出しでスキップ |
| `test_cleanup_old_logs_req902_deletes_expired` | 初期データの保持日数（3000 日）より前のログが削除される（MNT-02-D1） |
| `test_cleanup_old_logs_req902_keeps_recent` | 初期データの保持日数の内のログが保持される（MNT-02-D1） |
| `test_cleanup_old_logs_req902_records_cleanup` | 削除後にoperation_type='log_cleanup'のログが記録される |
| `test_cleanup_old_logs_req902_updates_date` | 実行後にlog_last_cleanup_dateが今日に更新される |

MNT-02-D1 の test（保持の lane で足す。名前は実装の PR で決め、Test Design Matrix `docs/archive/plans/test-matrices/2026-10-23-oplog-retention.md` の行に対応させる）: 新しい DB の `log_retention_days` が `3000`、設定の行が無いときは 3000 日で消す、読めない値・0 では何も消さず `log_last_cleanup_date` も変えない、日付の範囲の外の値で panic せず何も消さず、`log_last_cleanup_date` は今日にする（手順 4 の 0 件から手順 6 へ進む）。
