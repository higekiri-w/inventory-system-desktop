# Plan Packet: SD を直接読む設計（毎日の売上データの入力元をレジの SD にする）

wave に属さない単独の lane（design-first、docs だけ。runtime は後続の lane）。owner の lane 選択は 2026-10-06（TD-117、repo 外の回答台帳）。起票時の並走のうち `agent/z001-display`（PR #145）と `agent/stocktake-p1`（PR #148）は merge 済みで、本 branch は origin/main の取込みでその変更を含む。ほかの並走は `agent/plu-clear`（2026-10-06 起票）、`agent/npm-audit-1006`（npm の開発用依存の high、D-108）。本 lane の branch は `agent/sd-direct-read`。

## Workflow State

- Phase: archive
- Risk: R3
- Plan Commit: dbe6626d030fbe80c54f0c8990ee3239b8ff10b6
- Amendments: 49f4d97c7eb1dc04a49305f8785d52740937e39c
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（subagent_type: writer）
- Plan Reviewer: fresh Opus 5.5 + Codex（model は発注時に決める）
- Final Reviewer: Fable 5.1（Claude 側、R3）+ Codex（GPT-6.1 Sol 既定）。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Review Minimum: 1
- Human Gate: ready,merge
- Branch: agent/sd-direct-read

遷移の記録:

1. kickoff → spec-check（2026-10-06、起草役）: owner 決定（2026-10-06「毎日の売上データ〈Z001/Z002/Z004/Z005 と EJ〉は CV17 を開かずにアプリが SD から直接読む。CV17 は PLU の書込みだけ」、Coordinator の発注で受領）を Scope にし、Risk を R3 と記録した（下の Risk）。
2. spec-check → design（2026-10-06、起草役）: 標準手順の正本（`docs/project-memory.md` の決めた運用、`docs/function-design/55-ui-csv-import.md` UI-07-D12、`29-io-daily-report-parser.md` §29.4.1）が「SD → CV17 取込み → `EcrDatas` から選ぶ」で、SD を読む IO・候補の規則・二重取込みの照合が無い。同じ commit で設計正本を更新した（下の Design Readiness）。
3. design のまま止める（起草の時点）: owner の判断事項 1〜5 が残り、design → plan-draft の条件「未解決の設計の問いが無い」を満たさなかった。
4. design（owner の決定の反映、2026-10-06、起草役、本 commit）: owner の判断 1〜5 の決定（下の「owner の判断事項」）を D-111・設計正本・本 packet・Matrix に反映し、写しの設計（IO-10、BIZ-08-D5）を足した。Phase は design のまま。理由: 決定 1（SD は動かさない）の前提 P2・P3（Contract Probe）を Plan Gate の前に店の経験で確かめる。写しの細部の未決 A・B は Non-scope にしたので design → plan-draft を止める問いではない。
5. design → plan-draft（2026-10-07、起草役、本 commit）: 条件「設計の出力が正本にある」は、IO-09・IO-10・IO-07-D5・BIZ-08-D3〜D5・CMD-12-D1・UI-07-D12〜D14・`daily_report_imports.settlement_no`・D-111 が下の Design Readiness の引く正本にあることで満たす。条件「未解決の設計の問いが無い」は、owner の判断 1〜5 が 2026-10-06 に決まり、決定 1 の前提 P2・P3 の扱いが 2026-10-07 に決まったことで満たす（P3 は店の実績〈TD-139〉で合格。P2 は試しを行わず〈owner 決定、TD-176〉、owner の承認した代わりの扱いと運用の制約〈D-111〉で閉じた。下の Contract Probe）。残る P1 は設計に依らず runtime の lane の L3 の前に確かめるもの、`XZ` の file が数千本になったときのレジの振舞いは (b) の lane の前提（D-111 の Revisit）、写しの未決 A・B は Non-scope で、どれも本 lane の設計の問いではない。
6. plan-draft → plan-gate（2026-10-07、Coordinator、本 commit）: packet と Matrix（`docs/plans/test-matrices/2026-10-06-sd-direct-read.md`）が揃い commit されている（`docs/DEV_WORKFLOW.md` Workflow State の表）。Draft PR #150 で Plan Review（fresh Opus + Codex）に出す。
7. plan-gate のまま是正（round 1、2026-10-07、Coordinator）: round 1 の reject を起草役が `d14cfacc` で直した（下の Review Response）。Scope と設計の方向は変えないので plan-gate に留め、round 2 で再 review する（`docs/DEV_WORKFLOW.md` Workflow State「a plan-gate rejection corrected in place stays at plan-gate」）。遷移の記録 5 の「UI-07-D12〜D14」は round 1 の番号の振り直し（UI-07-D15・D16）より前の記録。
8. plan-gate のまま是正（round 2、2026-10-07、Coordinator）: round 2 の reject を起草役が `f8c816f5`（指摘 1〜9）と `6d3880c6`（Contract Probe P1）で直した（下の Review Response）。Scope に `24-io-csv-import-repo.md` を足したが Goal と設計の方向は変えないので plan-gate に留め、round 3（上限）で再 review する。
9. plan-gate（round 3、上限、2026-10-07、Coordinator）: round 3 は両者 reject（P1 0）。`docs/DEV_WORKFLOW.md` Review Rules の round 天井に達したので round 4 は回さず、残った findings を disposition「同型指摘の一括是正」とした（どれも正本の書き足りない所を埋める是正で、Goal と設計の方向を変えない）。起草役が `70387036` で直し、Coordinator が現物で確かめた（下の Review Response）。plan-approved は owner に諮り、reviewer の再確認は Final Review に回す（前例: `docs/archive/plans/2026-09-29-harness-pr5-gate-holes.md`）。
10. plan-gate → plan-approved（2026-10-07、Coordinator、本 commit）: round 3 の disposition（同型指摘の一括是正）の後、owner が plan-approved を承認した（repo 外の回答台帳 TD-180）。同じ問い合わせで介入の上限を 12 に（TD-181）、移行期の SD を戻す案内を「いつもの取込み」の文に（TD-182）決め、`dbe6626d` に反映した。Plan Commit = `dbe6626d`（承認した版）。reviewer の再確認は Final Review。
11. plan-approved → implementing（2026-10-07、Coordinator、state-only）: 本 lane の成果物（Scope の設計正本）は plan-first の change に同乗済みで、implementing で書く runtime のコードは無い（runtime は後続の lane）。Draft PR #150 で Final Review（Claude 側 Fable 5.1 と Codex、互いに独立、Final Review Minimum 1）へ進む（前例: `docs/archive/plans/2026-09-27-daily-report-z-display.md`）。
12. implementing（Final Review の P3 の是正、2026-10-07、Coordinator、本 commit）: Final broad（`bc92d56b`）は両者 approve（P1/P2 0、P3 10）。owner の質優先の方針に合わせ Ready の前に P3 を全件直した（起草役の `49f4d97c`、Gated Amendment として `Amendments` に記録）。closure の review で確かめる。
13. implementing → archive（2026-10-07、Coordinator の closeout、本 commit）: Codex の closure は approve（P 0）。Amendments の記録で Plan contract が変わったので fresh broad を Fable 5.1 で取り直して approve（P1 0 / P2 0 / P3 4）、helper の record は pass、owner が Ready を承認し（TD-183）、helper 経由の squash merge（PR #150、2026-10-07）。packet と Matrix を `docs/archive/plans/` へ移した。fresh broad の P3 4 件は後続の「SD 直読みの runtime」の lane の起票時に直す（`docs/backlog.md`）。

## Owner Effort Budget

- 介入回数上限: 15（owner 了承 2026-10-07、repo 外の回答台帳 TD-184。その前の 12 は TD-181。起票時は既定 6 から 10 に上げた。理由: 製品の振舞いの判断が 5 点〈owner の判断事項 1〜5〉と、店の経験の確認が 2 点〈Contract Probe の P2・P3〉ある。1 回の問い合わせにまとめても decision point の数で数える）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 15（TD-184） | 15（closeout 時点の実数。2026-10-06: 判断 1〜5 の 5。2026-10-07: 移す操作を後続の lane にする判断〈TD-142〉1、P3 の店の経験〈TD-139〉1、P2 の店の経験〈TD-152・TD-175〉1、P2 の試しを行わず代わりの扱いと運用の制約を承認〈TD-176〉1、round 3 の後の plan-approved〈TD-180〉1、上限の改定〈TD-181〉1、移行期の SD を戻す案内の文〈TD-182〉1、Ready〈TD-183〉1、上限の改定〈TD-184〉1、merge 1） | 0 | 0 | 15 = 15 + 0 + 0（起票時の見込みのとおり。上限の改定は Ready の問い合わせと同じ回で諮り、1 つの decision point に数えた〈`docs/DEV_WORKFLOW.md` の Owner Effort Budget〉） |

## Risk

Risk: R3

Reason:
operator の毎日の取込みの操作列、POS の file の入力経路、Tauri command と DTO（足す）、DB の列（足す）、重複取込みの判定を変える設計。本 lane は docs だけだが、契約を決めるので R3。データの破壊・実店舗データの露出は無い（SD に書かない設計。採取した事実は名前・形・hash の質的な要約だけを書く）ので R4 ではない。classifier の workflow の一覧（AGENTS・CLAUDE・DEV_WORKFLOW・MANUAL・code_review・project-profile・agent-guidance・templates・`.agents`・`.claude/{rules,commands,skills}`・PR template）に当たる file を Scope に持たないので Final Review Minimum 1。required gate の green / red は変わらない（docs だけ。function-design の新しい file を作らないので `design_compliance_test` の未登録の文書も生じない）。

## Goal

Goal Invariant:

### 最小完了条件

- 後続の runtime lane の Writer が、チャットの履歴を見ずに source docs だけで「レジの SD を探す → 売上の file を列挙して読む → 精算ごとの束を取込み済みと照らす → 利用者が選んで preview → commit → SD をレジへ戻す」を実装できる（IO-09・BIZ-08-D3 / D4 / D6・CMD-12-D1・UI-07-D12・D15・D16・IO-07-D5・`daily_report_imports.settlement_no`）。
- 旧い標準手順（`SD → CV17 取込み → EcrDatas から選ぶ` を通常の手順とする記述）の live な残りが 0（`rg` の結果を AC に書く）。
- owner の判断（2026-10-06 決定）が設計正本に反映され、(a) の段階（本 lane と SD 直読みの runtime の lane）では SD は動かさず、取り込んだ原本の写しが PC に残る設計がある（IO-10、BIZ-08-D5）。残る未決（写しの保持・backup）は D-111 と packet に `未決（owner）` で並んでいる。

### 失敗定義

- runtime の Writer が、SD の場所・読む file・取込み済みの見分け・形の外れた file の扱い・SD を書き換えない保証のどれかを推測しないと書けない。
- 設計が、同じ精算を別の bytes で二重に取り込む経路（追加確認だけで通る）を残す、または SD に書く API を許す。
- source docs に `EcrDatas` を標準の入力元とする live な記述が残る。

### 非目的

- runtime の code・test・fixture・bindings・migration（後続の lane）。
- EJ の取込みの BIZ・DB の設計（入力の経路と取込み済みの見分けの規則だけを引き継ぐ）。Z004 の取込みの再開（一時停止中、ADR の停止）。
- 窓より前の未取込みの精算の検知（欠けの検知）。Excel 印刷の代替の判定（backlog の別項目）。ECR+ の終了後の精算の経路。
- CV17 で PLU を書き込む経路の変更。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

設計を含む変更なので、取込みの毎日の操作列を置く。本 lane は docs だけで、表の通常運用は runtime の lane の後に成り立つ。**この文書を完了できる**（設計正本がそろう）ことと、**通常運用を達成できる**（店で SD から取り込める）ことは別で、後者は本 lane では未達。取込み済みを `XZ_BKUP` へ移す操作（D-111 (1) の (b)、後続の lane）ができるまで、店は CV17 の取込みを今の運用のまま続ける（日々の売上の Z は毎日、EJ は毎月末日。D-111 の運用の制約、owner 2026-10-07）。下の表はその間の列で、CV17 を開かずに済む運用は (b) の後に成り立つ。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 店は開いたまま、その日の精算前 | ECR+ かレジで精算する | レジが SD の `XZ\yyyy\mm\` に Z001/Z002/Z004/Z005、`XZ` 直下の EJ に記録を書く | SD に精算の file がある | 書込みの契機は精算の操作（説明書）。保存の設定は有効（`docs/project-memory.md` のレジの節） |
| SD がレジに入っている | SD を抜いて PC に差す | Windows が SD を取外し可能な drive として見せる | drive が見える | 店の PC で観測済み（29 §29.7.2 SD-25） |
| SD が PC に差してある（(b) の lane まで） | CV17 で今の順の取込みを済ませる（日々の売上の Z は毎日、EJ は毎月末日） | SD の `XZ` の Z・EJ が `XZ_BKUP` へ移り、PC の `EcrDatas` に複製される | CV17 の取込みが終わった | D-111 の運用の制約。アプリは `XZ` と `XZ_BKUP` の両方を読むので、CV17 の取込みの前に読んでも取り込める（下の移行期の行） |
| 売上データ取込み画面・日報取込みタブ | 「SD から読む」 | アプリが SD を探して読み、「読み終わりました。いつもの取込みが済んでいれば、SD はレジに戻してください（次の精算に要ります）。」と精算の一覧を出す（UI-07-D15 (2)） | 一覧に「取り込めます」の行がある（無く、取り込めない精算があれば「取り込める精算はありません。取り込めない精算が N 件あります…」、それも無ければ「新しい精算はありません」。UI-07-D15 (3)） | owner 決定 3: 自動で探し、見つからなければ選ぶ（UI-07-D15） |
| 一覧が出た | SD をレジへ戻す | 次の精算ができる（SD が無いと精算できない） | 読んだ内容はアプリが 30 分持つ | CMD-12-D1 |
| 一覧に「取り込めます」がある | その行の「確認する」 | 既存のプレビュー（対象日・総売上・支払・部門・警告・同日追加の確認） | プレビューが出る | — |
| プレビュー | 「取り込む」（同日追加なら確認） | 日報が保存され、「日次売上を見る」へ進める | 結果が出る | 同じ精算の別の bytes は BIZ-08-D4 で止まる |
| 「取り込む」の中で | — | アプリが読んだ 3 本の写しを PC のアプリの folder に残す（利用者の操作なし）。残せないと取り込まずに固定の文を出す | 写しが書けた | BIZ-08-D5 |
| 同じ日に 2 回目の精算をした | 同じ操作を繰り返す | 1 回目は「取込み済み」、2 回目は「取り込めます」 | — | 同じ日の複数の Z は観測済み（SD-08） |
| CV17 の日次の取込みを済ませた後（(b) の lane まで。D-111 の運用の制約） | 「SD から読む」 | `XZ_BKUP` の分も読み、取込み済みは hash で「取込み済み」になる | — | `XZ_BKUP` = `EcrDatas`（SD-22）。取込み前の原本も同じ bytes と推定（P1、CV17 の静的解析、推定・強。実機の前後比較は runtime の lane の L3 の前）。SD から取り込む束は精算回数のある束だけ（BIZ-08-D6）なので、推定が外れて bytes が違っても後から来た方は BIZ-08-D4 で止まる |
| 翌日 | 精算する（前日までの Z は CV17 の毎日の取込みで `XZ_BKUP` へ移っている） | レジは精算を続け、その日の Z を `XZ` に書く。EJ は毎月末日の CV17 の取込みまで `XZ` 直下で伸びる | 精算できる | Z の名前は日付で決まり、別の日の Z とぶつからない（SD-02・SD-08）。Z が `XZ` に何日も溜まる状態は (b) まで生じない（P2 の扱い）。EJ の約 1 か月は店の実績（P3 合格） |
| SD が読めない・過去の分 | 「ファイルを選んで取り込む」で 1 つずつ選ぶ | 既存の 3 ファイルの経路でプレビューへ。選んだ file が自動で見つけた SD の上なら SD の入力として扱い、写しを残し、精算回数の無い束を止める（37 §37.3 手順 1a） | 3 つそろう | owner 決定 4: 補助のリンク（UI-07-D16） |

Plan Review は、この列が「正常な条件で目的を達成できるか」と「危険な結果を出さないか」を別々に答える（`docs/DEV_WORKFLOW.md` Review Rules）。

## owner の判断事項（2026-10-06 決定）

確認済み事実: `XZ_BKUP` と PC の `EcrDatas` は全件で同じ bytes（29 §29.7.2 SD-22）。SD は店の PC で取外し可能な drive として見える（SD-25）。SD が無いとレジは精算できない（SD-18）。

| # | 判断事項 | 決定（owner 2026-10-06） | 設計への反映 |
|---|---|---|---|
| 1 | SD を読むだけにするか、`XZ_BKUP` へ移すか | (a) の段階では動かさない（読むだけ）。移す操作は 2026-10-07 に (b) の後続の lane と決めた（TD-142） | IO-09-D3（29 §29.7.6）、D-111 (8)。前提 P2・P3 は 2026-10-07 に扱いを決めた（P3 合格、P2 は D-111 の運用の制約。Contract Probe） |
| 2 | 読んだ原本の写しを PC に残すか | 残す | IO-10（29 §29.8）、BIZ-08-D5（37 §37.4 手順 2b、§37.9 の BIZ-08-D5）、pos-tables §12b の `source_files_json`、71 §71.1 の後の注記 |
| 3 | SD の場所を自動で探すか、利用者が選ぶか | 自動で探し、見つからなければ利用者が選ぶ | IO-09-D1、UI-07-D15 |
| 4 | ファイルを選ぶ経路の置き場所 | 同じ取込み画面の小さな補助のリンク（ひとまず。デザインの刷新で見直す） | UI-07-D16、D-111 の Revisit |
| 5 | backlog「Z004 layout B 対応」の重み | 下げる（layout B は CV17 の明示書出しだけが作り、店は基本もう使わない） | `docs/backlog.md` の当該 entry の注記 |

owner の方向（D-111 の Context）: アプリが気を利かせて、CV17 での作業を利用者から隠す。

残る未決（owner）: A 写しの保持期間と削除（既定は消さずに残す）。B 写しを backup に含めるか・PC の外へ出すか（今の backup は DB の 1 file だけ。backup の設計 lane の論点と一緒に決める）。どちらも本 lane の Non-scope で、runtime の既定（消さない・backup に入れない）で通常運用は成り立つ。

## Scope

本 lane は docs だけを変える（予定 file の全部。所有の列は並走 lane との重なり）。

| file | 変更 | 他 lane・#145 との重なり |
|---|---|---|
| `docs/function-design/29-io-daily-report-parser.md` | IO-07-D5（`settlement_no`）、§29.4.1 の標準経路の文、IO-07-D3 の「通常の手順」の文、§29.7 IO-09 の新設、§29.8 IO-10（写し）の新設 | なし |
| `docs/function-design/37-biz-daily-report-import-service.md` | §37.1 の入力、`DailyReportInputFile.sd_relative_path`、`CachedDailyReportPreview.settlement_no`・`sd_source_files`、§37.4 の signature（`app_data_dir`）と手順 2b（BIZ-08-D5）、§37.3 手順 8 の BIZ-08-D4、§37.3 手順 4・§37.4 手順 2a・§37.9 手順 7 の BIZ-08-D6、§37.4 手順 4a・6、§37.7 の行、§37.8 の非目的、§37.9 の新設（BIZ-08-D3）、更新履歴 | なし |
| `docs/function-design/45-cmd-daily-report-import.md` | AppState の scan cache、§45.4 の commit の `AppHandle` と `app_data_dir`、§45.6a・§45.6b（CMD-12-D1）、§45.8、更新履歴 | なし |
| `docs/function-design/55-ui-csv-import.md` | §55.0 の表の CMD、日報取込みの利用者フローの手順 2・4、UI-07-D12 の改訂、UI-07-D15・D16 の新設、§55.1 の `DailyReportImportPage.tsx` の行 | `agent/stocktake-p1` は PR #148 で merge 済みで、本 file を触らなかった。本 lane の所有は §55.0 の「画面構成」の表の日報の行・「日報取込みの利用者フロー」・「UI判断 ID」の D12・D15・D16・§55.1 の `DailyReportImportPage.tsx` の行だけ |
| `docs/function-design/29-io-ej-parser.md` | IO-08.10 の事実・入力の経路・取込み済みの見分け・後続が決めること | なし |
| `docs/architecture/io-task-specs.md` | IO-07 の出力に `settlement_no?`、IO-09 の task spec の新設（末尾） | なし |
| `docs/architecture/biz-task-specs.md` | BIZ-08 の入力・段階間データ・Stage 2 手順 5・Stage 4 手順 2 | `agent/stocktake-p1` は PR #148 で merge 済みで、本 file を触らなかった。本 lane は BIZ-08 の節だけ |
| `docs/architecture/cmd-task-specs.md` | CMD-12 の表と責務境界 | なし |
| `docs/architecture/ui-task-specs.md` | UI-07 の利用者操作フロー 1・4 | なし |
| `docs/ARCHITECTURE.md` | adapter の表の行、IO の task 一覧の IO-09・IO-10、依存の行、IO 層の文書の行の IO-01〜IO-10 | なし |
| `docs/FUNCTION_DESIGN.md` | IO 一覧と索引の IO-07・BIZ-08・CMD-12 の行 | なし |
| `docs/SCREEN_DESIGN.md` | 1 日の動線、売上データ取込み画面の節 | なし |
| `docs/function-design/24-io-csv-import-repo.md` | `NewDailyReportImport.settlement_no`、§14.14 の INSERT の列、§14.18a `find_same_settlement_daily_report_import`（新設）、更新履歴 | なし（#145 の §14.21 の変更は merge 済みで、本 lane は §14.14・§14.18a と構造体の節だけ） |
| `docs/db-design/pos-tables.md` | §12b の `settlement_no` の列と冪等性の行、B-2 の入力単位、更新履歴 | #145 は本 file を触らずに merge 済み。PR #148（stocktake-p1）は本 file の別の節を変えて merge 済みで、origin/main の取込みで解決した |
| `docs/DB_DESIGN.md` | POS 日報の境界の文 | なし |
| `docs/project-memory.md` | POS Facts の layout の行、在るものの CV17 の行、決めた運用の標準手順の行、レジの節の SD の配置の行 | A だけが SD→CV17 の標準手順の行を直す（発注どおり） |
| `docs/plu-export-and-real-csv-verification.md` | SDカード / PCツール保存領域の節、スマホアプリの扱いの文 | `agent/plu-clear` が PLU の節を触る可能性。本 lane は「SDカード / PCツール保存領域」の節と「スマホアプリの扱い」の最初の段落だけ |
| `docs/decision-log.md` | 末尾に D-111 だけを追記 | 全 lane が末尾に追記（A = D-111、B = D-109、C = D-110、`agent/npm-audit-1006` = D-108）。merge 順で両方を残す |
| `docs/backlog.md` | 着手対象に「SD 直読みの runtime」の entry を 1 つ足す。「Z004 layout B 対応」の entry に重みを下げる注記を 1 つ足す（owner 決定 5、Coordinator の許可） | 全 lane が自 lane の entry だけ |
| `docs/function-design/71-mnt-backup.md` | §71.1 の後に、backup の対象が DB だけで写し・画像を含まない注記を 1 段落 | なし |
| `docs/plans/2026-10-06-sd-direct-read.md`、`docs/plans/test-matrices/2026-10-06-sd-direct-read.md` | 新設 | なし |

触らない: `docs/Plans.md`（D-097）、`docs/function-design/90-traceability.md`（生成物）、`docs/spec/requirements.md`（REQ-401 の行の部品の列に IO-09 を足すのは runtime の lane。traceability の再生成が要るため）、`docs/design-system/reference/mockup-d-import-export.html`（参照の mockup で正本でない。131 行の「CV17取込み後の PC 側 EcrDatas フォルダ」は残す）。

### runtime の lane への申し送り（形が変わる型と、作る所・読む所）

本 lane は code を書かない。形を変える型の producer / consumer を `rg` で全件挙げる（2026-10-07 に `6b765f75` の現物で数え直した。行番号は literal の構築・定義・呼出しの行）。runtime の lane はこの一覧を Scope の出発点にする。

| 型・関数 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| `DailyReportParseResult`（`settlement_no` を足す） | IO-07-D5 | `src-tauri/src/io/daily_report_parser.rs:69`（定義）・`:79`（構築は 1 か所） | `src-tauri/src/biz/daily_report_import_service/parse.rs`、同 file の test の helper（`daily_report_parser.rs:701`・`:713`） |
| `CachedDailyReportPreview`（`settlement_no`） | BIZ-08-D4 | `biz/daily_report_import_service/mod.rs:121`（定義）・`parse.rs:216`（構築） | `commit.rs`、`cmd/daily_report_import_cmd.rs:250`（test の literal の構築は 1 か所。`:249` の helper `cached` の中で、`:281` の `cached_with_status` 等はその helper を呼ぶ） |
| `NewDailyReportImport`（`settlement_no`） | BIZ-08-D4、24 §14.14 | `db/sales_repo.rs:197`（定義）・`:596`（INSERT） | 構築: `biz/daily_report_import_service/commit.rs:102`、test: `db/sales_repo.rs:2222`（helper）・`:2509`・`:2518`、`biz/daily_report_import_service/tests.rs:293`、`biz/sales_service.rs:645`、`cmd/sales_cmd.rs:293`、`cmd/daily_report_import_cmd.rs:410` |
| 新 repository 関数 `find_same_settlement_daily_report_import` | BIZ-08-D4、24 §14.18a | `db/sales_repo.rs`（新設。`DailyReportImport` を返さず id だけ） | 呼出し: BIZ-08 §37.3 手順 8・§37.4 手順 4a・§37.9 手順 7。`DailyReportImport` を返す既存の SELECT（`sales_repo.rs:716`・`:735`・`:756`・`:830`）と `row_to_daily_report_import`（`:887`）は変えない |
| `daily_report_imports` の列 | migration（次の番号。v7 の次だが並走 lane の migration と番号を runtime の lane が決める） | `db/migration.rs`、新しい `db/schema_vN.rs` | `db/migration.rs:937`・`:973`・`:982`・`:1034`（INSERT の test）、`docs/function-design/22-mnt-migration.md` |
| `DailyReportImport`（list の DTO） | 変えない（wire に出さない） | `db/sales_repo.rs:181`・`:888` | — |
| `AppState`（`register_sd_scan_cache` を足す） | CMD-12-D1、45 §45.2 | `src-tauri/src/cmd/mod.rs:34`（定義） | literal の構築 15 か所（2026-10-07、`rg -n "AppState \{" src-tauri/src` から定義の行と `-> AppState {` の helper の signature の行を除いた数）。本番: `lib.rs:1264`（`app.manage`）。test: `cmd/product_cmd.rs:273`・`:293`、`cmd/settings_cmd.rs:358`、`cmd/plu_export_cmd.rs:231`、`cmd/stocktake_cmd.rs:201`、`cmd/csv_import_cmd.rs:308`・`:362`・`:438`・`:489`、`cmd/integrity_cmd.rs:56`、`cmd/daily_report_import_cmd.rs:291`、`cmd/sales_cmd.rs:117`・`:333`・`:352`。どの構築にも field を足す（`Default` にするなら定義 1 か所と構築の書き方を runtime の lane が決める） |
| 新 command 2 つと DTO 4 つ | CMD-12-D1 | `cmd/daily_report_import_cmd.rs`、`lib.rs` の `collect_commands` | `src/lib/bindings.ts`（再生成）、`src/features/daily-report-import/` |
| `DailyReportInputFile`（`sd_relative_path`・`source_path`） | BIZ-08-D5・D6、37 §37.3 手順 1a | `biz/daily_report_import_service/mod.rs:32`（定義）、`cmd/daily_report_import_cmd.rs:43`（構築） | test の構築: `biz/daily_report_import_service/tests.rs:15`（helper `source_file`）、`biz/sales_service.rs:1175`（helper `file` の中）・`:1360` |
| `DailyReportSourceFileRequest`（省略可の `source_path` を足す） | 45 §45.3、UI-07-D16 | `cmd/daily_report_import_cmd.rs:17`（定義）・`:43`（`DailyReportInputFile` への写像） | test: `cmd/daily_report_import_cmd.rs:242`（helper `request`）。frontend: `src/features/daily-report-import/hooks/useDailyReportImportFlow.ts:82`・`:165`（payload を作る所。今は path を捨てる `:207` の `extractFilename`）、`useDailyReportImportFlow.test.tsx:395`〜`:397`・`DailyReportImportPage.flow.test.tsx:136`〜`:138`（期待の payload）、`src/lib/bindings.ts:565`（再生成） |
| `DailyReportSdSelection`（BIZ-08 の enum、新設） | 37 §37.9、45 §45.6a | `biz/daily_report_import_service`（定義） | 構築: `cmd/daily_report_import_cmd.rs` の `scan_register_sd`。IO-09 の型にしない（`src-tauri/tests/architecture_test.rs:44`〜`:47` の LAYER_RULES、cmd → io は禁止） |
| `BizError::SourceCopyFailed(std::io::ErrorKind)`（新しい variant） | BIZ-08-D5、45 §45.7 | `src-tauri/src/biz/mod.rs:54`（`BizError` の定義。`Display` の match も足す）、返す所は BIZ-08 commit（37 §37.4 手順 2b） | `impl From<BizError> for CmdError`（`src-tauri/src/cmd/mod.rs:130`）に `CmdError::internal`（kind `internal`、固定の文）の分岐を足す。`BizError` を網羅で match する所は `rg -n "BizError::" src-tauri/src` で起票時に数え直す |
| UI の写しの失敗からの回復（`decideRecoverTo`・reducer） | 55 の日報取込みの利用者フローの手順 2 | 変えない: `src/features/daily-report-import/hooks/useDailyReportImportFlow.ts:35`〜`:38`（`import_error` だけ `idle`、ほかは `preview`）、`reducer.ts:56`（`import_failed` で preview・token・file 名を `previousState` に保つ）、`reducer.ts:77`（`dismiss_error` で `previousState` の preview へ戻る） | 写しの失敗を `internal` にすること（上の行）で preview に戻る。test は Matrix の hook / flow の行 |
| `parse_and_validate_daily_report_with_sd_lookup`（BIZ の内部関数、新設） | 37 §37.3 手順 1a | `biz/daily_report_import_service`（`find_sd_roots: impl FnOnce() -> Result<Vec<RegisterSdRoot>, RegisterSdError>` を受ける。公開の関数は IO-09 `find_register_sd_roots` を渡す） | test: 一時 directory の root を返す closure・失敗を返す closure（Matrix の §37.3 手順 1a の行） |
| `commit_daily_report_import`（`app_data_dir` を足す） | BIZ-08-D5 | `biz/daily_report_import_service/commit.rs:14` | 呼出し: `cmd/daily_report_import_cmd.rs:114`（1。`:70` は同名の CMD の定義）、`biz/sales_service.rs:1199`・`:1367`（2）、`biz/daily_report_import_service/tests.rs`（29 行） |
| `source_files_json` の要素（`sd_relative_path`・`copy_path` を足す） | BIZ-08-D5 | `biz/daily_report_import_service/commit.rs:83` | 読む所: `parse.rs:236`（`source_filenames`。無い field を許す） |
| 新 module `io::pos_source_copy` | IO-10 | `src-tauri/src/io/pos_source_copy.rs`、`io/mod.rs` | `design_compliance_test.rs` の map の同じ行に `io::pos_source_copy` |
| （後続の lane）`XZ_BKUP` へ移す操作 | D-111 (1) の (b)、TD-142 | 本 lane も SD 直読みの runtime の lane も作らない。`io::register_sd` に書込みの API を足さず、別の module に置く（IO-09-D3 の source の検査を保つ） | 後続の lane が起票時に Scope を決める |
| 新 module `io::register_sd` | IO-09 | `src-tauri/src/io/register_sd.rs`、`io/mod.rs` | `src-tauri/tests/design_compliance_test.rs` の `build_doc_to_modules_map()` の `29-io-daily-report-parser.md` の行に `io::register_sd` を足す |

注: `sales_service.rs`・`sales_cmd.rs`・`sales_repo.rs` の test を触った #145 は merge 済みで、上の行番号はその後の現物。runtime の lane は起票時の現物で数え直し、所有表を書く。

## Non-scope

- runtime の code・test・fixture・bindings・migration・`90-traceability.md`。
- SD の file を移す設計（owner 決定 1 で本 lane では採らない）。アプリの「取り込み済みの Z・EJ を `XZ_BKUP` へ移す（片付ける）」操作は後続の lane（owner 2026-10-07 決定、TD-142）。まず読むだけを作り、その後に移す操作を作る。SD への書込みの設計（CV17 と同じ `_nnnn` の改名、途中の失敗、CV17 の取込みとの両立）はその lane で行う（D-111 の Decision (1)）。その lane は起票時に、`XZ` の file が数千本になったときのレジの振舞い（P2 の残り）を前提として確かめる（D-111 の Revisit）。写しの保持期間・削除と、写しの backup・PC の外への持出し（未決 A・B）。
- EJ の取込みの BIZ・DB・画面、Z004 の取込みの再開、欠けの検知、Excel 印刷の代替。
- backlog の「日報取込み標準手順の残設計」の entry の書換え（他 entry。closeout に回す）。「Z004 layout B 対応」は注記を 1 つ足すだけで、本文は変えない。
- 日報の手でのファイル選択で「1 つずつ選び足し・個別に外す」の backlog 化（#145 の closeout〈PR #147〉で済んだ。設計の要件としては UI-07-D16 に入れた）。

## Acceptance Criteria

- AC1（旧い標準手順の live な残り 0）: `rg -n "EcrDatas" docs --glob '!docs/archive/**' --glob '!docs/research/**'` の hit が、すべて「事実（`XZ_BKUP` と同じ bytes・layout A の観測）」「予備の経路」「2026-10-06 に置き換えた旨の履歴」「D-111」「参照の mockup 1 行（`docs/design-system/reference/mockup-d-import-export.html:131`）」のどれかで、`EcrDatas` を通常の入力元とする文が 0。`rg -n "所定フォルダ|CV17取込み後のPC側" docs --glob '!docs/archive/**' --glob '!docs/research/**'` の hit は、`55-ui-csv-import.md` の UI-07-D12 の「旧版（2026-08-01）の…は置き換えた」の履歴の 1 行と本 AC の行だけ（起草時の実測で 2 件）。
- AC2（IO-09 の契約がある）: `docs/function-design/29-io-daily-report-parser.md` に `find_register_sd_roots`・`resolve_register_sd_root`・`list_register_sd_entries`・`read_register_sd_file` のシグネチャ（`rg -n "^fn (find|resolve|list|read)_register_sd" docs/function-design/29-io-daily-report-parser.md` が 4 行）と IO-09-D1〜D4 がある。
- AC3（候補の規則と二重取込みの拒否）: `37-biz-daily-report-import-service.md` に §37.9（BIZ-08-D3）と §37.3 手順 8 の BIZ-08-D4、§37.4 手順 4a、§37.3 手順 4・§37.4 手順 2a・§37.9 手順 7 の BIZ-08-D6 がある。`pos-tables.md` §12b に `settlement_no` の列がある。
- AC4（command と画面）: `45-cmd-daily-report-import.md` に §45.6a・§45.6b（CMD-12-D1）、`55-ui-csv-import.md` に UI-07-D12（改訂）・D15・D16 がある。
- AC5（決定の記録）: `docs/decision-log.md` の末尾に `## D-111` があり、owner の決定 1〜5・Context（CV17 の作業を利用者から隠す）・未決 A・B を挙げる。
- AC6（検査）: `bash scripts/doc-consistency-check.sh --target plan` と `bash scripts/doc-consistency-check.sh` が ERROR 0（WARN は報告）。
- AC7（写し）: `29-io-daily-report-parser.md` に `fn save_pos_source_copy` のシグネチャと §29.8 があり、37 に BIZ-08-D5（§37.4 手順 2b）がある。`rg -n '\bD-108\b' docs --glob '!docs/archive/**' --glob '!docs/plans/2026-10-06-sd-direct-read.md'` の hit が 0（本 packet は並走 lane の D-108 を前文と Scope で名指しするので除く。2026-10-07 の実測で 0 件。`agent/npm-audit-1006` が先に merge したときの hit はその lane の D-108 で、本 lane の記述ではない）。

## Design Readiness

- 引用する設計正本（節まで）: `docs/function-design/29-io-daily-report-parser.md` IO-07-D3・D5、§29.4.1、§29.7（IO-09-D1〜D4）／`37-biz-daily-report-import-service.md` §37.3 手順 4・5・8、§37.4、§37.9（BIZ-08-D3・D4・D6）／`45-cmd-daily-report-import.md` §45.2・§45.6a・§45.6b（CMD-12-D1）／`55-ui-csv-import.md` §55.0、UI-07-D12・D15・D16／`29-io-ej-parser.md` IO-08.10／`db-design/pos-tables.md` §12b・B-2／`ARCHITECTURE.md` POS Adapter Boundary・レイヤー間の呼び出し原則／`docs/adr/2026-09-18-stocktake-time-evidence.md`（Z004 の精算同一性 guard、変えない）。
- 必要な設計成果物: function-design（IO-09・BIZ-08・CMD-12・UI-07） = updated in this PR／DB（`settlement_no`） = updated in this PR（migration の番号は runtime）／SCREEN_DESIGN = updated in this PR／decision-log = D-111 を追加。
- plan にしかない durable な判断の昇格先: すべて D-111 と上の正本へ置いた。packet にだけある判断は無い（owner の判断事項は D-111 の「未決」にも書いた）。
- 前提・制約と、延期した design gap: BIZ-08-D4・D6 は、取込み前の `XZ` の原本の 3 本とも精算回数が読め、取込み後の file と同じ値であることに依る。この前提は Plan Gate の前に CV17 2.0.1 の静的解析で確かめた: CV17 の SD の取込みは `XZ` の原本を移して複製するだけで中身を書かないので、原本は取込み後の `XZ_BKUP`・`EcrDatas` の file と同じ bytes（推定・強。CV17 2.0.1 の静的解析、2026-10-07。下の Contract Probe P1）。原本は今の parser が読む `EcrDatas` の file と同じ形・同じ精算回数になる。設計は bytes の一致だけには頼らず（同じ bytes なら hash、違えば精算回数で照らす）、SD から取り込む束は 3 本とも精算回数が読めてそろう束だけにする（BIZ-08-D6）。残る確認は実機の前後比較（1 本の SHA-256）で、runtime の lane の L3 の前に行い、不一致なら L3 の前に BIZ-08-D4・D6 を見直す。店の CV17 が 1.1.1 であること（解析は 2.0.1）と .NET 側の handler の IL は未確認として P1 に残す。owner の判断 1（(a) の段階では SD は動かさない）の前提は 2026-10-07 に閉じた: P3（EJ を移さずに長く置いたときの追記）は店の実績で合格（TD-139）。P2（Z を `XZ` に溜めたときの精算）は試さず（TD-176）、Z の名前が日付で決まり別の日の Z とぶつからないこと（SD-02・SD-08）と、(b) の lane まで店が CV17 の取込みを今の運用のまま続ける制約（D-111）で扱う。制約の間、Z が `XZ` に何日も溜まる状態と、EJ が 1 か月を超えて `XZ` に残る状態は生じない。延期: `XZ` の file が数千本になったときのレジの振舞い（精算のときの処理時間、FAT32 の 1 directory の entry 数の上限）は (b) の lane（または CV17 の毎日の取込みをやめる時点）の前提に送る（D-111 の Revisit）。延期が安全な理由: 本 lane と runtime の lane は SD を読むだけで、CV17 の取込みが続く間 `XZ` の file の数は今と変わらない。欠けの検知・写しの保持と backup（未決 A・B）・移す設計も延期（Non-scope）。
- 絶対保証の自己点検: 「SD に書かない」の例外は Windows の FAT の最終アクセス日と `System Volume Information`（アプリでは止められない。IO-09-D3 に明記）。「二重に数えない」の例外は、手で選んだ `settlement_no` が None の束（layout B）と、NULL の既存の取込み（D-111 より前）だけ。SD の経路の束は None を取り込まない（BIZ-08-D6、scan・preview・commit の各段）ので、SD の束を先に取り込んで後から CV17 の移した別の bytes が来ても、逆の順でも照合で止まる。D-111 より前の取込みは `EcrDatas` からで SD の `XZ_BKUP` と同じ bytes なので hash で止まる。手で選んだ layout B の束と同じ精算の SD の束は同日追加の確認で通りうる（layout B は店が基本使わない。D-111 の Guarantee range）。精算回数が戻った場合は誤って拒む（安全側）。
- 判定: ready（plan-draft に進める）。理由: 設計の出力（IO-09・IO-10・IO-07-D5・BIZ-08-D3〜D6・CMD-12-D1・UI-07-D12・D15・D16・`settlement_no`・D-111）が上の正本にあり、owner の判断 1〜5 は 2026-10-06 に決まり、決定 1 の前提 P2・P3 は 2026-10-07 に扱いを決めた。BIZ-08-D4・D6 が依る P1 の前提（原本の精算回数の可読性と一致、bytes の同一性）は Plan Gate の前に静的解析で確かめた（推定・強。Contract Probe）。残る P1 の実機の前後比較（runtime の lane の L3 の前の確認）・`XZ` の多数の file・未決 A・B は、どれも本 lane の設計の問いではない。

## Registration / Generation Obligations

本 lane（docs だけ）は該当なし（function-design の新しい file・REQ の増減・route・画面の新設をしない）。runtime の lane の義務: 新しい 2 command（`scan_register_sd`・`parse_and_validate_daily_report_from_sd`）に `#[tauri::command]` + `#[specta::specta]` を付け、`lib.rs` の specta の `collect_commands!`（`src-tauri/src/lib.rs:277`）と runtime の `tauri::generate_handler!`（同 `:1272`）の両方に登録する（`scripts/check-command-drift.sh` は宣言 `D`・`generate_handler` `H`・`collect_commands` `S`・bindings `T` の 4 つの集合の一致を `bash scripts/doc-consistency-check.sh` の中で確かめる。一方だけの登録は red。件数は書かない。45 §45.8）、`cargo run --bin generate_bindings`、`design_compliance_test.rs` の `build_doc_to_modules_map()` の `29-io-daily-report-parser.md` の行に `io::register_sd` と `io::pos_source_copy` の両方（29 §29.7.1・§29.8.1）、`docs/spec/requirements.md` の REQ-401 の部品の列に IO-09 を足すなら `cargo run --bin generate_traceability`、migration と `22-mnt-migration.md`。

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | Which concepts belong to replaceable external adapters, and which concepts are stable app-core contracts? | Architecture, function design, decision-log, Plan Packet | SD の配置・名前・形は adapter（IO-09）。core は「精算ごとの束」「取込み済みの状態」「同じ精算の二重取込みを拒む」。BIZ-08-D3 の組分けは Z00k の名前に依るので CASIO 固有で、BIZ に置くのは日報の束の規則が BIZ-08 にあるため（D-023 の範囲内、D-111 Compatibility） | レジが替わったら IO-09 と §37.9 の組分けを取り替える |
| Fact check / design decision split | Which claims are observed facts from hardware/tool/files, and which are app decisions that need source-doc promotion? | Investigation doc, source design docs, decision-log | 事実は 29 §29.7.2 の表（観測 / 状態 / 未確認の分類）。判断は D-111・IO-09-D1〜D4・BIZ-08-D3 / D4 / D6・CMD-12-D1・UI-07-D12・D15・D16 | — |
| Lifecycle / retry | What happens before, during, after, and after failure for import/export, duplicate input, rollback, retry, cancellation, and re-run? | Function design, DB design, UI design, Test Matrix | 読取りの途中の失敗は全体の失敗（IO-09・§37.9 手順 3）。scan の snapshot は 30 分で、切れたら読み直し。取込み済みの再読みは「取込み済み」。取消の後は「取り込めます」に戻る。同じ精算の別の bytes は拒む | Matrix の State Lifecycle |
| Operator workflow | What does the operator do in the real sequence across app, external tool, media, print/export, backup, and recovery? | Screen/UI design, function design, Plan Packet manual checks | 精算 → SD を PC → 読む → SD を戻す → 選ぶ → 取り込む。CV17 は PLU の書込みだけ（(b) の lane までは今の CV17 の取込みも続ける。D-111 の運用の制約）。Excel 印刷の代替は別項目 | runtime の lane の L3 |
| Replacement path | If the external system changes, which files/modules/docs are replaced and which app-core contracts remain stable? | Architecture, function design, decision-log | `io::register_sd`・29 §29.7・§37.9 の組分けを取り替え、preview / commit / 照合は残る | — |
| Data safety / evidence | How can the claim be supported by anonymized shape/count/hash/procedure evidence without committing real store data? | Plan Packet Data Safety, investigation doc, review evidence | 事実は名前・形・hash の照合の質的な要約だけ（件数・実データは tracked に書かない）。SD に書かない（IO-09-D3） | Data Safety |
| Reporting / accounting semantics | Are totals, summaries, item records, returns, corrections, and inventory movements modeled separately enough to avoid false business meaning? | DB design, function design, report design, Test Matrix | 日報の二重計上を BIZ-08-D4 と BIZ-08-D6（SD の経路は精算回数のある束だけ）で止める。日報は在庫を動かさない（D-025、変えない） | — |
| Manual verification | Which assertions cannot be proven by automated tests and require Windows native L3, external tool import, or real-device confirmation? | Plan Packet, Test Matrix, PR body | 実 SD の自動の発見（`DRIVE_REMOVABLE`）、取込み前の原本の読取り（原本と取込み後の file が同じ bytes かの実機の前後比較、P1）、読んだ後に SD を戻して精算できること | runtime の lane の L3 |
| 環境・再現性 | 新設の環境依存（toolchain / CI runner / OS 差異等）を repo-pinned config で強制するか、明示的に defer するか | repo-pinned config, Plan Packet | drive の列挙は Windows だけ（既存の `windows-sys` の feature `Win32_Storage_FileSystem`。`src-tauri/Cargo.toml` の `[target.'cfg(windows)'.dependencies]`）。Windows 以外は空の列で、test は `resolve_register_sd_root` で一時 directory を使う | runtime の lane |

## Boundary / Wire Contract

- producer: BIZ-08 §37.9（`DailyReportSdScan`）、CMD-12 §45.6a・§45.6b
- consumer: UI-07（`scanRegisterSd`・`parseAndValidateDailyReportFromSd`）
- wire type: `DailyReportSourceFileRequest` に省略可の `source_path: Option<String>`（手で選んだ file の dialog の path、45 §45.3）。`RegisterSdScanResponse { scan: DailyReportSdScan, scan_token: String }`、`DailyReportSdCandidate { candidate_key, path_date, report_date?, source_filenames, status }`、`DailyReportSdCandidateStatus`（5 値）。既存の `DailyReportPreviewResponse` を返す
- internal type: `DailyReportSdScanSnapshot`（AppState、wire にしない）、`settlement_no: Option<i64>`（IO-07 → cache → DB。wire に出さない）、IO-09 の型（BIZ-08 の中だけ。CMD は BIZ-08 の `DailyReportSdSelection` を作り、IO-09 の型を使わない）、`source_files_json` の要素（BIZ-08 の内部の serialize 用の型。wire 型 `DailyReportSourceFileInfo` に field を足さない、37 §37.4 手順 6）
- precision/range: `settlement_no` は i64（先頭 0 を落とした整数、IO-07-D3）。日付は YYYY-MM-DD
- round-trip path: scan → candidate_key → snapshot の 3 本 → 既存の preview → commit（DB に `settlement_no`）
- invalid input: 空の `selected_path` は validation。期限切れの scan_token は import_error。取り込めない candidate_key は validation
- compatibility: 既存の command と DTO の wire は、`DailyReportSourceFileRequest` に省略可の `source_path`（45 §45.3）を足すほかは変えない（省いた呼出しは今どおり PC 上の file として通る）。`daily_report_imports` に nullable の列を足し、既存行は NULL

## Test Plan

Test Design Matrix: [2026-10-06-sd-direct-read](test-matrices/2026-10-06-sd-direct-read.md)（runtime の lane が実装する test の設計。本 lane は docs の検査だけ）。

- targeted tests: 本 lane は `bash scripts/doc-consistency-check.sh --target plan` と full。
- negative tests: Matrix の Negative Paths（runtime）。
- compatibility checks: 既存の CMD-12 の wire と既存の取込みの経路（Matrix）。
- data safety checks: 本 lane の差分に実データ（JAN・商品名・金額・件数の細部・ファイルの中身）を入れない。
- main wiring/integration checks: runtime の lane。

## Review Focus

- Ordinary Operation の列が、正常な条件で目的（アプリが SD から毎日取り込み、SD をレジへ戻す。CV17 の取込みは (b) の lane まで並行して続く）を達成できるか。
- BIZ-08-D4・D6 が、SD-23 の推定（原本と取込み後の file は同じ bytes）が実機で外れた場合も、取り込む順に依らず二重取込みを止めるか、正常な同じ日の 2 回目の精算を誤って止めないか。
- IO-09 の名前の規則（`XZ` は連番なし、`XZ_BKUP` は連番あり、大文字小文字を区別しない）が観測した場合をすべて覆い、未知の名前を読まないか。
- 窓（最後の取込みの日と 30 日前の早い方）で、普段の運用の精算が候補から落ちないか。
- 層: CMD が IO-09 を呼ばない、規則が BIZ にある。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| D-111 | `docs/decision-log.md` D-111 | 本 lane は設計の記録。runtime は後続 | runtime（Matrix） | — |
| IO-07-D5 | 29 IO-07-D5（§29.2 の後） | runtime: `daily_report_parser.rs` | Matrix の IO-07-D5 の行 | 非対象 |
| IO-09-D1 | 29 §29.7.4 | runtime: `io/register_sd.rs` | Matrix | L3: 店の PC で自動で見つかる |
| IO-09-D2 | 29 §29.7.5 | 同上 | Matrix | 非対象 |
| IO-09-D3 | 29 §29.7.6 | 同上 | Matrix（source の検査と前後の一覧の比較） | L3: 読んだ後に SD をレジへ戻して精算できる |
| IO-09-D4 | 29 §29.7.6 | 同上 | Matrix | 非対象 |
| BIZ-08-D3 | 37 §37.9 | runtime: `biz/daily_report_import_service` | Matrix | 非対象 |
| BIZ-08-D4 | 37 §37.3 手順 8・§37.4 手順 4a・6、24 §14.14・§14.18a、pos-tables §12b | runtime: BIZ-08・`sales_repo.rs`・migration | Matrix | 非対象 |
| BIZ-08-D6 | 37 §37.3 手順 4・§37.4 手順 2a・§37.9 手順 7 | runtime: BIZ-08 の parse・commit・scan | Matrix | 非対象 |
| CMD-12-D1 | 45 §45.2・§45.3（`DailyReportSourceFileRequest.source_path`）・§45.4 手順 3a（`app_data_dir` の取り方）・§45.6a・§45.6b | runtime: `daily_report_import_cmd.rs`・`lib.rs` | Matrix | 非対象 |
| IO-10 | 29 §29.8 | runtime: `io/pos_source_copy.rs` | Matrix | 非対象 |
| BIZ-08-D5 | 37 §37.4 手順 2b・6、§37.3 手順 1a、§37.9 の BIZ-08-D5、45 §45.7（`SourceCopyFailed` → `internal`）、pos-tables §12b、71 §71.1 の注記 | runtime: BIZ-08 commit | Matrix | L3: 取り込んだ後に `pos-sources/` に 3 本がある |
| UI-07-D12 | 55 UI-判断 ID | runtime: `features/daily-report-import` | Matrix | 目視の確認 |
| UI-07-D15 | 55 UI-判断 ID | 同上 | Matrix | L3: 一覧の状態の文言と icon、SD を戻す案内 |
| UI-07-D16 | 55 UI-判断 ID | 同上 | Matrix | 目視の確認 |
| IO-08-D10（改訂） | 29-io-ej-parser IO-08.10 | EJ の取込みの lane | 非対象（本 lane・runtime の SD lane とも） | 非対象 |
| 既存 BIZ-08-D1・D2、IO-07-D3、SPEC-SDI（同日追加の確認・insert-only・per-import rollback） | 37 §37.3〜§37.5、pos-tables §12b | 変えない（SD の経路も同じ §37.3〜§37.4 を通る） | 既存の test（runtime が回帰で回す） | 非対象 |
| 隣接の除外: Z004 の精算同一性 guard（ADR SPEC-STK-TIME、32 の時点証拠契約） | 32 §15 冒頭 | 変えない。§37.9 の引継ぎで参照だけ | 非対象 | 非対象 |
| 隣接の除外: UI-07 の Z004 タブの一時停止（SPEC-STOP-D4） | 55 §現行buildの一時停止 | 変えない | 非対象 | 非対象 |

## Contract Probe

- P1 SD-23（取込み前の `XZ` の原本と CV17 の取込み後の file が同じ bytes か。BIZ-08-D4・D6 が依る「原本の 3 本とも精算回数が読め、取込み後の file と同じ値」を含む）: **Plan Gate の前に静的解析で確かめた（2026-10-07、推定・強）**。出典: repo 外の解析記録（2026-10-07、Coordinator が発注した read-only の静的解析）。対象は CV17 Ver.2.0.1 の公式の配布 `CV17_V2P01.zip`（SHA-256 `3ccbf0e5a485034454c3b0eee2d0fc36072c90908845a2780a544c830fab97ab`）の `CV17ST.dll`（配布の中の名前 `te400st.dll`）で、install・実行・DLL のロードはしていない。結果: (a) SD の取込み（export `ImportSDCardData`）は、取込みの thread が Z001_・Z002_・Z005_・Z004_・Z006_・Z011_・Z009_ の種類ごとに年・月の folder の `*.CSV` を列挙し、`XZ` の原本を `MoveFileExA` で `XZ_BKUP\YYYY\MM\<元の名前の一部>_NNNN.CSV` へ改名・移動してから、移動後の file を `CopyFileA` で `EcrDatas` 側の `YYYY\MM\` の同じ名前へ複製する（folder は ini の `[SDCARD]` の `FOLDER_XZ` / `FOLDER_XZ_BKUP` から組む）。EJ の取込みも同じ「移動してから複製」の型。(b) この DLL は `CreateFileA/W`・`WriteFile`・`SHFileOperation` を import しておらず、データの中身を書き直す手段を持たない（書くのは取込みの log だけ）。(c) よって原本と `XZ_BKUP`・`EcrDatas` の file は同じ bytes と推定する（`MoveFileExA`・`CopyFileA` は中身を変えない）。店の実測 SD-22（`XZ_BKUP` と `EcrDatas` の全件の bytes 一致）と合う。原本は今の parser が読む `EcrDatas` の file と同じ形で、精算回数も同じ値になり、IO-07-D3・D5 の読み方がそのまま当たる。残る未確認: (i) 店の CV17 は 1.1.1 で、解析は 2.0.1（export の位置が 1.1.1 から一定の量ずれるだけと分かっている。店の PC を 2.0.1 に揃える方針が別の lane にある）。(ii) .NET 側の取込みの button の handler の中身（IL）は未読（その user string に `XZ_BKUP`・`EcrDatas` は無い）。(iii) 実機の前後比較。確認として残すこと（**runtime の lane の L3 の前**）: 精算の後、CV17 を開く前に SD の `XZ` の Z の 1 本の SHA-256 を読取り専用で採り、CV17 で取り込んだ後の `XZ_BKUP` の該当の file の SHA-256 と比べる。合否: 一致なら推定を確かめたとする。不一致なら、原本の精算回数が読めない・値が違う可能性があるので、runtime の lane の L3 の前に BIZ-08-D4 と BIZ-08-D6 を見直す（正規の SD の束が取り込めない、または二重取込みの照合が効かないため）。誰が: Codex が店の PC で hash だけを採る（店の R-50。中身は読まない。SD の採取は owner の別承認）。
- P2 SD-14（Z を `XZ` に何日も残したとき、レジの精算と SD の保存が続くか）: **扱いを決めた（2026-10-07）**。結果: 店は日々の売上（Z001 等）の CV17 の取込みを休んだ経験が無い（repo 外の回答台帳 TD-152・TD-175）。起票時の代わりの手順（普段どおりの精算を 2 日続ける間に取込みをしない試し）は行わない（owner 決定 2026-10-07、TD-176。取込みを止めることは店の業務を止めるため）。代わりの扱い（owner 承認 2026-10-07）: Z は精算のときだけ SD に書かれ、名前は日付と、同じ日の精算ごとの接尾字の 1 文字で決まる（SD-02・SD-08、説明書 C p.16）ので、別の日の Z と名前はぶつからない。同じ日の 2 回目以降の Z が、1 回目の Z を `XZ` に残したまま別の名前で書かれることは観測済み（SD-08）。EJ は約 1 か月 `XZ` に残っても精算が続いている（SD-09、P3）。運用の制約: 取込み済みを `XZ_BKUP` へ移す操作（D-111 (1) の (b)、TD-142、後続の lane）ができるまで、店は CV17 の取込みを今の運用のまま続ける（D-111）。よって Z が `XZ` に何日も溜まる状態は、本 lane と SD 直読みの runtime の lane では生じない。残る未確認（名前の衝突ではない）: `XZ` の file が数千本になったときのレジの振舞い（精算のときに既存の file を見て名前を決めるとみられることによる処理時間、FAT32 の 1 directory の entry 数の上限）。owner の見立て（要旨）は、名前は年月日と回数で決まり中身も同じなので、何千本でも衝突しない。これは (b) の lane（または CV17 の毎日の取込みをやめる時点）の前提に引き継ぐ（D-111 の Revisit）。
- P3 EJ を CV17 で移さずに長く置いたとき（店の実績の約 1 か月を超えて）、レジが `XZ` 直下の EJ への追記を続けるか: **合格（2026-10-07）**。結果: 店の EJ の取込みは毎月末日で、1 か月を超えて空けたことはなく、その間の異常は無い（repo 外の回答台帳 TD-139）。起票時の合否「経験上の異常が無ければ前提を受け入れ、runtime の lane の L3 の後の運用で `XZ` の EJ の size を見る」に当てて合格とする。扱い: (b) の lane まで店は CV17 の EJ の取込み（毎月末日）を続けるので（D-111 の運用の制約）、EJ が 1 か月を超えて `XZ` に残る状態は生じない。runtime の lane の L3 の後の運用で `XZ` の EJ の size を見る。
- P4 精算のときのスマホへの送信の失敗で、レジが「SD カードへバックアップ」した file（2026-10-07 にレジ本体の日計明細の精算で、レシートに「データは SD カードへバックアップしました。次回の精算時に自動送信されます」と出た。repo 外の回答台帳 TD-179）: その「バックアップ」が `XZ` の通常の Z・EJ を指すのか、別の file かは**未確認**。Plan Gate の前提ではない。設計は分岐しない: 名前の規則に合わない file は IO-09 が中身を読まずに数えるだけで（IO-09-D2、29 §29.7.5。未知の名前の file は SD-07 で観測済み）、規則どおりの名前の Z なら通常の候補になり、同じ精算の同じ bytes は hash で 1 つの候補にまとまり（§37.9 手順 6）、別の bytes でも精算回数で止まる（BIZ-08-D4・D6）。手順: runtime の lane の L3 の前に、送信の失敗が起きた日の後の SD の `XZ`・`XZ_BKUP` の名前の一覧を読取り専用で見て、規則に合わない名前の file が増えたかだけを確かめる（中身は読まない）。誰が: Codex が店の PC で名前の一覧だけを採る（SD の採取は owner の別承認）。
- 観測済みで probe の要らない前提: SD が取外し可能な drive として見える（SD-25）、`XZ_BKUP` = `EcrDatas`（SD-22）、名前の形・大文字の `EJ`・同じ日の複数の Z・未知の名前の file（SD-02・03・07・08）、`windows-sys` に `Win32_Storage_FileSystem` の feature がある（`src-tauri/Cargo.toml`）。

## Data Safety

- tracked に書かない: 実 SD・`EcrDatas` の file の中身、実 JAN・商品名・金額、実データの件数の細部、店主の発言の原文、repo 外の調査の path の詳細。本 lane の docs は名前の形・形の分類・照合の結果を質的に書いた。
- local-only: repo 外の SD の調査（2026-10-06 run01 / run02）と回答台帳。
- synthetic-only: runtime の lane の test は一時 directory に合成の SD の tree（合成の Z・EJ の bytes）を作る。

## Implementation Results

- 結果: レジの SD を探して売上の file を読み、精算ごとの束を取込み済みと照らし、選んだ束を既存の preview・commit に通し、読んだ原本の写しを PC に残す設計（IO-09・IO-10・IO-07-D5・BIZ-08-D3〜D6・CMD-12-D1・UI-07-D12・D15・D16・`daily_report_imports.settlement_no`）を設計正本に置き、判断を D-111 に記録した。旧い標準手順（CV17 の取込みの後に `EcrDatas` から選ぶ）を通常の手順とする live な記述を置き換えた。本 lane は docs だけで、runtime の code・test・migration は無い。
- 既存の保護: 既存の取込みの経路（3 ファイルを選ぶ経路・同日追加の確認・取込みごとの取消）と wire は、`DailyReportSourceFileRequest` に省略可の `source_path` を足すほかは変えない設計にした。(b) の lane まで店は CV17 の取込みを今の運用のまま続ける（D-111 の運用の制約）。
- review・CI・merge（closeout、2026-10-07）: Plan Review は 3 round（round 1・2 は reject で全件是正、round 3 は両者 reject〈P1 0〉で round 天井の disposition の後に owner が plan-approved を承認〈TD-180〉）。Final Review は broad が両者 approve、P3 を Ready の前に Gated Amendment で直し、Codex の closure と Fable 5.1 の fresh broad が approve。hosted CI の `Merge gate` は success で、owner の Ready の承認（TD-183）の後に helper 経由の squash merge。docs の検査の結果は PR の body が持つ。PR: [#150](https://github.com/higekiri-w/inventory-system-desktop/pull/150)
- 後続: runtime の実装・test・Windows の L3 は「SD 直読みの runtime」の lane、取込み済みを `XZ_BKUP` へ移す操作は (b) の lane（どちらも `docs/backlog.md`）。
- packet との食い違い: なし。

## Review Response

round 1（`3ea4d25c`）: Claude 側 fresh Opus 5.5 = reject（P2 2 / P3 5）、Codex GPT-6.1 Sol（発注 235）= reject（P1 1 / P2 2 / P3 3）。Coordinator が現物で裏取りし、全件を採用した。是正は起草役の `d14cfacc`。

- P1（Codex #1、Opus #1 と同じ筋）SD の経路で精算回数の読めない束（`settlement_no = None`）を取り込めると、CV17 が後で書いた別 bytes・精算回数ありの同じ精算が hash と BIZ-08-D4 をすり抜け、追加確認だけで二重に保存される。重大度は P1 とした（運用の制約で CV17 の取込みが毎日続くので、この筋は毎日開く）。是正: BIZ-08-D6 を新設し、SD の経路は 3 本とも精算回数が読め値がそろう束だけを scan・preview・commit で受ける。予備の「ファイルを選んで取り込む」の扱いは変えない（D-111 の Guarantee range）。P1（SD-23）の合否に「原本の 3 本とも精算回数が読め、取込み後と同じ値」を足した。正規の SD の束はレジの原本で、CV17 の書出し（layout B）ではないので、本 guard で止まらない見込み（runtime の L3 の前の P1 で確かめる）。
- P2（両者）画面と手順の正本が CV17 の取込みを無条件に「要らない」としていた。是正: D-111 の運用の制約（(b) まで CV17 の取込みを今の運用のまま続ける）を 55・SCREEN_DESIGN・ARCHITECTURE・io-task-specs・plu-export・project-memory に条件付きで同期した。
- P2（Codex #3）新設の UI-07-D13 / D14 が既存の同日追加確認（UI-07-D13）・per-import 取消（UI-07-D14）と衝突していた（55:229・:244 で裏取り）。是正: 新設の 2 つを未使用の UI-07-D15・D16 に振り直し、本 lane の参照を同期した。
- P3: 申し送りの表の file:line と site 数を `6b765f75` で数え直し、merge 済みの lane を並走と書く文を直した（Opus #3・Codex #5）。`RegisterSdError` の使われない variant を消した（Opus #4）。§45.4 の commit の signature に `AppHandle` を足した（Opus #5）。Matrix に 3 行を足した（Opus #6）。D-111 の Decision の番号と AC の並びを直した（Opus #7）。AC7 の検索に除外と実測の期待を足した（Codex #6）。37 の「BIZ-08 は精算回数を保存しない」を保存の契約の文に置き換えた（Codex #4）。
- 起草役の判断で Coordinator が受けたもの: D6 の commit の検査を TX の前（手順 1a）と写しの前に置いた（束だけで決まる検査で DB の状態に依らないため、TX の中と同値で、拒む束の写しを先に書かない）。D6 で止めたとき予備の経路へ案内しない（同じ SD の file を手で選び直して穴を開け直さないため）。
- round 1 の後に足した事実: 2026-10-07 にレジ本体で精算した際のスマホ送信の失敗と SD への「バックアップ」（repo 外の回答台帳 TD-179）を Contract Probe の P4 に足した（Plan Gate の前提ではない）。

round 2（`98aa7c24`）: Claude 側 fresh Opus 5.5 = reject（P2 1 / P3 3）、Codex GPT-6 Astra（発注 236）= reject（P2 7）。Coordinator が現物で裏取りし、全件を採用した。是正は起草役の `f8c816f5`・`6d3880c6`。Claude 側は旧前提の語の `rg` で packet の生 file を対象にし、本節の round 1 の 1 行（UI-07 の番号の衝突）を出力で見たと申告した（同じ内容は遷移の記録 7 にもあり、判定には使っていないと申告）。

- P2（Opus #1・Codex #2 同じ筋）CMD が IO-09 の型 `RegisterSdSelection` を作ると `src-tauri/tests/architecture_test.rs:44-47` の `LAYER_RULES`（cmd → io 禁止）に反する（裏取り済み）。是正: 選択の enum を BIZ-08 の `DailyReportSdSelection` にした。
- P2（Codex #1）IO-10 が一時 file を書いた後に `create_dir_all` する順で、初回に失敗する。是正: path を決める → `create_dir_all` → 一時 file → rename の順にし、Matrix に初回の行を足した。
- P2（Codex #3）repository の契約（24）に `settlement_no` と精算回数で照合の候補を取る検索が無く、24 が Scope に無かった。是正: 24 を Scope に足し、`NewDailyReportImport.settlement_no` と §14.18a `find_same_settlement_daily_report_import` を書いた（wire の DTO は変えない）。
- P2（Codex #4）予備の「ファイルを選んで取り込む」で SD 上の file を選ぶと出所が捨てられ、写し（D5）も D6 も効かなかった。是正: IO-09 `locate_in_register_sd_roots` で自動で見つけたレジの SD 上の file なら SD の入力として扱う。`DailyReportSourceFileRequest` に省略できる `source_path` を足す（新しい command は作らない。wire の契約を「この field を足すほかは変えない」に直した）。SD が固定 disk に見える reader の残余は D-111 の Guarantee range に書いた。
- P2（Codex #5）D6・D4 が依る「SD の原本の精算回数が読め、CV17 の取込み後と同じ値」は R3 が依る未確認の外部前提で、Plan Gate の前の Contract Probe に当たる（DEV_WORKFLOW `## Plan Packet Rules`）。是正: Coordinator が CV17 Ver.2.0.1 の `CV17ST.dll` を静的に解析し（install・実行・ロードなし）、SD 取込みが原本を `MoveFileExA` で `XZ_BKUP` へ移し、移動後の file を `CopyFileA` で `EcrDatas` へ複製するだけで、中身を書く API を持たないことを確かめた（推定・強、SD-22 と合う）。P1 の行と依る文を書き直した。実機の前後比較と店の 1.1.1 との差は runtime の lane の L3 の前の確認として残す。
- P2（Codex #6）TX の中の D4 の test が既存の snapshot の再検査に失敗を代行されていた。是正: active な ID に一致する cache を直接作り、D4 だけが拒む行にした。
- P2（Codex #7）UI-07-D15 で `Incomplete` / `Unreadable` だけが残るときにも「新しい精算はありません」が出た。是正: 状態の数で 3 つに分けた。
- P3（Opus #2・#3・#4）`source_files_json` は内部の serialize 用の型で書き wire 型に field を足さない、§37.4 の手順を 2a（D6）・2b（D5）にして読む順と実行の順をそろえた（round 1 の記録の「手順 1a」は今の 2a）、`XZ` と `XZ_BKUP` に同じ bytes があるとき写しの path は `XZ` を優先する、を書いた。

round 3（上限、`31c7bf7e`）: Claude 側 fresh Opus 5.5 = reject（P2 1 / P3 4）、Codex GPT-6.1 Sol（発注 237）= reject（P2 2 / P3 1）。P1 は 0。round 天井に達したので次の round を開始せず、全件を disposition「同型指摘の一括是正」とした（Coordinator が現物で裏取りし採用。是正は起草役の `70387036`）。Claude 側は AC1 の `rg` で packet の生 file が検索の対象に入ったが、件数だけを出し、hit の中身は Review Response より前の 1 行だけだったと申告した。

- P2（Opus #1）IO-09-D3 の「SD に書かない」の source 検査の禁止語が 8 個だけで、`File::create`・`fs::copy` 等が抜けていた。是正: IO-09-D3 を許可の列（`File::open`・`read_dir`・`metadata` 等の読取りだけ）にし、Matrix の検査を `module_uses_only_allowed_fs_api` にした。
- P2（Codex #1）予備の経路で SD の file を読んだ後に SD を抜くと、PC の file として扱われ写しと D6 が黙って外れた。是正: `locate` が `None` の file は preview の時点で在るかを確かめ（`check_selected_file_present`）、無ければ固定の文で止める（§37.3 手順 1a）。
- P2（Codex #2）写しの保存の失敗の後に UI が preview と token に戻れるかを検証していなかった。是正: 写しの失敗を `BizError::SourceCopyFailed`（kind `internal`）にし、現行の `decideRecoverTo`（`import_error` 以外は preview に戻る）と reducer を変えずに同じ preview・token に戻る契約を 55 に書き、flow の test の行を足した。
- P3（Opus #2）(b) までの毎日の手順で CV17 の取込みの位置が無かった。是正: Ordinary Operation に足し、UI-07-D15 (2) の SD を戻す案内を CV17 の取込みと矛盾しない文にした（画面に CV17 の語を出すかは owner 確認、下の「owner の判断」）。
- P3（Opus #3）「SD は動かさない」を範囲なしに書き D-111 (1)(b) と字面で食い違った。是正: (a) の段階に限ると足した。
- P3（Opus #4）test の入口が root の取得の失敗を注入できなかった。是正: 内部の関数が root を取る closure を受ける形にした。
- P3（Opus #5・Codex #3）登録の義務に `io::pos_source_copy` と `tauri::generate_handler!` が抜けていた、Scope の ARCHITECTURE の行が IO-01〜IO-09 だった。是正: 足し、IO-01〜IO-10 に直した。
- 起草役の判断で Coordinator が受けたもの: 写しの失敗を新しい kind でなく既存の `internal` にした（hook を変えずに preview に戻れるため。表示は `error_id` つきになる。runtime の lane の L3 で見る）。IO-09-D3 の許可の列は今の設計が要る API だけにした（足すときは設計の改訂）。

Final Review broad（`bc92d56b`）: Codex GPT-6.1 Sol（発注 238）= approve（P3 4、PR review 5440745352）、Claude 側 Fable 5.1 = approve（P3 6）。P1/P2 は 0。P3 10 件を全件採用し、Ready の前に起草役の `49f4d97c` で直した（Gated Amendment）。内訳: 写しの失敗の分岐の文の範囲を限った・SD から来た失敗はすべて一覧（`sd_list`）へ戻すと 1 通りに決めた・Contract Ledger に 45 §45.3・§45.4 3a・§45.7 を足した・Matrix に固定の文・読まなかった file・選び直しの行を足した・IO-09 の置き場所を `RegisterSdArea::{Sales, Backup}` にし BIZ-08 の `Imported` と分けた・`candidate_key` を固定した（Fable #1〜#6）。Owner Effort Budget を見込み 15 に直した・申し送りに `AppState` の 15 site を足した・状態の要約を UI-07-D15 の参照にした・registry の件数を消し 4 集合の一致を契約にした（Codex #1〜#4）。

Closeout（2026-10-07）: Codex の closure（GPT-6.1 Sol）は approve（P 0）。Amendments の記録で Plan contract が変わったので、fresh broad を Fable 5.1 で `460826e2` に取り直し approve（P1 0 / P2 0 / P3 4）。helper の record は pass で、owner が Ready を承認し（TD-183）、hosted CI の `Merge gate` success の後に helper 経由の squash merge（2026-10-07）。fresh broad の P3 4 件の disposition は backlog（後続の「SD 直読みの runtime」の lane の起票時に直す）: (1) 37 §37.3 手順 9 に `CachedDailyReportPreview.sd_source_files` を詰める手順（1a の後に `sd_relative_path` が Some の file）を足す、(2) BIZ-08-D5 の保証の範囲に「2b の後に 4a・手順 5 で拒んだ束の写しも `pos-sources/` に残りうる。取り込んだかは DB の `copy_path` が正本」を足す、(3) IO-09 `check_selected_file_present` の失敗を `Io { relative_path }` に入れるときの path の意味を明記するか専用の variant にする、(4) 本 packet の Scope の io-task-specs の行に IO-10 の task spec の新設が無く、Contract Ledger の IO-07-D5 の位置は §29.2 の末尾が正しい（archive の本 packet は直さず、runtime の packet の起票時に正しく書く）。
