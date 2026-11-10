# Plan Packet: 実測と POS の系列の対応を取得・保存する設計

wave に属さない単独の lane（design lane、docs だけ。runtime は後続の lane）。owner の lane 選択は 2026-10-11（repo 外の回答台帳 TD-222: 単位 U の runtime と本 lane の 2 本で始める）。並走は `agent/unit-runtime`（単位 U の runtime）、`agent/backup-offsite`（PR #155 Draft、D-114、docs の design）、`agent/plu-clear`（D-110）。本 lane の branch は `agent/pos-series-design`。

規模と手法（起票の最初に数え、file の数は最終の差分で数え直した）: 触る設計正本は 12 file（時点証拠 ADR、32、29 の EJ parser、35、37、io-task-specs、biz-task-specs、pos-tables、tracking-system-tables と、索引の `docs/DB_DESIGN.md`・`docs/ARCHITECTURE.md`・`docs/FUNCTION_DESIGN.md`。`git diff --name-only "$(git merge-base origin/main HEAD)" HEAD -- docs ':(exclude)docs/plans'` の 16 file から decision-log・backlog・Plans.md・project-memory を除いた数）と decision-log・backlog・Plans.md・project-memory。画面 0、migration 0（proposed の表 1 つと列 2 つを設計に足すだけ）、wire の型 0。横断の約束がかかる経路は 4 つ（取込みの preview / commit の分類、実測の保存 2 経路〈進行中の明細・独立再実測〉、SD の読取り、EJ の区間の完全性）。**1 lane のまま進める**。分ける案（対応の取得・保存の設計と、日次の EJ 取込みの「欠け」の設計を別 lane にする）は採らない。理由: どちらも同じ観測（Z004 の精算回数、EJ の精算票の題、取引一連番号）から決まり、どちらも時点証拠 ADR の D5 の同じ段を書き換えるので、分けると 2 本の lane が同じ段を取り合う。決める値が少ない（足すのは表 1 つと列 2 つ）ので、横断の約束は下の Scope の「値ごとの保証の表」1 つにまとめた。精算票と Z004 の対応づけを確定する条件の細部だけは、owner の決定（repo 外の回答台帳 TD-230）で日次の EJ 取込みの lane の設計へ送り、本 lane は骨までを決める（D10 は本 lane で決め切る）。

## Workflow State

- Phase: archive
- Risk: R3
- Plan Commit: c9623cc469517c7079be116325649e3797321140
- Amendments: b67c94b13f5d2fae7df190c818877befbc8131e7, ec178d317202532cfceb485c210a8b674579f673
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（subagent_type: writer）
- Plan Reviewer: fresh Opus 5.5 + Codex（model は発注時に決める）
- Final Reviewer: Fable 5.1（Claude 側、R3・design lane）+ Codex（GPT-6.1 Sol 既定）。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Review Minimum: 1
- Human Gate: ready,merge
- Branch: agent/pos-series-design

遷移の記録:

1. kickoff → spec-check（2026-10-11、起草役）: owner の決定（repo 外の回答台帳 TD-221「しばらくは CASIO のレジでいく。CASIO の着手も進める」、TD-223「着手条件 4 を外す」）で保留が解けた `docs/backlog.md` の「実測とPOS系列の対応を取得・保存する」を Scope にし、Risk を R3 と記録した（下の Risk）。
2. spec-check → design（2026-10-11、起草役）: 時点証拠 ADR（`docs/adr/2026-09-18-stocktake-time-evidence.md`）の「適用範囲の但し書き」が次の design lane へ送った事項（判定不能を実測前・実測後へ分ける方法、精算系列と同一性の比較、系列、計数のまとまり、時刻区間）と、`docs/function-design/29-io-ej-parser.md` IO-08.10 の「欠け」の定義が正本に無い。同じ commit で設計正本を更新した（下の Design Readiness）。
3. design のまま止める（2026-10-11、起草役）: owner の判断事項 J1（番号印字を採るか）が残り、design → plan-draft の条件「未解決の設計の問いが無い」を満たさない。J1 に依らない部分（SPEC-STK-TIME-D10・D5a、外部前提の観測、clear 行の段、後続の依存の表）は正本へ書いた。
4. design（owner の決定の反映、2026-10-11、起草役）: owner が J1 を A（番号印字を採らない）に決めた（repo 外の回答台帳 TD-224）。ADR D10・D-116・本 packet・Matrix に反映し、同じ問い合わせで出た懸念（設計が SD に寄る）に対して、表・列・用語から媒体の名前を外し（`pos_source_scans`・`source_scan_cursor`・「全数読取り」）、判定を有効にする・外す出口を D10 に定めた。SD のエラーで締められなかった日（TD-160・TD-161）を入力列で確かめ、媒体の入替とファイルを選ぶ経路の資料が根拠に混ざる列を見つけたので、全部実測後の条件を「同じ置き場の 2 回の全数読取りの間に現れた資料」に改めた。
5. design → plan-draft（2026-10-11、起草役、同じ commit）: 条件「設計の出力が正本にある」は Design Readiness の引く正本で満たす。条件「未解決の設計の問いが無い」は、J1 の決定（TD-224）で owner の判断事項が無くなったことで満たす。残る Contract Probe の P1・P3・P7（持ち帰りデータの集計中）と P10 は設計の問いではなく、Plan Gate の前と D10 を有効にする lane の前に確かめる外部前提で、結果に依らず設計は安全側（判定不能）に倒れる。plan-gate への遷移は Coordinator。
6. plan-draft のまま（集計の反映、2026-10-11、起草役）: 持ち帰りデータの集計（独立の数え直し済み）が戻り、Contract Probe の P1・P3・P5・P7 を結果で書き直した。集計で、精算票と Z004 を「題の末尾 + 1」の番号で対応づける起票時の D5a が、番号の差がずれると 1 つ後の精算の記録を黙って拾うと分かったので、対応づけを精算の日時の同一性（同じ分か 1 分前、ちょうど 1 件）に改めた。D10 の条件 3 の日付の比較に 1 日の余裕を足した。Goal と設計の方向は変えず、owner の判断が要る事項は増えていないので plan-draft に留める。
7. plan-draft → plan-gate（2026-10-11、Coordinator）: packet と Matrix は commit 済み。Coordinator は ADR の SPEC-STK-TIME-D10（全数読取りと全部実測後の条件、有効にする・外す）と D5a（区間の終端・開始）を実読し、持ち帰りデータの集計（独立の数え直し済み）が Contract Probe の P1・P3・P5・P7 に反映されていることを確かめた。P1 のうち保存が異常終了した精算の資料の残り方と、P2・P10 は未観測のまま残り、D10 を有効にする変更の owner-operated gate に置いてある（本 lane は判定を有効にしない）。
8. plan-gate → design（2026-10-12、Coordinator）: Plan Review round 1 の是正（Review Response 参照）が設計正本の条件・保存する値と、本 lane の Scope（区間の対応づけをどこまで決めるか）を変えるので、design へ戻して歩き直す（`docs/DEV_WORKFLOW.md` Workflow State の「Scope か design を無効にする差し戻しは design か plan-draft へ戻す」）。owner の決定（repo 外の回答台帳 TD-228・TD-229・TD-230）を起草役の commit で正本・packet・Matrix に反映した。
9. design → plan-draft（2026-10-12、Coordinator、同じ commit）: 設計の出力は正本にある（Design Readiness）。owner の判断事項は残っていない（区間の対応づけを確定する条件の細部は、TD-230 で日次の EJ 取込みの lane の設計へ送った。本 lane の設計の問いではない）。
10. plan-draft → plan-gate（2026-10-12、Coordinator、同じ commit）: packet と Matrix は起草役の commit にある。Coordinator は ADR の SPEC-STK-TIME-D10（全数読取り、資料の初回受領の読取り、全部実測後の条件、限界、有効にする・外す）を実読し、AC の検索式と docs の検査が通ることを確かめた。Plan Review round 2 へ出す。
11. plan-gate（round 2 の是正、2026-10-12）: Plan Review round 2 の是正（Review Response 参照）。D10 の条件の式・保存する表と列・Scope の範囲（区間の対応づけは骨まで）は変えず、その場で直したので plan-gate に留める。
12. plan-gate（round 3 の是正、2026-10-12）: Plan Review round 3（上限）の後の是正（Review Response 参照）。次の round は開始しない（`docs/DEV_WORKFLOW.md` Review Rules の round 天井）。
13. plan-gate（訪店の結果の反映、2026-10-15、Coordinator）: 持ち帰り資料で、Z004 と精算票の記録を日時のほかに番号でも結べる観測が得られ、正本の「結べる値は日時だけ」が狭いと分かったので、その場で事実の文を訂正した（Review Response 参照）。D10 の条件の式・保存する表と列・Scope の範囲は変えていない。
14. plan-gate → plan-approved（2026-10-15、Coordinator、本 commit）: round 3 の disposition と 13 の訂正の後、owner が plan-approved を承認した（repo 外の回答台帳 TD-255。round 3 で Codex 側に P1 が残ったので owner の承認による。先例 archive の `2026-10-07-unit-extension.md` の遷移の記録 10）。Plan Commit = `c9623cc4`（承認した版）。reviewer の再確認は Final Review。
15. plan-approved → implementing（2026-10-15、Coordinator。state だけ）: 本 lane は docs だけで、実装の段で足す code は無い。設計の正本は plan-first の commit にあるので、このまま Final Review（R3、Minimum 1）に出す。
16. implementing のまま Gated Amendment 1（2026-10-15、Coordinator）: Final Review broad（head `e482ad2a`）の是正（Review Response 参照）。起草役が `ae2d9abf` と `b67c94b1` で直し、Coordinator が差分を読んだ。D10 の条件の式・保存する表と列・全数読取りの条件・D5a・D3 は変えていない。Amendments = `b67c94b1`（GA1 の最後の commit）。次は現在の head の Final Review broad 2 本。
17. implementing のまま Gated Amendment 2（2026-10-15、Coordinator）: Final Review broad 2（head `c472650e`）の是正（Review Response 参照）。起草役が `e0c984fe` と `ec178d31` で直し、Coordinator が差分を読んだ。D10 の条件の式・保存する表と列・全数読取りの条件・D5a・D3 は変えていない。Amendments に `ec178d31`（GA2 の最後の commit）を足した。次は現在の head の Final Review broad 2 本。

## Owner Effort Budget

- 介入回数上限: 8（2026-10-12 に既定 6 から上げた、owner 承認〈repo 外の回答台帳 TD-239〉。理由: Plan Review が round 天井に達し、plan-approved を owner に諮るため）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 8 | 5（2026-10-11: J1 の判断〈TD-224〉、限界の受け入れ〈TD-228〉。2026-10-12: 区間の対応づけを本 lane でどこまで決めるかの判断〈TD-230〉、round 3 の disposition と上限の改定〈TD-236・TD-239〉。2026-10-15: 訂正を本 lane に入れてから plan-approved にする判断と、その承認〈TD-254・TD-255、1 つの decision point〉。TD-221・TD-222・TD-223 は起票の前の lane 選択で、TD-229 は owner の問いなので、本 lane の介入に数えない） | 2（Ready 1、merge 1） | 1 | 8 = 5 + 2 + 1 |

既定値・数え方・上限に届くときの扱いは `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
POS の入力（Z004・EJ）と棚卸しの実測の前後の判定、その証拠の保存先（DB の表と列）、operator の数え直しの頻度を決める設計。本 lane は docs だけだが、後続の runtime が写す契約を決めるので R3。データの破壊・実店舗データの露出・不可逆な操作は無い（設計は SD に書かず、既存の数量・履歴を変える migration を持たない。D10 を有効にする migration は、実測の `source_scan_cursor` と資料の `first_scan_id` だけを判定不能が増える向きへ変え、数量・在庫・売上は変えない。tracked には件数の細部・商品名・金額を書かない）ので R4 ではない。この変更で required gate の green / red は変わらない（docs だけで、function-design の新しい file・REQ の増減・code の変更が無い）。Scope の予定 path を `scripts/ci/classify-changes.sh` の workflow の一覧（`AGENTS.md`・`CLAUDE.md`・`docs/DEV_WORKFLOW.md`・`docs/ci.md`・`docs/AGENT_OPERATING_MANUAL.md`・`docs/code_review.md`・`docs/quality/review-checklist.md`・`docs/project-profile.md`・`docs/agent-guidance/*`・`docs/templates/*`・`.agents/*`・`.claude/{rules,commands,skills}/*`・PR template と、full に倒す `scripts/*`・`.github/*` の列）に当てて、当たる file は無い。Final Review Minimum 1。

## Goal

Goal Invariant:

### 最小完了条件

- 後続の runtime lane の Writer が、チャットの履歴を見ずに source docs だけで「数えた商品の、どの精算の資料を数え直しなしで在庫へ入れてよいか」と「その判定をいつ有効にし、どう外すか」を実装できる（時点証拠 ADR の SPEC-STK-TIME-D10 と D4 の表、32 の分類の手順 4、`pos_source_scans`・`pos_import_sources.first_scan_id`・`source_scan_cursor`）。D10 は本 lane で決め切る。
- 日次の EJ 取込みの lane の設計が、「EJ がそろっているか（欠け）」と「前回の Z004 はどれか」を推測せずに書け、「どの記録がどの Z004 の区間の端か」については、決まっている骨（終端の候補を日時で抽出する、候補が 1 件であることだけでは対応づけを確定しない、欠けと読めない記録 → 対応づけの確定 → 前回の証明の順に見る）と、その設計が決める未決の一覧を source docs から読める（SPEC-STK-TIME-D5a、29 の EJ parser IO-08.9・IO-08.10）。対応づけを確定する条件は本 lane で決めない（TD-230）。その設計が merge するまで、区間の対応づけは確定せず EJ の完全性に依る判定が働かないことが、正本に帰結として書かれている。
- ADR が次の design lane へ送っていた事項の全てに、決めた結果（番号印字は採らない）が正本に書かれ、「次の design lane」を担当とする live な記述が設計正本に残らない。
- 店の毎日の操作を増やさずに、数えた商品を数え直さずに置く限り、要再確認が「数えた（数え直した）日の精算と次の精算の資料で売れた商品」に限られる設計になっている。この 2 つの精算の資料は判定できず、数え直しの保存からも同じ 2 つの精算の資料が改めて判定できなくなること（続けて売れる商品は、売れるたびに数え直しが続きうる）が、限界として正本に書かれている（owner は一旦受け入れた。repo 外の回答台帳 TD-228）。判定の考え方と保存先の名前がレジ・媒体に依らず、媒体に依る部分が adapter と「今のレジでは」の文に限られている。

### 失敗定義

- 設計が、数える前の販売をもう一度引く・数えた後の販売を引かない結果を、要再確認を付けずに通す経路を持つ。
- runtime の Writer が、読取りの行をどの読取りがいつ置くか・読取りの間ほかの操作とどう排他するか・読取りをどの条件で完了にするか（窓の中の全ての資料の受領、列挙の順、終わりの識別子の取り直し）・資料の初回受領の読取りをいつ・どう書くか・実測のどの時点の値を保存するか・直前の精算の資料の選び方・判定の有効と無効の決まり方・有効にするときに既にある行と値の扱い・欠けの判定の単位のどれかを推測しないと書けない。
- 媒体の入替、別の媒体の読取り、利用者が場所を選んだ root（SD の写しの folder を含む）の読取り、重なる全数読取り、読取りの途中の媒体の差替え（終わりまでに元の媒体へ戻さないもの）、読取りの途中のレジスターツールの取込み、ファイルを選ぶ経路で取った資料、読取りの途中の別の経路の受領、途中で失敗した読取り、締められなかった日のどれかを含む入力列で、数える前の販売を含む資料が全部実測後になる。
- 無効の間に保存した行と値、保存の不具合のあった間の行と値（指す行が無い ID を持つ `first_scan_id` を含む）が、有効にした後・直した後に全部実測後の根拠になる。
- 日時の合う精算票が 1 件あることだけで EJ の区間を完全にし、売れて戻った商品を見逃す経路を持つ。
- 限界（数えた後の 2 つの精算、数え直しのたびの 2 つの精算、有効にした直後の 2 つの精算、ファイルで選んだ日・場所を選んで読んだ日・読取りが失敗した日、SD が固定の disk として見える reader、SD を 2 枚交互に使う運用、読取りの間に SD を 2 度差し替えて元へ戻した列を見分けられないこと、区間の対応づけが確定しない間の EJ 待ち）を、起きないこと・保証として正本に書く。
- 未検証の外部前提（番号印字の桁・位置、精算の書込みの時点）を、確かめた事実として正本に書く。
- 設計が安全側へ倒れすぎて、普通の日（数えない日、最後に数えて〈数え直して〉から 3 つ目以降の精算。どちらも、前回と今回の締めの資料を、自動で見つけた SD の完了した読取りで受領している場合。D10 を有効にした直後の 2 つの精算は限界として除く）に要再確認を付ける。

### 非目的

- runtime の code・test・fixture・migration・bindings（後続の lane）。`scripts/probes/stocktake_time_model.py` への D10 の追加（D10 を実装する runtime lane）。
- 番号印字の protocol（印の発行・保存・計数のまとまり・EJ での位置の決め方）の設計（採らない。TD-224）。
- EJ の記録の保存先・取込み画面・file の絞り方（日次の EJ 取込みの lane）。Z004 を SD から読む経路の窓と画面（接続の runtime lane）。
- 精算票と Z004 の対応づけを確定する条件の細部（候補を 1 件に絞る整合の条件、対応づけ済みの組が 1 つも無いときの扱い、日時の幅、同じ日時の精算票が複数あるときの扱い、前回の Z004 の終端の対応づけが確定していないだけのときの区間の開始の扱い）。日次の EJ 取込みの lane の設計が決める（TD-230）。
- ADR D1〜D9 の既存の契約、D4 の出口、UI の文言、準備照会の issue の変更。
- ADR の Status を accepted にすること（㉘ ⑤ で行う。D-109）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

設計を含む変更なので、数える日とその後の日の操作列を置く。本 lane は docs だけで、表の通常運用は後続の runtime（㉘ ④・⑤、SD 直読みの runtime、接続の runtime lane、日次の EJ 取込み）と、D10 を有効にする変更（外部前提の owner-operated gate の後）の後に成り立つ。**この文書を完了できる**（設計正本がそろう）ことと、**通常運用を達成できる**（店で自動在庫連動が動く）ことは別で、後者は本 lane では未達。番号印字は採らない（owner 決定 TD-224）ので、店の操作は今決まっている毎日の締め（SD を PC に差して読む。D-111）から増えない。精算の番号は例（数えた日の締めの精算を m とする）。「自動」は、売れた分が数え直しなしで在庫から一度だけ減ること。売れて同じ精算で戻った商品を EJ で見分ける相殺の判定は、日次の EJ 取込みの lane の設計が区間の対応づけを確定する条件を決め、その runtime が入った後にだけ働く（ADR D5a。それまでは下の「EJ の対応づけが確定しない間」の行）。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 数えない普通の日。在庫連動商品に実測の履歴が無い、または最後に数えて（数え直して）からも、D10 を有効にする変更の配布からも、精算を 3 回以上済ませていて、前回と今回の締めの資料を、自動で見つけた SD の完了した読取りで受領している | 締め: 精算 → SD を PC に差す → 「SD から読む」→ 取り込む | 売れた分が在庫から一度だけ減る。要再確認は出ない | 取込みの結果が出る | Z004 を SD から読む経路は接続の runtime lane。P1・P2・P3・P10 |
| 棚卸しの日（営業中）。前日の締めで精算 m − 1 まで取込み済み | 商品ごとに「今から数える」→ 数える → 「数を保存」 | 数と、その時点の読取りの位置がアプリに残る | 保存済みの表示 | ADR D1（既存） |
| 同じ日の締め | 精算（m）→ SD を読む → 取り込む | 数えた商品のうち、その日に売れた商品だけに「取り込んだ後に数の再確認が必要です」。売れなかった数えた商品と、数えていない商品には出ない（売れて同じ日に戻った商品は EJ の相殺の判定） | 要再確認の商品を数え直して保存すると消える（数え直すと、その保存から 2 つの精算の資料が改めて判定できなくなる。下の「連日売れる商品」の行） | 相殺の判定は日次の EJ 取込みの後（それまでは下の「EJ の対応づけが確定しない間」の行）。ADR D4（既存） |
| 翌営業日の締め | 精算（m + 1）→ SD を読む → 取り込む | 前日に数えた商品のうち、この日に売れた商品に要再確認。ほかは出ない | 同上 | 精算 m + 1 の区間は、前日の読取りより前の販売を含みうるので判定不能のまま（D10 の限界）。owner はこの「翌日」の分が数え直しになることを聞いた上で決めた（TD-224） |
| その次の営業日以降の締め（その間に数え直していない商品） | 精算（m + 2 以降）→ SD を読む → 取り込む | 数えた商品が売れても、数え直しなしで在庫から一度だけ減る | 取込みの結果が出る | D10 の全部実測後（精算 m + 1 の資料と精算 m + 2 の資料の両方が、数えた日の締めの読取りより後の、この SD の完了した読取りの中で初めて受領された）。P1・P2・P3・P10 |
| 数えた商品を廃番にした、またはその code を変えた PLU 書出しをレジに反映した日（その日に売れていると、未精算の売上は code が全桁 0 の行に移る） | 締め: 精算 → SD を読む → 取り込む | その商品は、精算の番号によらず D10 の前と同じく clear 行の判定になり、その日の EJ にその商品の行があれば要再確認。EJ の対応づけが確定しない間は相殺の確認待ち（数え直しで消える） | 要再確認の商品を数え直して保存する | D10 が決めるのは資料の行に載った販売・返品の前後だけで、売上がその商品の行に載っているかは決めない（ADR D4 の clear 行の規則・D10）。code を変える書出しが未精算の売上ごと旧い code の枠を消すこと（ADR D4 の clear 行の段、D5 の外部前提の clear の受付）に依る |
| 棚卸しの日の締めの取込みを済ませた後に数えた | 数えて保存 → 翌営業日・その次の営業日の締め | 精算 m + 1 と m + 2 の資料で売れた数えた商品に要再確認。m + 3 から自動 | 同上 | 保存の後の最初の読取りは翌営業日（D10） |
| 連日売れる商品を、要再確認が出るたびに数え直す（進行中の棚卸しの商品は、確定の前に数え直すしかない） | 毎日: 締め → 要再確認の商品を数え直して保存 | 数え直しのたびに、その先の 2 つの精算の資料が改めて判定できなくなるので、売れるたびに要再確認が出て、自動には移らない。在庫は数え直しのたびに現物の数へ合う（誤らないが、数え直しの手間は減らない） | 数え直した後の 2 つの精算の資料で売れなければ、その次の精算から自動。確定済みの商品は、数え直さずに置く選択がある（次の行） | D10 の限界。owner はこの限界を一旦受け入れた（TD-228）。番号印字を後から足す案は D-116 の Revisit（任意で使える形にする。TD-229） |
| 確定済みの棚卸し（または独立再実測）の商品に、数えた日の精算と次の精算の資料で要再確認が付いた | 数え直さずに、毎日の締めを続ける | 3 つ目の精算の資料から自動に戻る。先の 2 つの資料の要再確認は、取込みの準備の表示（商品単位）に残る。その資料に数える前の販売が入っていた場合、帳簿は数え直すまで現物より少ないままになりうる | 独立再実測か、次の棚卸しでその商品を数える（force_fill では飛ばせない）と表示が消え、帳簿が現物の数へ合う | ADR D4（確定済みの商品の要再確認は取込みを止めない。既存）、D10 の限界 |
| 数えた後、アプリを終了し、PC を再起動し、何日か取り込まなかった | 後日の締めで SD を読み、溜まった精算を古い順に取り込む。その後も毎日取り込む | 溜まっていた精算と、その次の精算の資料で売れた数えた商品に要再確認。その次の精算から自動。保存済みの数と読取りの位置は再起動で消えない | 同上 | ADR の「㉘ への引継ぎ」の合格条件（TD-223）。接続の runtime lane の L3 |
| レジに SD カードのエラーが出て、その日は締められなかった（店に実例がある。repo 外の回答台帳 TD-160・TD-161）。その日に数えた商品もある | 後日、精算をして SD を読み、取り込む。その後も毎日取り込む | 締められなかった日の販売は後日の精算の資料に入る。その資料と次の精算の資料で売れた数えた商品に要再確認。その次の精算から自動。読めない資料が SD に残っている間は、読取りが完了せず、数えた商品が売れるたびに要再確認が続く（自動に戻らない） | 読めない資料が窓から外れるか、読めるようになる | 精算をせずに終わった日か、精算は済んで保存だけ失敗した日かは分かっていない（P1）。後者で資料が後からまとめて書かれるなら D10 の外部前提が崩れるので、確かめるまで D10 を有効にしない |
| SD の読取りが途中で失敗した（読めない file があった、途中で抜いた）。その読取りで受領済みの資料がある | 読み直して完了する → 取り込む | 失敗した読取りの中で受領した資料と、その次の精算の資料は、数えた商品（数えてから日が経った商品を含む）が売れていれば要再確認 | その次の精算の資料から自動に戻る | 完了していない読取りの中の初回受領は根拠にしない（D10 の条件 3） |
| SD の読取りの途中で、SD を抜いて別の 1 枚を差した、またはレジスターツール（CV17）の取込みを動かした | 元の SD を差して読み直す → 取り込む | 差し替えた読取りは完了にならない（終わりに取り直した識別子が始めと違うか、取れない）。その読取りで受領済みの資料と、その次の精算の資料は、数えた商品が売れていれば要再確認。取込みで `XZ` から `XZ_BKUP` へ動いた資料は、受領されるか、読取りが失敗して読み直しになる | その次の精算の資料から自動に戻る | 読取りの間は SD を抜き差しせず、レジスターツールの取込みを動かさないことが前提（P4）。読取りの間に 2 度差し替えて元の SD へ戻した場合は、アプリは見分けられず、D10 は守れない（限界。D10 の全数読取りの 3・4） |
| SD を別の 1 枚に替えた（SD カードのエラーの後に替えた場合を含む） | 新しい SD で締めて読む、を続ける | 新しい SD の 1 回目と 2 回目の読取りで受領した精算の資料は、数えた商品が売れていれば要再確認。3 回目の読取りの資料から自動。古い SD にだけある資料や、ファイルを選ぶ経路で取った資料は取り込めるが、数えた商品が売れていれば要再確認になる | 新しい SD の 3 回目の読取り | 置き場の識別子（P10）。D10 の条件 3。SD を 2 枚交互に使うと、直前の精算の資料と当の資料の置き場が常に違い、自動に戻らない（安全側。本 lane は決めない） |
| 年末の棚卸し（何日もかけて数え、最後に確定） | 毎日: 数える → 締めで取り込む → 要再確認が出た商品を数え直す。最終日の締めの取込みの後に、要再確認の商品を数え直してから確定する | 確定できる。棚卸しの期間に続けて売れる商品は、毎日数え直す（上の「連日売れる商品」の行。今の ADR と同じ）。確定の後に取り込む資料で売れた商品は、上の「確定済み」の行と同じ扱い | 未解消の要再確認が 0 | ADR D4 の確定の拒否（既存） |
| SD を読めない日（ファイルを選ぶ経路か、SD の写しの folder を「場所を選ぶ」で読んで取り込む。SD カードのエラーで別の方法でデータを取った日を含む） | 「ファイルを選んで取り込む」か、場所を選んで読む | その日の資料は全部実測後にならず、数えた商品（数えてから日が経った商品を含む）が売れていれば要再確認 | 次の精算の資料も要再確認になり（直前の精算の資料がファイルで選んだ資料のため）、その次の精算の資料から自動に戻る | 資料と直前の精算の資料の両方に、SD の完了した読取りの中の初回受領を求める（D10 の条件 3）。ファイルを選ぶ経路と、利用者が場所を選んだ読取りは、全数読取りを作らない（行を置かない）。SD の写しを取外し可能な媒体に置いて差すと、アプリはレジの SD と区別できず行を置くので、D10 は守れない（P4） |
| SD が固定の disk として見える reader の PC（店の PC の reader では取外し可能に見えた。29 の日報 parser の SD-25） | 毎回、場所を選んで読む | 数えた商品が売れるたびに要再確認（D10 は働かない。今の ADR と同じ） | 自動には移らない（本 lane は決めない） | 行を置くのは自動で見つけた取外し可能な drive の root の読取りだけ（IO-09-D1、D10 の限界） |
| EJ の対応づけが確定しない間（日次の EJ 取込みの lane の設計と runtime が入る前。ADR D5a） | いつもの締め | 判定できない 2 つの精算の資料では、数えた商品のうち売れなかった商品（数量・金額とも 0 の行）にも相殺の確認待ちが付き、数え直しでしか消えない（売れて同じ精算で戻った商品を EJ で見分けられないため）。在庫連動は有効にできない（`ej_unverified`）ので、店の本番はこの状態では始まらない | 日次の EJ 取込みの lane の設計が対応づけを確定する条件を決め、その runtime が入る | ADR D5a が日次の EJ 取込みの lane の設計へ送った項目、D4 の在庫連動の有効化の条件 |
| D10 を有効にする前、または外した後の build | いつもの締め | 数えた商品が売れるたびに要再確認（今の ADR と同じ）。読取りと実測の位置の保存は続く | D10 を有効にする変更の配布 | D10 の「有効にする・外す」 |
| D10 を有効にする変更を配布した直後 | いつもの締め | 有効にする前に保存した読取りと実測の位置は使われない。配布の後の 2 つの精算の資料で売れた数えた商品（数えてから日が経った商品を含む）に要再確認 | その次の精算の資料から自動 | D10 の「有効にする・外す」の migration。待ちは「数えた後の 2 つの精算」と同じ形 |

Plan Review は、この列が「正常な条件で目的を達成できるか」と「危険な結果を出さないか」を別々に答える（`docs/DEV_WORKFLOW.md` Review Rules）。

## owner の判断事項（2026-10-11 決定）

### J1 番号印字を採るか → A（採らない）。owner 決定 2026-10-11、repo 外の回答台帳 TD-224

確認済み事実（起票時に示したもの）:

- 今の ADR のままでは、数えた商品は数えた後に売れるたびに毎日要再確認になる（ADR Consequences）。本 lane の D10 は、店の操作を増やさずに、これを「数えた日の精算と次の精算の資料で売れた商品」に限る（Ordinary Operation）。
- レジの番号印字は、取引の外で打つと EJ に行として残り、売上・現金・Z 帳票に影響しない（訪店の確認 2026-09-28、Issue #105 の A7。`docs/backlog.md` の同項目の着手条件 (2)）。独自の記録ヘッダと番号を持たず、直前の記録の本文の後ろか、続く記録の本文の中に現れる（`docs/function-design/29-io-ej-parser.md:477`）。
- 番号印字で判定不能をさらに狭めるには、EJ の取込み（日次の EJ 取込みの lane）が先に要る。D10 は EJ が無くても働く。
- 店は今、紙の棚卸しで、数え終わった商品が売れたらその場で数を直している（repo 外の回答台帳 L-069）。D10 の後に残る数え直しは、この今の作業より範囲が狭い。
- 着手条件 4 の筋書き（TD-223）のうち「翌日」の分は、A では満たさない。A で数え直しなしになるのは、取込みを再開した時点で置き場にあった精算より 2 つ以上後の精算から。

選択肢（決定の記録として残す）:

| 案 | 店の操作 | 数え直しになる範囲（数えた在庫連動商品） | 追加の設計・前提 | 結果 |
|---|---|---|---|---|
| A 番号印字を採らない（D10 だけ） | 増えない | 数えた後に初めて置き場を読んだ時点で置き場にあった精算と、その次の精算の資料で売れた商品 | 無し（D10 の外部前提だけ） | **採用**（起票時の推奨、candidate） |
| B D10 に番号印字を足す | 計数のまとまりの終わりに、アプリが出した番号をレジで 1 回打つ | 前の締めから番号を打つまでの間に売れた商品だけ | 印の発行と保存、計数のまとまり、EJ での位置の決め方、打ち間違い・打ち忘れ・二重印字の出口。番号印字の残る桁数と位置の確認。日次の EJ 取込みが前提 | 不採用（後から D10 の上に足せる。D-116 の Revisit） |
| C 番号印字だけ（D10 を外す） | B と同じ | 打ち忘れ・EJ の欠けのたびに、今の ADR の毎日の要再確認へ戻る | B と同じ | 不採用 |
| D 何も足さない（今の ADR のまま） | 増えない | 数えた後に売れるたびに毎日 | 無し。日次の自動在庫連動（D-070）に届かない | 不採用 |

決定の内容: A。owner は、A では TD-223 の筋書きの「翌日」の分が数え直しになることを聞いた上で選んだ。ADR の「㉘ への引継ぎ」の合格条件は起票時の形のままでよい。同じ問い合わせで owner が述べた懸念（設計が SD に寄る。店の操作が減るのは望ましい）に対し、次の 2 つを設計に入れた。

- 判定の考え方・表・列・用語をレジと媒体に依らない形にした（ADR D10 の「考え方」の段、`pos_source_scans`・`source_scan_cursor`・「全数読取り」。媒体に依るのは adapter〈IO-09〉と「今のレジでは」の文だけ）。
- 判定を有効にする・外す出口を 1 通りに定めた（ADR D10 の「有効にする・外す」、D-116 の (9)）。

その後の owner の決定（repo 外の回答台帳。要旨）:

- TD-228: A の限界「続けて売れる商品は、売れるたびに数え直しが続きうる」（数え直しの保存からも同じ 2 つの精算の資料が改めて判定できなくなる）を一旦受け入れる。
- TD-229: 番号印字を後から足すなら、任意で使える形（打たなければ今の結果に戻る）にする。
- TD-230: 精算票と Z004 の対応づけは、本 lane では骨だけを決め、細部は日次の EJ 取込みの lane の設計へ送る。

残る owner の判断事項は無い。

### 店主に聞く候補（判断は済んだので急がない。運用の見積りの材料）

- 年末の棚卸しの期間に、数え終わった商品が売れて数を直すことは、1 日に何品くらいあるか。台帳には「売れたら都度直す」事実（L-069）だけがあり、件数の答えは無い。D-116 の Revisit（番号印字を後から足すか）の材料になる。

## Scope

本 lane は docs だけを変える（予定 file の全部）。所有の列は並走 lane との重なり（`git diff --name-only 7070382f...agent/backup-offsite`・`...agent/plu-clear` の結果と、`agent/unit-runtime` の発注の範囲から）。

| file | 変更する節 | 他 lane との重なり |
|---|---|---|
| `docs/adr/2026-09-18-stocktake-time-evidence.md` | Status、適用範囲の但し書きの最終段落（決めたことの表）、D1 の S / E の 1 文、D4 の表と前後の文・相殺の段の冒頭の対象の文・clear 行の箇条・区間全体の箇条の対象の文、D5 の時刻の箇条・完全な EJ の条件の括弧・合計の照合の除外・戻りの箇条・帰属できない行の箇条の 1 文・外部前提の段落、D5a（新設。骨と、後続の設計へ送った項目の一覧）、D4 の EJ 待ちの箇条・再評価の順・在庫連動の有効化の条件の各 1 文、D8 の表と 1 文、D10（新設。考え方・全数読取り〈完了にする条件の列挙の順と識別子の取り直しを含む〉・読取りの排他・資料の初回受領の読取り・実測側の保存・全部実測後の条件・限界・読取りが途中で失敗した日・締められなかった日と媒体の入替・読取りの間の置き場の変化・外部前提・有効にする・外す〈有効にする migration、保存の不具合の出口、不変の例外を含む〉・番号印字は採らない）、Rejected Options の追記と 1 箇条、Consequences の 3 か所、㉘ への引継ぎ（依存の表と合格条件、接続の runtime lane への申し送り）、Evidence の 1 文、Revisit Trigger の最終段落、判定不能からの復旧の 1 箇条、D8 の表の flags の行 | なし |
| `docs/function-design/32-biz-csv-import-service.md` | 時点証拠契約の「一つの分類関数とcommit」の入力の文と手順 4、手順 5、手順 6 の冒頭の 2 文・clear 行の文・区間全体の対象の文と登録の変化の判定の 1 文、EJ の証拠の段落の再評価の順、外部 probe 表の EJ完全性の行の括弧と「精算の書込み」の行、表の後の観測の段落 | なし（`agent/unit-runtime` は Z004 の小数の配線をしない。D-113 (8)） |
| `docs/function-design/29-io-ej-parser.md` | IO-08.9 の表の 3 行、IO-08.10 の「欠け」の箇条と、後続の設計が先に決めること・小数の数量の取引の 2 箇条 | なし |
| `docs/architecture/io-task-specs.md` | IO-08 の「行わない」の 1 行 | なし |
| `docs/function-design/37-biz-daily-report-import-service.md` | §37.9 の「Z004 と EJ への引継ぎ」の後半の文 | なし |
| `docs/function-design/35-biz-stocktake-service.md` | 時点証拠契約の「beginとsaveの処理」の手順 5 に `source_scan_cursor` を保存 TX で採る 1 文（API と wire は変えない） | なし（`agent/unit-runtime` の範囲に 35 のこの節は無い） |
| `docs/architecture/biz-task-specs.md` | 時点証拠契約の冒頭の範囲の 1 語（D1〜D10）、BIZ-03 の箇条の再評価の順の 1 文 | なし |
| `docs/DB_DESIGN.md`・`docs/ARCHITECTURE.md`・`docs/FUNCTION_DESIGN.md` | 時点証拠契約の冒頭の範囲の 1 語（D1〜D10） | なし |
| `docs/db-design/pos-tables.md` | 時点証拠契約の表に `pos_source_scans` の 1 行と `pos_import_sources.first_scan_id` の 1 行、表の後の「初回受領は不変」の文に 1 句 | `agent/plu-clear` は `plu-tables.md` で別 file |
| `docs/db-design/tracking-system-tables.md` | 時点証拠契約の表に `source_scan_cursor` の 1 行、SPEC-STK-TIME-D8-K1 の箇条に 1 つ、理由の決め方の箇条の再評価の順・clear 行と区間全体の対象、要再確認 flag の規則の 1 箇条 | なし |
| `docs/decision-log.md` | 末尾に D-116 だけを追記 | 全 lane が末尾に追記（`agent/backup-offsite` = D-114 と D-111 の中の 1 行、`agent/plu-clear` = D-110）。merge 順で両方を残す |
| `docs/backlog.md` | 「実測とPOS系列の対応を取得・保存する」の項目（現在の状態・入力 3・入力 4・入力 5 の箇条を足し、日付つきの 2 箇条に 2026-10-11 の結果を足す）、「㉘ ③ 〜 ⑤ の申し送り」の箇条の 2 か所と、その下の「㉘ ④ と日次の EJ 取込みへの申し送り」の箇条（新設） | `agent/unit-runtime` は「単位の拡張」の項目だけ。`agent/backup-offsite` は backup の項目と「保存と起動の守りの follow-up」。行は重ならない |
| `docs/Plans.md` | 「次の着手順」の 1 の末尾の 1 句、その下の 2 段落の各 1 文、「次の行動」の後ろの製品の判断の節の STK-1 / STK-2 の行の 1 文。lane の行は足さない（D-097） | closeout の PR が「直近の完了」を触る。行は重ならない |
| `docs/project-memory.md` | 「精算の系列」の箇条の 2 文（「EJ タイトル末尾 4 桁 + 1 = Z004 の settlement_no」を、規則として読めない言い方に直す） | なし |
| `docs/plans/2026-10-11-pos-series-design.md`、`docs/plans/test-matrices/2026-10-11-pos-series-design.md` | 新設 | なし |

触らない: `src-tauri/`・`src/`・`scripts/`（`scripts/probes/stocktake_time_model.py` を含む）、`docs/function-design/90-traceability.md`（生成物）、`docs/function-design/23-io-z004-parser.md`（識別メタの抽出は変えない）、`docs/function-design/42-cmd-sales-stocktake.md`（保存の API と wire は変えない。`source_scan_cursor` は保存 TX の中で BIZ が採り、wire に出さない）、`docs/function-design/29-io-daily-report-parser.md`（IO-09。置き場の識別子の取得と `XZ` を先にする列挙の順は、接続の runtime lane が §29.7 に足す）。

### 値ごとの保証の表（DB・wire に出す値）

本 lane が足す値は全て DB の中だけで、wire（Tauri command の DTO、bindings）には出ない。

| 値 | 範囲 | 0・空の意味 | 失敗の時点と順序 | 種類の混在 | 保証する所 |
|---|---|---|---|---|---|
| `pos_source_scans.id` | 正の整数（AUTOINCREMENT） | 取らない | 行を置くのは、自動で見つけた取外し可能な drive の root の読取りだけ（利用者が場所を選んだ root の読取りは置かない）。置き場の file の列挙より前に置く。置かない・置けない読取りは全数読取りにせず、その読取りの中の受領の `first_scan_id` は NULL。行を置く前から完了・失敗まで DB の接続の排他を持ち続け、失敗・中止・アプリの異常終了の後は未完了のまま残す | 商品別売上の資料の読取りだけ | 接続の runtime lane の BIZ（ADR D10） |
| `pos_source_scans.medium_id` | 空でない文字列 | 空は許さない（取れなければ行を置かない） | 行を置く前に adapter から取る。読取りの終わり（全ての資料の受領・対応づけの後、完了を書く前）にもう一度取り、始めの値と違う・終わりに取れないときは完了にしない（行の値は始めの値のまま、行は未完了のまま残る）。窓に資料が 1 本も無い読取りでも取り直す | 等しいかどうかだけを比べる（大小・部分一致を使わない）。別の種類の置き場の識別子と重ならない形は adapter が決める | DB の CHECK（pos-tables）と adapter（IO-09） |
| `pos_source_scans.window_from` | YYYY-MM-DD、未完了は NULL | 取らない | 全数読取りの条件 2〜4（窓の中の全ての資料の受領、移動元を先にする列挙の順、終わりに取り直した識別子が始めと同じ）を満たした後に、`completed_at` と一緒に書く。途中の失敗・中止は NULL のまま | 資料の `settlement_date`（YYYY-MM-DD）と日付として比べる。形が違えば条件不成立。窓は終わりを持たない | BIZ（ADR D10 の条件 3）と DB の CHECK |
| `pos_source_scans.completed_at` | アプリの時計の日時（既存の表記）、未完了は NULL | NULL は未完了（判定に使わない行）。空文字列は取らない | 同上。完了した行は更新しない。未完了の行を後から完了にしない | `window_from` と両方 NULL か両方 NOT NULL。判定には NULL かどうかだけを使い、値の新旧を比べない | DB の CHECK（pos-tables）と BIZ |
| `pos_import_sources.first_scan_id` | 正の整数か NULL | NULL は、`pos_source_scans` に行を置いた読取りの外の受領（ファイルを選ぶ経路の受領、利用者が場所を選んだ root の読取りの中の受領。読取りの途中に別の経路で受領した場合を含む）と、有効にする migration の時点で既にあった資料。0 は取らない | source を INSERT する TX の中でだけ書く。同じ hash の再受領・取消・同一性拒否・再読取り・DB の restore の後の業務で変えない。例外は、有効にする migration（保存の不具合が分かったときの再適用を含む）が全ての資料の値を NULL にすることだけ。受領の時点では指す行は未完了で、完了したかは判定のときに行を読む | 商品別売上の資料の読取りの行の ID だけ。資料の ID の範囲や受領時刻から推さない。未完了の行を指す値と、指す行が無い値は条件不成立 | 資料の受領の BIZ（ADR D10）と DB の CHECK（pos-tables） |
| `source_scan_cursor`（実測の証拠） | 0 以上の整数 | 読取りの行が 1 つも無い | 実測の保存 TX の中で採る（開始時には採らない）。採れなければ保存 TX 全体が失敗 | measured は必須、uncounted・auto_filled・legacy は NULL、recount は NOT NULL。列は registry への登録より前に足す（後から足す migration は、既存の実測にその時点の読取りの最大 ID を入れる）。保存の後は変えない（例外は、有効にする migration〈保存の不具合が分かったときの再適用を含む〉が、その時点の読取りの最大 ID に置き換えることだけ。最大 ID より大きい不具合の値は下がる。実測でない kind の行は対象にしない） | DB の CHECK（SPEC-STK-TIME-D8-K1 の 6 つ目）と保存の BIZ |
| settlement_no の比較 | ASCII 数字だけの文字列を i64 として比べる | 0 の直前（−1）の資料は無いので条件不成立 | 比べられない値（数字以外・空・NULL・桁あふれ）は成り立たない側 | 同じ帳票種別・同じ machine_no の間だけ。精算票の題の末尾の番号とは比べない | BIZ（ADR D5a の番号の比べ方） |
| 区間の終端の候補の抽出（Z004 の精算日時と EJ の記録の日時） | 分の精度の日時。候補に入れる幅は後続の設計が決める（観測は、Z004 が記録と同じ分か 1 分後） | 日時の無い Z004 は候補 0 件 | 候補 0 件は EJ が届いていない扱い（EJ 待ち）。候補が 1 件でも、それだけでは対応づけを確定しない（確定の条件は後続の設計が決める。決まるまで EJ 待ち） | どちらもレジの時計の値。同じ machine_no の間だけ。アプリの時計と比べない。欠けの判定と前後の判定に使わない | 日次の EJ 取込みの lane の設計と BIZ（ADR D5a の骨と、後続の設計へ送った項目） |

### runtime の lane への申し送り（形が変わる所。本 lane は code を書かない）

`rg -n 'source_cursor' src-tauri/src` と `rg -n 'fn (classify|receive_source|save_stocktake)' src-tauri/src/biz` の現物（開始 HEAD `7070382f`）。行番号は起票時のもので、runtime の lane は起票時に数え直す。

| 形が変わるもの | 作る所 | 読む所・test |
|---|---|---|
| 実測の証拠に `source_scan_cursor` を足す（schema） | `src-tauri/src/db/schema_time_evidence.rs:83`〜`:91`（`stocktake_items` の列と K1 の CHECK）、`:130`（`stocktake_recounts` の列） | 同 file の test（`:357`・`:377` の列の一覧、`:496`〜`:554` の INSERT、`:652`、`:733`、`:795`、`:834`） |
| 資料に初回受領の読取りの ID を足す（`pos_import_sources.first_scan_id`）。`receive_source` は、`pos_source_scans` に行を置いた読取りの中の受領のときにその行の ID を受け取る引数を持ち、INSERT のときだけ書く（読取りの外の caller は「無し」を渡す） | `src-tauri/src/db/schema_time_evidence.rs:29`（`pos_import_sources` の CREATE TABLE）、`src-tauri/src/biz/csv_import_service/time_evidence.rs:26`（`receive_source`）、`src-tauri/src/db/sales_repo.rs:1440`（`NewPosImportSource`）・`:1507`（`upsert_pos_import_source`） | `receive_source` の呼出し: `src-tauri/src/biz/csv_import_service/tests/time_evidence_tests.rs:116`・`:117`・`:129`・`:139`、`src-tauri/src/biz/stocktake_service/time_evidence_tests.rs:556`。`pos_import_sources` へ直接 INSERT する test: `src-tauri/src/db/schema_time_evidence.rs:537`・`:547` |
| 実測の保存が cursor を採る | `src-tauri/src/biz/stocktake_service/time_evidence.rs:404`（`save_stocktake_count`）と、証拠を組む `:515` | `src-tauri/src/db/stocktake_repo.rs:837`〜`:886`（measured の UPDATE と recount の INSERT）、`:1084`、`src-tauri/src/biz/stocktake_service/time_evidence_tests.rs` |
| 最新の有効な実測の読取りに cursor を足す | `src-tauri/src/db/stocktake_repo.rs:677`〜`:740`（`source_cursor` を読む SELECT と型） | `src-tauri/src/biz/csv_import_service/time_evidence.rs:205`（`classify_product`） |
| 分類に全部実測後の枝を足す | `src-tauri/src/biz/csv_import_service/time_evidence.rs:205`〜`:226`（`Measured` で `source_id <= source_cursor` でない枝の前） | `:228`（`classify_stock_rows`）、`src-tauri/src/biz/csv_import_service/tests/time_evidence_tests.rs` |
| `pos_source_scans`（新しい表。列は id・scanned_at・medium_id・window_from・completed_at）、Z004 を SD から読む経路、置き場の識別子の取得 | 無い（`src-tauri/src/io/` に `register_sd.rs` はまだ無い。SD 直読みの runtime の lane が作る。設計の `RegisterSdRoot`〈29 の日報 parser §29.7〉は path と表示用の `volume_label` だけで、識別子を持たない。接続の runtime lane が IO-09 に読取り専用の取得を足し、IO-09-D3 の許可の列にもその API を足す。`list_register_sd_entries` は `XZ` と `XZ_BKUP` の列挙の順を決めていないので、`XZ` を先にする順も足す） | 接続の runtime lane が起票時に Scope を決める |

表のほかの申し送り:

- 保存の契約の test（Matrix の D10-3・D10-4・D10-5・D10-11・D10-16・D10-17・D10-22・D10-27・D10-28・D10-29・D10-30・D8-1）は、保存を入れる変更と同じ変更で入れる。D10 を有効にする変更の merge 条件が、保存を入れた変更からその test が main で green であることを含む（ADR D10 の「有効にする・外す」）。D10 を有効にする変更は、有効にする migration（1 つの TX で、全ての実測の `source_scan_cursor` をその時点の `pos_source_scans` の最大 ID〈未完了の行を含む。無ければ 0〉に置き換え、全ての `pos_import_sources.first_scan_id` を NULL にする。最大 ID より大きい不具合の値は下がる）を同じ変更に入れる。有効にする前に保存した行と値は判定に使わない。保存の code の不具合が分かったときは、不具合を直す変更が同じ migration をもう一度当てる。
- 全数読取りは、行を置く前から完了・失敗まで、同じ DB に対する唯一の操作にする（別の全数読取り・実測の保存・読取りの外の受領・取込みの commit と取消・backup・restore と重ならない）。DB の接続の排他を読取りの間持ち続ける（`docs/function-design/45-cmd-daily-report-import.md` §45.6a 手順 2 の日報の scan と同じ持ち方を必須にする）。失敗・中止・アプリの異常終了の後、行は未完了のまま残し、次の読取りは新しい行から始める。
- 行を置くのは、IO-09 が自動で見つけた取外し可能な drive の root の読取りだけ。利用者が場所を選んだ root（SD の写しの folder を含む）の読取りは行を置かず、受領に読取りの行の ID を渡さない。窓の始まりは読取りの日の前日以前にする（37 §37.9 手順 2 の窓なら満たす）。
- 読取りを完了にする条件（ADR D10 の全数読取りの 2〜4）: 窓の中の全ての商品別売上の資料を受領・対応づけし終える、`XZ` を `XZ_BKUP` より先に列挙する、読取りの終わり（全ての資料の受領・対応づけの後、完了を書く前）に置き場の識別子をもう一度取り、始めの値と同じときだけ完了にする。違う・終わりに取れないときは未完了のまま残す。識別子の取得と列挙の順は今の IO-09 の設計（29 の日報 parser §29.7）に無いので、接続の runtime lane が足す（取得の API は IO-09-D3 の許可の列にも足す）。
- 接続の runtime lane への候補（採っていない）: 読取りの間に媒体を 2 度差し替えて元へ戻した列は、識別子の取り直しでは見分けられない（ADR D10 の外部前提「読取りの間に置き場を変える操作をしない」に依る）。adapter が読取りの間じゅう媒体を掴む形（掴んだ媒体が差し替わったことを OS が失敗として返すこと）を OS が保証できるかは、Windows の実機か公式の資料で確かめる（Contract Probe / L3）。
- 接続の runtime lane への候補（採っていない）: 完了にする条件に「同じ識別子の直前の完了した読取りが窓の中で見た資料が、今回の窓の中に無ければ、別の媒体として完了にしない」を足すと、識別子が別の媒体と重なる場合を条件で落とせる。採らない理由は、条件が 1 つ増え、窓の始まりが動く日に完了できないこと。置き場の識別子の安定を確かめた結果で、その lane が見直す（ADR の「㉘ への引継ぎ」）。
- 接続の runtime lane の L3 の筋書きの候補: SD の写しを取外し可能な媒体に置いて差す（アプリはレジの置き場と区別できず行を置く。P4）、SD が固定の disk として見える reader で場所を選んで読む（行を置かない）、読取りの途中でアプリを終了して起動し直す（行は未完了のまま）、読取りの途中で SD を別の 1 枚に差し替える（行は未完了のまま）、読取りの途中でレジスターツールの取込みを動かす（動いた資料は受領されるか、読取りが未完了になる）。
- EJ の期間の証拠を受け取る BIZ の interface（㉘ ④）は、「区間の対応づけが確定しない」を不完全（EJ 待ち）として受け取る形にする。日次の EJ 取込みの lane の設計が ADR D5a の未決を決めて merge するまで、区間の対応づけは確定せず、EJ の完全性に依る判定（相殺の判定、在庫連動の有効化の条件のうち EJ の完全性）は働かない（`ej_unverified` は外れない）。再評価は、欠けと読めない記録 → 対応づけの確定 → 前回の証明の順に見る（ADR D4・D5a）。
- 小数の数量の取引は今の EJ parser では復元不能で、その取引を含む区間は完全にならない（Contract Probe の P13）。小数を読む変更は、単位の拡張の runtime と日次の EJ 取込みで扱う。

## Non-scope

- runtime の code・test・fixture・migration・bindings・`90-traceability.md`・`scripts/probes/stocktake_time_model.py`。
- 番号印字の protocol の設計（採らない。TD-224）。IO-09 の型と関数への識別子の取得の追加と、`XZ` を先にする列挙の順（接続の runtime lane が 29 の日報 parser §29.7 を改める。本 lane は D10 に要件だけを書く）。
- EJ の記録の保存先、同じ番号で中身の違う記録の保存の扱い、読む file の絞り方、取込み画面（日次の EJ 取込みの lane）。
- 精算票と Z004 の対応づけを確定する条件の細部（ADR D5a の未決の 5 項目）。日次の EJ 取込みの lane の設計が決める（TD-230）。本 lane は骨までを決め、決まるまでの帰結（区間の対応づけが確定せず、EJ の完全性に依る判定が働かない）を正本に書く。
- Z004 を SD から読む経路の窓の決め方、読み直さずに受領済みと対応づけてよい条件、画面（接続の runtime lane）。
- 名称の重なりの検知と登録名の付け直しの支援（PLU 書出し側の別の変更。ADR D4）。
- 共有 JAN の手軽な解除（DATA-2）、Z004 layout B。
- 持ち帰りデータの新しい集計（本 lane は、別の担当が数えて独立に数え直した集計の出力〈件数・日付・番号だけ〉を引くだけで、自分では数えない）。

## Acceptance Criteria

- AC1（D10 と D5a が正本にある）: `rg -c '^### SPEC-STK-TIME-D10: ' docs/adr/2026-09-18-stocktake-time-evidence.md` が 1、`rg -c '^#### SPEC-STK-TIME-D5a: ' docs/adr/2026-09-18-stocktake-time-evidence.md` が 1（開始 HEAD ではどちらも 0。`git show 7070382f:docs/adr/2026-09-18-stocktake-time-evidence.md | rg -c 'SPEC-STK-TIME-D(10|5a)'` は hit なしで exit 1）。
- AC2（「次の design lane」を担当とする live な記述が設計正本に 0）: `rg -n '次の ?design ?lane' docs --glob '!docs/archive/**' --glob '!docs/plans/**' --glob '!docs/backlog.md'` の hit が 0（開始 HEAD の実測: ADR 12、32 に 1、29 の EJ parser に 4、io-task-specs に 1、Plans.md に 2。起票後の実測: 0）。`docs/backlog.md` は日付つきの経緯の 3 行（同項目の本文と STK-1・STK-2 の行）が残る。
- AC3（決定の記録）: `rg -c '^## D-116: ' docs/decision-log.md` が 1 で、その entry が Decision (1)〜(9)・Guarantee range・owner の決定（TD-224）を持つ。
- AC4（保存先の設計）: `rg -c 'pos_source_scans' docs/db-design/pos-tables.md` が 2（表の行と、`first_scan_id` の行）、`rg -c 'source_scan_cursor' docs/db-design/tracking-system-tables.md` が 1 以上（開始 HEAD ではどちらも hit なし）。`rg -c 'first_scan_id' docs/db-design/pos-tables.md` が 2（前の版 `997bfeb7` では hit なし）、`rg -n 'source_cursor_start' docs --glob '!docs/archive/**' --glob '!docs/plans/**'` が hit なし（前の版 `997bfeb7` の実測: 4 行〈ADR 2、32 に 1、pos-tables に 1〉。読取りの中の受領を資料の ID の範囲から推す形が正本に残らない）。
- AC5（外部前提の観測と probe 表）: `rg -c '^\| 精算の書込み（ADR D10）' docs/function-design/32-biz-csv-import-service.md` が 1、`rg -c '^観測（2026-10-11' docs/function-design/32-biz-csv-import-service.md` が 1。ADR D5 の外部前提の段落が「観測したもの」と「まだ観測していない外部前提」に分かれ、観測を owner-operated gate の通過に数えない文を持つ。
- AC6（clear 行の段が実物に合う）: `rg -c 'Z004から落ち' docs/adr/2026-09-18-stocktake-time-evidence.md` が hit なし（開始 HEAD の実測: 3）。
- AC7（古い状態の文）: `rg -n 'Square か CASIO かの判断待ち）|実機確認が着手条件のまま' docs/Plans.md docs/backlog.md` の hit が 0（開始 HEAD の実測: Plans.md 2、backlog 1）。`docs/Plans.md` に lane の行・branch・Phase を足していない（`git diff "$(git merge-base origin/main HEAD)" HEAD -- docs/Plans.md | rg -n '^\+.*(agent/pos-series-design|#159|Phase)'` が hit なし。本 lane の branch 名・PR 番号・Phase の語だけを見る。既存の完了履歴の行を書き換えた `+` の行に残る、ほかの PR の番号は対象にしない。開始 HEAD `15c842ab` でも hit なし）。
- AC10（媒体の名前を core の名前から外した）: `rg -n 'pos_sd_observations|sd_observation_cursor|SD ?の全数受領' docs --glob '!docs/archive/**' --glob '!docs/plans/**'` の hit が、`docs/decision-log.md` の D-116 の Alternatives の 1 行（旧い名前を不採用の案として挙げる行）だけ。`rg -c '番号印字は採らない' docs/adr/2026-09-18-stocktake-time-evidence.md` が 1 以上、`rg -n '採否はownerの判断を待つ' docs/adr/2026-09-18-stocktake-time-evidence.md` が hit なし。
- AC11（有効にする・外す出口）: `rg -c '^\*\*有効にする・外す\*\*' docs/adr/2026-09-18-stocktake-time-evidence.md` が 1 で、その段が、決める所（code だけ）・有効にする migration・無効の間の保存と、その間の行と値を有効にした後に使わないこと・保存の不具合が分かったときの出口・不変の例外・外したときの適用済みの取込みの扱いを持つ。`rg -c '^- 保存の不具合が分かったとき: ' docs/adr/2026-09-18-stocktake-time-evidence.md` が 1（前の版 `997bfeb7` では hit なし）。
- AC12（対応づけに番号の差を使わない）: `rg -c '題の末尾の番号に1を足した値' docs/adr/2026-09-18-stocktake-time-evidence.md` が hit なし（前の版 `c46e7418` の実測: 1）、`rg -c 'の翌日以降' docs/adr/2026-09-18-stocktake-time-evidence.md` が 1 以上。
- AC13（D5a は骨と未決の一覧で、候補 1 件だけでは確定しない）: `rg -c '^- \*\*候補が1件であることだけでは対応づけを確定しない\*\*' docs/adr/2026-09-18-stocktake-time-evidence.md` が 1、`rg -c '^- \*\*未決（日次のEJ取込みのlaneの設計が決める）\*\*' docs/adr/2026-09-18-stocktake-time-evidence.md` が 1、`rg -c '^- \*\*決まるまでの帰結\*\*' docs/adr/2026-09-18-stocktake-time-evidence.md` が 1（前の版 `997bfeb7` ではどれも hit なし）。`rg -c 'ちょうど1件あるとき対応づく' docs/adr/2026-09-18-stocktake-time-evidence.md` が hit なし（前の版 `997bfeb7` の実測: 1）。
- AC14（限界が正本にある）: `rg -c '^\*\*限界（数えた後の2つの精算の資料は判定できない）\*\*' docs/adr/2026-09-18-stocktake-time-evidence.md` が 1（前の版 `997bfeb7` では hit なし）、`rg -c '毎日は続かない' docs/adr/2026-09-18-stocktake-time-evidence.md` が hit なし（前の版 `997bfeb7` の実測: 1）。
- AC15（読取りの排他と不変の例外が正本にあり、無効の間の値を使う文と旧い再評価の順が残らない）: `rg -c '^\*\*読取りの排他\*\*' docs/adr/2026-09-18-stocktake-time-evidence.md` が 1、`rg -c '^- 不変の例外: ' docs/adr/2026-09-18-stocktake-time-evidence.md` が 1（前の版 `1e08f9f5` ではどちらも hit なし）。`rg -n '判定に使ってよい' docs --glob '!docs/archive/**' --glob '!docs/plans/**'` が hit なし（前の版 `1e08f9f5` の実測: 2 行〈ADR 1、decision-log 1〉）。`rg -n '前回の証明を先に判定' docs --glob '!docs/archive/**' --glob '!docs/plans/**'` が hit なし（前の版 `1e08f9f5` の実測: 2 行〈32 に 1、biz-task-specs に 1〉）。
- AC16（読取りを完了にする条件と、migration の置換えが正本にある）: `rg -c '^4\. 読取りの終わり' docs/adr/2026-09-18-stocktake-time-evidence.md` が 1、`rg -c '^\*\*読取りの間の置き場の変化\*\*|^読取りの間の置き場の変化: ' docs/adr/2026-09-18-stocktake-time-evidence.md` が 1（開始 HEAD `15c842ab` ではどちらも hit なし）。`rg -n '引き上げる向き|大きくなる向きにだけ|値を大きくする向き' docs --glob '!docs/archive/**' --glob '!docs/plans/**'` が hit なし（開始 HEAD `15c842ab` の実測: 3 行〈ADR 2、tracking-system-tables 1〉）。
- AC8（docs だけ）: `git diff --name-only "$(git merge-base origin/main HEAD)" HEAD -- src src-tauri scripts package.json package-lock.json` の出力が空。
- AC9（検査）: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan` が ERROR 0・WARN 0。

## Design Readiness

設計の完了条件は `docs/DEV_WORKFLOW.md` Design Phase Rules の Design completion criteria。

- 引用する設計正本（節まで）: `docs/adr/2026-09-18-stocktake-time-evidence.md` 適用範囲の但し書き・D4 の表・D5 の外部前提・D5a・D8・D10・㉘ への引継ぎ・Revisit Trigger／`docs/function-design/32-biz-csv-import-service.md` 時点証拠契約の「一つの分類関数とcommit」と「外部probeと本番条件」／`docs/function-design/29-io-ej-parser.md` IO-08.9・IO-08.10／`docs/function-design/29-io-daily-report-parser.md` §29.7.2（SD-02・SD-18・SD-22・SD-23・SD-25。変えない）と §29.7.4（root の見つけ方。変えない）・§29.7.5（列挙。変えない）・IO-09-D3（許可する API の列。変えない）／`docs/function-design/35-biz-stocktake-service.md` 時点証拠契約の「beginとsaveの処理」の手順 5／`docs/function-design/45-cmd-daily-report-import.md` §45.6a 手順 2（日報の scan の DB の排他の持ち方。変えない）／`docs/function-design/37-biz-daily-report-import-service.md` §37.9 の引継ぎ／`docs/db-design/pos-tables.md`・`tracking-system-tables.md` の時点証拠契約／`docs/decision-log.md` D-103・D-105・D-109・D-111（変えない）・D-116。
- 必要な設計成果物（Design artifact selection の当たる行）: BIZ の判定の変更 = updated in this PR（ADR D10、32 手順 4）／表と列 = updated in this PR（pos-tables、tracking-system-tables。migration の番号と index は runtime）／EJ の入力の互換の規則（欠けと、区間の対応づけの骨）= updated in this PR（ADR D5a、29 の EJ parser）。対応づけを確定する条件 = defer（日次の EJ 取込みの lane の設計。TD-230）／durable な横断の決定 = D-116 と ADR／operator の流れ = existing sufficient（画面・文言・操作は増えない。ADR D9 を変えない）／Tauri command・DTO = 該当なし（wire に出す値が無い）。
- plan にしかない durable な判断の昇格先: 決めたことは全て ADR と D-116 に置いた（J1 の決定は D-116 の「owner の決定」と ADR D10 の番号印字の段）。packet にだけあるのは、J1 の選択肢の表（決定の記録）と、runtime の lane への行番号の申し送り。
- 前提・制約と、延期した design gap の follow-up: D10 は外部前提 P1〜P4・P10（Contract Probe）に依り、owner-operated gate の証拠がそろうまで全部実測後の枝を code に入れない（ADR D10 の「有効にする・外す」、32 の外部 probe 表の行）。延期: (a) 窓の決め方と、読み直さずに受領済みと対応づけてよい条件（接続の runtime lane。ADR D10 が「条件 2 を弱めない根拠を Contract Probe で示す」と縛る）。(b) 置き場の識別子を IO-09 のどの関数が返すかと、`XZ` を先にする列挙の順を IO-09 のどこに書くか（接続の runtime lane。D10 が要件〈同じ媒体で変わらない・別の媒体と違う・読取り専用・取れなければ行を置かない・読取りの終わりに取り直して始めの値と同じときだけ完了にする・移動元を先に列挙する〉を縛る）。(c) EJ の記録の保存と file の絞り方（日次の EJ 取込みの lane。D5a が欠けの判定を縛る）。(e) 精算票と Z004 の対応づけを確定する条件（ADR D5a の未決の 5 項目。日次の EJ 取込みの lane の設計。D5a の骨が、日時の候補が 1 件であることだけでは確定しないこと・評価の順を縛る）。(d) SD カード保存が異常終了した精算の Z004 の残り方（資料に例が無く確かめられない。D10 を有効にする変更の owner-operated gate に残す。証拠を得る手段が無いと分かった場合は、残る危険を受け入れて有効にするかを、その変更の lane が owner に諮る）。延期が安全な理由: どれも、決まるまで判定不能・EJ 待ち（数え直し）へ倒れ、在庫を黙って誤らせない。(e) の間は EJ の完全性に依る判定が働かず、`ej_unverified` が外れないので、在庫連動の本番は始まらない。
- 絶対保証（cannot happen / always happens）の例外と escape hatch の自己点検: 「全部実測後と判定した資料の販売は実測の後」の例外は、外部前提が崩れた場合（P1: 資料が精算の後でまとめて書かれる、P2: 精算で PLU の集計が 0 に戻らない、P3: path の日付と中身の精算日が違う、P4: 人が置き場に file を足す・消す・書き換える・改名する〈SD の写しを取外し可能な媒体に置いて差すことを含む〉、読取りの間に置き場を変える操作をする〈媒体を差し替えて元へ戻す、レジスターツールの取込みを動かす〉、P10: 識別子が同じ媒体で変わる・別の媒体と重なる）だけで、ADR D10 の外部前提に全て書いた。媒体の入替・別の媒体の読取り・利用者が場所を選んだ root の読取り・ファイルを選ぶ経路の資料・読取りの途中の別の経路の受領・途中で失敗した読取りの中の受領・後から受領した古い資料は、条件 3（資料と直前の精算の資料の両方が、実測の保存より後の完了した読取りより後に、同じ置き場の完了した読取りの中で初めて受領された）が成り立たないので例外にならない。重なる全数読取りは、読取りの排他（runtime が DB の接続の排他を、行を置く前から完了・失敗まで持ち続ける契約）で起きない。読取りの途中で媒体を別のものへ差し替えた読取りは、終わりに取り直した識別子が違うか取れないので完了にならず、例外にならない。差し替えて元の媒体へ戻した列だけは識別子で見分けられず、P4 の違反として例外に入る。読取りの途中のレジスターツールの取込みは、移動元を先にする列挙の順が受ける（移動が一方向であるという推定に依り、外れた場合は P4 が覆う）。条件は精算の系列を証明しないので、番号の reset を挟む列の全てを塞がない（reset は資料に現れておらず、起きる前に決め直す。ADR の Revisit Trigger）。「数えた後は数え直しなしで減る」は保証ではなく、数えた（数え直した）後の 2 つの精算の資料、D10 を有効にした直後の 2 つの精算の資料、ファイルで選んだ日・場所を選んで読んだ日・読取りが失敗した日の資料とその次の精算の資料は判定できず、SD が固定の disk として見える reader の PC と SD を 2 枚交互に使う運用では D10 が働かない（ADR D10 の限界）。外部前提が崩れたと分かった後の出口は ADR D10 の「有効にする・外す」の「外す」（枝を外す。適用済みは再判定せず、新しい実測で戻す）、保存の code の不具合が分かった後の出口は同じ段の「保存の不具合が分かったとき」（有効にする migration と同じ migration をもう一度当て、全ての実測の cursor をその時点の読取りの最大 ID に置き換え、全ての資料の `first_scan_id` を NULL にする）。「保存の後は変えない」（`source_scan_cursor`）・「INSERT の後は変えない」（`first_scan_id`）・append-only（`stocktake_recounts`）の例外は、この migration だけ。「EJ がそろえば相殺の確認待ちが解ける」の例外は、区間の対応づけが確定しない間（ADR D5a の未決が決まるまでは全ての区間）と、小数の数量の取引を含む区間。「D4 の出口は変えない」は、D10 の条件を満たさない資料の全てが今までどおり判定不能になることで保つ（D10 は After を足すだけで、Before・Unknown・flag の理由・解消の規則を変えない）。「実測開始前に受領済み」と D10 の条件が同時に成り立つ場合は、D4 の表の文で前者を先に見る。共有 JAN の行は、D10 の実測後でも未実測と同じく保留の対象のまま。
- 判定（ready / not ready）と理由: ready（plan-draft に進める）。理由: 設計の出力（D10・D5a の骨と未決の一覧・D4 の表・D8 の表と列・外部前提の観測・後続の依存の表と合格条件・D-116）が上の正本にあり、owner の判断事項 J1 は 2026-10-11 に決まった（TD-224）。限界の受け入れ（TD-228）と、対応づけを確定する条件を日次の EJ 取込みの lane の設計へ送ること（TD-230）も owner が決めた。残る P1 の異常終了・P2・P10 の確認は、設計の問いではなく D10 を有効にする前の外部前提の確認で、結果がどちらでも正本の設計は変わらない（崩れれば D10 を有効にしない、だけ）。

## Registration / Generation Obligations

本 lane（docs だけ）は該当なし（function-design の新しい file・REQ の増減・route・画面・command を足さない。ADR に足した D5a・D10 は既存の file の中の節）。runtime の lane の義務: 時点証拠 schema に `pos_source_scans` と `source_scan_cursor` を足すときは `docs/function-design/22-mnt-migration.md` と ㉘ ⑤ の registry の登録に含める。分類の関数を cfg(test) の外へ出すとき（㉘ ⑤）は `src-tauri/tests/design_compliance_test.rs` の対応表に載せる（D-109 (3)）。

## Impact Review Lenses

実機確認・POS 連携・持ち帰りデータ起点なので適用する。

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | Which concepts belong to replaceable external adapters, and which concepts are stable app-core contracts? | Architecture, function design, decision-log, Plan Packet | core: 「同じ置き場の 2 回の全数読取りの間に現れた資料の精算は、その間に行われた」と「実測の保存の順」で実測後を決める考え方、`pos_source_scans`・`pos_import_sources.first_scan_id`・`source_scan_cursor`・判定の条件。名前に媒体を入れていない。adapter（CASIO 固有）: 置き場が SD の `XZ` / `XZ_BKUP` であること、Z004 の名前、置き場の識別子の取り方（ボリュームの通し番号）、Z004 を書く精算票の種類（日計明細とＰＬＵ）と、精算日時が記録の日時と同じ分か 1 分後という対応、記録ヘッダの 4 桁の欄 = machine_no、取引一連番号。D5a は CASIO の形に依る節で、レジが替わると D5a と IO-09・EJ parser を取り替える。D10 は、精算のたびに資料を書く置き場をアプリが読めるレジでだけ働く（ADR Consequences） | レジの入替が決まったとき（D-116 の Revisit） |
| Fact check / design decision split | Which claims are observed facts from hardware/tool/files, and which are app decisions that need source-doc promotion? | Investigation doc, source design docs, decision-log | 観測は ADR D5 の「観測したもの」と 32 の観測の段落に分けて書き、gate の通過と区別した。未観測は「まだ観測していない外部前提」と D10 の外部前提。判断は D10・D5a・D-116 | Contract Probe |
| Lifecycle / retry | What happens before, during, after, and after failure for import/export, duplicate input, rollback, retry, cancellation, and re-run? | Function design, DB design, UI design, Test Matrix | 読取りの途中の失敗・中止・アプリの異常終了・媒体の差替え（終わりに取り直した識別子が始めと違う・取れない）は未完了の行で、判定に使わない。読取りは、行を置く前から完了・失敗まで、同じ DB に対するほかの操作（別の読取り・実測の保存・受領・取込みと取消・backup・restore）と重ならない。締められなかった日・媒体の入替・ファイルを選ぶ経路は判定不能に倒れる（Ordinary Operation）。判定そのものを外す出口は code の変更 1 通り（ADR D10）。読取りの途中で失敗した後に読み直しても、先に受領した資料の初回受領の読取り（未完了の行）は変わらず、その資料と次の精算の資料は判定不能になる。再起動の後も行と cursor は DB に残る。同じ資料の再選択は最初の受領 ID と初回受領の読取りのまま（D2・D10）。DB の restore では資料・読取り・実測が同じ時点へ戻る。取消の後の再取込みは同じ判定をやり直す。同一性拒否が後から付くと実測後から判定不能へ変わる（安全側） | Matrix の State Lifecycle |
| Operator workflow | What does the operator do in the real sequence across app, external tool, media, print/export, backup, and recovery? | Screen/UI design, function design, Plan Packet manual checks | 操作は増えない（番号印字は採らない。TD-224）。数えた（数え直した）日の精算と次の精算の資料で売れた商品の数え直しが残り、続けて売れる商品は数え直しが続きうる（TD-228 で受け入れた限界。Ordinary Operation） | 接続の runtime lane の L3（ADR の合格条件） |
| Replacement path | If the external system changes, which files/modules/docs are replaced and which app-core contracts remain stable? | Architecture, function design, decision-log | レジが替わると、D5a の対応づけの規則、置き場の読み方と識別子（IO-09）、D10 の外部前提の確認を取り替える。`pos_source_scans`・`first_scan_id`・`source_scan_cursor`・D10 の条件・D4 の表は名前も形も残る。新しいレジが置き場を持たない場合は D10 が働かず、要再確認へ戻る（D-116 の Revisit） | D-116 の Revisit |
| Data safety / evidence | How can the claim be supported by anonymized shape/count/hash/procedure evidence without committing real store data? | Plan Packet Data Safety, investigation doc, review evidence | 観測は形と規則だけを書き、件数は `docs/backlog.md` の同項目の入力 3〜5（独立に数え直した集計の、件数・日付・番号だけ）に置いて、ほかの file はそこを参照する。新しい表は売上値・file 名・file 本体を持たない | Data Safety |
| Reporting / accounting semantics | Are totals, summaries, item records, returns, corrections, and inventory movements modeled separately enough to avoid false business meaning? | DB design, function design, report design, Test Matrix | D10 は在庫の適用の可否だけを決め、売上の記録（sale_records）は変えない。全部実測後の資料の返品は通常の在庫の戻しで、行の前後を見る相殺の判定を要しない（計数の後の販売と返品は帳簿へそのまま入れてよい）。clear 行で消えて商品の行に載らない売上の判定（ADR D4 の clear 行の規則と前回の code が分からない区間の規則）は全部実測後の資料でも当てる | Matrix |
| Manual verification | Which assertions cannot be proven by automated tests and require Windows native L3, external tool import, or real-device confirmation? | Plan Packet, Test Matrix, PR body | 実 SD で、読取りの時点で無かった精算が後から現れること、SD カード保存が異常終了した精算の Z004 の残り方（P1）、ボリュームの通し番号が読取りをまたいで同じであること（P10）、通しの筋書き（数える → 終了 → 再起動 → 後日の取込み） | 接続の runtime lane の L3 と Contract Probe |
| 環境・再現性 | 新設の環境依存（toolchain / CI runner / OS 差異等）を repo-pinned config で強制するか、明示的に defer するか | repo-pinned config, Plan Packet | 置き場の識別子の取得は Windows の API（ボリュームの情報）に依る。Windows 以外と test では adapter が別の識別子を返す形にする（接続の runtime lane が決める。IO-09 の drive の列挙と同じ扱い） | — |

## Boundary / Wire Contract

本 lane は wire（Tauri command の DTO、bindings、JSON、CSV）を変えない。DB の保存の互換だけを書く。

- producer: 接続の runtime lane の BIZ（`pos_source_scans` の行）、資料の受領の BIZ（`pos_import_sources.first_scan_id`）、実測の保存の BIZ（`source_scan_cursor`）
- consumer: 取込みの分類（32 の手順 4）
- wire type: 無し（DB の中だけ。UI・command に出さない）
- internal type: 行の ID・`first_scan_id`・cursor は i64（`first_scan_id` は NULL あり）、`window_from` は YYYY-MM-DD の文字列、`completed_at` は既存の表記の日時の文字列（NULL かどうかだけを読む）、`medium_id` は adapter が返す文字列（等しいかどうかだけを比べる）
- precision/range: Scope の「値ごとの保証の表」
- round-trip path: 読取りの開始で DB の接続の排他を取り、識別子を持つ行を置く（自動で見つけた取外し可能な drive の root の読取りだけ）→ 新しく受領する資料の `first_scan_id` にその行の ID を書く → 移動元（`XZ`）を先に列挙し、窓の中の全ての資料を受領・対応づけする → 終わりに識別子を取り直し、始めの値と同じなら `window_from`・`completed_at` を書いて完了にする（違う・取れないときは未完了のまま）→ 実測の保存で読取りの最大 ID を cursor に採る → 取込みの分類が、資料と直前の精算の資料の `first_scan_id` が指す完了した行と、それより前の同じ識別子の完了した行を読む
- invalid input: 未完了の行、`first_scan_id` が NULL の資料・未完了の行を指す資料・指す行が無い資料、識別子の違う行の組、形の違う日付、比べられない settlement_no は、条件不成立（判定不能）
- compatibility: 既存の表の列は変えない。時点証拠 schema は migration の registry に未登録（D-109）なので、登録の前に足せば既存の DB の移行は要らない

## Test Plan

Test Design Matrix: [2026-10-11-pos-series-design](test-matrices/2026-10-11-pos-series-design.md)（runtime の lane が実装する test の設計。本 lane は docs の検査だけ）。

- targeted tests: 本 lane は `bash scripts/doc-consistency-check.sh --target plan` と full。
- negative tests: Matrix の Negative Paths（runtime）。
- compatibility checks: D4 の既存の分類（NeverObserved・LegacyObserved・Before・Unknown）と flag の理由が変わらないこと（Matrix）。
- data safety checks: 本 lane の差分に実データ（JAN・商品名・金額・新しい件数・発言の原文）を入れない。
- main wiring/integration checks: runtime の lane。

## Review Focus

- Ordinary Operation の列が、正常な条件で目的（数えた商品を数え直さずに置けば、数えた日の精算と次の精算を過ぎた後は数え直しなしで減る）を達成できるか。限界（数え直しのたびの 2 つの精算、ファイルで選んだ日、読取りが失敗した日、EJ の対応づけが確定しない間）が、保証の形で書かれずに全部並んでいるか。
- D10 の条件が、数える前の販売を含む資料を全部実測後と判定する入力・順序を持たないか（行を置く読取り、読取りの排他、読取りの行を列挙の前に置く規則、読取りを完了にする条件〈列挙の順、終わりの識別子の取り直し〉、読取りの途中の媒体の差替えとレジスターツールの取込み、資料の初回受領の読取り、保存の時点の cursor、直前の精算の資料がちょうど 1 つ、窓の日付、資料と直前の精算の資料の両方の初回受領、媒体の入替、ファイルを選ぶ経路、読取りの途中の別の経路の受領、途中で失敗した読取り、締められなかった日、後から受領した古い資料）。
- D5a の骨（終端の候補の抽出、候補が 1 件であることだけでは確定しない、評価の順）が、未決が決まるまでの間、誤った区間を完全にする経路を持たないか。日時を候補の抽出のほかに使っていないか。未決の一覧と、決まるまでの帰結が正本の全部で同じか。
- D5a の区間の終端・開始・欠けの定義が、ADR D4・D5 の既存の分岐（EJ 待ち・登録の変化・前回を証明できない・開始の証拠がない）と矛盾しないか。
- 観測と書いた前提が、tracked の出典（`docs/backlog.md` の入力 1・2、29 の構造所見、D-103・D-105・D-111）で裏づくか。観測を gate の通過と書いていないか。
- D10 の「有効にする・外す」が 1 通りに読めるか（決める所、有効にする migration、無効の間の保存とその間の行と値を使わないこと、保存の不具合が分かったときの出口、不変の例外、外したときの適用済みの取込み）。新しい設定・flag・画面を足していないか。
- core の名前（表・列・用語）に媒体やレジの名前が残っていないか。名前を変えて意味が曖昧になっていないか。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| SPEC-STK-TIME-D10（考え方・全数読取り〈完了にする条件: 列挙の順・識別子の取り直し〉・行を置く読取り・読取りの排他・置き場の識別子・資料の初回受領の読取り・実測側の保存・全部実測後の条件・限界・読取りが途中で失敗した日・締められなかった日と媒体の入替・読取りの間の置き場の変化・外部前提・有効にする・外す〈有効にする migration・保存の不具合の出口・不変の例外〉・番号印字は採らない） | ADR D10 | 本 lane は設計の記録。runtime は接続の runtime lane と ㉘ ④ か ⑤ | runtime（Matrix の D10 の行） | L3: ADR の「㉘ への引継ぎ」の合格条件 |
| SPEC-STK-TIME-D5a（番号の比べ方・区間の終端の候補・候補が 1 件であることだけでは確定しない・区間の開始・区切りにしない記録・欠け・読めない記録・評価の順・日時・「前回」の候補・同一性の比較と系列・後続の設計へ送った項目・決まるまでの帰結） | ADR D5a | 本 lane は設計の記録。送った項目は日次の EJ 取込みの lane の設計、runtime はその lane と ㉘ ④ | runtime（Matrix の D5a の行。送った項目の分は Residual Test Gaps） | 非対象 |
| SPEC-STK-TIME-D4（判定の表に全部実測後の行、相殺の判定の対象の範囲〈行の前後を見る判定は判定不能の行だけ、clear 行の規則と前回の code が分からない区間の規則は全部実測前でない数えた商品の全て〉、clear 行の箇条の事実の更新と code の単位、再評価で欠けと対応づけの確定を前回の証明より先に見る順、対応づけが確定しない区間を EJ 待ちに含める文。出口・flag・解消は変えない） | ADR D4 | 同上 | runtime（Matrix の D4 の行） | 非対象 |
| SPEC-STK-TIME-D5（外部前提を観測と未観測に分ける。合計の照合の除外の理由、戻りの箇条の文、完全な EJ の条件の括弧と帰属できない行の判定の時点を D5a・D4 の文に合わせる言い換え。条件そのものは変えない） | ADR D5 | 本 lane の docs | 本 lane の AC5・AC6 | 非対象 |
| SPEC-STK-TIME-D8（`source_scan_cursor`・`pos_source_scans`・`pos_import_sources.first_scan_id` の行）、SPEC-STK-TIME-D8-K1（6 つ目の項目） | ADR D8、pos-tables・tracking-system-tables の時点証拠契約 | runtime: `schema_time_evidence.rs` | runtime（Matrix の D8 の行） | 非対象 |
| ㉘ への引継ぎ（依存の表、接続の runtime lane の合格条件。TD-223） | ADR ㉘ への引継ぎ | 後続の lane の起票 | runtime（Matrix の通しの行） | L3: 通しの筋書き |
| BIZ-03 の分類の手順 4 と入力（資料の受領が初回受領の読取りを書くことを含む）、手順 6 の対応づけと評価の順の文・clear 行と区間全体の対象の文、外部 probe 表の「精算の書込み」の行と観測の段落 | 32 時点証拠契約 | runtime: `biz/csv_import_service/time_evidence.rs` | runtime（Matrix の D10 の行） | 非対象 |
| IO-08-D9（IO が判断しないことの表の 3 行）、IO-08-D10（欠けの定義の箇条、後続の設計が先に決めること、小数の数量の取引） | 29 の EJ parser IO-08.9・IO-08.10、io-task-specs の IO-08 | 変えない（parser の出力は今のまま。責務の持ち主の記述だけ） | 既存の parser の test を変えない | 非対象 |
| BIZ-08-D3 の引継ぎ（Z004 を SD から読む経路が D10 を満たす。行を置く root、読取りの排他、窓の始まり、列挙の順、識別子の取り直し） | 37 §37.9 の末尾 | 接続の runtime lane | runtime | 非対象 |
| D-116 | `docs/decision-log.md` | 本 lane の記録 | 本 lane の AC3 | 非対象 |
| BIZ-06 の保存の手順 5（`source_scan_cursor` を保存 TX で採る 1 文）と、索引の範囲の語（D1〜D10） | 35 の時点証拠契約、`docs/DB_DESIGN.md`・`docs/ARCHITECTURE.md`・`docs/FUNCTION_DESIGN.md`・biz-task-specs の時点証拠契約の冒頭 | runtime: `biz/stocktake_service/time_evidence.rs` | runtime（Matrix の D10-3） | 非対象 |
| 隣接の除外: SPEC-STK-TIME-D1（S / E の 1 文の言い換えだけ。context・保存・再送の契約は変えない）・D2（`source_cursor` を保存時に取り直さない）・D3（同一性 guard と拒否）・D6・D7・D9 | ADR | 変えない | 既存（㉘ ②・③ の test） | 非対象 |
| 隣接の除外: SPEC-STOP-D1〜D6（停止）、IO-09-D1〜D4・BIZ-08-D3〜D6（SD の読取りと日報）、IO-08-D1〜D8（EJ parser の文法と型）、SPEC-Z4A-D8（D-103） | 停止 ADR、29 の日報 parser、37、29 の EJ parser、23 | 変えない（参照だけ） | 非対象 | 非対象 |

## Contract Probe

本 run は実機・持ち帰りデータの新しい集計を行っていない。観測済みは tracked の出典の引用、未実測は確かめ方を書く。P1・P3・P5・P7 は持ち帰りデータの手元集計（2026-10-11、独立の数え直し済み。出力は件数・日付・番号だけ）の結果で、数は `docs/backlog.md` の同項目の入力 3 に置いた（`rg -n '入力 3（持ち帰りデータの手元集計 2026-10-11' docs/backlog.md` の行）。P2・P11〜P14 は持ち帰った Z004 の中身の手元集計（2026-10-11、独立の数え直し済み。数え直しの側の出力を正とした）の結果で、数は入力 4 に置いた（`rg -n '入力 4（持ち帰った Z004 の中身の手元集計 2026-10-11' docs/backlog.md` の行）。

- P1 商品別売上の資料（Z004）が置き場へ書かれるのは、その精算の操作の時点である（D10 の根拠）: 正常に保存された精算では観測した。2022-08-13 以降の完了した日計明細・ＰＬＵの精算票の全てが SD の Z004 の更新時刻の順位と 1 対 1 で、記録の日付と path の日付が全て同じ、更新時刻は記録の時刻の後で、後の精算でまとめて書かれた例は無い（入力 3 の (b)。`rg -c '後の精算でまとめて書かれた例は 0' docs/backlog.md` が 1）。完了した精算票の SD カード保存の状態は全て正常終了で、異常終了した精算の Z004 の残り方は資料に例が無く確かめられない（入力 3 の (c)）。店には SD カードのエラーで締められなかった日がある（repo 外の回答台帳 TD-160・TD-161）が、その日が精算をせずに終わった日か、保存だけ失敗した日かは分からず、資料の中の異常終了の行はスマートフォン送信と設定の書込みだけで、売上の保存の異常終了は無い。-> 正常な保存は観測、異常終了は未観測（確かめられない）。設計への効き方: 異常終了の項目は 32 の外部 probe 表の「精算の書込み」の行に残り、D10 を有効にする変更の owner-operated gate で扱う。証拠を得る手段（説明書・製造元の記載、実機）が無ければ、その変更の lane が残る危険の受け入れを owner に諮る。確かめるまで枝を入れない
- P2 資料の個数・金額は、直前の精算より後の販売・返品だけを持つ: 観測した（強さに限りがある）。EJ の区間の名称ごとの符号付き合計と Z004 の行を比べられた区間の全てで個数・金額が一致し、区間の合計と EJ の先頭からの累積が違う値になって差分か累積かを区別できる組は、全て区間の合計の側と一致した。2 つが同じ値で区別できない組もある（入力 4 の (a)。`rg -c '「累積と合う組は無い」とは言えない' docs/backlog.md` が 1）。PLU を 1 件登録して売ると個数の列に出ること、締めの後の新しい Z004 が開始時と整合したことは店の確認（repo 外の回答台帳 L-012・L-030）。-> 観測（PLU で売った実例が少ない。owner-operated gate の通過ではない）。ADR D5 の「EJ完全性」の owner-operated gate と同じ確認で、D10 を有効にする変更の merge 条件に残す（32 の外部 probe 表）
- P3 Z004 の path の日付（年・月の folder と名前の日）は、中身の精算日と同じ: 観測した。手元に中身のある Z004 の全てで同じ（入力 3 の (b)。`rg -c 'path の日付 = 中身の精算日' docs/backlog.md` が 1）。日付をまたぐ時刻の精算は資料に無く、その場合のずれは未観測。-> 観測（日付をまたぐ精算は未観測。D10 の条件 3 を「窓の始まりの翌日以降」にして 1 日のずれを受ける）
- P4 置き場の file を人が足さない・消さない・書き換えない: 店の運用の前提として ADR D10 に明記。改名しないこと（レジスターツールの取込みによる `XZ` から `XZ_BKUP` への移動と改名を除く）と、読取りの間に置き場を変える操作（媒体の抜き差し、レジスターツールの取込み）をしないことを含む。読取りの間に媒体を差し替えた読取りは、終わりに取り直した識別子で完了にならないが、差し替えて元の媒体へ戻した列は見分けられず、この前提だけが守る（下の列 18）。この前提は店の運用の決まりで、32 の外部 probe 表の gate の項目には入れていない。SD を別の 1 枚に替えること・古い SD を後から読むこと・ファイルを選ぶ経路で取ることは、この前提に含めない（D10 の条件 3 がどれも判定不能に倒す。下の入力列）。SD の写しを取外し可能な媒体に置いて差すことは、この前提の違反に当たる（アプリはその媒体をレジの置き場と区別できず行を置くので、D10 は守れない。古い写しの後に新しい写しを置く形で、数える前の販売を含む資料が全部実測後になりうる。接続の runtime lane の L3 の筋書きの候補）。写しの folder を利用者が選んで読む復旧は、行を置かないので判定不能に倒れる。-> 前提
- P5 精算票と Z004 の対応: 起票時に観測済みとした「題の末尾 + 1 = Z004 の settlement_no」は、0011 以降では成り立つが規則としては使えないと分かった。Z004 の settlement_no は Z004 を書いた精算の通し番号で、題の末尾とは別の番号で進み、差はＰＬＵだけの精算のたびに変わる（入力 3 の (a)。`rg -c '対応づけに番号の差を使わない' docs/backlog.md` が 1）。使い始めの 0001〜0010 は規則が外れ、うち 9 個は別の記録に 1 件として当たった。代わりに観測したのは、Z004 の中身の時刻が精算票の記録の時刻と同じ分か 1 分後であること（手元に中身のある Z004 の全て。入力 3 の (b)）。日時のほかに、2026-10-15 の手元集計（別の担当の数え直しで一致。repo 外の回答台帳 TD-252〜TD-254）で、Z004 → 同じ精算の Z001 → 精算回数 → 題の末尾が等しい完了した日計明細（ＰＬＵだけの精算は題の末尾 = settlement_no の完了したＰＬＵの精算票）の鎖が、手元に中身のある Z004 の全てで精算票をちょうど 1 件に決め、日時で結んだ相手と全て同じだった。弱いのは Z004 を同じ精算の Z001 と組にする所で、Z004 のメタは Z001 の精算回数を持たず、手掛かりは Z 帳票の時刻か SD の file 名である（数は backlog の入力 5。`rg -n '入力 5（持ち帰りデータの手元集計 2026-10-15' docs/backlog.md` の行）。-> D5a の終端の候補を日時で抽出する形にし、候補が 1 件であることだけでは対応づけを確定しない。鎖を採るかと Z004 を Z001 と組にする方法は、D5a の未決 (1) に含める。確定の条件は日次の EJ 取込みの lane の設計が決める（未決）。決まるまで、どの区間も EJ 待ち（数え直し）に倒れる
- P6 EJ の 6 桁の番号は記録ごとに +1 で file と精算をまたいでも戻らない、記録ヘッダの 4 桁の欄 = machine_no: 観測済み。出典 `rg -n '全期間 8,624 件が \+1 で連続' docs/backlog.md` の行と `rg -n '6 桁の番号は記録ごとに \+1' docs/function-design/29-io-ej-parser.md` の行、`rg -n 'EJ の見出しの 4 桁 = machine_no' docs/backlog.md` の行。-> 観測
- P7 ＰＬＵの精算票: 完了したＰＬＵの精算票は Z004 を 1 本ずつ書き、題の末尾は対応する Z004 の番号と同じと読めた（入力 3 の (a)、SD の一覧の順位で 3 件で、最後は 2022-08。中身で確かめた数は入力 5、2026-10-15、repo 外の回答台帳 TD-252〜TD-254。規則として確かめるには足りない）。D5a は番号の差を使わず、ＰＬＵの精算票も日計明細と同じく日時で終端の候補にする（相手がＰＬＵのものも日時が合った。`rg -c 'Daily. か .Plu' docs/adr/2026-09-18-stocktake-time-evidence.md` が 1）。-> 観測（母数が小さい）。外れれば候補が無く、EJ 待ちに倒れる
- P8 番号印字の残る桁数・行数の内訳・取引の途中で打ったときの位置: 番号印字を採らない（TD-224）ので確かめない。EJ に残り売上・現金・Z 帳票に影響しないことは観測済み（Issue #105 の A7、`rg -n '番号印字（EJ の記録と取引番号・時刻との並び）= 2026-09-28 確認済み' docs/backlog.md` の行）。-> 不要
- P9 IO-09 が窓の始まりの日付で SD の entry を列挙する設計である: `rg -c 'fn list_register_sd_entries' docs/function-design/29-io-daily-report-parser.md` が 1（実測、2026-10-11）。runtime は未実装（`src-tauri/src/io/` に `register_sd.rs` が無い）。-> D10 は窓つきの読取りで成り立つ形（条件 3 の `window_from`）にした
- P10 置き場の識別子（SD のボリュームの通し番号）を読取り専用で取れ、同じ SD では読取りをまたいで同じ、別の SD では違う: 取得の API を含む feature が依存にあること（`rg -n 'Win32_Storage_FileSystem' src-tauri/Cargo.toml` が 1 行、実測 2026-10-11）だけ確かめた。店の reader と SD での値の安定は未実測。設計の `RegisterSdRoot` は識別子を持たない（`rg -n 'volume_label' docs/function-design/29-io-daily-report-parser.md` の行は表示用の名前）。IO-09 の設計に識別子の取得は無く、IO-09-D3 の許可の列の `windows_sys` の関数は `GetLogicalDrives`・`GetDriveTypeW` だけである（`rg -c 'GetVolumeInformation|通し番号|medium_id' docs/function-design/29-io-daily-report-parser.md` が hit なし、実測 2026-10-12）。`XZ` と `XZ_BKUP` の列挙の順も決まっていない（同 file の §29.7.5 手順 2 は「`XZ` と `XZ_BKUP` のそれぞれで」）。どちらも接続の runtime lane が IO-09 に足す。店の PC の reader で SD が取外し可能な volume として見えたことは観測済み（`rg -c '^\| SD-25 \| 店の PC の reader で SD は取外し可能な volume として見え' docs/function-design/29-io-daily-report-parser.md` が 1、実測 2026-10-12）。SD が固定の disk として見える reader の PC では、自動で root を見つけず行が置かれないので、D10 は働かない（要再確認に戻る）。-> 未実測（接続の runtime lane の Contract Probe と L3。取れない間は行を置かず、何も全部実測後にならない）
- P11 PLU で売った分は部門別の集計（Z005）にも入る: 観測した。Z005 の個数の合計は同じ区間の EJ の全明細の符号付き合計と手元の全てで一致し、Z004 に 0 でない行がある区間では、PLU の行を除いた EJ の合計とは一致しない（入力 4 の (c)）。-> 観測。Z004 の商品別の売上と日報の部門別の合計を足し合わせない（既存の「sale_records と日報の分離」のまま。設計は変えない）
- P12 部門名と同じ名称の PLU の枠がある: 観測した。EJ の明細の行にコードは印字されず、名称だけでは PLU の販売と部門キーの販売を分けられない。その名称の EJ の明細の大半は、Z004 の行が 0 の区間（部門キーの販売）にある（入力 4 の (d)）。効く所は ADR D5 の商品別の符号付き合計の照合と行の分類で、その名称を含む単位は照合せず、その名称の行が区間に 1 つでもあれば、その名称の数えた商品（数量・金額とも 0）が相殺の行ありになる。-> 観測。見逃さない側（数え直しが増える側）に倒れる。名称の重なりの検知と登録名の付け直しの支援を PLU の本番より前に要る（ADR D4 の在庫連動の有効化の条件。変えない）
- P13 小数の数量の取引: Z004 の個数の欄と EJ の数量の行に小数の値がある（入力 4 の (e)）。今の EJ parser は小数の数量・点数の行を受理しない（`rg -c '小数の数量・点数を受理する' docs/function-design/29-io-ej-parser.md` が 1。不採用の表の行）ので、その取引は復元不能で、その取引を含む区間は ADR D5a の読めない記録により完全にならない。-> 観測。後続の lane（単位の拡張の runtime、日次の EJ 取込み）への申し送り。それまで EJ 待ち（数え直し）に倒れる
- P14 値引きの掛かった PLU の明細と、返品の取引でない取引の中の戻の印の付いた PLU の明細: 資料に現れていない（入力 4 の (b)。返品の取引の PLU の明細は、正味の符号が Z004 の行と一致した）。-> 未観測。ADR D5 の「まだ観測していない外部前提」に置いた。外れれば合計の不一致として EJ 待ちに倒れる
- 締められなかった日と媒体の入替の入力列（机上で D10 の条件に当てた。精算の番号は例、C は数えた商品の実測の保存）:
  - 列 1（精算をせずに終わった日）: C → エラーで締められず読取りも無い → 後日、精算 m → 全数読取り O（m を受領）→ 精算 m + 1 → 全数読取り O′（m + 1 を受領）→ 精算 m + 2 → 全数読取り O″（m + 2 を受領）。結果: m は P（m − 1）の初回受領が C の保存より前の読取りで条件 3 不成立、m + 1 は P（m）の初回受領が O で、それより前に C の保存より後の完了した読取りが無く不成立、m + 2 は P（m + 1）が O′、自分が O″ の中で初めて受領されて成立。締められなかった日の販売は m に入り、判定不能（要再確認）
  - 列 2（読めない資料が窓に残る）: 読取りは完了せず行は未完了のまま。完了した行が 2 つそろうまで何も成立しない
  - 列 3（SD を替え、古い SD の未読の資料を後からファイルで取る）: 古い SD で精算 n − 1（未読）→ SD を替える → 販売 → C → 新しい SD で精算 n → 読取り O（n を受領）→ 精算 n + 1 → 読取り O′ → 後日 n − 1 をファイルを選ぶ経路で受領 → n を取り込む。結果: P（n − 1）はどの読取りの中の受領でもない（`first_scan_id` が NULL）ので条件 3 不成立、n は判定不能。起票時の形（読取り 1 つの後に初めて受領したことだけを見る）では、この列で n が全部実測後になり、C の前の販売を二重に引いた。条件を、同じ置き場の完了した読取りの中の初回受領へ改めた理由
  - 列 4（古い SD を後から読む）: C → 古い SD の全数読取り O（識別子 A）→ 新しい SD の読取り（識別子 B）で P を初めて受領。結果: 識別子が違うので組にならず不成立
  - 列 5（精算は済んで保存だけ失敗し、後の精算でまとめて書かれる。P1 が崩れる場合）: 精算 n（保存に失敗）→ 販売 → C → 読取り O → 精算 n + 1 で n の資料も書かれる → 読取り O′ で n を初めて受領。結果: n + 1 が成立し、C の前の販売を含む。D10 の条件では防げないので、P1 を確かめるまで枝を入れない（ADR D10 の外部前提と「有効にする・外す」）
  - 列 6（SD を読めずファイルで選んだ日）: C は 10 日前、毎日 SD を読んでいる → 今日だけ SD を読めず、今日の精算 F をファイルを選ぶ経路で受領 → 翌日の精算 F₂ を読取りで受領 → その次の精算 F₃ を読取りで受領。結果: F は `first_scan_id` が NULL で不成立、F₂ は P（F）が NULL で不成立、F₃ は成立。F だけを見れば全部実測後としても誤らないが、F の経路を問わない形は列 8 を通すので、両方に条件を掛け、2 つの精算の資料の数え直しを費用として受ける
  - 列 7（読取りの途中の別の経路の受領）: 古い媒体 B に C より前の精算 P・F の資料がある → C → 媒体 A の全数読取り O → A の読取り O′ の途中で、B の写しから P をファイルを選ぶ経路で受領 → O′ 完了 → F を受領。結果: P の `first_scan_id` は NULL（読取りが自分の行の ID を渡した受領でない）で不成立。資料の ID が読取りの開始と完了の間にあることで所属を推す形では、この列で F が全部実測後になり、C の前の販売を二重に引いた。受領した読取りを資料に直接書く形へ改めた理由
  - 列 8（番号の reset の前の古い資料を後から受領）: 古い系列の精算 n（C より前の販売を含む。未受領）→ reset・媒体の入替 → C → 新しい媒体の全数読取り O → 新しい系列の精算 n − 1 → 読取り O′ で初めて受領 → 古い系列の n の資料を、ファイルを選ぶ経路か古い媒体の読取りで受領。結果: F（古い n）は `first_scan_id` が NULL か、O と別の置き場の読取りを指すので不成立。同じ媒体に残っていた場合は、精算日が O の窓に入れば O までに受領済みで O より後の初回受領にならず、窓より前なら日付の条件で不成立
  - 列 9（読取りが途中で失敗した日）: 読取り S が精算 k の資料を受領した後に失敗 → 同じ日に読み直して完了 → 翌日以降は毎日完了。結果: k は自分の初回受領が未完了の S で不成立、k + 1 は P（k）が未完了の S で不成立、k + 2 は成立
  - 列 10（連日売れる商品を毎日数え直す）: C₀ → 精算 m・読取り・取込み（判定不能）→ 数え直し C₁ → 精算 m + 1・読取り・取込み → 数え直し C₂ → 精算 m + 2・読取り・取込み。結果: m + 1 は C₁ から見て 1 つ目、m + 2 は C₂ から見て 1 つ目の精算で、どれも不成立（各 C の `source_scan_cursor` が直前の読取りの ID 以上になり、それより後の完了した読取りが 1 つしか無い）。数え直さなければ m + 2 は C₀ に対して成立する。限界として ADR D10 に書いた（TD-228）
  - 列 11（DB の restore）: C → 読取り O → backup → 精算・読取り → restore → 読取り。結果: 資料・読取りの行・実測が backup の時点へ戻り、restore の後の読取りで受領する資料はその読取りを初回受領にする。戻した DB の O の時点でその資料が置き場に無かったことは変わらないので、条件が成り立つ資料は実測の後の精算のもの。backup の後に保存して失われた実測は基準にならない（DB に無い）
  - 列 12（重なる全数読取り。排他が無い場合の列）: 精算 P → 販売 → C → 精算 F（C より前の販売を含む）→ 読取り O が行を置く → 別の読取り O′ が行を置き、P・F を受領して完了 → O が P・F を受領済みと対応づけて完了。結果: `C.source_scan_cursor < O.id < O′.id`、P・F の `first_scan_id` が O′ で条件 3 が成り立ち、F が全部実測後になる。読取りの排他（行を置く前から完了・失敗まで、同じ DB に対する唯一の操作）により、O′ は O の完了・失敗まで行を置けず、この列は起きない。契約として ADR D10 に書いた理由
  - 列 13（利用者が場所を選んだ root）: 精算 41（未読）→ 販売 → C → 古い写しの folder（41 より前の写し）を場所を選んで読む → 精算 42（C より前の販売を含む）→ SD を写した別の folder を場所を選んで読み、41 と 42 を初めて受領。結果: 選んだ root の読取りは行を置かないので、41・42 の `first_scan_id` は NULL で不成立。選んだ root で行を置く形では、2 つの folder の識別子がどちらも PC の disk の値で同じになり、条件 3 が成り立って 42 が全部実測後になった
  - 列 14（保存の不具合で、指す行が無い先の ID を持つ `first_scan_id`）: P・F の `first_scan_id` が 4・5 で、実在する読取りの行の最大 ID は 2 → 不具合を直す変更の migration（全ての実測の cursor を 2 へ、全ての `first_scan_id` を NULL へ）→ 読取り 3・4・5 が完了。結果: P・F は NULL で不成立。cursor だけを直す形では、行 3・4・5 がそろった時点で `2 < 3 < 4 <= 5` が成り立ち、F が全部実測後に戻った
  - 列 15（D10 を有効にした直後）: 有効にする migration → 精算 a・読取り → 精算 b・読取り → 精算 c・読取り。結果: a は直前の精算の資料が NULL で不成立、b は a の初回受領の読取りより前に、migration より後の完了した読取りが無く不成立、c は成立。migration の直後に読取りを一度済ませていれば b から成立
  - 列 16（SD を 2 枚交互に使う）: 毎日 2 枚を入れ替える。結果: 直前の精算の資料と当の資料の初回受領の読取りの識別子が常に違い、どの資料も不成立（安全側。店がそうしているかは確かめていない。本 lane は決めない）
  - 列 17（読取りの途中の媒体の差替え）: SD A に C より前の精算 P・F の資料がある（未読）→ C → 読取り O が A の識別子を取って行を置く → 列挙の前に空の SD B へ差し替える → O が B を列挙し、窓に資料が無いまま受領を終える → 終わりに識別子を取り直すと B の値（A と違う）。結果: O は完了にならず、行は未完了のまま残る。A を戻して読む次の読取り O′ が P・F を初めて受領しても、C の保存より後の完了した読取りが O′ より前に無く、F は不成立。終わりの取り直しが無い形では、O が A の識別子の完了した行になり、`C.source_scan_cursor < O.id < O′.id`、P・F の `first_scan_id` が O′ で条件 3 が成り立ち、F が全部実測後になった。差替えが列挙の途中・後でも、終わりの値が B なので同じ。B が別の drive として見えれば path が無くなり、読取りは失敗する
  - 列 18（差し替えて元の媒体へ戻す。閉じない列）: 列 17 で、O が B を列挙して受領を終えた後、終わりの取り直しの前に A へ戻す。結果: 始めと終わりの識別子がどちらも A で、O は完了する。次の読取り O′ が P・F を初めて受領すると条件 3 が成り立ち、F が全部実測後になる。識別子の取り直しでは見分けられず、D10 の外部前提（読取りの間に置き場を変える操作をしない）だけが守る。限界として ADR D10 に書いた。adapter が読取りの間じゅう媒体を掴む形で閉じられるかは、接続の runtime lane が Windows の実機か公式の資料で確かめる
  - 列 19（読取りの途中のレジスターツールの取込み）: `XZ` に C より前の精算 P・F の資料がある（未読）→ C → 読取り O が行を置く → 列挙の間に、レジスターツールが P・F を `XZ` から `XZ_BKUP` へ移す。結果: `XZ` を先に列挙する順では、(i) `XZ` の列挙より前に移れば `XZ_BKUP` の列挙に現れて受領される、(ii) `XZ` の列挙に現れた後で読む前に移れば、読めずに読取り全体が失敗して未完了、(iii) 読んだ後に移れば受領済み。どの場合も、P・F を受領しないまま O が完了することは無い。`XZ_BKUP` を先に列挙する順では、`XZ_BKUP` の列挙の後・`XZ` の列挙の前に移った P・F がどちらにも現れず、O が完了して、後の読取りで初めて受領された F が全部実測後になった。移動が一方向（`XZ` → `XZ_BKUP`）であることは、29 の日報 parser の SD-23（静的解析による推定）と SD-02・SD-11・SD-22（取込み後の配置の観測）からの推定で、逆向きの移動が無いことは実機で確かめていない。推定が外れた場合は、外部前提（読取りの間に置き場を変える操作をしない）が覆う
  - 列 20（保存の不具合で、最大 ID より大きい cursor）: 読取りの行の最大 ID は 2、ある実測の `source_scan_cursor` が 99（保存の不具合の値。非負の CHECK は通る）→ 不具合を直す変更の migration（全ての実測の cursor を 2 に置き換え、全ての `first_scan_id` を NULL へ）→ 読取り 3・4・5 が完了し、4 と 5 で直前の精算の資料と当の資料を初めて受領。結果: `2 < 3 < 4 <= 5` で成立し、有効にしたときと同じ待ちで戻る。値を大きくする向きにだけ動かす形では、99 が残り、読取りの行が 99 を越えるまでその商品は成立しなかった。置き換えた後の O は migration より後の行に限られ、既存の資料の `first_scan_id` は全て NULL なので、値が下がった実測でも、migration より前の行と資料は根拠にならない
- D5a の終端の候補の入力列（机上で骨に当てた。番号と日付は入力 3 の集計のもの。候補をひとつに絞って確定する条件は決めていないので、結果は「候補に入るか」までを書く）:
  - 使い始めの 0001〜0010: 番号の規則では 0001 に当たる記録が無く、0002〜0010 は別の記録ひとつに当たる。日時による抽出では、集計が「実際の相手」とした記録（日付と時刻が合う記録）が候補に入り、題の末尾の番号が合うだけの別の記録は入らない
  - Z004 を書かなかった精算（使い始めの日計明細。入力 3 の日付ごとの件数が違う日）: どの Z004 の日時とも合わず、候補にならない
  - ＰＬＵだけの精算がもう一度ある（番号の差が変わる）: ＰＬＵの精算票が自分の Z004 の候補になり、後の日計明細の精算票は自分の Z004 の候補になる。番号の差に依らないので、1 つ後の記録を拾わない
  - 中断と再実行（同じ題の末尾の記録が続く）: 中断した精算票は完了でないので候補に入らず、再実行の完了した精算票だけが候補
  - 勤怠の精算票が同じ頃にある: 帳票の種類で候補に入らない
  - レジの時計が戻った箇所（2024-11）: 記録の日時も Z004 の中身の日時もレジの時計に従うので、相手の記録は候補に入る。戻した先に同じ日時の完了した精算票がもうひとつあれば候補が複数になる（どう扱うかは未決）
  - 本当の精算票の記録が未取込みで、時計を戻した後に同じ日時になった古い完了した精算票だけが取り込んだ EJ にある（合成の列）: 古い記録だけが候補になる。候補がひとつであることだけで確定すると、古い区間（欠けが無く、対象の商品の行が無い）を完全として採り、売れて戻った商品を見逃す。骨はここで確定しない（EJ 待ち）。これを落とす整合の条件が未決の (1)
  - n − 1 の精算票を含む EJ の記録が足りない: 終端より前で最も近い完了した精算票が n − 2 の終端になり、前回を証明できない形に見える。評価の順（欠けを先に見る）により EJ 待ちで止まり、足りない記録の取込みで解ける。登録の変化（数え直しでしか解けない）には落ちない
- D10 の条件 3 の日付の入力列（机上）:
  - レジの時計が戻った日に精算した資料（P でも F でも同じ）: 精算日が窓の始まりより前になれば条件 3 が成り立たず、その資料と次の資料は判定不能。精算日が窓の中なら、O の時点で置き場にあれば列挙されて受領されているので、「O に無く O′ にある」は時計に依らず正しい
  - 時計が進められていた間の資料: path の日付が先の日付でも、窓は始まりだけを持つ（終わりを持たないと ADR D10 に書いた）ので列挙に入り、受領される。窓に終わりを付ける実装では、この資料が列挙されず、後の読取りで「置き場に無かった」と数えられる（Matrix の D10-22）
  - 日付をまたぐ時刻の精算（未観測）: path の日付が中身の精算日の前日で、窓の始まりの前日に当たると、置き場にあっても列挙されない。起票時の条件（精算日が窓の始まり以降）では、この資料が後の読取りで初めて受領されると「置き場に無かった」と数えた。条件を「窓の始まりの翌日以降」に改め、この列を成り立たなくした
  - 窓の始まりを読取りの当日にする: 同じ日の O より後の精算の資料の精算日が `O.window_from` の翌日以降にならず、その資料と次の資料が不成立になる（安全側。同じ日に 2 回精算する日のたびに要再確認が増える）。ADR D10 は窓の始まりを読取りの日の前日以前と決めた。37 §37.9 手順 2 の窓は最後の日報取込みの日付と今日の 30 日前の早い方で、常に 30 日以上前（`rg -c 'その `report_date` の最大と `today` − 30 日の早い方' docs/function-design/37-biz-daily-report-import-service.md` が 1、実測 2026-10-12）
  - 窓の始まりを決めるアプリの時計が狂っている: 窓が動くだけで、条件 3 は列挙の範囲と同じ値で比べるので、列挙されなかった資料を根拠にしない
- 観測済みで probe の要らない前提: SD の原本・取込み後の file・PC 側の複製が同じ bytes（29 の日報 parser の SD-22 の観測と SD-23 の推定）、戻の行に商品名が載る・締めで同じ日の EJ を SD から取れる・レジが clear を受け付ける（`docs/backlog.md` の同項目の 2026-10-06 の再確認）、clear した枠の行の残り方（D-103）。

## Data Safety

- tracked に書かない: 実 SD・EJ・Z 帳票の中身、実 JAN・商品名・部門名・金額、独立に数え直していない件数（件数・日付・番号は、独立に数え直した集計の出力にあるものだけを `docs/backlog.md` の同項目の入力 3〜5 に置き、ほかの file はそこを参照する）、店主・owner の発言の原文、repo 外の調査・相談の path の詳細。店の事実は「repo 外の回答台帳 TD-xxx / L-xxx」の参照と要旨で書いた。
- local-only: repo 外の回答台帳・外部設計相談の記録・持ち帰りデータと集計。
- synthetic-only: runtime の lane の test は合成の資料（合成の Z004・EJ の bytes、合成の SD の tree）だけを使う。

## Implementation Results

- 結果: 実測の後に行われた精算を、資料の置き場の全数読取りの順で見分ける規則（時点証拠 ADR SPEC-STK-TIME-D10。判定は外部前提の確認が済むまで有効にしない）と、EJ の区間の対応づけの骨・欠け・「前回」の候補（D5a）を設計正本に置き、D-116 に記録した。番号印字は採らない（repo 外の回答台帳 TD-224）。区間の終端の対応づけを確定する条件の細部は日次の EJ 取込みの lane の設計へ送った（TD-230）。本 lane は docs だけで、code・migration・bindings は変えていない（runtime は後続の lane）。
- 既存の保護: 実装は変えていないので、店の動作は今のまま（棚卸しは停止のまま、Z004 の在庫連動は今の運用）。D10・D5a は proposed・未実装。
- 未検証: Contract Probe の外部前提のうち未観測のもの（保存が異常終了した精算の資料の残り方、P2、P10）は、D10 を有効にする lane の owner-operated gate に置いた。
- review・CI・merge（closeout、2026-10-15）: Plan Review は round 3（上限）までで、round 3 で Codex 側に P1 が残り、plan-gate での事実の訂正（Z004 と精算票を結ぶ観測を、同じ精算の Z001 の精算回数を介した番号まで広げた）と再確認の後、owner の承認で plan-approved（TD-255）。Final Review は broad 3 回（下の Review Response の Closeout の段落）。helper の record は review = pass。hosted の `Merge gate` は success で、owner の Ready・merge の承認（TD-257）の後に helper 経由の squash merge。PR: [#159](https://github.com/higekiri-w/inventory-system-desktop/pull/159)
- 後続: runtime の依存は ADR の「㉘ への引継ぎ」の表。Final Review broad 3 の P3 と後続の lane への申し送りは `docs/backlog.md` の「実測とPOS系列の対応を取得・保存する」の項目の「㉘ ④ と日次の EJ 取込みへの申し送り」と、その下の正本の文の整理の follow-up。次は owner の方針（TD-258）で、棚卸しの時点証拠と POS の取込みの領域の設計の見直しを先に行い、日次の EJ 取込みの lane の起票はその後。

## Review Response

### Plan Review round 1（2026-10-11、対象は plan-gate へ進めた版）

- Codex（GPT-6.1 Sol / high）: reject、P1 4 / P2 3 / P3 0。Ordinary Operation は「具体的な反例あり」。Claude 側（fresh Opus 5.5、fork でない）: reject、P1 0 / P2 4 / P3 4。Ordinary Operation は「外部前提が未確認」。
- 裁定（Coordinator）: 2 本の全行を採用した。是正の形は、相談役（Fable 5.1）の反例探しと owner の決定を受けて下のとおり決めた。

| ID | 出どころ | 要旨 | 採否と是正の形 |
|---|---|---|---|
| C1 | Codex（P1） | 数え直しのたびに基準が動き、連日売れる商品は自動へ移れない | 採用。仕組みは変えず、限界として正本・Goal・Ordinary Operation・Matrix に書く（owner が限界を一旦受け入れた: repo 外の回答台帳 TD-228） |
| C2・F1 | Codex（P1）・Claude 側（P2） | 資料の ID の区間は「その読取りで受領した」証拠にならない（読取りの途中の別経路の受領） | 採用。資料に初回受領の読取りの ID（`first_scan_id`）を持たせ、条件はその値を直接見る。`pos_source_scans` の区間の 2 列は外し、完了の印（`completed_at`）に置き換える |
| C3 | Codex（P1） | 日時の合う精算票がただ一つあることは、区間の同一性の証明にならない | 採用。区間の対応づけ（D5a）は骨と、後続の設計へ送る項目の一覧に書き直し、確定する条件の細部は日次の EJ 取込みの lane の設計へ送る（owner の決定: TD-230）。決まるまで、EJ の完全性に依る判定は EJ 待ち・要再確認に倒れる |
| C4・F3 | Codex（P1）・Claude 側（P3） | 番号の reset をまたいで、別の系列の資料が直前の番号で結ばれる | 採用。条件 3 を「直前の資料と当の資料の両方が、同じ置き場の完了した読取りの中で初めて受領された」に直す |
| C5・F4・F8 | Codex（P2）・Claude 側（P2・P3） | 「時刻は対応づけに使わない」と D5a が食い違う、言い過ぎ | 採用。「レジの日時は区間の終端の候補の抽出にだけ使い、欠けの判定と実測との前後の判定には使わない」に統一 |
| C6・F5 | Codex（P2）・Claude 側（P2） | 無効の間の保存の不具合の世代を、後から判定の対象から外す出口が無い | 採用。migration で全ての実測の `source_scan_cursor` を引き上げる出口と、有効にする変更の merge 条件を足す |
| C7・F2 | Codex（P2）・Claude 側（P2） | 当の資料をファイルを選ぶ経路で受領した日の結果が、条件と操作表・Matrix で食い違う | 採用。C4 の条件で決まる（その日の資料と次の精算の資料は判定不能、その次から戻る）。正本・操作表・Matrix をそろえる |
| F6 | Claude 側（P3） | backlog の入力の数が文の中で突き合わない | 採用。集計の出力にある数で突き合わせ、出力に無い内訳は `未実測` |
| F7 | Claude 側（P3） | Matrix の AC の範囲の誤りと、足りない行 | 採用。AC の全部に直し、行を足す |

- 相談役（Fable 5.1）に、是正の前に方針への反例探しと同じ型の残りの洗い出しを頼んだ（合成モデルの総当たりと名指しの列）。採ったもの: `first_scan_id` に置き換えて区間の 2 列を外す形、「Z004 と精算票を結ぶ値は現物では日時しか無い」（起草役が `src-tauri/src/io/z004_parser.rs`・`ej_parser.rs` の型で確かめた）、EJ の評価の順を「欠け → 前回の証明」にすること、限界を保証として書いている所と、直前の資料の経路しか書いていない所の一括の是正。採らなかったもの: 途中で失敗した読取りの中の初回受領を認める緩和（条件を増やさず、限界として書く）。相談役の整合の条件の案（1 記録 1 Z004、番号順の整合）は決定にせず、未決の例として書いた。
- owner の決定（repo 外の回答台帳）: TD-228（限界の受け入れ）、TD-229（番号印字を後から足すなら任意で使える形。D-116 の Revisit に書いた）、TD-230（D5a は骨だけ書き、細部は日次の EJ 取込みの lane の設計へ送る）。
- Goal・Scope・AC の変更（TD-230 と是正による。前半の節に反映済み）: 最小完了条件の 2 つ目を「区間の端を決める」から「骨と未決の一覧を読める。決まるまでの帰結が正本にある」へ縮めた。保存する値に `first_scan_id` を足し、D10 は本 lane で決め切ると書いた。失敗定義に 2 行足した。AC4・AC11 を変え、AC12 の幅の検査を外し（幅は未決）、AC13（D5a の骨）と AC14（限界の段）を足した。Scope の表に `docs/project-memory.md` を足した。
- 起草役が表の指示のほかで決め、Coordinator が確かめて採ったもの: (1) 条件 3 に当の資料の日付（窓の始まりの翌日以降）を足した（窓の外にあって列挙されなかった古い資料が、後の読取りで初めて受領されて通る列を塞ぐ。安全側で、普通の運用の結果は変えない）。(2) 番号の reset を挟む列が 1 つ残る（当の資料の精算の後に reset があり、同じ直前の番号の精算までが、続く二つの完了した読取りの間に起き、本当の直前の資料が窓の外か別の置き場にある形）。系列の識別子を足さない限り塞げず、D-116 は足さないと決めているので、限界として ADR・D-116・Matrix の Residual Test Gaps に書き、reset が実際に起きる前に決め直す（Revisit Trigger）。精算の番号が大きい今のレジでは、reset の後に同じ番号まで精算を重ねる間に一度も読まないことが要る。
- Z004 の中身の集計（独立の数え直し済み）と、交換を差額だけで打つ運用の要旨（TD-225）を、Plan Review とは別に正本・backlog・Contract Probe に入れた。
- 是正は起草役の commit「全部実測後の条件を資料の初回受領の読取りで書き直し、限界と D5a の骨・未決を正本に書く」。設計と Scope を変えたので design へ戻して歩き直し（遷移の記録 8〜10）、round 2 へ進む（round 天井 3）。

### Plan Review round 2（2026-10-12、対象は round 1 の是正と記録の後の版）

- Codex（GPT-6 Astra / high）: reject、P1 2 / P2 1 / P3 0。Claude 側（fresh Opus 5.5、fork でなく round 1 とも別の context）: reject、P1 0 / P2 1 / P3 6。Ordinary Operation は 2 本とも「外部前提が未確認」。2 本とも、読取りが重ならず外部前提が成り立つ範囲では、D10 の条件から誤った全部実測後を出す列を見つけていない（合成モデルの総当たりと乱択）。区間の対応づけを後続の設計へ送ったことと、番号の reset を挟む列を限界に置いたことは、2 本とも finding にしていない。
- 裁定（Coordinator）: 2 本の全行を採用した。どれも「暗黙に置いていた前提を契約として書いていなかった」型なので、条件の式と保存する値を増やさず、範囲を狭める・手順をそろえる・限界として書く向きで直した。

| ID | 出どころ | 要旨 | 採否と是正の形 |
|---|---|---|---|
| R2-C1 | Codex（P1） | 重なる全数読取りが逆順に完了すると、実測前の販売を含む資料が全部実測後になる | 採用。全数読取りは、行を置く前から完了・失敗まで同じ DB に対する唯一の操作、と契約にした（runtime は DB の接続の排他を持ち続ける） |
| R2-C2 | Codex（P1） | 保存の不具合の出口が cursor だけを引き上げ、壊れた `first_scan_id` が残って後日誤った判定が戻る | 採用。下の R2-F4・R2-F6 とまとめて、「有効にする」と「保存の不具合の出口」を同じ migration にした（cursor の引上げと、全ての `first_scan_id` の NULL 化） |
| R2-C3 | Codex（P2） | EJ の再評価の順が、32 の手順の本文と tracking-system-tables で旧い順のまま | 採用。ADR の順（欠け・読めない記録 → 対応づけの確定 → 前回の証明）に同期した（`docs/architecture/biz-task-specs.md` の同じ漏れも直した） |
| R2-F1 | Claude 側（P2） | 利用者が場所を選んだ root（SD の写しの folder）でも行を置くと、別々の写しが同じ識別子になり誤る | 採用。行を置くのは自動で見つけた取外し可能な drive の root の読取りだけにした |
| R2-F2 | Claude 側（P3） | 同じ識別子の別の媒体は、gate の時点の店の SD でしか確かめられない | 採用（条件は足さない）。reviewer の案を接続の runtime lane への申し送りと Residual Test Gaps に候補として置いた |
| R2-F3 | Claude 側（P3） | 窓の始まりを読取りの当日にすると、同じ日の後の精算が成り立たない | 採用。窓の始まりは読取りの日の前日以前、と書いた |
| R2-F4 | Claude 側（P3） | 無効の間に保存した値を後から使う設計は、無効の間の保存の code の正しさに依る | 採用。無効の間の行と値は、有効にした後の判定に使わない（有効にする migration で失効する） |
| R2-F5 | Claude 側（P3） | 「完全な EJ」の条件に settlement_no の一致が残る | 採用。D5a に合わせた |
| R2-F6 | Claude 側（P3） | NOT NULL の列なのに「NULL の行」と書く。後から列を足す migration の値が決まらない | 採用。「実測でない kind の行は対象にしない」と言い換え、後から足す migration の値も同じ規則にした |
| R2-F7 | Claude 側（P3） | packet の引用の行番号のずれ | 採用 |

- 相談役（Fable 5.1）に、是正の前に方針への反例探しと同じ型の残りの洗い出しを頼んだ（合成モデルの総当たりと乱択、正本が既に扱っている重なりの実読）。採ったもの: 排他の契約を「読取り同士」でなく「同じ DB に対する唯一の操作」と広く書く形（読取りの途中の restore も 1 文で覆う）、条件 3 に「指す行が無い資料」を足すこと、限界 3 つ（SD が固定の disk として見える reader の PC では行が置かれず D10 は働かない／SD を 2 枚交互に使うと全部実測後に戻らない／SD の写しを取外し可能な媒体に置いて差すとアプリは置き場と区別できず、外部前提の違反で D10 は守れない）、37 §37.9 の 2 つの文の食い違いの是正。相談役は、cursor を引き上げた時点でそれ以前の `first_scan_id` はどれも条件 3 を満たせなくなっているので、NULL にして新しく失うものは無い、と確かめた。
- 捨てたもの: round 1 の設計の「無効の間に保存した行と値を、後から有効にしたときそのまま判定に使ってよい」。有効にした時点からやり直すので、有効にした直後の精算の資料は限界と同じ形で判定不能になる（Ordinary Operation に行を足した）。
- Goal・Scope・AC の変更（前半の節に反映済み）: 失敗定義に、行を置く読取り・排他・有効にするときの既存の行と値の扱いと、限界の列挙を足した。Scope の表に `docs/architecture/biz-task-specs.md` を足した。AC15（排他の段、不変の例外の箇条、削った文が残っていないこと）を足した。D10 の条件の式と保存する表・列は変えていない（起草役の報告と、Coordinator の ADR の該当の段の実読）。
- 起草役が確かめていないと報告したもの: 店が SD を 2 枚交互に使うか（正本には「確かめていない・本 lane は決めない」と書いた）。packet の Scope の表の「他 lane との重なり」の列の再確認。
- 是正は起草役の commit「全数読取りの排他と行を置く読取りを契約にし、判定を有効にする migration で既存の行と値を失効させる」。round 3（上限）へ進む。

### Plan Review round 3（上限、2026-10-12、対象は round 2 の是正と記録の後の版）

- Codex（GPT-6 Astra / high）: reject、P1 1 / P2 1 / P3 1。Claude 側（fresh Opus 5.5、fork でなく前の各 round とも別の context）: approve、P1 0 / P2 0 / P3 7。Ordinary Operation は 2 本とも「外部前提が未確認」。2 本とも、置き場の媒体が読取りの間変わらず外部前提が成り立つ範囲では、誤った全部実測後を出す列を見つけていない（合成モデルの総当たりと乱択）。round 2 の是正（読取りの排他、行を置く読取り、有効にする migration、区間の対応づけを後続の設計へ送ったこと）は、どちらの finding にもなっていない。
- round 天井に達したので次の round は開始せず、Coordinator は残った全行を disposition「同型指摘の一括是正」とした（どれも、暗黙の前提を契約として書くか、文をそろえる是正で、D10 の条件の式・保存する表と列・Scope の芯を変えない）。owner は、危ない穴は直すと決めた（repo 外の回答台帳 TD-236）。

| ID | 出どころ | 要旨 | 是正の形 |
|---|---|---|---|
| R3-C1 | Codex（P1） | 識別子を取った後、列挙の前に別の媒体へ差し替えると、空の読取りが完了になり、元の媒体を戻した後の読取りで実測前の資料が全部実測後になる | 全数読取りを完了にする条件に、読取りの終わりの識別子の取り直し（始めの値と同じときだけ完了）を足した。読取りの間に差し替えて元の媒体へ戻した列は見分けられないので、限界と外部前提（読取りの間は置き場を変える操作をしない）に書いた |
| R3-C2・R3-F3 | Codex（P2）・Claude 側（P3） | 有効にする migration の cursor が「引き上げる向きだけ」とも読め、大きすぎる値を戻せない | 「その時点の最大の ID に置き換える」に統一した |
| R3-C3 | Codex（P3） | AC7 の判定の文が実際の差分と字面で合わない | AC7 を、lane の行・branch・Phase を足していないことの検査に直した |
| R3-F1 | Claude 側（P3） | `docs/Plans.md` に、番号印字の採否が決まっていないと読める文が残る | 採らないと決めたこと（D-116）に合わせた |
| R3-F2 | Claude 側（P3） | 保存の振舞いの正本（35）に、`source_scan_cursor` を保存 TX で採ることが無い。索引が D1〜D9 のまま | 35 に 1 文、索引 4 file を D1〜D10 に（Scope に足した） |
| R3-F4 | Claude 側（P3） | 区間の開始で、前回の終端の対応づけが確定していないだけの場合が登録の変化に落ちる | 後続の設計へ送る項目の一覧に足した |
| R3-F5 | Claude 側（P3） | 条件 2 で、識別メタが NULL の資料の扱いが無い | どれかが NULL なら成り立たない、と足した |
| R3-F6・R3-F7 | Claude 側（P3） | 有効にする変更の merge 条件に引く行の列挙の漏れ、file の数の食い違い | 列挙に足し、数を最終の差分で数え直した |

- 相談役（Fable 5.1）に、一括是正の前に方針への反例探しと同じ型の残りの最後の洗い出しを頼んだ。採ったもの: R3-C1 は識別子の取り直しで閉じ、媒体を掴み続ける形は必須にしない（保証できるかは Windows の実機か公式の資料でしか確かめられず、接続の runtime lane の候補として送る）。同じ型の残りとして、読取りの間にレジスターツールが資料を `XZ` から `XZ_BKUP` へ動かす列を挙げたので、移動元を先に列挙する条件と外部前提で扱った（移動が一方向であることは静的解析による推定で、その推定に依ると正本に書いた）。
- 起草役が直さずに報告したもの: `docs/architecture/cmd-task-specs.md` の索引の 1 行も D1〜D9 のまま（発注が名指しした範囲の外）。Final Review の後の是正か follow-up で扱う。
- 限界・外部前提として残るもの（安全側に倒れない）: 読取りの間に媒体を差し替えて、終わるまでに元の媒体へ戻した場合。レジスターツールが資料を移動先から移動元へ戻すことがあった場合（無いと推定、未確認）。置き場の file を人が改名・追加した場合。どれも D10 を有効にする前に、接続の runtime lane と有効にする lane の gate で扱う。
- 是正は起草役の commit「全数読取りの完了に識別子の取り直しと列挙の順を足し、有効にする migration の cursor を最大 ID への置換えに統一する」。是正の後の版の再確認は Final Review が行う。

### plan-gate での訂正（2026-10-15、対象は round 3 の是正の後の版）

- 持ち帰り資料の集計（前の集計を見ない別の担当の数え直し済み。repo 外の回答台帳 TD-252〜TD-254）で、Z004 → 同じ精算の Z001 → 精算回数 → 題の末尾が等しい完了した日計明細、の鎖が、手元に中身のある Z004 の全てで精算票を 1 件に決め、日時で結んだ相手と全て同じだった（数は ADR D5 の外部前提。`rg -c '日時で結んだ相手と全て同じだった' docs/adr/2026-09-18-stocktake-time-evidence.md` が 1）。owner は訂正を本 lane に入れてから plan-approved にすると決め、設計を継ぎ足しで歪ませないよう求めた（TD-254）
- 起草役が「結べる値は日時だけ／日時のほかに無い」を観測の文に置き換えた（`f5979e31`、6 file の同じ事実の文を 1 対 1 で置換え。理由を失った型の列挙と出典の検索式を削った）。対応づけを確定する条件・鎖を採るか・Z004 を Z001 と組にする方法は決めず、D5a の未決 (1) の例を置き換えた。D10・D5a の欠け・評価の順・区間の開始・D3 は変えていない
- 再確認（fresh Opus 5.5、本 lane の各 round と別の context）: approve、P1 0 / P2 0 / P3 4（数え直しの書式の条件、母数 1 の観測の断定、P7 の文の継ぎ足し、D-116 の未決の書き方）。全件採用し、起草役が `c9623cc4` で直した。同じ reviewer の closure は closed（P 0）
- 直さずに残したもの: 同じ趣旨の観測の文が ADR・29・32・D-116・packet の 5 か所にある（元の「日時だけ」の文を 1 対 1 で置き換えた結果）。32 の「差は今は 1 だが」（事実として正しく、番号の差を使わない規則と一緒に書かれている）
- 後続の lane へ送るもの: 鎖を採るなら、EJ の精算票の題の末尾と、Z001 の精算回数・メタの時刻の保存が要る（今の `EjRecordKind::Settlement` は題の末尾を持たない）

### Final Review broad 1（2026-10-15、head `e482ad2a`）

- Claude 側（Fable 5.1、fresh context）: approve、P1 0 / P2 0 / P3 3。Codex（GPT-6 Astra / high）: reject、P1 1 / P2 0 / P3 0。Ordinary Operation は Claude 側が成立、Codex が具体的な反例あり。AC は 2 本とも全て一致
- record: Codex を pending、Claude 側と裁定を fail（`--pr-reviews 1`）。Coordinator は全件採用し、是正の前に相談役（Fable 5.1）へ直し方の反例探しを頼んだ

| ID | 出どころ | 要旨 | 是正の形 |
|---|---|---|---|
| F1-C1 | Codex（P1） | 全部実測後を相殺の判定の対象外にしたことで、PLU の clear で未精算の売上ごと code が消えた数えた商品の判定（D4）まで外れ、帳簿と現物の差が要再確認なしで残る | 相談役の結論で範囲を定義し直した: D10 が決めるのは資料の行に載った販売・返品の前後だけで、売上がその商品の行に載っているか（clear 行の規則〈code の単位。今回に別の code の行があっても〉と、前回の code が分からない区間の規則）は全部実測前でなければ分類によらず当てる。相談役が最初の案（範囲 =「今回に行のある商品」）の穴〈code を変える書出し〉を示し、起草役が前回の code が分からない区間の穴〈code を変えた商品に今回の行がある〉を報告したので、区間の規則の対象を今回の行の有無によらない形にした。Matrix D10-13 を置き換え、D10-31・D10-32 を足した。Ordinary Operation に書出しの日の行を足した |
| F1-F1 | Claude 側（P3） | 2026-10-15 の件数を ADR に直接書き、件数の置き場の規則（backlog の入力）と食い違う | backlog に入力 5 を足して件数を移し、ADR と P5・P7 は参照に置き換えた |
| F1-F2 | Claude 側（P3） | D-116 の Status と Implementation Results の状態の文が古い | Status の句を削り（proposed のまま）、Implementation Results を現在の状態に置き換えた |
| F1-F3 | Claude 側（P3） | Matrix の D10-19 の欠番 | 欠番の旨を注記した（ID は詰めない） |

- あわせて直したもの（起草役の sweep）: ADR の観測の段の「同じ集計」の指す先、tracking-system-tables の clear 行と区間全体の文を ADR と揃えた
- 是正は Gated Amendment 1（`ae2d9abf`・`b67c94b1`）。現在の head の Final Review broad 2 本で確かめる

### Final Review broad 2（2026-10-15、head `c472650e`）

- Claude 側（Fable 5.1、前回と別の fresh context）: reject、P1 0 / P2 1 / P3 4。Codex（GPT-6 Astra / high）: reject、P1 0 / P2 1 / P3 0。Ordinary Operation は Claude 側が成立、Codex が具体的な反例あり。AC は 2 本とも全て一致
- record: Codex を pending、Claude 側と裁定を fail（`--pr-reviews 1`）。2 本の P2 は同じ根（GA1 の範囲の定義が、flag を判定不能に結ぶ古い言い方に届いていない）と裁定した

| ID | 出どころ | 要旨 | 是正の形 |
|---|---|---|---|
| F2-C1 | Codex（P2） | 32 の出力契約が「Unknown の場合の flag」のままで、全部実測後の商品の flag を返せない | 分類と flag の理由を別々に返す形に置き換えた |
| F2-F1 | Claude 側（P2） | 区間全体の規則の括弧が全部実測後の商品に及ぶか読めない | 「判定不能か全部実測後かによらず含む」と置き換えた（ADR・32） |
| F2-F2 | Claude 側（P3） | D4 の表の「分類によらず」が全部実測前の除外とぶつかる | 「判定不能のときと同じく」に置き換えた |
| F2-F3 | Claude 側（P3） | 場所を選んだ root を識別子なしとする理由が規則の範囲より狭い | 理由を「媒体そのものか写しかを区別できない」に置き換えた |
| F2-F4 | Claude 側（P3） | 締められなかった日と媒体の入替の段に 8 つの状況が連結 | 文言を変えずに箇条に分けた |
| F2-F5 | Claude 側（P3） | 全数読取りの規則が 6 か所に全文で複製 | 見送り。縮める置換えは別の層の 3 file に及ぶので、runtime の lane がそれらを触るときに参照へ縮める（PR の Follow-up） |
| F2-F6 | Claude 側（Negative-space） | tracking の列を後から足す migration の経路に Matrix の行が無い | Matrix の Residual Test Gaps に 1 行 |

- 同じ根の残りとして、起草役が flag を判定不能に結ぶ言い方を sweep し、flag の保存の規則の主語（ADR・32・tracking）と 32 の通常適用の文の After の括弧を置き換えた（`ec178d31`）
- 是正は Gated Amendment 2（`e0c984fe`・`ec178d31`）。現在の head の Final Review broad 2 本で確かめる

先行 round の結果・評価（判定・件数・採否・reviewer の意見）はこの節にだけ書く。前半の節と遷移記録には「round N の是正（Review Response 参照）」だけを書く（独立 review は `## Review Response` より前だけを読む）。

Closeout（2026-10-15）: Final Review は broad 3 回で、毎回 Codex と Claude 側（Fable 5.1）の独立 2 本。1 回目 = Codex reject（P1 1）・Claude 側 approve（P3 3）→ Gated Amendment 1（D10 が決めるのは行の販売・返品の前後だけで、売上がその商品の行に載っているか〈clear 行の規則・前回の code が分からない区間の規則〉は分類によらず確かめる、と範囲を定義し直した）。2 回目 = 2 本とも reject（P2 1 ずつ。同じ根: flag を判定不能に結ぶ古い言い方の残り）→ Gated Amendment 2。3 回目（head `db4d3f4e`）= 2 本とも approve（Codex P3 1、Claude 側 P3 5。P1 / P2 は 0）。3 回目の P3 6 件は本 lane では直さず全件 follow-up とし、`docs/backlog.md` へ送った: ㉘ ④ への 2 件（分類の出力の形が変わること、D-116 Decision (1) の「全数読取りを残す」の読み方）は「㉘ ④ と日次の EJ 取込みへの申し送り」へ、正本の文の整理の 3 件（D10 の条件 3 の式を AC で pin していない、全数読取りの保存の契約の複製、ADR D5a と D-116 の「決まるまでの帰結」の要約の範囲）は 1 項目の follow-up へ。残る 1 件 F-P3-4（Scope の表の節名に Gated Amendment で触った 4 か所が無い）は、この closeout で Scope の表に節名を足して閉じた。merge は owner の Ready・merge の承認（TD-257）の後に helper 経由。
