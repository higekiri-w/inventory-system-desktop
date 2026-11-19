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
- EJ の欠け・保留・読めない file・食い違いを結果の文で名指しし、欠けと読めない file はその場の「EJ のファイルを選んで取り込む」で埋められる。応答待ちに閉じた後も、確認の窓の文で戻り方が分かる。
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

- BIZ-08-D8: `rg -n 'AdditionalImportConfirmationRequired' src-tauri/src/biz/daily_report_import_service` → 10 件（`mod.rs:120`、`commit.rs:56`、`parse.rs:210`、`tests.rs:308`・`:524`・`:582`・`:1854`・`:1872`・`:1893`・`:1924`）。分岐は `parse.rs:205-211`。基の束 `valid_files`（`tests.rs:93`）は精算回数 `0001` を持ち、`NoDuplicate` で確認の flag が `true` なら `commit.rs:49-53` が `ValidationFailed` にする。隔離した copy で D8 の分岐（束に精算回数があり、同じ日の completed が全部精算回数を持てば `NoDuplicate`）を入れた `cargo test --lib` は 13 failed（1626 passed、2026-11-20）で、13 本は Test Plan の D1〜D4 の行。`:1859`（NULL）は確認のまま、`:1898` は cache を直に作るので変わらない。同じ日の id の列の TX の中の再検査は `commit.rs:129-131` で、状態に依らない。同じ日の一覧 `same_date`（`parse.rs:192`、`sales_repo::find_daily_report_imports_by_report_date` の `:895-912`）は `status = 'completed'` だけを読み、行の `DailyReportImport` は `settlement_no` を持つ（`sales_repo.rs:201`）。
- migration: 最新は v13（`src-tauri/src/db/migration.rs:106-110`）、本 lane は v14。本 PR の外の open の PR は #291 だけで migration に触れない（`gh pr list --state open`、`git diff --name-only origin/main...origin/agent/win-test-premises`）。関門 16〜18・JAN の別名も migration を見込むが未起票で、22 §21 の規則（先に merge する側が小さい番号）に従う。
- scan: snapshot は `files_by_candidate` だけ（`mod.rs:222-226`）、手順 10 は未実装（`scan.rs:200-221`）。IO-09 は EJ を既に分類し（`io/register_sd.rs:46-48`・`:516`）、`XZ` 直下は窓に依らず・`XZ_BKUP` は窓の中で返す（`:398-401`）。形は private の `measure_shape`（`:355`、呼出し `:309`）。`parse_ej`（`io/ej_parser.rs:313`）は呼び手が無く、合成の EJ の組立ては同 tests の中だけ（`:1467-1590`）。
- 登録: `design_compliance_test.rs` の 24 の行（`:152-155`）・37 の行（`:207-217`）。UI の文書は `SKIP_DOCS`。command は `lib.rs` の `collect_commands!`（`:326-332`）と `generate_handler!`（`:1567-1573`）。
- 45 §45.6c は `#[tauri::command]` と書くが、写しと TX を伴う commit は `(async)`（§45.4 の「実行の thread」、UI-USW-D5 の 8。§45.6c はその前の文）。
- 操作ログの label は `operation-type-labels.ts:24-25` と 74（`ej_import` は無い）。離脱の block は `useDailyReportImportFlow.ts:195-198`、日報の invalidation は `invalidation-contract.ts:138` の `dailyReportImport`。

## Scope

- S1 migration v14（`src-tauri/src/db/schema_v14.rs`、`MigrationKind::Sql`、1 TX）: pos-tables の「SD の一括取込みと EJ の控えの契約」の `ej_records` を列・CHECK・`UNIQUE(machine_prefix, record_no)`・`printed_at` の index のまま作る。`migration.rs` に登録し、`expected_tables` に足す。22 に §22（MNT-03-D23。版の番号の規則は §21 と同じ）、DB_DESIGN の表の一覧に 1 行。
- S2 repo（新規 `src-tauri/src/db/ej_repo.rs`、24 §14.25 のうち (A-1) の分）: `NewEjRecord`・`EjRecordModeCode`・`EjRecordKindCode`・`EjNumberGap`、`find_ej_record_texts`・`insert_ej_record`・`list_ej_number_gaps`。型は `biz/mod.rs` で re-export。mode・kind の enum と CHECK の値の集合の parity を `finite_enum_contract_tests.rs` に足す（D-061）。
- S3 IO: `measure_shape` を `pub fn measure_register_sd_shape(bytes: &[u8]) -> RegisterSdShape` に改名し、`read_register_sd_file` と S4 が呼ぶ（29 §29.7.6 手順 3 の文も）。`ej_parser.rs` の tests の合成の EJ の組立て（`:1467-1590`）を `#[cfg(test)] pub(crate)` の module に移し、`ej_parser` の tests・S4・S6・S9 が使う（新しく書かない）。
- S4 EJ の取込み（新規 `biz/daily_report_import_service/ej.rs`）: 37 §37.11 の `EjSourceFile`・`EjImportPlan`・`EjImportResult`・`EjConflict`、`plan_ej_records`・`import_ej_records`（手順 1〜5 の 1 つの筋）。操作ログ `ej_import`（件数だけ）と、その label（`operation-type-labels.ts`・74）。
- S5 BIZ-08-D8: `parse.rs:205-211` を 37 §37.10 BIZ-08-D8 の規則にする（束の `settlement_no` が `Some` で、取得済みの `same_date` に `settlement_no` が NULL の行が無ければ `NoDuplicate`。`same_date` は completed だけなので取り消した NULL の取込みは数えない。新しい SQL の照会は足さない）。期待の変わる既存の 13 本は Test Plan の D1〜D4 の直し方で直し、それぞれの履歴・子行・副作用・stale の assertion は保つ。37 §37.3 手順 8、pos-tables §12b の「冪等性・同日追加方針」の 2 行目と Stage 2 手順 4・Stage 3 の「同日別bundle追加確認」、55 §55.0 の日報取込みの利用者フローの手順 4（`docs/function-design/55-ui-csv-import.md:37` の「同一日別bundleは既存分を残す追加確認を出す」）（どれも無条件の同日追加の確認。D8 の判定への参照にする）、55 UI-07-D13 を直す。37 §37.10 BIZ-08-D8 の NULL の判定の文を取得済みの `same_date` の行で決める形に直し、24 §14.18b（proposed・未実装の専用の照会）を消す（Design Readiness の (6)）。
- S6 scan と一括取込み: `scan.rs` の手順 9 の後に、列挙の EJ を手順 5 と同じ上限で `read_register_sd_file` で読み、読めた file を snapshot の `ej_files`（`SdSourceFile`）、`Io`・`TooLarge` の file 名を `ej_unreadable_files` に入れ、`ej_files` で `plan_ej_records` を呼んで `DailyReportSdScan.ej`（`RegisterSdEjSummary`、両方空なら None）を返す（37 §37.10 の「§37.9 に足すもの」の (A)）。`InvalidPath` は手順 5 と同じく全体の失敗。新規 `bulk.rs` に `commit_register_sd_bulk_import`（§37.10 手順 1 の (a)・(c)、手順 2・4・5・6。手順 3 と (b) は (B)）。
- S7 CMD（45 §45.6c のうち 2 本）: `commit_register_sd_bulk_import` と `import_ej_files` を `#[tauri::command(async)]` で足し（§45.6c の文も直す）、`RegisterSdBulkResponse`・`RegisterSdBulkItemDto`・`RegisterSdBulkOutcomeDto`（`ImportedZ004` を除く）・`EjFileRequest`。`Failed` は §45.7 の `From<BizError>` を 1 か所で当てる。`collect_commands!`・`generate_handler!` に登録し、`src/lib/bindings.ts` を再生成する。2 本を main thread の外の検査（`daily_report_import_cmd.rs:489` の `test_daily_report_cmd_req401_commit_and_rollback_run_off_main_thread`。名指しの数の固定を 4 に）に足す。
- S8 画面（55 UI-07-D17 のうち (A) の分）: 新規 `components/SdBulkImport.tsx` を SD の一覧（`DailyReportSourceStep`）の上に置き、reducer に一括の取込み中・結果の状態を足す。押せる条件・EJ の行・結果の文と label（icon と文字）、「EJ のファイルを選んで取り込む」（native の file dialog、`importEjFiles`、結果をその場に）、「日次売上を見る」。「EJ のファイルを選んで取り込む」は EJ の結果の `gaps` か `unreadable_files` が空でないときに出す（読めない file の文もこの操作を指すので、欠けが 0 でも出す）。応答待ちの止め: 一括の応答を待つ間と手で選ぶ EJ の応答を待つ間は、今の取込み中（`useDailyReportImportFlow.ts:195-198`）と同じ hook で route の移動・page の離脱（`useBlocker`）と native の close（`useWindowCloseGuard`）を止め、flow の `isBlocking` に含めてタブの切替えも止め（`CsvImportPage.tsx:51-55` の `onBlockingChange`）、ボタンを disabled にする。close の種類は `src/lib/window-close-guard.ts:12-17` に `sd-bulk-import`・`ej-import` を足し、確認の窓（`WindowCloseDialog`）の見出しは「取込みの途中です」、説明は次の文:
  - `sd-bulk-import`: 「取込みが終わるまでお待ちください。終わると閉じられます。いま閉じると、どこまで取り込めたかをこの画面で確かめられなくなります。日報は 1 つずつ確定するので、一部だけが入っていることがあります。閉じたときは、次にアプリを開いてから「売上データ取込み」の「日報取込み」タブで同じ SD を読み直し、「まとめて取り込む」が押せればもう一度押してください。取り込み済みの日報と控えは二重に入りません。」
  - `ej-import`: 「取込みが終わるまでお待ちください。終わると閉じられます。いま閉じると、取り込めたかどうかをこの画面で確かめられなくなります。閉じたときは、次にアプリを開いてから「売上データ取込み」の「日報取込み」タブで同じ SD を読み直し、「まとめて取り込む」が押せれば押してください。結果に欠けか読めないファイルの文が残っていれば、「EJ のファイルを選んで取り込む」で同じファイルを選び直してください。取り込み済みの控えは二重に入りません。」

  成功の後は `invalidationContract.dailyReportImport`。`saved: true` の食い違いの文は「前の控えを残しました」までにし、控え画面への link と「レシートの控えを見る」は (A-2) が足す（55 UI-07-D17 に書く）。
- S9 L3 の入力物: `src-tauri/src/io/l3_fixtures.rs` に、L3 の日 T から合成の SD の folder を書く ignore の test を足す（`write_b1_l3_fixtures` と同じ形と env）。日報は tracked の `tests/fixtures/daily-report/` の 3 本の日付と精算回数の行を置き換え、EJ は S3 の組立てで作る（CP932・CRLF、中身は `## L3` の表。`sd-c` は T+1 の束と、`sd-a` の `XZ` の EJ を T+1 の記録まで伸ばしたもの）。通常の test で固定の T で作り scan と plan で読み戻す。
- S10 正本と登録: 37 §37.10・§37.11・24 §14.25・45 §45.6c・pos-tables の proposed の注記を (A-1) の分だけ外し、(A-2) の分（閲覧の関数・command・UI-09c）に未実装の印を残す。37 §37.10 BIZ-08-D8 の「後続の runtime が直す所」の項目を消し、§37.9 手順 4 の「EJ は EJ の取込みの lane」と §37.9 末尾の「Z004 と EJ への引継ぎ」の EJ の分を §37.10・§37.11 への参照に直す。`docs/SCREEN_DESIGN.md:127` と 55 §55.0 の手順 2（「確認する」を押して手順 3 のプレビューへ進む文）の日々の主操作（「SD から読む→精算を選ぶ→確認→実行」）を一括取込みの操作列（「SD から読む→一覧を見て「まとめて取り込む」→結果」）に置き換え、個別の「確認する」は同日追加の確認の要る束・1 件ずつ取り込む道として残す。`design_compliance_test.rs` の 24 の行に `db::ej_repo`、37 の行に `::ej`・`::bulk`。90 を `cargo run --bin generate_traceability` で作り直す。test の ID は REQ-401（UI-07・BIZ-08・CMD-12）と設計の ID。

Non-scope: IO-09 の列挙・読取りの規則と IO-09-D3 の許可の列、`register_sd_move.rs`、Z004 と PLU の経路、`seed_demo.rs`。

## Acceptance Criteria

- AC1: Test Plan の各行の test が `cd src-tauri && cargo test`・`npm run test` で PASS し、Linux の hosted の `Merge gate` が通る。
- AC2: Test Plan の mutation を Writer が 1 つずつ入れ、`cargo test <Test Name>` の `FAILED`（UI は `npx vitest run <file>` の `FAIL`）を確かめて戻し、結果を PR の comment に残す。
- AC3: `bash scripts/doc-consistency-check.sh`（Command registry drift を含む）と `--target plan` が ERROR なし、`cd src-tauri && cargo run --bin generate_traceability -- --check` が ERROR 0、`bash scripts/check-workflow-git.sh` が OK、`git diff origin/main --stat -- tests/fixtures` が空。
- AC4: `## L3` の項目の PASS（owner）。Writer は L3 の build の前に `cargo check --release`。

## Design Readiness

- 引用する設計正本: Contract Ledger の節と、D-121、55 UI-07-D15、D-052、D-061、TD-296・TD-310。
- 足りる理由: 型・手順・文言・順・TX・失敗の扱いは正本が決めている。正本との違いは 6 つで、実装の commit で正本を直す: (1) §45.6c の 2 command を `(async)` にする（§45.4 と同じ理由。UI-USW-D5 の 8 の command の一覧にも足す）、(2) (A-2) の前は UI-07-D17 の控え画面への link を出さない、(3) scan の EJ は手順 10 の後でなく手順 9 の後に読む（手順 10 は未実装。順は結果を変えない）、(4) UI-07-D17 の「EJ のファイルを選んで取り込む」を欠けのときだけでなく、`gaps` か `unreadable_files` が空でないときに出す（(A-1) には控え画面の入口が無く、読めない file の文が指す操作がそこにしか無い）、(5) 一括と手で選ぶ EJ の応答待ちを止めに入れる: UI-USW-D5 の 1 の種類と 4 の表に `sd-bulk-import`・`ej-import`（S8 の文。一括は日報ごとの TX なので、今の表の「半端な状態は残らない」は当たらず、閉じた後は読み直し、押せれば押し直す。押せないこと・結果の文が無いことから取込みの完了を言わない〈TD-839〉）、55 §55.7 の止める範囲の種類と、タブの切替えの止めに 2 つの応答待ち、(6) BIZ-08-D8 の同じ日の精算回数の NULL の有無を、24 §14.18b の専用の照会でなく取得済みの `same_date` の行で決め、§14.18b を消す（§37.10 の「`DailyReportImport` の行の DTO は `settlement_no` を持たない」は今の事実と合わない。取り消した取込みを数えない条件は `same_date` の `status = 'completed'` で保つ）。新しい decision-log の項目は要らない。
- 前提・延期した gap: 締めのたびに出る EJ の保留の文と、保留だけで押せる「まとめて取り込む」が雑音かは L3 で決め、雑音なら UI-07-D17 の直し方（保留の文を出さず、保留をボタンの条件に入れない）にする（Gated Amendment）。欠けだけが残り EJ の 4 つの数が 0 の日は、一括が押せず手で選ぶ EJ に届かない（(A-1) だけの期間は店で使わず、店に入るときは関門 19 の (A-2) の欠けの入口〈UI-09c-D5〉がある）。採るときは欠けをボタンの条件に入れるか、手で選ぶ操作を一覧の側にも置き、S8 の close の文を見直す。
- 絶対保証の例外: 主張しない（伸びうるかは file 名だけ。BIZ-08-D9 の保証の範囲）。
- 判定: ready（計画の review の前）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| `ej_records`（MNT-03-D23） | pos-tables、22 §22（新規） | S1・S2 | M1〜M3 | — |
| BIZ-08-D9 取込み | 37 §37.11 手順 1〜5、24 §14.25、29 §29.7.6・IO-08 | S2・S3・S4 | E1〜E10 | L3-3 |
| BIZ-08-D8 | 37 §37.10、pos-tables §12b、55 §55.0 手順 4・UI-07-D13 | S5 | D1〜D5 | — |
| scan の EJ | 37 §37.10「§37.9 に足すもの」、45 §45.6c | S3・S6 | C1〜C3 | L3-1 |
| BIZ-08-D7 一括の順・止め方 | 37 §37.10 手順 1〜6 | S6 | B1〜B5 | L3-1 |
| CMD-12-D2（2 本） | 45 §45.4 の実行の thread・§45.6c・§45.7・§45.8 | S7 | K1〜K4、AC3 の Command registry drift | — |
| UI-07-D17（(A) の分） | 55 UI-07-D17 | S8 | U1〜U6 | L3-1〜L3-3 |
| BIZ-08-D10・D11 | 37 §37.10・§37.12 | 変えない | 既存の scan・tidy の test | 一括は SD に書かない |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| M1・M2 v14 の表（negative を含む） | 表・index・UNIQUE・CHECK が無い | Rust | migrate の後の列・index（`expected_tables`）、`mode='x'`・`record_no='12345'` の INSERT が失敗、有効な同じ `(machine_prefix, record_no)` を直に 2 回 INSERT して 2 回目が制約違反、別の `machine_prefix` の同じ番号は通る | index の行・CHECK・`UNIQUE` のどれかを消す |
| M3 enum と CHECK の parity | 片方だけに値を足す | Rust（mutation） | `finite_enum_contract_tests` の ej の 2 enum | enum だけに variant を足す（compile か assert が red） |
| E1 形の検査（negative） | LF・BOM・日時の外れを入れる | Rust | LF だけの file、`2026-02-30 12:00` の記録 → `unreadable_files`、0 件入る | 手順 1 (iii) の検査を消す |
| E2・E3 保留 | 書きかけを保存・最後が永久に入らない | Rust | `EJ261120.TXT`（手で選んだ形を含む）の最後は保留、`EJ261120_0001.TXT` は最後まで入り、最終改行なしは保留 | 判定を `sd_relative_path` の有無にする・全 file の最後を保留する |
| E4・E5 conflict（negative） | 上書き・片方の保存・書きかけを conflict に | Rust | 2 file の同じ鍵の別の文、保存済みと別の文、1 file に同じ鍵 2 回 → 入らず `saved` が正しい。保留だけで保存済みと違う → (d) | (a) で先の file の文を入れる・(a) を保留にも当てる |
| E6 状態の遷移 | 保留が次の読取りで入らない、2 度入る | Rust | 1 回目は保留、伸びた file の 2 回目に (c) で入る、3 回目は (b) | (c) で保存済みを見ない |
| E7 写しの失敗 | 写しの無い記録が入る | Rust | `app_data_dir` を file にして `SourceCopyFailed`、`ej_records` 0 件 | 写しを TX の後にする |
| E8 TX の中のやり直し | plan の後の取込みで UNIQUE の error | Rust（mutation） | `import_ej_records` の中の事前照合の後・TX の中の照合の前（`#[cfg(test)]` の差込み口）に同じ記録を入れる → `DatabaseError` にならず `already_count = 1`・追加 0 件 | TX の中で手順 3 をやり直さない（UNIQUE 違反で red） |
| E9 欠けとログ | 欠けを出さない、ログに番号・文 | Rust | 000101・000104 → 欠け 000102〜000103、`ej_import` の detail に番号・文が無い | `LAG` の差を 1 で比べる |
| E10 保存した本文 | 違う文を保存する（insert-only で後から補えない） | Rust | 合成の EJ 1 件を入口から入れ、`body_text` を見出し 2 行・行末の空白・`"\n"` のつなぎを含む独立の期待の文字列と完全一致で比べ、同じ行の `mode`・`kind`・`total_amount` も比べる | 見出しを 1 行にする・行末の空白を削る |
| D1 精算回数のそろう同じ日 | 毎回確認で止まる | Rust | `:267` `test_daily_report_req401_additional_confirmation_for_same_date_different_bundle`（途中の 2 回目の commit の flag も `false`）と `:1841` `test_req401_same_date_other_settlement_still_confirms` の期待を `NoDuplicate` に | 分岐を今のまま残す |
| D2 回数のそろう 2 本目の確定 | 正しい確定が `ValidationFailed` で止まり、後の assertion に届かない | Rust | 2 本目の確認の flag を `false` に: `:438` `test_daily_report_req401_commit_same_date_is_insert_only`、`:793` `test_daily_report_req401_rollback_keeps_same_date_sibling`、`:841` `test_daily_report_rollback_req401_logs_exact_selected_id_and_survives_log_failure`、helper `import(.., true)`（`:1174`）を使う `test_daily_report_req401_list_rows_carry_settlement_no`・`test_daily_report_req401_list_rows_carry_source_filenames_and_status`・`test_req401_source_filenames_accepts_missing_copy_fields`、`sales_service.rs:1866` `test_get_daily_sales_req501_summary_imports_from_fixture_bundles` | 分岐を今のまま残す |
| D3 NULL の取込みのある日（negative） | 確認を外す | Rust | `:1859` の 2 例。`:504` `test_daily_report_req401_commit_additional_unconfirmed_validation_failed`・`:562` `test_daily_report_req401_stale_additional_preview_same_bundle_conflicts` は前の取込みを `bundle_without_settlement` にして確認の経路（未確認の commit の `ValidationFailed`、stale の preview の `IdempotencyConflict`）を今のまま | 判定が `same_date` の NULL の行を見ない |
| D4 `NoDuplicate` の preview の後の同じ日の取込み | 確認なしの preview が、後から入った同じ日の取込みを見ずに確定する | Rust（mutation） | `:653` `test_daily_report_req401_snapshot_change_requires_repreview_without_side_effects`・`:722` `test_daily_report_req401_snapshot_replace_requires_repreview_without_side_effects` を、精算回数のそろう `NoDuplicate` の preview の後に同じ日の別の精算を flag `false` で入れ、前の preview の確定が副作用なしで止まる形に | TX の中の同じ日の id の列の再検査（`commit.rs:129-131`）を `AdditionalImportConfirmationRequired` のときだけにする |
| D5 取り消した NULL の取込み | 取消済みで確認が残る | Rust（mutation） | 同じ日に精算回数のある completed を残し、NULL の取込みを取り消した日 → `NoDuplicate` | `find_daily_report_imports_by_report_date` の `status = 'completed'` を外す |
| C1〜C3 scan の EJ | 読めない file を落とす・空の行・数の食い違い | Rust | `TooLarge` の EJ → `ej_unreadable_files` と数 1、EJ が無い → `ej` は None で一括の結果に EJ の項目なし、`new_count`・`held_count` が plan と同じ | `TooLarge` を飛ばす・空でも Some・`new_count` に保留を足す |
| B1 日報の順 | 新しい精算から入る | Rust | 同じ日の精算 6・5 と前の日 → 前の日・5・6 の順、parse の失敗は先頭で Skipped | 鍵の昇順を降順にする |
| B2 同じ鍵の順（決定的） | 実行ごとに入る束が変わる | Rust（mutation） | `XZ` と `XZ_BKUP` の同じ精算の別 bytes → `XZ` が入り他方が D4 で Skipped（100 回） | `files_by_candidate` の順のまま回す |
| B3 業務の拒否は続ける | 1 つの拒否で全部止まる | Rust | D4 の束の後の束と EJ が入る | `Skipped` で止める |
| B4 確定の段の失敗で止める | 失敗の後も続ける | Rust（mutation） | 2 つ目の束の写しの失敗 → `Failed`、後と EJ は `NotAttempted`、1 つ目は残る | `Failed` の後も続ける |
| B5 確認の要る束 | 確認なしで入れる | Rust | NULL の取込みのある日の束 → `NeedsIndividualConfirmation`、入らない | NoDuplicate として commit |
| K1〜K3 CMD（negative） | 古い scan・内部の文の漏れ・大きな file | Rust | 不正な UUID → `validation`、30 分超 → §45.6b の文、`SourceCopyFailed` → `internal` と §45.7 の文、手で選ぶ EJ の 20MB + 1 → `validation` | TTL を見ない・`Failed` に Display を入れる |
| K4 main thread の外 | 応答待ちの × に確認の窓を出せない | Rust（mutation） | `test_daily_report_cmd_req401_commit_and_rollback_run_off_main_thread` の名指しに 2 本を足して 4 | 2 本のどちらかの `(async)` を外す |
| U1 押せる条件 | 無いのに押せる・あるのに押せない | Vitest | `NotImported` 1・EJ の 4 つの数のどれか 1 → 押せる、全部 0 → 押せない | 条件から `unreadable_file_count` を外す |
| U2 応答待ち | 二度押し・離脱・タブの切替え・確認なしの close・閉じた後に戻る道が分からない | Vitest | 一括と手で選ぶ EJ のそれぞれの応答待ちで、ボタンが disabled、`useBlocker` の `shouldBlockFn` と `enableBeforeUnload` の両方が `true`（`useDailyReportImportFlow.test.tsx` の既存の検査を使う）、close の集合に `sd-bulk-import`・`ej-import`、`onBlockingChange(true)`（タブが disabled）。応答の後は `shouldBlockFn` と `enableBeforeUnload` の両方が `false` で全部解ける。close の確認の窓が種類ごとに S8 の文を出し、取込みの完了を言い切る文（「取り込めています」）を含まない（TD-839） | block を `importing` だけに戻す・`enableBeforeUnload` に応答待ちを足さない・`ej-import` を集合に入れない・close の文から条件の文を落とす・close の文に完了を言い切る文を足す |
| U3 結果の文 | 色だけ・取り違え | Vitest | 5 種の outcome の label と icon、止まったときの文 | `NotAttempted` を `Skipped` の label にする |
| U4・U5 EJ の結果 | 名指ししない・0 件の行・保留を注意で出す | Vitest | 欠けの区間（6 桁）・読めない file 名・`saved` の 2 つの文、0 件の行なし、保留の文に warning の icon・色なし | 0 件でも行を出す・保留を Alert にする |
| U6 手で選ぶ EJ | 欠け・読めない file を埋める道が無い | Vitest | 欠けがあるとき、と読めない file 1 本・欠け 0 のときに「EJ のファイルを選んで取り込む」が出て、`importEjFiles` の結果をその場に出す。どちらも 0 なら出さない | 出す条件を `gaps` だけにする・成功の後に結果を消す |
| L3 の入力物（S9） | 生成の誤りで L3 が取込みに届かない | Rust | 固定の T で `sd-a`・`sd-b`・`sd-c` を作り、scan と plan で読み戻す（`## L3` の表の数。`sd-b` は `sd-a` を取り込んだ DB で日報は全部取込み済み、`new_count`・`held_count`・`conflict_count`・`unreadable_file_count` が全部 0〈「まとめて取り込む」が押せない〉。`sd-c` は `sd-a` を取り込んだ DB で 1 回目に T の最後が新しい控えに入り、取り込んだ後の 2 回目が `new_count = 0`・`held_count = 1`・`conflict_count = 0`・`unreadable_file_count = 0`、日報は全部取込み済み） | 日付の置換を忘れる・`sd-b` に保留の残る EJ を入れる・`sd-c` に読めない file を入れる |

## L3

場は開発用の Windows の native build、空の DB。入力物は S9 で作る合成の SD の folder `sd-a`・`sd-b`・`sd-c` と手で選ぶ EJ 1 本（CP932・CRLF）。到達は「売上データ取込み」→ 日報取込みタブ →「場所を選ぶ」で folder → 一覧。表の行の順に行う（L3-2 の 2 回目を保留だけの状態にするため、欠けは L3-3 で先に埋める）。

| # | 入力 | 操作 | 合格基準 |
|---|---|---|---|
| L3-1 | `sd-a`（T−1 と T の束、T は精算 2 回。`XZ\EJ…`〈最後の記録が保留〉、`XZ_BKUP\EJ…_0001`〈欠け 1 区間〉、LF の読めない EJ 1 本） | 一覧を見る →「まとめて取り込む」→ 結果 | 押せると分かる。日報 3 行が「取り込みました」（同日追加の確認なし）、EJ の行に件数・欠けの区間・読めない file 名。文が自然で、色だけで分けていない |
| L3-3 | L3-1 の結果の欠けの文 | 「EJ のファイルを選んで取り込む」で手で選ぶ EJ を選ぶ | その場の結果で欠けが消え、取り込んだ件数が出る |
| L3-1b | 続けて `sd-b`（同じ束と `XZ_BKUP` の EJ だけ） | 一覧を見る | 「まとめて取り込む」が押せないと見分けられ、理由の文が分かる |
| L3-2 | `sd-c`（次の日の folder: T+1 の束と、T+1 の記録まで伸びた `XZ\EJ…`。読めない file と食い違いは無い）を 2 回読む | 1 回目: 一覧 →「まとめて取り込む」→ 結果。2 回目: もう一度読み、一覧 →「まとめて取り込む」→ 結果 | 1 回目で L3-1 の保留だった T の最後の控えが入る（件数に入り、保留の文は T+1 の最後の 1 件）。2 回目は保留だけの状態で、「まとめて取り込む」が押せ、結果は EJ の行の「新しい控えはありませんでした」と保留の文だけ（日報の行・欠け・読めない file の文は無い）。保留の文と保留だけで押せるボタンが雑音かは owner が判断し、雑音なら Design Readiness の直し方へ |
