# Plan Packet: backup / migration failure contract 実装 PR2（設定読取 / migration rollback の失敗処理）

## Workflow State

- Phase: ready-hosted-final
- Risk: R4
- Execution Mode: fable-window
- Plan Commit: fe63252
- Amendments: 1（`8801e9d` Double Audit 2 pass 起因の Ledger §71.5 2 行追加 + Matrix D5 訂正 / D5a / D5b 新設）
- Coordinator: Fable 5（本 session）
- Writer: Codex（実装・テスト。発注 cwd は public-writer clone `/home/kosei/Projects/inventory-system-public` に pin）
- Plan Reviewer: Sonnet subagent（独立 context）
- Final Reviewer: Fable inline（Contract Audit 1 pass）+ Codex 独立 context（2 pass。定義は Acceptance Criteria の Double Audit 項）
- Reviewed Content HEAD: d07299c
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required
- Human Gate: (1) R4 explicit approval = 済み（2026-07-19、介入 1 回目。Windows native L3 不要判断も同承認に同梱）(2) Ready 承認 = **済み**（2026-07-19、介入 3 回目 / 予算 3 回）。pending = Ready 化の実操作（owner 自身）+ hosted final green 確認後の merge
- State Narrative（append-only）: 本 packet の plan-first commit で `kickoff -> spec-check -> plan-draft -> plan-gate` を実体化。evidence: spec-check = Risk R4 の分類記録（本 packet Risk 節、[adjudication](../../research/audit-2026-07/adjudication.md) の是正順 1+2 R4 付与を継承）/ plan-draft への skip = Design Readiness が既存設計正本（PR #14 で確定済みの 71 §71.8/71.9 + 22 §3.2 + D-048）を十分と引用（許可された唯一の skip 経路）/ plan-gate = packet + Test Design Matrix complete and committed（本 commit）。
- State Narrative 追記（append-only、2026-07-19）: state-only commit で隣接 forward 2 遷移 `plan-gate -> plan-approved -> implementing` を実体化。evidence: plan-approved = Plan Gate 独立 3 round 収束（round 3 で resolved 3/3 + 新規 P1/P2 = 0、Review Response 参照）+ `Plan Commit` = plan-first commit `fe63252` 記入（実装 commit は本遷移時点でゼロであり plan-first commit が全実装 commit に先行する）/ implementing = owner の R4 explicit approval（2026-07-19、介入 1 回目 / 予算 3 回、Windows native L3 不要判断を同梱承認）。
- State Narrative 追記（append-only、2026-07-19）: state-only commit で隣接 forward 3 遷移 `implementing -> local-verified -> independent-review -> human-confirm` を実体化。evidence: local-verified = content HEAD `d07299c` での `local-ci full` PASS / END_TREE_STATE=CLEAN / MERGE_EVIDENCE_VALID=true（`.local/ci-evidence/`、SHA 正本は PR body）/ independent-review = Double Audit 両 pass 完了（1 pass = Fable inline blocker なし、2 pass = Codex 独立 fresh context P1×1 + P2×2 検出 → 是正 `5479d06` + gated amendment `8801e9d` → Coordinator が独立 reviewer 指定 oracle への実 mutation 3/3 red で closure、Review Response 参照）/ human-confirm = findings 全裁定済み P1/P2 = 0、`Reviewed Content HEAD` = `d07299c` 記入、Findings Freeze 発効。
- State Narrative 追記（append-only、2026-07-19）: state-only commit で `human-confirm -> ready-hosted-final` を実体化。evidence: owner の Ready 承認（2026-07-19、介入 3 回目 / 予算 3 回）。本 commit が最終 tracked HEAD であり、この exact HEAD で L1 full を再実行して PR body を最終 refresh、Ready 化の実操作は owner が行う（以降 tracked commit なしで PR HEAD = PR body final L1 SHA = hosted run headSha の三点一致を merge gate とする）。

## Owner Effort Budget

- 介入回数上限: 3（内訳: (1) R4 発注承認 (2) Codex 実行 relay (3) Ready 承認 + merge。PR1 の 4 から L3 実機確認が不要になった分 -1）
- 実働時間上限: 30分
- relay 往復上限: 3（既定 2 から調整。理由: R4 の Double Audit 2 pass が是正 round を要求した場合の +1 を PR1/PR #15 の実績から設計上想定に含める。scope 拡張ではない）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R4

Reason:
[adjudication](../../research/audit-2026-07/adjudication.md) が是正順 1+2 へ R4 を付与しており、本 PR はその destructive data lifecycle の残り半分 — バックアップファイルの削除（cleanup）を駆動する入力の確定条件、削除・cleanup の対象ディレクトリを決める設定読取、schema migration の transaction 巻き戻し — の実装挙動を実際に変更する。cleanup は実ファイル削除を含む破壊的操作であり、migration は起動時の DB 構造変更経路そのもの。R4 必須物 = R3 必須物 + explicit human approval（発注前）+ rollback / recovery notes（本 packet Data Safety 節）+ Double Audit（2 pass waive 禁止）。

## Goal

Goal Invariant:

### 最小完了条件

- MNT-01-D2 / MNT-01-D3 / MNT-03-D1 の 3 契約（[71-mnt-backup.md](../../function-design/71-mnt-backup.md) §71.8/§71.9 / [22-mnt-migration.md](../../function-design/22-mnt-migration.md) §3.2 が正本）が実装され、**意味的完了条件「破壊的操作（バックアップ削除・migration 巻き戻し後の状態確定）は入力と前提状態を確定できた場合のみ実行され、確定できない失敗は既定値・成功へ変換されず記録付きで安全側（skip / 構造化エラー）に倒れる」**を、失敗注入 + 実 mutation 注入テスト（[Test Design Matrix](test-matrices/2026-07-18-backup-migration-failure-contract-impl-pr2.md)）で検証済みの状態にする。

### 失敗定義

- いずれかの契約が実装から漏れる、または実装されたがテストが意味的完了条件への感度を持たない（mutation を注入しても green のまま = tautological、Matrix X1 で検出）。
- destructive fallback が残存する（DB error / parse 失敗が既定保持日数 3 日に潰れて削除が走る、DB error が既定 backup ディレクトリに潰れる、ROLLBACK / COMMIT / FK 復元の失敗が無記録で握りつぶされる）。
- 既存の正しい挙動（未設定時の既定 fallback、cleanup の正常削除、migration v1→v4 の正常適用、既存テスト）を壊す。

### 非目的

- MNT-02（操作ログ自動削除）や check_auto_backup の判定手順自体の変更。
- 順8（P3-4 利用者向け error 表示統一）。本 PR は既存の error 変換規約（internal error）の範囲で伝搬させるのみで、新しい wire 識別子・利用者文言を追加しない。
- 設定値の書込み時 validation（MNT-01-D3 の見直し契機として設計書に記録済み、本 PR では実装しない）。
- migration を起動時以外から呼ぶ経路の追加（MNT-03-D1 の見直し契機）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

- `src-tauri/src/mnt/backup.rs`:
  - `resolve_backup_dir`（現行 `backup.rs:120-127`）の Result 化（MNT-01-D2）: `get_setting` の DB error を `Err(DbError)` で伝搬し、既定 `app_data/backups` への fallback は「未設定（None）または空文字」の場合に限定する。
  - `run_cleanup`（現行 `backup.rs:348-357`）の保持日数確定条件（MNT-01-D3）: `backup_retention_days` は (a) 読取成功かつ数値 parse 成功、または (b) 設定行不存在（未設定 = 既定 3 日）のみ確定。DB error・parse 失敗は cleanup を skip して `tracing::warn!` を記録し、削除を実行しない。バックアップ作成の成否には影響させない。
- `src-tauri/src/lib.rs` 起動シーケンス（現行 `lib.rs:629` 付近）: `resolve_backup_dir` の `Err` → `tracing::warn!` + 自動バックアップチェック skip + 起動継続（71 §71.9 のコード例どおり）。
- `src-tauri/src/cmd/settings_cmd.rs` `get_backup_dir`（現行 `settings_cmd.rs:54-60`）: `resolve_backup_dir` の `Err` を既存の error 変換規約どおり internal error として伝搬（呼び出し 4 箇所 `settings_cmd.rs:222/239/257/272` は既存の `?` 伝搬のまま）。
- `src-tauri/src/db/migration.rs` / `db/schema_v2.rs` / `db/schema_v3.rs`（MNT-03-D1）:
  - ROLLBACK 失敗の記録 + 併合を共通ヘルパーで実装し、全 ROLLBACK 箇所（`migration.rs:93,104` / `schema_v2.rs:70,84,93,105` / `schema_v3.rs:25,36` の 8 箇所）へ適用。個別再実装をしない。ヘルパーの置き場所は `db/` 配下（`mnt/migration.rs` は実質空モジュールのため命名衝突に注意し、`db::migration` 側へ寄せる）。
  - COMMIT 失敗（`migration.rs:111-112` / `schema_v2.rs:113-114` / `schema_v3.rs:43-44`）: `Connection::is_autocommit()` で transaction 状態を確認し、transaction 中なら ROLLBACK を試行して結果を併合規則で報告。
  - `PRAGMA foreign_keys` 復元（`schema_v2.rs:30-57`）: `is_autocommit()` で transaction が閉じたことを確認してから復元し、**再読取で復元後の値が元値と一致することを検証**。transaction を閉じられない場合は復元を試みず接続破棄必須の構造化された致命エラーとして返す。復元 PRAGMA・再読取の失敗も記録対象（現行の inner Err 時無記録通過を是正）。
- テスト: Test Design Matrix の全行（失敗注入・回帰・実 mutation 注入）。テストが新規に参照する REQ ID（REQ-901 / REQ-903）は既存のため traceability は `cd src-tauri && cargo run --bin generate_traceability` で再生成し drift ゼロを確認する。
- テスト用の failpoint 機構を導入する。実現機構は **`#[cfg(test)]` の thread-local failpoint** に確定（Plan Gate round 2 P1 裁定）: (1) 設定読取の key 選択的失敗注入 — `mnt/backup.rs` 内に key 指定の設定読取 wrapper を置き、その内部の thread-local failpoint で `backup_retention_days` のみ Err を注入できるようにする（Matrix fixture 条件 (2)）。(2) migration の ROLLBACK / COMMIT 決定論的失敗注入 — MNT-03-D1 が要求する共通ヘルパー（ROLLBACK / COMMIT 実行部）の内部に同じ thread-local failpoint を置く（Matrix fixture 条件 (3)(4)）。trait 注入を採らない理由: `MigrationKind::Custom` は素の関数ポインタ dispatch（`db/migration.rs:23` の型 / `:137-139` の呼び出し）のため、trait object を運ぶには型・`migrations()` 登録・dispatch・既存テスト直呼びまで波及する。thread-local failpoint は本番経路・設計書シグネチャ（71 §71.8/71.9、22 §3.2）・`MigrationKind` を不変に保ち、cargo test の並列実行でも test thread 内で完結する。PR1 の `RestoreFileOps` からは「注入可能な決定論的 failpoint」という思想のみ踏襲し、trait 機構は流用しない。
- Writer の完了条件に release-profile compile check を含める: `cd src-tauri && cargo check --release` green（PR1 の release build 盲点の教訓、Plans.md 明記事項）。

## Non-scope

- frontend / UI / route / bindings の変更（新規 Tauri command なし、command シグネチャ変更なし、`CmdError.kind` の新識別子なし。`generate_bindings` の drift 確認は L1 で走るが差分ゼロが期待値）。
- 71 §71.7（restore）/ §71.4（create_backup）/ 22 §12（legacy 移行）の変更 — PR1 で実装済み。
- `DbError` の variant 構造変更（メッセージ内容の充実のみ。tuple-string 構造は維持）。
- 既存起動失敗 3 経路の無言クラッシュ可視化（Plans.md backlog 済み）。
- Plans.md「非目的」節記載の各項目。

## Acceptance Criteria

- `resolve_backup_dir` が `Result<PathBuf, DbError>` を返し、DB error 注入テスト（Matrix C1、`cd src-tauri && cargo test resolve_backup_dir`）で `Err` 伝搬、未設定/空文字テスト（C2/C3）で既定 fallback が green。
- cleanup の確定条件テスト（Matrix D1〜D5）が green: DB error / parse 失敗注入で**削除 0 件** + `tracing::warn!` 記録、設定行不存在で既定 3 日適用、有効値 90 で 90 日基準（`cd src-tauri && cargo test cleanup`）。
- migration 失敗注入テスト（Matrix E1〜E7）が green: ROLLBACK 失敗時の併合メッセージ（元エラー + ROLLBACK エラー + transaction 状態不明）、COMMIT 失敗注入時の `is_autocommit()` 確認 + ROLLBACK 試行、FK 復元の再読取一致検証、閉塞時の接続破棄必須エラー（`cd src-tauri && cargo test db::` — `db::migration` / `db::schema_v2` / `db::schema_v3` の module 横断 filter。`cargo test migration` は schema_v2/v3 側の新規テストを拾わないため使わない）。
- 実 mutation 注入（Matrix X1 の 5 種）で対応テストが red になることを Writer が実証する: mutation 適用 → `cd src-tauri && cargo test` の failed 出力（red になったテスト名）を採取 → revert、を 5 種それぞれで行い、mutation と red テスト名の対応表を PR body に記録する（推論ベース判定のみで完了扱いしない）。
- 既存テスト green 維持・削除/skip なし: `cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test` 全 green。
- release-profile compile check: `cd src-tauri && cargo check --release` green。
- traceability drift ゼロ: `cd src-tauri && cargo run --bin generate_traceability -- --check` exit 0。
- 生成 bindings 差分ゼロ: `cd src-tauri && cargo run --bin generate_bindings` 後の `git diff --exit-code src/lib/bindings.ts` exit 0。
- L1: `bash scripts/local-ci.sh full` PASS / CLEAN（評価 SHA と evidence は PR body、D-038 Evidence Ownership）。
- Double Audit: 1 pass = Fable inline の契約突合（`docs/function-design/71-mnt-backup.md` §71.8/71.9 + `docs/function-design/22-mnt-migration.md` §3.2 の全契約行 vs 実装・テスト）、2 pass = Codex 独立 fresh context（waive 禁止）。両 pass の findings 裁定が P1/P2 = 0 になるまで是正し、経緯を本 packet `Review Response` 節へ記録する。

## Design Sources

- Requirements / spec: `docs/spec/requirements.md` REQ-901（バックアップ）/ REQ-903（マイグレーション）
- Architecture: `docs/ARCHITECTURE.md`（`UI -> CMD -> BIZ -> IO/MNT` 層、MNT/DB の責務）
- Function / command / DTO: [71-mnt-backup.md](../../function-design/71-mnt-backup.md) §71.8（check_auto_backup + MNT-01-D3）/ §71.9（起動シーケンス + resolve_backup_dir + MNT-01-D2）、`settings_cmd` の既存 error 変換規約
- DB: [22-mnt-migration.md](../../function-design/22-mnt-migration.md) §3.2（migrate + MNT-03-D1）
- Screen / UI: 非該当（UI 変更なし）
- Decision log / ADR: [decision-log.md](../../decision-log.md) D-048（3 本柱、実装 2 PR 分割）、[archived design packet](2026-07-17-backup-migration-failure-contract-design.md) Contract Coverage Ledger の PR2 行、`.claude/rules/implementation-quality.md`（Result 握りつぶし禁止）

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | 71 §71.8/§71.9、22 §3.2、settings_cmd 既存規約 | existing sufficient（PR #14 design phase で確定済み） |
| Command / DTO / generated binding / wire shape | 変更なし（新識別子なし、シグネチャ不変） | existing sufficient |
| DB / transaction / audit / rollback / migration | 22 §3.2 MNT-03-D1（COMMIT / FK 復元含む） | existing sufficient |
| Screen / UI / route state / Japanese wording | 非該当 | 非該当 |
| CSV / TSV / report / import / export format | 非該当 | 非該当 |
| Durable decision / ADR | D-048 | existing sufficient |

## Registration / Generation Obligations

| 新規追加物 | 登録・生成義務 |
|---|---|
| Tauri command | 該当なし（新規 command なし） |
| function-design doc 新設 | 該当なし（既存 71 / 22 の実装のみ） |
| REQ coverage 追加（テスト追加） | 新規テストが REQ-901 / REQ-903 を引用するため `cargo run --bin generate_traceability` で `90-traceability.md` を再生成し `--check` drift ゼロを確認（Scope に明記済み、AC に観測 token あり） |
| route 新設 | 該当なし |
| operator 画面新設 | 該当なし |

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| REQ-901 / MNT-01 | 71 §71.9 | MNT-01-D2 | DB error を既定 dir へ潰すと保存先誤認 + 誤ディレクトリへの cleanup（P3-1 補強）。rejected: PathBuf 直接返却 + 内部 warn | `mnt/backup.rs` `resolve_backup_dir` / `lib.rs` 起動 / `settings_cmd.rs` | Matrix C1〜C6 |
| REQ-901 / MNT-01 | 71 §71.8 | MNT-01-D3 | 読取失敗を既定 3 日へ潰すと 90 日保持設定者のバックアップを誤削除（P3-1 中核）。skip は安全側で自然回復。rejected: `.ok().flatten().unwrap_or(3)` / parse 失敗も既定適用 | `mnt/backup.rs` `run_cleanup` | Matrix D1〜D5 |
| REQ-903 / MNT-03 | 22 §3.2 | MNT-03-D1 | ROLLBACK 失敗の `.ok()` 破棄は transaction 状態不明を隠す（P3-3）。COMMIT BUSY は transaction を残す（Probe A 実証）。FK 復元 PRAGMA は transaction 中 no-op（Probe B 実証）。rejected: 自動再試行 / 元エラー差し替え | `db/migration.rs` / `db/schema_v2.rs` / `db/schema_v3.rs` 共通ヘルパー | Matrix E1〜E7 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: yes — 71 §71.8/71.9 と 22 §3.2 に決定・Why・rejected alternatives・見直し契機が PR #14 で正本化済み。
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: なし（本 packet は既確定契約の実装 scope 化のみ。共通ヘルパーの置き場所 = `db/` 配下は実装詳細であり durable design ではない）。
- Assumptions and constraints: SQLite の COMMIT BUSY 挙動・PRAGMA no-op 挙動・rusqlite `is_autocommit()` の存在は Contract Probe 3 件で実証済み（Contract Probe 節）。
- Deferred design gaps, risk, and follow-up target: 設定値書込み時 validation（MNT-01-D3 見直し契機、設計書記録済み）。restore 遅延成功の起動通知（Plans.md backlog 済み）。
- Test Design Matrix can cite design decision IDs or source doc sections: yes — 全行が MNT-01-D2/D3 / MNT-03-D1 を引用。

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | not applicable — POS/外部ツール非接触。SQLite/rusqlite は core 依存であり adapter ではない | — |
| Fact check / design decision split | 適用: 契約が依存する SQLite 挙動 3 点（COMMIT BUSY 後の transaction 残存 / transaction 中 PRAGMA no-op / `is_autocommit` API）を Contract Probe で観測事実として確定 | Contract Probe 節 |
| Lifecycle / retry | 適用: cleanup skip は「バックアップが溜まる」安全方向の失敗で次回成功時に自然回復。migration 失敗は起動時実行のため再起動が再試行経路（自動再試行は rejected 済み） | Matrix D2/D3（skip 後の自然回復は D1/D4 の確定条件成立で担保）/ E 系 |
| Operator workflow | 変更なし — operator 可視の画面・文言・操作に変更なし。cleanup skip / migration エラーは tracing ログと既存エラー表示経路のみ | — |
| Replacement path | not applicable — 外部システム置換に非接触 | — |
| Data safety / evidence | 適用: テストの削除対象は temp dir 内 synthetic ファイルのみ。実 backup / 実 DB 非使用 | Data Safety 節 |
| Reporting / accounting semantics | not applicable — 集計・帳票非接触 | — |
| Manual verification | Windows native L3 不要と判断: 変更は backend 失敗処理経路のみで、(1) Windows/Tauri native でしか観測できない挙動がない（L3 Eligibility 条件 1 不成立）、(2) 失敗注入は DB lock 操作等 manual fault-injection 級で L3 Eligibility 条件 3 違反 — 自動テストへ route（UI-11c L3-7/8 waiver の教訓どおり）。operator-facing screen change ではないため human visual confirmation slot も非該当 | Matrix（全行自動化）+ 本判断の owner 確認は R4 approval に同梱 |

## Design Readiness

- Existing design docs are sufficient because: PR #14（design phase、squash merge `2e3e97d`）で 71 §71.8/71.9 と 22 §3.2 に MNT-01-D2/D3 / MNT-03-D1 の決定・Why・rejected alternatives・呼び出し元契約・併合メッセージ例まで確定済み。Codex 9 round + 敵対的独立検証 11 巡を通過した正本であり、未解決の設計問題はない。
- Source docs updated in this PR: なし（実装のみ）。
- Design gaps intentionally deferred: 設定値書込み時 validation（見直し契機として設計書記録済み）。
- Durable decisions discovered in this plan and promoted to source docs: なし。

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): MNT（backup）と DB（migration）内の失敗処理是正のみ。CMD は既存 error 変換規約の範囲、UI/BIZ 非接触。
- Backend function design: 71 §71.8/71.9 / 22 §3.2 にシグネチャ・処理ステップ・エラーハンドリングが確定済み。
- Command / DTO / data contract: 変更なし（bindings 差分ゼロが AC）。
- Persistence / transaction / audit impact: migration transaction の失敗時状態確定が本 PR の中核。schema 変更なし・新 migration なし。
- Operator workflow / Japanese UI wording: 変更なし。
- Error, empty, retry, and recovery behavior: cleanup skip = 自然回復、migration 失敗 = 再起動再試行、resolve 失敗 = 起動時 skip / CMD internal error。いずれも設計書の確定契約。
- Testability and traceability IDs: REQ-901 / REQ-903 + MNT-01-D2/D3 / MNT-03-D1 をテストコメントで引用（既存 `test_<関数>_req901_<内容>` 命名踏襲）。

## Contract Probe

- SQLITE_BUSY での COMMIT 失敗は transaction を active のまま残す（MNT-03-D1 COMMIT 契約の前提）: sqlite3 CLI 2 接続で shared lock 保持中に COMMIT → `database is locked (5)` 失敗後、同一接続の `BEGIN` が `cannot start a transaction within a transaction` -> **transaction 残存を実証**（2026-07-18、journal_mode=DELETE）。
- 本番 journal mode（WAL、`db/mod.rs` `configure_database` が設定）での COMMIT BUSY 再現性（Plan Gate round 1 P1 起因の追加 probe）: WAL + `busy_timeout=0` の 2 writer で、後発 writer の lock 衝突は `BEGIN IMMEDIATE` および deferred transaction の最初の write 文で `database is locked (5)` として顕在化し、先行 writer の COMMIT は成功 -> **WAL では contention 由来の COMMIT-時 BUSY は実質発生せず、lock 失敗は既存の「SQL 実行失敗 → ROLLBACK」分岐で先に顕在化することを実証**（2026-07-18）。よって Matrix E3 の COMMIT 失敗再現は実 lock ではなく決定論的な failpoint 注入を正とし、WAL 実 lock の顕在化位置は E3b が回帰として固定する。COMMIT 失敗契約自体は I/O error / disk full 等の contention 以外の失敗に対する defense として維持（22 §3.2 の要求どおり）。
- `PRAGMA foreign_keys` は transaction 中は成功を返しつつ no-op（再読取検証が必要な根拠）: sqlite3 CLI で `foreign_keys=ON` → `BEGIN` → `PRAGMA foreign_keys=OFF`（エラーなし）→ 再読取 = `1` のまま、COMMIT 後も `1` -> **no-op + 成功返却を実証**（2026-07-18）。
- rusqlite に `Connection::is_autocommit()` が存在する（本 repo の依存 version で使用可能）: `src-tauri/Cargo.toml` は rusqlite 0.31、cargo registry 実体 `rusqlite-0.31.0/src/lib.rs` に `fn is_autocommit` を確認 -> **API 存在を実証**（2026-07-18）。
- 登録漏れ是正を含む probe: 該当なし（新規登録物なし）。

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| MNT-01-D2: DB error は Err 伝搬、fallback は未設定/空文字のみ | `mnt/backup.rs` `resolve_backup_dir` | Matrix C1（Err 伝搬）/ C2〜C4（fallback 限定） | non-scope（自動化） |
| MNT-01-D2: lib.rs 起動時 Err → warn + auto-backup skip + 起動継続 | `lib.rs` 起動シーケンス step 7 | Matrix C5 | non-scope（自動化） |
| MNT-01-D2: settings_cmd は Err → internal error（既存規約） | `settings_cmd.rs` `get_backup_dir` | Matrix C6 | non-scope（自動化） |
| MNT-01-D2: 全 backup 操作（create/list/check/restore）が本ヘルパーで backup_dir を統一決定（補記: `restore_backup` command 自体は frontend から `backup_path` を直接受け取り本ヘルパー非経由 — `list_backups` が本ヘルパーの dir から列挙した候補を渡す間接統一 + D-032 の復元前強制バックアップが `get_backup_dir` 経由。71 §71.9 の表現との対応は C7/C8 が実態を固定） | `settings_cmd.rs` 呼び出し 4 箇所 + `lib.rs` | Matrix C7（呼び出し元 enumeration の regression） | non-scope（自動化 + Adjacent Pattern Audit） |
| MNT-01-D2: D-032 復元前強制バックアップ（break-glass 含む）は create_backup 経由の既存伝搬のままで矛盾なし | 変更なし（確認のみ） | Matrix C8（restore 経路の既存テスト green 維持） | non-scope（回帰） |
| MNT-01-D3: 保持日数確定条件 (a) 読取+parse 成功 / (b) 設定行不存在 = 既定 3 日 | `mnt/backup.rs` `run_cleanup` | Matrix D1（未設定 = 3 日）/ D4（有効値 90 適用） | non-scope（自動化） |
| MNT-01-D3: DB error / parse 失敗は cleanup skip + warn、削除実行しない | `mnt/backup.rs` `run_cleanup` | Matrix D2 / D3 | non-scope（自動化） |
| MNT-01-D3: cleanup の成否・skip はバックアップ作成の成否に影響しない | `mnt/backup.rs` `check_auto_backup` | Matrix D2（backup 成功 + 削除 0 件の同時 assert） | non-scope（自動化） |
| 71 §71.8 check_auto_backup 判定手順（既存挙動維持） | 変更最小化 | Matrix G1（既存テスト green） | non-scope（回帰） |
| 71 §71.5 ステップ2a-2d: ファイル名 `inventory_backup_YYYYMMDD_HHMMSS.db` 完全一致のみ削除対象（gated amendment: Double Audit 2 pass P1 で Ledger 欠落 + 既存実装欠陥を検出、same-PR 是正） | `mnt/backup.rs` `extract_date_from_backup` | Matrix D5a | non-scope（自動化） |
| 71 §71.5 ステップ2g: 個別ファイルの削除失敗は warn + 次ファイル継続（gated amendment: 同 2 pass P2） | `mnt/backup.rs` `cleanup_old_backups` | Matrix D5b | non-scope（自動化） |
| MNT-03-D1: ROLLBACK 失敗は tracing::error! 記録 + 元エラーと併合 + transaction 状態不明の明示 | `db/` 共通ヘルパー | Matrix E2 | non-scope（自動化） |
| MNT-03-D1: 共通ヘルパーを全 ROLLBACK 8 箇所へ適用、個別再実装なし | `migration.rs` ×2 / `schema_v2.rs` ×4 / `schema_v3.rs` ×2 | Matrix E6（全箇所 enumeration） | non-scope（自動化 + Adjacent Pattern Audit） |
| MNT-03-D1: COMMIT 失敗時は is_autocommit 確認 + transaction 中なら ROLLBACK 試行 + 併合報告 | `migration.rs` / `schema_v2.rs` / `schema_v3.rs` COMMIT 3 箇所 | Matrix E3（注入）/ E3b（WAL 実 lock の顕在化位置回帰）/ E6（COMMIT 3 箇所の enumeration） | non-scope（自動化 + Adjacent Pattern Audit） |
| MNT-03-D1: transaction を閉じられない場合は FK 復元を試みず接続破棄必須の構造化 fatal | `db/schema_v2.rs` | Matrix E4 | non-scope（自動化） |
| MNT-03-D1: FK 復元は is_autocommit 確認後 + 再読取一致検証、復元・再読取失敗も記録 | `db/schema_v2.rs` FK 復元部 | Matrix E5 / E7 | non-scope（自動化） |
| 22 §3.2 migrate 手順・エラーメッセージ（version + SQL 概要）（既存挙動維持） | 変更最小化 | Matrix E1 + G1 | non-scope（回帰） |

## Test Plan

- targeted tests: [Test Design Matrix](test-matrices/2026-07-18-backup-migration-failure-contract-impl-pr2.md) C1〜C8 / D1〜D5 / E1〜E7 + E3b。
- negative tests: DB error 注入（C1/C6/D2）、parse 不能値（D3）、ROLLBACK/COMMIT/FK 復元失敗注入（E2〜E5/E7）、WAL 実 lock（E3b）。
- compatibility checks: 既存 migration v1→v4 正常適用の回帰（G1）、bindings 差分ゼロ、traceability drift ゼロ。
- data safety checks: テスト削除対象は temp dir 内 synthetic ファイルのみ（Matrix Data Safety 節）。
- main wiring/integration checks: lib.rs 起動経路（C5）と settings_cmd 経路（C6）が新 Result 契約に実配線されていること。ヘルパーが 8 箇所全てに実適用されていること（E6）。

## Boundary / Wire Contract

- producer: 変更なし。`resolve_backup_dir` の Result 化は Rust 内部シグネチャで、Tauri command のシグネチャ・DTO・`CmdError.kind` 識別子は不変。
- consumer: frontend は非接触（bindings 差分ゼロが AC）。
- wire type: 変更なし。
- internal type: `resolve_backup_dir` の戻り値 `PathBuf` → `Result<PathBuf, DbError>`（内部契約のみ）。`DbError::MigrationFailed` のメッセージ文字列が併合情報を含むようになる（variant 構造不変）。
- precision/range: `backup_retention_days` の parse は `u32`（既存）。確定条件の (a)/(b) 判別が新規。
- round-trip path: 非該当。
- invalid input: parse 不能な保持日数 → cleanup skip（削除 0 件）。
- compatibility: 既存の設定値・設定行不存在 DB に対する挙動は不変（D1/C2 で固定）。

## Review Focus

- destructive fallback の除去が完全か: `rg '\.ok\(\)' src-tauri/src/mnt/backup.rs src-tauri/src/db/migration.rs src-tauri/src/db/schema_v2.rs src-tauri/src/db/schema_v3.rs` で失敗握りつぶしの残存を確認（意図的な `.ok()` が残る場合は理由コメント必須）。
- 共通ヘルパーが 8 箇所全てに適用され、個別再実装・適用漏れがないか（Adjacent Pattern Audit）。
- テストが tautological でないか: 各失敗注入テストは「壊れた実装（mutation X1）で red になるか」で判定。推論ベースの anti-tautology 判定を完了扱いしない（PR #15 の最大の教訓）。
- cleanup skip が backup 作成成否に影響していないか（D2 の同時 assert）。
- `is_autocommit()` の確認位置が契約どおりか（COMMIT 失敗直後 + FK 復元前の 2 文脈）。
- 既存挙動の回帰: 未設定時 fallback、正常 cleanup、migration v1→v4 正常適用。

## Spec Contract

Contract ID: SPEC-MNT-FAILURE-PR2

- 破壊的操作（バックアップ削除 / migration 状態確定）は入力と前提状態を確定できた場合のみ実行される。確定できない失敗（DB error / parse 失敗 / ROLLBACK・COMMIT・FK 復元失敗）は既定値・成功へ変換されず、記録付きで安全側（skip / 構造化エラー）に倒れる（D-048 柱 (1)(3)、テストは Matrix C1/D2/D3/E2〜E5/E7）。
- 「設定が無い」と「設定を読めない」は破壊的操作の前提として同値ではない（MNT-01-D2/D3、テストは Matrix C1 vs C2、D1 vs D2）。
- transaction 状態が不明のまま呼び出し元へ返らない: COMMIT/ROLLBACK 失敗時は is_autocommit で状態を確定し、併合メッセージで「transaction 状態不明」を明示する（MNT-03-D1、テストは Matrix E2/E3/E4）。

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| REQ-901 / MNT-01-D2 | resolve_backup_dir Result 化 + 呼び出し元 2 系統 | Matrix C1〜C8 | fallback 限定・Err 伝搬・配線 | cargo test + Double Audit 1/2 pass |
| REQ-901 / MNT-01-D3 | run_cleanup 確定条件 + skip | Matrix D1〜D5 | destructive fallback 除去・backup 成否非影響 | cargo test + mutation X1(c) |
| REQ-903 / MNT-03-D1 | ROLLBACK 共通ヘルパー + COMMIT + FK 復元 | Matrix E1〜E7 + E3b | 8 箇所適用・is_autocommit・再読取検証 | cargo test + mutation X1(a)(b)(e) |
| SPEC-MNT-FAILURE-PR2 | 全体 | Matrix X1 / G1 | tautology 排除・回帰 | mutation red 実証（PR body 記録）+ L1 full |

## Data Safety

- 実 POS / 店舗データ・実 backup ファイル・実 DB・実ログは commit しない（既存規約）。テスト fixture は synthetic のみ。
- cleanup テストの削除対象は `tempfile` 等で作る temp dir 内の synthetic ファイルに限定する。実 `app_data/backups` を参照するテストを書かない。
- rollback / recovery notes（R4 必須）:
  - 本 PR 自体の切り戻し: schema 変更・新 migration・データ形式変更・wire 変更を含まないため、squash commit の `git revert` で完全に戻せる。復旧手順に DB 操作は不要。
  - 変更の失敗方向はいずれも安全側: cleanup skip = ファイルが溜まる（次回成功時に自然回復）、resolve Err = 自動バックアップ skip で起動継続（手動バックアップ・CMD 経路は明示エラー）、migration 失敗 = 従来どおり起動時エラーで停止（変更前より診断情報が増える方向のみ）。
  - 実装ミスの最悪ケース想定: 確定条件の論理誤りで cleanup が誤実行される方向が唯一の破壊的リスク。D2/D3（削除 0 件 assert）と mutation X1(c) がこの方向への感度を持つ。

## Implementation Results

- MNT-01-D2 / MNT-01-D3 / MNT-03-D1 を実装し、設定読取失敗時の fail-safe、cleanup の保持日数確定条件、migration transaction / FK 復元の二次失敗併合を契約どおりに統一した。Matrix C1〜C8 / D1〜D5 / E1〜E7 + E3b / G1〜G3、実 mutation X1a〜X1e、R4 Double Audit はいずれも完了し、未解決 P1/P2 は 0。
- Draft PR: [#17](https://github.com/higekiri-w/inventory-system-public/pull/17)

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

## Review Response

- Plan Gate round 1（2026-07-18、Sonnet 独立 Plan Reviewer、対象 = plan-first commit `fe63252`）: P1=1 / P2=3 / P3=1、全件 accept。
  - P1（E3 の実 lock 前提が本番 WAL で不成立）: Coordinator が追加 probe（Probe A-2、WAL + busy_timeout=0 の 2 writer）で実証確定 — lock 衝突は `BEGIN IMMEDIATE` / write 文で顕在化し COMMIT は成功。是正 = E3 を failpoint 注入ベースへ書き換え + E3b（WAL 実 lock 顕在化位置の回帰）新設 + fixture 条件 (3) 改訂 + Contract Probe 節へ Probe A-2 追記。
  - P2-1（D2 の注入手段が key 選択的でない）: 是正 = fixture 条件 (2) を key 選択的注入可能抽象の必須化へ改訂、D2 を `check_auto_backup` 実経路の integration に変更、packet Scope へ注入可能抽象の導入を明記。
  - P2-2（`cargo test migration` filter が schema_v2/v3 の新規テストを拾わない）: 是正 = AC を `cargo test db::` へ変更。
  - P2-3（COMMIT 是正の Ledger 分類と Matrix evidence の矛盾）: 是正 = Ledger 行を「自動化 + Adjacent Pattern Audit」+ E3/E3b/E6 へ更新、E6 の enumeration を COMMIT 3 箇所へ拡張。
  - P3（Ledger の「create/list/check/restore 統一」文言と restore_backup 実装の字面乖離）: 是正 = Ledger 行へ補記（restore は backup_path 直接受取、list 経由の間接統一 + D-032 経路が get_backup_dir 経由）。
- Plan Gate round 2（2026-07-19、同 Plan Reviewer 継続 context、対象 = 是正 commit `0780888`）: round 1 の 5 findings は **resolved 5/5**。新規 P1=1 / P2=2 / P3=0、全件 accept。
  - P1（「RestoreFileOps trait パターン踏襲」が `MigrationKind::Custom` の fn pointer dispatch と両立しない）: Coordinator が `db/migration.rs:19-23,137-139` を実読して裁定確定。是正 = 実現機構を「MNT-03-D1 共通ヘルパー内 + 設定読取 wrapper 内の `#[cfg(test)]` thread-local failpoint」に確定し、Scope の機構記述を訂正（trait 機構は流用しない旨と理由を明記）。本番シグネチャ・`MigrationKind` は不変のため Boundary / Wire Contract への波及なし。
  - P2-1（E3b の oracle が v1 deferred BEGIN と v2/v3 BEGIN IMMEDIATE を混同）: 是正 = E3b を v1（write 文失敗 → ROLLBACK 分岐）に限定し、v2/v3 の BEGIN IMMEDIATE contention は「transaction 未開始の直接 Err（ROLLBACK 不要）」として oracle から明示除外。
  - P2-2（Negative Paths の dependency missing 行が fixture 条件 (2) の使用禁止と矛盾）: 是正 = D2 を注入手段候補から削除。
- Plan Gate round 3（2026-07-19、同 Plan Reviewer 継続 context、対象 = 是正 commit `3cdcdd5`）: round 2 の 3 findings は **resolved 3/3**。thread-local failpoint 機構の妥当性 3 点（本番シグネチャ不変 / test 並列安全 / E2・E3・E4・E7・D2 の実現可能性）も実読ベースで成立と判定。最終 sweep で新規 P1/P2 = 0 — **Plan Gate 収束**。P3=1（C1 注入手段の記述と採用機構の対応が古い）は同 commit で 1 行明記により消化。
- Writer 自己レビュー round 1（2026-07-19、契約正本 71 §71.5/§71.8/§71.9 + 22 §3.2 の各行 vs live 実装・テスト）: ROLLBACK 8 / COMMIT 3 箇所、Result 呼び出し元、cleanup 確定条件、FK 復元ゲートと再読取を走査。P1/P2=0。P3 相当の stale code comment 2 件（保持日数の既定条件 / schema_v2 の FK 復元条件）は同時修正。
- Writer 自己レビュー round 2（2026-07-19、Matrix C1〜C8 / D1〜D5 / E1〜E7 + E3b と anti-tautology の再突合）: P2=1（E7 が inner success + 復元系失敗のみで、Matrix 指定の inner Err + 復元系 Err の併合分岐を未検証）を検出・accept。synthetic FK 不整合で inner Err を作る fixture へ E7 2 tests を強化し、一次 `FK整合性エラー` + 二次復元/再読取エラー + ERROR log を同時 assert。targeted 3 tests green、是正後 P1/P2=0。
- R4 review-only sub-agent（2026-07-19、独立 context、live diff Contract Audit）: 初回 P1=0 / P2=1（Writer round 2 と同じ E7 二重失敗 fixture gap）。是正後 closure-only re-review で **RESOLVED / 未解決 0**。本番 `MigrationKind` / fn pointer 不変、ROLLBACK 8 / COMMIT 3、negative space、State Lifecycle、Data Safety、release cfg を確認。
- Coordinator 記録訂正（2026-07-19）: Implementation Results の「R4 Double Audit はいずれも完了し」は Writer 内部の review-only sub-agent を指しており、本 packet AC が定義する Double Audit（1 pass = Fable inline / 2 pass = Codex 独立 fresh context）とは別物。AC 上の Double Audit は下記のとおり本記録以降に実施する。
- Double Audit 1 pass（2026-07-19、Fable inline、対象 = content HEAD `fe2441d` の実装・テスト vs 71 §71.5/71.8/71.9 + 22 §3.2 + D-048）: **blocker なし（P1/P2 = 0）**。確認事項 = migration_tx.rs の併合規則・is_autocommit 2 文脈・FK 復元ゲート + 再読取検証 + no-op 注入 oracle / ROLLBACK 8 + COMMIT 3 箇所の共通ヘルパー全適用（source-scan テストによる機械固定 = E6 自動化を確認）/ resolve_backup_dir の Err 伝搬と fallback 限定 + 呼び出し元 2 系統の契約 / run_cleanup の (a)(b) 確定条件・skip + warn・backup 成否非影響（D2 は check_auto_backup 実経路で同時 assert）/ E3b の実 2 接続 WAL contention oracle / 裸 ROLLBACK・`.ok()` 握りつぶし残存ゼロ（テスト fixture 除く）/ Cargo.toml 依存追加なし / packet への Writer 追記が append-only 節に限定されること（zero-context hunk 確認）。L1 full evidence = PASS / CLEAN / MERGE_EVIDENCE_VALID=true（SHA は PR body 正本）。
- Double Audit 2 pass（2026-07-19、Codex 独立 fresh context、対象 = git diff fe63252..fe2441d + 契約正本直読 + 実 mutation 再実行）: P1=1 / P2=2 / P3=0、**全件 Coordinator 実証裁定で accept**。
  - P1（`extract_date_from_backup` が separator/HHMMSS を検証せず非 timestamp suffix を削除対象化）: 実文 + LSP 参照確認で確定。**既存欠陥**（plan-first `fe63252` 時点に同一実装 = PR2 非起因）だが、§71.5 は本 PR の touched contract（cleanup 駆動）であり R4 削除経路のため same-PR fix。Ledger 欠落は gated amendment で §71.5 2 行を追加。
  - P2-1（E3 oracle が実 commit_error の併合を検証しない — 2 pass の独自 mutation N1 が green で立証）: accept。E3 assert へ `injected COMMIT failure` + `ROLLBACK 成功` の実文検証を追加。drift-fix sweep で弱 assert は repo 内 1 箇所のみと確認。
  - P2-2（削除失敗 warn + 継続の未検証 — 独自 mutation N2 が green で立証）: accept。D5b 新設（ディレクトリによる決定論的 remove_file 失敗）。
  - 是正実装 = Coordinator 直接（2 file の軽微修正。owner effort budget 内で完結させるための conductor-mode 例外適用。自己承認を避けるため closure は独立 reviewer が指定した oracle への機械的再検証で判定）。是正 commit `5479d06`、全 gate green（fmt / clippy / cargo test / `cargo check --release`）。
  - Closure 再検証（Coordinator、実 mutation 注入）: mutation A（検証を日付のみへ戻す）→ D5a red / mutation B（併合から commit_error 除去 = 2 pass N1）→ E3 red / mutation C（warn 除去 = 2 pass N2）→ D5b red。**3/3 red を実証、全 mutation revert 済み・tree clean**。2 pass の他の確認事項（ROLLBACK 8 + COMMIT 3 + FK 復元 + 呼出し元 + 層境界 + release への failpoint 漏出なし）は一致報告のため追加対応なし。
- Findings Freeze: **frozen after Broad Audit**（Double Audit 両 pass + closure 完了、2026-07-19 発効）; post-freeze exceptions: none.

If R3 review-only sub-agent is skipped, record an explicit line beginning with `Review-only skipped because:` and the reason.
