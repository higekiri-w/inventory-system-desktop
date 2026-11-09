# UI-13: 在庫整合性検証画面（REQ-904 / BIZ-07 / CMD-11）

## Workflow State

- Phase: archive
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: e5776ab
- Amendments: 7af126c（Amendment 1: integrity_cmd.rs への `#[specta::specta]` 属性 2 行を Scope に追加）, 40200a2（Amendment 2: design_compliance_test.rs への 75-ui doc 登録 1 entry を Scope に追加）, f0812b7（Amendment 3: 90-traceability.md の generator 再生成を Scope に追加）, 0b0fcfb（Amendment 4: navigation ui-13 有効化を Scope に追加）, 52b47b2（Amendment 4 補遺: Ledger に到達導線契約行を追加）, 6e6bf83（Amendment 5: operator 語彙統一を Scope に追加）
- Coordinator: Fable
- Writer: Codex（機能実装。発注 relay、owner がコピペ実行）/ Claude Sonnet subagent（visual polish pass、owner 指示 2026-07-15。非重複 ownership: `IntegrityCheckPage.tsx` の表示層に限定し、polish writer は Final Review の承認者にならない。実施タイミング: `implementing` 内で機能実装完了・local テスト green 後、independent-review **前**に一次実施。independent-review 後に visual finding が出た場合は通常規則どおり `implementing` へ戻して修正し再レビューする）
- Plan Reviewer: independent Sonnet review context
- Final Reviewer: independent Sonnet review context + Fable 裁定（P1/P2/P3）
- Reviewed Content HEAD: ac2b6b9a6bc2381df449bd620bae396dbce03a32
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required
- Human Gate: none（全接点消化済み・いずれも 2026-07-15: Plan Gate approval = 介入 1 回目、relay 上限延長 = 介入 2 回目、L3 + owner visual confirmation + Amendment 5 是正指示 = 介入 3 回目、Ready/merge approval = 介入 4 回目 / 予算 4 回）

Plan Gate record（append-only、plan-draft → plan-gate → plan-approved をこの commit で materialize）:

- plan-draft → plan-gate: independent Sonnet plan review rally。round 1（P1×2 / P2×1 / P3×2、全件 accept・反映: polish pass の Phase 整合、L3 差異注入の fault-injection 疑義は `create_product` の初期在庫 movement 記録を実証して確定、remount/restart テスト追加、Impact Review Lenses 文言）/ round 2（P2×1 / P3×1、反映: Budget 超過理由の記録、D-043 の Design Sources 追記）/ round 3（新規指摘なし、Plan Gate 可）
- plan-gate → plan-approved: owner 承認（2026-07-15、介入 1 回目 / 予算 4 回、「承認、実装発注へ」）
- ready-hosted-final → merge → archive（2026-07-15、Post-Merge Closeout）: PR #5 squash merge（`d5a2d42`）。hosted final は Ready 後 head 更新の `synchronize` run success + 三点 SHA 一致（run URL / headSha / cancellation 証跡は PR body）。D-043 dogfood（synchronize trigger + cancel-in-progress 実測）を消化。本 commit で packet / matrix を `docs/archive/plans/` へ移動、WER は同ディレクトリの `2026-07-15-ui13-integrity-check-workflow-effectiveness-review.md`
- STATECAP 是正（2026-07-15、Coordinator、ready-hosted-final 遷移前の L1 full で検出）: Amendment 5 再走行の materialize を state-only 単独 commit（旧 60062e7、4 件目の forward state-only遷移）にしていたため、workflow-git gate が forward STATECAP 超過（4 > 3 / post-implementation 3 > 2）で FAIL。DEV_WORKFLOW compression 規則どおり当該遷移を本承認記録 commit（content 隣接）へ相乗りに再構成。対象は Reviewed Content HEAD `ac2b6b9` より後の docs 遷移 commit 2 件のみで、audited content・評価済み証跡の SHA は全て不変。下記「Amendment 5 是正後の再走行」entry の「単一 state-only commit で materialize」は本 commit への相乗りに読み替える（entry 本文は append-only のため改変しない）
- human-confirm → ready-hosted-final（2026-07-15、owner 承認 = 介入 4 回目 / 予算 4 回、「承認で良いかな」）: Human Gate の Ready/merge 承認を消化。承認時に owner が「システム在庫/入出庫の合計」→「DB在庫/在庫」への簡略化を検討したが、実 operator 検証（「DB在庫」が意味を運ばなかった L3-3 実証）+ StocktakePage 既存列名「システム在庫」との画面間一貫性 + 2 数値比較画面での「在庫」単体の曖昧さを理由に現行語彙維持を実判断。この遷移は本承認記録 commit（content 隣接）に相乗りし forward STATECAP を消費しない
- implementing → local-verified → independent-review → human-confirm（2026-07-15、Amendment 5 是正後の再走行、単一 state-only commit で materialize）: ① local-verified 証跡 = content candidate `ac2b6b9` での L1 full PASS / CLEAN / MERGE_EVIDENCE_VALID=true、② independent-review 証跡 = targeted re-reviewer（新規独立 context）が語彙変更の範囲・ロジック不変・旧語彙残置 0 件・73-ui UI-10-D10 との論理整合・テスト強度維持を確認、新規指摘なし、③ 裁定 = Fable、P1/P2/P3 = 0
- human-confirm → implementing（2026-07-15、state-backtrack 2 回目、前回 backtrack とは複数の実作業 commit を挟み非隣接）: owner L3-3 の実 operator 検証で語彙 finding が確定し、owner 指示で本 PR 内是正（Amendment 5）。是正後、通常規則で forward を再走行する
- implementing → local-verified → independent-review → human-confirm（2026-07-15、backtrack 後の再走行、単一 state-only commit で materialize、全遷移の証跡は commit 前に存在）: ① local-verified 証跡 = content candidate `83c57f9` での L1 full PASS / CLEAN / MERGE_EVIDENCE_VALID=true（1 回目の L1 は traceability T1 drift で FAIL → Amendment 3 範囲の generator 再生成で解消）、② independent-review 証跡 = targeted re-reviewer（新規独立 context）が audited content `2299e85` に対し Amendment 4 範囲外混入なし・到達テスト実質性・route 実配線成立を確認、新規 P1/P2 = 0 / P3 = 1（75-ui §75.16 の旧除外記述残置）、③ 裁定 = Fable、P3 を accept し reviewer 処方どおり doc 1 行訂正（`3778b97`）+ 生成物再生成（`45663d8`、diff は REQ-904 行への navigation.test.ts 追記のみを機械確認）。audited 後の content 変更はこの 2 件のみで、いずれも処方済み訂正 / generator 出力
- human-confirm → implementing（2026-07-15、state-backtrack）: owner L3 で navigation 導線 disabled 残置（Goal Invariant 違反）が発覚。最早影響 phase = implementing へ単一 backward 遷移し、Amendment 4 の scope で是正後、通常規則で forward を再走行する。`Plan Commit` / `Reviewed Content HEAD` は保持（後者は再レビュー後に更新）
- implementing → local-verified → independent-review → human-confirm（2026-07-15、単一 state-only commit で materialize、全遷移の証跡は commit 前に存在）: ① local-verified 証跡 = content candidate `99502d0` での L1 full PASS / CLEAN / MERGE_EVIDENCE_VALID=true（evidence file は PR body 記載）、② independent-review 証跡 = 独立 Sonnet Final Reviewer による R3 Contract Audit（Ledger 全 10 行 pass、Spec Contract 4 項目充足、Scope 逸脱なし）、③ 裁定 = Fable、P1 = 0 / P2 = 0 / P3 = 1（integrity_cmd.rs 既存 tautological test、本 PR diff 外につき backlog）
- implementing 中の Amendment 3（2026-07-15）: L1 full が `90-traceability.md`（AUTO-GENERATED）の drift を検知し Codex が fail-closed 停止。REQ-904 coverage 更新の generator 再生成を許容。Amendment 1（specta 属性）/ 2（compliance test 登録）/ 3（traceability 再生成）は同一 failure class =「新規 command / 新設 doc / 新規 REQ coverage に付随する登録・生成義務の plan 段階での列挙漏れ」。L1 full の生成系検査 3 種（bindings / frontend routes / traceability）は本 Amendment で全て消化済みであることを確認。Coordinator 起因として relay 3 の往復内として扱う
- implementing 中の Amendment 2（2026-07-15）: `cargo test` の `design_compliance_test` が新設 75-ui doc の module 登録を要求し Codex が fail-closed 停止（670 pass / 1 fail）。plan 段階で「新設 function-design doc には compliance test 登録義務がある」ことを見落とした Scope 欠落であり、Coordinator 起因として relay 3 の往復内として扱う。WER の plan-quality lesson 対象（Amendment 1 の probe 盲点と合わせ、登録系義務の事前列挙が必要）
- implementing 中の Coordinator 側是正（2026-07-15）: Codex が品質 gate 実行中に PK4 ERROR（Plans.md 次の行動の active packet リンク欠落 = Coordinator 管轄）で fail-closed 停止。Plans.md 側で是正。この停止は Coordinator 起因の中間 blocker であり relay 3 の往復内として扱う（新規 relay を消費しない裁定、根拠: relay 往復 = 発注→完了報告の単位で、本件は発注 scope 外の Coordinator 所有物による中断）
- plan-approved → implementing: 2026-07-15 Coordinator による state-only 遷移。初回 Codex 発注は `Plan Commit: pending` のまま + 遷移責務を実装者に誤委譲していたため Codex が fail-closed 停止（正しい挙動、relay 1 往復消費）。Plan Commit = e5776ab（plan-first commit、実装 commit なしを ancestry で確認済み）を記入して是正

## Owner Effort Budget

- 介入回数上限: 4
- 実働時間上限: 45分
- relay 往復上限: 2

既定値（介入 ≤3 / 30分）からの超過理由: Codex 機能実装 + Claude visual polish の二重 writer 体制で owner 承認接点が 1 回分増える（Plan Gate / Codex relay / L3 + visual confirmation / Ready・merge の 4 接点）。operator 向け新画面のため L3 目視は省略しない。

調整記録（append-only）: 2026-07-15 relay 往復上限 2→3 に延長（owner 承認、介入 2 回目 / 予算 4 回）。理由: relay 1 = Coordinator の発注前提不備（Plan Commit 未記入）、relay 2 = Contract Probe の盲点による正当な fail-closed 停止（Amendment 1 で gate 済み）。いずれも実装内容の欠陥ではなく、Codex 作業ツリーの未コミット実装は継続利用可能。

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
operator workflow の新画面（在庫補正という不可逆な業務操作を含む）+ 新 route + generated bindings（`src/lib/bindings.ts`）再生成による Tauri command wire shape の変動。DB/BIZ/CMD の Rust 側変更はなし（CMD-11 integrity は実装済み）。

## Goal

Goal Invariant:

### 最小完了条件

- 利用者が画面から「整合性チェック実行」を押し、差異なし（緑の成功表示 + 直近確認日時）/ 差異あり（商品別の差異一覧）を日本語で確認できる。
- 差異あり時、利用者が補正対象の行を個別に選択し、確認ダイアログを経て「棚卸し補正として確定」でき、確定後に補正結果（補正件数・内訳）が画面に表示される。
- 実行中は画面内オーバーレイで他操作が抑止され、完了で自動的に結果表示へ遷移する。

### 失敗定義

- チェック実行はできるが補正 flow が最後（confirm → fix_integrity → 結果表示）まで完結しない。
- 全差異の一括補正が 1 操作でできてしまう、または確認ダイアログなしで補正が走る（誤操作防御の欠落）。
- bindings 再生成を回避した手書き `invoke` 呼び出しが frontend に入る。

### 非目的

- REQ-403 / SP-403 の POS 部門別売上照合（別 deferred task、ui-task-specs UI-13 節で明示除外）。
- 整合性チェックの自動実行・スケジュール実行・自動修正。
- backend（CMD/BIZ/DB）のロジック変更。許容するのは `lib.rs` の specta 登録漏れ是正 2 行 + `integrity_cmd.rs` の `#[specta::specta]` 属性 2 行（Amendment 1）+ `design_compliance_test.rs` の doc 登録 1 entry（Amendment 2）+ `90-traceability.md` の generator 再生成（Amendment 3）のみ。それ以外の不足が発覚した場合は Amendment で gate してから着手する。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。

## Scope

- `docs/function-design/75-ui-integrity-check.md` 新設（UI-13 の設計正本。UI-13-D1〜D8 を記録）+ `docs/FUNCTION_DESIGN.md` 目次の「UI-13 未記載」記述更新
- `src-tauri/src/lib.rs`: specta `collect_commands` へ `run_integrity_check` / `fix_integrity` の 2 行追加（登録漏れ是正、Contract Probe で発覚。ロジック変更なし）
- operator 語彙統一（Amendment 5、2026-07-15、owner L3-3 の実 operator 検証起点）: `IntegrityCheckPage.tsx` の列名「DB在庫」→「システム在庫」/「移動合計」→「入出庫の合計」、badge「DB在庫が多い」→「システム在庫が多い」/「移動合計が多い」→「入出庫の合計が多い」、対応するテスト期待文字列の同期、`75-ui-integrity-check.md` §75.6 と wording 記述の同期（UI-10 の「現在在庫」との使い分け理由を明記）。根拠: 実 operator（店主の母）が数字からは意味を構成できたがラベル語彙は理解できず、「移動合計」が最も伝わらなかった。「現在在庫」は本画面では「何の現在か」の曖昧さを生むため不採用（owner 判断）。ロジック・構造変更は禁止のまま
- `src/config/navigation.ts`: ui-13 entry の有効化（`to: "/settings/integrity"` / `status: "active"` の 2 field 変更、Amendment 4、2026-07-15。owner L3 で「サイドバー導線が disabled のまま = 利用者が画面に到達できない」Goal Invariant 違反として発覚。`docs/function-design/75-ui-integrity-check.md` §75.13 の navigation 除外記述の訂正、navigation.test.ts の同期が必要な場合はそれも含む）
- `docs/function-design/90-traceability.md`: canonical generator（`cargo run --bin generate_traceability`）による再生成のみ（Amendment 3、2026-07-15。REQ-904 の UI 設計書・frontend test 追加に伴う covered 集計更新。AUTO-GENERATED file につき手動編集は引き続き禁止、diff は REQ-904 関連に限る）
- `src-tauri/tests/design_compliance_test.rs`: `build_doc_to_modules_map()` へ `75-ui-integrity-check.md` → `cmd::integrity_cmd` / `biz::integrity_service` の登録 1 entry 追加（Amendment 2、2026-07-15。新設 function-design doc の compliance test 登録義務。`74-ui-operation-logs.md` の既存 entry（同ファイル :258）と同型の機械的追加。既存テストの変更・削除は禁止のまま）
- `src-tauri/src/cmd/integrity_cmd.rs`: 上記 2 command への `#[specta::specta]` 属性追加 2 行のみ（Amendment 1、2026-07-15。collect_commands 登録後の再生成で顕在化した属性欠落。他 cmd 群は全て `#[tauri::command]` + `#[specta::specta]` の対で定義済みであることを実コード比較で確認。関数本体の変更は引き続き禁止）
- `src/lib/bindings.ts` 再生成（`runIntegrityCheck` / `fixIntegrity` / `IntegrityFixResult` / `StockAdjustment` 追加）
- `src/routes/settings/integrity.tsx` 新 route
- `src/features/integrity-check/IntegrityCheckPage.tsx` + `IntegrityCheckPage.test.tsx`
- `docs/Plans.md` 進行状態 sync
- workflow dogfood: D-038（Findings Freeze / Owner Effort Budget 実測 / L3 Eligibility）、D-046（承認カウンタ interface / Goal H3 形式）、CI `synchronize` + `cancel-in-progress` の実動作確認（Ready 後 head 更新で初検証、PR #4 積み残し）

## Non-scope

- POS 部門別売上照合（REQ-403 / SP-403）
- Rust 側（CMD / BIZ / IO / DB schema / migration）のロジック変更（`lib.rs` specta 登録 2 行の是正を除く）
- 操作ログ画面（UI-11c）側の変更（直近確認日時の導出は既存 `list_logs` 呼び出しのみ）
- 一括補正 UI（select-all）、チェックのスケジュール実行

## Acceptance Criteria

- `npm test` で `IntegrityCheckPage.test.tsx` が green。状態遷移 idle → running → completed のテスト名に `REQ-904` を含む。
- 差異あり mock で「行選択 → 確定ボタン → 確認ダイアログ → confirm」で `fixIntegrity` が選択した `product_codes` のみを引数に 1 回発火するテストが存在する（evidence: テスト名 + mock assertion）。
- select-all 相当の操作が存在しないこと、未選択時に確定ボタンが disabled であることのテストが存在する。
- 差異一覧が 100 件/ページで client-side paging されるテストが存在する（101 件 mock で 2 ページ）。
- 差異なし時に成功表示と「直近の確認日時」（`list_logs` operation_type=`integrity_check` 由来）が表示されるテストが存在する。
- `run_integrity_check` / `fix_integrity` の CmdError 時に日本語 error 表示 + retry 導線が状態を破壊しないテストが存在する。
- `rg "invoke\(" src/features/integrity-check/` が 0 件（bindings 経由のみ、exit code 1）。再生成後の `rg "runIntegrityCheck|fixIntegrity" src/lib/bindings.ts` が command wrapper を含む。
- specta `collect_commands` と `invoke_handler` の登録差分が空（plan 段階の突合手順を PR で再実行、evidence: diff 出力）。
- hosted CI green + 三点 SHA 一致（live PR HEAD = local full evidence HEAD = hosted headSha）。
- Ready 後の head 更新 push で `synchronize` トリガー起動と旧 run の cancellation を PR body に証跡記録（workflow dogfood AC）。

## Design Sources

- Requirements / spec: `docs/SCREEN_DESIGN.md` 画面16（REQ-904）、`docs/architecture/ui-task-specs.md` UI-13 節
- Architecture: `docs/ARCHITECTURE.md`（UI → CMD 一方向）
- Function / command / DTO: `docs/function-design/42-cmd-sales-stocktake.md` §22.7（CMD-11 integrity）、`docs/function-design/36-biz-integrity-check.md` §21.3/21.4（INV-2/INV-4）
- DB: 変更なし（`inventory_movements` へは既存 BIZ 経由でのみ書込み）
- Screen / UI: `docs/architecture/ui-task-specs.md` UI-13 節 → 本 PR で `docs/function-design/75-ui-integrity-check.md` に詳細化
- Decision log / ADR: D-038 / D-039 / D-046（本タスクは dogfood target）、D-043（CI `synchronize` / cancellation。Ready 後 head 更新の実測対象）

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status |
|---|---|---|
| Backend function / command / repository / validation / error | 42-cmd §22.7 / 36-biz §21.3-21.4 | existing sufficient（実装・テスト済み、変更なし） |
| Command / DTO / generated binding / wire shape | `bindings.ts` 再生成（specta 生成物） | updated in this PR |
| DB / transaction / audit / rollback / migration | 変更なし | existing sufficient |
| Screen / UI / route state / Japanese wording | `function-design/75-ui-integrity-check.md` | updated in this PR（新設） |
| CSV / TSV / report / import / export format | 該当なし | — |
| Durable decision / ADR | UI-13-D1〜D8 は 75-ui doc に記録 | updated in this PR |

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| REQ-904 | ui-task-specs UI-13 | UI-13-D1 | route は `/settings/integrity`（保守系画面として UI-11c logs と同区画）。URL search state なし: チェック結果は実行セッション限りの ephemeral で、リロード後は spec 通り idle に戻る。rejected: URL に page/結果を持つ（再現不能な状態を URL 化する矛盾） | `src/routes/settings/integrity.tsx` | route 存在 + 初期 idle テスト |
| REQ-904 | ui-task-specs UI-13 flow 3 | UI-13-D2 | 直近確認日時は operation_logs から導出（`list_logs` type=`integrity_check` 最新1件）。rejected: 専用 settings key 追加（backend 変更が必要でスコープ超過、log と二重管理） | `IntegrityCheckPage.tsx` | 直近確認日時表示テスト |
| REQ-904 | ui-task-specs UI-13 flow 5-7 / 制御構造 | UI-13-D3 | 行ごと「補正する」checkbox + select-all なし + 確認ダイアログに選択行の内訳（product_code / stock_quantity → movements_sum）列挙。spec「全差異を一括補正させない。1件ずつ確認させる」は「盲目的一括補正の防止」が意図であり、明示列挙付き複数選択で満たす。rejected: 1行ずつ modal 連鎖（操作者の反復負担で確認が形骸化） | `IntegrityCheckPage.tsx` | 選択→confirm→`fixIntegrity` 引数テスト、select-all 不在テスト |
| REQ-904 | ui-task-specs UI-13 制御構造 | UI-13-D4 | running 中は画面内オーバーレイ + Progress。mutation pending 中は実行/確定ボタン無効（二重実行防止） | `IntegrityCheckPage.tsx` | running 中操作抑止テスト |
| REQ-904 | ui-task-specs UI-13 flow 7 | UI-13-D5 | fix 成功後は自動再チェックしない（spec が「重い処理」と明記）。`IntegrityFixResult` の summary（fixed_count / adjustments）を表示し、補正済み行に badge、再確認は手動の「再度チェック」導線。rejected: 自動再実行（重い処理の暗黙起動） | `IntegrityCheckPage.tsx` | fix 後表示テスト |
| REQ-904 | 36-biz §21.4（部分失敗時 skipped） | UI-13-D6 | `skipped_count > 0` を成功 message に埋没させず警告表示（非色依存の文言 + badge）。error 時は日本語 message + retry で選択状態を保持 | `IntegrityCheckPage.tsx` | skipped 警告テスト、error/retry テスト |
| REQ-904 | 42-cmd §22.7 | UI-13-D7 | frontend からの呼び出しは再生成 bindings 経由のみ。手書き `invoke` 禁止 | `bindings.ts` 再生成 | `rg "invoke\(" src/features/integrity-check/` 0 件 |
| REQ-904 | inventory-operator-ui 制約 | UI-13-D8 | 差異/補正済み等の状態は色 + 文言で表現（color-only 禁止）、非 IT 高齢 operator 向け日本語文言、visual polish pass は Claude 陣営が実施 | `IntegrityCheckPage.tsx` | 状態文言存在テスト + L3 目視 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: 75-ui doc 新設後に成立（UI-13-D1〜D8 を本 PR で source doc 化）
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: UI-13-D1〜D8 → `function-design/75-ui-integrity-check.md`
- Assumptions and constraints: CMD-11 の Rust 実装・テストは完備で変更不要（36-biz / 42-cmd と `integrity_service.rs` の突合済み）
- Deferred design gaps, risk, and follow-up target: POS 照合（REQ-403）は将来設計として ui-task-specs に記載済み、本 PR で扱わない
- Test Design Matrix can cite design decision IDs or source doc sections: UI-13-D1〜D8 を cite

## Impact Review Lenses

not applicable — 本タスクは field investigation / 実機発見 / 外部ツール挙動 / POS 連携 / フォーマット変更のいずれも起点とせず、既存設計（ui-task-specs UI-13）からの計画的実装のため本節のトリガー条件に該当しない。操作者の誤操作防御・非色状態表示は Design Intent Trace の UI-13-D3 / UI-13-D8 で扱う。

## Design Readiness

- Existing design docs are sufficient because: 状態機械・flow・制御構造は ui-task-specs UI-13 節に、CMD/BIZ contract は 42-cmd §22.7 / 36-biz §21.3-21.4 に既存。UI 詳細（route、文言、選択 UX）のみ不足
- Source docs updated in this PR: `docs/function-design/75-ui-integrity-check.md` 新設、`docs/FUNCTION_DESIGN.md` 目次更新
- Design gaps intentionally deferred: POS 照合（REQ-403）
- Durable decisions discovered in this plan and promoted to source docs: UI-13-D1〜D8

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): UI 層のみ追加、CMD 以下変更なし
- Backend function design: 変更なし（36-biz / 42-cmd 既存）
- Command / DTO / data contract: `IntegrityResult` / `IntegrityMismatch` / `IntegrityFixResult` / `StockAdjustment`（specta 生成、snake_case field）
- Persistence / transaction / audit impact: fix は既存 BIZ の TX + operation_log 記録に委譲、frontend 起点の新規書込みなし
- Operator workflow / Japanese UI wording: UI-13-D3/D8、75-ui doc に文言表を置く
- Error, empty, retry, and recovery behavior: UI-13-D6（error/retry/skipped 警告）、差異 0 件 = 成功表示（empty とは区別しない: 差異なしが正常）
- Testability and traceability IDs: テスト名に REQ-904、Matrix が UI-13-D* を cite

## Contract Probe

- 前提: bindings 再生成経路（`cargo run --bin generate_bindings`）が integrity commands を `bindings.ts` に出力する: probe（2026-07-15 plan 段階で実行）-> **前提否定**。再生成は成功するが diff ゼロ。原因は `lib.rs` の specta `collect_commands` に integrity 2 command が未登録（`invoke_handler` のみ登録）。両リストの機械突合で他の登録漏れなし（specta 55 / handler 57、差分は integrity 2 件のみ）。是正（`collect_commands` へ 2 行追加 + 再生成）を Scope に反映済み
- probe 限界の追記（2026-07-15、Amendment 1 起因）: 上記 probe は「collect_commands 未登録状態」で再生成を回したため、登録後に初めて顕在化する `#[specta::specta]` 属性欠落（`integrity_cmd.rs`）は検出できなかった。実装フェーズで Codex の generate_bindings 失敗により発覚（fail-closed 停止、正しい挙動）。lesson: 登録漏れ是正を含む probe は「是正を仮適用した状態」で end-to-end に回すこと

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| UI-13-D1（route / ephemeral state） | `src/routes/settings/integrity.tsx` | route 初期 idle テスト | — |
| UI-13-D2（直近確認日時導出） | `IntegrityCheckPage.tsx` | 直近確認日時表示テスト | — |
| UI-13-D3（個別選択 + 確認ダイアログ） | `IntegrityCheckPage.tsx` | 選択→confirm→引数一致、select-all 不在、未選択 disabled | non-scope for L3: 差異注入は DB 直接操作（fault-injection 級）を要するため L3 対象外。component test + 既存 BIZ 統合テスト（`test_fix_integrity_*`）で担保 |
| UI-13-D4（running overlay / 二重実行防止） | `IntegrityCheckPage.tsx` | running 中操作抑止テスト | L3: 実機での overlay 表示 |
| UI-13-D5（fix 後の手動再チェック導線） | `IntegrityCheckPage.tsx` | fix 後 summary + badge テスト | — |
| UI-13-D6（error / retry / skipped 警告） | `IntegrityCheckPage.tsx` | CmdError 表示、retry 状態保持、skipped>0 警告 | — |
| UI-13-D7（bindings 経由のみ + specta 登録是正） | `src-tauri/src/lib.rs` + `src-tauri/src/cmd/integrity_cmd.rs`（属性 2 行、Amendment 1）+ `src/lib/bindings.ts` | `rg "invoke\("` 0 件検査 + 登録差分空の突合 + generate_bindings 成功 | — |
| UI-13-D8（非色状態表示 / operator 文言） | `IntegrityCheckPage.tsx` | 状態文言存在テスト | L3: owner visual confirmation |
| spec: 差異一覧 100 件/ページ paging | `IntegrityCheckPage.tsx` | 101 件 mock 2 ページテスト | — |
| spec: 差異表示列（code/名前/DB値/SUM/差異） | `IntegrityCheckPage.tsx` | 一覧列表示テスト | non-scope for L3（同上の理由。可読性は polish pass + owner visual confirmation を差異なし画面と component テスト描画で実施） |
| spec: operator 到達導線（navigation ui-13 active、Amendment 4 で追加。route 直 render テストは到達性を検証しない盲点の是正） | `src/config/navigation.ts` | `test_navigation_req904_ui13_active_at_settings_integrity` | L3: サイドバーから実際に画面へ遷移できること |

## Test Plan

Test Design Matrix: [test-matrices/2026-07-15-ui13-integrity-check.md](test-matrices/2026-07-15-ui13-integrity-check.md)

- targeted tests: 状態遷移（idle/running/completed）、選択→補正 flow、paging、直近確認日時
- negative tests: CmdError（run/fix 双方）、skipped_count>0、未選択 confirm 不可、巨大差異件数
- compatibility checks: bindings 再生成 diff が integrity 追加のみで既存 command 型を壊さない（PR diff レビュー）
- data safety checks: 実 DB を CI テストで使わない（mock のみ）、補正実行は L3 で synthetic データ
- main wiring/integration checks: route 登録、`npm run build` green、hosted CI 三点一致

## Boundary / Wire Contract

- producer: `cmd::integrity_cmd`（Rust, specta 生成）
- consumer: `IntegrityCheckPage.tsx`（`commands.runIntegrityCheck` / `commands.fixIntegrity`）
- wire type: `IntegrityResult { mismatches, mismatch_count, checked_count }` / `IntegrityMismatch { product_code, name, stock_quantity, movements_sum, difference }` / `fixIntegrity(productCodes: string[]) -> IntegrityFixResult { fixed_count, skipped_count, adjustments }`
- internal type: bindings 生成型をそのまま使用（frontend 側の再定義禁止）
- precision/range: 在庫数は i64 → TS number（本業務の在庫数量域では安全、既存画面と同判断）
- round-trip path: なし（読取 + 補正 command のみ、frontend からの型往復なし）
- invalid input: 空 `product_codes` は UI で防止（未選択 disabled）+ BIZ 側 validation に委譲
- compatibility: bindings 再生成は追加のみであること（PR diff で既存型の変更なしを確認）

## Review Focus

- 補正 flow の誤操作防御（UI-13-D3: select-all 不在、確認ダイアログの内訳列挙、未選択 disabled）
- fix_integrity 呼び出し引数が「選択した行のみ」であること
- running 中の操作抑止と二重実行防止（UI-13-D4）
- skipped_count / error の非埋没表示（UI-13-D6）
- bindings 再生成 diff の妥当性（追加のみか）

## Spec Contract

Contract ID: SPEC-UI13-INTEGRITY

- 整合性チェックは利用者の明示操作でのみ実行され、実行中は他操作を受け付けない
- 補正は行単位の明示選択 + 確認ダイアログを経由した場合のみ `fix_integrity` に到達し、引数は選択行の product_code 集合と一致する
- 補正結果（fixed / skipped）は利用者に日本語で可視化され、skipped は成功に埋没しない
- frontend は CMD-11 を bindings 経由でのみ呼び出す

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| REQ-904 / SPEC-UI13-INTEGRITY | function-design 75 新設 → 実装 → テスト | Matrix 記載の vitest 群（テスト名に REQ-904） | 誤操作防御・引数一致 | `npm test` green + hosted CI 三点一致 |
| BIZ-07 INV-2/INV-4（既存） | 変更なし（消費のみ） | 既存 Rust テスト（`integrity_service.rs`） | wire 型の消費が契約通りか | 既存 `cargo test` green |
| D-046 承認カウンタ dogfood | 全 owner 承認接点 | — | 承認依頼フォーマット遵守 | PR body / packet 記録 |
| D-043 synchronize/cancellation | Ready 後 head 更新で実測 | — | CI run の trigger/cancel 証跡 | PR body に run ID 記録 |

## Data Safety

- 実店舗 DB・実データを commit / テスト固定資産に含めない（mock / synthetic のみ）
- L3 は synthetic データ（アプリ内正規操作で構築）のみで行い、実店舗 DB での補正実行はしない。差異あり flow の実機確認は L3 対象外（Matrix「L3 対象外の明示」参照）
- スクリーンショット等の証跡に実商品名・実在庫数を含めない

## Implementation Results

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

- PR: https://github.com/higekiri-w/inventory-system-public/pull/5（Draft）
- Codex 機能実装: 設計 doc（75-ui 新設 + FUNCTION_DESIGN 目次）、specta 登録是正（Amendment 1）、compliance test 登録（Amendment 2）、traceability 再生成（Amendment 3、REQ-904 covered 化）、route + IntegrityCheckPage + Matrix 全テスト。品質 gate 全 green、L1 full CLEAN
- Claude 陣営 visual polish pass（independent-review 前、表示層のみ）: 補正確認ダイアログを不可逆操作 warning Alert パターン（StocktakeCompleteDialog / BackupRestorePage と同構成）に整合、内訳列挙の再構成、見出しサイズ統一
- Amendment 1〜3 は同一 failure class（登録・生成義務の plan 列挙漏れ）。WER で lesson 化予定

## Review Response

- R3 review-only: 実施（独立 Sonnet Final Reviewer、R3 Contract Audit。Ledger 全 10 行を実コード・実テストと突合し全行 pass、空疎 assertion 検査込み。Spec Contract 4 項目充足、wire 型消費一致、75-ui doc 双方向 drift なし、Scope 逸脱なし）
- 裁定（Fable）: P1 = 0 / P2 = 0 / P3 = 1
- P3-1（backlog、本 PR 対象外）: `src-tauri/src/cmd/integrity_cmd.rs` の `test_fix_integrity_req904_empty_codes_validation` が `fix_integrity` 本体を呼ばず検証ロジックをテスト内で再実装している tautological test。本 PR の diff（specta 属性 2 行）に含まれず Amendment 許容範囲外のため修正しない。次に同ファイルへ手を入れる PR で `super::fix_integrity` 実呼び化する
- visual polish 由来の follow-up 候補（owner visual confirmation で判断材料に）: 差異一覧の「DB在庫」列名・「DB在庫が多い」文言は operator に IT 略語を露出している。変更には 75-ui doc（§75.6 正本）+ テスト同期が必要なため polish では見送り
- Amendment 5 の実 operator 事後検証（2026-07-15、owner の新文言確認中に届いた遅延返信）: 旧語彙「移動合計」への追加証言（「実在庫なのか売れた数なのか分からない」= 旧語彙の不通を再確認）と、新語彙「入出庫」への正しい読み（「仕入れと売れた数だよね」= 意図したメンタルモデルと一致）を同一 operator から取得。Amendment 5 の語彙選択が実 operator で検証されたため追加変更なしと裁定（Fable）
- owner L3 実施結果（2026-07-15）: L3-0（サイドバー到達）pass / L3-1（差異なし実行 + 直近確認日時）pass / L3-2（running overlay、「在庫データを確認しています」を実機で目視、連打で二重実行なし）pass / L3-3 = 可読性 pass + 文言 finding（下記）。補足: owner 判断で synthetic DB に SQLite 直接操作で差異 2 件を仕込み、差異一覧・badge・補正 flow を非公式に目視（L3 証跡外の文言判断材料。実 operator = 店主の母による検証で「ラベル語彙（DB在庫/移動合計）は意味を運ばず、差異の +/- 数字から意味を構成した」「『チェックデジット不正』への疑問は synthetic テストデータの商品名であり画面文言 finding ではない」を確認）
- post-freeze exception 2 件目（Amendment 5）: 実 operator 検証による語彙 finding。owner 指示により本 PR 内で是正（follow-up 切り出しより merge 時点完成を優先）
- targeted re-review（Amendment 5 是正後、新規独立 context）: 語彙変更の範囲一致・differenceLabel 分岐/列順/className/a11y 不変・旧語彙残置 0 件・UI-10-D10「現在在庫」との使い分けは異なるデータモデル/改名理由で論理矛盾なし・assertion 強度維持。新規 P1/P2/P3 = 0
- targeted re-review（Amendment 4 是正後、新規独立 context）: Amendment 4 範囲外の混入なし、navigation 型適合、到達テストは ui-11c 前例と同強度で tautological でない、route 実配線成立。新規 P1/P2 = 0 / P3 = 1（75-ui §75.16 旧記述残置 → 訂正済み）
- Findings Freeze: frozen after Broad Audit（2026-07-15、independent Final Review 完了時点）; post-freeze exceptions: 1 件 — owner L3 で navigation 導線の disabled 残置（Goal Invariant「利用者が画面から実行できる」違反）を発見（2026-07-15）。freeze 後例外として許容する理由: owner の実機確認は freeze 対象外の検証面であり、かつ Goal Invariant 直撃のため defer 不可。Amendment 4 で gate し implementing へ backtrack して是正。**なお本 finding は plan 段階の登録義務列挙漏れ 4 件目（specta 属性 / compliance test / traceability / navigation）であり、Ledger にも「operator が画面に到達できる」導線契約の行が欠けていた（review が実装と doc の整合のみ検証し到達性を検証しない盲点）。WER の中心 lesson とする
