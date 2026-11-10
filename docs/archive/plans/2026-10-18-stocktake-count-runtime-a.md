# Plan Packet: 棚卸しの数の置き場と在庫数の求め直しの runtime (a)（並びの鍵・計数の表・写しの列、求め直し、入出庫と商品登録の切替、整合性の突合）

wave に属さない単独の lane（2026-10-17〜18 に Coordinator が同時に起票した 7 本の lane の 1 本。owner の決定は repo 外の回答台帳 TD-304「動かせるだけ動かす」・TD-295「並列にできるものは並列に」）。D-118（[ADR](../../adr/2026-10-16-stocktake-count-placement.md)、design lane = PR #167）の runtime の最初の配布の単位 (a)（ADR SPEC-STK-CNT-D12）を実装する。(b)（計数の保存・完了・Z004 の確定と取消・停止 ADR の 5 つの入口の撤去ほか）は本 lane の外。branch は `agent/stocktake-count-runtime-a`（main `f5d7aaf4` から）。新しい決定は D-123（owner の決定 TD-311 を正本へ置いた。下の「owner の判断事項」）。

**規模と手法（1 lane・1 PR のまま。横断の約束は下の 1 つの表に置き、実装は S1〜S7 の順に積む）**: 2026-10-18 に本 worktree（HEAD `f5d7aaf4`）で数えた。

- 触る既存の Rust は 40 file（`rg -l 'NewMovement|insert_movement\(|apply_stock_change\(|update_stock_quantity\(|INSERT INTO inventory_movements|sum_movements_(by|for)_product|IntegrityMismatch|StockAdjustment|find_all_stock_quantities|stock_quantity: (Some|None)|create_test_product(_with_jan|_with_unit)?\(|create_plu_exported_product\(|apply_void_stock_corrections|pub mod (integrity_service|inventory_service)|31-biz-inventory-service.md' src-tauri/src src-tauri/tests | wc -l` → `40`。helper を使うだけで変えない file を含む）に、版の期待を書き換える `src-tauri/src/db/schema_v2.rs` の 1 と新設 2（`biz/stock_ledger.rs`・`db/schema_v{N}.rs`）。TS は 5 file（`src/lib/bindings.ts`〈生成物〉、`src/features/integrity-check/IntegrityCheckPage.tsx` とその test、`src/features/operation-logs/OperationLogsPage.tsx` とその test）。
- migration 1（vL、[22 §19](../../function-design/22-mnt-migration.md)。`inventory_movements` の作り直しと `products` の列の追加と `stock_counts` の新設）。wire の型の変更 4（`IntegrityMismatch`・`StockAdjustment` の形の変更と、新しい `StockCopy`・`IntegrityField`）。command の名前と引数は変えない。
- 横断の約束（範囲・0・失敗の時点と順序・単位や種類の混在）がかかる経路は 8: 入出庫 4 業務の `create_*`、商品登録、商品 CSV の commit の新規行、整合性の補正、migration。ほかに test だけが呼ぶ旧本体 3（Z004 の確定・取消、棚卸しの確定）が同じ書込みの関数を通る。
- 分け方の判断: 分ける条件（前の段だけが merge された main で (1) 店の画面の値が今と変わらない (2) 写しを書く経路と整合性の検査の相手が食い違わない (3) 全 gate が緑）を現物に当てた。S1〜S5（migration・求め直し・書く側の切替）だけを先に入れる切れ目は、そのままでは compile が通らない（S2 で消す和の関数を整合性の検査〈`integrity_service.rs:70`・`:166`〉が使う）。通すには和の関数の削除を後の段へ回し、整合性の補正の書込み（`:193`）も前の段で `recompute_stock` に寄せる必要があり、それでも (2) は `IntegrityMismatch.movements_sum` が「記録から求め直した値」を和の名前で運ぶ所で破れる。S6（整合性の DTO と画面）だけを後に回す切れ目は、Plan Review・Final Review・owner の Ready と merge がもう 1 周増え、R4 の承認と L3 も分かれる。どの切れ目でも横断の約束は前の段から減らないので分けない。分ける場合の切れ目は S5 と S6 の間（Coordinator の判断。そのときは上の 2 つ〈和の関数の削除を後の段へ、補正の書込みを前の段へ〉を入れ直す）。
- 1 PR の大きさの目安: 書き換える test の多くは機械的な追随（`NewMovement` の field、raw の `INSERT INTO inventory_movements` に `business_date` を足す、商品を作る test helper が初期在庫の movement を作る）で、振舞いの判断を含む新しい test は S2・S3・S6 に集まる。既存の test の分類（書き換える / 変えない / 期待を変える）と数える command は [Matrix](test-matrices/2026-10-18-stocktake-count-runtime-a.md) の「既存の test の移し方」（T1〜T6）の 1 か所に置き、S1・S2・S4・S6 はそこを引く。

## Workflow State

- Phase: archive
- Risk: R4
- Plan Commit: 1928f836d2bb04f0fe493b27eaebf28aa85caf00
- Amendments: none
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（subagent_type: writer）
- Plan Reviewer: fresh Opus 5.5 + Codex（model は発注時に決める）。座組表どおり
- Final Reviewer: Fable 5.1（Claude 側、R3 以上）+ Codex（GPT-6.1 Sol 既定）。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Review Minimum: 2
- Human Gate: ready,merge,manual,r4
- Branch: agent/stocktake-count-runtime-a

遷移の記録:

1. kickoff → spec-check（2026-10-18、起草役）: Coordinator の発注（D-118 の runtime の (a)）を Scope にし、規模を数えて 1 lane・1 PR のままと決め、Risk を R4 と記録した（下の Risk）。
2. spec-check → design（2026-10-18、起草役）: 対象の設計正本は ADR SPEC-STK-CNT-D1〜D3・D9・D10・D12、tracking・master・transaction の「計数の置き場と在庫の求め直しの契約」、[21](../../function-design/21-io-inventory-repo.md)・[30](../../function-design/30-biz-product-service.md)・[31](../../function-design/31-biz-inventory-service.md)・[36](../../function-design/36-biz-integrity-check.md)・[75](../../function-design/75-ui-integrity-check.md) の同名の節、[22 §16](../../function-design/22-mnt-migration.md)。設計の更新が要る点が 7 つあった（下の Design Readiness の「必要な設計成果物」）。
3. design → plan-draft（2026-10-18、起草役、本 commit）: 上の 7 つを本 packet と同じ commit で正本に置いた（D-123 と D-118・D-119 の Superseded in part の行、22 §16 の順の段と §19、ADR D10・D12、tracking・pos・ARCHITECTURE・DB_DESIGN・mnt-task-specs の作り直しの文、21・31・36・75 の (a) の細部、backlog の 2 項目、Plans.md の順の 1 文）。owner に聞く設計の問いは残っていない（下の「owner の判断事項」）。plan-gate へは Coordinator が進める。
4. plan-draft → plan-gate（2026-10-18、Coordinator）: packet・Matrix と正本は起草役の commit（`82d6b030`）にある。Coordinator は Risk（R4 の理由は作り直しでなく、行のある `inventory_movements` の作り直しと写しの書換え）・Goal と失敗定義・Ordinary Operation・owner の判断事項・横断の約束 Y1〜Y8・Contract Probe P1〜P6・Data Safety・残りの危険を読み、TD-311 の写しが D-123 として置かれ D-118・D-119 に Superseded in part の行があること、作り直しが「消さずに移す」で閉じていること、止まる DB で原因と直し方を名指しすること（TD-310）、並走の lane の節（22 §17・§18、30）を書き換えていないことを確かめた。(a) と (b) の境の細部（Z004 の phase の CHECK ほかを (b) へ）と、1 lane・1 PR の判断は起草役の判断のまま Plan Review の判定に出す。Plan Review round 1 へ出す。
5. plan-gate（round 1 の是正、2026-10-18、Coordinator）: Plan Review round 1（Claude 側 fresh Opus 5.5・Codex GPT-6 Astra）の是正（Review Response 参照）。起草役が `3523401a` で直し、Coordinator が差分（Matrix の「既存の test の移し方」T1〜T6 と、S1・S2・S4・S6 がそれを引く形、22 §19 の手順 2 の暦日の検査と手順 2・8 の作り直しの案内、Contract Probe P7、S1〜S6 の検証の単位、L3-1 の控えと合格基準、`per_page`、数の集計、ADR の Context の注記）を読んだ。Scope の範囲と Risk は変えず、その場で直したので plan-gate に留める。Plan Review round 2 へ出す。
6. plan-gate（round 2 の是正、2026-10-18、Coordinator）: Plan Review round 2（Claude 側 fresh Opus 5.5・Codex GPT-6 Astra）の是正（Review Response 参照）。起草役が `c3f98aa6` で直し、Coordinator が差分（31 の業務日の形と暦の検査を書く側の 1 か所にしたこと、22 §19 の手順 2 の拒む 3 つの条件と手順 8・9、21・31 の新しい関数のシグネチャ、Contract Probe P6・P7 の実測、Matrix の M4・M6・W1・W5・W7・T2）を読んだ。Scope と Risk は変えず、その場で直したので plan-gate に留める。Plan Review round 3（天井）へ出す。
7. plan-gate（round 3 の是正、2026-10-18、Coordinator）: Plan Review round 3（Claude 側 fresh Opus 5.5・Codex GPT-6 Astra）の是正（Review Response 参照）。round 3 は round の天井で、owner が「直して、追加の round なしで plan-approved」と決めた（repo 外の回答台帳 TD-319、介入 1 回目 / 予算 9 回）。起草役が `390e29f5` で直し、Coordinator が差分（S6 の補正の操作ログを 75 の 4 項目の前後に揃えたこと、U6、demo の seed の写しを和へ揃えること、S7 の DB_DESIGN の節、L3-1 の (iii)、T5・T6 の局所変数名、backlog の (b) への申し送り）を読んだ。
8. plan-gate → plan-approved（2026-10-18、Coordinator、本 commit）: 独立の Plan Reviewer（Writer と別の context）のうち Claude 側が round 3 で P1・P2 が無いと報告し、Codex の P2 1 件は owner の決定（repo 外の回答台帳 TD-319）どおり直した（Review Response 参照）。Plan Commit は round 3 の是正と記録の後の版。実装の commit はまだ無い。R4 の承認（Human Gate `r4`）は実装に入る前に owner に諮る。
9. plan-approved → implementing（2026-10-18、Coordinator、本 commit。state だけ）: owner が実装の前の R4 の承認をした（repo 外の回答台帳 TD-321「承認して実装へ」、介入 2 回目 / 予算 9 回。対象は上の Risk の節の vL の作り直し・業務日の埋め・`sqlite_sequence` の復元・写しの書換え・止まる DB・失敗の巻き戻し。最後の head での `r4` の record は L3・Ready の依頼と同じ 1 回で諮る）。第 1 便（#170・#173・#174・#175・#177）の後の main `44f1e666` を merge した（`485582de`。衝突は docs だけで、22 §18・§19 と D-120〜D-123 を番号順に並べ、MSI の手順書と 22 §18 が引く 22 §16 の段落の名前を「DB の作り直しと順」に合わせた。code の差分は無い）。発注の直前に AC の baseline（上の 13 本の command）を逐語で測り直し、上の baseline と一致した。Writer（Opus 5.5 subagent、`subagent_type: writer`）へ 2 段で発注する: 1 段目 = S1〜S6 の Rust と bindings の再生成、2 段目 = S6 の画面・S7・traceability・PR。

## Owner Effort Budget

- 介入回数上限: 9（repo 外の回答台帳 TD-296 の 5a。既定の 6 を上げた理由: R4 の承認と L3 が加わり、owner は 2026-10-20 から手が使いにくく、L3 は Draft のまま溜めて後でまとめる）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 9 | 5（closeout 時点。Plan Review round 3 の天井の判断 1〈TD-319〉、実装の前の R4 の承認 1〈TD-321〉、L3 と最後の head の `r4` 1〈TD-323〉、Ready 1、merge 1〈Ready・merge は TD-325〉。TD-311 は本 lane の前の判断で数えない） | 0 | 4 | 9 = 5 + 0 + 4 |

plan-approved は Coordinator が進める（TD-296 の 4a）。介入のたびに、やろうとしていることが全体を顧みない対症療法になっていないかを考え直す（TD-296 の 5a）。L3 が 2 round 目に入るか、Plan Review が round 天井に達するときは、その判断と上限の改定を同じ 1 回で諮る。

既定値・数え方・上限に届くときの扱いは `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R4

Reason:
migration vL が行のある `inventory_movements` を作り直し（新しい表へ全行を写して `DROP TABLE` し、名前を変える）、業務日を参照先から埋め、写しと和の違う商品の `stock_quantity` を書き換える（[22 §19](../../function-design/22-mnt-migration.md)）。Risk Tiers の「Destructive data lifecycle」に当たる（先例: 単位の lane の v8 の `products` の作り直しを R4 とした。`docs/archive/plans/2026-10-11-unit-runtime.md`）。棚卸しか Z004 の記録のある開発・試験 DB は vL で止まり作り直すが、作り直しは owner がリリースの前まで包括で承認し（TD-311、D-123）、古い file を消さずに移すので、それ自体は不可逆な local の片付けにならない（R4 の理由は作り直しではなく、行のある DB に当たる vL の表の作り直しと写しの書換え）。Tauri command の DTO（整合性）・generated bindings・operator の画面（整合性検証・操作ログ）も変える（R3 の要素）。Human Gate は `ready,merge,manual,r4`（owner が migration と回復の実装を承認し、Windows の実機で L3-1・L3-2 を確かめる）、Final Review Minimum 2（R4 の Double Audit）。本番の DB は無い（`docs/project-memory.md`）。実店舗データの露出は無い。

required gate の green / red: gate の定義は変えない。実装は bindings と traceability の再生成、`design_compliance_test.rs` の対応表の更新を伴い、どれかを落とすと既存の required gate（Rust・bindings・traceability）が red になる（下の Registration / Generation Obligations）。Scope の予定 path を `scripts/ci/classify-changes.sh` の workflow の一覧（AGENTS・CLAUDE・DEV_WORKFLOW・MANUAL・code_review・project-profile・agent-guidance・templates・`.agents`・`.claude/{rules,commands,skills}`・PR template、`.github/workflows/*`・`scripts/ci/*`・`scripts/tests/*`・gate の `scripts/*.sh`・`scripts/pr-gate.py`）に当てると当たらない。Final Review Minimum 2 は R4 による。

## Goal

Goal Invariant:

### 最小完了条件

- 在庫を動かす全ての書込み（入庫・返品交換〈レジ未処理〉・手動販売・廃棄・商品登録の初期在庫・商品 CSV の新規行の初期在庫・整合性の補正）が、`products.stock_quantity` を「保存した数 + 差分」ではなく記録から求め直して書き、計数が無い間は在庫と `stock_after` の値が今と同じ（ADR SPEC-STK-CNT-D2・D12 の (a)）。
- 新しい版を、棚卸しと Z004 の記録の無い DB で起動すると vL が当たり、既存の movement に業務日が付き、写しの値は和と同じになる（違った商品は直して操作ログと整合性の画面で知らせる）。棚卸しか Z004 の記録のある開発・試験 DB では、何も変えずに止まり、起動の画面が原因と直し方（作り直しの手順）を名指しする（ADR SPEC-STK-CNT-D10、TD-310）。
- 整合性検証が「記録から求め直した値」と写しの 4 項目（数・要再確認・範囲）を比べ、違う項目を文字で出し、補正は求め直しの結果を書く（[36](../../function-design/36-biz-integrity-check.md)・[75](../../function-design/75-ui-integrity-check.md)）。
- 計数（`stock_counts`）があるときの求め直しが、計数・入出庫・初期在庫について ADR D3 の窓の判定表どおりに効く計数・範囲・要再確認の理由 2・3 を返す（(b) が計数の保存を入れたときに、その上に載る土台。test の fixture で確かめる）。

### 失敗定義

- 計数の無い DB で、どれかの書込みの後の `stock_quantity`・`stock_after`、または整合性の検査の結果が今と違う。
- `stock_quantity` を求め直し以外の経路で書く production の code が残る（`update_stock_quantity` を呼ぶ所が `recompute_stock` のほかにある、`ProductUpdates.stock_quantity` が残る）。
- vL が行・id・列の値・index を欠く、`sqlite_sequence` を前の値より戻す、TX の中で失敗したのに表・値・版のどれかが変わる、止まる DB で何かを変える、止まったときに原因と直し方を出さない。
- 形の違う日付（`2026-4-7` 等）か実在しない日付（`2026-02-30` 等）が `business_date` に入る経路が残る（窓の判定は文字列の大小で比べる）。vL が既存の記録の実在しない日付を写す。
- 求め直しが `phase = 1` の movement を黙って入出庫として扱う（(a) の間は誤りで止まる契約）。
- 整合性の画面が、数が同じで印・範囲だけ違う商品を「差異なし」と出す。

### 非目的

- 計数の保存（「数を保存」「入れ直す」「0 にする」）・要再確認の一覧・棚卸しの完了・評価額の差・停止 ADR の 5 つの入口の撤去（(b)）。
- Z004 の確定と取消の新しい経路、`csv_imports` の machine_no・settlement_no、`sale_records` の `stock_linked`、Z004 と phase を結ぶ CHECK、求め直しの「精算 k」の行と要再確認の理由 1（(b)。ADR D12 の境の細部）。
- D4 の共有 JAN の不変条件と移行の前からの違反の知らせ、D8 の在庫連動を始める検査、商品の結果・在庫推移の要再確認と範囲の表示、`MovementRecord` の並びの鍵（(b)）。
- 旧本体（`legacy_*`）・D-109 の時点証拠の `#[cfg(test)]` の schema と code の撤去、診断 2 本の PASS 化（(b)）。
- 原価の 1/100 円（原価の lane、vB）。単位の語の直し（並走の lane）。JAN の別名（並走の lane）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

設計を含む変更の実装なので、店の通常の操作列と開発の DB の更新の列を置く。例の数は説明用の合成の値。本 lane の merge の後に下の列が成り立つ。棚卸し（計数と確定）と Z004 の確定・取消は停止中のまま（`src-tauri/src/biz/stocktake_service.rs`・`src-tauri/src/biz/csv_import_service/commit.rs:27`〜`:32`）で、**この文書を完了できる**ことと、**棚卸しと POS の自動在庫連動の通常運用を達成できる**ことは別（後者は (b) の後）。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 棚卸しと Z004 の記録の無い DB（入庫・手動販売の記録あり） | 新しい版を起動する | 起動できる。写しと記録の和が同じ商品の在庫は起動の前と同じ（違った商品は下の「写しと和の違う商品があった」の行）。各 movement に業務日が付く | 起動できる | 22 §19 の手順 1〜10、Contract Probe P1〜P4・P7 |
| 棚卸しか Z004 の取込みの記録のある開発・試験 DB（owner の L3 用の DB がこれに当たるかは未確認） | 新しい版を起動する | 起動が止まり、「データベースの準備に失敗」の文に「棚卸しか商品別売上（Z004）の取込みの記録がある DB はこの版へ移せません。開発・試験 DB は作り直してください（22 §16 の作り直しの手順）: {表名}」が添えられる。DB は変わらない | 作り直しの手順（アプリの data の folder を丸ごと消さずに移す）の後に起動すると、空の DB で開く | D-123、22 §16 |
| 空の DB（作り直しの後） | 商品登録で初期在庫 `5` の商品を登録する | 在庫 5。在庫変動の履歴に初期在庫 `+5`・変動後 `5` | 保存できる | 30 の新契約 |
| 同じ商品 | 入庫（入庫日 = 今日）で `3`、手動販売で `1`、廃棄で `1`、返品交換（レジ未処理）で `1` を順に保存する | 在庫 8 → 7 → 6 → 7。履歴の変動後の値も同じ順 | 保存できる | 31 の新契約（今と同じ値） |
| 商品 CSV（新規行、`初期在庫` `4`） | 取り込む | その商品の在庫 4、履歴に初期在庫 `+4` | 取り込める | 30 の新契約 |
| 整合性検証の画面 | 「チェック」を押す | 「差異なし」（記録から求め直した在庫と、保存している在庫が全商品で同じ） | — | 36・75 の新契約 |
| 写しと和の違う商品があった開発・試験 DB を vL で移した後 | 整合性検証の画面を開く | 画面の上に「更新のときに在庫数を記録から求め直し、{N}商品の在庫数を直しました（{日時}）。直した前後は操作ログで見られます。」と操作ログへの導線 | — | 22 §19 の手順 8、75 の注記 |

失敗と再試行: header の日付が `YYYY-MM-DD` の形の実在する日付でない要求（画面の日付欄は今も ISO の形で送るので、command を直接呼んだときだけ起きる）は、movement を作らないレジ処理済みの返品・交換も含めて、TX を開く前に `ValidationFailed` で何も書かない（検査の中身は [31](../../function-design/31-biz-inventory-service.md) の「業務日の形と暦の検査」）。在庫の上限（絶対値 `999999999`）を越える結果は `recompute_stock` が写しを書く前に `ValidationFailed`（`在庫数が大きすぎます`）にし、TX 全体が戻る（今と同じ文）。vL が止まった DB は何も変わらないので、作り直すまで旧版でそのまま開ける（その起動の最初の migration で止まったとき）。

Plan Review は、この列が「正常な条件で目的を達成できるか」と「危険な結果を出さないか」を別々に答える（`docs/DEV_WORKFLOW.md` Review Rules）。

## owner の判断事項

無い。製品の振舞いの判断は D-118（TD-261・TD-263〜TD-268・TD-296・TD-298）と D-123（TD-311）で決まり、正本にある。本 lane で owner が答えるのは Human Gate（R4 の承認、L3-1・L3-2、Ready、merge）だけ。

発注書の要旨と台帳の原文の突合: TD-311 の原文（repo 外の回答台帳 `ledger-parts/today-2026-10-16.md` の TD-311 の行）は発注書の要旨と一致した（作り直しの回数を問わない・schema の拡張可・リリースの前だけ・包括の承認・古い file は消さずに移す）。初めて入れる版の条件（D-123 (4)）は発注書の文で、原文は MSI の lane の Plan Review の流れで出た決定と書く（矛盾しない）。

起票時に起草役が決めたこと（設計の値を変えない配布の割当てと細部。Coordinator が覆せる。正本へは本 commit で置いた）:

- D-123 を新しい決定にした（D-119 に置換えの行を足すだけにしなかった）理由: TD-311 は D-119 (8) の順だけでなく、D-118 (9) の「作り直しは別の承認」と、店の PC へ初めて入れる版の条件（MSI の lane も引く）を同時に決めており、1 つの決定の置換えの行では 3 つの決定から引けない。D-118・D-119 には Superseded in part の行を足した。
- (a) と (b) の境の細部（ADR D12 に足した）: Z004 の movement を `phase = 1` にする CHECK と `csv_imports`・`sale_records` の列、求め直しの「精算 k」の行と理由 1 を (b) へ回した。理由: 停止中の旧本体（test だけが呼ぶ Z004 の確定と取消。`src-tauri/src/biz/csv_import_service/commit.rs:40`・`rollback.rs:26`）の fixture（`src-tauri/src/biz/csv_import_service/test_support.rs:31`〜`:57` の `make_z004_bytes`）は精算回数の行を持たず、(a) で Z004 と phase を結ぶ CHECK を入れると、旧本体の回帰 test（`commit_tests.rs` 17・`rollback_tests.rs` 15 の呼出し。`rg -c 'legacy_commit_csv_import|legacy_rollback_csv_import'`）が (b) で旧本体ごと外れるまで書けない。(a) の間、旧本体は Z004 の movement を `phase = 0`・業務日 = 精算日で書く（test だけ。求め直しは計数が無い間は和と同じ）。発注書は (a) に「並びの鍵の列」を挙げているので、3 列と phase・settlement_no の対の CHECK は (a) に置いた。
- 共有 JAN の不変条件（D4）・在庫連動を始める検査（D8）・商品の結果の範囲の表示・`MovementRecord` の鍵は、発注書と ADR D12 の (a) の列挙に無いので (b) に置いた（どれも計数か Z004 の新しい経路が入って初めて効く）。
- `NewMovement` は `phase` の field を持たず、`settlement_no` から決める（21 に足した）。`stock_after` は呼出しごとに、その呼出しが入れた movement だけを書く（31 に足した。今と同じ値を保つため）。業務日の形と暦の検査は、書く側を 31 の「業務日の形と暦の検査」、vL を 22 §19 の手順 2 の各 1 か所に置いた（形〈長さ 10 の ASCII の `YYYY-MM-DD`〉を先に見て暦を見る。Contract Probe P6・P7）。
- 計数の保存の IO（`insert_stock_count`）は (a) では作らない（production の呼出しが無く dead code になる）。(a) の test は `stock_counts` へ test の helper から直に入れる。
- 整合性の型の形（`StockCopy`・`IntegrityField`・`before` / `after`）と、移行の知らせの出し方を 36・75 に足した（36 の旧文は `copy_*`・`derived_*` の名前の型までは決めていなかった）。

店主に新しく聞く必要がある事項: 無い（`docs/backlog.md` の「入庫・返品の記録を品物が届く前に入力することはあるか」は (b) の計数の窓に効く問いで、(a) の値は計数が無い間は今と同じ）。

## 横断の約束（DB・wire に出す値ごとに、どこが保証するか）

行番号は 2026-10-18 の HEAD `f5d7aaf4` の現物。Contract Ledger と Matrix はこの表の # を引く。

| # | 値（DB の列 / wire） | 範囲と、保証する所 | 0 と符号 | 失敗の時点と順序 | 単位・種類の混在 |
|---|---|---|---|---|---|
| Y1 | 業務日（`inventory_movements` の `business_date`） | `YYYY-MM-DD` の形の実在する日付。形と暦の検査は、書く側（4 業務の `create_*` と `sales_service`）が [31](../../function-design/31-biz-inventory-service.md) の「業務日の形と暦の検査」、vL が [22 §19](../../function-design/22-mnt-migration.md) の手順 2 の各 1 か所（中身はそこ）。DB の CHECK は形だけ（tracking）。初期在庫は PC の今日（`chrono::Local`） | — | 4 業務: DB を見ない検証の段（冪等の再送の判定〈入庫は `receiving.rs:100`〉と TX〈`:168`〉より前。返品交換は `register_processed` で分かれる前。31）で `ValidationFailed`、何も書かない。vL: 何かを変える前に止まり、原因と直し方（作り直し）を出す | 単位に依らない |
| Y2 | 写しの数（`products.stock_quantity`） | 絶対値 `999999999`（`constants.rs` の `MAX_STOCK_QUANTITY`）。`recompute_stock` が書く直前の 1 か所（今の `check_stock_range`〈`common.rs:52`〜`:60`〉を使う）。vL の手順 8 も同じ上限 | 0 と負（INV-3）を受ける | 書く前に `ValidationFailed`（`在庫数が大きすぎます`）、呼出し元の TX 全体が戻る。vL は止まり、原因と直し方（作り直し）を出す | 個数も長さも同じ上限（長さは cm の整数） |
| Y3 | 要再確認の印と範囲（`stock_recheck`・`stock_range_low`・`stock_range_high`） | 範囲の両端も Y2 の上限。DB の CHECK（印 0 なら両端 NULL、1 なら両端 NOT NULL で low ≤ high） | 端が 0 や負でも受ける | Y2 と同じ所で両端に当てる | 計数が無い間は常に 0・NULL |
| Y4 | 変動後（`inventory_movements.stock_after`） | その呼出しの `recompute_stock` の写しの値（Y2 の範囲） | 今と同じ | movement を仮の値で INSERT → 求め直し → その movement の id で UPDATE。どれかの失敗で TX 全体が戻る | — |
| Y5 | 計数の値（`stock_counts.count_value`） | DB の CHECK で 0 以上。上限の検査は保存の BIZ（(b)）。(a) の求め直しは計数を土台にした結果に Y2 を当てる | 0 は有効な計数（「0 にする」） | (a) には保存の経路が無い | 在庫の数量の単位（長さは cm） |
| Y6 | 整合性の結果（wire の `IntegrityMismatch.copy`・`derived`・`differing_fields`、`StockAdjustment.before`・`after`、操作ログの detail_json） | 各数は Y2 の範囲（読取りは範囲で拒まない）。`differing_fields` は `quantity` / `recheck` / `range` の重複なし・この順 | 数が同じで印・範囲が違えば一覧に出す | 補正は TX の中で求め直しを書く。失敗で TX 全体が戻る | `stock_unit` を運び、画面が単位で出す（今と同じ） |
| Y7 | 移行の補正の記録（`operation_logs` の `integrity_fix`、`trigger: migration`） | vL の TX の中で 1 行 | 違う商品が 0 なら行を書かない | vL の TX と一緒に戻る | Y6 と同じ形 |
| Y8 | `phase` と `settlement_no` | DB の CHECK（`(phase = 1) = (settlement_no IS NOT NULL)`）、`insert_movement` が `settlement_no` から `phase` を決める。(a) の書込みは全部 `None` | — | 求め直しは `phase = 1` の行を読んだら `DbError::QueryFailed` で止まる | — |

## Scope

本 lane が変える file と目的（行番号は 2026-10-18 の HEAD `f5d7aaf4`）。実装は S1 → S7 の順に積むが、S1〜S6 は 1 つの compile・回帰の検証の単位にし、途中の段では全 test の成功を求めない。理由: S1 で `business_date` が必須の列になると S2 で INSERT を直すまで movement を書く test が落ち、S2 で消す和の関数は S6 で入れ替えるまで整合性の検査が使い、S5 の「`update_stock_quantity` の呼出しが 1 つ」は S6 で整合性の補正を寄せて初めて成り立つ。途中の段は Test Plan の targeted tests のうち、その段の module の test で進み具合を見る。S6 の終わりで AC1〜AC10 を全部通し、S7（docs）の後に AC7 の gate をもう一度通す。既存の test をどう移すかは Matrix の「既存の test の移し方」（T1〜T6）を引く。

### 設計正本（本 packet の commit で済み）

D-123（`docs/decision-log.md` の末尾）と D-118・D-119 の Superseded in part の行、[22 §16](../../function-design/22-mnt-migration.md) の「DB の作り直しと順」と §19（MNT-03-D17）、ADR の SPEC-STK-CNT-D10 の作り直しの文と D12 の境の細部、tracking の業務日の CHECK・phase の CHECK の配布・移行の段、`pos-tables.md`・`ARCHITECTURE.md`・`DB_DESIGN.md`・`architecture/mnt-task-specs.md` の作り直しの文、[21](../../function-design/21-io-inventory-repo.md) の `NewMovement`・`update_movement_stock_after`、[31](../../function-design/31-biz-inventory-service.md) の `stock_after`・業務日の検査・(a) の求め直しの範囲、[36](../../function-design/36-biz-integrity-check.md) の型の形と移行の知らせ、[75](../../function-design/75-ui-integrity-check.md) の移行の知らせ、`docs/backlog.md` の「棚卸しの数の置き場と在庫数の求め直しの runtime」と「原価の lane（C）」、`docs/Plans.md` の runtime の順の 1 文。

### S1 migration vL

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| migration vL | 22 §19 の手順 1〜10。関数は `pub(crate)`。COMMIT・ROLLBACK は共有の helper（`db/migration_tx.rs` の `commit_transaction`・`rollback_after_error`）だけ | `src-tauri/src/db/schema_v{N}.rs`（新設。N は merge の時点の次の番号。起票の時点の最大は v8: `db/migration.rs:79`〜`:83`）、`src-tauri/src/db/mod.rs`（module の宣言）、`src-tauri/src/db/migration.rs:12`〜`:19`（use）・`:79`〜`:83` の後（登録） | MNT-03-D1 の構造の test（`migration.rs:379` 付近の `include_str!("schema_v8.rs")` のように migration の file を列挙して共有の helper だけを使うことを確かめる test）の列挙に新しい file を足す。`migration.rs:668` の `expected_tables` に `stock_counts` を足す |
| 既存の migration の test（版の期待・migrate の呼び方・開き直し・列の定義） | Matrix の T1 のとおり（新しい番号へ書き換える 12 行、`migrate_up_to(conn, 8)` に替えて 8 のままの 3 つの呼出し、呼び方も期待も 8 のままの 2 行、期待を変える 2 か所〈`:1627` の開き直し、`:1542` の列の定義〉、変えない `migrate(&conn)` 21 と migration.rs の外の 6）。`migrate_up_to` は test だけの helper で、§16 の vB の test と同じもの（先に入る lane が足す） | `src-tauri/src/db/migration.rs`（test の helper と T1 の test）、`src-tauri/src/db/schema_v2.rs`（版の期待） | 並走の §17・§18 の lane も版の期待の行を書き換える（下の所有表）。着手時に T1 の command を逐語で回し、表と違う所があれば表を直してから進める |
| 新しい test | Matrix の M1〜M9 | `src-tauri/src/db/schema_v{N}.rs` の test | — |

### S2 IO（`inventory_repo`・`product_repo`）

新しい関数（S2 の 3 つと S3 の 2 つ）のシグネチャは 21・31 の ```rust の code block に置いた（本 packet の commit）。`src-tauri/tests/design_compliance_test.rs` の allowlist（`:64`〜`:95`）に足さない。5 つとも crate の外（`src-tauri/tests`・`src/bin`）から呼ばないので `pub(crate)` にする（同じ形の先例: `apply_stock_change` は `pub(crate)` で 31 §12.2 の code block にある）。

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| `NewMovement` | `business_date: String`・`settlement_no: Option<i64>` を足す。`insert_movement` が `business_date`・`phase`（`settlement_no` から）・`settlement_no` を書く | `src-tauri/src/db/inventory_repo.rs:104`〜`:112`（型）・`:123`〜`:143`（INSERT） | 作る所は Matrix の T2（`NewMovement {` の hit 23 = 型の定義 1 + literal 22〈production の code 4・時点証拠の `#[cfg(test)]` の code 2・test 16〉）。production の 4 は 31・30 の業務日、時点証拠の 2 は PC の今日、test の 16 は固定の日付（例 `"2026-04-01"`）と `settlement_no: None` を足すだけ |
| raw の `INSERT INTO inventory_movements`（test・seed・時点証拠の code） | 列の並びに `business_date` を足す（`phase` は NOT NULL で既定値が無いので `0` も足す） | Matrix の T2（hit 32 = `insert_movement` の定義 1〈`:127`〉・demo の seed 2〈`src-tauri/src/seed_demo.rs:398` の初期在庫は `BASE_TIMESTAMP` の日付、`:462` の `sale_auto` は売上ごとの `sale_date`〈`:480`〉を業務日にする。seed の終わりに写しを和へ揃える〉・その他 29。その他 29 のうち書き換えるのは時点証拠の `#[cfg(test)]` の code 1〈`inventory_repo.rs:421`〉と test・test の support 25、変えないのは vL の前の版の DB に入れる `migration.rs` の 3） | `db/test_support.rs:122` の legacy の fixture は `seed_legacy_fixture`（時点証拠の test が vL の後の DB に入れる）なので足す。`sales_repo.rs`・`stocktake_repo.rs` の `reference_type = 'csv_import'`・`'stocktake'` の行も (a) の間は `phase = 0`（Z004 と phase を結ぶ CHECK は (b)） |
| `update_stock_quantity` | 引数を `(conn, product_code, stock: i64, range: Option<(i64, i64)>)` にし、`stock_recheck`（`range.is_some()`）・`stock_range_low`・`stock_range_high` も書く。呼ぶのは `recompute_stock` だけ | `inventory_repo.rs:151`〜`:163` | repo の test `:768`・`:786`・`:807` を新しい引数に。呼出し元の 4 か所（`common.rs:93`・`integrity_service.rs:193`・`csv_import_service/commit.rs:277`・`stocktake_service.rs:518`）は S4・S6 で `recompute_stock` に寄せる |
| `update_movement_stock_after` | 新設。`pub(crate)`、シグネチャは 21 の code block | `inventory_repo.rs` | `apply_stock_change` だけが呼ぶ |
| 求め直しの読取り | `list_stock_counts_for_product`（`stock_counts` の id・値・種類・業務日・settlement_no・seen_movement_id、id の順）と `list_ledger_movements_for_product`（`is_voided = 0` の id・quantity・business_date・phase・settlement_no・`reference_type IS NULL`、id の順）。どちらも `pub(crate)`、シグネチャは 21 の code block | `inventory_repo.rs` | `stock_ledger` だけが呼ぶ |
| `sum_movements_by_product`・`sum_movements_for_product`・`ProductMovementSum` | 消す（呼ぶ所は `integrity_service.rs:70`・`:166` だけ） | `inventory_repo.rs:94`〜`:98`・`:164`〜`:200` | repo の test `:847`〜`:993` の 5 本は、同じ和の意味を `stock_ledger` の test（計数の無い商品で和と同じ）へ移す（Matrix の T6） |
| `ProductUpdates.stock_quantity` | 消す（[20](../../function-design/20-io-product-repo.md) の新契約。production で `Some` を渡す所は無い: `rg -n 'stock_quantity: Some' src-tauri/src` が 0 行） | `src-tauri/src/db/product_repo.rs:205`・`:1078`〜`:1080` | 作る所は `src-tauri/src/biz/product_service.rs:1525` の `None` だけ（消す） |
| `find_all_stock_quantities` | 戻り値を写しの 4 項目と単位・名前の行の型にする | `product_repo.rs:1179`〜`:1202` | 呼ぶ所は `integrity_service.rs:67` だけ。test `test_find_all_stock_quantities_req301`（`rg -n 'fn test_find_all_stock_quantities' src-tauri/src/db/product_repo.rs`）を新しい型の同じ値に |
| 商品の読取りの型 | 写しの 3 列を `Product` に足すのは (b)（商品の結果の表示と一緒）。(a) は `SELECT` の列の並び（`product_repo.rs:342` の `row.get(9)` ほか、位置で読む所）を変えない | — | `ALTER TABLE … ADD COLUMN` は列を末尾に足すので、`SELECT *` で位置を読む所があれば崩れない（着手時に `rg -n 'SELECT \*' src-tauri/src/db/product_repo.rs` で確かめる） |

### S3 BIZ の求め直し（`biz::stock_ledger`、新設）

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| `derive_stock`・`recompute_stock`・`DerivedStock`・`RecheckReason` | [31](../../function-design/31-biz-inventory-service.md) の「求め直しの関数のシグネチャ」（`pub(crate)`。`recompute_stock` の第 1 引数は借りた transaction）。(a) は ADR D3 の判定表の「入出庫の movement」「初期在庫の movement」の行と「計数どうし」、要再確認の理由 2（`same_date_movement`）・3（`conflicting_counts`）。`RecheckReason` は (a) では 2 つの variant（(b) が `undetermined_settlement` を足す）。`phase = 1` の行は `DbError::QueryFailed` で止まる（Y8）。範囲は、決まらない記録のそれぞれを適用する・しないの最小と最大（正の和と負の和を分けて足す）。計数の衝突は衝突する計数のそれぞれを土台に求めた範囲を合わせ、写しは計数値が最小のものを土台にする。和は `checked_add`。Y2 の検査は `recompute_stock` が書く直前 | `src-tauri/src/biz/stock_ledger.rs`、`src-tauri/src/biz/mod.rs:6`〜`:16`（`pub mod stock_ledger`） | Matrix の L1〜L14（計数の無い商品で和と同じ、既定・fresh の計数と入出庫の日付の 3 通り、fresh のカーソル `seen_movement_id`、初期在庫、計数どうしの順と衝突、範囲の端、phase=1 の誤り、上限） |
| 対応表 | `31-biz-inventory-service.md` の module に `biz::stock_ledger` を足す。allowlist には足さない | `src-tauri/tests/design_compliance_test.rs:170`〜`:179` | 未文書化の `pub fn` は fail する（`:726`〜`:738`）。`pub(crate)` は検査が拾わない（`:482`）が、5 つのシグネチャは 31・21 の code block にある（S2 の冒頭）。code block にあって `pub` で無い関数は報告の INFO（未実装）に出るだけで fail しない（`:630`〜`:634`。`apply_stock_change` と同じ） |

### S4 書く側の切替（入出庫 4 業務・商品登録・商品 CSV・旧本体）

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| `apply_stock_change` | 引数に `business_date: &str`（と、(b) のための `settlement_no: Option<i64>`。(a) の呼出しは全部 `None`）を足す。手順: 商品の存在の確認 → movement を仮の `stock_after`（今の写しの値）で INSERT → `recompute_stock` → その movement の `stock_after` を写しの値で UPDATE。`StockChangeOutcome` は今と同じ（写しの値・負の警告・単位） | `src-tauri/src/biz/inventory_service/common.rs:68`〜`:120` | test `common.rs:176`〜`:330` の 6 つの呼出しに業務日を足し、期待は変えない（helper が初期在庫の movement を作るので値は同じ。Matrix の T3）。`:293`〜`:311` の movement の読取りは参照先でも絞る（T5） |
| 入出庫 4 業務 | header の日付を 31 の「業務日の形と暦の検査」にかけ（TX の前）、`apply_stock_change` へ渡す。その検査は `sales_service.rs:507` の `validate_date` に替わる `biz` の共有の関数 1 つで、4 業務と `sales_service` が使う（`sales_service` の今の振舞いは 2 つ変わる: `2026-4-7` を拒む。`2026-ab-07` のように 10 文字で `-` の位置だけ合う入力の文が「存在しない日付です」から「日付の形式が不正です（YYYY-MM-DD）」になる。既存の test〈`sales_service.rs:821` の `2026/03/21`・`:830` の `2026-02-30`〉は `ValidationFailed` だけを見て、その 2 つの入力の文も変わらない。日次売上の日付の欄は ISO の形で送るので画面の振舞いは変わらない）。返品交換は `validate_return_request_shape`（`returns.rs:78`、呼出し `:166`）で確かめ、`register_processed` で在庫を動かすかが分かれる所（`:255`）より前なので、movement を作らないレジ処理済みの返品・交換の保存も同じ検査を通る（4 業務で movement を作らない保存の経路はこれだけ: `rg -n 'register_processed' src-tauri/src/biz/inventory_service --glob '!*test*'` の production の分岐は `returns.rs:255` の 1 つ） | `receiving.rs:222`・`returns.rs:261`・`manual_sale.rs:277`・`disposal.rs:214`（呼出し）、各 `create_*` の検証の段（`receiving.rs:60`〜`:167` ほか）、`returns.rs:78` | 4 業務の既存の test は期待を変えない。足す test は Matrix の W1〜W7 |
| 商品登録・商品 CSV の新規行 | 初期在庫の movement に業務日（PC の今日）を付け、INSERT の後に `recompute_stock` で写しを書く（初期在庫 0 は movement を作らず、求め直しの値 0 が INSERT の値と同じ） | `src-tauri/src/biz/product_service.rs:273`〜`:285`（create）・`:1574`〜`:1590`（import の新規行） | 上書きの行は在庫を変えない（今のまま。`:1525` の `stock_quantity: None` は S2 で field ごと消える） |
| 旧本体（test だけが呼ぶ。停止の入口は変えない） | Z004 の確定（`csv_import_service/commit.rs:157`）は業務日 = 精算日・`None` で `apply_stock_change` を呼ぶ。Z004 の取消の補正（`commit.rs:248`〜`:285` の `apply_void_stock_corrections`）は、無効にした movement の商品を `recompute_stock` で求め直し、`StockCorrection` の old / new を求め直しの前後の写しで返す（値は今と同じ。`recompute_stock` が借りた transaction を取るので、`apply_void_stock_corrections` の第 1 引数を `&DbConnection` から `&rusqlite::Transaction<'_>` に変える。呼出しは `rollback.rs:62` の `&tx` だけ）。棚卸しの確定（`stocktake_service.rs:515`〜`:530`）は補正の movement（業務日 = 確定の日の PC の今日）を入れてから `recompute_stock` を呼ぶ（`update_stock_quantity` の直呼びをやめる） | `src-tauri/src/biz/csv_import_service/commit.rs`、`src-tauri/src/biz/stocktake_service.rs` | 旧本体の test（`commit_tests.rs`・`rollback_tests.rs`・`legacy_stop_tests.rs`・`cross_feature_tests.rs`・`stocktake_service.rs` の test）の期待は変えない。時点証拠の `#[cfg(test)]` の code（`stocktake_service/time_evidence.rs` の `update_stock_quantity_with_revision`）は写しを直に書くまま変えない（(b) で外す。`update_stock_quantity(` の検査に当たらない別の関数） |
| 在庫つきの商品の fixture（初期在庫の movement を作る形へ） | 在庫 ≠ 0 の商品を作り、その後に写しを求め直す経路（S4 の経路と整合性の補正）を通して写し・`stock_after`・負の警告のどれかを確かめる test が使う fixture は、同じ値の初期在庫の movement も入れる（記録が正で写しは求め直し、の契約に fixture を合わせる。値の期待は変えない）。共有の helper `insert_initial_stock_movement(conn, code, quantity, stock_after)`（`movement_type` `receiving`・`reference_type` NULL・業務日は固定の `"2026-04-01"`）を `src-tauri/src/db/test_support.rs` に置く。対象と除外は Matrix の T3（helper の定義ごと。同じ名前の別の定義も 1 つずつ分ける）と T4（写しを直に UPDATE する所）、その結果として読み方を直す test は T5 | `src-tauri/src/db/test_support.rs`、T3・T4 の「書き換える」の file | 書き換える定義: `biz/inventory_service/test_support.rs` の 3、`biz/csv_import_service/test_support.rs:14`、`biz/stocktake_service.rs:615`・`:662`・`:916`・`:945`。除外の理由を書く定義: `biz/integrity_service.rs:256`（写しと記録をわざと食い違わせる）ほか T3 の「変えない」。helper の呼出しの行は変えない。初期在庫の movement を helper の後で自分で入れている `cross_feature_tests.rs:86`〜`:100` は二重になるので自分の INSERT を消す |

### S5 （S4 の続き）`update_stock_quantity` の呼出しを 1 つに

S4 で入出庫・旧本体の 3 か所（`common.rs:93`・`csv_import_service/commit.rs:277`・`stocktake_service.rs:518`）を `recompute_stock` に寄せ、整合性の補正（`integrity_service.rs:193`）は S6 で寄せる。`rg -n 'update_stock_quantity\(' src-tauri/src --glob '!*test*' | rg -v 'db/inventory_repo.rs'` が `src-tauri/src/biz/stock_ledger.rs` の 1 行だけになるのは S6 の後（AC4）。

### S6 整合性の突合（BIZ・wire・画面・操作ログ）

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| `run_integrity_check` | 全商品の写しの 4 項目（`find_all_stock_quantities`）と `derive_stock` を比べ、違う商品を返す。`integrity_check` の操作ログの各 mismatch も同じ形 | `src-tauri/src/biz/integrity_service.rs:64`〜`:135` | 期待を変える test（契約の変更、36。Matrix の T6）: `movements_sum`・`difference` を見る test を `copy`・`derived`・`differing_fields` へ（`rg -n 'movements_sum|difference' src-tauri/src/biz/integrity_service.rs` の test の行。値は同じ） |
| `fix_integrity` | 選んだ商品ごとに、写しと `derive_stock` が同じなら skip、違えば `recompute_stock` で書き、`before`（写し）・`after`（求めた値）を返す。`integrity_fix` の操作ログも同じ形（`trigger` は書かない） | `integrity_service.rs:141`〜`:236` | `integrity_service.rs:524`〜`:527` の完全一致の期待を新しい形の同じ値に |
| 型 | 36 の `StockCopy`・`IntegrityField`・`IntegrityMismatch`・`StockAdjustment`（`specta::Type`、generated enum は snake_case） | `integrity_service.rs:19`〜`:52` | bindings `src/lib/bindings.ts:807`〜`:829`・`:1562`〜`:1569`（再生成） |
| 整合性の画面 | 差異の一覧: 列「商品コード」「商品名」「違う項目」「保存している値」「記録から求め直した値」。違う項目の語は `quantity` →「在庫数」、`recheck` →「要再確認」、`range` →「範囲」を「・」でつなぐ。値は在庫数（今の `formatLengthOrPlain` で単位どおり）と、要再確認なら後ろに「（要再確認 {low}〜{high}）」。今の差の Badge（「システム在庫が多い」「入出庫の合計が多い」）と差の列は消す。補正の確認 dialog と結果は商品ごとに「保存している値 → 記録から求め直した値」。移行の知らせ（75 の注記）: 開いたときに `listLogs({ page: 1, per_page: 100, operation_type: "integrity_fix", start_date: null, end_date: null })` を 1 回読み（新しい順の最新 100 行。`per_page` の上限は `PAGINATION_MAX_PER_PAGE` 200〈`src-tauri/src/constants.rs:6`〉）、その中に `detail_json` の `trigger` が `migration` の行があれば（その後に別の `integrity_fix` の行があっても）、最新のその行で「更新のときに在庫数を記録から求め直し、{N}商品の在庫数を直しました（{日時}）。直した前後は操作ログで見られます。」と「操作ログを開く」（`/settings/logs`）を Alert で出す（読取りに失敗したら知らせを出さず、チェックの操作は止めない） | `src/features/integrity-check/IntegrityCheckPage.tsx:68`〜`:72`（差の語）・`:293`〜`:310`（補正の結果）・`:326`〜`:490`（一覧と dialog）・`:91`〜`:95`（ログの読取り） | test `src/features/integrity-check/IntegrityCheckPage.test.tsx`: 差の語（`rg -F -c 'システム在庫が多い'` 2・`'入出庫の合計が多い'` 2・`'入出庫の合計'` 7）と `movements_sum`（1）を期待する行を新しい語と形へ（契約の変更、75。Matrix の T6）。新しい語は今の test に無い（`rg -F -c` で `記録から求め直した`・`違う項目`・`保存している`・`直した前後` が `src` の test で 0。`範囲` は 6 file にあるが整合性の画面の外） |
| 操作ログの画面 | `integrity_fix` の detail の読取り（`OperationLogsPage.tsx:96`〜`:139` の `parseIntegrityAdjustments`）が、新しい形（`before`・`after` の入れ子）と記録済みの形（`old_stock`・`new_stock`・`adjustment`）の両方を読む。新しい形は 75 の冒頭の注記の「4 項目の old / new」を正本とし、「{商品コード}: {before} → {after}」の before と after をそれぞれ数量・印・範囲つきで出す（各側の書き方は上の整合性の画面の値と同じ: 在庫数〈`formatLengthOrPlain` で単位どおり〉と、その側の印が 1 なら後ろに「（要再確認 {low}〜{high}）」。印 0 の側は在庫数だけで、範囲は DB の CHECK で NULL）。例: 在庫数 5・要再確認・範囲 4〜6 を範囲 3〜7 に補正した要素は「{商品コード}: 5（要再確認 4〜6） → 5（要再確認 3〜7）」。どちらにも合わない要素は今どおり技術情報（JSON）へ回す。`trigger: migration` の行の要約は summary の文のまま | `src/features/operation-logs/OperationLogsPage.tsx` | test `OperationLogsPage.test.tsx` の `integrity_fix` の行（記録済みの形は期待を変えない）に新しい形の行を足す（Matrix の U6。範囲だけが変わる要素で両側の表示を確かめる） |
| 正本の同期 | 75 §75.4〜§75.7 の数量だけの表示と test の oracle の記述を上の形へ、[74](../../function-design/74-ui-operation-logs.md) の UI-11c-D14 に新しい形を足す（実装と同じ PR） | `docs/function-design/75-ui-integrity-check.md`・`docs/function-design/74-ui-operation-logs.md` | — |

### S7 正本の proposed の注記を現行の記述へ（実装と同じ PR）

(a) の分の「proposed・未実装」の注記を、実装した範囲だけ現行の記述に直す（tracking・master・transaction の並びの鍵・写し・移行、21・30・31・36・75 の (a) の分、22 §19、`DB_DESIGN.md`・`ARCHITECTURE.md`・`architecture/*-task-specs.md` の該当の行）。`DB_DESIGN.md` の「stock_quantity整合性チェック」の節（`:137`〜`:143`）も対象に入れ、「SUM と突合」「自動上書きはしない」「入出庫の合計に合わせて補正」「movements_sum へ直接更新」の今の文を、36 の新契約（記録から求め直した値と写しの 4 項目を比べ、補正は求め直しの結果を書く）と vL の手順 8（写しを直して操作ログと整合性の画面で知らせる）に揃える。(b) の分は proposed のまま残し、どこまでが (a) で入ったかを各節に 1 文で書く。`docs/function-design/90-traceability.md` は `cargo run --bin generate_traceability` で再生成する（手で直さない）。

### 節の所有表（並走の lane と重なる file）

| file | 並走の lane が所有する節 | 本 lane が触る所 | merge の後に揃える所 |
|---|---|---|---|
| `docs/function-design/22-mnt-migration.md` | SD 直読みの runtime（PR #172）: §17（MNT-03-D15）。JAN の別名（PR #174）: §18（MNT-03-D16）。原価の lane（未起票）: §16 の vB | §16 の「DB の作り直しと順」の段（D-123）と §19（新設、MNT-03-D17） | 3 つの lane が末尾に節を足すので、後から merge する側が §17・§18・§19 の順に並べ直す。§18 の「`products` を作り直す migration の規則」は vL に当たらない（列の追加だけ。§19 の手順 6） |
| `src-tauri/src/db/migration.rs` | §17・§18 の lane: `migrations()` の登録と版の期待の行 | 登録・版の期待の行・`expected_tables`・`migrate_up_to` の helper | 版の番号は先に merge した側が若い番号。後の側が登録の順と版の期待（S1 の行）を取り直す。`migrate_up_to` は先に入れた lane が足し、後の lane は使う |
| `src-tauri/src/db/product_repo.rs` | 単位の語の直し（並走の 365）: `ProductUpdates.stock_unit` の field と UPDATE の組立て | `ProductUpdates.stock_quantity` の field と UPDATE の組立て（`:205`・`:1078`〜`:1080`）、`find_all_stock_quantities` | 隣の行。後から merge する側が両方を残す |
| `src-tauri/src/biz/product_service.rs` | 単位の語の直し（365）: `update_product`・`validate_update_request`・商品 CSV の preview と commit の上書きの行の単位 | `create_product` の初期在庫（`:273`〜`:285`）、`commit_import` の新規行の初期在庫（`:1574`〜`:1590`）、`:1525` の `stock_quantity: None` の削除 | `:1525` は上書きの行の `ProductUpdates` の literal で、365 が同じ literal に `stock_unit` を足す。後の側が両方を反映する |
| `docs/function-design/30-biz-product-service.md` | 365: §4.4・§4.9 の単位。JAN の別名（366）: §4.8・§4.9 の別名 | 触らない（30 の D-118 の節は (a) の範囲で変わらない。S7 で (a) の分を現行へ直すときは冒頭の D-118 の節だけ） | — |
| `docs/decision-log.md` | 各 lane の決定（D-120〜D-122・D-124） | 末尾の D-123 と、D-118・D-119 の末尾の Superseded in part の行 | 末尾への追記は merge の順で両方を残す |
| `docs/backlog.md` | 各 lane の項目 | 「棚卸しの数の置き場と在庫数の求め直しの runtime」と「原価の lane（C）」の 2 項目の文 | 他の行を並べ替えない |
| `docs/db-design/pos-tables.md` | SD 直読み（367）: §12b の `settlement_no`。JAN の別名（366）: B-1 | 冒頭の D-118 の節の移行の 1 文 | — |
| `docs/DB_DESIGN.md`・`docs/architecture/*` | JAN の別名（366）: `DB_DESIGN.md` の表の一覧の行、biz・cmd の task specs | `DB_DESIGN.md:11`・`ARCHITECTURE.md:13`・`mnt-task-specs.md:7` の作り直しの文 | — |
| `docs/Plans.md` | （lane の行は足さない。D-097） | `## 現在地` の runtime の順の 1 文（D-123 に合わせた） | — |

## Non-scope

- 上の Goal の「非目的」の全部（(b)、原価の lane、単位の語、JAN の別名）。
- 停止 ADR の 5 つの入口（`stocktake_service.rs` の開始・保存・確定、`csv_import_service/commit.rs:27`〜`:32` の確定、取消）の振舞いと文言。
- `MovementRecord`・在庫推移の画面（66）・商品の結果の型（40）・商品一覧と在庫照会の表示（50・58）。
- `sales_service` の日付の検査の文の変更（2 つの文はそのまま。どの入力がどちらの文になるかの変化は S4 の「入出庫 4 業務」の行）。
- 起動の失敗の画面の文の種類を増やすこと（vL の止まりは今の `DatabaseInit` の文に手順 1 の message を添える。「再起動してもう一度」の前置きは vU・vB と同じく残る。下の Residual）。

## Acceptance Criteria

- AC1（migration）: `cd src-tauri && cargo test --lib db::` が全 PASS し、Matrix の M1〜M9 の test 名が `rg -n 'fn <名前>' src-tauri/src/db` で 1 件ずつ見つかる。
- AC2（求め直し）: `cd src-tauri && cargo test --lib biz::stock_ledger` が全 PASS し、Matrix の L1〜L14 の test 名が見つかる。
- AC3（今と同じ値）: Matrix の E1〜E4 の同値の test（計数の無い DB で、4 業務・商品登録・商品 CSV・整合性の補正の後の `stock_quantity` と各 movement の `stock_after` が、書込みの前の写し + 差分の列と同じ）が PASS する。既存の 4 業務・商品・整合性・旧本体の test は期待を変えずに PASS する（Matrix の「既存の test の移し方」の「期待を変える」の行〈T1 の 2 か所と T6〉を除く）。
- AC4（写しを書く所は 1 つ）: `rg -n 'update_stock_quantity\(' src-tauri/src --glob '!*test*' | rg -v 'db/inventory_repo.rs'` が `src-tauri/src/biz/stock_ledger.rs` の 1 行だけ。`rg -n 'stock_quantity: Option<i64>' src-tauri/src/db/product_repo.rs` が 0 行。`rg -n 'sum_movements_(by|for)_product' src-tauri/src` が 0 行。
- AC5（業務日）: `rg -n 'fn apply_stock_change' -A8 src-tauri/src/biz/inventory_service/common.rs | rg -c 'business_date'` が 1 以上。Matrix の W1〜W7（形の違う日付・実在しない日付で 4 業務が何も書かない、レジ処理済みの返品も同じ、DB の CHECK が形を拒む）が PASS。
- AC6（整合性）: `rg -c 'movements_sum' src src-tauri/src` が 0（`--glob` なし。記録済みの操作ログの key の読取りは `old_stock` で行い `movements_sum` を使わない）。`src/lib/bindings.ts` に `export type StockCopy` と `export type IntegrityField` がある。`npm test -- src/features/integrity-check src/features/operation-logs` が全 PASS し、Matrix の U1〜U8 の test 名が見つかる。
- AC7（gate）: `cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`、`cargo run --bin generate_bindings` の後に `git diff --exit-code src/lib/bindings.ts`、`cargo run --bin generate_traceability -- --check`、`cargo test --test design_compliance_test`、`npm run typecheck && npm run lint && npm run format:check && npm test && npm run build`、`bash scripts/doc-consistency-check.sh` と `--target plan` が exit 0。L3 を含むので Writer は `cd src-tauri && cargo check --release` を通す。
- AC8（停止の入口）: 停止の test（`csv_import_service/tests/legacy_stop_tests.rs`、`stocktake_service.rs` の停止の test）が期待を変えずに PASS し、停止の文言（`IMPORT_SUSPENDED_MESSAGE` ほか）の `rg -c` が変わらない。
- AC9（既存の test の移し方）: Matrix の T1〜T5 の command を実装の後に逐語で回し、出力の各行を T1〜T5 の分類（と本 lane で足した test）に当てた表を PR の本文に貼る。T1〜T5 の「書き換える」「変えない」の所で値の期待（`assert` の右辺・件数・message）が変わっていない。T5 の「書き換える」3 か所（`common.rs:296`・`disposal.rs:369`・`returns.rs:461`）が `reference_type` でも絞る。
- AC10（新しい関数のシグネチャ）: 正本の `rg -n '^pub\(crate\) fn (derive_stock|recompute_stock)\(' docs/function-design/31-biz-inventory-service.md` が 2 行、`rg -n '^pub\(crate\) fn (update_movement_stock_after|list_stock_counts_for_product|list_ledger_movements_for_product)\(' docs/function-design/21-io-inventory-repo.md` が 3 行（本 packet の commit で済み）。実装の後に `rg -n 'pub\(crate\) fn (derive_stock|recompute_stock|update_movement_stock_after|list_stock_counts_for_product|list_ledger_movements_for_product)\(' src-tauri/src` が 5 行、`rg -c 'derive_stock|recompute_stock|update_movement_stock_after|list_stock_counts_for_product|list_ledger_movements_for_product' src-tauri/tests/design_compliance_test.rs` が 0 件（allowlist に足さない）。

AC の数の baseline（AC は AC1〜AC10 の 10。round 2 の是正で AC10 を足し、round 3 の是正〈どちらも Review Response 参照〉では AC を変えず、全部を逐語で実行し直した。2026-10-18、本 worktree〈code は main `f5d7aaf4` と同じ: `git diff --stat f5d7aaf4 HEAD -- src-tauri src` が空〉。rg の file の順は実行ごとに変わる）:

```
$ rg -n 'update_stock_quantity\(' src-tauri/src --glob '!*test*' | rg -v 'db/inventory_repo.rs'
src-tauri/src/biz/csv_import_service/commit.rs:277:        inventory_repo::update_stock_quantity(conn, product_code, new_stock)?;
src-tauri/src/biz/stocktake_service.rs:518:            inventory_repo::update_stock_quantity(&tx, &item.product_code, item.actual_count)?;
src-tauri/src/biz/inventory_service/common.rs:93:    let updated = inventory_repo::update_stock_quantity(conn, product_code, stock_after)?;
src-tauri/src/biz/integrity_service.rs:193:        inventory_repo::update_stock_quantity(&tx, product_code, new_stock)?;
exit=0
$ rg -n 'stock_quantity: Option<i64>' src-tauri/src/db/product_repo.rs
205:    pub stock_quantity: Option<i64>,
exit=0
$ rg -n 'sum_movements_(by|for)_product' src-tauri/src
src-tauri/src/biz/integrity_service.rs:70:    let movements = inventory_repo::sum_movements_by_product(conn)?;
src-tauri/src/biz/integrity_service.rs:166:        let movements_sum = inventory_repo::sum_movements_for_product(&tx, product_code)?;
src-tauri/src/db/inventory_repo.rs:167:pub fn sum_movements_by_product(conn: &DbConnection) -> Result<Vec<ProductMovementSum>, DbError> {
src-tauri/src/db/inventory_repo.rs:191:pub fn sum_movements_for_product(conn: &DbConnection, product_code: &str) -> Result<i64, DbError> {
src-tauri/src/db/inventory_repo.rs:847:    // ===== FUNC-10.6: sum_movements_by_product テスト =====
src-tauri/src/db/inventory_repo.rs:850:    fn test_sum_movements_by_product_req303_normal() {
src-tauri/src/db/inventory_repo.rs:898:        let result = sum_movements_by_product(&conn).unwrap();
src-tauri/src/db/inventory_repo.rs:907:    fn test_sum_movements_by_product_req303_voided_excluded() {
src-tauri/src/db/inventory_repo.rs:933:        let result = sum_movements_by_product(&conn).unwrap();
src-tauri/src/db/inventory_repo.rs:939:    fn test_sum_movements_by_product_req303_no_movements() {
src-tauri/src/db/inventory_repo.rs:943:        let result = sum_movements_by_product(&conn).unwrap();
src-tauri/src/db/inventory_repo.rs:947:    // ===== FUNC-10.6: sum_movements_for_product テスト =====
src-tauri/src/db/inventory_repo.rs:950:    fn test_sum_movements_for_product_req303_normal() {
src-tauri/src/db/inventory_repo.rs:982:        let sum = sum_movements_for_product(&conn, "SF-001").unwrap();
src-tauri/src/db/inventory_repo.rs:987:    fn test_sum_movements_for_product_req303_no_movements() {
src-tauri/src/db/inventory_repo.rs:991:        let sum = sum_movements_for_product(&conn, "NONEXISTENT").unwrap();
exit=0
$ rg -n 'fn apply_stock_change' -A8 src-tauri/src/biz/inventory_service/common.rs | rg -c 'business_date'
exit=1
$ rg -c 'movements_sum' src src-tauri/src
src-tauri/src/biz/integrity_service.rs:24
src/lib/bindings.ts:1
src-tauri/src/db/inventory_repo.rs:6
src/features/integrity-check/IntegrityCheckPage.test.tsx:1
src/features/integrity-check/IntegrityCheckPage.tsx:2
exit=0
$ rg -n 'export type (StockCopy|IntegrityField)' src/lib/bindings.ts
exit=1
$ rg -c 'pub mod stock_ledger' src-tauri/src/biz/mod.rs
exit=1
$ rg -U -n -o 'FROM inventory_movements[^";]*?product_code' src-tauri/src | rg ':FROM inventory_movements' | wc -l
16
exit=0
$ rg -n '^pub\(crate\) fn (derive_stock|recompute_stock)\(' docs/function-design/31-biz-inventory-service.md
20:pub(crate) fn derive_stock(conn: &DbConnection, product_code: &str) -> Result<DerivedStock, BizError>
21:pub(crate) fn recompute_stock(tx: &rusqlite::Transaction<'_>, product_code: &str) -> Result<DerivedStock, BizError>
exit=0
$ rg -n '^pub\(crate\) fn (update_movement_stock_after|list_stock_counts_for_product|list_ledger_movements_for_product)\(' docs/function-design/21-io-inventory-repo.md
18:pub(crate) fn update_movement_stock_after(conn: &DbConnection, movement_id: i64, stock_after: i64) -> Result<(), DbError>
20:pub(crate) fn list_stock_counts_for_product(conn: &DbConnection, product_code: &str) -> Result<Vec<StockCountRow>, DbError>
22:pub(crate) fn list_ledger_movements_for_product(conn: &DbConnection, product_code: &str) -> Result<Vec<LedgerMovementRow>, DbError>
exit=0
$ rg -n 'pub\(crate\) fn (derive_stock|recompute_stock|update_movement_stock_after|list_stock_counts_for_product|list_ledger_movements_for_product)\(' src-tauri/src
exit=1
$ rg -c 'derive_stock|recompute_stock|update_movement_stock_after|list_stock_counts_for_product|list_ledger_movements_for_product' src-tauri/tests/design_compliance_test.rs
exit=1
```

完了時の期待: AC4 の 1 行目が `stock_ledger.rs` の 1 行、2・3 行目が 0 行。AC6 の `movements_sum` が 0。AC5 が 1 以上。AC9 の T5 の command の出力から `inventory_repo.rs:170`・`:194`（S2 で消す和の関数）が消える。AC10 の実装の側の 1 つ目が 5 行、2 つ目が 0 件（正本の側の 2 行・3 行は上のとおり済み）。test の本数は AC にしない（PR と runner の出力を参照）。

## Design Readiness

設計の完了条件は `docs/DEV_WORKFLOW.md` Design Phase Rules の Design completion criteria。

- 引用する設計正本（節まで）: ADR SPEC-STK-CNT-D1（鍵）・D2（求め直しと写し・`stock_after`）・D3（窓の判定表の入出庫・初期在庫・計数どうしの行、理由 2・3、範囲）・D9（(a) の列と表）・D10（移行）・D12（配布の単位と境の細部）。[tracking](../../db-design/tracking-system-tables.md)・[master](../../db-design/master-tables.md)・[transaction](../../db-design/transaction-tables.md) の「計数の置き場と在庫の求め直しの契約」。[20](../../function-design/20-io-product-repo.md)・[21](../../function-design/21-io-inventory-repo.md)・[30](../../function-design/30-biz-product-service.md)・[31](../../function-design/31-biz-inventory-service.md)・[36](../../function-design/36-biz-integrity-check.md) の同名の節、[75](../../function-design/75-ui-integrity-check.md) の冒頭の注記、[22 §16](../../function-design/22-mnt-migration.md)（作り直しの手順と順、回復）・§19（vL）、D-118・D-123。
- 必要な設計成果物（状態）: (1) 作り直しの順と承認（D-123、22 §16、ADR D10、backlog）= updated in this PR。(2) migration の手順（22 §19）= updated in this PR。(3) (a) と (b) の境（ADR D12）= updated in this PR。(4) 業務日の形と暦の検査（書く側は 31、vL は 22 §19 の手順 2）と DB の CHECK（tracking）= updated in this PR。(5) `NewMovement` の phase の決め方と `stock_after` の書き方（21・31）= updated in this PR。(6) 整合性の型の形と移行の知らせ（36・75）= updated in this PR。(7) 画面の語と並び（75 §75.4〜§75.7、74 UI-11c-D14）= 本 packet の S6 に決め、実装と同じ PR で正本へ（S6 の「正本の同期」。75 は「見た目の細部は runtime の lane」と書く）。
- plan にしかない durable な判断の昇格先: 上の (1)〜(6) は正本と D-123 に置いた。packet にだけあるのは、規模と手法、S1〜S7 の順、所有表、既存の test の移し方（Matrix の T1〜T6）、横断の約束の表（Y1〜Y8 の保証する所の file:line）、画面の語（(7)。実装の PR で 75・74 へ）。
- 前提・制約と、延期した design gap の follow-up: (b) へ回したものは ADR D12 と backlog に書いた。移行の知らせは `integrity_fix` の新しい順の最新 100 行（`per_page` 100 の 1 page）だけを見るので、移行の後に `integrity_fix` が 100 行を超えると古い移行の行を見つけない（下の Residual。開発・試験 DB にだけ起きる）。
- 絶対保証の例外と escape hatch の自己点検: 「`stock_quantity` を書くのは `recompute_stock` だけ」の例外は vL の手順 8（db の層は BIZ を呼べないので SQL で同じ値を書く。手順 1 で計数・Z004・棚卸しの無い DB に限るので、値は求め直しと同じ。M6 と L1 で突き合わせる）と、時点証拠の `#[cfg(test)]` の `update_stock_quantity_with_revision`（test だけ、(b) で外す）。「計数が無い間は値が今と同じ」の例外は、写しと和がもともと違っていた DB（vL が直して知らせる）。「vL は止まる DB で何も変えない」は手順 1・2・8 の検査が書込みの前にある（M3〜M5）。
- 判定: ready。owner に聞く設計の問いは無く、(a) の Scope の全ての振舞いが正本か本 packet の S6（画面の語）から決まる。

## Registration / Generation Obligations

| 変更対象 | 登録・生成義務 |
|---|---|
| Tauri command（frontend から呼ぶ） | command の追加・改名は無い。`run_integrity_check`・`fix_integrity` の戻り値の型が変わるので `cd src-tauri && cargo run --bin generate_bindings` で `src/lib/bindings.ts` を再生成する（S6） |
| function-design doc 新設 | 該当なし（新しい doc は無い）。新しい module `biz::stock_ledger` を `src-tauri/tests/design_compliance_test.rs` の `31-biz-inventory-service.md` の対応に足す（S3）。新しい 5 つの関数は `pub(crate)` で、シグネチャは 31・21 の code block（allowlist に足さない。S2 の冒頭）。新しい `db::schema_v{N}` の `pub(crate)` 関数は検査の対象外（`pub fn` だけを見る）だが、`pub fn` にするなら 22 の対応へ足す |
| source / workflow doc 新設・改名・削除 | 該当なし |
| REQ / coverage の追加・変更・削除 | test 名・comment に REQ-201〜REQ-205・REQ-301・REQ-904・REQ-903 を付ける。`cd src-tauri && cargo run --bin generate_traceability` で `docs/function-design/90-traceability.md` を再生成する（S7） |
| route 新設・改名・削除 | 該当なし |
| operator 画面新設・改名・削除 | 該当なし（整合性検証と操作ログの画面の中身だけを変える。到達の導線は今のまま） |

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | Which concepts belong to replaceable external adapters, and which concepts are stable app-core contracts? | Architecture, function design, decision-log, Plan Packet | 当たらない。(a) は POS の adapter（Z004 の取込み）に触れない（停止中。Z004 の鍵は (b)） | — |
| Fact check / design decision split | Which claims are observed facts from hardware/tool/files, and which are app decisions that need source-doc promotion? | Investigation doc, source design docs, decision-log | 当たる。SQLite と chrono の振舞いは Contract Probe P1〜P7 で観測した。作り直しの順は owner の決定（TD-311 → D-123）。(a) と (b) の境・型の形は起草役の判断で正本へ置いた | D-123、ADR D12、21・31・36・75 |
| Lifecycle / retry | What happens before, during, after, and after failure for import/export, duplicate input, rollback, retry, cancellation, and re-run? | Function design, DB design, UI design, Test Matrix | 当たる。vL は 1 TX で、失敗は全部戻る（M3〜M5・M9）。止まった DB は作り直し。書込みの失敗（日付・上限）は TX の前か TX 全体の巻戻し（W1〜W7・L13）。冪等な再送は新しい movement を作らない（今の test のまま） | Matrix の State Lifecycle |
| Operator workflow | What does the operator do in the real sequence across app, external tool, media, print/export, backup, and recovery? | Screen/UI design, function design, Plan Packet manual checks | 当たる。店の操作は変わらない（値が同じ）。開発者は止まった DB を作り直す（22 §16）。整合性の画面の語が変わる | L3-1・L3-2 |
| Replacement path | If the external system changes, which files/modules/docs are replaced and which app-core contracts remain stable? | Architecture, function design, decision-log | 当たらない（外部の system に触れない） | — |
| Data safety / evidence | How can the claim be supported by anonymized shape/count/hash/procedure evidence without committing real store data? | Plan Packet Data Safety, investigation doc, review evidence | 当たる。fixture は全部合成。owner の L3 用の DB は作り直しで消さずに移す（下の Data Safety） | Data Safety |
| Reporting / accounting semantics | Are totals, summaries, item records, returns, corrections, and inventory movements modeled separately enough to avoid false business meaning? | DB design, function design, report design, Test Matrix | 当たる。`stock_after` は保存した時点の写しの値（D2）で、(a) の間は今と同じ。整合性の補正は movement を作らない（今のまま）。計数は movement を作らない（(b)） | E1〜E4 |
| Manual verification | Which assertions cannot be proven by automated tests and require Windows native L3, external tool import, or real-device confirmation? | Plan Packet, Test Matrix, PR body | 当たる。Windows の実 DB での vL の止まり方・当たり方と、整合性の画面の見た目（L3-1・L3-2）。差異ありの一覧と移行の知らせは DB の手での改変が要る（L3 Eligibility の (3)）ので自動 test と、Coordinator が渡す画面の写し（下の Test Plan） | Test Plan の L3 |
| 環境・再現性 | 新設の環境依存（toolchain / CI runner / OS 差異等）を repo-pinned config で強制するか、明示的に defer するか | repo-pinned config, Plan Packet | 当たる。列の追加の CHECK が他の列を参照し既存の行に当たる振舞いは SQLite 3.37.0 以降（同梱は `libsqlite3-sys` 0.28.0 の 3.45.0。`src-tauri/Cargo.toml:32` の `rusqlite = { version = "0.31", features = ["bundled"] }` で固定）。M8 が CHECK の効きを確かめるので、同梱の SQLite を下げれば red になる | M8 |

## Boundary / Wire Contract

- producer: BIZ `run_integrity_check`・`fix_integrity`（`IntegrityResult`・`IntegrityFixResult`）、vL と BIZ の `integrity_check`・`integrity_fix` の操作ログ（`operation_logs.detail_json`）、DB の `inventory_movements`・`products`・`stock_counts` の列。
- consumer: frontend の `IntegrityCheckPage`（command の結果と `listLogs` の `integrity_fix`）、`OperationLogsPage`（`integrity_check`・`integrity_fix` の detail）、BIZ の求め直し（DB の列）。
- wire type: `StockCopy { stock_quantity: number, stock_recheck: boolean, stock_range_low: number | null, stock_range_high: number | null }`、`IntegrityField = "quantity" | "recheck" | "range"`、`IntegrityMismatch { product_code, name, stock_unit: ProductStockUnit, copy: StockCopy, derived: StockCopy, differing_fields: IntegrityField[] }`、`StockAdjustment { product_code, stock_unit: ProductStockUnit, before: StockCopy, after: StockCopy }`。`IntegrityResult`・`IntegrityFixResult` の外側の field は変えない。操作ログの detail_json: `integrity_check` は `{checked_count, mismatch_count, mismatches: [{product_code, stock_unit, copy, derived, differing_fields}]}`、`integrity_fix` は `{…今の外側の key…, adjustments: [{product_code, stock_unit, before, after}]}`、vL の行は同じ形に `"trigger": "migration"`。
- internal type: `DerivedStock { stock: i64, range: Option<(i64, i64)>, recheck_reasons: Vec<RecheckReason> }`（wire に出さない）。`NewMovement { …, business_date: String, settlement_no: Option<i64> }`。
- precision/range: 数は全部 i64 で絶対値 `999999999` 以内（Y2・Y3）なので JavaScript の安全な整数に入る。`stock_recheck` は DB の 0/1 を bool にする。
- round-trip path: DB → BIZ の `StockCopy` → specta → bindings → 画面。操作ログは JSON の文字列 → 画面の parser（数は `Number.isSafeInteger` で確かめる今の型）。
- invalid input: 画面の parser は、新しい形にも記録済みの形にも合わない要素を技術情報（JSON）へ回す（今のまま）。`range` が片方だけ NULL の `StockCopy` は DB の CHECK で作れない。
- compatibility: 記録済みの操作ログ（`old_stock`・`new_stock`・`adjustment`、`stock_quantity`・`movements_sum`・`difference`）は書き換えず、画面が読み続ける（U6）。command の名前・引数は変えない。

## Test Plan

Test Design Matrix: [2026-10-18-stocktake-count-runtime-a](test-matrices/2026-10-18-stocktake-count-runtime-a.md)。

- If the Human Gate includes L3, Writer completion includes `cargo check --release` before the owner native build; this is not a CI gate.
- targeted tests: `cd src-tauri && cargo test --lib db::` / `cargo test --lib biz::stock_ledger` / `cargo test --lib biz::inventory_service` / `cargo test --lib biz::product_service` / `cargo test --lib biz::integrity_service` / `cargo test --lib biz::csv_import_service` / `cargo test --lib biz::stocktake_service`、`npm test -- src/features/integrity-check src/features/operation-logs`。
- negative tests: Matrix の M3〜M5・M9（vL が止まる）、W1〜W7（日付）、L12〜L14（phase=1・上限・衝突の端）、U4〜U6（画面の parser）。
- compatibility checks: 記録済みの操作ログの形を画面が読む（U6）。既存の 4 業務・商品・整合性・旧本体の test が期待を変えずに通る（AC3）。
- data safety checks: fixture は全部合成。L3 の DB の作り直しは消さずに移す（Data Safety）。
- main wiring/integration checks: E1〜E4（command に近い BIZ の入口から、写しと `stock_after` を確かめる）、M6（vL の後の写しと `derive_stock` の一致）、U1・U7（画面が command の新しい形を出す）。

L3（Human Gate `manual`。owner は 2026-10-20 から右手が使えず左手と音声で返し、PC は付けっぱなし。Draft のまま溜め、(a) の L3 は 1 round で PASS / FAIL を返してもらう）:

| # | 到達経路 | 入力物 | 観測可能な合格基準 | 依存する既知 backlog |
|---|---|---|---|---|
| L3-1 | (0) 更新の前に、今の版で (a) 在庫照会で代表の商品 3 つの在庫を控え、(b) 整合性検証の「チェック」の結果（「差異はありません」か、差異の一覧の商品コード・「システム在庫」・「入出庫の合計」）を控える（どちらも画面の写しでよい）。(1) 新しい版（MSI）を入れて起動する | owner の今の L3 用の DB（`%APPDATA%\com.kosei.inventory`）。棚卸しか Z004 の取込みの記録を持つかは未確認 | (i) 持つ場合: 起動が止まり、「データベースの準備に失敗」の文の下に「棚卸しか商品別売上（Z004）の取込みの記録がある DB はこの版へ移せません。開発・試験 DB は作り直してください（22 §16 の作り直しの手順）」と表名が出る。作り直し（下の Data Safety の手順）の後に起動すると空の DB で開く。(ii) 持たない場合: 起動できる。(0)(b) で差異の無かった商品（(0)(a) の代表の商品のうち差異の一覧に無いもの）の在庫は (0)(a) の値と同じ。(0)(b) の差異の一覧の商品は、在庫が控えた「入出庫の合計」になり、操作ログの `integrity_fix`（更新のときの補正）の前後の値と、整合性検証の画面の移行の知らせの商品の数が、その差異の一覧と一致する。差異が無かったときは、補正の操作ログも移行の知らせも出ない。(iii) 起動が止まり、(i) と別の止まりの文（「データベースの準備に失敗」の下に、22 §19 の手順 2・8・9 の、作り直しの案内を添えた文）が出た場合: その文の写しを返してもらい、Coordinator が 22 §19 の手順の文と突き合わせて判定する（作り直しの後は (i) と同じ） | 作り直しの手順は 22 §16（D-123） |
| L3-2 | 起動した版で、商品登録（初期在庫 5）→ 入庫 3 → 整合性検証の「チェック」 | 画面の入力だけ | 在庫照会で 8、整合性検証が「差異なし」、画面に「入出庫の合計」「システム在庫が多い」の語が無い | — |

- 差異ありの一覧・補正の dialog と結果・移行の知らせの見た目は、DB の手での改変が要る（L3 Eligibility の (3)）ので L3 に入れない。自動 test（U1〜U8）に加え、Coordinator が Ready の依頼と同時に、frontend の test の render（mock の command）か Vite の dev の画面の写しを owner に渡し、見た目の確認（Human Visual Confirmation）を L3 と同じ 1 回の PASS / FAIL に含める。写しを作れなければ、その残りの危険を PR の本文に書き、受け入れを Ready の判断に含める。
- fixture: L3-1 は owner の今の DB を使う（新しい fixture は要らない）。作り直しが要るときの移す先は Data Safety の手順。

## Review Focus

- 「計数が無い間は値が今と同じ」が、4 業務・商品登録・商品 CSV・整合性の補正・旧本体の全ての書込みで成り立つか（E1〜E4、既存の test の期待を変えていないか）。`stock_after` の書き方（呼出しごと）で、同じ商品の明細が複数ある要求の途中の値が今と同じか。
- vL: 止まる検査が書込みの前にあるか、`sqlite_sequence` を戻すか、業務日の埋め方（参照先・初期在庫の `created_at`）、手順 8 の値が `derive_stock` と同じになる根拠（手順 1 の制限）、版の番号と並走の §17・§18 の migration との順。
- 求め直しの窓の判定（ADR D3 の入出庫・初期在庫・計数どうしの行）と範囲の求め方、`phase = 1` の誤りで止まること。(b) の分を (a) に入れていないか、(a) の分を (b) に送りすぎていないか（ADR D12 の境の細部と発注書の列挙）。
- 写しを書く経路が `recompute_stock` だけになったか（AC4）。時点証拠の test の code が `update_stock_quantity_with_revision` で写しを直に書くことを残すのは (b) までの例外として妥当か。
- 整合性の型・画面・操作ログの記録済みの形の読取り、移行の知らせの読取りの失敗の扱い（チェックを止めない）。
- 公開 repo に実データを書いていないか。並走の lane の節を触っていないか（所有表）。

## Contract Ledger

触る設計正本の節の契約を行にした（隣接の契約の sweep: ADR D1〜D12 と、tracking・master・transaction・pos・20・21・30・31・32・35・36・40・50・51・58・65・66・73・74・75 の D-118 の節を当て、(a) の Scope が触り得ないものは下の「除外」に理由つきで書いた）。

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| SPEC-STK-CNT-D1（入出庫・初期在庫・計数の鍵） | ADR D1、tracking の鍵の表 | S1（列）、S2（`NewMovement`）、S4（業務日を渡す） | M1・M2・W1・W2・E1 | — |
| SPEC-STK-CNT-D2（求め直しと写し、`stock_after`、上限は書く直前） | ADR D2、tracking の求め直し、31 | S3・S4・S5 | L1・L13・E1〜E4 | L3-2 |
| SPEC-STK-CNT-D3（入出庫・初期在庫・計数どうしの行、理由 2・3、範囲） | ADR D3、31 | S3 | L2〜L12 | 非対象: 精算 k の行・理由 1（(b)） |
| SPEC-STK-CNT-D9（(a) の列と表） | ADR D9、tracking・master | S1 | M1・M7・M8 | 非対象: `csv_imports`・`sale_records`・棚卸しの完了の列（(b)） |
| SPEC-STK-CNT-D10（移行: 止まる・埋める・写しを直して知らせる） | ADR D10、tracking の移行、22 §19 | S1・S6（知らせ） | M3〜M6・M9・U8 | L3-1。非対象: 共有 JAN の違反の知らせ（(b)） |
| SPEC-STK-CNT-D12（(a) と (b) の境） | ADR D12 | 全段（停止の入口を変えない） | AC8 の停止の test | — |
| MNT-03-D17（vL の手順・sqlite_sequence・version） | 22 §19 | S1 | M1〜M9 | — |
| MNT-03-D1（COMMIT・ROLLBACK は共有の helper） | 22 §3.2 | S1 | 構造の test（file の列挙に足す） | — |
| D-123（作り直しの承認と順） | decision-log、22 §16 | Data Safety の作り直しの手順 | — | L3-1 |
| IO-21（`NewMovement` の phase、`update_stock_quantity` の写し、`update_movement_stock_after`、読取り） | 21 の新契約 | S2 | M2・L1・E1 | 非対象: `MovementRecord` の鍵、`insert_stock_count`（(b)） |
| IO-20（`ProductUpdates.stock_quantity` を除く） | 20 の新契約 | S2 | AC4 | 非対象: 商品の読取りの写しの 3 列（(b)） |
| BIZ-02（`apply_stock_change` の手順、`stock_after` は呼出しごと、業務日の検査） | 31 の新契約 | S4 | W1〜W7・E1・E2 | — |
| BIZ-01（初期在庫の movement と求め直し、上書きは在庫を変えない） | 30 の新契約、master | S4 | E3・既存の商品 CSV の test | 非対象: 共有 JAN・在庫連動を始める検査（(b)） |
| transaction の TX（鍵を書き同じ TX で求め直す、どれかの失敗で全体を戻す） | transaction-tables の新契約 | S4 | L13・W6 | — |
| BIZ-07（突合の相手・4 項目・補正は求め直し・ログの形・移行の trigger） | 36 の新契約 | S6 | I1〜I6 | — |
| UI-11（整合性の画面の比べ方・語・dialog・移行の知らせ） | 75 の冒頭の注記（S6 で §75.4〜§75.7 へ） | S6 | U1〜U5・U7・U8 | L3-2 と Human Visual Confirmation |
| UI-11c-D14（`integrity_fix` の detail の表示、記録済みの形） | 74（S6 で D14 へ） | S6 | U6 | — |
| X3 の上限（在庫 `999999999`） | 共通規則「入力の上限と安全な整数の範囲」 | S3（`recompute_stock`）、S1（vL 手順 8） | L13・M5 | — |

除外（触らない契約と理由）: SPEC-STK-CNT-D4（Z004 の識別・共有 JAN）・D5（計数の入力）・D6（完了と評価額）・D7（取消と取り込み直し）・D8（在庫連動の切替）は (b)。D11（EJ を在庫に使わない）は (a) で触る code が無い。32・35・40・41・42・50・51・58・65・66・73 の D-118 の節は (b) の経路と画面。停止 ADR の SPEC-STOP-D1〜D6 は変えない（AC8）。

## Contract Probe

本 lane が依る外部の前提（SQLite と chrono の振舞い）を、2026-10-18 に scratch の crate（`rusqlite = "=0.31.0"` bundled、`chrono = "=0.4.44"`。repo の外の作業 folder。commit しない）で確かめた。

- P1 列の追加の CHECK が他の列を参照し、既存の行に当たる: `ALTER TABLE products ADD COLUMN stock_recheck INTEGER NOT NULL DEFAULT 0 CHECK(stock_recheck IN (0,1))`、`… stock_range_low INTEGER`、`… stock_range_high INTEGER CHECK((stock_recheck = 0 AND … ) OR (stock_recheck = 1 AND … AND stock_range_low <= stock_range_high))` -> 同梱の SQLite 3.45.0 で全部 ok。印 1 で範囲なし・low > high・印 0 で範囲あり は `CHECK constraint failed`、印 1 で low ≤ high と既定の INSERT は ok。既存の行に違反する CHECK の列の追加は `CHECK constraint failed`（P4）。`products` を作り直さずに master の table の CHECK と同じ縛りを置ける。
- P2 業務日の NOT NULL は列の追加では足せない: `ALTER TABLE m ADD COLUMN business_date TEXT NOT NULL` -> `Cannot add a NOT NULL column with default value NULL`。`inventory_movements` を作り直す。
- P3 `inventory_movements` の作り直しと JAN の別名の trigger（22 §18 の 3 つ）: `foreign_keys = ON` のまま `inventory_movements_new` を作って写し、`DROP TABLE`・`RENAME` -> ok。`PRAGMA foreign_key_check` 0 行、`integrity_check` ok、別名の trigger は効いたまま（`jan_alias_held_by_product`）。**`sqlite_sequence` は写した最大の id に戻る**（作り直しの前の seq 3・最大の id 2 の DB で、作り直しの後の seq 2、次の id 3 = 消した行の id の使い回し）-> 22 §19 の手順 3・5 で前の値に戻すことにした（M2 で止める）。
- P4 phase と settlement_no の CHECK: `CHECK((phase = 1) = (settlement_no IS NOT NULL))` と `CHECK((reference_type IS 'csv_import') = (phase = 1))` -> NULL の reference_type で phase 0 は ok、csv_import で phase 0 と、phase 1 で settlement_no NULL と、receiving で phase 1 と、phase 0 で settlement_no ありは全部 `CHECK constraint failed`、csv_import・phase 1・settlement_no 0 は ok。(a) は前者だけを置く。
- P5 `stock_counts` の外部キーと、時点証拠の `#[cfg(test)]` の schema の `stocktake_items` の作り直し（`foreign_keys = OFF` で DROP・RENAME）-> ok、作り直しの後も `stock_counts` の外部キーは効く（存在しない明細の id は `FOREIGN KEY constraint failed`）。`count_value` の負は `CHECK constraint failed`。時点証拠の test の DB（vL の後に当てる）は壊れない。
- P6 `chrono::NaiveDate::parse_from_str(_, "%Y-%m-%d")`（0.4.44）と、31 の「業務日の形と暦の検査」の形の検査（長さ 10 バイトの ASCII で `[0-9]{4}-[0-9]{2}-[0-9]{2}`）: `2026-04-07` -> ok、`2026-4-7` -> ok（書き戻すと `2026-04-07`）、` 2026-04-07` -> ok、`+2026-04-07` -> ok、`2026-02-30` -> `input is out of range`、`2026/04/07` -> `input contains invalid characters`、`2026-04-07 ` -> `trailing input`、`02026-04-07` -> `input contains invalid characters`。書き戻しの一致が形を保証しない 2 つ: `-0001-04-07` -> ok（書き戻すと `-0001-04-07`、長さ 11）、`+10000-04-07` -> ok（書き戻すと `+10000-04-07`、長さ 12）。この 2 つは「読んで書き戻した文字列が入力と同じ」を満たすので、書き戻しの一致だけでは 10 文字の形を保証しない。形の検査を先に当てた判定: `2026-04-07` と `0000-01-01` -> ok、`2026-02-30` -> 存在しない日付、`2026-4-7`・` 2026-04-07`・`+2026-04-07`・`2026/04/07`・`2026-04-07 `・`02026-04-07`・`-0001-04-07`・`+10000-04-07`・`2026-ab-07`・空・全角の数字の `２０２６-04-07`（18 バイト）-> 日付の形式が不正。形の合う 90000 個の文字列（年 `0000`・`0001`・`1582`・`1900`・`2000`・`2024`・`2026`・`2100`・`9999` × 月 `00`〜`99` × 日 `00`〜`99`）で、chrono で読めることと SQLite の `date(d) IS d` が全部一致し（食い違い 0）、読めた日付の書き戻しは全部入力と同じ（違い 0）。形を先に見れば、書き戻しの比べは要らない（31 は形と chrono の 2 段だけにした）。今の `sales_service.rs:507` の `validate_date` は `2026-4-7` を通す。
- P7 SQLite の `date()` による暦の検査（22 §19 手順 2）: 同じ scratch の crate（`sqlite_version()` -> `3.45.0`）で `SELECT date(?1), date(?1) = ?1, date(?1) IS ?1, date(?1) IS NOT ?1, NOT (date(?1) = ?1)` -> `2026-02-28` と `2024-02-29` と `0000-01-01` と `9999-12-31` -> (同じ文字列, 1, 1, 0, 0)、`2026-02-30` -> (`2026-03-02`, 0, 0, 1, 1)、`2026-02-32`・`2026-13-01`・`2026/10/01` -> (NULL, NULL, 0, 1, NULL)。表に `2026-02-28`・`2026-02-30`・`2026-02-32`・`2026-13-01` を入れて `WHERE NOT (date(d) = d)` -> `2026-02-30` だけ、`WHERE date(d) IS NOT d` -> `2026-02-30`・`2026-02-32`・`2026-13-01`（`=` の比べは NULL の行を拾わず、`2026-02-32`・`2026-13-01` を通してしまうので、拒む条件は `date(d) IS NOT d`）。`date(NULL) IS NOT NULL` -> 0（参照先の無い行〈d が NULL〉は暦の条件では拾えないので、手順 2 の (i) で止める）。

## Data Safety

- commit しない物: 実 POS / 店舗データ、DB、backup、log、receipt。fixture は全部合成（商品コード・JAN・日付は合成の値）。
- local だけの場所: owner の PC の `%APPDATA%\com.kosei.inventory`（DB と `-wal`・`-shm`、`backups`、`logs`、`images/receipts`、復元の遺物）と `%LOCALAPPDATA%\com.kosei.inventory`（WebView の保存）、アプリを起動する方法ごとの作業フォルダの旧 DB 一式、設定した backup の保存先（`app_settings` の key `backup_path`）の backup の file。
- 作り直し（L3-1 で vL が止まったときだけ。D-123 でリリースの前まで包括で承認済み）: 対象は上の local だけの場所の全部。保全は 22 §16 の作り直しの手順のとおり、日付の付いた 1 つの folder（例 `%USERPROFILE%\inventory-db-moved\2026-10-2x\`）の下に元の場所ごとの sub folder を分けて**消さずに移す**（手順 1 の `backup_path` の控え、手順 2 のアプリの停止、手順 4 の移した元が空の確認を含む）。実行は owner か、owner の指示で Coordinator（WSL の `/mnt/c/Users/...` から移す。消す command を使わない）。移した folder は (b) と原価の lane の作り直しの後も残す（消すかは owner の判断）。
- 合成だけの場所: Rust と TS の test の fixture、L3-2 の画面の入力。

## 残りの危険（起票の時点）

- vL の止まりは、起動の画面の今の `DatabaseInit` の文（「アプリを再起動してもう一度お試しください」を前置きに持つ）に原因と直し方を添えて出す。前置きは決定的な止まり（再起動で直らない）には合わないが、vU・vB と同じで、出会うのは開発者だけ（本番の DB は無く、店の PC へ初めて入れる版は新しい DB で始まる。D-123 (4)）。文の種類を分けるのは別の小さな lane の候補（backlog に足すかは Coordinator）。
- vL の前の版で作った backup のうち、棚卸しか Z004 の記録を持つものを新しい版で復元すると、open の migrate が vL で止まり、復元は巻き戻る（[71](../../function-design/71-mnt-backup.md) §71.7。利用者には失敗に見える。vB と同じ。開発・試験 DB にだけ起きる）。
- 移行の知らせは `integrity_fix` の最新 100 行だけを見る（S6）。移行の後に `integrity_fix` が 100 行を超えると古い移行の行を見つけない（開発・試験 DB にだけ起きる。正本の記録は操作ログに残る）。
- `sales_service` の `validate_date` の形の穴（`2026-4-7` を通す。Contract Probe P6）は、共有の検査へ替えることで同時に塞がる（日次売上の日付の欄は ISO の形で送るので画面の振舞いは変わらない）。レジ処理済みの返品・交換は今は日付を検査せずに保存するので、command を直接呼んで形の違う日付を送る保存は (a) の後に拒まれる（画面は ISO の形で送る）。

## Implementation Results

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

- 結果: D-118 の runtime の (a)（ADR SPEC-STK-CNT-D12）を入れた。migration vL（22 §19）で `inventory_movements` を業務日・phase・settlement_no の列を持つ形に作り直し（消さずに写し、`sqlite_sequence` を前の値に戻す）、`products` に写しの 3 列を足し、`stock_counts` を新設した。棚卸しか Z004 の記録のある DB では何も変えずに止まり、原因と作り直しの手順を名指しする（D-123）。在庫を動かす書込み（入出庫 4 業務・商品登録・商品 CSV の新規行・整合性の補正、test だけが呼ぶ旧本体）は、`products.stock_quantity` を記録から求め直して書く形（`biz::stock_ledger` の `recompute_stock`）に切り替え、計数が無い間は在庫と `stock_after` の値が今と同じ。業務日の形と暦の検査は書く側の 1 か所（31）に置いた。整合性検証は記録から求め直した値と写しの 4 項目を比べ、違う項目を文字で出し、補正は求め直しの結果を書く（36・75）。操作ログ画面は `integrity_fix` の新しい形と記録済みの形の両方を読む（74 UI-11c-D14）。(a) の分の正本の `proposed・未実装` の注記を現行の記述に直した。
- 既存の保護: 停止 ADR の 5 つの入口（棚卸しの開始・保存・確定、Z004 の確定・取消）は変えていない。店の棚卸しと Z004 の業務 commit は停止のまま。
- 残り: (b)（計数の保存・棚卸しの完了・Z004 の確定と取消の新しい経路・停止 ADR の入口の撤去ほか、ADR D12）。
- review・CI・merge（closeout、2026-10-19）: Final Review は broad で、Claude 側（Fable 5.1）= approve（P3 3）、Codex（Astra）= reject（P2 2）。Coordinator が F1（P2、`fix_integrity` の同じ商品の二重の補正）・F2（P2、整合性の突合・補正・差の表示の live な旧契約の文）・F3（P3、計数の前後の循環で誤りで止める前提）を受理して是正し、closure 1・2（Codex Sol）= reject（F2 の残り）の是正を経て closure 3（Codex Sol）= approve（P0）。L3-1・L3-2・Human Visual Confirmation は PASS（owner が Windows native で `npm run tauri dev`）。R4 は実装の前（repo 外の回答台帳 TD-321）と最後の head（TD-323）で owner が承認した。helper の record は review・manual・r4 = pass。hosted の `Merge gate` は Ready の後の run で pass で、owner の Ready・merge の承認（TD-325）の後に helper 経由で squash merge した。Final Review の是正の詳細は PR の本文が正本。PR: [#176](https://github.com/higekiri-w/inventory-system-desktop/pull/176)
- 後続: `docs/backlog.md` の「棚卸しの数の置き場と在庫数の求め直しの runtime」の (b)。

## Review Response

先行 round の結果・評価（判定・件数・採否・reviewer の意見）はこの節にだけ書く。前半の節と遷移記録には「round N の是正（Review Response 参照）」だけを書く（独立 review は `## Review Response` より前だけを読む）。

### Plan Review round 1（2026-10-18、head `f8306208`）

Claude 側 fresh Opus 5.5（reject、P1 0・P2 2・P3 5）と Codex GPT-6 Astra（reject、P1 0・P2 6・P3 2）。どちらも Ordinary Operation は成立と判定した。Claude #1 と Codex #2、Claude #2 と Codex #4、Claude #6 と Codex #6、Claude #7 と Codex #7、Claude #4 と Codex #8 は同じ件。全て採った（Coordinator の裁定）。既存の test の移し方の指摘が重なったので、個別に塞がず Matrix の 1 つの節にまとめさせた（owner の決定 TD-296 の 5a）。起草役が `3523401a` で直した。

| # | 出所 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|
| R1 | Codex #2・Claude #1（P2） | 採る | 版の期待を書き換える一覧に 8 のままが正しい行が入り、棚卸しの行を持つ DB の開き直しと空 DB の列の一覧の test を見落としている | Matrix の「既存の test の移し方」T1 で版の期待を 4 つに分類し、開き直しと列の一覧の test の期待の変え方を決めた |
| R2 | Codex #4・Claude #2（P2） | 採る | test helper が初期在庫の movement を足すと、商品で絞って movement を読む既存の test が落ちる | T5 で数える command と全部の所を分類し、業務の参照先で絞る形にした（値の期待は変えない） |
| R3 | Codex #1（P2） | 採る | vL が業務日の書式だけを検査し、実在しない日付を通す | 22 §19 の手順 2 に暦日の検査を足し、Contract Probe P7 で同梱の SQLite の振舞いを確かめた。M4 に case を足した |
| R4 | Codex #3（P2） | 採る | 棚卸しの test の商品の作り方を「変えない」に分けているが、写しだけで商品を作るので求め直しで値が変わる | T3 で helper の定義を全部分類し、初期在庫の movement を作る対象に入れた。意図して食い違わせる fixture は除外の理由を書いた |
| R5 | Codex #5（P2） | 採る | S1→S7 の各段で test が通らない | S1〜S6 を 1 つの検証の単位にした |
| R6 | Codex #6・Claude #6（P2・P3） | 採る | L3-1 の合格基準が vL の許された補正と矛盾し、前の値を控える手順が無い | 更新の前に控える段を足し、差異の無い商品と補正した商品で基準を分けた |
| R7 | Claude #3（P3） | 採る | vL の手順 2・8 で止まったときに直し方を出さない | 作り直しの案内を足し、M4・M5 で確かめる |
| R8 | Claude #5（P3） | 採る | 移行の知らせを探す `listLogs` の件数が決まっていない | 最新 100 行にし、U8 に移行の行の後に別の行がある case を足した |
| R9 | Codex #7・Claude #7（P3） | 採る | site の数の集計の誤り | hit の数と直す対象の数を分け、同じ測定から表を直した |
| R10 | Codex #8・Claude #4（P3） | 採る | ADR の Context に旧い承認の条件が注記なしで残る | D-123 で置き換えた旨を足した |

起草役の問いの Coordinator の裁定（2026-10-18）: 業務日の DB の CHECK は形だけのまま（暦日は書く側の検査と vL の手順 2 が守る。CHECK に `date()` を入れると、作り直しの起点の判定が SQLite の関数に依る）。開き直しの test は発注どおり棚卸しの行を消して vL の版を確かめる形を採る。ADR の注記の形は起草役のとおり。

### Plan Review round 2（2026-10-18、head `3548771f`）

Claude 側 fresh Opus 5.5（approve、P1 0・P2 0・P3 4）と Codex GPT-6 Astra（reject、P1 0・P2 2）。全て採った（Coordinator の裁定）。日付の検査を個別に塞がず、書く側（31）と移行（22 §19）の各 1 か所に置かせた（owner の決定 TD-296 の 5a）。起草役が `c3f98aa6` で直した。

| # | 出所 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|
| R11 | Codex #1（P2） | 採る | vL の暦日の検査式の NULL の扱いが決まっておらず、暦にない日付が新しい表へ写りうる | 22 §19 の手順 2 を、参照先の行が無い・形が外れる・`date(d) IS NOT d` の 3 つで拒む形にした。Contract Probe P7 で同梱の SQLite の振舞いを測り、M4 に暦にない日付の case を足した |
| R12 | Codex #2（P2） | 採る | chrono の読み書きの一致だけでは `YYYY-MM-DD` の形を保証しない | 31 の業務日の検査を「形（長さと ASCII の数字）→ 暦（chrono）」の 2 段にし、レジ処理済みの返品・交換の保存も同じ検査を通ると書いた。P6 と W の入力に足した |
| R13 | Claude #1（P3） | 採る | 新しい関数が設計書に code block の形で無く、compliance の test が red になる | 21・31 に `rust` の code block のシグネチャを置き、`pub(crate)` で足りるものはそうした |
| R14 | Claude #2（P3） | 採る | demo の初期データの販売の記録の業務日 | 売上ごとの日付を業務日にすると分けて書いた |
| R15 | Claude #3（P3） | 採る | 手順 1 (b) の条件を M4 が試さない | case を足した |
| R16 | Claude #4（P3） | 採る | 手順 9 の失敗に原因と直し方が無い。手順 8 の操作ログの値が決まっていない | 手順 9 に作り直しの案内を足し、手順 8 の書き方と値を決めた |

起草役の問いの Coordinator の裁定（2026-10-18）: demo の初期データの販売の記録は参照先の種類が NULL で、計数が入った後の窓の判定で初期在庫と見分けられない。(a) では計数が無いので値は変わらず、(b) の起票で扱う（(b) の起票の発注に入れる）。計数の一覧の順を id の順にしたこと、`sales_service` の日付の誤りの文の出し分けが変わることは採る。

### Plan Review round 3（2026-10-18、head `9914dd9c`）

Claude 側 fresh Opus 5.5（approve、P1 0・P2 0・P3 4）と Codex GPT-6 Astra（reject、P1 0・P2 1・P3 1）。どちらも Ordinary Operation は成立と判定した。全て採った（Coordinator の裁定）。round の天井で P2 が残ったので owner に諮り、「直して、追加の round なしで plan-approved（R4 の承認は実装の前に別に諮る。跡は Final Review 2 本）」と決まった（repo 外の回答台帳 TD-319）。起草役が `390e29f5` で直した。

| # | 出所 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|
| R17 | Codex #1（P2） | 採る | 補正の操作ログの表示が数量の前後に範囲を片側だけ付ける指定で、75 の前後の 4 項目を満たさず、範囲だけの補正で前の範囲が消える | S6 を 75 を正本として前と後をそれぞれ数量・印・範囲つきで出す形にし、U6 に範囲だけが変わる例で両側の表示を確かめる条件を足した |
| R18 | Codex #2（P3） | 採る | 既存の関数の戻り値の説明の誤り | 単位を含む組に直した |
| R19 | Claude #1（P3） | 採る | demo の seed の写しと和が食い違ったまま残る | seed の終わりに写しを和へ揃える。参照先の種類が NULL の販売の記録の扱いを backlog の (b) に申し送った |
| R20 | Claude #2（P3） | 採る | DB_DESIGN に整合性の古い文が残り、同期の対象に名指しされていない | S7 の対象に名指しで足した |
| R21 | Claude #3（P3） | 採る | L3-1 の合格基準に別の止まりの場合が無い | 止まりの文の写しを返してもらい Coordinator が判定する場合を足した |
| R22 | Claude #4（P3） | 採る | 「変えない」とした test の局所変数名が AC6 に当たる | 局所変数名だけ直し、期待は変えないと書いた |
