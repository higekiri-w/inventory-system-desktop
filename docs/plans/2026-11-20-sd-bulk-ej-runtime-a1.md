# Plan Packet: SD の日報と EJ の一括取込み（(A-1)）

wave に属さない単独の lane。出所は [backlog](../backlog.md) の「SD の一括取込みと EJ の控えの runtime（A）」と D-121（設計 lane は PR #175）、owner の決定（repo 外の回答台帳 TD-808〈(A) を (B) より先に〉・TD-813 の (a)〈(B) は導入の前の関門 15、(A) はその前提〉・TD-296〈test の EJ は合成の fixture だけ〉）。起点は `2e058901`（origin/main、2026-11-20）。

大きさ: (A) を 2 本に分け、本 packet は (A-1)（SD の日報と EJ の一括取込み、UI-07-D17）だけ。(A-2)（EJ の控え画面 UI-09c、閲覧の 3 command）は backlog に残す。理由: (1) (A) 全体は migration・repo・BIZ 3 経路・CMD 5 本・画面 2 つ・route と navigation・新しい function-design の文書・REQ の採番に触れ、review が 1 回で読み切れない。(2) L3 が分かれる（(A-1) は取込みの操作と結果の文、(A-2) は控えの字形・等幅・探し方）。(3) 関門 15 の (B) が要るのは (A-1) の一括取込みと scan の snapshot だけ。(A-2) は (A-1) が入れた記録を読むので後（L3 で行を合成で入れられない。L3 Eligibility (3)）。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R3

Reason: migration（表の追加）、command 2 本と scan の wire、同日追加の確認の規則（BIZ-08-D8）、operator の取込みの流れ（Risk Tiers の R3）。R4 でない理由: 既存の行を写し直さず、取消・削除の経路を足さず（`ej_records` は insert-only）、SD へ書かない（BIZ-08-D10）。

## Goal

Goal Invariant:

### 最小完了条件

- 店の人が「SD から読む」の一覧で「まとめて取り込む」を 1 回押すと、取り込める日報の束が精算の順に入り、EJ の記録が `ej_records` に入り、項目ごとの結果が日本語の文で出る（37 §37.10・§37.11、55 UI-07-D17）。
- 同じ日に精算回数のそろう束どうしは同日追加の確認なしで入る（BIZ-08-D8）。精算回数の無い取込みが同じ日にある間は今どおり確認する。
- EJ の欠け・保留・読めない file・食い違いを結果の文で名指しし、欠けはその場の「EJ のファイルを選んで取り込む」で埋められる。
- (B) が Z004 を同じ流れに足せる（順と `RegisterSdBulkSource`・`RegisterSdBulkOutcome` の形が §37.10 のまま。(B) の分は足さない）。

### 失敗定義

- 書きかけの EJ を保存する、文の違う保存済みを上書きする、写しの無い SD の EJ を入れる。
- 確定の段の失敗（DB・写し）の後も続ける、業務の拒否で全体を止める。
- 同じ精算の日報を二重に入れる、精算回数の無い取込みのある日に確認を外す。
- 実の EJ・POS file・DB を fixture にする。

### 非目的

- (A-2) の全部（UI-09c、`list_ej_records`・`find_ej_records_by_no`・`get_ej_record`、`queryKeys.ejRecords`、控え画面への link、REQ の採番）。
- (B)（`z004_files`・`z004_new_count` 等の field と variant）、PLU の占有の照合（37 §37.9 手順 10、PLU の runtime の lane）、取込み済みの EJ を `XZ_BKUP` へ移す操作（backlog の「(b) の lane」）。
- EJ の小数の数量の受理（IO-08.5）、EJ を在庫・売上に使うこと。

## 確かめた事実（2026-11-20、`2e058901`）

- BIZ-08-D8: `rg -n 'AdditionalImportConfirmationRequired' src-tauri/src/biz/daily_report_import_service` → 9 件（`mod.rs:120`、`commit.rs:56`、`parse.rs:210`、`tests.rs:308`・`:524`・`:582`・`:1854`・`:1872`・`:1893`・`:1924`）。分岐は `parse.rs:205-211`。基の束 `valid_files`（`tests.rs:93`）は精算回数 `0001` を持つので、D8 の後は `:267`・`:504`・`:562` の test と `test_req401_same_date_other_settlement_still_confirms`（`:1841`）が `NoDuplicate` に変わる。`:1859`（NULL）は確認のまま、`:1898` は cache を直に作るので変わらない。
- migration: 最新は v13（`src-tauri/src/db/migration.rs:106-110`）、本 lane は v14。open の PR は #291 だけで migration に触れない（`gh pr list --state open`、`git diff --name-only origin/main...origin/agent/win-test-premises`）。関門 16〜18・JAN の別名も migration を見込むが未起票で、22 §21 の規則（先に merge する側が小さい番号）に従う。
- scan: snapshot は `files_by_candidate` だけ（`mod.rs:222-226`）、手順 10 は未実装（`scan.rs:200-221`）。IO-09 は EJ を既に分類し（`io/register_sd.rs:46-48`・`:516`）、`XZ` 直下は窓に依らず・`XZ_BKUP` は窓の中で返す（`:398-401`）。形は private の `measure_shape`（`:355`、呼出し `:309`）。`parse_ej`（`io/ej_parser.rs:313`）は呼び手が無く、合成の EJ の組立ては同 tests の中だけ（`:1467-1590`）。
- 登録: `design_compliance_test.rs` の 24 の行（`:152-155`）・37 の行（`:207-217`）。UI の文書は `SKIP_DOCS`。command は `lib.rs` の `collect_commands!`（`:326-332`）と `generate_handler!`（`:1567-1573`）。
- 45 §45.6c は `#[tauri::command]` と書くが、写しと TX を伴う commit は `(async)`（§45.4 の「実行の thread」、UI-USW-D5 の 8。§45.6c はその前の文）。
- 操作ログの label は `operation-type-labels.ts:24-25` と 74（`ej_import` は無い）。離脱の block は `useDailyReportImportFlow.ts:195-198`、日報の invalidation は `invalidation-contract.ts:138` の `dailyReportImport`。

## Scope

- S1 migration v14（`src-tauri/src/db/schema_v14.rs`、`MigrationKind::Sql`、1 TX）: pos-tables の「SD の一括取込みと EJ の控えの契約」の `ej_records` を列・CHECK・`UNIQUE(machine_prefix, record_no)`・`printed_at` の index のまま作る。`migration.rs` に登録し、`expected_tables` に足す。22 に §22（MNT-03-D23。版の番号の規則は §21 と同じ）、DB_DESIGN の表の一覧に 1 行。
- S2 repo（新規 `src-tauri/src/db/ej_repo.rs`、24 §14.25 のうち (A-1) の分）: `NewEjRecord`・`EjRecordModeCode`・`EjRecordKindCode`・`EjNumberGap`、`find_ej_record_texts`・`insert_ej_record`・`list_ej_number_gaps`。型は `biz/mod.rs` で re-export。mode・kind の enum と CHECK の値の集合の parity を `finite_enum_contract_tests.rs` に足す（D-061）。`sales_repo::exists_same_date_daily_report_import_without_settlement_no`（24 §14.18b）。
- S3 IO: `measure_shape` を `pub fn measure_register_sd_shape(bytes: &[u8]) -> RegisterSdShape` に改名し、`read_register_sd_file` と S4 が呼ぶ（29 §29.7.6 手順 3 の文も）。`ej_parser.rs` の tests の合成の EJ の組立て（`:1467-1590`）を `#[cfg(test)] pub(crate)` の module に移し、`ej_parser` の tests・S4・S6・S9 が使う（新しく書かない）。
- S4 EJ の取込み（新規 `biz/daily_report_import_service/ej.rs`）: 37 §37.11 の `EjSourceFile`・`EjImportPlan`・`EjImportResult`・`EjConflict`、`plan_ej_records`・`import_ej_records`（手順 1〜5 の 1 つの筋）。操作ログ `ej_import`（件数だけ）と、その label（`operation-type-labels.ts`・74）。
- S5 BIZ-08-D8: `parse.rs:205-211` を 37 §37.10 BIZ-08-D8 の規則にする（束の `settlement_no` が `Some` で、S2 の helper が false なら `NoDuplicate`）。`:267`・`:1841` の test は `NoDuplicate` を期待に直し、`:504`・`:562` は前の取込みを `bundle_without_settlement` にして確認の経路（未確認の commit の `ValidationFailed`、stale の preview の `IdempotencyConflict`）を今のまま確かめる。37 §37.3 手順 8・pos-tables §12b の「冪等性・同日追加方針」の 2 行目・55 UI-07-D13 を直す。
- S6 scan と一括取込み: `scan.rs` の手順 9 の後に、列挙の EJ を手順 5 と同じ上限で `read_register_sd_file` で読み、読めた file を snapshot の `ej_files`（`SdSourceFile`）、`Io`・`TooLarge` の file 名を `ej_unreadable_files` に入れ、`ej_files` で `plan_ej_records` を呼んで `DailyReportSdScan.ej`（`RegisterSdEjSummary`、両方空なら None）を返す（37 §37.10 の「§37.9 に足すもの」の (A)）。`InvalidPath` は手順 5 と同じく全体の失敗。新規 `bulk.rs` に `commit_register_sd_bulk_import`（§37.10 手順 1 の (a)・(c)、手順 2・4・5・6。手順 3 と (b) は (B)）。§37.9 手順 4 の「EJ は EJ の取込みの lane」を §37.10・§37.11 への参照に直す。
- S7 CMD（45 §45.6c のうち 2 本）: `commit_register_sd_bulk_import` と `import_ej_files` を `#[tauri::command(async)]` で足し（§45.6c の文も直す）、`RegisterSdBulkResponse`・`RegisterSdBulkItemDto`・`RegisterSdBulkOutcomeDto`（`ImportedZ004` を除く）・`EjFileRequest`。`Failed` は §45.7 の `From<BizError>` を 1 か所で当てる。`collect_commands!`・`generate_handler!` に登録し、`src/lib/bindings.ts` を再生成する。
- S8 画面（55 UI-07-D17 のうち (A) の分）: 新規 `components/SdBulkImport.tsx` を SD の一覧（`DailyReportSourceStep`）の上に置き、reducer に一括の取込み中・結果の状態を足す。押せる条件・EJ の行・結果の文と label（icon と文字）、欠けのときの「EJ のファイルを選んで取り込む」（native の file dialog、`importEjFiles`、結果をその場に）、「日次売上を見る」。取込み中は離脱を block し（`useDailyReportImportFlow.ts:195-198`）ボタンを disabled。成功の後は `invalidationContract.dailyReportImport`。`saved: true` の食い違いの文は「前の控えを残しました」までにし、控え画面への link と「レシートの控えを見る」は (A-2) が足す（55 UI-07-D17 に書く）。
- S9 L3 の入力物: `src-tauri/src/io/l3_fixtures.rs` に、L3 の日 T から合成の SD の folder を書く ignore の test を足す（`write_b1_l3_fixtures` と同じ形と env）。日報は tracked の `tests/fixtures/daily-report/` の 3 本の日付と精算回数の行を置き換え、EJ は S3 の組立てで作る（CP932・CRLF、中身は `## L3` の表）。通常の test で固定の T で作り scan と plan で読み戻す。
- S10 正本と登録: 37 §37.10・§37.11・24 §14.18b・§14.25・45 §45.6c・pos-tables の proposed の注記を (A-1) の分だけ外し、(A-2) の分（閲覧の関数・command・UI-09c）に未実装の印を残す。`design_compliance_test.rs` の 24 の行に `db::ej_repo`、37 の行に `::ej`・`::bulk`。90 を `cargo run --bin generate_traceability` で作り直す。test の ID は REQ-401（UI-07・BIZ-08・CMD-12）と設計の ID。

Non-scope: IO-09 の列挙・読取りの規則と IO-09-D3 の許可の列、`register_sd_move.rs`、Z004 と PLU の経路、`seed_demo.rs`。

## Acceptance Criteria

- AC1: Test Plan の各行の test が `cd src-tauri && cargo test`・`npm run test` で PASS し、Linux の hosted の `Merge gate` が通る。
- AC2: Test Plan の mutation を Writer が 1 つずつ入れ、`cargo test <Test Name>` の `FAILED`（UI は `npx vitest run <file>` の `FAIL`）を確かめて戻し、結果を PR の comment に残す。
- AC3: `bash scripts/doc-consistency-check.sh`（Command registry drift を含む）と `--target plan` が ERROR なし、`cd src-tauri && cargo run --bin generate_traceability -- --check` が ERROR 0、`bash scripts/check-workflow-git.sh` が OK、`git diff origin/main --stat -- tests/fixtures` が空。
- AC4: `## L3` の項目の PASS（owner）。Writer は L3 の build の前に `cargo check --release`。

## Design Readiness

- 引用する設計正本: Contract Ledger の節と、D-121、55 UI-07-D15、D-052、D-061、TD-296・TD-310。
- 足りる理由: 型・手順・文言・順・TX・失敗の扱いは正本が決めている。正本との違いは 3 つで、実装の commit で正本を直す: (1) §45.6c の 2 command を `(async)` にする（§45.4 と同じ理由）、(2) (A-2) の前は UI-07-D17 の控え画面への link を出さない、(3) scan の EJ は手順 10 の後でなく手順 9 の後に読む（手順 10 は未実装。順は結果を変えない）。新しい decision-log の項目は要らない。
- 前提・延期した gap: 締めのたびに出る EJ の保留の文と、保留だけで押せる「まとめて取り込む」が雑音かは L3 で決め、雑音なら UI-07-D17 の直し方（保留の文を出さず、保留をボタンの条件に入れない）にする（Gated Amendment）。
- 絶対保証の例外: 主張しない（伸びうるかは file 名だけ。BIZ-08-D9 の保証の範囲）。
- 判定: ready（計画の review の前）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| `ej_records`（MNT-03-D23） | pos-tables、22 §22（新規） | S1・S2 | M1〜M3 | — |
| BIZ-08-D9 取込み | 37 §37.11 手順 1〜5、24 §14.25、29 §29.7.6・IO-08 | S2・S3・S4 | E1〜E9 | L3-3 |
| BIZ-08-D8 | 37 §37.10、24 §14.18b、pos-tables §12b、55 UI-07-D13 | S2・S5 | D1〜D3 | — |
| scan の EJ | 37 §37.10「§37.9 に足すもの」、45 §45.6c | S3・S6 | C1〜C3 | L3-1 |
| BIZ-08-D7 一括の順・止め方 | 37 §37.10 手順 1〜6 | S6 | B1〜B5 | L3-1 |
| CMD-12-D2（2 本） | 45 §45.4 の実行の thread・§45.6c・§45.7・§45.8 | S7 | K1〜K3、AC4 の drift | — |
| UI-07-D17（(A) の分） | 55 UI-07-D17 | S8 | U1〜U6 | L3-1〜L3-3 |
| BIZ-08-D10・D11 | 37 §37.10・§37.12 | 変えない | 既存の scan・tidy の test | 一括は SD に書かない |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| M1・M2 v14 の表（negative を含む） | 表・index・UNIQUE・CHECK が無い | Rust | migrate の後の列・index（`expected_tables`）、`mode='x'`・`record_no='12345'` の INSERT が失敗 | index の行か CHECK を消す |
| M3 enum と CHECK の parity | 片方だけに値を足す | Rust（mutation） | `finite_enum_contract_tests` の ej の 2 enum | enum だけに variant を足す（compile か assert が red） |
| E1 形の検査（negative） | LF・BOM・日時の外れを入れる | Rust | LF だけの file、`2026-02-30 12:00` の記録 → `unreadable_files`、0 件入る | 手順 1 (iii) の検査を消す |
| E2・E3 保留 | 書きかけを保存・最後が永久に入らない | Rust | `EJ261120.TXT`（手で選んだ形を含む）の最後は保留、`EJ261120_0001.TXT` は最後まで入り、最終改行なしは保留 | 判定を `sd_relative_path` の有無にする・全 file の最後を保留する |
| E4・E5 conflict（negative） | 上書き・片方の保存・書きかけを conflict に | Rust | 2 file の同じ鍵の別の文、保存済みと別の文、1 file に同じ鍵 2 回 → 入らず `saved` が正しい。保留だけで保存済みと違う → (d) | (a) で先の file の文を入れる・(a) を保留にも当てる |
| E6 状態の遷移 | 保留が次の読取りで入らない、2 度入る | Rust | 1 回目は保留、伸びた file の 2 回目に (c) で入る、3 回目は (b) | (c) で保存済みを見ない |
| E7 写しの失敗 | 写しの無い記録が入る | Rust | `app_data_dir` を file にして `SourceCopyFailed`、`ej_records` 0 件 | 写しを TX の後にする |
| E8 TX の中のやり直し | plan の後の取込みで UNIQUE の error | Rust | plan と import の間に同じ記録を入れても `DatabaseError` にならず (b) | TX の中で手順 3 をやり直さない |
| E9 欠けとログ | 欠けを出さない、ログに番号・文 | Rust | 000101・000104 → 欠け 000102〜000103、`ej_import` の detail に番号・文が無い | `LAG` の差を 1 で比べる |
| D1 精算回数のそろう同じ日 | 毎回確認で止まる | Rust | `:1841` の test を `NoDuplicate` に | 分岐を今のまま残す |
| D2 NULL の取込みのある日（negative） | 確認を外す | Rust | `:1859` の 2 例、`:504`・`:562` の確認の経路 | helper が常に false を返す |
| D3 取り消した NULL の取込み | 取消済みで確認が残る | Rust（mutation） | NULL の取込みを取り消した日 → `NoDuplicate` | helper の `status = 'completed'` を外す |
| C1〜C3 scan の EJ | 読めない file を落とす・空の行・数の食い違い | Rust | `TooLarge` の EJ → `ej_unreadable_files` と数 1、EJ が無い → `ej` は None で一括の結果に EJ の項目なし、`new_count`・`held_count` が plan と同じ | `TooLarge` を飛ばす・空でも Some・`new_count` に保留を足す |
| B1 日報の順 | 新しい精算から入る | Rust | 同じ日の精算 6・5 と前の日 → 前の日・5・6 の順、parse の失敗は先頭で Skipped | 鍵の昇順を降順にする |
| B2 同じ鍵の順（決定的） | 実行ごとに入る束が変わる | Rust（mutation） | `XZ` と `XZ_BKUP` の同じ精算の別 bytes → `XZ` が入り他方が D4 で Skipped（100 回） | `files_by_candidate` の順のまま回す |
| B3 業務の拒否は続ける | 1 つの拒否で全部止まる | Rust | D4 の束の後の束と EJ が入る | `Skipped` で止める |
| B4 確定の段の失敗で止める | 失敗の後も続ける | Rust（mutation） | 2 つ目の束の写しの失敗 → `Failed`、後と EJ は `NotAttempted`、1 つ目は残る | `Failed` の後も続ける |
| B5 確認の要る束 | 確認なしで入れる | Rust | NULL の取込みのある日の束 → `NeedsIndividualConfirmation`、入らない | NoDuplicate として commit |
| K1〜K3 CMD（negative） | 古い scan・内部の文の漏れ・大きな file | Rust | 不正な UUID → `validation`、30 分超 → §45.6b の文、`SourceCopyFailed` → `internal` と §45.7 の文、手で選ぶ EJ の 20MB + 1 → `validation` | TTL を見ない・`Failed` に Display を入れる |
| U1 押せる条件 | 無いのに押せる・あるのに押せない | Vitest | `NotImported` 1・EJ の 4 つの数のどれか 1 → 押せる、全部 0 → 押せない | 条件から `unreadable_file_count` を外す |
| U2 取込み中 | 二度押し・離脱 | Vitest | 押した後 disabled、離脱の block | block を `importing` だけに戻す |
| U3 結果の文 | 色だけ・取り違え | Vitest | 5 種の outcome の label と icon、止まったときの文 | `NotAttempted` を `Skipped` の label にする |
| U4・U5 EJ の結果 | 名指ししない・0 件の行・保留を注意で出す | Vitest | 欠けの区間（6 桁）・読めない file 名・`saved` の 2 つの文、0 件の行なし、保留の文に warning の icon・色なし | 0 件でも行を出す・保留を Alert にする |
| U6 手で選ぶ EJ | 欠けを埋める道が無い | Vitest | 「EJ のファイルを選んで取り込む」で `importEjFiles`、結果をその場に | 成功の後に結果を消す |
| L3 の入力物（S9） | 生成の誤りで L3 が取込みに届かない | Rust | 固定の T で `sd-a`・`sd-b` を作り、scan と plan で読み戻す（`## L3` の表の数） | 日付の置換を忘れる |

## L3

場は開発用の Windows の native build、空の DB。入力物は S9 で作る合成の SD の folder `sd-a`・`sd-b` と手で選ぶ EJ 1 本（CP932・CRLF）。到達は「売上データ取込み」→ 日報取込みタブ →「場所を選ぶ」で folder → 一覧。

| # | 入力 | 操作 | 合格基準 |
|---|---|---|---|
| L3-1 | `sd-a`（T−1 と T の束、T は精算 2 回。`XZ\EJ…`〈最後の記録が保留〉、`XZ_BKUP\EJ…_0001`〈欠け 1 区間〉、LF の読めない EJ 1 本） | 一覧を見る →「まとめて取り込む」→ 結果 | 押せると分かる。日報 3 行が「取り込みました」（同日追加の確認なし）、EJ の行に件数・欠けの区間・読めない file 名。文が自然で、色だけで分けていない |
| L3-1b | 続けて `sd-b`（同じ束と `XZ_BKUP` の EJ だけ） | 一覧を見る | 「まとめて取り込む」が押せないと見分けられ、理由の文が分かる |
| L3-2 | `sd-a` を次の日を想定してもう一度 | 一覧と結果の保留の文 | 保留の文が毎日出ても雑音でないか（雑音なら FAIL とし、UI-07-D17 の直し方へ） |
| L3-3 | 結果の欠けの文 | 「EJ のファイルを選んで取り込む」で手で選ぶ EJ を選ぶ | その場の結果で欠けが消え、取り込んだ件数が出る |
