# Plan Packet: PLU の書出しの見直しの runtime（枠を消さない・占有は毎日の SD の読取りで照合する。関門 14 の前提、D-117・D-160）

起票: 2026-11-24。出所は [backlog](../backlog.md) の「PLU の書出しの見直しの runtime」。設計は D-117（設計の lane は PR #163）と D-160（PR #292）で正本に入っていて、この lane は code・migration・bindings・test をその形に合わせる。owner の決定（2026-11-25、repo 外の回答台帳 TD-896・TD-902）で、実機の L3 の当て方（D-073）を D-168 で改める。

## Workflow State

- Risk: R4
- Plan Commit: d278bb1c98f6cd9ecbb26334d0716afe3b4105ac
- Amendments: none

`Plan Commit`・`Amendments` は `scripts/plan-approve.sh` が書く（手で写さない）。

## Risk

Risk: R4

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の R4 の行に 2 つ当たる。(1)「破壊的なデータの lifecycle」: migration が行のある `plu_slots`（どの JAN をレジのどの枠に書いたかの記録）を作り直し（全行を写して `DROP TABLE`）、`release_pending` を `external` に書き換え、列 `released_at` と `app_settings` の key を消す。写し方を誤ると、次の書出しがレジの別の枠を上書きし、レジに書いたものは DB を戻しても戻らない。先例は行のある表の作り直しを R4 とした `docs/plans/2026-10-29-stock-transfer.md`（空の DB を通ることは R4 を下げる理由にしない、と書いた）。(2)「backup の復元」: restore（71 §71.7）に、復元した DB の key を消す手順を足す。

## Goal

Goal Invariant:

### 最小完了条件

- 店主が締めで「SD から読む」を押すだけで、レジの PLU の占有が照合される（Z004 を選ぶ操作が無い）。PLU 書出しの画面は最終照合日時を 1 行で出し、一度も照合していなければ書出しを拒んで売上データ取込みへ案内する。
- アプリは枠を消さない: 書き出す file に clear 行が無く、商品を対象から外す・廃番にする・JAN を変える操作で枠の状態が変わらない。
- 古い SD（精算回数が前回の照合より小さい）を読んでも照合せず、その場で理由つきで知らせる。
- 商品の修正で名称・部門・税率を変えると未反映になる。
- 段階導入の道具（商品一覧の「PLU」列・`plu` の絞り込み・一括の対象化）、件数の要約、「レジメモリNo.」が画面と wire から消える。
- 既存の開発・試験の DB と owner の PC の DB が migration を通り、枠（memory No. と JAN の組）が 1 つも変わらない。

### 失敗定義

- migration の後に、枠の memory No.・JAN・`reserved_at`・`activated_at` のどれかが変わる、または `reserved` / `active` の枠が別の状態になる。
- 一度も照合していない状態・restore の直後に、書出しが空きを仮定して予約できる。
- 古い SD の Z004 で照合して、前回の照合を上書きする。
- 照合の失敗で SD の読取り（日報の候補の一覧）が失敗になる。
- 書出しに、対象の商品を持たない JAN の枠・`external`・`free` の行が出る。
- 保存の後に出力の行（S8 の照合の値の入力）が変わった商品が、古い保存結果の「未反映から外す」で反映済みになる。共有 JAN の売価・税率の不一致を Diff が書き出す。
- 店の毎日の操作が増える。

### 非目的

- JAN の別名の runtime（D-122。この lane の後に入れる。この lane が clear と解放を撤去した後に D-122 の Guarantee range が成り立つ）。
- 部門リンクの決め方と 21 個目の部門（D-117 の範囲外。backlog の同名の項目）、JAN-8 と店内コードの書出し（D-165 の lane）。
- Z004 を SD の一括取込みに入れること（関門 15 の (B)。この lane が入れる手順 10 に依る。snapshot の `z004_files`・`z004_new_count` は (B) が足す）。
- 2 段（書出し → 未反映から外す）、Diff と Full、SD に書かないこと（D-117 (6)。変えない）。
- 初回の本番の PLU の書込みの手順書（D-160 (2)。部門リンクの決め方の後の docs の lane で `docs/initial-load-runbook.md` の流れ 3 に書く。owner の決定 TD-896）。
- CV17・SR-S4000 の実機の L3（owner の決定 TD-902。S12 の D-168）。
- 売り切った廃番の商品の枠の使い回し（owner の決定 TD-908 で後の lane。[backlog](../backlog.md) の「売り切った廃番の商品の PLU の枠の使い回し」。この lane は D-117 (1) のとおり使い回さない）。
- 操作ログの過去の記録の label（`product_bulk_plu_target`・`plu_register_snapshot_import`）の削除（過去の記録の表示に要る。74 は変えない）。

## Scope

D-160 (1) のとおり 1 本の lane。分けるべき技術の理由は見つからなかった: 照合の自動化の前に Z004 を選ぶ step を外すと gate を通る道が無くなり、`release_pending` の撤去は migration と同じ変更で、商品の側の撤去（S6〜S8）だけを先に出しても重なる生成物（bindings・D-052）は残る。

- S1 migration（plu-tables の「migration（D-117、runtime の lane）」の 4 手順）: `src-tauri/src/db/schema_v<N>.rs`（新規）、`src-tauri/src/db/mod.rs`、`src-tauri/src/db/migration.rs` の `migrations()`。形は 22 §21 の vT と同じ（1 つの TX で全行を新しい表へ写し、`DROP`・改名・index の作り直しの後、同じ TX で、作り直しの前に読んだ行と `memory_no` ごとに `scanning_code`・対応づけの後の status・`reserved_at`・`activated_at`・`updated_at` を NULL を含めて比べ、違えば ROLLBACK。共有の helper `commit_transaction`・`rollback_after_error`）。status の対応づけは写しの `INSERT … SELECT` の `CASE` で行う（旧い表を `UPDATE` するなら旧い index を `DROP` した後。v5 の partial UNIQUE は `external` を含むので、v5 で保存できる同じ JAN の `release_pending` と `external`、`release_pending` と `active` の組が、旧い index の下の `UPDATE` で UNIQUE 違反になる）。`plu_slots` を外部キーで参照する表は無い（2026-11-24 に `rg -n 'plu_slots' src-tauri/src/db/schema_v*.rs` で v5 だけと確かめた）。版 N は実装の時点の `origin/main` の最大 + 1（今の最大は 15。`rg -n 'version: 15' src-tauri/src/db/migration.rs` が 119 行目の `version: 15,` の 1 件を返す）。22 に新しい節と MNT-03-D の ID を足す。手順 4 は `plu_register_snapshot_summary` と一緒に `plu_register_snapshot_at` も消し（理由は S9）、plu-tables の手順 4 にその key を足す。
- S2 clear と解放の撤去（33 §16.2・§16.4〜§16.6、25 §12、D-117 (1)）: `src-tauri/src/constants.rs`（`PLU_CLEAR_ROW_ENABLED`）、`src-tauri/src/io/plu_formatter.rs`（`PluExportRowKind`・clear 行）、`src-tauri/src/db/plu_slot_repo.rs`（status を 4 値、`released_at`）、`src-tauri/src/biz/plu_export_service.rs`（`release_plu_slot_for_jan`・`append_clear_rows`・`clear_row_enabled`・`confirm_plu_export_saved_with_clear_enabled`〈wrapper を外して本体を `confirm_plu_export_saved` に戻す〉・`PluPreparedRowKind`・`row_kind`・`over_limit_warning`・`release_pending` の枝）、`src-tauri/src/biz/product_service.rs` の `release_plu_slot_for_jan` の 4 つの呼出し（`update_product`・`toggle_discontinue`・`bulk_set_plu_target`・`commit_import`。`rg -n 'release_plu_slot_for_jan' src-tauri/src` で全件）、`src-tauri/src/db/product_repo.rs` の `release_pending` の tie-break、`src-tauri/src/cmd/plu_export_cmd.rs` の DTO。
- S3 占有の照合（33 §16.3、37 §37.9 手順 10、45 §45.6a 手順 2a、D-117 (3)(4)、D-160 (1)(4)）:
  - BIZ-04: `refresh_plu_register_snapshot_from_sd` と `PluRegisterSnapshotRefresh` を足し、`import_plu_register_snapshot` を private の本体にして精算回数を受ける。今の本体と 33 §16.3 の表の差（2026-11-24 に `plu_export_service.rs` を読んで確かめた）を埋める: 判定を照合の前の状態で行う・同じ JAN の複数の観測は最小番号、アプリの JAN の重複の観測は `external`（今は `release_pending`）、対象の商品を持たない JAN の `active` の枠に別コード → `external`（今は conflict）、dirty にするのは `plu_target=1` の商品だけ（今は JAN の一致する全商品）、同じ TX で `plu_register_snapshot_settlement_no` を書く、操作ログの件数の内訳、gate の文言。`PluRegisterSnapshotSummary` は `snapshot_at` だけにし、`summarize_slots` と key `plu_register_snapshot_summary` の読み書きを外す。
  - 精算回数の読取り（backlog の P3 の (2)）: IO-02 に関数を足さず、既にある `parse_z004` の `ParseResult.settlement_metadata` を使う（`extract_settlement_metadata` は private のまま）。33 §16.7 と 23 §13.3.1 にその名前を書く。`parse_z004` が `Err` の bytes は「読めない bytes」として使わない。
  - BIZ-08: `src-tauri/src/biz/daily_report_import_service/scan.rs`・`mod.rs` に手順 10 と snapshot の `plu_register_z004s` を足す（読取りと形の検査は手順 5 の関数を使う）。読む本数（backlog の P3 の (3)）は窓の中の全部のまま、絞る規則を足さない（scan の時間は未実測。L3-1 で遅さを見る）。
  - CMD-12: `src-tauri/src/cmd/daily_report_import_cmd.rs` の `scan_register_sd_with_state` に手順 2a、`RegisterSdScanResponse` に `plu_snapshot_skipped_older`。CMD-08 の `import_plu_register_snapshot` と `src-tauri/src/lib.rs` の登録（2 つの列）を外す。
- S4 SD を読んだ画面の知らせと invalidation（55 UI-07-D15 (8)、67 §67.10、D-052、D-160 (4)）: `src/features/daily-report-import/` の `types.ts`・`reducer.ts`・`hooks/useDailyReportImportFlow.ts`・`DailyReportImportPage.tsx`（`tidy_failure` の知らせ〈(7)〉と同じ表示の形）。`src/lib/invalidation-contract.ts`・`src/test/invalidation-oracle.ts`・`src/lib/invalidation-contract.meta.test.ts`: C17 を「SD の読取りの成功で slot summary と未反映一覧を stale にする」に置き換え、C19 を外す（番号は詰めない）。`docs/UI_TECH_STACK.md` の D-052 の段落の件数は実装の時点で測り直す。
- S5 UI-08（67 UI-08-D5・D11、§67.4 手順 3、§67.8.1）: `src/features/plu-export/PluExportPage.tsx` から FilePicker の step・件数の要約・上限の警告・解除待ちの説明を外し、最終照合日時の 1 行と未照合の導線にする。CV17 に取り込めなかったときの案内を §67.9 の文（書き出し直す。前に保存した file は取り込まない。UI-08-D4・D9）にする（今の build は「保存済みファイルを再投入するか」と書く。2026-11-25 に `rg -n '保存済みファイルを再投入' src/features/plu-export` で `PluExportPage.tsx` の 2 行と test の 2 行）。「未反映から外す」の結果に未反映のまま残った商品があれば、その場で名指しして書き出し直すよう知らせる（S8）。復帰状態の旧い形（`overLimitWarning`・`row_kind`、照合の値を持たない形）は schema 不一致として捨てる。
- S6 段階導入の道具の撤去（30 §4.9.1、40、20、50 UI-01a-D10〜D12、D-117 (2)）: `product_service.rs` の `bulk_set_plu_target`、`src-tauri/src/cmd/product_cmd.rs` と `lib.rs` の登録、`product_repo.rs` の `find_products_for_bulk_plu_target`・`PluMigrationFilter`・`ProductSearchQuery.plu`・`ProductBulkFilter`、`src-tauri/src/biz/mod.rs` の再 export、`src/features/products/` の `ProductListPage.tsx`・`components/ProductTable.tsx`・`components/PluBulkTargetConfirmDialog.tsx`（削除）・`search.ts`・`lib/return-to.ts`・`priceRevisionSearch.ts`。URL に残った `plu` は無視する。
- S7 「レジメモリNo.」の撤去（51 UI-01b-D19、D-117 (5)）: `product_repo.rs` の `plu_memory_no` の JOIN、`src/features/products/ProductFormPage.tsx`、`plu_memory_no` を持つ test の fixture（`rg -l 'plu_memory_no' src` で全件）。
- S8 `plu_dirty` と、書き出した内容との照合（30 §4.4 手順 4b・4c、33 §16.4・§16.5、D-160 (5)）:
  - `update_product` で名称・部門・税率の変更を足す。最初の commit は red の test（今の code が立てないこと）。JAN の変更は商品 CSV の上書きが今も `plu_dirty=1` にする（変えない。test で固定する）。
  - Diff の群: 未反映の対象の商品から出力の候補の JAN を決め、その JAN の対象の商品の全員で不一致・代表の行・`target_product_codes` を決める（33 §16.4 手順 2・3。今の code は `find_plu_dirty_products_for_plu` の `plu_dirty=1` の商品だけで比べるので、共有 JAN の片方だけ売価・税率を直すと Diff が不一致を見逃して書き出す）。`plu_export_service.rs` の prepare と `product_repo.rs`、33 §16.4 手順 6 と 20 の文。
  - confirm: prepare が行ごとに照合の値（IO-04 が file に書くその行の全 11 列〈メモリNo.〜部門リンク。25 §12〉の CP932 の byte 列の SHA-256。入れてある `sha2`）を返し、confirm は今の商品から同じ値を求め直して、一致する行の商品だけ `plu_dirty=0` にする。一致しない行の商品は未反映のまま残して結果に返す（知らせは S5）。合否はこの byte 列だけで決める（名称を変えても、CP932 の 16 バイトへの切り詰めで落ちる末尾だけの変更は行が同じなので外れる）。復帰状態は照合の値を持ち、JAN・名称・価格の生の値は今までどおり持たない（67 §67.8.1）。33 §16.2・§16.5、41 §17.6、67 に入れる。
- S9 restore の後の key（33 §16.8、D-160 (7)）: `src-tauri/src/mnt/restore.rs` の `restore_backup` で、復元した DB の接続を開いた後・manifest を `committed` にする前に `plu_register_snapshot_at` だけを消す（key が無ければ成功）。消せなければ接続を閉じて今の復元の失敗の経路（71 §71.7 ステップ 8 の巻き戻し）へ返し、復元した DB の接続を公開しない（`committed` の後には元の DB へ戻す経路が無いので、前に置く）。`src-tauri/src/db/system_repo.rs` に key を消す関数は無い（`get_setting`・`get_all_settings`・`upsert_setting` だけ）ので、要る 1 つだけを足す。71 §71.7 に手順（ステップ 6 と 7a の間）と判断の ID を足す。`cleanup_committed`（best-effort で manifest を解放する枝を含む）と起動の経路は変えない（owner の決定 TD-912、2026-11-26）: 前の版の DB は、起動の migration（`complete_reconciled_restore` より前に流れる）と復元の open の migration が S1 の手順 4 で key を消すので、復元の中断・完了・manifest が読めない、のどの形でも接続を返す前に key が消える。費用は、版を上げた DB（本番の DB は無い。Design Readiness の前提）で、書出しの前に SD を 1 回読むこと（既にある `register_snapshot_required` の案内のとおり）。
- S10 L3 の入力物: `src-tauri/src/io/l3_fixtures.rs` の `write_sd_bulk_l3_fixtures` と同じ形（env の `INVENTORY_L3_DAY`・`INVENTORY_L3_OUT`、ignore の test）で、全枠の Z004（5,000 行、CP932・CRLF）を持つ合成の SD の folder を 2 つ作る: `sd-a`（読取りの窓の日数ぶんの、bytes の違う Z004）、`sd-b`（どの Z004 も精算回数が `sd-a` の最大より小さい）。
- S11 test: 下の Test Plan。撤去した機能の test（clear 行・解放・一括の対象化・移行の絞り込み・件数の要約）は、D-117 の設計の改訂を理由に、新しい契約の test へ書き換えるか消す（消す test の名前と理由を PR に並べる）。`src-tauri/tests/design_compliance_test.rs` は対応表に変更が要るときだけ触る。
- S12 設計正本と docs（同じ PR）:
  - 「runtime の状態」の注記・未実装の印・撤去する signature を消す: 33（冒頭・§16.6・§16.8）、plu-tables、25、23 §13.3.1、30（§4.4・§4.9.1）、37 §37.9、45 §45.6a、55 UI-07-D15 (8)、40、41（§17.6 ほか）、20、50、51、67、task-specs の BIZ-01・BIZ-04・IO-04・CMD-01・CMD-08・UI-01a・UI-01b・UI-08、`docs/SCREEN_DESIGN.md`、`docs/DB_DESIGN.md` D-2、`docs/spec/requirements.md` の REQ-907 の注、`docs/project-memory.md`。
  - PR #292 の P3 の 4 件: 67 UI-08-D5 の Why を D-160 の Guarantee range への参照に、55 UI-07-D15 (8) の最後の 2 文の出所（この lane の文、D-127）、`docs/SCREEN_DESIGN.md` の SD の読取りの知らせの要約に (8)、D-160 (7) の穴（中身は Design Readiness の抜け道）を D-168 の Guarantee range に。
  - `docs/decision-log.md` の末尾に D-168（予約済み）: D-052 の C17 の置換えと C19 の撤去、精算回数の読取りと読む本数、restore で key を消す位置、migration の形、Diff の群と confirm の照合（S8）、下の 2 つ。merge 済みの D の本文は書き換えず、D-052・D-073・D-117 に `Superseded in part by: D-168` の行を足す。
    - 実機の L3 の当て方（owner の決定 TD-902）: PLU の file の行の形が変わらない lane は、記録のある前の実機の確かめで CV17 の受理を受け、CV17・レジの実機の L3 を置かない。行の形を変える lane は今までどおり置く。D-073 の (3)（UI-08 の operator flow）は開発の build の L3 で見る。D-117 は Compatibility の「CV17 / SR-S4000 の L3 が要る」。
    - 前の実機の確かめが支える範囲（TD-902 の物差しの範囲として Coordinator が裁定した、2026-11-25）: 2026-08-20 の店の PC での L3 の記録（repo 外の回答台帳 L-012）が支えるのは差分の取込みと SD・レジへの設定の転送で、検証の条件は空き枠への追加だった（`docs/plu-export-and-real-csv-verification.md` の「店舗運用から導いた公開設計前提」）。この lane が新しく枠を取るのは空き枠と同じコードの `external` だけ（33 §16.4 手順 5）で、空き枠は L-012 で足り、同じコードの `external` の採用は今の main のまま（この lane は変えない）。別のコードが入った枠へ商品の行を書き直す経路（33 §16.3 の「別コード × active」の conflict と §16.8、D-117 (4)・Guarantee range）は今の main の build にもあり、この lane は広げない（S3 で、対象の商品を持たない JAN の枠の分だけ狭まる）。その経路の CV17・レジの受理と、前の精算の後の売上の出方は未実測で、次の訪店の上書きの実験（backlog の「売り切った廃番の商品の PLU の枠の使い回し」）が同じ操作を確かめる。
  - 上の 2 つに合わせる: `docs/plu-export-and-real-csv-verification.md` の冒頭の段と PLU-21・PLU-25 の行の「D-117 の runtime の lane の L3 で確かめる」を、PLU-21 は L-012 と owner の答えで受け、支える範囲と未実測の範囲を上のとおりに分けて書く、PLU-25 はこの lane の L3-4 と T20・T28 で受ける、にする。67 §67.12 の適用範囲の段と CV17 へ投入する 2 項目。
  - `docs/diagrams/current-system.md`、`docs/inventory_system_erd.html`（`plu_slots` の列と status）。
  - 片付け（TD-765 の B）: backlog の項目を済みの形にし（関連の「廃番表示 3 択…」「PLU slot 後続 follow-up」の runtime への参照も）、`docs/Plans.md` の関門 14 の「前提の lane は … 未起票」と「流れ 3 の中の位置と手順は、前提の lane か手順書の改訂で決める」を今の形にする（手順書は部門リンクの決め方の後の docs の lane、TD-896）。backlog の P3 の (1)（書き出す前にその日の SD を読む、の手順書の文）は、その docs の lane の入力として backlog の「PLU をレジへ書く時期と D-070 の自動在庫連動」へ移す。
- 生成物: `src/lib/bindings.ts`（`cargo run --bin generate_bindings`）、`docs/function-design/90-traceability.md`（`cargo run --bin generate_traceability`）。`src/routeTree.gen.ts` は route を足さないので変わらない見込み。

### 並走の lane と重なる file の所有

どちらが先に merge しても、後の側が `origin/main` の単段の同期の後に L3 をする。生成物は手で解かず、取り込んだ後に作り直す。

| 重なる file | この lane が持つ行・節 | 相手 |
|---|---|---|
| `src-tauri/src/db/migration.rs`・`db/mod.rs`、22 の末尾の節 | この lane の `schema_v<N>` の `mod`・`use`・`migrations()` の 1 件と 22 の 1 節。後に merge する側が版・file 名・節の番号を取り直す（22 §21 の「版の番号の取り直しと開発・試験 DB」。取り直しは runtime の変更なので L3 は同期の後） | 関門 17 |
| `src-tauri/src/biz/daily_report_import_service/`、`cmd/daily_report_import_cmd.rs`、`lib.rs` の登録、37・45・55 | scan の手順 10 と `plu_register_z004s`、`scan_register_sd` の手順 2a と response の 1 field、`import_plu_register_snapshot`・`bulk_set_plu_target` の登録の削除、37 §37.9・45 §45.6a・55 UI-07-D15 (8)。EJ の閲覧（`ej.rs`・§37.11・§45.6c・UI-07-D17）は持たない | 関門 19 |
| `src/lib/invalidation-contract.ts`・`src/test/invalidation-oracle.ts`・meta test、`docs/UI_TECH_STACK.md` の D-052 の段落 | C17・C19 の entry と、その説明の文。件数の文は後に merge する側が測り直す | 関門 19、PR #300（UI_TECH_STACK のほかの節） |
| `src/lib/bindings.ts`、`docs/function-design/90-traceability.md`、`docs/spec/requirements.md` | 生成物は作り直す。requirements は REQ-907 の行と注だけ | 関門 19 |
| `docs/decision-log.md` | D-168 と、D-052・D-073・D-117 への Superseded の行（末尾の追記は merge 順で両方を残す） | 関門 17（D-167）、関門 19（D-169）、PR #300（D-164）、PR #304（D-166） |
| 23、`docs/plu-export-and-real-csv-verification.md` | 23 §13.3.1 の入力の文、冒頭の段の PLU-21・PLU-25 の文と PLU-21・PLU-25 の行。`z004_parser.rs` は触らない | PR #304（23 §13.4.2・§13.5、Z004 の 8 桁の行） |
| `docs/backlog.md` | 「PLU の書出しの見直しの runtime」と上の関連の 2 項目、「売り切った廃番の商品の PLU の枠の使い回し」（計画の commit で足した） | PR #300（E-1 の項目） |

## Acceptance Criteria

- AC1（S1）: 5 つの status の行（同じ JAN の `release_pending` と `external`、同じ JAN の `release_pending` と `active` の 2 組を含む）を持つ直前の版の DB で migration が成功し、4,784 行の全部で、migration の前の行から test が独立に作った期待値（`release_pending` は `external`、ほかは同じ）と、`memory_no` ごとの `scanning_code`・status・`reserved_at`・`activated_at`・`updated_at` が NULL を含めて同じ。`released_at` 列と key `plu_register_snapshot_summary`・`plu_register_snapshot_at` が無い。同じ JAN の `external` が複数行と、`external` と `active` の同じ JAN を保存でき、`reserved` / `active` の同じ JAN の 2 行目は拒まれる。写しの後に 2 枠の JAN が入れ替わると migration は失敗し、旧い schema・行・版・key `plu_register_snapshot_summary` が元のまま（T1〜T4）。
- AC2（S2）: 対象外化・廃番化・JAN の変更の後も枠の状態が変わらず、Diff / Full の file にその枠の行も clear 行も無い。対象へ戻すと同じ memory No. を使い、confirm の前の書き出し直しは Diff も Full も同じ memory No. を返す（`cd src-tauri && cargo test --lib` の T10〜T12・T23・T28 が PASS）。IO-04 の product 行の既存の test（T23）は期待値を変えずに通る（変える要が出たら行の形が変わるので、D-168 の条件で CV17 の実機の L3 を置き直す。止めて報告する）。
- AC3（S3）: 精算回数が最大の 1 本だけを照合し、key が無い・より大きいときだけ照合する。等しい・小さい・key が読めない・使える Z004 が無いときは枠・app_settings・操作ログを何も変えない。33 §16.3 の表の全部の行が test にある（`cargo test --lib` の T5〜T9 が PASS）。
- AC4（S3）: `scan_register_sd` は照合の `Err` でも成功を返し、`SkippedOlder` のときだけ `plu_snapshot_skipped_older` が `true`。cache の snapshot に Z004 の bytes が残らない（T14・T15）。
- AC5（S4・S5）: 知らせの文は `true` のときだけ出て、読み直すと消える。SD の読取りの成功で slot summary と未反映一覧が stale になる。PLU 書出しの画面は未照合で warning と導線、照合済みで日時の 1 行。CV17 に取り込めなかったときの案内は §67.9 の文で、保存の後と復帰の Alert の 2 か所に出る（T20〜T22）。
- AC6（S6・S7）: 商品一覧に「PLU」列・PLU表示・一括操作が無く、URL の `plu` を無視する。商品フォームに「レジメモリNo.」が無い（T17〜T19）。
- AC7（S8）: T13 は実装の前に red（`cd src-tauri && cargo test --lib -- test_update_product_req907_name_department_tax_change_sets_plu_dirty` が exit 101）、実装の後に exit 0。原価だけ・単位だけ・同じ値の更新では立てない。共有 JAN の片方だけ税率を変えると、その JAN は Diff・Full とも `group_price_mismatch` で除外される。保存の後に出力の行（S8 の照合の値の入力）が変わった商品は、復帰状態を経ても「未反映から外す」で未反映のまま残り、その場で名指しの知らせが出て、行が変わっていない商品だけが外れる: 名称の先頭の 1 文字を変えた商品は残り、CP932 の 16 バイトへの切り詰めで落ちる末尾だけを変えた商品は外れる（T13・T20）。
- AC8（S9）: `restore_backup` の後、返った接続に `plu_register_snapshot_at` が無く、`plu_register_snapshot_settlement_no` は backup の値のまま。書出しは `register_snapshot_required` で拒まれる。key の削除が失敗すると `restore_backup` は失敗を返して元の DB に戻り、復元した DB の接続は公開されない（T16）。
- AC9: Test Plan の mutation（M1〜M10・M14〜M17・M20）を 1 つずつ注入すると、名指しの test が red（Rust は `cargo test --lib -- <test 名>` が exit 101、frontend は `npm test -- <test の file>` が exit 1）になり、戻すと exit 0。
- AC10（撤去の検査）: `rg -n 'release_pending|released_at|PLU_CLEAR_ROW_ENABLED|release_plu_slot_for_jan|bulk_set_plu_target|bulkSetPluTarget|find_products_for_bulk_plu_target|PluMigrationFilter|over_limit_warning|PluPreparedRowKind|PluExportRowKind|plu_memory_no|plu_register_snapshot_summary|importPluRegisterSnapshot|clear_row_enabled|with_clear_enabled|append_clear_rows' src src-tauri/src src-tauri/tests` の当たりが、`schema_v5.rs`・この lane の `schema_v<N>.rs`・`migration.rs` の v5 の test・UI-08 の旧い復帰状態を捨てる test だけ。`rg -c 'runtime の状態' docs --glob '!docs/archive/**' --glob '!docs/plans/**' --glob '!docs/research/**' --glob '!docs/decision-log.md' --glob '!docs/backlog.md'` が 0 件（起票の時点の `1edcedc3` は 6 file で 11 行: biz-task-specs 3・cmd-task-specs 2・io-task-specs 1・ui-task-specs 3・plu-tables 1・33 1。どれも PLU の注記）。
- AC11（gate）: `docs/DEV_WORKFLOW.md` Verification Gates の Rust・bindings・frontend・traceability・docs の command が全部通り、`bash scripts/doc-consistency-check.sh` が ERROR 0。`cargo check --release` が通る（L3 の前）。

## Design Readiness

- 引用する設計正本: 33 §16（全節）、plu-tables、25 §12、23 §13.3・§13.3.1、30 §4.4（4b・4c）・§4.9.1、37 §37.9（手順 10）、45 §45.6a（手順 2a）、55 UI-07-D15 (8)、67（UI-08-D4・D5・D9・D11、§67.4・§67.8.1・§67.9・§67.10・§67.12）、40、41 §17.6、20、50 UI-01a-D10〜D12、51 UI-01b-D19、71 §71.7、22 §21（作り直しの形の先例）、`docs/UI_TECH_STACK.md` の D-052、decision-log の D-052・D-072・D-073・D-103・D-117・D-122・D-160。
- 必要な設計成果物: existing sufficient（D-117・D-160 が正本に入れた）。updated in this PR は S12 と、22 の新しい節、71 §71.7 の手順、Diff の群と confirm の照合（S8 に挙げた節）、D-168。
- plan にしかない durable な判断の昇格先: 実機の L3 の当て方 → D-168（S12）。精算回数の読取りに `parse_z004` を使う・窓の中を全部読む → 33 §16.7・37 §37.9 手順 10・D-168。C17 の置換えと C19 の撤去 → D-168 と `docs/UI_TECH_STACK.md`。restore で key を消す位置 → 71 §71.7 と D-168。migration の形と手順 4 で消す key → 22 の新しい節と plu-tables。Diff の群と confirm の照合 → S8 に挙げた節と D-168。
- 前提: 本番の DB は無い（`docs/project-memory.md`）。migration の対象は開発・demo・試験の DB と owner の PC の DB。
- 計画の review round 1 の裁定で閉じた設計の問い（どれも agent が決めた。owner へ送る問いは残っていない）:
  - D-160 の wire（`PluRegisterSnapshotRefresh`・`plu_snapshot_skipped_older`）: 正本のまま（round 1 で指摘なし）。
  - restore で key を消す位置と、best-effort で manifest を解放する枝: S9。
  - migration は作り直す（plu-tables のまま）。status の CHECK を 5 値のまま `UPDATE` だけにする代案は、撤去した状態を DB が受け続け `released_at` 列も残るので採らない。対応づけの順は S1。
  - scan で読む Z004 は窓の中の全部のまま。時間は未実測で、L3-1 が窓の日数ぶんの本数（S10）で見る。
  - 実機の L3 を外す範囲: S12 の D-168。
- 絶対保証の例外と抜け道: 「アプリは枠を消さない」「使い回さない」は変えない（アプリ管理の枠が別のコードの枠になるのは、照合でレジ側に別のコードを観測したときだけ。33 §16.9）。「古い SD で照合しない」は、restore の後は backup の時点の精算回数より大きく restore の前の最後の照合より小さい SD を照合しうる（D-160 (7) の穴。直さず、受け入れる限界として D-168 の Guarantee range に書く。締めで今の SD を読む普通の流れでは起きない）。レジの精算回数が戻ると照合が止まり、自動の脱出は無い（D-117 の Revisit）。別のコードが入った枠へ書き直す経路の受理と売上の出方は未実測（S12 の D-168）。
- 判定: ready。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| MNT-03-D9 の後の migration（D-117 (1)） | plu-tables の migration、22 の新しい節 | S1 | T1〜T4 | 非対象（L3 は空の DB。行のある DB は T1〜T4） |
| BIZ-04-D3 / SPEC-PLS-D2 照合の表 | 33 §16.3 | S3 | T7・T8 | L3-1（照合が走ること。表の行は T7） |
| D-117 (3) 精算回数の選択と key | 33 §16.3 手順 1〜3 | S3 | T5・T6 | L3-2 |
| gate `register_snapshot_required` | 33 §16.3・§16.9 | S3・S5 | T9・T20 | L3-1 |
| BIZ-04-D4 prepare（sticky・clear 無し・Diff の群） | 33 §16.4 | S2・S8 | T10・T12・T13・T28 | L3-4 |
| BIZ-04-D5 confirm（IO-04 の行の byte 列が今と一致する分だけ外す） | 33 §16.5、41 §17.6、67 §67.8.1 | S2（clear の confirm を外す）・S8・S5 | T10・T13・T20 と既存の confirm の test | L3-4 |
| BIZ-04-D6 枠を解放しない | 33 §16.6、30 §4.4 4b | S2 | T11 | 非対象（BIZ） |
| IO-04-D2・D4（IO-04-D3 の撤去） | 25 §12 | S2 | T23 | 非対象（D-168） |
| BIZ-08 手順 10 | 37 §37.9 | S3 | T14 | L3-1 |
| CMD-12 手順 2a・`plu_snapshot_skipped_older` | 45 §45.6a | S3 | T15 | L3-2 |
| CMD-08-D4 の撤去・summary の wire | 41 §17.6 | S3 | T20、bindings の差分 | 非対象 |
| UI-07-D15 (8) | 55 | S4 | T21 | L3-2 |
| D-052 C17・C19 | `docs/UI_TECH_STACK.md`、67 §67.10、D-168 | S4 | T22 | 非対象 |
| UI-08-D5・D11 | 67 | S5 | T20 | L3-1 |
| BIZ-01-D4・CMD-01-D4・UI-01a-D10〜D12 の撤去 | 30 §4.9.1、40、20、50 | S6 | T17・T18 | L3-3 |
| UI-01b-D19・IO-01-D4 の `plu_memory_no` | 51、20 | S7 | T19 | L3-3 |
| D-160 (5) `plu_dirty` | 30 §4.4 4c | S8 | T13 | L3-4 |
| D-160 (7) restore の後の key | 33 §16.8、71 §71.7 | S9 | T16 | 非対象（restore の実機の確かめは足さない。71 の既存の L3 の範囲を変えない） |
| D-160 (7) の穴（PR #292 の P3 の 4） | D-168 の Guarantee range | 直さない（Design Readiness の抜け道） | — | 非対象 |
| D-073 の当て方（TD-902）と PLU-21・PLU-25 | D-168、67 §67.12、UI-08-D4・D9、§67.9 | S5・S12 | T20・T23・T28 | L3-4（開発の build）。CV17・レジの実機は非対象 |
| 隣の契約（除外）: BIZ-03 の Z004 の取込み（32）、D-122 の別名、停止 ADR | 33 §16.3 の末尾 | 触らない | — | 非対象 |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| migration | `release_pending` が残る・枠の memory No. と JAN の組が変わる・v5 で保存できる同じ JAN の 2 組（AC1）で失敗する | integration（期待値は migration の前の行から test が独立に作り、`memory_no` ごとに比べる） | T1 `test_migration_req907_plu_slots_rebuild_maps_release_pending_to_external` | status の対応づけを外す（M1）・旧い index を残して `UPDATE` する（M14）・写しで 2 枠の JAN を入れ替える（M15） |
| partial UNIQUE | `external` の重複を保存できない・管理の重複を許す | integration | T2 `test_migration_req907_plu_slots_rebuild_unique_only_reserved_active` | index の条件に `external` を戻す（M2） |
| migration の確かめ | 2 枠の JAN の入れ替えを見逃す、失敗の後に新しい schema・行・版が残る・key `plu_register_snapshot_summary` が消える | integration（test だけの hook〈`#[cfg(test)]`〉で、写しの後・確かめの前に 2 枠の JAN を入れ替える） | T3 `test_migration_req907_plu_slots_rebuild_verify_failure_rolls_back` | 確かめから `memory_no` ごとの比べを外す（M16）・TX の外で版を書く |
| key の削除 | `plu_register_snapshot_summary`・`plu_register_snapshot_at` のどちらかが残る | integration | T4 `test_migration_req907_plu_slots_rebuild_drops_summary_key` | 手順 4 を落とす・消す key から `plu_register_snapshot_at` を外す（M20） |
| 最大の 1 本 | 古い Z004 を選ぶ・同じ精算回数で bytes の違う 2 本で照合する | unit | T5 `test_refresh_plu_register_snapshot_from_sd_req907_picks_max_settlement_no` | 最初の 1 本を選ぶ（M4） |
| key の比較 | 等しい・小さい・読めない key で照合する（negative: 何も変えない） | unit | T6 `test_refresh_plu_register_snapshot_from_sd_req907_key_comparison` | 比較を `>=` にする（M3） |
| 照合の表 | 表の行の結果が違う（重複 → `external`、対象なしの別コード → `external`、照合の前の状態で判定） | unit（表の行ごと） | T7 `test_import_plu_register_snapshot_req907_reconciliation_table` | 対象なしの別コードを conflict に戻す（M5） |
| 照合の TX | 失敗で枠・key・ログの一部が残る | unit | T8 `test_import_plu_register_snapshot_req907_rolls_back_settings_log_and_slots`（既存を新しい key に合わせる） | key の更新を TX の外に出す |
| gate | 未照合で予約できる | unit | T9 `test_prepare_plu_export_req907_requires_snapshot_without_writes`（既存。文言を合わせる） | gate を外す |
| clear 無し | 対象を持たない JAN の枠・clear 行が出る | unit | T10 `test_prepare_plu_export_req907_never_emits_rows_for_untargeted_slots` | prepare に対象外の枠の行を足す（M10） |
| 枠を解放しない | 対象外化・廃番化・JAN の変更で枠が変わる | unit | T11 `test_product_service_req907_keeps_slot_on_untarget_discontinue_and_jan_change` | 解放の呼出しを戻す |
| sticky | 再対象化で別の枠を取る | unit | T12 `test_prepare_plu_export_req907_retarget_reuses_same_memory_no` | 既存の枠の再利用を外す |
| D-160 (5) と Diff の群・confirm の照合 | 名称・部門・税率の変更で未反映にならない／原価・単位だけで立つ、片方だけ税率を変えた共有 JAN を Diff が書き出す、保存の後に出力の行が変わった商品が confirm で外れる・行が同じ商品が残る | unit（red から。共有 JAN は Diff・Full の除外まで、confirm は保存の後に名称の先頭の 1 文字を変えると残る／切り詰めで落ちる末尾だけを変えると外れる、の対） | T13 `test_update_product_req907_name_department_tax_change_sets_plu_dirty`、`test_prepare_plu_export_req907_shared_jan_partial_change_excluded_in_diff_and_full`、`test_confirm_plu_export_saved_req907_clears_only_rows_with_unchanged_content` | 条件から税率を外す（M6）・Diff の群を未反映の商品だけで作る・confirm の照合を外す・照合の値を切り詰める前の名称から作る |
| 手順 10 | 窓の外・読めない Z004 を渡す、同じ bytes を重ねる、DB へ書く | integration（合成の SD の tree） | T14 `test_scan_register_sd_req907_collects_window_z004_bytes` | 形の検査を飛ばす |
| 手順 2a | 照合の `Err` で scan が失敗する・flag が違う・bytes が cache に残る | unit（CMD） | T15 `test_scan_register_sd_req907_refreshes_snapshot_and_reports_skipped_older` | flag を固定にする（M7） |
| restore の後 | 2 つの key を消す・消さない、key の削除が失敗しても復元した DB の接続を公開する | integration（削除の失敗は、backup の fixture に `plu_register_snapshot_at` の `DELETE` を拒む trigger を置いて注入する。production に hook を足さない） | T16 `test_restore_backup_req907_clears_snapshot_at_keeps_settlement_no` | `settlement_no` も消す（M8）・key の削除の失敗を warn にして接続を返す（M17） |
| search の状態 | URL の `plu` で絞り込む・戻り先に `plu` を運ぶ | frontend unit | T17 `search.test.ts`・`lib/return-to.test.ts` の `plu` を無視する case | schema に `plu` を残す |
| UI-01a | 列・絞り込み・一括操作が残る | frontend | T18 `ProductListPage.test.tsx`・`components/ProductTable.test.tsx` | 列を残す |
| UI-01b | 「レジメモリNo.」が残る | frontend | T19 `ProductFormPage.test.tsx` | 表示を残す |
| UI-08 | 未照合で書出しへ進める、step・要約が残る、旧い復帰状態で壊れる、取り込めなかったときの案内が前の file の再投入を勧める、復帰状態からの confirm で未反映のまま残った商品を知らせない | frontend（initial / pending / success / failure / retry / revisit / restart） | T20 `PluExportPage.test.tsx` | 未照合の分岐を外す・残った商品の知らせを外す |
| UI-07-D15 (8) | `false` で出る・`true` で出ない・読み直しても残る | frontend（success / refetch） | T21 `DailyReportImportPage.sd.test.tsx` | 表示の条件を反転する（M7 の UI 側） |
| D-052 | SD の読取りの後に summary・未反映一覧が古いまま | frontend（invalidate / refetch。oracle は独立に転記） | T22 `invalidation-contract.meta.test.ts` と scan の成功の handler の test | 集合から `pluDirty` を外す（M9） |
| IO-04 | clear 行の形が残る・product 行が変わる | unit | T23 `plu_formatter.rs` の既存の product 行の test（期待値を変えない。clear 行の test は消す） | 列・桁を変える |
| 書き出し直し（PLU-25） | confirm の前の書き出し直しで memory No. が変わる | unit | T28 `test_prepare_plu_export_req907_uses_min_free_and_sticky_allocation`（既存。Diff の後の Full が同じ memory No.） | 予約済みの枠の再利用を外す |

mutation（実装の run で 1 つずつ注入して red を確かめ、戻す）: M1〜M10・M14〜M17・M20 は表の「Would fail if...」の括弧のとおり（T24〜T27・M11〜M13 は、後の lane に分けた枠の使い回しの分、M18・M19 は外した起動時の削除の分で欠番）。

## L3

CV17・SR-S4000 の実機は使わない（S12 の D-168）。全部を開発用の Windows の native build・空の DB と S10 の合成の SD の folder（CP932・CRLF、「場所を選ぶ」で選ぶ）で行う。商品と JAN は合成の値で、L3 の依頼に書く。lane の runtime の変更が止まり、並走の lane との同期が済んだ後に頼む。

- L3-1 PLU 書出し / 空の DB で PLU 書出しを開く → 売上データ取込みで `sd-a` を読む → PLU 書出しへ戻る / 読む前は warning と売上データ取込みへの導線が出て書出しへ進めない。読んだ後は「レジの登録状況: {日時}」の 1 行が出て、Z004 を選ぶ step と件数の要約が無い。SD の読取りが待てる速さ。
- L3-2 売上データ取込み / 続けて `sd-b`（精算回数が `sd-a` より小さい Z004 だけ）を読む / 読み終えた文の直下に、照合しなかったことと理由の文が出る。日報の一覧（か「新しい精算はありません」）は今までどおり出る。文が自然に読める。
- L3-3 商品一覧と商品フォーム / 商品 A を登録し（13 桁の JAN、「レジにバーコード登録する」を入れる、初期在庫は既定の `0` のまま）、一覧と A の修正の画面を開く / 一覧に「PLU」列・PLU表示・PLU 一括操作が無い。フォームに「レジメモリNo.」が無い。
- L3-4（PLU-25） PLU 書出し / 差分を書き出して保存する → 未反映から外さずに、全件を書き出して別の名前で保存する → 2 つの file をメモ帳で開く → 「この書出しを未反映から外す」を押す → A の修正で名称の先頭の 1 文字を変え（英数字を別の英数字に。変える前後の名称は L3 の依頼に書く）、ホームと PLU 書出しを見る → 差分を書き出して保存する → A の名称の先頭の 1 文字をもう一度変える → PLU 書出しへ戻り「この書出しを未反映から外す」を押す / 保存の後に、書き出し直すことと、前に保存した file を取り込まないことの案内が出て、文が自然に読める。2 つの file の A の行のメモリNo. が同じ。外した後は A が未反映の一覧から消え、名称を変えるとホームに未反映 1 件が出て、PLU 書出しの一覧に A の名前が出る。保存の後に名称の先頭を変えた A は、外す操作の後も未反映の一覧に残り、書き出し直す知らせが A を名指しして出て、文が自然に読める。

## Recovery

- migration の失敗: 1 つの TX の中なので、この migration の変更と版は残らない（22 §21 の「止まった後の進め方」と同じ扱い）。直した build で同じ DB を開けば当たり直す。
- migration の COMMIT の後: 前の版のアプリでは開けない（`SchemaNewerThanApp`）。戻すのは更新の前の backup（22 §16 の「回復」、`docs/migration-recovery-runbook.md`）。migration が捨てるのは `released_at` の値と、`release_pending` と `external` の区別（この後の code はどちらも読まない）、最終照合日時（次に SD を読んで照合すれば入る）。
- code の revert: squash commit の revert では DB は戻らない（上の backup）。版の番号を取り直した build で開く前の扱いは 22 §21 の「版の番号の取り直しと開発・試験 DB」。
- restore で key を消せなかったとき: restore は失敗を返し、元の DB に戻る（71 §71.7 ステップ 8）。原因を直してから restore をやり直す。

## Data Safety

- commit しないもの: 店の SD の file・実 Z004・店の DB・backup・書き出した PLU の file。L3 の商品と JAN は合成の値。
- 合成だけの path: S10 の入力物は `INVENTORY_L3_OUT` の folder へ手元で作り、tracked にしない。test の Z004 と SD の tree は合成の値。
- 操作ログ（`plu_register_snapshot_import`）は件数だけで、実コード・名称・単価を入れない（33 §16.3）。
- migration は数量・在庫・売上・商品を変えない。写しの確かめ（S1）が通らなければ何も変えない。
- 復元した DB の接続は、`plu_register_snapshot_at` を消した後にだけ公開する（S9）。
