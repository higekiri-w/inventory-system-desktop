# Plan Packet: 更新・migration の前の保全（関門 3 の (3)）

導入の前の関門 3（[Plans](../Plans.md#導入の前の関門)）の残りの (3) の 1 つ目。出所は [backlog](../backlog.md) の「backup に、PC の外のコピーと復元の実証が運用として設計されていない」の「導入の前の関門」の段の (3)（owner の決定、repo 外の回答台帳 TD-674・TD-679・TD-843。まとめは [decision-log](../decision-log.md) D-162 (4)）。D-114 の Revisit「更新・migration の前の保全を設計するとき」に当たる。起点は `11b1d793`（2026-11-27）。

起票の commit は本 packet と backlog の (3) の段の link だけ。owner の答え（2026-11-27、repo 外の回答台帳 TD-930〜TD-933）の後の計画の commit で、22 §25 と §16「回復」の手順 1・71 §71.9 に proposed・未実装の文を入れた（`scripts/plan-approve.sh` は review の後の lane の commit を packet だけに限るので、設計正本の文は計画の review の前に置く）。計画の review round 1 の後の是正（owner の決定 TD-956・TD-957 と TD-930 の再確認、2026-11-28）と closure round 2 の後の是正（2026-11-28）も同じく計画の commit。`src-tauri/**`・proposed の注記の外し・22 §3.2 と §12.4 の文・手順書の文は計画の固定の後の実装の commit。

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

- 版が 1 以上でアプリの最大より小さい DB を新しい版のアプリで起動すると、最初の migration を当てる前に、その版のままの使える控え（71 §71.3 の名前、§71.12 の「使える控え」の検査に合格）が保存先にできる。これは今ある版と、#307・#306 などがこれから足す版の全部の migration の前に効く。
- 控えの中には PLU の照合の日時（`app_settings` の key `plu_register_snapshot_at`）を残さない（owner の決定 TD-956）。どの戻し方（アプリの中の復元でも、管理者が file のまま旧版へ戻しても）でも、SD を読み直すまで PLU の書出しが止まる。
- 控えを作れなければ、migration を 1 つも当てず（控えの段は DB の版と論理内容を変えない）起動を止め、失敗した段階（今回の起動では migration を始めていないこと）・試した保存先と理由・直し方を名指す文を窓の前の dialog に出す。
- 版がアプリと同じ DB、DB が無い初回の起動（版 0）、アプリより新しい DB（MNT-03-D11）では控えを作らず、今と同じに動く。

### 失敗定義

- 控えが migration の後の版で作られる、または 1 つでも migration を当ててから作られる。
- 控えを作れないのに migration を当てる・起動を続ける。止まったときに DB の版・表・行が変わる。
- 控えに `plu_register_snapshot_at` が残る。今の DB の `plu_register_snapshot_at` を消す（消すのは控えの中だけ）。消した後の bytes を検査せずに公開する。
- 自動 backup の設定（`backup_enabled`）が `'0'` だと控えを作らない（owner の決定は「作れなければ止める」で、設定で外せる形にしない）。
- 保存先の設定を読めないときに、既定の保存先を試さずに止まる（TD-930）。
- 止まったときの文が、試していない直し方（前の版を入れ直せば開ける等）を保証する。
- 版が同じ DB で起動のたびに控えができる。初回の起動・新しすぎる DB で何かを書く。
- 復元（71 §71.7）・取込み（22 §12）・reconcile（MNT-01-D5）の順と振舞いが変わる。復元の中の migrate の前に控えができる。

### 非目的

- 店の人が流せる更新の手順（(3) の 2 つ目）。本 lane の merge の後の別の lane（owner の決定 TD-932、[backlog](../backlog.md) の (3) の段）。本 lane は手順書の今の文のうち、この lane で事実が変わる文だけを直す（Scope）。
- 自動更新（backlog の保留「アプリの自動更新（updater）」）。
- 復元の中の migration（71 §71.7。戻す元の控えの file は残り、今の DB は復元の前の事前の backup〈D-032〉が守る）。
- 版 0 の既存の file（`schema_versions` の無い DB）の扱い。今のまま。
- PLU の精算回数（`plu_register_snapshot_settlement_no`）の扱い。控えの中の値はそのまま残す。#315（D-176）が復元の直前の精算回数を引き継ぐのはアプリの中の復元（`restore_backup` の経路）だけで、file のまま戻す道（この lane の控えでも、旧版で作った backup でも）には効かない。file のまま戻した後の古い SD の照合は、管理者の復旧の手順（手順書の lane、[backlog](../backlog.md) の (3) の段）で、戻した後は SD を読み直すまで書き出さない・戻すのは起動が作った更新の前の控えを既定にする、として扱う。
- 旧版で作った backup（手動・自動）の中の照合の日時。この lane が消すのは自分が作る控えの中だけで、22 §16「回復」の手順 1 の旧版で作った backup を file のまま戻す道は照合の日時を持つので、SD を読み直すまで書き出さない手順が要る（上と同じ手順書の lane）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 確かめた事実（2026-11-27、`11b1d793` の code の読み。F7 は 2026-11-28）

- F1 起動の順: `run` の setup は、診断ログ（`src-tauri/src/lib.rs:1474`）→ `prepare_database`（`:1489`）→ 操作ログの掃除（`:1504`）→ `run_startup_auto_backup`（`:1509`）。`prepare_database_with_init`（`:150-199`）の中は、`reconcile_restore`（`:162`）→ 新 DB が無ければ legacy の取込み（`:165-183`）→ `init`（= `db::init_database`、`:188`）→ `complete_reconciled_restore`（`:194`）。`prepare_database_with_init` に test だけの `init` を渡す既存の test は 3 本（`lib.rs:874`・`:1020`・`:1128`。どれも `Err` を返すか panic する）。
- F2 migration の入口: `init_database`（`src-tauri/src/db/mod.rs:189`）は open の後 `configure_database`（`:206`）で PRAGMA（`foreign_keys`・`journal_mode = WAL`・`busy_timeout`）の後に `migration::migrate`（`:219`）を呼ぶ。`migrate` は `read_current_version_without_ddl` で版を読み、アプリの最大より新しければ書かずに `SchemaNewerThanApp`、そうでなければ migration ごとに BEGIN〜COMMIT する（`src-tauri/src/db/migration.rs:217-251`）。
- F3 今、migration の前に控えは作られない: 起動の自動 backup（`run_startup_auto_backup`、`lib.rs:125-136`）は `prepare_database` の後で、migration は全部当たった後。しかも `check_auto_backup` は `backup_enabled` が `'1'` で今日の正式名の backup が無いときだけ作る（`src-tauri/src/mnt/backup.rs:825-860`）。手順書もこれを前提にし、更新の前に旧版で手動の backup を作る（[msi-distribution-runbook](../msi-distribution-runbook.md) の流れ 3、[migration-recovery-runbook](../migration-recovery-runbook.md) の手順 1、22 §16「回復」の手順 1、71 §71.9）。
- F4 控えが作れないとき: 起動の自動 backup の失敗は `tracing::warn!` だけで起動を続ける（`lib.rs:125-136`、MNT-01-D2）。migration の前の止める所は無い。
- F5 使える既存の手段: `create_backup`（`backup.rs:623`）は書く先の固定（MNT-01-D10）・作業名への `VACUUM INTO`（手順 4）・§71.12 の検査（手順 4a）・上書きしない公開（手順 4b）・操作ログ `backup_create` までを持ち、検査は版がアプリ以下の DB を受ける（`backup.rs:521` の `inspect_backup`。`immutable=1` で開くので、作業名の横の `-wal` は読まない）。`create_backup_with`（private、`create_backup` と test が呼ぶ）が本体。`resolve_backup_dir`（`backup.rs:610`）は `app_settings` の `backup_path` を `String` として読み、DB の error を `Err` で返す（review round 1 の合成の版 1 の DB で、`backup_path` だけを BLOB にすると読めないが、同じ DB の `VACUUM INTO` と検査は通った）。test だけの読取りの失敗の注入（`SETTING_READ_FAILURE`、`backup.rs:91`）がある。`app_settings` と `operation_logs` は v1 から形が同じ（`src-tauri/src/db/schema_v1.rs:232-246`、後の版で作り直していない。`rg -n operation_logs src-tauri/src/db/schema_v*.rs` は v1 の定義と v9 の test の読みだけ）。窓の前の dialog は `show_pre_window_fatal`（`lib.rs:205`、MNT-03-D4）。
- F6 呼出し側: `init_database` を runtime で呼ぶのは起動だけ（`rg -n 'init_database\(' src-tauri/src` の test の外は `lib.rs` の `prepare_database_with`）。復元は `open_existing_database`（`src-tauri/src/mnt/restore.rs:516`）で、同じ `configure_database` を通る。既存の `test_restore_backup_req901_runs_migration`（`backup.rs:1487`）は最新の版の DB から作った控えを戻すので、復元の経路に控えの作成が混ざっても検出しない（review round 1）。
- F7 PLU の照合の日時: key `plu_register_snapshot_at` は今の main の書出し（`src-tauri/src/biz/plu_export_service.rs:275`）が書き、PR #307 の v16 の migration（22 §24 の手順 6）と復元の手順 6a（71 §71.7、MNT-01-D13）が消す。#307 の書出しは key が無ければ `register_snapshot_required` で止める。review round 1 の合成の写しで、精算 40 の控えを file のまま戻すと、日時と精算回数が 40 の時点に戻った（Codex P1）。

## Scope

順: PR #307（PLU の runtime、`db/mod.rs`・`lib.rs`・migration・`mnt/restore.rs` を触る）の merge と `origin/main` の同期の後に実装を始める。

runtime（計画の固定の後）:

- `src-tauri/src/db/mod.rs`: `configure_database` の PRAGMA の段を `pub` の `open_configured_database`（仮の名。open と PRAGMA だけ）として切り出す。`init_database` は `open_configured_database` + `migrate` + 初期化完了のログで、振舞いは変えない。`open_existing_database` も今のまま（復元は控えを作らない、非目的）。
- `src-tauri/src/mnt/backup.rs`: `pub fn create_pre_migration_backup(conn, backup_dir)` を足し、`create_backup_with` を呼ぶ。`create_backup_with` は手順 4（作業名への `VACUUM INTO`）と手順 4a（検査）の間に、この呼出しのときだけ作業名の中の key `plu_register_snapshot_at` を消す段を持つ（TD-956。bool の引数 1 つ。`create_backup` と既存の test の呼出しは「消さない」を渡し、振舞いは変えない）。消す段: 作業名を別の接続で開き、`journal_mode = DELETE` で 1 つの TX に `DELETE FROM app_settings WHERE key = 'plu_register_snapshot_at'` を当て、同じ TX で key が無いことを `SELECT` で確かめてから COMMIT して閉じる（key が無ければ何も消えない。版で分けない。v16 より前の控えでも在れば消す。trigger の `RAISE(IGNORE)` などで `DELETE` が error なしに 0 件で終わり key が残れば「控えを作れなかった」にする）。閉じた後に作業名の横に `-wal`・`-journal` が残らないことを確かめる。失敗は今の手順 4 の失敗と同じく作業名を消して `Err`。
  - 消す途中で process が終わると作業名の横に `{作業名}-journal` が残り、今の `remove_stale_partials`（`.partial` で終わる名だけを拾う）は消さない（Codex P3、closure round 2）。実装の commit で、規約名の作業名を消すときに対応する `-journal` も消すか、`ponytail:` comment に限界（残るのは正式名の公開の前の小さな file だけ）と広げる条件（店で残りが見つかったとき）を書く。
  - 公開の後の `sync_parent` だけの失敗（検査済みの正式名が残って `Err`、§71.4 のエラーハンドリング）は、起動の控えでも作れなかったのと同じに扱う（既定へ移る・止める。設定の保存先に残った正式名は普通の backup）。実装の commit で扱いを変える（残った正式名を控えとみなす）か、`ponytail:` comment に限界（使える控えがあっても既定へ移る・止まる）と広げる条件（店でこの失敗が起きたとき）を書く。
- `src-tauri/src/lib.rs`:
  - `prepare_database_with` は `init` に `db::open_configured_database` を渡す。版の読み・控え・migrate は `prepare_database_with_init` の本体の `init` の後に置く（F1 の既存の 3 本の `init` の closure は直さない）: `read_current_version_without_ddl` → 版が `0 < 版 < app_max_version()` のときだけ控えを作る → `migration::migrate` → 初期化完了のログ → `complete_reconciled_restore`。`migrate` の `SchemaNewerThanApp` は今と同じ variant へ写す。控えは `resolve_backup_dir` と `create_pre_migration_backup` を呼ぶだけ。`backup_enabled` を見ない。`cleanup_old_backups` は呼ばない。
  - 保存先（owner の決定 TD-930、2026-11-28 の再確認）: `resolve_backup_dir` の保存先で控えが `Err` なら、その保存先が既定の `{app_data}/backups` と違うときだけ既定でもう 1 回作る（1 つ目の `Err` は診断ログに `warn`）。`resolve_backup_dir` が DB の error（設定を読めない）のときも、既定で 1 回作る（error は `warn`。起動の自動 backup の MNT-01-D2「読めない設定で既定へ fallback しない」は今のまま）。既定でも `Err` なら止める。
  - `StartupDatabaseError` に控えの失敗の variant を 1 つ足し、`operator_message` に Design Readiness の「文」を置く（末尾に今の variant と同じく `\n{details}`）。variant は試した保存先ごとに「場所と理由」を持つ（設定を読めなかったときは「設定の保存先（設定を読めませんでした）」を 1 つ目に置く）。理由は `create_backup_with` の `Err` の文で、`VACUUM INTO` の失敗では SQLite の英語の文（例: `database or disk is full`）が混ざり、店の人には「空き不足」に当たるか分からない（Claude P3、closure round 2）。実装の commit で、SQLite の空き不足（`SQLITE_FULL`）を日本語の「保存先の空きが足りません」と読める文にするか、`ponytail:` comment に限界と広げる条件（店で理由を読み違える事例が出たとき）を書く。
  - test（Test Plan の T1〜T8）。入力を直し期待を足す既存の test は 1 本: `test_startup_req903_vb_stop_after_earlier_commit_backup_path_readable_from_copy` は版 7 の DB の `backup_path` に `D:\合成の保存先` を入れて起動する。新しい振舞いでは控えがまずその path（Windows 以外では作業フォルダの中の相対の folder）へ書かれ、作れなければ `{app_data}/backups` に書かれ、後の `file_hashes(&app_data)` が `backups` の directory を `std::fs::read` で読んで落ちる。`backup_path` を一時 directory の下で `app_data` の外の path（名に日本語を残す: `{一時 directory}/合成の保存先`）にし（控えはそこにでき、`app_data` に `backups` ができない）、写しから読む値の期待をその path に合わせる。期待の意味（`DatabaseInit` と vB の文、写しから設定の値が取れる、版が `VB_VERSION - 1`、元の file が変わらない）は変えず、T8 の期待を足す。
  - 意図した簡略化は `ponytail:` comment に書く（下の Design Readiness の「抜け道」の 2 つ目）。
- `docs/function-design/22-mnt-migration.md`: §25 の MNT-03-D26 と §16「回復」の手順 1 の注は計画の commit で proposed・未実装として入れた。実装の commit で注記を外し、手順 1 の「起動時の自動の backup は migrate の後に作られる」の文を事実に合わせる。書き換えた手順 1 は、戻すのは起動が作った更新の前の控え（照合の日時を持たない）を既定にし、旧版で作った backup（照合の日時を持つ）は控えが無いときの道とする。§3.2 の MNT-03-D11 の「起動の文言」の段に、起動では版の読み → 控え → `migrate` の順になることを 1 文足し、§12.4 の文言の表に新しい variant の行（下の「文」）を足す。`open_configured_database` と `create_pre_migration_backup` の名を §25 に書く（`src-tauri/tests/design_compliance_test.rs` の突合）。
- `docs/function-design/71-mnt-backup.md` §71.9: 起動の順の migration の前の控えは計画の commit で proposed・未実装として入れた。実装の commit で注記を外す。
- `docs/migration-recovery-runbook.md` の手順 1 と `docs/msi-distribution-runbook.md` の流れ 3 の前文（実装の commit。今のアプリの振舞いを書く手順書なので、計画の commit では直さない）: 同じ「自動の backup は migration の後」の文を直し、「USB メモリなどを差し直した後は、今ある手動のバックアップ（バックアップ画面の「今すぐバックアップを作成」）で今のデータを保存できます。更新の前の控えは既定の場所（`{app_data}/backups`）にあります」の趣旨の 1 文を足す（TD-957。手動のバックアップが作るのは今の DB の新しい版の控えで、更新の前の控えを USB へ写すものではない。Codex P2-4、closure round 2）。migration-recovery-runbook の手順 1 も 22 §16 と同じく、戻すのは起動が作った更新の前の控えを既定にする。手順の作り直し（file のまま戻した後は SD を読み直すまで書き出さない、を含む）は本 lane の merge の後の手順書の lane。
- `docs/function-design/90-traceability.md`: generator で作り直す（手で編集しない）。
- `docs/backlog.md`: 実装の commit で (3) の段の「アプリが migration の前に控えを作り、作れなければ止める lane」を済みにする（関門の行は実装の PR で外す）。(3) の段の手順書の lane が残ることと owner の所感（TD-932）、その lane が持つ file のまま戻した後の照合の扱い（上の非目的）は計画の commit で書いた。

Non-scope: `src-tauri/src/db/migration.rs`（呼ぶだけ）・`schema_v*.rs`・`mnt/restore.rs`・`mnt/offsite.rs`・`cmd/**`・`src/**`・bindings、`Plans.md`、`docs/decision-log.md`（D-173 は足さない。owner の決定は D-162 (4) にあり、実装の判断は子の ID の MNT-03-D26 に置く）。

path の所有（並走の lane との突き合わせ）:

| file | 本 lane が触る所 | 重なる lane |
|---|---|---|
| `src-tauri/src/lib.rs` | `StartupDatabaseError`・`prepare_database_with`・`prepare_database_with_init` とその test の module | #307（同じ所を触りうる。#307 の後に始める）、#305（merge 済み） |
| `src-tauri/src/db/mod.rs` | `configure_database`・`init_database` | #307、#306（`mod` の行） |
| `src-tauri/src/mnt/backup.rs` | `create_backup_with` の手順 4 と 4a の間、`create_pre_migration_backup`、test の module | #315（同じ key `plu_register_snapshot_at` を復元の側〈`mnt/restore.rs` の手順 6a〉で扱う。本 lane は控えを作る側で消し、#315 はアプリの中の復元〈`restore_backup` の経路〉の後に精算回数を引き継ぐ。触る file は別。どちらも key を消すだけで、精算回数の key は本 lane が触らない。file のまま戻す道にはどちらの lane の引き継ぎも効かず、手順書の lane が持つ〈非目的〉） |
| 22 | 末尾の §25・§16「回復」の手順 1 の 1 文・§3.2 の 1 文・§12.4 の表の 1 行 | #307（§24・MNT-03-D25 を持つ。本 lane は §25・MNT-03-D26 を取る）、#306（同じく #307 の次の節と ID を見込む。後に merge する側が節・ID・test 名・comment を取り直す） |
| 71 | §71.9 だけ | #307（§71.7 の 6a）、#315、1159（§71.13） |
| `docs/msi-distribution-runbook.md`・`docs/migration-recovery-runbook.md` | 上の文 | 手順書の lane（本 lane の merge の後）、#312（復元の演習の手順書） |

## Acceptance Criteria

- AC1（全部の migration の前）: 版 `v` を 1 から `app_max_version() - 1` まで 1 つずつ回し（数は `app_max_version()` から導き、版の数を書かない）、版 `v` まで当てた DB を起動の DB の準備に渡すと、保存先に正式名の控えが 1 つでき、その控えの `inspect_backup` が `schema_version = v`・`quick_check_ok = true`、起動の後の DB の版は `app_max_version()`。#307・#306 の版はこの回し方に入る。版の 1 例（`app_max_version() - 1`）の DB には `plu_register_snapshot_at` と `plu_register_snapshot_settlement_no` を入れ、通常の起動でできた正式名の控えに `plu_register_snapshot_at` が無く精算回数が元の値で、起動の後の DB には日時が残る（起動側の呼出しを `create_backup` に戻すと red。Codex P2-2、closure round 2）。
- AC2（既定へ移る・両方だめなら止める）: 版 `app_max_version() - 1` の DB で、(i) 設定の保存先（`backup_path`）が file で folder を作れないと、`{app_data}/backups` に AC1 と同じ控えができて migrate が当たり、設定の保存先に正式名も作業名も残らない。(ii) 設定の保存先と `{app_data}/backups` の両方が file だと（`backup_path` が未設定で `{app_data}/backups` が file の場合も）、起動は新しい variant の error を返し、`operator_message` が Design Readiness の「文」の固定の部分と、試した保存先ごとの path と理由を含む。DB の `MAX(version)`・`sqlite_master` の `sql` の一覧・各表の行の数が前後で同じで、どちらの保存先にも正式名も作業名も残らない。(iii) `resolve_backup_dir` の DB の error（`SETTING_READ_FAILURE` の注入）では、既定に控えができて migrate が当たる。既定も file なら (ii) と同じく止まり、文の試した保存先の 1 つ目が「設定の保存先（設定を読めませんでした）」。
- AC3（作らない場合）: 版が `app_max_version()` の DB の起動、DB の無い初回の起動、版が `app_max_version() + 1` の DB の起動のどれでも、控えの準備の段は file を作らない（新しすぎる DB は今どおり `SchemaNewerThanApp` で、保存先も DB も変わらない）。
- AC4: `backup_enabled = '0'` の DB でも AC1 の控えができる。
- AC5（順）: legacy の取込み（作業フォルダの旧 DB、版 `app_max_version() - 1`）の後と、reconcile が退避から戻した後の DB でも、控えは migration の前の版で作られる。reconcile・取込みが失敗した起動では控えを作らない（今の test の panic の注入の形）。
- AC6: 既存の test が期待値を変えずに全部 PASS する（`lib.rs` の `test_startup_`・`db::migration`・`mnt::backup`・`mnt::restore`）。入力を直し期待を足すのは Scope に名指した 1 本だけ。`git diff --stat origin/main...HEAD` に `src/**`・`src-tauri/src/cmd/**`・`src-tauri/src/db/migration.rs`・`src-tauri/src/db/schema_v*.rs`・`src-tauri/src/mnt/restore.rs` が出ない。
- AC7（mutation）: 実装に注入して当たる test が red になることを確かめて PR に書く: (m1) 控えを migrate の後に作る → T1、(m2) 条件を `版 <= app_max` にする → T3、(m3) 控えの失敗を warn にして続ける → T2、(m4) 版 0 でも作る → T3、(m5) `backup_enabled` を見る → T4、(m6) 既定へ移らずに止める → T7、(m7) `resolve_backup_dir` の DB の error で既定を試さずに止める → T7、(m8) 控えから key を消さない → T9、(m9) key を今の DB から消す → T9、(m10) key を検査の後・公開の前に消す（検査していない bytes を公開する）→ T9（`create_backup_with` の source の中で消す段が `inspect_backup(&partial)` より前にあることの順の検査。既存の `test_startup_req901_b10_guard_registration_precedes_mutation_setup` と同じ形）、(m11) migration の失敗の後に最初の控えを消す → T8、(m12) `open_existing_database` の側に控えを入れる → T10、(m13) 消した後の key の不在の確認を外す → T9（`RAISE(IGNORE)` の trigger）、(m14) 起動側の呼出しを `create_backup` に戻す → T1（照合の日時を入れた 1 例）。
- AC8: `bash scripts/doc-consistency-check.sh` と `--target plan docs/plans/2026-11-27-migration-backup.md` が ERROR 0、`bash scripts/check-workflow-git.sh` が PASS、`cd src-tauri && cargo run --bin generate_traceability -- --check` が PASS。Rust の検証は `docs/DEV_WORKFLOW.md` の Verification Gates（hosted CI の `Merge gate`）。
- AC9（控えの中の照合の日時、TD-956）: `plu_register_snapshot_at` と `plu_register_snapshot_settlement_no` を持つ DB で `create_pre_migration_backup` を呼ぶと、正式名の控えに `plu_register_snapshot_at` が無く、`plu_register_snapshot_settlement_no` は元の値のまま、`inspect_backup` に合格し、正式名の横に `-wal`・`-journal` が無い。呼んだ接続の DB には `plu_register_snapshot_at` が元の値で残る。key の無い DB でも同じく合格する。作業名の中の `DELETE` を拒む trigger を置いた DB では、`RAISE(ABORT)`（error を返す）でも `RAISE(IGNORE)`（error なしに 0 件で終わり key が残る。closure round 2 の Codex の再現）でも `Err` で、正式名も作業名も残らない。`create_backup` は key を残す（既存の振舞い）。
- AC10（途中で止まったときの最初の控え）: AC6 の 1 本（版 7、商品 `R-OLD`、vB で止まる）で、止まった後に設定の保存先の正式名の控えが 1 つあり、`inspect_backup` が `schema_version = 7`・`quick_check_ok = true` で、控えを開くと商品 `R-OLD` がある。既存の「本体の写しの版は `VB_VERSION - 1`」の期待は残す（旧版へ戻せるのは控えのほうで、本体ではない）。
- AC11（復元の経路）: 版 `app_max_version() - 1` まで当てた合成の DB から `create_backup` で作った控えを `mnt::backup::restore_backup` に渡すと、戻った接続の版が `app_max_version()` で、保存先と `{app_data}/backups` の正式名の数が復元の前と同じ（復元の中の migrate の前に起動の控えを作らない。D-032 の復元の前の backup は command の側で、この test は通らない）。

## Design Readiness

- 引用する設計正本（節まで）: 22 §3.2（MNT-03-D1・D11）・§12.4（MNT-03-D4・D5〜D8 の文言の表）・§16「回復」・§24（#307 の v16 の手順 6）、71 §71.3・§71.4（MNT-01-D7・D10）・§71.5（MNT-01-D3）・§71.7（MNT-01-D13）・§71.8・§71.9（MNT-01-D2）・§71.11・§71.12、33 §16.8（PLU の書出しの gate）、D-032・D-114・D-132・D-162 (4)・D-176（#315）、`docs/msi-distribution-runbook.md` の流れ 3〜5、`docs/migration-recovery-runbook.md`。
- 必要な設計成果物: updated in this PR（22 §25 の MNT-03-D26 と §16「回復」の手順 1 の注、71 §71.9 の注を計画の commit で proposed・未実装として入れた。実装の commit で注記を外し、22 §3.2 の 1 文と §12.4 の表の 1 行を足す）。DB・migration・DTO・画面は不要。
- owner の決定（repo 外の回答台帳）: 保存先は、設定の保存先で作れなければ既定の `{app_data}/backups` で作り、両方だめなら止める（TD-930、2026-11-27。設定を読めないときも既定で試す、2026-11-28 の再確認）。控えは普通の backup と同じ名前と folder で、保持日数で消え、PC の外の控えにも写る（TD-931）。既定の folder へ移った控えが復元の一覧・掃除・PC の外の控えに入らないのは例外として受け入れ、後で USB を差し直して保存したいときは今ある手動のバックアップで保存できる、と一言入れる（TD-957、2026-11-28。手動のバックアップが保存するのは今のデータで、文はそのとおりに書く）。控えから PLU の照合の日時を消す（TD-956、2026-11-28）。店の人が流せる更新の手順は本 lane の merge の後の別の lane（TD-932）。L3 は置かない（TD-933。下の Test Plan）。
- 照合の日時を消す位置（TD-956 の「検査の前か後か」）: 消した後に検査する（作業名への `VACUUM INTO` → 作業名の中の key を消す → §71.12 の検査 → 公開）。検査の後に消すと、公開する bytes が検査を通っていない（MNT-01-D7 の「確かめてから公開」が崩れる）。消す段の失敗は作れなかったのと同じに扱う（作業名を消し、既定へ移る・止める）。`journal_mode = DELETE` で閉じるのは、検査が `immutable=1` で `-wal` を読まないことと、公開が 1 つの file だけを動かすことに合わせるため。
- plan にしかない durable な判断の昇格先: MNT-03-D26（上の owner の決定・棄却した案・見直し契機。22 §25）。
- 前提・制約と延期した design gap: 窓の前の起動で控えを作るので、DB の大きさの分だけ窓が出るまで待つ（Contract Probe の P1）。店の人の更新の手順は上の別の lane。
- 絶対保証（cannot happen / always happens）の例外と抜け道:
  - 控えは「その起動の最初の migration の前」の版。同じ起動で後の migration が失敗して止まり、次の起動で残りを当てるとき、次の起動の控えは途中の版になる（最初の起動の控えは保存先に残る。AC10。今の手順書の流れ 5 の「場面 2」と同じ）。
  - 失敗が続く起動（migration が止まる DB を何度も起動する）では、起動のたびに PC の中の保存先に控えが 1 つ増える（保持日数〈MNT-01-D3〉で消えるまで残る）。同じ日・同じ版の控えがあれば作らない形は足さない（D-147。広げる条件: 運用で再起動の繰返しが起きたとき）。
  - 控えは正式名なので、§71.8 の判定（手順 3 の今日の backup、手順 8 の定時の後の backup）に数える。定時（`backup_time`）より前に作れば、その日の起動の自動 backup（手順 4）は作られず、定時の後の起動で手順 9 が作る。定時より後に作れば、手順 8 がその日の定時の分も済んだとみなし、migration の後の版の自動 backup はその日のうちの起動では作られない（次の日の起動の手順 4 が作る）。既定の folder へ移った控えは、設定の保存先が別なので数えない。
  - 控えの後に `create_backup_with` が旧版の DB へ操作ログ `backup_create` を 1 行書く（F5。控えには入らない）。
  - 物理の書込み: PRAGMA（`journal_mode = WAL`）は今どおり控えの前に当たる（MNT-03-D11 の「物理的な書込み」と同じで、論理内容を変えない）。
  - 既定の folder へ移った控え（設定の保存先が別のとき）は、復元の画面の一覧・保持日数の掃除・PC の外の控えのどれにも入らない（どれも設定の保存先だけを見る。`resolve_backup_dir` の呼出し側）。owner はこれを例外として受け入れた（TD-957）。USB メモリなどを差し直した後は、今ある手動のバックアップ（バックアップ画面の「今すぐバックアップを作成」）で今のデータを保存できる。更新の前の控えは既定の場所に残る（手順書の 1 文、Scope。手動のバックアップは更新の前の控えを USB へ写さない）。旧版へ戻すときにこの控えを拾うのは管理者の手順（22 §16「回復」、手順書の lane）。
  - 控えの中の精算回数（`plu_register_snapshot_settlement_no`）は消さない。file のまま戻した後は照合の日時が無いので書出しの前に SD の読み直しが要るが、読んだ SD が戻した時点より新しく、戻す前の時点より古い（例: 精算 40 の控えへ戻し、精算 42 の古い SD を読む。戻す前は 45）と照合が通る。#315 の精算回数の引き継ぎは `restore_backup` の経路だけで、この道には効かない。管理者の復旧の手順（手順書の lane、backlog の (3) の段）で扱う（非目的）。
- 文（失敗の dialog の固定の部分。計画の review で確かめる。T6 が固定する）: 「更新の前の控え（バックアップ）を作れなかったため、起動を中止しました。今回の起動では、データの形を新しい版に切り替える作業を始めていません。試した保存先と理由: {保存先ごとに「場所（理由）」を ・ でつなぐ}。バックアップ用の USB メモリを外しているなら、その USB メモリを差してからもう一度起動してください。表示された理由が空き不足なら、その保存先の空きを作ってからもう一度起動してください。それ以外のときは、この画面の内容を添えて管理者へ連絡してください。」
  - 「この起動ではデータを変更していません」の保証は置かない（同じ起動で、控えの前に reconcile が退避していた DB を使用中の名前に戻していることがある。MNT-01-D5。Codex P2-6、closure round 2）。直し方は理由に合うものだけを条件つきで書く（「外付けなら差す」は、差してある媒体〈例: 設定の保存先のドライブ文字にレジの SD〉で同じ理由で止まり続ける。Codex P2-5、closure round 2）。
  - 前の版のアプリを入れ直す案内は置かない（前の起動で migration が途中まで当たっていると、前の版でも開けない。22 §16「回復」）。理由ごとに文を分ける code は足さず、理由は `create_backup_with` の `Err` の文（日本語。SQLite の英語の文が混ざる場合は Scope の `StartupDatabaseError` の 1 句）をそのまま出す（D-147。広げる条件: 店で理由を読み違える事例が出たとき）。
- Contract Probe: P1 店の大きさの DB での `VACUUM INTO` と検査の時間（窓が出るまでの待ち）。review round 1 で合成の商品 5,000 件・売上の明細 100,000 件の DB（10,780,672 bytes）を SQLite 3.46.1 で測り、`VACUUM INTO` 0.044 秒、`quick_check` 0.017 秒（1 回の測定。Windows の公開・sync と起動全体は含まない。店の DB の大きさは未確認）。待ちが問題になる大きさではないので、実装では測り直さない。
- 判定: ready（owner の未決は無い。TD-956・TD-957・TD-930 の再確認で review round 1 の判断事項は閉じた。closure round 2 の指摘は Coordinator の裁定で受け、owner の判断を要るものは無い）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| MNT-03-D26（新、proposed）migration の前の控え・既定へ移る・両方だめなら止める・照合の日時を消す | 22 §25 | `lib.rs` の DB の準備、`db/mod.rs` の分け方、`mnt/backup.rs` の `create_pre_migration_backup` | T1〜T5、T7〜T10 | 非対象（L3 なし、TD-933） |
| MNT-03-D11 新しすぎる DB は書く前に拒む | 22 §3.2 | 順を変えない（版の比べが控えより前） | T3、既存 `test_startup_req903_d11_newer_schema_stops_startup_with_own_message` | 非対象 |
| MNT-03-D1 migrate の TX・ROLLBACK | 22 §3.2 | 変えない | 既存 `db::migration` の test | 非対象 |
| MNT-03-D4・D5 起動の fail-closed と窓の前の dialog | 22 §12.4 | 新しい variant と文、表の 1 行 | T2、T6 | 非対象（L3 なし、TD-933） |
| 22 §12 legacy の取込み・MNT-01-D5 reconcile の順 | 22 §12、71 §71.7 | 変えない（控えはその後） | T5、既存 `test_startup_req903_m7_`・`m8_`・`req901_b6_`・`b9_` | 非対象 |
| 22 §16「回復」手順 1、作り直しの手順の起動の test | 22 §16 | 1 文を事実に合わせる | 既存 `test_startup_req903_` の 4 本（`test_startup_req903_vb_stop_after_earlier_commit_backup_path_readable_from_copy` は入力を直し T8 の期待を足す） | 非対象 |
| MNT-01-D7・D10 確かめてから公開・書く先の固定 | 71 §71.4 | `create_backup_with` に消す段を足す（検査の前） | T1（`inspect_backup` の合格）、T9、既存 `mnt::backup` | 非対象 |
| MNT-01-D13・D-176 照合の日時と精算回数 | 71 §71.7、33 §16.8 | 控えの中の `plu_register_snapshot_at` を消す（精算回数は触らない）。復元の 6a は変えない | T9 | 非対象（#315 と重なる、path の所有の表） |
| MNT-01-D2 `resolve_backup_dir` の error | 71 §71.9 | 起動の控えでは既定で試す（自動 backup は今の warn で skip のまま） | T7 | 非対象 |
| MNT-01-D3 保持・§71.8 今日の backup の判定 | 71 §71.5・§71.8 | 変えない（控えは普通の backup と同じ名前と folder、TD-931） | 既存 `test_check_auto_backup_req901_no_backup_today` | 非対象 |
| 71 §71.9 起動の順 | 71 §71.9 | 文を直す | 既存 `test_startup_req901_d2_resolve_error_warns_and_skips_auto_backup` | 非対象 |
| 71 §71.7 復元の中の migrate、D-032 | 71 §71.7 | 変えない（非目的） | T10、既存 `test_restore_backup_req901_runs_migration` | 非対象 |
| §71.11 PC の外の控え | 71 §71.11 | 変えない（設定の保存先の控えは最新の backup として写る。既定へ移った控えは写らない、TD-957） | 非対象 | 非対象 |
| D-162 (4)・D-114 Revisit | decision-log | 実装で当たる（本文は変えない） | 非対象 | 非対象 |
| 手順書の文 | msi-distribution-runbook 流れ 3、migration-recovery-runbook 手順 1 | 文を直し、TD-957 の 1 文を足す（実装の commit） | 非対象 | 非対象（手順は手順書の lane） |

## Test Plan

Rust の unit test（T1〜T8 は `src-tauri/src/lib.rs` の test の module、T9・T10 は `src-tauri/src/mnt/backup.rs` の test の module。一時 directory、`migrate_up_to` で版を作った合成の DB）。test の先頭の comment に `REQ-903 / MNT-03-D26` と節を書く。`## L3` 節は置かない（TD-933）: 成功の道は画面を変えず、失敗の dialog は保存先を書けなくする故障の注入が要る（[DEV_WORKFLOW](../DEV_WORKFLOW.md) の L3 Eligibility の (3)）ので下の自動 test で持ち、窓の前の dialog の見え方は MNT-03-D4 で確かめ済み。

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| MNT-03-D26 全部の migration の前 | 控えが migration の後か途中の版になる。新しく足した版で効かない | unit（T1、版を 1〜`app_max - 1` で回す。`app_max - 1` の 1 例は照合の日時と精算回数を入れる） | `test_startup_req903_d26_backup_before_every_migration` | 控えを migrate の後に作る（m1）、版を決め打ちする、起動側が `create_backup` を呼ぶ（m14。正式名の控えに照合の日時が残る） |
| MNT-03-D26 両方だめなら止める | 作れずに migration を当てる・起動を続ける・DB が変わる | unit（T2、表: 設定の保存先と `{app_data}/backups` の両方を file にする、`backup_path` 未設定で `{app_data}/backups` を file にする、`resolve_backup_dir` の error で `{app_data}/backups` を file にする） | `test_startup_req903_d26_backup_failure_stops_before_migration` | warn で続ける（m3）、止まる前に 1 つでも migrate する、文に試した保存先と理由が無い、設定を読めない case の 1 つ目の場所の文が無い |
| MNT-03-D26 既定へ移る | 設定の保存先で作れないと止まる（抜いた USB メモリで起動できない）、設定を読めないと止まる、既定へ移った後に設定の保存先に作業名が残る | unit（T7、表: 設定の保存先を file にする、`resolve_backup_dir` の error） | `test_startup_req903_d26_falls_back_to_default_dir` | 既定へ移らずに止める（m6・m7）、既定の控えが §71.12 の検査に合格しない、移った後に migrate しない |
| MNT-03-D26 作らない場合 | 起動のたび・初回・新しすぎる DB で書く | unit（T3、表: 版 = app_max、DB なし、版 = app_max + 1） | `test_startup_req903_d26_no_backup_when_nothing_to_migrate` | 条件を `<= app_max` にする（m2）、版 0 で作る（m4）、版の比べより前に作る |
| MNT-03-D26 設定に依らない | `backup_enabled = '0'` で作らない | unit（T4） | `test_startup_req903_d26_backup_ignores_backup_enabled` | `backup_enabled` を見る（m5） |
| MNT-03-D26・§12・MNT-01-D5 の順 | 取込み・reconcile の後の DB で控えが無い・後の版になる。reconcile・取込みの失敗で控えを作る | unit（T5） | `test_startup_req903_d26_backup_after_legacy_import_and_reconcile` | 控えを取込み・reconcile より前に置く、取込みの後の DB を見ない |
| MNT-03-D4・D5 文 | 原因・直し方の無い文、試していない直し方を保証する、variant が `DatabaseInit` に潰れる | unit（T6） | `test_startup_req903_d26_operator_message_names_stage_place_and_fix` | 文の固定の部分を変える（「前の版のアプリを入れ直す」・「この起動ではデータを変更していません」・条件の無い「外付けなら差して」を戻す。「バックアップ用の USB メモリを外しているなら」「理由が空き不足なら」「それ以外のときは」「データの形を新しい版に切り替える作業を始めていません」を固定する）、`\n{details}` を落とす |
| MNT-03-D26 途中で止まったときの最初の控え（AC10） | migration の失敗の処理が最初の控えを消す・後の版にする | unit（T8、既存の 1 本に期待を足す） | `test_startup_req903_vb_stop_after_earlier_commit_backup_path_readable_from_copy` | migration の失敗の後に最初の控えを消す（m11）、控えを後の migration の後に作る |
| MNT-03-D26 照合の日時（AC9、TD-956） | 控えに照合の日時が残る、今の DB の日時を消す、消した bytes を検査せずに公開する、消す段の失敗で作業名か正式名が残る | unit（T9、表: key あり、key なし、`DELETE` を拒む trigger〈`RAISE(ABORT)`・`RAISE(IGNORE)`〉、`create_backup` の対照） | `test_backup_req903_d26_pre_migration_backup_clears_snapshot_at` | key を消さない（m8）、今の DB から消す（m9）、検査の後に消す（m10）、消した後の不在を確かめない（m13）、`-wal` を残す |
| 71 §71.7 復元の経路（AC11） | 復元の中の migrate の前に起動の控えを作る | unit（T10、最大版の 1 つ前の合成の控えを `restore_backup` へ渡す） | `test_restore_backup_req903_d26_no_pre_migration_backup_on_restore` | `open_existing_database` の側に控えを入れる（m12） |
| 既存の契約 | 既存の振舞いが変わる | unit（既存） | Contract Ledger の既存の test | 期待値を変えないと通らない変更を入れる |

状態の遷移: initial（版 `v` の DB）→ 控え（success / failure）→ migrate → 起動の後の版。failure は T2（止まった後に同じ DB で保存先を直して起動し直すと控えができて migrate が当たる retry を同じ test に入れる）。migrate の failure は T8（最初の控えが残る）。restart は T1 の後にもう一度起動して控えが増えない（T3 の版 = app_max の case）。mutation は AC7。比べるのは file の bytes でなく版・`sqlite_master`・行の数（`journal_mode = WAL` の PRAGMA が header を書きうるため）。

## Recovery

- 本 lane が足すのは控えの file の作成（控えの中の key を 1 つ消すことを含む）と、作れないときの起動の停止だけで、今の DB の表・行・版を変えない。
- 誤って止まる（作れるはずの控えで止まる）とき: その起動では migration を始めていないので、DB は控えの段の前と同じ（同じ起動で reconcile が退避から戻していれば、戻した後の DB）。前の起動で migration が途中まで当たっていなければ前の版の MSI でそのまま開ける（[msi-distribution-runbook](../msi-distribution-runbook.md) の流れ 5 の 1、M6）。途中まで当たっていれば前の版では開けないので、修正版で前へ進めるか、22 §16「回復」の手順で更新の前の backup に戻す。
- 誤って止めない（控えなしに migration を当てる）とき: 今の main と同じ状態に戻るだけで、旧版へ戻すには流れ 3 の手動の backup が要る（手順書は手順書の lane まで今のまま流れ 3 を持つ）。
- 控えから照合の日時を消し損ねた（実装の欠陥で残った）控えを戻すとき: 今の main の手動の backup を戻すのと同じ状態で、アプリの中の復元なら #307 の復元の 6a が消す。file のまま戻す道は管理者の手順で SD を読み直してから書き出す（手順書の lane）。
- 旧版で作った backup（手動・自動）を file のまま戻すとき（22 §16「回復」の手順 1 の今の道）: その backup は照合の日時を持つので、戻した後に SD を読み直さずに書き出せる。SD を読み直すまで書き出さない手順が要る（手順書の lane、[backlog](../backlog.md) の (3) の段）。この lane の実装の commit で書き換える手順 1 は、戻すのは起動が作った更新の前の控えを既定にする（Scope）。
- code の戻し方: 修正の PR を既定にし、本 lane の PR の revert も入れられる（DB・migration・状態の file の形を変えないので、版を戻しても DB と控えはそのまま読める）。
- 作った控えは普通の backup で、保持日数で消える（TD-931）。既定へ移った控えは掃除の外で残る（Design Readiness の抜け道、TD-957）。消し方を足さない。

## Data Safety

- 書くもの: 保存先の作業名と正式名の控え（`create_backup_with` の既存の経路）、作業名の中の key `plu_register_snapshot_at` の削除（検査と公開の前）、旧版の DB への操作ログ `backup_create` 1 行。消すもの: 前回の中断の作業名（`create_backup_with` の手順 3 の既存の経路）と、失敗した作業名だけ。`cleanup_old_backups` を呼ばない。今の DB の照合の日時は消さない。
- 止まったとき: DB の論理内容と版は変えない。作業名は `create_backup_with` が消し、消せなければ次の作成が消す（MNT-01-D7）。
- commit しないもの: 実の店の DB・backup・診断ログ。test の入力は一時 directory の合成の DB だけで、tracked の fixture を足さない。
- owner の決定と店の事実は repo 外の回答台帳に残し、repo には TD 番号と要旨だけを書く。
