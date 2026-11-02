# Plan Packet: 店の PC で本物のデータを入れ始める初期投入の手順を決める

wave に属さない単独の lane（2026-10-23 に同時に起票した 2 本の 1 本。もう 1 本は操作ログの保持の lane `docs/archive/plans/2026-10-23-oplog-retention.md`〈別の branch〉。docs だけ）。導入の前の関門の candidate「初期投入の手順」（[Plans の導入の前の関門](../../Plans.md#導入の前の関門)、`docs/backlog.md` の同名の項目、owner の決定 repo 外の回答台帳 TD-345）を、関門に足す前提で起票する（owner の決定 repo 外の回答台帳 TD-402、2026-10-23）。

規模と手法（起票の最初に数えた）: 触る file は 3（新しい管理者の手順書 `docs/initial-load-runbook.md`、`docs/PROJECT_HANDOFF.md` の案内の 1 行、`docs/backlog.md` の「初期投入の手順」の項目）と本 packet と Test Design Matrix（`docs/archive/plans/test-matrices/2026-10-23-initial-load-procedure.md`）。設計正本（function-design・db-design・SCREEN_DESIGN・ADR・decision-log）・code・migration・wire の型は 0。手順書は既存の契約（26 §18.3、30 §4.8・§4.9 と BIZ-01-D5・D6・D8、60 UI-01c、master-tables の departments と独自コードルール、ADR SPEC-STK-CNT-D3・D4、68 §68.4、71 §71.5、D-123）を引いて並べ、正本に無い管理者の判断（行の種類の分け方、CSV の側の確かめ、予行演習、取り込む前の backup と照合、照合が合わないときの戻し方、入れた後の誤りの直し方）を足す。その組は下の Contract Ledger。手法: 1 lane のまま（手順書 1 つで、正本も code も変えないので分ける境が無い）。round 2 の是正で、owner の決定 J5 = b（repo 外の回答台帳 TD-421、2026-10-24）を受け、`docs/Plans.md` の導入の前の関門に 1 項と、その中身の backlog の項目を足した（触る file は 4。下の Scope）。round 2 の Codex の是正で、owner の決定（repo 外の回答台帳 TD-426、2026-10-24）を受け、関門の 8 の検査に `JANコード` の桁とチェックディジットを足した（触る file は変わらない）。

## Workflow State

- Phase: archive
- Risk: R4
- Plan Commit: 979f8e99ab7ebd399930447df1103b31c001af61
- Amendments: none
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（本 run の起草役）
- Plan Reviewer: fresh Opus 5.5 + Codex（座組表 docs/AGENT_OPERATING_MANUAL.md ## 座組 どおり）
- Final Reviewer: Fable 5（Claude 側、R3 以上。座組表の当面の Fable 5〈TD-410〉）+ Codex（座組表どおり）
- Final Review Minimum: 2
- Human Gate: ready,merge,r4
- Branch: agent/initial-load-procedure

遷移の記録:

1. kickoff → spec-check（2026-10-23、起草役）: 発注書の範囲（店の PC で本物のデータを入れ始める最初の手順〈商品・初期在庫・部門・設定・最初の backup〉の正本の置き場と、アプリの側に足りない機能があるか）を範囲にし、Risk を R3 と記録した（下の Risk）。
2. spec-check → design（2026-10-23、起草役）: 引く正本（上の規模と手法の一覧）は各機能の契約として足りるが、初期投入を 1 つに並べた手順が無い（30 BIZ-01-D8 は「Excel から作る CSV の換算は初期投入の作業の範囲」とだけ書く）。手順書を新しく置く。
3. design → plan-draft（2026-10-23、起草役、同じ commit）: 条件「設計の出力が正本にある」は、手順書 `docs/initial-load-runbook.md` を同じ plan-first の変更に置いて満たす（docs だけの lane の先例 `docs/archive/plans/2026-10-17-msi-distribution-docs.md` と同じ形）。条件「未解決の設計の問いが無い」は、owner の判断が要る 4 問（下の Review Focus の owner の判断事項）に全ての選択肢と推奨を並べ、推奨を既定として手順書を書き、答えで変わる所を手順書の表の「変わる所」に名指したことで満たす（owner の方針、repo 外の回答台帳 TD-401、2026-10-23）。実機でしか分からない点（P1・P2）は手順書に「未確認」と書き、予行演習で確かめる。
4. plan-draft → plan-gate（2026-10-24、Coordinator の代行）: 条件「packet complete and committed; Test Design Matrix committed for R3」は、packet・Matrix・手順書が起草役の commit `fac7575b` にあり、packet が template の全節と R3 の Contract Ledger・Data Safety・Contract Probe・Design Readiness を持つことで満たす。発注の直前に AC1〜AC6・AC8・AC9 の command を逐語で実行し、上の実測と同じ値（AC1 は exit 0・WARN 12 件、AC2・AC9 は exit 0）を確かめた。Contract Probe の P1・P2 は未実施で、Plan Gate（plan-approved）の前に要るかは Plan Review の問いに渡す。branch の起点 `eed07d37` の後に main が進んだ（#191・#195・#199・#200。`docs/backlog.md` が衝突する）ことは、base 同期の要否とともに Coordinator が決める。Plan Review round 1 へ出す。
5. plan-gate のまま是正（round 1、2026-10-24、起草役、`a9f0f439`）: round 1 の是正（Review Response 参照）。Risk を R4 に上げ（Final Review Minimum 2、Human Gate に `r4`）、Contract Probe の P2 を repo の中で回して記録した。手順書の方向（1 つの手順書、今のアプリのまま）は変えないので plan-gate に留め、round 2 で再 review する。
6. plan-gate のまま是正（round 2、2026-10-24、起草役、`ad709ba7`）: round 2 の Claude 側の是正と owner の決定 J5 = b（repo 外の回答台帳 TD-421）の反映（Review Response 参照）。初回の導入の版にプレビューの検査が入る（Plans の導入の前の関門の 8、runtime は別の lane）ので、手順書の流れ 1 の手順 4・5 の手作業の確かめをその分だけ外した。手順書の方向（1 つの手順書）は変えないので plan-gate に留め、Codex の round 2 で review する。
7. plan-gate のまま是正（round 2 の Codex、2026-10-24、起草役、`ea28d8a1`）: round 2 の Codex の是正と owner の決定 TD-426 の反映（Review Response 参照）。手順書の方向は変えないので plan-gate に留め、round 3（天井）で review する。
8. plan-gate のまま一括是正（round 3、2026-10-24、起草役、`aa8c251a`）: round 3（天井）の指摘の一括是正（Review Response 参照）。手順書の方向は変えないので plan-gate に留め、closure の review へ出す。
9. plan-gate のまま是正（closure の後、2026-10-25、起草役、`d9e9fcff`）: closure の指摘の是正（Review Response 参照）と base 同期（`origin/main` `8db97271` を merge。導入の前の関門の本 lane の項目は 6 から 8 になった）。手順書の方向は変えないので plan-gate に留める。
10. plan-gate のまま是正（closure 2、2026-10-26、起草役、`7174f0ba`）: closure 2 の Claude 側の reject（P2 1 / P3 3）を Coordinator の裁定（相談役の反例探しを入れた）に沿って直した（下の Review Response の closure 2）。設計の方向は変えないので plan-gate に留め、closure 3（Claude 側 fresh Opus 5.5 + Codex）に出す。
11. plan-gate のまま是正（closure 3、2026-10-26、起草役、`6f167bc1`）: closure 3 の 2 本の reject（B・G の未 closure、Claude 側の P3 5 件）を Coordinator の裁定（相談役の反例探しを入れた）に沿って直した（下の Review Response の closure 3）。同じ round の前に `origin/main`（`5506c7a4`）を単段で取り込んだ（`8ea82ab5`）。設計の方向は変えないので plan-gate に留め、closure 4（Claude 側 fresh Opus 5.5 + Codex）に出す。
12. plan-gate のまま closure 4 を記録し、P3 を直す（2026-10-26、Coordinator、`979f8e99`）: closure 4 の 2 本（Claude 側 fresh Opus 5.5 = approve P3 4、Codex = approve P1 / P2 / P3 0。Codex は是正の後の手順 5 の block を Windows PowerShell 5.1 で回した）が P1 / P2 = 0 を報告した（下の Review Response の closure 4）。Claude 側の P3 4 件と、Contract Probe P3・Matrix の Residual Test Gaps の「5.1 でまだ回していない」を、plan-gate のまま本 commit で直した（Plan Commit はこの版。前例: PR #197 の遷移 8・PR #201 の遷移 8）。
13. plan-gate → plan-approved（2026-10-26、Coordinator、本 commit）: 条件は `docs/DEV_WORKFLOW.md` の遷移表の「Writer でない独立の Plan Reviewer が P1 / P2 = 0」。closure 4 の 2 本（Claude 側 fresh Opus 5.5・Codex）が `6f167bc1` に P1 / P2 = 0 を報告し、その P3 を `979f8e99` で直した（遷移 12）。Plan Commit は `979f8e99`（前例: PR #197 の遷移 8）。
14. plan-approved → implementing（2026-10-26、Coordinator、本 commit。state だけ）: 遷移表の唯一の入口。本 lane は docs だけで、成果物の手順書は plan-first の変更に置いた（遷移 3）。Final Review の record は packet が implementing の head でしか受けない（helper）ので、plan-approved と同じ commit で進める（前例: PR #168・#202）。直後の commit で `origin/main`（PR #203 の merge の後）を単段で取り込み、その head を Final Review broad（Minimum 2）に出す。

## Owner Effort Budget

- 介入回数上限: 6（既定）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 6 | 2（J5 の答え〈repo 外の回答台帳 TD-421〉、関門の 8 の検査に JAN の桁とチェックディジットを足す決定〈同 TD-426〉、どちらも 2026-10-24） | 3（R4 の承認 1、Ready 1、merge 1） | 1 | 6 = 2 + 3 + 1 |

owner の判断事項 J1〜J4 は既定で進め、本 lane の完了に答えを要らない（答えが出れば手順書の該当の行を直す）ので、見込みに数えない。J5 は Plan Review round 1 の後に owner へ諮り、答え b を得た（介入 1 を消費）。TD-426 は Plan Review round 2 の後の owner の決定（介入 1 を消費）。手順書の予行演習（流れ 2）と店の PC での投入（流れ 3）は、初回の導入の版ができた後の運用で、本 lane の L3 ではない（下の Risk）。

承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R4

Reason:
手順書は店の本物のデータを入れる最初の手順で、取り込む前の backup を店の DB へ復元する判断（照合が合わないとき、その日に照合を終えられないとき）と、在庫 Excel から作った本物の商品 CSV を開発用の Windows に置いて予行演習する手順を定める。`docs/DEV_WORKFLOW.md` Risk Tiers の R4 の行（「backup restore, real POS/store data exposure」）に当たる。docs だけでも契約を決めるので R4 にする（先例: docs だけの backup の PC の外の控えの lane `docs/archive/plans/2026-10-07-backup-offsite.md` も R4）。入れた商品の記録は商品コードに積もって後から変えられず（SP-102-04、INV-8）、リリースの後は DB を作り直さない（D-123 (5)）。code・設定・DB・command の DTO・画面・merge gate は変えない。R4 の要件: R3 の Contract Ledger・Data Safety・Contract Probe・Test Design Matrix・Design Readiness に加え、Final Review Minimum 2、Human Gate `r4`、下の Rollback / recovery notes。classifier の workflow の path（`scripts/ci/classify-changes.sh`）に当たる file を Scope に持たない（下の AC の実測）。

Rollback / recovery notes（R4、`docs/DEV_WORKFLOW.md` Risk Tiers の R4 の行）: 本 lane は docs だけで、merge の後に戻すときは本 lane の commit の revert で足りる（code・DB・設定に触れない）。手順書が店の DB に対して持つ戻し方は 1 つだけで、業務の入力を始める前に、同じ日に作った取り込む前の backup をアプリの復元（68 §68.4。戻す前に今の状態の backup を自動で作る）で戻す（手順書の流れ 3 の手順 7）。その戻し方で商品と入出庫の記録が 0 件に戻ることは Contract Probe の P2 で code の上で確かめた。業務の入力を始めた後は戻さず、手順書の「入れた後に誤りを見つけたら」の表で直す。

L3 について: 本 lane の成果物は手順書で、画面も振舞いも変えないので、Windows native で確かめる本 lane の項目は無い。手順書の予行演習（流れ 2）は、初回の導入の版（Plans の導入の前の関門の confirmed を全部満たした版）と在庫 Excel の写しが揃った後に行う。MSI の予行演習 M1〜M8 は PR #191 で済んだので、同じ日に MSI の試験用のデータを使うときの境は手順書の流れ 2 の冒頭が持つ。

## Goal

Goal Invariant:

### 最小完了条件

- owner が `docs/PROJECT_HANDOFF.md` から辿れる 1 つの手順書で、在庫 Excel から商品 CSV を作る・予行演習で通す・店の PC で入れる・照合する・最初の backup を作る、を順に実行でき、各手順で何を確かめるか、合わなければ何をするかが分かる。
- owner の判断が要る所は既定と選択肢と「変わる所」が手順書にあり、答えが無くても手順を進められる。
- アプリの側に足りない機能があるかの答え（黙って入る 2 つの形と、`JANコード` の桁かチェックディジットが合わない行をプレビューが名指しで止める変更を初回の導入の版に入れる。手順書の J5 の答え b、repo 外の回答台帳 TD-421・TD-426。`docs/Plans.md` の導入の前の関門の 8 と、中身の backlog の項目）が、手順書・Plans・backlog に書かれている。

### 失敗定義

- 業務の入力を始めた後に、取り込む前の backup へ戻すように読める（その後の入力が消える）。
- 入れた後のやり直しを DB の作り直し（22 §16 の作り直しの手順）で行うように読める（D-123 (5)）。
- 照合が合わないのに業務の入力を始めるように読める、または取り込む前の backup を作らずに取り込むように読める。
- プレビューにエラー行があるまま取り込み、一部の行だけが入るように読める（30 §4.9 は正常な行だけを入れる）。
- 店の運用の細部・在庫 Excel の中身・店のデータを repo に書く。

### 非目的

- アプリの変更（商品 CSV の取込みの検査を足す、Excel を直接読む、売価の換算を持つ等）。取込みの検査は導入の前の関門の 8 の runtime の lane（backlog の「商品 CSV のプレビューで黙って入る形の行を止める」）が行う。
- 予行演習と店の PC での投入を本 lane で行う。
- 部門の初期データを変える（必要なら手順書の止まる所から別の lane）。
- PLU をレジへ書く手順（導入の前の関門の precondition-dependent の項目）、SD 直読み・日報の取込みの始め方。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

管理者（owner）の作業の列。アプリの中の操作は変えない。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 年末の棚卸しで埋めた在庫 Excel（J3 の既定）と、Excel を使える Windows がある（開発用の PC には Excel が無い。repo 外の回答台帳 TD-437） | 手順書の流れ 1（行の種類に分ける → 列を作る〈原価は店の決まりで丸める〉 → 表で確かめる → CSV で保存 → 保存した file を確かめる → SHA-256） | 商品 CSV と行の種類ごとの数 | 「どれにも当たらない」行と接頭辞の無い部門の独自コードの行が 0、`TEXT` の値が元の値と違う行（形式の選び違い、`m` の初期在庫の小数 3 桁以上、売価・個数の初期在庫の整数でない値、空のセルに値を書いた `TEXT`）が 0、丸めで原価が変わった行と 0 の行を一覧にし、5 つの数字の列（`商品コード`・`JANコード`・`売価`・`原価`・`初期在庫`）を全部の桁の文字列の値にし、数字だけの `商品コード` が `JANコード` と同じでない行と `JANコード` が 13 桁で `00000` から始まる行が 0（表の全部の行を数える）、CSV UTF-8 で保存し、保存した file の先頭が UTF-8 の BOM（`EF BB BF`。アプリの `decode_bytes` が UTF-8 として読む条件）で、5 つの列が手順 4 で決めた形でない行が 0 で、確かめた行の数が種類ごとの数の和と同じ | P1（Excel の CSV の保存がセルの文字のまま書くこと。安全を確かめに置かず〈repo 外の回答台帳 TD-437 の後の裁定〉、手順 5 が保存した file の中身を全部の行で確かめて外れた行を名指す。`TEXT` の丸めは置き換える前の元の値との比べが名指す。Contract Probe の P1） |
| 初回の導入の版の MSI と CSV と、元の在庫 Excel を開ける PC（照合に使う。開発用の PC には Excel が無い、TD-437）がある | 流れ 2（開発用の Windows で、開発のデータの folder を退避した空の DB に、取り込む前の backup → 取込み → 照合 → 復元で 0 件に戻る。手順 3〜7 は同じ日） | 同じ CSV と版で店の PC と同じ結果になることが分かる | プレビューのエラー行と重複行が 0（指数の形の値、13 桁の数字で `JANコード` が空の行、`JANコード` の桁かチェックディジットが合わない行はエラー行として名指される。J5 = b・TD-426）、照合が全部合い、同じ日のうちに復元で 0 件に戻る | P2（code の上で確かめた。画面の上は手順 7） |
| 店の PC に MSI が入り、商品が 0 件（MSI の手順書の流れ 2 の後）で、元の在庫 Excel を開ける PC（照合に使う）がある | 流れ 3（写して SHA-256 → 取り込む前の backup を 2 か所へ → 取込み → 開き直し → 照合 → 設定 → 最初の backup） | 店の PC で本物の商品と初期在庫が入り、業務の入力を始められる | 手順 2〜7 を同じ日に行い、照合が全部合う。合わないかその日に終えられなければ入力を始めずに取り込む前の backup へ復元し、合わなければ流れ 1 から | P2（code の上で確かめた） |
| 業務の入力を始めた後に誤りが見つかった | 「入れた後に誤りを見つけたら」の表 | 画面か CSV の上書き（今の `JANコード` を全部の行に入れる。空・列なしは JAN を消すが、商品コードが 13 桁の数字の商品の JAN はプレビューが止めるので消せない）か計数で直す。直せない誤り（商品コード・単位の種類・13 桁の数字の商品コードの商品の JAN を消すこと）は止めて owner に諮る | — | なし |

この lane で完了するのは「手順書が置かれ、owner が辿れる」まで。店の PC での投入は初回の導入の版と予行演習の後で、v1.0 の通常運用の達成はそれらを含む。

## Scope

- `docs/initial-load-runbook.md`（新規）: なぜ 1 度で入れ切るか、今の事実の表（repo の現物で確かめた出所つき）、入れる前に決めること（J1〜J4 の選択肢・既定・変わる所、J5 の選択肢・owner の答え・変わる所）、流れ 1（商品 CSV を作る）・流れ 2（予行演習）・流れ 3（店の PC で入れる）、入れた後に誤りを見つけたら。
- `docs/PROJECT_HANDOFF.md`: 案内の表に手順書の行を 1 行足す（template の Registration / Generation Obligations「source / workflow doc 新設」）。
- `docs/backlog.md`: 「初期投入の手順」の項目を、手順書への案内・業務の入力の前後の戻し方・残る作業に置き換え（「設計が無い」と「決めること」の列挙を外す）、そのすぐ下に導入の前の関門の 8 の runtime の項目「商品 CSV のプレビューで黙って入る形の行を止める」（R の見込み・触る正本・AC の要旨・起票で決める候補）を置く。他の行は変えない。
- `docs/Plans.md`: 導入の前の関門の confirmed に 8（商品 CSV のプレビューの検査、owner の決定 TD-421・TD-426）を 1 項足す。中身は backlog の上の項目を名で引く（既存の関門の 5 と同じ書き方）。D-097 は helper が `Plans.md` を読まないことと `## 次の行動` の pointer の規則で、関門の項目を足すことを妨げない。
- `docs/archive/plans/test-matrices/2026-10-23-initial-load-procedure.md`（新規、R4 の Test Design Matrix）: 手順の分岐ごとの期待と、それを壊す誤った読み。

型・JSON・関数・DTO の形は変えない（作る所・読む所・fixture・test の呼び方の列挙は不要）。

決めたこと（発注書の「決めること」）:

- 正本の置き場: 管理者の手順書 `docs/initial-load-runbook.md`。function-design には置かない。理由: 手順は 1 度きりの管理者の作業の列で、26・30・60・68・71・master-tables・ADR の契約を横に並べる。function-design の各 file は 1 つの module の契約を持つ所で、手順を置くと各 file に同じ列を写すか、1 つの file に他の module の手順が入る。先例は MSI の手順書（同じ管理者の手順で、PROJECT_HANDOFF から辿る）。
- アプリの側に足りない機能: 今の商品 CSV の取込み（新規の行の初期在庫の記録、CSV の中の重複の拒否、エラー行の除外、独自コードの番号を画面の発番が飛ばす D-106）と backup の復元で、手順は成り立つ（Contract Probe の P2）。ただし黙って入る形が 2 つある（`商品コード` が 13 桁の数字なのに `JANコード` が空の行、指数の形の値の行。preview で正常な行になり error も warning も出ない。P2 の probe で確かめた）。「問題は起きた画面で名指しする」（D-127 (2)）の形はプレビューが名指しで止めることで、手順書の J5 として owner に諮り、答え b（repo 外の回答台帳 TD-421、2026-10-24）を得た: プレビューがその行を名指しで止めて直し方を出す変更を初回の導入の版に入れる（`docs/Plans.md` の導入の前の関門の 8。runtime は別の lane で、中身は backlog の「商品 CSV のプレビューで黙って入る形の行を止める」）。round 2 の後に owner が同じ検査に `JANコード` が 8 桁・13 桁の数字でないかチェックディジットが合わない行を足した（repo 外の回答台帳 TD-426、2026-10-24。30 BIZ-01-D1 の手入力の検証と同じ判定を CSV の取込みにも当てる）。手順書の予行演習と店の PC での投入は関門の confirmed を全部満たした版で行うので、手順書はプレビューが止める形の手作業の確かめを持たず、「プレビューが名指した行を直し方のとおりに直す」形にした（流れ 1 の手順 4・流れ 2 の手順 4）。CSV の上書きが `JANコード` の空・列なしで JAN を消すこと（30 BIZ-01-D8 の上書きの段・§4.9.2 の意図した契約）は手順書に書き、JAN を消す行を名指す warning を同じ runtime の lane の起票で決める候補にした（AI の調べから出た候補で、owner の確認の前に必須にしない。AGENTS.md の Working Rules の「owner の所感は原文で扱う」の項）。関門の 8 の検査は上書きの候補の行にも当たる（`preview_import` は行の検証を重複の分類より前に行う）ので、商品コードが 13 桁の数字の商品の JAN を CSV の上書きで消す行は error になる。手順書はその JAN を消す訂正を示さず、商品コードの誤りと同じく止めて owner に諮る側に送った（Review Response の round 2 の Codex の #3）。

他の lane との所有（`docs/DEV_WORKFLOW.md` Wave Operation。open の PR #172・#188・#190・#191・#192・#193・#194 の差分〈#188 は packet の Scope、#194 は PR body の Scope〉と本 lane の予定の path を `comm -12` で突き合わせた。下の AC の実測）:

| file | 節 | 所有 |
|---|---|---|
| `docs/backlog.md` | 「初期投入の手順」の項目とその下の箇条 | 本 lane |
| `docs/backlog.md` | それ以外の項目（SD 直読みの runtime〈#172〉、PC の外の控えの runtime と保存と起動の守り〈#190〉、部門 12 の初期データ〈#192 が足す〉、#188 の Scope の各項目を含む） | 各 lane（本 lane は触らない） |
| `docs/backlog.md` | 「商品 CSV のプレビューで黙って入る形の行を止める」の項目（新規、「初期投入の手順」のすぐ下） | 本 lane |
| `docs/Plans.md` | 導入の前の関門の confirmed の 8（新規の 1 行） | 本 lane（open の #201 が同じ節の 5 の行を、#190 が 3 の行を変える〈2026-10-24 に `gh pr diff` で見た〉。隣の行なので、後に merge する側が base 同期で解く。#188 の merge で main に入った 6・7 の後に、2026-10-25 の base 同期で 8 として置いた） |
| `docs/PROJECT_HANDOFF.md` | 案内の表の手順書の行 | 本 lane（他の PR の差分と Scope に無い） |
| `docs/initial-load-runbook.md`・本 packet・Matrix | 全体 | 本 lane（新規） |

この変更で不要になる記述・条件・code（D-127 の問い）: backlog の「初期投入の手順」の項目の「設計が無い」と「決めること」の列挙（手順書の J1〜J5 と重なり、原価の 1/100 円は PR #184 で済んだ）と「入れた後のやり直しは DB の作り直しになり」は、手順書への案内と残る作業に置き換えた（上の Scope）。J5 = b（TD-421）で初回の導入の版のプレビューが止める形になったので、手順書の流れ 1 の手順 4・5 の手作業の確かめのうち、13 桁の数字の `商品コード` で `JANコード` が空の行の数え、保存した file のメモ帳での `E+` の検索と行の数、`商品コード` の重複と上限の確かめ（今のプレビューも行の error にする）を外した。backlog の後続の候補の行は、関門の runtime の項目に置き換えた。TD-426 で、backlog の項目の起票で決める候補から JAN の check digit の候補（旧 (3)）を外して確定の検査に移し、手順書の J4 の変わる所の「関門の 8 の検査が 8 桁に当たらない版では手順 4 で数える」の句と、流れ 1 の手順 4 の表示形式 `0` の別の道を外し、手順 4 の確かめの列を「数字だけの `商品コード` が `JANコード` と同じでない行」の 1 つにした（13 桁で空でなく違う行と J4 の a の 8 桁で空の行を 1 つの式で数える。先頭の 0 が落ちた 12 桁の行は、`商品コード` と `JANコード` が同じなら式が数えないので、手順 5 の形の確かめと関門 8 のプレビューが受け持つ）。桁の数どおりの `TEXT` で新しく生まれる誤り（8 桁の値に 13 桁の形式）は TD-426 の検査を通るので、手順 4 に確かめを 1 つ足した（backlog の候補 (4)）。30 BIZ-01-D1 と master-tables の「CSV の取込みは JAN の形の検証の対象外」の句は、関門の 8 の runtime の lane が置き換える（backlog の項目の触る正本。設計正本は本 lane の Non-scope）。30 BIZ-01-D8 の「Excel から作る CSV の換算は初期投入の作業の範囲で、本契約は CSV の値をそのまま読む」は今も正しく（換算は手順書の流れ 1 の手順 3）、#188 が 30 を触るので本 lane は変えない（手順書が 30 を引く）。closure の是正で、手順 4 の `COUNT` の確かめ（数値のまま残ったセル）を手順 5 の保存した file の確かめに置き換え（数値のまま残ったセルは file の形の外れとして名指される）、手順 4 の「最大」と 0 の行の数を控える作業を外し（後の段で使わない）、Contract Probe の P1 の延期の推論を外した。MSI の手順書（#190・#191 が触る）と 22 §16 の作り直しの手順の「6. 初期投入をやり直す」（開発・試験 DB の作り直しの手順で、店の DB に当てない）は変えない。

## Non-scope

- 設計正本（function-design・db-design・SCREEN_DESIGN・ADR）、`docs/decision-log.md`（新しい決定は無い。予約の D-135 は使わない。手順書が足した判断は手順書に置き、Contract Ledger で引く正本と組にする）。
- `docs/msi-distribution-runbook.md`（流れ 2 の後に本手順書へ進むことは本手順書の「いつ使うか」が書く）。`docs/Plans.md` は関門の 8 の 1 行の他（関門の candidate の行を含む）を変えない。
- code・設定・migration・bindings。
- 予行演習の実施、店の PC での投入。

## Acceptance Criteria

- AC1: `bash scripts/doc-consistency-check.sh` が exit 0（ERROR なし）。起票の前の baseline（2026-10-23、本 branch の起点 `eed07d37`）: exit 0、`結果: WARN 12 件（ERROR なし）`（WARN は既存の `cost_price` 系の DB_DESIGN の未定義）。
- AC2: `bash scripts/doc-consistency-check.sh --target plan docs/archive/plans/2026-10-23-initial-load-procedure.md` が exit 0。
- AC3: `rg -n 'initial-load-runbook.md' docs/PROJECT_HANDOFF.md docs/backlog.md` が両 file で hit する。baseline（`eed07d37`）: `rg exit=1`。
- AC4: `rg -n '^## ' docs/initial-load-runbook.md` が「なぜ 1 度で入れ切るか」「今の事実」「入れる前に決めること」「流れ 1」「流れ 2」「流れ 3」「入れた後に誤りを見つけたら」を出す。baseline: file が無い（`rg exit=2`）。
- AC5: `sed 's/。/。\n/g' docs/initial-load-runbook.md | rg -n '作り直' | rg -v '作り直さない|作り直しで行わない|作り直しの範囲'` が何も出さない（後ろの `rg` の exit 1。文ごとに分けてから除くので、同じ行の別の文に隠れない）と、`rg -c '入れた後のやり直しを DB の作り直しで行わない' docs/initial-load-runbook.md` が `1`。入れた後のやり直しを DB の作り直しで行う文が無く、行わない文があることを見る（Goal の失敗定義の 2 行目）。
- AC6: `rg -c '全部が合ったときだけ業務の入力を始める' docs/initial-load-runbook.md` が `1`（流れ 3 の手順 7）と、`rg -n '業務の入力を始めた後は、取り込む前の backup へ戻さない' docs/initial-load-runbook.md` が 1 行（Goal の失敗定義の 1・3 行目）。
- AC7（読んで確かめる、`docs/initial-load-runbook.md` の「入れる前に決めること」と流れ 2・3）: 手順書の J1〜J4 の各行に選択肢・既定・変わる所があり、J5 の行に選択肢・owner の答え（TD-421）・変わる所があり、「変わる所」が指す手順の番号が実在する。「未確認（P*n*）」が packet の Contract Probe の行を指す。流れ 2 の手順 4 と流れ 3 の手順 3 が、エラー行と重複行が 0 のときだけ取り込む（Goal の失敗定義の 4 行目）。流れ 2 と流れ 3 の両方に、取り込む前の backup から復元・照合までを同じ日に終え、終えられなければ復元してから閉じる規則がある。
- AC8: `git diff --name-only $(git merge-base origin/main HEAD) HEAD` が本 lane の 6 file（手順書・PROJECT_HANDOFF・backlog・Plans・本 packet・Matrix）だけを出す（店のデータの file が無い。main から取り込んだ変更を数えない）。
- AC9: `bash scripts/check-workflow-git.sh` が exit 0。
- AC10: `rg -c '商品 CSV のプレビューで黙って入る形の行を止める' docs/Plans.md docs/backlog.md docs/initial-load-runbook.md` が 3 file で 1 以上、`rg -c 'TD-421' docs/Plans.md docs/backlog.md docs/initial-load-runbook.md` が 3 file で 1 以上（J5 = b が関門・backlog の項目・手順書に届き、関門が backlog の項目を名で引く）。baseline（`a9f0f439`）: 両方とも rg exit=1。

- AC11: `rg -c 'TD-426' docs/Plans.md docs/backlog.md docs/initial-load-runbook.md` が 3 file で 1 以上（関門の 8 の JAN の桁とチェックディジットが関門・backlog の項目・手順書に届く）と、`rg -n 'TEXT\(セル,"0"\)' docs/initial-load-runbook.md` が何も出さない（rg exit 1。先頭の 0 を落とす式が手順書に残らない）。baseline（`ad709ba7`）: 前者は 3 file とも rg exit=1、後者は 83 行目が hit。

実測（2026-10-26、closure 3 の是正の commit の前。base は `5506c7a4`。各 command を逐語で実行した。baseline は各 AC の文に書く。AC8・AC9 と commit の後の値は PR body の Validation に書く）:

```text
AC1  bash scripts/doc-consistency-check.sh → exit=0、結果: WARN 12 件（ERROR なし）（起票の前の baseline と同じ件数）
AC2  bash scripts/doc-consistency-check.sh --target plan <本 packet> → exit=0、結果: 全チェック通過
AC3  docs/backlog.md:67・68 と docs/PROJECT_HANDOFF.md:15 が hit
AC4  8 なぜ 1 度で入れ切るか / 14 今の事実 / 43 入れる前に決めること / 55 流れ 1: 商品 CSV を作る / 125 流れ 2: 予行演習 /
     142 流れ 3: 店の PC で入れる / 158 入れた後に誤りを見つけたら
AC5  出力なし rg exit=1、rg -c → 1
     mutation（$TMPDIR の写しで「作り直しで行わない」→「作り直しで直す」）: 20:入れた後のやり直しを DB の作り直しで直す。（exit=0、red）、rg -c → exit=1（red）
AC6  rg -c → 1、160 行目が hit
     mutation（「全部が合ったときだけ」→「全部が合えば」）: rg -c → exit=1（red）
AC10 商品 CSV のプレビューで黙って入る形の行を止める: Plans 1 / backlog 1 / runbook 1。TD-421: Plans 1 / backlog 2 / runbook 1
AC11 TD-426: Plans 1 / backlog 2 / runbook 7。rg -n 'TEXT\(セル,"0"\)' runbook → 出力なし exit=1
     mutation（最初の `TEXT(セル,"0.00")` を `TEXT(セル,"0")` に替える）: 89 行目が hit（exit=0、red）
```

## Design Readiness

- 引く設計正本（節まで）: 26 §18.3（文字コード・ヘッダ・改行・引用符）、30 §4.8（必須列・行の検証・CSV の中の重複・既存のコードは重複行）・§4.9（1 つの TX、新規の行の初期在庫の記録、`PLU対象` の既定の導出、next_seq を進めない）・BIZ-01-D5（100 文字）・BIZ-01-D6（D-106、CSV の番号を画面の発番が飛ばす）・BIZ-01-D8（単位・`m` の初期在庫・原価の 1/100 円・Excel の税抜の換算は初期投入の作業）、30 の「計数の置き場と在庫の求め直しの契約」（初期在庫の movement、共有 JAN）、60 UI-01c-D5〜D8・D15・D16（プレビュー・重複の既定スキップ・上書きの確認・エラー行の除外・20MB・PLU対象）、master-tables の departments（初期データ 21 部門、独自コードルール C-1、SP-102-04）、10-common-rules の INV-8、ADR SPEC-STK-CNT-D3（初期在庫はどの計数よりも前）・D4（共有 JAN は在庫連動の外）、32 §15 の実データ行のマスタ照合（`jan_code` で引き当てる）、68 §68.4（手動バックアップ・復元と戻す前の自動の backup・PC の外の控え）、71 §71.5（保持日数での削除）・§71.13（別の PC に店のデータを置く扱い）、D-075（取引先を一括で入れない）、D-076（長く動いていないコードの無い品）、D-119（個数の単位どうしの直し）、D-123 (4)・(5)、D-128（原価は税抜）、MSI の手順書の流れ 2 と「予行演習の準備と片付け」。
- 必要な設計成果物: Design artifact selection の当たる行は無い（本 lane は振舞い・command・DB・画面・format を変えない）。管理者の手順書を新しく置く（updated in this PR、`docs/initial-load-runbook.md`）。J5 = b のプレビューの検査の設計（30 §4.8・60 UI-01c）は関門の 8 の runtime の lane が持つ（backlog の項目に触る正本を名指した）。
- plan にしかない durable な判断の昇格先: 無い。手順書が足した判断（行の種類の分け方、CSV の側の全部の行の確かめ、予行演習を開発のデータの folder を退避した空の DB で通す、取り込む前の backup を 2 か所へ写す、取り込む前の backup から照合までを同じ日に行う、エラー行と重複行が 0 のときだけ取り込む、照合が全部合うまで業務の入力を始めない、合わないかその日に終えられなければ取り込む前の backup へ復元する、業務の入力を始めた後は戻さない、直せない誤りは止めて owner に諮る）は手順書に置き、引く正本との組を Contract Ledger に置く。J1〜J4 の既定と J5 の答え（TD-421）は手順書の表に置き、J1〜J4 は owner の答えで直す。
- 前提・制約と延期した gap: 予行演習と投入は初回の導入の版（Plans の導入の前の関門の confirmed を全部満たした版）を待つ。延期しても安全な理由: 手順書は予行演習を通すまで店の PC に入れないと書く（流れ 3 の前提）。黙って入る 2 つの形と、`JANコード` の桁かチェックディジットが合わない行は、J5 の答え b（TD-421）と TD-426 で、初回の導入の版のプレビューが名指しで止める（関門の 8。runtime は別の lane）。手順書の予行演習と投入は関門の confirmed を全部満たした版で行う（流れ 2 の冒頭）ので、手順書がその手作業の確かめを持たなくても黙って入らない（上の Scope の「アプリの側に足りない機能」）。CSV の上書きで JAN が消える筋は、手順書の「入れた後に誤りを見つけたら」が今の `JANコード` を全部の行に入れると書き、名指す warning は同じ runtime の lane の候補にした。商品コードが 13 桁の数字の商品の JAN を消す訂正は関門の 8 の検査と両立しないので、手順書は示さず止めて owner に諮る。
- 絶対保証の自己点検: 「取り込む前の状態に戻る」は、業務の入力を始める前に限り、同じ日に同じ版で作った backup をアプリの復元で戻す場合に限って書いた（68 §68.4 の復元は戻す前に今の状態を自動で backup する。商品と入出庫の記録が 0 件に戻ることは Contract Probe の P2）。「直せない」は商品コード（SP-102-04）と単位の種類をまたぐ変更（30 BIZ-01-D8）に限った。
- 判定: ready（J1〜J4 は既定で手順が成り立ち、答えで変わる所を手順書が名指す〈owner の方針 TD-401〉。J5 は owner の答え b〈TD-421〉で閉じ、TD-426 の JAN の桁とチェックディジットとともに版の条件を Plans の関門の 8 に置いた。Contract Probe の P1 は実機で確かめず、手順の確かめの段〈流れ 1 の手順 5〉で外れを検出する〈TD-437、その節〉）。

## Contract Audit（新しく足す契約の文の境界の一巡）

発注書の境界の 4 観点で、手順書の今の文を一巡した。

| 観点 | 手順書の文 | 確かめたこと |
|---|---|---|
| 安全な整数の範囲 | 今の事実の売価・原価・初期在庫の行、流れ 1 の手順 3 の `税抜 × (100 + 税率) ÷ 100`、手順 4 の `TEXT` の形式と確かめの列 | 上限は 30 BIZ-01-D8 と共通規則の「入力の上限と安全な整数の範囲」の値（売価 999999、原価 999999.99 円、初期在庫 999999999、`m` は cm にした後）を写し、超えればプレビューが行の error にする（`preview_import` の `price_range_error`・`MAX_STOCK_QUANTITY`）。税抜の売価に 100〜110 を掛けた値は約 1 億までで Excel の数（倍精度）で誤差なく表せ、整数を掛けてから 100 で割るので `× 1.1` の形の誤差が出ない。`TEXT` の形式は `0` の数だけ桁を出し、足りない桁は先頭を 0 で埋める（Microsoft の TEXT 関数の説明の例 `=TEXT(1234,"0000000")` → `0001234`。2026-10-24 に support.microsoft.com の同じ頁を見た）ので 8 桁の JAN の先頭の 0 が残り、13 桁の数は有効 15 桁で誤差なく表せる。8 桁の値に 13 桁の形式を当てた `00000` + 8 桁は、EAN-8 の重みが 13 桁の位置 6 から先の重みとそろってチェックディジットが合う（合成の `96385074`・`49123456` で計算した）ので、手順 4 の `00000` の確かめが受け持つ。原価は手順 3 で店の決まり（小数第 3 位で四捨五入して小数第 2 位まで。SPEC-UNIT-D5 の理由、35 SPEC-STK-VAL-D3）で丸めてから `"0.00"` に当てるので、`TEXT` の列・比べの列・照合の相手が 1 通りになる（丸めで変わった行は一覧にして owner が見る）。`"0"`・`"0.00"` の形式は値を丸めるので、置き換える前に `TEXT` の値を元の値と数の値どうしで比べ（`=IFERROR(IF(元のセル="",TEXTのセル<>"",IF(TEXTのセル="",TRUE,--TEXTのセル<>--元のセル)),TRUE)`。両辺に `--` を当てるので `"12.00"` も文字列の `"12.34"` も数に読み、エラー値と数として読めないセルは外側の `IFERROR` で TRUE になって止まる。元のセルが空の行は `TEXT` の列も空のときだけ FALSE）、違う行が 0 であることを数える（形式の選び違いも、列が持てる桁を超える値〈丸めの決まりの無い `m` の初期在庫の小数 3 桁以上、売価・個数の初期在庫の整数でない値〉も同じ列で名指す）。手順 5 の保存した file の確かめは形だけを見て桁の数を限らず（`[0-9]+`）、上限はプレビューの行の error に任せる。`"0.00"` の `12.00` は `parse_hundredths`（小数 1〜2 桁を受ける）が 1200 にし、売価と個数の単位の初期在庫は `parse::<i64>` で読む（小数・桁区切りの値は行の error。どれも `src-tauri/src/biz/product_service.rs` を実読） |
| 0 の表示 | 今の事実の初期在庫「空は 0」、手順 4 の `TEXT` の空の扱い・確かめの列・0 の行の一覧、手順 5 の保存した file の確かめ | 確かめの列の合格は TRUE の数が 0、手順 5 の確かめの合格は外れた行が 0（出力が確かめた行の数の 1 行だけ）で、0 が合格の値。手順 5 は `初期在庫` と `JANコード` の空を通し、`原価` の空は止め（取込みでも行の error）、`売価` の `0` と `原価` の `0.00` は形が合うので通す（0 の行の一覧で owner が見る）。元の値との比べは、元のセルが空の行を `TEXT` の列も空のときだけ FALSE にし、`TEXT` の列に値があれば TRUE で止める（`IF` の無い `TEXT` の式で `0.00` になった行）。`TEXT` は空のセルを 0 として書く（空の原価が `0.00` になり、今は行の error になる空の原価〈`parse_cost_cell` の形式の文〉が黙って原価 0 に替わる。空の JAN が `0000000000000`・`00000000` になり、チェックディジットが合う）ので、空は空のまま残す `IF` の形にした。手順 3 の原価の `ROUND` も空のセルを 0 にするので、同じ `IF` の形にした。初期在庫の空と 0 は取り込んだ後に区別できない（どちらも初期在庫の記録を作らない、30 §4.9）。売価・原価の 0 は受理される（0 以上）ので、owner が予行演習で一覧を見る。`JANコード` の空は TD-426 の検査を通り（JAN を持たない商品）、数字だけの `商品コード` で `JANコード` が空の行は確かめの列で TRUE になる（J4 の a の 8 桁の行も数える） |
| 失敗の時点と順序 | 流れ 1 の手順 4・5、流れ 2 の冒頭の後の段落と手順 4・6、流れ 3 の前提の後の段落と手順 2〜7、入れた後の表の JAN の行 | 原価の丸め（手順 3）→ `TEXT` の列 → 元の値との比べ → 置き換え → コードの確かめの列 → sheet の形 → CSV UTF-8 で保存 → 保存した file の確かめ → SHA-256 の順で、SHA-256 は確かめた後の file の値になり、保存の後に Excel で上書きしない。保存した file の確かめは、先頭が UTF-8 の BOM でなければ行を読まずに止め（保存し直す）、行が出たら取り込まず、全部の列が空の行なら sheet のその行を消して保存から、sheet のセルが違えば列ごとの直し方（`商品コード` は手順 2、負の値と空の原価は元の資料から手順 3、`TEXT` の当て忘れと形式の選び違いは手順 4）から、sheet が正しく file が違えば止める。照合（流れ 2 の手順 6。元の在庫 Excel を開ける PC を手元に置く）は在庫の数を値を取った元のセルの数式バーの値と、原価をその値を店の決まりで丸めた値と比べるので、`TEXT` か保存で丸まった値と合ってしまわない（抜き取り）。プレビューは DB を変えず（30 §4.8）、確定はエラー行を除いて正常な行だけを入れる（30 §4.9、UI-01c-D8）ので、エラー行と重複行が 0 のときだけ取り込む。取り込む前の backup は取込みの前で、2 か所への写しが失敗したら取り込まない。取り込む前の backup から照合（予行演習は復元）までを同じ日に行い、終えられなければ復元してから閉じる（`mnt/backup.rs` の `cleanup_old_backups` の `age.num_days() > i64::from(retention_days)`、実読。作った日のうちは一覧にある）。プレビューの検査は上書きの候補の行にも当たる（`preview_import` は行の検証を重複の分類〈`// 4. 重複チェック`〉の前に行い、error の行は `continue`、実読）ので、13 桁の数字の `商品コード` の商品の JAN を消す行は取り込む前に止まる。それが成り立つのは関門の 8 を満たした版で、流れ 2 の冒頭が「関門の confirmed を全部満たした版」を求める |
| 単位や種類の混在 | 流れ 1 の手順 2 の表と箇条、手順 3・4、今の事実の在庫単位・初期在庫・原価・売価の行 | 行の種類は上から順に当てはめ、1 行が 2 つの種類に入らない（同じ 8 桁の JAN の 2 行は共有の群に入る）。master-tables の布の管理 5 パターンとの対応: 反物・カット生地 → J2 の行、パッチワーク（JAN あり・使ってない）→ 「JAN を使っていない」行（独自コード、JAN を保持、`POS在庫連動` = `0`、`PLU対象` = `0`。D-028 と master-tables の `plu_target` の「JANあり・使ってないグループコード商品」は対象外）、定番ファスナー → 共有 JAN の群、セール品 → 13 桁か J2 の行。保持した JAN は、共有 JAN の不変条件（在庫連動の商品と同じ JAN を持つ別の商品を置かない、30 の冒頭の共有 JAN）に当たらず（使っているかは JAN ごとに決めるので、同じ JAN の行は全部この種類で `POS在庫連動` = `0` になる。他の行が同じ JAN を持てば 13 桁・8 桁の「他の行が持たない」の種類に入らない）、TD-421 の停止（13 桁の数字の `商品コード` で `JANコード` が空）に独自コードなので当たらず、TD-426 の桁とチェックディジットの検査は受ける。`PLU対象` の `0` は `preview_import` が `Some(false)` に読み、空のときの導出（`commit_import` の `should_default_plu_target`、13 桁の数字なら対象）を通らない（実読）。`m` の初期在庫は m の数で `"0.00"`、個数の単位は整数で `"0"`、原価は税抜の円で小数 2 桁まで（1/100 円にしない。アプリが直す）、売価は税込の円の整数、税率は `10`・`8`・`0`。確かめの列は数字だけの `商品コード` に当たり、`-` を含む独自コードは `--` で数にならず FALSE。手順 5 の保存した file の確かめは `在庫単位` が `m` の行の `初期在庫` を小数 2 桁、ほかの行を整数として見る（`-ceq` で大文字と小文字を区別。手順 3 の `在庫単位` は語で入れ、`m` は語と code が同じ）。TD-426 の判定は BIZ-01-D1（`biz::jan_code::validate`、ASCII 数字 8 桁か 13 桁とモジュラス 10）と同じで、先頭 2 の 13 桁と共有 JAN もチェックディジットが合えば通る |

## Registration / Generation Obligations

| 変更対象 | 義務 | 本 lane |
|---|---|---|
| source / workflow doc 新設 | 親文書の目次・索引を更新 | `docs/PROJECT_HANDOFF.md` の案内の表に 1 行（Scope） |

ほかの行（Tauri command、function-design doc、REQ、route、operator 画面）は該当なし。

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | 外部の adapter と app-core の境 | — | 不該当: レジ・POS の adapter に触れない（Z004 の引き当てを今の事実として引くだけ） | なし |
| Fact check / design decision split | 観測した事実とアプリの判断の分け | 手順書の「今の事実」と「入れる前に決めること」 | 当たる: repo の現物で確かめた事実を出所つきで表にし、Excel の振舞いは P1 の未確認に、owner の判断は J1〜J5 に分けた | 予行演習（手順書の流れ 2） |
| Lifecycle / retry | 前・中・後・失敗の後 | 手順書の流れ 2・3 | 当たる: 取り込む前の backup、プレビューのエラー行で取り込まずに直す、照合が合わなければ復元して流れ 1 から、業務の入力の後は戻さない | Test Design Matrix の State Lifecycle Matrix |
| Operator workflow | 実際の順序 | 手順書 | 当たる（管理者の作業）: Excel → CSV → 予行演習 → 店の PC → 照合 → 設定 → 最初の backup。店の利用者の操作にしない | なし |
| Replacement path | 外部の系が変わったとき | — | 不該当: 取込みの形式を変える判断は無い | なし |
| Data safety / evidence | 店のデータを repo に入れない | 手順書の流れ 1 の冒頭、Data Safety | 当たる: 在庫 Excel・CSV・backup は repo の外、記録は管理者の保管場所。予行演習の folder の消す時期は owner | なし |
| Reporting / accounting semantics | 集計の意味 | 手順書の J3 | 当たる: 初期在庫の数を数えた日と入れた日の差、レジが部門キーで売る間の差は、次の計数で棚卸しの差として出る（店主の許容はロスの範囲、project-memory の利用者の節）。評価額の計算は変えない | なし |
| Manual verification | 自動 test で示せない主張 | 手順書の流れ 2 | 当たる: Excel の CSV の保存（P1）は実機で確かめず、流れ 1 の手順 5 の保存した file の確かめで外れを検出する（TD-437）。Windows の画面の上の復元で 0 件に戻ること（P2 は code の上で確かめ済み）は予行演習で確かめる。本 lane の L3 ではない | 予行演習 |
| 環境・再現性 | 新しい環境依存の pin | — | 不該当: 新しい環境依存を足さない（流れ 1 は Excel のある Windows で行い、開発用の PC には Excel が無い〈repo 外の回答台帳 TD-437〉。手順 5 の確かめは Windows の PowerShell） | なし |

## Boundary / Wire Contract

変えない。商品 CSV の形（26 §18.3、30 §4.8）を引いて CSV を作る手順を書くだけで、JSON・CSV の形・config・DTO・bindings・DB に触れない。

## Test Plan

Test Design Matrix: [2026-10-23-initial-load-procedure](test-matrices/2026-10-23-initial-load-procedure.md)（手順の分岐ごとの期待と、それを壊す誤った読み。自動 test は足さない）。

- targeted tests: 無い（code を変えない）。手順が引く振舞いの既存の自動 test は Matrix の Evidence に `rg` で確かめた名で書いた。
- negative tests: AC5・AC6（失敗定義に当たる文が無い・ある）、AC7 の読み取り。
- compatibility checks: AC1・AC2 の doc 検査（R3 以上の Markdown link の実在を含む）。
- data safety checks: AC8。手順書と packet に店の運用の細部・在庫 Excel の中身・金額・商品名を書かない。
- main wiring/integration checks: AC3（案内から辿れる）。

## Review Focus

- Goal の失敗定義の 5 行に当たる記述が手順書に無いか。特に流れ 3 の手順 7（照合が合わないときの復元）が、業務の入力の前に限られ、D-123 (5) の「作り直さない」と食い違わないか（復元は同じ版の backup をアプリの中で戻す操作で、22 §16 の作り直しの手順〈data の folder を移して空の DB から〉ではない、という読みの当否）。
- 今の事実の表の各行が出所（code・正本の節）と合っているか。特に `JANコード` を `商品コード` から補わないこと（`commit_import`）、`PLU対象` の既定の導出、`POS在庫連動` の共有 JAN の拒否が今の版にあること（`preview_import` の `import_row_link_error`。D-118 の (b) の b1）。
- 行の種類の表（流れ 1 の手順 2）が、JAN を使っていない行（布の管理 5 パターン）・共有 JAN の群（TD-352、ADR SPEC-STK-CNT-D4）・8 桁の JAN（J4）・コードの無い行（J2）・どれにも当たらない行を漏れなく 1 通りに分け、どの行も黙って入らないか。
- J5 = b（TD-421）の反映: Plans の導入の前の関門の 8 と backlog の runtime の項目（R の見込み・触る正本・AC の要旨・起票で決める候補）が owner の決定の範囲に合い、AI の調べから出た候補を必須にしていないか。手順書の流れ 1 の手順 4・5 から外した手作業の確かめが、関門の 8 を満たした版のプレビューか今のプレビューの行の error で代わりに止まるか。残した Excel の保存の手順（文字列の値にする・CSV UTF-8）が直せる形か。この変更で不要になる記述・条件・code が他に無いか。
- owner の判断事項（owner の方針 TD-401 のとおり、推奨を既定にして書き進めた。答えで変わる所は手順書の表の「変わる所」の列）:
  - J1 税抜の売価を税込へ換える丸め: a 切り捨て / b 四捨五入 / c 切り上げ / d 値札の税込の値を Excel の別の列に入れて使う。推奨 a（precondition-dependent: 予行演習で値札と照らし、1 件でも違えば全件が合う規則か d へ替える。手順書がこの切替えを持つので、答えが無くても誤った売価のまま入らない）。変わる所: 流れ 1 の手順 3 の `売価`。
  - J2 JAN もコードも無い行: a 部門の接頭辞で独自コードを振って入れる / b 入れずに後で画面から登録する。推奨 a（candidate: 数えた在庫を初日から持てる。画面の発番は CSV の番号を飛ばす〈D-106〉）。変わる所: 流れ 1 の手順 2 の表。
  - J3 初期在庫の数と最初の計数: a 年末の在庫 Excel の数を入れ、最初の計数は次の年末（アプリ）/ b 入れる直前に数え直す / c 初期在庫を空で入れアプリの棚卸しで数える。推奨 a（candidate: 年末の棚卸しは在庫 Excel を埋めれば終わる〈TD-347〉。差は次の計数で棚卸しの差として出る）。変わる所: 流れ 1 の手順 3 の `初期在庫`、流れ 3 の手順 9 の記録。
  - J4 8 桁の JAN の商品の商品コード: a 8 桁を商品コードにする / b 独自コードを振り 8 桁は `JANコード` に入れる。推奨 b（candidate: 商品コードは後から変えられず、`JANコード` は CSV の上書きで直せる）。変わる所: 流れ 1 の手順 2 の表。
  - J5 黙って入る 2 つの形（`商品コード` が 13 桁の数字で `JANコード` が空の行、指数の形の値の行）をどこで止めるか: a 手順書の手作業の確かめで止める / b 商品 CSV のプレビューがその行を名指しで止める形をアプリに入れ、初回の導入の版に含める（導入の前の関門に足す、別の lane）。**答え b**（owner の決定 2026-10-24、repo 外の回答台帳 TD-421）。同じ検査に `JANコード` の桁とチェックディジットを足した（owner の決定 2026-10-24、repo 外の回答台帳 TD-426）。反映: Plans の導入の前の関門の 8、backlog の「商品 CSV のプレビューで黙って入る形の行を止める」、手順書の J5・流れ 1 の手順 4・流れ 2 の手順 4・入れた後の表の JAN の行。
- 流れ 1 の手順 2〜5 と流れ 2 の手順 6 の今の形: 店が JAN を使っていない行が JAN を保持し、`POS在庫連動`・`PLU対象` を `0` にする形（D-028）が、関門の 8 の検査・共有 JAN の不変条件・J4・TD-421 の停止とぶつからないか。手順 4 の 5 つの数字の列の文字列化（`TEXT` の形式、空の扱い）・置き換える前の元の値との比べ（両辺を数にする）・手順 5 の保存した file の確かめが、Excel の表示の形式と保存の振舞いに頼らずに原価と `m` の初期在庫の全部の桁を CSV に書き、形式の選び違い・し忘れ・保存の書き替えを全部の行で名指しで見つけるか。`TEXT` の形式と `00000` の確かめが、8 桁の JAN の先頭の 0 と J4 の両方の答えで成り立つか。原価を手順 3 で店の決まりで丸め、丸めで変わった行を owner が一覧で見る形と、照合が在庫の数を値を取った元のセルの数式バーの値と、原価をその値を同じ決まりで丸めた値と比べる形が 1 通りか。Contract Probe の P1 を実機で確かめず、手順 5 の確かめで外れを検出する形の当否（形を保ったまま違う値を書く外れ方が残る）。商品コードが 13 桁の数字の商品の JAN を消す訂正を示さず止めて owner に諮る形（関門の 8 の検査を上書きの行で緩めない）が、TD-421 の範囲と製品の振舞いを変えずに両立するか。この変更で不要になる記述・条件が他に無いか。
- Risk を R4 にした理由（店の DB への backup の復元の判断と、本物の商品 CSV を開発用の Windows に置く手順。Risk Tiers の R4 の行）の当否。

## Contract Ledger

手順書が足した判断（「実装（Scope）」の列の手順書の箇所）と、それが引く正本の契約の組。手順書は正本を変えない（Non-scope）。触る正本の節の隣接の契約は sweep して、Scope が触り得るものを足すか除外を書いた（最後の 3 行）。Matrix の C*n* は Test Design Matrix の Contracts Under Test。

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| IO-03 の文字コード・ヘッダ・改行の扱い | 26 §18.3、`decode_bytes` | 今の事実の文字コード・列の行、流れ 1 の手順 5（CSV UTF-8〈BOM 付き〉で保存、Excel で上書きしない、保存した file の先頭の BOM を `decode_bytes` と同じ条件で確かめ、中身を全部の行で確かめる）、流れ 2 の手順 4（file の error なら手順 5 から）（Matrix C6） | 既存（`src-tauri/src/io/product_csv_importer.rs` の test。Matrix の Evidence） | 流れ 1 の手順 5 で P1 の外れを検出する |
| BIZ-01 の preview の必須列・行の検証・CSV の中の重複・既存のコード | 30 §4.8 の手順 3〜5、BIZ-01-D5 | 今の事実の列・商品コードの行、流れ 2 の手順 4 と流れ 3 の手順 3（エラー行と重複行が 0 のときだけ取り込む）（Matrix C3） | 既存（Matrix の Evidence） | 非対象（読み取り） |
| BIZ-01 の commit（1 つの TX、エラー行の除外、新規の行の初期在庫の記録、`PLU対象` の既定、`JANコード` を補わない、next_seq を進めない） | 30 §4.9、UI-01c-D8、30 の計数の置き場の契約 | 今の事実の確定・初期在庫の記録・JANコード・PLU対象の行、流れ 1 の手順 2 の表の `JANコード`・`PLU対象` の列（Matrix C2・C3・C4） | 既存（Matrix の Evidence） | 非対象 |
| BIZ-01-D8（単位・`m` の初期在庫・原価の 1/100 円・Excel の換算は初期投入の作業） | 30 BIZ-01-D8、共通規則 SPEC-UNIT-D5・D10、D-128、`parse_cost_cell`・`parse_hundredths` | 今の事実の売価・原価・在庫単位・初期在庫・Excel の CSV の保存の行、流れ 1 の手順 3、手順 3 の原価の丸め（店の決まり）、手順 4（数字の列を全部の桁の文字列にし、`TEXT` の値が元の値と違う行で止める。丸めで原価が変わった行を一覧にする）、流れ 2 の手順 6（在庫の数を値を取った元のセルの値と、原価をその値を店の決まりで丸めた値と照合する）（Matrix C6・C7） | 既存（Matrix の Evidence） | 非対象 |
| 独自コードの形と画面の発番（BIZ-01-D6、D-106）、接頭辞の無い部門 | master-tables の独自コードルール、30 BIZ-01-D6 | 流れ 1 の手順 2 の箇条（部門の接頭辞と 4 桁の番号、接頭辞が無ければ止める）（Matrix C4） | 既存（Matrix の Evidence） | 非対象 |
| 商品コードを変えない（SP-102-04）・商品を消さない（INV-8） | master-tables の「独自コードの接頭辞変更」、10-common-rules の INV-8 | なぜ 1 度で入れ切るか、入れた後に誤りを見つけたらの表（Matrix C5） | 非対象（読み取り） | 非対象 |
| 共有 JAN は在庫連動の外（SPEC-STK-CNT-D4） | ADR、30 の計数の置き場の契約、60 の同じ節 | 流れ 1 の手順 2 の表の共有 JAN の群の行（`POS在庫連動` = `0`）、今の事実の POS在庫連動の行（Matrix C4） | 非対象（D-118 の (b) の test。本 lane の範囲の外） | 非対象 |
| 初期在庫はどの計数よりも前（SPEC-STK-CNT-D3） | ADR の判定表の「商品の登録・一括 import の初期在庫」の行 | 今の事実の初期在庫の記録の行、J3（Matrix C8） | 既存（(a) の test。Matrix の Evidence） | 非対象 |
| Z004 は `jan_code` で商品に引き当てる | 32 §15 の実データ行のマスタ照合 | 今の事実の JANコード の行、流れ 1 の手順 2 の表の `JANコード` の列（Matrix C2） | 既存（Matrix の Evidence） | 非対象 |
| 手動バックアップ・復元と戻す前の自動の backup | 68 §68.4、71 §71.7 | 流れ 2 の手順 3・7、流れ 3 の手順 2・7（Matrix C1） | 既存（Matrix の Evidence）、Contract Probe P2（一時 test） | 画面の上は予行演習の手順 7 |
| 保持日数での削除 | 71 §71.5、`schema_v1.rs` の `backup_retention_days` | 流れ 3 の手順 2・9（保存先の外の 2 か所へ写す）（Matrix C1） | 非対象（読み取り） | 非対象 |
| リリースの後は作り直さない、初回の導入の版の条件 | D-123 (4)・(5) | なぜ 1 度で入れ切るか、流れ 1 の手順 2 と流れ 2 の手順 2（部門の初期データはリリースの前に直すのが安い。リリースの後は値を移す migration が要る）、流れ 2 の冒頭（初回の導入の版は Plans の導入の前の関門の confirmed を全部満たした版）（Matrix C1・C4・C9） | AC5 | 非対象 |
| 手順書が足した判断: エラー行と重複行が 0 のときだけ取り込む。取り込む前の backup から照合までを同じ日に行う（予行演習は復元まで）。照合が全部合ったときだけ業務の入力を始める。合わないかその日に終えられなければ取り込む前の backup へ復元し、合わなければ流れ 1 から。業務の入力を始めた後は戻さない | 引く正本: 30 §4.9（エラー行を除いて入れる）、68 §68.4（復元）、71 §71.5 と `cleanup_old_backups`（file 名の日付から数えた日数が保持日数を超えたら消す）、D-123 (5) | 流れ 2 の冒頭の後の段落と手順 4、流れ 3 の前提の後の段落と手順 3・7、入れた後に誤りを見つけたら（Matrix C1・C3） | AC6、Contract Probe P2 | 非対象（読み取り） |
| 手順書が足した判断: 予行演習を同じ版・同じ CSV で開発用の Windows で、開発のデータの folder を退避した空の DB に通す。同じ日に MSI の試験用のデータを使うときはその片付けの後に始め、商品が 1 件でもあれば止める。片付けは MSI の手順書の「予行演習の準備と片付け」、店のデータの folder の消す時期は owner | 引く正本: MSI の手順書の確かめる項目と準備と片付け、71 §71.13 | 流れ 2（Matrix C9） | 非対象 | 予行演習で 1 度通す |
| 手順書が足した判断: 行の種類（上から順に当てはめる排他の種類。部門 11・12・JAN を使っていない・共有 JAN の群・共有しない 13 桁 JAN・共有しない 8 桁 JAN・長く動いていないコードの無い品・コード無し・どれにも当たらない）。JAN を使っていない行は JAN を保持し、`POS在庫連動`・`PLU対象` を `0` にする。どれにも当たらない行・接頭辞の無い部門の独自コードの行（1 行でもあれば手順を止める）・`TEXT` の値が元の値と違う行・数字だけの `商品コード` が `JANコード` と同じでない行・`JANコード` が 13 桁で `00000` から始まる行で止める（表の全部の行を数える）。5 つの数字の列を全部の桁の文字列の値にし、sheet の形を整えて CSV UTF-8 で保存し、保存した file の 5 つの列が手順 4 で決めた形でない行で止める（全部の行） | 引く正本: 30 §4.8 の手順 4（商品コードの形を検査しない）、master-tables の departments（部門 11・12 は在庫管理の対象外）・布の管理 5 パターン・`plu_target`、D-028（JAN を空にしない）、D-076、30 の冒頭の共有 JAN の不変条件、26 §18.3（BOM の UTF-8）と `parse_product_csv`（1 行目が列名、同じ名の列は後ろの値）、TD-352、J2・J4 | 流れ 1 の手順 2・4・5（Matrix C2・C4・C6） | 非対象（読み取り） | 非対象 |
| J5 = b（TD-421）と TD-426: 指数の形の値の行、13 桁の数字の `商品コード` で `JANコード` が空の行、`JANコード` が 8 桁・13 桁の数字でないかチェックディジットが合わない行は、初回の導入の版のプレビューが名指しで止める（上書きの候補の行も） | Plans の導入の前の関門の 8、backlog の「商品 CSV のプレビューで黙って入る形の行を止める」（設計と実装は runtime の lane。30 §4.8・BIZ-01-D1・60 UI-01c） | 今の事実の黙って入る形の行、J5、流れ 1 の手順 4 の注、流れ 2 の手順 4、入れた後の表の JAN の行（Matrix C2・C6・C10） | runtime の lane が足す | 予行演習の手順 4 |
| 上書きは `JANコード` を CSV の値で書く（空・列なしは JAN を消す。JAN を消す道はこれだけ） | 30 BIZ-01-D8 の上書きの段の `plu_dirty` の項、30 §4.9.2 の経路と「JAN を空にする変更」、`commit_import` | 今の事実の上書きの JANコード の行、入れた後に誤りを見つけたらの 2 行（Matrix C10） | 無い（今の振舞いを直接確かめる test は見つからない。Matrix の Residual Test Gaps） | 非対象 |
| 隣接の除外: 取引先を一括で入れない（D-075）、長く動いていない品を商品にしない（D-076） | D-075・D-076 | 今の事実の取引先の行、J2 の既定、流れ 1 の手順 2 の表の長く動いていない品の行（入れない） | 非対象 | 非対象 |
| 隣接の除外: JAN の別名（D-122、proposed・未実装）、PLU の書出し（33）、SD 直読み（D-111） | 30 §4.9.2、33、29 | 変えない（初期投入は新しい DB で重複行が無く、別名の問いは出ない。PLU をレジへ書く手順は導入の前の関門の precondition-dependent の項目） | 非対象 | 非対象 |
| 隣接の除外: 22 §16 の作り直しの手順の「6. 初期投入をやり直す」 | 22 §16 | 変えない（開発・試験 DB の作り直しの手順で、店の DB に当てない） | 非対象 | 非対象 |

## Contract Probe

R4 の Contract Probe（`docs/DEV_WORKFLOW.md` Plan Packet Rules）は、外部の前提に頼る計画で Plan Gate の前に最小の実験を求める。

- P1 Excel の「CSV UTF-8（コンマ区切り）」の保存が BOM 付きの UTF-8 で、手順 4 で全部の桁の文字列の値にした 5 つの数字の列（`商品コード`・`JANコード`・`売価`・`原価`・`初期在庫`）をセルの中の文字のまま書く → **実機で確かめない。手順の確かめの段で外れを検出する**。理由: 手順書の安全を Excel の保存の振舞いの確かめに置かず、保存した file を流れ 1 の手順 5 で確かめる（repo 外の回答台帳 TD-437 の後の裁定）。手順書は Excel の保存の振舞いを前提にせず、流れ 1 の手順 5 で保存した file の中身そのものを取込みの前に確かめる: PowerShell の `Import-Csv` で全部の行を文字列のまま読み、5 つの列が手順 4 で決めた形（`商品コード` は 13 桁か 8 桁の数字か独自コード、`JANコード` は空か 13 桁か 8 桁の数字、`売価` は整数、`原価` は小数 2 桁、`初期在庫` は `m` の行で空か小数 2 桁、ほかの行で空か整数）でない行を行の番号・列・値で出し、確かめた行の数を手順 2 の和と比べる。手順 4 は 5 つの列を形の決まった文字列にするので、保存が値を変える外れ方は形の外れになる: 表示の桁で丸まる（原価 `12.34` → `12`、`m` の `1.50` → `1.5`）→ 小数 2 桁でない。先頭の 0 が落ちる・指数の形になる → 8 桁・13 桁の数字でない（関門の 8 の版のプレビューも止める）。桁区切りが入る → 数字だけでない。BOM の無い file（「CSV（コンマ区切り）」の CP932、BOM の無い UTF-8、UTF-16）→ 手順 5 の block が `Import-Csv` の前に先頭の 3 byte が `EF BB BF` かを見て、違えば保存し直しを名指して止まり、行を読まない。この確かめはアプリの `decode_bytes` の分岐の条件（先頭の 3 byte が BOM なら UTF-8、それ以外は CP932）と byte 単位で同じで、BOM があれば PowerShell の `-Encoding UTF8` とアプリが同じ読み方になることを保証する。BOM の無い file はアプリが必ず CP932 で読み、手順書は CP932 を使わない（Matrix C6）ので、アプリより厳しいのは意図どおり。BOM 付きで中身が UTF-8 として不正な file は、流れ 2 の手順 4 の取込みの file の error（「Excelで保存し直してください」と直し方を名指す）で止まる。行が出たら取り込まず、全部の列が空の行（sheet の下に残った空の行）なら sheet のその行を消して保存からやり直し、sheet のセルが手順 4 の形でなければ列ごとの直し方（`商品コード` は手順 2、負の値と空の原価は元の資料から手順 3、`TEXT` の当て忘れと形式の選び違いは手順 4）からやり直し、sheet が正しいのに file が違えば止めて保存の段を直す lane を起票する（手順書の手順 5）。値の中の改行は、プレビューの行の error で名指される。取り込む前に止まる。形を保ったまま違う値を書く外れ方はこの段で拾えず、照合の抜き取り（流れ 2 の手順 6）でしか拾わない（Matrix の Residual Test Gaps）。手順書の安全はこの段で P1 の外れを取り込む前に止めることに置き、P1 が成り立つことに置かない。CP932 の `?` への置換は「CSV（コンマ区切り）」の保存でだけ起き、手順 5 が使わず、その file は手順 5 の BOM の確かめで止まる。
- P2 取り込む前の backup をアプリの復元で戻すと商品の無い状態に戻る → code の上で確かめた（2026-10-24、round 1 の是正、head `e9493279`）。一時 test を `src-tauri/src/mnt/backup.rs` の test module に置き（実行後に `git checkout` で外し、`git status --porcelain` は空）、`CARGO_TARGET_DIR=/tmp/shared-cargo-target cargo test --lib probe_pr196_p2 -- --nocapture` で回した。手順: `db::init_database` の空の DB → `create_backup`（取り込む前の backup）→ BOM の UTF-8 の 4 行の商品 CSV（13 桁の JAN を両方の列に持つ行、13 桁の数字の商品コードで `JANコード` が空の行、`4.97638E+12` の行、`m` の独自コードの行）を `preview_import` → `commit_import` → `restore_backup` → 開き直し。出力:

  ```text
  PROBE preview valid=4 error=0 dup=0
  PROBE after import created=4 products=4 movements=4 jan_null_for_13digit=1
  PROBE after restore products=0 movements=0 departments=21 app_settings=9
  PROBE after reopen products=0
  test result: ok. 1 passed; 0 failed
  ```

  復元で商品と入出庫の記録が 0 件に戻り、部門と設定の行の数は取り込む前と同じ。同じ probe で、黙って入る 2 つの形（`JANコード` が空の 13 桁の行、指数の形の行）が preview で正常な行になることも確かめた（手順書の「今の事実」の「黙って入る形」の行）。Windows の画面の上の復元（「バックアップ・復元」の一覧から選ぶ）は予行演習の手順 7 で確かめる。
- P3 流れ 1 の手順 5 の PowerShell の block が Windows PowerShell 5.1 で外れた行を名指して止め、正しい行を通す → 是正の前の block で観測した（2026-10-26、closure 3 の Codex〈発注 584〉の最終報告。Windows PowerShell 5.1.26100.9711、店のデータを含まない合成の CSV）。観測の要旨（数は同じ報告の値）: 合成の 65 行の末尾 5 行に列ごとの外れを入れると 62〜66 行目の各列を名指し、確かめた行の数は 65。正しい行（独自コード、先頭が 0 の 8 桁、`m` の `1.50`、空の JAN・初期在庫、売価 `0`・原価 `0.00`）は通った。原価 `12`、`m` の `1.5`、先頭の 0 の落ち、指数の形、桁区切り、全角の数字、小文字の接頭辞、空の原価は名指して止まった。CP932 の file は `Import-Csv` が列名の重複（「メンバーが既に存在」）の読取りの error を出し、確かめた行の数は 0。区切りだけの空の record は名指され、行の数が増えた（ただの空の行は読み飛ばされた）。値の中の改行は PowerShell が 1 つの record として通し、アプリの `parse_product_csv` は 2 件の行の error にした。BOM の無い UTF-8 の 1 行の file は「確かめた行の数: 1」だけを出して通り、同じ byte 列をアプリに渡すと「ファイルの文字コードが判別できません。Excelで保存し直してください」の file の error になった（UTF-16 も同じ差）。この最後の観測が closure 3 の B の反例で、是正で `Import-Csv` の前に先頭の BOM の確かめ（`[System.IO.File]::ReadAllBytes`。PowerShell 5.1 と 7 の両方にある .NET の呼び方）を足した。是正の後の block は、closure 4 の Codex（発注 591 の最終報告）が 2026-10-26 に同じ Windows PowerShell 5.1.26100.9711 で、手順書から逐語で写して合成の 5 つの file で回した（数は同じ報告の値）: BOM 付きの UTF-8 は「確かめた行の数: 1」の 1 行だけ（`Import-Csv` の呼出し 1 回）、BOM の無い UTF-8・CP932・UTF-16 LE（BOM 付き）・空の file は保存し直しを名指す停止の 1 行だけ（呼出し 0 回）で、行ごとに入力する形（`-Command -`）でも同じだった。同じ byte 列をアプリの `parse_product_csv` に渡すと、BOM 付きは成功、BOM の無い UTF-8 と UTF-16 は文字コードの error、空の file は「ファイルが空です」、CP932 は成功（手順 5 がアプリより厳しく止めるのは C6 の意図どおり）。照合の `Out-GridView` は 5.1 で画面が出た（画面の上の操作はしていない）。PowerShell 7 では回していない。

## Data Safety

- commit しない: 在庫 Excel・その写し・商品 CSV・backup・管理者の記録・店の DB。
- repo の外にだけ置く: 上の全部（流れ 1 の冒頭、管理者の保管場所）。
- 試験用だけ: 本 lane は試験用のデータを作らない。予行演習は在庫 Excel から作った本物の CSV を開発用の Windows で使い、出来た folder の消す時期は owner が決める（71 §71.13 と同じ）。

## Implementation Results

本 lane は docs だけで、code・migration・wire の型・設計正本は変えていない。店の PC で本物のデータを入れ始める初期投入を、管理者の手順書 `docs/initial-load-runbook.md` に 1 つに並べた: 在庫 Excel から商品 CSV を作る流れ（行の種類の分け方、桁を保つ `TEXT` の列と確かめの列、保存した file の形の確かめ）、開発用の Windows での予行演習、店の PC での投入（取り込む前の backup、取込み、照合、合わないときの戻し方）、入れた後に誤りを見つけたときの直し方。owner の判断事項は J1〜J5 に既定と答えで変わる所を書き、J5 = b（TD-421）と TD-426 で、商品 CSV のプレビューが黙って入る形の行を止める検査を導入の前の関門の 8 として `docs/Plans.md` に足し、中身を backlog の「商品 CSV のプレビューで黙って入る形の行を止める」に置いた（runtime は PR #209 の lane）。`docs/PROJECT_HANDOFF.md` に手順書の案内を足し、backlog の「初期投入の手順」の項目を手順書への案内と残る作業に置き換えた。予行演習と店の PC での投入は初回の導入の版の後に残る。

## Review Response

先行 round の結果・評価（判定・件数・採否・reviewer の意見）はこの節にだけ書く。前半の節と遷移記録には「round N の是正（Review Response 参照）」だけを書く（独立 review は `## Review Response` より前だけを読む）。

round 1（`e9493279`）: Codex（発注 549）= reject（P1 0 / P2 8 / P3 2。Ordinary Operation に具体的な反例あり）、Claude 側 fresh Opus 5.5 = reject（P1 0 / P2 2 / P3 9）。PR #196 に投稿された review は 0 本（`gh api repos/kosei-w90607/inventory-system-desktop/pulls/196/reviews` が空）で、Codex の件数は最終報告の 10 件。重なる finding を束ねて 12 件にし、全部を受理した。一部を退けたのは 2 点（下の #2 の Design Readiness、#5 の P1）。是正は起草役の `a9f0f439`。

| # | finding（束ねた元） | 裁定 | 是正 |
|---|---|---|---|
| 1 | P2 照合を保持日数を越えて延ばすと、取り込む前の backup が一覧から消えて戻せない（Codex #1） | 受理（別の形で是正） | 保全した写しから戻す分岐を足さず、取り込む前の backup から照合までを同じ日に行い、その日に終えられなければ業務の入力の前に復元してから閉じる段落に置き換えた（手順書の流れ 3 の前提の後。保存先の中の backup は file 名の日付から数えた日数が保持日数を超えた日に消える〈`cleanup_old_backups`〉ので、同じ日なら一覧にある。写しから戻す入口〈68 の「控えを選んで確かめる」〉は D-114 の runtime の版にしか無い）。今の事実の backup の保持の行、Matrix の C1・FM3・C1 の読み取りの行、Contract Ledger、Contract Audit の追記 |
| 2 | P2 黙って入る 2 つの形を owner の手の確かめに回す判断が D-127 (2) と両立せず、手順 4・5 の確かめが抜き取りで file 全体を見ない（Codex #2、Opus #1・#2）。P3 preview の warning に時期の条件が無い（Opus #6） | 受理。ただし Design Readiness を ready のままにする点は退ける | 「アプリの変更は要らない」を撤回し、D-127 (2) との食い違いを手順書の J5（a 手順書の機械の確かめ / b プレビューが名指しで止める形を初回の導入の版に入れる。推奨 b、既定 a）として owner に諮る形にした（手順書の J5、packet の Scope の「アプリの側に足りない機能」・Goal・Design Readiness・Review Focus、backlog の後続の候補に時期の条件）。手順 4 を表の全部の行を数える確かめの列（13 桁の数字の `商品コード` で `JANコード` が違う・空の行が 0）に、手順 5 を保存した file の全体の `E+` の検索と行の数に替えた。今の事実に「黙って入る形」の行を足した。退ける理由: b はアプリの変更で、AI の調べから出た要件は owner の確認の前に必須にしない（AGENTS.md の Working Rules の「owner の所感は原文で扱う」）。owner の方針（repo 外の回答台帳 TD-401、2026-10-23）は推奨を既定にして書き進めることで、既定 a でも手順書の確かめが表と file の全部の行を取り込む前に見るので黙って入らない。J5 の答えは owner の判断として Coordinator に渡す |
| 3 | P2 共有 JAN と 8 桁の JAN の種類が排他でない（Codex #3）。P3 布の管理 5 パターン（パッチワーク）と部門 11・12 の扱いが無い（Opus #8） | 受理 | 手順 2 の表を上から順に当てはめる排他の種類にし、部門 11・12 の行（入れない）を先頭に、共有 JAN の群を桁を問わず 13 桁・8 桁の前に置き、13 桁・8 桁を「他の行が持たない」に限り、J2 の種類に店が JAN を使っていない行（布の管理 5 パターンの独自コードの行）を含めた。今の事実の部門の行、Matrix の C4・FM6・C4 の読み取りの行、Contract Ledger |
| 4 | P2 MSI の予行演習と同じ回にするときの試験用のデータとの境が無い（Codex #4）。P3 M1〜M8 は済んでいる（Opus #7） | 受理 | 流れ 2 の冒頭を、M1〜M8 は 2026-10-24 に済み、同じ日に MSI の試験用のデータを使うときはその片付けの後に手順 1 から始める形に置き換え、手順 2 に商品があれば止める段を足した。packet の Risk の L3 の段落・Owner Effort Budget の注・Ordinary Operation・Contract Ledger、Matrix の C9・FM12・C9 の読み取りの行 |
| 5 | P2 未確認と書いて止めることを Plan Gate の前の Contract Probe の代わりにしている（Codex #5）。P3 P2 は repo の中で probe できる（Opus #9） | P2 は受理。P1 を Plan Gate の前に回す点は退ける | P2 を一時 test で回して command と出力を記録した（Contract Probe の P2。4 商品・4 movement が復元で 0・0、部門 21・設定 9 は同数、開き直しても 0）。退ける理由: P1 は Excel が要り、本 run の環境に Excel が無い（Contract Probe の P1 に調べた path）。代わりに手順を P1 に安全を頼らない形にした（#2 の機械の確かめで、P1 が外れても取り込む前に止まる）。P1 の最小の実験（Excel のある Windows で合成の 3 行）は owner の手が要り、予行演習の手順 4 の前に行う |
| 6 | P2 R4 にしない理由が Risk Tiers に無い例外を作っている（Codex #6。Opus は R3 を妥当とした） | 受理 | Risk Tiers の R4 の行（backup restore、real store data exposure）に当たるので R4 にした。Final Review Minimum 2、Human Gate `ready,merge,r4`、Rollback / recovery notes を足した（packet の Workflow State・Risk・Owner Effort Budget の見込みに R4 の承認 1、Matrix の Risk）。先例は docs だけの R4 の `docs/archive/plans/2026-10-07-backup-offsite.md` |
| 7 | P2 初回の導入の版の条件が「関門 1〜4」のまま（Codex #7） | 受理 | 「Plans の導入の前の関門の confirmed を全部満たした版」への参照に置き換えた（手順書の流れ 2 の冒頭、packet の Risk の L3 の段落・Design Readiness・Contract Ledger、Matrix の C9・Compatibility Checks） |
| 8 | P2 AC5 の mutation が落ちない（Codex #8、Opus #3） | 受理 | AC5 を文ごとに分けてから除く形と、「入れた後のやり直しを DB の作り直しで行わない」の数え上げ `1` の 2 つにした。`$TMPDIR` の写しの mutation で red を確かめた（AC の実測、Matrix の Mutation-style Adequacy Questions） |
| 9 | P3 AC8 の起点が main の同期の前のまま（Codex #9、Opus #4） | 受理 | `git merge-base origin/main HEAD` を起点にした（実測 `843c4938`、5 file） |
| 10 | P3 backlog の項目に「設計が無い」「決めること」が残る（Codex #10、Opus #5） | 受理 | 項目を手順書への案内・業務の入力の前後の戻し方・残る作業に置き換え、起票の行を本文に入れた。packet の「この変更で不要になる記述」を「無い」から直した |
| 11 | P3 J1 の換算を `× 1.1` で行うと切り上げで 1 円ずれ、照合が端数の出る値を選ばない（Opus #10） | 受理 | 手順 3 を `税抜 × (100 + 税率) ÷ 100`（整数を掛けてから割る）と式の例に、流れ 2 の手順 6 に端数の出る売価の商品を必ず含める句を足した。Contract Audit の追記 |
| 12 | P3 「部門の初期データはリリースの前にしか変えられない」は D-123 (5) より強い（Opus #11） | 受理 | 「リリースの前に直すのが安い（リリースの後は値のある DB を移す migration が要る）」に弱めた（手順書の流れ 1 の手順 2、Matrix の C4、Contract Ledger） |

是正の中で足したもの（finding の付随）: Opus #1 の影響の欄の「CSV の上書きで JAN を入れても `PLU対象` が空なら今の 0 が保たれる」を、手順書の今の事実の PLU対象の行（`test_commit_import_req907_blank_plu_target_derives_for_new_and_preserves_overwrite`）と「入れた後に誤りを見つけたら」の JAN の行（13 桁の JAN を入れ直すときは `PLU対象` を `1`）に書いた。

round 2（`a9f0f439`）: Claude 側 fresh Opus 5.5 = reject（P1 0 / P2 2 / P3 4。Ordinary Operation に具体的な反例 2 つ）。round 1 の 12 件の closure は、#3 が「届いていない（一部）」（下の F1）で他の 11 件は届いた。Codex の round 2（発注 553）は J5 の反映を待たせたので、`ad709ba7` の head で走らせた。あわせて owner の決定 J5 = b（repo 外の回答台帳 TD-421、2026-10-24）を反映した。是正は起草役の `ad709ba7`。

| # | finding | 裁定 | 是正 |
|---|---|---|---|
| F1 | P2 「店がその JAN を使っていない」が JAN で分ける行より下にあり、Excel に JAN を持つパッチワークの行が 13 桁の JAN の行に先に当たって商品コード = JAN になる | 受理 | 「在庫 Excel に JAN があるが、店がその JAN を使っていない」を部門 11・12 の次の独立した行にし、JAN で分ける 3 行より上に置いた。J2 の行は「JAN もコードも無い」に戻した。master-tables の布の管理 5 パターンが 1 通りに当たることを Contract Audit の round 2 の表で確かめた（反物・カット生地 → J2、パッチワーク → JAN を使っていない、定番ファスナー → 共有 JAN の群、セール品 → 13 桁か J2）。Matrix の C4 と Would fail if、Contract Ledger、Review Focus |
| F2 | P2 CSV の上書きで `JANコード` が空・列なしだと JAN が NULL になることが手順書に無い | 受理。正本を実読し、意図した契約（(ii)）と判断した | 根拠: 30 BIZ-01-D8 の上書きの段の `plu_dirty` の項が、上書きで書く列を「§4.9 の手順 2 a. の UPDATE が書く JAN・商品名・部門・売価・原価・税率」と「CSV が値を持つときだけ書く取引先・メーカー品番・在庫連動・PLU対象」に分けて JAN を前者に置く。30 §4.9.2 は「JAN を空にする変更（`A` → なし）も『直す』に入る」「今 JAN を変える経路は商品 CSV の上書き（§4.9）だけ（商品フォームの edit は JAN を読取専用にする）」と書き、空を「今の値を保つ」にすると JAN を消す道が無くなる。`commit_import` も `jan_code: Some(row.jan_code.clone())` で正本どおり（`test_commit_import_req104_overwrite_preserves_optional_fields` の comment「任意列未指定は既存値保持」は在庫単位だけを確かめ、JAN には当たらない）。是正: 手順書の今の事実に「上書きの JANコード」の行、入れた後の表の商品名・売価などの行に「上書きの CSV には今の `JANコード` を全部の行に入れる」、JAN の行に「消すときは空にする」を足した。JAN を消す行を件数と行の番号で名指す warning を、関門の 6 の runtime の項目の起票で決める候補 (4) に足した（AI の調べから出た候補で必須にしない）。Matrix の C10・FM13・Residual Test Gaps、Contract Ledger、Ordinary Operation の 4 行目 |
| F3 | P3 手順 4・5 の直し方と確かめ方が直せない止まり方をする（数値のセルは書式を変えても数値、メモ帳の検索が大文字と小文字を区別しない、末尾の改行で行の数が合わない） | 受理（J5 = b の後に残る部分に当てた） | メモ帳の `E+` の検索と行の数は J5 = b で外した（関門の 6 の版のプレビューが止める。行の数はプレビューの新規登録候補の数〈流れ 2 の手順 4〉で見る）。残る Excel の保存の手順を、`=TEXT(セル,"0")` の列で置き換える（または表示形式 `0`）形にし、セルが既に指数の形の文字なら `TEXT` で戻さず元の資料から入れ直すと書いた。P1 の最小の実験に在庫 Excel から写した数値のセルを含めた（Contract Probe の P1） |
| F4 | P3 「CSV（コンマ区切り）」は CP932 で、CP932 に無い文字を `?` に替えて黙って入る | 受理 | 保存を「CSV UTF-8（コンマ区切り）」にした（取込みは先頭の BOM で UTF-8 として読む。26 §18.3、`decode_bytes` の `EF BB BF` の判定を実読）。今の事実の Excel の CSV の保存の行、Contract Probe の P1（「P1 が外れたときに変わるのは…だけ」を、プレビューで止まらない筋 2 つとその避け方に書き直した）、Matrix の C6・FM9、Contract Ledger の IO-03 の行 |
| F5 | P3 接頭辞が無いときの「止める」が 2 通りに読め、J4 の変わる所に 8 桁の扱いが無い | 受理 | 流れ 1 の手順 2 の箇条を「接頭辞が無い行が 1 行でもあれば手順を止める（その行を外して進まない）」にした。J4 の変わる所に、b では接頭辞の無い部門（初期データで `code_prefix` が NULL の部門）の 8 桁の JAN の行が全部止まること（数は手順 2 で控える）と、a では 8 桁の行を商品コード = JAN にして手順 4 の確かめを 8 桁にも当てることを足した。関門の 6 の検査を 8 桁に当てるかは、backlog の項目の起票で決める候補 (1)（推す） |
| F6 | P3 後続の候補の効く範囲が狭く書かれている | 受理 | J5 = b で関門の runtime の項目になったので、backlog の項目に「初期投入だけでなく、導入の後に CSV で商品を足すたびにも効く」と書いた |

J5 = b（TD-421）の反映（Coordinator の裁定 2026-10-24）:

- 手順書: J5 の行を「答え b（TD-421）」で閉じ、変わる所を関門の 6・流れ 1 の手順 4・5・流れ 2 の手順 4 にした。表の前文を「J1〜J4 は既定、J5 は owner の答え」にした。今の事実の黙って入る形の行の最後の文を「初回の導入の版ではプレビューが名指しで止める」にした。流れ 1 の手順 4 から 13 桁の数字で `JANコード` が空の行の数え・`商品コード` の重複・上限の確かめを、手順 5 からメモ帳の `E+` の検索と行の数を外し（D-127「使い手に押しつけない」、AGENTS.md の「継ぎ足さない」）、流れ 2 の手順 4 を「プレビューが名指した行と直し方のとおりに直す」にした。残したのは、プレビューが止めない形の確かめ（13 桁の数字の `商品コード` で `JANコード` が空でなく違う行。backlog の候補 (2) が入れば外す）と、Excel の保存の手順（F3・F4）
- `docs/Plans.md`: 導入の前の関門の confirmed に 6 を足した（既存の 5 と同じく owner の決定・台帳の ID・出所・backlog の項目の名を書く）
- `docs/backlog.md`: 「初期投入の手順」の下の後続の候補の行を、関門の runtime の項目「商品 CSV のプレビューで黙って入る形の行を止める」（R3 見込み、触る正本〈30 §4.8 の手順 4・§4.9 の手順 0・BIZ-01 の新しい Decision、60 UI-01c-D5・D8〉、AC の要旨、起票で決める候補 (1)〜(4)）に置き換えた
- packet: 規模と手法、遷移の記録 6、Owner Effort Budget（介入 1 を消費）、Goal、非目的、Ordinary Operation、Scope（Plans を Scope に入れ、Non-scope から外した。所有の表に Plans と backlog の新しい項目）、AC7・AC8（6 file）と AC10 の新設、Design Readiness、Contract Audit の round 2 の表、Contract Ledger（J5 = b と上書きの JAN の 2 行）、Contract Probe の P1、Review Focus。本 packet に Open Questions の節は無く、owner の判断事項は Review Focus の J1〜J5 の項（J5 を答え b に直した）

round 2 の Codex（発注 553、`ad709ba7`）= reject（P1 0 / P2 3 / P3 1。Ordinary Operation に具体的な反例あり）。round 1 の 12 件の closure は、#1 が「届いていない」（下の #2）、#5 が「P1 を退けた理由は成り立たない」（下の #1）で、他の 10 件は届いた。PR #196 に投稿された review は 0 本（`gh api repos/kosei-w90607/inventory-system-desktop/pulls/196/reviews` が `[]`）で、件数は最終報告の 4 件。あわせて owner の決定 TD-426（repo 外の回答台帳、2026-10-24）を反映した。裁定は Coordinator（2026-10-24）、是正は起草役の `ea28d8a1`。

| # | finding | 裁定 | 是正 |
|---|---|---|---|
| 1 | P2 `TEXT(セル,"0")`・表示形式 `0` は 8 桁の JAN の先頭の 0 を保たず、検査を通って 7 桁の JAN が入る。P1 に安全を頼らないの主張が成り立たない | 受理 | TD-426 で関門の 6 の検査が `JANコード` の桁（8 桁・13 桁）とチェックディジットを見るので、先頭の 0 の落ちと桁の打ち間違いはプレビューが名指しで止める。backlog の関門の 6 の項目で、起票で決める候補の旧 (3)（check digit）を外して確定の検査に移し（TD-426）、AC の要旨に判定（BIZ-01-D1 と同じ `biz::jan_code::validate`、8 桁にも当てる）・空は通る・先頭 2 の 13 桁（店内コードの範囲）と共有 JAN はチェックディジットが合えば通ること（GS1 の同じモジュラス 10。手入力の create は今もこの判定で受け付ける。51 UI-01b-D17 の理由の列「実バーコードはチェックディジット整合が保証される」）を書き、触る正本に BIZ-01-D1 と master-tables の「CSV の取込みは対象外」の句を足した。候補 (1) は 8 桁の数字の `商品コード` で `JANコード` が空の行に残し、(2) を「数字だけの `商品コード` が `JANコード` と同じでない行」にした。手順書の流れ 1 の手順 4 は、検査がある前提で最小にした（D-127「継ぎ足さない」）: 形式を桁の数の 0 を並べた `TEXT`（13 桁・8 桁）にし、表示形式 `0` の別の道を外し、確かめの列を 1 つの式（数字だけの `商品コード` が `JANコード` と同じでない行）にした。8 桁の値に 13 桁の形式を当てた `00000` + 8 桁はチェックディジットが合って TD-426 を通る（起草役が計算で確かめた）ので、`JANコード` が 13 桁で `00000` から始まる行の確かめを手順 4 に足し、backlog の候補 (4) にした。J4 の変わる所の候補 (1) の句を外した。流れ 2 の手順 6 の照合に商品名を足した（BOM が無く CP932 として読めた文字化けを拾う）。Contract Probe の P1 を判定し直し、Plan Gate の前に要らないとした（理由は Contract Probe の P1。外れ方ごとに予行演習で名指しの停止か照合の不一致になる）。AC11 を足した |
| 2 | P2 backup の保持期限の対策が流れ 3 にしかなく、流れ 2 の予行演習で日をまたぐと復元の元を失う | 受理 | 流れ 2 の冒頭の後に、手順 3〜7 を同じ日に行い、終えられなければ手順 3 の backup へ復元して 0 件を確かめてから閉じる段落を置いた（流れ 3 と同じ根拠〈`cleanup_old_backups` の `age.num_days() > i64::from(retention_days)`〉）。Ordinary Operation の 2 行目、Matrix の C1・FM3・C1 の読み取りの行、Contract Ledger、AC7 の読み取り |
| 3 | P2 JAN を空にする訂正手順が、関門の 6 の拒否条件（13 桁の数字の `商品コード` で `JANコード` が空）と衝突する | 受理。形は「検査を上書きの行でも緩めず、手順書はその JAN を消す訂正を示さず止めて owner に諮る」を選んだ | 正本と今の経路を実読した: JAN を変える経路は CSV の上書きだけ（30 §4.9.2、商品フォームの edit は JAN を読むだけ）、`preview_import` は行の検証を重複の分類より前に行う（`existing` を引いた後に `error_rows` へ入れて `continue`、`// 4. 重複チェック` の前）ので、関門の 6 の検査は上書きの候補の行にも当たる。選んだ理由: TD-421 は新規と上書きの行を分けておらず、13 桁の数字の商品コードで JAN が無い状態は TD-421 が「レジの売上に引き当たらない」として止める形そのもの。その商品の JAN を消したい理由は、その JAN を商品コードにしたこと自体の誤り（店がその JAN を使っていない等）で、手順書の「直せない誤り（商品コード）」と同じ扱いになる。上書きの行だけ warning にとどめる形は TD-421 の範囲を AI が狭める製品の振舞いの変更なので採らない（owner に諮る選択肢にもしない。今の決定のまま両立する）。是正: 手順書の入れた後の表の JAN の行（新しい JAN は 8 桁か 13 桁でチェックディジットの合う値、13 桁の数字の商品コードの商品の JAN は消せず止めて owner に諮る）、backlog の項目の AC の要旨（上書きの候補の行にも当てる）と候補 (3)（旧 (4)。JAN を消す warning の対象を 13 桁の数字でない商品コードに限った）、Ordinary Operation の 4 行目、Matrix の C10・FM13・C10 の読み取りの行・Negative Paths、Scope の「アプリの側に足りない機能」、Design Readiness |
| 4 | P3 Matrix の Residual Test Gaps が、外した手作業の CSV の中の重複の確かめを安全の根拠に残している | 受理 | プレビューの重複の行の error とエラー行 0 の確認に頼り、退行すると代わりの確かめが無いこと、直接の回帰 test は runtime の lane の残課題であることに書き換えた |

TD-426 の反映（関門の 6 の検査に「`JANコード` の列が 8 桁・13 桁でない、またはチェックディジットが合わない行」を足す、owner の決定 2026-10-24）:

- `docs/Plans.md`: 関門の 6 の行に 3 つ目の形と TD-426 を足した。
- `docs/backlog.md`: 関門の 6 の項目の本文・触る正本・AC の要旨・起票で決める候補（上の #1・#3）。
- 手順書: 今の事実の黙って入る形の行、J5 の答え、J4 の変わる所、流れ 1 の手順 4、流れ 2 の手順 4（名指された JAN は元の資料から入れ直し、合わなければ止めて owner に行の一覧を見せる）、入れた後の表の JAN の行。
- packet: 規模と手法、遷移の記録 7、Owner Effort Budget（介入 2 を消費）、Goal、Ordinary Operation、Scope、AC7・AC11、Design Readiness、Contract Audit の round 2 の Codex の是正の表、Contract Ledger、Contract Probe の P1、Review Focus。Matrix の C2・C6・C10、FM4・FM9・FM13、Test Matrix、Negative Paths、Residual Test Gaps。

AC の数え直し: AC1〜AC11 の 11 本（AC11 を新設。AC7 の読み取りに流れ 2 の同じ日の規則を足した。他の AC の command は変えない）。

round 3（天井、`ea28d8a1`）: Claude 側 fresh Opus 5.5 = reject（P1 0 / P2 1 / P3 5。Ordinary Operation は外部前提が未確認）、Codex（発注 561）= reject（P1 0 / P2 2 / P3 0。Ordinary Operation に具体的な反例あり）。round 1・2 の是正の確認は、Opus の R2-3 が「届いた（コードの列）。同じ仕組みが数値の列に残る」、R2-7 が「退けた理由が一部成り立たない」（どちらも下の #2）で、他は両者とも届いた。天井の disposition は一括是正（Coordinator の裁定、2026-10-24）。是正は起草役の `aa8c251a`。

| # | finding（出所） | 裁定 | 是正 |
|---|---|---|---|
| 1 | P2 手順 2 の表の「店がその JAN を使っていない」行の `JANコード` = 空が D-028（スキャンしない商品の JAN を空にする運用を、請求書の照合と Z004 の参照の価値を失うとして退けた）と衝突する。手順 3 の `PLU対象` の全行空と、入れた後の表の無条件の `PLU対象` = `1` も同じ規則に合わない（Codex F1） | 受理（Coordinator の裁定） | 手順 2 の表に `PLU対象` の列を足し、その行を「独自コード・Excel の JAN（空にしない）・`POS在庫連動` `0`・`PLU対象` `0`」にした。表の下に、JAN を空にしない理由（D-028、master-tables の `plu_target`）と、`POS在庫連動` を `0` にする理由（レジで読まないので売上で減らす在庫が無い。使っているかは JAN ごとに決めるので、同じ JAN の行は全部この種類になり、30 の冒頭の共有 JAN の不変条件に当たらない）の箇条を置いた。手順 3 を「`PLU対象` は手順 2 の表の値」に、入れた後の表の JAN の行を「レジで読むなら `1`、読まないなら `0`」とし、レジで読まなくなっただけの商品は JAN を消さず `PLU対象` を `0` にする句を足した。流れ 2 の手順 6 の照合に、その行が在庫連動でなく、JAN を持ってレジの書出しの対象外であることを足した。現物で確かめたこと: `preview_import` は `PLU対象` の `0` を `Some(false)` に読み、空のときの導出（`commit_import` の `should_default_plu_target`）を通らない。独自コードなので TD-421 の停止に当たらない。TD-426 の検査は受ける（名指されたら流れ 2 の手順 4 のとおり元の資料から入れ直す）。J4 の a・b のどちらでも、8 桁の使っていない JAN はこの行で独自コードになる。packet の Contract Audit・Contract Ledger・Review Focus、Matrix の C4・FM6・C4 の読み取りの行 |
| 2 | P2 Excel の CSV の保存が表示の値を書くと、原価・`m` の初期在庫が表示の桁で丸まり、プレビューが正常な行として入れる。守りがコードの列にしかなく、照合の相手の Excel の表示とも合ってしまう（Opus #1、Codex F2。同じ欠陥として束ねた） | 受理（Coordinator の裁定）。アプリの側で見つける案は backlog の関門の 6 の項目の候補 (5) に置き、必須にしない（出所は同じ round の review） | (a) 手順 4 を、5 つの数字の列（`商品コード`・`JANコード`・`売価`・`原価`・`初期在庫`）を全部の桁の文字列の値にする手順に置き換えた（`売価` と個数の `初期在庫` は `"0"`、`原価` と `m` の `初期在庫` は `"0.00"`。`parse_hundredths` は小数 1〜2 桁を受けて `12.00` を 1200 にし、売価と個数の初期在庫は `parse::<i64>` で読む）。表示形式「標準」の案は、標準の表示の桁が CSV にどう書かれるかが P1 のまま残るので採らなかった。`TEXT` は空のセルを 0 にする（空の原価が黙って 0 になり、空の JAN がチェックディジットの合う `0000000000000` になる）ので、空は空のまま残す `IF` の形にした。(b) `TEXT` の列を元の列と置き換える前に、`売価`・`原価`・`初期在庫` の `TEXT` の値が元の値と違う行を全部の行の確かめの列（`=IF(元のセル="",FALSE,--TEXTのセル<>元のセル)`）で数えて止める段を置いた。この 1 つの列が、形式の選び違い（原価に `"0"` を当てた類。小数の桁だけを数える確かめでは通り、照合の相手の文字列とも合ってしまう）と、列が持てる桁を超える値（原価・`m` の初期在庫の小数 3 桁以上、売価・個数の初期在庫の整数でない値。原価の 3 桁以上は店の決まり〈SPEC-UNIT-D5 の理由〉で直すかを owner が決める）の両方を名指す。置き換えた後は 5 つの列の `COUNT` が 0 であること（文字列にし忘れた列・行を名指す）を見る。流れ 2 の手順 6 の照合を、Excel の表示でなく文字列にした値と比べ、原価に小数のある商品と `m` の商品を必ず含める形にした。今の事実の Excel の CSV の保存の行に、数値の丸めと、丸まった値をプレビューが正常な行として受けることを書いた。(c) Ordinary Operation の 1 行目の P1 の列と Contract Probe の P1 に、この外れ方と、手順 4 で表示に頼らなくなることを足した。P1 を Plan Gate の前に回さない理由は今も成り立つ（CSV の値は表示の形式に依らず、外れても保存の前の確かめか予行演習で名指される）。手順 4 の確かめを飛ばした場合は P1 の外れ方ではなく手順を守らない場合で、照合の抜き取りでしか拾えないことを P1 と Matrix の Residual Test Gaps に書いた。Contract Ledger の BIZ-01-D8 の行、Matrix の C6・C7・FM9・FM10・C6 と C7 の読み取りの行・Boundary Checks |
| 3 | P3 `00000` の行の直し方が `TEXT` の形式の替えだけで、13 桁の種類に入れていた行の商品コード（J4）と種類ごとの数が残る。「桁は手順 2 の種類で決まる」が共有の群の行で決まらない（Opus #2） | 受理（Coordinator の裁定） | 直し方を「手順 2 の 8 桁の JAN の行（共有の群か JAN を使っていない行はその種類のまま）として直し、商品コードを `00000…` にしていたら J4 のとおり決め直し、種類ごとの数を数え直す」にし、`TEXT` の桁を「元の JAN の桁（商品のバーコードの数字の数）で選ぶ」にした（手順 4） |
| 4 | P3 J2 の既定の「長く動いていないコードの無い品は商品にしない（D-076）」に当たる行の種類が表に無い（Opus #3） | 受理（Coordinator の裁定） | 手順 2 の表の J2 の行の上に「JAN もコードも無く、棚卸しの数えから外している長く動いていない品（D-076。どの品かは owner が決める）→ 入れない」の行を置いた。Contract Ledger の行の種類の行と D-076 の隣接の行、Matrix の C4 |
| 5 | P3 CSV にする sheet の形（1 行目が列名、同じ名の列、作業の列）が書かれていない（Opus #4） | 受理（Coordinator の裁定） | 手順 5 の頭に、1 行目を列名にして同じ名の列を置かない段（`parse_product_csv` は 1 行目を列名にし、同じ名の列は `field_map.insert` で後ろの値が残る。product_csv_importer.rs を実読）と、作業の列・確かめの列・数えたセルを消す段（保存されるのは開いている sheet だけ）を置いた。Contract Ledger の行の種類の行、Matrix の C6・FM9 |
| 6 | P3 Contract Audit に round ごとの表が重なり、2 つは自分で今の文でないと書いている。J5 の「変わる所」に変更の履歴が残る（Opus #5） | 受理（Coordinator の裁定） | Contract Audit を今の文の一巡の表 1 つに置き換えた（round 3 の是正を含む）。J5 の変わる所を今の手順の番号だけにした。同じ形の継ぎ足しだった AC の round ごとの実測の段も今の head の実測 1 つに置き換え（AC5・AC6・AC11 の mutation を今の文で当て直した）、Review Focus の round 2 の Codex の是正の項を今の問いの項に置き換えた。Contract Probe の P1 から round の経緯の句を外した |
| 7 | P3 Plans の candidate の「初期投入の手順」の行が、手順書ができた後も candidate のまま（Opus #6） | 本 PR では直さない（Coordinator の裁定） | closeout で扱う。本 PR は Non-scope の「`docs/Plans.md` は関門の 6 の 1 行の他を変えない」のまま |

closure（`aa8c251a`）: Claude 側 fresh Opus 5.5 = approve（P1 0 / P2 0 / P3 5。Ordinary Operation は成立）、Codex（発注 566）= reject（P1 0 / P2 2 / P3 0。Ordinary Operation に具体的な反例あり）。round 3 の 6 件の closure は、Codex の F2 が「閉じていない」（下の B。Contract Probe の P1 の延期の推論）で、他は両者とも閉じた。全件を受理し、裁定は Coordinator（2026-10-25）、是正は起草役の `d9e9fcff`。あわせて base 同期（`origin/main` `8db97271` を merge）で、#188 が main に入れた導入の前の関門の 6・7 の後に本 lane の項目を 8 として置き、packet（本節より前）・Matrix・手順書・backlog の「関門の 6」を「関門の 8」にした（本節の先行 round の記述は当時の番号のまま）。

| # | finding（出所） | 裁定 | 是正 |
|---|---|---|---|
| A | P2 比べの式 `--TEXTのセル<>元のセル` が左辺だけを数にし、元のセルが文字列の数（`"12.34"`）の正しい行で TRUE になって止まる（Codex N1、Opus N3） | 受理 | 手順書の流れ 1 の手順 4 の比べの列を `=IF(元のセル="",FALSE,IFERROR(--TEXTのセル<>--元のセル,TRUE))` にし、違う行の原因に「数として読めないセルのある行」（元のセルが `-` や単位の付いた文字、`TEXT` の式が無い行）と直し方を足した。Matrix の Test Matrix に、数・文字列の数・空欄・文字列にし忘れた行の 4 つで止まる／進むを示す C7 の行を足した。Contract Audit の安全な整数の範囲 |
| B | P2 Contract Probe の P1 の「文字列がそのまま保存されなければコードも崩れ、JAN 検査で止まる」に根拠が無い（原価と JAN が必ず一緒に壊れるとは言えず、照合も全件でない）（Codex F2 の未 closure） | 受理。P1 は実機で確かめない（owner の PC に Excel が無い、repo 外の回答台帳 TD-437） | Excel の保存の振舞いを前提にせず、手順書の流れ 1 の手順 5 に、保存した file の中身を取込みの前に全部の行で確かめる段（PowerShell の `Import-Csv` で 5 つの数字の列が手順 4 で決めた形でない行を行の番号・列・値で出し、確かめた行の数を手順 2 の和と比べる。外れたら sheet のセルが違えば手順 4 から、sheet が正しく file が違えば取り込まず止める）を置いた。既にある段（SHA-256 は中身を見ず、プレビューは丸まった正しい形の数を受け、照合は抜き取り）では全部の行の値を覆えないので新しい段にし、同じ役の手順 4 の `COUNT` の確かめ（数値のまま残ったセルは file の形の外れになる）を外した。Contract Probe の P1 を「実機で確かめない。手順の確かめの段で外れを検出する」に書き換え、延期の推論を消した。今の事実の Excel の CSV の保存の行と冒頭の未確認の点、Ordinary Operation の 1 行目、Design Readiness、Contract Audit、Contract Ledger、Impact Review Lenses、Review Focus、Matrix の C6・FM9・C6 の読み取りの行・Residual Test Gaps |
| C | P3 流れ 2 の手順 6 の照合の相手が `TEXT` の出力そのもので、丸めの検出にならない（Opus N1） | 受理 | 照合の相手を、在庫の数と原価は流れ 1 の手順 3 で値を取った元のセルの数式バーの値、ほかの項目は手順 5 で確かめた CSV の値にした。Matrix の C7・Residual Test Gaps と Contract Probe の P1 を同じ 1 通りにした |
| D | P3 入れた後の表の「レジで読まなくなっただけの商品は `PLU対象` を `0`」が `POS在庫連動` の扱いを 2 通りに読ませ、その段がどの Would fail if にも当たらない（Opus N2） | 受理 | 「流れ 1 の手順 2 の JAN を使っていない行と同じ」を外し、`POS在庫連動` は空にして今の値を保つ（変えるときは在庫連動の行で直す）と書いた。Matrix の C10 の Would fail if に、13 桁の JAN を入れ直す行の `PLU対象` がレジで読むかで分かれないことと、`POS在庫連動` の扱いが 2 通りに読めることを足した |
| E | P3 手順 4 の「最大」を控える作業を後の段で使わない（Opus N4） | 受理 | 「最大」を外した。同じく後の段で使わない 0 の行の数を控える作業も外し、`売価` か `原価` が 0 の行の一覧だけを残した |
| F | P3 遷移の記録に round 3 と一括是正 `aa8c251a` の行が無い（Opus N5） | 受理 | 遷移の記録に 8（round 3 の一括是正、`aa8c251a`）と 9（本 commit、closure の後の是正と base 同期）を足した。5・7 と本節の先行 round の「本 commit」を各 round の commit（`a9f0f439`・`ad709ba7`・`ea28d8a1`・`aa8c251a`）に置き換えた |

### closure 2（head `d9e9fcff`、是正 2026-10-26）

Claude 側 fresh Opus 5.5 = reject（P1 0 / P2 1 / P3 3。Ordinary Operation は外部前提が未確認: 流れ 1 を行う PC に Excel があるか）。closure の A〜F は全部閉じた。是正の前に相談役（Fable 5）が反例を探し、finding 2・3 の closure の案に漏れを見つけた（下の F2・F3 の「案を替えた」）。全件を受理し、裁定は Coordinator（2026-10-26、相談役の案を採る）、是正は起草役の `7174f0ba`。

| # | finding（出所） | 裁定 | 是正 |
|---|---|---|---|
| F1 | P2 流れ 1 を「開発用の PC で」行うと書くが、開発用の PC には Excel が無い（repo 外の回答台帳 TD-437）。手順 3〜5 は Excel の `TEXT`・`COUNTIF`・CSV UTF-8 の保存を使う（closure 2 #1）。packet の Impact Review Lenses の「Excel は店と開発用の PC の既存の道具」も TD-437 と食い違う（相談役） | 受理。どの PC に Excel があるかを owner に新たに問わない（TD-437 と L-153・L-113・L-076・L-239 で足りる）。公開 repo には PC を名指さず条件の形で書く（Goal の失敗定義の 5 行目） | 手順書の流れ 1 の冒頭を「Excel のある Windows で行う（開発用の PC には Excel が無い。TD-437）」にし、「repo の外の folder」を「在庫 Excel・CSV・backup を repo と開発の folder（repo の checkout）に置かない」に、手順 1 の「repo の外の folder」を「別の folder」にした。流れ 2 の冒頭に CSV を開発用の Windows へ写して SHA-256 を確かめる句を、手順 6 に元の在庫 Excel を開ける PC を手元に置く句を足した。packet の Ordinary Operation の 1 行目の初期状態に「Excel を使える Windows」、Impact Review Lenses の環境・再現性、Contract Probe の P1 の理由（「確かめに安全を置かず、保存した file を手順 5 で確かめる〈TD-437 の後の裁定〉」）、Contract Audit の失敗の時点と順序。Matrix の Residual Test Gaps の P1 |
| F2 | P3 手順 4 で `=ROUND(セル,2)` にすると決めた行が、照合で元のセルの値と合わない（closure 2 #2）。相談役: 同じ行は手順 4 の比べの列でも TRUE のまま 0 にならず、置き換えへ進めない。手順 6 に 1 文を足す案では手順 4 が直らない | 受理。案を替えた（相談役の案） | 手順書の流れ 1 の手順 3 の `原価` を、最初から店の決まり（小数第 3 位で四捨五入して小数第 2 位まで。共通規則 SPEC-UNIT-D5 の理由、35 SPEC-STK-VAL-D3）で丸めた値にした（例 `=IF(税抜原価のセル="","",ROUND(税抜原価のセル,2))`。`ROUND` は空のセルを 0 にするので、空は空のまま残す。起草役が足した）。手順 4 に丸めで値が変わった行の一覧（0 の行の一覧と同じ形で owner が見る）を足し、行ごとに `ROUND` を決める分岐を外した。`m` の初期在庫の小数 3 桁以上は丸めの決まりが無いので止めたまま。手順 6 の照合を「原価は同じ元のセルの値を同じ店の決まりで丸めた値と比べる」にし、一覧を見る句に丸めの行を足した。packet の Ordinary Operation の 1 行目、Contract Audit の安全な整数の範囲・0 の表示・失敗の時点と順序、Contract Ledger の BIZ-01-D8 の行、Review Focus。Matrix の C7・FM10・C7 の読み取りの行・Boundary Checks の precision・Residual Test Gaps |
| F3 | P3 比べの列 `=IF(元のセル="",FALSE,IFERROR(…))` で、`IFERROR` が else の枝の中だけにあり、元のセルがエラー値の行は列がエラーのまま `COUNTIF` に数えられず通る。文字列にし忘れた行で元が 0 なら FALSE で通る（closure 2 #3）。相談役: closure の案 `IF(元のセル="",FALSE,…)` は、元が空で `TEXT` に値がある行（`IF` の無い `TEXT` で `"0.00"`）を FALSE で通し、空の原価が黙って 0 円で入る | 受理。案を替えた（相談役の案） | 式を `=IFERROR(IF(元のセル="",TEXTのセル<>"",IF(TEXTのセル="",TRUE,--TEXTのセル<>--元のセル)),TRUE)` にし（手順書の流れ 1 の手順 4）、違う行の原因に元のセルのエラー値と空のセルに値を書いた `TEXT` を足した。元のセル（エラー値・空・数・数に見える文字列）× `TEXT` のセル（空・数に見える文字列・そうでない文字列）の 12 の組を一巡し、止まるべき組が全部 TRUE、進むべき組（元も `TEXT` も空、同じ値の数どうし）が FALSE になることを確かめた（Excel の実機は使えず、Excel の一般の振舞い〈空のセルと `""` の比べが TRUE、エラーの伝播、`IFERROR` が全部のエラーを捕まえる〉から読んだ）。式が「真の空のセルの算術が 0」と「`TEXT` に文字列を渡したときの振舞い」に頼らなくなった。packet の Contract Audit の安全な整数の範囲・0 の表示。Matrix の C7 の比べの列の行（期待と Would fail if） |
| F4 | P3 手順 5 で行が出たときの見分けが、(a) Excel が sheet の下に書く全部の列が空の行と、(b) 値の中の改行を外す（closure 2 #4）。相談役: (a) は保存した CSV を Excel で開いて上書きすると手順 5 の禁止に当たり、行の数の 1 ずれが「sheet が正しく file が違う」の分岐へ誤って進む | 受理 | 手順書の流れ 1 の手順 5 に、出た行が全部の列が空の行なら CSV にする前の sheet のその行を消して手順 5 の保存からやり直す（保存した file を Excel で開いて上書きしない）、行の数が空の行の数だけ多いのも同じ原因で「sheet が正しく file が違う」には当たらない、を足した。改行の句を前提に依らない文「値の中の改行は、ここで数が合わなくなるか、プレビューの行の error で名指される。どちらでも取り込む前に止まる」に置き換えた（`Import-Csv` の引用符の中の改行の扱いは確かめていないので断定しない）。packet の Contract Probe の P1、Contract Audit の失敗の時点と順序。Matrix の C6 の読み取りの行 |

置き換えた旧い記述（D-127「継ぎ足さない」）:

- 手順書の流れ 1 の冒頭の「開発用の PC で、repo の外の folder で行う」と手順 1 の「repo の外の folder」（F1）
- packet の Impact Review Lenses の「Excel は店と開発用の PC の既存の道具」と、Contract Probe の P1・Matrix の Residual Test Gaps の「owner の PC に Excel が無く（無い）」の不可能性の理由（F1）
- 手順書の流れ 1 の手順 3 の「Excel の税抜の原価をそのまま入れる……小数 3 桁以上の値は手順 4 の確かめで止める」と、手順 4 の「原価の小数 3 桁以上が袋の値段 ÷ 入り数の計算なら……`=ROUND(セル,2)` にするかを owner が決める」の分岐、手順 4 と packet・Matrix の「原価・`m` の初期在庫の小数 3 桁以上」で止める句の原価の部分、照合の「在庫の数と原価を元のセルの数式バーの値と比べる」の原価の部分（F2）
- 比べの列の旧い式 `IF(元のセル="",FALSE,IFERROR(--TEXTのセル<>--元のセル,TRUE))` と、packet の「元の値との比べは空のセルを FALSE にする（`TEXT` の列も空のまま残すので比べない）」、Matrix の「文字列にし忘れた行 → `IFERROR` で TRUE になり止まる」の仕組みの説明（F3）
- 手順書の流れ 1 の手順 5 の「sheet の行と、値の中の改行（取込みは扱えない）を見て直す」（F4）
- 遷移の記録の 9 と closure の節の「本 commit」を、その commit `d9e9fcff` にした

### closure 3（head `7174f0ba`、是正 2026-10-26）

Claude 側 fresh Opus 5.5 = reject（P1 0 / P2 0 / P3 5。G が固定なしで未 closure。Ordinary Operation は外部前提が未確認: P1）、Codex（発注 584）= reject（P1 0 / P2 2〈B・G の未 closure〉/ P3 0。Ordinary Operation に具体的な反例あり: BOM の無い UTF-8 が手順 5 を通り、取込みで初めて file の error になる。Windows PowerShell 5.1.26100.9711 の実測）。closure 2 の F1〜F4 と closure の A・C〜F・H〜J は両者とも閉じた。是正の前に相談役（Fable 5）が反例を探し、B の最小の形（`Import-Csv` の前の BOM の確かめ）と G の固定の置き場（Matrix の C9）を出した。全件を受理し、裁定は Coordinator（2026-10-26、相談役の案を採る）、是正は起草役の本 commit。同じ round の前に `origin/main`（`5506c7a4`）を単段で取り込んだ（`8ea82ab5`）。

| # | finding（出所） | 裁定 | 是正 |
|---|---|---|---|
| B | P2 手順 5 の `Import-Csv -Encoding UTF8` は BOM の有無に依らず UTF-8 で読み、アプリの `decode_bytes`（先頭の 3 byte が BOM なら UTF-8、それ以外は CP932）と分岐の条件が違う。BOM の無い UTF-8・UTF-16 の file が手順 5 を通り、取込みの file の error で初めて止まり、その段で原因と直し方を名指さない。packet の P1 の「BOM が無く CP932 で書かれる → 列名が読めず、全部の行が出る」は実測（列名の重複の読取りの error、件数 0）と食い違う（Codex の B の未 closure。Claude 側 N1 と同じ根） | 受理（相談役の案） | 手順書の流れ 1 の手順 5 の block の先頭（`Import-Csv` の前）に、`[System.IO.File]::ReadAllBytes($f)` で先頭の 3 byte が `EF BB BF` かを見る確かめを置き、違えば「止める: 先頭に UTF-8 の BOM が無い。『CSV UTF-8（コンマ区切り）』で保存し直す」だけを出して行を読まない形にした（`if … else` の 1 つの文にしたので、貼った行が 1 行ずつ実行されても `Import-Csv` に進まない）。block の後の文に、止まったときの直し方、アプリより厳しいのは意図どおりであること、BOM 付きで中身が UTF-8 として不正な file は流れ 2 の手順 4 の file の error で止まることを書いた。流れ 2 の手順 4 に file の error のときの戻り先（流れ 1 の手順 5 の保存と確かめ）を足した。packet の P1 の CP932 の予測を BOM の確かめの文（`decode_bytes` と byte 単位で同じ条件）に置き換え、Claude 側 N1 の分岐は足さなかった（CP932 の file は `Import-Csv` に届かない）。値の中の改行の文を実測に合わせて「プレビューの行の error で名指される。取り込む前に止まる」に縮めた（手順書と P1）。Ordinary Operation の 1 行目の次へ進む条件、Contract Audit の失敗の時点と順序、Contract Ledger の IO-03 の行、Matrix の C6 の契約・C6 の行の Would fail if（「先頭の BOM の確かめが無い〈文字コードの外れが取込みまで素通りする〉」ほか）・FM9・Residual Test Gaps |
| G | P2 作業の場所（流れ 1 は Excel のある Windows、照合では元の在庫 Excel を開ける PC）は手順書で直っているが、戻しを捕まえる AC も Matrix の Would fail if も無い（Claude 側の G、Codex の G） | 受理（相談役の案。AC は足さない） | Matrix の C9 の契約に「流れ 1 は Excel のある Windows で行い、流れ 2・3 の照合では元の在庫 Excel を開ける PC を手元に置く（TD-437）。CSV は開発用の Windows へ写して SHA-256 を照らす」を、C9 の行の Would fail if に「流れ 1 が Excel の無い PC で行うと読める／流れ 2・3 の照合で元の在庫 Excel を開ける PC が無い／開発用の Windows への写しの SHA-256 の照合が消える」を足し、見る所に流れ 1 の冒頭と流れ 2 の手順 6 を足した。FM12 に同じ外れ方を足した |
| N2 | P3 手順 5 の「sheet のセルが手順 4 の形でない」の直し方が `TEXT` の段の 1 つだけで、独自コードの誤りや負の値は同じ段を繰り返しても直らない（Claude 側 N2） | 受理 | 直し方を列で分けた: `商品コード` の誤りは手順 2 の表と箇条、負の値と空の原価は元の資料から入れ直して手順 3 から、数値のまま・`TEXT` の当て忘れ・形式の選び違いは手順 4 の `TEXT` の段から、どれも直した後は手順 5 の保存から（手順書の流れ 1 の手順 5）。packet の P1 と Contract Audit、Matrix の C6 の行 |
| N3 | P3 流れ 2 の照合の「確かめた CSV の値と比べる」に CSV を見る手段が無く、Excel で開くと表示が崩れて上書きの筋を生む（Claude 側 N3） | 受理 | 手順書の流れ 2 の手順 6 に「CSV はメモ帳か `Import-Csv -LiteralPath $f -Encoding UTF8 \| Out-GridView` で見る。Excel で開かない」を足した |
| N4 | P3 手順 5 の安全の段が PowerShell 5.1 の振舞いに依るのに probe の記録が無い（Claude 側 N4） | 受理 | Contract Probe に P3 を足し、closure 3 の Codex（発注 584、2026-10-26、Windows PowerShell 5.1.26100.9711、合成の CSV）の観測の要旨を出所つきで書いた。是正の前の block の観測であることと、足した BOM の確かめの行は closure 4 の Codex で 4 つの合成の file（BOM 付きの正しい file・BOM の無い UTF-8・CP932・UTF-16）で回し直すことを書いた。Matrix の Residual Test Gaps |
| N5 | P3 (a) Ordinary Operation の 2 行目の初期状態に照合に要る PC が無い。(b) 手順書の今の事実の `POS在庫連動` の行が共有 JAN の拒否を先の版の話として書くが、今の main の `preview_import` に入っている（Claude 側 N5） | 受理 | (a) Ordinary Operation の 2 行目の初期状態に「元の在庫 Excel を開ける PC」を足した。(b) 手順書の行を「今の版に入っている（D-118 の (b) の b1、PR #188）」にし、出所に `import_row_link_error` を足した。packet の Review Focus の同じ句も今の版に合わせた |

置き換えた旧い記述（D-127「継ぎ足さない」）:

- packet の Contract Probe の P1 の「BOM が無く CP932 で書かれる → 列名が読めず、全部の行が出る」（B）
- 手順書と P1 の値の中の改行の「ここで（この段で）数が合わなくなるか、プレビューの行の error で名指される。どちらでも取り込む前に止まる」と、Matrix の C6 の「値の中の改行が数の不一致かプレビューの行の error で止まると書かれず前提に頼る」（B）
- 手順書の流れ 1 の手順 5 の「その列を手順 4 の `TEXT` の列を作る段からやり直して保存し直す（空の原価は元の資料から入れる）」と、packet の P1・Contract Audit、Matrix の C6 の「sheet のセルが違えば手順 4 から」（N2）
- 手順書の今の事実の `POS在庫連動` の行の「D-118 の (b) を含む版（初回の導入の版。…の 2）では」と、packet の Review Focus の「D-118 の (b) を含む版だけであること」（N5 (b)）
- 遷移の記録の 10 と closure 2 の節の「本 commit」を、その commit `7174f0ba` にした


### closure 4（head `6f167bc1`、2026-10-26）

Claude 側（fresh Opus 5.5）= approve（P1 0 / P2 0 / P3 4）、Codex = approve（P1 0 / P2 0 / P3 0）。B・G・N2〜N5 は 2 本とも閉じたと判定した。Codex は是正の後の手順 5 の block を Windows PowerShell 5.1.26100.9711 で合成の 5 つの file に回し、現物の `parse_product_csv` と突き合わせた（上の Contract Probe P3）。Claude 側の P3 を全件受理し、Coordinator が本 commit で直した。

| # | 出所 | 是正の場所 |
|---|---|---|
| C4-1 | Claude 側 1（P3） | 手順書の流れ 1 の手順 5 の直し方の最後の文を「直した段から順に手順 5 の保存と確かめまで進め直す」に（手順 3 から直した行が手順 4 の一覧と比べの列を飛ばす読みを消した） |
| C4-2 | Claude 側 2（P3） | 流れ 2 の手順 4 の file の error の戻り先に、必須の列の不足（sheet の 1 行目の列名）を足した |
| C4-3 | Claude 側 3（P3） | 流れ 2 の手順 6 の `Out-GridView` の前に `$f` に写した CSV の path を入れる、を足した |
| C4-4 | Claude 側 4（P3） | Ordinary Operation の 3 行目の初期状態に、元の在庫 Excel を開ける PC を足した |
| C4-5 | Coordinator | Contract Probe P3 と Matrix の Residual Test Gaps の「BOM の確かめの行は 5.1 でまだ回していない」を、closure 4 の Codex の実測で置き換えた |

Final Review と merge（2026-10-27、closeout で追記）: Final Review broad（head `fb5e2d05`）は Claude 側 Fable 5 = approve（P3 2）、Codex 600 = reject（P2 1: 手順書の J3 を c にすると、照合が元 Excel の数と比べるので正しい投入でも進めない）。相談役（Fable 5）の反例探しで J3 の b と J4 の a にも同じ型の穴があり、3 件を受理して `526317af` で手順書の J3・J4 の行の「変わる所」に照合の比べ先と 8 桁の数字の商品コードの JAN の守りを書いた。closure（head `526317af`）は Claude 側 Fable 5 = approve、Codex 603 = approve（P2 closed、新規 0）で、helper の review の record は pass。r4 は owner の承認で pass。Ready の後の hosted CI の `Merge gate` が success で、owner の R4 の承認と Ready・merge の承認（repo 外の回答台帳 TD-450）の後に helper 経由で merge した（[PR #196](https://github.com/kosei-w90607/inventory-system-desktop/pull/196)）。Final broad の Claude 側の P3 2 件は本 closeout で直した: (1) Workflow State の Final Reviewer 行の「Fable 5.1」を、実際に Final を行った Fable 5（座組表の当面の値、TD-410）にした。(2) D-127 の段落の「先頭の 0 が落ちた 12 桁の行を 1 つの式で数える」の句を、`商品コード` と `JANコード` が同じ 12 桁の行は式が数えず、手順 5 の形の確かめと関門 8 のプレビューが受け持つ、に直した。
