# Plan Packet: EJ parser に店が普段使う文法を足す（R3）

2026-10-04 起票。起点は `76de30d8`（origin/main）。2026-10-04 の材料の見落としチェックの後に owner が選んだ並走（lane A・D・E、回答台帳 TD-104、repo 外）の lane D。出典は `docs/backlog.md` の「EJ parser に、店がレジで普段使う値引き・訂正・取消の文法を足す」の項目とその入力 (1)〜(9)、全期間の EJ の構造所見（`docs/function-design/29-io-ej-parser.md` の「全期間の EJ の構造所見」）。前回の packet は `docs/archive/plans/2026-09-23-ej-parser-core.md`（PR #94）。設計の決定は D-105 と `29-io-ej-parser.md` の IO-08-D3a・D5a〜D5d・D6a〜D6d・D7a で、本 packet の plan-first の commit に含める。実装は plan-approved の後に Writer（別 context）が行う。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

実装後の状態はPR native state / 専用record / CIが所有し、trackedに書かない。

- Phase: archive
- Risk: R3
- Plan Commit: 116c118a391e95dd3a6d06d7076c75209ca99fb0
- Amendments: e430d45c655bba0bd439a01066e947de947f7fd4, 559f6ae1320d6753890643089b4bf0653cbf98e0
- Coordinator: Opus 5.5（Claude Code main session、effort high）
- Writer: Opus 5.5 subagent（`subagent_type: writer`、本 lane の worktree、branch `agent/ej-grammar`）
- Plan Reviewer: Opus 5.5（fresh `subagent_type: reviewer`）+ Codex（GPT-6.1 Sol、`.local/codex-orders/MODEL-SELECTION.md` の表。round 1〜3 とも同じ行）
- Final Reviewer: Fable 5.1（fresh subagent）+ Codex（GPT-6.1 Sol）。互いに独立で、後の reviewer に先の結果を見せない
- Final Review Minimum: 1
- Human Gate: ready,merge
- Branch: agent/ej-grammar

Final Review Minimum は規則どおり 1: R4 でなく、予定 file に `scripts/ci/classify-changes.sh` が workflow と判定する path が無い（「この変更でどれかの required gate の green / red が変わるか」= 変わらない。Rust の IO module 1 本と docs だけ）。座組の Final Reviewer 2 本は運用で回す。Human Gate に `manual` を足さない: operator 画面・配布物・wire の変化が無い IO 層の変更で、Windows native で見る対象が無い。実データでの確認（AC11）は Coordinator が手元で行い、owner の作業にしない。

遷移記録（append-only）:

- kickoff → spec-check → design → plan-draft（本 commit、plan-first、2026-10-04、起草役）: Risk R3 を記録（下の Risk）。in-scope の設計正本は `29-io-ej-parser.md` の IO-08.3〜IO-08.8 で、記録の種類をモード欄で決める契約（旧 IO-08.3）が実物と矛盾し、文法表に無い形が多いため更新が要ると判定した（spec-check → design）。設計の決定を `29-io-ej-parser.md`（IO-08-D3a・D5a〜D5d・D6a〜D6d・D7a、型定義、全期間の構造所見、採らなかった案）、`docs/architecture/io-task-specs.md` の IO-08 節、`docs/ARCHITECTURE.md` の IO-08 の行、`docs/decision-log.md` の D-105 に書き、owner の判断を要する未決の論点は無い（design → plan-draft）。packet と Test Design Matrix を同じ commit に置く。
- plan-draft → plan-gate（2026-10-04、Coordinator）: packet と Test Design Matrix は plan-first commit `59a4f6ca` で揃い、doc check（`--target plan` と full）は Coordinator の再実行でも exit 0。
- plan-gate（round 1 の是正、2026-10-04、起草役、本 commit）: Plan Review round 1 は Opus reject（P2 2・P3 5）・Codex reject（P2 3・P3 3）。Coordinator の裁定、相談役の反例探し、実データの確認を反映した。Plan Commit は pending のまま。
- plan-gate（round 2 の是正、2026-10-04、起草役、本 commit）: round 2 の是正（Review Response 参照）。Plan Commit は pending のまま。
- plan-gate（round 3 の後の一括是正、2026-10-04、起草役、本 commit）: round 天井に達し、Coordinator の disposition（一括是正）を反映した（Review Response 参照）。Plan Commit は pending のまま。
- plan-gate（Review Response の記入、2026-10-04、Coordinator、本 commit）: round 1〜3 と一括是正の裁定を Review Response に書いた。Plan Commit は pending のまま。
- plan-gate → plan-approved（2026-10-04、Coordinator、本 commit）: 上限の Plan Review round 3（対象 `bdb5bea5`）の残りを Coordinator の disposition（一括是正、`d3489d00`）で閉じ、Coordinator が現物で確かめた（doc check の `--target plan` と full が ERROR 0）。一括是正の独立の再確認は Final Review が兼ねる。owner の判断（2026-10-04、一括是正の後に plan-approved）。Plan Commit は `116c118a`。
- plan-approved → implementing（2026-10-04、Coordinator、state-only、本 commit）: Writer（Opus 5.5 subagent、`subagent_type: writer`）へ実装を発注する。発注の直前に baseline を逐語で再実行し、起票時実測 #1（io::ej_parser、49 passed・ignored 1）、#2（exit 1）、#3（7）、#5（traceability OK）が上の baseline と一致。

## Owner Effort Budget

- 介入回数上限: 6（既定）
- 実働時間上限: 30分（既定。画面・L3 が無く、owner の作業は Codex の起動と Ready / merge の判断に限られる見込み）
- Plan Review round 天井: 3（既定 3）

| 種別 | 上限 | 消費（2026-10-04 起票時） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 6 | 2（起票の判断 TD-104、本 lane を含む lane 選択 1、round 3 の後の disposition と plan-approved 1〈2026-10-04〉） | 2（Ready 1、merge 1） | 2（実装中の Gated Amendment などで owner の判断が要る場合） | 6 = 2 + 2 + 2 |

消費と残りの見込みは owner の判断の回数だけを数える。relay（Coordinator が Codex 等を起動する往復）は数えない（`docs/DEV_WORKFLOW.md` `Owner Effort Budget`）。

既定値・数え方・上限に届くときの扱いは `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の R3「POS CSV」（POS の取込み形式）と `docs/project-profile.md` の High-risk Changes（CP932 の扱い、返品の負値の扱い）を、EJ parser の文法と出力型の変更に当てた。parser の復元規則は、後続の在庫連動が「この取引の明細は確かか」を判断する入口で、訂正・取消・練習の誤読は在庫の誤った減算に直結する。DB・command・画面・生成物は変えず、merge gate の合否も変えない。R4 の条件（実店舗データの露出）は Data Safety で閉じる。

## Goal

Goal Invariant:

### 最小完了条件

- 店が普段レジで打つ取引（値引き・訂正・取消・現金以外の支払）と精算が、EJ の 1 file を `parse_ej` に渡したとき安全側の「復元不能」に倒れず、設計正本（`29-io-ej-parser.md` IO-08-D3a〜D7a）どおりに読める: 訂正で取り消した明細を除いた明細と値引きが返り、取引中止・練習・精算票（次のヘッダで閉じた中断を含む）・点検票・設定・領収書は明細を返さない既知の記録として返る。file の最後の記録で終わりの印字の無い精算票は、EOF を閉じの根拠にせず今どおり復元不能（IO-08-D4）。合成 fixture の test で形ごとに示す（AC1〜AC8）。
- 記録の種類（`EjRecord.kind`）が見出しのモード欄でなく本文の行で決まり、モード欄が `精算` でない完了した精算票も `Settlement { completed: true }` になる（AC1）。
- 読めない形（未知の行・未知のモード・照合の不一致・訂正の取消先の不一致・本書に無い並び）は、今どおりその記録だけの復元不能（明細を返さない）に倒れる（AC9）。
- 旧 parser（`76de30d8`）で `Restored` / `NoItems` だった実物の記録は、新 parser でも同じ復元状態で、`Restored` の明細は同じになる。実物で残る復元不能は Non-scope の形の記録だけになる（AC11、Coordinator の手元の確認。件数は tracked に書かない）。

### 失敗定義

- 訂正で取り消した明細・取引中止の記録の明細・練習の記録の明細・値引きの行が、在庫を減らす明細として `items` に入る。
- `Unresolved` の記録から明細か値引きを読み出せる。file のどこかの行が、記録にも先頭断片にも現れずに消える。
- 旧 parser で復元できた記録の明細が変わる（backlog の「既存の復元結果を変えない追加に限る」）。
- 実データ（名称・金額・日時・番号・件数）が repo・PR・診断の文言に入る。
- 他の module（Z004 / 日報 parser、BIZ、command、bindings、DB）の振舞いが変わる。

### 非目的

- 精算区間・欠けの定義・file の選び方・番号の連続・Z004 との対応・番号印字と時計の痕跡の意味づけ（IO-08.9 / IO-08.10 と ADR は後続の design lane「実測と POS 系列の対応」の持ち場。本 lane は渡す事実を follow-up に書く）。
- 小数の数量・点数の受理（単位の拡張の lane、`docs/backlog.md`）。
- 値引きの金額を明細へ配ること・取引の効果（返品・取消）を在庫へ当てること（BIZ）。
- 日次の EJ 取込みの画面・BIZ・command。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

本 packet は data 契約（EJ の記録の種類・文法・出力型）を変えるため表を置く。この lane の時点で利用者は画面から EJ を扱わず、表の「利用者」は parser の呼出し側（後続の BIZ と訪店の確認）。「この packet を完了できる」と「店の通常運用（毎日の締めで EJ を取り込み在庫へ反映する）を達成できる」は別で、後者は後続の design lane と日次取込みの lane（IO-08.10）を要する。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 普段の一日の EJ 1 file（販売・訂正つきの販売・％値引き・クレジットだけの支払・取引中止・入金・領収書・精算） | `parse_ej` に生バイトを渡す | 記録ごとに `kind` と復元状態。販売は訂正で取り消した明細を除く明細と値引きつきの `Restored`、取引中止・入金・領収書・精算票は `NoItems` | 後続が区間の照合と在庫反映に使える | 形は全期間の EJ の構造所見で確認済み（Contract Probe 1） |
| 精算をモード欄が空・`点検`・`PGM` のまま打った日 | 同上 | 精算票が `Settlement { report: Daily, completed: true }` | 後続が区間の区切りに使える | Contract Probe 1 |
| 送信の失敗で中断した精算票と再実行の精算票が並ぶ file | 同上 | 中断は `completed: false`、再実行は `completed: true`。どちらも `NoItems`（中断は次のヘッダで閉じた記録。file の最後の記録で終わりの印字が無ければ `completed: false` の `Unresolved`〈`IncompleteRecord`〉） | 区切りに使う精算の選び方は後続の design lane | Contract Probe 1 の実データ (a)（中断はどれも file の最後の記録でない） |
| 練習モードで打った取引・入金と、トレーニング開始 / 終了の表示 | 同上 | `Training` の `NoItems`。明細を返さない | — | 練習が売上に入らないことは精算票の件数で確認（Contract Probe 1） |
| 部門キーで小数の数量を売った取引 | 同上 | その記録だけ `Unresolved`（`UnknownLine`）。他の記録は変わらない | 単位の拡張の lane で受ける | Non-scope |
| 本書に無い並び（例: 訂正の直前が値引きでも明細でもない） | 同上 | その記録だけ `Unresolved`（`InconsistentRecord`） | 実物の形を確かめてから文法を足す | — |
| 訪店後、全期間の EJ（repo 外）が手元にある | Coordinator が ignore 付きの probe test を環境変数で実行 | 件数と記録の番号・診断 code だけの出力 | 旧 parser との突合で差が無い（AC11） | AC11 |

## 起票時実測（2026-10-04、`76de30d8`）

この worktree で実行した出力。

| # | 対象 | command | 出力 | 扱い |
|---|---|---|---|---|
| 1 | 対象 test | `cd src-tauri && cargo test --lib io::ej_parser` | `test result: ok.`（失敗 0、ignored 1 = `real_ej_structure_probe`） | 実装後も全 PASS（AC10） |
| 2 | 新しい型 | `rg -c 'enum EjRecordKind\|struct EjAdjustment' src-tauri/src/io/ej_parser.rs` | 出力なし（exit 1、0 件） | 実装後 2（AC10） |
| 3 | 旧い行種名 | `rg -n 'SettlementTitle\|SettlementEnd' src-tauri/src/io/ej_parser.rs \| wc -l` | 7 | 実装後 0（`ReportTitle` / `ReportEnd` へ） |
| 4 | branch の差分 | `git diff --name-only origin/main...HEAD \| wc -l` | 0 | plan-first の後は Scope の file だけ（AC12） |
| 5 | traceability | `cd src-tauri && cargo run --bin generate_traceability -- --check` | `traceability check: OK（ERROR 0 件 / WARN 0 件）`（設計正本の更新を当てた worktree で実行） | 実装後も OK（AC10） |
| 6 | 旧 parser の実物での結果 | Contract Probe 2 | 件数は数え直しの前のため tracked に書かない（未実測扱い） | AC11 の突合の基準 |

## 段階（slice）

1 packet・1 PR のまま、Writer は次の 3 段階で commit を分け、各段階の終わりで対象 test を全 PASS にする（途中の段階を main に置かない）。分けるのは review と bisect のためで、Goal の最小完了条件は 3 段階すべて。

| 段階 | 中身 | 設計 |
|---|---|---|
| 1 記録の種類と明細を持たない記録 | 出力型（`EjRecordKind` ほか）、モード欄の既知の値、精算票（ＰＬＵ Z・勤怠 Z・表に無い行・中断）、点検票、設定、練習、領収書、戻のモードの入金 / 出金 / 替、番号印字 | IO-08-D3a / D5d / D7a |
| 2 取引の合計域 | 合計域の折返し、現金以外の支払行、照合の金額の選び方、合計域の訂正、軽減税率の行、全角 `－` の金額 | IO-08-D5a / D5b / D6c |
| 3 取引の明細域と取引中止 | 明細の折返し、訂正（負・正）、小計値引き・明細値引き、マイナスキー、戻の印の明細、取引中止の記録、`adjustments` | IO-08-D5a / D5c / D6a / D6b / D6d / D7a |

複数 packet に割らない理由: 出力型の変更（`kind`・`adjustments`）が 1 つの契約で、段階 2・3 は段階 1 の記録の種類の判定の上に立つ。module も 1 file で、Contract Audit を 1 回にまとめられる。

## Scope

- S1 `src-tauri/src/io/ej_parser.rs`: `29-io-ej-parser.md` の IO-08-D3a・D5a〜D5d・D6a〜D6d・D7a の実装と、同じ file の `#[cfg(test)]` の test（Matrix）。出力型の変更（`EjRecord.kind`、`EjRecordKind`、`EjSettlementReport`、`EjMode` の variant、`EjLineKind` の variant と `SettlementTitle` / `SettlementEnd` → `ReportTitle` / `ReportEnd`、`Restored.adjustments`、`EjAdjustment`、`EjAdjustmentKind`）。`parse_ej` 以外の `pub fn` を作らない。合成 fixture は既存の builder（`lr` / `zen` / `yen` / `item` / `qty` ほか）で 24 バイトの行を組む。fixture file は置かない。既存 test（`parse_ej_amount_formats`・`parse_ej_amount_i64_bounds`・`parse_ej_zero_quantity_or_negative_unit_price_unresolves`）が直接呼ぶ内部関数 `parse_amount(&str) -> Option<(i64, bool)>` と `parse_quantity(&str) -> Option<(i64, i64)>` の signature を保つ（全角 `－` の符号への対応は `parse_amount` の内部で行う。AC7 でこれらの test を変えないため）。
- S2 同 file の既存 test の更新（設計の変更に合わせる。理由は Matrix の「既存 test の変更」）: `parse_ej_unrecognized_header_mode_unresolves_record`（`点検` が既知になったため未知の値の文字列へ替え、`点検` のモードの取引の本文は別 test で `Unclassified` を見る）、`parse_ej_program_body_with_other_line_unresolves`（PGM の本文の通貨記号つきの行は `UnknownLine` のまま、区切りだけの PGM は `Settings` の `NoItems` へ）、`parse_ej_paid_in_line_in_return_mode_unresolves`（戻のモードの入金を `CashMovement` へ）、`mixed_file()`（`parse_ej_every_line_is_accounted_for` と `parse_ej_diagnostic_messages_are_fixed_texts` の fixture。`sale("点検", …)` のモードを未知の値 `ZZZ` に替え、dedup した code 列の期待値〈7 code〉は保つ。相談役の走査で、是正後の dedup 列は期待の 7 code と一致。`every_line` の名称の検査の `"点検"` も同じ値へ。G-X1 で `fixed_texts` に記録を足すときはこの 7 code の順を崩さない位置に限る）、`parse_ej_settlement_and_program_records` ほか `SettlementTitle` / `SettlementEnd` / `Restored { items, item_count }` の pattern を使う test（型の追従だけ）。どの test も assert を弱めず、期待値は設計正本から導く。
- S3 同 file の `real_ej_structure_probe`（T-R1）: 出力に記録の種類 × 復元状態の件数を足し、`Unresolved` の記録ごとに file 名・記録の番号・種類・診断 code（初出順）を 1 行ずつ出し、`Restored` / `NoItems` の記録ごとに file 名・記録の番号・復元状態・明細と値引きの digest を出す。digest は次の文字列の SHA-256（小文字 hex）の先頭 16 桁: 明細ごとに名称・数量・単価・金額の 4 field を `\x1f`（U+001F）で区切り、明細の境目を `\x1e`（U+001E）で区切って出現順に並べる。数量・単価・金額は 10 進の ASCII（負は先頭に `-`）、単価の `None` は `-` 1 文字。`adjustments` が空でなければ、明細の列の後に `\x1d`（U+001D）を置き、値引きごとに種類の名前（`ItemDiscount` / `SubtotalDiscount` / `MinusKey`）と金額を `\x1f` で区切って `\x1e` で並べる。`adjustments` が空なら明細の列だけで、base の式と同じ文字列になる。base 側（AC11）は同じ関数を scratch の T-R1 に写して使い、式を別に書き直さない。値・名称・行の文字列は出さない。probe の `assert_eq!(restorations.get("Unresolved"), None)` と `assert!(codes.is_empty())` の両方を外し、`fatal == 0` と読んだ file 数 > 0 の assert は残す（Non-scope の形と診断が実物に残るため。残る復元不能の許容は AC11 (d) で判定し、突合は Coordinator が手元で行う）。probe の `match record.mode` は exhaustive なので `EjMode` の新しい variant に追従させる。probe の変更の記述は本項だけを正本とし、Matrix は本項を参照する。
- S4 設計正本（plan-first の commit に含める。Writer は実装と食い違えば Gated Amendment で直す）: `docs/function-design/29-io-ej-parser.md`（冒頭・型定義・IO-08.1 の手順 7・IO-08.3〜IO-08.8・構造所見・採らなかった案。IO-08.9 / IO-08.10 は触らない）、`docs/architecture/io-task-specs.md` の IO-08 節、`docs/ARCHITECTURE.md` の IO-08 の行、`docs/decision-log.md` の末尾の D-105。
- plan-first の commit（起草役）: 本 packet、Test Design Matrix、S4。`docs/Plans.md` は触らない（PK4 は `## 次の行動` の `docs/plans/` への pointer の行だけを見る。D-097）。

生成物: `src/lib/bindings.ts` は再生成しない（command / DTO の変更なし）。`docs/function-design/90-traceability.md` は再生成しない（新しい test は REQ token を持たず spec ID `IO-08-Dn` だけを使う。起票時実測 #5 で設計正本の更新後も `--check` OK）。

呼出し側の確認: `rg -n 'ej_parser|EjMode|parse_ej' --glob '!src-tauri/src/io/ej_parser.rs' -g '!docs/archive/**'` の hit は `src-tauri/src/io/mod.rs:4`（module 宣言）、`src-tauri/tests/design_compliance_test.rs:162`（doc と module の対応）と docs だけで、出力型を使う BIZ・command は無い（2026-10-04）。`design_compliance_test` は `pub fn` だけを突き合わせるため、型の追加で変わらない。

### 共有 file の所有（4 lane 並走）

| file | 本 lane（D）の持ち場 | 他 lane | 重なりの見込み |
|---|---|---|---|
| `src-tauri/src/io/ej_parser.rs` | 全体 | なし | なし |
| `docs/function-design/29-io-ej-parser.md` | 冒頭〜IO-08.8、構造所見、採らなかった案 | IO-08.9 / IO-08.10 は後続の design lane（並走しない） | なし |
| `docs/architecture/io-task-specs.md` | IO-08 節だけ | A が IO-02、B が IO-07 の節を触る可能性 | 節が違う。merge 順で両方を残す |
| `docs/ARCHITECTURE.md` | IO 層の表の IO-08 の行だけ | A / B が IO-02 / IO-07 の行を触る可能性 | 行が違う |
| `docs/decision-log.md` | 末尾の D-105 の 1 件だけ | A = D-103、B = D-104、E = D-106 | 末尾への追記。merge 順で並べ、番号は予約どおり |
| `docs/plans/2026-10-04-ej-grammar.md` と Matrix | 全体 | 各 lane は自分の packet | なし |
| `docs/db-design/pos-tables.md`、ADR、`docs/project-memory.md`、`docs/backlog.md`、`docs/Plans.md`、`docs/PROJECT_HANDOFF.md` | 触らない | A / B は pos-tables の自分の節だけ。他はどの lane も触らない | なし |

`29-io-daily-report-parser.md`（日報 parser、lane B）は番号 29 を共有する別 file で、本 lane は触らない。

## Non-scope

- `29-io-ej-parser.md` の IO-08.9 / IO-08.10、ADR（`docs/adr/2026-09-18-stocktake-time-evidence.md`）、`32-biz-csv-import-service.md` の外部 probe 表: 後続の design lane「実測と POS 系列の対応」の持ち場。IO-08.9 の「未観測の形式」の列挙は本 lane の後に古くなるが、IO-08.5 の冒頭に「本節の表が受理する形は本節に従う」と書き、列挙の更新は follow-up にする。
- 小数の数量・点数・総売の点数の受理（`docs/backlog.md` の単位の拡張の項目が EJ の数量の行を範囲に含める）。それまで小数の行は `Unknown` で、その記録は復元不能のまま。
- `AUTO` / `BT` / 結果の 3 行の記録（2023-09 が最後、意味が未確認）と、最初の file の先頭の `INIT` / `PGM` の記録（番号行が 6 桁だけ）: 復元不能・先頭断片のまま。follow-up。
- 時計の再設定の記録の意味づけ（時計の痕跡は backlog で「実施しない」）。本 lane は `Settings` の `Text` として読めるようにするだけ。
- 番号印字・`一連No.`・精算票の値（件数の行を含む）の解釈。行種として読むだけ。
- 値引きの金額の明細への配分、返品・取消の在庫への効果（BIZ）。
- 他の parser・BIZ・command・bindings・DB・画面。新しい crate 依存。

## Acceptance Criteria

baseline は起票時実測。test 名は Matrix のもの。

- AC1 記録の種類（IO-08-D3a）: `cd src-tauri && cargo test --lib io::ej_parser` で Matrix の G-K1〜G-K8 が PASS。モード欄が空・`点検`・`PGM` の完了した精算票が `Settlement { report: Daily, completed: true }`、モード欄が空・`精算` の点検票の題の記録が `Inspection`、`練習` のモードの取引と `点検` のモードのトレーニングの表示が `Training`、未知のモードが `Unclassified` + `UnrecognizedMode`、`精算` のモードの題の無い記録が `Unclassified` + `IncompleteRecord`。練習の記録でも 24 バイトでない行は `InvalidWidth` で `Unresolved`（G-K3）。
- AC2 精算票・点検票（IO-08-D5d / D7a）: G-S1〜G-S6 が PASS。ＰＬＵ Z・勤怠 Z が `completed: true`、`信在高` / `貸在高` / `券在高` / `領収書 N 件` / `取引中止 N 件` / 送信の異常終了の後の 4 桁の行 / 案内文の行を含む精算票が `NoItems`・診断 0、終わりの印字の無い精算票が、次のヘッダで閉じた記録なら `completed: false` の `NoItems`、file の最後の記録なら `completed: false` の `Unresolved`（`IncompleteRecord`。既存の `parse_ej_record_truncated_at_eof_unresolves` は変えずに PASS）、4 形の点検票の題が `Inspection` の `NoItems`、精算票の本文の未知の行（数字を含む名称の行）は `UnknownLine`。
- AC3 設定・練習・入金 / 出金・領収書・番号印字（IO-08-D3a / D5d / D7a）: G-O1〜G-O6 が PASS。`SD設定読込み` / `自動設定保存` の状態の記録、通貨記号の無い設定の印字、本文 0 行の `PGM1` が `Settings` の `NoItems`、設定の記録で通貨記号を含む行（状態ラベルで始まる行を含む）と半角空白 + `点` を含む行は `UnknownLine`。戻のモードの入金が `CashMovement`。領収書の 3 行が `Receipt`。番号印字の行が合計域・設定・入金の記録の中で `NumberPrint` になり、記録の種類と照合を変えない。取引の明細域の番号印字の行は `NumberPrint` にならず `UnknownLine` で `Unresolved`（G-O6）。
- AC4 合計域（IO-08-D5a / D5b / D6c）: G-T1〜G-T8 が PASS。`対象計` の折返し、支払行の折返し、`お預り` / `お  釣` の折返し、`合  計` の無い支払行 1 行だけの取引（`ｸﾚｼﾞｯﾄ電子M` / `売掛` / `商品券`）、`合  計` + 支払行 + 現金、合計域の訂正（直前の支払行・現金行の取消。`合  計` の無い取引で、取消の後に差し替えた支払行 1 行で照合する形を含む）、軽減税率の行、全角 `－` の負の取引が `Restored` で、`items` と `item_count` が fixture の期待値どおり。
- AC5 明細域（IO-08-D5a / D5c / D6a / D6b / D6d）: G-M1〜G-M9 が PASS。負の訂正で取り消した明細が `items` に入らず（数量行つき・折返しの明細を含む）、`item_count` が取り消した数量を除く。正の訂正で取り消した値引きが `adjustments` に入らない。小計値引きが `SubtotalDiscount`、明細値引きが `ItemDiscount { item_line_no }`（直前の明細の行番号）、マイナスキーが `MinusKey` で `adjustments` に入る。戻の印の明細が数量 -1・負の金額。名称だけの行 + 金額の行が 1 明細（数量行が前に付く形を含む）。すべて取り消した取引が `items` 空の `Restored`。
- AC6 取引中止（IO-08-D7a）: G-C1〜G-C3 が PASS。明細・数量・訂正・値引きの後に `取引中止 ････` で終わる記録が `Cancelled` の `NoItems`（明細を返さない）。項目の規則の不一致は `InconsistentRecord`。
- AC7 既存の復元の互換: 起票時実測 #1 の既存 test（S2 の更新を除く）が変更なしで PASS し、既存の `Restored` の fixture の `adjustments` が空。
- AC8 型による閉じ: `Unresolved` は明細・値引きの field を持たない（型の定義を review で確認し、既存の `parse_ej_unknown_line_in_item_region_unresolves_record_only` が variant を assert）。既存の `parse_ej_every_line_is_accounted_for` と `parse_ej_diagnostic_messages_are_fixed_texts` に新しい行種の fixture を足して PASS（行の網羅と固定文言、Matrix G-X1）。
- AC9 fail-closed: Matrix の G-F1〜G-F15 が PASS。直前でない同額の明細への訂正（G-F1）、直前の項目の無い訂正・訂正の連続・金額 0 の訂正（G-F2）、率の行の後に ％値引きが無い・％値引きの前に率の行が無い（G-F3）、小計の不一致（G-F4）、`*` つきの ％値引きの直前が小計でない・`*` なしの ％値引きの直前が小計（G-F5）、％値引き・マイナスキーの金額が正（G-F6）、戻のモードの戻の印（G-F7）、戻の印の無い負の明細・戻の印の直後が正の明細（G-F8）、`合  計` が無く支払行 2 行・0 行（G-F9）、合計域の訂正の直前が支払行でない・金額が合わない（G-F10）、取引の明細域の取引中止の印（G-F11）、幅の混ざった金額 token（G-F12）、前の行の無い続きの行・金額が `i64` に収まらず読めない明細の行の後の続きの行（G-F13）、小数の数量・点数（G-F14）、`合  計` が無く取り消した支払行だけが候補・候補 2 行で先頭の 1 行だけが明細の和と一致（G-F15）が、その記録だけ `Unresolved`（Matrix の各行の診断 code）で、同じ file の他の記録の結果は変わらない。`点検` のモードの取引の本文は AC1（G-K8）で見る。
- AC10 gate: `cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test` PASS、`cargo test --test design_compliance_test` PASS、`cargo run --bin generate_traceability -- --check` が `OK`（起票時実測 #5）、`rg -c 'enum EjRecordKind|struct EjAdjustment' src-tauri/src/io/ej_parser.rs` = 2（baseline 0）、`rg -n 'SettlementTitle|SettlementEnd' src-tauri/src/io/ej_parser.rs | wc -l` = 0（baseline 7）。
- AC11 実物での確認（Coordinator、repo 外）: 実装 HEAD と base `76de30d8` の両方で、S3 と同じ digest を出す probe を全期間の EJ に当て（base 側は Coordinator が scratch で T-R1 に同じ出力と S3 の digest の関数を写して足す）、(a) fatal 0、(b) base で `Restored` の記録はすべて新でも `Restored` で digest が一致、(c) base で `NoItems` の記録はすべて新でも `NoItems`、(d) 新で `Unresolved` の記録が Non-scope の形（小数の数量・点数、`AUTO` / `BT` の記録）だけであることを、repo 外の集計 script（本文で記録の種類と行種を決める見落としチェックの script）の分類と記録の番号で突き合わせる。出力と突合の結果は repo 外（`.local`）に置き、件数は数え直しの前のため tracked と PR body に書かない（PR body には a〜d の成否だけを書く）。差があれば、どの規則が足りないかを構造だけで報告し Gated Amendment で直す。(d) が許すのは観測した Non-scope の形だけで、練習のモード以外で文法表に当たらない行（`29-io-ej-parser.md` の構造所見のとおり、観測ではなく規則として `Unknown` にする行）による `Unresolved` が出たら、Ready へ進めずに止まり、どの形かを構造だけで報告する（文法に足すか Non-scope に加えるかは Gated Amendment で決める）。
- AC12 範囲と data safety: `git diff --name-only origin/main...HEAD` が S1〜S4 と packet・Matrix の file だけ（`src/lib/bindings.ts`・`docs/function-design/90-traceability.md`・他の `src-tauri/src` の file を含まない）。追加・変更の fixture の文字列は合成（架空の名称・金額・日時・番号）で、実物の file・probe の出力を commit しない（`git status --short` に `.TXT` と log が現れない）。
- AC13 docs: `bash scripts/doc-consistency-check.sh --target plan` と `bash scripts/doc-consistency-check.sh` が ERROR 0。

## Design Readiness

設計の完了条件は `docs/DEV_WORKFLOW.md` Design Phase Rules の Design completion criteria。

- 引用する設計正本（節まで）: `docs/function-design/29-io-ej-parser.md` の型定義・IO-08.3（IO-08-D3a）・IO-08.5（D5a〜D5d、金額 token）・IO-08.6（D6・D6a〜D6d）・IO-08.7（D7a）・IO-08.8・全期間の EJ の構造所見・採らなかった案（本 commit で更新）。上位: ADR SPEC-STK-TIME-D5（`docs/adr/2026-09-18-stocktake-time-evidence.md:173`、`:179`「日計合計一致だけを完全性の証明にしない」、`:180`「訂正・取消を区別する。未知の行・未知の取引形式を黙って除外しない」）、D-023（`docs/decision-log.md:162`）、D-105（本 commit）。
- 必要な設計成果物: Backend function / IO の契約 → `29-io-ej-parser.md`・`docs/architecture/io-task-specs.md` IO-08・`docs/ARCHITECTURE.md` IO-08 の行（updated in this PR、plan-first の commit）。Command / DTO / binding → なし（existing sufficient）。DB → なし。Screen / UI → なし。Import format → EJ の行の文法を `29-io-ej-parser.md` と本 packet の Boundary / Wire Contract に記録（updated in this PR）。Durable decision → D-105（updated in this PR）。
- plan にしかない durable な判断の昇格先: 記録の種類の決め方・訂正 / 値引きの意味・照合の金額の選び方・値引きを配らないこと・中断した精算票の扱いは `29-io-ej-parser.md` と D-105 に置いた。本 packet にだけある判断は段階の切り方と AC11 の突合の手順だけ。
- 前提・制約と、延期した design gap の follow-up: 文法は全期間の EJ の観測に限る（未観測の並びは復元不能）。延期: 小数（単位の拡張）、`AUTO` / `BT`、先頭の `INIT`、IO-08.9 の列挙、区間・番号印字の意味（下の Follow-up）。
- 絶対保証の例外と escape hatch の自己点検: 「どの行も失われない」は先頭断片・ヘッダ・本文のどれかに必ず入り、練習・未知のモードの本文も `Text` / `Unknown` で残る（例外なし）。「`Unresolved` から明細を読めない」は型で保証し、本文の行から組み直す経路は呼出し側の責任（IO-08.8）。「取り消した明細は `items` に入らない」は、訂正が直前の項目の取消として照合された場合だけで、取消先が合わなければ記録ごと `Unresolved`（例外なし）。「練習・取引中止は明細を返さない」は種類で決まり、照合を経ない（例外なし）。
- 判定（ready / not ready）と理由: ready。将来の実装者は `29-io-ej-parser.md` と D-105 だけで、何を作るか・なぜか・何を採らなかったかに答えられる。Matrix の各行は設計正本の決定 ID から導ける。owner の判断を要する未決の論点は無い（製品の振舞いの決め事は backlog の入力と既存の owner 決定〈取消・練習は在庫を減らさない記録として読む〉の範囲）。

## Registration / Generation Obligations

| 変更対象 | 登録・生成義務 |
|---|---|
| function-design doc の更新（新設ではない） | `design_compliance_test` の対応（`29-io-ej-parser.md` → `io::ej_parser`）は既存のまま。`parse_ej` 以外の `pub fn` を作らない |
| REQ / coverage | 該当なし（新しい test は REQ token を持たない。AC10 の `--check`） |
| Tauri command / route / operator 画面 | 該当なし |

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | Which concepts belong to replaceable external adapters, and which concepts are stable app-core contracts? | Architecture, function design, decision-log, Plan Packet | CASIO の行の形（ラベル・折返し・全角 `－`・`*` / `※`）は adapter の事実。記録の種類・明細・値引き・復元状態は後続の BIZ が使う core 側の契約で、レジが替わっても保つ形にする（D-023）。モード欄は adapter の内側で使い、種類は本文で決める | D-105、IO-08-D3a |
| Fact check / design decision split | Which claims are observed facts from hardware/tool/files, and which are app decisions that need source-doc promotion? | Investigation doc, source design docs, decision-log | 事実: 行の形・題の位置・訂正が直前の取消であること・点数の数え方・練習が純売に入らないこと（構造所見、件数は repo 外）。決定: 種類の判定順・値引きを配らない・照合の金額の選び方・次のヘッダで閉じた中断を `NoItems` にし、file の最後の記録では終わりの印字を必須にする・小数を受けない（IO-08-D3a〜D7a、D-105） | `29-io-ej-parser.md` |
| Lifecycle / retry | What happens before, during, after, and after failure for import/export, duplicate input, rollback, retry, cancellation, and re-run? | Function design, DB design, UI design, Test Matrix | parser は純関数で状態を持たない。中断した精算と再実行は別の記録として返り、どちらを区切りにするかは IO の外 | IO-08-D7a、Follow-up |
| Operator workflow | What does the operator do in the real sequence across app, external tool, media, print/export, backup, and recovery? | Screen/UI design, function design, Plan Packet manual checks | この lane で operator の操作は変わらない。店の運用（値引き・訂正・取消・クレジット払い・練習・領収書）を変えずに読めるようにする | — |
| Replacement path | If the external system changes, which files/modules/docs are replaced and which app-core contracts remain stable? | Architecture, function design, decision-log | レジが替われば `ej_parser.rs` と `29-io-ej-parser.md` を取り替え、出力型（種類・明細・値引き・復元状態）は保つ | `29-io-ej-parser.md` |
| Data safety / evidence | How can the claim be supported by anonymized shape/count/hash/procedure evidence without committing real store data? | Plan Packet Data Safety, investigation doc, review evidence | 形の確認は値を伏せた probe、実物との突合は記録の番号と digest だけ。件数は数え直しの前のため tracked に書かない | Data Safety、AC11 |
| Reporting / accounting semantics | Are totals, summaries, item records, returns, corrections, and inventory movements modeled separately enough to avoid false business meaning? | DB design, function design, report design, Test Matrix | 明細（取り消しを除く）・値引き（種類つき）・取引中止・練習・返品のモード・戻の印の明細を区別して返し、値引きを明細に混ぜない。支払の種類は照合にだけ使い、出力にしない（売上の支払の内訳は Z 帳票の持ち場） | IO-08-D6a〜D6d |
| Manual verification | Which assertions cannot be proven by automated tests and require Windows native L3, external tool import, or real-device confirmation? | Plan Packet, Test Matrix, PR body | 実物の形への一致だけで、AC11 の Coordinator の手元の確認で閉じる。Windows native L3 は不要 | AC11 |
| 環境・再現性 | 新設の環境依存（toolchain / CI runner / OS 差異等）を repo-pinned config で強制するか、明示的に defer するか | repo-pinned config, Plan Packet | 新しい環境依存なし（既存 crate の `sha2` / `encoding_rs` だけ）。T-R1 は ignore 付きで CI では実行しない | — |

## Boundary / Wire Contract

- producer: `parse_ej`（`src-tauri/src/io/ej_parser.rs`）。入力は店のレジの EJ 1 file（CP932・CRLF・24 バイト固定幅）。
- consumer: 未実装の後続 BIZ（日次 EJ 取込み）と訪店の確認。repo に呼出し側は無い。
- wire type: なし（Tauri command・DTO・bindings を経ない Rust の内部型）。
- internal type: `EjParseResult` / `EjRecord`（`kind` を追加）/ `EjRecordKind` / `EjSettlementReport` / `EjMode`（variant 追加）/ `EjLineKind`（variant 追加、`ReportTitle` / `ReportEnd` へ改名）/ `EjRestoration::Restored { items, item_count, adjustments }` / `EjAdjustment` / `EjAdjustmentKind`。正本は `29-io-ej-parser.md` の型定義。
- precision/range: 金額・数量・点数は `i64`。小数は受けない。金額 token の字形は IO-08.5。
- round-trip path: なし（読み取りだけ）。
- invalid input: 致命的エラー 3 種（不変）と記録単位の `Unresolved` + 診断 7 code（不変）。
- compatibility: 呼出し側が無いため既存の consumer は壊れない。旧 parser で復元できた実物の記録の結果は変えない（AC11）。

## Test Plan

Test Design Matrix: [2026-10-04-ej-grammar](test-matrices/2026-10-04-ej-grammar.md)。

- targeted tests: `cd src-tauri && cargo test --lib io::ej_parser`（Matrix の G-K / G-S / G-O / G-T / G-M / G-C / G-X）。
- negative tests: Matrix の G-F1〜G-F15 と既存の負の test。
- compatibility checks: 既存 test（S2 を除く）の無変更 PASS、AC11 の旧 parser との digest の突合。
- data safety checks: fixture は合成だけ。T-R1 の出力は件数・file 名・記録の番号・code・digest だけ。
- main wiring/integration checks: `design_compliance_test`、`generate_traceability -- --check`、`cargo clippy`。

## Review Focus

- 記録の種類の判定順（IO-08-D3a）: モード欄を使う 4 つの場面がモード欄で種類を決めた旧契約の取りこぼし（精算票・点検票）を再び生まないか。設定の記録をモード欄で決めることが、取引の行の紛れ込みを通さないか（行種の分類より前に通貨記号の行を、`Text` の候補で半角空白 + `点` を含む行を `Unknown` にする規則。負の明細の形は `Text` で通る限界は `29-io-ej-parser.md` の採らなかった案）。
- 訂正・値引きの項目の規則（IO-08-D6a / D6b）: 取り消した明細・値引きが `items` / `adjustments` に残る経路、または残すべきものが消える経路が無いか。「直前の項目」の定義と、数量行・折返しとの組合せ。
- 照合の金額の選び方（IO-08-D6c）: `合  計` が無いときの支払行 1 行の規則と合計域の訂正が、未観測の並びを受理しないか。
- 旧 parser の復元結果を変えない（AC7・AC11）: 旧で `Restored` だった並びが新でも同じ判定順で `Sale` になり、同じ明細になるか。`小計` / `訂正` の行が旧では `Item` と読めた形（ラベル + 通貨記号つきの金額）を新で取り違えないか。
- 精算票の `completed` と中断（IO-08-D7a）: 区切りの判断を IO に持ち込んでいないか（IO-08.9）。
- 段階の切り方と AC11 の突合の手順が成立するか。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| IO-08-D1 純関数・生バイト入力・file_hash・致命的エラー 3 種 | IO-08.1 | `parse_ej`（不変） | `parse_ej_file_hash_is_raw_sha256` / `parse_ej_decode_failure_is_fatal` / `parse_ej_no_record_header_is_fatal` / `parse_ej_empty_input_is_fatal` | AC11 (a) |
| IO-08-D2 CRLF・24 バイト固定幅・最終改行 | IO-08.2 | 不変 | `parse_ej_missing_final_crlf_reports_diagnostic` / `parse_ej_invalid_width_line_unresolves_record` / G-K3（練習の記録の幅違反） | — |
| IO-08-D3 2 行ヘッダ・モード欄の既知の値・日時と番号は文字列 | IO-08.3 | S1 `EjMode` | G-K1 / G-K7 / `parse_ej_normal_sale_restores_items` | — |
| IO-08-D3a 記録の種類を本文で決める（判定順の表、番号印字を除いて判定） | IO-08.3 記録の種類 | S1 `EjRecordKind` の判定 | G-K1〜G-K8 / G-O6 | AC11 |
| IO-08-D4 先頭断片・EOF は閉じの根拠にしない | IO-08.4 | 不変 | `parse_ej_record_truncated_at_eof_unresolves` / `parse_ej_leading_lines_before_first_header_are_reported` / G-S4（EOF 側の case） | — |
| IO-08-D5 位置による行分類・Unknown を残す・金額 token（全角 `－`、印 `*` / `※`） | IO-08.5 | S1 行分類・`parse_amount` | G-T7 / G-T8 / G-M8 / `parse_ej_amount_formats` / `parse_ej_unknown_line_in_item_region_unresolves_record_only` | AC11 |
| IO-08-D5a 折返し（名称だけの行 + 続き、ラベルだけの行 + 続き） | IO-08.5 | S1 `ItemName` / `Continued` | G-T1 / G-T2 / G-T3 / G-M6 / G-F13 | AC11 |
| IO-08-D5b 合計域の追加ラベル（支払 4 種・軽減税率・注記）と合計域の訂正の行 | IO-08.5 | S1 合計域の分類 | G-T4〜G-T7 | AC11 |
| IO-08-D5c 明細域の操作の行（訂正・小計・率・％値引き・マイナスキー・戻の印・取引中止の印） | IO-08.5 | S1 明細域の分類 | G-M1〜G-M9 / G-C1 / G-F7 | AC11 |
| IO-08-D5d 精算票・点検票・設定・練習・入金・領収書の行 | IO-08.5 | S1 | G-S1〜G-S6 / G-O1〜G-O6 | AC11 |
| IO-08-D6 明細の復元と照合（数量×単価・点数・非負・同名非合算・符号非反転） | IO-08.6 | S1 `restore_items` | `parse_ej_quantity_line_applies_to_next_item` / `parse_ej_repeated_same_name_lines_kept_separate` / `parse_ej_return_mode_keeps_positive_amounts` / `parse_ej_item_count_mismatch_unresolves` / `parse_ej_total_mismatch_unresolves` | AC11 (b) |
| IO-08-D6a 訂正 = 直前の項目の取消（負 = 明細、正 = ％値引き、合計域 = 支払行） | IO-08.6 | S1 | G-M1 / G-M2 / G-M3 / G-T6 / G-F1 / G-F2 / G-F10 / G-F15 | AC11 |
| IO-08-D6b 値引きを配らず `adjustments` で返す（小計値引き・明細値引き・マイナスキー、小計の照合） | IO-08.6 | S1 `EjAdjustment` | G-M4 / G-M5 / G-M7 / G-F3〜G-F6 | AC11 |
| IO-08-D6c 照合の金額 = `合  計`、無ければ取り消されていない支払行 1 行 | IO-08.6 | S1 | G-T4 / G-T5 / G-T6 / G-F9 / G-F15 / `parse_ej_exact_cash_tender_without_total_line` | AC11 |
| IO-08-D6d 戻の印の明細（空のモードだけ、数量 -1・負の金額、点数に −1） | IO-08.6 | S1 | G-M8 / G-F7 / G-F8 | AC11 |
| IO-08-D7 明細を持たない記録（入金 / 出金 / 替・設定・精算） | IO-08.7 | S1 | `parse_ej_non_item_records_paid_in_paid_out_exchange` / `parse_ej_settlement_and_program_records` / G-O3 | AC11 (c) |
| IO-08-D7a 取引中止・領収書・精算票（次のヘッダで閉じた中断を含む。file の最後の記録は `ReportEnd` 必須で `IncompleteRecord`）・点検票・設定・練習は `NoItems`、`Unclassified` は `Unresolved` | IO-08.7 | S1 | G-C1〜G-C3 / G-S1〜G-S6 / G-O1〜G-O5 / G-K5〜G-K8 / `parse_ej_record_truncated_at_eof_unresolves` | AC11 |
| IO-08-D8 型による閉じ・行の網羅・診断の範囲と固定文言（7 code 不変） | IO-08.8 | S1 | G-X1 / `parse_ej_every_line_is_accounted_for` / `parse_ej_diagnostic_messages_are_fixed_texts` | review（`Unresolved` に明細・値引きの field が無い） |
| IO-08-D9 IO が判断しないこと | IO-08.9（触らない） | 非実装 | なし | non-scope（`completed`・番号印字・精算票の値を解釈しないことを review で確認） |
| IO-08-D10 日次取込みとの接続 | IO-08.10（触らない） | なし | なし | non-scope |
| D-105 本文で種類を決め、普段の文法を足す | `docs/decision-log.md` | S1 / S4 | 上の各行 | — |
| 隣接: ADR D5「未知の行・未知の取引形式を黙って除外しない」「訂正・取消を区別する」 | ADR `:180` | IO-08-D3a / D5 / D7a | G-K5 / G-F11 / G-C1 | — |
| 隣接: ADR D5「日計合計一致だけを完全性の証明にしない」 | ADR `:179` | 照合は取引の構造の確認（IO-08.6 末尾） | なし | non-scope（区間は design lane） |
| 隣接: backlog「既存の復元結果を変えない追加に限る」 | `docs/backlog.md` の EJ の項目 | S1 | AC7（既存 test）/ G-T8 | AC11 (b)(c) |
| 隣接: 単位の拡張（小数は受けない） | IO-08.5 金額 token | 不変 | `parse_ej_decimal_quantity_unresolves` / G-F14 | non-scope |

## Contract Probe

- 全期間の EJ の行の形（記録の種類・題の位置・ラベル・折返し・訂正と値引きの並び・金額の字形・練習が純売に入らないこと）: repo 外の材料の見落としチェックの集計 script（本文で記録の種類と行種を決める、標準ライブラリだけ・read-only）の出力（2026-10-04）と、Coordinator 側 scratch の 2 つの probe（ラベルが固定の行だけ数字を `9` / `９` に伏せた形を出す `shape_probe.py`、記録の種類の判定順の前提〈題が先頭の行か・トレーニングの表示の記録の他の行・領収書の行の順・取引中止の最後の行・設定のモードの記録の種類と通貨記号・取引の区切りの数〉を数える `order_probe.py`）-> 形は `29-io-ej-parser.md` の「全期間の EJ の構造所見」のとおり。判定順の前提はすべて例外なし（精算票・点検票の題はどの記録でも先頭の行、トレーニングの表示の記録に他の行なし、領収書は常に `一連No.` → `領収No.` → `領収書`、取引中止の最後の行は常に `取引中止 ････` で区切りなし、設定のモードの記録で通貨記号を含むのは題で始まる精算票だけ、取引の区切りは常に 1 本）。件数は数え直しの前のため未実測扱いで tracked に書かない。
- Plan Review round 1 の後の実データの確認（Coordinator、手元の全期間。件数は tracked に書かず、形の事実だけ）: (a) 中断した精算票（終わりの印字の無い `日計明細 Z`）は、どれも file の最後の記録ではなく、すぐ後ろに同じ日・同じ末尾番号の終わりの印字のある精算票（再実行）がある。file の最後の記録はどれも終わりの印字のある精算票 -> IO-08-D7a の「次のヘッダで閉じた中断は `NoItems`、file の最後の記録は `ReportEnd` 必須」で実物の中断はすべて `NoItems` になる。(b) 本文の形で設定（区切り + 状態）と判定される記録に、半角空白 + `点`・半角 `\`・全角 `￥` を含む行は無い（CP932 で decode した後の文字列で判定）。モード欄が `PGM` で本文が `日計明細 Z` の精算票の記録には ` 点` と `\` の行があるが、題で先に精算票と決まる（IO-08-D3a の 2）-> 通貨記号の規則は、前項の「設定のモードの記録で通貨記号を含むのは題で始まる精算票だけ」と合わせて実物の設定の記録を復元不能にしない。` 点` の規則は区切りと状態の形の記録で成立し、設定の印字の記録に ` 点` の行があれば AC11 (d) の差として現れる（復元不能の側）。(c) 訂正のラベルの無い負の明細は、どれも直前が戻の印（`戻` で始まる印の行）で、戻の印の次の行はどれも負の明細 -> IO-08-D6d の「戻の印を持たない負の `Item` は `InconsistentRecord`」は実物の記録を復元不能にしない。
- マイナスキーの行の直前の項目（2026-10-04、起草役。repo 外の `.local/field-data/recount/plan-probes-2026-10-04/probe7_minus_key_prev.py`、標準ライブラリだけ・read-only、出力は件数と行種の名前だけ）: 全期間の EJ の明細域のマイナスキーの行（`－` + 空白 + 負の金額、`*` の有無の両方）は、どれも直前の項目が明細（戻の明細でない）で、モード欄が空の取引と取引中止の記録にだけ現れる -> IO-08-D6b の「マイナスキーの直前の項目は有効な明細（戻の明細を除く）」は実物の記録を復元不能にしない。件数は数え直しの前のため tracked に書かない。
- 旧 parser（`76de30d8`）の実物での結果: Coordinator 側 scratch で全期間の EJ の file を 1 つの dir に symlink し、`INVENTORY_EJ_PROBE_DIR=<dir> cargo test --lib io::ej_parser -- --ignored real_ej_structure_probe --nocapture` を実行 -> 全 file が `Ok`（fatal 0）、診断 code は `UnknownLine` / `UnrecognizedMode` / `IncompleteRecord` / `LeadingFragment` の 4 種で `InvalidWidth` と `MissingFinalNewline` は出ない。`Restored` / `NoItems` / `Unresolved` の件数は数え直しの前のため未実測扱い（repo 外に保存、AC11 の基準）。
- 既存 crate の前提（`encoding_rs::SHIFT_JIS` の strict decode が全角の `－` `％` `＃` `※` `･` と半角カナを decode できる）: 上の旧 parser の probe で全 file が decode できた（DecodeFailed 0）-> 成立。
- 未検証の外部前提（OS / 外部ライブラリの新規依存）: なし。

## Data Safety

- commit しない: 実物の EJ（`.TXT`）、probe の出力・log、数え直し前の件数、名称・金額・日時・番号・JAN・部門名・生の行。PR body にも件数を書かない（AC11 は成否だけ）。
- repo 外だけに置く: 全期間の EJ（`~/downloads/inventory-field-check/approved-readable/ej/`）、集計 script と出力（`.local/field-data/recount/`）、Coordinator 側 scratch の probe と symlink の dir。
- 合成だけ: `ej_parser.rs` の test の fixture（架空の名称・金額・日時・番号を builder で組む）。T-R1 の出力は件数・file 名・記録の番号・診断 code・digest（SHA-256 の先頭 16 桁）だけで、値を復元できない。

## Follow-up

後続の design lane「実測と POS 系列の対応」（IO-08.9 / IO-08.10・ADR・`32-biz-csv-import-service.md` の probe 表の持ち主）へ渡す事実と論点:

1. 精算票の件数の行は区間の記録数と一致する（repo 外の見落としチェックで全区間）: `純売 N 件` = 区間の通常 + 返品の取引の数、`純客 N 名` = 通常 − 返品、`総売 N 点` = 通常の点数 − 返品の点数、`取引中止 N 件` = 区間の取引中止の記録の数、`領収書 N 件` = 区間の領収書の発行の記録の数。区間 = 前の完了した日計明細 Z の後から次の完了した日計明細 Z まで（中断の後も数え続ける）。練習と取引中止は純売に入らない。EJ だけで区間の件数を照合できる（完全性の証明ではなく照合の材料）。
2. 区切りの候補は `Settlement { report: Daily, completed: true }`。Z004 は単独の `ＰＬＵ Z` でも 1 つ使う（`Settlement { report: Plu }`）。中断（`completed: false`）は区切りに使わない前提で決める。
3. 番号印字は `NumberPrint` の行（取引の合計域・自動設定保存の記録・入金の記録の中）で、独自の見出しを持たない。意味づけと記録の種類にするかは design lane。
4. 時計の再設定の記録は `Settings` の `Text` として読める。時計の痕跡の扱いは backlog の決定（実施しない）のまま。
5. IO-08.9 の「未観測の形式」の行の列挙が古くなる（本 lane の IO-08.5 が受理する形を含む）。列挙を「本書 IO-08.5 が受理しない形」へ直す。
6. 領収書の `一連No.` は直前の取引の記録の番号。ヘッダの 4 桁の欄は Z004 の machine_no と一致を観測（`29-io-ej-parser.md` の `number_prefix` の「意味は未検証」の注記を直すかは design lane）。
7. 最初の file の先頭の `INIT` / `PGM` の 2 記録は番号行が 6 桁だけで先頭断片になる。番号は 1 から欠けなく連続する。

その他: 小数の数量・点数は単位の拡張の lane（`docs/backlog.md`）。`AUTO` / `BT` の記録は意味が分かったら文法に足す（2023-09 が最後）。取消と訂正の店の呼び方の対応は未確認（店主は「取消を 1 週間以内に使った」と答えたが、EJ の最後の取引中止はそれより前で、直近は訂正がある。回答台帳 TD-050、repo 外）。文法の影響は無い。closeout で `docs/backlog.md` の EJ の文法の項目を消す。

## Implementation Results

exact-HEAD SHA と test 件数は書かない（D-035 / D-038）。

- 実装の要旨（closeout、2026-10-06）: `parse_ej` は記録の種類（`EjRecordKind`）を本文の行で決め、精算票（モード欄に依らない完了、ＰＬＵ Z・勤怠 Z、次のヘッダで閉じた中断）・点検票・設定・練習・入金 / 出金 / 替・領収書・取引中止を、明細を返さない記録として返す。取引の合計域（`対象計`・支払行・`お預り` / `お  釣` の折返し、`合  計` の無い取引の照合の金額）と明細域（訂正は直前の項目の取消、小計値引き・明細値引き・マイナスキーは `adjustments`、戻の印の明細）を読み、規則に合わない記録は今どおり `Unresolved` に倒す。Gated Amendment 1・2 の是正で、ほかの行種の形の行と `i64` に収まらない金額の行は `ItemName` にならず、数量行の単価は全角の `－` も拒み、合計域の `Continued` の受け手は正本の定義どおりになった（D-105）。
- review・CI・merge: Final Review の是正（Gated Amendment 1・2）の後の closure と、Amendments が変わった後の Claude 側の broad の取り直しがともに approve で、記録した。AC11（Coordinator、実物の EJ、repo 外）は是正の前後とも (a)〜(d) が PASS。Draft の run は success で、helper 経由の squash merge（owner の Ready・merge、2026-10-05）。manual は not-required。検証の実測値は PR の body が持つ。PR: [#141](https://github.com/higekiri-w/inventory-system-desktop/pull/141)（Writer = Opus 5.5 subagent）

### Gated Amendment 1（2026-10-05、Coordinator）

- Final Review broad の Codex 側（P1 1 / P2 1）と Claude 側の P3 を受けた是正。相談役（Fable 5.1）が反例を探し、29 の表の全行種を明細域に置いて確かめた。
- `ItemName` は、本書の表でほかの行種として形が定まった行（取引中止の印・番号印字・状態の行・合計域 / 入金・出金・替 / 領収書 / 精算票・点検票のラベルの行・題・終わりの印字・トレーニングの表示）に当たらない（29 の `ItemName` の行と「`ItemName` が当たらない形」）。今の実装ではこれらが明細域で名称の行になり、続きの行と組んで架空の明細で `Restored` になっていた（P1）。
- 数量行の単価は全角の `－` も符号として拒む（29 の `Quantity` の行と小数の項。P2）。
- 合計域の `Continued` の受け手を、29 の定義どおりラベルだけの行（注記を除く）と `対象計` の率だけの行に限る（Claude 側の P3 #2）。
- Matrix に G-F16〜G-F18・G-T9 と Mutation の 4 行を足す。Scope・AC・Goal は変えない（S1 の「IO-08-D5a〜D5d の実装」の範囲）。実装は本 Amendment の後の commit。

### Gated Amendment 2（2026-10-05、Coordinator）

- Final Review broad の Codex 側が PR に投稿した 2 本目の review（P1 EJ-2）を受けた是正。Gated Amendment 1 は 1 本目の review の件数（P1 1 / P2 1）だけを拾い、EJ-2 を落としていた。Coordinator が是正後の実装（`1cad2187`）の隔離 copy で再現した: `－ -9223372036854775809*` → ` ￥１００` → 区切り → ` 1 点` → `合  計` が、その行を名称とする架空の明細で診断 0 の `Restored` になる。
- 契約は変えない。29 の `MinusKey` の行（`－` + 空白 + 金額 token + 任意の `*`）、金額 token の「収まらない桁は受理しない（その行は `Unknown`）」、`ItemName` の「`i64` に収まらない桁で `Unknown` になる行は当たらない」で既に決まっている。実装の範囲外の判定が末尾の `※` だけを外し `*` を外していなかった（P1）。
- Matrix に G-F19 と Mutation の 1 行を足す。Scope・AC・Goal は変えない（S1 の範囲）。実装は本 Amendment の後の commit。

## Review Response

review の後に記入する。

- Plan Review round 1（2026-10-04、対象 `87c6c490`、互いに独立の 2 本）: fresh Opus 5.5 = reject（P2 2 / P3 5）、Codex GPT-6.1 Sol（発注 202）= reject（P2 3 / P3 3）。Coordinator が現物で裏取りして全件を採用し、相談役（Fable 5.1）の反例探しの後、起草役（Opus 5.5）が是正した（`03cbc27d`）。Phase は plan-gate のまま。
  - Opus F1（P2、終わりの印字の無い精算票の扱いが IO-08-D4 と食い違う）: 採用。IO-08-D7a を「次のヘッダで閉じた記録は中断の NoItems、file の最後の記録は `ReportEnd` を必須行とし IncompleteRecord」にし、G-S4 に EOF 側の case を足した。
  - Codex #1（P2、設定の記録の通貨記号の拒否が `Text` にだけ掛かり、先に判定する `Status` から抜けられる）・Opus F4（P3、設定の記録に紛れる取引の形）: 採用。設定の記録では行種の分類より前に、通貨記号の行と「半角空白 + 点」を含む行を `Unknown` にする（decode 後の文字列で判定）。G-O1・G-O2 に case を足した。
  - Codex #2・Opus F2（P2、S2 の期待値の変更だけでは AC7 の既存 test 無変更が成り立たない。`mixed_file()` の `点検` の記録）: 採用。S2 に `mixed_file()` の `点検` → `ZZZ` を足した。
  - Codex #3（P2、S3 が probe の assert の片方だけを外す）・Opus F7（P3、digest の形）: 採用。S3 で probe の assert を 2 つとも外し、digest は `adjustments` が空なら base と同じ式にした。
  - Codex #4〜#6・Opus F3・F5・F6（P3）: 採用。小計の和にマイナスキーを含める、`Sale` の注釈、負の fixture の値、G-T8 の並び、互換の対象を実物と既存 fixture に揃えた。Contract Probe に round 1 の後の実データの確認 (a)〜(c) を形の事実として足した。
- Plan Review round 2（2026-10-04、対象 `03cbc27d`、互いに独立の 2 本）: fresh Opus 5.5 = approve（P3 6）、Codex GPT-6.1 Sol（発注 207）= reject（P2 1）。Coordinator が全件を採用し、起草役（Opus 5.5）が是正した（`bdb5bea5`）。Phase は plan-gate のまま。
  - Codex #1（P2、`合  計` の無い取引で、照合の金額の候補の選び方を壊す誤実装を検出できない。G-T6 はすべて `合  計` 付きで、G-F9 の負例は和が明細の合計と一致する形なので「先頭の 1 行を選ぶ」誤実装も期待どおりの診断になる）: 採用。G-F15 を新設し（取り消した支払で候補 0 行 → IncompleteRecord、候補 2 行で先頭だけが一致 → InconsistentRecord）、G-T6 に `合  計` の無い差し替えの正の case を足した。
  - Opus #1〜#6（P3）: 採用。練習・未知のモードの番号印字の行種を 1 通りにし、digest の区切りと `None` の表し方を S3 で固定し、AC9 の本文を G-F に揃え、構造所見の未知の行を「規則」と明記して AC11 に止まる条件を書き、S1 に既存 test 3 本が呼ぶ `parse_amount` / `parse_quantity` の signature を保つことを書き、`－ … *` の直前の項目の観測を Contract Probe に足した（持ち帰りデータの集計。件数は tracked に書かず、形の事実だけ）。
- Plan Review round 3（上限）（2026-10-04、対象 `bdb5bea5`、互いに独立の 2 本）: fresh Opus 5.5 = approve（P3 4）、Codex GPT-6.1 Sol（発注 210）= reject（P2 1 / P3 1）。round 天井に達したので次の round を起こさず、Coordinator が `docs/DEV_WORKFLOW.md` Review Rules の disposition で全件を「一括是正」とした（owner 2026-10-04 が「一括是正の後に plan-approved、独立の再確認は Final Review」を選んだ）。起草役（Opus 5.5）が是正し（`d3489d00`）、Coordinator が現物で確かめた。
  - Codex #1（P2、`ItemName` の fallback が、`i64` に収まらない金額の明細の行の後に続きの行がある取引を架空の明細として受ける誤実装を test が見ていない）: 採用。G-F13 に case を足し（期待は先頭の行と続きの行が `Unknown` で `Unresolved`）、Mutation 節に 1 行。あわせて 29 :259 の `ItemName` の条件に「`i64` に収まらない桁で `Unknown` になる行は当たらない」を足し、:318 との食い違いを無くした（起草役が指摘し、Coordinator が採用。受理の経路を増やさない明文化）。
  - Codex #2（P3、G-M9 の `現金 ￥0` が全角の通貨記号と半角の数字を混ぜる）: 採用。`wide("現金", 0)` と明記した。
  - Opus #1（P3、取引の明細域の `＃ N` を見る行が無い）・Opus #2（P3、練習の記録の幅の違反を見る行が無い）: 採用。G-O6 と G-K3 に case を足し、Mutation 節に行を足した。
  - Opus #3（P3、行の分類の表の「先頭の行」「最後の行」が番号印字の行を除くか書いていない）: 採用。29 の `CancelMark`・`ReportTitle`（Z / X）・`Cancelled` の条件に「番号印字の行を除いて数える」を足した。
  - Opus #4（P3、Owner Effort Budget の表が Codex の起動を数えている）: 採用。relay を数えない形に直した。
  - Final Review で独立に確かめる点: 一括是正の全体（G-F13・G-O6・G-K3・G-M9 の case と 29 の 5 箇所の句）。

- Closeout（2026-10-06）: Final Review broad（互いに独立の 2 本、head `23b21a97`）は Fable 5.1 = approve（P3 4）、Codex GPT-6.1 Sol（発注 214）= reject。Codex は PR に summary の review を 2 本投稿していて、Coordinator は 1 本目の件数だけを record と Gated Amendment 1 に写し、2 本目の P1（EJ-2）を落とした。Gated Amendment 1（`e430d45c`）と是正 `1cad2187`、EJ-2 を Coordinator が見つけて再現した後の Gated Amendment 2（`559f6ae1`）と是正 `b3a8a497`、main（PR #140 の merge）の取込み `595e27f0`。closure（Fable 5.1、`b3a8a497`）= approve（前回の findings はすべて closed）。Amendments が変わったので helper は前の broad を引き継がず、Fable 5.1 の broad を `b3a8a497` で取り直した = approve（P3 1）。専用 record に pass で記録し、前の broad・closure は evidence に添えた。AC11（Coordinator、実物の EJ）は是正の後も、是正の前と記録の行が完全に一致して PASS（件数は tracked に書かない）。取り直しの broad の P3 のうち、29 の `ItemName` の除外の文は本 closeout で実装（`item_token_overflows`）に合わせて直した。`Item` の形から末尾の `*` だけが外れた行の件は `docs/backlog.md` の保留へ送った。同じ head の 2 本目の review の取りこぼしは PR #142（D-107 (4) の `--pr-reviews`）で record の前に止まる。`docs/backlog.md` の「EJ parser に、店がレジで普段使う値引き・訂正・取消の文法を足す」は解消の書式にした。本 packet の `## Follow-up` の 1〜7 は後続の design lane「実測と POS 系列の対応」への入力として本 packet に残る。PR #141 は helper 経由の squash merge（2026-10-05）で閉じた。
