# Plan Packet: Z004 の取込みを取り消す入口（結果の画面を離れた後にも誤った版を取り消せるようにする）

wave に属さない単独の lane。導入の前の関門の項目（owner の決定、repo 外の回答台帳 TD-380〈2026-10-22〉で関門の項目になった。起票は TD-402〈2026-10-23〉）。棚卸しの runtime b1（PR #188、branch `agent/stocktake-count-runtime-b`）の上に積む lane で、merge は #188 の後。本 commit は Plan Packet と Test Design Matrix の起票と、b1 が書き換えない節への plan-first の正本の追記だけで、実装は plan-approved の後の別の run。branch は `agent/z004-rollback-entry`（main `eed07d37` から）。新しい決定の番号（予約の D-134）と migration の番号（予約の 22 §24・MNT-03-D22）は使わない（決めたことは各正本の節の ID〈BIZ-03-D2・UI-07-D18・TRACE-D13〉に置き、DB の列を足さない）。

## b1 との関係（stacked、b1 の後に merge）

- b1 の契約の正本は、b1 の Plan Commit `aba17e2e2422695b2ffff6bcd3fd9bebebf24339` と、その後の Gated Amendment 2 本: `5b53294040ff38d2aeb8366aefd02142ffb2c764`（fresh な計数の判定の n と完了の時点の精算の n を取込み ID のカーソル〈`seen_import_id`・`cutoff_import_id`〉から求める。owner の決定、repo 外の回答台帳 TD-400）と `bc0bf2f5d5a6a573bd76cf14e11380d8c407ddbe`（取り込み直した取込みは判定の n の算出に入らず、その精算 k は k ≤ 判定の n なら数に入る。日付で判定するのは、取り込み直した精算が判定の n だった場合だけ。ADR D3・D7）。本 packet の正本の読みは最後の版を基準にした（`git show bc0bf2f5:<path>`）。
- 実装の名前は、b1 の branch の実装の 1 段目（`agent/stocktake-count-runtime-b`、`a7912eea`。起票の途中で push された）で確かめた（`src-tauri/src/biz/csv_import_service/rollback.rs` の `rollback_csv_import`、`src/features/csv-import/components/ResultStep.tsx` の `RollbackResultCard`、`src/features/stocktake/components/RecheckTargetList.tsx`）。b1 の merge の後の版で名前・形が変われば、同期で本 packet・Matrix・正本の追記を当て直す（下の「b1 への依存」の各行が当て直しの対象）。
- 本 lane の branch は main から作り、b1 の merge の後に `origin/main` を単段で取り込む（`git -c merge.directoryRenames=false merge origin/main`。`docs/DEV_WORKFLOW.md` Stacked train）。plan-gate へは Coordinator が進める（b1 の merge の前に Plan Review を回すかは Coordinator が決める。b2〈PR #193〉は owner の決定 TD-402 で前倒しした）。
- 本 commit の正本の追記は、b1 の branch（`40ee8de4` と `a7912eea`）が変える hunk と重ならない位置に、新しい節として置いた（下の「本 commit で変えた正本」。merge の衝突を作らないため。重なる節の書き換えは「b1 の merge の後に同期する節」）。

## 規模と手法

2026-10-23 に本 worktree（HEAD `eed07d37`）で数えた（command は逐語、出力を写した）:

| 区分 | 数 | command と出力 |
|---|---|---|
| 新しい Tauri command | 1（`preview_rollback_csv_import`）。CMD-07 の今の command は 5 | `rg -n 'cmd::csv_import_cmd::' src-tauri/src/lib.rs` → 10 行（Specta の登録〈`export_bindings_to` の `collect_commands!`、316〜320 行〉と runtime の登録〈`run` の `tauri::generate_handler!`、1531〜1535 行〉に 5 つずつ: `parse_and_validate_csv`・`commit_csv_import`・`rollback_csv_import`・`list_csv_imports`・`get_csv_import_record`） |
| 新しい wire の型 | 2（`RollbackPreview`・`AffectedStocktake`）。形の変わる今の型は 1（`CmdError` に任意の field `related_csv_import_id` を 1 つ足す。順 4 の拒否の link のため。下の「起票時に起草役が決めたこと」の 8）。`RollbackResult`・`CsvImportRecordDetail` は変えない | `rg -c '^export type RollbackPreview =' src/lib/bindings.ts` → exit 1（無い）。`rg -c 'related_csv_import_id' src/lib/bindings.ts` → exit 1（無い。2026-10-24、HEAD `f062843c`） |
| 取消を呼ぶ画面の code | 1 file（`commands.rollbackCsvImport` は Z004 の取込みの hook だけ） | `rg -l 'commands.rollbackCsvImport' src --glob '!src/lib/bindings.ts' --glob '!*.test.*'` → `src/features/csv-import/hooks/useCsvImportFlow.ts` |
| destructive の確認 dialog の今の置き場 | 11 file（`AlertDialogAction` を使う production の file） | `rg -l 'AlertDialogAction' src -g '!*.test.*'` → 11（下の Matrix の Adjacent Pattern Audit） |
| 画面 | 3（CSV取込みの詳細〈65〉、Z004 タブの結果の画面と上の導線〈55〉、`/csv-import` の search） | 下の Scope の S3 |
| migration・DB の列 | 0（取消の本体と列は b1。preview は書かない） | — |
| 予定 path | 50（Rust 17・TS 18・docs 15〈packet と Matrix を含む〉。下の「予定 path の一覧」。Gated Amendment 1 で `src-tauri/src/cmd/stocktake_cmd.rs` を、Gated Amendment 3 で `docs/UI_TECH_STACK.md` と `src/hooks/unsaved-changes-guard-sweep.test.ts` を足した） | 下の一覧の行数（`grep -c` で拡張子ごとに数えた） |

手法: **1 lane・1 PR のまま S1〜S4 の段に積む。** 理由: 新しい書く経路は無く（取消の本体は b1）、足すのは読取り 1 つ（取消を TX の中で当てて戻す preview）と、順 4 の拒否の文と詳細への link（error に取込みID の任意の field）、入口・確認・導線の画面だけで、48 path は 1 回の L3（10 分の見込み）で確かめられる（Plan Review round 3 の是正で足した 8 path のうち 4 つ〈`daily_report_import_cmd.rs`・`plu_export_cmd.rs`・`product_cmd.rs`・`settings_cmd.rs`〉は、`CmdError` の literal ごとに None の 1 行を足すだけ）。段は検証の単位で切る（S1 BIZ・IO と S2 CMD・wire は 1 つの Rust の検証の単位、S3 画面は frontend の検証の単位、S4 は docs と生成物）。b1 の後に merge するので、実装は b1 の merge と同期の後に着手する（b1 の取消の本体と `RollbackResult.recheck_targets`・要再確認の一覧の部品に依る）。

## Workflow State

- Phase: implementing
- Risk: R3
- Plan Commit: a87c5601dc6010e086a68dc7f3c38e60faaa95e0
- Amendments: 34343a1e5bef75f7b2ebb0cb04f8225b11d6151f, 47b76883d028a6102ece60557958fd6b2eceebc2, 60e82b42cb31476e74c77c639bd5ebae63ca47eb, 5f02e52cc2f74fb1c4285f235c54caeb18aaa438
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（起票。実装の Writer は plan-approved の後に Coordinator が発注で決める）
- Plan Reviewer: fresh Opus 5.5 + Codex（model は発注時に決める）。座組表どおり
- Final Reviewer: Fable 5（Claude 側、R3 以上）+ Codex（GPT-6.1 Sol 既定）。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Review Minimum: 1
- Human Gate: ready,merge,manual
- Branch: agent/z004-rollback-entry

遷移の記録:

1. kickoff → spec-check（2026-10-23、起草役）: Coordinator の発注（TD-380 の関門の項目、TD-402 の起票）を範囲にし、規模を数えて 1 lane にした（上の「規模と手法」）。Risk を R3 と記録した（下の Risk）。
2. spec-check → design（2026-10-23、起草役）: 対象の設計正本は ADR SPEC-STK-CNT-D2・D3・D6・D7（Gated Amendment の版）、[32](../function-design/32-biz-csv-import-service.md) の取消と §15.5・§15.6a、[41](../function-design/41-cmd-pos.md) §17.5、[55](../function-design/55-ui-csv-import.md) の冒頭の D-118 の表・UI-07-D14・§55.4 の手順 18・§55.9、[65](../function-design/65-inventory-record-traceability.md) §65.5・§65.8.1・§65.10 の 4b、[35](../function-design/35-biz-stocktake-service.md) の完了の後の比較の範囲、design-system の DSR-07・DSR-08・DSR-20。設計の更新が要る点が 3 つあった（取消の前の読取りの関数と wire、CSV取込みの詳細の取消の入口、2 つの入口に共通の確認と導線。下の Design Readiness）。
3. design → plan-draft（2026-10-23、起草役、本 commit）: 上の 3 つを 32・41・55・65 の新しい節（BIZ-03-D2・CMD-07 の command・UI-07-D18・TRACE-D13）に本 packet と同じ commit で置いた。b1 が書き換える節（55 §55.4 の手順 18、65 §65.10 の 4b、32 の順 4 の message ほか）は main の版で先に直さず、b1 の merge の後の同期で直す節として Scope に挙げた。製品の振舞いで owner の判断が要る問いは 3 つ（下の「owner の判断事項」）で、推奨を既定に書き進めた（owner の方針、repo 外の回答台帳 TD-401）。plan-gate へは Coordinator が進める。
4. plan-draft → plan-gate（2026-10-24、Coordinator、本 commit）: owner の判断事項は owner の方針（repo 外の回答台帳 TD-401）で推奨を既定にした（TD-404。後で owner に見せ、答えが変われば plan-gate のまま直す）。packet と Matrix が揃い commit されている。main（`843c4938`、PR #191・#200 の後）を単段で取り込んだ（衝突: なし）。Plan Review round 1（fresh Opus 5.5 + Codex）に出す。
5. plan-gate のまま是正（round 1、2026-10-24、Coordinator、本 commit）: round 1（Claude 側 fresh Opus 5.5 と Codex）の reject を起草役が `fd2c1f8c` で直した（下の Review Response）。設計の方向は変えないので plan-gate に留め、round 2（fresh Opus 5.5 + Codex）で再 review する。
6. plan-gate のまま是正（round 2、2026-10-24、Coordinator の代行、本 commit）: round 2（Claude 側 fresh Opus 5.5 と Codex）の reject を Coordinator の裁定に沿って直した（下の Review Response）。設計の方向は変えないので plan-gate に留め、round 3（fresh Opus 5.5 + Codex。Plan Review round 天井の 3 回目）で再 review する。
7. plan-gate（round 3、天井、2026-10-24、Coordinator の disposition、起草役、本 commit）: round 3 は Claude 側 approve・Codex reject（P1 0）。`docs/DEV_WORKFLOW.md` Review Rules の round 天井に達したので round 4 は回さず、Coordinator が残った findings を「同型指摘の一括是正」とし、起草役がその disposition に沿って直した（下の Review Response）。是正は順 4 の拒否の error に取込みID の任意の field と Alert の link を足す（wire の契約が増え、予定 path が 47 になる）ので、plan-approved は、fresh の reviewer の closure（Claude 側 + Codex）が本 commit の版に P1 / P2 = 0 を報告した後（遷移表の plan-gate → plan-approved の条件。前例: PR #193 の遷移 7・8）。
8. plan-gate → plan-approved（2026-10-24、Coordinator、本 commit）: 条件は `docs/DEV_WORKFLOW.md` の遷移表（99 行）の「Writer でない独立の Plan Reviewer が P1 / P2 = 0」。遷移 7 は closure を Claude 側 + Codex と書いたが、遷移 7 の版の closure（`ff911e61`）で Codex が挙げた P2（N1）は Claude 側も挙げた同じ点で、Coordinator はその後の closure を、直した点だけを見る fresh Opus 5.5 の狭い closure 1 本に絞った（2026-10-24）。closure 2（`432d48c4`）の是正を経て、closure 3（`af32f558`）が approve（P1 / P2 = 0、P3 1。下の Review Response）。closure 3 の P3 1 件は plan-gate のまま Coordinator が `a87c5601` で直した（Plan Commit はその版。前例: PR #201 の遷移 8 で closure の P3 を直した版を Plan Commit にした）。実装は b1（PR #188）の merge の後に `origin/main` を単段で取り込んでから始める（上の「b1 との関係」）。
9. plan-approved → implementing（2026-10-25、Coordinator、本 commit）: 遷移表の唯一の入口。直前に `origin/main`（`8db97271`、#188 b1 の merge を含む）を単段で merge した（`6e622424`、衝突なし）。AC の baseline の command を同期の後に逐語で再測し、変わったのは b1 の merge で本 packet が見込んだ 4 つだけ（`parse.rs` の `既に取り込まれています（精算 {}）` が 1、`RecheckTargetList.tsx` が在り `renderAction?: ` は 0、`bash scripts/check-command-drift.sh` が `D=71 H=71 S=71 T=71`〈b1 の command 3 つ〉、55 §55.0 の `取消の画面入口は結果 step だけ` が 0〈b1 が節ごと外した〉）。どれも上の b1 の head の版の baseline と同じ値で、AC の完了時の期待は変わらない。
10. implementing のまま Gated Amendment 1（2026-10-26、Coordinator、本 commit）: 実装の Writer の S1〜S4 の後の Draft の run（40124364647）が Rust の 1 件だけ red だった。b1 の `src-tauri/src/cmd/stocktake_cmd.rs` の `test_complete_stocktake_cmd_req205_completion_blocked_wire`（b1 Matrix W4）が `CmdError` の wire を JSON 全体の完全一致で照合し、本 lane が足した `related_csv_import_id`（W3 の契約で順 4 の拒否のほかは `null`。`src-tauri/src/cmd/mod.rs` の `CmdError` は `skip_serializing_if` を持たない）が期待に無い。file が予定 path に無かったので Writer は編集前に止まった。起票の Adjacent Pattern Audit の `CmdError` に field を足す行が、wire 全体を照合するこの test を拾っていなかった（`serde_json::to_value(&error)` の完全一致は repo でこの 1 本。相談役の反例探しで確かめた）。本 amendment で予定 path に `stocktake_cmd.rs` を足し（48 path、Rust 17）、所有表に行を足し、Matrix の Adjacent Pattern Audit の行と「既存の test の移し方」の T6 を足した。test の期待 JSON に 1 key を足すだけで、test の名前・REQ-205 の参照・完全一致の照合の強さは変えない（REQ-205 の振舞いは変えず、wire の契約〈41 §17.4〉への期待の追随）。`skip_serializing_if` で key を消す案は W3・U11 (3)・41 §17.4 と食い違い、tauri-specta の command ごとの検証（unified mode）も拒むので採らない。AC は変わらない（AC4 の `cargo test` が覆う。traceability は test の名前と REQ の参照が変わらないので変わらない）。登録は次の commit で本 commit の SHA を `Amendments` に足す。review は Final Review broad に含める（前例: PR #172 の Gated Amendment 1）。
11. implementing のまま Gated Amendment 2（2026-10-27、Coordinator の裁定、文面は起草役〈Opus 5.5〉、`47b76883`）: Final Review broad（head `bf2eae8e`）の 2 本は Claude 側 Fable 5 = approve（P1/P2/P3 0）、Codex 589 = reject（P2 3 / P3 1）で、P2 が 1 件も重ならなかった。相談役（Fable 5）の反例探しで 4 件とも現物で成立し、Coordinator が 4 件とも受理した。P2-1（preview の途中の失敗で TX を明示に閉じない）は 32 §15.6b 手順 5 の今の契約に code を合わせる（`tx.finish()` の形。無条件の `tx.rollback()` は真因を覆う）。P2-2（取消の応答を待つ間に結果の画面・詳細の画面から離脱できる）は、Contract Ledger の除外「55 §55.7（取消と preview は block しない）」を覆した。理由は、PR #172 で日報取込みタブの同型の P2 を受理して止めたので同じ画面の 2 つのタブで振舞いを揃えることと、離脱の後に失敗すると別の画面の Toast だけになり再試行の導線を失うこと（D-127「問題は起きたその画面で名指し」）。タブの切替えは router の遷移でなく blocker が効かないので、取消中は TabsTrigger を disabled にする（相談役の反例）。P2-3（詳細の取消の結果が `importId` に紐づかない）は内側の component を `key={importId}` で作り直す。P3-1 は RB6 に等値境界の fixture を足す。起草役の観察のうち、取込みの応答を待つ間と日報取込みタブの取消の応答を待つ間のタブの切替え（同型の穴）と、param の route の state（`key`・`remountDeps` 0 件）は本 lane の範囲の外で、Matrix の Adjacent Pattern Audit の除外と PR の Follow-up に置く。止めたときに文を出さない点は #172 と同じ扱いにし、残りの危険に記録した。AC の数（5）と予定 path（48）は変わらない。Final Reviewer の Claude 側を座組表の当面 Fable 5（TD-410）に揃えた。broad の後の packet の変更なので、是正の後は新しい head の fresh broad（Minimum の 2 本）を取る。
12. implementing のまま Gated Amendment 3（2026-10-27、Coordinator）: GA2 の是正の Writer が、4 件の是正と commit の後の full の `npm test` で、予定 path の外の既存 test `src/hooks/unsaved-changes-guard-sweep.test.ts` の `T16`（`useBlocker` を直接使う production の file を 3 つの完全一致で固定する。UI-12 / UI-USW-D3）が red になり、編集の前に止まった。GA2 の Adjacent Pattern Audit の `useBlocker(` の行が `-g '!*.test.*'` で test を外して数えたので、test の中に production の path の集合を固定する型を拾わなかった（GA1 の T6 と同じ型の取りこぼし）。相談役（Fable 5）の反例探しで、同じ型の固定の test を test を含めて洗い直し、当たるのは T16 だけ（ほかの sweep の 7 file と Rust の 4 binary は green）と確かめた。あわせて、`useBlocker` の直接使用の例外の正本は UI_TECH_STACK §6.11 UI-USW-D1 の「importing ガード 2 hook のみ」の列挙で、test だけ直すと正本と食い違うこと、GA2 で足した 55 §55.7 の「UI-07-D15 (6) と揃える」が PR #172 で入る節への前方の参照で、本 lane が先に merge すると宙づりになることが分かった。裁定: 予定 path に `docs/UI_TECH_STACK.md` と `src/hooks/unsaved-changes-guard-sweep.test.ts` を足し（50 path）、UI-USW-D1 と T16 の題名は hook を列挙する言い方を 55 §55.7 の持ち主で決める言い方に置き換え（D-127 継ぎ足さない）、T16 の期待の集合に 1 行足す（完全一致は弱めない）。55 §55.7 の前方の参照は外す（上の「Gated Amendment 3 で是正の Writer が直す正本と test」）。Matrix に T8 を足し、Adjacent Pattern Audit の `useBlocker(` の行を test を含めて数える形にした。AC は変わらない（AC4 の full の `npm test` が T16 を覆う）。登録は次の commit で本 commit の SHA を `Amendments` に足す。review は GA2 の是正と同じ fresh broad に含める。
13. implementing のまま Gated Amendment 4（2026-10-27、Coordinator）: fresh broad（head `b55e1d1f`）の Codex 605 の P2（取消の応答を待つ間に native の window の close〈×・Alt+F4〉が止まらない）と P3（55 §55.7 が日報取込みの hook を名指さない）。相談役（Fable 5）が Tauri 2.11.1 の source と公式資料で事実を確かめ、UI-USW-D4 が native close を保証しない・別 backlog と既に書いているので、食い違いは 55 §55.7 の文（既存の「webview は beforeunload を尊重する」と GA2 の「window の close を止める」）の側と分かった。owner の決定（repo 外の回答台帳 TD-458、案 A）で、code は変えずに文を UI-USW-D4 に合わせ、native close は取込み・取消・未保存を横断する backlog の 1 項目にした（backlog は docs の小口 3 の lane が足す）。GA4 の commit は `3803edbc`（起草役: packet・Matrix・55）と `5f02e52c`（Coordinator: 65 §65.5a と hook の comment）。`Amendments` には後者を登録する。packet が変わったので、次は是正の後の head の fresh broad 2 本（Fable 5 + Codex）。

## Owner Effort Budget

- 介入回数上限: 7（既定の 6 を上げた理由: owner の判断事項 3 つは推奨を既定にして書き進め、後で 1 回の問い合わせに束ねても decision point の数で数える〈`docs/DEV_WORKFLOW.md` Owner Effort Budget〉。L3 の 2 round 目の予備に 1）
- 実働時間上限: 30 分（既定。L3 は見込み 10 分）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 7 | 0（2026-10-23） | 6（owner の判断事項 (i)〜(iii) の 3、L3 と画面の確かめ 1、Ready 1、merge 1） | 1（L3 の 2 round 目） | 7 = 0 + 6 + 1 |

owner の操作の回数を減らす工夫: (1) owner の判断事項は推奨つきで返事が記号 1 つの形にし、b1・b2 の L3 か Ready の依頼と同じ問い合わせに束ねてよい（decision point は数える）。答えが推奨と違えば plan-gate のまま Scope を改める（遷移 4。下の各項の「答えで変わる所」）。(2) L3 の入力物は b1 の L3 の fixture の生成（b1 の S9 の `src-tauri/src/io/l3_fixtures.rs`）が作る `products-new.csv`・`z004-0101.csv`・`z004-0102.csv` だけで、本 lane は fixture を足さない。Coordinator が L3 を頼む日に作り、Windows の path を省略せずに渡す（owner は file を作らない）。(3) L3 の DB は Coordinator が b1 の L3 の「差し替え」と同じ手順（22 §16 の作り直しの手順 2〜4 で data の folder を消さずに移す）で空の DB にする（owner は起動と終了だけ）。(4) L3 は 1 本の通しの台本で、各段の PASS / FAIL だけを返してもらい、画面の確かめ（Human Visual Confirmation）を同じ 1 回に含める。(5) Ready と merge は owner が 1 回の返事で「Ready にして hosted が緑なら merge」と委ねられる（decision point は 2 で数える）。

既定値・数え方・上限に届くときの扱いは `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason（行ごとの判定。発注の「R3 か R4」を行ごとに当てた）:

| 変更 | Risk Tiers の行 | 判定 |
|---|---|---|
| CSV取込みの詳細に取消の入口を足す | R3 の「operator workflow」（取消の画面の入口が 1 つから 2 つになる） | R3 |
| 取消の前の読取り `preview_rollback_csv_import`（新しい Tauri command と DTO・bindings） | R3 の「Tauri command DTO」 | R3 |
| preview が取消を TX の中で当てて戻す（BIZ-03-D2） | R4 の「Destructive data lifecycle」に当たるかを確かめた: 何も残さない（明示の rollback。Contract Probe P1）。戻しの失敗は TX の外へ書きを出さない（SQLite の TX の原子性）。当たらない。誤って COMMIT した場合の害は「確認なしの取消」で、取消そのものは論理の無効化（`is_voided`）で行を消さず、正しい版の取り込み直し（ADR D7）で戻せる | R3（Matrix の RB2 が何も書かないことを止める） |
| `/csv-import` の search に `tab` | R3 の「route/search state」 | R3 |
| 順 4 の拒否の error に取込みID（`CmdError` の任意の field、BIZ の variant 1 つ） | R3 の「Tauri command DTO」（今の wire の型に任意の field を足す。kind と今の field は変えない） | R3 |
| 取消の本体（無効化・求め直し） | 変えない（b1。b1 の packet が R4 で扱った） | 対象外 |

R4 の行（破壊的なデータの寿命・backup の復元・実データの露出・secret・不可逆な git の操作）には当たらない: migration も DB の列の追加も無く、取消は今の論理の無効化のままで、本番の DB は無い（`docs/project-memory.md`）。workflow の file（`.github/`・`scripts/`）に触れないので classifier の `workflow=true` にならず Final Review Minimum は 1。operator に見える状態（取消済みの表示と要再確認の一覧）の lifecycle を扱うので、Contract Audit の 2 本目は任意の推奨（`docs/DEV_WORKFLOW.md` Contract Audit の Double audit）として Coordinator が決める。

Rollback / recovery notes: DB を変えないので、merge の後に戻すのは revert の PR で足りる（入口・確認・preview の command が戻る）。取消した取込みは、今と同じく正しい版の取り込み直しで戻る（ADR D7）。

required gate の green / red: gate の定義は変えない。bindings と traceability の再生成を落とすと既存の required gate（bindings・traceability）が red になる（下の Registration / Generation Obligations）。

## Goal

Goal Invariant:

### 最小完了条件

- 取り込んだ後に結果の画面を離れても、入出庫履歴（種別「CSV取込み」）か Z004 タブの「取り込んだ記録を見る」から CSV取込みの詳細を開き、「この取込みを取り消す」で取り消せる（[65](../function-design/65-inventory-record-traceability.md) §65.5a）。取消は今の取消と同じ経路（ADR SPEC-STK-CNT-D7、b1 の新しい経路）で、売上と movement を無効にし、在庫を記録から求め直す。
- 取り消す前の確認が、取り消した後に要再確認である商品（商品・範囲・理由。取り消す前から要再確認の商品を含む）、取り込み直す精算の番号、記録の詳細の差異・ロス原価・参考の差が変わることがある完了した棚卸し（この取込みの精算が完了の後の比較の範囲に入る棚卸し）を名指し、取込みの結果の画面の「取り消す」も同じ確認になる（[55](../function-design/55-ui-csv-import.md) UI-07-D18、[32](../function-design/32-biz-csv-import-service.md) §15.6b）。
- 取り消した後に、その画面で要再確認の一覧（入れ直しへの導線）と、正しい版を取り込む導線（Z004 タブを直接開く）が出る。

### 失敗定義

- 結果の画面を離れた後に、アプリの中で取込みを取り消す操作が無い、または取消済みの取込みに取消の入口が出る。
- 確認が名指す要再確認の商品が、同じ状態で取り消したときの `RollbackResult.recheck_targets` と違う。確認の読取り（preview）が売上・movement・写し・取込みの状態・操作ログのどれかを変える。
- 確認の読取りが終わる前か失敗したときに「取り消す」を押せる（何が起きるかを示さないまま取り消させる）。
- 2 つの入口で確認の中身・取消の後の処理（Toast・再取得・失敗の再試行）が違う。
- 取消の応答を待つ間に、route の移動・リロード等の page の離脱・タブの切替えで画面を離れられる（離れた後に失敗すると、知らせが別の画面の Toast だけになり、同じ画面で再試行できない。D-127）。
- CSV取込みの詳細で、別の取込みの詳細へ移った後に、前の取込みの取消の結果が残る。
- 取消の前の読取り（preview）が、途中の失敗のときに TX を明示に閉じずに返る（drop に頼る）。
- 取消の後に詳細・入出庫履歴・在庫・要再確認の一覧が古いまま残る（D-052 の C9 の再取得が当たらない）。
- 要再確認を色だけで示す。

### 非目的

- 取消の本体（無効化・求め直し・要再確認の判定）を変えること（b1）。日報の取込みの取消の入口（日報には CSV取込みの詳細のような記録の詳細の画面が無い。日報の取消は今の結果の画面のまま）。
- 入庫・返品交換・手動販売・廃棄の業務記録の取消（REQ-208、[65](../function-design/65-inventory-record-traceability.md) §65.7.2。`coverage=deferred`）。
- 専用一覧 `/csv-import/records`（§65.3 の完成形。横断検索で探せるので runway のまま）。
- 取り込み直したときに数え直しが要る見込みの商品を、取り消す前に並べること（下の「owner の判断事項」の (ii)。推奨は並べない）。
- 照合の基準（32 の順 2 の machine_no）を入れ替える操作（b1 の残りの危険。owner の決定 TD-388 で作らない）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

設計を含む変更（operator の操作と command の契約）なので、店の通常の操作列を置く。例の数・日付は説明用の合成の値（下の L3 の台本と同じ形）。本 lane の merge の後に下の列が成り立つ。**この文書を完了できる**ことと、**結果の画面を離れた後に誤った版を取り消して取り込み直す通常運用を達成できる**ことは同じ（b1 の取消の経路に、入口と確認が揃う）。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 精算 101（10-01、商品 B と C の売上）と 102（10-02、商品 A の売上）を取り込み、結果の画面を離れた。A は 10-01 の日付で数 10 を保存した（既定の計数。10-01 に精算 101 があるので 102 は数に入らず、A の在庫 9、要再確認でない） | 101 が別の日の版だったと気づき、Z004 タブの「取り込んだ記録を見る」を押す | 入出庫履歴が種別「CSV取込み」で開き、取込みの行が並ぶ | 行の「詳細」で CSV取込みの詳細へ進める | 65 §65.8.1、55 UI-07-D18 |
| 同じ | 101 の詳細の「この取込みを取り消す」 | 確認に、取込みID・精算日 10-01・「精算 101」・ファイル名・件数・金額、「取り消した後、次の商品は数え直しが必要です（今すでに必要な商品を含みます）」と A・範囲「9〜10」・理由、取り込み直しの文（精算 101）。完了した棚卸しは無いので、その文は出ない。読み終わるまで「取り消す」は押せない | 「取り消す」かキャンセル | ADR D3 の A1（10-01 に精算が無くなると 102 が最初の 1 本で決まらない）。32 §15.6b、Contract Probe P1 |
| 同じ | 「取り消す」 | Toast「取込みを取り消しました」。詳細は「この取込みは取消済みです」になり、取消の入口が消える。同じ画面に取消の結果（取り消した件数、A の要再確認と「棚卸しの画面で入れ直す」）と「商品別CSV取込み（Z004）を開く」 | — | 65 §65.5a、b1 の `RollbackResultCard` |
| 同じ | 「商品別CSV取込み（Z004）を開く」→ 正しい版の 101 を取り込む | Z004 タブが直接開き、取り込める（取り消した取込みは同じ精算の比べる先にならない。ADR D7）。10-01 に精算が戻り 102 は数に入らない側に決まり、A の要再確認が解ける | — | 55 UI-07-D18 の `tab`、32 の順 3〜5 |
| 10-02 の取込みの結果の画面にいる | 「取り消す」 | 上と同じ確認（読取り・名指し・押せる条件が同じ） | — | 55 UI-07-D18 |
| 年末の棚卸しを 12-31 に完了した後、12-30 の取込みが誤った版だったと気づいた | 詳細の「この取込みを取り消す」 | 確認に「完了した棚卸し（12-31）の評価額は変わりません。棚卸しの記録の詳細の「差異」「ロス原価」と「完了の後に入った記録による差」が変わることがあります。」（名指すのは、12-30 の精算が完了の後の比較の範囲に入るから。記録の詳細は差異とロス原価を今 active な精算から求め直す〈b1 の `stocktake_service.rs` の 720 行の `ledger_at_cutoff` と 724 行のロス原価〉。どの数が変わるかは他の精算の判定〈ADR D3〉でも決まるので、確認の文は変わりうると言うだけにする） | 取り消して正しい版を取り込み直す | ADR D6 の非遡及、35 の完了の後の比較の範囲 |
| 精算 101 の正しい版を先に取り込もうとして、preview が「同じ精算と思われる別の資料が既に取り込まれています（精算 101、取込みID: {id}）」で止まった（取消の前） | 拒否の Alert の「取込みID {id} の記録を見る（取り消すときはここから）」 | その取込みの CSV取込みの詳細が開き、2 行目からと同じく取り消して取り込み直せる（詳細の「前の画面へ戻る」は Z004 タブ） | 詳細の「この取込みを取り消す」 | 55 UI-07-D18 の順 4 の拒否からの導線、32 の表の順 4、「起票時に起草役が決めたこと」の 8 |

失敗と再試行: 確認の読取りの失敗は dialog の中の文で、「取り消す」を押せない（閉じて開き直すと読み直す）。取消済みの取込みを古い画面から開いた確認は、BIZ の文（「この取込みは取り消し済みです」）で「取り消す」を押せない。取消の応答を待つ間は、どちらの入口でも画面を移れず（route の移動・リロード等の page の離脱。Z004 タブではタブの切替えも。native の window の close は 55 §55.7 の「止める範囲」のとおり保証しない）、取消の失敗は同じ画面の Toast で、同じ取込みID のまま再試行できる（55 §55.9）。

Plan Review は、この列が「正常な条件で目的を達成できるか」と「危険な結果を出さないか」を別々に答える（`docs/DEV_WORKFLOW.md` Review Rules）。

## owner の判断事項

入口の置き場と取り消せる範囲は正本で一意に決まる（下の「起票時に確かめた 3 つ」）。正本で一意に決まらず、owner の判断が要る問いは次の 3 つ（推奨は candidate）。owner の方針（repo 外の回答台帳 TD-401）で owner に今は聞かず、推奨を既定に書き進めた。

(i) 売上データ取込みの Z004 タブに、過去の取込みへの導線を置くか。

- A（推奨）: Z004 タブの上に「取り込んだ記録を見る（取り消すときはここから）」を置き、入出庫履歴を種別「CSV取込み」で開く（55 UI-07-D18）。理由: 誤った版に気づくのは Z004 の取込みの最中（preview の拒否「同じ精算と思われる別の資料が既に取り込まれています」）か、結果の後に Z004 タブへ戻ったときが多く、その画面から取消の入口へ行ける（AGENTS.md の「使い手に押しつけない」）。部品は既存の link 1 つで、新しい一覧を作らない。
- B: 置かない（入出庫履歴は Sidebar から開ける）。
- C: Z004 タブに取込みの履歴の一覧を足し、行から取り消す（65 §65.8.1 の「行内に取消/訂正ボタンは置かず」と合わず、入出庫履歴と同じ検索を 2 つ持つ）。
- 答えで変わる所: B なら、S3 の Z004 タブの link（`CsvImportPage.tsx` の link。`tab` の search は取消の結果の導線が使うので残す）、Matrix の U8 の link の case、AC3 の `rg -c 'recordType: "csv_import"' src/features/csv-import/CsvImportPage.tsx` とその baseline、55 §55.11 の「取り込んだ記録への導線」の項、65 §65.5a の「到達」の Z004 タブの導線、Goal の最小完了条件の 1 つ目と Ordinary Operation の 1 行目の「取り込んだ記録を見る」（入出庫履歴を Sidebar から開く操作に）、L3 の R-3 と R-8（同じく Sidebar から入出庫履歴を開き、種別「CSV取込み」で絞る操作に）、S4 の SCREEN_DESIGN の同期の行の導線を外す。C なら design へ戻る（65 の改訂が要る）。

(ii) 取り消す前の確認に、取り込み直したときに数え直しが要る見込みの商品を並べるか（ADR D3・D7 の Gated Amendment 2 本の版: 取り込み直した取込みは判定の n の算出に入らず、その精算 k は k ≤ 判定の n なら数に入る。日付で判定されるのは取り込み直した精算が判定の n だった場合〈取り消すと n が下がり k > n になる〉だけで、そのとき精算日が計数の日なら、その精算に行のある商品は要再確認になる。owner の決定 TD-400）。

- A: 並べる（取り消す版の行と、その精算日の計数から見込みを求める。BIZ の読取りが増える）。
- B（推奨）: 並べず、「取り込み直した結果に、数え直しが必要な商品が出ることがあります」と書く。どの商品かは取り込み直した取込みの結果の要再確認の一覧（b1 の `ImportResult.recheck_targets`）で名指す。理由: 見込みは取り消す誤った版の行から求めるので、取り込み直す正しい版の行で外れうる（誤った版と正しい版で売れた商品が違うことが、取り消す理由そのもの）。確かな一覧は取り込み直した結果にあり、その画面で入れ直しへ進める。
- 答えで変わる所: A なら S1 に読取り、`RollbackPreview` に field（32 §15.6b の field と処理ステップ、41 の出力型、Boundary の wire type、AC2 の型の rg）、55 §55.11 の「取り込み直し」の項の文、Matrix の RB と U3 に行が加わり、Goal の非目的と Non-scope と残りの危険の 4 つ目から外す。

(iii) 記録の詳細の差異・ロス原価・参考の差が変わることがある完了した棚卸しを、確認で名指すか（ADR D6 の非遡及。取り消しても完了した評価額は変わらず、記録の詳細の明細の差異・ロス原価と参考の差が変わることがある）。

- A（推奨）: 名指す（`RollbackPreview.affected_stocktakes`、35 の完了の後の比較の範囲の規則で決める）。名指すのは範囲の判定による候補で、記録の詳細のどの数が実際に変わるかは見ない（例: 精算 101・102 が明細の商品を売っておらず、全商品を fresh に数えて 12-31 に完了した後に 101 を取り消すと、101 は範囲に入るので名指すが、差異・ロス原価・参考の差のどれも変わらない）。だから文は「変わることがあります」と書く。理由: 年末の棚卸しの後に年末の取込みの誤りに気づく場面で、取り消すと棚卸しの評価額が変わるのかを利用者が心配せずに済み、記録の詳細のどの数（「差異」「ロス原価」と参考の差）が変わりうるかを確認の画面で知れる。範囲の規則は BIZ-06 の共有の関数を使い、写さない。
- B: 名指さず、確認に常に「完了した棚卸しの評価額は変わりません」と書く（読取りは要らないが、棚卸しの無い店の毎回の確認に要らない文が出る）。
- C: 何も書かない。
- 答えで変わる所: B・C なら、S1 の stocktake_repo の読取りと BIZ-06 の共有の関数の公開、`AffectedStocktake` と `RollbackPreview.affected_stocktakes`（Boundary の wire type、41 の出力型）、AC2 の `rg -c '^export type AffectedStocktake =' src/lib/bindings.ts`、32 §15.6b の `affected_stocktakes` の field と処理ステップの手順 3、65 §65.5a の「確認」の項の名指し、Matrix の RB6 と U3 の棚卸しの case、「b1 の merge の後に同期する節」の 35・20 の行、予定 path の `src-tauri/src/biz/stocktake_service.rs`・`src-tauri/src/db/stocktake_repo.rs`・`docs/function-design/35-biz-stocktake-service.md`・`docs/function-design/20-io-product-repo.md`（20・35 は所有表の行も）、Goal の最小完了条件の 2 つ目と Ordinary Operation の最後の行、Contract Ledger の `affected_stocktakes` の行を外す。55 §55.11 の「完了した棚卸し」の項は、B なら常に出す 1 文（「完了した棚卸しの評価額は変わりません」）に、C なら外す（U3 の期待も同じに）。

店主に新しく聞く必要がある事項: 無い。

## 起票時に確かめた 3 つ（Coordinator が確定した事実と、発注の「正本で一意に決まらなければ」）

1. 入口の場所: **正本で一意に決まる**。[65](../function-design/65-inventory-record-traceability.md) §65.5「取消/訂正 CTA は、`status === "active"` かつ対象記録種別の cancel / correct command が実装済みの場合だけ表示する」と、同節の表の「取消/訂正情報」の行の CSV取込みの列「rollback」、§65.8.1「行内に取消/訂正ボタンは置かず、詳細画面で確認してから行う」。CSV取込みの取消 command は `rollback_csv_import`（実装済み、b1 で新しい経路）なので、§65.5 の条件は CSV取込みの詳細に CTA を出すことを求める。§65.10 の 4b の「rollback CTA は置かず（取込み画面 UI-07 の責務のまま）」は詳細の画面を足したスライスの範囲の記録で、置き場の決定ではない（本 lane の同期で「このスライスでは置かなかった」に直す）。b1 の packet の Non-scope の「正本は決めていない」は、§55.0 の停止の節の「取消の画面入口は結果 step だけ」（b1 が停止の節ごと外す。b1 の `c2ca5aad` で外れた）と 4b の文を読んだもので、§65.5 の条件を当てると一意に決まる。
2. 取り消せる範囲: **正本で一意に決まる**。どの active な取込みでも取り消せる。[32](../function-design/32-biz-csv-import-service.md) §15.5 は指定した ID だけを無効にし（同じ日の他の取込みを残す）、ADR D7 は対象の取込みに条件を置かず、取消の後の求め直しは精算の一覧の今の状態から決まる（ADR D2・D3。直近の取込みかどうかに依らない）。§65.5 の表示条件は「active」（取消済みには出さない）。
3. 確定の後の期間の制限: **正本で一意に決まる**。制限しない。ADR D6 は完了した評価額を後の記録で変えず（非遡及）差を示し、Gated Amendment 2 本目（`bc0bf2f5`）の版の ADR の Consequences は「完了の時点の精算の n の精算を完了の後に取り消して取り込み直すと、…その精算で在庫が動いた明細の記録の詳細の参考の差の、確定の差に出る（評価額と在庫は変わらない。挙動は D6 のまま）」と、完了の後の取消を前提に書く。35 の完了の後の比較の範囲は今の記録（取り消した取込みを除く）から求める。

## 起票時に起草役が決めたこと

設計の値を変えない技術の判断と配布の細部。Coordinator が覆せる。32・41・55・65 の新しい節に置いた（1〜5 と、8 の画面の導線〈55 §55.11〉）か、実装の PR で正本へ置く（6・7 と、8 の error の形〈32・40・41〉）。

1. 取消の前の読取りは、取消の本体を同じ TX の中で当てて明示の rollback で戻す（32 BIZ-03-D2）。理由と棄却案は 32 §15.6b。b1 の `rollback_csv_import`（`a7912eea`）の手順 2〜3（無効化・状態・`recompute_for_settlement_change`・`recheck_targets`）を TX を受ける 1 つの関数に切り出し、取消と preview がそれを呼ぶ（同じ規則を 2 つ持たない）。
2. preview の wire は `RollbackPreview { csv_import_id, settlement_no, recheck_targets, affected_stocktakes }`。取込みID・精算日・ファイル名・件数・金額は呼ぶ側が既に持つ（結果の画面は `ImportResult` と preview の file 名、詳細は `CsvImportRecordDetail`）ので preview に写さない。精算回数は今の 2 つの型に無いので preview が返す（`CsvImport`・`CsvImportRecordDetail` は新しい列を読まない b1 の決定を変えない）。
3. 取消済みの取込みの preview は `ValidationFailed`（「この取込みは取り消し済みです」）にする。取消の本体の冪等の Ok（取消済みには 0 件で成功）は変えない（2 つの画面の間の競合で、押した側を失敗にしないため）。
4. 確認の部品は 1 つ（`src/features/csv-import/components/RollbackConfirmDialog.tsx`）で、取込みの結果の画面と CSV取込みの詳細が使う。取消の mutation（成功の Toast の表示時間 8 秒・D-052 C9 の再取得・失敗の Toast）は `src/features/csv-import/hooks/useRollbackCsvImport.ts` の 1 つにし、`useCsvImportFlow` は成功の後の reducer の dispatch と b1 の `rollbackResult` の保持だけを持つ。
5. Z004 タブを直接開く search `tab`（`daily-report` | `z004`）の値の集合は、feature の `src/features/csv-import/types.ts` が readonly tuple `CSV_IMPORT_TAB_VALUES` と Zod schema `csvImportSearchSchema`（`tab: z.enum(CSV_IMPORT_TAB_VALUES).optional().catch(undefined)`）と型 `CsvImportSearch` で持つ（UI_TECH_STACK UI-STATE-D2〈`docs/UI_TECH_STACK.md` の 316〜320 行〉。同じ形の先例は `src/features/daily-sales/types.ts` の `dailySalesSearchSchema` と `src/routes/reports/daily.tsx` の import）。`src/routes/csv-import/index.tsx` はその schema を import して `validateSearch` に渡すだけで、値の集合を持たない。`CsvImportPage` の `Tabs` の `value` と取消の結果の link の `tab` も同じ tuple・型から取る。`tab` は `Tabs` の `defaultValue` にだけ使う（初期選択だけ。タブの切替で URL を書き換えず、同じ mount の間に戻る・進むで search が変わってもタブの状態を保つ）。route の新設は無い。
6. 32 の順 4 の拒否の文に取込みID を足す（「同じ精算と思われる別の資料が既に取り込まれています（精算 {n}、取込みID: {id}）」。取込みID の書き方は順 2 の文と同じ）。理由: 誤った版に気づくのはこの拒否のときが多く、名指した取込みの詳細へその場で進める（下の 8 の link。順 2 の拒否の文は取込みID を名指す。b1 の closure の是正と同じ形）。文を作るのは b1 の `src-tauri/src/biz/csv_import_service/parse.rs` の `check_settlement_rules`（`c667d52c` の 95〜98 行は「精算 {}」だけ）で、preview（`parse_and_validate`）と確定の TX の取り直しの両方がこの関数を通る。名指す ID は `sales_repo::find_active_imports_for_settlement`（`c667d52c` の 328 行。`(id, file_hash)` を id の昇順で返す）の先頭の行の id（順 4 があるので同じ精算の active な取込みは通常 1 つ）。b1 の file と b1 が持つ表の文なので、b1 の merge の後に本 lane の S1 で code・既存 test の期待・32 の表を直す（Scope の S1、Matrix の RB7）。
7. preview の query key は `queryKeys.inventoryRecords.csvImportRollbackPreview(importId)`（`inventoryRecords.root()` の下）。D-052 の C9（`csvImportRollback`）は `inventoryRecords.root()` を含むので、SSOT の entry を変えずに取消の後に残らない（`src/lib/invalidation-contract.ts` は変えない）。
8. 順 4 の拒否の error は、文が名指す取込みID を構造でも持ち、拒否の Alert から CSV取込みの詳細へ進める（Plan Review round 3 の是正。55 UI-07-D18 の「順 4 の拒否からの導線」）。形: BIZ に variant `BizError::SettlementConflict { message: String, csv_import_id: i64 }` を足し（`src-tauri/src/biz/mod.rs`。返すのは `check_settlement_rules` の順 4 だけで、message は 6 の文、`csv_import_id` は同じ id）、CMD の変換（`From<BizError> for CmdError`、`src-tauri/src/cmd/mod.rs`）は kind を今の `import_error` のまま、`CmdError` に足す field `related_csv_import_id: Option<i64>` に入れる（ほかの variant と、CMD が literal で作る error は None）。TS では任意の field（`related_csv_import_id?: number | null`）にする。付け方（`#[serde(default)]` か `#[specta(optional)]`）は実装で選ぶ（`#[serde(default)]` の field が bindings で `?:` になる先例: `c667d52c` の `src-tauri/src/biz/product_service.rs` の 1404〜1407 行と `src/lib/bindings.ts` の 856〜857 行の `plu_target?`・`warnings?`）。画面は `src/features/csv-import/components/ErrorState.tsx` の Alert に、field が数のときだけ（`!= null` で判定し、0 を落とさない）「取込みID {id} の記録を見る（取り消すときはここから）」の link（`/csv-import/records/{id}`、search の `returnTo` は `/csv-import?tab=z004`。DSR-18）を置く。preview（`parse_and_validate_csv`）と確定の取り直し（`commit_csv_import`）の拒否はどちらも `useCsvImportFlow` の `onError` から同じ `ErrorState` に出る（`c667d52c` の `useCsvImportFlow.ts` の 89・116 行）。
   最小の形を選んだ理由（今の error の型と wire を `c667d52c` で読んだ）: kind を変えないので、`decideRecoverTo`（`import_error` → idle）・`ErrorState` の見出し（`titleForKind`）・確定の token を拒否で残す扱い（variant や message で分岐しない。`csv_import_cmd.rs` の `commit_csv_import`）は今のまま。TS で任意にするので、`CmdError` を literal で書く TS の file（`git grep -l 'stocktake_recovery' -- 'src/**'` で 42。bindings を含む）と `src/lib/invoke.ts` の `toCmdError` は変えない（b1 が必須の field `stocktake_recovery` を足したときは、この 42 file が変わった）。Rust は struct の field の省略を許さないので、`CmdError` を literal で作る 23 か所（`git grep -c -E 'error_id: (None|Some)'` の和。`cmd/mod.rs` 11・`daily_report_import_cmd.rs` 5・`csv_import_cmd.rs` 4・`plu_export_cmd.rs`・`product_cmd.rs`・`settings_cmd.rs` 1 ずつ）に `related_csv_import_id: None` を足す（中身の変更は無い）。棄却: (a) 画面が message の文から ID を読む（文を変えると黙って壊れ、型で守れない。Matrix の U11 が止める。id を文から読む形は (2)・(5)、出すかを文で決める形は (3)〜(6)）、(b) 今の `field`（バリデーションの field 名）に ID を入れる（意味が違う）、(c) TS でも必須の field にする（42 file の mock が変わり、本 lane の範囲を超える）、(d) 新しい kind（`decideRecoverTo`・`titleForKind`・D-061 の `CMD_ERROR_KIND` の分岐が増える）、(e) `BizError::ImportError(String)` を struct に変える（`ImportError` を作る・照合する全ての所が変わる）。ID を構造で持つのは順 4 だけで、順 2（取り消した取込みとレジの番号の食い違い）・順 3（同じ file の重複）の文の取込みID は今の文のまま（field は None。下の Non-scope）。

## b1 への依存

b1 の Plan Commit と Gated Amendment の版の名前で書き、b1 の `a7912eea`で確かめた名前を添える。b1 の merge の後の版で名前・形が変われば、同期で本 packet・Matrix・正本の追記を当て直す。

1. 取消の本体: b1 の `rollback_csv_import`（32 の取消、b1 の S5）。売上と movement の無効化、取込みの状態、ADR D2 の求め直す商品（`a7912eea` では `biz::stock_ledger::recompute_for_settlement_change`）、要再確認の一覧（`a7912eea` では `biz::stocktake_service::recheck_targets`）。`c667d52c` の `rollback.rs` の 51〜55 行は求め直した全ての商品（計数を持つ全ての商品とこの取込みの商品）を `recheck_targets` に通し、`stocktake_service.rs` の 372 行の `recheck_targets` は写しが要再確認の商品を全部返すので、一覧には取り消す前から要再確認の商品も入る（確認の文はこれに合わせて因果を言わない。55 UI-07-D18）。本 lane はこれを TX を受ける関数に切り出すだけで、規則を変えない。
2. `RollbackResult.recheck_targets` と `CountRecoveryTarget`（40、b1 の S7）。preview の `recheck_targets` は同じ型。
3. 要再確認の一覧の部品: `src/features/stocktake/components/RecheckTargetList.tsx`（`c667d52c` の 25 行で `renderAction` は必須。36 行の「棚の数を確かめて入れ直してください。」と 43 行の「操作」列を常に出す。呼出し元は `ResultStep.tsx` の `RecheckTargets` と `StocktakePage.tsx` の 2 つ）と、`src/features/csv-import/components/ResultStep.tsx` の `RollbackResultCard`（取消の結果）。確認の中の一覧のために、本 lane は b1 の file の `RecheckTargetList` を変え、`renderAction` を任意にして、省略したときは「操作」列と入れ直しの文を出さない（S3。2 つの呼出し元は `renderAction` を渡したままで、表示と操作を変えない）。詳細の取消の結果は `RollbackResultCard` をそのまま使う。
4. 精算回数: 24 の「active な Z004 の精算の一覧」（import ID・`settlement_no`。b1 の S2）。
5. 完了の後の比較の範囲: 35 の `get_stocktake_record` の範囲の規則と、完了の時点のカーソル（Gated Amendment の版の `cutoff_import_id`・完了日）。BIZ-06 の中の判定を `pub(crate)` の関数にして BIZ-03 から呼ぶ（規則を写さない）。完了した棚卸しの一覧の読取りは stocktake_repo に足す（20 に置く。b1 が触る 20 の D-118 の節の外）。
6. 取消の再取得: D-052 の `csvImportRollback`（b1 の S8 で要再確認の一覧の key を足した版）。本 lane は entry を変えない。
7. 取込みの結果の画面: b1 の S8 が `ResultStep.tsx`・`useCsvImportFlow.ts`（`rollbackResult` の保持）・`CsvImportPage.tsx`（停止の Alert を外し、取消の結果を出す）を変える。本 lane は確認の中身と mutation の置き場だけを変える。
8. `CmdError` の形: b1 が `stocktake_recovery` を足す。本 lane はその版に任意の field `related_csv_import_id` を足す（「起票時に起草役が決めたこと」の 8）。TS の mock は同期の後の形に合わせる（足す field は任意なので、今の mock に書き足さない）。
9. 生成物: `src/lib/bindings.ts` と `docs/function-design/90-traceability.md` は b1 の merge の後に本 lane の head で再生成する（手で解かない。D-098）。
10. L3 の fixture: b1 の S9 の `src-tauri/src/io/l3_fixtures.rs` が作る `products-new.csv`・`z004-0101.csv`・`z004-0102.csv`（b1 の packet の「L3 の fixture の表」）。

## この変更で不要になるもの

- 記述（b1 の merge の後の同期で置き換える）: [65](../function-design/65-inventory-record-traceability.md) §65.10 の 4b の「rollback CTA は置かず（取込み画面 UI-07 の責務のまま）」（「このスライスでは置かなかった。取消の入口は §65.5a」に）。[ui-task-specs](../architecture/ui-task-specs.md) の UI-07 の「Z004商品別CSV完了取消し → CMD-07 rollback_csv_import（既存トラック）」（入口 2 つと preview に）。[55](../function-design/55-ui-csv-import.md) §55.4 の手順 18 の確認の文（UI-07-D18 を引く形に）。32・41・55・65 の本 commit の新しい節の「proposed・未実装」。
- code: `src/features/csv-import/hooks/useCsvImportFlow.ts` の `rollbackMutation` の本体（Toast・再取得・失敗の Toast）を共有の hook へ移して外す。`src/features/csv-import/components/ResultStep.tsx` の確認の `AlertDialog` の中身（今の文と dl）を共有の確認へ置き換えて外す。`src-tauri/src/biz/csv_import_service/parse.rs` の順 4 の文の「（精算 {}）」だけの形と `BizError::ImportError` で返す形、それを固定の文と `ImportError` の照合で見る既存の test の期待 3 件（取込みID を入れた文と `BizError::SettlementConflict` の照合へ置き換える。Matrix の T5）。
- 記述と code（Gated Amendment 2）: 55 §55.7 の「`result` / `error` / `idle` / `parsing` / `preview` の各 state では block しない」の列挙（取消の応答を待つ間の `result` を止めるので、止めない state の書き分けに）と、本 packet の Contract Ledger の除外にあった 55 §55.7 の行（取消の応答を待つ間を除外していた。本 GA で外した）。`src-tauri/src/biz/csv_import_service/rollback.rs` の preview の「4〜5. … 明示の rollback で戻す（drop に頼らない。…）」の comment と `apply_rollback(&tx, csv_import_id)?` の早期 return（手順 4 の成否によらず閉じる形の comment と code に）。
- 記述（Gated Amendment 4）: 55 §55.7 の「Tauri webview は beforeunload を尊重する」と、取消の応答を待つ間の止めを window の close まで止めると書いた文（55 §55.7・§55.11、本 packet、Matrix）。native の window の close は UI-USW-D4 のとおり保証しないので、止める範囲は route の移動とリロード等の page の離脱に直した。
- 確かめるだけ（b1 が外す）: 55 §55.0 の「取消の画面入口は結果 step だけ」（b1 が停止の節ごと外す。AC5 の `rg` で 0 を確かめる）。
- 不要にならないと確かめたもの: 取消の本体の冪等の Ok（2 つの入口の間の競合で要る。上の「起票時に起草役が決めたこと」の 3）、b1 の `RollbackResultCard`（詳細でもそのまま使う）、§65.8.1 の「行内に取消/訂正ボタンは置かず」（入口を詳細に置く根拠）、日報の取消の確認（`src/features/daily-report-import/DailyReportImportPage.tsx`。日報は在庫を動かさず要再確認が出ないので、Z004 の確認の部品に寄せない）、`useCsvImportFlow` の importing の間の `useBlocker`（取込みの応答を待つ間の止めとして残る。取消の止めは共有の hook に別に置く）。

## Scope

本 lane が変える file と目的。実装は S1 → S4 の順に積む。S1〜S2 は 1 つの Rust の検証の単位（S2 の wire の変更は S3 の画面と bindings で揃う）、S3 は frontend の検証の単位、S4 は docs と生成物。既存の test の移し方は Matrix の「既存の test の移し方」（T1〜T5）を引く。

### 本 commit で変えた正本（plan-first）

- [32](../function-design/32-biz-csv-import-service.md) §15.6b（新設、BIZ-03-D2）: `preview_rollback_csv_import` の関数要求・シグネチャ・`RollbackPreview`・処理ステップ・設計判断（同じ TX で当てて戻す理由と棄却案）。
- [41](../function-design/41-cmd-pos.md) §17.5 の `preview_rollback_csv_import`（新設）: CMD の薄い command と error の kind。
- [55](../function-design/55-ui-csv-import.md) §55.11（新設、UI-07-D18）: 2 つの入口に共通の確認の中身・押せる条件・読取りの cache の扱い、取消の mutation の共有、Z004 タブの「取り込んだ記録を見る」、`/csv-import` の `tab`（初期選択だけ。値の集合は feature の `types.ts` が持つ。UI-STATE-D2）。
- [65](../function-design/65-inventory-record-traceability.md) §65.5a（新設、TRACE-D13）: CSV取込みの詳細の取消の入口（表示条件・確認・成功と失敗・到達・理由と棄却案）。

### b1 の merge の後に同期する節（本 commit では直さない）

b1 の branch が書き換えた節か、その隣の節。本 lane の実装の PR で、b1 の merge の後の版を起点に直す。

- [65](../function-design/65-inventory-record-traceability.md): §65.10 の 4b の文（上の「この変更で不要になるもの」。b1 の `a7912eea` の hunk が 4b の 2 行後から始まる）、§65.5a の「proposed・未実装」。
- [55](../function-design/55-ui-csv-import.md): §55.4 の手順 18（b1 の `a7912eea` が手順 15〜19 の付近を書き換える）を UI-07-D18 を引く形に。§55.1 の module の表に `RollbackConfirmDialog.tsx`・`useRollbackCsvImport.ts`。§55.3 の `rollbackMutation` の行（共有の hook）。§55.5 の kind 別表示の `import_error` の行（順 4 の拒否の link。§55.11 を引く）。更新履歴の行。§55.11 の「proposed・未実装」。
- [32](../function-design/32-biz-csv-import-service.md): 冒頭の D-118 の節の表の順 4 の message と、拒否が名指す取込みID を構造で持つこと（上の「起票時に起草役が決めたこと」の 6・8）、§15.1 の module の表、§15.6b の「proposed・未実装」。
- [41](../function-design/41-cmd-pos.md): §17.4 の変換表の `SettlementConflict` の行（kind `import_error`、`related_csv_import_id`）、§17.9 の登録の一覧、`preview_rollback_csv_import` の「proposed・未実装」、更新履歴の行。
- [40](../function-design/40-cmd-product.md): §5.3 の `CmdError` の field に `related_csv_import_id`（任意。`SettlementConflict` だけ Some。b1 が同じ節に `stocktake_recovery` を足す）。
- [35](../function-design/35-biz-stocktake-service.md): 完了の後の比較の範囲の判定を BIZ-03 が呼ぶ共有の関数として書く（規則は変えない。b1 への依存の 5）。
- [20](../function-design/20-io-product-repo.md): stocktake_repo の完了した棚卸しの一覧の読取り（id・`completed_at`・完了の時点のカーソル）。
- [SCREEN_DESIGN](../SCREEN_DESIGN.md): 売上データ取込み画面の節に Z004 タブの「取り込んだ記録を見る」と、取消の確認の名指し（b1 が同じ節の停止の行を外す）。
- [FUNCTION_DESIGN](../FUNCTION_DESIGN.md)・[cmd-task-specs](../architecture/cmd-task-specs.md)・[ui-task-specs](../architecture/ui-task-specs.md): BIZ-03 の関数の一覧、CMD-07 の command の表の行、UI-07 の取消の行。

### Gated Amendment 2 で是正の Writer が直す正本の節

Final Review broad（head `bf2eae8e`）の差し戻しの是正。本 GA の commit は packet と Matrix だけで、下の正本は是正の Writer が code と同じ PR で直す（32 は契約を変えないので直さない）。

- [55](../function-design/55-ui-csv-import.md) §55.7「`useBlocker` による importing 中の常時 block」の節: Z004 タブは importing の間に加えて、取消の応答を待つ間（共有の hook `useRollbackCsvImport` の `mutation.isPending`）も route の移動とリロード等の page の離脱を止める（`useBlocker`・`enableBeforeUnload`。hook の中の 1 か所で、CSV取込みの詳細にも同じく効く）。タブの切替えは router の遷移でなく blocker が効かないので、取消の応答を待つ間は 2 つの `TabsTrigger` を disabled にする。解除の行に「取消の応答の後（成功・失敗とも）に解く」を足し、「`result` / `error` / `idle` / `parsing` / `preview` の各 state では block しない」の行を、止めない state の書き分け（取消の応答を待っていない `result` と、`error`・`idle`・`parsing`・`preview`）に書き直す。この行は PR #172 も変える（下の所有表）。
- [55](../function-design/55-ui-csv-import.md) §55.11 の「取消の実行と後」: 上の止めを 2 つの入口に共通の処理として足す（取消が失敗すれば止めを解き、元の画面のまま同じ取込みID で再試行できる）。
- [65](../function-design/65-inventory-record-traceability.md) §65.5a: 「失敗」の項に取消の応答を待つ間の止め（55 §55.7・§55.11 を引く）、「成功」の項に、取消の結果は表示している取込みのものだけで、別の取込みの詳細へ移ると出さないこと。
- 55・65 の更新履歴の行。

### Gated Amendment 3 で是正の Writer が直す正本と test

GA2 の是正の Writer が、予定 path の外の既存 test で止まった（下の遷移の記録 12）。本 GA の commit は packet と Matrix だけで、下の正本と test は是正の Writer が同じ PR で直す。

- `docs/UI_TECH_STACK.md` §6.11 の UI-USW-D1: 「画面側での `useBlocker` 直接使用は禁止し、sweep test で機械強制する（既存例外 = CSV 取込み / 日報取込みの importing ガード 2 hook のみ）」の括弧の列挙を、持ち主で決める言い方に置き換える: 例外は 55 §55.7 が持つ進行中の処理のガードの hook（取込みの応答と、取消の応答を待つ間）だけで、どの hook かは 55 §55.7 が名指す。hook の名や数を UI_TECH_STACK に並べない（hook を足すたびに列挙を継ぎ足さない。D-127）。UI-USW-D3 の除外 (a)「進行中処理ガード既設（§55.7 所有）」は今の文のままで合う。更新履歴の行を足す。
- `src/hooks/unsaved-changes-guard-sweep.test.ts` の `T16`: 期待の集合（完全一致）に `src/features/csv-import/hooks/useRollbackCsvImport.ts` を 1 行足す（照合の形と強さは変えない）。test の題名の「既存2 hook」を UI-USW-D1 と同じ持ち主の言い方（55 §55.7 の進行中の処理のガードの hook と共通警告 hook）に直す。traceability は describe の `UI-12` の token だけを見るので変わらない。
- 55 §55.7 の GA2 で足した段落の末尾「日報取込みタブの取消の止め（UI-07-D15 (6)）と揃える。」を外す。UI-07-D15 (6) は PR #172 が足す節で、本 branch には無い（本 lane が先に merge すると宙づりの参照が main に入る）。#172 との相互の参照は、後に merge する方が足す（下の所有表の 55 の行）。

### Gated Amendment 4 で直した正本の節

Final Review broad（head `b55e1d1f`）の差し戻し（Codex 605 の P2・P3）の是正。GA2・GA3 と違い、本 GA の commit が packet・Matrix と 55 を直す。code と test は変えない（今の `useRollbackCsvImport` の `useBlocker` + `enableBeforeUnload` が直した契約に合う）。

- [55](../function-design/55-ui-csv-import.md) §55.7 の importing の段落の「止める範囲」: route の移動（`useBlocker`）とリロード等の page の離脱（`enableBeforeUnload`）。native の window の close（×・Alt+F4）は [UI_TECH_STACK](../UI_TECH_STACK.md) UI-USW-D4 のとおり保証しない（Tauri 2.11.1 の native の close は `WindowEvent::CloseRequested` の経路で、webview の `beforeunload` を経ない）。DB は取消の単一の transaction（`src-tauri/src/biz/csv_import_service/rollback.rs` の `rollback_csv_import`）で守られ、失うのは結果の通知だけ。取消の応答を待つ間の段落（UI-07-D18）と §55.11 の「取消の実行と後」は、この範囲を引く。
- 55 §55.7 の importing の段落: 本節が名指す進行中の処理のガードの hook に `useDailyReportImportFlow` を足し、3 つ（`useCsvImportFlow`・`useDailyReportImportFlow`・`useRollbackCsvImport`）と書いた（UI-USW-D1 が 55 §55.7 に委ねる名指し。T16 の期待の 4 件から共通警告 hook の `useUnsavedChangesWarning` を除いた 3 つと同じ）。#172 との重なりは下の所有表の 55 §55.7 の行。
- 本 packet と Matrix の、取消の止めが window の close を止めると書いた文を、止める範囲の言い方に直した。Matrix の U12・U13 は native の close を止める test を求めていない（U12 は `enableBeforeUnload` に渡す値、U13 は push の止め）ので、test は足さない。
- 同じ前提の文が残っていた [65](../function-design/65-inventory-record-traceability.md) §65.5a の「失敗」の項と `src/features/csv-import/hooks/useRollbackCsvImport.ts` の comment も、同じ止める範囲の言い方に直した（Coordinator、GA4 の 2 つ目の commit。code の振舞いは変えない）。

### S1 BIZ と IO（`csv_import_service`・`stocktake_service`・`stocktake_repo`）

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| 取消の本体の切り出し | b1 の `rollback_csv_import` の TX の中の手順（無効化・状態・求め直し・`recheck_targets`）を、`&Transaction` を受ける `pub(crate)` の関数 1 つに切り出す。`rollback_csv_import` はそれを呼んで COMMIT と操作ログ（今のまま） | `src-tauri/src/biz/csv_import_service/rollback.rs` | Matrix の T1（今の取消の test は期待を変えない） |
| preview | `pub fn preview_rollback_csv_import(conn: &mut DbConnection, csv_import_id: i64) -> Result<RollbackPreview, BizError>`（32 §15.6b の手順 1〜6。手順 4 は上の切り出した関数。手順 5 は手順 4 の成否によらず `tx.finish()` で明示に閉じる: `let applied = apply_rollback(&tx, csv_import_id); tx.finish().map_err(DbError::from)?; applied?` の形。`conn.transaction()` の既定の `DropBehavior::Rollback` で `finish` は ROLLBACK を出して error を返し、SQLite が自動で戻した TX では autocommit を見て何もしないので、閉じる失敗を先に返し、閉じられたときは手順 4 の失敗〈真因〉をそのまま返す〈rusqlite 0.31.0 の `src/transaction.rs` の 116〜129 行の `new_unchecked` と 214〜228 行の `finish_` で確かめた〉。無条件の `tx.rollback()` は自動で戻った TX に「no transaction is active」を返して真因を覆うので採らない。Gated Amendment 2） | `rollback.rs`、`src-tauri/src/biz/csv_import_service/mod.rs`（`RollbackPreview`・`AffectedStocktake` の型と re-export） | Matrix の RB1〜RB6 |
| 記録の詳細の差異・ロス原価・参考の差が変わることがある完了した棚卸し | 35 の完了の後の比較の範囲の判定（その取込みの精算が範囲に入るか。範囲による候補で、どの数が実際に変わるかは見ない）を `pub(crate)` の関数にし、完了した棚卸しの一覧の読取りを足す | `src-tauri/src/biz/stocktake_service.rs`、`src-tauri/src/db/stocktake_repo.rs` | Matrix の RB6 |
| test | Matrix の RB1〜RB6 | `src-tauri/src/biz/csv_import_service/tests/rollback_tests.rs` | 既存の test の fixture（b1 の `make_z004_bytes` ほか）を使う |
| 順 4 の拒否の文と variant（b1 の file を本 lane で変える） | `check_settlement_rules` の順 4 の文を「同じ精算と思われる別の資料が既に取り込まれています（精算 {n}、取込みID: {id}）」にし、`BizError::SettlementConflict { message, csv_import_id }` で返す（`find_active_imports_for_settlement` の先頭の行の id。上の「起票時に起草役が決めたこと」の 6・8）。variant と `Display` は `src-tauri/src/biz/mod.rs` に足す。3 test は message と `csv_import_id` の両方を見る。既存の test の期待を、固定の文から実際の取込みID を入れた文へ当て直し、3 test とも順 4 が名指す取込みの ID が 1 以外になるように、その取込みの前に別の取込みを 1 件入れる（b1 の 3 test は名指す取込みが DB の最初の取込み〈ID 1〉なので、ID を固定値 1 にする誤実装が通る。形は Matrix の RB7）（preview の経路: `parse_tests.rs` の `test_parse_req401_rejects_other_version_of_same_settlement_and_allows_after_rollback`〈`c667d52c` の 664 行〉。確定の経路: `commit_tests.rs` の `test_commit_req401_rejects_shared_or_newly_matched_jan_and_rechecks_settlement_from_cache`〈682 行〉と `csv_import_cmd.rs` の `test_csv_cmd_req401_commit_keeps_token_on_rejection_and_removes_on_success`〈341 行〉） | `src-tauri/src/biz/csv_import_service/parse.rs`、`src-tauri/src/biz/mod.rs`、`tests/parse_tests.rs`・`tests/commit_tests.rs`、`src-tauri/src/cmd/csv_import_cmd.rs` | Matrix の RB7（契約の変更: 32 の表の順 4。期待を変える理由は Matrix の T5） |

### S2 CMD と wire

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| command | `preview_rollback_csv_import(state, csv_import_id)`（41。`#[tauri::command]` と `#[specta::specta]` の対） | `src-tauri/src/cmd/csv_import_cmd.rs` | Matrix の W1 |
| 登録 | Specta の登録（`export_bindings_to` の `collect_commands!`）と runtime の登録（`run` の `tauri::generate_handler!`）に 1 つずつ | `src-tauri/src/lib.rs` | Matrix の W2（`scripts/check-command-drift.sh`）、AC2 |
| 拒否の error の取込みID | `CmdError` に `related_csv_import_id: Option<i64>`（TS で任意）。`From<BizError>` で `SettlementConflict` を kind `import_error`・`related_csv_import_id: Some(id)` に。CMD が `CmdError` を literal で作る 23 か所に `related_csv_import_id: None`（「起票時に起草役が決めたこと」の 8） | `src-tauri/src/cmd/mod.rs`、`src-tauri/src/cmd/csv_import_cmd.rs`・`daily_report_import_cmd.rs`・`plu_export_cmd.rs`・`product_cmd.rs`・`settings_cmd.rs`（literal ごとに None の 1 行だけ） | Matrix の W3、AC2 |
| bindings | `cd src-tauri && cargo run --bin generate_bindings` で再生成 | `src/lib/bindings.ts`（生成物） | AC2 |

### S3 画面

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| 確認の部品 | 55 UI-07-D18 の確認（開いたときに preview を読む useQuery〈`enabled` は開いている間、`staleTime: 0`・`gcTime: 0`・`retry: 0`〉、対象の項目と精算回数、要再確認の一覧〈下の行で `renderAction` を任意にした `RecheckTargetList` を `renderAction` を渡さずに〉か無いときの文、取り込み直しの文、完了した棚卸しの文、読取りの間と失敗と取消済みのときに「取り消す」を押せない、DSR-20 の destructive の配置） | `src/features/csv-import/components/RollbackConfirmDialog.tsx`（新設） | Matrix の U2〜U4・U10 |
| 要再確認の一覧の部品（b1 の file を本 lane で変える） | `RecheckTargetListProps.renderAction` を任意にする。省略したときは「操作」列（見出しと各行の cell）と「棚の数を確かめて入れ直してください。」を出さない（確認の中では入れ直しへ進めないため。55 UI-07-D18）。渡したときの表示は今のまま。今の 2 つの呼出し元（`ResultStep.tsx` の `RecheckTargets`、`StocktakePage.tsx`）は変えない | `src/features/stocktake/components/RecheckTargetList.tsx` | Matrix の U3（省略の形）。渡す形の回帰は b1 の今の test（`ResultStep.test.tsx` の 143〜148 行の入れ直しの文と link、`StocktakePage.test.tsx` の 1667〜1671 行の「入れ直す」）を期待を変えずに通す |
| 取消の mutation | `useRollbackCsvImport({ onSuccess })`: `commands.rollbackCsvImport`、成功の Toast（8 秒）、`invalidateByContract(queryClient, invalidationContract.csvImportRollback())`、失敗の Toast。取消の応答を待つ間（`mutation.isPending`）の離脱の止めを hook の中の 1 か所に置く: `useBlocker({ shouldBlockFn: () => mutation.isPending, enableBeforeUnload: () => mutation.isPending })`（route の移動とリロード等の page の離脱。2 つの入口が同じ hook を使うので両方に効く。Gated Amendment 2） | `src/features/csv-import/hooks/useRollbackCsvImport.ts`（新設）、`src/features/csv-import/hooks/useCsvImportFlow.ts`（共有の hook を使い、成功で dispatch と `rollbackResult`。importing の間の `useBlocker` は今のまま） | Matrix の U5・U6・U9・U12・U13 |
| 取込みの結果の画面 | 「取り消す」の確認を共有の部品に（中身の文と dl を外す） | `src/features/csv-import/components/ResultStep.tsx` | Matrix の U7 |
| Z004 タブ | 上に「取り込んだ記録を見る（取り消すときはここから）」（`/inventory/records` へ `recordType: "csv_import"`）、`tab` の search を `Tabs` の `defaultValue` に（初期選択だけ）。Z004 タブの取消の応答を待つ間（flow の `isRollingBack`）は 2 つの `TabsTrigger` を disabled にする（タブの切替えは router の遷移でなく blocker が効かない。Radix の `TabsContent` は選ばれていないタブの中身を unmount するので、切り替えると flow ごと消える。Gated Amendment 2）。`isRollingBack` は `CsvImportFlowPanel` から `CsvImportPage` へ上げるが、Z004 の flow の寿命（タブを切り替えると初期化される。55 §55.7 の「page unmount 後の state 喪失問題」）は変えない（`useCsvImportFlow` を `CsvImportPage` へ移さない。panel から上へ知らせる形） | `src/features/csv-import/CsvImportPage.tsx` | Matrix の U8・U14 |
| `tab` の値の集合（UI-STATE-D2） | feature が readonly tuple `CSV_IMPORT_TAB_VALUES`・Zod schema `csvImportSearchSchema`・型 `CsvImportSearch` を持ち（上の「起票時に起草役が決めたこと」の 5）、route は schema を import して `validateSearch` に渡すだけ（route に `z.enum` を書かない）。`Tabs` の値と取消の結果の link の `tab` も同じ tuple・型から取る | `src/features/csv-import/types.ts`（b1 の file を本 lane で変える）、`src/routes/csv-import/index.tsx` | Matrix の U8、AC3 の 4 つの `rg` |
| CSV取込みの詳細 | header の操作に「この取込みを取り消す」（状態が `completed`・`completed_partial` のときだけ）、確認、成功の後に取消の結果（b1 の `RollbackResultCard`）と「商品別CSV取込み（Z004）を開く」（`/csv-import` へ `tab: "z004"`）。取消の結果と取消の mutation の状態は表示している取込みID のものだけ: page の中で state を持つ部分（取消の結果の `useState`・`useRollbackCsvImport`。detail の query も含めてよい）を内側の component に分け、外側が `key={importId}` で渡す（同じ route の param の変化で作り直す。route の `remountDeps` は予定 path の外なので採らない。Gated Amendment 2） | `src/features/inventory-records/CsvImportRecordDetailPage.tsx` | Matrix の U1・U5・U6・U13・U15 |
| 順 4 の拒否の Alert の link | `error.cmdError.related_csv_import_id` が数のとき（`!= null`）だけ（出すかと id を field だけで決め、文では決めない）、Alert に「取込みID {id} の記録を見る（取り消すときはここから）」（`/csv-import/records/{id}`、`returnTo` は `/csv-import?tab=z004`）。見出しと「最初に戻る」は今のまま | `src/features/csv-import/components/ErrorState.tsx` | Matrix の U11 |
| query key | `queryKeys.inventoryRecords.csvImportRollbackPreview(importId)` | `src/lib/query-keys.ts` | Matrix の U10 |
| test | Matrix の U1〜U15 | `CsvImportRecordDetailPage.test.tsx`（U13・U15 を含む）・`RollbackConfirmDialog.test.tsx`（新設）・`ResultStep.test.tsx`・`useCsvImportFlow.test.tsx`（U12）・`CsvImportPage.test.tsx`（U8・U11・U14） | 既存の test は Matrix の T2〜T4・T7 |

### S4 docs・生成物

- 上の「b1 の merge の後に同期する節」の全部と、本 commit の新しい節の「proposed・未実装」を現行に。
- 生成物: `src/lib/bindings.ts`（S2）、`docs/function-design/90-traceability.md`（test の REQ の参照が増えるので `cd src-tauri && cargo run --bin generate_traceability`。手で直さない）。

### 節の所有表（並走の lane と重なる file）

`comm -12` の結果（2026-10-24、Plan Review round 3 の是正で予定 path が 47 になったので、全ての open な PR について取り直した。差分は remote-tracking の head の `git diff --name-only origin/main...origin/<branch>`（括弧の中の SHA）、本 packet の予定 path 47 と各 path の集合を `LC_ALL=C sort -u` して `comm -12` で突き合わせた。PR の一覧は `gh pr list --state open`）:

- b1（PR #188、`agent/stocktake-count-runtime-b`、`c667d52c`）: 差分（176）とは 39（予定 path 47 から、本 lane だけの 8 path〈packet・Matrix・`RollbackConfirmDialog.tsx` とその test・`useRollbackCsvImport.ts`・`CsvImportRecordDetailPage.tsx`・`ErrorState.tsx`・`src/routes/csv-import/index.tsx`〉を除いた全部）。round 3 の是正で足した 7 path（`docs/function-design/40-cmd-product.md`・`src-tauri/src/biz/mod.rs`・`src-tauri/src/cmd/mod.rs`・`daily_report_import_cmd.rs`・`plu_export_cmd.rs`・`product_cmd.rs`・`settings_cmd.rs`）は全部 b1 の差分にある（b1 が `CmdError` に `stocktake_recovery` を足し、23 か所の literal を変えた file）。b1 の packet（`docs/plans/2026-10-22-stocktake-count-runtime-b1.md`）の「予定 path の一覧」（148）とは 37（上の 39 から `CsvImportRecordDetailPage.test.tsx` と `RecheckTargetList.tsx` を除いたもの）。
- PR #172（`agent/sd-direct-read-runtime`、`c4b907bd`、差分 59）: 差分とは 12（`docs/SCREEN_DESIGN.md`・`55-ui-csv-import.md`・`90-traceability.md`・`src-tauri/src/biz/mod.rs`・`src-tauri/src/cmd/mod.rs`・`csv_import_cmd.rs`・`daily_report_import_cmd.rs`・`plu_export_cmd.rs`・`product_cmd.rs`・`settings_cmd.rs`・`src-tauri/src/lib.rs`・`src/lib/bindings.ts`）。#172 の `csv_import_cmd.rs`・`plu_export_cmd.rs`・`product_cmd.rs`・`settings_cmd.rs` の変更は test の `AppState` の fixture の 1 行〈`register_sd_scan_cache`〉、`daily_report_import_cmd.rs` は SD の command の追加で `CmdError` の literal を 4 か所足す（`git diff origin/main...origin/agent/sd-direct-read-runtime -- src-tauri/src` の `+` の行の `error_id: None|Some` が 4）、`biz/mod.rs` と `cmd/mod.rs` は `BizError::SourceCopyFailed` とその変換の先頭の分岐。本 lane の variant・変換の arm・literal の 1 行とは別の行で、隣り合う enum と match の arm は merge の後に両方を残す。#172 と本 lane のうち後に merge する側は、同期で #172 の 4 か所の literal にも `related_csv_import_id: None` を足す（足し忘れは Rust の build が止める）。packet が名を挙げる path とも 12。
- PR #190（`agent/backup-offsite-runtime`、`000345f1`、差分 7）: 差分とは 0。packet が名を挙げる path とは 5（`docs/FUNCTION_DESIGN.md`・`90-traceability.md`・`settings_cmd.rs`・`src-tauri/src/lib.rs`・`src/lib/bindings.ts`）。
- PR #193（b2、`agent/stocktake-count-runtime-b2`、`abec3f50`、差分 5）: 差分とは 0。packet が名を挙げる path とは 12（`docs/FUNCTION_DESIGN.md`・`docs/SCREEN_DESIGN.md`・`cmd-task-specs.md`・`20-io-product-repo.md`・`40-cmd-product.md`・`65-inventory-record-traceability.md`・`90-traceability.md`・`src-tauri/src/biz/mod.rs`・`product_cmd.rs`・`src-tauri/src/lib.rs`・`RecheckTargetList.tsx`・`src/lib/bindings.ts`）。b2 は 40 の商品の結果の列、`biz/mod.rs` の re-export、`RecheckTargetList` の範囲の文字の部品を名指す（本 lane の `CmdError`・variant・`renderAction` とは別の所）。
- PR #194（`agent/backup-offsite-runtime-b`、`f508537f`、差分 7）: 差分とは 0。packet が名を挙げる path とは 6（`docs/FUNCTION_DESIGN.md`・`90-traceability.md`・`settings_cmd.rs`・`src-tauri/src/lib.rs`・`src/lib/bindings.ts`・`src/lib/query-keys.ts`）。
- PR #196（`agent/initial-load-procedure`、`a9f0f439`、差分 5）: 差分とは 0。packet が名を挙げる path とは 1（`parse_tests.rs`。docs だけの lane が test を出典に引く）。
- PR #198（`agent/oplog-retention`、`a403f9fb`、差分 13）: 差分とは 2（`docs/SCREEN_DESIGN.md`・`20-io-product-repo.md`。どちらも本 lane が同期で触る節〈Z004 タブの導線、stocktake_repo の読取り〉と別の節）。packet が名を挙げる path とは 5（ほかは `90-traceability.md`・`settings_cmd.rs`・`src-tauri/src/lib.rs`）。
- PR #201（`agent/dev-bins-out-of-msi`、`07ea21fd`、差分 5）: 差分とは 0。packet が名を挙げる path とは 4（`docs/FUNCTION_DESIGN.md`・`90-traceability.md`・`src-tauri/src/lib.rs`・`src/lib/bindings.ts`）。
- PR #202（`agent/screen-elements-redesign`、`1ad573bf`、差分 1）: 差分とは 0。packet が名を挙げる path とは 1（`docs/SCREEN_DESIGN.md`）。
- PR #203（`agent/docs-small-batch-1024`、`d3007c38`、差分 1）: 差分とは 0。packet が名を挙げる path とは 4（`docs/SCREEN_DESIGN.md`・`ui-task-specs.md`・`35-biz-stocktake-service.md`・`stocktake_service.rs`。#188 に任せる旧表記の記録）。
- PR #204（`agent/docs-file-line-check`、`04aff55f`、差分 2）: 差分とも、packet が名を挙げる path とも 0。
- 起票の時点の表にあった PR #191・#192 は main に merge 済み（本 branch は `843c4938` を取り込んだ）なので外した。

（「packet が名を挙げる path」は各 lane の packet〈`- Branch:` 行で特定〉と Matrix の本文の path の形の語を全部拾い、その lane の差分を足した集合で、予定 path より広い。重なりを少なく見積もらないために使った。）

本 commit の 4 つの正本の追記は、b1・#172 が変える hunk の外に置いた。本 commit を作った後に `git merge-tree --write-tree --name-only HEAD <branch>` を b1（`a7912eea`）・#172（`origin/agent/sd-direct-read-runtime`）・b2（`origin/agent/stocktake-count-runtime-b2`）に当て、3 つとも衝突の file 名を出さずに tree の id だけを返した（衝突なし）。b1 とは stacked で、本 lane は b1 の merge の後に `origin/main` を単段で取り込み、b1 の版を起点に直す。生成物（`src/lib/bindings.ts`・`docs/function-design/90-traceability.md`）は取り込んだ後に再生成する（D-098）。

| file | 並走の lane が所有する節 | 本 lane が触る所 | merge の後に揃える所 |
|---|---|---|---|
| `src-tauri/src/biz/csv_import_service/rollback.rs`・`mod.rs`・`tests/rollback_tests.rs` | b1: 取消の新しい経路と `RollbackResult` | 本体の切り出し・preview・型・test | b1 の版を起点に切り出す（b1 への依存の 1） |
| `src-tauri/src/biz/stocktake_service.rs`・`src-tauri/src/db/stocktake_repo.rs` | b1: 計数・完了・記録の詳細 | 完了の後の比較の範囲の判定の公開、完了した棚卸しの一覧の読取り | 別の関数（b1 への依存の 5） |
| `src-tauri/src/biz/csv_import_service/parse.rs`・`tests/parse_tests.rs`・`tests/commit_tests.rs`（b1 の file を本 lane で変える） | b1: 順 1〜5 の拒否（`check_settlement_rules`）と、その test | 順 4 の拒否の文に取込みID を入れ、既存の test の期待を当て直す（S1 の順 4 の行） | b1 の版を起点に順 4 の文と期待だけを直す（起票時に起草役が決めたことの 6） |
| `src-tauri/src/cmd/csv_import_cmd.rs`・`src-tauri/src/lib.rs` | b1: Z004 の command の入出力と、その test（順 4 の文の期待を含む）。#172・#190・#194・#198・#201: 各 lane の packet が名指す `lib.rs` の変更 | preview の command と登録の 2 行、`csv_import_cmd.rs` の test の順 4 の文と `related_csv_import_id` の期待、`CmdError` の literal 4 か所の None | 一覧の行は両方を残す |
| `src-tauri/src/biz/mod.rs`・`src-tauri/src/cmd/mod.rs`・`daily_report_import_cmd.rs`・`plu_export_cmd.rs`・`product_cmd.rs`・`settings_cmd.rs`（round 3 の是正で足した） | b1: `BizError::StocktakeGuard` と `CmdError.stocktake_recovery`（literal 23 か所）。#172: `BizError::SourceCopyFailed` とその変換、SD の command（`daily_report_import_cmd.rs` に literal 4 か所）、test の `AppState`。b2: `biz/mod.rs` の re-export。#190・#194・#198: 各 packet が名指す `settings_cmd.rs` の変更 | `BizError::SettlementConflict` と `Display`、`CmdError.related_csv_import_id` と変換の arm、literal ごとに None の 1 行 | b1 の版を起点に足す。#172 の後なら #172 の 4 か所の literal にも None（上の #172 の行） |
| `src-tauri/src/cmd/stocktake_cmd.rs`（Gated Amendment 1 で足した） | b1: 棚卸しの command と、`completion_blocked` の wire を完全一致で照合する test（b1 Matrix W4）。#172: test の `AppState` の fixture の 1 行（`register_sd_scan_cache`。`git diff origin/main...origin/agent/sd-direct-read-runtime -- src-tauri/src/cmd/stocktake_cmd.rs` の 1 hunk、2026-10-26） | W4 の test の期待 JSON に `"related_csv_import_id": null` の 1 key（Matrix の T6） | 別の行。後に merge する側が両方を残す |
| `src/features/csv-import/*`（`CsvImportPage`・`ResultStep`・`useCsvImportFlow`・`types.ts` と test） | b1: 停止の Alert を外す、取消の結果、`confirmImport` の引数、`rollbackResult`、`types.ts` の state と action の型（`additionalImportConfirmed` を外す） | 確認の部品・mutation の共有・Z004 タブの導線・`tab`（`types.ts` には `tab` の tuple・schema・型を足すだけ） | b1 の版を起点に直す（b1 への依存の 3・7） |
| `src/features/stocktake/components/RecheckTargetList.tsx`（b1 の file を本 lane で変える） | b1: 要再確認の一覧の部品（棚卸しの画面と取込み・取消の結果が `renderAction` を渡して使う） | `renderAction` を任意にし、省略したときは「操作」列と入れ直しの文を出さない | b1 の版を起点に props の 1 つと省略の分岐だけを足す。今の 2 つの呼出し元は変えない（b1 への依存の 3） |
| `src/lib/query-keys.ts` | b1: `stocktake.recheckList`。#194: backup の key | `inventoryRecords.csvImportRollbackPreview` | 別の key |
| 32・41・55・65 | b1: D-118 の節・停止の節・取消と確定の手順。#172: 55 の UI-07-D15・UI-07-D16・§55.7 の解除の行と更新履歴 | 本 commit の新しい節と、上の「b1 の merge の後に同期する節」 | 新しい節は hunk が重ならない。同期する節は b1 の版を起点に本 lane の分だけ直す |
| `docs/function-design/55-ui-csv-import.md` §55.7 の解除の行（Gated Amendment 2 で足した）と importing の段落（Gated Amendment 4 で直した） | #172（`agent/sd-direct-read-runtime`、`f0d6d1b0`）: 「`result` / `error` / `idle` / `parsing` / `preview` の各 state では block しない」の行に「（本節は Z004 タブの `useCsvImportFlow`。日報取込みタブが取消の応答を待つ間も止めるのは UI-07-D15 (6)）」を足した（`git show origin/agent/sd-direct-read-runtime:docs/function-design/55-ui-csv-import.md` の 411 行。本 branch では 413 行で、足す前の文のまま。2026-10-27） | 同じ行を、取消の応答を待つ間の `result` を止める形に書き直す（上の「Gated Amendment 2 で是正の Writer が直す正本の節」）。importing の段落で本節の hook に `useDailyReportImportFlow` を名指す（Gated Amendment 4。#172 の解除の行の「本節は Z004 タブの `useCsvImportFlow`」と食い違う） | 同じ行で衝突する。後に merge する方が先の版を起点に当て直す（両方の意味を残す: Z004 タブは取消の応答を待つ間も止める、日報取込みタブの止めは UI-07-D15 (6)）。Gated Amendment 3 で本 lane の §55.7 の段落から UI-07-D15 (6) への前方の参照を外したので、相互の参照（Z004 タブの止めと日報取込みタブの止めを並べる文）は後に merge する方が足す |
| `docs/UI_TECH_STACK.md` §6.11 UI-USW-D1（Gated Amendment 3） | PR #212（`agent/jan-dup-custom-code`）: §6 の invalidation の表の行（別の節）。PR #172 は触らない | UI-USW-D1 の括弧の例外の言い方 | 別の節。衝突しない |
| `src/hooks/unsaved-changes-guard-sweep.test.ts` T16（Gated Amendment 3） | どの open な lane も触らない（PR #172 は日報取込みの止めを既存の hook の中に置き、この test を変えない） | 期待の集合に 1 行、題名 | — |
| 20・35・40・SCREEN_DESIGN・FUNCTION_DESIGN・cmd-task-specs・ui-task-specs | b1: D-118 の節と停止の記述、40 §5.3 の `stocktake_recovery`。b2: 20・40・65・SCREEN_DESIGN の b2 の分。#198: 20・SCREEN_DESIGN の操作ログの保持。#203: 35・ui-task-specs・SCREEN_DESIGN の旧表記の記録 | 上の「b1 の merge の後に同期する節」 | b1・b2 の版を起点に本 lane の分だけ直す |
| `src/lib/bindings.ts`・`docs/function-design/90-traceability.md` | 生成物 | 生成物 | 取り込んだ後に再生成 |

### 予定 path の一覧（`comm -12` の入力）

50 path（Rust 17・TS 18・docs 15。Gated Amendment 1 で `src-tauri/src/cmd/stocktake_cmd.rs` を、Gated Amendment 3 で `docs/UI_TECH_STACK.md` と `src/hooks/unsaved-changes-guard-sweep.test.ts` を足した）。所有表の突き合わせは L3 を頼む時点の head で取り直す（D-124）。

```
docs/FUNCTION_DESIGN.md
docs/SCREEN_DESIGN.md
docs/UI_TECH_STACK.md
docs/architecture/cmd-task-specs.md
docs/architecture/ui-task-specs.md
docs/function-design/20-io-product-repo.md
docs/function-design/32-biz-csv-import-service.md
docs/function-design/35-biz-stocktake-service.md
docs/function-design/40-cmd-product.md
docs/function-design/41-cmd-pos.md
docs/function-design/55-ui-csv-import.md
docs/function-design/65-inventory-record-traceability.md
docs/function-design/90-traceability.md
docs/plans/2026-10-23-z004-rollback-entry.md
docs/plans/test-matrices/2026-10-23-z004-rollback-entry.md
src-tauri/src/biz/csv_import_service/mod.rs
src-tauri/src/biz/csv_import_service/parse.rs
src-tauri/src/biz/csv_import_service/rollback.rs
src-tauri/src/biz/csv_import_service/tests/commit_tests.rs
src-tauri/src/biz/csv_import_service/tests/parse_tests.rs
src-tauri/src/biz/csv_import_service/tests/rollback_tests.rs
src-tauri/src/biz/mod.rs
src-tauri/src/biz/stocktake_service.rs
src-tauri/src/cmd/csv_import_cmd.rs
src-tauri/src/cmd/daily_report_import_cmd.rs
src-tauri/src/cmd/mod.rs
src-tauri/src/cmd/plu_export_cmd.rs
src-tauri/src/cmd/product_cmd.rs
src-tauri/src/cmd/settings_cmd.rs
src-tauri/src/cmd/stocktake_cmd.rs
src-tauri/src/db/stocktake_repo.rs
src-tauri/src/lib.rs
src/features/csv-import/CsvImportPage.test.tsx
src/features/csv-import/CsvImportPage.tsx
src/features/csv-import/components/ErrorState.tsx
src/features/csv-import/components/ResultStep.test.tsx
src/features/csv-import/components/ResultStep.tsx
src/features/csv-import/components/RollbackConfirmDialog.test.tsx
src/features/csv-import/components/RollbackConfirmDialog.tsx
src/features/csv-import/hooks/useCsvImportFlow.test.tsx
src/features/csv-import/hooks/useCsvImportFlow.ts
src/features/csv-import/hooks/useRollbackCsvImport.ts
src/features/csv-import/types.ts
src/features/inventory-records/CsvImportRecordDetailPage.test.tsx
src/features/inventory-records/CsvImportRecordDetailPage.tsx
src/features/stocktake/components/RecheckTargetList.tsx
src/hooks/unsaved-changes-guard-sweep.test.ts
src/lib/bindings.ts
src/lib/query-keys.ts
src/routes/csv-import/index.tsx
```

## Non-scope

- 上の Goal の「非目的」の全部（取消の本体、日報の取消の入口、業務記録の取消〈REQ-208〉、専用一覧、取り込み直しの見込みの一覧、照合の基準の入替）。
- 入出庫履歴の一覧（§65.8.1）の行の取消の操作（§65.8.1 で置かない）と、一覧の表示列の変更（精算回数・ファイル名を行に出すこと。今は業務日付・代表商品・状態で探す。順 4 の拒否の取込みID の名指しと Alert の link〈「起票時に起草役が決めたこと」の 6・8〉と、Z004 タブの導線で足りる）。
- 順 2（取り消した取込みとレジの番号の食い違い）・順 3（同じ file の重複）の拒否の Alert の link（文の取込みID は今のまま。`CmdError.related_csv_import_id` は None）。順 4 以外の拒否から取消へ進む場面は、本 lane の Goal の誤った版の取消に当たらない。
- 操作ログ（`csv_rollback`）の形と、操作ログの画面の関連記録の link（65 §65.8.3 で `csv_import` は許可リストの外のまま）。
- 取消の後の要再確認の一覧の文と部品（b1）。
- 店に確かめる問い: 無い。

## Acceptance Criteria

- AC1（BIZ）: `cd src-tauri && cargo test --lib biz::csv_import_service` が全 PASS し、Matrix の RB1〜RB6 の test 名が `rg -n 'fn <名前>' src-tauri/src/biz/csv_import_service` で 1 件ずつ見つかる。`rg -c 'pub fn preview_rollback_csv_import\(' src-tauri/src/biz/csv_import_service` が 1、`rg -c 'tx\.finish\(\)' src-tauri/src/biz/csv_import_service/rollback.rs` が 1（preview が手順 4 の成否によらず TX を明示に閉じる）、`rg -c 'apply_rollback\(&tx, csv_import_id\)\?' src-tauri/src/biz/csv_import_service/rollback.rs` が 1（`?` で早期に返すのは取消の COMMIT の経路だけで、preview は手順 4 の失敗で閉じる前に返らない。Gated Amendment 2）。b1 の取消の test（Matrix の T1）が期待を変えずに PASS する。順 4 の拒否の文: `rg -c '既に取り込まれています（精算 \{\}、取込みID: \{\}）' src-tauri/src/biz/csv_import_service/parse.rs` が 1、`rg -c '既に取り込まれています（精算 \{\}）' src-tauri/src/biz/csv_import_service/parse.rs` が 0（exit 1）、`rg -c 'BizError::SettlementConflict' src-tauri/src/biz/csv_import_service/parse.rs` が 1 で、Matrix の RB7 の 3 つの test が PASS する（preview と確定の両方の経路で実際の取込みID が文と error の構造に出る）。
- AC2（CMD と wire）: `rg -c 'preview_rollback_csv_import' src-tauri/src/lib.rs` が 2（2 つの一覧に 1 つずつ）、`rg -c '^export type RollbackPreview =' src/lib/bindings.ts` が 1、`rg -c '^export type AffectedStocktake =' src/lib/bindings.ts` が 1、`rg -c 'related_csv_import_id\?: number' src/lib/bindings.ts` が 1（`CmdError` の任意の field）。`cd src-tauri && cargo test --lib cmd::csv_import_cmd` と `cargo test --lib cmd::tests` が全 PASS し、Matrix の W1・W3 の test 名が見つかる。`bash scripts/check-command-drift.sh` が exit 0（宣言・runtime の登録・Specta の登録・bindings の wire 名の 4 面が一致。Matrix の W2）。
- AC3（画面）: `npm test -- src/features/csv-import src/features/inventory-records src/lib` が全 PASS し、Matrix の U1〜U15 の test 名が見つかる。`rg -c 'useBlocker\(' src/features/csv-import/hooks/useRollbackCsvImport.ts` が 1（取消の応答を待つ間の止めは共有の hook の 1 か所）、`rg -c 'key=\{importId\}' src/features/inventory-records/CsvImportRecordDetailPage.tsx` が 1（詳細の state を取込みID で作り直す。どちらも Gated Amendment 2）。`rg -c 'この取込みを取り消す' src/features/inventory-records/CsvImportRecordDetailPage.tsx` が 1 以上、`rg -l 'commands.rollbackCsvImport' src --glob '!src/lib/bindings.ts' --glob '!*.test.*'` が `src/features/csv-import/hooks/useRollbackCsvImport.ts` の 1 行だけ（取消の mutation を 2 つ持たない）、`rg -c 'validateSearch: csvImportSearchSchema' src/routes/csv-import/index.tsx` が 1、`rg -c 'export const CSV_IMPORT_TAB_VALUES = \[' src/features/csv-import/types.ts` が 1、`rg -c 'from "@/features/csv-import/types"' src/routes/csv-import/index.tsx` が 1 以上、`rg -c 'z004' src/routes/csv-import/index.tsx` が 0（exit 1。route が値を literal で持たない。comment を含む）と、U8 の `Route.options.validateSearch` が feature の `csvImportSearchSchema` と同じもの（`toBe`）であること（route が値の集合を持たない。UI-STATE-D2）、`rg -c 'renderAction\?: ' src/features/stocktake/components/RecheckTargetList.tsx` が 1（`renderAction` が任意）、`rg -c 'recordType: "csv_import"' src/features/csv-import/CsvImportPage.tsx` が 1 以上、`rg -c 'related_csv_import_id' src/features/csv-import/components/ErrorState.tsx` が 1 以上（順 4 の拒否の Alert の link。出すかと id を field だけで決め、文から読まないことは U11 の (2)〜(6) で見る）。
- AC4（gate）: `cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`、`cargo run --bin generate_bindings` の後に `git diff --exit-code src/lib/bindings.ts`、`cargo run --bin generate_traceability -- --check`、`cargo test --test design_compliance_test`、`npm run typecheck && npm run lint && npm run format:check && npm test && npm run build`（`src/routeTree.gen.ts` は `.gitignore` の 33 行で追跡外なので diff を AC にしない。`tab` の search の型は typecheck と Matrix の U8 で見る）、`bash scripts/doc-consistency-check.sh` と `--target plan` が exit 0。L3 を含むので Writer は `cd src-tauri && cargo check --release` を通す。
- AC5（正本）: `rg -c 'rollback CTA は置かず' docs/function-design/65-inventory-record-traceability.md` が 0（exit 1）、`rg -c 'TRACE-D13、proposed・未実装|UI-07-D18、proposed・未実装|BIZ-03-D2、proposed・未実装' docs/function-design` が 0（exit 1）、`rg -c 'preview_rollback_csv_import（proposed・未実装）' docs/function-design/41-cmd-pos.md` が 0（exit 1）、`rg -l '取消の画面入口は結果 step だけ' docs --glob '!docs/archive/**' --glob '!docs/plans/**'` が 0 件（exit 1。b1 が外す記述が残っていない）。`rg -c '既に取り込まれています（精算 \{n\}、取込みID: \{id\}）' docs/function-design/32-biz-csv-import-service.md` が 1（順 4 の拒否の文）。

AC の数は AC1〜AC5 の 5。

AC の数の baseline（2026-10-23、本 worktree の HEAD `eed07d37` に本 commit の docs の差分。code は main と同じ。scratch の script〈repo の外〉で逐語に回した）:

```
$ rg -c 'preview_rollback_csv_import' src-tauri/src/lib.rs
exit=1
$ rg -c '^export type RollbackPreview =' src/lib/bindings.ts
exit=1
$ rg -c 'この取込みを取り消す' src/features/inventory-records/CsvImportRecordDetailPage.tsx
exit=1
$ rg -l 'commands.rollbackCsvImport' src --glob '!src/lib/bindings.ts' --glob '!*.test.*'
src/features/csv-import/hooks/useCsvImportFlow.ts
exit=0
$ rg -c 'validateSearch' src/routes/csv-import/index.tsx
exit=1
$ rg -c 'tx.rollback\(\)' src-tauri/src/biz/csv_import_service/rollback.rs
exit=1
$ rg -c 'rollback CTA は置かず' docs/function-design/65-inventory-record-traceability.md
1
exit=0
$ rg -c 'recordType: "csv_import"' src/features/csv-import/CsvImportPage.tsx
exit=1
$ rg -c 'pub fn preview_rollback_csv_import\(' src-tauri/src/biz/csv_import_service
exit=1
```

Plan Review round 1 の是正で足した AC の baseline（2026-10-24、本 worktree の HEAD `d4183273`。b1 の file は b1 の remote-tracking の head `c667d52c` の版も `git show` で回した。scratch の script〈repo の外〉で逐語に回した）:

```
$ rg -c '既に取り込まれています（精算 \{\}、取込みID: \{\}）' src-tauri/src/biz/csv_import_service/parse.rs
exit=1
$ rg -c '既に取り込まれています（精算 \{\}）' src-tauri/src/biz/csv_import_service/parse.rs
exit=1
$ git show origin/agent/stocktake-count-runtime-b:src-tauri/src/biz/csv_import_service/parse.rs | rg -c '既に取り込まれています（精算 \{\}）'
1
exit=0
$ rg -c 'renderAction\?: ' src/features/stocktake/components/RecheckTargetList.tsx
rg: src/features/stocktake/components/RecheckTargetList.tsx: IO error for operation on src/features/stocktake/components/RecheckTargetList.tsx: No such file or directory (os error 2)
exit=2
$ git show origin/agent/stocktake-count-runtime-b:src/features/stocktake/components/RecheckTargetList.tsx | rg -c 'renderAction\?: '
exit=1
$ bash scripts/check-command-drift.sh
D=68 H=68 S=68 T=68
exit=0
```

（main の版の `parse.rs` には順 4 の文が無く〈b1 が足す〉、`RecheckTargetList.tsx` は b1 が作る。どちらも b1 の merge の後に本 lane の head で 1・0 と 1 になる。）

Plan Review round 2 の是正で足した AC3 の baseline（2026-10-24、本 worktree の HEAD `fe7d10fd`。scratch の shell〈repo の外〉で逐語に回した）:

```
$ rg -c 'export const CSV_IMPORT_TAB_VALUES = \[' src/features/csv-import/types.ts
exit=1
$ rg -c 'validateSearch: csvImportSearchSchema' src/routes/csv-import/index.tsx
exit=1
$ rg -c 'z\.enum\(' src/routes/csv-import/index.tsx
exit=1
```

（`z.enum(` の 0 は今の route が `validateSearch` を持たないことによる。完了の時点の 0 は route が feature の schema を import して値の集合を持たないことを示す。AC3 の `validateSearch` の数は `validateSearch: csvImportSearchSchema` の数に置き換えた。上の起票の baseline の `rg -c 'validateSearch'` はその前の式の記録。）

Plan Review round 3 の是正で足した AC の baseline（2026-10-24、本 worktree の HEAD `f062843c`。scratch の shell〈repo の外〉で逐語に回した。`parse.rs` の variant は main の版に順 4 が無いので b1 の merge の後に 1 になる）:

```
$ rg -c 'from "@/features/csv-import/types"' src/routes/csv-import/index.tsx
exit=1
$ rg -c '"z004"' src/routes/csv-import/index.tsx
exit=1
$ rg -c 'related_csv_import_id\?: number' src/lib/bindings.ts
exit=1
$ rg -c 'related_csv_import_id' src/features/csv-import/components/ErrorState.tsx
exit=1
$ rg -c 'BizError::SettlementConflict' src-tauri/src/biz/csv_import_service/parse.rs
exit=1
```

（AC3 の route の `z.enum(` の 0 は、U8 の `toBe` の同一性の検査と、route の import・literal の 2 つの `rg` に置き換えて外した。`z.enum(` の数は route が自分の schema を `z.union` や literal で作る形を見落とし〈round 3 の是正。Review Response 参照〉、同一性の検査がそれを含めて止めるため。上の round 2 の baseline の `z.enum(` の行はその前の式の記録。route の literal の数は AC では引用符を付けずに `z004` で数える〈comment と単引用符も拾う〉。上の `'"z004"'` の行も今は exit 1 で、`z004` でも同じ〈今の route に `z004` の文字は無い〉。）

AC5 の baseline（同じ日、本 commit の docs の差分を当てた作業木で逐語に回した）:

```
$ rg -c 'TRACE-D13、proposed・未実装|UI-07-D18、proposed・未実装|BIZ-03-D2、proposed・未実装' docs/function-design
docs/function-design/32-biz-csv-import-service.md:1
docs/function-design/65-inventory-record-traceability.md:1
docs/function-design/55-ui-csv-import.md:1
exit=0
$ rg -c 'preview_rollback_csv_import（proposed・未実装）' docs/function-design/41-cmd-pos.md
1
exit=0
$ rg -l '取消の画面入口は結果 step だけ' docs --glob '!docs/archive/**' --glob '!docs/plans/**'
docs/function-design/55-ui-csv-import.md
exit=0
$ rg -c '既に取り込まれています（精算 \{n\}、取込みID: \{id\}）' docs/function-design/32-biz-csv-import-service.md
exit=1
```

（`rg -c 'rollback CTA は置かず'` は上の baseline で 1。`取消の画面入口は結果 step だけ` の 1 件は §55.0 の停止の節で、b1 が節ごと外す。）

Gated Amendment 2 で足した AC の baseline（2026-10-27、本 worktree の HEAD `42316f3a`〈`origin/main` `50f97a76` を取り込んだ merge〉。作業木で逐語に回した。修正前の値）:

```
$ rg -c 'tx\.finish\(\)' src-tauri/src/biz/csv_import_service/rollback.rs
exit=1
$ rg -c 'apply_rollback\(&tx, csv_import_id\)\?' src-tauri/src/biz/csv_import_service/rollback.rs
2
exit=0
$ rg -c 'tx.rollback\(\)' src-tauri/src/biz/csv_import_service/rollback.rs
1
exit=0
$ rg -c 'useBlocker\(' src/features/csv-import/hooks/useRollbackCsvImport.ts
exit=1
$ rg -c 'key=\{importId\}' src/features/inventory-records/CsvImportRecordDetailPage.tsx
exit=1
```

（`apply_rollback(&tx, csv_import_id)?` の 2 は preview の 73 行と取消の 109 行。`tx.rollback()` の 1 は preview の 74 行で、手順 4 が失敗すると届かない〈Final Review の P2〉。AC1 の `tx.rollback\(\)` の数は `tx\.finish\(\)` と `apply_rollback(…)?` の数に置き換えた。上の起票の baseline の `tx.rollback\(\)` の行はその前の式の記録。）

完了時の期待: AC1 の `pub fn preview_rollback_csv_import(` が 1、`tx.finish()` が 1、`apply_rollback(&tx, csv_import_id)?` が 1。AC2 が 2・1・1。AC1 の順 4 の文が新しい形 1・古い形 0（exit 1）。AC2 の `check-command-drift.sh` が exit 0。AC1 の `BizError::SettlementConflict` が 1。AC2 の `related_csv_import_id?: number` が 1。AC3 の詳細の文が 1 以上、取消の mutation の file が `useRollbackCsvImport.ts` の 1 つ、`validateSearch: csvImportSearchSchema` が 1、`CSV_IMPORT_TAB_VALUES` の定義が 1、route の feature の types の import が 1 以上、route の `z004` が 0（exit 1）、`renderAction?: ` が 1、`recordType` が 1 以上、`ErrorState.tsx` の `related_csv_import_id` が 1 以上、`useRollbackCsvImport.ts` の `useBlocker(` が 1、`CsvImportRecordDetailPage.tsx` の `key={importId}` が 1。AC5 の「置かず」・「proposed・未実装」の 2 本・「結果 step だけ」が exit 1、順 4 の文が 1。test の本数は AC にしない（PR と runner の出力を参照）。

## Design Readiness

設計の完了条件は `docs/DEV_WORKFLOW.md` Design Phase Rules の Design completion criteria。

- 引用する設計正本（節まで）: ADR SPEC-STK-CNT-D2（求め直す商品）・D3（判定の n、A1、「D に精算があるか」「最初の 1 本」）・D6（非遡及と完了の後の差）・D7（取消と取り込み直し）と Consequences（Gated Amendment の版）、[32](../function-design/32-biz-csv-import-service.md) の冒頭の D-118 の節の取消と表の順 2〜5・§15.5・§15.6a・§15.6b（本 commit）、[41](../function-design/41-cmd-pos.md) §17.4・§17.5、[55](../function-design/55-ui-csv-import.md) 冒頭の D-118 の表の「取消の結果」・UI-07-D14・§55.4 の手順 18・§55.5・§55.9・§55.11（本 commit）、[65](../function-design/65-inventory-record-traceability.md) §65.4.1・§65.5・§65.5a（本 commit）・§65.8.1・§65.8.2・§65.10 の 4b、[35](../function-design/35-biz-stocktake-service.md) の D-118 の節の「完了」の後半（完了の後の比較の範囲）、[40](../function-design/40-cmd-product.md) の CountRecoveryTarget、design-system [01](../design-system/01-decision-rules.md) の DSR-07・DSR-08・DSR-15・DSR-18・DSR-20、[04](../design-system/04-backbone.md) の「同じ操作は同じ顔と挙動」、D-052（C9）、[UI_TECH_STACK](../UI_TECH_STACK.md) の UI-STATE-D2（有限 URL search 値の feature-local SSOT）、D-127。
- 必要な設計成果物（状態）: (1) 取消の前の読取りの関数と wire（Design artifact selection の「New or changed BIZ … behavior」と「Tauri command, DTO」の行）= updated in this PR（32 §15.6b、41）。(2) CSV取込みの詳細の取消の入口（「operator workflow」の行）= updated in this PR（65 §65.5a）。(3) 2 つの入口に共通の確認・取消の mutation の共有・Z004 タブの導線・`/csv-import` の `tab`（「operator workflow, route/search state, Japanese wording」の行）= updated in this PR（55 §55.11）。(4) 35・20 の共有の判定と読取り、SCREEN_DESIGN の画面の行 = 実装の PR で同期（b1 の版を起点に。規則は変えない）。
- plan にしかない durable な判断の昇格先: 上の (1)〜(3) は正本に置いた。packet にだけあるのは、規模と手法、S1〜S4 の順、所有表、既存の test の移し方（Matrix の T1〜T5）、L3 の台本、「起票時に起草役が決めたこと」の 6・7・8（実装の PR で 32・40・41・55 へ。8 の画面の導線は本 commit で 55 §55.11 に置いた）。
- 前提・制約と、延期した design gap の follow-up: owner の判断事項 (i)〜(iii) は推奨を既定にした（答えで変わる所を各項に書いた）。取消の本体は b1 に依る（b1 への依存）。日報の取消の入口と業務記録の取消は非目的。
- 絶対保証（cannot happen / always happens）の例外と escape hatch の自己点検: 「preview は何も書かない」は、手順 4 の成否によらず TX を明示に閉じる `tx.finish()`（既定の `DropBehavior::Rollback` で ROLLBACK。Gated Amendment 2）と、TX の外で書かない（操作ログを書かない）ことで守る。例外: SQLite の TX が rollback の前に COMMIT される経路は無い（preview の関数は COMMIT を呼ばない。Matrix の RB2 が DB の全ての関係する表と `operation_logs` の行数で止める）。「確認と取消が同じ商品を名指す」は同じ関数と同じ DB の状態のときに限る（間に記録が入れば取消の結果が正。32 §15.6b の制約）。「取消済みには入口が出ない」は詳細の表示条件と、古い画面から開いた確認の preview の `ValidationFailed` の 2 重で守る。
- 判定: ready。本 lane の Scope の全ての振舞いが正本（本 commit の新しい節を含む）か本 packet の Boundary・S3 から決まる。owner の判断事項 (i) が C なら design へ戻す。

## Registration / Generation Obligations

| 変更対象 | 登録・生成義務 |
|---|---|
| Tauri command（frontend から呼ぶ） | `preview_rollback_csv_import` を `src-tauri/src/lib.rs` の Specta の登録（`export_bindings_to` の `collect_commands!`）と runtime の登録（`run` の `tauri::generate_handler!`）に足す。2 面と宣言・bindings の一致は `bash scripts/check-command-drift.sh` が静的に確かめる（`scripts/doc-consistency-check.sh` も同じ script を呼ぶ）。command 関数に `#[tauri::command]` と `#[specta::specta]` の対。`cd src-tauri && cargo run --bin generate_bindings` で `src/lib/bindings.ts` を再生成（S2。`CmdError` の任意の field の追加も同じ再生成で出る） |
| function-design doc 新設 | 該当なし（新しい doc は無い。`preview_rollback_csv_import` は `pub fn` で 32 §15.6b の code block にシグネチャを置いた。`src-tauri/tests/design_compliance_test.rs` の 32 の行の module は `csv_import_service` のまま） |
| source / workflow doc 新設・改名・削除 | 該当なし（既存の 32・41・55・65 に節を足しただけ） |
| REQ / coverage の追加・変更・削除 | test 名・comment に REQ-401・REQ-206・REQ-207 を付ける（REQ-208 は業務記録の取消で `coverage=deferred` のまま。CSV の取消は REQ-401 の範囲）。`cd src-tauri && cargo run --bin generate_traceability` で `docs/function-design/90-traceability.md` を再生成（S4） |
| route 新設・改名・削除 | 該当なし（`/csv-import/` の search に `tab` を足すだけ。`src/routeTree.gen.ts` は追跡外の生成物で、search の型は AC4 の `npm run typecheck` と Matrix の U8 で見る） |
| operator 画面新設・改名・削除 | 該当なし（CSV取込みの詳細・Z004 タブ・結果の画面の中身だけ。到達の導線は入出庫履歴と在庫変動履歴の今の link と、Z004 タブの新しい link） |

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | Which concepts belong to replaceable external adapters, and which concepts are stable app-core contracts? | Architecture, function design, decision-log, Plan Packet | 当たらない。Z004 の形（IO-02）に触れず、取消は app-core の記録の無効化と求め直し | — |
| Fact check / design decision split | Which claims are observed facts from hardware/tool/files, and which are app decisions that need source-doc promotion? | Investigation doc, source design docs, decision-log | 当たる。SQLite の TX の rollback の振舞いは観測（Contract Probe P1）。入口の置き場・範囲・期間は正本の読み（上の「起票時に確かめた 3 つ」）。確認の中身と導線は起草役の判断で正本へ置いた | 32 §15.6b、55 §55.11、65 §65.5a |
| Lifecycle / retry | What happens before, during, after, and after failure for import/export, duplicate input, rollback, retry, cancellation, and re-run? | Function design, DB design, UI design, Test Matrix | 当たる。確認を開く（preview の読取り）→ キャンセル（何も起きない）/ 取り消す（取消の本体。応答を待つ間は画面を離れられない）→ 取消の結果 → 取り込み直し（ADR D7）。preview の失敗は押せない、取消の失敗は同じ画面で同じ ID で再試行、別の取込みの詳細へ移ると前の取消の結果を出さない、取消済みへの重ねた取消は冪等の Ok、取消済みの preview は拒否 | Matrix の State Lifecycle |
| Operator workflow | What does the operator do in the real sequence across app, external tool, media, print/export, backup, and recovery? | Screen/UI design, function design, Plan Packet manual checks | 当たる。締め → SD → Z004 の取込み → 結果の画面を離れる → 誤りに気づく（preview の拒否・売上レポート・棚卸し）→ Z004 タブの「取り込んだ記録を見る」か入出庫履歴 → 詳細 → 確認 → 取消 → 入れ直し → 正しい版の取り込み直し | L3 の台本 |
| Replacement path | If the external system changes, which files/modules/docs are replaced and which app-core contracts remain stable? | Architecture, function design, decision-log | 当たらない。レジが替わっても取消は取込みの単位の無効化のまま | — |
| Data safety / evidence | How can the claim be supported by anonymized shape/count/hash/procedure evidence without committing real store data? | Plan Packet Data Safety, investigation doc, review evidence | 当たる。fixture は全部合成（b1 の L3 の fixture と Rust・TS の test）。L3 の DB は Coordinator が空の DB に差し替え、owner の今の DB は消さずに移して残す | Data Safety |
| Reporting / accounting semantics | Are totals, summaries, item records, returns, corrections, and inventory movements modeled separately enough to avoid false business meaning? | DB design, function design, report design, Test Matrix | 当たる。取消は売上と movement を無効にし（補償の movement を作らない）、完了した評価額を変えない（非遡及）。確認は「評価額は変わらず、記録の詳細の「差異」「ロス原価」と参考の差が変わることがある」と分けて書く（名指すのは比較の範囲による候補） | RB6、55 §55.11 |
| Manual verification | Which assertions cannot be proven by automated tests and require Windows native L3, external tool import, or real-device confirmation? | Plan Packet, Test Matrix, PR body | 当たる。Windows native の dialog の見た目と destructive の配置、通しの操作列（下の L3 の台本）。完了した棚卸しの名指しと順 4 の拒否の Alert（b1 の fixture では台本が順 4 を通らない）は自動 test と画面の写しで見る | Test Plan の L3 と画面の写し |
| 環境・再現性 | 新設の環境依存（toolchain / CI runner / OS 差異等）を repo-pinned config で強制するか、明示的に defer するか | repo-pinned config, Plan Packet | 当たらない。新しい依存は無い（TX の rollback は同梱の SQLite と rusqlite 0.31 の今の振舞い。`src-tauri/Cargo.toml` の `rusqlite = { version = "0.31", features = ["bundled"] }`） | — |

## Boundary / Wire Contract

- producer: BIZ-03 `preview_rollback_csv_import`（32 §15.6b）、CMD-07 `preview_rollback_csv_import`（41）。取消の `rollback_csv_import` と `RollbackResult` は b1 のまま。順 4 の拒否: BIZ-03 `check_settlement_rules` の `BizError::SettlementConflict` → CMD の `From<BizError> for CmdError`。
- consumer: 確認の部品 `RollbackConfirmDialog`（取込みの結果の画面と CSV取込みの詳細）、`describeError`（preview の失敗の文）。順 4 の拒否の link: `ErrorState`（取込みの画面の error の表示）。
- wire type（新しい型）: `RollbackPreview { csv_import_id: number, settlement_no: number, recheck_targets: CountRecoveryTarget[], affected_stocktakes: AffectedStocktake[] }`、`AffectedStocktake { stocktake_id: number, completed_at: string }`。command `previewRollbackCsvImport(csvImportId: number)`。error は今の `CmdError`（kind `not_found` / `validation` / `internal`）。変わる今の型: `CmdError` に `related_csv_import_id?: number | null`（任意。順 4 の拒否〈kind `import_error`〉だけが数を入れ、ほかは null か無し）。
- internal type: Rust の `RollbackPreview`・`AffectedStocktake`（`specta::Type`、`csv_import_service` の型）。`BizError::SettlementConflict { message: String, csv_import_id: i64 }`（順 4 だけが返す）。取消の本体を切り出した `pub(crate)` の関数（TX を受け、無効にした件数と `recheck_targets` を返す）。
- route / search: `/csv-import/` の `tab?: "daily-report" | "z004"`（zod の `.catch(undefined)`。欠落・ほかの値は既定の日報取込み）。値の集合は `src/features/csv-import/types.ts` の `CSV_IMPORT_TAB_VALUES` と `csvImportSearchSchema` が持ち、route は import する（UI-STATE-D2）。`/inventory/records` へは今の search の `recordType: "csv_import"` を渡すだけ（search の型は変えない）。
- precision/range: `settlement_no` は精算回数の範囲（b1 の 32 の順 1: 0 以上 `9007199254740991` 以下）で wire の number で正確。`csv_import_id`・`stocktake_id`・`related_csv_import_id` は SQLite の id（今の wire の `csv_import_id` と同じ扱い）。`recheck_targets` の数は b1 の `CountRecoveryTarget` の範囲（b1 の packet の Z4・Z18）。`completed_at` は棚卸しの完了の日時の文字列（今の `Stocktake.completed_at` と同じ形）で、確認は日付の部分を出す。
- round-trip path: DB → BIZ（TX で当てて戻す）→ specta → bindings → 確認の部品。search: 取消の結果の link `{ to: "/csv-import", search: { tab: "z004" } }` → route の `validateSearch` → `Tabs` の `defaultValue`。
- invalid input: 無い取込みID は `not_found`（「CSV取込み記録が見つかりません: ID {id}」）、取消済みは `validation`（「この取込みは取り消し済みです」）。確認はどちらも「取り消す」を押せない。不正な `tab` は既定に落ちる（拒否の画面を出さない）。
- compatibility: `RollbackResult`・`ImportResult`・`CsvImportRecordDetail`・`CsvImport` の形は変えない。`CmdError` は任意の field を 1 つ足すだけで、kind の集合・今の field・TS の今の literal（`toCmdError` と test の mock）は変えない。今の command の名前・引数は変えない。操作ログは preview で書かない（取消の `csv_rollback` は今のまま）。

## Test Plan

Test Design Matrix: [2026-10-23-z004-rollback-entry](test-matrices/2026-10-23-z004-rollback-entry.md)。

- If the Human Gate includes L3, Writer completion includes `cargo check --release` before the owner native build; this is not a CI gate.
- Human Gate に manual（L3）を含むときは、L3 の項目ごとに到達経路・必要な入力物（DB のデータ・import する file・CSV・scan）・依存する既知 backlog を並べ、受理される fixture を Ready の依頼と同時に渡す（`docs/DEV_WORKFLOW.md` Human Visual Confirmation）。

- targeted tests: `cd src-tauri && cargo test --lib biz::csv_import_service` / `cargo test --lib biz::stocktake_service` / `cargo test --lib cmd::csv_import_cmd`、`npm test -- src/features/csv-import src/features/inventory-records src/lib`。
- negative tests: Matrix の RB3（無い ID・取消済み）、RB2（何も書かない）、U4（preview の失敗・取消済みで押せない）、U6（取消の失敗と再試行）、U12〜U14（取消の応答を待つ間に離れられない。失敗の後は離れられる）、U15（別の取込みの詳細に前の取消の結果を出さない）、U8（不正な `tab`）、W3・U11（順 4 以外の error に取込みID が無く〈wire の null と field の無い形〉、link が出ない。U11 は順 4 の文でも field が null なら出さない）。
- compatibility checks: b1 の取消の test を期待を変えずに通す（Matrix の T1）。今の結果の画面の取消の flow の test は確認の中身の case だけを契約の変更（55 UI-07-D18）として直す（T2）。
- data safety checks: fixture は全部合成。
- mutation の確かめ: RB7 は 3 test とも順 4 が名指す取込みの ID が 1 以外になる fixture で、production の側で ID を固定値 1 にする mutant を当て、3 test とも red になることを確かめる（b1 の今の fixture は名指す取込みが ID 1 なので、この mutant でも 3 test とも GREEN になる）。cmd の test は名指す取込みの後にも取込みを 1 件足し、DB の最新の取込みの ID を名指す mutant（`SELECT MAX(id) FROM csv_imports` か、確定の経路の `last_insert_rowid()`）でも red にする。U8 は route の中で別の schema を作る mutant（`z.union` の literal で同じ値を受ける schema）で red（`toBe` の同一性）。U11 は次の mutant で red: link の id を `related_csv_import_id` 以外から取る（文の精算回数は (1)、文の取込みID と、field で出すかを決めて id だけ文から読む混成は (2)・(5)）、出す条件を文から取る（文の取込みID で出すのは (3)・(4)、順 4 の文の頭で出すのは (3)、field と順 4 の文の積は (5)、field と「取込みID」の語の積は (6)）、field の有無の判定を誤る（`!== undefined` と `"related_csv_import_id" in` は (3)、`!== null` は (4)）。Gated Amendment 2 の行（是正の Writer が実注入で red を確かめ、毎回撤去する）: RB6 は `settlement_kept` の `settlement.settlement_no <= no` を `<` にする mutant を (f)、`settlement.settlement_date.as_str() < date` を `<=` にする mutant と、`completed_stocktakes_comparing_settlement` の `completed_at.get(..10)` を外して完了の日時の全体と比べる mutant を (g) で red。U12 は `useRollbackCsvImport` の `useBlocker` を外す・`shouldBlockFn` を常に false にする・`enableBeforeUnload` を外す（mock の options に無く、実行時は既定の true で取消を待たない間もリロード等の離脱を止める）mutant を応答待ちの側で、`shouldBlockFn` と `enableBeforeUnload` を常に true にする mutant を失敗の後の側で red。U13 は止めを `useCsvImportFlow` だけに置く（共有の hook に置かない）mutant で red。U14 は `TabsTrigger` の disabled を外す mutant を応答待ちの側で、常に disabled にする mutant を応答を待たない側で red。U15 は `key={importId}` を外す mutant で red。P2-1 の preview の失敗の経路（手順 4 で `?` の早期に返る形に戻す mutant）は注入の道具が無く test で止まらない（Matrix の Residual Test Gaps。AC1 の `apply_rollback(&tx, csv_import_id)?` の数と Final Review の code 読みで止める）。
- main wiring/integration checks: W2（command の登録）、RB1（preview と取消が同じ商品を名指す）、U5（詳細の取消の後の D-052 C9 の再取得）、U7（結果の画面が同じ確認を使う）。

L3（Human Gate `manual`。owner は 2026-10-21 から数日、手が使いにくい〈左手だけ〉。1 回の通しの台本で、各段の PASS / FAIL を返してもらう。起動は今の L3 と同じく Windows native の `npm run tauri dev`）。L3 の日を T と書く（PC の今日）。入力物は b1 の S9 の生成の test が T から作る合成の file（**Coordinator** が L3 を頼む日に `cd src-tauri && INVENTORY_L3_DAY=その日 INVENTORY_L3_OUT=repo の外の folder CARGO_TARGET_DIR=/tmp/shared-cargo-target cargo test --lib -- --ignored io::l3_fixtures` で作り、Windows の path を省略せずに渡す）。

L3 の DB（Coordinator、L3 を頼む前）: owner がアプリを止めた状態で、b1 の packet の Test Plan の「L3 の DB の用意と差し替え」の 3. と同じ手順（22 §16 の作り直しの手順 2〜4）で、data の folder を日付の付いた folder（例 `%USERPROFILE%\inventory-db-moved\{L3 の日}-z004-rollback-before\`。前の lane の folder は上書きしない）へ消さずに移す。次の起動で空の DB が作られて最新の版まで当たり、初期の部門だけがある。b1・b2 の L3 の後の DB は移した folder に残る。

使う fixture（b1 の packet の「L3 の fixture の表」の行。売価・原価・初期在庫はその表のとおり）: `products-new.csv`（P1〜P6）、`z004-0101.csv`（精算 101、精算日 T−3、P3 × 1〈1000〉・P5 × 1〈1000〉）、`z004-0102.csv`（精算 102、精算日 T−2、P1 × 1〈1000〉）。

| # | 前の状態 | 操作 | 入力物 | 観測可能な合格基準 | 後の状態 |
|---|---|---|---|---|---|
| R-1 | 差し替えた空の DB | 起動する → 商品一括 import で `products-new.csv` を取り込む → Z004 タブで `z004-0101.csv`、続けて `z004-0102.csv` を取り込む（どちらも結果の画面の「ホームに戻る」で離れる） | 3 file | 2 つとも取り込め、結果に要再確認の一覧が出ない（数えた商品が無い） | 取り込み済み 101・102。P1 の在庫 9 |
| R-2 | 同じ | 棚卸しの画面で「棚卸しを開始する」→ P1 を「前の日の分を入れる」で日付 T−3、数 10 で保存 | 画面の入力 | P1 の数 10（日付 T−3）が出て、「数え直しが必要な商品」に P1 が出ない（T−3 に精算 101 があるので、102 は数に入らない側に決まる。ADR D3 の A1） | P1 の効く計数 10（T−3）、在庫 9 |
| R-3 | 同じ | 売上データ取込みの Z004 タブの「取り込んだ記録を見る（取り消すときはここから）」を押す | — | 入出庫履歴が種別「CSV取込み」で開き、取込みの行が 2 つ（業務日付 T−3 と T−2、どちらも「有効」） | 同じ |
| R-4 | 同じ | T−3 の行の「詳細」→「この取込みを取り消す」→ 確認を読む →「キャンセル」→ もう一度「この取込みを取り消す」 | — | 確認に、取込みID・精算日 T−3・「精算 101」・ファイル名 `z004-0101.csv`・件数 2 件・金額 ¥2,000、「取り消した後、次の商品は数え直しが必要です（今すでに必要な商品を含みます）」と P1・範囲「9〜10」・理由の文（文字と icon）。一覧に「操作」列と「棚卸しの画面で入れ直す」は無い、取り込み直しの文（精算 101）。完了した棚卸しの文は無い。読み終わるまで「取り消す」は押せない。「取り消す」は destructive の見た目で、「キャンセル」で何も起きない（詳細は「有効」のまま） | 同じ |
| R-5 | 同じ | 確認の「取り消す」 | — | Toast「取込みを取り消しました」。詳細が「この取込みは取消済みです」になり、「この取込みを取り消す」が消える。同じ画面に「取込みを取り消しました」（取り消した売上 2 件、取り消した在庫の変動 1 件）、「数え直しが必要な商品」に P1 と範囲「9〜10」と「棚卸しの画面で入れ直す」、「商品別CSV取込み（Z004）を開く」 | 101 は取消済み。P1 は要再確認 |
| R-6 | 同じ | 「商品別CSV取込み（Z004）を開く」→ `z004-0101.csv` をもう一度取り込む | `z004-0101.csv` | 売上データ取込みが「商品別CSV取込み（Z004）」のタブで開く。preview に拒否が無く取り込め、結果に要再確認の一覧が出ない。棚卸しの画面の「数え直しが必要な商品」から P1 が消えている | 取り込み済み 101（2 回目の取込み）・102 |
| R-7 | 同じ（2 回目の 101 の結果の画面） | 結果の画面の「取り消す」→ 確認を読む →「キャンセル」 | — | R-4 と同じ形の確認（取込みID は 2 回目の取込みの ID、精算 101、P1・範囲「9〜10」）が出て、「キャンセル」で何も起きない | 同じ |
| R-8 | 同じ | Z004 タブの「取り込んだ記録を見る」→ 取消済みの行の「詳細」 | — | 入出庫履歴の CSV取込みの行が 3 つ（101 の取消済み 1・有効 1、102 の有効 1）。取消済みの詳細に「この取込みを取り消す」が無い | 同じ |

- L3 に入れないもの（Windows native でなくても見られるもの、または台本を長くするもの）: 完了した棚卸しの名指し（完了まで進めると台本が倍になる。自動 test の RB6・U3 と下の画面の写し 1 で見る）、順 4 の拒否の Alert（b1 の L3 の fixture には同じ精算の別の版の file〈`z004-0102-other.csv`〉があるが、順 4 の拒否は自動 test の U11 と下の画面の写し 2 で確かめるので、L3 の台本には加えない）、preview の失敗と取消の失敗（DB の手での改変が要る。L3 Eligibility の (3)。U4・U6）、取消の応答を待つ間の離脱の止めとタブの disabled（L3 の DB では応答が短く、手で離脱を試せない。U12〜U14。Gated Amendment 2）、別の取込みの詳細へ移ったときの作り直し（R-5 の後に別の取込みの詳細を開く操作は台本を長くする。U15）。
- 画面の写し（Human Visual Confirmation。`docs/DEV_WORKFLOW.md` の同名の節は L3 Eligibility に依らず目視の枠を求める）: Coordinator が Ready の依頼と同時に owner に渡し、L3 と同じ 1 回の PASS / FAIL に含める。Windows native の L3 は要らない（どちらも Windows 固有の振舞いでない。Windows native の見た目は R-4・R-5 の確認と Toast で見る）。
  - 作り方（2 つとも同じ 1 通り）: 実装の head で、下の各写しの Matrix の case と同じ mock の render を、作業木に一時に置く test file（commit せず、写した後に消す）で回し、render の後の happy-dom の `document.documentElement.outerHTML` を repo の外の HTML の file に書き出す。その HTML に `npm run build` が作る `dist/assets/` の CSS の `<link>` を足して browser で開き、画面を写す（Vite の dev の画面は Tauri の command が無いので順 4 の拒否も完了した棚卸しも出せず、使わない）。
  - 写し 1（完了した棚卸しの名指し）: U3 の `affected_stocktakes` が非空の case の `RollbackConfirmDialog`。見る所: 確認の dialog の中の「完了した棚卸し（{完了の日付}）の評価額は変わりません。棚卸しの記録の詳細の「差異」「ロス原価」と「完了の後に入った記録による差」が変わることがあります。」の文が、要再確認の一覧と取り込み直しの文と分かれて読めること。
  - 写し 2（順 4 の拒否の Alert）: U11 の (1)（文の取込みID と field が同じ値の mock）の `CsvImportPage`。見る所: route `/csv-import` の Z004 タブの error の表示で、destructive の Alert の見出し「プレビューが利用できません」、順 4 の文（「同じ精算と思われる別の資料が既に取り込まれています（精算 {n}、取込みID: {id}）」）、その下の Alert の中の link「取込みID {id} の記録を見る（取り消すときはここから）」が link と分かる見た目で destructive の Alert の上でも読め、Alert の外の「最初に戻る」のボタンと、Z004 タブの上の「取り込んだ記録を見る（取り消すときはここから）」の link（U8。`/inventory/records` へ進む別の行き先）の両方と見分けられること。
  - 写しを作れなければ、その残りの危険を PR の本文に書き、受け入れを Ready の判断に含める。
- fixture: b1 の生成の test の 3 file だけ（本 lane は fixture を足さない）。owner の今の DB は消さずに移して残す（Data Safety）。

## Review Focus

- 「起票時に確かめた 3 つ」の読み（65 §65.5 の表示条件が CSV取込みの詳細に CTA を求めること、取り消せる範囲と期間が正本で一意に決まること）が正しいか。owner の判断事項を正本で決まる問いに混ぜていないか、逆に owner の判断が要る問いを起草役が決めていないか。
- preview が取消と同じ関数を同じ TX で当てて戻す形（BIZ-03-D2）が、何も残さず（RB2）、取消と同じ商品を名指すか（RB1）。preview が書込みの TX を開くことが、他の command（取込みの確定・計数の保存）と交わらないか（`AppState.db` の Mutex）。
- 確認の「取り消す」を押せる条件（読取りの間・失敗・取消済みで押せない）と、2 つの入口が同じ部品・同じ mutation を使うこと。取消の応答を待つ間の離脱の止め（route・リロード等の page の離脱・タブ）が両方の入口に効き、失敗の後に解けること。詳細の取消の結果が表示している取込みのものだけであること。preview が手順 4 の失敗でも TX を明示に閉じること（Gated Amendment 2）。
- 取消の後の再取得（D-052 C9 の `inventoryRecords.root()` が preview の key と詳細を含むこと）と、詳細の表示の切替（取消済みで CTA が消える）。
- **この変更で不要になる記述・条件・code が、ほかに残っていないか**（例: 「取消は結果の画面だけ」と読める文、`useCsvImportFlow` の取消の本体、ResultStep の今の確認の文、65 §65.10 の 4b）。
- 順 4 の拒否の error の形（「起票時に起草役が決めたこと」の 8: BIZ の variant と `CmdError` の任意の field、kind は `import_error` のまま）が最小か。TS の今の literal・`toCmdError`・`decideRecoverTo`・確定の token の扱いを変えずに済むか。
- L3 の台本の各段が、前の段の後の状態から ADR D3 の判定で 1 通りに決まるか（とくに R-2 の P1 が要再確認でないこと、R-4 の P1 の範囲「9〜10」、R-6 で解けること）。
- 公開 repo に実データを書いていないか。並走の lane の節を触っていないか（所有表）。

## Contract Ledger

触る設計正本の節の契約を行にした（隣接の契約の sweep: ADR D2・D3・D6・D7、32 の D-118 の節の取消と順 2〜5・§15.5・§15.6a・§15.6b、41 §17.4・§17.5、55 の D-118 の表・UI-07-D13・D14・§55.4 の手順 18・§55.5・§55.7・§55.9・§55.11、65 §65.4.1・§65.5・§65.5a・§65.7.2・§65.8.1・§65.8.2・§65.8.3・§65.10、35 の完了の後の比較の範囲、DSR-07・08・15・18・20、UI_TECH_STACK の UI-STATE-D2、D-052 C9 を当て、本 lane の Scope が触り得ないものは下の「除外」に理由つきで書いた）。

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| BIZ-03-D2（preview は取消を同じ TX で当てて戻す、何も書かない） | 32 §15.6b | S1 | RB1・RB2 | — |
| 32 §15.6b の手順 5（手順 4 の成否によらず明示に閉じる。drop に頼らない） | 32 §15.6b | S1（`tx.finish()`。Gated Amendment 2） | 無し（注入の道具が無い。AC1 の 2 本の `rg` と Final Review の code 読み。Matrix の Residual Test Gaps） | — |
| 32 §15.6b の手順 1（無い ID・取消済み） | 32 §15.6b | S1 | RB3 | — |
| 32 §15.6b の `settlement_no`（取り込み直す精算） | 32 §15.6b、24 の active な精算の一覧 | S1 | RB4 | R-4 |
| 32 §15.6b の `affected_stocktakes`（35 の完了の後の比較の範囲） | 32 §15.6b、35、ADR D6 | S1 | RB6 | 非対象: L3（台本を長くしない。Test Plan の画面の写し 1） |
| SPEC-STK-CNT-D2・D3（取消の後の求め直しと要再確認。A1・最初の 1 本） | ADR D2・D3、32 の取消 | S1（b1 の本体を切り出すだけ） | RB5・T1 | R-2・R-4・R-5 |
| SPEC-STK-CNT-D7（取消と取り込み直し） | ADR D7、32 の順 3〜5 | 変えない（b1） | T1 | R-6 |
| 32 §15.5 の冪等（取消済みへの取消は 0 件の成功） | 32 §15.5 | 変えない | T1 | — |
| 41 の `preview_rollback_csv_import`（薄い command、error の kind） | 41 §17.4・§17.5 | S2 | W1 | — |
| 41 §17.9（command の登録） | 41 §17.9 | S2 | W2 | — |
| TRACE-D13（詳細の取消の入口: active だけ、成功と失敗、到達） | 65 §65.5a、§65.5 | S3 | U1・U5・U6 | R-3〜R-5・R-8 |
| TRACE-D13（詳細の取消の結果は表示している取込みのものだけ。別の取込みの詳細へ移ると作り直す） | 65 §65.5a | S3（`CsvImportRecordDetailPage.tsx` の `key={importId}`。Gated Amendment 2） | U15 | 非対象: L3（台本を長くしない） |
| UI-07-D18・§55.7（取消の応答を待つ間は 2 つの入口で route の移動とリロード等の page の離脱を止め、Z004 タブではタブの切替えも止める。失敗すれば解く） | 55 §55.7・§55.11、65 §65.5a | S3（`useRollbackCsvImport.ts` の `useBlocker`、`CsvImportPage.tsx` の `TabsTrigger`。Gated Amendment 2） | U12・U13・U14 | 非対象: L3（応答が短く手で試せない） |
| §65.8.1（一覧の行に取消を置かない） | 65 §65.8.1 | 変えない | U1（行に無い） | R-3 |
| §65.5・DSR-18（詳細の戻り導線 `returnTo`） | 65 §65.5、DSR-18 | 変えない | T4（今の test） | — |
| UI-07-D18（共通の確認の中身と押せる条件、読取りの cache） | 55 §55.11 | S3 | U2〜U4・U7・U10 | R-4・R-7 |
| UI-07-D18（取消の mutation を 1 つに） | 55 §55.11、§55.9 | S3 | U5・U6・U9 | R-5 |
| UI-07-D18（Z004 タブの導線と `tab`。`tab` は初期選択だけ） | 55 §55.11、DSR-15 | S3 | U8 | R-3・R-6 |
| UI-STATE-D2（有限 URL search 値は feature の `types.ts` が tuple と schema を持ち、route は import する） | UI_TECH_STACK の UI-STATE-D2、55 §55.11 | S3（`src/features/csv-import/types.ts`・`src/routes/csv-import/index.tsx`） | U8（`Route.options.validateSearch` の同一性）、AC3 の 4 つの `rg` | — |
| UI-07-D14（正確な import ID、確認の項目、失敗で同じ ID の再試行） | 55 UI-07-D14 | S3 | U3・U6 | R-4 |
| §55.5・§55.9（取消の失敗は Toast だけ、state を変えない） | 55 §55.5・§55.9 | S3 | U6・T2 | — |
| 55 冒頭の表の「取消の結果」（取消の完了と要再確認の一覧） | 55 の D-118 の表 | 変えない（b1 の部品を詳細でも使う） | U5 | R-5 |
| D-052 C9（取消の再取得） | decision-log D-052、`src/lib/invalidation-contract.ts` | 変えない（共有の hook が呼ぶ） | U5 | — |
| 32 の表の順 4 の文（取込みID を名指す） | 32 の冒頭の D-118 の節の表 | S1（b1 の `parse.rs`） | RB7 | — |
| 順 4 の拒否の error が取込みID を構造で持つ（`BizError::SettlementConflict` → kind `import_error` と `CmdError.related_csv_import_id`。ほかは None） | 32 の表の順 4、40 §5.3、41 §17.4、55 §55.11 | S1・S2（`biz/mod.rs`・`cmd/mod.rs` と literal の 5 file） | RB7・W3 | 非対象: L3（台本は順 4 の拒否を通らない。自動 test） |
| 順 4 の拒否の Alert から名指した取込みの詳細へ（D-127 のその場で直せる） | 55 §55.11、65 §65.5a の到達 | S3（`ErrorState.tsx`） | U11 | 非対象: L3（画面の写しで確かめる。Test Plan の写し 2。台本は順 4 の拒否を通らない） |
| 要再確認の一覧の部品の操作の省略（確認の中で入れ直しへ進めない） | 55 §55.11 | S3（b1 の `RecheckTargetList.tsx`） | U3 | R-4 |
| DSR-07・DSR-20（destructive の確認） | 01 | S3 | U2 | R-4 |
| DSR-08（色だけで示さない） | 01、55 の D-118 の節 | S3（b1 の部品） | U3 | R-4 |

除外（触らない契約と理由）: 32 の順 1〜5 の判定と知らせ（取り込み直しは今の経路。変えるのは順 4 の文だけで、上の 32 の表の順 4 の行）。55 の UI-07-D13（同じ日の追加の確認。Z004 では b1 で外れる）。65 §65.7.2（業務記録の取消。REQ-208、非目的）。65 §65.8.3（操作ログの関連記録の許可リスト。`csv_import` は外のまま）。55 §55.7 の importing の間の止め（`useCsvImportFlow` の `useBlocker`。今のまま）と、確認の読取り（preview）の間（読取りだけで副作用が無いので止めない。止めるのは取消の応答を待つ間だけ）。

## Contract Audit（起票の時点の一巡）

新しく足す契約の文（本 commit の 32 §15.6b・41・55 §55.11・65 §65.5a と、上の Boundary・「起票時に起草役が決めたこと」）を、境界の 4 観点で一巡した。

- 安全な整数の範囲: 新しい wire の数は `settlement_no`（b1 の 32 の順 1 で 0 以上 `9007199254740991` 以下に限った値で、number で正確）と、SQLite の id 2 つ（`csv_import_id`・`stocktake_id`）だけ。範囲を持つ数（`recheck_targets` の在庫と範囲）は b1 の `CountRecoveryTarget` の型をそのまま使う。→ 不足は無かった。
- 0 の表示: 精算回数 0 は「精算 0」と出す（0 を「無い」の印にしない。b1 の Z5 と同じ）。`recheck_targets` が空なら「取り消しても、数え直しが必要になる商品はありません」と言い切り、空の一覧を出さない。`affected_stocktakes` が空なら文を出さない。取消の結果の件数 0（取消済みへの冪等の Ok）は「0 件」と出る（下の残りの危険）。→ 正本の不足は 55 の空の一覧の文で、本 commit の §55.11 に書いた。
- 失敗の時点と順序: preview は TX の前に対象を確かめ（無い・取消済みは何も開かずに拒否）、TX の中で取消を当て、明示の rollback で戻す。戻しの失敗は `DatabaseError` で、TX の外に何も書いていない。確認は読取りが成功するまで「取り消す」を押せない。取消は今の 1 TX（b1）で、失敗は Toast と同じ ID の再試行。preview と取消の間に記録が入れば取消の結果が正（確認の名指しは読んだ時点のもの）。→ 正本の不足は 2 か所（preview の戻し方と、押せる条件）で、本 commit の 32 §15.6b と 55 §55.11 に書いた。
- 単位や種類の混在: 確認の範囲は商品の在庫の単位（長さは cm を `stock_unit` で m に。b1 の部品）。日付は 3 種類を語で分ける: 精算日（`settlement_date`、レジの日付）、取込み日時（`imported_at`、PC の日時。詳細の「記録日時」）、完了の日付（`completed_at` の日付の部分）。番号も 2 種類を分ける: 取込みID（記録の番号）と精算回数（「精算 {n}」）。→ 不足は無かった。

この起票で、DB の列・migration・既存の wire の型の形は変えていない（新しい型は `RollbackPreview`・`AffectedStocktake` の 2 つ）。消した記述: 無し（置き換える記述は b1 の merge の後の同期で消す。上の「この変更で不要になるもの」）。

Plan Review round 3 の是正で足した契約（順 4 の拒否の error が取込みID を構造で持つこと〈`BizError::SettlementConflict`・`CmdError.related_csv_import_id`〉と、拒否の Alert の link。「起票時に起草役が決めたこと」の 8、55 §55.11）を同じ 4 観点で一巡した。

- 安全な整数の範囲: 足す数は `related_csv_import_id` だけで、`csv_import_id` と同じ SQLite の id（今の wire の `ImportResult.csv_import_id`・`CsvImport.id` と同じ扱い）。→ 不足は無かった。
- 0 の表示: SQLite の AUTOINCREMENT の id は 1 から（0 は来ない）が、画面は `!= null` で判定して 0 を「無い」の印にしない。field が null（wire の形。`c667d52c` の `src-tauri/src/cmd/mod.rs` の 71〜84 行の `CmdError` は `skip_serializing_if` を持たず、順 4 以外の error は `related_csv_import_id: null` で届く）でも、無い（`c667d52c` の `src/lib/invoke.ts` の 86〜92 行の `toCmdError` の internal への正規化と、既存の mock の形）でも link を出さない（U11 の (3) が null、(4) が無い形）。→ 不足は無かった。
- 失敗の時点と順序: field を入れるのは順 4 の拒否だけで、順 1〜3・5 とほかの error は None（W3）。順 4 の判定と名指す ID は同じ読取り（`find_active_imports_for_settlement` の先頭の行）から取り、読取りを足さない。確定の取り直しでは TX の中の読取りの ID（確定の時点の状態）を名指す。拒否の後の token の扱い（残す）と復帰先（`import_error` → idle）は kind も分岐も変えないので今のまま。link を押すまでに名指した取込みが取り消されれば、詳細が取消済みを出す（残りの危険）。→ 正本の不足は 55 §55.11 の導線の項で、本 commit で書いた。
- 単位や種類の混在: 拒否の文は精算回数（「精算 {n}」）と取込みID の 2 つの番号を持つ。link は取込みID だけを使い（`related_csv_import_id`）、文から番号を読まない。RB7 の fixture は取込みID を 1 以外・精算回数と別の値にし（cmd の test は最大でもない）、U11 の mock は取込みID と精算回数を別の値にし、文の取込みID と field も別の値にした case（(2)・(5)）を置いて、link の id が field の取込みID であることを見る（closure の是正。Review Response 参照）。→ 不足は無かった。

この是正で消した記述・条件: AC3 の route の `z.enum(` の 0（U8 の同一性の検査と route の import・literal の `rg` が含む）、Ordinary Operation の最後の行と 32 §15.6b の「売っていなければ変わらない」の断定と、それに伴う「参考の差は変わらない」の断定（どの数が変わるかは他の精算の判定でも決まる）、U3 の旧い文の 2 本の oracle（共通の語 1 本にした。Matrix の U3）、「起票時に起草役が決めたこと」の 6 の入出庫履歴の「記録ID」の絞り込みへの案内（8 の link に置き換えた）、所有表の merge 済みの PR #192 の注記。

Gated Amendment 2 で足した契約（preview が手順 4 の成否によらず TX を明示に閉じること、取消の応答を待つ間の離脱の止めとタブの disabled、詳細の取消の結果を取込みID で作り直すこと、RB6 の等値境界）を同じ 4 観点で一巡した。

- 安全な整数の範囲: 新しい wire の数は無い。`key={importId}` の取込みID は route の param の `Number(importId)` で、今の詳細の query key と同じ値（数でない param の扱いは今のまま）。RB6 の fixture の精算回数は b1 の範囲の内。→ 不足は無かった。
- 0 の表示: 新しい表示は無い（止めは真偽、`isPending` だけで決める）。RB6 (g) の空の期待は確認に文を出さない今の表示（U3 の空の case）。AUTOINCREMENT の取込みID は 0 にならないが、`key` は 0 でも作り直しの意味が変わらない。→ 不足は無かった。
- 失敗の時点と順序: preview は手順 4 が失敗しても手順 5 で閉じ、閉じる失敗（`DatabaseError`）を先に、閉じられたら手順 4 の失敗を返す（SQLite が自動で戻した TX では `finish` が何もしないので真因を覆わない）。止めは取消の mutation の pending の間で、React Query v5 は options の `onSuccess` / `onError` を待ってから pending を外す（`node_modules/@tanstack/query-core/build/modern/mutation.js` の 123 行の `await this.options.onSuccess` が 144 行の success の dispatch より前、159 行の `await this.options.onError`）ので、成功は Toast と C9 の再取得の後、失敗は失敗の Toast の後に解ける。確認の読取り（preview）の間は止めない（読取りだけ）。取消の応答を待つ間は route の移動が止まるので、詳細の `key` が pending の mutation を持ったまま変わることは無い。→ 正本の不足は 55 §55.7・§55.11 と 65 §65.5a で、是正の Writer が直す（上の「Gated Amendment 2 で是正の Writer が直す正本の節」）。
- 単位や種類の混在: 「待つ間」は 2 種類（確認の読取り〈止めない〉と取消の応答〈止める〉）、「離れる」は 3 種類（router の遷移〈`useBlocker`〉、リロード等の page の離脱〈`enableBeforeUnload`〉、タブの切替え〈router の遷移でないので `TabsTrigger` の disabled〉）で、手段を分けて書いた。native の window の close は止めない（55 §55.7 の「止める範囲」、UI-USW-D4。Gated Amendment 4）。RB6 の日付は精算日（`YYYY-MM-DD`）と完了の日時（`completed_at`、日付の部分 `get(..10)` で比べる）の 2 種類で、(g) は完了の日時に 0 時でない時刻を置き、日時の全体と比べる mutant も止める。→ 不足は無かった。

この GA で消した記述・条件: Contract Ledger の除外の 55 §55.7 の行のうち取消の応答を待つ間を除外していた部分（取消の応答を待つ間を止めるので、importing の止めと確認の読取りを止めないことだけを残した）、AC1 の `tx.rollback\(\)` の数（`tx\.finish\(\)` と `apply_rollback(…)?` の数に置き換えた）、Matrix の Mutation-style の境界の問いの答えの「RB6 の (b)(c)」（等値の境界を持たず、問いに答えていなかった。(f)(g) に置き換えた）。

## Contract Probe

本 lane が依る外部の前提を、2026-10-23 に確かめた。

- P1 SQLite の TX の中の書込みは同じ TX の読取りに見え、ROLLBACK で全部戻り、AUTOINCREMENT の連番も動かない（preview が取消を当てて読み、戻す前提）: repo の外の scratch の python の script（`sqlite3` の版 3.51.3。commit しない）で、movement の表（AUTOINCREMENT）と商品の表を作り、`BEGIN` → movement の `is_voided` の UPDATE → 商品の在庫を movement の和から UPDATE → 同じ TX で読む → `ROLLBACK` → 読む、を当てた -> 出力 `sqlite 3.51.3 in_tx 9 after 7 voided_after 0 seq (2,) (2,)`（TX の中で 9、戻した後に元の 7、無効の行 0、連番 2 のまま）。同梱の SQLite（`libsqlite3-sys` 0.28.0 の 3.45.0。b1 の packet の環境・再現性の lens）でも TX の意味は同じ（SQLite の TX の契約）。
- P2 rusqlite 0.31.0 の `Transaction` は明示の `rollback()` を持ち、drop の既定も rollback: `~/.cargo/registry/src/*/rusqlite-0.31.0/src/transaction.rs` の `drop_behavior: DropBehavior::Rollback`（`new_unchecked`）と `DropBehavior::Rollback => self.rollback_()` を読んだ。preview は drop に頼らず、手順 4 の成否によらず明示の `tx.finish()` を呼ぶ（32 §15.6b の手順 5。同じ file の 214〜228 行: `finish` は autocommit なら何もせず、そうでなければ `DropBehavior::Rollback` で ROLLBACK を出し、drop と違って error を返す。Gated Amendment 2）。repo の中の明示の rollback の先例は `src-tauri/src/biz/csv_import_service/time_evidence.rs` の `check_settlement_identity`（b1 が撤去する）と `src-tauri/src/db/inventory_repo.rs`・`src-tauri/src/db/plu_slot_repo.rs` の test。
- P3 preview の TX は他の command と交わらない: `src-tauri/src/cmd/mod.rs` の `AppState` は `pub db: Mutex<DbConnection>` の 1 本の接続で、CMD は `state.db.lock()` の後に BIZ を呼ぶ（41 §17.5 の各 command の手順 1）。

## Data Safety

- commit しない物: 実 POS / 店舗データ、DB、backup、log、receipt。fixture は全部合成（b1 の L3 の fixture と、Rust・TS の test の値）。
- local だけの場所: owner の PC のアプリの data の folder（名前は `src-tauri/tauri.conf.json` の `identifier`、今は `com.inventory-system.desktop`。Windows は `%APPDATA%` と `%LOCALAPPDATA%` の下の同じ名前の folder）、起動の作業フォルダの旧 DB 一式、設定した backup の保存先。保全の対象の一覧の正本は [22](../function-design/22-mnt-migration.md) §16 の「作り直しの手順」の「アプリの data の folder」と「folder の外でアプリが読む場所」で、L3 を頼む日にその版で確かめる。
- L3 の DB の差し替え（Test Plan の「L3 の DB」。D-123 でリリースの前まで開発・試験 DB の作り直しは包括で承認済み）: L3 の前に 1 回、owner の L3 用の data の folder を 22 §16 の作り直しの手順のとおり日付の付いた folder の下へ**消さずに移す**（backup の保存先の file は動かさない）。実行は owner の L3 の依頼の範囲で Coordinator（WSL の `/mnt/c/Users/...` から。消す command を使わない）。L3 の後は戻さず、移した folder は残す（消すかは owner の判断）。
- 合成だけの場所: Rust と TS の test の fixture、b1 の生成の test が作る L3 の file（repo の外の folder）。

## 残りの危険（起票の時点）

- 確認の名指しは確認を開いた時点の状態で、開いたまま別の画面で記録を入れてから取り消すと、名指しと取消の結果が違うことがある（単店舗の 1 台の PC で、取消の結果の一覧が正。32 §15.6b の制約）。
- 2 つの画面（取込みの結果の画面と CSV取込みの詳細）で同じ取込みを続けて取り消すと、後の側は取消の本体の冪等の Ok で「取り消した売上 0 件」と要再確認の無い結果を出す（詳細の再取得で「取消済み」に変わる。黙って在庫を動かしはしない）。
- preview は取消と同じ求め直し（計数を持つ全ての商品）を当てるので、商品と計数が増えると確認を開いてから読み終わるまでの時間が延びる（その間は「取り消す」を押せず、spinner と文を出す。32 §15.6b の見直しの契機）。
- 取り込み直したときに数え直しが要る商品は、取り込み直した結果で初めて名指される（owner の判断事項 (ii) の推奨。確認は「出ることがあります」と書く）。
- 32 の順 4 の拒否の文に取込みID と詳細への link を足すのは b1 の merge の後の本 lane の S1〜S3（b1 の merge から本 lane の merge までは、b1 の文のとおり精算回数だけを名指し、link は無い）。
- 順 4 の拒否の link の先の取込みが、link を押すまでの間に別の画面で取り消されると、詳細は取消済みの表示で取消の入口を出さない（詳細が正。単店舗の 1 台の PC）。
- 取消の応答を待つ間の Sidebar・戻るの操作は、文を出さずに止まる（取込み中の「他の画面に移れません」の banner は出さない。待つ間は取消の button の「取り消し中…」と spinner だけで、応答は短い。日報取込みタブ〈PR #172 の UI-07-D15 (6)〉と同じ扱い。Gated Amendment 2）。
- 取消の応答を待つ間に native の window の close（×・Alt+F4）で閉じると止まらず、取消の結果の通知を失う（取消は単一の transaction で、済んだか済まなかったかのどちらか。CSV取込みの詳細の状態で分かる）。importing・未保存の編集の止めと同じ水準で、UI-USW-D4 の別 backlog（Gated Amendment 4）。

## Implementation Results

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

（実装の後に書く。）

## Review Response

先行 round の結果・評価（判定・件数・採否・reviewer の意見）はこの節にだけ書く。前半の節と遷移記録には「round N の是正（Review Response 参照）」だけを書く（独立 review は `## Review Response` より前だけを読む）。

（Plan Review の後に書く。）

round 1（`d4183273`）: Claude 側 fresh Opus 5.5 = reject（P1 0 / P2 1 / P3 7）、Codex = reject（P1 0 / P2 4 / P3 1）。重なりは 2 件（`RecheckTargetList` の `renderAction`、「参考の差が変わります」の言い切り）。Coordinator は全 11 件を採用した。是正は起草役の `fd2c1f8c`。

- P2: `RecheckTargetList` の `renderAction` を任意にして省略時は操作の列と入れ直しの文を出さない（b1 の file を本 lane で変える）、名指す棚卸しを比較の範囲の候補と定義し「変わることがあります」に、確認の文を因果を言わない形に（取消の前から要再確認の商品を含む）、順 4 の拒否で取込み ID を名指す実装と既存 test の期待の更新を Scope・Matrix（RB7）に、L3 の保全の path を `tauri.conf.json` の identifier と 22 §16 を引く形に。
- P3: Specta と runtime の登録の 2 面と W2 を command drift の静的な検証に、正本の読みの基準に Gated Amendment `bc0bf2f5` を、routeTree の AC を外す、判断事項の答えで変わる所の列挙、RB1 の mutant の帰属を RB5 へ、U10 に 2 回目の読取りの間の disabled。
- 起草役の見送り（Coordinator が受理）: 32 §15.6b の求め直しの理由の文（取消の前に要再確認でなかった商品にも出る）は、確認の文の因果の言い切りとは別なので残す。

round 2（`fe7d10fd`）: Claude 側 fresh Opus 5.5 = reject（P1 0 / P2 1 / P3 1）、Codex = reject（P1 0 / P2 2 / P3 2）。重なりは 1 件（S3 の query key の行の崩れ）。Coordinator は全 5 件（重なりを 1 件に数えて）を受理し、最小の修正案に沿って是正した（是正は Coordinator の代行の Opus 5.5 subagent、本 commit）。

- P2（Opus #1）完了した棚卸しの確認の文が、取消で変わりうる記録の詳細の「差異」「ロス原価」を名指さない（b1 の `stocktake_service.rs` の 720 行の `ledger_at_cutoff` と 724 行のロス原価で確かめた）: 文を「完了した棚卸し（{完了の日付}）の評価額は変わりません。棚卸しの記録の詳細の「差異」「ロス原価」と「完了の後に入った記録による差（参考）」が変わることがあります。」に。反映: 55 §55.11 の「完了した棚卸し」の項と理由、65 §65.5a の「確認」、32 §15.6b の関数要求・`affected_stocktakes`・手順 3、packet の Goal の最小完了条件の 2 つ目・Ordinary Operation の最後の行・owner の判断事項 (iii) の問いと A の説明・S1 の行・Reporting の lens、Matrix の Contracts Under Test・Failure Modes・RB6（範囲が差異・ロス原価の使う精算を含む理由の 1 行）・U3（新しい文の exact と旧い文の 0）。名指す集合（35 の範囲）は変えない。owner の判断事項 (iii) は推奨 A の中身（範囲で名指す）を変えず、起草役が書いた問いと説明の語だけを揃えた。
- P2（Codex #1）RB7 の固定 `1` の mutant が b1 の今の fixture では red にならない（3 test とも名指す取込みが ID 1）: 3 test とも名指す取込みの前に別の取込みを 1 件入れて ID を 1 以外にし、ID が 1 でないことを assert し、固定 `1` の mutant で 3 test とも red を求める。反映: Matrix の RB7・T5・Mutation-style の 1 つ目、packet の S1 の順 4 の行・Test Plan の「mutation の確かめ」。
- P2（Codex #2）`tab` の値の集合の所有が UI-STATE-D2 に繋がらない: feature の `src/features/csv-import/types.ts` が tuple `CSV_IMPORT_TAB_VALUES`・schema `csvImportSearchSchema`・型 `CsvImportSearch` を持ち、route は import する形に（先例 `src/features/daily-sales/types.ts` と `src/routes/reports/daily.tsx`）。反映: 55 §55.11 の `tab` の項、packet の「起票時に起草役が決めたこと」の 5・S3 の新しい行・Boundary の route / search・Contract Ledger の UI-STATE-D2 の行と sweep の一覧・Design Readiness の引用・AC3 の 3 つの `rg` と round 2 の baseline・予定 path（39。`types.ts` は b1 の差分と b1 の packet の予定 path にある）と所有表、Matrix の Contracts Under Test・Failure Modes・U8・Adjacent Pattern Audit。
- P3（Opus #2・Codex #3、重なり）S3 の query key の行に `RecheckTargetList` の所有表の行が混ざった: query key を `src/lib/query-keys.ts`・U10 の 1 行に戻し、`RecheckTargetList.tsx` の行を「節の所有表」へ移した。
- P3（Codex #4）Matrix の「戻る・進むで search のとおり」が、`defaultValue` の初期選択だけという正本と合わない: Matrix の State Lifecycle の Revisit を「再 mount 時は search のとおり、同じ mount の間はタブの状態を保つ」に。55 §55.11 の `tab` の項と packet の「起票時に起草役が決めたこと」の 5 にも初期選択だけであることを書いた。
- Gated Amendment: 無し（plan-gate のままの是正で、Plan Commit は pending）。

round 3（`f062843c`、天井）: Claude 側 fresh Opus 5.5 = approve（P1 0 / P2 0 / P3 5）、Codex = reject（P1 0 / P2 1 / P3 1）。重なりは 2 組（Codex #1 と Opus F3、Codex #2 と Opus F1）。round 天井なので round 4 は開かず、Coordinator は全 5 件（重なりを 1 件に数えて）を受理し「同型指摘の一括是正」とした（2026-10-24）。是正は起草役の Opus 5.5 subagent（本 commit）。

| finding | 重大度 | disposition | 是正（反映先） |
|---|---|---|---|
| Codex #1 + Opus F3: AC3 の 3 本の `rg` と U8 の振舞いの期待は、route が同じ値を受ける別の schema を自分で作る形を見落とす | P2 / P3 | 受理 | U8 に `expect(Route.options.validateSearch).toBe(csvImportSearchSchema)` を足し、route の中の別 schema の mutant で red を求めた。AC3 に route の feature の types の import（`rg -c 'from "@/features/csv-import/types"'` ≥ 1）と route の `z004` の文字 0 を足し、同一性の検査に含まれる `z.enum(` の 0 を外した（継ぎ足さない）。反映: packet の AC3・round 3 の baseline・完了時の期待・S3 の行・Contract Ledger の UI-STATE-D2・Test Plan の mutation、Matrix の U8・Mutation-style |
| Codex #2 + Opus F1: 32 §15.6b と packet の「その精算が明細の商品を売っていなければ変わらない」は成り立たない（他の精算の「D に精算があるか」「最初の 1 本」が変わる） | P3 / P3 | 受理 | 断定を外し、「どの数が実際に変わるかは他の精算の判定でも決まるので求めず、確認の文は変わりうると言うだけにする」の 1 文にまとめた。Ordinary Operation の最後の行は、同じ根で成り立たない「参考の差は変わらない」の断定も外した。反映: 32 §15.6b の `affected_stocktakes`、packet の Ordinary Operation の最後の行。名指す集合（35 の範囲）と確認の文は変えない |
| Opus F2: RB7 で名指す取込みが 3 test とも DB の最新なので、最新の取込みの ID を名指す mutant が残る | P3 | 受理 | cmd の test で、名指す取込みの後にも順 2〜5 の比べる先にならない取込み（`rolled_back`）を 1 件足し、名指す ID が 1 でも最大でもないことを assert する。反映: Matrix の RB7・Mutation-style、packet の Test Plan の mutation |
| Opus F4: 確認の文の「完了の後に入った記録による差（参考）」が b1 の記録の詳細の見出しと一字一句合わない | P3 | 受理 | b1 の `c667d52c` の `src/features/inventory-records/StocktakeRecordDetailPage.tsx` を `git show` で読み（列の見出し 189 行「差異」・192 行「ロス原価」、265 行の aria-label「完了の後に入った記録による差」、267 行の見出し「完了の後に入った記録による差（参考。評価額は変わりません）」）、文の語を aria-label と同じ「完了の後に入った記録による差」にした。U3 の旧い文の oracle は round 1・2 の文に共通の「「完了の後に入った記録による差（参考）」」1 本にした。反映: 55 §55.11 の「完了した棚卸し」の項、packet の Ordinary Operation の最後の行、Matrix の U3 |
| Opus F5: 順 4 の拒否の Alert から、名指した取込みの詳細へその場で進めない（D-127） | P3 | 受理（plan で直す） | 拒否の error が取込みID を構造で持つ形と、Alert の link を足した。形は `BizError::SettlementConflict { message, csv_import_id }` → kind `import_error` のままの `CmdError.related_csv_import_id`（TS で任意）、`ErrorState` の Alert に「取込みID {id} の記録を見る（取り消すときはここから）」（`/csv-import/records/{id}`、`returnTo` は `/csv-import?tab=z004`）。最小の形を選んだ理由と棄却案は「起票時に起草役が決めたこと」の 8（今の error の型と wire を `c667d52c` の `cmd/mod.rs`・`biz/mod.rs`・`invoke.ts`・`ErrorState.tsx`・`useCsvImportFlow.ts`・`csv_import_cmd.rs` で読んだ）。境界の 4 観点の一巡は Contract Audit の round 3 の段落。反映: 55 §55.11（順 4 の拒否からの導線）、65 §65.5a の到達、packet の規模・Risk・Ordinary Operation（新しい行）・起草役が決めたこと 6・8・b1 への依存 8・この変更で不要になるもの・Scope（S1・S2・S3 の行と同期する節の 32・40・41・55）・所有表（取り直し）・予定 path（47: Rust 16・TS 17・docs 14）・Non-scope・AC1〜AC3・Boundary・Test Plan・Contract Ledger・Contract Audit・残りの危険・Design Readiness・Review Focus、Matrix の Contracts Under Test・Failure Modes・RB7・W3・U11・T5・Adjacent Pattern Audit・Boundary・Compatibility・Mutation-style・Negative Paths |

- 照合（scope の追加）: F5 は Scope に error の形と link を足す提案で、owner の記録（AGENTS.md の D-127「使い手に押しつけない」、TD-380 の関門の項目「結果の画面を離れた後も取り消せる」）と照合し、Goal（誤った版を取り消して取り込み直す）の到達の手数を減らす範囲で採った。順 2・順 3 の拒否の link は Goal の外として Non-scope に置いた。
- 所有表: 予定 path が変わったので全ての open な PR で取り直した（round 3 の Codex が挙げた #198 の差分 13・重なり 2 の食い違いもここで直した。#172 は head が `c4b907bd` に進み、`daily_report_import_cmd.rs` に `CmdError` の literal を 4 か所足すので、後に merge する側の同期で None が要る）。
- Gated Amendment: 無し（plan-gate のままの是正で、Plan Commit は pending）。plan-approved の前に、本 commit の版で fresh の reviewer の closure（Claude 側 + Codex）が要る（遷移 7）。

closure（`ff911e61`、round 3 の是正の版）: Codex = reject（P1 0 / P2 1 / P3 1）、Claude 側 fresh Opus 5.5 = approve（P1 0 / P2 0 / P3 1。F1〜F4 closed、F5 部分）。両方が同じ 2 点を挙げた（Codex は N1 を P2、Opus は P3）。Coordinator は 2 点とも受理した（2026-10-24）。是正は起草役の Opus 5.5 subagent（本 commit）。遷移は記録しない（plan-gate のまま。plan-approved は、本 commit の版を独立の reviewer が確かめた後に Coordinator が行う）。

| finding | 重大度 | disposition | 是正（反映先） |
|---|---|---|---|
| N1（Codex P2 / Opus P3）: round 3 の是正で足した順 4 の拒否の Alert の link が、人の目視の確認の対象に無い（`docs/DEV_WORKFLOW.md` の Human Visual Confirmation は L3 Eligibility に依らず目視の枠を求める。台本 R-1〜R-8 は順 4 を通らず、画面の写しは完了した棚卸しの文だけだった） | P2 / P3 | 受理 | Test Plan の「L3 に入れないもの」に順 4 の拒否の Alert を足し、「画面の写し」の項を立てて、完了した棚卸しの名指し（写し 1）と順 4 の拒否の Alert（写し 2）を同じ扱いにした。b1 の L3 の fixture に順 4 を通る file が無いので、写しの作り方を 1 通り（Matrix の case と同じ mock の render を一時の test で回し、happy-dom の DOM を HTML に書き出して `npm run build` の CSS で browser に開いて写す）に決め、写し 2 の見る所（route、見出し、順 4 の文、Alert の中の link の文と置き場所、「最初に戻る」との見分け）を書いた。写し 1 の「frontend の test の render か Vite の dev の画面」の 2 択は、Vite の dev の画面では Tauri の command が無く出せないので、同じ 1 通りに置き換えた（継ぎ足さない）。反映: packet の Test Plan、Contract Ledger の順 4 の拒否の Alert の行（「非対象: L3（画面の写しで確かめる…）」）と `affected_stocktakes` の行、Manual verification の lens。Owner Effort Budget は変えない（写しの確認は「L3 と画面の確かめ 1」の同じ 1 回） |
| F5 の残り（Codex P3 / Opus P3）: U11 の mock は文の取込みID と field がどちらも 7 で、link の id を文の `取込みID: (\d+)` から読む実装（field で出すかを決めて id だけ文から読む混成を含む）が通る。負の case の文も取込みID を含まない | P3 / P3 | 受理 | U11 を同じ test の中の 3 つの case にした: (1) 今の正の case、(2) 文の取込みID 9・field 7 で link の文と href が 7 で `/csv-import/records/9` が無いこと、(3) 負の case の mock を b1 の `c667d52c` の `parse.rs` の順 3 の preview の文（「このファイルは取込み済みです（取込みID: {}、取込み日: {}）」、`git show` で読んだ）にし、取込みID 5 があっても link が出ないこと。反映: Matrix の U11・Negative Paths・Mutation-style、packet の AC3（`ErrorState.tsx` の `rg` の注）・Test Plan の mutation・「起票時に起草役が決めたこと」の 8 の棄却 (a)・Contract Audit の round 3 の段落 |

- 不要になった記述: 写し 1 の作り方の「Vite の dev の画面」の択（上の N1 の行）。ほかに是正で不要になる記述は見つからなかった。
- Gated Amendment: 無し（plan-gate のままの是正で、Plan Commit は pending）。

closure 2（`432d48c4`、closure の是正の版）: Claude 側 fresh Opus 5.5 = reject（P1 0 / P2 1 / P3 2。N1 closed、F5 部分）。Coordinator の裁定は 3 件とも受理（2026-10-24。#1 の根拠の `CmdError` の serialize は Coordinator が b1 の head `c667d52c` の `src-tauri/src/cmd/mod.rs` の 71〜84 行で実読した）。是正は Writer の Opus 5.5 subagent（本 commit）。遷移は記録しない（plan-gate のまま）。

| finding | 重大度 | disposition | 是正（反映先） |
|---|---|---|---|
| Opus #1: U11 の負の case は field の「無い」mock だけで、wire の形（`CmdError` は `skip_serializing_if` を持たず、順 4 以外の error は `related_csv_import_id: null` で届く）の case が無い。`!== undefined` で出す実装が全ての error に `/csv-import/records/null` の link を出しても通る | P2 | 受理 | U11 に (3) 順 4 の文（取込みID 7）＋ `related_csv_import_id: null` → link 無し、を足した。field の無い形は `c667d52c` の `src/lib/invoke.ts` の 86〜92 行の `toCmdError` が internal へ正規化するときに実行時にも出る（本 lane は `toCmdError` を変えない）ので、(4) として残した（順 3 の文、取込みID 5、field 無し → link 無し。`!== null` で `/csv-import/records/undefined` を作る実装が red）。反映: Matrix の U11・unknown reference・optional field behavior、packet の Test Plan の negative tests・Contract Audit の「0 の表示」 |
| Opus #2: 出す条件を順 4 の文で決める実装（文の頭で出す、field と文の積）が 3 case とも通る（決定 8 の棄却 (a) の形が残る） | P3 | 受理 | 文の頭で出す形は #1 の (3) が兼ねて止める。積の形は (5) 順 3 の文（取込みID 5）＋ field 7 → Alert の中の link 7・`/csv-import/records/5` 無し、を足して止めた。残りの危険として書く案は採らなかった: 積の形は 32 の順 4 の文を言い換えると link が黙って消える棄却 (a) そのものの失敗で、同じ test の mock を 1 つ足せば止まり、画面の契約（S3 の行「field が数のときだけ」）も条件を field だけで決めているので、W3 の契約では来ない入力でも test が契約の形を写す。Matrix の Mutation-style の U11 の文を、どの case がどの実装を止めるかの対応に書き直した。反映: Matrix の U11・Mutation-style、packet の起草役が決めたこと 8 の棄却 (a)・S3 の Alert の link の行・AC3 の `ErrorState.tsx` の `rg` の注・Test Plan の mutation・Contract Audit の「単位や種類の混在」 |
| Opus #3: U11 の link の照合が Alert に限られず、Z004 タブの上の U8 の「取り込んだ記録を見る（取り消すときはここから）」に当たる。写し 2 の見る所もその link との識別を見ない | P3 | 受理 | U11 の link がある側の照合を Alert（`role="alert"`。`c667d52c` の `src/components/ui/alert.tsx` の 37 行）の中に限り、無い側は Alert の中の link と画面全体の href が `/csv-import/records/` で始まる link の両方を見る形にした（U8 の link は `/inventory/records` へ進むので当たらない）。写し 2 の見る所に、Z004 タブの上の「取り込んだ記録を見る（取り消すときはここから）」の link と見分けられることを足した。反映: Matrix の U11、packet の Test Plan の写し 2 |

- U11 の case は 5 つ（(1) 順 4 の文 7・field 7 → link 7、(2) 順 4 の文 9・field 7 → link 7・9 無し、(3) 順 4 の文 7・field null → link 無し、(4) 順 3 の文 5・field 無し → link 無し、(5) 順 3 の文 5・field 7 → link 7・5 無し）。写し 2 は今も (1) の mock。
- 不要になった記述: 無し（U11 の case の数を書いた所は Matrix の U11 の行だけで、そこを置き換えた。closure の項の「3 つの case」は履歴として残す）。
- Gated Amendment: 無し（plan-gate のままの是正で、Plan Commit は pending）。

closure 3（`af32f558`、closure 2 の是正の版）: Claude 側 fresh Opus 5.5 = approve（P1 0 / P2 0 / P3 1。closure 2 の #1〜#3 は closed）。P1 / P2 = 0 で遷移 7 の条件（fresh の reviewer の closure）を満たす。残った P3 1 件は Coordinator が plan-gate のまま直し、その版を Plan Commit にする（前例: PR #201 の遷移 8 で closure の P3 を直した版を Plan Commit にした）。

| finding | 重大度 | disposition | 是正（反映先） |
|---|---|---|---|
| Opus N1: U11 の (5) に使う順 3 の文も「取込みID」を含むので、field と「どの文にもある語」の積で link を出す実装（`related_csv_import_id != null && message.includes("取込みID")`、id は field）が (1)〜(5) を全部通る。順 4 の文から「取込みID」を外すと link が黙って消える（決定 8 の棄却 (a) の壊れ方） | P3 | 受理（Coordinator が直した） | Matrix の U11 に (6)（kind `import_error`、取込みID も数字も含まない文、field 7 → Alert の中に link 7）を足し、case の数を 6 にした。Matrix の mock の値の問い、packet の決定 8 の棄却 (a)・AC3 の注・Test Plan の mutation の case の範囲を (6) まで揃えた |

- 不要になった記述: 無し（case の範囲を書いた 4 か所を置き換えた。closure・closure 2 の項の case の数は履歴として残す）。
- Gated Amendment: 無し（plan-gate のままの是正）。
