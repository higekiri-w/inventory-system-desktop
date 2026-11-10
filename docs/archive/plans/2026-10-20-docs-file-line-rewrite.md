# Plan Packet: 設計の正本・code の comment・backlog の file:行 の引用を symbol・節 ID・時点の commit で引き直す（docs の棚卸し lane 2a）

wave に属さない単独の lane（docs 全体の棚卸し〈owner 2026-10-18、計画は repo 外の回答台帳 TD-327〉の lane 2 の前半。owner の決定〈repo 外の回答台帳 TD-353、2026-10-20〉で lane 2 を 2 つに分けた: 2a = 既存の引用の書き換え〈本 lane、R2〉、2b = 違反 0 の状態から検査を足す〈`scripts/doc-consistency-check.sh` を変える R3 の別 lane、2a の merge の後〉）。docs と code の comment だけ。code の振舞い・migration・bindings・画面は変えない。

出所: 書き換えの規則は D-125 と [DOC_STYLE_GUIDE](../../DOC_STYLE_GUIDE.md) §1「code と文書の箇所の引き方（行番号で引かない）」（規則の lane、PR #179）。D-125 の Guarantee range が「既存の引用の書き換えと検査は lane 2 が持つ」とする。起点 `18db621e` の read-only の調査（repo 外の報告。live な引用 505〈正本 221・backlog 121・code の comment 14・時点の記録 149〉、抜き取り 150 のずれ率 45%〈正本 29%・backlog 56%・comment 73%〉。調査の手の分類で command の出力ではない）の後に #179〜#182 で docs が動いたので、AC の baseline は本 packet の起票で今の base（`6a25f63f`）を command で数え直した出力（下の Acceptance Criteria、D-062）。

規模と手法: 書き換える対象は正本 79 行・code の comment 22 行・backlog 31 行（下の数え直し）。1 lane のまま、作業を S1〜S5 に分け、段ごとに AC を置く。本 run（起票）は packet と `docs/Plans.md` の pointer の 1 句だけを置き、引用の書き換えは plan-approved の後の別の run で行う。

## Workflow State

- Phase: archive
- Risk: R2
- Plan Commit: 7b525591464c86520db9f2a7efad27438097e2ff
- Amendments: none
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（起票は本 run。書き換えは plan-approved の後の別の run）
- Plan Reviewer: fresh Opus 5.5（fork でない）+ Codex。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Reviewer: fresh Opus 5.5（Claude 側。R2 で design lane でないので座組表の「それ以外」の行）+ Codex。座組表どおり
- Final Review Minimum: 1
- Human Gate: ready,merge
- Branch: agent/docs-file-line-rewrite

manual なし: 製品の runtime・画面・配布物を変えない docs と comment の変更で、Windows native で確かめる物が無い。

遷移の記録:

1. kickoff → spec-check（2026-10-20、起草役）: 範囲を D-125 の Guarantee range のうち「既存の引用の書き換え」（TD-353 の 2a）にし、Risk を R2 と記録した（下の Risk）。
2. spec-check → plan-draft（2026-10-20、起草役、同じ commit）: 表の唯一の skip。規則は D-125 と DOC_STYLE_GUIDE §1 に在り、本 lane は規則を変えずに既存の引用へ当てるだけなので、design の段で足す設計の出力が無い（下の Design Readiness）。
3. plan-draft（round 1 の是正）: Plan Review round 1 の是正（Review Response 参照）。
4. plan-draft（round 2 の是正）: Plan Review round 2 の是正（Review Response 参照）。
5. plan-draft（round 3 の P3 の一括是正）: Plan Review round 3 の後の disposition（Review Response 参照）。
6. plan-draft（closure の P3 の是正）: round 3 の後の closure の P3 4 件（Review Response 参照）。
7. plan-draft → plan-gate（Plan Review round 1〜3、2026-10-21）: Claude 側 fresh Opus 5.5・Codex（3 round とも Sol）。round 3 は 2 本とも approve（P1/P2 0）。各 round の是正は Review Response 参照。
8. plan-gate → plan-approved（2026-10-21、Coordinator、本 commit。state だけ）: round 3 の後の P3 の一括是正（`0a8e8fcd`）を fresh Opus 5.5 の closure が approve（P1/P2 0）し、その closure の P3 の是正（`7b525591`）を Coordinator が差分と AC-S5-5 の loop の再実行で確かめた。Plan Commit は `7b525591`。
9. plan-approved → implementing（2026-10-21、Coordinator、本 commit。state だけ）: Plan Commit `7b525591` は実装の commit より前にある。書き換えは Writer（Opus 5.5 subagent）への発注で行う。

## Owner Effort Budget

- 介入回数上限: 6（既定）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 6 | 1（起票の時点。TD-353〈lane 2 を 2a と 2b に分ける、2026-10-20〉1） | 2（Ready 1・merge 1） | 3 | 6 = 1 + 2 + 3 |

承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R2

Reason:
docs と code の comment だけの変更で、runtime の契約を変えない。直すのは引用の書き方（行番号 → path と symbol・節 ID、backlog は時点の commit）で、引用が支える文の中身・command・DTO・DB・画面の振舞い・merge gate を変えない。`docs/DEV_WORKFLOW.md` Risk Tiers の境の規則（安定した契約・出力 schema・data safety の境・command の形・route・workflow gate に触れるなら R3）には当たらない。`scripts/ci/classify-changes.sh` の workflow の一覧に当たる file のうち引用を持つのは `docs/ci.md`・`docs/agent-guidance/merge-evidence.md`（書き換える対象）と `docs/quality/review-checklist.md`（変更履歴の行で残す）の 3 つで、本 lane はどれも触らない。前の 2 つは lane 2b へ送る（下の S1 の前文と Review Focus Q1）。workflow の file の文言の変更で gate の green / red は変わらないが、触れば classifier が `workflow=true` を出し、helper（`scripts/pr-gate.py` の Double Audit の require）と `docs/DEV_WORKFLOW.md` Workflow State の `Final Review Minimum` の規則が 2 を求めるので、触らないことで R2・Final Review Minimum 1 を保つ。code の comment を触るので classifier の rust・frontend、function-design を触るので traceability の job は走る（comment と引用の文だけで、結果は変わらない）。

## Goal

Goal Invariant:

### 最小完了条件

- 設計の正本（`docs/` の設計書・runbook。lane 2b へ送る workflow の 2 file を除く）と code の comment（`src/`・`src-tauri/src/`）を読んだ人が、引用を `rg` で今の箇所へ辿れる。行番号の引用が無く（残すのは library の版を同じ行に書いた観測・変更履歴の行・規則の文の例だけ）、書き換えた symbol・節は、その引用が書かれた時点に指していた箇所と同じ物である。
- `docs/backlog.md` の行番号の引用は、全部「書いた時点の main の commit」を同じ行に持ち、`git show` でその commit の行を引くと引用の意図の箇所が出る。
- 書かれた時点の意図を取り戻せない引用は、本 packet の分類表に理由と処置が残り、今の行の中身を見て別の箇所へ付け替えていない。
- lane 2b が違反 0 の状態から検査を足せる: 2b へ送る workflow の 2 件を除き、正本と comment に残す行は分類表の「残す」の行（版を足した library の行の「残す（library の版を足した観測）」を含む）だけである。

### 失敗定義

- ずれた行番号の今の行を見て、別の箇所の symbol・節へ書き換える（意図の取り違え。起点の調査の抜き取りでずれ率 45%）。
- backlog に添えた commit の時点で、その行番号が引用の意図の箇所を指さない。
- 時点の記録（`docs/design-system/reference/**`・`docs/decision-log.md` の本文・dated の Plan Packet と Matrix・`docs/archive/**`）、規則の文（D-125・DOC_STYLE_GUIDE §1）、data の行番号を書き換える。
- 引用以外の文を直す（引用を外したときに文が成り立つための最小の言い換えを越える）、code の振舞いを変える、bindings に出る doc comment を直して再生成を落とす。
- 店の運用の細部・実値・発言の原文を書く。

### 非目的

- 検査を足す（lane 2b）。規則の文（D-125・DOC_STYLE_GUIDE §1）を変える。
- 時点の記録の file（`docs/design-system/reference/**`）の頭に「行番号は某 commit の時点」の注記を足す（起点の調査の提案。DOC_STYLE_GUIDE §1 はそのまま残すと決めている）。
- 外部 library の行が今の版でも合うかを確かめ直す。
- 引用が支える文の中身が今の code と食い違っていても、その文を直す（S4 で backlog の 1 行へ送る）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

not applicable: operator の操作・data / command の契約・業務の状態遷移を変えない docs と comment の変更で、workflow の変更でもない（通常列は発注 → review → owner の Ready / merge）。

## Scope

### 数え直し（起点 `6a25f63f`）

下の AC の block の変数 `P`（拡張子つきの `path:N`・bare の `:N`・`L<N>`・`#L<N>` の 4 形）で数えた。区分と数（command と出力は Acceptance Criteria の baseline）:

| 区分 | 範囲（rg の path と glob） | 引用 | 行 | 本 lane |
|---|---|---|---|---|
| live な docs 全体 | `docs`・`AGENTS.md`・`CLAUDE.md`（`docs/archive/**`・`docs/evidence/**`・`docs/research/**`・`docs/plans/**` を除く） | 466 | — | 下の 4 区分の和 |
| 正本 | 上から `docs/backlog.md`・`docs/decision-log.md`・`docs/design-system/reference/**` を除く | 241 | 86 | S1（うち 7 行は残すか lane 2b へ送る） |
| backlog | `docs/backlog.md` | 85 | 31 | S3 |
| decision-log | `docs/decision-log.md` | 4 | — | 残す（append-only の本文） |
| 時点の記録 | `docs/design-system/reference/**` | 136 | — | 残す（下の表） |
| code の comment | `src`・`src-tauri/src`（`src/lib/invalidation-contract.static.test.ts` を除く） | 22 | 22 | S2 |
| 対象外 | `src/lib/invalidation-contract.static.test.ts` | 11 | — | 残す（test の data の文字列で comment でない） |

起点の調査（`18db621e`）との差（`18db621e` の tree を今の `P` で数え直した数 → 今の数）: backlog が 124 → 85（lane 1 が閉じた項目を archive へ移した）、正本が 242 → 241（`10-common-rules.md` 43 → 42、`73-ui-stocktake.md` 3 → 2、`DOC_STYLE_GUIDE.md` の規則の例 +1）。`.agents`・`.claude/{rules,commands,agents}`・`README.md`・`.github`・`scripts` の hit は 0（`rg -c -e "$P" .agents .claude/rules .claude/commands .claude/agents README.md .github scripts` が何も出さない）。

引用の数え方: `,N`（`CsvImportRecordDetailPage.tsx:41,140`）と、`L<N>` の後の `・N`・` / L<N>`（`SCREEN_DESIGN L42・62・426`、`SCREEN_DESIGN.md L120-133 / L167-176`）と、拡張子の無い略称の `catalog:588` は `rg -o` の数に入らないが、1 つずつ別の引用として分類表の引用の列と照合の入力に書く（分類表に手で足した 14 個）。`/:N`（`61-ui-receiving.md:30/:73`）は `rg -o` が別の token として数え、これも別の引用。範囲 `N-M` と `:N`〜`:M` は 1 つの引用で、照合の入力の N は `N-M`。分類表の引用の列は書かれた形のまま（`-` の範囲は 1 つの token `:N-M`、`〜` の範囲は両端の 2 つの token）で、AC-S5-5 は両側の数字の多重集合で照らす。

### 書き換えの方法（全段で共通）

最大の危険は、ずれた行番号の今の行を見て別の箇所の symbol に書き換えること。各引用について、書かれた時点の意図を次の手順で取り戻す。手順の 1〜4 の結果（採った版と書き換え先）を分類表の行に書く。

1. 候補を引く: 引用を含む行から、その引用を一意に含む文字列 X を決める。X の path は、書かれた path の basename から書く（例 backlog の `src-tauri/src/seed_demo.rs:462` も X は `seed_demo.rs:462`）。path つきの引用は basename から行番号まで。bare の `:N` と `,N` は、同じ行で前の path の basename から `:N` までが記号（backtick・`・`・`〜`・`/`・`,`）だけで続くならその塊（例 `` lib.rs:118`〜`:120 ``・`CsvImportRecordDetailPage.tsx:41,140`）、間に文を挟むなら `:N` の前の 1 文字（backtick か括弧）と `:N`。`L<N>` の X は path の basename か `SCREEN_DESIGN`・`mockup` から `L<N>` まで（例 `SCREEN_DESIGN.md L100`。path が backtick で閉じていれば、その backtick も含める）、`L<N>` の後の `・N` と ` / L<N>` は記号で続く同じ塊（例 `SCREEN_DESIGN L42・62`、`SCREEN_DESIGN.md（在庫照会画面 L120-133 / L167`）。`git log --format='%h %p %ad %s' --abbrev=8 --date=short --pickaxe-regex -S'<X の正規表現の記号を \ で逃がした形>([^0-9]|$)' origin/main` の全 hit と、各 hit の親（`%p`。root `902647b8` には親が無い）を候補にし、古い順に並べる（squash merge なので、引用が hit の PR の中の変更の前の行を指すことがあり、親も候補に入れる）。次の 5 つは使わない（どれも分類表の中に誤った版を採る実例がある）:
   - path で絞る（`-- <doc>`）: backlog への移送・複写の commit を採り、その時点で行がずれている（`MonthlySalesPage.tsx:177` は path を絞ると `bc01698f`〈Plans → backlog の移送〉で、その版の 177 行は `<div className="rounded-md border">`。絞らない候補の `6fc4abac` とその親 `7e0ccf1b` では `<TableHead>部門</TableHead>`）。
   - 最古の 1 つを採る: 別の文書が別の文で同じ file:N を先に書いた commit が最古になる（`StockStatusBadge.tsx:36` の最古の hit `2f8a6f61` は `docs/Plans.md` の別の文で、その版の 36 行は「通常」。引用の「在庫あり」は `f2ef9e52` の版。X に閉じの backtick を付けても最古は `2f8a6f61` のまま。後ろの `([^0-9]|$)` が防ぐのは前方一致〈`lib.rs:118` が `lib.rs:1188-1200` に当たる〉だけ）。
   - 引用の前後の文を含む長い文字列: 同じ行の後の書き換えの commit を採る（`PriceRevisionTable.tsx:104` の長い文字列の最古は `c1c1c5bf` で、その版の 104 行は「最近改定」。引用の「入力中」は `07302b5e`〜`e188366e` の版）。
   - `-L N,N:<doc>`: 行の枠を遡り、引用より前の別の内容に抜ける（`docs/backlog.md:15` の `-L` の最古は `02f497ea` で、その版の 15 行は別の項目。引用を足したのは `c1c1c5bf`）。
   - `git blame`: 最後に行を触った commit を返す（`docs/backlog.md` の `seed_demo.rs:462` の行は blame が `18db621e`、引用を足した hit は `c1c1c5bf`）。

   target は引用の path（basename だけのときは 2 の末尾）。path の無い bare の `:N` と `L<N>` の target は、同じ文が語で名指す file（例 `SCREEN_DESIGN.md L100` → `docs/SCREEN_DESIGN.md`、「catalog `:841`」→ `docs/design-system/02-component-catalog.md`、`mockup L313` → `docs/screen_mockups.html`）、無ければ同じ節の直前の path、節に無ければ文書の対象 module（例 `29-io-ej-parser.md` は `src-tauri/src/io/ej_parser.rs`）。code の comment の bare `:N` は、文が名指す語（test の名前・要素・class）を comment の在る file と test の対象の file で `rg` し、語が在る方を target にする（両方に在るか両方に無ければ 6）。どれで決めたかを処置の欄に書く。
2. 版と symbol を決める: 先に、引用の文が名指す語（識別子・文字列・class 名。例「在庫あり」、`price_range_error`、`prev_month_comparison = Some(prev_items)`）を書き出す。名指す語が無い引用は、その引用を含む文（段落でなく）の主題の語を書き出し、判じた語を処置の欄に書く。候補を古い順に `git show {rev}:{target path} | sed -n '{N-3},{M+3}p'`（M は範囲の終わり。範囲でなければ N）で読み、行 N を含む最も内側の要素・文（JSX の要素・文・呼び出し・定数・test の 1 つ。関数の単位で判じない）が書き出した語を含む最古の候補を採る。範囲 `N-M`・`:N`〜`:M` は 1 つの引用で、N〜M 全体が同じ symbol の中にある版を採る。`,N`・`/:N`・`L<N>・N` は別の引用で、別の symbol に当たりうるので 1 つずつ照らす（`CsvImportRecordDetailPage.tsx:41,140` は `2c125d1a` で 41 = `STATUS_LABELS`、140 = 本体の Badge）。JSX の開始 tag や fn の署名が行 N で、語がその中の後の行にある形は合う（`ManualSalePage.tsx:310` の `<PageHeader` と 312 行の `subtitle`。行 N の語だけで判じるとこの形を落とす）。±3 行に語があるだけでは採らない（`2f8a6f61` の 35 行に引用と同じ class 名があっても、36 行の「通常」は「在庫あり」の要素でない）。名指す語が無い引用は、採った版の行 N の中身も処置の欄に書く。合う候補が複数あり symbol が違えば 6。symbol が同じなら最古を採る（`lib.rs:118` は親 `02f497ea` と hit `dbb83251` のどちらも 118 行が `current_dir` の closure なので `02f497ea`）。target が basename だけのときは `git ls-tree -r --name-only {rev} | rg '/{basename}$'` で候補を出し、文脈で 1 つに決める（決まらなければ 6）。採った版の欄には commit の先頭 8 桁と、hit か親か（例 `44f1e666`〈親〉）を書く。
3. その版の行が属する symbol（関数・型・定数・test の名前と、中の目印の語）か節（節の ID か見出し）を決める（3・4 は正本と code の comment だけ。backlog は 2 で採った版の句を付けるので使わない）。
4. 今の base で `rg -n '{symbol か節の ID・見出し}' {今の path}` がその symbol・節を出すことを確かめる（file が改名されていれば `git log --follow --name-status` で今の path を引く）。
5. 書き換える:
   - 正本と code の comment: repo root からの path と symbol（範囲は symbol と中の目印の語）。文書は節の ID か見出し（`SCREEN_DESIGN.md L<N>` の形も同じ）。同じ文書の中の bare の `:N` も節の ID か見出しにする。`docs/screen_mockups.html` への引用は html に節が無いので、文が成り立てば引用を外し、成り立たなければ 6。bare の `:N`・`L<N>` の隣に symbol が既に書いてある引用（例 `parse_item_count`〈`:1289`〉、`run_startup_step` L84）は、採った版でその symbol が行 N を含むことを確かめて行番号だけを外す。引用の近くの時点の句（例 S2-008 の「`b2389b19` 起源」）は、symbol で引き直した後も残す理由があるかを判じ、処置の欄は先頭を「書き換えた」にして判じた結果を括弧の中に書く（例「書き換えた（`b2389b19` 起源の句は残す: 理由）」）。
   - backlog: 「`{sha8}` の時点で `{path}:{N}`」（sha8 は 2 で採った版の先頭 8 桁。親を採ったら親の SHA。`{path}` は repo root からの path〈1・2 で決めた target〉）。target が `docs/archive/**` のときも同じ句を付ける。同じ行の引用が別の版から来るときは、版ごとにこの句を置く（AC-S3-3 が行ごとに照らす）。
   - 外部 library（`node_modules` の file・vendored の source）: 行番号は残し、同じ行に library の名前と版を書く（例「TanStack Router 1.168.23」）。版は、引用を足した hit（その文書の差分の `+` 行にその引用がある最古の hit）の時点の `package-lock.json` か `Cargo.lock` から、引いている package の版で引く（`@tanstack/router-core` と `@tanstack/react-router` は版が違う。`01-decision-rules.md:389` は 2 つを書き分けている）。DOC_STYLE_GUIDE §1 の「library の版を書いた観測」の形。名前は下の AC の変数 `K` の library の名前の一覧の綴りで、backtick で囲まず `名前 版` の形で書く（例 `@tanstack/router-core 1.168.15`・`tauri-plugin-dialog 2.7.1`。`K` はこの並びだけを拾う）。分類表の処置は「残す（library の版を足した観測）」。
   - repo の外の資料: 同じ行に節の ID（例 `Q12 §1`）があれば行番号を外す。
6. 取り戻せない（候補のどれも 2 で合わない〈root `902647b8` だけが候補で合わない行を含む〉、正本と comment で 3 の symbol・節が今の base に無い、target が決まらない）引用は、書き換えずに分類表の処置を「消えた（理由）」か「曖昧（理由）」とし、S4 で始末する。読んだ候補と行 N の中身は S5 の照合の入力（採った版 `-`）の evidence に出る。

手順 1〜2 の試し（2 件）:

- `docs/backlog.md` の `src-tauri/src/seed_demo.rs:462`（「`sale_auto` の movement」）: 候補は親 `44f1e666` と hit `c1c1c5bf`（PR #176）。古い順に、`git show 44f1e666:src-tauri/src/seed_demo.rs` の 462 行は `sale_auto` の movement の INSERT の文で合う → 採る版は `44f1e666`〈親〉（`c1c1c5bf` の 462 行は冪等性の検査の comment で合わない）。親を候補に入れる理由の実例。
- `docs/function-design/10-common-rules.md` の `product_service.rs:949`〜`:957`（「`price_range_error`」）: 候補は親 `67870e10` と hit `6cac578c`（PR #158）。`67870e10` の版には `price_range_error` が無く、949 行は別の関数の中で合わない。`git show 6cac578c:src-tauri/src/biz/product_service.rs` の 949 行が `fn price_range_error(`、950〜957 行がその本体で合う → 採る版は `6cac578c`〈hit〉、書き換え先は `src-tauri/src/biz/product_service.rs` の `price_range_error`。今の base では `rg -n 'fn price_range_error' src-tauri/src/biz/product_service.rs` が 965 行に出す（行番号はずれた）。

### S1 正本（86 行、248 引用〈`rg -o` の 241 と手で足した 7〉）

分類表の S1 の行（下の「分類表」）を、上の方法で書き換える。対象の file: `docs/design-system/00-foundations.md`・`01-decision-rules.md`・`02-component-catalog.md`、`docs/function-design/` の 10・22・24・29・31・32・37・45・54・55・56・57・58・66・70・71・73、`docs/msi-distribution-runbook.md`。

起票の時点で処置が決まっている 7 行（表に書き込み済み）:

- 残す 5 行: `docs/DOC_STYLE_GUIDE.md:87`（規則の文の例。時点の commit つき）、`docs/quality/review-checklist.md:123`・`docs/design-system/02-component-catalog.md:1063`・`docs/function-design/53-ui-home.md:312`（変更履歴の表の行。時点の記録）、`docs/design-system/01-decision-rules.md:389`（「TanStack Router 1.168.23」の版つきの library の観測）。
- lane 2b へ送る 2 行: `docs/ci.md:29`（`scripts/pr-gate.py:398-410`）と `docs/agent-guidance/merge-evidence.md:160`（`docs/templates/subagent-review-packet.md:15`）。workflow の file で、触ると Final Review Minimum が 2 になる（上の Risk）。lane 2b は `scripts/doc-consistency-check.sh` を変えて元から `workflow=true` の Double Audit なので、同じ lane で直してから検査を入れる。

型が決まっている行: `01-decision-rules.md` の 362・419・420・421・423 と `22-mnt-migration.md` の 234 は外部 library（方法の 5 の library の形。01:362 は `@radix-ui/react-alert-dialog` の `index.d.mts`、22:234 は `tauri-plugin-dialog` の vendored の source）、`01-decision-rules.md:138` は repo の中の `RootLayout.tsx:62` と repo の外の資料（`ui-design-rules-qa-v2.md` の `:419`・`:424-433`。同じ行に `Q12 §1`・`Q12 §2/§3` がある）。

### S2 code の comment（22 行、25 引用〈`rg -o` の 22 と手で足した 3〉）

分類表の S2 の行を、上の方法で書き換える（正本と同じ形。外部 library の 3 行〈S2-009〜S2-011、TanStack Router の `scroll-restoration.js`・happy-dom の `Element.js`・`HTMLElement.js`〉は library の形）。`DSR-22` の bare の `` `:443` ``（S2-002・S2-003）は `docs/design-system/01-decision-rules.md` の DSR-22 で引く。

bindings: 直す comment が `#[derive(specta::Type)]` の型か `#[specta::specta]` の command の doc comment なら、同じ段で `cd src-tauri && cargo run --bin generate_bindings` を回し、`src/lib/bindings.ts` の差分を同じ commit に入れる（2026-10-18 に再生成の書き落としで Draft CI が赤になった先例）。起票の時点で S2 の Rust の行は S2-001 と S2-015 で、どちらも bindings に出ない: S2-001 は `src-tauri/src/biz/csv_import_service/time_evidence.rs` の `pub(crate) fn receive_source` の doc comment（`csv_import_service/mod.rs` で `#[cfg(test)]` の module）、S2-015 は `src-tauri/src/lib.rs` の `pub fn export_specta_bindings` の doc comment（`#[specta::specta]` の command でも specta の型でもない）。AC-S2-3 で確かめる。

### S3 backlog（31 行、89 引用〈`rg -o` の 85 と手で足した 4〉）

分類表の S3 の行を、上の方法で書き換える（backlog の形）。採った版の SHA は main に在る commit（AC-S3-2）。同じ行に別の版から来た引用があれば、版ごとに句を置く。引用の文の中身（項目の申し送りの内容）は変えない。

### S4 消えた・曖昧の始末

S1〜S3 で処置を「消えた（理由）」「曖昧（理由）」にした行を、1 行ずつ次のどれかにし、処置の欄に書く:

- 引用を外す: 引用が無くても文が成り立つ（引用は根拠の添え物）。文は変えない。
- 最小の言い換え: 引用を外すと文が成り立たない。引用の語だけを、今の base で在る symbol・節に替える。替える symbol は引用の文自身が名指す語から決め（例 `prev_month_comparison = Some(prev_items)` → `rg` で `get_monthly_sales`）、名指す語が無いか、名指す語が今の base の `rg` で 1 つの symbol に決まらなければ、言い換えず「引用を外す」か backlog の 1 句へ送る。今の行 N の中身（例 `sales_service.rs:196` の今の `pub count: usize`）は根拠にしない。替えた語が書かれた時点の意図と合う根拠（名指す語と、それが今どの symbol にあるか）を処置の欄に 1 句で書く。
- 引用が支える文の中身が今の code・正本と食い違う（指す物が消えた・変わった）ときは、文を直さず、`docs/backlog.md` の「次の docs の lane の候補（docs の棚卸し lane 1 の残り）」の項目に 1 句を足す（file と節と食い違いの要旨）。足した句を処置の欄に書く。

### S5 分類表の照合

分類表の全行の処置の欄の先頭が「書き換えた」「残す（理由）」「lane 2b へ送る」「消えた（理由）→ 処置」「曖昧（理由）→ 処置」のどれかで、未処置が無いこと（AC-S5-5 は処置を先頭の語で読む）。

全引用の照合の evidence: 分類表の S1〜S3 の行のうち処置が「書き換えた」「消えた」「曖昧」の行の、repo の中の file を target にする引用ごとに、下の「照合の入力」の block に 1 行（`ID 採った版 target N W X`。採った版は 2 で採った commit の先頭 8 桁で、取り戻せない引用は `-`。N は行か範囲 `N-M`。W は 2 で書き出した名指す語から採った空白の無い 1 語〈例 `price_range_error`・`在庫あり`〉で、名指す語が複数あるときは行 N〜M に literal に在る語を選び〈無ければ `-`〉、名指す語が無い引用も `-`。X は 1 の文字列で、行の残り全部）を書く。`,N`・`/:N`・`L<N>・N` は 1 つずつ別の行、範囲 `N-M`・`:N`〜`:M` は 1 行。外部 library・repo 外の資料の引用は書かない。AC-S5-5 の loop の出力（引用ごとの候補の一覧と各候補の行 N−3〜M+3）を PR に残す。人手の判断（行 N を含む最も内側の要素・文が名指す語を含むか）は、出力の `CHECK` の付いた塊と採った版の抜き取り（下の Implementation Results の約束）の表で確かめる。`CHECK` は、`*` の版の N〜M に W が無いとき、`*` より古い候補〈採った版が `-` なら全候補〉の N〜M に W が在るとき、W が `-` のときに出る。Final Review の reviewer が読むのは `CHECK` の塊と AC-S5-2 の抜き取り。

### 分類表

型の語: 正本・comment・backlog（上の段）、外部 library、repo 外の資料、変更履歴、規則の例、workflow。処置の語: 「書き換えた」「残す（理由）」「lane 2b へ送る」「消えた（理由）→ 処置」「曖昧（理由）→ 処置」。処置の欄の先頭はこのどれかの語で、書き換えた行の補足（例: 時点の句を残す理由）は「書き換えた（…）」の括弧の中に書く。採った版は方法の 2 で採った commit の先頭 8 桁と、hit か親か（例 `44f1e666`〈親〉）を引用の順に `・` で区切って書き（取り戻せない引用は `-`）、照合の入力の行と同じ版にする（AC-S5-5 が照らす）。書き換え先は symbol・節の名前、backlog は付けた句。引用の列は `rg -o` の順に、書かれた形のまま並べる（`-` の範囲は `:N-M` の 1 つ、`〜` の範囲は両端の 2 つ）。`rg -o` が拾わない `,N`・`L<N>` の後の `・N`・` / L<N>`・`catalog:588` は手で足した（S1-010・S1-019・S1-020・S1-026・S1-027・S1-084・S2-011・S2-018・S3-009・S3-027・S3-029 の 14 個）。S1-079〜S1-086・S2-015〜S2-022 は `P` を 4 形に広げて足した行（起票の後に足したので段の末尾に置く）。場所の行は起点の行番号。AC-S5-5 はこの行を `origin/main` で読んで X が在ることを確かめ、AC-S3-3 は S3 の行を作業中の `docs/backlog.md` で読む（引用の書き換えでは行は増減しない）。同期で行がずれたら、場所の path と引用の文字列で今の行を引き直して場所の列を直す。

#### S1 の行

| ID | 場所（`6a25f63f` の行） | 引用（`rg -o` の順） | 型 | 採った版 | 書き換え先 | 処置 |
|---|---|---|---|---|---|---|
| S1-001 | `docs/DOC_STYLE_GUIDE.md:87` | `src-tauri/src/biz/product_service.rs:965` | 規則の例 | — | — | 残す（規則の文の例。時点の commit つき） |
| S1-002 | `docs/agent-guidance/merge-evidence.md:160` | `docs/templates/subagent-review-packet.md:15` | workflow | — | — | lane 2b へ送る（workflow の file） |
| S1-003 | `docs/ci.md:29` | `scripts/pr-gate.py:398-410` | workflow | — | — | lane 2b へ送る（workflow の file） |
| S1-004 | `docs/design-system/00-foundations.md:188` | `StocktakePage.tsx:933` | 正本 | `654f95d5`〈親〉 | 仕入原価総額の card の値（`StocktakePage`） | 書き換えた |
| S1-005 | `docs/design-system/01-decision-rules.md:120` | `ListShell.tsx:69` | 正本 | `ff655efd`〈親〉 | `ListShell` の引数 `topSummary` の既定 | 書き換えた |
| S1-006 | `docs/design-system/01-decision-rules.md:126` | `InventoryRecordsPage.tsx:356`・`:353`・`:349-359` | 正本 | `c9fabc17`〈hit〉・`c9fabc17`〈hit〉・`c9fabc17`〈hit〉 | `InventoryRecordsPage` の表頭（`TableHeader`） | 書き換えた（`:349-359` は採用後の 7 列の並びの版 `c9fabc17` を採った。古い `43b69bea` の表頭は記録ID 列を含む） |
| S1-007 | `docs/design-system/01-decision-rules.md:127` | `StockMovementsPage.tsx:73-101`・`MovementTable.tsx:52-53` | 正本 | `654f95d5`〈親〉・`654f95d5`〈親〉 | `StockMovementsPage` の `PageHeader` と商品の見出し、`MovementTable` の表頭 | 書き換えた |
| S1-008 | `docs/design-system/01-decision-rules.md:128` | `OperationLogsPage.tsx:461-462` | 正本 | `654f95d5`〈親〉 | `OperationLogsPage` の表頭 | 書き換えた |
| S1-009 | `docs/design-system/01-decision-rules.md:129` | `SupplierUsageTable.tsx:19` | 正本 | `654f95d5`〈親〉 | `SupplierUsageTable` の表頭 | 書き換えた |
| S1-010 | `docs/design-system/01-decision-rules.md:130` | `BackupRestorePage.tsx:520`・`:522` | 正本 | `654f95d5`〈親〉・`654f95d5`〈親〉 | `BackupRestorePage` の表頭 | 書き換えた |
| S1-011 | `docs/design-system/01-decision-rules.md:132` | `2026-08-23-current-design-analysis.md:18` | 正本 | `9f90964d`〈hit〉 | `2026-08-23-current-design-analysis.md` の 1-8 | 書き換えた |
| S1-012 | `docs/design-system/01-decision-rules.md:134` | `InventoryRecordsPage.tsx:349-359` | 正本 | `43b69bea`〈親〉 | `InventoryRecordsPage` の表頭 | 書き換えた（文は記録ID 列のあった時点の表頭を指す） |
| S1-013 | `docs/design-system/01-decision-rules.md:136` | `StockStatusBadge.tsx:41-44` | 正本 | `7ae4ade6`〈親〉 | `StockStatusBadge` の中立の Badge | 書き換えた（文の「`通常`」は今の「在庫あり」と食い違う → backlog の「次の docs の lane の候補」の (8)） |
| S1-014 | `docs/design-system/01-decision-rules.md:138` | `RootLayout.tsx:62`・`:419`・`:424-433` | 正本 + repo 外の資料 | `654f95d5`〈親〉 | `RootLayout` の `<aside>` | 書き換えた（repo 外の資料の `:419`・`:424-433` は同じ行に `Q12 §1`・`Q12 §2/§3` があるので行番号を外した） |
| S1-015 | `docs/design-system/01-decision-rules.md:389` | `router-core/dist/esm/router.d.ts:309-334`・`router.js:415`・`:427`・`:118`・`:119`・`:146-160`・`:147-161` | 外部 library | — | — | 残す（library の版つきの観測） |
| S1-016 | `docs/design-system/01-decision-rules.md:414` | `RootLayout.tsx:65` | 正本 | `f1d5590c`〈hit〉 | `RootLayout` の `<main>` | 書き換えた |
| S1-017 | `docs/design-system/01-decision-rules.md:419` | `router-core/dist/esm/router.js:415`・`scroll-restoration.js:118-171` | 外部 library | — | @tanstack/router-core 1.168.15（`f1d5590c` の lock） | 残す（library の版を足した観測） |
| S1-018 | `docs/design-system/01-decision-rules.md:420` | `scroll-restoration.js:122` | 外部 library | — | @tanstack/router-core 1.168.15（`f1d5590c` の lock） | 残す（library の版を足した観測） |
| S1-019 | `docs/design-system/01-decision-rules.md:421` | `scroll-restoration.js:73`・`:106`・`:114` | 外部 library | — | @tanstack/router-core 1.168.15（`f1d5590c` の lock） | 残す（library の版を足した観測） |
| S1-020 | `docs/design-system/01-decision-rules.md:423` | `router.js:118`・`:146-160`・`react-router/dist/esm/Match.js:112-125` | 外部 library | — | @tanstack/router-core 1.168.15・@tanstack/react-router 1.168.23（`f1d5590c` の lock） | 残す（library の版を足した観測） |
| S1-021 | `docs/design-system/02-component-catalog.md:53` | `SupplierManagementPage.tsx:35-39` | 正本 | `c9fabc17`〈hit〉 | `SupplierManagementPage` の `PageHeader` | 書き換えた |
| S1-022 | `docs/design-system/02-component-catalog.md:55` | `ReceivingPage.tsx:295`・`ManualSalePage.tsx:310`・`ReturnExchangePage.tsx:419`・`DisposalPage.tsx:285`・`SupplierManagementPage.tsx:35-39`・`PageHeader.tsx:47-53`・`PageHeader.tsx:31-45` | 正本 | `43b69bea`〈親〉・`43b69bea`〈親〉・`43b69bea`〈親〉・`43b69bea`〈親〉・`c9fabc17`〈hit〉・`c9fabc17`〈hit〉・`bb1862a5`〈hit〉 | 4 画面と `SupplierManagementPage` の `PageHeader`、`PageHeader` の `actions` なし・ありの分岐 | 書き換えた（`PageHeader.tsx:31-45` は文の「2 段配置」「PR #63」に合う `bb1862a5` を採った。古い候補の同じ `actions` の分岐は 1 段の旧構造） |
| S1-023 | `docs/design-system/02-component-catalog.md:200` | `MovementTable.tsx:93-99`・`OperationLogsPage.tsx:533`・`MovementTable.tsx:93-99`・`ManualSaleRecordDetailPage.tsx:149`・`MovementTable.tsx:82-91`・`StocktakeRecordDetailPage.tsx:234` | 正本 | `1c896139`〈親〉・`43b69bea`〈親〉・`c9fabc17`〈hit〉・`1c896139`〈親〉・`1c896139`〈親〉・`1c896139`〈親〉 | `MovementTable` の備考の `TableCell`・`movement.source` の分岐、`OperationLogsPage` の概要の `TableCell`、`ManualSaleRecordDetailPage` の `detail.note`、`StocktakeRecordDetailPage` | 書き換えた（`MovementTable.tsx:93-99` の 1 つめは三項演算子の版 `1c896139`、2 つめは「本 PR（⑮）で是正済み」の版 `c9fabc17`） |
| S1-024 | `docs/design-system/02-component-catalog.md:415` | `PriceRevisionPage.tsx:79-83`・`DailySalesPage.tsx:175`・`MonthlySalesPage.tsx:166` | 正本 | `2c125d1a`〈親〉・`7ae4ade6`〈親〉・`7ae4ade6`〈親〉 | `PriceRevisionPage` の注意の `Alert`、`DailySalesPage`・`MonthlySalesPage` の未取込みの表示 | 書き換えた |
| S1-025 | `docs/design-system/02-component-catalog.md:888` | `StockStatusBadge.tsx:29`・`StocktakePage.tsx:396-401`・`csv-import/components/PreviewStep.tsx:75-80`・`ProductTable.tsx:88`・`ResultStep.tsx:47`・`CsvImportRecordDetailPage.tsx:140` | 正本 | `eabc549c`〈親〉・`7ae4ade6`〈親〉・`7ae4ade6`〈親〉・`2c125d1a`〈親〉・`7ae4ade6`〈親〉・`07302b5e`〈親〉 | 各 file の Badge の label（path と label） | 書き換えた |
| S1-026 | `docs/design-system/02-component-catalog.md:889` | `IntegrityCheckPage.tsx:389`・`StocktakePage.tsx:404`・`ProductTable.tsx:93`・`ResultStep.tsx:47`・`DailyReportImportPage.tsx:164`・`:186`・`CsvImportRecordDetailPage.tsx:140`・`DailyReportImportPage.tsx:322` | 正本 | `2c125d1a`〈親〉・`7ae4ade6`〈親〉・`2c125d1a`〈親〉・`7ae4ade6`〈親〉・`7ae4ade6`〈親〉・`7ae4ade6`〈親〉・`07302b5e`〈親〉・`7ae4ade6`〈親〉 | 各 file の Badge の label（path と label） | 書き換えた |
| S1-027 | `docs/design-system/02-component-catalog.md:890` | `StockStatusBadge.tsx:20`・`CsvImportRecordDetailPage.tsx:41`・`:140`・`:192` | 正本 | `eabc549c`〈親〉・`7ae4ade6`〈親〉・`7ae4ade6`〈親〉・`1ad9aa74`〈親〉 | `StockStatusBadge`「在庫切れ」、`CsvImportRecordDetailPage` の `STATUS_LABELS`・状態の Badge・明細の Badge | 書き換えた |
| S1-028 | `docs/design-system/02-component-catalog.md:891` | `StockStatusBadge.tsx:36`・`inventory-records/types.ts:87-94` | 正本 | `f2ef9e52`〈hit〉・`7ae4ade6`〈親〉 | `StockStatusBadge`「在庫あり」、`formatRecordStatus` | 書き換えた |
| S1-029 | `docs/design-system/02-component-catalog.md:893` | `IntegrityCheckPage.tsx:469`・`PriceRevisionTable.tsx:104`・`ProductTable.tsx:74`・`CsvImportRecordDetailPage.tsx:194`・`ReturnExchangePage.tsx:97`・`ReturnExchangePage.tsx:592`・`ReturnExchangePage.tsx:602` | 正本 | `c1c1c5bf`〈hit〉・`07302b5e`〈親〉・`07302b5e`〈親〉・`7ae4ade6`〈親〉・`2c125d1a`〈親〉・`2c125d1a`〈親〉・`2c125d1a`〈親〉 | 各 file の要素（path と label）、`formatRegisterProcessed` | 書き換えた |
| S1-030 | `docs/design-system/02-component-catalog.md:895` | `ProductTable.tsx:56`・`ProductTable.tsx:74`・`PriceRevisionTable.tsx:98`・`daily-sales/components/ProductTable.tsx:133` | 正本 | `7ae4ade6`〈親〉・`07302b5e`〈親〉・`7ae4ade6`〈親〉・`7ae4ade6`〈親〉 | 各 file の Badge の label（path と label） | 書き換えた |
| S1-031 | `docs/design-system/02-component-catalog.md:899` | `PriceRevisionTable.tsx:104` | 正本 | `07302b5e`〈親〉 | `PriceRevisionTable`「入力中」 | 書き換えた |
| S1-032 | `docs/design-system/02-component-catalog.md:1063` | `PriceRevisionPage.tsx:112-116`・`:158` | 変更履歴 | — | — | 残す（変更履歴の行） |
| S1-033 | `docs/function-design/10-common-rules.md:121` | `stocktake_service.rs:529`・`src/features/stock-inquiry/lib/filter-low-stock-list.ts:62`・`:63`・`src/features/products/search.ts:30` | 正本 | `6cac578c`〈hit〉・`67870e10`〈親〉・`67870e10`〈親〉 | `legacy_complete_stocktake`・`filterAndSortLowStockList`・`PRODUCT_SORT_OPTIONS` | 書き換えた |
| S1-034 | `docs/function-design/10-common-rules.md:165` | `receiving.rs:126`・`manual_sale.rs:141`・`disposal.rs:123`・`returns.rs:99`・`returns.rs:254`・`:255` | 正本 | `6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉 | `create_receiving`・`create_manual_sale`・`create_disposal`・`validate_return_request_shape`・`create_return` | 書き換えた |
| S1-035 | `docs/function-design/10-common-rules.md:166` | `csv_import_service/parse.rs:98`・`:110`・`csv_import_service/commit.rs:138`・`:167` | 正本 | `6cac578c`〈hit〉・`c85e135d`〈親〉 | `parse_and_validate`・`execute_commit` の `for row in matched_rows` | 書き換えた |
| S1-036 | `docs/function-design/10-common-rules.md:167` | `inventory_service/common.rs:82`・`:87`・`product_service.rs:1062`・`:1069`・`:1316`・`:1324`・`csv_import_service/commit.rs:276`・`integrity_service.rs:192` | 正本 | `6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉 | `inventory_service/common.rs`（隣の `apply_stock_change`）・`validate_create_request`・`preview_import`・`apply_void_stock_corrections`・`fix_integrity` | 書き換えた（目印だった `check_stock_range` は今は `src-tauri/src/biz/stock_ledger.rs` の `recompute_stock` の中なので、symbol だけで引いた） |
| S1-037 | `docs/function-design/10-common-rules.md:168` | `product_service.rs:949`・`:957`・`:1059`・`:1103`・`:473`・`:474`・`:1285` | 正本 | `6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉 | `price_range_error` と呼ぶ所 `validate_create_request`・`validate_update_request`・`revise_product_price`・`preview_import` | 書き換えた |
| S1-038 | `docs/function-design/10-common-rules.md:169` | `product_service.rs:1059`・`:1106`・`:473`・`:474`・`:1291`・`receiving.rs:138`・`disposal.rs:135` | 正本 | `6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉 | `validate_create_request`・`validate_update_request`・`revise_product_price`・`preview_import`・`create_receiving`・`create_disposal` | 書き換えた |
| S1-039 | `docs/function-design/10-common-rules.md:173` | `src/features/products/lib/price-revision-math.ts:1`・`:9` | 正本 | `c1beb334`〈親〉 | `deriveProposedCost` | 書き換えた |
| S1-040 | `docs/function-design/10-common-rules.md:209` | `product_service.rs:907`・`:927`・`product_repo.rs:1310`・`:1311` | 正本 | `67870e10`〈親〉・`6cac578c`〈hit〉 | `list_low_stock`・`list_low_stock_products` | 書き換えた |
| S1-041 | `docs/function-design/22-mnt-migration.md:234` | `lib.rs:355-356` | 外部 library | — | tauri-plugin-dialog 2.7.1（`2e3e97d3` の lock） | 残す（library の版を足した観測） |
| S1-042 | `docs/function-design/22-mnt-migration.md:353` | `product_service.rs:274`・`:285` | 正本 | `6cac578c`〈hit〉 | `create_product` の `if req.initial_stock > 0` | 書き換えた（`67870e10` の 274〜285 行は次の `#[cfg(test)]` へ跨ぐので、範囲全体が 1 つの文に入る `6cac578c` を採った） |
| S1-043 | `docs/function-design/22-mnt-migration.md:360` | `schema_time_evidence.rs:120`・`:124` | 正本 | `67870e10`〈親〉 | `schema_time_evidence.rs` の `SCHEMA_SQL` の `stocktake_recounts` | 書き換えた |
| S1-044 | `docs/function-design/22-mnt-migration.md:378` | `src-tauri/src/lib.rs:1214`・`:1223`・`lib.rs:161`・`src-tauri/src/db/mod.rs:298`・`src-tauri/src/mnt/restore.rs:11`・`:13`・`src-tauri/src/mnt/backup.rs:206`・`:210`・`lib.rs:1227`・`src-tauri/src/io/image_manager.rs:15`・`:58` | 正本 | `02f497ea`〈親〉・`02f497ea`〈親〉・`02f497ea`〈親〉・`02f497ea`〈親〉・`02f497ea`〈親〉・`02f497ea`〈親〉・`02f497ea`〈親〉・`02f497ea`〈親〉 | `run`・`prepare_database_with_init`・`migrate_legacy_db_with_ops`・`MANIFEST_SUFFIX` ほか・`resolve_backup_dir`・`DiagnosticLogConfig`・`RECEIPTS_DIR`・`save_receipt_image_with_date` | 書き換えた |
| S1-045 | `docs/function-design/22-mnt-migration.md:379` | `lib.rs:118`・`:120`・`lib.rs:165`・`:183`・`db/mod.rs:296` | 正本 | `02f497ea`〈親〉・`02f497ea`〈親〉・`02f497ea`〈親〉 | `prepare_database`・`prepare_database_with_init`・`migrate_legacy_db_with_ops` | 書き換えた |
| S1-046 | `docs/function-design/22-mnt-migration.md:386` | `lib.rs:1219`・`:1223` | 正本 | `02f497ea`〈親〉 | `run` の `StartupFailureKind::CreateAppDataDir` | 書き換えた |
| S1-047 | `docs/function-design/22-mnt-migration.md:389` | `restore.rs:690`・`:700`・`lib.rs:188`・`:195` | 正本 | `02f497ea`〈親〉・`02f497ea`〈親〉 | `reconcile_restore_with_ops`・`prepare_database_with_init` | 書き換えた |
| S1-048 | `docs/function-design/24-io-csv-import-repo.md:556` | `src-tauri/src/db/sales_repo.rs:760` | 正本 | `3933f0d4`〈親〉 | `find_daily_report_imports_by_report_date` | 書き換えた |
| S1-049 | `docs/function-design/24-io-csv-import-repo.md:705` | `src-tauri/src/biz/mod.rs:26`・`:28`・`:30`・`src-tauri/src/db/sales_repo.rs:13`・`:19`・`:862`・`:873` | 正本 | `3933f0d4`〈親〉・`6cac578c`〈hit〉・`3933f0d4`〈親〉・`3933f0d4`〈親〉 | `biz/mod.rs` の re-export、`CsvImportStatus`・`row_to_csv_import` | 書き換えた（`:28`〜`:30` の target は同じ文の `src-tauri/src/biz/mod.rs`） |
| S1-050 | `docs/function-design/29-io-ej-parser.md:323` | `:41`・`:190`・`:57`・`:1289`・`:174`・`:776`・`:814`・`:707`・`:717` | 正本 | `9d351d5b`〈親〉・`c1beb334`〈親〉・`c1beb334`〈親〉・`c1beb334`〈親〉・`c1beb334`〈親〉・`c1beb334`〈親〉・`c1beb334`〈親〉・`c1beb334`〈親〉 | `EjLineKind::Quantity`・`EjItem`・`EjLineKind::ItemCount`・`parse_item_count`・`EjRestoration::Restored`・`item_region`・`restore_items` | 書き換えた（target は文書の対象 module `src-tauri/src/io/ej_parser.rs`。隣に symbol がある引用は行番号だけを外した） |
| S1-051 | `docs/function-design/31-biz-inventory-service.md:10` | `src-tauri/src/biz/csv_import_service/commit.rs:157` | 正本 | `249df0ca`〈親〉 | `execute_commit` の `apply_stock_change` | 書き換えた |
| S1-052 | `docs/function-design/31-biz-inventory-service.md:28` | `receiving.rs:126`・`manual_sale.rs:141`・`disposal.rs:123`・`returns.rs:99`・`returns.rs:254`・`:255`・`receiving.rs:156`・`disposal.rs:159` | 正本 | `6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉 | `create_receiving`・`create_manual_sale`・`create_disposal`・`validate_return_request_shape`・`create_return`・`check_safe_integer` の呼出し | 書き換えた |
| S1-053 | `docs/function-design/31-biz-inventory-service.md:29` | `receiving.rs:138`・`disposal.rs:135` | 正本 | `6cac578c`〈hit〉・`6cac578c`〈hit〉 | `create_receiving`・`create_disposal` の `MAX_COST_PRICE` | 書き換えた |
| S1-054 | `docs/function-design/31-biz-inventory-service.md:32` | `receiving.rs:234`・`returns.rs:273`・`manual_sale.rs:289`・`disposal.rs:226` | 正本 | `6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉 | `create_*` の `negative_stock_warning_message` | 書き換えた |
| S1-055 | `docs/function-design/32-biz-csv-import-service.md:21` | `src-tauri/src/biz/csv_import_service/parse.rs:63-77` | 正本 | `ea15360d`〈親〉 | `parse_and_validate` の `2a. settlement_date 業務妥当性チェック` | 書き換えた |
| S1-056 | `docs/function-design/37-biz-daily-report-import-service.md:256` | `src-tauri/src/biz/mod.rs:54`・`useDailyReportImportFlow.ts:35` | 正本 | `6b765f75`〈親〉・`6b765f75`〈親〉 | `biz/mod.rs`（隣の `BizError`）・`decideRecoverTo` | 書き換えた |
| S1-057 | `docs/function-design/37-biz-daily-report-import-service.md:261` | `src/lib/bindings.ts:558`・`commit.rs:83-85` | 正本 | `6b765f75`〈親〉・`6b765f75`〈親〉 | `bindings.ts`（隣の `DailyReportSourceFileInfo`）・`execute_commit` の `source_files_json` | 書き換えた |
| S1-058 | `docs/function-design/37-biz-daily-report-import-service.md:552` | `src-tauri/src/biz/daily_report_import_service/parse.rs:159`・`tests.rs:275`・`:481`・`:536` | 正本 | `3933f0d4`〈親〉・`3933f0d4`〈親〉・`3933f0d4`〈親〉・`3933f0d4`〈親〉 | `parse_and_validate_daily_report`・test 3 つ | 書き換えた |
| S1-059 | `docs/function-design/37-biz-daily-report-import-service.md:633` | `src-tauri/src/db/sales_repo.rs:13`・`:19`・`:862`・`:873` | 正本 | `3933f0d4`〈親〉・`3933f0d4`〈親〉 | `CsvImportStatus`・`row_to_csv_import` の `match` | 書き換えた |
| S1-060 | `docs/function-design/37-biz-daily-report-import-service.md:669` | `src-tauri/src/io/ej_parser.rs:455`・`:460` | 正本 | `3933f0d4`〈親〉 | `parse_header` の `date_ok` | 書き換えた |
| S1-061 | `docs/function-design/37-biz-daily-report-import-service.md:673` | `src-tauri/src/io/ej_parser.rs:678`・`:689`・`:575`・`:581` | 正本 | `3933f0d4`〈親〉・`3933f0d4`〈親〉 | `check_record` の `NoItems`・`build_record` の `MissingFinalNewline` | 書き換えた（`:575`〜`:581` の target は「同」の `src-tauri/src/io/ej_parser.rs`） |
| S1-062 | `docs/function-design/37-biz-daily-report-import-service.md:674` | `:832`・`:834` | 正本 | `3933f0d4`〈親〉 | `item_region` の `adjustments.pop()` | 書き換えた（target は「同」の `src-tauri/src/io/ej_parser.rs`） |
| S1-063 | `docs/function-design/45-cmd-daily-report-import.md:56` | `src-tauri/src/cmd/daily_report_import_cmd.rs:43`・`:242` | 正本 | `6b765f75`〈親〉・`e7c22f8f`〈親〉 | `parse_and_validate_daily_report` の `DailyReportInputFile`・`tests` の module | 書き換えた |
| S1-064 | `docs/function-design/45-cmd-daily-report-import.md:242` | `src/features/daily-report-import/hooks/useDailyReportImportFlow.ts:35`・`src-tauri/src/cmd/mod.rs:130` | 正本 | `6b765f75`〈親〉・`0989ced0`〈親〉 | `decideRecoverTo`・`cmd/mod.rs`（隣の `impl From<BizError> for CmdError`） | 書き換えた |
| S1-065 | `docs/function-design/45-cmd-daily-report-import.md:246` | `src-tauri/src/lib.rs:277`・`:1272` | 正本 | `6b765f75`〈親〉・`6b765f75`〈親〉 | `export_bindings_to`・`run` | 書き換えた |
| S1-066 | `docs/function-design/55-ui-csv-import.md:35` | `useDailyReportImportFlow.ts:35`・`reducer.ts:56`・`reducer.ts:77`・`src/features/daily-report-import/types.ts:4` | 正本 | `6b765f75`〈親〉・`6b765f75`〈親〉・`6b765f75`〈親〉・`c85e135d`〈親〉 | `decideRecoverTo`・`dailyReportImportReducer`・`types.ts`（隣の `DailyReportErrorRecoverTo`） | 書き換えた |
| S1-067 | `docs/function-design/56-ui-daily-sales.md:375` | `compute-summary.ts:14-17` | 正本 | `902647b8`〈hit〉 | `computeSalesLineSummary` | 書き換えた（文の「防御的 if-else」は今の `switch` と食い違う → backlog の (8)） |
| S1-068 | `docs/function-design/56-ui-daily-sales.md:579` | `navigation.ts:76` | 正本 | `902647b8`〈hit〉 | `navigation.ts` の `ui-09a` | 書き換えた（候補は root `902647b8` だけで、76 行は `ui-09a` の項目の中） |
| S1-069 | `docs/function-design/56-ui-daily-sales.md:581` | `RootLayout.tsx:33` | 正本 | `902647b8`〈hit〉 | `deriveTitle` | 書き換えた |
| S1-070 | `docs/function-design/57-ui-monthly-sales.md:31` | `sales_service.rs:196-197` | 正本 | - | `get_monthly_sales` | 消えた（候補は root `902647b8` だけで、196〜197 行は `get_daily_sales` の部門の行で名指す語が無い）→ 最小の言い換え（名指す語 `prev_month_comparison = Some(prev_items)` は今の base で `get_monthly_sales` の中にだけ在る） |
| S1-071 | `docs/function-design/57-ui-monthly-sales.md:164` | `sort-items.test.ts:79-90` | 正本 | `902647b8`〈hit〉 | test「G-3: ranking field 残存確認」 | 書き換えた |
| S1-072 | `docs/function-design/57-ui-monthly-sales.md:422` | `sales_service.rs:196-197` | 正本 | - | `get_monthly_sales` | 消えた（S1-070 と同じ）→ 最小の言い換え（S1-070 と同じ根拠） |
| S1-073 | `docs/function-design/58-ui-stock-inquiry.md:613` | `DisposalPage.tsx:134` | 正本 | `c6167c4d`〈親〉 | `DisposalPage` の `returnTo` | 書き換えた |
| S1-074 | `docs/function-design/66-ui-stock-movements.md:71` | `product_repo.rs:1845` | 正本 | `c6167c4d`〈親〉 | test `test_search_products_req103_keyword_product_code` | 書き換えた（1845 行は test の名前の行で、名指す語「コード検索」はその test の 1846 行の comment） |
| S1-075 | `docs/function-design/71-mnt-backup.md:201` | `backup.rs:263` | 正本 | - | — | 消えた（今の `backup.rs` の `restore_backup` は `restore.rs` へ委ねる包みで、`.restore_backup` を持たない。文は manifest 導入前の実装の記録）→ 引用を外す |
| S1-076 | `docs/function-design/73-ui-stocktake.md:68` | `product_repo.rs:804-805`・`02-component-catalog.md:157` | 正本 | `a1e98114`〈親〉・`a1e98114`〈親〉 | `search_products` の `query.is_discontinued`、catalog の ③ テーブル | 書き換えた |
| S1-077 | `docs/msi-distribution-runbook.md:27` | `:201`・`:203`・`:1214`・`:1223`・`:1225`・`:1233`・`:1241` | 正本 | `a5fb9b16`〈親〉・`0223ef18`〈親〉・`294b0dfe`〈親〉・`294b0dfe`〈親〉 | `record_pre_window_fatal`（隣の symbol）・`run` の setup の順 | 書き換えた |
| S1-078 | `docs/quality/review-checklist.md:123` | `:86` | 変更履歴 | — | — | 残す（変更履歴の行） |
| S1-079 | `docs/design-system/01-decision-rules.md:362` | `node_modules/@radix-ui/react-alert-dialog/dist/index.d.mts:23` | 外部 library | — | @radix-ui/react-alert-dialog 1.1.15（`72d32d75` の lock） | 残す（library の版を足した観測） |
| S1-080 | `docs/function-design/53-ui-home.md:312` | `Plans.md L23` | 変更履歴 | — | — | 残す（変更履歴の行） |
| S1-081 | `docs/function-design/54-ui-shortcuts.md:107` | `` `src/components/ui/dialog.tsx` L62-69 `` | 正本 | `902647b8`〈hit〉 | `DialogContent` の `DialogPrimitive.Close` | 書き換えた（候補は root `902647b8` だけ） |
| S1-082 | `docs/function-design/56-ui-daily-sales.md:26` | `SCREEN_DESIGN.md L100` | 正本 | - | — | 消えた（候補は root `902647b8` だけで、100 行は日報取込みのウィザードの文。今の SCREEN_DESIGN.md に主動線のこの文は無い）→ 引用を外す・backlog の (8) |
| S1-083 | `docs/function-design/56-ui-daily-sales.md:580` | `mockup L313` | 正本 | `902647b8`〈hit〉 | mockup の「売上レポート」 | 書き換えた（`docs/screen_mockups.html` は節が無く、文が成り立つので `L313` を外した） |
| S1-084 | `docs/function-design/58-ui-stock-inquiry.md:2` | `SCREEN_DESIGN.md（在庫照会画面 L120-133`・`L167-176` | 正本 | -・- | — | 消えた（候補は root `902647b8` だけで、L120-133・L167-176 は PLU 書出し・操作ログ・棚卸しの節）→ 引用を外す（節の名「在庫照会画面」は文に残る） |
| S1-085 | `docs/function-design/58-ui-stock-inquiry.md:676` | `SCREEN_DESIGN.md L120-133` | 正本 | - | — | 消えた（S1-084 と同じ）→ 引用を外す |
| S1-086 | `docs/function-design/70-mnt-diagnostic-log.md:258` | `` `src-tauri/src/lib.rs` `run_startup_step` L84 `` | 正本 | `e1c668ad`〈親〉 | `run_startup_step`（隣の symbol） | 書き換えた |

#### S2 の行

| ID | 場所（`6a25f63f` の行） | 引用（`rg -o` の順） | 型 | 採った版 | 書き換え先 | 処置 |
|---|---|---|---|---|---|---|
| S2-001 | `src-tauri/src/biz/csv_import_service/time_evidence.rs:25` | `parse.rs:80` | comment | `09f65412`〈親〉 | `parse_and_validate` の `CSV_IMPORT_LINE_LIMIT` | 書き換えた |
| S2-002 | `src/components/ui/segmented-control.test.tsx:7` | `:443` | comment | `31c069d5`〈親〉 | DSR-22 | 書き換えた（隣の DSR-22 を残して `:443` を外した） |
| S2-003 | `src/components/ui/segmented-control.tsx:6` | `:443` | comment | `31c069d5`〈親〉 | DSR-22 | 書き換えた（隣の DSR-22 を残して `:443` を外した） |
| S2-004 | `src/features/monthly-sales/components/SummaryCardsBar.tsx:97` | `sales_service.rs:196-197` | comment | - | `get_monthly_sales` | 消えた（S1-070 と同じ）→ 最小の言い換え（S1-070 と同じ根拠） |
| S2-005 | `src/features/monthly-sales/hooks/useMonthlySalesReport.test.tsx:156` | `sales_service.rs:196-197` | comment | - | `get_monthly_sales` | 消えた（S1-070 と同じ）→ 最小の言い換え（S1-070 と同じ根拠） |
| S2-006 | `src/features/monthly-sales/hooks/useMonthlySalesReport.ts:76` | `sort-items.test.ts:79-90` | comment | `902647b8`〈hit〉 | test「G-3: ranking field 残存確認」 | 書き換えた |
| S2-007 | `src/features/monthly-sales/lib/compute-comparison.ts:10` | `sales_service.rs:196-197` | comment | - | `get_monthly_sales` | 消えた（S1-070 と同じ）→ 最小の言い換え（S1-070 と同じ根拠） |
| S2-008 | `src/features/products/ProductListPage.bulk-dialog-target.test.tsx:6` | `ProductListPage.tsx:303-309` | comment | `31c069d5`〈親〉 | `ProductListPage` の `PluBulkTargetConfirmDialog` の props | 書き換えた（`b2389b19` 起源の句は残す: latent bug の出どころの記録） |
| S2-009 | `src/features/products/ProductListPage.scroll-restoration.test.tsx:16` | `scroll-restoration.js:147` | comment（外部 library） | — | @tanstack/router-core 1.168.15（`b59d7331` の lock） | 残す（library の版を足した観測） |
| S2-010 | `src/features/products/ProductListPage.scroll-restoration.test.tsx:62` | `node_modules/happy-dom/lib/nodes/element/Element.js:133-174` | comment（外部 library） | — | happy-dom 20.10.6（`b59d7331` の lock） | 残す（library の版を足した観測） |
| S2-011 | `src/features/products/ProductListPage.scroll-restoration.test.tsx:64` | `html-element/HTMLElement.js:460`・`:468` | comment（外部 library） | — | happy-dom 20.10.6（`b59d7331` の lock） | 残す（library の版を足した観測） |
| S2-012 | `src/features/products/components/ProductForm.test.tsx:915` | `ProductForm.tsx:284` | comment | `689ac4c3`〈hit〉 | `ProductForm` の部門の `Select` の `onValueChange` | 書き換えた（親 `41336c28` の 284 行は native の `<select>` の開始 tag の閉じで要素が違い、test と `Select` 化が同じ commit の `689ac4c3` を採った） |
| S2-013 | `src/features/products/components/ProductForm.test.tsx:924` | `ProductForm.tsx:282` | comment | `689ac4c3`〈hit〉 | `ProductForm` の部門の `Select` の `value` | 書き換えた |
| S2-014 | `src/features/stocktake/StocktakePage.tsx:759` | `Pagination.tsx:48` | comment | `b59d7331`〈hit〉 | `Pagination` の早期 return | 書き換えた（`page > totalPages` の回復の条件を持つのは早期 return の版 `b59d7331`。親 `ff655efd` の 48 行は `canPrev`） |
| S2-015 | `src-tauri/src/lib.rs:244` | `.gitignore:38` | comment | `902647b8`〈hit〉 | `.gitignore` の `# src/lib/bindings.ts` の行 | 書き換えた（target は bindings の在る repo root の `.gitignore`。候補は root `902647b8` だけ） |
| S2-016 | `src/features/csv-import/components/AdditionalImportConfirmDialog.tsx:78` | `alert-dialog.tsx L53` | comment | `04514154`〈親〉 | `AlertDialogContent` | 書き換えた |
| S2-017 | `src/features/daily-report-import/DailyReportImportPage.test.tsx:400` | `ResultStep.tsx L67` | comment | `04514154`〈親〉 | `ResultStep` の action の container | 書き換えた |
| S2-018 | `src/features/integrity-check/IntegrityCheckPage.test.tsx:124` | `SCREEN_DESIGN L42`・`L62`・`L426` | comment | `5aad8f59`〈親〉・`04514154`〈親〉・`04514154`〈親〉 | SCREEN_DESIGN の「年に数回 / 初回のみ」・「全体構成（4エリア）」・「実装フェーズ分類」 | 書き換えた |
| S2-019 | `src/features/return-exchange/ReturnExchangePage.test.tsx:778` | `:147` | comment | `07302b5e`〈親〉 | `registerOptionClass`（隣の symbol） | 書き換えた（target は `registerOptionClass` を定義する `ReturnExchangePage.tsx`。test file には test の名前として出るだけ） |
| S2-020 | `src/features/stock-movements/StockMovementsPage.test.tsx:262` | `:98` | comment | `ff655efd`〈親〉 | `StockMovementsPage` の商品情報の section | 書き換えた（target は商品情報の card の要素を持つ `StockMovementsPage.tsx`。test file の `bg-card` は assertion の文字列） |
| S2-021 | `src/hooks/useUnsavedChangesWarning.test.tsx:154` | `:133` | comment | `1ad9aa74`〈親〉 | sibling test「DSR-20 D-E T10: Escape と外側 pointer-down では閉じず callback も発火しない」 | 書き換えた（target は comment の在る test file。「sibling test」は同じ file の test） |
| S2-022 | `src/hooks/useUnsavedChangesWarning.test.tsx:156` | `:133` | comment | `1ad9aa74`〈親〉 | 同じ sibling test | 書き換えた（S2-021 と同じ） |

#### S3 の行

| ID | 場所（`6a25f63f` の行） | 引用（`rg -o` の順） | 型 | 採った版 | 書き換え先 | 処置 |
|---|---|---|---|---|---|---|
| S3-001 | `docs/backlog.md:15` | `src-tauri/src/seed_demo.rs:462` | backlog | `44f1e666`〈親〉 | `44f1e666` の時点で `src-tauri/src/seed_demo.rs:462` | 書き換えた |
| S3-002 | `docs/backlog.md:27` | `parse.rs:159`・`tests.rs:275`・`:481`・`:536` | backlog | `3933f0d4`〈親〉・`3933f0d4`〈親〉・`3933f0d4`〈親〉・`3933f0d4`〈親〉 | `3933f0d4` の時点で `src-tauri/src/biz/daily_report_import_service/parse.rs:159`・`src-tauri/src/biz/daily_report_import_service/tests.rs:275`・`src-tauri/src/biz/daily_report_import_service/tests.rs:481`・`src-tauri/src/biz/daily_report_import_service/tests.rs:536` | 書き換えた |
| S3-003 | `docs/backlog.md:49` | `:164`・`:657`・`:658`・`:370`・`:196`・`:751`・`:754`・`:907`・`:30`・`:179`・`docs/function-design/62-ui-manual-sale.md:167` | backlog | `dda8560a`〈親〉・`6cac578c`〈親〉・`6cac578c`〈親〉・`31ec7501`〈hit〉・`6cac578c`〈親〉・`6cac578c`〈親〉・`85b18a04`〈親〉・`67870e10`〈親〉・`6cac578c`〈親〉 | `dda8560a` の時点で `docs/function-design/58-ui-stock-inquiry.md:164`・`6cac578c` の時点で `docs/function-design/58-ui-stock-inquiry.md:657-658`・`docs/function-design/58-ui-stock-inquiry.md:370`・`docs/function-design/20-io-product-repo.md:751-754`・`docs/function-design/20-io-product-repo.md:907`・`docs/function-design/62-ui-manual-sale.md:167`・`31ec7501` の時点で `docs/function-design/20-io-product-repo.md:196`・`85b18a04` の時点で `docs/function-design/51-ui-product-form.md:30`・`67870e10` の時点で `docs/function-design/51-ui-product-form.md:179` | 書き換えた |
| S3-004 | `docs/backlog.md:50` | `src-tauri/src/biz/inventory_service/manual_sale.rs:147` | backlog | `6cac578c`〈親〉 | `6cac578c` の時点で `src-tauri/src/biz/inventory_service/manual_sale.rs:147` | 書き換えた |
| S3-005 | `docs/backlog.md:51` | `src/features/manual-sale/lib/manual-sale-row-utils.ts:67`・`:82` | backlog | `6cac578c`〈親〉・`6cac578c`〈親〉 | `6cac578c` の時点で `src/features/manual-sale/lib/manual-sale-row-utils.ts:67`・`src/features/manual-sale/lib/manual-sale-row-utils.ts:82` | 書き換えた |
| S3-006 | `docs/backlog.md:53` | `docs/function-design/35-biz-stocktake-service.md:473`・`docs/function-design/44-cmd-inventory.md:966`・`docs/DB_DESIGN.md:187`・`:763`・`:1460`・`:1520`・`:1528`・`:1541`・`:1708`・`:452` | backlog | `67870e10`〈親〉・`02f497ea`〈親〉・`02f497ea`〈親〉・`02f497ea`〈親〉・`02f497ea`〈親〉・`02f497ea`〈親〉・`02f497ea`〈親〉・`02f497ea`〈親〉・`def86e19`〈親〉 | `67870e10` の時点で `docs/function-design/35-biz-stocktake-service.md:473`・`02f497ea` の時点で `docs/function-design/44-cmd-inventory.md:966`・`docs/DB_DESIGN.md:187`・`src-tauri/src/db/migration.rs:763`・`src-tauri/src/db/migration.rs:1460`・`src-tauri/src/db/migration.rs:1520`・`src-tauri/src/db/migration.rs:1528-1541`・`src-tauri/src/db/migration.rs:1708`・`def86e19` の時点で `src-tauri/src/db/schema_v2.rs:452` | 書き換えた（migration.rs と schema_v2.rs の行は、文が書くとおり 2026-10-17 の main〈`02f497ea` の親の版〉の `migrate(&conn)` の行） |
| S3-007 | `docs/backlog.md:57` | `src-tauri/src/db/schema_v8.rs:255`・`:10`・`:15`・`:14`・`:262`・`src-tauri/src/constants.rs:65` | backlog | `6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉・`6cac578c`〈hit〉 | `6cac578c` の時点で `src-tauri/src/db/schema_v8.rs:255`・`src-tauri/src/db/schema_v8.rs:10-15`・`src-tauri/src/db/schema_v8.rs:14`・`src-tauri/src/db/schema_v8.rs:262`・`src-tauri/src/constants.rs:65` | 書き換えた |
| S3-008 | `docs/backlog.md:77` | `src-tauri/src/biz/product_service.rs:1536`・`src-tauri/src/constants.rs:38` | backlog | `f5d7aaf4`〈親〉・`f5d7aaf4`〈親〉 | `f5d7aaf4` の時点で `src-tauri/src/biz/product_service.rs:1536`・`src-tauri/src/constants.rs:38` | 書き換えた |
| S3-009 | `docs/backlog.md:95` | `docs/function-design/21-io-inventory-repo.md:103`・`:354`・`docs/function-design/62-ui-manual-sale.md:26`・`:165`・`src/features/manual-sale/lib/manual-sale-row-utils.ts:24`・`:40` | backlog | `32273dc2`〈親〉・`32273dc2`〈親〉・`32273dc2`〈親〉・`32273dc2`〈親〉・`32273dc2`〈親〉・`32273dc2`〈親〉 | `32273dc2` の時点で `docs/function-design/21-io-inventory-repo.md:103`・`docs/function-design/21-io-inventory-repo.md:354`・`docs/function-design/62-ui-manual-sale.md:26`・`docs/function-design/62-ui-manual-sale.md:165`・`src/features/manual-sale/lib/manual-sale-row-utils.ts:24`・`src/features/manual-sale/lib/manual-sale-row-utils.ts:40` | 書き換えた |
| S3-010 | `docs/backlog.md:98` | `product_service.rs:153` | backlog | `249df0ca`〈親〉 | `249df0ca` の時点で `src-tauri/src/biz/product_service.rs:153` | 書き換えた |
| S3-011 | `docs/backlog.md:118` | `src/features/csv-import/lib/formatErrorRow.ts:22` | backlog | `ff05436f`〈親〉 | `ff05436f` の時点で `src/features/csv-import/lib/formatErrorRow.ts:22` | 書き換えた |
| S3-012 | `docs/backlog.md:119` | `src/features/products/components/PriceRevisionTable.tsx:104` | backlog | `07302b5e`〈親〉 | `07302b5e` の時点で `src/features/products/components/PriceRevisionTable.tsx:104` | 書き換えた |
| S3-013 | `docs/backlog.md:121` | `src/components/patterns/EmptyState.tsx:32` | backlog | `ff05436f`〈親〉 | `ff05436f` の時点で `src/components/patterns/EmptyState.tsx:32` | 書き換えた |
| S3-014 | `docs/backlog.md:129` | `:841`・`:70` | backlog | `51b1f484`〈hit〉・`51b1f484`〈親〉 | `51b1f484` の時点で `docs/design-system/02-component-catalog.md:841`・`51b1f484` の時点で `docs/Plans.md:70` | 書き換えた（`:70` の target は書かれた時点に backlog を含んでいた `docs/Plans.md` 自身。70 行は「C6 方向 badge は…撤回」） |
| S3-015 | `docs/backlog.md:145` | `docs/design-system/reference/mockup-d-lists.html:112` | backlog | `ff655efd`〈親〉 | `ff655efd` の時点で `docs/design-system/reference/mockup-d-lists.html:112` | 書き換えた |
| S3-016 | `docs/backlog.md:173` | `:155` | backlog | `294ba366`〈親〉 | `294ba366` の時点で `scripts/check-workflow-git.sh:155` | 書き換えた |
| S3-017 | `docs/backlog.md:183` | `PluExportPage.tsx:188` | backlog | `909fe096`〈hit〉 | `909fe096` の時点で `src/features/plu-export/PluExportPage.tsx:188` | 書き換えた |
| S3-018 | `docs/backlog.md:184` | `ReturnExchangePage.tsx:185` | backlog | - | — | 曖昧（候補のどの版でも 185 行は `max-lines-per-function` の要素でない）→ 最小の言い換え（名指す語 `max-lines-per-function` は今の base で `ReturnExchangePage.tsx` の `ReturnExchangePage` の上の 1 か所だけ） |
| S3-019 | `docs/backlog.md:188` | `src-tauri/src/biz/stocktake_service.rs:4`・`docs/function-design/35-biz-stocktake-service.md:374`・`docs/architecture/biz-task-specs.md:451`・`docs/db-design/tracking-system-tables.md:137`・`docs/screen_mockups.html:208`・`docs/screen_mockups.html:259`・`docs/SCREEN_DESIGN.md:213` | backlog | `a38597e8`〈親〉・`a3f08cdd`〈親〉・`a3f08cdd`〈親〉・`a3f08cdd`〈親〉・`a3f08cdd`〈親〉・`a3f08cdd`〈親〉・`e40e50b9`〈親〉 | `a38597e8` の時点で `src-tauri/src/biz/stocktake_service.rs:4`・`a3f08cdd` の時点で `docs/function-design/35-biz-stocktake-service.md:374`・`docs/architecture/biz-task-specs.md:451`・`docs/db-design/tracking-system-tables.md:137`・`docs/screen_mockups.html:208`・`docs/screen_mockups.html:259`・`e40e50b9` の時点で `docs/SCREEN_DESIGN.md:213` | 書き換えた |
| S3-020 | `docs/backlog.md:204` | `40-cmd-product.md:279` | backlog | `0a83374b`〈hit〉 | `0a83374b` の時点で `docs/function-design/40-cmd-product.md:279` | 書き換えた |
| S3-021 | `docs/backlog.md:223` | `61-ui-receiving.md:30`・`:73`・`63-ui-return-exchange.md:36`・`:71` | backlog | `949a5c4c`〈親〉・`949a5c4c`〈親〉・`949a5c4c`〈親〉・`949a5c4c`〈親〉 | `949a5c4c` の時点で `docs/function-design/61-ui-receiving.md:30`・`docs/function-design/61-ui-receiving.md:73`・`docs/function-design/63-ui-return-exchange.md:36`・`docs/function-design/63-ui-return-exchange.md:71` | 書き換えた |
| S3-022 | `docs/backlog.md:245` | `MonthlySalesPage.tsx:177-180` | backlog | `7e0ccf1b`〈親〉 | `7e0ccf1b` の時点で `src/features/monthly-sales/MonthlySalesPage.tsx:177-180` | 書き換えた |
| S3-023 | `docs/backlog.md:247` | `ReceivingPage.tsx:543-549` | backlog | `31c069d5`〈親〉 | `31c069d5` の時点で `src/features/receiving/ReceivingPage.tsx:543-549` | 書き換えた |
| S3-024 | `docs/backlog.md:249` | `ReturnExchangePage.tsx:790-791` | backlog | `31c069d5`〈親〉 | `31c069d5` の時点で `src/features/return-exchange/ReturnExchangePage.tsx:790-791` | 書き換えた（790〜791 行は `{errors.items}` を出す段落で、同じ版の `return-exchange-request.ts` が入れる「交換では戻り明細と渡し明細がそれぞれ必要です」を、文は言い換えて名指している） |
| S3-025 | `docs/backlog.md:251` | `ReturnExchangePage.tsx:624` | backlog | `31c069d5`〈親〉 | `31c069d5` の時点で `src/features/return-exchange/ReturnExchangePage.tsx:624` | 書き換えた（親 `31c069d5` の 624 行はレシート画像の `FilePicker`、hit `7e0ccf1b` の 624 行はその `Label` と `FilePicker` を包む div で、どちらも同じレシート画像の欄なので最古を採った） |
| S3-026 | `docs/backlog.md:259` | `InventoryRecordsPage.tsx:156-245`・`ReceivingPage.tsx:393`・`DisposalPage.tsx:509`・`OperationLogsPage.tsx:338-352` | backlog | `7e0ccf1b`〈親〉・`7e0ccf1b`〈親〉・`7e0ccf1b`〈親〉・`7e0ccf1b`〈親〉 | `7e0ccf1b` の時点で `src/features/inventory-records/InventoryRecordsPage.tsx:156-245`・`src/features/receiving/ReceivingPage.tsx:393`・`src/features/disposal/DisposalPage.tsx:509`・`src/features/operation-logs/OperationLogsPage.tsx:338-352` | 書き換えた |
| S3-027 | `docs/backlog.md:261` | `Pagination.tsx:52-53`・`:99` | backlog | `7e0ccf1b`〈親〉・`7e0ccf1b`〈親〉 | `7e0ccf1b` の時点で `src/components/patterns/Pagination.tsx:52-53`・`src/components/patterns/Pagination.tsx:99` | 書き換えた |
| S3-028 | `docs/backlog.md:299` | `ProductImportPreview.tsx:76`・`ProductRankingTable.tsx:80`・`BackupRestorePage.tsx:533` | backlog | `7ae4ade6`〈親〉・`7ae4ade6`〈親〉・`7ae4ade6`〈親〉 | `7ae4ade6` の時点で `src/features/products/import/ProductImportPreview.tsx:76`・`src/features/monthly-sales/components/ProductRankingTable.tsx:80`・`src/features/backup-restore/BackupRestorePage.tsx:533` | 書き換えた |
| S3-029 | `docs/backlog.md:303` | `StockStatusBadge.tsx:42`・`01-decision-rules.md:445`・`:451`・`04-backbone.md:44`・`02-component-catalog.md:158`・`catalog:588`・`DailyReportImportPage.tsx:322`・`ProductTable.tsx:133` | backlog | `7ae4ade6`〈親〉・`7ae4ade6`〈親〉・`7ae4ade6`〈親〉・`7ae4ade6`〈親〉・`7ae4ade6`〈親〉・`7ae4ade6`〈親〉・`7ae4ade6`〈親〉・`7ae4ade6`〈親〉 | `7ae4ade6` の時点で `src/features/stock-inquiry/components/StockStatusBadge.tsx:42`・`docs/design-system/01-decision-rules.md:445`・`docs/design-system/01-decision-rules.md:451`・`docs/design-system/04-backbone.md:44`・`docs/design-system/02-component-catalog.md:158`・`docs/design-system/02-component-catalog.md:588`・`src/features/daily-report-import/DailyReportImportPage.tsx:322`・`src/features/daily-sales/components/ProductTable.tsx:133` | 書き換えた |
| S3-030 | `docs/backlog.md:334` | `button.tsx:15-16` | backlog | `7e0ccf1b`〈親〉 | `7e0ccf1b` の時点で `src/components/ui/button.tsx:15-16` | 書き換えた |
| S3-031 | `docs/backlog.md:341` | `list.rs:21`・`archive/plans/2026-09-03-ui-list-backbone-d-lane2.md):642-648` | backlog | `31c069d5`〈親〉・- | `31c069d5` の時点で `src-tauri/src/biz/inventory_service/list.rs:21` | 書き換えた（`):642-648` は archive の packet の 642〜648 行が「owner run 3 所感」でなく Gated Amendment 2 の対応の節で、取り戻せない → 引用を外した。link は残る） |

#### 照合の入力

S5 の全引用の照合の evidence の入力（形は S5。実装の run が埋める）。

```text 照合の入力
# ID 採った版 target N W X
S1-004 654f95d5 src/features/stocktake/StocktakePage.tsx 933 text-3xl StocktakePage.tsx:933
S1-005 ff655efd src/components/patterns/ListShell.tsx 69 topSummary ListShell.tsx:69
S1-006 c9fabc17 src/features/inventory-records/InventoryRecordsPage.tsx 356 記録日時 InventoryRecordsPage.tsx:356
S1-006 c9fabc17 src/features/inventory-records/InventoryRecordsPage.tsx 353 代表商品 `:353
S1-006 c9fabc17 src/features/inventory-records/InventoryRecordsPage.tsx 349-359 業務日付 `:349
S1-007 654f95d5 src/features/stock-movements/StockMovementsPage.tsx 73-101 PageHeader StockMovementsPage.tsx:73
S1-007 654f95d5 src/features/stock-movements/components/MovementTable.tsx 52-53 日時 MovementTable.tsx:52
S1-008 654f95d5 src/features/operation-logs/OperationLogsPage.tsx 461-462 日時 OperationLogsPage.tsx:461
S1-009 654f95d5 src/features/suppliers/components/SupplierUsageTable.tsx 19 取引先名 SupplierUsageTable.tsx:19
S1-010 654f95d5 src/features/backup-restore/BackupRestorePage.tsx 520 日時 BackupRestorePage.tsx:520
S1-010 654f95d5 src/features/backup-restore/BackupRestorePage.tsx 522 ファイル名 BackupRestorePage.tsx:520,522
S1-011 9f90964d docs/design-system/reference/2026-08-23-current-design-analysis.md 18 仮置き 2026-08-23-current-design-analysis.md:18
S1-012 43b69bea src/features/inventory-records/InventoryRecordsPage.tsx 349-359 記録ID InventoryRecordsPage.tsx:349
S1-013 7ae4ade6 src/features/stock-inquiry/components/StockStatusBadge.tsx 41-44 通常 StockStatusBadge.tsx:41
S1-014 654f95d5 src/components/layout/RootLayout.tsx 62 aside RootLayout.tsx:62
S1-016 f1d5590c src/components/layout/RootLayout.tsx 65 data-scroll-restoration-id RootLayout.tsx:65
S1-021 c9fabc17 src/features/suppliers/SupplierManagementPage.tsx 35-39 subtitle SupplierManagementPage.tsx:35
S1-022 43b69bea src/features/receiving/ReceivingPage.tsx 295 PageHeader ReceivingPage.tsx:295
S1-022 43b69bea src/features/manual-sale/ManualSalePage.tsx 310 PageHeader ManualSalePage.tsx:310
S1-022 43b69bea src/features/return-exchange/ReturnExchangePage.tsx 419 返品・交換 ReturnExchangePage.tsx:419
S1-022 43b69bea src/features/disposal/DisposalPage.tsx 285 PageHeader DisposalPage.tsx:285
S1-022 c9fabc17 src/features/suppliers/SupplierManagementPage.tsx 35-39 subtitle SupplierManagementPage.tsx:35
S1-022 c9fabc17 src/components/patterns/PageHeader.tsx 47-53 description PageHeader.tsx:47
S1-022 bb1862a5 src/components/patterns/PageHeader.tsx 31-45 space-y-1 PageHeader.tsx:31
S1-023 1c896139 src/features/stock-movements/components/MovementTable.tsx 93-99 — MovementTable.tsx:93
S1-023 43b69bea src/features/operation-logs/OperationLogsPage.tsx 533 truncate OperationLogsPage.tsx:533
S1-023 c9fabc17 src/features/stock-movements/components/MovementTable.tsx 93-99 break-words MovementTable.tsx:93
S1-023 1c896139 src/features/inventory-records/ManualSaleRecordDetailPage.tsx 149 detail.note ManualSaleRecordDetailPage.tsx:149
S1-023 1c896139 src/features/stock-movements/components/MovementTable.tsx 82-91 movement.source MovementTable.tsx:82
S1-023 1c896139 src/features/inventory-records/StocktakeRecordDetailPage.tsx 234 MovementTable StocktakeRecordDetailPage.tsx:234
S1-024 2c125d1a src/features/products/PriceRevisionPage.tsx 79-83 Alert PriceRevisionPage.tsx:79
S1-024 7ae4ade6 src/features/daily-sales/DailySalesPage.tsx 175 未取込み DailySalesPage.tsx:175
S1-024 7ae4ade6 src/features/monthly-sales/MonthlySalesPage.tsx 166 未取込み MonthlySalesPage.tsx:166
S1-025 eabc549c src/features/stock-inquiry/components/StockStatusBadge.tsx 29 在庫少 StockStatusBadge.tsx:29
S1-025 7ae4ade6 src/features/stocktake/StocktakePage.tsx 396-401 未入力 StocktakePage.tsx:396
S1-025 7ae4ade6 src/features/csv-import/components/PreviewStep.tsx 75-80 同日データあり csv-import/components/PreviewStep.tsx:75
S1-025 2c125d1a src/features/products/components/ProductTable.tsx 88 未反映 ProductTable.tsx:88
S1-025 7ae4ade6 src/features/csv-import/components/ResultStep.tsx 47 部分成功 ResultStep.tsx:47
S1-025 07302b5e src/features/inventory-records/CsvImportRecordDetailPage.tsx 140 STATUS_LABELS CsvImportRecordDetailPage.tsx:140
S1-026 2c125d1a src/features/integrity-check/IntegrityCheckPage.tsx 389 Badge IntegrityCheckPage.tsx:389
S1-026 7ae4ade6 src/features/stocktake/StocktakePage.tsx 404 bg-success StocktakePage.tsx:404
S1-026 2c125d1a src/features/products/components/ProductTable.tsx 93 Badge ProductTable.tsx:93
S1-026 7ae4ade6 src/features/csv-import/components/ResultStep.tsx 47 isPartial ResultStep.tsx:47
S1-026 7ae4ade6 src/features/daily-report-import/DailyReportImportPage.tsx 164 variant DailyReportImportPage.tsx:164
S1-026 7ae4ade6 src/features/daily-report-import/DailyReportImportPage.tsx 186 確認済み DailyReportImportPage.tsx:164,186
S1-026 07302b5e src/features/inventory-records/CsvImportRecordDetailPage.tsx 140 STATUS_LABELS CsvImportRecordDetailPage.tsx:140
S1-026 7ae4ade6 src/features/daily-report-import/DailyReportImportPage.tsx 322 成功 DailyReportImportPage.tsx:322
S1-027 eabc549c src/features/stock-inquiry/components/StockStatusBadge.tsx 20 在庫切れ StockStatusBadge.tsx:20
S1-027 7ae4ade6 src/features/inventory-records/CsvImportRecordDetailPage.tsx 41 取消済み CsvImportRecordDetailPage.tsx:41
S1-027 7ae4ade6 src/features/inventory-records/CsvImportRecordDetailPage.tsx 140 STATUS_LABELS CsvImportRecordDetailPage.tsx:41,140
S1-027 1ad9aa74 src/features/inventory-records/CsvImportRecordDetailPage.tsx 192 明細取消済み `:192
S1-028 f2ef9e52 src/features/stock-inquiry/components/StockStatusBadge.tsx 36 在庫あり StockStatusBadge.tsx:36
S1-028 7ae4ade6 src/features/inventory-records/types.ts 87-94 formatRecordStatus inventory-records/types.ts:87
S1-029 c1c1c5bf src/features/integrity-check/IntegrityCheckPage.tsx 469 fieldsLabel IntegrityCheckPage.tsx:469
S1-029 07302b5e src/features/products/components/PriceRevisionTable.tsx 104 入力中 PriceRevisionTable.tsx:104
S1-029 07302b5e src/features/products/components/ProductTable.tsx 74 Badge ProductTable.tsx:74
S1-029 7ae4ade6 src/features/inventory-records/CsvImportRecordDetailPage.tsx 194 有効 CsvImportRecordDetailPage.tsx:194
S1-029 2c125d1a src/features/return-exchange/ReturnExchangePage.tsx 97 レジ未処理 ReturnExchangePage.tsx:97
S1-029 2c125d1a src/features/return-exchange/ReturnExchangePage.tsx 592 aria-label ReturnExchangePage.tsx:592
S1-029 2c125d1a src/features/return-exchange/ReturnExchangePage.tsx 602 レジ未処理 ReturnExchangePage.tsx:602
S1-030 7ae4ade6 src/features/products/components/ProductTable.tsx 56 廃番 ProductTable.tsx:56
S1-030 07302b5e src/features/products/components/ProductTable.tsx 74 Badge ProductTable.tsx:74
S1-030 7ae4ade6 src/features/products/components/PriceRevisionTable.tsx 98 Badge PriceRevisionTable.tsx:98
S1-030 7ae4ade6 src/features/daily-sales/components/ProductTable.tsx 133 Badge daily-sales/components/ProductTable.tsx:133
S1-031 07302b5e src/features/products/components/PriceRevisionTable.tsx 104 入力中 PriceRevisionTable.tsx:104
S1-033 6cac578c src-tauri/src/biz/stocktake_service.rs 529 棚卸し補正 stocktake_service.rs:529
S1-033 67870e10 src/features/stock-inquiry/lib/filter-low-stock-list.ts 62-63 stock_quantity filter-low-stock-list.ts:62
S1-033 67870e10 src/features/products/search.ts 30 在庫数 search.ts:30
S1-034 6cac578c src-tauri/src/biz/inventory_service/receiving.rs 126 MAX_LINE_QUANTITY receiving.rs:126
S1-034 6cac578c src-tauri/src/biz/inventory_service/manual_sale.rs 141 MAX_LINE_QUANTITY manual_sale.rs:141
S1-034 6cac578c src-tauri/src/biz/inventory_service/disposal.rs 123 MAX_LINE_QUANTITY disposal.rs:123
S1-034 6cac578c src-tauri/src/biz/inventory_service/returns.rs 99 MAX_LINE_QUANTITY returns.rs:99
S1-034 6cac578c src-tauri/src/biz/inventory_service/returns.rs 254-255 register_processed returns.rs:254
S1-035 6cac578c src-tauri/src/biz/csv_import_service/parse.rs 98-110 MAX_LINE_QUANTITY parse.rs:98
S1-035 c85e135d src-tauri/src/biz/csv_import_service/commit.rs 138-167 pos_stock_sync commit.rs:138
S1-036 6cac578c src-tauri/src/biz/inventory_service/common.rs 82-87 check_stock_range common.rs:82
S1-036 6cac578c src-tauri/src/biz/product_service.rs 1062-1069 MAX_STOCK_QUANTITY product_service.rs:1062
S1-036 6cac578c src-tauri/src/biz/product_service.rs 1316-1324 MAX_STOCK_QUANTITY `:1316
S1-036 6cac578c src-tauri/src/biz/csv_import_service/commit.rs 276 check_stock_range commit.rs:276
S1-036 6cac578c src-tauri/src/biz/integrity_service.rs 192 check_stock_range integrity_service.rs:192
S1-037 6cac578c src-tauri/src/biz/product_service.rs 949-957 price_range_error product_service.rs:949
S1-037 6cac578c src-tauri/src/biz/product_service.rs 1059 selling_and_cost_range_error `:1059
S1-037 6cac578c src-tauri/src/biz/product_service.rs 1103 price_range_error `:1103
S1-037 6cac578c src-tauri/src/biz/product_service.rs 473-474 selling_and_cost_range_error `:473
S1-037 6cac578c src-tauri/src/biz/product_service.rs 1285 price_range_error `:1285
S1-038 6cac578c src-tauri/src/biz/product_service.rs 1059 selling_and_cost_range_error product_service.rs:1059
S1-038 6cac578c src-tauri/src/biz/product_service.rs 1106 price_range_error product_service.rs:1059`・`:1106
S1-038 6cac578c src-tauri/src/biz/product_service.rs 473-474 selling_and_cost_range_error product_service.rs:1059`・`:1106`・`:473
S1-038 6cac578c src-tauri/src/biz/product_service.rs 1291 price_range_error product_service.rs:1059`・`:1106`・`:473`〜`:474`・`:1291
S1-038 6cac578c src-tauri/src/biz/inventory_service/receiving.rs 138 MAX_COST_PRICE receiving.rs:138
S1-038 6cac578c src-tauri/src/biz/inventory_service/disposal.rs 135 MAX_COST_PRICE disposal.rs:135
S1-039 c1beb334 src/features/products/lib/price-revision-math.ts 1-9 deriveProposedCost price-revision-math.ts:1
S1-040 67870e10 src-tauri/src/biz/product_service.rs 907-927 list_low_stock product_service.rs:907
S1-040 6cac578c src-tauri/src/db/product_repo.rs 1310-1311 stock_quantity product_repo.rs:1310
S1-042 6cac578c src-tauri/src/biz/product_service.rs 274-285 initial_stock product_service.rs:274
S1-043 67870e10 src-tauri/src/db/schema_time_evidence.rs 120-124 system_stock schema_time_evidence.rs:120
S1-044 02f497ea src-tauri/src/lib.rs 1214-1223 app_data_dir lib.rs:1214
S1-044 02f497ea src-tauri/src/lib.rs 161 inventory.db lib.rs:161
S1-044 02f497ea src-tauri/src/db/mod.rs 298 migrating mod.rs:298
S1-044 02f497ea src-tauri/src/mnt/restore.rs 11-13 restore_manifest restore.rs:11
S1-044 02f497ea src-tauri/src/mnt/backup.rs 206-210 backups backup.rs:206
S1-044 02f497ea src-tauri/src/lib.rs 1227 logs lib.rs:1227
S1-044 02f497ea src-tauri/src/io/image_manager.rs 15 receipts image_manager.rs:15
S1-044 02f497ea src-tauri/src/io/image_manager.rs 58 RECEIPTS_DIR image_manager.rs:15`・`:58
S1-045 02f497ea src-tauri/src/lib.rs 118-120 current_dir lib.rs:118
S1-045 02f497ea src-tauri/src/lib.rs 165-183 new_db_exists lib.rs:165
S1-045 02f497ea src-tauri/src/db/mod.rs 296 old_db mod.rs:296
S1-046 02f497ea src-tauri/src/lib.rs 1219-1223 create_dir_all lib.rs:1219
S1-047 02f497ea src-tauri/src/mnt/restore.rs 690-700 CommittedPendingLog restore.rs:690
S1-047 02f497ea src-tauri/src/lib.rs 188-195 complete_reconciled_restore lib.rs:188
S1-048 3933f0d4 src-tauri/src/db/sales_repo.rs 760 completed sales_repo.rs:760
S1-049 3933f0d4 src-tauri/src/biz/mod.rs 26 CsvImport mod.rs:26
S1-049 6cac578c src-tauri/src/biz/mod.rs 28-30 Stocktake `:28
S1-049 3933f0d4 src-tauri/src/db/sales_repo.rs 13-19 CsvImportStatus sales_repo.rs:13
S1-049 3933f0d4 src-tauri/src/db/sales_repo.rs 862-873 CsvImportStatus sales_repo.rs:13`〜`:19`・`:862
S1-050 9d351d5b src-tauri/src/io/ej_parser.rs 41 quantity `:41
S1-050 c1beb334 src-tauri/src/io/ej_parser.rs 190 quantity `:190
S1-050 c1beb334 src-tauri/src/io/ej_parser.rs 57 ItemCount `:57
S1-050 c1beb334 src-tauri/src/io/ej_parser.rs 1289 parse_item_count `:1289
S1-050 c1beb334 src-tauri/src/io/ej_parser.rs 174 item_count `:174
S1-050 c1beb334 src-tauri/src/io/ej_parser.rs 776 (1, `:776
S1-050 c1beb334 src-tauri/src/io/ej_parser.rs 814 -1 `:814
S1-050 c1beb334 src-tauri/src/io/ej_parser.rs 707-717 quantity_sum `:707
S1-051 249df0ca src-tauri/src/biz/csv_import_service/commit.rs 157 apply_stock_change commit.rs:157
S1-052 6cac578c src-tauri/src/biz/inventory_service/receiving.rs 126 MAX_LINE_QUANTITY receiving.rs:126
S1-052 6cac578c src-tauri/src/biz/inventory_service/manual_sale.rs 141 MAX_LINE_QUANTITY manual_sale.rs:141
S1-052 6cac578c src-tauri/src/biz/inventory_service/disposal.rs 123 MAX_LINE_QUANTITY disposal.rs:123
S1-052 6cac578c src-tauri/src/biz/inventory_service/returns.rs 99 MAX_LINE_QUANTITY returns.rs:99
S1-052 6cac578c src-tauri/src/biz/inventory_service/returns.rs 254-255 register_processed returns.rs:254
S1-052 6cac578c src-tauri/src/biz/inventory_service/receiving.rs 156 check_safe_integer receiving.rs:156
S1-052 6cac578c src-tauri/src/biz/inventory_service/disposal.rs 159 check_safe_integer disposal.rs:159
S1-053 6cac578c src-tauri/src/biz/inventory_service/receiving.rs 138 MAX_COST_PRICE receiving.rs:138
S1-053 6cac578c src-tauri/src/biz/inventory_service/disposal.rs 135 MAX_COST_PRICE disposal.rs:135
S1-054 6cac578c src-tauri/src/biz/inventory_service/receiving.rs 234 negative_stock_warning_message receiving.rs:234
S1-054 6cac578c src-tauri/src/biz/inventory_service/returns.rs 273 negative_stock_warning_message returns.rs:273
S1-054 6cac578c src-tauri/src/biz/inventory_service/manual_sale.rs 289 negative_stock_warning_message manual_sale.rs:289
S1-054 6cac578c src-tauri/src/biz/inventory_service/disposal.rs 226 negative_stock_warning_message disposal.rs:226
S1-055 ea15360d src-tauri/src/biz/csv_import_service/parse.rs 63-77 未来 parse.rs:63
S1-056 6b765f75 src-tauri/src/biz/mod.rs 54 BizError mod.rs:54
S1-056 6b765f75 src/features/daily-report-import/hooks/useDailyReportImportFlow.ts 35 decideRecoverTo useDailyReportImportFlow.ts:35
S1-057 6b765f75 src/lib/bindings.ts 558 DailyReportSourceFileInfo bindings.ts:558
S1-057 6b765f75 src-tauri/src/biz/daily_report_import_service/commit.rs 83-85 source_files_json commit.rs:83
S1-058 3933f0d4 src-tauri/src/biz/daily_report_import_service/parse.rs 159 AdditionalImportConfirmationRequired parse.rs:159
S1-058 3933f0d4 src-tauri/src/biz/daily_report_import_service/tests.rs 275 AdditionalImportConfirmationRequired tests.rs:275
S1-058 3933f0d4 src-tauri/src/biz/daily_report_import_service/tests.rs 481 AdditionalImportConfirmationRequired tests.rs:275`・`:481
S1-058 3933f0d4 src-tauri/src/biz/daily_report_import_service/tests.rs 536 AdditionalImportConfirmationRequired tests.rs:275`・`:481`・`:536
S1-059 3933f0d4 src-tauri/src/db/sales_repo.rs 13-19 CsvImportStatus sales_repo.rs:13
S1-059 3933f0d4 src-tauri/src/db/sales_repo.rs 862-873 CsvImportStatus sales_repo.rs:13〜:19・:862
S1-060 3933f0d4 src-tauri/src/io/ej_parser.rs 455-460 printed_at ej_parser.rs:455
S1-061 3933f0d4 src-tauri/src/io/ej_parser.rs 678-689 NoItems ej_parser.rs:678
S1-061 3933f0d4 src-tauri/src/io/ej_parser.rs 575-581 MissingFinalNewline `:575
S1-062 3933f0d4 src-tauri/src/io/ej_parser.rs 832-834 adjustments `:832
S1-063 6b765f75 src-tauri/src/cmd/daily_report_import_cmd.rs 43 DailyReportInputFile daily_report_import_cmd.rs:43
S1-063 e7c22f8f src-tauri/src/cmd/daily_report_import_cmd.rs 242 request `:242
S1-064 6b765f75 src/features/daily-report-import/hooks/useDailyReportImportFlow.ts 35 decideRecoverTo useDailyReportImportFlow.ts:35
S1-064 0989ced0 src-tauri/src/cmd/mod.rs 130 From mod.rs:130
S1-065 6b765f75 src-tauri/src/lib.rs 277 collect_commands lib.rs:277
S1-065 6b765f75 src-tauri/src/lib.rs 1272 generate_handler `:1272
S1-066 6b765f75 src/features/daily-report-import/hooks/useDailyReportImportFlow.ts 35 decideRecoverTo useDailyReportImportFlow.ts:35
S1-066 6b765f75 src/features/daily-report-import/reducer.ts 56 import_failed reducer.ts:56
S1-066 6b765f75 src/features/daily-report-import/reducer.ts 77 dismiss_error reducer.ts:77
S1-066 c85e135d src/features/daily-report-import/types.ts 4 DailyReportErrorRecoverTo types.ts:4
S1-067 902647b8 src/features/daily-sales/lib/compute-summary.ts 14-17 source compute-summary.ts:14
S1-068 902647b8 src/config/navigation.ts 76 日次売上 navigation.ts:76
S1-069 902647b8 src/components/layout/RootLayout.tsx 33 title RootLayout.tsx:33
S1-070 - src-tauri/src/biz/sales_service.rs 196-197 prev_month_comparison sales_service.rs:196
S1-071 902647b8 src/features/monthly-sales/lib/sort-items.test.ts 79-90 ranking sort-items.test.ts:79
S1-072 - src-tauri/src/biz/sales_service.rs 196-197 prev_month_comparison sales_service.rs:196
S1-073 c6167c4d src/features/disposal/DisposalPage.tsx 134 returnTo DisposalPage.tsx:134
S1-074 c6167c4d src-tauri/src/db/product_repo.rs 1845 コード検索 product_repo.rs:1845
S1-075 - src-tauri/src/mnt/backup.rs 263 restore_backup backup.rs:263
S1-076 a1e98114 src-tauri/src/db/product_repo.rs 804-805 is_discontinued product_repo.rs:804
S1-076 a1e98114 docs/design-system/02-component-catalog.md 157 廃番 02-component-catalog.md:157
S1-077 a5fb9b16 src-tauri/src/lib.rs 201-203 record_pre_window_fatal `:201
S1-077 0223ef18 src-tauri/src/lib.rs 1214-1223 app_data_dir `:1214
S1-077 294b0dfe src-tauri/src/lib.rs 1225-1233 log_config `:1225
S1-077 294b0dfe src-tauri/src/lib.rs 1241 prepare_database `:1241
S1-081 902647b8 src/components/ui/dialog.tsx 62-69 DialogPrimitive.Close dialog.tsx` L62
S1-082 - docs/SCREEN_DESIGN.md 100 売上レポート確認 SCREEN_DESIGN.md L100
S1-083 902647b8 docs/screen_mockups.html 313 売上レポート mockup L313
S1-084 - docs/SCREEN_DESIGN.md 120-133 在庫照会画面 SCREEN_DESIGN.md（在庫照会画面 L120
S1-084 - docs/SCREEN_DESIGN.md 167-176 在庫照会画面 SCREEN_DESIGN.md（在庫照会画面 L120-133 / L167
S1-085 - docs/SCREEN_DESIGN.md 120-133 詳細カード SCREEN_DESIGN.md L120
S1-086 e1c668ad src-tauri/src/lib.rs 84 run_startup_step lib.rs` `run_startup_step` L84
S2-001 09f65412 src-tauri/src/biz/csv_import_service/parse.rs 80 CSV_IMPORT_LINE_LIMIT parse.rs:80
S2-002 31c069d5 docs/design-system/01-decision-rules.md 443 segmented `:443
S2-003 31c069d5 docs/design-system/01-decision-rules.md 443 segmented `:443
S2-004 - src-tauri/src/biz/sales_service.rs 196-197 prev_month_comparison sales_service.rs:196
S2-005 - src-tauri/src/biz/sales_service.rs 196-197 prev_month_comparison sales_service.rs:196
S2-006 902647b8 src/features/monthly-sales/lib/sort-items.test.ts 79-90 G-3 sort-items.test.ts:79
S2-007 - src-tauri/src/biz/sales_service.rs 196-197 prev_month_comparison sales_service.rs:196
S2-008 31c069d5 src/features/products/ProductListPage.tsx 303-309 bulkTarget ProductListPage.tsx:303
S2-012 689ac4c3 src/features/products/components/ProductForm.tsx 284 Number ProductForm.tsx:284
S2-013 689ac4c3 src/features/products/components/ProductForm.tsx 282 departmentId ProductForm.tsx:282
S2-014 b59d7331 src/components/patterns/Pagination.tsx 48 totalPages Pagination.tsx:48
S2-015 902647b8 .gitignore 38 bindings.ts .gitignore:38
S2-016 04514154 src/components/ui/alert-dialog.tsx 53 max-w-lg alert-dialog.tsx L53
S2-017 04514154 src/features/csv-import/components/ResultStep.tsx 67 flex-wrap ResultStep.tsx L67
S2-018 5aad8f59 docs/SCREEN_DESIGN.md 42 整合性 SCREEN_DESIGN L42
S2-018 04514154 docs/SCREEN_DESIGN.md 62 整合性 SCREEN_DESIGN L42・62
S2-018 04514154 docs/SCREEN_DESIGN.md 426 整合性 SCREEN_DESIGN L42・62・426
S2-019 07302b5e src/features/return-exchange/ReturnExchangePage.tsx 147 bg-background （:147
S2-020 ff655efd src/features/stock-movements/StockMovementsPage.tsx 98 bg-card （:98
S2-021 1ad9aa74 src/hooks/useUnsavedChangesWarning.test.tsx 133 T10 // :133
S2-022 1ad9aa74 src/hooks/useUnsavedChangesWarning.test.tsx 133 T10 // :133
S3-001 44f1e666 src-tauri/src/seed_demo.rs 462 inventory_movements seed_demo.rs:462
S3-002 3933f0d4 src-tauri/src/biz/daily_report_import_service/parse.rs 159 AdditionalImportConfirmationRequired parse.rs:159
S3-002 3933f0d4 src-tauri/src/biz/daily_report_import_service/tests.rs 275 AdditionalImportConfirmationRequired tests.rs:275
S3-002 3933f0d4 src-tauri/src/biz/daily_report_import_service/tests.rs 481 AdditionalImportConfirmationRequired tests.rs:275`・`:481
S3-002 3933f0d4 src-tauri/src/biz/daily_report_import_service/tests.rs 536 AdditionalImportConfirmationRequired tests.rs:275`・`:481`・`:536
S3-003 dda8560a docs/function-design/58-ui-stock-inquiry.md 164 format-stock-display `:164
S3-003 6cac578c docs/function-design/58-ui-stock-inquiry.md 657-658 stock_unit `:657
S3-003 6cac578c docs/function-design/58-ui-stock-inquiry.md 370 長さの単位 `:370
S3-003 31ec7501 docs/function-design/20-io-product-repo.md 196 stock_unit `:196
S3-003 6cac578c docs/function-design/20-io-product-repo.md 751-754 SUM `:751
S3-003 6cac578c docs/function-design/20-io-product-repo.md 907 find_all_stock_quantities `:907
S3-003 85b18a04 docs/function-design/51-ui-product-form.md 30 stock_unit='cm' `:30
S3-003 67870e10 docs/function-design/51-ui-product-form.md 179 数量単位を選択してください `:179
S3-003 6cac578c docs/function-design/62-ui-manual-sale.md 167 cm 62-ui-manual-sale.md:167
S3-004 6cac578c src-tauri/src/biz/inventory_service/manual_sale.rs 147 amount manual_sale.rs:147
S3-005 6cac578c src/features/manual-sale/lib/manual-sale-row-utils.ts 67 selling_price manual-sale-row-utils.ts:67
S3-005 6cac578c src/features/manual-sale/lib/manual-sale-row-utils.ts 82 unitPrice manual-sale-row-utils.ts:67`・`:82
S3-006 67870e10 docs/function-design/35-biz-stocktake-service.md 473 10 35-biz-stocktake-service.md:473
S3-006 02f497ea docs/function-design/44-cmd-inventory.md 966 10 44-cmd-inventory.md:966
S3-006 02f497ea docs/DB_DESIGN.md 187 10 DB_DESIGN.md:187
S3-006 02f497ea src-tauri/src/db/migration.rs 763 migrate(&conn) `:763
S3-006 02f497ea src-tauri/src/db/migration.rs 1460 migrate(&conn) `:1460
S3-006 02f497ea src-tauri/src/db/migration.rs 1520 migrate(&conn) `:1460`・`:1520
S3-006 02f497ea src-tauri/src/db/migration.rs 1528-1541 migrate(&conn) `:1528
S3-006 02f497ea src-tauri/src/db/migration.rs 1708 migrate(&conn) `:1708
S3-006 def86e19 src-tauri/src/db/schema_v2.rs 452 migrate(&conn) `:452
S3-007 6cac578c src-tauri/src/db/schema_v8.rs 255 test_v8_req903_limits_match_runtime_constants schema_v8.rs:255
S3-007 6cac578c src-tauri/src/db/schema_v8.rs 10-15 MAX_LINE_QUANTITY `:10
S3-007 6cac578c src-tauri/src/db/schema_v8.rs 14 MAX_COST_PRICE `:14
S3-007 6cac578c src-tauri/src/db/schema_v8.rs 262 MAX_COST_PRICE `:262
S3-007 6cac578c src-tauri/src/constants.rs 65 MAX_COST_PRICE constants.rs:65
S3-008 f5d7aaf4 src-tauri/src/biz/product_service.rs 1536 release_plu_slot_for_jan product_service.rs:1536
S3-008 f5d7aaf4 src-tauri/src/constants.rs 38 PLU_CLEAR_ROW_ENABLED constants.rs:38
S3-009 32273dc2 docs/function-design/21-io-inventory-repo.md 103 cost_price 21-io-inventory-repo.md:103
S3-009 32273dc2 docs/function-design/21-io-inventory-repo.md 354 line_loss_cost `:354
S3-009 32273dc2 docs/function-design/62-ui-manual-sale.md 26 UI-04-D6 62-ui-manual-sale.md:26
S3-009 32273dc2 docs/function-design/62-ui-manual-sale.md 165 UI-04-D6 62-ui-manual-sale.md:26`・`:165
S3-009 32273dc2 src/features/manual-sale/lib/manual-sale-row-utils.ts 24 selling_price manual-sale-row-utils.ts:24
S3-009 32273dc2 src/features/manual-sale/lib/manual-sale-row-utils.ts 40 selling_price manual-sale-row-utils.ts:24,40
S3-010 249df0ca src-tauri/src/biz/product_service.rs 153 トランザクション product_service.rs:153
S3-011 ff05436f src/features/csv-import/lib/formatErrorRow.ts 22 destructive formatErrorRow.ts:22
S3-012 07302b5e src/features/products/components/PriceRevisionTable.tsx 104 入力中 PriceRevisionTable.tsx:104
S3-013 ff05436f src/components/patterns/EmptyState.tsx 32 text-stone-500 EmptyState.tsx:32
S3-014 51b1f484 docs/design-system/02-component-catalog.md 841 個別確認 `:841
S3-014 51b1f484 docs/Plans.md 70 撤回 `:70
S3-015 ff655efd docs/design-system/reference/mockup-d-lists.html 112 max-height mockup-d-lists.html:112
S3-016 294ba366 scripts/check-workflow-git.sh 155 wait `:155
S3-017 909fe096 src/features/plu-export/PluExportPage.tsx 188 complexity PluExportPage.tsx:188
S3-018 - src/features/return-exchange/ReturnExchangePage.tsx 185 max-lines-per-function ReturnExchangePage.tsx:185
S3-019 a38597e8 src-tauri/src/biz/stocktake_service.rs 4 大晦日 stocktake_service.rs:4
S3-019 a3f08cdd docs/function-design/35-biz-stocktake-service.md 374 大晦日 35-biz-stocktake-service.md:374
S3-019 a3f08cdd docs/architecture/biz-task-specs.md 451 大晦日 biz-task-specs.md:451
S3-019 a3f08cdd docs/db-design/tracking-system-tables.md 137 大晦日 tracking-system-tables.md:137
S3-019 a3f08cdd docs/screen_mockups.html 208 大晦日 screen_mockups.html:208
S3-019 a3f08cdd docs/screen_mockups.html 259 閉店後 screen_mockups.html:259
S3-019 e40e50b9 docs/SCREEN_DESIGN.md 213 数週間 SCREEN_DESIGN.md:213
S3-020 0a83374b docs/function-design/40-cmd-product.md 279 ページング引数 40-cmd-product.md:279
S3-021 949a5c4c docs/function-design/61-ui-receiving.md 30 result 61-ui-receiving.md:30
S3-021 949a5c4c docs/function-design/61-ui-receiving.md 73 result 61-ui-receiving.md:30/:73
S3-021 949a5c4c docs/function-design/63-ui-return-exchange.md 36 result 63-ui-return-exchange.md:36
S3-021 949a5c4c docs/function-design/63-ui-return-exchange.md 71 result 63-ui-return-exchange.md:36/:71
S3-022 7e0ccf1b src/features/monthly-sales/MonthlySalesPage.tsx 177-180 部門 MonthlySalesPage.tsx:177
S3-023 31c069d5 src/features/receiving/ReceivingPage.tsx 543-549 原価 ReceivingPage.tsx:543
S3-024 31c069d5 src/features/return-exchange/ReturnExchangePage.tsx 790-791 errors.items ReturnExchangePage.tsx:790
S3-025 31c069d5 src/features/return-exchange/ReturnExchangePage.tsx 624 レシート ReturnExchangePage.tsx:624
S3-026 7e0ccf1b src/features/inventory-records/InventoryRecordsPage.tsx 156-245 select InventoryRecordsPage.tsx:156
S3-026 7e0ccf1b src/features/receiving/ReceivingPage.tsx 393 select ReceivingPage.tsx:393
S3-026 7e0ccf1b src/features/disposal/DisposalPage.tsx 509 select DisposalPage.tsx:509
S3-026 7e0ccf1b src/features/operation-logs/OperationLogsPage.tsx 338-352 date OperationLogsPage.tsx:338
S3-027 7e0ccf1b src/components/patterns/Pagination.tsx 52-53 text-sm Pagination.tsx:52
S3-027 7e0ccf1b src/components/patterns/Pagination.tsx 99 font-semibold Pagination.tsx:52-53,99
S3-028 7ae4ade6 src/features/products/import/ProductImportPreview.tsx 76 上書き ProductImportPreview.tsx:76
S3-028 7ae4ade6 src/features/monthly-sales/components/ProductRankingTable.tsx 80 ranking ProductRankingTable.tsx:80
S3-028 7ae4ade6 src/features/backup-restore/BackupRestorePage.tsx 533 最新 BackupRestorePage.tsx:533
S3-029 7ae4ade6 src/features/stock-inquiry/components/StockStatusBadge.tsx 42 通常 StockStatusBadge.tsx:42
S3-029 7ae4ade6 docs/design-system/01-decision-rules.md 445 3:1 01-decision-rules.md:445
S3-029 7ae4ade6 docs/design-system/01-decision-rules.md 451 3:1 01-decision-rules.md:445,451
S3-029 7ae4ade6 docs/design-system/04-backbone.md 44 3:1 04-backbone.md:44
S3-029 7ae4ade6 docs/design-system/02-component-catalog.md 158 3:1 02-component-catalog.md:158
S3-029 7ae4ade6 docs/design-system/02-component-catalog.md 588 aria-label catalog:588
S3-029 7ae4ade6 src/features/daily-report-import/DailyReportImportPage.tsx 322 成功 DailyReportImportPage.tsx:322
S3-029 7ae4ade6 src/features/daily-sales/components/ProductTable.tsx 133 Badge ProductTable.tsx:133
S3-030 7e0ccf1b src/components/ui/button.tsx 15-16 outline button.tsx:15
S3-031 31c069d5 src-tauri/src/biz/inventory_service/list.rs 21 MAX_PER_PAGE list.rs:21
S3-031 - docs/archive/plans/2026-09-03-ui-list-backbone-d-lane2.md 642-648 run ):642
```

#### 残す区分（file ごと）

| file | 引用 | 処置 |
|---|---|---|
| `docs/design-system/reference/2026-08-20-proposal-A-rules-bound.md` | 53 | 残す（dated の提案） |
| `docs/design-system/reference/2026-08-20-proposal-B-blank-slate.md` | 37 | 残す（dated の提案） |
| `docs/design-system/reference/mockup-e-badge-cta-samples.html` | 14 | 残す（参考の mockup） |
| `docs/design-system/reference/mockup-d-import-export.html` | 9 | 残す（参考の mockup） |
| `docs/design-system/reference/mockup-d-history.html` | 6 | 残す（参考の mockup） |
| `docs/design-system/reference/mockup-d-home-sales-admin.html` | 4 | 残す（参考の mockup） |
| `docs/design-system/reference/mockup-d-stocktake.html` | 4 | 残す（参考の mockup） |
| `docs/design-system/reference/mockup-d-lists.html` | 3 | 残す（参考の mockup） |
| `docs/design-system/reference/mockup-d-forms-b.html` | 2 | 残す（参考の mockup） |
| `docs/design-system/reference/2026-08-23-current-design-analysis.md` | 2 | 残す（dated の分析） |
| `docs/design-system/reference/2026-09-04-owner-l3-feedback-raw.md` | 1 | 残す（dated の L3 の記録） |
| `docs/design-system/reference/README.md` | 1 | 残す（参考資料の索引。頭に「実装の正本ではない」と書く。引用は mockup の説明の中） |
| `docs/decision-log.md` | 4 | 残す（append-only の本文） |
| `src/lib/invalidation-contract.static.test.ts` | 11 | 対象外（test の data の文字列） |

### 他の lane との重なり

- Draft の PR #172（`agent/sd-direct-read-runtime`）は `docs/backlog.md` と `docs/function-design/` の 22・29・37・45・55 を、Draft の PR #155（`agent/backup-offsite`）は `docs/backlog.md` と 22・71 を触る（`gh pr view {番号} --json files`、起票の時点）。後に merge する側が `origin/main` の単段 merge で同期する（`docs/DEV_WORKFLOW.md` Wave Operation）。本 lane が後なら、同期の後に下の baseline の command を測り直し、増えた引用の行を分類表に足して同じ方法で始末する。
- lane 2b（検査）: 本 lane の merge の後。分類表の「lane 2b へ送る」2 行と、残す行の形（`K` の 3 つの形）を 2b の検査の入力にする。

## Non-scope

- 検査の追加（lane 2b）と、`scripts/doc-consistency-check.sh` を含む workflow の file の変更（`docs/ci.md`・`docs/agent-guidance/merge-evidence.md` の引用は lane 2b へ送る）。
- 規則の文（D-125・DOC_STYLE_GUIDE §1）の変更。
- 引用以外の文の直し（直す文の中に引用がある場合の最小の言い換えはよい。S4）。
- 時点の記録（`docs/design-system/reference/**`・`docs/decision-log.md` の本文・dated の Plan Packet と Matrix・`docs/archive/**`・`docs/evidence/**`・`docs/research/**`）と data の行番号。
- reference の file の頭に時点の注記を足すこと、外部 library の行の今の版での照合。
- code の振舞い・test の assertion の変更（comment の行だけを直す）。

## Acceptance Criteria

AC の command は repo root で、次の 2 つの変数を同じ shell に定義してから実行する（bash・zsh）。`P` は file:行 の引用の 4 つの形（拡張子つきの `path:N`、bare の `:N`〈backtick・括弧・`/`・`〜`・`・`・空白の後。前が英数字・`_`・`:`・`.`・`{`・`}`・`[`・引用符なら時刻・比・CSS・format の指定なので除く〉、path か backtick か `SCREEN_DESIGN`・`mockup` の後の `L<N>`〈2 桁以上。Level の `L3` と Matrix の行の label `Matrix L10` を除く〉、`#L<N>`）、`K` は残す行の 3 つの形（library の名前と版、backlog の時点の句、変更履歴の表の行）。

```sh
P='\.[A-Za-z][A-Za-z0-9]*`?:[0-9]+|(^|[^0-9A-Za-z_:.{}\["\x27])\s?:[0-9]+|(`|\.[A-Za-z][A-Za-z0-9]*|SCREEN_DESIGN[^ ]*|mockup) ?L[0-9]{2,}|#L[0-9]+'
K='(TanStack Router|@tanstack/[a-z-]+|@radix-ui/[a-z-]+|happy-dom|tauri-plugin-dialog) [0-9]+\.[0-9]+\.[0-9]+|`[0-9a-f]{8}` の時点で|^[^:]+:[0-9]+:\| 20[0-9]{2}-[0-9]{2}-[0-9]{2} \|'
```

- AC1: `bash scripts/doc-consistency-check.sh` が exit 0。
- AC2: `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-10-20-docs-file-line-rewrite.md` が exit 0。
- AC3: `cd src-tauri && cargo run --bin generate_traceability -- --check` が exit 0（function-design を触る。REQ の参照は増減させない）。
- AC-S1-1（正本に行番号が残らない）: `rg -n -e "$P" -g '!docs/archive/**' -g '!docs/evidence/**' -g '!docs/research/**' -g '!docs/plans/**' -g '!docs/backlog.md' -g '!docs/decision-log.md' -g '!docs/design-system/reference/**' -g '!docs/ci.md' -g '!docs/agent-guidance/**' docs AGENTS.md CLAUDE.md | rg -v -e "$K"` が何も出さない（後ろの `rg` の exit 1）。
- AC-S1-2（残す行は表の残す行だけ）: AC-S1-1 の前半を `rg -e "$K" | cut -d: -f1,2` に通した出力が 11 行で、分類表の S1-001・S1-015・S1-017・S1-018・S1-019・S1-020・S1-032・S1-041・S1-078・S1-079・S1-080（`DOC_STYLE_GUIDE.md` の例、`01-decision-rules.md` の 389・419・420・421・423・362、`02-component-catalog.md`・`review-checklist.md`・`53-ui-home.md` の変更履歴、`22-mnt-migration.md` の 234）の行である。
- AC-S1-3（lane 2b へ送る行は変えない）: `rg -n -e "$P" docs/ci.md docs/agent-guidance | cut -d: -f1,2` が `docs/ci.md:29` と `docs/agent-guidance/merge-evidence.md:160` の 2 行（起点と同じ）。
- AC-S1-4（`K` の時点の句が正本に漏れない）: `rg -n '`[0-9a-f]{8}` の時点で' -g '!docs/archive/**' -g '!docs/evidence/**' -g '!docs/research/**' -g '!docs/plans/**' -g '!docs/backlog.md' -g '!docs/decision-log.md' -g '!docs/design-system/reference/**' -g '!docs/ci.md' -g '!docs/agent-guidance/**' docs AGENTS.md CLAUDE.md | cut -d: -f1,2` が `docs/DOC_STYLE_GUIDE.md:87` の 1 行だけ（起点と同じ。正本の行に時点の句を書いて AC-S1-1 を抜けることを防ぐ）。
- AC-S2-1（comment に行番号が残らない）: `rg -n -e "$P" -g '!src/lib/invalidation-contract.static.test.ts' src src-tauri/src | rg -v -e "$K"` が何も出さない（後ろの `rg` の exit 1）。
- AC-S2-2: AC-S2-1 の前半を `rg -e "$K" | cut -d: -f1,2` に通した出力が `src/features/products/ProductListPage.scroll-restoration.test.tsx` の 3 行（S2-009〜S2-011）だけ。
- AC-S2-3（bindings）: `cd src-tauri && cargo run --bin generate_bindings` の後、`git status --porcelain -- src/lib/bindings.ts` が何も出さない（S2 で bindings に出る doc comment を直したなら、再生成の差分が commit に入っていて、同じく何も出さない）。
- AC-S2-4: `npm run lint`・`npm run format:check`・`cd src-tauri && cargo fmt --check` がそれぞれ exit 0（comment の行だけを変えたことの確かめ）。
- AC-S3-1（backlog の行番号は全部時点つき）: `rg -n -e "$P" docs/backlog.md | rg -v -e '`[0-9a-f]{8}` の時点で'` が何も出さない（後ろの `rg` の exit 1）。
- AC-S3-2（添えた SHA が main に在る）: `for s in $(rg -o -N '`([0-9a-f]{8})` の時点で' -r '$1' docs/backlog.md | sort -u); do git merge-base --is-ancestor "$s" origin/main || echo "NG $s"; done` が何も出さない。
- AC-S3-3（行ごとの句の版）: 下の loop が何も出さない。分類表の S3 の行ごとに、場所の行の `docs/backlog.md` の句の SHA の集合が、照合の入力のその ID の採った版〈`-` を除く〉の集合と等しいことを照らす（AC-S3-1 は行の単位なので、1 行に句が 1 つあれば同じ行の別の版の引用が漏れても通る）。

```sh
F=docs/plans/2026-10-20-docs-file-line-rewrite.md
sed -n '/^#### S3 の行$/,/^#### 照合の入力$/p' "$F" | rg '^\| S3-[0-9]{3} \|' | while IFS='|' read -r z id loc z2; do
  id=${id// /}; l=$(printf '%s' "$loc" | rg -o '[0-9]+')
  t=$(sed -n "${l}p" docs/backlog.md | rg -o '`[0-9a-f]{8}` の時点で' | rg -o '[0-9a-f]{8}' | sort -u | tr '\n' ' ')
  i=$(sed -n '/^```text 照合の入力$/,/^```$/p' "$F" | rg "^$id " | awk '$2 != "-" {print $2}' | sort -u | tr '\n' ' ')
  [ "$t" = "$i" ] || echo "NG $id: backlog の $l 行の句の SHA [$t] / 照合の入力の版 [$i]"
done
```

- AC-S4-1（読んで確かめる）: 処置が「消えた」「曖昧」の行ごとに、理由と処置（引用を外した・言い換えた語と意図の照合の 1 句・backlog に足した句）が書いてある。backlog に足した句は `rg -n '次の docs の lane の候補' docs/backlog.md` の項目の中に在る。
- AC-S5-1: `rg -n '^\| S[1-3]-[0-9]{3} \|.*\| 未処置' docs/plans/2026-10-20-docs-file-line-rewrite.md` が何も出さない（exit 1。行末の空白や「未処置（…）」の形も拾う）。
- AC-S5-2（方法の抜き取り）: Implementation Results に抜き取りの表があり、行は分類表の ID の番号が S1 は 5 の倍数（S1-005〜S1-085 の 17 行）、S2 は 3 の倍数と S2-001（8 行）、S3 は 3 の倍数（S3-003〜S3-030 の 10 行）の計 35 行。処置が残す・lane 2b へ送るの行か、型が外部 library・repo 外の資料を含む行に当たったら、同じ段の抜き取りに入っていない次の ID の行で代える。各行に、候補を引く command（方法の 1）と候補の一覧、採った版〈hit か親〉の `git show {rev}:{path} | sed -n '{範囲}p'` の command、その版の行が属する symbol・節、書き換え先、今の base の `rg -n` の hit がある。reviewer は表から 5 行以上を選び、command を実行して「採った版の行の symbol・節 = 書き換え先」を確かめる。
- AC-S5-3（触る file の範囲）: `git diff --name-only origin/main...HEAD | rg -v -e '^docs/function-design/' -e '^docs/design-system/0[0-2]-' -e '^docs/msi-distribution-runbook\.md$' -e '^docs/backlog\.md$' -e '^docs/Plans\.md$' -e '^docs/plans/2026-10-20-docs-file-line-rewrite\.md$' -e '^src/' -e '^src-tauri/src/'` が何も出さない（後ろの `rg` の exit 1。AC-S2-3 で bindings を再生成したら `src/lib/bindings.ts` もこの範囲に入る）。`bash scripts/ci/classify-changes.sh | rg -x 'workflow=false'` が 1 行を出す。
- AC-S5-4（code は comment だけ）: `git diff origin/main...HEAD -- src src-tauri/src | rg '^[-+]' | rg -v '^(\+\+\+|---)' | rg -v '^[-+]\s*(//|/\*|\*|\{/\*)'` が何も出さない（後ろの `rg` の exit 1。bindings を再生成したときは `src/lib/bindings.ts` の行が出るので、その行が doc comment の JSDoc だけであることを読んで確かめる）。
- AC-S5-5（全引用の照合）: 下の loop を repo root で実行した出力に `NG` で始まる行が無い（`| rg '^NG'` が何も出さない、exit 1）。分類表の行は `#### S1 の行`〜`#### 照合の入力` の間だけから読む。`NG` は次を出す: 照合の入力の行ごとに、X が無い〈列が足りない〉、分類表に同じ ID の行が無い、X が分類表の場所の行〈`origin/main` のその file・行〉に無い、target が場所の行で X の直前に続く path〈X の中の拡張子つきの名前を、場所の行で前に続く path の文字まで伸ばしたもの〉で終わらない、候補が無い、採った版が候補〈hit と親〉に無い、その行の採った版の欄〈第 6 列〉に採った版が無い〈採った版が `-` なら欄に `-` が無い〉、その版に target の file が無い。分類表の行ごとに、処置の欄の先頭が「残す」「lane 2b」でなく型が正本・comment・backlog で、引用の列の行番号の多重集合〈`-` の範囲は両端〉と照合の入力のその ID の N の数字の多重集合が違う。`--` で始まる行（外部 library・repo 外の資料を含む行）は数を読んで確かめる。各候補の行は N−3〜M+3 を出す。`CHECK <ID>` は `NG` でなく、人が読む塊を絞る印: `*` の版の N〜M に W が無い、`*` より古い候補〈採った版が `-` なら全候補〉の N〜M に W が在る、W が `-`、のどれかで出る。Writer は出力の全文を PR の comment に残し（大きければ段ごとに分ける）、Implementation Results にはその link だけを書く。Final Review の reviewer は `CHECK` の付いた塊を全部、「`*` の版で行 N を含む最も内側の要素・文が名指す語を含み、それより古い候補ではそうでない（同じ symbol なら最古が `*`）」で照らし、AC-S5-2 の 35 行で symbol の判断を確かめる。場所の行は `origin/main` の行なので、同期で `origin/main` が動いたら場所の列を今の行に直してから回す。

```sh
F=docs/plans/2026-10-20-docs-file-line-rewrite.md
rows(){ sed -n '/^#### S1 の行$/,/^#### 照合の入力$/p' "$F" | rg '^\| S[1-3]-[0-9]{3} \|'; }
inp(){ sed -n '/^```text 照合の入力$/,/^```$/p' "$F" | rg -v '^(```|#)'; }
inp | while read -r id rev target n w x; do
  [ -n "$x" ] || { echo "NG $id: X が無い（列が足りない）"; continue; }
  row=$(rows | rg "^\| $id \|")
  [ -n "$row" ] || { echo "NG $id: 分類表に無い"; continue; }
  loc=$(printf '%s' "$row" | awk -F'|' '{print $3}' | rg -o '[^` ]+:[0-9]+')
  line=$(git show "origin/main:${loc%:*}" 2>/dev/null | sed -n "${loc##*:}p")
  printf '%s' "$line" | grep -qF -- "$x" || echo "NG $id: X が場所 $loc の行に無い"
  re="$(printf '%s' "$x" | sed 's/[][\\.*^$+?(){}|]/\\&/g')([^0-9]|\$)"
  a=${n%-*}; b=${n#*-}
  xp=$(printf '%s' "$x" | rg -o '[A-Za-z0-9_./-]*\.[A-Za-z0-9]+' | head -n 1)
  wp=$(printf '%s' "${line%%"$x"*}" | sed 's/.*[^A-Za-z0-9_./-]//')$xp
  [ -z "$xp" ] || case "$target" in "$wp"|*/"$wp") ;; *) echo "NG $id: target $target が場所の行の path $wp で終わらない";; esac
  hits=$(git log --format=%H --pickaxe-regex -S"$re" origin/main)
  cands=$(printf '%s\n' "$hits" | rg . | while read -r h; do git rev-parse "$h" "$h^@"; done | sort -u)
  full=$(git rev-parse -q --verify "$rev^{commit}")
  echo "== $id X=$x W=$w target=$target:$n rev=$rev"
  [ -n "$hits" ] || echo "NG $id: 候補が無い"
  printf '%s\n' "$cands" | rg . | xargs -r git show -s --format='%ct %H %ad' --date=short | sort -n | while read -r t c d; do
    printf '%s\n' "$hits" | rg -qx "$c" && k=hit || k=親
    [ "$c" = "$full" ] && m='*' || m=' '
    echo "  $m $(printf %.8s "$c") $k $d"
    git show "$c:$target" 2>/dev/null | sed -n "$((a>3?a-3:1)),$((b+3))p" | sed "s/^/      | /"
  done
  ck=$(printf '%s\n' "$cands" | rg . | xargs -r git show -s --format='%ct %H' | sort -n | while read -r t c; do
    git show "$c:$target" 2>/dev/null | sed -n "${a},${b}p" | grep -qF -- "$w" && h=1 || h=0
    if [ "$c" = "$full" ]; then [ "$h" = 1 ] || echo x; break; fi
    [ "$h" = 1 ] && echo x
  done)
  { [ "$w" = - ] || [ -n "$ck" ]; } && echo "CHECK $id"
  if [ "$rev" = - ]; then
    printf '%s' "$row" | awk -F'|' '{print $6}' | rg -q -- '-' || echo "NG $id: 分類表の採った版の欄に - が無い"
    continue
  fi
  printf '%s\n' "$cands" | rg -qx "$full" || echo "NG $id: $rev が候補に無い"
  printf '%s' "$row" | awk -F'|' '{print $6}' | rg -q "\`$rev\`" || echo "NG $id: 分類表の採った版の欄に $rev が無い"
  git cat-file -e "$rev:$target" 2>/dev/null || echo "NG $id: $rev に $target が無い"
done
rows | while IFS='|' read -r z id loc cites type rv dest act z2; do
  id=${id// /}; type=${type# }; type=${type% }; act=${act# }
  case "$act" in 残す*|'lane 2b'*) continue;; esac
  nt=$(printf '%s' "$cites" | rg -o '[:L][0-9]+(-[0-9]+)?' | rg -o '[0-9]+' | sort -n | tr '\n' ' ')
  ni=$(inp | rg "^$id " | awk '{print $4}' | rg -o '[0-9]+' | sort -n | tr '\n' ' ')
  case "$type" in
    正本|comment|backlog) [ "$nt" = "$ni" ] || echo "NG $id: 引用の行番号 [$nt] / 照合の入力 [$ni]";;
    *) echo "-- $id（$type）: 引用 [$nt] / 照合の入力 [$ni]（読んで確かめる）";;
  esac
done
```

baseline（起点 `6a25f63f`、2026-10-21 に測り直した。base の tree に本 packet と `docs/Plans.md` の 1 句〈引用を含まない〉を足した branch の working tree で、上の変数を定義して各 command を逐語で実行した。`rg` の数は `docs/plans/**` を除くので base と同じ。`wc -l` は数を見るため、AC-S1-2 の `sort` は順を揃えるために付けた。`# exit` は pipe の最後の command の終了値。AC1〜AC3・AC-S2-3・AC-S2-4 は PR の Validation の欄。AC-S5-5 の loop は上の形〈X の場所の行の確認と `CHECK` を含む〉で測った。AC-S5-1 の 132 は S1〜S3 の表の 139 行から起票の時点で処置を決めた 7 行を除いた未処置の行。bash と zsh で出力は同じ〈`rg` の出力の順を除く〉）:

```text
$ git rev-parse --short origin/main
6a25f63f
# exit=0

$ rg -o -e "$P" -g '!docs/archive/**' -g '!docs/evidence/**' -g '!docs/research/**' -g '!docs/plans/**' docs AGENTS.md CLAUDE.md | wc -l
466
# exit=0

$ rg -o -e "$P" -g '!docs/archive/**' -g '!docs/evidence/**' -g '!docs/research/**' -g '!docs/plans/**' -g '!docs/backlog.md' -g '!docs/decision-log.md' -g '!docs/design-system/reference/**' docs AGENTS.md CLAUDE.md | wc -l
241
# exit=0

$ rg -n -e "$P" -g '!docs/archive/**' -g '!docs/evidence/**' -g '!docs/research/**' -g '!docs/plans/**' -g '!docs/backlog.md' -g '!docs/decision-log.md' -g '!docs/design-system/reference/**' docs AGENTS.md CLAUDE.md | wc -l
86
# exit=0

$ rg -o -e "$P" docs/backlog.md | wc -l
85
# exit=0

$ rg -n -e "$P" docs/backlog.md | wc -l
31
# exit=0

$ rg -o -e "$P" docs/decision-log.md | wc -l
4
# exit=0

$ rg -o -e "$P" docs/design-system/reference | wc -l
136
# exit=0

$ rg -o -e "$P" -g '!src/lib/invalidation-contract.static.test.ts' src src-tauri/src | wc -l
22
# exit=0

$ rg -o -e "$P" src/lib/invalidation-contract.static.test.ts | wc -l
11
# exit=0

$ rg -c -e "$P" .agents .claude/rules .claude/commands .claude/agents README.md .github scripts
(出力なし)
# exit=1

## AC-S1-1
$ rg -n -e "$P" -g '!docs/archive/**' -g '!docs/evidence/**' -g '!docs/research/**' -g '!docs/plans/**' -g '!docs/backlog.md' -g '!docs/decision-log.md' -g '!docs/design-system/reference/**' -g '!docs/ci.md' -g '!docs/agent-guidance/**' docs AGENTS.md CLAUDE.md | rg -v -e "$K" | wc -l
79
# exit=0

## AC-S1-2
$ rg -n -e "$P" -g '!docs/archive/**' -g '!docs/evidence/**' -g '!docs/research/**' -g '!docs/plans/**' -g '!docs/backlog.md' -g '!docs/decision-log.md' -g '!docs/design-system/reference/**' -g '!docs/ci.md' -g '!docs/agent-guidance/**' docs AGENTS.md CLAUDE.md | rg -e "$K" | cut -d: -f1,2 | sort
docs/DOC_STYLE_GUIDE.md:87
docs/design-system/01-decision-rules.md:389
docs/design-system/02-component-catalog.md:1063
docs/function-design/53-ui-home.md:312
docs/quality/review-checklist.md:123
# exit=0

## AC-S1-3
$ rg -n -e "$P" docs/ci.md docs/agent-guidance | cut -d: -f1,2
docs/ci.md:29
docs/agent-guidance/merge-evidence.md:160
# exit=0

## AC-S1-4
$ rg -n '`[0-9a-f]{8}` の時点で' -g '!docs/archive/**' -g '!docs/evidence/**' -g '!docs/research/**' -g '!docs/plans/**' -g '!docs/backlog.md' -g '!docs/decision-log.md' -g '!docs/design-system/reference/**' -g '!docs/ci.md' -g '!docs/agent-guidance/**' docs AGENTS.md CLAUDE.md | cut -d: -f1,2
docs/DOC_STYLE_GUIDE.md:87
# exit=0

## AC-S2-1
$ rg -n -e "$P" -g '!src/lib/invalidation-contract.static.test.ts' src src-tauri/src | rg -v -e "$K" | wc -l
22
# exit=0

## AC-S2-2
$ rg -n -e "$P" -g '!src/lib/invalidation-contract.static.test.ts' src src-tauri/src | rg -e "$K" | cut -d: -f1,2
(出力なし)
# exit=0

## AC-S3-1
$ rg -n -e "$P" docs/backlog.md | rg -v -e '`[0-9a-f]{8}` の時点で' | wc -l
31
# exit=0

## AC-S3-2
$ for s in $(rg -o -N '`([0-9a-f]{8})` の時点で' -r '$1' docs/backlog.md | sort -u); do git merge-base --is-ancestor "$s" origin/main || echo "NG $s"; done
(出力なし)
# exit=0

## AC-S3-3（照合の入力が空で、backlog に句が無い起票の時点）
$ { AC-S3-3 の loop; }
(出力なし)
# exit=0

## AC-S5-1
$ rg -n '^\| S[1-3]-[0-9]{3} \|.*\| 未処置' docs/plans/2026-10-20-docs-file-line-rewrite.md | wc -l
132
# exit=0

## AC-S5-5（照合の入力が空の本 packet）
$ { AC-S5-5 の loop; } | rg -c '^NG'
122
# exit=0

$ { AC-S5-5 の loop; } | rg -c '^--'
10
# exit=0

$ { AC-S5-5 の loop; } | rg -c '^CHECK'
(出力なし)
# exit=1
```

完了の時点の期待: AC-S1-1・AC-S2-1・AC-S3-1 は 0 行（後ろの `rg` の exit 1）、AC-S1-2 は 11 行、AC-S1-3 は起点と同じ 2 行、AC-S1-4 は起点と同じ 1 行、AC-S2-2 は 3 行、AC-S3-2・AC-S3-3 は出力なし、AC-S5-1 は 0 行、AC-S5-5 は `NG` の行なし（起票の時点の 122 は、型が正本・comment・backlog で未処置の行〈S1 72・S2 19・S3 31〉の引用と照合の入力の食い違い。`--` の 10 は S1-014・S1-017〜S1-020・S1-041・S1-079・S2-009〜S2-011。完了の時点の `--` は S1-014 だけで、library の行は処置の先頭が「残す」なので数の検査から外れる）。`CHECK` は起票の時点で 0（照合の入力が空）で、完了の時点の数は Final Review の reviewer が読む塊の数として PR に書く。

## Design Readiness

- 引用する設計正本（節まで）: [DOC_STYLE_GUIDE](../../DOC_STYLE_GUIDE.md) §1「code と文書の箇所の引き方（行番号で引かない）」、[decision-log](../../decision-log.md) の D-125（Decision の (1)〜(3)、Guarantee range）、[DEV_WORKFLOW](../../DEV_WORKFLOW.md) Workflow State の `Final Review Minimum`（workflow gate の変更は 2）と Wave Operation（base の同期）、`scripts/ci/classify-changes.sh` の workflow の path の一覧。
- 必要な設計成果物: Design artifact selection の当たる行は無い（振舞い・command・DB・画面・format を変えない）。状態は existing sufficient。
- plan にしかない durable な判断の昇格先: 本 packet が決める判断は 3 つ。(1) 書かれた時点の意図の取り戻し方（`git log -S` の全 hit と親を候補に古い順に歩き、行 N を含む最も内側の要素・文が引用の文の名指す語を含む最古の版を採る）、(2) 外部 library の引用は版を同じ行に書いて行番号を残す、(3) workflow の 2 件を lane 2b へ送る。(1) は書き換えの手順で、規則でない（D-125 の文は変えない）。(2) は DOC_STYLE_GUIDE §1 の「library の版を書いた観測はそのまま」の当てはめ。(3) は lane の範囲の分け方。どれも lane 2b が検査を足すときに、2b の packet が検査の形として引き継ぐ（D-125 の Revisit「lane 2 が検査を足すとき」）。
- 前提・制約と延期した gap: workflow の 2 件は lane 2b へ延期する。延期しても安全な理由: 2 件は今も指す先が合っている（起点の調査の抜き取りで `docs/ci.md:29` と `merge-evidence.md:160` は「合う」）うえ、2b は検査を入れる前に直す。
- 絶対保証の自己点検: 「正本と comment に行番号が残らない」は AC-S1-1・AC-S2-1 の `P` の 4 つの形（D-125 の 3 つの形と、`#` の無い `L<N>`）についての主張。行番号を文で書く形（「65 行」）と、`P` の外の形は対象外。起票の base で試して引用が 0 だった形: 全角のコロン・`:N:M`・`file(N)`・`line N`・link の中の `#L`・path に隣る `N 行目`。
- 判定: ready。

## Registration / Generation Obligations

| 変更対象 | 義務 | 本 lane |
|---|---|---|
| Tauri command・specta の型の doc comment | `cargo run --bin generate_bindings` で `src/lib/bindings.ts` を再生成 | 起票の時点で当たる comment は無い（S2-001 は `#[cfg(test)]` の module の `pub(crate) fn`）。AC-S2-3 で確かめ、当たれば S2 の同じ commit で再生成する |
| REQ / coverage の追加・変更・削除 | `cargo run --bin generate_traceability` で 90 を再生成 | REQ の参照は増減させない。AC3 の `--check` で確かめ、増減が出たら再生成して Scope に足す |
| function-design doc の節 | `design_compliance_test` の必須節 | 節の見出しを足さず消さない（引用の文だけ） |

ほかの行（route、operator 画面、source / workflow doc の新設）は該当なし。

## Impact Review Lenses

not applicable: 起点は現地調査・実機確認・外部 tool の振舞いでなく、repo の中の docs と code の引用の照合（read-only の調査 1 本と git の履歴）。外部 library の行は版を書いた観測として残し、今の版で確かめ直さない（非目的）。

## Boundary / Wire Contract

変えない。JSON・CSV・config・manifest・DTO・DB に触れない。bindings は doc comment が当たるときだけ再生成し、型は変わらない（AC-S2-3）。

## Test Plan

Test Design Matrix は置かない（R2 で任意。各段の AC が `rg` の件数と分類表の行で、書き換えた所と残した所を確かめる）。

- targeted tests: AC3（`generate_traceability -- --check`）、AC-S2-3（bindings の再生成の差分なし）、AC-S2-4（lint・format・`cargo fmt`）。comment だけなので test は足さない（hosted CI の rust・frontend の job が既存の test を回す）。
- negative tests: AC-S1-1・AC-S2-1・AC-S3-1（行番号の引用が残らない）、AC-S1-4・AC-S3-3（`K` の時点の句で行ごと抜けない）、AC-S5-4（code は comment の行だけ）、AC-S5-3（時点の記録と workflow の file を触らない）。
- compatibility checks: AC1・AC2（Markdown の link の先の file の実在）。
- data safety checks: AC-S5-3 の範囲と、書き換えた文に店の運用の細部・実値・発言の原文・金額・商品名を足していないこと（引用を symbol・節に替えるだけ）。
- main wiring/integration checks: AC-S5-5（全引用の候補と採った版の行を loop で照らす）と AC-S5-2（symbol の判断の抜き取りを reviewer が `git show` で再現できる）。

起票の run の検証（AC1・AC2 と `bash scripts/check-workflow-git.sh`）の結果は PR の Validation の欄に書く。

## Review Focus

- 方法: 候補（path で絞らない `--pickaxe-regex -S` の全 hit と親）を古い順に歩き、行 N を含む最も内側の要素・文が引用の文の名指す語を含む最古の版を採る照合で、書かれた時点の意図を取り戻せるか。照合で合わない引用を「消えた・曖昧」に落とす条件が緩すぎないか（今の行を見て付け替える逃げ道が無いか）。
- 意味の照合: Final Review の reviewer は AC-S5-5 の出力の `CHECK` の付いた塊と AC-S5-2 の抜き取りを読む。`CHECK` の無い塊を読まないことが、W の決め方（S5）で許されるか。
- AC: AC-S1-1・AC-S2-1・AC-S3-1 の `K` の除外が、残す行の形だけを外し、書き換え漏れを隠さないか（AC-S1-2・AC-S2-2 の行の数と ID、AC-S1-4・AC-S3-3 で照らす）。`P` の 4 つの形が拾い過ぎ（時刻・比・CSS・format・Level の `L3`・Matrix の行の label）も拾い漏らしもしないか。
- 分類表: 起票の時点で処置を決めた 7 行と、型を決めた行（外部 library・repo 外の資料）の当否。
- Q1（Plan Review で答える）: workflow の 2 件（`docs/ci.md:29`・`docs/agent-guidance/merge-evidence.md:160`）を lane 2b へ送るか、本 lane で直して Final Review Minimum を 2 にするか。既定: 2b へ送る（発注の Final Review Minimum 1 と、workflow の file を触ると 2 を求める規則〈DEV_WORKFLOW Workflow State・helper〉を両立する。2b は元から Double Audit）。
- Q2（Plan Review で答える）: 外部 library の引用（正本 7 行・comment 3 行）は、版を同じ行に書いて行番号を残すか、library の symbol（関数名）で引き直すか。既定: 版を書いて残す（DOC_STYLE_GUIDE §1 の版を書いた観測の形。library の中の関数名は版で変わり、行番号と同じく版に依る）。
- Q3（D-127、Plan Review で答える）: この変更で不要になる記述・条件は無いか。起草の答え: 規則の文・検査は本 lane で変えない。起点の調査の提案のうち「reference の file の頭に時点の注記を足す」は DOC_STYLE_GUIDE §1 の「そのまま残す」と重なるので採らない（非目的）。backlog の「次の docs の lane の候補」の項目に file:行 の引用を扱う句は無い（`rg -n '行番号|file:行' docs/backlog.md` が何も出さない）。`docs/Plans.md` の `## 次の行動` の lane 2 の文は、本 lane の merge の後の closeout で 2b だけの文に替える。

## Implementation Results

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

- 結果（PR #183、2026-10-21 merge）: 設計の正本と code の comment の file:行 の引用を path と symbol・節で、backlog の引用を「`{sha8}` の時点で `{path}:{N}`」の句で引き直した。版の決め方は方法の 1〜2（path で絞らない候補と親を古い順に歩く）で、各行の採った版と書き換え先は分類表に、repo の中の引用ごとの照合は照合の入力と下の PR の comment にある。
- 段ごと: S1 は書き換えのほかに、起票で決めた残す行、版を足した外部 library の観測（残す）、lane 2b へ送る workflow の 2 file、取り戻せない行（S1-070・S1-072・S1-075・S1-082・S1-084・S1-085）に分かれた。S2 は書き換えのほかに、版を足した library の行（残す）と取り戻せない行（S2-004・S2-005・S2-007）で、直した comment に bindings に出る doc comment は無い。S3 は 1 行（S3-018 の曖昧）を除いて版ごとの句で書き換え、S3-031 の `):642-648` は取り戻せない引用として行の中で外した（Final Review の是正で、S3-014 の `:70` は書かれた時点の `docs/Plans.md` の行、S3-024 は `{errors.items}` の段落として取り戻した）。
- S4: 消えた・曖昧は、引用を外すか、引用の文が名指す語から今の symbol への最小の言い換え（`prev_month_comparison = Some(prev_items)` → `get_monthly_sales`、`max-lines-per-function` → `ReturnExchangePage`）にした。引用が支える文と今の code・正本の食い違い（S1-013・S1-067・S1-082）は文を直さず、backlog の「次の docs の lane の候補」の (8) に 1 句を足した。
- S5: 分類表の全行の処置を埋め、照合の入力に repo の中の引用を 1 行ずつ書いた。
- 後続: lane 2b「file:行 の検査」（R3 見込み、`docs/backlog.md` の項目）。

計画からの外れと判断（reviewer が見る所）:

- 名指す語が同じ symbol の中にあっても、文の中身が古い候補と合わない引用は新しい版を採った: S1-006（`:349-359` は採用後の 7 列）、S1-023 の 2 つめ（「本 PR で是正済み」）、S1-042（古い版の範囲は次の文へ跨ぐ）、S2-014（回復の条件を持つのは早期 return の版）。このうち S1-006・S1-042 は AC-S5-5 の `CHECK` が出る。S1-023 の 2 つめと S2-014 は、採った版の行にだけ W が在るので `CHECK` が出ない。S1-022・S2-012 も文の中身で版を採った（処置の欄に根拠）。
- code の comment の bare `:N` の target は、語の要素を定義する file を採った（S2-019 は `registerOptionClass` の在る `ReturnExchangePage.tsx`、S2-020 は商品情報の card の在る `StockMovementsPage.tsx`。test file には test の名前や assertion の文字列として語が出るだけ）。S2-021・S2-022 の X は comment の印を含めた `// :133`。
- backlog の句は、同じ行の同じ版の引用を 1 つの句にまとめた（例「`7e0ccf1b` の時点で `…InventoryRecordsPage.tsx:156-245`、`…ReceivingPage.tsx:393`」）。
- library の行（S1-017〜S1-020・S1-041・S1-079・S2-009〜S2-011）は、引用を足した hit の時点の lock の版を、`K` の綴りで同じ行に書いた。
- S3-014 の `:70` の target は、手順 1 の「同じ文が語で名指す file」でなく、書かれた時点の文書自身（backlog がその中にあった時点の `docs/Plans.md`）にした（手順の書き方からの外れ。Final Review の closure の P3 で closeout に記した）。

AC-S5-5 の loop の出力: PR の comment（Coordinator が投稿）— 目次と `CHECK` の一覧 https://github.com/higekiri-w/inventory-system-desktop/pull/183#issuecomment-6309869869 、全文は続く 13 本、Final Review の是正で変わった S3-014・S3-024 の塊の差し替え https://github.com/higekiri-w/inventory-system-desktop/pull/183#issuecomment-6311300061

AC-S5-2 の抜き取り（35 行。各行の最初の引用。抜き取りに当たった library の S2-009 は S2-013 で代えた。S1-015・S1-080 は S1-016・S1-081 で、S1-020 は S1-021 で代えた）:

| ID | 引用 | 候補を引く command と候補（古い順） | 採った版と `git show` | その版の行の symbol・節 | 書き換え先 | 今の base の `rg -n` |
|---|---|---|---|---|---|---|
| S1-005 | `ListShell.tsx:69` | `git log --format=%h --pickaxe-regex -S'ListShell\.tsx:69([^0-9]\|$)' origin/main` → ff655efd〈親〉・b59d7331 | `ff655efd`〈親〉: `git show ff655efd:src/components/patterns/ListShell.tsx \| sed -n '69,69p'` | `ListShell` の引数 `topSummary = false` | `ListShell` の引数 `topSummary` の既定 | `rg -n 'topSummary = false' src/components/patterns/ListShell.tsx` → 128 行 |
| S1-010 | `BackupRestorePage.tsx:520` | `git log --format=%h --pickaxe-regex -S'BackupRestorePage\.tsx:520([^0-9]\|$)' origin/main` → 654f95d5〈親〉・9f90964d・ff05436f〈親〉・e188366e | `654f95d5`〈親〉: `git show 654f95d5:src/features/backup-restore/BackupRestorePage.tsx \| sed -n '520,520p'` | `BackupRestorePage` の表頭「日時」 | `BackupRestorePage` の表頭 | `rg -n '<TableHead>日時</TableHead>' src/features/backup-restore/BackupRestorePage.tsx` → 505 行 |
| S1-016 | `RootLayout.tsx:65` | `git log --format=%h --pickaxe-regex -S'RootLayout\.tsx:65([^0-9]\|$)' origin/main` → d21c9613〈親〉・22504af9・696e3e71〈親〉・f1d5590c・31c069d5〈親〉・7e0ccf1b・ff655efd〈親〉・b59d7331 | `f1d5590c`〈hit〉: `git show f1d5590c:src/components/layout/RootLayout.tsx \| sed -n '65,65p'` | `RootLayout` の `<main data-scroll-restoration-id="main">` | `RootLayout` の `<main>` | `rg -n 'data-scroll-restoration-id="main"' src/components/layout/RootLayout.tsx` → 68 行 |
| S1-021 | `SupplierManagementPage.tsx:35` | `git log --format=%h --pickaxe-regex -S'SupplierManagementPage\.tsx:35([^0-9]\|$)' origin/main` → 1c896139〈親〉・ee8e294a・41336c28〈親〉・689ac4c3・43b69bea〈親〉・c9fabc17・f39c560a〈親〉・bc01698f | `c9fabc17`〈hit〉: `git show c9fabc17:src/features/suppliers/SupplierManagementPage.tsx \| sed -n '35,39p'` | `SupplierManagementPage` の `PageHeader` の `subtitle` | `SupplierManagementPage` の `PageHeader` | `rg -n 'subtitle=' src/features/suppliers/SupplierManagementPage.tsx` → 42 行 |
| S1-025 | `StockStatusBadge.tsx:29` | `git log --format=%h --pickaxe-regex -S'StockStatusBadge\.tsx:29([^0-9]\|$)' origin/main` → eabc549c〈親〉・f2ef9e52 | `eabc549c`〈親〉: `git show eabc549c:src/features/stock-inquiry/components/StockStatusBadge.tsx \| sed -n '29,29p'` | `StockStatusBadge` の「在庫少」の Badge | 各 file の Badge の label（path と label） | `rg -n '在庫少' src/features/stock-inquiry/components/StockStatusBadge.tsx` → 29 行 |
| S1-030 | `ProductTable.tsx:56` | `git log --format=%h --pickaxe-regex -S'ProductTable\.tsx:56([^0-9]\|$)' origin/main` → 7ae4ade6〈親〉・6ff22472・04113f90〈親〉・4d1ce4c1 | `7ae4ade6`〈親〉: `git show 7ae4ade6:src/features/products/components/ProductTable.tsx \| sed -n '56,56p'` | `ProductTable` の廃番の Badge | 各 file の Badge の label（path と label） | `rg -n '廃番' src/features/products/components/ProductTable.tsx` → 3・65 行 |
| S1-035 | `parse.rs:98` | `git log --format=%h --pickaxe-regex -S'parse\.rs:98([^0-9]\|$)' origin/main` → 67870e10〈親〉・6cac578c | `6cac578c`〈hit〉: `git show 6cac578c:src-tauri/src/biz/csv_import_service/parse.rs \| sed -n '98,110p'` | `parse_and_validate` の数量の上限の検査 | `parse_and_validate`・`execute_commit` の `for row in matched_rows` | `rg -n 'pub fn parse_and_validate' src-tauri/src/biz/csv_import_service/parse.rs` → 22 行 |
| S1-040 | `product_service.rs:907` | `git log --format=%h --pickaxe-regex -S'product_service\.rs:907([^0-9]\|$)' origin/main` → 67870e10〈親〉・6cac578c | `67870e10`〈親〉: `git show 67870e10:src-tauri/src/biz/product_service.rs \| sed -n '907,927p'` | `list_low_stock`（doc comment と本体） | `list_low_stock`・`list_low_stock_products` | `rg -n 'pub fn list_low_stock' src-tauri/src/biz/product_service.rs` → 923 行 |
| S1-045 | `lib.rs:118` | `git log --format=%h --pickaxe-regex -S'lib\.rs:118([^0-9]\|$)' origin/main` → 02f497ea〈親〉・dbb83251 | `02f497ea`〈親〉: `git show 02f497ea:src-tauri/src/lib.rs \| sed -n '118,120p'` | `prepare_database` の `std::env::current_dir()` の closure | `prepare_database`・`prepare_database_with_init`・`migrate_legacy_db_with_ops` | `rg -n 'fn prepare_database\(' src-tauri/src/lib.rs` → 114 行 |
| S1-050 | `` `:41 `` | `git log --format=%h --pickaxe-regex -S'`:41([^0-9]\|$)' origin/main` → 902647b8・7ae4ade6〈親〉・6ff22472・ff655efd〈親〉・b59d7331・f184d5a3〈親〉・909fe096・3ded9034〈親〉・2a36b086・9d351d5b〈親〉・dc2f14f1・ff05436f〈親〉・e188366e・df0488eb〈親〉・294ba366・a5fb9b16〈親〉・8bb5e5a6・2194ed56〈親〉・2ba91dea・c1beb334〈親〉・3d71cc23・67870e10〈親〉・6cac578c・2753e047〈親〉・294b0dfe・c1c1c5bf〈親〉・c7ce0599 | `9d351d5b`〈親〉: `git show 9d351d5b:src-tauri/src/io/ej_parser.rs \| sed -n '41,41p'` | `EjLineKind::Quantity` の `quantity` | `EjLineKind::Quantity`・`EjItem`・`EjLineKind::ItemCount`・`parse_item_count`・`EjRestoration::Restored`・`item_region`・`restore_items` | `rg -n '    Quantity \{' src-tauri/src/io/ej_parser.rs` → 40 行 |
| S1-055 | `parse.rs:63` | `git log --format=%h --pickaxe-regex -S'parse\.rs:63([^0-9]\|$)' origin/main` → ea15360d〈親〉・02f497ea | `ea15360d`〈親〉: `git show ea15360d:src-tauri/src/biz/csv_import_service/parse.rs \| sed -n '63,77p'` | `parse_and_validate` の 2a の精算日の検査 | `parse_and_validate` の `2a. settlement_date 業務妥当性チェック` | `rg -n '2a. settlement_date' src-tauri/src/biz/csv_import_service/parse.rs` → 63 行 |
| S1-060 | `ej_parser.rs:455` | `git log --format=%h --pickaxe-regex -S'ej_parser\.rs:455([^0-9]\|$)' origin/main` → 3933f0d4〈親〉・f1274d77 | `3933f0d4`〈親〉: `git show 3933f0d4:src-tauri/src/io/ej_parser.rs \| sed -n '455,460p'` | `parse_header` の `date_ok` | `parse_header` の `date_ok` | `rg -n 'let date_ok' src-tauri/src/io/ej_parser.rs` → 456 行 |
| S1-065 | `lib.rs:277` | `git log --format=%h --pickaxe-regex -S'lib\.rs:277([^0-9]\|$)' origin/main` → 6b765f75〈親〉・3c568364 | `6b765f75`〈親〉: `git show 6b765f75:src-tauri/src/lib.rs \| sed -n '277,277p'` | `export_bindings_to` の `collect_commands!` | `export_bindings_to`・`run` | `rg -n 'fn export_bindings_to' src-tauri/src/lib.rs` → 274・564・588・602 行 |
| S1-070 | `sales_service.rs:196` | `git log --format=%h --pickaxe-regex -S'sales_service\.rs:196([^0-9]\|$)' origin/main` → 902647b8 | — | 取り戻せない（root の 196〜197 行は `get_daily_sales` の部門の行） | `get_monthly_sales` | `rg -n 'prev_month_comparison = Some\(prev_items\)' src-tauri/src/biz/sales_service.rs` → 305 行 |
| S1-075 | `backup.rs:263` | `git log --format=%h --pickaxe-regex -S'backup\.rs:263([^0-9]\|$)' origin/main` → 7f98205c〈親〉・2e3e97d3 | — | `restore_backup` の `.restore_backup`（今の base は `restore.rs` へ委ねる包み → 消えた） | — | `rg -n 'pub fn restore_backup' src-tauri/src/mnt/backup.rs` → 95 行 |
| S1-081 | `` dialog.tsx` L62 `` | `git log --format=%h --pickaxe-regex -S'dialog\.tsx` L62([^0-9]\|$)' origin/main` → 902647b8 | `902647b8`〈hit〉: `git show 902647b8:src/components/ui/dialog.tsx \| sed -n '62,69p'` | `DialogContent` の `DialogPrimitive.Close` | `DialogContent` の `DialogPrimitive.Close` | `rg -n 'DialogPrimitive.Close' src/components/ui/dialog.tsx` → 22・23・63・69 行 |
| S1-085 | `SCREEN_DESIGN.md L120` | `git log --format=%h --pickaxe-regex -S'SCREEN_DESIGN\.md L120([^0-9]\|$)' origin/main` → 902647b8 | — | 取り戻せない（root の L120-133 は PLU 書出し・バックアップの節） | — | `rg -n '^### 在庫照会画面' docs/SCREEN_DESIGN.md` → 251 行 |
| S2-001 | `parse.rs:80` | `git log --format=%h --pickaxe-regex -S'parse\.rs:80([^0-9]\|$)' origin/main` → 09f65412〈親〉・86c32817・6b765f75・8cff232a〈親〉・db0b8509 | `09f65412`〈親〉: `git show 09f65412:src-tauri/src/biz/csv_import_service/parse.rs \| sed -n '80,80p'` | `parse_and_validate` の行数ガード | `parse_and_validate` の `CSV_IMPORT_LINE_LIMIT` | `rg -n 'CSV_IMPORT_LINE_LIMIT' src-tauri/src/biz/csv_import_service/parse.rs` → 80 行 |
| S2-003 | `` `:443 `` | `git log --format=%h --pickaxe-regex -S'`:443([^0-9]\|$)' origin/main` → 31c069d5〈親〉・7e0ccf1b・07302b5e〈親〉・1ad9aa74・7ae4ade6〈親〉・6ff22472・1c896139〈親〉・ee8e294a・ff655efd〈親〉・b59d7331・43b69bea〈親〉・c9fabc17・85b18a04〈親〉・42eddd0a・a5fb9b16〈親〉・8bb5e5a6 | `31c069d5`〈親〉: `git show 31c069d5:docs/design-system/01-decision-rules.md \| sed -n '443,443p'` | DSR-22 の節 | DSR-22 | `rg -n '^## DSR-22' docs/design-system/01-decision-rules.md` → 118 行 |
| S2-006 | `sort-items.test.ts:79` | `git log --format=%h --pickaxe-regex -S'sort-items\.test\.ts:79([^0-9]\|$)' origin/main` → 902647b8 | `902647b8`〈hit〉: `git show 902647b8:src/features/monthly-sales/lib/sort-items.test.ts \| sed -n '79,90p'` | test「G-3: ranking field 残存確認」 | test「G-3: ranking field 残存確認」 | `rg -n 'G-3' src/features/monthly-sales/lib/sort-items.test.ts` → 81 行 |
| S2-012 | `ProductForm.tsx:284` | `git log --format=%h --pickaxe-regex -S'ProductForm\.tsx:284([^0-9]\|$)' origin/main` → 41336c28〈親〉・689ac4c3 | `689ac4c3`〈hit〉: `git show 689ac4c3:src/features/products/components/ProductForm.tsx \| sed -n '284,284p'` | `ProductForm` の部門の `Select` の `onValueChange` | `ProductForm` の部門の `Select` の `onValueChange` | `rg -n 'update\("departmentId", Number\(value\)\)' src/features/products/components/ProductForm.tsx` → 288 行 |
| S2-013 | `ProductForm.tsx:282` | `git log --format=%h --pickaxe-regex -S'ProductForm\.tsx:282([^0-9]\|$)' origin/main` → 41336c28〈親〉・689ac4c3 | `689ac4c3`〈hit〉: `git show 689ac4c3:src/features/products/components/ProductForm.tsx \| sed -n '282,282p'` | `ProductForm` の部門の `Select` の `value` | `ProductForm` の部門の `Select` の `value` | `rg -n 'values.departmentId === null' src/features/products/components/ProductForm.tsx` → 286 行 |
| S2-015 | `.gitignore:38` | `git log --format=%h --pickaxe-regex -S'\.gitignore:38([^0-9]\|$)' origin/main` → 902647b8 | `902647b8`〈hit〉: `git show 902647b8:.gitignore \| sed -n '38,38p'` | `.gitignore` の `# src/lib/bindings.ts` の行 | `.gitignore` の `# src/lib/bindings.ts` の行 | `rg -n 'src/lib/bindings.ts' .gitignore` → 38 行 |
| S2-018 | `SCREEN_DESIGN L42` | `git log --format=%h --pickaxe-regex -S'SCREEN_DESIGN L42([^0-9]\|$)' origin/main` → 5aad8f59〈親〉・b3921181・04514154〈親〉・c6b6eb7a | `5aad8f59`〈親〉: `git show 5aad8f59:docs/SCREEN_DESIGN.md \| sed -n '42,42p'` | 「年に数回 / 初回のみ」の表の 16 | SCREEN_DESIGN の「年に数回 / 初回のみ」・「全体構成（4エリア）」・「実装フェーズ分類」 | `rg -n '^### 年に数回 / 初回のみ' docs/SCREEN_DESIGN.md` → 45 行 |
| S2-021 | `// :133` | `git log --format=%h --pickaxe-regex -S'// :133([^0-9]\|$)' origin/main` → 1ad9aa74〈親〉・e78d3c5c | `1ad9aa74`〈親〉: `git show 1ad9aa74:src/hooks/useUnsavedChangesWarning.test.tsx \| sed -n '133,133p'` | test「DSR-20 D-E T10: Escape と外側 pointer-down では閉じず callback も発火しない」 | sibling test「DSR-20 D-E T10: Escape と外側 pointer-down では閉じず callback も発火しない」 | `rg -n 'DSR-20 D-E T10: Escape と外側' src/hooks/useUnsavedChangesWarning.test.tsx` → 133・154 行 |
| S3-003 | `` `:164 `` | `git log --format=%h --pickaxe-regex -S'`:164([^0-9]\|$)' origin/main` → dda8560a〈親〉・2c53f79c・d2d41d21〈親〉・a5fb9b16・2194ed56〈親〉・2ba91dea・6cac578c〈親〉・18c6b591・44f1e666〈親〉・c1c1c5bf | `dda8560a`〈親〉: `git show dda8560a:docs/function-design/58-ui-stock-inquiry.md \| sed -n '164,164p'` | （backlog は版の句。手順 3・4 は使わない） | `dda8560a` の時点で `docs/function-design/58-ui-stock-inquiry.md:164` | `` rg -n '`dda8560a` の時点で' docs/backlog.md `` → 49 行 |
| S3-006 | `35-biz-stocktake-service.md:473` | `git log --format=%h --pickaxe-regex -S'35-biz-stocktake-service\.md:473([^0-9]\|$)' origin/main` → 67870e10〈親〉・6cac578c・02f497ea〈親〉・dbb83251 | `67870e10`〈親〉: `git show 67870e10:docs/function-design/35-biz-stocktake-service.md \| sed -n '473,473p'` | （backlog は版の句。手順 3・4 は使わない） | `67870e10` の時点で `docs/function-design/35-biz-stocktake-service.md:473` | `` rg -n '`67870e10` の時点で' docs/backlog.md `` → 49・53 行 |
| S3-009 | `21-io-inventory-repo.md:103` | `git log --format=%h --pickaxe-regex -S'21-io-inventory-repo\.md:103([^0-9]\|$)' origin/main` → 32273dc2〈親〉・3b94a58a・63896441 | `32273dc2`〈親〉: `git show 32273dc2:docs/function-design/21-io-inventory-repo.md \| sed -n '103,103p'` | （backlog は版の句。手順 3・4 は使わない） | `32273dc2` の時点で `docs/function-design/21-io-inventory-repo.md:103` | `` rg -n '`32273dc2` の時点で' docs/backlog.md `` → 95 行 |
| S3-012 | `PriceRevisionTable.tsx:104` | `git log --format=%h --pickaxe-regex -S'PriceRevisionTable\.tsx:104([^0-9]\|$)' origin/main` → 07302b5e〈親〉・1ad9aa74・7ae4ade6〈親〉・6ff22472・ff05436f〈親〉・e188366e | `07302b5e`〈親〉: `git show 07302b5e:src/features/products/components/PriceRevisionTable.tsx \| sed -n '104,104p'` | （backlog は版の句。手順 3・4 は使わない） | `07302b5e` の時点で `src/features/products/components/PriceRevisionTable.tsx:104` | `` rg -n '`07302b5e` の時点で' docs/backlog.md `` → 119 行 |
| S3-015 | `mockup-d-lists.html:112` | `git log --format=%h --pickaxe-regex -S'mockup-d-lists\.html:112([^0-9]\|$)' origin/main` → ff655efd〈親〉・b59d7331・f39c560a〈親〉・bc01698f | `ff655efd`〈親〉: `git show ff655efd:docs/design-system/reference/mockup-d-lists.html \| sed -n '112,112p'` | （backlog は版の句。手順 3・4 は使わない） | `ff655efd` の時点で `docs/design-system/reference/mockup-d-lists.html:112` | `` rg -n '`ff655efd` の時点で' docs/backlog.md `` → 145 行 |
| S3-018 | `ReturnExchangePage.tsx:185` | `git log --format=%h --pickaxe-regex -S'ReturnExchangePage\.tsx:185([^0-9]\|$)' origin/main` → f184d5a3〈親〉・909fe096・f39c560a〈親〉・bc01698f | — | 取り戻せない（185 行はどの候補でも `max-lines-per-function` の要素でない） | `ReturnExchangePage` の上の `max-lines-per-function` | `rg -n 'max-lines-per-function' src/features/return-exchange/ReturnExchangePage.tsx` → 184 行 |
| S3-021 | `61-ui-receiving.md:30` | `git log --format=%h --pickaxe-regex -S'61-ui-receiving\.md:30([^0-9]\|$)' origin/main` → 949a5c4c〈親〉・c3a006ec | `949a5c4c`〈親〉: `git show 949a5c4c:docs/function-design/61-ui-receiving.md \| sed -n '30,30p'` | （backlog は版の句。手順 3・4 は使わない） | `949a5c4c` の時点で `docs/function-design/61-ui-receiving.md:30` | `` rg -n '`949a5c4c` の時点で' docs/backlog.md `` → 223 行 |
| S3-024 | `ReturnExchangePage.tsx:790` | `git log --format=%h --pickaxe-regex -S'ReturnExchangePage\.tsx:790([^0-9]\|$)' origin/main` → 31c069d5〈親〉・7e0ccf1b・f39c560a〈親〉・bc01698f | `31c069d5`〈親〉: `git show 31c069d5:src/features/return-exchange/ReturnExchangePage.tsx \| sed -n '790,791p'` | `{errors.items}` の段落（同じ版の `return-exchange-request.ts` が「交換では戻り明細と渡し明細がそれぞれ必要です」を入れる） | `31c069d5` の時点で `src/features/return-exchange/ReturnExchangePage.tsx:790-791` | `` rg -n '`31c069d5` の時点で' docs/backlog.md `` → 247・249・251・341 行 |
| S3-027 | `Pagination.tsx:52` | `git log --format=%h --pickaxe-regex -S'Pagination\.tsx:52([^0-9]\|$)' origin/main` → 7e0ccf1b〈親〉・6fc4abac・f39c560a〈親〉・bc01698f | `7e0ccf1b`〈親〉: `git show 7e0ccf1b:src/components/patterns/Pagination.tsx \| sed -n '52,53p'` | （backlog は版の句。手順 3・4 は使わない） | `7e0ccf1b` の時点で `src/components/patterns/Pagination.tsx:52-53` | `` rg -n '`7e0ccf1b` の時点で' docs/backlog.md `` → 245・259・261・334 行 |
| S3-030 | `button.tsx:15` | `git log --format=%h --pickaxe-regex -S'button\.tsx:15([^0-9]\|$)' origin/main` → 7e0ccf1b〈親〉・6fc4abac・f39c560a〈親〉・bc01698f | `7e0ccf1b`〈親〉: `git show 7e0ccf1b:src/components/ui/button.tsx \| sed -n '15,16p'` | （backlog は版の句。手順 3・4 は使わない） | `7e0ccf1b` の時点で `src/components/ui/button.tsx:15-16` | `` rg -n '`7e0ccf1b` の時点で' docs/backlog.md `` → 245・259・261・334 行 |

## Review Response

先行 round の結果・評価（判定・件数・採否・reviewer の意見）はこの節にだけ書く。前半の節と遷移記録には「round N の是正（Review Response 参照）」だけを書く（独立 review は `## Review Response` より前だけを読む）。

### Plan Review round 1（監査 `4e4bc1ce`）

- Codex（発注 460）: approve、P3 1。
- Claude 側（fresh Opus 5.5、fork でない）: reject、P2 2 / P3 2。
- 相談役（Fable 5.1、fresh、read-only）の反例探しを是正に使った。分類表の全 326 引用に候補の出し方 4 通りを機械で当て、F1 の代わりの形（候補の集合を古い順に歩く）と AC の追加を出した。相談の run は review の数に入らない。
- 裁定: Coordinator が全 finding を受理。

| # | 出所 | 重大度 | 指摘 | 直した所 |
|---|---|---|---|---|
| F1 | Opus | P2 | C を「path で絞らない `-S` の最も古い commit」とする定義が、別の文書の同じ file:N（S1-028 `StockStatusBadge.tsx:36`〈最古 `2f8a6f61`、正しい版 `f2ef9e52`〉・S1-016・S1-031）と前方一致（S1-045 `lib.rs:118` → `lib.rs:1188`）で誤った版を採り、±3 行の窓の照合が誤りを通す | Scope「書き換えの方法」の 1・2 を置き換えた: 候補 = path で絞らない `--pickaxe-regex -S'<X>([^0-9]\|$)'` の全 hit と親を古い順に、名指す語を含む symbol・要素の中に行 N がある最古の候補を採る。path で絞る・最古の 1 つを採る・長い文字列・`-L`・`git blame` を使わない理由を実例 1 つずつで書いた。手順の試し 2 件・分類表の凡例（hit か親か）・AC-S5-2・Design Readiness (1)・Review Focus の方法・Implementation Results の約束を同じ語に揃えた |
| F2 | Opus | P2 | root `902647b8` が C になる 10 行で `C^` が無く手順 2 が定義されない。S4 の最小の言い換えの根拠が定まらず、今の行（`sales_service.rs:196` の `pub count: usize`）から付け替える道が残る | 手順 6 に「root だけが候補で合わない行」を含めて S4 へ送ると書いた（root の特例の条項は置かない）。S4「最小の言い換え」の根拠を、引用の文自身が名指す語と、それが今どの symbol にあるかに限り、今の行 N の中身を根拠にしないと書いた |
| F3 | Opus | P3 | 行に path が無い bare の `` `:N` `` の target の決め方が無い | 手順 1 の末尾に、target を同じ節の直前の path（無ければ文書の対象 module）とし根拠を処置の欄に書くと足した。手順 5 の正本の形に、symbol が隣にあれば採った版でその symbol が行 N を含むと確かめて `:N` だけを外すと足した |
| F4 | Opus | P3 | backlog の句の `{path}` が basename か repo root からの path か書かれていない | 手順 5 の backlog の形に「`{path}` は repo root からの path」を足した |
| C1 | Codex | P3 | 試し 2 の `price_range_error` の定義の行は 949（948 は doc comment） | 手順 1〜2 の試しを 949 行に直した（新しい手順で当て直し、親 `67870e10` が合わず hit `6cac578c` を採ると書いた） |
| AC | 相談役の提案（Coordinator が受理） | — | 抜き取り 30 行だけでは方法の誤りを拾えない | S5 に全引用の照合の evidence の段、分類表の後に「照合の入力」の block、AC-S5-5（block を読み、候補と各候補の行 N、採った版の行を出し、機械で分かる食い違いを `NG` にする loop）を足した。出力は PR の comment に置き、Implementation Results には link だけ（D-038） |

あわせて直した所: 手順 5 の library の形に、版を引いている package の版で書く 1 句（Opus の Q2 の補足）。手順 5 の正本の形に、`b2389b19` 起源のような時点の句を引き直しの後に残すかを処置の欄で判じる 1 句（Opus の Q3 の補足）。置き換えた手順の旧い文（最も古い commit を C とする・C と C^ の照合）は、この節の指摘の引用の他に残していない。

### Plan Review round 2（監査 `dc17f183`）

- Codex（発注 464）: approve、P3 1。
- Claude 側（fresh Opus 5.5、fork でない）: reject、P2 3 / P3 8。
- 相談役（Fable 5.1、fresh、read-only）の反例探し 2 回目を是正に使った。`P` の外の形の網羅（a〜n の 14 形）、AC-S5-5 の loop への注入（T1・T6・T10・T3b・T11）、方法の文の曖昧さ A1〜A6 と、段落ごとの置き換えの案を出した。相談の run は review の数に入らない。案は reviewer でない相談役の案なので、現物で確かめてから入れた（下の「案から変えた点」）。
- 裁定: Coordinator が全 finding を受理。

| # | 出所 | 重大度 | 指摘 | 直した所 |
|---|---|---|---|---|
| F1 | Opus | P2 | `P` の拡張子の一覧が `index.d.mts:23`（`01-decision-rules.md:362`）と `.gitignore:38`（`src-tauri/src/lib.rs:244`）を拾わず、`catalog:588` が S3-029 の引用の列に無い。自己点検は `P` を D-125 の 3 つの形と同じ範囲と書く | AC の `P` を 4 つの形（拡張子つきの `path:N`・bare の `:N`・`L<N>`・`#L<N>`）に置き換え、`K` に `@radix-ui/[a-z-]+` を足した。分類表に S1-079〜S1-086・S2-015〜S2-022 を足し、既存の行の引用の列に 17 個（`,N` 10・bare の `:N` 6・`catalog:588`）を足した。拡張子の一覧と、それに付けた `.rb:N` の例外を消し、Design Readiness の自己点検を 4 つの形の主張に置き換えた。Scope の数え直しの表・S1〜S3 の見出し・規模・baseline を新しい `P` で測り直した |
| F2 | Opus | P2 | `,N` の 2 つ目以降（`CsvImportRecordDetailPage.tsx:41,140` ほか）が引用の列・照合の入力・方法に無い | Scope に「引用の数え方」の段落を置いた: `,N`・`/:N`・`L<N>` の後の `・N` は別の引用、範囲 `N-M`・`:N`〜`:M` は 1 つの引用（照合の入力の N は `N-M`）。分類表の引用の列を書かれた形のまま（`-` の範囲は `:N-M`）にし、S5 の照合の入力の書き方に 1 行ずつと書いた。手順 2 に範囲と `,N` の文（`41,140` の例） |
| F3 | Opus | P2 | 処置の列に「残す」の語があるだけで AC-S5-5 の数の検査を飛ばす | loop の skip を先頭一致（`残す*`・`'lane 2b'*`）にした。手順 5 の時点の句は処置の先頭を「書き換えた」にして括弧の中に理由。分類表の凡例と S5 に処置を先頭の語で読むと書いた。S4 の backlog の「当時の行として残す」は、backlog を句の形 1 つにしたので消した（下の「案から変えた点」） |
| F4 | Opus | P3 | AC-S3-1 と `K` の除外は行の単位で、1 行に句が 1 つあれば同じ行の別の版の引用が漏れても通る | AC-S3-3 を、場所の行の `docs/backlog.md` の句の SHA の集合と照合の入力のその ID の版の集合を照らす loop に置き換えた |
| F5 / C1 | Opus / Codex 464 | P3 | AC-S5-5 の loop が候補ごとに行 N の 1 行しか出さず、`<PageHeader` の後の `subtitle` が出ない | 候補ごとに N−3〜M+3 行を出す |
| F6 | Opus | P3 | symbol をどの粒度で判じるか書いていない | 手順 2 を「行 N を含む最も内側の要素・文が名指す語を含む最古の候補」に置き換えた（関数の単位で判じない） |
| F7 | Opus | P3 | bare の `:N` の塊が文を含みうる。target に文が語で名指す file を使えない | 手順 1 の X の文（記号だけで続く塊、文を挟むなら `:N` だけ、`L<N>` の X）と target の文（同じ文が名指す file、code の comment は comment の在る file と test の対象の file を `rg`）を置き換えた |
| F8 | Opus | P3 | S4 の最小の言い換えで名指す語が複数の symbol に当たるときの決め方が無い | S4 に「今の base の `rg` で 1 つの symbol に決まらなければ、引用を外すか backlog の 1 句へ送る」 |
| F9 | Opus | P3 | AC-S5-2 の抜き取りに外部 library の行が当たる。AC-S5-5 の採った版の照合が行のどの列でも通る | AC-S5-2 に、外部 library・repo 外の資料の行も「抜き取りに入っていない次の ID」で代えると足し、行の数を新しい ID の範囲の 35 にした。AC-S5-5 は採った版の欄（第 6 列）だけを照らす |
| F10 | Opus | P3 | Owner Effort Budget が Ready と merge を合わせて 1 つの介入と数える（decision point の単位で数える `docs/DEV_WORKFLOW.md` Owner Effort Budget と合わない） | 残りの見込み 2（Ready 1・merge 1）、予備 3、合計 6 = 1 + 2 + 3 |
| F11 | Opus | P3 | 差の段落の 231 が前文の 221 と同じ「起点の調査」の数に読める | 差の段落を「`18db621e` の tree を今の `P` で数え直した数 → 今の数」にし、新しい `P` で測り直した（正本 242 → 241、backlog 124 → 85） |
| T1・T6・T10・T3b | 相談役 §2（Coordinator が受理） | — | loop が X の複製・target の差し替え・表の外の同じ ID・`'lane 2b'` の語を通す | AC-S5-5 の loop を block ごと置き換えた（分類表の行を `#### S1 の行`〜`#### 照合の入力` の間だけから読む、X の中の path で target が終わる、行番号の多重集合）。AC-S5-1 を ID の行の `\| 未処置` に置き換えた。AC-S1-4 を足した |
| A3・A5・A6 | 相談役 §3（Coordinator が受理） | — | `docs/screen_mockups.html`・target が archive・名指す語が無い引用の主題 | 手順 5 に `docs/screen_mockups.html`（節が無いので外すか 6）と archive の target の句、手順 2 に名指す語が無い引用は引用を含む文の主題の語を書き出して処置の欄に書くと足した |

案から変えた点（現物で確かめた結果）:

- loop の X の path の取り出し: 案の `${x%%:*}`・`${x%' L'*}` は、`（:147` から `（`、`` `src/components/ui/dialog.tsx` L62 `` から閉じの backtick つきの path を取り、正しい入力にも `NG` を出す（bash で確かめた）。X の中の最初の拡張子つきの名前を取る形にした。
- `-` の範囲の引用の列: 案の数の照合は引用の列の `N-M` を両端で数えるが、分類表の列は `rg -o` の token で `-M` を落としていた。既存の行（S1-001〜S1-078・S2-001〜S2-014・S3）で範囲を書いた引用は `0a8e8fcd` で数えて 48 個（`rg -o` の token に `-M` を書き足した 46 個、S1-020 の手で足した `:146-160`、S3-031 に足した `:642-648`）で、その `-M` を書き足した（足さないと正しい入力でも `NG`）。
- 手で足す引用: 案の一覧（§5.3）は 15 個で案の本文の「17」と合わず、`docs/backlog.md:261` の `Pagination.tsx:52-53,99` の `,99` と、S1-019 の `scroll-restoration.js:73,106,114` の `,114` が無い。S3-027 に足した。bare の 6 個（S1-059・S3-021・S3-031）は新しい `P` の `rg -o` が拾うので、手で足したのは `,N` 10・`catalog:588`・`L<N>` の後の 3 個の 14 個。
- S1 の残す・lane 2b の行: 案の「8 行」は 7 行（起票の 6 行と S1-080）。
- 範囲の決め: 案の §5.2 は `〜:N` を別の引用の例に挙げつつ `:N`〜`:M` を 1 つの引用とし、食い違う。`〜` は範囲の記号として 1 つの引用にした（相談役 A1。数の照合は両端の数字なのでどちらでも同じ結果）。
- backlog の形: AC-S3-3 の「句の SHA の集合 = 照合の入力の版の集合」は、backlog の引用を symbol で引き直す道を残すと（照合の入力に版はあるが句が無い）成り立たない。backlog は句の形だけにし、Goal の「または symbol・節で引き直してある」、手順 5 の「symbol・節で引き直してもよい」、S4 の「当時の行として残す」を消した（D-127。手順 3・4 は正本と comment だけ）。
- AC-S3-3 の loop は案に無く、新たに書いた。

数え直しの再現: 案の §1.2 の値（live な docs 全体 466・AC-S1-1 79・AC-S1-2 5・src 22・backlog の `rg -n` 31・AC-S1-3 2 行・invalidation-contract 11）は bash と zsh で逐語に再現した。案に無かった値: 正本 241 / 86、backlog の `rg -o` 85、reference 136、AC-S5-1 132、AC-S5-5 の `NG` 122・`--` 10。

注入の再現（packet の copy、bash と zsh で出力は同じ）: 正しい入力 T0 は `NG` なし。T1 は `NG S1-037: 引用の行番号 […] / 照合の入力 [949 ×7]`、T6 は `NG S3-001: target … が X の path … で終わらない`、T10 は `NG S2-014: 分類表の採った版の欄に ff655efd が無い`、T3b は `NG S2-004`。round 2 の packet の loop は、この 4 つのどれにも `NG` を出さなかった。

置き換えた旧い記述（拡張子の一覧の `P` と `.rb:N` の例外、旧い loop と `*残す*`・`*'lane 2b'*` の skip、旧い AC-S5-1 の式、AC-S3-3 の「読んで確かめる」、引用の列の「範囲の終わりと `,` の 2 つ目以降は省いている」、backlog の symbol の道）は、この節の指摘の引用の他に残していない。

### Plan Review round 3（監査 `8b322d78`、天井）

- Codex（発注 469、Sol）: approve、P3 1。
- Claude 側（fresh Opus 5.5、fork でない）: approve、P3 5。
- 裁定（disposition）: 天井の後なので round は足さず、Coordinator が全部の P3 を同型の一括是正として受理した。新しい契約は P3-2 の `CHECK` だけ。

| # | 出所 | 指摘 | 直した所 |
|---|---|---|---|
| P3-1 | Opus #1 | AC-S5-5 の loop が、照合の入力の X が分類表の場所の行に在るかを確かめない（S1-028 の X を `` `:36 `` と書くと候補が広がり、`NG` が出ない） | loop に、`origin/main` の場所の file・行に X が在るかの確認を足し、無ければ `NG`。AC-S5-5 の `NG` の説明と分類表の凡例（場所の行の読み方）を合わせた |
| P3-2 | Opus #2 | 全部の候補の塊（`1,675` 個の実測）を reviewer 1 人が読む前提が重い | 照合の入力に名指す語 W の列を足し（`ID 採った版 target N W X`）、`*` の版の N〜M に W が無い・より古い候補に W が在る・W が `-` のときだけ `CHECK <ID>` を出す。S5・AC-S5-5・Review Focus に、Final Review の reviewer は `CHECK` の塊と AC-S5-2 の抜き取りを読むと書いた |
| P3-3 | Opus #3 | `K` は「名前 版」の並びだけを拾うのに、手本の行は名前を backtick で囲む | 方法の 5 の library の項に、`K` の綴りで backtick で囲まず `名前 版` の形で書くと書いた |
| P3-4 | Opus #4 | library の行の処置の先頭が「書き換えた」か「残す」か決まっていない | 方法の 5 に処置を「残す（library の版を足した観測）」と書き、Goal の 4 つめの文を揃えた |
| P3-5 | Opus #5 | X の path が full path か basename か曖昧。` / L<N>` の X が無い | 手順 1 に「X の path は書かれた path の basename から」と書き、` / L<N>` を記号で続く塊に入れた |
| P3-6 | Codex #1 | S1-019 の `,114`、S1-020 の `146-160`、S1-086 の途中の backtick が引用の列に無い | 3 行の引用の列を現物どおりに転記し、S1 を 248 引用・手で足した 7 に直した |

- round 2 の数の訂正: round 2 の是正で「手で足した 13 個」「既存の行で `-M` を落とした 46 個」と書いたのは、S1-019 の `,114` と S1-020 の `:146-160` の `-160` を数え漏らしていた。正しくは 14 個と、範囲を書いた引用 48 個（`0a8e8fcd` で数え直した。closure の N4 で 47 から直した）で、Scope・分類表の凡例・S1 の見出し・round 2 の節の数を直した。
- 測り直し（bash と zsh で出力は同じ）: baseline の値は変わらず、AC-S5-5 は `NG` 122・`--` 10・`CHECK` 0。注入（packet の copy。照合の入力の行は下の逐語）では、S1-028 の X を `` `:36 `` にした入力で `NG S1-028: X が場所 docs/design-system/02-component-catalog.md:891 の行に無い` が出て、正しい X では出なかった。S3-001 の正しい入力は `NG`・`CHECK` なし、候補の中の誤った版（`c1c1c5bf`）と W `-` は `CHECK S3-001`。

```text
S1-028 f2ef9e52 src/features/stock-inquiry/components/StockStatusBadge.tsx 36 在庫あり `:36
S1-028 f2ef9e52 src/features/stock-inquiry/components/StockStatusBadge.tsx 36 在庫あり StockStatusBadge.tsx:36
S3-001 44f1e666 src-tauri/src/seed_demo.rs 462 inventory_movements seed_demo.rs:462
S3-001 c1c1c5bf src-tauri/src/seed_demo.rs 462 inventory_movements seed_demo.rs:462
S3-001 44f1e666 src-tauri/src/seed_demo.rs 462 - seed_demo.rs:462
```

  W `inventory_movements` は `44f1e666` の 462 行の語。手順 1〜2 の試しの語 `sale_auto` は 465 行にあるので、W を `sale_auto` にすると正しい版でも `CHECK S3-001` が出る（closure の U0）。

### closure（監査 `0a8e8fcd`）

- Claude 側（fresh Opus 5.5、fork でない）: approve。round 3 の P3 は 6 つとも closed、新しい P1/P2 は無く、新しい P3 は 4 つ。
- 裁定: Coordinator が 4 つとも受理した。新しい契約・検査の仕組みは足さず、今ある文と loop の最小の変更で直した。

| # | 指摘 | 直した所 |
|---|---|---|
| N1 | W の列の無い 5 列の行は X が空になり、AC-S5-5 の `NG` の検査を全部通る（closure の T5） | loop の頭で X が空なら `NG $id: X が無い（列が足りない）` を出して次へ |
| N2 | basename の X で target の検査が弱くなり、同じ basename の別の module を通す（closure の V1: S1-035 の target を `daily_report_import_service/parse.rs`） | 案 (a) を採った。X は basename のまま、target の検査だけ、場所の行で X の直前に続く path（X の中の拡張子つきの名前を前に伸ばしたもの）で終わることを見る。案 (b)（X を書かれた path に戻す）は文だけの変更で小さいが、basename で書いた入力（V1 そのもの）を通してしまうので採らなかった。loop は場所の行の取り出しを頭へ移し、target の検査の式を 1 行替えた |
| N3 | S1-081・S1-084・S3-031 の引用の列が現物と違う | 3 行を現物どおりに写した。全行を現物の行と照らし直し、残る差は手で足した `,N` 系（凡例の 14 個）だけ |
| N4 | (a) round 3 の注入の W が記録に無い。W の選び方が書かれていない (b) 「47 個」が現物と合わない | (a) round 3 の節に使った入力の行を逐語で足し、S5 に「名指す語が複数あるときは行 N〜M に literal に在る語を選ぶ〈無ければ `-`〉」の 1 句。(b) round 2・round 3 の節の数を、`0a8e8fcd` で数え直した 48 個（S3-031 の `:642-648` を含む）に直した |

- 測り直し（bash と zsh で出力は同じ）: baseline の値は変わらず、AC-S5-5 は `NG` 122・`--` 10・`CHECK` 0、AC-S3-3 は出力なし。
- 注入（packet の copy。照合の入力の行は逐語）: T5 で `NG S1-028: X が無い（列が足りない）`。V1・V2 で `NG S1-035: target src-tauri/src/biz/daily_report_import_service/parse.rs が場所の行の path csv_import_service/parse.rs で終わらない`。V0・正しい S1-028・S1-030 の 2 つの module・S1-086・S3-001 では target の `NG` が出なかった。

```text
S1-028 f2ef9e52 src/features/stock-inquiry/components/StockStatusBadge.tsx 36 StockStatusBadge.tsx:36
S1-035 6cac578c src-tauri/src/biz/csv_import_service/parse.rs 98-110 parse parse.rs:98`〜`:110
S1-035 6cac578c src-tauri/src/biz/daily_report_import_service/parse.rs 98-110 parse parse.rs:98`〜`:110
S1-035 6cac578c src-tauri/src/biz/daily_report_import_service/parse.rs 98-110 parse csv_import_service/parse.rs:98`〜`:110
S1-028 f2ef9e52 src/features/stock-inquiry/components/StockStatusBadge.tsx 36 在庫あり StockStatusBadge.tsx:36
S1-030 - src/features/daily-sales/components/ProductTable.tsx 133 - ProductTable.tsx:133
S1-030 - src/features/products/components/ProductTable.tsx 56 - ProductTable.tsx:56
S1-086 - src-tauri/src/lib.rs 84 run_startup_step lib.rs` `run_startup_step` L84
S3-001 44f1e666 src-tauri/src/seed_demo.rs 462 inventory_movements seed_demo.rs:462
```

  上から T5、V0、V1、V2、正しい S1-028、S1-030 の 2 行、S1-086、S3-001。

### Final Review・merge・closeout（2026-10-21）

Final Review は broad の 3 本（Codex〈発注 478〉= approve〈P3 1〉、Claude 側 fresh Opus 5.5 の全体 = approve〈P3 1〉、fresh Opus 5.5 の深掘り = reject〈P2 2・P3 3〉）で、是正の後の closure 2 本（Codex〈発注 481〉= approve〈指摘 0〉、fresh Opus 5.5 = approve〈P3 1〉）で P1/P2 の残りは 0。broad と closure は専用 record に記録し、Ready の後の Merge gate は pass、Human Gate の ready・merge は owner の承認（repo 外の回答台帳 TD-369）で、PR #183 は helper 経由で 2026-10-21 に merge した。finding の disposition: closure の P3（S3-014 の `:70` の target を、同じ文の別の file でなく書かれた時点の文書自身に決めたのは手順の書き方からの外れ）は本 closeout で Implementation Results の「計画からの外れと判断」に 1 句を足した。broad の深掘りの #3 の P3（AC-S5-5 の loop は target が `docs/archive/**` の file の古い候補を改名前の path で読まない。`git log --follow` で改名前の path も読む形）は lane 2b の候補として `docs/backlog.md` の lane 2b の項目に入れた。他の P3 は是正の commit で直した。
