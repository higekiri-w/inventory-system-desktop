# Plan Packet: 棚卸しの数の置き場と在庫数の求め直しの runtime (b1)（計数の保存と完了、Z004 の確定と取消の新しい経路、停止の入口の解除と旧本体・時点証拠の撤去）

wave に属さない単独の lane。D-118（[ADR](../../adr/2026-10-16-stocktake-count-placement.md)、design lane = PR #167）の runtime の 2 つ目の配布の単位 (b)（ADR SPEC-STK-CNT-D12。(a) は PR #176 で 2026-10-19 に merge）を 2 つの lane に分けた最初の lane（b1）。導入の前の関門 2（[Plans](../../Plans.md#導入の前の関門) の 2.、D-070 で v1.0 の条件）。owner の決定（2026-10-22。repo 外の回答台帳 TD-372）: 次の lane に (b) と PR #155（backup の外の控え）を並走で選び、docs の棚卸し lane 2b は両方の merge の後に直列。本 commit は Plan Packet と Test Design Matrix の起票と、plan-first の正本の修正だけで、runtime の実装は plan-approved の後の別の run。branch は `agent/stocktake-count-runtime-b`（main `c98fe7dd` から）。新しい決定の番号（D-129）は使わない（起票で決めたことは ADR・設計正本の該当の節に置いた。下の「起票時に起草役が決めたこと」）。

## 規模と手法

**手法: lane を 2 つに分け（b1 = 停止を外す単位、b2 = 要再確認の範囲と計数の履歴を商品の画面で見せる単位）、b1 は 1 lane・1 PR のまま S1〜S9 の段に積む。横断の約束は下の「横断の約束」の 1 つの表に先に置く。** 本 packet が起票するのは b1 だけで、b2 は下の「後続の lane」に範囲を書く。境は ADR SPEC-STK-CNT-D12 の「(b) を 2 つの lane に分けた」に置いた（本 commit）。

2026-10-22 に本 worktree（HEAD `c98fe7dd`）で数えた（command は repo の外の scratch の script に置き、出力をここに写した）:

| 区分 | (b) 全体 | うち b1 | command と出力 |
|---|---|---|---|
| 触る正本 | 47 file（(b) の proposed の節を持つ 36、停止 ADR を引く 19、時点証拠 ADR を引く 11 の和集合）+ 22（新しい §20）+ 10・36・75 | 下の Scope の S9 の一覧（b2 の 4 つの UI の文書と 20・40 の b2 の項を除く） | `rg -l '\(b\) は proposed\|proposed・未実装、D-118\|\(b\)（proposed・未実装' docs --glob '!docs/archive/**'` → 36、`rg -l 'legacy-stocktake-z004-write-stop\|一時停止中\|SPEC-STOP-D' docs --glob '!docs/archive/**'` → 19、`rg -l 'stocktake-time-evidence\|SPEC-STK-TIME' docs --glob '!docs/archive/**'` → 11、和集合 47 |
| 画面 | 9（73 棚卸し、55 Z004 タブ、65 棚卸しの記録の詳細、51 商品フォーム、60 商品一括 import、75 整合性、50 商品一覧、58 在庫照会、66 在庫変動） | 6（73・55・65・51 の拒否と回復の導線・60 の行の error・75） | `rg -l '計数の置き場と在庫の求め直し' docs/function-design/5*.md docs/function-design/6*.md docs/function-design/7*.md` → 13 file（61〜64 は日付の欄の意味の注記で画面の変更なし、66 は b2）。SCREEN_DESIGN の D-118 の節の画面の link は 73・55・51・58・66 |
| migration | 1（vZ、[22 §20](../../function-design/22-mnt-migration.md) MNT-03-D18） | 1 | `ls src-tauri/src/db/schema_v*.rs` → v1〜v10。`rg -n 'version: [0-9]+' src-tauri/src/db/migration.rs` → 最後は `version: 10` |
| wire の型 | b1 で形が変わる・外れる今の型 17 + 新しい型（下の Boundary）、b2 で形が変わる型 3（`ProductWithRelations`〈`search_products`・`list_low_stock` の要素〉・`StockDetail`・`MovementRecord`。`rg -c '^export type (ProductWithRelations\|StockDetail\|MovementRecord) =' src/lib/bindings.ts` → 3） | 17 + 新しい型 | `rg -c '^export type (CmdError\|CmdErrorKind\|StartStocktakeResult\|StocktakeItemDetail\|StocktakeResult\|AdjustedItem\|UpdateCountResult\|StocktakeRecordDetail\|StocktakeRecordDetailItem\|PreviewData\|DuplicateCheck\|DuplicateStatus\|SameDateCsvImportSummary\|ImportResult\|RollbackResult\|StockCorrection\|IntegrityResult) =' src/lib/bindings.ts` → 17 |
| Tauri command | b1: 新設 4・外す 1（`update_count`）・入出力が変わる 9。b2: 入出力が変わる 5 | 14 | `rg -c '^\| (save_stock_count\|preview_stock_count_placement\|zero_uncounted_items\|list_recheck_products) \|' docs/function-design/42-cmd-sales-stocktake.md` → 4。`rg -c '^        cmd::(stocktake_cmd\|csv_import_cmd\|integrity_cmd)::' src-tauri/src/lib.rs` → 15（棚卸し 8・Z004 5・整合性 2。変わる 9 = `start_stocktake`・`get_stocktake_items`・`find_stocktake_item`・`get_stocktake_record`・`complete_stocktake`・`parse_and_validate_csv`・`commit_csv_import`・`rollback_csv_import`・`run_integrity_check`）。b2 の 5 は `rg -c '^        cmd::(product_cmd::(get_product\|search_products)\|inventory_cmd::(get_stock_detail\|list_low_stock\|list_movements))' src-tauri/src/lib.rs` → 5 |
| 横断の約束がかかる経路 | b1: 13、b2: 2 | 13 | 下の「横断の約束」の表の「経路」の列（計数の保存〈既定・fresh〉・0 にする・完了・記録の詳細の読取り・要再確認の一覧・Z004 の preview・確定・取消・商品の登録と修正と一括 import の共有 JAN・在庫連動を始める検査・整合性の検査・migration vZ） |
| 撤去 | 停止の入口 5、旧本体 `legacy_*` 5、時点証拠の schema と code（`#[cfg(test)]`）14 file、probe 1 | 全部 | `rg -n 'fn legacy_(start_stocktake\|update_count\|complete_stocktake\|commit_csv_import\|rollback_csv_import)\(' src-tauri/src \| wc -l` → 5。`rg -l 'time_evidence\|stock_revision\|pos_import_sources\|stocktake_recount\|observation_kind\|stocktake_adjustment_kind' src-tauri/src scripts \| wc -l` → 14。撤去する file の行数の和（時点証拠の 5 file・停止の test・probe）→ 6100 |

手法の選び方（3 つの案を現物に当てた）:

- 1 lane のまま段に分ける: (a) の 40 file より大きい（b1 だけで予定 path 148 = Rust 60・TS 36・docs 50〈packet と Matrix を含む〉・fixture と probe 2。round 1 の是正で Rust の 11、round 2 の是正で Rust 4・TS 5、round 3 の是正で TS 1 を足した。下の「予定 path の一覧」。b2 を足すと Rust と TS がさらに増える）。横断の約束は (a) と同じく 1 つの表に置けるが、表示だけの変更（b2）を含めると、R4 の Final Review の Double Audit と L3 の項目が、停止を外す安全と関係の無い画面へ広がる。
- 発注の例の 3 つの lane（b1 計数と停止の解除、b2 Z004 の新しい経路、b3 旧本体と時点証拠の撤去）: 当てると 2 か所で配布の単位として成り立たない。(1) 停止 ADR の SPEC-STOP-D6 と owner の決定（TD-296「停止は一括で外す」）は 5 つの入口を一度に外すことを求めるので、棚卸しの入口だけを外して Z004 の入口を残す版は作れない（部分的な解除）。(2) ADR D12 の「(b) へ回すもの」の Z004 と phase を結ぶ CHECK（`inventory_movements` の 2 回目の作り直し）は、旧本体の fixture（`src-tauri/src/biz/csv_import_service/test_support.rs` の `make_z004_bytes` は精算回数を持たない）と時点証拠の test の fixture（`src-tauri/src/db/test_support.rs` の `seed_legacy_fixture` は `reference_type` が `csv_import` で `phase` が 0 の行を入れる）を CHECK で拒むので、撤去を後の lane（b3）へ送ると、b1 の migration の後に旧本体と時点証拠の test が書けない。撤去は CHECK と同じ lane に要る。
- 採った案（2 つの lane）: 停止を外して「黙ってずれない」（owner の決定 TD-267）ために同じ版に要るのは、書く経路（計数・完了・Z004 の確定と取消）と守り（D4 の共有 JAN の不変条件と移行の前からの違反の知らせ、D8 の在庫連動を始める検査、Z004 の整数の数量の換算）と要再確認を見せる所（棚卸しの画面の一覧・取込みの結果・整合性の検査）。これを b1 に入れ、読取りの表示だけ（商品の結果の写しの 3 列と商品の画面の範囲の表示、在庫変動の履歴の計数の行、`MovementRecord` の並びの鍵）を b2 に分けた。b2 は migration を持たず R3 で済む。

各 lane の配布の単位としての安全（途中の版で店が壊れた状態にならない）:

- b1 だけが入った版: 5 つの入口が一度に外れ、新しい経路が揃う（SPEC-STOP-D6）。在庫・要再確認・評価額の記録と写しは正しい。商品一覧・在庫照会・商品フォームの在庫の欄は写しの値だけを出し（要再確認の範囲を出さない）、在庫変動の履歴に計数の行が無いので、計数を保存した前後で変動後の値が差分と合わないことがある（表示の欠けで、記録は正しい。ADR D12 に書いた）。店へ入れる版にしない（関門 2 は b2 の merge で満たす）。
- b2 が入った版: (b) の全部。初めて店へ入れる版は b2 まで含む。

関門 2. を満たす全体の順: b1（本 packet）→ b2（b1 の merge の後に起票。下の「後続の lane」）。どちらも初回の導入の前。b1 の migration vZ は開発・試験 DB の通常の形にはそのまま当たり、作り直しの起点ではない（22 §20）。並走の PR #155 は b1 より先に merge する見込みで、b1 は `origin/main` の単段 merge で同期する（下の「節の所有表」）。「Z004 を SD の一括取込みに入れる runtime（B）」は b1 の Z004 の確定の新しい経路と (A) の後。

## Workflow State

- Phase: archive
- Risk: R4
- Plan Commit: aba17e2e2422695b2ffff6bcd3fd9bebebf24339
- Amendments: 5b53294040ff38d2aeb8366aefd02142ffb2c764, bc0bf2f5d5a6a573bd76cf14e11380d8c407ddbe
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（起票。実装の Writer は plan-approved の後に Coordinator が発注で決める）
- Plan Reviewer: fresh Opus 5.5 + Codex（model は発注時に決める）。座組表どおり
- Final Reviewer: Fable 5.1（Claude 側、R3 以上）+ Codex（GPT-6.1 Sol 既定）。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Review Minimum: 2
- Human Gate: ready,merge,manual,r4
- Branch: agent/stocktake-count-runtime-b

遷移の記録:

1. kickoff → spec-check（2026-10-22、起草役）: Coordinator の発注（D-118 の (b)）を範囲にし、規模を数えて 2 つの lane に分け、本 packet を最初の lane（b1）にした（上の「規模と手法」）。Risk を R4 と記録した（下の Risk）。
2. spec-check → design（2026-10-22、起草役）: 対象の設計正本は ADR SPEC-STK-CNT-D1〜D12、停止 ADR SPEC-STOP-D1〜D6、tracking・pos・master・transaction の「計数の置き場と在庫の求め直しの契約」、[21](../../function-design/21-io-inventory-repo.md)・[24](../../function-design/24-io-csv-import-repo.md)・[30](../../function-design/30-biz-product-service.md)・[31](../../function-design/31-biz-inventory-service.md)・[32](../../function-design/32-biz-csv-import-service.md)・[35](../../function-design/35-biz-stocktake-service.md)・[36](../../function-design/36-biz-integrity-check.md)・[40](../../function-design/40-cmd-product.md)・[41](../../function-design/41-cmd-pos.md)・[42](../../function-design/42-cmd-sales-stocktake.md)・[51](../../function-design/51-ui-product-form.md)・[55](../../function-design/55-ui-csv-import.md)・[60](../../function-design/60-ui-product-import.md)・[65](../../function-design/65-inventory-record-traceability.md)・[73](../../function-design/73-ui-stocktake.md)・[75](../../function-design/75-ui-integrity-check.md) の同名の節、[22 §16・§19](../../function-design/22-mnt-migration.md)、共通規則 SPEC-UNIT-D3・D4・D6・D8・D11 と「入力の上限と安全な整数の範囲」。設計の更新が要る点が 8 つあった（下の Design Readiness の「必要な設計成果物」）。
3. design → plan-draft（2026-10-22、起草役、本 commit）: 上の 8 つを本 packet と同じ commit で正本に置いた（下の「本 commit で変えた正本」）。製品の振舞いで owner の判断が要る問いは 3 つあり、どれも b1 の契約を決める問いではない（下の「owner の判断事項」。(i) が A なら b1 の Scope を広げて design へ戻す）。plan-gate へは Coordinator が進める。
4. plan-draft → plan-gate（2026-10-22、Coordinator、本 commit）: owner が判断事項 (i)〜(iii) に答えた（repo 外の回答台帳 TD-377〜TD-379。(i) B: D1 は b1 に入れず v1.0 の前のデザインの刷新で、(ii) A: 振替の `movement_type` は vZ に足さない、(iii) A: 返品・交換の `register_processed` の既定と TD-225 は b1 の外の小さな lane）。どれも推奨どおりで、b1 の Scope・契約は起票のまま。packet と Matrix が揃い commit されている。Plan Review round 1（fresh Opus 5.5 + Codex）に出す。
5. plan-gate のまま是正（round 1、2026-10-22、Coordinator）: round 1 の reject を起草役が `9adc7047`・`097d319c` で直した（下の Review Response）。設計の方向は変えないので plan-gate に留め、round 2 で再 review する（`docs/DEV_WORKFLOW.md` Workflow State「a plan-gate rejection corrected in place stays at plan-gate」）。owner が過去の取込みを記録から取り消す入口を導入の前の関門に入れた（repo 外の回答台帳 TD-380。b1 の範囲は変えない）。
6. plan-gate のまま是正（round 2、2026-10-22、Coordinator）: round 2 の reject を起草役が `1688ccb9` で直した（下の Review Response）。設計の方向は変えないので plan-gate に留め、round 3（上限）で再 review する。
7. plan-gate（round 3、上限、2026-10-23、Coordinator）: round 3 は両者 reject（Codex に P1 1）。`docs/DEV_WORKFLOW.md` Review Rules の round 天井に達したので round 4 は回さず、残った findings を disposition「同型指摘の一括是正」とした。起草役が `8655cb64` で直した（下の Review Response）。plan-approved は、fresh の reviewer の closure（Claude 側 + Codex）が P1 / P2 = 0 を報告し、owner が実装の前の R4 を承認した後。
8. plan-gate → plan-approved（2026-10-23、Coordinator、本 commit）: 独立の Plan Reviewer（Writer と別の context）のうち Claude 側の fresh Opus 5.5 が closure で P1・P2 が無いと報告した（Review Response 参照）。Codex の closure は利用上限で 2026-10-24 まで得られず、`docs/AGENT_OPERATING_MANUAL.md` §3.3・座組表の Codex 停止中の列（Plan Review は fresh Opus 5.5 だけで進める）に従い、owner の決定（repo 外の回答台帳 TD-389。TD-387 を見直した）で進めた。closure の P3 4 件は plan-gate のまま起草役が `5c45d253`・`aba17e2e` で直した（Plan Commit は是正の後の版）。owner は同じ回答で実装の前の R4 を承認した（TD-389）。実装の commit はまだ無い。
9. plan-approved → implementing（2026-10-23、Coordinator、本 commit）: main（`eed07d37`、PR #155・#189 の後）を単段で取り込み（衝突は `docs/Plans.md` の 1 か所）、AC の数の baseline の command 21 本を逐語で再測して上の値と一致した（`rg` の出力の並びだけが違う）。実装の Writer に S1〜S7 と S9 の L3 の fixture の生成から発注する。
10. implementing のまま Gated Amendment（2026-10-23、Coordinator、本 commit）: closure の Codex 側（owner の決定〈repo 外の回答台帳 TD-389〉で plan-approved の後に流した）が P1 1 件を出した（下の Review Response）。Coordinator が受理し、起草役の直し方の案 1（fresh な計数の精算の判定を取込み ID のカーソルで求める）と owner の答え（TD-400: 取り込み直した精算は要再確認）を `5b532940` に置いた。packet の契約が変わるので Gated Amendment として `Amendments` に登録し、amendment の review（fresh Opus 5.5 + Codex）に出す。Coordinator は review の前に code の実装（1 段目の上の commit）まで同じ発注で進めさせた。review で契約が変われば code も直す。
11. implementing のまま Gated Amendment（2 本目、2026-10-24、Coordinator、本 commit）: 1 本目の amendment review の findings を起草役と Writer が直し、取り込み直しの判定を D3 の表に合わせ、検索で開く欄の文を「入れ直す」へ導く形にした（`bc0bf2f5` が packet・ADR・32・73・Matrix、`a6f6cefc`・`0c0dca85` が code と test）。packet の契約が変わるので `bc0bf2f5` を `Amendments` に登録する。2 本目は closure を立てず、Final Review broad（Claude 側 + Codex）で見る（Claude 側は当面 Fable 5。owner の決定 2026-10-24）。Writer の判断 2 つ（T6 を Matrix §T5 の行に置く、欄の文を出す条件を「入れ直す」の button と同じ `canRecount` にする）は受理した。`docs/SCREEN_DESIGN.md`・`docs/FUNCTION_DESIGN.md` の UI-10 の要約に残る「上書き再入力」は変えない（正本の 73 の UI-10-D2 が b1 の注記で導線を「入れ直す」と名指しし、要約はその節を指す）。

## Owner Effort Budget

- 介入回数上限: 9（既定の 6 を上げた理由: R4 の承認〈実装の前と最後の head〉と L3 が加わり、起票の owner の判断事項が 3 つあり、owner は 2026-10-21 から数日、手が使いにくい〈左手だけ〉ので 1 回の問い合わせに束ねても decision point の数で数える）
- 実働時間上限: 40 分（既定の 30 分を上げた理由: L3 は棚卸し・Z004 の取込み・完了・記録の詳細・在庫連動の回復を 1 回の通しの台本で確かめる〈見込み 25 分〉。台本と fixture は Coordinator が Ready の依頼と同時に渡し、owner の操作を画面の入力と PASS / FAIL の返事に限る）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 9 | 7（2026-10-22: 起票の owner の判断事項 (i)〜(iii) の 3〈TD-377〜TD-379、1 回の問い合わせに束ねた〉、記録から取り消す入口を関門に入れる判断 1〈TD-380〉。2026-10-23: Codex の closure を待つ判断 1〈TD-387〉、closure の N1 の直し方 1〈TD-388〉、Codex を待たずに進める判断と実装の前の R4 の承認 1〈TD-389〉） | 3（L3 と最後の head の `r4` 1、Ready 1、merge 1） | 0（予備の 1 を TD-388 に使った） | 10 = 7 + 3 + 0（上限を 1 超える見込み。次の owner の判断〈L3 と最後の head の `r4`〉と同じ 1 回で上限の改定を諮る。`docs/DEV_WORKFLOW.md` Owner Effort Budget） |

owner の操作の回数を減らす工夫: (1) 起票の判断事項は 1 回の問い合わせに束ね、選択肢に推奨を付け、返事は記号 1 つで済む形にする。(2) L3 と最後の head の `r4` の承認を同じ 1 回で頼む（(a) と同じ）。(3) L3 の入力物（商品 CSV と合成の Z004）は S9 の tracked の生成の test から L3 の日に Coordinator が作り、L3 の DB も Coordinator が用意して差し替え（Test Plan の「L3 の DB の用意と差し替え」）、Windows の path を省略しない形で台本と一緒に渡す（owner は file も DB も作らない。round 2 の是正で生成の方式に合わせた）。(4) L3 の項目を 1 本の通しの台本に並べ、前の段の結果を次の段の入力にする（計数 → 取込み → 要再確認 → 入れ直し → 完了 → 記録の詳細）。(5) L3 Eligibility の (3)（DB の手での改変や PC の時計の変更）に当たる項目は L3 に入れず、自動 test と Coordinator が渡す画面の写しで見る（下の Test Plan）。(6) Ready と merge は owner が 1 回の返事で「Ready にして hosted が緑なら merge」と委ねられる（decision point は 2 で数える）。

plan-approved は Coordinator が進める（TD-296 の 4a）。介入のたびに、やろうとしていることが全体を顧みない対症療法になっていないかを考え直す（TD-296 の 5a）。

既定値・数え方・上限に届くときの扱いは `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R4

Reason:
migration vZ（[22 §20](../../function-design/22-mnt-migration.md)）が行のある `inventory_movements` を 2 回目に作り直し（新しい表へ全行を写して `DROP TABLE` し、名前を変え、`sqlite_sequence` を戻す）、`csv_imports`・`sale_records`・`stocktakes`・`stocktake_items` に列を足す。Risk Tiers の「Destructive data lifecycle」に当たる（先例: (a) の v9 を R4 とした）。停止 ADR の 5 つの入口を外し、Z004 の取込みで在庫を動かし、在庫数の求め直しに精算の窓を入れる（POS の取込みと在庫の求め直し）。旧本体と時点証拠の code の撤去は test だけの code で、DB の表を消さない（時点証拠の schema は registry に無く、今の DB に無い）。棚卸しか Z004 の記録のある DB は v9 で既に止まって作り直されているので、vZ は開発・試験 DB の通常の形にはそのまま当たり、作り直しを要さない（22 §20 の手順 1 の止まる検査は外側の守り）。Tauri command の DTO・generated bindings・operator の画面も変える（R3 の要素）。本番の DB は無い（`docs/project-memory.md`）。実店舗データの露出は無い。

Rollback / recovery notes: vZ は 1 つの TX で、止まる検査と確かめの失敗は全部戻り版も記録しない（22 §20 の手順 1・6）。COMMIT の後に旧版へ戻すのは、22 §16 の「回復」（更新の前の backup を管理者が置き換える。vZ の後に入れた記録は戻らない）。b1 の merge の後に不具合が見つかり停止へ戻したいときは、停止 ADR の入口を戻すのではなく修正の PR で直す（停止の入口と旧本体は撤去するので戻せない。開発・試験 DB だけで、本番の DB は無い）。

required gate の green / red: gate の定義は変えない。実装は bindings と traceability の再生成、`design_compliance_test.rs` の対応表の更新を伴い、どれかを落とすと既存の required gate（Rust・bindings・traceability）が red になる（下の Registration / Generation Obligations）。Scope の予定 path を `scripts/ci/classify-changes.sh` の workflow の一覧に当てると当たらない（`scripts/probes/stocktake_time_model.py` と `tests/fixtures/z004/` の file は workflow の一覧に無く、どの分類にも当たらないので unknown になり full の job が走る。green / red は変わらない）。Final Review Minimum 2 は R4 による。

## Goal

Goal Invariant:

### 最小完了条件

- 棚卸しの画面で、商品ごとに「数を保存」（既定の計数。日付は既定で今日、「前の日の分を入れる」で選べる）と「入れ直す」（fresh な計数）ができ、保存のたびに在庫（写し）が記録から求め直され、ADR D3 の窓の判定で要再確認になった商品が画面の上部の一覧に理由つきで出て、一覧から入れ直すと解ける（ADR SPEC-STK-CNT-D3・D5）。
- Z004 の取込みが精算回数で識別・並び・欠けの知らせを持ち（精算回数の無い file・別のレジ・同じ精算の別の版・日付と番号の順の食い違いは取り込まない）、確定で Z004 の movement を精算の鍵で書いて計数を持つ商品と取込みの商品を求め直し、取込みの結果に要再確認の一覧を出し、取消は movement と売上を無効にして求め直す（補償の movement を作らない）（ADR D1・D2・D4・D7）。
- 棚卸しの完了は、決まらない精算の未取込み・要再確認・未計数があれば理由ごとの一覧で拒み、未計数は「0 にする」で片付けられ、完了で評価額を固定し（在庫を動かさない）、記録の詳細に完了の後の差を参考として出す（ADR D6）。
- 停止 ADR の 5 つの入口が一度に外れ、旧本体 `legacy_*`・D-109 の時点証拠の `#[cfg(test)]` の schema と code・probe が repo から無くなり、STK-1・STK-2 の診断 2 本が新しい経路で PASS する回帰 test になる（SPEC-STOP-D6、ADR D12）。
- 在庫連動の商品と同じ JAN を別の商品が持つ変更をどの経路でも拒み、移行の前からある組を整合性の画面に並べ、連動なしの売上が計数に入っていない商品の在庫連動を始める変更を回復先つきで拒む（ADR D4・D8・D10）。

### 失敗定義

- 5 つの入口のどれかが停止のまま残る、またはどれかだけが外れる（部分的な解除）。
- 計数の保存・完了・Z004 の確定と取消のどれかの後に、`stock_quantity`・要再確認の印・範囲が、記録から求め直した値（`derive_stock`）と違う。
- 決まらない記録があるのに要再確認にならない、または入っている記録だけなのに要再確認になる（ADR D3 の判定表の行と違う）。
- 精算回数の無い Z004・同じ精算の別の版・順の食い違う Z004・別のレジの Z004（取り消した取込みしか無いときも）が取り込まれる。fresh な計数の「k ≤ n」の n が、保存の後の取込み（取消の後に同じ番号を使う取り込み直し・初期化の後の新しい系列・別のレジ）の精算回数から求まる。preview で確かめた引き当てと違う商品へ確定が売上か在庫を入れる。
- 長さの商品の Z004 の行が 100 分の 1 の数で在庫と売上に入る。共有 JAN の行が候補のどれかの在庫を動かす。
- 完了が未計数・要再確認・未取込みの精算の理由を無視して進む、完了が在庫を動かす、完了の後の記録で評価額が書き換わる。評価額の総額が安全な整数の範囲を越えて保存される。
- 旧本体・時点証拠の code・停止の文言と定数が repo に残る。既存の test を契約の置き換えの根拠なしに消す・弱める。
- migration vZ が行・id・列の値・index を欠く、`sqlite_sequence` を前の値より戻す、止まる DB で何かを変える。

### 非目的

- b2 の範囲（商品の結果の写しの 3 列と、商品一覧・在庫照会・商品フォームの在庫の欄の範囲の表示、在庫変動の履歴の計数の行、`MovementRecord` の並びの鍵）。下の「後続の lane」。
- Z004 の小数の数量の受理（SPEC-UNIT-D8 の IO の `quantity_hundredths` と整数でない個数の行。backlog の「単位の拡張の残りの申し送り」の P）。
- 棚卸し画面の案 D1 の見た目（部門をページ内の列にする形。下の「owner の判断事項」の (i)）。
- Z004 を SD の一括取込みに入れること（backlog の「Z004 を SD の一括取込みに入れる runtime（B）」）、EJ（ADR D11）、JAN の別名（D-122 の runtime）、在庫の振替の `movement_type`（(ii)）、返品・交換の `register_processed` の既定（(iii)）、レジの打ち方の点数の検査（TD-333）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

設計を含む変更の実装なので、店の通常の操作列を置く。例の数・日付は説明用の合成の値。本 lane の merge の後に下の列が成り立つ（b2 の表示を除く）。**この文書を完了できる**ことと、**棚卸しと POS の自動在庫連動の通常運用を達成できる**ことは同じ（b1 の merge で停止が外れ、通常運用の書く経路が揃う）。商品の画面の範囲の表示は b2。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
|---|---|---|---|---|
| vB（版 10）を当てた後に商品と入出庫の記録を作った開発・試験 DB（棚卸しと Z004 の確定は停止中なので、その記録は無い。v9 の版で商品を入れた DB は vB の起点の検査で止まり、D-123・22 §16 の作り直しの経路を通ってここに来る） | 新しい版を起動する | 起動できる。在庫は起動の前と同じ（vZ は写しを変えない） | 起動できる | 22 §20、Contract Probe P1〜P3 |
| 棚卸しをしていない | 棚卸しの画面で「棚卸しを開始する」 | 進行中の棚卸しができ、全ての商品が明細になる（自動の 0 の入力は無い） | 開始できる | 35 の新契約 |
| 商品 A（在庫 5）、今日 2026-10-02 | 棚卸しの画面で A を検索し、数 4 を「数を保存」 | A の在庫 4。一覧に保存済みと差異 −1 | 保存できる | ADR D5 |
| 同じ。A の精算 102（精算日 10-02）が未取込み | 締めの後に精算 102（A が 1 個売れた）を取り込む | 取込みの結果に「A は数えた日に売り買い（入出庫）がありました。棚の数を確かめて入れ直してください」と範囲「3〜4」。棚卸しの画面の上部の「数え直しが必要な商品」にも出る | 取込みは完了 | ADR D3 の窓（精算日 = 計数の日付は決まらない） |
| 同じ | 一覧の A の「入れ直す」で棚を見た数 3 を保存 | A の在庫 3、要再確認が解ける | — | fresh な計数（保存の時点の精算 102 と movement の最大 id と取込みの最大 id を持つ） |
| 前の日の数を紙で持っている | 「前の日の分を入れる」で 10-01 を選び商品 B の数を保存 | 10-01 の既定の計数として保存。後の日付の計数が既にあれば保存の前に「後の数が既にあり、そちらが効きます」 | 保存できる（拒まない） | ADR D5 |
| 精算 104 を取り込もうとする（103 が未取込み） | Z004 の preview | 「精算 103 が取り込まれていません」の知らせ（取込みは止めない） | 取り込める | ADR D4 |
| 精算回数の無い file・別の版・順の合わない file | Z004 の preview | 拒否の文（「精算回数の無い file は取り込めません。…」ほか 32 の表） | 選び直す | 32 の新契約 |
| 未計数・未取込みの精算が残る | 「棚卸しを完了する」→ 確認（「今日の最後の精算（締め）を取り込みましたか…」） | 完了できない理由ごとの一覧（数え直しが必要 / まだ数えていない / 取り込んでいない精算がある） | 理由を片付ける | 35 の新契約 |
| 未計数の明細がある | 「残りをまとめて 0 にする」（確認を挟む。日付は選べる） | 未計数が 0 の計数で片付く | — | ADR D6 |
| 全部片付いた | 完了 | 評価額と明細の数。在庫は動かない | 完了 | §20.5a |
| 完了の後に完了日以前の日付の入庫を入れた | 棚卸しの記録の詳細を開く | 完了した評価額は変わらず、「完了の後に入った記録による差（参考）」に差のある明細と差の合計 | — | ADR D6 |
| 連動なしの商品 C に連動なしの Z004 の売上がある | 商品フォームで C を在庫連動にする | 拒否「連動しない間の売上があります。今の数を入れてから在庫連動にしてください」と「今の数を入れる」の導線 → 離脱の確認で「破棄して移動」→ 棚卸しの画面で入れ直す → 戻った商品フォーム（DB の値）で在庫連動を付け直して保存できる | 保存できる | ADR D8、51・73（round 3 の是正） |

失敗と再試行: 計数の保存の拒否（未来の日付・形の違う日付・負・上限の外・別の商品の明細）は TX の前か TX の中の書く前で、何も書かない。Z004 の確定の拒否（preview の後の状況の変化を含む）は何も書かず preview token を残す（同じ token で確定し直しても同じ拒否になる。ファイルを選び直す）。完了の拒否は何も書かない。在庫の上限・評価額の総額の範囲の外は TX 全体が戻る。商品一括 import は、共有 JAN と在庫連動を始める検査に当たる行を preview で error 行にして他の行を取り込み、preview の後に登録や売上が変わったか上書きを選んだ行どうしが当たって確定の TX で行が拒まれたときは、行を名指して確定全体を戻し（何も取り込まない）、preview の画面と上書きの選択に戻る（その行をスキップにするか、ファイルを選び直す。30 の D-118 の節。round 2 の是正）。

Plan Review は、この列が「正常な条件で目的を達成できるか」と「危険な結果を出さないか」を別々に答える（`docs/DEV_WORKFLOW.md` Review Rules）。

## owner の判断事項

製品の振舞いの判断は D-118（TD-261・TD-263〜TD-268・TD-296・TD-298）と D-123（TD-311）で決まり、正本にある。設計正本・ADR で一意に決まらず、owner の判断が要る問いは次の 3 つ（どれも b1 の契約を決める問いではない。推奨は candidate）。

(i) 棚卸し画面の案 D1（部門をページ内の列にする見た目。backlog の「棚卸し画面 D1 の実装」は「(b) と組にする」と書く）を b1 に入れるか。

- A: b1 に入れる（棚卸しの画面の振舞いの変更と見た目の変更を同じ L3 で見る。開始前の画面と中断から戻った画面の見本は未作成なので、b1 の前に見本を作る design の段が要り、b1 の Scope と L3 が広がる）。
- B（推奨）: b1 は今の外枠の画面の上に新しい振舞いを入れ、D1 の見た目は v1.0 の前のデザインの刷新（owner の決定 TD-296「機能が固まった後に画面を一新する」）で入れる。理由: backlog の「組にする」は見た目だけを先に変えないための条件で、振舞いを先に入れて見た目を後で一新するのはその条件に反しない。R4 の lane の L3 の round を増やさない。
- 前提: A を選ぶなら b1 は design へ戻り、本 packet の Scope・L3・Owner Effort Budget を改める（Plan Gate の前）。

(ii) 在庫の振替（詰め替え・切り分け）の `movement_type`（TD-345。backlog は「(b) の `inventory_movements` の 2 回目の作り直しにこの種類を足すかを、起票の前に決める（Q-092 の答えの後）」と書く。Q-092 は未回答）。

- A（推奨）: b1 の vZ には足さない。Q-092 の答えで要ると決まれば、振替の機能の lane が自分の作り直しで足す（リリースの前なら D-123 で作り直しも自由で、リリースの後も v9・vZ と同じ全行を写す作り直しの migration で足せる）。理由: 使い方の決まっていない値を CHECK に先に入れない。後から足す作り直しは vZ と同じ手順で、値を失わない。
- B: 予約として vZ の `movement_type` の CHECK に足す（読む所・書く所の無い値を持つ）。
- b1 は答えを待たない（A を既定にして進める）。

(iii) 返品・交換の `register_processed` の既定（[63](../../function-design/63-ui-return-exchange.md) UI-03-D8、既定 true）と店の運用（TD-225: 差額だけの戻しの交換は Z004 に出入りが載らない）の関係（backlog は「本 runtime の起票で確かめる（直し方はまだ決めていない）」と書く）。起票で確かめた関係: UI-03-D8 の既定 true の理由（「レジ操作に載り Z004 取込みで反映される」）は、PLU で売る在庫連動の商品の返品にだけ成り立つ。差額だけの戻しの交換と、部門キーで売った商品の返品は Z004 に出ないので、既定のまま保存すると在庫が動かない。b1 はこの経路を変えず、悪くもしない（b1 の前は Z004 の確定が停止中で、既定 true の返品はどの商品でも在庫に入らなかった）。

- A（推奨）: b1 の外の小さな lane で直し方を決める（例: 商品が在庫連動で PLU の対象のときだけ既定 true、ほかは既定 false。交換の差額だけの戻しの案内）。b1 は答えを待たない。
- B: b1 に入れる（返品・交換の画面と BIZ が b1 の Scope に加わる）。

決定（2026-10-22、owner、repo 外の回答台帳 TD-377〜TD-379）: (i) B、(ii) A、(iii) A（どれも推奨どおり）。

店主に新しく聞く必要がある事項: 無い（backlog の「入庫・返品の記録を、品物が届く・戻る前に入力することはあるか」は ADR の受け入れた穴の頻度の問いで、答えは b1 の契約を変えない。ADR の Revisit）。

## 起票時に起草役が決めたこと

設計の値を変えない配布の割当てと細部。Coordinator が覆せる。正本へは本 commit で置いた（下の「本 commit で変えた正本」）。

1. (b) を b1・b2 に分けた（上の「規模と手法」。ADR D12）。
2. 初期在庫の見分け方を `reference_type` が NULL で `movement_type` が `receiving` の movement にした（ADR D9・D1、tracking、21）。理由: demo の seed（`src-tauri/src/seed_demo.rs` の `seed_sales`）の売上の movement（`sale_auto`、`reference_type` が NULL）が初期在庫と見分けられない（(a) の Plan Review の申し送り）。`reference_type` が NULL の movement を作る production の code は商品の登録と一括 import の初期在庫（`movement_type` は `receiving`）だけ（`rg -n 'reference_type: None' src-tauri/src --glob '!*test*'` は 8 行で、`#[cfg(test)]` の module の外にあるのは `src-tauri/src/biz/product_service.rs` の `create_product` と `commit_import` の 2 行。残り 6 行は `integrity_service.rs` と `inventory_repo.rs` の `mod tests` の中。raw の SQL では demo の seed の `seed_sales` が `sale_auto` を NULL で入れる）。seed を Z004 の取込みの形に作り直す案は採らなかった（seed の書き直しと作り直しを要し、窓の判定の正しさに足すものが無い）。seed の売上は業務日で窓を持つ入出庫の movement として判定される。
3. 共有 JAN の移行の前からの違反を、操作ログではなく整合性の検査の結果（`shared_jan_conflicts`）で今の DB から並べる（ADR D10、36・75）。理由は ADR D10 に書いた。
4. b1 の migration（vZ、22 §20 MNT-03-D18）の手順。`csv_imports` の 2 列は列の CHECK で「値を持つ」を書く（Contract Probe P1・P2）。
5. preview の引き当てと確定の取り直しが違う行があれば確定を拒む（D-122 からの申し送り、32）。preview token は確定の成功のときだけ消し、message の文字列で token を消す今の分岐を外す（41）。
6. Z004 の整数の数量の換算（SPEC-UNIT-D8 の BIZ の段のうち整数の分。長さの単位は × 100 の cm）を b1 に入れた（32）。理由: Z004 の確定を再開すると、在庫連動にした長さの商品の行が 100 分の 1 の数で在庫と売上に入る（今の旧本体と同じ誤り。在庫連動の自動提案は長さの単位で false〈51 UI-01b-D6〉だが、利用者は true にできる）。小数の数量の受理は P に残す（今の parser の行の error で黙って入らない）。
7. 精算回数の範囲を 0 以上 `9007199254740991` 以下にした（32 の表の順 1）。知らせの区間の端と精算回数を wire の number で正確に運ぶ。日次売上と日報の紙の wire の i64 の横断の規則（backlog）とは「揃える」と決めた: 入力の境で範囲を決められる値（精算回数・計数・在庫・範囲・評価額の総額）は検査して number で運び、範囲の決まらない読取りの金額（記録の詳細のロス原価の行・完了の後の評価額の差）は十進の文字列で運ぶ（SPEC-UNIT-D6 の行の金額と同じ）。日次売上と日報の紙の値の扱いはその項目が決める。
8. 入出庫の記録の一覧の棚卸しの件数を明細の数にした（65）。棚卸しの記録の詳細は補正の movement の節を持たない（65）。
9. 評価額の総額は完了の TX で `check_safe_integer` に通す（共通規則の「入力の上限と安全な整数の範囲」の (c) の表の「棚卸しの確定」の行。正本は既にある）。これで backlog の「棚卸しの runtime (b) で `total_cost` を wire・表示まで正確に扱う」は、総額が安全な整数の範囲に入り wire の number で正確に運べるので、十進の文字列にする変更が要らなくなる（closeout で backlog の項目を閉じる。例の cm の原価は vB で無くなった）。
10. 73 の Windows L3 の対象 (e)（日付の変わり目の確認）は、PC の時計を変える手順が L3 Eligibility の (3) に当たるので、自動 test（時計を差し替える）と画面の写しで見る。73 の L3 の一覧は実装の PR の S9 で直す。

## 横断の約束（DB・wire に出す値ごとに、どこが保証するか）

Contract Ledger と Matrix はこの表の # を引く。(a) の Y1〜Y8 は変えない（Y4・Y8 の (a) の「`phase = 1` は誤りで止まる」は b1 で精算の窓に置き換わる）。

| # | 値（DB の列 / wire） | 範囲と、保証する所 | 0 と符号 | 失敗の時点と順序 | 単位・種類の混在 | 経路 |
|---|---|---|---|---|---|---|
| Z1 | 計数の値（`stock_counts.count_value`、wire の `count_value`） | 0 以上 `999999999`（在庫の上限）。BIZ の `save_stock_count` と `zero_uncounted_items` が TX の書く前に 1 か所で検査。DB の CHECK は 0 以上 | 0 は有効（「0 にする」・売り切れ）。負は拒む | `ValidationFailed`、何も書かない | 在庫の数量の単位（長さは cm。UI は m の小数 2 桁を cm に直す。SPEC-UNIT-D3） | 計数の保存（既定・fresh）・0 にする |
| Z2 | 計数の業務日（`stock_counts.business_date`） | `validate_business_date`（31 の形と暦の検査）を通り、既定は今日以前、fresh は今日と同じ。今日は CMD が渡す PC の日付（BIZ は時計を読まない） | — | 書く前に `ValidationFailed`（「未来の日付には保存できません」ほか） | — | 同上 |
| Z3 | fresh のカーソル（`stock_counts.settlement_no`・`seen_movement_id`・`seen_import_id`） | 同じ TX で読んだ active な Z004 の取込みの最大の精算回数（無ければ NULL。ADR D1 の鍵の n で、判定に使わない）と `MAX(inventory_movements.id)`（無ければ 0）と `MAX(csv_imports.id)`（取り消したものを含む。無ければ NULL）。DB の CHECK（fresh なら `seen_movement_id` が NOT NULL。`seen_import_id` は NULL か、fresh で 1 以上）。判定の n は `seen_import_id` 以下の取込みID を持つ、今 active な Z004 の取込みの最大の精算回数（無ければ NULL。ADR D3。Codex の closure の是正で、round 3 の「n の active な取込みがあるときだけ n を使う」を置き換えた） | 精算回数 0 は実在しうるので NULL を印にする（判定の n = 0 もカーソル以下の active な取込みから決める）。取込みのカーソルの NULL は「保存の時点に取込みが無かった」（id は 1 から。0 は CHECK で拒む）で、movement のカーソルの 0 と印の形が違う（列を後から足すので既存の行に値を入れられず、NULL が意味として正しい。22 §20） | 読取りと INSERT が同じ TX。判定の n は求め直し・完了の判定・記録の詳細の読取りの TX で取込みの今の状態から求める（保存しない） | 判定の n は取込みID をカーソルと比べて求め、精算回数をカーソルにしない（取消の後に同じ番号を使う取込みを数えない）。保存した `settlement_no`（鍵の n）と判定の n を取り違えない | fresh な計数の保存、求め直し、完了の (ii)(iii)、在庫連動を始める検査 |
| Z4 | 写しの数・印・範囲（`products.stock_quantity`・`stock_recheck`・範囲） | (a) の Y2・Y3 のまま（`recompute_stock` が書く直前に上限 `999999999`）。求め直す商品は ADR の横断の表の「求め直す商品」の列 | 0 と負を受ける | 呼出し元の TX 全体が戻る | 計数・movement と同じ在庫の数量の単位 | 全ての書く経路 |
| Z5 | Z004 の精算回数（`csv_imports` の `settlement_no`、知らせの区間の端） | ASCII の数字だけで 0 以上 `9007199254740991` 以下（32 の表の順 1）。preview と確定の TX で検査。DB の列の CHECK は NOT NULL と 0 以上 | 0 は有効な精算回数 | 外れれば preview も確定も拒否（何も書かない） | — | Z004 の preview・確定 |
| Z6 | マシンNo.（`csv_imports` の `machine_no`） | 空でない文字列（leading zero を保つ）。DB の列の CHECK。32 の順 2 は取り消した取込みとも比べ（round 3 の是正）、取り消した取込みとだけ合わないときは拒否の文でその取込み（取込みID・精算回数・取込み日・レジ番号）を名指す（closure の是正） | — | 同上 | 別のレジの精算回数を同じ系列に混ぜない | 同上 |
| Z7 | 精算日（`csv_imports` の `settlement_date`、Z004 の movement の `business_date`） | IO-02 の日付の規則（今のまま）。精算日と精算回数の順の食い違いは拒否（32 の順 5）。精算日が今日より後は知らせ（拒否しない） | — | preview と確定の TX | — | 同上 |
| Z8 | Z004 の行の数量（`sale_records.quantity`、movement の `quantity`） | IO-02 の整数を在庫の数量へ換える（個数の単位はそのまま、長さの単位は × 100）。換えた後の絶対値が `9999999` を越える行は preview の行の error | 正 = 販売、負 = 返品。数量・金額とも 0 の行は売上も movement も作らない。数量 0 で金額が 0 でない行は売上だけ | preview で行の error。確定は preview の行を取り直す | 長さの商品は cm（SPEC-UNIT-D4・D8）。小数は P（行の error のまま） | Z004 の preview・確定 |
| Z9 | 取込みのときの連動（`sale_records` の `stock_linked`） | 確定の TX の `pos_stock_sync`。共有 JAN の行と商品の無い行は 0。後から書き換えない | 0/1 の CHECK | 確定の TX | 共有 JAN の行は売上だけ（先頭の商品）で在庫連動の外 | Z004 の確定 |
| Z10 | 欠けの知らせ（wire の `SettlementGap { from, to }`） | Z5 の範囲の整数。連続した欠けを 1 つの区間にまとめ、数は区間の数（欠けた番号の数に依らない） | 1 つの番号なら `from = to` | preview と確定の結果。取込みを止めない | — | Z004 の preview・確定 |
| Z11 | 評価数量・カーソル（`stocktake_items` の `valuation_quantity`、`stocktakes` の `cutoff_import_id`・`cutoff_count_id`・`cutoff_movement_id`） | `max(写しの在庫, 0)`（在庫の上限の中）。カーソルは完了の時点の `MAX(csv_imports.id)`（取り消したものを含む。無ければ NULL。Codex の closure の是正で精算回数のカーソルを置き換えた）と `stock_counts` の最大 id と `MAX(inventory_movements.id)`（無ければ 0。round 3 の是正）。完了の後の比較は、完了の時点にあった記録（カーソル以下）に完了の後に入れた完了の前の入出庫・初期在庫・精算の記録を足して求める（計数はカーソル以下だけ。精算は完了の時点の精算の n〈`cutoff_import_id` 以下の今 active な取込みの最大の精算回数〉以下と精算日 < 完了日。35。closure の是正） | 負の在庫は 0 で評価する。movement のカーソル 0 は movement が無い（id は 1 から）。取込みのカーソル NULL は取込みが無い（完了の時点の精算の n も NULL で、精算の範囲は精算日 < 完了日だけ） | 完了の TX。完了の後に書き換えない | 計数・movement・取込みのカーソルはそれぞれ自分の種類の id とだけ比べる（Z004 の movement は、取込みのカーソルから求めた完了の時点の精算の n と精算回数で比べる） | 完了、記録の詳細の読取り |
| Z12 | 評価額の総額（`stocktakes` の `total_cost`、wire の `total_cost`） | §20.5a の関数の後に `check_safe_integer`（±`9007199254740991`）。外れれば `ValidationFailed`（「合計金額が大きすぎます」） | 0 は有効 | 完了の TX 全体が戻る | 価格の基準数量で割る（長さは 1 m あたり） | 完了 |
| Z13 | 記録の詳細の金額（行のロス原価、完了の後の評価額の差の行と合計） | 読取りで求め、保存しない。行のロス原価は 1/100 円の十進の文字列（SPEC-UNIT-D6）、評価額の差の行と合計は円の十進の文字列（範囲の決まらない読取りの合計） | 差 0 の明細は差の一覧に出さない。負の数量を評価の関数に渡さない（`max(…,0)`） | 溢れは既存の `ValidationFailed`（読取りの失敗） | 価格の基準数量で割る | 記録の詳細の読取り |
| Z14 | 有限の値（`StockCountKind`・`RecountReason`・`CountRecoveryAction`・`CompletionBlockReason`・`StocktakeRecoveryCode`、`CmdErrorKind` の `stocktake_guard`） | Rust の enum を正とする generated literal union（D-061）。`recount_reasons` は重複なし・`undetermined_settlement`・`same_date_movement`・`conflicting_counts` の順 | — | — | — | 計数・完了・要再確認の一覧・在庫連動の拒否 |
| Z15 | 共有 JAN の不変条件と一覧（`IntegrityResult.shared_jan_conflicts`） | 変更の後の状態で TX の中で検査（BIZ-01）。一括 import の preview は新規の行を CSV の順に先の新規の行を当てた状態で、重複の行を全ての新規の行を当てた状態で判定し、確定は送られた行の順に TX のその時点の状態で判定する（30。round 2 の是正）。一覧は JAN の昇順、商品コードの昇順 | — | 拒否は書く前。一括 import は preview で当該行を error 行にして他の行は続け、確定の TX で当たった行は最初の 1 行を名指して `validation`（field なし）で確定全体を戻す（何も書かない。round 2 の是正） | — | 商品の登録・修正・一括 import、整合性の検査 |
| Z16 | 在庫連動を始める検査（`recount_required`） | ADR D8 の条件。拒否は回復先（`active_count` / `count_now`）と商品を型で返す（一括 import は行の error の文） | — | 書く前。共有 JAN と両方に当たれば共有 JAN の拒否を先に返す。一括 import の確定の TX で当たれば Z15 と同じ形で確定全体を戻す（round 2 の是正） | — | 商品の修正・一括 import の上書き |
| Z17 | vZ の列と CHECK（`(reference_type IS 'csv_import') = (phase = 1)` ほか） | 22 §20 の手順 | — | 止まる検査は何かを変える前。確かめの失敗は全部戻る | — | migration |
| Z18 | 要再確認の対象と完了の拒否の直し方（wire の `CountRecoveryTarget` の `stock`・範囲・`stock_unit`、`SettlementPendingHint`。round 1 の是正） | `stock` と範囲は target を作った TX の求め直しの後の写し（Z4 の範囲）。`missing` の端は Z5 の範囲。`on_or_after`・`before` は明細の効く計数の日付（Z2 を通った `YYYY-MM-DD`） | 範囲は両方 null か両方の値。`missing` が空で日付が 2 つとも null なら `settlement_pending` 自体を None にする | 読取りで作り、書く前の拒否（完了・在庫連動）と取込み・取消の結果に載る | 範囲は商品の在庫の数量の単位（長さは cm）で、画面が `stock_unit` で m に出す | 要再確認の一覧・完了の拒否・在庫連動の拒否・取込みと取消の結果 |
| Z19 | preview の cache（`MatchedRow.normalized_jan`・`shared_jan`、`CachedPreview.settlement`。round 1 の是正） | parse の時点の値を server の cache に 30 分持ち、確定の TX で今の DB と照合する（32 の「preview の cache の契約」の (4)） | 未登録の行（error 行）も照合に入る | 食い違いは確定の TX の中で書く前に `ImportError`、token は残す（Z004 の確定・Z8） | 共有 JAN かどうか（候補の数）も照合する | Z004 の確定 |
| Z20 | 計数の欄の商品の文脈（`CountPlacementPreview` の `stock`・範囲・効く計数。round 1 の是正） | 読取りだけ（写しと計数を読む）。値は Z1・Z4 の範囲 | 計数が無ければ `count_*` は null | 保存の前の読取りで、失敗しても何も書かない | `stock_unit` で m に出す | 「今の数を入れる」と「前の日の分を入れる」 |

## この変更で不要になるもの（撤去と置き換えの全件）

撤去する code（実装の PR の S9。file ごと消すものは file 名、残る file の中で消すものは symbol）:

- 停止の入口と停止の文言: `src-tauri/src/biz/stocktake_service.rs` の `WRITES_SUSPENDED_MESSAGE`・`writes_suspended` と、`update_count`・`start_stocktake`・`complete_stocktake` の停止の本体。`src-tauri/src/biz/csv_import_service/commit.rs` の `IMPORT_SUSPENDED_MESSAGE` と停止の分岐、`src-tauri/src/biz/csv_import_service/rollback.rs` の停止の分岐。`src-tauri/src/cmd/stocktake_cmd.rs` の `BIZ06_STOP` と停止の test、`src-tauri/src/cmd/csv_import_cmd.rs` の `BIZ03_STOP` と停止の test。画面の停止の定数と案内（`src/features/stocktake/StocktakePage.tsx` の `writesSuspended` の prop と Alert、`src/features/csv-import/CsvImportPage.tsx` の停止の Alert と `PreviewStep.tsx` の「取り込む」の無効化）。
- 旧本体: `legacy_start_stocktake`・`legacy_update_count`・`legacy_complete_stocktake`（`stocktake_service.rs`）、`legacy_commit_csv_import`（`commit.rs`）、`legacy_rollback_csv_import`（`rollback.rs`）と、旧本体だけが使う関数（`apply_void_stock_corrections`・`StockCorrection`、`stocktake_repo` の `get_stocktake_items_for_complete` と型 `StocktakeItemForComplete`。round 1 の是正で Matrix の T11 に分類を決めた: `update_stocktake_item_count` は明細の表示用の写しに、`find_stocktake_item_with_parent_status` は計数の保存の明細の確かめに、`count_uncounted_items`・`list_uncounted_items` は「計数済み」の定義を改めて完了と 0 にするに使い続ける）。`apply_stock_change` の `settlement_no` の引数（b1 で Z004 の確定が使わなくなり、呼ぶのは全部 `None` の入出庫 4 業務だけ。31）と、`MatchedRow.pos_stock_sync`（確定の TX で取り直す。32 の「preview の cache の契約」）も外す。
- 停止の test: `src-tauri/src/biz/csv_import_service/tests/legacy_stop_tests.rs`（file ごと。F1〜F7 の入力は新しい経路の test の入力に移す。Matrix の T2）。
- 時点証拠の schema と code（D-109 の `#[cfg(test)]`）: `src-tauri/src/db/schema_time_evidence.rs`、`src-tauri/src/biz/stocktake_service/time_evidence.rs`・`time_evidence_tests.rs`、`src-tauri/src/biz/csv_import_service/time_evidence.rs`・`tests/time_evidence_tests.rs`（file ごと）。`src-tauri/src/db/inventory_repo.rs`・`stocktake_repo.rs`・`sales_repo.rs` の `pub(crate) mod time_evidence`、`src-tauri/src/db/mod.rs` の `mod schema_time_evidence`、`src-tauri/src/db/test_support.rs` の `apply_time_evidence`・`setup_time_evidence_db`・`LEGACY_ITEM_KINDS`・`seed_legacy_fixture`、`src-tauri/src/biz/csv_import_service/mod.rs`・`tests/mod.rs`・`stocktake_service.rs` の module の宣言。
- probe: `scripts/probes/stocktake_time_model.py`（directory ごと空になる）。時点証拠 ADR の Evidence の link は「撤去した」の文に直す（link の切れを残さない）。
- 置き換わる wire と画面: `update_count` の command と `UpdateCountRequest`・`UpdateCountResult`、`complete_stocktake` の `force_fill`、`StartStocktakeResult.auto_filled_count`、`StocktakeResult.adjusted_items` と `AdjustedItem`、`StocktakeRecordDetail.movements`・`corrected_count` と `StocktakeRecordDetailItem` の補正の field、`commit_csv_import` の `additional_import_confirmed`、`PreviewData.duplicate_check` と `DuplicateCheck`・`DuplicateStatus`・`SameDateCsvImportSummary`（Z004 だけが使う。状態は「重複なし」と「同じ日の追加の確認が要る」の 2 つで、後者が無くなる。同じ hash の重複は今も preview の拒否〈32 の順 3〉で、この型を使わない。日報の取込みは別の型〈`DailyReportDuplicateCheck`〉で変えない）、`CachedPreview.active_same_date_import_ids`、`src/features/stocktake/hooks/useUpdateCount.ts`、Z004 の `PreviewStep.tsx` の `AdditionalImportConfirmDialog` の使用（component は日報取込みタブが使うので残す）。
- 置き換わる正本の文: 停止 ADR は superseded にし、35 §20.0・32 §15.0・55 §55.0 の停止の節・73 §73.15 を外す。35・32・42・41・55・73・65 の旧本体の処理ステップと force_fill・同じ日の追加の確認の記述は、D-118 の節の契約へ置き換える（S9）。(a) の「`phase = 1` の movement は誤りで止まる」（31・tracking・ADR D12 の (a) の段）は精算の窓に置き換える。

不要にならないと確かめたもの: `AdditionalImportConfirmDialog.tsx`（日報取込みタブが使う。`rg -n 'AdditionalImportConfirmDialog' src/features/daily-report-import` で 3 行）、日報の取込みの `additional_import_confirmed`（BIZ-08 の同日追加の確認。SD 直読みの runtime〈PR #172〉の範囲）、`movement_type` の `stocktake` の値（既存の記録の型。v9 で止まった DB を作り直したので行は無いが、enum と画面の label は b1 では外さない。外すなら別の小口）。

## Scope

本 lane が変える file と目的。実装は S1 → S9 の順に積むが、S1〜S7 は 1 つの compile・回帰の検証の単位（途中の段では全 test の成功を求めない。理由: S1 の CHECK で旧本体と時点証拠の fixture が落ち、S4・S5 で撤去するまで通らない。S7 の wire の変更は S8 の画面と bindings で揃う）。S8 は frontend の検証の単位、S9 は docs と生成物。S7 の終わりで AC1〜AC7 の Rust の分、S8 の終わりで AC8、S9 の後に AC9〜AC11。既存の test の移し方は Matrix の「既存の test の移し方」（T1〜T12）を引く。

### 本 commit で変えた正本（plan-first）

- [ADR](../../adr/2026-10-16-stocktake-count-placement.md): Status に b1・b2 の分け、D1 の表の初期在庫の行と D9 の初期在庫の見分け方（`movement_type` の条件と demo の seed の扱い）、D10 の最後の文（共有 JAN の違反を整合性の検査で並べる）、D12 に「(b) を 2 つの lane に分けた」。
- [tracking](../../db-design/tracking-system-tables.md): 初期在庫の見分け方、b1 の migration への参照（冒頭の文と `settlement_no` の行）。
- [21](../../function-design/21-io-inventory-repo.md): `list_ledger_movements_for_product` の comment の初期在庫の印。
- [22 §20](../../function-design/22-mnt-migration.md)（新設、MNT-03-D18）: vZ の手順・止まった後・回復・test。
- [32](../../function-design/32-biz-csv-import-service.md): 表の順 1 の精算回数の範囲、preview と確定の引き当ての食い違いの拒否、整数の数量の換算、知らせの wire。
- [41](../../function-design/41-cmd-pos.md): preview token を消す条件。
- [36](../../function-design/36-biz-integrity-check.md)・[75](../../function-design/75-ui-integrity-check.md): 共有 JAN の違反の一覧。
- [65](../../function-design/65-inventory-record-traceability.md): 入出庫の記録の一覧の棚卸しの件数、記録の詳細の補正の movement の節を外す。
- [Plans](../../Plans.md): `## 次の行動` と `## 導入の前の関門` の 2. に本 packet への pointer。
- round 1 の是正で直した正本（Review Response 参照）: [40](../../function-design/40-cmd-product.md)（`CountRecoveryTarget` の単位・写しの在庫・範囲、`StocktakeRecovery.settlement_pending` と `SettlementPendingHint`）、[35](../../function-design/35-biz-stocktake-service.md)（完了の拒否の直し方、`preview_stock_count_placement` の計数の欄の文脈）、[42](../../function-design/42-cmd-sales-stocktake.md)（`preview_stock_count_placement` の出力）、[73](../../function-design/73-ui-stocktake.md)（要再確認の一覧の範囲、完了の拒否の直し方の文、「今の数を入れる」の文脈の読み先）、[30](../../function-design/30-biz-product-service.md)（共有 JAN の不変条件の経路: update は在庫連動の変更だけ、JAN の変更は一括 import の上書き）、[31](../../function-design/31-biz-inventory-service.md)（Z004 の確定は `insert_movement` と商品ごとの求め直し、`apply_stock_change` の `settlement_no` を b1 で外す）、[32](../../function-design/32-biz-csv-import-service.md)（preview の cache の契約: `MatchedRow`・`CachedPreview`・`CommitRequest` と確定の照合）。
- round 2 の是正で直した正本: [30](../../function-design/30-biz-product-service.md)（商品一括 import の判定の状態と確定の TX の拒否の形と message、在庫連動を始める拒否の順、§4.9 のエラーハンドリング）、[35](../../function-design/35-biz-stocktake-service.md)（`zero_uncounted_items` の明細の参照と拒否、操作ログ）、[42](../../function-design/42-cmd-sales-stocktake.md) §22.6（`commit_import` の確定の拒否は 30 を引く）、[60](../../function-design/60-ui-product-import.md)（確定の拒否の表示）、[UI_TECH_STACK](../../UI_TECH_STACK.md)（再取得の経路と entry と要再確認の一覧の key）。
- round 3 の是正で直した正本: [ADR](../../adr/2026-10-16-stocktake-count-placement.md) D3（fresh な計数の n を使うのは n の active な取込みがあるときだけ。Codex の closure の Gated Amendment で取込みのカーソルの形に置き換えた）・D4（machine_no は取り消した取込みとも比べる）・D6（完了の後の比較の範囲と movement のカーソル）・D9（`cutoff_movement_id`）、[32](../../function-design/32-biz-csv-import-service.md)（表の順 2 と取り消した取込みの比べ方）、[24](../../function-design/24-io-csv-import-repo.md)・[POS](../../db-design/pos-tables.md)（machine_no の読取りと判定）、[35](../../function-design/35-biz-stocktake-service.md)（完了の (ii)(iii) の n、完了のカーソル、完了の後の比較の範囲）、[tracking](../../db-design/tracking-system-tables.md)（n の比べ方、完了のカーソル）、[22 §20](../../function-design/22-mnt-migration.md)（手順 5 の列）、[51](../../function-design/51-ui-product-form.md)・[73](../../function-design/73-ui-stocktake.md)（「今の数を入れる」の導線が離脱の確認を通ること、`/stocktake/` の search の `product` と `returnTo`）。
- Codex の Plan Review closure（P1）の Gated Amendment で直した正本（2026-10-23。owner の決定〈repo 外の回答台帳 TD-389・TD-400〉）: [ADR](../../adr/2026-10-16-stocktake-count-placement.md) D1（鍵の n）・D3（fresh な計数の判定の n を取込みのカーソルから求める。round 3 の「n の active な取込みがあるときだけ」を置き換えた）・D5・D6（完了の (i)〜(iii) の n、`cutoff_import_id`、完了の後の比較の範囲の精算）・D7（取り込み直した取込みは判定の n の算出に入らず、取り込み直した精算が判定の n だったときに日付で判定する）・D9（`stock_counts.seen_import_id`、`stocktakes.cutoff_import_id`）・D12・Consequences（取り込み直しの後の要再確認）・Revisit、[32](../../function-design/32-biz-csv-import-service.md)（順 2 の理由、順 5 の後の判定、取り込み直し）、[24](../../function-design/24-io-csv-import-repo.md)（取込みの最大の取込みID の読取りを新設）、[22 §20](../../function-design/22-mnt-migration.md)（手順 5 の列）、[tracking](../../db-design/tracking-system-tables.md)・[POS](../../db-design/pos-tables.md)（取込みのカーソル、取込みID を使い回さない）、[35](../../function-design/35-biz-stocktake-service.md)（保存のカーソル、完了の n とカーソルと範囲）、[21](../../function-design/21-io-inventory-repo.md)・[31](../../function-design/31-biz-inventory-service.md)（読取りの列）。
- Plan Review closure の是正で直した正本: [ADR](../../adr/2026-10-16-stocktake-count-placement.md) D4（取り消した取込みとだけ合わないときに拒否の文で名指す。owner の決定〈2026-10-23、repo 外の回答台帳 TD-388〉）・D6（完了の後の比較の範囲から完了の後に保存した計数を除く）・D7（取り込み直しと machine_no の照合）、[32](../../function-design/32-biz-csv-import-service.md)（表の順 2 の拒否の文と比べる先）、[24](../../function-design/24-io-csv-import-repo.md)（machine_no の照合先の読取りを新設する 1 通りに）、[35](../../function-design/35-biz-stocktake-service.md)・[tracking](../../db-design/tracking-system-tables.md)（完了の後の比較の範囲の計数）。正本を変えない分は Matrix の K2 に取消と取り込み直しの列（N3。35 の完了の (iii) の n を n の active な取込みがあるときだけ使うことを観測する）。

### S1 migration vZ（22 §20）

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| migration vZ | 22 §20 の手順 1〜7。関数は `pub(crate)`。COMMIT・ROLLBACK は共有の helper（`db/migration_tx.rs` の `commit_transaction`・`rollback_after_error`）だけ。版の番号の定数（例 `VZ_VERSION`）を `schema_v{N}.rs` に置き、test はそれを引く（vB の `VB_VERSION` と同じ形） | `src-tauri/src/db/schema_v{N}.rs`（新設。N は merge の時点の次。起票の時点では 11）、`src-tauri/src/db/mod.rs`（module の宣言）、`src-tauri/src/db/migration.rs` の `migrations()`（登録）と use | MNT-03-D1 の構造の test（`migration.rs` の、migration の file を `include_str!` で列挙して共有の helper だけを使うことを確かめる test）の列挙に新しい file を足す。`expected_tables` は表を足さないので変えない |
| 既存の migration の test（最新の版の期待） | Matrix の T1（最新の版を期待する 9 行のうち 7 行と、`(10, 10)` の形を外れた 5 行〈Codex の closure の Gated Amendment で足した〉は最新の版 `app_max_version()` と比べる形へ〈版の定数から替えた。版が増えても直さずに済み、並走の PR #172 も同じ形〉、全 object の SQL か vB の成功を見る `schema_v10.rs` の 2 本は 22 §19 の先例どおり `migrate_up_to(&conn, VB_VERSION)` に替えて期待を変えない。round 2 の是正で command を広げた。`migrate_up_to` で版を指定する他の test と `app_max_version()` を引く test は変えない） | `src-tauri/src/db/migration.rs`、`src-tauri/src/lib.rs` の `test_startup_req903_` の test、`src-tauri/src/db/schema_v9.rs`・`src-tauri/src/db/schema_v10.rs` の test | 並走の §17 の lane（PR #172）も版の期待の行を書き換える（下の所有表）。着手時に T1 の command を逐語で回し、表と違う所があれば表を直してから進める |
| 新しい test | Matrix の M1〜M7 | `schema_v{N}.rs` の test | — |

### S2 IO（`inventory_repo`・`sales_repo`・`stocktake_repo`・`product_repo`・`disposal_repo`）

新しい関数のシグネチャは実装の S2 で 21・24・20 の ```rust の code block に置く（(a) と同じ形。`pub(crate)` で、`src-tauri/tests/design_compliance_test.rs` の allowlist に足さない）。

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| 初期在庫の印 | `list_ledger_movements_for_product` の初期在庫の印を `reference_type IS NULL AND movement_type = 'receiving'` に（ADR D9） | `src-tauri/src/db/inventory_repo.rs` の `list_ledger_movements_for_product` と `LedgerMovementRow` の doc comment | Matrix の L8 |
| 計数の保存 | `insert_stock_count`（`stock_counts` へ 1 行。UPDATE・DELETE の関数は持たない）と、保存の時点のカーソルの読取り（`MAX(inventory_movements.id)`、active な Z004 の最大の精算回数、`MAX(csv_imports.id)`〈24。完了の TX も同じ読取り。Codex の closure の Gated Amendment〉） | `inventory_repo.rs`（21 の新契約の「計数の保存」）、`sales_repo.rs`（24 の「Z004 の取込みの最大の取込みID」） | BIZ-06 の `save_stock_count` だけが呼ぶ |
| 精算の読取り | active な Z004 の精算の一覧（import ID・`machine_no`・`settlement_no`・`settlement_date`、精算回数の昇順）、machine_no の違う Z004 の取込み 1 件（取り消したものを含み active を優先。import ID・`machine_no`・`settlement_no`・`status`・`imported_at`。32 の順 2 の判定と拒否の文の名指しに使う。新設する: active な一覧は rolled_back を返さず、`CsvImport` は新しい列を読まない。round 3 と closure の是正）、同じ精算の active な取込み（0 件か 1 件）、取込みの商品（売上か movement を持つ商品）の一覧、商品の決まらない精算の行の判定に要る行（その取込みの `sale_records` の `stock_linked`・個数・金額） | `src-tauri/src/db/sales_repo.rs`（24 の新契約）、`inventory_repo.rs` | BIZ-03 の識別・順・欠け、BIZ-02 の求め直し（精算の窓と理由 1） |
| 取込みの保存 | `NewCsvImport` に `machine_no`・`settlement_no`。`NewSaleRecord` に `stock_linked: bool`（Z004 の確定だけが取込みのときの `pos_stock_sync` を渡し、手動の売上ほかは false。DB の既定 0 に頼らず、作る所が値を書く。round 1 の是正）。読取りの型 `CsvImport`（一覧の行）・`CsvImportRecordDetail`・`Stocktake` は形を変えない（新しい列を読まない。画面と TS の mock を変えない） | `sales_repo.rs` の `NewCsvImport`・`NewSaleRecord` と INSERT の関数 | `NewCsvImport` を作る所は BIZ-03 の確定だけ（旧本体は S5 で消える。test は Matrix の T10）。`NewSaleRecord` を作る所は `rg -c 'NewSaleRecord \{' src-tauri/src` の 5 file（`sales_repo.rs` 9〈定義と test〉・`csv_import_service/commit.rs`・`inventory_service/manual_sale.rs`・`sales_service.rs`・`cmd/sales_cmd.rs` 各 1。Matrix の T12） |
| 棚卸しの明細と記録 | 明細の読取り（`list_stocktake_items`・`find_stocktake_item_by_code`）は明細と商品の列に、明細を参照する計数の有無と単位を足す（効く計数・差異・範囲・理由は BIZ が求める）。進捗の「計数済み」は明細を参照する計数があること。開始は自動の 0 の入力をやめる。完了は `valuation_quantity`・カーソル・`total_cost`・`completed_at` を書く。記録の詳細は明細と header から読む（補正の movement から読まない） | `src-tauri/src/db/stocktake_repo.rs` | 明細の型 `StocktakeItemDetail`・`StocktakeRecordDetailItem` の形は下の Boundary。旧本体だけが使う関数は S4 で `rg` で数えて消す |
| 共有 JAN の読取り | 変更の後の状態で、在庫連動の商品と同じ `jan_code` を持つ別の商品の一覧（BIZ-01 の検査）と、全ての組（整合性の検査） | `src-tauri/src/db/product_repo.rs`（今の `find_by_jan_code` で足りるなら新設しない。S2 で決める） | BIZ-01・BIZ-07 |
| 入出庫の記録の一覧の棚卸し | `list_inventory_records` の棚卸しの `RecordSpec` の件数と絞り込みを、補正の movement（`reference_type = 'stocktake'`）から明細（`stocktake_items`）へ（65 の新契約） | `src-tauri/src/db/disposal_repo.rs` の `list_inventory_records` の棚卸しの spec | test `test_list_inventory_records_req206_stocktake_item_count_diff_only` は契約の変更（65）なので期待を変える（Matrix の T2） |

### S3 BIZ の求め直しの精算の窓（`biz::stock_ledger`）

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| 精算の窓 | ADR D3 の判定表の「精算 k」の行（既定の計数・fresh な計数のそれぞれ、A1 の近似、「D に精算があるか」「最初の 1 本」は取り込み済みの精算で決める）と要再確認の理由 1（`UndeterminedSettlement`: 決まらない精算の取込みにその商品の有効な Z004 の movement があるか、`stock_linked=1` の行が個数 ≠ 0 か金額 ≠ 0）。fresh な計数の判定の n は `seen_import_id` 以下の取込みID を持つ、今 active な取込みの最大の精算回数（無ければ NULL。ADR D3。Codex の closure の Gated Amendment で round 3 の形を置き換えた）。`phase = 1` で止まる今の分岐（`derive` の `movement.phase != 0`）を外す | `src-tauri/src/biz/stock_ledger.rs` の `derive`・`judge`・`RecheckReason` | Matrix の L1〜L13。`RecheckReason` の 3 つは wire の `RecountReason` に同じ順で写す（S7） |
| 求め直す商品 | Z004 の確定・取消の TX で「計数を持つ全ての商品 + その取込みの売上か movement を持つ全ての商品」を `recompute_stock` する共有の関数（ADR の横断の表） | `stock_ledger.rs` か `csv_import_service` の共有の関数 1 つ | Matrix の L12・Z8・Z14 |
| 計数の前後が循環する誤り | 今のまま（計数の保存の検査〈Z2〉が循環を作らない前提を守る） | — | Matrix の C3 |

### S4 BIZ-06 計数・完了・読取り（`stocktake_service`）

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| `save_stock_count`・`preview_stock_count_placement`・`zero_uncounted_items`・`list_recheck_products` | 35 の新契約（D-118 の節の「計数の保存」「完了」の未計数を片付ける操作、「読取り」）。`preview_stock_count_placement` は `later_count_exists` に加えて計数の欄の商品の文脈（商品名・単位・写しの在庫と範囲・理由・効く計数）を返す（42・35。棚卸しの明細の無い商品の「今の数を入れる」の画面が読む。round 1 の是正）。`list_recheck_products` と取込み・取消の `recheck_targets` の `CountRecoveryTarget` は単位・写しの在庫・範囲を持つ（40。round 1 の是正）。`today` は引数（CMD が渡す）。検査の順: 形と暦（`validate_business_date`）→ 今日との比べ（既定は今日以前、fresh は今日）→ 値の範囲（Z1）→ TX の中で明細の進行中と商品の一致 → INSERT → `recompute_stock` → 明細の `actual_count`・`counted_at` の写し。`zero_uncounted_items` の各計数は進行中の棚卸しの当該明細を参照し、明細の無い商品が 1 つでもあれば何も書かずに拒む（35。round 2 の是正） | `src-tauri/src/biz/stocktake_service.rs`（大きくなれば `stocktake_service/` の module に分ける。分けるなら `design_compliance_test.rs` の 35 の行に module を足す） | Matrix の C1〜C11・W3 |
| `start_stocktake` | 停止を外し、明細を作る（自動の 0 の入力をやめる。`StartStocktakeResult` から `auto_filled_count` を外す）。進行中があれば `StocktakeInProgress`、対象 0 件は今の文言。操作ログは 35 の新契約の「操作ログ」（detail から `auto_filled_count` を外す。round 2 の是正） | 同上 | Matrix の K7・K8 |
| `complete_stocktake` | 停止を外し、35 の新契約の完了（`settlement_pending` の (i)〜(iii)・`recheck`・`uncounted`〈廃番かつ写し 0 は除く〉を全部調べてから `BizError::StocktakeGuard`〈`completion_blocked`〉で何も書かない。通れば評価数量・評価原価・総額〈§20.5a の後に `check_safe_integer`〉・カーソル〈`cutoff_import_id`・`cutoff_count_id`・`cutoff_movement_id`〉を書き、movement を作らない）。`settlement_pending` のときは 40 の `SettlementPendingHint`（欠けた精算回数の区間、(i) の日付の最大、(iii) の日付の最小）を添える（round 1 の是正）。`force_fill` を外す。操作ログは 35 の新契約の「操作ログ」（summary の差異の件数は差異のある明細の数、detail は `difference_count`。round 2 の是正） | 同上 | Matrix の K1〜K6・K8 |
| 読取り | `get_stocktake_items`・`find_stocktake_item`（効く計数の値・種類・日付、写しの在庫と範囲、理由、差異、単位）、`get_stocktake_record`（完了の時点のカーソルで求めた明細ごとの数と差、行のロス原価、完了の後の差。35 の新契約の「完了」の後半）、`get_stocktake_progress`（計数済みの定義） | 同上 | Matrix の C10・R1〜R5 |
| `BizError::StocktakeGuard` | `{ message, recovery: StocktakeRecovery }` を足す（40 の新契約）。Display と CMD の変換（S7） | `src-tauri/src/biz/mod.rs` | 30（D8）と 35（完了）が返す |
| 撤去 | 旧本体 3 つと停止の本体、時点証拠の module の宣言、旧本体だけが使う関数 | 同上 | 上の「この変更で不要になるもの」 |

### S5 BIZ-03 Z004 の preview・確定・取消（`csv_import_service`）

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| `parse_and_validate` | 32 の新契約の表の順 1〜5（順 3 は今の重複の拒否のまま）、知らせ（欠けの区間・精算日が今日より後）、今の「精算日が未来の日付です」の拒否と同じ日の取込みの一覧を外す、Z8 の数量の換算と上限の行の error、正常な行が全て 0 の file を拒まない。`today` は引数。preview の cache を 32 の「preview の cache の契約」の形で作る（`MatchedRow` の `normalized_jan`・`shared_jan`・在庫の数量、`CachedPreview.settlement`。round 1 の是正） | `src-tauri/src/biz/csv_import_service/parse.rs`、`mod.rs`（型と §15.2 の型の節。`src-tauri/tests/import_internal_contract_test.rs` の固定を同じ形に。Matrix の T10） | Matrix の Z1〜Z7・Z9・Z12・Z22 |
| `commit_csv_import` | 停止を外し、確定の TX で順 1〜5（cache の `settlement` で。file を読み直さない）と JAN の候補（cache の `normalized_jan` で引き直し、候補が無い・先頭の商品が違う・共有 JAN かどうかが違う・未登録だった行に候補ができた、のどれかなら拒否。32 の「preview の cache の契約」の (4)）と連動の設定を取り直す → `csv_imports` に精算の鍵 → 売上（`stock_linked`）→ 在庫連動の行の movement（`business_date = settlement_date`・`settlement_no`。下の「Z004 の movement の書き方」のとおり `insert_movement` に精算回数を渡す）→ 求め直す商品の `recompute_stock` → 結果に `recheck_targets` と知らせ。`additional_import_confirmed` を外し、`today` を受ける | `commit.rs` | Matrix の Z8〜Z13・Z15・Z16・Z21・Z23 |
| `rollback_csv_import` | 停止を外し、売上と movement を無効にして求め直す商品を `recompute_stock`（補償の movement を作らない。`StockCorrection` を外す）。結果に `recheck_targets`。取消済みへの再度の取消は今の冪等の Ok のまま | `rollback.rs` | Matrix の Z14 |
| Z004 の movement の書き方 | Z004 の確定は `apply_stock_change` を行ごとに呼ばない（行ごとに求め直すと、同じ商品の複数の行の `stock_after` が途中の値になり、31 の「Z004 の同じ商品の複数の行も同じ値」と合わない）。在庫連動の行の movement を全部、仮の `stock_after`（今の写しの値）で `insert_movement`（`settlement_no` を渡し `phase = 1`）してから、求め直す商品ごとに `recompute_stock` を 1 回呼び、その TX で書いた movement の `stock_after` を写しの値で `update_movement_stock_after` で書く（31 の `apply_stock_change` の順を商品ごとにまとめた形。入出庫 4 業務の `apply_stock_change` は変えない） | `commit.rs`。`apply_stock_change` の `settlement_no` の引数は外す（31。呼ぶのは入出庫 4 業務だけで全部 `None` だった。`src-tauri/src/biz/inventory_service/common.rs` の定義と test の 6 か所、`receiving.rs`・`returns.rs`・`manual_sale.rs`・`disposal.rs` の呼出し。Matrix の T12） | Matrix の Z8 |
| test の fixture | `make_z004_bytes` を layout A のメタ（マシンNo.・精算回数）を持つ形へ（精算回数を引数に取る）。旧本体の test は Matrix の T2 の分類で新しい経路へ移す。`csv_imports` を直に書く test・`NewCsvImport` の literal・`csv_import` の movement を `phase = 0` で入れる test は vZ の CHECK で落ちるので、Matrix の T10 の分類で fixture を直す | `src-tauri/src/biz/csv_import_service/test_support.rs`、`tests/*.rs`（`list_tests.rs` を含む）、`src-tauri/src/db/sales_repo.rs`・`disposal_repo.rs` の test、`src-tauri/src/cmd/csv_import_cmd.rs` の test | — |
| 撤去 | 旧本体 2 つと停止の分岐、`time_evidence` の module、`legacy_stop_tests.rs` | 同上 | 上の「この変更で不要になるもの」 |
| 診断 2 本 | `diagnostic_cross_feature_req205_count_then_movement`・`diagnostic_cross_feature_req205_req401_late_import` の `#[ignore]` を外し、新しい経路で PASS する回帰 test にする（名前は `test_` で始まる名に改め、`XFA_TEMPORAL_FAIL`・`XFA_LATE_IMPORT_FAIL` の失敗の文を外す） | `tests/cross_feature_tests.rs` | Matrix の X1・X2 |

### S6 BIZ-01 共有 JAN と在庫連動を始める検査、BIZ-07 共有 JAN の一覧

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| 共有 JAN の不変条件 | create・update（在庫連動の変更だけ。update は JAN を変えられない: `ProductUpdateRequest` に `jan_code` が無く、商品フォームの edit の JAN 欄は読取専用）・商品一括 import（新規の行と上書きの行。JAN が変わる経路は上書きの行だけ）の TX の中で、変更の後の状態で検査（30 の新契約。`ValidationFailedAt`、field `pos_stock_sync` か `jan_code`）。round 1 の是正で経路を 30 と揃えた。一括 import は、preview で新規の行を CSV の順に先の新規の行を当てた状態で、重複の行を全ての新規の行を当てた状態で判定して当該行だけ error 行にし、確定は送られた行の順に TX のその時点の状態で判定して、当たる行があれば最初の行を名指して `ValidationFailed`（field なし）で確定全体を戻す（30 の D-118 の節。round 2 の是正。`ImportResult` の形は変えない） | `src-tauri/src/biz/product_service.rs` の `create_product`・`update_product`・`preview_import`・`commit_import` | Matrix の J1〜J3 |
| 在庫連動を始める検査 | update と一括 import の上書きの false → true で ADR D8 の条件（`BizError::StocktakeGuard`〈`recount_required`〉と回復先。一括 import は行の error の文）。共有 JAN と両方に当たれば共有 JAN の拒否を先に返す。一括 import の確定の TX で当たれば上の行と同じ形で確定全体を戻す（round 2 の是正）。連動の切替の TX はその商品を `recompute_stock` | 同上 | Matrix の J4〜J6 |
| 共有 JAN の一覧 | `run_integrity_check` に `shared_jan_conflicts`（36 の新契約） | `src-tauri/src/biz/integrity_service.rs` | Matrix の I1・I2 |

### S7 CMD と wire（`cmd`・`lib.rs`・bindings）

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| `CmdErrorKind`・`CmdError` | `stocktake_guard` と `stocktake_recovery: Option<StocktakeRecovery>`（必須の nullable。既存の `field`・`error_id` と同じ形）。`From<BizError>` が `StocktakeGuard` を写す。既存の kind では None | `src-tauri/src/cmd/mod.rs` | `CmdError` の struct の literal 22 か所（`rg -U -c 'CmdError \{\s*\n\s*[a-z_]+:' src-tauri/src` の和 21〈`csv_import_cmd.rs` 4・`daily_report_import_cmd.rs` 5・`cmd/mod.rs` 9・`plu_export_cmd.rs` 1・`product_cmd.rs` 1・`settings_cmd.rs` 1〉と、`cmd/mod.rs` の `Self {` の 1。`rg -c 'CmdError \{'` の 32 は定義・`impl`・戻り型・doc comment を含むので literal の数ではない。round 1 の是正）に `stocktake_recovery: None` を足す（値の期待は変えない。Matrix の T7）。`cmd/mod.rs` の kind の test に 1 行 |
| 棚卸しの command | 新設 4（`save_stock_count`・`preview_stock_count_placement`・`zero_uncounted_items`・`list_recheck_products`。42 の新契約の表）、`update_count` を外す、`complete_stocktake(stocktake_id)`、ほかは出力の型が変わる。CMD は PC の今日（`chrono::Local`）を BIZ へ渡す | `src-tauri/src/cmd/stocktake_cmd.rs`、`src-tauri/src/lib.rs` の 2 つの一覧（`collect_commands` の一覧と、その test の一覧）と CMD-10 の doc comment | Matrix の W1〜W4 |
| Z004 の command | `commit_csv_import(preview_token)`（`additional_import_confirmed` を外す）、token は成功のときだけ消す（同じ日の状況の変化の message の照合を外す）、出力に知らせと `recheck_targets`。preview と確定に今日を渡す | `src-tauri/src/cmd/csv_import_cmd.rs` | Matrix の W5 |
| 生成の enum の test | 新しい有限の値（Z14）を response の enum の round-trip の test に足す | `src-tauri/src/finite_enum_contract_tests.rs` | Matrix の W6 |
| bindings | `cd src-tauri && cargo run --bin generate_bindings` で `src/lib/bindings.ts` を再生成 | `src/lib/bindings.ts`（生成物） | — |

### S8 画面（今の外枠の上に振舞いを入れる。D1 の見た目は入れない）

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| 棚卸しの画面 | 73 の D-118 の節の表（商品を特定・「前の日の分を入れる」・商品の「入れ直す」・「今の数を入れる」・保存・日付の変わり目・要再確認の一覧・完了・完了の結果）と m の入力（SPEC-UNIT-D3 の表の 7）。UI-10-D1〜D13 の今の振舞い（検索・HID・候補・IME・focus・廃番 badge・絞り込み・前回の要約）は保つ。停止の prop と Alert を外す。完了の確認の文に H2 の問いを足し、`completion_blocked` は `block_reason` ごとの一覧と、「取り込んでいない精算がある」の直し方の文（40 の `SettlementPendingHint`。73 の新契約の「完了」の行）、「0 にする」と「残りをまとめて 0 にする」（確認を挟む。日付を選べる）。要再確認の一覧の各行は商品・範囲（`CountRecoveryTarget` の範囲を単位どおり）・理由（round 1 の是正） | `src/features/stocktake/StocktakePage.tsx`（大きくなれば同じ feature の component に分ける）、`hooks/`（`useUpdateCount.ts` を消し、保存・placement・0 にする・要再確認の一覧の hook）、`lib/stocktake-formatters.ts`、`types.ts` | Matrix の U1〜U9・U15・U16。既存の test は Matrix の T5 |
| 「今の数を入れる」の入口 | route `/stocktake/` の search に `product`（商品コード）と `returnTo` を足し（51 の回復先 `active_count`・`count_now` から開く）、保存の後に `returnTo` へ戻る。`returnTo` は `/products/{product と同じ商品コード}/edit` だけを受け、外れれば `/stocktake/` に留まる（73。round 3 の是正）。欄の商品の文脈（商品名・単位・写しの在庫と範囲・前に保存した数）は `preview_stock_count_placement` の結果から出す（棚卸しの明細が無いので `StocktakeItemDetail` から取れない。round 1 の是正） | `src/routes/stocktake/index.tsx` の `searchSchema`、`src/features/stocktake/types.ts` の `StocktakeSearch`（round 2 の是正） | Matrix の U9。route の新設は無い（`npm run generate:routes` は search の変更だけで routeTree が変わらないことを確かめる） |
| Z004 タブ | preview の拒否の文、知らせ、同じ日の追加の確認の dialog を Z004 では出さない、取込みの結果と取消の結果に要再確認の一覧（商品・範囲〈`CountRecoveryTarget` の範囲を単位どおり〉・理由）と棚卸しの画面への導線（55 の新契約）。取消の入口は今のとおり結果の画面の「取り消す」だけ（55 §55.4 の手順 18。取込みの記録の詳細からの取消は Non-scope）。停止の Alert を外す | `src/features/csv-import/` の `CsvImportPage.tsx`・`components/PreviewStep.tsx`・`components/ResultStep.tsx`・`hooks/useCsvImportFlow.ts`・`reducer.ts`・`types.ts` | Matrix の U10・U11。既存の test は Matrix の T6 |
| 商品フォーム | 共有 JAN の拒否（field 付きの validation の今の出し方）と、`stocktake_guard`（`recount_required`）の拒否の文と「今の数を入れる」の導線（51 の新契約の 1・2 つ目の項目。「在庫数の表示」の項目は b2）。導線は今の離脱の確認（`useUnsavedChangesWarning`）を通る。guard を外さず、未保存の入力を持ち越さない（51。round 3 の是正） | `src/features/products/ProductFormPage.tsx`、`src/features/products/ProductFormPage.unsaved-guard.test.tsx` | Matrix の U12 |
| 商品一括 import | 行の error の文は今の preview の表示で出る（BIZ の文）。確定の TX の拒否（kind `validation`、field なし）は今の error の Alert（`describeError` で BIZ の文）に出て、「戻る」で preview と上書きの選択に戻る（`useProductImportFlow.ts` の `decideRecoverTo` が `validation` を preview に戻す今の分岐。60 の D-118 の節）。画面・reducer・wire は変えず、test だけを足す（round 2 の是正で決めた） | `src/features/products/ProductImportPage.test.tsx` | Matrix の J3 の画面側 |
| 棚卸しの記録の詳細 | 明細ごとの数（日付・入れ直しか）・差異・評価数量・ロス原価（BIZ の十進の文字列）、「完了の後に入った記録による差（参考。評価額は変わりません）」の確定の差と要再確認の差、「在庫変動」の節を外す（65） | `src/features/inventory-records/StocktakeRecordDetailPage.tsx` | Matrix の U13。既存の test は Matrix の T8 |
| 整合性の画面 | 共有 JAN の一覧の欄（75） | `src/features/integrity-check/IntegrityCheckPage.tsx` | Matrix の U14 |
| error の表示 | `CMD_ERROR_KIND` に `STOCKTAKE_GUARD`、`describeError` が `stocktake_guard` を扱う（回復先は画面が型で分岐し、message を解析しない） | `src/lib/invoke.ts`、`src/lib/describe-error.ts` | Matrix の W7。TS の `CmdError` の mock の写しは T7 |
| 再取得 | UI_TECH_STACK の D-118 の段落の経路と entry のとおり、D-052 の SSOT（`invalidation-contract.ts`）の entry に要再確認の一覧の新しい key `queryKeys.stocktake.recheckList()` を足し（`stocktakeStart`・`stocktakeCountUpdate`・`stocktakeComplete`・`csvImportCommit`・`csvImportRollback`・`receiving`・`returnExchange(false)`・`manualSale`・`disposal`・`productUpdate`・`productImport`・`integrityFix` の 12）、`stocktakeCountUpdate` に写しの consumer（商品一覧・在庫少・在庫照会・在庫変動・商品フォーム）を足す。「0 にする」は `stocktakeCountUpdate` を使う（entry の数は 22 のまま）。独立の oracle はその段落から写す。hook は今のとおり `invalidateByContract` だけを呼ぶ（round 2 の是正で source を D-052 の SSOT に直した） | `src/lib/invalidation-contract.ts`、`src/lib/query-keys.ts`、`src/test/invalidation-oracle.ts`。meta test（`src/lib/invalidation-contract.meta.test.ts`）は entry の名前と数を変えないので変えない | Matrix の U15 |

### S9 撤去・docs・生成物・L3 の fixture

- 撤去: 上の「この変更で不要になるもの」の全件。
- 正本の同期（実装と同じ PR）: b1 の分の「proposed・未実装」の注記を現行の記述に直す（ADR の Status、tracking・pos・master・transaction、20・21・24・30・31・32・35・36・40・41・42・51・55・60・65・73・75 の D-118 の節、`DB_DESIGN.md`・`ARCHITECTURE.md`・`FUNCTION_DESIGN.md`・`SCREEN_DESIGN.md`・`UI_TECH_STACK.md`・`architecture/*-task-specs.md` の該当の行、61〜64 の注記の「(b) は proposed」）。b2 の分は proposed のまま残し、各節に「b1 で入った所」を 1 文で書く。停止 ADR を superseded にし（Status と、置き換えた先）、`docs/adr/README.md` の一覧を同期。35 §20.0・32 §15.0・55 §55.0 の停止の節・73 §73.15 を外し、35・32 の旧本体の処理ステップを新しい経路の手順に置き換える（42・41 の旧 command の記述も）。新しい関数のシグネチャを 21・24・20・31・35・32 の code block に置く。42 に新しい DTO の field を写す（下の Boundary）。73 の Windows L3 の一覧から (e) を自動 test へ移したことを書く。10 の「入力の上限と安全な整数の範囲」の (c) の表の「棚卸しの確定」の状態と、棚卸しの再開（⑤）の項目を現行に直す。22 §3.1 の module の一覧に `schema_v{N}.rs`。時点証拠 ADR の Evidence の probe の link を撤去の文に。`docs/diagrams/current-system.md`・`cross-feature-verification.md`・`docs/project-memory.md` の停止の記述を現行に。`docs/SCREEN_DESIGN.md` の停止の 2 行（売上データ取込みの「現行 build では一時停止中（Z004 の取込みの確定・取消）」と、棚卸しの「現行 build では一時停止中（棚卸しの開始・カウント入力・確定）」。2026-10-22 の 122・213 行目）を外す（AC11 の `一時停止中` の検索が docs 全体〈archive・plans・decision-log・backlog を除く〉で 0。round 2 の是正）。decision-log は D-109 の (2)(3)（`#[cfg(test)]` の置き場）が外れたことを Superseded in part の 1 行で書く。撤去した code の symbol と file の名前（AC11 の撤去した名前の検索の語）は、archive・plans・research・superseded の ADR 2 本・decision-log・backlog を除く docs から消す（「外した」と書くときも名前を引かない。2026-10-22 の baseline の 13 file は下の AC の baseline。上の一覧に無い所は [22](../../function-design/22-mnt-migration.md) §16 の時点証拠の schema の文と、[20](../../function-design/20-io-product-repo.md) の完了の読取り〈`StocktakeItemForComplete`〉。生成物の 90 は再生成で消える。round 3 の是正）。
- demo の seed（round 2 の是正）: `src-tauri/src/seed_demo.rs` の `delete_all` の表の先頭に `stock_counts` を足す（`stock_counts` は `products` と `stocktake_items` を外部キーで参照するので先に消す。足さないと、b1 で計数を保存した開発 DB の `cargo run --bin seed_demo_data -- --reset` が外部キーの失敗で止まる）。test は Matrix の E1（`src-tauri/tests/seed_test.rs`）。
- 生成物: `src/lib/bindings.ts`（S7）、`docs/function-design/90-traceability.md`（`cd src-tauri && cargo run --bin generate_traceability`。手で直さない）。
- L3 の fixture（合成の値だけ。TD-296。round 1 の是正で作り方を 1 通りに決めた）: 精算日を L3 の日 T からの相対で決めるので、file は tracked にせず、tracked の生成の test から作る。**Writer**（実装の S9）が `src-tauri/src/io/l3_fixtures.rs`（`#[cfg(test)]` の module。`src-tauri/src/io/mod.rs` に宣言）に、(a) 下の Test Plan の「L3 の fixture の表」の 9 file（商品一括 import の CSV 2 つと layout A の Z004 7 つ。CP932・CRLF、マシンNo. `0001`。商品の CSV の部門ID は作り直した空の DB にある初期の部門 `1`〈`その他小物`。`src-tauri/src/db/schema_v1.rs` の `departments` の初期データ〉）を T から作る関数、(b) 固定の T（`2026-01-15`）で一時の folder に作り、各 file を `io::z004_parser::parse_z004` と `io::product_csv_importer::parse_product_csv` で読み戻して、表の精算回数・精算日・行・商品を確かめる通常の test（CI で生成の誤りを止める）、(c) env の `INVENTORY_L3_DAY`（`YYYY-MM-DD`）と `INVENTORY_L3_OUT`（出力の folder）を受けて書き、同じ読み戻しを当てる `#[ignore]` の test（手元で走らせる。先例は `src-tauri/src/io/ej_parser.rs` の `real_ej_structure_probe`）を置き、`tests/fixtures/z004/README.md` に表と走らせ方を書く。**Coordinator** は L3 を頼む日（Ready の依頼と同じ日）に `cd src-tauri && INVENTORY_L3_DAY=その日 INVENTORY_L3_OUT=repo の外の folder CARGO_TARGET_DIR=/tmp/shared-cargo-target cargo test --lib -- --ignored io::l3_fixtures` で作り、owner に Windows の path を省略せずに渡す（owner は file を作らない）。今の従来の形の fixture（`normal-small.csv` ほか、tracked）は、精算回数の無い file の拒否の入力として残す（README の用途の列を直す）。

### 節の所有表（並走の lane と重なる file）

`comm -12` の結果（2026-10-22、round 1 の是正で取り直した。本 packet の予定 path 138〈round 2 の是正の前〉と、PR #155〈`origin/agent/backup-offsite` `e6158aa6`〉の差分 14・PR #172〈`origin/agent/sd-direct-read-runtime` `722f1b46`〉の差分 9）: #155 と 7 file（`docs/ARCHITECTURE.md`・`docs/FUNCTION_DESIGN.md`・`docs/SCREEN_DESIGN.md`・`docs/architecture/cmd-task-specs.md`・`docs/architecture/mnt-task-specs.md`・`docs/decision-log.md`・`docs/function-design/22-mnt-migration.md`）、#172 と 3 file（`docs/db-design/pos-tables.md`・`docs/function-design/22-mnt-migration.md`・`docs/function-design/55-ui-csv-import.md`）。round 2 の是正の後の予定 path 147 で取り直しても同じ 7 file と 3 file（#155 は remote-tracking の `db83fa66`〈差分 15〉、#172 は `722f1b46`〈差分 9〉。round 2 で足した 9 path と、round 2 で直した正本〈30・35・42・60・`UI_TECH_STACK.md`〉はどちらとも重ならない）。round 3 の是正の後の 148 でも同じ 7 file と 3 file（#155 は remote-tracking の `ee10506d`〈差分 15〉、#172 は `722f1b46`〈差分 9〉。足した 1 path は TS の test。round 3 で直した正本のうち `pos-tables.md` は冒頭の D-118 の節、`22-mnt-migration.md` は §20 の手順 5 だけで、所有表の範囲の内）。重なりを避けるために本 lane の範囲は変えない。後に merge する側が `origin/main` の単段 merge（`git -c merge.directoryRenames=false merge origin/main`）で同期する（`docs/DEV_WORKFLOW.md` Wave Operation）。

| file | 並走の lane が所有する節 | 本 lane が触る所 | merge の後に揃える所 |
|---|---|---|---|
| `docs/function-design/22-mnt-migration.md` | #155: §3.2（`migrate` の段落）。#172: §17（MNT-03-D15、日報の束の精算回数の migration） | §20（新設、MNT-03-D18）と §3.1 の module の一覧（S9） | 末尾に節を足すので、後から merge する側が §17・§18・§19・§20 の順に並べる。#172 の §17 の文の「棚卸しの … runtime の migration が先に merge したら番号を取り直す」は vZ を指す |
| `src-tauri/src/db/migration.rs`・`src-tauri/src/db/mod.rs`（#172 の実装で触る） | #172: `migrations()` の登録と版の期待の行 | 登録・版の期待の行（S1） | 版の番号は先に merge した側が若い番号。後の側が file 名・登録の順・版の定数を取り直す |
| `docs/db-design/pos-tables.md` | #172: §12b（`daily_report_imports.settlement_no`）と更新履歴の末尾 | 冒頭の D-118 の節（S9 で現行に） | 更新履歴の末尾への追記は merge の順で両方を残す |
| `docs/function-design/55-ui-csv-import.md` | #172: UI-07-D16（日報取込みタブの選択の一覧）と更新履歴の末尾 | 冒頭の D-118 の節、§55.0 の停止の節、Z004 タブの preview と結果の節（S9） | 同上 |
| `docs/ARCHITECTURE.md`・`docs/FUNCTION_DESIGN.md`・`docs/SCREEN_DESIGN.md`・`docs/architecture/cmd-task-specs.md`・`docs/architecture/mnt-task-specs.md` | #155: backup の外の控えの行（各文書の後半の backup の節） | 冒頭の D-118 の節と、棚卸し・Z004 の行（S9） | 別の段落。衝突したら両方を残す |
| `docs/decision-log.md` | #155: D-114（番号順の位置に新設） | D-109 の末尾の Superseded in part の 1 行（S9） | 別の決定の節 |
| `docs/backlog.md` | #155・#172 の各項目 | 本 lane の PR では触らない（(b) の項目の完了の整理・b2 の項目・閉じる項目は closeout。D-124 (4)） | — |
| `docs/Plans.md` | （lane の行は足さない。D-097） | `## 次の行動` と `## 導入の前の関門` の 2. の pointer の句（本 commit） | — |

JAN の別名の runtime（D-122、未起票）は `product_service.rs`・30 §4.8・§4.9・22 §18 の migration を触る。b1 の後に起票するなら b1 の共有 JAN の検査（S6）と同じ関数に別名の規則が載る。

### 予定 path の一覧（`comm -12` の入力）

148 path（Rust 60・TS 36・docs 50・ほか 2。起票の時点の 127 に、round 1 の是正で `NewSaleRecord` を作る所 3〈`manual_sale.rs`・`sales_service.rs`・`sales_cmd.rs`〉、`apply_stock_change` の引数の撤去 4〈`common.rs`・`receiving.rs`・`returns.rs`・`disposal.rs`〉、漏れた test 2〈`list_tests.rs`・`import_internal_contract_test.rs`〉、L3 の fixture の生成 2〈`io/l3_fixtures.rs`・`io/mod.rs`〉を足した。round 2 の是正で、demo の seed 2〈`seed_demo.rs`・`seed_test.rs`〉、D-052 の SSOT 3〈`invalidation-contract.ts`・`query-keys.ts`・`invalidation-oracle.ts`〉、棚卸しの route の search の型 1〈`stocktake/types.ts`〉、商品一括 import の確定の拒否の画面の test 1〈`ProductImportPage.test.tsx`〉、最新の版を期待する既存の test の file 2〈`schema_v9.rs`・`schema_v10.rs`。Matrix の T1〉を足して 147。round 3 の是正で、商品フォームの離脱の確認の test 1〈`ProductFormPage.unsaved-guard.test.tsx`〉を足して 148）。このほかに、新設の file（fixture の Z004 と商品の CSV、計数の保存・placement・0 にする・要再確認の一覧の hook と component、`stocktake_service` を module に分けたときの file）と、Matrix の T7 の TS の `CmdError` の mock を持つ file（`rg -l 'error_id: null' src --glob '*.ts*' | wc -l` → 34）と、Codex の closure の Gated Amendment で足した既存の test の 2 file（`src-tauri/src/db/schema_v2.rs`〈Matrix の T1〉・`src-tauri/tests/architecture_test.rs`〈Registration の D-083 の行〉）が加わる。所有表の突き合わせは L3 を頼む時点の head で取り直す（D-124）。

```
docs/ARCHITECTURE.md
docs/DB_DESIGN.md
docs/FUNCTION_DESIGN.md
docs/Plans.md
docs/SCREEN_DESIGN.md
docs/UI_TECH_STACK.md
docs/adr/2026-09-18-stocktake-time-evidence.md
docs/adr/2026-09-23-legacy-stocktake-z004-write-stop.md
docs/adr/2026-10-16-stocktake-count-placement.md
docs/adr/README.md
docs/architecture/biz-task-specs.md
docs/architecture/cmd-task-specs.md
docs/architecture/io-task-specs.md
docs/architecture/mnt-task-specs.md
docs/architecture/ui-task-specs.md
docs/db-design/master-tables.md
docs/db-design/pos-tables.md
docs/db-design/tracking-system-tables.md
docs/db-design/transaction-tables.md
docs/decision-log.md
docs/diagrams/cross-feature-verification.md
docs/diagrams/current-system.md
docs/function-design/10-common-rules.md
docs/function-design/20-io-product-repo.md
docs/function-design/21-io-inventory-repo.md
docs/function-design/22-mnt-migration.md
docs/function-design/24-io-csv-import-repo.md
docs/function-design/30-biz-product-service.md
docs/function-design/31-biz-inventory-service.md
docs/function-design/32-biz-csv-import-service.md
docs/function-design/33-biz-plu-export-service.md
docs/function-design/35-biz-stocktake-service.md
docs/function-design/36-biz-integrity-check.md
docs/function-design/40-cmd-product.md
docs/function-design/41-cmd-pos.md
docs/function-design/42-cmd-sales-stocktake.md
docs/function-design/51-ui-product-form.md
docs/function-design/55-ui-csv-import.md
docs/function-design/60-ui-product-import.md
docs/function-design/61-ui-receiving.md
docs/function-design/62-ui-manual-sale.md
docs/function-design/63-ui-return-exchange.md
docs/function-design/64-ui-disposal.md
docs/function-design/65-inventory-record-traceability.md
docs/function-design/73-ui-stocktake.md
docs/function-design/75-ui-integrity-check.md
docs/function-design/90-traceability.md
docs/plans/2026-10-22-stocktake-count-runtime-b1.md
docs/plans/test-matrices/2026-10-22-stocktake-count-runtime-b1.md
docs/project-memory.md
scripts/probes/stocktake_time_model.py
src-tauri/src/biz/csv_import_service/commit.rs
src-tauri/src/biz/csv_import_service/mod.rs
src-tauri/src/biz/csv_import_service/parse.rs
src-tauri/src/biz/csv_import_service/rollback.rs
src-tauri/src/biz/csv_import_service/test_support.rs
src-tauri/src/biz/csv_import_service/tests/commit_tests.rs
src-tauri/src/biz/csv_import_service/tests/cross_feature_tests.rs
src-tauri/src/biz/csv_import_service/tests/legacy_stop_tests.rs
src-tauri/src/biz/csv_import_service/tests/list_tests.rs
src-tauri/src/biz/csv_import_service/tests/mod.rs
src-tauri/src/biz/csv_import_service/tests/parse_tests.rs
src-tauri/src/biz/csv_import_service/tests/rollback_tests.rs
src-tauri/src/biz/csv_import_service/tests/time_evidence_tests.rs
src-tauri/src/biz/csv_import_service/time_evidence.rs
src-tauri/src/biz/integrity_service.rs
src-tauri/src/biz/inventory_service/common.rs
src-tauri/src/biz/inventory_service/disposal.rs
src-tauri/src/biz/inventory_service/manual_sale.rs
src-tauri/src/biz/inventory_service/receiving.rs
src-tauri/src/biz/inventory_service/returns.rs
src-tauri/src/biz/mod.rs
src-tauri/src/biz/product_service.rs
src-tauri/src/biz/sales_service.rs
src-tauri/src/biz/stock_ledger.rs
src-tauri/src/biz/stocktake_service.rs
src-tauri/src/biz/stocktake_service/time_evidence.rs
src-tauri/src/biz/stocktake_service/time_evidence_tests.rs
src-tauri/src/cmd/csv_import_cmd.rs
src-tauri/src/cmd/daily_report_import_cmd.rs
src-tauri/src/cmd/disposal_cmd.rs
src-tauri/src/cmd/inventory_cmd.rs
src-tauri/src/cmd/manual_sale_cmd.rs
src-tauri/src/cmd/mod.rs
src-tauri/src/cmd/plu_export_cmd.rs
src-tauri/src/cmd/product_cmd.rs
src-tauri/src/cmd/receiving_cmd.rs
src-tauri/src/cmd/return_cmd.rs
src-tauri/src/cmd/sales_cmd.rs
src-tauri/src/cmd/settings_cmd.rs
src-tauri/src/cmd/stocktake_cmd.rs
src-tauri/src/db/disposal_repo.rs
src-tauri/src/db/inventory_repo.rs
src-tauri/src/db/migration.rs
src-tauri/src/db/mod.rs
src-tauri/src/db/product_repo.rs
src-tauri/src/db/sales_repo.rs
src-tauri/src/db/schema_time_evidence.rs
src-tauri/src/db/schema_v10.rs
src-tauri/src/db/schema_v9.rs
src-tauri/src/db/schema_v11.rs（番号は merge の時点の次）
src-tauri/src/db/stocktake_repo.rs
src-tauri/src/db/test_support.rs
src-tauri/src/finite_enum_contract_tests.rs
src-tauri/src/io/l3_fixtures.rs
src-tauri/src/io/mod.rs
src-tauri/src/lib.rs
src-tauri/src/seed_demo.rs
src-tauri/tests/design_compliance_test.rs
src-tauri/tests/import_internal_contract_test.rs
src-tauri/tests/seed_test.rs
src/features/csv-import/CsvImportPage.test.tsx
src/features/csv-import/CsvImportPage.tsx
src/features/csv-import/components/PreviewStep.test.tsx
src/features/csv-import/components/PreviewStep.tsx
src/features/csv-import/components/ResultStep.test.tsx
src/features/csv-import/components/ResultStep.tsx
src/features/csv-import/hooks/useCsvImportFlow.test.tsx
src/features/csv-import/hooks/useCsvImportFlow.ts
src/features/csv-import/reducer.test.ts
src/features/csv-import/reducer.ts
src/features/csv-import/types.ts
src/features/integrity-check/IntegrityCheckPage.test.tsx
src/features/integrity-check/IntegrityCheckPage.tsx
src/features/inventory-records/StocktakeRecordDetailPage.test.tsx
src/features/inventory-records/StocktakeRecordDetailPage.tsx
src/features/products/ProductFormPage.test.tsx
src/features/products/ProductFormPage.tsx
src/features/products/ProductFormPage.unsaved-guard.test.tsx
src/features/products/ProductImportPage.test.tsx
src/features/stocktake/StocktakePage.suggest.test.tsx
src/features/stocktake/StocktakePage.test.tsx
src/features/stocktake/StocktakePage.tsx
src/features/stocktake/hooks/useCompleteStocktake.ts
src/features/stocktake/hooks/useStocktakeItems.ts
src/features/stocktake/hooks/useUpdateCount.ts
src/features/stocktake/lib/stocktake-formatters.test.ts
src/features/stocktake/lib/stocktake-formatters.ts
src/features/stocktake/types.ts
src/lib/bindings.ts
src/lib/describe-error.test.ts
src/lib/describe-error.ts
src/lib/invalidation-contract.ts
src/lib/invoke.ts
src/lib/query-keys.ts
src/routes/stocktake/index.tsx
src/test/invalidation-oracle.ts
tests/fixtures/z004/README.md
```

## 後続の lane

### b2: 要再確認の範囲と計数の履歴を商品の画面で見せる（R3 見込み）

- 範囲（ADR D12 の「(b) を 2 つの lane に分けた」）: [20](../../function-design/20-io-product-repo.md) の商品の読取りが写しの 3 列（`stock_recheck`・`stock_range_low`・`stock_range_high`）を返すこと、[40](../../function-design/40-cmd-product.md) の商品の get / search の結果（`ProductWithRelations` と一覧の行、在庫照会の詳細）への 3 列、[50](../../function-design/50-ui-product-list.md)・[58](../../function-design/58-ui-stock-inquiry.md)・[51](../../function-design/51-ui-product-form.md) の在庫の欄の範囲と「要再確認」の表示（色だけで示さない）、[65](../../function-design/65-inventory-record-traceability.md) の D-118 の節の 1 つ目の項（在庫変動の履歴に計数の行〈「数を確認（{数}、{日付}）」・「数を入れ直し（{数}、{日付}）」〉を movement と並びの鍵の順に混ぜる）と [66](../../function-design/66-ui-stock-movements.md)（見出しの現在庫の範囲）、[21](../../function-design/21-io-inventory-repo.md) の `MovementRecord` の並びの鍵と [44](../../function-design/44-cmd-inventory.md) の `list_movements` の結果の型。
- 入らないもの: migration（列は v9 にある）、書く経路（b1）。
- Risk: R3（Tauri command の DTO・generated bindings・operator の画面。`docs/DEV_WORKFLOW.md` Risk Tiers の R3 の行。値を書き換える migration も破壊的な操作も無い）。Human Gate は `ready,merge,manual`（画面の変更の Human Visual Confirmation と L3）。Final Review Minimum 1。
- 順: b1 の merge の後に起票する（L3 に要る要再確認の商品と計数の記録は b1 の書く経路で作る。b1 と同時に作ると `src/lib/bindings.ts`・51 の画面を両方が触り、L3 を束ねられない）。b2 の merge で関門 2 を満たす。
- 起票時に確かめること: 在庫少の判定・並べ替え・ホームの件数は写しの値のまま（50・58 の注記）。在庫変動の履歴の計数の行の読取りの合成を IO と BIZ のどちらに置くか（21・31）。

## Non-scope

- 上の Goal の「非目的」の全部（b2、P の小数、D1 の見た目、Z004 の SD の一括取込み〈(B)〉、EJ、JAN の別名、振替の `movement_type`、`register_processed` の既定、TD-333 の点数の検査）。TD-333（レジの打ち方と Z004 の数量の入れ替わり）は設計の種で決定ではなく、どの行を「不自然」とするかに店の事実が要る。b1 の契約の正しさ（記録と写しの一致）は変えないので backlog に残す。
- 日報の取込みの同日追加の確認（BIZ-08。`AdditionalImportConfirmDialog` と日報の `additional_import_confirmed`）。SD 直読みの runtime（PR #172）の範囲。
- 取込みの記録の詳細（`/csv-import/records/$importId`）から過去の Z004 の取込みを取り消す入口（round 1 の是正で判断した）。正本は決めていない: 55 は取消の画面の入口を取込みの結果の step だけとし（§55.0 の「取消の画面入口は結果 step だけ」、§55.4 の手順 18 の「取り消す」CTA）、65 は CSV 取込みの詳細に「rollback 状態」を出すとだけ書き、取消・訂正の CTA は「対象記録種別の cancel / correct command が実装済みの場合だけ表示する」の完成形の契約（業務記録の取消の lane。今は無い）に置く。ADR D7 と 24 は取消の処理だけを決め、入口を決めていない。b1 は今の入口（結果の画面の「取り消す」）を新しい経路に繋ぐだけにし、L3-10 も結果の画面で取り消す。残る使い手の穴（取り込んだ後に結果の画面を離れると、誤った版を取り消して取り込み直す〈ADR D7〉入口が無い）は下の「残りの危険」に書く。owner は 2026-10-22 に、この入口を「導入の前の関門」に入れると決めた（repo 外の回答台帳 TD-380。D-127 の「その場で直せる」に当たる）。b1 の範囲は変えず、別の lane で入れる。closeout で [Plans](../../Plans.md) の `## 導入の前の関門` に項目を、backlog に「Z004 の取込みの記録の詳細から取り消す入口」を足す。
- 「店に確かめる問い（入庫・返品の記録を品物が届く前に入力することはあるか）」の答え（ADR の Revisit。b1 の契約を変えない）。
- 停止 ADR の撤去で外す以外の旧い記録の型（`movement_type` の `stocktake`・`ReferenceType::Stocktake` の enum と画面の label）。
- 起動の失敗の画面の文の種類を増やすこと（vZ の止まりは今の `DatabaseInit` の文に手順 1 の message を添える。(a) の Residual と同じ）。

## Acceptance Criteria

- AC1（migration）: `cd src-tauri && cargo test --lib db::` が全 PASS し、Matrix の M1〜M9 の test 名が `rg -n 'fn <名前>' src-tauri/src/db` で 1 件ずつ見つかる。
- AC2（精算の窓）: `cd src-tauri && cargo test --lib biz::stock_ledger` が全 PASS し、Matrix の L1〜L13 の test 名が見つかる。`rg -c 'phase != 0' src-tauri/src/biz/stock_ledger.rs` が 0（exit 1）、`rg -c 'RecheckReason::UndeterminedSettlement' src-tauri/src/biz/stock_ledger.rs` が 1 以上、`rg -n "reference_type IS NULL AND movement_type = 'receiving'" src-tauri/src/db/inventory_repo.rs | rg -v '^[0-9]+:\s*//' | wc -l` が 1（`//`・`///` で始まる comment の行を除いた SQL の行だけを数える。S2 の doc comment に同じ文字列を写しても数が変わらない。round 3 の是正）。
- AC3（計数・完了・記録の詳細）: `cd src-tauri && cargo test --lib biz::stocktake_service` が全 PASS し、Matrix の C1〜C11・K1〜K8・R1〜R6・W3 の test 名が見つかる（W3 は round 2 の是正で BIZ の test にした）。`rg -n 'pub fn (save_stock_count|preview_stock_count_placement|zero_uncounted_items|list_recheck_products)\(' src-tauri/src/biz | wc -l` が 4。`rg -c "reference_type = 'stocktake'" src-tauri/src/db/disposal_repo.rs src-tauri/src/db/stocktake_repo.rs` が 0（exit 1）。
- AC4（Z004）: `cd src-tauri && cargo test --lib biz::csv_import_service` が全 PASS し、Matrix の Z1〜Z16・Z21〜Z24・X1〜X3 の test 名が見つかる。`rg -c '#\[ignore' src-tauri/src/biz` が 0（exit 1）。
- AC5（共有 JAN・在庫連動・整合性）: `cd src-tauri && cargo test --lib -- biz::product_service biz::integrity_service` が全 PASS し、Matrix の J1〜J7・I1・I2 の test 名が見つかる。
- AC6（撤去）: `rg -n 'fn legacy_(start_stocktake|update_count|complete_stocktake|commit_csv_import|rollback_csv_import)\(' src-tauri/src | wc -l` が 0、`rg -l 'time_evidence|stock_revision|pos_import_sources|stocktake_recount|observation_kind|stocktake_adjustment_kind' src-tauri/src scripts | wc -l` が 0、`rg -l '一時停止中' src src-tauri/src --glob '!src/lib/bindings.ts' | wc -l` が 0、`test -d scripts/probes && echo present || echo absent` が `absent`、`rg -l 'updateCount|update_count|forceFill|force_fill' src src-tauri/src --glob '!src/lib/bindings.ts' | wc -l` が 0、`rg -l 'additionalImportConfirmed' src/features/csv-import | wc -l` が 0、`rg -l 'apply_void_stock_corrections|StockCorrection' src src-tauri/src --glob '!src/lib/bindings.ts' | wc -l` が 0。
- AC7（wire と command）: `rg -c '"stocktake_guard"' src/lib/bindings.ts` が 1 以上、`rg -c '^export type (SaveStockCountRequest|StockCountSaveResult|CountPlacementPreview|CountRecoveryTarget|StocktakeRecovery|SharedJanConflict|SettlementGap) =' src/lib/bindings.ts` が 7、`rg -c 'cmd::stocktake_cmd::(save_stock_count|preview_stock_count_placement|zero_uncounted_items|list_recheck_products),' src-tauri/src/lib.rs` が 8（2 つの一覧に 4 つずつ）。`cd src-tauri && cargo test --lib -- cmd:: finite_enum_contract_tests` が全 PASS し、Matrix の W1・W2・W4〜W6・W8 の test 名が見つかる。
- AC8（画面）: `npm test -- src/features/stocktake src/features/csv-import src/features/products src/features/inventory-records src/features/integrity-check src/features/receiving src/features/disposal src/features/manual-sale src/features/return-exchange src/lib` が全 PASS し（round 2 の是正で U15 が引く入庫・廃棄・手動販売・返品交換の feature を足した）、Matrix の U1〜U16・W7 と J3 の画面の test 名が見つかる。
- AC9（gate）: `cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`、`cargo run --bin generate_bindings` の後に `git diff --exit-code src/lib/bindings.ts`、`cargo run --bin generate_traceability -- --check`、`cargo test --test design_compliance_test`、`npm run typecheck && npm run lint && npm run format:check && npm test && npm run build`、`bash scripts/doc-consistency-check.sh` と `--target plan` が exit 0。`cd src-tauri && cargo test --test seed_test` が全 PASS し、Matrix の E1 の test 名が `rg -n 'fn <名前>' src-tauri/tests/seed_test.rs` で見つかる（round 2 の是正）。L3 を含むので Writer は `cd src-tauri && cargo check --release` を通す。
- AC10（既存の test の移し方）: Matrix の T1〜T12 の command を実装の後に逐語で回し、出力の各行（test の名前）を T1〜T12 の分類（移す / 期待を変える / 契約ごと外す）に当てた表を PR の本文に貼る。「移す」の test で値の期待（`assert` の右辺・件数・message）が変わっていない。「契約ごと外す」の各行に置き換えた ADR・設計正本の節と、代わりに同じ失敗を止める Matrix の行を書く。
- AC11（正本）: `rg -c '^superseded' docs/adr/2026-09-23-legacy-stocktake-z004-write-stop.md` が 1、`rg -l '一時停止中' docs --glob '!docs/archive/**' --glob '!docs/plans/**' --glob '!docs/decision-log.md' --glob '!docs/backlog.md'` が 0 件（exit 1。round 2 の是正で `docs/function-design` から docs 全体へ広げた。`docs/SCREEN_DESIGN.md`・`docs/diagrams/current-system.md` を含む）、`rg -n 'proposed・未実装' docs/function-design/32-biz-csv-import-service.md docs/function-design/35-biz-stocktake-service.md docs/function-design/42-cmd-sales-stocktake.md docs/function-design/73-ui-stocktake.md` が b1 の分で 0 行（b2・P・(B) の分の注記だけが残る。残る行ごとに b2・P・(B) のどれかを書く）。撤去した名前の検索（round 3 の是正。S9 の撤去の一覧の code の symbol と file の名前。日報の取込みが使い続ける名前〈`active_same_date_import_ids`・`DailyReportDuplicateCheck` ほか〉と、「持たない」の文に残る field・command の名前〈`force_fill`・`update_count` ほか〉は語に入れない）: `rg -l 'WRITES_SUSPENDED|IMPORT_SUSPENDED|BIZ0[36]_STOP|writes_?[sS]uspended|legacy_(start_stocktake|update_count|complete_stocktake|commit_csv_import|rollback_csv_import|stop_tests)|apply_void_stock_corrections|StockCorrection|get_stocktake_items_for_complete|StocktakeItemForComplete|time_evidence|LEGACY_ITEM_KINDS|seed_legacy_fixture|stocktake_time_model|UpdateCount(Request|Result)|useUpdateCount|AdjustedItem|SameDateCsvImportSummary' docs --glob '!docs/archive/**' --glob '!docs/plans/**' --glob '!docs/research/**' --glob '!docs/adr/2026-09-18-stocktake-time-evidence.md' --glob '!docs/adr/2026-09-23-legacy-stocktake-z004-write-stop.md' --glob '!docs/decision-log.md' --glob '!docs/backlog.md'` が 0 件（exit 1。decision-log と backlog は上の `一時停止中` の検索と同じく履歴と closeout の文書なので除く）。
- AC12（L3 の fixture の生成。round 1 の是正）: `cd src-tauri && cargo test --lib io::l3_fixtures` が全 PASS し（固定の T で 9 file を作って読み戻す通常の test。Matrix の F1）、`rg -c '#\[ignore' src-tauri/src/io/l3_fixtures.rs` が 1（手元で T を与えて作る test）、`tests/fixtures/z004/README.md` に Test Plan の「L3 の fixture の表」と走らせ方がある（`rg -c 'INVENTORY_L3_DAY' tests/fixtures/z004/README.md` が 1 以上）。

AC の数は AC1〜AC12 の 12（round 1 の是正で AC12 を足した）。

AC の数の baseline（2026-10-22、round 3 の是正の作業木〈HEAD `c7468bd1` に本 commit の docs の差分〉で取り直した。code は main `c98fe7dd` と同じ。round 3 の是正で AC2 の `inventory_repo.rs` の command を SQL の行だけを数える形に替え、AC11 に撤去した名前の検索を足した。逐語で実行した）:

```
$ rg -c 'phase != 0' src-tauri/src/biz/stock_ledger.rs
1
exit=0
$ rg -c 'RecheckReason::UndeterminedSettlement' src-tauri/src/biz/stock_ledger.rs
exit=1
$ rg -n 'pub fn (save_stock_count|preview_stock_count_placement|zero_uncounted_items|list_recheck_products)\(' src-tauri/src/biz | wc -l
0
exit=0
$ rg -c '#\[ignore' src-tauri/src
src-tauri/src/biz/csv_import_service/tests/cross_feature_tests.rs:2
src-tauri/src/io/ej_parser.rs:1
exit=0
$ rg -n 'fn legacy_(start_stocktake|update_count|complete_stocktake|commit_csv_import|rollback_csv_import)\(' src-tauri/src | wc -l
5
exit=0
$ rg -l 'time_evidence|stock_revision|pos_import_sources|stocktake_recount|observation_kind|stocktake_adjustment_kind' src-tauri/src scripts | wc -l
14
exit=0
$ rg -l '一時停止中' src src-tauri/src --glob '!src/lib/bindings.ts' | wc -l
10
exit=0
$ rg -c '"stocktake_guard"' src/lib/bindings.ts
exit=1
$ rg -c '^export type (SaveStockCountRequest|StockCountSaveResult|CountPlacementPreview|CountRecoveryTarget|StocktakeRecovery|SharedJanConflict|SettlementGap) =' src/lib/bindings.ts
exit=1
$ rg -c 'cmd::stocktake_cmd::(save_stock_count|preview_stock_count_placement|zero_uncounted_items|list_recheck_products),' src-tauri/src/lib.rs
exit=1
$ rg -l 'updateCount|update_count|forceFill|force_fill' src src-tauri/src --glob '!src/lib/bindings.ts' | wc -l
14
exit=0
$ rg -l 'additionalImportConfirmed' src/features/csv-import | wc -l
5
exit=0
$ rg -n "reference_type IS NULL AND movement_type = 'receiving'" src-tauri/src/db/inventory_repo.rs | rg -v '^[0-9]+:\s*//' | wc -l
0
exit=0
$ rg -c "reference_type = 'stocktake'" src-tauri/src/db/disposal_repo.rs src-tauri/src/db/stocktake_repo.rs
src-tauri/src/db/stocktake_repo.rs:2
src-tauri/src/db/disposal_repo.rs:2
exit=0
$ rg -c '^superseded' docs/adr/2026-09-23-legacy-stocktake-z004-write-stop.md
exit=1
$ test -d scripts/probes && echo present || echo absent
present
exit=0
$ rg -l '一時停止中' docs --glob '!docs/archive/**' --glob '!docs/plans/**' --glob '!docs/decision-log.md' --glob '!docs/backlog.md'
docs/diagrams/current-system.md
docs/SCREEN_DESIGN.md
docs/function-design/42-cmd-sales-stocktake.md
docs/function-design/55-ui-csv-import.md
docs/function-design/35-biz-stocktake-service.md
docs/function-design/41-cmd-pos.md
docs/function-design/32-biz-csv-import-service.md
docs/function-design/73-ui-stocktake.md
exit=0
$ rg -l 'WRITES_SUSPENDED|IMPORT_SUSPENDED|BIZ0[36]_STOP|writes_?[sS]uspended|legacy_(start_stocktake|update_count|complete_stocktake|commit_csv_import|rollback_csv_import|stop_tests)|apply_void_stock_corrections|StockCorrection|get_stocktake_items_for_complete|StocktakeItemForComplete|time_evidence|LEGACY_ITEM_KINDS|seed_legacy_fixture|stocktake_time_model|UpdateCount(Request|Result)|useUpdateCount|AdjustedItem|SameDateCsvImportSummary' docs --glob '!docs/archive/**' --glob '!docs/plans/**' --glob '!docs/research/**' --glob '!docs/adr/2026-09-18-stocktake-time-evidence.md' --glob '!docs/adr/2026-09-23-legacy-stocktake-z004-write-stop.md' --glob '!docs/decision-log.md' --glob '!docs/backlog.md'
docs/diagrams/cross-feature-verification.md
docs/architecture/cmd-task-specs.md
docs/FUNCTION_DESIGN.md
docs/function-design/41-cmd-pos.md
docs/function-design/20-io-product-repo.md
docs/function-design/24-io-csv-import-repo.md
docs/function-design/35-biz-stocktake-service.md
docs/function-design/10-common-rules.md
docs/function-design/42-cmd-sales-stocktake.md
docs/function-design/73-ui-stocktake.md
docs/function-design/22-mnt-migration.md
docs/function-design/32-biz-csv-import-service.md
docs/function-design/90-traceability.md
exit=0
$ rg -l 'apply_void_stock_corrections|StockCorrection' src src-tauri/src --glob '!src/lib/bindings.ts' | wc -l
3
exit=0
$ rg -c '#\[ignore' src-tauri/src/io/l3_fixtures.rs
rg: src-tauri/src/io/l3_fixtures.rs: IO error for operation on src-tauri/src/io/l3_fixtures.rs: No such file or directory (os error 2)
exit=2
$ rg -c 'INVENTORY_L3_DAY' tests/fixtures/z004/README.md
exit=1
```

完了時の期待: AC2 の 1 つ目が exit 1、2 つ目と 3 つ目が 1 以上と 1（3 つ目は comment の行を除いた数）。AC3 の数が 4、`reference_type = 'stocktake'` が exit 1。AC4 の `#[ignore]` が `src-tauri/src/biz` で exit 1（`ej_parser.rs` の 1 は本 lane の外で変えない。`io/l3_fixtures.rs` の 1 は AC12 の手元の test）。AC6 の数が全部 0（補償の型の残りの 3 を含む）と `absent`。AC7 の数が 1 以上・7・8。AC11 の superseded が 1、`一時停止中` と撤去した名前の検索が exit 1。AC12 の `#[ignore]` が 1、README の `INVENTORY_L3_DAY` が 1 以上。test の本数は AC にしない（PR と runner の出力を参照）。

## Design Readiness

設計の完了条件は `docs/DEV_WORKFLOW.md` Design Phase Rules の Design completion criteria。

- 引用する設計正本（節まで）: ADR SPEC-STK-CNT-D1〜D12（とくに D3 の判定表の「精算 k」の行・理由 1・A1、D4、D5、D6、D7、D8、D9、D10、D12 の「(b) を 2 つの lane に分けた」）、停止 ADR SPEC-STOP-D1〜D6、[tracking](../../db-design/tracking-system-tables.md)・[POS](../../db-design/pos-tables.md)・[master](../../db-design/master-tables.md)・[transaction](../../db-design/transaction-tables.md) の「計数の置き場と在庫の求め直しの契約」、[21](../../function-design/21-io-inventory-repo.md)・[24](../../function-design/24-io-csv-import-repo.md)・[30](../../function-design/30-biz-product-service.md)・[31](../../function-design/31-biz-inventory-service.md)・[32](../../function-design/32-biz-csv-import-service.md)・[35](../../function-design/35-biz-stocktake-service.md)（D-118 の節と §20.5a）・[36](../../function-design/36-biz-integrity-check.md)・[40](../../function-design/40-cmd-product.md)・[41](../../function-design/41-cmd-pos.md)・[42](../../function-design/42-cmd-sales-stocktake.md)・[51](../../function-design/51-ui-product-form.md)・[55](../../function-design/55-ui-csv-import.md)・[60](../../function-design/60-ui-product-import.md)・[65](../../function-design/65-inventory-record-traceability.md)・[73](../../function-design/73-ui-stocktake.md)・[75](../../function-design/75-ui-integrity-check.md) の D-118 の節、[UI_TECH_STACK](../../UI_TECH_STACK.md) の D-118 の節、[22 §16・§19・§20](../../function-design/22-mnt-migration.md)、[共通規則](../../function-design/10-common-rules.md) SPEC-UNIT-D3・D4・D6・D8・D11 と「入力の上限と安全な整数の範囲」の (b)(c)、D-061、D-118、D-123。
- 必要な設計成果物（状態）: (1) (b) を 2 つの lane に分けた境（ADR D12）= updated in this PR。(2) 初期在庫の見分け方と demo の seed（ADR D1・D9、tracking、21）= updated in this PR。(3) 共有 JAN の移行の前からの違反の知らせ方（ADR D10、36、75）= updated in this PR。(4) migration vZ の手順（22 §20）= updated in this PR。(5) Z004 の preview と確定の引き当ての食い違い・整数の数量の換算・精算回数の範囲・知らせの wire（32）= updated in this PR。(6) preview token を消す条件（41）= updated in this PR。(7) 入出庫の記録の一覧の棚卸しの件数と記録の詳細の節（65）= updated in this PR。(8) wire の DTO の field の名前と画面の語の細部（下の Boundary と S8。42・73・55・65 の契約の表は既にある）= 本 packet で決め、実装と同じ PR で正本へ（S9）。
- plan にしかない durable な判断の昇格先: 上の (1)〜(7) は正本に置いた。packet にだけあるのは、規模と手法、S1〜S9 の順、所有表、既存の test の移し方（Matrix の T1〜T12）、横断の約束の表（Z1〜Z20）、DTO の field の名前（(8)。実装の PR で 42 へ）、L3 の台本。
- 前提・制約と、延期した design gap の follow-up: b2 の範囲（下の「後続の lane」と ADR D12）、P の小数（backlog）、D1 の見た目（owner の判断事項 (i)）、振替（(ii)）、`register_processed`（(iii)）、TD-333。どれも b1 の書く経路の正しさに効かない（b2 は表示、P は今の行の error で黙って入らない、ほかは b1 が触らない経路）。
- 絶対保証（cannot happen / always happens）の例外と escape hatch の自己点検: 「`stock_quantity` を書くのは `recompute_stock` だけ」は (a) のまま（vZ は写しを書かない。時点証拠の `update_stock_quantity_with_revision` は撤去で無くなり、例外が 1 つ減る）。「5 つの入口を一度に外す」の escape hatch は無い（S4・S5 で停止の本体を消し、AC6 の `一時停止中` の 0 で確かめる）。「Z004 の movement は `phase = 1`」は vZ の CHECK と `insert_movement` の `settlement_no` から決める `phase` の両方で守る。「計数は消さない・書き換えない」は `stock_counts` の UPDATE・DELETE の関数を持たないこと（`rg -n -i 'stock_counts' src-tauri/src --glob '!*test*' | rg -i 'update|delete'` が 0 行を Final Review で確かめる。例外は開発の道具の demo の seed の `--reset`〈`src-tauri/src/seed_demo.rs` の `delete_all` が表ごと消す。店の画面からは呼べず、行は `"stock_counts",` の名前だけなので上の `rg` に当たらない。round 2 の是正の S9〉）。「完了は在庫を動かさない」は完了の TX が movement を書かず `recompute_stock` を呼ばないこと（K1）。例外: 完了の時点で廃番かつ写し 0 の明細は未計数でも完了を止めない（35・ADR D6 の明記の例外）。
- 判定: ready。b1 の Scope の全ての振舞いが正本か本 packet の Boundary・S8 から決まる。owner の判断事項 (i) が A なら design へ戻す。

## Registration / Generation Obligations

| 変更対象 | 登録・生成義務 |
|---|---|
| Tauri command（frontend から呼ぶ） | 新設 4 を `src-tauri/src/lib.rs` の `collect_commands` の一覧と、同じ file の test の一覧に足し、`update_count` を両方から外す。新しい command 関数に `#[tauri::command]` と `#[specta::specta]` の対。`cd src-tauri && cargo run --bin generate_bindings` で `src/lib/bindings.ts` を再生成（S7） |
| function-design doc 新設 | 該当なし（新しい doc は無い）。L3 の fixture の生成の `io::l3_fixtures` は `#[cfg(test)]` の module で、`pub fn` を持たないので対応表に足さない（round 1 の是正）。`stocktake_service` を module に分けるなら `src-tauri/tests/design_compliance_test.rs` の `build_doc_to_modules_map()` の 35 の行に新しい module を足す。新しい関数は `pub` なら 35・32・30・36 の code block にシグネチャを置く（`pub(crate)` は検査の対象外。(a) と同じく正本の code block に置く） |
| source / workflow doc 新設・改名・削除 | 停止 ADR を superseded にする（`docs/adr/README.md` の一覧の状態）。新設・改名は無い |
| REQ / coverage の追加・変更・削除 | test 名・comment に REQ-205・REQ-206・REQ-207・REQ-401・REQ-101・REQ-102・REQ-104・REQ-904・REQ-903 を付ける。撤去する test の REQ の参照が減るので `cd src-tauri && cargo run --bin generate_traceability` で `docs/function-design/90-traceability.md` を再生成（S9） |
| DB/IO の re-export の allowlist（D-083。`src-tauri/tests/architecture_test.rs` の `biz_mnt_direct_db_io_reexport_allowlist`） | S4 で `StocktakeItemDetail` を BIZ の型にした（`src-tauri/src/biz/mod.rs` は `stocktake_service::StocktakeItemDetail` を re-export し、`crate::db::stocktake_repo::StocktakeItemDetail` を re-export しない）ので、一覧から `("biz/mod.rs", "StocktakeItemDetail", "crate::db::stocktake_repo::StocktakeItemDetail")` を外す（機械的な追従。Codex の closure の Gated Amendment で足した行） |
| route 新設・改名・削除 | 該当なし（`/stocktake/` の search の field を足すだけ。`npm run generate:routes` で routeTree が変わらないことを確かめる） |
| operator 画面新設・改名・削除 | 該当なし（棚卸し・Z004 タブ・商品フォーム・棚卸しの記録の詳細・整合性の中身だけ。到達の導線は今のまま。「今の数を入れる」は既存の `/stocktake/` へ search で入る） |

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | Which concepts belong to replaceable external adapters, and which concepts are stable app-core contracts? | Architecture, function design, decision-log, Plan Packet | 当たる。IO-02（CASIO の Z004 の形・マシンNo.・精算回数の文字列）は adapter、精算の並び・窓・要再確認は app-core（ADR D1・D3・D4）。数量の換算（レジの数量 1 = 1 m）は BIZ（SPEC-UNIT-D4・D8） | 23・32 |
| Fact check / design decision split | Which claims are observed facts from hardware/tool/files, and which are app decisions that need source-doc promotion? | Investigation doc, source design docs, decision-log | 当たる。精算回数の単調・欠けの観測は backlog の「実測とPOS系列の対応の入力 1〜5」（観測）、窓の原理と A1 は owner の決定（D-118）。SQLite の CHECK の振舞いは Contract Probe P1〜P3 で観測した。b1 の分け方・初期在庫の見分け方ほかは起草役の判断で正本へ置いた | ADR D9・D10・D12、22 §20、32、41、36、75、65 |
| Lifecycle / retry | What happens before, during, after, and after failure for import/export, duplicate input, rollback, retry, cancellation, and re-run? | Function design, DB design, UI design, Test Matrix | 当たる。Z004 の preview → 確定（TX で取り直し、拒否で何も書かない、token を残す）→ 取消（無効にして求め直す）→ 取り込み直し（取り消した精算と同じ番号を新しい取込みで）。計数は消さず入れ直しで直す。完了の拒否は何も書かない。vZ は 1 TX | Matrix の State Lifecycle |
| Operator workflow | What does the operator do in the real sequence across app, external tool, media, print/export, backup, and recovery? | Screen/UI design, function design, Plan Packet manual checks | 当たる。締め → SD → 取込み → 要再確認の数個を棚で見て一覧から入れ直す → 印刷（ADR の Consequences）。大晦日は締め → 取込み → 入れ直し → 完了（締めが翌日なら完了も翌日）。完了の確認の文で最後の精算の取込みを問う（H2） | L3 の台本 |
| Replacement path | If the external system changes, which files/modules/docs are replaced and which app-core contracts remain stable? | Architecture, function design, decision-log | 当たる。レジが替わると IO-02 と精算の識別メタの読み方を替え、精算の並びの軸（精算日・精算回数）と窓は残る（ADR の Revisit: 精算回数の reset・レジの入替） | ADR の Revisit |
| Data safety / evidence | How can the claim be supported by anonymized shape/count/hash/procedure evidence without committing real store data? | Plan Packet Data Safety, investigation doc, review evidence | 当たる。tracked の Z004 と商品の fixture は合成の値だけ（TD-296）。L3 は Coordinator が置いた合成の DB（L3-1）と差し替えた空の DB（L3-2 以降）で行い、owner の今の DB は消さずに移して残す（下の Data Safety。round 2 の是正） | Data Safety |
| Reporting / accounting semantics | Are totals, summaries, item records, returns, corrections, and inventory movements modeled separately enough to avoid false business meaning? | DB design, function design, report design, Test Matrix | 当たる。売上（`sale_records`）と在庫（movement）を分け、共有 JAN の行は売上だけ・在庫連動の外（D4）。完了は在庫を動かさず評価額を固定し、完了の後の差は参考（非遡及）。長さの商品の売上の数量を cm にそろえる（日次・月次の `length_cm` と同じ） | Z8・Z10・K1・R2 |
| Manual verification | Which assertions cannot be proven by automated tests and require Windows native L3, external tool import, or real-device confirmation? | Plan Packet, Test Matrix, PR body | 当たる。Windows の実 DB での vZ、HID・IME の連続入力、native の file dialog での CP932 の Z004 の取込み、通しの操作列（下の L3 の台本）。日付の変わり目・移行の前からの共有 JAN の組は L3 Eligibility の (3) に当たるので自動 test と画面の写し | Test Plan の L3 |
| 環境・再現性 | 新設の環境依存（toolchain / CI runner / OS 差異等）を repo-pinned config で強制するか、明示的に defer するか | repo-pinned config, Plan Packet | 当たる。列の追加の CHECK が既存の行に当たる振舞いは SQLite 3.37.0 以降（同梱は `libsqlite3-sys` 0.28.0 の 3.45.0。`src-tauri/Cargo.toml` の `rusqlite = { version = "0.31", features = ["bundled"] }` で固定）。M5 が CHECK の効きを確かめるので、同梱の SQLite を下げれば red になる | M5 |

## Boundary / Wire Contract

- producer: BIZ-06（`save_stock_count`・`preview_stock_count_placement`・`zero_uncounted_items`・`list_recheck_products`・`start_stocktake`・`complete_stocktake`・`get_stocktake_items`・`find_stocktake_item`・`get_stocktake_record`）、BIZ-03（`parse_and_validate`・`commit_csv_import`・`rollback_csv_import`）、BIZ-01（拒否と回復先）、BIZ-07（`run_integrity_check`）、CMD の `CmdError`、migration vZ と DB の列。
- consumer: 棚卸しの画面、Z004 タブ、商品フォーム、棚卸しの記録の詳細、整合性の画面、`describeError`、BIZ の求め直し（DB の列）。
- wire type（新しい型。field の名前は本 packet で決め、実装の PR で 42・40・41 へ写す）:
  - `SaveStockCountRequest { product_code: string, count_value: number, fresh: boolean, business_date: string, stocktake_item_id: number | null }`（42 の表）。
  - `StockCountKind = "default" | "fresh"`、`RecountReason = "undetermined_settlement" | "same_date_movement" | "conflicting_counts"`、`CountRecoveryAction = "active_count" | "count_now"`、`CompletionBlockReason = "recheck" | "uncounted" | "settlement_pending"`、`StocktakeRecoveryCode = "recount_required" | "completion_blocked"`。
  - `StockCountSaveResult { id: number, count_kind: StockCountKind, stock: number, stock_range_low: number | null, stock_range_high: number | null, recount_reasons: RecountReason[], later_count_exists: boolean }`、`CountPlacementPreview { later_count_exists: boolean, product_name: string, stock_unit: ProductStockUnit, stock: number, stock_range_low: number | null, stock_range_high: number | null, recount_reasons: RecountReason[], count_value: number | null, count_kind: StockCountKind | null, count_business_date: string | null }`（計数の欄の商品の文脈。42・35。round 1 の是正）。
  - `CountRecoveryTarget { product_code: string, product_name: string, stock_unit: ProductStockUnit, stock: number, stock_range_low: number | null, stock_range_high: number | null, stocktake_item_id: number | null, stocktake_id: number | null, action: CountRecoveryAction, block_reason: CompletionBlockReason | null, recount_reasons: RecountReason[] }`（`stock_unit`・`stock`・範囲は round 1 の是正。範囲は両方 null か両方 number）、`StocktakeRecovery { code: StocktakeRecoveryCode, targets: CountRecoveryTarget[], settlement_pending: SettlementPendingHint | null }`、`SettlementPendingHint { missing: SettlementGap[], on_or_after: string | null, before: string | null }`（40 の新契約。`settlement_pending` は `completion_blocked` で `settlement_pending` の target があるときだけ）。
  - `CmdErrorKind` に `"stocktake_guard"`、`CmdError` に `stocktake_recovery: StocktakeRecovery | null`（既存の kind では null）。
  - `SettlementGap { from: number, to: number }`。`PreviewData` と `ImportResult` に `settlement_gaps: SettlementGap[]` と `settlement_date_after_today: boolean`（知らせ）。`ImportResult`・`RollbackResult` に `recheck_targets: CountRecoveryTarget[]`。`PreviewData.duplicate_check` は外す（`DuplicateCheck`・`DuplicateStatus`・`SameDateCsvImportSummary` ごと。上の「この変更で不要になるもの」）。
  - `SharedJanConflict { jan_code: string, product_codes: string[] }`、`IntegrityResult.shared_jan_conflicts: SharedJanConflict[]`。
  - 棚卸しの明細 `StocktakeItemDetail`: 今の field（`id`・`stocktake_id`・`product_code`・`name`・`department_name`・`system_stock`・`actual_count`・`counted_at`・`current_stock`・`is_discontinued`）に `stock_unit: ProductStockUnit`、`count_value: number | null`・`count_kind: StockCountKind | null`・`count_business_date: string | null`（効く計数。衝突のときは写しの土台の計数）、`stock_range_low: number | null`・`stock_range_high: number | null`、`recount_reasons: RecountReason[]`、`difference: number | null`（35 の新契約の差異。計数が無ければ null）を足す（`current_stock` は写しの値）。
  - `StartStocktakeResult { stocktake_id: number, item_count: number }`（`auto_filled_count` を外す）、`StocktakeResult { total_cost: number, total_items: number, integrity_result: IntegrityResult | null }`（`adjusted_items` を外す）。
  - 棚卸しの記録の詳細 `StocktakeRecordDetail { id, started_at, completed_at, status, total_cost, item_count, items: StocktakeRecordDetailItem[], after_completion: AfterCompletionDiff | null }`（`corrected_count`・`movements` を外す）。`StocktakeRecordDetailItem { product_code, product_name, department_name, stock_unit, count_value: number | null, count_kind: StockCountKind | null, count_business_date: string | null, difference: number | null, valuation_quantity: number | null, valuation_cost_price_centi: number | null, loss_cost_centi: string | null }`（完了した棚卸しはカーソルの時点の効く計数、進行中は今の効く計数。`loss_cost_centi` は差異が負で評価原価があるときの 1/100 円の十進の文字列）。`AfterCompletionDiff { confirmed: AfterCompletionDiffItem[], recheck: AfterCompletionDiffItem[], confirmed_total_yen: string }`、`AfterCompletionDiffItem { product_code, product_name, stock_unit, valuation_quantity: number, stock_at_completion: number, stock_at_completion_low: number | null, stock_at_completion_high: number | null, valuation_diff_yen: string, valuation_diff_low_yen: string | null, valuation_diff_high_yen: string | null }`（完了していなければ `after_completion` は null）。
  - 外す型と field: `UpdateCountResult`、`AdjustedItem`、`StockCorrection`、`SameDateCsvImportSummary`、`complete_stocktake` の `force_fill`、`commit_csv_import` の `additional_import_confirmed`。
- internal type: `BizError::StocktakeGuard { message: String, recovery: StocktakeRecovery }`。`RecheckReason` に `UndeterminedSettlement`（wire の `RecountReason` と同じ順）。preview の cache は 32 の「preview の cache の契約」の形（round 1 の是正）: `MatchedRow { line_no, normalized_jan, product_code, shared_jan: bool, quantity: i64, amount }`（`quantity` は在庫の数量に換えた後の値。今の i32 の IO-02 の数量を i64 で × 100 してから上限 `9999999` と比べるので、換算で i32 を溢れさせない。`pos_stock_sync` は持たない）、`CachedPreview { created_at, matched_rows, error_rows, preview_data, settlement: { machine_no, settlement_no, settlement_date } }`（`active_same_date_import_ids` を外す）、`CommitRequest { cached_data, today }`（`additional_import_confirmed` を外す）。どれも wire に出ない。`NewSaleRecord` に `stock_linked: bool`。`apply_stock_change` から `settlement_no` の引数を外す（31）。
- precision/range: 数の wire は全部 i64 で、計数・在庫・範囲・評価数量は絶対値 `999999999` 以内、精算回数と区間の端は `9007199254740991` 以内、評価額の総額は `check_safe_integer` の後なので JavaScript の安全な整数に入る。範囲の決まらない読取りの金額（行のロス原価、完了の後の評価額の差）は十進の文字列（SPEC-UNIT-D6、横断の約束の Z13）。
- round-trip path: DB → BIZ → specta → bindings → 画面。`SaveStockCountRequest` は画面 → CMD → BIZ（`business_date` は画面の日付の欄の ISO の形、`count_value` は m の入力を cm に直した整数）。
- invalid input: 計数の値・日付・明細の不一致は `validation`（field 付きは `ValidationFailedAt`）。Z004 の拒否は `import_error` と BIZ の文。完了の拒否と在庫連動の拒否は `stocktake_guard` と回復先。画面は kind・code・action・block_reason を生成型で網羅し、未知の code を「そのまま続行」にしない（40）。商品一括 import の確定の TX で行が共有 JAN か在庫連動を始める検査に当たれば `validation`（field なし。最初に当たった行を名指す message）で確定全体を戻し、`ImportResult`（TS 名 `ProductImportResult`）の形と結果の画面は変えない（30 の D-118 の節、42 §22.6。round 2 の是正）。
- compatibility: 記録済みの操作ログ（`stocktake_complete` の旧い detail ほか）は書き換えず、読み手は今のまま。新しい行の detail の key は 35 の「操作ログ」（`stocktake_start` は `stocktake_id`・`item_count`、`stocktake_complete` は `stocktake_id`・`total_cost`・`total_items`・`difference_count`。[74](../../function-design/74-ui-operation-logs.md) は operation_type の label だけを読む。round 2 の是正）。command の名前は `update_count` を外す以外は変えない。`CmdErrorKind` は 13 値になる（D-061 の pattern で値を足す。停止 ADR の「12 値のまま」は停止の撤去と一緒に外れる）。`CsvImport`（一覧の行）・`CsvImportRecordDetail`・`Stocktake`・`StocktakeProgress`・`LastStocktakeSummary`・`MovementRecord` は形を変えない（新しい列を wire に出さない。ホーム・取込みの記録の詳細ほかの画面と TS の mock を変えない。`MovementRecord` は b2）。

## Test Plan

Test Design Matrix: [2026-10-22-stocktake-count-runtime-b1](test-matrices/2026-10-22-stocktake-count-runtime-b1.md)。

- If the Human Gate includes L3, Writer completion includes `cargo check --release` before the owner native build; this is not a CI gate.
- targeted tests: `cd src-tauri && cargo test --lib db::` / `cargo test --lib biz::stock_ledger` / `cargo test --lib biz::stocktake_service` / `cargo test --lib biz::csv_import_service` / `cargo test --lib biz::product_service` / `cargo test --lib biz::integrity_service` / `cargo test --lib cmd::` / `cargo test --lib finite_enum_contract_tests` / `cargo test --lib io::l3_fixtures` / `cargo test --test import_internal_contract_test` / `cargo test --test seed_test`、`npm test -- src/features/stocktake src/features/csv-import src/features/products src/features/inventory-records src/features/integrity-check src/features/receiving src/features/disposal src/features/manual-sale src/features/return-exchange src/lib`。
- negative tests: Matrix の M3〜M5・M8・M9（vZ が止まる・CHECK が拒む・作り直しの後の失敗が戻る）、C3〜C5（計数の拒否）、K2〜K5（完了の拒否と範囲）、Z1〜Z5・Z12・Z13・Z15・Z21・Z22（Z004 の拒否と確定の照合）、J1〜J5（共有 JAN と在庫連動の拒否）。
- compatibility checks: 既存の test のうち契約が残るものを新しい経路で期待を変えずに通す（AC10）。記録済みの操作ログの読み手は今のまま。
- data safety checks: fixture は全部合成（Rust と TS の test、L3 の Z004 と商品の CSV）。
- main wiring/integration checks: X1・X2（STK-1・STK-2 の診断を新しい経路で PASS）、X3（cross feature の model）、Z8・Z14（確定と取消の後の写しと `derive_stock` の一致）、W2（command の登録）、U15（再取得）。

L3（Human Gate `manual`。owner は 2026-10-21 から数日、手が使いにくい〈左手だけ〉。1 回の通しの台本で、各段の PASS / FAIL を返してもらう。起動は今の L3 と同じく Windows native の `npm run tauri dev`）。round 1 の是正で、台本を状態遷移の表にした（前の段の後の状態が次の段の前の状態で、期待は 1 通りに決まる）。L3 の日を T と書く（PC の今日）。入力物は S9 の生成の test が T から作る合成の file（**Coordinator** が L3 を頼む日に作り、Windows の path を省略せずに渡す）と、tracked の従来の形の `tests/fixtures/z004/normal-small.csv`。L3 は既知の合成の状態の DB で始める（round 2 の是正。owner の今の DB にほかの商品があると、その商品に当日付の入出庫があれば「残りをまとめて 0 にする」の後も要再確認で完了が止まり、L3-11 の期待に届かない）。DB は下の「L3 の DB の用意と差し替え」の 1 通りで、Coordinator が用意し差し替える（owner は起動と終了だけ）。

L3 の DB の用意と差し替え（round 2 の是正。先例は (a)〈PR #176〉と原価の lane（PR #184）の L3-1〈`docs/archive/plans/2026-10-18-stocktake-count-runtime-a.md`・`docs/archive/plans/2026-10-21-cost-lane-c.md` の Test Plan の L3〉と [22 §16](../../function-design/22-mnt-migration.md) の作り直しの手順。どちらも Coordinator が写しを読み取りだけで確かめ、owner の指示の範囲で WSL の `/mnt/c/Users/...` から folder を消さずに移した）:

1. 用意（Coordinator、L3 を頼む前）: (a) b1 を merge する前の main の commit（vZ の直前の版まで当たる。起票の時点では vB = 10）の build で `cd src-tauri && CARGO_TARGET_DIR=/tmp/shared-cargo-target cargo run --bin seed_demo_data -- --db {repo の外の一時の folder}/inventory.db` を走らせ、vZ の直前の版の DB（demo の seed の商品・初期在庫・入庫・売上〈`sale_auto`、`reference_type` NULL〉の movement）を作る。その DB を読み取りだけで開き、`schema_versions` の最大の `version` が vZ の直前の版、`stocktakes`・`csv_imports` が 0 行、`products`・`inventory_movements` が 1 行以上であることを確かめ、`SELECT product_code, stock_quantity FROM products ORDER BY product_code` の結果を控える（owner に画面で控えてもらう段の代わり）。(b) owner がアプリを止めた状態で、owner の今の L3 用の data の folder を 22 §16 の作り直しの手順 2〜4 で日付の付いた folder（例 `%USERPROFILE%\inventory-db-moved\{L3 の日}-b1-before\`。前の lane の folder は上書きしない）へ消さずに移す（`%APPDATA%\com.inventory-system.desktop` と `%LOCALAPPDATA%\com.inventory-system.desktop` を丸ごと、起動の作業フォルダの旧 DB 一式があればそれも。backup の保存先の file は動かさない: 置く DB は既定の設定で、その保存先を読まない）。(c) `%APPDATA%\com.inventory-system.desktop\` を作り、(a) の DB を `inventory.db` の名前で置く。
2. L3-1（owner）: 新しい版を起動し、起動できたら閉じる。Coordinator はアプリが止まった後に DB の写しを読み取りだけで開き、版が vZ、`stocktakes`・`csv_imports` が 0 行、1. (a) で控えた在庫と全商品で同じことを確かめる。
3. 差し替え（Coordinator、L3-1 の後）: 1. (b) と同じ手順で、data の folder を `…\{L3 の日}-b1-l3-1\` へ消さずに移す。次の起動で空の DB が作られて最新の版まで当たり（22 §16 の手順 5）、初期の部門（`src-tauri/src/db/schema_v1.rs` の `departments` の初期データ）だけがあり、商品と記録は無い。L3-2 以降はこの状態から始まる。
4. 終わった後は戻さない: L3 の後の DB は合成の値だけの開発・試験 DB で、次の L3（b2）の起点にも使える。移した 2 つの folder は残し、消すかは owner の判断（先例どおり）。L3 を 2 round 目にするときは、3. と同じ手順でもう一度空の DB にして L3-2 から始める（L3-1 は 1 round 目の結果を引き継ぐ）。

L3 の fixture の表（生成の test〈S9〉と `tests/fixtures/z004/README.md` はこの表を写す。JAN は合成の 13 桁で check digit が合うものを生成の関数が決め、README に書く。売価は 1 個・1 m あたり）:

| 商品 | 単位 | 在庫連動 | 初期在庫 | 売価 | 原価 | 役割 |
|---|---|---|---|---|---|---|
| P1 | 個 | する | 10 | 1000 | 500.00 | 前の日の分の計数、決まらない精算、共有 JAN の拒否（L3-12） |
| P2 | 個 | する | 10 | 1000 | 500.00 | 打ち直し、完了の後の差、共有 JAN の相手 |
| P3 | 個 | する | 10 | 1000 | 500.00 | 計数の前の売上（101・103。どちらも計数に入る） |
| P4 | m | する | 5 m | 300 | 150.00 | m の入力 |
| P5 | 個 | しない | 10 | 1000 | 500.00 | 連動なしの売上（101）と在庫連動を始める拒否 |
| P6 | 個 | する | 10 | 1000 | 500.00 | 数えない（「まだ数えていない」と「0 にする」） |

| file | 精算回数 | 精算日 | 行（商品 × 数量、金額） | 使う段 |
|---|---|---|---|---|
| `products-new.csv`（商品一括 import、CP932） | — | — | P1〜P6 の新規の行 | L3-2 |
| `z004-0101.csv` | 101 | T−3 | P3 × 1（1000）、P5 × 1（1000） | L3-3 |
| `z004-0102.csv` | 102 | T−2 | P1 × 1（1000） | L3-8 |
| `z004-0102-other.csv` | 102 | T−2 | P1 × 2（2000） | L3-10（順 4 の拒否） |
| `z004-0099.csv` | 99 | T | P2 × 1（1000） | L3-10（順 5 の拒否） |
| `z004-0104.csv` | 104 | T | 正常な行が全て 0 | L3-10（欠けの知らせ、取消と取り込み直し） |
| `z004-0200.csv` | 200 | T+365 | P2 × 1（1000） | L3-10（欠けと未来の日付の知らせ。取り込まない） |
| `z004-0103.csv` | 103 | T−1 | P3 × 1（1000） | L3-11 |
| `products-jan-conflict.csv`（商品一括 import、CP932） | — | — | P1 の上書きの行（JAN を P2 の JAN に） | L3-12 |

| # | 前の状態 | 操作 | 入力物 | 観測可能な合格基準 | 後の状態 |
|---|---|---|---|---|---|
| L3-1 | 上の「L3 の DB の用意と差し替え」の 1. で置いた vZ の直前の版の DB（demo の seed の商品と入出庫の記録。棚卸しと Z004 の記録は無い） | 新しい版を起動し、起動できたら閉じる | Coordinator が置いた DB | 起動できる（止まらない）。閉じた後に Coordinator が写しで、版が vZ で、全商品の在庫が置く前に控えた値と同じことを確かめる。止まった場合は「データベースの準備に失敗」の文の写しを返してもらい、Coordinator が 22 §20 の手順 1・6 の文と突き合わせる | 版 vZ。Coordinator が上の 3. で空の DB へ差し替える |
| L3-2 | 差し替えた空の DB（版 vZ、初期の部門だけ、商品と記録は無い） | 起動する → 商品一括 import で `products-new.csv` を取り込む | `products-new.csv` | P1〜P6 が登録され、在庫照会で P1・P2・P3・P5・P6 が 10、P4 が 5 m | 棚卸しは無い |
| L3-3 | 棚卸しは無い、計数は無い | 売上データ取込みの Z004 タブで `z004-0101.csv` を取り込む | `z004-0101.csv` | preview に拒否も知らせも無く取り込め、結果に要再確認の一覧が出ない。在庫照会で P3 が 9、P5 が 10（連動しないので減らない） | 取り込み済みの精算 101 |
| L3-4 | 同じ。進行中の棚卸しは無い | 商品フォームで P5 の在庫連動を付けて保存 → 拒否の文の「今の数を入れる」を押す → 「編集内容が保存されていません」の確認で「破棄して移動」→ 棚卸しの画面で P5 の数 9 を保存 → 戻った P5 の商品フォームで在庫連動を付け直して保存（round 3 の是正） | 画面の入力 | 1 回目の保存で「連動しない間の売上があります。今の数を入れてから在庫連動にしてください」と「今の数を入れる」の導線が出て、在庫連動の checkbox は付いたまま。導線を押すと離脱の確認が出る。「破棄して移動」で棚卸しの画面に P5 の名前・単位・在庫 10 が出て（計数の欄の文脈）、保存の後に P5 の商品フォームへ戻り、在庫連動は付いていない（DB の値。未保存の変更は破棄した）。付け直して保存すると成功して商品一覧へ戻り、在庫連動が付く | P5 は連動し、fresh な計数 9（今日、精算 101 まで） |
| L3-5 | 同じ | 棚卸しの画面で「棚卸しを開始する」 | — | 進行中の棚卸しができ、P1〜P6 の 6 商品（空の DB から始めたので他の商品は無い）が未計数の明細に出る（自動の 0 は無い。P5 の L3-4 の計数は明細を参照しないので未計数） | 進行中の棚卸し |
| L3-6 | 同じ | (a) 検索欄に P2・P3・P4・P5 を順に入れ、数を入れて Enter（マウスを使わない）: P2 10、P3 9、P4 `4.5`（m）、P5 9 | 画面の入力 | 4 商品とも保存され、保存の日付が今日で画面に出ている。一覧の差異は P2 0、P3 0、P4 −0.5 m、P5 −1 | P2・P3・P4・P5 の既定の計数（日付 T） |
| L3-7 | 同じ | (b) P1 を「前の日の分を入れる」で日付 T−2、数 10 で保存 → 続けて P1 を日付 T−3、数 11 で保存しようとする | 画面の入力 | 2 回目の保存の前に「後の数が既にあり、そちらが効きます」が出て、保存はできる。一覧の P1 の数は 10（日付 T−2） | P1 の効く計数は 10（T−2） |
| L3-8 | 同じ | (c) Z004 タブで `z004-0102.csv` を取り込む → 結果の要再確認の一覧を見る → 棚卸しの画面の「数え直しが必要な商品」の P1 の「入れ直す」で 9 を保存 | `z004-0102.csv` | 取込みの結果に P1 と範囲「9〜10」と「数えた日に売り買い（入出庫）がありました…」の文。棚卸しの画面の上部の一覧にも P1 と範囲。入れ直すと一覧から消え、P1 の在庫が 9 | 取り込み済み 101・102。P1 は fresh な計数 9 |
| L3-9 | 同じ | (c2) P2 の「入れ直す」で 8 を保存（打ち直し） | 画面の入力 | P2 の数 8、差異 −2 | P2 は fresh な計数 8 |
| L3-10 | 同じ | Z004 の preview を順に: `normal-small.csv`、`z004-0102-other.csv`、`z004-0099.csv`（3 つとも取り込まずに選び直す）→ `z004-0104.csv` を取り込む → 結果の画面の「取り消す」→ もう一度 `z004-0104.csv` を取り込む → `z004-0200.csv` を preview する（取り込まずに選び直す） | 各 file | 順に「精算回数の無い file は取り込めません…」「同じ精算と思われる別の資料が既に取り込まれています（精算 102）」「レジの日付か番号が前の精算と合いません…」。`z004-0104.csv` の preview に「精算 103 が取り込まれていません」、取り込めて結果に要再確認の一覧が出ない。「取り消す」で取消の結果が出る。取り消した後の `z004-0104.csv` は同じ知らせで取り込める（取り込み直し）。`z004-0200.csv` の preview に「精算 103 が取り込まれていません」・「精算 105〜199 が取り込まれていません」と「レジと PC の日付がずれている可能性があります」（103 はまだ取り込んでいない） | 取り込み済み 101・102・104（104 は 2 回目の取込み） |
| L3-11 | 同じ。P6 は未計数 | (d) 「棚卸しを確定する」→ 確認の文 → 完了できない理由の一覧 → 「残りをまとめて 0 にする」（確認。日付は今日）→ `z004-0103.csv` を取り込む → もう一度「棚卸しを確定する」 | `z004-0103.csv` | 確認の文に「今日の最後の精算（締め）を取り込みましたか…」。理由の一覧に「まだ数えていない」（P6）と「取り込んでいない精算がある」（P1〜P5）が文字で分かれて出て、後者に「精算 103 を取り込んでください」。0 にした後と 103 の取込みの後に完了でき、評価額 ¥18,175（P1 9・P2 8・P3 9・P5 9 は 500.00 円、P4 4.5 m は 150.00 円/m、P6 0）と明細の数 6 と「在庫は動きません」の旨が出る | 完了した棚卸し |
| L3-12 | 同じ | 商品一括 import で `products-jan-conflict.csv` を preview する（取り込まない） | `products-jan-conflict.csv` | P1 の行が error 行になり「同じ JAN の商品（P2 の商品コード）があるため、在庫連動にできません」が出る | 同じ |
| L3-13 | 同じ | 入庫で、日付 T で P2 を 1 つ入庫する → 入出庫の記録の一覧から棚卸しの記録の詳細を開く | 画面の入力 | 一覧の棚卸しの件数が明細の数 6（棚卸しの画面の明細の数と同じ）。詳細の明細の差異は P1 0、P2 −2（ロス原価 1,000.00 円）、P3 +1（103 が計数に入るため）、P4 −0.5 m（75.00 円）、P5 −1（500.00 円）、P6 −10（5,000.00 円）。「完了の後に入った記録による差（参考。評価額は変わりません）」の要再確認の差に P2 が範囲（完了の時点の在庫 8〜9）で出て、確定の差の一覧は空、確定の差の合計は ¥0。評価額は ¥18,175 のまま。「在庫変動」の節が無い | 同じ |

- L3 に入れないもの（L3 Eligibility の (3)、または Windows native でなくても見られるもの）: 73 の (e) 日付の変わり目（PC の時計を変える）、移行の前からある共有 JAN の組の整合性の画面の一覧（DB の手での改変が要る）、要再確認の理由 3（計数の衝突。同じ日付の別の値の計数を 2 つ入れれば作れるが、通しの台本を長くしないために自動 test と画面の写しで見る）。自動 test（U5・U14・C10）に加え、Coordinator が Ready の依頼と同時に、frontend の test の render（mock の command）か Vite の dev の画面の写しを owner に渡し、見た目の確認（Human Visual Confirmation）を L3 と同じ 1 回の PASS / FAIL に含める。写しを作れなければ、その残りの危険を PR の本文に書き、受け入れを Ready の判断に含める。
- fixture: L3-1 は Coordinator が main の build の demo の seed で作った vZ の直前の版の DB、L3-2 以降は差し替えた空の DB（上の「L3 の DB の用意と差し替え」。round 2 の是正）。owner の今の DB は消さずに移して残す（Data Safety）。
- 商品一括 import の確定の TX の拒否（30 の D-118 の節。round 2 の是正）は L3 に入れない: preview の後の DB の変化か、上書きを選んだ行どうしの JAN の重なりを作る入力が要り、画面は今の commit の失敗の Alert と「戻る」の経路そのままなので、Matrix の J3 の BIZ の test と画面の test で見る。

## Review Focus

- 「停止を外して黙ってずれない」の範囲が b1 に揃っているか（書く経路・守り・要再確認を見せる所）。b2 へ送った表示が b1 の安全に要らないか（ADR D12 の「(b) を 2 つの lane に分けた」と上の「各 lane の配布の単位としての安全」）。逆に、b1 に入れたが停止を外すのに要らないものは無いか。
- 精算の窓（ADR D3 の判定表の「精算 k」の行、A1、欠けがあるときの過剰側への倒れ方）と理由 1 の判定（取込みのときの `stock_linked`、個数 ≠ 0 か金額 ≠ 0）。求め直す商品の集合（計数を持つ全ての商品 + 取込みの商品。在庫連動の on / off を問わない）。
- 計数の保存の検査の順（形と暦 → 今日 → 値 → 明細 → INSERT → 求め直し）と、計数の前後が循環しない前提（既定の日付 ≤ 保存の日、fresh は今日）。
- 完了の拒否の 3 つの理由（とくに `settlement_pending` の (i)〜(iii)）と、評価額の総額の範囲の検査。記録の詳細の「完了の時点の在庫」の求め方（カーソル・業務日・精算日）。
- Z004 の確定の TX の取り直しの順、preview と確定の引き当ての食い違いの拒否、長さの商品の × 100、共有 JAN の行、正常な行が全て 0 の file。
- 撤去の全件（上の「この変更で不要になるもの」）と、既存の test の移し方（Matrix の T1〜T12。「契約ごと外す」の根拠と代わりの行）。**この変更で不要になる記述・条件・code が、ほかに残っていないか**（例: 停止を前提にした文、同じ日の追加の確認の Z004 の側、`phase = 1` で止まる (a) の分岐、時点証拠の語）。
- 初期在庫の見分け方の変更（ADR D9）が、商品の登録・一括 import の初期在庫と demo の seed の両方で正しいか。共有 JAN の違反を整合性の検査で並べる形（ADR D10）。
- round 1 の是正: 画面の約束に要る値が wire の型にあるか（`CountRecoveryTarget` の範囲と単位、`SettlementPendingHint`、`CountPlacementPreview` の文脈）、preview の cache が確定の照合に要る値（元の JAN・共有 JAN かどうか・精算の識別メタ）を持つか、L3 の台本の各段が前の段の後の状態から一意に決まるか、L3 と test が今のアプリで操作できる入口だけを使うか（JAN は一括 import の上書き、取消は結果の画面）。
- round 2 の是正: 商品一括 import の確定の TX の拒否の形（行を名指して確定全体を戻す。30 の D-118 の節）が、今の確定の再検証（BIZ-01-D8、D-122 の J3・J4・J7）と同じ形で、画面・wire を変えずに名指しと直し方に届くか。L3 の DB の用意と差し替え（Coordinator が置く合成の DB → 空の DB）で、台本の各段の前の状態と期待が 1 通りに決まるか。Matrix の T1〜T12 の「移す」に外す出力・引数の assert が残っていないか（「assert の単位で分けた test」）。「Would fail if」が kill を主張する mutant が全部観測の手段を持つか（観測できないものは Matrix の Residual Test Gaps の Final Review の code 読みの項目）。
- round 3 の是正: 取消・取り込み直しで照合や比較の根拠が消える・入れ替わる所（32 の順 2 は取り消した取込みとも比べる、fresh な計数の n は n の active な取込みがあるときだけ使う〈ADR D3。Codex の closure の Gated Amendment で取込みのカーソルの形に置き換えた〉）が、誤った版を取り消して正しい版を取り込む経路（D7）を壊さず、偽の拒否を作らないか。完了の後の比較の範囲（完了の時点にあった記録〈3 つのカーソル以下〉に、完了の後に入れた完了の前の入出庫・初期在庫・精算の記録を足す。計数はカーソル以下だけ。ADR D6・35。closure の是正）が、未来の日付の精算と入出庫で偽の差を出さないか。商品フォームの「今の数を入れる」が離脱の確認を通り、戻った後の操作が 1 通りに決まるか（51・73・L3-4）。
- closure の是正: 32 の順 2 で取り消した取込みとだけ合わないときの拒否の文が、原因の取込み（取込みID・精算回数・取込み日・レジ番号）を名指し、machine_no の違う active な取込みがあるときは今の文のままか（24 の照合先は active を優先する）。完了の後の比較の範囲が完了の後に保存した計数を入れないか（ADR D6・35）。
- Codex の closure の是正（Gated Amendment）: fresh な計数の判定の n と完了の時点の精算の n を取込みのカーソル（取込みID）から求める形（ADR D3・D6）が、取消の後に同じ番号を使う取込み（Codex の反例: 精算 1 → 入れ直す → 取消 → 初期化 → 新しい精算 1）を数えず、D7 の取り込み直しを拒まず、在庫連動を始める検査（D8）・要再確認の理由 1・完了の (ii)(iii)・完了の後の差の範囲の全てに同じく当たるか。保存した `settlement_no`（鍵の n）を判定に使う所が残っていないか。取り込み直しの後の要再確認（owner の決定 TD-400）が過剰側だけで、入れ直しで解けるか。
- 公開 repo に実データを書いていないか。並走の lane の節を触っていないか（所有表）。

## Contract Ledger

触る設計正本の節の契約を行にした（隣接の契約の sweep: ADR D1〜D12、停止 ADR SPEC-STOP-D1〜D6、tracking・master・transaction・pos、20・21・23・24・30・31・32・35・36・40・41・42・44・50・51・55・58・60・61〜64・65・66・73・74・75 の D-118 の節、共通規則 SPEC-UNIT-D3・D4・D6・D8・D11・入力の上限、§20.5a を当て、b1 の Scope が触り得ないものは下の「除外」に理由つきで書いた）。

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| SPEC-STK-CNT-D1（Z004 の movement の鍵 `(d_n, 1, n, 0)`、fresh の計数の鍵） | ADR D1、tracking | S1（CHECK）、S5（`settlement_no` を渡す） | M4・Z8 | L3-8 |
| SPEC-STK-CNT-D2（求め直す商品の原理、Z004 の確定・取消の求め直し） | ADR D2・横断の表、31 | S3・S5 | L12・Z8・Z14・X2 | L3-8・L3-10 |
| SPEC-STK-CNT-D3（精算 k の行、A1、理由 1、範囲、解け方、fresh な計数の判定の n を取込みのカーソルから求める〈round 3 の是正を Codex の closure の Gated Amendment で置き換えた〉） | ADR D3、31 | S3 | L1〜L11・L13・X1・Z23・Z24・C2 | L3-8 |
| SPEC-STK-CNT-D4（精算回数の識別・並び・欠け・未来の日付・全て 0 の file・共有 JAN の行、machine_no は取り消した取込みとも比べる〈round 3 の是正〉、取り消した取込みとだけ合わないときは拒否の文で名指す〈closure の是正〉） | ADR D4、32、24、pos | S2・S5 | Z1〜Z7・Z9・Z10・Z22 | L3-3・L3-10 |
| SPEC-STK-CNT-D4（BIZ-01 の共有 JAN の不変条件） | ADR D4、30、master | S6 | J1〜J3 | L3-12 |
| SPEC-STK-CNT-D5（既定の計数・fresh な計数・後入力の確認・日付の変わり目） | ADR D5、35、73 | S4・S8 | C1〜C8・U1〜U5 | L3-4・L3-6〜L3-9 |
| SPEC-STK-CNT-D6（完了の条件・0 にする・評価額・カーソル・完了の後の差と比較の範囲〈round 3 の是正〉、`cutoff_import_id` と完了の時点の精算の n〈Codex の closure の Gated Amendment〉） | ADR D6、35、65、tracking | S1・S2・S4・S8 | M1・M6・K1〜K8・R1〜R6・U7・U8・U13 | L3-11・L3-13 |
| SPEC-STK-CNT-D7（取消・取り込み直し。取り込み直した取込みは判定の n の算出に入らず、取り込み直した精算が判定の n だったときに日付で判定する〈Codex の closure の Gated Amendment と、その 2 本目。owner の決定 TD-400〉） | ADR D7、32 | S5 | Z4・Z14・Z22〜Z24・L13・K2 | L3-10 |
| SPEC-STK-CNT-D8（在庫連動を始める検査、切替の求め直し） | ADR D8、30、51、60 | S6・S8 | J4〜J7・U12 | L3-4 |
| SPEC-STK-CNT-D9（b1 の列〈`stock_counts.seen_import_id`・`stocktakes.cutoff_import_id` を含む〉、初期在庫の見分け方） | ADR D9、tracking・pos・master | S1・S2 | M1・M5・M6・L8・C2・K1 | — |
| SPEC-STK-CNT-D10（移行の前からの共有 JAN の組の知らせ） | ADR D10、36、75 | S6・S8 | I1・U14 | 非対象: L3（DB の手での改変が要る。画面の写し） |
| SPEC-STK-CNT-D12（b1 と b2 の境、停止を一度に外す） | ADR D12 | 全段 | AC6・U16 | — |
| SPEC-STOP-D1〜D6（停止の入口・文言・旧本体・画面・fixture・解除の仕方） | 停止 ADR（superseded にする） | S4・S5・S7・S8・S9 | AC6、T2・T4 の置き換え（X1・X2・Z/K/C の各行） | — |
| MNT-03-D18（vZ の手順） | 22 §20 | S1 | M1〜M7 | L3-1 |
| MNT-03-D1（COMMIT・ROLLBACK は共有の helper） | 22 §3.2 | S1 | 構造の test（file の列挙に足す） | — |
| D-123（作り直しの承認と順） | decision-log、22 §16 | Data Safety | — | L3-1（止まったとき） |
| IO-21（`insert_stock_count`、精算の読取り、初期在庫の印） | 21、24 | S2 | C1・C2・L8・Z8 | — |
| IO-24（`NewCsvImport` の精算の鍵、active な精算の一覧、machine_no の照合先〈取り消したものを含む 1 件。closure の是正〉、取込みの最大の取込みID〈Codex の closure の Gated Amendment〉、同じ精算の取込み） | 24 | S2 | Z1〜Z5・Z22・C2・K1 | — |
| IO-20（棚卸しの明細の読取り、共有 JAN の読取り） | 20 | S2 | C10・J1・I1 | 非対象: 商品の読取りの写しの 3 列（b2） |
| BIZ-06（計数・完了・読取り・`StocktakeGuard`・操作ログ〈round 2 の是正〉） | 35 | S4 | C1〜C11・K1〜K8・R1〜R5・W3 | — |
| BIZ-03（preview・確定・取消・引き当ての食い違い・整数の数量） | 32 | S5 | Z1〜Z16・Z21〜Z23 | L3-3・L3-8・L3-10 |
| BIZ-02（求め直しの精算の窓、`apply_stock_change` の精算回数） | 31 | S3・S5 | L1〜L13・Z8 | — |
| BIZ-01（共有 JAN・在庫連動を始める検査、一括 import の判定の状態と確定の拒否〈round 2 の是正〉） | 30 | S6 | J1〜J6 | L3-4・L3-12。非対象: 確定の TX の拒否（Test Plan の L3 の「入れない」の項） |
| BIZ-07（共有 JAN の一覧、精算の窓を持つ `derive_stock` との比較） | 36 | S6 | I1・I2 | — |
| CMD-01（`stocktake_guard`・`StocktakeRecovery`・`CountRecoveryTarget`） | 40 | S7 | W1・W8・C9・K2・Z8 | L3-4・L3-8・L3-11。非対象: 商品の結果の写しの 3 列（b2） |
| CMD-07（`additional_import_confirmed` を外す、知らせ・`recheck_targets`、token、今日） | 41 | S7 | W5 | — |
| CMD-10（新設 4、`update_count` を外す、`force_fill` を外す、今日） | 42 | S7 | W2〜W4 | — |
| D-061（生成の literal union） | decision-log | S7 | W6・W7 | — |
| UI-10（棚卸しの画面の新契約、UI-10-D1〜D13 を保つ） | 73 | S8 | U1〜U9・U15・U16 | L3-4〜L3-11 |
| UI-07（Z004 タブの拒否・知らせ・結果の要再確認） | 55 | S8 | U10・U11 | L3-3・L3-8・L3-10 |
| UI-01b（共有 JAN と `recount_required` の表示と導線、導線が離脱の確認を通ること〈round 3 の是正〉） | 51 | S8 | U12 | L3-4（JAN は商品フォームで変えられないので、共有 JAN の拒否は L3-12 の一括 import で見る）。非対象: 在庫数の表示の範囲（b2） |
| UI-01c（共有 JAN と連動を始める行の error 行、確定の拒否の表示〈round 2 の是正〉） | 60 | S6・S8 | J3（BIZ と画面）・J5 | L3-12 |
| 65（棚卸しの記録の詳細・入出庫の一覧の件数） | 65 | S2・S4・S8 | R1〜R5・U13 | L3-13。非対象: 在庫変動の履歴の計数の行（b2） |
| UI-13（共有 JAN の一覧） | 75 | S8 | U14 | 非対象: L3（画面の写し） |
| UI_TECH_STACK の D-118 の節（再取得の経路と entry と要再確認の一覧の key〈round 2 の是正〉、未保存の入力だけを UI の状態に） | UI_TECH_STACK | S8 | U15 | — |
| demo の seed の reset（`delete_all` の表に `stock_counts`。round 2 の是正） | packet の S9（開発の道具で設計正本の節は無い） | S9 | E1 | — |
| SPEC-UNIT-D3・D11（棚卸しの実数の m の入力と表示、明細の `stock_unit`） | 共通規則 | S4・S8 | U4・C10 | L3-6 |
| SPEC-UNIT-D4・D8（Z004 の整数の数量の換算） | 共通規則、32 | S5 | Z12 | 非対象: 小数の数量（P） |
| SPEC-UNIT-D6・SPEC-STK-VAL-D1〜D6（評価額と行のロス原価） | 共通規則、35 §20.5a | S4 | K1・K5・K6・R1 | — |
| 入力の上限（計数 `999999999`、Z004 の数量 `9999999`、評価額の総額の検査） | 共通規則「入力の上限と安全な整数の範囲」の (a)(c) | S4・S5 | C4・K5・Z12 | — |

除外（触らない契約と理由）: SPEC-STK-CNT-D11（EJ を在庫に使わない）は b1 で触る code が無い。21 の `MovementRecord` の並びの鍵、40 の商品の結果の写しの 3 列、44 の `list_movements`、50・58・66 の範囲の表示、51 の在庫数の表示、65 の在庫変動の履歴の計数の行は b2。61〜64 の日付の欄の意味の注記は (a) で入った契約で画面を変えない（S9 で注記の「(b) は proposed」の文だけ直す）。74 の操作ログの表示は operation_type の label だけを読み detail の key を読まない（`rg -n 'adjusted_count|force_fill_used|auto_filled_count|difference_count' docs/function-design/74-ui-operation-logs.md src/features --glob '!*.test.*'` が 0 件、2026-10-22）ので、35 の「操作ログ」で detail の key を変えても触らない（round 2 の是正）。23 の小数の数量（`quantity_hundredths`）は P。37 の SD の一括取込みは (B)。

## Contract Audit（起票の時点の一巡）

新しく足す契約の文（本 commit の正本の修正と、上の横断の約束・Boundary）を、境界の 4 観点で一巡した。

- 安全な整数の範囲: 精算回数（32 の順 1）は入力の境で `9007199254740991` 以下に限り、知らせの区間の端も同じ範囲（Z5・Z10）。計数・在庫・範囲・評価数量は `999999999`（Z1・Z4・Z11）。評価額の総額は完了の TX で `check_safe_integer`（Z12。backlog の `total_cost` の項目はこれで不要になる）。読取りで求める金額（行のロス原価・完了の後の評価額の差）は範囲が決まらないので十進の文字列（Z13）。Z004 の長さの商品の × 100 は 1 明細の数量の上限で行の error（Z8）。→ 正本の不足は 32 の 2 か所（順 1 の範囲、数量の上限）で、本 commit で直した。
- 0 の表示: 計数 0 は有効（売り切れ・「0 にする」）。精算回数 0 は実在しうるので、fresh な計数の「取込みが無い」は NULL で表す（tracking のまま）。正常な行が全て 0 の Z004 は取り込む（32 のまま）。数量・金額とも 0 の行は売上も movement も作らない。差 0 の明細は完了の後の差の一覧に出さない（Z13）。`valuation_quantity` は `max(…,0)`。→ 不足は無かった。
- 失敗の時点と順序: 計数の保存は形と暦 → 今日 → 値 → 明細 → INSERT → 求め直し（S4、Z1・Z2）。Z004 の確定は TX の中で順 1〜5 → JAN の候補の取り直し（食い違いは拒否）→ 書く → 求め直し → 結果（32、S5）。完了は 3 つの理由を全部調べてから拒否し、何も書かない（35）。preview token は成功のときだけ消す（41）。vZ は止まる検査が何かを変える前（22 §20）。→ 正本の不足は 2 か所（preview と確定の引き当ての食い違い〈D-122 からの申し送り〉、token の消し方）で、本 commit で直した。
- 単位や種類の混在: 計数・movement・写しは同じ在庫の数量の単位（長さは cm）。Z004 の行の数量は × 100 で cm にそろえ、売上の数量も同じ単位（日次・月次の `length_cm` と同じ）。評価額と行のロス原価は価格の基準数量で割る。共有 JAN の行は売上だけで在庫連動の外、`stock_linked` は取込みのときの連動（今の設定と混ぜない）。初期在庫と demo の seed の売上の movement を `movement_type` で分ける。→ 正本の不足は 2 か所（Z004 の整数の数量の換算〈32〉、初期在庫の見分け方〈ADR D9〉）で、本 commit で直した。

round 1 の是正で足した契約の文（Z18〜Z20、32 の「preview の cache の契約」、30 の経路、31 の Z004 の movement の書き方、`NewSaleRecord.stock_linked`、`apply_stock_change` の引数の撤去、22 §20 の失敗の test）の一巡:

- 安全な整数の範囲: `CountRecoveryTarget` と `CountPlacementPreview` の `stock`・範囲は写しの値で `999999999` の内（Z4）、`SettlementPendingHint.missing` の端は精算回数の範囲（Z5）で、どれも wire の number で正確。cache の `MatchedRow.quantity` は i64 で × 100 の後に `9999999` と比べる（Z8）。→ 不足は無かった。
- 0 の表示: 範囲は両方 null か両方の値（DB の CHECK と同じ対）。`SettlementPendingHint` は直し方が 1 つも無ければ None（`settlement_pending` の target があれば (i)〜(iii) のどれかに当たるので、少なくとも 1 つが値を持つ）。計数の無い商品の文脈は `count_*` が null。`stock_linked` は手動の売上ほかで false を明示する。→ 不足は無かった。
- 失敗の時点と順序: 確定の照合（Z19）は TX の中で書く前に行い、未登録だった行に候補ができた場合も含めて拒否し、token を残す。vZ の作り直しの後の失敗（COMMIT の失敗と、手順 6 の外部キーの確かめの失敗）が全部戻ることを M8・M9 で確かめる。→ 正本の不足は 32 の cache の契約（照合に要る元の JAN と精算の識別メタを持つ所が無かった）で、本 commit で直した。
- 単位や種類の混在: 要再確認の範囲は商品の在庫の単位で `stock_unit` と一緒に運ぶ（長さは cm を画面が m に）。共有 JAN かどうか（候補の数）を照合に入れる（単独 → 共有に変わると在庫を動かすかが変わる）。JAN の変更は一括 import の上書きの経路だけ（商品フォームは JAN を変えない）。→ 正本の不足は 40（範囲と単位を運ぶ型が無かった）と 30（update が JAN を変える前提の文）で、本 commit で直した。

round 2 の是正で足した契約の文（30 の一括 import の判定の状態・確定の拒否の形と message・在庫連動を始める拒否の順、35 の `zero_uncounted_items` の明細の参照と拒否の message・操作ログの detail と summary の差異の件数、UI_TECH_STACK の再取得の経路と entry と key `queryKeys.stocktake.recheckList()`、60 の確定の拒否の表示、L3 の DB の用意と差し替えの手順、demo の seed の reset）の一巡:

- 安全な整数の範囲: 新しい数は `difference_count`（明細の数以下の非負の整数。明細の数は商品の数で、安全な整数の範囲の内）だけで、DB にも wire にも出ない（操作ログの JSON の中）。→ 不足は無かった。
- 0 の表示: 差異のある明細が 0 なら summary は「差異: 0件」（今の形と同じ）。差異が null の明細（完了の時点で未計数の廃番かつ写し 0 の明細）は数えない。「0 にする」の計数は値 0 で明細を参照する（参照しない 0 の計数は未計数のまま残る穴を C8 が止める）。L3 の空の DB は商品 0 件から始まり、評価額と明細の数は P1〜P6 の分だけになる。→ 不足は無かった。
- 失敗の時点と順序: 一括 import は preview で行の error、確定は TX の中で送られた行の順に判定して最初に当たった行で全体を戻す（何も書かない。名指すのはその 1 行で、他の行は preview のやり直しで見える）。在庫連動を始める保存は共有 JAN の拒否を `recount_required` より先に返す。`zero_uncounted_items` は書く前に全ての商品の明細を確かめる（1 つでも無ければ何も書かない）。操作ログは今のとおり COMMIT の後で、書けなくても業務を戻さない。L3 の差し替えはアプリが止まった後だけ。→ 正本の不足は 30（確定の拒否の形と判定の状態・順）と 35（明細の参照）で、本 commit で直した。
- 単位や種類の混在: 確定の拒否の message は preview の行の error の文をそのまま含み（共有 JAN と `recount_required` の 2 種類）、UI は解析しない。要再確認の一覧の key は写しの consumer と別の key で、開始・完了（写しを変えないが回復先を変える）にも足す。操作ログの summary の「差異」は数量でなく明細の数。→ 不足は無かった。

この是正で、新しい型・新しい wire の field・新しい画面の状態は足していない（確定の拒否は今の `validation` と Alert、invalidation は今の SSOT の entry の中身、操作ログは今の 2 行の key）。足した契約の文は上の一覧と、Matrix の観測の手段（C5 の外部キー、K1・L12 の写しの食い違い、W3 の過去の今日、X の trace の日付と精算回数）。

round 3 の是正で足した契約の文（32 の順 2 を取り消した取込みとも比べる、ADR D3 の fresh な計数の n を使える条件、ADR D6・35 の完了の後の比較の範囲と `cutoff_movement_id`、51・73 の導線と離脱の確認と `/stocktake/` の search の `product`・`returnTo`）の一巡:

- 安全な整数の範囲: `cutoff_movement_id` は SQLite の id で DB の中だけ（wire に出ない。`StocktakeRecordDetail` は形を変えない）。n は Z5 の範囲。新しい wire の数は無い。→ 不足は無かった。
- 0 の表示: 精算回数 0 の n も active な取込みの有無で決める（0 を「無い」の印にしない）。movement のカーソルは movement が無ければ 0（id は 1 から。`seen_movement_id` と同じ）。完了の直後の比較は評価と同じ記録になるので差 0 で、差の一覧は空、確定の差の合計は ¥0（R2）。→ 不足は無かった。
- 失敗の時点と順序: 順 2 は preview と確定の TX の両方で取り直し、書く前に拒む（Z15）。n を使えるかは保存せず、求め直しと完了の TX が取込みの今の状態から決める（取消・取り込み直しの TX の求め直しで写しが揃う）。導線は離脱の確認の後にだけ移り、「破棄して移動」は何も書かない。棚卸しの画面は保存が成功した後にだけ `returnTo` へ移る。→ 不足は 3 か所（取消の後の順 2 と n、完了の後の比較の範囲、導線の guard）で、本 commit で直した。
- 単位や種類の混在: machine_no（レジ）と精算回数（系列の中の順）を分け、n は同じ系列の精算回数とだけ比べる。完了のカーソルは計数・movement・精算の種類ごとに自分の種類とだけ比べ、Z004 の movement は精算の規則で切る。`returnTo` は path だけを受け、商品フォーム自身の `returnTo`（一覧の search）を入れ子にしない。→ 不足は無かった。

この是正で、新しい wire の型・field・画面の状態は足していない（DB の列は vZ の手順 5 に `stocktakes` の `cutoff_movement_id` を 1 つ。route の search の key は S8 で予定していた商品コードと `returnTo` に名前を付けた）。消した記述: tracking の「入出庫の movement はカーソルを持たず、業務日で切る」、ADR D6 と 35 の「完了日より後の日付の記録は入れない」、51 の回復先の行き先の分け（`active_count` なら棚卸しの明細）、32 の「取り消した取込みは比べる先にしない」を順 2 にも当てる読み、L3-4 の「元の商品フォームに戻って、もう一度保存」。

Plan Review closure の是正で足した契約の文（32 の順 2 の取り消した取込みを名指す拒否の文と比べる先、24 の machine_no の照合先の読取り〈新設〉、ADR D4・D7 の同じ文、ADR D6・35・tracking の完了の後の比較の範囲から完了の後に保存した計数を除くこと）の一巡:

- 安全な整数の範囲: 文に入る精算回数は Z5 の範囲、取込みID は SQLite の id で、どちらも `import_error` の message の文字列の中に入り、wire の number にならない。新しい wire の数・DB の列は無い。→ 不足は無かった。
- 0 の表示: 精算回数 0 は「精算 0」と出す（0 を「無い」の印にしない。Z5 と同じ）。machine_no は leading zero を保つ文字列のまま出す（Z6）。照合先が None なら順 2 を通る（取込みが無い DB と、同じ machine_no の取込みだけの DB）。完了の後に保存した値 0 の計数も、ほかの計数と同じく比較の範囲に入れない。→ 不足は無かった。
- 失敗の時点と順序: 順 2 は順 1 の後・順 3 の前で、preview と確定の TX の両方で照合先を読み直し、書く前に拒む（Z15。名指すのはその TX で読んだ取込み）。machine_no の違う active な取込みがあれば今の文を返し、取り消した取込みだけのときに名指す文を返す（24 の照合先が active を優先するので、1 回の読取りで分岐が決まる）。完了の後の比較は読取りで求め、何も書かない。→ 不足は 32 の順 2（原因を名指さない）と 24（読取りが 1 通りに決まっていない）で、本 commit で直した。
- 単位や種類の混在: 名指す日付は取込み日（`imported_at`、PC の日時）で、精算日（`settlement_date`、レジの日付）と取り違えない（文の語を「取込み日」にする。既存の重複の拒否の文と同じ）。machine_no（レジ）と精算回数（系列の中の順）を分けて出す。完了の後の比較は、入出庫・初期在庫・精算を「完了の前の記録」で足し、計数はカーソル以下だけ（計数は観測で、記録の追加と種類が違う）。→ 不足は ADR D6・35（計数を「完了の前の記録」に含める読みを許した）で、本 commit で直した。

この是正で、新しい wire の型・field・画面の状態は足していない（拒否の文は今の `import_error` の BIZ の message。[41](../../function-design/41-cmd-pos.md)・[55](../../function-design/55-ui-csv-import.md) は BIZ の文をそのまま出す）。Matrix の K2 に足した列（N3）は契約の文を足さず、round 3 で置いた 35 の完了の (iii) の n の扱い（n の active な取込みが無ければ NULL）を完了の判定で観測する（4 観点では、失敗の時点と順序の「取消・取り込み直しの TX の後の完了の判定」に当たり、不足は無かった。精算回数は Z5 の範囲、hint の日付は Z18 の形）。消した記述: 24 の「今の一覧の読取りで足りれば新設しない」、残りの危険の「レジの入替と同じ止まり方…で続け方を決める」、ADR D4 の順 2 と順 5 を 1 つにまとめた文、ADR D7 の「D4 の比べる先に取り消した取込みは入らない」、35 の非遡及の例の「後入力の計数」。

Codex の Plan Review closure（P1）の Gated Amendment で足した・置き換えた契約の文（2026-10-23。owner の決定〈repo 外の回答台帳 TD-389・TD-400〉）:

1. ADR D3（tracking・35・横断の約束の Z3）: fresh な計数の判定の n は、保存の時点の取込みのカーソル c（`seen_import_id`）以下の取込みID を持つ、今 active な Z004 の取込みの最大の精算回数（無ければ NULL）。求め直し・完了の判定・記録の詳細の読取りのたびに今の取込みから求め、保存しない。保存した `settlement_no` は D1 の鍵の n で、判定に使わない（round 3 の「n を使うのは精算回数 n の active な取込みがあるときだけ」を置き換えた）。
2. ADR D1・D5・D9、tracking、22 §20 の手順 5、35: fresh な計数は保存の TX で `MAX(csv_imports.id)`（取り消したものを含む。無ければ NULL）を `seen_import_id` に残す。列は `INTEGER CHECK(seen_import_id IS NULL OR (count_kind = 'fresh' AND seen_import_id >= 1))` で、vZ の前の行は NULL（保存の時点に取込みが無かった）。
3. ADR D6、35、tracking、横断の約束の Z11: 完了の (i)〜(iii) の n は判定の n。完了のカーソルは `cutoff_import_id`（完了の時点の `MAX(csv_imports.id)`、取り消したものを含む、無ければ NULL）・`cutoff_count_id`・`cutoff_movement_id`（`cutoff_settlement_no` を置き換えた）。完了の後の比較の範囲の精算は、完了の時点の精算の n（`cutoff_import_id` 以下の今 active な取込みの最大の精算回数）以下のものと精算日が完了日より前のもの。
4. ADR D7、32 の取消の節: 取り込み直した取込みは、それより前に保存した fresh な計数と完了の時点のカーソルにとって後の取込みなので、判定の n・完了の時点の精算の n の算出に入らない。取り込み直した精算は D3 の表で判定し、判定の n だった精算を取り込み直したときに日付の判定になり、精算日が計数の日なら、その精算に行のある商品は要再確認になる（owner の決定 TD-400 の A。「日付で判定する」を全ての取り込み直しに広げた初めの文は、Gated Amendment の 2 本目で D3 の表に合わせた。下の 2 本目の段）。
5. 24: 「Z004 の取込みの最大の取込みID」（取り消したものを含む `MAX(id)`、無ければ None）の読取りを新設する。BIZ-06 が保存と完了の TX で読む。判定の n は BIZ が active な一覧（import ID を含む）から求める。
6. POS: 取込みID（`csv_imports.id`、AUTOINCREMENT）は使い回さない（取消は行を消さない）。取込みのカーソルはこれに依る。
7. 21・31: `list_stock_counts_for_product` は `seen_import_id` を返し、`insert_stock_count` が書く。求め直しの入力の精算の一覧は import ID を含む。
8. 32 の順 2 と順 5 の注: 順 2 を取り消した取込みとも比べる理由は「店のレジは 1 台の前提」と owner の決定（TD-388）で、判定の n が取り消した取込みと保存の後の取込みを数えないので、番号の戻りと別のレジの file は日付の判定に倒れる（「k ≤ n が別の系列と比べられる」の理由は案 1 で無くなった）。
9. ADR の Consequences（過剰側）と Revisit、本 packet の「残りの危険」: 取り込み直しの後の要再確認と完了の後の参考の差、同じ日のうちの初期化の残る穴、TD-400 の B を見直す契機。

境界の 4 観点の一巡:

- 安全な整数の範囲: 取込みのカーソル（`seen_import_id`・`cutoff_import_id`）は SQLite の id で DB の中だけ（wire に出ない。`StockCountSaveResult`・`CountPlacementPreview`・`StocktakeRecordDetail` の形は変えない）。判定の n と完了の時点の精算の n は Z5 の範囲の値の最大なので範囲の中。新しい wire の数は無い。→ 不足は無かった。
- 0 の表示: 取込みのカーソルは NULL が「取込みが無い」（id は 1 から。0 は CHECK で拒む）。判定の n の NULL と精算回数 0 を分ける（`Option`。精算 0 は実在しうる。Matrix の L4 の「NULL を 0 に倒す」の mutant がそのまま当たる）。完了の時点の精算の n が NULL のとき、範囲の精算は精算日 < 完了日だけ。movement のカーソルは 0 が「無い」のままで、計数の中で印の形が 2 通りになる。→ 不足は 1 か所（印の形が違う理由）で、tracking と横断の約束の Z3 に書いた（列を後から足すので既存の fresh の行に値を入れられず、NULL が意味として正しい）。
- 失敗の時点と順序: c は計数の INSERT と同じ TX で読む（Z3 の `seen_movement_id` と同じ。TX の外で読む mutant は 1 本の接続の test で観測できないので、Matrix の Residual Test Gaps の (1) の code 読みに足した）。`cutoff_import_id` は完了の TX で読む。判定の n は保存しないので、取消・取り込み直し・取込みの確定の TX の求め直し（計数を持つ全ての商品。ADR D2）で写しが揃い、新しい求め直しの契機は要らない。確定の TX の中で新しい取込みの id は INSERT で決まり、既存の計数の c より必ず大きい。→ 不足は無かった。
- 単位や種類の混在: 4 種類の整数を混ぜない。取込みのカーソルは `csv_imports.id` とだけ、判定の n と完了の時点の精算の n は `settlement_no` とだけ、m は movement の id とだけ、`cutoff_count_id` は計数の id とだけ比べる。保存した `settlement_no`（鍵の n）と判定の n を取り違えない（Codex の P1 はこの取り違えと同じ形）。c は取り消したものを含む全ての取込みの最大、判定の n は active な取込みだけから求める（取り消した取込みを判定の n に含めると Codex の列が戻る。Matrix の L13 の (a) の mutant）。fixture は精算回数と取込みID の値を離す（`k ≤ c` の mutant を観測するため）。machine_no は判定の n に入らない（active な取込みは 32 の順 2 で 1 台に揃う）。→ 不足は無かった。

この是正で、新しい wire の型・field・画面の状態は足していない（DB の列は vZ の手順 5 に `stock_counts.seen_import_id` を 1 つ足し、vZ の未 merge の列 `cutoff_settlement_no` を `cutoff_import_id` に置き換えた。vZ の列は 8 つ）。消した記述: ADR D3 の round 3 の特例（「n を使うのは精算回数 n の active な取込みがあるときだけ」「n の精算を取り込み直すと n はまた使える」）、35 の完了の表の同じ括弧、tracking の「settlement_no の精算の active な取込みが無いときも NULL とみなす」、横断の約束の Z3 の同じ文、32 の順 2 の理由の「k ≤ n が別の系列の精算回数と比べられる」、「残りの危険」の 2 項（番号の戻りが古い fresh な計数の n に届く穴、n の精算を取り消してから取り込み直すまでの過剰側）、Matrix の L13・K2 の旧い列、vZ の未 merge の列 `cutoff_settlement_no`。

Gated Amendment の 2 本目（2026-10-23。Amendment の review の Codex 側 P2 1 件と Claude 側 P3 3 件、Coordinator の裁定）で直した・足した契約の文:

1. ADR D3 の判定の n の項・D7、32 の取り込み直し: 取り込み直した取込みは判定の n・完了の時点の精算の n の算出に入らない。取り込み直した精算 k は D3 の表のとおり k ≤ n なら入り、それ以外を日付で判定する。日付の判定で要再確認になるのは、取り込み直した精算が判定の n だった場合（Codex の反例: 同じ日の精算 99・100 → fresh な計数 → 99 を取消 → 正しい 99 を取り込み直す、では n = 100 のままで 99 は入る。Matrix の L13 の (c)）。Rust の判定は変えていない（初めの文が D3 の表より広かった）。
2. ADR の Consequences: 受け入れた穴の列挙の後に、同じ日のうちの初期化は列挙の外の前提で不足側へ倒れることを 1 文。完了の後の取り込み直しが記録の詳細の参考の差の確定の差に出ることを、過剰側の項から別の項に分けた（要再確認にならない。挙動は D6 のまま）。本 packet の「残りの危険」も同じ形に分けた。
3. 73 の D-118 の節と UI-10-D2 の注記: 検索で開く欄で「入れ直す」を出すときの文を「数を直すときは「入れ直す」を押してください。同じ日付で「数を保存」から違う数を入れると、数え直しが必要になります」にした（b1 の前の「入力済みの数を上書きできます」は、同じ日付の違う値の既定の計数が要再確認〈理由 3〉になる b1 の振舞いと合わない）。文を出す条件は「入れ直す」の button と同じ（前に保存した数があり、前の日の分を入れていない）。
4. 本 packet の L3-11 の台本の button の名を画面の「棚卸しを確定する」に合わせた。上の段の DB の列の書き方を、表名を付けない `cutoff_settlement_no` にした（`doc-consistency-check.sh --target plan` の WARN）。

境界の 4 観点の一巡:

- 安全な整数の範囲: 新しい数・wire の field は無い（L13 の (c) は既存の精算回数と取込みID の範囲）。→ 不足は無かった。
- 0 の表示: 判定の n の NULL と精算回数 0 の扱いは変えない。欄の文を出す条件は前に保存した数の有無（`count_value` が null か）で、数 0 の計数も前に保存した数として文が出る。→ 不足は無かった。
- 失敗の時点と順序: 取消・取り込み直しの TX の求め直しは変えない。判定の n は保存しないので、取り込み直した精算が判定の n だったかは読取りのたびに今の取込みから決まる（L13 の (b)・(c)）。完了の後の差は読取りで求めるので、取り込み直しの後の読取りに出る（挙動を変えないので test は足していない）。→ 不足は無かった。
- 単位や種類の混在: 「判定の n の算出に入らない」（取込みID のカーソルの話）と「k ≤ n に当たらない」（精算回数の話）を分けて書いた（Codex の P2 はこの 2 つを 1 つにした文）。完了の後の差の確定の差と要再確認の差を分けた。→ 不足は 1 か所（上の 1）で、直した。

この是正で、新しい wire の型・field・画面の状態・DB の列は足していない。消した記述: 32・ADR D3・D7・本 packet の「取り込み直した取込みは日付で判定する」の一般化、Consequences の過剰側の項の完了の後の取り込み直しの文（別の項へ移した）、画面の「入力済みの数を上書きできます」と、その文を出す条件の `hasItemCount`（「入れ直す」の button と同じ条件にした）。

## Contract Probe

本 lane が依る外部の前提（同梱の SQLite の振舞い）を、2026-10-22 に repo の外の scratch の crate（`rusqlite = "=0.31.0"` bundled、`sqlite_version()` → `3.45.0`。commit しない）で確かめた。同じ文を python の sqlite3（3.51.3）でも当て、同じ結果だった。

- P1 行の無い表への列の追加で「値を持つ」を列の CHECK で書く: `ALTER TABLE csv_imports ADD COLUMN machine_no TEXT CHECK(machine_no IS NOT NULL AND machine_no <> '')` と `ALTER TABLE csv_imports ADD COLUMN settlement_no INTEGER CHECK(settlement_no IS NOT NULL AND settlement_no >= 0)` -> どちらも ok。その後の INSERT で `machine_no` が NULL・空、`settlement_no` が NULL・-1 は全部 `CHECK constraint failed`、`('0001', 42)` は ok。
- P2 行のある表への同じ追加は止まる: 行が 1 つある表で同じ `machine_no` の CHECK の追加 -> `CHECK constraint failed`。`ADD COLUMN x TEXT NOT NULL`（既定値なし）-> `Cannot add a NOT NULL column with default value NULL`。`ADD COLUMN stock_linked INTEGER NOT NULL DEFAULT 0 CHECK(stock_linked IN (0,1))` -> ok、既存の行は 0、2 への UPDATE は `CHECK constraint failed`。→ 22 §20 の手順 1 で `csv_imports` に行のある DB を先に止め、手順 5 の CHECK の失敗を原因の分からない失敗にしない。
- P4 行のある `stock_counts` への取込みのカーソルの列の追加（2026-10-23、Codex の closure の Gated Amendment。python の sqlite3 3.51.3 で当てた。同梱の 3.45.0 では Matrix の M6 が同じことを確かめる）: fresh と default の行がある表で `ALTER TABLE stock_counts ADD COLUMN seen_import_id INTEGER CHECK(seen_import_id IS NULL OR (count_kind = 'fresh' AND seen_import_id >= 1))` -> ok、既存の 2 行は NULL。その後の INSERT で fresh・NULL、fresh・1、default・NULL は ok、fresh・0 と default・1 は `CHECK constraint failed`。
- P3 Z004 と phase を結ぶ CHECK `(reference_type IS 'csv_import') = (phase = 1)`（`(phase = 1) = (settlement_no IS NOT NULL)` と一緒）: `receiving`・NULL・0、`sale_auto`・NULL・0（demo の seed の形）、`csv_import`・1・7、`csv_import`・1・0 -> ok。`csv_import`・0、`receiving_record`・1、NULL・1 -> `CHECK constraint failed`。→ seed の売上の movement は vZ の CHECK に当たらず、初期在庫の見分け方（ADR D9）だけで足りる。

## Data Safety

- commit しない物: 実 POS / 店舗データ、DB、backup、log、receipt。fixture は全部合成（商品コード・JAN・品名・日付・金額は合成の値。tracked に置く POS の sample は合成の fixture だけ〈owner の決定 TD-296〉）。
- local だけの場所: owner の PC の `%APPDATA%\com.inventory-system.desktop`（DB と `-wal`・`-shm`、`backups`、`logs`、`images/receipts`、復元の遺物）と `%LOCALAPPDATA%\com.inventory-system.desktop`（WebView の保存）、起動の作業フォルダの旧 DB 一式、設定した backup の保存先。
- L3 の DB の用意と差し替え（round 2 の是正。Test Plan の「L3 の DB の用意と差し替え」。D-123 でリリースの前まで開発・試験 DB の作り直しは包括で承認済み）: L3-1 の前と後の 2 回、owner の L3 用の data の folder（上の local だけの場所のうち `%APPDATA%\com.inventory-system.desktop` と `%LOCALAPPDATA%\com.inventory-system.desktop` の全部と、起動の作業フォルダの旧 DB 一式）を、22 §16 の作り直しの手順のとおり、日付の付いた folder の下に元の場所ごとの sub folder を分けて**消さずに移す**（backup の保存先の file は動かさない）。L3-1 の前に置く DB は Coordinator が main の build の demo の seed で作った合成の DB だけ。実行は owner の L3 の依頼の範囲で Coordinator（WSL の `/mnt/c/Users/...` から。消す command を使わない）。L3 の後は戻さず、移した folder は残す（消すかは owner の判断）。L3-1 で vZ が止まったときは、文の写しを 22 §20 の手順 1・6 と突き合わせ、3. の差し替えで空の DB にして L3-2 から続ける。
- 合成だけの場所: Rust と TS の test の fixture、`tests/fixtures/z004/` の L3 の Z004 と商品の CSV。

## 残りの危険（起票の時点）

- b1 の後・b2 の前の版では、商品の画面に要再確認の範囲が出ず、在庫変動の履歴に計数の行が無い（ADR D12。店へ入れる版にしない）。
- preview の後の拒否（状況の変化）で token を残すので、利用者が同じ「取り込む」を押し直すと同じ拒否が出る（文が「ファイルを選び直してください」を含み、token は今の 30 分の期限で消える）。
- 移行の前からの共有 JAN の組は、整合性の画面を開いて「チェック」を押すまで知らせが出ない（ホームや取込みの結果には出ない）。Z004 の取込みでは、その商品の行は売上だけで在庫が動かない（D4 のとおり）。
- Z004 の小数の数量の行は、P が入るまで行の error で取り込まれない（売上も在庫も入らない。黙っては入らない）。2026-09-30 の実物の Z004 に小数の個数の行が 1 つあった（backlog の入力 4 の (e)）。
- 計数の衝突（同じ日付の別の値の既定の計数）は L3 で見ず、自動 test と画面の写しで見る。
- 商品一括 import で、上書きを選んだ行どうしが同じ新しい JAN になり片方が在庫連動になる組は、preview では知らせられず（上書きするかが preview の時点で分からない）、確定で行を名指して拒む（30 の D-118 の節。利用者は preview に戻り、その行をスキップにするか CSV を直す。round 2 の是正）。
- Z004 の取込みを取り消す入口は取込みの結果の画面だけ（55 §55.4 の手順 18）。結果の画面を離れた後に誤った版に気づくと、アプリの中で取り消して取り込み直せない（b1 の版は店へ入れない。入口は owner の決定〈2026-10-22、repo 外の回答台帳 TD-380〉で「導入の前の関門」の項目になり、別の lane で入れる。closeout で [Plans](../../Plans.md) の `## 導入の前の関門` と backlog に足す。Non-scope の項）。
- 取り消した取込みしか無い DB でも、別の machine_no の Z004 は拒む（照合は残す。owner の決定〈2026-10-23、repo 外の回答台帳 TD-388〉）。拒否の文は原因の取り消した取込み（取込みID・精算回数・取込み日・レジ番号）を名指す（32 の順 2）が、照合の基準を入れ替える操作は無いので、別のレジの file を誤って最初に取り込んで取り消した DB では、店のレジの file を取り込めない（b1 の版は店へ入れない）。これが実際に起きたとき、または ADR の Revisit の契機（レジの入替・初期化）で、基準を入れ替える操作を見直す（closure の是正）。
- fresh な計数と完了の判定の n は取込みのカーソルから求めるので、取消の後に同じ番号を使う取込み（取り込み直した版・初期化の後の新しい系列・別のレジ）を数えず、番号の戻りでは黙ってずれない（Codex の closure の P1 を閉じた。ADR D3・D6。完了の後の差の範囲も同じ）。残るのは同じ日のうちの初期化: 初期化した新しい系列の精算を、精算回数がそれより大きい active な取込みと同じ精算日で取り込むと、32 の順 5 は同じ精算日を食い違いとしないので止めず、その精算回数が fresh な計数の判定の n 以下なら計数に入っているとみなす（不足側。レジの初期化は ADR の Revisit の契機で、店の資料と実機でしか確かめられない）。
- fresh な計数の保存の前に取り込んだ精算のうち判定の n の精算を取り消して取り込み直すと、取り込み直した精算は保存の後の取込みとして日付で判定され、精算日が計数の日なら、その精算に行のある商品は要再確認になる（過剰側。入れ直すと解ける。owner の決定〈2026-10-23、repo 外の回答台帳 TD-400〉。Matrix の L13・Z24・K2）。年末の「締め → 取込み → 入れ直し → 完了」の途中で誤った版に気づいて取り込み直すと、入れ直した商品のうちその精算に行のあるものをもう一度入れ直す（b1 では取消の入口が取込みの結果の画面だけ〈上の項〉なので、入れ直しの後の取消はほぼ起きない）。判定の n の精算を取り消すだけなら、それより前に保存の前に取り込んだ精算はカーソル以下に残るので、決まらない側へ倒れるのは取り消した精算そのものだけ。判定の n でない精算（n より小さい精算回数）を取り消して取り込み直すと、n は変わらず、取り込み直した精算は k ≤ n で計数に入る（要再確認にならない。Matrix の L13 の (c)）。
- 完了の時点の精算の n の精算を完了の後に取り消して取り込み直すと、取り込み直した精算は精算日が完了日より前でなければ完了の後の比較の範囲から外れ、その精算で在庫が動いた明細の記録の詳細の参考の差の、確定の差に出る（要再確認にはならない。評価額と在庫は変わらない。ADR の Consequences）。

## Implementation Results

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

PR: [#188](https://github.com/kosei-w90607/inventory-system-desktop/pull/188)（2026-10-24 に helper 経由で squash merge）。

- 1 段目（S1〜S7 と S9 の L3 の fixture の生成）: migration v11（vZ）、精算の窓と理由 1、計数の保存・0 にする・要再確認の一覧・完了の新しい経路、Z004 の preview・確定・取消の新しい経路、共有 JAN と在庫連動を始める検査、`stocktake_guard` と回復型、新設 4 command と bindings、`io::l3_fixtures`。停止の入口・旧本体・時点証拠の code と probe を撤去し、STK-1・STK-2 の診断 2 本を通常の回帰 test にした。Gated Amendment（取込みのカーソル）の code も同じ段に入った。
- 2 段目（S8 と S9 の残り）: 棚卸しの画面（数を保存・入れ直す・前の日の分・日付の変わり目の確認・m の入力・要再確認の一覧・「今の数を入れる」の入口〈`/stocktake/` の search の `product`・`returnTo`〉・完了の拒否の理由ごとの一覧と「0 にする」・在庫を動かさない結果）、Z004 タブ（知らせ・取込みと取消の結果の要再確認の一覧、停止と同じ日の追加の確認を外す）、商品フォームの `recount_required` の導線（離脱の確認を通る）、商品一括 import の確定の拒否の画面の test、棚卸しの記録の詳細（BIZ の差異とロス原価・完了の後の差）、整合性の画面の共有 JAN の欄、`CMD_ERROR_KIND` と D-052 の SSOT の要再確認の一覧の key。正本の「proposed・未実装」の b1 の分を現行に直し、停止 ADR を superseded に、停止の節（35 §20.0・32 §15.0・55 §55.0・73 §73.15）を外し、旧本体の手順を新しい経路に置き換え、ERD・db-design の列・`tests/fixtures/z004/README.md` の L3 の fixture の表を足した。
- 実装で決めた画面の細部と packet との食い違いは PR の本文と 73 の D-118 の節（「b1 の画面の細部」）に書いた。

closeout（2026-10-24）:

- L3（2026-10-24、owner が Windows native の `npm run tauri dev` で最後の head を使って実施）: L3-1〜L3-13 は全段 PASS。r4 は L3 の全段 PASS を条件に最後の head で owner が承認した（repo 外の回答台帳 TD-431）。介入の上限は 9 から 11 に改定し（TD-432）、merge までの消費は 10。
- L3 での台本の読み替え (a): data の folder は PR #191 で `identifier` が変わったので今の名 `com.inventory-system.desktop` に読み替えた。Test Plan の L3 の DB の用意と Data Safety の 3 行は closeout で今の名に直した。
- L3 での台本の読み替え (b): L3 の fixture は、Windows の `cargo test --lib` の test の実行 file に tauri-build の manifest が入らず `STATUS_ENTRYPOINT_NOT_FOUND`（`TaskDialogIndirect`）で起動しないため、WSL で同じ head・同じ日付で生成して写した。
- L3 での台本の読み替え (c): L3-7 の期待「一覧の P1 の数は 10（日付 T−2）」の日付は、一覧でなく商品を選んだときの「前に保存した数」の欄に出る（一覧の「最終カウント」は 73 UI-10-D10 の保存の時刻）。DB の業務の日付は期待どおりだった。
- review・CI・merge: Final Review の結果は下の Review Response の closeout の段。Ready 後の hosted CI は全 job が success（`Merge gate` success）で、helper 経由で squash merge した（2026-10-24）。
- 後続: L3 の owner の所感 2 件（TD-433）は `docs/Plans.md` の `## 導入の前の関門` と `docs/backlog.md` へ、closure 3 回目の P3・packet との食い違い (1)〜(4)・Windows の `cargo test --lib` が起動しない観測は `docs/backlog.md` へ送った。backlog の「棚卸しの数の置き場と在庫数の求め直しの runtime」の (b) は b1 の完了と b2 に分け、「棚卸しの runtime (b) で `total_cost` を wire・表示まで正確に扱う」は閉じた（上の「起票時に起草役が決めたこと」の 9）。b2 は b1 の merge の後に base を同期して進める。

## Review Response

先行 round の結果・評価（判定・件数・採否・reviewer の意見）はこの節にだけ書く。前半の節と遷移記録には「round N の是正（Review Response 参照）」だけを書く（独立 review は `## Review Response` より前だけを読む）。

（Plan Review の後に書く。）

round 1（`5ada58d3`）: Claude 側 fresh Opus 5.5 = reject（P2 4 / P3 4）、Codex GPT-6 Astra（発注 493）= reject（P2 6）。重なりは 3 件（範囲の型 = Opus F1 / Codex #1、L3 の台本 = Opus F3 / Codex #5、漏れた既存 test = Opus F4 / Codex #4）。Coordinator が採否を決め、全件を採用した（Opus F2 の JAN の経路と Codex #3 の取消の入口は現物で確かめた）。是正は起草役の `9adc7047`（同型の穴の洗い出しで 6 か所を足した）と `097d319c`（Matrix の記号の重なりの付け直しと TD-380 の反映）。

- P2: 要再確認の範囲・単位を運ぶ型（`CountRecoveryTarget` に `stock_unit`・`stock`・範囲）、商品の修正では JAN を変えられない（J2 は在庫連動の開始だけ、JAN の向きは一括 import の上書き J3、L3-12 を一括 import の preview に）、L3 の台本を状態遷移の表と fixture の表に（生成は S9 の test、L3 の日に Coordinator が作る）、vZ・内部型で red になる既存 test（Matrix T10〜T12、予定 path 138）、確定の JAN の再照合のための preview の cache の契約（32）、過去の取込みの記録からの取消は正本に入口が無いので Non-scope（L3-10 は結果の画面の取消。TD-380 で関門の項目）、vZ の作り直しの後の失敗（Matrix M8・M9）。
- P3: Z004 の確定は `insert_movement` → `recompute_stock`、M4 の NULL・`phase = 1` の拒否、`settlement_pending` の回復先の hint、予定 path と数え方と AC6 の rg。
- 既知: Matrix の Z004 の行の記号 Z1〜Z16 と packet の「横断の約束」の Z1〜Z16 は別の記号系（起票の時点から。Matrix の冒頭に注記）。

round 2（`e16b9eda`）: Claude 側 fresh Opus 5.5 = approve（P3 10）、Codex GPT-6 Astra（発注 497）= reject（P2 4 / P3 2）。重なりは 2 件（S9 の file の数 = Opus #9 / Codex #5、観測できない mutant C2・C5 = Opus #4 / Codex #6）と同じ筋の 1 件（外す出力の assert の移し方 = Opus #2 / Codex #3）。Coordinator が採否を決め、全件を採用した（Codex の P2 4 件と Opus #1・#7・#8 は現物で確かめた）。是正は起草役の `1688ccb9`（同型の穴の洗い出しで、完了を通す test の fixture、X1〜X3 の trace、`key_movement` の fixture、要再確認の一覧の再取得、W3・M7・L12・U15 の観測の手段を足した）。Owner Effort Budget の工夫 (3) の旧い fixture の方式は Coordinator が本 commit で直した。

- P2: 一括 import の確定の TX で当たった行は最初の行を名指して確定全体を戻す（既存の `import_row_commit_error` の形、wire は変えない。30・42・60、J3）、Ordinary Operation と L3 の起点を vB の後に作った DB に（v9 の商品入りの DB は vB で止まり 22 §16 の作り直し）、L3 の DB は Coordinator が demo の seed で用意し差し替える、外す出力の assert を assert の単位で分けた（T2 の表、35 の操作ログの新契約）。
- P3: C2・C5・K1 の観測の手段（C2 の TX の外は Final Review の code 読みへ）、S9 は 9 file・T7 は 22 か所、T1 の command を広げ 9 行、invalidation の source を D-052 の SSOT に（U15 は 12 entry）、`zero_uncounted_items` は明細を参照する、同じ CSV の新規の行どうしの JAN、`seed_demo` の `delete_all` に `stock_counts`、AC11 を docs 全体に、共有 JAN の拒否を `recount_required` より先に。

round 3（`c7468bd1`、上限）: Claude 側 fresh Opus 5.5 = reject（P2 1 / P3 3）、Codex GPT-6 Astra（発注 503）= reject（P1 1 / P2 1 / P3 1）。重なりなし。Coordinator が全件を採用した（Codex の P1 は 32 の順 2 の表と「取り消した取込みは比べる先にしない」の文、Opus の P2 は商品フォームの `useUnsavedChangesWarning` を現物で確かめた）。round 天井のため disposition は「同型指摘の一括是正」。是正は起草役の `8655cb64`。

- P1: 唯一の取込みを取り消した後に、別のレジ・番号の戻った精算が fresh な計数のカーソルの内と扱われる → 32 の順 2 は取り消した取込みとも machine_no を比べ（ADR D4）、fresh な計数の n は n の active な取込みがあるときだけ使う（ADR D3）。順 5 は active だけのまま（取り消した誤った版の精算日との偽の拒否を避ける）。Matrix L13・Z22・Z23。
- P2: 未来の日付の精算と入出庫が完了の評価に入り、後の比較で外れて偽の差 → 完了の後の比較の範囲をカーソル（精算・movement・計数）と完了の前の記録で決める（35・ADR D6・D9。`stocktakes` に `cutoff_movement_id` を足し vZ の列は 7 つ。起草役が原則を入出庫にも当てたのを Coordinator が採った）。Matrix R2 (a)〜(c)。商品フォームから棚卸しへの導線は離脱の確認（UI-USW-D1）を通り、戻ったフォームで在庫連動を付け直す（51・73、L3-4、U9・U12）。
- P3: AC11 に撤去した名前の docs の検索（除外に decision-log・backlog）、M1 の列を名指し、AC2 は comment の行を除く、K8・T2 の開始の操作ログの detail の key。

closure（`995c288b`）: Claude 側 fresh Opus 5.5 = approve（P1 0 / P2 0 / P3 4）。Codex GPT-6 Astra（発注 508）は利用上限で途中で終わり結果が無い。owner の決定（repo 外の回答台帳 TD-389）で §3.3 に従い Claude 側の closure で plan-approved へ進め、Codex が戻ったら 508 を Plan Commit の版に当てて流し、指摘は Gated Amendment で扱う。Coordinator が P3 4 件を全部採用し、plan-gate のまま起草役が直した（`5c45d253`・`aba17e2e`）。

- N1（owner の決定 TD-388）: 32 の順 2 は取り消した取込みとも machine_no を比べたまま、比べる先が取り消した取込みだけのときは拒否の文でその取込み（取込みID・精算回数・取込み日・レジ番号）を名指す。照合の基準を入れ替える操作は作らない（残りの危険と ADR の Revisit）。ADR D4・D7、24 の照合先の読取り、Matrix Z2・Z22。
- N2: 完了の後の比較の範囲の原則から計数を除く（完了の後に保存した計数は次の観測）。ADR D6、35、tracking、Matrix R2 (d)。
- N3: Matrix の K2 に取消と取り込み直しの列（完了の (iii) の n を n の active な取込みがあるときだけ使うことを観測する）。発注の初めの列は (iii) でなく (i) が出て観測にならず、起草役が止めて代案を出した（MANUAL §5.6）。
- N4: 24 の読取りを「新設する」と 1 通りに（取り消したものを含めて探し、active を優先する照合先を返す）。

closure の Codex 側（発注 508、`aba17e2e`。owner の決定 TD-389 で plan-approved の後に流した）: reject（P1 1 / P2 0 / P3 0）。round 3 の P1（取消の後の照合と fresh なカーソル）の残り: 取り消した取込みと同じ精算回数を、レジの初期化の後の新しい系列が使うと、fresh な計数の判定の n が生き返り、計数の後の売上を計数に入っていると扱って黙ってずれる。本 packet の「残りの危険」は n 回の精算の後の話と書いていたが、n = 1 なら初期化の後の最初の精算で起きる。Coordinator が ADR D3 の判定表の下の項と 32 の順 2〜5 で確かめて受理した。read-only の起草役（Opus 5.5）が直し方を 3 案出し、案 1（`stock_counts.seen_import_id` と `stocktakes.cutoff_import_id`。判定の n は「c 以下の今 active な取込みの最大の精算回数」を毎回求める）を採った。owner の判断 (i) は A（TD-400）、(ii)（順 2 の照合を active だけに戻す）は行わない（TD-388 のまま）。Gated Amendment `5b532940`。同じ回に、実装の 1 段目で Matrix の T の表に無いのに期待が旧い契約のままだった test 4 群を、packet の契約に合わせて直し、T の表に行を足した（AGENTS.md の「誤ったテストは設計正本との不一致と理由を示して修正する」）。

Final Review と closeout（2026-10-24）: Final Review（Minimum 2）は broad 2 本（Claude 側 Fable 5〈owner の決定 2026-10-24 で当面 Fable 5.1 でなく 5。Workflow State の Final Reviewer の行は起票の時点の座組の記録として残す〉+ Codex）と closure 3 回で、3 回目は Claude 側 Fable 5 = approve（P3 2: 「今の数を入れる」の欄で route の `product` が settle なしで外れると保存の成功の後も欄が閉じない。closure 2 回目で届かないとした見送りを、3 回目で届くと分かった）。P3 は `docs/backlog.md` へ送った。L3 は owner が最後の head で L3-1〜L3-13 を全段 PASS とし、同じ head で r4 を承認した（repo 外の回答台帳 TD-431）。L3 の台本の読み替え 3 つは Implementation Results の closeout の段で、(a) の 3 行は本 closeout で今の data の folder の名に直した。Ready 後の hosted CI は緑（`Merge gate` success）で、helper 経由で squash merge した（2026-10-24）。PR の Follow-up の packet との食い違い (1)〜(4) は Coordinator の裁定待ちのまま `docs/backlog.md` の 1 項目にした。
