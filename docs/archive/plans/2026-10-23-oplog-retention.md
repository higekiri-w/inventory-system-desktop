# Plan Packet: 操作ログの保持を 7 年程度に延ばす

wave に属さない単独の lane（2026-10-23 に同時に起票した 2 本の 1 本。もう 1 本は初期投入の手順の lane `docs/archive/plans/2026-10-23-initial-load-procedure.md`〈別の branch〉）。導入の前の関門の candidate「操作ログの保持を 7 年程度へ」（[Plans の導入の前の関門](../../Plans.md#導入の前の関門)、`docs/backlog.md` の「操作ログの保持を 7 年程度に延ばす」、owner の決定 repo 外の回答台帳 TD-348）を、関門に足す前提で起票する（owner の決定 repo 外の回答台帳 TD-402、2026-10-23）。

規模と手法（起票の最初に数えた）: 実装で触る code は 3 file（`src-tauri/src/db/schema_v1.rs` の `app_settings` の初期データ 1 行、`src-tauri/src/mnt/log_manager.rs` の `cleanup_old_logs` と test、`src-tauri/src/db/system_repo.rs` の `delete_old_logs` と test）と生成物 1（`docs/function-design/90-traceability.md`）。正本は plan-first で 11 file を直した（72・74・tracking-system-tables・DB_DESIGN・ARCHITECTURE・REQ-700・decision-log〈D-136 と D-051 の Superseded in part の行〉・backlog、Plan Review round 1 の是正で SCREEN_DESIGN・requirements-coverage・20〈§2.8 の `delete_old_logs`〉）。command・DTO・bindings・画面・migration は 0。関数の形（`cleanup_old_logs(conn) -> Result<(), DbError>`、`delete_old_logs(conn, retention_days: u32) -> Result<usize, DbError>`）は変えない。手法: 1 lane のまま（既定の値と掃除の守りは同じ関数の同じ段で、分けると同じ test を 2 回直す）。

## Workflow State

- Phase: archive
- Risk: R3
- Plan Commit: 715148294e8b8d137496b788d2334ef5540c1af8
- Amendments: none
- Coordinator: Opus 5.5 main session
- Writer: 起票は Opus 5.5 subagent（本 run の起草役）。実装の Writer は座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）の Writer 行から Coordinator が発注時に決める
- Plan Reviewer: fresh Opus 5.5 + Codex（座組表どおり）
- Final Reviewer: Fable 5.1（Claude 側。座組表の「R3 以上と design lane で Fable 5.1」）+ Codex（座組表どおり）
- Final Review Minimum: 1
- Human Gate: ready,merge
- Branch: agent/oplog-retention

遷移の記録:

1. kickoff → spec-check（2026-10-23、起草役）: 発注書の範囲（保持の既定の値の変更だけか、掃除の仕組みも変えるか、既存の設定値を持つ DB の扱い、7 年分の大きさの見積り）を範囲にし、Risk を R3 と記録した（下の Risk）。
2. spec-check → design（2026-10-23、起草役）: 72 §72.4 の保持日数の取得（読めない値は既定 365 日で消す）と、DB_DESIGN の D-5・tracking-system-tables の初期データ・ARCHITECTURE の MNT-02 の行・74 の 365 日の記述・D-051 の Retention を変える必要がある。
3. design → plan-draft（2026-10-23、起草役、同じ commit）: 設計の出力（72 MNT-02-D1、decision-log D-136、上の各文書の直し）は本 commit に置いた。未解決の設計の問いは無い（owner の判断事項 1 問は推奨を既定にした。下の Review Focus。owner の方針、repo 外の回答台帳 TD-401）。
4. plan-draft → plan-gate（2026-10-24、Coordinator、本 commit）: owner の判断事項は owner の方針（repo 外の回答台帳 TD-401）で推奨を既定にした（TD-407。後で owner に見せ、答えが変われば plan-gate のまま直す）。packet と Matrix が揃い commit されている。main（`843c4938`、PR #191・#200 の後）を単段で取り込んだ（衝突: decision-log の 1 か所（D-132 と D-136 が同じ末尾に足された。番号の順に両方を残した））。Plan Review round 1（fresh Opus 5.5 + Codex）に出す。
5. plan-gate のまま是正（round 1、2026-10-24、Coordinator、本 commit）: round 1（Claude 側 fresh Opus 5.5 と Codex）の reject を起草役が `879a4495` で直した（下の Review Response）。設計の方向は変えないので plan-gate に留め、round 2（fresh Opus 5.5 + Codex）で再 review する。
6. plan-gate のまま是正（round 2、2026-10-24、Coordinator の代行の起草役、本 commit）: round 2 は両 reviewer とも approve（P1 0 / P2 0、P3 は各 3 件）。P3 の全 6 件と、K1 の owner の決定（repo 外の回答台帳 TD-418、2026-10-24。d 約 3000 日）を反映し、既定の日数を 2557 から 3000 に改めた（下の Review Response）。round 2 の approve は日数を改める前の版の plan に対するものなので、plan-gate に留め、契約の変わった版の確認（round 3）を受けてから plan-approved へ進む。
7. plan-gate → plan-approved（2026-10-24、Coordinator の代行の起草役、本 commit）: round 3（Plan Review round 天井 3）は独立の Plan Reviewer（Writer と別の context）の Claude 側 fresh Opus 5.5 = approve（P1 0 / P2 0 / P3 6）、Codex（発注 552）= approve（P1 0 / P2 0 / P3 1）で、日数を 3000 日に改めた版の plan に P1/P2 = 0（下の Review Response）。P3 の 7 件は Coordinator の disposition（2026-10-24、round 4 は開かない）に従い plan-gate のまま起草役が `715148294e8b8d137496b788d2334ef5540c1af8` で直した（6 件の是正と backlog の 1 件の起票。Plan Commit はその版）。実装の commit はまだ無い。Scope の migration の段の前提（初回の導入が済んでいないこと）は、Coordinator が Ready の依頼の前に確かめ、済んでいれば Ready を依頼せず plan-gate に戻す。
8. plan-approved → implementing（2026-10-24、Coordinator、本 commit）: base の main（`843c4938`）は Plan Commit の祖先のままで同期は要らない。AC4・AC5 の rg の baseline を逐語で再測して上の値と一致した（AC4 exit 1、`schema_v1.rs:302` は `'365'`。AC5 exit 1、`log_manager.rs:10` は `= 365`）。migration を足さないので、PR #188・#172 の migration の版 11 とは重ならない。

## Owner Effort Budget

- 介入回数上限: 6（既定）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 6 | 1（K1 の答え、2026-10-24、TD-418） | 2（Ready 1、merge 1） | 3 | 6 = 1 + 2 + 3 |

owner の判断事項 K1 は owner が答えた（d 約 3000 日。下の Review Focus）。L3 は無い（下の Risk の L3 の段）。

承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の R3「DB ... changes」に当たる: DB を作るときの初期データ（`app_settings` の key `log_retention_days`）と、操作ログの物理削除の条件（audit / log の persistence の振舞い）を変える。R4 にしない理由: 新しい削除の経路を足さず、消す範囲を狭める向きだけの変更（保持を 365 日から 3000 日へ延ばし、保持日数を確定できない値では今の「既定 365 日で消す」をやめて消さない）。backup・restore・migration・店の実データに触れない（本番の DB は無い、`docs/project-memory.md` の「無いもの」）。R3 の要件: Contract Ledger・Data Safety・Contract Probe・Test Design Matrix・Design Readiness を置いた。required gate: 実装の PR は Rust を変えるので `rust=true`（classify-changes）。Final Review Minimum は 1、Human Gate は `ready,merge`（R3 の既定）。

L3 について: L3 の条件（`docs/DEV_WORKFLOW.md` Human Visual Confirmation の L3 Eligibility）の (1)「Windows/Tauri native でしか観測できない」に当たる項目が無い。掃除は起動時の setup で DB に対して行い、振舞いは全て自動 test で示せる（Matrix）。7 年の経過は実機で見られない。画面の文言・route は変えない。owner の操作の回数を減らす工夫: L3 の round を作らず、初回の導入の版の MSI の予行演習（`docs/msi-distribution-runbook.md` の確かめる項目）で操作ログの画面を開いたときに記録が出ることを見るだけで足りる（追加の操作は無い）。Human Gate に manual を入れないので、`cargo check --release` の Writer の完了条件は当たらない。

## Goal

Goal Invariant:

### 最小完了条件

- 初回の導入の版で作られる店の DB では、操作ログが 3000 日（1 月 1 日の取引でも、帳簿書類の 7 年〈申告の期限の翌日から〉に足りる長さ）消えずに残り、それより古いものだけが起動時に消える。
- 保持日数の設定が読めない値・0 になっても、操作ログは消えない（記録が溜まる側に倒れる）。

### 失敗定義

- 新しい DB の `log_retention_days` が 3000 でない、または掃除が 3000 日より新しい記録を消す。
- 設定の値が読めない・0 のときに記録を消す、または大きな値で起動時に panic する（setup の中で起動が止まる）。
- 既にある開発・試験 DB の値を書き換える migration を足す（本 lane は足さない。D-136 (4)）。
- 操作ログの画面・command・wire が変わる。

### 非目的

- 保持日数を画面で変えられるようにする（69 UI-11a-D1 のまま）。
- 操作ログの archive・CSV 出力・手動削除（74 の Non-scope のまま）。
- backup の保持日数（3 日）を変える。
- `operation_logs` に index を足す（合成の 2557 日分で一覧の取得が約 0.07 秒。Contract Probe）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 初回の導入の版で DB が空から作られた（`log_retention_days` = 3000） | 毎日アプリを起動する | 3000 日より前の操作ログだけが 1 日 1 回消え、それより新しい記録は操作ログの画面で見える | — | なし |
| 設定の値が壊れた（数でない・0。画面からは変えられない。汎用の `update_setting` command は key を問わず値を検査せずに書けるが frontend はこの key を送らないので、その command を直に呼ぶか DB を手で触った場合） | 起動する | 操作ログは消えず、診断ログに warn が残る。アプリは起動する | 値を直せば次の起動から掃除が戻る | なし |
| 既にある開発・試験 DB（`log_retention_days` = 365） | 起動する | 今と同じく 365 日で消す。22 §16 の作り直しで新しい既定になる | — | なし |

## Scope

決めたこと（発注書の「決めること」）:

- 既定の値の変更と、掃除の守りの小さな変更の両方。掃除の仕組み（起動時に 1 日 1 回、物理削除、`log_cleanup` の記録）は変えない。守りの変更は、保持日数を確定できない値（数でない・0）で既定に置き換えて消すのをやめて消さないことと、日付の範囲の外の値で panic しないこと（72 MNT-02-D1。backup の 71 MNT-01-D3 と同じ形）。
- 既存の設定値を持つ DB: migration は足さない。新しい DB の値は DB を作るときの初期データ（`schema_v1.rs`）で決まる。既にある DB は開発・試験 DB だけで、今の値（365）のまま、作り直せば 3000（D-123 (1)、D-136 (4)）。予約の 22 §25・MNT-03-D23 は使わない（PR #188 が migration v11 を足す予定で、足すと番号も後ろにずれる）。migration を足さない判断は、本 lane が初回の導入の版に含まれる前提に立つ。導入の後なら、既存の DB の値を直す migration が要る（D-136 の Guarantee range）。この前提は Coordinator が Ready の依頼の前に確かめる（`docs/Plans.md` の導入の前の関門と repo 外の回答台帳で、初回の導入が済んでいないこと）。済んでいれば Ready を依頼せず plan-gate に戻し、既存の DB の値を直す migration を Scope に足す。
- 保持の長さ分の大きさ: 合成の 2557 日分の測定と、3000 日分への行数の比（下の Contract Probe P1）。店の 1 日の行数は未実測。

実装（plan-approved の後）:

- `src-tauri/src/db/schema_v1.rs`: `app_settings` の初期データの `('log_retention_days', '365', …)` を `'3000'` にする（他の行は変えない）。
- `src-tauri/src/mnt/log_manager.rs`: `DEFAULT_RETENTION_DAYS` を 3000 にする（設定の行が無いときの値）。設定の値が `u32` として読めないか 0 のときは、`tracing::warn!` を記録して削除も `log_last_cleanup_date` の更新もせず `Ok(())` を返す（72 §72.4 の手順 3）。doc comment の「デフォルト365日」を直す。
- `src-tauri/src/db/system_repo.rs`: `delete_old_logs` の cutoff を `checked_sub_days`（`chrono::Days`）で求め、日付の範囲の外なら何も消さず `Ok(0)` を返す（72 §72.4 の手順 4）。
- test（同じ 2 file の `mod tests`）:
  - 直す: `test_cleanup_old_logs_req902_deletes_expired`（366 日前 → 3001 日前）、`test_cleanup_old_logs_req902_keeps_recent`（364 日前 → 3000 日前。初期データの値の境）、固定の日付 `2025-01-01T10:00:00` を古いログに使う `test_cleanup_old_logs_req902_first_run`・`test_cleanup_old_logs_req902_same_day_skip`・`test_cleanup_old_logs_req902_records_cleanup`（今日から 3001 日前の相対の日付にする。365 日より古いだけでは 3000 日の保持で消えない）。期待（消える・残る・記録される・飛ばす）は変えない。誤った test ではなく、設計正本の値（72 MNT-02-D1）が変わったための直し（AGENTS.md の Working Rules）。
  - 足す（名前は実装で決め、REQ-902 を付ける）: 新しい DB の `log_retention_days` が `"3000"`。設定の行を消した DB で 3001 日前が消え 3000 日前が残る。値が `"abc"`・`"-1"`・`"0"` の DB で何も消えず `log_last_cleanup_date` が変わらない。値が `"4294967295"` の DB で panic せず何も消えない（`cleanup_old_logs` と `delete_old_logs` の両方。`cleanup_old_logs` では 0 件から §72.4 の手順 6 へ進み、`log_last_cleanup_date` が今日になる）。
- 生成物: `docs/function-design/90-traceability.md`（`cd src-tauri && cargo run --bin generate_traceability`。REQ-902 の test の数が変わる。手で直さない）。

作る所・読む所の全件（`rg -n "log_retention_days|DEFAULT_RETENTION_DAYS|delete_old_logs|cleanup_old_logs" src-tauri/src src` で確かめた、2026-10-23）: 作る所は `schema_v1.rs` の初期データ 1 行と、key を問わない汎用の書込みの経路（`update_setting` command〈`src-tauri/src/cmd/settings_cmd.rs:137`〉→ `biz::system_service::upsert_setting`〈`src-tauri/src/biz/system_service.rs:19`〉→ `system_repo::upsert_setting`。値を検査しない）。command は書けるが frontend は送らない（`rg -n log_retention src` で hit 無し。`updateSetting` を呼ぶ backup の画面は `BACKUP_SETTING_KEYS` の key に限り、閾値の画面は閾値の key だけを送る）。不正な値は MNT-02-D1 で消さない側に倒れる。この経路の API は変えない。読む所は `log_manager.rs` の `cleanup_old_logs` だけ。`delete_old_logs` の呼び出し元は `cleanup_old_logs` と `system_repo.rs` の test 2 本（`test_delete_old_logs_req902_*`、引数に 365 を直接渡すので変えない）。`cleanup_old_logs` の呼び出し元は `src-tauri/src/lib.rs` の `run` の setup（失敗を warn して続ける、変えない）と `log_manager.rs` の test。`system_repo.rs` の `log_retention_days` の key の存在を見る test は変えない。`backup.rs` の `DEFAULT_RETENTION_DAYS`（3、backup の保持）は別の定数で変えない。

正本（本 commit、plan-first）:

- `docs/function-design/72-mnt-log-manager.md`: §72.4 の手順 3・4、MNT-02-D1 の新設、§72.5 の追加箇所とコード例を現物の起動の手順 5 に合わせる（Plan Review round 3）、§72.6 の test の表（名前を実在の `_req902_` に直し、MNT-02-D1 の test を足す）。
- `docs/decision-log.md`: D-136 の新設（末尾）、D-051 の `Superseded in part by: D-136` の行。
- `docs/DB_DESIGN.md` の D-5、`docs/db-design/tracking-system-tables.md` の app_settings の初期データ例、`docs/ARCHITECTURE.md` の MNT-02 の行、`docs/REQ-700_diagnostic_log.md` の比較の表、`docs/function-design/74-ui-operation-logs.md` の 365 日の記述 6 か所（UI-11c-D10・D13・D14、§74.10 と §74.12 の各 1 か所、§74.16 非目的の表）。
- `docs/backlog.md`: 「操作ログの保持を 7 年程度に延ばす」の項目に起票の行。項目の本文の「今は…既定 365」は code の今の値で、実装の merge の後の closeout で直す。その直後に「操作ログの掃除の記録「（{cutoff_date}以前）」を実際の境に合わせる」の項目（Plan Review round 3 の P3 の起票）。
- `docs/function-design/20-io-product-repo.md` の §2.8 `delete_old_logs` の処理ステップ 1（`checked_sub_days`、`None` で `Ok(0)`）、`docs/SCREEN_DESIGN.md` の操作ログ画面のレイアウト判断の 1 行、`docs/spec/requirements-coverage.md` の QR-06 の行（Plan Review round 1 の是正）。

他の lane との所有（`docs/DEV_WORKFLOW.md` Wave Operation。open の PR #172・#188・#190・#191・#192・#193・#194 の差分〈#188 は packet の Scope〉と本 lane の予定の path を `comm -12` で突き合わせた。下の AC の実測）:

| file | 節 | 所有 |
|---|---|---|
| `docs/decision-log.md` | D-136（末尾に足す）と D-051 の Superseded in part の行 | 本 lane。#188（D-109 の Superseded の行と新しい決定）・#191・#194 の新しい決定は別の節。末尾への追記は merge の順で両方を残す |
| `docs/DB_DESIGN.md` | D-5 の節 | 本 lane（#188 の Scope は別の節） |
| `docs/ARCHITECTURE.md` | MNT-02 の行 | 本 lane（#188 の Scope は D-118 の記述） |
| `docs/db-design/tracking-system-tables.md` | §19 app_settings の初期データ例の `log_retention_days` の行 | 本 lane（#188 は inventory_movements ほか） |
| `docs/backlog.md` | 「操作ログの保持を 7 年程度に延ばす」の項目とその下の箇条、その直後の「操作ログの掃除の記録「（{cutoff_date}以前）」を実際の境に合わせる」の項目 | 本 lane（#172・#188・#190・#192 は別の項目） |
| `docs/function-design/90-traceability.md` | 生成物（REQ-902 の行） | 生成物。衝突したら手で解かず、取り込んだ後に再生成する（D-098。#188 も再生成する） |
| `src-tauri/src/db/schema_v1.rs` | `app_settings` の初期データの `log_retention_days` の行 | 本 lane。#188 は読むだけ（L3 の手順で初期の部門を引く）。#192 が backlog に足す後続の lane（部門 12 の `code_prefix`）は同じ file の `departments` の初期データの行で、別の行 |
| 72・74・REQ-700・`log_manager.rs`・`system_repo.rs`・本 packet・Matrix | 全体 | 本 lane（他の PR の差分と Scope に無い） |
| `docs/function-design/20-io-product-repo.md` | §2.8 の `delete_old_logs` の処理ステップ 1 | 本 lane。#188 は §2.9 以降の stocktake の節と冒頭（2026-10-24 に open の PR の `gh pr diff --name-only` と hunk で確かめた） |
| `docs/SCREEN_DESIGN.md` | 操作ログ画面のレイアウト判断の「閲覧 MVP」の行 | 本 lane。#188 は冒頭と棚卸しの節の別の hunk |
| `docs/spec/requirements-coverage.md` | QR-06 の行 | 本 lane（2026-10-24 の open の PR の差分に無い） |

この変更で不要になる記述・条件・code（D-127 の問い）: DB_DESIGN の D-5 の「1年以上前のログが必要な場合はバックアップから復元」は backup の保持（3 日）と合わず誤っていたので置き換えた。72 の「パース失敗 → デフォルト365日」の fallback は消す側の既定で、MNT-02-D1 で外す（既定は設定の行が無いときだけ）。74 の「365日 cleanup」の 6 か所は日数を書かずに「保持日数の cleanup」にし、日数の正本を 72 MNT-02-D1 にした。SCREEN_DESIGN の「365日超の自動削除」と requirements-coverage の QR-06 の「365 日超の削除」「365 日 cleanup」も同じく保持日数の形に置き換え（round 1 の是正で足した既定の日数は、74 の UI-11c-D13・D14、SCREEN_DESIGN、QR-06 から round 2 の是正で外し、MNT-02-D1 を引く形にした）、20 §2.8 の panic する引き算（`- chrono::Duration::days(..)`）は `checked_sub_days` に置き換えた（Plan Review round 1）。`docs/DEV_SETUP_CHECKLIST.md` の「MNT-02 log_manager: cleanup_old_logs（起動時自動削除、365 日超）」は済んだ作業の記録の行で、変えない。decision-log の D-051 の本文は append-only で、Superseded in part の行だけを足した。72 MNT-02-D1 の Why にあった 7 年の数え方は D-136 の Why (1) と 2 重だったので外し、D-136 を引く形にした（Plan Review round 3）。

## Non-scope

- 保持日数の画面（69 UI-11a-D1）、操作ログの画面・command・wire・bindings。
- migration（22 §25・MNT-03-D23 の予約は使わない）、既にある開発・試験 DB の値の書換え。
- backup の保持日数、`operation_logs` の index、診断ログ（MNT-04、30 日）。
- `docs/Plans.md`（D-097）。

## Acceptance Criteria

- AC1: `bash scripts/doc-consistency-check.sh` が exit 0（ERROR なし）。baseline（2026-10-23、起点 `eed07d37`）: exit 0、`結果: WARN 12 件（ERROR なし）`。
- AC2: `bash scripts/doc-consistency-check.sh --target plan docs/archive/plans/2026-10-23-oplog-retention.md` が exit 0（起票の時の path は `docs/plans/`、closeout で archive の path に直した）。
- AC3（正本の旧い文）: `rg -n 'log_retention_days \| 365|デフォルト365日|365 ?日 cleanup|365日自動削除|365 ?日超|\| 保持期間 \| 365日|Duration::days\(retention_days\)' docs/ARCHITECTURE.md docs/DB_DESIGN.md docs/REQ-700_diagnostic_log.md docs/db-design/tracking-system-tables.md docs/function-design/72-mnt-log-manager.md docs/function-design/74-ui-operation-logs.md docs/SCREEN_DESIGN.md docs/spec/requirements-coverage.md docs/function-design/20-io-product-repo.md` が何も出さない（exit 1）。baseline（`git grep -n -P` で `eed07d37`）: 13 行（ARCHITECTURE 208、REQ-700 90、SCREEN_DESIGN 185、tracking 228、20 の 663、72 の 47、74 の 60・63・64・272・297・568、requirements-coverage 66）。是正の commit の後: exit 1。2 本目（旧い既定の日数 2557 の live な文。round 2 の是正）: `` rg -n '既定 ?2557|2557 日[（〈]7 年以上|7 年以上|\| 2557 \||保持日数（2557 日）|`2557`|2557 日で消す' `` を上の 9 file と `docs/decision-log.md`・`docs/backlog.md` に当てて何も出さない（exit 1）。baseline（`git grep -n -P` で `16d7dd00`）: 17 行。round 2 の是正の commit の後: exit 1（2026-10-24）。残る `2557` は棄てた案・測定の記録・改めた経緯だけ（decision-log の D-136 の Status・Alternatives・Guarantee range、72 の Rejected alternatives と大きさ、backlog の起票の行）。
- AC4（初期データ）: `rg -n "'log_retention_days',\s+'3000'" src-tauri/src/db/schema_v1.rs` が 1 行。baseline（`eed07d37`）: exit 1（値は `'365'`）。
- AC5（既定の定数）: `rg -c 'DEFAULT_RETENTION_DAYS: u32 = 3000' src-tauri/src/mnt/log_manager.rs` が `1`。baseline: exit 1（`= 365`）。
- AC6（test）: `cd src-tauri && cargo test --lib log_manager` と `cd src-tauri && cargo test --lib system_repo` が全 PASS で、Scope の「足す」の 4 群の test が含まれる（test の名前と本数は PR の runner の出力で示す）。baseline: 未実測（本 run は cargo を回さない。発注書の条件）。
- AC7（mutation）: 実装の後、`schema_v1.rs` の値を旧い既定の案の `'2557'` にすると 3000 日前が残る test が red、`log_manager.rs` の 0 と読めない値の分岐を外す（既定で消す）と読めない値・0 の test が red、`delete_old_logs` の `checked_sub_days` を `-` に戻すと大きな値の test が panic で red になる（Matrix の Mutation-style Adequacy Questions。実注入の結果を PR に書く）。
- AC8（生成物）: `cd src-tauri && cargo run --bin generate_traceability -- --check` が exit 0。
- AC9: `cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings` が exit 0。
- AC10: `bash scripts/check-workflow-git.sh` が exit 0。

実測（本 commit の前、2026-10-23。逐語で実行した。AC4〜AC9 は実装の後）:

```text
AC1  bash scripts/doc-consistency-check.sh   → 起票の前 exit=0、結果: WARN 12 件（ERROR なし）
AC3  baseline: git grep -n -P '<AC3 の式>' eed07d37 -- <AC3 の 9 file> → 13 行 / 是正の commit の後: rg exit=1（2026-10-24、Plan Review round 1 の是正で式と file を広げて測り直した）
AC4  baseline: src-tauri/src/db/schema_v1.rs:302 ('log_retention_days', '365', ...)
AC5  baseline: src-tauri/src/mnt/log_manager.rs:10 const DEFAULT_RETENTION_DAYS: u32 = 365;
```

## Design Readiness

- 引く設計正本（節まで）: 72 §72.4（手順 1〜7）・§72.5・§72.6、71 MNT-01-D3（読めない保持日数で消さない）、69 UI-11a-D1（保持日数を画面で変えない）、74 UI-11c-D10・D13・D14 と §74.10・§74.12・§74.16、tracking-system-tables §19 app_settings、DB_DESIGN の D-5、ARCHITECTURE の MNT-02、D-051（Retention）、D-123 (1)、22 §3.2 MNT-03-D11、REQ-902（`docs/spec/requirements.md`）、QR-06。
- 必要な設計成果物: Design artifact selection の「audit/log ... persistence behavior」の行 → DB_DESIGN と db-design（updated in this PR）、「New or changed ... MNT ... behavior」の行 → 72（updated in this PR、MNT-02-D1）、「Durable cross-cutting choice」の行 → decision-log D-136（updated in this PR）。
- plan にしかない durable な判断の昇格先: 無い（全て 72 MNT-02-D1 と D-136 に置いた）。
- 前提・制約と延期した gap: 店の 1 日の行数は未実測（導入の後に操作ログの件数で分かる。D-136 の Revisit）。延期しても安全な理由: 合成の 1 日 200 行の 2557 日分で 210006016 byte、一覧は約 0.07 秒、3000 日分は行数の比で約 246389538 byte（未実測。Contract Probe P1）で、運用に障る大きさではない見込み（店の PC では測っていない）。
- 絶対保証の自己点検: 「3000 日残る」は、設定が既定のまま（画面からは変えられず、汎用の `update_setting` command で書き換えない限り）で、DB と backup が残る限りに限って書いた。保持期間を過ぎた後の復元は保証しない（残る backup の作成の時点による）。保持日数の内でも DB を失えば backup（3 日）と PC の外の控えの範囲だけが戻る。「消えない」（読めない値）は削除を飛ばす分岐だけを指し、他の削除の経路（無い。`operation_logs` の DELETE は `delete_old_logs` だけ。`rg -n "DELETE FROM operation_logs" src-tauri/src` で 1 か所）を含めて確かめた。
- 判定: ready。

## Contract Audit（新しく足す契約の文の境界の一巡）

| 観点 | 契約の文（72 MNT-02-D1、§72.4） | 確かめたこと |
|---|---|---|
| 安全な整数の範囲 | 「日付の範囲の外になる大きな値は、消す対象が無いとして 0 件」 | 値は `u32`（最大 4294967295）で読み、`chrono` の `NaiveDate` から引くと範囲の外で panic する（今の `date_naive() - Duration::days(..)`）。setup の中の panic は起動を止めるので、`checked_sub_days` の `None` を 0 件にする。`i64` の幅の問題は無い（`u32` → `Days`）。JavaScript の安全な整数の範囲は当たらない（wire に出ない） |
| 0 の表示 | 「0 のときは削除を飛ばす」 | 0 を受けると cutoff が今日の 0 時になり、今日より前の全ての記録を消す。0 は既定（3000）とも「消さない」とも区別して、読めない値と同じく飛ばす。負の値は `u32` で読めないので同じ分岐 |
| 失敗の時点と順序 | 「削除も日付の更新もせず Ok(()) を返す」 | 飛ばすときは `log_last_cleanup_date` を更新しないので、同じ日の次の起動でも同じ判断になる（設定を直せば次の起動で掃除する）。DB の読取りの失敗は今のまま `Err` で返り、呼び出し元が warn して起動を続ける（§72.4 のエラーハンドリング）。削除と `log_cleanup` の記録と日付の更新の順は変えない |
| 単位や種類の混在 | 「既定は 3000 日」 | 単位は日で、年を持たない（帳簿書類の 7 年〈申告の期限の翌日から〉を、1 月 1 日の取引にも足りる日数に直した。数え方は D-136 の Why (1)）。cutoff は今日から日数を引いた日の 0 時（ローカル時刻）で、`created_at` の文字列と比べる今の形を変えない。設定は TEXT で、数は文字列 `'3000'` |

## Registration / Generation Obligations

| 変更対象 | 義務 | 本 lane |
|---|---|---|
| REQ / coverage の追加・変更 | `cd src-tauri && cargo run --bin generate_traceability` で `docs/function-design/90-traceability.md` を再生成 | REQ-902 の test を足すので再生成する（Scope の生成物、AC8） |

ほかの行（Tauri command、function-design doc 新設、route、operator 画面、source / workflow doc 新設）は該当なし。

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | 外部の adapter と app-core の境 | — | 不該当: 外部の系に触れない | なし |
| Fact check / design decision split | 観測した事実とアプリの判断の分け | Contract Probe、D-136 | 当たる: 大きさは合成の測定（事実）、店の行数は未実測、保持の長さは owner の決定 | D-136 の Revisit |
| Lifecycle / retry | 前・中・後・失敗の後 | 72 §72.4、Contract Audit | 当たる: 読めない値で飛ばし、日付を更新しないので次の起動で同じ判断。設定を直せば戻る | なし |
| Operator workflow | 実際の順序 | — | 不該当: 店の操作は変わらない（起動するだけ） | なし |
| Replacement path | 外部の系が変わったとき | — | 不該当 | なし |
| Data safety / evidence | 店のデータを repo に入れない | Data Safety | 当たる: 測定は合成のデータだけ（scratch の DB、commit しない） | なし |
| Reporting / accounting semantics | 集計の意味 | — | 不該当: 集計に触れない | なし |
| Manual verification | 自動 test で示せない主張 | Risk の L3 の段 | 当たる（L3 無し）: 全て自動 test で示せる | なし |
| 環境・再現性 | 新しい環境依存の pin | — | 不該当: 新しい依存を足さない（`chrono` の `Days` は既存の依存） | なし |

## Boundary / Wire Contract

- producer: `schema_v1.rs` の初期データ（DB を作るとき）。汎用の `update_setting` command も key を問わず書けるが、frontend はこの key を送らない。不正な値は消さない側に倒れる（MNT-02-D1）
- consumer: `cleanup_old_logs`（起動時）。`get_settings` は全ての key を返すが、frontend は `log_retention_days` を読まない（`rg -n log_retention src` で hit 無し）
- wire type: 変えない（`app_settings.value` は TEXT）
- internal type: `u32` の日数（今と同じ）
- precision/range: 1〜`u32` の最大。0 と読めない値は飛ばす。日付の範囲の外は 0 件
- round-trip path: 該当なし
- invalid input: 数でない・負・0 → 飛ばす（warn）
- compatibility: 既にある開発・試験 DB の値（365）はそのまま働く。前の版のアプリは同じ DB を今と同じく扱う（migration を足さないので DB の版は変わらない）

## Test Plan

Test Design Matrix: [2026-10-23-oplog-retention](test-matrices/2026-10-23-oplog-retention.md)。

- targeted tests: `cd src-tauri && cargo test --lib log_manager`、`cd src-tauri && cargo test --lib system_repo`。
- negative tests: 読めない値・0・大きな値（Scope の「足す」）。
- compatibility checks: 設定の行が無い DB（既定）、既にある DB の値を変えないこと（migration が無いこと: `rg -n "log_retention_days" src-tauri/src/db/schema_v*.rs` が `schema_v1.rs` の 1 行だけ）。
- data safety checks: 合成のデータだけ。
- 0 時の跨ぎ: 境の日ちょうどの記録と今日の日付を見る test（C2 の `test_cleanup_old_logs_req902_keeps_recent` の 3000 日前、C3 の 3000 日前、C5 の `log_last_cleanup_date` が今日）は、test が日付を求めてから `cleanup_old_logs`・`delete_old_logs` が `Local::now()` を取るまでに 0 時を跨ぐと cutoff と今日が 1 日進み、偽の red になる。test の前後で `Local::now().date_naive()` が変わっていないことを確かめ、変わっていれば同じ日の中でやり直す等で偽の red を避ける（Plan Review round 3 の P3）。
- main wiring/integration checks: `lib.rs` の `run` の setup が `cleanup_old_logs` を呼び、失敗を warn して起動を続ける形（`src-tauri/src/lib.rs:1475`〜`:1478`、DB の初期化の後・自動 backup の前）を変えない。これを確かめる自動 test は無いので、PR の差分に `lib.rs` が無いこと（`git diff --name-only`）と、この行の静的照合で確かめる。

## Review Focus

- 72 MNT-02-D1 と D-136 が 71 MNT-01-D3 と同じ形で、読めない値・0 で消さず、`log_last_cleanup_date` を更新しない順が正しいか。設定の行が無いときだけ既定を使う形の当否。
- migration を足さない判断（D-136 (4)）と、`schema_v1.rs` の初期データを直す形の当否（D-123 (1)。リリースの前に v1 の初期データを直す先例は #192 が backlog に足す部門 12 の後続の lane）。
- 既存の test の直し（Scope の「直す」）が期待を変えずに値の境だけを直しているか。弱めていないか。
- この変更で不要になる記述・条件・code が他に無いか（Scope の D-127 の段）。
- owner の判断事項（起票では owner の方針 TD-401 のとおり推奨を既定にし〈TD-407、a 2557 日〉、round 2 の後に owner に諮った）:
  - K1 保持の長さ: 決定は d 約 3000 日（owner の決定、repo 外の回答台帳 TD-418、2026-10-24）。選択肢は a 2557 日（取引の日から 7 暦年）/ b 2555 日（365 × 7）/ c 消さない（掃除を外す）/ d 約 3000 日（1 月 1 日の取引でも帳簿書類の 7 年〈申告の期限の翌日から〉に足りる）。正確な日数は 3000 日（数え方は D-136 の Why (1)）。
    - plan の段階で直した所（round 2 の是正の commit。日数を 3000 日に改めた）: 72 の §72.4 の手順 3・MNT-02-D1・§72.6 の test の表と MNT-02-D1 の test の段、decision-log の D-136 と D-051 の Superseded in part の行、DB_DESIGN の D-5、tracking §19 の初期データ例、ARCHITECTURE の MNT-02 の行、REQ-700 の比較の表、backlog の起票の行。packet の Goal・Ordinary Operation・Risk・Scope の実装の段・AC3〜AC5・AC7・Design Readiness・Contract Audit・Contract Ledger・Contract Probe P1。Matrix の前文・C1〜C3・FM1・FM2・Test Matrix・State Lifecycle・Boundary・Compatibility・Mutation の各行。
    - 日数を外して 72 MNT-02-D1 を引く形にした所（同じ commit）: 74 の UI-11c-D13・D14、SCREEN_DESIGN の操作ログ画面の「閲覧 MVP」の行、requirements-coverage の QR-06 の行。
    - 実装で直す所（plan-approved の後、Scope の実装の段）: `schema_v1.rs` の初期データ、`log_manager.rs` の `DEFAULT_RETENTION_DAYS` と doc comment、`log_manager.rs`・`system_repo.rs` の test の境の日数（3000 日前・3001 日前）、生成物 `90-traceability.md`。closeout で直す所: backlog の項目の本文の「今は…既定 365」。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| MNT-02-D1（既定 3000 日、設定の行が無いときも 3000 日） | 72 §72.4 の手順 3、MNT-02-D1、tracking §19、DB_DESIGN の D-5、D-136 (1) | `schema_v1.rs` の初期データ、`log_manager.rs` の `DEFAULT_RETENTION_DAYS` | 直す `test_cleanup_old_logs_req902_deletes_expired`・`test_cleanup_old_logs_req902_keeps_recent`、足す「新しい DB の値」「設定の行が無い DB」 | 非対象 |
| MNT-02-D1（読めない値・0 で消さず、日付を更新しない） | 72 §72.4 の手順 3、MNT-02-D1、D-136 (2) | `log_manager.rs` の `cleanup_old_logs` | 足す「読めない値・0」 | 非対象 |
| MNT-02-D1（日付の範囲の外は 0 件、panic しない） | 72 §72.4 の手順 4・6、20 §2.8 の `delete_old_logs`、D-136 (2) | `system_repo.rs` の `delete_old_logs` | 足す「大きな値」（`cleanup_old_logs` と `delete_old_logs`。`cleanup_old_logs` では `log_last_cleanup_date` が今日になる） | 非対象 |
| §72.4 の 1 日 1 回・`log_cleanup` の記録・日付の更新（変えない） | 72 §72.4 の手順 1・2・5・6 | 変えない | 既存 `test_cleanup_old_logs_req902_same_day_skip`・`test_cleanup_old_logs_req902_records_cleanup`・`test_cleanup_old_logs_req902_updates_date`・`test_cleanup_old_logs_req902_first_run`（古いログの日付だけを直す） | 非対象 |
| `delete_old_logs` の cutoff（今日から日数を引いた日の 0 時より前を消す。変えない） | 20 §2.8 の `delete_old_logs`（72 §72.1 の system_repo の拡張） | `system_repo.rs`（求め方を checked にするだけ） | 既存 `test_delete_old_logs_req902_*` 2 本（引数 365 のまま） | 非対象 |
| §72.5 の起動の順と、失敗を warn して起動を続けること（変えない） | 72 §72.5 | 変えない | 無い（自動 test は無い。PR の差分に `lib.rs` が無いことと `lib.rs:1475`〜`:1478` の静的照合。Test Plan の main wiring） | 非対象 |
| 保持日数を画面で変えない（変えない） | 69 UI-11a-D1 | 変えない | 非対象 | 非対象 |
| 隣接の除外: backup の保持日数と MNT-01-D3 | 71 §71.5・MNT-01-D3 | 変えない（形だけを借りる） | 既存 | 非対象 |
| 隣接の除外: D-051 の `integrity_fix` の監査の痕跡（同じ TX、ログに残す） | D-051、36 BIZ-07-D3、74 UI-11c-D14 | 変えない（残る期間だけが延びる。D-051 の Superseded in part の行） | 既存 | 非対象 |
| 隣接の除外: migration の版と前の版の拒否 | 22 §3.2 MNT-03-D11 | migration を足さない | 非対象 | 非対象 |

## Contract Probe

R3 の Contract Probe は外部の前提に頼る計画で Plan Gate の前に最小の実験を求める。本 lane の前提で外部のものは 2 つで、どちらも Plan Gate の前に確かめた（P1 は起票の run、P2 は Plan Review round 1 の是正の run）。

- P1 保持の長さ分の操作ログの大きさと一覧の速さ（起票の時の既定 2557 日分で測った）: 合成の DB（`operation_logs` は `schema_v1.rs` と同じ定義、index 無し）に 1 日 200 行 × 2557 日 = 511400 行（summary は日本語 40 字、detail_json は約 300 byte）を入れ、`sqlite3 "$TMPDIR/oplog-size.db" < oplog-size.sql`（scratch の script、commit しない）→ 結果（2026-10-23、開発用の PC の WSL、sqlite3 3.51.3）: `rows|511400`、`bytes|210006016`、1 か月の範囲の `COUNT(*)` が `Run Time: real 0.072777`、同じ範囲の `ORDER BY created_at DESC, id DESC LIMIT 50 OFFSET 0` が `Run Time: real 0.067429`。`VACUUM INTO`（backup の作り方、71 §71.4）が `Run Time: real 0.461897` で写しは 210006016 byte。既定を 3000 日に改めた後の 3000 日分（600000 行）は測り直さず、行数の比で `python3 -c "print(round(210006016*3000/2557))"` → `246389538`（byte、未実測）。店の PC と店の 1 日の行数では測っていない（未実測）。
- P2 `chrono` の `NaiveDate` から範囲の外の日数を引くと panic し、`checked_sub_days` は `None` を返す: lock の版 `chrono 0.4.44`（`src-tauri/Cargo.lock`）の source（`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/chrono-0.4.44/src/naive/date/mod.rs`）で、`checked_sub_days` は `days.0 <= i32::MAX as u64` でなければ `None`、それ以下なら `add_days` の範囲の検査で `Option` を返し（`:689`〜`:694`）、`impl Sub<TimeDelta> for NaiveDate` は `` checked_sub_signed(rhs).expect("`NaiveDate - TimeDelta` overflowed") ``（`:2139`〜`:2146`）。最小の実験（2026-10-24、`$TMPDIR/chrono-probe` に `chrono = { version = "=0.4.44", default-features = false, features = ["clock"] }` の crate を作り、今日を `2026-10-24` に固定して `cargo run --offline -q`。scratch で commit しない）→ 出力 `checked_sub_days(2557) = Some(2019-10-24)`、`checked_sub_days(100000000) = None`、`checked_sub_days(4294967295) = None`、`today - Duration::days(i64::from(u32::MAX))` は `` `NaiveDate - TimeDelta` overflowed `` で panic（`catch_unwind` で `panicked = true`）。Plan Review round 1 の reviewer の観測（今の `system_repo.rs:224`〜`:225` の式が `u32::MAX` で panic する）と一致する。

## Data Safety

- commit しない: 合成の測定の DB と script（scratch に置き、測った後に消した）、P2 の実験の crate（`$TMPDIR` の scratch）。店の DB・backup・ログ。
- local-only: 無い。
- 試験用だけ: 測定と test は合成のデータだけ。

## Implementation Results

[PR #198](https://github.com/kosei-w90607/inventory-system-desktop/pull/198) で Scope の実装の段のとおりに入れ、2026-10-26 に helper 経由で main へ merge した（owner の承認の後）。新しく作る DB の `log_retention_days` の初期データと、設定の行が無いときの既定を 3000 日にし、保持日数が読めない値・0 のときは warn を残して削除も `log_last_cleanup_date` の更新もせず、日付の範囲の外の大きな値では panic せず 0 件にした（72 MNT-02-D1、D-136）。既存の test は期待を変えずに境の日数だけを直し、Scope の「足す」の test を REQ-902 付きで足し、`90-traceability.md` を再生成した。migration・command・DTO・bindings・画面・`lib.rs` の起動の順は変えていない。test の名前と件数・検証の出力は PR と GitHub が正本。L3 は無い（Risk の L3 の段）。

## Review Response

Fill after review.

round 1（`e05eb9fd`）: Claude 側 fresh Opus 5.5 = reject（P1 0 / P2 2 / P3 3）、Codex = reject（P1 0 / P2 2 / P3 3）。重なりは 3 件（SCREEN_DESIGN・requirements-coverage の旧い日数、範囲の外の掃除の日、汎用の設定の更新の経路）。Coordinator は全 7 件を採用した。是正は起草役の `879a4495`。

- P2: SCREEN_DESIGN と requirements-coverage の旧い日数を保持日数の形に直し AC3 の対象に。20 §2.8 の `delete_old_logs` を `checked_sub_days` と範囲の外の 0 件に。Matrix の範囲の外の行を 72 §72.4 の手順 6（掃除の日を今日にする）に合わせ C4・C5 に分けた。
- P3: 汎用の `update_setting` の経路（command は書けるが frontend は送らない）、backup からの復元の文を「保証しない」に、Contract Probe P2 を chrono の source と最小の実験で実施済みに、migration を足さない判断の前提（初回の導入の版に含める）。
- 起草役の見送り（Coordinator が受理）: `docs/backlog.md` の「既定 365」は closeout で直す。`docs/architecture/mnt-task-specs.md` の MNT-02 の手順の要約に「確定できないときは消さない」の分岐が無い件は、round 2 の reviewer の判断に委ねる。

round 2（`16d7dd00`）: Claude 側 fresh Opus 5.5 = approve（P1 0 / P2 0 / P3 3）、Codex（発注 546）= approve（P1 0 / P2 0 / P3 3）。重なりは 1 件（K1 を変えたときの更新先の漏れ）。Coordinator は全 6 件を受理した（Opus #3 は K1 の問い直しで、owner の決定で閉じる）。是正は Coordinator の代行の起草役が本 commit で行った。

- Codex #1・Opus #1（K1 の更新先の漏れ。Scope の D-127 の段の「日数の正本を 1 か所にした」が現物と合わない）: 74 の UI-11c-D13・D14、SCREEN_DESIGN、QR-06 から既定の日数を外して 72 MNT-02-D1 を引く形にした（D-050 の、別の正本の値を転記しない形）。日数を書く所は 3000 日に改め、Review Focus の K1 に直した所の全件を、plan の段階（正本・packet・Matrix）と実装（code・test・生成物）と closeout に分けて書いた。AC3 に旧い既定の日数の式を 2 本目として足した。backlog の項目の本文「今は…既定 365」は code の今の値で、closeout で直す（Scope の正本の段）。
- Codex #2（起動の順・warn の続行を「既存の起動の test」で確かめるとした誤り）: 確かめる自動 test は無い。Test Plan の main wiring・Contract Ledger の §72.5 の行・Matrix の Main Wiring と Residual Test Gaps を、PR の差分に `lib.rs` が無いことと `src-tauri/src/lib.rs:1475`〜`:1478`（DB の初期化の後・自動 backup の前で呼び、失敗を warn して続ける）の静的照合に直した。
- Codex #3（74 の箇所を §74.6 とした誤り）: Scope の正本の段と Design Readiness を §74.10・§74.12・§74.16 に直した（`rg -n '保持日数の cleanup' docs/function-design/74-ui-operation-logs.md` の 272・297 行は見出し 265 行の §74.10 と 292 行の §74.12、非目的の表の 568 行は 563 行の §74.16）。
- Opus #2（migration を足さない前提が repo の中で強制されていない）: Scope の migration の段に、Coordinator が Ready の依頼の前に初回の導入が済んでいないことを確かめ、済んでいれば plan-gate に戻して migration を Scope に足す、と書いた。
- Opus #3（2557 日は取引の日から 7 年で、帳簿書類の 7 年の数え方より短い。K1 を owner に見せる時点が未定）: K1 を owner に諮り、owner は d 約 3000 日を選んだ（repo 外の回答台帳 TD-418、2026-10-24）。D-136 と 72 MNT-02-D1 の Why を「同じ長さ」から数え方つきに書き直し、Owner Effort Budget の K1 の段を「答えた」に直した。
- K1 の日数（3000 日）の数え方:
  - 起算日: 個人の帳簿書類の 7 年は、その年の翌年 3 月 15 日の翌日から数える（所得税法施行規則 63 条 4 項。日付で決まり、休日で動かない）。法人は確定申告書の提出期限の翌日から数え（国税庁 タックスアンサー No.5930。欠損金の生じた年度は 10 年）、期限は事業年度の終わりの翌日から 2 か月（法人税法 74 条 1 項）、申請で原則 1 か月〈会計監査人を置く場合などは更に長く〉延ばせ（同 75 条の 2）、期限が休日なら翌日（国税通則法 10 条 2 項）。法令は 2026-10-24 に e-Gov 法令 API の本文で、No.5930 は国税庁の頁で確かめた。
  - 店の形: repo 外の回答台帳の L-132・L-217（個人店）、L-284（確定申告で 1 年分の売上と出費をまとめる）、L-059（棚卸しは税理士の指示で大晦日）。暦年で申告する個人事業と読めるが、明言した記録は無いので、12 月末の決算の法人の場合も満たす日数にした。
  - 数え方: 掃除は今日から N 日を引いた日の 0 時より前を消すので（72 §72.4 の手順 4）、ある日の記録はその日から N 日後の日まで残る。要る N は、1 月 1 日の取引の日から 7 年の最後の日までの日数の最大。個人: `python3 -c "from datetime import date as d; r=[(d(y+8,3,15)-d(y,1,1)).days for y in range(2026,2060)]; print(min(r),max(r))"` → `2995 2996`。法人（12 月末の決算、期限が土日なら次の平日）: `python3 -c "from datetime import date as d,timedelta as t;import calendar as c;s=lambda x:x if x.weekday()<5 else s(x+t(1));e=lambda dl:(lambda a:d(a.year+7,a.month,a.day)-t(1))(s(dl)+t(1));print(max((e(d(y+1,2,c.monthrange(y+1,2)[1]))-d(y,1,1)).days for y in range(2026,2060)),max((e(d(y+1,3,31))-d(y,1,1)).days for y in range(2026,2060)))"` → `2983 3014`（2 か月の期限、1 か月延ばした期限）。
  - 丸め: 個人の最大 2996 日と、期限を延ばさない法人の最大 2983 日の両方に足りる切りのよい値として 3000 日にした（4 日の余裕。owner の「約 3000 日」と合い、test の境〈3000 日前・3001 日前〉が読みやすい）。期限を延ばした法人（1 か月の延長で 3014 日、更に長い延長ではそれ以上）と欠損金の年度（10 年）には足りないが、店は個人と読めるので当てず、法人と分かれば D-136 の Revisit で見直す。大きさは 2557 日分の測定の行数の比で約 246389538 byte（Contract Probe P1、未実測）。
- 照合した先（Review Rules の scope の提案の照合）: K1 の答えは repo 外の回答台帳 TD-418 の原文、店の形は同じ台帳の L-132・L-217・L-284・L-059。
- plan-approved へ: 日数を改めたので、round 2 の approve は前の版の plan に対するもの。契約の変わった版の確認を round 3（Plan Review round 天井 3 の最後の round）で受ける。

round 3（`a403f9fb`、Plan Review round 天井 3）: Claude 側 fresh Opus 5.5 = approve（P1 0 / P2 0 / P3 6）、Codex（発注 552）= approve（P1 0 / P2 0 / P3 1）。重なりは無い。Coordinator の disposition（2026-10-24）で 6 件を受理し、Opus #3 は backlog に起票した。round 4 は開かない。是正は起草役が本 commit で行った。

- Codex N1（D-136 の Alternatives の「読めない値を既定 3000 日で掃除する」の括弧の大小が逆）: 括弧を「既定が設定より短いと、残すべき記録を消す」に置き換えた（72 の Rejected alternatives の「既定が設定より短いと消しすぎる」と同じ向き）。
- Opus #1（法人の 2983・3014 日は、申告の期限を休日で翌日にずらす国税通則法 10 条 2 項を保存の起算日に当てた誤り）: 法人の帳簿書類の起算日は「事業年度終了の日の翌日から二月（申告の期限を法人税法 75 条の 2 で延ばした年度は、延ばした月数に二を加えた月数）を経過した日」で、月で決まり休日でずれない（法人税法施行規則 59 条 2 項と同項 1 号。2026-10-24 に e-Gov 法令 API〈`https://laws.e-gov.go.jp/api/1/articles;lawNum=昭和四十年大蔵省令第十二号;article=59`〉の本文で確かめた）。12 月末の決算なら起算日は翌年 3 月 1 日、保存の最後の日は 8 年後の 2 月の末日（1 か月延ばすと 4 月 1 日と 8 年後の 3 月 31 日）。D-136 の Why (1) の法人の段を規則 59 条 2 項の形に書き直して通則法 10 条 2 項の句を外し、Why (1) の 2983 を 2981 に、Guarantee range の 3014 を 3012 にした。結論の 3000 日は変えない（2996 と 2981 に足り、3012 に足りない）。再計算（1 月 1 日の取引の日から保存の最後の日までの日数の最小と最大。個人・延ばさない法人・1 か月延ばした法人の順）: `python3 -c "from datetime import date as d,timedelta as t;R=range(2026,2060);p=[(d(y+8,3,15)-d(y,1,1)).days for y in R];c=[(d(y+8,3,1)-t(1)-d(y,1,1)).days for y in R];e=[(d(y+8,4,1)-t(1)-d(y,1,1)).days for y in R];print(min(p),max(p),min(c),max(c),min(e),max(e))"` → `2995 2996 2980 2981 3011 3012`。round 2 の段の式と `2983 3014` は当時の記録として残す。
- Opus #2（Contract Audit の「単位や種類の混在」の行と Review Focus の K1 が、数え方の置き場として Review Response を指す）: 2 か所とも「D-136 の Why (1)」を指す形にした。
- Opus #3（`log_cleanup` の summary「（{cutoff_date}以前）」が、cutoff の日の記録は残る実際の境と 1 日食い違う。保持の lane の前からある）: 本 lane では直さず、`docs/backlog.md` の「操作ログの保持を 7 年程度に延ばす」の項目の直後に「操作ログの掃除の記録「（{cutoff_date}以前）」を実際の境に合わせる」を起票した（起源: PR #198 の Plan Review round 3）。他の lane との所有の表の backlog の行と、Scope の正本の段に足した。
- Opus #4（境の日ちょうどの test が 0 時を跨ぐと偽の red）: Test Plan に「0 時の跨ぎ」の項を、Matrix の Test Matrix の前文に同じ注を足した（C2 の `keeps_recent`、C3 の 3000 日前、C5 の `log_last_cleanup_date` が今日。test の前後で `Local::now().date_naive()` が変わっていないことを確かめ、変わっていれば同じ日の中でやり直す等）。`cleanup_old_logs` は `Local::now().date_naive()` で今日を、`delete_old_logs` は同じく `Local::now()` から cutoff を求める（`src-tauri/src/mnt/log_manager.rs:23`、`src-tauri/src/db/system_repo.rs:225`）。
- Opus #5（72 §72.5 の手順番号とコード例の `// 6.` が古い）: §72.5 を現物に合わせ、`run` の setup の手順 5、DB の初期化（手順 3-4 の `prepare_database`）の後・自動バックアップの確認（手順 6 の `run_startup_auto_backup`）の前とし、コード例のコメントを `// 5. 操作ログ自動削除（DB初期化後に実行。失敗してもアプリ起動は続行）` にした（`src-tauri/src/lib.rs:1475`〜`:1481`）。コメントを目印にする test は `test_startup_spec_sfv_t3_database_init_failure_has_operator_message`（`lib.rs:800`、`mod bindings_generation_tests` の中。`:819` で `// 5. 操作ログ自動削除` を文字列で探す）で、§72.5 に名前を書いた（D-125 で設計正本は行番号で引かない）。Scope の正本の段の 72 に §72.5 を足した。同じ古い番号は 71 にもある（`docs/function-design/71-mnt-backup.md` の自動バックアップの追加箇所「MNT-02 操作ログ自動削除（ステップ6）の後、State管理（ステップ8）の前」）が、71 は本 lane の所有でないので直さず、Coordinator に報告する。
- Opus #6（数え方が D-136 と 72 に 2 重）: 72 MNT-02-D1 の Why から 63 条 4 項・3 月 15 日・2995/2996 の文を外し、「帳簿書類の 7 年の数え方と、それに要る日数（個人・法人）は D-136 の Why (1)」と引く形にした（Scope の D-127 の段に記録）。
- 是正で不要になった記述: 72 の Why の「（法人の場合との比べは D-136）」（数え方ごと D-136 を引く形に吸収）、D-136 の Why (1) の法人税法 74 条 1 項と通則法 10 条 2 項の句（保存の起算日の根拠でない）。
- 旧語の sweep（`rg -n '2983|3014|通則法|10 条 2 項|Review Response の (round 2 の )?K1|ステップ8|// 6\. 操作ログ' docs -g '!docs/archive/**' -g '!docs/research/**'`）: 残る hit は本 packet の round 2 の Review Response の段（当時の記録）と本 round の段、71 の自動バックアップの追加箇所（上の Opus #5、範囲の外）、他の章の「ステップ8」（35・71 の別の手順の番号で無関係）だけ。
- 境界の 4 観点（是正で新しく書いた契約文・数について）: 安全な整数の範囲・0 の表示・失敗の時点と順序は、本 round の是正が code の契約（`u32` の日数、0 で飛ばす、飛ばすとき日付を更新しない）に触れないので Contract Audit の表のまま。単位や種類の混在は、法人の起算日が月で決まり（規則 59 条 2 項）、runtime の保持は日のままで、月の数え方は既定値の根拠（D-136 の Why (1)）にだけ現れる。数は上の再計算の出力から写した。

Final Review と merge（2026-10-26、closeout で追記）: broad の review と、#188 の merge の後の base 同期での相互作用の closure、base 同期の closure を経て、最後の base 同期だけを見る closure 2 本（Codex 579 = approve P1/P2/P3 0、PR review 5650786523。Claude 側 fresh Opus 5.5 = approve 0）が head `af2448dc` で揃い、helper の review の record は pass（`docs/AGENT_OPERATING_MANUAL.md` §3.3 の Codex の枠が揃った）。Ready の後の hosted CI run 40314529448 の `Merge gate` が success で、owner の承認の後に helper 経由で merge した（[PR #198](https://github.com/kosei-w90607/inventory-system-desktop/pull/198)）。closeout で packet と Matrix を `docs/archive/plans/` へ移し、`docs/Plans.md` と `docs/backlog.md` に完了を書いた。
