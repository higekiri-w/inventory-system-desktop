# マージ検証と証跡作業の整理

Status: accepted。本番保護の現在の有効化事実はhelper statusを正とする。移行・有効化の手順、測定の判断基準、公式根拠とprobe、有効化payloadの記録は[archive](../archive/2026-09-14-merge-evidence-rollout.md)へ移した。
Contract ID: SPEC-MERGE-EVIDENCE

## 動機と目的

目的は、検証した版を確実にマージしながら、AIとownerが証跡の転記・読み直し・状態記録に費やす手間を減らすこと。モデルの思考と独立レビューは、設計判断と欠陥検出に使う。SHAを短く表示するだけでは、記録更新でcommitが増え、再検証と説明が連鎖する原因を除けない。

[PR #52](https://github.com/kosei-w90607/inventory-system-desktop/pull/52)では、実装・独立reviewを終えた内容に対してhuman-confirmとReadyのstate-only commitを追加し、Ready HEADでRust/frontendを含むfullを再実行した。これは当時の規定に従った処理である。D-033はprivate Freeでrequired checksを利用できなかった事情、D-035は自分のSHAを書き込む自己参照問題への対応だった。現在はpublicで、GitHubによる強制を選べる。2026-09-14にownerは「GitHubで強制し、docs・後処理は軽いPR経路」を選択した。

成功は、通常のReady/mergeでSHAの手転記と実装後の状態記録commitを不要にし、必要な検証結果を短い機械判定で確認できること。Plan Gate、非Writerによる独立review、ownerのReady/merge判断、Windows L3・R4の保護は維持する。CI未検証版をGitHubが通すこと、helper経由の正規mergeで未了review/manual/R4を通すこと、必要jobのskipを成功扱いすること、記録を作ること自体が目的になることを失敗とする。token削減量は未実測で、異なるreview roundを比較して効率改善を主張しない。

## 設計判断

| ID | 決定 | 理由・棄却案 |
|---|---|---|
| MG-D1 | mainへPRを必須化し、GitHub Actions由来の`Merge gate`をrequired checkにする。strict（最新mainとの整合）を使う | CLIを通らない操作でもCI/PR要件をGitHubで強制する。review/manual/R4の強制はhelperの正規経路が担う。CLIのみの自主運用はownerが選ばなかった。別GitHubアカウントのreview承認を追加要求する案は、モデル間reviewとアカウントの独立性を混同するため採らない |
| MG-D2 | `Merge gate`は変更分類と必要jobの実成功を集約する。必要jobのfailure/cancelled/skipped/欠落/未知値は非0 | 単に「CIが緑」、Rust aggregateだけ、PR本文のpass宣言だけを根拠にしない |
| MG-D3 | docs-onlyにも軽いCIを実行する。Draftはマージ根拠となるcheckを発行しない（Superseded in part by: D-145 (9)。Draft の run も `Merge gate` を出し、同じ head の run を merge の根拠に使う。docs-only にも軽い CI を回す部分は残る） | paths-ignoreと`Hosted CI: skip`によるmerge根拠の欠落をなくす。全docsにRust/frontend全量検証を課す案も採らない |
| MG-D4 | 既存classifierとlocalのworkflow回帰suiteを再利用し、hostedとの検証範囲を揃える | local fullの証拠だけを外して検出力を落とさない。新しい分類体系やテスト一覧の複製を避ける |
| MG-D5 | 新運用ではGitのpacketは計画と実装開始の根拠、実装後の状態はPRのnative state・review記録・CIから導く | 完了段階のstate-only commitをなくす。全工程を外部サービスへ移し、オフラインの計画/実装を止める案は採らない |
| MG-D6 | 非CIのreview/manual結果はPR内の専用記録1つへまとめ、対象版を自動取得する | PR本文全体を毎回書き換えない。署名チェーン、独自DB、別bot、SHA除外pathを持つ独自content hashは作らない |
| MG-D7 | 通常の最終merge根拠はhosted gateに集約し、localは変更中のfeedbackとhosted外の検証に使う | local fullを削除せず、通常mergeでの一律のlocal/full/SHA三点照合を撤去する。manual/R4結果はCI成功で代替しない |
| MG-D8 | helperはstatus・review対象capture・記録・Ready・mergeだけを扱う。既存`gh`とPython標準ライブラリを使う | AIが多数のAPI出力を読んで組み立てる工程を減らす。reviewerの起動、採否の自動判断、Plan Gateの自動承認は担当しない |
| MG-D9 | 直接mainへpushしていたdocs/closeoutも、docs CIを通る小さなPRへ変更する（Superseded in part by: D-145 (5)。merge の後の closeout と closeout の PR をなくした。main へ直接 push しない部分は残る） | main保護を恒常bypassで骨抜きにしない。R0 closeoutにPlan Packetや新しい独立reviewを要求せず、closeoutの再帰を起こさない |
| MG-D10 | 移行期の旧packetの扱い（完了。履歴は[archive](../archive/2026-09-14-merge-evidence-rollout.md)） | 現行の方式は本書だけ。archiveは一括変換しない |
| MG-D11 | CIとhelperを旧gate下で完成・dogfoodしてからmain保護を有効化する（完了。履歴は[archive](../archive/2026-09-14-merge-evidence-rollout.md)） | 保護だけ先に入れて正規のmerge/closeoutを閉塞させない。失敗時の自動bypassは禁止 |
| MG-D12 | 人とモデルへの出力は状態・次の行動・阻害理由を中心にし、実行ログと集計を分ける | 成功時のログ全文再読や証跡の再証明を常設しない。レビュー要約だけで検証を済ませる案も採らない |
| MG-D14 | 人が書くのは判断だけにし、状態は helper が導く（[D-145](../decision-log.md#d-145-開発の手続きを守ることの芯から組み直し人が書くのは判断だけにする2026-10-28)）。計画の固定は `scripts/plan-approve.sh`、Minimum・Human Gate は Risk・classifier・`## L3` 節から、active な packet は起点の tree との比較から導く。lane の docs だけの commit（C）は L3 と review の closure を helper の記録で引き継ぐ。R0〜R2 は packet なしの道 | 写した状態は写した瞬間から正本より古くなりうる（TD-492）。状態を手で書き直す commit とその review を増やさない。発注書に監査の SHA を写して照合する案、R2 の packet を最小形で残す案（TD-498 の B）は採らない |
| MG-D13 | base同期だけでheadが変わった版は、同期だけの条件（lane の差分の同一性）をhelperが機械で確かめる。取り込んだmain側の変更がcodeに触れなければ、manualを自動で引き継ぎ、reviewのclosureをhelperの記録で代える。codeに触れるときは、同期のclosure 2本とownerの承認でmanualを使い回す（[D-144](../decision-log.md#d-144-同期で-head-が変わった後の-l3-と-closure-を同期だけの条件で使い回す2026-10-27)） | PR差分のbyte同一は、両方が触るfileのindex行と周りの行が同期でずれ、共有fileを触るPRどうしで構造的に満たせなかった（PR #197・#209）。正規化した差分の比較（patch-id相当）、同期の前後の差分のbyte比較は採らない |

## 強制範囲（owner選択）

2026-09-14にownerは「CIはGitHub、review等はhelperで確認。直接UI mergeは運用上禁止」を選択した。GitHubが強制するのはPRとCIの条件であり、review/manual/R4のrecordはCI evaluatorへ入れない。これらはhelperとownerの規律で確認する。新modeの正規Ready/mergeはhelper経由とし、直接UI mergeや確認を飛ばす直接gh mergeを行わない。

GitHub UIがCI成功だけでmerge可能と表示する場合があることは既知の残存リスクで、未実装のserver保護があるとは主張しない。helperはその状態でも古い/不足したreview/manual/R4を拒否する。CI以外もserver側に追加の必須checkで強制する案は、同期処理と再実行を増やすため今回は採らない。既存のモデルreview・owner承認を省略する許可ではない。

## CIとmain保護

現在のRust集約jobを全体の`merge_gate`へ置き換える。既存のRust lint/test/drift、frontend、docs、env jobを保持し、workflow回帰suiteのjobを追加する。aggregateは全jobを`needs`に持ち、aggregate の if は `always()` だけで、名は常に `Merge gate`（D-145 (9)）。`needs.changes.result == success`をjobのifへ入れてはならず、分類失敗は実際に起動したevaluatorが非0で処理する。既存ci-workflow.test.shの全always jobへのsuccess前提をaggregateについて置換し、前提を付け足すmutantを拒否する。分類成功が必須で、選択されたjobは`success`だけを受理し、非対象jobの`skipped`だけを許す。未知pathはfull fallbackとして受理するが、分類key/値の不備や未知のjob結果は拒否する。npm auditのwarn-onlyという現行契約はこの変更で改変しない。

`pull_request`はmain対象のopened/reopened/synchronize（Ready にしたときは回し直さない、D-145 (9)）、dispatchは現行どおりfull。paths-ignoreと本文のskip tokenで全workflowを止める経路は廃止する。body編集では実行しない。base付替え等でrunが無い場合は、同じ HEAD の run（成功か進行中）が無いことを確かめてから明示recovery dispatchを案内する。予防的な重複dispatchは行わない。

Draft でも changes と分類された job と aggregate が走り、aggregate は `Merge gate` を出す（D-145 (9)。同じ head の CI を 2 回回さない）。helper の `ci` は同じ head の最新の run が `pull_request`（Draft を含む）か `workflow_dispatch` で成功し、その check suite に GitHub Actions（15368）の `Merge gate` が成功で 1 つだけあるときに受け、別の head・失敗・欠け・別の app を拒む。Draft の run は Draft の間に選んだ job を回すので、Ready で job の選び方は変わらない（`ci.yml` で `draft` を読むのは消した name 式だけだった）。**required名のskipped checkを発行してはならない**（GitHubはskipped/neutralもrequired checkの成功条件に含める）ので、aggregate の if は `always()` のまま、必要 job の skipped を evaluator が拒む。

docs-onlyを含むhostedの全正規経路には実PR headに対するPK5（Plan Commit/Amendmentsの祖先・不変確認）も必須接続する。該当jobはfetch-depth: 0とPR head/baseを取得し、履歴不足をskip成功にしない。parityの対象はlocal-ci.sh fullの全gate（workflow-git、shell-syntax、workflow-yamlも含む）。回帰suiteの実行と、実PRのPK5検査を別の義務として扱う。必要なRuby/ripgrep/PythonとNode pinをhostedに用意し、CI tokenはcontents:readに固定してownerの運用commentを書き換える権限を渡さない。

分類は既存の出力keyを保つ。CI/ゲート実行コード、tests、未知pathは全gate。AGENTS/CLAUDE、agent-guidance、workflow/CI/review正本、workflow templates、Skills、Claude rules/commandsはdocs＋workflow回帰を要求する。一般のPlans/backlog/archive/説明文書はdocs経路。function-design/REQ/bindings等の既存traceability/drift分類は維持する。consumerは明示されたarea flagを使い、`workflow=true`だけで無条件にRust/frontendへ再昇格させない。rename/copyの両pathと削除も分類する。

`workflow`出力は「workflow回帰が必要」の意味へ限定し、旧の自動full昇格を次の表へ置換する。consumerとfixtureを同時に更新する。

| 分類 | 具体pathと扱い |
|---|---|
| 実行制御/full | `.github/workflows/**`、`.github/actions/**`、`.github/merge-gate-ruleset.json`、`scripts/ci/**`、`scripts/tests/**`、`scripts/{local-ci,pre-push,doc-consistency-check,check-env-safety,check-workflow-git,check-command-drift}.sh`、`scripts/pr-gate.py`、`.codex/bin/**`、`.claude/settings.json`、`.claude/hooks/**`、`.claude/agents/**`、`**/.claude-plugin/**`（skill folderをpluginにするmanifest。中身のpathはfullにせず、manifestが無いことをhook testが求める） |
| policy/docs＋workflow回帰 | `AGENTS.md`、`CLAUDE.md`、`docs/{DEV_WORKFLOW,ci,AGENT_OPERATING_MANUAL,code_review,project-profile}.md`、`docs/quality/review-checklist.md`、`docs/agent-guidance/**`、`docs/templates/**`、`.agents/**`、`.claude/{rules,commands,skills}/**`、`.github/pull_request_template.md` |
| 一般docs | `docs/Plans.md`、`docs/backlog.md`、`docs/archive/**`等。既存のfunction-design/REQ/bindings/drift分類が重なる場合はそのflagも保持 |
| その他 | 既存のRust/frontend/env/generated/traceability分類を維持。未登録pathは全gateへfallback |

helperのRisk/review要件確認ではcurrent main側の分類を使い、PRがclassifier/evaluatorを変更して自分の必要reviewを減らせないようにする。helperは自分のfileがPRのbaseの`scripts/pr-gate.py`と同じときだけ動き、違えば`helper differs from base …`でbaseの版を使うcommandを示して止まる（D-099。この照合を載せた版以降のhelperに効く）。示すcommandは、baseの版を取り出して元の引数のまま実行する完全なcommandで、末尾を省かない（D-107）。CI判定自体を二重実装せず、helperは正しいworkflow/app/対象版の実行済みMerge gateを確認する。

ruleset案は`inventory-main-merge`、target=branch、include=`refs/heads/main`、exclude=[]、enforcement=active、bypass_actors=[]。rulesはpull_request（required_approving_review_count=0）、required_status_checks（context=`Merge gate`、integration_id=15368、strict_required_status_checks_policy=true）、deletion禁止、non_fast_forward禁止。15368は現repoのCI check APIで確認したGitHub Actions app IDで、設定直前にも照合する。review承認数0はモデルによるPlan/Final Reviewを免除せず、別GitHubユーザーを増やさないための設定。既存rulesetを上書きせず、別設定が増えていれば適用差分を再確認する。

desired payloadは実装時に`.github/merge-gate-ruleset.json`へ置く。helperの実効照合はcurrent main側のpolicyを使用し、PRが自分用に弱めたpolicyを信頼しない。payloadの自動適用はhelperに含めない。保護はCI改変の悪意やモデルreviewの独立性を証明するものではなく、workflow/policy変更の独立Double Auditを維持する。

native拒否のprobeは、ownerに具体的対象を示したうえで`ci-probe/merge-gate-<candidate>`という検証用base refと、rules内容が同じで対象refだけ異なる一時rulesetを使う。PRを伴わない合成commit、必要checkのないPR、期待appでないstatusを拒否することを確認する。positive controlはbootstrapの実CIが成功したcandidateとその検証時baseを使い、検証用PRだけをmergeする。mainへ失敗実験を行わず、結果を保存して一時ref/rulesetを片付ける。最後にproduction rulesをread-backする。GitHubのPR要件は「PRに対応する検証済み変更」を要求するもので、正当なPRに対応したmanual mergeまで暗号学的に禁止する仕組みとは扱わない。

### 実応答の既定項目との互換性（MG-D1a）

GitHubのruleset detailは、送信payloadにない既定parameterを返す。[検証用PR #57](https://github.com/kosei-w90607/inventory-system-desktop/pull/57) / [#58](https://github.com/kosei-w90607/inventory-system-desktop/pull/58)の試験ではnativeの拒否・正常mergeが成立した一方、`pull_request.parameters`に追加された次の項目でhelperの全体比較が不一致になった。

| 応答だけに存在する項目 | 許容する値・型 |
|---|---|
| `required_reviewers` | 空のJSON配列 `[]` |
| `require_extra_approval_for_unattributed_changes` | JSON boolean `true`（数値 `1` は不可） |

helperは取得したcurrent main側のdesired policyを基準とする。上の項目をdesired側が明示していない場合に限り、応答に存在すれば値・型を表どおり検査し、比較用のコピーから除いて既存の全体比較を行う。応答に存在しない従来形式も受け入れる。desired側が明示する項目は除外せず、desiredとの比較対象として残す。元のAPI応答・desired・保存証拠は変更しない。

未知のparameter、表と異なる値・型、既存の保護項目の欠落・変更は拒否する。rulesの配列順序やallowed_merge_methodsの順序はこの修正で正規化せず、従来の比較を維持する。name/target/enforcement/conditions/bypass、実効PR必須・Merge gate/15368/strict・削除/force禁止の検査を維持し、status/Ready/mergeは同じ`Gate.rules`を通る。

これは観測した応答形を扱う限定的な互換処理で、GitHub側の設定やreview要求を変更するものではない。`required_reviewers`は[公式Rules API](https://docs.github.com/en/rest/repos/rules)に記載がある。もう一方の項目の入力契約・全状況での意味は未確認で、今回のnative試験を越えて無害と一般化しない。送信payloadへの未確認項目の追加、未知fieldの一括無視、再帰的な部分一致、恒常bypassは採らない。実応答が許容形から変わった場合は利用を止め、実物確認と別の変更判断を行う（D-086）。

## 状態と非CI記録

Gitで管理するWorkflow Stateは、新形式の packet で Risk、Plan Commit、Amendments の 3 行（任意で Final Review Minimum・Human Gate）。旧形式の packet（`Phase` の行があるもの）は今までの行を受け、`Phase` の値と担当 4 行・`Branch` を評価しない（D-145 (15)）。形式は packet を最初に足した commit（起点の後、`--no-renames`）の親の tree に `scripts/plan-approve.sh` が在るかで決め、形式と `Phase` の行の有無が食い違う packet と新形式の R2 を拒む（親が無い・shallow の境は判定不能として止まる。関数 `packet_form`）。`Evidence Mode`行は任意で、書くなら`github`だけを受理する。その他の行はhelperが評価しない。archiveは非遡及。実装後の状態をGitへ書く必要はない。計画の前後は `Plan Commit` が `pending` か SHA かだけで決める（`pending` なら status は `Plan Gate incomplete`、capture は拒む）。

要件の導出（`requirements`、D-145 (4)）: 新形式は Final Review Minimum を R3・R4・classifier `workflow=true` で 2、packet に書いた値があれば大きい方（下げる向きは受けない）。Human Gate は `ready,merge` を常に、`r4` は Risk R4、`manual` は `## L3` 節があるとき、書いた `r4`・`manual` は足す。旧形式は書いた Final Review Minimum と Human Gate をそのまま使い、R4・workflow gate で 2 を下回らない。自由文の条件を推測して免除しない。

helperは承認の条件を、Plan Commit（Amendmentsがあれば最後の登録SHA）のpacket snapshotとheadで照合する（旧形式は Risk / Final Review Minimum / Human Gate、新形式は Risk・`## L3` 節の有無・書いた Final Review Minimum と Human Gate。関数 `gate_conditions`）。未追補の変更は拒否し、snapshot当時の自己SHAがpendingでもこの照合には影響させない。あわせて、計画の節（Scope・Acceptance Criteria・Contract Ledger〈旧形式は Spec Contract と Trace Matrix〉、R4 は Data Safety と見出しに Recovery か Rollback を含む節）が承認の最新の snapshot と byte で同じことを求め、違えば `approved plan changed` で止めて `scripts/plan-approve.sh <packet> --amend <REVIEWED_HEAD>` を案内する（関数 `plan_drift`。`check-workflow-git.sh` も `approval_state` で同じ判定をする）。旧形式で Phase が計画の承認の前の値（kickoff〜plan-gate）の packet は、SHA があっても承認の前として扱う。これは登録された承認内容への結合であり、人の承認を署名やmetadataだけで証明する仕組みではない。

Reviewed Content HEAD / Final Exact-HEAD Evidence / Hosted CI Requirementの行は拒否する。元のPlan CommitとAmendmentsの不変性・祖先確認は残す。Amendmentsは過去の登録列が現在列のprefixになる追記だけを許す。SHAの間の区切りや空白は変更できるが、登録順序・SHAの差替え・削除・表記の置換は許さない。Plan CommitとAmendmentsの各SHAは40桁の小文字hexだけを受理し、Amendmentsの重複も拒む。PK5が受けるSHAの間の区切りはカンマとASCIIの空白・タブだけで、helperより厳しい。helperが拒むWorkflow Stateの値（末尾の空白・注記、短縮SHAを含む）は、active な R3/R4 の packet ではPK4がhelperの判定そのもので、PK5がSHAの桁数で、pushの前に拒む（D-107）。実装開始後に計画の契約が変わる場合の再設計・再reviewも維持する。

非CI結果はowner名義の専用PR comment（marker=`inventory-workflow-v1`）に保存する。PR本文や他commentを編集しない。正当なauthorの記録が複数あれば曖昧として止める。CIのSHAや成功フラグは複製せずGitHub APIから取得する。

wireの正本は次のRecordV1だけとする。SHAはこのrepoのfull SHA、Evidenceは公開可能な証拠pointer（会話全文・vendor session ID・実データは含めない）。Riskの意味やreviewの独立性は人とモデルが確認し、metadataだけで証明しない。

```text
RecordV1 = {
  version: 1, repo: owner/name, pr: positive_integer, head: SHA, base: SHA,
  review: {outcome: Outcome, broad: Broad|null, closure: Closure|null},
  manual: {outcome: Outcome, evidence: [Evidence], source_head?: SHA, reuse_approval?: Evidence},
  r4: {outcome: Outcome, evidence: [Evidence]}
}
Outcome = pending | pass | fail | not-required
Audit = {model: string, run_ref: string, evidence: [Evidence]}
Broad = {head: SHA, base: SHA, plan_commit: SHA|null, amendments: [SHA], audits: [Audit]}
Closure = {head: SHA, base: SHA, audit: Audit} | {head: SHA, base: SHA, sync_from: SHA}
```

`sync_from`の形のclosureは、helperが同期だけの条件とcodeに触れない条件で記録したもの（後述の「base同期だけでheadが変わる場合」）で、独立監査を表さない。

初回のbroad監査はpacketのFinal Review Minimum以上の独立auditを必要とする。auditsは監査の実施記録で、各監査でfindingが出たことを隠さない。現在版の全findingが裁定・解消された場合だけreview.outcomeをpassにする。run_refは重複不可の公開可能な識別子であり、モデルのsession識別子ではない。packet の review を not-required にはできず、Broad.plan_commitは承認されたPlan Commitと一致するfull SHAを必須としnullを拒否する。schemaのplan_commit=nullは packet なしの道（R0〜R2）だけに許す。R0/R1の不要reviewはbroad/closureともnull、R2 は差分の review 1 本の broad を持つ。

broadと現在のhead/baseが同じならclosureはnull。異なる場合は、同じPlan Commit/Amendmentsの範囲で、必要数を満たすbroadを保持し、現在head/baseに一致する独立closureを1回以上必要とする。closureのauditは過去の監査・既存finding・累積修正を踏まえた現在候補の完了判定を含む。単に最新の1行だけ見た結果を完了判定にしない。Plan契約が変わった場合やbroadの同一scope適用を確認できない場合は、新しいbroadを必要とする。base同期にもこのclosure数を適用し、初回Double Auditを無条件にやり直さない。同期だけでcodeに触れない版は、独立closureの代わりにhelperが`sync_from`の形のclosureを記録する（MG-D13）。

manual/r4のpassは非空evidenceが必要。manual.source_headは後述するbase同期の使い回しでだけ受理し、reuse_approvalはsource_headと組でだけ置く（自動の引継ぎはsource_headだけ、ownerの承認の使い回しは両方）。それ以外のmanual結果はrecord.headでの実施/確認を表す。r4に再利用fieldはなく、当該head/操作条件の明示承認を必要とする。manualはL3等の追加検証・承認をまとめた結果で、内容はpacketの該当手順が所有する。

captureはrepo/PR/head/base、Plan Commit/Amendmentsと現在の正当なrecordを取得し、cleanなlocal HEADとの一致を確認して一意のlocalファイルへ保存する。captureは上書きしない。record時にPRと既存commentを再取得し、対象版または取り込む元recordが変わっていれば停止する。新しい版にR4のpassを継承しない。manualのpassを引き継ぐのは後述の同期だけの条件とcodeに触れない条件を満たす版だけ。旧broadを使うclosureは、serverに現在も存在する正当なrecordを基に作り、消失したcommentをcacheだけから復元しない。

書込みはsingle-writerが担当し、reviewerはcommentを編集しない。IDを指定して専用commentだけを更新し、前後のhead/baseを確認する。途中で版が変わった記録は有効扱いせず、任意の最新commentで補完しない。commentへの書込みは当該PRの記録更新の明示承認範囲で行う。

| 観測状態 | 有効な現在地 / 次の行動 |
|---|---|
| 計画未承認（Plan Commit が pending） | Plan Review の後に `scripts/plan-approve.sh`。実装不可（先走りの検査が止める） |
| 実装開始済み、PRなし / local先行 | 対象検証とDraft準備 |
| Draft、必要review未了/古い版 | 修正または独立review。Ready不可 |
| Draft、review通過、manual/R4が未了 | 該当するowner検証/承認待ち |
| Draft、必要review/manual/R4充足 | ownerのReady判断待ち |
| CI未了/失敗 | 同じ head の run 待ち。成功前はmerge不可、必要な修正はDraftへ |
| Ready、現版の必要CI成功 | ownerのmerge指示があればhelperでmerge |
| merged | 完了（closeout の PR は作らない。D-145 (5)） |
| offline / API不明 / 壊れた記録 | merge権限は確定しない。計画/実装の許可済み作業は継続可 |

## base同期だけでheadが変わる場合

strictによりmainの取込みが必要な場合、merge 済みの packet は active でない（起点の tree に在る）ので helper と PK5 の判定に入らない。対象を Draft へ戻してから `git -c merge.directoryRenames=false merge origin/main` で取り込む。GitHubのUpdate branchも同じ扱いで、hookを通らない更新を信頼しない。新headではCIを実行し、独立reviewは新たなdelta/相互作用のclosureに限定する。下の同期だけの条件とcodeに触れない条件を満たす版では、そのclosureもhelperの記録で代える。全面監査を最初から繰り返す義務はない。並行PRではmain更新に伴う再Ready判断とCIが追加され得る。これはstrictを選ぶ運用コストとして各作業の予算に含め、今回のrollout予算だけで将来分も賄えるとはしない。

**同期だけの条件**（lane の差分の同一性、MG-D13）: 起点の版からnew headまでのfirst parentの列が、次の2種のcommitだけでできていることをhelperがlocalのgitで確かめる。(a) 2親のmergeで、second parentがPRの現在のbaseの祖先、2親のmerge-baseが一意、treeが`git -c merge.directoryRenames=false merge-tree --write-tree FIRST SECOND`の結果（自動の3-way merge）と同じもの。違ってよいのは生成物（`src/lib/bindings.ts`と`docs/function-design/90-traceability.md`の2 pathだけ。hostedの`rust_drift`のjobのdrift検査が当たるpathで、helperが定数で持つ。classifierの`generated`のcaseから解析して得ない）だけで、自動のmergeが衝突してよいのも生成物だけ（生成物は手で解かず再生成する。D-098）。(b) 親との差分のpathが生成物だけの1親のcommit（取り込んだ後の再生成）。first parentの列の中で最後のmergeのsecond parentが現在のbaseで、その後ろ（headまで）は(b)だけ。(a)の違うpathと(b)の差分のpathは`git diff --name-only --no-renames`で列挙し、renameは両側のpathで見る（通常のpathから生成物へのrenameは通常のpathの削除として満たさない）。列に他のcommit（laneの編集）、生成物の外の衝突、自動のmergeと違う生成物の外のpath、mainの外をsecond parentに持つmerge、merge-baseが一意でないmergeがあれば満たさない。`merge-tree --write-tree`（0=衝突なし、1=衝突あり）と`merge-base --is-ancestor`（0=祖先、1=祖先でない）の終了値0/1は判定の結果として扱い、それ以外の終了値は判定不能（`command failed`）とする。判定不能も満たさない扱いで、helperは次の一手に`git fetch origin`（localにbaseや旧headのobjectが無いとき）を添える。`docs/decision-log.md`の衝突と自動のmergeとの違いは、mergeの結果のそのblobが`git merge-file -p --union`（FIRST・2親のmerge-base・SECONDのそのblob）の出力とbyteで一致するときだけ生成物と同じく受ける（並走するlaneの末尾へのDの追記どうしの衝突。手で解いた結果は外れる）。同期で衝突したら`git merge-file -p --union --object-id HEAD:docs/decision-log.md $(git merge-base HEAD MERGE_HEAD):docs/decision-log.md MERGE_HEAD:docs/decision-log.md > docs/decision-log.md`で解く。`.gitattributes`の`merge=union`は同期の`git merge`がlaneの作業treeの属性を読み、属性を足した後の最初の同期で効かないので使わない（D-144）。生成物の中身はhostedのdrift検査（bindings・traceability）がsourceとの一致を確かめる。patch-idや正規化した差分の同値は根拠にしない。

**codeに触れない条件**: 同期だけの条件で辿った列の各mergeについて、2親のmerge-baseからsecond parentまでのmain側の変更のpath（renameは両側、`--no-renames`）を集め、その和集合をcurrent main側のclassifierへ渡し、`rust`・`frontend`・`env`・`unknown`が全部false。起点と最後のbaseの差分だけを見ない（列の途中でmainがcodeを変えて後で戻すと、最後の差分に残らない）。docs・packet・Matrix・`docs/function-design/90-traceability.md`はcodeでなく、`src/**`（`src/lib/bindings.ts`を含む）・`src-tauri/**`・依存とbuildの設定・CIとgateの実行code・未知のpathはcode。

**manualの使い回し**: 起点は旧recordのhead（そのrecordのmanual.passが有効だった版）。(A 自動) 同期だけの条件とcodeに触れない条件を満たせば、helperは新しいheadでの最初のrecordで、manualを元の`source_head`（元のL3の版。旧manualが使い回しなら同じ値）と元のevidenceのままpassとして引き継ぐ。旧manualの`reuse_approval`はそのまま残す。(B 承認) codeに触れる同期は、同期だけの条件を満たし、同期だけを見るclosureの2本（[座組表](../AGENT_OPERATING_MANUAL.md#座組)のFinal ReviewerのClaude側とCodex）がL3の項目に影響なしと判定し（影響なしは、同期で取り込んだ変更がL3の各項目の合否と、確かめた操作・業務の結果を変えないこと。変わるのが画面の表層〈表示の値・文言・配置〉だけなら、項目が見る値〈合格基準が名指す値を含む〉でも影響に数えない。項目の操作が通る処理〈裏のロジック〉を変え、合否や業務の結果を変え得る変更は影響ありとする。ownerの裁定TD-516、2026-10-28。D-146）、ownerが適用を1行で承認したときだけ、`record --kind manual --outcome pass --evidence <旧manualのevidence> --reuse-from SOURCE --reuse-approval POINTER`で使い回す（SOURCEは旧manualの`source_head`、無ければ旧recordのhead）。`reuse_approval`はownerの承認のpointer、closureの2本の証跡はreviewのclosureのevidenceに並べる（後述の実行手順の2本のclosureの記録）。reviewの要らないR0/R1のmanualのlaneはreviewのclosureを持たないので、closureの2本の証跡を`reuse_approval`のpointerの先（ownerの承認と同じ場所）に置く。Bのmanualは新しいheadでの最初のrecordにする。R0/R1のmanualのlaneで`--reuse-from`だけを指定する明示のAは、旧manualの`reuse_approval`をそのまま引き継ぐ（自動のAと同じ形）。どちらも新headで実施し直したとは記録しない。使い回せないとき、helperは新しいheadでのmanual以外のrecordを何も書かずに止め、外れた理由（laneのcommit・自動のmergeと違うpath・衝突のpath・codeに触れるmain側のpath）と次の一手を名指す。次の一手は理由で分ける: 同期だけの条件で外れたら`record --kind manual --outcome pending --evidence POINTER`（旧passを外してL3をやり直す）だけ（Bもこの条件を要るので案内しない）、同期だけの条件を満たしcodeに触れない条件だけで外れたらBの上の完全なcommandと要るもの、または`--outcome pending`のcommand。codeに触れない条件は同期だけの条件を満たしたときだけ評価する。`--kind manual`で`--reuse-from`の無いrecord（L3をやり直す`--outcome pending`や新しいL3の記録）ではこの判定を省き、L3をやり直す出口を塞がない。headが旧recordのheadと同じでbaseだけが変わったrecordも同じに止め（同期だけの条件はheadが変わることを前提にするので満たさない）、理由は「baseが進んだ。先に同期する」とする。機械条件はmanual対象への無影響を証明しない（Aはownerの決定でcodeに触れない同期に限って受け入れ、Bはclosureとownerが判断する）。R4承認は新head/操作条件に対する明示承認を必要とし、このmanualの使い回しでは代用しない。

**reviewのclosure**: 起点は旧recordの最後の監査の版（監査のclosureならそのhead、`sync_from`の形のclosureならその`sync_from`、closureが無ければbroadのhead）。旧recordのreviewがpassで、起点から同期だけの条件とcodeに触れない条件を満たせば、`record --kind review --review-stage sync --outcome pass`でhelperがclosureを`{head, base, sync_from}`として記録し、独立reviewのclosureを要らない。`--review-stage sync`は新しいheadでの最初のrecordでだけ使える（先に他のkindを記録するとreviewがpendingになり、版が変わったrecordでなくなるので、監査のclosureが要る）。helperはこれで止めるとき、理由の文にこの順序を書く。今のheadに本文のあるPR reviewが投稿済みなら（`--pr-reviews`の検査と同じく全pageを数える）、helperは各reviewのidと投稿時刻と通常のclosureの記録手順を示して書込みの前に止まる（D-107の取りこぼしの防止をsyncの経路でも保つ）。codeに触れる同期は監査のclosureを要る（Bのclosureの2本はそのclosureを兼ねる）。

## lane の docs だけの commit（C）と GA の後の closure（D-145 (6)(7)）

**L3 の使い回し C**: 上の同期だけの条件の first parent の列に、1 親の lane の commit で、その `--no-renames` の path を current main 側の classifier が docs だけと見るもの（`rust`・`rust_drift`・`frontend`・`env`・`generated`・`traceability`・`workflow`・`unknown` が全部 false）を許す（`Gate.sync_merges` の `docs`）。merge が無いときは起点の版が今の base を含む（`merge-base --is-ancestor base 起点`）ことを求める。そのような commit が列にあれば、manual の版（`source_head`）と今の head で packet の L3 の比較対象が byte で同じことを求める（関数 `l3_basis`）: `## L3` 節があればその節、旧形式で無ければ `## Test Plan` 節の全体と、そこから link する Test Design Matrix の file の全体。どちらも無い・link が 1 本でない・file が無いときは比べられないとして使い回さない。`--packet` が無い PR（R0〜R2）と R4 は C を使わない。A・B と組み合わせて manual を引き継ぐ。

**docs の closure**: `record --kind review --review-stage docs --outcome pass --evidence <P3 だけと裁定した pointer>`。C と同じ条件に加えて、旧 record の review が pass、broad の Plan 契約が今の packet と同じ（下の GA を含む）、計画の節（`## Scope`・`## Acceptance Criteria`・`## Contract Ledger`〈旧形式で無ければ `## Spec Contract` と `## Trace Matrix`〉、R4 は `## Recovery`・`## Data Safety`〈旧形式は見出しに `Recovery` か `Rollback` を含む節〉。節の本文の byte、末尾の空行を除く。関数 `plan_sections`）が起点と今で同じ、L3 の比較対象が同じ（比較元と比較先がどちらも manual なしならこの比較だけを外し、manual の要否が版の間で変われば受けない）とき、helper が `{head, base, sync_from}` の closure を書く（`--review-stage sync` と同じ形。RecordV1 は変えない）。`workflow=true` の docs・計画の節の変更・R4・`--evidence` の無い入力は受けない（gate の文書の P3 の是正も人の closure へ）。`--evidence` の pointer は入力として求め、record には載せない（wire の形を変えない）。P3 だけと裁定した pointer の実体は PR の comment に置く（record からは辿れないので、裁定は PR の comment が正本）。status・Ready・merge は `sync_from` から今の head まで同じ条件を確かめ直す。

**GA の後の closure**: broad の版と今の版で、Plan Commit が同じで、今の Amendments が broad の Amendments を prefix に持ち、計画の節（上と同じ集合）が byte で同じなら、`FRESH_BROAD` の代わりに closure を求める（関数 `plan_same`）。`## Contract Ledger` だけが変わっても、節が読めなくても `FRESH_BROAD`。

使い回しの元は現在serverに存在する正当なrecordのpassに限り、source_headはその確認対象版、evidenceは元記録と一致しなければ拒否する。元recordの欠落・pending/fail・evidenceの差替えや、capture後の元record変更をcacheやCLIのpass宣言で補完しない。status・Ready・mergeは、manualの`source_head`とclosureの`sync_from`から現在のheadまで同期だけの条件を確かめ直し、`reuse_approval`の無いmanualと`sync_from`の形のclosureはcodeに触れない条件も確かめ直す。

manual失敗の修正や対象挙動を変える修正は、既存L3の復旧・改版・canonical先頭からの再実施に従う。L3の証拠の扱いはこの規則に一本化する。

## Helperの境界

active な packet は PR の差分で `added`・`renamed`・`copied` の dated packet（base の tree に無いもの、D-145 (5)）。`modified` の dated packet は merge 済みの packet で、R0/R1 の道で直してよい。R3/R4 は active な packet がちょうど`--packet`の1つで、差分が他の packet を `docs/plans/` から外さないことを確かめる。packet なしの道（R0〜R2）は差分が active な packet を足さないことを確かめ、`docs/plans/` から出る packet は同じPRの差分の中で同名の`docs/archive/plans/`へ移すもの（`renamed`、または`removed`と`added`/`modified`の組。旧い closeout の形）だけを受け、archive の中身は読まない（AR1）。削除だけ・archive の外への移動は拒む。触るpacketは差分の`filename`と`previous_filename`で決め、headの`docs/plans/`の一覧とPlans.mdは読まない。

経路は`python3 scripts/pr-gate.py status|capture|record|ready|merge --pr NUMBER`。R3/R4では`--packet docs/plans/FILE.md`を指定し、当該PRの active な packet と照合する。対象を複数packetへ曖昧に結び付ける入力は拒否する。packet なしの道（D-145 (8)）は`--risk R0|R1|R2`と`--manual required|not-required`を必須とし、PR body の `Risk: R<0|1|2>（目視: 要|不要）` の行（1 行だけ、HTML comment の外）と照合して食い違い・欠け・重複・書式違いを拒む。R2 は Minimum 1（差分の review 1 本）・`plan_commit` null。どの Risk でも migration（`src-tauri/src/db/migration.rs`・`migration_tx.rs`・`schema_v*.rs`）・依存（`package.json`・`package-lock.json`・`.npmrc`）の path、classifier の `unknown=true`、CI制御の実行code（`workflow=true` かつ `rust=true`）を拒み、R2 は `workflow=true` も拒む。理由には path と「R3 の packet で起票する」を出す。Risk値でCIの実行範囲を縮めず、policy文書の意味変更のRisk判定はowner/modelが行う。packet不在をmanual免除とみなさない。業務的Riskの分類自体はowner/モデルが担う。PR作成前はtrackedの計画phaseを使い、helperで架空のPR状態を作らない。

statusはread-only（refと作業treeを変えない。同期だけの条件の確かめ直しの`merge-tree --write-tree`はobjectだけを書く）、結果は短い状態と阻害理由（`--json`で構造化）。recordのbroadのPlan Commit/Amendmentsがpacketと違えば、head/baseの古さより先に`broad Plan contract changed; fresh broad required`を阻害理由に出す（closureでは足りないことを先に言う）。recordにbroadが無い（契約の違いで落としたものを含む）ときも、statusはreviewのoutcomeより先に`broad required`を出し、closureのrecordで使えるbroadがserverに無いときも同じ文で拒む。どちらも要るreview（現在のheadでのbroad）を言う（D-107）。captureはignored `.local/pr-gate/`だけへ書く。recordは`--capture FILE --kind review|manual|r4 --outcome VALUE --evidence POINTER`を受ける。reviewでは`--review-stage broad|closure --pass-model MODEL --run-ref REF --reviewed-head SHA --pr-reviews N`を指定し（`--reviewed-head`はreviewで必須、captureのheadと一致しなければ何も書かず止まる。`--report FILE` は reviewer の報告の `REVIEWED_HEAD=<40hex>` の 1 行を読んで `--reviewed-head` に使い、行が無い・2 行ある・形が違う・`--reviewed-head` と両方は入力の誤り〈exit 2〉、D-145 (10)）、Auditを追加/更新する。`--pr-reviews`もreviewで必須で、reviewed headに付いた本文のあるPR reviewの数を申告する。helperがGitHubのPR review（全page）から数えた数と違えば、各reviewのidと投稿時刻（未提出のreviewは`not-submitted`）を示して何も書かずに止まる。投稿者とreviewの状態は見ず、本文が空のreview（inlineのcommentだけのもの）は数えない（D-107）。同じrun_refを別監査として数えない。manualのBの使い回しには`--outcome pass --evidence <旧manualのevidence> --reuse-from SHA --reuse-approval POINTER`を指定し、Aの引継ぎにCLIの指定は要らない（reviewの要らないR0/R1のmanualのlaneは`--reuse-from SHA`だけで明示に求め、旧manualの`reuse_approval`は引き継ぐ）。`--reuse-from`だけの明示のAはR0/R1のlaneで使う（packet のある R3/R4 では自動のAが`--review-stage sync`と同じ最初のrecordで効くので要らず、先に記録すると`--review-stage sync`が使えなくなる）。同期だけでcodeに触れない版のclosureは新しいheadでの最初のrecordで`--review-stage sync --outcome pass`として記録し、`--pass-model`・`--run-ref`・`--evidence`・`--reviewed-head`・`--pr-reviews`を取らない。今のheadに本文のあるPR reviewがあれば、id・投稿時刻と`--review-stage closure`の手順を示して何も書かずに止まる。helperがcapture/serverの正当なbroadを保持し、CLI入力から架空のbroadを作らない。対象commentだけを作成/更新する。対象版が変わったrecordでは他kindもpendingへ戻し（Aで引き継ぐmanualを除く。manualが必須で旧manualがpassなのに使い回せない版〈headが同じでbaseだけが変わった版を含む〉では、manual以外のrecordを何も書かずに止める）、必須でないものだけnot-requiredを設定する。Ready/mergeはownerの明示指示を前提にする。

merge時はGitHubのPR head/base・実効rules・必要checkをfreshに取得し、対象CI workflow、GitHub Actions app、必要job成功、review/manualの対応版、Ready、merge可能状態を確認する。`gh pr merge --match-head-commit`で確認後のhead変更を拒否し、strict ruleでbase更新も防ぐ。helperはsettingsを変更せず、`--admin`やbypass fallbackを持たない。未知/不足/HTTP失敗/permission failureは非0で終了する。

通常操作に対するexitは0=成功、1=条件不足、2=入力/通信エラー。statusの待機状態は正常な読取り結果として出し、CI未完了をツール故障として扱わない。PR/branch名やJSONはargv/構造化dataとして扱い、shellへ展開しない。ローカルcacheは表示用の写しで、offline mergeや失われた承認の復元根拠にしない。

## Actions停止時

merge の後の closeout の PR は作らない（D-145 (5)。packet は `docs/plans/` に残り、起点の tree に在るので active でない）。通常squashを維持し、既存commitの履歴保持が必要な例外はownerの明示範囲で扱う。

Actions利用不能時はmergeを停止し、許可済みのlocal作業と証拠を保存する。設定やCIの障害では保護を残して修正PRを準備する。rulesが失われたPRはDraftで保持し、手動mergeで回避しない。設定の復旧が必要なら、事前snapshotと正確な対象をownerへ示して別途操作し、無承認で保護を解除してmergeしない。単なるGitHub障害を理由に再reviewを発注しない。

## 実行手順

R3/R4の例（`PR`は対象PR番号、`PACKET`はそのPRの単一のactive packet）。以下のrecord/Ready/mergeはその操作のowner指示を得てから実行する。captureの返すpathを使い、SHAを手転記しない。reviewed headは reviewer の報告の `REVIEWED_HEAD=<40hex>` の 1 行から `--report FILE` で読む（[review packet](../templates/subagent-review-packet.md) の「報告の先頭の 1 行」）。captureの出力や現在のHEADから取らない（監査していないheadを記録する誤りをhelperがcaptureのheadとの照合で止める）。`PR_REVIEWS`はreviewed headに付いた本文のあるPR reviewを全部読んでから、その数を書く（PRへ投稿しないreviewのrecordでも同じheadの数を書く）。

```bash
python3 scripts/pr-gate.py status --pr "$PR" --packet "$PACKET"
python3 scripts/pr-gate.py capture --pr "$PR" --packet "$PACKET"
python3 scripts/pr-gate.py record --pr "$PR" --packet "$PACKET" \
  --capture "$CAPTURE" --kind review --review-stage broad \
  --pass-model "$MODEL" --run-ref "$PUBLIC_REVIEW_REF" --report "$REVIEW_REPORT" \
  --pr-reviews "$PR_REVIEWS" --outcome pending --evidence "$EVIDENCE"
```

Double Auditは最初のauditをpendingで記録し、fresh captureから次のauditを追加する。必要数とfinding裁定が揃ったときだけpassにする。改版後のclosureはserverに残るbroadを使い、Bのmanualは新しいheadでの最初のrecordにする（前節）。モデル名/run_refは公開可能な実施記録であり独立性の機械的証明ではない。

broad は head を変える push（是正・base 同期）の前に、監査した head の capture で record する。同じ head に closure run が 2 本あるときは、後に完了判定を出した run を model・run_ref にし、両方の証跡 pointer を evidence に並べて 1 回 record する。1 つの review の run が同じ head に PR review を 2 本以上投稿したときも、全部の finding を裁定の対象にし、`--pr-reviews` にその本数を含める（D-107）。
