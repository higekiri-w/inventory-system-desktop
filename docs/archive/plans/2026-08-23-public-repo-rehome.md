# Plan Packet: public repository rehome — README 紹介文化 + 移植記録の正本化（docs-only、R2）

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

If a state-only commit materializes multiple phases, list the complete adjacent forward sequence and the pre-existing evidence for every intermediate transition in an append-only review/evidence record. Recording compression never permits a gate skip.

- Phase: ready-hosted-final
- Risk: R2
- Execution Mode: fable-window
- Plan Commit: a98563e
- Amendments: none
- Coordinator: Fable
- Writer: Codex（発注）
- Plan Reviewer: Sonnet（独立 context）
- Final Reviewer: Sonnet（D-062 の vendor 分離 — Writer = Codex、Plan Reviewer / Final Reviewer = Sonnet、別 vendor）
- Reviewed Content HEAD: 63eb6a1
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required
- Human Gate: owner plan approval 済み（2026-08-23、介入 1/2。採否 5 点 = README 7 節構成 / 主な機能 9 項目 / 公開とライセンス節の All rights reserved 1 行 + LICENSE file 無し / D-077 + PUBLIC_REPO_MIGRATION Rehome addendum / DEV_SETUP_CHECKLIST §4.6 の 2 箇所のみ、すべて採用）→ owner Ready 承認済み（2026-08-24、介入 2/2、README 文言の修正要望なし）→ hosted final（workflow_dispatch 1 run）→ merge（docs-only のため visual confirmation なし）

### 遷移記録（2026-08-23、state-only 遷移 plan-draft -> plan-gate -> plan-approved -> implementing）

- plan-draft -> plan-gate の evidence: packet を plan-first commit `a98563e` で commit 済み（R2 docs-only、Test Matrix 省略）、`doc-consistency-check.sh` ERROR 0（WARN 2 既存）/ `check-env-safety.sh` exit 0 / repo 内 session URL 0、Draft PR #1（inventory-system-desktop）open。
- plan-gate -> plan-approved の evidence: 独立 Sonnet Plan Reviewer 3 round（round 1 P2 2 / P3 2 → 是正 `17d7e02`、round 2 P2 1 → 是正 `3771e34`、round 3 fresh delta 検証 P1 0 / P2 0 / P3 0 = pass、収束記録 `0858657`、Review Response 参照）、owner plan approval（2026-08-23、介入 1 回目 / 予算 2 回。採否 5 点すべて採用、Human Gate 行参照）、Plan Commit = plan-first commit `a98563e`（本 branch の全 commit の祖先）。
- plan-approved -> implementing の evidence: Writer = Codex 発注（Execution Mode `fable-window`、Coordinator が Plan Commit 記入と本遷移を完了してから発注書を提示）。Plan Gate 時に検出した `SCREEN_DESIGN.md` §1 等の stale 実装状況表記は本 packet の Non-scope とし、`docs/Plans.md` に「docs の実装状況表記の一括棚卸し」entry を起票して受け皿にした（owner 指示 2026-08-23）。

### 遷移記録（2026-08-24、state-only 遷移 implementing -> local-verified -> independent-review -> human-confirm）

- implementing -> local-verified の evidence: Writer（Codex）が実装 commit `843844b`（README / D-077 / Rehome addendum / DEV_SETUP_CHECKLIST §4.6）+ `63eb6a1`（Implementation Results）を push、`bash scripts/local-ci.sh full` RESULT=PASS / END_TREE_STATE=CLEAN、AC 12 本 + Contract Probe 1〜3 達成を報告。Coordinator が HEAD / AC 8 本 / src 無変更 / attribution 無しを独立再現。
- local-verified -> independent-review の evidence: Sonnet Final Review（独立 context、2026-08-24、対象 `63eb6a1`）: AC oracle 14 本を自力再実行して全一致、README 9 機能の実在性を SCREEN_DESIGN / FUNCTION_DESIGN の行で確認、D-077 の数値・固有名は packet 固定事実と一致、PUBLIC_REPO_MIGRATION は追加のみ、DEV_SETUP_CHECKLIST は 2 行のみ、Implementation Results に SHA / test 件数なし。P1 0 / P2 0 / P3 2、verdict pass（Review Response 参照）。
- independent-review -> human-confirm の evidence: Reviewed Content HEAD = `63eb6a1`（P1/P2 = 0 確定後に設定）。owner Ready 承認（介入 2/2）待ち。承認後に human-confirm -> ready-hosted-final を記録し Draft を Ready 化する。

### 遷移記録（2026-08-24、state-only 遷移 human-confirm -> ready-hosted-final）

- human-confirm -> ready-hosted-final の evidence: owner Ready 承認（2026-08-24、介入 2 回目 / 予算 2 回、「承認するよ、これでいいとおもう」。README 文言の修正要望なし）。完成 HEAD（本 commit）で `bash scripts/local-ci.sh full` を実行し、その HEAD SHA と RESULT を PR body の Validation に記入（tracked docs には転記しない、D-038）。hosted final: 本 change は classifier で docs=true のみ（workflow=false）で `paths-ignore` 対象のため Ready event では自動 run が作られない。先例（design-backbone-reference、docs-only R2）に倣い `Hosted CI Requirement: required` を維持し、Ready 化後に同一 HEAD の run が 0 件であることを確認してから owner-directed `workflow_dispatch` を 1 run（docs/ci.md CI-TRIGGER-D1 表の 2 行目）。merge gate = 生 PR HEAD = PR body の local full evidence SHA = hosted run headSha の三点一致。

## Owner Effort Budget

- 介入回数上限: 2（Plan approval / Ready 承認。既定 3 より狭める理由: docs-only R2 で L3 / visual confirmation が無く、owner の decision point がこの 2 点しか存在しないため。PR #87 / #93 と同型）
- 実働時間上限: 30分
- relay 往復上限: 2
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

§5.5を使わないchangeは両方`none`のままにする。使う場合はtarget branch / PRへorder commitを混ぜず、artifact pathと専用remote order branch refを宣言する。

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R2

Reason:
docs-only（`README.md` 全面書き換え、`docs/decision-log.md` 新規 entry、`docs/PUBLIC_REPO_MIGRATION.md` 追記節、`docs/Plans.md` entry、旧 repo 名 current-state 記述の是正）。runtime contract・DB・CSV・DTO・route・operator workflow の変更なし。DEV_WORKFLOW Risk Tiers の R2「docs change that affects maintainability but not runtime contracts」に該当し、Test Design Matrix は省略（PR #87 / PR #93（design-first）と同型）。

## Goal

Goal Invariant: 新 repository `kosei-w90607/inventory-system-desktop`（旧 public repo `inventory-system-public` からの 2026-08-23 full-history replay 移植先、現在 private）を public 化する前に、(1) `README.md` を利用者向けの紹介文へ書き換え、(2) 移植の事実を正本 docs（decision-log / PUBLIC_REPO_MIGRATION）に記録し、(3) 現在形で旧 repo 名を名指ししている記述を新 repo 前提に更新する。

### 最小完了条件

- `README.md` が紹介文として全面書き換えられ、Scope で定める必須 `## ` 7 節（概要 / 主な機能 / 技術構成 / 設計書 / 開発の進め方 / ビルドと起動 / 公開とライセンス）を exact 見出しで持ち、H1 タイトル + 1 行説明を先頭に置く。
- `docs/decision-log.md` に D-077 が新設され、移植の方式・検証・LICENSE 非採用の判断が記録されている。
- `docs/PUBLIC_REPO_MIGRATION.md` に 2026-08-23 の追記節があり、public→public rehome の方式が既存の private→public Phase B 手順と区別して記録されている。
- `docs/DEV_SETUP_CHECKLIST.md` の Windows clone remote 確認手順（current-state で旧 repo 名を GitHub リモートとして指している箇所）が新 repo 名に更新されている。
- `docs/Plans.md` に本 packet の entry がある。

### 失敗定義

- README に旧 repo の URL を「現在の repo」として書く、または `claude[.]ai/code/session` URL が残る。
- LICENSE file を追加する（owner 決定 = 非採用、Non-scope 逸脱）。
- D-077 の記述が owner 確定済みの事実（trailer 除去済み・full-history replay・identity/日付保持・root snapshot byte 同一・14 項目検証 PASS・独立 2 run 一致）と矛盾する。
- 歴史記述（過去 PR/issue への言及、archive された packet の記録）を「現在形」に書き換えて事実を損なう。

### 非目的

- LICENSE file の追加。
- local directory の改名（WSL / Windows clone のディレクトリパス名は本 packet の対象外）。
- Draft PR #96 の内容変更（`docs/Plans.md` の UI 一覧の背骨 D entry は L130 の壊れたリンク是正を除き既存行のまま、PR 番号更新は将来の別 amendment）。
- AI workflow 系 file（`AGENTS.md` / `CLAUDE.md` / `.agents/` 等）を README の紹介文脈から除外する設計判断（いわゆる C 案）。
- 旧 repo（`inventory-system-public`）側の変更。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

- `README.md`（全面書き換え、日本語、紹介文寄りでポートフォリオ色を抑える）。必須節と内容:
  - タイトル `# 在庫管理システム` + 1 行説明（手芸店向け在庫管理デスクトップアプリ）。
  - `## 概要`: 個人経営手芸店の POS 連携在庫管理、Windows デスクトップ、ローカル SQLite（外部送信なし）、非 IT の店舗運営者が日常運用する前提を明記。
  - `## 主な機能`: `docs/FUNCTION_DESIGN.md` 対象モジュールと `docs/SCREEN_DESIGN.md` §1 画面一覧に実在する機能から 6〜10 項目（商品検索・一覧・登録・修正 / 入出庫管理〈入庫・返品交換・手動販売出庫・廃棄〉/ 在庫照会・入出庫履歴・在庫変動追跡 / 棚卸し / POS レジ連携〈売上データ取込み・PLU 書出し〉/ 売上レポート〈日次・月次〉/ 一括価格改定 / 商品一括インポート / バックアップ・復元・在庫少基準設定・操作ログ・在庫整合性検証などの運用管理機能）。根拠 doc の節（`SCREEN_DESIGN.md` §1、`FUNCTION_DESIGN.md` 対象モジュール一覧）へのリンクを 1 行添える。
  - `## 技術構成`: Tauri 2 / React 19 / TypeScript / SQLite（rusqlite）/ Vite / Rust、テスト基盤（`cargo test`、Vitest + React Testing Library）。
  - `## 設計書`（各設計書への入口）: `docs/ARCHITECTURE.md` / `docs/DB_DESIGN.md` / `docs/FUNCTION_DESIGN.md` / `docs/SCREEN_DESIGN.md` / `docs/design-system/README.md` へのリンク。
  - 必須 `## ` 節は上記 7 つ（概要 / 主な機能 / 技術構成 / 設計書 / 開発の進め方 / ビルドと起動 / 公開とライセンス）を exact 見出しで置く。H1 はタイトル 1 つ。必要なら追加節（例: スクリーンショット）を置いてよいが必須ではない。
  - `## 開発の進め方`: 2 行以内、AI 協働 workflow の詳細は書かず `AGENTS.md` と `docs/DEV_WORKFLOW.md` へ誘導するリンクのみ。
  - `## ビルドと起動`: `docs/DEV_SETUP_CHECKLIST.md` の実コマンドのみ（`npm ci --ignore-scripts` を含む、install script 実行なしの注記）+ `npm run tauri dev`。前提バージョン（Rust 1.83+、Node 24 系）を 1 行。
  - `## 公開とライセンス`: 「閲覧用に公開しています。All rights reserved — コードの再利用・再配布は許諾していません。」相当の 1〜2 行。LICENSE file は置かない。
  - 禁止事項（Writer 遵守）: 実店舗名・個人情報を書かない。`claude[.]ai/code/session` URL を書かない。旧 repo（`inventory-system-public`）の URL を「現在の repo」として書かない。
- `docs/decision-log.md`（末尾に追記、既存 D-076 の後ろ）: `## D-077: public repository の再移植 — inventory-system-public から inventory-system-desktop（2026-08-23）` を新設。
  - Decision: 旧 public repo `kosei-w90607/inventory-system-public`（main 216 commit 中 135 commit に Claude Code が付与する session link の `Claude-Session:` trailer が混入。decision-log / README / runbook には session URL そのものを転記しない）について、owner 判断で (a) GitHub 上の PR/issue/comment 本文から該当文字列を除去済み、(b) 履歴は in-place force push ではなく新 repo `kosei-w90607/inventory-system-desktop` へ full-history replay で移植（trailer 除去 + 再生成で変わった commit SHA への docs/message 内言及を追随、author/committer identity と日付は保持、tree は SHA 言及以外 byte 同一）。移植は 2026-08-23 完了（新 main `a16d57f…` 216 commit、branch `agent/ui-list-backbone-d` `20c4600…`、root snapshot `902647b` は byte 同一で SHA 不変、検証 14 項目 PASS + 独立 2 run 一致）。新 repo は本 packet の PR merge 後に owner が public 化し、旧 repo を private 化する。LICENSE file は置かない（All rights reserved を README に明記し、後から MIT 等を足せる非対称性を理由とする）。
  - Status: accepted（owner 判断 2026-08-23）。
  - Why: private→public 初回移行（`docs/PUBLIC_REPO_MIGRATION.md` の parentless snapshot 手順、2026-07 実施）は既に public だった履歴には適用できない — 対象は「trailer を含む既存 public 履歴」であり、parentless 単一 commit 化は PR 由来 commit と編集履歴（issue 対応・review 是正の積み重ね）を失う。full-history replay は個々の commit の trailer のみ除去し、それ以外の履歴・author/committer identity・日付を保持する。
  - Impact: `docs/PUBLIC_REPO_MIGRATION.md` に public→public rehome 追記節。`docs/Plans.md` の「UI 一覧の背骨 D」entry が言及する Draft PR #96 は新 repo で再 open が必要（番号更新は本 packet の Non-scope、別 amendment）。旧 repo の PR/issue 参照番号は旧 repo のものとして歴史記述に残る。
  - Alternatives considered: in-place force push で旧 repo の履歴を書き換える案（trailer 除去のための rewrite でも PR 由来 commit・レビュー往復の編集履歴はそのまま残り、object identity の破壊的書き換えを public repo に対して行うリスクがあるため却下）。LICENSE file を今回追加する案（棄却理由は README/公開判断と同じ非対称性論拠、Non-scope）。
  - Revisit: 新 repo の public 化後に予期しない public surface（Actions/Security & Analysis 等）が観測されたとき、または LICENSE 方針を変更するとき。
- `docs/PUBLIC_REPO_MIGRATION.md`（追記、`## Closeout` の後ろに新設）: `## Rehome addendum（2026-08-23）: public → public full-history replay` を追加。
  - 本 runbook の "Prepare the snapshot" 〜 "Visibility and development cutover" は private→public の初回移行（sanitized な parentless single-commit snapshot からの開始）を対象とし、既に public だった履歴の再移植には適用しないことを明記。
  - 2026-08-23 の rehome は Claude-Session trailer を含む旧 public repo `inventory-system-public` の履歴を、trailer 除去 + 再生成 SHA への docs 内言及追随を行いながら新 repo `inventory-system-desktop` へ full-history replay した旨、author/committer identity と日付を保持し tree は SHA 言及箇所以外 byte 同一である旨を記録（D-077 参照）。
  - 検証結果（14 項目 PASS、独立 2 run 一致、root snapshot `902647b` byte 同一で SHA 不変）と、旧 repo は private 化・新 repo は本 packet merge 後に public 化する運用順序を記録。
  - 数値・SHA・run 詳細は D-077 の記載と重複させず、本節は「Phase B 手順との区別」と「1 行サマリ」に留める（節同士の乖離防止）。
- `docs/DEV_SETUP_CHECKLIST.md`（該当箇所のみ更新、他行は変更しない）: §4.6「L3 利用者デモのコード同期手順」内、`git remote get-url origin` のコメント「public repo（inventory-system-public）を指しているか必ず確認」と、その下の「origin が旧 private repo（`inventory-system`）のままだと... `git remote set-url origin git@github.com:kosei-w90607/inventory-system-public.git`」の 2 箇所を、新 repo 名 `inventory-system-desktop` を参照する記述に更新する。`旧 private repo（inventory-system）` という別の歴史的事実（現行 public repo とは異なる、さらに古い private repo の名称）への言及はそのまま残す。
- `docs/Plans.md`（entry 追加 + 壊れたリンク 1 箇所の是正）: 「次の行動」セクションの「UI 一覧の背骨 D」entry の直後に、本 packet の進行中 entry（`- [ ]`）を 1 件追加する。文言は本 packet の Goal と現 Phase（plan-draft）を要約する。加えて、同 entry（L130）内の `design-system/reference/mockup-d-lists.html` への markdown リンクは main 上にリンク先 file が無く `doc-consistency-check.sh` R3 で ERROR になる（main `a16d57f` 時点で既存、PR #96 相当の branch にのみ file がある）ため、plan-first commit で Coordinator がリンク表記を backtick path + 注記に置き換える（R0 link cleanup、内容の意味は変えない）。これ以外の既存行には触れない。

## Non-scope

- `LICENSE` / `SECURITY` / `CONTRIBUTING` file の追加。
- WSL / Windows local clone directory の改名。
- Draft PR #96 の内容や、新 repo での再 open・番号更新作業そのもの（本 packet は docs 記述のみ）。
- 旧 repo（`inventory-system-public`）側の設定・可視性変更。
- `docs/evidence/`、`docs/archive/plans/`、`docs/research/` 配下などに残る歴史記述（過去の PR/issue URL、当時の作業ディレクトリ言及）の書き換え。これらは移植前の事実の記録であり、現在形の記述ではない。
- WSL 側の auto-memory sanitize path（`~/.claude/projects/-home-kosei-Projects-inventory-system-public/`）や local ディレクトリパスへの言及（`AGENTS.md` / `CLAUDE.md` / `docs/DEV_SETUP_CHECKLIST.md` 該当箇所 / `scripts/tests/codex-safe-wrappers.test.sh`）。これらはローカルディレクトリ名に基づく記述で GitHub repo 名の current-state 記述ではない。

## Acceptance Criteria

- README: `rg -c '^## (概要|主な機能|技術構成|設計書|開発の進め方|ビルドと起動|公開とライセンス)$' README.md` = 7（Scope の必須 7 節が exact 見出しで存在。追加節は可）、`rg -c 'All rights reserved' README.md` = 1、`rg -c 'claude[.]ai/code/session' -g '!.git' .` = 0、`rg -c 'inventory-system-public' README.md` = 0（現在の repo として旧名を書かない）。
- rg oracle の規約: 本 packet の「`rg -c ... = 0`」は「出力なし・exit 1」を 0 とみなす（`rg -c` は 0 件のとき何も出力しない）。
- decision-log: `rg -c '^## D-077' docs/decision-log.md` = 1。
- PUBLIC_REPO_MIGRATION: `rg -c '^## Rehome addendum' docs/PUBLIC_REPO_MIGRATION.md` = 1。
- DEV_SETUP_CHECKLIST: `rg -c 'inventory-system-desktop' docs/DEV_SETUP_CHECKLIST.md` ≥ 1（更新箇所に新 repo 名が入っている）、§4.6 の 2 箇所以外に既存の `inventory-system-public` 言及（ローカルパス由来）が変化していないことを `git diff` で目視確認。
- `bash scripts/doc-consistency-check.sh` ERROR 0（既存 WARN は許容）。
- `bash scripts/check-env-safety.sh` exit 0。
- `bash scripts/local-ci.sh full` RESULT=PASS / END_TREE_STATE=CLEAN、hosted final と exact-HEAD 三点一致。
- Plans.md: 本 packet の entry が「次の行動」に存在する（新規 1 行追加 + L130 リンク表記是正、他の既存行の差分なし）。`rg -c '\]\(design-system/reference/mockup-d-lists\.html\)' docs/Plans.md` = 0。

## Design Sources

- Requirements / spec: N/A（docs-only、runtime contract 非該当。README/decision-log/runbook の記述整合が対象）。
- Architecture: `docs/ARCHITECTURE.md` 冒頭（README の設計書リンク先の内容確認用）。
- Function / command / DTO: `docs/FUNCTION_DESIGN.md` 目次・対象モジュール一覧（README 機能一覧の根拠）。
- DB: N/A。
- Screen / UI: `docs/SCREEN_DESIGN.md` §1 画面一覧と使用頻度（README 機能一覧の根拠）。
- Decision log / ADR: `docs/decision-log.md` D-075 / D-076（末尾の既存 entry、書式の手本）、`docs/PUBLIC_REPO_MIGRATION.md`（Phase B 手順、public→public rehome との区別対象）。

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | — | 該当なし（docs-only） |
| Command / DTO / generated binding / wire shape | — | 該当なし |
| DB / transaction / audit / rollback / migration | — | 該当なし |
| Screen / UI / route state / Japanese wording | README 機能一覧の根拠 = `FUNCTION_DESIGN.md` / `SCREEN_DESIGN.md` | existing sufficient（README はリンクのみ、画面設計自体は変更しない） |
| CSV / TSV / report / import / export format | — | 該当なし |
| Durable decision / ADR | `docs/decision-log.md` D-077（本 packet で新設）、`docs/PUBLIC_REPO_MIGRATION.md` 追記節 | updated in this PR |

## Registration / Generation Obligations

該当なし（Tauri command / route / REQ coverage / function-design doc 新設のいずれも発生しない。README・decision-log・runbook 追記・Plans.md entry のみ）。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| public repo 移植の記録 | `docs/PUBLIC_REPO_MIGRATION.md` Closeout | SPEC-REHOME-D3 | full-history replay を選択、in-place rewrite は却下 | decision-log D-077 / PUBLIC_REPO_MIGRATION 追記節 | AC rg |
| README 紹介文化 | `docs/SCREEN_DESIGN.md` §1、`docs/FUNCTION_DESIGN.md` 対象モジュール | SPEC-REHOME-D1 | 実在機能のみ列挙、ポートフォリオ色を抑える | README.md | AC rg |
| LICENSE 非採用 | owner 決定（本 packet 固定事実） | SPEC-REHOME-D2 | All rights reserved を README に明記、LICENSE file は追加しない | README.md / D-077 | AC rg |
| 旧 repo名 current-state 是正 | `docs/DEV_SETUP_CHECKLIST.md` §4.6 | SPEC-REHOME-D5 | 現在形記述のみ更新、歴史記述は残す | DEV_SETUP_CHECKLIST.md | AC rg / git diff |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: yes（D-077 に移植事実の by/impact/alternatives を記録し、PUBLIC_REPO_MIGRATION 追記節が Phase B との区別を記録する）。
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: SPEC-REHOME-D1〜D5 は README / decision-log / PUBLIC_REPO_MIGRATION.md / DEV_SETUP_CHECKLIST.md に転記（本 PR）。
- Assumptions and constraints: 移植の事実（trailer 混入 135/216 commit、新 main `a16d57f…` 216 commit、root snapshot `902647b` byte 同一、検証 14 項目 PASS + 独立 2 run 一致）は本 packet の発注者（Coordinator）が固定した既定事実として扱い、本 packet では再検証しない。README の機能一覧は `SCREEN_DESIGN.md` §1 / `FUNCTION_DESIGN.md` 対象モジュール一覧に実在する項目のみを採用する。
- Deferred design gaps, risk, and follow-up target: 新 repo の public 化・development-remote cutover・旧 repo の private 化・Draft PR #96 の再 open は本 packet の Non-scope。旧 repo名への歴史的言及の網羅的棚卸し（archive/evidence/research 配下）は本 packet では実施せず、必要になれば別 change で扱う。
- Test Design Matrix can cite design decision IDs or source doc sections: R2 docs-only で Matrix 省略。AC の rg presence oracle が代替。
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: docs-only。runtime 契約・DB・route は不変。escape hatch なし。

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | 該当なし（docs-only） | — |
| Fact check / design decision split | 事実 = owner 確定済みの移植実績（Coordinator 発注に固定記載）。判断 = SPEC-REHOME-D1〜D5（README 構成・LICENSE 非採用・記録の置き場・現在形是正範囲） | 本 packet |
| Lifecycle / retry | 該当なし | — |
| Operator workflow | 該当なし（店舗運営者向け画面・業務フローへの影響なし） | — |
| Replacement path | 新 repo の public 化・development-remote cutover は本 packet merge 後の別ステップ（owner 実施） | `docs/PUBLIC_REPO_MIGRATION.md` 追記節 |
| Data safety / evidence | README/decision-log/runbook に実店舗名・個人情報・session URL・detailed scan log を書かない | Data Safety |
| Reporting / accounting semantics | 該当なし | — |
| Manual verification | docs-only で L3 なし。owner が Plan Gate と Ready の 2 回介入で確認する | Human Gate |
| 環境・再現性 | 該当なし（新設の環境依存なし） | — |

## Design Readiness

- Existing design docs are sufficient because: README の機能一覧・技術構成・設計書リンクは `docs/SCREEN_DESIGN.md` / `docs/FUNCTION_DESIGN.md` / `docs/ARCHITECTURE.md` / `docs/DEV_SETUP_CHECKLIST.md` に既存の記述があり、新規設計判断を要しない。移植の事実は Coordinator 発注に固定済みで、decision-log / runbook への転記のみが残作業。
- Source docs updated in this PR: `docs/decision-log.md`（D-077 新設）、`docs/PUBLIC_REPO_MIGRATION.md`（追記節）、`README.md`（全面書き換え）、`docs/DEV_SETUP_CHECKLIST.md`（該当箇所のみ）。
- Design gaps intentionally deferred: Design Intent Audit 参照。
- Durable decisions discovered in this plan and promoted to source docs: SPEC-REHOME-D1〜D5。

Minimum design checks for business-app work:

- Layer ownership: 該当なし（docs-only）。
- Backend function design: 該当なし。
- Command / DTO / data contract: 該当なし。
- Persistence / transaction / audit impact: 該当なし。
- Operator workflow / Japanese UI wording: README は日本語、口語・人格表現は使わない技術文書として書く（Coordinator 発注の指示）。
- Error, empty, retry, and recovery behavior: 該当なし。
- Testability and traceability IDs: SPEC-REHOME-D1〜D5、D-077。

## Contract Probe

- Probe 1: `doc-consistency-check.sh` が `decision-log.md` の新規 `## D-NNN` 見出しや `PUBLIC_REPO_MIGRATION.md` の新規節を検査対象にして ERROR を出さないか（D-075/D-076 追加時の先例で通過実績あり）: Writer が追記後に `bash scripts/doc-consistency-check.sh` を実行して確認する。
- Probe 2: README の `All rights reserved` が既存 doc（LICENSE 相当の記述がどこにも存在しない現状）と重複・矛盾しないか: `rg -n -i 'license|all rights reserved' README.md docs/*.md`（本 packet 起草時点の既存ヒット = `docs/PUBLIC_REPO_MIGRATION.md` の LICENSE/SECURITY/CONTRIBUTING 方針行、`docs/decision-log.md` D-062 の同方針 gate 行、`docs/Plans.md` の本 packet entry。いずれも「方針決定の要求」であり README の All rights reserved 文言と矛盾しない。Writer は追記後に再実行し、README 1 行 + D-077 の LICENSE 非採用記述が増えるだけで他に重複行が無いことを確認）。
- Probe 3: `docs/DEV_SETUP_CHECKLIST.md` §4.6 の更新対象 2 箇所が、他の `inventory-system-public` 言及（ローカルパス由来、Non-scope）と隣接していないか: Writer は `sd` ではなく該当行のみを明示指定した `Edit` 相当の置換を用い、置換直後に `rg -n 'inventory-system-public|inventory-system-desktop' docs/DEV_SETUP_CHECKLIST.md` の全行を目視確認する（feedback: sd silent no-op / literal mode の教訓）。

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| SPEC-REHOME-D1 README 紹介文化（必須節・機能一覧・禁止事項） | README.md | AC rg: 必須 7 見出しの exact 一致 = 7、`inventory-system-public` = 0、`claude[.]ai/code/session` = 0 | — |
| SPEC-REHOME-D2 LICENSE 非採用 + All rights reserved 明記 | README.md / D-077 | AC rg: `All rights reserved` = 1 | — |
| SPEC-REHOME-D3 D-077 新設 | decision-log.md | AC rg: `^## D-077` = 1 | — |
| SPEC-REHOME-D4 PUBLIC_REPO_MIGRATION 追記節 + Phase B との区別 | PUBLIC_REPO_MIGRATION.md | AC rg: `^## Rehome addendum` = 1 | — |
| SPEC-REHOME-D5 旧 repo名 current-state 是正 | DEV_SETUP_CHECKLIST.md §4.6 | AC rg: `inventory-system-desktop` ≥ 1 + git diff 目視（2 箇所のみ） | — |
| Plans.md entry | Plans.md | 目視（新規行のみ追加、既存行差分なし） | — |
| 全体整合 | docs | `doc-consistency-check.sh` ERROR 0、`check-env-safety.sh` exit 0 | — |

## Test Plan

Test Design Matrix: 省略（R2 docs-only、PR #87 / PR #93 design-first と同型）。

- targeted tests: AC の rg presence/absence oracle、`doc-consistency-check.sh`、`check-env-safety.sh`。
- negative tests: `inventory-system-public` を現在の repo として書いた記述の残存 0（README / DEV_SETUP_CHECKLIST §4.6 更新箇所）、`claude[.]ai/code/session` の残存 0（repo 全体）。
- compatibility checks: `git diff --stat` で `src/**` / `src-tauri/**` に変更がないこと。`docs/DEV_SETUP_CHECKLIST.md` の diff が §4.6 の該当 2 箇所に限定されること。
- data safety checks: README / decision-log / runbook 追記節に実店舗名・個人情報・session URL・detailed scan log が含まれないこと（Data Safety 節参照）。
- main wiring/integration checks: README の設計書リンク先 5 file（ARCHITECTURE / DB_DESIGN / FUNCTION_DESIGN / SCREEN_DESIGN / design-system README）が実在すること（`git ls-files` 確認）。

Human Gate: owner plan approval（介入 1/2）→ Ready 承認（介入 2/2）。docs-only のため visual confirmation なし。

## Boundary / Wire Contract

該当なし（docs-only、JSON API・browser state・CSV・config・manifest・cache schema・Tauri command DTO・generated binding・report output・DB-backed compatibility のいずれにも触れない）。

## Review Focus

- README の機能一覧が `SCREEN_DESIGN.md` §1 / `FUNCTION_DESIGN.md` 対象モジュール一覧に実在する項目のみで構成されているか（架空の機能を書いていないか）。
- README・decision-log・PUBLIC_REPO_MIGRATION 追記節に実店舗名・個人情報・`claude[.]ai/code/session` URL・旧 repo URL の「現在の repo」表記が無いか。
- D-077 の記述が owner 確定済みの移植事実（trailer 混入件数、新 main SHA、root snapshot SHA、検証 14 項目 PASS）と数値・固有名で食い違っていないか。
- PUBLIC_REPO_MIGRATION.md の追記節が既存 Phase B 手順（private→public、parentless snapshot）の記述を書き換えず、区別のみを追加しているか。
- `docs/DEV_SETUP_CHECKLIST.md` の更新が指定 2 箇所に限定され、ローカルディレクトリパスへの言及（Non-scope）を誤って書き換えていないか。
- `docs/Plans.md` の diff が「新規 1 行の追加 + L130 のリンク表記是正」のみで、他の既存行に変更がないか。

## Spec Contract

Contract ID: SPEC-REHOME

- SPEC-REHOME-D1〜D5（本 packet）を正とし、採用後は README.md / decision-log.md D-077 / PUBLIC_REPO_MIGRATION.md 追記節 / DEV_SETUP_CHECKLIST.md §4.6 の本文が正本になる。

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-REHOME-D1 | README 全面書き換え | AC rg | 機能一覧の実在性 / 禁止事項 | rg 出力 |
| SPEC-REHOME-D2 | README 公開とライセンス節 | AC rg | LICENSE 非採用の一貫性 | rg 出力 |
| SPEC-REHOME-D3 | decision-log D-077 | AC rg | 移植事実との数値一致 | rg 出力 |
| SPEC-REHOME-D4 | PUBLIC_REPO_MIGRATION 追記節 | AC rg | Phase B との区別 | rg 出力 |
| SPEC-REHOME-D5 | DEV_SETUP_CHECKLIST §4.6 | AC rg / git diff | 更新範囲の限定 | rg / diff 出力 |
| Plans.md entry | Plans.md entry + L130 リンク表記是正 | 目視 + AC rg（壊れたリンク形 0） | Plans.md diff の限定 | diff 出力 |
| 全体 | doc-consistency-check / check-env-safety | exit 0 | — | log |

## Data Safety

- README / decision-log / PUBLIC_REPO_MIGRATION 追記節に実店舗名・個人情報・`claude[.]ai/code/session` URL・詳細スキャンログ・認証情報やその hash を書かない（commit SHA の短縮表記は事実記録として可）。
- local-only paths: なし。
- synthetic-only paths: 該当なし（docs のみ）。

## Implementation Results

README を利用者向けの紹介文へ全面改訂し、移植判断 D-077、public→public rehome の runbook addendum、Windows native clone の current remote 記述を同期した。実装対象は発注書で許可された docs 5 file に限定した。

Draft PR: https://github.com/higekiri-w/inventory-system-desktop/pull/1

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

## Review Response

If R3 review-only sub-agent is skipped, record an explicit line beginning with `Review-only skipped because:` and the reason.
- Findings Freeze: frozen at Final Review round 1（2026-08-24、Reviewed Content HEAD `63eb6a1`）; post-freeze exceptions: none.
- Final Review round 1（Sonnet、独立 context、2026-08-24、対象 `843844b` + `63eb6a1`）: P1 0 / P2 0 / P3 2、verdict pass。AC oracle 14 本を自力再実行して全一致、README 9 機能の実在性（棚卸し「前回結果との比較」/ 一括価格改定「取引先や部門で絞り込める」等）を SCREEN_DESIGN / FUNCTION_DESIGN の行で確認、技術構成とビルド手順は package.json / Cargo.toml / DEV_SETUP_CHECKLIST と一致、README 内アンカー実在。P3-1 README 概要の「CASIO レジ」は Scope の根拠 doc（SCREEN / FUNCTION）に語が無い → Coordinator 裁定 = 変更なし（`docs/project-memory.md` の CASIO SR-S4000 / ECR+ に根拠がある製品事実で、店舗・個人識別子ではなく Data Safety 非該当）。P3-2 Plans.md の状態行が implementing のまま → human-confirm 遷移 commit で同期（受理）。
- Plan Review round 1（Sonnet、独立 context、2026-08-23、packet `a98563e`）: P1 0 / P2 2 / P3 2、verdict fail。oracle 19 本実行（anchor 実在 / template 25 見出し / DEV_SETUP_CHECKLIST 2 箇所 / D-077 未使用 / Closeout 末尾 / AC baseline / doc-consistency ERROR 0 / env-safety / vendor 分離）。全件 Coordinator が rg で裏取りのうえ accept して是正: P2-1 README 見出し数 AC `≥ 8` が Scope の 7 節と不一致 → AC / Ledger を必須 7 見出しの exact 一致 oracle に置換し、Scope で `## 設計書` を命名・必須 7 節を明示 / P2-2 Probe 2 の「既存ヒットなし」は誤り（PUBLIC_REPO_MIGRATION / decision-log D-062 / Plans.md entry にヒット）→ baseline を実態に訂正 / P3-1 Trace Matrix に Plans.md 行を追加 / P3-2 `rg -c = 0` の規約（無出力・exit 1）を AC に明記。併せて Owner Effort Budget 介入 2 の理由を記載、Plan Commit を `a98563e` で記入。
- Plan Review round 2（Sonnet、fresh context、delta 検証、2026-08-23、packet `17d7e02`）: round 1 の 4 指摘の反映を確認、新 AC oracle 6 本は現行 baseline で有効（未実装のため期待どおり未達）、Plan Commit 祖先 ok、origin/main..HEAD = 2 commit、doc-consistency ERROR 0 / env-safety exit 0。P1 0 / P2 1 / P3 0、verdict fail。P2-1 Goal 最小完了条件 L53 の必須節列挙が旧称「設計書の入口」のまま → accept、`rg -n '設計書の入口'` で packet 全節を sweep し、L53 を必須 7 見出し名（設計書）に統一、Scope L83 の括弧書きも「各設計書への入口」に言い換えて同語形の残存 0（本記録行の引用を除く）。
- Plan Review round 3（Sonnet、fresh context、delta 検証、2026-08-23、packet `3771e34`）: round 2 是正の反映を確認、必須 7 見出し名は Goal / Scope / AC で同一集合・同一順、AC oracle 6 本は現行 baseline で有効（未実装のため期待どおり未達）、Plan Commit `a98563e` 祖先 ok、origin/main..HEAD = 3 commit、doc-consistency ERROR 0 / env-safety exit 0、Data Safety ok。P1 0 / P2 0 / P3 0、verdict pass。Plan rally 収束（round 3/3、天井内）。
