# Test Design Matrix: SD 直読みの runtime

Plan Packet: [2026-10-17-sd-direct-read-runtime](../2026-10-17-sd-direct-read-runtime.md)。出発点は設計 lane の Matrix（`docs/archive/plans/test-matrices/2026-10-06-sd-direct-read.md`）で、全行を写し、2026-10-17 の HEAD `f5d7aaf4` の正本と現物で確かめ直した。足した行は「2026-10-17 追加」と書いた（IO-07-D5 の `i64` の範囲、§37.3 手順 9 の `sd_source_files`、BIZ-08-D5 の拒んだ束の写し、22 §17 の migration の詳細、UI-07-D16 の選択の一覧の細部、`selectFiles` を外すこと）。37 §37.9 手順 10 と 45 §45.6a 手順 2a（D-117 の占有の照合）の行は持たない（PLU の書出しの見直しの runtime の lane）。test 名は予定（新設）。既存の test として引用するものは `rg` で確かめた。test の SD の tree・Z の bytes はすべて合成で、名前の形だけを 2026-10-06 版での SD の採取（29 §29.7.2）に合わせる。oracle（固定の文・状態・path）は設計正本（29・37・45・55・22）から写し、実装の出力から作らない。2026-10-23 に main `eed07d37` を取り込んで当て直し、29 §29.7.4 手順 1 の 2026-10-23 の文（thread の error mode、71 MNT-01-D10 と共有する folder 名の定数）の行を足した（「2026-10-23 追加」）。同じ日の Plan Review round 1 の是正で、reducer の SD から来たことの持ち越し（55 UI-07-D15 (5)）・path の文字列の判定（29 §29.7.4 手順 4）・source の検査の red の形・folder 名の定数の置き場の行を直し・足した（「round 1」）。同じ日の Plan Review round 2 の是正で、error mode の guard の範囲（29 §29.7.4 手順 1）・source の検査の参照の規則（IO-09-D3）・folder 名の検査の対象の file・手で選んだ preview の選び直し（55 UI-07-D16 (a)）・版の期待を持つ既存の test の形（packet の S2）を直した（「round 2」）。round 2 の Codex 側の是正で、読取りの上限の読んだ byte の数・結果から一覧へ戻る遷移（55 UI-07-D15 (6)）・手で選んだ preview の選び直しの後の再解析の行を直し・足した（「round 2 Codex」）。Plan Review の closure の Codex 側の是正で、root の下かを実体の path で比べる行（29 IO-09-D5）と、その `InvalidPath` の固定の文（37 §37.9 手順 3）を足した（「closure」）。

## Risk

Risk: R3

## Contracts Under Test

- IO-07-D5（束の `settlement_no`）
- IO-09-D1〜D5（root の発見、名前の分類、読取り専用・書込み API なし・path の封じ込め・上限、形の測定、root の下かを実体の path で比べる〈closure〉）
- 29 §29.7.4 手順 1 の 2026-10-23 の文（IO-09 の関数が file system を見る間の thread の error mode〈round 2 で 5 関数の guard に広げた〉、71 MNT-01-D10 と共有する folder 名の定数、D-114）
- IO-09 §29.7.4 手順 4（手で選んだ file が SD の root の下か）と BIZ-08 §37.3 手順 1a（手で選んだ SD 上の file を SD の入力にする）
- 24 §14.14・§14.18a（`settlement_no` の INSERT と、精算回数で照合の候補を取る repository 関数）
- IO-10 §29.8.3（初回に保存先の directory を作ってから書く）
- BIZ-08-D3（SD の候補: 窓、組分け、状態、snapshot）
- BIZ-08-D4（同じ精算を別の bytes で取り込まない。preview と commit の TX）
- BIZ-08-D6（SD の経路は精算回数のある束だけ。scan・preview・commit）
- CMD-12-D1（scan cache、2 command）
- UI-07-D12・D15・D16（SD から読む画面、状態の表示、SD を戻す案内、予備のファイル選択。D16 の選択の一覧の細部は 2026-10-17 に 55 へ埋めた）
- 22 §17 MNT-03-D15（`daily_report_imports.settlement_no` を足す migration）
- §37.3 手順 9（`sd_source_files` を詰める）、BIZ-08-D5 の保証の範囲（TX の中で拒んだ束の写しが残りうる。取り込んだかは `copy_path`）

## Failure Modes

- 観測した名前の場合（接尾字 A/B、`XZ_BKUP` の `_nnnn`、大文字の `EJ`、同じ日の複数の Z、未知の名前）を落とす・誤って読む
- 状態の値（系列の有無、CP932・CRLF・BOM、`RESERVE`・`XZ_BKUP` の有無）を決め打ちにして、外れた file を推測で読む
- 取込み済みを名前で判定し、`EcrDatas` から取り込み済みの分を「取り込めます」と出す
- 取込み前の原本と取込み後の file は同じ bytes と推定する（SD-23、推定・強）が、推定が外れて bytes が違うとき、同じ精算を 2 回取り込める
- 同じ日の 2 回目の精算を誤って「取込み済み」にする
- SD の経路で精算回数の無い束（`settlement_no` が None）を取り込み、後で別の bytes の同じ精算が照合をすり抜ける
- 取込み済み・取り込めない候補を snapshot に入れ、選べてしまう
- 写しの失敗の後に preview token を失い、同じ preview で再試行できない
- SD に書く、または root の外を読む（root の下の link・junction が外を指すときを含む）
- 読取りの途中の失敗で途中までの一覧を出す
- CMD が IO を直接呼ぶ、規則が CMD / UI にある

## Test Matrix

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| IO-07-D5 | 3 本の精算回数がそろうのに None | unit | `daily_report_parser::settlement_no_is_some_when_all_three_agree` | 3 本とも `精算回数` が `0005`・`5`・`05` の束で `Some(5)` を返さない |
| IO-07-D5 | 1 本読めないのに Some | unit | `daily_report_parser::settlement_no_is_none_when_any_unreadable` | Z001 だけ精算回数の行があり、Z002 / Z005 が layout B の束で `Some` を返す |
| IO-07-D5 | 不一致でも Some | unit | `daily_report_parser::settlement_no_is_none_on_mismatch` | `settlement_mismatch` の束で `Some` を返す |
| IO-07-D5（2026-10-17 追加） | `i64` に収まらない値を Some にする・0 を落とす | unit | `daily_report_parser::settlement_no_out_of_i64_is_none_and_zero_is_some` | 3 本とも `9223372036854775808`（`i64::MAX` + 1）の束で `Some` を返す（`as i64` で負の値に化ける mutant で red）、3 本とも `9223372036854775807` の束で `Some(9223372036854775807)` にならない、3 本とも `0000` の束で `Some(0)` にならない。どの束も `settlement_mismatch` を出さない（D3 の判定を変えない） |
| IO-09-D1 | 選んだ path から root を解決できない | unit | `register_sd::resolve_accepts_drive_casio_and_model_folder` | 一時 directory の drive 相当・`CASIO`・`SR500_550_4000`（大文字小文字を混ぜる）の 3 通りのどれかで root にならない |
| IO-09-D1 | 別の folder を root にする | unit | `register_sd::resolve_rejects_non_register_folder` | `CASIO\SR500_550_4000` の無い folder で `NotRegisterSd` にならない |
| IO-09-D2 | 接尾字・連番の名前を落とす | unit | `register_sd::list_classifies_observed_names` | `XZ\2026\10\` の `Z001_06 .CSV`・`Z001_06A.CSV`・`Z001_06B.CSV` と `XZ_BKUP\2026\10\Z001_06 _0001.CSV`、`XZ\EJ261006.TXT`、`XZ_BKUP\EJ261005_0001.TXT`、小文字の `ej261004.txt`・`Ej261003.TXT`（`XZ` 直下）のどれかが期待の kind・area・date にならない |
| IO-09-D2 | 未知の名前を Z / EJ として読む | unit | `register_sd::list_returns_unknown_without_reading` | `XZ\2023\02\` の規則外の名前、`XZ` の連番つきの Z、`XZ_BKUP` の連番なしの Z、暦日として不正な `Z001_31 .CSV`（`2026\02\`）が `Unknown` にならない。読取りの関数が呼ばれる（test の stub で数える） |
| IO-09-D2 | 系列を決め打ちにする | unit | `register_sd::list_keeps_other_series` | `Z006_06 .CSV`・`Z123_06 .CSV` が返らない、またはエラーになる |
| IO-09-D2 | 窓の外を返す・窓の中を落とす | unit | `register_sd::list_filters_by_from_month_and_date` | `from` = 2026-10-03 で `XZ\2026\09\` の Z が返る、`XZ\2026\10\Z001_03 .CSV` が返らない、`XZ` 直下の古い日付の EJ が返らない |
| IO-09-D2 | `XZ_BKUP` が無いと失敗 / `XZ` が無いのに続ける | unit | `register_sd::list_area_presence` | `XZ_BKUP` の無い tree でエラー、`XZ` の無い tree で `MissingSalesArea` 以外 |
| IO-09-D3 | SD に書く | unit | `register_sd::scan_leaves_tree_unchanged` | list と read の前後で、一時 tree の全 file の名前・size・内容の hash・更新時刻が 1 つでも変わる |
| IO-09-D3 | 許可の列にない file system の API を使う | unit（source の検査） | `register_sd::module_uses_only_allowed_fs_api` | `src-tauri/src/io/register_sd.rs` の `std::fs` / `fs::` / `File::` / `OpenOptions` / `std::io::Write` / `windows_sys` の出現のどれかが、29 §29.7.6 IO-09-D3 の許可の列（`File::open`・`read_dir`・`metadata`・`canonicalize`〈closure、IO-09-D5〉・型の `File`・`ReadDir`・`DirEntry`・`Metadata`・`FileType`、`windows_sys` の `GetLogicalDrives`・`GetDriveTypeW`・`DRIVE_REMOVABLE`・`SetThreadErrorMode`・`SEM_FAILCRITICALERRORS`・型 `THREAD_ERROR_MODE`〈round 1〉）の外にあるとき、または `File` の `set_len`・`set_modified`・`set_times`・`set_permissions` の呼出し、`use std::fs::*`、`std` の項目を `as` で改名する形（`use std::fs as f`・`use std::{fs as f}`・`use std::fs::{self as f}`・`use std::fs::File as F`・`use std::io::{Write as W}`。round 1）、`crate::`・`super::` で始まる path（`use` の group の中の項目を含む）のうち `super::` の folder 名の定数（29 §29.7.4 手順 1）以外のもの（round 2。round 1 の「書く関数を持つ module〈`pos_source_copy`・`mnt::`〉への参照」を置き換えた）があるとき red にならない。mutant: module に `File::create(…)` を足す、`fs::copy(…)` を足す、`OpenOptions::new()` を足す、`use std::io::Write;` を足す、`file.set_modified(…)` を足す、`use std::{fs as f};` を足す、`use std::fs::{self as f};` を足す、`use super::pos_source_copy;` を足す、`use super::image_manager;` を足す、`use crate::db::sales_repo;` を足す、`use super::{REGISTER_SD_の定数, pos_source_copy};` のように group に混ぜる（定数の名は実装の名に読み替える。round 2）、のどれでも red。`super::` の folder 名の定数だけの参照は green |
| 29 §29.7.4 手順 1・71 MNT-01-D10（2026-10-23 追加） | folder 名の文字列を IO-09 と MNT に二重に持つ | unit（source の検査） | `register_sd::register_sd_folder_name_has_one_definition` | `src-tauri/src` の各 `.rs` の `#[cfg(test)]` より前の部分で、文字列 literal の中の `SR500_550_4000` の出現（comment は数えない）が定数の定義の 1 か所（`src-tauri/src/io/mod.rs` の `pub(crate) const`。29 §29.7.4 手順 1、round 1）でない（`mnt/backup.rs`・`mnt/offsite.rs` か `io/register_sd.rs` の本体に literal を足す mutant、定義を `io/register_sd.rs` に移す mutant で red）。`#[cfg(test)] mod <名>;` で読み込まれる file（例 `biz/daily_report_import_service/tests.rs`。親の file のその行から `<名>.rs` か `<名>/` の下を決める）は、中に `#[cfg(test)]` が無くても全体を test として数えない（round 2。Matrix の BIZ-08-D5・D6・手順 1a の行の一時の tree をその file で literal で作っても red にしない）。test の tree も定数から作る（literal で書いた test が残っても、上の除外で red にしない）。PC の外の控えの runtime が先に merge して同じ検査を置いていれば、その test を使い、本行は足さない（その test にこの除外が無ければ足す） |
| IO-09-D3 | root の外を読む | unit | `register_sd::read_rejects_escaping_paths` | `..\x`・`C:\x`・`\x`・空の要素の相対 path で `InvalidPath` にならない |
| IO-09-D5（closure） | root の下の link を辿って外を読む | unit（一時 tree、`#[cfg(unix)]`。link は test の中で `std::os::unix::fs::symlink` で作る） | `register_sd::list_and_read_reject_links_outside_root` | 合成の tree: 一時 directory の `outside/XZ/2026/10/Z001_06 .CSV`（合成の bytes、`max_bytes` より大きい）と、`root/CASIO/SR500_550_4000/XZ` を `outside/XZ` への directory の link にした root。(1) `list_register_sd_entries` が `Err(InvalidPath)` でない（外の entry を含む `Ok` は red）。(2) `read_register_sd_file(root, "XZ\2026\10\Z001_06 .CSV", max_bytes)` が `Err(InvalidPath)` でない（外の bytes を返す `Ok`、開いて読んだ印の `TooLarge`、`Io` のどれも red）。(3) `XZ` は実体の directory のまま、その下の `2026\10\Z001_06 .CSV` だけを外の file への link にした tree でも (1)・(2) が同じ。(4) 通す側: root の下の別の file への link（`XZ\2026\10\Z001_06 .CSV` → 同じ root の `XZ\2026\10\Z001_06A.CSV`）と、root より上に link を含む root（一時 directory の `alias` → `real` で、root を `alias/…/SR500_550_4000` の形で渡す）は、列挙が `Ok` で、読んだ bytes が実体の file と同じ。mutant: 包含の検査を path の文字列だけに戻す（(1)・(2)・(3) が red）、対象の path だけを解いて root を解かない（(4) の `alias` の root が `InvalidPath` で red）、検査を読んだ後に置く（(2) が `TooLarge` で red） |
| IO-09-D3（round 2 Codex で読んだ byte の数を足した） | 上限を超えて全部読む | unit | `register_sd::read_stops_at_limit` | `max_bytes` + 1 byte の file で `TooLarge` にならない。加えて、§29.7.6 手順 2 の読み方の非公開の関数（packet の S1）に、読んだ byte の数を数える reader（`std::io::Read` を実装し、`read` が返した byte を足す test の型）で `max_bytes` × 4 byte の入力を渡したとき、`TooLarge` にならない、または数えた byte の数が `max_bytes + 1` を超える（全部を読んでから長さを比べる mutant、上限を `max_bytes` × 2 にする mutant で red） |
| IO-09-D4 | 形を直して返す・測らない | unit | `register_sd::read_measures_shape` | BOM 付き・孤立 LF・孤立 CR・最終改行なし・CP932 でない byte 列のそれぞれで、該当の flag が立たない、または bytes が元と違う |
| BIZ-08-D3 | 窓の始まりを誤る | unit | `scan::window_from_latest_import_or_30_days` | completed の最大 report_date が 60 日前なら `from` がその日、昨日なら today − 30、取込みなしなら today − 30 にならない。rolled_back だけの日を最大に数える |
| BIZ-08-D3 | 同じ日の 2 つの精算を 1 つにする | integration（一時 tree + DB） | `scan::same_day_two_settlements_are_two_candidates` | 接尾字が空白と `A` の 2 組（精算回数 5・6）が 2 つの `NotImported` にならない |
| BIZ-08-D3 | `XZ` と `XZ_BKUP` の同じ bytes を 2 つ出す | integration | `scan::same_bytes_in_both_areas_collapse` | 同じ bytes の組が `XZ` と `XZ_BKUP` にあるとき候補が 2 つになる、またはまとめた候補の snapshot の 3 本の `sd_relative_path` が `XZ\` で始まらない（§37.9 手順 6。`XZ_BKUP` を先に列挙する tree でも `XZ` を残す。優先を消す mutant で red） |
| BIZ-08-D3 | `EcrDatas` から取込み済みの分を取り込めると出す | integration | `scan::imported_by_hash` | 同じ bytes の束を既存の手でのファイル選択の経路で commit した後、scan でその束が `Imported` にならない |
| BIZ-08-D3 | 取消の後に取り込めない | integration | `scan::rolled_back_is_not_imported` | rollback した束が `NotImported` に戻らない |
| BIZ-08-D3 | そろわない束を読む | integration | `scan::incomplete_group` | Z002 の無い組が `Incomplete` にならない、またはその組の file が読まれる |
| BIZ-08-D3 | 形の外れた束を取り込める | integration | `scan::shape_violation_is_unreadable` | Z005 だけ BOM 付き（または孤立 LF）の組が `Unreadable` にならない、snapshot に入る |
| BIZ-08-D3 | 途中の失敗で一部を返す | integration | `scan::io_failure_returns_error_without_candidates` | 2 組目の読取りで IO error を注入したとき `Ok` を返す |
| BIZ-08-D3（§37.9 手順 1・3） | root と読取りの失敗を別の文・推測の選択にする | integration（一時 tree、root を返す処理を test で差し替える） | `scan::root_and_read_failures_use_fixed_texts` | 次のどれかで、`BizError::ImportError` の文が右の固定の文と完全に一致しない、または `Ok` を返す: 自動で root が 0 件 →「SD が見つかりません。レジの SD をこの PC に差してから、もう一度読んでください。」、2 件 →「SD が 2 枚以上見つかりました。売上を読む SD だけを差してください。」、`Selected` が `NotRegisterSd` →「選んだ場所はレジの SD ではありません。SD の中の CASIO の folder か、SD そのものを選んでください。」、`XZ` の無い root →「SD に売上の folder（XZ）がありません。レジの SD か確かめてください。」、列挙の途中の `Io` →「SD を読めませんでした。SD を差し直して、もう一度読んでください。」、列挙の `InvalidPath`（closure。`XZ` を root の外への link にした一時 tree、`#[cfg(unix)]`）→「SD の中に、SD の外の場所を指す folder かファイル（リンク）があるため、読みませんでした。レジの SD そのものを差して読むか、リンクの無い SD の写しを選んでください。」。2 件のときに 1 つ目を選んで続ける mutant、文を io error の詳細から作る mutant、`InvalidPath` を `Io` の文にする mutant で red |
| BIZ-08-D3 | 並びが新しい順でない | unit | `scan::candidates_sorted_newest_first` | 10-05 と 10-06、同じ日の精算回数 6 と 5 の並びが 10-06(6)・10-06(5)・10-05 にならない |
| BIZ-08-D3 | 同じ精算の別の bytes の束を「取り込めます」と出す | integration（一時 tree + DB） | `scan::same_settlement_other_bytes_is_same_settlement_imported` | DB に report_date・精算回数 5 の completed（hash H1）があり、SD に同じ日・精算回数 5 で 1 byte 違う束があるとき、状態が `SameSettlementImported` にならない、または snapshot に入る（手順 7 の照合を消す mutant で red） |
| BIZ-08-D3 | 取り込めない候補を snapshot に入れる | integration | `scan::snapshot_holds_only_not_imported` | `Imported`・`SameSettlementImported`・`Incomplete`・`Unreadable` の候補を 1 つずつ含む tree で、`files_by_candidate` の key が `NotImported` の候補だけにならない（手順 9 の絞り込みを消して全候補を入れる mutant で red）。続けて `Imported` の candidate_key で `parse_and_validate_daily_report_from_sd` が `validation` にならない |
| BIZ-08-D6 | scan で精算回数の無い束を取り込めると出す | integration | `scan::bundle_without_settlement_no_is_unreadable` | Z001 だけ精算回数の行があり Z002 / Z005 に無い（形は layout A のまま）束が `Unreadable` にならない、または snapshot に入る（手順 7 の None の分岐を消す mutant で red） |
| BIZ-08-D6 | preview で SD の None の束を通す | integration | `daily_report_import_service::sd_bundle_without_settlement_no_is_rejected` | 同じ束を `sd_relative_path` つきで `parse_and_validate_daily_report` に渡したとき、§37.3 手順 4 の固定の文の `ImportError` にならない（手順 4 の検査を消す mutant で red） |
| BIZ-08-D6 | 手で選んだ None の束まで止める | integration | `daily_report_import_service::manual_bundle_without_settlement_no_still_previews` | 同じ bytes を `sd_relative_path` なし・`source_path` が test の SD の root の外（`parse_and_validate_daily_report_with_sd_lookup` に一時 directory の root を返す closure を渡し、`source_path` は在る PC 側の一時 file）で渡したとき preview が成功しない（PC 上の file を手で選ぶ経路の扱いを壊す） |
| BIZ-08-D6 | 手で選んだ SD 上の None の束を通す | integration | `daily_report_import_service::manual_sd_bundle_without_settlement_no_is_rejected` | 同じ bytes を `sd_relative_path` なし・`source_path` が test の SD の root の下で渡したとき、§37.3 手順 4 の固定の文の `ImportError` にならない（§37.3 手順 1a を消す mutant で red） |
| §37.3 手順 1a | SD 上かを確かめられないのに PC の file として通す | integration | `daily_report_import_service::sd_root_lookup_failure_stops_preview` | `parse_and_validate_daily_report_with_sd_lookup` に `Err(RegisterSdError::Io { .. })` を返す closure を渡したとき、§37.3 手順 1a (i) の固定の文の `ImportError` にならない（失敗を PC 扱いにする mutant で red） |
| §37.3 手順 1a (iii) | SD を抜いた後の SD の file を PC の file として通す | integration（一時 directory） | `daily_report_import_service::manual_sd_file_removed_before_preview_is_rejected` | 順: 一時 directory に `CASIO\SR500_550_4000\XZ\2026\10\` の 3 本を作って bytes を読む → その directory を消す（SD を抜いた代わり）→ root の列が空の `Ok(vec![])` を返す closure と、消した path の `source_path` で preview。このとき「選んだファイルが見つからなくなりました。SD から選んだときは、SD をこの PC に差したまま、もう一度選んでください。」の `ImportError` にならない、または preview が成功する（`locate` が `None` の file を無条件に PC 扱いにする mutant で red） |
| IO-09（§29.7.4 手順 5） | 無い file・directory を在ると答える | unit | `register_sd::check_selected_file_present_rejects_missing_and_dirs` | 在る一時 file で `Ok` にならない、消した file・存在しない drive の path・directory の path で `Err(Io { .. })` にならない |
| BIZ-08-D6 | commit で SD の None の cache を通す | integration | `daily_report_import_service::commit_rejects_sd_cache_without_settlement_no` | `sd_source_files` が空でなく `settlement_no` が None の cache を直接作って commit したとき、`ImportError` にならない、`daily_report_imports` の行が増える、または `pos-sources/` に写しができる（§37.4 手順 2a を消す mutant で red。preview の検査が残っていても落ちる） |
| BIZ-08-D4 | 別の bytes の同じ精算を追加確認で通す | integration | `daily_report_import_service::same_settlement_different_bytes_is_rejected` | 同じ report_date・精算回数で 1 byte 違う束（Z002 の末尾の行の金額だけ違う）の preview が `ImportError`（固定の文）にならず `AdditionalImportConfirmationRequired` になる |
| BIZ-08-D4 | 同じ日の正常な 2 回目を止める | integration | `daily_report_import_service::same_date_other_settlement_still_confirms` | 精算回数 6 の束（既存は 5）が `AdditionalImportConfirmationRequired` 以外になる |
| BIZ-08-D4 | NULL・None を照合する | integration | `daily_report_import_service::null_settlement_no_is_not_compared` | `settlement_no` NULL の既存行、または layout B の束（None）で `ImportError` になる |
| BIZ-08-D4 | TX の間に入った同じ精算を通す | integration | `daily_report_import_service::commit_rechecks_same_settlement_in_tx` | 先に同じ日・精算回数 5・hash H1 の completed の行 A を repository で入れ、精算回数 5・hash H2 の cache を直接作る（`duplicate_check.status` = `AdditionalImportConfirmationRequired`、`active_same_date_import_ids` = [A の id] で TX の中の手順 5 の snapshot の再検査は通る形）。`additional_import_confirmed = true` で commit したとき、§37.3 手順 8 の BIZ-08-D4 の固定の文の `ImportError` にならない、または `daily_report_imports` の行が増える（§37.4 手順 4a を消す mutant で red。手順 5 は同じ snapshot なので先に拒まない） |
| BIZ-08-D4 | 保存しない | integration | `daily_report_import_service::commit_stores_settlement_no` | commit した行の `settlement_no` が束の値でない |
| BIZ-08-D4（24 §14.18a） | 照合の候補を誤って取る | integration（DB） | `sales_repo::find_same_settlement_daily_report_import_matches_only_completed_other_hash` | 同じ日・精算回数 5 の行が (H1, completed)・(H2, rolled_back)・(H3, completed、精算回数 NULL) のとき、hash H1 で呼んで `None` にならない、hash H9 で呼んで H1 の行の id にならない（rolled_back・NULL・同じ hash を除く条件のどれかを消す mutant で red） |
| BIZ-08-D4（24 §14.14） | INSERT が精算回数を落とす | integration（DB） | `sales_repo::insert_daily_report_import_stores_settlement_no` | `settlement_no: Some(5)` で入れた行の列が 5 でない、`None` で入れた行が NULL でない |
| BIZ-08-D4・22 §17 | migration | integration | `migration::adds_daily_report_settlement_no_nullable` | 日報の行がある直前の版の DB を移行した後、既存行の `settlement_no` が NULL でない、`pragma_table_info('daily_report_imports')` に `settlement_no`（型 `INTEGER`、`notnull` 0）が無い、2 回目の `migrate` で版の行が増える（重複して適用する）、`settlement_no = 5` を入れた行が 5 で読めない |
| IO-10 | 同じ path の違う bytes を上書きする | unit | `pos_source_copy::different_bytes_get_hash_suffixed_name` | 同じ相対 path に違う bytes を 2 回書くと、1 回目の file の内容が変わる、または 2 回目が `~` + hash 12 桁の名前にならない |
| IO-10 | 同じ bytes を書き直す | unit | `pos_source_copy::same_bytes_not_rewritten` | 同じ bytes の 2 回目が `written: true` になる、更新時刻が変わる |
| IO-10 | 半端な file を最終の名前に残す | unit | `pos_source_copy::writes_via_temp_and_rename` | 書込みの途中の失敗を注入したとき、最終の名前の file ができる |
| IO-10 | 初回に保存先の directory が無いと書けない | unit | `pos_source_copy::creates_missing_directory_before_temp_file` | 空の一時 directory を app_data_dir にして `XZ\2026\10\Z001_06 .CSV` を書いたとき、`Ok` にならない、または `pos-sources/casio-sr-s4000/sd/XZ/2026/10/` の下に最終の名前の file ができない（`create_dir_all` を一時 file の後へ戻す mutant で red） |
| IO-10 | app data の外に書く | unit | `pos_source_copy::rejects_escaping_paths` | `..\x`・`C:\x` の相対 path で書く |
| BIZ-08-D5 | 写しの失敗でも取り込む | integration | `daily_report_import_service::copy_failure_aborts_commit` | 書けない app_data_dir（読取り専用の一時 directory）で commit が成功する、`daily_report_imports` の行が増える |
| BIZ-08-D5 | 写しの失敗の後に同じ token で再試行できない | integration（CMD + 一時 directory） | `daily_report_import_cmd::copy_failure_keeps_preview_token_for_retry` | 読取り専用の app_data_dir で commit が kind `internal`・message「SD から読んだファイルの写しを PC に保存できなかったため、取り込みませんでした。PC の空き容量を確かめて、もう一度取り込んでください。」（45 §45.7 の `SourceCopyFailed` の行）にならない（`import_error` になる mutant で red）、またはその後、`daily_report_preview_cache` に同じ preview_token が残らない、または書ける directory に替えて同じ token で commit が成功しない（行が 1 つ増え、写しが 3 本できる）。失敗時に token を消す mutant で red |
| §37.3 手順 9（2026-10-17 追加） | `sd_source_files` に PC の file を入れる・SD の file を落とす | integration | `daily_report_import_service::cached_preview_holds_only_sd_files` | (a) SD の候補の 3 本（`sd_relative_path` つき）の preview で `sd_source_files` が 3 件で、各 `source` が Z001 / Z002 / Z005・`sd_relative_path` と `bytes` が入力と同じでない、(b) `source_path` が test の SD の root の外の 3 本で空でない、(c) 3 本のうち 1 本だけ root の下（手順 1a (ii)）で 1 件でない（その 1 本の `sd_relative_path` が root からの相対 path でない） |
| BIZ-08-D5（2026-10-17 追加） | TX の中で拒んだ束の写しを「取り込んだ」と記録する | integration | `daily_report_import_service::rejected_in_tx_leaves_copy_but_no_copy_path` | 先に同じ bundle_hash の completed を repository で入れてから、その束の cache（`NoDuplicate` の形）で commit したとき、`IdempotencyConflict` にならない、または `daily_report_imports` の行が増える。`pos-sources/` には写しが残ってよい（assert しない）が、DB のどの行の `source_files_json` にもこの commit の `copy_path` が増えない |
| BIZ-08-D5 | 写しの path を記録しない | integration | `daily_report_import_service::commit_records_copy_paths` | SD の束の commit の `source_files_json` に `sd_relative_path`・`copy_path` が無い、`copy_path` の file の bytes が束と違う |
| BIZ-08-D5（55 の手順 2） | 写しの失敗の後に UI が preview と token を失う | flow（component、`DailyReportImportPage.flow.test.tsx`） | 写しの失敗からの再試行 | `commitDailyReportImport` を 1 回目は kind `internal`・上の固定の文で失敗、2 回目は成功にした mock で、preview →「取り込む」→ error に固定の文が出る →「戻る」→ 同じ preview（同じ file 名・対象日）に戻る →「取り込む」の順に操作したとき、error の後に `idle`（最初の画面）へ戻る、または 2 回の commit の `previewToken` が同じでない、または 2 回目の後に結果が出ない（`decideRecoverTo` が `internal` で `idle` を返す mutant、`import_failed` で `previousState` を preview にしない mutant で red） |
| BIZ-08-D5 | PC 上の file を手で選んだ束も写す | integration | `daily_report_import_service::pc_bundle_has_no_copy` | `source_path` が test の SD の root の外（または `None`）の束の commit で `pos-sources/` に file ができる |
| BIZ-08-D5 | 手で選んだ SD 上の file を写さない | integration | `daily_report_import_service::manual_sd_files_are_copied` | `source_path` が test の SD の root の下の 3 本の commit で、`pos-sources/casio-sr-s4000/sd/` の下にその相対 path の写しが 3 本できない、または `source_files_json` に `sd_relative_path`・`copy_path` が無い（§37.3 手順 1a を消す mutant で red） |
| BIZ-08-D5 | 保存先の無い初回で取り込めない | integration | `daily_report_import_service::first_commit_creates_copy_directory` | `pos-sources/` の無い app_data_dir（空の一時 directory）で SD の束を commit したとき、写し 3 本と `daily_report_imports` の 1 行ができない |
| BIZ-08-D5 | 古い JSON を読めない | integration | `daily_report_import_service::source_filenames_accepts_missing_copy_fields` | `copy_path` の無い既存の `source_files_json` で同日の summary が失敗する |
| CMD-12-D1 | 期限切れの scan を使う | unit | `daily_report_import_cmd::from_sd_rejects_expired_scan` | 31 分前の snapshot で preview を返す |
| CMD-12-D1 | 取り込めない候補を受ける | unit | `daily_report_import_cmd::from_sd_rejects_unknown_candidate` | snapshot に無い candidate_key で preview を返す |
| CMD-12-D1 | 新しい scan で古い snapshot が残る | unit | `daily_report_import_cmd::new_scan_replaces_cache` | 2 回目の scan の後に 1 回目の scan_token で preview を返す |
| CMD-12-D1 | CMD が IO を呼ぶ | 既存の構造 test | `src-tauri/tests/architecture_test.rs`（`rg -n "fn " src-tauri/tests/architecture_test.rs` で存在を確かめた。`LAYER_RULES` の `cmd` の行が cmd → db・io を禁止） | `cmd` から `io::register_sd`（`RegisterSd*` の型を含む）を use する。選択は BIZ-08 の `DailyReportSdSelection` で渡す |
| UI-07-D15 | 状態を色だけで示す・文言違い | component | `DailyReportImportPage.sd.test.tsx` の状態ごとの label | 5 状態の label（「取り込めます」「取込み済み」「同じ精算を取込み済み」「ファイルがそろっていません」「読めません」）と icon が出ない |
| UI-07-D15 | 取り込めない精算を「無い」と出す | component | `DailyReportImportPage.sd.test.tsx` の一覧の上の文 | (a) `NotImported` 1 件と `Unreadable` 1 件で文が出る、(b) `Incomplete` 1 件・`Unreadable` 1 件だけで「取り込める精算はありません。取り込めない精算が 2 件あります（「ファイルがそろっていません」「読めません」の行）。」が出ない、または「新しい精算はありません」が出る、(c) 候補 0 件と、`Imported`・`SameSettlementImported` だけのときに「新しい精算はありません」が出ない |
| UI-07-D15 | SD を戻す案内が出ない・CV17 の取込みの前に戻させる | component | 同上 | scan 成功の後に「読み終わりました。いつもの取込みが済んでいれば、SD はレジに戻してください（次の精算に要ります）。」が出ない（「いつもの取込みが済んでいれば、」の無い文も red） |
| UI-07-D15 | 見つからないときの予備が出ない | component | 同上 | 「SD が見つかりません」の error のとき「場所を選ぶ」が出ない |
| UI-07-D15 (3) | 読まなかった file の数を出さない・誤る | component | `DailyReportImportPage.sd.test.tsx` の補足 | `skipped_unknown_count` = 3 の scan で「読まなかったファイル 3 件」が出ない、0 の scan でその補足が出る（0 の分岐を消す mutant で red） |
| UI-07-D15 (4) | 「ファイルを選び直す」で最初へ戻る | component（flow） | `DailyReportImportPage.sd.test.tsx` の選び直し | `sd_list` →「確認する」→ preview →「ファイルを選び直す」で `sd_list`（同じ一覧と「SD から読む」）に戻らない、または `idle` に戻る（SD から来たかの分岐を消す mutant で red）。UI は期限を確かめないので、期限の判定を UI に足す mutant も、30 分を過ぎた時刻の mock で一覧に戻らず最初へ行くので red |
| UI-07-D15 (5) | 期限切れの scan で最初へ戻る・message で分岐する | component（flow） | `DailyReportImportPage.sd.test.tsx` の期限切れ | `parseAndValidateDailyReportFromSd` を kind `import_error`・固定の文「SD を読んでから時間がたちました。もう一度 SD を読んでください。」で失敗させたとき、error に固定の文が出ない、「戻る」で `sd_list` に戻らない、または一覧の上の「SD から読む」で `scanRegisterSd` が呼ばれない。kind `validation`（取り込めない候補）でも同じ `sd_list` に戻らない（kind・message で分岐する mutant で red） |
| UI-07-D15 | reducer の遷移 | unit | `reducer.test.ts`（既存 file、`src/features/daily-report-import/reducer.test.ts`） | `scanning` → `sd_list` → `parsing` → `preview` と、SD から来た parse 失敗の recoverTo `sd_list` が成り立たない |
| UI-07-D15 (5)（round 1） | SD から来た parse の失敗で最初へ戻る・手で選んだ失敗まで一覧へ戻す | unit | `reducer.test.ts` の `parse_failed` の分岐 | `sdOrigin` を持つ `parsing` で `parse_failed` → recoverTo が `sd_list` でない、previousState が同じ scan と scanToken の `sd_list` でない、`dismiss_error` で `sd_list` に戻らない（`dismiss_error` の `sd_list` の分岐を消す mutant で `idle`、red）。`sdOrigin` の無い `parsing` で recoverTo が `idle` でない |
| UI-07-D15 (4)・BIZ-08-D5（round 1） | 写しの失敗の後に SD から来たことを落とす | unit | `reducer.test.ts` の `import_failed` の作り直し | `sdOrigin` を持つ `preview` → `confirm_import` → `import_failed`（recoverTo `preview`）→ `dismiss_error` の後の `preview` が同じ `sdOrigin` を持たない、その後の一覧へ戻る action で同じ scan の `sd_list` にならない（`import_failed` の作り直しで `sdOrigin` を落とす mutant で red）。`sdOrigin` の無い preview で一覧へ戻る action が状態を変える |
| UI-07-D15 (4)・BIZ-08-D5（round 1） | 同上（画面） | component（flow） | `DailyReportImportPage.sd.test.tsx` の写しの失敗からの選び直し | `sd_list` →「確認する」→ preview →「取り込む」（`commitDailyReportImport` を kind `internal`・45 §45.7 の `SourceCopyFailed` の固定の文で失敗）→「戻る」→ 同じ preview →「ファイルを選び直す」の順で、同じ一覧（同じ行と「SD から読む」）に戻らない、file の dialog（`open`）が呼ばれる、または最初の画面に戻る |
| UI-07-D15 (6)（round 2 Codex） | 取込みの成功の後に SD から来たことを落とす・取り込んだ行を「取り込めます」のまま出す | unit | `reducer.test.ts` の `import_succeeded` の後の一覧へ戻る | `sdOrigin`（候補 A・B が `NotImported` の scan、`candidateKey` = A）を持つ `importing` → `import_succeeded` の `result` が同じ `sdOrigin` を持たない、その `result` で一覧へ戻る action の後が `sd_list` でない、scanToken が同じでない、A の `status` が `Imported` でない、B の `status` が `NotImported` でない（`import_succeeded` で `sdOrigin` を落とす mutant、`status` を替えない mutant、全行を替える mutant で red）。`sdOrigin` の無い `result` で一覧へ戻る action が状態を変える。`sdOrigin` のある `result` の `rollback_succeeded` が `idle` でない |
| UI-07-D15 (6)（round 2 Codex） | 同じ scan の次の候補へ進めない | component（flow） | `DailyReportImportPage.sd.test.tsx` の続けての取込み | 候補 A・B が「取り込めます」の scan で、A の「確認する」→ preview →「取り込む」（成功）の後、結果に「SD の一覧へ戻る」が出ない、押して同じ一覧に戻らない、A の行が「取込み済み」で「確認する」なしにならない、B の行が「取り込めます」でない、B の「確認する」で `parseAndValidateDailyReportFromSd` が 1 回目と同じ scanToken と B の candidate_key で呼ばれない、または `scanRegisterSd` が 2 回目に呼ばれる（読み直す mutant で red）。手で選んだ経路の結果に「SD の一覧へ戻る」が出る |
| UI-07-D16 | 1 つずつ選び足し・外す | component | `DailyReportImportPage.files.test.tsx` | Z001 → Z005 → Z002 の順に 1 つずつ足して「確認する」が有効にならない、1 つ外すと無効に戻らない |
| UI-07-D16 (c)（2026-10-17 追加） | 系列の決まらない file・同じ系列の重複を足す | component | `DailyReportImportPage.files.test.tsx` の拒否 | `notes.CSV` を選ぶと一覧に足される、または「Z001・Z002・Z005 のどれかの名前のファイルを選んでください。」が出ない。1 回の dialog で `Z001_a.CSV` と `z001_b.csv` を選ぶと、どちらかが足される、または「Z001・Z002・Z005 は 1 つずつ選んでください（同じ種類のファイルが 2 つ以上あります）。」が出ない |
| UI-07-D16 (d)（2026-10-17 追加） | 確かめずに置き換える・キャンセルで変える | component | `DailyReportImportPage.files.test.tsx` の置き換え | Z001 を選んだ後に別の Z001 を選ぶと、title「Z001 のファイルを置き換えますか？」・description「選んであった {前の file 名} を外して、{新しい file 名} を使います。」の dialog が出ない、「キャンセル」で一覧の Z001 の file 名が変わる、「置き換える」で新しい file 名にならない |
| UI-07-D16 (a)（2026-10-17 追加、round 2 で行き先を足した） | 一覧が preview の後まで残る・手で選んだ preview の選び直しの行き先を誤る | component（flow） | `DailyReportImportPage.files.test.tsx` の一覧の寿命 | 3 つ選んで「確認する」→ preview →「ファイルを選び直す」で file の dialog（`open`）が呼ばれない。dialog で Z001 を 1 つ選ぶと、`idle` の一覧が選んだ Z001 の 1 行だけにならない（前の 3 つが残る、または preview から直接 parse が呼ばれる）。dialog をキャンセルすると preview（同じ file 名・対象日）のままでない。Z001 を選んだ後、続けて Z002・Z005 を選び足して「確認する」を押したとき、`parseAndValidateDailyReport` が選び直した 3 つの `source_path` で呼ばれない、または新しい preview が出ない（round 2 Codex） |
| UI-07-D16（2026-10-17 追加） | path を持たない `File` の入力が残る | unit（hook） | `useDailyReportImportFlow.test.tsx` の公開の API | hook の戻り値に `selectFiles` がある（55 UI-07-D16 で外した。既存の `selectFiles` を使う test は `chooseFiles` と dialog の mock に書き換え、assert の内容は弱めない） |
| UI-07-D16 | dialog の path を捨てる | unit（hook） | `useDailyReportImportFlow.test.tsx` の payload | dialog が返した 3 つの path が、`parseAndValidateDailyReport` の各要素の `source_path` に入らない（今の `{ filename, file_bytes }` だけの payload のまま） |
| IO-09（§29.7.4 手順 4） | SD の root の下かを誤る | unit（OS に依らない文字列の判定。`cfg(windows)` にしない、round 1） | `register_sd::locate_in_roots_maps_paths` | 同じ検査を 2 つの形で行う。Windows の形: root `E:\CASIO\SR500_550_4000` に対し、`e:\casio\sr500_550_4000\XZ\2026\10\Z001_06 .CSV` と `E:/CASIO/SR500_550_4000/XZ/2026/10/Z001_06 .CSV` が `XZ\2026\10\Z001_06 .CSV` にならない。一時 directory の形: root `/tmp/t/CASIO/SR500_550_4000` に対し、`/tmp/t/casio/SR500_550_4000/XZ/2026/10/Z001_06 .CSV` が同じ `relative_path` にならない。どちらの形でも、`…\SR500_550_4000X\a.CSV`（名前の前方一致だけ）・root の外（`C:\EcrDatas\a.CSV`・`/tmp/other/a.CSV`）・相対 path（`XZ\a.CSV`）・`..` を含む path・空の root の列のどれかが `None` にならない（`Path::components` で分ける mutant は Linux で Windows の形が red、`\` だけで分ける mutant は一時 directory の形が red） |

## State Lifecycle Matrix

| State / subject | Initial | Pending | Success | Invalidate | Refetch | Revisit | Restart | Failure | Retry | Evidence |
|---|---|---|---|---|---|---|---|---|---|---|
| SD の scan（snapshot） | idle | scanning（ボタン disabled、離脱は block しない） | sd_list と scan_token、SD を戻す案内 | 新しい scan で置き換え、30 分で失効 | 「SD から読む」をもう一度 | 画面を離れて戻ると idle（snapshot は 30 分残るが UI は持たない） | アプリの再起動で失効 | 見つからない・2 つ以上・読めない → 固定の文と予備の操作、候補は出さない | もう一度読む | CMD-12-D1 の 3 行、UI の状態 test |
| 候補の状態 | — | — | NotImported / Imported / SameSettlementImported / Incomplete / Unreadable（精算回数の無い束を含む、BIZ-08-D6） | commit で NotImported → Imported（結果の「SD の一覧へ戻る」で、取り込んだ行だけ UI の表示を替える〈CMD の snapshot は替えない、55 UI-07-D15 (6)〉。再 scan でも反映） | 再 scan | — | — | — | rollback で Imported → NotImported | BIZ-08-D3 の行 |
| 日報の取込み | preview | importing（既存の block） | result（SD から来たときは「SD の一覧へ戻る」で同じ scan の一覧へ） | rollback（今どおり idle） | 既存 | 既存 | 既存 | BIZ-08-D4 の拒否 | 前の取込みを取り消してから読み直す | BIZ-08-D4 の行 |

## Adjacent Pattern Audit

| Source pattern / contract | Repository sites inspected | Ported sites | Explicit exclusions and reason | Test / evidence |
|---|---|---|---|---|
| preview cache（AppState、30 分、UUID token） | `45-cmd-daily-report-import.md` §45.2・§45.3・§45.4 | scan cache（§45.6a・§45.6b）に同じ 30 分・UUID | scan は commit しないので「失敗時に残す」の規則は不要。新しい scan で置き換える | CMD-12-D1 の行 |
| 重複判定（bundle_hash の AlreadyImported、同日追加の確認） | 37 §37.3 手順 8、§37.4 手順 4〜5 | SD の候補の状態（§37.9 手順 7）が同じ hash の照合を使う | — | BIZ-08-D3 の imported_by_hash |
| 利用者向けの固定の文（BIZ-08-D1・D2） | 37 §37.3 手順 3 | BIZ-08-D4 と §37.9 の文も raw detail を含めない固定の文 | — | BIZ-08-D4 の行 |
| UI の入力エラーの 1 スロット表示（destructive テキスト + icon、ボタン直下） | 55 §55.0 手順 2 | 「SD から読む」の直下の error | 上部 Alert 帯はデータ安全系だけのまま | UI-07-D15 の行 |
| 確認の dialog（`src/components/ui/alert-dialog.tsx`、UI-07-D13 の `AdditionalImportConfirmDialog` と同じ部品） | `rg -l 'AlertDialog' src/features src/components`（2026-10-23、11 file） | UI-07-D16 (d) の置き換えの確認 | 同日追加の確認の文言・状態は変えない | UI-07-D16 (d) の行 |
| 空の読取り機の口で error の dialog を出さない（71 §71.11.3 の `mnt::offsite` の媒体を探す関数、D-114。2026-10-23 追加） | 71 §71.11.3 の「error の dialog を出さない」 | 29 §29.7.4 手順 1（IO-09 の file system を見る 5 関数が最初に作る 1 つの guard。round 2 で `find_register_sd_roots` の中だけから広げた） | 書く・読み戻すの関数は IO-09 に無い（IO-09-D3）ので、列挙・読取り・在否の確認の間だけ。純関数の `locate_in_register_sd_roots` は作らない | 許可の列の行（source の検査）、Final Review の source の読み（5 関数の最初の guard）と L3-1 |
| 前回選択フォルダの記憶 | 55 §55.1 `useDailyReportImportFlow.ts` | 予備の経路だけに残す | SD の経路は場所を記憶しない（自動で探す） | UI-07-D16 |

## Negative Paths

- missing input: SD が無い（`find_register_sd_roots` が空の列、`scan::root_and_read_failures_use_fixed_texts`）、`XZ` が無い（同）、Z002 の無い組
- invalid input: SD の root の下かを確かめられない手で選んだ file（§37.3 手順 1a）、規則外の名前、暦日として不正な日付、BOM・孤立 LF / CR・CP932 でない bytes、parse_errors のある束、SD の経路で精算回数の無い束（BIZ-08-D6）
- duplicate/ambiguous input: SD が 2 枚（`scan::root_and_read_failures_use_fixed_texts`）、同じ bytes が `XZ` と `XZ_BKUP`、同じ精算の別の bytes
- unknown reference: snapshot に無い candidate_key、期限切れの scan_token
- dependency missing: Windows 以外では自動の発見が空（予備の経路だけ）
- permission/write failure: 読取りの途中の IO error（全体の失敗）。書込みは無い
- dry-run side effect: scan は DB にも SD にも書かない（scan_leaves_tree_unchanged、scan の後の DB の行数が同じ）

## Boundary Checks

- threshold: 窓の 30 日、scan cache の 30 分、1 file の 20MB
- null/default: `settlement_no` None・NULL、`selected_path` None
- empty/non-empty: 候補 0 件と全件取込み済み（「新しい精算はありません」）、取り込めない候補だけ（「取り込める精算はありません。取り込めない精算が N 件あります…」）
- min/max: 接尾字 `A`〜`Z` と空白、連番 4 桁
- status/policy enum: 候補の状態 5 値
- wire type: `DailyReportSdCandidateStatus` は specta の string union
- internal type: `settlement_no: Option<i64>`
- producer/consumer: BIZ-08 → CMD-12 → UI-07
- round-trip token: scan_token、candidate_key（bundle_hash）
- precision/range: 精算回数の先頭 0
- cross-language parse: path の区切り `\`（Windows）と test の一時 directory。`locate_in_register_sd_roots` は `\` と `/` の両方で区切る文字列の判定で、両方の形を同じ test が確かめる（29 §29.7.4 手順 4、round 1）

## Compatibility Checks

- old schema/input: `settlement_no` NULL の既存行、手で選んだ 3 ファイル（既存の command）。版の期待を持つ既存の test 14 本は、最新の版の期待を `app_max_version()`、vB が対象の 2 本を `migrate_up_to(&conn, VB_VERSION)` にし、期待の意味は変えない（packet の S2、round 2）。`DailyReportImportPage.flow.test.tsx` の 2 本の選択の test は D16 の 1 つずつの選択に書き換え、件数の error の assert を「確認する」が無効で parse が呼ばれない assert に移す（packet の S5、round 2）。既存の BIZ の test のうち同じ日に別の bytes の束を取り込む 8 本は、束ごとに精算回数を分け、assert は変えない（packet の S3、round 1。同じ精算回数で別の bytes の束を拒むのは D4 の行が確かめる）
- new schema/input: SD の候補からの preview
- output order: 候補の並び（新しい順）
- optional field behavior: `report_date` は読めた束だけ

## Data Safety Checks

- source-derived data: test の SD の tree は合成（実の名前の形だけを使い、中身は合成）
- generated outputs: bindings は generator で再生成
- secrets: 無し
- local-only files: 実 SD の採取は repo 外
- synthetic sample boundaries: 合成の Z は layout A の形で、値は合成。L3-1・L3-3 の USB の tree も同じ helper から `scan::write_l3_register_sd_tree`（`#[ignore]`、出力先 `INVENTORY_L3_SD_TREE_OUT`）で書く（packet の S3、round 1）。組の日付は生成した日の前日・前々日（固定の日付にしない。scan の窓の外にしないため、packet の S3、round 2）

## Main Wiring / Integration Checks

- helper connected to main path: `scan_register_sd` → BIZ-08 §37.9 → IO-09、`parse_and_validate_daily_report_from_sd` → §37.3
- output reaches manifest/report: commit した日報が日次売上に出る（既存）
- effective config reaches runtime: specta の登録（`lib.rs`）
- CLI arg reaches implementation: 該当なし

## Mutation-style Adequacy Questions

- 窓の `min` を `max` にすると `window_from_latest_import_or_30_days` が落ちる。
- BIZ-08-D4 の照合から `settlement_no` を外す（report_date だけ）と `same_date_other_settlement_still_confirms` が落ちる。
- `XZ` / `XZ_BKUP` の連番の有無の規則を外すと `list_returns_unknown_without_reading` が落ちる。
- 形の検査を外すと `shape_violation_is_unreadable` が落ちる。
- IO-09-D3 の source の検査は、許可の列の外の API（`File::create`・`fs::copy`・`OpenOptions`・`std::io::Write`・`set_modified`）を 1 つ足すと `module_uses_only_allowed_fs_api` が落ちる。`super::` の folder 名の定数の外の `crate::`・`super::` の参照（`use super::image_manager;`・`use crate::db::…`）を 1 つ足しても落ちる（round 2）。
- §37.3 手順 1a (iii) の在否の確認を外し、`locate` が `None` の file を PC 扱いにすると `manual_sd_file_removed_before_preview_is_rejected` が落ちる。
- 写しの失敗を `import_error` に変えると `copy_failure_keeps_preview_token_for_retry` と flow の「写しの失敗からの再試行」が落ちる。
- reducer の `import_failed` の preview の作り直しで `sdOrigin` を落とすと、`reducer.test.ts` の `import_failed` の作り直しの行と `DailyReportImportPage.sd.test.tsx` の写しの失敗からの選び直しが落ちる。`dismiss_error` の recoverTo `sd_list` の分岐を消すと `parse_failed` の分岐の行が落ちる（round 1）。
- BIZ-08-D6 の 3 つの検査（scan の手順 7、preview の §37.3 手順 4、commit の §37.4 手順 2a）は、どれか 1 つを消すとその段の行（`bundle_without_settlement_no_is_unreadable`・`sd_bundle_without_settlement_no_is_rejected`・`commit_rejects_sd_cache_without_settlement_no`）が落ちる。検査の条件を「SD の経路」でなく全部の束にすると `manual_bundle_without_settlement_no_still_previews` が落ちる。
- 大文字小文字の無視を外すと `list_classifies_observed_names` の `ej261004.txt` が落ちる。
- §29.7.6 手順 2 の上限の読み方を「全部読んでから長さを比べる」に替えると、`read_stops_at_limit` の数えた byte の数が `max_bytes + 1` を超えて落ちる（round 2 Codex）。
- reducer の `import_succeeded` で `sdOrigin` を落とすと、`reducer.test.ts` の一覧へ戻る行と `DailyReportImportPage.sd.test.tsx` の続けての取込みが落ちる（round 2 Codex）。
- root の下かの検査を path の文字列だけに戻すと、`list_and_read_reject_links_outside_root` の外への link の 3 つが落ちる。root を解かずに対象だけを解くと、同じ test の `alias` の root が落ちる（closure）。

## Residual Test Gaps

- 実 SD の自動の発見（`DRIVE_REMOVABLE`）と、空の読取り機の口で error の dialog が出ないこと（29 §29.7.4 手順 1）は Windows の実機だけ（packet の L3-1）。test は `resolve_register_sd_root` と一時 directory で root を作る。自動 test は許可の列の source の検査が `SetThreadErrorMode` を許すことまでで、元の値へ戻すこと（guard の drop、途中で返る経路を含む）と、file system を見る 5 関数（`find_register_sd_roots`・`resolve_register_sd_root`・`check_selected_file_present`・`list_register_sd_entries`・`read_register_sd_file`）がどれも最初にその guard を作ることは Final Review の source の読みで確かめる（round 2）。`cfg(windows)` の本体（FFI の型・feature を含む）は hosted CI（ubuntu）でも WSL の `cargo check --release` でも compile されず、Final Review の FFI の読みと、L3 の build の前の Coordinator の照合で抑える（packet の Test Plan と Residual Risk、round 1）。
- 29 IO-09-D5 の Windows の側（junction と Windows の symbolic link を `canonicalize` が辿った先の実体にすること）は自動 test に無い（CI は ubuntu、test の link は Unix の symbolic link）。Rust の資料（`canonicalize` は Windows で `CreateFile` と `GetFinalPathNameByHandle`）と、71 §71.4 の「書く先の固定」の同じ前提（junction・symbolic link は辿った先の実体になる）に依り、L3 にも足さない（owner の手を増やさない。packet の Residual Risk）。Windows の `canonicalize` が SD（L3-1）と PC の folder の中の写し（L3-2）で成功することは、L3-1・L3-2 の通常の操作で観測される（失敗すれば `Io` で一覧が出ない）。列挙・読取りが確かめた解いた path をそのまま開くこと（元の path で開き直さない）は、静的な tree の test では見分けられないので Final Review の source の読みで確かめる（closure）。
- 取込み前の `XZ` の原本の形・精算回数が読めるか、取込みの後の file と同じ bytes か（SD-23）は自動 test にできない。packet の Contract Probe P1: (a)（形と精算回数）は 2026-10-23 に 2026-10-18 版での SD の写しで合格、(b)（bytes の比較）は未確認で、L3 の前に確かめる（packet の AC10。不一致なら L3 を頼まず design へ戻す。round 2 Codex で「merge の前提にしない」を置き換えた）。
- 規則に合わない名前の file が送信の失敗で増えるか（P4）は、packet の Contract Probe で 2026-10-23 に合格（増えたのは `XZ`・`XZ_BKUP` の外の file だけで、IO-09 は列挙しない）。設計は分岐しない（`Unknown` として数えるだけ）。
- 店で読んだ後に SD をレジへ戻して精算できること（IO-09-D3 の実機の確認）は、本番の DB が無い今は店の運用に載らず、導入の時の確認へ送る（packet の L3 の表）。
- 窓より前の未取込みは検知しない（D-111 の Non-scope）。
- 37 §37.9 手順 10・45 §45.6a 手順 2a（D-117）の test は PLU の書出しの見直しの runtime の lane。
