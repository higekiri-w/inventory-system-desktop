# Plan Packet: EJ（電子ジャーナル）parser の core — 取引単位の構造復元（R3）

2026-09-23 起草。出典は `docs/Plans.md`「次の行動」1（owner 決定 2026-09-22「並走で EJ parser の core を合成データで進める」）と [Backlog](../../backlog.md#やると決めたもの順番未定)「実測とPOS系列の対応を取得・保存する」（「EJ parser の構造復元は並行可能だが、時刻分割は本 lane の結論を待つ」）。EJ を PLU 販売の本番開始の前提にする owner 判断（2026-09-18、[ADR SPEC-STK-TIME-D5](../../adr/2026-09-18-stocktake-time-evidence.md#spec-stk-time-d5-ejの完全と商品を判定可能を分ける)）の部品を、実機確認を待たずに作る。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

- Evidence Mode: github
- Phase: archive
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: 6a827b907949fb16dd220c426b29c7fa09f63264
- Amendments: none
- Coordinator: Opus 5.5（Claude Code main session、effort high）
- Writer: Opus 5.5 subagent（worktree `agent/ej-parser-core`）
- Plan Reviewer: Opus 5.5（fork でない fresh subagent、Writer と別 context。Codex は rate limit 中のため Opus のみ）
- Final Reviewer: Opus 5.5（fresh subagent）+ Codex（互いに独立、Writer・Plan Reviewer とも別 context。Final Review は Codex の復帰を待つ）
- Final Review Minimum: 2
- Human Gate: ready,merge

effort: owner 方針（2026-09-23）は Writer / review の Opus = medium。Claude Code の Agent tool では subagent ごとの effort を指定できないため、実行時は session の設定を継承する。実効値は run 報告に記録する。

manual なし: operator 画面・配布物・wire の変化がない IO 層だけの変更で、Windows native の確認対象がない。実物 EJ での構造確認は Coordinator が手元で行い（AC9）、owner の作業にしない。

座組の根拠と規則との不一致（append-only の記録）: 座組は owner 決定 2026-09-23（「Opus」= Opus 5.5、Writer / Coordinator / レビューに全面解禁、Plan Review は Codex 稼働時 fresh Opus + Astra・停止中は Opus のみ、Final Review は fresh Opus + Codex で Codex を待つ）による。現行の tracked 規則は `docs/AGENT_OPERATING_MANUAL.md:38`（D-056: 高自律・低制約適性 slot は read-only の Reviewer / Explorer 専任で Writer / Coordinator に割り当てない）と §3.4 表（`:97`、Opus = Claude Opus 5）のままであり、本座組と literal に衝突する。owner は D-056 の制限を Opus 5 の性格から決めたものとして Opus 5.5 へ引き継がないと決めた。tracked 規則の改訂は並走の「docs 復元 + 規則改訂」lane が所有し、本 packet は規則を変更しない。Execution Mode は現行 enum のうち、希少・最高能力 slot（Fable 5.1）が相談役として利用可能な期間を示す `fable-window` を選ぶ（owner は Fable 不在型の mode を廃止方向としたが enum は現行のまま）。Plan Reviewer と Writer は同じ model・同じ vendor で、`docs/DEV_WORKFLOW.md` Review Rules の vendor 条項（Writer が Codex の packet が対象）は literal に掛からない。同 model の残余 risk は owner 決定 2026-09-23（Codex 停止中は Plan Review を Opus のみで進める）で受容し、別 vendor の目は Final Review の Codex が担う。

遷移記録（append-only）:

- kickoff → spec-check → design → plan-draft → plan-gate（本 commit、plan-first）: Risk は R3（下記 Risk）。in-scope の source docs は Design Sources に列挙し、IO-08 の関数設計書の新設が要ると判定した（spec-check → design）。設計判断は本 packet の Spec Contract（IO-08-D1〜D10）で確定し、owner の判断を要する未決の論点は無い。source doc への反映は本 PR 内で Writer が行う（Z004 layout A packet `docs/archive/plans/2026-08-16-z004-layout-a-parser.md` と同じ「updated in this PR」形）（design → plan-draft）。packet と Test Design Matrix を同じ commit に置く（plan-draft → plan-gate）。
- plan-gate → plan-approved → implementing（2026-09-24、Coordinator、state-only）: Plan Review round 1（fresh Opus、P1 0 / P2 2 / P3 9、操作列は `具体的な反例あり`〈合計域の全角数字〉）→ 全件採用し是正 `8a364a3d` → round 2 closure（別の fresh Opus、P1/P2 = 0、P3 2、操作列は `成立`）→ P3 を反映。Plan Commit = `6a827b90`（plan-first `d4951245` → 是正を含む確定版）。実装は Opus 5.5 subagent の worktree run。
- implementing → archive（closeout、本 commit）: PR #94 squash merge `8bd2bc9a`（2026-09-25）。packet / Matrix を `docs/archive/plans/` へ移送し、Implementation Results / Review Response を記録した。Final Review closure（Opus、`8d2975de`）の P3 2 件を PR #94 の Follow-up の裁定どおり処理した: Spec Contract IO-08-D2 / D5 / D6 の文法は 29 を正本とする注記（Review Response）と `docs/architecture/io-task-specs.md` IO-08 の処理構造 5 の文言、Matrix の Test Matrix に Opus 是正（`e0cd106d`）の test 8 本の行と Mutation-style の 2 項。`docs/backlog.md` へ EJ の文法の後続 lane を起票した。

## Owner Effort Budget

- 介入回数上限: 6（当初 3〈Final Review の Codex 起動 1、Ready 1、merge 1〉。2026-09-25 に owner 承認で引き上げ「引き上げしていいよ」「その数で記録していいよ」: Coordinator が PR body に Opus 側の要約を載せたまま Codex broad を発注し独立性が崩れたため取り直しの relay 1 が増え、引き上げの承認と数え直しの確認で 2 回を使った。消費 4〈relay 77、relay 77b、引き上げの承認、数え直しの確認〉、残り Ready 1・merge 1）
- 実働時間上限: 15分（既定 30 分から引下げ。画面・L3 が無く、owner の作業は Codex 起動の 1 行と Ready / merge の判断に限られる見込みのため）
- relay 往復上限: 2
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md:49` の R3「POS CSV」と `docs/project-profile.md:90` の High-risk Changes（Z004 parser behavior、CP932 handling、negative return handling）を、新しい POS 取込み形式（EJ）の parser に当てた。本 change は DB・command・画面・生成物を変えず、merge gate の合否も変えない。それでも parser の復元規則は、後続の在庫連動が「この取引の明細は確かか」を判断する入口であり、誤復元は在庫の誤った増減に直結する。file の種類でなく影響で R3 とする。R4 の条件（実店舗データの露出）は Data Safety の規則で閉じる。実データは repo に入れず、実物での確認は値を出力しない手元の probe に限る。

## Goal

Goal Invariant:

### 最小完了条件

- 店のレジ（CASIO SR-S4000）が SD に保存する EJ の 1 file（24 バイト固定幅・CP932・CRLF）の生バイト列を、DB 非依存の純関数 `parse_ej` に渡すと、記録（取引・返品・入金 / 出金・精算・設定書込み）の列が元 file の行番号つきで返る。
- 通常販売と返品モードの取引は、観測済みの形式に一致する場合に限り、明細（印字名称・数量・単価・金額）へ復元される。一致しない取引は、その取引だけが「復元不能」として返り、明細を読み出せない型になる。どの行も黙って捨てない。
- 実物 6 本（repo 外）を同じ関数に通すと、全記録が復元済みまたは明細なしとして返り、復元不能と診断が 0 件になる（AC9）。

### 失敗定義

- 未知の行・未知のモード・点数や合計の不一致を含む取引から、明細が「確かなもの」として返る（後続が在庫へ反映できてしまう）。
- file 中のどこかの行が、記録にも先頭断片にも診断にも現れずに消える。
- 実データ（名称・金額・日時・番号）が repo・PR・診断文言に入る。
- 既存の Z004 / 日報 parser、command、bindings、DB の振舞いが変わる。

### 非目的

- 一連番号の連続性・区間の完全性・分割 file の連結・再出力の重複判定、Z004 との対応づけ、印字名称から商品への同定、実測をまたぐ取引の時刻分割（いずれも次の design lane「実測と POS 系列の対応を取得・保存する」と BIZ の責務。時刻分割は backlog:24 どおりその結論を待つ）。
- 日次の EJ 取込みの画面・BIZ・file 探索の実装（Spec Contract IO-08-D10 に設計の申し送りだけを置く）。
- 実物 6 本に現れない形式（訂正・取消・値引き・番号印字・点検を含む）を推測で受理すること。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

本 packet は data 契約（EJ の復元規則と出力型）を決めるため、表を置く。利用者はこの lane の時点では画面から EJ を扱わない。表の「利用者」は parser の呼出し側（後続の BIZ と訪店用の検証）であり、店の通常運用（毎日の締めで EJ を取り込み在庫へ反映する）はこの lane では達成しない。「この packet を完了できる」と「通常運用を達成できる」は別で、後者は次の design lane と日次取込みの lane（IO-08-D10）を要する。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 普段の一日（販売数件・入金・出金・替・精算 1 回）の EJ 1 file | `parse_ej` に生バイトを渡す | 記録の列。販売の記録は明細（名称・数量・単価・金額）つきの復元済み、入金 / 出金 / 替・設定書込み・精算は明細なし。診断 0 件 | 後続（次の design lane / BIZ）が区間の照合に使える | 形状は Contract Probe の実物 6 本で確認済み |
| 同じ日に精算 3 回（`_0001` に 2 回、`_0002` に 1 回。Contract Probe の実物と同じ並び） | 各 file を別々に渡す | file ごとの記録の列。file 名・file 内の日付で区間を切らない。連結はしない | 連結と完全性は次の design lane が決める | 分割の規則（どこで次の file になるか）は実機で未確認 |
| 翌日の file の先頭が前回精算の後の記録（前日付の設定書込み）で始まる | そのまま渡す | 先頭の記録も普通の記録として返る。前日付を異常扱いしない | 同上 | Contract Probe で 6 本中 5 本が該当 |
| 値引き（％- キー）・訂正・取消・番号印字を含む未観測の形式の取引 | そのまま渡す | その取引だけ復元不能＋診断（行番号つき）。他の取引は復元済みのまま | 後続はその取引を自動の在庫反映に使わない。形状を実物で確かめてから文法を足す | ％- キーの店での使用頻度は店の事実として owner に確認したい（報告の問い 1） |
| 取込み途中で切れた file（最終改行なし、または合計行の前で終わる） | そのまま渡す | 末尾の取引だけ復元不能＋診断。EOF を取引の終わりの根拠にしない | 再取得した file で再度 parse する | — |
| 訪店後、実物 EJ（repo 外）が手元にある | Coordinator が ignore 付きの probe test を環境変数で実行 | 件数だけの出力（記録の種類別件数・復元不能 0・診断 0） | 0 でなければ文法の不足を Gated Amendment で足す | AC9 |

## Scope

- S1 `src-tauri/src/io/ej_parser.rs`（新設）: IO-08 の実装（Spec Contract IO-08-D1〜D9）と、同じ file 内の `#[cfg(test)]` unit tests（Matrix の T-P / T-N / T-F / T-I）。合成 fixture は test 内の builder で 24 バイトの行を組み立てる（CP932 で encode し長さを assert する）。fixture file は置かない。実物確認用の `#[ignore]` test 1 本（T-R1）も同じ file に置く。T-R1 は環境変数 `INVENTORY_EJ_PROBE_DIR` の dir から拡張子が `.TXT`（大文字小文字を区別しない）の file だけを読み、読んだ file 数・読まなかった file 数と、記録・復元状態・診断 code の件数だけを出力する。`--ignored` で実行されたのに環境変数が無ければ panic する（何も確かめずに PASS しない）。
- S2 `src-tauri/src/io/mod.rs`: `pub mod ej_parser;` を 1 行追加する。
- S3 `src-tauri/tests/design_compliance_test.rs`: `build_doc_to_modules_map()` に `"29-io-ej-parser.md" => ["io::ej_parser"]` を追加する（`:158` の日報 parser と同じ形）。
- S4 `docs/function-design/29-io-ej-parser.md`（新設）: IO-08 の関数設計書。`docs/DOC_STYLE_GUIDE.md` の関数設計書テンプレート（親文書・入力ドキュメント、シグネチャ / 処理ステップ / エラーハンドリング）に従い、Spec Contract IO-08-D1〜D10 を正本化する。行の文法表（IO-08-D5）と Contract Probe の構造所見（値なし）を含める。節番号は `IO-08.1`〜`IO-08.10`（IO-08-D1〜D10 に対応）とし、`29-io-daily-report-parser.md` の `§29.n` と衝突させない。本 packet と後続の文書は新しい文書を略さず `29-io-ej-parser.md` と書く。
- S5 `docs/FUNCTION_DESIGN.md`: 「現時点の対象モジュール」（`:53` の IO-07 の次）と目次の IO 層（`:117` の IO-07 の次）へ IO-08 を 1 行ずつ追加する。冒頭の「時点証拠契約」節（`:3`〜`:17`）には触れない。
- S6 `docs/ARCHITECTURE.md`: IO 層の表（`:197` の IO-07 の次）へ IO-08 の行を追加し、サブドキュメント表（`:306`）の範囲を `IO-01〜IO-08` にする。
- S7 `docs/architecture/io-task-specs.md`: 末尾（IO-07 の後、`:227` の後）に `### IO-08: EJパーサー` 節を追加する。冒頭の「時点証拠契約」節（`:3`〜`:9`）には触れない。
- plan-first commit（Coordinator）: 本 packet、Test Design Matrix、`docs/Plans.md`「次の行動」への active link（PK4 は plan-first 段階から link を要求する）。Plans.md の編集は Coordinator が行う。

生成物: `src/lib/bindings.ts` は再生成しない（command / DTO の変更なし）。`docs/function-design/90-traceability.md` は再生成しない（新しい test は REQ token を持たず spec ID `IO-08-Dn` だけを使う。IO-08 を対応タスクに持つ REQ は無く、索引へ IO-08 を足しても生成結果は変わらない。AC7 の `--check` で確認する）。`src/routeTree.gen.ts` は対象外。

並走 lane との footprint: S1〜S4 は新設 file と、他 lane が編集しない 2 file（`io/mod.rs`、`design_compliance_test.rs`）。S5〜S7 の 3 file は ㉗ ADR の同期先一覧に含まれ、並走の ADR 修正 lane（`agent/stocktake-time-evidence-adr-fix`）が同じ file の「時点証拠契約」節を編集する可能性がある。本 lane の hunk は IO-07 の行・節の直後への追加だけで、その節とは行が重ならない。`docs/DEV_WORKFLOW.md` Wave Operation の「同じ source document を編集する lane は同居させない」を満たすかは、Coordinator が ADR 修正 lane の Scope 確定時に突合し、重なる場合は merge 順を決めて後着側が `origin/main` を 1 回 merge する（`docs/DEV_WORKFLOW.md` Stacked train の単段 merge）。㉘ runtime lane（`stocktake_service.rs`、`stocktake_repo.rs`、`csv_import_service/commit.rs`、`src/features/stocktake/**`）とは file が重ならない。

## Non-scope

- 一連番号の連続性・欠番、精算区間の完全性（`Proven` 等の区間判定）、分割 file の連結、再出力と真の重複の区別、Z004 の精算メタとの対応づけ（ADR SPEC-STK-TIME-D5 の 1〜3 点目。次の design lane と BIZ）。
- 印字名称から商品への同定、部門名との衝突、名称辞書（D5 の 5 点目。BIZ と次の design lane）。
- 時刻の解釈と時刻分割（D5 の 6 点目。backlog:24 のとおり次の design lane の結論待ち）。parser は印字された日時を文字列のまま返す。
- 未観測の形式（訂正・取消・値引き・番号印字・点検 / 精算以外のモード・小数数量・クレジット等の支払行）の受理。すべて復元不能として返す（IO-08-D5 / D6）。実物を採取してから Gated Amendment または後続 change で足す。
- 日次の EJ 取込みの画面・BIZ・command・file 探索（EcrDatas の探索、file 名の解釈）。設計の申し送りだけを S4 に置く（IO-08-D10）。
- 既存の Z004 parser（IO-02）・日報 parser（IO-07）・BIZ-03・command・bindings・DB の変更。
- 訪店用の検証 bin（`bin/ej_probe.rs` 等）。T-R1 の ignore 付き test で足りる。実機確認の形が決まってから要否を判断する。
- 新しい crate 依存（proptest 等）。既存の `sha2` / `encoding_rs` だけを使う。
- decision-log への新 D の追加。日次 EJ 取込みの owner 決定（2026-09-23）は S4 の IO-08-D10 に記録し、decision-log / ADR への昇格は次の design lane が行う（並走の規則改訂 lane が decision-log へ採番するため、連番の衝突を避ける）。

## Acceptance Criteria

- AC1: 合成 fixture の普段の一日の file（通常販売・数量行つき・同名反復・返品・現金ちょうど・入金・出金・替・設定書込み・精算）を `parse_ej` に渡すと `Ok` で、販売・返品の記録は `EjRestoration::Restored`、それ以外は `EjRestoration::NoItems`、`diagnostics` は空（`cargo test --lib io::ej_parser` の T-P1〜T-P13 が PASS）。
- AC2: 復元済みの明細は、印字名称・数量・単価（数量行があるときだけ）・金額を持ち、同名の行を合算せず、返品モードでも金額の符号を反転しない（T-P2 `parse_ej_quantity_line_applies_to_next_item` / T-P3 `parse_ej_repeated_same_name_lines_kept_separate` / T-P4 `parse_ej_return_mode_keeps_positive_amounts` が PASS）。
- AC3: 未知の行・未知のモード・数量×単価 / 点数 / 合計の不一致・負の明細金額・数量行の後に明細が無い・EOF で切れた取引・幅違反・最終改行なしは、その記録だけが `EjRestoration::Unresolved` になり、該当の `EjDiagnosticCode` と行番号が `diagnostics` に入る。同じ file の他の記録の復元結果は変わらない（T-N1〜T-N18 が PASS）。
- AC4: `Unresolved` の記録から明細を読み出す API が無い（`EjRestoration::Unresolved` は明細の field を持たない。型の定義を review で確認し、T-N1 が variant を assert する）。
- AC5: どの行も失われない: 先頭断片の行数 + 記録ヘッダの行数（記録ごとに 2）+ 記録本文の行数 = file の行数で、先頭断片・ヘッダ（`header_line_no` と次の行）・本文の行番号を合わせると 1〜N をちょうど 1 回ずつ覆う（T-I1 `parse_ej_every_line_is_accounted_for` が PASS）。診断の文言に行の生テキストを含めない（同じ test が全 diagnostic の message に fixture 内の名称文字列が含まれないことを assert する）。各診断の message は code ごとの固定文言と完全に一致する（T-I2 `parse_ej_diagnostic_messages_are_fixed_texts` が PASS）。
- AC6: CP932 として decode できない入力、記録ヘッダが 1 つも無い入力、空の入力は `Err(EjParseError::…)` になる（T-F1〜T-F3 が PASS）。
- AC7: 既存の契約が変わらない: `cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test` が PASS、`cargo test --test design_compliance_test` が PASS（`io::ej_parser::parse_ej` が `29-io-ej-parser.md` に対応づく）、`cargo run --bin generate_traceability -- --check` が差分なしで PASS、`src/lib/bindings.ts` と `docs/function-design/90-traceability.md` の差分が 0（repo root を問わない pathspec の `git diff --stat origin/main -- ':/src/lib/bindings.ts' ':/docs/function-design/90-traceability.md'` が空）。
- AC8: docs が同期している: S4〜S7 の反映後に `bash scripts/doc-consistency-check.sh` が ERROR 0。`29-io-ej-parser.md` に Spec Contract IO-08-D1〜D10 と行の文法表がある。
- AC9: 実物での確認（Coordinator、repo 外）: 実装 HEAD で `cd src-tauri && INVENTORY_EJ_PROBE_DIR=<実物のdir> cargo test --lib io::ej_parser -- --ignored real_ej_structure_probe --nocapture` を実行し、6 file すべてが `Ok`、`Unresolved` 0、`diagnostics` 0 で、出力が件数（読んだ file 数 6・読まなかった file 数、記録・復元状態・診断 code の件数）だけであること。結果は PR body に件数だけで記録する（値・名称・日時・番号を書かない）。0 でなければ、どの規則が足りないかを構造だけで報告し、Gated Amendment で文法を直す。
- AC10: repo に実データが入っていない: 追加・変更 file の fixture 文字列は合成（架空の名称・金額・日時・番号）であり、実物から写した値が無いことを review で確認する。実物の file・probe の出力 log を commit しない（`git status --short` に `.TXT` と log が現れない）。

## Design Sources

- Requirements / spec: IO-08 を対応タスクに持つ REQ は無い（`docs/spec/requirements.md`）。EJ を PLU 販売の本番開始の前提にする owner 判断は ADR SPEC-STK-TIME-D5（`docs/adr/2026-09-18-stocktake-time-evidence.md:153`）と Evidence（`:339`）。本 lane の出発点は `docs/backlog.md:24`。
- Architecture: `docs/ARCHITECTURE.md` IO 層の表（`:197`、IO 層は純粋なファイル形式処理・業務ロジックなし）、`docs/architecture/io-task-specs.md` IO-07（`:189`、CASIO adapter を IO に閉じる先例）、D-023（`docs/decision-log.md:162`、レジ依存部分を取り替えられる境界）。
- Function / command / DTO: `docs/function-design/23-io-z004-parser.md`（`:11` の「EJ parser は別 lane で、未知行の無視・純日計一致だけでの完全扱いは禁止」、`:13` 以降の Z004 parser の型と致命的エラー / 行単位エラーの二層）、`docs/function-design/32-biz-csv-import-service.md` の外部 probe 表（`:69`「EJ完全性」、`:70`「EJ復元」、`:74`）。command / DTO の変更なし。
- DB: 変更なし（parser は DB 非依存）。
- Screen / UI: 変更なし。日次 EJ 取込みの UI は IO-08-D10 の申し送りだけ。
- Decision log / ADR: ADR SPEC-STK-TIME-D5（本 lane の必須契約。`:19`「EJ parser の実装は別laneだが、下記の証拠条件と実行拒否はそのlaneの必須契約」）、D-023。
- 店の事実（repo 外の回答台帳 `.local/reports/store-premises/answer-ledger.md`）: L-042（SD に EJ が約 4 年分、CV17「電子ジャーナルを閲覧する」で取り込むと SD の XZ_BKUP と PC の EcrDatas に残る）、L-044（売上/EJ 保存設定は有効、EJ は Z004 と同じ SD フォルダ）、L-045 / TD-013 / TD-014（EJ の PC 取込みは月 1 回程度、毎日の精算に EJ 取込みを足すことを owner が了承）、L-046（EJ を PLU 本番の前提にする）、L-027 / L-028（返品は戻モード）、L-038（同日に複数回の精算はありうる）、L-014（その場の値引きは ％- キーでできる）、L-041（PLU 名称は 16 バイトまで）。本 packet は要旨だけを引く。

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | `docs/function-design/29-io-ej-parser.md`（新設、IO-08-D1〜D10）、`docs/FUNCTION_DESIGN.md` 索引、`docs/ARCHITECTURE.md` IO 表、`docs/architecture/io-task-specs.md` IO-08 節 | updated in this PR |
| Command / DTO / generated binding / wire shape | なし（command を追加しない。bindings 再生成なし） | existing sufficient |
| DB / transaction / audit / rollback / migration | なし（DB 非接触） | existing sufficient |
| Screen / UI / route state / Japanese wording | なし。日次 EJ 取込みの UI は IO-08-D10 で後続へ申し送る | intentionally deferred |
| CSV / TSV / report / import / export format | EJ の入力形式（24 バイト固定幅・CP932・CRLF・行の文法）を `29-io-ej-parser.md` と本 packet の Boundary / Wire Contract に記録 | updated in this PR |
| Durable decision / ADR | IO-08-D1〜D10 は `29-io-ej-parser.md` の局所決定。日次 EJ 取込みの owner 決定の decision-log / ADR への昇格は次の design lane | intentionally deferred |

## Registration / Generation Obligations

| 変更対象 | 登録・生成義務 |
|---|---|
| function-design doc 新設（29-io-ej-parser.md） | `src-tauri/tests/design_compliance_test.rs` の `build_doc_to_modules_map()` へ entry 追加（S3）/ 関数設計書の必須セクション（シグネチャ / 処理ステップ / エラーハンドリング）の充足（S4）。`parse_ej` 以外の `pub fn` を作らない（作るなら `29-io-ej-parser.md` に記載する） |
| source doc 新設 | 親文書 `docs/FUNCTION_DESIGN.md` の目次へ登録（S5）、`docs/ARCHITECTURE.md` の IO 表とサブドキュメント表の範囲（S6）、`docs/architecture/io-task-specs.md` の IO-08 節（S7） |
| REQ / coverage | 該当なし（新しい test は REQ token を持たない。`generate_traceability -- --check` で差分なしを確認、AC7） |
| Tauri command / route / operator 画面 / §5.5 | 該当なし |

file 名の番号: `docs/DOC_STYLE_GUIDE.md` は `2x` = IO 層とし、`20`〜`29` はすべて使用済み。`29-io-ej-parser.md` は同じ CASIO SR-S4000 adapter の日報 parser（`29-io-daily-report-parser.md`）と番号を共有する。番号の一意性を求める規則・checker は無い（`design_compliance_test` と `generate_traceability` は file 名全体で引く）。IO 帯の外の番号（`8x` 等）は層の規約を破るため、既存 file の番号振り直しは多数の link を壊すため採らない。参照は常に file 名全体で書く。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| ADR SPEC-STK-TIME-D5 / 32-biz 外部 probe「EJ復元」 | `29-io-ej-parser.md` IO-08.1 | IO-08-D1 | Z004 と同じ純関数・生バイト入力・SHA-256。file 探索・file 名・現在時刻を core に入れない（runtime と訪店の確認で共用するため）。file 名から日付を取る案は、file 先頭が前日付の記録で始まる実物があるため不採用 | `parse_ej` | T-P11 / T-F1〜T-F3 |
| 同上 | `29-io-ej-parser.md` IO-08.2 | IO-08-D2 | 行は CRLF 区切りの 24 バイト固定幅。Z004 の改行正規化（NEL / LF / CR を LF へ）は採らない: 固定幅の形式で孤立した LF / CR は破損の兆候であり、正規化すると幅の検査が効かない | 行分割・幅検査 | T-N10 / T-N11 |
| 同上 | `29-io-ej-parser.md` IO-08.3 | IO-08-D3 | 記録の境界は 2 行のヘッダ（モード欄 5 バイト + 分精度の日時、4 桁 - 6 桁の番号行）。ダッシュ 24 本の行を境界にする案は、実物で明細と合計の区切りだったため不採用。番号は意味が検証されるまで文字列（23-io の精算メタと同じ方針） | ヘッダ検出 | T-P1 / T-P9 / T-N3 |
| 同上 | `29-io-ej-parser.md` IO-08.4 | IO-08-D4 | 最初のヘッダより前の行は記録に含めず先頭断片として返す。EOF を取引の終わりの根拠にしない（q7 相談の `Unsynchronized` 起点の考え方を最小化） | 先頭断片・閉じ判定 | T-N9 / T-N12 |
| ADR D5 4 点目「未知の行を黙って除外しない」 | `29-io-ej-parser.md` IO-08.5 | IO-08-D5 | 記録の種類と記録内の位置で行を分類し、どれにも当たらない行は Unknown として残す。行の形だけで分類する案は、区切り後の「現金」行が明細と同じ形になるため不採用 | 行分類 | T-P6 / T-N1 / T-N2 / T-I1 |
| ADR D5 3〜4 点目 / 32-biz「EJ復元」の取引内点数 | `29-io-ej-parser.md` IO-08.6 | IO-08-D6 | 区切りより前だけを明細とし、数量行は直後の 1 明細だけに掛ける。点数行 = Σ数量、合計（なければ現金）= Σ金額、数量×単価 = 金額を記録内で照合する。同名行は合算しない。返品は符号を反転しない（効果の解釈は BIZ）。Z004 との照合は非目的 | 明細復元・照合 | T-P1〜T-P6 / T-N4〜T-N8 / T-N13 |
| 同上 | `29-io-ej-parser.md` IO-08.7 | IO-08-D7 | 入金 / 出金 / 替・設定書込み・精算は明細なしとして返す。精算票の値は解釈しない（区間・系列は次の design lane） | 記録種別 | T-P7 / T-P8 |
| ADR D5 冒頭「完全」と「判定可能」を分ける | `29-io-ej-parser.md` IO-08.8 | IO-08-D8 | 復元不能の記録から明細を読めない型にする（後続が誤って在庫へ使う経路を型で閉じる）。bool の flag 案は読み出しを防げないため不採用。診断の文言に生の行を入れない | `EjRestoration` / `EjDiagnostic` | T-N1 / T-I1 |
| ADR D5 1〜3・5・6 点目 | `29-io-ej-parser.md` IO-08.9 | IO-08-D9 | 系列・完全性・連結・重複・Z004 対応・商品同定・時刻を IO で判断しない（信用と系列は BIZ / 外部 probe の責務。23-io:11 と同じ線引き） | なし（非実装の明記） | review |
| owner 決定 2026-09-23（台帳 TD-013 / TD-014 / L-045） | `29-io-ej-parser.md` IO-08.10 | IO-08-D10 | 日次取込みの支援は設計の申し送りだけを置き、実装しない（理由は Design Readiness） | なし | review |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: 本 PR で `29-io-ej-parser.md` を新設し、IO-08-D1〜D10、行の文法表、Contract Probe の構造所見（値なし）、却下案を置く。ADR D5 と 32-biz 外部 probe 表が上位の要求。
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: IO-08-D1〜D10 を `29-io-ej-parser.md` へ正本化する（本 PR）。日次 EJ 取込みの owner 決定は `29-io-ej-parser.md` の IO-08-D10 に記録し、decision-log / ADR への昇格は次の design lane（Non-scope の理由どおり）。
- Assumptions and constraints: 文法は実物 6 本（2026-08-14〜08-31 の採取、うち 1 日は複数精算）の観測に限られる。未観測の形式はすべて復元不能に倒すので、文法の不足は「取りこぼし」ではなく「復元不能の増加」として現れる。PLU 名称が 16 バイト以内であること（L-041）は parser の前提にしない（名称欄の幅は行の文法で決まる）。
- Deferred design gaps, risk, and follow-up target: (1) 系列・完全性・連結・Z004 対応・商品同定・時刻分割 → 次の design lane（backlog:24）。(2) 訂正・取消・値引き・番号印字・点検の実物形状 → 訪店時の採取（owner の許可範囲 TD-016 の拡張が要る。報告の問い 1）。(3) 日次 EJ 取込みの UI / BIZ → IO-08-D10 の申し送り。backlog への entry は closeout で Coordinator が足す。
- Test Design Matrix can cite design decision IDs or source doc sections: 可（IO-08-D1〜D9）。
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: 「どの行も失われない」（AC5）の例外は無い。先頭断片・ヘッダ・本文・Unknown のいずれかに必ず入る。「Unresolved から明細を読めない」は型で保証し、例外の経路（raw の `lines` から明細を組み直す）は呼出し側の責任として `29-io-ej-parser.md` に明記する。`lines` は診断と訪店の確認のために残す。

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | EJ の固定幅・ラベル・モード欄は CASIO SR-S4000 の adapter の事実。出力型（記録・明細・復元状態・診断）は後続の BIZ が使う core 側の契約で、レジが替わっても保つ形にする（D-023、ECR+ の終了とリース満了でレジ入替えの可能性がある） | `29-io-ej-parser.md`、IO-08-D5 / D8 |
| Fact check / design decision split | 観測した事実: 行幅・文字コード・改行・記録ヘッダ・区切りの位置・数量行・点数と合計の一致・番号の +1 の並び・file 先頭の前日付・1 file 内の複数精算（Contract Probe）。app の決定: 位置による分類・fail-closed・照合 3 種・型による読み出し禁止（IO-08-D5〜D8）。番号の意味（4 桁の欄・6 桁の採番規則）と精算票の Z 番号の意味は未検証で、IO は解釈しない | Contract Probe、`29-io-ej-parser.md` |
| Lifecycle / retry | parser は純関数で状態を持たない。同じ入力には同じ結果（file_hash も）。途中で切れた file は末尾の記録だけ復元不能になり、再取得した file で再度 parse すれば復元される | T-N9 / T-N10 |
| Operator workflow | この lane で operator の操作は変わらない。店の日次運用へ EJ 取込みを足す設計は IO-08-D10 で後続へ | IO-08-D10 |
| Replacement path | レジが替われば `ej_parser.rs` と `29-io-ej-parser.md` を取り替え、後続 BIZ が使う出力型は保つ。Z004 / 日報 parser とは独立 | `29-io-ej-parser.md` |
| Data safety / evidence | 実物は repo に入れない。確認は値を出力しない手元の probe と、件数だけを出す ignore 付き test に限る。診断文言に生の行を入れない（後の log・画面への流出を防ぐ） | Data Safety、AC5 / AC9 / AC10 |
| Reporting / accounting semantics | 取引内の明細・点数・合計・返品モード・入金 / 出金を区別して返し、復元直後に商品別の純計へ潰さない。返品は金額の符号でなくモードで区別する | IO-08-D6 / D7 |
| Manual verification | 自動 test で証明できないのは実物の形状への一致だけで、AC9 の Coordinator 手元確認で閉じる。Windows native L3 は不要 | AC9 |
| 環境・再現性 | 新しい環境依存なし（純 Rust、既存 crate のみ）。T-R1 は ignore 付き test で CI では実行しない。`--ignored` で実行したのに環境変数が無ければ panic する | — |

## Design Readiness

- Existing design docs are sufficient because: 上位の要求（ADR D5、32-biz 外部 probe 表、23-io:11）は既存。不足は IO-08 の関数設計そのもので、本 PR の `29-io-ej-parser.md` 新設（IO-08-D1〜D10）で満たす。
- Source docs updated in this PR: `29-io-ej-parser.md`（新設）、FUNCTION_DESIGN.md（索引 2 行）、ARCHITECTURE.md（IO 表 1 行 + 範囲）、io-task-specs.md（IO-08 節）。
- Design gaps intentionally deferred: Non-scope の各項目。日次 EJ 取込みの UI 支援は設計の申し送りだけで、画面・BIZ の設計と実装は後続。理由は次の 3 点。(1) owner 決定の「EJ が欠けたら知らせる」の「欠け」は、番号の採番規則・精算との対応・file の分割規則が決まって初めて定義できる。これは実機確認を着手条件にした次の design lane の出力であり、いま画面を設計すると未検証の意味を固定してしまう。(2) 取込み画面と取込み BIZ（`55-ui-csv-import.md`、`32-biz-csv-import-service.md`、`csv_import_service`）は ㉗ ADR の同期先と ㉘ runtime lane の footprint に入っており、並走では disjoint にできない。(3) EJ を PC へ移す操作（CV17「電子ジャーナルを閲覧する」）はアプリの外にあり、アプリ側の支援（EcrDatas の探索、file 名の解釈、欠けの通知、Windows L3）は file 探索の設計と実機での確認を要する。parser core はこれらのどれにも依存せずに先に作れる。
- Durable decisions discovered in this plan and promoted to source docs: IO-08-D1〜D10（`29-io-ej-parser.md`）。

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): IO-08 単独の新設。呼出し側（BIZ）はまだ無い。IO は業務判断（系列・完全性・同定・時刻）を持たない（IO-08-D9）。
- Backend function design: `parse_ej` 1 関数と出力型を `29-io-ej-parser.md` に正本化する。
- Command / DTO / data contract: command / DTO / bindings の変更なし。内部の Rust 型だけ。
- Persistence / transaction / audit impact: なし。
- Operator workflow / Japanese UI wording: 画面なし。診断の日本語文言は開発者向けで、画面に出す wire は無い。
- Error, empty, retry, and recovery behavior: 致命的エラー 3 種（IO-08-D1）、記録単位の復元不能（IO-08-D8）、先頭断片（IO-08-D4）。再試行は再取得した file の再 parse。
- Testability and traceability IDs: IO-08-D1〜D9 を test 名の近くの comment に書く（`// IO-08-D6: …`）。REQ token は使わない（該当 REQ が無い）。

## Contract Probe

- 実物 EJ の形状（行幅・文字コード・記録の文法）: Coordinator 手元の構造 probe `python3 ej_structure_probe.py <実物のdir>`（repo 外の scratchpad、値・名称・日時・番号を出力せず件数と種別名だけを出す）→ `files 6 cp932_strict_ok 6 ends_crlf 6` / `line_byte_widths {24: 309}` / `leading_lines 0` / 記録ヘッダのモード `normal 25 / 戻 2 / 精算 7 / PGM 7` / 取引の記録 `item_records 11` で数量×単価・点数 = Σ数量・合計（なければ現金）= Σ金額・金額と点数が非負の 4 検査すべて `11` / 記録本文の行種の並びは 13 種で、すべて IO-08-D5 の文法に収まる（`PGM | SEP STATUS SEP`、`normal | QTY ITEM+ SEP COUNT 対象計 内税 合計 お預り お釣`、`normal | ITEM SEP COUNT 対象計 内税 現金`、`normal | 入金` / `出金` / `替`、`戻 | ITEM+ SEP COUNT 対象計 内税 合計 お預り お釣`、`精算 | ZTITLE 総売 AMT 純売 AMT 現金在高 [対象計 内税 消費税合計] 純客 SEP ZEND STATUS+` 等）/ ダッシュ 24 本の行は取引の境界でなく明細と合計の区切り（区切りの後の「現金」行は明細と同じ形）/ 番号行の 4 桁の欄は `header_prefix_distinct 1`、6 桁の欄は記録ごとに +1（`serial_steps {1: 35}`）で精算をまたいでも戻らず、連続する日付の file 間も +1（`file_boundary_step_1: 4`、採取していない期間を挟む 1 か所だけ差が開く）/ 数字の字形（同じ probe の `digits` 行）: `合計` / `お預り` / `お釣` / `入金` / `出金` / `現金` はすべて `fullwidth_digits`（例 `digits 9 合計 fullwidth_digits nonneg cur`）、明細は `digits 18 ITEM ascii_digits nonneg cur`、`対象計` / `内税` / 精算票の値は `ascii_digits` / 通貨記号なしの負値は `digits 4 AMT_ONLY ascii_digits neg nocur` と `対象計` / `内税` / `消費税合計` / `現金在高` 各 `2`・`総売` / `純客` 各 `1`（返品だけの精算 2 回分）/ 1 file に精算 2 回が `1` 本（その日は `_0002` にもう 1 回で計 3 回）/ file 先頭の記録が file 名の日付より前の日付を持つ file が `5` 本 -> 文法は確定。番号の採番規則（取扱説明書の「精算ごとにリセット」と観測の不一致を含む）と Z 番号の意味は未検証のまま IO で解釈しない（IO-08-D3 / D9）。
- 観測していない形式（訂正・取消・値引き・番号印字・点検・小数数量・クレジット等）: 実物 6 本に現れない（上記 probe の行種に無い）-> 受理規則を作らず、復元不能に倒す（IO-08-D5 / D6）。実物は訪店時の採取を待つ。
- 既存 crate の前提（`encoding_rs::SHIFT_JIS` の strict decode が 0x5C を U+005C に写し、全角の円記号・全角の読点を含む行を decode できる）: 既存の Z004 parser が同じ decode を使っている（`src-tauri/src/io/z004_parser.rs:118`）-> 実装時に T-P10 で確認する。未実測。
- 未検証の外部前提（OS / 外部ライブラリの新規依存）: なし。

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| IO-08-D1 純関数・生バイト入力・file_hash・致命的エラー 3 種 | `parse_ej`、`EjParseError` | T-P11 / T-F1 / T-F2 / T-F3 | AC9（実物 6 本が `Ok`） |
| IO-08-D2 CRLF 区切り・24 バイト固定幅・最終改行 | 行分割・幅検査 | T-N10 / T-N11 | — |
| IO-08-D3 2 行ヘッダ・モード欄・日時と番号は文字列 | ヘッダ検出・`EjMode` | T-P1（先頭 0 の保持）/ T-P4 / T-P8 / T-P9 / T-N3 | — |
| IO-08-D4 先頭断片・EOF は閉じの根拠にしない | 先頭断片・閉じ判定 | T-N9 / T-N12 | — |
| IO-08-D5 位置による行分類・Unknown を残す | 行分類 | T-P6 / T-N1 / T-N2 / T-N13 / T-I1 | AC9（実物で Unknown 0） |
| IO-08-D6 明細の復元と記録内照合（数量×単価・点数・合計 / 現金・非負・同名非合算・符号非反転） | 明細復元 | T-P1〜T-P5 / T-P10 / T-P12 / T-P13 / T-N4〜T-N8 / T-N17 / T-N18 | AC9 |
| IO-08-D7 明細を持たない記録（入金 / 出金 / 替・設定書込み・精算） | 記録種別 | T-P7 / T-P8 / T-N14 / T-N15 / T-N16 | AC9 |
| IO-08-D8 復元状態の型・診断の範囲・文言に生の行を入れない | `EjRestoration`、`EjDiagnostic`、`header_line_no` | T-N1 / T-I1 / T-I2 | review（型に明細の field が無い） |
| IO-08-D9 IO が判断しないこと | 非実装 | なし | non-scope（review で `parse_ej` が番号の連続・日付・名称辞書を見ないことを確認） |
| IO-08-D10 日次取込みとの接続（申し送り） | なし | なし | non-scope（`29-io-ej-parser.md` の記載を review） |
| 隣接: ADR D5「未知の行を黙って除外しない」 | IO-08-D5 / D8 | T-I1 / T-N1 | — |
| 隣接: ADR D5「日計合計一致だけを完全性の証明にしない」 | 取引内の照合は完全性の証明ではない旨を `29-io-ej-parser.md` に明記 | なし | non-scope（区間の完全性は次の design lane） |
| 隣接: 23-io:11「番号は意味が検証されるまで文字列」「信用フラグを立てない」 | IO-08-D3 / D9 | T-P1 | — |
| 隣接: INV-6 相当の file_hash（生バイト SHA-256、小文字 hex 64 文字） | `parse_ej` | T-P11 | — |
| 隣接: 既存 parser（IO-02 / IO-07）と BIZ-03 の不変 | 変更なし | 既存 test（非接触、AC7） | non-scope |
| 隣接: design compliance の doc ↔ module 対応 | S3 | `design_compliance_test`（AC7） | — |

## Test Plan

Test Design Matrix: [test-matrices/2026-09-23-ej-parser-core.md](test-matrices/2026-09-23-ej-parser-core.md)

- targeted tests: `cd src-tauri && cargo test --lib io::ej_parser`（T-P1〜T-P13、T-N1〜T-N18、T-F1〜T-F3、T-I1、T-I2）
- negative tests: 未知の行（区切りの前 / 後）、未知のモード、照合 3 種の不一致、負の金額、数量行の孤立、EOF で切れた取引・精算、最終改行なし、幅違反、先頭断片、小数数量、decode 失敗、ヘッダなし、空入力
- compatibility checks: `cargo test` 全体（既存 parser・BIZ の test が不変）、`cargo test --test design_compliance_test`、`cargo run --bin generate_traceability -- --check`、bindings / traceability の差分 0（AC7）
- data safety checks: fixture が合成であること（AC10）、診断文言に生の行が入らないこと（T-I1）、T-R1 の出力が件数だけであること（AC9 の実行時に Coordinator が確認）
- main wiring/integration checks: 呼出し側はまだ無い（非目的）。`io/mod.rs` への登録と design compliance の対応づけまで（S2 / S3）
- L3: なし（Human Gate に manual を置かない）

## Boundary / Wire Contract

- producer: CASIO SR-S4000 の売上/EJ 保存設定が SD に書く EJ の text file（file 名は `EJyymmdd_NNNN.TXT` の形を観測。本 lane は file 名を解釈しない）。PC へは CV17「電子ジャーナルを閲覧する」で取り込む（台帳 L-042）。
- consumer: `io::ej_parser::parse_ej`（本 lane）。その先の BIZ（区間の照合・商品同定・在庫反映）は後続。
- wire type: 生バイト列。CP932、改行 CRLF、BOM なし、各行 24 バイト固定幅（全角 1 文字 = 2 バイト）。
- internal type: `EjParseResult`（file_hash、先頭断片の行、記録の列、診断の列）。記録 = ヘッダ（モード、印字日時の文字列、4 桁の欄、6 桁の番号）+ 本文の行（行番号・生の文字列・行種）+ 復元状態（`Restored`：明細の列と点数 / `NoItems` / `Unresolved`：理由の code の列）。明細 = 名称行の行番号、印字名称、数量（整数）、単価（数量行があるときだけ）、金額（整数、円）。
- precision/range: 金額・数量・単価は `i64`。金額の token は 1 つの共通規則で読む: 任意の `-`、任意の通貨記号（半角 `\` = CP932 の 0x5C、全角 `￥`）、数字（ASCII `0`〜`9` または全角 `０`〜`９` = U+FF10〜U+FF19）、桁区切り（半角 `,` / 全角 `，`）。数字・桁区切り・通貨記号の幅が 1 つの token の中でそろわなければ受理しない（半角 `\` + ASCII 数字 + 半角 `,`、または全角 `￥` + 全角数字 + 全角 `，`。`-` は半角だけ）。`替` の行は金額 token を持たない（`替` + 空白 + 半角カナの文字列）。観測した字形: `合  計` / `お預り` / `お  釣` / `入金` / `出金` / `現金` の金額は全角 `￥` + 全角数字 + 全角 `，`。明細・`対象計`・`内税`・精算票の値は ASCII 数字（明細は半角 `\`）。精算票では `AmountOnly` と `対象計` / `内税` / `消費税合計` / `現金在高` / `総売` / `純客` が通貨記号なしの負値（`-` + ASCII 数字）になることがある（精算 7 回中 2 回）。日時は分精度の文字列のまま（変換・時計の信用をしない）。番号は先頭 0 を保つ文字列。小数の数量は未観測のため受理しない（復元不能）。
- round-trip path: なし（読み取り専用。parser は書き出さない）。
- invalid input: decode 失敗・ヘッダなし・空は致命的エラー。それ以外の異常は記録単位の復元不能 + 診断（IO-08-D8）。
- compatibility: 新しい入力形式で、既存の形式・保存データとの互換の論点は無い。未観測の形式が見つかった場合は受理規則を足す（既存の復元結果を変えない追加に限る）。

## Review Focus

- 失敗定義の 1 点目: 未知・不一致を含む取引から明細が返る経路が無いか。特に、区切りの前の行の分類（名称と金額の間の空白、半角 / 全角の通貨記号）、数量行の掛かり方、合計が無く現金だけの取引、返品モードの扱い。
- 「どの行も失われない」（AC5）の不変条件が、幅違反・最終改行なし・先頭断片・EOF で切れた記録でも成り立つか。
- IO が判断しないこと（IO-08-D9）を越えていないか（番号の連続を見て診断を出す、file 名や日付で区間を切る、名称を正規化する、等）。
- Contract Probe の文法が `29-io-ej-parser.md` の文法表と一致し、実値を含まないか。fixture が合成か（AC10）。
- 座組の根拠（owner 決定 2026-09-23）と D-056 の不一致の記録が十分か。
- IO-08-D10 の判断（日次取込みの UI は申し送りだけ）の理由が妥当か。

## Spec Contract

Contract ID: IO-08-D1〜D10（source doc 正本は本 PR で新設する `docs/function-design/29-io-ej-parser.md`）

- IO-08-D1 入力と致命的エラー: `fn parse_ej(raw_bytes: &[u8]) -> Result<EjParseResult, EjParseError>`。純関数・DB 非依存・副作用なし。file 名・file 探索・現在時刻を受け取らない。`file_hash` は生バイト列の SHA-256（小文字 hex 64 文字）。致命的エラーは 3 種で、部分結果を返さない: `DecodeFailed`（どれかの行が CP932 strict で decode できない）、`NoRecords`（記録ヘッダが 1 つも無い）、`Empty`（0 バイト）。文言は「〜。ファイル形式を確認してください」の既存規約に合わせる。
- IO-08-D2 行の単位: 生バイトを CRLF で分割する。最後の CRLF の後が空なら行として数えない。空でなければ最後の行として扱い、診断 `MissingFinalNewline` を出し、その行を含む記録を復元不能にする。各行は 24 バイトちょうどでなければ診断 `InvalidWidth` を出し、その行は Unknown として残し、その行を含む記録を復元不能にする（孤立した LF / CR はこの形で現れる）。改行の正規化はしない。行番号は 1 始まり。
- IO-08-D3 記録ヘッダ: 連続する 2 行が次の両方を満たすとき、記録の始まりとする。1 行目 = 先頭 5 バイトのモード欄 + `YYYY-MM-DD HH:MM`（16 バイト）+ 空白 3 バイト。2 行目 = 空白 13 バイト + 数字 4 桁 + `-` + 数字 6 桁。モード欄を空白除去した値で `EjMode` を決める: 空 → `Normal`、`戻` → `Return`、`精算` → `Settlement`、`PGM` → `Program`、それ以外 → `Unrecognized(raw)`（その記録は復元不能、診断 `UnrecognizedMode`）。日時は分精度の文字列のまま返し、変換・妥当性の判断・時計の信用をしない。4 桁の欄と 6 桁の番号は先頭 0 を保つ文字列で返し、連続・欠番・リセットを IO で判断しない。
- IO-08-D4 先頭断片と EOF: 最初のヘッダより前の行は記録に入れず `leading_lines` に行番号つきで返し、診断 `LeadingFragment`（範囲 file）を出す。記録の本文は次のヘッダの直前まで、最後の記録は EOF まで。EOF を記録の閉じの根拠にせず、閉じは IO-08-D6 / D7 の必須行で判断する。
- IO-08-D5 行の分類（位置で決める。文法表は `29-io-ej-parser.md`）: 通常・返品の記録は、最初の区切り（`-` 24 個）より前を明細域、後を合計域とする。明細域の行は `Quantity`（空白 + 整数 + ` 点` + 空白 + `@` + 単価 + 空白）または `Item`。`Item` は行末（末尾の空白を除く）から通貨記号つきの金額 token を取り、その直前の空白の連続より前をすべて名称とする（名称は空白でない文字で始まり、名称の中の空白を許す）。合計域の行は `ItemCount`（空白 + 整数 + ` 点` + 空白）と、ラベル `対象計` / `内税` / `合  計` / `お預り` / `お  釣` / `現金` で始まりその後が空白の `Labeled`。区切りの無い通常の記録は、明細域の規則より先に 1 行の `入金` / `出金` / `替` の `Labeled` かを判定し、本文がちょうどその 1 行なら明細を持たない記録とする。`替` の行は金額 token を持たない（ラベル + 空白 + 文字列、値を解釈しない）。`Labeled` は prefix だけで判定し、金額を読むのは D6 の `合  計` / `現金` だけとする。精算の記録は、`NNNN 日計明細 ... Z NNNN` の `SettlementTitle`（両端の数字を文字列で保持、意味は解釈しない）、ラベル `総売` / `純売` / `純客` / `現金在高` / `対象計` / `内税` / `消費税合計` の `Labeled`、空白 + 金額 token だけの `AmountOnly`（符号つき・通貨記号の有無を問わない）、区切り、`日計明細` だけの `SettlementEnd`、`Status`。設定書込み（`PGM`）の記録は区切りと `Status`。`Status` は `SD設定書込み` / `SDｶｰﾄﾞ保存` / `ｽﾏ-ﾄﾌｫﾝ送信` で始まる行で、結果欄（観測は `正常終了`）を文字列で保持し、成否を判断しない。どれにも当たらない行は `Unknown` として残し、診断 `UnknownLine` を出し、その記録を復元不能にする。ラベルの判定は位置ごとの候補に限る（明細域では「現金」で始まる名称も `Item`）。合計域の `合  計` / `お預り` / `お  釣` / `現金` と `入金` / `出金` の金額は全角数字で印字される（Boundary / Wire Contract の precision/range）。
- IO-08-D6 明細の復元（`Normal` / `Return`）: 復元済みにする条件は全部そろうこと。区切りがあり、本文がヘッダだけではない。明細域の全行が `Item` または `Quantity` で、`Quantity` の直後の行が `Item` であり、数量 × 単価 = その `Item` の金額。明細の金額は 0 以上（0 円の明細は受理する）。合計域に `ItemCount` があり、その値 = 明細の数量の合計。`合  計` の金額（無ければ `現金` の金額。どちらも全角数字の token を読む）= 明細の金額の合計（どちらも無ければ `IncompleteRecord`）。`対象計` / `内税` / `お預り` / `お  釣` の値は照合しない。合計域に Unknown が無い。数量行が掛かる明細の数量はその値、それ以外の明細の数量は 1。同じ名称の行を合算しない。返品モードでも符号を反転しない（効果の解釈は BIZ）。条件が 1 つでも欠ければ復元不能で、欠け方に応じて診断 `InconsistentRecord`（照合の不一致・負の金額・数量行の孤立）または `IncompleteRecord`（本文がヘッダだけ・区切り・`ItemCount`・合計 / 現金の欠落）を出す。記録内の照合は取引の構造の確認であり、区間の完全性の証明ではない。
- IO-08-D7 明細を持たない記録: 通常の記録で本文が `入金` / `出金` / `替` の 1 行だけ、`Program` で本文が区切りと `Status` だけ（`Status` が 1 行以上）、`Settlement` で本文の先頭行が `SettlementTitle`・`SettlementEnd` があり全行が既知の行種、のとき `NoItems`。`Program` の本文に区切り・`Status` 以外の行があれば `UnknownLine`、`Status` が無ければ `IncompleteRecord`。`Settlement` の本文の先頭が `SettlementTitle` でなければ `IncompleteRecord`、未知の行があれば `UnknownLine` で、いずれも復元不能。精算票の値（総売・純売・現金在高・対象計・内税・消費税合計・純客）を解釈せず、値の検査もしない（負値・通貨記号の有無を問わず、ラベルで始まれば `Labeled`）。`Settlement` に `SettlementEnd` が無ければ `IncompleteRecord` で復元不能。
- IO-08-D8 復元状態の型と診断: `EjRestoration = Restored { items, item_count } | NoItems | Unresolved { reasons }`。`Unresolved` は明細の field を持たない。記録はヘッダ 1 行目の行番号 `header_line_no`（2 行目は `header_line_no + 1`）と本文の行（行番号・生の文字列・行種）を常に保持する（診断と訪店の確認のため。そこから明細を組み直すのは呼出し側の責任で、`29-io-ej-parser.md` に明記する）。`EjDiagnostic = { line_no（範囲 file のときは無し）, code, scope（Line / Record / File）, message }`。code は `UnknownLine` / `InvalidWidth` / `UnrecognizedMode` / `InconsistentRecord` / `IncompleteRecord` / `MissingFinalNewline` / `LeadingFragment` の 7 種。message は code ごとの固定文言（下の 7 つ）と完全に一致し、行の生の文字列（名称・金額・日時・番号）を含めない。`UnknownLine`「解釈できない行があります。この記録は明細を復元できません」、`InvalidWidth`「行の長さが24バイトではありません。この記録は明細を復元できません」、`UnrecognizedMode`「未知のモードの記録です。この記録は明細を復元できません」、`InconsistentRecord`「取引内の数量・点数・合計が一致しません。この記録は明細を復元できません」、`IncompleteRecord`「記録に必須の行がありません。この記録は明細を復元できません」、`MissingFinalNewline`「ファイルの最後の行に改行がありません。最後の記録は途中で切れたものとして扱います」、`LeadingFragment`「最初の記録より前に行があります。ファイルは記録の途中から始まっています」。
- IO-08-D9 IO が判断しないこと: 番号の連続・欠番・リセット、精算区間の完全性、分割 file の連結、再出力と重複、Z004 の精算メタとの対応、印字名称から商品への同定と部門名との衝突、日時の解釈と時刻分割、file 名の日付。これらは次の design lane と BIZ が、`parse_ej` の出力を入力にして決める。
- IO-08-D10 日次取込みとの接続（設計の申し送り、実装しない）: owner 決定 2026-09-23 = 店の EJ の PC 取込みは今は月 1 回程度で、毎日の精算に EJ の取込みを足す。アプリは Z004 と一緒に EJ を拾い、欠けたら知らせる。確定している事実 = EJ は CV17「電子ジャーナルを閲覧する」で取り込むと SD の XZ_BKUP と PC の EcrDatas に残る（台帳 L-042）、売上/EJ 保存設定は有効で EJ は Z004 と同じ SD フォルダにある（L-044）、1 file に複数の精算が入ることがあり file の先頭は前回精算の後の記録で始まる（Contract Probe）。後続が決めること = 「欠け」の定義（番号の採番規則・精算との対応・file の分割規則。次の design lane の実機確認の結果による）、EcrDatas からの file の選び方と file 名の扱い、取込み画面での置き場所と文言、Windows L3。`parse_ej` はこれらに依存しない入力（1 file の生バイト）と出力（記録の列）を提供する。

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| IO-08-D1 | S1 `parse_ej` | `parse_ej_file_hash_is_raw_sha256` / `parse_ej_decode_failure_is_fatal` / `parse_ej_no_record_header_is_fatal` / `parse_ej_empty_input_is_fatal` | 部分結果を返さない | `cargo test --lib io::ej_parser` |
| IO-08-D2 | S1 行分割 | `parse_ej_missing_final_crlf_reports_diagnostic` / `parse_ej_invalid_width_line_unresolves_record` | 孤立 LF / CR | 同上 |
| IO-08-D3 | S1 ヘッダ | `parse_ej_normal_sale_restores_items` / `parse_ej_unrecognized_header_mode_unresolves_record` | 先頭 0 の保持、日時を解釈しない | 同上 |
| IO-08-D4 | S1 先頭断片 | `parse_ej_leading_lines_before_first_header_are_reported` / `parse_ej_record_truncated_at_eof_unresolves` | EOF を閉じにしない | 同上 |
| IO-08-D5 | S1 行分類 | `parse_ej_item_named_like_label_before_separator_is_item` / `parse_ej_unknown_line_in_item_region_unresolves_record_only` / `parse_ej_unknown_line_after_separator_unresolves_record` | 位置による分類 | 同上 |
| IO-08-D6 | S1 明細復元 | `parse_ej_quantity_line_applies_to_next_item` / `parse_ej_repeated_same_name_lines_kept_separate` / `parse_ej_return_mode_keeps_positive_amounts` / `parse_ej_exact_cash_tender_without_total_line` / `parse_ej_item_count_mismatch_unresolves` / `parse_ej_quantity_price_mismatch_unresolves` / `parse_ej_total_mismatch_unresolves` / `parse_ej_count_without_total_or_cash_unresolves` / `parse_ej_header_only_record_unresolves` / `parse_ej_zero_amount_item_is_restored` / `parse_ej_item_name_with_inner_space` | 照合 3 種、名称の切り出し | 同上 |
| IO-08-D7 | S1 記録種別 | `parse_ej_non_item_records_paid_in_paid_out_exchange` / `parse_ej_settlement_and_program_records` / `parse_ej_unknown_line_in_settlement_unresolves` / `parse_ej_settlement_not_starting_with_title_unresolves` / `parse_ej_program_body_with_other_line_unresolves` | 精算票を解釈しない | 同上 |
| IO-08-D8 | S1 型・診断 | `parse_ej_every_line_is_accounted_for` / `parse_ej_diagnostic_messages_are_fixed_texts` | Unresolved に明細が無い、行番号が 1〜N を覆う、文言が固定文言と一致し生の行が無い | 同上 + review |
| IO-08-D5〜D7（実物） | AC9 | `real_ej_structure_probe`（ignore、Coordinator 手元） | 出力が件数だけ | PR body（件数だけ） |
| IO-08-D1〜D10（docs） | S3〜S7 | `design_code_compliance_phase_a`（`cargo test --test design_compliance_test`） | `29-io-ej-parser.md` の必須セクション | `bash scripts/doc-consistency-check.sh` |

## Data Safety

- repo に入れないもの: 実物の EJ file（`~/downloads/inventory-field-check/approved-readable/EJ_採取ファイルとおまけ/` の 6 本と同じ場所の log 2 本）、その内容（名称・金額・日時・番号・店の印字）、probe の出力 log、CV17 の取込み log。
- local-only: 実物の EJ、Coordinator の構造 probe script（scratchpad）、T-R1 の実行出力。
- 合成だけにするもの: `ej_parser.rs` の test 内 fixture（架空の名称・金額・日付・番号。実物の値を写さない）。Writer は実物の EJ を開かない。文法は本 packet と `29-io-ej-parser.md` の文法表だけから組み立てる。
- `29-io-ej-parser.md` と本 packet に書いてよいのは、レジが印字する固定のラベル（`合  計`、`SDｶｰﾄﾞ保存` 等）、行の幅と並び、件数だけ。Z004 の実ヘッダ表記を 23-io に記録した先例と同じ扱い。
- T-R1 は環境変数で与えた dir を読み、件数（記録のモード別・復元状態別・診断 code 別）だけを出力する。assert の失敗文言にも行の生の文字列を含めない。CI では実行しない（`#[ignore]`）。
- 診断の message に行の生の文字列を含めない（IO-08-D8、T-I1）。後続が診断を log・画面へ出しても実データが流れない。

## Implementation Results

[PR #94](https://github.com/higekiri-w/inventory-system-desktop/pull/94) で実装し squash merge 済み（`8bd2bc9a`、2026-09-25）。S1〜S7 を実装した: `src-tauri/src/io/ej_parser.rs` に純関数 `parse_ej`（IO-08）と出力型を新設し、行を記録の種類と位置で分類して、通常・返品の取引は数量×単価・点数・合計（無ければ現金）が記録内でそろうときだけ明細へ復元する。そろわない記録・未知の行・幅違反（24 バイトの中の孤立 CR / LF を含む）は記録単位の復元不能と固定文言の診断にし、行を捨てない。関数設計書 `docs/function-design/29-io-ej-parser.md` を新設して IO-08-D1〜D10 と行の文法表を正本化し、`docs/FUNCTION_DESIGN.md`・`docs/ARCHITECTURE.md`・`docs/architecture/io-task-specs.md`・`design_compliance_test` へ登録した。Final Review の是正として、数量 0 と `-` つきの単価の数量行の拒否、24 バイト内の孤立 CR / LF の検査（先頭断片・ヘッダ・本文）、`i64::MIN` の金額の読取り、fail-closed の分岐の test と弱い負例の fixture の補強を入れた。実物 6 本の構造確認（AC9、Coordinator 手元、件数だけ）は是正の前後とも復元不能 0・診断 0 で、件数は PR body に記録した。

## Review Response

- Findings Freeze: not yet frozen; post-freeze exceptions: none.

Final Review broad Codex 側（2026-09-25、GPT-6 Astra high、対象 `fc73b59f`、裁定 Coordinator）: P1 0 / P2 2 / P3 1。Codex は PR body の Validation 節にあった Opus 側の要約を読んだと申告し、独立した 1 本と扱えない → 是正後の head に Codex broad を取り直す（PR body と本節の Final Review の段落は読まない指示）。全件採用、`6124e9b3` で是正。P2-1（幅 24 を保った孤立 CR / LF が `trim()` に隠れて復元済みになる）= 幅の検査 3 か所を「24 バイトちょうどで CR / LF を含まない」判定に揃え、該当行は既存の `InvalidWidth`（29 :138 の「孤立した LF / CR はこの形で現れる」）で記録を復元不能にする。P2-2（合計行なしの現金の不一致を拒否する負例が無い）= test を 1 件追加し、現金の値を明細合計へ差し替える mutation で red を確認（実装は変更なし）。P3（`i64::MIN` の金額を読めない）= 負の値を符号つきの checked 演算で積み上げ、i64 の両端と 1 つ外側の test を追加。

Final Review broad Codex 側の取り直し（2026-09-25、GPT-6 Astra high、対象 `a96bd88e`、裁定 Coordinator）: P1 0 / P2 1 / P3 1。PR body と前回の結果を読まずに行った独立の 1 本。P2（孤立数量行の負例が点数・合計の不一致で先に落ち、拒否ガードを外しても全 test が通る）= 採用、`e03bedcf` で fixture の点数・合計を明細にそろえ、ガードを外す mutation でその test だけが red になることを確認。同じ型の弱い負例（数量行の連続、負の単価）も直し、負例の無かった点数の行の欠けに test を 1 本足した（主要な拒否ガード 23 種を 1 つずつ外して確認、残る 2 種は到達しない等価 mutant）。P3（`InvalidWidth` の発生条件が 24 バイトの中の孤立 CR / LF を含まない）= 採用、29 の D2 と診断表と同じ食い違いの 2 か所を実装に合わせ、固定文言は変えないと明記。

Final Review の closeout 時点の結論（2026-09-25、Coordinator）: helper の専用 record（`#issuecomment-5819567040`）に載る broad は 2 本で、互いに独立した Double Audit。(1) Opus 5.5 fresh の broad（対象 `83a55cb0`、`#issuecomment-5819559986`、P1 0 / P2 1 / P3 6。P2・P3-1〜P3-4・P3-6 を採用し `e0cd106d` / `fc73b59f` で是正、P3-5〈採取期間は metadata〉は不採用）と、同じく fresh の Opus による closure（対象 `8d2975de`、`#issuecomment-5819560364`、閉じる対象の P2 はすべて閉鎖を mutation で確認、新規 P1 0 / P2 0 / P3 2）。(2) Codex broad の取り直し（GPT-6 Astra high、対象 `a96bd88e`、`#pullrequestreview-5308090364`、上の段落）。record の closure は base 同期後の fresh Opus（対象 `0018a604`、`#issuecomment-5819942211`、新規 P1 0 / P2 0 / P3 0。`8d2975de` 以降の delta は clippy の `byte_char_slices` に合わせた test の 2 行〈`309c09dd`〉と origin/main `94e4d58a` の単段 merge だけで、新しい main の checker / helper も packet を受理する）。初回の Codex broad（対象 `fc73b59f`、`#pullrequestreview-5307873862`）は PR body の Opus 側の要約を読んでおり独立性が成立しないため record に入れない（その P2 2 / P3 1 は採用し `6124e9b3` で是正済み）。manual / R4 は not-required。Opus closure（`8d2975de`）の新規 P3 2 件は head を動かさず本 closeout で処理した（PR #94 Follow-up の裁定）: P3-1 = 本 packet の Spec Contract IO-08-D2 / D5 / D6 の文法（24 バイトの中の孤立 CR / LF も `InvalidWidth` とすること、数量行は数量 1 以上・単価非負だけを受理すること、`合  計` / `現金` の金額 token は共通の規則で読み、観測は全角であること）は `docs/function-design/29-io-ej-parser.md`（IO-08.2 / IO-08.5 / IO-08.6 と金額 token）を正本とし、本 packet の該当文は 29 に置き換わった。`docs/architecture/io-task-specs.md` IO-08 の処理構造 5 も 29 の IO-08.2 の文言に揃えた。P3-2 = Matrix の Test Matrix に Opus 是正の test 8 本の行、Mutation-style に 2 項（数量 0 / `-` つきの単価の受理、重複行の最初の行の採用）を足した。どちらも実装は 29 どおり fail-closed で、在庫への害はない。
