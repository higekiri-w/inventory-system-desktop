# ruleset応答の既定項目によるhelperの互換性問題を修正する

## Workflow State

- Phase: archive
- Evidence Mode: legacy
- Risk: R3
- Execution Mode: codex-only
- Plan Commit: 8dd32dab7428862d093071c38cdb6189772bc02e
- Amendments: none
- Coordinator: owner（起草は現在のCodex）
- Writer: 現在のAstra（owner指定。D-087により起草・実装・検証・状態記録を一貫して担当）
- Plan Reviewer: Sonnet high（独立Plan Review完了、実効primary modelはSonnet 5をmetadataで確認。highは要求effort）
- Final Reviewer: Sonnet high + Opus high（独立fresh contextのDouble Audit）
- Reviewed Content HEAD: 577b09b4e4cef4e41ad323dfbd0a45ba1d88f190
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required
- Human Gate: none

main保護は未有効のためlegacyで修正を完成させる。旧bootstrapのarchiveは変更せず、このpacketを現在の修正範囲とする。ownerは計画採用の判断に対し、Astra自身で作業を進め、独立レビューだけSonnet / Opusへ依頼すること、およびworkflowに但し書きを残すことを指示した。D-087を適用し、同じsessionで起草・実装・必要なpacket/状態記録を担当する。追加したworkflow文言は限定Plan Reviewで確認してから実装へ進む。非Codex reviewer利用不能時は可用性規定に従いpendingとし、自己承認しない。

## Owner Effort Budget

- 介入回数上限: 12（マージ検証整理のchange全体、owner 2026-09-14承認の規範値）
- 実働時間上限: 30分（規範値、実績は未実測）
- relay 往復上限: 2（規範値）
- Plan Review round 天井: 3（規範値）

旧bootstrapからの累計をリセットしない。試験承認まで介入7回、上限変更が8回目、計画をAstra自身で進める指定と但し書き追加が9回目。残る判断はReady、mergeと機械的closeout、本番有効化。次の承認依頼は介入10回目 / 予算12回。詳細は旧[packet](2026-09-14-merge-evidence-simplification.md)と現owner指示。elapsed hands-on timeは未実測。レビューの原文は機械経由で保存し、ownerの手作業relayを前提にしない。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

helperのReady/mergeを許可する照合処理が変わるためworkflow gate change。製品・DB・本番設定の変更は含めない。

## Goal

Goal Invariant:

### 最小完了条件

観測したGitHubの既定項目を含む正常なruleset応答をhelperが受理し、必要な保護の欠落・変更は引き続き拒否する。修正を旧gate下でmergeして、本番有効化の判断へ渡せる状態にする。

### 失敗定義

未知fieldや異なる既定値、弱められた保護を通すこと。desiredをPR側で差し替えて許可を減らせること。正常な観測応答が引き続き止まること。有効化前に新modeやbypassを使うこと。

### 非目的

設定方式の再設計、API応答の汎用正規化、CI/helper全体の作り直し、他のP3・製品lane・依存更新・実データ処理。

## Scope

- S1: `scripts/pr-gate.py`の`Gate.rules`でMG-D1aの限定照合を実装する。既存`copy`と共有経路を使い、設定・記録・CI評価の他の処理を増やさない。
- S2: `scripts/tests/pr-gate.test.py`の既存CLI fixtureを拡張する。API応答の既定項目をdesiredとは独立したliteralで与え、status/Ready/mergeの正負例へ接続する。raw live API全量はcommitしない。
- S3: `docs/agent-guidance/merge-evidence.md`のMG-D1a、`docs/decision-log.md`のD-086、本packet/Matrix、`docs/Plans.md`を同期する。D-086の採用状態はowner Plan Gate後に記録する。
- S4: legacyの検証・独立Double Auditを完了する。公開/Ready・mergeはそれぞれowner判断後。通常docs-only closeoutへ移送し、追加の再帰closeoutを作らない。
- S5: owner指定のD-087を`docs/AGENT_OPERATING_MANUAL.md`へ記録し、`docs/DEV_WORKFLOW.md`から参照、`docs/decision-log.md`と本packet/Matrix/Plansを同期する。Astraの一貫担当だけを追加し、独立レビュー・Plan Gate・ownerの裁定を変更しない。

## Non-scope

`.github/merge-gate-ruleset.json`とGitHub設定は不変更。新たなnative拒否試験・ref/ruleset作成・削除は実行しない。将来の本番適用は既存MG-D11と別の具体的owner承認に従う。アプリ/DB/実POS/secret、global設定、classifier/CI/PK5、他のP3を変更しない。

## Acceptance Criteria

- AC1: 既定項目なし、許容項目の片方だけ、両方を含む応答で`pr-gate.test.py`のstatusにrules blockerがなく、正当なReady/mergeが成功する。現行実装で両方の正常回帰を先にREDにする。
- AC2: `required_reviewers`が非空・null・異なる型、追加approval項目がfalse/null/文字列/数値の場合は非0またはrules blockerを返し、Ready/mergeの実呼出がない。
- AC3: `scripts/tests/pr-gate.test.py`で未知parameter、既存のparameter drift、rule欠落、app/context/strict不足、bypass/inactive/target/conditions/name不一致を拒否する。既存testを保持し、互換応答と同時に弱化を与えた場合もReady/mergeが非0となり、gh pr呼出がない。
- AC4: `scripts/tests/pr-gate.test.py`でdesired側が許容名の項目を明示するときは比較から除かず、一致は受理、不一致は拒否する。PR側policyで必要条件を減らす経路を増やさない。API応答とdesiredの元objectの呼出前後一致をassertする。
- AC5: live応答から既知の追加項目の値・型だけを取り出し、`scripts/tests/pr-gate.test.py`の既存合成fixtureへdesiredと独立したliteralとして注入して、現行の不一致と修正後の通過を確認する。probe固有のID/name/日時/node_id/_links等はtracked fixtureへ移さない。保存応答全体のreplayは別のignored local検証として扱い、main用name/refへの置換を記録する。既定値検査を除く実mutationでAC2が非0、無変更positive controlはexit 0になることを確認する。
- AC6: `python3 scripts/tests/pr-gate.test.py`、`bash scripts/tests/run-workflow-tests.sh`、必要なdoc/PK5、legacyのCLEANな`bash scripts/local-ci.sh full`、Sonnet/Opusの独立監査を通す。Readyのexact HEADで旧L1/PR/hosted CI一致を満たす。本番適用済みとは報告しない。
- AC7: `docs/AGENT_OPERATING_MANUAL.md`のD-087とDEV_WORKFLOWの参照がowner指定の一貫担当を表し、Fableの分業、別vendor Plan Review、Double Audit、Plan Gate、Gated Amendment、ownerの裁定が維持されることを独立レビューとdoc/workflow検証で確認する。

## Design Sources

- [MG-D1a・D1/D8/D11](../../agent-guidance/merge-evidence.md)、[D-085/D-086](../../decision-log.md)。
- [workflow](../../DEV_WORKFLOW.md)、[CI](../../ci.md)、[役割](../../AGENT_OPERATING_MANUAL.md)、[profile](../../project-profile.md)。
- D-087（owner指定のAstra一貫担当と既存gateの維持）。
- 実物: `Gate.rules`、`Gate.status`、`Gate.mutate`、`CLI.setUp`、`CLI.assert_rules_blocked`。
- 製品のARCHITECTURE/FUNCTION/DB/SCREEN契約は非接触。

## Required Design Artifacts

変更するJSON境界とdurable decisionはMG-D1a/D-086で定義した。アプリの関数・DB・画面・生成物の追加設計は該当しない。

## Registration / Generation Obligations

新規runtime module・command・test entryは追加しない。既存`pr-gate.test.py`はshared workflow suiteから実行される。新packetはPlansから、Matrixは本packetから参照する。bindings/routes/traceability生成は非接触。

## Design Intent Trace

| Spec | Source / Decision | 意図 | 実装先 | 検証 |
|---|---|---|---|---|
| SPEC-MERGE-EVIDENCE | MG-D1a / D-086 | 既知の応答だけを扱いdrift検出を維持 | Gate.rules | AC1〜5 |
| SPEC-MERGE-EVIDENCE | MG-D1 / D8 | current mainのpolicy、共有経路、freshな保護確認を維持 | Gate.rules/status/mutate | AC3〜4 / CLI |
| SPEC-MERGE-EVIDENCE | MG-D11 | 有効化前のlegacy検証を維持 | 修正PR / closeout | AC6 |
| D-087 | Agent Operating Manual §3.2 | owner指定の担当形と独立性を両立 | S5のworkflow docs | AC7 / 限定Plan Review / Double Audit |

## Design Intent Audit

sourceに動機・限定した値/型・棄却案・見直し条件を置いた。未確認項目の一般的な意味や送信APIの受付を推測せず、観測した応答だけを扱う。未知のserver変更を自動吸収する保証はしない。

## Impact Review Lenses

外部adapter/core: GitHub JSON応答とdesired照合の境界。Fact/design: 追加fieldの実測と許容方針を分離。Lifecycle: GET→比較用copy→許可/拒否、改版時のfresh取得は既存経路を維持。Replacement/環境・再現性: 新依存なし、未観測の応答形は拒否。Data safety: 合成fixtureと公開可能な設定形のみ。Operator/会計/実機: 製品非接触。manual: 保存応答のreplayと将来の本番read-backを区別する。

## Design Readiness

MG-D1aで実装に必要な条件を定義済み、採用はPlan Gate待ち。PR必須/CI/app/strict/bypass条件は既存sourceを継承する。公開ドキュメントで確認できないfieldの意味は本修正の入力条件にせず、送信policyを維持する。

## Contract Probe

- `.local/merge-evidence/activation-20260914/probe/reproduce-rules.py`を`python3`で実行 → `REPRODUCED: captured GitHub response -> main ruleset drift: rules`、既定項目だけ除いたcontrol → `CONTROL PASS`。main用name/refの置換を含むlocal replayで、production適用の証拠ではない。
- native拒否・成功・cleanupは[PR #57](https://github.com/higekiri-w/inventory-system-desktop/pull/57) / [PR #58](https://github.com/higekiri-w/inventory-system-desktop/pull/58)とlocal `probe/RESULT.md` / `result.json`に保存済み。policy不変更の本修正で同じnative負例を再実行しない。liveの設定応答は本番適用のread-backで再照合する。

## Contract Coverage Ledger

| 契約 | 実装先 | Automated test | L3 / Non-scope |
|---|---|---|---|
| MG-D1a 既知fieldの不在/許容値/型 | Gate.rules | 新規CLI正常・値/型負例 | 保存実応答replay |
| MG-D1a desired明示値と元data不変 | Gate.rules | 新規priority / no-mutation | 設定書込みなし |
| MG-D1a 未知field/順序の扱い | Gate.rules | 新規unknown / order negative | 全面正規化しない |
| MG-D1 name/target/enforcement/conditions/bypass、PR/check/app/strict/deletion/force | Gate.rules | 既存rules負例＋互換応答との組合せ | native実績保持 |
| MG-D8 status/Ready/merge同一経路、current main、通信失敗で停止 | Gate.status/mutate/rules | CLI / 既存offline/race | schema/record変更なし |
| MG-D11 旧gateとowner判断、closeout/activation順序 | legacy PR運用 | full / PK5 / hosted | 本番は別承認 |
| D-087 一貫担当と既存の独立性/承認条件 | AOM / DEV_WORKFLOW / D-087 | doc / 既存workflow suite | 限定Plan ReviewとFinal Double Audit |

隣接監査はCIとmain保護/Helperの境界/移行・復旧のsource節を確認した。記録schema、manual再利用、CI分類・draftの実装は非接触で、既存suiteの回帰を維持する。

## Test Plan

[Test Design Matrix](test-matrices/2026-09-14-merge-rules-compatibility.md)。対象は`python3 scripts/tests/pr-gate.test.py`、shared suite、doc/plan、legacy full。製品UI非接触なのでWindows L3は不要。テスト数・所要時間・token削減量は未実測。

## Boundary / Wire Contract

- producer: GitHub ruleset detail / current mainのdesired JSON。
- consumer: `Gate.rules`（status/Ready/mergeの共有検査）。
- wire/internal: JSON object→Python dict/list/bool。JSON trueと数値1を区別する。
- round trip: GET原文を保持し、copy上だけで既知の応答項目を除く。設定APIへの書戻しなし。
- invalid: 異なる値/型・未知field・不足/変更した保護は拒否。offline/不明応答を成功へ補完しない。
- compatibility: 未指定の既定項目は不在または表の値のみ。desiredが明示する値は比較対象。配列順序と他の照合契約は不変更。

## Review Focus

許容範囲が固定の値/型を越えていないか、desiredまで削っていないか、statusだけ直ってmutationへ接続しない修正にならないか。正常API fixtureがdesiredをそのまま返す自己充足を避けたか。Plan/公開/有効化の権限を混同していないか。

## Spec Contract

Contract ID: SPEC-MERGE-EVIDENCE

MG-D1aの既知応答だけを受理し、MG-D1/D8の保護を維持する。新規test名はMatrixで予定と明記し、実装前のcoverageと主張しない。

D-087の一貫担当は作業の割当だけを変え、既存の独立性・計画先行・owner裁定を維持する。

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-MERGE-EVIDENCE / MG-D1a | S1/S2 | Matrix C1〜C5 | 限定受理とdrift拒否 | CLI / replay / mutation |
| SPEC-MERGE-EVIDENCE / MG-D1/D8/D11 | S3/S4 | Matrix C6 | 保護/独立性/移行順 | source / legacy PR evidence |
| D-087 | S5 | Matrix C7 | 一貫担当と独立性/承認の維持 | 限定Plan Review / source / Final Double Audit |

## Data Safety

raw API、vendor metadata、review出力、実行logはignored `.local/merge-rules-compatibility/`または既存probe directory。tracked testは合成値だけ。実データ・secret・個人設定を読まずcommitしない。Plan Gate前のruntime変更と、未承認のGitHub設定変更は行わない。

## Implementation Results

MG-D1aの限定照合を共有`Gate.rules`へ実装した。既定項目を含む正常応答の回帰を現行実装でREDにした後、helperの全テストと保存実応答のreplayがPASS。無変更copyのpositive controlと、既定値・型・未知field・desired優先を壊すmutationで検出力を確認した。初回候補のCLEAN fullと独立Double Auditを完了し、P1/P2なし。下記のP3整理後の最終検証はlocal evidenceと公開後のPR本文へ記録する。正確な候補版・test数・実行logをtrackedに自己記録しない。

## Review Response

- Findings Freeze: frozen after Broad Audit

最初のSonnet high CLI起動はOAuth期限切れ（HTTP 401）で認証に失敗し、review未実施としてpendingにした。ownerの再ログイン後、計画commit `1c89ab9a3f302133066e92e6f0e5f544ff16bdb5`への独立Plan Reviewを完了した。結果は「P1/P2のstopperはなし」、P3は以下の記録更新・文言明確化。要求モデル/effortはSonnet/high、実効primary modelはmetadataの`claude-sonnet-5`で確認し、実効effortは取得できていない。原文はlocal-only `.local/merge-rules-compatibility/plan-review-r1-retry.json`。

- P3-1（owner裁定候補）: 認証待ち表記を完了したreviewの事実へ更新する案を反映した。
- P3-2（owner裁定候補）: AC5の「name/ref置換だけ」とData Safetyの「合成値だけ」の曖昧さを確認し、tracked fixtureには追加項目の値・型だけを抽出する案を反映した。raw応答全体を読む既存`reproduce-rules.py`はlocal-onlyの別経路なので、reviewer修正案の「同方式」という補足は転載せず、両者の境界を明記した。S2とMG-D1aの範囲・許容値は変更していない。

packet/Matrixの既存commitと文書検証を根拠に、AC5の明確化を含むcontent commitで`plan-draft -> plan-gate`を記録する。P3の反映案と計画採用はowner判断待ちで、採用済み・実装許可とは扱わない。P3-onlyの追加reviewは発注していない。Plan Commitは採用対象の確定content commitをowner Plan Gate後に設定する。次は介入9回目 / 予算12回。外部公開・Ready・merge・本番有効化も未承認のまま。

### Ownerの実施指示と担当形の補足

ownerは上の採用判断に対し、Astra自身で進め、レビューだけSonnet / Opusを呼ぶよう指定し、但し書きの保存も依頼した（累計はOwner Effort Budgetを参照）。P3反映案を含む互換性修正をこの担当形で進める。S5 / AC7とD-087はこの明示指示の同期であり、Ready/merge/本番有効化や自己承認の許可ではない。追加のworkflow文言と担当条件を独立Plan Reviewerへ限定確認する。これはP3-onlyの再reviewではなく、新しいowner指示で追加した契約範囲の確認。Phaseはその結果が揃うまでplan-gateを保持する。

### 限定Plan Review完了と実装開始

Sonnet highの限定Plan Reviewは追加したD-087/S5/AC7を直接確認し、P1/P2なし、既存の独立性・Plan Gate・計画先行・Gated Amendment・owner裁定を維持と判定した。実効primary modelは`claude-sonnet-5`で確認、highは要求effort。原文はlocal-only `.local/merge-rules-compatibility/plan-review-d087.json`。

P3として「D-087がExecution Modeの追加に見えないための補足」をowner裁定候補として保持する。reviewerのcodex-only限定案はまだ裁定しておらず、新しいmodeは追加していない。P3だけを理由に作業や追加reviewを増やさない。

元の独立Plan Review、P3明確化、今回の限定確認、ownerの実施指示、および計画/Matrixと追加sourceの確定commitを根拠に`plan-gate -> plan-approved -> implementing`を隣接遷移として記録する。Plan Commitを設定し、現在のAstraが実装を担当する。公開・Ready・merge・本番有効化は後段のowner判断を維持する。

D-087のP3については、現行enumを変更していない事実をAOMへ明記する案を反映した。codex-onlyへ新たに適用範囲を狭める変更は加えていない。ownerへの候補引渡しでこの反映案も示し、独立監査では元の指摘とsourceの整合を確認する。

### 独立Double AuditとP3の整理

Sonnet high / Opus highが同じ実装候補を独立にContract Auditし、両方がP1/P2なしと判定した。実効primary modelはmetadataの`claude-sonnet-5` / `claude-opus-5`で確認。両者はRead/Grepによる静的照合を実施し、Bashはreview環境の`.git/config.lock`書込み制限で起動できなかったため、独立テスト再実行・git差分取得・hash再計算済みとは扱わない。rootによる実検証と両者の静的な実物/evidence照合を区別する。原文はlocal-only `.local/merge-rules-compatibility/final-sonnet.json` / `final-opus.json`。

OpusのP3は、pull_request以外のruleへの同名項目とallowed_merge_methods順序の負例補強、再現可能なreplayと正しいL1 log参照、owner relay文言、Matrix/現在地の同期。指摘に沿った反映案として整理し、実装本体と送信policyは変更していない。初回監査後の差分はnegative fixtureと文書・証跡の整理で、追加Broad Auditは発注しない。整理後の必要検証を行い、反映案をownerの公開・Ready判断へ提示する。

SonnetのP3（内外の局所変数`params`の改名）は、挙動に影響しない後続候補としてownerへ保持案を提示する。採否を自己裁定しない。正確な監査対象とP3後の検証対象の差分・SHAはPR本文へ記録する。

### 公開と承認済みのReady準備

ownerが「締めまで進めていい」と明示指示し、公開・Ready、mergeと機械的closeout、準備済み本番rulesetの有効化/read-backを承認した。介入は公開・Ready、merge/closeout、本番有効化の各判断を分けて予算内に記録し、同じ承認を再要求しない。P3反映案と局所変数名整理の後続保持も、この候補の採用として記録する。承認原文と範囲はlocal-only `.local/merge-rules-compatibility/finish-authorization.json`、公開可能な要約は[PR #59](https://github.com/higekiri-w/inventory-system-desktop/pull/59)本文。

PRをDraftで公開し、候補`17ca1ad7d9a44a49117a1d4ba24c36e300498cf5`のCLEAN full・独立監査・P3整理・owner裁定を本文へ記録した。PR head/base/本文を実物照合済み。Reviewed Content HEADは実監査対象を保持し、P3後もhelper本体と送信policyが不変であることを確認した。

これらの既存evidenceとReady承認を根拠に、`implementing -> local-verified -> independent-review -> human-confirm -> ready-hosted-final`を隣接遷移としてこのstate-only commitで実体化する。Scope/AC/Matrix/実装を変更しない。PRはDraftのまま、このcommitで確定するHEADに対してCLEAN L1を実施し、本文を更新してからReadyにする。成功したhosted headShaとPR head/L1を照合するまでmergeしない。承認済み本番操作も、修正merge・closeout・実効保護の確認順序を維持する。


### 修正mergeと機械的closeout

[PR #59](https://github.com/higekiri-w/inventory-system-desktop/pull/59)をsquash mergeした。merge commitは`4e33153ccc10a42a1ea0adf7dd7f19632da96139`、実時刻はGitHubのmerge記録を参照する。Ready対象のCLEAN L1、PR head、successful hosted headShaの一致と、全必要job/GitHub ActionsのMerge gate成功はPR本文へ記録した。local mainはmerge結果と同じtreeへ同期済み。

このcloseoutはpacket/Matrixの移送・相対link・Plansだけを同期するR0のdocs-only PR。製品、CI、helper、送信policyを変更しない。docs/実PK5とMerge gateの成功を同じHEADで確認してからmergeする。自身の追加closeoutは作らない。

ownerは締めまでの操作を承認済み。本番main rulesetは修正とこのcloseoutの完了後に適用し、有効化/read-backの事実は[PR #59](https://github.com/higekiri-w/inventory-system-desktop/pull/59)の専用記録・ignored evidence・helper statusを正本にする。現在のarchiveは修正計画の完了を表し、未取得の本番適用結果を先取りしない。残る局所変数名整理と旧bootstrapのP3は着手条件付き後続保持を維持する。workflow effectivenessのdogfoodは新mode有効化後の最初のR2+ PR。
