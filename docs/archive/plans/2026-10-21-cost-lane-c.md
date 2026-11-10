# Plan Packet: 原価の lane（C）— 原価を 1/100 円の列にし、巻・cm を単位から外す起点の migration（vB）

wave に属さない単独の lane（導入の前の関門 1。`docs/Plans.md` の `## 導入の前の関門`、owner の決定は repo 外の回答台帳 TD-345）。owner の決定 TD-355（2026-10-21）で、lane 2a（PR #183、docs の file:行 の引用の書き換え）の Plan Review と並べて起票した。設計は [共通規則](../../function-design/10-common-rules.md) SPEC-UNIT-D1・D5（D-113・D-119）、[22](../../function-design/22-mnt-migration.md) §16 の vB（MNT-03-D14）、[master-tables](../../db-design/master-tables.md) の「単位と原価の精度の契約」、[51](../../function-design/51-ui-product-form.md) UI-01b-D24 にあり、本 lane はその契約を実装する。branch は `agent/cost-lane-c`（main `6a25f63f` から）。新しい決定は D-128（本 commit。下の「owner の判断事項」）。本 run は起票（packet・Matrix・plan-first の正本の直し）と Draft PR までで、runtime の実装は plan-approved の後の別の run。

**規模と手法（1 lane・1 PR のまま。実装は S1〜S7 の順に積み、S1〜S6 を 1 つの compile・回帰の検証の単位にする）**: 2026-10-21 に本 worktree（HEAD `6a25f63f`）で数えた（command と出力は下の AC の baseline と Scope の各表）。

- 触る設計正本: plan-first（本 commit）で 6 file（decision-log・10・22・51・master-tables・backlog と Plans の pointer）、実装の PR の S7 で 20 file 前後（下の S7 の表。数は S7 の command の出力で決める）。
- 画面: 原価を入力・表示する画面 9（商品登録・修正、価格改定、入庫と原価差分の dialog、廃棄、入庫・廃棄・棚卸しの記録詳細、商品一覧、在庫照会の詳細、価格の履歴）と、単位の select を持つ商品フォーム。新しい route・画面は無い。
- migration: 1（vB）。
- wire の型: 原価の field を持つ型 13（`CostDiff`・`DisposalItemInput`・`DisposalRecordDetailItem`・`ImportRow`・`PriceHistoryEntry`・`PriceRevisionInput`・`Product`・`ProductCreateRequest`・`ProductResponse`・`ProductUpdateRequest`・`ReceivingItemInput`・`ReceivingRecordDetailItem`・`StocktakeRecordDetailItem`）と、単位の enum `ProductStockUnit` の 1。command の名前と引数の数は変えない。内部の型（wire に出ない）は 6（`NewProduct`・`ProductUpdates`・`NewPriceHistory`・`NewReceivingItem`・`NewDisposalItem`・`ProductForStocktake`）。
- 横断の約束（範囲・0・失敗の時点と順序・単位や種類の混在）がかかる経路は 12: 商品の create・update・価格改定・商品 CSV の preview と commit、入庫と廃棄の create、入庫の原価差分の検出、入庫・廃棄・棚卸しの記録詳細の読取り（行の金額と合計）、価格の履歴、在庫少の単位の並び、migration（vB）。ほかに test だけが呼ぶ旧本体（棚卸しの確定、`#[cfg(test)]` の時点証拠の確定）が同じ原価の関数を通る。下の「横断の約束」の表に Y1〜Y10 で置く。
- 既存の code の hit（S1〜S6 の作業量の目安）: 原価の旧い名前を持つ Rust 34 file・TS 46 file、巻・cm の variant と文字列を持つ Rust 18 file・TS 36 file（Scope の各表の command）。

分け方の判断（`1 lane のまま段に分ける` と `lane を分ける〈C1 原価の centi と vB、C2 巻・cm の除去〉` を比べた）: 分ける条件は、前の段だけが merge された main で (1) 店の画面と保存の値が壊れない (2) DB の列の名前と意味が食い違わない (3) 全 gate が緑、の 3 つ。巻・cm の除去は vB の手順 4（`products` の作り直しで CHECK を 10 個にする）で、22 §16 と D-119 (2) は「外すのはその起点だけで、今の DB に新しい migration で外すことはしない」と決めている。よって、vB を持つ lane が巻・cm の CHECK も持つ。C1（vB の全部と原価）を先に入れると、CHECK が `roll`・`cm` を拒むのに enum と画面の select が 12 個のまま残り、巻・cm を選んだ商品の保存が DB の CHECK の error で落ちる（(1) が破れる）。C2（enum と画面から巻・cm を外す）を先に入れるには vB が要り、vB は原価の 6 列の改名を同じ migration に持つので、原価の code が古い列名のまま壊れる（(3) が破れる）。vB を「改名だけ」と「CHECK だけ」の 2 本に割ると、作り直しの起点の migration が 2 本になり、D-119 (2) と、作り直しを要する起点を初回の導入の版に全部入れる条件（D-123 (4)）の範囲を広げる。原価の列の改名と意味の変更（1/100 円）を分ける切れ目は、改名だけが入った main で `_centi` の列に円の値が溜まり、後の段で値の変換を持たない vB の前提（D-119 (1)）が破れる。どの切れ目でも (1)〜(3) のどれかが破れるので分けない。初回の導入の版に vB が入る条件（導入の前の関門 1）は、1 lane のまま満たす。

1 PR の大きさの目安: 書き換える test の多くは機械的な追随（struct の field 名の `_centi`、SQL の列名、円の fixture の値を 1/100 円の値へ、`cm` の fixture を `m` へ）で、振舞いの判断を含む新しい test は S1（vB）・S3（原価の上限と CSV の解析）・S5（原価の入力・表示・補助）・S6（10 個の一覧）に集まる。既存の test の分類（書き換える / 変えない / 期待を変える）と数える command は [Matrix](test-matrices/2026-10-21-cost-lane-c.md) の「既存の test の移し方」（T1〜T5）の 1 か所に置き、Scope の各段はそこを引く。

## Workflow State

- Phase: archive
- Risk: R3
- Plan Commit: a0f622ebe81621121ea64fd2ba1caf742c7332b5
- Amendments: none
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（起票は fresh context の Writer。実装の Writer は plan-approved の後に Coordinator が発注する）
- Plan Reviewer: fresh Opus 5.5 + Codex（model は発注時に決める）。座組表どおり
- Final Reviewer: Fable 5.1（Claude 側、R3 以上）+ Codex（GPT-6.1 Sol 既定）。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Review Minimum: 2
- Human Gate: ready,merge,manual
- Branch: agent/cost-lane-c

遷移の記録:

1. kickoff → spec-check（2026-10-21、起草役）: Coordinator の発注（導入の前の関門 1、TD-355）を Scope にし、規模を数えて 1 lane・1 PR のままと決め、Risk を R3 と記録した（下の Risk）。
2. spec-check → design（2026-10-21、起草役）: 対象の設計正本は SPEC-UNIT-D1〜D3・D5・D6・D9〜D11 と「入力の上限と安全な整数の範囲」、22 §16（vB・作り直しの手順）と §19、master-tables の「単位と原価の精度の契約」、51 UI-01b-D22〜D24、61 UI-02-D7・D15、64 UI-05-D8、77 の新原価（案）、30・31・35 の原価の注記。設計の更新が要る点が 6 つあった（下の Design Readiness の「必要な設計成果物」）。
3. design → plan-draft（2026-10-21、起草役、本 commit）: 上の 6 つを本 packet と同じ commit で正本に置いた（下の「本 commit で変えた正本」）。owner に聞く設計の問いは残っていない（下の「owner の判断事項」）。plan-gate へは Coordinator が進める。
4. plan-draft（round 1 の是正）: Plan Review round 1 の是正（Review Response 参照）。
5. plan-draft（round 2 の是正）: Plan Review round 2 の是正（Review Response 参照）。
6. plan-draft（round 3〈天井〉の是正）: owner の決定（repo 外の回答台帳 TD-367）で、指摘を直して closure で確かめる（Review Response 参照）。
7. plan-draft → plan-gate（Plan Review round 1〜3、2026-10-21）: Claude 側 fresh Opus 5.5（round 1 は全体と網羅の深掘り、round 2 は全体、round 3 は全体と既存 test の移し方の深掘り）・Codex（3 round とも Astra）。各 round の是正は Review Response 参照。
8. plan-gate → plan-approved（2026-10-21、Coordinator、本 commit。state だけ）: owner の決定（repo 外の回答台帳 TD-367）どおり、round 3 の是正（`a0f622eb`）を fresh Opus 5.5 と Codex（Sol）の closure が 2 本とも approve（P1/P2 0）した。closure の P3 2 件（S5 の廃棄の行の「今のまま」の句、T2b の例の 1 file）は契約を一意に読めるので packet を直さず、実装の発注で Writer に渡す。Plan Commit は `a0f622eb`。
9. plan-approved → implementing（2026-10-21、Coordinator、本 commit。state だけ）: Plan Commit `a0f622eb` は実装の commit より前にある。実装は Writer（Opus 5.5 subagent）への発注で行う。

本 commit で変えた正本（plan-first。round 1 の是正の commit で足した所は各項目の末尾の「round 1 の是正」）:

- `docs/decision-log.md`: D-128（原価は税抜、巻・cm は vB で必ず外す、10 cm は刻みで検査は店主の答えの後）と、D-119 の `Superseded in part by: D-128` の行。round 3 の是正: D-128 の Guarantee range の現掛率を比の定義（原価〈税抜〉÷ 売価〈税込〉）で書いた（決定の中身は変えない）。
- `docs/function-design/10-common-rules.md`: SPEC-UNIT-D1 の巻・cm の前提が成り立ったこと、D2 の理由、D5 の税の区分（未決 → 税抜）と表示の記号・入力の欄の初期値の文字列、更新履歴。round 1 の是正: D5 の表示の記号を半角の `¥` に一意にした。D10 の商品 CSV の `原価` の行の error の分け方と、`在庫単位` の `巻`・`cm` の行の error に直し方を添えること。round 2 の是正: D5 の入力の欄の初期値の場所に価格改定の 2 つ。
- `docs/function-design/22-mnt-migration.md` §16: vB の番号の決め方、手順 2 の止まりの文（原因と直し方を名指す）、手順 4 の条件の分岐を外して §19 の写しの 3 列と §18 の規則を足したこと、止まった後の起動の画面、「確かめていない前提」（WebView の保存の置き場と確かめ方）、vB の test の文。round 1 の是正: vB の test の文の手順 4 の後の `products` の定義の比べ方。
- `docs/function-design/51-ui-product-form.md`: UI-01b-D22 の原価の lane の後の label と価格の節の説明、UI-01b-D24 の補助を開き直すと空から始まること、更新履歴。
- `docs/db-design/master-tables.md`: `cost_price` の税の区分（税抜）、巻・cm の前提、vB の番号の決め方。
- `docs/backlog.md`・`docs/Plans.md`: 原価の lane の項目と導入の前の関門 1・次の行動に packet への pointer と、揃った前提（U6・U1）の 1 句。lane の行は足さない（D-097）。round 1 の是正: backlog の保留に 2 項目（商品 CSV の preview で error の行をその場で直す経路、入庫・廃棄・価格改定の原価の欄の見出しの税抜。どちらも出所は Plan Review の AI review）。

## Owner Effort Budget

- 介入回数上限: 6（既定）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 6 | 3（closeout 時点。Plan Review の round 天井の判断 1〈TD-367〉、L3 1〈owner の L3 用の DB の作り直しを含む 1 round、TD-370〉、Ready と merge 1〈TD-371〉。TD-355 は本 lane の前の判断で数えない） | 0 | 3 | 6 = 3 + 0 + 3 |

plan-approved は Coordinator が進める（TD-296 の 4a）。L3 が 2 round 目に入るか、Plan Review が round 天井に達するときは、その判断と上限の改定を同じ 1 回で諮る。owner は 2026-10-21 から数日は手が使いにくいので、L3 は Draft のまま溜め、作り直しの file の移動は owner の指示で Coordinator が行う（下の Data Safety）。

既定値・数え方・上限に届くときの扱いは `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
DB の列（原価の 6 列の改名、`products` の作り直しで `stock_unit` の CHECK を 10 個に）、Tauri command の DTO と generated bindings（原価の field を持つ 13 型、`ProductStockUnit`）、operator の入力（原価の小数 2 桁、袋の値段 ÷ 入り数の補助、単位の select）を変える（`docs/DEV_WORKFLOW.md` Risk Tiers の R3 の行）。vB は原価の列を持つ 5 つの表に行の無い DB にだけ当たり、値を書き換えない（行のある DB は何も変えずに止まる）。止まった開発・試験 DB の作り直しは、22 §16 の作り直しの手順のとおり古い file を消さずに日付の付いた folder へ移す（D-123 でリリースの前まで包括で承認済み）ので、不可逆な local の片付けにならない（R4 に当たらない。backlog の項目の最後の文の条件）。本番の DB は無い（`docs/project-memory.md`）。実店舗データの露出は無い。

required gate の green / red: gate の定義は変えない。実装は bindings と traceability の再生成を伴い、落とせば既存の required gate（Rust・bindings・traceability・frontend）が red になる（下の Registration / Generation Obligations）。S7 の正本の同期で `docs/project-profile.md`（`scripts/ci/classify-changes.sh` の workflow の一覧に入る file。今の「Fabric stock can use `cm`」の 1 行が巻・cm を外した後の事実と合わなくなる）を直すので、実装の PR の分類は `workflow=true` になり、共有の workflow の suite が走る。よって Final Review Minimum 2（workflow gate の Double Audit）。plan-first の本 commit は `docs/project-profile.md` に触れない（下の検証の分類の出力）。

## Goal

Goal Invariant:

### 最小完了条件

- 新しい版を、商品と原価の記録の無い DB（作り直した開発・試験 DB、初回の導入の空の DB）で起動すると vB が当たり、原価の 6 列が `_centi` の名前になり、`products.stock_unit` の CHECK が個数の単位 9 個と `m` になる。商品か原価の記録のある DB では何も変えずに止まり、起動の画面が原因と直し方（作り直しの手順）を名指しする（22 §16 MNT-03-D14、D-127 (2)）。
- 原価を持つ全ての入口（商品の登録・修正・価格改定・商品 CSV、入庫・廃棄）が 1/100 円の整数を `0`〜`99999999` で受け、画面は小数 2 桁までの円で入力・表示し（`83.33` → 8333 → `¥83.33`）、入庫・廃棄・棚卸しの行の金額と合計は 1/100 円の原価から求める（SPEC-UNIT-D5・D6）。
- 商品フォームの原価の欄に「袋の値段から出す」の補助があり、個数の単位の商品で `1000` 円 ÷ 12 → `83.33` を入れる（UI-01b-D24）。原価の label は税抜（D-128）。
- 単位の一覧が 10 個（個数の単位 9 個と `m`）になり、enum・bindings・formatter・商品 CSV の `在庫単位`・商品フォームの select・在庫少の単位の並び・seed が同じ一覧を使う（SPEC-UNIT-D1・D10）。

### 失敗定義

- vB が行のある DB で何かを変える、止まったときに原因と直し方を出さない、§19 の写しの 3 列・CHECK・index・外部キーのどれかを作り直しで落とす。
- 円の値が 1/100 円の列・wire に入る経路が残る（変換なしで円の整数を `_centi` に渡す、`i128::from(円) * 100` の呼出しが原価の列の値に残る）、または 1/100 円の値を円として出す画面が残る（`¥8,333` と出る）。
- 浮動小数を通る原価の解析・補助の割り算が残る（`0.29` → 28、`1000` ÷ 12 → 83.34）。
- `roll`・`cm` を受ける経路（enum の variant、bindings、商品 CSV の語、select の選択肢）が残る、または DB の CHECK と enum の一覧が食い違う。
- 既存の test を、期待を弱めて通す（Matrix の T1〜T5 の「期待を変える」以外で `assert` の右辺・件数・message を変える）。

### 非目的

- 売価の精度（円の整数のまま。SPEC-UNIT-D5 の不採用 (d)）。在庫・数量の契約（SPEC-UNIT-D2〜D4 は変えない。10 cm の刻みの検査は店主の答え〈Q-098〉の後）。
- 棚卸しの評価額の出力の文に税抜と書くこと、棚卸し記録詳細のロス原価を BIZ へ移すこと（D-118 の (b)。本 lane は単位を 1/100 円に合わせるだけ）。
- 初期投入の手順（backlog の「初期投入の手順」）。Z004・EJ の数量（P、SPEC-UNIT-D8）。
- 既存の file:行 の引用の書き換え（lane 2a、PR #183）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

設計を含む変更の実装なので、開発の DB の更新と、店の通常の操作列を置く。例の値は説明用の合成の値。本 lane の merge の後に下の列が成り立つ。棚卸し（計数と確定）は停止中のまま（D-118 の (b) の後）で、**この文書を完了できる**ことと、**棚卸しの評価額を税抜と書いて出す通常運用を達成できる**ことは別（後者は (b)）。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 商品の行のある開発・試験 DB（owner の L3 用の DB） | 新しい版を起動する | 起動が止まり、「データベースの準備に失敗」の文に「商品か原価の記録がある DB はこの版へ移せません。原価を 1/100 円で持つ版は空の DB から始めます。開発・試験 DB は作り直してください（22 §16 の作り直しの手順）: products・…」が添えられる。DB は変わらない | 作り直しの手順（アプリの data の folder を `%APPDATA%`・`%LOCALAPPDATA%` とも丸ごと消さずに移す）の後に起動すると、空の DB で開く | 22 §16、D-123、Contract Probe P1・P5 |
| 空の DB（作り直しの後） | 商品登録で単位 `玉`・売価 `100`・原価 `83.33` の商品を登録する | 保存できる。商品一覧・在庫照会の詳細の原価が `¥83.33` | 保存できる | SPEC-UNIT-D5 |
| 商品登録の画面（単位 `個`） | 原価の下の「袋の値段から出す」を開き、袋の値段 `1000`・入り数 `12` で「原価に入れる」 | 原価の欄が `83.33` になる。保存の request に袋の値段と入り数は入らない。閉じて開き直すと 2 つの欄は空 | 保存できる | UI-01b-D24 |
| 商品登録の画面 | 単位の select を開く | 個・枚・本・袋・箱・組・セット・玉・丁・m の 10 個（巻・cm が無い）。`m` を選ぶと補助が閉じて隠れ、原価の label が `原価（税抜、必須、1 m あたり）` | — | SPEC-UNIT-D1、UI-01b-D22 |
| 上の `玉` の商品 | 入庫（数量 `3`・原価 `83.33`）を保存する | 記録詳細の原価 `¥83.33`・原価小計 `249.99 円`・原価合計 `¥250` | 保存できる | SPEC-UNIT-D6 |
| 同じ商品 | 入庫（数量 `1`・原価 `85`）を保存する | 保存の後に原価差分の dialog（マスタ原価 `¥83.33`・今回の実原価 `¥85`）。「マスタ原価をこの実原価に更新する」で商品の原価が `¥85`、価格の履歴に `¥83.33 → ¥85` | — | UI-02-D15、31 §12.3 |
| 同じ商品（原価 `¥85`・売価 `100`） | 価格改定で新売価 `110` を入れる | 現掛率 `85.0`、新原価（案）`93`（`floor(110 × 8500 ÷ 10000) × 100` = 9300） | 確定できる | 77 の新原価（案） |
| 商品 CSV（原価 `83.33`・在庫単位 `玉` の行と、在庫単位 `巻`・`cm` の行） | 取り込む（preview） | 前の行は原価 8333 で取り込める。後の 2 行は `在庫単位` の行の error で、文が直し方を添える（`cm` は m で、`巻` は 個 で入れる） | — | SPEC-UNIT-D10 |

失敗と再試行: 原価の欄の入力が `^[0-9]+(\.[0-9]{1,2})?$` に合わない（小数 3 桁、指数、符号、空）か上限 `999999.99` を越えると、その欄に入力の error（`原価は0以上で、小数は2桁までで入力してください`・`原価が大きすぎます`）を出し、保存の command を呼ばない。UI を通らない command は BIZ が同じ上限で `ValidationFailed` にし、何も書かない（各入口の今の検証の順のまま。TX の中なら TX 全体を戻す。Y1）。vB が止まった DB は何も変わらないので、作り直すまで旧版でそのまま開ける（その起動の最初の migration で止まったとき）。

Plan Review は、この列が「正常な条件で目的を達成できるか」と「危険な結果を出さないか」を別々に答える（`docs/DEV_WORKFLOW.md` Review Rules）。

## owner の判断事項

無い。製品の振舞いの判断は D-113・D-119・D-123 と D-128（店の答え TD-331・TD-332、owner の決定 TD-345・TD-355）で決まり、正本にある。本 lane で owner が答えるのは Human Gate（L3〈owner の L3 用の DB の作り直しの指示を含む〉、Ready、merge）だけ。

前提の答え（claim として使う。Coordinator の発注の要旨）: U6（原価の税の区分）= 税抜き（TD-331、2026-10-20）。U1 = 2026-10-18 版の在庫 Excel でも巻 0 行・cm 0 行（repo 外の調査、2026-10-20）。開発・試験 DB の作り直しは承認済み（TD-297、古い file は消さずに移す。D-123 でリリースの前まで包括）。

起票時に起草役が決めたこと（設計の値を変えない配布の割当てと細部。Coordinator が覆せる。正本へは本 commit で置いた）:

- D-128 を新しい決定にした（D-119 に置換えの行を足すだけにしなかった）理由: U6 の答えは D-119 (5) の既定（原価の lane を起票しない）を置き換え、U1 の答えは (2) の前提を満たし、10 cm の刻みの答えは (4) の理由を置き換える。3 つを 1 つの決定にまとめ、10・22・51・master-tables が 1 つの ID で引けるようにした。
- vB の手順 4 の条件の分岐を外した（D-127「継ぎ足さない」の問い「この変更で不要になる記述・条件は無いか」）。同じく不要になった記述: SPEC-UNIT-D1 の「起票の前に確かめる。0 でなければ外さず聞き直す」、D5 の「答えが無い間は原価の lane を起票しない」、master-tables の「税抜か税込かは決めていない」、Plans の「残る前提は U1」、backlog の「起票時に決める」の 2 件（決めた先を書いた）。
- 止まりの文（22 §16 手順 2）を §19 と同じ形で名指しにした（D-127 (2)。22 §16 は `作り直し` と表名を含めるとだけ書いていた）。
- 原価の表示の記号（D5。round 1 の是正、Review Response 参照）: 正本の例は `1,000 円` の形だった。今の画面の原価の表示は 3 つの形が混ざる: 半角の `¥`（`src/features/inventory-records/types.ts` の `formatYen`〈記録詳細の 3 画面〉、`src/features/products/components/PriceRevisionTable.tsx` の `yen`、`src/features/disposal/DisposalPage.tsx` の `formatYen`）、全角の `￥`（`Intl.NumberFormat` の `currency: "JPY"`: `src/features/products/components/ProductTable.tsx`・`src/features/products/components/PriceHistorySection.tsx` の `yenFormatter`、`src/features/stock-inquiry/components/StockDetailContent.tsx` の `priceFormatter`）、`1,000 円`（`src/features/receiving/CostDiffDialog.tsx` の `formatYen`）。数の規則（割り切れれば円の整数、そうでなければ小数 2 桁）と記号（半角の `¥`。多数派の形。Coordinator の判断、デザインは owner から理論で委任）を D5 の契約にし、原価を出す全ての画面を 1 つの formatter に揃える。全角と `円` の形の 4 画面は表示が変わる（Matrix の T5）。売価・合計など原価でない金額の記号は本 lane で変えない（Non-scope）。入力の欄の初期値の文字列（`83.33`）も同じ数の規則で決めた。
- UI-01b-D24 の「開き直すと空」: D-119 の Matrix にだけあった期待（backlog の「起票時に決める」(1)）を正本に置いた（保存しない値を画面に残すと、別の商品の袋の値段で原価を入れうる）。
- 棚卸し記録詳細のロス原価（今は画面が `|差異| × 評価原価` を number で掛ける。35 §20.5a の既知の不整合）: 本 lane は単位だけを合わせ、`BigInt(|差異|) × BigInt(評価原価(1/100 円))` を 1/100 円の十進の文字列にして入庫の記録詳細と同じ `formatCentiYen` で出す（1/100 円の後は number の積が安全な整数の範囲を越えうる: `99999999 × 999999999`）。ロスの無い行（差異が 0 以上か評価原価が無い。今は 0 として `¥0`）も同じ形で `0.00 円` と出す（round 3 の是正。入庫・廃棄の記録詳細の行の金額の 0 と同じ形）。基準数量で割ることと BIZ へ移すことは (b)。正本へは S7 で 35 §20.5a の「範囲の外」の段に 1 文を足す。

店主に新しく聞く必要がある事項: 無い（10 cm の刻みを検査するかの問い Q-098 は本 lane の振舞いに効かない）。

## 横断の約束（DB・wire に出す値ごとに、どこが保証するか）

symbol は 2026-10-21 の HEAD `6a25f63f` の現物。Contract Ledger と Matrix はこの表の # を引く。

| # | 値（DB の列 / wire） | 範囲と、保証する所 | 0 と符号 | 失敗の時点と順序 | 単位・種類の混在 |
|---|---|---|---|---|---|
| Y1 | 原価（DB の 6 列 `*_centi`、wire の 13 型の原価の field） | 1/100 円の整数、`0`〜`99999999`（`constants.rs` の新しい定数 `MAX_COST_PRICE_CENTI`。今の `MAX_COST_PRICE` 円 `999_999` を置き換える）。BIZ の検証: `product_service.rs` の `validate_create_request`・`validate_update_request`・`revise_product_price`・商品 CSV の preview と commit（`price_range_error` の原価の呼出し）、`receiving.rs`・`disposal.rs` の明細の検証。画面も同じ上限（`src/lib/unit-quantity.ts` の新しい定数 `MAX_COST_PRICE_CENTI`） | 0 を受ける。負は今の文（`原価は0以上で入力してください`・明細の文）で拒む | 各入口の今の検証の位置と優先順位を保ち、原価の検証に届いたときに `ValidationFailed`（`原価が大きすぎます`・`明細{n}: 原価が大きすぎます`）で拒む。書込みは行わない。原価の検証より先にあるものは今のまま先: 商品の修正と価格改定は商品の取得（無ければ `NotFound`。[30](../../function-design/30-biz-product-service.md) §4.4 の手順 1）、入庫と廃棄は業務日の検査と冪等性の判定（同じ冪等キーで内容が違えば `IdempotencyConflict`。[31](../../function-design/31-biz-inventory-service.md) の入庫・廃棄の処理ステップ 0・1）と、明細ごとの数量の検査（明細ごとに数量 → 原価 → 商品の取得の順）。商品の登録は商品名の検査の後で、部門・取引先の存在の検査より前（今の `validate_create_request` の順）。商品 CSV の preview は行の error の列の順、commit は TX の中で、新規の行は商品を引かずに、上書きの行は既存の商品を引いた後に `import_row_commit_error` で再検証して TX 全体を戻す（BIZ-01-D8。上書きの行で商品が無いものは再検証せずに skip するので、commit の上限の検査は新規の行か既存の上書きの行で確かめる: Matrix の C3）。棚卸しの旧本体は DB の評価原価を読むだけで入力の検証は無い | 原価は価格の基準数量あたり（個数 1、`m` は 1 m）。単位に依らず同じ上限 |
| Y2 | 原価の入力の文字列 → 1/100 円 | `^[0-9]+(\.[0-9]{1,2})?$`（前後の空白を除いた文字列）を、浮動小数を通さず整数部 × 100 + 小数部（右を 0 で埋めた 2 桁）にする。同じ規則の既存の関数を使い、新しい解析を書かない: TS は `src/lib/unit-quantity.ts` の `readQuantity` の `m` の分岐（m の数を cm の整数にする）を単位に依らない関数に切り出し、`parseCostInput` と数量の解析が共有する（商品フォーム・価格改定・入庫・廃棄・補助の袋の値段が使う）。Rust は `src-tauri/src/biz/product_service.rs` の `parse_meters_to_cm`（同じ規則を `i64` の checked の演算で。単位に依らない名前へ改める）を商品 CSV の `原価` の列にも使う（商品 CSV の importer が値の前後の空白を除いて渡す） | `0`・`0.00` は 0 で受ける。商品 CSV の `-0` は下の負の文（今の `parse::<i64>` は 0 で受ける） | 合わなければ丸めずに止める。TS はその欄の入力の error（SPEC-UNIT-D5 の文 `原価は0以上で、小数は2桁までで入力してください`）。商品 CSV の行の error は 2 つに分ける: 先頭が `-` で残りが規則に合えば今の負の文 `原価は0以上で入力してください`（`-1`・`-0`。今の preview の `price_range_error` の文）、それ以外の形の誤り（小数 3 桁、指数、`+`、`.5`、`5.`、空）は D5 の文（今の `原価が不正です` を置き換え、直し方を名指す。SPEC-UNIT-D10）。規則に合うが `i64` に収まらない値（`99999999999999999999` 等）は上限を越える値として `原価が大きすぎます`（TS も同じ: 上限より大きい数として `"too_large"`）。行の中の error の並び（列の順）は今のまま | — |
| Y3 | 原価の表示 | 割り切れれば円の整数、そうでなければ小数 2 桁、記号は半角の `¥`（SPEC-UNIT-D5）。TS の新しい formatter 1 つ（`src/features/stock-inquiry/lib/format-stock-display.ts` の今の `formatCentiYen` の隣）を原価を出す全ての画面が使う。入力の欄の初期値は同じ規則で記号と桁区切りを付けない | `¥0` | 読取りは範囲で拒まない（共通規則 (d)） | 同じ画面の売価は今の記号のまま（Non-scope） |
| Y4 | 行の金額（`line_cost_centi`・`line_loss_cost_centi`、十進の文字列）と円の合計（`total_cost`・`total_loss_cost`） | `biz::unit_amount::cost_line_centi` に DB の 1/100 円を `i128::from` で渡す（今の `i128::from(円) * 100` の 5 か所: `list.rs` の 2、`receiving.rs`・`disposal.rs`・`stocktake_service.rs` の各 1。`#[cfg(test)]` の `time_evidence.rs` も同じ）。合計は `cost_total_yen` と `check_safe_integer`（今のまま） | 0 は 0 | 合計が `±9007199254740991` の外なら保存の前に `ValidationFailed`（`合計金額が大きすぎます`、今のまま） | 基準数量で割る（今のまま） |
| Y5 | 袋の値段 ÷ 入り数（UI だけ） | 袋の値段は Y2 の規則、入り数は 1〜`9999999` の整数。`P ÷ n` を商 q・余り r で `2r >= n` なら q + 1（TS の純関数 `unitCostFromPack` 1 つ。割り算は既存の `src/lib/unit-quantity.ts` の `divideRoundHalfUp`〈同じ `2r >= d`〉を使い、袋の値段の解析は Y2 の関数）。結果は P 以下なので Y1 の上限を越えない | 入り数 0 は error | 入力の error で原価の欄を変えない | 個数の単位の商品だけに出す |
| Y6 | 原価差分（`CostDiff` の `master_cost_price_centi`・`received_cost_price_centi`） | 1/100 円どうしの完全一致で比べる（31 §12.3）。COMMIT の後に検出し、保存に影響させない（今のまま） | — | 読取りの失敗は空の配列（今のまま） | — |
| Y7 | 入庫・廃棄の fingerprint の item 行の原価 | 1/100 円の整数の十進の文字列（31 §12.8。値の意味だけが変わり、組み立ての code は field 名の追随） | — | — | — |
| Y8 | 新原価（案）と現掛率（価格改定の画面） | 新原価（案）= `floor(新売価 × 現原価(1/100 円) ÷ (現売価 × 100)) × 100`、現掛率 = `現原価(1/100 円) ÷ (現売価 × 100)` を % の小数 1 桁（77）。number の積は `999999 × 99999999` 以下で安全な整数の範囲（共通規則 (b)） | 現売価 0 は今どおり（現原価を初期値、掛率 `—`） | 案が上限を越えたら欄の error（今のまま） | — |
| Y9 | 単位の code（`products.stock_unit`、wire `ProductStockUnit`） | 10 個（`pcs` `sheet` `hon` `bag` `box` `kumi` `set` `ball` `cho` `m`）。DB の CHECK（vB の手順 4）と enum `ProductStockUnit::ALL` が同じ一覧（vB の test で止める）。商品 CSV の `在庫単位` は 10 個の code と 10 個の表示の語、在庫少の並びは `ALL` を `stock_unit_kind` で 9 と 1 に分ける | 空の `在庫単位` は `pcs`（今のまま） | 一覧に無い値は、CSV は preview の行の error（`巻`・`roll`・`cm` は直し方を添える: S6）、DB の読取りは `parse_stock_unit` の error（今のまま） | 個数 9・長さ 1 |
| Y10 | vB の起点の検査 | `products`・`receiving_items`・`disposal_items`・`stocktake_items`・`price_history` を別々に数え、1 行でもあれば止まる（22 §16 手順 2） | — | 何かを変える前に ROLLBACK し、`foreign_keys` を戻し、版を記録せず `MigrationFailed`（手順 2 の文と行のある表名の全部） | — |

## Scope

本 lane が変える file と目的（symbol は 2026-10-21 の HEAD `6a25f63f`）。実装は S1 → S7 の順に積むが、S1〜S6 は 1 つの compile・回帰の検証の単位にする。理由: S1 で列の名前を変えると S2 で SQL を直すまで原価を読む test が全部落ち、S4 で wire の field 名を変えると S5 で TS を直すまで typecheck が落ち、S6 で enum から variant を外すと fixture を直すまで compile が通らない。途中の段は Test Plan の targeted tests のうち、その段の module の test で進み具合を見る。S6 の終わりで AC1〜AC10 を全部通し、S7（docs）の後に AC9 の gate をもう一度通す。既存の test をどう移すかは Matrix の「既存の test の移し方」（T1〜T5）を引く。着手時に各表の command を逐語で回し、表と違えば表を直してから進める（2026-10-17〜19 の数え方と今の base の差を埋めた先例: 下の T1）。

### 設計正本（本 packet の commit で済み）

上の「本 commit で変えた正本」。S7 で、実装した範囲の `proposed・未実装` を現行の記述に直す。

### S1 migration vB

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| migration vB | 22 §16 の vB の手順 1〜6（MNT-03-D14）。関数は `pub(crate)`。`foreign_keys` の退避と復元、COMMIT・ROLLBACK は共有の helper（`db/migration_tx.rs` の `commit_transaction`・`rollback_after_error`・`restore_foreign_keys`）だけを使う（MNT-03-D1）。手順 4 の `products_new` は vB の時点の `products`（v9 の後: v8 の列と §19 の写しの 3 列、`stock_range_high` の列の CHECK）を写し、`stock_unit` の CHECK を 10 個にする。index は `idx_products_jan_code`・`idx_products_department_id`・`idx_products_is_discontinued`（と、その時点で `products` にあるほかの index。着手時に `rg -n 'ON products' src-tauri/src/db` で数え直す）を作り直す。§18 の migration が先に merge されていれば §18 の規則（trigger の消去と作り直し）を入れる | `src-tauri/src/db/schema_v{N}.rs`（新設。N は merge の時点の次の番号。起票の時点の最大は v9: `db/migration.rs` の `migrations()` の `schema_v9::apply_v9_stock_ledger`）、`src-tauri/src/db/mod.rs`（module の宣言）、`src-tauri/src/db/migration.rs` の `migrations()`（登録。description は `原価を1/100円の列へ、単位から巻・cmを外す（作り直しの起点）`） | MNT-03-D1 の構造の test（`migration.rs` の、migration の file を `include_str!` で列挙して共有の helper だけを使うことを確かめる test）の列挙に新しい file を足す。`expected_tables` は変えない（表を足さない） |
| 既存の migration の test（版の期待・migrate の呼び方・開き直し・CHECK の一覧） | Matrix の T1 のとおり（新しい番号へ書き換える 12 行、`migrate_up_to(&conn, 9)` に替えて期待を変えない 8 か所、呼び方か期待を変える 2 か所、変えない所）。v8 の履歴の test の 2 つは T4。`migrate_up_to` は (a) の lane が足した test だけの helper（`migration.rs` の `pub(crate) fn migrate_up_to`）をそのまま使う | `src-tauri/src/db/migration.rs`・`src-tauri/src/db/schema_v2.rs`・`src-tauri/src/db/schema_v8.rs`・`src-tauri/src/db/schema_v9.rs` の test | 並走の §17（PR #172）・§18 の lane も版の期待の行を書き換える（下の所有表） |
| 新しい test | Matrix の B1〜B10（B9 は §18 が先のとき。B は定数 `VB_VERSION` から vB の前の版を作る） | `schema_v{N}.rs` の test | — |
| 作り直しの手順の起動の test | 22 §16 の「テスト」の作り直しの手順の (i)(ii)(iii) と対照（一時の `{app_data}` と作業フォルダを `prepare_database_with_init` に渡す）。Matrix の R1〜R4 | `src-tauri/src/lib.rs` の test（`prepare_database_with_init` を呼ぶ既存の test の隣。着手時に `rg -n 'prepare_database_with_init\(' src-tauri/src/lib.rs` で置き場を決める） | — |

### S2 DB の列名と内部の型（repo・seed・test の support）

数える command（着手時に逐語で回す）: `rg -l -w 'cost_price|valuation_cost_price|old_cost|new_cost|old_cost_price|new_cost_price|master_cost_price|received_cost_price|MAX_COST_PRICE' src-tauri/src src-tauri/tests | sort`（起票の時点で 34 file）。

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| SQL の列名 | 6 列を `_centi` で読み書きする | `src-tauri/src/db/product_repo.rs`（`products.cost_price`、`price_history.old_cost`・`new_cost`）、`receiving_repo.rs`・`disposal_repo.rs`（明細の `cost_price`）、`stocktake_repo.rs`（`valuation_cost_price` と `ProductForStocktake` の読取り）、`sales_repo.rs`（test の商品の INSERT） | 各 repo の test。`#[cfg(test)]` の `db/schema_time_evidence.rs`（`stocktake_items` を作り直す SQL が `valuation_cost_price` を写す。vB の後の DB に当てるので `_centi` へ）と `db/test_support.rs` |
| 内部の型の field | `NewProduct.cost_price`・`ProductUpdates.cost_price`・`NewPriceHistory.old_cost`/`new_cost`・`NewReceivingItem.cost_price`・`NewDisposalItem.cost_price`・`ProductForStocktake.cost_price` を `_centi` に | 上の repo の struct 定義 | 構築する所: `rg -n -w 'cost_price:' src-tauri/src src-tauri/tests`（起票の時点で 81 行・22 file。production の構築と test の fixture の両方）。値は円の fixture を 1/100 円の値へ書き換えるか（期待の金額が変わる所は Matrix の T2）、意味が問われない fixture は値を変えない。生の SQL・JSON の文字列（test の INSERT・UPDATE、`serde_json` に渡す JSON）はこの command に当たらないので、Matrix の T2a の 2 つの command で拾い、T2a の分類のとおり最新の版の DB に当てる所の列名と key を直す（失敗だけを確かめていた test は error の文か正の対照を足す。`product_service.rs` の test の `{"cost_price":null}` の null の case を含む: key を直さないと serde が未知の key を無視して緑のまま、null の検査を失う） |
| demo の seed | 原価を `(selling_price as f64 * 0.6) as i64` の円から、浮動小数を通さない 1/100 円 `selling_price * 60` へ（売価 × 0.6 円を 1/100 円で正確に持つ。今の値は floor(売価 × 0.6) 円なので、売価が 5 の倍数でない商品では今の値の 100 倍にならない: 売価 101 は今 60 円、新しい式では 6060〈60.60 円〉。demo の値だけで契約に効かない）。生地の部門の単位 `cm` を `m` に | `src-tauri/src/seed_demo.rs` | `src-tauri/tests/seed_test.rs` の `stock_unit = 'cm'` の 2 か所を `'m'` に（在庫少の閾値の条件。期待は変えない） |

### S3 BIZ の原価

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| 上限の定数 | `MAX_COST_PRICE: i64 = 999_999` を `MAX_COST_PRICE_CENTI: i64 = 99_999_999` に置き換える（名前に単位を付け、古い読み手を compile error で止める。D5 の `_centi` と同じ理由） | `src-tauri/src/constants.rs` | 使う所: `product_service.rs`・`receiving.rs`・`disposal.rs`（`rg -n -w 'MAX_COST_PRICE' src-tauri/src` の production の行）。`schema_v8.rs` の自分の定数 `MAX_COST_PRICE`（v8 の履歴の円の上限）は変えない |
| v8 の上限の test の原価の行 | `test_v8_req903_limits_match_runtime_constants` の `assert_eq!(MAX_COST_PRICE, constants::MAX_COST_PRICE)` を `assert_eq!(MAX_COST_PRICE * 100 + 99, constants::MAX_COST_PRICE_CENTI)`（v8 の円の上限 `999999` と今の上限 `999999.99` 円が円の桁で同じことを止める。共通規則「掛ける lane」の「桁が増えるだけで、円での上限は同じ桁」） | `src-tauri/src/db/schema_v8.rs` の test | Matrix の T4 |
| 商品の検証 | create・update・価格改定・商品 CSV の preview と commit の原価の上限を `MAX_COST_PRICE_CENTI` で検査する（文は今のまま） | `src-tauri/src/biz/product_service.rs`（`price_range_error` の原価の呼出し 3 と commit の `import_row_commit_error`） | 既存の上限の test（`1_000_000` で `原価が大きすぎます` を期待する case: `rg -n '1_000_000' src-tauri/src/biz/product_service.rs`）は、1/100 円の境界（`99_999_999` は受け、`100_000_000` は error）に書き換える（Matrix の T2。境界の意味は変えない） |
| 商品 CSV の `原価` | `row.fields.get("原価").and_then(|v| v.parse::<i64>().ok())` を、Y2 の既存の関数（`parse_meters_to_cm` を単位に依らない名前に改めたもの）で 1/100 円にする形に替える（`83.33` → 8333、`1000` → 100000）。行の error は Y2 の分け方: 先頭が `-` で残りが規則に合えば今の負の文 `原価は0以上で入力してください`（`-1`・`-0`）、それ以外の形の誤り（`83.333`・`1e3`・`+5`・`.5`・`5.`・空）は SPEC-UNIT-D5 の文 `原価は0以上で、小数は2桁までで入力してください`（今の `原価が不正です` を置き換える。直し方を名指す: AGENTS.md の「使い手に押しつけない」。出所は Plan Review の AI review）。上限は今の `price_range_error`。`ImportRow.cost_price` → `cost_price_centi`。`在庫単位` の `巻`・`cm` の行の文は S6 | `product_service.rs` の商品 CSV の preview（必須の列 `原価` の読取り）と `ImportRow` | `src-tauri/src/io/product_csv_importer.rs` は文字列のまま渡す（変えない）。CSV の test の原価の値の期待（Matrix の T2） |
| 入庫・廃棄の明細 | 原価の上限を `MAX_COST_PRICE_CENTI` で。行の金額は `cost_line_centi(i128::from(item.cost_price_centi), …)`（`* 100` を外す）。原価差分は 1/100 円どうしで比べ、`CostDiff` の field を `master_cost_price_centi`・`received_cost_price_centi` に | `src-tauri/src/biz/inventory_service/receiving.rs`（`ReceivingItemInput`・`CostDiff`・明細の検証・行の金額・差分の検出）、`disposal.rs`（`DisposalItemInput`・明細の検証・行の金額） | `receiving.rs` の `cost_diffs` の test（`master_cost_price` 500・`received_cost_price` 501 の期待）は 1/100 円の値で同じ形に（Matrix の T2） |
| 記録詳細の行の金額 | `list.rs` の入庫・廃棄の記録詳細の 2 か所で `* 100` を外す | `src-tauri/src/biz/inventory_service/list.rs` | 記録詳細の test の金額の期待（Matrix の T2） |
| 価格改定・価格の履歴 | `PriceRevisionInput.new_cost_price` → `new_cost_price_centi`、`PriceHistoryEntry.old_cost_price`/`new_cost_price` → `_centi` | `product_service.rs`（`PriceRevisionInput`）、`product_repo.rs`（`PriceHistoryEntry`） | 価格改定の test |
| 棚卸し（旧本体、test だけが呼ぶ） | 確定時評価原価を `valuation_cost_price_centi` で書き読みし、評価額の `cost_line_centi` に `* 100` を付けずに渡す | `src-tauri/src/biz/stocktake_service.rs`、`#[cfg(test)]` の `stocktake_service/time_evidence.rs` | 旧本体と時点証拠の test の評価額の期待（Matrix の T2。円の原価 a を 1/100 円の `a * 100` で入れれば評価額は今と同じ） |
| 商品の型 | `Product.cost_price`・`ProductCreateRequest.cost_price`・`ProductUpdateRequest.cost_price` を `_centi` に。BIZ-01-D7 の更新の log の key（`cost_price`）と、価格改定の操作ログ `product_price_revise` の detail の key（`revise_product_price` の `detail_json` の `"cost_price":{"old","new"}`）も `cost_price_centi` に | `product_repo.rs`（`Product`）、`product_service.rs`（Request の 2 つ、update の log、価格改定の log） | `product-update-patch-contract.test.ts` の field の一覧（S5） |

### S4 wire と bindings

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| 13 型の原価の field 名 | S2・S3 で変えた field が specta で `_centi` の名前になる。`plu_export_cmd.rs` の `ProductResponse.cost_price` も `_centi` に | `src-tauri/src/cmd/plu_export_cmd.rs`（`ProductResponse` と変換）、ほかは S2・S3 の struct | `cmd` の test（`inventory_cmd.rs`・`sales_cmd.rs`・`csv_import_cmd.rs`・`plu_export_cmd.rs` の fixture） |
| bindings | `cd src-tauri && cargo run --bin generate_bindings` で `src/lib/bindings.ts` を再生成する（手で直さない）。`ProductStockUnit` も S6 の後に 10 個の union になる | `src/lib/bindings.ts` | AC6 |

### S5 画面の原価（入力・表示・補助）

数える command: `rg -l -w 'cost_price|valuation_cost_price|old_cost_price|new_cost_price|master_cost_price|received_cost_price|MAX_COST_PRICE' src | sort`（起票の時点で 46 file。bindings を含む）。

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| 原価の解析・表示・補助の関数（TS） | Y2 の解析（`parseCostInput`。結果は 1/100 円 / `"too_large"` / null。`readQuantity` の `m` の分岐を切り出した関数を使う）、Y3 の表示（原価の formatter〈半角の `¥`〉と、入力の欄の初期値の文字列の関数）、Y5 の補助（`unitCostFromPack`。`divideRoundHalfUp` を使う）、定数 `MAX_COST_PRICE_CENTI = 99_999_999`（今の `MAX_COST_PRICE` を置き換える）。売価は今の `parsePriceInput` のまま | `src/lib/unit-quantity.ts`、`src/features/stock-inquiry/lib/format-stock-display.ts`（formatter） | `src/lib/unit-quantity.test.ts` の `MAX_COST_PRICE` の期待（`999999`）は `MAX_COST_PRICE_CENTI` の `99999999` へ（Matrix の T4）。新しい golden は Matrix の C10〜C13 |
| 商品フォーム | 原価の欄を Y2 で読み（create・update）、修正の画面の初期値を Y3 の初期値の文字列にする。label と価格の節の説明を UI-01b-D22（原価の lane の後）の文に。UI-01b-D24 の補助（個数の単位の商品だけ。開き直すと空、`m` に変えたら閉じて隠す）。原価の形式の error の文を SPEC-UNIT-D5 の文 `原価は0以上で、小数は2桁までで入力してください` にする（売価と共有の `readPrice` の文 `${label}は0以上の整数で入力してください` から原価を分け、売価の文は今のまま）。原価の欄は `inputMode="decimal"`（今は `numeric`。`type="number"` の欄ではないので `step` は付けない。先例は `src/lib/unit-quantity.ts` の `quantityInputAttributes`） | `src/features/products/components/ProductForm.tsx`、`src/features/products/lib/product-form-request.ts`（`readPrice` の原価の呼出し 2 と request の field） | `ProductForm.test.tsx`・`product-form-request.test.ts`・`product-update-patch-contract.test.ts`（field の一覧に `cost_price_centi`）。新しい RTL は Matrix の C14 |
| 価格改定 | 現原価・新原価（案）・現掛率を Y8 の式で求め、新原価の欄を Y2 で読む。表示は Y3。新原価の欄に入れる新原価（案）と、現売価 0 の行の現原価は Y3 の入力の欄の初期値の文字列にする（今の `setCost(String(…))` のまま 1/100 円の値を入れると、欄に `9100` と出て 910000 を送る）。新原価の欄の形式の error の文（今は `0以上の整数で入力してください`）を SPEC-UNIT-D5 の文に（新売価の欄の文は今のまま）。新原価の欄の属性を `inputMode="decimal"`・`step="0.01"`（今は `type="number"`・`step={1}`） | `src/features/products/components/PriceRevisionTable.tsx`、`src/features/products/lib/price-revision-math.ts` | `PriceRevisionPage.test.tsx`、`usePriceRevisionList.test.tsx`。77 の例（現原価 8333・現売価 100・新売価 110 → 9100）を Matrix の C15、現原価の列の表示を C21 |
| 入庫 | 行の原価の初期値を商品の原価の Y3 の初期値の文字列に、原価の欄を Y2 で読み `cost_price_centi` で送る。原価の形式の error の文（`receiving-request.ts` の `原価は0以上の整数で入力してください`）を SPEC-UNIT-D5 の文に。原価の欄の属性を `inputMode="decimal"`・`step="0.01"`（今は `type="number"`・`inputMode="numeric"`） | `src/features/receiving/lib/receiving-row-utils.ts`・`receiving-request.ts`・`ReceivingPage.tsx`・`types.ts` | `receiving-request.test.ts`・`receiving-row-utils.test.ts`・`ReceivingPage.test.tsx`・`src/lib/request-helpers.test.ts` |
| 原価差分の dialog | マスタ原価・今回の実原価を Y3 で出し（今の `1,000 円` の形から半角の `¥` へ）、更新は `new_cost_price_centi: diff.received_cost_price_centi` で送る | `src/features/receiving/CostDiffDialog.tsx` | `ReceivingPage.test.tsx` の `cost_diffs` の case |
| 廃棄 | 行の原価の初期値・欄の解析・候補の一覧の原価の表示（今の `formatYen(candidate.cost_price)`）・入力画面の合計の BigInt の twin（共通規則 (c)。原価を 1/100 円のまま `BigInt` で掛ける。`calculateLossTotal` は `parseCostInput` の結果が number でない行〈`"too_large"`・null〉を 0 として数える: 今の「入力の error の行は 0 として数える」のまま。今の `=== null` だけの guard を残すと `BigInt("too_large")` が投げて画面が落ち、`BigInt` は string も受けるので typecheck は止めない。round 3 の是正）。原価の形式の error の文（`disposal-request.ts` の `原価は0以上の整数で入力してください`）を SPEC-UNIT-D5 の文に。原価の欄の属性を `inputMode="decimal"`・`step="0.01"`（今は `type="number"`・`inputMode="numeric"`） | `src/features/disposal/lib/disposal-row-utils.ts`・`disposal-request.ts`・`DisposalPage.tsx`・`types.ts` | `disposal-request.test.ts`（1/100 円の原価の BigInt の例: 共通規則 (c) の `10999998790000049`）、`DisposalPage.test.tsx` |
| 記録詳細 | 入庫・廃棄の明細の原価を Y3 で（行の金額は今の `formatCentiYen` のまま）。棚卸しの明細の評価原価を Y3 で、ロス原価を上の「起票時に起草役が決めたこと」の BigInt の形で | `src/features/inventory-records/ReceivingRecordDetailPage.tsx`・`DisposalRecordDetailPage.tsx`・`StocktakeRecordDetailPage.tsx` | 各 test の原価の表示の期待（Matrix の T2） |
| 商品一覧・在庫照会の詳細・価格の履歴 | 原価の表示を Y3 の formatter に（今の全角の `￥`〈`Intl.NumberFormat` の currency〉から半角の `¥` へ。同じ画面の売価の表示は今の formatter のまま: Non-scope） | `src/features/products/components/ProductTable.tsx`・`src/features/stock-inquiry/components/StockDetailContent.tsx`・`src/features/products/components/PriceHistorySection.tsx` | 各 test（表示の期待を変える所は Matrix の T5）。新しい RTL は Matrix の C19・C20 |
| test の fixture | 原価の field 名を `_centi` に（値は表示の期待に合わせる） | `src/features/products/lib/test-fixtures.ts`・`src/features/stock-inquiry/lib/test-fixtures.ts` ほか上の command の test file | Matrix の T2 |

### S6 巻・cm を単位から外す

数える command: Rust `rg -n -w 'Roll|Cm' src-tauri/src src-tauri/tests`（起票の時点で 21 行・5 file）と `rg -l "[\"']cm[\"']|[\"']roll[\"']" src-tauri/src src-tauri/tests | sort`（17 file）、TS `rg -l "[\"']cm[\"']|[\"']roll[\"']|\broll:|\bcm:" src | sort`（36 file）。

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| enum | `ProductStockUnit` から `Roll`・`Cm` を外し、`ALL` を `[Self; 10]` に、`as_str` の 2 行を外す | `src-tauri/src/db/product_repo.rs` | `finite_enum_contract_tests.rs`（`ALL` の数と対の表）、`unit_amount.rs` の `stock_unit_kind`・`price_basis_quantity` の match と golden（`(U::Roll, Count, 1)`・`(U::Cm, Length, 100)` の 2 行を外す） |
| 商品 CSV の語 | `stock_unit_label` の 2 行を外す（`巻`・`cm` を受けなくなる。`normalize_stock_unit` は `ALL` から引くので自動）。preview の行の error の今の文 `在庫単位が正しくありません: {値}` に、外した語と code（`巻`・`roll`・`cm`）だけ直し方を添える（SPEC-UNIT-D10。出所は Plan Review の AI review）: `cm` は `在庫単位が正しくありません: cm（長さは m で入れてください。初期在庫も m の数〈cm の数を 100 で割る〉にしてください）`、`巻`・`roll` は `在庫単位が正しくありません: 巻（巻で数える商品は 個 で入れてください。数は変わりません）`（`{値}` は CSV の値のまま。巻は個数の単位で m と換算しない: SPEC-UNIT-D1。個数の単位どうしは数の意味が同じ: D2）。ほかの値は今の文のまま。判別は `product_service.rs` の新しい関数 `retired_stock_unit_hint`（CSV の値を受け、`cm` と `巻`・`roll` のときだけ添える文を返す）の 1 か所に置き、preview と commit の再検証（`import_row_commit_error` の単位の code の検査）の両方がこの文を使う（round 3 の是正）。本番の `"cm"`・`"roll"` の文字列をそこだけにする（AC5）。preview の画面の中で行を直して再検証する経路は作らない（Non-scope） | `src-tauri/src/biz/product_service.rs` | 12 個の code と語が通る test（`("roll", "巻")` を含む表）を 10 個に、`roll`・`巻`・`cm` が行の error になる case を足す（Matrix の U4） |
| 在庫少の並び | `ALL` を `stock_unit_kind` で分ける今の形のまま（9 と 1） | 変えない | `product_service.rs` の 2 つの並びが合わせて 12 個の test を 10 個に（`vec![ProductStockUnit::M, ProductStockUnit::Cm]` の期待を `vec![ProductStockUnit::M]` に）、`inventory_cmd.rs` の「12 個の単位を 1 つずつ在庫 0 で置くと 12 個とも出る」を 10 個に（Matrix の T3） |
| v8 の CHECK の test | `test_v8_req903_check_codes_match_enum` を、v8 の CHECK を固定の 12 個の一覧（v8 の履歴）と比べる形に直す（v8 の SQL は変えない）。enum との一致は vB の CHECK の test（B3）へ移す（backlog の (i)） | `src-tauri/src/db/schema_v8.rs` の test | Matrix の T4 |
| formatter・単位の関数（TS） | `STOCK_UNIT_LABELS` の `roll`・`cm` を外す（型の網羅で compile が通らなくなるので自動で気づく）。`priceBasisQuantity`・`isLengthUnit` の `case "roll"`・`case "cm"` を外す。`cm` の商品の入力の分岐（cm の整数）を外す | `src/features/stock-inquiry/lib/format-stock-display.ts`、`src/lib/unit-quantity.ts` | `format-stock-display.test.ts`・`unit-quantity.test.ts` の 12 個の表を 10 個に |
| 商品フォームの select | `STOCK_UNIT_LABELS` から作るので自動で 10 個。画面の note `長さ（m・cm）の商品は在庫同期しない設定を提案します。必要なら変更できます。` を `長さ（m）の商品は…` に、comment の `m・cm` と「12 個」も直す（`StockUnitField.test.tsx` の note の期待は `/在庫同期しない設定を提案します/` の正規表現で `（m・cm）` を見ないので、残りは AC5 の `m・cm` の検査で止める） | `src/features/products/components/StockUnitField.tsx`（note の文と comment） | `StockUnitField.test.tsx`・`ProductForm.test.tsx` の 12 個の語の期待を 10 個に（Matrix の T3） |
| `cm` を fixture にする test | 長さの商品の fixture を `cm` から `m` に替える（`m` と `cm` は在庫・価格・POS の意味が同じで、違いは入力と表示だけ: SPEC-UNIT-D1）。`cm` の入力の分岐（cm の整数）だけを確かめる case は、契約（SPEC-UNIT-D3 の `cm` の入力）ごと外し、同じ意味の `m` の case があることを Matrix の T3 に 1 対 1 で並べる | 上の command の file（Rust・TS） | Matrix の T3 |
| 在庫少の閾値の単位 | 変えない（`extract-thresholds.ts` の `unit: "cm"` は在庫少の基準の単位の語で、SPEC-UNIT-D10 は「基準の画面の語と cm の入力は変えない」） | — | AC5 の除外 |

### S7 正本の同期・traceability・ERD（実装と同じ PR）

実装した範囲の `proposed・未実装` を現行の記述に直し、巻・cm の記述を外す。対象は次の 6 つの command の出力と、この節で名指した行（51 §7.6・30 §4.8・`SCREEN_DESIGN.md` の入庫・35 §20.5a・ERD・58・90・code の comment）で全部。command はどれも本 packet と Matrix（`docs/plans/**`）を除く（AC10 と同じ。検索式と移行前の期待を同期の対象に混ぜない）。数は round 1 の是正の時点（2026-10-21、本 worktree）:

- `rg -n '10 種|10 単位|10 個|個数の 10' . --glob '!docs/archive/**' --glob '!docs/decision-log.md' --glob '!docs/plans/**'`（23 行）の単位の数の記述（live な hit: 10 の SPEC-UNIT-D2 の続きの項・D3 の集計の規則・D10、35 §20.5a の単位の拡張の段、44 の在庫少の SQL、34 の冒頭、`DB_DESIGN.md` の在庫少の適用ルール、master-tables の価格の基準数量、`src/lib/unit-quantity.test.ts` の test 名）を 9 個・9 単位へ。`10 個` の数量の表示の例（58・`format-stock-display` の `10 個`、10 の (c) の「10 個で越える」）は単位の数でないので変えない。
- `rg -n '12 個|12 種|12 単位|12 語|12 の語|12 個の code' . --glob '!docs/archive/**' --glob '!docs/decision-log.md' --glob '!docs/plans/**'` の単位の一覧の記述（46 行。起票の 47 行は main `6a25f63f` の値で、plan-first の正本の直しで 1 行減った。10・22・30・51・58・master-tables・`migration-recovery-runbook.md`・code の comment と test 名）を 10 個へ。v8 の履歴を書いた所（22 §16 の vU、`schema_v8.rs`、`migration-recovery-runbook.md` の v8 の説明、`docs/evidence/` の聞き取り記録）は時点の記録なので変えない。
- `rg -c '\`cm\`|\`roll\`' docs --glob '!docs/archive/**' --glob '!docs/decision-log.md' --glob '!docs/evidence/**' --glob '!docs/plans/**'` の巻・cm の記述（20 file）を、vB の後の一覧に合わせて直す（10 の D1 の表から `roll`・`cm` の行、D3 の `cm` の入力、61・62・63・64 の「`cm` の商品は整数」、master-tables の布の 5 パターン表の `cm`、`SCREEN_DESIGN.md`、`project-memory.md`、`project-profile.md`〈workflow の file〉ほか）。
- `rg -n -w 'cost_price|valuation_cost_price|old_cost|new_cost|old_cost_price|new_cost_price|master_cost_price|received_cost_price|MAX_COST_PRICE' docs --glob '!docs/archive/**' --glob '!docs/decision-log.md' --glob '!docs/plans/**'` の原価の列名・wire の field 名・上限の定数の記述（130 行・26 file。round 2 の是正で AC2 と同じ名前の一覧にした: 旧い 4 つの名前だけでは `-w` が `new_cost_price` 等の wire の名前に当たらず、同じ 26 file の 15 行を拾わなかった）を `_centi` に（master-tables の各表のカラム定義、20・21・30・31・35・40・44・51・61・64・76・77、`architecture/*-task-specs.md`、tracking・transaction ほか）。[51](../../function-design/51-ui-product-form.md) §7.6 の `cost_price` の行（文 `原価は0以上の整数で入力してください` と上限 `999999`）は SPEC-UNIT-D5 の文と上限 `999999.99` に直す。どの command にも当たらない原価を整数と書く行も名指して直す（`rg -n '原価[^。]{0,12}整数|整数[^。]{0,6}原価' docs …` で round 2 の是正の時点に洗った）: [30](../../function-design/30-biz-product-service.md) §4.8 の手順 4 の `売価・原価: 0以上の整数`（商品 CSV の行の検証。原価は SPEC-UNIT-D10 の小数 2 桁）と、`SCREEN_DESIGN.md` の入庫の明細の `原価は整数 >= 0`（小数 2 桁まで）。
- `rg -n '原価が円の|円の整数で 0 以上|新原価は 0 以上|× 100' docs --glob '!docs/archive/**' --glob '!docs/decision-log.md' --glob '!docs/plans/**'`（18 行）の、原価が円の整数の間の時点つきの記述・原価の上限 `999999`・原価の `× 100` の式（30 §20 の BIZ の上限、77 の新原価の上限、10 の D6・(a)〜(c) の「原価が円の間」、35 §20.5a の `原価 × 数量 × 100`）を 1/100 円の後の記述に。原価でない `× 100`（10 の D3 の m の入力と単価の例）は変えない。
- `rg -n "'cm'|\"cm\"|m・cm|cm管理" docs --glob '!docs/archive/**' --glob '!docs/decision-log.md' --glob '!docs/evidence/**' --glob '!docs/plans/**'`（22 行）の引用符つきの `cm` と `m・cm`（51 UI-01b-D6 と Test Focus の `stock_unit='cm'` の 2 行、UI-01b-D22 の説明の今の文、`DB_DESIGN.md` の CHECK の一覧、`SCREEN_DESIGN.md`、master-tables の `pos_stock_sync` の理由と布の管理、34 の CSV の `数量`、10 の D3 の表示、58、44 の例、`architecture/*-task-specs.md`）を vB の後の一覧に。時点の記録（`ARCHITECTURE.md` の判断の履歴の表、`project-memory.md` の店の答えの記録、`docs/design-system/reference/` の mockup）は変えない。
- 35 §20.5a の「範囲の外」の段に、棚卸し記録詳細のロス原価を 1/100 円の BigInt で出すこと（上の「起票時に起草役が決めたこと」）を 1 文足す。
- `docs/inventory_system_erd.html` の原価の列名と単位の CHECK と、版の範囲の記述（`v1〜v9` を vB の番号まで）（実装と同じ PR。JAN の別名の lane の先例）。
- [58](../../function-design/58-ui-stock-inquiry.md) の表示の書式の表の `金額（売価 / 原価）`「¥1,234」`Intl.NumberFormat` の行を、原価は SPEC-UNIT-D5 の formatter（半角の `¥`）、売価は今のまま、に分ける。
- `docs/function-design/90-traceability.md` は `cd src-tauri && cargo run --bin generate_traceability` で再生成する（手で直さない）。
- code の comment の旧い原価の記述（round 3 の是正）: `rg -n '原価が円|原価（円）|原価[^。]{0,10}円の整数' src src-tauri/src` の hit（round 3 の是正の時点で 10 行）のうち、`src-tauri/src/biz/unit_amount.rs` の `cost_line_centi` の doc comment（「原価が円の整数の間は、呼ぶ側が `i128::from(原価(円)) * 100` を渡す」）と test の helper の comment（円の 35 §20.5a の golden を確かめる helper と書く）、`src/features/products/lib/product-form-request.ts` の `readPrice` の comment、`src/lib/unit-quantity.ts` の `MAX_COST_PRICE` と `parsePriceInput` の comment（売価だけ）を 1/100 円の後の記述に直す。`ProductForm.tsx` の説明の文は S5、test の期待は Matrix の T5、`schema_v8.rs` の v8 の上限の comment は履歴で変えない。`disposal_repo.rs`・`bindings.ts` の「行のロス原価（1/100 円の整数…）」は誤検出。

新しく書く引用は D-125・`docs/DOC_STYLE_GUIDE.md` §1 の形（path と symbol・節 ID）。触る文の中の既存の file:行 の引用は lane 2a（PR #183）が直すので、本 lane では直さない。

### 節の所有表（並走の lane と重なる file）

| file | 並走の lane が所有する節・行 | 本 lane が触る所 | merge の後に揃える所 |
|---|---|---|---|
| `docs/function-design/22-mnt-migration.md` | SD 直読み（PR #172）: §17（MNT-03-D15）。JAN の別名（未起票、22 §18 MNT-03-D16 を予約）。backup の外の控え（PR #155）: §3.2 の MNT-03-D11「復元への波及」の 1 行。lane 2a（PR #183）: 全体の file:行 の引用 | §16（vB の段・作り直しの手順・確かめていない前提・テスト）。新しい節は作らない（要れば §20・MNT-03-D18） | 後から merge する側が `origin/main` の単段 merge で両方を残す。lane 2a が先なら、本 lane の §16 の文の中の書き換え済みの引用を保つ |
| `src-tauri/src/db/migration.rs` | §17・§18 の lane: `migrations()` の登録と版の期待の行 | 登録・版の期待の行（T1）・MNT-03-D1 の構造の test の列挙 | 版の番号は先に merge した側が若い番号。後の側が file 名・`migrations()` の登録の順・版の期待の行（T1）・B の test の定数 `VB_VERSION`（Matrix の B の前書き）を取り直す（22 §16 の vB の番号の段、`docs/DEV_WORKFLOW.md` Review Rules の連番の registry の項） |
| `src-tauri/src/db/schema_v9.rs`・`schema_v2.rs` の test | §17・§18 の lane: 版の期待の行 | T1 の行 | 同上 |
| `src-tauri/src/db/sales_repo.rs`・`src-tauri/src/biz/sales_service.rs`・`src-tauri/src/cmd/sales_cmd.rs`・`src-tauri/src/biz/daily_report_import_service/tests.rs` | SD 直読み（PR #172）: `NewDailyReportImport.settlement_no` の構築と INSERT | test の商品の fixture の原価の field 名と値、`ProductStockUnit::Cm` の fixture | 隣の行。後から merge する側が両方を残す |
| `docs/decision-log.md` | backup の外の控え（PR #155）: D-114 の本文と D-113 の近くの 1 行 | D-119 の末尾の Superseded in part の行、末尾の D-128 | 末尾への追記は merge の順で両方を残す |
| `docs/backlog.md` | lane 2a: file:行 の引用。PR #155・#172: 各項目の文 | 「原価の lane（C）」の項目の 4 か所の句 | 他の行を並べ替えない |
| `docs/function-design/10-common-rules.md` 他の function-design | lane 2a: file:行 の引用 | 本 commit の SPEC-UNIT-D1・D2・D5 の句と更新履歴、S7 の同期 | 同上 |
| `docs/db-design/pos-tables.md` | SD 直読み（PR #172）: §12b の `settlement_no` | 触らない | — |
| `docs/Plans.md` | lane 2a・各 lane: 次の行動の文（lane の行は足さない。D-097） | `## 次の行動` と `## 導入の前の関門` の原価の lane の句 | — |

重なりを避けるために本 lane の範囲は変えない（発注書の指示）。

## Non-scope

- 上の Goal の「非目的」の全部（売価、数量の契約、10 cm の刻みの検査、棚卸しの評価額の出力の文と記録詳細のロス原価の BIZ への移動、初期投入、Z004・EJ、既存の file:行 の引用）。
- 原価の基準数量（入り数）を商品に持つこと（SPEC-UNIT-D5 の不採用 (a)）。
- 円の値を 1/100 円へ移す migration（D-119 (1)。vB は値を変換しない）。
- 起動の失敗の画面の文の種類を増やすこと（vB の止まりは今の `DatabaseInit` の文に手順 2 の message を添える。§19 と同じ。下の残りの危険）。
- 停止 ADR の入口（棚卸しの開始・保存・確定、Z004 の確定・取消）の振舞いと文言。
- 原価でない金額（売価・合計・行の金額）の記号と書き方を揃えること（backlog の「金額の表示の書き方を 1 つにそろえる」）。本 lane は原価の表示だけを半角の `¥` の formatter にする。
- 商品 CSV の preview の画面の中で error の行を直して再検証する経路（今の CSV の error の全部に共通の制約。backlog の「商品 CSV の preview で error の行をその場で直す経路」）。本 lane は error の文で直し方を名指すまで（S3・S6）。
- 入庫・廃棄・価格改定の原価の欄の見出しに税抜を書くこと（backlog の「入庫・廃棄・価格改定の原価の欄の見出しに税抜が無い」。下の残りの危険）。

## Acceptance Criteria

- AC1（migration）: DB 群は `cd src-tauri && cargo test --lib db::` が全 PASS し、Matrix の B1〜B8・B10 の test（名前は Matrix の名前、または PR 本文の Matrix の ID との対応表の名前）が `rg -n 'fn <名前>' src-tauri/src/db` で 1 件ずつ見つかる（B9 は §18 の lane が先に merge したときだけ、その lane の test）。起動群は `cd src-tauri && cargo test --lib test_startup_req903_` が全 PASS し、Matrix の R1〜R4 の test（名前は同じく Matrix か PR 本文の対応表）が `rg -n 'fn <名前>' src-tauri/src/lib.rs` で 1 件ずつ見つかる（filter は起動の test 名の接頭辞。今の `test_startup_req903_` の 3 つも一緒に走る）。`rg -n 'MigrationKind::Custom\(schema_v' src-tauri/src/db/migration.rs | wc -l` が起票の時点の 7 に、本 lane の vB と、先に merge した並走の lane の migration（`MigrationKind::Custom` のもの）の数を足した値。
- AC2（原価の旧い名前）: `rg -l -w 'cost_price|valuation_cost_price|old_cost|new_cost|old_cost_price|new_cost_price|master_cost_price|received_cost_price' src-tauri/src src-tauri/tests src | sort` が、履歴の migration と vB の前の版に行を入れる migration の test の file（`src-tauri/src/db/schema_v1.rs`・`schema_v2.rs`・`schema_v8.rs`・`schema_v9.rs`・`migration.rs`）と vB の file（`schema_v{N}.rs`）だけ。同じ名前の一覧の `rg -n -w '…' src-tauri/src src-tauri/tests src` の出力（hit ごと）を PR の本文に貼り、hit ごとに、履歴の SQL・vB の前の版（`VB_VERSION - 1` 以下）の DB に当てる fixture・vB の改名の SQL のどれかであることを表にする（同じ file の中の、最新の版の DB に当てる test の hit が紛れないように。Matrix の T2a）。
- AC3（上限）: `rg -n -w 'MAX_COST_PRICE' src-tauri/src src` が `src-tauri/src/db/schema_v8.rs` の行だけ。`rg -n 'pub const MAX_COST_PRICE_CENTI: i64 = 99_999_999;' src-tauri/src/constants.rs` と `rg -n 'export const MAX_COST_PRICE_CENTI = 99_999_999;' src/lib/unit-quantity.ts` が 1 行ずつ。Matrix の C1〜C3 の境界（`99999999` は受け、`100000000` は `原価が大きすぎます`）が PASS。
- AC4（円を 1/100 円にする掛け算）: `rg -n 'i128::from\([^)]*cost[^)]*\) \* 100' src-tauri/src` が `src-tauri/src/biz/unit_amount.rs` の test の 1 行（円の原価で 35 §20.5a の golden を確かめる helper）だけ。
- AC5（巻・cm）: `rg -n -w 'Roll|Cm' src-tauri/src src-tauri/tests` が 0 行。`rg -n 'pub const ALL: \[Self; 10\]' src-tauri/src/db/product_repo.rs` が 1 行。`rg -n "[\"']roll[\"']|[\"']cm[\"']" src --glob '!*.test.*'` が `src/features/threshold-settings/lib/extract-thresholds.ts` の在庫少の基準の単位の 1 行だけ。`rg -n 'm・cm' src --glob '!*.test.*'` が 0 行。Rust の `"cm"`・`'cm'`・`"roll"`・`'roll'` の残りは、`rg -l "[\"']cm[\"']|[\"']roll[\"']" src-tauri/src src-tauri/tests | sort` の出力を PR に貼り、履歴の migration（`schema_v1.rs`・`schema_v8.rs`）、vB の前の版の fixture（`migration.rs`）、vB が拒む値の test（`schema_v{N}.rs`）、行の error を確かめる商品 CSV の case（Matrix の U4）、商品 CSV で外した単位を判別して直し方の文を返す本番の拒否の分岐（`src-tauri/src/biz/product_service.rs` の新しい関数 `retired_stock_unit_hint` の本体の行だけ。S6）のどれかであることを表にする。本番の分岐の検査: `awk '/^mod tests/{exit} {print FILENAME":"NR": "$0}' src-tauri/src/biz/product_service.rs | rg "[\"']cm[\"']|[\"']roll[\"']"`（`product_service.rs` の test の module より前の行）の hit が `fn retired_stock_unit_hint` の本体の行だけ（今は `stock_unit_label` の `ProductStockUnit::Cm => "cm"` の 1 行で、S6 で外れる）。
- AC6（wire）: `src/lib/bindings.ts` で `rg -n 'export type ProductStockUnit = "pcs" \| "sheet" \| "hon" \| "bag" \| "box" \| "kumi" \| "set" \| "ball" \| "cho" \| "m";' src/lib/bindings.ts` が 1 行、`rg -n '\b(cost_price|valuation_cost_price|old_cost_price|new_cost_price|master_cost_price|received_cost_price)\??:' src/lib/bindings.ts` が 0 行、`rg -c '_centi\??:' src/lib/bindings.ts` が起票の時点の 2（`line_cost_centi`・`line_loss_cost_centi`）に 13 型の 16 field を足した 18。
- AC7（画面の原価）: `npm test -- src/lib src/features/products src/features/receiving src/features/disposal src/features/inventory-records src/features/stock-inquiry` が全 PASS し、Matrix の C10〜C21 と U5・U6 の test（名前は Matrix の名前、または PR 本文の Matrix の ID との対応表の名前。C10〜C21 の Matrix の名前は説明の語なので、対応表の名前で `rg -n` する）が見つかる。原価の形式の error の文（SPEC-UNIT-D5）: `rg -c '原価は0以上で、小数は2桁までで入力してください' <file>` が `src/features/receiving/lib/receiving-request.ts`・`src/features/disposal/lib/disposal-request.ts`・`src/features/products/lib/product-form-request.ts`・`src/features/products/components/PriceRevisionTable.tsx` の file ごとに 1 以上、`rg -n '原価は0以上の整数' src --glob '!*.test.*'` が 0 行。`rg -n '原価（税抜、必須' src/features/products/components/ProductForm.tsx` が 1 行以上、`rg -F -n '原価は税抜の円で小数 2 桁まで入力します' src/features/products/components/ProductForm.tsx` が 1 行。
- AC8（既存の test の移し方）: Matrix の T1〜T5 の command を実装の後に逐語で回し、出力の各行を T1〜T5 の分類（と本 lane で足した test）に当てた表を PR の本文に貼る。「書き換える」「変えない」の所で、意味の期待（境界の位置、件数、message、行の error の有無）が変わっていない（値の単位の書き換え〈円 → 1/100 円〉と `cm` → `m` は T2・T3 の表の 1 対 1 の対応で示す）。
- AC9（gate）: `cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`、`cargo run --bin generate_bindings` の後に `git diff --exit-code src/lib/bindings.ts`、`cargo run --bin generate_traceability -- --check`、`cargo test --test design_compliance_test`、`npm run typecheck && npm run lint && npm run format:check && npm test && npm run build`、`bash scripts/doc-consistency-check.sh` と `--target plan` が exit 0。L3 を含むので Writer は `cd src-tauri && cargo check --release` を通す。
- AC10（正本の同期）: S7 の 6 つの command を実装の後に逐語で回し（名指しの行も見て）、残る hit が S7 の「変えない」の条件（時点の記録、数量の表示の例、v8 の履歴）だけであることを PR の本文に表で貼る。`rg -n 'proposed・未実装' docs --glob '!docs/archive/**' --glob '!docs/plans/**' --glob '!docs/decision-log.md' | rg '原価の lane|D-119|D-128' | rg -v '^\S+:[0-9]+:\| 20'`（更新履歴の行を除いた原価の lane の `proposed・未実装` の注記）が 0 行（起票の時点で 24 行。round 1 の是正で SPEC-UNIT-D10 の商品 CSV の `原価` と `在庫単位` の文に注記を 1 つ足して 25 行）。

AC の数の baseline（AC は AC1〜AC10 の 10。2026-10-21、本 worktree〈code は main `6a25f63f` と同じ〉で逐語で実行した。round 1 の是正の時点で全部を回し直し、動いたのは最後の `proposed・未実装` の行だけ〈理由は AC10〉。round 2 の是正の時点でも回し直して同じ値で、AC2 の hit ごとの数と AC5 の本番の分岐の検査を足した。round 3 の是正の時点でも回し直して同じ値で、AC8 の T5 の 2 つ目の command を足した。rg の file の順は実行ごとに変わる）:

```
$ rg -n 'MigrationKind::Custom\(schema_v' src-tauri/src/db/migration.rs | wc -l
7
$ rg -l -w 'cost_price|valuation_cost_price|old_cost|new_cost|old_cost_price|new_cost_price|master_cost_price|received_cost_price' src-tauri/src src-tauri/tests src | sort | wc -l
77
$ rg -n -w 'MAX_COST_PRICE' src-tauri/src src | wc -l
32
$ rg -n 'i128::from\([^)]*cost[^)]*\) \* 100' src-tauri/src
src-tauri/src/biz/inventory_service/list.rs:119:                    i128::from(item.cost_price) * 100,
src-tauri/src/biz/inventory_service/list.rs:201:                    i128::from(item.cost_price) * 100,
src-tauri/src/biz/stocktake_service.rs:510:            i128::from(valuation_cost_price) * 100,
src-tauri/src/biz/inventory_service/receiving.rs:152:            i128::from(item.cost_price) * 100,
src-tauri/src/biz/inventory_service/disposal.rs:155:            i128::from(item.cost_price) * 100,
src-tauri/src/biz/unit_amount.rs:140:        cost_line_centi(i128::from(cost_yen) * 100, quantity, basis, "L")
src-tauri/src/biz/stocktake_service/time_evidence.rs:706:            i128::from(product.cost_price) * 100,
$ rg -n -w 'Roll|Cm' src-tauri/src src-tauri/tests | wc -l
21
$ rg -n 'pub const ALL: \[Self; (10|12)\]' src-tauri/src/db/product_repo.rs
52:    pub const ALL: [Self; 12] = [
$ rg -n "[\"']roll[\"']|[\"']cm[\"']" src --glob '!*.test.*'
src/lib/unit-quantity.ts:26:    case "roll":
src/lib/unit-quantity.ts:33:    case "cm":
src/lib/bindings.ts:1296:export type ProductStockUnit = "pcs" | "sheet" | "hon" | "bag" | "box" | "roll" | "kumi" | "set" | "ball" | "cho" | "m" | "cm";
src/features/threshold-settings/lib/extract-thresholds.ts:53:    unit: "cm",
src/features/stock-inquiry/lib/format-stock-display.ts:25:  cm: "cm",
$ rg -n '\b(cost_price|valuation_cost_price|old_cost_price|new_cost_price|master_cost_price|received_cost_price)\??:' src/lib/bindings.ts | wc -l
16
$ rg -c '_centi\??:' src/lib/bindings.ts
2
$ rg -n 'migrate\(&conn\)' src-tauri/src | wc -l
34
$ rg -n 'proposed・未実装' docs --glob '!docs/archive/**' --glob '!docs/plans/**' --glob '!docs/decision-log.md' | rg '原価の lane|D-119|D-128' | rg -v '^\S+:[0-9]+:\| 20' | wc -l
25
$ rg -n 'fn test_startup_req903_' src-tauri/src/lib.rs | wc -l
3
$ rg -n 'deriveProposedCost|formatMarkupRate|parseRequiredSafeInteger|原価（必須|原価も円の整数' src --glob '*.test.*' | wc -l
28
$ rg -n -w 'cost_price|valuation_cost_price|old_cost|new_cost|old_cost_price|new_cost_price|master_cost_price|received_cost_price' src-tauri/src src-tauri/tests src | wc -l
455
$ awk '/^mod tests/{exit} {print FILENAME":"NR": "$0}' src-tauri/src/biz/product_service.rs | rg "[\"']cm[\"']|[\"']roll[\"']"
src-tauri/src/biz/product_service.rs:997:         ProductStockUnit::Cm => "cm",
$ rg -n 'm・cm' src --glob '!*.test.*'
src/features/products/components/StockUnitField.tsx:3:// UI-01b-D6 / UI-01b-D22: 単位は 12 個の表示の語から選ぶ。長さの単位（m・cm）を選ぶと
src/features/products/components/StockUnitField.tsx:48:    // 長さの単位（m・cm）で false を提案、個数の単位で既定値 true を復元する。
src/features/products/components/StockUnitField.tsx:106:            長さ（m・cm）の商品は在庫同期しない設定を提案します。必要なら変更できます。
src/features/products/components/ProductForm.tsx:355:        description="売価は税込の円の整数、原価も円の整数で入力します。長さ（m・cm）の商品は 1 m あたりの値段です。"
$ rg -c '原価は0以上で、小数は2桁までで入力してください' <AC7 の 4 file のそれぞれ>
（4 file とも出力なし = 0）
$ rg -n '原価は0以上の整数' src --glob '!*.test.*'
src/features/disposal/lib/disposal-request.ts:85:    if (costPrice === null) messages.push("原価は0以上の整数で入力してください");
src/features/receiving/lib/receiving-request.ts:56:    if (costPrice === null) messages.push("原価は0以上の整数で入力してください");
```

完了時の期待: AC2 の file が 6（履歴の 4 と `migration.rs` と vB の file）。AC3 の `MAX_COST_PRICE` が `schema_v8.rs` の行だけ。AC4 が 1 行。AC5 の `Roll|Cm` が 0、`ALL` が 10、TS の production の hit が `extract-thresholds.ts` の 1 行、`m・cm` が 0。AC7 の新しい文が 4 file で 1 以上、`原価は0以上の整数` が 0（`product-form-request.ts` は今 `${label}は0以上の整数` の template なので、この検査に当たらない。新しい文の検査が拾う）。AC1 の `test_startup_req903_` が今の 3 と R1〜R4 の 4 で 7。AC6 の旧い名前が 0、`_centi` が 18。`migrate(&conn)` の行の数は T1 の書換え（`migrate_up_to` への置換え 8 行）の後に 26 前後（数そのものは AC にしない。T1 の表との対応で確かめる）。test の本数は AC にしない（PR と runner の出力を参照）。

## Design Readiness

設計の完了条件は `docs/DEV_WORKFLOW.md` Design Phase Rules の Design completion criteria。

- 引用する設計正本（節まで）: [共通規則](../../function-design/10-common-rules.md) SPEC-UNIT-D1（一覧と作り直しの起点の項）・D2・D3（`cm` の入力と表示を外す）・D5（精度・入れ方・税の区分・入力の補助・入力・表示）・D6（`cost_line_centi` の引数）・D9（formatter の網羅）・D10（在庫少と商品 CSV）・D11（wire の enum）と「入力の上限と安全な整数の範囲」の (a)〜(c)。[22](../../function-design/22-mnt-migration.md) §16（MNT-03-D14 vB、作り直しの手順、DB の作り直しと順、テスト）と §3.2 MNT-03-D1・§18 の規則・§19。[master-tables](../../db-design/master-tables.md) の「単位と原価の精度の契約」と `products` の表。[51](../../function-design/51-ui-product-form.md) UI-01b-D22・D23・D24。[61](../../function-design/61-ui-receiving.md) UI-02-D7・D15、[64](../../function-design/64-ui-disposal.md) UI-05-D8、[77](../../function-design/77-ui-bulk-price-revision.md) の新原価（案）、[30](../../function-design/30-biz-product-service.md)・[31](../../function-design/31-biz-inventory-service.md)（§12.3 の原価差分、§12.8 の fingerprint）・[35](../../function-design/35-biz-stocktake-service.md) §20.5a の原価の注記、[40](../../function-design/40-cmd-product.md)・[44](../../function-design/44-cmd-inventory.md) の原価の注記。D-113・D-119・D-123・D-128。
- 必要な設計成果物（状態）: (1) 原価の税の区分（U6 → 税抜）と画面の文 = updated in this PR（D-128、SPEC-UNIT-D5、master-tables、UI-01b-D22）。(2) 巻・cm を外す前提の確定と vB の手順 4 の分岐の除去 = updated in this PR（D-128、SPEC-UNIT-D1、22 §16、master-tables）。(3) vB の止まりの文（原因と直し方）と起動の画面 = updated in this PR（22 §16 手順 2・止まった後の進め方）。(4) 作り直しの手順の確かめていない前提（WebView の保存）= updated in this PR（22 §16）。(5) 原価の表示の記号（半角の `¥`）と入力の欄の初期値の文字列（価格改定の新原価〈案〉と現売価 0 の現原価を含む）= updated in this PR（SPEC-UNIT-D5）。(5b) 商品 CSV の `原価` の行の error の分け方と、`在庫単位` の `巻`・`roll`・`cm` の行の error の直し方 = updated in this PR（SPEC-UNIT-D10）。(6) 補助を開き直すと空 = updated in this PR（UI-01b-D24）。(7) 棚卸し記録詳細のロス原価の単位の合わせ方 = 本 packet で決め、実装と同じ PR で 35 §20.5a へ（S7）。ほかの振舞い（vB の手順、原価の入力・上限・表示の数の規則、補助、行の金額、原価差分、fingerprint、新原価〈案〉、単位の 10 個）は既存の正本で足りる（D-119 の design lane で `proposed・未実装` として書いた）。
- plan にしかない durable な判断の昇格先: 上の (1)〜(6) は正本と D-128 に置いた。packet にだけあるのは、規模と手法、S1〜S7 の順、所有表、既存の test の移し方（Matrix の T1〜T5）、横断の約束の表（Y1〜Y10 の保証する所の symbol）、定数と関数の名前（`MAX_COST_PRICE_CENTI`・`parseCostInput`・`unitCostFromPack`）、(7)（実装の PR で 35 へ）。
- 前提・制約と、延期した design gap の follow-up: 10 cm の刻みの検査（Q-098 の答えの後。D-128 (3)）。棚卸しの評価額の出力の税抜の文と記録詳細のロス原価の BIZ への移動（D-118 の (b)）。WebView の保存の置き場は L3 の作り直しで確かめる（Contract Probe P5）。
- 絶対保証（cannot happen / always happens）の例外と escape hatch の自己点検: 「vB は行のある DB で何も変えない」は手順 2 の検査が DDL の前にあり、失敗は ROLLBACK と `foreign_keys` の復元（B4・B6）。「円の値が `_centi` に入らない」の例外は、vB の前の版の DB（vB で止まる）と、test だけの helper `cost_line_centi(i128::from(円) * 100, …)`（AC4、35 §20.5a の golden）。「巻・cm を受ける経路は無い」の例外は、vB の前の版の DB（vB で止まる。止まらずに `cm` の行が残る DB は、vB が `products` に行が無いことを求めるので作れない）と、履歴の migration の SQL（v1・v8）。
- 判定: ready。owner に聞く設計の問いは無く、Scope の全ての振舞いが正本か本 packet（(7) と名前）から決まる。

## Registration / Generation Obligations

| 変更対象 | 登録・生成義務 |
|---|---|
| Tauri command（frontend から呼ぶ） | command の追加・改名は無い。DTO の field 名と `ProductStockUnit` が変わるので `cd src-tauri && cargo run --bin generate_bindings` で `src/lib/bindings.ts` を再生成する（S4） |
| function-design doc 新設 | 該当なし（新しい doc は無い）。新しい `db::schema_v{N}` の関数は `pub(crate)` にする（`design_compliance_test.rs` は `pub fn` だけを見る）。TS の新しい関数（`parseCostInput`・`unitCostFromPack`・原価の formatter）は design compliance の対象外 |
| source / workflow doc 新設・改名・削除 | 該当なし（S7 は既存の文書の文を直すだけ。`docs/project-profile.md` は workflow の分類に入る: 上の Risk） |
| REQ / coverage の追加・変更・削除 | 新しい test に REQ-101・REQ-102・REQ-104（商品 CSV）・REQ-105・REQ-201・REQ-204・REQ-209・REQ-903 を付ける（新しい REQ は無い）。`cd src-tauri && cargo run --bin generate_traceability` で `docs/function-design/90-traceability.md` を再生成する（S7） |
| route 新設・改名・削除 | 該当なし |
| operator 画面新設・改名・削除 | 該当なし（既存の画面の中身だけを変える。到達の導線は今のまま） |

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | Which concepts belong to replaceable external adapters, and which concepts are stable app-core contracts? | Architecture, function design, decision-log, Plan Packet | 当たらない。POS の adapter（PLU・Z004・EJ）に触れない（PLU の単価は売価で、原価は出ない） | — |
| Fact check / design decision split | Which claims are observed facts from hardware/tool/files, and which are app decisions that need source-doc promotion? | Investigation doc, source design docs, decision-log | 当たる。店の答え（U6 = 税抜、10 cm は刻み）と調べ（U1 = 巻・cm 0 行）は事実として D-128 に置き、税の区分を原価の列に持たせることと vB の分岐を外すことはアプリの決定として D-128 に置いた。SQLite の振舞いは Contract Probe P1〜P4 で観測した | D-128、Contract Probe |
| Lifecycle / retry | What happens before, during, after, and after failure for import/export, duplicate input, rollback, retry, cancellation, and re-run? | Function design, DB design, UI design, Test Matrix | 当たる。vB は 1 TX で、失敗は全部戻り `foreign_keys` を戻す（B4〜B6）。再実行で重複適用しない（B7）。止まった DB は作り直す（R1〜R4）。原価の入力の失敗は保存の前（C1〜C3・C10） | Matrix の State Lifecycle |
| Operator workflow | What does the operator do in the real sequence across app, external tool, media, print/export, backup, and recovery? | Screen/UI design, function design, Plan Packet manual checks | 当たる。店の操作は原価を小数 2 桁で入れられることと補助が増えるだけ。開発者は止まった DB を作り直す（22 §16）。商品 CSV の `原価` は小数 2 桁まで受ける | L3-1〜L3-3 |
| Replacement path | If the external system changes, which files/modules/docs are replaced and which app-core contracts remain stable? | Architecture, function design, decision-log | 当たらない（外部の system に触れない） | — |
| Data safety / evidence | How can the claim be supported by anonymized shape/count/hash/procedure evidence without committing real store data? | Plan Packet Data Safety, investigation doc, review evidence | 当たる。fixture は全部合成。owner の L3 用の DB は作り直しで消さずに移す（下の Data Safety）。U1 の調べは行の数だけを claim にし、店の商品の名前を書かない | Data Safety |
| Reporting / accounting semantics | Are totals, summaries, item records, returns, corrections, and inventory movements modeled separately enough to avoid false business meaning? | DB design, function design, report design, Test Matrix | 当たる。原価は税抜で、入庫の原価合計・廃棄のロス原価・棚卸しの評価額は税抜の値（D-128）。行の金額は 1/100 円の原価から求め、円の合計は今の丸め（SPEC-UNIT-D6） | C4〜C6 |
| Manual verification | Which assertions cannot be proven by automated tests and require Windows native L3, external tool import, or real-device confirmation? | Plan Packet, Test Matrix, PR body | 当たる。Windows の実 DB での vB の止まり方と作り直しの後の起動（L3-1）、商品フォームの原価の小数・補助・単位の select の見た目（L3-2）、入庫と原価差分・価格改定の原価の表示（L3-3） | Test Plan の L3 |
| 環境・再現性 | 新設の環境依存（toolchain / CI runner / OS 差異等）を repo-pinned config で強制するか、明示的に defer するか | repo-pinned config, Plan Packet | 当たる。`ALTER TABLE … RENAME COLUMN` は SQLite 3.25 以降（同梱は `libsqlite3-sys` 0.28.0 の 3.45.0。`src-tauri/Cargo.toml` の `rusqlite = { version = "0.31", features = ["bundled"] }` で固定）。B1 が改名の後の列名を確かめるので、同梱の SQLite を下げれば red になる | B1 |

## Boundary / Wire Contract

- producer: DB の 6 列（vB の後）、BIZ の `Product`・`PriceHistoryEntry`・`ImportRow`・`CostDiff`・記録詳細の明細の型、cmd の `ProductResponse`。frontend の Request（`ProductCreateRequest`・`ProductUpdateRequest`・`PriceRevisionInput`・`ReceivingItemInput`・`DisposalItemInput`）。
- consumer: frontend の商品フォーム・価格改定・入庫と原価差分の dialog・廃棄・記録詳細・商品一覧・在庫照会の詳細・価格の履歴。BIZ の検証と金額の関数。
- wire type: 13 型の原価の field を `cost_price_centi`・`valuation_cost_price_centi`・`old_cost_price_centi`・`new_cost_price_centi`・`master_cost_price_centi`・`received_cost_price_centi`（`number`、`ProductUpdateRequest` は `cost_price_centi?: number | null` と Serialize の `number | null` の今の形）。行の金額 `line_cost_centi`・`line_loss_cost_centi`（十進の文字列）と円の合計 `total_cost`・`total_loss_cost`（`number`）は今のまま。`ProductStockUnit = "pcs" | "sheet" | "hon" | "bag" | "box" | "kumi" | "set" | "ball" | "cho" | "m"`。
- internal type: Rust の原価は `i64`（1/100 円）、金額の計算は `i128`（`cost_line_centi`）。TS の原価は `number`（1/100 円。上限 `99999999` で安全な整数）、廃棄の合計と棚卸し記録詳細のロス原価は `BigInt`。
- precision/range: Y1（`0`〜`99999999`）。行の金額は十進の文字列のまま（棚卸しの行は number の範囲を越えうる。SPEC-UNIT-D6）。
- round-trip path: 画面の文字列 → `parseCostInput` → 1/100 円の number → specta → BIZ の検証 → DB → 読取り → bindings → 原価の formatter（`¥83.33`）と入力の欄の初期値（`83.33`）。入力の欄の初期値を `parseCostInput` に戻すと同じ 1/100 円になる（C12）。
- invalid input: Y2 に合わない文字列は画面で止まる。UI を通らない command は BIZ が上限と負を拒む。商品 CSV の `原価` は行の error。`roll`・`cm` は enum に無いので deserialize で拒まれ（command の引数）、商品 CSV では行の error、DB の CHECK でも拒む。
- compatibility: 旧い名前の field を受ける互換は置かない（vB の前の版の DB は vB で止まり、bindings は同じ PR で再生成する。名前の違いで古い読み手を止めるのが `_centi` の目的。SPEC-UNIT-D5）。記録済みの操作ログの detail_json に原価の key（`cost_price`）を持つ行は書き換えない（作り直した DB には無い）。操作ログの画面は detail の key を生で出す（`src/features/operation-logs/OperationLogsPage.tsx` の `KNOWN_KEYS[key] ?? key` と技術情報の JSON）ので、商品の更新と価格改定の log の key を `cost_price_centi` にすれば、key の名前で値の単位が読める（S3）。

## Test Plan

Test Design Matrix: [2026-10-21-cost-lane-c](test-matrices/2026-10-21-cost-lane-c.md)。

- If the Human Gate includes L3, Writer completion includes `cargo check --release` before the owner native build; this is not a CI gate.
- targeted tests: `cd src-tauri && cargo test --lib db::` / `cargo test --lib biz::product_service` / `cargo test --lib biz::inventory_service` / `cargo test --lib biz::unit_amount` / `cargo test --lib biz::stocktake_service` / `cargo test --lib cmd::` / `cargo test --test seed_test`、`npm test -- src/lib src/features/products src/features/receiving src/features/disposal src/features/inventory-records src/features/stock-inquiry`。
- negative tests: Matrix の B4〜B6（vB が止まる・戻る）、B8・B10（作り直しの後の CHECK。正の対照と error の文つき）、C1〜C3・C7・C10（上限・解析の error）、C22（原価の検査より先の商品の取得と冪等性の判定の順）、T2a の (i)（旧い名前で書いて失敗だけを確かめていた test を直したもの）、U4（巻・cm の語の行の error と直し方の文）。
- compatibility checks: v8 の CHECK と上限の履歴の test（T4）、vB の前の版に行を入れる migration の test を上限 9 で回す（T1）。
- data safety checks: fixture は全部合成。L3 の DB の作り直しは消さずに移す（Data Safety）。
- main wiring/integration checks: B1（`init_database` の新規の DB で vB まで当たる）、R1〜R4（作り直しの後の起動）、C14〜C21（画面が command の新しい field を送り・出す）。

L3（Human Gate `manual`。owner は 2026-10-21 から数日は手が使いにくいので、Draft のまま溜め、1 round で PASS / FAIL を返してもらう。作り直しの file の移動は owner の指示で Coordinator が行う）:

| # | 到達経路 | 入力物 | 観測可能な合格基準 | 依存する既知 backlog |
|---|---|---|---|---|
| L3-1 | (0) L3 を頼む前に、Coordinator が owner の今の L3 用の DB の写しを作り、写しを読み取りだけで開いて `SELECT COUNT(*) FROM products` が 1 以上で、版の記録の表 `schema_versions` の最大の `version` が vB の直前の版（`VB_VERSION - 1`）であることを確かめる（件数が 0 なら vB がそのまま当たり、版が古ければ先の migration が別の文で止まりうるので、どちらも止まりを観測できない。違えば L3-1 の入力物を作り直してから頼む。元の DB は動かさない）。(1) 新しい版を起動する（owner の今の L3 用の DB のまま）。(2) 止まったら、Coordinator が 22 §16 の作り直しの手順で移す（下の Data Safety。移す前に `%APPDATA%\com.kosei.inventory` と `%LOCALAPPDATA%\com.kosei.inventory` の中の folder の名前の一覧を取る）。(3) もう一度起動する | owner の今の L3 用の DB（商品の行がある） | (1) で起動が止まり、「データベースの準備に失敗」の文の下に「商品か原価の記録がある DB はこの版へ移せません。原価を 1/100 円で持つ版は空の DB から始めます。開発・試験 DB は作り直してください（22 §16 の作り直しの手順）」と行のある表名が出る。(3) で空の DB で開く（商品一覧が空）。(2) の一覧で WebView の保存の folder がどちらにあったかを Coordinator が記録する（Contract Probe P5） | 作り直しの手順は 22 §16（D-123） |
| L3-2 | 起動した版で、商品登録を開く → 単位の select を開く → 単位 `個` で原価の下の「袋の値段から出す」を開き、袋の値段 `1000`・入り数 `12` で「原価に入れる」→ 補助を閉じて開き直す → 単位を `m` に変える → `個` に戻し、下の入力物で保存する | 画面の入力だけ。固定の入力: JAN は空、商品名 `合成の試験品`、部門 `その他小物`（作り直した空の DB にある初期の部門で、`code_prefix` `KM` で独自コードを発番できる: `src-tauri/src/db/schema_v1.rs` の `departments` の初期データ）、売価 `100`、原価は補助で入れた `83.33`、初期在庫 `0`（既定のまま）、税率は既定のまま | select が 10 個（巻・cm が無い）。原価の欄が `83.33`。開き直した補助の 2 つの欄が空。`m` で補助が隠れ、label が `原価（税抜、必須、1 m あたり）`。価格の節の説明が `売価は税込の円の整数、原価は税抜の円で小数 2 桁まで入力します。長さ（m）の商品は 1 m あたりの値段です。`。保存の後、商品一覧の原価が `¥83.33`（記号は半角の `¥`。同じ行の売価の表示は lane の前と変わらない） | — |
| L3-3 | L3-2 の商品（売価 `100`・原価 `83.33`）で、入庫（数量 `3`・原価 `83.33`）を保存 → 記録詳細を開く → 入庫（数量 `1`・原価 `85`）を保存し、原価差分の dialog で「マスタ原価をこの実原価に更新する」→ 価格改定で新売価 `110` を入れる | 画面の入力だけ | 記録詳細の原価 `¥83.33`・原価小計 `249.99 円`（8333 × 3 = 24999）・原価合計 `¥250`（24999 を円未満で四捨五入）。dialog のマスタ原価 `¥83.33`・今回の実原価 `¥85`、更新の後の商品の原価 `¥85`（売価は `100` のまま）。価格改定の現原価 `¥85`・現掛率 `85.0`（8500 ÷ 10000）・新原価（案）`93`（`floor(110 × 8500 ÷ 10000) × 100` = 9300）。記号は全部半角の `¥` | — |

- 原価の上限・解析の error の見た目、商品 CSV の preview の行の error の文、廃棄の候補の一覧・在庫照会の詳細・価格の履歴・価格改定の現原価の原価の表示は自動 test（C1〜C21、U4〜U6。在庫照会の詳細・価格の履歴・価格改定の現原価は C19〜C21）に任せ、Coordinator が Ready の依頼と同時に frontend の test の render か Vite の dev の画面の写しを owner に渡し、見た目の確認（Human Visual Confirmation）を L3 と同じ 1 回の PASS / FAIL に含める。写しを作れなければ、その残りの危険を PR の本文に書き、受け入れを Ready の判断に含める。
- fixture: L3-1 は owner の今の DB を使う（新しい fixture は要らない）。L3-2・L3-3 は画面の入力だけ（商品 CSV の取込みは L3 に入れないので、CP932 の fixture は要らない）。

## Review Focus

- vB: 起点の検査が DDL の前にあるか、手順 4 が §19 の写しの 3 列・CHECK・index・外部キーを落とさないか、止まりの文が原因と直し方を名指すか、版の番号と並走の §17・§18 の migration との順（取り直しの対象が Scope と T1 にあるか）。
- 円の値が 1/100 円の列・wire に入る経路、または 1/100 円を円として出す画面が残っていないか（Y1〜Y4、AC2〜AC4）。浮動小数の経路（Y2・Y5、demo の seed）。
- 既存の test の移し方（T1〜T5）で、期待を弱めていないか。`cm` の入力の分岐の case を外すとき、同じ意味の `m` の case があるか（T3）。
- 単位の 10 個が DB の CHECK・enum・bindings・formatter・商品 CSV・select・在庫少の並び・seed で同じか（Y9）。
- D-128 と正本の直し（税抜、手順 4 の分岐の除去、10 cm の刻み）が、店の答えの要旨を超えていないか。この変更で不要になる記述・条件が残っていないか（D-127）。
- 公開 repo に実データを書いていないか。並走の lane の節を触っていないか（所有表）。

## Contract Ledger

触る設計正本の節の契約を行にした（隣接の契約の sweep: SPEC-UNIT-D1〜D11 と「入力の上限と安全な整数の範囲」、22 §3.2・§16・§18・§19、master-tables の「単位と原価の精度の契約」と `products`・tracking・transaction の原価の列、20・21・30・31・34・35・40・44・51・58・61・62・63・64・77 の原価と単位の節を当て、Scope が触り得ないものは下の「除外」に理由つきで書いた）。

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| MNT-03-D14（vB の手順 1〜6・止まりの文・番号） | 22 §16 | S1 | B1〜B8・B10（B9 は条件つき） | L3-1 |
| MNT-03-D1（共有の helper） | 22 §3.2 | S1 | 構造の test（file の列挙に足す） | — |
| 22 §16 の作り直しの手順と確かめていない前提 | 22 §16 | S1（起動の test）、Data Safety | R1〜R4 | L3-1（P5） |
| MNT-03-D16 の `products` を作り直す規則（§18 が先に入ったとき） | 22 §18 | S1（条件つき） | B9（条件つき） | 非対象: §18 が後なら §18 の lane の義務 |
| SPEC-UNIT-D1（10 個の一覧、巻・cm を外す） | 共通規則 | S1（CHECK）、S6 | B3・U1〜U3 | L3-2 |
| SPEC-UNIT-D2（理由の更新、振舞いは変えない） | 共通規則 | — | 既存の数量の test（変えない） | 非対象: 刻みの検査（Q-098） |
| SPEC-UNIT-D3（`cm` の入力と表示の分岐を外す） | 共通規則 | S6 | T3 の `m` の case | — |
| SPEC-UNIT-D5（精度・入れ方・税・補助・入力・上限・表示〈半角の `¥`〉・error の文） | 共通規則 | S3・S5 | C1〜C3・C7・C10〜C21、AC7 の文の検査 | L3-2・L3-3 |
| SPEC-UNIT-D6（`cost_line_centi` に 1/100 円をそのまま） | 共通規則 | S3 | C4〜C6 | — |
| SPEC-UNIT-D9（formatter の網羅） | 共通規則 | S6 | U5 | — |
| SPEC-UNIT-D10（在庫少の並び、商品 CSV の `在庫単位`・`原価` と行の error の文） | 共通規則 | S3・S6 | C7・U3・U4 | — |
| SPEC-UNIT-D11（wire の enum） | 共通規則 | S4・S6 | AC6、U1 | — |
| 入力の上限と安全な整数の範囲の原価の行・(b)・(c) | 共通規則 | S3・S5 | C1〜C3・C6・C11 | — |
| 単位と原価の精度の契約・`products` の列 | master-tables | S1・S2 | B1・B3 | — |
| UI-01b-D22（原価の label と説明、上限）・D23（個数の単位どうしの変更、候補 9 個）・D24（補助） | 51 | S5・S6 | C14・U6 | L3-2 |
| UI-02-D7（入庫の原価の入力）・UI-02-D15（原価差分の dialog） | 61 | S5 | C16 | L3-3 |
| UI-05-D8（廃棄の原価の入力） | 64 | S5 | C17 | — |
| 77 の新原価（案）と現掛率（SPEC-PRV-D4） | 77 | S5 | C15 | L3-3 |
| BIZ-01（原価の検証と順、商品 CSV の `原価` の小数、update の log の key） | 30 | S3 | C1・C7・C22 | — |
| BIZ-02（明細の原価の検証と順、§12.3 の原価差分、§12.8 の fingerprint、記録詳細の金額） | 31 | S3 | C2・C4・C5・C8・C9・C22 | — |
| SPEC-STK-VAL-D3・D4 の評価原価（旧本体、test だけ） | 35 §20.5a | S3 | 旧本体の test（T2） | 非対象: 評価額の出力の文と記録詳細のロス原価の BIZ への移動（(b)） |
| CMD の DTO の原価の field（`ProductResponse` ほか） | 40・44 | S4 | AC6 | — |
| D-128（税抜、手順 4 の分岐の除去、10 cm の刻み） | decision-log | 本 commit の正本、S3・S5 | C14 | L3-2 |
| D-123（作り直しの承認と順） | decision-log、22 §16 | Data Safety | — | L3-1 |

除外（触らない契約と理由）: SPEC-UNIT-D4（価格の基準数量と POS の数量 1。変えない）、D7（売価 × 数量。売価は円のまま）、D8（Z004・EJ の数量）。22 §17（SD 直読みの migration、PR #172）・§19 の手順（vL は v9 で済み。本 lane は §19 の列を写すだけ）。58 の在庫照会の数量の表示（原価の行だけを Y3 にする。数量の表示は変えない）。62・63（手動販売・返品は原価を持たない。`cm` の入力の記述だけを S7 で直す）。PLU の書出し（25。単価は売価）。停止 ADR の SPEC-STOP-D1〜D6 は変えない。

## Contract Probe

本 lane が依る外部の前提（同梱の SQLite の振舞い）を、2026-10-21 に scratch の crate（`rusqlite = "=0.31.0"` bundled。repo の外の作業 folder。commit しない）で確かめた。

- P1 原価の 6 列の `RENAME COLUMN`（`foreign_keys = OFF`、1 つの transaction）: `sqlite_version()` -> `3.45.0`。`products`・`receiving_items`・`disposal_items`・`stocktake_items`・`price_history` の 6 列を改名 -> ok、`pragma_table_info` に新しい名前だけがある（`products`: `product_code,department_id,selling_price,cost_price_centi,stock_unit,stock_recheck`、`price_history`: `…,old_cost_centi,new_cost_centi`）。行のある表の改名は値を変えず（`8333` のまま）、古い名前の `SELECT` は `no such column` で止まる（`_centi` の名前が古い SQL を止める前提。SPEC-UNIT-D5）。
- P2 改名の後の `products` の作り直し（`products_new` を作って写し、`DROP TABLE products`、`ALTER TABLE products_new RENAME TO products`、index の作り直し）: 同じ transaction で ok。`PRAGMA foreign_key_check` 0 行、COMMIT の後の `integrity_check` -> `ok`。子の表の外部キー（`receiving_items` -> `products(product_code)`、`stock_counts` -> `products`・`stocktake_items`）は残り、存在しない商品の子の行は `FOREIGN KEY constraint failed`。index は `sqlite_autoindex_products_1`・`idx_products_department_id`。
- P3 作り直しの後の CHECK: `pcs` `sheet` `hon` `bag` `box` `kumi` `set` `ball` `cho` `m` -> ok、`roll`・`cm`・`kg` -> `CHECK constraint failed: stock_unit IN ('pcs','sheet','hon','bag','box','kumi','set','ball','cho','m')`。
- P4 §19 の写しの 3 列の CHECK（`stock_range_high` の列の CHECK が他の列を参照する）を作り直しで写す: (a) の lane の Contract Probe P1 で、同じ SQLite で列の CHECK が他の列を参照できることを確かめた（`docs/archive/plans/2026-10-18-stocktake-count-runtime-a.md`）。本 lane の作り直しは CREATE TABLE の中に同じ CHECK を書くので、列の追加の制約（P1 の 3.37.0 以降）に依らない。B5 で作り直しの後に CHECK が効くことを確かめる。
- P5 WebView の保存（localStorage）が Windows で `%LOCALAPPDATA%\com.kosei.inventory` の下（WebView2 の既定では `EBWebView`）に置かれる: Tauri の既定の振舞いで、本 run は確かめていない -> 未確認。作り直しの手順は識別子の名前の folder を `%APPDATA%` と `%LOCALAPPDATA%` の両方とも丸ごと移すので、置き場がどちらでも移る（22 §16 の「確かめていない前提」）。L3-1 の (2) で folder の名前の一覧を取って確かめる。

## Data Safety

- commit しない物: 実 POS / 店舗データ、DB、backup、log、receipt。fixture は全部合成（商品コード・JAN・原価・日付は合成の値）。U1 の調べの対象（在庫 Excel）の中身と、店の商品の名前・原価の実値を書かない（行の数だけ）。
- local だけの場所: owner の PC の `%APPDATA%\com.kosei.inventory`（DB と `-wal`・`-shm`、`backups`、`logs`、`images/receipts`、復元の遺物）と `%LOCALAPPDATA%\com.kosei.inventory`（WebView の保存）、アプリを起動する方法ごとの作業フォルダの旧 DB 一式、設定した backup の保存先（`app_settings` の key `backup_path`）の backup の file。
- 作り直し（L3-1 で vB が止まったとき。D-123 でリリースの前まで包括で承認済み）: 対象は上の local だけの場所の全部。保全は 22 §16 の作り直しの手順のとおり、日付の付いた 1 つの folder（例 `%USERPROFILE%\inventory-db-moved\2026-10-2x\`）の下に元の場所ごとの sub folder を分けて**消さずに移す**（手順 1 の `backup_path` の控え、手順 2 のアプリの停止、手順 4 の移した元が残っていないことの確認を含む）。(a) の lane で作り直した folder があれば、別の日付の folder にし、上書きしない。実行は owner の指示で Coordinator（WSL の `/mnt/c/Users/...` から移す。消す command を使わない）。移した folder は残す（消すかは owner の判断）。
- 合成だけの場所: Rust と TS の test の fixture、demo の seed、L3-2・L3-3 の画面の入力。

## 残りの危険（起票の時点）

- vB の止まりは、起動の画面の今の `DatabaseInit` の文（「アプリを再起動してもう一度お試しください」を前置きに持つ）に原因と直し方を添えて出す。前置きは再起動で直らない止まりに合わないが、§19 と同じで、出会うのは開発者だけ（本番の DB は無く、店の PC へ初めて入れる版は空の DB で始まる。D-123 (4)）。文の種類を分けるのは (a) の lane の残りの危険と同じ候補。
- vB の前の版で作った backup を新しい版で復元すると、open の migrate が vB で止まり、復元は巻き戻る（[71](../../function-design/71-mnt-backup.md) §71.7。利用者には失敗に見える。22 §16 の「作り直しの後に残す物」の文のとおり。開発・試験 DB にだけ起きる）。
- 並走の §17・§18 の migration と番号を取り合う。後から merge する側が file 名・登録・版の期待を取り直す（所有表）。取り直しの漏れは版の期待の test が red にする。
- 1/100 円の原価で `売価 × 原価` を TS の number で掛ける所（価格改定の twin）は、共通規則 (b) のとおり `999999 × 99999999` 以下で安全な整数の範囲に入る。上限を変える lane はこの式を見直す。
- 商品 CSV の `在庫単位` が `巻`・`cm` の行は、文が直し方を名指すが、直すのは CSV を外で直して選び直す今の経路のまま（preview の画面の中で直す経路は backlog）。最新の在庫 Excel で巻・cm は 0 行（U1）なので、当たる見込みは小さい。
- 入庫・廃棄・価格改定の原価の欄の見出しは「原価」のままで、税抜と書くのは商品フォームの label だけ（D-128 (1) は明細の原価も税抜）。仕入先の伝票の税込の単価を入れうる。見出しを直すかは backlog の UI を整えるターンの材料（出所は Plan Review の AI review）。
- 商品一覧・在庫照会の詳細・価格の履歴では、原価が半角の `¥`、同じ画面の売価が今の全角の `￥` になり、1 つの画面に 2 つの記号が並ぶ（売価の記号は本 lane で変えない: Non-scope。揃えるのは backlog の「金額の表示の書き方を 1 つにそろえる」）。

## Implementation Results

初回実装（[PR #184](https://github.com/higekiri-w/inventory-system-desktop/pull/184)）。Scope の S1〜S7 を 2 段の Writer の run で実装した。

- S1: vB は版 10（`src-tauri/src/db/schema_v10.rs`）。手順 2 の止まりの文、6 列の改名、`products` の作り直し（CHECK 10 個、§19 の写しの 3 列・`tax_rate` の CHECK・外部キー・index を保つ）、検証と COMMIT の失敗の巻き戻し。Matrix の B1〜B8・B10 と R1〜R4。既存の migration の test は T1・T4 のとおり移した。
- S2〜S4・S6（Rust）: 列名と内部・wire の型を `_centi` に、上限を `MAX_COST_PRICE_CENTI` に、`cost_line_centi` の `* 100` を外し、商品 CSV の `原価` を `parse_hundredths`（`parse_meters_to_cm` を改名）で読む。`ProductStockUnit` から `Roll`・`Cm` を外し、商品 CSV の `巻`・`roll`・`cm` の行の error に `retired_stock_unit_hint` の直し方を添える。bindings を再生成。
- S5・S6（TS）: `parseCostInput`・`formatCostInput`・`unitCostFromPack`（`src/lib/unit-quantity.ts`）と `formatCostYen`（`format-stock-display.ts`）を原価の全ての入口と画面で使う。商品フォームの補助と税抜の label、価格改定の Y8 の式、廃棄の合計の BigInt の twin（形式の誤りと上限越えの行は 0）、棚卸し記録詳細のロス原価の BigInt。
- S7: 正本の同期（共通規則・22 §16・master-tables ほか、`proposed・未実装` の注記を現行の記述に）、ERD、traceability の再生成。
- 計画からの外れと、AC の書き漏れ（AC5 の分類に U1 の enum の test が無い）は PR の本文に書いた。
- 結果: 原価を 1/100 円の列（`*_centi`）で持ち、商品フォーム・入庫・廃棄・価格改定で小数 2 桁の円を入力し、半角の `¥` で表示し、巻・cm を単位から外した。起点の migration vB（版 10）は商品か原価の記録がある DB で何も変えずに止まり、開発・試験 DB は 22 §16 の作り直しで空から始める。原価は税抜（D-128）。
- review・CI・merge（closeout、2026-10-21）: Final Review は broad 3 本（Codex 482〈Astra〉、Claude 側 Fable 5.1、fresh Opus の深掘り）がどれも approve（P3 だけ）で、P3 の是正（E1〜E5 のうち E1〜E4）と main（PR #183・#186）の取込みの base 同期の後、closure 2 本（Codex 486〈Sol〉、Claude 側 Fable 5.1）がどちらも approve（P0）。L3-1〜L3-4 は PASS（owner、Windows native、repo 外の回答台帳 TD-370）。helper の専用 record に broad・closure・manual を記録し、Ready の後の run の `Merge gate` は pass で、owner の Ready・merge の承認（TD-371）の後に helper 経由で squash merge した。PR: [#184](https://github.com/higekiri-w/inventory-system-desktop/pull/184)
- 自動 test に任せた所: 商品 CSV の preview の行の error の文、原価の上限の error、廃棄の候補の一覧の原価の見た目は L3 で見ていない（PR の本文の残りの危険）。
- 後続: `docs/backlog.md` に、原価の形式の error の文を 1 つの定数にする項目（Final Review の E5）。

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

## Review Response

先行 round の結果・評価（判定・件数・採否・reviewer の意見）はこの節にだけ書く。前半の節と遷移記録には「round N の是正（Review Response 参照）」だけを書く（独立 review は `## Review Response` より前だけを読む）。

### Plan Review round 1（2026-10-21、head `ecb94c3b`）

Claude 側 fresh Opus 5.5 の全体（reject、P1 0・P2 4・P3 7、F1〜F11。Ordinary Operation は成立）、Claude 側 fresh Opus 5.5 の Scope の網羅の深掘り（reject、P1 0・P2 2・P3 6、#1〜#8）、Codex 465（Astra、reject、P1 0・P2 3・P3 3、#1〜#6。Ordinary Operation は外部前提 P5〈WebView の保存の置き場〉が未確認で、L3-1 で確かめる）。重なりは 1 行にまとめた（「全体 F」= Opus 全体、「網羅 #」= Opus 網羅、「Codex #」= Codex 465）。裁定は Coordinator。AI の review から出た製品の要件（R8 の文の内容、R10）は出所を書き、owner の確認の前に Human Gate の必須にしない。

| # | 出所 | 重大度 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|---|
| R1 | 全体 F1 | P2 | 採る | D5 が「記号は各画面の今の形」と「1 つの formatter」を同時に求めるが、今の画面の原価の記号は全角の `￥`・半角の `¥`・`1,000 円` が混ざる | 10 SPEC-UNIT-D5 の表示を半角の `¥` に一意にし、売価の記号は変えないと書いた（更新履歴も）。packet の「起票時に起草役が決めたこと」の事実の文を現物に直し、Y3・S5（商品一覧・在庫照会の詳細・価格の履歴・原価差分の dialog）・Non-scope・残りの危険（1 つの画面に 2 つの記号）・L3-2・L3-3 の記号。Matrix の Contracts Under Test、T5 に「期待を変える: 原価の表示の記号」の行（`rg` の出力から: `ProductTable.test.tsx`・`ProductForm.test.tsx`・`ReceivingPage.test.tsx`）、Adjacent Pattern Audit の Y3。S7 に 58 の表示の書式の表 |
| R2 | 全体 F2 | P2 | 採る | 在庫照会の詳細・価格の履歴・価格改定の現原価の原価の表示を固定する test が無い | Matrix に C19〜C21。Test Plan の「自動 test に任せ」の文を C19〜C21 に向け直した。AC7 を C10〜C21 に |
| R3 | 全体 F3 = Codex #1 | P2 | 採る | 商品 CSV の `原価` の `-1` の期待が S3（`原価が不正です`）と C7（負の文）で食い違う | 負の文を保つ。Y2・S3 に「先頭が `-` で残りが規則に合えば負の文、それ以外の形の誤りは形式の文」と書き（`-0` も負の文）、C7・Negative Paths を揃えた。10 SPEC-UNIT-D10 に同じ分け方 |
| R4 | 全体 F4 | P2 | 採る | B2 の oracle が `products` 自身の外部キー・型・NOT NULL・既定値・`tax_rate` の CHECK を見ない | B2 を既存の helper `products_definition` の前後比較（`cost_price` の名前だけを差し替えた期待）にし、B10（`tax_rate = '5'` の INSERT が拒まれる）を足した。FM2、AC1、Contract Ledger、22 §16 の vB の test の文 |
| R5 | 網羅 #1 = 全体 F6 | P2 | 採る | 原価の欄の形式の error の文の置き換えが Scope に無く、今の文（整数）を期待する test が T5 に無い | S5 の商品フォーム・価格改定・入庫・廃棄の 4 行に SPEC-UNIT-D5 の文（売価の文と分ける）。Matrix の T5 に「期待を変える: 原価の形式の error の文」の行（`rg` の出力の 5 か所）。AC7 に 4 file の新しい文の検査と production の `原価は0以上の整数` が 0 行。S7 に 51 §7.6 の `cost_price` の行 |
| R6 | 網羅 #2 | P2 | 採る | 商品フォームの note `長さ（m・cm）の商品は…` が S6 から漏れる | S6 の select の行に note と comment。AC5 に `rg -n 'm・cm' src --glob '!*.test.*'` が 0 行（baseline 4 行）。S7 の引用符つきの `cm` の command で 51 UI-01b-D6 の `stock_unit='cm'` の 2 行を名指した |
| R7 | Codex #2 = 全体 F9 | P2 | 採る | L3-2 の登録の入力（商品名・部門・売価）が決まらず、L3-3 の期待が条件つき | L3-2 に固定の入力（合成の商品名、初期の部門 `その他小物`〈`code_prefix` `KM`。`schema_v1.rs` の初期データで確かめた〉、売価 `100`、初期在庫 `0`）。L3-3 の条件を外し、期待を固定の入力から計算し直して式を添えた（値は同じ） |
| R8 | Codex #3 + 全体 F7 | P2 / P3 | 一部採る | 商品 CSV で新しく拒む巻・cm の行を画面で直す経路が無く、`原価` の error が直し方を言わない | error の文で直し方を名指すところまでを Scope にした: `原価` の形式の誤りは SPEC-UNIT-D5 の文（S3）、`在庫単位` の `cm` は m で入れて初期在庫も m の数にする旨、`巻`・`roll` は 個 で入れる旨（S6・U4・10 SPEC-UNIT-D10）。preview の画面の中で行を直して再検証する経路は、今の CSV の error 全部に共通の制約で本 lane の範囲を超えるので採らない（Non-scope、backlog の保留に 1 項目、残りの危険に 1 行〈巻・cm は最新の在庫 Excel で 0 行〉） |
| R9 | 全体 F5 = 網羅 #3 | P3 | 採る | 長さどうしの単位の変更を拒む既存の case が T3 のどの分類にも無い | T3 に「長さどうしの単位の変更を拒む case を契約ごと外す」の行（3 つの test。種類をまたぐ case が残ることを対で示す。順の確かめは残す）。網羅 #3 の 4 つ目（既存の単位の保持）は 1 対 1 で `m` へ（T3 の fixture の行） |
| R10 | 全体 F8 | P3 | 残りの危険へ | 入庫・廃棄・価格改定の原価の欄の見出しに税抜が無い | 残りの危険に 1 行と、backlog の保留（見た目・UX、UI を整えるターンの材料）に 1 項目。Scope に入れない（Non-scope）。理由: 3 画面の見出しの文言は AI の review から出た製品の要件で、画面単位の design と一緒に決める方が筋がよく、本 lane の Goal（原価の精度と単位の一覧）を前進させない |
| R11 | 全体 F10 | P3 | 採る | B の test が v9 を固定で書き、番号の取り直しの範囲が読めない | Matrix の B の前書きに定数 `VB_VERSION` と `VB_VERSION - 1`、B2・B4 を書き直した。所有表の取り直しの対象に足した |
| R12 | 全体 F11 | P3 | 採る | 同じ規則の関数が既にあるのに新しい関数を書く形 | Y2 に `parse_meters_to_cm`（単位に依らない名前へ）と `readQuantity` の `m` の分岐、Y5 に `divideRoundHalfUp`。S3・S5。Matrix の Adjacent Pattern Audit に 2 行 |
| R13 | 網羅 #4 | P3 | 採る | TS の test は入力の文字列を渡すので、`cm` → `m` で 100 倍になる | T3 に TS の入力の文字列の行（数える command と 58 行・11 file。入力を 1/100 に、上限・0 の case の扱い） |
| R14 | 網羅 #5 | P3 | 採る | 価格改定の操作ログの detail の key が S3 に無く、compatibility の文が事実と違う | S3 の商品の型の行に `product_price_revise` の detail の key。Boundary / Wire Contract の compatibility を「操作ログの画面は key を生で出す」に直した |
| R15 | 網羅 #6 = Codex #5 | P3 | 採る | S7 の網が時点つきの原価の記述・引用符つきの `cm`・ERD の版の範囲を拾わず、command が packet と Matrix 自身を数える | S7 に command を 2 本（時点つきの原価の記述と `× 100`、引用符つきの `cm` と `m・cm`）。6 本とも `--glob '!docs/plans/**'` を付けて baseline を取り直した。ERD の行に版の範囲。AC10 を 6 本に |
| R16 | 網羅 #7 | P3 | 採る | 原価の欄の属性（`step` 1・`inputMode` numeric）が小数の入力に合わない | S5 の 4 画面の行に `inputMode="decimal"`・`step="0.01"`（`type="number"` の欄だけ `step`）。先例 `quantityInputAttributes`。Adjacent Pattern Audit に 1 行 |
| R17 | 網羅 #8 | P3 | 採る | S2 の command が test の生の SQL・JSON の文字列を数えない | S2 の内部の型の行に、AC2 の file の一覧で拾って直すこと（`{"cost_price":null}` の case を含む） |
| R18 | Codex #4 | P3 | 採る | AC1 の探索先・filter が起動の test の置き場と合わない | AC1 を DB 群と起動群に分け、起動群は `src-tauri/src/lib.rs` と `cargo test --lib test_startup_req903_` |
| R19 | Codex #6 | P3 | 採る | 商品 CSV の test の例の名前が `req103` | C3・C7・U4 を `req104` に、Registration に REQ-104 |

是正で足した契約の文（D5 の記号、商品 CSV の `原価` の error の分け方、`在庫単位` の直し方の文、L3 の固定の入力、原価の欄の属性、B2 の oracle）を境界の 4 観点で一巡して足したこと: (1) 安全な整数の範囲 — 規則に合うが `i64` に収まらない `原価` は `原価が大きすぎます`（Y2・C7）。(2) 0 — 商品 CSV の `-0` は今は 0 で受かるが、負の文に分けた（Y2・C7）。(4) 単位や種類の混在 — `巻` は個数の単位で m と換算しない（SPEC-UNIT-D1）ので、`巻` の行の直し方を m でなく 個 にした。原価と売価の記号が 1 つの画面に並ぶことを残りの危険に書いた。(3) 失敗の時点と順序は、行の中の error の並びを今のまま（Y2）と書くだけで、変わる所は無かった。

### Plan Review round 2（2026-10-21、head `5e00d4e8`）

Claude 側 fresh Opus 5.5（reject、P1 0・P2 1・P3 7、#1〜#8。Ordinary Operation は成立）と Codex 468（Astra、reject、P1 0・P2 2、#1・#2。Ordinary Operation は外部前提 P5 が未確認）。裁定は Coordinator で全部採った。次の round 3 が天井なので、同じ型の穴を repo 全体で洗ってから直した（Q1・Q2・Q4・Q6）。

| # | 出所 | 重大度 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|---|
| Q1 | Opus #1 | P2 | 採る | 最新の版の DB に旧い列名で書き、失敗だけを確かめる test が T2 の「vB の前の版の fixture」に紛れ、改名の後に中身が抜けて緑のまま残る | Matrix の T2 の「変えない」を `VB_VERSION - 1` 以下の DB に限り、「最新の版の DB に当てる test は列名・key を `_centi` に直す」の行を足した（負の INSERT は error の文か正の対照）。repo 全体を洗った表を T2a に置いた（SQL の書込み 66 か所・JSON の key 9 行を、(i) 改名で中身の抜ける 4 test〈Opus が挙げた `migration.rs` の 2 つに加え、`disposal_repo.rs` の `test_insert_disposal_req204_check_disposal_type` と serde の null の case〉、(ii) vB の前の版の fixture 32 か所、(iii) 直さないと red になる 30 か所・8 行、に分けた。TS は typecheck が止めるので対象外）。AC2 を hit ごとの表にした。S2 の生の SQL・JSON の文を T2a に向けた |
| Q2 | Codex #1 | P2 | 採る | Y1 の「DB を見ない検証の段で」が、商品の取得（修正・価格改定）と冪等性の判定（入庫・廃棄）が先の今の順と食い違う | Y1 を「各入口の今の検証の位置と優先順位を保ち、原価の検証に届いたときに拒む。書込みは行わない」にし、入口ごとの先の検査を書いた（他の入口も現物で確かめた: 登録・CSV の preview と commit・棚卸しの旧本体）。Ordinary Operation の「失敗と再試行」を合わせ、Matrix に C22（`NotFound`・`IdempotencyConflict` が先）を足した |
| Q3 | Codex #2 | P2 | 採る | AC5 の Rust の残りの分類に、S6 が求める商品 CSV の本番の拒否の分岐が無い | S6 に判別の関数 `retired_stock_unit_hint`（`product_service.rs`）を名指し、AC5 に「本番の分岐」の分類と、test の module より前の行だけを見る検査の command（baseline 1 行）を足した |
| Q4 | Opus #2 | P3 | 採る | 棚卸し記録詳細のロス原価の表示の期待の変更が T5 に無い | T5 に `"¥600"` → `"600.00 円"` の行。formatter を替える画面の test の表示の期待を `rg` で洗い、T5 の外で形が変わるのはこの 1 行だけと確かめた |
| Q5 | Opus #3 | P3 | 採る | 価格改定の新原価（案）と現売価 0 の現原価が入力の欄の初期値の場所に無い | 10 SPEC-UNIT-D5 の初期値の場所（更新履歴も）、S5 の価格改定の行、C15 に「欄が `91`、送る値が `new_cost_price_centi: 9100`」と現売価 0 の `83.33` |
| Q6 | Opus #4 | P3 | 採る | S7 の command に当たらない、原価を整数と書く行が残る | S7 に 30 §4.8 の手順 4 と `SCREEN_DESIGN.md` の入庫の行を名指した（`rg -n '原価[^。]{0,12}整数|整数[^。]{0,6}原価'` で洗った）。「6 つの command の出力で全部」を名指しの行を含む形に直した。同じ型の穴として、S7 の 4 本目の command を AC2 と同じ名前の一覧にした（旧い 4 つの名前では `-w` が wire の名前に当たらず、15 行を拾わなかった。115 → 130 行、26 file のまま） |
| Q7 | Opus #5 | P3 | 採る | B8・B10 が負の形だけで、旧い列名の helper を写すと常に緑 | B8・B10 に正の対照と `CHECK constraint failed` の確認を足し、`can_insert_unit` が旧い列名で書くことを書いた |
| Q8 | Opus #6 | P3 | 採る | L3-1 の入力物（商品の行がある DB）を確かめる手順が無い | L3-1 に (0) の段（写しを読み取りだけで開き `SELECT COUNT(*) FROM products` が 1 以上。0 なら入力物を作り直す） |
| Q9 | Opus #7 | P3 | 採る | AC1・AC7 の test 名が Matrix の目安の名前と食い違いうる | AC1・AC7 の test 名を「Matrix の名前、または PR 本文の対応表の名前」にした |
| Q10 | Opus #8 | P3 | 採る | demo の seed の「同じ値の 100 倍」は誤り | S2 の seed の行を「売価 × 0.6 円を 1/100 円で正確に。今の値は floor(売価 × 0.6) 円」に直した |

是正で足した契約の文（Y1 の順、T2 の直し方と正の対照、`retired_stock_unit_hint`、価格改定の初期値、B8・B10 の対照、L3-1 の (0)）を境界の 4 観点で一巡した: (1) 安全な整数の範囲 — 価格改定の初期値は 1/100 円の 100 の倍数なので `91` の形で、`parseCostInput` に戻すと同じ値。(2) 0 — 現売価 0・現原価 0 の行の初期値は `0`。L3-1 の (0) の件数 0 は入力物の作り直しに回す。(3) 失敗の時点と順序 — 入口ごとに現物の順（登録: 商品名 → 売価・原価 → 初期在庫 → 部門 → 取引先 → JAN、修正: 商品の取得 → 検証、価格改定: 商品の取得 → 売価・原価 → 取引先、CSV の preview: 行の error を列の順に積む、commit: TX → 既存の商品 → 再検証、入庫・廃棄: 業務日 → 冪等性 → 明細ごとに数量 → 原価 → 商品）を Y1 に書き、変える所は無い。(4) 単位や種類の混在 — 本番の分岐の検査は `"cm"`・`"roll"` を見るだけで、`巻` は日本語の文字列で AC5 の検査に当たらない（分岐の関数の中に置く）。

### Plan Review round 3（2026-10-21、head `7c1f308e`。天井）

Claude 側 fresh Opus 5.5 の全体（reject、P1 0・P2 1・P3 6、#1〜#7。Ordinary Operation は成立）、Claude 側 fresh Opus 5.5 の深掘り（既存 test の移し方と失敗の順。reject、P1 0・P2 1・P3 6、#1〜#7。段 1 の表で OK でない 17 行）、Codex 472（Astra、reject、P1 0・P2 1、#1。Ordinary Operation は成立）。裁定は Coordinator で全部採った。round 3 は天井で、owner の決定（repo 外の回答台帳 TD-367）により、指摘を直してその差分を closure（fresh Opus と Codex）で確かめ、P1/P2 が 0 なら plan-approved にする（新しい round は開かない）。closure の対象を小さくするため、新しい契約は足さず、既存の表と文の最小の変更で直した。Owner Effort の消費は 1（TD-367）。

| # | 出所 | 重大度 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|---|
| Z1 | 全体 #1 | P2 | 採る | 廃棄の合計の twin が `parseCostInput` の `"too_large"` を `BigInt` に渡して画面が落ちうる（typecheck は止めない） | S5 の廃棄の行に「number でない結果の行は 0 として数える」。C17 に `1000000` と `83.333` の行。`parseCostInput` の結果を受ける全 site を洗った（下） |
| Z2 | Codex #1 | P2 | 採る | 原価 `i64::MAX / 2 + 1` の overflow の test は ×100 できない | T2 に例外の行（原価を保ち数量を ×100）。×100 で範囲を越える fixture を全件洗った表を Matrix の T2b に（下） |
| Z3 | 深掘り #1 | P2 | 採る | 原価の field 名を含まないのに期待が変わる test が T5 に無い | T5 に「期待を変える: 原価の field 名を含まない…」の行（`price-revision-math.test.ts` の golden と exact の case の作り直し、`ProductForm.test.tsx` の label と説明、`request-helpers.test.ts` の `integerCallCount`）。T5・AC8 の数える command に 1 本足し、baseline 28 行。C15 に `8349/100/120 → 10000` |
| Z4 | 全体 #2 | P3 | 採る | code の comment の旧い原価の記述 | S7 に code の comment の command と直す所（10 行のうち 5 行を直し、ほかは S5・T5・履歴・誤検出） |
| Z5 | 全体 #3 | P3 | 採る | T1 の (b) が版の数を直接書く | (b) を `migrate_up_to(&conn, VB_VERSION - 1)` にし、期待はその版（下げない） |
| Z6 | 全体 #4 | P3 | 採る | ロスの無い行の表示が決まっていない | (7) と T5 に「ロスの無い行も `0.00 円`」 |
| Z7 | 全体 #5 | P3 | 採る | L3-1 の (0) が DB の版を見ない | (0) に版の記録の表 `schema_versions` の最大の `version` が vB の直前の版 |
| Z8 | 全体 #6 | P3 | 採る | L3-2 の売価の記号が環境に依る | 合格基準を「売価の表示は lane の前と変わらない」 |
| Z9 | 全体 #7 | P3 | 採る | D-128 の Guarantee range が現掛率を税抜の値に含める | 現掛率を「原価（税抜）÷ 売価（税込）の比で、税抜の値ではない」と書いた（決定の中身は変えない） |
| Z10 | 深掘り #2 | P3 | 採る | TS の上限の test と商品 CSV の文字列の受ける端が端でなくなる | T2 の境界の行に CSV の文字列（円のまま、受ける端 `999999.99`）、T5 に TS の入力の上限の端の行 |
| Z11 | 深掘り #3 | P3 | 採る | T3 の対の case が単位の拒否を確かめない | 更新の対を `test_update_product_req102_rejects_cross_kind_stock_unit` に |
| Z12 | 深掘り #4 | P3 | 採る | 在庫連動の提案を `cm` で確かめる商品フォームの test が T3 に無い | T3 の「`cm` の fixture を `m` に」の TS の一覧に 10 本を名指し、`m` に替えると決めた |
| Z13 | 深掘り #5 | P3 | 採る | T2a の `tsconfig.json` の前提が現物と違う | 前提を直し、型で止まらない 2 file を実行時に red になる側と書いた |
| Z14 | 深掘り #6 | P3 | 採る | commit の再検証は商品の無い上書きの行で観測できない | Y1 と C3 に「新規の行か既存の上書きの行で確かめる」 |
| Z15 | 深掘り #7 | P3 | 採る | commit の再検証の外した単位の文が決まっていない | S6 に「preview と commit の両方が `retired_stock_unit_hint` の文を使う」。T3 で `("RV-M", "cm", …)` を外さず期待を変える、U4 に commit の文 |

洗い出しの要約（全件）:

- Z1（`parseCostInput` の結果を受ける site）: `disposal-request.ts` の `calculateLossTotal`（`BigInt(costPrice)`。型で止まらないので S5 で 0 として数えると決めた）、`disposal-request.ts`・`receiving-request.ts` の request を組む所（`> MAX`・`cost_price: costPrice ?? 0` は `"too_large"` で型の error になり、`原価が大きすぎます` の分岐を書くことになる）、`product-form-request.ts` の `readPrice`（今の `parsePriceInput` が同じ形の union を返し、`"too_large"` を扱い済み）、`PriceRevisionTable.tsx` の新原価の欄（`> MAX` と request の field が型の error になる）、新しい `unitCostFromPack`（Y5 の入力の error）。入庫の画面に合計の twin は無い（`rg -n 'BigInt|calculate|Total' src/features/receiving --glob '!*.test.*'` が 0 行）。number として扱って黙って通る site は `calculateLossTotal` だけだった。
- Z2: Matrix の T2b（×100 で `i64` を越える 2 test、円の原価の helper、×100 しても上限の中の fixture、原価でない大きな値）。
- Z3: T5 の 2 つ目の command の 28 行・3 file を全部分類した。

是正で足した文（S5 の 0 として数える、T2 の例外と CSV の文字列、T2b、T5 の 2 行、T1 (b) の版、L3-1 の版、Y1・C3 の commit の行、S6・U4 の commit の文、ロスの無い行、D-128 の現掛率）を境界の 4 観点で一巡した。
- (1) 安全な整数の範囲: exact の case を上限の積 `999999 × 99999999` に作り直した（安全な整数の範囲の中）。overflow の 2 test は数量を ×100 しても円の合計が `i64` を越える（Codex の probe と同じ）。
- (2) 0: ロスの無い行は `0.00 円`。`"too_large"`・null の行は合計で 0。
- (3) 失敗の時点と順序: commit の外した単位は、今の `import_row_commit_error` の単位の code の検査の位置で文だけが変わる。commit の上限の検査は、新規の行と既存の上書きの行で観測する。
- (4) 単位や種類の混在: 商品フォームの在庫連動の提案は `m` で同じ契約。

Closeout（2026-10-21）: Final Review は broad 3 本で、Codex 482（Astra）= approve（P3 1）、Claude 側 Fable 5.1 = approve（P3 3）、fresh Opus の深掘り = approve（P3 2）。P3 の disposition（Coordinator の裁定）: E1〜E4 は是正の commit で直した。E5（原価の形式の error の文の literal が `src/lib/unit-quantity.ts` の `COST_FORMAT_ERROR` と 4 file にある）は follow-up で、`docs/backlog.md` へ 1 項目として申し送った。是正の後に main（PR #183・#186）を取り込んで base を同期し、closure 2 本（Codex 486〈Sol〉= approve〈P0〉、Claude 側 Fable 5.1 = approve〈P0〉）。L3（2026-10-21、owner、Windows native、head `06f24747`）は L3-1〜L3-4 すべて PASS（repo 外の回答台帳 TD-370）。L3-1 の (0) は owner の開発用 DB が版 7 だったので、main の demo の seed で版 9・商品 106 件の合成 DB を作って入力物にした。vB は止まりの文 `…（22 §16 の作り直しの手順）: products` を出し、DB の hash は変わらなかった。22 §16 の作り直し（2 つの folder を丸ごと移す）の後、空の DB で開いた。L3-4 は見た目の短い確認（原価の形式の error・在庫照会の詳細・価格の履歴）。Contract Probe P5 は確かめた: 作り直しの前の 2 つの folder の一覧で、WebView2 の保存（`EBWebView`）は `%LOCALAPPDATA%\com.kosei.inventory` の下にあり、`%APPDATA%\com.kosei.inventory` の下には無い（Windows、`npm run tauri dev` の起動、2026-10-21）。22 §16 の「確かめていない前提」の段をこの事実に書き直した。商品 CSV の preview の行の error の文・原価の上限の error・廃棄の候補の一覧の原価の見た目は L3 で見ず自動 test に任せた（PR の本文の残りの危険）。helper の専用 record に broad・closure・manual を記録し、Ready の後の `Merge gate` は pass で、owner の Ready・merge の承認（TD-371）の後に merge した。packet の本文（Scope・Contract Ledger・AC・Contract Probe）は当時の契約の記録として書き換えない。Owner Effort Budget の表は closeout の時点の値（上限 6、消費 3）に直した。
