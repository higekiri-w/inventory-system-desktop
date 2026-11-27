# Plan Packet: restore の後に古い SD で照合して、レジで使われている PLU の枠に重ねて書く道を塞ぐ

起票: 2026-11-27。出所は PR #307（PLU の書出しの見直しの runtime、R4）の Final（Codex 1174、PR #307 の issuecomment-7220169252）が合成で再現した経路と、それを直すという owner の決定（repo 外の回答台帳 TD-948）。#307 の中で直すと Amendment と fresh broad の Final 2 本が要るので、#307 の直後の別の lane にした。実装は #307 の merge と、この branch の `origin/main` への同期の後に始める（Scope）。

## Workflow State

- Risk: R4
- Plan Commit: pending
- Amendments: none

`Plan Commit`・`Amendments` は `scripts/plan-approve.sh` が書く（手で写さない）。

## Risk

Risk: R4

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の R4 の「backup の復元」に当たる: restore（71 §71.7 `restore_backup`）のステップ 6a に、復元の前の DB から値を読み、復元した DB へ書く手順を足し、その書込みの失敗を復元の失敗（元の DB への巻き戻し）にする。守る対象は、レジに書いたら DB を戻しても戻らない PLU の枠（D-168 の Why (1) と同じ物差し。先例は #307 の packet の Risk の (2)）。

## Goal

Goal Invariant:

### 最小完了条件

- 同じ PC で backup を restore した後、restore の前に最後に照合した精算回数より小さい精算回数の SD の Z004 では、レジの占有を照合しない。照合しなかったことと理由は、SD を読んだその場（売上データ取込みの読み終えた画面）に今の知らせ（55 UI-07-D15 (8)）で出る。
- PR #307 の Final の経路（精算 40 で backup → 45 で 217 番の占有を照合 → 40 へ restore → 217 番が空の精算 42 の SD を読む）で、217 番が空きとして予約されない。精算 45 より新しい SD を読めば照合し、書出しが戻る。
- 店の操作は増えない（restore の操作も SD を読む操作も今のまま）。

### 失敗定義

- restore の後に、restore の前の最後の照合より小さい精算回数の Z004 で照合が成り立つ（`plu_slots` か `plu_register_snapshot_at` が変わる）。
- 精算回数を引き継ぐ書込みが失敗したのに、復元した DB の接続が公開される。
- 引き継ぎで、復元した DB の精算回数が backup の値より小さくなる。
- 引き継ぎの値の読取りの失敗で、店が restore できなくなる（Design Readiness の DR-2 で owner が「復元を失敗にする」を選んだときを除く）。
- restore の前と後で、PLU 以外の DB の内容・backup・PC の外の控え（`offsite-backup.json`）・restore の manifest の契約（MNT-01-D1・D4・D5）が変わる。

### 非目的

- レジの精算回数が戻ったとき（初期化・修理）の照合の脱出（D-117 の Revisit のまま）。
- 同じ精算回数の SD で restore の後の照合が止まること（D-168 の Guarantee range の「同じ日の backup を restore した後」）の解消。この lane の引き継ぎで起きる場面が増えるので、扱いは DR-3 で owner が決める。DR-3 で知らせか照合を選んだときだけ Scope に入れる。
- PC を失って新しい PC に restore するとき（71 §71.13 の本番の復元）に、古い PC の最後の照合を新しい PC へ運ぶこと（DR-4。既定は運ばない）。
- CV17・SR-S4000 の実機の L3（行の形を変えない。D-168 (7)）。
- #307 の Scope（照合の規則・gate・migration・Diff の群・confirm の照合）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

着手の条件: PR #307 が main に merge され、この branch を `origin/main` に単段で同期した後に実装を始める。#307 の Final の是正で 71 §71.7 ステップ 6a・33 §16.8・D-168 の文が変わっていれば、下の S1〜S3 をその版に当て直す（変わり方が塞ぐ形に触れれば計画の review をやり直す）。

塞ぐ形: restore で失われるのは「restore の前に最後に照合した精算回数」で、その値は restore の直前まで今の DB の `app_settings` の key `plu_register_snapshot_settlement_no` にある。restore がその値を復元の前の DB から読み、復元した DB の同じ key へ、backup の値と比べて大きい方を書く（ステップ 6a の照合日時の削除と同じ位置）。新しい置き場を作らない（比べた候補は Design Readiness）。照合の規則（33 §16.3 手順 2）は変えない: 引き継いだ値より小さい SD は今の `SkippedOlder` になり、今の知らせが出る。

- S1 restore（71 §71.7 ステップ 2〜3 と 6a、MNT-01-D13）: `src-tauri/src/mnt/restore.rs` の `restore_backup_with_ops`。
  - `drop(current_conn)` の前に、今の接続で key `plu_register_snapshot_settlement_no` を読む（無ければ引き継ぐ値は無い）。
  - ステップ 6a の `install_result` の中、`plu_register_snapshot_at` の削除の後に、引き継ぐ値があれば、復元した DB の同じ key を `max(backup の値, 引き継ぐ値)` にする（backup の値が無い・読めないなら引き継ぐ値）。引き継ぐ値が無ければ書かない。書込みの失敗は照合日時の削除の失敗と同じく `install_result` の `Err` にし、ステップ 8 の巻き戻しで `RestoreError::Recovered`（復元した DB の接続を公開しない）。
  - 引き継ぐ値が読めない（文字列が整数として読めない）・読むときに SQL の error が出たときの扱いは DR-2 の owner の選択に従う（推奨は下）。
  - 起動時の reconcile（`reconcile_restore_with_ops`）と `cleanup_committed` には足さない（引き継ぎは `committed` の前に復元した DB に入っているので、中断しても元の DB か引き継ぎ済みの DB のどちらかが残る。MNT-01-D5 の不変条件のまま）。
  - test は同じ file の test module（今の `test_restore_backup_req907_clears_snapshot_at_keeps_settlement_no` の隣。この module は既に `crate::biz::plu_export_service` を test から呼んでいる）。
- S2 精算回数の読み方を 1 か所に（D-147 の 4）: 今は BIZ-04 の private な `parse_settlement_no` と key 名の定数（`SNAPSHOT_AT_KEY`・`SNAPSHOT_SETTLEMENT_NO_KEY`）が `src-tauri/src/biz/plu_export_service.rs` にあり、`src-tauri/src/mnt/restore.rs` は key 名を文字列で持つ。MNT は BIZ を呼ばない（`UI -> CMD -> BIZ -> IO/MNT`）ので、読み方と 2 つの key 名を `src-tauri/src/db/plu_slot_repo.rs` へ移し、BIZ-04 と restore の両方がそこを使う（読み方は今の規則のまま: ASCII の数字だけ、`i64` に収まる、前の 0 を許す）。`rg -n 'plu_register_snapshot_(at|settlement_no)|parse_settlement_no' src-tauri/src` で production の呼出し側を全部当てる（test の文字列はそのままでよい）。
- S3 設計正本（実装の PR で書き換える。計画の段では書かない）:
  - `docs/function-design/71-mnt-backup.md` §71.7 のステップ 6a と MNT-01-D13（決定・Why・Rejected alternatives・Guarantee range）、§71.10 の MNT-01-D13 の行。
  - `docs/function-design/33-biz-plu-export-service.md` §16.8 の「DB の restore の後」（「受け入れる限界」の文を引き継ぎに置き換える）と §16.3 の「照合済みの記録」の段落（restore が引き継ぐことを 1 文）。
  - `docs/db-design/plu-tables.md` の key `plu_register_snapshot_settlement_no` の行（restore が復元の前の値と比べて大きい方を残す）。
  - `docs/decision-log.md` に D-176（予約済み）を足し、D-160 (7) の Guarantee range・Why と D-168 の Guarantee range の「D-160 (7) の穴。直さずに受け入れる」に `Superseded in part by: D-176` を付ける。
  - DR-4 で手順の案内を選んだときだけ `docs/function-design/71-mnt-backup.md` §71.13 の本番の復元の行。
  - traceability: 新しい test が REQ-907 を使うなら `cd src-tauri && cargo run --bin generate_traceability` で `docs/function-design/90-traceability.md` を作り直す。
- DR-3 で (b)・(c) を選んだときだけ入る（今は Scope の外。選ばれたら計画の固定の前にこの packet を直して計画の review を受ける）: (b) は `src-tauri/src/biz/plu_export_service.rs` の `PluRegisterSnapshotRefresh`、`src-tauri/src/cmd/` の scan の response、`src/lib/bindings.ts`、`src/features/daily-report-import/` の知らせ、45 §45.6a 手順 2a、55 UI-07-D15 (8)。(c) は S1 に引き継ぐ値を 1 つ足す。

Non-scope: `src-tauri/src/lib.rs`・`src-tauri/src/db/mod.rs`・`src-tauri/src/db/migration.rs`（#311 の範囲。この lane は migration を足さない）、`offsite-backup.json` と PC の外の控え（§71.11）、22 の migration の手順、67 の UI-08。

## Acceptance Criteria

- AC1（S1）: 今の DB の key が `45`、backup の key が `40` のとき、`restore_backup` の後の接続で key `plu_register_snapshot_settlement_no` が `45`、`plu_register_snapshot_at` が無く、`prepare_plu_export` が `register_snapshot_required` で拒まれる。PLU 以外の内容は backup のまま（今の test の supplier の確かめと同じ形）。
- AC2（S1）: backup の key が今の DB より大きい（backup `50`・今 `45`）なら `50` のまま。今の DB に key が無ければ backup の値のまま。どちらにも無ければ key は無いまま。
- AC3（S1・DR-2）: 今の DB の key が整数として読めない・読むときに SQL の error が出るときの結果が DR-2 で owner が選んだ扱いのとおり（推奨の A なら: 読めない文字列はそのまま復元した DB へ書き、照合は今の「key が読めない」の規則で止まる。SQL の error なら backup の値のまま復元を終え、`tracing::warn!` を残す）。
- AC4（S1）: 復元した DB の中の trigger が key `plu_register_snapshot_settlement_no` の `INSERT`・`UPDATE` を拒むとき、`restore_backup` が `RestoreError::Recovered` を返し、元の DB に戻り（key が `45` のまま）、restore の遺物（manifest・`.restore_backup`）が残らない。
- AC5（S1・S2、Goal の #307 の経路）: 精算 40 で作った backup、精算 45 で 217 番が埋まった Z004 で照合した今の DB から restore し、217 番が空の精算 42 の Z004 で `refresh_plu_register_snapshot_from_sd` を呼ぶと `SkippedOlder` が返り、`plu_slots`・`plu_register_snapshot_at` が変わらず、`prepare_plu_export` が `register_snapshot_required` で拒まれる。続けて 217 番が埋まった精算 46 の Z004 で `Refreshed` になり、217 番が `external`、次の `prepare_plu_export` が 217 番を予約しない。
- AC6（S2）: BIZ-04 の照合の既存の test（33 §16.3 の手順 2 の等しい・小さい・読めない・無い）と restore の既存の test（`test_restore_backup_req907_clears_snapshot_at_keeps_settlement_no`）が期待値を変えずに通る。`rg -n '"plu_register_snapshot_(at|settlement_no)"' src-tauri/src` の hit のうち `#[cfg(test)]` の module の外のものが `src-tauri/src/db/plu_slot_repo.rs` の定数だけ（hit ごとに実読で確かめる）。
- AC7（mutation）: S1 の引き継ぎの書込みを外すと AC1・AC5 の test が red、`max` を「引き継ぐ値で上書き」に変えると AC2 の test が red、書込みの error を握りつぶすと AC4 の test が red（`cd src-tauri && cargo test --lib -- <test 名>` が exit 101）。戻すと exit 0。
- AC8（S3、置き換えの検査）: 33 §16.8 と 71 の MNT-01-D13 の今の文に「restore の前の最後の照合より小さい精算回数の SD は、restore の後に照合しうる」の形の受け入れの文が無く（`rg -n '最後の照合より小さい' docs/function-design/33-biz-plu-export-service.md docs/function-design/71-mnt-backup.md` が、引き継ぎを書いた文と D-176 を引く文の外で 0 件）、D-160 と D-168 に `Superseded in part by: D-176` の行がある。
- AC9（gate）: `docs/DEV_WORKFLOW.md` Verification Gates の Rust・traceability・docs の command が通り、`bash scripts/doc-consistency-check.sh` が ERROR 0。bindings の diff が無い（DR-3 で (b) を選んだときは生成の diff が `plu_snapshot_*` の 1 つだけ）。

## Design Readiness

- 引用する設計正本（節まで）: 71 §71.7（ステップ 2〜3・6a・7・8、MNT-01-D1・D4・D5・D13）と §71.10・§71.13、33 §16.3（手順 2〜4 と照合済みの記録の段落）・§16.7・§16.8、`docs/db-design/plu-tables.md` の `app_settings` の key の表、D-117 の Revisit、D-160 (4)(7)、D-168 の (3) と Guarantee range、55 UI-07-D15 (8)、45 §45.6a 手順 2a。いずれも #307 の branch の版で読んだ（#307 の merge の後に S3 を当て直す）。
- 必要な設計成果物: updated in the implementation PR（S3。71 §71.7 ステップ 6a・MNT-01-D13・§71.10、33 §16.3・§16.8、plu-tables、D-176）。55・45・67 は existing sufficient（今の `SkippedOlder` の知らせの文「この SD の精算回数が、前にレジの登録状況を読んだときより小さいためです。前に使った古い SD を読んだのでなければ、管理者に伝えてください。」は、restore の前に読んだ照合にもそのまま当たる）。DR-3 で (b) を選べば 55・45 を updated にする。
- plan にしかない durable な判断の昇格先: 置き場の比べと引き継ぎの規則（max・失敗は巻き戻し・読めないときの扱い）は D-176 と MNT-01-D13 へ。D-160 (7) の「restore の直後に古い SD を止めるのは backup の時点の値だけ」と D-168 の Guarantee range の受け入れ（TD-948 で owner が直すと決めた）は D-176 が置き換える。
- 置き場の比べ（D-147。既にある仕組みを先に使う）:

  | 候補 | 何が要るか | restore で戻らないか | 引越し（#312） | 採否 |
  |---|---|---|---|---|
  | 1 restore が復元の前の DB から読み、復元した DB へ大きい方を書く（採る） | ステップ 6a に読みと書きを 1 つずつ。新しい file・schema・migration は無い | 戻らない（`committed` の前に復元した DB に入る。失敗は今の巻き戻し） | DB の中なので、DB と一緒に運ばれる。「DB の外の物」の一覧は増えない | 採る |
  | 2 app data の file（`offsite-backup.json` と同じ置き場の新しい file） | 照合のたびに DB の TX の外で file を書く（DB と file の 2 か所の書込みで、片方だけ成功する形が増える）。無い・壊れた・書けないの扱い、§71.11.5a のような作り直しの道 | 戻らない | #312 の「DB の外の物」に入り、引越しで写し忘れると値が無い | 採らない（候補 1 で足り、壊れ方が増える） |
  | 3 backup の manifest・backup の file の metadata | backup は SQLite の写しで、backup ごとの manifest は無い（`{db_path}.restore_manifest` は復元の途中だけの file）。値は backup の時点のもので、後の照合を持たない | — | — | 採らない（失われる値が backup の後に生まれる） |
  | 4 `offsite-backup.json`・PC の外の媒体の目印 | 写すのは媒体を差している時だけ。§71.11 の状態の file の形（`format`）を変える | 戻らない | 媒体で運べる（PC を失っても残る） | 既定では採らない。DR-4 の (c) の候補 |

- 前提・制約と延期した design gap:
  - 引き継ぐ値は、今の DB で key が単調にしか増えないこと（33 §16.3 手順 2 は大きいときだけ更新する）に依る。例外は開発者が key を消すとき（D-117 の Revisit）と、別の DB の系列の backup を戻すとき。どちらも `max` で backup の値より下げない。
  - 同じ PC の restore だけを塞ぐ。新しい PC への restore（71 §71.13 の本番の復元）は今の DB が空なので引き継ぐ値が無く、backup の後・PC を失う前の照合は失われる（DR-4）。
  - Contract Probe P1（`未実測`）: 毎日の自動の backup（D-162 の定時）が、その日の締めの SD の読取りの後に作られるか。後なら同じ日の backup の key はその日の照合を持ち、候補 1 が無くても穴は backup の後に読んだ日の分だけになる。前なら毎日 1 日分の差がある。どちらでも候補 1 の正しさは変わらず、DR-3・DR-4 の重みだけが変わる。確かめ方: 店の PC の操作ログ（`plu_register_snapshot_import` と backup の記録）の時刻の並びを owner に見てもらう（店の実データなので repo に写さない）。
  - Contract Probe P2（`未実測`）: 精算 45 の照合の後に、旧い DB で prepare・confirm して CV17 でレジに書く運用がどのくらいあるか（DR-3 の (c) が安全になる前提）。
- 絶対保証（cannot happen / always happens）の例外と抜け道: 「同じ PC の restore の後、restore の前の最後の照合より小さい SD では照合しない」の抜け道は、(i) DR-2 の A で引き継ぐ値を読めなかったとき（今の DB の `app_settings` が読めない。そのとき D-168 の範囲の穴が残る）、(ii) 新しい PC への restore（DR-4）、(iii) 開発者が key を消した後（D-117 の Revisit）。(i)(ii) は D-176 の Guarantee range に書く。
- DR-1（owner に決めてもらう事項。D-160 (7) の受け入れを改める）: TD-948 で直すと決まったので、D-160 (7) の Guarantee range「restore の直後に古い SD を止めるのは backup の時点の精算回数だけ」と D-168 の Guarantee range の受け入れを D-176 で置き換える。選択肢は (a) 置き換える（候補 1 で塞ぐ）、(b) 置き換えず受け入れのまま。推奨: (a)、confirmed（TD-948 の決定そのもの。確認だけ）。
- DR-2（owner に決めてもらう事項。引き継ぐ値が読めないとき）: 今の DB の key が整数として読めない、または読むときに SQL の error が出るとき。選択肢:
  - A 読めない文字列はそのまま引き継いで照合を今の規則（key が読めなければ照合しない）で止め、SQL の error なら backup の値のまま復元を終えて記録だけ残す。
  - B どちらでも復元を失敗にして元の DB へ戻す。
  - C どちらでも、復元した DB の key を読めない値にして照合を止める（開発者が key を直すまで書出しは止まる）。
  - 推奨: A、candidate。読めない文字列は restore の前から照合が止まっていた状態なので、そのまま止めるのが安全側で、restore で解かない。SQL の error は今の DB が壊れているときで、restore はその直し方なので、PLU の値のために復元を止めない（B は使い手が DB を直せなくなり D-127 に反する。C は直し方を画面に出せず開発者を要る）。A の SQL の error の場合だけ D-168 の範囲の穴が残る。
- DR-3（owner に決めてもらう事項。同じ精算回数の SD で止まること）: 引き継ぐと、restore の前に最後に読んだ SD（多くはレジに差している今の SD）と同じ精算回数の SD は照合しない（33 §16.3 手順 2 の「等しければ照合しない」）。照合日時は消えているので、次の締めの SD を読むまで書出しが `register_snapshot_required` で止まり、SD を読んだ画面に知らせは出ず、PLU 書出しの画面は「売上データ取込みでレジの SD を一度読んでください」と案内する（読んでも戻らない）。今の D-168 でも同じ日の backup を restore した後に起きるが、この lane で restore の後のほとんどで起きるようになる。選択肢:
  - (a) 受け入れる。D-176 の Guarantee range に書き、次の締めの後に SD を読めば戻る。
  - (b) SD を読んだ画面で知らせる（`PluRegisterSnapshotRefresh` に値を 1 つ足し、scan の response と 55 UI-07-D15 に文を足す。例「この SD は、前にレジの登録状況を読んだときと同じ精算のものです。次の締めの後の SD を読むと、PLU の書出しができるようになります」）。
  - (c) restore の前の DB で最後の照合の後に PLU の書出し（prepare）が無かったときだけ、等しい SD でも照合する（引き継ぐ値を 1 つ増やす。最後の照合の後に書き出してレジに書いた枠は、その精算の Z004 に無いので、書出しがあったときは照合しない）。
  - 推奨: (b)、precondition-dependent（P1 で同じ日の SD の後に backup ができているなら、restore の後の止まりはその日の残りだけで (a) でもよい）。(b) は止まったことと直し方をその場で名指す（D-127）。(c) は書出しの有無の判定を間違えると重ねて書く道に戻るので、P2 の実測の前には推奨しない。(b)・(c) を選ぶと UI の文が変わるので `## L3` を足すかを計画の review で決める。
- DR-4（owner に決めてもらう事項。新しい PC への restore）: PC を失って新しい PC に restore するとき（71 §71.13 の本番の復元）、古い PC の最後の照合は失われ、backup の時点より新しく最後の照合より古い SD で照合しうる。選択肢:
  - (a) 受け入れ、D-176 の Guarantee range に書く（照合は毎日の締めでレジに差している今の SD を読むので、古い SD を残していなければ起きない）。
  - (b) (a) に加え、71 §71.13 の本番の復元の手順に「戻した後は、レジに差してある SD を次の締めの後に読む。手元の古い SD を読まない」と 1 文足す。
  - (c) 最後の照合の精算回数を PC の外の媒体にも書く（候補 4。§71.11 の状態の file か媒体の目印の形を変える）。
  - 推奨: (a)、candidate。PC を失い、かつ古い SD を読むときだけで、普段の締めの流れでは起きない。(b) は店の手順を 1 つ増やし、(c) は PC の外の控えの形を変える R4 で、塞ぐ範囲に見合わない。
- 判定: not ready（DR-1〜DR-4 の owner の選択の前）。選択の後、DR-3 が (a) なら Scope・AC はこのまま、(b)・(c) なら Scope と AC を足してから計画の review に出す。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| MNT-01-D13（restore の後の照合日時と精算回数） | 71 §71.7 ステップ 6a、MNT-01-D13 | S1・S3 | `test_restore_backup_req907_carries_newer_settlement_no`・`test_restore_backup_req907_keeps_larger_or_missing_settlement_no`・`test_restore_backup_req907_unreadable_settlement_no_follows_dr2`・`test_restore_backup_req907_settlement_no_write_failure_recovers` | 非対象（L3 Eligibility の (3): 合成の SD と restore の状態が要る） |
| MNT-01-D1・D4・D5（巻き戻し・失敗の分類・manifest の中断の契約） | 71 §71.7 ステップ 3.5〜8、MNT-01-D5 | S1（ステップ 6a の失敗を今の `install_result` の巻き戻しに載せるだけ。変えない） | `test_restore_backup_req907_settlement_no_write_failure_recovers`、既存の restore の test | 非対象 |
| BIZ-04-D3 / SPEC-PLS-D2（占有の照合の規則と gate） | 33 §16.3 手順 1〜4、plu-tables の key の表 | S2（読み方の移動だけ。規則は変えない） | `test_parse_settlement_no_req907_ascii_digits_only`、既存の `refresh_plu_register_snapshot_from_sd` の test | 非対象 |
| 33 §16.8 の「DB の restore の後」 | 33 §16.8 | S3 | `test_restore_then_stale_sd_req907_does_not_free_register_slot` | 非対象 |
| D-160 (7)・D-168 の Guarantee range（受け入れた穴） | decision-log の D-160・D-168 | S3（D-176 で置き換え） | `test_restore_then_stale_sd_req907_does_not_free_register_slot` | 非対象 |
| D-160 (4)・UI-07-D15 (8)・45 §45.6a 手順 2a（古い SD の知らせ） | 55 UI-07-D15 (8)、45 §45.6a | 変えない（今の `SkippedOlder` の知らせが restore の後にも出る）。DR-3 (b) なら変える | `test_restore_then_stale_sd_req907_does_not_free_register_slot`（`SkippedOlder` が返ること） | 非対象（DR-3 (b) なら計画の review で L3 を決める） |
| 22 §24 手順 6（migration が照合日時を消す） | 22 §24 | 変えない（精算回数の key は migration で消さない。この lane は migration を足さない） | 既存 | 非対象 |
| 71 §71.13 の本番の復元 | 71 §71.13 | DR-4 (b) のときだけ S3 | — | 非対象 |
| §71.11 PC の外の控え（`offsite-backup.json`） | 71 §71.11 | 変えない（候補 2・4 を採らない。DR-4 (c) なら変える） | — | 非対象 |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| MNT-01-D13 引き継ぎ（success） | restore で最後の照合の精算回数が backup の値へ戻る | integration（`src-tauri/src/mnt/restore.rs` の test module。実 file の SQLite） | `test_restore_backup_req907_carries_newer_settlement_no` | ステップ 6a の引き継ぎの書込みを外す（M1）、読む位置を `drop(current_conn)` の後にする |
| MNT-01-D13 max・無い（edge） | backup の値を下げる、無い値を作る | integration | `test_restore_backup_req907_keeps_larger_or_missing_settlement_no` | `max` を引き継ぐ値の上書きに変える（M2）、引き継ぐ値が無いときに key を消す・空にする |
| MNT-01-D13 読めない（negative、DR-2） | 読めない値・読めない DB で照合の止まりが解ける、または restore が DR-2 の選択と違う結果になる | integration | `test_restore_backup_req907_unreadable_settlement_no_follows_dr2` | 読めない文字列を捨てて backup の値にする（DR-2 A のとき M3）、SQL の error で復元を止める（A のとき） |
| MNT-01-D13・D5 書込みの失敗（failure → rollback） | 引き継ぎに失敗したのに復元した DB の接続を公開する | integration（backup の中の trigger で `INSERT`・`UPDATE` を拒む。production に hook を足さない） | `test_restore_backup_req907_settlement_no_write_failure_recovers` | 書込みの error を握りつぶす（M4）、巻き戻しの後に restore の遺物が残る |
| 33 §16.3 手順 2 × restore（#307 の Final の経路、restore → stale SD → refetch） | restore の後の古い SD で照合し、レジで使われている枠を空きとして予約する | integration（restore と BIZ-04 を同じ test で。合成の Z004） | `test_restore_then_stale_sd_req907_does_not_free_register_slot` | M1、または `refresh_plu_register_snapshot_from_sd` の比べを等しい・小さいでも照合する形にする |
| S2 読み方の 1 か所 | BIZ-04 と restore で精算回数の読み方が食い違う | unit（`src-tauri/src/db/plu_slot_repo.rs`） | `test_parse_settlement_no_req907_ascii_digits_only` | 前の 0・符号・全角の数字・`i64` を超える値の扱いを変える |
| 既存の回帰 | restore の照合日時の削除と BIZ-04 の照合の規則が変わる | integration / unit | `test_restore_backup_req907_clears_snapshot_at_keeps_settlement_no`、`refresh_plu_register_snapshot_from_sd` の既存の test | S1・S2 で既存の振舞いを変える |

状態の遷移: initial（今の DB に key が無い）・success（引き継ぐ）・failure（書込みの失敗 → 巻き戻し → 元の DB で再試行できる）・restart（`committed` の前の中断は元の DB、後は引き継ぎ済みの DB。MNT-01-D5 の既存の test が持つ）・refetch（restore の後の古い SD → 新しい SD）を上の表で持つ。mutation は M1〜M4 を実装に注入して red を確かめる（AC7）。

## Recovery

- この lane の変更は restore の手順の中の key 1 つの書込みで、schema・migration は無い。戻すときは実装の PR を revert する（revert の後の build は D-168 の形に戻り、引き継いだ値は DB に残るが、値は「最後に照合した精算回数」の意味を変えないので害が無い）。
- 引き継ぎの書込みが店の PC で失敗すると、restore は元の DB へ戻って `restore_failed_recovered` を返す（今の巻き戻しの経路）。店は同じ操作をやり直せる。続けて失敗するなら開発者が原因を見る（今の照合日時の削除の失敗と同じ扱い）。
- 引き継いだ値が誤って大きいときは照合が止まり（`SkippedOlder` の知らせが出る）、書出しが止まる側に倒れる。レジの枠を重ねて書く側には倒れない。直し方は D-117 の Revisit と同じ（開発者が key を直す）。

## Data Safety

- 実 POS の file・実 DB・実 SD・backup・secret を読まない・commit しない。test の DB・backup・Z004 は test の中で作る合成だけ（精算回数・枠・コードは合成の値。今の `snapshot_bytes` の形）。
- 店の実際の精算回数を repo に書かない（P1 の確かめは owner が店の PC で見て、結果は repo の外の回答台帳に要旨と ID で残す）。
- restore の test は `tempfile::tempdir()` の下だけで file を作り、作業の checkout の外の path を使わない。
