# Plan Packet: ㉗ 棚卸しと後着売上の時点証拠（STK-1 / STK-2、design-only、R3）

2026-09-16起票、2026-09-18にownerがCodexへ設計を委任。起票時の資料と旧案は内容commit feeb3fe9、および本書末尾の非遡及なReview Responseで追跡できる。現在の設計案は [時点証拠ADR](../../adr/2026-09-18-stocktake-time-evidence.md) に一本化する。旧案の実装指示や数値oracleを現在の契約として使わない。

## Workflow State

- Evidence Mode: github
- Phase: archive
- Risk: R3
- Execution Mode: codex-only
- Plan Commit: b7f195140ef03147d46d3a8cc26e4f39dcf5e062
- Amendments: 4298e0cbcd8a4f12cd2a161395509da03c75e5b2, 29b147cda9f58f898bdb1f90f210c8cfaea6fea9
- Coordinator: Codex（ownerの明示した設計委任による起草・統合。Human Gateと正式なreview承認は代行しない）
- Writer: Codex（発注65のsource詳細同期。合成モデルは検証入力として保持、runtimeは未着手）
- Plan Reviewer: Sonnet + Opus（非Codex vendorのfresh context。source同期版の正式Plan Gateは2026-09-19に通過）
- Final Reviewer: Sonnet + Opus（Writerと独立したfresh context）
- Final Review Minimum: 2
- Human Gate: ready,merge

ownerの今回の指示はsource詳細同期とplan-draftの準備を許可する。実装開始・Ready・mergeの許可には読み替えない。source docsは現行実装とproposedの新契約を区別する。design出力が揃えばplan-draftへ進め、独立Plan Gateなしにplan-approvedへ進めない。設計の技術的選択は委任範囲で詰め、未確認の店舗運用・ハードウェア事実を受容済みにしない。

遷移記録（append-only）:
- kickoff → spec-check → design（`24294ec4`）: Risk R2（docs-only。BIZ-06 / BIZ-03 / UI-10 の設計正本と decision-log を更新し、runtime 契約はこの lane では変えない）。Design Phase = 本 packet で方式を比較し Coordinator 既定案を置く。owner 回答待ちのため plan-draft へは進めない（DEV_WORKFLOW「design → plan-draft: no unresolved design questions」）。
- design → plan-draft → plan-gate（本 commit）: owner 回答（Q1 / Q3 candidate 採用、反例 2 件）と Opus 設計レビュー round 0 を `fd28766d` で反映、Q2 は店の回答「時間帯は関係なし、売れたらその場で訂正」で (b) `<=` に確定（未解決の設計質問なし）。docs-only R2 のため Test Matrix は本 packet の AC1〜AC9。Plan Review は Sonnet + Opus の独立 fresh context。
- plan-gate → design（state-backtrack、2026-09-17）: owner が Q2「同日販売はカウント前扱い `<=`」の一律適用を不承認（「実測の再入力と、販売に伴う数量訂正を区別し、記録済み入出庫・POS 販売・返品を混ぜても重複や欠落が起きない契約を先に定める」）。未解決の設計質問が復活したため design へ戻す。Plan Review は round 天井 3 に到達済み（round 1 Sonnet / Opus、round 2〜3 Sonnet closure）で disposition = owner escalation。設計確定後の plan-gate は D-D4 / Q2 の契約本体が変わるため新しい rally として round を再計上する。
- design → plan-draft → plan-gate（本 commit、2026-09-18）: 未解決の設計質問を解消。Q2 = owner 回答 2026-09-18（決めること 3 点: ゼロ行を含む保守案 / 余裕幅 10 分・条件付き / 完了済みへの後着は判定不能なら取込み全体を保留）と保留の出口の方向指示（現物数の入力・保存を新しい基準にする。確認チェックだけ・既存導線前提は不可）を D-D4 同日規則 / D-D8 / D-D9 へ反映。D-D9 の解除方式は Coordinator 案で、Plan Review の検証対象に置く。子 ID の衝突（`BIZ-03-D1` は 32 で使用済み）を D2〜D5 へ振り直し。Plan Review は新しい rally（round 1 から、Sonnet + Opus の独立 fresh context）。
- plan-gate → design（state-backtrack、2026-09-18）: owner は Plan Gate と介入上限 8 を承認したうえで、同じ発言で設計要求を追加した =「仕組みで守る = 人が間違えた場合に検知・停止・訂正できる設計。設計で防げるのに運用へ委ねている部分を先に塞ぎ、そのうえで残存リスクを提示する」「EJ は PLU 販売の本番開始の前提、欠落・不完全・判定不能は実行拒否まで」「保留解除後でも誤った再実測を訂正できる導線」。D-D4 の判定の土台（日付比較）が変わる案を含むため、未解決の設計質問ありとして design へ戻す。Plan Commit は pending のまま。案は「設計判断」節の D-D10 候補、owner 回答後に D-D4 / D-D8 / D-D9 / Scope / AC を同期して新しい rally で Plan Review。

引継ぎ（phase変更なし、2026-09-18）: owner「普通に君に設計任せる」を受領。反例ごとの条件追加を止め、時点証拠・復旧・保存順序をADRへ統合する。新しいsource契約はDB/command/保存・訂正の操作を直接定めるため、影響基準でRiskをR3へ更新しMatrixを作成する。旧rallyの評価はその対象commitに対する履歴として保持し、新案の承認には流用しない。

- design → plan-draft（2026-09-19、発注65のsource同期content commitに同乗）: ADRの意味を変えずにS1〜S6の詳細契約とS7の対応表を同期。合成モデル・docs検査・禁止pathと要求token照合が成功し、新たな設計質問はない。実機依存は成立まで非有効化とする契約を明記。正式Plan GateはFableが新rallyで発注し、Plan Commitは承認までpendingを維持する。
- plan-draft → plan-gate（2026-09-19、state-only）: packet と Test Design Matrix は content commit `9bed4f57` で commit 済み。同 commit で設計モデル・docs 検査（plan / full）・workflow-git が成功し、未解決の設計質問なし。正式 Plan Review の新しい rally（round 1、Sonnet + Opus の独立 fresh context、対象 `9bed4f57` の計画内容）を Fable が発注する。Plan Commit は pending のまま。
- plan-gate → plan-approved（2026-09-19、state-only）: 正式 Plan Review の新 rally は round 1（Sonnet 通過可 / Opus P2 4）→ round 2（P2 1）→ round 3（新規 P2 1、天井到達、owner escalation）→ owner 決定の反映（`b7f19514`）→ 追加 Opus closure で P1/P2 = 0・通過可。Plan Reviewer は Writer（Codex）と別 vendor の独立 fresh context。Plan Commit = 承認された計画の content commit `b7f19514`。owner は同日に介入上限 9 を承認。runtime は未着手で、本 lane は docs-only のまま Final Review → Ready → merge へ進む。本体実装は後続 runtime lane の Gate を満たしてから。
- plan-approved → implementing（2026-09-21、state-only）: 本 lane の実装対象は docs のみ。Plan Commit `b7f19514` の後の content commit は follow-up P3 の是正 `e6189a5c`（ADR Consequences の literal を戻す）だけで、Plan Commit はその祖先。Draft PR を開き、Final Review（Sonnet + Opus、Writer と独立した fresh context）へ進む。以後の状態は GitHub evidence mode の helper と専用 record から導く。runtime は未着手のまま。

## Owner Effort Budget

- 介入回数上限: `10`（owner 承認 2026-09-22。Final Review の round 天井で owner escalation が 1 回必要になったため。それ以前の承認済み上限は 9〈owner 承認 2026-09-19〉、その前は 8）
- 介入実績: 設計委任までの8/8に、2026-09-19のowner escalationへの決定を1回加え実績9回。上限9の承認により9/9。既に指示された本runの是正をまとめ、追加の介入を無断で発生させない。2026-09-22 に Final Review の round 天井の owner escalation への決定を 1 回加え、実績 `10/10`
- 実働時間上限: 60分（既存承認。ownerの実測作業時間は未実測）
- relay往復上限: 2（ownerを伝書鳩にせず、read-only reviewは担当が回収する）
- Plan Review round 天井: 3。source同期版はround 3まで消化（2026-09-19、対象 `a2a265bf`）、disposition = owner escalation。通常のround 4は開始しない。owner決定反映後の追加確認は1回（Opus closure、対象は発注68の是正差分）として実施済み（対象 `b7f19514`、P1/P2 = 0・通過可）

このturnの最小完了経路は、ownerが確認した初導入の前提と移行判断を正本・preflight・Matrixへ同期し、追加確認へ渡せる差分をまとめること。未承認の予算改訂やruntime実装へ広げない。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

remote order branchを使うconsultation relayは本runで使用しない。正式Plan Review round 3で天井に到達し、owner escalationを経た。発注68の決定反映差分はFableがOpus closureとして追加確認を発注する。初導入前提に限る同種点検を行い、正式レビューの代行や新しいbroad/rallyにはしない。

## Risk

Risk: R3

Reason: 現在のdiffは設計文書と合成probeのみだが、定める契約はDBの永続化・POS取込み・command token・棚卸し保存・訂正・移行に及ぶ。Riskはfile種別でなく影響で判定する。runtime lane㉘もR3で、別のplan-first commitと必要な実装検証を持つ。

## Goal

Goal Invariant: 実測より後の記録済み入出庫を棚卸し確定で消さず、実測へ含まれた後着売上を二重に反映しない。アプリが検知できる状態変化は保存前に止め、前後を証明できない取込みは再確認へ回し、誤った実測は後から現在の現物で訂正できる。提供されていない事実を正しいと推測しない。

### 最小完了条件

- 実測・受領・精算区間・在庫ledgerの意味と前後判定が、ADRとsourceの具体的な保存・関数契約で追える。
- 通常、初回、時計不明、EJ不完全、同秒、取消、途中再開、旧DBについて拒否と復旧が定義される。
- 設計モデルが主要な数値反例を再現し、採用案のassertが通る。
- Matrixがruntimeで必要な検証と、モデルでは証明できない境界を分ける。
- 本書・Plans・source docの詳細が、現行実装と提案を混同させず、保存先から画面の回復までを接続する。

### 失敗定義

未検証の時刻へ仮値を入れる、安全を任意の60分や確認チェックで保証する、同秒の順序をtimestampで決める、旧snapshotを新方式へ読み替える、Plan Gate前にruntimeを書く、未実施のレビュー・実機検証を完了と表現すること。

### 非目的

棚卸し確定の取消、過去評価額の自動訂正、部門売りの商品配賦、DATA-2全体の解消、実POSデータのgit格納、実機への書込み、Ready/merge。

## 設計判断と保持する合意

現在の詳細はADRのSPEC-STK-TIME-D1〜D9を正とする。

| 既存判断 | 現在の扱い |
|---|---|
| D-D1 / D-D2 | 実測時snapshot、補正N-Lを維持。計数contextと版・cursorを加える |
| D-D3 | カウント時在庫・入力値・差を維持。古い入力を現在の実測として再送しない |
| D-D4 | 日付/保存時刻だけの比較を廃止する案。受領順と証明済みの時間範囲で分類 |
| D-D5 | 最初の有効な吸収先を補正する方針を維持。秒精度timestampでなくledger cursorと実測順序 |
| D-D6 / D-D7 | durable判断はsourceへ。過去評価額は非遡及、現在庫復旧と区別 |
| D-D8 | 要再確認は永続化し、force_fillで迂回不可。ゼロ行も検査 |
| D-D9 | 商品ごとの独立再実測を維持。取消で消さず、過去の記録からも訂正可能 |
| D-D10 | 仮始端・一律60分・大差閾値を安全条件にする案は不採用候補。ADRの証拠・拒否・復旧に統合 |

共有JANは行単位で先に検査し、全在庫連動候補が実測前と証明できる場合だけ在庫を全スキップしてcommit可能とする案。未実測の候補へ先頭配賦してから他の商品だけ再確認する経路は採らない。参照できる棚卸し明細がない候補の手軽な解除は現行の限定APIでは未対応であり、DATA-2側との接続が残る制限。架空の明細や自動の全店棚卸しで隠さない。

## Scope

source同期で確定したS1〜S7を、発注68（着手基準 `a2a265bf`）のowner決定反映へ適用し、S7へproject-memoryを追加する。ADR D3 / D8 / Consequencesと展開先へ、初導入・本番履歴なし、新形式での本番開始、旧取込みのメタ不足拒否と日付つきpreflightを同期する。proposed節を点検し、旧DBの互換機構は維持したまま適用前提・費用を区別する。合成モデルは、Gated Amendment 1（Final Review 初回 broadの是正）が列挙する箇所に限って変更する。それ以外は変更しない。機構削除や別の設計判断が必要なら停止してCoordinatorへ返す。

- S1（IO）: `docs/function-design/20-io-product-repo.md`（数量の汎用更新撤去、実測・再実測・flag・cursor取得）、`21-io-inventory-repo.md`（数量と版の不可分更新、補正区分）、`23-io-z004-parser.md`（精算メタ・ゼロ行）、`24-io-csv-import-repo.md`（受領記録、取消movement ID）。いずれも同じfunction-designディレクトリ内。
- S2（BIZ）: `docs/function-design/30-biz-product-service.md`（設定変更・共有JANの検査）、`31-biz-inventory-service.md`（版更新の呼出し側）、`32-biz-csv-import-service.md`（受領・分類・保留・取消・外部probe条件）、`35-biz-stocktake-service.md`（context・即保存・確定・訂正）、`36-biz-integrity-check.md`（fix_integrityの版更新）。
- S3（CMD）: `docs/function-design/40-cmd-product.md`（共通CmdErrorの新しい回復payload）、`41-cmd-pos.md`（preview/commit/取消のwire）、`42-cmd-sales-stocktake.md`（begin/save/abandon、DTO・error・登録義務）、`43-cmd-settings-log.md`（復元時のDB接続交換前のcontext失効）。
- S4（operator）: `docs/function-design/51-ui-product-form.md` / `60-ui-product-import.md`（発注66 P3-5: 商品マスタwrite guardの拒否表示・入力保持）、`55-ui-csv-import.md`（保留・再preview）、`65-inventory-record-traceability.md`（補正区分・訂正への到達・評価額）、`73-ui-stocktake.md`（計数開始・保存・回復・native検証）。
- S5（DB）: `docs/db-design/master-tables.md`（stock_revision）、`transaction-tables.md`（既存入出庫TXと版の関係のみ）、`pos-tables.md`（source受領とimport）、`tracking-system-tables.md`（実測種別・証拠・再実測・flag・補正区分・migration）、`docs/DB_DESIGN.md`（proposed保存契約への索引）。
- S6（境界・親文書）: `docs/ARCHITECTURE.md`、`docs/architecture/biz-task-specs.md` / `io-task-specs.md` / `cmd-task-specs.md` / `ui-task-specs.md` / `mnt-task-specs.md`（各層の責務と詳細契約への参照）、`docs/FUNCTION_DESIGN.md`、`docs/SCREEN_DESIGN.md`、`docs/UI_TECH_STACK.md`（到達導線、状態所有、失効と既存invalidation契約の接続）。
- S7（引継ぎ）: 本packet、`docs/plans/test-matrices/2026-09-18-stocktake-time-evidence.md`、`docs/Plans.md`、`docs/project-memory.md`（発注68: owner回答の要旨・確認日付きの初導入前提と本番開始時の更新義務。Gated Amendment 2: owner発言の原文は公開repositoryに置かない）。Required Design Artifacts / Contract Coverage Ledger / Trace Matrix / runtime申し送り・現在地をsource本文に同期する。Gated Amendment 1で `docs/backlog.md` のSTK-1 / STK-2 entryの現在地同期と、`scripts/probes/stocktake_time_model.py` の限定是正を追加する。

S1〜S6の各sourceに「時点証拠契約（proposed・未実装）」を区別して置く。project-memoryはownerが確認した現在の店舗事実を記す。現行のシグネチャ・schema・UIが既に新方式になったとは記載しない。関数設計の新規fileは作らず、既存file内で詳細化するためmodule-map登録の追加はない。実装・生成物・新しい要求IDの追加は行わず、既存要求tokenの増減が必要なら停止して再発注を求める。

波及候補の裁定: 30は商品一括importも汎用更新を通るため採用、31は共通数量更新のcallerなので採用。43のfix_integrity自体は42への参照だけだが、実際のDB接続交換が43.9にあるためD1の失効を接続する目的で採用。DB masterは版の保存先、transactionは既存入出庫のTX契約との接続として採用する（header/item schemaは変更しない）。71の復元本体の復旧規則・error kindは変更せず、43から既存正本へ委譲したままにする。decision-logは非編集。ADRはD3 / D8 / Consequencesへのowner決定同期を許可し、合成モデルは検証入力として保持する。

隣接契約の追加: CmdErrorの共通所有先は40 §5.3であり、現行はkind/message/field/error_idだけで回復対象を運べない。D8の機械判別とD9の対象商品への案内をmessage解析で代替しないため、40をS3へ追加してから編集する。既存kind・restoreの分類は維持し、新しい回復payloadはproposedとして区別する。

## Non-scope

- src-tauri、src、migration、bindings、traceabilityの変更。
- GitHub操作、実機操作、実データの読取り拡大・複製。
- workflowそのものの変更、過去rallyの書換え、無関係なcleanup。

## Acceptance Criteria

source詳細同期のAC。件数一致だけを契約充足の代用にしない。

- AC1: `rg '^### SPEC-STK-TIME-D' docs/adr/2026-09-18-stocktake-time-evidence.md` の出力にD1〜D9がある。実測窓、資料受領、未知の境界、保留と再確認、訂正、取消、legacyの意味は独立設計点検で確認する。
- AC2: `python3 scripts/probes/stocktake_time_model.py` がexit 0でPASSを出力し、before/afterの断定が合成oracleに反しない。モデルのPASSは実装テストと別に扱う。
- AC3: S1〜S6のsourceを `rg -n '時点証拠契約|SPEC-STK-TIME'` で辿れ、DBの保存条件→IO入出力→BIZのTX/拒否→CMDのwire→UIの回復の対応がContract Coverage Ledgerから追える。各sourceはproposed・未実装と現行本文を区別する。`rg -n '2026-09-19|本番|初導入' docs/project-memory.md` で確認日・owner回答の要旨・本番開始時の追記義務を確認し（Gated Amendment 2: 原文の存在は条件にしない。初導入の前提を開発/試験データの削除許可と読ませない但し書きがあることも確認する）、`rg -n 'project-memory' AGENTS.md docs/PROJECT_HANDOFF.md` で入口から辿れることを確認する。
- AC4: `git diff --name-only 36891af8 -- src src-tauri` の出力が空。全変更がScopeのdocsに属する。`git diff 36891af8 -- docs/function-design/90-traceability.md docs/DEV_WORKFLOW.md docs/AGENT_OPERATING_MANUAL.md docs/templates` も空。`scripts/probes/stocktake_time_model.py` の差分はGated Amendment 1が列挙する是正（取消補償の商品別純量化と対応case、検出力のないassert・コメントの是正）だけで、隔離copyへのmutation（是正前の明細ごとの補償へ戻す）でassertが落ちることを確認する。変更docごとの要求tokenを基準版と比較して同一であることを確認する。
- AC5: bash scripts/doc-consistency-check.sh --target plan とfullがERRORなし、git diff --checkが成功。
- AC6: `32-biz-csv-import-service.md` の外部probe表が、精算系列・EJ完全性・商品同定・時計対応の観測項目、許可条件、不成立時の動作を持つ。`42-cmd-sales-stocktake.md` と `73-ui-stocktake.md` にWindows監視・native検証の失敗条件を持つ。未実施の実機検証をpassと扱わない。
- AC7: `git diff --check` と `bash scripts/check-workflow-git.sh` が成功する。Review Responseへfinding別の是正先・初導入前提の同種点検・検証を追記する。51/60のstocktake_guardとMatrixの商品master各write oracleを維持し、Matrixには移行前importの拒否・日付つきpreflight・新形式の後追い成功を追加する。Writerの是正runはWorkflow Stateのfieldを変更しない（発注68時点の「plan-gate / Plan Commit pendingを維持し、介入上限の変更案はowner承認待ちと記す」は当時の指示で、Plan Gate通過後は適用しない。Gated Amendment 1）。

## Design Sources

- 要求: REQ-205 / REQ-401、docs/spec/requirements.md / requirements-coverage.md。
- 新案: ADR SPEC-STK-TIME-D1〜D9（proposed）。
- 詳細同期: Scope S1〜S6の各sourceの「時点証拠契約（proposed・未実装）」。現行本文は保持し、変更予定との差分を同じ文書内で示す。
- 境界: UI → CMD → BIZ → IO/MNT、INV-2、D-051、D-025。
- 旧案の事実調査: feeb3fe9の起票時実測、承認済みEJサンプルの匿名化構造所見。raw値は持ち込まない。

## Required Design Artifacts

| Area | Artifact | Status |
|---|---|---|
| 時点・順序・復旧 | ADR D1〜D9、32 / 35 | メタ不足拒否を維持し、D3 / D8へ初導入と本番開始条件を同期 |
| 店舗の導入前提 | project-memory | owner確認日/回答の要旨、本番履歴なし、本番開始時の更新義務を正本化。AGENTS / PROJECT_HANDOFFから到達可能 |
| DB / TX / migration | master / transaction / pos / tracking、20 / 21 / 24 / 36 | 論理列・型・制約・保存順・旧DB分類をproposedとして追加。SQL適用と故障注入はruntime |
| CMD / DTO / token | 40 / 41 / 42 / 43 | API/DTO・回復payload・保管/失効・再送・登録と生成の義務を具体化。未実装 |
| operator flow | 55 / 65 / 73、SCREEN_DESIGN / UI_TECH_STACK | 到達・focus・状態・中断/再開・差0商品の訂正・native合格条件を追加。UI実装は不変 |
| POS / EJ / 時計 | 23 / 32 / 42、MNT task | 外部probeの観測項目・許可条件・不成立時の出口を列挙。実機は未実施 |
| 検証 | Matrix、既存合成モデル | source別のruntime検証と本runのdocs/モデル検証を区別 |

## Registration / Generation Obligations

このturnは新CMD・Rust関数・schemaの実装なし。bindings / route tree / traceabilityは生成しない。runtimeではcommand登録、生成binding、schema migration、REQ test traceability、record detailのkind伝播を必須とする。

- runtimeの公開command追加はbegin/save/abandonとget_pos_stock_readiness。旧update_countの公開登録を外し、tauri/specta属性・collect_commands・bindings・caller/mockを同じ変更で更新する。
- 共通CmdErrorのstocktake_guard/回復payloadに合わせ、全constructor・generated enum・CMD_ERROR_KIND・unwrapResultの保持・describeError・UI分岐/fixtureを同期する。
- function-design fileの新設・改名なし。module-mapやroute登録の追加は本runでは不要。要求tokenの増減なしを基準版と照合し、90-traceabilityは手編集も再生成もしない。

## Design Intent Trace

| Spec | Source / decision | Why | Future implementation | Verification |
|---|---|---|---|---|
| REQ-205 | D1 / D7 / D8、20/21/35/40/42/43/65/73、master/tracking | stale入力、同秒、旧snapshotを誤適用しない | BIZ-06 / IO / CMD-10 / UI-10 | Matrixの保存・訂正・migration |
| REQ-401 | D2 / D3 / D4、23/24/30/32/41/55、pos | 知らない前後を推測しない | IO-02 / BIZ-03 / CMD-07 / UI-07 | Matrixの受領・区間・0行・共有JAN |
| REQ-205 / REQ-401 | D6、24/32/35、tracking | 実測へ吸収した数量を二重に戻さない | rollback / stocktake repo | 合成モデル、runtime TX検証 |
| REQ-401 | D5、23/32の外部probe表 | 不完全なEJで自動分割しない | 別EJ lane | 合成fixture、sanitized sample、native gate |

## Design Intent Audit

- 現行実装とproposed設計を明示して区別する。
- durableな新判断はADRに置く。packetはscopeと状態を所有する。
- D-D1/D-D2の算術とD-D5の吸収先はモデルで検算し、任意の値へoracleを寄せない。
- 実POSの時刻・完全性・計数そのものをアプリが直接観測したと主張しない。

## Impact Review Lenses

- Adapter/core: IOはraw metadata、BIZは証拠・時点分類。未知形式を正常扱いしない。
- Fact/decision: sampleの形状は確認済み。精算reset系列と実時計の適用範囲は未確認。
- Lifecycle/retry: 受領・preview・保留・計数・保存・中断・取消・再取込みをMatrixで追う。
- Operator: 商品ごとの即保存と、失敗後の次の一手。色だけで状態を示さない。
- Replacement: EJやレジが変わっても、受領証拠と実測窓のBIZ契約を維持。
- Data safety: 合成モデルのみ。実データ・DB・backupを使わない。
- Reporting: 現在庫の復旧、売上欠落、過去評価額を区別。
- Manual: UI-07/UI-10/記録詳細のnative L3と、POS時刻・系列probeは未実施。

## Design Readiness

Status: implementing（2026-09-21）。plan-approved時点の評価: source同期版は正式Plan Reviewの新rally（round 1〜3、owner escalation、追加Opus closure）を経てP1/P2 = 0で通過した。初導入・本番履歴なしを正本化し、旧importの拒否と日付つきpreflightを同期済み。D1のOS監視必須は維持する。これは設計と計画の承認であり、runtime実装ready・実機成立を意味しない。

新たにownerへ返す設計質問はない。未検証の外部条件は、32のprobe表で対応する自動処理を有効にしない条件として明示した。Windows監視の実装/native probe・EJ全形状・レジ系列/時計の本番成立は未実施で、正式Plan Gateでは依存する前提の検証計画と非有効化条件を確認する。明細のない共有JAN候補は既知の非対応として保持し、店舗運用の受容済み扱いへ変更しない。

## Contract Probe

- 合成モデル: python3 scripts/probes/stocktake_time_model.py。実行結果はReview Responseに記録する。
- 既存実装確認: movements.idはDBで採番、同秒timestampでは順序不能。snapshotとcursorの同TX保存が必要。
- 外部: Z004のメタ項目、EJの分精度・取引番号・点数欄・精算境界を既存所見で確認。精算番号のreset系列・時計差の保証期間・EJ全形式は未検証。これらが証明されるまで対応する自動分類を有効にしない。
- 安全なfallback: 資料を受領してからの新しい実測。仮時刻、任意の時間幅、EJ合計だけの安全宣言は使わない。
- 詳細なprobeの正本: 32「外部probeと本番条件」、42のWindows/native境界。系列・時計対応・EJ完全性・取引復元・商品同定・OS監視の観測と不成立時の動作を対応させた。実POS/DB/backupへの新規アクセスは本runで行っていない。
- source接続probe: 既存CmdErrorは回復対象を持たず40が共通所有先、DB接続交換は43.9に存在、汎用数量更新は20のProductUpdates経路に存在。各sourceと該当codeの読取りで確認し、追加/撤去予定を相手側と同時にproposedへ記した。

## Test Design Matrix

[Test Design Matrix](test-matrices/2026-09-18-stocktake-time-evidence.md)。対象契約、失敗条件、状態遷移、モデルとruntime検証の境界を記録する。

## Contract Coverage Ledger

詳細は [Test Design Matrix](test-matrices/2026-09-18-stocktake-time-evidence.md)。

| Contract | Future target | Automated evidence | Native / external |
|---|---|---|---|
| D1 数量/版（20/21/30/31/35/36、master/transaction） | 共通repo、既存数量writer、数量なしの状態更新 | req205_stock_revision_all_writers / req205_stock_revision_non_quantity / ABA | operator手動故障注入は対象外 |
| D1 context（35/42/43、MNT task） | begin/save/abandon、DB/環境世代、PC時計epochの発番・保存、保存済み要求照会 **時刻経路は確定していない・㉘の実装対象外**。 | req205_count_save_idempotency / count_context_invalidation / req205_count_context_db_generation / req401_pc_epoch_across_basis_renewal | Windows監視probe、73の計数L3 |
| D2 受領（23/24/32/41、pos） | parser任意メタ、hash一意source、開始source_cursor | req401_receipt_before_count_start / req401_source_survives_zero_and_rollback | 実ファイル系列 |
| D3 時刻/精算同一性（23/24/32/41/55、pos、MNT task） | gate基準のpc_clock_epoch/file境界のproducer、実測epochとの比較、昇格/失効伝播、精算同一性guard、独立拒否証拠、settlement_missing **時刻経路は確定していない・㉘の実装対象外**。 | check_counterexamples / check_diagnostic_order / req401_settlement_identity_conflict / req401_same_day_missing_identity_rejected / req401_identity_conflict_recovery_limit / req401_time_evidence_validation / req401_time_evidence_expiry / req401_time_evidence_promotion / req401_file_bounds_producer / req401_file_bounds_not_derived / req401_file_bounds_propagation / req401_rejected_settlement_visible / req401_clock_invalidation_survives_rejected_commit / req401_pc_epoch_across_basis_renewal | 32の時計・系列probe |
| D4 分類/設定（30/32/35/40/41/51/55/60） | 共通分類、全候補、active/完了、preflight、master各writeの拒否と非連動化記録/回復 **時刻経路は確定していない・㉘の実装対象外**。 | req401_shared_jan_all_candidates / req401_active_legacy_import_recheck / req401_ambiguous_jan_sync_rejected / req401_master_sync_guard_ui / req401_sync_disable_retains_recovery / pos_stock_readiness_preflight | 準備不足と保留からの回復 |
| D4 ゼロ行/flag（23/32/35、tracking） | 売上行と証拠行の分離、確定拒否、再preview **時刻経路は確定していない・㉘の実装対象外**。 | req401_zero_net_nonzero_after_count / req205_recount_flag_blocks_complete | 売上0の正常完了表示 |
| D5（23/32） | 別EJ laneのadapter/照合/実行拒否 **時刻経路は確定していない・㉘の実装対象外**。 | req401_ej_coverage_boundaries / req401_ej_per_receipt_validation | 32の本番有効化probe |
| D6（20/24/32/35、tracking） | movement ID、最初の吸収先、legacy停止・専用再確認 | check_lifecycle / check_legacy_recovery / req401_rollback_count_cursor | 取消→復旧の可視動作 |
| D7（20/21/31/35/42/65/73） | 独立再実測、明示的kind、差0対象への到達、非遡及評価 | req205_recount_active_owner_guard / req205_stocktake_movement_kind | 73の確定後訂正L3 |
| D3 / D8 取込み移行（project-memory、pos、23/24/32/41/55） | 新メタ抽出→保存、本番前preflightの日付、旧importの拒否、後追い | req401_pre_migration_import_identity_guard / req401_import_identity_preflight_dates / req401_new_metadata_late_import_allowed | 初導入に旧本番履歴なし。旧試験DBの作り直しは別作業 |
| D8 DB（DB設計、20/21/24） | CHECK/FK/要求ID/内部上限、移行の全体rollback | check_migration_and_fill / req205_legacy_count_requires_recount | 旧データがある環境だけ件数/移行負担を確認。初導入には対象なし |
| D8 wire（40/41/42） | 新command/共通error/生成binding/旧入口撤去 | req205_recovery_wire_contract、runtimeのbindings/finite-enum検証 | UIはmessage解析しない |
| D9（55/65/73、SCREEN_DESIGN/UI_TECH_STACK） | focus・IME・再開・型付き回復・invalidation | runtime UI state/route/error/再送試験、D-052 consumer照合 | 73のL3、Windows以外で代用しない |
| D1/D6/D8/D9 旧指摘の継続 | 確定数量/評価額、商品別補償、同一release、実測/補完表示 | req205_complete_snapshot_and_valuation / req401_rollback_compensation_grouped / req205_count_schema_writer_atomic_rollout / req205_count_snapshot_labels / req205_count_monitor_unavailable | 旧testからの移行はMatrixの申し送りに従う |

## Test Plan

- 設計probe: Python標準ライブラリのみ。外部入力なし。
- docs: plan / full、diff --check、workflow-git。基準版36891af8との差分をScope・禁止path・要求tokenの観点で確認する。
- runtime test: Matrixの予定名。未実施であり、本turnでRust/frontend全量を回して実装済みの証拠にしない。
- read-only独立設計点検: 反例と最小修正を求める。正式Plan Gateと区別。

## Boundary / Wire Contract

- 現在のwireは変更しない。新案ではUIはopaque count tokenと数量だけを送る。
- source/revision/cursor/日時の任意指定をUIから受け付けない。
- previewは資料受領を保存し得るが、売上・在庫をcommitしない。
- 保存・取消・信用失効のTX、request重複、record kindのproducer/consumerはsourceへ展開済み。40/41/42の有限enumと回復payloadをgenerated bindingへ反映する実装はruntime packetへ渡す。

## Review Focus

- 受領IDを保存時に取り直していないか。
- 保存時刻Eだけでbeforeを判定していないか。
- 未知の始端、legacy、共有JANを通常適用へ落としていないか。
- 取消で吸収先を二度補正しないか。同秒と0差の実測を区別できるか。
- 現在庫の復旧を売上欠落・過去評価額の解決と言っていないか。
- 新機能の対象・migration・UI復旧を既存のscopeと混同していないか。

## 後続 runtime lane ㉘ への申し送り

2026-09-23 の改訂で本節の一部は、ADRの[㉘ への引継ぎ](../../adr/2026-09-18-stocktake-time-evidence.md#㉘-への引継ぎ)と、同節が示す改訂laneのTest Design Matrixに置き換わった。以下の本文は書き換えず、差分は同節を優先する。

発注書72のowner dispositionを以下の申し送りより優先する。線引きの正本は[ADRの適用範囲の但し書き](../../adr/2026-09-18-stocktake-time-evidence.md#適用範囲の但し書き)。既存の契約本文は保持するが、時刻経路を㉘の実装仕様として確定した扱いにはしない。

- **㉘で実装しないもの**: gateによるPOS基準の認定、pos_time_bases、TimeBasis / TimeEvidenceの保存とtyped decode、file境界導出、sourceの昇格 / 失効の伝播、PC時計epochの発番・保存と一致条件による時刻比較、EJ時刻分割、clock_unverifiedの準備issue。同じapp sessionでしか成立せず、次のdesign laneで置き換わるため。
- **㉘で実装するもの**: 商品単位の即保存（begin / save / abandon、context、revision、冪等な再送）と数量・版の不可分更新。受領記録とhash・単調cursor、精算メタの抽出・保存、精算同一性guardと拒否記録（settlement_missing）。受領順によるBefore、NeverObserved / LegacyObservedの扱い、判定不能からの保留と再実測。商品別の純量による取消補償、訂正、共有JANの検査、非連動化の記録（pos_sync_disabled_revision、sync_disabled_unreconciled）。migrationとlegacy分類、初導入の前提、UIの回復導線。計数中のOS監視・generationとcontext失効も維持する。
- **残る運用費用**: 時刻でAfterと判定できないため、過去に実測した商品を含む資料は受領順のBefore以外は判定不能となり、保留 / 再実測へ落ちる。active所属の通常適用と要再確認flag、完了済み / 独立再実測所属のfile全体保留という既存の分岐を維持する。NeverObservedの商品は共有JAN等のguardを満たせば通常の在庫連動となる。これだけではv1.0の恒常運用（日次の自動在庫連動）として足りない。
- **同名別物の保存列**: 実測側time_basis_idはTEXTのepoch、source側はINTEGERの基準FKで別物だが、どちらも㉘では列を作らず、次のlaneが保存形を決めるときに同名を避ける。
- **次のdesign lane**: 実測とPOS系列の対応を取得・保存する（design-first、R3）。実機確認を着手条件とし、(a)精算系列の意味（Z004のリセット範囲、同日複数精算、番号リセット、再出力・訂正版の識別）、(b)番号印字のEJ記録と取引一連番号・時刻との並び、売上・Z004への副作用の有無、(c)レジ時計を進める / 戻す操作のZ004 / EJへの痕跡、(d)棚卸し→アプリ終了→OS再起動→翌日の遅延・逆順取込みで、境界確定済みの商品に再計数を求めず実測後の販売だけが一度減るか、を確かめる。番号印字による接続は未検証の候補であり採用決定ではない。EJ parserの構造復元は並行して進められるが、時刻分割の実装は次のlaneの結論を待つ。

旧案のfile:line指示はfeeb3fe9の履歴。新しいruntimeのscopeはADR、S1〜S6のproposed詳細、Matrixから起こし直し、別のplan-first commitとGateを持つ。

- source受領の永続化、productsのrevision、count/recountの開始・終了・cursor・request識別子、明示的movement kind。
- 本番は初導入であり、project-memoryの確認済み前提を使用する。識別メタ（マシンNo.・精算回数・精算日時）の抽出→受領→保存の実装・検証が本番開始の前提で、列追加だけでは完了しない。メタなし旧Z004試験履歴を本番へ持ち込まず、開発/試験DBの作り直しは残すマスタ確認後、売上・在庫移動と整合した単位で別作業とする。取込み履歴の表だけ消す案は不可。本番開始時はproject-memoryの「Store Premises Facts」へ開始日と変わった前提を追記する。
- preflightのimport_identity_missing + settlement_datesをIO→BIZ→CMD/bindings→UIへ配線し、移行前sourceなし/メタ不足のactive importを全日付から検出する。該当日の比較先拒否を確認操作で迂回しない。新形式どうしの過去日追加は他のguardを満たせば通し、更新前の全取り漏らし解消や後追い禁止を条件にしない。旧観測・legacy取消の互換機構とテストは維持する。
- 数量更新の版増分はinventory_repo::update_stock_quantity内部で強制する。既存caller = inventory_service/common.rs、csv_import_service/commit.rsの取消戻し、stocktake_service.rsの確定補正、integrity_service.rsのfix_integrity。未使用のProductUpdates.stock_quantity分岐も撤去し、数量以外の基準・flag・設定・純量0取消・差0確定は同TX内で共通の版更新処理を使う。Matrixで各経路・overflow・TX rollbackを確認する。
- 旧明細はuncounted / auto_filled / legacyへ分類し、旧force_fillはlegacyのまま。0/0の廃番自動入力をlegacy件数へ加算しない。既知producerに一致しない形は保守的にlegacyとして表示する。
- schema migration・全writerのkind/証拠対応・無検査update_count撤去・新UIは同じruntime配布単位。DB laneだけの先行出荷/稼働を禁止し、ALTERの恒久DEFAULTで旧writerを通さない。実装commitを分けることと中間版を運用することを区別する。
- BIZ内の計数context検査、同一判定関数を使うpreview/commit、取消の吸収先探索。
- 精算同一性guardは同日active全件を取得し、対象sourceまたは比較先sourceのmachine_no / settlement_noの片方でも欠ければ追加確認trueでも業務write前に拒否する。メタNULL/source取得不可を候補なしにしない。activeなしの例外で全受領sourceの別hash衝突検査まで省略しない。衝突版の採用決定・メタ補完は非対応で、訂正版や系列未証明resetの拒否も含め、現在庫の再実測では未取込み売上を復旧しないことをUIへ伝える。
- UI-07/UI-10/記録詳細の即保存・再開・訂正、既存IME/Enter/focus/returnToを維持。
- legacy migration、全行0、共有JAN、逆順資料、時計異常、EJ欠落を負の経路として実装前にfixture化。
- EJはPLU本番前提。取得失敗で再実測の復旧まで閉ざさず、自動分割の許可と区別する。
- **time_basis_idと時計失効は確定していない・㉘の実装対象外**: 新たに具体化したtime_basis_id、型付き回復payload、準備照会、補正kind/recount参照をproducerから全consumerへ配線する。時計失効は業務rollbackで消さない。legacyのUnknown後は所属で一意に分岐する。
- 各sourceの「現行本文」を新runtimeの完成形として混ぜず、対応するproposed節を実装し、旧仕様からの置換箇所を実装差分で確認する。profile/現行図面・D-052の実装SSOTはruntimeの挙動変更と同時に同期する。
- 既存green testと既知ignore診断はMatrix「既存テストの移行先」を正として移す。計数前の在庫変更で差3は維持し、保存後の移動で差を固定する。負在庫force_fill、L≠liveのformatter/画面、確定時の評価数量、商品別SUM補償を値で検証する。未実測「—」/自動入力の注記は一覧・選択商品・保存結果のoracleへ明記する。
- **時刻部分は確定していない・㉘の実装対象外（契約本文を保持）**: Final Review是正のproducerを省略しない。gateはpos_time_basesの基準認定、BIZ-03はsettled_at/精度と同系列前回sourceからfile境界を導出してsourceへ保存する。基準認定/前回後着の適格sourceだけ昇格、失効は基準と全参照sourceへ伝播。通常操作から基準の認定は不可。実測側time_basis_idはMNTが発番するPC時計epochのTEXTとし、BIZ-06がbeginで固定、save TXで一致検証してitem/recountへ保存する。POS基準の既存TimeBasis JSONへpc_clock_epochを保存し、基準JOINで現在epoch/実測epochと比較する。source側time_basis_idだけがINTEGER FK。同じepochなら期限更新/reset後も既存実測と比較可能で、POS期間を実測窓へ課さない。epoch不一致/NULLは時刻Unknown、受領Beforeは維持する。
- 欠落表示は拒否された未取込み資料だけ（案a）。同一性拒否証拠を業務TXと独立保存し、PosStockReadiness.issuesのsettlement_missing/source_ids/settlement_datesへ接続する。未提出/欠番全般は本laneで検出せず、issueなしを全資料完備と表示しない。source単位のissueでID/精算日を各1要素にし、そのsourceの理由説明と組にする。再実測/再起動では消さない。
- pos_stock_syncは変更前値の取得・true→false判定・版/切替記録の保存を単一の共通repo書込み経路で強制し、汎用更新の直接set分岐を残さない。true→falseは`products`の`pos_sync_disabled_revision`を同TX保存し、非連動/廃番もsync_disabled_unreconciledの検出対象にする。商品get/updateのsync_disabled_recoveryと商品一括importの対象Vec、stock_review/ImportResult、準備照会から既存CountRecoveryTargetへ接続する。active保存だけでは未調整を残し、切替後の適用済み新実測で解消する。未調整issueを商品単位にし、切替後のmeasured pendingだけ確定を案内する。それ以前の入力は数え直しを案内し、51/73へ既存準備照会の説明を渡す。無明細には棚卸し開始を案内し、既存flag・再有効化guardを維持する。
- 本packetのdesignはruntime実装許可ではない。

## Spec Contract

Contract ID: SPEC-STK-TIME-EVIDENCE

- 根拠を持つbefore/afterだけを確定し、unknownを推測で埋めない。
- 実測は商品単位の即保存。任意の古い値の再送で新しい基準を作らない。
- snapshot差分は後続movementを保存し、取消は吸収済み数量を二重に戻さない。
- 復旧の新しい現物確認は過去の売上・評価額を自動で書き換えない。

## Trace Matrix

| Spec | Scope | Evidence | Review |
|---|---|---|---|
| SPEC-STK-TIME-EVIDENCE | S1〜S7 | AC1〜AC7、Matrix、source読取り点検 | 層間接続、時点・順序・復旧。時刻経路は確定していない・㉘の実装対象外。 |
| REQ-205 | S1 / S2 / S3 / S4 / S5 / S6 | AC1 / AC2 / AC3 / AC4 / AC5 / AC7 | snapshot・訂正・legacy。時刻経路は確定していない・㉘の実装対象外。 |
| REQ-401 | S1 / S2 / S3 / S4 / S5 / S6 | AC1 / AC2 / AC3 / AC4 / AC5 / AC6 / AC7 | 受領・区間・EJ・共有JAN。時刻経路は確定していない・㉘の実装対象外。 |

## Data Safety

このturnは合成値のみ。実POSファイル・DB・backup・secretを読み込むprobeは作らない。外部reviewへ渡すのは設計・合成モデルだけ。設計案はローカルのcontent commitに記録し、push・PR操作・実機の変更は行わない。

## Implementation Results

未実装。設計モデルのPASSをruntime completionと呼ばない。


## Gated Amendments

### Gated Amendment 1（2026-09-21、Codex 発注書 69 run 1 の着手前停止）

- 経緯: Final Review 初回 broad（対象 `552a7be7`、pass A Sonnet / pass B Opus）で pass B が P2 を挙げ、Coordinator（Fable）が全件 accept。うち P2-1（取消補償が商品別純量で1本になっていない）と P3 2件は `scripts/probes/stocktake_time_model.py` の是正を要する。発注書 69 run 1 は、Scope の「合成モデルは変更しない」と AC4 のモデル差分が空という条件に衝突するため、Writer（Codex）が編集前に停止した（正しい挙動。HEAD `552a7be7` のまま、tracked tree clean）。
- 原因: Coordinator が発注前に Scope / AC4 を読まず、モデルの是正を Scope 内と誤認した。
- 変更（本 commit、packet のみ）:
  - Scope: 合成モデルの変更を、本 Amendment が列挙する箇所に限って許可する。S7 へ `docs/backlog.md` の STK-1 / STK-2 entry の現在地同期を追加する。
  - AC4: モデルを差分が空の対象から外し、差分が本 Amendment の是正に限られることと、隔離 copy への mutation で assert が落ちることを条件にする。
  - AC7: 発注68時点の「plan-gate / Plan Commit pending を維持」の指示を当時のものと明記し、Writer の是正 run は Workflow State の field を変更しない、へ置き換える。
- モデルで変更してよい箇所: `cancel_import` の補償を商品別純量で吸収先へ1回だけ適用する形へ直す / 純量0と純量非0の適用済み吸収先 case を足す / 未知の始端への仮値挿入を落とす assert を足すか Matrix の Mutation-style Adequacy Questions から当該項を runtime test へ割り当て直す / 常に真になる assert 2件とコメントを、検査できる主張へ直す。
- 不変: Risk、Execution Mode、Final Review Minimum、Human Gate、Plan Commit、SPEC-STK-TIME-D1〜D9 の決定の意味、Non-scope。ADR・source docs・Matrix の是正は従来の Scope S1〜S7 の内側。
- 影響: Amendments の範囲が変わるため、是正後の head に対して Final Review の broad 2本を取り直す（`552a7be7` への broad 2本は finding の出所として保持する）。

### Gated Amendment 2（2026-09-21、owner決定「発言の原文を公開repositoryに置かない」の反映）

- 経緯: owner は 2026-09-21 に、店の事実についての自分の発言原文を外から見える場所へ置かず、要旨と確認日だけを置くと決めた（PR #84 で `docs/project-memory.md` の「Store Premises Facts」へ適用済み）。発注書 70 の是正は pass A の P3（同じ事実の二重所有）に従い `docs/project-memory.md` の Initial deployment baseline 行を同節への参照へ縮約し、結果として原文が正本から消えた。GA1 後の broad（対象 `7de29376`）で pass B が、Scope S7 と AC3 が「owner原文」を条件にしたまま不成立になっていること、claimed validation が成立していないことを P2 として指摘した。
- 裁定（Coordinator = Fable）: 指摘の事実（AC3 / Scope S7 / claimed validation の不成立）は accept。是正の向きは reviewer 案（原文を正本へ戻す）と逆で、owner 決定に合わせて条件のほうを改める。pass B の発注文にこの owner 決定を書かなかったのは Coordinator の漏れ。
- 変更（本 commit、packet のみ）:
  - Scope S7: `docs/project-memory.md` の内容物を「owner回答の要旨・確認日付き」へ改める。
  - AC3: 確認対象を「確認日・owner回答の要旨・本番開始時の追記義務」へ改め、初導入の前提を開発/試験データの削除許可と読ませない但し書きの存在を条件に足す（縮約で落ちた旧文 `This baseline is not permission to delete development/test data.` に相当。pass A の新規 P3）。
  - Review Response に残っていた owner 発言の原文 1 箇所と、原文に近い引用 1 箇所を要旨へ置き換えた。過去 rally の記録の書換えは通常しないが、公開範囲についての owner 決定を優先する。要旨化した以外の記録（当時「owner原文付きで正本化した」と書いた行を含む）は当時の事実として残す。
- 不変: Risk、Execution Mode、Final Review Minimum、Human Gate、Plan Commit、SPEC-STK-TIME-D1〜D9 の決定の意味、Non-scope、GA1 の内容。
- 影響: Amendments の範囲が変わるため、是正後の head に対して Final Review の broad 2本をもう一度取り直す。Final Review の rally としては 3 回目で、DEV_WORKFLOW の round 天井に当たる。そこで P1/P2 が残った場合は次の round を開始せず、同型指摘の一括是正 / backlog 化 / owner escalation のいずれかへ disposition する。

## Review Response

- Findings Freeze: 2026-09-19のsource同期版Plan Review round 1（対象63d7507d、Sonnet + Opus）で指摘集合を固定。次は本指摘のclosure確認。旧統合案broadは下記履歴として保持。post-freeze exceptions: none.

### 統合案broadへの対応（2026-09-19、design、closure未確認）

- ownerから受領したread-only結果: 対象4319fe36、SonnetはP1/P2なし・P3あり、OpusはP2ありのためpassでない。原文はcanonical checkoutのignored `.local/reports/stocktake-time-evidence/design-review-2026-09-19.md`。早期点検や今回の修正を正式Plan Gateの承認へ読み替えない。
- Opus P2-1（bug / 契約不足）採用: D1/D8で数量更新と版増分をupdate_stock_quantityへ集約。現行callerを検索し、共通入出庫・取消戻し・確定補正・fix_integrityを確認。関連するProductUpdates.stock_quantityは業務callerで未使用だが迂回可能なので、runtimeで撤去する契約を追加。数量を変えない状態更新の版増分も明記し、Matrixで別に検証する。
- Opus P2-2（migration drift）採用: start_stocktakeの0/0自動入力、insertでcounted_at省略、通常保存と旧force_fillでcounted_at設定を確認。`rg -n 'counted_at' src-tauri/src/db/schema_v{1,2,3,4,5,6}.rs` → schema_v1.rsの列定義だけ。登録済みmigrationでの後付けなし。通常形は3分類、矛盾形はlegacy + 表示。auto_filled自体は基準にもlegacy件数にも使わないが、別の旧実測は隠さない。
- Opus P3-1（復旧負担）一部採用: 見え方は明記、pendingだけで旧取消を解除する案は不採用。10→販売-2→旧実測8確定→新pending8では、通常取消直後の現在庫が10になる。後の確定で8へ戻せても、その間の誤在庫を許可しない。再確認の差は再実測区分で残し、N/N明細を確定差異件数へ水増ししない。
- Opus P3-2/3/4（lifecycle / drift）採用: 保留は同file再選択・再previewで再生成し、新tableを増やさない。受領順と時計の矛盾を早期return前に検査して時計対応を無効化し、受領による復旧は維持。復旧導線は未計数も含め進行中明細を優先する。
- Opus P3-5（test gap）採用: pending→complete→cancel、force_fill、計数側の粒度拡張、純量0取消、独立再実測の取消耐性、migration種別、未計数activeのlegacy復旧を合成モデルへ追加。SQL/Tauri/UIやflag永続化はモデルの証明範囲外としてMatrixへ残す。
- Opus P3-6（過剰設計候補）は削除不採用、理由追記: header世代は確定時の式・表示の意味を保持し、後から変わる明細kindでは代替しない。abandonはUIの切替・離脱をBIZへ伝える失効操作であり、timeoutと異なる。
- Sonnet P3-1/2（表記drift）採用: MatrixのSUM不変条件はD-051、INV-2は層の責任と分離。REQ-401の経路はCMD-07を明記。
- 検証: `python3 scripts/probes/stocktake_time_model.py` → `PASS: temporal bounds, causal receipt, clock conflict, zero-net split, revision, count/rollback lifecycle, legacy recovery, migration kinds, force_fill`。`runpy.run_path`で読み込んだモデルの関数をメモリ内だけで置換し、auto分類喪失・時計検査除去・精度拡張除去・pendingによるlegacy取消解除でそれぞれRED、元の関数へ戻して全check GREENを確認。P3-1のガードを外した検算は「取消直後10、後の確定8（現物8）」を再現した。
- `bash scripts/doc-consistency-check.sh` → ERRORなし、既存履歴のPK6 WARN 3件のみ。`bash scripts/doc-consistency-check.sh --target plan` → 同じ結果を確認。`git diff --check` 成功、`bash scripts/check-workflow-git.sh` → PK5/STATECAP OK。`git diff --name-only -- src src-tauri migrations` → 空。runtime・source詳細展開・正式Plan Gateは未実施、phaseはdesignを維持する。

### Codex設計引継ぎと早期点検（2026-09-18〜19、design、正式Plan Gateではない）

- ownerの設計委任を受け、旧案のprospectiveなScope/AC/実装指示をADR・Matrixへ置き換えた。以下の過去rallyは対象commit当時の履歴として変更しない。
- read-onlyの独立計算点検で、同秒の取消順序、旧active snapshotの移行、補正区分の時刻推定を確認。cursor・商品単位の観測順序・legacy移行・明示的kindを新案へ採用した。
- Sonnet早期点検: 時計変更/sleepの失効機構が未記載という指摘に、Windows通知generationとmonotonic/wall-clock差の検査、native gateを追記。legacyのUI案内と両cursorの明記も反映。レビューは読取りのみで、モデル実行はRootが担当した。
- Opus早期点検とclosure: 共有JANは本番前の不適合設定検査とwhole-file holdの既存owner判断を前提に、P1としてはclosedと再評価された。kindの明示、legacy取消の停止と再確認、継続的な再確認負担、時計誤差を拡張した境界、観測順序、旧pendingの移行を順に反映した。行単位の部分commit案は採用していない。
- 最終closureで、新しい再確認から派生するactive基準のS/E・source_cursor等の引継ぎ、旧DB移行直後の大量保留preflightについてclosedとの評価を得た。残った再有効化の再確認経路は、measured保存が必要・明細なしは既知の非対応範囲・checkbox代替不可と明記して一括是正した。request IDは公開tokenと同一、派生basisは内部専用IDとして確定した。round上限に従い追加のrallyは行わず、正式Plan Gateで文面を確認する。
- 検証command: `python3 scripts/probes/stocktake_time_model.py` → `PASS: temporal bounds, causal receipt, zero-net split, revision, count/rollback lifecycle, legacy recovery`。合成モデルの結果であり、Rust/SQLite/nativeの実装済み証拠ではない。
- 保存時刻だけでbeforeとする変異、および取消の吸収補正を除く変異をメモリ内で注入し、対応assertの失敗を確認。元の関数へ戻すと成功した。trackedモデルは改変した状態にしない。
- Windows通知とRustの時計仕様はADRの一次資料で確認した。実機での登録・通知順序、レジ時計、精算系列、EJ全形状、migration故障注入、正式Plan Gateは未実施。
- 未確認の店の運用や本番移行の件数を受容済みにせず、runtimeには着手していない。Ready・merge・GitHub mutationは行っていない。

### owner 回答（2026-09-16、design）

- Q1 = (a) candidate 採用 / Q3 = (a) candidate 採用 / Q2 = precondition-dependent（店の聞き取り待ち）。反例 2 件（force_fill は実測でない / 確定後の CSV 取消で二重に戻る）→ D-D1 / D-D5 を改訂。UI-10-D2 の「商品単位で記録されていない販売は自動補正できない」制約は残す。Writer は Codex（owner「Codex に回したほうが質良い」）

### Design Review round 0（2026-09-16、design、Opus、裁定 Coordinator。Plan Review round には数えない）

- P1 F1（force_fill が `counted_at` を書き D-D4 の境界になる → 後着販売を skip して過大）= accept → D-D1（`counted_at` NULL）、F13 の廃番自動入力も同じ扱い
- P2 F2（完了済み棚卸し後の取消で void 分が二重に戻る）= accept → D-D5 (ii) 打ち消し movement / F3（commit 結果に warning 欄が無い）= accept → preview 段階の既存 `warnings` に載せ wire 不変、S3 に §15.3 / F4（AC8 の帰属）= accept → `check_template_conformance`（full）へ / F5（73 §73.9 の参照誤り）= accept → 32 §15.3 へ / F6（確定後の負在庫）= accept → D-D2 に方針を明記、`stock_after` 既存列で表示 / F7（既存 green test の反転漏れ）= accept → 申し送りに 4 件 / F8（`VoidedMovement` に `created_at` が無い）= accept → `csv_imports.imported_at` 1 点で比較 / F9（Plans.md の「Human Gate に置く」表記）= accept → 本 commit で Plans.md を是正
- Q2 確定（2026-09-16、店の回答 owner 伝聞「時間帯関係なし、カウント済みでも売れたらそこで数量訂正」）= (b) `<=`。Coordinator の一時案「(a) 誤差が目に見える向き」は、訂正運用の下では訂正済み販売を二重に引くため撤回
- P3 F10（行番号 2 件）/ F11（AC9 の regex）/ F12（AC7 の pathspec）/ F14（Q3 の「1 query」表現）/ F15（Q2 の「前日分取込み」前提を外す）/ F16（日付 / 時刻の橋渡し不変条件）/ F17（SQL の比較形）/ F18（fix_integrity 対象外 1 行）/ F19（backlog の「日報取込み」表記）/ F20（Q1 に「確定後の在庫は数えた数と一致しない」）/ F21（35 §20.6a の設計ノート）= すべて accept、本 commit で反映
- 反例なし（F0）: 恒等式と数値例 3 本、順列 7 系列。壊れたのは F1 のみ
- 判定: owner 回答と本 review を反映した改訂版で、Q2 の聞き取り後に plan-draft → plan-gate（Sonnet + Opus の Plan Review）へ

### owner 追加指示（2026-09-16、plan-gate 中）

- 「日報は既存の置換方針を完成させる。棚卸しは『いつ数えても年末時点へ数量を繰り越せる』ことを中心に設計する。商品別に記録できた入出庫は自動反映し、記録できない部分だけ店の訂正を受ける。その二つを重ねて減算しない契約を作る。在庫の snapshot 補正が正しくても、評価額を各商品の古い実測数のまま合計すると年末時点の要求とは別物になる」→ Goal Invariant を言い直し、D-D7（評価数量 = 確定時点在庫）を新設、`total_cost` 据え置きの記述を撤回。日報は Non-scope に明記

### Plan Review round 1（2026-09-16、plan-gate、Sonnet、裁定 Coordinator）

- reviewer 実施: 起票時実測の file:line / `rg -c` を全数実物と突合し一致、AC7 の diff 3 file を確認、doc check（plan / full）ERROR 0 と check-workflow-git PASS を実行、D-D1〜D-D5 の算術と Q2 `<=` の反例を独立再現、INV-3 が一般方針であることを `10-common-rules.md:24` で確認
- P2（20 §2.11a `:878` と 65 `:267` が「差異は補正 movement の quantity、snapshot 差では定義しない」「開始時システム在庫」の旧契約のままで Scope 外、AC も検出しない）= accept → S7 / S8a に追加、AC2b を新設
- P3（35 §20.8 INV-3 行 `:451`「stock_after < 0 にはならない」が D-D2 と矛盾するのに S2 が名指ししていない）= accept → S2 に明記、AC2b に oracle
- 判定: round 1 は通過不可（P2 1）。本 commit で是正し、Opus round 1 と Sonnet closure で確認する

### Plan Review round 1 / 2 pass 目（2026-09-16、plan-gate、Opus、裁定 Coordinator）

- reviewer 実施: 起票時実測と `rg -c` baseline を全数一致、AC7 = 0、AC8 の 3 gate を実行（plan / full ERROR 0、workflow-git PASS）、design_compliance_test が未実装関数を red にしないことを確認、D-D1〜D-D7 の 9 系列を独立追跡（整合性は D-D5 (ii) を含めて保たれる）、Scope 過不足（FUNCTION_DESIGN / DB_DESIGN / 41 / 44 / 55 / 90 / requirements-coverage は除外妥当）
- P2-1（73 UI-10-D10 `:100` / §73.10 `:224` の `current_stock - actual_count` と D-D3 の衝突、UI-10-D10 の「同一ソース」根拠に未回答）= accept → D-D3 を「在庫列をカウント時在庫へ戻し差異と同一ソース、UI-10-D10 を UI-10-D14 で supersede」に、S4 に `:100` `:224` `:347`、AC4 に oracle
- P2-2（旧記述 sweep 漏れ 5 箇所: 35 `:36` `:45` `:204` §20.9、biz-task-specs `:476`）= accept → S2 / S6 に追記、AC2b を拡張
- P2-3（packet 内の `total_cost` 据え置き文 2 箇所）= accept → Lenses 行と Q1 を D-D7 準拠に
- P2-4（D-D5 (ii) の打ち消し movement が記録詳細に別行で出て「同値」が普遍でない）= accept → D-D5 に記録詳細での見え方と同値の範囲を明記、S7 / S8a の文言を限定
- P2-5（削除 oracle と撤回記録の書き方の衝突）= accept → AC 前文に 1 文
- P2-6（owner 追加指示を decision point として未計上）= accept → 介入 2/4（上限 3 → 4、理由付き）、Plans.md 同期
- P2-7（翌日入力の残差が過小）= accept → Q2 の残差記述を是正、運用ルールに「数えたその場で入力する」を追加
- P3-1（D-D7 の子 ID 欠落）= accept → BIZ-06-D4 を採番し Trace / Ledger / Spec / Readiness / Writer 行へ / P3-2（D-D7 の位置）= accept → D-D6 の後ろへ / P3-3（打ち消し量の粒度）= accept → 商品ごと SUM で 1 本 / P3-4（AC2b の 35 向け oracle が空撃ち）= accept → `採用しない` = 0 へ / P3-5（backlog の STK-1 行が stale）= accept → 本 commit / P3-6（行番号 `:52` `:124`）= accept
- 判定: round 1 は通過不可（P2 7）。本 commit で是正し、round 2 = Sonnet closure（両 pass の指摘の閉じ方を確認）

### Plan Review round 2（2026-09-16、closure、Sonnet、裁定 Coordinator）

- round 1 の Sonnet P2 / P3、Opus P2-1〜P2-7 / P3-1〜P3-6 = すべて closed（AC baseline 全件一致、doc check plan / full ERROR 0、workflow-git PASS、design_compliance_test PASS を reviewer が実行）
- 新規 P2（S4 の UI-10-D14 説明に「『現在在庫』列は維持」が残り D-D3 の棄却案と literal 一致）= accept → 本 commit で「カウント時在庫に一本化」へ
- 新規 P3（AC2b の `動的計算` baseline が `:218` だが実物は `:217` の見出し）= accept → baseline と S2 を訂正
- 判定: 通過不可（新規 P2 1）。本 commit で是正し round 3 = closure（天井 3 の最終回）

### Plan Review round 3（2026-09-17、closure、Sonnet、裁定 Coordinator）

- round 2 P2 / P3、owner 追加指摘 3 件 = closed（AC baseline 全件一致、doc check plan / full ERROR 0、workflow-git PASS）
- 新規 P2（未実測「—」/ 自動補完注記の表示契約に AC oracle が無く、Spec Contract / 申し送りにも未反映）= accept → AC4 に 2 oracle、Spec Contract と申し送りに 1 行ずつ（本 commit）
- reviewer 判定は「通過不可（新規 P2 1）」、round 天井到達。同日に owner が Q2 を不承認したため disposition = owner escalation → `state-backtrack plan-gate->design`（`9a02f8f9`）。本 rally はここで閉じ、設計確定後の plan-gate は新しい rally

### owner 判断（2026-09-17、Q2 不承認と方式の評価）

- 「`<=` の一律適用は現段階では承認しない。実測の再入力と、販売に伴う数量訂正を区別し、記録済み入出庫・POS 販売・返品を混ぜても、重複や欠落が起きない契約を先に定める」。反例 = 引き算訂正で記録済み入庫が消える / 返品の増加を落とす経路 / `<` に戻しても解決しない
- 骨子の評価: ① 実測の再入力と販売訂正の分離 = 賛成（店には手作業を終える変更として説明）/ ② 自動繰り越し = 条件付き賛成（実測より後に発生し実測数に未含有の増減に限る。PLU 未移行商品は自動追跡の対象外）/ ③ (i) 不採用、(ii) 第一候補（商品・数量・取引時刻・返品／取消を復元でき前後を判定できること。判定不能は残す）、(iii) 条件付き代替（再確認状態の永続化、未解消なら force_fill でも確定不可、同日追加取込み・取消の再確認条件）。`sale_date` は精算日で、日跨ぎ精算では「対象期間が実測の前後どちらか確定できる」前提が要る
- 推奨: (ii) の実データでの成立確認を先に行い、判別できない部分だけ必須の再確認へ回す → D-D1 / D-D2 / D-D4 / Q2 を改訂、次の行動 = EJ の採取と形状確認。介入 3/4
- Coordinator が field-check の公式マニュアルで EJ の存在（`Ejyymmdd.TXT`、取引ごとの時刻と PLU 行、売上 /EJ 保存設定）を確認し、起票時実測へ追記。sample 未採取

### owner 追加指摘（2026-09-16、round 2 と同時）

- 在庫列の変更（カウント時在庫へ）と介入 2/4 の計上 = 賛成。Q2 の同日販売ルール全体への承認は分けて扱う（owner 承認待ちとして Q2 に明記）
- P2（未カウント行の表示条件）= accept → D-D1 / D-D3 に「未実測は『—』、自動補完は注記で区別」/ P2（S4 の逆指示）= accept（round 2 P2 と同件）/ P2（「その場」≠「当日中」、10 → 8 → 古い 10 入力で確定 10 に戻る反例）= accept → 運用ルールを owner の文言に置換、残差記述を是正 / 「現在在庫は在庫照会で確認できる」の言い方 = accept

### Plan Review 新 rally round 1（2026-09-18、plan-gate、Sonnet + Opus 独立 fresh context、対象 `75dcf494`、裁定 Coordinator）

- P1 なし。Sonnet P2 2 / P3 2、Opus P2 5 / P3 6。全件 accept（Sonnet P2-2 の「対称の分岐を足す」案だけ reject、理由付きで据え置きを明記）
- Opus P2-1（Tz なし file の同日解除で二重減算）→ D-D9: スキップ対象 = 保留対象集合、Tz なしは `date(C') > D` / P2-2（段 0 の日跨ぎ精算の前提が source docs に落ちない）→ D-D4、S1、S4、AC10 / P2-3（preview cache の TTL と入力の喪失）→ S3 §15.9、S11 / P2-4（記録詳細に区分の器が無い）→ S7、S8a / P2-5（段 1 は保存時刻基準で非保守的）→ D-D4 に前提と破れたときの向きを明記、運用文言を「精算後に数え始め」へ。owner が受容する残存リスクとして提示する
- Sonnet P2-1（Risk）→ R2 維持を tie-break 規則への反論付きで裁定（Opus も R2 妥当）/ P2-2（対称の分岐）→ reject: Z004 は期間の終わりしか持たず最初の販売時刻が分からない。既知の受容済みトレードオフとして D-D4 に明記 / P3（ゼロ行の照合規則）→ D-D8 / P3（二重入力）→ D-090 Revisit
- Opus P3-1（列挙の追随）/ P3-2（見出しの D-D8 表記）/ P3-3（保留の放置）/ P3-4（精算前の解除で再保留）/ P3-5（エラー契約の oracle）/ P3-6（Plans.md の撤回済み表現）→ 反映
- 反例にならなかったもの（両 reviewer が独立に追跡）: D-D8 の再保存と取消・追加取込みの順列、D-D9 の reference を完了済み棚卸しにする判断、別の進行中棚卸しがある場合の分岐、解除した取込みの取消 → 再取込み、`stock_quantity = SUM(movement)`、保留放置のまま後続日の取込み
- 残る不確実性: レジ時計と PC 時計の実差は未実測 / 従来 shape（Tz なし）の Z004 が現運用で出るかは未確認

### Plan Review 新 rally round 2（closure、2026-09-18、対象 `3220c92c`、裁定 Coordinator）

- Opus = 通過可。round 1 の P2 5 / P3 6 は全件 closed、新規 P1 / P2 なし、修正が生んだ反例なし。P2-5（段 1 の保存時刻基準）は「運用条件 + 破れたときの向きの明記」で不足なしと判定。新規 P3 3 件（S3 §15.4 の指示に D-D9 の新規則が無い / §15.9 の「再起動後も入力保持」は不成立 / `corrected_count` の置き場と述語）= accept、本 commit で反映。任意提案「warning を段 0 / 段 1 由来で分ける」= 採用（既存 warnings 経路だけで段 1 の効きを可視化できる）
- Sonnet = round 1 の P2 2 / P3 2 は全件 closed、対称の分岐の reject に反論なし（Z004 は累計帳票で取引時刻を持つのは EJ だけ）。新規 P2 1（申し送りと Ledger に round 1 の新決定 3 つが未反映 = Goal 最小完了条件 (6) に関わる）/ P3 1（S7 の引用が rally を跨いで曖昧）= accept、本 commit で反映
- 本 commit の修正は Writer への指示と申し送りの同期だけで、契約本体（D-D4 / D-D8 / D-D9）は round 1 反映後から変えていない（warning の件数分けを除く）

### owner review（2026-09-18、plan-gate、対象 `3220c92c` 時点の読み。裁定 Coordinator）

- 残存リスク（段 1 の保存時刻基準）= precondition-dependent の受容候補。条件「実測から保存まで対象商品の数量を動かさない、動いたら再確認する」を現場で守れること。補正 2 点 = 「精算後に数え始める」だけでは足りない（精算後にも販売・返品・入庫は起こる）/ 誤差は過大・過小の両方向 → D-D4、S1、S4 に反映。派生して段 0 の穴（精算後〜実測の POS 販売は翌日付の Z004 に入り通常適用で二重減算）を Coordinator が特定し、前提と運用条件として D-D4 に明記
- P2-1 全商品一括保存と「その場で保存」が両立しない = accept。保存単位の選択（案 A / 案 B）を「D-D9 の未決」に置き owner 判断待ち。**Plan Gate の承認依頼はこの決定と反映、closure の後**
- P2-2 共有 JAN のゼロ行が対象から消える = accept → D-D8（候補の全商品を判定対象、ゼロ行・非ゼロ行とも）。round 1 Sonnet P3 への Coordinator 対応「複数件は対象から外す」は ① の合意と衝突していたため撤回
- P2-3 ゼロ行だけの file が `parse.rs:156` の 0 件ガードで止まる = accept（実物確認）→ D-D8、S3、Ledger、AC10、申し送り
- P2-4 画面 state では再起動後の入力保持にならない = accept。`19b29673` で「再起動では入力やり直し」へ修正済み。「復元した値を無条件に再送しない」は P2-1 と同根で「D-D9 の未決」に含めた（案 B なら論点ごと消える）
- Sonnet の対称の分岐を採らない判断 = owner も賛成

### owner 決定（2026-09-18、plan-gate、D-D9 の保存単位）と反映

- **案 B を採用**: 商品ごとに実測と補正を保存し、取込み本体から分離する（「保存値は保存時点の現物数」の原則に合う）。D-D9 を書き換え、案 A は棄却へ移した
- owner が設計と検証に含めるよう指定した 4 点と置き場: (1) 通常の commit 時の最新状態確認と重複防止は維持 → D-D9「commit 時の確認は維持する」、S3 §15.4、Ledger / (2) 取込み前の再実測を未作成の `csv_import_id` へ依存させない → `stocktake_recounts` から `csv_import_id` を外し、movement の note も取込み id を持たない、S5、Ledger / (3) ゼロ行だけの file も再実測後の再 preview から完了できる → D-D8 の 0 件ガードを「棚卸しの境界判定に掛かった行がある file は通す」に定義し直し（再実測後は再確認対象・保留対象とも 0 になるため、前の規則のままでは拒否されていた）、Ledger / (4) 一部保存後の入出庫・中断・再起動で保存済みの補正が失われない → D-D9（1 商品 1 TX、基準在庫は保存時点）、S11、Ledger
- 案 B で論点ごと消えたもの: `CommitRequest.recounts`、保留対象集合の TX 内再検証、入力値の画面 state 保持と cache の TTL（新 rally Opus round 1 P2-3、round 2 P3-2）、「スキップ対象 = 保留対象集合」と Tz なし file の解除条件の BizError（Opus round 1 P2-1 は「条件を満たさない再実測では保留が外れない」で構造上塞がる）
- 精算後の販売が翌日付の Z004 に入る問題 = 保存単位とは別の運用条件として D-D4 に置く。現場で守れることが条件で、**現時点で受容済みとは書かない**（owner 2026-09-18）
- EJ lane への申し送りに「翌日付の精算区間が実測時点をまたぐ場合も判定する」を追加（owner 2026-09-18）
- 契約が動いたため、変更箇所に絞った closure を再度掛ける（round 3 = 天井）

### Plan Review 新 rally round 3（closure、天井、2026-09-18、対象 `02ca6dec`、裁定 Coordinator）

- Opus = 通過可。新規 P1 / P2 なし。案 B の D-D9 を状態機械として下の系列を追跡し反例なし（余裕幅未満の再実測 / Tz なし file の同日再実測 / 同日の追加取込み / 再実測後〜commit の販売・入出庫 / 一部保存後の中断・再起動 / 保存後に取込みをやめる / 別の進行中棚卸しが後から始まる / preview → commit で持ち主が変わる / 同一商品の再実測 2 回 / 保留中に別日の file を取り込む）。owner 指定の各点は契約と Ledger の両方に所在を確認。実質ゼロ件ガード（`parse.rs:156`、reviewer が現物確認）の新定義は無関係な空 file を通さず、通すべき file を止めない。段 0 の前提の反例を合成値で独立に再現し（在庫が現物より過小になる向き）、現行の扱いを proportionate と判定。P3 は全件 accept、本 commit で反映（段 0 の前提と両方向の oracle / 売上 0 件の取込みの一覧表示 / 共有 JAN が保留側にも効くこと）
- Sonnet = 通過可。round 2 の P2 / P3 は closed。案 A 由来の記述の残存は棄却理由・起案時の記録 block・履歴だけ。AC の baseline は全件一致、`BIZ-06-D6` の衝突なし。P3（新しい関数の名前と節番号が未指定）= accept、`record_stocktake_recount` / §20.4.1 を S2 に明記
- 本 commit の修正は oracle・表示規則・命名の追記だけで、契約本体（D-D4 / D-D8 / D-D9）は `02ca6dec` から変えていない。rally は天井の round 3 で close。次 = owner の Plan Gate 承認
- 残る不確実性（owner へ提示）: 段 0 / 段 1 の前提は現場で守れることが条件で、受容済みではない / レジ時計と PC 時計の実差は未実測 / 従来 shape（Tz なし）の Z004 が現運用で出るかは未確認

### owner Plan Gate 承認（2026-09-18）

- owner が Plan Gate を承認、介入上限 8 への再改訂も承認。Plan Commit の確定と state 遷移は、下の「仕組みで守る」への回答を packet に入れてから行う（確定後の追記は Amendment になるため）
- owner の懸念: 段 0 / 段 1 の前提は「仕組み的に守らざるを得ない形にしないとヒューマンエラーで容易に壊れる」。Coordinator の整理 = app が持つ事実は app の時計・取込みの順序・記録済み movement だけで、POS の取引時刻を持たない。前提を仕組みに変えられるのは取引時刻を持つ EJ（段 2）だけ。EJ の位置づけ（任意の部品か、PLU 販売の本番開始の前提条件か）は owner 回答待ち
- 未確認 2 点の解消: レジ時計 = 説明書に月差 ± 40 秒・手動設定のみ（Contract Probe に記載）/ 従来 shape の Z004 = app が当初の仕様から想定した形（1 行目が日付だけ）で、実機から採った file では観測されていない。CV17 の SD 取込みで `EcrDatas` に残る実 file はメタ 6 行 + header の layout A で、6 行目に時刻を持つ（field-check `summaries/2026-07-06-z00x-shape-analysis.md`、23 `:5`）。PC ツールで取り込んだ Z004 をそのまま使えばよく、別の書出しは要らない。従来 shape が来ても段 3 へ落ちるため安全側

### 統合案のclosure passとfollow-up是正（2026-09-19、design、発注64）

- 設計レビューbroad（対象 `4319fe36`、2026-09-19）: Sonnet + Opusの独立fresh context、裁定Coordinator = Fable。SonnetはP1/P2なし・P3 2件、OpusはP2 2件（版増分のchoke point、migrationのauto_filled欠落）・P3 6件。
- 是正 `68c3d0d3` に対するOpus closure（2026-09-19）はpass、P1/P2の残存なし。P3-1は数値例の成立を追跡し不採用妥当、P3-6も不採用妥当、ほかはCLOSEDと確認された。
- closureの新規指摘 `P3-A / P3-B / P3-C` はfollow-up。owner決定（2026-09-19）どおりclosure passを維持し、Codexが本commitでまとめて是正した。これはDesign Phaseの設計レビューであり、正式Plan Gateではない。Plan Commitはpending、Phaseはdesign、runtimeは未着手のまま。
- P3-A: pendingの取消TX内昇格は算術上の不成立を理由にせず、取消が確定前の棚卸し差異を暗黙に在庫へ適用し、確定時補正・差異件数の意味を変えるため不採用とD6へ明記した。既存の明示的な再実測による復旧を維持する。
- P3-B: D3と `qualified_source` を、取引下限が開始の最遅候補 `S_latest` を厳密に超えた場合だけ因果矛盾とする条件へ同期した。開始自身の粒度・観測誤差を含め、保存Eによる検出遅れと最早始端による誤検出を避ける。AFTER判定のE比較は不変。
- P3-C: ledgerの自動補完→移行→取消、確定→独立再実測、`(8, None, 0) → legacy`、legacy早期return前の診断実行、開始と保存の間の因果矛盾をassertで固定した。開始の誤差範囲内・接触では時計を無効化しないことも追加し、Matrixを同期した。
- 検証のred/green: 新しい開始境界のassertは変更前のE閾値で `AssertionError`、変更後にモデル全体PASS。temporary copyで自動補完のlegacy化・active解除除去・actual=0条件除去・診断のlegacy分岐後移動・閾値をEへ変更・閾値を最早始端へ変更・境界接触も矛盾扱い、の各変異を実行し、全てREDを確認した。trackedモデルは変異させず、copyの原本との `cmp` が一致し、モデル全体を再実行してGREENを確認した。
- Review-only skipped because: 既存closureでpassした設計のfollow-upに限定し、採用済みの操作方式は変更しない。新規broad/rallyは起こさず、対応assert・実変異・文書検査で是正を確認し、詳細source同期後の正式Plan Gateへ引き継ぐ。
- 検証command: `python3 scripts/probes/stocktake_time_model.py` → exit 0、`PASS: temporal bounds, causal receipt, clock conflict, zero-net split, revision, count/rollback lifecycle, legacy recovery, migration kinds, force_fill`。`git diff --check` 成功、`git diff --name-only HEAD -- src src-tauri` → 空。`rg '^### SPEC-STK-TIME-D' docs/adr/2026-09-18-stocktake-time-evidence.md` → D1〜D9の9見出しを維持。`bash scripts/check-workflow-git.sh` → PK5/STATECAP OK。
- 文書検証: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-09-16-stocktake-count-baseline.md` → ともにexit 0、ERRORなし、既存履歴のPK6 WARN 3件のみ。計画側で業務上の状態語を未決マーカーと誤検出した箇所は「確定前」へ言い換え、検査ルールは変更していない。

### source詳細同期とplan-draft（2026-09-19、発注65）

- 開始版 `36891af8` は単段merge後のcleanなtree。Scope/ACをsource編集前に同期作業用へ置き換え、共通CmdErrorの所有先40を現物確認後に追加した。ADRと合成モデルの意味・内容は変更していない。
- 波及採用は30（商品更新型/一括import）、31（共通在庫writer）、43（DB接続交換）、master（版の保存）、transaction（既存TXへの版の接続）。71の復元本体とdecision-logは非編集。profileと現行図面は現行実装の説明なので本runでは変更せず、runtime切替時の同期対象として申し送った。
- 読取り専用の別contextでDB/IO/BIZ/CMDの新節をADRと照合。P2候補のactive legacy分岐と、業務失敗で時計失効が消えるTX境界を現物で確認して採用した。所属での一意な分岐と独立証拠TXへ修正し、相手側24/posにも同期した。再確認で両指摘CLOSED、是正箇所のP1/P2なし。これは正式Plan Gateではなく、UI/親文書を含むsource全体の承認へ流用しない。
- 実施: `python3 scripts/probes/stocktake_time_model.py` → exit 0、既存PASS行を維持。`bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-09-16-stocktake-count-baseline.md` → ERRORなし、既存PK6 WARN 3件のみ。`git diff --check` 成功。`git diff --name-only 36891af8 -- src src-tauri` およびAC4の禁止path差分は空。
- 要求参照の照合: 変更docごとに `git show 36891af8:<path>` とworktreeの `rg -o 'REQ-[0-9]+'` をsortしてdiff → 全て同一。新しい関数設計file・要求tokenの増減がなく、module-map/traceability/bindings/route treeの生成は不要。本runではこれらを変更していない。
- 未実施: runtimeのSQL・TX故障注入・binding/enum配線・UI/native・実機EJ/時計/精算系列probe、正式Plan Gate。自動分類とPLU本番の成立を本runの成功で代用しない。設計変更を要して停止した項目はなし。

fileごとの変更要点（function-designは同ディレクトリ内の番号、DB/architectureは各サブディレクトリ内）:

| file | 変更要点 |
|---|---|
| 20-io-product-repo | 汎用数量欄撤去、context読取り、両cursor・実測/再実測・flag・要求照会 |
| 21-io-inventory-repo | 数量/版の不可分更新、補正kind伝播、差異集計 |
| 23-io-z004-parser | 任意精算メタ、正常JANゼロ行、信用判定のBIZ境界 |
| 24-io-csv-import-repo | source upsert、証拠TX、取消前読取りとmovement ID |
| 30-biz-product-service | 設定/マスタ変更時の全候補検査と版更新 |
| 31-biz-inventory-service | 共通repoの版増分と既存TX/冪等性の接続 |
| 32-biz-csv-import-service | 準備照会、受領/分類/commit/取消、時計失効、外部probe表 |
| 35-biz-stocktake-service | contextの意味、save TX、N-L確定、legacy復旧、詳細読取り |
| 36-biz-integrity-check | movementを作らない整合性補正でも版を進める契約 |
| 40-cmd-product | 共通のstocktake_guard、型付き回復payloadと生成義務 |
| 41-cmd-pos | stock_review/準備照会/結果の伝播、cacheとBIZの境界 |
| 42-cmd-sales-stocktake | begin/save/abandon wire、保管/失効、旧入口撤去、native境界 |
| 43-cmd-settings-log | DB交換前のcontext世代失効、復元本体の規則は維持 |
| 55-ui-csv-import | 保留→商品別保存→再preview、再起動後の再選択 |
| 65-inventory-record-traceability | 差0を含む訂正への到達、kind表示、過去評価の非遡及 |
| 73-ui-stocktake | 明示begin・focus/IME・保存・回復とnative合格条件 |
| master-tables | stock_revisionの型/制約と専用更新口 |
| transaction-tables | header/item schema不変、既存業務TXへ版更新を接続 |
| pos-tables | source/hash/メタ/時刻証拠/import参照と独立した失効保存 |
| tracking-system-tables | kind・証拠・recount・flag・補正区分・移行の保存条件 |
| DB_DESIGN | 新保存契約の索引と現行ER/schemaの区別 |
| ARCHITECTURE | UI/CMD/BIZ/IO/MNTの新契約の所有表 |
| biz-task-specs | 各BIZの役割と詳細sourceへの接続 |
| io-task-specs | parser/永続化と業務判断の分離 |
| cmd-task-specs | token/wireと登録・生成の相手側 |
| ui-task-specs | 保存状態の所有と保留/訂正の導線 |
| mnt-task-specs | OS監視・時刻対応・DB交換・移行の境界 |
| FUNCTION_DESIGN | 関数詳細の索引、現行/新契約の区別 |
| SCREEN_DESIGN | 既存画面での計数/保留/訂正と到達性 |
| UI_TECH_STACK | feature-local入力、生成wire、D-052 consumerと失効 |
| 本packet | 同期Scope/AC、source対応表、runtime申し送り、phase前進 |
| test-matrices/2026-09-18-stocktake-time-evidence | source対応とruntime検証の不足を明記 |
| Plans | closure/P3是正・source同期の完了と次の正式Plan Gate |

新たな設計質問はなく、source出力と検証を根拠に `design → plan-draft` をcontent commitへ同乗する。独立した正式Plan Gate、Plan Commitの確定、runtime着手、push/PR/Ready/mergeは行わない。Fableへsource同期版のレビュー発注を引き継ぐ。

### Plan Review 新 rally round 1（2026-09-19、plan-gate、Sonnet + Opus の独立 fresh context、対象 `63d7507d`、裁定 Coordinator = Fable）

- 対象 `63d7507d` のSonnet = P1/P2/P3なし・通過可。Opus = P2 4件・P3 7件・通過不可。以下は是正結果であり、round 2 closureの通過判定ではない。Workflow Stateのfieldは変更せず、Plan Commitはpendingを維持する。
- P2-1採用: 32のpreview/commit TXに精算同一性guardを明記し、24/pos/40/41/55へ同期した。別hashの候補衝突はsource_identity_conflictで拒否し、確認同意・再実測・取消を解除条件にしない。reset/別系列は検証された証拠で区別し、生の番号組へ一律UNIQUEを張らない。元資料の採用を決める新しい解決操作は追加していない。
- P2-2採用: 35の§20.6a/§20.8/§20.9、73のD10と一覧/文言、20 §2.11a、65 slice 4cの旧契約の適用範囲を明示した。完了済みversion=0の表示は保存し、新方式は補正N-L・表示差異L-N・後続移動保持・負在庫許容/評価数量だけ非負とする。
- P2-3採用: tracking/42とruntime申し送りへ、migration・全writerのkind/証拠対応・旧update_count公開撤去・新UIを同じ配布単位にする制約を追加した。observation_kindは最終schemaにDEFAULTを置かず、旧行は移行分類で明示充填する。Matrixに同一releaseとkind省略失敗のoracleを追加した。
- P2-4採用: MatrixとLedgerに `req401_ambiguous_jan_sync_rejected` を追加。create/update/商品一括import、batch内共有、preview後の変化をTX全体write0で検証し、表示だけのpreflightを代用しない。
- P3-1/2採用: ADR D3へ独立証拠TXを同期。verifiedはowner-operated gate成立済み証拠のBIZ内部反映に限定し、通常操作/設定キーから昇格せず、期限超過はunverified扱いとする。32/24/posへ同じ責務を記した。
- P3-3/4採用: 20のledger上限は呼出し時点とし、Rの補正前と派生N/Nの補正後を区別。32の共有JAN guardを個別分類/通常適用より前の行全体guardへ並べ直した。
- P3-5採用: kind名stocktake_guardは維持し、shared_jan_unresolvedのBIZ-01 producerと51/60の対象表示・入力/選択保持・成功扱い/自動再送禁止を定義。編集前にS4とAC7へ対象UI docを追加し、FUNCTION_DESIGNの索引も同期した。
- P3-6不採用: OS監視は外部時刻の信用だけでなく、sleep/時刻変更を跨いだ未保存contextを失効させるD1の条件であり、time_basis_id=NULLだけでは代替できない。監視不成立の発生率は未実測だが、黙って保護を弱めず、42/73で実測begin/未保存saveの拒否、保存済みデータ保持、再起動・担当者のnative診断、復旧後の新beginを明示した。closure pass済みのD1判断と設計モデルは変更していない。
- P3-7採用: MatrixにTimeEvidenceの不正JSON・欠落・範囲・期限・verified昇格権限のoracleを追加した。これらはruntime予定であり、本runで実装/成功済みとはしていない。

#### 旧rallyのaccept済み事項の追加sweep

履歴を一覧してADRと現sourceへ照合した。別contextの読取りsweepは補助とし、既存テストの値・現sourceはRootも確認した。過去rally本文は変更しない。

| 旧指摘の群 | 現設計での扱い・是正先 |
|---|---|
| Design R0 F1/F13（補完は実測でない） | D1/D4/D8、35/tracking/Matrixで保持。NULLだけの識別はkindへ発展 |
| R0 F6/F20、ownerの確定数量/評価額 | 式は保持、旧INV/差異のsupersedeを今回補足。Matrixへ在庫11・評価11C、在庫-2・評価0のoracleを復元 |
| ownerの実測と販売訂正の分離、旧Opus P2-7 | D1/35/73のbegin→実測→即保存で保持。古い数の自動転記は禁止 |
| 日付<=、Tzなし翌日解除、精算後/当日精算の前提 | 旧解決策は不適用。D2〜D5の受領/区間/実測窓へ置換し、日跨ぎ・精算後販売の問題はMatrixに保持 |
| R0 F2/F8、最初の有効吸収先 | D6/20/24/32/35で保持。imported_at一点比較・VoidedMovement拡張不要は不適用、ID/cursorを使う |
| 旧Opus P3-3（商品別SUM補償） | 展開から粒度が落ちていたため32へ商品別非0は1本・純量0は0本を復元、Matrixへoracle追加 |
| 補正区分、corrected_countの見え方 | 20/21/35/65/trackingで保持。時刻比較の旧述語は明示kindへ置換 |
| 旧Sonnet/Opusの20/35/65/73 supersede漏れ | 今回P2-2と同じ対象として是正。既存の完了記録と新方式を区別 |
| 旧round 3・ownerの未実測/自動入力表示 | 73の文言は保持されていたがUI oracle/申し送りが欠落。Matrixとruntime申し送りへ復元 |
| R0 F7（既存green test更新） | 申し送りが欠落。Matrixに現物確認したstocktake_service、formatter/Page、cross-feature診断の移行先を記録。計数前変更の差3を-2へ単純反転する旧指示は不適用 |
| ゼロ行のみfile、解消後対象0の再preview | 23/32/55、Matrixで保持。売上0の正常完了を維持 |
| 共有JANのゼロ行/全候補/保留 | D4/30/32で保持。複数一致を除外する旧案は撤回のまま。今回write guardとUI oracleも追加 |
| 案Bの商品別即保存・import非依存・再起動 | 20/35/tracking/55で保持。CommitRequest.recounts・画面入力の再起動復元・保留入力用TTL拡張は不適用 |
| commit再検査・flag・同日追加/取消順列 | 32/35/tracking、Matrixで保持。今回精算同一性の再検査も具体化 |
| R0 F3/F5、新rally warning/エラー | 型付き回復と結果warningで保持。previewだけ/wire不変・段0/1別件数という旧制約は新分類へ置換 |
| ownerの仕組みで守る・EJ本番前提・訂正 | D1/D5/D7/D9と32/35/65/73で保持。純合計だけでEJ完全扱いせず、再実測の出口を残す |
| Risk・旧Gate・介入・関数名/子ID | 現R3/Plan Commit pendingを維持。旧R2裁定/旧Gate承認は流用せず、旧record_stocktake_recount名はbegin/save/purposeへ置換 |
| R0 F4/F9〜12/F14〜19/F21、旧rallyの行番号/regex/列挙/参照/申し送り | 旧位置/件数oracleは履歴限定。現在のScope/AC/Ledgerで追跡。fix_integrityは版更新callerとして現D1へ接続。日報は本laneで変更しない |

復元した取りこぼしは、既知supersede範囲に加えて、既存test移行、未実測/自動入力UI oracle、商品別補償粒度、確定時評価額oracle。実装テストの削除・skip・無条件の期待値反転は行わず、Matrixの対応契約からruntimeで更新する。

#### 発注66の検証

- AC1: `rg '^### SPEC-STK-TIME-D' docs/adr/2026-09-18-stocktake-time-evidence.md` → D1〜D9を維持。
- AC2: `python3 scripts/probes/stocktake_time_model.py` → exit 0、既存の `PASS: temporal bounds, causal receipt, clock conflict, zero-net split, revision, count/rollback lifecycle, legacy recovery, migration kinds, force_fill` を維持。モデル自体は変更なし。
- AC3/AC6: Scopeのsourceへ `rg -n '時点証拠契約|SPEC-STK-TIME'` を実行し、保存→IO→BIZ→wire→UIを照合。51/60もproposedを明示。32の外部probe表、42/73の監視不成立・native境界と回復条件を確認した。
- AC4: `git diff --name-only 36891af8 -- src src-tauri`、同基準の90-traceability/model/workflow/template差分、`git diff --name-only 63d7507d -- src src-tauri` → 全て空。変更docごとの要求tokenを開始版とsort比較して一致。Workflow Stateのfield行も開始版とのdiffが空。
- AC5/AC7: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-09-16-stocktake-count-baseline.md` → ERRORなし・既知PK6 WARN 3件のみ。`bash scripts/check-workflow-git.sh`、`git diff --check` 成功。source追加に合わせScope/ACを同期し、phaseはplan-gateのまま。
- 未実施: Matrixで追加したruntimeテスト、移行/配線の実装、Windows/EJ/時計の実機検証、round 2 closure。停止・backtrackを要した項目はなし。push/PR操作は行わない。

### Plan Review 新 rally round 2（closure、2026-09-19、Opus、対象 `0c5186f6`、裁定 Coordinator = Fable）

- closure結果: P2-1はメタあり経路のみCLOSED、メタ欠落経路がOPENのためPlan Gate通過不可。P2-2〜P2-4、P3-1〜P3-5 / P3-7はCLOSED、P3-6の不採用も妥当と判定された。発注66の旧指摘sweepの抽出群は現物一致。過去のReview Responseは書き換えない。
- P2-1残件を採用し、設計者判断で(A)同日追加拒否に確定した。(B)確認同意で許可は二重計上を利用者の注意に委ねるため採らない。ADR D3 / 32へ、同日activeがあり対象側または既存側のmachine_no / settlement_noが片方でも欠ければsource_identity_conflictで業務write前に拒否する規則を置いた。24 / posはNULLやsource取得不可を候補なしへ落とさず、41 / 55は追加確認で解除しない。Ledger / runtime申し送り / Matrixも同期した。
- この拒否はZ004の同一性guardであり、同日複数精算そのものやZ001/Z002/Z005日報取込みを禁止しない。識別メタが揃った別精算は他のguard通過後に追加可能。同日activeなしはメタ不足の追加条件では拒否しないが、全受領sourceの別hash衝突等は維持する。従来shapeの同日追加が止まり得る費用は残る。観測済みの実機資料はlayout Aだが、常に同じ形という保証は未検証で、費用ゼロとは扱わない。
- 新規P3（旧oracle）を採用: 73冒頭の置換範囲へ§73.12のcurrent_stock差異/主列とテストoracleを含め、新方式のL-Nとの適用範囲を明確にした。
- 新規P3（拒否の業務上の帰結）を採用: ADR D3 / 32 / 55 / Plansへ、誤版取消後の訂正版や系列未証明resetは取込み不能で売上・当該取込みの在庫減算が欠け、再実測では売上欠落を復旧しないことを明記。メタ不足の同日追加も拒否条件が残る間は同じ制限。55は元資料/既存記録の確認先と画面で解決できない場合の案内を持ち、正しい既存取込みの取消を迂回手順にしない。
- `req401_same_day_missing_identity_rejected`は対象/既存双方の部分・全部欠落、activeの各status、追加確認false/true、source欠落、preview後の同日active追加、許可側の境界をruntime oracleとして固定した。メタとactive集合を持たない合成モデルは変更せず、そのPASSを新guardの検証に流用しない。
- Review-only skipped because: 今回はclosure残件と直接の波及先に限定する。既存sweepをやり直す独立broadは追加せず、正式なround 3 closureはFableがOpusへ発注する。天井は据置きで、承認・phase前進は代行しない。

#### 発注67の検証

- AC1: `rg '^### SPEC-STK-TIME-D' docs/adr/2026-09-18-stocktake-time-evidence.md` → D1〜D9を確認。D3の同日追加拒否をADR→IO/DB→BIZ→CMD→UI→Matrixで照合した。
- AC2: `python3 scripts/probes/stocktake_time_model.py` → exit 0、`PASS: temporal bounds, causal receipt, clock conflict, zero-net split, revision, count/rollback lifecycle, legacy recovery, migration kinds, force_fill`。モデル自体は変更なし。
- AC3/AC6: Scopeのsourceに `rg -n '時点証拠契約|SPEC-STK-TIME'` を実行してproposedを確認。32の外部probe表、42/73のWindows監視/native失敗条件と回復を維持。51/60のstocktake_guard表示とMatrixの商品master各write oracleも保持した。
- AC4: `git diff --name-only 36891af8 -- src src-tauri` と `git diff --name-only 0c5186f6 -- src src-tauri` は空。同基準の90-traceability/model/workflow/template差分も空。変更pathはScope内docsだけ。変更docごとの `REQ-[0-9]+` tokenのsort比較は基準 `36891af8` / 開始 `0c5186f6` と一致、Workflow Stateのfield行も開始版と一致した。
- AC5/AC7: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-09-16-stocktake-count-baseline.md` → ERRORなし、既知PK6 WARN 3件のみ。`bash scripts/check-workflow-git.sh` と `git diff --check` 成功。phaseはplan-gate、Plan Commit pendingを維持した。
- 未実施: 新しいMatrix oracleを含むruntimeテスト、migration/bindings、Windows/EJ/時計/系列の実機検証、round 3 closure。停止・backtrack項目なし。push/PR操作なし。次の正式判定はFable発注のOpus closureへ渡す。

### Plan Review 新 rally round 3（closure、天井、2026-09-19、Opus、対象 `a2a265bf`、裁定 Coordinator = Fable）

- round 2のP2-1残件と新規P3は全てCLOSED。層間整合・Matrixの検出力・Scope/Workflow State/要求tokenも確認済み。一方、移行前importがメタ不足の比較先となる新規P2によりPlan Gate通過不可、天井到達のdisposition = owner escalationとなった。
- 新規P2を採用: 移行前importは原本がlayout Aでも識別メタを保存していないため、メタ完備の正当な同日追加も拒否される。P3の費用開示も同じ是正へ含める。round 2記録は履歴として保持するが、費用の範囲は従来shapeに限られず、sourceなし/メタ不足のactive importがある全日付へ及ぶと訂正する。

### owner escalation と決定（2026-09-19）

- owner回答の要旨（2026-09-19。原文はGated Amendment 2により本packetから除いた）: 店にあるのはレジの仕組みだけで、レジは取引の記録以外をせず、レジスターツールの出力も手作業で写している。本アプリはほぼ初導入に当たる。持ち帰った判断:「本番履歴なしを移行前提に記録し、新形式で本番開始。安全側の拒否は維持し、将来の後追い取込みは禁止しない」。
- 初導入の事実は[project-memory](../../project-memory.md)のPOS Factsへ確認日/owner原文付きで正本化した。レジ周り以外の紙運用、手作業コピー、本番DB・本番取込み履歴なし、存在するapp DBは開発/demo/試験用、本番開始日を同じ位置へ追記する義務を記した。AGENTS Session StartとPROJECT_HANDOFFの既存参照から辿れるため、入口/workflow文書の変更は不要。
- ADR D3 / D8 / Consequencesへ新形式での本番開始と移行費用を同期した。識別メタの抽出から保存までを実装して開始し、旧Z004試験履歴は本番に持ち込まない。旧importが存在すれば拒否を維持し、32 / 24 / pos / 41 / 55のpreflightはimport_identity_missing + settlement_datesで該当精算日を示す。参照先の商品がないことを理由に旧importを隠さない。
- 採らなかった案: (a)更新前の全取り漏らし解消は、存在しない本番履歴の更新手順を課し将来の後追いを閉ざすため不採用。(b)legacyだけ確認で許可は二重計上の判断を利用者へ戻すため不採用。(c)識別メタのbackfillは旧DBだけでは復元できず、原本再提示/hash一致による補完機能も本番履歴のない今は追加しない。既存のhash単位source参照backfillまで撤去する意味ではない。
- Matrixへ移行前sourceなし/メタ不足の拒否、日付つきpreflightのwire/UI、新形式どうしの後追い成功と抽出→保存のoracleを追加した。旧DBの分類・拒否・取消復旧は合成試験で維持する。開発/試験DBを作り直す場合は残すマスタを確認して売上・在庫移動と整合させる別作業であり、本runはDBを読取・変更していない。
- Owner Effort Budgetへ、round天井消化、owner escalation後の追加確認予定（Opus closure、是正差分のみ、未実施）、今回のowner決定を加えた介入実績と上限改訂案を記録した。数値は同節を正とし、上限引上げはowner承認待ち。Writerは承認・Plan Gate通過・phase前進を代行しない。
- Review-only skipped because: owner決定のdocs-only反映と同種の前提点検に限定する。独立broadや通常roundを追加せず、是正差分の正式確認はFable発注のOpus closureへ渡す。

#### 初導入前提の同種点検（発注68）

ADR全文、packetの現在契約、S1〜S6のproposed節で旧DB/移行/legacy/preflight/本番費用を照合した。機構の削除・弱体化が必要な箇所はなかった。

| 点検対象 | 扱い |
|---|---|
| ADR Consequencesの旧DB・多数の再確認・実データ件数受入 | 旧観測がある環境だけの費用/受入条件へ限定。初導入の本番に該当履歴を仮定せず、取込み側の拒否費用も対で追加 |
| ADR D8、trackingの旧観測分類、35のlegacy復旧 | 開発・試験/将来更新の互換機構として維持する前提を補足。存在するlegacyの検査は省略しない |
| ADR D6、20 / 24 / 32 / 35 / 40 / 41 / 42 / 73のlegacy取消/表示/回復 | 旧観測・旧movementがある場合の条件付き規則であり、D8の適用前提と両立。算術・操作・wire用途は不変 |
| 32 / 24 / pos / 41 / 55の取込みpreflight | 商品以外の旧取込みも検出し、欠けたsourceを候補なしへ落とさず日付を返す。初導入という申告で検査を飛ばさない |
| Matrixの互換性・本番移行preflight、packetのLedger/申し送り | 初導入と旧データ保有環境を区別。時刻証拠だけの不足の復旧を識別メタ不足拒否へ一般化していた文も分離 |
| 親DB/architectureの参照・30の商品preflight | 既存データ存在時の安全規則または個別正本への参照であり、現在の本番履歴を断定していない。変更不要 |

#### 発注68の検証

- AC1: `rg '^### SPEC-STK-TIME-D' docs/adr/2026-09-18-stocktake-time-evidence.md` → D1〜D9を確認。D3 / D8の移行前提、拒否と後追い、Consequencesの費用開示を相互照合した。
- AC2: `python3 scripts/probes/stocktake_time_model.py` → exit 0、`PASS: temporal bounds, causal receipt, clock conflict, zero-net split, revision, count/rollback lifecycle, legacy recovery, migration kinds, force_fill`。モデルは変更せず、新preflight/移行前importのoracleはruntime Matrixへ分離した。
- AC3: S1〜S6へ `rg -n '時点証拠契約|SPEC-STK-TIME'` を実行しproposedを確認。`rg -n '2026-09-19|本番|初導入' docs/project-memory.md` でowner原文と更新義務、`rg -n 'project-memory' AGENTS.md docs/PROJECT_HANDOFF.md` で既存の到達経路を確認した。32が日付一覧をproduceし、41→55へtyped fieldのまま渡す契約を照合した。
- AC4: `git diff --name-only 36891af8 -- src src-tauri` と `git diff --name-only a2a265bf -- src src-tauri` は空。同基準の90-traceability/model/workflow/template差分も空。変更docごとの `REQ-[0-9]+` tokenを基準 `36891af8` / 開始 `a2a265bf` とsort比較して一致。Workflow Stateのfieldも開始版とのdiffが空。project-memoryはScope/ACを先に拡張してから編集し、全変更を許可docsへ限定した。
- AC5/AC7: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-09-16-stocktake-count-baseline.md` → ERRORなし・既知PK6 WARN 3件のみ。初回の追加PK6警告は、重複していた予算数値をOwner Effort Budgetへの参照に整理して解消。`bash scripts/check-workflow-git.sh` と `git diff --check` 成功。
- AC6: 32の外部probe表（精算系列・時計対応・EJ完全性・商品同定）と42/73の監視/native失敗条件を確認し、未実施のまま保持した。51/60のmaster拒否表示とMatrixの対応oracleも維持している。
- 未実施/停止条件: runtime/migration/bindings、実機/Windows、本番・開発DBの読取/作り直し、追加Opus closureは本run外。機構変更を要する点検所見や是正作業のblockerはなし。介入上限改訂はowner承認待ち、正式Plan Gateとphase前進は未実施。push/PR操作なし。

### owner escalation 後の追加 closure と Plan Gate 通過（2026-09-19、Opus、対象 `b7f19514`、裁定 Coordinator = Fable）

- round 3 の残件（移行前 import が比較先になる場合の規則、費用記述）はいずれも CLOSED。P1/P2 = 0、Plan Gate 通過可。owner 決定は ADR D3 / D8・DB・IO / BIZ / CMD / UI・Matrix へ同じ意味で反映され、拒否は fail-closed 側で前提（本番にメタなし import が無い）が破れても二重計上は通らず preflight で見える。「初導入の申告で検査を省略しない」は ADR / 32 / 41 / 55 に明文。後追い取込みは日付で止まらず、D4 の保留・要再確認と整合する。
- 発注 68 の点検 3 は妥当: legacy 観測の移行・legacy 取消の復旧・legacy_movement_ceiling は維持、費用記述だけを旧観測を持つ DB へ限定。機構の削除・弱体化なし。
- Coordinator の追加確認（owner回答 2026-09-19の要旨: 営業中もPC作業があり、商品の登録・編集・廃番のいずれも営業中に起こり得る）: 棚卸し中の商品登録で明細を足す writer（BIZ-01 create_product ステップ 6）は、Matrix の `req205_count_schema_writer_atomic_rollout` が kind 明示の対象として観測している。
- follow-up P3（blocker ではない）: ADR Consequences の棚卸し側の移行費用の段落から「旧データを持つ環境の更新を追加作業なしとは説明しない」の literal が置換で消えた。実質要件は同段落に残る。owner 指示（2026-09-19「君が戻していいよ」）により Coordinator が同日に 1 文を戻した（設計の意味は不変、Plan Commit の後の docs 是正）。
- owner 承認 2026-09-19:「9で承認」（介入上限 9）。


### Final Review 初回 broad（2026-09-21、pass A Sonnet / pass B Opus、対象 `552a7be7`、裁定 Coordinator = Fable）

- Findings Freeze: 本節の指摘集合（`P2 4件 / P3 4件、全件accept`）で固定する。出所はPR #85の[pass A](https://github.com/higekiri-w/inventory-system-desktop/pull/85#issuecomment-5760084126) / [pass B](https://github.com/higekiri-w/inventory-system-desktop/pull/85#issuecomment-5760084479)。過去rallyの記録は変更しない。
- Gated Amendment 1適用後の発注70として是正。Workflow Stateのfield、Scope、ACは変更しない。GA1でAmendmentsの範囲が変わったため、次はCoordinatorが是正後headへ`broad 2本`を取り直し、現在版のclosureを確認する。今回の自己点検を独立Final Review通過とは扱わない。

| Finding / 種別 | disposition | 是正と根拠 / path |
|---|---|---|
| P2-1 / model bug・ADR drift | accept | ADR D6に商品別checked SUM・非0だけ補償・最初の吸収先へ一度を明記。scripts/probes/stocktake_time_model.pyのcancel_importを全void→純量補正に変更し、check_lifecycleへ純量0/非0とpending/appliedのoracleを追加。32とMatrixの規則へ一致 |
| P2-2 / producer欠落 | accept | ADR D3/D4/D8、pos-tables、tracking、23/24/32、architectureのIO/BIZ/MNT、DB_DESIGN、Matrix、Ledger/㉘申し送り。gate基準とfile境界を別所有にし、入力・適格条件・保存・昇格/失効伝播を確定 |
| P2-3 / 表示producer欠落 | accept | ADR D3/D8/D9、pos-tables、24/32/41/55、architectureのBIZ/CMD/UI、Matrix、申し送り。案(a)の拒否された未取込み資料に限定し、独立保存の拒否証拠からsettlement_missingを導出。stock_reviewに欠落状態を複製しない |
| P2-4 / 設定操作による迂回 | accept | ADR D4/D8/D9、master、20/30/32/35/40/41/51/55/60/73、architectureのBIZ/CMD/UI、SCREEN_DESIGN/UI_TECH_STACK、Matrix。true→falseを商品に記録、適用済みの新実測まで未調整を可視化。非連動化自体は許可 |
| P3-B1 / モデル証拠の過大主張 | accept | Matrixの未知始端補完と保存時cursor更新をruntime req401_unknown_start_no_midnight / req401_receipt_before_count_startへ割当。モデルは生成処理を持たないため、コメントを入力済み境界の分類assertへ限定 |
| P3-B2 / tautological assert | accept | モデルのABA箇所は相殺後の数量だけ、force_fill箇所は適用済み吸収先を保った取消後数量だけを主張。実装のcontext ABA/証拠非上書きはMatrixのruntime oracleを維持 |
| P3-A1 / 現在地drift | accept | backlogのSTK-1/STK-2だけをR3・implementing・時点証拠方式・Plan Gate通過日に同期。Plansの㉗ entryと同じFinal Review再取得/closureへ接続 |
| P3-A2 / 前提の二重所有 | accept | project-memoryのInitial deployment baseline行をStore Premises Factsへの参照と本番開始時追記義務に縮約。Store Premises Facts本文は不変。liveな㉘申し送りの追記先を揃え、過去Review Responseは保存 |

#### 委任範囲内の決定と理由

- P2-2: 別表pos_time_basesをgate基準の所有先とし、sourceと実測は同じ基準IDを参照する。fileの境界はBIZがsettled_at/精度と証明済み同系列前回sourceから導出する。基準の複製と無条件昇格を避け、失効を基準と参照sourceへ同じTXで伝えられる。初回の上限だけの成功、後着前回による下限追加、未証明/reset/メタ不足で導出しない条件をMatrixへ固定した。
- P2-3: 許容案(a)を採用。受領/拒否というアプリ内の確定事実だけを入力にでき、未提出の資料の存在や別売上の欠落量を発明しない。推奨案(b)のverified区間の欠番検出は本laneで提供しないとADRに明記する。EJの完全性検査は別契約のまま維持。拒否証拠はsourceに残し、表示所有はPosStockReadiness.issuesに固定した。
- P2-4: held履歴は非永続でsourceメタにも商品集合がないため、過去heldの検出を切替時の前提にしない。既存商品のtrue→falseを全て版付き記録とし、未調整issue・操作前警告・既存計数導線を接続する。未反映売上がない商品でも確認を要する保守的な費用は残る。activeの保存だけでは現在庫を補正しないので、適用済み新実測までissueを残す。非連動化禁止・確認checkbox・新しい計数APIは採らない。

#### 同種点検（S1〜S6、producerと迂回経路）

| 範囲 | 点検した接続 / 結果 |
|---|---|
| S1: 20/21/23/24 | parserメタ→source→BIZ境界、拒否証拠→準備照会、商品切替版→適用済み観測を接続。数量/版・movement ID・cursor・flag保存の既存producerは保持。sourceから元明細を復元する前提は追加しない |
| S2: 30/31/32/35/36 | 商品一括importも非連動化記録の対象、全非連動の共有JAN分岐にも未調整表示を残す。fix_integrityや通常入出庫は実測としてissueを消さない。active保存/force_fillによる早期解消を禁止し、通常確定は循環拒否しない。preview/commitの同一判定と証拠TXの独立性を維持 |
| S3: 40/41/42/43 | 商品get/update・一括import結果、準備issue/source_ids、stock_review/結果warningから既存CountRecoveryTargetへ伝播。CMDは基準認定を受け付けず、汎用設定やDB交換後の古いtokenで迂回しない。新しい回復actionは不要 |
| S4: 51/60/55/65/73 | 操作前警告→成功/拒否→refetch→再訪/再起動→回復を点検。商品off後の売上のみ成功でも未調整を維持。資料拒否の表示は再実測で消さない。記録詳細の訂正・明細なし・active優先の制限は保持 |
| S5: master/transaction/pos/tracking/DB_DESIGN | 基準とsource/実測のFK型を一致、rawメタ/精度/精算日・拒否証拠・商品切替版の保存先を確定。参照信用の失効と昇格を再評価する規則を対応させた。入出庫TX・旧履歴/評価額の非変更は保持 |
| S6: ARCHITECTURE/FUNCTION_DESIGN/SCREEN_DESIGN/UI_TECH_STACK・各task | 親索引と責務を辿り、BIZのproducerをIO/CMD/UIへ移さないことを確認。認定/失効、拒否応答、非連動化、計数保存/確定のquery再取得もruntimeへ明記。現行本文・図を新方式へ読み替えない |

上記では凍結指摘の是正と直接波及を閉じた。別の是正・機構削除・新GAを要する同型の未解決事項は見つからなかった。実装配線・実機前提の検証完了を意味しない。

- Review-only skipped because: 発注70が単一thread・直列・subagent不使用を指定。正式な独立監査はCoordinatorが是正後headへ発注し、Writerは代行しない。

#### 発注70の検証

- AC1: `rg '^### SPEC-STK-TIME-D' docs/adr/2026-09-18-stocktake-time-evidence.md` → D1〜D9、同patternの `rg -c` → `9`。
- AC2: `python3 scripts/probes/stocktake_time_model.py` → exit 0、`PASS: temporal bounds, causal receipt, clock conflict, zero-net split, revision, count/rollback lifecycle, legacy recovery, migration kinds, force_fill`。追加caseだけ先に実行した修正前版はcheck_lifecycleの補償一覧assertで失敗、修正後は成功。
- AC3/AC6: S1〜S6へ `rg -n '時点証拠契約|SPEC-STK-TIME'` を実行し、全対象からproposed契約へ到達。`rg -n '2026-09-19|本番|初導入' docs/project-memory.md` と `rg -n 'project-memory' AGENTS.md docs/PROJECT_HANDOFF.md` で確認日・原文・追記義務と入口を確認。32の外部probe表、42/73のWindows監視不成立・native失敗/復旧条件を点検し、実機は未実施と維持した。
- AC4: `git diff --name-only 36891af8 -- src src-tauri` と `git diff 36891af8 -- docs/function-design/90-traceability.md docs/DEV_WORKFLOW.md docs/AGENT_OPERATING_MANUAL.md docs/templates` → 空。変更docごとの要求tokenを基準36891af8/開始644090cfと多重集合比較して一致。Workflow State field・Scope・AC・GA1とStore Premises Facts本文は開始版と一致、過去Review Responseはprefix一致。モデル差分はGA1の列挙箇所のみ。
- mutation: TMPDIR配下の隔離copyでcancel_importだけを是正前のmovement単位へ戻し、check_lifecycleの補償一覧assertがAssertionError（exit 1）になることを確認。未知始端補完はP3-B1でruntimeへ割当て直したため、本発注の対象外条件を適用してmutation未実施。tracked成果物へmutationは適用していない。
- AC5/AC7: `bash scripts/doc-consistency-check.sh` / `bash scripts/doc-consistency-check.sh --target plan` → ともにexit 0、`ERROR 0 / WARN 3（既知PK6のみ）`。初回は追記の非箇条書きskip記録とproposed列の表記で追加WARNが出たため、既存の箇条書き形式と表/列を分けた参照表記へ是正して再実行した。検査scriptと既知履歴は変更なし。`bash scripts/check-workflow-git.sh` → PK5/STATECAP OK、`git diff --check` → exit 0。
- 旧文言sweep: verifiedの所有・TimeEvidenceの旧shape・精算欠落の表示・古いSTK現在地/初導入参照をrepoでrg照合。変更対象の旧契約のlive残存はなし。packetの不採用を記した遷移履歴と過去Review Responseは保存した。
- 未実施: runtime/SQL migration/bindings/traceability生成、Rust/frontend全量検証、Windows/EJ/時計/系列の実機、独立Final Review。push/PR操作/helperのrecord・ready・mergeは行わない。是正を停止した項目・Scope/ACとの衝突はなし。


### Final Review broad `2 回目`（2026-09-21、pass A Sonnet / pass B Opus、対象 `7de29376`、裁定 Coordinator = Fable）

- 出所はPR #85の[pass A](https://github.com/higekiri-w/inventory-system-desktop/pull/85#issuecomment-5760937476) / [pass B](https://github.com/higekiri-w/inventory-system-desktop/pull/85#issuecomment-5760937847)。前回指摘は両passで`P2 4件 / P3 4件、全件CLOSED`と確認された。今回のFindings Freezeは`P2 3件 / P3 5件、全件accept`の下記集合で固定する。
- 発注71のdocs是正。開始HEADは`0d8ae025`、tracked tree clean。Workflow Stateのfield・Scope・AC・Gated Amendments・過去Review Response・Store Premises Factsは不変。次はCoordinatorが是正後headへ`broad 2本、rally 3回目（round天井）`を取り直す。Writerの自己点検を独立Final Review通過とは扱わない。

| Finding / 種別 | disposition | 是正先と結果 |
|---|---|---|
| P2-N1 / producer・識別意味の欠落 | accept | ADR D1/D3/D4/D8/Consequences、tracking/pos、20/21/24/32/35/41/42/43、architectureのIO/BIZ/CMD/MNT、Matrix、Ledger/㉘申し送り。実測epochとPOS基準FKを分離し、producer・TX・比較条件・失効・費用を確定 |
| P2-N2 / 解消条件と案内のdrift | accept | ADR D4、32/35/41/51/55/73、UI_TECH_STACK、architectureのBIZ/CMD/UI、Matrix。切替後のmeasured pendingだけ確定を案内し、それ以前の入力は数え直しへ。既存の準備issueの説明を商品ごとに渡す |
| P2-N3 / Scope・AC・claimed validationの不一致 | accept | 是正の向きはownerの公開範囲決定に合わせGA2で処理済み。project-memoryのInitial deployment baselineへ削除許可ではない但し書きを復元。ADR/Matrix/本packetのliveな原文参照を要旨基準へ変更し、GA2後AC3を下記で再実行。過去の検証記録は当時の記録として保存 |
| P3-N1 / 共通経路の強制不足 | accept | ADR D4、20/30、architecture BIZ、Matrix、㉘申し送り。pos_stock_syncの共通repo更新で旧値読取り・変更時の版更新・true→false時だけ切替版保存を強制し、汎用直接setを残さない |
| P3-N2 / sourceと日付・理由の対応欠落 | accept | ADR D3、32/41/55、architecture BIZ/CMD/UI、Matrix。source単位のissue、ID/日付各単一要素、既存説明に同じsourceの拒否理由を載せる |
| P3-N3 / 準備完了表示の制限欠落 | accept | 55のfile選択前表示へ、準備完了は未提出の精算資料がないことを意味しないと明記。ADR/Matrixの既存oracleと一致 |
| P3-N4 / JAN変更によるheld解除 | accept（残存リスクを明記） | ADR Consequences。一括importのJAN変更（NULL化も含む）で候補から外れ、在庫未調整でもheldが消える経路を受容済み残存リスクとして保持 |
| P3-A / 初導入但し書きの欠落 | accept | P2-N3と同じproject-memoryの但し書き復元で対応 |

#### 決定と最小の契約面

- P2-N1は案(i)。実測はPOSのマシン/精算系列に属さず、MNTが発番するPC時計epochを既存time_basis_idへTEXTで保存する。BIZ-06のbeginが固定し、saveの単一商品TXが検証/保存する。新しいPOS基準でも同じepochなら比較可能とし、実測窓へPOS基準の適用期間を課さない。epoch不一致/NULLでは時刻Unknown、受領Beforeは維持する。連続性を証明できない再起動/sleep等の費用はADRへ記録した。
- 新たな保存fieldは既存TimeBasis JSONのpc_clock_epochのみ。BIZ-03のgate認定TXがMNTから保存し、24の保存/基準JOIN、32の分類・準備照会がconsumeする。欠落/不正/不一致ではverifiedでも使わず、Matrixのreq401_pc_epoch_across_basis_renewalに成功/拒否/保存故障のoracleを置いた。fieldなしで実測とPOS基準を別概念にするだけでは、変換先のPC epochを証明できない。起動のたびに全基準を無条件更新する別の同期機構も増やさず、既存JSONの識別値で閉じた。
- 新しい表・列・issue code・named type・関数は追加しない。既存CountContext/CountEnvironmentの時計対応をepochとして具体化し、既存get_pos_stock_readinessのBIZ入力へCountEnvironmentを渡す。公開commandの引数は不変。source側time_basis_idのINTEGER FKは保持する。
- P3-N2はsource単位の案。集約された配列から日付と理由の対応を復元させず、既存issueの説明を使えるため新しいreason型は不要。P2-N2も商品単位のissueに既存説明を載せ、51/73が既存準備照会を読む。UIへ版fieldやaction/codeを増やして業務判定を複製しない。
- P3-N4は残存リスク記録の案。既存pos_sync_disabled_revisionはtrue→falseだけを表し、false→false/新規falseを対象外にし、UIも非連動化を説明する。連動を維持したJAN変更を載せると保存意味・再有効化guard・表示・oracleまで広がり、既存経路への文の追加だけでは整合しない。個別フォームはJAN read-only、一括importだけの経路として許可されたリスク明記で閉じる。
- findingでない残る不確実性も記録した。51/73とADR/Matrixへno_count_targetから棚卸し開始→active_countの次操作を追記。取込み済みimportが後から衝突した前回sourceの下限に依存する場合の自動再判定/訂正が非対応である点をADR Consequencesへ明記した。

#### 同種点検

| 点検観点 | 現物と結果 |
|---|---|
| 保存値のproducer | pos_time_basesはgate→BIZ-03→24、file境界は32→24、実測はMNT→35→20、拒否証拠は32→24、切替版は30→共通20、flagは取込みTX→20。epochの保存/JOIN/現在値照合を追加し、基準認定数に依存する実測選択を除去。派生N/NはRのepochをコピー。点検範囲の全保存値についてproducerを追跡できた |
| 解消条件と案内 | sync_disabled_unreconciledは適用済み観測の版、recount_requiredは同規則を共用。切替前pendingの確定・auto_filled・同版/版なしでは解消しない。商品単位の既存issue説明で案内を接続。settlement_missingは当該sourceのactive成立だけで非表示、再実測/取消/再起動を解消と表示しない |
| heldの別解除経路 | runtimeのupdate_product/commit_import/toggle_discontinue、価格変更、PLU対象/slot/export確認、find_by_jan_code、棚卸し開始をread-only確認。廃番/PLU状態はJAN候補を除外せず、価格/取引先/名称はZ004のJAN照合を変えない。商品追加による共有はD4のguard、preview後の候補変化は再previewで検出。JAN変更/NULL化はP3-N4の残存リスクに含めた。通常APIから商品物理削除する経路は見つからず、test内DELETEを製品経路とは数えない |
| 復旧の迂回 | 通常入出庫/fix_integrity/force_fillは新しい適用済み実測を作らず、切替記録を解消しない。取消は元flagだけを解消し、独立再実測を消さない。棚卸し開始は明細を作るだけで、未調整を解消しない。実DB復元は許可済みの別ライフサイクルで、本是正のheld解除手段にしない |
| 前提・旧文言 | 原文所在のliveな主張を要旨へ同期。project-memoryは参照行の但し書きだけを追加し、Store Premises Factsと出典ラベルは不変。ADR/32/Matrixは現在の要旨と初導入条件を参照し、消えた旧文の内容を前提にしない |

- Review-only skipped because: 発注71が単一thread・直列・subagent不使用を指定。独立Final ReviewはCoordinatorの次の発注で行う。

#### GA2後AC3と発注71の検証

- AC1: `rg '^### SPEC-STK-TIME-D' docs/adr/2026-09-18-stocktake-time-evidence.md` → D1〜D9。同patternの`rg -c` → `9`。
- AC2: `python3 scripts/probes/stocktake_time_model.py` → exit 0、`PASS: temporal bounds, causal receipt, clock conflict, zero-net split, revision, count/rollback lifecycle, legacy recovery, migration kinds, force_fill`。モデルは変更していない。epochのproducer/保存・UI案内の新oracleはruntime Matrixにあり、モデルPASSで実装済みとは扱わない。
- AC3: S1〜S6へ `rg -n '時点証拠契約|SPEC-STK-TIME'` を実行しproposedの正本へ到達。`rg -n '2026-09-19|本番|初導入' docs/project-memory.md` で確認日・owner回答の要旨・本番開始時の追記義務を再確認。`rg -n 'project-memory' AGENTS.md docs/PROJECT_HANDOFF.md` で入口を確認。`rg -n 'permission|許可' docs/project-memory.md` はInitial deployment baselineの但し書きへ一致。非archiveの公開docsにowner発言の禁止原文は一致なし。
- 原文所在sweep: `rg -n 'owner原文' docs/ --glob '!docs/archive/**' --glob '!docs/evidence/**'` の残りはproject-memoryの出典ラベル、packetの過去Review Response/GAの履歴のみ。ADR/Matrix/Required Design Artifactsのliveな原文参照は要旨へ変更した。過去のclaimed validationを現在の成立証拠に流用せず、本節のGA2後再検証を正とする。
- AC4: `git diff --name-only 36891af8 -- src src-tauri`、`git diff 36891af8 -- docs/function-design/90-traceability.md docs/DEV_WORKFLOW.md docs/AGENT_OPERATING_MANUAL.md docs/templates`、`git diff 7de29376 -- scripts/probes/stocktake_time_model.py` → 空。変更docごとの要求tokenは基準36891af8/開始0d8ae025と多重集合で一致。保護節・履歴・Store Premises Factsは開始版と一致。
- mutation: TMPDIRの隔離copyでcancel_importだけを是正前のmovement単位補償へ戻すと、check_lifecycleの補償一覧assertがAssertionError（exit 1）。trackedモデルは不変。GA1の既存差分は取消純量化/対応case/検出力のないassertとコメントの限定是正である。
- AC5/AC7: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan` → exit 0、`ERROR 0 / WARN 3（既知PK6のみ）`。追記した点検結果の否定文を未解決マーカーと拾うplan検査エラーが出たため、意味を保った肯定文へ直して再実行した。エラー出力の途中切断によるUTF-8 decode失敗は生byte保存で切り分けた。`bash scripts/check-workflow-git.sh` → PK5/STATECAP OK。`git diff --check` → exit 0。
- AC6: 32の外部probe表（精算系列/時計対応/EJ完全性/商品同定）、42/73のWindows監視不成立と失敗/復旧条件を再点検。51/60のmaster拒否とMatrixのoracleも保持した。
- 未実施: runtime/SQL migration/bindings/traceability生成、Rust/frontend全量、Windows/EJ/時計/系列の実機、独立Final Review。push/PR操作/helper record・ready・mergeは実施しない。停止項目・Scope/ACとの衝突なし。

### Final Review broad `3 回目`（round 天井）と disposition（2026-09-21〜22、pass A Sonnet / pass B Opus、対象 `bf107424`、裁定 Coordinator = Fable）

- 結果: broad `2 回目` の finding は pass A / pass B とも全件 CLOSED と確認。新規は pass A が P1/P2 なし・P3 `2 件`、pass B が P2 `1 件`・P3 `2 件`。3 回の broad を通じて、誤った前後判定を通す経路（fail-open）は見つかっていない。
- 残った P2: PC 時計 epoch が process 起動ごとに発番されるため、時刻による前後判定・EJ 時刻分割・POS 基準の使用が「gate 認定と実測と分類が同じ app session に収まる場合」にしか成立しない。日常運用では POS 基準が次回起動で使用不可になり、過去に実測した商品を含む日次資料は毎回判定不能になる。ADR Consequences はこの頻度と、恒常運用の前提（時計 / EJ）へ到達できないことを書いていない。
- 同型性: broad 初回の P2-2（file 境界の producer なし）、`2 回目` の P2-N1（実測側の時刻識別と POS 基準の併合）、今回と、同じ一点（PC の時計と POS の時計の対応を、再起動を跨いで信用できるか）の周りに形を変えて出た。docs の是正を重ねて閉じる種類の問題ではないと判断した。
- disposition: DEV_WORKFLOW Review Rules の round 天井により次の broad round は開始しない。残 finding は **owner escalation**。Coordinator は (A) 本 lane は費用と未解決を正直に書いて閉じ、対応の取得方法を別の design lane に切り出す、(B) 本 lane の中で設計し直す（GA と新しい rally が要る）を提示し、(A) を推奨した。
- 外部の設計相談（owner が実施、2026-09-21〜22）の要点: 確定済みの前後関係を、判定時点の PC 時計の状態へ従属させる必要はない。保存すべきは「実測が POS のどの精算境界より前 / 後か」という関係と、その根拠。実測は必ずどれか 1 つの精算区間の中に入るので、その区間は Z004 だけでは解けず、取引単位の資料（EJ）が要る。受領後の数え直しだけでは、毎回数え直す循環から抜けられない。現在の時計状態との一致条件を単に外すだけでは、未証明の後着資料まで信用することになり不足。
- owner 決定（2026-09-22）: (A) で閉じる。介入上限 `10` を承認。実機確認が要る事項は後回しにし、他の作業を先に進める。
- 本 lane での是正（発注書 72、Writer = Codex。Scope / AC は不変、Gated Amendment なし）:
  - ADR の時刻判定に関わる部分（gate による POS 基準の認定、`pos_time_bases`、source の昇格 / 失効の伝播、PC 時計 epoch の一致条件、EJ 時刻分割）に、同じ app session に限って成立する安全性の設計であること、v1.0 の恒常運用は未解決であること、runtime の実装仕様としては確定していないことを明記する。Consequences の費用記述を事実に合わせる。
  - ㉘ 申し送り: 上記の機構を実装しない。安定部分（商品単位の即保存、revision、受領記録と精算同一性 guard、取消の補償、拒否と回復、初導入の前提、非連動化の記録）を実装対象とする。次の design lane =「実測と POS 系列の対応を取得・保存する」。実機で確かめる事項を列挙する。
  - P3 `4 件`（41 の分類経路への `CountEnvironment` の受渡し、backlog STK-1 / STK-2 の現在地、`time_basis_id` の同名別物の注意、起動頻度と `clock_unverified` の恒常化を店主向け案内と Matrix へ反映）。
- 次: 是正後の head に対し、独立 closure を 1 本（Opus）。新しい broad round は開始しない。


### 発注書 72 の是正（2026-09-22）

開始HEADは`b5822644`、指定branch / worktreeのtracked treeはclean。直前のCoordinatorのdispositionをそのまま保持し、決定済みの適用範囲と費用を反映した。Workflow Stateのfield、Scope、AC、Gated Amendments、Owner Effort Budget、過去Review Response、project-memory、設計モデルは変更していない。

- pass BのP2とpass AのP3（起動頻度）: `docs/adr/2026-09-18-stocktake-time-evidence.md` のStatus直下「適用範囲の但し書き」に節別の線引きを置いた。Consequencesに同じapp sessionの条件、clock_unverifiedの恒常化、valid_untilの実効寿命と日次運用の不足を明記。外部設計相談は次のlaneの出発点として記録し、番号印字を未検証候補に留めた。Revisit Triggerに置換先を接続した。
- pass AのP3（同名別物）: 本packet「後続 runtime lane ㉘ への申し送り」で実測TEXT epochとsource INTEGER FKを区別し、両列とも㉘では作らず次の保存形で同名を避けると明記した。同節に実装するもの / しないもの、残る保留・再実測の費用、次のdesign laneと実機確認の項目を置いた。Contract Coverage LedgerとTrace Matrixの時刻部分を対象外として標示し、確定部分と既存行は保持した。
- pass BのP3（CMD受渡し）: `docs/function-design/41-cmd-pos.md` と `docs/architecture/cmd-task-specs.md` に分類経路のCountEnvironmentは次のlaneで決めることと、BIZからMNTを直接呼ばないことを記録した。
- pass BのP3（現在地）: `docs/Plans.md` の㉗の次の行動 / Wave Registry laneと、`docs/backlog.md` のSTK-1 / STK-2をround天井・owner disposition・closure → Ready → mergeへ同期した。backlogの着手対象へ次のdesign laneを起票した。
- source / UI / Matrix: 下記sourceのproposed節は参照文のみを追加し、契約本文を保持した。`30-biz-product-service.md` / `55-ui-csv-import.md` はclock_unverifiedを㉘の準備表示へ出さないことを明記。`docs/plans/test-matrices/2026-09-18-stocktake-time-evidence.md` は時刻oracleとlifecycleの印・混在行の区分を追加し、Residual Test Gapsに実測後販売の時刻Afterが㉘では未実装となることを記録した。OS監視・generationによる計数context失効は確定部分に残した。

#### 対象箇所の全数と参照

指定command `rg -n 'pos_time_bases|pc_clock_epoch|TimeEvidence|time_basis_id|clock_unverified' docs --glob '!docs/archive/**' --glob '!docs/plans/**'` の着手時出力は`39 行 / 13 file`。内訳はADRが`11 行`、S1〜S6のsourceが`28 行 / 12 file`。一致行の開始版lineは以下のとおり（件数はtoken数ではなくrgの出力行数）。

- `docs/adr/2026-09-18-stocktake-time-evidence.md:21,37,39,95,97,99,107,212,218,219,221`。
- `docs/db-design/pos-tables.md:12,13,27,29,31,33`、`docs/db-design/tracking-system-tables.md:15,19,23,40`。
- `docs/function-design/20-io-product-repo.md:14,15,18`、`21-io-inventory-repo.md:8`、`24-io-csv-import-repo.md:13,15`、`32-biz-csv-import-service.md:9,25,27`、`35-biz-stocktake-service.md:17,25,38`、`42-cmd-sales-stocktake.md:15,25`、`73-ui-stocktake.md:17`。
- `docs/architecture/biz-task-specs.md:9`、`io-task-specs.md:5`、`mnt-task-specs.md:8`。

ADRの但し書きでADR内の一致を扱い、sourceはproposed節の冒頭で時刻部分を限定して参照した。指定patternに現れないepoch / CountEnvironment / 準備表示も周辺検索・本文で確認し、`43-cmd-settings-log.md` / `41-cmd-pos.md` / `30-biz-product-service.md` / `55-ui-csv-import.md` / `architecture/cmd-task-specs.md`を補った。

`rg -n '適用範囲の但し書き' docs/function-design docs/db-design docs/architecture` → 参照は`19 箇所 / 17 file`。是正後の参照位置は、DBのpos / trackingが各`:5`、function-designの20 / 21 / 24 / 30 / 32 / 35 / 42 / 43 / 73が各`:5`、41が`:7,:11`、55が`:8`、architectureのbiz / io / mntが各`:5`、cmdが`:5,:11`。追加文を含む指定patternの再検索は`49 行 / 16 file`で、開始時の全一致はADR本文または上記source参照の適用範囲内に残る。

#### 実施した検証

- AC1: `rg '^### SPEC-STK-TIME-D' docs/adr/2026-09-18-stocktake-time-evidence.md` → D1〜D9、出力`9 行`。
- AC2: `python3 scripts/probes/stocktake_time_model.py` → `exit 0 / PASS`。モデルの成功はruntime実装や恒常運用の成立を意味しない。
- AC3: S1〜S6の対象sourceへ `rg -n '時点証拠契約|SPEC-STK-TIME'` を実行してproposedの入口と層間の対応を確認。`rg -n '2026-09-19|本番|初導入' docs/project-memory.md` と `rg -n 'project-memory' AGENTS.md docs/PROJECT_HANDOFF.md` で確認日・回答要旨・本番開始時の追記義務・初導入が削除許可でない但し書きと入口を確認した。
- AC4: `git diff --name-only 36891af8 -- src src-tauri`、`git diff 36891af8 -- docs/function-design/90-traceability.md docs/DEV_WORKFLOW.md docs/AGENT_OPERATING_MANUAL.md docs/templates`、`git diff 7de29376 -- scripts/probes/stocktake_time_model.py` → すべて空。基準`36891af8`からの既存モデル差分はGA1の限定是正のまま。変更docの要求tokenは基準`36891af8` / 開始`b5822644`と多重集合で一致。保護節と過去記録は開始版と一致し、sourceの旧本文行は順序も含め全て保持した。
- AC4 mutation: TMPDIR配下の隔離copyだけでcancel_importを`36891af8`の明細ごとの補償へ戻した。実行は`exit 1 / AssertionError`となり、check_lifecycleの補償一覧assertで検出した。trackedモデルへmutationは適用していない。
- AC5 / AC7: `bash scripts/doc-consistency-check.sh` / `bash scripts/doc-consistency-check.sh --target plan` → `exit 0 / ERROR 0 / WARN 3（既知PK6のみ）`。新規追加行のM3警告語は`0 件`。`bash scripts/check-workflow-git.sh` → PK5/STATECAP OK。`git diff --check` → `exit 0`。
- AC6 / AC7: 32の外部probe表の精算系列・時計対応・EJ完全性・商品同定と不成立時の動作、42 / 73のWindows監視・native失敗 / 復旧条件を点検。51 / 60のstocktake_guardとMatrixのmaster write、移行前import拒否・日付preflight・新形式後追いoracleを保持した。実機は未実施。
- 公開範囲: 発注書の禁止原文patternによる非archive docsの検索は`0 件（rg exit 1）`。新しいowner発言原文は転記していない。
- Review-only skipped because: 発注書72が単一thread・直列・subagent不使用を指定。Writerの自己点検は独立closureの代替ではなく、Coordinatorが是正headにOpus closureを発注する。
- 未実施: runtime / migration / bindings / traceability生成、Rust / frontend全量、実機、独立Final Review、push / PR操作、helper status / record / ready / merge。PR操作禁止の指示に従い、このrunのhelper照会も行わず、serverの現状態は新たに確定していない。
- 停止した項目・Scope / ACとの衝突はなし。後続の時刻経路はowner dispositionによる実装対象外であり、このrunの是正漏れではない。
