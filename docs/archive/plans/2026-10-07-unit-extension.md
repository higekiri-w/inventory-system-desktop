# Plan Packet: 単位の拡張と、原価 × 数量の価格の基準数量（設計）

第 1 段の 3 lane の 1 つ（2026-10-07 起票。design-first、docs だけ。runtime は別 lane）。並走は `agent/stocktake-p1-3`（棚卸し ③）・`agent/backup-offsite`（backup の PC 外の控え）、ほかに `agent/npm-audit-1006`（PR #146、package 依存）・`agent/plu-clear`（D-110）。本 lane の branch は `agent/unit-extension`（main `95c0aeb0` から）。D 番号は D-113 を使う（Coordinator の指定）。対象の backlog は「単位の拡張」と「原価 × 数量の式が価格の基準数量を持たない残り」。

## Workflow State

- Phase: archive
- Risk: R4
- Plan Commit: 2fc7c022d1c1480868e02ca08c375703bcb0f131
- Amendments: 9a6929fce0262a422828f39e3aa7d7442a7e1d95, eaba9918f85a1b272afeb94ff295fbe1bd40d0b4, 8a2481d302c447c3f03152dd82855a462864f38e, 8fdda36494ee20c974c4348c6355728393476c87, 66c360196cce0bef292cb3550176fd6c84a2d33d
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（subagent_type: writer）
- Plan Reviewer: fresh Opus 5.5 + Codex（model は発注時に決める）
- Final Reviewer: Fable 5.1（Claude 側、R3 以上）+ Codex（GPT-6.1 Sol 既定）。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Review Minimum: 2
- Human Gate: ready,merge,r4
- Branch: agent/unit-extension

遷移の記録:

1. kickoff → spec-check（2026-10-07、起草役）: Coordinator の発注（単位の拡張と、原価 × 数量の基準数量の残り (1)(2)(4)(5)。runtime は別 lane）を Scope にし、Risk を R4 と記録した（下の Risk）。
2. spec-check → design（2026-10-07、起草役）: 正本（`docs/db-design/master-tables.md` の products、`docs/function-design/35-biz-stocktake-service.md` §20.5a SPEC-STK-VAL-D1、`31-biz-inventory-service.md`、`62-ui-manual-sale.md` UI-04-D6、`23-io-z004-parser.md` §13.4.1、`29-io-ej-parser.md` IO-08.5）は単位 2 値・原価の円の整数・数量の整数の parse で、12 単位・m の入力・原価の小数・小数の POS の数量の契約が無い。同じ commit で設計正本を更新した（下の Design Readiness）。
3. design のまま止める（起草の時点）: owner の判断事項 J1〜J4 が残り、design → plan-draft の条件「未解決の設計の問いが無い」を満たさない。
4. design → plan-draft（2026-10-08、起草役）: owner が J1〜J4 をすべて推奨の案に決めた（repo 外の回答台帳 TD-195。下の owner の判断事項）。設計正本と D-113 の未決の記述を決定の文に直し（AC4）、設計の出力は正本にある。残る延期（Contract Probe P3、棚卸し記録詳細のロス原価を移す lane、`tracking-system-tables.md` の列の表）は本 lane の設計の値を変えず、runtime の lane の起票時に決めれば足りるので、未解決の設計の問いは無い（Design Readiness）。plan-gate へは Coordinator が進める。
5. plan-draft → plan-gate（2026-10-08、Coordinator、本 commit）: packet と Matrix（`docs/plans/test-matrices/2026-10-07-unit-extension.md`）が揃い commit されている（`docs/DEV_WORKFLOW.md` Workflow State の表）。AC1〜AC6 の command を plan-gate の直前に逐語で再測し、反映後の期待と一致（AC1 `11`、AC2 1 行と `2`、AC3 `2`・`4`・`1`、AC4 J1〜J4 の 4 行と `0`、AC5 `0`、AC6 `0`）。Draft PR で Plan Review（fresh Opus + Codex）に出す。
6. plan-gate → plan-draft（round 1 の是正、2026-10-08、Coordinator、本 commit）: round 1 の reject の是正（起草役の `8981cbc9`・`a3b64f3f`）と owner の決定（数量を出す画面はすべて m〈TD-203〉、単位をまたぐ集計は個数の種類の「点」と長さの「m」の 2 本〈TD-206・TD-207〉）で Goal・Ordinary Operation・Scope の予定 file（34・56・57 を追加）と wire の型が変わった。`docs/DEV_WORKFLOW.md` Workflow State「a rejection that invalidates Scope or design returns to plan-draft or design」により plan-draft へ戻す（設計の出力は正本にあり、未解決の設計の問いは無いので design までは戻さない）。
7. plan-draft → plan-gate（round 2 へ、2026-10-08、Coordinator、本 commit）: 是正後の packet と Matrix が commit されている（`a3b64f3f`）。AC1〜AC6 の期待は変わらず、起草役が逐語で再測して一致（`11`・`2`・`4`・`0`・`0`・`0`）。round 2 で再 review する。
8. plan-gate のまま是正（round 2、2026-10-08、Coordinator）: round 2 の reject を起草役が `74125ea7` で直した（下の Review Response）。Goal・AC・Scope の予定 file は変えないので plan-gate に留め、round 3（上限）で再 review する（`docs/DEV_WORKFLOW.md` Workflow State「a plan-gate rejection corrected in place stays at plan-gate」）。
9. plan-gate（round 3、上限、2026-10-08、Coordinator）: round 3 は Claude 側 approve・Codex 側 reject（P1 0）。`docs/DEV_WORKFLOW.md` Review Rules の round 天井に達したので round 4 は回さず、残った findings を disposition「同型指摘の一括是正」とした（どれも正本の書き足りない所を埋める是正で、Goal・Scope・AC と設計の方向を変えない）。起草役が `e6aead3d` で直し、Coordinator が現物で確かめた（下の Review Response）。plan-approved は owner の承認を待つ。reviewer の再確認は Final Review。
10. plan-gate → plan-approved（2026-10-08、Coordinator、本 commit）: round 3 の disposition（同型指摘の一括是正）の後、owner が plan-approved を承認した（repo 外の回答台帳 TD-210）。同じ問い合わせで介入の上限を 12 に（TD-211）決め、`2fc7c022` に反映した。Plan Commit = `2fc7c022`（承認した版）。reviewer の再確認は Final Review。
11. plan-approved → implementing（2026-10-08、Coordinator、本 commit。state だけ）: 本 lane は docs だけで、実装の段で足す code は無い。設計の正本は plan-first の commit にあるので、このまま Final Review（R4、Minimum 2）に出す。
12. implementing のまま Gated Amendment 1（2026-10-08、Coordinator）: Final Review broad（head `c4f9822e`）の Claude 側 Fable 5.1 の reject（P2 1 / P3 1）と Codex GPT-6 Astra（発注 255）の reject（P2 4）を全件採用し、起草役が `9a6929fc` で直した（設計正本 10・34・36・51・53・57・66・67・74・75 と、packet の Scope の表・申し送り・Registration・Wire Contract、Matrix の Test Matrix の表）。Goal・AC・Risk・Human Gate は変えない。Plan contract が変わったので、本 Amendment を含む head で fresh broad を 2 本（R4 の Minimum 2）取り直す。
13. implementing のまま Gated Amendment 2（2026-10-08、Coordinator）: fresh broad（head `1c64fc5a`）の Codex GPT-6 Astra（発注 257）の reject（P2 2）と Claude 側 Fable 5.1 の approve（P3 1）を採用した。正本だけで閉じる 2 件（日次・月次の商品別の各段の安全な整数の検査、44・20・40 の proposed の pointer）は起草役の `fe662ef6`（packet・Matrix に触れない）。旧版へ戻す回復の手順で確かめを file の移動の前に置く 1 件は、同じ手順が packet の Impact Review Lenses と Matrix の MNT-03-D13 の行にもあるので、22 §16 と合わせて `eaba9918` で直した（本 Amendment）。Goal・AC・Risk・Human Gate は変えない。Plan contract が変わったので、本 Amendment を含む head で fresh broad を 2 本取り直す。
14. implementing のまま Gated Amendment 3（2026-10-11、Coordinator。起草役の commit は 2026-10-08）: fresh broad（head `12959e48`）の Codex GPT-6 Astra（発注 258）の reject（P2 1 / P3 2）と Claude 側 Fable 5.1 の approve（P3 4）を全件採用し、起草役が `8a2481d3` で直した（設計正本 10・29・31・36・SCREEN_DESIGN と、packet の Scope の表・申し送りの表・Contract Ledger、Matrix の Test Matrix の表）。P2 は入庫の後の在庫の 1 経路の指摘だったが、同じ型の指摘が 3 回続いたので、DB・wire・画面に出す値ごとに「保証する所」を 10 の「安全な整数の範囲」の表にまとめ、lane 全体で塞いだ（この lane の中で塞ぐのは owner の決定、repo 外の回答台帳 TD-212）。Goal・AC・Risk・Human Gate は変えない。Plan contract が変わったので、本 Amendment を含む head で fresh broad を 2 本取り直す。
15. implementing のまま Gated Amendment 4（2026-10-11、Coordinator）: Gated Amendment 3 の記録（`ce904618`）の後に origin/main（`afccc1d0`）を取り込んだ（`2f628124`、競合なし）。取り込んだ後の head で AC を回すと、AC6 の `git diff --name-only 95c0aeb0…` が `4` を返した（main 側の `.prettierignore`・`package.json`・`package-lock.json`・`src/hooks/useUnsavedChangesWarning.router.test.tsx` を数える。本 lane の差分は docs だけのまま）。AC5・AC6 の command が diff の基点を起票の時の main の SHA に固定していたためで、基点を `"$(git merge-base origin/main HEAD)"` に直した（`8fdda364`。同じ形の先例は `docs/archive/plans/2026-10-06-z001-display.md` の AC8）。確かめる中身（traceability の入力を変えない・docs だけ）と期待値 `0` は変えない。直した command の実測は AC5 `0`・AC6 `0`（2026-10-11、`2f628124` に本修正を重ねた作業 tree）。Goal・Scope・Risk・Human Gate は変えない。fresh broad の 2 本は本 Amendment を含む head で取る（Gated Amendment 3 の取り直しと同じ 2 本で、本数は増えない）。
16. implementing のまま Gated Amendment 5（2026-10-11、Coordinator）: fresh broad（head `5d039c84`）の Codex GPT-6 Astra（発注 261）の reject（P2 4）と Claude 側 Fable 5.1 の reject（P2 1 / P3 3）を全件採用した。P2 の 5 件はどれも Gated Amendment 3 で足した「安全な整数の範囲」の表の中か、その例で、同じ型の指摘が 5 回続いた。経路ごとに保証する所を書き足すのをやめ、入力に現実的な上限を掛けて範囲に届く筋を消す形に変えた（owner の決定、repo 外の回答台帳 TD-213。上限の数字は TD-217。介入の上限 12 → 14 は TD-214）。是正の前に相談役（Fable 5.1）に反例探しを頼み、その結果（円の合計は上限からは言えないので保存の前に 1 回検査する、売価・原価の上限は 6 桁にする、集計の読取りの拒否はやめる、移行の前に保存済みの値を確かめる）を採った。起草役が `0e4ab7bb` と `66c36019` で直した（設計正本 10・22・29・30・31・34・36・40・56・57・73、decision-log の D-113、packet の Owner Effort Budget・Scope の表・申し送りの表・Boundary / Wire Contract・Impact Review Lenses・Contract Ledger・Registration、Matrix）。Goal・Non-scope・AC・Risk・Human Gate は変えない。Plan contract が変わったので、本 Amendment を含む head で fresh broad を 2 本取り直す。

## Owner Effort Budget

- 介入回数上限: 14（2026-10-11 に 12 から上げた、owner 承認〈repo 外の回答台帳 TD-214〉。理由: Final Review の 4 回目の指摘の収め方を owner に諮ったため）。前の改定: 12（2026-10-08 に 10 から上げた、owner 承認〈repo 外の回答台帳 TD-211〉。理由: Plan Review が round 天井に達し、plan-approved を owner に諮るため）。その前の改定: 10（既定 6 から上げる。理由: 製品の振舞いの判断が 6 点〈J1〜J4、Plan Review round 1 の TD-203・TD-206/207〉と R4 の承認がある。2026-10-08 に 8 から 10 へ上げた〈owner の承認、repo 外の回答台帳 TD-208〉。1 回の問い合わせにまとめても decision point の数で数える）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 14 | 11（2026-10-08 の 9: 判断 J1〜J4 の 4〈TD-195〉、表示の範囲 TD-203 の 1、集計 TD-206/207 の 1、上限の改定 TD-208 の 1、round 3 の disposition の後の plan-approved の承認 TD-210 の 1、上限の改定 TD-211 の 1。2026-10-11 の 2: Final Review の 4 回目の指摘の収め方〈入力に現実的な上限を掛ける〉の判断 TD-213 の 1、上限の改定 TD-214 の 1） | 3（R4 の承認 1、Ready 1、merge 1） | 0 | 14 = 11 + 3 + 0 |

既定値・数え方・上限に届くときの扱いは `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R4

Reason:
本 lane は docs だけだが、決める契約は DB の CHECK を変えるための products の表の作り直し（migration vU）と、原価の 6 列を改名して既存の値を 100 倍する変換（migration vC）、Rust の enum・generated bindings・商品 CSV の検査・数量の wire の解釈を含む。表の作り直しと値の変換は Risk Tiers の「Destructive data lifecycle」に当たり、後続の runtime の lane は R4 になる。その rollback / recovery の契約（22 §16 の「回復」）を本 lane で決めるので、設計の段から R4 の審査（Final Review Minimum 2、Human Gate `r4` = owner が migration と回復の契約を承認する）にする（Coordinator の発注「R4 見込み」どおり）。実店舗データの露出は無い（店の事実は repo 外の回答台帳の番号と要旨だけ）。

required gate の green / red は変わらない（docs だけ。function-design の新しい file を作らず、`> 対応仕様:` の行を変えないので traceability の入力も変わらない。AC5）。Scope の予定 path を `scripts/ci/classify-changes.sh` の workflow の一覧（AGENTS・CLAUDE・DEV_WORKFLOW・MANUAL・code_review・project-profile・agent-guidance・templates・`.agents`・`.claude/{rules,commands,skills}`・PR template）に当てると当たらない。Final Review Minimum 2 は R4 による。

## Goal

Goal Invariant:

### 最小完了条件

- 後続の runtime の lane の Writer が、チャットの履歴を見ずに設計正本だけで「12 個の単位で商品を登録する → 長さの商品を m で入庫・販売・廃棄・棚卸しする → 在庫は cm の整数、数量を出す画面はすべて m・単価は 1 m あたり（売上・在庫変動・棚卸しの確定の結果を含む。TD-203） → 入庫・廃棄・棚卸しの金額と手動販売の金額の初期値が 1 m あたりの価格で正しく出る → 1 個の原価を小数 2 桁で持つ」を実装できる（[共通規則](../../function-design/10-common-rules.md) SPEC-UNIT-D1〜D11、22 §16、UI-01b-D22、UI-04-D18、BIZ-01-D8）。
- Z004・EJ の数量の型（小数 2 桁までの 100 倍の整数、BIZ で商品を引いた後に換算）が決まり、後続の Z004・EJ の lane へ申し送られている（SPEC-UNIT-D8）。
- 「長さ商品で 100 倍になる」既知の不整合 (1)(2)(4)(5) の直し方が正本にあり、(3) の扱いが申し送られている。owner の判断 J1〜J4 が決まり（2026-10-08、TD-195）、正本に決定として書かれている。

### 失敗定義

- runtime の Writer が、単位の code・数量の種類・基準数量・m の入力の規則・丸め・migration の手順と回復・型を変える所のどれかを推測しないと書けない。
- 設計が、在庫の数量を浮動小数で扱う、既存の `cm` の商品の値や意味を変える（商品 CSV の上書きで単位を変える経路を含む）、在庫少の判定から新しい単位を漏らす経路を残す。
- 長さの商品の数量を cm の整数のまま出す画面が残る（日次売上の `130`・`¥7` 等。TD-203）。
- 丸めの規則が場所ごとに違い、同じ明細から入庫・廃棄・棚卸しで違う金額が出る。

### 非目的

- runtime の code・test・fixture・bindings・migration・`90-traceability.md`（後続の lane）。
- Z004 の取込みの再開と EJ の取込みの配線（CASIO 固有、④ 以降）。数量の型の契約だけを決める。
- 売価の小数、重さの単位、箱と、ばらした個数を別々に数える model、巻と m の換算、商品の単位の登録後の変更、在庫少の基準の画面の語と m 表示、在庫計数 Excel から作る CSV の税抜 → 税込の換算（初期投入の作業）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

設計を含む変更なので、店の通常の操作列を置く。本 lane は docs だけで、表の通常運用は後続の単位の lane（行 1〜5・8・9）と原価の lane（行 7）の後に成り立つ。行 6（棚卸し）はそれに加えて棚卸しの再開（⑤、35 §20.0 SPEC-STOP-D6）の後に成り立つ（今の build では棚卸しの開始・保存・確定が停止中〈SPEC-STOP-D1、`stocktake_service.rs:282`〜`:290`〉で、確定の新方式は ⑤ が作る）。**この文書を完了できる**（設計正本がそろう）ことと、**通常運用を達成できる**（店で 12 単位と m で扱える）ことは別で、後者は本 lane では未達。それまで店は今の `個` / `cm` の 2 つで登録する（長さの商品の入庫・廃棄・手動販売の金額は 100 倍に出るので、長さの商品の登録は単位の lane を待つのが安全）。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
|---|---|---|---|---|
| 商品登録画面 | 反物を単位 `m`、売価 700・原価 400（どちらも 1 m あたりの label が付く）、初期在庫 `25` で登録する | 在庫 2,500 cm で保存され、在庫照会で `25 m` | 保存できる | UI-01b-D22、SPEC-UNIT-D3 |
| 入庫画面 | 同じ反物を数量 `2.5`（m）・原価 333 で入庫する。保存の後の原価差分のダイアログ（マスタ原価 400 と実原価 333）は「見送って閉じる」を選ぶ | 在庫 2,750 cm（`27.5 m`）。マスタ原価は 400 のまま。入庫記録詳細の原価小計 `832.50 円`、合計 `833 円`（J3 の決定） | 保存できる | J3（決定）、SPEC-UNIT-D6、UI-02-D15（「マスタ原価をこの実原価に更新する」を選ぶとマスタ原価が 333 になり、下の棚卸しの評価額は 333 × 2,615 ÷ 100 = 8,707.95 → 8,708 円） |
| 手動販売画面（反物は JAN が無く PLU にしない） | 反物を追加して数量を `1.3` にする | 数量 1.3 m、金額の初期値 `910`（700 × 130 ÷ 100）。金額を直さず保存すると在庫 2,620 cm | 保存できる | J4（決定: 割り切れない値は四捨五入）、UI-04-D18 |
| 日次売上画面（販売日） | 上の手動販売の日を開く | 反物の行が数量 `1.3 m`・単価 `¥700/m`・金額 `¥910`。部門小計・合計・販売点数は点と m の 2 本で、その日の売上が反物だけなら `1.3 m`、ほかに個数の商品 3 個と 2 枚があれば `5 点・1.3 m`（長さは点に入れない） | 画面が出る | TD-203・TD-206/207、SPEC-UNIT-D3 の表示する所と集計の規則 |
| 廃棄画面 | 反物を `0.5` m、原価 333 で廃棄する | 在庫 2,570 cm（`25.7 m`）。入力画面の合計 `167 円`（166.5 の四捨五入）、記録詳細の行 `166.50 円` | 保存できる | TD-023 に合わせた案。runtime の L3 で owner が確かめる |
| 棚卸し画面 | 反物の実数を `26.15`（m）と入れる | 2,615 cm を保存。確定の結果の一覧はシステム在庫 `25.7 m`（上の廃棄の後の 2,570 cm）・実際の数 `26.15 m`・差異 `-0.45 m`（今の符号: システム在庫 − 実数）。評価額は 400 × 2,615 ÷ 100 = 10,460 円（マスタ原価 400、入庫の行のとおり見送った場合） | 確定できる | SPEC-STK-VAL（値は今と同じ規則）、TD-203。棚卸しの再開（⑤）の後。確定の新方式は ⑤ が `biz::unit_amount` と `stock_unit` で作る |
| 原価の lane の後: 袋で仕入れてばらす商品（単位 `pcs`） | 1 袋 1,000 円・12 個入りを、原価 `83.33`・数量 36 で入庫する | 原価小計 `2,999.88 円`、合計 `3,000 円`（伝票の 3 袋 3,000 円と合う） | 保存できる | J2・J3、TD-058 |
| 商品 CSV の取込み | `在庫単位` の列に `玉` や `m` を書いた CSV を読む（`m` の行は `初期在庫` に `25`） | 取り込んだ後の在庫照会で `m` の商品の在庫が `25 m`（2,500 cm で保存）、毛糸の商品の単位が `玉`。`kg` の行と、既存の商品と違う単位で上書きする行は preview のエラー行（取込み全体は止まらない。今の preview の表は `在庫単位`・`初期在庫` の列を出さないので、値は在庫照会で見る） | 取り込める | SPEC-UNIT-D10、J1、BIZ-01-D8（`初期在庫` は Coordinator の決定、L-214） |
| 在庫照会の在庫少 | 単位 `ball` の毛糸が一般の基準以下になる | 在庫少の一覧に出る | — | SPEC-UNIT-D10（今の SQL では `pcs` 以外の個数の単位が漏れる） |

Plan Review は、この列が「正常な条件で目的を達成できるか」と「危険な結果を出さないか」を別々に答える（`docs/DEV_WORKFLOW.md` Review Rules）。

## owner の判断事項

**決定済み（2026-10-08、repo 外の回答台帳 TD-195）**: owner が J1〜J4 をすべて推奨の案 (a) に決めた。下の表の選択肢・推奨は起票時の経緯として残す。正本への反映は `決定（owner、D-113 Jn）` の記述と D-113 の「owner の決定」。

確認済み事実（repo 外の回答台帳の番号と要旨）: 店は 12 種の数え方をすべて使う（L-215）。長さの商品は最小 10 cm、残り・仕入れ・値札は m、値札は 1 m あたり（L-214）。レジは数量に小数 1 桁を打て、数量 1 = 1 m（`project-memory.md`、2026-09-15）。PLU の商品を小数で売ると Z004・EJ・精算レシートとも小数 1 桁（TD-147）。袋で仕入れてばらす商品の 1 個の原価は小数第 2 位まで（TD-058）。棚卸しの評価額は商品別に小数第 3 位で四捨五入して第 2 位まで、最終合計で四捨五入（TD-023）。メーカーは小数点以下の切り捨てが多く、店の切り売りの端数は切り上げ（L-135）。レジは四捨五入（L-136）。毛糸の Excel の「個」は owner の入力の誤りで、数え方は「玉」（TD-192）。

| # | 判断事項 | 選択肢 | 推奨（起票時の区分） | 決定 | 根拠 |
|---|---|---|---|---|---|
| J1 | 単位の一覧に `玉`（`ball`）を入れるか | (a) 入れる（12 種） (b) 入れない（11 種、毛糸は `個`） | (a)（confirmed: 店の答え L-215・FC-072 に直接依る） | (a) 入れる（12 種）。2026-10-08、TD-195 | 店は 12 種すべてを使い、毛糸は玉で数える。TD-192 は Excel の入力の誤りの話で、外す理由にならない。code が 1 つ増えるだけで計算は `個` と同じ |
| J2 | 原価を 1/100 円で持つか | (a) 持つ（6 列を `_centi` へ改名・100 倍、原価の runtime の lane を単位の lane の後に置く） (b) 円の整数のまま（83.33 は 83 で入れる） | (a)（candidate: 店の持ち方 TD-058 に合う。runtime の範囲が広い〈申し送りの表の原価の行〉ことを受け入れるかは owner） | (a) 持つ（原価の runtime の lane の範囲を受け入れた）。2026-10-08、TD-195 | (b) では 1,000 個で 330 円の評価額の差。(a) は Rust 28 file・TS 48 file（bindings を含む）の原価の参照を変える（2026-10-07、`rg -c` の file の数。下の申し送り） |
| J3 | 入庫の原価小計・合計の丸め | (a) TD-023 と同じ（行を 1/100 円で四捨五入、合計を円で四捨五入） (b) 行ごとに円未満を切り捨て（メーカーの伝票、L-135） | (a)（candidate: 規則が 1 つにまとまり、ばらした商品の伝票と合う。長さの端数の伝票とは 1 円ずれうる） | (a) TD-023 と同じ。2026-10-08、TD-195 | ばらした商品 83.33 円 × 36 = 2,999.88 → (a) 3,000 円・(b) 2,999 円（伝票は 3,000 円）。長さ 2.5 m × 333 円 = 832.5 → (a) 833 円・(b) 832 円（伝票は切り捨てが多い）。長さの仕入れは m の整数が多く端数は稀 |
| J4 | 手動販売の金額の初期値の丸め | (a) 四捨五入（レジ、L-136） (b) 切り上げ（店の切り売りの端数、L-135） | (a)（candidate: 手動販売はレジで打った売上の在庫の付け替えで、レジに数量 × 単価で打てば四捨五入になる。初期値は利用者が直せる） | (a) 四捨五入。2026-10-08、TD-195 | L-135 は価格を決めるときの端数の話として `project-memory.md` が記録している。今の店は分類キーに計算機の金額を打つので、どちらに合わせるかは owner |

廃棄のロス原価の丸めは店の直接の答えが無いので TD-023 に合わせる（判断事項に数えず、runtime の lane の L3 で owner が確かめる）。

**Plan Review round 1 の後の決定（2026-10-08）**:

- owner の決定（repo 外の回答台帳 TD-203）: 長さの商品は、数量を出す画面をすべて m で出し、単価は 1 m あたりで出す（日次・月次売上、在庫変動、棚卸しの確定の結果、入出庫・返品・廃棄の記録、在庫照会ほか）。在庫を cm の整数で持つ契約は変えない。正本は SPEC-UNIT-D3 の「表示する所」の表。
- owner の決定（repo 外の回答台帳 TD-206 と、それを精密にした TD-207）: 単位の違う商品をまたいだ点数・数量の集計（日次・月次売上の部門小計・合計・販売点数、売上の CSV の集計の行）は数量の種類ごとの 2 本に分ける。個数の種類の 10 単位は単位をまたいでそのまま足して `点` で出す（3 個 + 2 枚 = `5 点`）。長さの種類（`m`・`cm`）は点に入れず、cm の整数で足して m で出す（3 個 + 2 枚 + 1.3 m + 50 cm の日は `5 点・1.8 m`）。レジの数量で足す案（`4.3 点`）と単位ごとに 12 本に分ける案は採らない。レジの日報の点数との照合は日次売上の「レジ日報（公式）」の節（Z001 の表）が受け持つ。正本は SPEC-UNIT-D3 の集計の規則と 34 冒頭。wire は `count_points`・`length_cm` の 2 つの整数。
- Coordinator が owner の決定 TD-203・TD-207 から決めた（2026-10-08、Plan Review round 2 の指摘による）: 数量の表示は長さの種類（`m`・`cm`）ともに m に揃える（`cm` の商品の在庫 130 も `1.3 m`）。TD-203（長さの商品の数量は m）と TD-207（長さは m・cm とも m で出す）を延ばした解釈。入力の単位（`cm` の商品は cm の整数）と保存値（cm の整数）は変えない。正本は SPEC-UNIT-D3 の表示と 58 §58.6。owner の介入に数えない。
- Coordinator が店の事実から決めた（台帳 L-214: 残り・仕入れ・値札は m）: 商品 CSV で `在庫単位=m` の行の `初期在庫` は m の数として読む（`m,25` → 2,500 cm、`m,26.15` → 2,615 cm。小数 3 桁以上は行の error、10 cm 未満は D3 のとおり受ける）。正本は SPEC-UNIT-D3 の適用欄と BIZ-01-D8。owner の介入に数えない。

店主に新しく聞く必要がある事項: 無い（上の判断はどれも台帳の答えで足りる）。runtime の lane の L3 で owner が確かめる項目は申し送りの L3 の表。

## Scope

本 lane は docs だけを変える（予定 file の全部。所有の列は並走 lane との重なり）。

| file | 変更 | 他 lane との重なり |
|---|---|---|
| `docs/function-design/10-common-rules.md` | 「単位・数量・金額の共通規則」の節（SPEC-UNIT-D1〜D11、D3 の表示する所の表〈TD-203〉、「入力の上限と安全な整数の範囲」〈TD-213、Gated Amendment 5〉、`biz::unit_amount` のシグネチャ）、更新履歴 | なし |
| `docs/db-design/master-tables.md` | 冒頭に「単位と原価の精度の契約」の節（CHECK の 12 code、原価の 6 列の表）、価格の基準数量の設計意図の末尾に D-113 の pointer | なし |
| `docs/DB_DESIGN.md` | CHECK 制約方針の `products.stock_unit` の行と D-4 の適用ルールに proposed の注記 | なし |
| `docs/db-design/transaction-tables.md` | 冒頭に「原価の 1/100 円と金額の丸め」の節 | なし |
| `docs/function-design/22-mnt-migration.md` | §16（MNT-03-D13: vU・vC の手順、vU の上限の検査、回復の手順、テスト） | `agent/backup-offsite` が本 file の §3 の MNT-03-D11 の「復元への波及」の段落を変える（2026-10-08、`git diff --name-only 95c0aeb0 agent/backup-offsite`）。本 lane の所有は §16 だけ |
| `docs/function-design/31-biz-inventory-service.md` | 冒頭に「単位と原価の精度の契約」の節 | なし |
| `docs/function-design/21-io-inventory-repo.md` | §10.2・§10.5 の手順 3 に proposed の注記 | なし |
| `docs/function-design/35-biz-stocktake-service.md` | §20.5a の末尾に「単位の拡張の後」の段落だけ | `agent/stocktake-p1-3` が本 file の別の節（冒頭の時点証拠の付近・§20.1・§20.4 の付近）を変える。本 lane の所有は §20.5a の末尾の段落だけ |
| `docs/function-design/62-ui-manual-sale.md` | UI-04-D18 の行、§62.8 の非目的の 1 行、§62.9 の 1 行、更新履歴 | なし |
| `docs/function-design/51-ui-product-form.md` | UI-01b-D22 の行、§7.6 の 1 行、更新履歴 | なし |
| `docs/function-design/58-ui-stock-inquiry.md` | §58.6 に「単位の拡張の後」、更新履歴 | なし |
| `docs/function-design/30-biz-product-service.md` | 冒頭に「単位と原価の精度の契約」（BIZ-01-D8） | なし |
| `docs/function-design/23-io-z004-parser.md` | §13.4.1 の却下案の後に proposed の段落 | なし |
| `docs/function-design/29-io-ej-parser.md` | IO-08.5 の小数の数量の項の後に proposed の項。暗黙の数量 ±1 を書く 4 行に proposed の括弧書き（Gated Amendment 5） | なし |
| `docs/function-design/77-ui-bulk-price-revision.md` | SPEC-PRV-D4 の新原価（案）の項に proposed の注記（例の値を含む） | なし |
| `docs/function-design/34-biz-sales-service.md` | 冒頭に「単位の拡張の後の数量」（売上の wire の単位、点と m の 2 本の集計、CSV の数量）、更新履歴（TD-203・TD-206/207、Plan Review round 1 で追加） | なし |
| `docs/SCREEN_DESIGN.md`（生地の cm の表示の 4 行） | 末尾に proposed の括弧書き（Final Review の Gated Amendment 3 で追加） | なし |
| `docs/function-design/66-ui-stock-movements.md`（UI-06c-D5）・`67-ui-plu-export.md`（`PluDirtyProductTable`）・`75-ui-integrity-check.md`（UI-13-D3）・`53-ui-home.md`（昨日売上カードの点数）・`74-ui-operation-logs.md`（UI-11c-D14）・`36-biz-integrity-check.md`（`StockAdjustment` の後） | 該当の行に SPEC-UNIT-D3 の「表示する所」の表への proposed の pointer を 1 行ずつ（Final Review broad で追加） | `agent/backup-offsite` が 53 の別の節（:35・:207〜:214・:313 付近）を変える。本 lane の所有は昨日売上カードの行の括弧書きだけ |
| `docs/function-design/56-ui-daily-sales.md`・`57-ui-monthly-sales.md` | §56.10 の単価派生・§57.6 compute-summary に proposed の 1 項、更新履歴（TD-203、Plan Review round 1 で追加）。今の型を書く 3 行（56 の `DailySaleItem` の field と `GrandTotal`、57 の `MonthlySaleItem.quantity`）に proposed の括弧書き（Gated Amendment 5） | なし |
| `docs/function-design/44-cmd-inventory.md` | `list_low_stock` の SQL の後に proposed の注記 | なし |
| `docs/function-design/40-cmd-product.md`・`20-io-product-repo.md` | wire の DTO の定義の後に proposed の注記（Final Review の fresh broad の正本だけの是正 `fe662ef6` で足した。40 の原価・売価の上限は Gated Amendment 5 で合わせた） | `agent/stocktake-p1-3` が 20 の別の節を変える。本 lane の所有は proposed の注記だけ |
| `docs/function-design/61-ui-receiving.md`・`63-ui-return-exchange.md`・`64-ui-disposal.md` | 数量・原価の検証の行（UI-02-D7・UI-03-D12・UI-05-D8）の決定の末尾に proposed の括弧書き | なし |
| `docs/function-design/73-ui-stocktake.md` | 冒頭の数量の検査の文に proposed の括弧書き（`StocktakeItemDetail.stock_unit`・`AdjustedItem.stock_unit`、一覧と確定の結果の m の表示） | `agent/stocktake-p1-3` は起票時点で本 file を触っていない。本 lane の所有はその括弧書きだけ |
| `docs/project-memory.md` | Store Premises Facts に 2 行（TD-147 の小数 1 桁、TD-192 の毛糸の「個」） | なし |
| `docs/decision-log.md` | 末尾に D-113 だけを追記 | 全 lane が末尾に追記。merge 順で両方を残す |
| `docs/backlog.md` | 「単位の拡張」と「原価 × 数量 … 残り」の entry に設計の sub-bullet を 1 つずつ | 全 lane が自 lane の entry だけ |
| `docs/plans/2026-10-07-unit-extension.md`、`docs/plans/test-matrices/2026-10-07-unit-extension.md` | 新設 | なし |

触らない: `docs/Plans.md`（D-097）、`docs/function-design/90-traceability.md`（生成物）、`docs/spec/requirements.md`、`docs/db-design/tracking-system-tables.md`（`agent/stocktake-p1-3` が変更中。`stocktake_items.valuation_cost_price` と `price_history` の列の改名は master-tables の表が持ち、tracking-system-tables の列の表は原価の runtime の lane が直す）、`docs/function-design/71-mnt-backup.md`（`agent/backup-offsite` が変更中。22 §16 は link だけ）、`docs/function-design/69-ui-threshold-settings.md`（基準の画面は変えない）、`docs/function-design/25-io-plu-formatter.md`（PLU の単価は変えない、SPEC-UNIT-D4）。

### runtime の lane への申し送り（形が変わる型と、作る所・読む所）

本 lane は code を書かない。2026-10-07 に main `95c0aeb0` の現物で `rg` して数えた（行番号は定義・構築・呼出しの行）。runtime の lane は起票時の現物で数え直し、所有表を書く。lane の割当: **U** = 単位の lane（先）、**C** = 原価の lane（後）、**P** = Z004 の取込みの再開・EJ の取込みの lane（CASIO 固有）。

J2 が (a) に決まった（2026-10-08、TD-195）ので、下の表の lane C の行（原価の 6 列と migration vC、原価の Rust の型・wire・frontend、操作ログの原価の key）が原価の runtime の lane の範囲として確定した。

| 型・関数・場所 | 変更 | lane | 作る所 | 読む所・test |
|---|---|---|---|---|
| `ProductStockUnit`（12 variant） | SPEC-UNIT-D1 | U | `src-tauri/src/db/product_repo.rs:35`（定義）・`:41`〜`:46`（`as_str`）・`:62`（`parse_stock_unit`） | `biz/product_service.rs:32`（create の request）・`:248`（`as_str`）、`db/product_repo.rs:89`（Product）・`:305`（行の読取り）、`biz/stocktake_service.rs:441`（`price_basis_quantity`、`biz::unit_amount` へ移す）。test: `src-tauri/src/finite_enum_contract_tests.rs:63`（wire の対の表）・`:134`（`rejects!`）・`:188`〜`:189`（`parity!`。12 個に広げる）、`biz/stocktake_service.rs:1402`〜`:1403`。bindings `src/lib/bindings.ts:1272`（再生成） |
| products の CHECK と migration vU | SPEC-UNIT-D1、22 §16 | U | `db/migration.rs`（登録）、新しい `db/schema_vN.rs`。手順 2 に上限の検査（保存済みの数量・在庫・変動・売価・原価・円の合計。22 §16 の「vU の上限の検査」の表の列で全部）を入れ、外れた DB は何も変えずに `MigrationFailed` | 既存の test `db/migration.rs:1378`〜`:1386`（`kg` を CHECK が拒む。そのまま通る）。`schema_v1.rs:40` は書き換えない（migration の履歴） |
| wire の `stock_unit: String` → `ProductStockUnit`（6 種） | SPEC-UNIT-D11 | U | `db/receiving_repo.rs:52`（ReceivingRecordDetailItem）・`:243`、`db/disposal_repo.rs:111`・`:661`、`db/return_repo.rs:110`・`:294`、`db/manual_sale_repo.rs:68`・`:180`、`db/sales_repo.rs:134`（CsvImportRecordDetailItem）・`:512`、`db/stocktake_repo.rs:116`（StocktakeRecordDetailItem）・`:613` | bindings の 6 型（`src/lib/bindings.ts:470`・`:668`・`:915`・`:1365`・`:1437`・`:1635`）。`NewProduct.stock_unit: String`（`product_repo.rs:136`）と `ProductUpdates.stock_unit`（`:167`・`:1045`）は内部の型で、enum にするかは U が決める |
| `StocktakeItemDetail` に `stock_unit` を足す | SPEC-UNIT-D11、73 | U | `db/stocktake_repo.rs:64`（定義）と、その行を読む SELECT | `src/lib/bindings.ts:1584`、`src/features/stocktake/StocktakePage.tsx:484`（実数の入力欄）。`agent/stocktake-p1-3` と file が重なるので U の起票時に所有表を書く |
| 在庫少の SQL | SPEC-UNIT-D10、44 | U | `db/product_repo.rs:1258`〜`:1259` | `cmd/inventory_cmd.rs:207`（`LS-CM` の test）ほかの list_low_stock の test。単位の全 variant が一般か生地のどちらかに入る test を足す |
| `biz::unit_amount`（新 module） | SPEC-UNIT-D3・D6・D7・D8 | U（D8 の関数は P） | `src-tauri/src/biz/unit_amount.rs`、`biz/mod.rs`。`cost_line_centi` の原価の引数は `i128`（1/100 円）。今の `valuation_line_centi`（`stocktake_service.rs:453`、引数は円の `i64`）を置き換える。数量の種類ごとの集計は BIZ-05 が持ち、種類は `stock_unit_kind` で決める（下の売上の行） | 呼出し: `biz/stocktake_service.rs:561`〜`:566`（評価額。U の間は `i128::from(valuation_cost_price) * 100` を渡す）、BIZ-02 の記録詳細（§12.6a）、BIZ-05 の集計（下の売上の行）。今の 3 関数の test（`stocktake_service.rs:1402`〜`:1482`）は、原価の引数を `i128::from(円) * 100` にして期待を変えずに移す（`:1411`〜`:1412` の境界は 100 × 1 ÷ 200 → 1・÷ 201 → 0 になり、`:1457`・`:1461` の `i64::MAX` の円の原価と `:1475` の `92_233_720_368_547_759` の期待もそのまま）。golden の表は TS の twin と共有（SPEC-UNIT-D9） |
| 入庫・廃棄の行の金額（IO → BIZ） | SPEC-UNIT-D6、21・31 | U | IO の計算をやめる: `db/receiving_repo.rs:229`（`ri.quantity * ri.cost_price AS line_cost`）・`:250`（合計）、`db/disposal_repo.rs:647`・`:670` | wire `line_cost`・`line_loss_cost`（`receiving_repo.rs:55`・`disposal_repo.rs:116`）を 1/100 円の整数の十進の文字列 `line_cost_centi`・`line_loss_cost_centi` に（Rust は `String`、bindings は `string`。SPEC-UNIT-D6、31 の proposed 節）。TS の表示は文字列のまま桁を区切る（number に直さない）。test: `db/receiving_repo.rs:604`、`db/disposal_repo.rs:1094`・`:1098`。frontend: `src/features/inventory-records/ReceivingRecordDetailPage.tsx:164`、`DisposalRecordDetailPage.tsx:170` |
| 棚卸し記録詳細のロス原価 | SPEC-UNIT-D6（backlog (3)） | U か棚卸し ⑤（起票時に決める） | 今は画面が計算: `src/features/inventory-records/StocktakeRecordDetailPage.tsx:182`〜`:183` | BIZ が返す形にして画面の掛け算を消す |
| formatter | SPEC-UNIT-D3・D9、58 §58.6 | U | `src/features/stock-inquiry/lib/format-stock-display.ts:15`・`:34`（param を `ProductStockUnit` に、`default` を消す） | 呼出し 22 か所（2026-10-07、`rg -n "formatStock(Display|UnitLabel)\(" src --glob '!*.test.*'` から定義の 2 行を除いた数）: `DisposalPage.tsx:466`・`:521`・`:568`、`CsvImportRecordDetailPage.tsx:209`、`ManualSaleRecordDetailPage.tsx:174`、`StockMovementsPage.tsx:136`、`ReceivingRecordDetailPage.tsx:158`、`ReceivingPage.tsx:575`、`DisposalRecordDetailPage.tsx:164`、`StocktakeRecordDetailPage.tsx:46`・`:50`・`:193`（`:196`・`:199` は `:46`・`:50` の wrapper を呼ぶ）、`ReturnRecordDetailPage.tsx:208`、`ManualSalePage.tsx:558`・`:611`・`:635`、`ReturnExchangePage.tsx:782`・`:833`・`:883`、`StockDetailContent.tsx:82`、`ProductListTable.tsx:100`、`ProductTable.tsx:79`。test: `format-stock-display.test.ts`（18 行の参照） |
| 行の型の `stockUnit: string` | SPEC-UNIT-D11 | U | `src/features/disposal/types.ts:10`、`receiving/types.ts:6`、`return-exchange/types.ts:10`、`manual-sale/types.ts:9` | 構築: `disposal-row-utils.ts:48`、`receiving-row-utils.ts:13`、`return-exchange-row-utils.ts:31`、`manual-sale-row-utils.ts:20` |
| m の数量の入力（文字列 → cm） | SPEC-UNIT-D3 | U | 新しい TS の純関数（`src/lib/` に置く。`request-helpers.ts:15` の `parseRequiredSafeInteger` の隣） | 使う所: `receiving-request.ts:51`、`disposal-request.ts:38`・`:60`、`return-exchange-request.ts:74`、`manual-sale-request.ts:52`、`product-form-request.ts:86`（初期在庫）・`:98`（文）、棚卸しの実数（`StocktakePage.tsx:484` の周辺と `useUpdateCount.ts:14`）。今の文「数量は1以上の整数で入力してください」に当たる test（`ReceivingPage.test.tsx` 等 22 行）は個数・cm の商品で残る |
| 手動販売の数量と金額 | UI-04-D18、SPEC-UNIT-D7 | U | `src/features/manual-sale/lib/manual-sale-row-utils.ts:4`（`nextQuantity`）・`:10`（`nextAmount`）・`:24`（初期値の `String(product.selling_price)`）・`:40` | `ManualSalePage.tsx:620`〜`:654`（数量・金額の入力）。金額の編集の有無を row に持つ（`types.ts`）。test: `manual-sale-row-utils.test.ts`、`ManualSalePage.test.tsx` |
| 入力の行の数量の加算（再追加・統合） | SPEC-UNIT-D3 | U | 今は 4 つの helper が `Number.isInteger` で整数だけを足す（`1.3` の再追加は `"1"` に戻り、廃棄の `0.5` と `0.3` の統合は `"0.5"` のまま）: `src/features/receiving/lib/receiving-row-utils.ts:4`（`nextQuantity`、呼出し `:27`）、`return-exchange/lib/return-exchange-row-utils.ts:4`（`nextQuantity`、`:49`）・`:9`（`sumQuantities`、方向の切替え `:91`）、`disposal/lib/disposal-row-utils.ts:22`（`nextQuantity`、`:69`）・`:27`（`mergeQuantity`、`:94`）、`manual-sale/lib/manual-sale-row-utils.ts:4`（`nextQuantity`、`:39`）。m の入力の純関数で cm にして整数で足し、入力の文字列へ戻す 1 つの helper にまとめる | test: `receiving-row-utils.test.ts`、`return-exchange-row-utils.test.ts`、`manual-sale-row-utils.test.ts`（既存）。廃棄は helper の test を新設（今は `DisposalPage.test.tsx` だけ）。Matrix の SPEC-UNIT-D3 の加算の行 |
| 売上の wire と集計（TD-203・TD-206/207） | SPEC-UNIT-D3・D11、34 冒頭 | U | `biz/sales_service.rs:40`（`DailySaleItem` に `stock_unit`）・`:62`（`DeptSubtotal.quantity` → `count_points`・`length_cm`）・`:71`（`GrandTotal` も同じ）・`:141`（`MonthlySaleItem` も同じ。商品別の行はどちらか一方だけで、`stock_unit: Option<ProductStockUnit>` を足して種類を運ぶ）、構築 `:203`・`:224`〜`:245`（部門小計の `BTreeMap` の `i64` と総合計の `sum()` を種類ごとの checked の和に）・`:549`・`:561`、CSV `:337`・`:364`（商品の行は表示の単位の数）・`:388`（部門別の行は `5 点・1.8 m` の文字列）。repo の SELECT に `p.stock_unit` を足す: `db/sales_repo.rs:997`（日次）・`:1340`（月次の商品別）・`:1373`（月次の部門別。部門と単位で GROUP BY して BIZ で種類ごとに足すか、`SUM(CASE …)` の並びを全 variant から作る） | test: `sales_service.rs:707`・`:737`・`:801`（`grand_total.quantity` → `count_points`。個数の商品だけなら値は今と同じ、`length_cm` 0）・`:760`・`:766`（部門小計、同じ）・`:976`（明細の `quantity`、そのまま）。期待を型に合わせて書き換えるだけで弱めない。長さと個数の商品が混ざる日の test を足す（Matrix） |
| 売上の画面（TD-203） | SPEC-UNIT-D3・D9、56・57 | U | 日次: `src/features/daily-sales/components/ProductTable.tsx:90`・`:110`・`:136`、`lib/calculate-unit-price.ts:14`〜`:16`、`lib/group-items.ts:30`・`:39`（部門小計を点と m の 2 本に足す TS の twin）、`types.ts:51`（`subtotal: DeptSubtotal`）、`lib/sort-items.ts:53`（`(count_points, length_cm)` の順）、`components/SummaryCardsBar.tsx:54`、`DailySalesPage.tsx:129`、`src/features/home/components/SummaryCards.tsx:43`。月次: `monthly-sales/components/ProductRankingTable.tsx:89`、`lib/compute-summary.ts:16`（2 本それぞれに足す）、`components/SummaryCardsBar.tsx:50`、`lib/sort-items.ts:62`、`lib/pick-top-ranking.ts:26`、`types.ts:95`（`ProductRankingRow.quantity`）。`count_points`・`length_cm` を `5 点・1.8 m` にする formatter を `format-stock-display.ts` の隣に足す | fixture: `daily-sales/lib/test-fixtures.ts:14`、`monthly-sales/lib/test-fixtures.ts:15`・`:28`、`monthly-sales/components/SummaryCardsBar.test.tsx:19`、`grand_total` を作る所 `daily-sales/components/SummaryCardsBar.test.tsx:18`・`:26`・`:168`〜`:169`・`:188`〜`:189`・`:206`、`daily-sales/DailySalesPage.test.tsx:38`・`:86`、`daily-sales/components/ProductTable.test.tsx:134`、`home/HomePage.test.tsx:66`、`home/hooks/useHomeSummary.test.tsx:45`、`home/components/SummaryCards.test.tsx:164`・`:218`、`backup-restore/BackupRestorePage.flow.test.tsx:110`。test: `calculate-unit-price.test.ts`、`group-items.test.ts:23`、`ProductTable.test.tsx`、`compute-summary.test.ts`、`sort-items.test.ts`、`SummaryCards.test.tsx`。公式日報の数量（`DailySalesPage.tsx:233`〜`:285`、`MonthlySalesPage.tsx:194`）は D-104 のまま変えない |
| 在庫変動の表（TD-203） | SPEC-UNIT-D3 | U | `src/features/stock-movements/components/MovementTable.tsx:25`（props に必須の `stockUnitByProductCode: ReadonlyMap<string, ProductStockUnit>` を足す）・`:62`・`:74`・`:79`（数量・変動後の在庫を行の商品の単位で出す）、`lib/movement-formatters.ts:20`（`formatMovementQuantity` に単位を渡す）。呼ぶ所 7 つ（`rg -n 'MovementTable' src --glob '!*.test.*'`、2026-10-08）: `StockMovementsPage.tsx:261`（`:137` が読む商品 1 つから対応を作る）、記録詳細 6 つは同じ記録の `detail.items` から作る: `inventory-records/ReceivingRecordDetailPage.tsx:190`・`ReturnRecordDetailPage.tsx:234`・`ManualSaleRecordDetailPage.tsx:201`・`DisposalRecordDetailPage.tsx:197`・`CsvImportRecordDetailPage.tsx:286`・`StocktakeRecordDetailPage.tsx:233` | test: `MovementTable.test.tsx:14`（fixture）・`:26`、記録詳細 6 画面の test（`ReceivingRecordDetailPage.test.tsx` ほか、`OtherRecordDetailPages.test.tsx`）に m と個数の混ざった記録を 1 つ。対応を渡さない呼出しは必須の prop なので TS の型検査で止まる |
| 棚卸しの一覧・確定の結果（TD-203） | SPEC-UNIT-D3・D11、73、35 §20.5a | U（型と画面）・⑤（新方式の確定） | `biz/stocktake_service.rs:59`（`AdjustedItem` に `stock_unit`）。`:587`（構築）と評価額の呼出し `:561`〜`:566` は旧本体 `legacy_complete_stocktake`（`#[cfg(test)]` の test だけが呼ぶ）の中で、U は旧本体の test を通すために直すだけ。新方式の確定の `AdjustedItem` に当たる型と評価額の呼出しは、棚卸しの再開（⑤、SPEC-STOP-D6）が `biz::unit_amount` と `stock_unit` で作り、補正の note（今は `:582`）を D3 の形で書く。画面: `src/features/stocktake/StocktakePage.tsx:675`（入力欄の現在在庫）・`:885`〜`:886`・`:898`（一覧の現在在庫・実数・差異）・`:1035`〜`:1046`（確定の結果）、`lib/stocktake-formatters.ts:21`（`formatListDifference` に単位を渡す） | `StocktakeItemDetail` の行（上）と同じ所有表で `agent/stocktake-p1-3` と調整する |
| 整合性チェック（TD-203） | SPEC-UNIT-D3・D11 | U | `biz/integrity_service.rs:26`（`IntegrityMismatch` に `stock_unit`）・`:79`（構築）・`:44`（`StockAdjustment` に `stock_unit`）・`:175`（構築）・`:191`〜`:192`（`integrity_fix` の detail_json の各 adjustment に `stock_unit` を足す）、`db/product_repo.rs:1149`（`find_all_stock_quantities` に単位を足す。呼出しは `integrity_service.rs:67` だけ）。画面: `src/features/integrity-check/IntegrityCheckPage.tsx:368`・`:385`・`:460`（不一致の一覧）・`:296`（補正の結果 `old_stock → new_stock`）、`src/features/operation-logs/OperationLogsPage.tsx:95`〜`:120`（`integrity_fix` の detail の parse に任意の `stock_unit` を足す）・`:182`〜`:187`（`旧在庫 … → 新在庫 …` と補正の量。`stock_unit` の key が無い記録済みの log は今どおり整数） | fixture: `IntegrityCheckPage.test.tsx:22`、`OperationLogsPage.test.tsx`（`old_stock` の fixture） |
| 在庫警告の文（TD-203） | SPEC-UNIT-D3・D9、31 | U | `biz/inventory_service/receiving.rs:206`〜`:211`・`returns.rs:261`〜`:266`・`manual_sale.rs:277`〜`:282`・`disposal.rs:198`〜`:203`（`{}: 在庫がマイナスになりました（{}）` の `stock_after` を `biz::unit_amount::format_length_m` で整形。長さの商品だけ、個数の商品の文は今どおり） | 画面は文をそのまま出す（`ManualSalePage.tsx:375`、`DisposalPage.tsx:330`、`ReceivingPage.tsx:336` 付近）。test: 4 つの BIZ の在庫警告の test（`m` の商品で `（-0.3 m）`、個数の商品は今どおり） |
| PLU 書出しの未書出しの一覧（TD-203） | SPEC-UNIT-D3・D11 | U | `cmd/plu_export_cmd.rs:62`（`ProductResponse` に `stock_unit`）・`:192`（構築）。画面: `src/features/plu-export/PluExportPage.tsx:676` | fixture: `backup-restore/BackupRestorePage.flow.test.tsx:141`、`home/HomePage.test.tsx:97`、`home/hooks/useHomeSummary.test.tsx:76` |
| 廃棄の入力画面の合計 | SPEC-UNIT-D6・D9 | U | `src/features/disposal/lib/disposal-request.ts:40`（`quantity * costPrice`） | TS の twin で行を 1/100 円、合計を円で丸める |
| 商品フォーム | UI-01b-D22 | U | `src/features/products/components/StockUnitField.tsx:46`・`:59`〜`:70`（select の 2 項目）・`:88`（cm の提案の文）、`ProductForm.tsx:349`（価格の節の説明）・`:352`・`:364`（label）、`product-form-request.ts:44`・`:70`（`product.stock_unit === "cm" ? "cm" : "pcs"` の 2 値の写像）・`:126` | test: `StockUnitField.test.tsx`（41 行）、`ProductForm.test.tsx`（11 行）、`product-form-request.test.ts` |
| 商品 CSV の `在庫単位`・`初期在庫`・上書き | SPEC-UNIT-D2・D3・D10、BIZ-01-D8 | U | `biz/product_service.rs:1041`（ImportRow）・`:1199`（`在庫単位`）・`:1204`〜`:1213`（`初期在庫` の `parse::<i64>`。`m` の行は m の規則に）・`:1307`（duplicate_rows。既存の商品と単位が違えば行の error）・`:1377`（上書きの `stock_unit: row.stock_unit.clone()` → 常に `None`、TX の中で既存の単位と比べて違えば `ValidationFailed`）・`:1402`（空 → `pcs`） | test: `product_service.rs:3294`（空 → `pcs`、そのまま）・`:3324`（`kg` で取込み全体が rollback → preview の行の error に期待を変える。契約の変更による書換えで、弱めない）・`:3342`（上書き）・`:3372`〜`:3394`（上書きで任意の列を保つ。単位の空は今の単位のまま）。足す test は Matrix の BIZ-01-D8 の行 |
| demo の seed | SPEC-UNIT-D1・D7、SPEC-UNIT-D5 | U（`:469` は C の後も同じ）・C（`:232`） | `src-tauri/src/seed_demo.rs:46`・`:237`・`:246`（`"cm"` の分岐）。`:469`（`selling_price * quantity`。長さの商品で 100 倍になるので `sale_amount_yen` で求める）。`:232`（`cost_price` を円で作る。C で 1/100 円にする） | `tests/seed_test.rs` |
| 入力の上限（10 の「入力の上限と安全な整数の範囲」の (a)） | SPEC-UNIT-D11 の続き、30・31 | U（原価の上限の 1/100 円への書換えは C、POS の換算の後は P） | 上限の 4 つの値を定数で 1 か所に（`src-tauri/src/constants.rs`、TS は `src/lib/`）。1 明細の数量: `biz/inventory_service/receiving.rs:112`・`returns.rs:89`（`validate_return_request_shape`。在庫を動かさない明細も通る）・`manual_sale.rs:131`・`disposal.rs:109`、商品別売上 CSV の preview `biz/csv_import_service/parse.rs:92`〜`:134`（越える行は `invalid_number` の行の error）。在庫: `biz/inventory_service/common.rs:54`〜`:58`（`apply_stock_change` の計算の後）、`biz/product_service.rs:957`（create の初期在庫）・`:1204`〜`:1213`（CSV の初期在庫、preview と commit）、`biz/csv_import_service/commit.rs:257`〜`:274`（取消の補正。和と補正後の在庫を checked で求めて検査）、`biz/integrity_service.rs:173`（整合性の補正の後）。売価: `biz/product_service.rs:947`・`:994`・`:471`・`:1178`〜`:1183`。原価（U は円の整数で `999999`。C が `99999999`〈1/100 円〉へ書き換える）: `:952`・`:1001`・`:476`・`:1185`〜`:1190`、`receiving.rs:118`・`disposal.rs:115`。TS: 入庫・返品・手動販売・廃棄の数量と原価の欄（`src/lib/request-helpers.ts:15`〜`:20` の `parseRequiredSafeInteger`）、商品フォームの売価・原価・初期在庫の欄（`src/features/products/lib/product-form-request.ts:52`〜`:55` の `parseNonNegativeInteger`、`:84`〜`:86`）、m の入力の純関数に同じ上限 | Matrix の「SPEC-UNIT-D11 入力の上限」の 4 行。既存の `common.rs:138`〜`:298` の test は上限の外の値を使わないので期待は変わらない |
| 円の合計の保存の前の検査（同 (c)） | SPEC-UNIT-D11 の続き、31 | U（棚卸しの確定は ⑤） | `biz::unit_amount::check_safe_integer`（新設）。呼ぶ所は `create_receiving`（`biz/inventory_service/receiving.rs:53`）と `create_disposal`（`disposal.rs:62`）の、最初の書込みの前（明細から `cost_line_centi`・`cost_total_yen` で合計を求める。今の create は合計を求めない）。記録詳細の読取り（BIZ へ移す今の合計 `db/receiving_repo.rs:250`・`db/disposal_repo.rs:670`）には範囲の検査を置かない。TS: 廃棄の入力画面の合計の twin（`src/features/disposal/lib/disposal-request.ts:36`〜`:42`）を `BigInt` に。手動販売の金額の初期値の twin は number の整数の演算のまま | Matrix の「SPEC-UNIT-D11 円の合計の保存の前の検査」の行 |
| 集計と整合性の演算を checked に（同 (d)） | SPEC-UNIT-D3・D11 の続き、34・36 | U | 今は plain の演算で、release では溢れが wrap する: `biz/sales_service.rs:229`（`+=`）・`:244`（`sum()`）、`biz/integrity_service.rs:77`・`:165`・`:172`（`-`）。checked にし、溢れは `ValidationFailed`（範囲の契約ではなく溢れの扱い。読取りを範囲で検査して拒まない） | Matrix の月次の相殺の行（上限の端の集計が成功する）と「SPEC-UNIT-D11 入力の上限（在庫）」の行（整合性チェックは読取りを拒まない） |
| 棚卸しの実数の上限 | SPEC-UNIT-D11 の続き | U か ⑤ | 棚卸しの実数の上限は在庫の上限。単位の runtime の lane（U）か ⑤ が、#152 の merge の後に `src-tauri/src/biz/stocktake_service/time_evidence.rs` の N の範囲と文言、`time_evidence_tests.rs` の N = 9007199254740991 を成功とする行を書き換える。±9007199254740991 の検査（`safe`）は外側の守りとして残す | 並走 lane の棚卸しの新方式（PR #152）は、実数 N を 0 以上 9007199254740991 以下で受け、test が端の値の成功を固定している。本 lane は #152 の file を変えない |
| 原価の 6 列と migration vC | SPEC-UNIT-D5、22 §16 | C | 新しい `db/schema_vN.rs`。BIZ の原価の上限（U が円で入れた `999999`）を `99999999`〈1/100 円〉へ書き換える。手順 1 の範囲検査（`?1 = 999999`）は外側の守りで、vU を通った DB では通る | 時点証拠の migration（`db/schema_time_evidence.rs:64`・`:82`〜`:84`、未配線）が `valuation_cost_price` を列名で写す。後に入る方がその時点の列名で書く |
| 原価の Rust の型 | SPEC-UNIT-D5 | C | `db/product_repo.rs:85`・`:132`・`:163`・`:214`〜`:215`・`:224`〜`:225`、`db/receiving_repo.rs:31`・`:54`、`db/disposal_repo.rs:61`・`:114`、`db/stocktake_repo.rs:43`・`:120`、`biz/inventory_service/receiving.rs:28`・`:46`〜`:47`、`biz/inventory_service/disposal.rs:28`、`biz/product_service.rs:30`・`:57`・`:77`・`:1039`、`cmd/plu_export_cmd.rs:68` | test の参照（`rg -c "cost_price|old_cost|new_cost|valuation_cost" src-tauri/src src-tauri/tests` の 28 file。多い順に `biz/product_service.rs` 61、`db/product_repo.rs` 35、`db/disposal_repo.rs` 19、`biz/inventory_service/receiving.rs` 19、`db/stocktake_repo.rs` 17、`biz/stocktake_service.rs` 17、`db/receiving_repo.rs` 11） |
| 原価の wire（bindings の 14 型: CostDiff・DisposalItemInput・DisposalRecordDetailItem・ImportRow・PriceHistoryEntry・PriceRevisionInput・Product・ProductCreateRequest・ProductResponse・ProductUpdateRequest_Deserialize・ProductUpdateRequest_Serialize・ReceivingItemInput・ReceivingRecordDetailItem・StocktakeRecordDetailItem。2026-10-08、`awk '/^export type [A-Za-z0-9_]+ *=/{name=$3} /cost_price\|old_cost\|new_cost\|valuation_cost/{print name}' src/lib/bindings.ts \| sort -u \| wc -l` → `14`） | SPEC-UNIT-D5 | C | bindings の再生成 | `src/lib/bindings.ts:418`〜`:419`（CostDiff）・`:647`・`:671`・`:784`（ImportRow）・`:1153`〜`:1154`・`:1162`・`:1186`・`:1214`・`:1244`・`:1285`・`:1298`・`:1342`・`:1367`・`:1639` |
| 原価の frontend | SPEC-UNIT-D5・D9 | C | 入力の文字列 → 1/100 円の TS の純関数、表示の formatter（`src/features/inventory-records/types.ts:104` の `formatYen` の隣） | 非 test の参照: `DisposalPage.tsx:100`・`:468`・`:578`・`:587`、`disposal-request.ts:24`・`:39`・`:61`・`:68`・`:76`、`disposal-row-utils.ts:50`・`:53`、`ReceivingRecordDetailPage.tsx:161`、`StockDetailContent.tsx:91`、`DisposalRecordDetailPage.tsx:167`、`StocktakeRecordDetailPage.tsx:182`〜`:204`、`receiving-request.ts:21`・`:52`・`:55`・`:61`・`:79`〜`:82`、`receiving-row-utils.ts:15`、`CostDiffDialog.tsx:59`・`:64`・`:125`・`:131`、`ReceivingPage.tsx:84`・`:601`・`:610`、`price-revision-math.ts:1`（`deriveProposedCost`。SPEC-PRV-D4 の円の単位の切り捨て）・`:11`〜`:12`（現掛率。test は `price-revision-math.test.ts`、Matrix の SPEC-PRV-D4 の行）、`product-form-request.ts:68`・`:85`・`:97`・`:124`・`:143`・`:149`・`:159`、`ProductForm.tsx:368`、`PriceHistorySection.tsx:97`〜`:98`、`PriceRevisionTable.tsx:48`〜`:129`、`ProductTable.tsx:76`。文「原価は0以上の整数で入力してください」に当たる test: `request-helpers.test.ts:237`・`:287`、`DisposalPage.test.tsx:254`、`receiving-request.test.ts:58` |
| 操作ログの原価の key | BIZ-01-D8 | C | `biz/product_service.rs:423`〜`:427`（BIZ-01-D7 の商品修正の detail_json の `"cost_price"`）・`:536`〜`:548`（`product_price_revise` の `format!`、`:542` が `"cost_price"`） | `docs/function-design/74-ui-operation-logs.md` §74.8 と操作ログ画面の既知 key の要約（`cost_price`〈円〉と `cost_price_centi` の両方） |
| Z004 の数量 | SPEC-UNIT-D8、23 §13.4.1 | P | `src-tauri/src/io/z004_parser.rs:57`（`quantity: i32`）・`:363`・`:408`（`parse_z004_int`） | `biz/csv_import_service/mod.rs:131`（`quantity: i32`）、`biz/csv_import_service/time_evidence.rs:139`（同）、parser の test（`z004_parser.rs:614` ほか） |
| EJ の数量と照合 | SPEC-UNIT-D8、IO-08.5 | P | `src-tauri/src/io/ej_parser.rs:41`・`:190`（`quantity: i64`）・`:1274`（`parse_quantity`）・`:787`（`quantity × unit_price = amount`）。同じ 100 倍に揃える: `:57`（`EjLineKind::ItemCount.count`）・`:1289`（`parse_item_count`）・`:174`（`Restored.item_count`）・`:776`（数量行の無い明細の `+1` → `+100`）・`:814`（戻の `-1` → `-100`）・`:707`〜`:717`（記録全体の照合） | `ej_parser.rs:1557`（test の helper `qty`）。共有する helper は `io/daily_report_parser.rs:535`（`parse_optional_hundredths`、D-104） |

### runtime の lane の L3（owner が Windows の実機で確かめる。Human Gate に manual を足す）

| # | lane | 到達経路 | 入力物 | fixture | 依存 |
|---|---|---|---|---|---|
| L3-1 | U | 商品登録 → 入庫 → 在庫照会 → 入庫記録詳細 | 長さの商品を画面で登録（`m`、売価 700・原価 400、初期在庫 25）、入庫 2.5 m・原価 333。保存の後の原価差分のダイアログは「見送って閉じる」を選ぶ（マスタ原価 400 を保つ。L3-5 の評価額の前提）。期待: 在庫照会 `27.5 m`、記録詳細の行 `832.50 円`・合計 `833 円` | 画面の入力だけ（DB の用意は不要） | J3、UI-02-D15 |
| L3-2 | U | 手動販売 → 日次売上 → 在庫照会 | L3-1 の商品を 1.3 m（金額の初期値 `910` のまま保存）。期待: 日次売上の行が数量 `1.3 m`・単価 `¥700/m`・金額 `¥910`、販売点数は点と m の 2 本（その日の売上がこの行だけなら `1.3 m`、ほかに個数の商品 5 点があれば `5 点・1.3 m`。長さは点に入れない）、在庫照会 `26.2 m` | 同上 | J4、TD-203・TD-206/207 |
| L3-3 | U | 廃棄 → 廃棄記録詳細 | L3-1 の商品を 0.5 m・原価 333（合計 167 円、行 166.50 円）。owner が丸めを受け入れるかを PASS / FAIL で答える | 同上 | TD-023 に合わせた案 |
| L3-4 | U | 商品 CSV の取込み → 在庫照会 | `在庫単位` に `玉`・`m`・`kg` を書いた CSV（`m` の行の `初期在庫` は `25`）。期待: 取り込んだ後の在庫照会で `m` の商品が `25 m`、毛糸の商品の単位が `玉`、`kg` の行は preview のエラー行 | 合成の CP932 の CSV（実商品名を使わない。`docs/DEV_WORKFLOW.md` の fixture の encoding の規則） | J1 |
| L3-5 | ⑤（棚卸しの再開の L3 へ移す。U の L3 には含めない） | 棚卸し → 確定 → 記録詳細 | L3-1〜L3-3 の後の商品の実数 26.15 m。期待: 確定の結果の一覧がシステム在庫 `25.7 m`・実際の数 `26.15 m`・差異 `-0.45 m`、評価額 `10,460 円`（マスタ原価 400） | 画面の入力だけ | U の merge と、棚卸しの再開（⑤、SPEC-STOP-D6）。今の build では棚卸しが停止中（SPEC-STOP-D1）で実行できない。TD-203 |
| L3-6 | C | 入庫 → 入庫記録詳細 → 在庫照会 | ばらす商品に原価 83.33・数量 36。期待: 入庫記録詳細の行 `2,999.88 円`・合計 `3,000 円`、在庫照会の原価 `83.33 円` | 画面の入力だけ | J2・J3 |

## Non-scope

- runtime の code・test・fixture・bindings・migration・`90-traceability.md`（後続の U・C・P の lane）。
- Z004 の取込みの再開（ADR の停止中）と EJ の取込みの配線。小数の数量の照合（EJ）の実データでの確認。
- 売価の小数。重さの単位。箱と、ばらした個数を別々に数える model と、入庫の「箱数 × 入数」の補助（店の運用は、ばらした個数で入れる。D-113 の Revisit）。巻と m の換算。商品の単位の登録後の変更。在庫少の基準の画面の語と m の表示。10 cm 刻みの検査。
- 在庫計数 Excel（売価・原価とも税抜、TD-166）から作る商品 CSV の税込への換算（初期投入の作業）。原価の税の扱いの定義（今の master-tables は書いていない。本 lane は変えない）。
- `docs/db-design/tracking-system-tables.md` の列の表の改名（原価の lane。並走 lane の変更中の file）。

## Acceptance Criteria

baseline は 2026-10-07 に本 worktree（HEAD `95c0aeb0`、編集前）で逐語に実行した出力。完了時の期待値と分けて書く。

- AC1（共通規則がある）: `rg -c '^\*\*SPEC-UNIT-D[0-9]+ ' docs/function-design/10-common-rules.md` が `11`。baseline: `rg -n 'SPEC-UNIT-D[0-9]+' docs --glob '!docs/archive/**' | wc -l` → `0`。
- AC2（決定の記録）: `rg -n '^## D-113' docs/decision-log.md` が 1 行で、その節に `owner の決定（2026-10-08、TD-195` の J1〜J4 と `Revisit` がある（`sed -n '/^## D-113/,$p' docs/decision-log.md | rg -c '^- owner の決定（2026-10-08、TD-195|^- Revisit'` が `2`）。baseline: `rg -n '\bD-113\b' docs | wc -l` → `0`。
- AC3（既知の不整合の pointer）: 既知の不整合を書いた 3 file（`docs/backlog.md`・`docs/db-design/master-tables.md`・`docs/function-design/35-biz-stocktake-service.md`）がどれも `D-113` を含む（`rg -c 'D-113' <file>` が 1 以上）。baseline: `rg -n '100 倍になる' docs --glob '!docs/archive/**' --glob '!docs/research/**'` → 3 行（`docs/backlog.md:86`・`docs/db-design/master-tables.md:56`・`docs/function-design/35-biz-stocktake-service.md:451`）。
- AC4（決定が正本に見える）: `rg -o '決定（owner、D-113 J[1-4]）' docs --glob '!docs/archive/**' --glob '!docs/plans/**' | sed 's/.*J/J/' | sort -u` が `J1）` `J2）` `J3）` `J4）` の 4 行で、`rg -n '未[決]（owner、D-113' docs --glob '!docs/archive/**' --glob '!docs/plans/**' | wc -l` が `0`（2026-10-08 に J1〜J4 の決定で「未決の 4 つが正本に見える」から改めた）。
- AC5（traceability の入力を変えない）: `git diff "$(git merge-base origin/main HEAD)" -- docs | grep -c '^[-+]>.*対応仕様'` が `0`。
- AC6（docs だけ）: `git diff --name-only "$(git merge-base origin/main HEAD)" | grep -vc '^docs/'` が `0`。
- AC7（検査）: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-10-07-unit-extension.md` が ERROR 0（WARN は報告）。

再実行（2026-10-08、J1〜J4 の決定の反映。本 worktree で逐語に実行。左は編集前 HEAD `749b21fed5b03ccf66146977f68acee5e99572d4`、右は反映後の作業木）:

| AC | command | 編集前 | 反映後 |
|---|---|---|---|
| AC1 | `rg -c '^\*\*SPEC-UNIT-D[0-9]+ ' docs/function-design/10-common-rules.md` | `11` | `11` |
| AC2 | `rg -n '^## D-113' docs/decision-log.md` | `938:## D-113: …` の 1 行 | 同じ 1 行 |
| AC2 | `sed -n '/^## D-113/,$p' docs/decision-log.md \| rg -c '^- owner の決定（2026-10-08、TD-195\|^- Revisit'` | `1`（Revisit だけ） | `2` |
| AC3 | `rg -c 'D-113' <file>`（backlog・master-tables・35） | `2`・`4`・`1` | `2`・`4`・`1` |
| AC4 | `rg -o '決定（owner、D-113 J[1-4]）' docs --glob '!docs/archive/**' --glob '!docs/plans/**' \| sed 's/.*J/J/' \| sort -u \| wc -l` | `0` | `4`（`J1）` `J2）` `J3）` `J4）`） |
| AC4 | `rg -n '未[決]（owner、D-113' docs --glob '!docs/archive/**' --glob '!docs/plans/**' \| wc -l` | `19` | `0` |
| AC5 | `git diff "$(git merge-base origin/main HEAD)" -- docs \| grep -c '^[-+]>.*対応仕様'` | `0` | `0` |
| AC6 | `git diff --name-only "$(git merge-base origin/main HEAD)" \| grep -vc '^docs/'` | `0` | `0` |

## Design Readiness

- 引用する設計正本（節まで）: `docs/function-design/10-common-rules.md` の「単位・数量・金額の共通規則」SPEC-UNIT-D1〜D11／`docs/db-design/master-tables.md` の「単位と原価の精度の契約」・products の設計意図の価格の基準数量／`docs/function-design/22-mnt-migration.md` §16（MNT-03-D13）・§15（v7、同じ形）／`35-biz-stocktake-service.md` §20.5a（SPEC-STK-VAL-D1〜D6、値は変えない）／`31-biz-inventory-service.md` の「単位と原価の精度の契約」・§12.6a・§12.8／`21-io-inventory-repo.md` §10.2・§10.5／`30-biz-product-service.md` BIZ-01-D7・D8、§4.8・§4.9／`51-ui-product-form.md` UI-01b-D5・D6・D22／`34-biz-sales-service.md` 冒頭・`56-ui-daily-sales.md` §56.10・`57-ui-monthly-sales.md` §57.6（TD-203）／`62-ui-manual-sale.md` UI-04-D6・D7・D18／`58-ui-stock-inquiry.md` §58.6／`23-io-z004-parser.md` §13.4.1／`29-io-ej-parser.md` IO-08.5・IO-08-D6／`77-ui-bulk-price-revision.md` SPEC-PRV-D4／`44-cmd-inventory.md` `list_low_stock`／`DB_DESIGN.md` CHECK 制約方針・D-4／decision-log D-061・D-064・D-104・D-113。
- 必要な設計成果物: function-design（共通規則・BIZ-01・BIZ-02・IO-01 の repo・IO-02・IO-08・UI-01b・UI-04・UI-06a・UI-14） = updated in this PR（proposed）／DB（CHECK・原価の列・migration） = updated in this PR（migration の番号は runtime）／decision-log = D-113 を追加／SCREEN_DESIGN = existing sufficient（画面の構成・動線は変えず、欄の単位と文言は各 UI の function-design が持つ）。
- plan にしかない durable な判断の昇格先: すべて D-113 と上の正本へ置いた。runtime の lane の分け方（U・C・P）は D-113 の Decision (8)。申し送りの表と L3 の表は runtime の lane の起票の出発点で、durable な判断ではない。
- 前提・制約と、延期した design gap: 原価の 1/100 円（J2）・入庫の丸め（J3）・手動販売の丸め（J4）・玉（J1）は 2026-10-08 に owner が推奨の案に決めた（TD-195）。起票時に推奨の案で書いた正本の記述を決定の文に直した（値は変わらない）。延期: EJ の小数の数量の照合の丸め（レジの四捨五入）を実データで確かめること（P の lane の Plan Gate の前。下の Contract Probe P3）、棚卸し記録詳細のロス原価をどの lane が BIZ へ移すか（U か棚卸し ⑤）、`tracking-system-tables.md` の列の表の更新（C）。延期が安全な理由: どれも本 lane の設計の値を変えず、runtime の lane の起票時に決めれば足りる。
- 絶対保証（cannot happen / always happens）の例外と escape hatch の自己点検: 「在庫の数量は浮動小数を通らない」の例外は無い（入力は文字列から、表示は整数の演算から。D-104 の日報の wire の `f64` は在庫に効かない表示用で、本契約の外）。「既存の `cm` の行の値と意味を変えない」: vU は値を写すだけ、基準数量は今と同じ 100。「新しい単位が在庫少から漏れない」: SQL の並びを enum の全 variant から作り test で止める（SPEC-UNIT-D10）。「金額はどの画面でも同じ値」の例外: 入力中の見込み（廃棄の合計、手動販売の初期値）は TS の twin で、golden の表で BIZ と揃える（SPEC-UNIT-D9）。twin がずれた場合も保存された記録は BIZ の値。「原価の変換で値を失わない」: vC は範囲検査・件数・`typeof`・余りの検査で、外れれば vC の変更を何も残さない（22 §16。同じ起動で先に COMMIT した vU の版は残るので、回復は 22 §16 の手順）。
- 判定（ready / not ready）と理由: ready（2026-10-08、plan-draft へ進めた）。設計の出力は上の正本にあり、owner の判断 J1〜J4 は決まって正本と D-113 に決定として書いた（AC2・AC4）。上の延期は runtime の lane の起票時に決めれば足り、未解決の設計の問いは無い。起票時（2026-10-07）の判定は not ready（J1〜J4 が残っていた）。

## Registration / Generation Obligations

本 lane（docs だけ）は該当なし（function-design の新しい file・REQ の増減・route・画面・command の新設をしない）。runtime の lane の義務: U は `cargo run --bin generate_bindings`（`ProductStockUnit` の 12 値、6 型の `stock_unit`、`StocktakeItemDetail.stock_unit`、行の金額の `_centi`〈十進の文字列〉、TD-203 の `DailySaleItem`・`AdjustedItem`・`IntegrityMismatch`・`StockAdjustment`・`ProductResponse` の `stock_unit`、`DeptSubtotal`・`GrandTotal`・`MonthlySaleItem` の `count_points`・`length_cm`、`MonthlySaleItem.stock_unit`）と migration の登録（`db/migration.rs` の migrations()、22 §16）。設計と code の検査（`src-tauri/tests/design_compliance_test.rs` の `design_code_compliance_phase_a`）は、map に無い module か、map の設計書の code block に無い `pub fn` を「未文書化」として fail にする（`:726`〜`:738`。`SKIP_DOCS` の設計書は関数を読まれないので、そこにだけ書いた `pub fn` も未文書化になる）。よって U は `10-common-rules.md` を `SKIP_DOCS`（`:25`〜`:26`）から外し、`build_doc_to_modules_map()`（`:108`）に `"10-common-rules.md" => ["biz::unit_amount"]` を足す（10 の code block の `fn` はシグネチャの 8 つだけで、まだ作らない `stock_quantity_from_pos`〈P〉は INFO の未実装で fail にならない）。migration の新しい関数は今の v6・v7 と同じ `pub(crate)` にする（`pub` にすると `db::schema_vN` が map に無く fail）。BIZ-05 等の既存 module に `pub fn` を足すなら、その設計書（34 ほか）の code block にシグネチャを書く。棚卸しの評価額の 3 関数は今 private なので、移しても `35-biz-stocktake-service.md` の map（`biz::stocktake_service`）は変えない。C は bindings の再生成と migration の登録。test の名前に `_reqNNN` を足すか変えるなら `cargo run --bin generate_traceability`（T4 の baseline を含む）。

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | Which concepts belong to replaceable external adapters, and which concepts are stable app-core contracts? | Architecture, function design, decision-log, Plan Packet | レジの数量の書式（小数 2 桁、カンマ）は adapter（IO-02・IO-08）。core は「数量の種類」「価格の基準数量」「レジの数量 1 = 基準数量」「100 倍の整数の POS の数量を商品を引いた後に換算」。IO は単位を知らない（SPEC-UNIT-D8） | P の lane |
| Fact check / design decision split | Which claims are observed facts from hardware/tool/files, and which are app decisions that need source-doc promotion? | Investigation doc, source design docs, decision-log | 事実: 12 種・m・10 cm・数量 1 = 1 m・小数 1 桁・原価の小数第 2 位・評価額の丸め（台帳の番号と `project-memory.md`）。判断: SPEC-UNIT-D1〜D11・D-113。レジの丸め（L-136）は店の答えで、データでは未確認（Contract Probe P3） | — |
| Lifecycle / retry | What happens before, during, after, and after failure for import/export, duplicate input, rollback, retry, cancellation, and re-run? | Function design, DB design, UI design, Test Matrix | migration vU・vC はそれぞれ 1 つの TX で、失敗は失敗した migration の変更と版だけを戻す（vU の上限の検査と vC の範囲検査は、何かを変える前に止まる）（`migrate` は migration ごとに COMMIT するので、同じ起動で先に成功した版は残る）。再実行で重複適用しない。旧版のアプリは開けない（MNT-03-D11）ので、旧版へ戻すのはアプリを止めて行う 22 §16 の回復の手順（復元の中断の遺物の不在と、更新の前の backup の版と整合性の確かめ → 今の DB と WAL・SHM の保全 → file の置き換え → 旧版の起動）だけ。冪等の fingerprint は移行の前の記録を書き換えない（31 の proposed 節） | Matrix の State Lifecycle |
| Operator workflow | What does the operator do in the real sequence across app, external tool, media, print/export, backup, and recovery? | Screen/UI design, function design, Plan Packet manual checks | 上の Ordinary Operation。長さの商品は PLU にせず手動販売で在庫を減らす（今の運用、`plu_target=0`） | runtime の L3 の表 |
| Replacement path | If the external system changes, which files/modules/docs are replaced and which app-core contracts remain stable? | Architecture, function design, decision-log | レジが替われば IO-02・IO-08 の数量の書式を替え、SPEC-UNIT-D8 の BIZ の換算と D4 の不変条件は残る。新しいレジで数量 1 ≠ 基準数量なら D4 を見直す（D-113 の Revisit） | — |
| Data safety / evidence | How can the claim be supported by anonymized shape/count/hash/procedure evidence without committing real store data? | Plan Packet Data Safety, investigation doc, review evidence | 店の事実は台帳の番号と要旨だけ。例の金額（700・333・83.33）は説明用の合成の値で、店の商品の値ではない | Data Safety |
| Reporting / accounting semantics | Are totals, summaries, item records, returns, corrections, and inventory movements modeled separately enough to avoid false business meaning? | DB design, function design, report design, Test Matrix | 原価の金額（入庫・廃棄・棚卸し）は 1 つの関数で同じ丸め。数量・在庫・売価・原価は入力の上限で、円の合計は保存の前の検査で、JS の number で正確に運べる範囲に収める（10 の「入力の上限と安全な整数の範囲」。売上の金額とその集計は表の外）。売上の金額は POS の金額のままで、在庫の数量から作り直さない。日報の個数（D-104）は在庫に効かず、本契約で変えない。個数の商品の整数でない POS の数量は在庫に効かせず示す | Matrix |
| Manual verification | Which assertions cannot be proven by automated tests and require Windows native L3, external tool import, or real-device confirmation? | Plan Packet, Test Matrix, PR body | 丸めの受入れ（廃棄の TD-023 の案、J3・J4 の結果を実物の伝票・レシートと比べる）、m の入力と表示の読みやすさ、商品 CSV の CP932 の取込み | runtime の L3 の表 |
| 環境・再現性 | 新設の環境依存（toolchain / CI runner / OS 差異等）を repo-pinned config で強制するか、明示的に defer するか | repo-pinned config, Plan Packet | 新しい環境依存は無い。`RENAME COLUMN` は同梱の SQLite 3.45.0（`libsqlite3-sys` 0.28.0、`src-tauri/Cargo.lock`）で使える | — |

## Boundary / Wire Contract

- producer: BIZ-01（商品・CSV の preview）、BIZ-02 §12.6a（記録詳細の金額）、BIZ-05（売上の数量と単位、TD-203）、BIZ-06（棚卸し）、整合性チェック・PLU 書出しの DTO、repo（`stock_unit` の読取り）、後続の IO-02・IO-08（POS の数量）。
- consumer: UI-01b・UI-02・UI-03・UI-04・UI-05・UI-06a・棚卸し・業務記録詳細の画面・UI-14・操作ログ画面・日次売上・月次売上・ホーム・在庫変動・整合性チェック・PLU 書出し（TD-203）。
- wire type: `ProductStockUnit` = 12 個の string literal（generated enum）。数量は今どおり整数（長さは cm）。売上の集計の数量は数量の種類ごとの 2 つの整数（`count_points`・`length_cm`、TD-206/207、SPEC-UNIT-D3。和は checked で、溢れは `ValidationFailed`。1 明細の数量の上限の下では `±9007199254740991` に届かないので、読取りを範囲で拒まない）。月次の商品別の行は `stock_unit` で種類を運ぶ。行の金額は 1/100 円の整数の十進の文字列（`line_cost_centi`・`line_loss_cost_centi`、i128 の値を損失なく運ぶ）、合計は円の整数（入庫・廃棄の create と棚卸しの確定が、保存の前に `±9007199254740991` で検査する）。原価は 1/100 円の整数（`cost_price_centi` ほか、C）。
- internal type: `StockUnitKind`、`quantity_hundredths: i64`（POS、P）、金額の中間は i128。
- precision/range: 長さの最小は 1 cm、m の入力は小数 2 桁まで。入力の上限（10 の「入力の上限と安全な整数の範囲」）: 1 明細の数量は 1 以上 `9999999`（長さは `99999.99` m）、在庫は絶対値 `999999999`（`9999999.99` m）、売価は `999999` 円、原価は 1/100 円で `99999999`（`999999.99` 円）、原価が円の間（単位の lane）は `999999` 円（vU の上限の検査と vC の範囲検査も `999999` 円）。円の合計は `±9007199254740991`（保存の前の検査）。行の金額は i128 の範囲（十進の文字列）。POS の数量は小数 2 桁まで。
- round-trip path: 画面の文字列（m・原価）→ TS の純関数で整数 → wire → BIZ の検証 → DB → repo → wire → TS の formatter（整数の演算）→ 画面。表示した文字列を保存し直す経路は作らない（127 cm を 1.3 m と見せない）。
- invalid input: m の小数 3 桁以上・指数・符号は入力の error。0 を受けない数量欄の `0` は入力の error。数量・在庫・売価・原価の上限超えは `ValidationFailed`（画面は入力の error、file の行は行の error）で何も書かない。原価の小数 3 桁以上は error。DB の一覧に無い単位は読取りの error。商品 CSV の不正な `在庫単位` は preview の行の error。個数の商品の整数でない POS の数量は在庫に効かせない（P）。
- compatibility: 既存の `pcs` / `cm` の値は変えない。商品 CSV の上書きは単位を変えない（BIZ-01-D8）。TS の型検査が止めるのは、formatter の param を `ProductStockUnit` にした後に `string` のまま渡す所（行の型の `stockUnit: string` 4 つ等）と、改名・削除した field を読む所（原価の field の改名〈C〉、売上の `quantity`）。wire の enum を `string` の変数に入れる所は止まらない（広げる代入は通る）ので、行の型 4 つは申し送りの表のとおり直す。移行の前の操作ログの `cost_price`（円）は書き換えない。

## Test Plan

Test Design Matrix: [2026-10-07-unit-extension](test-matrices/2026-10-07-unit-extension.md)（runtime の lane が実装する test の設計。本 lane は docs の検査だけ）。

- targeted tests: 本 lane は `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-10-07-unit-extension.md` と full。
- negative tests: Matrix の Negative Paths（runtime）。
- compatibility checks: 既存の `cm` の商品の評価額が今と同じ値（Matrix）、vU の前後で全列が同じ（22 §16）。
- data safety checks: 本 lane の差分に実データ（JAN・商品名・金額・件数の細部・店主の発言の原文）を入れない。
- main wiring/integration checks: runtime の lane。Human Gate に manual（L3）を含むのは runtime の lane で、Writer は L3 の前に `cargo check --release` を回す。

## Review Focus

- Ordinary Operation の列が、正常な条件で目的（12 単位と m で日々の入出庫・棚卸しができ、金額が 1 m あたりで正しく出る）を達成できるか。危険な結果（在庫の数量の変化・既存の `cm` の値の変化・在庫少の漏れ）を出さないか。
- SPEC-UNIT-D1 の code 1 列の判断（基準と表示を 2 列にしない）が、店の 12 種と箱・袋の商品を表せるか。
- SPEC-UNIT-D6 の丸めが、入庫・廃棄・棚卸しで同じ明細から同じ値を出すか。棚卸しの今の値を変えないか。
- 22 §16 の vU（表の作り直し）・vC（改名と 100 倍）の手順と回復が、時点証拠の migration と並べても成り立つか。
- 申し送りの表に、型を変える所・読む所・test の漏れが無いか（特に wire の `stock_unit` の 6 型、`StocktakeItemDetail`、在庫少の SQL、TD-203 で単位を足す売上・棚卸しの確定の結果・整合性チェック・PLU 書出しの wire）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| D-113 | `docs/decision-log.md` D-113 | 本 lane は設計の記録。runtime は U・C・P | runtime（Matrix） | — |
| SPEC-UNIT-D1 | 10 共通規則 | U: `product_repo.rs` の enum、vU | Matrix | L3-4 |
| SPEC-UNIT-D2 | 10 共通規則 | U（数量の型は変えない） | Matrix | 非対象 |
| SPEC-UNIT-D3 | 10 共通規則（表示する所の表、TD-203）、30・34・36・51・53・56・57・58・61・62・63・64・66・67・73・74・75、SCREEN_DESIGN の生地の行 | U: TS の純関数と各画面、BIZ-05 の集計、BIZ-02 の在庫警告 | Matrix | L3-1・L3-2・L3-4（棚卸しの L3-5 は ⑤） |
| UI-02-D7・UI-03-D12・UI-05-D8（数量の入力）、UI-06c-D5（在庫変動の増減）、UI-13-D3（整合性チェックの確認）、UI-11c-D14（操作ログの補正の要約） | 61・63・64・66・75・74 の該当行の proposed の pointer | U: SPEC-UNIT-D3 の表示する所の表と入力の規則に従う | Matrix の SPEC-UNIT-D3 の入力・加算の行と TD-203 の RTL の行 | L3-1・L3-3 |
| SPEC-UNIT-D4 | 10 共通規則 | U（PLU は変えない） | Matrix（PLU の単価の回帰） | 非対象 |
| SPEC-UNIT-D5 | 10 共通規則、master-tables、22 §16（vC） | C | Matrix | L3-6 |
| SPEC-UNIT-D6 | 10 共通規則、21・31・35 §20.5a、transaction-tables | U: `biz::unit_amount` | Matrix | L3-1・L3-3 |
| SPEC-UNIT-D7 | 10 共通規則、62 UI-04-D18 | U | Matrix | L3-2 |
| SPEC-UNIT-D8 | 10 共通規則、23 §13.4.1、29 IO-08.5 | P | Matrix（P の lane が起票時に足す） | P の lane |
| SPEC-UNIT-D9 | 10 共通規則、58 §58.6 | U | Matrix（golden の twin） | 非対象 |
| SPEC-UNIT-D10 | 10 共通規則、44、DB_DESIGN D-4、30 BIZ-01-D8 | U | Matrix | L3-4 |
| SPEC-UNIT-D11 | 10 共通規則（wire の単位の型と「入力の上限と安全な整数の範囲」）、22 §16（vU の上限の検査）、30・31・34・36・40・73 | U（数量・在庫・売価・原価〈円〉の上限、入庫・廃棄の合計の検査、集計と整合性の checked、vU の上限の検査）・⑤（棚卸しの実数の上限と確定の総額の検査）・C（原価の上限を 1/100 円へ書き換える、vC の範囲検査）・P（POS の数量の換算の後の上限） | Matrix（型検査、「SPEC-UNIT-D11 入力の上限」の 4 行、円の合計の保存の前の検査の行、vU の上限の検査の行） | 非対象 |
| MNT-03-D13 | 22 §16 | U（vU）・C（vC） | Matrix | 非対象 |
| UI-01b-D22 | 51 | U | Matrix | L3-1 |
| UI-04-D18 | 62 | U | Matrix | L3-2 |
| BIZ-01-D8 | 30 | U（単位・CSV）・C（原価・ログ） | Matrix | L3-4 |
| SPEC-PRV-D4（proposed の注記） | 77 | C | Matrix（`price-revision-math.test.ts`） | 非対象 |
| 隣接の除外: SPEC-STK-VAL-D2〜D6 | 35 §20.5a | 変えない（関数を移すだけ、値は同じ） | 既存の `stocktake_service.rs:1402`〜`:1482` を移して回す | 非対象 |
| 隣接の除外: D-104（日報の個数） | 29 IO-07-D2、pos-tables | 変えない（helper を共有するだけ） | 既存の日報の test | 非対象 |
| 隣接の除外: IO-04（PLU の単価） | 25 | 変えない（SPEC-UNIT-D4 の不変条件） | 既存の PLU の test | 非対象 |
| 隣接の除外: UI-11a（在庫少の基準の画面） | 69 | 変えない（基準の語と cm の入力） | — | 非対象 |
| 隣接の除外: UI-01b-D5（単位は登録後に変えない） | 51 | 変えない | 既存 | 非対象 |

## Contract Probe

- P1 SQLite で CHECK を変えるには表の作り直しが要り、列の改名は `RENAME COLUMN` でできるか: **repo の現物で確かめた（2026-10-07）**。表の作り直しの先例は `src-tauri/src/db/schema_time_evidence.rs:45`〜`:92`（`stocktakes_new`・`stocktake_items_new` を作って写し、`RENAME TO`）と `:143`〜`:158`（TX の外で `foreign_keys` を OFF にして戻す）。列の改名と 100 倍の先例は `src-tauri/src/db/schema_v7.rs:52`〜`:56`（`RENAME COLUMN quantity TO quantity_hundredths` と `* 100`）。同梱の SQLite は `libsqlite3-sys` 0.28.0（`src-tauri/Cargo.lock`。22 §15 が 3.45.0 と記録）。products に trigger と view は無い（`rg -n "CREATE TRIGGER|CREATE VIEW" src-tauri/src/db/` は migration の test の 1 行だけ）。index は 3 つ（`schema_v1.rs:251`〜`:253`）。
- P2 レジの数量の小数の桁: **文書と店の答えで確かめた**。取説 S p.154 の乗算の数量は 0.01〜9999.99（repo 外の取説の検証記録、2026-10-06）。店の実例は Z004・EJ・精算レシートとも小数 1 桁（台帳 TD-147）。よって 100 倍の整数（小数 2 桁まで）で足りる。Z004 の数量欄の生の文字列の形（`1.3` か `1.30` か）は未確認で、どちらも受ける（SPEC-UNIT-D8）。
- P3 レジは小数の数量 × 単価の端数を四捨五入するか（EJ の照合が依る。J4 は店の答え L-136 で決めた）: **未確認（店の答え L-136 だけ）**。本 lane の設計の値は依らない（EJ の照合は P の lane、J4 は owner が決めた）。確認: P の lane の Plan Gate の前に、持ち帰った EJ の小数の数量の明細で `四捨五入(数量 × 単価) = 金額` が成り立つかを数える（件数だけを出す。金額・商品名は出さない。`.local/checklists/field-data.md` の手順）。
- P4 JS の number で正確に運べる最大の整数は `9007199254740991`（ECMAScript の `Number.MAX_SAFE_INTEGER`）: 言語の定数で、probe は要らない。
- 観測済みで probe の要らない前提: 店の 12 種・m・10 cm・数量 1 = 1 m・原価の小数第 2 位・評価額の丸め（台帳と `project-memory.md`）。今の評価額の関数の値（`stocktake_service.rs:1402`〜`:1482` の test）。

## Data Safety

- tracked に書かない: 実 JAN・商品名・金額・件数の細部、店主の発言の原文、持ち帰りデータの中身。本 lane の docs は店の事実を台帳の番号と要旨で書き、例の金額は説明用の合成の値。
- local-only: repo 外の回答台帳（`.local/reports/store-premises/`）と相談の記録（`.local/consultations/2026-09-22-q4-*`）。
- synthetic-only: runtime の lane の test と L3 の fixture（合成の商品・合成の CP932 の CSV）。

## Implementation Results

- 結果: 単位を 12 種へ広げ（m は在庫を cm の整数で持ち、1 m あたりで値付けする）、数量・原価の精度（原価を 1/100 円で持つ）と金額の丸めを共通規則 SPEC-UNIT-D1〜D11 として決め、D-113 に記録した。数量・在庫・売価・原価の入力に上限を掛け（1 明細の数量 9,999,999、在庫 999,999,999、売価 999,999 円、原価 999,999.99 円）、上限からは言えない円の合計は保存の前に検査する。移行（vU・vC）の手順、移行の前の上限の検査、途中で失敗したときの回復の手順を 22 §16 に決めた。本 lane は docs だけで、code・migration・bindings は変えていない（runtime は後続の lane）。
- 既存の保護: 実装は変えていないので、店の動作は今のまま（単位は `個` / `cm` の 2 つ、原価は円の整数）。SPEC-UNIT-D1〜D11 は proposed・未実装で、runtime の lane が入るまで各文書の現行の記述が実装の正本（上の Ordinary Operation のとおり、12 単位と m の通常運用は本 lane では未達）。
- 未検証: 移行の「連続適用の失敗」の例の probe は sqlite 3.51.3 で確かめた。同梱の SQLite 3.45.0 と Rust の実装では回していない（runtime の lane の test が確かめる）。
- review・CI・merge（closeout、2026-10-11）: Plan Review は round 3（上限）までで、round 3 は Claude 側 approve・Codex 側 reject（P1 0）、残りを disposition「同型指摘の一括是正」とした後に owner が plan-approved を承認した。Final Review は毎回 Codex GPT-6 Astra と Claude 側 Fable 5.1 の独立 2 本で 5 回（下の Review Response の Closeout の段落）。helper の record は review = pass、manual = not-required、r4 = pass（owner の R4 の承認 2026-10-11。docs だけで DB には触れない。DB を書き換える runtime の lane はそれぞれの R4 を別に取る）。Ready の後の hosted run の `Merge gate` は success で、owner の R4・Ready・merge の承認（2026-10-11、介入 14 回 / 予算 14 回）の後に helper 経由の squash merge。検査の結果は PR と GitHub が持つ。PR: [#154](https://github.com/higekiri-w/inventory-system-desktop/pull/154)
- 後続: runtime は単位の lane（U）→ 原価の lane（C）、Z004・EJ の数量は CASIO 固有の lane（P）（上の「runtime の lane への申し送り」の表）。Final Review の 5 回目の P3 は `docs/backlog.md` の「単位の拡張」の項の「後続の lane の起票の入力（Final Review の P3）」。

## Review Response

round 1（`b99ef7f4`）: Claude 側 fresh Opus 5.5 = reject（P2 4 / P3 3）、Codex GPT-6 Astra（発注 246）= reject（P2 8 / P3 1）。P1 は 0。重なりは 3 件（Opus F1 = Codex #3、F5 = #4、F6 = #7）。Coordinator が採否を決め、全件を採用した。是正は起草役の `8981cbc9`・`a3b64f3f`。Scope が広がったので plan-draft へ戻し、plan-gate へ出し直した（遷移の記録 6・7）。

- P2（Opus F2）m で出す範囲が在庫の数の表示だけで、日次売上・在庫変動・棚卸しの確定結果などが cm の数のままだった（L3-2 で数量 `130`・単価 `¥7`）。owner に諮り、数量を出す画面はすべて m・単価は 1 m あたり（repo 外の回答台帳 TD-203）。是正: 10 の SPEC-UNIT-D3 に表示する所の表、D11 に単位を持たない wire への `stock_unit` の追加、34・56・57・58・73 に注記、申し送りに 6 行。単位をまたぐ集計は owner に諮り、個数の種類の 10 単位を「点」でまとめ、長さ（m・cm）を別に m で出す 2 本（TD-206・TD-207。例 `5 点・1.8 m`。レジの数量のまま足す `4.3 点` は採らない。日報の点数との照合はレジ日報〈公式〉の節が受け持つ）。wire は `count_points`・`length_cm` の 2 field。
- P2（Opus F1・Codex #3）評価額の原価の引数が 1/100 円の i64 で、今の test の範囲と両立しない。是正: 引数を i128 にし、U の間の 100 倍と乗算を checked で行う。今の test は原価を 100 倍して同じ期待で回す。
- P2（Codex #1）CSV の上書きで単位を変えられる。是正: preview と commit の再検証で既存の商品と違う単位を拒む。
- P2（Codex #2）旧版のアプリで更新前の backup を戻す経路に到達できない。是正: 22 §16 に管理者が行う 6 手順のオフラインの回復を書いた（71 の restore の契約は変えない）。
- P2（Opus F5・Codex #4）vC の範囲検査の上限が JS の安全な整数とつながっていない。是正: 100 倍の後が安全な整数に入ることを移行の前に検査する。
- P2（Codex #5）新設する行金額の wire が i128 の結果を損失なく運べない。是正: 1/100 円の整数の十進の文字列にした（合計の円は number のまま、残る制約として明記）。
- P2（Codex #6）入庫・返品・廃棄の数量の再追加・統合の helper が整数。是正: cm の整数で足す契約と `1.3→2.3`・`0.5+0.3→0.8` の操作の test を申し送りと Matrix に。
- P2（Opus F6・Codex #7）Ordinary Operation と L3-1 に原価差分のダイアログの選択が無く、L3-5 の期待が一意でない。是正: 「見送って閉じる」でマスタ原価 400 を保つ操作を書いた。
- P2（Codex #8）SPEC-PRV-D4 の原価案・掛率の Matrix の行が無い。是正: `8333,100,110 → 9100`・掛率 83.3%・現売価 0 の fallback の行と、既存の `price-revision-math.test.ts` を対象に。
- P2（Opus F3）`product_price_revise` の操作ログの key の改名が漏れていた。P2（Opus F4）商品 CSV の m の行の `初期在庫` の読み方が無かった。是正: Coordinator が店の事実（台帳 L-214、残り・仕入れ・値札は m）から m の数として読むと決めた（`m,25` → 2500 cm）。介入に数えない。
- P3: demo の seed の 2 か所（Opus F7）、原価の wire の型の数を 14 に（Codex #9）。
- 起草役の判断で Coordinator が受けたもの: 67（PLU 書出し）と 53（ホーム）は並走 lane が触るので編集せず、10 の表と申し送りで持たせた。月次の CSV の部門別の行は `5 点・1.8 m` の文字列（個数だけの部門は今と同じ整数）。

round 2（`0b3dc340`）: Claude 側 fresh Opus 5.5 = reject（P2 2 / P3 4）、Codex GPT-6 Astra（発注 251）= reject（P2 5）。P1 は 0。重なりは Opus #2 = Codex #2、Opus #4 = Codex #3、Opus #6 と Codex #1 は同じ所。Coordinator が採否を決め、全件を採用した。是正は起草役の `74125ea7`（自己点検で同型の穴も直した）。

- P2（Codex #1・Opus #6）`cm` 単位の商品の表示が「長さはすべて m」と食い違っていた。Coordinator の決定: 数量の表示は長さの種類（m・cm）ともに m に揃える（TD-203 と TD-207〈長さは m・cm とも m で出す〉を延ばした解釈。入力の単位と保存値は変えない）。介入に数えない。
- P2（Opus #1）棚卸しは今の build で停止中で、棚卸しの m 表示と L3-5 は ⑤ に依る。是正: Ordinary Operation の行 6 と L3-5 の依存に ⑤ を書き、L3-5 を ⑤ の L3 へ移した。新方式の確定は ⑤ が `unit_amount` と `stock_unit` で作ると申し送りと 35 §20.5a に足した。
- P2（Opus #2・Codex #2）BIZ が文字列に埋め込む在庫警告・整合性チェックの補正結果・操作ログの補正の量が「表示する所」の表から漏れていた。是正: 表に行 10〜12、31 に在庫警告の 4 か所の整形の契約、記録済みの値は表の外と明記。
- P2（Opus #4・Codex #3）migration の失敗で戻るのは失敗した migration の TX だけで、成功済みの版は残る。是正: 22 §16 の回復を `migrate` の現物（migration ごとの COMMIT）に合わせ、更新の前の backup へ戻す分岐と連続適用の失敗の Matrix の行を足した。
- P2（Codex #4）vC の範囲外の値が画面で直せない履歴の列にある。是正: 更新保留・原本保全・管理者対応（R4 の判断として owner に諮る）の回復の分岐。
- P2（Codex #5）原価の parser の oracle が浮動小数の誤りを落とせない。是正: `0.29→29`・`0.57→57`・`45035996273704.95` 等の境界を足した。
- P3: m の入力の上限（Opus #3）、vC で止まったときに開ける版（Opus #4 の残り）、UI-04-D18 の求め直しの範囲（Opus #5）。
- 起草役の判断で Coordinator が受けたもの: UI-04-D18 で数量に合わせて金額を求め直すのは長さの商品だけ（個数の商品の今の振舞いを変えない）。58 の現行の実装の説明に残る `300 cm` は今の正本として残し、変更後の契約は §58.6 の proposed の項に置く。

round 3（上限、`5abaa7ed`）: Claude 側 fresh Opus 5.5 = approve（P3 5）、Codex GPT-6 Astra（発注 253）= reject（P2 2 / P3 2）。P1 は 0。round 天井に達したので次の round を開始せず、全件を disposition「同型指摘の一括是正」とした（Coordinator が採用。是正は起草役の `e6aead3d`、同型の穴の自己点検を含む）。Coordinator の確かめ: 10 の表の行 6 が `MovementTable` の 7 経路と `product_code → stock_unit` の対応を必須で渡す契約を持つ、Registration の U の義務が `src-tauri/tests/design_compliance_test.rs` の現物（`SKIP_DOCS` の `:25`〜`:26` に `10-common-rules.md` がある）に合わせて「`SKIP_DOCS` から外し対応付ける」になっている、を実読で確かめた。

- P2（Codex #1）`MovementTable` の単位の渡し方が商品別の画面だけで、記録詳細の 6 経路が単位を持たない。是正: 上のとおり、混在した伝票の Matrix の oracle。
- P2（Codex #2）新 module の登録が不要という記述が検査の実装と逆。是正: 上のとおり。
- P3: 統合の fallback を今の動作どおり廃棄と返品で分けた（Codex #3）。網羅の行を表の全行に（Codex #4）。CSV の取込みの確かめを在庫照会で（Opus #1）。L3-6 の期待（Opus #2）。CSV の初期在庫の上限（Opus #3）。入庫・廃棄の原価の上限（Opus #4）。行番号（Opus #5）。
- 自己点検で直したもの: 「enum にすると TS の型検査が全部止める」の言い過ぎを、止まる所と止まらない所に分けた。migration の新しい関数を `pub(crate)` に、既存の module に足す `pub fn` は設計書の code block に書く義務を足した。`generate_bindings` の対象に TD-203・TD-207 の field を足した。

Final broad（`c4f9822e`）: Claude 側 Fable 5.1 = reject（P2 1 / P3 1）、Codex GPT-6 Astra（発注 255、PR review 5447043067）= reject（P2 4）。P1 は 0。Coordinator が全件を採用し、起草役が `9a6929fc`（Gated Amendment 1）で直した。broad は helper に pending で記録済み。

- P2（Fable）m の商品の初期在庫 0 を D3 が受けない。是正: 0 を受ける欄に商品登録の初期在庫を足し、51 §7.6 と Matrix の UI-01b-D22 の行に。
- P2（Codex #1）個数の単位へ替えたときの在庫連動の提案が `ej_unverified` の保存の検査に当たる。是正: 51 UI-01b-D22 の提案を全 12 単位で 1 つの規則に（`ej_unverified` の間は全単位で false）。
- P2（Codex #2）月次の商品別の数量が相殺で 0 になると種類を失う。是正: `MonthlySaleItem` に `stock_unit` を足し、行の単位で `0 m` / `0 点` を出す。
- P2（Codex #3）新しい数量の集計の wire に安全な整数の境界が無い。是正: `count_points`・`length_cm` の ±9007199254740991 の検査と、TS の再集計の `Number.isSafeInteger`。
- P2（Codex #4）合計の丸めの順序を誤る実装を落とす golden が無い。是正: `[16650, 50, 600]` centi → 173 円の golden を `cost_total_yen` を通る全経路の行に。
- P3（Fable）TD-203 の表示する所の表の行を持つ 66・67・75・53・74・36 に proposed の pointer を足した。
- 自己点検で直したもの: 部門別・合計の行が両方 0 なら `0 点` と明記、`cost_total_yen` を通る全経路で同じ golden を回す。

fresh broad（`1c64fc5a`）: Codex GPT-6 Astra（発注 257、PR review 5447378114）= reject（P2 2）、Claude 側 Fable 5.1（新しい context）= approve（P3 1）。P1 は 0。Coordinator が全件を採用した。helper に 2 本とも pending で記録済み（1 本目の run_ref は同じ run の本文の無い inline の review を指し、2 本目の evidence で訂正した）。

- P2（Codex #1）日次の商品別の数量に wire の安全な整数の範囲の検査が無い（合計が範囲内でも商品別の行が範囲外になりうる）。是正: 34 と 10 D3 で、商品別・部門・合計・月次の各段を wire にする前に検査する（`fe662ef6`）。
- P2（Codex #2）旧版へ戻す回復の手順で、DB を退避した後に遺物と backup を確かめる順序。是正: 確かめを `{db_path}` を動かす前に置き、止まったときに元の file が変わらない（Gated Amendment 2、`eaba9918`）。
- P3（Fable）wire の DTO を定義する 44・20・40 に proposed の pointer（`fe662ef6`）。

fresh broad（`12959e48`）: Codex GPT-6 Astra（発注 258、PR review 5447708867）= reject（P2 1 / P3 2）、Claude 側 Fable 5.1（新しい context）= approve（P3 4）。P1 は 0。Coordinator が全件を採用し、起草役が `8a2481d3`（Gated Amendment 3）で直した。helper に 2 本とも pending で記録済み。

- P2（Codex #1）m の入力の安全な整数の検査が、入庫の後の在庫につながらない（初期在庫 `90071992547409.91` m に `0.02` m を入庫すると number で 1 cm ずれる）。是正: 10 に「安全な整数の範囲」の表を足し、在庫の 5 経路が通る `apply_stock_change` が `quantity` と計算後の `stock_after` を書く前に検査する（31）。同じ型の穴を残さないよう、初期在庫・CSV の取込みの取消の補正・整合性の補正と detail_json・円の合計・手動販売の金額の初期値・売上の集計も同じ表に入れ、Matrix に 4 行を足した。
- P3（Codex #2）EJ の暗黙の数量・末尾の点数・記録全体の照合が申し送りに無い。是正: 29 の proposed の注記、申し送りの EJ の行、Matrix の SPEC-UNIT-D8 の行に、同じ 100 倍に揃える site と照合の例を足した。
- P3（Codex #3）Matrix の個数の商品の日次の明細の期待が `3`。是正: `3 個`（formatter の形）。
- P3（Fable #1）Gated Amendment 1 で触った正本の契約が Contract Ledger に無い。是正: SPEC-UNIT-D3 の行に 36・53・66・67・74・75 と SCREEN_DESIGN を足し、UI-02-D7・UI-03-D12・UI-05-D8・UI-06c-D5・UI-13-D3・UI-11c-D14 の行を足した。
- P3（Fable #2）SCREEN_DESIGN の生地の cm の記述 4 行に D-113 の pointer が無い。是正: proposed の括弧書きを足し、Scope の表に行を足した。
- P3（Fable #3）`integrity_check` の detail_json の mismatch に `stock_unit` が無い。是正: 36 の proposed の注記に足した（表示は今のまま）。
- P3（Fable #4）m の入力の `trim`。是正: 10 SPEC-UNIT-D3 の入力の文に前後の空白を除くことを足し、Matrix の m の入力の行に前後の空白の例を足した。
- Coordinator が現物で確かめたもの（2026-10-11）: 10 の表の前文の `rg` を回し、在庫を書く test 以外の site が表の 1〜5 のどれかに入ること（`apply_stock_change` の呼出しは入庫・返品・手動販売・廃棄・商品別売上 CSV の取込みの 5 か所、取消の補正 `csv_import_service/commit.rs:274`、整合性の補正 `integrity_service.rs:173`、初期在庫 `product_service.rs:247`・`:1416`、今の方式の棚卸しの確定 `stocktake_service.rs:571` は表の 5 の対象で保証は ⑤ の新方式）。表と申し送りが引く行番号（`inventory_service/common.rs:40`、`product_service.rs:957`・`:1204`〜`:1213`、`commit.rs:257`〜`:274`、`integrity_service.rs:77`・`:98`〜`:99`・`:165`・`:173`・`:191`〜`:192`、`sales_service.rs:203`・`:224`〜`:245`・`:549`・`:561`、`receiving_repo.rs:250`、`disposal_repo.rs:670`、`disposal-request.ts:40`）が現物と合うこと。
- Gated Amendment 4（`8fdda364`、Coordinator）: reviewer の指摘ではなく、base 同期の後に AC を回して見つけた。AC5・AC6 の diff の基点を固定の SHA から merge-base に直した（上の遷移の記録 15）。

fresh broad（`5d039c84`）: Codex GPT-6 Astra（発注 261、PR review 5480376565）= reject（P2 4）、Claude 側 Fable 5.1（新しい context）= reject（P2 1 / P3 3）。P1 は 0。Coordinator が全件を採用した。helper に 2 本とも pending で記録済み。是正は Gated Amendment 5（起草役の `0e4ab7bb`・`66c36019`）で、経路ごとの検査の表を、入力の上限の形に書き直した（上の遷移の記録 16）。

- P2（Codex F1）在庫を動かさない明細の数量が検査を通らない。是正: 1 明細の数量の上限を、在庫を動かす分岐より前の検証で掛ける（返品は `validate_return_request_shape`）。POS の非連動の行は取込みの preview で行の error。
- P2（Codex F2）移行の前に保存済みの範囲外の値が未検査。是正: vU が何かを変える前に、数量・在庫・売価・原価・円の合計の保存済みの値を上限で確かめ、外れた DB は無変更で止める（22 §16）。
- P2（Codex F3）・P3（Fable #4）読取りを範囲で拒んだ後の回復の契約が無い。是正: 読取りを範囲で拒む契約そのものをやめた。集計は上限 × 件数では届かない根拠を書き、演算は checked のまま。
- P2（Fable #1）入庫・廃棄の合計の検査が保存の前に無い。是正: 入庫・廃棄の create と棚卸しの確定が、保存の前に合計を検査する（上限からは範囲内と言えない所）。
- P2（Codex F4）Matrix の取込みの取消の境界の例の符号が逆。是正: 在庫の上限の例に置き換えた（在庫 999999999 で、数量 1 の販売の取込みを取り消す）。Coordinator は Gated Amendment 3 の照合でこの行の言い方の曖昧さに気付いたが、括弧書きで足りると判断して通していた。
- P3（Fable #2）m の入力の `0`・`0.00` の扱い。是正: 10 SPEC-UNIT-D3 に 1 文、Matrix に例。
- P3（Fable #3）56・57・29 の旧前提の文に proposed の pointer。是正: 括弧書きを足した。
- 起草役の判断で Coordinator が受けたもの: 変動の数量の移行の上限は在庫の上限の 2 倍（初期在庫と棚卸しの補正の変動が 1 明細の数量の上限を越えうる）。vU の検査に売価の列・`stocktakes.total_cost`・入庫と廃棄の記録ごとの合計を足す。POS 由来の上限外は取込み全体でなく行の error。合計が届く行数は確定した上限で計算し直した値（入庫・廃棄 901 行、棚卸し 10 商品）。
- Coordinator が差し戻して直させたもの: 原価の上限を原価の lane で入れる形（単位の lane だけの間は原価に上限が無い）を、単位の lane で円の上限を入れ、vU が原価の列も確かめる形に。これに合わせて、連続適用の失敗の例を、範囲外の原価から、vC の手順 4 の検証で止まる例に作り直した。
- Coordinator が現物で確かめたもの（2026-10-11）: vU の上限の検査が挙げる列名が schema の CREATE 文に全部あること、上限から導ける範囲の式のうち 1 行の金額・合計の 900 行と 901 行・棚卸しの 9 商品と 10 商品・集計の件数・価格改定の積を計算し直して一致すること。

Closeout（2026-10-11）: Final Review は 5 回で、毎回 Codex GPT-6 Astra と Claude 側 Fable 5.1 の独立 2 本。1 回目 = 2 本とも reject（Gated Amendment 1）、2 回目 = Codex reject・Fable approve（Gated Amendment 2）、3 回目 = Codex reject・Fable approve（Gated Amendment 3・4）、4 回目 = 2 本とも reject（Gated Amendment 5）、5 回目（head `7be82eec`）= 2 本とも approve（Codex P3 2、Fable P3 5。P1 / P2 は 0）。P1 は全回 0。3 回目・4 回目の P2 は、計算の後の値が JavaScript の安全な整数の範囲を越える経路の指摘で、同じ型が続いた。4 回目の後、owner の決定（2026-10-11）で、経路ごとに検査を書き足す形をやめ、入力に現実的な上限を掛けて範囲に届く筋を消す形に変えた（D-113 に記録済み）。5 回目の P3 は全件を本 lane では直さず、後続の lane の起票の入力として `docs/backlog.md` の「単位の拡張」の項へ送った: 61〜64 の明細表の「生地は cm 単位を主表示」の文への SPEC-UNIT-D3 の proposed の pointer、価格改定の既存の test（大きな値の保存の成功を固定）を新しい上限の中の保存と上限の外の拒否へ書き換えること、22 §16 の `schema_time_evidence.rs` の行番号のずれ、棚卸し記録詳細のロス原価の合計の範囲の検査と ⑤ の Matrix の fixture、棚卸しの実数の上限の担当を ⑤ に揃えることと範囲の検査の関数の共有（棚卸し ③ の実数 N の上限の書き換えを含む）、在庫少の基準（`app_settings`）を 10 の「表の外」に足すこと。helper の record は review = pass、manual = not-required、r4 = pass。Ready の後の `Merge gate` は success で、owner の承認（2026-10-11）で helper 経由の squash merge（PR #154）。
