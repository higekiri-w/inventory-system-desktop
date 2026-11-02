# Plan Packet: 取り込んだ精算の Z をアプリが SD の `XZ_BKUP` へ移し、CV17 の毎日の Z の取込みをやめられるようにする

wave に属さない単独の lane（導入の前の関門 11）。設計の判断は起票の commit で設計正本に置いた（D-149〈proposed〉、29 §29.9 IO-11、37 §37.12 BIZ-08-D11、24 §14.18d、45 §45.6a、55 UI-07-D15 (2)・(7)）。runtime は同じ lane で、計画の固定の後に実装する。

出所: owner の決定（repo 外の回答台帳 TD-570、2026-11-02。関門の confirmed の 11 に足した）と TD-296 の 11a（v1.0 の前）。D-111 (1) の (b)（owner の決定 2026-10-07、TD-142）。設計の起点は D-111・D-114・D-121 の BIZ-08-D10 と、backlog の「取込み済みの Z・EJ を `XZ_BKUP` へ移す操作（(b) の lane）」とその D-121 の申し送り。

decision-log の番号: 本 lane は D-149 を予約する（main は D-147 まで。open の PR #216 が D-143 を足す。同じ日に起票する返品の lane〈関門 10〉が D-148 を取りうるので避けた。返品の lane が D を足さなければ、merge の前に繰り上げてよい）。

## Workflow State

- Risk: R4
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R4

Reason:
レジの SD の上の店の実データ（精算の Z の原本。SD が無いとレジは精算できない、29 §29.7.2 の SD-18）をアプリが初めて書き換える（改名・移動・folder の作成）。`docs/DEV_WORKFLOW.md` Risk Tiers の「破壊的なデータの lifecycle」に当たり、誤ると店の精算と原本を損ないうる。

## Goal

Goal Invariant:

### 最小完了条件

- 店の PC で「SD から読む」（自動で見つけた SD）を押すと、前に取り込んだ精算の Z001・Z002・Z005 と、同じ精算の名前の Z004 が、CV17 の取込みの後と同じ形の名前で SD の `XZ_BKUP\yyyy\mm\` に移り、PC の `pos-sources/` に同じ bytes の写しがある。
- 移した後の SD でも、アプリの読取り・取込み・二重取込みの拒否は今と同じ結果になる（移した file は `XZ_BKUP` の側で「取込み済み」）。
- 移せなかったときは、読取りと取込みは続き、画面が原因と直し方を名指す。
- これにより、店は CV17 の毎日の Z の取込みをやめられる（D-149 (7)。EJ は (A) まで月末の CV17）。

### 失敗定義

- 取り込んでいない日報の file、EJ、名前の規則に合わない file、`XZ_BKUP` の file を移す・名前を変える。
- SD の上の既存の file を置き換える・消す・中身を書く。
- PC に写せていない file を移す。
- 移した file が IO-09 の分類で `Unknown` になる、または scan の結果（候補・状態）が移す前と変わる（同じ bytes が「取り込めます」に戻る等）。
- 片付けの失敗で読取りや取込みが止まる、または失敗が画面に出ない。
- 「場所を選ぶ」で読んだ folder（PC の SD の写しを含む）を書き換える。
- `io::register_sd` に書込みの API が入る（IO-09-D3）。
- backup・restore の上書きしない公開の振舞いが変わる。

### 非目的

- EJ を移すこと（D-121 の (A) の後。保留した記録の確定と移す順、conflict を含む EJ の扱いはそのとき決める）。
- Z004 の取込み・PLU の占有の照合（(B)・D-117 の lane）。
- CV17 の連番の振り方に合わせること、CV17 と同じ folder に名前を付ける期間の衝突の防止（置き換えない移動で既存を守るだけ。D-149 の Guarantee range）。
- 移した file を `XZ` へ戻す操作、`XZ_BKUP` の古い file の掃除、空の `XZ\yyyy\mm` の folder の削除。
- 固定 disk に見える reader での片付け（「場所を選ぶ」の経路）。
- 窓より前の `XZ` の file。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## owner の判断事項

確認済みの事実: main（`c66ef81d`）では、アプリは SD を読むだけで（`src-tauri/src/io/register_sd.rs` の許可の列の検査 `test_req401_module_uses_only_allowed_fs_api`）、SD から取り込むのは日報（Z001・Z002・Z005）だけ。EJ の取込み（D-121 の (A)）と Z004 の SD からの取込み（(B)）・PLU の照合（D-117 の手順 10）は main に無い（`rg -n "plan_ej_records|plu_register_z004s|import_ej_records" src-tauri/src` で 0 件）。CV17 の Z の取込みは帳票の画面の操作、EJ は「電子ジャーナルを閲覧する」の別の操作（[project-memory](../project-memory.md) のレジの節）。下の推奨を既定にして設計正本を書いた。

**J1 何を移すか**

- 案 A: 取り込んだ Z001・Z002・Z005 だけ（BIZ-08-D10 の約束のまま）。Z004 は `XZ` に毎日 1 本ずつ溜まり、店は CV17 の Z の取込みを月末（EJ と同じ日）に続ける。`XZ` に約 1 か月の Z004 が溜まったときのレジの振舞いは未確認（D-111 の運用の制約の理由で試していない）。
- 案 B（推奨、設計の既定）: A に加え、同じ精算の名前（同じ folder・日・接尾字）の Z004 を、PC に写してから一緒に移す。EJ は移さない。店は CV17 の Z の取込みをやめ、EJ だけ月末に CV17 で取り込む（今の習慣のまま）。Z004 は今アプリが取り込まず（PLU を書く前）、移しても SD の `XZ_BKUP` と PC の写しに残り、scan は `XZ_BKUP` も読むので後の (B)・D-117 は落とさない。
- 案 C: EJ の取込み（(A)）の後まで本 lane の runtime を待ち、EJ も含めて全部を移す。CV17 の取込みを全部やめられ、CV17 とアプリが同じ folder に名前を付ける期間が無いが、関門 11 が (A) を待つ（(A) は関門に無い）。
- 推奨の理由: 目的（毎日の CV17 の取込みをやめる）を今の main の上で満たし、`XZ` に溜まる状態（未確認のレジの振舞い）を作らない。EJ の月末の CV17 は店の今の習慣で、D-111 の運用の制約の「EJ は約 1 か月 `XZ` に残っても精算が続いた実績」の範囲に収まる。
- 回答で変わる所: 案 A は 37 §37.12 手順 2 の Z004 の箇条を外す（Test Plan の Z004 の行を「移さない」へ）。案 C は本 lane を設計だけで止め、runtime を (A) の後に起こし直す（37 §37.12 に EJ の条件、29 §29.9 の名前の規則に EJ、D-149 の (2)・(7)）。

**J2 いつ移すか**

- 案 (a)（推奨、設計の既定）: 「SD から読む」の中で、候補を作る前に、前に取り込んだ分を移す。利用者の操作は増えない。その日の分は次の読取りで移る（`XZ` には前の読取りの後の精算の分だけが残る。CV17 を毎日使った今と同じ）。
- 案 (b): 取り込んだ直後に、SD がまだ差してあれば移す。SD をレジに戻した後の分は残るので、(a) も要り、仕組みが 2 つになる。
- 案 (c): 一覧に「SD を片付ける」のボタンを足す。毎日の操作が 1 つ増える（D-127 の使い手に押しつけない）。
- 回答で変わる所: (b)・(c) は 37 §37.12 の呼び手（§37.9 手順 2a）と 45 §45.6a、55 UI-07-D15 (2)・(7)、L3-1 の手順。

**J3 店の運用をいつ切り替えるか**

- 推奨（設計の既定）: 本 lane の版を店の PC に入れた後も、最初の数日は今の順（CV17 の取込み → 「SD から読む」）を続け、その間に片付けが失敗の表示なしで済み、レジの精算が普段どおり終わるのを店主か owner が確かめてから、CV17 の Z の取込みをやめる（D-149 (7)。F1）。EJ は (A) まで月末の CV17 のまま。移行の間は CV17 が先に移すので、アプリが移す物はほぼ無い（同じ形の SD のまま）。
- 案: 版を入れた日から CV17 の Z の取込みをやめる（確かめの日数が無い）。
- 回答で変わる所: D-149 (7) と 55 §55.0 の利用者フロー手順 2 の段の文。画面の文は変わらない。

**J4 移せたことを画面に出すか**

- 推奨（設計の既定）: 出さない。移せなかったときだけ原因と直し方を出す（55 UI-07-D15 (7)）。本数と path は診断ログ。毎日の画面に作業の報告を足さない。
- 案: 「前に取り込んだ分 N 本を SD の中で片付けました」を出す（owner が実機で確かめやすいが、毎日出る）。
- 回答で変わる所: 37 §37.12 の返り値（本数を wire に足す）、45 §45.6a、55 UI-07-D15 (7)。

**店の事実・実機で確かめること**（repo の資料で決まらない。どれも設計を分岐させず、F1・F2 は運用の切替え〈J3〉の前、F3・F4 は導入の前に確かめる）

- F1: アプリが移した後も、レジが次の精算で普段どおり `XZ` に Z を書き、精算が終わるか。CV17 の取込みの後と同じ SD の状態なので見込みは高い（推定）。J3 の最初の数日で、店主か owner が「精算が普段どおり終わった」を PASS / FAIL で答える。
- F2: CV17 の「電子ジャーナルを閲覧する」の取込みが Z に触れないか（EJ だけを移すか）。月末の EJ の取込みの前後の SD の写しの名前の一覧（中身は読まない）を比べれば決まる。触れるなら、CV17 がアプリの移した `XZ_BKUP` の folder に Z の名前を付けることになり、F5 が要る。
- F3: 店の PC の SD の読取り機（取外し可能な drive）の書込みの cache の方針（「クイック削除」か）と、店が SD を抜く前に「安全な取り外し」を使っているか。クイック削除なら書込みはすぐ SD に届く（Windows の既定。推定で、Microsoft の資料で確かめる）。片付けは `MOVEFILE_WRITE_THROUGH` で書き切るが、方針が違えば抜く前の案内を見直す。
- F4: SD が読めなくなったときのレジの側の戻し方（予備の SD、レジでの初期化の手順）。CV17 の取込みと同じ種類の危険なので、今の店の手順を確かめる（Recovery の前提）。
- F5（店が CV17 の Z の取込みを続ける場合だけ）: `XZ_BKUP` にアプリが移した名前があるとき、CV17 が同じ名前を上書きするか、別の連番を付けるか。owner の PC の予備の SD で試せる（店の SD を使わない）。J3 の推奨どおり CV17 の Z の取込みをやめれば要らない。

## Scope

runtime（計画の固定の後）:

- `src-tauri/src/io/mod.rs`: `pub mod register_sd_move;` と、71 §71.4.1 の置き換えない移動を下ろした `pub(crate) fn move_no_replace(source, destination) -> std::io::Result<()>`（Windows は `MoveFileExW(…, MOVEFILE_WRITE_THROUGH)`、Windows 以外は test だけの `hard_link` → `remove_file`。今の `mnt/restore.rs` の `StdRestoreFileOps::publish_no_replace` の本体を移す）。
- `src-tauri/src/mnt/restore.rs`: `StdRestoreFileOps::publish_no_replace` を `crate::io::move_no_replace` を呼ぶだけにする。flags の source の検査 test（`test_publish_no_replace_req901_d7_windows_flags_exclude_replace_existing`）は検査する本体を `io/mod.rs` の `move_no_replace` へ向け直し、mutant（`MOVEFILE_REPLACE_EXISTING` を足すと red）を保つ。`mnt/backup.rs` の呼出しと注入の型（`RestoreFileOps`）は変えない。
- `src-tauri/src/io/register_sd.rs`: IO-11 が使う読むだけの helper（`ErrorModeGuard`、相対 path の要素の検査と `inside_root`、Z の名前の分類）を `pub(crate)` にする。許可の列と検査 test は変えない。
- `src-tauri/src/io/register_sd_move.rs`（新規、29 §29.9）: `move_register_sd_file_to_backup` と `RegisterSdMoveError`、本 module の API の許可の列の source の検査 test（IO-11-D3）と単体 test。
- `src-tauri/src/db/sales_repo.rs`: 24 §14.18d `exists_completed_daily_report_source_file_hash` と test。
- `src-tauri/src/biz/daily_report_import_service/`: `mod.rs`（`RegisterSdTidyFailure`、`DailyReportSdScan.tidy_failure`）、`scan.rs`（引数 `app_data_dir` と手順 2a）、新規 `tidy.rs`（37 §37.12 `tidy_register_sd_imported`）、`tests.rs`（scan の呼出しの引数の追加と新しい test。L3 の tree の helper `write_l3_register_sd_tree` に Z004 と `XZ` 直下の EJ を足す）。
- `src-tauri/src/cmd/daily_report_import_cmd.rs`: `scan_register_sd` に `app: tauri::AppHandle` を足し、`app_data_dir` を BIZ へ渡す（45 §45.6a）。
- `src/lib/bindings.ts`: generator で作り直す（手で編集しない）。
- `src/features/daily-report-import/DailyReportImportPage.tsx`: 読取り中のボタンの文、読み終えた文、`tidy_failure` の文（55 UI-07-D15 (2)・(7)）。`DailyReportImportPage.sd.test.tsx` の期待の文と新しい test。reducer・hook は変えない（`tidy_failure` は scan の結果の一部として `sd_list` が持つ）。
- `src-tauri/tests/design_compliance_test.rs`: `build_doc_to_modules_map()` の `29-io-daily-report-parser.md` の行に `io::register_sd_move`。
- 設計正本: 37 §37.9 の本文へ「§37.9 に足すもの」を移す。29 §29.9・37 §37.12・24 §14.18d・45 §45.6a・55 の proposed の注記を外す。D-149 の Status を accepted に、D-111 に「Superseded in part by: D-149（運用の制約）」の行を足す。[project-memory](../project-memory.md) の「取込み済みの Z・EJ を `XZ_BKUP` へ移すアプリの操作…ができるまで」の行を D-149 (7) に合わせる。

起票の commit（本 commit）: 上の設計正本の proposed の節（D-149、29 §29.9、37 §37.12 と BIZ-08-D10 の答えの行、24 §14.18d、45 §45.6a、55 §55.0 手順 2・UI-07-D15 (2)・(5)・(7)、各 doc の更新履歴）、backlog の起票の行、本 packet。

Non-scope: `src-tauri/src/io/pos_source_copy.rs`（IO-10 はそのまま呼ぶ）、parse・commit・rollback の経路、`mnt/backup.rs`、migration、`Plans.md`（関門の完了は merged の PR が持つ）、CV17 の取込みの経路の画面（`CsvImportPage`）。

## Acceptance Criteria

- AC1: `XZ\yyyy\mm\` に取り込んだ束の Z001・Z002・Z005 と同じ名前の Z004 がある合成の SD の tree で、BIZ の scan（`Auto`）の後、4 本が `XZ_BKUP\yyyy\mm\` に `_0001` の名前で同じ bytes のままあり、`XZ` に無く、`pos-sources/casio-sr-s4000/sd/XZ/yyyy/mm/` に同じ bytes の写しがある。同じ scan の候補の状態は `Imported` で、`tidy_failure` は `None`。
- AC2: 取り込んでいない束、取り消した（rolled_back）取込みだけの束、精算回数が同じで bytes の違う束（`SameSettlementImported`）、EJ、`Unknown`、Z003 等の系列、`XZ_BKUP` の file は移らない（名前・場所・bytes が scan の前と同じ）。
- AC3: 移す先に同じ名前の file があれば、次の空いた連番で移し、既存の file の bytes を変えない。`move_no_replace` は既存の移す先を置き換えず `AlreadyExists` を返す。
- AC4: 写しの失敗を注入すると、その file 以後を移さず `tidy_failure = Some(PcCopy)`。SD の側の失敗（読取り専用・移動の失敗・確かめの食い違い）を注入すると、以後を移さず `Some(SdWrite)`。DB の読取りの失敗は何も移さず `Some(Other)`。どの場合も scan の候補は返る。
- AC5: `Selected` の scan は SD の tree を変えない（全 file の名前と bytes が前後で同じ）。
- AC6: 途中で止めた後（束の一部だけ移した）にもう一度 scan すると、残りが移り、候補の一覧に「ファイルがそろっていません」の行が出ない（`tidy_resumes_after_partial_move`）。
- AC7: IO-11 の名前の規則: `Z001_06 .CSV` → `Z001_06 _0001.CSV`（接尾字の空白を残す）、IO-09 の `list_register_sd_entries` が移した file を `XZ_BKUP` の Z（`backup_seq = Some(1)`）に分類する。`..`・root の外を指す link・`XZ_BKUP` が file の場合は移さない。
- AC8: `register_sd.rs` の許可の列の検査 test が変わらず PASS し、`register_sd_move.rs` の検査 test が許可の列の外（`remove_file`・`fs::rename`・`fs::copy`・`OpenOptions`・`std::io::Write` 等の mutant）で red になる。
- AC9: backup・restore の既存の test が全部 PASS し、`move_no_replace` の flags の検査 test が `MOVEFILE_REPLACE_EXISTING` の mutant で red になる。
- AC10: 画面: 読取り中のボタンが「SD を読んでいます…終わるまで SD を抜かないでください」、読み終えた文が「読み終わりました。SD はレジに戻してください（次の精算に要ります）。」、`tidy_failure` の 3 つの値で 55 UI-07-D15 (7) の文が出て、`None` なら出ない（vitest）。
- AC11: `bash scripts/doc-consistency-check.sh` と `--target plan docs/plans/2026-11-02-xz-bkup-move.md` が ERROR 0、`bash scripts/check-workflow-git.sh` が PASS。Rust・frontend・bindings・traceability の検証は `docs/DEV_WORKFLOW.md` の Verification Gates（hosted CI の `Merge gate`）。
- AC12: L3 の前に Writer が `cd src-tauri && cargo check --release` を通す（`cfg(windows)` の `MoveFileExW` の呼出しは WSL の test で compile されない）。

## Design Readiness

- 引用する設計正本（節まで）: D-149、D-111（(1)(b)・(8)・運用の制約・Revisit）、D-114（写しを backup に含めない）、D-121 の BIZ-08-D10、29 §29.7（IO-09、SD-02・03・08・11・18・22・23・25、IO-09-D2・D3・D5）・§29.8（IO-10）・§29.9（IO-11）、37 §37.9（BIZ-08-D3・D5・D6）・§37.12（BIZ-08-D11）、24 §14.18c・§14.18d、45 §45.6a（CMD-12-D1）、55 §55.0 手順 2・UI-07-D15、71 §71.4.1（上書きしない公開）。
- 必要な設計成果物: updated in this PR（上の proposed の節。runtime が本文へ移し proposed を外す）。DB・migration は不要（読むだけ。§14.18d）。
- plan にしかない durable な判断の昇格先: なし（判断は D-149・BIZ-08-D11・IO-11-D1〜D3 に置いた。owner の判断事項の回答は D-149 の owner の決定の行へ）。
- 前提・制約と延期した design gap: owner の判断事項 J1〜J4（既定で設計した）。店の事実 F1〜F5（設計を分岐させない。J3 の運用の切替えと導入の前の確かめ）。SD-23（取込み前の原本と取込み後の file が同じ bytes）は sd-direct-read-runtime の Contract Probe P1 の (b) で 2026-10-31 に合格（`docs/plans/2026-10-17-sd-direct-read-runtime.md`）で、移した file を scan が `Imported` と見る前提はこれに依る。EJ を移す設計は (A) の後へ延期（D-149 の Revisit）。
- 絶対保証の例外と抜け道: 「既存の file を置き換えない」は `MoveFileExW` に置き換えの flag を付けないことに依る（71 §71.4.1 の資料）。Windows 以外は test だけの経路。「取り込んでいない日報の file を移さない」は completed の取込みの file の hash に依り、別の取込みが同じ bytes の file を持てば移す（同じ bytes なので失うものは無い）。Z004 は名前で組むので、別の精算の Z004 を移しうる（失うものは無い。D-149 の Why）。途中で SD を抜いたときの FAT の状態は保証しない（CV17 と同じ種類。Recovery）。
- 判定: ready（J1〜J4 の回答が既定と違えば、上の「回答で変わる所」を計画の review の後に Gated Amendment で直す。J1 が案 C なら runtime を起こさず設計だけで閉じる）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| D-149 | decision-log | 全体 | 下の各行 | L3-1・L3-2 |
| IO-11-D1（CV17 と同じ形・置き換えない・改名だけ） | 29 §29.9.3 手順 4・5、IO-11-D1 | `io/register_sd_move.rs`、`io/mod.rs` の `move_no_replace` | `register_sd_move::moves_with_cv17_shape_name`、`register_sd_move::skips_taken_sequence_without_replacing`、`register_sd_move::listed_as_backup_after_move` | L3-1 |
| IO-11-D2（1 本ずつ確かめる） | 29 §29.9.3 手順 6 | 同上 | `register_sd_move::verify_detects_size_mismatch` | 非対象 |
| IO-11-D3（使う API の許可の列） | 29 §29.9.3 IO-11-D3 | 同上 | `register_sd_move::module_uses_only_allowed_fs_api` | 非対象 |
| IO-09-D3（`register_sd` は書かない） | 29 §29.7.6 | `io/register_sd.rs`（helper の公開だけ） | 既存 `test_req401_module_uses_only_allowed_fs_api` | 非対象 |
| IO-09-D5（root の外を辿らない） | 29 §29.7.6 | IO-11 手順 1〜3 | `register_sd_move::rejects_outside_root_and_bad_paths` | 非対象 |
| 71 §71.4.1 / MNT-01-D7（上書きしない公開） | 71 §71.4.1 | `io/mod.rs` の `move_no_replace`、`mnt/restore.rs` の委譲 | `move_no_replace_does_not_replace_existing`、向け直した `test_publish_no_replace_req901_d7_windows_flags_exclude_replace_existing`、既存の backup・restore の test | 非対象 |
| BIZ-08-D11（何を・いつ・失敗で止める） | 37 §37.12 | `biz/daily_report_import_service/tidy.rs`・`scan.rs` | 下の Test Plan の BIZ の行 | L3-1・L3-2 |
| BIZ-08-D10（取込み済みは中身で決める、EJ は (A) の前に移さない） | 37 §37.10 | 同上 | `tidy_moves_only_hash_matched_daily_files`、`tidy_leaves_ej` | 非対象 |
| BIZ-08-D3（scan の候補の規則） | 37 §37.9 | `scan.rs`（手順 2a の後の列挙は今どおり） | `scan_after_tidy_lists_moved_bundle_as_imported`、既存の scan の test | 非対象 |
| BIZ-08-D5（写し）・IO-10 | 37 §37.9、29 §29.8 | `tidy.rs` が `save_pos_source_copy` を呼ぶ | `tidy_copies_before_move`、`tidy_copy_failure_stops` | 非対象 |
| 24 §14.18d | 24 §14.18d | `db/sales_repo.rs` | `test_req401_exists_completed_daily_report_source_file_hash` | 非対象 |
| CMD-12-D1（CMD は渡すだけ） | 45 §45.6a | `cmd/daily_report_import_cmd.rs`、`src/lib/bindings.ts` | 既存の scan の CMD の test の引数の追加、bindings の drift の検査 | 非対象 |
| UI-07-D15 (2)・(5)・(7) | 55 | `DailyReportImportPage.tsx` | `DailyReportImportPage.sd.test.tsx` の下の Test Plan の行 | L3-1・L3-2 |
| UI-07-D12（CV17 の役割の文） | 55 §55.0 手順 2 | 画面の文は (2) のとおり。運用の文は docs | 非対象 | 非対象（運用は J3・F1） |
| D-114（写しを backup に含めない） | 71 §71.1、D-114 | 変えない | 非対象 | 非対象 |
| 隣接の除外: Z004 の取込み（32 の新契約）、EJ の取込み（§37.11）、PLU の照合（D-117） | 32、37 §37.11、33 §16.3 | 変えない | 非対象 | 非対象 |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| IO-11-D1 | 名前が CV17 の形でない・接尾字の空白が落ちる | unit | `register_sd_move::moves_with_cv17_shape_name` | 連番の桁・`_` の位置・空白の扱いを変える |
| IO-11-D1 | 既存の file を置き換える | unit | `register_sd_move::skips_taken_sequence_without_replacing` | 連番の選び方を「常に 0001」にする、`move_no_replace` を `std::fs::rename` にする（mutation） |
| IO-11-D1 | 移した file を IO-09 が `Unknown` にする | unit | `register_sd_move::listed_as_backup_after_move` | 名前の規則か置き場所（`XZ_BKUP\yyyy\mm`）を変える |
| IO-11 手順 3 | `XZ_BKUP`・年・月の folder が無いと失敗する／`XZ_BKUP` が file のとき上書きする | unit | `register_sd_move::creates_missing_backup_folders`、`register_sd_move::backup_area_not_directory_is_error` | `create_dir` の段を外す、directory の確かめを外す |
| IO-11 手順 1・2・IO-09-D5 | EJ・`Unknown`・`XZ_BKUP` の file・`..`・root の外への link を移す | unit | `register_sd_move::rejects_outside_root_and_bad_paths` | 名前の規則の検査か root の下の確かめを外す |
| IO-11-D2 | 移動の後の食い違いを見逃す | unit | `register_sd_move::verify_detects_size_mismatch` | 手順 6 を外す（`expected_len` を違えて渡すと `Verify`） |
| IO-11-D3 | この module に削除・複写・置換えの API が入る | unit（source の検査） | `register_sd_move::module_uses_only_allowed_fs_api` | `remove_file`・`fs::rename`・`fs::copy`・`OpenOptions`・`std::io::Write`・改名した `use` の mutant が red にならない |
| 71 §71.4.1 | 移す先を置き換える | unit | `io::move_no_replace_does_not_replace_existing`、向け直した flags の検査 test | Windows の flags に `MOVEFILE_REPLACE_EXISTING` を足す（mutation）、hard_link の経路を rename にする |
| 24 §14.18d | rolled_back の file を数える・別の key を見る | unit（DB） | `test_req401_exists_completed_daily_report_source_file_hash` | `status` の条件か `$.file_hash` の path を外す |
| BIZ-08-D11 手順 2 | 取り込んでいない・取り消した・`SameSettlementImported` の束を移す | unit（BIZ、一時 directory の tree） | `tidy_moves_only_hash_matched_daily_files` | hash の照合を束の名前の一致や `Incomplete` を含む条件に替える |
| BIZ-08-D11 手順 2（J1 案 B） | Z004 を移さない／別の日・接尾字の Z004 を移す | unit | `tidy_moves_z004_of_same_settlement_name` | 組の鍵（folder・日・接尾字）を変える。J1 が案 A なら期待を「移さない」に替える |
| BIZ-08-D10 | EJ・Z003 等・`Unknown` を移す | unit | `tidy_leaves_ej_and_other_series` | 対象の系列の条件を外す |
| BIZ-08-D5・D11 手順 3a | 写す前に移す・写しの失敗で移す | unit（失敗の注入） | `tidy_copies_before_move`、`tidy_copy_failure_stops` | 3a と 3b の順を入れ替える、`PcCopy` で続ける（mutation） |
| BIZ-08-D11 手順 3b | SD の失敗の後も移し続ける | unit（読取り専用の file・失敗の注入） | `tidy_sd_failure_stops_rest` | 失敗の後の `break` を外す、`SdWrite` と `Other` の分け方を変える |
| BIZ-08-D11 手順 2 | DB の失敗で一部だけ移す | unit | `tidy_db_failure_moves_nothing` | 対象を決めながら移す形にする |
| BIZ-08-D11（restart） | 途中で止めた後に残りが移らない・`Incomplete` が残る | unit | `tidy_resumes_after_partial_move` | 束の hash で決める形にする |
| BIZ-08-D11（Selected） | 「場所を選ぶ」の folder を書き換える | unit | `scan_selected_does_not_touch_tree` | 手順 2a の `Auto` の条件を外す |
| BIZ-08-D3 | 移した後の scan の結果が変わる | unit | `scan_after_tidy_lists_moved_bundle_as_imported` | 手順 2a を列挙の後に置く、移した file の bytes を変える |
| BIZ-08-D11 手順 1 | 列挙の失敗を片付けの失敗として重ねて出す | unit | `tidy_listing_error_returns_none` | 列挙の `Err` で `Some(Other)` を返す |
| UI-07-D15 (2) | 読取り中・読み終えた文が古いまま | vitest | `DailyReportImportPage.sd.test.tsx` の `shows do-not-remove text while scanning` と読み終えた文の期待 | 文を替えない、条件つきの文を残す |
| UI-07-D15 (7) | 失敗が出ない・成功で文が出る・message で分岐する | vitest | `shows tidy failure text per kind`、`shows nothing when tidy_failure is null` | 3 つの値の文の対応を変える、`null` で文を出す |
| CMD-12-D1・bindings | wire の型が生成物と食い違う | CLI | `bash scripts/doc-consistency-check.sh`（Command registry drift）と bindings の再生成の差分の検査（CI） | `tidy_failure` を手で bindings に書く、command の登録が片方だけ |

状態の遷移: 片付けは scan の 1 回の呼出しの中で終わり、画面の状態（`scanning` → `sd_list`）は増やさない。initial（移す物なし → `None`）、success（全部移す → `None`）、failure（3 種）、retry（もう一度「SD から読む」で残りが移る = `tidy_resumes_after_partial_move`）、revisit（移した後の scan = `scan_after_tidy_lists_moved_bundle_as_imported`）を上の行が持つ。

## L3

Windows の実機で、owner の PC と予備の SD カード（か USB メモリ。取外し可能な drive に見えるもの）を使う。店の SD・店の PC は使わない（F1 は J3 の運用の確かめで扱う）。入力物は合成の tree で、`write_l3_register_sd_tree`（`INVENTORY_L3_SD_TREE_OUT`、既存の ignore の test に Z004 と `XZ` 直下の EJ を足す）が作る CP932・CRLF の file を、予備の SD の root に写して使う。DB は開発用の空の DB。

- L3-1 / 売上データ取込み画面の日報取込みタブ / 予備の SD を差して「SD から読む」→ 一覧の「取り込めます」の 1 行を取り込む → もう一度「SD から読む」 / 2 回目の読取りの間のボタンが「SD を読んでいます…終わるまで SD を抜かないでください」、読み終えた文が「読み終わりました。SD はレジに戻してください（次の精算に要ります）。」で、片付けの失敗の文が出ない。エクスプローラで SD を開くと、取り込んだ精算の Z001・Z002・Z005 と同じ名前の Z004 が `CASIO\SR500_550_4000\XZ_BKUP\yyyy\mm\` に `_0001` の付いた名前であり、`XZ\yyyy\mm\` に無い。取り込んでいない精算の file と `XZ` 直下の EJ は `XZ` に残る。一覧の取り込んだ行は「取込み済み」。
- L3-2 / 同じ画面 / L3-1 の後に別の 1 行を取り込み、SD の書込み禁止のつまみを「LOCK」にして差し直し、「SD から読む」 / 一覧が出て、その下に「取り込んだ分を SD の中で片付けられませんでした。SD の横のつまみが「LOCK」の側になっていないか確かめ…」の文が出る。つまみを戻して差し直し、もう一度読むと文が消え、その行の file が `XZ_BKUP` へ移る。読取り機がつまみを無視して書けてしまうときは、この項目を「対象外（読取り機がつまみを見ない）」と記録する（故障注入の道具は使わない）。

## Recovery

- 移した file は消えない: bytes は SD の `XZ_BKUP` に同じまま残り（改名だけ）、移す前に PC の `pos-sources/casio-sr-s4000/sd/` に写しがある。アプリの取込みは `XZ_BKUP` も読むので、誤って移した file も取り込める（取り込んでいない束を移す誤りがあっても、scan は `XZ_BKUP` の側で「取り込めます」と出す）。
- 店の運用の戻し方: 片付けに問題があれば、店は CV17 の Z の取込みを今の順（CV17 の取込み → 「SD から読む」）で再開する。CV17 が先に移すので、アプリが移す物は無くなる（設定の切替えは要らない）。J3 のとおり CV17 の Z の取込みをやめるのは確かめの後なので、切り替える前の版の問題はこの順のまま吸収される。
- code の戻し方: 本 lane の PR を revert した版を入れる。既に移した file は `XZ_BKUP` に残り、CV17 の取込みの後と同じ状態なので戻さない（`XZ` へ戻す操作は作らない）。DB の schema を変えないので、版を戻しても DB はそのまま開ける。
- SD が読めなくなったとき（途中で抜いて FAT が壊れた等）: CV17 の取込みと同じ種類の危険で、レジの側の戻し方は店の今の手順（F4。導入の前に確かめる）。アプリの側の原本は PC の写し（移した分）と取込みの DB にある。移していない `XZ` の file は SD の側にしか無い（今の CV17 の運用と同じ）。
- 診断ログに、移した file と移せなかった file の元と先の相対 path が残るので、どの file がどこにあるかを後から追える。

## Data Safety

- commit しないもの: 店の SD の file・写し・manifest、実 DB、診断ログ、`pos-sources/` の中身。test と L3 の入力は合成の値だけ（`write_l3_register_sd_tree` の tree。tracked の fixture を足さない）。
- local だけの path: L3 の合成の tree の出力先（`INVENTORY_L3_SD_TREE_OUT` に与える repo の外の folder）と予備の SD。店の SD は L3 で使わない。
- 診断ログは相対 path と本数と失敗の種類だけを残し、file の中身・金額を残さない。
- 店の事実の問い（F1〜F5）の答えは repo の外の回答台帳に残し、repo には TD 番号と要旨だけを書く。
