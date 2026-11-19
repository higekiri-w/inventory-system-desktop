# Plan Packet: 先の日付の backup で PC の外の控えが黙って止まる件を直す

第 1 波の lane（owner の決定 2026-11-19、repo 外の回答台帳 TD-807）。導入の前の関門 3（[Plans](../Plans.md#導入の前の関門)）の残りの 1 つで、出所は [backlog](../backlog.md) の「backup に、PC の外のコピーと復元の実証が運用として設計されていない」の項目の「(b)（PR #194）の申し送り」の行（AI の調べ、2026-11-13。owner の決定 TD-672 の (ii): 導入の前に別の lane で直す）。申し送りは code を読んだ判断だったので、下の「調べ」で今の main の現物と一時の test で確かめ直した。

起票の commit は本 packet と、設計正本の proposed の文（[71](../function-design/71-mnt-backup.md) §71.10 の表の行・§71.11.1 の `OFFSITE_KEEP` の注記・§71.11.4 手順 2・4g・§71.11.5・§71.11.7・MNT-01-D8 の保持の文・MNT-01-D12）。`src-tauri/**` は計画の固定の後の実装で変える。decision-log に新しい D は足さない（局所の判断で、子の ID の MNT-01-D12 に置く。DEV_WORKFLOW の Design Phase Rules）。

## Workflow State

- Risk: R4
- Plan Commit: 82d9e9d1bb0a91baa407474fab17d04c24be4e57
- Amendments: none

## Risk

Risk: R4

Reason:
PC の外の控えの媒体（USB メモリ）の上の掃除（71 §71.11.4 手順 4g、`src-tauri/src/mnt/offsite.rs` の `copy_to_medium`）が消す file の組を変える: 名前の日付が今日より後の写しを 30 本の数に入れず、消さない（owner の決定 TD-820、2026-11-19）。消す経路の変更で、`docs/DEV_WORKFLOW.md` Risk Tiers の R4「破壊的なデータの lifecycle」に当たる。あわせて写す元の選び方と古さの知らせ（`stale`・`local_backup_old`）の判定を変える（MNT-01-D12 (1)・(2)）。写す元の選び方が変わるので、今の main では手順 4g に届かなかった媒体でも、写せた後に通常の保持の削除が動く（下の「調べ」の F11・S4）。PC の中の file は変わらない: PC の中の掃除（71 §71.5）、上書きしない公開（§71.4.1）、復元（§71.7）の経路と規則を変えず、DB・migration・command・DTO の形・状態の file の形・画面の file に触れない。

## Goal

Goal Invariant:

### 最小完了条件

- PC の中に PC の日付より先の日付の名前の backup が残っていても、用意した USB メモリが見えていれば、PC の日付で今日までの最新の backup が写る（PC の時計が先へ進んでから直った後、時計が戻っている間、手で置いた file のどれでも）。
- 先の日付の成功の記録（時計が先のときに写した記録、時計が戻る前に写した記録）だけを理由に、写せていないのに知らせが出ない状態が続かない: その記録は無いものとして扱い、今の「まだ写せていません」の知らせが出る。成功の記録の有効さ（無いものとして扱うか、何日前と数えるか）は読取りの日と記録の `min(写した日, 控えの日付)` を比べて決め（71 §71.11.5。古い中身を写しただけでは知らせは消えない）、記録の更新は 71 §71.11.4 手順 5・6 に従う。
- PC の中の backup の古さ（`local_backup_old`）は、先の日付の file を数えずに決まる（今日までの backup が 3 日以上無ければ、PC の中の backup の直し方の文が出る）。
- 媒体の本当の日付の写しが、先の日付の名前の写しに押し出されて消えない: 媒体の掃除は名前の日付が今日以前の写しだけを数えて新しい 30 本を残し、今日より後の名前の写しは数えず消さない。

### 失敗定義

- 先の日付の file が写す元になる、または先の日付の file があることで今日までの最新が写らない。
- 名前の日付が今日の backup を写す元から外す、または明日の backup を写す元にする（境界の誤り）。
- 先の日付の成功の記録で `stale = false` を返す。
- 写した日・控えの日付の片方だけが先の記録（例: 今の main で先の日付の file を写した記録）の日数の数え方が変わる。
- 媒体の今日以前の名前の写しが 30 本以下なのに消える（先の日付の名前の写しに押し出される）。媒体の先の日付の名前の写し・今写した 1 本が消える。
- 媒体の今日以前の名前の写しの保持（新しい 30 本を残し、30 本目より後の写しを消す）が変わる。
- アプリが先の日付の file・状態の file の記録を消す・名前を変える・書き換える。
- PC の中の掃除・一覧・今日の backup の判定・復元の振舞いが変わる。
- 画面の文字列・DTO の形・bindings が変わる。

### 非目的

- PC の中の保持日数の掃除の規則（71 §71.5、MNT-01-D3。owner の決定 TD-673 で今のまま）。先の日付の file は PC の日付がその日付 + 保持日数を過ぎるまで残る。
- 先の日付の file があること・PC の時計がずれていることを画面で知らせる新しい文（TD-816 で採らなかった。71 MNT-01-D12 の Rejected alternatives）。
- バックアップ画面の一覧の並びと先頭の「最新」の印（68 UI-11b-D7。先の日付の file が先頭に出る。TD-816 の J1 = A で一覧は出し分けない）。
- 媒体の上に既にある先の日付の名前の写しの片付け（消さない・退避しない。PC の日付がその日付になるまで 30 本の外に残り、本数の上限は無い。`OffsiteMediumView.newest_copy` もそれを返す〈画面は使っていない〉）。
- 同じ日の中の時刻まで比べること（下の Design Readiness の絶対保証の例外）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 調べ（2026-11-19、main `3b7823b5`。code の読みと、commit しない一時の test の実行）

申し送りの筋は今の main で成り立つ。ただし「知らせも出ない」は筋によって違う（下の S2）。

- F1 写す元: `check_body` は `list_backups(backup_dir)` の先頭をそのまま写す元にする（`src-tauri/src/mnt/offsite.rs:469-473`）。`list_backups` は名前の日時の降順（`src-tauri/src/mnt/backup.rs:801`・`:815`）で、名前の日付と今日を比べる所はどこにも無い。
- F2 「同じ写しがある」: 見えている媒体の全部に同じ名前・同じ size の正式名があれば `UpToDate` を返し、状態を書かない（`offsite.rs:513-518`、判定は `:1097-1102`）。
- F3 日数と `stale`: `today - copied_on.min(content_on)`（`offsite.rs:722-728`）。写した日と控えの日付の両方が今日より後なら負になり、`stale = prepared && days.is_none_or(|days| days >= OFFSITE_STALE_DAYS)`（`:754`）は false。
- F4 `local_backup_old`: `list_backups` の先頭の名前の日付で `(today - date).num_days() >= OFFSITE_STALE_DAYS`（`offsite.rs:684-691`）。先頭が先の日付なら負で false。
- F5 PC の中の掃除: `age.num_days() > retention_days`（`backup.rs:763-765`）。先の日付の file は負の日数で、PC の日付がその日付 + 保持日数を過ぎるまで消えない（非目的。変えない）。
- F6 今日の backup の判定: 今日の日付の prefix の正式名で数える（`backup.rs:833-835`・`:854-858`）ので、先の日付の file があっても今日の backup は作られる。作られた今日の backup が写す元にならないのが F1。
- F7 直す場所: `check_body` は確認（`offsite.rs:448`）・用意の手順 6（`:431`）・作り直しの手順 7（`:771`・`:838`）の全部が通る 1 か所。古さの判定は `status_with`（`:678`）の 1 か所で、CMD は今日のローカル日付を渡す（`src-tauri/src/cmd/settings_cmd.rs:431-441`）。`list_backups` は一覧の command（`settings_cmd.rs:355-364`）も使うので、そこでは除かない。
- F8 時計の注入: 確認は `Env.now`（`offsite.rs:191-198`）、古さは `status_with` の `today` 引数で既に注入できる。既存の test は `now()` を固定し（`:1305`）、`Fx`（`:1333`）・`name_on`（`:1399`）・`put_backup`（`:1403`）で一時 directory に合成の控えを置く。別の `now` の `Env` を作れば「時計が先」「時計が戻る」を再現でき、新しい注入の口は要らない。
- F9 画面の文: `src/features/backup-restore/offsite-status-text.ts:70-74` は `stale` のとき日数を「{N} 日写せていません」と出し、`last_success_at === null` なら「まだ写せていません」の文にする。日数が負のまま `stale` にすると「-30 日」と出る。
- F10 自動 backup: `backup_enabled` の既定は `'1'`（`src-tauri/src/db/schema_v1.rs:296`）。共通の確認は 60 秒ごとに `checkAutoBackup` の後で `checkOffsiteBackup` を呼ぶ（68 UI-11b-D16、`src/features/backup-restore/useAutoBackupCheck.ts:20`）。
- F11 媒体の掃除: `copy_to_medium` の手順 4g は `official_copies(dir)` を名前の降順に並べ、`OFFSITE_KEEP`（30）本目より後を消す。守るのは今写した名前だけで、名前の日付と今日を比べない（`offsite.rs:665-674`）。削除の失敗は `tracing::warn!` で続ける（`:671-673`）。`official_copies` は `extract_datetime_from_backup` に通る名前の、reparse point でない file（`:1081-1090`）で、呼ぶのは 4g と `newest_copy`（`:1092-1094`）の 2 か所。手順 4g に届くのは写した後だけなので、今の main は F1・F2 で `UpToDate` に止まる間は掃除が動かない。写す元を直すと、今日の backup が写るたびに動く。4g の「今」は確認の始めに取る `Env.now`（`:249-257`）で、手順 2 と同じ値を使える。

### 一時の test での再現

`src-tauri/src/mnt/offsite.rs` の `mod tests` に、既存の fixture（`Fx`・`put_backup`・`name_on`）と別の `now` の `Env` で 3 つの筋を流し、結果を `println!` する test を一時に足して流した（commit していない。同じ筋は Test Plan の T3・T1・T4 が回帰 test として持つ）。`now()` は fixture の固定値 2026-11-02 09:00:00。

測定の command（作業 clone の `src-tauri` で）: `CARGO_TARGET_DIR=<scratchpad> cargo test --offline --lib probe_s -- --nocapture --test-threads=1`。出力の末尾: `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1636 filtered out`、exit 0。

S4 は媒体の掃除の筋で、別の回に流した（head `a64f114b` の隔離した copy に一時の test を 2 つ足した。commit していない。同じ組は T7 が持つ）。測定の command（隔離した copy の `src-tauri` で）: `CARGO_TARGET_DIR=/tmp/shared-cargo-target CARGO_BUILD_JOBS=4 cargo test --offline --lib probe_pr287 -- --nocapture --test-threads=1`。出力の末尾: `test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 1636 filtered out`、exit 101（(a) の「全部が残る」の assertion が今の code で red、(b) は green）。

| 筋 | 入力 | 出力（原文） | 読み |
|---|---|---|---|
| S1 時計が先 → 直る | 時計が 30 日先（2026-12-02）のときに、その日付の控えを作って確認 → 時計を戻し、今日の控えを置いて確認 | `S1 first(clock ahead) = Ok(Copied { file_name: "inventory_backup_20261202_090000.db", labels: ["控え 1"] })`／`S1 second(clock fixed) = Ok(UpToDate)`／`S1 medium = ["inventory_backup_20261202_090000.db", "offsite-medium.json"]`／`S1 status(+0) days=Some(-30) stale=false local_backup_old=false`／`S1 status(+29) days=Some(-1) stale=false local_backup_old=false`／`S1 no medium(+10) days=Some(-20) stale=false attached=0` | 今日の控えは写らず、知らせも出ない。媒体が見えなくても出ない。PC の日付が先の日付になるまで続く |
| S2 時計は正しい・先の日付の file がある | 30 日先の日付の控えと今日の控えを置いて確認 2 回 → 翌日の控えを置いて翌日の時計で確認 | `S2 first = Ok(Copied { file_name: "inventory_backup_20261202_090000.db", labels: ["控え 1"] })`／`S2 second = Ok(UpToDate)`／`S2 next day = Ok(UpToDate)`／`S2 status(+2) days=Some(2) stale=false local_backup_old=false attached=1`／`S2 status(+3) days=Some(3) stale=true local_backup_old=false attached=1` | 先の日付の file が写り、今日と翌日の控えは写らない。3 日後に知らせは出るが、条件（`attached` あり・`local_backup_old` false・失敗の記録なし）は UI-11b-D14 の (5) の「USB メモリは差してありますが、写せません。…別の USB メモリを…用意してください」に当たり、直し方が外れる（文の選び方は F9 の file の読み） |
| S3 時計が戻る | 正しい時計で今日の控えを写す → 時計を 10 日戻し、その日付の控えを置いて確認 | `S3 first = Ok(Copied { file_name: "inventory_backup_20261102_080000.db", labels: ["控え 1"] })`／`S3 second(clock back) = Ok(UpToDate)`／`S3 medium = ["inventory_backup_20261102_080000.db", "offsite-medium.json"]`／`S3 status(back+0) days=Some(-10) stale=false local_backup_old=false` | 時計が戻っている間に作った控えは写らず、知らせも出ない |
| S4 媒体の掃除 | (a) 媒体に先の日付の名前 30 本（`20261103`〜`20261202`、各 `120000`）と今日より前の名前 5 本（`20261028`〜`20261101`）を置き、PC の中に `20261102_080000` だけを置いて確認（PC の中に先の日付の file が無いので、今の code でも今日の file が写す元になる）。(b) 先の日付の名前の無い媒体に今日より前の名前を 29 本・30 本置き、同じく確認 | `T7a check = Ok(Copied { file_name: "inventory_backup_20261102_080000.db", labels: ["控え 1"] })`／`T7a kept_real = [] kept_future = 30 today = true`／`T7b on_medium=29 check=Ok(Copied { file_name: "inventory_backup_20261102_080000.db", labels: ["控え 1"] }) removed=[] today=true`／`T7b on_medium=30 check=Ok(Copied { file_name: "inventory_backup_20261102_080000.db", labels: ["控え 1"] }) removed=["inventory_backup_20261003_120000.db"] today=true` | (a) 今日の file が写ると、今日より前の 5 本が全部消え、先の日付の 30 本と今写した 1 本だけが残る。(b) 先の日付の名前が無ければ、今日以前が 30 本ちょうどで消えず、31 本で最も古い 1 本が消える（今の保持の規則） |

未実測: Windows の実機と実際の PC の時計の変更での挙動、店の PC の時計がずれた実績。上の 4 つは WSL の unit test（一時 directory、注入した時計）での再現。

## owner の決定（2026-11-19、TD-816・TD-820）

repo 外の回答台帳の ID と要旨で書く（原文は台帳）。選ばなかった案は 71 MNT-01-D12 の Rejected alternatives。

- TD-816 の J1 = A: PC の中の先の日付の file はそのままにする（消さない・退避しない・一覧で出し分けない）。写す元と古さの判定に使わないだけで、file は今の掃除の規則（PC の日付がその日付 + 保持日数を過ぎた日）で消える。
- TD-816 の J2 = (a): 先の日付の成功の記録は記録なしの扱いにし、今ある「まだ写せていません」の文で知らせる。画面の文字列と DTO は変えない。
- TD-816: L3 は置かない。
- TD-820: 媒体の掃除（手順 4g）も直す。先の日付の名前の写しは 30 本の数に入れず、消さない。今日以前の名前だけで新しい 30 本を残す。消す経路の変更なので R4。

理由（起票の調べと計画の review の根拠。owner の発言の原文ではない）: 目的（写しが止まらない・知らせが出る）は PC の中の file を消さない形で満たせ、知らせは画面と DTO を変えずに知らせる側へ倒せる。写す元だけを直すと、止まっていた媒体の掃除が動いて本当の日付の写しを押し出す（調べの S4）ので、掃除も同じ日付の比べ方に揃える。

## Scope

runtime（計画の固定の後）:

- `src-tauri/src/mnt/offsite.rs`:
  - private の関数を 1 つ足す: `list_backups` の結果（新しい順）と今日の日付を受け、名前の日付（`extract_date_from_backup`）が今日以前の最初の 1 本を返す。
  - `check_body` の手順 2 の写す元を、その関数（今日 = `env.now.date()`）で選ぶ。無ければ今どおり `NoLocalBackup`。`list_backups` の `Err` の扱いは変えない。
  - `copy_to_medium` の手順 4g: `official_copies(dir)` のうち、名前の日付（`extract_date_from_backup`。写す元と同じ解析）が `env.now.date()` より後の写しを、`copies` の数と削除の両方から外す。今写した 1 本を消さない今の規則は残す。日付を読めない名前は `official_copies` に入らないので今のまま。削除の失敗は今どおり `tracing::warn!` で続ける。`official_copies`・`newest_copy`・`has_same_copy` は変えない。
  - `status_with` の `local_backup_old` を同じ関数（`today`）で決める。`list_backups` の `Err` は今どおり false。
  - `status_with` は、写した日と控えの日付の古い方が `today` より後の成功の記録を無いものとして扱う（`last_success_at`・`last_success_label`・`days_since_last_success` が `None`、`prepared` なら `stale = true`）。状態の file は読むだけ。
  - 意図した簡略化は `ponytail:` comment に書く（日付で比べ時刻を見ない。広げる条件は MNT-01-D12 の見直し契機）。
  - test（下の Test Plan の T1〜T7、AC7 の入力）。`mod tests` に別の `now` の `Env` を作る小さい helper を 1 つ足してよい。
- `docs/function-design/71-mnt-backup.md`: MNT-01-D12 の見出しの proposed の注記を外す。実装で文と食い違った所があれば同じ commit で直す。
- `docs/function-design/90-traceability.md`: generator で作り直す（`cd src-tauri && cargo run --bin generate_traceability`。手で編集しない）。
- `docs/backlog.md`: 「backup に、PC の外のコピーと復元の実証が運用として設計されていない」の項目で、この lane が済ませる「(b)（PR #194）の申し送り」の行を外し、「導入の前の関門」の段の「残り:」から「先の日付の backup の件（下の「(b)（PR #194）の申し送り」）」を外す（同じ PR で行い、merge の後の宿題にしない）。一覧の「最新」の印が先の日付の file に付く件を、残りとして 1 行置く（出所: 本起票の調べ〈AI〉。owner の確認の前の要件にしない）。

起票の commit: 本 packet と、71 の §71.10 の表（MNT-01-D8 保持・状態の行、MNT-01-D12 の行）・§71.11.1 の `OFFSITE_KEEP` の注記・§71.11.4 手順 2・4g・§71.11.5（DTO の注記、規則、`local_backup_old`）・§71.11.7 の 2 つの箇条・MNT-01-D8 の決定の保持の文・MNT-01-D12（proposed）。

path の所有（第 1 波の他の lane との突き合わせ）: `src-tauri/src/mnt/offsite.rs` と 71 の上の「起票の commit」の箇所は本 lane。`docs/backlog.md` は上の項目の 2 か所と足す 1 行だけ。`docs/function-design/90-traceability.md` は生成物で、同期で衝突したら取り込んだ後に作り直す。

Non-scope: `src-tauri/src/mnt/backup.rs`（`list_backups`・`cleanup_old_backups`・`check_auto_backup`・`extract_date_from_backup` は呼ぶだけ）、`src-tauri/src/cmd/**`、`src/**`（画面・`offsite-status-text.ts`・bindings）、68・53・43 の文、`docs/decision-log.md`、`Plans.md`、migration、状態の file の形、媒体の目印の形。

## Acceptance Criteria

- AC1（写す元の境界）: PC の中が今日の `23:59:59` の名前の正式名と明日の `00:00:00` の名前の正式名の 2 本で（`now()` は今日の 09:00:00。今日の名前の時刻は今より後）、用意済みの媒体が見えているとき、確認が `Copied { file_name: 今日の file }` を返し、媒体に今日の file があって明日の file が無く、PC の中の明日の file は残る。次の確認は `UpToDate`。用意（手順 6）の最初の写しも同じ file を選ぶ。
- AC2: PC の中が先の日付の file だけのとき、確認は `NoLocalBackup` を返し、媒体の一覧と状態の file の bytes が前後で同じ。
- AC3（時計が先 → 直る）: 時計が 30 日先のときに写した後、時計を戻した今日の日付の `offsite_status` は `last_success_at = None`・`last_success_label = None`・`days_since_last_success = None`・`stale = true` を返す（媒体が見えていても見えなくても）。その後に今日の控えを PC の中に置くと、媒体が見えない確認は `MediumMissing` で状態の file の bytes を変えず、同じ file のまま媒体を差し直した確認は `Copied`（今日の file）。写した後の状態の file の成功の記録は `at` が注入した今の時刻、`file_name` が今日の file で、`offsite_status` は `days_since_last_success = Some(0)`・`stale = false`。次の確認は `UpToDate` で状態を変えない。
- AC4（時計が戻る）: 正しい時計で写した後に時計を 10 日戻すと、その日付の `offsite_status` は成功の 3 項目が `None`・`stale = true`。その後にその日付の控えを PC の中に置くと、媒体が見えない確認は `MediumMissing`（状態を変えない）、差し直した確認は `Copied`（その日付の file）。媒体に前の写し（戻した時計では先の日付の名前）と今写した file の両方があり、状態の file の成功の記録は `at` が戻した時計の時刻、`file_name` がその日付の file で、`stale = false`。
- AC5: `local_backup_old` は、先の日付の file と 10 日前の file がある dir で `true`、先の日付の file と 2 日前の file で `false`、先の日付の file だけで `true`。
- AC6: 成功の記録の日数は、写した日が 5 日先・控えの日付が 1 日前なら 1、写した日が 1 日前・控えの日付が 5 日先なら 1（どちらも今どおり古い方から数え、`stale = false`）。両方が 1 日先なら記録なしの扱い（`stale = true`）、両方が今日なら 0。`offsite_status` の前後で状態の file の bytes が同じ。
- AC7: 既存の test が期待値を変えず、入力も reparse point の test の (3) のほかは変えずに全部 PASS する（`mnt::offsite`・`mnt::backup`・`cmd::settings_cmd`。特に `test_offsite_status_req901_d8_local_backup_old`・`test_offsite_status_req901_d8_stale_boundary_three_days`・`test_offsite_req901_d8_stale_clears_only_after_new_copy`・`test_check_offsite_req901_d8_retention_keeps_newest_and_foreign_files` の case 1・case 2）。`git diff --stat origin/main...HEAD` に `src/**`・`src-tauri/src/cmd/**`・`src-tauri/src/mnt/backup.rs` が出ない。
- AC8: mutation を実装に注入し、当たる test が red になることを確かめて PR に書く: (m1) 写す元を一覧の先頭に戻す → T1・T3・T4、(m2) 写す元の日付の比較を「今日より前」にする → T1（`NoLocalBackup` になる）、(m3) `local_backup_old` を一覧の先頭に戻す → T5、(m4) 先の日付の記録の扱いを外す → T3・T4・T6、(m5) 「両方が先」を「どちらかが先」にする → T6、(m6) 写す元の比べる先を `today + 1` にする → T1（明日の file が写る）、(m7) 手順 4g を名前の全部で数える今の形に戻す → T7 (a)、(m8) 手順 4g の比べる先を `today + 1` にする → T7 (c)、(m9) 手順 4g の今写した 1 本を消さない除外を外す → T7 (e)（今写した 1 本が媒体から消える）、(m10) 手順 4g で link を数えるが消さない → AC7 の (3)。
- AC9: `rg -n "proposed" docs/function-design/71-mnt-backup.md` に MNT-01-D12 の行が出ない。`rg -n "\(b\)（PR #194）の申し送り" docs/backlog.md` が 0 件。
- AC10: `bash scripts/doc-consistency-check.sh` と `--target plan docs/plans/2026-11-19-offsite-future-dated-backup.md` が ERROR 0、`bash scripts/check-workflow-git.sh` が PASS、`cd src-tauri && cargo run --bin generate_traceability -- --check` が PASS。Rust の検証は `docs/DEV_WORKFLOW.md` の Verification Gates（hosted CI の `Merge gate`）。
- AC11（媒体の掃除、`now()` = 2026-11-02。PC の中は `20261102_080000` の 1 本で、確認はどれも `Copied`）:
  - (a) 媒体に先の日付の名前 30 本（`20261103`〜`20261202`）と今日より前の名前 5 本（`20261028`〜`20261101`）があるとき、写した後の媒体に、先の日付の 30 本・今日より前の 5 本・今写した 1 本の全部と目印が残る。
  - (b) 先の日付の名前の無い媒体で、今日より前の名前が 29 本なら写した後に何も消えず（今日以前が 30 本ちょうど）、30 本なら最も古い 1 本だけが消える（31 本）。その 30 本のうち 1 本を今日の `23:59:59` の名前に替えても、最も古い 1 本だけが消える（日付だけで比べ、時刻が今より後の今日の名前も数える）。
  - (c) 媒体に明日の `00:00:00` の名前 1 本と今日より前の名前 29 本があるとき、写した後に何も消えない（明日の名前は数えない。今日以前は 30 本ちょうど）。
  - (d) 媒体に今日より前の名前が 31 本あるとき（消す対象は古い 2 本）、その 2 本のどちらの削除の失敗を注入しても、確認は `Copied` のままで、失敗した 1 本は残り、もう 1 本は消え、成功の記録が書かれる。
  - (e) 媒体に今日の名前で今写す file より時刻が後の写しが 30 本（`20261102` の `090000`〜`092900`、1 分ごと）あるとき（今写す file は今日以前の名前の新しい順で 30 本目より後になる）、写した後の媒体に、その 30 本・今写した 1 本の全部と目印が残り、成功の記録の `file_name` が今写した file。

## Design Readiness

- 引用する設計正本（節まで）: 71 §71.3（名前の規約）・§71.5（MNT-01-D3）・§71.6・§71.8・§71.11.1（`OFFSITE_KEEP`）・§71.11.3（媒体の固定・reparse point）・§71.11.4（手順 2・3a・4a・4g、エラーの表）・§71.11.5・§71.11.7・MNT-01-D8・MNT-01-D11・MNT-01-D12、68 UI-11b-D7・D14・D16、53 UI-00-D12、43 §43.8.4、D-114、D-127。
- 必要な設計成果物: updated in this PR（71 の proposed の文は起票の commit。runtime で注記を外す）。68・53・43 は existing sufficient（`offsite_status` が返す形は今の「一度も写せていない」と同じで、画面の文の選び方を変えない）。DB・migration は不要。
- plan にしかない durable な判断の昇格先: なし（owner の決定 TD-816・TD-820 と選ばなかった案は MNT-01-D12 の Why と Rejected alternatives に置いた）。
- 前提・制約と延期した design gap: 一覧の「最新」の印が先の日付の file に付く件は非目的で、backlog に残りとして 1 行置く（Scope）。Windows の実機での確かめは置かない（TD-816）: 状態を作るには PC の時計の変更か file の名前の書換えが要り、L3 Eligibility の条件 (3)（故障注入の手順）に当たるので、注入した時計の自動 test で持つ。画面の file と文字列を変えないので L3 の節は置かない。
- 絶対保証（cannot happen / always happens）の例外と抜け道:
  - 「今日までの最新が写る」は名前の日付で比べる。同じ日の中で時計が数時間先へ進んでから直ると、その日の残りの backup は名前の時刻が前なので写らず、翌日の backup から写る（1 日以内。店主の許容は 3 日、L-220）。
  - 先の日付の file は、PC の日付がその日付になった日に写す元に戻る。同じ日の backup より名前の時刻が後なら、その日は古い中身のその file が最新として写る（それより名前の日時が後の backup ができた時に直る）。媒体の上の先の日付の名前の写しも、同じ日に 30 本の数に戻る。
  - 先の日付の記録が無いものとして扱われる間は、確認が `UpToDate` でも知らせが出続ける（TD-816 の J2 = (a) で残るもの。この扱いが終わるのは、71 §71.11.4 手順 5・6 が記録を更新した時か、読取りの日が記録の `min(写した日, 控えの日付)` になった時）。USB メモリが見えなければ「…USB メモリが差してあるか確かめてください。」、差してあれば「USB メモリは差してありますが、写せません。…」の文で、後者は直し方が外れる。自動 backup が有効なら、今日の backup がまだ無い日は次の 60 秒の確認で backup が作られて写り、今日の backup が既に写っている日は次の backup（設定時刻か翌日）まで続く見込み（F6・F10 の code の読み。未実測）。公開の後の sync の失敗で記録が古いまま `UpToDate` になる今の場合と同じ扱い（71 §71.11.4 のエラーの表）。
  - 記録が更新されても知らせが消えるとは限らない: 写した file の名前の日付が 3 日以上前なら、更新された記録でも `stale` のまま（中身の古さで知らせる今の規則。71 §71.11.5）。
  - PC の中の保存先が媒体の `InventoryBackup` と同じ folder（MNT-01-D11）で、そこが先の日付の file だけのとき、確認は手順 2 の `NoLocalBackup` で終わり手順 3 の `BackupDirIsMediumFolder` を記録しない（手順の順は今のまま。空の folder と同じ）。`local_backup_old` の文が導く「今すぐバックアップを作成」の後の確認で記録される。
  - 「消さない・書き換えない」は、PC の中では `check_body` と `status_with` に削除・改名・状態の書込みを足さないことに依る（T2・T6 が前後の bytes で確かめる）。媒体の上で消すのは手順 4g だけで、消す対象は今日以前の名前の、新しい順で 30 本目より後の写しに限る（今写した 1 本を除く。T7）。
  - 媒体の写しは 30 本を超えうる: 先の日付の名前の写しは数えないので、その本数だけ多く残る（上限は無い）。空きが足りなくなれば写しは `StorageFull` で失敗し、今の文で知らせる（71 §71.11.7）。
  - 時計が先へ進んでいる間は、その日付が PC の「今日」なので、媒体の掃除は今と同じに数える（先へ進んだまま 30 本を超えて写せば、前の本当の日付の写しは今と同じく古い順に消える。時計が今ずれているかをアプリは見分けられない）。時計が戻っている間は、本当の日付の写しが先の日付の名前になるので数えず消さない。
  - 手順 4g の「今日」は確認の始めの `Env.now` の日付で、手順 2 と同じ値（1 回の確認の途中で日付が変わっても、写す元と掃除の比べ方は割れない）。
- 判定: ready。owner の決定（TD-816・TD-820）を 71 と本 packet に入れ、残る未決の判断は無い。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| MNT-01-D12 (1) 写す元 | 71 §71.11.4 手順 2 | `offsite.rs` の新しい関数と `check_body` | T1・T2・T3・T4 | 非対象（L3 なし。TD-816、Design Readiness） |
| MNT-01-D12 (1) `local_backup_old` | 71 §71.11.5 | `offsite.rs` の `status_with` | T5 | 非対象 |
| MNT-01-D12 (2) 先の日付の成功の記録 | 71 §71.11.5 | `offsite.rs` の `status_with` | T3・T4・T6 | 非対象 |
| MNT-01-D12 (3) 消さない・書き換えない | 71 §71.11.7 | PC の中は足さない。媒体の上は (4) | T1（PC の中の file が残る）・T2・T6（前後の bytes）・T7（媒体の先の日付の名前の写しが残る） | 非対象 |
| MNT-01-D12 (4) 媒体の掃除（先の日付の名前の写しを数えず消さない） | 71 §71.11.4 手順 4g、§71.11.7 | `offsite.rs` の `copy_to_medium` の手順 4g | T7 | 非対象 |
| MNT-01-D8 保持（今日以前の名前の新しい 30 本。今写した file・規約外の file を消さない。削除の失敗は warn で続ける） | 71 §71.11.1 の `OFFSITE_KEEP`、§71.11.4 手順 4g とエラーの表の「掃除の失敗」、MNT-01-D8 の決定 | 今日以前の名前の中の数え方・今写した file の扱い・失敗の扱いは変えない | 既存 `test_check_offsite_req901_d8_retention_keeps_newest_and_foreign_files`（期待値を変えない。case 2 の 30 本は全部が明日以降の名前で、新しい規則では数える前に外れ、今写した file の除外を通らない。除外は T7 (e)）、T7 (b)・(d)・(e) | 非対象 |
| MNT-01-D8 媒体の固定・reparse point（掃除の直前の確かめ直し、link を写しに数えず消さない） | 71 §71.11.3 | 変えない（手順 4g の確かめ直しと `official_copies` をそのまま通る） | 既存 `test_check_offsite_req901_d8_medium_swap_after_discovery_writes_nothing`・`test_check_offsite_req901_d8_reparse_points_are_not_followed`。(3) の 60 日先の link は日付で外れるので今日の `120000` にする | 非対象 |
| MNT-01-D8（手順 3a〜6、`stale` の 3 日の境界、日数は古い方から） | 71 §71.11.4・§71.11.5 | 変えない | 既存 `test_check_offsite_req901_d8_up_to_date_skips_copy`・`test_offsite_status_req901_d8_stale_boundary_three_days`・`test_offsite_req901_d8_stale_clears_only_after_new_copy`、T6 | 非対象 |
| MNT-01-D8 用意の手順 6・作り直しの手順 7 | 71 §71.11.2・§71.11.5a | 変えない（同じ `check_body` を通る） | T1 の用意の case | 非対象 |
| MNT-01-D8 `OffsiteMediumView.newest_copy` | 71 §71.11.2 | 変えない（先の日付の名前も返す。非目的） | 非対象（画面は使っていない） | 非対象 |
| MNT-01-D11（手順 3 の同じ folder の判定） | 71 §71.11.4 手順 3 | 変えない（手順の順も変えない） | 既存 `test_check_offsite_req901_d11_backup_dir_same_folder_writes_nothing` | 非対象 |
| MNT-01-D3・§71.5（PC の中の掃除） | 71 §71.5 | 変えない | 既存 `test_cleanup_old_backups_req901_deletes_expired`・`test_cleanup_old_backups_req901_keeps_recent` | 非対象 |
| §71.6 `list_backups`（一覧の command も使う） | 71 §71.6 | 変えない | 既存 `test_list_backups_req901_returns_sorted` | 非対象 |
| §71.8 今日の backup の判定 | 71 §71.8 | 変えない | 既存 `test_check_auto_backup_req901_no_backup_today` | 非対象 |
| 43 §43.8.4（`today` は今日のローカル日付） | 43 §43.8.4 | 変えない | 非対象（CMD は渡すだけ） | 非対象 |
| UI-11b-D14・UI-00-D12（原因の文の選び方） | 68 §68.5、53 | 変えない（返す形は「一度も写せていない」と同じ） | 既存 `offsite-status-text.test.ts` の「用意の前・写せた後・まだ写せていない」「(2) PC の中の backup が古い（(3) より先）」「(5) 差してあるかで 2 つの文（札を名指さない）」 | 非対象 |
| UI-11b-D7（一覧の並びと「最新」の印） | 68 §68.5 | 変えない | 非対象 | 非対象（TD-816 の J1 = A） |
| D-114（Guarantee range） | decision-log | 変えない（正本は 71） | 非対象 | 非対象 |

## Test Plan

Rust の unit test（`src-tauri/src/mnt/offsite.rs` の `mod tests`。一時 directory と合成の控え、注入した時計）。test の先頭の comment に `REQ-901 / MNT-01-D12` と節を書く。

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| MNT-01-D12 (1) 写す元 | 明日の名前の file が写る。今日の file を外す | unit（T1。入力は今日の `23:59:59` の名前と明日の `00:00:00` の名前の 2 本だけ。昨日以前の file を置かない〈m2 が `NoLocalBackup` で red になる組を保つ〉） | `test_check_offsite_req901_d12_skips_future_dated_source` | 写す元を一覧の先頭に戻す（m1）、比較を「今日より前」にする（m2。`NoLocalBackup` になる）、比べる先を `today + 1` にする（m6。明日の file が写る）、日時で比べる（今日の `23:59:59` を外す）、用意の手順 6 が別の選び方をする |
| MNT-01-D12 (1)・(3) | 先の日付の file だけのときに写す・失敗を記録する・何かを消す | unit（T2） | `test_check_offsite_req901_d12_only_future_dated_is_no_local_backup` | 無いときに先の日付の file へ戻す fallback を足す、`last_failure` を書く |
| MNT-01-D12 (1)・(2)（時計が先 → 直る、S1） | 今日の控えが写らない。先の日付の記録で知らせが出ない。写した後も記録が先の日付のまま残る | unit（T3、状態の遷移。30 日先の時計のずれはここが持つ） | `test_offsite_req901_d12_clock_ahead_then_fixed_copies_and_clears_stale` | m1、先の日付の記録の扱いを外す（m4）、媒体が見えないときだけ別の判定にする、`Copied` で成功の記録の `at` を書き直さない（読取りの成功の 3 項目が `None` と、写した後の `at` = 注入した時刻・`file_name` = 今日の file の対で見る） |
| MNT-01-D12 (1)・(2)（時計が戻る、S3） | 戻っている間の控えが写らない・知らせが出ない | unit（T4、状態の遷移） | `test_offsite_req901_d12_clock_set_back_copies_backdated_backup` | m1、m4 |
| MNT-01-D12 (1) `local_backup_old` | 先の日付の file があると PC の中の backup が新しいと数える | unit（T5、表） | `test_offsite_status_req901_d12_local_backup_old_ignores_future_dated` | `local_backup_old` を一覧の先頭に戻す（m3） |
| MNT-01-D12 (2)、MNT-01-D8 の日数 | 片方だけ先の記録を記録なしにする・日数が負になる。`offsite_status` が状態を書く | unit（T6、表） | `test_offsite_status_req901_d12_future_record_is_treated_as_absent` | 「両方が先」を「どちらかが先」にする（m5）、m4、状態の file を書き直す |
| MNT-01-D12 (3)・(4)、MNT-01-D8 保持 | 先の日付の名前の写しが本当の日付の写しを押し出して消す。先の日付の名前の写しを消す。今日以前の名前の保持が変わる。今写した 1 本を消す。削除の失敗で止まる | unit（T7、表。AC11 の (a)〜(e) の組を固定する） | `test_check_offsite_req901_d12_retention_ignores_future_dated_copies` | 手順 4g を名前の全部で数える今の形に戻す（m7。(a) で今日より前の 5 本が消える）、先の日付の名前を数えるが消さない形にする（(a)）、先の日付の名前を消す（(a)・(c)）、比べる先を `today + 1` にする（m8。(c) で最も古い 1 本が消える）、今日の名前を数えない・日時で比べる（(b) で消えるはずの 1 本が残る）、削除の失敗を `Err` で返す（(d)）、今写した 1 本を消さない除外を外す（m9。(e) で確認は `Copied` のまま今写した 1 本が媒体から消える） |
| MNT-01-D8・D11・D3・§71.6・§71.8 | 既存の振舞いが変わる | unit（既存） | Contract Ledger の既存の test | 期待値を変えないと通らない変更を入れる |

状態の遷移（T3・T4 が持つ）: initial（先の日付の記録だけ → 成功の 3 項目が `None`・`stale`）、failure（媒体が見えない確認 → `MediumMissing`、状態を変えず `stale` のまま）、retry・success（同じ file のまま媒体が見える確認 → `Copied`、記録の `at` と `file_name` が置き換わる）、refetch（`offsite_status` → `Some(0)`・`stale = false`）、revisit（次の確認 → `UpToDate`、状態を変えない）、restart（時計が直った後の最初の `offsite_status` と確認の順。process の状態を持たないので、同じ file の状態から始める T3 がそのまま持つ）。操作の順は、時計を戻す → `offsite_status` を見る → 今日（T4 は戻した日付）の file を PC の中に置く → 媒体なしの確認 → 差し直した確認（file を置く前の確認は手順 2 の `NoLocalBackup` が先に返り、`MediumMissing` に届かない）。negative は T2・T6・T7 の (c)・(d)・(e)。mutation は AC8 を実装に注入して red を確かめる。

## Recovery

- 消えた写しは戻らない: 手順 4g は媒体の file を `remove_file` で消す。誤った版が媒体の写しを消しても、アプリにも本 lane にも戻す手立ては無い。
- 代わりになるもの: 失った世代と同じ中身の控えが PC の中か別の媒体に残る場合だけ、それが代わりになる（同じ名前でも中身が同じとは限らない。手で置いた規約の名前の file は掃除の対象で、中身は別でありうる、71 §71.11.7）。PC の中の backup は 71 §71.5 の規則で残る。店の型 C（71 §71.11.6）で持ち帰っている 1 本には最後の入れ替えの時点までの写しが残るが、その媒体も差して写せば同じ掃除が動く。
- 止め方（消しすぎる誤りが分かったとき）: 直した版を入れるまで、用意した USB メモリを全部抜く。全部を外している間、その媒体への写しと保持の削除は走らない（写しと手順 4g は見えている媒体にだけ動く。71 §71.11.4 手順 3・4）。
- code の戻し方: 修正の PR で直すのを既定にする。本 lane の PR を revert した版も入れられる（DB・migration・状態の file の形・媒体の目印の形を変えないので、版を戻しても DB・状態の file・媒体はそのまま読める）。revert した版は名前の全部で 30 本を数える今の規則に戻るので、媒体に先の日付の名前の写しがある間は、次に写した時に本当の日付の写しが押し出されて消えうる（調べの S4 (a)）。
- 消さなすぎる誤り（30 本を超えて残る）: 媒体の上の写しは失わない。媒体の空きが減り、足りなくなれば写しが `StorageFull` で失敗して知らせが出て、新しい世代はその媒体へ写らない。直した版でも満杯の媒体が戻るとは限らない（写しが空きの不足で失敗すれば手順 4g に届かず、先の日付の名前の写しは消さない）。満杯の媒体はそのまま保全し、空きのある別の USB メモリを既存の「この USB メモリを控えの保存先にする」で用意する（画面は変えない）。
- 知らせの誤り（`status_with`）: file を消さない経路で、失うのは「知らせが出る／出ない」。成功の記録の有効さは読取りのたびに決まり直す（記録の更新は 71 §71.11.4 手順 5・6。状態の file の形は変えないので、作り直しは要らない）。写す元の誤りは上の止め方と Data Safety の「今の規則との関係」が受け持つ。
- 本 lane の前に消えた写し: 今の main の版が消した媒体の写しは戻らない（71 MNT-01-D12 の Compatibility）。

## Data Safety

- 手順 4g が消すもの: 媒体の `InventoryBackup\` の直下の、§71.3 の規約の名前の file（reparse point の entry を除く）のうち、名前の日付が今日以前で、新しい順に 30 本目より後の写しだけ（71 §71.11.4 手順 4g）。
- 手順 4g が消さないもの: 名前の日付が今日より後の写し、今写した 1 本、目印、規約に合わない名前の file、reparse point の entry、`InventoryBackup\` の外の file。PC の中の DB・backup・状態の file には届かない（消す対象は媒体の `InventoryBackup\` の一覧から作り、PC の中の保存先が同じ folder のときは手順 3 が先に止める、MNT-01-D11）。
- 今の規則との関係: 同じ媒体の状態から 1 回の掃除で比べると、新しい規則が残す写しは、名前の全部で数える今の規則が残す写しを必ず含む（先の日付の名前は全部残り、今日以前の名前の順位は下がらない）。写す元の選び方が変わるので、今の main では手順 4g に届かなかった媒体でも、写せた後に通常の保持の削除が動く（消す対象は 71 §71.11.4 手順 4g）。
- 誤ったときに失うもの: 媒体の上の古い世代の写し（Recovery のとおり戻らない）。失う範囲は、誤って消えた写しのうち、同じ中身の控えが PC の中にも別の媒体にも残っていない世代。`status_with` の知らせの誤りは file を失わない。
- 時計のずれと安全側: 時計が戻っている間は、本当の日付の写しを数えず消さない（残す側）。時計が先へ進んでいる間は、アプリからはその日付が今日なので今の規則と同じに数え、先へ進んだまま 30 本を超えて写せば前の写しは古い順に消える（今と同じ。本 lane は変えない）。
- commit しないもの: 実の店の DB・backup・媒体の file・状態の file・診断ログ。test の入力は一時 directory の合成の控え（空の DB を `VACUUM INTO` した bytes と合成の名前）だけで、tracked の fixture を足さない。調べの一時の test は隔離した copy で流し、commit しない。
- local だけの path: test の一時 directory（`tempfile`）と cargo の target。実機の USB メモリと PC の時計の変更は使わない（L3 なし）。
- owner の決定と店の事実は repo 外の回答台帳に残し、repo には TD 番号と要旨だけを書く。
