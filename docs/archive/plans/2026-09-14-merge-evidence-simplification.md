# マージ検証をGitHubへ集約し、証跡の手作業を減らす

## Workflow State

- Phase: archive
- Evidence Mode: legacy
- Risk: R3
- Execution Mode: codex-only
- Plan Commit: 64cedd7e6b1a459ae5d780920b5aab21123baf3c
- Amendments: 84c9f414b7a55ca73d88f70791132a8495aeb83d
- Coordinator: owner（起草・調査は現在のCodex）
- Writer: Codex / Astra xhigh（owner 2026-09-14指定、単独実装。要求モデル・effortであり、実装runの実効metadataは未確認）
- Plan Reviewer: Opus（独立fresh context。初回xhigh、修正確認high）
- Final Reviewer: Sonnet high + Opus xhigh（各fresh contextで独立Contract Audit。workflow gate変更の難度に応じた割当。修正確認のeffortは難度で選び、P3のみの再依頼はしない）
- Reviewed Content HEAD: f28e5891ac42cc8c180de711c151c0aa2e102d3b
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required
- Human Gate: 未実施のruleset probe（合成ref/PR作成・close・削除を含む） / 本番有効化

現行のlegacy workflowで設計・Plan Gateを通過済み。新modeの保護をこの計画に先取り適用しない。ownerは「GitHubで強制し、docs・後処理は軽いPR経路」を選択し、2026-09-14にAstra xhighの単独実装と難度に応じたSonnet / Opusレビューを指定して実装開始を承認した。Codexが計画・amendmentを起草し、ownerが採否を決める個別依頼であり、D-084の一般の役割制限を書き換えない。計画を更新するrunと実装runを分け、実装runはpacketを編集しない。

## Owner Effort Budget

- 介入回数上限: 8（change全体の規範値、owner 2026-09-14承認）
- 実働時間上限: 30分（規範の既定値、実績は未実測）
- relay 往復上限: 2（規範の既定値）
- Plan Review round 天井: 3（規範の既定値）

今回の計画準備は既定3回以内で行い、設計方式の選択を介入1回目、CIのみをGitHubで強制する境界の選択を2回目として記録した。介入3回目の実装採用に付随して、ownerが準備済みGA1とchange全体の介入上限8回（規範値、実績ではない）を承認した。段階的なCI移行・検証用設定・本番有効化に判断が必要なため上限を調整し、元Plan Commitを保持する。既知の判断は方式選択、強制範囲の選択、実装採用、bootstrap Ready、bootstrap merge/closeout、検証用設定の実験、本番有効化であり、残りはreview裁定の余地。まとめて承認できる手順はまとめるが、decision pointの計数を隠さない。過去PRの例外は継承しない。外部モデルへのrepoの公開可能な指示・設計・差分のread-only送信許可は既存の明示承認を引き継ぐ。

現在は介入3回目まで承認済み。8回はこのchange全体の上限で、残り8回の追加ではない。実働時間・relay・Plan Review roundの上限は変更しない。実働時間の累計は未実測。実装開始・GA1・予算の承認を再要求せず、未了の外部操作gateは具体的な成果物の完成・検証後に提示する。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

merge gate、CI選択、Workflow Stateの保存先、main保護を変える。アプリやDBへの変更はないが、古いgreenやskipで未検証の版がmergeされない検証が必要。

## Goal

Goal Invariant:

### 最小完了条件

必要な検証・独立review・owner承認を維持し、通常のReady/mergeでAIがSHAを手集めせず、状態記録だけのcommit/full再実行を増やさない。GitHubがmainへのPR/CI条件を強制し、review/manual/R4はhelperの正規経路で確認する。直接UI mergeは禁止し、docs/closeoutは軽いPR経路で完了する。

### 失敗定義

GitHubが古い/未検証のCI、必要jobのskip/欠落を通すこと。helper経由の正規mergeが古いhead/baseや未了review/manual/R4を通すこと。CI以外をGitHub自体が強制するとの誤説明も失敗とする。保護を有効にして正規のdocs/closeoutを閉塞させること。証跡管理を別の巨大な状態管理基盤へ移しただけになること。

### 非目的

Plan Gateや独立性の撤廃、アプリ/DB/POS変更、依存更新、global Skill/plugin整理、独自署名・DB・GitHub bot・merge queueの導入、発注書46の実行。

## Scope

- S1 CI: `.github/workflows/ci.yml`、`.github/merge-gate-ruleset.json`（新規desired policy）、`scripts/ci/classify-changes.sh`、`scripts/ci/check-required-jobs.sh`（新規）、`scripts/tests/run-workflow-tests.sh`（新規）を使い、全体aggregate・Draftのcheck名分離・docs経路・検証parityを実装する。
- S2 helper: `scripts/pr-gate.py`（新規）。status/capture/record/ready/mergeと、PR内の非CI記録、表示用local cache。既存gh/Python/Bashを使い、新規packageを導入しない。
- S3 state: `scripts/doc-consistency-check.sh`、`scripts/check-workflow-git.sh`、`scripts/pre-push.sh`、`scripts/local-ci.sh`。github markerの新packetとlegacyを区別し、新modeの実装後state-only/三点一致を撤去。Plan Commit/Amendmentsの保護は残す。
- S4 tests: 既存`classify-changes.test.sh`、`ci-workflow.test.sh`、`local-ci.test.sh`、`pre-push.test.sh`、`workflow-git-checks.test.sh`、`doc-consistency-plan-packet.test.sh`、`reading-order-drift.test.sh`。新規`merge-gate.test.sh`、`pr-gate.test.py`。synthetic fixtureで失敗を検出し、helperを実際の呼出経路へ接続する。
- S5 docs: `AGENTS.md`、`CLAUDE.md`、`docs/{DEV_WORKFLOW,ci,AGENT_OPERATING_MANUAL,project-profile,code_review,decision-log,Plans,PROJECT_HANDOFF}.md`、`docs/agent-guidance/{README,shared,context-efficiency,merge-evidence}.md`、`docs/templates/{plan-packet,test-design-matrix,subagent-review-packet,pr-review-prompt,workflow-effectiveness-review}.md`、`.github/pull_request_template.md`、`.codex/README.md`。
- S6 Skills: `.agents/skills/{inventory-workflow-start,inventory-implementation,inventory-code-review,pr-review,review-only-subagent}/SKILL.md`、`.claude/rules/{commands,review-workflow,implementation-quality}.md`、`.claude/commands/{check,test,phase-complete,plan-rally,design-review}.md` の今回の契約参照だけを同期。無関係な本文は編集しない。
- S7 activation: 現repoのmain rulesetの具体的payload・snapshot・read-back手順を用意し、ownerが有効化を承認した後にだけ適用する。検証用GitHub fixtureは合成資料だけとし、mainを変更する負例を実運用のmainへ投げない。

この準備段階で編集するのはsource design、本packet、Matrix、indexとこのbranchのPlansだけ。既存の実行コード・CI・GitHub設定は変更しない。

## Non-scope

- アプリ、DB、実POS/店舗データ、認証情報、公開範囲の変更。
- 新GitHubアカウント/外部サービス、native review承認者の増員。
- global/user-local config、発注書46、既存archiveのschema一括変換。
- 既存doc WARN、npm/Cargo advisoryの便乗修正。

## Acceptance Criteria

- AC1: `merge-gate.test.sh`で必要jobのfailure/cancelled/skipped/欠落/未知値、分類失敗が非0。docsのみではdocs（実PRのPK5含む）/aggregateが成功し、Rust/frontendを必要扱いしない。
- AC2: `ci-workflow.test.sh`とlive dogfoodで、Ready時の分類job failure/cancelledをaggregateのifでskipせずfailureにする。`needs.changes.result == success`をaggregateのifへ足すmutantを検出し、Draft runがrequired名`Merge gate`のsuccess/skippedを作らず、Readyの対象job成功後だけ同名gateがsuccessになる。paths-ignore/本文skip tokenによる抜け道がない。
- AC3: `doc-consistency-plan-packet.test.sh` / `workflow-git-checks.test.sh`の新旧fixtureで、legacyの拒否条件とPlan Gate/Plan Commit/Amendmentsを維持し、新modeは実装後のphase保存のためにtracked変更を要求しない。markerなしactive packet、未知mode、github modeのlocal-verified以後のtracked Phase、必要field不足は非0。legacy archiveは非遡及。
- AC4: `pr-gate.test.py`でstale head/base、複数/不正author記録、必要review/manual/R4欠落、API失敗、誤repo、shell特殊文字、確認後head更新を拒否。正常系では必要APIだけで結果を返し、無関係なPR本文を変更しない。
- AC5: 新modeの通常caseで、Ready/merge前後の`git status`が不変、capture/recordは自動取得した対象版に結び付き、SHA手転記と実装後state-only commitを要求しない（規範の目標。token削減量は未実測）。
- AC6: localとhostedの必要検証集合を比較し、現行local fullにある実PRのPK5、workflow回帰、shell-syntax、workflow-yaml、bindings、traceability、frontend tests/build、env、docsが失われない。hosted PK5はfetch-depth: 0を使い、履歴不足を成功にしない。`local-ci.sh full`自体は保持する。
- AC7: owner承認後に`gh api repos/kosei-w90607/inventory-system-desktop/rules/branches/main`でPR必須・GitHub Actions/15368のMerge gate・strict・bypassなし・削除/force禁止を確認する。sourceの検証用ref手順で拒否とdocs-only PR成功を残す。成功前は旧gateを外さない。
- AC8: repo-wideのlive参照を更新し、旧の無条件三点照合・state-only強制・main直接closeoutはlegacy/migration説明だけに残る。archiveは書き換えない。`doc-consistency-check.sh`と必要testsが通る。
- AC9: bootstrapは旧契約の`local-ci.sh full`がCLEAN/PASS、独立Double Audit、owner Ready/merge、hosted証拠を満たす。新modeの有効化後のcloseoutはdocs-only PRで完了し、自身のcloseoutを再帰要求しない。

## Design Sources

- [目的・動機とMG-D1〜D12](../../agent-guidance/merge-evidence.md)
- [workflow](../../DEV_WORKFLOW.md)、[CI](../../ci.md)、[役割](../../AGENT_OPERATING_MANUAL.md)、[profile](../../project-profile.md)
- [D-033/D-035等](../../decision-log.md)、[review](../../code_review.md)
- 実行の正本は`ci.yml`、classifier、local-ci、pre-push、PK4/PK5と既存tests。アプリ設計正本は非接触。

## Required Design Artifacts

source designはmerge-evidence.md。実装時にCI/workflow正本とdecision-logへ採用内容を同期する。DB/function/UIの新規設計は不要。

## Registration / Generation Obligations

aggregateのrequired contextとGitHub appを実効rulesへ登録する。新testsを共通workflow suiteへ接続し、そのsuiteをlocal/hostedから呼ぶ。新modeをテンプレート・checker・resume/レビューSkillへ同時に反映する。appのbindings/routes/REQ登録は変更せず、既存の生成drift検査を維持する。

## Design Intent Trace

| Spec | Decision | 実装先 | 検証 |
|---|---|---|---|
| SPEC-MERGE-EVIDENCE | MG-D1〜D4 | CI / classifier / gate / ruleset | gate負例、検証parity、live dogfood |
| SPEC-MERGE-EVIDENCE | MG-D5〜D8 | template / checker / pr-gate | 新旧schema、状態遷移、stale/競合/権限 |
| SPEC-MERGE-EVIDENCE | MG-D9〜D12 | closeout / 移行 / docs / 出力 | docs PR、rollback、参照sweep、読取り量 |

## Design Intent Audit

動機、守る失敗条件、採らない案をsourceへ置いた。外部のcheck評価は公式契約とread-only probeで確認し、dynamic名・ruleset拒否のruntimeは有効化前dogfoodへ分離した。分量・token効果の実測値は作らず、実装の判断基準として手転記/状態commitを要求しないことを固定する。

## Impact Review Lenses

環境・再現性: GitHub event/job/checkと実効rules。Fact/design: APIの観測値と提案payloadを区別。Lifecycle: Draft/Ready、stale head/base、offline、失敗、再開。Replacement: old/new packetとbootstrap。Data safety: 合成fixtureと最小の公開metadata。製品UI/会計/POS adapterは非接触。

## Design Readiness

GitHub強制と軽いdocs PR経路はowner選択済み。変更対象・各契約・実装順・負例をこのpacket/Matrixへ具体化した。Plan Gate前にproduction codeへ着手しない。settingsの有効化は完成したpayloadと実効検証をownerが確認する最終段階であり、設計選択を有効化許可へ読み替えない。

## Contract Probe

- `gh api repos/kosei-w90607/inventory-system-desktop --jq '{visibility,permissions}'` → public、admin/maintain/push/pull可（2026-09-14 read-only）。
- `gh api repos/kosei-w90607/inventory-system-desktop/branches/main --jq '{name,protected}'` → main / false。`gh api .../rules/branches/main` → []。
- `gh api repos/.../commits/fda74d083c7b3d0a1f531eaaa3e29693dae094d1/check-runs` →既存CIのappはgithub-actions、id=15368。これはtokenやcredentialではなく公開app識別子。merge方式のallow_squash_merge/allow_merge_commit/allow_rebase_mergeはtrue、全rulesets（無効/評価中含む）の一覧も[]。
- 現物`ci.yml`と`local-ci.sh`を対照し、Rust aggregateがfrontend/docs/envを覆わず、localのworkflow回帰suiteがhostedには未接続と確認。これを直す前にlocal final要件を外さない。
- 公式Contexts表はjobs.nameでgithub contextを許可。dynamic名のDraft/Ready実挙動と設定後の拒否は未実測で、有効化前の必須確認へ置く。
- 一時的な合成Git repoでsource記載の`git diff --binary --full-index --no-ext-diff --no-textconv BASE...HEAD --`を比較した。metadataのみのmain取込みは同一PR差分と親関係を満たし、競合をfeature側へ解消したcaseは差分不一致で再利用対象外となった。出力はlocal-onlyの`base-sync-probe.json`。代表caseの確認であり、manual対象への無影響の証明ではない。

## Contract Coverage Ledger

| 契約 | 実装先 | 自動/独立検証 | manual / non-scope |
|---|---|---|---|
| MG-D1 | ruleset + helper | payload/実効rules照合 | owner有効化承認 |
| MG-D2 | aggregate evaluator | 必要job全結果の負例 | native評価のdogfood |
| MG-D3 | event/名前/分類 | Draft/Ready/docs/skip fixture | 実check名確認 |
| MG-D4 | shared test driver/consumer | local-hosted parity | 現行gateを先に保持 |
| MG-D5 | new marker/template/checker | old/new phase/required fields | archive非変換 |
| MG-D6 | comment record/capture | author/重複/stale/改版 | 独立性と採否はowner/model |
| MG-D7 | local feedback / final条件 | local failure保持・manual未了拒否 | Windows/R4は現行保護 |
| MG-D8 | pr-gate | CLI/HTTP/race/side effect | settings自動変更なし |
| MG-D9 | closeout PR | docs経路、再帰なし | 親の後処理承認を引継ぐ |
| MG-D10 | mode selector/legacy | 既存fixture維持 | 古い成果を勝手に変換しない |
| MG-D11 | rollout/rollback | 前提欠落で停止 | 正確なpayloadのowner承認 |
| MG-D12 | status/JSON/docs | 通常出力/異常詳細 | token効果は未実測 |

## Test Plan

[Test Design Matrix](test-matrices/2026-09-14-merge-evidence-simplification.md)。実装時は意味のある負例を先にREDへし、対象tests→local full→独立review→live dogfoodへ進める。実装中の全量再実行は新しい変更/失敗/未解決懸念に対応する範囲だけ。planning-onlyではdocs/plan checkとread-only Plan Reviewを行い、fullや外部設定mutationを行わない。

## Boundary / Wire Contract

producerはGitHub REST/GraphQLとreview capture。RecordV1のconsumerはhelperだけで、CI evaluatorは分類flagとjob結果だけを扱う。repo/PRとhead/base（現repoのfull SHA）を検証し、未知status・不正JSON・重複marker・不正author・欠落gateは拒否する。comment JSONはsource designのRecordV1を唯一のwire契約とし、Broad/Closureとmanual再利用fieldを含む。R2+のBroad.plan_commitは承認されたPlan Commitと一致するfull SHAを必須とし、nullを拒否する。CI結果は複製しない。branch名等をshellへ展開しない。active packetは明示legacy/github markerを使い、legacyは旧13-fieldを維持する。markerのないarchiveは非遡及。未知modeや新activeでのmarker欠落は拒否する。

## Review Focus

Draftのskippedがrequired成功になる穴、CIの検証parity欠落、settings有効化順序、review/manualの古い版への誤結合、legacyからの無断gate省略、docs closeoutの閉塞/再帰、手作業が新しい記録基盤へ移っただけにならないこと。

## Spec Contract

Contract ID: SPEC-MERGE-EVIDENCE

MG-D1〜D12を実装する。Plan Gate/独立review/owner権限/Windows・R4保護は保持し、証跡の所有・最後の判定・実装後の状態保存を変更する。

## Trace Matrix

| Spec | Step | Test | Review focus | Evidence |
|---|---|---|---|---|
| SPEC-MERGE-EVIDENCE | S1/S4 | merge-gate.test.sh、ci-workflow.test.sh、classify-changes.test.sh | false green、parity | local/CI |
| SPEC-MERGE-EVIDENCE | S2/S3 | pr-gate.test.py、PK4/PK5既存fixture | stale/race/mode | local/独立review |
| SPEC-MERGE-EVIDENCE | S5〜S7 | docs check、rules read-back、docs PR | 移行・権限・手作業削減 | PR/live dogfood |

新規test名は予定であり、既存の実行済みcoverageとは扱わない。

## Data Safety

実データ、auth/env、credential、個人memoryを読まない/commitしない。API probeはrepo/rules/checksの公開可能metadataだけ。capture/cacheと詳細logsはignored `.local/merge-evidence/`または`.local/pr-gate/`。mainを改変する失敗実験はせず、synthetic API fixtureと検証用refで試す。

## Implementation Results

設計・計画・MatrixとPlan Reviewは完了済み。2026-09-14にownerが実装開始、座組、準備済みGA1とrollout予算を承認した。GA1のcontent commitに`plan-approved -> implementing`を同乗させる。元Plan Commitを保持し、amendment SHAを記録して別runのAstra xhigh単独実装へ渡す。実装の検証結果は取得後に記録する。

最新結果: 実装候補`4c5a241b`のlegacy fullと独立最終closureは完了し、P1/P2なし。詳細は末尾の「最終検証・引継ぎ」を参照。残るP3のowner裁定前なので、最後に実体化したPhaseはimplementingを保持する。裁定後、既に証拠が揃った隣接遷移をまとめて記録する。

## Review Response

- Findings Freeze: initial Broad Audit completed on 8c35a79b（Sonnet / Opusの独立2監査完了、以後closure）

Opus初回reviewのP2を修正する案として、aggregate ifの負例、hosted PK5/parity、base同期時のclosure/manual適用判断、docs probeの実施順序、条件付きsource、rollout予算案を具体化した。方向性の変更や稼働設定の適用は行っていない。追加Plan Reviewで確認し、必要なowner裁定は実装開始の判断に提示する。R2の強制範囲指摘についてはownerがCIのみをGitHub、その他をhelperと運用規律で確認する案を採用した。schemaはsourceのRecordV1へ統一する。現行コードや設定を変更せずに、すぐ実装へ移れるところまで具体化する。

### Plan Review closure（2026-09-14）

独立Opus Highが候補`f378da4d`をround 3で確認し、P1/P2=0、技術的Plan Gate通過と報告した。R2 N-1（強制範囲）とN-2（wire契約）はclosed。初回plan-first commitとMatrixが実装に先行し、production codeは未変更。この結果を根拠に`plan-gate -> plan-approved`をmaterializeした。この時点では実装開始、rollout予算案、本番設定の承認は未取得だった。現在の承認状態はWorkflow Stateと次のGA1候補に記録する。

レビューの補足P3は追加reviewを回さず、実装開始時の予算amendmentと合わせて扱う確認事項として保存する。現候補の契約を変更済みとは扱わない。

- P3-1: manual再利用の元server recordが実際にpassだったこととevidenceの出所を確認する負例、版変更後の記録順序を具体化する。closureを先に記録して旧manualを失う費用と、未passの結果を再利用する危険を防ぐ。
- P3-2: PacketのBoundaryに、RecordV1のconsumerはhelperだけであることを明示する。CI evaluatorが扱うjob結果との記述を分ける。
- P3-3: main向け合成failure PRの作成・closeを、bootstrap Ready時に提示する具体的な操作範囲へ含める案とする。ruleset probeの後の承認を先取りしない。
- P3-4: 運用正本・入口・PR templateへの同期確認に、helperの強制範囲、直接UI merge禁止、server側の残存リスクを含める。
- 補足: R2+のBroad.plan_commitにnullを許さない扱いを明文化する。契約を変えないamendmentでも同一性の機械照合は新しいbroadを要求し得るため、当面は安全側の費用として扱い、証明形式を追加して回避しない。

詳細reviewと検証ログはlocal-onlyの`.local/merge-evidence/plan-review-r3.json` / `plan-doc-check-r3.log`。GitHub上のdynamic check名・実効拒否・docs経路は未実測で、source記載の有効化前検証を維持する。

### Gated Amendment 1（2026-09-14、owner採用済み）

ownerは実装開始とWriterのAstra xhigh単独指定に続き、提示済みの予算案と以下の補足を「いいよ」と承認した。元Plan Commitを保持し、GA1のcontent commitで`plan-approved -> implementing`へ遷移する。根拠は独立Plan ReviewのP1/P2=0、実装開始承認、GA1の採用。amendment SHAは後続commitでAmendmentsへ追記する。

- rolloutの介入上限はchange全体で8回へ変更した（規範値、実績は未実測）。
- P3-1: manual再利用は元server recordのpass・source_head・元evidenceとの一致を検証し、再利用manualを先に記録してからfresh captureで現在版closureを記録する。元記録の欠落・fail/pending・改変と、closureを先に記録して旧manualを失った場合を拒否する負例をMatrixへ追加した。
- P3-2 / 補足: RecordV1はhelper専用、CI evaluatorはjob結果専用と明記し、R2+のBroad.plan_commit=nullを拒否する条件をsource / Boundary / Matrixへ同期した。
- P3-3: main向け合成failure PRの作成・closeは、bootstrap Ready時に提示する具体的操作範囲へ含める。今回の実装開始承認をその操作やruleset probeの承認へ読み替えない。
- P3-4: 運用正本・入口・PR templateへの同期確認に、helperの強制範囲、直接UI merge禁止、server側の残存リスクを明記した。
- レビューはSonnet highとOpus xhighの独立Double Audit。両方が同じsource contracts全体を監査する。修正確認は難度でeffortを選び、P3だけで追加reviewを発注しない。要求値と取得可能な実効metadataを区別して実装・reviewのevidenceへ残す。

### 実装と初回Double Audit（2026-09-14）

単独WriterがCI/helper・新旧gate・tests・正本/Skills同期・desired payloadと有効化手順を実装した。候補`8c35a79b`のlegacy local fullは開始/終了CLEANでPASS。固有evidenceと初回候補の失敗経緯はlocalのPR本文下書きに保持する。旧ignored snapshotの生成TSがlint対象へ混入したため、対象artifactだけhash一致で可逆退避し、復元情報をignored evidenceへ保存した。製品やlint設定は変更していない。

Sonnet highとOpus xhighが同じ候補を相互の結果を見ずに独立監査した。実効primary modelは取得metadataでSonnet 5 / Opus 5を確認。SonnetはP1/P2なし、Opusは次のP2を報告し、rootもsource/実装/fixtureの該当箇所を照合した。sourceにある既存条件を満たす修正として同じWriterが再現・修正を進める。Phaseはimplementingを保持し、未解決のままhuman-confirmへ進めない。

- O-P2-1: active packetのdirectoryがGit上にない正当なR0/R1を、APIの一般失敗と区別して処理する。親directoryで不在を確認し、他のHTTP失敗は拒否を維持する。
- O-P2-2: headのRisk/mode/review下限/manual・R4要件を承認snapshotに結び付け、追補なしで必須条件を弱める変更を拒否する。業務Risk判断やowner承認そのものを機械が証明する仕組みは追加しない。
- O-P2-3: review下限、R4要件、strict/bypass/enforcement/driftの負例を追加し、guardを外す実mutationで検出力を確認する。
- S-P3-1（追補削除のnegative）とO-P3-5（full分類pathのnegative）は既存Matrixの負例補強に合わせて扱う。他のP3は原文と確認事実を保持し、ownerの裁定候補とする。P3だけを理由に追加Broad Auditを発注しない。

`scripts/tests/claude-hooks.test.sh`のwiring同期は、S1のshared suite抽出・S4の実呼出経路接続・MatrixのCI/分類の全consumer監査に伴う従属作業として実施した。local→suiteとsuite→auditの切断mutantでD-059の保護を確認した。新しい契約やgateを追加・緩和するScope拡張ではない。

### 限定closureと追補順序の再現（2026-09-14）

候補`3997ee7e`でO-P2-1〜3を修正し、legacy local fullはCLEAN/PASS。Opus highが静的な実物・既存evidenceの照合で元のP2をclosedと判定した。独立実行を試す前の広域探索commandが拒否され、reviewerは許可済みtest/driverを実行していなかったため、独立再実行済みとは扱わない。拒否対象はrepo外を含む探索であり、Bash全体の利用不能ではない。

同closureのF1（Amendmentsの並べ替え）はFreeze後の新規指摘だったため、まず合成git/CLI fixtureで再現した。正順ではmanual必須だった登録済み追補を逆順にして古い条件へ戻すと、PK5が成功し、helper captureもmanual不要の条件を保存することを確認した。期待拒否のassertionがREDになった実測は`.local/merge-evidence/amendment-order/red-f1.log`。runtime failureを根拠にP2として扱い、S3の共有PK5で順序も含むappend-onlyを守る最小修正を進める。helper用の別履歴サービスは追加しない。

mutation用の一時copyには、同じ選択testが無変更時にGREENになるpositive controlも追加する。作業状態の古い記述は最終結果と合わせて更新する。その他のP3は引き続きownerの裁定候補として保持する。

### 最終検証・引継ぎ（2026-09-14）

F1は共有PK5の順序付きprefix検査で修正した。候補`4c5a241b5f594a3fb6e0838c88e31b5d14b3756c`のlegacy local fullは開始/終了とも同じHEAD・CLEANでPASSし、MERGE_EVIDENCE_VALID=true。固有evidenceは`.local/ci-evidence/local-ci-full-4c5a241b5f594a3fb6e0838c88e31b5d14b3756c-20260914T102350502999470+0900.log`。既存docs/npm auditのWARNは元の扱いを維持している。

同じ候補へのOpus highの独立最終closureはF1/F2と元のO-P2-1〜3をclosed、現在のP1/P2なしと判定した。実効primary modelはmetadataでOpus 5を確認。独立側がhelper test、PK5 fixture、positive control付きmutation driverを実行し、無変更GREEN→guard欠落RED、差分とdriverのbytes/hash一致を確認した。full自体の再実行を行ったとの主張ではなく、full evidenceの該当行も独立に照合した。初回Sonnet/Opus Double Audit、修正差分、各closureの原文と実行metadataは`.local/merge-evidence/`に保持し、公開用の要約はPR本文下書きへまとめた。

P3のうち追補削除/full分類のnegative、F2のpositive control、F3の現在地同期は対応した。以下は未採用の裁定候補として保持し、勝手にrebut/no-action/受容へ変更しない。

- native rulesetの配列順序/既定parameterによる安全側の誤拒否（S-P3-2 / O-P3-6）。有効化前のlive read-backで実物を照合する候補。
- workflow回帰が必要な変更と、CI実行制御/Double Auditが必要な変更の判定を分ける改善（O-P3-4）。
- 非owner markerとowner record重複の拒否理由の区別（O-P3-7）。
- shell列挙失敗と文字列parity assertionの検出力補強（O-P3-8）。
- PK5のWorkflow State節限定（O-P3-9）。元SHAの表記変更を許す提案は原識別子不変の契約と整合する裁定が必要。
- merge commitのみの登録や履歴改変の検出範囲、古い未登録commitを追補として採用する場合の扱い（最終closureの追加P3）。実際のowner承認の機械的証明は非目的で、追加P3にruntime failureの根拠は提示されていない。

GitHubへのpush/PR作成・Ready・merge・合成probe・ruleset有効化は未実行。実装開始・GA1・累計介入上限の承認を再要求せず、次は残るP3のowner裁定。以後の公開/Readyでは具体的なbootstrap操作範囲を提示し、MG-D11のdocs-only dogfood・probe・有効化順序を維持する。現在の保留は製品側の未決事項や衛生batch 4を採用する判断ではない。

### Owner裁定と公開準備（2026-09-14）

ownerが「着手条件付きで後続保持する」と明示裁定した（介入4回目、累計上限8回という規範値は変更しない。実働時間は未実測）。上記の未採用P3はdeferredとして保持し、以下の条件で扱う。実装開始・GA1・予算・このP3裁定を再要求しない。

- GitHub設定応答との互換性は新方式の利用開始前に実物を確認し、不一致なら先に修正する。
- 誤拒否や過剰なreview要求は、試運用での再現または実際の負担が確認されたときに着手する。
- test/履歴検査の補強は関連する改修や特殊な履歴操作を計画する時点で再評価する。
- 検証・承認をすり抜けるruntime failureが確認された場合は保留を解除して修正する。本番で問題が出るまで待つ裁定ではない。

実装候補4c5a241bのCLEAN local full、Sonnet/Opusの初回Double Audit、修正後の独立Opus最終closure、P1/P2なし、今回のowner裁定は揃った。Draft PRを公開してその本文へevidenceを記録した後、`implementing -> local-verified -> independent-review -> human-confirm`を隣接遷移として実体化する。現在はPR未作成なのでPhaseとReviewed Content HEADを先取りしない。4c5a241b以降の差分はWorkflow State・Plans・append-onlyの進捗/証跡だけで、Scope/AC/Matrix/実装を変更していない。公開・Readyと後段の外部操作は具体的な操作範囲を提示する。

### 公開前L0是正・PR公開とhuman-confirm（2026-09-14）

ownerは提示済みの本branch公開・Ready・分類失敗fixture PRの作成/検証/closeを明示承認した（介入5回目、上限8回は規範値のまま。実働時間は未実測）。本PRのmerge、ruleset適用、試験branch削除はこの承認に含めない。

公開前の実L0は、旧い全Rust関数名へのREQ番号述語が既存のSPEC技術テストを拒否してFAILした。S3/S4のpre-pushの不整合としてコード変更を状態記録と分離し、canonical traceabilityへ集約した候補`f28e5891ac42cc8c180de711c151c0aa2e102d3b`で実L0とlegacy fullがCLEAN/PASS。T1〜T4とsrc-tauriは変更していない。Sonnet highの限定closureは実物/evidenceを照合してP1/P2なし・契約維持と判定した。元のDouble Auditと後続保持裁定は維持する。

[PR #54](https://github.com/higekiri-w/inventory-system-desktop/pull/54)をDraftで公開し、本文へ当該contentのfull、監査、P3裁定を記録した。公開時のPR head/base/本文を実物照合済み。これらの証拠とowner裁定を根拠に、`implementing -> local-verified -> independent-review -> human-confirm`を隣接遷移として実体化し、Reviewed Content HEADを当該監査contentへ設定する。この遷移にはScope/AC/Matrix/実装を同乗させない。既に承認されたReadyは次のstate-onlyとそのexact HEADのL1を完了してから実行する。

### Ready遷移とDraft観測（2026-09-14）

ownerの公開・Ready承認に基づき、Draft中に`human-confirm -> ready-hosted-final`を記録する。Reviewed Content HEADはf28e5891を保持し、このstate-only後の確定HEADでCLEAN L1 fullを実行してPR本文へ記録してからReadyにする。現HEADのL1 SHAをこのtracked fieldへ自己記録しない。

初回Draftの実GitHub runで全jobがskipped、runner割当なし、required名のMerge gateが存在しないことを確認した。ただしaggregateの表示名は設計の短いDraft名ではなく未展開の条件式だった。差異を`.local/merge-evidence/pr54-draft-observation.json`に保存し、Ready時の実check名・結果を続けて確認する。正規Readyの成功と分類失敗fixtureのfailureを実証するまではマージ証拠が揃ったとせず、この観測だけで本番保護を有効化しない。


### 実装mergeと機械的closeout（2026-09-14）

[PR #54](https://github.com/higekiri-w/inventory-system-desktop/pull/54)を2026-09-14T03:51:19Zにsquash mergeした。merge commitは`786824584778cca9c85596c749c8f1e3cb39f848`。ownerは介入6回目 / 予算8回で、実装PRのmergeと本R0 docs-only closeoutの公開・Ready・CI確認・mergeまでを承認済み。現在のPhase=archiveは実装PRの記録移送を表し、ruleset有効化の完了を表さない。上の「未実行」「修正中」等は各記録時点の履歴として保存する。

実装PRのReady対象版は`d483ad5f4145e7e190ab254a8bf8dd4928f7f88d`。[正常run](https://github.com/higekiri-w/inventory-system-desktop/actions/runs/34802569373)は全必要jobとGitHub Actions/15368の`Merge gate`がsuccess。legacyのCLEAN L1・PR head・successful hosted headShaの一致はPR #54本文に記録し、独立Double Auditと各修正closure、owner Ready/mergeを完了した。

[Draft run](https://github.com/higekiri-w/inventory-system-desktop/actions/runs/34801155403)ではrunner割当なし・required名なしを確認した。aggregateの表示は短いDraft名ではなく未展開の条件式だった差異を保持する。Ready時は実際の`Merge gate`名で成功を確認し、[分類失敗fixture PR #55](https://github.com/higekiri-w/inventory-system-desktop/pull/55) / [run](https://github.com/higekiri-w/inventory-system-desktop/actions/runs/34802715739)ではfilter/changesと、起動した評価器/`Merge gate`のfailureを確認した。fixtureはmainへmergeせずclose済み。試験branchの削除は今回の承認範囲に含めず保持する。

このcloseoutはpacket/Matrixの移送とPlans更新だけを行うR0 PR。docs（実PK5を含む）と`Merge gate`の同一HEAD成功、Rust/frontend非実行は、このcloseout PRのhosted結果で確認してからmergeする。現時点でそのdocs-only経路を成功済みとは記録しない。成功head/baseは次のruleset probeのpositive controlへ渡し、このcloseout自身の後処理PRは再帰作成しない。

未実施は検証用ruleset/refの作成・拒否試験・cleanupと、本番main rulesetの適用・read-back。いずれもownerの具体的な承認後に行い、成功前は新modeのReady/mergeを利用しない。「Owner裁定と公開準備」のP3着手条件をそのまま引き継ぐ。設定応答の互換性は利用開始前に実物確認し、不一致なら先に修正する。workflow effectivenessのdogfood対象は**新mode有効化後の最初のR2+ PR**とする。token削減量とowner実働時間は未実測のまま。
