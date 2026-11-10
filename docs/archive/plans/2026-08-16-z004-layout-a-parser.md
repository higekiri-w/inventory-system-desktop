# Plan Packet — Z004 parser layout A 対応（D-070 runway ②）

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

If a state-only commit materializes multiple phases, list the complete adjacent forward sequence and the pre-existing evidence for every intermediate transition in an append-only review/evidence record. Recording compression never permits a gate skip.

- Phase: ready-hosted-final
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: 8aa96a5
- Amendments: 8236176, 4efda0e, a5f7af5, a9fd1db
- Coordinator: Fable (main thread)
- Writer: Codex
- Plan Reviewer: Sonnet subagent（独立、Writer と別 context）
- Final Reviewer: Sonnet subagent（独立、Writer と別 context）
- Reviewed Content HEAD: 19d3285
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required
- Human Gate: Ready トリガー（owner 承認済み 2026-08-18、介入 4 回目 / 予算 3 回・超過 1 明示記録）/ merge

Transition narrative（append-only）:

- 本 packet 作成 commit で kickoff → spec-check → design → plan-draft → plan-gate を materialize する。evidence: task scope と Risk は本 packet に記録（kickoff → spec-check）/ in-scope source docs は Design Sources に列挙し設計更新要と判定（spec-check → design）/ 設計判断は本 packet Spec Contract に確定済みで未解決の設計問題なし、source doc への反映は本 PR 内で Writer が実施（design → plan-draft、PR #77 先例の「updated in this PR」形）/ packet + Test Design Matrix を同一 commit で commit（plan-draft → plan-gate）。
- state-only 遷移 commit で plan-gate → plan-approved → implementing を materialize する（recording compression の正規例）。evidence: 独立 Plan Reviewer rally 3 round で P1/P2 = 0 収束（Review Response の round 1〜3 記録、最終 round 3 は対象 ce6869d）/ owner plan 承認 2026-08-17（介入 1 回目 / 予算 3 回）/ plan-first commit 8aa96a5 は全実装 commit に先行（PK5 ancestry）。
- state-only 遷移 commit で implementing → local-verified を materialize する。evidence: content candidate に対する L1 `local-ci.sh full` は CLEAN / PASS、candidate SHA と evidence 位置は Draft PR #81 body に記録。独立 Final Review と owner L3 は未実施のため Phase は local-verified に留める。
- **state-backtrack human-confirm → implementing（2026-08-17）**: owner L3 round 1 FAIL（true positive）— 実 Z004（layout A）が「精算日を抽出できません」で安全停止。根本原因 = 実ヘッダ第 2 フィールドは半角カナ「ｽｷｬﾆﾝｸﾞｺｰﾄﾞ」であり、SPEC-Z4A-D1/D2・実装・synthetic fixture の全角「コード」照合が実形状と不一致（owner handoff + Coordinator の実ファイル機械抽出で二重確認。あわせてメタ実ラベル = マシンNo./ファイル/モード/精算回数/日付/時刻、7 行目空行区切りの新事実を確定）。取込み確定なし・在庫売上変更なし、recovery baseline = inventory_backup_20260817_014028.db。是正は gated Amendment 4 + 実装修正で行い、implementing から再前進する（Reviewed Content HEAD は pending へ戻す。a5f7af5 の監査記録は Review Response に保持）。
- **state-backtrack local-verified → implementing（2026-08-17、2 回目）**: Final Review の a9fd1db 再検証が実体全 PASS の一方、doc 同期の取りこぼし（23-io 内の旧ヘッダ表記 2 箇所・入力例の行番号 off-by-one・plu-export 冒頭要約 1 箇所）+ PR body の孤立 evidence 行を新規 P2 として検出。是正は docs / PR body のみ（実装・test 非接触）だが content candidate が変わるため正規 backtrack で implementing へ戻し、是正 commit に implementing → local-verified を同乗して再前進する。reviewer 推奨の機械的 grep sweep（「メモリNo」パターン全数）を是正手順に組み込み、forward-looking 残存が指摘 3 箇所のみであることを実測確認済み。
- **backtrack 後の再前進（2026-08-17）**: gated Amendment 4 + 半角カナ是正の content commit に implementing → local-verified を同乗させる（PR #58 先例の正規手段、state-only slot 不使用）。evidence: 是正後 tree での L1 `local-ci.sh full` CLEAN（commit 直前に実行、candidate SHA と evidence 位置は PR body に記録）。以後の local-verified → independent-review → human-confirm → ready-hosted-final は、Final Review 再検証 CLOSED + owner L3 PASS + Ready 承認の evidence が揃った時点で単一 state-only commit の隣接 forward compression として materialize する（STATECAP 総数 3/3・post-impl 2/2 に適合する唯一の経路）。
- state-only 遷移 commit で local-verified → independent-review → human-confirm を materialize する（隣接 forward の recording compression）。evidence: L1 full CLEAN（candidate a5f7af5、evidence は PR body）/ 独立 Final Reviewer engaged・Contract Audit 実施（local-verified → independent-review）/ Final Review CLOSED P1/P2 = 0（Review Response の round 1 + 統合再検証記録）+ Reviewed Content HEAD = a5f7af5 設定（independent-review → human-confirm）。
- **STATECAP 是正の consolidation（2026-08-17、履歴統合の逸脱記録）**: 上記の implementing → local-verified を state-only 遷移 commit（旧 8450de5）として立てた記録方式は、STATECAP の非 sanctioned slot を消費し（総数 cap 3 / post-impl cap 2 に対し、以後の human-confirm + ready-hosted-final で両 cap 超過が必発の潜伏欠陥）、checker 実読で Coordinator が検出した。PR #64 先例（recording compression 統合是正）に従い、Draft 段階で旧 content commit 1c87b27 + 旧 state-only 8450de5 を単一 content commit へ統合し、implementing → local-verified はその content commit 同乗（PR #58 先例の正規手段）へ改めた。統合 commit には Final Review round 1 の P2 是正（drift sweep）と gated Amendment 3 も同乗する。L1 full は統合後 tree で再実行し、evidence は PR body を正とする。Plan Commit 8aa96a5 / Amendments 8236176, 4efda0e は不変（PK5 維持）。
- **state-only 遷移 commit で local-verified → independent-review → human-confirm → ready-hosted-final を materialize する（2026-08-18、隣接 forward の recording compression、上記「backtrack 後の再前進」予告の実行）**。evidence: L1 full CLEAN / PASS（candidate 19d3285 = CSV-05/09/10 L3 完了記録 + Plans.md 進行行同期の docs-only content commit、evidence 位置は PR body）+ 独立 Final Reviewer engaged・Contract Audit 実施済み（local-verified → independent-review）/ Final Review CLOSED P1/P2 = 0（candidate 65ce4b5、Review Response 記録）+ docs-only delta の独立再検証 P1/P2 = 0（candidate 19d3285、Review Response 記録）+ Reviewed Content HEAD = 19d3285 設定（independent-review → human-confirm）/ owner L3 round 2 PASS（2026-08-18、CSV-05/09/10 消化、evidence = PR body / PR comment）+ owner Ready 承認（2026-08-18、介入 3 回目 / 予算 3 回）（human-confirm → ready-hosted-final）。Amendments へ a5f7af5（consolidation、Amendment 3 同乗）/ a9fd1db（Amendment 4 同乗）を追記。遷移後に本 state-only HEAD で L1 full を再実行し、PR body を全面 refresh してから owner が Ready をトリガーする。
- **計上是正（2026-08-18、owner 裁定）**: 直前 entry および Human Gate field の「介入 3 回目 / 予算 3 回」は Coordinator の計上誤り。D-055（decision point 単位計上）+ PR #74 先例（L3 round 個別計上・超過明示記録）に従い、正 = plan 承認(1) / L3 round 1(2) / L3 round 2(3) / Ready 承認(4) = **介入 4 回目 / 予算 3 回・超過 1**。超過分は L3 round 1 FAIL（true positive）連鎖起因の round 2。owner 裁定で超過明示記録を採用（承認自体は有効、誤りは計上数のみ）。本是正 commit は計上数値のみの tracking-record 修正（実装・設計 doc 非接触）であり、Reviewed Content HEAD = 19d3285 を維持する。

## Owner Effort Budget

- 介入回数上限: 3
- 実働時間上限: 30分
- relay 往復上限: 2
- Plan Review round 天井: 3（既定）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

§5.5を使わないchangeは両方`none`のままにする。

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
POS CSV（Z004）の parser 契約変更。取込みパイプライン（BIZ-03 → sale_records / pos_stock_sync 在庫増減）の入口であり、誤 parse は在庫・売上データの誤登録に直結する。Risk Tiers の「POS CSV」に該当。

## Goal

Goal Invariant:

### 最小完了条件

- 店舗の標準手順（SD → CV17 取込み → PC 側 `EcrDatas`）で得られる実 Z004 ファイル（layout A: メタ行群 + ヘッダ 1 行 + 全スロットダンプ、精算日はメタ内日付行）を、既存の Z004 取込み画面からエラー停止なしで parse でき、既存 BIZ-03 パイプライン（sale_records 作成・pos_stock_sync 在庫増減・rollback）へ従来どおり流れること。

### 失敗定義

- 実 Z004（layout A）が引き続き「精算日を抽出できません」で停止する、または layout A 対応の副作用で従来 shape の parse 結果・既存 test が変化する。
- どちらの shape でもない不正ファイルが安全停止せずに誤 parse される（fail-closed の毀損）。

### 非目的

- BIZ-03 / UI-07 / bindings の契約変更。parser の出力契約（ParseResult）は不変のまま完結させる。
- layout B（連結型）対応、PLU slot 永続割当、bulk onboarding、受入台本第2版（いずれも D-070 runway の後続便）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。

## Scope

- `src-tauri/src/io/z004_parser.rs`: 二形状対応（従来 shape + layout A）の layout 検出・精算日抽出の実装（Spec Contract SPEC-Z4A-D1〜D6）。
- `docs/function-design/23-io-z004-parser.md`: 契約 amendment — 二形状受理・layout A 構造・精算日抽出規則・E パディング語義の正本化（SPEC-Z4A-D1〜D6 の doc 側正本）、`total_data_lines` 定義の一般化、2026-08-01 evidence boundary 注記の実態同期。
- `docs/function-design/55-ui-csv-import.md`: 「店舗採取 layout A は IO-02 未対応」記述 2 箇所（冒頭 evidence sync 注記・取込み種別表）の実態同期。
- `docs/plu-export-and-real-csv-verification.md`: CSV-05 / CSV-06 / CSV-07 / CSV-09 / CSV-10 の状態行を実態へ同期（CSV-06/07 は issue #76 実機バッチ evidence〈2026-08-15 PASS〉で即時同期可、CSV-09/10 のみ L3 evidence 確定後に完了記録。Plan Gate round 1 F2）。
- 新規 synthetic layout A fixture（CP932 / CRLF / 全フィールドクォート / 実形状 exact = メタ 6 行〈マシンNo./ファイル/モード/精算回数/日付/時刻、12byte 固定幅 padding、日付 5 行目〉+ 7 行目空行 + 半角カナヘッダ / 13桁JAN+E・8桁+EEEEEE・全ゼロ・返品マイナス・複数数量行を含む匿名化 shape。gated Amendment 4 で実形状へ是正）と新規 unit tests。
- 新規 test が REQ token を含む場合は `cargo run --bin generate_traceability` で `90-traceability.md` 再生成を同一 PR で実施。

## Non-scope

- layout B（連結型）対応。Z004 の layout B 実サンプルは未採取であり、D-070 の runway ②は「layout A 対応」に scope を pin している。店舗標準手順は `EcrDatas`（layout A）に固定済み（2026-08-01 owner 判断、UI-07-D12）。layout B は現行どおり安全停止を維持し、CV17 明示書出し経路（復旧・調査用途）の実需と実サンプルが揃った時点で別 change として判断する。
- BIZ-03 / CMD-07 / UI-07 の契約・実装変更（ParseResult 出力契約不変のため不要）。同日冪等（D-071 / SPEC-SDI-D1〜D8）は merge 済みの前提基盤であり本 change で触らない。
- 8桁独自コード行の業務的取込み（商品マスタは JAN 前提。SPEC-Z4A-D5 で行単位エラーとしての可視性を維持するに留める）。
- PLU slot 永続割当 / bulk onboarding / 受入台本第2版（runway ③④⑤）。
- 既存の従来 shape unit tests の改変（凍結、SPEC-Z4A-D7）。

## Acceptance Criteria

- AC1: synthetic layout A fixture の parse が成功し、settlement_date がメタ日付行から YYYY-MM-DD で得られる（新規 unit test、`cargo test -p` 対象 crate の該当 test green）。adversarial case（非日付メタ値に日付様文字列を混在させても layout A 検出・「日付」ラベル行優先が保たれる）は `parse_z004_layout_a_datelike_meta_first_line` green で拘束する。
- AC2: 従来 shape の既存 unit tests が全件無改変のまま green（`git diff` で既存 test 関数の変更ゼロ + L1 full CLEAN）。
- AC3: どちらの shape でもない入力（日付行なし・ヘッダ行なし・メタ走査上限超過）が致命的エラーで安全停止する（新規 negative tests `parse_z004_layout_a_no_date_fails` / `parse_z004_layout_a_no_header_fails` / `parse_z004_unrecognized_shape_fails` green）。
- AC4: `YYYY/M/D` 形式のメタ日付が `YYYY-MM-DD` へゼロ埋め正規化される（新規 unit test green）。
- AC5: 8桁+EEEEEE 行が invalid_jan の行単位エラーとして報告され、他行の parse は継続する（新規 unit test `parse_z004_layout_a_8digit_code_invalid_jan` green）。
- AC6: 全ゼロコード行（13桁・14桁）が空スロット skip のまま（既存 test 凍結 + layout A fixture 内の該当行が parsed_rows / parse_errors のどちらにも入らないことを新規 test `parse_z004_layout_a_slot_dump_counts` で assert、green）。
- AC7: 23-io / 55-ui / plu-export doc の同期が完了し `bash scripts/doc-consistency-check.sh` CLEAN（evidence: L1 full log）。
- AC8: owner L3 — 実 Z004（店舗採取、local-only）をバックアップ済みテスト DB へ取り込み、取込み成功・在庫増減・日次売上反映・返品マイナスを確認（CSV-09/CSV-10 相当。evidence = PR body / PR comment）。

## Design Sources

- Requirements / spec: `docs/function-design/23-io-z004-parser.md`（IO-02 現行契約）/ REQ-401 系（Z004 取込み）
- Architecture: `docs/ARCHITECTURE.md`（IO 層 = 純関数・DB 非依存の境界、変更なし）
- Function / command / DTO: `docs/function-design/55-ui-csv-import.md`（UI-07 / CMD-07 の呼び出し文脈、契約変更なし）/ `docs/function-design/32-biz-csv-import-service.md`（BIZ-03、契約変更なし・SPEC-SDI-D1〜D8 前提基盤）
- DB: 変更なし（parser は DB 非依存）
- Screen / UI: 変更なし（UI-07 の operator 動線・文言は不変）
- Decision log / ADR: D-070（Z004 を v1.0 gate に含める・runway ②の scope 根拠）/ D-071（同日冪等、前提基盤）/ 2026-08-01 owner 判断（標準入力元 = `EcrDatas` 固定、UI-07-D12）
- 形状 evidence: `docs/plu-export-and-real-csv-verification.md`「2026-07-06 実機確認・実ファイル形状の確定事項」/ `docs/function-design/29-io-daily-report-parser.md` §29.4（layout A/B 知見の移植元）/ issue #76 実機バッチ（2026-08-15、Z-LAYOUT 差分なし・Z-CSV-06/07 PASS）

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status |
|---|---|---|
| Backend function / command / repository / validation / error | `docs/function-design/23-io-z004-parser.md` | updated in this PR（SPEC-Z4A-D1〜D6 正本化） |
| Command / DTO / generated binding / wire shape | なし（ParseResult 不変、bindings 再生成不要） | existing sufficient |
| DB / transaction / audit / rollback / migration | なし（DB 非接触） | existing sufficient |
| Screen / UI / route state / Japanese wording | `docs/function-design/55-ui-csv-import.md`（注記同期のみ、契約不変） | updated in this PR |
| CSV / TSV / report / import / export format | `docs/plu-export-and-real-csv-verification.md`（検証状態同期） | updated in this PR |
| Durable decision / ADR | D-070 が scope 根拠として既存。新規 durable decision は SPEC-Z4A-D* として 23-io へ正本化（decision-log 新設は不要） | updated in this PR |

## Registration / Generation Obligations

| 新規追加物 | 登録・生成義務 |
|---|---|
| REQ coverage 追加（新規 test が REQ token を含む場合） | `cargo run --bin generate_traceability` で `90-traceability.md` 再生成（PR #72 の教訓: targeted checks では未検出、L1 full / CI で顕在化） |

上記以外（Tauri command / doc 新設 / route / 画面）は該当なし。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| REQ-401（Z004 取込み） | 23-io §13.3 | SPEC-Z4A-D1 | 二形状受理 + fail-closed。従来 shape 判定は 1 行目日付 + 2 行目中間強度検査（5 フィールド + 第 2『コード』）の二重条件（日付様メタ値による誤ルーティングの構造排除。D2 全アンカー照合は凍結 BIZ fixture の省略ラベル「額」と衝突するため gated Amendment 1 で不採用、第 2『コード』は全凍結 fixture pass 実測により Amendment 2 で保持）。従来 shape 廃止案は既存 fixture/test 凍結と後方互換を壊すため不採用 | `parse_z004` | T-A1 / T-A7 / T-A8 / T-N1〜N3 |
| REQ-401 | 23-io §13.3（新設節） | SPEC-Z4A-D2 | layout A 検出はヘッダ検査（5 フィールド + 位置アンカー付きラベル照合）方式。field 数のみの検出は移植元 `is_header_fields` より弱く不採用、位置非依存ラベル照合も round 2 N3 で位置アンカーへ強化。メタ固定 6 行依存は CV17 版差に脆いため不採用。ヘッダ未検出は `NoSettlementDate` variant + 原因別新文言（凍結 negative test は variant のみ assert で互換） | `parse_z004` layout 検出 | T-A1 / T-A5 / T-A7 / T-N2 |
| REQ-401 | 23-io §13.3（新設節） | SPEC-Z4A-D3 | 精算日は「日付」ラベル行優先 + 最初の日付パターン fallback。`YYYY/M/D` 受理 + ゼロ埋め正規化（29-io §29.4 と同基準）。メタ 5 行目固定参照は不採用（同上） | 日付抽出・正規化 | T-A2 / T-A3 / T-A7 / T-N1 |
| REQ-401 | 23-io §13.2 | SPEC-Z4A-D4 | ParseResult 出力契約不変。下流（BIZ-03 / SPEC-SDI 基盤）を無改変で成立させる | 型定義（変更なしの確認） | T-A1 + 既存 test 凍結 |
| REQ-401 | 23-io §13.5 | SPEC-Z4A-D5 | E = 14 桁固定幅パディングの語義正本化。8桁+EEEEEE は invalid_jan 行エラー維持（silent skip は売上行の不可視喪失のため不採用） | doc 語義修正（挙動不変） | T-A4 |
| REQ-401 | 23-io §13.6 | SPEC-Z4A-D6 | 全スロットダンプ（5,000 行規模）受理。既存上限 10,000 行 / 20MB（BIZ-03 検査）内で新規ガード不要 | 境界仕様（確認） | T-A6 |
| — | DEV_WORKFLOW（既存 test 凍結原則） | SPEC-Z4A-D7 | 従来 shape 既存 test は無改変凍結。layout A は新規 test のみで拘束 | test diff | AC2 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: 可。形状事実は plu-export doc「2026-07-06 実機確認」、layout 知見は 29-io §29.4、scope 根拠は D-070 に既存。本 change で 23-io に契約として正本化する。
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: SPEC-Z4A-D1〜D6 を 23-io へ正本化（本 PR 内）。
- Assumptions and constraints: 実ファイルは CP932 / CRLF / 全フィールドクォート / メタ行 2 列（ラベル + 値: マシンNo./ファイル/モード/精算回数/日付/時刻、12byte 固定幅 padding）+ 空行区切り + ヘッダ行 5 フィールド（第 2 = 半角カナ『ｽｷｬﾆﾝｸﾞｺｰﾄﾞ』）・データ行 5 フィールド（2026-08-17 実ファイル機械抽出で確定。旧記載「管理No./帳票/番号」「メモリNo./コード」は evidence doc の転記誤りで Amendment 4 で是正）。非日付メタ値が日付パターンを含まないことには**依存しない**設計とする（従来 shape 判定の 2 行目中間強度検査〈5 フィールド + 第 2『コード』〉の重畳 + 「日付」ラベル行優先で誤ルーティング・誤抽出を構造的に排除し、T-A7 / T-A8 で拘束。Plan Gate round 1 F3 / gated Amendment 1・2）。IO 層は純関数・DB 非依存を維持。
- Deferred design gaps, risk, and follow-up target: layout B（連結型）は実サンプル未採取のため安全停止のまま defer（Non-scope 参照）。`EcrDatas` の保持期間・命名等の運用設計は runway 外の既存 backlog（日報取込み運用設計）。
- Test Design Matrix can cite design decision IDs or source doc sections: 可（SPEC-Z4A-D1〜D7）。
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: 完了。例外 = 二形状のどちらでもない入力は全て致命的エラー安全停止（escape hatch なし）。従来 shape の互換は既存 test 凍結で機械保証。

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | layout A/B は CASIO adapter facts。core 契約への昇格は 23-io の SPEC-Z4A-D* として実施（29-io の先例と同型） | 23-io amendment |
| Fact check / design decision split | 形状事実（メタ 2 列・日付 5 行目・E パディング）は 2026-07-06 解析 + 2026-08-15 Z-LAYOUT で検証済み。設計判断（検出方式・走査上限・エラー可視性）は本 packet で確定 | Contract Probe 節 |
| Lifecycle / retry | 取込み lifecycle（preview / commit / rollback / 同日冪等）は BIZ-03 既存契約のまま不変。parser は純関数で retry 概念なし | Matrix State Lifecycle 節 |
| Operator workflow | operator の操作は不変（UI-07 の同一画面・同一手順）。変わるのは「実ファイルでエラー停止しなくなる」結果のみ | AC8（L3） |
| Replacement path | 従来 shape を置換せず並存（後方互換）。既存 fixture・取込み済みデータへの影響なし | SPEC-Z4A-D1 |
| Data safety / evidence | 実 CSV・実店舗値は repo に入れない。fixture は匿名化 synthetic のみ | Data Safety 節 |
| Reporting / accounting semantics | Z004 は商品別 track。Z001/Z002/Z005 公式日次との加算禁止は既存契約のまま（本 change 非接触） | — |
| Manual verification | L3 = 実ファイル取込み + 在庫・日次売上・返品確認（CSV-09/10 の消化） | AC8 |
| 環境・再現性 | 新設の環境依存なし（純 Rust、既存 crate のみ。新規依存追加なし） | — |

## Design Readiness

- Existing design docs are sufficient because: 形状事実と layout 知見は既存 doc（plu-export / 29-io §29.4）に正本化済み。不足は IO-02 契約への適用のみで、それは本 PR の 23-io amendment（SPEC-Z4A-D1〜D6）で満たす。
- Source docs updated in this PR: 23-io / 55-ui（注記 2 箇所）/ plu-export（検証状態）。
- Design gaps intentionally deferred: layout B 対応（Non-scope 参照）。
- Durable decisions discovered in this plan and promoted to source docs: SPEC-Z4A-D1〜D6。

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): IO-02 単独の変更。層間契約（ParseResult）は不変。
- Backend function design: `parse_z004` の処理ステップ改訂を 23-io に正本化。
- Command / DTO / data contract: 不変（bindings 再生成不要）。
- Persistence / transaction / audit impact: なし（DB 非接触、BIZ-03 の operation_logs 分岐は既存契約のまま）。
- Operator workflow / Japanese UI wording: 操作は不変。致命的エラー文言は原因別 2 文言へ改訂（SPEC-Z4A-D2/D3 に exact 文言を確定済み、23-io の文言表と同一 PR で同期。既存の「〜。ファイル形式を確認してください」規約を踏襲）。
- Error, empty, retry, and recovery behavior: 二形状いずれでもない入力は致命的エラー安全停止。行単位エラー・空スロット skip の既存意味論は不変。
- Testability and traceability IDs: SPEC-Z4A-D1〜D7 + REQ-401。REQ token を含む test 追加時は traceability 再生成。

## Contract Probe

- 実 Z004 layout A の形状（メタ行 2 列・日付 5 行目・ヘッダ/データ 5 フィールド・CP932・CRLF・全クォート・E パディング）: 2026-07-06 実機採取の匿名化解析（plu-export doc「2026-07-06 実機確認・実ファイル形状の確定事項」）+ 2026-08-15 店舗再採取の Z-LAYOUT 差分なし（issue #76 evidence） -> 検証済み。新規 probe 不要、最終確認は L3（AC8）。
- 現行 parser が layout A で安全停止する事実: `z004_parser.rs` の Step 6（1 行目日付抽出）実読 + plu-export doc の実サンプル再確認記録 -> 検証済み（`NoSettlementDate` 停止）。
- 未検証の外部前提: なし（新規 crate 依存なし、OS/hardware 依存なし）。

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| SPEC-Z4A-D1 二形状受理（二重条件判定、2 行目中間強度検査）+ fail-closed | `parse_z004` layout 検出分岐 | T-A1（layout A 成功）/ T-A7（日付様メタ値の誤ルーティング排除）/ T-A8（『コード』条件の拘束）/ T-N3（どちらでもない入力の安全停止）/ 既存従来 shape tests（凍結） | L3: 実ファイル取込み成功 |
| SPEC-Z4A-D2 ヘッダ検査（5 フィールド + 位置アンカー付きラベル照合）のメタ読み飛ばし・未検出 NoSettlementDate + 新文言 | layout A 走査 | T-A1 / T-A5（メタ行数変動耐性）/ T-A7（decoy 5 フィールド非ヘッダ行の排除）/ T-N2（ヘッダ不在、`NoSettlementDate` variant + 文言） | — |
| SPEC-Z4A-D3 「日付」ラベル行優先 + fallback 走査 + 正規化 | 日付抽出 | T-A2（YYYY-MM-DD）/ T-A3（YYYY/M/D ゼロ埋め）/ T-A7（ラベル優先）/ T-N1（日付なし） | — |
| SPEC-Z4A-D4 ParseResult 出力契約不変 | 型・下流無改変 | T-A1 の全 field assert + 既存 test 凍結（AC2） | — |
| SPEC-Z4A-D5 E パディング語義・8桁行の可視エラー維持 | `normalize_jan`（挙動不変）+ 23-io 語義修正 | T-A4（8桁+EEEEEE → invalid_jan、他行継続） | — |
| SPEC-Z4A-D6 全スロットダンプ受理・全ゼロ skip | 既存境界の確認 | T-A6（大型 fixture: 空スロット多数 + 実データ行混在の件数 assert） | — |
| SPEC-Z4A-D7 既存 test 凍結 | test diff | AC2（diff 検分 + L1 full） | — |
| 隣接契約: BIZ-03 上限 10,000 行 / 20MB（23-io §13.6、無変更） | 変更なし | 既存 BIZ-03 tests（非接触） | non-scope（明示除外: parser 側に上限検査を新設しない） |
| 隣接契約: INV-6 file_hash（生バイト SHA-256、無変更） | 変更なし | 既存 tests（凍結） | non-scope |
| 隣接契約: SPEC-SDI-D1〜D8 同日冪等（無変更の前提基盤） | 変更なし | 既存 tests（非接触） | non-scope（layout A 実ファイルの同日複数精算は L3 で観察のみ、契約変更なし） |

## Test Plan

Test Design Matrix: [test-matrices/2026-08-16-z004-layout-a-parser.md](test-matrices/2026-08-16-z004-layout-a-parser.md)

- targeted tests: `z004_parser` unit tests（新規 T-A1〜T-A6 / T-N1〜T-N3、既存凍結）
- negative tests: 日付なし / ヘッダなし / メタ走査上限超過 / 二形状いずれでもない構造
- compatibility checks: 従来 shape fixture の parse 結果不変（既存 test 凍結）
- data safety checks: fixture が匿名化 synthetic であること（実 JAN・実店舗値・実商品名の不使用）
- main wiring/integration checks: BIZ-03 経由の既存 integration tests が無改変 green（parser 改修が主経路に届いていることは既存経路 test で担保）
- Human Gate L3 を含むため、Writer 完了条件に `cargo check --release` を含める（owner native build 前、CI gate ではない）

## Boundary / Wire Contract

- producer: CASIO CV17（SD 取込み後の `EcrDatas` 常在ファイル = layout A）/ 従来 shape（既存 fixture・過去採取分）
- consumer: `parse_z004` → BIZ-03（無改変）
- wire type: CP932 生バイト列（CSV、全フィールドクォート、CRLF/NEL 正規化は既存契約）
- internal type: `ParseResult`（不変）
- precision/range: quantity / amount = i32（不変）。日付は YYYY-MM-DD / YYYY/M/D 受理 → YYYY-MM-DD 正規化
- round-trip path: なし（読み取り専用の一方向 parse）
- invalid input: 二形状いずれでもない → 致命的エラー安全停止。行単位不正 → parse_errors 蓄積（不変）
- compatibility: 従来 shape の parse 結果・エラー文言は不変（既存 test 凍結が機械保証）

## Review Focus

- layout 検出の fail-closed 性: 二重条件判定（1 行目日付 + 2 行目中間強度検査〈5 フィールド + 第 2『コード』〉）と layout A 走査が、不正ファイルを誤受理しないか（メタ走査上限・layout A ヘッダ検査の位置アンカー照合・「日付」ラベル行優先の堅牢性）。
- 致命的エラー文言の原因別分岐（ヘッダ未検出 / 精算日未検出）が契約どおり実装され、旧「1行目から」文言が残存しないか。
- 従来 shape の完全非退行: 既存 test 凍結の diff 検分と、検出分岐追加による従来経路の意味論変化がないこと。
- 日付正規化のゼロ埋め・境界（1 桁月日、不正日付文字列）。
- fixture の匿名化 shape 準拠（Data Safety）。
- 23-io amendment と実装の一致（Contract Audit で行単位再検証）。

## Spec Contract

Contract ID: SPEC-Z4A-D1〜D7

- SPEC-Z4A-D1（二形状受理・fail-closed）: `parse_z004` は (a) 従来 shape（1 行目に `YYYY-MM-DD` 日付パターン、2 行目ヘッダ、3 行目以降データ）と (b) layout A（メタ行群 + ヘッダ 1 行 + データ行群）の両方を受理する。判定は「改行正規化後、1 行目に日付パターンがあり**かつ** 2 行目が 5 フィールドに分割でき第 2 フィールドに『コード』または半角カナ『ｺｰﾄﾞ』を含む場合に従来 shape、それ以外は layout A 走査」。2 行目条件は **中間強度（5 フィールド + 第 2 フィールドのコード label〈全角・半角両形〉含有、第 5 フィールドのラベル照合なし）** とする — 凍結 BIZ test の従来 shape fixture には省略ラベルヘッダ（`"No.","コード","名","個","額"` = 第 5 フィールド「額」）が実在し、D2 の全アンカー照合を従来 shape 判定に課すと凍結 test `test_parse_and_validate_req401_invalid_settlement_date` が red になる（gated Amendment 1、Writer fail-closed true positive）。一方、第 2 フィールド『コード』は全凍結 fixture で一貫して pass することを focused review が実測済みのため保持する（gated Amendment 2 = focused review P2-1 修正案の採用。互換コストゼロで防御強度を最大化）。二重条件により、日付様文字列を含むメタ値による従来 shape への誤ルーティング（誤 settlement_date + 行ずれの大量 parse_errors）は構造的に排除される（実 layout A のメタ行は 2 列で field 数条件により、5 フィールドの adversarial 行は『コード』条件により排除。Plan Gate round 1 F3 の防御は保持）。layout A 走査側のヘッダ検査（SPEC-Z4A-D2 の位置アンカー照合）は不変。どちらの shape としても成立しない入力は致命的エラー（`Z004ParseError`）で安全停止し、部分 parse 結果を返さない。
- SPEC-Z4A-D2（layout A 構造検出・ヘッダ検査）: ヘッダ検査は「5 フィールドに分割でき、**第 2 フィールドに『コード』または半角カナ『ｺｰﾄﾞ』、第 5 フィールドに『金額』を含む**行」とする（29-io 移植元 `is_header_fields` と同型の field 数 + 位置アンカー付きラベル照合。従来 shape ヘッダ `No,コード,名称,個数,金額` も**実ファイルヘッダ〈レコード/ｽｷｬﾆﾝｸﾞｺｰﾄﾞ/キャラクター/個数/金額 — 第 2 フィールドは半角カナ、CP932 12byte 固定幅 padding。2026-08-17 実ファイル機械抽出、gated Amendment 4〉**も同一基準で pass する。初稿〜Amendment 3 の「実ファイルヘッダ = メモリNo./コード/名称/個数/金額」という事実主張は誤りで、L3 round 1 FAIL の根本原因（Amendment 4 で是正）。メタ行は 2 列のため field 数でも排除され、位置アンカー照合は 5 フィールドの非ヘッダ行に対する防御の重畳。Plan Gate round 2 N3）。layout A 走査では先頭 20 行以内で最初にヘッダ検査を満たす行をヘッダとし、それより前の行群をメタとして扱う。20 行以内に未検出の場合は `NoSettlementDate` variant・文言「ヘッダ行を検出できません。ファイル形式を確認してください」で安全停止する（既存凍結 negative test `test_parse_z004_req401_no_settlement_date` は variant のみ assert〈message assert なしを rg で実測済み〉のため variant 互換で凍結維持。`NoDataLines` は既存どおり 2 行未満の pre-check 専用）。メタ行数の固定値（6 行）には依存しない（CV17 の版差・帳票差への堅牢性。事実としてのメタ 6 行は fixture に反映する）。データ行はヘッダ行の次行以降とし、行単位処理（空行 skip / 空スロット skip / 行単位エラー）は既存契約のまま適用する。
- SPEC-Z4A-D3（精算日抽出・正規化）: layout A では、メタ行群のうち第 1 フィールドに「日付」を含む行の値からの日付パターン抽出を優先し、該当行がない場合のみメタ行群を先頭から走査した最初の日付パターンへ fallback する（実ファイルのメタはラベル 2 列構成〈マシンNo./ファイル/モード/精算回数/日付/時刻〉であり、ラベル優先は非日付メタ値の偶然一致に対する防御。ラベル実名は Amendment 4 の機械抽出で是正）。受理形式は `YYYY-MM-DD` / `YYYY-M-D` / `YYYY/M/D`（dash / slash 両区切りとも月日 1〜2 桁）で、出力は常にゼロ埋め `YYYY-MM-DD` に正規化する（29-io §29.4 の日付受理と同基準。dash 形の 1〜2 桁明示は gated Amendment 3 = Final Review F-DATE-1 の実装整合）。メタ行群に日付が見つからない場合は `NoSettlementDate` variant・文言「精算日を抽出できません。ファイル形式を確認してください」で停止する。従来 shape の 1 行目抽出は現行契約のまま不変。**利用者向け文言の改訂（Plan Gate round 2 N1）**: 旧文言「1行目から精算日（YYYY-MM-DD）を抽出できません」は「1行目」という位置限定が二形状対応後は事実誤認となるため退役し、上記 2 文言（原因別: ヘッダ未検出 / 精算日未検出）へ置き換える。凍結 tests は variant のみ assert のため無改変で green（実測済み）。23-io の処理ステップ・エラーハンドリング表の文言も同一 PR で同期する。
- SPEC-Z4A-D4（出力契約不変）: `ParseResult` / `ParsedRow` / `ParseError` / `Z004ParseError` の型・意味論は変更しない。`line_no` は物理行番号（1 始まり）のまま。`total_data_lines` の定義は「3 行目以降」から「ヘッダ行より後の行のうちフィールド分割を試みた行」へ一般化する（従来 shape ではヘッダ = 2 行目のため実質同値）。BIZ-03 以降・bindings は無改変。
- SPEC-Z4A-D5（E パディング語義）: コード欄の `E` は 14 桁固定幅の右パディングであり識別子ではない（2026-07-06 確定事項）。23-io §13.5 の「レジ固有サフィックス」記述をパディング語義へ是正する。正規化の挙動は不変: 13 桁 JAN + `E` は 13 桁化、8 桁独自コード + `EEEEEE` は invalid_jan の行単位エラーとして可視のまま維持する（silent skip は売上行の不可視喪失となるため不採用。当該行は商品マスタ非対象で取込み不能が正であり、エラー可視性が operator への正直な報告である）。
- SPEC-Z4A-D6（全スロットダンプ受理）: layout A は売上有無を問わない全スロットダンプ（5,000 行規模）である。既存上限（10,000 行 / 20MB、BIZ-03 検査）の範囲内であり、parser 側に新規サイズガードは設けない。全ゼロコード行の空スロット skip は既存契約のまま。
- SPEC-Z4A-D7（既存 test 凍結）: 従来 shape の既存 unit tests は削除・改変・skip いずれも禁止。layout A の拘束は新規 test / fixture のみで行う。

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-Z4A-D1 | layout 検出分岐（二重条件）の実装 | T-A1 / T-A7 / T-N3 + 既存凍結 tests | fail-closed 性 | L1 full + L3 |
| SPEC-Z4A-D2 | ヘッダ検査・メタ読み飛ばし | T-A1 / T-A5 / T-N2 | 検出条件の堅牢性・error variant 互換 | L1 full |
| SPEC-Z4A-D3 | 日付ラベル優先走査・正規化 | T-A2 / T-A3 / T-A7 / T-N1 | ゼロ埋め・境界 | L1 full |
| SPEC-Z4A-D4 | 出力契約の非変更確認 | T-A1 全 field assert + AC2 | 型 diff ゼロ | L1 full |
| SPEC-Z4A-D5 | 23-io 語義修正（挙動不変） | T-A4 | doc と実装の一致 | Contract Audit |
| SPEC-Z4A-D6 | 境界確認 | T-A6 | 件数 assert | L1 full |
| SPEC-Z4A-D7 | test diff 凍結 | AC2 | diff 検分 | Final Review |

## Data Safety

- 実 Z004 CSV・実店舗値（実 JAN、実商品名、実売上数値、管理 No. 等のメタ実値）を commit しない。
- local-only paths: 店舗採取の実ファイル（owner PC の `EcrDatas` / ローカル解析 dir）。repo へは参照名のみ。
- synthetic-only paths: 新規 fixture は匿名化 shape（synthetic JAN 13 桁・架空名称・架空数値）のみ。29-io fixture と同基準。

## Implementation Results

- 従来 shape の中間強度検査と layout A の位置アンカー付きヘッダ走査を分離し、ラベル優先の精算日抽出、原因別文言、走査上限、後方互換の出力契約を実装した。synthetic layout A tests と mutation 実注入で SPEC-Z4A-D1〜D6 を拘束し、既存従来 shape tests / ParseResult 型 / BIZ・CMD・UI は非改変を維持した。
- Draft PR: https://github.com/higekiri-w/inventory-system-public/pull/81

## Review Response

Fill after review.
If R3 review-only sub-agent is skipped, record an explicit line beginning with `Review-only skipped because:` and the reason.
- Review-only skipped because: packet は独立 Sonnet Final Reviewer を割当済みで、Draft 後の independent-review phase で Contract Audit を行う。同一 vendor の Codex subagent は D-062 の独立性を満たさないため、Draft checkpoint での代替起動はしない。
- Findings Freeze: frozen after Broad Audit（Final Review round 1 + 統合 commit 再検証、2026-08-17）; post-freeze exceptions: none.

### Plan Gate round 1（独立 Sonnet Plan Reviewer、対象 = plan-first commit 8aa96a5）

P1 = 0 / P2 = 3 / P3 = 1。全件 accept、同一 packet 内で是正（plan-gate に留まったままの in-place 修正、Coordinator が引用を実読裏取りのうえ裁定）。

- F1 (P2) accept: ヘッダ検出が field 数のみで移植元 `is_header_fields`（field 数 + ラベル内容照合）より弱い → SPEC-Z4A-D2 をラベル照合込みのヘッダ検査へ改訂。
- F2 (P2) accept: plu-export doc の CSV-06/07 が issue #76 evidence（2026-08-15 PASS）と drift したまま同期対象から漏れ → Scope を CSV-05/06/07/09/10 へ拡張。
- F3 (P2) accept（構造是正の強い形を採用）: 従来 shape 判定が「1 行目日付のみ」で日付様メタ値による誤ルーティングの新失敗モードを持つ → SPEC-Z4A-D1 を二重条件（1 行目日付 + 2 行目ヘッダ検査）へ、SPEC-Z4A-D3 を「日付」ラベル行優先へ改訂し、T-A7 で拘束。
- F4 (P3) accept: ヘッダ走査上限 20 の off-by-one が mutation 拘束されていない → Matrix の boundary case と mutation 設問を追加。
- Coordinator 自己検出 (P2 相当): 初稿 SPEC-Z4A-D2 の「ヘッダ未検出 → `NoDataLines` 系」は凍結 negative test `test_parse_z004_req401_no_settlement_date`（`NoSettlementDate` variant を assert）と衝突 → `NoSettlementDate` variant 互換へ是正。

### Plan Gate round 2（fresh 独立 Sonnet Plan Reviewer、対象 = 571dc4b）

P1 = 0 / P2 = 2 / P3 = 1。round 1 是正 5 件中 4 件適正・F4 不十分の判定。F3 は凍結 fixture 全数の机上トレースで健全確認。全新規 findings accept、in-place 是正。

- N1 (P2) accept: `NoSettlementDate` の旧文言「1行目から…」が二形状対応後は事実誤認 → 原因別 2 文言（ヘッダ未検出 / 精算日未検出）を SPEC-Z4A-D2/D3 に exact 確定。凍結 tests は variant のみ assert を rg 実測で確認済み。
- N2 (P2) accept: ラベル照合の mutation 設問が既存 fixture では拘束されない（5 フィールド非ヘッダ行が fixture 群に不存在）→ T-A7 fixture に decoy（5 フィールド・ラベル位置不一致の非ヘッダ行）を追加し、照合除去で確実に red になる構成へ。
- N3 (P3) accept（強い形を採用）: 「is_header_fields と同型」の精度過大 → 位置アンカー付き照合（第 2 フィールド『コード』+ 第 5 フィールド『金額』）へ設計自体を強化し、記述と実体を一致させた。

### Plan Gate round 3（fresh 独立 Sonnet Plan Reviewer、対象 = ce6869d、収束確認）

P1 = 0 / P2 = 0 / P3 = 1（記録のみ）。round 2 是正 3 件（N1/N2/N3）は source 実測（凍結 test の assert 実読 / 両ヘッダのフィールド位置逐次照合 / 移植元 `is_header_fields` 実読）で全件適正判定。sweep 漏れなし。**Plan Gate 収束（round 3/3、これ以上の round 不要）**。

- P3-1（記録のみ、disposition = 発注書へ反映）: T-A7 が 3 つの mutation 標的（誤ルーティング排除 / 日付ラベル優先 / decoy ヘッダ誤認排除)を 1 test に束ねており fail 時の切り分けがやや弱い → Writer 発注書に「T-A7 内の assert に契約 ID コメントを付し、切り分け可能な assert 順で書く」を含める。新規 test 分割はコスト対効果で不採用。

### gated Amendment 1（2026-08-17、Writer fail-closed true positive）

Codex Writer が実装時に凍結 BIZ test `test_parse_and_validate_req401_invalid_settlement_date`（`src-tauri/src/biz/csv_import_service/tests/parse_tests.rs`）との衝突を検出し fail-closed 停止（発注どおりの正動作）。Coordinator が実読で再現確認 — 当該 fixture の 2 行目は `"No.","コード","名","個","額"` で、SPEC-Z4A-D2 の第 5 フィールド『金額』照合を通らず、初稿 D1 の「2 行目 = D2 ヘッダ検査」条件では従来 shape 判定に失敗して `NoSettlementDate` 停止 → BIZ の「不正な日付」検証に到達しない。

- 全数 sweep（Coordinator 実測）: `精算日` fixture の出現は 5 file。共有 builder `test_support.rs`（ヘッダ「スキャニングコード/金額」）と `csv_import_cmd.rs` fixture は位置アンカー照合を pass、衝突は当該 1 fixture のみ。BIZ の no-date negative fixture（`"no date here"...` 5 フィールド × 2 行）は layout A 走査 → ヘッダ未検出 → `NoSettlementDate` で従来どおり green（机上トレース）。
- 裁定: SPEC-Z4A-D1 の従来 shape 判定 2 行目条件を「D2 ヘッダ検査」→「5 フィールド検査のみ（ラベル照合なし）」へ変更。F3 の誤ルーティング防御は field 数条件で保持（実 layout A のメタ行は 2 列）。layout A 走査側の D2 位置アンカー照合・decoy mutation 拘束（round 2 N2）は不変。
- 棄却代替案: (a) 凍結 test の fixture 修正 = 既存 test 凍結原則に違反。(b) D2 ラベル照合の「額」への緩和 = layout A 側の検出強度を道連れに弱めるため不採用。
- 付記: BIZ-03 `parse.rs` は `Z004ParseError` を variant 単位で自前文言に写像するため、SPEC-Z4A-D2/D3 の原因別 2 文言は BIZ 経由の operator 表示には現れない（parser 層 Display / 診断用の正確性改善に留まる）。round 2 N1 の「operator に事実誤認メッセージが出る恐れ」は BIZ 経路では実害なしと実読確認。BIZ 文言の原因別化は BIZ-03 契約変更となるため Non-scope 維持。

### gated Amendment 2（2026-08-17、focused review P2-1 修正案の採用）

Amendment 1 の focused review（独立 Sonnet、対象 8236176 + 094485e）は 5 検証項目全適正・承認可（P1 = 0 / P2 = 1 / P3 = 1）。P2-1 の修正案を採用し、従来 shape 判定の 2 行目条件を「5 フィールドのみ」から「5 フィールド + 第 2 フィールド『コード』含有」の中間強度へ再強化した。

- 採用根拠: 凍結 fixture の不一致は第 5 フィールド「額」のみで、第 2 フィールド『コード』は全凍結 fixture で一貫 pass することを focused reviewer が実測済み（互換コストゼロ）。F3 防御の劣化幅を最小化し、P3-1（メタ 1 行 + 日付様値の病的 layout A 亜種の誤ルーティング）も『コード』条件で大幅に狭まる。
- 新規拘束: T-A8（1 行目日付様 + 2 行目 5 フィールド非ヘッダ〈『コード』なし〉→ layout A 走査で正しく parse。『コード』条件を外した mutant で red）を Matrix に追加。
- P3-1 residual: メタ 1 行のみ + 日付様値 + 5 フィールド + 第 2『コード』を偶然満たす入力は理論上残るが、実ファイル前提（メタ 2 列・6 行、2026-07-06 + 2026-08-15 検証）から乖離が大きく、CV17 版差の顕在化時に再判断（記録のみ、非 blocking）。
- 本 amendment は focused reviewer 自身が実測裏付きで提案した修正案の採用であり、独立再レビューは要さない（相互修正案方式）。実装への反映は Writer 再開発注に含める。

### Coordinator mutation 独立再実測（2026-08-17）

Test Design Matrix の adequacy questions から記録非参照で 13 形を独立導出し、隔離 worktree（baseline green）で逐次実注入 — **全 kill・survivor 0**。境界系（走査上限の off-by-one 両側 / decoy によるラベル照合・位置非依存化の弱体化 2 形 / 「日付」ラベル優先除去 / silent skip 化 / ヘッダ行 data 混入）と、gated Amendment 1 起源の凍結 BIZ test による layout 判定恒偽化の kill を含む。全 mutant 復元・worktree clean 確認済み。注入形と red test の対応表は session 記録（scratchpad mutation-plan）に保持、要旨のみ本記録とする。

### Final Review round 1（独立 Sonnet、対象 = 統合前 content 1c87b27 相当、2026-08-17）

Ledger 10/10 適合・P1 = 0 / P2 = 1 / P3 = 2。Coordinator 裁定（全件実測裏取り済み）:

- F-DRIFT-1 (P2) accept・same-PR 是正: 「layout A は IO-02 未対応」旧文言が live source docs 5 file 6 箇所（ARCHITECTURE ×2 / PROJECT_HANDOFF / pos-tables / ui-task-specs / 32-biz）に残存 — Writer の sweep は 23-io / 55-ui / plu-export の 3 文書限定で、repo 全体 grep が対象を捕捉。6 箇所を実態同期（drift-fix sweep、DEV_WORKFLOW Review Rules の same-PR fix 該当）。
- F-DATE-1 (P3) accept・gated Amendment 3: 実装の dash 形 `YYYY-M-D`（月日 1〜2 桁）受理が Spec 文言より広い → 23-io と本 packet D3 の文言を実装へ整合（受理拡張の明示、fail-closed 性の変更なし）。
- F-EXPECT-1 (P3) 記録のみ: `conventional_date.expect(...)` は bool 判定で論理的に到達不能。if-let 化はコード変更の再検証コストに見合わないため非採用、将来の同 file 改修時に同乗可。

### Final Review 統合 commit 再検証（同一 Final Reviewer、対象 = a5f7af5、2026-08-17）

再検証 5 項目（drift 是正文言 / 残存 grep 0 / Amendment 3 整合 / 統合の src 無欠損性 `git diff 8450de5 a5f7af5 -- src-tauri/` = 0 行 / packet 記録整合）**全 PASS**。gate 独立再現（z004 unit tests / clippy / doc check）も green。新規 findings の裁定:

- 新規 P2（PR body 鮮度）: reviewer の読取りと並行して Coordinator が既に body 更新を適用済み（非同期競合の見かけ上の指摘）。現 body の candidate SHA / evidence path / drift grep 行が a5f7af5 系であることを機械確認し解消 — reviewer 自身が「更新済みなら CLOSED 相当・実体側の追加是正不要」と明示。
- 新規 P3（Findings Freeze 未更新）: 本 state-only commit で frozen へ更新（是正済み）。

**Final Review CLOSED（P1/P2 = 0）**。

### Final Review a9fd1db 再検証（同一 Final Reviewer、2026-08-17）

Amendment 4 の実体は全 PASS（契約-実装一致 / fixture 実形状整合・file_hash を reviewer が別言語独立再計算で一致確認 / 凍結維持 diff 検分 / mutation 2 設問の机上トレース妥当）。新規 findings の裁定（全件 accept）:

- 新規 P2-a（doc 同期取りこぼし 3 箇所: 23-io evidence note の旧ヘッダ列挙 / 23-io 入力例の旧ヘッダ行重複 + 行番号 off-by-one / plu-export 冒頭要約）: same-PR 是正。「列意味」と「実ヘッダ表記」を分離する形で全 3 箇所を更新し、「メモリNo」パターンの機械的 grep sweep で forward-looking 残存 0 を実測確認。
- 新規 P2-b（PR body の孤立 evidence 行 = 旧 1c87b27 参照）: PR body から削除・更新。
- 新規 P3（PR body の test 件数表記の off-by-one 疑い）: PR body 側で実測値へ更新（tracked docs には件数を書かない D-038 を維持）。
- process 採用: reviewer 推奨の「是正時は関連 doc 全文への機械的 grep sweep を手順に組み込む」を本是正から適用（doc 同期取りこぼしの同型 2 巡目に対する構造対策。WER 候補として記録）。

### Final Review 最終確認（同一 Final Reviewer、対象 = 65ce4b5、2026-08-17）

再検証 P2×2 / P3×1 の是正を全項目 PASS 判定（3 箇所の是正文言と実測事実の一致・入力例の行番号と実 fixture 定数 padding の一致を diff 精読で確認 / PR body の SHA・evidence・件数整合 / 残存 P1/P2 = 0）。手続き（backtrack 2 回目 4ed5983 → 是正 content commit 同乗）も遷移表・STATECAP 制約と整合と判定。**Final Review CLOSED（P1/P2 = 0、candidate = 65ce4b5）**。以後は owner L3 round 2 + Ready 承認の evidence が揃った時点で、local-verified → independent-review → human-confirm → ready-hosted-final を単一 state-only commit で materialize する（Reviewed Content HEAD = 65ce4b5、Amendments へ a5f7af5 / a9fd1db を追記予定）。

### owner L3 round 1 FAIL + gated Amendment 4（2026-08-17、実ヘッダ半角カナ是正）

owner L3 round 1 = **FAIL（true positive、L3 gate の実効性実証）**。実 Z004（Issue 採取ファイルの Z004、preview 段階・取込み確定なし・在庫売上変更なし、recovery baseline = inventory_backup_20260817_014028.db）が「精算日を抽出できません」で安全停止。

- 根本原因（owner handoff + Coordinator の実ファイル機械抽出の二重確認）: 実ヘッダ第 2 フィールドは半角カナ「ｽｷｬﾆﾝｸﾞｺｰﾄﾞ」で、契約・実装・synthetic fixture の全角「コード」照合が実形状と不一致。あわせて evidence doc（plu-export）のメタラベル記載「管理No./帳票/番号」が実物「マシンNo./モード/精算回数」と相違、7 行目空行区切りが未記載という evidence 転記誤りも確定（synthetic fixture がこの誤 evidence から構築されたことが真因の構図）。
- Amendment 4 の裁定: (a) D1/D2 のコード label 照合を全角『コード』+ 半角『ｺｰﾄﾞ』の両形受理へ拡張（共通 helper `contains_code_label`）。(b) primary fixture を実形状 exact（実メタラベル 12byte padding + 空行 + 半角ヘッダ）へ是正し、全角形は独立 test（fullwidth_header_variant）で拘束。(c) 23-io / plu-export の実形状記述を機械抽出結果で是正。(d) 実装は Coordinator/Writer 兼務で実施（相当規模 = anchor 1 helper + fixture、PR #75 の兼務先例。relay 予算 2/2 消化済みの budget 判断込み。独立 Final Review の再検証で writer 自己承認を回避）。
- 検証: req401 filter 全 pass（新規 fullwidth variant 込み）+ **実ファイル直接投入の一時 test で parse 成功を確認**（精算日 = 採取日 2026-07-06 一致・total_data_lines 5,000・行単位エラー 10 件 = evidence 記録の 8 桁独自コード既存別商材 10 件と完全一致）後、一時 test は撤去済み。
- 教訓（WER 候補）: L3 round 1 FAIL の真因は「実 evidence の転記誤りの上に synthetic fixture と契約を構築したこと」。実ファイル shape を契約化する change では、局所 shape 事実の機械抽出を Plan Gate 前の Contract Probe に含めるべきだった（本 change の Probe は既存 doc 引用で済ませ、実物との突合を L3 まで遅延させた）。

### owner L3 round 2 PASS + delta 再検証 + Ready 承認（2026-08-18）

- owner L3 round 2 = **PASS**（Windows native、店舗採取 Z004 layout A で CSV-05/06/07/09/10 全項目 PASS・blocker なし。evidence = PR #81 comment〈匿名化観測記録: 在庫増減・返品戻り・複数数量・日次/月次売上・inventory_movements〉）。L3 で判明した後続 UI 改善 6 項目は issue #83 へ起票（本 PR non-scope、既存 backlog との重複も同 issue に明記）。
- L3 完了記録の docs-only content commit（plu-export CSV-05/09/10 状態行 + Plans.md 進行行同期、In-scope の Plan Gate round 1 F2 残置き分）で content candidate は 19d3285 へ更新。独立 Sonnet delta 再検証で **P1/P2 = 0（対象 diff）** — L3 evidence との事実突合 / packet 義務適合 / D-038 遵守 / 旧表記 sweep 残存 0 / issue #83 実在を確認。「PR body 未同期」の P1 指摘は、遷移表が定める Ready 前の PR body 全面 refresh（本遷移直後に実施）で解消する段取り済み事項と Coordinator 裁定。P3（PROJECT_HANDOFF 最終更新スタンプの陳腐化）は scope 外・owner へ backlog 提案として報告。
- owner Ready 承認 = 2026-08-18。前節「Final Review 最終確認」の Reviewed Content HEAD = 65ce4b5 予定は、candidate 更新に伴い 19d3285 へ差し替えて設定する。
- **計上是正（2026-08-18、owner 裁定）**: Ready 承認依頼時の「介入 3 回目 / 予算 3 回」は計上誤りで、正 = **介入 4 回目 / 予算 3 回・超過 1**（plan 承認 1 / L3 round 1 = 2 / L3 round 2 = 3 / Ready 承認 = 4。D-055 + PR #74 先例）。超過分 = L3 round 2（round 1 FAIL true positive 連鎖起因）。owner 裁定により超過明示記録を採用。Transition narrative の同日付是正 entry 参照。
