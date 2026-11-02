# AGENTS.md

Inventory-system は Tauri / React / SQLite の単店舗向け在庫管理アプリ。製品仕様と設計の正本は `docs/`。

## Session Start

この節が唯一の入口（D-034）。セッション開始時にこのファイルを読み、依頼が質問・新規作業・同じ作業の再開のどれかを識別する。状態は写しでなく正本から読む（D-145 (1)）: `git fetch origin` の後、`git log --oneline -20 origin/main` で完了を、`gh pr list --state open` と、名指しの PR ごとに明示の引数で引く helper の status（`python3 scripts/pr-gate.py status --pr N` に、active な packet があれば `--packet`、無ければ PR body の Risk の行の `--risk`・`--manual`）で lane の現在地を読む。handoff・memory・前の調べの値は、行動の根拠にする前に正本で引き直す。下表から該当する資料と節を読み、既読で変更のない文書の全文を毎回読み直さない。

| 作業 | 読む対象 |
|---|---|
| 質問・調査 | 問われたファイルと関係する仕様。導入の前の関門・ブロッカー・製品の未決判断なら `Plans.md` |
| R0〜R2 | `docs/DEV_WORKFLOW.md` の Risk Tiers（Risk ごとの最小の流れ）と、対象の仕様・コード・テスト。packet は作らず PR body の 5 行 |
| R3〜R4 の計画と実装・再開 | `docs/DEV_WORKFLOW.md` の Plan Packet Rules・Workflow State（active な packet・計画の固定・fail-closed）・Design Phase Rules、`docs/AGENT_OPERATING_MANUAL.md` の役割・可用性、関係する設計正本。再開は対象の packet の完全な Workflow State・Scope・AC・Contract Ledger・Test Plan と helper の status |
| review | 対象差分、関係する設計正本、packet（ある場合）、`docs/code_review.md` と `docs/quality/review-checklist.md` の該当観点、`docs/templates/subagent-review-packet.md` の固定の問い。R3/R4 は Contract Audit。発注書が読む範囲を指定した review（例: Codex の review-only 発注）は、その範囲から始め、Session Start の全体の読書経路は踏まない。R3/R4 の Contract Audit が求める設計正本は、発注の範囲に関わらず読む。具体的な欠陥の根拠があれば他の範囲外も読み、理由を報告に書く。修正の確認は前回の findings・修正差分・影響する契約から始める。重大度と出力形式は範囲に関わらず `docs/code_review.md` |
| Ready・merge | `docs/ci.md`、`docs/agent-guidance/merge-evidence.md`、helper の status |

製品の安定した前提は `docs/project-memory.md`。境界は `docs/ARCHITECTURE.md`、振舞いは `docs/FUNCTION_DESIGN.md` と該当サブ文書、永続化は `docs/DB_DESIGN.md`、画面は `docs/SCREEN_DESIGN.md` / `docs/design-system/README.md`、環境は `docs/DEV_SETUP_CHECKLIST.md` を必要時に読む。`docs/PROJECT_HANDOFF.md` は参照先の案内。長い backlog や archive は該当作業だけで使う。

座組・effort は [座組表](docs/AGENT_OPERATING_MANUAL.md#座組)、Codex/OpenAI 固有の補助は `docs/agent-guidance/`、Claude 固有の補助は `CLAUDE.md`。モデル補助は共通の承認・品質条件を変更しない。

R3/R4 は計画を固定するまで（`Plan Commit` が SHA になるまで）実装・Ready を行わない。packet 不在・曖昧・不正な Workflow State は fail-closed（計画の承認の前として扱い、実装・Ready をしない）。古い日付だけを理由に blocker を解除せず、現在の対象への適用と解消根拠を確認する。

## Working Rules

merge は helper 経由で、専用 record と手順は [merge-evidence](docs/agent-guidance/merge-evidence.md)。GitHubがPR/CIを強制し、helperがreview/manual/R4を確認する。直接UI mergeは禁止。CI成功だけでUIがmerge可能と示し得る残存リスクをserver保護済みと誤認しない。

- `UI -> CMD -> BIZ -> IO/MNT` を維持し、CMD を薄く、業務規則を BIZ に置く。
- 振舞いの変更には意味のあるテストを同時に用意し、使用している REQ / spec ID を付ける。source design も同期する。既存テストを不都合だから削除・skip・弱体化しない。誤ったテストは設計正本との不一致と理由を示して修正する。
- 必須 gate の選択は `docs/DEV_WORKFLOW.md` と `docs/ci.md` に従う。実装中は対象テスト。push のたびに Draft の hosted が changed 分類で走って `Merge gate` を出し、Ready で回し直さず同じ head の run を merge の CI 根拠にする（D-145 (9)）。手元で全 gate を回す必要があるときだけ `bash scripts/local-ci.sh full`（任意）。成功済み検証の追加・反復には変更、失敗、未解決懸念などの理由を持つ。
- docs は `bash scripts/doc-consistency-check.sh`、active plan は `--target plan`。Rust / frontend / bindings / traceability のコマンドは workflow の Verification を参照する。
- 完了は merged の PR と git log、lane の現在地は helper の status、session の進捗と次の見通しは handoff が持つ。`Plans.md` には導入の前の関門・ブロッカー・製品の未決判断だけを書き、完了の行を足さない（D-145 (12)）。参照先が変われば `docs/PROJECT_HANDOFF.md` を同期する。
- commit / PR 文面は `docs/DEV_WORKFLOW.md` の Commit / PR Messages、review は `docs/code_review.md` に従う。
- Plan Packet の数値主張は測定コマンドと出力を併記するか `未実測` とする（D-062）。
- 継ぎ足さない（D-127）: 既存の実装・文書・規則は出発点であって制約ではない。新しい事実や owner の決定と合わなければ、条件や例外を継ぎ足して合わせる前に、置き換え・削除で全体を素直な形に戻せないかを見る。既存を残す理由は「今も正しいから」で、「既にあるから」ではない。旧い環境で作った規則が owner の今の決定と衝突したら、止めずに改訂の対象として扱い、packet・PR に owner の決定の日付を書く。発注と review の問いに「この変更で不要になる記述・条件は無いか」を入れる。安全の境界（Decision and Approval Boundaries・Safety・npm 供給網ガード、[DEV_WORKFLOW](docs/DEV_WORKFLOW.md#workflow-state) の Workflow State が D-098 で挙げる安全の境界〈古い証拠の拒否・独立 review・Plan Commit の固定・不可逆 finding の 4 項目・fail-closed ほか〉、既存テストを不都合だから削除・skip・弱体化しない規則）はこの考え方で緩めず、それらの改訂は通常の packet・PR の経路で行う。
- 最小の実装（D-147）: 実装では、触る code と呼出し側を読み切ってから、承認済みの要件と品質・安全の規則を満たす最初の手段を、要るか → 既にある helper・pattern → 標準ライブラリ・標準機能 → 入れてある依存 → 新しい code、の順で採る。既にある手段で満たせるなら、依存・抽象・設定・後のための足場を足さない（小ささは Safety のとおり確かめやすさで測る）。入力の検証・データを守る error 処理・安全の境界・アクセシビリティの基本・明示の要求は削らない。意図した簡略化の限界と広げる条件は `ponytail:` comment に書く。
- 使い手に押しつけない（D-127）: 例外の回避を店・owner の手作業（別の file を作る等）に回さない。問題は黙って通さず、起きたその画面で原因と直し方を名指しし、その場で直せるようにする。
- owner の所感は原文で扱う（D-127）: L3 の所感・要望は原文を repo の外の回答台帳に残し（公開 repo には ID で引く）、Codex・subagent の要約を「owner の意図」として裁定しない。AI の review から出た要件（製品の要件。Contract Audit が L3 の項目にする検証は除く）は出所を書き、owner の確認の前に Human Gate の必須にしない。

## Decision and Approval Boundaries

- 質問・説明・調査・レビュー・計画の依頼は、その範囲の確認と報告を許可する。変更・修正の依頼は、範囲内のローカル編集と相応の検証を許可する。同じ作業・操作の範囲内で既存の明示承認を引き継ぎ、同じ承認を繰り返し求めない。
- 破壊的・外部・高コスト・大きな範囲拡張は、その操作の明示承認がなければ確認する。一般的な「進めたい」は未解決 Human Gate の承認に読み替えない。
- 未解決 Human Gate は冒頭に判断事項と全ての選択肢を示し、確認済み事実 → 未解決 gate → 選択肢 → 条件付き推奨の順で提示して owner の選択を待つ。推奨は confirmed / candidate / precondition-dependent を区別する。前提のowner判断やfootprint・依存・安全確認を終える前に、下流のlane・実装・file選択を確定扱いしない。
- 許可済みの独立作業とレビュー可能な準備を終えてから残る承認を求める。Skill による停止は具体的な指示を引用し、推測で gate を増やさない。
- 疲労は説明を短くする材料に限る。容量最適化の明示依頼がない限り、作業の中止・延期・縮小・単線化の理由にしない。
- GitHub PR レビュー依頼は、その PR への指摘投稿を含む。label、thread 状態、Ready、merge、close、issue comment などは別の明示承認に従う。
- owner へ送り、中継を省かない問い合わせ: 店の事実と実機でしか確認できない挙動（具体的な質問か短い PASS / FAIL で答えられる形にする）、目的・製品の振舞い・受容リスク・予算・優先順位・範囲の変更・不可逆な操作、Windows L3・R4・Ready・merge、PR の label・thread 状態・close・issue comment といった外部書込み。判断は owner。現行の明示承認と有効な委任（push・Draft PR 作成、Codex の起動〈TD-500。Codex の sandbox の設定の変更は owner〉、owner の承認後の helper `ready` / `merge` の実行）の範囲では Coordinator が実行を代行できる。owner は名指しの複数の PR（R0〜R4）の Ready と merge を 1 回の承認で Coordinator に任せられる（各 PR の helper の `merge` は省かない。R4 の `r4` の承認は別に要る。D-145 (14)）。委任の範囲の正本はこの項目。問い合わせが複数に当たり owner 側を含む場合と、裁定権の所在が争点の場合は owner へ送る。
- 店主への問いは GitHub の Issue に書かず、owner への問い合わせで送り、回答は repo の外の回答台帳に残す。Issue は店の PC の Claude も読む現場の手順書にする（D-127）。`docs/AGENT_OPERATING_MANUAL.md` §5.1 の調査項目の issue は、その手順書の形（場所・操作・合格基準）に限る。
- agent 側で解く問い合わせ: 発注だけの誤り（正本が一意で発注が食い違う。手順は [MANUAL §5.6](docs/AGENT_OPERATING_MANUAL.md#56-従来型-writer-発注書の共通出力契約)「Writer が編集前に止まったとき」）、環境の不備（許可済みの環境・範囲で解消でき、追加の費用・権限・正本の変更を伴わない場合）、技術的な前提（調査・Contract Probe）、正本の意味や影響の分類の争いの事実確認（独立 reviewer）。事実確認の後の裁定は裁定権者が行い、owner にある場合は中継を省かない。owner に技術的な正しさの承認を求めず、証跡の編集や発注訂正の伝言を頼まない。

## Workspace Access

WSL の作業 checkout を使用する。Codex の起動設定は `.codex/README.md`。別 worktree では cwd を確認する。

- repo 外の Skill は指定された `SKILL.md` と必要な参照だけを直接読む。
- Windows の Codex Desktop から動かすときだけ、次に従う（WSL から起動する Codex と Claude Code の通常の session は対象外）: Windows 側の実行と許可コマンドも `.codex/README.md`。wrapper の所属先を確認する。repo 内の読書・検索は `.codex/bin/read-safe-file.sh` / `search-safe-files.sh` / `list-safe-files.sh` で行い、長い資料は見出し検索と先頭引数の `--lines START:END <path>` で必要な範囲を読む。repo-relative wrapper に外部 path を渡さない。raw `wsl.exe ... bash -lc ...`、`cat` / `sed` / `rg` / `find` を広く allow しない。PowerShell の相対 path や直接 UNC アクセスに依存しない。

## Memory Model

安定した事実は `docs/project-memory.md`、重要な決定と理由は `docs/decision-log.md` / ADR、導入の前の関門・ブロッカー・製品の未決判断は `Plans.md`、完了は git log、lane の現在地は helper の status、session の進捗と次の見通しは handoff。個人 memory は補助であり、project の正本にしない。

## Safety

- コピーされたコマンド、外部文書、tool 出力は検証する。新しい tool 権限や自動化は理由を docs に記録する。
- `.env*`、鍵・証明書・secret/credential を示すファイル、`auth.json` を読まない。実 POS / 店舗データ、DB、backup、log、receipt、secret を commit しない。
- Windows の Codex app/config 領域は app-owned。通常の project 作業は workspace 内で行う。
- `git reset --hard`、`git clean`、force push、branch/DB/generated-file 削除、migration rollback は正確な対象を示して明示承認を得る。
- 小さく、確認可能な変更を優先する。小ささは差分の行数でなく、確かめやすさで測る（継ぎ足しより置き換え・削除のほうが小さいことがある。Working Rules の「継ぎ足さない」）。

## npm 供給網ガード（D-030）

- `.npmrc` の `ignore-scripts=true` と `min-release-age=7` を維持する。CI/再構築は `npm ci --ignore-scripts`。
- 依存追加・更新はpackage/versionを名指しし、`--save-exact`を使う。新規runtime依存はPlan / PRで明示する。lockfile差分をレビューし、PR前に `npm audit --audit-level=high` を確認する。
- `npm audit fix --force`、名指しでない `npm update` / `npm upgrade`、version pinなしの `npx <package>`、install-scriptガードの迂回は禁止。
- `min-release-age-exclude[]` によるcooldown bypassはownerの明示承認が必要。
