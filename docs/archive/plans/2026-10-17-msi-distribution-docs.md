# Plan Packet: アプリの MSI の配布・更新・前の版へ戻す手順を docs にする

wave に属さない単独の lane（2026-10-17 に同時に起票した 6 本の 1 本。docs だけ）。v1.0 の gate の 1 つ「MSI 配布手順 docs 化」（`docs/backlog.md` の同名の項目）を、店の Windows 11 Home の PC へアプリの MSI を入れる・更新する・前の版へ戻す手順として置く。

規模と手法（起票の最初に数え、Plan Review round 2 の是正で数え直した）: 触る file は 3（新しい手順書 `docs/msi-distribution-runbook.md`、`docs/PROJECT_HANDOFF.md` の案内の 1 行、`docs/backlog.md` の MSI の項目）と本 packet と Test Design Matrix（`docs/plans/test-matrices/2026-10-17-msi-distribution-docs.md`）。設計正本（function-design・db-design・SCREEN_DESIGN）・画面・migration・wire の型は 0。手順書は既存の契約（MNT-03-D11、22 §16 の回復、71・68 の backup）を引いて並べ、正本に無い店の DB の安全の判断（流れ 5 の前へ進むか戻すかの場面の分け方、起動しないときの保全、更新を始めない条件、起動の後の照合、保管の規則、初回の導入の条件、予行演習の退避と復帰）を足す。その組は下の Contract Ledger。失敗の時点と順序（migration が止まった時点の DB の版）は流れ 5 の「場面を決める」が扱う。手法: 1 lane のまま。

## Workflow State

- Phase: archive
- Risk: R3
- Plan Commit: 85d9b45e7dce840f76caa067de958f84db008792
- Amendments: none
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（本 run の起草役）
- Plan Reviewer: fresh Opus 5.5 + Codex（round 1 は GPT-6.1 Sol / high〈R2 の時点の `.local` の model の選び方の「R2 以下の Plan Review」の行〉。round 2 は Astra / high〈同じ表の「前の round で Codex が P1 を出した lane の再 review」の行〉。R3 へ上げた後の round も Astra / high〈同じ表の「データの安全（migration・restore・backup・削除）」の行〉）。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Reviewer: Fable 5.1（Claude 側。座組表の「R3 以上と design lane で Fable 5.1」。round 2 で R3 へ上げた）+ Codex（`.local` の model の選び方の「データの安全」の行で Astra / high）。座組表どおり
- Final Review Minimum: 1
- Human Gate: ready,merge
- Branch: agent/msi-distribution-docs

遷移の記録:

1. kickoff → spec-check（2026-10-17、起草役）: 発注書 368 の Scope（手順書の置き場と 4 つの流れ、更新のときの DB の安全、未確認の点の確かめ方）を範囲にし、Risk を R2 と記録した（下の Risk）。
2. spec-check → design（2026-10-17、起草役）: 引く正本（22 §3.2 MNT-03-D11、22 §16 の回復、71 §71.4・§71.5・§71.9、68 §68.4）は振舞いの契約として足りるが、配布・更新・戻す流れを 1 つに並べた管理者の手順が無い（`docs/migration-recovery-runbook.md` は v8 から戻す手順だけ）。手順書を新しく置く。
3. design → plan-draft（2026-10-17、起草役、同じ commit）: 条件「設計の出力が正本にある」は、手順書 `docs/msi-distribution-runbook.md` を同じ plan-first の変更に置いて満たす（docs だけの lane の先例 `docs/archive/plans/2026-10-17-unit-redesign-design.md` と同じ形）。条件「未解決の設計の問いが無い」は、実機でしか分からない点（M1〜M9）を手順書に「未確認」と書き、結果で変わる所を表に置いたことと、製品名と upgrade code の固定を初回の導入の前提として本 lane の外の項目（backlog）に出したことで満たす。製品名を何にするかは owner の問い（下の Review Focus の Q1）で、本 lane の手順書は名を引かずに書いてあるので答えを待たない。
4. plan-draft → plan-gate（2026-10-17、Coordinator）: packet と手順書は起草役の commit（`1ef45cbd`）にある。Coordinator は手順書の全文と packet の全文（Goal の失敗定義・Ordinary Operation・Scope と所有表・AC・Design Readiness・Contract Probe・Data Safety）を読み、流れ 5 が DB の版の上がり方と入力の前後で分かれていること、未確認の振舞いが M1〜M9 に揃っていること、差分が docs の 4 file だけで店の運用の細部や実データが無いことを確かめた。手順書を plan-first の commit に置く形は docs だけの lane の先例どおりとして採る。Plan Review round 1 へ出す。
5. plan-gate（round 1 の是正、2026-10-17、Coordinator）: Plan Review round 1（Claude 側 fresh Opus 5.5・Codex GPT-6.1 Sol）の是正（Review Response 参照）。是正の前に相談役（Fable 5.1）に反例探しを頼み、「22 §16 の回復の番号が無い段は置かない」の原理と予行演習の DB の隔離を是正に入れた。起草役が `356663cd` で直し、Coordinator が差分（流れ 5 の表と 1・2、保管の規則、予行演習の準備と片付け、AC6）を読んだ。Scope の範囲と方向は変えず、その場で直したので plan-gate に留める。Plan Review round 2 へ出す。
6. plan-gate（round 2 の是正、2026-10-18、Coordinator）: Plan Review round 2（Claude 側 fresh Opus 5.5・Codex GPT-6.1 Astra）の是正（Review Response 参照）。Risk を R3 に上げた。起草役が `c68f6504` で直し、Coordinator が差分（初回の導入の条件、流れ 4 の照合と起動が止まったとき、流れ 5 の今の DB の版の知り方と表と 1・2、予行演習の片付け、Risk・Contract Ledger・Matrix の新設）を読んだ。流れ 4・5 は今の DB の版を正とする 1 つの筋に書き直され、店の DB に作り直しが当たる経路が無いことを確かめた。Scope の範囲は変えず、その場で直したので plan-gate に留める。Plan Review round 3（天井）へ出す。
7. plan-gate（round 3 の是正、2026-10-18、Coordinator）: Plan Review round 3（Claude 側 fresh Opus 5.5・Codex GPT-6 Astra）の是正（Review Response 参照）。round 3 は round の天井で、owner が「全件を直し、指摘と差分だけを確かめる round を 1 回足す」と決めた（repo 外の回答台帳 TD-315、介入 1 回目 / 予算 9 回）。起草役が `4dcd4a45` で直し、Coordinator が差分（流れ 5 の場面と表、アプリが起動しないときの保全と前進、予行演習の順と LOCALAPPDATA、M6 の合格条件、写しの SHA-256、診断ログの文）を読んだ。流れ 4・5 は「場面を決める → 表の行で決める」の 1 つの筋で、決めるまで今の DB を動かさない。確認の round（closure）へ出す。
8. plan-gate（確認の round の是正、2026-10-18、Coordinator）: 確認の round（Codex GPT-6.1 Sol）の是正（Review Response 参照）。owner が「直して、Claude 側と Codex の両方で確認する」と決めた（repo 外の回答台帳 TD-316、介入 2 回目 / 予算 9 回）。起草役が `f4f13ad6` で直し、Coordinator が差分（流れ 5 の「前の版と入力の基準」の 1 か所の定義とそれを引く場面・表・1〜3、前へ進むときの照合、Matrix の C13・FM19・FM20）と、起草役の組の一巡の表を読んだ。2 回目の確認（Claude 側・Codex）へ出す。
9. plan-gate（2 回目の確認の是正、2026-10-18、Coordinator）: 2 回目の確認（Claude 側 fresh Opus 5.5・Codex GPT-6.1 Sol）の是正（Review Response 参照）。owner が「直して、追加の確認なしで plan-approved」と決めた（repo 外の回答台帳 TD-318、介入 3 回目 / 予算 9 回）。起草役が `140a11fe` で直し、Coordinator が差分（流れ 5 の「前の版と入力の基準」の条件、流れ 3 の手順 5 の追加の写し、MSI の選び方、Matrix の C10・C13・FM21・FM22）を読み、起草役が発注の文言から変えた 2 点を具体的な組で追って採った。
10. plan-gate → plan-approved（2026-10-18、Coordinator、本 commit）: 独立の Plan Reviewer（Writer と別の context）のうち Codex が 2 回目の確認で P1・P2 が無いと報告し、Claude 側の P2 1 件は owner の決定（repo 外の回答台帳 TD-318）どおり直した（Review Response 参照）。Plan Commit は 2 回目の確認の是正と記録の後の版。
11. plan-approved → implementing（2026-10-18、Coordinator、本 commit）: docs だけの lane で、手順書は Plan Commit までに置いてある。状態だけを進め、Final Review（Minimum 1）へ出す。

## Owner Effort Budget

- 介入回数上限: 9（既定の 6 から owner が改定、repo 外の回答台帳 TD-296 の 5a）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 9 | 5（closeout 時点。確認の round を足す決定 1〈TD-315〉、直して両方で確認する決定 1〈TD-316〉、直して進める決定 1〈TD-318〉、Ready 1、merge 1。L3・manual は無し） | 0 | 4 | 9 = 5 + 0 + 4 |

Q1（製品名）は本 lane の完了に要らない（後続の設定の lane の問い）ので、本 lane の介入に数えない。予行演習（M1〜M8）と店の PC の M9 の L3 は後続の項目で、本 lane の介入に数えない。

承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の境の規則（`DEV_WORKFLOW.md:52`「If uncertain between R2 and R3, choose R3 when the change touches ... data safety boundary」）に当たる（Plan Review round 2 の Codex C4・Claude O6、Coordinator が R2 から上げた）。code・設定（`src-tauri/tauri.conf.json` を含む）・DB・command の DTO・画面・merge gate は変えないが、手順書は正本に無い店の DB の安全の判断を足す: 流れ 5 の前へ進むか戻すかの分け方（場面で今の DB の版を、戻す先の backup を基準に入力の有無を知り、表の行で決める）、アプリが起動しないときの保全（DB 一式を写す）と前進（修正版を流れ 4 の手順 2 から）、流れ 3 の更新を始めない条件、流れ 4 の起動の後の照合を業務の再開の条件にすること、保管の規則、初回の導入の条件（予行演習・製品名と upgrade code の固定・作り直しの起点の migration を含む版〈TD-311〉）、予行演習の退避と復帰。各判断と引く正本（22 §3.2 MNT-03-D11・§12・§16、71、68、D-119）の組は Contract Ledger にある。R4 にしない理由: DB の file を動かすのは 22 §16 の回復の手順 5・6（runbook の手順 5・6）の中だけで、手順書はその外で DB を動かす段を足さず（AC6。起動しないときの保全は元の file を残して写すだけ）、消す段も足さない。v8 より後の migration から戻す手順は一般化せず、「その migration の 22 の節に戻す手順が無ければ DB を戻さない」と安全側に倒した。R3 の要件（`DEV_WORKFLOW.md:60`〜`:61`）: Contract Ledger・Data Safety・Contract Probe・Test Design Matrix（`docs/plans/test-matrices/2026-10-17-msi-distribution-docs.md`）・Design Readiness を置いた。required gate の green / red は変わらない（docs だけで、`scripts/ci/classify-changes.sh` の workflow の一覧〈AGENTS・CLAUDE・DEV_WORKFLOW・ci・MANUAL・code_review・review-checklist・project-profile・agent-guidance・templates ほか〉に当たる path が無い）。Final Review Minimum は 1、Human Gate は `ready,merge`（R3 の既定）。

## Goal

Goal Invariant:

### 最小完了条件

- owner が `docs/PROJECT_HANDOFF.md` から辿れる 1 つの手順書で、MSI を作る・初回に入れる・更新の前に backup を取る・更新する・前の版へ戻す、を順に実行でき、各手順で何を確かめるか、止まったら何をするかが分かる。
- 実機で確かめていない振舞いは「未確認」と書かれ、確かめ方（予行演習の項目）と結果で変わる所が手順書にある。

### 失敗定義

- 新しい版で migration が当たった後に、前の版の MSI を入れるだけで戻せるように読める（前の版は起動を止める。MNT-03-D11）。
- 起動時の自動の backup や新しい版で作った backup で前の版へ戻せるように読める（71 §71.9）。
- 実機で確かめていない MSI の振舞い（更新でデータが残る、古い版で上書きできる、アンインストールでデータが残る等）を確かめた事実として書く。
- 前へ進むか戻すかを決める前や、22 §16 の回復の手順 5・6 の外で、今の DB を動かすように読める。
- 前の版へ戻す可能性があるうちに、前の版の MSI か更新の前の backup を手放すように読める。
- 予行演習が開発に使っている DB に当たる、または試験用のデータが版 A で開けない版で入る。
- 店の運用の細部や店のデータを repo に書く。

### 非目的

- `tauri.conf.json` の設定（製品名・upgrade code・版・署名・WebView2 の導入方法）を変える（後続の lane）。
- 予行演習（M1〜M8）と店の PC の M9 を本 lane で行う（owner の手が要り、後続の項目で 1 回にまとめる）。
- 自動更新・署名の導入・Windows の CI build。
- `docs/migration-recovery-runbook.md` と 22 §16 を一般化する・書き換える。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

管理者（owner）の作業の列。アプリの中の操作は変えない。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 配布する commit が決まった | 開発用の Windows の clone で流れ 1（`npm ci --ignore-scripts` → `npm run tauri build` → SHA-256） | 配布する MSI と配布の記録の 1 行 | MSI が見つかり記録が書けた | M1（file の名前と場所） |
| 店の PC にアプリが無い。製品名と upgrade code が固定済み、予行演習済み | 流れ 2（写す → SHA-256 を照らす → 入れる → 起動 → 手動バックアップ） | 店の PC でアプリが起動し、DB と backup ができている | ホームが出て backup が 1 つある | M2（UAC・入る場所）、M3（SmartScreen）、M9（WebView2） |
| 前の版が動いている | 流れ 3（前の版で手動バックアップ → 2 か所へ写す → アプリを閉じる） | 前の版で開ける backup が PC の内と外にある | 写しが 2 か所にある。失敗したら更新しない | なし（既存の 68・71 の機能） |
| 流れ 3 が済んだ | 流れ 4（新しい MSI を照らす → 入れる → 起動 → 照合） | 新しい版で前のデータが見える | 照合の全項目が合った（1 つでも合わなければ入力を始めずに止まる） | M4（置き換えとデータの残り）、M8（動いたままの更新） |
| 翌日以降に不具合が見つかった、流れ 4 で起動か照合が止まった、または入力の後に起動が止まった | 流れ 5（場面 1〈新しい版が 1 度でも照合まで進んだ〉か場面 2〈新しい版の最初の起動が止まった〉で今の DB の版を、戻す先の backup〈最後に流れ 3 で作った backup。照合で入力を始めてよいと決まっていない版で流れ 3 をし直しても替えない〉より後に入力したかで入力の有無を知り、表の行で、戻す〈前の版の MSI を入れ直すか runbook の 7 手順〉か前へ進む〈流れ 3 から、起動しなければ DB 一式を写して保全し修正版を流れ 4 の手順 2 から〉を owner が決める） | 決めるまで今の DB を動かさない。戻せば前の版で開け、今の DB の版が前の版の扱える版以下なら入力も残る。前へ進めば修正版で前のデータが見える | 前の版か修正版が起動し、照合が合う | M6（古い版の上書き）、M7（アンインストール） |

この lane で完了するのは「手順書が置かれ、owner が辿れる」まで。店の PC への初回の導入は、製品名と upgrade code の固定と予行演習（backlog の 2 項目）の後で、v1.0 の通常運用の達成はそれらを含む。

## Scope

- `docs/msi-distribution-runbook.md`（新規）: 今の事実の表（出所つき。Tauri の公式の資料は 2026-10-17 に確かめた URL を各行に）、初回の導入の前に決めること（TD-311 の前提を含む）、配布の記録の欄、流れ 1〜5（流れ 5 は場面で今の DB の版を、戻す先の backup を基準に入力の有無を知り、表の行で前へ進むか戻すかを分ける）、確かめる項目 M1〜M9 と予行演習の準備と片付け。
- `docs/PROJECT_HANDOFF.md`: 案内の表に手順書の行を 1 行足す（template の Registration / Generation Obligations「source / workflow doc 新設」）。
- `docs/backlog.md`: 「MSI 配布手順 docs 化」の項目の下に、本 lane の起票と、残る 2 項目（製品名と upgrade code の固定、予行演習）と、初回の導入の版の前提（TD-311）を足す。他の行は変えない。
- `docs/plans/test-matrices/2026-10-17-msi-distribution-docs.md`（新規、R3 の Test Design Matrix）: 手順の分岐ごとの期待と、それを壊す誤った読み。

型・JSON・関数・DTO の形は変えない（作る所・読む所・fixture・test の呼び方の列挙は不要）。

他の lane との所有（`docs/DEV_WORKFLOW.md` Wave Operation）:

| file | 節 | 所有 |
|---|---|---|
| `docs/backlog.md` | 「MSI 配布手順 docs 化」の項目とその下の箇条 | 本 lane |
| `docs/backlog.md` | それ以外の項目（日報の紙、SD の一括取込み、単位の語の直し、JAN の別名、SD 直読みの runtime の各項目を含む） | 各 lane（本 lane は触らない） |
| `docs/PROJECT_HANDOFF.md` | 案内の表の手順書の行 | 本 lane（他の 5 本の発注書に PROJECT_HANDOFF は出ない。`rg` で確かめた） |
| `docs/plans/test-matrices/2026-10-17-msi-distribution-docs.md` | 全体 | 本 lane（新規） |

## Non-scope

- `src-tauri/tauri.conf.json`・`Cargo.toml`・`package.json` の変更（製品名・upgrade code・版・署名・WebView2）。
- `docs/migration-recovery-runbook.md`・`docs/function-design/22-mnt-migration.md`・71・68 の変更。
- `docs/DEV_SETUP_CHECKLIST.md` §4.6（「Phase 2 完了時に MSI ビルドを生成して動作確認」の旧い記述は履歴として残す）。
- 予行演習の実施、店の PC での導入。
- `docs/decision-log.md`（手順書が足した安全の判断は手順書に置き、Contract Ledger で引く正本と組にする。owner の決定 TD-311 は手順書の「初回の導入の前に決めること」と backlog の MSI の項目に置く）。

## Acceptance Criteria

- AC1: `bash scripts/doc-consistency-check.sh` が exit 0（`結果: 全チェック通過`）。起票の前の baseline（2026-10-17、本 branch の起点 `f5d7aaf4`）: 全チェック通過、WARN 0。
- AC2: `bash scripts/doc-consistency-check.sh --target plan` が exit 0。
- AC3: `rg -n 'msi-distribution-runbook.md' docs/PROJECT_HANDOFF.md docs/backlog.md` が両 file で hit する。baseline（起点 `f5d7aaf4`）: hit 無し（`rg exit=1`）。
- AC4: `rg -n '^## ' docs/msi-distribution-runbook.md` が「今の事実」「初回の導入の前に決めること」「配布の記録」「流れ 1」〜「流れ 5」「確かめる項目」を出す。baseline: file が無い（`rg exit=2`）。
- AC5（読んで確かめる）: 手順書 `docs/msi-distribution-runbook.md` の「未確認（M*n*）」がすべて「確かめる項目」の表の行を指し、表の各行に確かめ方と結果で変わる所がある。流れ 5 が「今の DB の版が前の版の扱える版を超えたら、前の版のアプリは今の DB を開けない（MSI の入れ直しだけでは戻せない）」と「流れ 3 の backup〈前の版で作った backup〉の写しを使う」を書いている（Goal の失敗定義の 1・2 行）。
- AC6: `rg -n '移す|移し|置く|置き|置いて|置か' docs/msi-distribution-runbook.md | rg -v 'runbook の手順 [56]'` が何も出さない（後ろの `rg` の exit 1）。DB を動かす語が 22 §16 の回復の手順 5・6 の引用の外に出ないことを見る（Goal の失敗定義の 4 行目）。baseline（`3e132095`、round 1 の是正の前）: 5・22・72・82・97・102・117 行の 7 行が出た（97 行は「入力した後」に決める前に移す段で、Plan Review round 1 の P1 の箇所）。round 2 の是正の後: 前の `rg` が出すのは 117 行（runbook の手順 6 を引く行）だけで、後ろの `rg` は何も出さない。round 3 の是正の後: 前の `rg` が出すのは 127 行（同じ行）だけで、後ろの `rg` は何も出さない。確認の round の是正の後: 前の `rg` が出すのは 129 行（同じ行）だけで、後ろの `rg` は何も出さない。流れ 5 の「アプリが起動しないときの保全」は「写す」と書き、元の file を動かさない。予行演習の開発機の folder の退避と復帰は「退避」「別の名へ変えて保全」「名を戻す」と書き、この検査の対象の外（開発機の DB で、店の DB ではない）。
- AC7: `rg -n 'TD-311' docs/msi-distribution-runbook.md docs/backlog.md` が両 file で hit し、`rg -c '店の DB に当てない' docs/msi-distribution-runbook.md` が `1`（流れ 4）。初回の導入の版の前提と、作り直しを店の DB に当てないことを見る（Plan Review round 2 の Claude O1）。baseline（`ad57008c`、round 2 の是正の前）: 前の `rg` は exit 1、後ろの `rg -c` は exit 1（0 件）。
- AC8: `rg -n '戻す先と今の版で比べる|DB の版が同じとき|DB の版が上がったとき' docs/msi-distribution-runbook.md` が何も出さない（exit 1）。流れ 5 が配布の記録の版の比較で戻し方を決める撤回した形を残さないことを見る（Plan Review round 2 の Codex C1）。baseline（`ad57008c`）: 97・104・105 行の 3 行。

実測（2 回目の確認の是正の後、commit の前、2026-10-18。行番号は確認の round の是正の後と同じ。各 command を逐語で実行した。AC3・AC7 の hit の行は長いので先頭だけ写した。AC5 は読み取りで、手順書の流れ 5 の 2 と表の超える行が当たる。AC の数は 8 のまま）:

```text
AC1  bash scripts/doc-consistency-check.sh                → exit=0、結果: 全チェック通過（WARN 0）
AC2  bash scripts/doc-consistency-check.sh --target plan  → exit=0、結果: 全チェック通過（WARN 0）
AC3  docs/backlog.md:91:  - 手順書の lane を起票した（2026-10-17、R3〈Plan Review round 2 …
     docs/PROJECT_HANDOFF.md:14:| アプリの MSI を作る・入れる・更新する・前の版へ戻す管理者の手順 | …
AC4  7:## 今の事実 / 29:## 初回の導入の前に決めること / 36:## 配布の記録 / 52:## 流れ 1: MSI を作る /
     62:## 流れ 2: 初回の導入 / 73:## 流れ 3: 更新の前の backup / 86:## 流れ 4: 版を更新する /
     100:## 流れ 5: 起動が止まったとき・不具合のときに前へ進むか戻す / 138:## 確かめる項目
AC6  前の rg: 129 行（runbook の手順 6 を引く行）だけ。後ろの rg:（出力なし）rg exit=1
AC7  docs/backlog.md:94:  - 前提（owner の決定、repo 外の回答台帳 TD-311、2026-10-17）…
     docs/msi-distribution-runbook.md:32:- **初回の導入（リリース）の版は、…（TD-311）…
     rg -c '店の DB に当てない' → 1
AC8  （出力なし）rg exit=1
```

## Design Readiness

- 引く設計正本（節まで）: 22 §3.2 MNT-03-D11（新しすぎる DB の拒否と回復）と処理ステップ 5・6（migration ごとの COMMIT）、22 §12（作業フォルダーの旧 DB の取込み）、22 §16 の「回復」7 手順と vU・vB の「止まった後の進め方」・「DB の作り直しを 1 回にまとめる順」、D-119（本番の DB は無い、作り直しは開発・試験 DB だけ）、`docs/migration-recovery-runbook.md`（22 §16 の写し）、71 §71.4（backup の file 名）・§71.5（保持日数での削除）・§71.9（起動時の自動の backup が migration の後）、68 §68.4（手動バックアップ・実効保存先）。
- 必要な設計成果物: Design artifact selection の当たる行は無い（振舞い・command・DB・画面・format を変えない）。管理者の手順書を新しく置く（updated in this PR、`docs/msi-distribution-runbook.md`）。
- plan にしかない durable な判断の昇格先: 無い。手順書が足した安全側の判断（予行演習の後に初回の導入、初回の導入の版は作り直しの起点の migration を全部含む〈TD-311〉、backup が失敗したら更新しない、照合が全部合うまで入力を始めない、v8 より後の戻す手順が無ければ DB を戻さない、前の版の MSI と更新の前の backup を戻す可能性を閉じるまで保つ、場面で今の DB の版を、戻す先の backup を基準に入力の有無を知って表の行で前へ進むか戻すかを決め、決めるまで今の DB を動かさない、起動しないときは DB 一式を写して保全し修正版を流れ 4 の手順 2 から入れる、予行演習の退避と復帰の順）は手順書に置き、引く正本との組を Contract Ledger に置く。
- 今の DB の版の出所（code で確かめた）: 新しい版の migration は 1 つずつ COMMIT し、失敗した migration だけが戻る（`src-tauri/src/db/migration.rs:175`〜`:204` の `migrate`、22 §3.2 の処理ステップ 5）。COMMIT の後の後始末の失敗でも `MigrationFailed` で起動が止まり、版は記録済み（22 §16「回復」、`test_migration_req903_v8_commit_and_fk_restore_failures`〈`migration.rs:1578`〉）。前の版の起動の拒否の文言は末尾に「（データの版: *N*、このアプリが扱える版: *M*）」を持つ（`src-tauri/src/db/mod.rs:124`〜`:131`、`src-tauri/src/lib.rs:62`〜`:64`、`test_startup_req903_d11_newer_schema_stops_startup_with_own_message`〈`lib.rs:828`〉）。起動を止めた文言は、診断ログが使えればそこにも記録される（`lib.rs:201`〜`:203` の `record_pre_window_fatal`、`lib.rs:1227` の `log_dir`）。データの folder の確認・作成（`lib.rs:1214`〜`:1223`）はログの初期化（`lib.rs:1225`〜`:1233`）より前で、ログの初期化の失敗は起動を続ける（`lib.rs:1231`〜`:1233`、22 §12.4 の MNT-03-D6）ので、手順書は画面の文言を全文控えることを正にし、ログを控えの出所にしない。新しい版の起動の失敗の文言（`DatabaseInit`、`lib.rs:58`〜`:60`）は失敗した migration の番号（`migration.rs:142`〜`:161` の `v{n} …`）を持つが、COMMIT の後の失敗があるので版の出所にしない。
- 前提・制約と延期した gap: 実機の MSI の振舞い M1〜M9 は未確認で、予行演習（backlog）へ延期する。延期しても安全な理由: 手順書は未確認の点を手順の前提にせず、予行演習（M1〜M8）と店の PC の M9 の前に店の PC へ入れないと書いている。製品名と upgrade code の固定は設定の lane へ延期（初回の導入の前提）。
- 絶対保証の自己点検: 手順書の「データは変わっていない」は MNT-03-D11 の「論理内容を変えない」を引く（SQLite の open による物理的な書込みは 22 §3.2 の同じ節が扱う。手順書の表も「論理内容を変えずに」と書く）。「前の版は今の DB を開けない」と「新しい版で作った backup は前の版で開けない」は DB の版が前の版の扱える版を超えた場合に限って書いた。
- 判定: ready。

## Registration / Generation Obligations

| 変更対象 | 義務 | 本 lane |
|---|---|---|
| source / workflow doc 新設 | 親文書の目次・索引を更新 | `docs/PROJECT_HANDOFF.md` の案内の表に 1 行（Scope） |

ほかの行（Tauri command、function-design doc、REQ、route、operator 画面）は該当なし。

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | 外部の adapter と app-core の境 | Architecture | 不該当: レジ・POS の adapter に触れない | なし |
| Fact check / design decision split | 観測した事実とアプリの判断の分け | 手順書の「今の事実」 | 当たる: repo の現物と Tauri の公式の資料で確かめた事実を出所つきで表にし、実機の振舞いは M1〜M9 の未確認に分けた | 予行演習（backlog） |
| Lifecycle / retry | 前・中・後・失敗の後 | 手順書の流れ 3〜5 | 当たる: 更新の前の backup、起動の拒否、起動の後の照合、migration の後に前へ進むか戻すかを、場面（新しい版が 1 度でも照合まで進んだ／最初の起動で止まった）と今の DB の版と入力の前・後で分けた。起動しないときの前進は保全の後に修正版を流れ 4 の手順 2 から | Test Design Matrix の State Lifecycle Matrix |
| Operator workflow | 実際の順序 | 手順書 | 当たる（管理者の作業）: 作る → 入れる → backup → 更新 → 戻す の列。店の利用者の操作にしない | なし |
| Replacement path | 外部の系が変わったとき | — | 不該当: installer の方式（MSI）を変える判断は無い | なし |
| Data safety / evidence | 店のデータを repo に入れない | 手順書の「配布の記録」と M の表 | 当たる: 配布の記録は repo の外、予行演習は試験用のデータだけ | なし |
| Reporting / accounting semantics | 集計の意味 | — | 不該当: 集計に触れない | なし |
| Manual verification | 自動 test で示せない主張 | 手順書の M1〜M9 | 当たる: MSI の振舞いはすべて Windows の実機でしか分からない。後で 1 回にまとめる（owner の手が 2026-10-20 から使いにくい） | 予行演習（backlog） |
| 環境・再現性 | 新しい環境依存の pin | — | 当たる: MSI の build は Windows と WiX に依る。CI に入れず、開発用の Windows の clone で作る（明示的に defer）。Windows の toolchain の pin は本 lane で足さない | なし |

## Boundary / Wire Contract

変えない。JSON・CSV・config・manifest・DTO・bindings・DB に触れない（`src-tauri/tauri.conf.json` は読んだだけ）。

## Test Plan

Test Design Matrix: [2026-10-17-msi-distribution-docs](test-matrices/2026-10-17-msi-distribution-docs.md)（手順の分岐ごとの期待と、それを壊す誤った読み。自動 test は足さない）。

- targeted tests: 無い（code を変えない）。手順が引く振舞いの既存の自動 test は Matrix の最後の 2 行。
- negative tests: AC5 の読み取り（失敗定義の 1・2 行に当たる記述が無いこと）、AC6・AC8（撤回した形が残らないこと）。
- compatibility checks: AC1・AC2 の doc 検査（R3 の Markdown link の実在を含む）。
- data safety checks: `git diff --cached --name-only` で docs の file（手順書・PROJECT_HANDOFF・backlog・本 packet・Matrix）だけを commit する。手順書と packet に店の運用の細部・金額・商品名を書かない。
- main wiring/integration checks: AC3（案内から辿れる）。

## Review Focus

- Goal の失敗定義の 4 行に当たる記述が手順書に無いか。特に流れ 5 の分け方（入力の前・後、今の DB の版が前の版の扱える版以下か、前へ進むか戻すか）と、「場面を決める」の 2 つの場面（場面 2 は 3 つの結果）が 22 §3.2 の処理ステップ 5・6 と 22 §16「回復」に合っているか、v8 より後の migration の扱い、起動しないときの保全と前進が AC6 と 22 §16 の手順 5・6 に食い違わないか。
- D-119 の前提（本番の DB はまだ無い。vB は行の無い DB にだけ当たり、行のある開発・試験 DB は作り直す）との整合: 初回の導入（リリース）が本番の DB を作る時点で、その版が作り直しの起点の migration を全部含む（TD-311）ので、店の DB が vB と D-118 の棚卸しの runtime の migration で止まって作り直しの対象になる経路が無いか。流れ 4・5 のどの枝も店の DB を作り直しへ送らないか。
- Contract Ledger の各行で、手順書が足した判断が引く正本の契約を崩していないか（R3 の Contract Audit）。
- 「今の事実」の表の各行が出所（repo の現物・公式の資料）と合っているか。公式の資料で確かめていない振舞いが「未確認」になっているか。
- 製品名と upgrade code の固定を初回の導入の前提にしたことの当否。
- Q1（owner 向け、後続の設定の lane で聞く。本 lane の完了に要らない）: 店の PC のスタートメニューに出るアプリの名を何にするか。選択肢 a = 今の雛形の名（`inventory-system-tauri-scaffold`）のまま upgrade code だけ固定、b = 日本語の名を owner が決める、c = 英字の短い名を owner が決める。推奨 b（利用者が見つけやすい）。答えが無い間の既定: 店の PC に入れない（手順書の「初回の導入の前に決めること」）。

## Contract Ledger

手順書が足した判断（「手順書の箇所」の列）と、それが引く正本の契約の組。手順書は正本を変えない（Non-scope）。触る正本の節の隣接の契約は sweep して、Scope が触り得るものを足すか除外を書いた（最後の 2 行）。Matrix の C*n* は Test Design Matrix の Contracts Under Test。

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| MNT-03-D11（新しすぎる DB を論理的な書込みの前に拒否、文言の末尾に版） | 22 §3.2 MNT-03-D11、§12.4 の文言の表 | 手順書の今の事実の「新しすぎる DB」、流れ 4 の「より新しい版」の行（「扱える DB の版」が文言の「データの版: *N*」と同じで照合が全部合ったことのある版の MSI を選び、入れ直した後に照合する）、流れ 5 の場面 2（前の版を起動して版を知る）と 1 の末尾（Matrix C2・C10） | `test_startup_req903_d11_newer_schema_stops_startup_with_own_message`、`test_migrate_req903_d11_rejects_newer_schema_without_writes` | 予行演習で前の版の拒否の文言を見る組は無い（Matrix の Residual Test Gaps） |
| 22 §3.2 の処理ステップ 5・6 と MNT-03-D1（migration ごとの COMMIT、COMMIT の後の失敗は版が記録済み） | 22 §3.2、22 §16「回復」 | 手順書の今の事実の「migration の当たり方」、流れ 5 の「場面を決める」（場面 1 は新しい版の扱える版、場面 2 は文言の migration の番号から版を決めない）（Matrix C2） | `test_migration_req903_v8_commit_and_fk_restore_failures` | 非対象 |
| 22 §16「回復」の「旧版でそのまま開けるのは、残った版が旧版のアプリの最大以下のとき」 | 22 §16「回復」 | 流れ 5 の冒頭と表（今の DB の版と入力の有無で分け、どの場面も表の行で決める）、知る手段が無いときは配布の記録の版で安全側（Matrix C2） | AC8（撤回した形が残らない） | 読み取り（Final Review） |
| 22 §16「回復」の 7 手順（手順 5・6 の外で DB を動かさない） | 22 §16「回復」、`docs/migration-recovery-runbook.md` | 流れ 5 の 2（runbook を引くだけ）、流れ 5 の冒頭の「決めるまで今の DB を動かさない」、アプリが起動しないときの保全（元の file を残して写す）と 3（Matrix C1・C11） | AC6 | 非対象 |
| v8 より後の migration から戻す手順を一般化しない | 22 §16「回復」（vU の例）、runbook の「いつ使うか」 | 流れ 5 の 2 の 1 つ目の箇条（Matrix C9） | 非対象（読み取り） | 非対象 |
| MNT-03-D14（vB）の「止まった後の進め方」と作り直しの手順、「DB の作り直しを 1 回にまとめる順」、D-119 の Guarantee range（作り直しは開発・試験 DB だけ）＋ owner の決定 TD-311 | 22 §16、D-119 | 初回の導入の前に決めること（初回の導入の版は作り直しの起点の migration を全部含む）、流れ 4 の「店の DB に当てない」、backlog の MSI の項目の前提（Matrix C3） | AC7 | 非対象 |
| 71 §71.9（起動時の自動の backup は migration の後）と 22 §16「回復」の手順 1 | 71 §71.9、22 §16 | 流れ 3（前の版で手動 backup、毎回）（Matrix C5） | 非対象（読み取り） | 非対象 |
| 71 §71.5（保持日数での削除） | 71 §71.5 | 流れ 3 の手順 4（保存先の外の 2 か所）、流れ 3 の末尾（失敗したら更新しない）、配布の記録の保管の規則（Matrix C5・C6） | 非対象（読み取り） | 非対象 |
| 68 §68.4（手動バックアップ・実効保存先） | 68 §68.4 | 流れ 2 の手順 5、流れ 3 の手順 3、流れ 5 の 1（新しい版で手動 backup） | 非対象 | 非対象 |
| 22 §12.4 の MNT-03-D6（ログの初期化の前の失敗は画面だけ）と診断ログの初期化の失敗で起動を続けること | 22 §12.4、`lib.rs:1214`〜`:1233` | 今の事実の「起動を止めた文言の控え」、流れ 2 の手順 4、流れ 4 の「起動が止まったとき」（画面の文言を全文控える。ログを控えの出所にしない） | 非対象（読み取り） | 非対象 |
| 22 §12（作業フォルダーの旧 DB の取込み）、MNT-03-D4 | 22 §12.2・§12.4 | 流れ 2 の手順 3（起動の前の独立した段、あれば起動しない）、予行演習の準備（版 A の最初の起動の前、あれば起動しない）（Matrix C7） | 非対象（読み取り） | M2（ショートカットの作業フォルダー） |
| 手順書が足した判断: 照合が全部合ったときだけ業務の入力を始める。前へ進む修正版の照合も全項目の一致を求め、owner が在庫の数を見て決めるのは今入っている版が起動しないうえに入力の後の経路だけ | 引く正本: 22 §16「回復」（入力の後は保全した file にだけ残る） | 流れ 4 の手順 5、流れ 3 の手順 2、流れ 5 の表の下の「前へ進むときの照合」（入力が無ければ控え直さない、入力の後に起動する版で流れ 3 をし直したらその控えと比べる）（Matrix C4） | 非対象（読み取り） | 非対象 |
| 手順書が足した判断: 流れ 5 の入力の有無は、戻す先の backup（最後に流れ 3 を行った前の版が作った backup）より後に入力したかの 1 つで決め、修正版を重ねても、入力を始めてよいと決まったことのある版で流れ 3 をし直すまで変えない。照合で入力を始めてよいと決まっていない版で流れ 3 をし直したときは、前の版と戻す先の backup を変えず、そのとき作った backup は追加の写しとして保つ | 引く正本: 22 §16「回復」（戻した後に残らない入力は、手順 5 で保全した file にだけ残る） | 流れ 5 の「前の版と入力の基準」と、それを引く場面 1・2、表、2、3 の手順 3、流れ 3 の手順 5（Matrix C13） | 非対象（読み取り） | 非対象 |
| 手順書が足した判断: アプリが起動しないときに前へ進む（DB 一式を元の file を残して 2 か所へ写し SHA-256 を照らす → 更新の前の backup と前の版の MSI を保つ → 修正版を流れ 4 の手順 2 から） | 引く正本: 22 §16「回復」の手順 5・6 の外で DB を動かさない、71 §71.9（起動しないアプリでは backup を作れない） | 流れ 5 の「アプリが起動しないときの保全」と 3、表の「前へ進むとき」の列（Matrix C11） | 非対象（読み取り） | 非対象 |
| 手順書が足した判断: 初回の導入の条件（製品名と upgrade code の固定、予行演習 M1〜M8 と店の PC の M9、`identifier` を変えない、版を毎回上げる） | 引く正本: Tauri の公式の資料（今の事実の表の出所）、22 §16 の作り直しの手順の「アプリの data の folder」 | 初回の導入の前に決めること、流れ 2 の前提 | 非対象 | M1〜M9（予行演習、後続の項目） |
| 手順書が足した判断: 予行演習の退避と復帰（全プロセスの停止 → `%APPDATA%` と `%LOCALAPPDATA%` の 2 つの folder を別名で保全 → 元の名が空いていること → 戻して確かめる）と、M6 の合格条件に試験データの中身が同じであること | 引く正本: 22 §16 の作り直しの手順（folder ごと丸ごと扱う、アプリを止める）と「アプリの data の folder」（WebView の保存の folder を含む） | 確かめる項目の準備と片付け、M6 の行（Matrix C8・C12） | 非対象（読み取り） | 予行演習で 1 度通す |
| 隣接の除外: 71 §71.7（アプリの中の復元、新しい版のまま古い時点へ戻す経路）、MNT-03-D1 の ROLLBACK 失敗の扱い、vU の上限の検査の「止まった後の進め方」（更新を保留、R4 の判断） | 71 §71.7、22 §3.2、22 §16 の vU | 変えない（流れ 5 の 2 が「アプリの中の復元も前の版へは戻せない」と引くだけ。vU で止まったときは流れ 4 の「それ以外の文言」の行の前進か戻すかの判断に入る） | 既存 | 非対象 |
| 隣接の除外: D-118 の SPEC-STK-CNT-D10（作り直しの承認）、停止 ADR SPEC-STOP-D1 | D-118 の ADR、停止 ADR | 変えない（初回の導入の版の前提が D-118 の migration を名で引くだけ） | 非対象 | 非対象 |

## Contract Probe

R3 の Contract Probe（`DEV_WORKFLOW.md:63`）は、外部の前提に頼る計画で Plan Gate の前に最小の実験を求める。本 lane の成果物（手順書）は MSI の振舞い（M1〜M9）を手順の前提にせず、「未確認」と書いて、予行演習（M1〜M8）と店の PC の M9 の前に店の PC へ入れないと決めている（手順書の「初回の導入の前に決めること」と流れ 2 の前提）。よって本 lane の Plan Gate はこれらの実験に依らず、実験は予行演習（backlog）で行う。実験は Windows の実機が要り、本 run では行っていない。手順書の M1〜M9 と同じ。下の公式の資料の確かめは 2026-10-17 に行い、upgrade code と `allowDowngrades` の 2 行は 2026-10-18 に同じ頁で確かめ直した。

- MSI の出来る場所・file の名前・NSIS も出来るか: 開発用の Windows で流れ 1 → 未実施（M1）
- MSI の導入で UAC が出るか・入る場所: 試しに入れる → 未実施（M2）。公式の資料（2026-10-17、Windows Installer の頁）は install mode を NSIS についてだけ書き、MSI の範囲を書いていない
- USB で写した MSI で SmartScreen が出るか: browser を通さない file で入れる → 未実施（M3）。公式の資料（同日、code signing の頁）は「browser から download したとき」の警告を書く
- 版を上げた MSI の置き換えとデータの残り、upgrade code の固定が効くか: 版 A → 版 B、固定した設定で製品名だけを変えた MSI → 未実施（M4）。upgrade code が `productName` から作られる（UUID v5）ことは公式の資料（config の WixConfig「By default, tauri generates this code by generating a Uuid v5」）で確かめた。製品名を変えると別のアプリとして並ぶ見込みはこの資料から導き、実機では確かめない（固定の後は起きない）
- 同じ版の再導入: → 未実施（M5）
- 古い版の上書き: `allowDowngrades` の既定 `true` は公式の資料（同日、config の WindowsConfig）で確かめた。実際の上書き → 未実施（M6）
- アンインストールでデータの folder が残るか: → 未実施（M7）。公式の資料（同日、Windows Installer の頁）に記述が無い
- 動いたままの更新: → 未実施（M8）
- 店の PC の WebView2: 店の PC で見る → 未実施（M9）

## Data Safety

- commit しない: 店の DB・backup・MSI・配布の記録・ログ。
- repo の外にだけ置く: 配布の記録（管理者の保管場所）。
- 試験用だけ: 予行演習のデータは `seed_demo_data` の決定的な試験データ。全プロセスを止めてから開発機の `%APPDATA%\com.kosei.inventory` と `%LOCALAPPDATA%\com.kosei.inventory` を別名へ退避し、版 A の最初の起動の前に作業フォルダーに `inventory.db` が無いことを確かめ、版 A と同じ commit で `--db` を明示して投入する。終わったら、全プロセスの停止 → 試験用の 2 つの folder を別名で保全 → 元の名が空いていることを確かめる → 退避した folder の名を戻して確かめる、の順で戻す（手順書の「確かめる項目」の準備と片付け）。
- 店の DB: 手順書は店の DB を作り直す段を持たない。初回の導入の版は作り直しの起点の migration を全部含み、リリースの後は作り直さない（owner の決定、repo 外の回答台帳 TD-311）。

## Implementation Results

- 結果: MSI を作る・初回に入れる・更新の前に backup を取る・更新する・前の版へ戻す管理者の手順書 `docs/msi-distribution-runbook.md` を置き、`docs/PROJECT_HANDOFF.md` の案内から辿れるようにした。手順書は既存の契約（MNT-03-D11、22 §16 の回復、71・68 の backup）を引いて並べ、正本に無い店の DB の安全の判断（流れ 5 の場面と表、アプリが起動しないときの保全と前進、更新を始めない条件、起動の後の照合、保管の規則、初回の導入の条件〈TD-311〉、予行演習の退避と復帰）を足し、引く正本との組を Contract Ledger に置いた。実機の MSI の振舞いは M1〜M9 の未確認として手順書に書き、予行演習（backlog）へ送った。code・設定（`src-tauri/tauri.conf.json`）・設計正本は変えていない。
- 既存の保護: アプリの振舞いと設定は変えていない。
- 未検証: M1〜M8（開発用の Windows での予行演習）と M9（店の PC）。製品名と upgrade code の固定は後続の設定の lane で、owner への Q1（製品名）はその lane で聞く。
- review・CI・merge（closeout、2026-10-18）: Plan Review は round 3（天井）の後に owner の決定で確認の round を 2 回行い（TD-315・TD-316）、2 回目の確認の残りを owner の決定（TD-318）で直して plan-approved。Final Review は broad 1 回と是正の後の closure（下の Review Response の Closeout の段落）。helper の record は review = pass。hosted の `Merge gate` は Ready の後の run で success で、owner の Ready・merge の承認の後に helper 経由の squash merge。PR: [#170](https://github.com/higekiri-w/inventory-system-desktop/pull/170)
- 後続: `docs/backlog.md` の「MSI 配布手順 docs 化」の残りの 2 項目（製品名と upgrade code の固定、予行演習）と、v8 より後の migration を足す lane の義務（Final Review の是正で backlog に入った）。

## Review Response

### Plan Review round 1（2026-10-17、head `3e132095`）

Claude 側 fresh Opus 5.5（reject、P1 0・P2 1・P3 6）と Codex GPT-6.1 Sol（reject、P1 1・P2 2・P3 2）。Claude #1 と Codex #1 は同じ件で、重い方の P1 とした。全て採った（Coordinator の裁定）。是正の前に相談役（Fable 5.1）に反例探しを頼み、是正の原理を「手順書に新しい安全の契約を足さず、22 §16 の回復の番号が無い段は置かない」にした（相談役は、案にあった「決める前に写しを作る段」も 22 §16 に無いと指摘し、外した）。

| # | 出所 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|
| R1 | Codex #1（P1）・Claude #1（P2） | 採る | 入力の後の枝が、回復の方針を決める前・止める前・遺物を確かめる前に DB を移させる。版が同じ後退では空の DB ができ、データが 2 つに分かれる | 流れ 5 を「DB の版が同じ／上がった」×「入力の前／後」の表にし、決めるまで今の DB を動かさない形にした。保全は runbook の手順 5 だけ。AC6（「移す」「置く」が runbook の手順 5・6 の外に出ない）を足した |
| R2 | Codex #2（P2） | 採る | 更新の前の backup の写しの保持が起動の後の確認までで終わり、数日後に戻せない | 前の版の MSI と更新の前の backup の写しを、owner が「その版へ戻す可能性を閉じる」と決めるまで一緒に保つと書いた |
| R3 | Codex #3（P2）・相談役 | 採る | 予行演習の試験データの投入先が決まっておらず、MSI のアプリと開発中のアプリが同じ DB を共有するので開発の DB に当たる。版 B の commit で投入すると版 A が起動しない | 準備と片付け（データの folder の退避と復帰、版 A の commit で `--db` を明示して投入、`--reset` を使わない、作業フォルダに `inventory.db` を置かない）を足した |
| R4 | 相談役 | 採る | 開発 PC で確かめた写しと店 PC に置く写しが別物になり得る | 確かめた写しそのものを写し戻し、SHA-256 の一致を確かめてから置くと書いた |
| R5 | Claude #3（P3） | 採る | 入力の前後と DB の版の 2 つの枝が掛け合わされていない | R1 の表 |
| R6 | Claude #4（P3） | 採る | 流れ 1・2 に止まったときの行き先が無い | `light.exe` の失敗は VBSCRIPT を有効に（Tauri の公式の資料、2026-10-17）、SHA-256 が違えば入れない、起動しなければ止めて文言を控える |
| R7 | Claude #5（P3） | 採る | M9 の場所が食い違う | M1〜M8 は開発機、M9 は店の PC で初回の導入の前、に揃えた |
| R8 | Claude #6・Codex #5（P3） | 採る | 「何も書かずに」が MNT-03-D11 より強い | 「論理内容を変えずに」 |
| R9 | Claude #7（P3） | 採る | `identifier` を変えないことと、在庫の数の比べる相手が無い | 初回の導入の前に決めることに 1 文、流れ 3 で代表の商品の在庫の数を控える |
| R10 | Codex #4（P3） | 採る | 新しい版で作った backup が旧版で開けないことを無条件に書いている | DB の版が前の版の扱える版を超えたときに限った（流れ 3 は毎回のまま、22 の文は変えない） |
| R11 | Claude #2（P3） | 採る | R2 と R3 の境 | Risk の Reason に、新しい手順の順序が 22 §16 の 2〜5 を崩していないことを Final Review が確かめる旨を足した |

### Plan Review round 2（2026-10-18、head `ad57008c`）

Claude 側 fresh Opus 5.5（reject、P1 0・P2 1・P3 5）と Codex GPT-6.1 Astra（reject、P1 0・P2 4）。Codex #1 と Claude #5 は同じ件で、重い方の P2 とした。Codex #4 と Claude #6 は同じ件で、R3 へ上げた。全て採った（Coordinator の裁定）。起草役が `c68f6504` で直した。

| # | 出所 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|
| R12 | Claude #1（P2） | 採る | 店の PC へ初めて入れる版が作り直しの起点の migration を含まないと、後の更新がそこで止まり、店の DB を作り直す経路になる | 初回の導入の前に決めることに、初回の導入をリリースとし、その版は作り直しの起点の migration を全部含む（owner の決定、repo 外の回答台帳 TD-311）を足した。流れ 4 に「作り直しは開発・試験 DB の手順で、店の DB に当てない」。backlog にも同じ前提 |
| R13 | Codex #1（P2）・Claude #5（P3） | 採る | 戻し方を配布の記録の最大の版で決めていて、migration が失敗して DB の版が変わらなかったときに不要な差替えへ誘導する | 流れ 4 の起動が止まったときと流れ 5 を、今の DB の版を正とする形に段落ごと書き直した。版の知り方（起動できた／止まった → 前の版を起動して決める／分からない → 記録の版で安全側）を書き、表の行 1 に前の版を入れる前の手動バックアップを足した |
| R14 | Codex #2（P2） | 採る | 予行演習の後、試験用の folder が元の名を占めたまま退避した folder を戻させる | 全プロセスの停止 → 試験用の folder を別名で保全 → 元の名が空いていることを確かめる → 戻して確かめる、の順にした |
| R15 | Codex #3（P2） | 採る | 起動の後の照合が合わないときの停止と行き先が無い | 全部が合ったときだけ業務の入力を始め、合わなければ控えて止める、を流れ 4 の手順 5 に書いた |
| R16 | Codex #4（P2）・Claude #6（P3） | 採る（R3 へ上げる） | 手順書は正本に無い店の DB の安全の判断を足しており、`docs/DEV_WORKFLOW.md` の「R2 と R3 で迷えば data safety boundary に触れるなら R3」に当たる | Risk を R3 にし、Contract Ledger と Test Design Matrix を足した。Review Focus に D-119 の前提との整合を足した |
| R17 | Claude #2（P3） | 採る | M4 の後半は upgrade code の固定の後では製品名の見込みを確かめない | M4 を固定が効くことを見る形にし、見込みの出所を公式の資料にした |
| R18 | Claude #3（P3） | 採る | 流れ 1 の手順 1 に変更があったときの行動が無い。MSI の場所が未確認なのに事実として書いてある | `package-lock.json` だけなら戻す、それ以外なら配布しない。場所に（M1） |
| R19 | Claude #4（P3） | 採る | 作業フォルダーの確かめが起動の後に読める | 独立した手順にし、あれば起動せずに止める（ショートカットは MSI を入れた後にできるので、入れた後・起動の前に置く）。M2 にショートカットの作業フォルダーを見ることを足した |

起草役が出した問いの Coordinator の裁定（2026-10-18）:

- TD-311 の durable な記録: 並走の棚卸しの runtime の lane（D-123 の予定）が D-119 (8)・22 §16 を TD-311 に合わせて直す。本 lane は台帳の ID を引くだけにする。先に merge した側の 22 §16 の文に、後の側が参照を合わせる。
- 初回の導入（リリース）が原価の lane と棚卸しの runtime の配布の後になる帰結: owner に確かめる（本 round の review が通った後に他の packet の問いとまとめて聞く）。答えが無い間は手順書のとおり（作り直しの起点を全部含む版まで店の PC に入れない）。
- 前の版の拒否の文言を実機で見る組が予行演習に無いこと: Residual Test Gaps のまま残す（拒否の文言は自動 test が固定している）。round 3 で reviewer が足すべきと判断すれば足す。

### Plan Review round 3（2026-10-18、head `5416a14f`）

Claude 側 fresh Opus 5.5（reject、P1 0・P2 1・P3 3）と Codex GPT-6 Astra（reject、P1 0・P2 3・P3 1）。全て採った（Coordinator の裁定）。round の天井で P2 が残ったので owner に諮り、「全件を直し、指摘と差分だけを確かめる round を 1 回足す」と決まった（repo 外の回答台帳 TD-315）。起草役が `4dcd4a45` で直した。

| # | 出所 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|
| R20 | Claude #1（P2） | 採る | 流れ 5 の版の知り方の場面が最初の起動に限られず、入力の後に止まった場合も表を通らずに DB を移す手順へ進む | 流れ 5 を「場面を決める → 表の行で決める」に書き直した。場面 1 = 1 度でも起動の後の照合まで進んだ版、場面 2 = 最初の起動が止まった版。決めるまで今の DB を動かさないことをどの場面にも掛けた |
| R21 | Codex #1（P2） | 採る | 起動できないときの前進が、起動したアプリを前提にする流れ 3 を指す | 「アプリが起動しないときの保全」（元の file を残して写し、SHA-256 を照らす）と、流れ 5 の 3（保全 → backup と前の版の MSI を保つ → 修正版を流れ 4 の手順 2 から）を足した |
| R22 | Codex #2（P2） | 採る | 予行演習で作業フォルダーの旧 DB の確かめが最初の起動より後 | 確かめを版 A の最初の起動の前へ移し、あれば起動しない |
| R23 | Codex #3（P2） | 採る | M6・M7 が戻した後の DB の中身を判定しない | M6 の合格条件に、版 B で控えた商品・在庫・履歴が版 A で同じであることを足した |
| R24 | Claude #2（P3） | 採る | 「より新しい版」で止まったときの MSI の選び方が今入れた MSI を指しうる | 「扱える DB の版」が「データの版: *N*」以上の MSI にし、記録に無ければ止める |
| R25 | Claude #3（P3） | 採る | 2 か所への写しを確かめる方法が無い | 元と写しの SHA-256 を照らして記録し、違えば更新を始めない |
| R26 | Claude #4（P3） | 採る | 予行演習の退避が `%APPDATA%` 側だけ | `%LOCALAPPDATA%\com.kosei.inventory` を退避・復帰に足した |
| R27 | Codex #4（P3） | 採る | 起動を止めた文言が診断ログに必ず残ると書いている | 「診断ログが使えれば記録される。画面の文言は必ず全文控える」に揃えた |

起草役の問いの Coordinator の裁定（2026-10-18）: 修正版で入力の後に照合するとき、在庫の数は owner が見て入力を始めると決める形を採る（比べる控えが無い失敗の経路だけで、介入が増えるのはその場合だけ）。`%LOCALAPPDATA%` の folder を開発中のアプリも使うかは 22 §16 の未確認の前提のまま、予行演習の項目に足すかは後続の項目で決める。

### 確認の round（2026-10-18、head `306d11da`）

Codex GPT-6.1 Sol（reject、P1 0・P2 2）。round 3 の 8 件のうち 7 件は閉じ、1 件（round 3 の R20）は下の N1 と同じ原因で閉じていなかった。Claude 側はこの版では回さず、是正の後の版で確かめる（owner の決定 TD-316）。全て採った（Coordinator の裁定）。起草役が `f4f13ad6` で直した。

| # | 出所 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|
| R28 | Codex N1（P2） | 採る | 修正版の最初の起動も止まったときに場面 2（入力は無い）へ戻り、前の版の backup の後の入力が引き継がれない | 「前の版と入力の基準」を流れ 5 に 1 か所で定義した（前の版 = 最後に流れ 3 を行った版、入力の有無 = その backup より後に入力したか）。場面・表・1〜3 はそれを引く |
| R29 | Codex N2（P2） | 採る | 入力の後は一律に在庫の照合を外している | 修正版の照合も全項目の一致を求め、流れ 3 をし直せる場合はその控えと比べる。owner が在庫の数を見て決めるのは、今入っている版が起動せず入力の後の経路だけ |

起草役の問いの Coordinator の裁定（2026-10-18）: 起動はできるが照合が合わない版で流れ 3 をし直すと、その版が「前の版」になり、さらに修正版も止まると戻す先はその版の backup になる（さらに前の版へ戻す手順は持たない）。失うものの説明は事実と合い、戻す先の backup より前の状態へ戻す必要は今の範囲に無いので、変えない。

### 2 回目の確認（2026-10-18、head `2320900a`）

Codex GPT-6.1 Sol（approve、P1 0・P2 0・P3 0。round 3 の 8 件と N1・N2 の 10 件が全部閉じた）と Claude 側 fresh Opus 5.5（reject、P1 0・P2 1・P3 1。10 件は全部閉じた）。全て採った（Coordinator の裁定）。Claude 側の R1 は、確認の round で起草役が挙げた問いを Coordinator が確かめずに「変えない」と裁定していた点で、裁定の誤り（教訓を Coordinator の手順に入れた）。owner に諮り、「直して、追加の確認なしで plan-approved（跡は Final Review）」と決まった（repo 外の回答台帳 TD-318）。起草役が `140a11fe` で直した。

| # | 出所 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|
| R30 | Claude R1（P2） | 採る | 照合の合わない新しい版で流れ 3 をし直すと、前の版と戻す先の backup がその版のもの（照合の合わなかった状態）に替わり、戻すと説明と食い違う | 「前の版と入力の基準」に、流れ 4 で入れた版のうち入力を始めてよいと 1 度も決まっていない版で流れ 3 をし直しても、前の版と戻す先の backup は変えない（そのとき作った backup は追加の写し）を足した。起草役は発注の「照合が 1 度も全部合っていない」を「入力を始めてよいと決まっていない」に、対象を「流れ 4 で入れた版」に変えた。Coordinator が、起動しない版で owner が入力を始めると決めた組と、戻した前の版で入力した組を追い、発注の文言では入力を要らずに失うことを確かめて起草役の形を採った |
| R31 | Claude R2（P3） | 採る | 「より新しい版」で止まったときの MSI の候補が複数あるときの選び方が無い | 扱える DB の版がデータの版と同じで、照合が全部合ったことのある版から選び、入れ直した後に照合する、とした |

Closeout（2026-10-18）: Final Review は broad 1 回で、Claude 側（Fable 5.1）= approve（P3 3）、Codex（Astra）= reject（P2 1）。是正は、戻した後の照合、WebView の保存の folder を未確認の M7 に入れたこと、upgrade code を公式の CLI で印字して固定する 1 句。Claude 側の O2 は公式の頁で事実誤認と確かめて取り下げ、O3 は packet の判断を保ち、後続の migration の lane の義務として `docs/backlog.md` の「MSI 配布手順 docs 化」の項目に是正で入れた。是正の closure（Codex Sol）は approve（指摘なし）で、main の取込みの差分は Sonnet 5.5 の delta の確認 = 可。helper の record は review = pass。hosted の `Merge gate` は Ready の後の run で success で、owner の Ready・merge の承認の後に helper 経由で squash merge した。packet の本文は書き換えず、merge の版との食い違いをここに記録する: (1) AC の実測の段の AC4 の見出しの行番号は、是正で「確かめる項目」が 138 行目から 141 行目に移った（merge の版の手順書で `141:## 確かめる項目`）。Owner Effort Budget の表は closeout の時点の値（上限 9、消費 5）に直した。
