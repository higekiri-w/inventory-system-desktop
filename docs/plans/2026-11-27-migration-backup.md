# Plan Packet: 更新・migration の前の保全（関門 3 の (3)）

導入の前の関門 3（[Plans](../Plans.md#導入の前の関門)）の残りの (3) の 1 つ目。出所は [backlog](../backlog.md) の「backup に、PC の外のコピーと復元の実証が運用として設計されていない」の「導入の前の関門」の段の (3)（owner の決定、repo 外の回答台帳 TD-674・TD-679・TD-843。まとめは [decision-log](../decision-log.md) D-162 (4)）。D-114 の Revisit「更新・migration の前の保全を設計するとき」に当たる。起点は `11b1d793`（2026-11-27）。

起票の commit は本 packet と backlog の (3) の段の link だけ。owner の答え（2026-11-27、repo 外の回答台帳 TD-930〜TD-933）の後の計画の commit で、22 §24 と §16「回復」の手順 1・71 §71.9 に proposed・未実装の文を入れた（`scripts/plan-approve.sh` は review の後の lane の commit を packet だけに限るので、設計正本の文は計画の review の前に置く）。`src-tauri/**`・proposed の注記の外し・手順書の 2 文は計画の固定の後の実装の commit。

## Workflow State

- Risk: R4
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R4

Reason: 戻せない変更（migration は版ごとに COMMIT し、旧版のアプリは新しい版の DB を開けない。22 §16「回復」）の前に置く保全の control で、作れなければ起動を止める。`docs/DEV_WORKFLOW.md` Risk Tiers の R4「backup の復元」（この控えは旧版へ戻す唯一の材料になる。[migration-recovery-runbook](../migration-recovery-runbook.md) の手順 1）と「破壊的なデータの lifecycle」（migration の入口の順を変える）に当たる。owner も「R4 見込み」とした（D-162 (4)）。DB の表・migration の版・command・DTO・画面の file は変えない。

## Goal

Goal Invariant:

### 最小完了条件

- 版が 1 以上でアプリの最大より小さい DB を新しい版のアプリで起動すると、最初の migration を当てる前に、その版のままの使える控え（71 §71.3 の名前、§71.12 の「使える控え」の検査に合格、71 §71.4 の `create_backup`）が保存先にできる。これは今ある版と、#307・#306 などがこれから足す版の全部の migration の前に効く。
- 控えを作れなければ、migration を 1 つも当てず（DB の版と論理内容は変わらない）起動を止め、原因と直し方を名指す文を窓の前の dialog に出す。
- 版がアプリと同じ DB、DB が無い初回の起動（版 0）、アプリより新しい DB（MNT-03-D11）では控えを作らず、今と同じに動く。

### 失敗定義

- 控えが migration の後の版で作られる、または 1 つでも migration を当ててから作られる。
- 控えを作れないのに migration を当てる・起動を続ける。止まったときに DB の版・表・行が変わる。
- 自動 backup の設定（`backup_enabled`）が `'0'` だと控えを作らない（owner の決定は「作れなければ止める」で、設定で外せる形にしない）。
- 版が同じ DB で起動のたびに控えができる。初回の起動・新しすぎる DB で何かを書く。
- 復元（71 §71.7）・取込み（22 §12）・reconcile（MNT-01-D5）の順と振舞いが変わる。

### 非目的

- 店の人が流せる更新の手順（(3) の 2 つ目）。本 lane の merge の後の別の lane（owner の決定 TD-932、[backlog](../backlog.md) の (3) の段）。本 lane は手順書の今の文のうち、この lane で事実が変わる 1 文ずつだけを直す（Scope）。
- 自動更新（backlog の保留「アプリの自動更新（updater）」）。
- 復元の中の migration（71 §71.7。戻す元の控えの file は残り、今の DB は復元の前の事前の backup〈D-032〉が守る）。
- 版 0 の既存の file（`schema_versions` の無い DB）の扱い。今のまま。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 確かめた事実（2026-11-27、`11b1d793` の code の読み）

- F1 起動の順: `run` の setup は、診断ログ（`src-tauri/src/lib.rs:1474`）→ `prepare_database`（`:1489`）→ 操作ログの掃除（`:1504`）→ `run_startup_auto_backup`（`:1509`）。`prepare_database_with_init`（`:150-199`）の中は、`reconcile_restore`（`:162`）→ 新 DB が無ければ legacy の取込み（`:165-183`）→ `init`（= `db::init_database`、`:188`）→ `complete_reconciled_restore`（`:194`）。
- F2 migration の入口: `init_database`（`src-tauri/src/db/mod.rs:189`）は open の後 `configure_database`（`:206`）で PRAGMA（`foreign_keys`・`journal_mode = WAL`・`busy_timeout`）の後に `migration::migrate`（`:219`）を呼ぶ。`migrate` は `read_current_version_without_ddl` で版を読み、アプリの最大より新しければ書かずに `SchemaNewerThanApp`、そうでなければ migration ごとに BEGIN〜COMMIT する（`src-tauri/src/db/migration.rs:217-251`）。
- F3 今、migration の前に控えは作られない: 起動の自動 backup（`run_startup_auto_backup`、`lib.rs:125-136`）は `prepare_database` の後で、migration は全部当たった後。しかも `check_auto_backup` は `backup_enabled` が `'1'` で今日の正式名の backup が無いときだけ作る（`src-tauri/src/mnt/backup.rs:825-860`）。手順書もこれを前提にし、更新の前に旧版で手動の backup を作る（[msi-distribution-runbook](../msi-distribution-runbook.md) の流れ 3、[migration-recovery-runbook](../migration-recovery-runbook.md) の手順 1、22 §16「回復」の手順 1、71 §71.9）。
- F4 控えが作れないとき: 起動の自動 backup の失敗は `tracing::warn!` だけで起動を続ける（`lib.rs:125-136`、MNT-01-D2）。migration の前の止める所は無い。
- F5 使える既存の手段: `create_backup`（`backup.rs:623`）は書く先の固定（MNT-01-D10）・作業名への `VACUUM INTO`・§71.12 の検査・上書きしない公開・操作ログ `backup_create` までを持ち、検査は版がアプリ以下の DB を受ける（`backup.rs:521` の `inspect_backup`）。`resolve_backup_dir`（`backup.rs:610`）は `app_settings` の `backup_path` を読み、DB の error を `Err` で返す。`app_settings` と `operation_logs` は v1 から形が同じ（`src-tauri/src/db/schema_v1.rs:232-246`、後の版で作り直していない。`rg -n operation_logs src-tauri/src/db/schema_v*.rs` は v1 の定義と v9 の test の読みだけ）。窓の前の dialog は `show_pre_window_fatal`（`lib.rs:205`、MNT-03-D4）。
- F6 呼出し側: `init_database` を runtime で呼ぶのは起動だけ（`rg -n 'init_database\(' src-tauri/src` の test の外は `lib.rs` の `prepare_database_with`）。復元は `open_existing_database`（`src-tauri/src/mnt/restore.rs:516`）で、同じ `configure_database` を通る。

## Scope

順: PR #307（PLU の runtime、`db/mod.rs`・`lib.rs`・migration を触る）の merge と `origin/main` の同期の後に実装を始める。

runtime（計画の固定の後）:

- `src-tauri/src/db/mod.rs`: `configure_database` を PRAGMA の段と migrate の段に分け、起動が間に控えを挟めるようにする。`init_database`・`open_existing_database` の振舞いは変えない（復元は今のまま、非目的）。
- `src-tauri/src/lib.rs`:
  - 起動の DB の準備（`prepare_database_with_init` の `init` の実体）を、open と PRAGMA → `read_current_version_without_ddl` → 版が `0 < 版 < app_max_version()` のときだけ控えを作る → migrate、の順にする。控えは `resolve_backup_dir` と `create_backup` を呼ぶだけ（新しい backup の関数を書かない）。`backup_enabled` を見ない。`cleanup_old_backups` は呼ばない。保存先（owner の決定 TD-930）: `resolve_backup_dir` の保存先で `create_backup` が `Err` なら、その保存先が既定の `{app_data}/backups` と違うときだけ既定でもう 1 回 `create_backup` を呼び（1 つ目の `Err` は診断ログに `warn`）、両方 `Err` なら止める。
  - `StartupDatabaseError` に控えの失敗の variant を 1 つ足し、`operator_message` に固定の文を置く（原因と直し方を名指す。文の案は Design Readiness の「文」。末尾に今の variant と同じく `\n{details}`）。`resolve_backup_dir` の DB の error は既定へ移らずに止める（設定を読めない DB は控えの元としても読めない見込みで、MNT-01-D2 の「読めない設定で既定へ fallback しない」と同じ。起動の自動 backup の「warn で skip」とは違う）。
  - test（Test Plan の T1〜T7）。入力だけ直す既存の test は 1 本: `test_startup_req903_vb_stop_after_earlier_commit_backup_path_readable_from_copy` は版 7 の DB の `backup_path` に `D:\合成の保存先` を入れて起動する。新しい振舞いでは控えがまずその path（Windows 以外では作業フォルダの中の相対の folder）へ書かれ、作れなければ (B) で `{app_data}/backups` に書かれ、後の `file_hashes(&app_data)` が `backups` の directory を `std::fs::read` で読んで落ちる。`backup_path` を一時 directory の中で `app_data` の外の path にし（控えはそこにでき、`app_data` に `backups` ができない）、写しから読む値の期待をその path に合わせる。期待の意味（`DatabaseInit` と vB の文、写しから設定の値が取れる、元の file が変わらない）は変えない。
  - 意図した簡略化は `ponytail:` comment に書く（下の Design Readiness の「抜け道」の 2 つ目）。
- `docs/function-design/22-mnt-migration.md`: §24 の MNT-03-D25 と §16「回復」の手順 1 の注は計画の commit で proposed・未実装として入れた。実装の commit で注記を外し、手順 1 の「起動時の自動の backup は migrate の後に作られる」の文を事実に合わせる。`db/mod.rs` で分けた関数を `pub` にするなら、その名を §24 に書く（`src-tauri/tests/design_compliance_test.rs` の突合）。
- `docs/function-design/71-mnt-backup.md` §71.9: 起動の順の migration の前の控えは計画の commit で proposed・未実装として入れた。実装の commit で注記を外す。
- `docs/migration-recovery-runbook.md` の手順 1 と `docs/msi-distribution-runbook.md` の流れ 3 の前文（実装の commit。今のアプリの振舞いを書く手順書なので、計画の commit では直さない）: 同じ「自動の backup は migration の後」の文だけを直す（手順の作り直しは本 lane の merge の後の手順書の lane）。
- `docs/function-design/90-traceability.md`: generator で作り直す（手で編集しない）。
- `docs/backlog.md`: 実装の commit で (3) の段の「アプリが migration の前に控えを作り、作れなければ止める lane」を済みにする（関門の行は実装の PR で外す）。(3) の段の手順書の lane が残ることと owner の所感（TD-932）は計画の commit で書いた。

Non-scope: `src-tauri/src/db/migration.rs`（呼ぶだけ）・`schema_v*.rs`・`mnt/restore.rs`・`mnt/backup.rs`（呼ぶだけ）・`mnt/offsite.rs`・`cmd/**`・`src/**`・bindings、`Plans.md`、`docs/decision-log.md`（D-173 は足さない。owner の決定は D-162 (4) にあり、実装の判断は子の ID の MNT-03-D25 に置く）。

path の所有（並走の lane との突き合わせ）:

| file | 本 lane が触る所 | 重なる lane |
|---|---|---|
| `src-tauri/src/lib.rs` | `StartupDatabaseError`・`prepare_database_with_init` とその test の module | #307（同じ所を触りうる。#307 の後に始める）、#305（`invoke_handler` の command の登録。行が別） |
| `src-tauri/src/db/mod.rs` | `configure_database`・`init_database` | #307 |
| 22 | 末尾の §24 と §16「回復」の手順 1 の 1 文 | #307・#306（末尾の追記は merge 順で両方を残す。節の番号は後の側が取り直す） |
| 71 | §71.9 だけ | #307、1159（§71.13） |
| `docs/msi-distribution-runbook.md`・`docs/migration-recovery-runbook.md` | 上の 1 文ずつ | 手順書の lane（本 lane の merge の後） |

## Acceptance Criteria

- AC1（全部の migration の前）: 版 `v` を 1 から `app_max_version() - 1` まで 1 つずつ回し（数は `app_max_version()` から導き、版の数を書かない）、版 `v` まで当てた DB を起動の DB の準備に渡すと、保存先に正式名の控えが 1 つでき、その控えの `inspect_backup` が `schema_version = v`・`quick_check_ok = true`、起動の後の DB の版は `app_max_version()`。#307・#306 の版はこの回し方に入る。
- AC2（既定へ移る・両方だめなら止める）: 版 `app_max_version() - 1` の DB で、(i) 設定の保存先（`backup_path`）が file で folder を作れないと、`{app_data}/backups` に AC1 と同じ控えができて migrate が当たり、設定の保存先に正式名も作業名も残らない。(ii) 設定の保存先と `{app_data}/backups` の両方が file だと（`backup_path` が未設定で `{app_data}/backups` が file の場合も）、起動は新しい variant の error を返し、`operator_message` が Design Readiness の「文」の固定の部分と試した保存先の path を含む。DB の `MAX(version)`・`sqlite_master` の `sql` の一覧・各表の行の数が前後で同じで、どちらの保存先にも正式名も作業名も残らない。(iii) `resolve_backup_dir` の DB の error では既定へ移らずに (ii) と同じく止まる。
- AC3（作らない場合）: 版が `app_max_version()` の DB の起動、DB の無い初回の起動、版が `app_max_version() + 1` の DB の起動のどれでも、控えの準備の段は file を作らない（新しすぎる DB は今どおり `SchemaNewerThanApp` で、保存先も DB も変わらない）。
- AC4: `backup_enabled = '0'` の DB でも AC1 の控えができる。
- AC5（順）: legacy の取込み（作業フォルダの旧 DB、版 `app_max_version() - 1`）の後と、reconcile が退避から戻した後の DB でも、控えは migration の前の版で作られる。reconcile・取込みが失敗した起動では控えを作らない（今の test の panic の注入の形）。
- AC6: 既存の test が期待値を変えずに全部 PASS する（`lib.rs` の `test_startup_`・`db::migration`・`mnt::backup`・`mnt::restore`）。入力を直すのは Scope に名指した 1 本だけ。`git diff --stat origin/main...HEAD` に `src/**`・`src-tauri/src/cmd/**`・`src-tauri/src/db/migration.rs`・`src-tauri/src/db/schema_v*.rs`・`src-tauri/src/mnt/restore.rs` が出ない。
- AC7（mutation）: 実装に注入して当たる test が red になることを確かめて PR に書く: (m1) 控えを migrate の後に作る → T1、(m2) 条件を `版 <= app_max` にする → T3、(m3) 控えの失敗を warn にして続ける → T2、(m4) 版 0 でも作る → T3、(m5) `backup_enabled` を見る → T4、(m6) 既定へ移らずに止める → T7、(m7) `resolve_backup_dir` の DB の error でも既定へ移る → T2。
- AC8: `bash scripts/doc-consistency-check.sh` と `--target plan docs/plans/2026-11-27-migration-backup.md` が ERROR 0、`bash scripts/check-workflow-git.sh` が PASS、`cd src-tauri && cargo run --bin generate_traceability -- --check` が PASS。Rust の検証は `docs/DEV_WORKFLOW.md` の Verification Gates（hosted CI の `Merge gate`）。

## Design Readiness

- 引用する設計正本（節まで）: 22 §3.2（MNT-03-D1・D11）・§12.4（MNT-03-D4・D5〜D8 の文言の表）・§16「回復」、71 §71.3・§71.4（MNT-01-D7・D10）・§71.5（MNT-01-D3）・§71.7・§71.8・§71.9（MNT-01-D2）・§71.11・§71.12、D-032・D-114・D-132・D-162 (4)、`docs/msi-distribution-runbook.md` の流れ 3〜5、`docs/migration-recovery-runbook.md`。
- 必要な設計成果物: updated in this PR（22 §24 の MNT-03-D25 と §16「回復」の手順 1 の注、71 §71.9 の注を計画の commit で proposed・未実装として入れた。実装の commit で注記を外す）。DB・migration・DTO・画面は不要。
- owner の決定（2026-11-27、repo 外の回答台帳）: 保存先は、設定の保存先で作れなければ既定の `{app_data}/backups` で作り、両方だめなら止める（TD-930）。控えは普通の backup と同じ名前と folder で、保持日数で消え、PC の外の控えにも写る（TD-931）。店の人が流せる更新の手順は本 lane の merge の後の別の lane（TD-932）。L3 は置かない（TD-933。下の Test Plan）。
- plan にしかない durable な判断の昇格先: MNT-03-D25（上の owner の決定・棄却した案・見直し契機。22 §24）。
- 前提・制約と延期した design gap: 窓の前の起動で控えを作るので、DB の大きさの分だけ窓が出るまで待つ（未実測。Contract Probe の P1）。店の人の更新の手順は上の別の lane。
- 絶対保証（cannot happen / always happens）の例外と抜け道:
  - 控えは「その起動の最初の migration の前」の版。同じ起動で後の migration が失敗して止まり、次の起動で残りを当てるとき、次の起動の控えは途中の版になる（最初の起動の控えは保存先に残る。今の手順書の流れ 5 の「場面 2」と同じ）。
  - 失敗が続く起動（migration が止まる DB を何度も起動する）では、起動のたびに控えが 1 つ増える。保存先は保持日数（MNT-01-D3）で、PC の外の媒体は 30 本の保持（MNT-01-D12 (4)）で古い順に消えるので、30 回を超えて起動すると媒体の古い本当の控えが押し出される。同じ日・同じ版の控えがあれば作らない形は足さない（D-147。広げる条件: 運用で再起動の繰返しが起きたとき）。
  - 控えは今日の正式名なので、起動の自動 backup（§71.8）は今日の backup があると数え、その日の migration の後の backup は定時（`backup_time`）まで作られない（既定の folder へ移った控えは数えない）。
  - 控えの後に `create_backup` が旧版の DB へ操作ログ `backup_create` を 1 行書く（F5。控えには入らない）。
  - 物理の書込み: PRAGMA（`journal_mode = WAL`）は今どおり控えの前に当たる（MNT-03-D11 の「物理的な書込み」と同じで、論理内容を変えない）。
  - 既定の folder へ移った控え（設定の保存先が別のとき）は、復元の画面の一覧・保持日数の掃除・PC の外の控えのどれにも入らない（どれも設定の保存先だけを見る。`resolve_backup_dir` の呼出し側）。旧版へ戻すときにこの控えを拾うのは管理者の手順（22 §16「回復」、手順書の lane）で、店の人に folder を探させない。店の人の画面に出す機能は今は足さない（広げる条件: 手順書の lane が店の人の手順にこの控えを要るとしたとき、backlog に出す）。
- 文（失敗の dialog。固定の部分の案、計画の review で確かめる）: 「更新の前の控え（バックアップ）を作れなかったため、データの形を新しい版に切り替えずに起動を中止しました（データは変更していません）。試した保存先: {試した path を ・ でつなぐ}。PC の空きが少なければ空けてから、もう一度起動してください。直らなければ、前の版のアプリを入れ直すとそのまま使えます。わからない場合は管理者へ連絡してください。」
- Contract Probe: P1 店の大きさの DB での `VACUUM INTO` と検査の時間（窓が出るまでの待ち）。店の DB の大きさが未定なので、計画の review で測る大きさ（商品の数・明細の数）を決めて合成の DB で測る。未実測。
- 判定: ready（owner の未決は無い。計画の review に出す。P1 の測る大きさは計画の review で決める）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| MNT-03-D25（新、proposed）migration の前の控え・既定へ移る・両方だめなら止める | 22 §24 | `lib.rs` の DB の準備、`db/mod.rs` の分け方 | T1〜T5、T7 | 非対象（L3 なし、TD-933） |
| MNT-03-D11 新しすぎる DB は書く前に拒む | 22 §3.2 | 順を変えない（版の比べが控えより前） | T3、既存 `test_startup_req903_d11_newer_schema_stops_startup_with_own_message` | 非対象 |
| MNT-03-D1 migrate の TX・ROLLBACK | 22 §3.2 | 変えない | 既存 `db::migration` の test | 非対象 |
| MNT-03-D4・D5 起動の fail-closed と窓の前の dialog | 22 §12.4 | 新しい variant と文 | T2、T6 | 非対象（L3 なし、TD-933） |
| 22 §12 legacy の取込み・MNT-01-D5 reconcile の順 | 22 §12、71 §71.7 | 変えない（控えはその後） | T5、既存 `test_startup_req903_m7_`・`m8_`・`req901_b6_`・`b9_` | 非対象 |
| 22 §16「回復」手順 1、作り直しの手順の起動の test | 22 §16 | 1 文を事実に合わせる | 既存 `test_startup_req903_` の 4 本（`test_startup_req903_vb_stop_after_earlier_commit_backup_path_readable_from_copy` は入力だけ直す） | 非対象 |
| MNT-01-D7・D10 確かめてから公開・書く先の固定 | 71 §71.4 | `create_backup` を呼ぶだけ | T1（`inspect_backup` の合格）、既存 `mnt::backup` | 非対象 |
| MNT-01-D2 `resolve_backup_dir` の error | 71 §71.9 | 起動の控えでは既定へ移らずに止める（自動 backup は今の warn のまま） | T2 | 非対象 |
| MNT-01-D3 保持・§71.8 今日の backup の判定 | 71 §71.5・§71.8 | 変えない（控えは普通の backup と同じ名前と folder、TD-931） | 既存 `test_check_auto_backup_req901_no_backup_today` | 非対象 |
| 71 §71.9 起動の順 | 71 §71.9 | 文を直す | 既存 `test_startup_req901_d2_resolve_error_warns_and_skips_auto_backup` | 非対象 |
| 71 §71.7 復元の中の migrate、D-032 | 71 §71.7 | 変えない（非目的） | 既存 `test_restore_backup_req901_runs_migration` | 非対象 |
| §71.11 PC の外の控え | 71 §71.11 | 変えない（設定の保存先の控えは最新の backup として写る。既定へ移った控えは写らない） | 非対象 | 非対象 |
| D-162 (4)・D-114 Revisit | decision-log | 実装で当たる（本文は変えない） | 非対象 | 非対象 |
| 手順書の 1 文 | msi-distribution-runbook 流れ 3、migration-recovery-runbook 手順 1 | 文を直す（実装の commit） | 非対象 | 非対象（手順は手順書の lane） |

## Test Plan

Rust の unit test（`src-tauri/src/lib.rs` の test の module。一時 directory、`migrate_up_to` で版を作った合成の DB）。test の先頭の comment に `REQ-903 / MNT-03-D25` と節を書く。`## L3` 節は置かない（TD-933）: 成功の道は画面を変えず、失敗の dialog は保存先を書けなくする故障の注入が要る（[DEV_WORKFLOW](../DEV_WORKFLOW.md) の L3 Eligibility の (3)）ので下の自動 test で持ち、窓の前の dialog の見え方は MNT-03-D4 で確かめ済み。

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| MNT-03-D25 全部の migration の前 | 控えが migration の後か途中の版になる。新しく足した版で効かない | unit（T1、版を 1〜`app_max - 1` で回す） | `test_startup_req903_d25_backup_before_every_migration` | 控えを migrate の後に作る（m1）、版を決め打ちする |
| MNT-03-D25 両方だめなら止める | 作れずに migration を当てる・起動を続ける・DB が変わる | unit（T2、表: 設定の保存先と `{app_data}/backups` の両方を file にする、`backup_path` 未設定で `{app_data}/backups` を file にする、`resolve_backup_dir` の error） | `test_startup_req903_d25_backup_failure_stops_before_migration` | warn で続ける（m3）、DB の error で既定へ移る（m7）、止まる前に 1 つでも migrate する、文に試した保存先・直し方が無い |
| MNT-03-D25 既定へ移る | 設定の保存先で作れないと止まる（抜いた USB メモリで起動できない）、既定へ移った後に設定の保存先に作業名が残る | unit（T7、設定の保存先を file にする） | `test_startup_req903_d25_falls_back_to_default_dir` | 既定へ移らずに止める（m6）、既定の控えが §71.12 の検査に合格しない、移った後に migrate しない |
| MNT-03-D25 作らない場合 | 起動のたび・初回・新しすぎる DB で書く | unit（T3、表: 版 = app_max、DB なし、版 = app_max + 1） | `test_startup_req903_d25_no_backup_when_nothing_to_migrate` | 条件を `<= app_max` にする（m2）、版 0 で作る（m4）、版の比べより前に作る |
| MNT-03-D25 設定に依らない | `backup_enabled = '0'` で作らない | unit（T4） | `test_startup_req903_d25_backup_ignores_backup_enabled` | `backup_enabled` を見る（m5） |
| MNT-03-D25・§12・MNT-01-D5 の順 | 取込み・reconcile の後の DB で控えが無い・後の版になる。reconcile・取込みの失敗で控えを作る | unit（T5） | `test_startup_req903_d25_backup_after_legacy_import_and_reconcile` | 控えを取込み・reconcile より前に置く、取込みの後の DB を見ない |
| MNT-03-D4・D5 文 | 原因・直し方の無い文、variant が `DatabaseInit` に潰れる | unit（T6） | `test_startup_req903_d25_operator_message_names_cause_and_fix` | 文の固定の部分を変える、`\n{details}` を落とす |
| 既存の契約 | 既存の振舞いが変わる | unit（既存） | Contract Ledger の既存の test | 期待値を変えないと通らない変更を入れる |

状態の遷移: initial（版 `v` の DB）→ 控え（success / failure）→ migrate → 起動の後の版。failure は T2（止まった後に同じ DB で保存先を直して起動し直すと控えができて migrate が当たる retry を同じ test に入れる）。restart は T1 の後にもう一度起動して控えが増えない（T3 の版 = app_max の case）。mutation は AC7。比べるのは file の bytes でなく版・`sqlite_master`・行の数（`journal_mode = WAL` の PRAGMA が header を書きうるため）。

## Recovery

- 本 lane が足すのは控えの file の作成と、作れないときの起動の停止だけで、DB の表・行・版を変えない。
- 誤って止まる（作れるはずの控えで止まる）とき: DB は migration の前の版のまま変わっていないので、前の版の MSI を入れればそのまま開ける（[msi-distribution-runbook](../msi-distribution-runbook.md) の流れ 5 の 1、M6）。修正版で前へ進んでもよい。
- 誤って止めない（控えなしに migration を当てる）とき: 今の main と同じ状態に戻るだけで、旧版へ戻すには流れ 3 の手動の backup が要る（手順書は手順書の lane まで今のまま流れ 3 を持つ）。
- code の戻し方: 修正の PR を既定にし、本 lane の PR の revert も入れられる（DB・migration・状態の file の形を変えないので、版を戻しても DB と控えはそのまま読める）。
- 作った控えは普通の backup で、保持日数で消える（TD-931）。既定へ移った控えは掃除の外で残る（Design Readiness の抜け道）。消し方を足さない。

## Data Safety

- 書くもの: 保存先の作業名と正式名の控え（`create_backup` の既存の経路）、旧版の DB への操作ログ `backup_create` 1 行。消すもの: 前回の中断の作業名（`create_backup` の手順 3 の既存の経路）だけ。`cleanup_old_backups` を呼ばない。
- 止まったとき: DB の論理内容と版は変えない。作業名は `create_backup` が消し、消せなければ次の作成が消す（MNT-01-D7）。
- commit しないもの: 実の店の DB・backup・診断ログ。test の入力は一時 directory の合成の DB だけで、tracked の fixture を足さない。P1 の測る DB も合成。
- owner の決定と店の事実は repo 外の回答台帳に残し、repo には TD 番号と要旨だけを書く。
