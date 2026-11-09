# Plan Packet: 棚卸しの数の置き場と在庫数の求め直しを設計正本へ置き換える

wave に属さない単独の lane（design lane、docs だけ。runtime は後続の lane）。設計の見直し（事実だけを集め、白紙の骨組みと今の設計を突き合わせ、owner が採否を決める）を棚卸しの時点証拠と POS（Z004・EJ）の取込みの領域で行い（repo 外の回答台帳 TD-258）、owner が採否を決めた（TD-261・TD-263〜TD-268、店の事実は TD-279・TD-280）結果のうち、棚卸しの数と売上の前後・在庫数の持ち方・Z004 の識別を正本へ置き換える。置換えの順は PLU（D-117、済み）の後（TD-275）。本 lane の branch は `agent/stocktake-pos-redesign-design`（起点 main `ea15360d`）。

規模と手法（起票の最初に数え、file の数は最終の差分で数え直した）:

- 見直しの置換えの全体（発注の Scope）を数えると、旧 ADR（時点証拠）の「時点証拠契約」の節を持つ正本が 32 file（`rg -l '^#+ 時点証拠契約' docs --glob '!docs/archive/**'` を開始 HEAD `ea15360d` の docs で数えた出力 32）、在庫数の持ち方 (m) の code の対象が約 17 file（repo 外の Q8 の範囲の調査）、画面が棚卸し・取込み・商品フォーム・在庫照会・在庫変動・一括インポートの 6、migration の設計 1、wire の型の変更が計数・取込み・商品・回復型の 4 群、横断の約束がかかる経路が 5（計数の保存・入出庫 4 業務・Z004 の確定と取消・商品の登録・整合性。ADR の「横断の約束の表」）。これに加えて、売上の SD からの一括取込み（Z004・日報・EJ を一度に、TD-265）、EJ の控えの閲覧（Q13）、日報の紙と意味づけ（Q10・TD-260・TD-280）があり、1 lane には収まらない。
- **3 つの lane に分けることを提案し、最初の lane（本 packet）だけを起草した。分けるかの最終判断は Coordinator**:
  1. **本 lane: 棚卸しの数の置き場と在庫数の求め直し**（Q1・Q2・Q3・Q4・Q7・Q8・Q9 の Z004 の側・Q11・Q12、停止 ADR の改訂、整合性の縮小、文書の誤り P-05〜P-07 と事実の食い違い X-01・X-03・X-06・X-09〜X-12）。R3。
  2. **売上の SD からの一括取込みと EJ の控えの閲覧**（TD-265・TD-277・Q13、日報の同日追加の守り〈D-071 の日報の側〉）。本 lane の Z004 の契約（32 の新契約）を使うので本 lane の後。R3 の見込み。
  3. **日報の紙と意味づけ**（TD-260・TD-280、紙の並びは見やすさを優先し今の Excel の横長の並びを写さない〈TD-296〉、O-18、D-022・D-025・D-096）。本 lane と file がほぼ分かれる（37・29D・34・56・pos-tables の §12c〜12e）ので並走できる。R3 の見込み（帳票の形と画面）。
  - 後続の runtime（本 lane の後）は 2 つの配布の単位に分けられる（ADR SPEC-STK-CNT-D12。[backlog](../../backlog.md) の「棚卸しの数の置き場と在庫数の求め直しの runtime」）。
- 本 lane の差分: 触った正本は 44 file と新しい ADR 1（`git diff --name-only ea15360d HEAD -- docs ':(exclude)docs/plans'` の出力 45 から新しい ADR を除いて数えた。function-design 24、db-design 4、architecture 5、索引の ARCHITECTURE・DB_DESIGN・FUNCTION_DESIGN・SCREEN_DESIGN・UI_TECH_STACK の 5、ADR 2 と ADR の README、decision-log・backlog・project-memory）。migration 0（後続の runtime が足す 1 本を tracking ほかに設計した）。wire の型 0（後続の runtime で変わる型は 42・41・40 の新契約）。
- 横断の約束は ADR の「横断の約束の表」1 か所にまとめ、各文書の節はそれを引く（PLU の lane と同じ手法）。

## Workflow State

- Phase: archive
- Risk: R3
- Plan Commit: 47dad019d18faf021e16f5804b5f57f6d2f50932
- Amendments: none
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（本 run の起草役）
- Plan Reviewer: fresh Opus 5.5 + Codex（model は発注時に決める）
- Final Reviewer: Fable 5.1（Claude 側、R3・design lane）+ Codex（GPT-6.1 Sol 既定）。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Review Minimum: 1
- Human Gate: ready,merge
- Branch: agent/stocktake-pos-redesign-design

遷移の記録:

1. kickoff → spec-check（2026-10-16、起草役）: owner の決定（repo 外の回答台帳 TD-261・TD-263〜TD-268・TD-279・TD-280）と見直しの成果物（repo 外。事実の一覧・白紙の骨組み・突き合わせ・Q1 の素描 第 3 版・Q2 の第二意見・Q8 の範囲の調査・鮮度の監査）の置換えを Scope にし、規模を数えて 3 lane に分ける案を立て、最初の lane の Risk を R3 と記録した（下の Risk）。
2. spec-check → design（2026-10-16、起草役）: 触る正本（時点証拠 ADR、停止 ADR、D-051・D-071・D-109・D-116、tracking・pos-tables・master-tables・transaction-tables、20・21・22・23・24・29E・30・31・32・33・35・36・37・40・41・42・43・51・55・58・60・65・66・73、architecture の 5、索引の 5）が、旧 ADR の計数 context・受領順の判定・全数読取り・EJ の完全性・legacy 互換・在庫数の保存を契約として持ち、前提の文の一部が事実と合わない。同じ commit で設計正本を更新した（下の Design Readiness）。
3. design → plan-draft（2026-10-16、起草役、同じ commit）: 条件「設計の出力が正本にある」は新しい ADR（SPEC-STK-CNT-D1〜D12）と D-118 と各文書の「計数の置き場と在庫の求め直しの契約」の節で満たす。条件「未解決の設計の問いが無い」は、owner の判断事項（Q1〜Q4・Q7〜Q9・Q11・Q12）が決まり、素描の §6 の 3 つの未決を正本で決めた（入れ直しを省く操作は置かない〈TD-261 の「約束より警告」〉、要再確認の在庫は範囲で見せる、完了の後の要再確認で評価額は変えず差を示す。いずれも owner の決定 TD-296 で確認・決定）ことで満たす。owner に確かめたい 1 点（下の Design Readiness の「前提・制約」の Q-A）は、答えが無いときの既定が今の振舞いで、設計を止めない。plan-gate への遷移は Coordinator。
4. plan-draft → plan-gate（2026-10-17、Coordinator）: packet と Matrix は起草役の commit（`1f2ab8f3`・`ccdc38ff`）にある。Coordinator は packet の全文（Goal・Ordinary Operation・Scope・AC・Design Readiness・Boundary / Wire Contract・Contract Ledger・Contract Probe・Data Safety）と Matrix の節の構成を読み、owner の決定（repo 外の回答台帳 TD-261・TD-263〜TD-268・TD-296）の写しに食い違いが無いこと、差分が docs だけであること（AC8）、追加の行に新しい実データが無いことを確かめた。3 lane に分ける提案を採る（lane 3 は本 lane と file がほぼ分かれ並走でき、lane 2 は本 lane の Z004 の契約に依る）。介入の上限を TD-296 で 9 に改めた。Plan Review round 1 へ出す。
5. plan-gate（round 1 の是正、2026-10-17、Coordinator）: Plan Review round 1（Claude 側 fresh Opus 5.5・Codex GPT-6 Astra）の是正（Review Response 参照）。是正の前に相談役（Fable 5.1）が反例を探し、計数が記録を含むかを 1 つの原理（窓の重なり）から導き直す方向を出した。起草役が `150aa677` で直し、Coordinator が差分（ADR D2・D3 の窓の表と判定の表、D4 の全行 0 の Z004）を読んだ。Scope の範囲と設計の方向は変えず、その場で直したので plan-gate に留める。Plan Review round 2 へ出す。
6. plan-gate（round 2 の是正、2026-10-17、Coordinator）: Plan Review round 2（Claude 側 fresh Opus 5.5・Codex GPT-6 Astra）の是正（Review Response 参照）。原理（窓の重なり）への反例は無く、当て方の抜けを直したうえで、記録が計数に入るかの判定・求めた値の届き先・設定に依る判定の 3 つを全文書で洗い直させた。起草役が `5f96bdfd` で直し、Coordinator が差分（ADR D3 の理由 1 と判定の表、pos-tables の `stock_linked`）を読んだ。Scope に 75 を足したほかは範囲と設計の方向を変えず、その場で直したので plan-gate に留める。Plan Review round 3（天井）へ出す。
7. plan-gate（round 3 の一括是正、2026-10-17、Coordinator）: Plan Review round 3（天井。Claude 側 fresh Opus 5.5 = reject P2 2・P3 5、Ordinary Operation は成立、Codex GPT-6 Astra = reject P2 4・P3 1）の残りを Coordinator が一括是正に振り分けた（DEV_WORKFLOW Review Rules。Review Response 参照）。owner は是正の差分だけを Codex の closure で確かめ、P1/P2 = 0 なら Coordinator が plan-approved に進めると決め、介入の上限を 11 に改めた（repo 外の回答台帳 TD-299、天井を 1 回越える）。起草役が `ae076826` で直し、Coordinator が差分の要所を読んだ。Plan Review closure へ出す。
8. plan-gate（closure の残りの是正、2026-10-17、Coordinator）: Plan Review closure（Codex GPT-6.1 Sol、head `0217ee50`）= reject P2 1・P3 2（Matrix の書き方の残りだけ。正本の是正は妥当）。owner は是正の後に Codex の 2 回目の closure で確かめ、P1/P2 = 0 なら Coordinator が plan-approved に進めると決めた（repo 外の回答台帳 TD-300）。起草役が `b248602c` で直し、Coordinator が差分（Matrix の完了の後の差の case と Ordinary Operation の 12/31 の行）を読んだ。Plan Review closure 2 へ出す。
9. plan-gate → plan-approved（2026-10-17、Coordinator、本 commit）: Plan Review closure 2（Codex GPT-6.1 Sol、head `47dad019`）が approve（P1 0・P2 0・P3 0）。独立の Plan Reviewer が P1 / P2 = 0 を報告し、owner の決定（repo 外の回答台帳 TD-300: closure 2 が P1/P2 = 0 なら Coordinator が plan-approved に進める）により Coordinator が進める。Plan Commit = `47dad019`（承認した版）。
10. plan-approved → implementing（2026-10-17、Coordinator、本 commit。state だけ）: 本 lane は docs だけで、実装の段で足す code は無い。設計の正本は plan-first の commit にあるので、このまま Final Review（R3、Minimum 1）に出す。

## Owner Effort Budget

- 介入回数上限: 11（既定の 6 から owner が 9 に改定〈repo 外の回答台帳 TD-296〉、round 3 の後に 11 に改定〈TD-299〉。介入の時点で全体を顧みずに対症療法へ寄っていないかを考え直してから計画を仕上げる、という条件つき）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 11（TD-296・TD-299） | 11（closeout 時点。判断点ごとに数えた。TD-296 の本 lane に効く判断点 6〈共有 JAN、停止の一括の解除、入れ直しを省く操作を置かない、範囲の表示、完了の後の差、POS の sample は合成だけ〉、Q-A 1〈TD-298〉、round 3 の後の closure の扱いと予算の改定 1〈TD-299〉、closure の後の 2 回目の closure 1〈TD-300〉、Ready 1・merge 1〈TD-302〉。TD-261〜TD-268 は起票の前の見直しの採否で、本 lane の介入に数えない。round の是正は Review Response 参照） | 0 | 0 | 11 = 11 + 0 + 0 |

既定値・数え方・上限に届くときの扱いは `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
DB の表と列（並びの鍵・`stock_counts`・写しの列・`csv_imports` の精算回数）、Tauri command の DTO（計数・取込み・商品・回復型）、POS CSV（Z004 の取り込む条件）、operator workflow（棚卸しの入力・完了・取込みの結果）の契約を決める設計。本 lane は docs だけだが、後続の runtime が写す契約を決めるので R3。R4 ではない: 本 lane はデータを壊す・実店舗データを出す・不可逆な操作をしない（docs の差分だけで、`git revert` で戻る）。後続の runtime は、棚卸しか Z004 の取込みを持つ開発・試験 DB の作り直しと、写しの求め直しを含むので R4 の見込みで、その承認（対象と保全の範囲）は runtime の lane が owner に求める（ADR SPEC-STK-CNT-D10、backlog の該当項目）。在庫の正本の持ち方と停止 ADR に触れるが、停止の範囲・停止 error・部分的な解除の禁止（SPEC-STOP-D6）は変えず、「⑤」が指す配布の単位の名前だけを改めた。この変更で required gate の green / red は変わらない: docs だけで、classifier は function-design と ADR の変更で docs と traceability の検査（rust_drift）を回す。REQ の文と担当の task は変えていないので `docs/function-design/90-traceability.md` は再生成しない（`cargo run --bin generate_traceability -- --check` は OK）。function-design の file の新設・改名・削除は無く、撤去する pub fn は無い（旧 ADR の code は `#[cfg(test)]` の中で、設計照合 test が数えない）ので、`cargo test --test design_compliance_test` は green のまま（AC9）。Scope の予定 path を `scripts/ci/classify-changes.sh` の workflow の一覧（`AGENTS.md`・`CLAUDE.md`・`docs/DEV_WORKFLOW.md`・`docs/ci.md`・`docs/AGENT_OPERATING_MANUAL.md`・`docs/code_review.md`・`docs/quality/review-checklist.md`・`docs/project-profile.md`・`docs/agent-guidance/*`・`docs/templates/*`・`.agents/*`・`.claude/{rules,commands,skills}/*`・PR template と、full に倒す `scripts/*`・`.github/*`）に当てて、当たる file は無い（`docs/project-profile.md`・`docs/code_review.md` の NEL の文は parser の振舞い〈正規化がある〉として今も正しいので触らない）。Final Review Minimum 1。

## Goal

Goal Invariant:

### 最小完了条件

- 後続の runtime の lane の Writer が、チャットの履歴と repo 外の見直しの成果物を見ずに、source docs だけで次を実装できる: 並びの鍵の列と `stock_counts` と写しの列の migration（tracking・pos-tables・master-tables）、求め直しの関数と要再確認の判定（31）、入出庫 4 業務と商品登録の切替（31・30）、Z004 の識別・順の検査・欠けの知らせと確定・取消（32・24）、計数の保存・要再確認の一覧・完了と評価額（35・42・73）、整合性の突合の相手の変更（36）、旧 ADR の `#[cfg(test)]` の code と旧本体の撤去と停止の一括の解除（停止 ADR、ADR SPEC-STK-CNT-D12）。
- 数えた数に入っているか決まらない記録が、要再確認（範囲の表示と入れ直し）か、ADR の Consequences に列挙した受け入れた穴のどれかに必ず当たり、黙ってずれる経路が列挙の外に無い（owner TD-267「黙ってずれなければよい」）。
- 店の操作が今の Excel の棚卸しより増えない: 数えたら保存、前の日の分は日付を選ぶ、締めの取込みの後に要再確認の数個を入れ直す（下の Ordinary Operation）。
- 前提の食い違い（P-05〜P-07、X-01・X-03・X-06・X-09〜X-12）の文が事実に合う。P-08・X-07・X-08 は D-117 で直っていることを確かめた。
- D-118 が decision-log にあり、D-051・D-071・D-109・D-116 に置換えの行がある。時点証拠 ADR は superseded、停止 ADR は「⑤」の指す単位だけを改め SPEC-STOP-D1〜D6 は変わらない。runtime・lane 2・lane 3 が backlog にある。

### 失敗定義

- 正本のどこかに、旧 ADR の計数 context・受領順の判定・全数読取り・相殺の判定・`ej_unverified`・legacy 互換・取消の補償・版（`stock_revision`）を live な契約として残す（置換えの注記・履歴・現行の実装の記述は除く）。
- 写しを書く経路が求め直しの関数の外に残る、または計数が movement を作る契約が残る（求め直しと二重に効く）。
- 決まらない記録のうち、要再確認にも受け入れた穴の列挙にも当たらない経路が残る（例: 同じ置き場の値の違う計数を新しい保存で効かせる）。
- SPEC-STOP-D6 を変える、または「部分的な停止の解除」に読める文を作る。
- 設計照合 test・traceability の検査を red にする変更を入れる。D-111・D-117 の段を書き換える。lane 2・lane 3 の中身を本 lane で決める。
- 公開 repo に店の運用の細部・私的な事情・金額・商品名・JAN・部門名・発言の原文を書く。

### 非目的

- code・migration・bindings・test の変更（runtime の lane）。
- 売上の SD からの一括取込み、EJ の控えの閲覧、日報の紙と意味づけ（lane 2・lane 3）。日報の同日追加の利用者確認（D-071 の日報の側）。
- preview → commit の 2 段、`completed_partial`、商品ごとの在庫連動の切替（`pos_stock_sync`）を外すこと（owner が決めていない。今のまま残す）。
- 共有 JAN の商品群の扱い（Q5、店内バーコードの lane）。在庫連動の外に置く今の規則を引き継ぐだけ。
- `docs/Plans.md` の更新（D-097。closeout で「次の行動」の時点証拠 ADR の記述を直す）、現行の実装を写す図（`docs/diagrams/`）、archive。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

設計を含む変更なので、棚卸しの期間と普段の日の操作列を置く。本 lane は docs だけで、表の通常運用は後続の runtime（本 lane の runtime の 2 つの配布の単位と、Z004 を SD から読む経路〈D-111 の runtime・lane 2〉）の後に成り立つ。**この文書を完了できる**（設計正本がそろう）ことと、**通常運用を達成できる**（店で計数と取込みが動く）ことは別で、後者は本 lane では未達。各行の結果は ADR D3 の窓の判定表から導いた（round 1 の是正、Review Response 参照）。「既定の計数」は「数を保存」、「fresh な計数」は「入れ直す」。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 棚卸しの期間の営業中（日付 D）。棚卸しは開始済み | 商品を数えて「数を保存」（既定の計数、日付は既定の今日） | 在庫 = 数えた数 + 計数に入っていない記録。D の全ての精算と D の日付の入出庫は決まらない | — | ADR D3・D5 |
| 同じ日の締め（D の 1 回目の精算） | 精算 → SD を差す → Z004 を取り込む | 計数を持つ全ての商品が求め直され、取込みの結果に「数え直しが必要な商品」の一覧（この取込みで求め直した商品のうち要再確認の商品。D の精算でその商品が売れた・戻った〈個数 ≠ 0 か金額 ≠ 0〉、または D の日付の入出庫があった商品）。在庫は範囲で出る | — | ADR D3。Z004 を SD から読むのは D-111 の runtime・lane 2（それまではファイルを選ぶ経路） |
| 一覧がある | 棚で見て一覧から「入れ直す」（fresh な計数） | 先に保存した計数を置き換え、取り込み済みの精算と保存の前に入れた入出庫を含む数になり、要再確認が解ける。今日のこれからの精算（2 回目）と、保存の後に今日の日付で入れる入出庫だけが決まらない | — | ADR D3・D5 |
| 締めの取込みの後に客へ売り、その日にもう一度精算した | 2 回目・3 回目の精算を取り込む | fresh な計数にとって同じ日の n より後の全ての精算が決まらない精算で、売れた商品は要再確認（3 回目も二重に引かない） | 入れ直し | 店の回答（TD-279）。その日にもう一度精算しない日は受け入れた穴（A1） |
| 翌日の営業中に入れ直す | 一覧から「入れ直す」（今日は D と違う） | 今日の fresh な計数。今日の精算だけが決まらない（今日売れれば今日の分だけまた要再確認） | — | ADR D3 |
| 誤って入れた数を直す | 商品の「入れ直す」 | 先に保存した計数（同じ日の既定の計数を含む）を置き換える | — | ADR D3・D5 |
| 紙やメモに数えて翌日に入れる | 「前の日の分を入れる」で日付 D を選んで保存（既定の計数） | D の全ての精算と D の日付の入出庫が決まらない。それより後の計数（D3 の「計数どうし」）が既にあれば「後の数が既にあり、そちらが効きます」 | — | ADR D5。日付の選び忘れは受け入れた穴（R6） |
| 同じ日に紙の古い数を後から入れた（棚の数を入れ直した後） | 「前の日の分」ではなく今日の日付で「数を保存」 | fresh な計数の後に保存した今日の既定の計数は fresh と重なり、値が違えば要再確認（衝突）。写しは小さい方の数を土台にする | 入れ直す | ADR D3 の「計数どうし」 |
| 休業日（その日に精算が無い日）に数えた | 「数を保存」 | 次の精算日の精算の全体が決まらない精算になる（過剰側、H4）。取り込んだ後の入れ直しで解ける | 次の精算の取込み | ADR D3 |
| 精算の欠けがある（取り込み済み 100・102 が D、101 が未取込み） | 「棚卸しを完了」 | D の計数があれば `settlement_pending` で拒まれる（d_100 ≤ D ≤ d_102） | 101 を取り込む | ADR D6 の 1 |
| 大晦日 | 締め → 取込み → 入れ直し → 「棚卸しを完了」 | 未取込みの精算・要再確認・未計数があれば一覧で拒まれる。未計数は「0 にする」（選んだ商品・まとめて）。確認の文で「今日の最後の精算を取り込みましたか」を確かめる。全て片付くと評価額（総額）が固定される | 締めが翌日なら完了も翌日 | ADR D6（最後の精算の取込み忘れは H2） |
| 年明けに 12/31 の納品書を入れる（12/31 に締めの取込みの後で「入れ直す」をして完了した商品） | 入庫を日付 12/31 で保存 | fresh な計数の後に保存した同じ日付の入庫なので決まらず、要再確認になり在庫は範囲で出る。完了した評価額は変わらず、棚卸しの記録に差が「要再確認」と範囲で出る（確定の差の合計には入らない） | 入れ直し | ADR D3・D6（TD-296、closure の是正） |
| 正常な行が全て 0 の Z004（その精算で PLU の売上が無い） | 取り込む | 売上も在庫の変動も無いが、精算として記録され、欠けにも決まらない精算の判定にも入る | — | ADR D4 |
| Z004 の取込みで精算回数が飛んだ | 取り込む | 「精算 n が取り込まれていません」。後から n を取り込めば、計数を持つ全ての商品が求め直される | — | ADR D4・D2 |
| 精算回数の無い file・同じ精算の別の資料・レジの日付か番号が合わない file を選んだ | preview | 拒まれ、理由が出る | 正しい file を選び直す | ADR D4 |
| 取込みを取り消した | 取消 | 売上と在庫の変動が無効になり、計数を持つ全ての商品とその取込みの商品が求め直され、要再確認の一覧が出る | 正しい file を取り込み直す | ADR D7 |
| 要再確認の商品の在庫連動を切った | 商品フォームで在庫連動を off | 連動中に取り込んだ売上の movement は残り、要再確認と範囲も残る（取込みのときの連動で判定する）。それより後の取込みだけが売上のみになる | 入れ直し | ADR D3・D8（round 2 の是正） |
| 取込みがまだ 1 件も無い（導入の直後） | 棚を見て「入れ直す」→ 同じ日に売れる → その日の精算を取り込む | fresh な計数の精算のカーソルは「取込みなし」（NULL）なので、その精算は決まらない精算になり要再確認（精算回数が 0 でも入っているとみなさない） | 入れ直し | ADR D3（round 2 の是正） |
| 色違いで同じ JAN の商品がスキャンで売れた | Z004 を取り込む | 売上だけが記録され（先頭の商品）、どの候補の在庫も動かない。在庫連動にしようとすると「同じ JAN の商品（{商品コード}）があるため、在庫連動にできません」（`validation`） | 店内コードの lane | ADR D4（TD-296） |
| 連動しない商品で、計数も棚卸しも無い | 商品フォームで在庫連動を on → 拒否の案内から「今の数を入れる」 | 棚卸しの画面の「今の数を入れる」が商品を受けて開き、棚を見た数を fresh な計数として保存できる（棚卸し・既存の計数・要再確認の有無によらない）。その後に on が通る | — | 73・51（round 3 の一括是正） |
| 連動していなかった商品を在庫連動にする（その間に Z004 の売上がある） | 商品フォームで在庫連動を on | 売上が効く計数に確実に入っていなければ（入っていない・決まらない）「連動しない間の売上があります。今の数を入れてから在庫連動にしてください」で拒まれる | 棚を見て「入れ直す」→ on | ADR D8（round 2 の是正） |

## Scope

本 lane は docs だけ。下の file の、下の節を書き換えた（全部 plan-first の commit）。

1. 新しい ADR（SPEC-STK-CNT-D1〜D12）: [ADR](../../adr/2026-10-16-stocktake-count-placement.md)。横断の約束の表、並びの鍵、求め直し、要再確認、Z004 の識別・順・欠け、計数の入力、完了と評価額、取消、在庫連動、保存の形、移行、EJ、停止 ADR との関係。[ADR の README](../../adr/README.md) の索引。
2. 旧 ADR と停止 ADR: [時点証拠 ADR](../../adr/2026-09-18-stocktake-time-evidence.md) の Status を superseded にした（本文は記録として残す）。[停止 ADR](../../adr/2026-09-23-legacy-stocktake-z004-write-stop.md) の Status に改訂の段を足し、Context・SPEC-STOP-D3・D6・Revisit の「⑤」の指す単位を ADR SPEC-STK-CNT-D12 に合わせた（停止の範囲・error・部分的な解除の禁止は変えない）。
3. decision-log: D-118 を足し、D-051・D-071・D-109 に Superseded in part、D-116 に Superseded の行を足した。D-111・D-117 の段は変えない。
4. db-design の「時点証拠契約」を「計数の置き場と在庫の求め直しの契約」に置き換えた: [tracking](../../db-design/tracking-system-tables.md)（並びの鍵・`stock_counts`・求め直しと写し・棚卸し・移行。§14 の `stock_after`、§16-17 の `system_stock` の時点〈旧 SPEC-STK-TIME-D8-L1 の置換え〉・`actual_count`・ケース 3）、[pos-tables](../../db-design/pos-tables.md)（`csv_imports` の `machine_no`・`settlement_no`。§12 のケース 2・3 と Stage 3 の冪等性の表示、§12c の NEL の文〈P-07〉）、[master-tables](../../db-design/master-tables.md)（写しの列、在庫連動・共有 JAN）、[transaction-tables](../../db-design/transaction-tables.md)（入出庫の TX と並びの鍵）、[DB_DESIGN](../../DB_DESIGN.md) の入口。
5. function-design の同名の節: [20](../../function-design/20-io-product-repo.md)（と §2 の `system_stock` の型・棚卸しの補正の表示の段）、[21](../../function-design/21-io-inventory-repo.md)、[23](../../function-design/23-io-z004-parser.md)（「精算の識別メタ」に改名、0/0 行の保持を外す、NEL の表の行〈P-07〉）、[24](../../function-design/24-io-csv-import-repo.md)、[30](../../function-design/30-biz-product-service.md)、[31](../../function-design/31-biz-inventory-service.md)、[32](../../function-design/32-biz-csv-import-service.md)、[35](../../function-design/35-biz-stocktake-service.md)（と §20.2・§20.4・§20.5a・§20.6a・§20.8 の新方式への言及）、[36](../../function-design/36-biz-integrity-check.md)、[40](../../function-design/40-cmd-product.md)、[41](../../function-design/41-cmd-pos.md)、[42](../../function-design/42-cmd-sales-stocktake.md)、[43](../../function-design/43-cmd-settings-log.md)、[51](../../function-design/51-ui-product-form.md)（と UI-01b-D22 の `ej_unverified` の分岐）、[55](../../function-design/55-ui-csv-import.md)、[60](../../function-design/60-ui-product-import.md)、[65](../../function-design/65-inventory-record-traceability.md)（と slice 4c の注記）、[73](../../function-design/73-ui-stocktake.md)。新しい注記: [58](../../function-design/58-ui-stock-inquiry.md)・[66](../../function-design/66-ui-stock-movements.md)・[50](../../function-design/50-ui-product-list.md)・[75](../../function-design/75-ui-integrity-check.md)（round 2 の是正、Review Response 参照）。旧 ADR を live に引く所の訂正: [22](../../function-design/22-mnt-migration.md)（vU・vC の時点証拠の migration の段）、[29E](../../function-design/29-io-ej-parser.md)（入力ドキュメント、IO-08.9・IO-08.10 の冒頭に D-118 の注記、P-05・P-06）、[33](../../function-design/33-biz-plu-export-service.md)（§16.3 の占有の照合と全数読取りの段）、[37](../../function-design/37-biz-daily-report-import-service.md)（§37.9 の手順 2〈P-05〉と Z004・EJ への引継ぎ）。
6. 索引と architecture: [ARCHITECTURE](../../ARCHITECTURE.md)・[FUNCTION_DESIGN](../../FUNCTION_DESIGN.md)・[SCREEN_DESIGN](../../SCREEN_DESIGN.md)・[UI_TECH_STACK](../../UI_TECH_STACK.md) の入口の節、[biz](../../architecture/biz-task-specs.md)・[cmd](../../architecture/cmd-task-specs.md)・[io](../../architecture/io-task-specs.md)（と IO-08 の理由）・[mnt](../../architecture/mnt-task-specs.md)・[ui](../../architecture/ui-task-specs.md) の task spec の入口の節。
7. 事実の食い違いの訂正（[project-memory](../../project-memory.md)）: X-10（`NEL`）、X-06（日報の紙は税理士への提出と控えを兼ねる）、X-12（レジ本体だけで精算できた）、X-11（今の parser の読み方）、X-03（入力の仕方の回答と観察の食い違いを、設計が依らないこととして記録）、EJ の対応づけの段に D-118 の注記。P-05 は 37 §37.9 と 29E、P-06（X-09）は 29E、P-07（X-10）は pos-tables・23・project-memory、X-01 は P-05 と同じ。P-08（X-08）と X-07 は D-117 で直っている（`rg -c '配分変更の設定は取説に見当たらない' docs/function-design/25-io-plu-formatter.md` が 0、`docs/project-memory.md` の 10 枠の行は owner の書換えの記述）。X-02・X-04・X-05・X-13〜X-18 は repo 外の資料どうしの食い違いで、tracked の正本に当たる文が無い。
8. backlog: 「棚卸しの数の置き場と在庫数の求め直しの runtime」（2 つの配布の単位、DB の作り直しの承認、test helper の直し）、「売上の SD からの一括取込みと EJ の控えの閲覧」（lane 2）、「日報の紙と意味づけ」（lane 3）を足し、「実測とPOS系列の対応を取得・保存する」と STK-1・STK-2 の申し送りに D-118 で置き換えた行を足した。

発注の 3 つの候補（repo 外の回答台帳 TD-295）の扱い:

- 整合性チェックと修復（BIZ-07・UI-13）: **本 lane で片付く**。36 の新契約で「写しと求め直しの比較、補正は求め直しを写しへ書く」に縮めた（D-051 の Superseded in part）。比べる項目は数・要再確認の印・範囲の 4 つで、UI-13 の差異の一覧・確認・補正の結果と操作ログに違う項目と 4 項目の old / new を出す内容を 75 の注記に置いた（round 2 の是正、Review Response 参照。見た目の細部は runtime の lane）。
- 同日の追加取込みの守り（D-071）: **Z004 の側は本 lane で片付く**（精算回数の照合で利用者確認を置き換えた。32・41・55・pos-tables）。**日報の側は片付かない**: 日報の束の同日追加の確認は、SD から読む束の精算回数の照合（BIZ-08-D4）で置き換えられるかを、SD からの一括取込みと一緒に決める方が確か（ファイルを選ぶ経路の精算回数の無い束の扱いが残る）。置き場は lane 2（backlog「売上の SD からの一括取込みと EJ の控えの閲覧」）。
- 日報の取込みと意味づけ（D-022・D-025・D-096、O-18）: **本 lane では片付かない**。在庫を動かさない日報の意味づけは、owner の Q10（今の Excel の並び、1 日 1 枚）の紙の形と一緒に決める。置き場は lane 3（backlog「日報の紙と意味づけ」）。

対象を使う呼出し側・隣接 test・helper / mock・生成物（runtime の lane が触る。本 lane は変えない）: `src-tauri/src/biz/inventory_service/common.rs`（`apply_stock_change`）と入出庫 4 業務、`src-tauri/src/biz/csv_import_service/commit.rs`・`rollback.rs`・`time_evidence.rs`（撤去）・`tests/legacy_stop_tests.rs`、`src-tauri/src/biz/stocktake_service.rs`・`stocktake_service/time_evidence.rs`（撤去）・`time_evidence_tests.rs`（撤去）、`src-tauri/src/biz/integrity_service.rs`、`src-tauri/src/biz/product_service.rs`（初期在庫・共有 JAN・在庫連動を始める検査）、`src-tauri/src/db/inventory_repo.rs`・`sales_repo.rs`・`stocktake_repo.rs`・`product_repo.rs`・`schema_time_evidence.rs`（撤去）・`migration.rs`、`src-tauri/src/cmd/stocktake_cmd.rs`・`csv_import_cmd.rs`・`product_cmd.rs`・`cmd/mod.rs`（`CmdErrorKind` と回復型）、`src-tauri/src/biz/csv_import_service/tests/cross_feature_tests.rs`（診断 2 本）、`scripts/probes/stocktake_time_model.py`（撤去）、`src/features/stocktake/`・`src/features/csv-import/` の Z004 の flow・商品フォーム・在庫照会・在庫変動、`src/lib/bindings.ts`、D-052 の SSOT（`src/lib/invalidation-contract.ts`）、商品を作る test helper 8 個（repo 外の Q8 の調査 §4.4。runtime の lane が `rg` で数え直す）。

## Non-scope

- code・migration・bindings・test（runtime の lane。backlog「棚卸しの数の置き場と在庫数の求め直しの runtime」）。
- lane 2（売上の SD からの一括取込み・EJ の控えの閲覧・日報の同日追加の守り）、lane 3（日報の紙と意味づけ）。
- preview → commit の 2 段、`completed_partial`（owner の決定が無い）、`pos_stock_sync` の切替の撤去（owner が TD-298 で切替を残すと確かめた）。
- 共有 JAN の商品群（Q5、店内バーコードの lane）。
- D-111・D-117 の段、PLU の書出し（33 は旧 ADR を引く 1 段だけ直した）。
- `docs/Plans.md`（closeout）、`docs/diagrams/`（現行の実装の図）、`docs/research/`、archive、`docs/project-profile.md`・`docs/code_review.md`（workflow の file。NEL の文は parser の振舞いとして今も正しい）。
- 現行の実装を記す本文（35 §20.3〜§20.5 の force_fill、73 の UI-10-D4・F5、42 §22 の `complete_stocktake` の force_fill、20 の force_fill の読取り、architecture の BIZ-06・CMD-10 の表、FUNCTION_DESIGN・SCREEN_DESIGN の UI-10 の行）。各文書の新契約の節が「runtime で置き換える」と書く。

## Acceptance Criteria

数は開始 HEAD `ea15360d`（`git archive ea15360d docs` を展開した版）と起票後の版を同じ数え方（`rg -c` はその file の中で式に当たる行の数、`rg -l … | wc -l` は file の数）で測った。測った script の式は各 AC に書いた。

- AC1（新しい ADR と決定の記録）: `rg -c '^### SPEC-STK-CNT-D' docs/adr/2026-10-16-stocktake-count-placement.md` が 12（開始 0。file が無い）。`rg -c '^## D-118: ' docs/decision-log.md` が 1（開始 0）。`rg -c '^superseded — 2026-10-16' docs/adr/2026-09-18-stocktake-time-evidence.md` が 1（開始 0）。`rg -c '^- Superseded (in part )?by: D-118' docs/decision-log.md` が 4（D-051・D-071・D-109・D-116。開始 0）。
- AC2（旧 ADR の契約の節の置換え）: `rg -l '^#+ 時点証拠契約' docs --glob '!docs/archive/**' | wc -l` が 0（開始 32）。`rg -l '^#+ 計数の置き場と在庫の求め直しの契約' docs --glob '!docs/archive/**' | wc -l` が 31（開始 0。23 は「精算の識別メタ」に改名した）。
- AC3（停止 ADR）: `rg -c '部分的な解除（例: 開始だけ許可、同日 import がなければ commit を許可）をしない' docs/adr/2026-09-23-legacy-stocktake-z004-write-stop.md` が 1（開始 1。SPEC-STOP-D6 の本文が変わっていない）。`rg -c 'SPEC-STK-CNT-D12' docs/adr/2026-09-23-legacy-stocktake-z004-write-stop.md` が 3（開始 0）。`git diff ea15360d HEAD -- docs/adr/2026-09-23-legacy-stocktake-z004-write-stop.md` の削除行に `### SPEC-STOP-D` の見出しが無い。
- AC4（旧 ADR の語が live に残らない）: `rg -n 'ej_unverified|get_pos_stock_readiness|PosStockReadiness' docs/function-design docs/db-design docs/architecture | rg -v '旧 ADR|D-118' | wc -l` が 0（開始 25）。`rg -n 'source_cursor|stock_revision|pos_sync_disabled_revision|ledger_cursor|rollback_compensation' docs/function-design docs/db-design docs/architecture | rg -v '旧 ADR|D-118' | wc -l` が 0（開始 33）。
- AC5（中心の契約の所在）: `rg -c 'recompute_stock\(tx: &Transaction, product_code: &str\)' docs/function-design/31-biz-inventory-service.md` が 1、`rg -c '^\| stock_counts（新設）' docs/db-design/tracking-system-tables.md` が 1、`rg -c '精算回数の無い file は取り込めません' docs/function-design/32-biz-csv-import-service.md` が 1、`rg -c '^\| \`(settlement_pending|recheck|uncounted)\` \|' docs/function-design/35-biz-stocktake-service.md` が 3（開始はどれも 0）。ADR の「横断の約束の表」が計数・入出庫・Z004 の確定・商品の登録・整合性の 5 行を持つ。
- AC6（前提の訂正）: `rg -c 'line separators may include' docs/project-memory.md`・`rg -c 'or external/tax submissions' docs/project-memory.md`・`rg -c 'ECR\+ の終了後に精算を続けられるかは未確認' docs/project-memory.md`・`rg -c '最大 31 日の遅れ）を覆う値' docs/function-design/37-biz-daily-report-import-service.md`・`rg -c '取扱説明書の「精算ごとにリセット」と一致しない。意味は' docs/function-design/29-io-ej-parser.md`・`rg -c 'Z002はNEL / raw 0x85 を含む可能性があるため' docs/db-design/pos-tables.md` がどれも 0（開始はどれも 1）。
- AC7（backlog）: `rg -c '^- \*\*棚卸しの数の置き場と在庫数の求め直しの runtime\*\*' docs/backlog.md`・`rg -c '^- \*\*売上の SD からの一括取込みと EJ の控えの閲覧\*\*' docs/backlog.md`・`rg -c '^- \*\*日報の紙と意味づけ\*\*' docs/backlog.md` がどれも 1（開始はどれも 0）。
- AC8（docs だけ、D-111・D-117 の段）: `git diff --name-only ea15360d HEAD -- src src-tauri scripts package.json package-lock.json` の出力が空。`diff <(awk '/^## D-111/,/^## D-113/' "$OLD/docs/decision-log.md") <(awk '/^## D-111/,/^## D-113/' docs/decision-log.md)` と `diff <(awk '/^## D-117/,/^## D-118/' "$OLD/docs/decision-log.md") <(awk '/^## D-117/,/^## D-118/' docs/decision-log.md)` は、後者の末尾に `## D-118` の見出しの行が増えるだけ（`$OLD` は開始 HEAD の展開先。開始の版には D-118 が無く awk が file の終わりまで読む）。
- AC10（round 1 の是正: 窓の原理と帰結。開始は `ef5debeb`）: `rg -c '^\*\*原理\*\*: 記録 R と計数 C' docs/adr/2026-10-16-stocktake-count-placement.md` が 1（開始 0）。`rg -c 'seen_movement_id' docs/db-design/tracking-system-tables.md` が 1（開始 0）。`rg -c 'd_a ≤ X ≤ d_b' docs/function-design/35-biz-stocktake-service.md` が 1（開始 0）。`rg -c 'derive_stock\(conn: &Connection, product_code: &str\)' docs/function-design/31-biz-inventory-service.md` が 1（開始 0）。`rg -l 'shared_jan_unresolved' docs --glob '!docs/archive/**' --glob '!docs/plans/**' | wc -l` が 0（開始 2: 40・51。同じ範囲で `ef5debeb` を数えた）。`rg -c '^- 同じ鍵の中の順（保存の連番）は、D3 の判定には使わない' docs/adr/2026-10-16-stocktake-count-placement.md` と `rg -c '精算 n\+1 の精算日が d_n と同じ日' docs/adr/2026-10-16-stocktake-count-placement.md` がどちらも 0（開始はどちらも 1）。
- AC11（round 2 の是正。開始は `6e867fb9`）: `rg -c 'stock_linked' docs/db-design/pos-tables.md` が 1（開始 0）。`rg -c 'stock_linked=1' docs/adr/2026-10-16-stocktake-count-placement.md` が 1（開始 0）。`rg -c 'cutoff_movement_id' docs/function-design/35-biz-stocktake-service.md` が 0（開始 2）。`rg -c 'differing_fields' docs/function-design/36-biz-integrity-check.md` が 1（開始 0）。`rg -c 'n が NULL でなく k ≤ n' docs/adr/2026-10-16-stocktake-count-placement.md` が 1（開始 0）。`rg -c '在庫連動でない商品は Z004 の movement を持たないので、理由 1 を判定しない' docs/function-design/31-biz-inventory-service.md` が 0（開始 1）。
- AC12（round 3 の一括是正。開始は `49811f9c`）: `rg -c "IS 'csv_import'" docs/db-design/tracking-system-tables.md` が 1（開始 0）。`rg -c 'この明細は別の商品です' docs/function-design/35-biz-stocktake-service.md` が 1（開始 0）。`rg -c '精算日が X より前の取り込み済みの精算が無い' docs/function-design/35-biz-stocktake-service.md` が 1（開始 0）。`rg -c '確定の差の合計に入れない' docs/function-design/35-biz-stocktake-service.md` が 1（開始 0）。
- AC9（検査）: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan` が ERROR 0・WARN 0。`cd src-tauri && cargo run --bin generate_traceability -- --check` が OK（ERROR 0・WARN 0）。`cd src-tauri && cargo test --test design_compliance_test` が pass。

## Design Readiness

設計の完了条件は `docs/DEV_WORKFLOW.md` Design Phase Rules の Design completion criteria。

- 引用する設計正本（節まで）: [ADR](../../adr/2026-10-16-stocktake-count-placement.md) SPEC-STK-CNT-D1〜D12 と「横断の約束の表」／[停止 ADR](../../adr/2026-09-23-legacy-stocktake-z004-write-stop.md) SPEC-STOP-D1〜D6 と改訂の段／tracking・pos-tables・master-tables・transaction-tables の「計数の置き場と在庫の求め直しの契約」、tracking §14・§16-17／31・32・35・36・30・20・21・23・24・40・41・42・43・51・55・58・60・65・66・73 の同名の節／35 §20.5a（SPEC-STK-VAL-D1〜D6、変えない）／D-051・D-070・D-071・D-109・D-111・D-113・D-116・D-117・D-118。
- 必要な設計成果物（Design artifact selection の当たる行）: BIZ・IO・CMD の振舞い = updated in this PR（31・32・35・36・30・20・21・23・24）／command・DTO = updated in this PR（40・41・42。wire の変更は runtime の lane、下の Boundary / Wire Contract）／表・migration = updated in this PR（tracking・pos-tables・master-tables・transaction-tables・DB_DESIGN。migration の番号と実装は runtime）／operator workflow・文言 = updated in this PR（73・55・51・58・60・65・66・SCREEN_DESIGN・UI_TECH_STACK）／POS CSV の取り込む条件 = updated in this PR（32・23・pos-tables）／durable な判断 = updated in this PR（新しい ADR・D-118・D-051・D-071・D-109・D-116 の行・停止 ADR の改訂）。
- plan にしかない durable な判断の昇格先: 決めたことは全て新しい ADR・D-118・各文書の新契約に置いた。packet にだけあるのは、規模と手法の数え、3 lane の分け方の提案、runtime への file の申し送り（Scope の末尾）、AC の数。
- 前提・制約と、延期した design gap の follow-up:
  - owner に確かめた点: **Q-A** Q9 = c（「ファイルの経路は残し、精算回数の無い file は受けない」）は、商品ごとの在庫連動の切替（`pos_stock_sync`）を外す意味を含むか → owner が 2026-10-17 に a（含まない。切替を残す）と確かめた（repo 外の回答台帳 TD-298）。ADR D8 に反映した。完了の後の要再確認の評価額は、owner が TD-296 で「評価額を変えず差を示す」と決めた（ADR D6・35・65・73・tracking）。
  - 延期: lane 2・lane 3（backlog）、共有 JAN（Q5、店内バーコードの lane）、runtime（backlog。2 つの配布の単位）。延期が安全な理由: どれも今の code の振舞いを変えず、本 lane の決定と矛盾しない（lane 2 は本 lane の Z004 の契約を使い、lane 3 は在庫を動かさない日報だけを扱う）。今の店の棚卸しと Z004 の取込みは停止 ADR のとおり止まったまま（本 lane は停止を解かない）。
  - 外部前提: 下の Contract Probe。
- 絶対保証（cannot happen / always happens）の例外と escape hatch の自己点検:
  - 「写しを書くのは求め直しの関数だけ」: 例外は新しい商品の INSERT の初期数量で、同じ TX の初期在庫の movement から求め直した値と等しいこと（master-tables）を条件にした。`fix_integrity` も求め直しの結果を書く。migration の写しの書込みも求め直しで行う（tracking の移行）。
  - 「STK-1・STK-2・取消の補償は構造上起きない」: 計数が movement を作らず、確定が在庫を動かさず、取消は無効にして求め直すことに依る。完了の評価数量は完了の時点の写しで、計数の時点の値ではない（owner 2026-09-16 の「いつ数えても年末時点へ繰り越す」）。
  - 「黙ってずれない」: 例外は ADR の Consequences の受け入れた穴（締めの後の販売（A1。その日にもう一度精算しない場合）、日付のずれ、同じ精算で個数・金額とも 0 に打ち消し合う売りと戻り、共有 JAN（在庫連動の外）、部門キーの販売、日付の選び忘れ、入出庫を事象の前に入力した場合、その日の最後の精算の取込み忘れ（H2））で、owner が受け入れた（TD-266・TD-267）。round 3 の一括是正（Review Response 参照）で ADR・D-118・本節の列挙を揃えた。
  - 「精算日と精算回数の順の食い違いは取り込まない」: レジの reset・入替・時計の戻りで、その後の Z004 が取り込めなくなる（止まる側）。Revisit で切替の前に決め直す。
- 判定（ready / not ready）と理由: ready（plan-draft に進める）。理由: 設計の出力が上の正本と D-118 にあり、owner の判断事項は TD-261・TD-263〜TD-268 で決まり、素描の未決 3 つは owner の既存の原則から正本で決め、Q-A は TD-298 で確かめた。round 1 の是正で、計数が記録を含むかを 1 つの原理（ADR D3 の窓）から導き直した（Review Response 参照）。共有 JAN・停止の解除・入れ直しを省く操作・範囲の表示・完了の後の差は owner の決定 TD-296 で決まった。

## Registration / Generation Obligations

本 lane: function-design の file の新設・改名・削除は無い（`docs/adr/2026-10-16-stocktake-count-placement.md` は ADR で、ADR の README に索引の行を足した）。REQ の文と担当の task は変えていないので traceability の再生成は要らない（AC9 の `--check`）。runtime の lane の義務: 新しい command（`save_stock_count`・`preview_stock_count_placement`・`zero_uncounted_items`・`list_recheck_products`）の specta の登録と `cargo run --bin generate_bindings`、旧 `update_count` の公開登録の削除、`complete_stocktake` の入力から `force_fill` を外す、`commit_csv_import` の入力から `additionalImportConfirmed` を外す、CmdErrorKind の `stocktake_guard` と回復型、migration の registry（`src-tauri/src/db/migration.rs`）と 22 への登録、新しい BIZ の module（`biz::stock_ledger`）の pub fn を 31 の設計の code block に書く（`src-tauri/tests/design_compliance_test.rs` の突合せ）、撤去する `#[cfg(test)]` の時点証拠の code と旧本体、D-052 の Contract 行と SSOT と独立 oracle の同期。

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | Which concepts belong to replaceable external adapters, and which concepts are stable app-core contracts? | Architecture, function design, decision-log, Plan Packet | 当たる。精算の識別メタの抽出と file の形は adapter（IO-02）、並びの鍵・求め直し・要再確認は core（BIZ-02・03・06）。レジが変わっても core は「精算の順と日付」があれば残り、machine_no ごとの軸が要るなら Revisit | ADR D1・D4、Revisit |
| Fact check / design decision split | Which claims are observed facts from hardware/tool/files, and which are app decisions that need source-doc promotion? | Investigation doc, source design docs, decision-log | 当たる。精算回数の連続・日付をまたがない精算・CRLF だけ・レジ本体だけの精算は repo 外の事実の一覧の観測（確かめた）、頻度（数え終わった商品が売れて直すのは 1 日に数品、締めの後の販売の量）は店主の回答か未計測で、設計はそれを受け入れた穴として扱う。CV17 の推定は根拠に使っていない | ADR の Context・Consequences、Contract Probe |
| Lifecycle / retry | What happens before, during, after, and after failure for import/export, duplicate input, rollback, retry, cancellation, and re-run? | Function design, DB design, UI design, Test Matrix | 当たる。取込みの拒否（前・中）、取消と取り込み直し、同じ精算の別版、欠けの後追い、後入力、日付の変わり目、完了の後の記録、migration の失敗（全体を戻す） | ADR D4・D5・D6・D7・D10、Matrix |
| Operator workflow | What does the operator do in the real sequence across app, external tool, media, print/export, backup, and recovery? | Screen/UI design, function design, Plan Packet manual checks | 当たる。数えたら保存・締めの取込みと入れ直し・大晦日の完了（上の Ordinary Operation）。CV17 と SD の操作は D-111・lane 2 のまま | 73・55、Ordinary Operation |
| Replacement path | If the external system changes, which files/modules/docs are replaced and which app-core contracts remain stable? | Architecture, function design, decision-log | 当たる。レジの入替・reset は D4 の取り込まない規則で止まり、切替の前に鍵の続け方を決める（Revisit）。EJ は在庫から外れたので、EJ の形の変化は在庫に効かない | ADR Revisit |
| Data safety / evidence | How can the claim be supported by anonymized shape/count/hash/procedure evidence without committing real store data? | Plan Packet Data Safety, investigation doc, review evidence | 当たる。tracked には事実の要旨と ID（台帳 TD-nnn）だけを書き、金額・商品名・JAN・部門名・発言の原文を書いていない | Data Safety |
| Reporting / accounting semantics | Are totals, summaries, item records, returns, corrections, and inventory movements modeled separately enough to avoid false business meaning? | DB design, function design, report design, Test Matrix | 当たる。計数（`stock_counts`）と movement と売上（`sale_records`）を分け、計数は movement を作らない。評価額は完了の時点の写しで固定し非遡及。売上の集計は変えない | tracking、35、65 |
| Manual verification | Which assertions cannot be proven by automated tests and require Windows native L3, external tool import, or real-device confirmation? | Plan Packet, Test Matrix, PR body | 当たる（runtime）。73 の (a)〜(e)、55 の拒否と知らせ、商品フォームの在庫連動の拒否。本 lane は docs だけで L3 は無い | Matrix の Manual、backlog の runtime の項目 |
| 環境・再現性 | 新設の環境依存（toolchain / CI runner / OS 差異等）を repo-pinned config で強制するか、明示的に defer するか | repo-pinned config, Plan Packet | 当たる（runtime、小）。CMD が渡す「PC の今日」はローカルの日付で、OS の時刻帯の設定に依る（店の PC は 1 台）。runtime の lane が取得の関数を 1 つに決める | 42 の新契約 |

## Boundary / Wire Contract

本 lane は wire を変えない（bindings も変えない）。後続の runtime の lane が変える wire の契約を正本に書いた。

- producer: CMD-10（`save_stock_count`・`preview_stock_count_placement`・`zero_uncounted_items`・`list_recheck_products`・`complete_stocktake`・`get_stocktake_items`・`get_stocktake_record`）、CMD-07（`parse_and_validate_csv`・`commit_csv_import`・`rollback_csv_import`）、CMD-01（商品の get / search / create / update）、共通の CmdError
- consumer: UI-10（73）、UI-07 の Z004 の flow（55）、UI-01b（51）、UI-06a（58）、UI-06c（66）、UI-01c（60）
- wire type: 新しい command 4 つの入出力（42）。`complete_stocktake` の入力から `force_fill` を外す。`commit_csv_import` の入力から `additionalImportConfirmed` を外す（Z004）。preview と `ImportResult` に `notices`、`ImportResult`・`RollbackResult` に `recheck_targets`（41）。商品の結果に `stock_recheck`・`stock_range_low`・`stock_range_high`（40）。CmdErrorKind に `stocktake_guard`、`StocktakeRecovery`（code 2 値。共有 JAN の拒否は `validation`）と `CountRecoveryTarget`（action 2 値・`block_reason` 3 値・`recount_reasons` 3 値）（40）
- internal type: `DerivedStock`（31）、`NewMovement` の並びの鍵（21）、`csv_imports` の `machine_no: String`・`settlement_no: i64`（24）
- precision/range: 計数の値は 0 以上で在庫の上限 `999999999` 以下（長さは cm）。範囲の 2 値と写しは在庫の上限の検査を通す（31）。精算回数は `i64` に収まる 10 進の整数だけ（23 の文字列を BIZ が整数にする）。日付は YYYY-MM-DD
- round-trip path: 計数の保存 → BIZ が `stock_counts` に INSERT → 求め直し → 写し → UI が商品・棚卸しの一覧を取り直す。Z004 の preview → 検査と知らせ → 確定 → movement（並びの鍵）→ 求め直し → 結果の要再確認の一覧 → 73 の入れ直し（`fresh=true`。保存の時点の精算回数と movement の id を残す）
- invalid input: 未来の日付の計数は `ValidationFailed`。負の数・上限超えは `ValidationFailed`。精算回数の無い file・別の machine_no・同じ精算の別版・精算の順の食い違いは `import_error`。完了の拒否は `stocktake_guard` の `completion_blocked`
- compatibility: 本番の DB は無い。棚卸しか Z004 の取込みを持つ開発・試験 DB では runtime の migration が止まり、作り直しは別の承認（ADR D10）。旧い UI と新しい backend の混在は無い（1 つの build）。停止中の入口は新しい経路と同じ配布の単位で開く（ADR D12）

## Test Plan

Test Design Matrix: [2026-10-16-stocktake-count-placement-design](test-matrices/2026-10-16-stocktake-count-placement-design.md)（runtime の lane が実装する test の設計。本 lane は docs の検査だけ）。

- targeted tests: 本 lane は `bash scripts/doc-consistency-check.sh` と `--target plan`、`cargo run --bin generate_traceability -- --check`、`cargo test --test design_compliance_test`（AC9）。
- negative tests: Matrix の Negative Paths（runtime）。
- compatibility checks: Matrix の Compatibility Checks（migration が止まる DB・止まらない DB で写しが SUM と等しい）。
- data safety checks: 本 lane の差分に実データ（JAN・商品名・金額・部門名・発言の原文）を入れない。
- main wiring/integration checks: runtime の lane（求め直しが全 writer から呼ばれる、停止の一括の解除、診断 2 本の PASS）。

## Review Focus

- Ordinary Operation の列が、正常な条件で目的（数えたら保存し、締めの後に数個を入れ直せば在庫が合う。黙ってずれない）を達成できるか。各行で、在庫が何になり、要再確認がいつ付いていつ解けるかが正本から 1 通りに読めるか。
- ADR D1 の並びの鍵と D3 の決まらない記録の規則で、計数・入出庫・精算の全ての組合せ（同じ日・前の日・休業日・同じ日の 2 回目の精算・後から届く古い精算・取消・後入力）に結果があるか。黙ってずれる組合せが、ADR の Consequences の受け入れた穴の外に無いか。
- 「写しを書くのは求め直しの関数だけ」が、入出庫 4 業務・Z004 の確定と取消・商品の登録と一括 import・整合性の補正・migration の全てで成り立つか（31・30・32・36・tracking の移行）。
- 停止 ADR の改訂が SPEC-STOP-D6 を変えていないか、「部分的な解除」に読める文が無いか（ADR D12 の (a) の単位は停止の入口に触れない）。
- 旧 ADR の契約が live に残っていないか（AC2・AC4 と、Matrix の sweep の残りの理由）。現行の実装を記す本文と新契約の節の関係が、runtime の lane の手順として 1 通りに読めるか。
- 訂正した前提の文が出典（事実の一覧の ID、店主の回答、現物の code）で裏づくか。確かめていないものを確かめたように書いていないか。
- 3 lane の分け方と、TD-295 の 3 つの候補の扱い（片付く・片付かないの理由と置き場）が妥当か。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| SPEC-STK-CNT-D1（並びの鍵） | ADR D1、tracking の「並びの鍵」、21・24・transaction-tables | runtime: migration、`db/inventory_repo.rs`・`csv_import_repo`、入出庫 4 業務が日付を渡す | runtime（Matrix の D1 の行） | 非対象 |
| SPEC-STK-CNT-D2（求め直しと写し、D-051 の Superseded in part） | ADR D2、tracking の「求め直しと写し」、31、master-tables | runtime: `biz/stock_ledger.rs`（新）、`apply_stock_change`、`update_stock_quantity` の呼出し元の一本化 | runtime（Matrix の D2 の行） | 非対象 |
| SPEC-STK-CNT-D3（要再確認） | ADR D3、31、35 の「読取り」、55・73 の文言 | runtime: `recompute_stock` の判定、一覧 | runtime（Matrix の D3 の行） | L3: 73 の (c) |
| SPEC-STK-CNT-D4（Z004 の識別・順・欠け・共有 JAN、D-071 の Superseded in part） | ADR D4、32 の表と知らせ、pos-tables、24、30、23 | runtime: `csv_import_service`、`product_service` | runtime（Matrix の D4 の行） | L3: 55 の拒否と知らせ |
| SPEC-STK-CNT-D5（計数の入力） | ADR D5、35「計数の保存」、42、73 | runtime: `stocktake_service`、`stocktake_cmd`、棚卸し画面 | runtime（Matrix の D5 の行） | L3: 73 の (a)(b)(e) |
| SPEC-STK-CNT-D6（完了と評価額、SPEC-STK-VAL は変えない） | ADR D6、35「完了」・§20.5a、tracking の「棚卸し」、73、65 | runtime: `complete_stocktake`、`zero_uncounted_items` | runtime（Matrix の D6 の行）と既存の §20.5a の test | L3: 73 の (d) |
| SPEC-STK-CNT-D7（取消） | ADR D7、32「取消」、41 | runtime: `rollback.rs` | runtime（Matrix の D7 の行） | 非対象 |
| SPEC-STK-CNT-D8（在庫連動の切替） | ADR D8、30、51、60 | runtime: `product_service` | runtime（Matrix の D8 の行） | L3: 商品フォームの拒否の表示 |
| SPEC-STK-CNT-D9（保存の形） | ADR D9、tracking・pos-tables・master-tables | runtime: migration | runtime（schema の CHECK の test） | 非対象 |
| SPEC-STK-CNT-D10（移行） | ADR D10、tracking の「移行」、22（作らない時点証拠の migration の注記）、mnt task | runtime: migration | runtime（Matrix の D10 の行） | DB の作り直しは別の承認（owner） |
| SPEC-STK-CNT-D11（EJ は在庫に使わない） | ADR D11、29E の注記、io task | runtime: 時点証拠の code の撤去 | runtime（撤去後に EJ parser の既存の test が通る） | 非対象 |
| SPEC-STK-CNT-D12 / SPEC-STOP-D6（停止の一括の解除） | ADR D12、停止 ADR の改訂の段・D6・Revisit、32 §15.0、35 §20.0、55・73 の停止の節 | runtime: 5 入口の停止の撤去、旧本体の撤去 | runtime（`legacy_stop_tests.rs` の撤去、診断 2 本の PASS） | L3: 73 と 55 の通しの操作 |
| BIZ-07 の突合（D-051 の Superseded in part） | 36 の新契約 | runtime: `integrity_service.rs` | runtime（Matrix の BIZ-07 の行） | 非対象 |
| 文書の誤りの訂正（P-05〜P-07、X-01・X-03・X-06・X-09〜X-12） | 37 §37.9 手順 2、29E、pos-tables、23、project-memory | 本 lane（docs） | AC6 | 非対象 |
| D-116 の Superseded・D-109 の Superseded in part | decision-log | 本 lane（docs）、runtime の撤去 | AC1 | 非対象 |

## Contract Probe

本 lane の設計が頼る外部の前提（レジ・持ち帰った資料の観測）。どれも repo 外の事実の一覧で確かめた（「確かめた」の行は独立の数え直し済み）。実機の通しの確認は runtime の lane の L3。

- P1 精算回数は machine_no ごとに減少・重複せず、飛びは SD の本数の差と一致する: 持ち帰った EJ と Z 帳票の全期間の手元集計（事実の一覧 F-057・F-062・F-069、確かめた）→ 成立。reset と再出力は資料に現れていない（F-070・F-073・F-199 は取説と回答）→ 未観測。設計は reset を検知して取り込まない（ADR D4）側に倒した。
- P2 精算は日付をまたがない時刻に行われる（精算日で手入力の置き場を決めてよい）: 全期間の精算の時刻の範囲（事実の一覧 `F-085`、確かめた）→ 成立。時計が戻ったのは 4 年で 1 回（事実の一覧 `F-079`、EJ の記録の日時の全期間の走査）→ 精算日と精算回数の順の食い違いは取り込まない（ADR D4）。
- P3 店のレジは 1 台（machine_no は 1 種類）: EJ の 4 桁の欄と Z004 のマシンNo. が全期間で 1 種類（F-056・X-14）→ 成立。別の machine_no は取り込まない（ADR D4）。
- P4 Z004 は精算ごとの差分で、精算の日時は持つが商品別の取引の時刻を持たない: F-017（確かめた）→ 成立（同じ精算の中の前後が決まらない根拠。精算の窓の根拠）。
- P5 1 商品の求め直しは書くたびに行っても遅くない: repo 外の Q8 の範囲の調査 §5 の合成データの実測（Python 3 の `sqlite3`、SQLite 3.51.3、WSL、メモリ上の DB、4,000 商品・10 年・1 日 60 件の記録で 1 商品 0.004 ms）→ 成立（WSL の合成データ）。店の Windows の PC での値は**未実測**（runtime の lane が測る）。

## Data Safety

- 本 lane の差分に、実データ（JAN・商品名・金額・部門名・個々の日付の売上）と店主・owner の発言の原文を入れない。
- tracked に置く POS の sample（Z004・日報・EJ）は合成の fixture だけ（owner の決定 TD-296）。runtime の lane の test もこれに従う（backlog の runtime の項目）。事実は要旨と repo 外の回答台帳の ID（TD-nnn）と事実の一覧の ID で引いた。
- repo 外のまま置くもの: 見直しの成果物（`.local/reports/redesign-audit-2026-10/`）、回答台帳（`.local/reports/store-premises/`）、持ち帰りデータ（`~/downloads/inventory-field-check/`。本 run は開いていない）。
- 合成だけ: runtime の test の fixture（Matrix）。

## Implementation Results

- 結果: 棚卸しの時点証拠と POS の取込みの見直し（repo 外の回答台帳 TD-258）で owner が採った結果のうち、棚卸しの数と売上の前後・在庫数の持ち方・Z004 の識別を設計正本へ置き換え、新しい ADR（SPEC-STK-CNT-D1〜D12）と D-118 に記録した。計数が記録を含むかは記録と計数の窓の重なり（ADR D3）の 1 つの原理で決め、在庫数は記録から求め直して写しを持つ形にし（D-051 の Superseded in part）、Z004 は精算回数で識別・順・欠けを決め、EJ を在庫に使わないことにした。時点証拠 ADR は superseded、D-116 は Superseded、D-071・D-109 は Superseded in part。停止 ADR は残し、停止の範囲と部分的な解除の禁止（SPEC-STOP-D6）は変えず、解除の段の名前だけを改めた。見直しの残りは 2 つの design lane（売上の SD からの一括取込みと EJ の控えの閲覧、日報の紙と意味づけ）に分けて backlog に置いた。本 lane は docs だけで、code・migration・bindings は変えていない（runtime は後続の lane）。
- 既存の保護: 実装は変えていないので、店の動作は今のまま（旧棚卸しと Z004 の業務 commit は停止 ADR のまま止まっている）。撤去する pub fn は無く、設計照合 test と traceability の検査は green のまま。
- 記録して閉じる食い違い（Final Review の Codex の P3〈C3〉）: 冒頭の「規模と手法」の触った正本の数（`git diff --name-only` の出力 45）と AC5 の「横断の約束の表」の行数（5 行）は、起票と round の是正の時点の数で、merge の版の実測は正本 47 file、表は 6 行。表の 1 行は round 1 の是正で在庫連動の切替の行を足したため、file の数は round の是正で触る file が増えたため。本文の数は書き換えず、ここに記録する。
- 未検証: Contract Probe の P5（1 商品の求め直しの速さ）の店の Windows の PC での値は、runtime の lane が測る。73 の (a)〜(e) と Z004 の取込みの preview の拒否と知らせは runtime の lane の L3。
- review・CI・merge（closeout、2026-10-17）: Plan Review は round 3（天井）までで、round 1 の後に相談役が窓の重なりの原理を出して ADR D3 を書き直し、round 3 の後に owner の決定で天井を越えて closure 2 回で確かめた（TD-299・TD-300）。Final Review は broad 1 回と是正の後の closure（下の Review Response の Closeout の段落）。helper の record は review = pass。hosted の `Merge gate` は success で、owner の Ready・merge の承認（TD-302）の後に helper 経由の squash merge。PR: [#167](https://github.com/higekiri-w/inventory-system-desktop/pull/167)
- 後続: `docs/backlog.md` の「棚卸しの数の置き場と在庫数の求め直しの runtime」（Final Review の closure の P3 の follow-up を含む）、「売上の SD からの一括取込みと EJ の控えの閲覧」（本 lane の後）、「日報の紙と意味づけ」（並走できる）。DB の作り直しを 1 回にするため、原価の lane（D-119）を runtime の最初の配布の単位より先に入れる。

## Review Response

Fill after review.
先行 round の結果・評価（判定・件数・採否・reviewer の意見）はこの節にだけ書く。前半の節と遷移記録には「round N の是正（Review Response 参照）」だけを書く（独立 review は `## Review Response` より前だけを読む）。

### Plan Review round 1（2026-10-17、head `ef5debeb`）

- 判定: Claude 側 fresh Opus 5.5 = reject（P1 1・P2 5・P3 6）、Codex GPT-6 Astra = reject（P1 4・P2 7・P3 1）。
- 是正の方針（Coordinator の裁定。owner の方針 TD-296 の 5a「個別の穴埋めでなく、1 つの原理から導き直す」）: findings の多くが「計数がどの記録を含むと分かるか」と「どの商品を求め直すか」の定め方に集まったので、相談役（Fable 5.1）の答えを採り、ADR D3 を 1 つの原理（記録と計数の窓の重なり）で書き直し、各 finding をその帰結として直した。受け入れた近似は A1（計数の日付に精算がある日は、それより後の日付の精算を入っていないとみなす。TD-266）だけ。
- finding ごとの採否と直した所（番号は相談の relay の表の R1〜R13・H1〜H5・P3）:
  - R1（P1、採る）: 入れ直し（fresh な計数）は保存の前に保存した同じ日付の入出庫を含む。保存の時点の `seen_movement_id` を 1 列持つ（tracking の `stock_counts`、ADR D3 の判定表・D5・D9、35）。吸収した集合を保存する案は採らない（ADR の Rejected）。
  - R2・H1（P1、採る）: 既定の計数の決まらない精算 = その日付の全ての精算（無ければ後の最初の 1 本）。fresh な計数 = 同じ日の n より後の全ての精算（ADR D3 の判定表。同じ日の 3 本目の精算の二重の引きを塞ぐ）。
  - R3・H3（P1・P2、採る）: D2 に求め直しの原理（入力が変わる TX はそれに依る全ての商品を求め直す）を置き、横断の表の「求め直す商品」の列で、精算の一覧が変わる TX は計数を持つ全ての商品とその取込みの商品（連動の on / off を問わない）にした（ADR、31、32）。
  - R4（P1、採る）: 正常な行が全て 0 の Z004 は精算のメタだけの取込みとして受ける（ADR D4、32）。
  - R5（P2、採る）: `settlement_pending` を欠けの隣の精算日で判定する式にした（ADR D6、35）。
  - R6・R7（P2、採る）: fresh な計数は先に保存された計数を全て置き換え、既定の計数どうしと fresh の後の同じ日付の既定の計数は重なれば衝突（写しの土台は最小値）。商品の「入れ直す」を出した（ADR D3 の「計数どうし」・D1・D5、35、42、73）。置換えの記録・UPDATE の案と、締めの取込みの後に入れ直す案内は採らない。
  - R8（P2、採る）: 初期在庫はどの計数より前で、決まらない入出庫に入れない（ADR D1・D3、tracking）。
  - R9（P2、採る）: 完了の後の差は凍結した原価で旧数量と求め直した数量をそれぞれ評価し、丸めた総額を差し引く（ADR D6、35）。
  - R10（P2、採る）: 共有 JAN の在庫連動の拒否は既存の `validation`（field `pos_stock_sync` か `jan_code`）に揃え、`shared_jan_unresolved` を外した（ADR D4、30、40、51）。現物の `src-tauri/src/cmd/mod.rs` の `CmdErrorKind` と 40 §5.3 の `ValidationFailedAt` で足りる。
  - R11・R12（P2、採る）: Matrix の Negative Paths の共有 JAN を直し、writer ごとの後入力の行（`backdated_record_before_count_does_not_change_copy_per_writer`）を足した。
  - R13（P2、採る）: 介入を判断点ごとに数え直した（消費 7、残り 2、予備 0、上限 9）。Q-A は TD-298 で確かめた。
  - H2（採る）: 最後の精算の取込み忘れは受け入れた穴として Consequences に置き、73 の完了の確認の文に足した。
  - H4・H5（採る）: 過剰側で安全な記録として Consequences に 1 行ずつ。入出庫を事象の前に入力した場合を受け入れた穴として Consequences に足し、店への問いを backlog に置いた。
  - P3（採る）: 停止 ADR の D6 の括弧を ADR D12 の (b) に揃えた。読取りの求め直し `derive_stock` を 31 に置き 36 が使う。完了の後の差は完了の時点のカーソル（精算回数・movement・計数の id）で、完了日の完了の後の記録を入れない。migration は写しの不一致を黙って書き換えず操作ログに残し知らせる。取込みの結果の要再確認の一覧の範囲を 32・55・packet で「この取込みで求め直した商品のうち要再確認の商品」に揃えた。「Z004 は時刻を持たない」を「精算の日時は持つが商品別の取引の時刻を持たない」に直した（ADR Context、D-118、packet の Contract Probe P4）。
- round 1 の反例を新しい D3 に当てた結果は、Ordinary Operation の各行と Matrix の D3・D6 の行（`fresh_count_all_later_same_day_settlements_are_undetermined`・`fresh_count_absorbs_movements_saved_before_it`・`settlement_pending_detects_gap_between_imported`・`fresh_count_supersedes_earlier_counts`・`default_count_after_fresh_same_day_conflicts`・`settlement_list_change_recomputes_all_counted_products`・`all_zero_z004_is_imported_as_settlement_only`）に置いた。

### Plan Review round 2（2026-10-17、head `6e867fb9`）

- 判定: Claude 側 fresh Opus 5.5 = reject（P1 0・P2 2・P3 5）、Codex GPT-6 Astra = reject（P1 2・P2 5・P3 1）。原理（ADR D3 の窓の重なり）への反例は無く、残ったのは原理の当て方が抜けた所と古い説明の残り。全て採った。
- finding ごとの直した所:
  1. P1 在庫連動の設定に依る判定: 理由 1 を、決まらない精算の有効な Z004 の movement か、取込みのときに在庫連動だった行（sale_records の `stock_linked=1`）で判定し、今の `pos_stock_sync` に依らないようにした（ADR D3 の理由 1・D8・D9、31、32、pos-tables、21）。連動の切替は後の取込みにだけ効く。Matrix に `sync_off_keeps_recheck_and_range`。
  2. P1 取込みなしの印と精算回数 0: fresh な計数の精算のカーソルは取込みが無ければ NULL にし、比べ方（n が NULL ならどの精算も「k ≤ n」に当たらない）と CHECK を ADR D1・D3・D9、tracking、35 に 1 通りに書いた。完了のカーソルも同じ。Matrix に `fresh_count_with_no_imports_then_settlement_zero`。
  3. P2 在庫連動を始める拒否: 連動しない間の売上が全て効く計数に確実に入っているときだけ許し、決まらない・衝突も拒否にした（ADR D8、30）。Matrix に `enable_sync_rejected_when_unsynced_sale_is_undetermined`。
  4. P2 同じ値の既定と fresh: fresh な計数を代表にし、その窓とカーソルで判定する（ADR D3 の「計数どうし」）。Matrix に `equal_default_and_fresh_counts_use_fresh_window`。
  5. P2 完了の後の差と完了日の後入力: 入出庫の movement は業務日だけで切り、`cutoff_movement_id` を外した（ADR D6・D9、35、tracking、Matrix の `post_completion_difference_is_shown_not_saved`）。
  6. P2 整合性の項目と画面: 36 で比べる項目を数・印・範囲の 4 つにし、`differing_fields` と 4 項目の old / new を DTO とログに置いた。75 に表示する内容の注記を足し（Scope に 75 を足した）、42 で透過を書いた。Matrix に `integrity_detects_recheck_or_range_only_difference`。
  7. P2 古い説明の残り: 32 の確定の段、transaction-tables の header の日付の段、tracking の system_stock の段を ADR D3 の判定表を引く形にした。
  8. P3: AC10 の式に `--glob '!docs/plans/**'` を足した。31 の `apply_stock_change` に Z004 の確定（`commit.rs:157`）からの呼出しの phase=1 と精算回数の渡し方を書き、tracking に `(reference_type = 'csv_import') = (phase = 1)` の CHECK を足した。32 に今の未来の精算日の拒否（`parse.rs:63-77`）を外して知らせに変えることを書いた。ADR D1 の鍵の文を「判定は常に D3 の表、鍵は並びだけ」に直した。一覧から入れ直すときに `stocktake_item_id` を渡すことを 35・73 に書いた。
- 全体の洗い直し:
  - 記録が計数に入るかを決める文: live な正本の「計数」を含み「入る・前・後・鍵・同じ日付・同じ日」に当たる行を ADR 23 行・ほかの文書 22 行挙げた。直したのは ADR の横断の表（「同じ日付の計数があるとき」→「判定表で決まらないとき」）と、上の 7 の 3 か所。残りは D3 の表を引く文、表示・保存の順だけの「鍵」、計数どうしの関係（D3 の「計数どうし」）を述べる文で、D3 と同じ結果になる。
  - 求めた値が使う所まで届くか: 写しの数・印・範囲・理由を、商品の DTO（40）→ 商品フォーム（51）・在庫照会（58）・在庫変動（66）・商品一覧（50。注記を足した）、棚卸しの一覧と完了（35・42・73）、取込みの結果（32・41・55）、整合性（36・42・75。上の 6 で直した）、操作ログ（36 の `integrity_fix`、ADR D10 の移行）で追った。値を落としていたのは整合性と商品一覧で、直した。ホームの件数（53）は写しの数だけを使い、要再確認は棚卸しの画面で扱う（50 の注記）。
  - 設定や状態に依る判定: 在庫連動（理由 1・連動を始める検査。取込みのときの値 `stock_linked` に直した）、共有 JAN（確定のときの候補で決め `stock_linked=0` を残す）、廃番（完了の時点の判定で、完了の後は変わらない）、進行中の棚卸し（一覧の行き先と明細の参照だけで、在庫の判定に入らない）、評価原価（完了で凍結）を挙げた。後から設定が変わって過去の記録の扱いが黙って変わる所は、在庫連動の 1 つで、直した。
- round 2 の反例（連動を切って完了、空の履歴から精算 0、連動しない間の決まらない売上で on、完了日の後入力）は、Ordinary Operation の行と Matrix の上の 5 行で解ける。

### Plan Review round 3（2026-10-17、head `49811f9c`、天井）

- 判定: Claude 側 fresh Opus 5.5 = reject（P1 0・P2 2・P3 5。Ordinary Operation は成立）、Codex GPT-6 Astra = reject（P1 0・P2 4・P3 1）。原理への反例は無い。round の天井に届いたので次の round は開かず、Coordinator が全件を一括是正に振り分けた（DEV_WORKFLOW Review Rules）。是正の差分は closure で確かめる。全て採った。
- finding ごとの直した所:
  1. P2 Matrix の古い期待: `fresh_count_stores_cursors_and_today` と Boundary Checks の null/default を、精算のカーソルは取込みなしなら NULL、movement のカーソルだけ 0 に揃えた。
  2. P2 完了の後の差と決まらない記録: 求め直した完了の時点の在庫が要再確認の明細は、差を確定の差と分け、要再確認の印と範囲で示し確定の合計に入れない（ADR D6、35、65、73）。Matrix に `post_completion_difference_of_undetermined_record_is_range`。
  3. P2 明細の id と商品の一致: 同じ TX で確かめ、違えば書く前に `ValidationFailed`（35、42）。Matrix に `stocktake_item_of_other_product_is_rejected`。
  4. P2 `count_now` の行き先: 73 に、商品を受けて開き、棚卸し・既存の計数・要再確認の有無によらず fresh な計数を保存できる「今の数を入れる」を定め、51 から商品コードと returnTo を渡す。Ordinary Operation に行。
  5. P2 CHECK と NULL: tracking の CHECK を `(reference_type IS 'csv_import') = (phase = 1)` にした。Matrix に `movement_reference_type_null_with_phase_1_is_rejected`。同じ形の洗い出し: 新契約の CHECK（tracking の `stock_counts` の種類とカーソル、movement の settlement_no と phase、master の写しの印と範囲、pos-tables の `stock_linked`）を `rg -n 'CHECK' docs/db-design` で見て、NULL になりうる列を `=` で比べて真偽を作る形はこの 1 つだけだった（ほかは NOT NULL の列か、「なら NULL / NOT NULL」の形）。
  6. P3 `settlement_pending` の (iii): 既定の計数（と n が NULL の fresh な計数）で、精算日が X より前の取り込み済みの精算が無ければ pending（ADR D6、35）。Matrix に `settlement_pending_when_no_settlement_before_count_date`。
  7. P3 「0 にする」の日付: 選べるようにし、既定は今日（ADR D6、35、42、73）。Matrix に `zero_uncounted_with_selected_date`。
  8. P3 受け入れた穴の列挙: ADR の Consequences、D-118 の Guarantee range、本 packet の Design Readiness の 3 か所を同じ列挙（入出庫を事象の前に入力した場合と H2 を含む）に揃えた。
  9. P3 `stock_after`: 同じ TX で求め直した値を、その TX の新しい movement に書く（Z004 の同じ商品の複数の行も同じ値）と 31 に 1 文で決めた。
  10. P3 Matrix の test の名前: Residual Test Gaps の名前を `fresh_count_next_day_settlement_is_not_undetermined` に揃え、既定の計数の A1 を固定する `default_count_next_day_settlement_is_not_undetermined` を足した。
  11. P3 Matrix の mutation の説明: 旧 writer の残りの検出先を `backdated_record_before_count_does_not_change_copy_per_writer` にし、計数なしの SUM の一致では検出しない理由を書いた。

### Plan Review closure（2026-10-17、head `0217ee50`、Codex GPT-6.1 Sol）

- 判定: reject（P1 0・P2 1・P3 2）。S1・S3・S4・S5・S7〜S10 は closed。正本の設計の是正は妥当で、残りは Matrix の書き方。全て採った。
- 直した所:
  1. P2（S2）: Matrix の `post_completion_difference_of_undetermined_record_is_range` の手順を、取込みの後に fresh な計数で保存 → 完了 → 同じ日付の入庫 +5 を後入力、に直した（既定の計数で同じ日の入庫を先に入れると、その時点で要再確認になり完了できず差の検査に届かなかった）。期待は差の範囲 0〜+5、要再確認の印、確定の差の合計に入らない。Ordinary Operation の 12/31 の行も同じ手順（締めの取込みの後の「入れ直す」で完了した商品）に揃えた。
  2. P3（S6）: ADR D6 の (iii) の回復の説明を「(iii) は解ける。欠けた精算があれば (ii) の pending はそれを取り込むまで続く」に限り、Matrix の `settlement_pending_when_no_settlement_before_count_date` に (ii) が続く期待を書いた。
  3. P3（S11）: Matrix の `copy_equals_sum_when_no_count` の用途を写しと SUM の一致に限り、Adjacent Pattern Audit と Main Wiring の writer の切替の検出先を `backdated_record_before_count_does_not_change_copy_per_writer` に揃えた。`rg -n 'SUM|copy_equals_sum'` で Matrix を洗い、ほかに同じ説明は無かった（BIZ-07 の行の SUM は「SUM と違っても不一致に出ない」の対照）。

Closeout（2026-10-17）: Final Review は broad 1 回（head `7ebd412d`）で、Codex（Astra）= reject（P2 1: 欠けの知らせに上限が無い、P3 2）、Claude 側（Fable 5.1）= approve（P3 5）。是正（`c29211ee`）は packet を触らず正本と Matrix だけで、P2 と Codex の P3 1 件・Fable の P3 5 件を直した。是正の closure（Codex Sol）は approve（P3 1）で、helper の record は review = pass。閉じていない P3 の disposition（Coordinator の裁定）: Codex の C3（packet の数が古い: 規模の正本の file の数と AC5 の横断の約束の表の行数）は、この closeout で上の Implementation Results に食い違いと理由を記録して閉じた（本文の数は書き換えない）。closure の P3 1 件（欠けの区間の端の i64 が JavaScript の number で丸まりうる wire の精度）は `docs/backlog.md` の「棚卸しの数の置き場と在庫数の求め直しの runtime」へ follow-up として送った（runtime の lane が `notices` の区間の型を決める）。Owner Effort Budget の表は closeout の時点の値（上限 11、消費 11）に直した。
