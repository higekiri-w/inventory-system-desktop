# Plan Packet: PLU の書出しの見直しの runtime（枠を消さない・占有は毎日の SD の読取りで照合する。関門 14 の前提、D-117・D-160）

起票: 2026-11-24。出所は [backlog](../backlog.md) の「PLU の書出しの見直しの runtime」。設計は D-117（設計の lane は PR #163）と D-160（PR #292）で正本に入っていて、この lane は code・migration・bindings・test をその形に合わせる。計画の review と実装は次の session。

## Workflow State

- Risk: R4
- Plan Commit: pending
- Amendments: none

`Plan Commit`・`Amendments` は `scripts/plan-approve.sh` が書く（手で写さない）。

## Risk

Risk: R4

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の R4 の行に 2 つ当たる。(1)「破壊的なデータの lifecycle」: migration が行のある `plu_slots`（どの JAN をレジのどの枠に書いたかの記録）を作り直し（全行を写して `DROP TABLE`）、`release_pending` を `external` に書き換え、列 `released_at` と `app_settings` の key を消す。写し方を誤ると、次の書出しがレジの別の枠を上書きし、レジに書いたものは DB を戻しても戻らない。先例は行のある表の作り直しを R4 とした `docs/plans/2026-10-29-stock-transfer.md`（空の DB を通ることは R4 を下げる理由にしない、と書いた）。(2)「backup の復元」: restore の後処理（71 §71.7）に key を消す処理を足す。あわせて R3 の行の DB・PLU TSV（clear 行を書かなくなる）・Tauri command の DTO・route と search の状態（商品一覧の `plu`）・operator の業務の流れに当たる。backlog は「R3 見込み」と書いていたが、設計の時点の見込みで、migration の形と restore を Risk Tiers に当てたのはこの起票が初めて。workflow の file（`.github/`・`scripts/`・classifier の `workflow=true` の docs）には触れない。

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
- 店の毎日の操作が増える。

### 非目的

- JAN の別名の runtime（D-122。この lane の後に入れる。この lane が clear と解放を撤去した後に D-122 の Guarantee range が成り立つ）。
- 部門リンクの決め方と 21 個目の部門（D-117 の範囲外。backlog の同名の項目）、JAN-8 と店内コードの書出し（D-165 の lane）。
- Z004 を SD の一括取込みに入れること（関門 15 の (B)。この lane が入れる手順 10 に依る。snapshot の `z004_files`・`z004_new_count` は (B) が足す）。
- 2 段（書出し → 未反映から外す）、Diff と Full、SD に書かないこと（D-117 (6)。変えない）。
- 初回の本番の PLU の書込みの手順書（D-160 (2)。下の「owner に決めてもらう事項」の O2）。
- 操作ログの過去の記録の label（`product_bulk_plu_target`・`plu_register_snapshot_import`）の削除（過去の記録の表示に要る。74 は変えない）。

## Scope

D-160 (1) のとおり 1 本の lane。分けるべき技術の理由は見つからなかった: 照合の自動化の前に Z004 を選ぶ step を外すと gate を通る道が無くなり、`release_pending` の撤去は migration と同じ変更で、商品の側の撤去（S6〜S8）だけを先に出しても重なる生成物（bindings・D-052）は残る。

- S1 migration（plu-tables の「migration（D-117、runtime の lane）」の 4 手順）: `src-tauri/src/db/schema_v<N>.rs`（新規）、`src-tauri/src/db/mod.rs`、`src-tauri/src/db/migration.rs` の `migrations()`。形は 22 §21 の vT と同じ（1 つの TX で全行を写し、`DROP`・改名・index の作り直しの後、同じ TX で行数と memory No. の集合と列の一致〈status は対応づけの後〉を確かめ、違えば ROLLBACK。共有の helper `commit_transaction`・`rollback_after_error`）。`plu_slots` を外部キーで参照する表は無い（2026-11-24 に `rg -n 'plu_slots' src-tauri/src/db/schema_v*.rs` で v5 だけと確かめた）。版 N は実装の時点の `origin/main` の最大 + 1（今の最大は 15。`rg -n 'version: 15' src-tauri/src/db/migration.rs` が 119 行目の `version: 15,` の 1 件を返す）。22 に新しい節と MNT-03-D の ID を足す。
- S2 clear と解放の撤去（33 §16.2・§16.4〜§16.6、25 §12、D-117 (1)）: `src-tauri/src/constants.rs`（`PLU_CLEAR_ROW_ENABLED`）、`src-tauri/src/io/plu_formatter.rs`（`PluExportRowKind`・clear 行）、`src-tauri/src/db/plu_slot_repo.rs`（status を 4 値、`released_at`）、`src-tauri/src/biz/plu_export_service.rs`（`release_plu_slot_for_jan`・`append_clear_rows`・`PluPreparedRowKind`・`row_kind`・`over_limit_warning`・`release_pending` の枝）、`src-tauri/src/biz/product_service.rs` の `release_plu_slot_for_jan` の 4 つの呼出し（`update_product`・`toggle_discontinue`・`bulk_set_plu_target`・`commit_import`。`rg -n 'release_plu_slot_for_jan' src-tauri/src` で全件）、`src-tauri/src/db/product_repo.rs` の `release_pending` の tie-break、`src-tauri/src/cmd/plu_export_cmd.rs` の DTO。
- S3 占有の照合（33 §16.3、37 §37.9 手順 10、45 §45.6a 手順 2a、D-117 (3)(4)、D-160 (1)(4)）:
  - BIZ-04: `refresh_plu_register_snapshot_from_sd` と `PluRegisterSnapshotRefresh` を足し、`import_plu_register_snapshot` を private の本体にして精算回数を受ける。今の本体と 33 §16.3 の表の差（2026-11-24 に `plu_export_service.rs` を読んで確かめた）を埋める: 判定を照合の前の状態で行う・同じ JAN の複数の観測は最小番号、アプリの JAN の重複の観測は `external`（今は `release_pending`）、対象の商品を持たない JAN の `active` の枠に別コード → `external`（今は conflict）、dirty にするのは `plu_target=1` の商品だけ（今は JAN の一致する全商品）、同じ TX で `plu_register_snapshot_settlement_no` を書く、操作ログの件数の内訳、gate の文言。`PluRegisterSnapshotSummary` は `snapshot_at` だけにし、`summarize_slots` と key `plu_register_snapshot_summary` の読み書きを外す。
  - 精算回数の読取り（backlog の P3 の (2)）: IO-02 に関数を足さず、既にある `parse_z004` の `ParseResult.settlement_metadata` を使う（`extract_settlement_metadata` は private のまま）。33 §16.7 と 23 §13.3.1 にその名前を書く。`parse_z004` が `Err` の bytes は「読めない bytes」として使わない。
  - BIZ-08: `src-tauri/src/biz/daily_report_import_service/scan.rs`・`mod.rs` に手順 10 と snapshot の `plu_register_z004s` を足す（読取りと形の検査は手順 5 の関数を使う）。読む本数（backlog の P3 の (3)）は窓の中の全部のまま、絞る規則を足さない（scan の時間は未実測。L3-1 で遅さを見る）。
  - CMD-12: `src-tauri/src/cmd/daily_report_import_cmd.rs` の `scan_register_sd_with_state` に手順 2a、`RegisterSdScanResponse` に `plu_snapshot_skipped_older`。CMD-08 の `import_plu_register_snapshot` と `src-tauri/src/lib.rs` の登録（2 つの列）を外す。
- S4 SD を読んだ画面の知らせと invalidation（55 UI-07-D15 (8)、67 §67.10、D-052、D-160 (4)）: `src/features/daily-report-import/` の `types.ts`・`reducer.ts`・`hooks/useDailyReportImportFlow.ts`・`DailyReportImportPage.tsx`（`tidy_failure` の知らせ〈(7)〉と同じ表示の形）。`src/lib/invalidation-contract.ts`・`src/test/invalidation-oracle.ts`・`src/lib/invalidation-contract.meta.test.ts`: C17 を「SD の読取りの成功で slot summary と未反映一覧を stale にする」に置き換え、C19 を外す（番号は詰めない）。`docs/UI_TECH_STACK.md` の D-052 の段落の件数は実装の時点で測り直す。
- S5 UI-08（67 UI-08-D5・D11、§67.4 手順 3、§67.8.1）: `src/features/plu-export/PluExportPage.tsx` から FilePicker の step・件数の要約・上限の警告・解除待ちの説明を外し、最終照合日時の 1 行と未照合の導線にする。復帰状態の旧い形（`overLimitWarning`・`row_kind`）は schema 不一致として捨てる。
- S6 段階導入の道具の撤去（30 §4.9.1、40、20、50 UI-01a-D10〜D12、D-117 (2)）: `product_service.rs` の `bulk_set_plu_target`、`src-tauri/src/cmd/product_cmd.rs` と `lib.rs` の登録、`product_repo.rs` の `find_products_for_bulk_plu_target`・`PluMigrationFilter`・`ProductSearchQuery.plu`・`ProductBulkFilter`、`src-tauri/src/biz/mod.rs` の再 export、`src/features/products/` の `ProductListPage.tsx`・`components/ProductTable.tsx`・`components/PluBulkTargetConfirmDialog.tsx`（削除）・`search.ts`・`lib/return-to.ts`・`priceRevisionSearch.ts`。URL に残った `plu` は無視する。
- S7 「レジメモリNo.」の撤去（51 UI-01b-D19、D-117 (5)）: `product_repo.rs` の `plu_memory_no` の JOIN、`src/features/products/ProductFormPage.tsx`、`plu_memory_no` を持つ test の fixture（`rg -l 'plu_memory_no' src` で全件）。
- S8 `plu_dirty`（30 §4.4 手順 4b・4c、D-160 (5)）: `update_product` で名称・部門・税率の変更を足す。最初の commit は red の test（今の code が立てないこと）。JAN の変更は商品 CSV の上書きが今も `plu_dirty=1` にする（変えない。test で固定する）。
- S9 restore の後処理（33 §16.8、D-160 (7)）: `src-tauri/src/mnt/restore.rs` の committed の後処理（`cleanup_committed`。直後と次回起動の持ち越しの両方が通る）で `plu_register_snapshot_at` だけを消す。消せなければ committed の manifest を残して次回起動へ持ち越す（監査記録と同じ扱い）。`src-tauri/src/db/system_repo.rs` に key を消す関数は無い（`get_setting`・`get_all_settings`・`upsert_setting` だけ）ので、要る 1 つだけを足す。71 §71.7 に手順と判断の ID を足す。
- S10 L3 の入力物: `src-tauri/src/io/l3_fixtures.rs` の `write_sd_bulk_l3_fixtures` と同じ形（env の `INVENTORY_L3_DAY`・`INVENTORY_L3_OUT`、ignore の test）で、全枠の Z004（5,000 行、CP932・CRLF）を持つ合成の SD の folder を作る。
- S11 test: 下の Test Plan。撤去した機能の test（clear 行・解放・一括の対象化・移行の絞り込み・件数の要約）は、D-117 の設計の改訂を理由に、新しい契約の test へ書き換えるか消す（消す test の名前と理由を PR に並べる）。`src-tauri/tests/design_compliance_test.rs` は対応表に変更が要るときだけ触る。
- S12 設計正本と docs（同じ PR）:
  - 「runtime の状態」の注記・未実装の印・撤去する signature を消す: 33（冒頭・§16.6・§16.8）、plu-tables、25、23 §13.3.1、30（§4.4・§4.9.1）、37 §37.9、45 §45.6a、55 UI-07-D15 (8)、40、41（§17.6 ほか）、20、50、51、67、task-specs の BIZ-01・BIZ-04・IO-04・CMD-01・CMD-08・UI-01a・UI-01b・UI-08、`docs/SCREEN_DESIGN.md`、`docs/DB_DESIGN.md` D-2、`docs/spec/requirements.md` の REQ-907 の注、`docs/project-memory.md`。
  - PR #292 の P3 の 4 件: 67 UI-08-D5 の Why を D-160 の Guarantee range への参照に、55 UI-07-D15 (8) の最後の 2 文の出所（この lane の文、D-127）、`docs/SCREEN_DESIGN.md` の SD の読取りの知らせの要約に (8)、D-160 (7) の穴（下の Contract Ledger の行）を D-168 に。
  - `docs/decision-log.md` の末尾に D-168（予約済み）: D-052 の C17 の置換えと C19 の撤去、精算回数の読取りと読む本数、restore の後処理の置き場と D-160 (7) の穴、migration の形。D-052 に Superseded in part の行（merge 済みの D の本文は書き換えない）。
  - `docs/plu-export-and-real-csv-verification.md` の PLU-21・PLU-25 に L3 の記録、`docs/diagrams/current-system.md`、`docs/inventory_system_erd.html`（`plu_slots` の列と status）。
  - 片付け（TD-765 の B）: backlog の項目を済みの形にし（関連の「廃番表示 3 択…」「PLU slot 後続 follow-up」の runtime への参照も）、`docs/Plans.md` の関門 14 の「前提の lane は … 未起票」を今の形にする。backlog の P3 の (1)（書き出す前にその日の SD を読む、の手順書の文）は O2 の答えの置き場へ送る。
- 生成物: `src/lib/bindings.ts`（`cargo run --bin generate_bindings`）、`docs/function-design/90-traceability.md`（`cargo run --bin generate_traceability`）。`src/routeTree.gen.ts` は route を足さないので変わらない見込み。

### 並走の lane と重なる file の所有

どちらが先に merge しても、後の側が `origin/main` の単段の同期の後に L3 をする。生成物は手で解かず、取り込んだ後に作り直す。

| 重なる file | この lane が持つ行・節 | 相手 |
|---|---|---|
| `src-tauri/src/db/migration.rs`・`db/mod.rs`、22 の末尾の節 | この lane の `schema_v<N>` の `mod`・`use`・`migrations()` の 1 件と 22 の 1 節。後に merge する側が版・file 名・節の番号を取り直す（22 §21 の「版の番号の取り直しと開発・試験 DB」。取り直しは runtime の変更なので L3 は同期の後） | 関門 17 |
| `src-tauri/src/biz/daily_report_import_service/`、`cmd/daily_report_import_cmd.rs`、`lib.rs` の登録、37・45・55 | scan の手順 10 と `plu_register_z004s`、`scan_register_sd` の手順 2a と response の 1 field、`import_plu_register_snapshot`・`bulk_set_plu_target` の登録の削除、37 §37.9・45 §45.6a・55 UI-07-D15 (8)。EJ の閲覧（`ej.rs`・§37.11・§45.6c・UI-07-D17）は持たない | 関門 19 |
| `src/lib/invalidation-contract.ts`・`src/test/invalidation-oracle.ts`・meta test、`docs/UI_TECH_STACK.md` の D-052 の段落 | C17・C19 の entry と、その説明の文。件数の文は後に merge する側が測り直す | 関門 19、PR #300（UI_TECH_STACK のほかの節） |
| `src/lib/bindings.ts`、`docs/function-design/90-traceability.md`、`docs/spec/requirements.md` | 生成物は作り直す。requirements は REQ-907 の行と注だけ | 関門 19 |
| `docs/decision-log.md` | D-168 と、D-052 への Superseded の行（末尾の追記は merge 順で両方を残す） | 関門 17（D-167）、関門 19（D-169）、PR #300（D-164）、PR #304（D-166） |
| 23、`docs/plu-export-and-real-csv-verification.md` | 23 §13.3.1 の入力の文、PLU-21・PLU-25 の行。`z004_parser.rs` は触らない | PR #304（23 §13.4.2・§13.5、Z004 の 8 桁の行） |
| `docs/backlog.md` | 「PLU の書出しの見直しの runtime」と上の関連の 2 項目 | PR #300（E-1 の項目） |

## Acceptance Criteria

- AC1（S1）: 5 つの status の行を持つ直前の版の DB で migration が成功し、`release_pending` が `external` になり、ほかの行の全列と、4,784 行の memory No. の集合が同じ。`released_at` 列と key `plu_register_snapshot_summary` が無い。同じ JAN の `external` が複数行と、`external` と `active` の同じ JAN を保存でき、`reserved` / `active` の同じ JAN の 2 行目は拒まれる。確かめの失敗で schema・値・版が何も変わらない（T1〜T4）。
- AC2（S2）: 対象外化・廃番化・JAN の変更の後も枠の状態が変わらず、Diff / Full の file にその枠の行も clear 行も無い。対象へ戻すと同じ memory No. を使う（`cd src-tauri && cargo test --lib` の T10〜T12・T23 が PASS）。
- AC3（S3）: 精算回数が最大の 1 本だけを照合し、key が無い・より大きいときだけ照合する。等しい・小さい・key が読めない・使える Z004 が無いときは枠・app_settings・操作ログを何も変えない。33 §16.3 の表の全部の行が test にある（`cargo test --lib` の T5〜T9 が PASS）。
- AC4（S3）: `scan_register_sd` は照合の `Err` でも成功を返し、`SkippedOlder` のときだけ `plu_snapshot_skipped_older` が `true`。cache の snapshot に Z004 の bytes が残らない（T14・T15）。
- AC5（S4・S5）: 知らせの文は `true` のときだけ出て、読み直すと消える。SD の読取りの成功で slot summary と未反映一覧が stale になる。PLU 書出しの画面は未照合で warning と導線、照合済みで日時の 1 行（T20〜T22）。
- AC6（S6・S7）: 商品一覧に「PLU」列・PLU表示・一括操作が無く、URL の `plu` を無視する。商品フォームに「レジメモリNo.」が無い（T17〜T19）。
- AC7（S8）: T13 は実装の前に red（`cd src-tauri && cargo test --lib -- test_update_product_req907_name_department_tax_change_sets_plu_dirty` が exit 101）、実装の後に exit 0。原価だけ・単位だけ・同じ値の更新では立てない。
- AC8（S9）: restore の後に `plu_register_snapshot_at` が無く、`plu_register_snapshot_settlement_no` は backup の値のまま。書出しは `register_snapshot_required` で拒まれる。後処理が失敗したときは次回起動で消える（T16）。
- AC9: Test Plan の mutation（M1〜M10）を 1 つずつ注入すると、名指しの test が red（Rust は `cargo test --lib -- <test 名>` が exit 101、frontend は `npm test -- <test の file>` が exit 1）になり、戻すと exit 0。
- AC10（撤去の検査）: `rg -n 'release_pending|released_at|PLU_CLEAR_ROW_ENABLED|release_plu_slot_for_jan|bulk_set_plu_target|bulkSetPluTarget|find_products_for_bulk_plu_target|PluMigrationFilter|over_limit_warning|PluPreparedRowKind|PluExportRowKind|plu_memory_no|plu_register_snapshot_summary|importPluRegisterSnapshot' src src-tauri/src src-tauri/tests` の当たりが、`schema_v5.rs`・この lane の `schema_v<N>.rs`・`migration.rs` の v5 の test・UI-08 の旧い復帰状態を捨てる test だけ。`rg -c 'runtime の状態' docs --glob '!docs/archive/**' --glob '!docs/plans/**' --glob '!docs/research/**' --glob '!docs/decision-log.md' --glob '!docs/backlog.md'` が 0 件（起票の時点の `1edcedc3` は 6 file で 11 行: biz-task-specs 3・cmd-task-specs 2・io-task-specs 1・ui-task-specs 3・plu-tables 1・33 1。どれも PLU の注記）。
- AC11（gate）: `docs/DEV_WORKFLOW.md` Verification Gates の Rust・bindings・frontend・traceability・docs の command が全部通り、`bash scripts/doc-consistency-check.sh` が ERROR 0。`cargo check --release` が通る（L3 の前）。

## Design Readiness

- 引用する設計正本: 33 §16（全節）、plu-tables、25 §12、23 §13.3・§13.3.1、30 §4.4（4b・4c）・§4.9.1、37 §37.9（手順 10）、45 §45.6a（手順 2a）、55 UI-07-D15 (8)、67（UI-08-D5・D11、§67.4・§67.8.1・§67.10）、40、41 §17.6、20、50 UI-01a-D10〜D12、51 UI-01b-D19、71 §71.7、22 §21（作り直しの形の先例）、`docs/UI_TECH_STACK.md` の D-052、decision-log の D-052・D-072・D-073・D-117・D-160。
- 必要な設計成果物: 振舞いは existing sufficient（D-117・D-160 が正本に入れた）。updated in this PR は S12 と、22 の新しい節、71 §71.7 の後処理、D-168。
- plan にしかない durable な判断の昇格先: 精算回数の読取りに `parse_z004` を使う・窓の中を全部読む → 33 §16.7・37 §37.9 手順 10・D-168。C17 の置換えと C19 の撤去 → D-168 と `docs/UI_TECH_STACK.md`。restore の後処理の置き場 → 71 §71.7 と D-168。migration の形 → 22 の新しい節。
- 前提・制約と延期した design gap（計画の review で確かめる）:
  - PR #292 の Residual: D-160 の wire（`PluRegisterSnapshotRefresh`、`plu_snapshot_skipped_older`、restore の後の key）は確定済みとして写さず、計画の review の対象にする。
  - restore の後処理の S9 の形（committed の後処理に置く、失敗は持ち越す）はこの起票の案で、71 に無い。best-effort で manifest を解放する枝（監査記録を書けなかったとき）でも key を消すかを 71 で決める。
  - migration を作り直さない代案: status の CHECK を 5 値のまま・`released_at` 列を残し、`UPDATE` と index の作り直しと key の削除だけにすれば、行のある表の作り直しが無くなる。plu-tables は作り直しと決めているので Scope は正本のまま。採るなら plu-tables と D-117 (1) の文の改訂が要り、Risk の理由の (1) が変わる（(2) の restore は残る）。
  - 本番の DB は無い（`docs/project-memory.md`）。migration の対象は開発・demo・試験の DB と owner の PC の DB。
  - scan で読む Z004 の本数と時間は未実測。
- 絶対保証の例外と抜け道: 「アプリは枠を消さない・使い回さない」の例外は 33 §16.9 のとおり、照合でレジ側に別のコードを観測したときだけ。「古い SD で照合しない」は、restore の後は backup の時点の精算回数より大きく restore の前の最後の照合より小さい SD を照合しうる（Contract Ledger の D-160 (7) の行）。レジの精算回数が戻ると照合が止まり、自動の脱出は無い（D-117 の Revisit）。
- 判定: ready（計画の review の前提として）。上の gap は計画の review の裁定で閉じ、Plan Commit の前に packet へ入れる。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| MNT-03-D9 の後の migration（D-117 (1)） | plu-tables の migration、22 の新しい節 | S1 | T1〜T4 | 非対象（L3 は空の DB。行のある DB は T1〜T4） |
| BIZ-04-D3 / SPEC-PLS-D2 照合の表 | 33 §16.3 | S3 | T7・T8 | L3-4 |
| D-117 (3) 精算回数の選択と key | 33 §16.3 手順 1〜3 | S3 | T5・T6 | L3-2 |
| gate `register_snapshot_required` | 33 §16.3・§16.9 | S3・S5 | T9・T20 | L3-1 |
| BIZ-04-D4 prepare（sticky・clear 無し） | 33 §16.4 | S2 | T10・T12 | L3-4 |
| BIZ-04-D5 confirm | 33 §16.5 | S2（clear の confirm を外す） | T10 と既存の confirm の test | L3-4 |
| BIZ-04-D6 枠を解放しない | 33 §16.6、30 §4.4 4b | S2 | T11 | 非対象（BIZ） |
| IO-04-D2・D4（IO-04-D3 の撤去） | 25 §12 | S2 | T23 | L3-4（CV17 の受理） |
| BIZ-08 手順 10 | 37 §37.9 | S3 | T14 | L3-1 |
| CMD-12 手順 2a・`plu_snapshot_skipped_older` | 45 §45.6a | S3 | T15 | L3-2 |
| CMD-08-D4 の撤去・summary の wire | 41 §17.6 | S3 | T20、bindings の差分 | 非対象 |
| UI-07-D15 (8) | 55 | S4 | T21 | L3-2 |
| D-052 C17・C19 | `docs/UI_TECH_STACK.md`、67 §67.10、D-168 | S4 | T22 | 非対象 |
| UI-08-D5・D11 | 67 | S5 | T20 | L3-1 |
| BIZ-01-D4・CMD-01-D4・UI-01a-D10〜D12 の撤去 | 30 §4.9.1、40、20、50 | S6 | T17・T18 | L3-3 |
| UI-01b-D19・IO-01-D4 の `plu_memory_no` | 51、20 | S7 | T19 | L3-3 |
| D-160 (5) `plu_dirty` | 30 §4.4 4c | S8 | T13 | L3-3 |
| D-160 (7) restore の後の key | 33 §16.8、71 §71.7 | S9 | T16 | 非対象（restore の実機の確かめは足さない。71 の既存の L3 の範囲を変えない） |
| D-160 (7) の穴（PR #292 の P3 の 4） | D-168 の Guarantee range | 直さない: restore の後、backup の時点より大きく最新より小さい精算回数の SD を読むと古い占有で照合する。締めで今の SD を読む普通の流れでは起きない | — | 非対象（受け入れる限界として書く） |
| D-073 L3 の適用範囲 | decision-log D-073、67 §67.12 | L3-4・L3-5 | — | L3-4・L3-5 |
| 隣の契約（除外）: BIZ-03 の Z004 の取込み（32）、D-122 の別名、停止 ADR | 33 §16.3 の末尾 | 触らない | — | 非対象 |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| migration | `release_pending` が残る・枠の列が変わる | integration | T1 `test_migration_req907_plu_slots_rebuild_maps_release_pending_to_external` | status の対応づけを外す（M1）・列を取り違える |
| partial UNIQUE | `external` の重複を保存できない・管理の重複を許す | integration | T2 `test_migration_req907_plu_slots_rebuild_unique_only_reserved_active` | index の条件に `external` を戻す（M2） |
| migration の確かめ | 失敗の後に半端な schema・版が残る | integration | T3 `test_migration_req907_plu_slots_rebuild_verify_failure_rolls_back` | 確かめを外す・TX の外で版を書く |
| key の削除 | `plu_register_snapshot_summary` が残る | integration | T4 `test_migration_req907_plu_slots_rebuild_drops_summary_key` | 手順 4 を落とす |
| 最大の 1 本 | 古い Z004 を選ぶ・同じ精算回数で bytes の違う 2 本で照合する | unit | T5 `test_refresh_plu_register_snapshot_from_sd_req907_picks_max_settlement_no` | 最初の 1 本を選ぶ（M4） |
| key の比較 | 等しい・小さい・読めない key で照合する（negative: 何も変えない） | unit | T6 `test_refresh_plu_register_snapshot_from_sd_req907_key_comparison` | 比較を `>=` にする（M3） |
| 照合の表 | 表の行の結果が違う（重複 → `external`、対象なしの別コード → `external`、照合の前の状態で判定） | unit（表の行ごと） | T7 `test_import_plu_register_snapshot_req907_reconciliation_table` | 対象なしの別コードを conflict に戻す（M5） |
| 照合の TX | 失敗で枠・key・ログの一部が残る | unit | T8 `test_import_plu_register_snapshot_req907_rolls_back_settings_log_and_slots`（既存を新しい key に合わせる） | key の更新を TX の外に出す |
| gate | 未照合で予約できる | unit | T9 `test_prepare_plu_export_req907_requires_snapshot_without_writes`（既存。文言を合わせる） | gate を外す |
| clear 無し | 対象を持たない JAN の枠・clear 行が出る | unit | T10 `test_prepare_plu_export_req907_never_emits_rows_for_untargeted_slots` | prepare に対象外の枠の行を足す（M10） |
| 枠を解放しない | 対象外化・廃番化・JAN の変更で枠が変わる | unit | T11 `test_product_service_req907_keeps_slot_on_untarget_discontinue_and_jan_change` | 解放の呼出しを戻す |
| sticky | 再対象化で別の枠を取る | unit | T12 `test_prepare_plu_export_req907_retarget_reuses_same_memory_no` | 既存の枠の再利用を外す |
| D-160 (5) | 名称・部門・税率の変更で未反映にならない／原価・単位だけで立つ | unit（red から） | T13 `test_update_product_req907_name_department_tax_change_sets_plu_dirty` | 条件から税率を外す（M6） |
| 手順 10 | 窓の外・読めない Z004 を渡す、同じ bytes を重ねる、DB へ書く | integration（合成の SD の tree） | T14 `test_scan_register_sd_req907_collects_window_z004_bytes` | 形の検査を飛ばす |
| 手順 2a | 照合の `Err` で scan が失敗する・flag が違う・bytes が cache に残る | unit（CMD） | T15 `test_scan_register_sd_req907_refreshes_snapshot_and_reports_skipped_older` | flag を固定にする（M7） |
| restore の後 | 2 つの key を消す・消さない、持ち越しで消えない | integration | T16 `test_restore_backup_req907_clears_snapshot_at_keeps_settlement_no` | `settlement_no` も消す（M8） |
| search の状態 | URL の `plu` で絞り込む・戻り先に `plu` を運ぶ | frontend unit | T17 `search.test.ts`・`lib/return-to.test.ts` の `plu` を無視する case | schema に `plu` を残す |
| UI-01a | 列・絞り込み・一括操作が残る | frontend | T18 `ProductListPage.test.tsx`・`components/ProductTable.test.tsx` | 列を残す |
| UI-01b | 「レジメモリNo.」が残る | frontend | T19 `ProductFormPage.test.tsx` | 表示を残す |
| UI-08 | 未照合で書出しへ進める、step・要約が残る、旧い復帰状態で壊れる | frontend（initial / pending / success / failure / retry / revisit / restart） | T20 `PluExportPage.test.tsx` | 未照合の分岐を外す |
| UI-07-D15 (8) | `false` で出る・`true` で出ない・読み直しても残る | frontend（success / refetch） | T21 `DailyReportImportPage.sd.test.tsx` | 表示の条件を反転する（M7 の UI 側） |
| D-052 | SD の読取りの後に summary・未反映一覧が古いまま | frontend（invalidate / refetch。oracle は独立に転記） | T22 `invalidation-contract.meta.test.ts` と scan の成功の handler の test | 集合から `pluDirty` を外す（M9） |
| IO-04 | clear 行の形が残る・product 行が変わる | unit | T23 `plu_formatter.rs` の既存の product 行の test（clear 行の test は消す） | 列・桁を変える |

mutation（実装の run で 1 つずつ注入して red を確かめ、戻す）: M1〜M10 は表の「Would fail if...」の括弧のとおり。

## L3

D-073 の (1)・(3) に当たる（PLU file の形と UI-08 の operator flow）。L3-1〜L3-3 は開発用の Windows の native build・空の DB と S10 の合成の SD の folder（CP932・CRLF、「場所を選ぶ」で選ぶ）で行い、L3-4・L3-5 は CV17 と SR-S4000 の実機で行う（場と書く商品は O1）。lane の runtime の変更が止まり、並走の lane との同期が済んだ後に頼む。

- L3-1 PLU 書出し / 空の DB で PLU 書出しを開く → 売上データ取込みで `sd-a` を読む → PLU 書出しへ戻る / 読む前は warning と売上データ取込みへの導線が出て書出しへ進めない。読んだ後は「レジの登録状況: {日時}」の 1 行が出て、Z004 を選ぶ step と件数の要約が無い。SD の読取りが待てる速さ。
- L3-2 売上データ取込み / 続けて `sd-b`（精算回数が `sd-a` より小さい Z004 だけ）を読む / 読み終えた文の直下に、照合しなかったことと理由の文が出る。日報の一覧（か「新しい精算はありません」）は今までどおり出る。文が自然に読める。
- L3-3 商品一覧と商品フォーム / 一覧を開き、商品を 1 つ修正して名称を変える / 一覧に「PLU」列・PLU表示・PLU 一括操作が無い。フォームに「レジメモリNo.」が無い。保存の後、ホームの未反映の知らせにその商品が出る。
- L3-4（PLU-21） CV17 とレジ / 少数の商品を Diff で書き出し、CV17 で取り込み、SD 設定書込み → レジで設定読込み → 次の精算の後の SD を読む / CV17 が file を受ける。書いた商品がレジで呼び出せ、file に無い枠の登録が保たれる。SD を読んだ後、その商品が未反映に戻らない。
- L3-5（PLU-25） CV17 / 書き出した file を取り込まずに、同じ mode で書き出し直す / 書き出し直した file の memory No. が前と同じで、未反映の一覧に商品が残る。書き出し直した file を取り込める。

## Recovery

- migration の失敗: 1 つの TX の中なので、この migration の変更と版は残らない（22 §21 の「止まった後の進め方」と同じ扱い）。直した build で同じ DB を開けば当たり直す。
- migration の COMMIT の後: 前の版のアプリでは開けない（`SchemaNewerThanApp`）。戻すのは更新の前の backup（22 §16 の「回復」、`docs/migration-recovery-runbook.md`）。migration が捨てるのは `released_at` の値と、`release_pending` と `external` の区別で、この後の code はどちらも読まない。
- code の revert: squash commit の revert では DB は戻らない（上の backup）。版の番号を取り直した build で開く前の扱いは 22 §21 の「版の番号の取り直しと開発・試験 DB」。
- restore の後処理の失敗: committed の manifest が残り、次回起動で消し直す。key の削除は何度行っても同じ結果。
- レジへ書いた PLU（L3-4・L3-5）は DB を戻しても戻らない。書く前に SD の設定 file（`AUTOPGM\FILE092P.092`）を PC へ写す（D-160 (2)）。写しから戻す手順を実機で試した記録は無い（PLU-05）。

## Data Safety

- commit しないもの: 店の SD の file・実 Z004・`FILE092P.092` の写し・店の DB・backup・書き出した PLU の file。L3-4・L3-5 で書いた商品の名前・JAN・値段は repo に書かず、記録は件数と匿名の枠の挙動だけ（PLU-21 の証跡の形）。
- 合成だけの path: S10 の入力物は `INVENTORY_L3_OUT` の folder へ手元で作り、tracked にしない。test の Z004 と SD の tree は合成の値。
- 操作ログ（`plu_register_snapshot_import`）は件数だけで、実コード・名称・単価を入れない（33 §16.3）。
- migration は数量・在庫・売上・商品を変えない。写しの確かめ（S1）が通らなければ何も変えない。

## owner に決めてもらう事項

- O1 L3-4・L3-5 の場と書く商品。A: 次の訪店で、開発用の PC から、店で実際に売る商品の少数を書く（確かめた 3 部門の商品だけ。書いた枠は本番の登録として残り、本番の DB は後の SD の照合で引き継ぐ）。B: 合成の JAN で書く（レジに使わない枠が残る。アプリは枠を消さず、レジの取説に枠を消す操作は無い〈D-117 の Why〉）。C: この lane では実機の確かめを行わず、初回の本番の書込みで PLU-21・PLU-25 を取る（D-117 (7) と D-073 の改訂が要る）。推奨は A（candidate。部門リンクの決め方が未決でも、3 部門は CV17 の取込みで受理を確かめてある〈backlog の「PLU の部門リンクの値と 21 個目の部門」〉）。
- O2 初回の本番の書込みの手順書（D-160 (2) は「この lane か手順書の改訂で決める」）。A: この lane に入れず、部門リンクの決め方の後の docs の lane で `docs/initial-load-runbook.md` の流れ 3 に書く。B: この lane の PR に入れる。推奨は A（candidate。手順は部門リンクの決め方に依り、この lane の範囲の外）。
