# Plan Packet: docs の小口 2（座組表の Fable を当面 Fable 5 に・Fable 5 の定義・ARCHITECTURE §4 の決まった項・ECR+ の精算の前提・棚卸しの期間の旧表記・71 の起動の順序）

wave に属さない単独の lane。docs と、`.claude/agents/` の subagent の定義 2 つの追加。code・migration・bindings・画面の振舞い・helper・classifier・CI の設定は変えない。

出所: (1) owner の決定（2026-10-24、repo 外の回答台帳 TD-410）: 座組表の Fable を当面 Fable 5 にする。(2) `docs/backlog.md` で「次の docs の小口」「次の docs の lane で直す」「docs 小口」とされた未着手の項目のうち、agent が直し方を一意に決められるもの。(3) Coordinator の裁定（2026-10-24、Plan Review round 1 の是正）: Final Review の Claude 側と相談役を Fable 5 で起動する定義を、PR #127（`.claude/agents/{writer,reviewer}.md` を tracked にした前例、D-099 の (D3)）に倣い本 lane で tracked にする。(4) Coordinator の追加（2026-10-24、PR #198 の起草役の sweep）: `docs/function-design/71-mnt-backup.md` §71.9 の手順の番号が今の `src-tauri/src/lib.rs` の `run` の setup と合わない。(5) owner の決定（2026-10-24、repo 外の回答台帳 TD-429）: backlog の「squash merge の subject 明示」の項目を外す（S5-3）。下の行番号と数は起点 `843c4938`（`origin/main`）で command を逐語で実行した出力（D-062）。本 packet は dated の Plan Packet なので、行番号を起票の時点の観測として書く（`docs/DOC_STYLE_GUIDE.md` §1 の対象外の項）。直した後の正本の文は行番号で引かない（D-125）。

規模と手法: 触る file は 9（`docs/AGENT_OPERATING_MANUAL.md`・`docs/ARCHITECTURE.md`・`docs/backlog.md`・`docs/architecture/biz-task-specs.md`・`docs/db-design/tracking-system-tables.md`・`docs/SCREEN_DESIGN.md`・`docs/function-design/71-mnt-backup.md` の 7 つの docs と、新設の `.claude/agents/reviewer-fable5.md`・`.claude/agents/consult-fable5.md`）と本 packet。段を S1〜S7 に分け、段ごとに AC を置く。起票と Plan Review の是正の run は packet だけを置き、docs の直しと定義の追加は plan-approved の後の別の run で行う（先例: docs の小口〈`docs/archive/plans/2026-10-23-docs-small-batch.md`〉。定義の追加も Scope に書いてから実装の run で足す: PR #127〈`docs/archive/plans/2026-09-29-harness-pr5-gate-holes.md` の S10〉）。

起票の時点で済んでいたもの（Scope に入れない）: backlog の「次の docs の lane の候補（docs の棚卸し lane 1 の残り）」の項目（2026-10-20）は PR #192 が (1)〜(8) を直すか直さない理由を残して項目ごと外し（同 packet の S6b）、`docs/Plans.md` の `## 次の行動` の同じ名の句は PR #199（closeout）が外した。`843c4938` で `rg -n '次の docs の lane の候補' docs --glob '!docs/archive/**'` は何も出さない（exit 1）。#192 の Non-scope に残った backlog の「MSI の製品名と upgrade code の固定」の旧名の句も、項目ごと PR #191 の後に無い（`rg -n 'MSI の製品名と upgrade code の固定' docs/backlog.md` は「⑤ go-live 検証 flow」の 1 行の「済んだ」の句だけを出す）。

## Workflow State

- Phase: archive
- Risk: R3
- Plan Commit: f628c918027295b19c6572433ca30b7629996c5f
- Amendments: 975090bff4d2b7220c2a38a5561e70e0a4fa6772, ffa57b09590a8d08193585d547afc9968e691c84
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（起票と Plan Review の是正は packet だけ。docs の直しと定義の追加は plan-approved の後の別の run）
- Plan Reviewer: fresh Opus 5.5（fork でない）+ Codex。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Reviewer: fresh Opus 5.5（Claude 側。R3 なので座組表の「R3 以上」の行は Fable 5〈TD-410〉だが、本 lane は座組表の §3.3 の代え方で fresh Opus 5.5 にする。理由は下の注）+ Codex。座組表どおり
- Final Review Minimum: 2
- Human Gate: ready,merge
- Branch: agent/docs-small-batch-1024

Final Review Minimum 2 は、classifier が `workflow=true` を出すため（`docs/DEV_WORKFLOW.md` Workflow State の `Final Review Minimum`）: S1 の `docs/AGENT_OPERATING_MANUAL.md` は `scripts/ci/classify-changes.sh` の policy docs の列、S6 の `.claude/agents/*` は同じ file の実行制御の列（full の areas、`workflow=true` を含む。`printf '%s\n' .claude/agents/reviewer-fable5.md | bash scripts/ci/classify-changes.sh --files-from-stdin` が `rust=true`・`workflow=true`・`unknown=false`）に当たる。manual なし: 製品の runtime・画面・配布物を変えず、Windows native で確かめる物が無い。r4 なし: data・DB・破壊的な git 操作を含まず、revert で戻せる。

Final Reviewer の起動の注: 本 lane の Final Review の Claude 側は、座組表の Final Reviewer の行の「Fable が使えないときは §3.3 に従い理由を 1 行残して fresh Opus 5.5 で代えてよい」で、fresh Opus 5.5（`subagent_type: reviewer`）にする（Coordinator の裁定、2026-10-24）。残す理由: Fable 5 は model ID でしか選べず（公式 [model-config](https://code.claude.com/docs/en/model-config)「Work with Fable」）、その定義 `reviewer-fable5` は本 lane の merge まで本体の checkout に無く、本体の `.claude/agents` は sandbox から書けない（P-5）ので、本 lane の Final Review の時点では review 用の定義（`reviewer-fable5`）で Fable 5 を起動できない（§3.3 の一時的な利用不能で、merge で解ける。Agent tool の呼出しの `model` は alias の enum だけを受け、full model ID を受けない。P-8）。

遷移の記録:

1. kickoff → spec-check（2026-10-24、起草役）: 範囲を TD-410 と backlog の 3 つの項目（下の S2〜S5）にし、Risk を R2 と記録した（下の Risk）。
2. spec-check → plan-draft（2026-10-24、起草役、同じ commit）: 表の唯一の skip。直す先の設計正本（下の Design Readiness）は契約として足りていて、本 lane は新しい設計の選択をしない。S1 は owner の決定（TD-410）を座組表へ書き写し、S2〜S4 は文を既にある正本・`docs/project-memory.md` の店の事実に合わせるだけ。
3. plan-draft → plan-gate（2026-10-24、起草役、同じ commit）: owner の判断事項は Scope に残らない（owner の選択が要る候補は Scope に入れず、起票の報告で Coordinator へ渡した。下の Non-scope）。packet が揃い commit されている。R2 で Test Design Matrix は任意で、置かない。Plan Review に出す。Codex が使えれば Codex と fresh Opus 5.5 を同じ round で、使えなければ fresh Opus 5.5 で始め Codex 分を後から足す（`docs/AGENT_OPERATING_MANUAL.md` §3.3）。
4. plan-gate のまま round 2 へ（2026-10-24、起草役）: Plan Review round 1（対象 `d3007c38`）の是正を入れた（下の Review Response）。Coordinator の裁定で S6（定義 2 つの追加）と S7（71 §71.9）を足し、Risk を R3 に上げた（下の Risk）。R3 なので Test Design Matrix・Contract Ledger・Contract Probe・Data Safety を足し、Final Reviewer を座組表の「R3 以上」の行にした。spec-check → plan-draft の skip は変えない（S6 は D-099 の (D3) の形を写し、S7 は §71.9 の文を今の `lib.rs` に合わせるだけで、新しい設計の選択をしない）。
5. plan-gate のまま round 3 へ（2026-10-24、起草役）: Plan Review round 2（対象 `a7a66e1e`）の是正と、owner の決定 TD-429（S5-3）を入れた（下の Review Response）。Risk・Minimum・Human Gate は変えない。本 lane の Final Review の Claude 側を fresh Opus 5.5 にした（Workflow State の注）。round 3 は Plan Review の天井（下の Owner Effort Budget）。
6. plan-gate → plan-approved（2026-10-24、Coordinator、本 commit）: 条件は `docs/DEV_WORKFLOW.md` の遷移表（99 行）の「Writer でない独立の Plan Reviewer が P1 / P2 = 0」。Plan Review round 3（天井、対象 `57b01d3d`）の後、Coordinator の disposition「一括是正」で起草役が `f71d131c` で直し、fresh の reviewer の closure 2 本（Claude 側 fresh Opus 5.5 と Codex）が対象 `f71d131c` に P1 / P2 = 0 を報告した（下の Review Response）。closure の P3 3 件は plan-gate のまま起草役が `f628c918` で直した（Plan Commit はその版。前例: PR #201 の遷移 8）。docs の直しと定義の追加は、implementing へ進めてから別の run で行う。
7. plan-approved → implementing（2026-10-25、Coordinator、本 commit）: 遷移表の唯一の入口。直前に `origin/main`（`8db97271`）を単段で merge した（`62ed8ce2`、衝突なし）。AC の baseline を同期の後に逐語で再測し、値は起票時と同じで、行番号だけがずれた（`docs/SCREEN_DESIGN.md` 218 → 216、`docs/backlog.md` 172 → 176・109 → 113・193 → 197・180 → 184）。AC の期待は変わらない。
8. implementing のまま Gated Amendment（2026-10-25、Coordinator、本 commit）: 実装の run が S5-2 だけを直さずに返した。S5-2・AC-S5-2・Matrix の S5 の行が「35 の 2 つの設計判断の期間の句は PR #188 が外す（merge まで残りに数える）」を前提にしていたが、#188 は `22823ba5` で merge 済みで 2 句は既に無い。遷移 7 の再測は AC の baseline の値だけを見て、Scope の散文の前提の崩れを見落とした。相談役（Fable 5、反例探し）の所見を入れて、`975090bf` で 35 の句を残りに数えない形に直した（S4 の直さないものの 35 と module doc の 2 行、S5-2、Non-scope、durable な判断の (2)、延期した gap、Matrix の S5 の行）。AC-S5-2 は 2 つの見出しが backlog に出る検査を消し（反転すると baseline も期待も exit 1 で判別しない）、2 つめの検査に backlog に残る 35 の file:行（`35-biz-stocktake-service.md:374`）を足した。mockup の 2 か所は歴史資料の正本が決めているので項目に書かない。登録は本 commit で `975090bf` を `Amendments` に足す。review は Final Review broad に含める（前例: PR #172 の amendment 1）。
9. implementing のまま Gated Amendment 2（2026-10-26、Coordinator、本 commit）: Final Review broad（`40d8c611`）の Claude 側の P3（Ordinary Operation の 1 行目と Residual Test Gaps の effort の出所の導出が、呼出しで `effort` を渡した run の実効を見ない。公式 env-vars に実効の値を返す `CLAUDE_EFFORT` がある）を受理した。相談役（Fable 5、反例探し）の所見を入れ、導出の段（`CLAUDE_CODE_EFFORT_LEVEL` の値、空なら定義の `effort`）を残して前に段を足すのでなく、出所の順を「`/tasks` → `echo $CLAUDE_EFFORT` → 未実測」の 3 段に置き換えた（`echo $CLAUDE_CODE_EFFORT_LEVEL` を報告できる run は `echo $CLAUDE_EFFORT` も報告できるので導出の段は通らず、`CLAUDE_EFFORT` が空になるのは model が effort を受けないときで、そこを定義の `effort` で埋めると誤記になる）。P-7 と公式の資料の行を足し、P-6 の「導出もその上限で切る」の句を外した。座組表（MANUAL `## 座組`）は出所を書いていないので変えない。同じ commit で broad の Claude 側の P3 の 2 件目（`docs/backlog.md` の項目の見出しの「動線」）を直した。broad の後の packet の変更なので、本 lane の Final Review は新しい head の fresh broad（Minimum 2）を取り直す（`docs/DEV_WORKFLOW.md` Workflow State、helper の「broad Plan contract changed; fresh broad required」）。登録は次の commit で本 commit の SHA を `Amendments` に足す。

## Owner Effort Budget

- 介入回数上限: 6（既定）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 6 | 2（decision point の単位、`docs/DEV_WORKFLOW.md` Owner Effort Budget: TD-410〈Fable を当面 Fable 5〉1、TD-429〈backlog の「squash merge の subject 明示」の項目を外す〉1） | 3（Ready 1・merge 1・merge 後の本体の checkout の同期 1〈下の Ordinary Operation の 1 行目。本体の `.claude/agents` は sandbox から書けず、untracked の `consult-fable5.md` を先に外す。先例: PR #127 の G1〉） | 1（Ready の後の base 同期で再 Ready を諮る 1） | 6 = 2 + 3 + 1 |

承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
S6 が `.claude/agents/` に subagent の定義を 2 つ足す。`docs/decision-log.md` の D-099 の (D1) は「agent 定義は model・effort・tools・`hooks`・`permissionMode` を決める設定で、full にすると R0 / R1 の経路で拒まれ R3 の packet と Minimum 2 が要る」と決めていて、classifier はその path を実行制御（full）に置き（`scripts/ci/classify-changes.sh` の 1 つめの `case`）、packet の無い経路の helper は `CI execution change requires R3 packet` で止める（`scripts/pr-gate.py` の `require(not (flags['workflow'] == 'true' and flags['rust'] == 'true'), …)`）。packet の付いた経路の helper は R2 を機械的には拒まないが、D-099 の決定と、定義を tracked にした前例 PR #127 の packet の Risk（R3、`docs/archive/plans/2026-09-29-harness-pr5-gate-holes.md`）に合わせる。加えて、`reviewer-fable5` は merge の根拠になる Final Review（R3 以上・design lane・closure の Claude 側）の model・effort・tool・system prompt を決めるので、`docs/DEV_WORKFLOW.md` Risk Tiers の境の規則（「If uncertain between R2 and R3, choose R3 when the change touches … workflow gate」）にも当たる。

R3 にして変わるもの: Test Design Matrix・Contract Ledger・Contract Probe・Data Safety の節が要り（`docs/DEV_WORKFLOW.md` Plan Packet Rules）、Final Reviewer の Claude 側が座組表の「R3 以上」の行（Fable）になり、reviewer は Contract Audit を行う。変わらないもの: Final Review Minimum は R2 のときも `workflow=true` で 2（上の Workflow State の注）。Human Gate は `ready,merge`（manual・r4 の理由は上の注）。

R0 / R2 にしない理由: S1 は reviewer と相談役に充てる model と起動の仕方を変え、S2〜S4・S7 は文の意味を変えるので R0（「Typo, link, non-semantic doc cleanup.」）でない。round 1 までの R2 の判定は「`.claude/agents/*` を触らない」を前提にしていて、S6 でその前提が崩れた。

## Goal

Goal Invariant:

### 最小完了条件

- 座組表（`docs/AGENT_OPERATING_MANUAL.md` `## 座組`）を読んだ Coordinator が、Fable の枠（Final Reviewer の Claude 側の R3 以上・design lane・closure と、相談役）に Fable 5 を充て、tracked の定義（Final Reviewer は `reviewer-fable5`、相談役は `consult-fable5`。どちらも frontmatter が `model: claude-fable-5`・`effort: high`・編集の tool なし）で、呼出しで `model`・`effort` を渡さずに起動でき、その出所（TD-410）を読める。実効の model か effort が発注と違う run を Final Review の本数に数えない（S1・S6、round 2 の Codex F1）。
- Final Review の Claude 側を Fable 5 で起動したとき、その run が相談役の prompt でなく review 用の定義（`reviewer` と同じ本文: review packet に従い `docs/code_review.md` の形で報告する）で走る（S6、round 1 の Opus F1）。
- `docs/ARCHITECTURE.md` の `## 4. 未確定事項` に、決まった事項（IO-04 の PLU の形と反映の手順、宅急便部門）が残らず、その中身は今の正本（[67](../../function-design/67-ui-plu-export.md) UI-08-D1〜D3・[25](../../function-design/25-io-plu-formatter.md)・[master-tables](../../db-design/master-tables.md) の初期データの部門 12 の行）で辿れる（S2）。
- backlog の「ECR+ のサービス終了後の精算」の項目が、`docs/project-memory.md` の店の事実（2026-10 にレジ本体だけで精算できた）と合う前提と、残る論点だけを書く（S3）。
- 棚卸しの期間を「10月〜大晦日の長期作業」「数週間スパン」と書く 3 か所が、[73](../../function-design/73-ui-stocktake.md) §73.1 の店の事実（10 月からの準備と大晦日に終える正式カウントに分かれる、一人で数千点を回る）と合う（S4）。
- backlog の済んだ項目（ARCHITECTURE §4 の宅急便部門）と、owner が外すと決めた項目（squash merge の subject 明示、TD-429）が外れ、期間の旧表記の項目が残りだけを書く（S5、D-127）。
- [71](../../function-design/71-mnt-backup.md) §71.9 が、自動 backup の確認を起動のどこで呼ぶかを手順の番号でなく順序（DB の初期化と操作ログの自動削除の後、State 管理の前）と今の関数の名で書き、`src-tauri/src/lib.rs` の `run` の setup と合う（S7）。

### 失敗定義

- S1 で座組表の規則（枠の本数・独立性・Codex の合否・Fable が使えないときの代え方・effort の値）を変える。取り直しの規則は、S1-1 が実効の effort を条件に足すほかは変えない。Fable 5 の起動の仕方を、公式の資料と食い違う形で書く。
- S1 で `docs/AGENT_OPERATING_MANUAL.md` の外（decision-log の決定の本文、archive、並走の lane の packet）の「Fable 5.1」を書き換える。
- S6 の定義が、既存の `reviewer`・`writer` の定義や hook test の監査（`scripts/tests/claude-hooks.test.sh` の `validate_inventory`: frontmatter に `hooks`・`permissionMode`・`mcpServers` を持たない）と食い違う。`consult-fable5` を本体の checkout の今の file から理由なく変える。
- S2 で外した項の中身に、今の正本に無く今も効くものがあったのに、確かめずに消す。
- S3 で店の運用の細部・実値・発言の原文を repo に書く。
- S4 で棚卸しの期間・作業量について、73 §73.1 と `docs/project-memory.md` に無い事実（例: 正式カウントの日数）を書く。
- S7 で起動の順序を今の `lib.rs` と違えて書く。`lib.rs` を変える。
- 並走の lane の行（下の「他の lane との所有」）を触る。

### 非目的

- code・migration・bindings・route・画面の文言を変える（`src-tauri/src/biz/stocktake_service.rs` の module doc の comment と `src-tauri/src/lib.rs` の setup の comment を含む）。
- 座組表の他の枠（Writer・Plan Reviewer・Codex の model）と effort の方針を見直す（Writer の行は、S1-6 が再起動の句を設定と定義に分けるほかは変えない）。
- `.claude/agents/` の既存の定義（`reviewer`・`writer`）を変える。
- backlog の他の項目を洗い直す。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

workflow の変更（S1・S6 は review と相談役の起動の仕方を変える）なので、発注 → 停止と訂正 → review → owner 判断の通常列を置く（`docs/templates/plan-packet.md` の同節）。S2〜S5・S7 は文の同期で、operator の操作・data / command の契約を変えない。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 本 lane が main に入り、owner が本体の checkout（`/home/kosei/projects/inventory-system-public`）を sandbox の外で `origin/main` に同期した。同期の前に untracked の `.claude/agents/consult-fable5.md` を外す（git は同じ名の untracked の file があると merge を止める。P-4）。同期の後、定義が在るのは本体の checkout が main か本 lane の merge を含む branch にある間だけで、merge より前に分かれた branch を本体で checkout している間は `reviewer-fable5`・`consult-fable5` が見つからない（起動の失敗として見え、`reviewer-fable5` は座組表の Final Reviewer の行の §3.3 の代え方へ、`consult-fable5` は相談役の行の代え方〈省くか fresh Opus 5.5 で代え、遷移を止めない〉へ落ちる） | Coordinator が R3 以上・design lane・closure の Final Review の Claude 側を、Agent tool の `subagent_type: reviewer-fable5` で発注する（呼出しで `model`・`effort` を渡さない） | subagent が Fable 5・effort high・編集の tool なしで、review packet に従い `docs/code_review.md` の形で返す | Coordinator が実効の model と effort を run 報告に記録する。model の出所は `/tasks`、取れなければ run の自己申告（system prompt の model 名）、どちらも無ければ未実測と書く。effort の出所は `/tasks`、取れなければ run が報告する `echo $CLAUDE_EFFORT`（その subprocess の開始の時点の実効の effort。P-7）、どちらも無ければ未実測と書く（`CLAUDE_EFFORT` は model が effort を受けるときだけ立つので、空のときに定義の `effort` で埋めない）。実効の model と effort が発注（Fable 5・high）と同じなら Final Review の本数に数える（環境変数が定義と同じ値を立てていても、実効が同じなら数える）。未実測の項は未実測と書いて数え、違いが分かった run だけを 2 行目で取り直す（座組表の S1-1 の規則） | P-1（実効の model・effort の `/tasks` の実測は未取得。版の前提）、P-3（既存の `.claude/agents/` に足した定義は、公式の 3 つの例外に当たらなければ再起動なしで効く）、P-6（環境変数は定義の `effort` より優先される）、P-7（`CLAUDE_EFFORT` は実効の effort を返す） |
| 1 行目と同じ。実効の model か effort が発注と違う（宣言なしの切替え、呼出しで `model`・`effort` を渡した、`CLAUDE_CODE_EFFORT_LEVEL` が定義の `effort` を違う値で上書きした等） | Coordinator が run 報告の実効の model・effort と環境変数を見る | その run は Final Review に数えない | 座組表の Final Reviewer の行の規則（S1-1 の後: 発注と実効の model か effort が違う run は数えず取り直す）どおり取り直す。定義の `model`・`effort`、呼出し、環境変数を確かめ、Fable が使えなければ §3.3 に従い理由を 1 行残して fresh Opus 5.5 で代える | P-2（alias の `fable` は既定で Fable 5.1）、P-6 |
| 1 行目と同じ。Plan Review・Final Review の P1/P2 の差し戻しか Gated Amendment を書く前 | Coordinator が `subagent_type: consult-fable5` で反例探しを頼む（呼出しで `model`・`effort` を渡さない） | Fable 5 が読むだけで助言を返し、Coordinator が是正を書く | 実効の model・effort を 1 行目と同じ出所の順で記録する。相談の run は closure に数えず、closure は fresh context で行う（座組表の既存の規則） | P-1 |
| Final Review の本数が揃い、finding の裁定が済んだ | Coordinator が owner に Ready を諮り、helper `ready` の後 merge を諮る | owner が Ready・merge を判断する | `docs/DEV_WORKFLOW.md` Workflow State の順（helper `ready` → hosted CI 成功 → owner の merge 指示 → helper `merge`） | なし |
| 本 lane 自身の Final Review。本体の checkout に `reviewer-fable5` がまだ無い（本体の `.claude/agents` は sandbox から書けない。P-5） | Coordinator が Claude 側を fresh Opus 5.5（`subagent_type: reviewer`）で発注し、上の Workflow State の注の理由を run 報告に 1 行残す | fresh Opus 5.5 の review | 実効の model・effort を記録する（座組表の Final Reviewer の行の Opus の値 medium と比べる） | なし（Coordinator の裁定） |

この列で、S1・S6 の後に Coordinator が座組表と tracked の定義だけを読んで Fable 5 の枠を起動でき、model・effort の食い違いを run 報告で捕まえて取り直せる。

## Scope

段ごとの対象と直し方。各段の表の「結果」列は実装の run が「直した」か「残す（理由）」で埋める（AC-S0-1）。直す前に、主張の根拠（正本・`docs/project-memory.md`・公式の資料）を `rg` か実読で確かめ、確かめられなければ直さずに「残す（未確認）」とする。新しく書く文の引用は D-125・`docs/DOC_STYLE_GUIDE.md` §1 の形（path と symbol、節の ID か見出し）。新しい文言に作業の印の語（AC-S9-1 の command が探す 7 語）を使わない。

### S1 座組表の Fable を当面 Fable 5 にする（TD-410）

規則: 座組表の注記「モデル更改時は owner 決定を受けてこの表だけを書き換える。座組・effort を他の文書へ複製しない」。`docs/AGENT_OPERATING_MANUAL.md` の `## 座組` の節（表と直後の箇条）の中だけを直す。

公式の資料（2026-10-24 に WebFetch で読んだ）:

- [sub-agents](https://code.claude.com/docs/en/sub-agents)「Choose a model」: frontmatter の `model` は alias か full model ID を受ける。順は「The per-invocation `model` parameter → The subagent definition's `model` frontmatter → `CLAUDE_CODE_SUBAGENT_MODEL` → The main conversation's model」で、呼出しごとの `model` は定義の frontmatter より優先される。呼出しの `model` が受ける値（alias か full model ID か）は書かれていない（Agent tool の schema は P-8）。この順は版で変わった: 「Before v2.1.251, `CLAUDE_CODE_SUBAGENT_MODEL` came first in this order and overrode both the per-invocation parameter and the frontmatter」。同じ page の「Run every subagent on one model」: `CLAUDE_CODE_SUBAGENT_MODEL_FORCE` を立てると「Claude Code ignores the `model` field in subagent definitions」。
- 同じ page の「Choose an effort level」: 「The parameter overrides the `effort` field … The `CLAUDE_CODE_EFFORT_LEVEL` environment variable takes precedence over both.」（呼出しの `effort` は定義の `effort` より、環境変数は両方より優先される）。
- 同じ page の「Write subagent files」: 「When you add or edit a subagent file on disk, … Claude Code detects the change within a few seconds and the next delegation uses the updated definition, with no restart needed.」。続けて「Three cases still need a restart」として、session の開始時に無かった `agents` の dir に最初の定義を置いたとき（「The watcher covers only directories that existed when the session started」）、`--add-dir`・`/add-dir` で足した dir の中の `.claude/agents/` の定義を足すか直したとき、session を `--disable-slash-commands` で始めたとき（「don't watch these directories at all」）の 3 つを挙げる。
- [env-vars](https://code.claude.com/docs/en/env-vars)「Set variables in settings files」: 「A running session applies new and changed values to its environment when you save the file, but a feature that reads its variables once at startup … keeps its startup values until you relaunch.」「Removing a variable from the file doesn't unset it in a running session」。`CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH` がどちらに当たるかは同じ page の変数の行に書かれていない（S1-6）。
- 同じ page の `CLAUDE_EFFORT` の行（2026-10-26 に WebFetch）: 「Set automatically in Bash tool subprocesses and hook commands to the effort level in effect when the subprocess starts: `low`, `medium`, `high`, `xhigh`, or `max`. … Only set when the current model supports the effort parameter」（Ordinary Operation の 1 行目の effort の出所。P-7）。
- [model-config](https://code.claude.com/docs/en/model-config)「Model aliases」: 「Unless you set `ANTHROPIC_DEFAULT_FABLE_MODEL`, the `fable` alias resolves to Fable 5.1, except in Claude apps gateway sessions」。同じ page の「Work with Fable」: 「Fable 5: select it by model ID」（`claude-fable-5`）。
- [advisor](https://code.claude.com/docs/en/advisor)「Choose an advisor model」: main が Opus 5 / Opus 5.5 なら advisor に Fable を受け、full model ID も渡せる。
- [Fable 5 prompting guide](https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/prompting-claude-fable-5)「Consider all effort levels」: 多くの作業の既定を `high` にする。

| # | 場所（`843c4938` の行） | 今の句 | 直す向き | 結果 |
|---|---|---|---|---|
| S1-1 | Final Reviewer の行の「Codex 稼働時の担当」の列（72） | 「Claude 側は R3 以上と design lane で Fable 5.1」「closure（base 同期を含む）の Claude 側の既定は Fable 5.1（owner 2026-09-25）」 | 2 か所の「Fable 5.1」を「Fable 5」にする。owner の日付（2026-09-25）は枠の決定の出所として残す。取り直しの句「発注と実効 model が違う（宣言なしに切り替わった）run は数えず取り直す」を、次のとおり逐語にする（AC-S1-5 が数える。round 2 の Codex F1、Coordinator の裁定）: 「発注と実効の model か effort が違う（宣言なしに切り替わった、環境変数 `CLAUDE_CODE_EFFORT_LEVEL` が定義の effort を違う値で上書きした等）run は数えず取り直す」。ほかの句（Fable が使えないときの代え方、Codex の本務）は変えない |  直した |
| S1-2 | 同じ行の effort の列（72） | 「Fable 5.1 = high（owner 2026-09-25。定義を使わず session の値を継承）」 | 「Fable 5 = high（owner 2026-09-25。`subagent_type: reviewer-fable5`、`.claude/agents/reviewer-fable5.md`、effort high、編集の tool なし。起動の注意は相談役の行と同じ）」の向き（Plan Reviewer の行の Opus の句と同じ形）。「定義を使わず session の値を継承」は定義で起動する形と合わないので外す |  直した |
| S1-3 | 相談役の行の担当の列（73） | 「Fable 5.1。」と、advisor の句「使う場合は main の Opus 5.5 に Fable 5.1 を組み」 | 頭を「Fable 5（当面。owner 2026-10-24、repo 外の回答台帳 TD-410）。」にし、起動の句を足す。起動の句は次のとおり逐語（AC-S1-1・AC-S1-2 が数える）: 「起動: `subagent_type: consult-fable5`（`.claude/agents/consult-fable5.md`、frontmatter の `model: claude-fable-5`・`effort: high`、編集の tool なし）で起動し、Agent tool の呼出しで `model`・`effort` を渡さない（呼出しの値は定義の frontmatter より優先され、環境変数 `CLAUDE_CODE_EFFORT_LEVEL` は定義の `effort` より優先される。公式 [sub-agents](https://code.claude.com/docs/en/sub-agents)「Choose a model」「Choose an effort level」。alias の `fable` は既定で Fable 5.1 になる〈`ANTHROPIC_DEFAULT_FABLE_MODEL` と Claude apps gateway の session で変わる〉。公式 [model-config](https://code.claude.com/docs/en/model-config)「Model aliases」）。定義は本体の checkout に在れば次の委譲から効き、再起動は要らない。ただし、session の開始時に `.claude/agents/` が無かった、定義が `--add-dir`・`/add-dir` で足した dir の中にある、session を `--disable-slash-commands` で始めた、のどれかに当たるときは Claude Code を再起動してから効く（公式 sub-agents「Write subagent files」）。実効の model と effort を run 報告に記録する。」。advisor の句は次のとおり逐語: 「使う場合は main の Opus 5.5 に Fable 5（full model ID `claude-fable-5`。alias の `fable` は既定で Fable 5.1 になる）を組み」 |  直した |
| S1-4 | 表の後の「確認日」の箇条（76） | 「確認日: 2026-09-30。担当と effort は owner 決定 2026-09-07 / 08・14・23・24・25・30 による。」 | 確認日を 2026-10-24 にし、owner 決定の列に「2026-10-24（Fable を当面 Fable 5 に。repo 外の回答台帳 TD-410）」を足す |  直した |
| S1-5 | 「根拠」の箇条（80） | 「Fable 5.1 prompting guide への link（URL の末尾 `prompting-claude-fable-5-1`）」 | 「[Fable 5 prompting guide](https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/prompting-claude-fable-5)「Consider all effort levels」」にする |  直した |
| S1-6 | Writer の行の effort の列（70） | 「定義と設定は本体の checkout を同期し Claude Code を再起動してから効く（公式 [sub-agents](https://code.claude.com/docs/en/sub-agents)「Write subagent files」・「Supported frontmatter fields」の `effort`）」 | 設定と定義を分け、次のとおり逐語（AC-S1-6 が数える。round 3 の Opus #3、Coordinator の裁定）: 「設定（`.claude/settings.json`）は本体の checkout を同期し、Claude Code を再起動してから頼る（公式 [env-vars](https://code.claude.com/docs/en/env-vars)「Set variables in settings files」は、保存で走行中の session に効く値と、起動時にだけ読む機能・file から外した変数を分け、`CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH` がどちらかは書いていない）。定義は相談役の行の起動の句と同じ条件で効く（公式 [sub-agents](https://code.claude.com/docs/en/sub-agents)「Write subagent files」・「Supported frontmatter fields」の `effort`）」。Plan Reviewer の行の「起動と実効の注意は Writer 行と同じ」は変えない（Writer の行を引くので、直した後も 1 通りに読める） |  直した |

直さない: 「Fable は context を絞った subagent の発注で使い、advisor は常用しない」の箇条（79）は版を書かない。decision-log の D-092 の Decision と Why の「Fable 5.1」は決定の本文（append-only、D-126）。backlog と archive の「Fable 5.1」は誰が review したかの記録。並走の lane の packet の `Final Reviewer` の値（下の「他の lane との所有」）。

sweep（`843c4938` で `rg -n --hidden 'Fable 5\.1' --glob '!docs/archive/**' --glob '!node_modules' --glob '!.git' .`）: 本 packet の外では `docs/AGENT_OPERATING_MANUAL.md` の 72・73・80、`docs/decision-log.md` の 781・782（D-092 の本文）、`docs/backlog.md` の 174・178・197・198・204（review の記録）を出す。座組の値として「Fable 5.1」を書くのは `docs/AGENT_OPERATING_MANUAL.md` の 3 行（`rg -o 'Fable 5\.1' docs/AGENT_OPERATING_MANUAL.md | wc -l` が `6`）だけ。hidden の dir（`rg -n --hidden -i 'fable' .claude .agents .github .codex` は何も出さない）・`AGENTS.md`・`CLAUDE.md`・`docs/templates/**`・`scripts/**` に「Fable 5.1」は無い（`scripts/tests/doc-consistency-plan-packet.test.sh` の `Fable 5（fixture coordinator）` は PK4 の fixture の文字列で、座組の値でない）。tracked でない `.local/codex-orders/templates/subagent-review.md`・`codex-review.md` は「Fable 5.1」を書くが、本 lane の外（起票の報告で Coordinator へ渡した）。

### S2 ARCHITECTURE §4 の決まった 2 項を外す（backlog 109 行の項目）

出所: backlog の「`docs/ARCHITECTURE.md` §4 の宅急便部門の項を整理する」（PR #195 の Plan Review round 1 の P3 F4）。同じ項目が「同じ節の IO-04 の項も決まった中身で、一緒に扱うかは着手のときに決める」と書くので、本 lane で一緒に扱うと決める（理由: どちらも中身が決まり、今の正本が持つ。項を残すと、§4 の節が決まった事項をまだ決まっていない事項と読ませる。D-127）。

| # | 場所（`843c4938` の行） | 今の句 | 直す向き | 中身の置き場（確かめた所） | 結果 |
|---|---|---|---|---|---|
| S2-1 | `## 4. 未確定事項` の IO-04 の項（299） | 「IO-04（PLUフォーマッター）: 2026-07-03 field gate により…`.txt`、11列、13桁JAN必須、通常PLU216枠使用時217始まりの profile と…一連フローを確認済み。アプリはPCツール受理やレジ反映を自動検知しないため、保存後のapp-side confirmationは利用者の明示操作に分離する。」 | 項を外す | profile と手順は [67](../../function-design/67-ui-plu-export.md) UI-08-D3（11 列・217 始まり・13 桁 JAN 必須・field gate の手順）と [25](../../function-design/25-io-plu-formatter.md)。保存後の確認を利用者の明示操作に分けることは 67 の関数要求（「利用者が保存済み扱いを明示した後にだけ `confirm_plu_export_saved` を呼ぶ」）と UI-08-D1・UI-08-D2。同じ file の §2 のタスク一覧の IO-04 の行も profile を書く |  直した |
| S2-2 | 同じ節の宅急便部門の項（300） | 「宅急便部門: ~~在庫管理対象かどうか利用者返答待ち~~ → 在庫管理対象外と確定（2026-04-10）。サービス取次であり追跡すべき在庫が存在しない。departmentsにはZ005対応で残すがcode_prefix=NULL」 | 項を外す | [master-tables](../../db-design/master-tables.md) の「初期データ（全21部門、C-1/C-3 2026-03-29 確定）」の部門 12 の行（`NULL`、「在庫管理対象外。サービス取次（ヤマト/佐川）。Z005に載るがシステムでは追跡しない」）。code の初期データも PR #195 で `NULL` |  直した |

節の見出しと Q40 の項は残す（Q40 は backlog の項目を引く、まだ開いた事項）。§4 を引く文は、`843c4938` で `rg -n 'ARCHITECTURE[^ ]*( |` )?§ ?4|ARCHITECTURE.md#4' docs --glob '!docs/archive/**'` が本 packet の外に出す `docs/backlog.md` の 109 と 324 の 2 行。109 は S5-1 で外す。324（「包括的な障害時対応方針としての残余は `docs/ARCHITECTURE.md` §4と…を参照」）は残る Q40 の項を引くので、S2 の後も切れず、直さない。

### S3 backlog の ECR+ の精算の項目の前提を店の事実に合わせる（backlog 172 行の項目）

出所: 同じ項目の「「精算に ECR+ が要る」とする文は、project-memory への link の 2026-10 の観測（スマホを切ってレジ本体だけで精算できた日がある）と合わない。次の docs の lane で直す（2026-10-20）」。食い違う文は、この項目自身の「店主は精算に ECR+ を要すると答えている（project-memory への link Store Premises Facts）ため、ECR+ の終了後に精算を続けられるかを確かめる」（`843c4938` で `rg -n 'ECR\+.{0,40}精算|精算.{0,40}ECR\+' --glob '!docs/archive/**' .` が出す本 packet の外の行は、`docs/project-memory.md` の 2 行と本項目と、`docs/plu-export-and-real-csv-verification.md` の APP-01〈調査項目で、精算に ECR+ が要るとは書かない〉の 4 行。project-memory の 2 行は 2026-10 の観測を既に書いている）。

| # | 今の句 | 直す向き | 根拠 | 結果 |
|---|---|---|---|---|
| S3-1 | 「店主は精算に ECR+ を要すると答えている（project-memory への link Store Premises Facts）ため、ECR+ の終了後に精算を続けられるかを確かめる。」 | レジ本体だけで精算できることは 2026-10 に確かめた（店主の 2026-03 の答えと違った）と書き、残る論点を「ECR+ の終了の後に、送信の失敗の印字と SD の控えがどう積もるか」にする。どちらも project-memory を引き、回数・日付の細部は写さない | `docs/project-memory.md` の Store Premises の ECR+ の行（2026-03 の店主の答えと、2026-10 のレジ本体だけの精算）と、同じ file の「一方、店主は精算に ECR+ を要すると答えていたが…」の行（終了の後の積もり方は未確認で、backlog のこの項目に置く） |  直した |
| S3-2 | 「「精算に ECR+ が要る」とする文は、…次の docs の lane で直す（2026-10-20）。」 | 外す（S3-1 で直したので役目が終わる。D-127） | — |  直した |

項目の名（「ECR+ のサービス終了後の精算」）、CASIO の公式ページの句、ジャーナル閲覧の代替の句、「導入の後でよいと確かめた」の句は変えない（project-memory が「`docs/backlog.md`（ECR+ のサービス終了の項）」と名で引く）。

### S4 棚卸しの期間の旧表記を 73 §73.1 に合わせる（backlog 193 行の項目）

出所: backlog の「期間・動線の旧表記の残り 7 か所を店の事実に合わせる」（PR #93 の Non-scope と Final Review broad の申し送り）。店の事実の正本は [73](../../function-design/73-ui-stocktake.md) §73.1（「一人で商品マスタ全件（数千点規模）を回り、10月からの準備（…）と、年末の正式カウント（大晦日に完了）に分かれる」）と `docs/SCREEN_DESIGN.md` の `### 利用者の1日の動線`。

| # | 場所（`843c4938` の行） | 今の句 | 直す向き | 結果 |
|---|---|---|---|---|
| S4-1 | `docs/architecture/biz-task-specs.md` BIZ-06 の「理由」（453） | 「年末棚卸し（10月〜大晦日の長期作業）で、」 | 「年末棚卸し（10 月からの準備と、大晦日に終える正式カウントに分かれる。[73](../../function-design/73-ui-stocktake.md) §73.1）で、」の向き |  直した |
| S4-2 | `docs/db-design/tracking-system-tables.md` の stocktakes + stocktake_items の「役割」（133） | 「年末棚卸しの記録。10月〜大晦日の長期作業に対応する中断・再開機能付き。」 | 「年末棚卸しの記録。一人で数千点の商品を回る作業（10 月からの準備と、大晦日に終える正式カウント。[73](../../function-design/73-ui-stocktake.md) §73.1）に対応する中断・再開機能付き。」の向き |  直した |
| S4-3 | `docs/SCREEN_DESIGN.md` 棚卸しの画面の「利用者配慮」（218） | 「一人運用・数週間スパンの反復作業を前提に、」 | 「一人で数千点の商品を回る反復作業（73 への link〈`function-design/73-ui-stocktake.md`〉 §73.1）を前提に、」の向き（link は SCREEN_DESIGN からの相対 path） |  直した |

旧表記の全数（`843c4938` で `rg -n -e '10月〜大晦日' -e '数週間スパン' -e '長期作業' docs src-tauri src --glob '!docs/archive/**' --glob '!docs/plans/**'`）は、上の 3 か所と backlog の項目そのもの（193）のほかに、下の 4 行を出す。backlog の項目の 7 か所のうち本 lane で直さないもの:

- `docs/function-design/35-biz-stocktake-service.md` の force_fill の設計判断の「年末棚卸しは10月〜大晦日の長期作業。…」（377）と、評価原価の「理由: 棚卸しは数ヶ月に及ぶ長期作業であり、…」（382）: PR #188 が 2 行とも外した（`22823ba5` で merge 済み。本 branch は遷移 7 で取り込んだ）。本 lane は 35 を触らず、backlog の残りに数えない（Gated Amendment 1）。
- `src-tauri/src/biz/stocktake_service.rs` の module doc の「年末の長期作業（10月〜大晦日）で、…」（4）: code の file で、本 lane の外。PR #188 は同じ module doc の隣の行（3 行目と 6 行目以降）を書き換えて merge され、この句は残った。code の file を触る後の lane で直す（S5 で backlog の項目に残す）。
- `docs/screen_mockups.html` の図の「10月〜大晦日」（208）とボタン説明「閉店後にレジのCSVを読み込む」（259）: 初期提案の歴史資料で、今の画面の図でない（`docs/SCREEN_DESIGN.md` の頭の「このファイルの目的」、`docs/architecture/ui-task-specs.md` の頭、[52](../../function-design/52-ui-shared-layout.md) の「モックアップアーカイブ扱い」「触らない」の行）。当時の提案の記録として残す。

### S5 backlog の項目を直す

| # | 項目（`843c4938` の行） | 直す向き | 結果 |
|---|---|---|---|
| S5-1 | 「`docs/ARCHITECTURE.md` §4 の宅急便部門の項を整理する」（109） | 項目を外す（S2 で済む。残りが無い） |  直した |
| S5-2 | 「期間・動線の旧表記の残り 7 か所を店の事実に合わせる」（193） | 残りだけを書く項目にする: `src-tauri/src/biz/stocktake_service.rs` の module doc の期間の句（code comment 1 行。code の file を触る後の lane で直す）を書く。35 の 2 句は PR #188 が外し（merge 済み）、mockup の 2 か所は歴史資料として残すと正本（上の S4 の直さないものの mockup の行が引く節）が決めているので、どちらも項目に書かない。本 lane が直した 3 か所の名と、PR #93 の起源の句の長い列挙と、35 の file:行（`a3f08cdd` の時点の 374 行）は外す（本 packet に残る）。見出しから数を外す（例: 「期間・動線の旧表記の残りを店の事実に合わせる」。数を写すと次の直しでまた古くなる） | 直した（Gated Amendment 1 の後の契約で、2026-10-25、Coordinator。残りを module doc の 1 行だけにし、35 の file:行と本 lane が直した 3 か所の列挙と mockup の文を外し、見出しから数を外した） |
| S5-3 | 「squash merge の subject 明示」（180） | 項目を外す（owner の決定 2026-10-24、repo 外の回答台帳 TD-429）。理由: 今の merge は helper だけを通り（`scripts/pr-gate.py` の `merge` が `gh pr merge --squash --match-head-commit` を呼び、`--subject` を渡さない）、squash の subject は PR title〈commit が 1 つの PR では commit の subject〉になる（`gh api repos/kosei-w90607/inventory-system-desktop --jq .squash_merge_commit_title` → `COMMIT_OR_PR_TITLE`、2026-10-24）。正本に PR title の規則は無い（`docs/DEV_WORKFLOW.md` Commit / PR Messages は commit の subject の形だけを決め、`.github/pull_request_template.md`・`docs/agent-guidance/merge-evidence.md`・`docs/ci.md` にも無い）。観測として、`843c4938` から遡る 30 commit の subject は、`git log --format='%h %s' -30 843c4938` を Commit / PR Messages の type の形で数えて外れが 0 |  直した |

S3 は項目そのものを直す段なので、S5 に入れない。

### S6 Fable 5 の起動の定義を tracked にする（round 1 の Opus F1、Coordinator の裁定）

前例: PR #127 が `.claude/agents/{writer,reviewer}.md` を tracked にした（D-099 の (D3)、`docs/archive/plans/2026-09-29-harness-pr5-gate-holes.md` の S10）。`.gitignore` は `.claude/agents` を除外しない（`git check-ignore -v .claude/agents/reviewer-fable5.md .claude/agents/consult-fable5.md` が exit 1）。本 lane の worktree の `.claude/agents` は書ける（2026-10-24、`test -w` が真）ので、#127 の G1 の (a)（owner の書込みの代行）は要らない。

| # | file | 中身 | 結果 |
|---|---|---|---|
| S6-1 | `.claude/agents/reviewer-fable5.md`（新設） | frontmatter は `---`、`name: reviewer-fable5`、`description: Review-only reviewer on Fable 5 for Final Review and closure (the Claude-side slot the 座組表 assigns to Fable), in a context separate from the Writer.`、`model: claude-fable-5`、`effort: high`、`disallowedTools: Edit, Write, NotebookEdit`、`---` の順。本文は `.claude/agents/reviewer.md` の本文（frontmatter の後の 2 行: review packet〈`docs/templates/subagent-review-packet.md`〉に従うことと、`docs/code_review.md` の出力の形）と同じ。`reviewer.md` との違いは `name`・`description`（Plan Review を含めない。Fable の枠は Final Review と closure）・`model`・`effort` の 4 行だけ |  直した |
| S6-2 | `.claude/agents/consult-fable5.md`（新設） | 本体の checkout の untracked の `/home/kosei/projects/inventory-system-public/.claude/agents/consult-fable5.md`（2026-10-23 に置かれた。`sha256sum` が `92e309d2b6d81c54ad4baa941203f8020f98fcf3c45c2143e836265924144dfd`）を変えずに写す。frontmatter は `name: consult-fable5`・`model: claude-fable-5`・`effort: high`・`disallowedTools: Edit, Write, NotebookEdit`、本文は読むだけの相談役（tracked の file を編集しない・commit / push / PR の操作をしない、渡された原資料から答え、言えることと言えないことを分ける、依頼の形で報告する）。変える理由は見当たらない（座組表の相談役の行と、Fable 5・effort high・編集の tool なしで合う） |  直した |

S6 の後、hook test の監査（`scripts/tests/claude-hooks.test.sh` の `validate_inventory` が `.claude/agents/*.md` の frontmatter に `hooks`・`permissionMode`・`mcpServers` が無いことを求める）を 2 つの定義が通る。classifier の分類と helper は変えない。

### S7 71 §71.9 の起動の位置を順序で書く（Coordinator の追加）

現物（`843c4938` の `src-tauri/src/lib.rs` の `run` の setup）: `// 3-4. reconcile → legacy migration → DB初期化` の `prepare_database`（1461）→ `// 5. 操作ログ自動削除`（1475、`mnt::log_manager::cleanup_old_logs`）→ `// 6. 自動バックアップチェック`（1480、`run_startup_auto_backup(&conn, &app_data)`）→ `// 7. State管理`（1483、`app.manage(AppState { … })`）。`resolve_backup_dir` → `check_auto_backup` の match は同じ file の `fn run_startup_auto_backup`（125）の本体。

| # | 場所（`843c4938` の 71 の行） | 今の句 | 直す向き | 結果 |
|---|---|---|---|---|
| S7-1 | §71.9 の「追加箇所」（450） | 「MNT-02 操作ログ自動削除（ステップ6）の後、State管理（ステップ8）の前」 | 手順の番号を書かず、順序と関数の名で書く: 「`src-tauri/src/lib.rs` の `run` の setup で、DB の初期化（`prepare_database`）と操作ログの自動削除（`mnt::log_manager::cleanup_old_logs`）の後、State 管理（`app.manage`）の前に `run_startup_auto_backup` を呼ぶ」の向き |  直した |
| S7-2 | §71.9 の code block の 1 行目（453） | `// 7. 自動バックアップチェック（起動時）` | 手順の番号を外し、続く match が `run_startup_auto_backup` の本体であることを書く向き（例: `// 自動バックアップチェック（起動時。lib.rs の run_startup_auto_backup の本体）`）。match の中身（`resolve_backup_dir` → `check_auto_backup`、設定読取の DB error でスキップ）は今の関数と同じなので変えない |  直した |

番号を外す理由: setup の手順の番号は手順の足し引きで動き（§71.9 は「ステップ6」「ステップ8」、今の `lib.rs` は 5・7）、順序と関数の名は動きにくい。同じ型の [72](../../function-design/72-mnt-log-manager.md) §72.5 の「追加箇所」（「DB初期化（ステップ5）の後、State管理（ステップ8）の前」）は並走の PR #198 が直す（`origin/agent/oplog-retention` の 72 §72.5 は今の手順 5 と `prepare_database`・`run_startup_auto_backup` で書く）ので、本 lane は触らない。兄弟の 72 §72.5 は #198 で番号つき（「手順 5」「手順 3-4」「手順 6」）に書き直されていて、本 lane の 71 §71.9 と書き方が分かれる。両方の merge の後に揃えるかは closeout で見る。`843c4938` で `rg -n 'State管理（ステップ|（ステップ[0-9]+）の(後|前)' docs --glob '!docs/archive/**' --glob '!docs/plans/**'` は 71 の 450 と 72 の 66 の 2 行だけを出す。

### 他の lane との所有

並走の Draft の PR（`843c4938` の時点で #172・#188・#190・#193・#194・#196・#197・#198・#201・#202）と予定の path の重なりを `comm -12` で見た（`gh pr diff <N> --name-only`）。重なりは下の 6 file。どの PR の差分も本 lane の対象の行を `+`/`-` に持たない（`gh pr diff` の差分を対象の句〈`10月〜大晦日`・`数週間スパン`・`ECR+ のサービス終了`・`宅急便部門`・`IO-04（PLUフォーマッター）`・`§4 の宅急便`・`期間・動線の旧表記`〉で `rg` して、#188 の 35 の行の外は何も出ない）。

| file | 重なる PR（head） | 本 lane の行と並走の hunk |
|---|---|---|
| `docs/ARCHITECTURE.md` | #172（`8fd5b875`）・#188（`c667d52c`）・#198（`16d7dd00`） | 並走の hunk は頭・§2 の表・§3 の依存。§4 に hunk は無い |
| `docs/SCREEN_DESIGN.md` | #172・#188・#198 | #188 が棚卸しの画面の「現行 build では一時停止中」の 1 行を外し、本 lane の S4-3 の行とは 1 行離れる（間に「利用者配慮」の見出しの行）。#172・#198 の hunk は離れた節 |
| `docs/architecture/biz-task-specs.md` | #172・#188 | #188 は頭の 8 行、#172 は BIZ-08 の辺り。BIZ-06 の「理由」に hunk は無い |
| `docs/db-design/tracking-system-tables.md` | #188・#198 | #188 の hunk は頭・29 行の辺り・140 行以降、#198 は 225 行の辺り。133 行の「役割」に hunk は無い |
| `docs/backlog.md` | #172・#190・#196・#198 | 本 lane の 4 つの項目（109・172・180・193）の行は、どの PR の差分にも無い |
| `docs/function-design/71-mnt-backup.md` | #190（`000345f1`）・#194（`f508537f`）（round 1 の是正の時点、2026-10-24） | #190 の hunk は 71〜78 行・700 行・857 行の辺り、#194 は 552 行以降。§71.9（448〜465 行）に hunk は無い |
| `docs/AGENT_OPERATING_MANUAL.md`・`.claude/agents/*` | なし | — |

並走の lane の packet（#172・#188・#190・#196・#198）の `Final Reviewer` の行は「Fable 5.1」を値に持つ。本 lane は触らない（各 lane の packet の値で、所有は各 lane。TD-410 を各 lane に写すかは Coordinator が決める）。同じ file を触る lane は、後に merge する方が `origin/main` の単段 merge で同期する（`docs/DEV_WORKFLOW.md` Wave Operation）。

## Non-scope

- code・migration・bindings・route・画面の文言（`src-tauri/src/biz/stocktake_service.rs` の module doc を含む。S5-2 で backlog に残す。`src-tauri/src/lib.rs` の setup の comment の番号も変えない: 起動の test が `// 5. 操作ログ自動削除` を文字列で探す〈`lib.rs` の `.find("// 5. 操作ログ自動削除")`〉）。
- 35 の force_fill と評価原価の句（PR #188 が外した。merge 済み）と `docs/screen_mockups.html`（歴史資料）。
- 72 §72.5 の「追加箇所」（PR #198 が直す）。
- `.claude/agents/` の既存の定義（`reviewer`・`writer`）と、classifier・helper・hook test の変更。
- 座組表の外の「Fable 5.1」（decision-log の決定の本文、backlog と archive の記録、並走の lane の packet、tracked でない `.local/` の雛形、owner の個人の `~/.claude/CLAUDE.md`）。
- backlog の「単位の拡張の残りの申し送り」の子の「S7 の対象の行の外に残る古い型・SQL・表示例」（「小口のまとめで行う」）: docs の小口でなく、4 つの function-design と code の照合が要る。別の lane で扱う。
- backlog の「`docs/Plans.md` の `### Wave Registry` が見出しと形式の行だけになる」: closeout の手順（`docs/DEV_WORKFLOW.md` Post-Merge Closeout）と PK4 の test（`scripts/tests/doc-consistency-plan-packet.test.sh`）が見出しを使い、workflow の変更になる。
- `docs/Plans.md`（lane ごとの行を置かない、D-097）。

## Acceptance Criteria

baseline は起点 `843c4938` で下の command を逐語で実行した出力。git の差分の範囲は `origin/main...HEAD`（merge-base から HEAD まで。並走の lane が main に足した変更を数えない）。

- AC1: `bash scripts/doc-consistency-check.sh` が exit 0、末尾の WARN の件数が 12 以下（baseline: exit 0、「結果: WARN 12 件（ERROR なし）」）。
- AC2: `bash scripts/doc-consistency-check.sh --target plan docs/archive/plans/2026-10-24-docs-small-batch-2.md` が exit 0。
- AC-S0-1: Scope の各段の表の「結果」列が全行埋まっている（読んで確かめる。`docs/archive/plans/2026-10-24-docs-small-batch-2.md` の S1〜S7 の表）。
- AC-S1-1: `rg -o 'Fable 5\.1' docs/AGENT_OPERATING_MANUAL.md | wc -l` が `2` で、`rg -o 'Fable 5\.1 になる' docs/AGENT_OPERATING_MANUAL.md | wc -l` も `2`（残る「Fable 5.1」は S1-3 の逐語の 2 つの句の alias の注だけ。baseline: 前者 `6`、後者 `0`）。
- AC-S1-2: ``rg -c '`claude-fable-5`' docs/AGENT_OPERATING_MANUAL.md``・`rg -c 'TD-410' docs/AGENT_OPERATING_MANUAL.md`・`rg -c 'reviewer-fable5' docs/AGENT_OPERATING_MANUAL.md`・`rg -c 'consult-fable5' docs/AGENT_OPERATING_MANUAL.md` がどれも 1 以上、`rg -c 'prompting-claude-fable-5\)' docs/AGENT_OPERATING_MANUAL.md` と `rg -c 'model-config\)「Model aliases」' docs/AGENT_OPERATING_MANUAL.md` がどちらも `1`（baseline: 6 つとも何も出さない、exit 1。backtick 付きで探すのは、今の根拠の link の `prompting-claude-fable-5-1` に当たらないようにするため）。
- AC-S1-3（`## 座組` の外を変えていない）: `git diff -U0 origin/main...HEAD -- docs/AGENT_OPERATING_MANUAL.md` の hunk の行の範囲（`@@` の行の `+` 側）が、全部 `rg -n '^## 座組$|^## 4\. 既存資産 router 表$' docs/AGENT_OPERATING_MANUAL.md` が出す 2 行の間に収まる（baseline: 65 行と 82 行。読んで確かめる）。
- AC-S1-4（読んで確かめる。`docs/AGENT_OPERATING_MANUAL.md` の `## 座組`）: 座組表の Fable の枠の規則（Final Reviewer の Claude 側の充て方の条件〈R3 以上・design lane・closure〉、Fable が使えないときの代え方、発注と違う run を数えず取り直すこと〈S1-1 が条件に実効の effort を足すほか〉、相談役を Writer・reviewer の数に入れないこと、effort の値 high）が変わっていない。S1-3 の起動の句が S1 の公式の資料の読みと合い、再起動が要る 3 つの例外を落としていない。Writer の行の再起動の句が設定と定義に分かれ、定義の側が S1-3 の起動の句と同じ条件を引く（S1-6）。
- AC-S1-5: `rg -c '発注と実効の model か effort が違う' docs/AGENT_OPERATING_MANUAL.md` が `1`、`rg -c '発注と実効 model が違う' docs/AGENT_OPERATING_MANUAL.md` が何も出さない（exit 1）、`rg -c 'CLAUDE_CODE_EFFORT_LEVEL' docs/AGENT_OPERATING_MANUAL.md` が `2`（S1-1 の取り直しの句と S1-3 の起動の句。baseline: 前から順に exit 1・`1`・exit 1）。
- AC-S1-6（再起動の条件。S1-3・S1-6）: `rg -c '定義と設定は本体の checkout を同期し' docs/AGENT_OPERATING_MANUAL.md` が何も出さない（exit 1）、`rg -c '定義は相談役の行の起動の句と同じ条件で効く' docs/AGENT_OPERATING_MANUAL.md`・`rg -c 'env-vars\)「Set variables in settings files」' docs/AGENT_OPERATING_MANUAL.md`・`rg -c -e '--disable-slash-commands' docs/AGENT_OPERATING_MANUAL.md` がどれも `1`（baseline: 前から順に `1`・exit 1・exit 1・exit 1）。
- AC-S2-1: `rg -n -e '^- \*\*IO-04（PLUフォーマッター）\*\*' -e '^- \*\*宅急便部門\*\*' docs/ARCHITECTURE.md` が何も出さない（exit 1。baseline は 2 行: 299・300）。`rg -c '^- \*\*Q40（障害時の対応）\*\*' docs/ARCHITECTURE.md` が `1`（baseline: `1`）。
- AC-S2-2（読んで確かめる。`docs/ARCHITECTURE.md` と S2 の表の置き場）: S2 の表の「中身の置き場」が直した時点の正本にあり、外した項の中身（PLU の profile と手順、保存後の確認を利用者の操作に分けること、宅急便部門の `code_prefix=NULL` と在庫管理対象外の理由）を失っていない。
- AC-S3-1: `rg -n -e 'ECR\+ を要すると答えている（' -e '次の docs の lane で直す' docs/backlog.md` が何も出さない（exit 1。baseline は 1 行: 172）。`rg -c '^- \*\*ECR\+ のサービス終了後の精算\*\*' docs/backlog.md` が `1`（baseline: `1`）。
- AC-S3-2（読んで確かめる。`docs/backlog.md` と `docs/project-memory.md`）: S3 の直した文が `docs/project-memory.md` の ECR+ の 2 行と合い、店の運用の細部（精算した日・回数）を写していない。
- AC-S4-1: `rg -n -e '10月〜大晦日' -e '数週間スパン' docs/architecture/biz-task-specs.md docs/db-design/tracking-system-tables.md docs/SCREEN_DESIGN.md` が何も出さない（exit 1。baseline は 3 行: biz-task-specs:453・tracking-system-tables:133・SCREEN_DESIGN:218）。
- AC-S4-2: `rg -c '73-ui-stocktake.md\) §73.1' docs/architecture/biz-task-specs.md docs/db-design/tracking-system-tables.md docs/SCREEN_DESIGN.md` が 3 file でそれぞれ 1 以上（baseline: 何も出さない、exit 1）。読んで確かめる: 新しい句が 73 §73.1 に無い期間・作業量（例: 正式カウントの日数）を書いていない。
- AC-S5-1: `rg -n -e 'ARCHITECTURE.md` §4 の宅急便部門' docs/backlog.md` が何も出さない（exit 1。baseline は 1 行: 109）。
- AC-S5-2: `rg -c 'stocktake_service.rs' docs/backlog.md` が 1 以上で、その行が期間の旧表記の項目にある（baseline: `1`、同じ項目の今の列挙。読んで確かめる）。`rg -n -e 'biz-task-specs.md:451' -e 'tracking-system-tables.md:137' -e 'SCREEN_DESIGN.md:213' -e '35-biz-stocktake-service.md:374' docs/backlog.md` が何も出さない（exit 1。baseline は 1 行: 197〈Gated Amendment 1 の時点〉）。`rg -n '旧表記の残り 7 か所' docs/backlog.md` が何も出さない（exit 1。見出しから数を外す。baseline は 1 行: 193）。
- AC-S5-3: `rg -n 'squash merge の subject 明示' docs/backlog.md` が何も出さない（exit 1。baseline は 1 行: 180）。
- AC-S6-1: `git ls-files .claude/agents` が `.claude/agents/consult-fable5.md`・`.claude/agents/reviewer-fable5.md`・`.claude/agents/reviewer.md`・`.claude/agents/writer.md` の 4 行（baseline: `reviewer.md` と `writer.md` の 2 行）。`git diff --exit-code origin/main...HEAD -- .claude/agents/reviewer.md .claude/agents/writer.md` が exit 0。
- AC-S6-2: `rg -n -e '^name: reviewer-fable5$' -e '^model: claude-fable-5$' -e '^effort: high$' -e '^disallowedTools: Edit, Write, NotebookEdit$' .claude/agents/reviewer-fable5.md` が 4 行。`diff <(awk 'f>=2; /^---$/{f++}' .claude/agents/reviewer.md) <(awk 'f>=2; /^---$/{f++}' .claude/agents/reviewer-fable5.md)` が exit 0（本文が `reviewer.md` と同じ。baseline: `reviewer-fable5.md` が無く `rg` は exit 2）。
- AC-S6-3: `git show HEAD:.claude/agents/consult-fable5.md | sha256sum` が `92e309d2b6d81c54ad4baa941203f8020f98fcf3c45c2143e836265924144dfd`（本体の checkout の untracked の file と同じ。baseline: HEAD に無く `git show` が失敗）。
- AC-S6-4: `bash scripts/tests/claude-hooks.test.sh` が exit 0（`validate_inventory` が新しい 2 つの定義の frontmatter を監査する。baseline: exit 0）。
- AC-S7-1: `rg -n -e 'State管理（ステップ8）' -e '操作ログ自動削除（ステップ6）' -e '^// 7\. 自動バックアップチェック' docs/function-design/71-mnt-backup.md` が何も出さない（exit 1。baseline は 2 行: 450・453）。`rg -c 'run_startup_auto_backup' docs/function-design/71-mnt-backup.md` が 1 以上（baseline: 何も出さない、exit 1）。読んで確かめる: §71.9 の順序（DB の初期化と操作ログの自動削除の後、State 管理の前）が `src-tauri/src/lib.rs` の `run` の setup と合う。
- AC-S9-1（新しい文言）: `git diff origin/main...HEAD -- docs ':!docs/plans/**' | rg '^\+' | rg -n 'T[O]DO|T[B]D|F[I]XME|H[A]CK|未確[定]|要確[認]|要検[討]'` が何も出さない（exit 1）。文字の組の形は、本 packet の文が doc check の印の検査に当たらないようにするためで、探す語は 7 語のまま。
- data safety: `git diff --name-only origin/main...HEAD` が上の 9 file と本 packet だけ（docs と `.claude/agents/` の 2 つの定義）。`git diff --name-only origin/main...HEAD -- src-tauri src scripts .github` が何も出さない。店の運用の細部・実値・発言の原文を書いていない（読んで確かめる）。

## Design Readiness

- 引用する設計正本（節まで）: `docs/AGENT_OPERATING_MANUAL.md` `## 座組`（表と直後の箇条）と §3.3（S1）、`docs/decision-log.md` の D-099 の (D1)・(D3)（S6）、[67](../../function-design/67-ui-plu-export.md) の関数要求・UI-08-D1〜D3、[25](../../function-design/25-io-plu-formatter.md)、[master-tables](../../db-design/master-tables.md) の「初期データ（全21部門、C-1/C-3 2026-03-29 確定）」（S2）、`docs/project-memory.md` の Store Premises の ECR+ の 2 行（S3）、[73](../../function-design/73-ui-stocktake.md) §73.1 と `docs/SCREEN_DESIGN.md` の `### 利用者の1日の動線`（S4）、[71](../../function-design/71-mnt-backup.md) §71.9 と `src-tauri/src/lib.rs` の `run` の setup・`run_startup_auto_backup`（S7）。
- 必要な設計成果物: Design artifact selection の当たる行は無い（振舞い・command・DB・画面・format を変えない）。状態は existing sufficient。
- plan にしかない durable な判断の昇格先: 本 packet が決める判断は 3 つ。(1) S2 で IO-04 の項も一緒に外す（backlog の項目が「着手のときに決める」とした事項。昇格先は ARCHITECTURE §4 そのもの）。(2) S4 で code comment・mockup を直さない（35 は PR #188 が外した。code comment の理由は本 packet と、S5-2 で直す backlog の項目に残り、mockup は歴史資料の扱いの正本に従う）。(3) S6 で Fable の枠を tracked の定義で起動する（Coordinator の裁定。昇格先は座組表〈S1〉と定義の file そのもの。D-099 の (D3) の適用で、decision-log には足さない）。どれも規則の新設でなく既存の規則（D-127、歴史資料の扱い、D-099）の適用。S1 の owner の決定は座組表そのものが置き場（表の注記の経路）で、decision-log には足さない。
- 前提・制約と延期した gap: `stocktake_service.rs` の module doc の句は code の file を触る後の lane で直す（S5-2。PR #188 は merge 済みで、この句を変えなかった）。送っても安全な理由: comment で振舞いに効かない。定義は本体の checkout の同期で効く（Ordinary Operation の 1 行目、owner の介入 1）。
- 絶対保証の自己点検: S1-3 の「alias の `fable` は既定で Fable 5.1 になる」は公式 model-config「Model aliases」の 2026-10-24 の記述で、Claude Code の版・`ANTHROPIC_DEFAULT_FABLE_MODEL`・Claude apps gateway の session で変わり得る（句は条件を括弧で書く）。定義の `model: claude-fable-5` は alias に依らないが、`CLAUDE_CODE_SUBAGENT_MODEL` より先に効くのは v2.1.251 以上で `CLAUDE_CODE_SUBAGENT_MODEL_FORCE` を立てないときだけ（P-1）。定義の `effort: high` は環境変数 `CLAUDE_CODE_EFFORT_LEVEL` と呼出しの `effort` に上書きされ得る。座組表の規則（S1-1 の後: 発注と実効の model か effort が違う run は数えず取り直す。実効の model と effort を run 報告に記録する）が、どちらの食い違いの検出も持つ。
- 判定: ready。

## Registration / Generation Obligations

| 変更対象 | 義務 | 本 lane |
|---|---|---|
| source / workflow doc 新設・改名・削除 | 親文書の目次・索引 | 本 packet の新設だけ（`docs/plans/` の dated packet で、索引は持たない。D-097）。`.claude/agents/` の定義は索引を持たず、座組表（S1）が名で引く |

ほかの行（Tauri command、function-design の新設、REQ / coverage、route、operator 画面）は該当なし（function-design は 71 の §71.9 の文だけを直し、新設せず、REQ の参照を増減させない）。

## Impact Review Lenses

not applicable: 起点は現地調査・実機確認・外部 tool の振舞いの発見でなく、owner の座組の決定（TD-410）と Coordinator の裁定と repo の中の docs の照合。S3 の店の事実は `docs/project-memory.md` の既存の行を引くだけ。S1・S6 が頼る Claude Code の振舞いは公式の資料で確かめ、外部前提として下の Contract Probe に 1 行ずつ置いた。

## Boundary / Wire Contract

変えない。JSON・CSV・config・manifest・DTO・bindings・DB に触れない（`.claude/agents/*.md` は Claude Code の subagent の定義で、製品の境界でない）。

## Test Plan

Test Design Matrix は下の `## Test Design Matrix` の節（R3）。各段の AC が `rg` の件数・hash・hook test と読んで確かめる項目で直した所を確かめる。

- targeted tests: AC1・AC2・AC-S6-4（`bash scripts/tests/claude-hooks.test.sh`）。code を変えないので Rust・frontend の test と traceability の検査は足さない。classifier が `.claude/agents/*` で full を出すので、hosted の Draft run が Rust・frontend を含む全 suite を回す（本 lane の差分では結果が変わらない見込みで、red なら base の問題として切り分ける）。
- negative tests: AC-S1-1・AC-S1-5・AC-S1-6・AC-S2-1・AC-S3-1・AC-S4-1・AC-S5-1・AC-S5-2・AC-S5-3・AC-S7-1・AC-S9-1（旧い表現・禁じた語が残らないこと）、AC-S1-3（`## 座組` の外を変えていないこと）、AC-S6-1 の後半（既存の定義を変えていないこと）。
- compatibility checks: AC1（link の先の file の実在）、AC-S6-2・AC-S6-3（定義の形が `reviewer.md` と本体の今の file に合う）。
- data safety checks: 上の data safety の AC。
- main wiring/integration checks: AC-S2-2（外した項の中身の置き場）、AC-S1-4（座組表の規則が変わらない）、merge 後の最初の `reviewer-fable5`・`consult-fable5` の run の実効 model の記録（Ordinary Operation の 1・3 行目。Ready・merge の条件にしない。Contract Probe の P-1）。

起票と是正の run の検証（AC1・AC2 と `bash scripts/check-workflow-git.sh`）の結果は PR の Validation の欄に書く。

## Test Design Matrix

Risk は上の `## Risk` と同じ（R3。helper の `parse_packet` は行頭の Risk 行を 1 つだけ受けるので、ここに書き写さない）。

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| 座組表の Fable の枠が Fable 5（S1-1・S1-2・S1-4・S1-5） | 「Fable 5.1」が座組の値として残る | regression（`rg` の件数） | AC-S1-1 | 72・73・80 のどれかの「Fable 5.1」を直し忘れると前者が `2` を超える |
| 相談役の起動の句が逐語（S1-3） | 句を言い換えて alias の注が抜ける | regression | AC-S1-1 の後者・AC-S1-2 | 「Fable 5.1 になる」が 2 つ揃わない、`model-config)「Model aliases」` か `consult-fable5` が無い |
| 実効の effort が違う run を数えない（S1-1、round 2 の Codex F1） | 取り直しの条件が model だけのまま、環境変数の注が抜ける | regression（`rg` の件数） | AC-S1-5 | 取り直しの句を直し忘れると新しい句が `1` でなく旧い句が残る、起動の句か取り直しの句の環境変数の注を落とすと `2` を割る |
| 定義の反映の条件が公式の 3 つの例外を持ち、Writer の行と 1 通りに読める（S1-3・S1-6、round 3 の Codex F2・Opus #3） | 例外を落とす、Writer の行に定義と設定を合わせた旧い句が残る | regression（`rg` の件数） | AC-S1-6 | 旧い句が残ると 1 つめが exit 0、例外の句か Writer の行の新しい句を落とすと残りのどれかが `1` でない |
| 座組表の規則を変えない（失敗定義） | 代え方・取り直し・本数の句を消す | 読んで確かめる | AC-S1-4・AC-S1-3 | `## 座組` の外の hunk、規則の句の欠け |
| Final Review の定義が review 用（S6-1、F1） | 本文が相談役の prompt、または `model` が alias | data / schema（frontmatter と本文の比較） | AC-S6-2 | 本文が `reviewer.md` と違う、`model: fable` と書く、編集の tool を外し忘れる |
| 相談役の定義が本体の file と同じ（S6-2） | 写す時に中身が変わる | data（hash） | AC-S6-3 | 1 byte でも違えば hash が合わない |
| 定義が hook の監査を通る（D-099 の (D3)） | frontmatter に `hooks`・`permissionMode`・`mcpServers` | CLI | AC-S6-4 | 3 つの key のどれかを足すと `validate_inventory` が red |
| 既存の定義を変えない | `reviewer.md`・`writer.md` の書換え | regression | AC-S6-1 | `git diff --exit-code` が exit 1 |
| §4 から決まった 2 項が外れる（S2） | 項が残る、Q40 まで消える | regression | AC-S2-1 | 2 項のどれかが残るか、Q40 が `1` でない |
| ECR+ の前提（S3） | 旧い前提の句が残る | regression | AC-S3-1 | 172 行の句が残る |
| 期間の旧表記（S4） | 3 か所のどれかが残る、link が切れる | regression | AC-S4-1・AC-S4-2・AC1 | 旧表記が出る、`73-ui-stocktake.md) §73.1` が file のどれかに無い |
| backlog の項目（S5） | 済んだ項目・外すと決めた項目が残る、35 の 2 つの句（#188 が外した）を残りに数える | regression | AC-S5-1・AC-S5-2・AC-S5-3 | 109・180 の項目が残る、35 の file:行（`35-biz-stocktake-service.md:374`）が backlog に残る、見出しに「7 か所」が残る |
| §71.9 の順序（S7） | 手順の番号が残る、関数の名が無い | regression | AC-S7-1 | 「ステップ6」「ステップ8」か `// 7.` が残る、`run_startup_auto_backup` が無い |

State Lifecycle Matrix: not applicable（UI・data・cache の状態を持たない。定義の反映の前後は Ordinary Operation の 1・5 行目）。Adjacent Pattern Audit: 「Fable 5.1」の全 site（S1 の sweep）、期間の旧表記の全 site（S4 の全数）、setup の手順の番号を引く全 site（S7 の `rg`）を列挙した。Residual Test Gaps: 定義で起動した run の実効の model・effort は自動 test で確かめられず、merge 後の最初の run の記録に依る（P-1）。`/tasks` を取れない run は、model を run の自己申告、effort を run の `echo $CLAUDE_EFFORT` で記録し、どちらも無い項は未実測と書いて数える（Ordinary Operation の 1 行目）。

## Review Focus

- S1・S6: 起動の句（`subagent_type: reviewer-fable5`・`consult-fable5`、frontmatter の `model: claude-fable-5`・`effort: high`、呼出しで `model`・`effort` を渡さない）が公式の資料と合うか。Final Review の Claude 側が review 用の本文で走るか。実効の model か effort が発注と違う run が本数に入る経路が残らないか。座組表の規則（枠の本数・独立性・代え方・effort の値）を S1-1 の取り直しの条件のほかに変えていないか。R3 の判定と、本 lane 自身の Final Review を §3.3 で fresh Opus 5.5 にする理由（Workflow State の注）が成り立つか。座組表の外に、座組の値として「Fable 5.1」を書く tracked の所が残っていないか。定義の反映の条件（S1-3 の 3 つの例外、S1-6 の Writer の行の分け方）が公式の資料と合い、表の中で 1 通りに読めるか。
- S2: IO-04 の項を一緒に外す判断と、外した中身が 67・25・master-tables で辿れるか。
- S3: 直した前提が project-memory の 2 行と合い、残る論点を落としていないか。
- S4: 新しい句が 73 §73.1 の店の事実の範囲に収まるか。35・code comment・mockup を直さない理由が成り立つか。
- S7: 順序が今の `lib.rs` の `run` の setup と合うか。
- Q1（D-127）: この変更で不要になる記述・条件は無いか。例: S1-2 の「定義を使わず session の値を継承」（定義で起動する形と合わないので外す）、S3-2 の「次の docs の lane で直す」の句、S5 の済んだ backlog の項目と見出しの数、Writer の行の定義と設定を合わせた再起動の句（S1-6）、S7 の手順の番号。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| 座組表の Final Reviewer の Fable の枠（owner 2026-09-25、TD-410） | `docs/AGENT_OPERATING_MANUAL.md` `## 座組` の Final Reviewer の行 | S1-1・S1-2 | AC-S1-1・AC-S1-2・AC-S1-5（`rg`） | 非対象（読んで確かめる AC-S1-4） |
| 座組表の相談役の行（owner 2026-09-25、TD-410） | 同じ節の相談役の行 | S1-3 | AC-S1-1・AC-S1-2 | 非対象 |
| 座組表の確認日と根拠 | 同じ節の表の後の箇条 | S1-4・S1-5 | AC-S1-2 | 非対象 |
| 座組表の定義と設定の反映の条件 | 同じ節の Writer の行と相談役の行 | S1-3・S1-6 | AC-S1-6 | 非対象（読んで確かめる AC-S1-4） |
| D-099 (D3) tracked の subagent 定義 | `docs/decision-log.md` D-099 | S6-1・S6-2 | AC-S6-1〜AC-S6-3 | 非対象 |
| SPEC-WF-HARNESS5-D3 定義の frontmatter の監査 | D-099 (D3)、`scripts/tests/claude-hooks.test.sh` の `validate_inventory` | S6（変えない契約に定義を足す） | AC-S6-4（`bash scripts/tests/claude-hooks.test.sh`） | 非対象 |
| SPEC-WF-HARNESS5-D1 `.claude/agents/*` は full | D-099 (D1)、`scripts/ci/classify-changes.sh` | 変えない（本 lane の Risk と Minimum の根拠） | 既存の `scripts/tests/classify-changes.test.sh` | 非対象 |
| UI-08-D1〜D3・IO-04 の profile | [67](../../function-design/67-ui-plu-export.md)・[25](../../function-design/25-io-plu-formatter.md) | S2-1（ARCHITECTURE §4 の重複を外す） | AC-S2-1 | 非対象（AC-S2-2） |
| 部門 12 の `code_prefix=NULL` | [master-tables](../../db-design/master-tables.md) の初期データ | S2-2 | AC-S2-1 | 非対象（AC-S2-2） |
| ECR+ の店の事実 | `docs/project-memory.md` Store Premises | S3 | AC-S3-1 | 非対象（AC-S3-2） |
| 棚卸しの期間の店の事実 | [73](../../function-design/73-ui-stocktake.md) §73.1 | S4・S5-2 | AC-S4-1・AC-S4-2・AC-S5-2 | 非対象 |
| MNT-01 起動時の自動 backup の位置 | [71](../../function-design/71-mnt-backup.md) §71.9 | S7 | AC-S7-1 | 非対象 |

adjacent-contract sweep: 座組表の Writer の行は再起動の句だけを S1-6 で設定と定義に分け、Plan Reviewer の行（「起動と実効の注意は Writer 行と同じ」）は Writer の行を引くので触らない。72 §72.5（同じ型の手順の番号）は PR #198 が持つ（S7）。classifier・helper・hook test は変えない。

## Contract Probe

- P-1 frontmatter に `model: claude-fable-5` を書いた定義で起動した subagent が Fable 5 で動く: 公式 sub-agents「Choose a model」（呼出しの `model` が無ければ定義の frontmatter が `CLAUDE_CODE_SUBAGENT_MODEL` より先）を 2026-10-24 に読んだ -> 文書は一致。この環境の `env` には `CLAUDE_CODE_SUBAGENT_MODEL=claude-sonnet-5` があり、frontmatter が勝つのは v2.1.251 以上で、`CLAUDE_CODE_SUBAGENT_MODEL_FORCE` を立てていないときだけ（v2.1.251 より前は `CLAUDE_CODE_SUBAGENT_MODEL` が frontmatter より先、FORCE を立てると定義の `model` は無視される。S1 の公式の資料）。この環境は v2.1.310（`claude --version`）で、`env | grep -E '^CLAUDE_CODE_SUBAGENT'` は `CLAUDE_CODE_SUBAGENT_MODEL` だけを出し FORCE は無い（2026-10-24）。版を下げるか FORCE を立てた session では、定義の run が Sonnet 5 で動き得て、捕まえるのは Ordinary Operation の 1・2 行目の実効の model の記録と取り直し。観測: PR #198 の Final Review の Fable の run が、model `claude-fable-5`・effort high を自己申告した（Coordinator の記録、2026-10-24）。`/tasks` の実効の model・effort の metadata は未取得（未実測）。merge の後の最初の `reviewer-fable5`・`consult-fable5` の run で記録する（Ordinary Operation の 1・3 行目）。
- P-2 alias の `fable` は既定で Fable 5.1 に解決される: 公式 model-config「Model aliases」の「Unless you set `ANTHROPIC_DEFAULT_FABLE_MODEL`, the `fable` alias resolves to Fable 5.1, except in Claude apps gateway sessions」（2026-10-24 に WebFetch）、`.claude/settings.json` の `env` は `CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH` だけで `ANTHROPIC_DEFAULT_FABLE_MODEL` が無い（`jq '.env' .claude/settings.json`） -> 文書は一致。session が Claude apps gateway でないことは確かめていない（未確認）。定義は alias を使わない（`model: claude-fable-5`）ので、この前提は座組表の句が alias を避ける理由にだけ効く。
- P-6 定義の `effort` は呼出しの `effort` と環境変数 `CLAUDE_CODE_EFFORT_LEVEL` に上書きされる（前提: 公式 [env-vars](https://code.claude.com/docs/en/env-vars) の `CLAUDE_CODE_EFFORT_LEVEL` の行の「A `maxEffortLevel` cap still applies」〈2026-10-24 に WebFetch〉により、`maxEffortLevel` があれば実効はその上限で切られる。今の `~/.claude/settings.json`・`.claude/settings.json`・`.claude/settings.local.json` に `maxEffortLevel` は無く〈`rg -n 'maxEffortLevel'` が exit 1〉、`/etc/claude-code` も無い〈2026-10-24〉）: 公式 sub-agents「Choose an effort level」（「The parameter overrides the `effort` field … The `CLAUDE_CODE_EFFORT_LEVEL` environment variable takes precedence over both.」）と「Supported frontmatter fields」の `effort`（「Overrides the session effort level, but not the `CLAUDE_CODE_EFFORT_LEVEL` environment variable」）を 2026-10-24 に WebFetch -> 一致。`.claude/settings.json` の `env` に `CLAUDE_CODE_EFFORT_LEVEL` は無く（上の `jq`）、本 run の `echo $CLAUDE_CODE_EFFORT_LEVEL` は空（2026-10-24）。
- P-7 run の Bash の `echo $CLAUDE_EFFORT` は、その run の実効の effort を返す（前提: 公式 [env-vars](https://code.claude.com/docs/en/env-vars) の `CLAUDE_EFFORT` の行「Set automatically in Bash tool subprocesses and hook commands to the effort level in effect when the subprocess starts … Only set when the current model supports the effort parameter」〈2026-10-26 に WebFetch〉。subagent の subprocess が親の session でなく subagent 自身の値を返すことは公式の文に無く、2026-10-18 の試験〈main session は `high` のまま、起動の `effort` を `medium` にした subagent で `medium` が出た。repo の外の記録〉で確かめた。model に対応する自動の環境変数は同じ page に無いので、model の出所は Ordinary Operation の 1 行目のまま）。
- P-3 既存の `.claude/agents/` に足した定義は、公式の 3 つの例外に当たらなければ再起動なしで次の委譲から効く: 公式 sub-agents「Write subagent files」（「the next delegation uses the updated definition, with no restart needed」。「Three cases still need a restart」: session の開始時に無かった `agents` の dir、`--add-dir`・`/add-dir` で足した dir の中の定義、`--disable-slash-commands` で始めた session。2026-10-24 に WebFetch）、本体の checkout の `.claude/agents/` は `reviewer.md`・`writer.md`・`consult-fable5.md` を持って在る（`ls -la /home/kosei/projects/inventory-system-public/.claude/agents`、2026-10-24） -> 1 つめの例外に当たらない。本体の checkout は session の起点の dir で `--add-dir` の側でないとみなすが、main session を `--disable-slash-commands` で始めていないことは sandbox の中から確かめられない（`ps` は sandbox の外の process を出さない。未確認）。当たっていれば定義が見つからないか古いままで、Ordinary Operation の 1・2 行目の起動の失敗か実効の記録で見える。
- P-4 git は同じ名の untracked の file を上書きせず merge を止める: scratch の repo で、同じ中身の untracked の `a/f.md` を置いた clone に `a/f.md` を足した commit を `git pull --ff-only` した（git 2.56.0、2026-10-24） -> 「Please move or remove them before you merge.」「Aborting」で止まった。中身が同じでも止まる。
- P-5 本体の `.claude/agents` は sandbox から書けず、worktree の側は書ける: `test -w /home/kosei/projects/inventory-system-public/.claude/agents` が偽、本 lane の worktree の `test -w .claude/agents` が真（2026-10-24） -> 本体の同期は owner、定義の追加は Writer が worktree で行う。
- P-8 Agent tool の呼出しの `model` が受ける値: Coordinator が自分の Agent tool の schema を読み、`model` は enum `sonnet`・`opus`・`haiku`・`fable` だけで full model ID を受けないことを記録した（Coordinator の記録、2026-10-24） -> 呼出しで Fable 5 を選べず、Fable 5 は定義の frontmatter（`model: claude-fable-5`）で選ぶ。本 lane の Final Review を §3.3 で fresh Opus 5.5 にする理由の 1 つ（Workflow State の注）。

## Data Safety

- commit しないもの: 店の運用の細部・実値・発言の原文（S3・S4）、回答台帳の本文（TD-410 は ID で引く）、`.local/` の相談の記録の中身。
- local-only: `/home/kosei/projects/inventory-system-public/.claude/agents/consult-fable5.md`（untracked の原本。S6-2 は中身を写すだけで、hash を AC に置く）、`.local/reports/**`。
- synthetic-only: なし（data・fixture を足さない）。

## Implementation Results

[PR #203](https://github.com/kosei-w90607/inventory-system-desktop/pull/203) で Scope の S1〜S7 を入れ、2026-10-26 に helper 経由で main へ merge した（owner の承認の後）。座組表の Fable の枠を当面 Fable 5 にし（TD-410）、Final Review の Claude 側の `reviewer-fable5` と相談役の `consult-fable5` の定義を tracked にして、呼出しで `model`・`effort` を渡さない起動の形と、実効の model か effort が発注と違う run を数えず取り直す規則を座組表に書いた。ARCHITECTURE §4 の決まった 2 項（IO-04・宅急便部門）を外し、backlog の ECR+ の精算の項目の前提を `docs/project-memory.md` の店の事実に合わせ、棚卸しの期間の旧表記 3 か所を 73 §73.1 に合わせ、71 §71.9 の起動の位置を手順の番号でなく順序と関数の名で書いた。backlog は済んだ項目と owner が外すと決めた項目（TD-429）を外し、期間の旧表記の項目を残りの code comment 1 行だけにした。S5-2 は実装の run が前提の食い違い（PR #188 が merge 済み）で止まり、Gated Amendment 1 で前提を合わせて直した。Final Review broad の P3 は Gated Amendment 2 で直し、新しい head で fresh broad を取り直した。

## Review Response

### Plan Review round 1（対象 `d3007c38`）

reviewer は Codex（発注 554、GPT-6.1 Sol）と fresh Opus 5.5（fork でない）。どちらも reject（Codex: P2 1 / P3 2、Opus: P2 1 / P3 5）。裁定は Coordinator（2026-10-24）、是正は起草役。

| finding | 重大度 | 裁定 | 是正（packet の所） |
|---|---|---|---|
| Codex #1: S1 は起動の仕方を変えるのに Ordinary Operation を `not applicable` にした | P2 | 受理 | `## Ordinary Operation` を workflow の操作列の表にした（`docs/templates/plan-packet.md` の同節「workflow の変更は、発注 → 停止と訂正 → review → owner 判断の通常列を使う」）。発注 → 実効 model の記録 → 食い違い時の取り直し → 相談役 → owner の Ready / merge、本 lane 自身の Final Review の起動の 5 行 |
| Opus F1: Final Reviewer の Claude 側を、未 tracked の相談役の定義（本文が相談役の prompt）で起動させる | P2 | 受理 | S6 を足し、Final Review 用の `reviewer-fable5`（本文は `reviewer.md` と同じ、frontmatter は `model: claude-fable-5`・`effort: high`・編集の tool なし）と相談役の `consult-fable5`（本体の untracked の file をそのまま、`sha256sum` を AC-S6-3 に置いた）を tracked にする。前例は PR #127（D-099 の (D3)、`git log --oneline --grep '#127'` が `a5fb9b16`）。S1-2・S1-3 は `subagent_type` で定義を名指しし、呼出しで `model` を渡さない形にした。Non-scope の「`consult-fable5` を tracked にするか」の項は外した。classifier が `.claude/agents/*` を full にすることから Risk を R3 に上げた（`## Risk`。D-099 の (D1)、#127 の packet も R3）。Final Review Minimum（2）と Human Gate（`ready,merge`）は変わらない。起票の報告で owner の選択とした「`consult-fable5` を tracked にするか」は、Coordinator の裁定で (a) tracked にすると決めた（前例 #127、可逆） |
| Codex #2 ＋ Opus F3: §4 を引くのは backlog 109 だけとしたが、324 もある | P3 | 受理 | S2 の表の後を「109 と 324 の 2 行。324 は残る Q40 の項を引くので切れず、直さない」に直した（`843c4938` で同じ command を取り直した） |
| Codex #3 ＋ Opus F5: 35 の 382 行「数ヶ月に及ぶ長期作業」の漏れ。S5-2 の「#188 が外し」を済んだことと書いた | P3 | 受理 | S4 の直さないものに 382 を足し、全数の command（`-e '長期作業'` を含む）を書いた。S5-2 を「35 の 377・382 行の句は PR #188 が外す（#188 の merge まで残りに数える）」にし、AC-S5-2 に `rg -n '377・382' docs/backlog.md` を足した |
| Opus F2: alias の `fable` が 5.1 になる出所は model-config「Model aliases」 | P3 | 受理 | 公式 model-config を 2026-10-24 に WebFetch で確かめ（「Unless you set `ANTHROPIC_DEFAULT_FABLE_MODEL`, the `fable` alias resolves to Fable 5.1」）、S1 の公式の資料と S1-3 の逐語の句の出所を model-config「Model aliases」にした。sub-agents は優先順の出所として残した。Design Readiness の自己点検と Contract Probe の P-2 も同じ出所にした |
| Opus F4: sweep の command に `--hidden` が無い | P3 | 受理 | S1 の sweep を `rg -n --hidden 'Fable 5\.1' --glob '!docs/archive/**' --glob '!node_modules' --glob '!.git' .` にして取り直し、出た行を全部書いた。hidden の dir は `rg -n --hidden -i 'fable' .claude .agents .github .codex` で何も出さない |
| Opus F6: AC-S1-1 が逐語の句を求めるのに S1-3 は向きだけ | P3 | 受理 | S1-3 の起動の句と advisor の句を「次のとおり逐語」とし、全文を書いた |
| Coordinator の追加: 71 §71.9 の手順の番号が今の `lib.rs` と合わない | — | 追加 | S7 を足した。現物は `843c4938` の `src-tauri/src/lib.rs` の `run` の setup（`// 5. 操作ログ自動削除`・`// 6. 自動バックアップチェック`・`// 7. State管理`、`fn run_startup_auto_backup`）。72 §72.5 は PR #198 が持つので触らない |

起草役が是正の中で見つけたもの: 「他の lane との所有」の重なりの file の数を、round 1 の版は表に 5 file を挙げて「4 file」と書いていた。S7 の 71 を足して数え直し「6 file」にした（`.claude/agents/*` と `docs/AGENT_OPERATING_MANUAL.md` は重なりなしの行）。

旧い前提の sweep（`docs/plans/2026-10-24-docs-small-batch-2.md` の中、遷移の記録と本節を除く）: 「定義の file の名は書かない」「`.claude/agents/*` を触らず」「`.claude/agents/` の定義を tracked にする・変える」（非目的）「R2 で Test Design Matrix は任意」の live な文は是正の後に残らない（遷移の記録 3 は append-only の当時の記録）。

### Plan Review round 2（対象 `a7a66e1e`）

reviewer は Codex（発注 557）と fresh Opus 5.5（fork でない）。Codex は reject（P2 1 / P3 2）、Opus は approve（P3 5）。裁定は Coordinator（2026-10-24。全件受理、本 lane の Final Review の Claude 側を fresh Opus 5.5 にする、P-1 と Codex F1 の閉じ方）、是正は起草役。

| finding | 重大度 | 裁定 | 是正（packet の所） |
|---|---|---|---|
| Codex F1: 実効 effort が発注と違っても（`CLAUDE_CODE_EFFORT_LEVEL=medium` が定義の high を上書き）本数に数えられる | P2 | 受理（Coordinator の裁定: 本数に数える条件に実効の effort を入れ、座組表の取り直しの句に effort を揃える） | S1-1 で座組表の取り直しの句を「発注と実効の model か effort が違う（…環境変数 `CLAUDE_CODE_EFFORT_LEVEL` が定義の effort を上書きした等）run は数えず取り直す」に書き換える（逐語、AC-S1-5）。S1-3 の起動の句を「呼出しで `model`・`effort` を渡さない」「実効の model と effort を run 報告に記録する」にし、環境変数の優先を書いた。Ordinary Operation の 1・2 行目を、実効の model・effort と run の `echo $CLAUDE_CODE_EFFORT_LEVEL` で数えるか取り直すかを決める形にした。S1 の公式の資料に「Choose an effort level」を足し、Contract Probe に P-6 を足した。Goal・失敗定義・AC-S1-4・Matrix・Contract Ledger・Review Focus・Design Readiness の自己点検・Residual を同じ条件にした |
| Codex F2 ＋ Opus #1: S5-2 が backlog に行番号「377・382」を書かせ、時点の commit が無い（`docs/DOC_STYLE_GUIDE.md` §1） | P3 | 受理 | 行番号でなく 35 §20.5 の 2 つの見出し（「設計判断 — force_fill パラメータ」「設計判断 — valuation_cost_price_centi のタイミング」）で引く形にした（S5-2）。#188 の merge で行が動いても指す先が変わらないので、commit を添える形より素直。AC-S5-2 の 3 つめの検査を見出しの `rg` にした |
| Codex F3: 逐語の句と P-2 が alias の解決先を無条件に Fable 5.1 と書く（Claude apps gateway では違う） | P3 | 受理 | S1-3 の 2 つの句を「alias の `fable` は既定で Fable 5.1 になる」にし、起動の句に条件（`ANTHROPIC_DEFAULT_FABLE_MODEL` と Claude apps gateway の session）を書いた。AC-S1-1 の「Fable 5.1 になる」は 2 のまま。P-2 に gateway の除外の引用を足し、gateway でないことは未確認と書いた。Design Readiness の自己点検も同じ |
| Opus #2: (b) が §3.3 の「一時的に利用不能」に当たる理由が無い | P3 | 受理 | Workflow State の注を (b) だけの現行の文に書き直し、残す理由に「Fable 5 は model ID でしか選べず、定義は merge まで本体の checkout に無く、本体の `.claude/agents` は sandbox から書けない。呼出しの `model` が full model ID を受けるかは公式に書かれていない」を書いた（§3.3 の一時的な利用不能で、merge で解ける） |
| Opus #3: (a) で置いた定義を起動前に確かめる手順が無い | P3 | 受理（Coordinator の裁定で (a) を外したので、手順は要らない） | (a) を Workflow State の注・Ordinary Operation の 5 行目から外し、Workflow State の `Final Reviewer` を fresh Opus 5.5 にした。Owner Effort Budget の予備から (a) の配置の 1 を外した |
| Opus #4: S3 の出所の `rg` の主張が実測と合わない（APP-01 が漏れる） | P3 | 受理 | S3 の出所の括弧を「…と本項目と、`docs/plu-export-and-real-csv-verification.md` の APP-01〈調査項目で、精算に ECR+ が要るとは書かない〉の 4 行」にした（`843c4938` と同じ tree で同じ command を取り直した） |
| Opus #5: Writer の行は再起動を書き、Fable の行は書かず、表の中で 2 つの規則が並ぶ | P3 | 受理 | S1-3 の起動の句に「定義は本体の checkout に在れば次の委譲から効く（session の開始時に `.claude/agents/` が在れば再起動は要らない）」を足した。Non-scope の Writer の行の項を、Writer の句が定義と設定を合わせて書く句で安全側に倒れること、Fable の行は起動の句が条件を書くことに書き直した |
| P-1（相談役の実効 model の記録） | — | Coordinator の裁定 | P-1 に PR #198 の Final Review の Fable の run の自己申告（`claude-fable-5`・effort high）を観測として書き、`/tasks` の metadata は未取得（未実測）とした。plan-approved の前の起動の予定の文は、この観測と merge 後の最初の run の記録に置き換えた |
| owner の決定 TD-429: backlog の「squash merge の subject 明示」の項目を外す | — | owner の決定（2026-10-24、repo 外の回答台帳 TD-429。Coordinator の中継） | Non-scope の同じ項を外し、Scope に S5-3（項目を外す。理由: PR title を Conventional にする今の運用で足りる）と AC-S5-3 を足した。出所の (5)、Goal、Owner Effort Budget の消費（2）、「他の lane との所有」の backlog の行（180 を足した。並走の 10 本の `gh pr diff` に同じ項目の `+`/`-` は無い）も合わせた。backlog の項目そのものは、本 packet の「起票と Plan Review の是正の run は packet だけを置く」に従い、plan-approved の後の実装の run で外す |

### Plan Review round 3（対象 `57b01d3d`、天井）

reviewer は Codex（発注 564）と fresh Opus 5.5（fork でない）。Codex は reject（P2 2）、Opus は approve（P3 8）。裁定は Coordinator の裁定（天井の disposition = 一括是正、2026-10-24。全件受理）、是正は起草役。

| finding | 重大度 | 裁定 | 是正（packet の所） |
|---|---|---|---|
| Codex F1 ＋ Opus #1: Ordinary Operation の 1 行目の「環境変数が空なら数える」は S1-1 の「実効が違う run を取り直す」より強く、環境変数が定義と同じ `high` の正常な run を数えられない。model の出所が `/tasks` だけ | P2 | 受理 | 1 行目の次へ進む条件を、実効の model と effort が発注と同じなら数える（環境変数が定義と同じ値を立てていても実効が同じなら数える）形にした。model の出所を「`/tasks`、取れなければ run の自己申告（system prompt の model 名）、どちらも無ければ未実測」、effort の出所を「`/tasks`、取れなければ `echo $CLAUDE_CODE_EFFORT_LEVEL` からの導出、どちらも無ければ未実測」と同じ形にした。2 行目と S1-1 の逐語の句の環境変数の例を「定義の effort を違う値で上書きした」にした。3 行目・Residual Test Gaps も同じ出所の順にした |
| Codex F2: S1-3 の逐語の句・P-3・Non-scope の「開始時に `.claude/agents/` が在れば再起動は要らない」が公式の例外を落とす | P2 | 受理 | 公式 [sub-agents](https://code.claude.com/docs/en/sub-agents)「Write subagent files」を 2026-10-24 に WebFetch で読み直し、「Three cases still need a restart」の 3 つ（開始時に無かった `agents` の dir、`--add-dir`・`/add-dir` の dir の中の定義、`--disable-slash-commands` で始めた session）を S1 の公式の資料、S1-3 の逐語の句、P-3 に書いた。P-3 に main session の起動の flag は sandbox の中から確かめられない（未確認）と書いた。Non-scope の Writer の行の項は S1-6 に移したので外した。AC-S1-6 の `--disable-slash-commands` の件数で落としを捕まえる |
| Opus #2: P-1 の「frontmatter が勝つ」に版の前提が無い | P3 | 受理 | P-1 に「v2.1.251 以上で `CLAUDE_CODE_SUBAGENT_MODEL_FORCE` を立てないときだけ」と、この環境の v2.1.310・FORCE なし（`claude --version`・`env`）を書いた。S1 の公式の資料に「Before v2.1.251」と FORCE の文を足し、Design Readiness の自己点検にも同じ前提を書いた |
| Opus #3: 座組表の Writer の行の「定義と設定は…再起動してから効く」が S1-3 と 2 つの規則になる | P3 | 受理 | S1-6 を足し、Writer の行を設定と定義に分ける逐語の句にした。修正案の「設定は再起動の後に効く」は、公式 [env-vars](https://code.claude.com/docs/en/env-vars)「Set variables in settings files」（2026-10-24 に WebFetch）が保存で走行中の session に効く値と起動時にだけ読む機能・外した変数を分け、`CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH` がどちらかを書いていないので、そのまま写さず「再起動してから頼る」と出所を書く形にした。定義の側は相談役の行の起動の句と同じ条件を引く。非目的・adjacent-contract sweep・Contract Ledger・Matrix・AC-S1-4・AC-S1-6・Review Focus を合わせた |
| Opus #4: backlog の項目の見出しの「残り 7 か所」をどう直すかが無い | P3 | 受理 | S5-2 に「見出しから数を外す」を足し、AC-S5-2 に `rg -n '旧表記の残り 7 か所' docs/backlog.md` が exit 1 を足した（baseline は 193 の 1 行）。Matrix の S5 の行も合わせた |
| Opus #5: 定義を tracked にした後、定義が在るのは本体の checkout が merge を含む branch にある間だけ | P3 | 受理 | Ordinary Operation の 1 行目の初期状態に、main か本 lane の merge を含む branch にある間だけ定義が在り、前に分かれた branch の間は見つからず §3.3 の代え方へ落ちることを書いた |
| Opus #6: 呼出しの `model` が full model ID を受けるかを確かめていない | P3 | 受理 | Contract Probe に P-8 を足した（Coordinator が自分の Agent tool の schema を読み、`model` は enum `sonnet`・`opus`・`haiku`・`fable` だけで full model ID を受けない、2026-10-24）。Workflow State の注の理由を P-8 で書き直した |
| Opus #7: S5-3 の理由の「PR title を Conventional にする今の運用」は正本に規則として無い | P3 | 受理 | S5-3 の理由を「正本に PR title の規則は無い。squash の subject は PR title で、直近 30 commit の外れは 0」の形にした（`rg -n -i 'title' docs/DEV_WORKFLOW.md .github/pull_request_template.md docs/agent-guidance/merge-evidence.md docs/ci.md` は closeout の `Plans.md` の行の title だけを出す） |
| Opus #8: 71 §71.9 を番号なしで書き、兄弟の 72 §72.5 は #198 で番号つきになる | P3 | 受理 | S7 の注に、#198 の 72 §72.5 は番号つき（`origin/agent/oplog-retention` の `9a9dbe09` の 72:76）で書き方が分かれ、両方の merge の後に揃えるかは closeout で見る、を足した。#198 へは Coordinator が別に伝える |

旧い前提の sweep（本 packet の中、遷移の記録と本節を除く）: 「環境変数が空」、例外の無い「再起動は要らない」、「full model ID を受けるか」、「今の運用で足りる」、Writer の行の Non-scope の項の live な文は是正の後に残らない（`rg -n -e '環境変数が空' -e '再起動は要らない' -e 'full model ID を受けるか' -e '今の運用で足りる'` の hit は、例外を書いた S1-3 の逐語の句と、round 1・2 の Review Response だけ）。

### Plan Review closure（対象 `f71d131c`）

reviewer は Codex（発注 568）と fresh Opus 5.5（fork でない）。Codex は approve（新規 0、F1〜F9 closed）、Opus は approve（P3 3、F1〜F9 closed）。出所は repo 外の `.local/reports/plan-review/pr203-closure-opus-f71d131c.md` と `.local/codex-orders/last-568-plan-closure-docs-small-batch-2-codex.md`。P3 3 件は Coordinator の裁定（2026-10-24、全件受理）で plan-gate のまま直した（前例: PR #201 の遷移 8）。是正は Writer。

| finding | 重大度 | 裁定 | 是正（packet の所） |
|---|---|---|---|
| Opus #1: 定義が見つからないとき、`consult-fable5` も Final Reviewer の行の §3.3 の代え方へ落ちると読める | P3 | 受理 | Ordinary Operation の 1 行目の初期状態の括弧を、`reviewer-fable5` は Final Reviewer の行の §3.3、`consult-fable5` は相談役の行の代え方（省くか fresh Opus 5.5 で代え、遷移を止めない）に分けた |
| Opus #2: effort の導出が `maxEffortLevel` の上限を見ない | P3 | 受理 | P-6 に前提として、公式 env-vars の `CLAUDE_CODE_EFFORT_LEVEL` の行の「A `maxEffortLevel` cap still applies」（2026-10-24 に WebFetch）と、今の user・project の settings に `maxEffortLevel` が無いこと（`rg -n 'maxEffortLevel'` が exit 1、`/etc/claude-code` も無い）を書いた |
| Opus #3: S5-3 の「squash の subject は PR title」が repo の設定と合わない | P3 | 受理 | S5-3 の理由を「PR title〈commit が 1 つの PR では commit の subject〉」にし、`gh api repos/kosei-w90607/inventory-system-desktop --jq .squash_merge_commit_title` → `COMMIT_OR_PR_TITLE` を併記した。結論（項目を外す）は変わらない |

Final Review と merge（2026-10-26、closeout で追記）: Gated Amendment 2 の後の fresh broad（Codex 583 = approve、Claude 側 fresh Opus 5.5 = approve）と、base 同期だけを見る closure（Codex 587 = approve P1/P2/P3 0、head `8fe2336f`）で helper の review の record は pass。Ready の後の hosted CI run 40328725627 の `Merge gate` が success で、owner の承認の後に helper 経由で merge した（[PR #203](https://github.com/kosei-w90607/inventory-system-desktop/pull/203)）。fresh broad で受けた P3 のうち packet の文の 2 件は、merge の前に直すと Gated Amendment で fresh broad の取り直しになるので、本 closeout で直した: (1) Contract Probe の `P-7` が 2 つあったので、後の方（Agent tool の呼出しの `model` が受ける値）を `P-8` にし、それを指す参照（Workflow State の注、S1 の公式の資料の sub-agents の行、round 3 の Opus #6 の行）を `P-8` に直した（`CLAUDE_EFFORT` の文脈の `P-7` は変えない）。(2) Workflow State の注の「本 lane の Final Review の時点では Fable 5 を起動できない」を「review 用の定義（`reviewer-fable5`）で Fable 5 を起動できない」に直した。closeout で packet を `docs/archive/plans/` へ移し、`docs/Plans.md` に完了を書いた。並走の lane の packet の `Final Reviewer: Fable 5.1` を当面 Fable 5 に揃えることは、各 lane の closeout か次の packet の編集で行う（PR #203 の Follow-up）。
