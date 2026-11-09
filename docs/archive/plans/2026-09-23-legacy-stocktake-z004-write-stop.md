# Plan Packet: ㉘ runtime ① 旧棚卸しとZ004業務commit・取消の停止と再現fixture（R3）

2026-09-23 起草。出典は `docs/Plans.md`「次の行動」1（owner 決定 2026-09-22: ㉘ runtime の最初の lane = 既存の危険な操作〈旧棚卸しの開始・入力・確定、POS の業務 commit・取消〉の停止と再現 fixture）と、[㉗ packet](2026-09-16-stocktake-count-baseline.md) の節「後続 runtime lane ㉘ への申し送り」、[時点証拠 ADR](../../adr/2026-09-18-stocktake-time-evidence.md)（[適用範囲の但し書き](../../adr/2026-09-18-stocktake-time-evidence.md#適用範囲の但し書き)を含む）、[監査 STK-1 / STK-2](../../research/2026-09-16-diagram-audit.md)。lane の切り方（① 停止と再現 fixture → ② 受領・判定・保存の基盤 → ③ 計数と補正 → ④ 取込み・取消・回復 → ⑤ 一括切替）は 2026-09-22 の外部設計相談（owner 実施、回答は local-only）を Coordinator が採用したもので、本 packet は ① だけを扱う。本 lane は D-090 の dogfood 対象であり、`Ordinary Operation` 節を置く。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

- Evidence Mode: github
- Phase: archive
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: b569ca243964d1ca76b493ffe3dc02261064548c
- Amendments: none
- Coordinator: Opus 5.5（Claude Code main session、effort high）
- Writer: Opus 5.5 subagent（worktree run、effort medium）
- Plan Reviewer: Opus 5.5（fork でない fresh context、effort medium。Codex は rate limit 中のため Opus のみ）
- Final Reviewer: Opus 5.5 + Codex（互いに独立、Writer・Plan Reviewer とも別の fresh context。Final Review は Codex の復帰を待つ）
- Final Review Minimum: 2
- Human Gate: ready,merge,manual

座組の根拠: owner 決定 2026-09-23（Opus = Opus 5.5 は Writer / Coordinator / レビューに全面解禁、Sonnet 5 / Opus 5 は座組から退役、Codex 停止中は Final Review だけ待つ）。現行の tracked 文書とは衝突する: `docs/AGENT_OPERATING_MANUAL.md` §3 の高自律・低制約適性 slot 項（D-056）は「Opus」slot を read-only の Reviewer / Explorer 専任とし、§3.4 の対応表は「Opus」を Claude Opus 5 に対応させている。owner は D-056 の制限を Opus 5 の性格から決めたものとし Opus 5.5 へ引き継がないと決め、規則文の改訂は並走する docs 復元 + 規則改訂 lane が担う。本 packet はその改訂の merge を待たず owner 決定を根拠に Opus 5.5 を Writer に置く（§3 冒頭の独立性制約〈Writer ≠ Plan Reviewer ≠ Final Reviewer、自己承認禁止〉は維持）。Execution Mode は現行 enum の定義（希少・最高能力 slot = Fable 5.1 が利用可能な期間）で `fable-window` とする。Fable 5.1 は本 lane の役割に割り当てず、難所の相談役として必要時に呼ぶ。

Plan Reviewer と Writer は同じ model（Opus 5.5）で、`docs/DEV_WORKFLOW.md` Review Rules の vendor 条項（Writer が Codex の場合）は literal に掛からない。PR #88 の所見（計画と実装を同じ vendor が見ると packet の前提誤りが Plan Gate を通り得る）の残余 risk は、Final Review の Codex（別 vendor）で受ける。

manual の対象: 棚卸し画面と売上データ取込み画面（Z004 タブ）に停止の案内と入口の無効化が加わる operator 画面の変更（`docs/DEV_WORKFLOW.md` Human Visual Confirmation For Screen Changes）。

遷移記録（append-only）:

- kickoff → spec-check → design → plan-draft → plan-gate（本 commit、plan-first）: Risk R3（`docs/project-profile.md` High-risk Changes の stocktake / CSV import / rollback の BIZ 振舞い、daily workflow screens に該当。R4 の破壊的な data lifecycle には当たらない = DB の行・schema を変えず、停止は build の差し替えで戻せる）。Design Phase = 停止の契約を本 packet の Spec Contract（SPEC-STOP-D1〜D6）に置き、実装と同じ PR で新設 ADR と Scope S6 の source docs へ昇格する（Writer が実装 run で同期）。owner の設計判断を要する未決の論点なし（停止の対象は owner 決定 2026-09-22 で確定済み、画面文言は manual で確認する）。
- plan-gate → plan-approved → implementing（2026-09-24、Coordinator、state-only）: Plan Review round 1（fresh Opus、P1 0 / P2 3 / P3 8、操作列は `成立`）→ 全件採用し是正 `94528ef8` → round 2 closure（別の fresh Opus、P1/P2 = 0、P3 5、操作列は `成立`）→ P3 を反映。Plan Commit = `b569ca24`（plan-first `d2acc6e9` → 是正を含む確定版）。実装は Opus 5.5 subagent の worktree run。
- implementing → archive（closeout、本 commit）: PR #95 squash merge `33b0dbbb`（2026-09-25）。packet / Matrix を `docs/archive/plans/` へ移送し、Implementation Results / Review Response を記録した。Non-scope に置いた `docs/backlog.md` の STK-1 / STK-2 entry の現在地（① の停止の完了）を更新した。Residual（CMD の token 削除 3 契約の到達不能、開発 DB の旧進行中棚卸し、ホームの Z004 未取込み警告）と ② 〜 ⑤ の lane は、停止 ADR の Consequences / Revisit Trigger と時点証拠 ADR の「㉘ への引継ぎ」節が正本のため、backlog には重ねない。

## Owner Effort Budget

- 介入回数上限: 5（内訳の見込み: push と Draft PR 作成の承認 1、Codex の Final Review relay 1、manual の目視確認 1、Ready・merge の判断 1。既定 3 から 1 増やす理由 = 画面変更の manual が加わるため。2026-09-25 に owner 承認で 5 へ引き上げ「全部承認、承認して大丈夫って状態になってるならね」: main の取込みで `docs/Plans.md` の競合を解くため、`docs/agent-guidance/merge-evidence.md` の base 同期の規則により manual の PASS を引き継げず、目視確認をもう 1 回行う。同じ承認で Ready・merge も承認済み〈ただし同期後の closure・CI・manual が揃ってから〉。消費 5 = push と Draft PR 作成 1、relay 1、manual 1、Ready・merge と引き上げの承認 1、同期後の manual 1）
- 実働時間上限: 30分（既定。manual は 2 画面の目視で 10 分程度の見込み、未実測）
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
影響で判定した（`docs/DEV_WORKFLOW.md` Risk Tiers、`docs/project-profile.md` High-risk Changes）。棚卸し（BIZ-06）と Z004 取込み・取消（BIZ-03）の BIZ 振舞いを「拒否」に変え、在庫・売上・棚卸しの write 経路を止める。棚卸し画面と売上データ取込み画面の operator の操作が変わる。command の引数・戻り型・`CmdErrorKind`・登録・`src/lib/bindings.ts` は変えない（SPEC-STOP-D2）。merge 可否に効く test / workflow gate は変えない。R4 に当たらない: DB の行・schema・migration を変えず、既存の進行中棚卸しや取込み記録を書き換えない。停止の解除は ⑤ の build で行う。

## Goal

Goal Invariant: 現行実装のまま使うと在庫・売上を誤らせる入口（旧棚卸しの開始・カウント保存・確定、Z004 の業務 commit・取消）が、画面からも command からも業務 write を 1 行も起こさずに止まる。止まったことと理由が画面で分かり、閲覧と影響外の既存操作は従来どおり使える。後続 lane が同じ入力を再利用できる合成 fixture が残る。

### 最小完了条件

- 5 つの入口（BIZ の公開関数 `start_stocktake` / `update_count` / `complete_stocktake` / `commit_csv_import` / `rollback_csv_import` と、それを呼ぶ同名の Tauri command）が、どの DB 状態でも固定文言の error を返し、DB 全 table の内容（`operation_logs` を含む）を変えない。
- 棚卸し画面は停止の案内を出し、開始・カウント入力・確定を操作できない。進捗・一覧・絞り込み・前回の棚卸し・記録詳細は見られる。
- 売上データ取込み画面の Z004 タブは停止の案内を出し、「取り込む」を押せない。ファイル選択とプレビューはできる。日報タブは変わらない。
- 入庫・手動販売・返品・廃棄・日報取込みと取消は従来どおり動く。
- 再現 fixture F1〜F7（Spec Contract SPEC-STOP-D5）が test code に置かれ、① の停止検査に使われている。既存の `#[ignore]` 診断 2 本は旧処理の本体を呼んで引き続き不具合を再現する。

### 失敗定義

- 停止した入口のどれかが、条件付きで（force_fill なし、同日 import なし等）旧処理を通す。「この条件なら安全」という例外を足す。
- 停止中の呼出しが `operation_logs` を含む何かを書く、または preview cache を消す。
- 旧処理を部分修正する（例: 確定だけ `現在庫 + N − system_stock` に直す、`pos_stock_sync` を自動で off にして売上だけ通す）。
- 既存 test を削除・skip・弱体化する。停止で到達不能になった契約の test を、移行先を記録せずに消す。
- 停止を画面の非表示だけで実現し、command から旧処理が動く。
- `CmdErrorKind`・command の型・`bindings.ts` が変わる。

### 非目的

- 新しい計数・受領・判定・取消の実装（② 〜 ④）、migration・新 command・新 UI への切替と停止の解除（⑤）。
- 時刻による前後判定・OS 監視・legacy 専用復旧・判定不能の扱い（ADR 修正 lane と次の design lane の範囲）。
- 開発・demo DB の既存行の書換え・削除（進行中の旧棚卸しは閲覧だけ可能なまま残す）。
- ホーム画面の「前日分が未取込みです」警告の変更（Non-scope に理由）。
- 日次の自動在庫連動（v1.0 の恒常運用）の達成。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

店の事実は `.local/reports/store-premises/answer-ledger.md` を正とした（L-076: 棚卸しは今 owner が渡した Excel へ机の PC と棚を往復して入力 / L-113: 日報は Excel へ貼って印刷 / L-237: D-070 維持、店は間に合わせの Excel で大まかな在庫管理を続け、v1.0 は自動在庫連動の完成まで先行導入しない / L-240: 実店舗の本番 DB と本番取込み履歴は無い）。したがって本 lane の停止が効くのは開発・demo・試験 DB だけで、店の普通の一日は変わらない。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 店: アプリ未導入、棚卸しは Excel、売上は日報を Excel へ貼って印刷（L-076 / L-113 / L-237） | いつもどおり営業・精算・Excel 入力 | 本 lane の影響なし | v1.0（⑤ + 次の design lane + 実機確認）まで Excel を継続 | なし（台帳の現行行） |
| 開発・demo DB、進行中の棚卸しなし | 棚卸し画面を開く | 停止の案内（見出し + 理由 + 見られるもの）。「棚卸しを開始する」は押せない。前回の棚卸しの要約は見られる | ⑤ で停止を解除 | なし |
| 開発 DB に旧方式の進行中棚卸しがある | 棚卸し画面を開く | 停止の案内。進捗・一覧・部門絞り込み・未入力のみ・ページ送りは使える。カウント入力欄と「棚卸しを確定する」は操作できない。旧明細は書き換わらない。ただし商品の新規登録・商品一括 import は停止対象外で、進行中の旧棚卸しへ未計数の明細を追加し続ける（`product_service.rs:261-268` / `:1346-1352`。STK-1 / STK-2 は起こさないが、⑤ の移行対象の明細が増える） | ⑤ の migration が旧明細を legacy 等へ分類する（ADR D8、ADR 修正 lane の版に従う） | 開発 DB に進行中棚卸しがあるかは未確認（M1 は画面の表示で分岐、両分岐とも AC5 の自動 test で固定） |
| 売上データ取込み画面（既定の日報タブ） | 日報 3 ファイルを選び、取り込む・取り消す | 従来どおり。在庫は変わらない | なし | なし |
| 同画面の Z004 タブ | 合成 Z004 を選ぶ | 停止の案内。プレビュー（正常行・スキップ行・警告）は表示され、「取り込む」は押せない。「ファイルを選び直す」は使える | ⑤ で停止を解除 | なし |
| 入庫・手動販売・返品・廃棄を記録する | 各画面で保存 | 従来どおり在庫が動く。棚卸し確定が無いので STK-1（確定がカウント後の増減を打ち消す）は起きない。Z004 commit が無いので STK-2（後着売上の二重減算）も起きない | なし | なし |
| 古い画面を開いたまま・直接 command 呼出し（devtools 等） | 停止した command を呼ぶ | backend が固定文言の error を返し、DB は変わらない。画面は既存の error 表示で文言を出す | なし | なし |
| アプリ再起動・翌日 | 同じ操作 | 停止は build に固定されており、状態を持たないため同じ結果 | ⑤ の build | なし |
| 開発 DB に過去の Z004 取込みがある | ホームを開く | 「前日分が未取込みです」（destructive）が出続けるが、Z004 は取り込めない | 受容（本番 DB は Z004 取込み 0 件で開始し、`needsImportWarning` は取込み 0 件では出ない = `53-ui-home.md` の派生値表） | なし |

この packet を完了できること（誤った結果を出す入口が残らない）と、通常運用を達成できること（日次の自動在庫連動）は別である。後者は ② 〜 ⑤、次の design lane「実測と POS 系列の対応を取得・保存する」、実機確認を経るまで未達のまま。

## Scope

本 packet と Matrix の `D1`〜`D6` は子 ID `SPEC-STOP-D1`〜`SPEC-STOP-D6` の略記（`docs/decision-log.md` の D-n とは別）。

- S1 BIZ の停止（SPEC-STOP-D1〜D3）:
  - `src-tauri/src/biz/stocktake_service.rs`: 公開関数 `update_count`（:260）、`start_stocktake`（:312）、`complete_stocktake`（:400）の先頭で、引数・DB に触れる前に `BizError::ValidationFailed(<BIZ-06 停止文言>)` を返す。現行の本体は `pub(crate)` の `legacy_update_count` / `legacy_start_stocktake` / `legacy_complete_stocktake` として残し、production の呼出し元を持たせない（非 test build の dead_code は旧本体の root に `#[cfg_attr(not(test), allow(dead_code))]` を置いて抑える。Contract Probe 参照）。同 file 内の既存 test は旧本体へ呼出し先を付け替え、assert は変えない。
  - `src-tauri/src/biz/csv_import_service/commit.rs`: `commit_csv_import`（:21）の先頭で `BizError::ImportError(<BIZ-03 停止文言>)` を返す（`csv_import_failed` の operation log も書かない）。本体は `legacy_commit_csv_import`（`pub(crate)`）へ。
  - `src-tauri/src/biz/csv_import_service/rollback.rs`: `rollback_csv_import`（:15）も同様（冪等の「rolled_back なら Ok」も含め停止）。本体は `legacy_rollback_csv_import`。
  - `src-tauri/src/biz/csv_import_service/mod.rs`: 旧本体を test module から呼ぶための `pub(crate) use` が要る場合だけ変更する。公開 `pub use`（:19 / :25）の名前は変えない。
- S2 既存 BIZ test の付け替え: `src-tauri/src/biz/csv_import_service/tests/commit_tests.rs`、`rollback_tests.rs`、`cross_feature_tests.rs`（World の `commit_bytes` :214 / `rollback` :237、Temporal の :440 / :454 / :484）の呼出し先を旧本体へ変える。assert・入力・`#[ignore]` 属性（:581 / :608）は変えない。
- S3 再現 fixture と停止検査（SPEC-STOP-D5）: 新設 `src-tauri/src/biz/csv_import_service/tests/legacy_stop_tests.rs` と `tests/mod.rs` への登録。F1〜F7 の builder（合成データのみ）と、各 fixture 状態で 5 つの BIZ 入口を呼び、停止 error と DB 全 table の不変を確認する test。共通 helper が要る場合は `test_support.rs`（`pub(super)`）へ置いてよい。
- S4 CMD の検査（CMD の本体は変えない）:
  - `src-tauri/src/cmd/stocktake_cmd.rs` の test module: `test_update_count_req205_negative_validation` / `test_update_count_req205_zero_is_valid` の期待値を停止へ改める（負数・0 とも停止 error、DB 不変。`docs/function-design/42-cmd-sales-stocktake.md` §22.10 の方式〈production command を呼び、source design から独立転記した kind / message / field を完全一致比較〉は維持）。`start_stocktake` / `complete_stocktake` の停止を同じ方式で 1 test 追加する。
  - `src-tauri/src/cmd/csv_import_cmd.rs` の test module: `test_csv_cmd_req401_snapshot_mismatch_deletes_preview_token`（:396）は停止により mismatch 分岐へ到達できなくなるため、有効な preview token で `commit_csv_import` が停止 error を返し token を cache に残し DB を変えないことを確かめる test へ改める（到達不能になった mismatch 削除契約の移行先は Matrix に記録）。`rollback_csv_import` の停止を 1 test 追加する。
- S5 UI（SPEC-STOP-D4）:
  - `src/features/stocktake/StocktakePage.tsx`: 停止状態を持つ定数 1 つと、それを既定値にする `StocktakePage` の prop（例 `writesSuspended`）。停止中は PageHeader の直下に warning の Alert（icon + 見出し + 本文）を出し、`StocktakeStartPanel` の開始ボタン（:367）、`StocktakeCountEntry` の fieldset（:557 付近、`disabled` prop）、「棚卸しを確定する」（:304）を無効にする。`StocktakeItemList`（部門絞り込み・未入力のみ・ページ送り）は無効にしない。route（`src/routes/stocktake/index.tsx`）は変えない。
  - `src/features/stocktake/StocktakePage.test.tsx` / `StocktakePage.suggest.test.tsx`: 既存の render helper（`renderPage` :123、suggest :69）に停止 off を渡して従来の flow test を維持し、停止中（既定）の test を追加する。
  - `src/features/csv-import/components/PreviewStep.tsx`: Z004 の停止状態の定数をこの file で定義して export し、それを既定値にする prop で「取り込む」（:128）を無効にする。「ファイルを選び直す」は有効のまま。`src/features/csv-import/CsvImportPage.tsx`: その定数を `PreviewStep.tsx` から import し、停止中は `CsvImportFlowPanel` の先頭（Z004 タブ内）に warning の Alert を出す（定数を 2 箇所に持たない）。
  - `src/features/csv-import/CsvImportPage.test.tsx` / `components/PreviewStep.test.tsx`: 既存 test の render（PreviewStep :44 / :142）に停止 off を渡し、停止中の test を追加する。
- S6 設計正本（Writer が実装と同じ PR で同期。ADR 修正 lane が編集する「時点証拠契約（proposed・未実装）」節には触れない）。停止文言の正本を先に置くため、Writer は S6 の docs を実装 code（S1〜S5）より前の別 commit にする（test の期待値は docs から転記する。42 §22.10）:
  - 新設 `docs/adr/2026-09-23-legacy-stocktake-z004-write-stop.md`（templates/adr.md の形。SPEC-STOP-D1〜D6、Rejected Options、Revisit Trigger = ⑤）と `docs/adr/README.md` の Existing Decision Records へ 1 行。
  - `docs/function-design/35-biz-stocktake-service.md`: 現行本文側に停止の節（BIZ-06 停止文言の正本、3 関数が先頭で返すこと、§20.3〜§20.5 の処理ステップは旧本体の記述であること）。
  - `docs/function-design/32-biz-csv-import-service.md`: 同じく停止の節（BIZ-03 停止文言の正本、§15.4 / §15.5 は旧本体の記述）。
  - `docs/function-design/42-cmd-sales-stocktake.md` §22.5（start_stocktake / update_count / complete_stocktake）と `docs/function-design/41-cmd-pos.md` §17.5（commit_csv_import / rollback_csv_import）: CMD の処理ステップは不変で、停止中は BIZ の停止 error を通常変換で返す旨の注記。
  - `docs/function-design/73-ui-stocktake.md`: 停止中の画面（案内の見出し・本文の正本、無効にする操作と使える操作）。
  - `docs/function-design/55-ui-csv-import.md`: §55.0「既存Z004 UIの扱い」付近に停止中の Z004 タブ（案内の見出し・本文の正本、「取り込む」の無効化）。
  - `docs/SCREEN_DESIGN.md`（owner 決定 2026-09-24: 同 file の重なりは merge 順で解く）: 「売上データ取込み画面（日報 / 商品別CSV）」の節（:113-125）と「棚卸し画面」の節（:204-213）のレイアウト判断に各 1 行「現行 build では一時停止中。73 / 55 / 新設 ADR 参照」（取込み画面は Z004 の確定・取消、棚卸し画面は開始・カウント入力・確定が対象）。実装状況表（:26 / :50）は変えない。
  - `docs/diagrams/current-system.md`: 「在庫の増減と取消」（:172〜）と「棚卸しの時間と在庫」（:196〜）に、現行 build では該当入口が停止している旨。
  - `docs/diagrams/cross-feature-verification.md`: XFA-D2 / D3 に、停止後の harness は 5 入口の旧本体を呼ぶこと（診断の再現性は維持）。
- S7 生成物: `docs/function-design/90-traceability.md` を `cd src-tauri && cargo run --bin generate_traceability` で再生成する（REQ 付き test の追加で件数が変わる。手動編集しない）。
- S8 本 packet・Matrix・`docs/Plans.md`: 計画・現在地の同期は Coordinator が担当し、Writer は編集しない。

呼出し側の確認（起票時、main `3148347b`）: `rg -n "stocktake_service::(start_stocktake|update_count|complete_stocktake)\b|csv_import_service::(commit_csv_import|rollback_csv_import)\b|\b(start_stocktake|update_count|complete_stocktake|commit_csv_import|rollback_csv_import)\(" src-tauri --type rust` の一致 file は、定義 file（`stocktake_service.rs` / `commit.rs` / `rollback.rs`）、`cmd/stocktake_cmd.rs`、`cmd/csv_import_cmd.rs`、`csv_import_service/tests/{commit,rollback,cross_feature}_tests.rs`、`db/stocktake_repo.rs`（同名の IO 関数 `complete_stocktake` 等で、対象外）だけ。`src-tauri/src/seed_demo.rs` は SQL 直書きで BIZ を呼ばない。`src-tauri/tests/*` に呼出しなし。frontend の呼出しは `StocktakePage.tsx:170`、`hooks/useUpdateCount.ts:14`、`hooks/useCompleteStocktake.ts:14`、`csv-import/hooks/useCsvImportFlow.ts:100 / :121` だけ（`rg -n 'commands\.(startStocktake|updateCount|completeStocktake|commitCsvImport|rollbackCsvImport)' src -g '!src/lib/bindings.ts' -g '!*.test.*'`）。`rollbackCsvImport` の画面入口は Z004 の結果 step（`ResultStep`）だけで、commit 停止により到達しない。

## Non-scope

- 上記以外の tracked file。特に `src-tauri/src/cmd/*.rs` の production 本体、`src-tauri/src/lib.rs`（command 登録）、`src/lib/bindings.ts`、`src/lib/invoke.ts`、`src-tauri/src/biz/mod.rs`（`BizError`）、`src-tauri/src/cmd/mod.rs`（`CmdErrorKind`）、migration・schema、`src/features/home/**`、`src/features/csv-import/components/ResultStep.tsx`、`src/features/stocktake/hooks/**`、`src/lib/invalidation-contract.ts`（D-052 の mutation entry は残る。停止中は呼ばれないだけ）。
- `docs/SCREEN_DESIGN.md` の実装状況表（:26 / :50 の「実装済み」）: 実装の存在を示す表で、停止中の振舞いは S6 の 1 行と 73 / 55 / ADR が持つ。`docs/UI_TECH_STACK.md`: 新しい state 所有・query・invalidation の型を足さない。
- `docs/function-design/40-cmd-product.md` §5.3 / `30-biz-product-service.md` §4.10: `CmdErrorKind` と `BizError` の値を変えない（SPEC-STOP-D2）。
- `docs/decision-log.md`: 決定は新設 ADR に置く（並走する docs 復元 lane が新しい D-n を足すため、番号の衝突を避ける）。
- `docs/adr/2026-09-18-stocktake-time-evidence.md` とその「時点証拠契約（proposed・未実装）」節: ADR 修正 lane の範囲。
- ホーム画面の「前日分が未取込みです」警告: 本番 DB は Z004 取込み 0 件で始まり、この警告は取込み 0 件では出ない（`53-ui-home.md` §53.2 派生値 `needsImportWarning`）。出続けるのは過去の Z004 取込みを持つ開発 DB だけで、文言（前日分が未取込み）は事実として正しい。停止中だけの抑止のために home の契約と test を変える費用に見合わない。
- STK-1 / STK-2 を起こさない他の writer（`fix_integrity`、商品一括 import、商品フォームの数量・連動設定）。㉗ 申し送りの版更新は ② 以降。
- `docs/backlog.md` の STK-1 / STK-2 entry の現在地更新（closeout）。

## Acceptance Criteria

AC の文言・command は起票時 main `3148347b` の現物で確認した。test 本数・件数は PR / runner 出力を正とし、ここに書かない。

- AC1（BIZ の停止）: `legacy_stop_tests.rs` の test が、F1〜F7 の各 fixture 状態で 5 つの BIZ 入口を呼び、`Err` の variant と文言が Spec Contract の BIZ-06 / BIZ-03 停止文言と完全一致し、DB 全 table（`sqlite_master` の `type='table'` かつ `name NOT LIKE 'sqlite_%'` の全行・全列。`operation_logs`・`stocktakes`・`stocktake_items` を含む）が呼出し前後で一致することを確かめ、停止した入口を呼ぶ前に各 fixture の前提値を assert し（Matrix の Fixture 表「前提 assert」列。例: F1 = 在庫 8・明細 actual 10・棚卸し in_progress）、`cd src-tauri && cargo test --lib legacy_stop` が PASS。Matrix の Mutation-style Adequacy Questions の観測（停止文を除いた隔離 copy で F1 の確定後の在庫 10、F2 の commit 後の在庫 6）を command と出力つきで PR body に記録する。F1 は連動商品 1 件だけの DB で作り、旧 complete を force_fill=false で呼べるようにする。`update_count` は負数・0・正数・存在しない明細 ID のどれでも停止文言を返す（検査より停止が先）。
- AC2（command の停止）: `cd src-tauri && cargo test --lib -- stocktake_cmd csv_import_cmd` が PASS。5 つの production command 関数（`stocktake_cmd::start_stocktake` / `update_count` / `complete_stocktake`、`csv_import_cmd::commit_csv_import` / `rollback_csv_import`）が kind = `validation`（棚卸し）/ `import_error`（Z004）、message = 停止文言、field = null、error_id = null を返し、DB が不変。`commit_csv_import` は有効な preview token を cache に残す。期待値は production 定数を import せず source design から転記する（42 §22.10）。
- AC3（旧本体の保持と入口の閉鎖）: `rg -n 'legacy_(start_stocktake|update_count|complete_stocktake|commit_csv_import|rollback_csv_import)\(' src-tauri/src` のすべての一致が、旧本体の定義行か、`tests/` 配下の file か、`#[cfg(test)]` の module 内の行である（reviewer が一致行ごとに確認し、Writer は一致行の分類を PR body に列挙する）。S2 の既存 test の差分は呼出し先の名前（と必要な `use`）だけで、assert・入力値の行に差分がない（reviewer が `git diff origin/main -- src-tauri/src/biz/csv_import_service/tests/commit_tests.rs src-tauri/src/biz/csv_import_service/tests/rollback_tests.rs src-tauri/src/biz/csv_import_service/tests/cross_feature_tests.rs` と `stocktake_service.rs` の test module の hunk で確認）。`cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings` が成功。
- AC4（診断の維持）: `rg -c '#\[ignore' src-tauri/src/biz/csv_import_service/tests/cross_feature_tests.rs` が `2`（baseline: 同 command で `2`）。`cd src-tauri && cargo test --offline --lib cross_feature_tests -- --ignored --nocapture` の出力に `XFA_TEMPORAL_FAIL` と `XFA_LATE_IMPORT_FAIL` が出て、この 2 本は FAIL する（既知の不具合を旧本体で再現し続けることの確認。① の拒否で解消扱いにしない）。ignored を含めない通常の `cargo test` は PASS。
- AC5（棚卸し画面）: `npm test -- src/features/stocktake` が PASS。停止中（prop 省略 = 既定）の test が、案内の見出し・本文（73 の正本と完全一致）、icon（非色の signal）、「棚卸しを開始する」の disabled、進行中の棚卸しがあるときのカウント入力欄 fieldset の disabled と「棚卸しを確定する」の disabled、部門絞り込み・未入力のみ・ページ送りが有効であることを確かめる。停止中はどの操作でも `startStocktake` / `updateCount` / `completeStocktake` の mock が呼ばれない。既存の flow test は停止 off で assert を変えずに PASS。
- AC6（Z004 タブ）: `npm test -- src/features/csv-import` が PASS。停止中の test が、Z004 タブの案内（55 の正本と完全一致）、プレビュー表示、「取り込む」の disabled、「ファイルを選び直す」が有効、「取り込む」の click で `PreviewStep` の `onConfirm` mock が呼ばれないことを確かめる。`CsvImportPage.test.tsx` の停止 test は test ごとの `useCsvImportFlow` mock で preview 状態と安定した `confirmImport` 参照を返して render し（既存 test の assert は不変）、click しても `confirmImport` が呼ばれないことを確かめる。`commitCsvImport` の未呼出しは assert しない（`useCsvImportFlow` を mock するため到達経路が無い）。「取り込む」の disabled を外す mutation でこの test が red になることを実注入で確かめる。日報タブの既存 test（`src/features/daily-report-import/**`）は不変で PASS。
- AC7（wire 不変）: `cd src-tauri && cargo run --bin generate_bindings` の後の `git diff --exit-code -- src/lib/bindings.ts` が exit 0。`git diff origin/main --stat -- src-tauri/src/lib.rs src-tauri/src/biz/mod.rs src-tauri/src/cmd/mod.rs src/lib/invoke.ts src/lib/bindings.ts src/lib/invalidation-contract.ts src/features/home` が空。
- AC8（設計正本と生成物）: `rg -n '^### SPEC-STOP-D[1-6]' docs/adr/2026-09-23-legacy-stocktake-z004-write-stop.md` が 6 行。`rg -l '一時停止中' docs/function-design/35-biz-stocktake-service.md docs/function-design/32-biz-csv-import-service.md docs/function-design/41-cmd-pos.md docs/function-design/42-cmd-sales-stocktake.md docs/function-design/73-ui-stocktake.md docs/function-design/55-ui-csv-import.md docs/diagrams/current-system.md docs/SCREEN_DESIGN.md` が 8 file すべてを出す。各 file の「時点証拠契約（proposed・未実装）」節に差分がない（reviewer が hunk で確認）。`cd src-tauri && cargo run --bin generate_traceability -- --check` が exit 0。
- AC9（footprint）: `git diff --name-only origin/main...HEAD` が Scope S1〜S8 に列挙した file（S6 の `docs/SCREEN_DESIGN.md` を含む）だけ。S6 の docs の commit が S1〜S5 の実装 code を含む最初の commit より前にある（`git log --reverse --name-only origin/main..HEAD` で確認）。
- AC10（検証 gate）: `bash scripts/local-ci.sh changed` が成功。`bash scripts/doc-consistency-check.sh` と `--target plan` が ERROR なし。Human Gate の manual 前に Writer が `cd src-tauri && cargo check --release` を成功させる。
- AC11（manual）: owner の目視で Test Plan の M1 / M2 が PASS し、`python3 scripts/pr-gate.py record` の manual record に結果が残る。

## Design Sources

- Requirements / spec: `docs/spec/requirements.md` REQ-205（棚卸しによる在庫数の補正）/ REQ-401（POS 売上データの取込み）。
- Architecture: `docs/ARCHITECTURE.md`（UI → CMD → BIZ → IO、CMD は薄く業務規則は BIZ）。
- Function / command / DTO: `docs/function-design/35-biz-stocktake-service.md` §20.3〜§20.5 / §20.10、`32-biz-csv-import-service.md` §15.4 / §15.5 / §15.10、`42-cmd-sales-stocktake.md` §22.3 / §22.5 / §22.10、`41-cmd-pos.md` §17.4 / §17.5、`40-cmd-product.md` §5.3（`CmdErrorKind` 12 値凍結、D-061）、`30-biz-product-service.md` §4.10。
- DB: 変更なし（`docs/db-design/tracking-system-tables.md`、`pos-tables.md` の現行定義を読むだけ）。
- Screen / UI: `docs/function-design/73-ui-stocktake.md` §73.7 / §73.9 / §73.10、`55-ui-csv-import.md` §55.0 / §55.4 / §55.5、`53-ui-home.md` §53.2（Non-scope の根拠）、`docs/design-system/02-component-catalog.md`（Alert）。
- Decision log / ADR: `docs/adr/2026-09-18-stocktake-time-evidence.md`（Status、適用範囲の但し書き、D8 の初導入と互換性、Consequences）、D-052（invalidation SSOT）、D-061（`CmdErrorKind` 凍結）、D-070（Z004 自動在庫連動は v1.0 必須、owner 2026-09-22 再確認）。
- 出典: `docs/research/2026-09-16-diagram-audit.md` STK-1 / STK-2、`docs/diagrams/cross-feature-verification.md` XFA-D2〜D4、`docs/archive/plans/2026-09-16-stocktake-count-baseline.md`「後続 runtime lane ㉘ への申し送り」。

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | 35 / 32（BIZ 停止の節と文言の正本）、41 / 42（CMD 注記）、新設 ADR | updated in this PR（S6） |
| Command / DTO / generated binding / wire shape | 40 §5.3（kind 不変）、41 / 42 | existing sufficient（型・kind・登録・bindings を変えない。停止文言は 35 / 32 が所有） |
| DB / transaction / audit / rollback / migration | db-design（変更なし） | existing sufficient（停止は DB を読まず書かない） |
| Screen / UI / route state / Japanese wording | 73 / 55（案内の文言と無効化の正本） | updated in this PR（S6、SCREEN_DESIGN の 2 節に各 1 行を含む）。UI_TECH_STACK と SCREEN_DESIGN の実装状況表は Non-scope に理由 |
| CSV / TSV / report / import / export format | 32 / 23（変更なし） | existing sufficient（parse・preview は不変） |
| Durable decision / ADR | 新設 `docs/adr/2026-09-23-legacy-stocktake-z004-write-stop.md` | updated in this PR（S6） |

## Registration / Generation Obligations

| 変更対象 | 登録・生成義務 |
|---|---|
| Tauri command | 該当なし（追加・改名・削除なし。`lib.rs` の `collect_commands` 不変、AC7） |
| function-design doc 新設 | 該当なし（既存 file の節追加だけ。公開関数名は不変で `design_compliance_test` の対象は増えない。旧本体は `pub(crate)` で同 test の抽出対象外 = `src-tauri/tests/design_compliance_test.rs:479`） |
| source / workflow doc 新設 | ADR 新設 → `docs/adr/README.md` の Existing Decision Records に 1 行（S6） |
| AGENT_OPERATING_MANUAL §5.5 | 該当なし |
| REQ / coverage | REQ-205 / REQ-401 付き test の追加で件数が変わる → `cd src-tauri && cargo run --bin generate_traceability` で `docs/function-design/90-traceability.md` を再生成（S7、AC8） |
| route | 該当なし |
| operator 画面新設 | 該当なし（既存画面の状態追加） |

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| REQ-205 / REQ-401 | 新設 ADR、35 / 32 の停止の節 | SPEC-STOP-D1 | 部分修正は開始から実測までの移動を誤算する（確定式だけの修正）。条件付き許可は例外が増えて停止 lane が本実装に膨らむ。自動 off で売上だけ通すと連動の意味が変わる | S1 | AC1 / AC2 |
| REQ-205 / REQ-401 | 35 / 32 §20.10 / §15.10、40 §5.3 | SPEC-STOP-D2 | 新 kind は D-061 の 12 値凍結を解き bindings・invoke・41 / 42 / 40 / 30 を動かすが、⑤ で消える一時状態で、UI は kind で分岐しない（入口を先に無効化）。既存 kind の用法に合う: BIZ-06 は状態による拒否を `ValidationFailed` で返している（「棚卸し対象の商品がありません」「未入力の商品が{N}件あります」）。BIZ-03 の `ImportError` は取込み pipeline の拒否（「このファイルは既に取込み済みです」） | S1 | AC2 / AC7 |
| REQ-205 / REQ-401 | 新設 ADR、cross-feature-verification XFA-D3 | SPEC-STOP-D3 | 旧本体の削除は既存 test（BIZ の commit / rollback / 棚卸し）と診断を消し、③ / ④ が移行先にする回帰を失う。`#[cfg(test)]` 化は非 test build で旧本体が使う helper を dead_code にし clippy `-D warnings` が落ちる | S1 / S2 | AC3 / AC4 |
| REQ-205 / REQ-401 | 73 / 55 の停止の節 | SPEC-STOP-D4 | 画面の非表示だけでは command が通る（D1 と併用）。押せるボタンで error を返すだけでは、入力後に拒否を知る。画面ごとの定数を既定値にする prop にし、既存 flow test は off で維持する（UI の flag を安全の根拠にしない） | S5 | AC5 / AC6 / M1 / M2 |
| REQ-205 / REQ-401 | 新設 ADR、Matrix「後続 lane への期待結果」 | SPEC-STOP-D5 | 後続 lane の期待結果を ① で実行可能な oracle として固定すると、ADR 修正 lane の結論（判定不能の扱い）と衝突する。① は入力と停止の検査だけを固定する | S3 | AC1 |
| REQ-205 / REQ-401 | 新設 ADR Revisit Trigger | SPEC-STOP-D6 | 解除は ⑤（migration・新 writer・新 command・新 UI を一組で切替）だけ。部分解除（例: 開始だけ許可）は旧 writer と新 schema の混在を生む | ADR | review |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: 新設 ADR（決定・却下案・解除条件）と 35 / 32 / 73 / 55（文言と振舞いの正本）で答えられる（S6 の同期後）。
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: SPEC-STOP-D1〜D6 を新設 ADR へ昇格する。
- Assumptions and constraints: 店は未導入（L-237 / L-240）。停止は build 固定で状態を持たない。`pub(crate)` + `#[cfg_attr(not(test), allow(dead_code))]` で非 test build の clippy が通る（Contract Probe）。
- Deferred design gaps, risk, and follow-up target: ⑤ まで開発 DB の旧進行中棚卸しは閲覧だけ（完了・中止できない）。ホームの Z004 未取込み警告は過去の取込みを持つ開発 DB で出続ける（Non-scope）。CMD `commit_csv_import` の mismatch 時 token 削除（SPEC-SDI-D4）は停止中は到達不能（⑤ で再有効化、Matrix に移行先）。同じく `csv_import_cmd.rs:438-459` が固定していた成功時だけの token 削除と AdditionalImportConfirmationRequired → commit 成功（41 §17.5）も停止中は到達不能になり、⑤ で新 commit 経路へ再接続する。後続 lane の期待結果のうち判定不能に関わるものは ADR 修正 lane の版に従う。
- Test Design Matrix can cite design decision IDs or source doc sections: Matrix の各行は SPEC-STOP-Dn と 35 / 32 / 42 / 41 / 73 / 55 の節を引く。
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: 「停止した入口は業務 write を起こさない」の例外候補を点検した — (a) `rollback_csv_import` の冪等 Ok 経路 → 停止を先に置き一律 error、(b) `commit_csv_import` の失敗時 `csv_import_failed` log → 停止は `execute_commit` より前で log を書かない、(c) CMD `commit_csv_import` の cache 検査（UUID・TTL）→ BIZ より前に走り token 不正・期限切れは従来 error のまま（DB write なし）、有効 token は BIZ の停止へ到達し cache を消さない、(d) 他の writer（入庫・手動販売・返品・廃棄・日報・`fix_integrity`・商品一括 import）→ 停止対象外で、STK-1 / STK-2 は棚卸し確定と Z004 commit を要するため起きない、(e) `seed_demo` → SQL 直書きで BIZ を経由しない（停止の対象外、既存どおり）。

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | Z004 adapter（IO-02 parser）と preview は不変。停止は BIZ の core 入口だけ | ADR |
| Fact check / design decision split | 事実 = 店は未導入（台帳 L-237 / L-240）、STK-1 / STK-2 は合成 DB で再現済み（監査）。決定 = 停止の位置・kind・UI | ADR、本 packet |
| Lifecycle / retry | 停止中の再試行・再起動・翌日は同じ拒否。preview token は消さない。旧進行中棚卸しは ⑤ まで閲覧のみ | Matrix State Lifecycle |
| Operator workflow | 店は Excel 継続（L-076）。開発・demo の利用者は案内で理由と使える操作が分かる | 73 / 55、M1 / M2 |
| Replacement path | ⑤ が停止を外し新入口へ切替える。旧本体 `legacy_*` は ③ / ④ の移行後に ⑤ で撤去 | ADR Revisit Trigger |
| Data safety / evidence | 合成データのみ。実 POS・実 DB・実 EJ を fixture に使わない | Data Safety |
| Reporting / accounting semantics | 停止中は Z004 由来の商品別売上が増えない（日報の公式集計は不変）。棚卸しの評価額は増えない | 55 / ADR Consequences |
| Manual verification | 案内の見え方・見出しの全文表示・無効化の見分けは目視（M1a / M1b / M2）。M1 は既存 data の表示で分岐し、合成行の挿入をしない（L3 Eligibility (3)）。表示されなかった分岐は自動 test（AC5）が固定 | Test Plan |
| 環境・再現性 | 新しい環境依存なし。dead_code の扱いは rustc の挙動で、CI の clippy `-D warnings` が検出する | Contract Probe |

## Design Readiness

- Existing design docs are sufficient because: 旧処理の振舞い（35 / 32 / 41 / 42 / 73 / 55 の現行本文）と STK-1 / STK-2 の根拠（監査・XFA）は既存で足りる。停止の決定だけが未記載で、本 PR の S6 で足す。
- Source docs updated in this PR: 新設 ADR、ADR README、35、32、41、42、73、55、SCREEN_DESIGN（2 節に各 1 行）、current-system、cross-feature-verification、90-traceability（生成）。
- Design gaps intentionally deferred: 新しい計数・受領・判定・取消の設計の実装（② 〜 ⑤）、時刻判定（次の design lane）。
- Durable decisions discovered in this plan and promoted to source docs: SPEC-STOP-D1〜D6 → 新設 ADR。

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): 停止の業務規則は BIZ の公開関数が持つ。CMD は不変で BIZ の error を通常変換する。UI は入口を無効化し理由を示すだけで、安全の根拠にしない。
- Backend function design: 5 関数の先頭 guard と旧本体の分離（S1）。
- Command / DTO / data contract: 型・kind・登録・bindings 不変（AC7）。停止文言は Spec Contract と 35 / 32。
- Persistence / transaction / audit impact: なし（停止は TX を開かず、operation log も書かない）。
- Operator workflow / Japanese UI wording: Spec Contract の案内文言、73 / 55 が正本。
- Error, empty, retry, and recovery behavior: 停止 error は既存の error 表示に載る（棚卸し: 上部の「操作できませんでした」Alert、Z004: `ErrorState` の import_error 表示と recoverTo = idle）。回復操作は無い（⑤ まで待つ）。
- Testability and traceability IDs: REQ-205 / REQ-401、SPEC-STOP-D1〜D6。

## Contract Probe

- rustc の dead_code 解析は `#[cfg_attr(not(test), allow(dead_code))]` を付けた `pub(crate)` 関数を root として扱い、その関数だけが呼ぶ private helper を非 test build で未使用と警告しない: scratchpad の最小 crate（`mod biz { fn helper(); pub fn entry() -> Err; #[cfg_attr(not(test), allow(dead_code))] pub(crate) fn legacy_entry() { helper() } }`）で `cargo clippy --offline --all-targets -- -D warnings` → 成功、`cargo test` → 1 passed。対照として allow 行を外すと `error: function 'helper' is never used` と `error: function 'legacy_entry' is never used` で失敗（rustc 1.94.1、2026-09-24 実測）。本 repo での成立は AC3 の clippy が確認する。

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| SPEC-STOP-D1: 5 つの BIZ 入口が先頭で停止 error、DB 全 table 不変 | `stocktake_service.rs` / `commit.rs` / `rollback.rs` | `legacy_stop_tests.rs`（AC1） | — |
| SPEC-STOP-D1: 停止は検査より先（負数・0・不存在 ID でも停止文言） | `stocktake_service::update_count` | `legacy_stop_tests.rs`、`stocktake_cmd` の改めた 2 test（AC1 / AC2） | — |
| SPEC-STOP-D1: `rollback_csv_import` の rolled_back 冪等 Ok も停止 | `rollback.rs` | `legacy_stop_tests.rs`（F3 の rolled_back import） | — |
| SPEC-STOP-D1: commit 停止は `csv_import_failed` log を書かない | `commit.rs` | AC1 の全 table 比較（`operation_logs` を含む） | — |
| SPEC-STOP-D2: kind / message / field / error_id の wire | CMD（不変）+ BIZ | `stocktake_cmd` / `csv_import_cmd` test（AC2） | — |
| SPEC-STOP-D2: 型・kind・登録・bindings 不変 | — | AC7 の generate_bindings diff | — |
| SPEC-STOP-D2: CMD commit は停止時に preview token を消さない | `csv_import_cmd::commit_csv_import`（不変） | `csv_import_cmd` の改めた test（AC2） | — |
| SPEC-STOP-D3: 旧本体は production から呼ばれない | `legacy_*` の定義位置 | AC3 の一致行の分類 | — |
| SPEC-STOP-D3: 旧本体の既存振舞いは不変 | `legacy_*` | 付け替えた既存 BIZ test（AC3） | — |
| SPEC-STOP-D3: 診断 2 本は旧本体で再現を続ける | `cross_feature_tests.rs` | AC4（`--ignored` で FAIL） | — |
| SPEC-STOP-D4: 棚卸し画面の案内・無効化・閲覧維持 | `StocktakePage.tsx` | `StocktakePage.test.tsx`（AC5） | M1 |
| SPEC-STOP-D4: Z004 タブの案内・「取り込む」無効・プレビュー維持 | `CsvImportPage.tsx` / `PreviewStep.tsx` | `CsvImportPage.test.tsx` / `PreviewStep.test.tsx`（AC6） | M2 |
| SPEC-STOP-D4: 日報タブ・他画面は不変 | — | 既存 test（`daily-report-import/**` 等） | — |
| SPEC-STOP-D5: F1〜F7 の builder が停止検査に使われる | `legacy_stop_tests.rs` | AC1 | — |
| SPEC-STOP-D5: 後続 lane の期待結果 | — | 未実行（Matrix に記録、ADR 修正 lane 依存を明示） | non-scope（② 〜 ④） |
| SPEC-STOP-D6: 解除は ⑤ だけ | ADR | review | non-scope（⑤） |
| 隣接: 42 §22.10 の CMD validation test 方式 | `stocktake_cmd` test | AC2 | — |
| 隣接: 41 §17.5 / SPEC-SDI-D4 の mismatch 時 token 削除 | CMD（不変、停止中は到達不能） | 到達不能のため test なし（Matrix 移行先 = ⑤） | non-scope（⑤ で再有効化） |
| 隣接: 41 §17.5 成功時のみ token 削除 / CMD の AdditionalImportConfirmationRequired → commit 成功（`csv_import_cmd.rs:438-459`、`:396` の test の後半） | CMD（不変、停止中は到達不能） | 到達不能のため test なし（Matrix 移行先 = ⑤） | non-scope（⑤ で新 commit 経路へ再接続） |
| 隣接: 73 §73.9 の kind 別回復（`stocktake_in_progress` / `stocktake_not_in_progress`） | `StocktakePage.tsx`（不変） | 既存 test を停止 off で維持 | — |
| 隣接: 55 §55.5 の kind 別表示・recoverTo | `useCsvImportFlow.ts` / `ErrorState.tsx`（不変） | 既存 test を維持 | — |
| 隣接: D-052 の mutation entry と invalidation | `invalidation-contract.ts`（不変） | 既存 static / meta test | — |
| 隣接: 73 UI-10-D11 の HID 連続スキャン focus・IME Enter | `StocktakeCountEntry`（不変、停止中は fieldset disabled） | 既存 test を停止 off で維持 | — |

## Test Plan

Test Design Matrix: [2026-09-23-legacy-stocktake-z004-write-stop.md](test-matrices/2026-09-23-legacy-stocktake-z004-write-stop.md)。Human Gate に manual を含むため、Writer は owner の native build の前に `cd src-tauri && cargo check --release` を成功させる。

- targeted tests: `cargo test --lib legacy_stop`、`cargo test --lib -- stocktake_cmd csv_import_cmd`、`npm test -- src/features/stocktake src/features/csv-import`。
- negative tests: 停止中の各入口に負数・0・不存在 ID・rolled_back 済み import・有効 / 期限内の preview token を渡し、どれも停止 error かつ DB 不変。停止中の UI で入口を操作しても command mock が呼ばれない。
- compatibility checks: 付け替えた既存 BIZ test と停止 off の既存 UI test が assert 不変で PASS。`generate_bindings` の diff なし。
- data safety checks: fixture は合成 JAN（既存 test と同じ `29xxxxxxxxxxx` / `4912345678xxx` 系）と合成商品名だけ。
- main wiring/integration checks: production command 関数を `tauri::test::mock_builder` の managed `AppState` で直接呼ぶ（AC2）。route `/stocktake` は prop を渡さず停止中の既定値を使う（AC5 の既定 render が同じ経路）。
- manual（owner の目視、native build）:
  - 種別: 画面変更の目視確認（Human Visual Confirmation）を native build で行う。Windows native 固有の L3 項目は無い（案内と disabled は jsdom でも観測でき、L3 Eligibility (1) を満たさない）。M1 / M2 とも、案内の見出し（AlertTitle、`src/components/ui/alert.tsx:45` の `line-clamp-1`）が省略記号で切れず全文読めることを合格条件に含める。
  - M1 棚卸し画面 `/stocktake`。到達手順: 既存のアプリ data のまま起動し、サイドバーの「棚卸し」を開く。合成行の挿入や DB の作り直しはしない。画面の表示で分岐し、表示された方だけを確認する（もう一方は AC5 の自動 test が固定）。
    - M1a（「棚卸しの開始」が表示される = 進行中なし）: PageHeader の直下に warning の案内（icon + 見出し + 本文）が出る。「棚卸しを開始する」が押せない見た目で、押しても何も起きない。前回の棚卸しの要約が見える。合格 = 3 点と見出しの全文表示を満たし、文言が読みやすい。
    - M1b（進捗ヘッダと棚卸し一覧が表示される = 旧方式の進行中あり）: 案内が出る。カウント入力欄と「棚卸しを確定する」が操作できない。部門絞り込み・未入力のみ・ページ送り（2 ページ以上ある場合）が使える。合格 = 3 点と見出しの全文表示を満たす。
  - M2 売上データ取込み `/csv-import` → 「商品別CSV取込み（Z004）」タブ: 案内が出る。Coordinator が渡す合成 Z004（CP932）を選ぶとプレビューが出て、「取り込む」が押せず「ファイルを選び直す」は押せる。「日報取込み」タブは従来どおり。合格 = 4 点と見出しの全文表示を満たす。

## Boundary / Wire Contract

- producer: BIZ（`stocktake_service` / `csv_import_service` の公開関数）→ CMD の既存 `From<BizError> for CmdError`（`src-tauri/src/cmd/mod.rs:130`）。
- consumer: 既存の frontend error 表示（`StocktakePage.tsx` の `describeError`、`useCsvImportFlow.ts` の `decideRecoverTo` と `ErrorState`）。停止中は UI が入口を先に無効化するため通常は到達しない。
- wire type: `CmdError { kind: "validation" | "import_error", message: <停止文言>, field: null, error_id: null }`。型・値の集合は不変。
- internal type: `BizError::ValidationFailed(String)` / `BizError::ImportError(String)`（既存 variant）。
- precision/range: 該当なし。
- round-trip path: 該当なし（UI は停止文言を解析しない。停止の判定は UI 側の定数と backend の拒否で独立に行う）。
- invalid input: 停止が引数の検査より先で、どの入力でも同じ停止 error。
- compatibility: command 名・引数・戻り型・登録・`bindings.ts` 不変。⑤ で新 command へ切り替える時点で本 wire は撤去される。

## Review Focus

- 5 つの入口のどれにも、停止より前の DB 読書き・operation log・cache 削除が無いか（`rollback_csv_import` の冪等経路、`commit_csv_import` の失敗 log、CMD の cache 分岐）。
- 旧本体への付け替えが呼出し先の名前だけで、既存 test の assert を弱めていないか。停止で到達不能になった契約（mismatch 時 token 削除）の扱いが移行先付きで記録されているか。
- 既存 kind の流用（SPEC-STOP-D2）が consumer の既存分岐（`stocktake_*` の回復、`import_error` → idle）を誤作動させないか。
- UI の停止定数が安全の根拠になっていないか（backend が独立に拒否するか）。閲覧が本当に残るか（一覧の絞り込み・ページ送り、記録詳細）。
- home と SCREEN_DESIGN の実装状況表を Non-scope にした理由の妥当性。
- ADR 修正 lane・EJ parser lane・docs 復元 lane との footprint の重なり（Plans.md への申し送りを参照）。

## Spec Contract

Contract ID: SPEC-STOP

- SPEC-STOP-D1（停止の位置と範囲）: BIZ の公開関数 `start_stocktake` / `update_count` / `complete_stocktake`（BIZ-06）と `commit_csv_import` / `rollback_csv_import`（BIZ-03）は、関数の最初の文で停止 error を返す。DB を読まず、TX を開かず、operation log を書かない。条件による例外を持たない。
- SPEC-STOP-D2（停止 error の wire）: BIZ-06 は `BizError::ValidationFailed(<BIZ-06 停止文言>)`、BIZ-03 は `BizError::ImportError(<BIZ-03 停止文言>)`。CMD の既存変換で kind = `validation` / `import_error`、message = 停止文言、field = null、error_id = null。`CmdErrorKind` は 12 値のまま（D-061）。
  - BIZ-06 停止文言: `棚卸しの開始・数の保存・確定は一時停止中です。数えた後の入出庫が確定で打ち消される不具合を直すまで使えません。`
  - BIZ-03 停止文言: `商品別CSV（Z004）の取込みの確定と取消は一時停止中です。在庫が二重に減ったり戻ったりする不具合を直すまで使えません。`
- SPEC-STOP-D3（旧本体）: 現行の処理本体は crate 内部の `legacy_*`（`pub(crate)`）として残し、呼出し元は `#[cfg(test)]` の test・診断・fixture だけとする。③ / ④ が新処理へ移すときの回帰の対照に使い、⑤ で撤去する。
- SPEC-STOP-D4（画面）: 停止状態は画面ごとの定数 1 つが所有し、画面 component はそれを既定値にする prop で受ける。停止中は warning の Alert（icon + 見出し + 本文）を出し、書く操作の入口を無効化する。閲覧・絞り込み・プレビュー・他タブは無効化しない。案内文言:
  - 棚卸し 見出し: `棚卸しの入力は一時停止中です`
  - 棚卸し 本文: `数えた後の入出庫が、確定のときに在庫から打ち消されてしまう不具合を直しています。直るまで、棚卸しの開始・数の保存・確定はできません。これまでの棚卸しの記録と一覧は見られます。`
  - Z004 見出し: `商品別CSV（Z004）の取込みは一時停止中です`
  - Z004 本文: `取込みや取消で在庫が二重に減ったり戻ったりする不具合を直しています。直るまで、取込みの確定と取消はできません。ファイルの内容確認（プレビュー）と日報の取込みはできます。`
- SPEC-STOP-D5（再現 fixture）: 合成データの builder F1〜F7（Matrix の Fixture 表）を test code に置き、① では各 fixture 状態での停止と DB 全 table の不変だけを assert する。後続 lane の期待結果は Matrix に記録し実行しない。判定不能の扱いに依存する期待結果は ADR 修正 lane の版に従い、本 lane では固定しない。既存 `#[ignore]` 診断 2 本は旧本体で再現を続け、⑤ で Matrix 指定の正規回帰へ移す。
- SPEC-STOP-D6（解除）: 停止は ⑤（migration 登録・共通 writer・新 command / bindings / 画面の一括切替）でだけ外す。部分的な解除をしない。ADR を superseded にして記録する。

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-STOP-D1 | S1 / S3 | `legacy_stop_tests.rs` | 停止より前の読書きなし | AC1 |
| SPEC-STOP-D2 | S1 / S4 | `stocktake_cmd` / `csv_import_cmd` test、generate_bindings diff | kind 流用の誤作動なし | AC2 / AC7 |
| SPEC-STOP-D3 | S1 / S2 | 付け替えた既存 BIZ test、`cross_feature_tests --ignored` | assert の弱体化なし | AC3 / AC4 |
| SPEC-STOP-D4 | S5 | `StocktakePage.test.tsx`、`CsvImportPage.test.tsx`、`PreviewStep.test.tsx` | 閲覧の維持、UI 定数を安全の根拠にしない | AC5 / AC6 / M1 / M2 |
| SPEC-STOP-D5 | S3 | `legacy_stop_tests.rs` | 後続 lane の期待結果を固定しない | AC1、Matrix |
| SPEC-STOP-D6 | S6 | review | 部分解除なし | AC8 |

## Data Safety

- commit しないもの: 実 POS の Z004 / 日報 / EJ、実店舗 DB・backup・log、店の商品名・JAN・価格。
- local-only: `.local/**`（台帳・相談記録）は読むだけで引用は要旨と行 ID に限る。M2 用の合成 Z004 は Coordinator が local で作り、repository に置かない。
- synthetic-only: F1〜F7 と既存 test の fixture はすべて合成値（合成 JAN、合成商品名、合成数量）。

## Implementation Results

[PR #95](https://github.com/higekiri-w/inventory-system-desktop/pull/95) で実装し squash merge 済み（`33b0dbbb`、2026-09-25）。S1〜S7 を実装した: (1) 旧棚卸しの開始・カウント保存・確定（BIZ-06 の `start_stocktake` / `update_count` / `complete_stocktake`）と Z004 の業務 commit・取消（BIZ-03 の `commit_csv_import` / `rollback_csv_import`）の 5 入口が、関数の最初の文で既存 kind の固定文言 error を返し、DB を読まず書かない（operation log・preview cache も触らない）。command の型・kind・登録・`bindings.ts` は不変。(2) 旧本体は `legacy_*`（`pub(crate)`）へ移し、既存 BIZ test と `#[ignore]` 診断 2 本（`XFA_TEMPORAL_FAIL` / `XFA_LATE_IMPORT_FAIL`）は旧本体を呼んで不具合の再現を続ける。旧本体の root は Final Review P3-1 の裁定で `#[cfg_attr(not(test), expect(dead_code))]` にし、production から呼ぶと clippy `-D warnings` が unfulfilled expectation で失敗する（Plan 時点の `allow` の記述は S1・Contract Probe に残る）。(3) 棚卸し画面（`StocktakePage`、`writesSuspended`）と売上データ取込み画面の Z004 タブ（`PreviewStep` が export する停止定数を既定値にする `importSuspended`、`CsvImportPage` が同じ定数で案内を出す）に warning の案内を出し、開始・カウント入力・確定と「取り込む」を無効にした。閲覧・絞り込み・ページ送り・プレビュー・日報タブは従来どおり。(4) 再現 fixture F1〜F7（合成データ）と、各 fixture の前提を assert してから 5 入口の停止と DB 全 table の不変を確かめる `legacy_stop_tests.rs`、CMD の停止 wire と preview token の保持の test。Codex broad C1 の是正で F1 / F3 にも合成 preview を持たせた。(5) 停止 ADR [`docs/adr/2026-09-23-legacy-stocktake-z004-write-stop.md`](../../adr/2026-09-23-legacy-stocktake-z004-write-stop.md)（SPEC-STOP-D1〜D6）を新設し、35 / 32 / 41 / 42 / 73 / 55・SCREEN_DESIGN・現行図・横断検証図に停止中である旨と文言の正本を置き、traceability を再生成した。Gated Amendment 0 回。

## Review Response

- Findings Freeze: frozen after round 1（94528ef8）; post-freeze exceptions: none.

### Plan Review round 1（2026-09-24、plan-gate、Opus 5.5 fresh、対象 plan-first `d2acc6e9`、裁定 Coordinator）

操作列 = `成立`。P1 0 / P2 3 / P3 8、全件採用し packet / Matrix を in-place で是正した。P2-1: `csv_import_cmd.rs:438-459`（成功時のみ token 削除 / 追加確認 → commit 成功）の到達不能化を Ledger・Matrix 移行先・Residual に記録。P2-2: T1 に fixture ごとの前提 assert を追加し、停止文を除く mutation で F1 = 在庫 10 / F2 = 在庫 6 を観測する証跡を要求。P2-3: M1 を DB の状態で M1a / M1b に分け到達手順を記載。P3-1: SCREEN_DESIGN の 2 節へ各 1 行を S6 / AC8 / AC9 に追加（owner 決定 2026-09-24、重なりは merge 順で解く。review が示した行番号 :116-125 / :204-210 は節が逆で、:113-125 = 取込み画面、:204-213 = 棚卸し画面に置いた）。P3-2: AC3 を `src-tauri/src` 全体の一致行の分類へ置換（`cfg_attr(expect)` 案は未 probe のため不採用）。P3-3: 商品登録・一括 import による旧進行中棚卸しへの明細追加を操作列と Adjacent Pattern Audit に記載。P3-4: 時計異常・EJ 欠落 fixture を除いた理由を Fixture 表の下に記載。P3-5: AC6 / T15 に `onConfirm` / `confirmImport` 未呼出しと disabled 除去 mutation を追加。P3-6: Z004 の停止定数を `PreviewStep.tsx` が定義・export し `CsvImportPage.tsx` が import。P3-7: M2 に route `/csv-import`、native 固有 L3 なしの判定、AlertTitle（`line-clamp-1`）の全文表示を合格条件に追加。P3-8: S6 の docs を実装 code より前の commit にする指示を S6 / AC9 に追加。

### Final Review broad（Opus 5.5 fresh、対象 `f17ab303`、裁定 Coordinator）

P1 0 / P2 0 / P3 3。SPEC-STOP-D1〜D6 は成立。P3-1（旧本体の `allow(dead_code)` では production からの呼出しを検出しない）= 採用、`94eb4956` で `cfg_attr(not(test), expect(dead_code))` に置き換え、ADR SPEC-STOP-D3 の文を合わせた。本 packet の S1・Assumptions・Contract Probe の `allow` の記述は Plan 時点の判断として残し、以後はこの裁定を正とする（必須 gate の clippy `-D warnings` が unfulfilled で失敗することを注入で確認。素の `cargo build` は warning に留まる）。P3-2（PR body の AC1 / AC3 の証跡と kind の誤記）= 採用、PR body を更新。P3-3（停止中も入力を促す subtitle と説明文）= M1 で owner の所感を聞いて判断する → 2026-09-24 の M1 で owner「案内は案内として読めるから見出し下の説明とは全然違うし大丈夫じゃないかな」により変更しない。

### Final Review broad Codex 側（GPT-6 Astra high、対象 `faa7f9e3`、裁定 Coordinator）

P1 0 / P2 0 / P3 1、Ordinary Operation 成立。Opus 側の結果（本節の前の小節と PR body の該当行）を読まずに行った独立の 1 本。C1（F1 と F3 の fixture は preview を持たず、`commit_csv_import` の停止を 0 回しか検査していない。production の停止は成立）= 採用、`196b21c2` で F1 / F3 に合成 preview を 1 件ずつ持たせ、検査の前に preview が空でないことを assert した（assert だけを入れた状態で F1 / F3 の 2 本が red、preview の追加後に 7 本 green）。production code の変更はなく、manual M1a / M1b / M2 の対象（画面）に影響しない。

Final Review の closeout 時点の結論（2026-09-25、Coordinator）: helper の専用 record（`#issuecomment-5823886887`）の review は pass。broad は 2 本で、互いに独立した Double Audit。(1) Opus 5.5 fresh の broad（対象 `f17ab303`、`#issuecomment-5823882101`、P1 0 / P2 0 / P3 3。P3-1・P3-2 を採用し是正、P3-3 は owner の所感で変更なし）。(2) Codex broad（GPT-6 Astra high、対象 `faa7f9e3`、`#pullrequestreview-5307935006`、P1 0 / P2 0 / P3 1。C1 を採用し `196b21c2` で是正）。closure は Opus 5.5 fresh（対象 `93f729c8`、`#issuecomment-5823882398`、閉じる対象 4 件をすべて閉鎖と確認、新規 P3 1〈P3-3 の owner の結論の記録漏れ〉は `b9644702` で記録）と、base 同期（`docs/Plans.md` の衝突を解いた merge）後の Codex closure（対象 `b86fde68`、`#pullrequestreview-5311377167`、P1 0 / P2 0 / P3 0、合否 = 通す）。manual は pass（owner、tested head `b86fde68`、`#issuecomment-5824051403`: M1a PASS・M2 PASS。M1b は DB の状態で表示されない分岐で、自動 test〈AC5〉が固定する）。1 回目の manual は `faa7f9e3` で M1a / M1b / M2 PASS だったが、base 同期で衝突を解いたため `docs/agent-guidance/merge-evidence.md` の規則により引き継がず再実施した（Owner Effort Budget の引上げの理由）。R4 は not-required。Final Review P3-3（停止中も入力を促す subtitle と説明文）は M1 での owner の所感により変更なし。
