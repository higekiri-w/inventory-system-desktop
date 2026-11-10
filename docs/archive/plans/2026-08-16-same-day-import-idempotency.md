# Plan Packet: 同日複数精算の冪等取込み再設計（design-first）

## Workflow State

- Phase: ready-hosted-final
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: ba0732e
- Amendments: 4de6be86bfdc77a1be782acfd01a2f1a6c09f247
- Coordinator: Fable
- Writer: Codex
- Plan Reviewer: Sonnet
- Final Reviewer: Sonnet
- Reviewed Content HEAD: 5b2c1b8
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required
- Human Gate: owner plan approval / Ready / merge。Windows native L3 は docs-only design-first PR ではなし

## Owner Effort Budget

- 介入回数上限: 3
- 実働時間上限: 30分
- relay 往復上限: 2
- Plan Review round 天井: 3（既定 hard cap）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` に従う。本発注の停止点は plan-draft の Draft PR であり、owner 介入はまだ消費しない。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
POS raw file の同一性、Tauri command DTO / generated bindings、取込み TX、論理 rollback、在庫補正、日次・月次の会計表示、operator の追加確認を両 pipeline 横断で変更する。誤ると同日売上の過小計上または二重計上になるため、docs-only の起草段階から R3 とする。

## Goal

Goal Invariant:
同じ営業日の別 hash ファイルを「前回分の置換」ではなく「独立した追加入力」として扱い、同一 hash の二重取込みだけを hard block し、取消は対象 import 単位の明示 rollback、読み出しは有効な同日 import 全件の合算とする設計を、Z004 と日報の双方で同じ operator 意味へ揃える。

### 最小完了条件

- 本 design-first PR の完了時、source design docs と candidate D-071 が本 packet の SPEC-SDI-D1〜D8 どおりに改訂され、後続実装者が chat や issue を読まずに wire、TX、集計、UI 文言、rollback、テストを実装できる。
- 本発注の停止時、packet / Test Design Matrix / `Plans.md` が plan-first commit と Draft PR で review 可能になり、source design amendments と実装 code は未着手のまま残る。

### 失敗定義

- 同日別 hash の commit が既存 import を自動で `rolled_back` / void する余地が残る。
- 同一 hash の再取込みが通る、または byte 違いの同内容ファイルが確認なしで追加される。
- 日次表示が最新 import だけを返す、同日複数 import を別行のまま「商品別集計」と誤表示する、あるいは rollback が同日の他 import まで取り消す。
- `OverwriteRequired` / `overwrite_confirmed` / 「上書き」語彙が sales import の active contract に残る。

### 非目的

- 本 design-first PR で Rust / TypeScript 実装、generated bindings、DB schema、実 POS fixture を変更しない。
- hash が異なるファイルの意味的同一性を自動判定しない。合法な同日複数精算と byte 違い同内容の完全自動識別は、確認済み raw contract だけからはできない。
- 日報と Z004 の金額を互いに加算しない。公式日報集計と商品別売上は引き続き別表示である。
- 新しい取込み履歴 route、印刷、CV17 export 手順、layout A parser、PLU slot、bulk onboarding を同乗させない。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。

## Scope

本 design-first PR の将来 amendment scope:

- candidate D-071 と SPEC-SDI-D1〜D8 を durable source docs へ昇格する。
- BIZ-03 / BIZ-08 の同日判定、commit TOCTOU、rollback、operation log の契約を追加取込み意味論へ改訂する。
- IO query の「同日上書き確認用」を「同日追加確認 context / active import 集合取得」へ改訂し、DB は現行の複数行許容 schema を維持する。
- CMD 引数、DuplicateStatus 系 enum、preview DTO、generated binding 影響を wire contract として確定する。
- UI-07 の両タブを同じ追加確認ダイアログ構造・日本語語彙へ揃え、rollback 表示を per-import と明記する。
- BIZ-05 / IO の日次公式日報と商品別売上を同日 active import 全件の合算へ改訂し、月次の既存 additive query を regression contract として固定する。
- active source / implementation / test の stale overwrite 語彙を後続実装 scope として全列挙する。

本発注で実際に編集する scope:

- 本 packet、対応 Test Design Matrix、`Plans.md` active link のみ。
- source design amendments は plan-approved 後の次発注まで intentionally deferred。

## Non-scope

- `docs/function-design/32-biz-csv-import-service.md`、`37-biz-daily-report-import-service.md`、`55-ui-csv-import.md` その他 source docs の本文 amendment。
- `src-tauri/**`、`src/**`、`src/lib/bindings.ts` の変更。例外: `src-tauri/tests/import_internal_contract_test.rs` の期待値を docs 側 / code 側で分離 pin する test-only 変更のみ本 design-first PR で行う（SPEC-SDI-D3 遷移状態の機械追跡。gated amendment 2026-08-16）。
- schema migration、既存データ変換、物理 DELETE、同日一括 rollback。
- archive packet / matrix の歴史的記録の書換え。
- 実店舗 CSV、hash、金額、DB、backup、receipt の commit。

## Acceptance Criteria

- `docs/plans/2026-08-16-same-day-import-idempotency.md` が template 必須節、13 field Workflow State、SPEC-SDI-D1〜D8、機械 sweep 結果、amendment 契約を持つ。
- `docs/plans/test-matrices/2026-08-16-same-day-import-idempotency.md` が本 design-first PR の検証行と後続実装予約を明示的に分離する。
- `Plans.md` の `進行中・直近完了した作業` に本 packet と matrix の active link がある。
- `git diff --name-only origin/main...HEAD` の計画 commit 対象が上記 3 path のみで、`src/` / `src-tauri/` / source design docs を含まない。
- `bash scripts/doc-consistency-check.sh --target plan` を pipe なしで実行し、exit code `0` を確認する。
- 使用した `rg` command と active hit / exclusion が `Mechanical Impact Inventory` に残り、最新 1 件、単数 ID、overwrite 語彙の既知 site を sample ではなく分類し切る。
- Draft PR body が `.github/pull_request_template.md` と `docs/DEV_WORKFLOW.md` `Commit / PR Messages` に従い、Risk R3、Phase plan-draft、source amendment 未着手、Hosted CI required を明記する。

## Design Sources

- Requirements / spec: `docs/spec/requirements.md` REQ-401 / REQ-501 / REQ-502
- Architecture: `docs/ARCHITECTURE.md` POS Adapter Boundary、`docs/architecture/biz-task-specs.md` BIZ-03 / BIZ-08、`docs/architecture/cmd-task-specs.md` CMD-07 / CMD-12、`docs/architecture/ui-task-specs.md` UI-07
- Function / command / DTO: `docs/function-design/24-io-csv-import-repo.md` §14.6 / §14.7 / §14.17 / §14.18 / §14.21 / §14.22、`32-biz-csv-import-service.md` §15.2〜15.6、`34-biz-sales-service.md` §19.3〜19.4、`37-biz-daily-report-import-service.md` §37.2〜37.7、`41-cmd-pos.md` §17.5、`45-cmd-daily-report-import.md` §45.3〜45.8
- DB: `docs/db-design/pos-tables.md` §11 / §12b〜12e / B-1 / B-2、`src-tauri/src/db/schema_v1.rs`、`schema_v4.rs`（schema 実査のみ）
- Screen / UI: `docs/function-design/55-ui-csv-import.md`、`56-ui-daily-sales.md`、`57-ui-monthly-sales.md`、`docs/design-system/01-decision-rules.md`、`02-component-catalog.md`
- Decision log / ADR: `docs/decision-log.md` D-023 / D-025 / D-052 / D-070、candidate D-071
- Primary field evidence: GitHub issue #76 の 2026-08-15 実機バッチ comment `https://github.com/higekiri-w/inventory-system-public/issues/76#issuecomment-5301473247`

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status |
|---|---|---|
| Backend function / command / repository / validation / error | 24 / 32 / 34 / 37 / 41 / 45 function-design、architecture task specs | intentionally deferred to plan-approved 後の次発注 |
| Command / DTO / generated binding / wire shape | 32 / 37 / 41 / 45 + Boundary / Wire Contract | packet で amendment 契約確定、source は intentionally deferred |
| DB / transaction / audit / rollback / migration | `pos-tables.md` B-1/B-2 + 32/37 TX/rollback | packet で契約確定、source は intentionally deferred。migration は不要候補 |
| Screen / UI / Japanese wording | 55 + design-system rules/catalog。56/57 は表示意味のみ | packet で文言確定、source は intentionally deferred |
| CSV / report compatibility | D-070 / issue #76 fact、POS adapter boundary | existing fact sufficient。raw 内容は commit しない |
| Durable decision / ADR | candidate D-071 | next amendment で `docs/decision-log.md` 新 ID として起案 |

## Registration / Generation Obligations

| 新規追加物 | 登録・生成義務 |
|---|---|
| Tauri command | 新規 command なし。既存 2 command の引数名変更は `#[specta::specta]` 登録を維持し `cargo run --bin generate_bindings` で `src/lib/bindings.ts` を再生成 |
| DTO / enum | specta producer と generated TS consumer を同じ implementation commit で更新し、hand edit の binding を残さない |
| REQ coverage | 新規 REQ は作らない。既存 REQ-401 / 501 / 502 の test 追加後に traceability generator/check を実行 |
| source docs | 新設なし。親索引の追加義務なし。既存 doc の更新履歴を追記 |
| route / operator screen | 新設なし。navigation / routeTree 生成義務なし |

## Design Decisions / Amendment Contract

以下は本 plan-draft の採用候補であり、Plan Gate で Coordinator が裁定する。採用後は同じ ID と意味を source docs に昇格し、実装 PR は source docs を引用する。実装開始時に unresolved placeholder を残さない。

### SPEC-SDI-D1 — identity は content hash、business date は group key

- Z004 は `file_hash`、日報は `bundle_hash` を file/bundle 単位の冪等 identity とする。active status の同一 hash は従来どおり hard block、`rolled_back` の同一 hash は再取込み可能。
- `settlement_date` / `report_date` が同じでも hash が異なれば独立 input。commit は既存 import / sale_records / movements / daily_report lines を void / rollback せず、新しい import を追加する。
- business date は確認 context と集計 group key であって uniqueness key ではない。
- 却下: 同日最新 1 件を正とする replacement。実機 raw の `_0001` / `_0002` 独立保全と矛盾し、1 回目売上を消す。

### SPEC-SDI-D2 — correction は rollback + re-import の明示 2 操作

- 訂正対象は import ID で選び、その 1 import だけを既存 rollback 関数で論理取消する。取消後に正しい file/bundle を改めて取り込む。
- Z004 rollback は対象 import に属する sale_records / movements だけを void し、その movement 分だけ在庫補正する。同日の他 import は active のまま残す。
- 日報 rollback は対象 parent だけを `rolled_back` にし、同日の他 parent とその表示を残す。在庫を変更しない。
- rollback の再実行は冪等成功を維持する。operation log は対象 import ID を記録する。
- 却下: 「置換」button から暗黙 rollback と commit を 1 TX に束ねる方式。operator が何を消すか確認できず、同日複数時の対象選択も曖昧になる。

### SPEC-SDI-D3 — DuplicateStatus と wire context

採用候補の wire contract:

```text
DuplicateStatus = NoDuplicate | AdditionalImportConfirmationRequired
DailyReportDuplicateStatus = NoDuplicate | AlreadyImported | AdditionalImportConfirmationRequired

DuplicateCheck {
  status,
  same_date_imports: Vec<SameDateCsvImportSummary>
}
SameDateCsvImportSummary {
  id, filename, total_items, total_amount, imported_at
}

DailyReportDuplicateCheck {
  status,
  same_date_imports: Vec<SameDateDailyReportImportSummary>
}
SameDateDailyReportImportSummary {
  id, source_filenames, gross_amount, net_amount, imported_at
}
```

- `existing_import_id: Option<i64>` は単一 import/日を表現するため廃止し、active same-date import の ordered snapshot を返す。
- order は `imported_at DESC, id DESC`。`source_filenames` は保存済み `source_files_json` から operator 表示用 filename だけを BIZ で安全に取り出す。hash は operator wire に出さない。
- Z004 の同一 hash は parse/preview 前の hard error を維持する。日報の `AlreadyImported` は既存 UI hard-block 状態として維持し、commit は `IdempotencyConflict`。
- commit command の bool は双方 `additional_import_confirmed`（generated TS `additionalImportConfirmed`）へ rename する。`overwrite_confirmed` は wire から削除する。
- enum 名の代替 `SameDateAdditionRequired` は短いが「確認」が wire 上から欠落するため不採用候補。Coordinator が naming を変える場合も意味と variants は変えない。

### SPEC-SDI-D4 — confirmation snapshot と TOCTOU / retry

- cached preview は `same_date_imports` の active import ID 列を保持する。commit TX 内で最初に同一 hash を再チェックし、次に active same-date ID 列を再取得する。
- preview snapshot と TX 内 ID 列が一致しない場合は、確認 flag にかかわらず副作用なしで「同日の取込み状況が変わりました。再度プレビューしてください」を返す。新しい preview で operator に最新件数・金額・時刻を見せ直す。
- `NoDuplicate` preview に `additional_import_confirmed=true`、または confirmation-required preview に false は validation error。`AlreadyImported` は常に block。
- snapshot 一致 + confirmation true の場合だけ追加 INSERT する。既存 import を触らない。
- parse/commit failure 時の preview token retry と成功時 token 削除は現行 CMD lifecycle を維持する。snapshot mismatch は同じ token の blind retry ではなく re-preview を要求する。
- 却下: bool だけを信用して preview 後に増えた import も無条件追加する方式。D4 の operator 確認内容が stale になる。

### SPEC-SDI-D5 — 両タブの追加確認 UI と日本語

- Z004 / 日報の両方で warning Alert + AlertDialog を使い、checkbox-only と destructive overwrite dialog の差を廃止する。表示を色だけに依存しない。
- Alert title: `同じ日の取込みがあります`
- Alert description: `既存分を残したまま今回分を追加します。内容を確認してください。`
- Dialog title: `同じ日のデータを追加で取り込みますか？`
- 共通説明: `この操作は既存の取込みを置き換えません。対象日の売上に今回分を追加します。復旧用に書き出した同内容のファイルを選んでいないか確認してください。`
- 既存分は取込み回数、各 import の ID / filename(s) / 金額 / 取込み日時を一覧表示する。Z004 は件数・合計金額、日報は総売上・純売上を表示する。今回分も同じ見出し順で filename(s) と件数/金額を表示する。
- actions: `キャンセル` / `追加で取り込む`。Badge は `追加確認`。`上書き`、`置換`、`既存分を取り消す`を使わない。
- 一覧が表示領域を超える場合も全件へ到達できる scroll region とし、最初の 1 ID だけを代表表示しない。
- hash 異なり同内容を自動検知できない残存リスクは、この operator confirmation と source filename/time/amount の比較で guard する。確認は hard idempotency の代替ではない。

### SPEC-SDI-D6 — 同日 aggregate の正本契約

- Z004 商品別日次: `sale_date` が一致し `is_voided=0` の全 sale_records を対象に、`product_code + source` ごとに quantity / amount を合算する。department subtotal と grand total はその合算結果から作る。manual と auto は source badge を守るため混ぜない。
- 日報公式日次: `report_date` が一致し `status='completed'` の全 parent を対象にする。`OfficialDailyReportSummary` の singular `daily_report_import_id` は削除し、`source_import_count` を追加して UI に `N回の取込みを合算` と明示する。
- `gross_amount` / `net_amount` は active parents の値を合算する。nullable の完全性は安全側とし、対象 parent のどれかが NULL なら aggregate も NULL。payment は `payment_key`、department は `department_id`、未対応部門は normalized name fallback raw name を identity として合算する。optional quantity/count も group 内に NULL があれば NULL。
- label / sort order は active rows の最小 `sort_order`、同値時は最小 row ID を deterministic representative とする。unmatched warning は aggregate 後の未対応 group 数から作り、同じ未対応部門を import 回数分だけ重複警告しない。
- 月次商品/部門と月次公式部門 query は既に全 active row を `SUM` するため構造変更対象外。同日 active parent を 2 つ置く regression を追加して additive behavior を固定する。将来の「日報取込み済み日数」は import count でなく `COUNT(DISTINCT report_date)` とする。
- 公式日報と Z004 商品別売上は別 series のまま。両者を足して「日売上」にしない。

### SPEC-SDI-D7 — per-import rollback 表示と list semantics

- 現在の result 画面 rollback は exact import ID を渡しており backend 契約は per-import で成立する。両 confirmation に `この取込みだけを取り消します。同じ日の他の取込みは残ります。` を追加する。
- Z004 は ID / 精算日 / filename / 件数 / 金額、日報は ID / 対象日 / source filenames / 総売上 / 純売上を表示する。取込み直後 result state が持つ preview snapshot を使い、rollback のためだけに日単位検索しない。
- `list_csv_imports` / `list_daily_report_imports` は import 行単位を維持し、同日複数行を collapse しない。order は date DESC, imported_at DESC, id DESC。新しい履歴 route は non-scope。
- rollback success は daily/product/monthly/list/detail の既存 D-052 invalidation contract を維持し、refetch 後は remaining active imports の aggregate を表示する。failure は result state を保持し、同じ import ID で再試行できる。

### SPEC-SDI-D8 — durable decision と stale vocabulary closure

- candidate `D-071`: `business date は file import の uniqueness key ではない。content hash 単位の active identity、per-import rollback、active imports の additive read を app-core contract とする。upstream が delta file から cumulative snapshot へ変わる場合は adapter 契約を再設計する。`
- active sales-import contract から `OverwriteRequired`、`overwrite_confirmed` / `overwriteConfirmed`、`OverwriteConfirmDialog`、`requiresOverwrite*`、同日取込みを指す「上書き」「置換」を除去する。
- product master import、stocktake 再入力、backup restore、PLU slot、Excel の現行運用、CSS/config 値の overwrite は別意味のため変更しない。
- archive plans / matrices は当時の実装証跡であり書き換えない。active source docs が新契約の唯一の current truth になる。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| REQ-401 | 32 §15.3-15.5 / 37 §37.3-37.7 / pos-tables B-1/B-2 | SPEC-SDI-D1 / D2 | same date replacement は実機 raw contract と矛盾 | BIZ-03 / BIZ-08 parse+commit+rollback | Matrix I-B1〜I-B9 |
| REQ-401 | 32/37 DTO / 41/45 CMD | SPEC-SDI-D3 / D4 | singular ID と overwrite bool は multiple active imports を表せない | Rust DTO, CMD args, preview cache, bindings | Matrix I-W1〜I-W5 |
| REQ-401 | 55 UI-07 | SPEC-SDI-D5 / D7 | 両タブの checkbox/dialog 差と置換語彙を廃止 | sales import UI features | Matrix I-U1〜I-U8 |
| REQ-501 | 24 §14.21 / 34 §19.3 / 56 | SPEC-SDI-D6 | latest 1 parent は同日売上を過小表示 | sales_repo / BIZ-05 / DailySalesPage | Matrix I-R1〜I-R6 |
| REQ-502 | 24 §14.22 / 34 §19.4 / 57 | SPEC-SDI-D6 | monthly は既に additive、same-date regression と distinct-day 契約だけを追加 | sales_repo monthly query / docs | Matrix I-R7〜I-R8 |
| REQ-401/501/502 | decision-log | SPEC-SDI-D8 / candidate D-071 | date uniqueness の再導入を横断で防ぐ | decision-log + active design docs | Matrix M-D1〜M-D8 |

## Design Intent Audit

- Source docs can answer what/why without chat history: **現時点では No**。current docs は replacement を正本化している。next amendment で SPEC-SDI-D1〜D8 と candidate D-071 を昇格するまで implementation forbidden。
- Plan-only durable decisions found and promoted: candidate D-071 を特定済み。promotion は本発注では禁止されているため次 amendment の mandatory target。
- Assumptions and constraints: issue #76 の raw は settlement ごとの独立 file。byte 違い同内容や将来の cumulative snapshot は hash だけで識別不能。single-process AppState / DB mutex は現在の runtime 事実だが、snapshot recheck はそれに依存せず契約化する。
- Deferred design gaps: source-doc amendment、Coordinator の D3/D6 naming/aggregation 裁定、Plan Reviewer review。いずれも implementation 前 blocker。
- Test Design Matrix can cite decisions: Yes。別紙が SPEC-SDI-D1〜D8 を root にする。
- Absolute guarantee self-check: 同一 hash hard block には rolled_back 再取込み escape hatch がある。追加確認には concurrent/stale preview escape hatch があり re-preview へ fail closed。operator confirmation は semantic duplicate の絶対防止とは表現しない。

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | `_0001` / `_0002`、layout、CV17 は CASIO adapter fact。hash identity、per-import lifecycle、aggregate は app core | D-071、24/32/34/37、pos-tables |
| Fact check / design decision split | issue #76 comment は観測 fact。additive admission と UI confirmation は design decision | Contract Probe、D-071、55 |
| Lifecycle / retry | same hash、different hash、stale preview、commit failure、rollback retry、rolled_back re-import を D1/D2/D4/D7 で分離 | 32/37/41/45 + Matrix lifecycle |
| Operator workflow | 追加確認で既存/今回を比較し、訂正は rollback 後 re-import | 55 + design-system |
| Replacement path | upstream が cumulative snapshot へ変わる場合は adapter provenance/semantic mode を再設計。core に CASIO filename 規則を埋めない | D-071 Revisit |
| Data safety / evidence | issue URL と sanitized command output のみ。raw bytes/hash/amount/database は commit しない | Data Safety / PR body |
| Reporting / accounting semantics | official daily と product detail は別 series。各 series 内だけ active same-date imports を合算 | 24/34/56/57 + Matrix I-R |
| Manual verification | design-first PR は docs-only のため L3 なし。後続実装は synthetic UI tests を先行し、native L3 は dialog/wording visibility のみ candidate | implementation packet |
| 環境・再現性 | toolchain 変更なし。public-writer clone / origin/main 基準を実測 | Contract Probe |

## Design Readiness

State: **not ready for implementation; ready for Plan Review of this design-first draft**。

- Existing design docs are sufficient because: current behavior / code mapping / rollback / schema factsの調査元としては sufficient。ただし desired behavior は stale。
- Source docs updated in this PR: 本発注時点では none（明示 non-scope）。
- Design gaps intentionally deferred: SPEC-SDI-D1〜D8 の source promotion と D-071 起案を plan-approved 後の次発注へ defer。
- Durable decisions discovered: candidate D-071。next amendment で decision-log に追加しない限り本 design-first PR は完了扱いにしない。

Minimum design checks:

- Layer ownership: UI は確認/表示、CMD は bool/token bridge、BIZ は admission/snapshot/aggregate、IO は active-row query、DB は status/FK、MNT 変更なし。
- Backend function design: D1/D2/D4/D6/D7 で inputs/outputs/TX/error/rollback を定義。
- Command / DTO / data contract: D3 と Boundary / Wire Contract に破壊的 rename と generated binding を定義。
- Persistence / transaction / audit: schema migration なし。commit は insert-only、rollback は per-import、operation log ID 維持。
- Operator workflow / wording: D5/D7 で両タブ同型と exact Japanese wording を定義。
- Error/empty/retry/recovery: exact hash block、stale preview re-preview、failure retry、rollback+reimport を分離。
- Testability / traceability: REQ-401/501/502 と matrix I-* 予約を結線。

## Contract Probe

- public-writer environment: `pwd` -> `/home/kosei/Projects/inventory-system-public`; `git remote get-url origin` -> `https://github.com/kosei-w90607/inventory-system-public.git`; `git fetch origin`; `git rev-parse origin/main` -> `314bc636823f12a714fc8680e79bdda27feffd9d`; `git merge-base --is-ancestor 314bc63 origin/main` -> exit `0`; `git status --short --branch` -> `## agent/same-day-import-idempotency...origin/main` with no path entries。
- real-device premise: `gh issue view 76 --repo kosei-w90607/inventory-system-public --json title,url,comments` -> 2026-08-15 comment が SD / PC の `_0001` / `_0002` 独立保存、CV17 日付集約表示、raw の返品負数・複数数量、同日後続 file は置換でないという結論を記録。source URL は Design Sources に固定。
- schema premise: `rg -n "CREATE TABLE (IF NOT EXISTS )?csv_imports|CREATE TABLE (IF NOT EXISTS )?daily_report_imports|UNIQUE.*settlement_date|UNIQUE.*report_date|settlement_date.*UNIQUE|report_date.*UNIQUE" src-tauri/src/db docs/db-design` -> table definitions は `schema_v1.rs:132` / `schema_v4.rs:8`、date UNIQUE hit なし。migration 不要候補を確認。

## Mechanical Impact Inventory

### 使用 command

```bash
rg -n -i "OverwriteRequired|overwrite_confirmed|overwriteConfirmed|OverwriteConfirmDialog|requiresOverwrite" \
  docs/architecture docs/function-design docs/db-design docs/design-system src src-tauri \
  docs/ARCHITECTURE.md docs/SCREEN_DESIGN.md docs/Plans.md docs/decision-log.md docs/spec/requirements.md

rg -n "上書き" \
  docs/architecture docs/function-design docs/db-design docs/design-system src src-tauri \
  docs/ARCHITECTURE.md docs/SCREEN_DESIGN.md docs/Plans.md docs/decision-log.md docs/spec/requirements.md

rg -n "get_latest_completed_daily_report|最新completed|daily_report_import_id|same_date\\[0\\]|existing_import_id|imports\\[0\\]|find_imports_by_settlement_date|find_daily_report_imports_by_report_date" \
  docs/function-design src-tauri/src/biz src-tauri/src/db src-tauri/src/cmd src/features src/lib/bindings.ts

rg -n "report_count|COUNT\\(DISTINCT report_date\\)|日報取込み済み日数|日報取込み済み日" \
  docs/function-design src-tauri/src src/features

rg -l "上書き" docs/archive --glob '*.md'
rg -l -i "OverwriteRequired|overwrite_confirmed|overwriteConfirmed|OverwriteConfirmDialog|requiresOverwrite" docs/archive --glob '*.md'
```

### 単一 import/日 前提 — 是正対象

| Hit | 現行前提 | 分類 / amendment or implementation target |
|---|---|---|
| `docs/function-design/24-io-csv-import-repo.md:543-553` | latest completed 1 parent | 14.21 を all completed aggregate input へ改訂 |
| `docs/function-design/34-biz-sales-service.md:65,168-174` | singular `daily_report_import_id` / latest 1 | D6 DTO と daily aggregate へ改訂 |
| `src-tauri/src/db/sales_repo.rs:938-1089` | singular row、`ORDER BY id DESC LIMIT 1` | `get_completed_daily_reports_aggregate` 相当へ置換。予約 test は I-R |
| `src-tauri/src/db/sales_repo.rs:2026-2119` | latest/overwrite tests | same-day 2 active + per-import rollback aggregate tests へ置換 |
| `src-tauri/src/biz/sales_service.rs:77-86,225-226,451-487` | singular ID mapping | `source_import_count` と grouped lines へ改訂 |
| `src-tauri/src/biz/sales_service.rs:731-780` / `src-tauri/src/cmd/sales_cmd.rs:288-343` | singular official fixture/assert | aggregate fixture/assert へ改訂 |
| `src/lib/bindings.ts:972-981` / `src/features/daily-sales/DailySalesPage.test.tsx:61-105` | singular official wire fixture | generated field と `N回分合算` UI assert へ改訂 |
| `docs/function-design/32-biz-csv-import-service.md:64,182,261-270` / `parse.rs:172-181` / `commit.rs:32-117` | first same-date ID を replacement target | D1/D3/D4 の list snapshot + insert-only へ改訂 |
| `docs/function-design/37-biz-daily-report-import-service.md:71-76,177,193-204` / `parse.rs:119-132` / `commit.rs:17-79` | `same_date[0]` / all same-date rollback | D1/D3/D4 の list snapshot + insert-only へ改訂 |
| `src/lib/bindings.ts:451-454,649-658` | singular `existing_import_id` | D3 `same_date_imports` list へ再生成 |
| `src/features/csv-import/components/PreviewStep.tsx:33-127` / `OverwriteConfirmDialog.tsx:18-50` | first ID replacement display | D5 shared addition dialog へ改訂 |
| `src/features/daily-report-import/DailyReportImportPage.tsx:132-236` | checkbox で same-date old data 全取消 | D5 dialog へ改訂 |
| `src/features/csv-import/components/ResultStep.tsx:31-91` / `DailyReportImportPage.tsx:275-323` | rollback current ID だが「同日他 import」説明なし | backend は正、D7 文言/filename 表示を補強 |
| `src-tauri/src/db/sales_repo.rs:978-1009` / `sales_service.rs:175-226` | product rows を item-level のまま返す | all active rows は含むが商品別合算表示でない。D6 `product_code + source` group へ是正 |

### Before / After（上記 `rg -n` output から機械抽出）

| Current hit / before | After contract | Decision |
|---|---|---|
| `32-biz-csv-import-service.md:182` / `parse.rs:180-181`: same date -> `OverwriteRequired` + `imports[0].id` | same date + distinct hash -> `AdditionalImportConfirmationRequired` + ordered `same_date_imports[]` | D1/D3 |
| `32-biz-csv-import-service.md:263-270` / `commit.rs:96-105`: confirm -> old sale/movement void + old import rollback（107-117 は TOCTOU recheck で別掲） | snapshot-confirmed insert only; old import/sales/movements unchanged | D1/D4 |
| `37-biz-daily-report-import-service.md:177,202-204` / `parse.rs:131-132` / `commit.rs:69-78`: `same_date[0]` and same-date parents rollback | all same-date summaries shown; distinct bundle insert only | D1/D3/D4 |
| `55-ui-csv-import.md:83,142,172,239-244`: `OverwriteConfirmDialog` / `overwriteConfirmed` | common addition Alert/Dialog / `additionalImportConfirmed` | D3/D5 |
| `24-io-csv-import-repo.md:545-553` / `34-biz-sales-service.md:168-174` / `sales_repo.rs:1012-1089`: latest completed parent only | all completed same-date parents aggregated; singular import ID removed | D6 |
| `sales_repo.rs:978-1009`: each non-voided product sale row returned separately | same product and source rows summed across active imports | D6 |
| `sales_repo.rs:1096-1140`: all completed monthly parents already SUM | keep implementation shape; add same-date regression and distinct-day future-count contract | D6 |
| result rollback dialogs: exact ID but no remaining-import statement | exact ID/files/amount + `この取込みだけ` / other same-date imports remain | D7 |

### 単一 import/日 前提 — 実査で対象外または contract hardening のみ

| Hit | 実査結果 | 除外 / 追随理由 |
|---|---|---|
| `src-tauri/src/db/sales_repo.rs:1096-1140` | monthly official は全 completed parent を JOIN し SUM | 構造変更対象外。same-date regression と NULL/identity contract を追加 |
| `src-tauri/src/db/sales_repo.rs:1142-1205` 付近の monthly product/department | all non-voided sale_records を SUM | 既に additive。regression のみ |
| `docs/function-design/34-biz-sales-service.md:262-266` / `57-ui-monthly-sales.md:38` | 「取込み済み日」の合計 | 該当 UI 表示は未実装（34-biz の後続 UI PR メモ段階）。将来実装時の coverage count に `COUNT(DISTINCT report_date)` を明記 |
| `src-tauri/src/db/sales_repo.rs:790-850` | daily report list は import row 単位、date/time/id order | collapse しないため正。D7 として固定 |
| `src/features/home/hooks/useHomeSummary.ts:63` | global latest import の日付 | 日次 aggregate ではないため対象外 |
| `find_blocking_*_by_*_hash` の `LIMIT 1` | 1件でも active same hash があれば hard block | D2 の正しい idempotency guard。対象外 |
| product / stocktake / operation log の `LIMIT 1` hits | date-import aggregate と無関係 | pattern false positive。対象外 |

### sales-import overwrite 語彙 — active stale targets

| Layer | Hit group | 取扱い |
|---|---|---|
| architecture | `docs/architecture/biz-task-specs.md:223,228,300,305,309`; `cmd-task-specs.md:61,70`; `ui-task-specs.md:195,205` | next amendment で addition/rollback 2-operation contract へ |
| DB/function docs | `docs/db-design/pos-tables.md:50-51,127-129,231,234-240,339`; `24:220,492`; `32:64,71,99,105,182,261-270,313,512,520`; `37:71,177,193,202,204,264`; `41:201,208,229,238`; `45:81`; `55:21,23,48,67,83,109,125,142,158,161,172,189,196,239,244,343` | next amendment target |
| design-system adjacency | `docs/design-system/01-decision-rules.md:55,65,143`; `02-component-catalog.md:472,474`; `docs/function-design/51-ui-product-form.md:28` | stale sales-import exemplar/path を addition confirm または discontinue canonical へ追随 |
| Rust production | `csv_import_service/{mod.rs:99-109,145;parse.rs:180;commit.rs:35-117}`; `daily_report_import_service/{mod.rs:100-107;parse.rs:131;commit.rs:17-71}`; CMD files | 後続 implementation target |
| SPEC-SDI-D3 遷移 pin | `src-tauri/tests/import_internal_contract_test.rs:108` | docs 側 pin は本 PR で新契約へ、code 側 pin は実装 PRで再統一（I-W4） |
| Rust tests | `csv_import_service/tests/{parse_tests.rs:353,commit_tests.rs:39-366,rollback_tests.rs:29}`; `daily_report_import_service/tests.rs:218-410` | overwrite oracle を additive/per-import oracle へ置換 |
| frontend production/tests | `src/features/csv-import/**` と `src/features/daily-report-import/**` の `overwriteConfirmed` / `OverwriteRequired` / dialog hits; `src/lib/bindings.ts:144,164,451,454,649-658` |後続 implementation + bindings regeneration target |
| borrowed-reference comments | `src/features/products/components/DiscontinueConfirmDialog.tsx:6` / catalog / 51-ui | deleted/renamed dialog path を active canonical へ追随 |

### 「上書き」hit — 明示除外

| Hit group | 除外理由 |
|---|---|
| product master bulk import: `ARCHITECTURE.md:118`, `SCREEN_DESIGN.md:317-325`, `architecture/biz-task-specs.md:75-76`, `ui-task-specs.md:135,144`, `30-biz-product-service.md:319,352`, `60-ui-product-import.md`（file 内全 hit 同カテゴリ）, `src/features/products/import/**`, `product_service.rs` | 重複 product row の operator-selected UPDATE。same-day sales import と別 contract |
| stocktake: `SCREEN_DESIGN.md:39,178`, `73-ui-stocktake.md`（file 内全 hit 同カテゴリ）, `src/features/stocktake/StocktakePage.tsx:614` | counted quantity の再入力 |
| backup restore: `ui-task-specs.md:320` と backup docs/code | DB restore の destructive replacement。必須確認を維持 |
| PLU/CV17: `architecture/biz-task-specs.md:395`（amendment 後の行番号、旧 :378）, `25-io-plu-formatter.md:5`, `67-ui-plu-export.md:47` | memory slot overwrite risk。D-070 runway の別 change |
| Excel: `pos-tables.md:218`, `Plans.md:204`, `56-ui-daily-sales.md` の現行 Excel 上書き | 外部現行運用 fact。sales import semantics ではない |
| CSS/config/comment: `SCREEN_DESIGN.md:475`, `52-ui-shared-layout.md:215-216`, status chip/TanStack retry、decision-log tooling override | programming/config の override。operator contract ではない |
| stock_quantity 直接更新 / integrity-fix / traceability: `21-io-inventory-repo.md:391`, `architecture/biz-task-specs.md:535`, `65-inventory-record-traceability.md:37` | 在庫値・既存明細の上書き semantics（BIZ-02 共通在庫変動の実装記述 / integrity-fix の自動上書き禁止 / 既存明細上書きを禁じる traceability 根拠）。same-day sales import の置換とは無関係（Final Review round 1 P2-1 起源の cite 補完） |
| `docs/Plans.md:100` の completed record | 旧実装が同一TX上書きだった歴史的記録。current contract source ではない |
| `docs/archive/plans/**` | immutable historical evidence。`rg -l` の hit file listを sweep 済みだが書き換えない |

Archive hit files（`rg -l` の機械出力。historical evidence として全て除外）:

```text
docs/archive/plans/2026-04-20-ui-phase1-kickoff-plan.md
docs/archive/plans/2026-04-22-phase-1-seed-env.md
docs/archive/plans/2026-05-09-phase-2-ui-00-rally-and-resume.md
docs/archive/plans/2026-05-13-phase-2-ui-07.md
docs/archive/plans/2026-05-20-phase-2-ui-06a.md
docs/archive/plans/2026-05-21-phase-2-ui-06a-demo-fixes.md
docs/archive/plans/2026-06-12-ui01b-polish.md
docs/archive/plans/2026-06-13-design-system-pr-c.md
docs/archive/plans/2026-06-25-ui01c-implementation.md
docs/archive/plans/2026-06-27-inventory-record-traceability-design.md
docs/archive/plans/2026-07-03-d028-janless-plu-implementation.md
docs/archive/plans/2026-07-03-post-ui08-janless-plu-target-design.md
docs/archive/plans/2026-07-04-plans-dashboard-cleanup.md
docs/archive/plans/2026-07-04-req401-sales-daily-report-implementation.md
docs/archive/plans/2026-07-05-ui08-notice-placement.md
docs/archive/plans/2026-07-07-ui10-stocktake-design.md
docs/archive/plans/2026-07-07-ui10-stocktake-implementation.md
docs/archive/plans/2026-07-10-workflow-model-neutral-redesign.md
docs/archive/plans/2026-07-16-sidebar-pending-links.md
docs/archive/plans/2026-07-21-integrity-fix-semantics-design.md
docs/archive/plans/2026-07-22-mutation-consumer-query-contract.md
docs/archive/plans/2026-07-26-file-contract-unification.md
docs/archive/plans/2026-07-28-gpt56-agent-guidance-local-override.md
docs/archive/plans/2026-08-01-field-evidence-operations-sync-workflow-effectiveness-review.md
docs/archive/plans/2026-08-01-field-evidence-operations-sync.md
docs/archive/plans/2026-08-03-csv-import-record-detail.md
docs/archive/plans/2026-08-03-ui-polish-batch-b.md
docs/archive/plans/test-matrices/2026-07-03-d028-janless-plu-implementation.md
docs/archive/plans/test-matrices/2026-07-04-req401-sales-daily-report-implementation.md
docs/archive/plans/test-matrices/2026-07-04-req401-sales-slice2-official-reports.md
docs/archive/plans/test-matrices/2026-07-16-sidebar-pending-links.md
docs/archive/plans/test-matrices/2026-07-18-backup-migration-failure-contract-impl-pr2.md
docs/archive/plans/test-matrices/2026-07-22-mutation-consumer-query-contract.md
docs/archive/plans/test-matrices/2026-07-28-gpt56-agent-guidance-local-override.md
docs/archive/plans/test-matrices/2026-08-01-field-evidence-operations-sync.md
docs/archive/plans/test-matrices/2026-08-03-ui-polish-batch-b.md
```

このうち sales-import 固有 token（`OverwriteRequired|overwrite_confirmed|overwriteConfirmed|OverwriteConfirmDialog|requiresOverwrite`）の hit は次の機械出力であり、同じ archive 除外理由を適用する。

```text
docs/archive/plans/2026-05-13-phase-2-ui-07.md
docs/archive/plans/2026-06-30-sales-daily-report-design.md
docs/archive/plans/2026-07-04-req401-sales-daily-report-implementation.md
docs/archive/plans/test-matrices/2026-07-04-req401-sales-daily-report-implementation.md
docs/archive/plans/test-matrices/2026-07-28-import-internal-contract-minimization.md
```

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| SPEC-SDI-D1 | 24/32/37 + both parse/commit | I-B1〜I-B5 | real raw re-capture は issue evidenceで完了、implementation L3 non-scope候補 |
| SPEC-SDI-D2 | rollback services, logs, UI result | I-B6〜I-B9 / I-U7 | physical delete non-scope |
| SPEC-SDI-D3 | DTO/CMD/bindings/UI types | I-W1〜I-W4 | wire review automated |
| SPEC-SDI-D4 | preview cache + TX recheck | I-W5 / I-B4〜I-B5 | concurrency manual injection non-L3、automated |
| SPEC-SDI-D5 | 55 + both import tabs | I-U1〜I-U6 | native dialog visual wording candidate only |
| SPEC-SDI-D6 | 24/34/56/57 + sales repo/BIZ/UI | I-R1〜I-R8 | accounting values synthetic only |
| SPEC-SDI-D7 | rollback/list/result UI + invalidation | I-B6〜I-B9 / I-U7〜I-U8 | new history route non-scope |
| SPEC-SDI-D8 / D-071 | decision-log + sweep closure | M-D1〜M-D8 / I-G1 | archive rewrite non-scope |

## Test Plan

Test Design Matrix: [2026-08-16-same-day-import-idempotency.md](test-matrices/2026-08-16-same-day-import-idempotency.md)

- targeted tests: design anchor/sweep checks in this PR; later Rust BIZ/repo/CMD + RTL/hook tests reserved as I-*.
- negative tests: exact hash hard block、confirmation false/invalid true、stale snapshot、rollback one import、NULL aggregate、semantic-duplicate residual warning。
- compatibility checks: rolled_back hash re-import、manual/auto source separation、monthly additive behavior、existing query invalidation、generated binding drift。
- data safety checks: synthetic fixtures only、no POS raw/hash/amount/database、no physical delete/schema migration。
- main wiring/integration checks: UI bool -> generated command -> CMD -> cached BIZ preview -> IO active set -> insert; commit/rollback -> D-052 invalidation -> aggregate refetch。

## Boundary / Wire Contract

- producer: BIZ-03 / BIZ-08 preview DTOs and BIZ-05 report DTO; CMD-07 / CMD-12 command signatures; specta generator。
- consumer: `src/lib/bindings.ts` commands/types、csv-import/daily-report-import flows、DailySalesPage、tests。
- wire type: D3 の enum / `same_date_imports` summaries、`additional_import_confirmed: bool`、D6 の `source_import_count`。IDs/amounts are JSON number。
- internal type: cached preview は same-date ordered ID snapshot と parsed rowsを保持。DB models/hash は wire へ露出しない。
- precision/range: DB `i64` を existing project convention どおり JS number に生成する。本 change は range policy を変更しない。count は non-negative integer、money は返品を含み signed。
- round-trip path: parse -> specta response -> UI dialog -> generated command camelCase -> CMD snake_case -> BIZ cached preview。frontend から import ID snapshot/hash を送り返さない。
- invalid input: unknown enum は generated TS/Rust contract mismatch、true/false status mismatch は ValidationFailed、same hash は IdempotencyConflict/ImportError、snapshot mismatch は re-preview error。
- compatibility: wire breaking rename のため old frontend/new backend 混在は非互換。Tauri app は同一 bundle 配布なので producer/consumer/bindings を同一 implementation commit で切替える。DB schema と保存済み rows は互換。

## Review Focus

- D1/D4 が「同日 distinct hash を許可」だけで終わらず、stale preview 下でも既存 import を一切 void しないか。
- D5 が legitimate second settlement と recovery duplicate の両方を正直に説明し、operator confirmation を絶対保証と誤記していないか。
- D6 が official daily と product detail を横加算せず、各 series 内の additive semantics と nullable/group identity を定義しているか。
- mechanical inventory に active stale hit の漏れ、unrelated overwrite の誤変更、archive rewrite がないか。
- current order の non-scope（source amendment / implementation）を越えていないか。

## Spec Contract

Contract ID: SPEC-SDI-2026-08-16

- Active same hash is hard-blocked; rolled-back same hash may be imported again.
- Same date + distinct hash is confirmation-gated additive input and never implicit replacement.
- Confirmation is bound to the active same-date import snapshot; drift fails closed to re-preview.
- Correction is explicit per-import rollback followed by a separate re-import.
- Daily/product/monthly reads include all active same-date imports within their own reporting series.
- UI exposes existing and incoming summaries and says the operation adds without replacing.

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-SDI-D1 | source amendment then implementation | M-D1 / I-B1〜B5 | additive admission / exact hash block | issue #76 + BIZ tests |
| SPEC-SDI-D2 | rollback amendments/implementation | M-D2 / I-B6〜B9 | only selected import changes | DB before/after asserts |
| SPEC-SDI-D3 | DTO/CMD amendments/implementation | M-D3 / I-W1〜W4 | no singular ID/overwrite wire | generated binding diff |
| SPEC-SDI-D4 | lifecycle implementation | M-D4 / I-W5 | stale preview no side effect | TX integration test |
| SPEC-SDI-D5 | UI source amendment/implementation | M-D5 / I-U1〜U6 | adjacent tabs / exact wording | RTL + optional native L3 |
| SPEC-SDI-D6 | read contract amendment/implementation | M-D6 / I-R1〜R8 | accounting series and aggregate | repo/BIZ/UI tests |
| SPEC-SDI-D7 | rollback UI/list contract | M-D7 / I-U7〜U8 | selected ID and remaining totals | RTL + invalidation tests |
| SPEC-SDI-D8 / D-071 | durable promotion/sweep | M-D8 / I-G1 | stale vocabulary closure | rg output / source docs |

## Data Safety

- Commit してはいけないもの: 実 POS CSV、raw bytes、実 hash、店舗名/端末情報、実金額、DB、backup、receipt、issue comment の非公開添付、secret。
- local-only paths: `.local/**`、実店舗 artifact の保存先、Tauri app data、CI evidence logs。
- synthetic-only paths: 後続 implementation の Rust fixture / frontend mock は synthetic dates/products/amounts のみ。
- generated outputs: `src/lib/bindings.ts` は後続 implementation で generator から再生成し、hand edit しない。
- source-derived data: 本 packet は issue URL と anonymized shape（連番別 file / negative return / multiple quantity）だけを記録し、raw 値を複製しない。

## Implementation Results

source design amendment を実施した。

- durable decision: `docs/decision-log.md`（D-071 accepted）
- architecture / DB: `docs/architecture/biz-task-specs.md`, `docs/architecture/cmd-task-specs.md`, `docs/architecture/ui-task-specs.md`, `docs/db-design/pos-tables.md`
- function design: `docs/function-design/10-common-rules.md`, `24-io-csv-import-repo.md`, `32-biz-csv-import-service.md`, `34-biz-sales-service.md`, `37-biz-daily-report-import-service.md`, `41-cmd-pos.md`, `45-cmd-daily-report-import.md`, `51-ui-product-form.md`, `55-ui-csv-import.md`, `56-ui-daily-sales.md`, `57-ui-monthly-sales.md`
- design-system adjacency: `docs/design-system/01-decision-rules.md`, `docs/design-system/02-component-catalog.md`
- quality adjacency: `docs/quality/review-checklist.md`
- mechanical sweep: active sales-import contractの旧固有tokenはhitなし。「上書き」の残存hitはMechanical Impact Inventoryの明示除外（商品master、棚卸し、backup restore、PLU/CV17、Excel、CSS/config、historical record）と後続implementation対象だけである。archiveは変更していない。
- gated amendment: SPEC-SDI-D3 の移行途中を静的契約testの docs側新契約 / code側現契約の分離pinで追跡し、後続implementation I-W4のcode renameと同時に単一pinへ再統一する。

## Review Response

- Findings Freeze: not yet frozen; post-freeze exceptions: none

### Plan Gate（2026-08-16）

- Plan Review round 1（独立 Sonnet）: P1/P2 = 0、P3 × 2（Before/After 表の引用範囲特定化 / 未実装 UI の表現是正）→ 両 accept、`fdea103` で是正済み。引用実在性（32-biz / 37-biz / 両 pipeline 実装 / bindings / issue #76 comment 本文）、`bundle_hash` 実在（migration 不要判断の裏付け）、独立 rg sweep の収載漏れなし、scope 3 path、doc-check exit 0 を reviewer が独立実測。
- Coordinator 裁定: SPEC-SDI-D3 の wire 命名・形状 = packet 案どおり accept / SPEC-SDI-D6 の集計契約（singular ID 廃止・`source_import_count`・NULL 安全側伝播・grouping identity・`product_code + source` 合算）= accept / candidate D-071 文言 = accept（Revisit 条件 = upstream が精算単位 delta から cumulative snapshot へ変わる証跡の観測時）。
- owner plan approval: 2026-08-16（介入 1/3）。
- 遷移: plan-draft -> plan-gate -> plan-approved -> implementing を本 state-only commit で materialize。各遷移の評価証跡 = packet/Matrix committed（`ba0732e`）、Plan Review P1/P2 = 0、Plan Commit `ba0732e` が全実装（amendment）commit に先行。

### Final Review（2026-08-16）

- Final Review round 1（独立 Sonnet、Plan Reviewer とは別 context）: Ledger 8/8 適合、Matrix M-D1〜M-D8 全 PASS、amendment 忠実性（D5 文言の一字一句一致含む）・doc 横断 Contract Audit・旧語彙 active hit 0 の独立再実測・gated amendment scope（test 分離 pin の非弱体化）を確認。P1 = 0 / P2 × 2（除外表の cite 完全性 / Plans.md エントリ陳腐化）→ Coordinator 両 accept、reviewer 修正案どおり `5b2c1b8` で是正（Coordinator は新規 cite 8 箇所を実読裏取りの上で適用）。reviewer 再検証で漏れ 0・先取り記述なし・契約変更混入なしを確認し round 1 CLOSED（P1/P2 = 0）。
- 遷移: implementing -> local-verified -> independent-review -> human-confirm を本 state-only commit で materialize。評価証跡 = content candidate `5b2c1b8` の L1 full RESULT=PASS（evidence 位置は PR body 記載）、Final Reviewer engaged + P1/P2 = 0、Reviewed Content HEAD = `5b2c1b8`。

### Ready（2026-08-16）

- owner Ready authorization: 2026-08-16（介入 2/3）。Human Gate の残りは Ready 化操作と merge のみ。
- 遷移: human-confirm -> ready-hosted-final を本 state-only commit で materialize（Draft のまま）。本 commit 後の exact HEAD で L1 full を実行し、evidence は PR body に記録する（D-038、tracked file へは書かない）。
