# Plan Packet: 先の日付の backup で PC の外の控えが黙って止まる件を直す

第 1 波の lane（owner の決定 2026-11-19、repo 外の回答台帳 TD-807）。導入の前の関門 3（[Plans](../Plans.md#導入の前の関門)）の残りの 1 つで、出所は [backlog](../backlog.md) の「backup に、PC の外のコピーと復元の実証が運用として設計されていない」の項目の「(b)（PR #194）の申し送り」の行（AI の調べ、2026-11-13。owner の決定 TD-672 の (ii): 導入の前に別の lane で直す）。申し送りは code を読んだ判断だったので、下の「調べ」で今の main の現物と一時の test で確かめ直した。

起票の commit は本 packet と、設計正本の proposed の文（[71](../function-design/71-mnt-backup.md) §71.10 の表の行・§71.11.4 手順 2・§71.11.5・§71.11.7・MNT-01-D12）。`src-tauri/**` は計画の固定の後の実装で変える。decision-log に新しい D は足さない（局所の判断で、子の ID の MNT-01-D12 に置く。DEV_WORKFLOW の Design Phase Rules）。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R3

Reason:
PC の外の控え（MNT-01-D8）の写す元の選び方と、古さの知らせ（`stale`・`local_backup_old`）の判定を変える。データの安全の境界に触れる MNT の契約の変更で、`docs/DEV_WORKFLOW.md` Risk Tiers の「R2 と R3 で迷えば R3（データの安全の境界に触れるとき）」に当たる。R4 にしない理由: 消す・上書きする・復元する経路と規則（71 §71.5 の掃除、§71.11.4 手順 4g の媒体の掃除、§71.4.1 の上書きしない公開、§71.7 の復元）を変えず、DB・migration・command・DTO の形・状態の file の形・画面の file に触れない。誤っても失うのは「写しが 1 回遅れる・知らせが出る／出ない」で、既にある file は変わらない（下の Design Readiness の絶対保証）。R4 になる条件: owner の判断事項 J1 で案 C（アプリが先の日付の file を消す・退避する）を選んだとき（破壊的なデータの lifecycle）。

## Goal

Goal Invariant:

### 最小完了条件

- PC の中に PC の日付より先の日付の名前の backup が残っていても、用意した USB メモリが見えていれば、PC の日付で今日までの最新の backup が写る（PC の時計が先へ進んでから直った後、時計が戻っている間、手で置いた file のどれでも）。
- 先の日付の成功の記録（時計が先のときに写した記録、時計が戻る前に写した記録）だけを理由に、写せていないのに知らせが出ない状態が続かない: その記録は無いものとして扱い、今の「まだ写せていません」の知らせが出て、次に写せた時に消える。
- PC の中の backup の古さ（`local_backup_old`）は、先の日付の file を数えずに決まる（今日までの backup が 3 日以上無ければ、PC の中の backup の直し方の文が出る）。

### 失敗定義

- 先の日付の file が写す元になる、または先の日付の file があることで今日までの最新が写らない。
- 名前の日付が今日の backup を写す元から外す（境界の誤り）。
- 先の日付の成功の記録で `stale = false` を返す。
- 写した日・控えの日付の片方だけが先の記録（例: 今の main で先の日付の file を写した記録）の日数の数え方が変わる。
- アプリが先の日付の file・状態の file の記録を消す・名前を変える・書き換える。
- PC の中の掃除・一覧・今日の backup の判定・媒体の掃除・復元の振舞いが変わる。
- 画面の文字列・DTO の形・bindings が変わる。

### 非目的

- PC の中の保持日数の掃除の規則（71 §71.5、MNT-01-D3。owner の決定 TD-673 で今のまま）。先の日付の file は PC の日付がその日付 + 保持日数を過ぎるまで残る。
- 先の日付の file があること・PC の時計がずれていることを画面で知らせる新しい文（owner の判断事項 J2 の案 (b)）。
- バックアップ画面の一覧の並びと先頭の「最新」の印（68 UI-11b-D7。先の日付の file が先頭に出る。owner の判断事項 J1 の案 B）。
- 媒体の上に既にある先の日付の名前の写し（消さない。媒体の掃除は今のまま新しい名前の 30 本を残す）。
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

### 一時の test での再現

`src-tauri/src/mnt/offsite.rs` の `mod tests` に、既存の fixture（`Fx`・`put_backup`・`name_on`）と別の `now` の `Env` で 3 つの筋を流し、結果を `println!` する test を一時に足して流した（commit していない。同じ筋は Test Plan の T3・T1・T4 が回帰 test として持つ）。`now()` は fixture の固定値 2026-11-02 09:00:00。

測定の command（作業 clone の `src-tauri` で）: `CARGO_TARGET_DIR=<scratchpad> cargo test --offline --lib probe_s -- --nocapture --test-threads=1`。出力の末尾: `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1636 filtered out`、exit 0。

| 筋 | 入力 | 出力（原文） | 読み |
|---|---|---|---|
| S1 時計が先 → 直る | 時計が 30 日先（2026-12-02）のときに、その日付の控えを作って確認 → 時計を戻し、今日の控えを置いて確認 | `S1 first(clock ahead) = Ok(Copied { file_name: "inventory_backup_20261202_090000.db", labels: ["控え 1"] })`／`S1 second(clock fixed) = Ok(UpToDate)`／`S1 medium = ["inventory_backup_20261202_090000.db", "offsite-medium.json"]`／`S1 status(+0) days=Some(-30) stale=false local_backup_old=false`／`S1 status(+29) days=Some(-1) stale=false local_backup_old=false`／`S1 no medium(+10) days=Some(-20) stale=false attached=0` | 今日の控えは写らず、知らせも出ない。媒体が見えなくても出ない。PC の日付が先の日付になるまで続く |
| S2 時計は正しい・先の日付の file がある | 30 日先の日付の控えと今日の控えを置いて確認 2 回 → 翌日の控えを置いて翌日の時計で確認 | `S2 first = Ok(Copied { file_name: "inventory_backup_20261202_090000.db", labels: ["控え 1"] })`／`S2 second = Ok(UpToDate)`／`S2 next day = Ok(UpToDate)`／`S2 status(+2) days=Some(2) stale=false local_backup_old=false attached=1`／`S2 status(+3) days=Some(3) stale=true local_backup_old=false attached=1` | 先の日付の file が写り、今日と翌日の控えは写らない。3 日後に知らせは出るが、条件（`attached` あり・`local_backup_old` false・失敗の記録なし）は UI-11b-D14 の (5) の「USB メモリは差してありますが、写せません。…別の USB メモリを…用意してください」に当たり、直し方が外れる（文の選び方は F9 の file の読み） |
| S3 時計が戻る | 正しい時計で今日の控えを写す → 時計を 10 日戻し、その日付の控えを置いて確認 | `S3 first = Ok(Copied { file_name: "inventory_backup_20261102_080000.db", labels: ["控え 1"] })`／`S3 second(clock back) = Ok(UpToDate)`／`S3 medium = ["inventory_backup_20261102_080000.db", "offsite-medium.json"]`／`S3 status(back+0) days=Some(-10) stale=false local_backup_old=false` | 時計が戻っている間に作った控えは写らず、知らせも出ない |

未実測: Windows の実機と実際の PC の時計の変更での挙動、店の PC の時計がずれた実績。上の 3 つは WSL の unit test（一時 directory、注入した時計）での再現。

## owner の判断事項

確認済みの事実: 上の「調べ」のとおり、今の main では S1・S3 で写しと知らせの両方が止まり、S2 で写しが止まって 3 日後に直し方の外れた文が出る。下の J1・J2 は推奨（candidate。owner の答えの前）を既定にして 71 の proposed の文と本 packet を書いた。答えが違えば、計画の固定の前に 71 と packet を直す。

**J1 PC の中の先の日付の file をどう扱うか**

- 案 A（推奨、設計の既定）: そのままにする。アプリは消さず・名前を変えず、写す元と古さの判定に使わないだけ。file は今の掃除の規則（PC の日付がその日付 + 保持日数を過ぎた日）で消える。バックアップ画面の一覧には今どおり出る。
- 案 B: A に加え、一覧でその行に先の日付であることを出す（今は一覧が名前の新しい順で、先の日付の file が先頭に出て「最新」の印が付く。68 UI-11b-D7）。画面の文字列と 68 の変更、目視が要る。復元の前の確かめ（68 UI-11b-D15）は今も最後の記録の日時を出す。
- 案 C: アプリが先の日付の file を別の名前へ退避するか消す。破壊的で R4 になり、「PC の中の掃除は今のまま」（TD-673）の見直しが要る。時計が戻っているだけの間は、本当の日付の正しい backup を退避・削除することになる。
- 推奨の理由: 目的（写しが止まらない・知らせが出る）は A で満たせ、file を消さないので誤っても失うものが無い。一覧の「最新」の印は申し送りの外で見つけた別の症状（出所: 本起票の調べ〈AI〉。owner の確認の前の要件にしない）で、A なら backlog に残りとして 1 行置く。

**J2 先の日付の成功の記録をどう知らせるか**

- 案 (a)（推奨、設計の既定）: 記録を無いものとして扱い、今ある「まだ写せていません」の文で知らせる。画面の文字列は変えない。USB メモリが見えなければ「…USB メモリが差してあるか確かめてください。」が出て、差せば写って消える。残るもの: USB メモリが差してあり、今日までの最新が既に媒体にある間（`UpToDate`）は、次に新しい backup を写せるまで「USB メモリは差してありますが、写せません。…」の文が出る（直し方が外れる）。自動 backup が有効なら、今日の backup がまだ無い日は次の 60 秒の確認で backup が作られて写り、今日の backup が既に写っている日は次の backup（設定時刻か翌日）まで続く見込み（F6・F10 の code の読み。未実測）。公開の後の sync の失敗で記録が古いまま `UpToDate` になる今の場合と同じ扱い（71 §71.11.4 のエラーの表）。
- 案 (b): (a) に加え、原因を名指す文を 1 つ足す（例「最後に PC の外へ写した記録の日付が、今日より先です。PC の日付と時刻を確かめ、USB メモリを差して「今すぐバックアップを作成」を押してください。」。文は owner が決める）。68 UI-11b-D14・53 UI-00-D12・`offsite-status-text.ts` と目視が要る。(a) の返す形では画面が「一度も写せていない」と見分けられないので、`OffsiteBackupStatus` に原因の材料を 1 つ足す（DTO と bindings、43 §43.8.4 の変更）。時計が今ずれているのか、前にずれていた跡なのかをアプリは見分けられないので、文は両方に当たる形になる。
- 案 (c): 知らせを足さない（写す元だけ直す）。USB メモリが差してあれば次の写しで記録が直るが、見えないままだと PC の日付が記録の日付 + 3 日になるまで知らせが出ない（S1 の `no medium` の行と同じ）。
- 推奨の理由: (c) は「黙って止まる」が USB メモリの見えない場合に残る。(a) は画面と DTO を変えずに知らせる側へ倒せ、残る外れた文は今の設計が既に受けている場合と同じ扱い。(b) は原因を名指せるが、時計の変更が要る状態の文で、目視の手順が重い。

## Scope

runtime（計画の固定の後）:

- `src-tauri/src/mnt/offsite.rs`:
  - private の関数を 1 つ足す: `list_backups` の結果（新しい順）と今日の日付を受け、名前の日付（`extract_date_from_backup`）が今日以前の最初の 1 本を返す。
  - `check_body` の手順 2 の写す元を、その関数（今日 = `env.now.date()`）で選ぶ。無ければ今どおり `NoLocalBackup`。`list_backups` の `Err` の扱いは変えない。
  - `status_with` の `local_backup_old` を同じ関数（`today`）で決める。`list_backups` の `Err` は今どおり false。
  - `status_with` は、写した日と控えの日付の古い方が `today` より後の成功の記録を無いものとして扱う（`last_success_at`・`last_success_label`・`days_since_last_success` が `None`、`prepared` なら `stale = true`）。状態の file は読むだけ。
  - 意図した簡略化は `ponytail:` comment に書く（日付で比べ時刻を見ない。広げる条件は MNT-01-D12 の見直し契機）。
  - test（下の Test Plan の T1〜T6）。`mod tests` に別の `now` の `Env` を作る小さい helper を 1 つ足してよい。
- `docs/function-design/71-mnt-backup.md`: MNT-01-D12 の見出しの proposed の注記を外す。実装で文と食い違った所があれば同じ commit で直す。
- `docs/function-design/90-traceability.md`: generator で作り直す（`cd src-tauri && cargo run --bin generate_traceability`。手で編集しない）。
- `docs/backlog.md`: 「backup に、PC の外のコピーと復元の実証が運用として設計されていない」の項目で、この lane が済ませる「(b)（PR #194）の申し送り」の行を外し、「導入の前の関門」の段の「残り:」から「先の日付の backup の件（下の「(b)（PR #194）の申し送り」）」を外す（同じ PR で行い、merge の後の宿題にしない）。J1 が案 A なら、一覧の「最新」の印の件を残りとして 1 行置く（出所: 本起票の調べ〈AI〉）。

起票の commit: 本 packet と、71 の §71.10 の表の MNT-01-D12 の行・§71.11.4 手順 2・§71.11.5 の規則と `local_backup_old`・§71.11.7 の箇条・MNT-01-D12（proposed）。

path の所有（第 1 波の他の lane との突き合わせ）: `src-tauri/src/mnt/offsite.rs` と 71 の §71.10 の表の MNT-01-D12 の行・§71.11.4 手順 2・§71.11.5・§71.11.7・MNT-01-D12 は本 lane。`docs/backlog.md` は上の項目の 2 か所だけ。`docs/function-design/90-traceability.md` は生成物で、同期で衝突したら取り込んだ後に作り直す。

Non-scope: `src-tauri/src/mnt/backup.rs`（`list_backups`・`cleanup_old_backups`・`check_auto_backup`・`extract_date_from_backup` は呼ぶだけ）、`src-tauri/src/cmd/**`、`src/**`（画面・`offsite-status-text.ts`・bindings）、68・53・43 の文、`docs/decision-log.md`、`Plans.md`、migration、状態の file の形。

## Acceptance Criteria

- AC1: 先の日付（今日 + 30 日）の正式名と今日の日付の正式名が PC の中にあり、用意済みの媒体が見えているとき、確認が `Copied { file_name: 今日の file }` を返し、媒体に今日の file があって先の日付の file が無く、PC の中の先の日付の file は残る。次の確認は `UpToDate`。境界: 名前の日付が今日の file は写す元になり、明日の file はならない。用意（手順 6）の最初の写しも同じ file を選ぶ。
- AC2: PC の中が先の日付の file だけのとき、確認は `NoLocalBackup` を返し、媒体の一覧と状態の file の bytes が前後で同じ。
- AC3（時計が先 → 直る）: 時計が 30 日先のときに写した後、今日の日付の `offsite_status` は `last_success_at = None`・`days_since_last_success = None`・`stale = true` を返す（媒体が見えていても見えなくても）。今日の控えを置いて確認すると `Copied`（今日の file）で、`offsite_status` は `days_since_last_success = Some(0)`・`stale = false`、状態の file の成功の記録は今日の file。媒体が見えない間の確認は `MediumMissing` で状態を変えない。
- AC4（時計が戻る）: 正しい時計で写した後に時計を 10 日戻すと、その日付の `offsite_status` は `stale = true`。その日付の控えを置いて確認すると `Copied`（その日付の file）で、媒体に前の写しと今写した file の両方があり、`stale = false`。
- AC5: `local_backup_old` は、先の日付の file と 10 日前の file がある dir で `true`、先の日付の file と 2 日前の file で `false`、先の日付の file だけで `true`。
- AC6: 成功の記録の日数は、写した日が 5 日先・控えの日付が 1 日前なら 1、写した日が 1 日前・控えの日付が 5 日先なら 1（どちらも今どおり古い方から数え、`stale = false`）。両方が 1 日先なら記録なしの扱い（`stale = true`）、両方が今日なら 0。`offsite_status` の前後で状態の file の bytes が同じ。
- AC7: 既存の test が期待値を変えずに全部 PASS する（`mnt::offsite`・`mnt::backup`・`cmd::settings_cmd`。特に `test_offsite_status_req901_d8_local_backup_old`・`test_offsite_status_req901_d8_stale_boundary_three_days`・`test_offsite_req901_d8_stale_clears_only_after_new_copy`・`test_check_offsite_req901_d8_retention_keeps_newest_and_foreign_files`）。`git diff --stat origin/main...HEAD` に `src/**`・`src-tauri/src/cmd/**`・`src-tauri/src/mnt/backup.rs` が出ない。
- AC8: mutation を実装に注入し、当たる test が red になることを確かめて PR に書く: (m1) 写す元を一覧の先頭に戻す → T1・T3・T4、(m2) 日付の比較を「今日より前」にする → T1 の境界、(m3) `local_backup_old` を一覧の先頭に戻す → T5、(m4) 先の日付の記録の扱いを外す → T3・T6、(m5) 「両方が先」を「どちらかが先」にする → T6。
- AC9: `rg -n "proposed" docs/function-design/71-mnt-backup.md` に MNT-01-D12 の行が出ない。`rg -n "\(b\)（PR #194）の申し送り" docs/backlog.md` が 0 件。
- AC10: `bash scripts/doc-consistency-check.sh` と `--target plan docs/plans/2026-11-19-offsite-future-dated-backup.md` が ERROR 0、`bash scripts/check-workflow-git.sh` が PASS、`cd src-tauri && cargo run --bin generate_traceability -- --check` が PASS。Rust の検証は `docs/DEV_WORKFLOW.md` の Verification Gates（hosted CI の `Merge gate`）。

## Design Readiness

- 引用する設計正本（節まで）: 71 §71.3（名前の規約）・§71.5（MNT-01-D3）・§71.6・§71.8・§71.11.4（手順 2・3a・4a・4g、エラーの表）・§71.11.5・§71.11.7・MNT-01-D8・MNT-01-D11・MNT-01-D12、68 UI-11b-D7・D14・D16、53 UI-00-D12、43 §43.8.4、D-114、D-127。
- 必要な設計成果物: updated in this PR（71 の proposed の文は起票の commit。runtime で注記を外す）。68・53・43 は existing sufficient（`offsite_status` が返す形は今の「一度も写せていない」と同じで、画面の文の選び方を変えない）。DB・migration は不要。
- plan にしかない durable な判断の昇格先: なし（MNT-01-D12 に置いた。J1・J2 の答えは、答えの後に MNT-01-D12 の Why と Rejected alternatives に TD 番号で足す）。
- 前提・制約と延期した design gap: J1・J2 は owner の答えの前（推奨を既定にした）。一覧の「最新」の印が先の日付の file に付く件は非目的（J1）。Windows の実機での確かめは置かない: 状態を作るには PC の時計の変更か file の名前の書換えが要り、L3 Eligibility の条件 (3)（故障注入の手順）に当たるので、注入した時計の自動 test で持つ。画面の file と文字列を変えないので L3 の節は置かない。
- 絶対保証（cannot happen / always happens）の例外と抜け道:
  - 「今日までの最新が写る」は名前の日付で比べる。同じ日の中で時計が数時間先へ進んでから直ると、その日の残りの backup は名前の時刻が前なので写らず、翌日の backup から写る（1 日以内。店主の許容は 3 日、L-220）。
  - 先の日付の file は、PC の日付がその日付になった日に写す元に戻る。同じ日の backup より名前の時刻が後なら、その日は古い中身のその file が最新として写る（次の backup で直る）。
  - 先の日付の記録で `UpToDate` の間は、次に新しい backup を写せるまで知らせが出続ける（J2 の案 (a) の残るもの）。
  - PC の中の保存先が媒体の `InventoryBackup` と同じ folder（MNT-01-D11）で、そこが先の日付の file だけのとき、確認は手順 2 の `NoLocalBackup` で終わり手順 3 の `BackupDirIsMediumFolder` を記録しない（手順の順は今のまま。空の folder と同じ）。`local_backup_old` の文が導く「今すぐバックアップを作成」の後の確認で記録される。
  - 「消さない・書き換えない」は、`check_body` と `status_with` に削除・改名・状態の書込みを足さないことに依る（T2・T6 が前後の bytes で確かめる）。
- 判定: ready（J1・J2 の答えが推奨と違えば、計画の固定の前に直す）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| MNT-01-D12 (1) 写す元 | 71 §71.11.4 手順 2 | `offsite.rs` の新しい関数と `check_body` | T1・T2・T3・T4 | 非対象（L3 なし。Design Readiness） |
| MNT-01-D12 (1) `local_backup_old` | 71 §71.11.5 | `offsite.rs` の `status_with` | T5 | 非対象 |
| MNT-01-D12 (2) 先の日付の成功の記録 | 71 §71.11.5 | `offsite.rs` の `status_with` | T3・T4・T6 | 非対象 |
| MNT-01-D12 (3) 消さない・書き換えない | 71 §71.11.7 | 足さない | T1（PC の中の file が残る）・T2・T6（前後の bytes） | 非対象 |
| MNT-01-D8（手順 3a〜6、`stale` の 3 日の境界、日数は古い方から） | 71 §71.11.4・§71.11.5 | 変えない | 既存 `test_check_offsite_req901_d8_up_to_date_skips_copy`・`test_offsite_status_req901_d8_stale_boundary_three_days`・`test_offsite_req901_d8_stale_clears_only_after_new_copy`、T6 | 非対象 |
| MNT-01-D8 手順 4g（媒体の掃除。今写した file を消さない） | 71 §71.11.4 | 変えない | 既存 `test_check_offsite_req901_d8_retention_keeps_newest_and_foreign_files`（case 2 が、媒体に今写す file より新しい名前の写しが 30 本ある場合） | 非対象 |
| MNT-01-D8 用意の手順 6・作り直しの手順 7 | 71 §71.11.2・§71.11.5a | 変えない（同じ `check_body` を通る） | T1 の用意の case | 非対象 |
| MNT-01-D11（手順 3 の同じ folder の判定） | 71 §71.11.4 手順 3 | 変えない（手順の順も変えない） | 既存 `test_check_offsite_req901_d11_backup_dir_same_folder_writes_nothing` | 非対象 |
| MNT-01-D3・§71.5（PC の中の掃除） | 71 §71.5 | 変えない | 既存 `test_cleanup_old_backups_req901_deletes_expired`・`test_cleanup_old_backups_req901_keeps_recent` | 非対象 |
| §71.6 `list_backups`（一覧の command も使う） | 71 §71.6 | 変えない | 既存 `test_list_backups_req901_returns_sorted` | 非対象 |
| §71.8 今日の backup の判定 | 71 §71.8 | 変えない | 既存 `test_check_auto_backup_req901_no_backup_today` | 非対象 |
| 43 §43.8.4（`today` は今日のローカル日付） | 43 §43.8.4 | 変えない | 非対象（CMD は渡すだけ） | 非対象 |
| UI-11b-D14・UI-00-D12（原因の文の選び方） | 68 §68.5、53 | 変えない（返す形は「一度も写せていない」と同じ） | 既存 `offsite-status-text.test.ts` の「用意の前・写せた後・まだ写せていない」「(2) PC の中の backup が古い（(3) より先）」「(5) 差してあるかで 2 つの文（札を名指さない）」 | 非対象 |
| UI-11b-D7（一覧の並びと「最新」の印） | 68 §68.5 | 変えない | 非対象 | 非対象（J1 の案 B） |
| D-114（Guarantee range） | decision-log | 変えない（正本は 71） | 非対象 | 非対象 |

## Test Plan

Rust の unit test（`src-tauri/src/mnt/offsite.rs` の `mod tests`。一時 directory と合成の控え、注入した時計）。test の先頭の comment に `REQ-901 / MNT-01-D12` と節を書く。

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| MNT-01-D12 (1) 写す元 | 先の日付の file が写り、今日の file が写らない（S2）。今日の file を外す | unit（T1） | `test_check_offsite_req901_d12_skips_future_dated_source` | 写す元を一覧の先頭に戻す（m1）、比較を「今日より前」にする（m2。今日の file が `NoLocalBackup` になる）、用意の手順 6 が別の選び方をする |
| MNT-01-D12 (1)・(3) | 先の日付の file だけのときに写す・失敗を記録する・何かを消す | unit（T2） | `test_check_offsite_req901_d12_only_future_dated_is_no_local_backup` | 無いときに先の日付の file へ戻す fallback を足す、`last_failure` を書く |
| MNT-01-D12 (1)・(2)（時計が先 → 直る、S1） | 今日の控えが写らない。先の日付の記録で知らせが出ない。写した後も知らせが残る | unit（T3、状態の遷移） | `test_offsite_req901_d12_clock_ahead_then_fixed_copies_and_clears_stale` | m1、先の日付の記録の扱いを外す（m4）、媒体が見えないときだけ別の判定にする |
| MNT-01-D12 (1)・(2)（時計が戻る、S3） | 戻っている間の控えが写らない・知らせが出ない | unit（T4、状態の遷移） | `test_offsite_req901_d12_clock_set_back_copies_backdated_backup` | m1、m4 |
| MNT-01-D12 (1) `local_backup_old` | 先の日付の file があると PC の中の backup が新しいと数える | unit（T5、表） | `test_offsite_status_req901_d12_local_backup_old_ignores_future_dated` | `local_backup_old` を一覧の先頭に戻す（m3） |
| MNT-01-D12 (2)、MNT-01-D8 の日数 | 片方だけ先の記録を記録なしにする・日数が負になる。`offsite_status` が状態を書く | unit（T6、表） | `test_offsite_status_req901_d12_future_record_is_treated_as_absent` | 「両方が先」を「どちらかが先」にする（m5）、m4、状態の file を書き直す |
| MNT-01-D8・D11・D3・§71.6・§71.8 | 既存の振舞いが変わる | unit（既存） | Contract Ledger の既存の test | 期待値を変えないと通らない変更を入れる |

状態の遷移（T3・T4 が持つ）: initial（先の日付の記録だけ → `stale`）、failure（媒体が見えない確認 → `MediumMissing`、状態を変えず `stale` のまま）、retry・success（媒体が見える確認 → `Copied`、記録が置き換わる）、refetch（`offsite_status` → `stale = false`）、revisit（次の確認 → `UpToDate`、状態を変えない）、restart（時計が直った後の最初の `offsite_status` と確認の順。process の状態を持たないので、同じ file の状態から始める T3 がそのまま持つ）。negative は T2・T6。mutation は AC8 の m1〜m5 を実装に注入して red を確かめる。
