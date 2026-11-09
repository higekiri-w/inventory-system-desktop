# Plan Packet: 衝突する PR の L3 の順・file:行 の引用・decision-log の Status・設計の考え方の規則を正本にする

docs 全体の棚卸し（owner 2026-10-18、計画は owner 2026-10-19、repo 外の回答台帳 TD-327）の wave の lane「進め方と書き方の規則」。並走の lane は `agent/docs-contract-refresh`（lane 1、契約の文を今に合わせる）。file:行 の引用の書き換えと検査は、この後に起こす棚卸しの lane 2 が持つ。

## Workflow State

- Phase: archive
- Risk: R3
- Plan Commit: 4d33d2df14fe3a61918bb2b88da9e5c2ae5a5caa
- Amendments: none
- Coordinator: Opus 5.5（Claude Code main session）
- Writer: Opus 5.5 subagent
- Plan Reviewer: fresh Opus 5.5（`subagent_type: reviewer`）+ Codex（model は発注時に決める）
- Final Reviewer: Fable 5.1 + Codex
- Final Review Minimum: 2
- Human Gate: ready,merge
- Branch: agent/docs-writing-rules
- kickoff → spec-check → plan-draft（2026-10-19、起草役、`131c0809`）: owner の決定 TD-324・TD-326・TD-327 で 3 つの規則の中身が決まり、Risk は R3（`docs/DEV_WORKFLOW.md` を変える workflow gate の変更。`scripts/ci/classify-changes.sh` の workflow の一覧に `docs/DEV_WORKFLOW.md` が当たり `workflow=true`）、Final Review Minimum 2。spec-check → plan-draft の skip で進めた
- plan-draft → design → plan-draft（2026-10-19、起草役、`a84994e5`）: round 1 の是正（Review Response 参照）と、owner の決定 TD-328 による D-127 の追加。spec-check → plan-draft の skip を取り消して design へ戻り、design の出力の D-124〜D-127（durable decision）を plan-first の変更として `docs/decision-log.md` の末尾に置いた。未解決の設計の問いは無い。実装の commit に残るのは、置き場（DEV_WORKFLOW・DOC_STYLE_GUIDE・decision-log の頭・AGENTS.md・CLAUDE.md）へ規則の文を入れることだけ
- plan-draft（2026-10-19、起草役、`91e00bbd`）: round 2 の是正（Review Response 参照）。Phase は plan-draft のまま。実装の commit の置き場に `docs/quality/review-checklist.md`（D-125 への経路）と `docs/backlog.md`（1 項）を足した
- plan-draft（2026-10-19、起草役、`4d33d2df`）: round 3 の是正（Review Response 参照）。owner の決定 TD-329 で D-124 を先頭で頼む形に置き換えた。Phase は plan-draft のまま
- plan-draft → plan-gate（2026-10-19、起草役、`a23c51a9`）: Plan Review round 1〜3（Claude 側 fresh Opus 5.5・Codex〈round 1 は Sol が満員で Astra、round 2・3 は Sol〉）。packet と Test Design Matrix は plan-first の commit で揃っている
- plan-gate → plan-approved（2026-10-19、起草役、`a23c51a9`）: owner の決定 TD-330（round 3 の天井の後、直して追加の round なしで進める）。Plan Commit は round 3 の是正の commit `4d33d2df`。plan-first の commit は実装の commit より前に在る
- plan-approved → implementing（2026-10-19、Coordinator、本 commit。state だけ）: Plan Commit `4d33d2df` は実装の commit より前にある。実装は Writer（Opus 5.5 subagent）への発注 444 で行う

## Owner Effort Budget

- 介入回数上限: 9（既定の 6 から上げた。理由: docs の棚卸しの途中で owner の決定〈TD-328〜TD-330〉が加わり、それぞれを decision point として数えたため。TD-296 の 5a の先例と同じく、予算の改定を owner の決定と同じ問い合わせで扱った）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 9 | 8（closeout 時点。2026-10-19 の TD-324・TD-326・TD-327・TD-328・TD-329・TD-330 の 6〈decision point ごとに数える〉、Ready 1〈2026-10-20〉、merge 1〈2026-10-20〉） | 0 | 1 | 9 = 8 + 0 + 1 |

承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md` `## Wave Operation` に L3 の頼み方の規則を、`AGENTS.md` `## Working Rules` に設計の考え方を足す。どちらも workflow の正本の変更で、DEV_WORKFLOW の Risk Tiers の「workflow gate」に当たる（classifier `workflow=true`）。runtime・DB・wire・script は変えない。

## Goal

Goal Invariant:

### 最小完了条件

- 次の 4 つの規則が、それぞれ 1 か所の正本に、読み手がそのまま従える文で在る: (1) L3 は merge train の先頭で最新の main を取り込んだ head で頼み、束ねるのは先に merge する全ての PR と path を共有しない PR だけ（D-124）、(2) 設計の正本と code の comment は行番号で引かず symbol か節の ID で引く（D-125）、(3) decision-log の `Status` は決定の状態だけを書く（D-126）、(4) 既存を出発点として扱い設計を継ぎ足しで歪ませず、使い手に押しつけない（D-127）
- 決定の記録 D-124〜D-127 が `docs/decision-log.md` の末尾に在り、理由と棄却案を持つ（plan-first の変更で置いた）
- 4 つの規則と矛盾する live な規則の文が repo に残っていない

### 失敗定義

- 規則が 2 か所以上に書かれる、または置き場が読み手の通る経路から外れて読まれない
- D-124 が manual 再利用の機械条件（[merge-evidence](../../agent-guidance/merge-evidence.md)「base同期だけでheadが変わる場合」と helper の `reuse_geometry`）より緩く、L3 を持つ PR が base 同期の後にまた L3 を要する（L3 の後に共有の path を触る PR が先に merge する、L3 済みの PR を後ろへ回す、L3 の後に 2 回以上 base 同期する、L3 の head が `origin/main` を含まない）。または束ねるときの確かめ方の command の失敗を「共有なし」と誤る
- D-125 が既存の時点つきの記録・data の行番号・review の一時出力まで禁じ、守れない規則になる。または review の経路から辿れず読まれない
- D-126 が lane 1 の Status の直しと食い違う語・書式を定める
- D-127 が安全の境界（D-098 の列挙・既存テストを弱めない規則を含む）まで「置き換え・削除」で緩めてよいと読める、または `AGENTS.md` Safety の「小さく、確認可能な変更を優先する」と食い違ったまま残る

### 非目的

- 既存の file:行 の引用の書き換えと、それを止める検査（lane 2）
- 既存の D の Status の直し（lane 1）
- 並走の本数・共有 file の Ready をずらす規則（`docs/backlog.md` の「並走の本数と共有 file の Ready をずらす規則を正本にする」。owner の決定が無い）
- script・hook・CI の変更

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

workflow の変更なので、規則ごとに通常の作業の列で書く。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 同じ便に L3 を持つ PR B が Draft で並ぶ（他の PR の有無を問わない） | Coordinator は B が merge train の先頭に来たときに、B の head が `origin/main` を含むことを確かめ（含まなければ base 同期）、その head で L3 を頼む。複数の PR を束ねたいときは、束ねる各 PR と、それより先に merge する全ての PR の path の共有を Wave Operation の手順で確かめる | owner は B の L3 を 1 回だけ受ける。B の L3 の後は他の PR を B より先に merge せず、B は base 同期せずに merge へ進む。束ねた PR は、それぞれが先頭になったときに `origin/main` を 1 回だけ取り込み、manual を再利用する | owner の L3 の結果（PASS / FAIL）が B の現在の head に対して記録され、束ねた PR では base 同期の後も manual の再利用の機械条件を満たす | Contract Probe P1〜P4 |
| 束ねるときの確かめ方の command の 1 本が失敗する（ref の取得漏れ・PR の番号の誤り） | 手順は各 PR の終了値を確かめ、失敗したら比べない | 「比べられない」として束ねず、先頭で 1 本ずつ頼む既定に戻る | 失敗の原因を直して取り直す | Contract Probe P3 |
| Writer が設計書か code の comment で code の箇所を引く | DOC_STYLE_GUIDE §1 の新しい節に従い、path と symbol（文書は節の ID）で書く | 後の変更で行が動いても、読み手が `rg` で同じ箇所へ着く | Plan Review・Final Review が行番号の引用を finding にできる | 無し |
| Coordinator が新しい D を decision-log に足す | owner の決定の前は `Status: proposed`、決定の後は `accepted`。実装の merge では Status を変えない | 読み手が Status から決定の状態だけを読み、実装の状態は code・Plans で見る | 全体が置き換わったら Status を `superseded（by D-xxx。…）` にして `- Superseded by:` の行、一部なら `- Superseded in part by:` の行を足す | 無し |
| 新しい事実で正本か実装を直す（Writer・Coordinator） | 条件や例外を足す前に、置き換え・削除で素直な形に戻せないかを見る。発注と review が「この変更で不要になる記述・条件は無いか」を問う | 正本と実装が継ぎ足しの例外で読みにくくならず、店・owner に手作業の回避が回らない | review が残った継ぎ足し・手作業の回避を finding にできる。安全・データ保護・外部書込みの承認の境界は変えない | 無し |

通常の列（発注 → 停止と訂正 → review → owner 判断）: plan-draft の packet を Plan Review（fresh Opus 5.5 + Codex）に出し、P1/P2 = 0 で plan-approved。実装の run が Scope の置き場へ規則の文を入れ、Final Review 2 本（Double Audit）の後、owner の Ready・merge の判断を待つ。

## 規則の文と置き場

実装の run はこの節の文を置き場へ書く（語の小さな直しはよいが、規則の中身を変えるなら Gated Amendment）。同じ規則を 2 か所に書かない。決定・理由・棄却案は decision-log の D-124〜D-127（plan-first で置いた）が持ち、規則の文と手順は置き場の 1 か所だけが持つ。

### D-124 の置き場: `docs/DEV_WORKFLOW.md` `## Wave Operation`

置き場の理由: owner の決定（TD-324）が置き場を `## Wave Operation` と指定した。同節は便の PR の並列・Ready の順・base 同期を持ち、L3 の頼み方はその間の手順になる。既存の箇条「先頭 lane の merge 後、後続 lane は main との同期を…」の直後に 1 つの箇条として足す。round 1〜3 で足した「束ねる前に確かめる」「確かめ直す」「既定案は L3 を持つ PR を先に」の形は、owner の決定 TD-329 で「先頭で頼む形」へ置き換えた（D-127 の (a)。足した句を残して追記しない）。同じ節の Ready の箇条の「Coordinator は既定案として review 通過順を提示する。」は、既定の train 順と L3 の関係を示すため、次の 1 句を足して D-124 の箇条を指す（規則は D-124 の箇条だけが持つ）。

```markdown
Coordinator は既定案として review 通過順を提示する（L3 を済ませた PR を後ろへ回す順にしない。L3 は下の D-124 の箇条のとおり先頭で頼む）。
```

D-124 の箇条:

```markdown
- L3（manual）は、その PR が merge train の先頭に来て、最新の `origin/main` を取り込んだ head で頼むのが既定で、その後は他の PR を先に merge しない。L3 を頼む前に、その PR の head が `origin/main` を含むこと（`git merge-base --is-ancestor origin/main SHA` の終了値が 0）を確かめ、含まなければ先に base 同期する。複数の PR の L3 を 1 回の依頼で束ねてよいのは、束ねる各 PR が、自分より先に merge する全ての PR（同じ便・closeout・便の外を問わない）と path を共有せず、各 PR が自分が先頭になったときに `origin/main` を 1 回だけ取り込む場合に限る。どちらでも、L3 の後に train の順を変えて L3 済みの PR を後ろへ回さない。束ねるときの path の共有は L3 を頼む時点の head で確かめる: `git fetch origin` の後、各 PR の `git rev-parse origin/BRANCH` が `gh pr view N --json headRefOid -q .headRefOid` と一致することを確かめ、PR ごとに `git diff --name-only --no-renames origin/main...SHA > FILE` の終了値が 0 であることを確かめ、全ての PR で成功したときだけ `comm -12 <(sort FILE_A) <(sort FILE_B)` で比べる（出力が空なら共有なし）。どれかが失敗したら比べられないとして、束ねない。manual を持つ lane は、backlog の完了の整理のように設計の契約でない共有の更新を closeout へ寄せ、共有の path を減らす。防ぐ失敗: helper（`scripts/pr-gate.py` の `reuse_geometry` と `record` の manual の再利用）が manual を再利用できるのは、L3 の head に `origin/main` を 1 回 merge しただけの head で、PR 差分の全体が byte で一致するときの 1 回だけで、再利用した manual の再利用は拒む。共有の file は base 同期で pre-image の blob が変わり `--full-index` の index の行が必ず変わるので、L3 の後に共有の path を触る PR が先に merge するか、L3 の後に 2 回以上 base 同期すると、L3 が 2 回になる（第 2 便の PR #171・#176。D-124）。
```

### D-125 の置き場: `docs/DOC_STYLE_GUIDE.md` `## 1. 参照の書き方`

置き場の理由: §1 は設計書と code の doc comment の参照の書き方を持つ既存の節で（Rust の doc comment・Markdown リンク・§ 記号）、行番号の引き方はその続きになる。DEV_WORKFLOW は作業の進め方を持ち、書き方の規則を置くと §1 と 2 か所に分かれる。decision-log の頭は決定の記録の書式の場所で、合わない。§1 の最後（「タスクID・要件ID」の後）に節を 1 つ足し、file の頭の「対象」の行に範囲を 1 句足す。

```markdown
### code と文書の箇所の引き方（行番号で引かない）

設計の正本（`docs/` の設計書・workflow の正本・runbook）と code の comment（`src/`・`src-tauri/src/`）は、箇所を行番号（`path:N`・`` `:N` ``・`#LN`）で引かない。行番号は code や文書の変更で動き、読み手を別の箇所へ導く（D-125）。

- code は repo root からの path と symbol（関数・型・定数・test の名前）で引く。範囲は symbol と、その中の目印の語で書く
- 文書は節の ID（`§12.3`・`BIZ-01-D8`・`DSR-22` 等）か節の見出しで引く
- `docs/backlog.md` の申し送りで行番号を使うときは、書いた時点の main の commit を同じ行に添える（例: 「`18db621e` の時点で `src-tauri/src/biz/product_service.rs:965`」）。`docs/Plans.md` は現在地を持つ所で、行番号の申し送りを置かない
- 時点を書いた記録（commit・library の版・日付を書いた観測、dated の調査・提案・L3 の記録、decision-log の append-only の本文）の行番号は、その時点の観測としてそのまま残す
- 対象外: dated の Plan Packet と Test Design Matrix（起票の時点の観測）、data の行番号（CSV・EJ の `line_no` 等の製品の契約）、review の finding と subagent の報告の `path:line`（その時点の版に対する一時の出力。[code_review.md](code_review.md)、[DEV_WORKFLOW.md](DEV_WORKFLOW.md) の Subagent Budget）
```

file の頭の「対象」の行は「docs/ 配下の全設計書、src-tauri/src/ 内の doc comment（§1 の箇所の引き方は `src/` の comment と backlog の申し送りも対象）」にする。

読み手の経路: review の観点の `docs/quality/review-checklist.md` `### 8. 設計-コード整合` に、規則を写さない 1 行の参照を足し、同 file の `## 更新履歴` に 1 行足す。`docs/templates/` には Writer の発注の雛形が無く（雛形は repo の外）、Plan Packet・Test Design Matrix の雛形は D-125 の対象外の dated の記録なので、参照を足さない。

```markdown
- [ ] 設計書・code の comment が code と文書の箇所を行番号で引いていないか（引き方は [DOC_STYLE_GUIDE.md](../DOC_STYLE_GUIDE.md) §1 の箇所の引き方。D-125）
```

```markdown
| 2026-10-19 | PR #179 | カテゴリ 8 に箇所の引き方（D-125）の確認行を追加。 |
```

### D-126 の置き場: `docs/decision-log.md` の頭（`## Purpose` の後）

置き場の理由: Status は decision-log の各 D の書式で、D を足す人が最初に読むのは file の頭の `## Purpose` になる。DEV_WORKFLOW の Design Phase Rules は decision-log を設計の成果物として選ぶ所までを持ち、D の書式は持たない。DOC_STYLE_GUIDE は設計書（function-design・db-design・architecture）の型を持ち、decision-log の型を持たない。`## Purpose` と `## D-001` の間に節を 1 つ足す。全体の置換えの書式は lane 1 の適用（D-116 の形）にそろえる。

```markdown
## Status の書き方

- `Status` は決定の状態だけを書く: `accepted`（決まった）、`superseded`（全体が置き換わった）、`withdrawn`（取り下げた）。`proposed` は owner の決定の前だけに使う。括弧に出所（owner の決定の日付・repo 外の回答台帳の ID・design lane）を添えてよい。superseded は `superseded（by D-xxx。元の出所）` と書く
- 実装が済んだか、どの lane・PR が反映するかは code と `docs/Plans.md`・Plan Packet が持ち、Status に書かない
- 決定の本文（Decision・Why・Alternatives ほか）は、main に merge された後は append-only で書き換えない（同じ PR の merge の前の直しは対象外）。全体が置き換わったときは Status を `superseded（by D-xxx。元の出所）` にし、本文の後に `- Superseded by: D-xxx（日付）。置き換わった中身` の行を足す。一部が置き換わったときは `- Superseded in part by: D-xxx（置き換わった所）` の行を足す。後から直してよいのは Status の行（状態の語と出所）と、Status の行が無い D への Status の行の追加と、Superseded の行の追加と、file の移送に伴う link の path の直しだけ
- 理由と棄却案は D-126
```

### D-127 の置き場: `AGENTS.md` `## Working Rules`

置き場の理由: owner の決定（TD-328）が置き場を `AGENTS.md` と指定した。`AGENTS.md` は Claude と Codex の共通の入口で、`## Working Rules` は作業の進め方の原則を持つ。この repo の `CLAUDE.md` は Claude だけの補助で共通の条件を変えない場所なので置かない。`## Working Rules` の箇条の最後（「Plan Packet の数値主張は…」の後）に 3 つの箇条を足す。owner 2026-10-15 の「設計を継ぎ足しで歪ませない」と owner 2026-09-24 の「規則は環境が変わるたびに変える」は TD-328 と同じ向きなので 1 つ目の箇条にまとめる。委任の範囲（memory にあった方針）は Decision and Approval Boundaries に既にあるので足さない。

```markdown
- 継ぎ足さない（D-127）: 既存の実装・文書・規則は出発点であって制約ではない。新しい事実や owner の決定と合わなければ、条件や例外を継ぎ足して合わせる前に、置き換え・削除で全体を素直な形に戻せないかを見る。既存を残す理由は「今も正しいから」で、「既にあるから」ではない。旧い環境で作った規則が owner の今の決定と衝突したら、止めずに改訂の対象として扱い、packet・PR に owner の決定の日付を書く。発注と review の問いに「この変更で不要になる記述・条件は無いか」を入れる。安全の境界（Decision and Approval Boundaries・Safety・npm 供給網ガード、[DEV_WORKFLOW](docs/DEV_WORKFLOW.md#workflow-state) の Workflow State が D-098 で挙げる安全の境界〈古い証拠の拒否・独立 review・Plan Commit の固定・不可逆 finding の 4 項目・fail-closed ほか〉、既存テストを不都合だから削除・skip・弱体化しない規則）はこの考え方で緩めず、それらの改訂は通常の packet・PR の経路で行う。
- 使い手に押しつけない（D-127）: 例外の回避を店・owner の手作業（別の file を作る等）に回さない。問題は黙って通さず、起きたその画面で原因と直し方を名指しし、その場で直せるようにする。
- owner の所感は原文で扱う（D-127）: L3 の所感・要望は原文を repo の外の回答台帳に残し（公開 repo には ID で引く）、Codex・subagent の要約を「owner の意図」として裁定しない。AI の review から出た要件（製品の要件。Contract Audit が L3 の項目にする検証は除く）は出所を書き、owner の確認の前に Human Gate の必須にしない。
```

(f) の置き場は `## Decision and Approval Boundaries` の「owner へ送り、中継を省かない問い合わせ」の箇条の直後（問い合わせの行き先の規則の隣）。

```markdown
- 店主への問いは GitHub の Issue に書かず、owner への問い合わせで送り、回答は repo の外の回答台帳に残す。Issue は店の PC の Claude も読む現場の手順書にする（D-127）。`docs/AGENT_OPERATING_MANUAL.md` §5.1 の調査項目の issue は、その手順書の形（場所・操作・合格基準）に限る。
```

両立の句: `## Safety` の「小さく、確認可能な変更を優先する。」を次の文にする（同じ規則を 2 か所に書かず、Working Rules の箇条を指す）。

```markdown
- 小さく、確認可能な変更を優先する。小ささは差分の行数でなく、確かめやすさで測る（継ぎ足しより置き換え・削除のほうが小さいことがある。Working Rules の「継ぎ足さない」）。
```

`docs/code_review.md` の finding の形の `smallest safe fix`（と DEV_WORKFLOW の Commit / PR Messages の同じ句）は finding の修正案の形で、置き換え・削除の修正案も含むので変えない（D-127 の Compatibility）。

### D-127 の Claude だけの運用の置き場: この repo の `CLAUDE.md`

置き場の理由: (g)(h) は Claude の返答の書き方と Claude の subagent の経路の話で、Codex には当たらない。(h) は Final Review の本数の数え方を変えず（座組表と DEV_WORKFLOW の Double Audit のまま）、枠の扱いを共通の正本で決める項目を backlog に足す（下）。`CLAUDE.md` は Claude だけの補助で、共通の承認・品質の条件を変えない場所。(g) は `## 長い作業の報告`、(h) は `## 座組と effort` の段落の後に 1 段落ずつ足す。

```markdown
owner に渡すコピペ物（command・URL の一覧・checklist）は返答の本文に inline で書き、path は省略しない完全な形で書く（WSL は `/home/kosei/...`、Windows は `C:\Users\Owner\...`、Windows から WSL の file は `\\wsl.localhost\Ubuntu-22.04\home\kosei\...`。D-127）。
```

```markdown
安全境界（safe wrapper・path traversal・repo の外の参照・execpolicy）の敵対的な検証は Claude の subagent に発注しない。発注の分野そのものが dual-use の安全の判定に掛かり、model が切り替わって止まる（PR #15 で語の言い換えでも再現した）。この分野の review は Coordinator が source を直接読む inline の review か、Codex への発注で行う。Coordinator の inline の review は fresh context でないので Final Review の本数に数えない。この分野の lane の Claude 側の枠の扱いは、共通の正本で決めるまで、その lane の起票で owner に諮る（D-127。backlog に項目がある）。
```

### backlog に足す 1 項（`docs/backlog.md` `#### やると決めたもの（順番未定）`）

置き場: 項目「同じ便で実装 file が衝突する PR の L3 の頼み方を Wave Operation に足す」の直後（lane 1 の S5 が閉じる行・S6 が照合する項目に当たらない位置）。

```markdown
- **安全境界の lane の Final Review の Claude 側の枠を共通の正本で決める**（D-127、2026-10-19。起源: docs の棚卸しの lane「進め方と書き方の規則」の Plan Review round 2）: 安全境界（safe wrapper・path traversal・repo の外の参照・execpolicy）の敵対的な検証は Claude の subagent に発注できず（`CLAUDE.md` `## 座組と effort`）、Coordinator の inline の review は fresh context でない。この分野の lane の Claude 側の枠を、座組表（`docs/AGENT_OPERATING_MANUAL.md` `## 座組`）か DEV_WORKFLOW の Double Audit で決める。
```

### 決定の記録（`docs/decision-log.md` の末尾、plan-first で置いた）

D-127 は owner の決定 TD-328（2026-10-19）で本 lane に足した（Review Response 参照）。D-124・D-125・D-126・D-127 は本 lane が持つ番号で（[Review Rules](../../DEV_WORKFLOW.md#review-rules) の連番 registry。並走の lane 1 は番号を新設しない）、D-123 と同じ項目（Status・Context・Decision・Why・Alternatives・Guarantee range・Compatibility・owner の決定・Revisit）で書いた。D の本文は置き場を名指し、規則の文と手順を写さない（AC5）。D-124 の Decision は、判定を merge の衝突から path の共有へ広げたこと（起票の Writer の Contract Probe の結果）と、先頭で頼む形へ置き換えたことを owner が TD-329 で決めたこと（owner の決定の目的は L3 を 2 回にしないこと）を書く。

## Scope

plan-first の変更（本 commit まで）: `docs/decision-log.md` の末尾の D-124〜D-127、本 packet、Test Design Matrix。

実装の run（plan-approved の後）が直す file と目的:

- `docs/DEV_WORKFLOW.md` `## Wave Operation`: D-124 の箇条を 1 つ足し、Ready の箇条の既定案の文に 1 句足す（上の文）
- `docs/DOC_STYLE_GUIDE.md`: §1 の最後に D-125 の節を足し、file の頭の「対象」の行に 1 句足す（上の文）
- `docs/quality/review-checklist.md`: `### 8. 設計-コード整合` に D-125 への参照の 1 行、`## 更新履歴` に 1 行（上の文）
- `docs/backlog.md`: 安全境界の lane の Final Review の枠の 1 項を足す（上の文）
- `docs/decision-log.md`: `## Purpose` と `## D-001` の間に `## Status の書き方` を足す（先頭の BOM を保つ）。D-124〜D-127 は変えない
- `AGENTS.md`: `## Working Rules` の最後に D-127 の箇条を 3 つ、`## Decision and Approval Boundaries` に (f) の箇条を 1 つ足し、`## Safety` の「小さく、確認可能な変更を優先する。」に両立の句を足す（上の文）
- `CLAUDE.md`: `## 長い作業の報告` に (g)、`## 座組と effort` に (h) の段落を 1 つずつ足す（上の文）
- 本 packet の Workflow State と `## Implementation Results`

隣接の確認（変えない）: `docs/agent-guidance/merge-evidence.md`「base同期だけでheadが変わる場合」（D-124 が引く条件）、DEV_WORKFLOW の `### Stacked train`・Subagent Budget・Commit / PR Messages の `path:line`、`docs/code_review.md` の finding の形、`docs/AGENT_OPERATING_MANUAL.md` の Explorer と報告フォーマットの `file:line`、`docs/templates/adr.md` の Status、`scripts/doc-consistency-check.sh`（新しい h2 の節と箇条は PK・M・R の検査の対象を変えない）。

### 並走の lane との所有表

| file | 本 lane | lane 1（`agent/docs-contract-refresh`） |
|---|---|---|
| `docs/decision-log.md` | 頭（`## Purpose` と `## D-001` の間に置く節 `## Status の書き方`）と、末尾に足す `## D-124`〜`## D-127` | 既存の D の `- Status:` の行（Status の行が無い D に足す行を含む）と、足す `- Superseded by:` の行（lane 1 の S3） |
| `docs/backlog.md` | 1 項を足すだけ（「同じ便で実装 file が衝突する PR の L3 の頼み方…」の項目の直後。lane 1 の整理と別の位置） | S5 の整理（閉じる項目の移送・見出しの直し）と S6 の照合 |

merge の順はどちらが先でもよい。後の lane は `origin/main` の単段 merge で両方の行を残す（Wave Operation）。lane 1 が先に merge して backlog の項目の位置が動いたら、本 lane は同じ項目の直後に足し直す。どちらの lane も Human Gate に manual を持たないので、D-124 の L3 の順は当たらない。本 lane の他の file（DEV_WORKFLOW・DOC_STYLE_GUIDE・`docs/quality/review-checklist.md`・`AGENTS.md`・`CLAUDE.md`・本 packet と matrix）は lane 1 の対象外（lane 1 は workflow の正本〈`AGENTS.md`・`CLAUDE.md`・`docs/quality/review-checklist.md` を含む〉を触らず、DOC_STYLE_GUIDE は同 lane の調査で直す対象に無い）。

## Non-scope

- 既存の file:行 の引用の書き換え・file の頭の時点の注記・検査の追加（棚卸しの lane 2）
- 既存の D の Status の直しと Superseded の行の追加（lane 1）
- `docs/backlog.md` の項目「同じ便で実装 file が衝突する PR の L3 の頼み方を Wave Operation に足す」を外すこと（本 lane の closeout で外す。本 lane の PR は backlog に 1 項を足すだけで、完了の整理は D-124 (3) の「共有の更新を closeout へ寄せる」と同じ向きで closeout へ寄せる）
- `docs/backlog.md` の「並走の本数と共有 file の Ready をずらす規則を正本にする」（owner の決定が無く、本 lane の D-124〜D-127 に入っていない。backlog に残す）
- stacked train の後続 lane の L3 の頼み方、ADR の Status の語、`docs/Plans.md` の lane の行（D-097）
- script・hook・CI・runtime の変更
- Coordinator の個人の auto-memory から上げた方針の項を外すこと（repo の外。Coordinator が行う）、repo の外の発注書の雛形への問いの追加、owner の個人の `~/.claude/CLAUDE.md`
- D-127 を既存の正本へ一斉に当てはめ直すこと（触る所から適用する。D-127 の Guarantee range）
- 安全境界の review の Double Audit の数え方を変えること（座組表と DEV_WORKFLOW のまま。変えるなら共通の正本で別に決める）

## Acceptance Criteria

AC の command は bash で、repo root で逐語に実行する（zsh では `$R` が 1 語のまま渡り exit 2 になる）。先に bash で `R="AGENTS.md CLAUDE.md docs/DEV_WORKFLOW.md docs/AGENT_OPERATING_MANUAL.md docs/DOC_STYLE_GUIDE.md docs/ci.md docs/templates docs/agent-guidance docs/code_review.md docs/quality/review-checklist.md docs/adr/README.md .agents/skills/inventory-workflow-start .agents/skills/inventory-implementation .agents/skills/inventory-code-review .agents/skills/inventory-operator-ui"` と `G='--glob !docs/plans/** --glob !docs/archive/**'`（対話の bash の history 展開を避けて single quote）を置く。baseline は本 commit の tree（D-124〜D-127 を置いた後、Scope の置き場へ規則の文を入れる前）で、bash で逐語に実行した出力。

- AC1 D-124 の置き場: `awk '/^## Wave Operation/,/^### Stacked train/' docs/DEV_WORKFLOW.md | rg -c 'name-only --no-renames'` が `1`、同じ範囲の `rg -c 'merge-base --is-ancestor'` が `1`、`rg -c 'gh pr view'` が `1`、`rg -c '先頭に来て'` が `1`、`rg -c 'L3 を済ませた PR を後ろへ回す順にしない'` が `1`（Ready の箇条の 1 句）、`rg -c 'merge-tree'` と `rg -c '確かめ直す'` と `rg -c 'manual の有無を問わない'` が出力なし（exit 1。round 1〜3 の形が残っていない）。baseline: 8 本とも出力なし・exit 1
- AC2 D-125 の置き場: `rg -n '^## 1\. 参照の書き方|^### code と文書の箇所の引き方|^## 2\. ' docs/DOC_STYLE_GUIDE.md` で新しい節の行番号が §1 と §2 の見出しの間にある。baseline: `35:## 1. 参照の書き方` と `83:## 2. 関数設計書テンプレート（function-design/） ✅M2` の 2 行
- AC3 D-126 の置き場: `rg -n '^## Status の書き方|^## D-001$|^## Purpose' docs/decision-log.md` が `## Purpose`・`## Status の書き方`・`## D-001` の順の 3 行。baseline: `3:## Purpose` と `8:## D-001` の 2 行
- AC4 決定の記録: `rg -c '^## D-12[4567]: ' docs/decision-log.md` が `4`。`rg -A2 '^## D-12[4567]: ' docs/decision-log.md | rg -c '^- Status: accepted（owner の決定、repo 外の回答台帳 TD-32[4678]、2026-10-19'` が `4`。`rg -A2 '^## D-12[4567]: ' docs/decision-log.md | rg '^- Status:' | rg 'runtime|実装の PR|merge で'` が出力なし（exit 1）。`rg -n '^- Status: superseded by ' docs/decision-log.md` が出力なし（exit 1。全体の置換えは `superseded（by D-xxx。…）` の形）。baseline: 本 commit で `4`・`4`・出力なし・出力なし
- AC5 1 か所だけ（正本の中の出現の数を数え、指定の位置の 1 か所と照合する。`rg -l` の file 数でなく `rg -n -o` の出現の行で見る。literal に `)` を含むので `-F` で探す。正規表現にすると exit 2）: 次の 15 本の `rg -n -o -F 'LITERAL' docs AGENTS.md CLAUDE.md $G` がそれぞれちょうど 1 行を出し、その行が指定の位置に在る:
  - `name-only --no-renames` → `docs/DEV_WORKFLOW.md`、行番号が `## Wave Operation` と `### Stacked train` の見出しの間
  - `code と文書の箇所の引き方` → `docs/DOC_STYLE_GUIDE.md` の新しい節の見出し
  - `書いた時点の main の commit を同じ行に添える` → `docs/DOC_STYLE_GUIDE.md`、行番号が新しい節の見出しと `## 2. ` の間
  - `## Status の書き方` → `docs/decision-log.md`、行番号が `## Purpose` と `## D-001` の間
  - `後から直してよいのは Status の行` → `docs/decision-log.md`、行番号が `## Status の書き方` と `## D-001` の間
  - `継ぎ足さない（D-127）` と `押しつけない（D-127）` → それぞれ `AGENTS.md`、行番号が `## Working Rules` と `## Decision and Approval Boundaries` の見出しの間
  - `差分の行数でなく` → `AGENTS.md`、行番号が `## Safety` と `## npm 供給網ガード（D-030）` の見出しの間
  - `所感は原文で扱う（D-127）` → `AGENTS.md`、行番号が `## Working Rules` と `## Decision and Approval Boundaries` の見出しの間
  - `現場の手順書にする（D-127）` → `AGENTS.md`、行番号が `## Decision and Approval Boundaries` と `## Workspace Access` の見出しの間
  - `返答の本文に inline で書き` → `CLAUDE.md`、行番号が `## 長い作業の報告` と `## 座組と effort` の見出しの間
  - `Final Review の本数に数えない` → `CLAUDE.md`、行番号が `## 座組と effort` と `## 再開とmemory` の見出しの間
  - `それらの改訂は通常の packet・PR の経路で行う` → `AGENTS.md`、行番号が `## Working Rules` と `## Decision and Approval Boundaries` の見出しの間
  - `Claude 側の枠を共通の正本で決める` → `docs/backlog.md`、項目「同じ便で実装 file が衝突する PR の L3 の頼み方…」の次の行
  - `DOC_STYLE_GUIDE.md) §1 の箇所の引き方` → `docs/quality/review-checklist.md`、行番号が `### 8. 設計-コード整合` と `### 9. Operator UI visibility` の見出しの間
  - baseline: 15 本とも出力なし・exit 1（D-124〜D-127 の本文はこれらの literal を持たない）
- AC6 矛盾する live な規則の文が無い（下の 5 本の `rg` の各 hit を読んで判定する）:
  - D-124: `rg -n 'L3.{0,30}束ね|束ね.{0,30}L3' $R` が DEV_WORKFLOW `## Wave Operation` の新しい箇条の行だけ。baseline: 出力なし・exit 1
  - D-125: `rg -n 'file:line|path:line|file:行' $R | cut -d: -f1,2` が、baseline の 5 行（`docs/DEV_WORKFLOW.md:198`〈Subagent Budget〉・`docs/DEV_WORKFLOW.md:351`〈Commit / PR Messages〉・`docs/code_review.md:98`・`docs/AGENT_OPERATING_MANUAL.md:21`・`docs/AGENT_OPERATING_MANUAL.md:152`。どれも review・subagent の一時の出力）と、DOC_STYLE_GUIDE の新しい節の対象外の行だけ（DEV_WORKFLOW の行番号は Wave Operation の追記で動いてよい）
  - D-126: `rg -n 'accepted にする|merge で accepted' $R` が出力なし・exit 1（baseline も同じ）。`docs/decision-log.md` の既存の D の Status の行の同じ語は lane 1 の所有（所有表）で、本 lane の AC に入れない
  - D-127: `rg -n '継ぎ足|不要になる|押しつけ|押し付け|尊重|小さな変更|小さく|smallest|既存の実装|既存を|黙って通さ' $R | cut -d: -f1,2` が、baseline の 3 行（`AGENTS.md:65`〈Safety の「小さく、確認可能な変更を優先する」〉・`docs/code_review.md:98` と `docs/DEV_WORKFLOW.md:351`〈finding の修正案の `smallest safe fix`。D-127 の Compatibility で変えない〉）と、`AGENTS.md` の D-127 の箇条の行だけ（`AGENTS.md` と DEV_WORKFLOW の行番号は追記で動いてよい）。Safety の行は両立の句を持つ
  - D-127 の (d)〜(h): `rg -n -i '環境が変わる|原文|issue|コピペ|dual-use|敵対的' $R | cut -d: -f1,2` が、baseline の 14 行と、`AGENTS.md`・`CLAUDE.md` に足した D-127 の行だけ（行番号は追記で動いてよい）。baseline の内訳: `docs/DEV_WORKFLOW.md:269`（reviewer の scope の提案を owner の原文と照合する。(e) と同じ向き）、`docs/DEV_WORKFLOW.md:351`・`docs/code_review.md:33`・`docs/code_review.md:98`・`docs/code_review.md:107`（finding の語の issue）、`AGENTS.md:44`・`AGENTS.md:45`（issue comment の外部書込み）、`docs/AGENT_OPERATING_MANUAL.md:104`（§5.1 の調査項目を GitHub issue に L3 checklist の形で置く。現場の手順書なので (f) と両立）、`.agents/skills/inventory-code-review/SKILL.md:82`・`:83`・`:84`・`:85`・`:89`・`:91`（GitHub の API の issue comment の endpoint と encoding の語）。委任の範囲は Decision and Approval Boundaries の既存の項のままで、同じ趣旨の箇条を足していない
- AC7 BOM: `head -c3 docs/decision-log.md | od -An -tx1` が ` ef bb bf`（baseline も同じ）
- AC8 検査: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-10-19-docs-writing-rules.md` が exit 0、`bash scripts/check-workflow-git.sh` が exit 0、Draft の hosted run（`Draft (no merge evidence)`）が success

## Design Readiness

設計の完了条件は `docs/DEV_WORKFLOW.md` Design Phase Rules の Design completion criteria。当たらない箇条は 1 行の理由で閉じる。

- 引用する設計正本（節まで）: [DEV_WORKFLOW](../../DEV_WORKFLOW.md) `## Wave Operation`・`### Stacked train`・Review Rules の連番 registry と Double Audit・Workflow State の Evidence Ownership（D-038）・Plan Packet Rules の Evidence Ownership の design-document extension（D-050）・Post-Merge Closeout の `## 直近の完了`、[merge-evidence](../../agent-guidance/merge-evidence.md)「base同期だけでheadが変わる場合」、[DOC_STYLE_GUIDE](../../DOC_STYLE_GUIDE.md) `## 1. 参照の書き方`、[decision-log](../../decision-log.md) `## Purpose` と D-123 の項目の型
- 必要な設計成果物: 「Durable cross-cutting choice, workflow gate change」の行 = decision-log の D-124〜D-127（updated in this PR、plan-first の commit で置いた）。ほかの行（BIZ・DTO・DB・画面・CSV）は当たらない
- plan にしかない durable な判断の昇格先: 決定・理由・棄却案は decision-log の D-124〜D-127（置いた）。規則の文は置き場の正本で、同じ PR の実装の commit で書き、merge の前に揃う
- 前提・制約と、延期した design gap の follow-up: 既存の引用の書き換えと検査は lane 2、既存の D の Status の直しは lane 1、並走の本数の規則は backlog に残す（Non-scope）
- 絶対保証（cannot happen / always happens）の例外と escape hatch の自己点検: D-124 の「path を共有せず `origin/main` を 1 回だけ取り込めば PR 差分は byte で変わらない」は Contract Probe P2 と helper の `reuse_geometry` の現物で確かめた性質で、manual 再利用の必要条件にとどめ、owner の判断を残す。先頭で頼む既定では L3 の後の base 同期が無く、比べる相手の漏れが起きない。束ねる場合の比べる相手は先に merge する全ての PR（便の外を含む）
- 判定（ready / not ready）と理由: ready。規則の中身は owner の決定 TD-324・TD-326〜TD-329 ほか（D-127 の Status の出所）で決まり、design の出力の D-124〜D-127 は plan-first の変更で decision-log に在る。前提は Contract Probe P1〜P3 で確かめた

## Registration / Generation Obligations

該当なし（新しい file・route・command・REQ を作らない。DOC_STYLE_GUIDE と decision-log に足す節は既存の file の中で、目次・索引の登録は無い）。

## Impact Review Lenses

not applicable: 現地の調査・実機・外部の tool・POS・CSV の形式・operator の作業から始まる task ではなく、docs の棚卸しの調査から決まった workflow と書き方の規則の変更。外部の前提（git の振舞い）は Contract Probe で確かめた。

## Boundary / Wire Contract

not applicable: JSON・CSV・config・DTO・bindings・DB を触らない。

## Test Plan

[Test Design Matrix](test-matrices/2026-10-19-docs-writing-rules.md) を使う。L3 は無い（docs だけで operator の画面を変えない）。

- targeted tests: AC1〜AC7 の `rg`・`awk`・`od` を実装の後に bash で逐語に実行する
- negative tests: AC6 の旧い書き方の検索（矛盾する live な規則の文が 0、または既知の一時出力の行だけ）、AC4 の `superseded by ` の 0 件
- compatibility checks: `bash scripts/doc-consistency-check.sh` が exit 0（新しい h2・h3 の節が M・R・PK の検査を壊さない）、`bash scripts/check-workflow-git.sh` が exit 0
- data safety checks: 店の実値・発言の原文を書かない。owner の決定は TD の ID で引く
- main wiring/integration checks: 置き場が読み手の経路に在る（DEV_WORKFLOW は AGENTS.md の Session Start の R2+ の経路、DOC_STYLE_GUIDE は DEV_WORKFLOW の Source Index、decision-log の頭は D を足すときに開く所、`AGENTS.md` は Claude と Codex の毎 session の入口、`CLAUDE.md` は Claude の毎 session の入口）

## Review Focus

- D-124 の既定（L3 は merge train の先頭で、`origin/main` を含む head で頼む）と束ねる条件（先に merge する全ての PR と path を共有しない、先頭になったときに 1 回だけ取り込む）が、helper の manual の再利用の条件（`reuse_geometry`、再利用済みの拒否）と owner の決定の目的（L3 を 2 回にしない）に合うか（Contract Probe P1〜P4）
- D-124 の束ねるときの確かめ方の手順が、command の失敗を「共有なし」と誤らないか（Contract Probe P3）
- D-125 の申し送りの対象を backlog に絞ったことと、Evidence Ownership（D-038・D-050）・Post-Merge Closeout の「SHA を書かない」の関係
- D-126 の書式が lane 1 の適用（`superseded（by D-xxx。…）` と `- Superseded by:` の行）と同じか
- 置き場の選び方と、同じ規則を 2 か所に書いていないか（AC5）
- D-127 の (a)〜(f) を `AGENTS.md`、(g)(h) を `CLAUDE.md` に分けたことと、(h) が共通の品質の条件（座組表・Double Audit の数え方）を `CLAUDE.md` で変えていないか
- D-127 の「置き換え・削除」が安全の境界（D-098 の列挙・既存テストを弱めない規則を含む）を緩めると読めないか、`AGENTS.md` Safety の「小さく、確認可能な変更」との両立の句が足りるか
- Ready の箇条の既定案（review 通過順）の句と、D-124 の「L3 済みの PR を後ろへ回さない」が食い違わないか
- D-125 が review の経路（review-checklist）から辿れるか

## Contract Ledger

ここから Data Safety までが R3/R4 の部分。

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| D-124（新設） | DEV_WORKFLOW `## Wave Operation`、decision-log D-124 | D-124 は plan-first で置いた（TD-329 で先頭で頼む形へ置き換えた）。実装は箇条を 1 つ足し、Ready の箇条の既定案の文に 1 句 | AC1・AC5・AC6 の `rg`（matrix の行 1〜3・13・18） | L3 なし |
| D-125（新設） | DOC_STYLE_GUIDE `## 1. 参照の書き方`、`docs/quality/review-checklist.md` `### 8. 設計-コード整合`、decision-log D-125 | D-125 は plan-first で置いた。実装は節を 1 つ足し、頭の「対象」の行、review-checklist の参照の 1 行と更新履歴 | AC2・AC5・AC6 の `rg`（matrix の行 4〜6・19） | L3 なし |
| D-126（新設） | decision-log の頭（`## Purpose` の後）、decision-log D-126 | D-126 は plan-first で置いた。実装は節 `## Status の書き方` | AC3・AC4・AC5・AC6 の `rg`（matrix の行 7〜9・14・21） | L3 なし |
| D-055 Wave Operation の既存の箇条 | DEV_WORKFLOW `## Wave Operation` | Ready の箇条の既定案（review 通過順）に、L3 済みの PR を後ろへ回さない句を足すだけで、ほかは変えない | AC1 の `L3 を済ませた PR を後ろへ回す順にしない`、AC8 の doc check | 非対象 |
| merge-evidence と helper の manual 再利用の条件 | merge-evidence「base同期だけでheadが変わる場合」、`scripts/pr-gate.py` の `reuse_geometry` と `record` | 変えない。D-124 が引く | Contract Probe P2・P4 | 非対象 |
| D-074 stacked train | DEV_WORKFLOW `### Stacked train` | 変えない（D-124 の対象外） | AC1 の `awk` の範囲の終端 | 非対象 |
| D-038・D-050 Evidence Ownership | DEV_WORKFLOW Workflow State・Plan Packet Rules | 変えない。D-125 の Compatibility に書いた | 無し（文の review） | 非対象 |
| Post-Merge Closeout の `## 直近の完了` | DEV_WORKFLOW Post-Merge Closeout | 変えない（D-125 は Plans に行番号の申し送りを置かない） | 無し（文の review） | 非対象 |
| DOC_STYLE_GUIDE §1 の R0・R1・R3 | DOC_STYLE_GUIDE §1・§6 | 変えない | AC8 の doc check | 非対象 |
| D-127（新設） | `AGENTS.md` `## Working Rules`・`## Decision and Approval Boundaries`・`## Safety`、`CLAUDE.md` `## 長い作業の報告`・`## 座組と effort`、`docs/backlog.md`、decision-log D-127 | D-127 は plan-first で置いた。実装は箇条 3 つ・箇条 1 つ・両立の句・段落 2 つ・backlog の 1 項 | AC5・AC6 の `rg`（matrix の行 15〜17・20） | L3 なし |
| AGENTS.md の委任の範囲 | `AGENTS.md` Decision and Approval Boundaries | 変えない（memory の同じ方針は既にある） | AC6 の (d)〜(h) の `rg` | 非対象 |
| finding の修正案 `smallest safe fix` | `docs/code_review.md`、DEV_WORKFLOW Commit / PR Messages | 変えない（D-127 の Compatibility） | AC6 の D-127 の `rg` | 非対象 |
| 座組表と Double Audit の数え方 | AGENT_OPERATING_MANUAL `## 座組`、DEV_WORKFLOW Contract Audit | 変えない（`CLAUDE.md` の (h) は経路と、inline の review を本数に数えないことだけ。枠は backlog の 1 項で共通の正本へ） | 無し（文の review） | 非対象 |
| D-098 の安全の境界と既存テストを弱めない規則 | DEV_WORKFLOW Workflow State、`AGENTS.md` の Working Rules・npm 供給網ガード | 変えない。D-127 の「継ぎ足さない」の除外に入れた | AC5 の `それらの改訂は通常の packet・PR の経路で行う` | 非対象 |
| 連番 registry の番号予約 | DEV_WORKFLOW Review Rules | D-124〜D-127 を本 lane が持つ（plan-first で置いた） | AC4 の `rg -c` | 非対象 |
| lane 1 の Status の適用の書式 | lane 1 の packet の S3（`superseded（by D-xxx。…）` と `- Superseded by:` の行） | D-126 の書式をそろえた | AC4 の `superseded by ` の 0 件 | 非対象 |
| ADR の Status | `docs/templates/adr.md` | 変えない（D-126 の対象外） | 無し | 非対象 |
| D-097 Plans に lane の行を足さない | decision-log D-097、DEV_WORKFLOW Wave Operation | 変えない（Plans を触らない） | 無し | 非対象 |

## Contract Probe

- P1 `git merge-tree --write-tree --name-only A B` は衝突のとき exit 1 で衝突の path を出し、衝突が無いとき exit 0 で tree だけを出す: 第 2 便の PR #171 の base 同期の前の head `2f71fab1` と PR #176 の head `daa23a40` で実行 → exit 1、衝突の path は `docs/backlog.md`・`docs/function-design/20-io-product-repo.md`・`docs/function-design/90-traceability.md`・`src-tauri/src/db/product_repo.rs`（実装 file の衝突を含む）。`2f71fab1` と祖先の `44f1e666` で実行 → exit 0、tree の 1 行だけ（git 2.53.0）
- P2 両方の PR が触る file は、merge の衝突が無くても base 同期で PR 差分の byte が変わり、片方だけが触る file は変わらない: `git diff --binary --full-index --no-ext-diff --no-textconv FROM TO -- FILE | sha256sum` を、同期の前（`44f1e666` → `2f71fab1`）と後（`c1c1c5bf` → `87a96fb9`、PR #171 の最終の head）で file ごとに比べた。両方が触り P1 で衝突に出なかった `src-tauri/src/biz/product_service.rs` → `5e322fd6…` と `be9481b7…` で不一致。理由は pre-image の blob が変わること: `--full-index` の index の行が `e8e383d5…..c3c685d7…` から `82e493b6…..7362e89f…` へ変わり（共有の file では必ず変わる）、hunk の見出しも `@@ -63,6 +63,8 @@` から `@@ -64,6 +64,8 @@` へ動いた（round 2 の是正で `grep '^index'` を前後に当てて確かめた）。#171 だけが触る file は前後で一致: `src/features/products/components/ProductForm.tsx` → `ffc8c0c0…`、`docs/SCREEN_DESIGN.md` → `7143e6af…`（起票の版の `117a0c11…` はこの 2 file をまとめた 1 本の差分の sha256 で、前後で一致した）。PR が触る path の一覧は `git diff --name-only --no-renames 44f1e666...2f71fab1` と `44f1e666...daa23a40` で取った。結論: merge-tree の衝突だけで束ねる PR を選ぶと、後の PR の manual を再利用できない場合が残るので、D-124 は path の共有で判定する
- P3 起票の版の command（`comm -12 <(git diff … origin/main...origin/A | sort) <(…)`）は、ref が無いと `git diff` が `fatal` を出しても `comm` が exit 0 で空を返し、「共有なし」と誤る: `origin/no-such-branch-p3` で実行 → `fatal: ambiguous argument`、全体の exit 0・出力なし。直した手順（PR ごとに `git diff --name-only --no-renames origin/main...SHA > FILE` の終了値を確かめる）は、`gh pr view 179 --json headRefOid` の head では exit 0、無い SHA では exit 128 で、比べずに「比べられない」へ進んだ（probe の script は scratchpad、起票の後の round 1 の是正で実行）
- P4 helper の manual の再利用は、L3 の head に `origin/main` を 1 回 merge しただけの head で、PR 差分の全体が byte で一致するときの 1 回だけ: `scripts/pr-gate.py` の現物で、`reuse_geometry` は新しい head の親が `[old_head, base]` であること（`not a single main merge`）と、`git diff --binary --full-index … BASE...HEAD` の byte 一致（`PR diff changed; manual reuse forbidden`）を要求し、`record` の manual の再利用は元の manual が `source_head` を持たないこと（再利用済みでないこと）を要求する（round 3 の是正で `rg -n 'not a single main merge|PR diff changed|reuse_from' scripts/pr-gate.py` で読んだ）。Plan Review round 3 の Codex は、合成の repo で、L3 済みの共有の PR を後ろへ回す順と、束ねた PR の 2 回目の base 同期で、helper がそれぞれ `PR diff changed; manual reuse forbidden` と `not a single main merge` を返すことを確かめた（repo 外の報告）。結論: L3 の後に共有の path を触る PR を先に merge する、または L3 の後に 2 回以上 base 同期すると L3 が 2 回になるので、D-124 は先頭で頼む形を既定にする

## Data Safety

- 店の実値・発言の原文・実 POS / 店舗のデータを書かない。owner の決定は repo 外の回答台帳の ID（TD-324・TD-326〜TD-330、D-127 の出所の TD-310）と日付で引く
- probe は公開 repo の commit（PR #171・#176・#179）だけを使い、作業の出力は `$TMPDIR` に置いた
- 調査の報告（repo 外の `.local/`）の全件の表を写さない

## Implementation Results

- 「規則の文と置き場」の fenced block（12 個）を、本文を変えずに置き場の 7 file へ入れた: `docs/DEV_WORKFLOW.md`（Wave Operation の Ready の箇条の句と D-124 の箇条）、`docs/DOC_STYLE_GUIDE.md`（頭の「対象」の行と §1 の D-125 の節）、`docs/quality/review-checklist.md`（カテゴリ 8 の参照の行と更新履歴）、`docs/backlog.md`（Final Review の枠の 1 項）、`docs/decision-log.md`（頭の `## Status の書き方`。先頭の BOM を保った。D-124〜D-127 は変えていない）、`AGENTS.md`（Working Rules の 3 箇条、Decision and Approval Boundaries の (f)、Safety の両立の句）、`CLAUDE.md`（(g)(h) の 2 段落）
- AC1〜AC7 を bash で逐語に実行し、全て実装の後の期待どおり（AC5 の 15 本は指定の節の中の 1 行ずつ。AC6 の hit は baseline の行と足した行だけ）。AC8 の doc check（full・`--target plan`）と `bash scripts/check-workflow-git.sh` は exit 0。Draft の hosted run と PR は [PR #179](https://github.com/higekiri-w/inventory-system-desktop/pull/179) を参照
- 直した文の旧い文言（Ready の箇条の「Coordinator は既定案として review 通過順を提示する。」、Safety の「小さく、確認可能な変更を優先する。」で終わる行、DOC_STYLE_GUIDE の旧い「対象」の行）を repo 全体に `rg` で当て、archive を除く live な残りは 0（packet の置き場の説明が旧い文を引く 1 行だけ）
- review・CI・merge（closeout、2026-10-20）: Final Review は broad の Double Audit で、Codex（6.1 Sol）= approve（P0）、Claude 側（Fable 5.1）= approve（P3 2。下の Review Response の Closeout）。hosted CI の Merge gate は Ready の後の run で pass、専用 record の review は pass、manual・R4 は無い。PR #179 は helper 経由で 2026-10-20 に merge
- 後続: 棚卸しの lane 2「file:行 の引用の書き換えと検査」（D-125〈TD-326〉、`docs/Plans.md` の次の行動）。安全境界の lane の Final Review の Claude 側の枠（`docs/backlog.md` の項目）

## Review Response

### Plan Review round 1（対象 `131c0809`、2026-10-19）

- fresh Opus 5.5（`subagent_type: reviewer`）: reject（P1 0 / P2 3 / P3 5）。Ordinary Operation は「具体的な反例あり」（manual を持たない PR が先に merge して L3 を持つ PR と path を共有すると L3 が 2 回。D-126 の書式が lane 1 と違う）。#1 P2 D-124 の発火が「束ねる前」だけ、#2 P2 D-126 の全体の置換えの書式が lane 1 と違い Status の行の新設が範囲に無い、#3 P2 spec-check → plan-draft の skip が条件を満たさない（D-124〜D-126 は Plan の前に要る）、#4 P3 AC5 が packet に当たる、#5 P3 AC6 が zsh で exit 2、#6 P3 D-125 が Plans に SHA を書かせ読み違えを招く、#7 P3 Plan Packet・Matrix が D-125 の対象か書いていない、#8 P3 (a) Probe P2 の hash の値 (b) 判定の拡張が agent の解釈と書いていない (c) merge の順を誰がいつ決めるか無い
- Codex（GPT-6 Astra、high。Sol が満員で Astra で走った）: reject（P2 3 / P3 1）。#1 P2 D-124 の command が ref の取得漏れで空出力・exit 0 になり「重なりなし」と誤る、#2 P2 durable decision を packet だけで定義して design を skip した、#3 P2 AC5 が packet を拾い、同じ file の中の重複を拾えない、#4 P3 `117a0c11…` は 2 file をまとめた hash。D-123 は既に accepted、所有表の lane 1 の行の表記（`Superseded by:`）が lane 1 の packet と違う、を確認
- Coordinator の裁定（全件受理）と反映先:
  1. 設計の順（Opus #3・Codex #2）→ Workflow State を design へ戻して D-124〜D-126（追加の D-127 も同じ）を plan-first で decision-log の末尾に置き、遷移を plan-draft → design → plan-draft で記録した。実装の commit は置き場の 3 文だけ（Scope、Design Readiness、「決定の記録」の節）。D-123 は D-126 の例に入れていない（D-126 の Context は D-111・D-113 と D-116〜D-122）
  2. D-124 の発火の範囲（Opus #1・#8c）→ 主語を「L3 を頼む PR が、同じ便でそれより先に merge する PR（manual の有無を問わない）と path を共有するなら」にし、2 つの手段と「Coordinator が既定案を示し、L3 の依頼と同じ問い合わせで owner が決める」、共有の更新を closeout へ寄せる 1 文を足した（規則の文、D-124 の Decision (1)(3)）。同じ便の外の PR の残りのリスクを D-124 の Guarantee range と Matrix の Residual に書いた
  3. command の頑健さ（Codex #1）→ fetch と `gh pr view` の照合、PR ごとの終了値の確認、全て成功したときだけ `comm -12`、失敗は共有ありと同じ（規則の文、D-124 の Decision (2)、Contract Probe P3）
  4. D-126 の書式（Opus #2、Codex の所有表）→ `superseded（by D-xxx。元の出所）` と `- Superseded by: D-xxx（日付）。…` の行、一部は `- Superseded in part by:`、Status の行の無い D に足してよい（規則の文、D-126 の Decision (3)）。所有表の lane 1 の行を lane 1 の packet の表記にした。AC4 に `rg -n '^- Status: superseded by '` の 0 件を足した
  5. AC5（Opus #4・Codex #3）→ packet・archive を除く正本の中の出現の行を `rg -n -o` で数え、指定の位置の 1 か所と照合する形にした
  6. AC6 の shell（Opus #5）→ AC の頭に bash で実行することと `R`・`G` の置き方を書いた
  7. D-125 と Plans（Opus #6）→ 申し送りに commit を添える対象を `docs/backlog.md` に絞り、Plans は行番号の申し送りを置かないと書いた（規則の文、D-125 の Decision (2)・Alternatives・Compatibility）
  8. D-125 の対象（Opus #7）→ 対象外に dated の Plan Packet と Test Design Matrix を足した（規則の文、D-125 の Decision (3)）
  9. Contract Probe の記録（Opus #8a・b、Codex #4）→ P2 に file ごとの値（`ffc8c0c0…`・`7143e6af…`）と `117a0c11…` が 2 file まとめの値であることを書き、D-124 の Decision に判定の拡張が Writer の probe の結果で Coordinator が採ったことを書いた
  10. 追加（owner の決定 TD-328、2026-10-19）→ D-127 を本 lane に足した。既存を出発点として扱い設計を継ぎ足しで歪ませない（owner 2026-10-15 の方針を 1 項にまとめた）、使い手に押しつけない（TD-310）。規則の文の置き場は `AGENTS.md` `## Working Rules`、`## Safety` の「小さく、確認可能な変更を優先する」に両立の句（「D-127 の置き場」の節）。D-127 は plan-first で decision-log の末尾に置き、AC5・AC6 に置き場の出現の数と重複・矛盾の `rg` を足した
  11. 追加 2（owner 2026-10-19、Coordinator の個人の memory の方針を正本へ上げる）→ (d) 規則は環境が変わるたびに変える（owner 2026-09-24）を `AGENTS.md` の「継ぎ足さない」の箇条に、(e) 所感の原文の扱いを `AGENTS.md` Working Rules の箇条に、(f) 店主への問いを Issue に書かないを `AGENTS.md` Decision and Approval Boundaries に、(g) コピペ物の inline と完全な path を `CLAUDE.md` `## 長い作業の報告` に、(h) 安全境界の review の経路を `CLAUDE.md` `## 座組と effort` に置く文を決め、D-127 の Decision (1)(3)〜(6) と出所に足した。委任の範囲は既に Decision and Approval Boundaries にあり足していない。(e) の原文は公開 repo に書かず repo の外の回答台帳に残す形にした。(h) の発注にあった「workflow gate の Double Audit は inline 1 本と Codex 1 本で満たす」は、Claude だけの補助の `CLAUDE.md` が共通の品質の条件（座組表・Double Audit）を変えることになるので（AGENTS.md の「モデル補助は共通の承認・品質条件を変更しない」）、本 packet では入れず、数え方は座組表のままと書いた。入れるなら座組表か DEV_WORKFLOW で別に決める（Coordinator の裁定に返す）

### Plan Review round 2（対象 `a84994e5`、2026-10-19）

- Codex（GPT-6.1 Sol）: approve（P1 0 / P2 0 / P3 1）。Ordinary Operation は「成立」。#1 P3 D-127 の追加の後も live な行に D-127 の前の数（置き場の数・D-124〜D-126・AC5 の本数）が残る。AC の baseline・Contract Probe の値・lane 1 の S3 の書式は現物と一致、と確認
- fresh Opus 5.5（`subagent_type: reviewer`）: reject（P1 0 / P2 4 / P3 5）。Ordinary Operation は「具体的な反例あり」（L3 の後に先の PR の head が変わる場合と、batch Ready の既定の順）。#1 P2 D-124 の確かめが L3 の時点だけ、#2 P2 `CLAUDE.md` (h) の inline の review は fresh context でなく Final Review の枠が埋まらない、#3 P2 「継ぎ足さない」の除外が D-098 の安全の境界と既存テストの規則を含まない、#4 P2 DOC_STYLE_GUIDE が読み手の経路から辿れない、#5 P3 byte が変わる理由、#6 P3 D-127 の前の数、#7 P3 D-126 の allow-list に移送の link の直しが無い、#8 P3 (e) の「要件」の読み分け、#9 P3 AC6 の Issue の検索が大文字小文字を区別する
- Coordinator の裁定（全件受理）と反映先:
  1. D-124 の確かめ直し（Opus #1）→ 箇条と D-124 の Decision (2) に、先の PR の head が変わったときと batch Ready で train 順を示すときに確かめ直すこと、共有があれば L3 を持つ PR を train の先にする案が既定であることを書いた。Wave Operation の Ready の箇条の「既定案は review 通過順」の文に、D-124 の箇条を指す 1 句を足す（規則は D-124 の箇条だけが持つ）。AC1 に `確かめ直す`・`D-124 の箇条の既定案` を足した
  2. `CLAUDE.md` (h) と Final Review の枠（Opus #2）→ (h) に「inline の review は Final Review の本数に数えない。枠の扱いは共通の正本で決めるまで lane の起票で owner に諮る」を足し、backlog に 1 項を足す（Scope・所有表に backlog の行。lane 1 の整理と別の位置）。AC5 の (h) の literal を `Final Review の本数に数えない` に替え、backlog の literal を足した
  3. 「継ぎ足さない」の除外（Opus #3）→ 安全の境界に npm 供給網ガード、DEV_WORKFLOW の D-098 の列挙、既存テストを弱めない規則を入れ、改訂は通常の packet・PR の経路で行うとした（規則の文、D-127 の Decision (1) と Guarantee range）。AC5 に literal を足した
  4. DOC_STYLE_GUIDE への経路（Opus #4）→ `docs/quality/review-checklist.md` の `### 8. 設計-コード整合` に参照の 1 行と更新履歴の 1 行（Scope・AC5）。`docs/templates/` に Writer の発注の雛形は無く、Plan Packet・Matrix の雛形は D-125 の対象外なので足さない
  5. byte が変わる理由（Opus #5）→ 箇条・D-124 の Why・Contract Probe P2 を「pre-image の blob が変わり `--full-index` の index の行が必ず変わる（hunk の位置も動き得る）」にし、P2 に index の行の前後の値を書いた
  6. D-127 の前の数（Codex #1・Opus #6）→ packet・Matrix・D-127 の live な行を D-127 込みの数に同期した（`rg` の hit と直した所は Review Response の外の各節。遷移の記録と Review Response の round 1 の段は履歴として残す）
  7. D-126 の allow-list（Opus #7）→ 規則の文と D-126 の Decision (3) に「file の移送に伴う link の path の直し」を足した
  8. (e) の「要件」（Opus #8）→ 規則の文と D-127 の Decision (3) に「製品の要件。Contract Audit が L3 の項目にする検証は除く」を添えた
  9. AC6 の Issue（Opus #9）→ `rg -n -i` にし、baseline の 14 行の内訳を書いた。MANUAL §5.1 の調査項目の issue は手順書なので (f) と両立と、Matrix の Adjacent Pattern Audit に書いた

### Plan Review round 3（対象 `91e00bbd`、2026-10-19。Plan Review round の天井）

- Codex（GPT-6.1 Sol）: reject（P1 0 / P2 2 / P3 0）。Ordinary Operation は「具体的な反例あり」。#1 P2 既定案の「L3 を頼む PR を先頭へ」が L3 済みの共有の PR を追い越せ、追い越された PR は差分が変わり L3 がまた要る（合成の repo で helper が `PR diff changed; manual reuse forbidden` を返した）、#2 P2 path を共有しない PR を束ねても、L3 の後の base 同期の回数を縛っておらず、2 回目の同期で再利用が拒まれる（helper が `not a single main merge` と再利用済みの拒否を返した）。AC の baseline は現物と一致
- fresh Opus 5.5（`subagent_type: reviewer`）: reject（P1 0 / P2 1 / P3 4）。Ordinary Operation は「具体的な反例あり」。#1 P2 比べる相手が同じ便で先に merge する PR だけで、L3 を頼む時点の `origin/main` に在り head に無い commit（前の便の closeout）と、L3 の後に merge する同じ便の closeout が漏れる、#2 P3 L3 を持つ PR どうしが共有するときの既定案、#3 P3 append-only を当てる本文の範囲、#4 P3 (f) の問いの置き場と MANUAL §5.1 の線引き、#5 P3 path の共有へ広げたのが TD-324 の文より広く owner に確かめる要
- 天井の disposition（owner の決定、repo 外の回答台帳）: TD-329 で D-124 を「先頭で頼む形」に置き換え、TD-330 で直して追加の round なしで plan-approved へ進めると決めた。反映先:
  1. D-124 の置き換え（Codex #1・#2、Opus #1）→ round 1〜3 で足した「束ねる前に確かめる」「確かめ直す」「既定案は L3 を持つ PR を先に」の句を消し、次の形に置き換えた（規則の文、decision-log の D-124 の全項目、Ordinary Operation、失敗定義、Review Focus、Ledger、Matrix の行 2・13・18、Residual）。L3 は train の先頭で `origin/main` を含む head（`git merge-base --is-ancestor`）で頼むのが既定で、その後は他の PR を先に merge しない。束ねるのは、束ねる各 PR が先に merge する全ての PR（同じ便・closeout・便の外）と path を共有せず、先頭になったときに `origin/main` を 1 回だけ取り込む場合だけ。L3 済みの PR を後ろへ回さない。path の共有の確かめ方（round 2 の手順）は、束ねるときの確かめ方として残した。Wave Operation の Ready の箇条の句は「L3 を済ませた PR を後ろへ回す順にしない」に直した。Contract Probe P4 に helper の現物（`reuse_geometry`・`record`）と round 3 の Codex の probe を書いた
  2. Opus #2 → 置き換えで解消した（L3 を持つ PR どうしの共有は、先頭で頼む既定で順に 1 本ずつ）
  3. Opus #3 → D-126 の規則の文と D-126 の Decision (3) に「main に merge された後の本文に当てる（同じ PR の merge の前の直しは対象外）」を足した
  4. Opus #4 → (f) の規則の文と D-127 の Decision (4) に「問いは owner への問い合わせで送り、回答は repo の外の回答台帳に残す。MANUAL §5.1 の調査項目の issue は手順書の形（場所・操作・合格基準）に限る」を添えた
  5. Opus #5 → owner が TD-329 で確かめた。D-124 の Status の出所と owner の決定に TD-329 を足した
  6. Owner Effort Budget → TD-329・TD-330 を decision point として数え、上限を 9 に上げて理由を書いた

### Closeout（2026-10-20）

Final Review（broad、Double Audit）は Codex（6.1 Sol）= approve（P0）、Claude 側（Fable 5.1）= approve（P3 2）で、P1/P2 は無かった。P3 の 2 件は PR の Follow-up で closeout へ寄せ、本 lane の closeout の PR で直した。#1（`docs/backlog.md` の「同じ便で実装 file が衝突する PR の L3 の頼み方を Wave Operation に足す」が TD-324 の旧い形のまま live に残る）は、項目を backlog から外して `docs/archive/2026-10-19-backlog-closed.md` へ移し、「D-124（TD-329）で先頭で頼む・path の共有で判定へ置き換え済み」の根拠の句を付けた。#2（遷移の記録の「本 commit」が 2 行に重なる）は、先の行（plan-draft → plan-gate）を、その遷移を書いた commit の短 SHA `a23c51a9`（plan-approved への遷移と同じ commit）に直した。後の行（plan-approved → implementing）の「本 commit」は state の commit を指すまま残した。
