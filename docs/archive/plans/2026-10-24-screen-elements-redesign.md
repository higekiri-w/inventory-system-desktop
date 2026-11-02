# Plan Packet: 画面ごとの要素の見直し（商品・在庫を探す・見る・直す画面）

wave に属さない単独の lane（design lane、docs だけ。画面の code は変えず、設計が決まった後の実装は別の lane）。owner の所感（repo 外の回答台帳 TD-413・TD-415、2026-10-24、PR #191 の L3 の途中。原文は台帳にあり、公開 repo には書かない）を起点に、商品・在庫を探す・見る・直す画面が、それぞれどの要素（検索の道具・初めて開いたときに出るもの・一覧の列・行き先）を持つかを一旦並べ直して整える。進め方は棚卸しの数の置き場の design lane（D-118、`docs/archive/plans/2026-10-16-stocktake-count-placement-design.md`、repo 外の回答台帳 TD-258）の型に寄せる: 段 1 で今の画面の事実だけを集め、段 2 で白紙の骨組みを組み、段 3 で今の設計と突き合わせて owner が採否を決め、その結果を正本へ置き換える。決定の番号は D-138（Coordinator が割り振った。段 3 で owner が採否を決めた後に decision-log へ置く）。本 lane の branch は `agent/screen-elements-redesign`（起点 main `843c4938`）。

着手の順: 店の PC への導入の前の関門ではない。並走の製品の lane（棚卸しの b1・b2 ほか）が今の画面を触っているので、本 lane は docs だけで進め、画面の code・設計正本の文を段 1・段 2 では変えない。

本 run（起票）でやったこと: 起票と段 1（事実を集める）まで。段 2 以降は後の run。

段 2 の run（2026-10-24）でやったこと: 段 2 の白紙の骨組み（`## 段 2: 白紙の骨組み`）と、段 3 の入力の整理（同 2.8）。正本と画面の code は変えていない。段 3（owner の採否と正本の置換え）は後の run。

段 2 の直しの run（2026-10-24）でやったこと: 相談役の反例探しを Coordinator が受理した 7 点で段 2 を直した（QU-004 の出所の範囲、Q-1 の d、案 A の失うもの、大きさの見込み、K-5 の限定、原則の伸ばしの限定、案 C の位置づけ）。正本と画面の code は変えていない。

段 3 の run（2026-10-24）でやったこと: owner の採否（決定 1 = A 統合〈TD-424〉、決定 2 = a 全商品の一覧〈TD-425〉）と店主の答え（Q-1 = a〈Q-101、TD-422〉、Q-2 = c〈Q-102、TD-423〉）を受け、採った形を正本へ置き換え、D-138 を decision-log に、後続の runtime の項目を backlog に置き、Test Design Matrix を書いた（`## 段 3: 突き合わせと owner の採否、正本の置換え`）。画面の code は変えていない。

## Workflow State

- Phase: archive
- Risk: R3
- Plan Commit: e948ac1b1df12bc8583acd647a60929713dcc8cd
- Amendments: none
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（本 run の起草役）
- Plan Reviewer: fresh Opus 5.5 + Codex（model は発注時に決める）
- Final Reviewer: Fable 5（Claude 側、R3・design lane。座組表の当面の Fable 5〈TD-410〉）+ Codex（GPT-6.1 Sol 既定）。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Review Minimum: 1
- Human Gate: ready,merge
- Branch: agent/screen-elements-redesign

遷移の記録:

1. kickoff → spec-check（2026-10-24、起草役）: owner の所感（TD-413・TD-415）を受けた Coordinator の発注で、対象を「商品・在庫を探す・見る・直す画面の要素の見直し」とし、段 1〜段 3 と置換えまでを本 lane の範囲にした。Risk を R3 と記録した（下の Risk）。
2. spec-check → design（2026-10-24、起草役、同じ commit）: 触る正本は `docs/SCREEN_DESIGN.md`（§1 画面一覧、§3 の在庫照会・商品検索・一覧・入出庫履歴ほか、§4「在庫照会のREQ統合」）、`docs/function-design/` の 50・53・58・61〜66・67・73・77、`docs/design-system/02-component-catalog.md` の ⑥・⑨・⑮、D-047 と `docs/function-design/52-ui-shared-layout.md` の UI-12-D1。在庫照会の契約 I（58 §58.10）と商品一覧の UI-01a-D1（50 §50.2）が「初めて開いたときに何を出すか」を画面ごとに逆に決めており、owner の所感がその片方を問うので、設計の更新が要る（design へ進む）。設計の中身は段 1〜段 3 で作る。design → plan-draft は段 3 で owner が採否を決め、正本の置換えの文ができてから。
3. design → plan-draft（2026-10-24、起草役、段 3 の commit）: 条件「設計の出力が正本にある」は、D-138（decision-log）と [50](../../function-design/50-ui-product-list.md) UI-01a-D1・D14〜D19・§50.4・§50.6・§50.7、[58](../../function-design/58-ui-stock-inquiry.md) の冒頭の注記・契約 H・UI-06a-D8・廃番除外・§58.7、52 UI-12-D1、53 UI-00-D13、61 UI-02-D10・D18、62〜64 の保存の結果、66 UI-06c-D9、02 ⑥・⑨・⑮、`docs/SCREEN_DESIGN.md` §1〜§4 で満たす（段 3 の対応表）。条件「未解決の設計の問いが無い」は、決定 1・決定 2（TD-424・TD-425）と Q-1・Q-2（TD-422・TD-423）で決まり、段 3 の細部は design-system の原則から決めた（段 3 の 3.4）ことで満たす。Open Questions の Q-3（商品ごとの最低限の数）は答えが無いときの既定が D-138 の形（今の基準との比べ）で、設計を止めない。
4. plan-draft → plan-gate（2026-10-24、起草役、次の commit）: 条件「packet が完成して commit 済み、R3 の Test Design Matrix が commit 済み」は、段 3 の commit `e58f8aaf`（packet の段 3・Scope・Boundary / Wire Contract・Contract Ledger と `docs/archive/plans/test-matrices/2026-10-24-screen-elements-redesign.md`）で満たす。owner の判断を待つ事項は無い（Q-3 は既定つきで止めない）。Plan Review へ出すのは Coordinator。
5. plan-gate → plan-approved（2026-10-26、Coordinator、本 commit）: 条件は `docs/DEV_WORKFLOW.md` の遷移表の「Writer でない独立の Plan Reviewer が P1 / P2 = 0」。Plan Review round 3（天井）の後、Coordinator の disposition「一括是正」で `2f359519` を作り（同時に base を同期）、closure（Claude 側 fresh Opus 5.5 と Codex）の Codex 側の固定の不足（A〜D）を `edbd6046` で、Claude 側の closure 2 の P3 を `e948ac1b` で直した。closure 2 の 2 本（Claude 側 fresh Opus 5.5 が `edbd6046` に approve P3 2、Codex が P3 を直した `e948ac1b` に approve P1 / P2 / P3 0、未 closure 0）が P1 / P2 = 0 を報告した（下の Review Response）。Plan Commit は Codex の closure が監査した `e948ac1b`（前例: PR #203 の遷移 6）。
6. plan-approved → implementing（2026-10-26、Coordinator、本 commit。state だけ）: 遷移表の唯一の入口。本 lane は docs だけの design lane で、Final Review の record は packet が implementing の head でしか受けない（helper）ので、plan-approved と同じ commit で進める（前例: PR #168・#163）。直後の commit で `origin/main`（PR #198 の merge の後）を単段で取り込み、その head を Final Review broad に出す。

## Owner Effort Budget

- 介入回数上限: 6（既定）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 6 | 2（段 3 の時点。決定 1〈TD-424〉・決定 2〈TD-425〉。店主への問い Q-1・Q-2〈TD-422・TD-423〉は同じ問い合わせに束ね、数え方は Coordinator が決める。TD-413・TD-415 は起票の前の所感で数えない） | 2（Ready 1、merge 1） | 2 | 6 = 2 + 2 + 2 |

既定値・数え方・上限に届くときの扱いは `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
本 lane は docs だけだが、段 3 の結果で置き換える正本が、route / search state（在庫照会の `status`・`q`・`page`、商品一覧の search params、サイドバーの deep-link、`returnTo`）と operator workflow（どの画面で商品を探し、初めて開いたときに何が出るか）の契約を持ち、後続の runtime がそれを写す。Risk Tiers の R3 の行（route/search state、operator workflow）に当たり、「R2 と R3 で迷えば、UI の route/search の振舞いに触れるなら R3」にも当たる。先例の docs だけの design lane（D-118、`docs/archive/plans/2026-10-16-stocktake-count-placement-design.md`）も後続の runtime が写す契約を決めるので R3 だった。R4 ではない: データを壊す・実店舗のデータを出す・不可逆な操作をせず、docs の差分だけで `git revert` で戻る。DB・Tauri command の DTO・bindings は段 3 の結果しだいで後続の runtime が変える見込みがあり、その lane の起票で Risk を決め直す。Final Review Minimum 1（workflow gate の変更を含まない。classifier の workflow の一覧に当たる path は Scope に無い）。

## Goal

Goal Invariant:

### 最小完了条件

- 商品・在庫を探す・見る・直す画面のそれぞれについて、何のための画面か・検索の道具に何を置くか・初めて開いたときに何を出すか・一覧に何の列を出すか・行からどこへ行けるかが、source docs（`docs/SCREEN_DESIGN.md` と該当の function design、design-system）だけで答えられ、owner が段 3 で採った形になっている。
- 在庫照会を「すべて」で開いて何も出ない形（今の契約 I）を残すか変えるかが、owner の採否として正本に書かれている（変えないなら、その理由と、使い手が何も見えないまま迷わない手当て）。
- 重なる画面の組（段 1 の重なりの一覧）のそれぞれに、統合する・役割を分ける・今のまま、のどれにしたかと理由が書かれている。
- 後続の runtime の lane の Writer が、チャットの履歴と repo 外の資料を見ずに、正本だけで画面の変更を実装できる。

### 失敗定義

- 段 1 の事実を確かめずに（code の現物か正本の片方だけで）骨組みや選択肢を作る。
- owner の採否の前に、画面の code・設計正本の文を書き換える。並走の lane（棚卸しの b1・b2 ほか）が触る画面の code に手を入れる。
- 使い手（店主）に、今より多くの手作業や「どの画面か迷う」場面を残す形を、理由を書かずに採る（AGENTS.md の「使い手に押しつけない」）。
- owner の所感の原文を公開 repo に書く、または Coordinator・subagent の要約を owner の意図として裁定する。
- D-118 の (b)（棚卸し・在庫照会・在庫変動の要再確認の表示）、D-117 の PLU の書出しの見直しの runtime の決定を、本 lane で書き換える。

### 非目的

- 画面の code・test・bindings・migration の変更（後続の runtime の lane）。
- 見た目の刷新（書体・色・余白。デザインの刷新は v1.0 の前に入れる別の作業〈owner の決定 TD-296〉）。
- 売上の画面（日次・月次）、取込み、システム管理の画面の要素の見直し（下の段 1 の「外した画面」）。
- D-118 の (b) と D-117 の runtime の中身（本 lane は、それらが決めた画面の変更と矛盾しない形を選ぶだけ）。
- `docs/Plans.md` の lane ごとの行の追加（D-097。`## 次の行動` の `docs/plans/` への pointer の行が既にある）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

本 lane は operator の操作を決め直す design lane なので、店主の普通の一日のうち本 lane が扱う操作を、今の結果と並べて置く（段 3 で「採った形での結果」の列を足す）。この文書を完了できる（段 3 の採否と正本の置換え）ことと、通常運用を達成できる（後続の runtime の lane が画面を変える）ことは別で、本 lane だけでは後者は達しない。

| 初期状態 | 操作 | 利用者が得る結果（今） | 採った形での結果（D-138、後続の runtime の後） | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- | --- |
| 開店の前後、発注を考える | サイドバーかホームの「在庫照会」を開く | 一覧は出ず、検索の案内だけが出る（S-03、E-01） | 「商品と在庫」を開くと全商品の一覧が出る。発注の準備はサイドバー・ホームの「在庫切れ・在庫少」で、在庫切れと在庫少が取引先順に 1 つの一覧で出る（メーカー品番つき） | 後続の runtime の L3 で、一覧の列と道具が読めるかを owner が確かめる | 店主は PC で在庫を見るとき全部の商品の一覧が助かると答えた（Q-101・TD-422） |
| 品物が届いた | 入庫記録で商品を入れて保存し、「在庫照会へ戻る」を押す | 在庫照会が案内だけで開き、入れた商品の在庫は検索し直さないと見えない（S-07、E-02） | 保存の結果に保存した商品ごとの「在庫を見る」が出て、押すとその商品の行と詳細が開き、状態の欄が在庫少の基準との比べ（在庫少 / 在庫あり）を示す | 同上 | 店主は時々、限られた商品について最低限要る数に達したかを確かめる（Q-102・TD-423）。商品ごとに違う数は Q-3 |
| 商品を直したい | 商品検索・一覧を開いて探し、「修正」を押す | 一覧が初めから出ていて探せる。修正の後は一覧へ戻る（S-02、S-14） | 「商品と在庫」で探し、行を開いて「商品修正」（2 click）。保存の後は来た一覧の条件へ戻る | 同上 | なし |
| 在庫照会で商品を開き、商品修正へ進む | 詳細の「商品修正」を押して保存する | 保存の後は商品一覧へ戻り、在庫照会へは戻らない（S-03、S-14） | 同じ画面なので、保存の後は来た一覧（在庫切れ・在庫少を含む）へ戻る | 同上 | なし |

## Scope

段 1（本 run で済み）:

- 商品・在庫を探す・見る・直す画面の事実の表（下の「段 1: 今の画面の事実」）。画面ごとに、目的・検索の道具・初めて開いたときに出るもの・一覧の列・行から行ける先・重なる画面を、code の現物（route・component・hook・BIZ / IO の検索の関数）と正本の両方で確かめ、`file:line` か節の ID で引く。
- 正本と code の食い違いの表。
- 重なる画面の組と、初めて開いたときに何も出ない所の一覧（解決案は書かない）。

段 2・段 3（下の「段 2・段 3 の進め方」）:

- 段 2: 白紙の骨組み（本 packet の後半の `## 段 2: 白紙の骨組み`。2026-10-24 の段 2 の run で済み）。
- 段 3: 今の設計との突き合わせと owner の採否の選択肢（本 packet の後半に置く）。採否の後、正本を置き換える: `docs/SCREEN_DESIGN.md`（§1・§2・§3 の対象の画面・§4「在庫照会のREQ統合」）、`docs/function-design/` の 50・53・58・61〜66・67・73・77 のうち採った形が触る節、`docs/design-system/02-component-catalog.md` の ⑥・⑨・⑮ のうち触る所、`docs/decision-log.md` の D-138、D-047 の置換えの行（変えるなら）、`docs/backlog.md` の後続の runtime の項目。段 3 の後に Scope をこの粒度から file の一覧へ詰める。

段 3 で置き換えた file（`git diff --name-only origin/main...HEAD` の出力から本 packet と Matrix を除いたもの）:

- 画面の正本: `docs/SCREEN_DESIGN.md`（§1 #4・#7、§2 のエリア・動線・遷移、§3 の商品と在庫画面〈在庫照会画面と商品検索・一覧画面を置換え〉・入出庫履歴・在庫変動追跡・入庫記録・返品・交換・PLU書出し、§4 の商品と在庫の統合・UI-06a Phase 2 の注記）、`docs/UI_TECH_STACK.md`（query の表の UI-06b の行、HID の UI-06a の行）
- function design: `docs/function-design/50-ui-product-list.md`（冒頭、§50.1、§50.2 UI-01a-D1・D14〜D19、§50.4、§50.6、§50.7、§50.8）、`58-ui-stock-inquiry.md`（冒頭、§58.1、§58.4、§58.6、§58.7、§58.8、§58.10 契約 H・契約 I・UI-06a-D7・D8・廃番除外、§58.12、§58.13、更新履歴）、`52-ui-shared-layout.md`（§52.3 の UI-06a・UI-01a・UI-06b の行とナビに出さない route、§52.6 UI-12-D1）、`53-ui-home.md`（§53.1 の QuickActionGrid の行、§53.8 UI-00-D13）、`51-ui-product-form.md`（UI-01b-D2）、`60-ui-product-import.md`（UI-01c-D11・D13、§60.5 の result）、`61-ui-receiving.md`（UI-02-D10・D13・D18、§61.3、§61.5、§61.7、§61.9）、`62-ui-manual-sale.md`（UI-04-D12・D14、§62.3、§62.5 の候補の列）、`63-ui-return-exchange.md`（UI-03-D15・D18、§63.3、§63.7、§63.5 の候補の列）、`64-ui-disposal.md`（UI-05-D11・D14、§64.3、§64.7、§64.5 の候補の列）、`66-ui-stock-movements.md`（§66.1、UI-06c-D9、§66.3）、`67-ui-plu-export.md`（Diff 0 件の文言、§67.7 の `confirmed` の操作の label）、`69-ui-threshold-settings.md`（§69.2・§69.9 の区画の説明・§69.12 L3-1 の画面名）、`77-ui-bulk-price-revision.md`（Empty の導線の label）
- 索引・要約: `docs/ARCHITECTURE.md`（UI-01a・UI-06a・UI-06b の行）、`docs/FUNCTION_DESIGN.md`（UI-06a の要約の 2 行の「検索駆動表示契約 I」に撤去の注記。索引の link の文字は変えない）、`docs/architecture/ui-task-specs.md`（UI-12 のサイドバー、UI-00 の大ボタン、UI-01a、UI-06a・UI-06b）
- design-system: `docs/design-system/01-decision-rules.md`（DSR-18 の木の図の例）、`docs/design-system/02-component-catalog.md`（⑥ 空状態の説明・filter-empty の採用箇所、⑧ 廃番の確認の説明の画面名、⑨ 商品を探す検索欄の列、⑮ 取引 4 画面の先頭の列、更新履歴）
- 決定と後続: `docs/decision-log.md`（D-138、D-047 の Superseded in part の行）、`docs/backlog.md`（「商品と在庫の画面の統合の runtime」、「商品ごとの最低限の在庫数」）
- Matrix: `docs/archive/plans/test-matrices/2026-10-24-screen-elements-redesign.md`
- Plan Review round 1 の是正で触った節（上の file の中。`## Review Response` の round 1）: 50 UI-01a-D3・D8・D15・D17・D19・§50.4・§50.5・§50.7・§50.8、51 UI-01b-D2、58 の冒頭の注記・テンプレ判定・§58.1・§58.4 の `selected` ライフサイクル・§58.7・契約 H・UI-06a-D4・D7・§58.12・更新履歴、52 §52.3 の UI-06c の行・§52.4 の型の例、53 §53.8、61 UI-02-D10・§61.9、62〜64 の result の行、66 の入力ドキュメント・§66.1・UI-06c-D9・§66.3、ui-task-specs の UI-06b、backlog の runtime の項目

変えないもの: `docs/spec/requirements.md`（REQ と task の対応）、各設計書の `> 対応仕様:` の行、`docs/FUNCTION_DESIGN.md` の索引の link の文字（traceability の生成の入力は link の文字だけ〈`src-tauri/src/bin/generate_traceability.rs` の `parse_index_links`〉。UI-01a と UI-06a の ID を 1 つの画面で持つので変えない）、`docs/diagrams/current-system.md`（今の build の図。runtime の lane が直す）。`docs/Plans.md` は変えない（`## 次の行動` に `docs/plans/` を指す pointer の行が既にあり、D-097 の D6 は lane ごとの行を足さない）。

## Non-scope

- 画面の code・test・bindings・migration（後続の runtime の lane）。
- 売上の画面・取込み・システム管理の画面（段 1 の「外した画面」の理由）。
- D-118 の (b) と D-117 の runtime の中身。
- 見た目の token（色・書体・余白）の変更。
- 並走の lane（棚卸しの b1・b2 ほか）の packet と code。

## Acceptance Criteria

- AC1（段 1）: 段 1 の表が、`docs/SCREEN_DESIGN.md` §1 の画面一覧と `src/routes/` の route のうち商品・在庫を探す・見る・直す画面を全部持ち、外した画面に理由がある。各行の根拠が `file:line`（起点 main `843c4938` の時点）か節の ID で引ける。
- AC2（段 1）: 正本と code の食い違いが `### 正本と code の食い違い` の表にあり、各行に正本の節と code の `file:line` の両方がある。
- AC3（段 1）: 重なる画面の組（`### 重なる画面の組` の O-n）と、初めて開いたときに何も出ない所（`### 初めて開いたときに何も出ない所` の E-n）の一覧があり、解決案を含まない。
- AC4（段 2）: 白紙の骨組みが、店の仕事の流れ（`docs/project-memory.md`・`docs/spec/requirements.md`）と design-system の正本が引く文献だけから組まれ、今の画面の形を前提にしていない。各要素に根拠（文献の原則か店の事実の ID）がある。
- AC5（段 3）: 段 1 の重なりの組と何も出ない所の全部に、owner が選べる選択肢（今のままを含む）と、各案の利点・失うもの・後続の runtime の大きさの見込み（`未実測` を含む）がある。owner の採否が repo 外の回答台帳の ID で引ける。
- AC6（置換え）: 採った形が正本に書かれ、旧い契約（契約 I ほか）の置換えの行がある。`bash scripts/doc-consistency-check.sh` と `--target plan docs/archive/plans/2026-10-24-screen-elements-redesign.md` が ERROR 0。
- AC7: 差分が docs だけ（`git diff --name-only origin/main...HEAD` が `docs/` の下だけ）。公開 repo に店のデータ・owner の原文が無い。

## Design Readiness

設計の完了条件は `docs/DEV_WORKFLOW.md` Design Phase Rules の Design completion criteria。

- 引用する設計正本（節まで）: `docs/SCREEN_DESIGN.md` §1・§2・§3（ホーム・PLU書出し・棚卸し・在庫照会・入出庫履歴・在庫変動追跡・入庫記録・返品・交換・商品検索・一覧）・§4「在庫照会のREQ統合」、50 §50.2〜§50.7、53 §53.1・§53.4、58 §58.1・§58.4・§58.7・§58.10・§58.13、61 §61.5、62・63・64 の product add の行、65 §65.4・§65.8.1、66 §66.2・§66.3、67 §67.5、73 §73.3〜§73.6・§73.15、77 §77.2〜§77.6、52 UI-12-D1、02 の ⑥・⑨・⑮、D-047・D-089。
- 必要な設計成果物: operator workflow / route・search state / empty UI の行 = `docs/SCREEN_DESIGN.md`・`docs/UI_TECH_STACK.md`（触るなら）・対象の UI function design（状態 = 段 3 の後に updated in this PR）。durable な判断の行 = decision-log の D-138（状態 = 段 3 の後に updated in this PR）。BIZ・CMD・DB の行は段 3 の採った形しだい（画面の統合で検索の command を足すなら function design の 30・40 と Boundary / Wire Contract）。
- plan にしかない durable な判断の昇格先: 段 3 の採否は D-138 と各正本の節。段 1 の表と段 2 の骨組みは packet に置き、正本には写さない（archive の packet が経緯を持つ）。
- 前提・制約と、延期した design gap の follow-up: SP-301-01〜03・SP-302-01〜04（在庫照会の仕様の ID）の本文は tracked docs に無い（`rg -n 'SP-301-01|SP-302-01' docs --glob '!docs/archive/**'` の出力 2 行、どちらも ID の引用だけ〈`docs/SCREEN_DESIGN.md` と 58 §58.1〉）。段 2 で REQ の文（`docs/spec/requirements.md` の REQ-103・REQ-301〜303）と店の事実から組み、SP の本文を前提にしない。
- 絶対保証の例外と escape hatch の自己点検: 段 1 では絶対保証を書いていない。段 3 で「何も出ない画面は無い」と書くなら、読込みの失敗・0 件・範囲外の page・商品が 1 件も無い DB の場合を同じ PR で点検する。
- 段 3 の後の状態: operator workflow / route・search state / empty UI の行 = updated in this PR（上の Scope の file）。durable な判断 = D-138（updated in this PR）。BIZ・CMD・DB = 変えない（`search_products`・`list_low_stock`・`get_stock_detail` の既存の型で足りる。Boundary / Wire Contract）。
- 絶対保証の点検（段 3）: 「何も出ない画面は作らない」（50 UI-01a-D18、D-138 の Guarantee range）の例外を同じ PR で点検した: 読込みの失敗（Alert と再試行）、絞り込みの 0 件（条件に合わせた説明と「絞り込みを解除」）、範囲外の page（UI-06a-D3 の回復）、商品が 1 件も無い DB（「商品がありません」と「商品を登録する」）、在庫少の集合だけの失敗（58 契約 H の「—」と知らせ）・読込み中（Skeleton）。
- 隣の仕様との比べ（Design completion criteria の route/search state の行）: 戻り先（商品修正の `returnTo` は作る側 `buildProductListReturnTo` と受け口 `parseProductListSearchFromReturnTo` の両方に `status`・`selected` を足し、作る側は手組みのまま〈51 UI-01b-D2〉。在庫変動履歴は 66 UI-06c-D9 の仕組みのまま pin を `/products` へ、fallback に `discontinued=all`。2 つの導線で作る側の規約が違う〈58 §58.7〉）、絞り込みの解除（02 ⑥ の採用箇所の統合の後の数）、行の選択（58 §58.4 の `selected` ライフサイクル。検索前ガードを撤去し、部分一致で 1 ページ目に無い商品の `selected` は外れる限定）、数字だけの商品コードの往復（66 の GA5 を `/products` にも当てる。商品修正の `returnTo` は router を通らない typed parse-back）。
- 判定: ready（design → plan-draft の条件を満たす。Workflow State の遷移の記録 3）。

## Registration / Generation Obligations

段 3 の後も本 lane は該当なし: function-design の file を新設・改名せず、`> 対応仕様:` の行・`docs/spec/requirements.md`・`docs/FUNCTION_DESIGN.md` の索引の link の文字を変えないので、`build_doc_to_modules_map()` と traceability の再生成は要らない。（起票の時の注記: function-design の file を新設・改名するなら `src-tauri/tests/design_compliance_test.rs` の `build_doc_to_modules_map()` と親の索引、REQ の参照を増減するなら `cargo run --bin generate_traceability` を Scope に入れる。route の新設・改名・operator 画面の新設・統合は後続の runtime の lane の義務（`npm run generate:routes`、`src/config/navigation.ts` と到達の test）。D-138 の runtime では `/stock` の index の redirect と navigation の 2 項目の変更・1 項目の削除がこれに当たる（backlog の該当項目）。）

## Impact Review Lenses

起点が owner の L3 の途中の所感（operator workflow の気付き）なので当てる。

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | Which concepts belong to replaceable external adapters, and which concepts are stable app-core contracts? | Architecture, function design, decision-log, Plan Packet | 不該当: 外部の system を変えない。検索は既存の BIZ `search_products`・`list_low_stock` の上の画面の話 | なし |
| Fact check / design decision split | Which claims are observed facts from hardware/tool/files, and which are app decisions that need source-doc promotion? | Investigation doc, source design docs, decision-log | 該当: 段 1 は code と正本の事実だけ。段 3 で根拠にした library の観測（router の search の直列化）は下の Contract Probe。契約 I・UI-01a-D1 は app の決定で、段 3 で採否を決め正本へ昇格する | 段 1 の表、段 3、D-138 |
| Lifecycle / retry | What happens before, during, after, and after failure for import/export, duplicate input, rollback, retry, cancellation, and re-run? | Function design, DB design, UI design, Test Matrix | 一部該当: 読込みの失敗・0 件・範囲外の page の表示（58 §58.8、UI-06a-D3）を、初めて開いたときに一覧を出す形にするなら同じ扱いで点検する | 段 3 の選択肢 |
| Operator workflow | What does the operator do in the real sequence across app, external tool, media, print/export, backup, and recovery? | Screen/UI design, function design, Plan Packet manual checks | 該当: 発注を考える・入庫の後に確かめる・商品を直す、の列（上の Ordinary Operation）。店の事実は段 2 で台帳を引く | 段 2 の骨組み |
| Replacement path | If the external system changes, which files/modules/docs are replaced and which app-core contracts remain stable? | Architecture, function design, decision-log | 不該当: 外部の system を置き換えない | なし |
| Data safety / evidence | How can the claim be supported by anonymized shape/count/hash/procedure evidence without committing real store data? | Plan Packet Data Safety, investigation doc, review evidence | 該当: owner の原文は台帳の ID だけで引き、店の商品数・商品名は書かない | Data Safety |
| Reporting / accounting semantics | Are totals, summaries, item records, returns, corrections, and inventory movements modeled separately enough to avoid false business meaning? | DB design, function design, report design, Test Matrix | 該当（段 3 で点検）: 一覧を全商品で開くと、今の契約 H では在庫少の商品を「在庫あり」と出す（基準以上を保証しない）。D-138 で「すべて」でも在庫少の集合で決め、「在庫あり」を基準を上回る意味にした。集合が読めないときは言い切らない | 58 契約 H、Matrix の契約 H の行 |
| Manual verification | Which assertions cannot be proven by automated tests and require Windows native L3, external tool import, or real-device confirmation? | Plan Packet, Test Matrix, PR body | 本 lane は docs だけで L3 は無い。採った形の見え方は後続の runtime の L3 で確かめる（design-system 04 原則 11） | 後続の runtime の packet |
| 環境・再現性 | 新設の環境依存（toolchain / CI runner / OS 差異等）を repo-pinned config で強制するか、明示的に defer するか | repo-pinned config, Plan Packet | 不該当: 環境依存を足さない | なし |

## Boundary / Wire Contract

command・DTO・bindings は変えない（`search_products`・`list_low_stock(includeDiscontinued)`・`get_stock_detail` の既存の型）。変わるのは route の search params（URL state）で、後続の runtime の lane が実装する。

- producer: 統合の画面（`/products` の navigate）、行の詳細（「商品修正」の `returnTo`、「在庫変動履歴」の `returnTo`、「入庫記録」の `productCode`）、入出庫 4 画面の保存の結果（「在庫を見る」の `q`・`selected`）、在庫変動履歴の「商品と在庫へ戻る」、`/stock` の redirect、サイドバーの「在庫切れ・在庫少」（`status=low_stock`）
- consumer: `/products` の route schema（`src/routes/products/index.tsx`・`src/features/products/search.ts` の `validateSearch`）、入庫の route schema（`src/routes/inventory/receiving.tsx`）、商品修正の `returnTo` の作る側 `buildProductListReturnTo` と受け口 `parseProductListSearchFromReturnTo`（`src/features/products/lib/return-to.ts:30-65`。許す範囲〈`/products` とその search〉は変えず、両方に `status`・`selected` を足す。51 UI-01b-D2）、在庫変動履歴の `returnToLinkProps` の pin（66 UI-06c-D9）
- wire type: URL の search params（string。router の `parseSearch` を通る。商品修正の `returnTo` の中身だけは `URLSearchParams` の手組みで、受け口が型つきで読み直し、router の `parseSearch` を通らない。作る側に router の href を使うと数字だけの値が引用符つきで直列化され〈`@tanstack/router-core` 1.171.3（lockfile の版）の `src/searchParams.ts:58-80` の `stringifySearchWith`。下の Contract Probe〉、受け口が引用符ごと受ける）
- internal type: `status: "all" | "low_stock"`、`selected: string`（1〜100 文字）、`productCode: string`（1〜100 文字）、今の `q`・`dept`・`discontinued`・`sort`・`dir`・`perPage`・`page`（50 §50.4）。`StockStatus` に在庫少の集合が読めない状態を足す（58 契約 H）
- precision/range: 文字列の長さ 100（`selected` の上限は 58 §58.4・66 UI-06c-D9 と同じ）
- round-trip path: 統合の画面 → 商品修正 → 保存・戻る → 来た一覧とその行（例外: `recount_required` の回復で棚卸しを経た後は `/products`、b1 の決定。51 UI-01b-D2）。統合の画面 → 在庫変動履歴 →（業務記録詳細 → 戻る）→「商品と在庫へ戻る」→ 来た一覧（限定は受け口ごと、66 §66.3・UI-06c-D9: 在庫変動履歴の `returnTo` が 500 字を超えると既定の戻り先で同じ商品に着く。業務記録詳細の `returnTo`〈入れ子の `detailReturnTo`〉が 500 字を超えると詳細の「前の画面へ戻る」は詳細の既定 `/inventory/records` に着き、在庫変動履歴と商品は失う。どちらも今の挙動のまま）。入出庫の保存の結果 →「在庫を見る」→ その商品の行。数字だけの商品コードも string のまま往復する（66 の GA5）
- invalid input: zod の `.catch(undefined)` で既定へ（今の契約）。`status` の旧値 `stockout` は `low_stock` として読む。`productCode` の不明・廃番は明細を入れず案内（61 UI-02-D18）
- compatibility: `/stock` は `/products` へ寄せ、`q`・`dept`・`selected`・`page` を写し（`page` を写すときは `perPage=50` を足す）、`status` を読み替える（50 UI-01a-D19）。限定: 統合の前の在庫照会で選んだ表示件数 100・200 は URL に無いローカルの値で再現できず、その `page` は別の行を指す（範囲外 page の回復か `selected` が外れる）。統合の前に作られた `returnTo=/stock?...` は在庫変動履歴の pin に合わず fallback（`/products?q=<code>&selected=<code>&discontinued=all`）で同じ商品に着く（66 UI-06c-D9。部分一致で 1 ページ目に無ければ `selected` は外れる、58 §58.4）。`/stock/$code/movements` の path は変えない

## Test Plan

本 lane は docs だけで、自動 test の追加は無い。Test Design Matrix は `docs/archive/plans/test-matrices/2026-10-24-screen-elements-redesign.md`（段 3 で置いた）で、採った形の契約ごとに後続の runtime が足す test の行（節の ID つき）を持つ。

- targeted tests: 本 lane は無い。`bash scripts/doc-consistency-check.sh` と `--target plan docs/archive/plans/2026-10-24-screen-elements-redesign.md` を ERROR 0 で通す
- negative tests: 本 lane は無い。0 件・読込みの失敗・範囲外の page・在庫少の集合の失敗は Matrix の UI-01a-D18・契約 H の行
- compatibility checks: URL の search params の互換（`/stock` の寄せ・`returnTo` の着地・数字だけの商品コード）は Matrix の UI-01a-D19・UI-06c-D9・§50.4 の行
- data safety checks: 差分に owner の原文・店のデータが無いことを目で確かめる
- main wiring/integration checks: 本 lane は無い（後続の runtime の到達の test）

## Review Focus

- 段 1 の表が、code の現物と正本の両方に合っているか（行の根拠の `file:line` と節を開いて確かめる）。外した画面の理由が妥当か。
- 重なりと何も出ない所の一覧に漏れが無いか。解決案が混ざっていないか。
- 食い違いの表に、どちらが正しいかの裁定を書いていないか（裁定は段 3）。
- この変更で不要になる記述・条件は無いか（段 3 の置換えで、契約 I・UI-01a-D1 の「機械的に横展開しない」・D-047 の文が要らなくなる見込みがあるか）。段 3 の後: 旧い言い方の sweep（段 3 の 3.6）の分類が妥当か、「今の build」と書いて残した記述が runtime の lane の Scope（backlog）に入っているか。
- 段 3（Plan Review）: 対応表（3.2）の全行が採った結果と直した正本の箇所を持ち、正本の文と食い違わないか。D-138 の失うもの・Guarantee range が正本（50・58）の文と合うか。「すべて」の在庫少の判定（58 契約 H）が、廃番の表示・集合の読込み中・失敗の全部の組で言い切らないか。Q-2 の「最低限の数」の扱い（3.3）が、新しい設定・field を足さずに言える範囲に留まっているか。

## 段 1: 今の画面の事実

対象の決め方: `docs/SCREEN_DESIGN.md` §1 の画面一覧（#1〜#21、#12a・#12b）と `src/routes/` の route（`find src/routes -name '*.tsx' -not -name '*.test.tsx' -not -name '-*' | wc -l` の出力 36。`__root` と子 route の layout を含む）と `src/config/navigation.ts` の 22 項目（`grep -c 'status: "active"' src/config/navigation.ts` の出力 22）から、商品か在庫を探す・見る・直す画面を挙げた。行番号は起点 main `843c4938` の時点。

### 対象の画面の表（14 画面）

| # | 画面（route） | 目的（正本） | 検索の道具 | 初めて開いたときに出るもの | 一覧の列 | 行から行ける先 | 重なる画面 |
|---|---|---|---|---|---|---|---|
| S-01 | ホーム（`/`、UI-00） | 朝一の状況把握と各画面への入口（SCREEN_DESIGN §3 ホーム画面、53 §53.1） | なし。件数 3 枚（昨日の売上・在庫切れ・在庫少、`src/features/home/components/SummaryCards.tsx:27`・`:54`・`:74`）。在庫切れ・在庫少は `list_low_stock(false)` を取り（`src/features/home/hooks/useHomeSummary.ts:34`）、在庫 0 以下と在庫ありに分けて数える（`src/features/home/lib/count-stock-status.ts:14-18`）。PLU 未反映は通知バー（`src/features/home/HomePage.tsx:72`） | 件数・通知・入口 card | なし | 件数 card は押しても遷移しない（D-089）。入口 card「在庫照会」は `/stock` を引数なしで開く（`src/features/home/components/QuickActionGrid.tsx:13` → `src/config/navigation.ts:89-97`）→ S-03 の案内。「商品管理」は `/products`（`QuickActionGrid.tsx:14`） | O-2、O-5 |
| S-02 | 商品検索・一覧（`/products`、UI-01a、REQ-103） | 商品管理の入口。既存商品の把握と修正の対象を探す（50 §50.1、UI-01a-D1。SCREEN_DESIGN §3 商品検索・一覧画面） | 検索欄（live、商品コード・商品名・JAN・メーカー品番。`src/features/products/ProductListPage.tsx:111-118`、BIZ の keyword は `maker_code` を含む `src-tauri/src/db/product_repo.rs:853`）、部門（部門の master 全件、UI-01a-D7）、廃番の表示（表示中・すべて・廃番のみ）、PLU の表示（すべて・対象・未反映・反映済み・対象外、`src/features/products/search.ts:20-26`）、並べ替え（商品コード・商品名・在庫数・売価 × 昇順・降順、`search.ts:27-36`）、表示件数 50・100・200（既定 100、`search.ts:129-134`）、ページと件数（`ProductListPage.tsx:281-300` の `ListShell`）。全部 URL に持つ（`search.ts:54-69`） | 廃番以外の商品の一覧（商品コードの昇順、100 件。UI-01a-D1、`search.ts:140-153`） | 商品コード・商品名（廃番の badge と JAN を同じ欄）・部門・売価・原価・在庫数・PLU・操作（`src/features/products/components/ProductTable.tsx:40-49`） | 行の「修正」→ `/products/$code/edit`（一覧の条件を `returnTo` で渡す、`ProductTable.tsx:103-111`）。「商品登録」→ `/products/new`（`ProductListPage.tsx:273`）。絞り込みに一致する全件の「PLU 対象にする / 外す」（`ProductListPage.tsx:216-248`） | O-1、O-4、O-5、O-6 |
| S-03 | 在庫照会（`/stock`、UI-06a、REQ-301・REQ-302） | 商品ごとの在庫数と状態を確かめ、発注の判断から入出庫へ進む（58 §58.1 の主動線「検索欄 → 結果一覧 → 詳細カード展開 → 発注判断」、SCREEN_DESIGN §3 在庫照会画面・§4「在庫照会のREQ統合」） | 検索欄（live、案内の文は商品コード・商品名・JAN。`src/features/stock-inquiry/StockInquiryPage.tsx:107-118`）、部門（master 全件、UI-06a-D2）、表示件数 50・100・200（URL に無いローカルの値、既定 50、`StockInquiryPage.tsx:45`）、状態のチップ すべて・在庫切れ・在庫少（`src/features/stock-inquiry/types.ts:13-17`）。ページは「すべて」だけ（`StockInquiryPage.tsx:251-260`）。件数は「すべて」がページの要約、在庫切れ・在庫少が「全 N 件」（`StockInquiryPage.tsx:227-240`、UI-06a-D6）。並べ替えの道具は無く、「すべて」は商品コードの昇順（`src/features/stock-inquiry/hooks/useStockInquiry.ts:61`）、在庫切れ・在庫少は取引先名 → 在庫数 → 商品名（`src/features/stock-inquiry/lib/filter-low-stock-list.ts:49-66`、UI-06a-D4）。廃番は常に除く（58 §58.10「廃番除外」） | 状態が「すべて」で検索語が空なら一覧を取らず、「商品コード、商品名、または JAN コードで検索してください」だけを出す（`useStockInquiry.ts:50`・`:81`、`StockInquiryPage.tsx:177-178`、`src/features/stock-inquiry/components/EmptySearchPlaceholder.tsx:9`。58 §58.10 契約 I）。部門だけを選んでも同じ（`isAllEmpty` は部門を見ない、`useStockInquiry.ts:50`） | 商品コード・商品名・部門・取引先・状態・在庫数・売価（`src/features/stock-inquiry/components/ProductListTable.tsx:61-67`）。状態は「すべて」では在庫切れと在庫ありだけ（58 契約 H） | 行のクリックで行の下に詳細（在庫数・売価・原価・最終入庫日・最終販売日、58 §58.7）。詳細から「在庫変動履歴」（今の URL を `returnTo` で渡す、`src/features/stock-inquiry/components/StockDetailContent.tsx:40`、UI-06a-D7）、「商品修正」（`returnTo` なし、`StockDetailContent.tsx:97`。修正の後は商品一覧へ戻る、S-14）、「入庫記録」（商品の事前入力なし、`StockDetailContent.tsx:103`） | O-1、O-2、O-3、O-4 |
| S-04 | 在庫少一覧（サイドバーの項目、UI-06b、REQ-302） | 在庫少の常設の入口（58 §58.1、D-047。独立の画面を作らず S-03 への deep-link） | S-03 と同じ。サイドバーは `/stock?status=low_stock` を開く（`src/config/navigation.ts:206-214`）。在庫切れにはサイドバーの入口が無い（チップだけ） | 在庫少（`list_low_stock(false)` のうち在庫 > 0）の全件、ページなし。0 件なら「該当する商品がありません」と「商品コード・商品名・JANコードを変えてもう一度検索してください」と「絞り込みを解除」（`StockInquiryPage.tsx:212-224`。状態が既定でないので解除の button が出る `:79`） | S-03 と同じ | S-03 と同じ | O-2、O-3 |
| S-05 | 在庫変動履歴（`/stock/$code/movements`、UI-06c、REQ-303） | 1 商品の在庫がなぜ増減したかを時系列で追う（66 §66.1、SCREEN_DESIGN §3 入出庫履歴・在庫変動追跡） | 商品は path で 1 つに固定（UI-06c-D1）。期間（`dateFrom`・`dateTo`）、種別（すべて・入庫・返品・交換・POS売上・手動販売・廃棄・破損・棚卸し、`src/features/stock-movements/types.ts:8-16`）、表示件数 50・100・200（ローカル、UI-06c-D2）、ページ | その商品の全期間の変動（日付の既定なし、`types.ts:28-42`、`src/features/stock-movements/hooks/useStockMovements.ts:46-53`）と商品のサマリ | 日時・種別・増減・変動後在庫・元記録・備考（`src/features/stock-movements/components/MovementTable.tsx:62-67`） | 元記録 → 各記録の詳細（UI-06c-D6）。「在庫照会へ戻る」→ `returnTo` か `/stock?q=商品コード&selected=商品コード`（`src/features/stock-movements/StockMovementsPage.tsx:76`・`:101`、UI-06c-D9）。入口は S-03 の詳細と 6 つの記録の詳細（`src/features/inventory-records/*RecordDetailPage.tsx` の `/stock/$code/movements` の link） | O-8 |
| S-06 | 入出庫履歴（`/inventory/records`、UI-02b/05b、REQ-206〜208） | 過去の業務記録を探して確かめる（65 §65.8.1、SCREEN_DESIGN §3 入出庫履歴・在庫変動追跡） | 記録種別、期間、記録 ID、検索欄（live、商品コード・商品名・JAN、`src/features/inventory-records/InventoryRecordsPage.tsx:201`、BIZ は `src-tauri/src/db/disposal_repo.rs:461-475`）、部門、状態（すべて・有効・取消済み・進行中）、表示件数、ページ（`InventoryRecordsPage.tsx:87-97`）。並べ替えの道具は無い（既定は業務日付の降順、65 §65.4.2） | 全種別の記録の一覧 | 種別・業務日付・代表商品・状態・記録日時・操作（`InventoryRecordsPage.tsx:338-343`） | 各記録の詳細 | O-8 |
| S-07 | 入庫記録（`/inventory/receiving`、UI-02、REQ-201） | 仕入れた商品が届いたときに記録する（61、SCREEN_DESIGN §3 入庫記録画面） | 商品追加欄: Enter で `search_products`（廃番を除く・10 件、`src/features/receiving/ReceivingPage.tsx:59-66`・`:236-262`。1 件なら追加、複数なら候補、0 件なら商品登録への導線、UI-02-D4）。入力中の候補は 5 件と超過の footer（`src/components/patterns/useProductAddSuggest.ts:6-16`、catalog ⑮ SPEC-SUGGEST-D3）。部門・並べ替えは無い | 空の明細と最近の入庫 10 件 | 候補: 商品コード・商品名・部門・操作（`ReceivingPage.tsx:517-520`）。明細: 商品コード・商品名・単位・数量・原価・操作（`ReceivingPage.tsx:563-568`） | 0 件で「商品登録へ進む」→ `/products/new`（`ReceivingPage.tsx:505`）。「在庫照会へ戻る」→ `/stock` を引数なしで（`ReceivingPage.tsx:297`・`:366`）→ S-03 の案内。最近の記録 → 記録の詳細 | O-7 |
| S-08 | 返品・交換（`/inventory/return`、UI-03、REQ-202） | お客様の返品・交換を記録する（63、SCREEN_DESIGN §3 返品・交換画面） | S-07 と同じ形の商品追加欄（`src/features/return-exchange/ReturnExchangePage.tsx:75-82`、UI-03-D9） | 空の明細と最近の返品・交換 10 件 | 候補: 商品コード・商品名・部門・現在庫（`ReturnExchangePage.tsx:769-773`）。明細: 商品コード・商品名・部門・現在庫・方向・数量（`:818-824`） | 「在庫照会へ」→ `/stock` を引数なしで（`:424`・`:501`） | O-7 |
| S-09 | 手動販売出庫（`/inventory/manual-sale`、UI-04、REQ-203） | レジを通さず売った商品の在庫を減らす（62） | S-07 と同じ形（`src/features/manual-sale/ManualSalePage.tsx:72-79`、UI-04-D4） | 空の明細と最近の手動販売（`ManualSalePage.tsx:739-746`） | 候補: 商品コード・商品名・部門・現在庫・売価（`ManualSalePage.tsx:544-549`）。明細: 商品コード・商品名・部門・現在庫・数量・販売金額（`:596-602`） | 「在庫照会へ」→ `/stock` を引数なしで（`:318`・`:408`） | O-7 |
| S-10 | 廃棄・破損（`/inventory/disposal`、UI-05、REQ-204） | 傷んだ・破損した商品を在庫から除く（64） | S-07 と同じ形（`src/features/disposal/DisposalPage.tsx:72-79`、UI-05-D5） | 空の明細と最近の廃棄 | 候補: 商品コード・商品名・部門・現在庫・原価（`DisposalPage.tsx:453-458`）。明細: 商品コード・商品名・部門・現在庫・種別・数量・原価・理由（`:505-513`） | 「在庫照会へ」→ `/stock` を引数なしで（`:290`・`:353`） | O-7 |
| S-11 | 棚卸し（`/stocktake`、UI-10、REQ-205） | 実在庫を数えてシステム在庫と突き合わせる（73 §73.1、UI-10-D1〜D3、SCREEN_DESIGN §3 棚卸し画面） | 対象確認欄: 商品コード・JAN の完全一致（`find_stocktake_item`）、無ければ商品名で `search_products`（廃番を含む・10 件、`src/features/stocktake/StocktakePage.tsx:116-123`・`:490-520`）。入力中の候補も廃番を含む（`:467`、SPEC-SUGGEST-D8）。一覧: 部門・「未入力のみ表示」（URL の `counted_only=false` が未入力のみ、`:785-795`、`src/routes/stocktake/index.tsx:12-16`）・表示件数・ページ | 進行中が無ければ開始の案内と前回の棚卸し（`StocktakePage.tsx:373-391`）、進行中なら一覧（UI-10-D1）。今の build は書く操作（開始・数の保存・確定）を止めている（`StocktakePage.tsx:91`、73 §73.15、停止 ADR） | 一覧: 商品コード・商品名・部門・現在在庫・実際の数・差異・最終カウント（`:861-867`）。候補: 商品コード・商品名・部門（`:631-634`） | 結果の画面（確定の後）。D-118 の (b) が画面の形を変える案を持つ（73 冒頭、proposed・未実装） | O-7 |
| S-12 | PLU書出し（`/products/plu-export`、UI-08、REQ-402） | 商品マスタからレジに登録する PLU のデータを書き出す（67 §67.1、SCREEN_DESIGN §3 PLU書出し画面） | 書出しの方法（差分・全件、`src/features/plu-export/PluExportPage.tsx:697`）。検索・部門・ページは無い | レジの登録状況の件数と、未反映の商品の全件（`list_plu_dirty`、`PluExportPage.tsx:202`）。0 件なら「PLU書出しが必要な商品はありません。」（`:640-642`） | 商品コード・JANコード・商品名・売価・在庫（`:654-658`） | 行からの行き先は無い | O-5 |
| S-13 | 一括価格改定（`/products/price-revision`、UI-14、REQ-105・REQ-106） | 値上げのリストと商品を照らし、1 行ずつ売価・原価を確定する（77 §77.1、SPEC-PRV-D3〜D5） | 検索欄（商品コード・商品名・JAN・メーカー品番、`src/features/products/components/PriceRevisionFilters.tsx:52-55`）、取引先（と「取引先未設定の商品も含める」）、部門、「廃番を含む」、並べ替え（商品コード・商品名・在庫数・売価、昇順だけ、`src/features/products/priceRevisionSearch.ts:125-138`）、表示件数（既定 50、`:87-92`）、ページ。URL に持つ（`:20-35`） | 廃番以外の全商品の一覧 | 商品コード・JAN・メーカー品番・商品名・現売価・現原価・現掛率・新売価・新原価（案）・確定（`src/features/products/components/PriceRevisionTable.tsx:234-246`） | 行ごとにその場で確定。0 件（絞り込みなし）で「商品一覧を開く」（`src/features/products/PriceRevisionPage.tsx:120-127`） | O-6 |
| S-14 | 商品修正（`/products/$code/edit`、UI-01b、REQ-102） | 1 商品の項目を直す（51、SCREEN_DESIGN §3 商品登録・修正画面） | なし（1 商品） | その商品のフォーム。現在庫は表示だけで直せない（`src/features/products/components/ProductForm.tsx:412-422`） | なし | 保存・戻るで `/products` へ（`returnTo` は `/products` の URL だけを受ける、`src/features/products/lib/return-to.ts:7-27`、`src/routes/products/$code.edit.tsx:28-29`）。S-03 から来ても商品一覧へ戻る | O-4 |

### 外した画面と理由

- 売上データ取込み（`/csv-import`、`/csv-import/records/$importId`）: レジの資料を取り込む作業の画面で、商品を探す・在庫を見る道具を持たない。
- 日次売上・月次売上（`/reports/daily`・`/reports/monthly`）: 売上を見る画面。部門の絞り込み（`src/features/daily-sales/DailySalesPage.tsx:14`）はあるが、在庫を出さない（`rg -l 'stock_quantity|在庫' src/features/daily-sales src/features/monthly-sales` の出力が空）。
- 商品登録（`/products/new`）と一括インポート（`/products/import`）: 既存の商品を探さない新規の作成。
- 記録の詳細 7 つ（`/inventory/{receiving,return,manual-sale,disposal}/records/$recordId`・`/csv-import/records/$importId`・`/stocktake/records/$stocktakeId`）: 1 つの記録の中身を見る画面で探す道具を持たない。在庫変動履歴への行き先として S-05 に書いた。
- 整合性検証（`/settings/integrity`）: 在庫の写しと記録を突き合わせるシステム管理の画面で、商品を探す道具を持たない（検査の結果の一覧と補正だけ）。段 2 で必要なら入れる。
- 在庫少の基準（`/settings/thresholds`）: S-04 の判定の基準を設定する画面で、商品を出さない。
- バックアップ・復元・操作ログ・取引先管理（`/settings/*`）: 商品・在庫を扱わない。
- レシートの控え（UI-09c）: 未実装（proposed、D-121）。

### 正本と code の食い違い

裁定はしない（段 3 で、採った形に合わせて直す）。

| # | 正本 | code の現物 | 備考 |
|---|---|---|---|
| X-01 | 58 §58.4 の schema の例は `selected` の上限が 20 | `src/features/stock-inquiry/types.ts:37` は 100。66 UI-06c-D9 は 100 へ上げたと書く | 58 の例が古い |
| X-02 | 61 §61.5 と SCREEN_DESIGN §3 入庫記録画面: 複数の候補と明細に現在庫を出す | 入庫の候補（`ReceivingPage.tsx:517-520`）と明細（`:563-568`）に現在庫の列が無い（`grep -c 'stock_quantity\|currentStock\|現在庫' src/features/receiving/ReceivingPage.tsx` の出力 0）。他の 3 画面は出す（S-08〜S-10） | 画面の間で列が揃っていない |
| X-03 | 50 §50.2 UI-01a-D10〜D12（撤去）: 商品一覧は PLU の列・`plu` の絞り込み・PLU 対象の一括操作を持たない | 3 つとも残る（`ProductTable.tsx:48`、`search.ts:20-26`・`ProductListPage.tsx:143-151`、`ProductListPage.tsx:216-248`） | 撤去は後続の「PLU の書出しの見直しの runtime」（backlog の該当項目、D-117）。今の食い違いは決定済みの runtime 待ち |
| X-04 | 50 §50.6: 行のクリックで商品修正へ遷移する | 行にクリックの処理は無く、操作の列の「修正」の button が遷移する（`ProductTable.tsx:53-111`） | |
| X-05 | SCREEN_DESIGN §3 在庫照会画面: 検索欄は商品コード・商品名・JAN | 「すべて」は `search_products` で メーカー品番にも当たる（`product_repo.rs:853`）。在庫切れ・在庫少は画面の中の絞り込みで 商品コード・商品名・JAN だけ（`filter-low-stock-list.ts:41`）。同じ検索欄で、チップによって当たる列が変わる | |
| X-06 | SCREEN_DESIGN §3 PLU書出し画面: 0 件なら「差分はありません」 | 「PLU書出しが必要な商品はありません。」（`PluExportPage.tsx:640-642`） | 文言だけ |

未確認（段 2 の前に確かめる）: 65 §65.4.1 の「理由キーワード」と 65 §65.4.2 の sort の範囲は、入出庫履歴の画面（S-06）に道具が無い。BIZ の契約の一覧で画面が使わない項目なのか、画面の欠けなのかは 65 の他の節と突き合わせていない。→ 段 2 の run で確かめた（`## 段 2: 白紙の骨組み` の冒頭。画面の欠けではなく、65 の完成形の残り）。

### 重なる画面の組

解決案は書かない（段 2 の入力）。

- O-1: 商品検索・一覧（S-02）と在庫照会の「すべて」（S-03）。どちらも `search_products` を keyword・部門で呼び（`useProductList.ts` の `productsQuery`、`useStockInquiry.ts:55-73`）、在庫数・売価を出す。S-02 は原価・PLU・並べ替え・廃番の表示・URL の表示件数を持ち、S-03 は取引先・状態・詳細（最終入庫日・最終販売日）と在庫切れ・在庫少への切替を持つ。初めて開いたときは S-02 が一覧、S-03 が案内だけ（UI-01a-D1 と契約 I が逆に決めている。UI-01a-D1 は「在庫照会 UI-06a の検索駆動表示を機械的に横展開しない」と書く）。
- O-2: ホームの在庫切れ・在庫少の件数（S-01）と在庫照会の在庫切れ・在庫少（S-03・S-04）。同じ `list_low_stock(false)` を同じ規則（在庫 0 以下と在庫あり）で分けるが、関数が 2 つある（`count-stock-status.ts:14-18` と `filter-low-stock-list.ts:28-34`）。件数 card から一覧へは行けない（D-089）。
- O-3: サイドバーの「在庫少一覧」（S-04）と在庫照会の「在庫少」のチップ（S-03）。同じ route と同じ表示（D-047、UI-12-D1）。在庫切れには同じ形の入口が無い。
- O-4: 在庫照会の詳細（S-03。在庫数・売価・原価・最終入庫日・最終販売日）、商品修正（S-14。現在庫・売価・原価ほか）、在庫変動履歴の商品のサマリ（S-05）。1 商品の情報を 3 か所で出し、S-03 → S-14 → 保存の後は S-02 に着く。
- O-5: 商品一覧の PLU の列と「未反映」の絞り込み（S-02）、PLU書出しの未反映の一覧（S-12）、ホームの PLU 通知バー（S-01）。商品一覧の側は撤去が決まっている（X-03）。
- O-6: 一括価格改定（S-13）と商品検索・一覧（S-02）。どちらも全商品を keyword・部門・廃番で絞り、売価・原価を出し、ページを持つ。取引先の絞り込みは S-13 だけが持つ（在庫照会の在庫切れ・在庫少は取引先名で並べるが絞れない）。
- O-7: 入出庫 4 画面と棚卸しの商品を選ぶ欄（S-07〜S-11）。同じ `search_products` を件数 10（Enter）と 5（入力中）で呼ぶが、候補・明細の列が画面ごとに違う（X-02）。棚卸しだけ廃番を含み、完全一致を先に引く。
- O-8: 入出庫履歴（S-06。商品の語で記録を探す）と在庫変動履歴（S-05。1 商品の変動を追う）。どちらも「なぜ在庫が変わったか」に答える（SCREEN_DESIGN §3 入出庫履歴・在庫変動追跡）。在庫変動履歴には商品を選ぶ道具が無く、S-03 か記録の詳細からしか入れない。

### 初めて開いたときに何も出ない所

- E-01: 在庫照会（S-03）を状態「すべて」・検索語なしで開いたとき（部門だけを選んでも同じ）。一覧を取らず検索の案内だけ（契約 I）。この状態に着く入口: サイドバーの「在庫照会」（`navigation.ts:89-97`）、ホームの入口 card（`QuickActionGrid.tsx:13`）。
- E-02: 入出庫 4 画面の「在庫照会へ戻る / 在庫照会へ」（S-07〜S-10 の行の file:line）。引数なしの `/stock` へ行くので E-01 と同じ案内に着き、入れたばかりの商品も検索し直さないと見えない。
- E-03: 在庫少一覧・在庫切れ（S-04）が 0 件のとき。「該当する商品がありません」に、検索語を変えるよう促す文と「絞り込みを解除」が付く（検索語を入れていなくても出る。`StockInquiryPage.tsx:212-224`）。
- E-04: 棚卸し（S-11）に進行中が無いとき。一覧は無く、開始の案内と前回の要約だけ（UI-10-D1）。今の build は開始も止めている（73 §73.15）。
- E-05: PLU書出し（S-12）で未反映が 0 件のとき。文だけ（X-06）。
- 参考（何も出ないのではなく、入力の画面として空から始める）: 入出庫 4 画面の明細（S-07〜S-10）は空から始め、下に最近の記録を出す。

## 段 2・段 3 の進め方（後の run）

### 段 2: 白紙の骨組み

今の画面の形を前提にせず、店の仕事から「どの場面で、何を探し・見て・直すか」を組み、それに要る画面と要素を置く。

- 入力 1（店の仕事の流れ）: `docs/project-memory.md` の「Store Premises Facts」（「いまの手作業」「営業中の作業と中断」「決めた運用」「利用者」）と `docs/SCREEN_DESIGN.md` §2「利用者の1日の動線」、`docs/spec/requirements.md` の REQ-103・REQ-201〜205・REQ-301〜303・REQ-402・REQ-105。店の事実で足りないもの（発注を考えるときに在庫照会で最初に見たいもの、入庫の後に確かめたいもの、在庫少と在庫切れを見る頻度）は repo 外の回答台帳を先に引き、無ければ owner への問いの束にする（具体的な問いか短い選択で答えられる形。owner は数日手が使いにくいので選択肢で問う）。
- 入力 2（確立した UI の考え方）: design-system の正本が引く文献だけを使う（`docs/design-system/03-philosophy.md` の「出典の一覧」）。主に当てるもの: GOV.UK の *Start with user needs* と *Do less*（画面と道具を増やさない）、Shopify Polaris の情報アーキテクチャと「リソースページ + アクションページ」（探す画面と直す画面の分け方）、IBM Carbon の Progressive disclosure とデータテーブルの型、Nielsen のヒューリスティクス（#1 状態の見える化、#6 記憶より認識）、Laws of UX の Hick の法則・チャンク化、Microsoft Fluent 2 の *Coherent*（どの画面でも同じ場所に同じもの）、原田本の Q5（一覧の操作）・Q12（操作体系はシンプルに）・Q15（ページング）。原則は `docs/design-system/04-backbone.md` の 11 の原則（特に 3・5・6・7）、owner の受け取り方は 03 の「owner の受け取り方（L3 の原文から）」（「壊さない優先」「共通化は道具」「飾りより情報」「配置の常識」）。
- 出力: 場面ごとの「探す・見る・直す」の要素の表（場面 → 要る情報 → 要る道具 → 初めて開いたときに出すもの → 次に行く先）と、それを画面へまとめた骨組み（画面の数と各画面の役割）。各要素に根拠（文献の原則か店の事実の ID）を添える。今の画面の名前と route を使わずに書く。

### 段 3: 今の設計との突き合わせと owner の採否

- 段 2 の骨組みと段 1 の表を、画面ごと・要素ごとに突き合わせ、合う所・違う所・骨組みに無い今の要素を並べる。
- 段 1 の重なりの組（O-1〜O-8）と何も出ない所（E-01〜E-05）の全部に、選択肢（少なくとも「今のまま」「役割を分け直す」「統合する」の 3 つを検討し、成り立たない案は理由を添えて外す）を作る。各案に、使い手から見て変わること、失うもの、後続の runtime の大きさの見込み（触る画面・command・route の数。測っていなければ `未実測`）、並走の lane（D-118 の (b)、D-117 の runtime、棚卸しの b1・b2）との重なりを書く。推奨は confirmed / candidate / precondition-dependent を分けて添える。
- owner に、新しい案を今の画面と並べて見せる（03 の「壊さない優先」）。採否は repo 外の回答台帳に残し、packet と正本は ID で引く。
- 採った形を正本へ置き換え（Scope の段 3 の行）、D-138 を decision-log に置き、後続の runtime の項目を backlog に置き、Test Design Matrix を書いて plan-draft へ進む。

## 段 2: 白紙の骨組み

2026-10-24、起草役（段 2 の run）。今の画面の構成を前提にせず、店の仕事の場面（2.1）と design-system が引く原則（2.2）から要素を組み（2.3）、画面へまとめた（2.4・2.5）。段 1 の O・E・X との突き合わせと段 3 の入力は 2.6〜2.8。店の事実は repo 外の回答台帳の ID（L-・CB-・QU-・FC-・TD-）と `docs/project-memory.md` の行で引き、原文は書かない。行番号は HEAD `0f100fec`（code は起点 main `843c4938` と同じ。`git diff --stat 843c4938 HEAD` の出力は本 packet の 1 file だけ）。

段 1 の未確認の解消: 65 §65.4.1 の「理由キーワード」と §65.4.2 の sort の範囲は、65 の完成形の BIZ の契約で、§65.10 が段階に分けて実装する。画面の契約 §65.8.1 の上部の道具の列挙（日付範囲・種別・記録ID・商品検索・部門・状態）には入っていない。今の入出庫履歴（S-06）の道具は §65.8.1 と合い、画面の欠けではない（完成形の残り）。O-8 の中身は変わらない。

### 2.1 店の仕事の場面

根拠のある場面だけを置く。「いつ」が台帳に無いものは無いと書く。

| # | 場面 | 誰が・いつ | 手に持つもの | 知りたいこと・したいこと | 根拠 |
|---|---|---|---|---|---|
| B-01 | 発注の準備 | 店主。発注書を書くとき。用紙は週末と週明けで分けている。1 日のどの時点かは台帳に無い | 紙の発注書（1 枚に複数の取引先を、取引先別に書く） | 在庫が無い・少ない商品を取引先別に見る。発注書に書く番号（メーカー品番）。発注するかは在庫の無いものから決め、予算で絞る。今は発注書を持って棚を目で見て回る。在庫の画面に検索の条件を足す必要は無い（QU-004。店の原文でなく、agent の要約に店が合意した形。「足さない」で、今の道具を外す根拠ではない） | L-124、L-126、L-127、L-129・CB-009、QU-004、L-130、L-174、TD-362、REQ-302、`docs/evidence/hearing-2026-09-05-stock-inquiry.sanitized.md` |
| B-02 | 入荷の検品と入庫 | 店主。品物が届いたとき。営業中にも起こり、接客で中断される | 納品書（伝票）・発注書・品物 | 伝票と発注書の数を突き合わせる。入庫を記録する。原価がマスタと違えば知る。未登録の新商品は登録する。検品の後に値付けして品出しし、値段の控えを直す（値札はアプリの外）。伝票でいちばん見る番号はメーカー品番。取り寄せ（客注）の品は入庫に入れない。入れた後に在庫の画面で何を確かめたいかは台帳に無い（2.6 の Q-2） | L-131（FC-022〜FC-026）、REQ-201、REQ-209、`docs/project-memory.md`「営業中の作業と中断」、TD-362、TD-350 |
| B-03 | 商品を探す | 店主。営業中を含む | 商品名・現物・品番・色番・サイズ | どの商品か。商品名から入り、PC で検索してメーカーを調べ、カタログか仕入先に当たり、迷えば最後に JAN で決める。サイズ・規格・色番で探すこともよくある（色番・サイズが商品のどの項目に入るかは確かめていない）。取引先で絞ってから探す使い方も想像しやすい（owner 判断。商品検索への取引先の絞り込みの露出は backlog） | L-122・FC-078、L-193・FC-077、FC-076、L-202、`docs/project-memory.md`「いまの手作業」の Product lookup の行、REQ-103 |
| B-04 | 商品の項目を直す・登録する・廃番にする | 店主（owner も時々）。営業中にも、締めの後にも。接客で中断される | 探した 1 商品か、新しい商品 | 1 商品の項目を直す・登録する | REQ-101、REQ-102、`docs/project-memory.md`「いまの手作業」の締めの後の PC 作業の行、「営業中の作業と中断」、「利用者」 |
| B-05 | 値上げで売価・原価を直す | 店主。値上げの連絡が来たとき。長いリストを数日かけて、よく売れる物から | 印刷して書き込んだ値上げのリスト（バーコード・商品名・メーカー品番・旧売価・新売価が載る） | リストの行から店の商品を見つけ、売価・原価を 1 行ずつ直す。値札を付け替える（アプリの外） | `docs/project-memory.md`「いまの手作業」の Price-revision の 2 行と値上げの PDF の行、L-197、L-067、REQ-105 |
| B-06 | 在庫の増減の理由を追う | 店主。システムの在庫と棚の数のずれに気付いたとき（いつ起きるかは台帳に無い） | 1 つの商品 | その商品の在庫が、いつ・何の記録で増減したか。元の記録 | REQ-303、REQ-207、`docs/project-memory.md`「利用者」の在庫が実在庫と合うことを望む行 |
| B-07 | 年末の棚卸し | 店主。大晦日の時点、年 1 回（多くて 2 回）。準備は 10 月から。数える時間帯は決まっていない。途中で PC を閉じて後で続ける | 棚の順の一覧（今は owner が渡した Excel）、ノート PC | 数えていない商品はどれか、前年と比べる | `docs/project-memory.md`「いまの手作業」の棚卸しの行、L-066、REQ-205。画面の形は D-118 の (b) が持ち、本 lane は要素を決めない |
| B-08 | 売れた・戻った・傷んだ分を記録する | 店主。起きたとき（頻度は台帳に無い） | 品物、返品・交換ならレシート | 手動販売・返品交換・廃棄を記録する。期限切れ・破損は今は記録せず、棚卸しでロスにしている | REQ-202〜REQ-204、L-081 |
| B-09 | 記録を後から確かめる | 店主（いつかは台帳に無い） | 日付・商品・記録の種類 | 過去の入出庫の記録を探して中身を見る | REQ-206、REQ-208 |

根拠が足りず、骨組みの前提にしない場面:

- 開店の時点で在庫の画面を開いて発注を判断する: `docs/SCREEN_DESIGN.md` §2「利用者の1日の動線」は「開店 → 在庫照会（発注判断）」と書くが、設計の想定で、台帳の店の事実（L-124・L-126）は発注書を持って棚を回る形。PC で在庫を最初に何から見るかは Q-1 に回す。
- 客に在庫を聞かれて答える: 台帳に無い。扱っていない商品を客に聞かれたときはカタログで品番を確かめる（L-151・FC-017）が、アプリの外。
- レジで売る商品を反映する（PLU）: D-117 の runtime が画面を決める（本 lane の非目的）。

### 2.2 骨組みに当てる原則

design-system の正本（04 の 11 の原則、01 の DSR、02 のパターン）と、それが引く 03 の出典の一覧だけから引く。

| # | 原則（1 行） | 出所 | 骨組みでの使い方 |
|---|---|---|---|
| P-01 | 画面は使い手の場面から組む | GOV.UK *Start with user needs*（03 出典の一覧・③）、04 原則 5 | 2.1 の場面から要素を組み、今の画面の名前から始めない |
| P-02 | 画面と道具は「なければ業務が止まる」ものだけにする | GOV.UK *Do less*（03 ③） | 同じ一覧を 2 つ持たない。店が合意した要約で「足す必要は無い」とされた検索条件（QU-004）を足さない |
| P-03 | 探して見る画面（リソース）と、記録する画面（アクション）を分ける | Shopify Polaris「リソースページ + アクションページ」（03 ⑤） | 商品を探す・見る一覧と、入庫などの入力の画面を分ける。値上げの作業は入力を持つアクションの側 |
| P-04 | 同じものに同じ言葉を使う | Polaris の業務語彙の統一（03 ⑤）、04 原則 5 | 「在庫切れ」「在庫少」「取引先」を画面をまたいで同じ意味で使う |
| P-05 | 一覧は要る列だけ、詳細は開いて見る | IBM Carbon の Progressive disclosure（03 ④） | 一覧の列を場面ごとに絞り、最終入庫日などは行を開いて見る |
| P-06 | 業務データは密に、一度に全体を見せる | IBM Carbon のデータテーブル（03 ④）、japanese-webdesign の情報密度 = 信頼（03 観点借用）、04 原則 8 | 発注の準備は 1 つの一覧で全体を見る |
| P-07 | 開いた画面が、いま何を出しているかを見せる | Nielsen #1 状態の見える化（03 ②、ux-principles は 10 のヒューリスティクスを全て採る） | 初めて開いたときに何を出しているかを見せ、出していないなら何を出していないかと次の一手を必ず出す（一覧を出すこと自体の主な根拠は P-08 と TD-413） |
| P-08 | 覚えさせず、見て選ばせる | Nielsen #6 記憶より認識（03 ②） | 何を入れれば何が出るかを覚えていなくても、開けば商品が見える |
| P-09 | 選択肢を少なくする | Hick の法則（03 の Laws of UX、04 原則 3。04 原則 3 の元の意味は強調の段階で、目立つものを少なくして、いま押すもの・見るものを 1 つに決める。入口・画面の数への適用は伸ばし） | 入口・チップ・画面の数と、1 画面の道具の数を減らす |
| P-10 | 関係する情報を塊にまとめる | チャンク化（03 の Laws of UX、04 原則 5） | 発注の準備は取引先ごとの塊に並べる |
| P-11 | どの画面でも同じ所に同じものを置き、同じ操作は同じに動く | Fluent 2 *Coherent*（03 ⑦）、04 原則 6・7 | 検索欄が当たる列、商品を選ぶ欄の列、行を開く動きを全画面でそろえる |
| P-12 | 予測どおりに動き、驚かせない | Fluent 2 *Relevant*（03 ⑦） | 「在庫照会へ戻る」のような導線は、押す前に想像した先（いま扱った商品）へ着く |
| P-13 | 一覧の操作は 1 か所にまとめ、並べ順を明示する | 原田本 Q5（03、04 原則 6） | 検索・絞り込みは一覧の上の枠に、並べ順を見せる |
| P-14 | 操作体系はシンプルにする | 原田本 Q12（03、04 原則 6。Q12 §1 の元の意味は、初めて触る人には操作体系がシンプルなほうがインタラクションの手間が少ない。design-system はこれを pager の button を上下 2 組出さない規則に当てている〈04 原則 6、02 の該当の禁止〉。画面の経路の数への適用は伸ばし） | 同じ目的に 2 つの経路を作らない |
| P-15 | 長い一覧はページで分ける | 原田本 Q15（03、04 原則 6） | 全商品の一覧はページを持つ。短い作業の一覧（発注の準備）は件数しだい |
| P-16 | 空のときは「何が無いか」と「次の一手」を言う | DSR-11（01）、02 ⑥ の空状態の標準 UI | 0 件の文言は、選んでいる条件に合わせる（検索語を入れていないのに「検索語を変えて」と言わない） |
| P-17 | 戻る導線は来た所へ戻す | DSR-18（01） | 1 商品を直した後・増減を見た後は、来た一覧の条件へ戻す |
| P-18 | 詳細は選んだ行の直下に開き、スクロールの往復を減らす | 02 ⑫ 行インライン展開 | 1 商品の詳細は一覧の行の下に開く |
| P-19 | 保存の後は結果を見せ、作業の区切りを付ける | DSR-19（01、Peak-End の法則・Zeigarnik 効果） | 記録の保存の後に、入れた商品とその在庫を見せる |
| P-20 | 中断した作業の場所がすぐ分かる | 04 原則 10（Zeigarnik 効果） | 営業中の中断（B-02・B-04）から戻ったとき、一覧の条件と位置が残る |
| P-21 | 読める大きさと区別を最低線にし、実機で確かめる | 04 原則 1・2・11（WCAG、Laws of UX。原田本は高齢の利用者への直接の規定を持たない〈Q12 caution〉） | 列を増やす案は、16px のまま横に収まるかを後続の runtime の L3 で確かめる |
| P-22 | 新しい案は今の形と並べて見せ、実機を見てから決める | 03「owner の受け取り方」の「壊さない優先」「分からないときは…実機を見てから決める」 | 段 3 で、案を今の画面と並べて owner に見せる |
| P-23 | 共通にするのは道具で、画面の構造はその画面の意味を優先する | 03「owner の受け取り方」の「共通化は道具」 | 統合は「同じ問いに答える一覧」に限り、別の問いに答える画面（記録を探す・値上げの作業）はまとめない |
| P-24 | データに無い欄や、押せると誤解させる文を置かない | 03「owner の受け取り方」の「飾りより情報」 | 足す列はデータにあるもの（メーカー品番・取引先）に限る |
| P-25 | 在庫少の一覧は状態 → 在庫数 → 売価の順 | 03「owner の受け取り方」の「配置の常識」 | 発注の準備の一覧の列の順 |

「1 つの目的に 1 つの画面」は design-system に同じ名の原則が無く、P-02・P-03・P-14 と `docs/SCREEN_DESIGN.md` §4「在庫照会のREQ統合」の理由（別画面にすると、どっちの画面かと迷う）から導いた。限定: §4 の理由は在庫の中の統合（REQ-301 と REQ-302 を 1 画面に）の記録で、商品一覧と在庫照会の間への適用は類推である。今の正本の UI-01a-D1（`docs/function-design/50-ui-product-list.md:19`）は、商品一覧を商品管理の入口（既存の商品の把握と修正の対象探し）として在庫照会と別の目的に置き、在庫照会の初めの表示を写さないと明文化している。P-09・P-14 も上の表のとおり出所の元の意味からの伸ばしを含むので、この原則は段 3 で owner に「類推」と示して使う。

### 2.3 場面ごとの要素

今の画面の名前と route を使わずに書く。要素の名は 2.4 で使う。

| 場面 | 要る情報 | 要る道具 | 初めて開いたときに出すもの | 次に行く先 | 根拠 |
|---|---|---|---|---|---|
| B-01 発注の準備 | 在庫切れ・在庫少の商品、取引先、メーカー品番、状態（切れか少か）、在庫数、売価 | 並べ順（取引先順）。今の在庫切れ・在庫少の一覧が持つ検索語・部門の絞り込みより足さない（QU-004） | 在庫切れと在庫少を 1 つの一覧に、取引先ごとの塊で（ページを分けるかは件数しだいで `未実測`） | 1 商品の詳細 → 入庫する・増減を見る | QU-004、L-130、L-174、TD-362、P-02・P-06・P-10・P-25 |
| B-02 入荷の検品と入庫 | 伝票の商品・数量・原価、マスタの原価との差、保存の後の在庫 | 商品を選ぶ欄（商品名・商品コード・JAN・メーカー品番、スキャナの Enter） | 空の明細と最近の記録（入力の画面は空から始める） | 保存の後に、入れた商品とその在庫を見る所。未登録なら 1 商品の項目（登録） | L-131、REQ-209、TD-362、P-03・P-12・P-19 |
| B-03 商品を探す | 商品名・商品コード・JAN・メーカー品番・部門・取引先・在庫数・売価・原価 | 1 つの検索欄（4 つの列に当たる）、部門、取引先（露出は backlog）、廃番の表示、並べ替え | 商品の一覧（廃番以外） | 1 商品の詳細 → 直す・増減を見る・入庫する | L-122、L-193、L-202、P-07・P-08・P-11・P-15 |
| B-04 1 商品を直す | 1 商品の全項目 | 入力の form | その商品の今の値 | 保存・戻るで来た一覧の条件と位置へ | REQ-102、P-17・P-20 |
| B-05 値上げで売価・原価を直す | 旧と新の売価・原価、メーカー品番、JAN | 1 つの検索欄（B-03 と同じ列）、取引先、部門 | 作業の一覧（廃番以外） | 行ごとにその場で確定 | L-197、REQ-105、P-03・P-23 |
| B-06 増減の理由を追う | 1 商品の増減の時系列と元の記録 | 期間・種別 | その商品の全期間 | 元の記録、来た所へ戻る | REQ-303、REQ-207、P-17 |
| B-07 棚卸し | （D-118 の (b) が決める） | 同左 | 同左 | 同左 | 非目的 |
| B-08 売れた・戻った・傷んだ | 品物・数量・種別 | B-02 と同じ商品を選ぶ欄 | 空の明細と最近の記録 | 保存の後に、扱った商品とその在庫を見る所 | REQ-202〜REQ-204、P-11・P-19 |
| B-09 記録を後から確かめる | 記録の種別・日付・代表の商品・状態 | 期間・種別・記録ID・商品の語・部門・状態 | 全種別の記録の一覧 | 記録の詳細 | REQ-206、P-23 |

### 2.4 骨組みの案

要素の名: K-1 商品の一覧（探す）、K-2 1 商品の詳細（行の下に開く）、K-3 発注の準備の一覧、K-4 1 商品の項目、K-5 記録の入力（入庫・返品交換・手動販売・廃棄）、K-6 記録を探す、K-7 1 商品の増減、K-8 値上げの作業の一覧、K-9 棚卸し、K-10 入口（件数と各画面への入口）。

統合の形（案 A）と役割の分け直しの形（案 B）を置く。2 案で違うのは K-1・K-2・K-3 の置き方だけで、他の要素は 2.5 の共通の画面に置く。

#### 案 A: 商品と在庫を 1 つの画面にまとめる（統合）

| 画面 | 受け持つ場面 | 検索の道具 | 絞り込み | 初めて開いたときに出すもの | 一覧の列 | 行から行ける先 |
|---|---|---|---|---|---|---|
| 商品と在庫（K-1 + K-2 + K-3） | B-01、B-03、B-04・B-06 の入口 | 1 つの検索欄（商品名・商品コード・JAN・メーカー品番。どの状態を選んでも同じ列に当たる） | 状態（すべて / 在庫切れ・在庫少）、部門、廃番の表示、並べ替え、表示件数。全部 URL に持つ | 段 3 の決定 2 で選ぶ（全商品か、在庫切れ・在庫少か）。どちらでも一覧が出る | すべて: 商品コード・商品名（廃番の badge と JAN を同じ欄）・部門・取引先・状態・在庫数・売価・原価。在庫切れ・在庫少: 取引先・メーカー品番・商品コード・商品名・状態・在庫数・売価（取引先順） | 行の下に詳細（在庫数・売価・原価・最終入庫日・最終販売日）→「直す」（K-4）・「増減を見る」（K-7）・「入庫する」（K-5。その商品を明細に入れた状態で開く） |

- 状態は「在庫切れ・在庫少」を 1 つにまとめ、切れか少かは状態の列で見分ける（B-01 は両方を 1 枚で見る、QU-004。状態の列は L-248）。
- 入口: サイドバーとホームから「商品と在庫」を 1 つ。発注の準備の常設の入口を残すなら、同じ画面の「在庫切れ・在庫少」を開く項目にする（今の D-047 の deep-link と同じ形）。

#### 案 B: 探す画面と発注の準備の画面に分ける（役割の分け直し）

| 画面 | 受け持つ場面 | 検索の道具 | 絞り込み | 初めて開いたときに出すもの | 一覧の列 | 行から行ける先 |
|---|---|---|---|---|---|---|
| 商品（K-1 + K-2） | B-03、B-04・B-06 の入口、1 商品の在庫を見る（REQ-301） | 1 つの検索欄（4 つの列） | 部門、廃番の表示、並べ替え、表示件数。URL に持つ | 全商品の一覧（廃番以外、商品コード順） | 商品コード・商品名（廃番の badge と JAN）・部門・取引先・状態・在庫数・売価・原価 | 行の下に詳細（案 A と同じ）→ 直す・増減を見る・入庫する |
| 発注の準備（K-3 + K-2） | B-01（REQ-302） | 1 つの検索欄（4 つの列。今の在庫切れ・在庫少の一覧の検索語の絞り込みを引き継ぐ） | 部門（今の一覧の部門の絞り込みを引き継ぐ）。これより足さない | 在庫切れと在庫少の全件を、取引先ごとの塊で | 取引先・メーカー品番・商品コード・商品名・状態・在庫数・売価 | 行の下に詳細（同じ部品）→ 入庫する・増減を見る・直す |

- 在庫の画面から「全商品を見る」形が無くなり、1 商品の在庫は「商品」で探す。サイドバーの在庫の入口は「発注の準備」の 1 つになる。
- 発注の準備に検索語と部門の絞り込みを残す理由: 今の在庫切れ・在庫少の一覧はこの 2 つで絞れる（`src/features/stock-inquiry/lib/filter-low-stock-list.ts` の `filterAndSortLowStockList(rows, q, dept, status)`）。QU-004 は「足す必要は無い」で、今の道具を外す根拠にならない（しかも agent の要約に店が合意した形で、店の原文でない）。外すと、取引先の塊の中から 1 商品や 1 部門を探す手段が無くなり、店主に「商品の画面へ移って探し直す」往復を残す（AGENTS.md「使い手に押しつけない」、03「壊さない優先」〈P-22〉）。足さないことは P-02 で守る。
- 取引先の塊を注文先（問屋）で作るか、今のメーカー/ブランドのまま作るかは、注文先の軸の設計（TD-193、`docs/backlog.md` の取引先の意味の項目、`docs/project-memory.md` の注文先の訂正の行）しだい。それまではメーカー/ブランドで並ぶ（L-130 の今の形）。案 A の「在庫切れ・在庫少」も同じ。

### 2.5 どの案にも共通に置く要素と画面

| 画面・要素 | 受け持つ場面 | 中身 | 解く段 1 の項目 | 根拠 |
|---|---|---|---|---|
| 1 商品の項目（K-4） | B-04 | 保存・戻るで、来た一覧（案 A の商品と在庫、案 B の商品・発注の準備）の条件と位置へ戻る | O-4 の戻り先、S-14 の `returnTo` が商品一覧だけを受ける点 | P-17・P-20 |
| 記録の入力（K-5、4 画面） | B-02、B-08 | 商品を選ぶ欄の候補と明細の先頭の列を「商品コード・商品名・部門・現在庫」にそろえ、画面ごとの列はその後ろに置く。保存の後の扱いは Q-2 の答え待ち: a・c なら「在庫を見る」は扱った商品だけを出した一覧へ行く（引数なしの一覧へ行かない）、b なら保存の結果の囲みに在庫を出すだけにし、在庫の画面への導線は目立たせない（どちらでも引数なしの一覧へは行かない） | O-7、X-02、E-02 | P-11・P-12・P-19、`docs/SCREEN_DESIGN.md` §3 入庫記録画面の明細に現在庫 |
| 検索欄の当たる列 | B-01・B-03・B-05 | 商品を探す欄は、どの画面・どの状態でも「商品名・商品コード・JAN・メーカー品番」に当たる | X-05 | P-11、L-174・TD-362（発注と伝票はメーカー品番） |
| 0 件の文言 | B-01・B-03 | 選んでいる条件に合わせて「何が無いか・次の一手」を言う（検索語を入れていないのに検索語を変えるよう促さない）。PLU の書出しの 0 件の文言は正本か code の片方にそろえる | E-03、E-05・X-06 | P-16 |
| 1 商品の増減（K-7） | B-06 | 今の要素のまま。入口は 1 商品の詳細（K-2）と記録の詳細。戻るは来た所へ | O-8 の「商品を選ぶ道具が無い」は、K-1 から K-2 を経て入れるので足さない | P-02・P-17 |
| 記録を探す（K-6） | B-09 | 今の要素のまま（65 §65.8.1） | O-8 は残す（記録を探す問いと、1 商品の増減の問いは別。P-23） | P-03・P-23 |
| 値上げの作業の一覧（K-8） | B-05 | 今の要素のまま。検索欄の当たる列だけ上の共通にそろえる | O-6 は残す（行ごとの確定と新しい売価・原価の入力を持つアクションの画面。P-03・P-23） | P-03・P-23 |
| 入口（K-10） | 件数と入口 | 件数の card は今のまま（D-089）。入口 card は案 A なら商品と在庫、案 B なら商品と発注の準備 | O-2 は残す（件数の数え方の関数が 2 つある点は使い手に見えない code の重複で、後続の runtime で 1 つにする候補） | P-09 |
| 棚卸し（K-9） | B-07 | D-118 の (b) が決める | E-04 は本 lane の外 | 非目的 |
| PLU | （場面から外した） | D-117 の runtime が商品一覧の PLU の列・絞り込み・一括の操作を外す | O-5・X-03 は D-117 の runtime で解く | 非目的 |

### 2.6 案ごとの突き合わせと、店の方の操作の変化

操作の数は設計からの数えで、測っていない（`未実測`）。

| 案 | 解く O・E・X | 残るもの | 初めて開いたときに見えるもの | 往復（今との比べ、`未実測`） |
|---|---|---|---|---|
| 今（比べる元） | — | O-1〜O-8、E-01〜E-05、X-01〜X-06 | 在庫の画面は案内だけ（E-01）。商品一覧は一覧 | 在庫の一覧を見るのに、検索語を入れるか状態を選ぶ操作が 1 つ要る。発注の準備は在庫少と在庫切れを状態のチップで切り替えて 2 つの一覧を見る。入庫の後の「在庫照会へ戻る」は案内に着き、商品を入れ直す。在庫照会から商品を直すと保存の後は商品一覧に着き、在庫照会へは画面を移って探し直す |
| A（統合） | O-1、O-3（在庫切れにも同じ入口が付く）、E-01、X-04（行は詳細を開き、直すは詳細から。50 §50.6 を置き換える）＋ 2.5 の共通（O-4、O-7、E-02、E-03、E-05、X-01〈58 を書き直すとき〉、X-02、X-05、X-06） | O-2・O-6・O-8（2.5 の理由で残す）、O-5・X-03（D-117）、E-04（D-118） | 一覧（全商品か在庫切れ・在庫少かは決定 2） | 開けば一覧が出る（追加の操作 0）。発注の準備は状態を 1 つ選ぶか、常設の入口から開く（切れと少は同じ一覧）。入庫の後は扱った商品が出る（Q-2 しだい、2.5 の K-5）。直した後は来た一覧の条件へ戻る。商品を直すのは 1 click から 2 click に増える（今は商品一覧の行の「修正」、`src/features/products/components/ProductTable.tsx:103-111`。案 A は行を開く → 詳細の「直す」）。1 画面の道具が 6 つ（検索欄・状態・部門・廃番の表示・並べ替え・表示件数）、「すべて」の列が 8 つになる（上の案 A の表の数え） |
| B（分け直し） | O-1（在庫の画面から全商品を外す）、O-3（入口が 1 つ）、E-01（「すべて」が無い）、X-04 ＋ 2.5 の共通 | A と同じ | 商品は全商品の一覧。発注の準備は在庫切れ・在庫少の全件 | 開けば一覧が出る（追加の操作 0）。発注の準備は常設の入口から開くだけ。1 商品の在庫を見るのは商品の画面で探す（在庫の画面からは探さない）。入庫の後と直した後は A と同じ。商品の画面も行を開いて直すので、直すのは A と同じく 2 click |

失うもの:

- 案 A: 商品管理の入口としての独立の画面（サイドバーの区分〈今は商品管理と毎日の業務に分かれる〉のどこに置くかを決め直す）。状態のチップが 3 つから 2 つになり、在庫切れだけ・在庫少だけの一覧は状態の列で見分ける。今の 2 つの URL の bookmark（旧い route を新しい画面へ寄せる手当てが要る）。商品を直す 1 click（商品一覧の行の「修正」、`ProductTable.tsx:103-111`）が 2 click（行を開く → 直す）になる。在庫だけを見たい人にも商品管理の道具（廃番の表示・並べ替え・原価の列）が見え、1 画面の道具が 6 つ・列が 8 つに増える。これは 03 の GOV.UK ③（*Do less*、典型の適用に「フィルタの組合せ」）と P-09 が案 A に逆向きに当たる所で、統合で画面の数を減らす利点と引き換えになる。
- 案 B: 在庫の画面で全商品を眺める形。商品を直す 1 click（A と同じく 2 click になる）。商品の画面は道具 5 つ（検索欄・部門・廃番の表示・並べ替え・表示件数）・列 8 つで、1 商品の在庫を見る人に商品管理の道具が見える点は A と同じ。REQ-301 を受け持つ画面が変わり、`docs/spec/requirements.md` の対応タスクの列と traceability が変わる。発注の準備は今の検索語・部門の絞り込みを残すので、それは失わない（2.4 案 B の注記）。

### 2.7 owner への問いの候補（本 run では問わない）

台帳に無く、決定 1・決定 2 の推奨を変える店の事実。owner は数日手が使いにくいので選択で答えられる形にする。段 3 の採否と同じ問い合わせに束ねられる（束ねるかと介入の数え方は Coordinator が決める）。

- Q-1（店主へ、決定 1・決定 2 を変える）: 「PC で在庫を見るとき、最初に何が出ていると助かりますか」— a 全部の商品の一覧（上から眺める）／ b 在庫切れ・在庫少の商品だけ（取引先別）／ c 何も出さず、名前や番号で探す（今の形）／ d どれでもよい。前提として伝える: 今は発注書を持って棚を回って確かめている（L-124）、欲しいのは取引先別の在庫切れ・在庫少の一覧と伺った（QU-004）。答えで変わる所: a なら決定 2 は a、b なら決定 2 は b で案 B の推奨が上がる、c なら決定 2 は c（契約 I を残し、E-01 には説明の手当てだけをする）、d なら推奨は動かず、決定 1 は A・決定 2 は a のまま（candidate。台帳の QU-004 と 2.2 の原則だけで決める）。Ordinary Operation の行が挙げた「部門で見る」は選択肢に入れない（台帳に根拠が無い。a・b のどちらでも開いた後に部門で絞れる）。
- Q-2（店主へ、2.5 の E-02 の着地を変える）: 「入庫を入れた後、すぐにその商品の在庫の数を確かめますか」— a 確かめる ／ b 確かめない（次の入庫か品出しへ）／ c 時々。答えで変わる所: a・c なら保存の後の「在庫を見る」は扱った商品を出した一覧へ。b なら保存の結果の囲みに在庫を出すだけにし、在庫の画面への導線は目立たせない。

注文先の軸（発注の準備の塊を問屋で作るか）は新しい問いにしない。owner は機能を入れると決めている（TD-193）が、設計は `docs/backlog.md` の取引先の意味の項目の lane が持つ。本 lane はそれを前提条件として扱う（2.4 案 B の注記）。

### 2.8 段 3 の入力の整理

#### 案ごとの置き換える正本と、後続の runtime の大きさの見込み

段 3 の「今のまま」を含む選択肢のために、骨組みの案ではない比べる案 C・D も置く。C は今の 2 画面の割り当てを残し、何も出ない所と戻り先だけを直す形（03 の「壊さない優先」に近い形）、D は何も変えない形。C は「E-01 だけ直して O-1 の裁定を先送りする」案で、「O-1 を今のままと裁定する」案ではない（O-1 を今のままと裁定するなら、UI-01a-D1 の「別の目的」を残す理由を D-138 に書く）。C が素直になる条件: Q-1 の答えが c（何も出さず探す）のとき、取引先の軸（TD-193）と D-117 の runtime が未解決で統合の画面の列・絞り込みが決まらないとき、並走の棚卸しの b1・b2 が棚卸しの画面を触っている間（A・B は在庫の画面と商品を選ぶ欄の共通の規則を広く触り、下の「大きさの見込みに足すもの」のとおり棚卸しの候補の列に当たり得る）。

後続の runtime の大きさは全部 `未実測` の見込み。数えた根拠: `rg -n 'to[=:] ?"/stock"' src --glob '!*.test.*' | wc -l` の出力 11（記録の 4 画面の 8 行、サイドバー 2 行、在庫変動履歴 1 行）。`find src/features/stock-inquiry -name '*.test.ts*' | wc -l` の出力 11、`find src/features/products -name '*.test.ts*' | wc -l` の出力 24（どれだけ書き換えるかは数えていない）。

| 案 | 置き換える正本の契約（Contract Ledger の行と追加） | 触る画面 | route | command | 備考 |
|---|---|---|---|---|---|
| A | 58 契約 I（§58.10、§58.1 失敗 4 状態の 4）、58 §58.10「廃番除外」（統合の画面に廃番の表示を置くため）、UI-01a-D1（50 §50.2。「機械的に横展開しない」の文は要らなくなる）、50 §50.6（X-04）、D-047・UI-12-D1（deep-link の先）、UI-06a-D1・UI-06a-D6（ページと件数を一覧の器へ）、UI-06a-D7・UI-06c-D9（戻り先）、UI-02-D4 系と SPEC-SUGGEST の列、`docs/SCREEN_DESIGN.md` §1（#4・#7）・§2（エリアと遷移）・§3（在庫照会・商品検索・一覧）・§4「在庫照会のREQ統合」（REQ-103 も入れた統合の記録へ）、02 ⑨・⑫ の canonical の記述 | 見込み 10 前後（商品一覧と在庫照会を 1 つに、サイドバー、ホームの入口 card、記録の 4 画面の戻り、在庫変動履歴の戻り、商品修正の戻り） | 一覧の route 2 → 1（旧い route は新しい画面へ寄せる）。子 route 6 つの置き場も決め直す（下の「足すもの」。2 → 1 は一覧の route だけの数で、触る route は実態より多い） | 新設 0 の見込み（`search_products` と `list_low_stock` を使い続ける） | route の統合と search params の互換は R3。段 3 の後に Boundary / Wire Contract を埋める |
| B | 58 契約 I（「すべて」ごと外すので条項を置き換える）、58 §58.1 の主動線（発注の準備へ）、UI-01a-D1（残し、後半の文を削る）、50 §50.6 と行の詳細の追加（02 ⑫）、D-047・UI-12-D1（サイドバーの入口を 1 つに）、UI-06a-D1・UI-06a-D6（「すべて」とともに消える）、UI-06a-D7・UI-06c-D9、UI-02-D4 系の列、`docs/SCREEN_DESIGN.md` §1・§2・§3・§4（REQ-301 は商品の画面、REQ-302 は発注の準備）、`docs/spec/requirements.md` の REQ-301 の対応タスク | 見込み 10 前後（A と同じ範囲。商品一覧に行の詳細を足し、在庫照会を発注の準備に変える） | 新設 0・既存の `/stock` の意味を変える（`status` の値と bookmark の互換） | 新設 0 の見込み。注文先の軸を入れるなら DB と command を足す（別 lane） | REQ の対応が変わるので traceability の再生成が要る |
| C | 58 契約 I、UI-01a-D1 の後半の文、UI-06a-D7・UI-06c-D9（記録の画面の戻り先）、58 §58.8 の 0 件の文言、UI-02-D4 系の列 | 見込み 7 前後（在庫照会、記録の 4 画面、商品修正の戻り、在庫少の 0 件） | 0 | 0 | O-1・O-3・X-04 は残る |
| D | なし（E-01・E-02 に説明の手当てを足すかは別に選ぶ） | 0 | 0 | 0 | O・E・X が全部残る |

大きさの見込みに足すもの（全部 `未実測`。上の表の「触る画面」の数に入っていない）:

- 一括価格改定の 0 件の導線「商品一覧を開く」（`src/features/products/PriceRevisionPage.tsx:120-127`、`to="/products"`）: A は行き先を統合の画面へ直す。B・C は `/products` を残すなら触らない。
- 子 route の移設（A。B は `/stock` の意味を変えるので `/stock/$code/movements` の戻り先だけ）: `/stock/$code/movements` と、`/products/new`・`/products/$code/edit`・`/products/plu-export`・`/products/price-revision`・`/products/import`。`find src/routes/products src/routes/stock -name '*.tsx' -not -name '*.test.tsx' | wc -l` の出力 8（一覧の route 2 と子 route 6）。残す親の path の下へ移すなら、各 route への link・`returnTo` の受け口（`src/features/products/lib/return-to.ts`）・到達の test も触る。
- 棚卸しの候補の列（S-11、`src/features/stocktake/StocktakePage.tsx:631-634` の 商品コード・商品名・部門）: 2.5 の K-5 の列そろえを 02 ⑮ の SPEC-SUGGEST の共通の規則として書くと棚卸しの候補にも当たり（02 ⑮ SPEC-SUGGEST-D13 は棚卸しを同じ部品の利用者に数える）、並走の b1・b2 と同じ画面を触る。A・B・C のどれでも起きる。避けるなら、そろえる対象を記録の入力の 4 画面に限ると 02 ⑮ に書く。
- traceability の再生成（A でも起き得る）: A は §4「在庫照会のREQ統合」に REQ-103 を入れ、`docs/spec/requirements.md` の REQ-103・REQ-301 の対応タスクの列が変わるなら `cargo run --bin generate_traceability` が要る（B は確実に要る）。
- 58 §58.10「廃番除外」との衝突（A）: 在庫照会は `include_discontinued = false` 固定で UI のトグルを持たない契約。統合の画面に廃番の表示を置く A は、この条項も置き換える（上の A の行の契約に足す）。B・C は在庫の一覧に廃番の表示を足さないので衝突しない。
- D-117 の runtime との順（A）: 商品一覧の PLU の列・`plu` の絞り込み・一括の操作の撤去（X-03）は D-117 の runtime の受け持ち。A の runtime が先に走ると、統合の画面に PLU の列と絞り込みを一旦載せるか、D-117 の撤去を先に入れる順にするかを決める（本 lane は D-117 の runtime の中身を決めない、非目的）。B・C は今の商品一覧に残る PLU の列が D-117 まで残るだけで、今と同じ。

#### owner の採否の選択肢の下書き

決定は 2 つ（Owner Effort Budget の段 3 の 2 回と同じ）。2.5 の共通の要素は、どの案でも入れるものとして決定 1 の説明に含め、別の決定にしない。

- 決定 1: 商品と在庫の画面の割り当て — A 統合 ／ B 分け直し ／ C 今の 2 画面のまま何も出ない所と戻り先だけ直す ／ D 今のまま。
  - 推奨 A（candidate）。理由: 2 つの一覧が同じ `search_products` を呼び、検索語・部門・廃番の除外・商品コード順の並びは既定では同じ（O-1。S-02 だけが廃番の表示・PLU・並べ替え・URL の表示件数を持ち、表示件数の既定も 100 と 50 で違う。`src/features/stock-inquiry/hooks/useStockInquiry.ts:55-73`、`src/features/products/hooks/useProductList.ts:44`）。同じ問いに答える一覧を 2 つ持つ理由が段 1 の事実に無い（P-02・P-14）。`docs/SCREEN_DESIGN.md` §4 が在庫照会の統合に挙げた理由（どっちの画面かと迷う）を、商品一覧と在庫照会の間にも類推で当てる（2.2 の限定。今の UI-01a-D1 は 2 画面を別の目的に置いている）。今の在庫照会の状態のチップと行の下の詳細を残すので、店主が見たことのある形を大きく変えない（P-22）。引き換え: 直すのが 2 click になり、1 画面の道具が 6 つに増える（2.6 の失うもの。GOV.UK ③・P-09 が逆向きに当たる）。
  - B は precondition-dependent: Q-1 の答えが b（在庫切れ・在庫少だけが助かる）で、注文先の軸の設計が見えてから推奨が上がる。
  - C は candidate の控え: owner が「壊さない優先」を強く取るなら、E-01・E-02 を最小の差で解く。C は O-1 の裁定を先送りする案で、O-1 を今のままと裁定する案ではない。素直になる条件は 2.8 の冒頭（Q-1 が c、取引先の軸・D-117 が未解決、並走の b1・b2 が棚卸しの画面を触っている間）。
  - D は推奨しない: E-01 が owner の所感（TD-413）の起点で、そのまま残る。
  - 答えで変わる所: 段 3 で置き換える正本の範囲（上の表の行）、D-047 の扱い、REQ の対応（B のときだけ）、後続の runtime の大きさ。
- 決定 2: 在庫を見る一覧を初めて開いたときに出すもの（A なら商品と在庫、C なら在庫照会。B は画面ごとに固定なので問わない。D は今の契約 I のまま） — a 全商品の一覧（廃番以外、商品コード順、ページあり）／ b 在庫切れ・在庫少（取引先順）／ c 案内だけ（今の契約 I）。
  - 推奨 a（candidate）。理由: 何を入れれば何が出るかを覚えていなくても、開けば商品が見える（P-08、E-01 が owner の所感 TD-413 の起点）。商品一覧が既に同じ形（UI-01a-D1）で、同じ一覧が同じに開く（P-11）。P-07 は a・b・c のどれでも「何を出しているか（出していないなら何を出していないかと次の一手）」を見せる条件で、a を選ぶ根拠には使わない。Q-1 の答えが b なら b に変える（precondition-dependent）。
  - c を選ぶなら、E-01 に「何を出していないか・次に何をするか」の説明を足す（P-16。今の案内の文は何を入れるかだけを言う）。
  - 答えで変わる所: 契約 I の置換えの文、読込みの失敗・0 件・範囲外の page・商品が 1 件も無い DB の場合の点検（Design Readiness の「絶対保証の例外」）、後続の runtime の test の行。
- confirmed（どの案でも同じで、根拠が design-system の既存の規則と台帳にあるもの）: 2.5 の共通の要素（戻り先〈DSR-18〉、商品を選ぶ欄の列〈Coherent〉、検索欄の当たる列〈Coherent・TD-362〉、0 件の文言〈DSR-11〉）。

段 1 の項目と決定の対応（段 3 で各項目の選択肢を作るときの入口）:

| 段 1 の項目 | 扱う所 |
|---|---|
| O-1・O-3・E-01・X-04 | 決定 1（E-01 は決定 2 も） |
| O-4・O-7・E-02・E-03・E-05・X-02・X-05・X-06 | 2.5 の共通（決定 1 の説明に含める。E-02 の着地は Q-2） |
| O-2・O-6・O-8 | 2.5 で「残す」と理由を書いた。段 3 で今のまま／変える案を owner に並べる |
| O-5・X-03 | D-117 の runtime（本 lane は変えない） |
| E-04 | D-118 の (b)（本 lane は変えない） |
| X-01 | 58 を置き換えるときに同じ PR で直す（案 A・B・C） |

## 段 3: 突き合わせと owner の採否、正本の置換え

2026-10-24、起草役（段 3 の run）。行番号は起点 main `843c4938` の code の時点（段 1 と同じ）。

### 3.1 owner と店主の答え

repo 外の回答台帳の ID で引く（原文は台帳にあり、公開 repo には書かない）。

| 問い | 答え | 台帳 |
|---|---|---|
| Q-1（店主、2.7） | a 全部の商品の一覧 | Q-101、TD-422 |
| Q-2（店主、2.7） | c 時々確かめる。確かめるのは限られた商品で、1 単位でできることが限られる品について、最低限要る在庫の数に達したかを見る | Q-102、TD-423 |
| 決定 1（2.8） | A 統合 | TD-424 |
| 決定 2（2.8） | a 全商品の一覧（廃番以外、商品コード順、ページあり） | TD-425 |

Q-1 が a なので 2.8 の推奨（決定 1 = A・決定 2 = a）は動かず、owner はその推奨を採った。Q-2 が c なので、2.5 の K-5 の a・c の扱い（保存の後の「在庫を見る」は扱った商品を出す）を採った。

### 3.2 段 1 の項目の対応表

全部の O・E・X に、採った結果と直した正本の箇所を対応させた。

| 項目 | 採った結果 | 直した正本の箇所 |
|---|---|---|
| O-1 商品検索・一覧と在庫照会の「すべて」 | 統合（決定 1 = A）。1 つの画面「商品と在庫」、route `/products`、`/stock` は寄せる | 50 冒頭・§50.1・UI-01a-D14・D19、58 冒頭、`docs/SCREEN_DESIGN.md` §1 #4・#7、§3 商品と在庫画面、§4 商品と在庫の統合、52 §52.3 の UI-01a・UI-06a の行、ARCHITECTURE・ui-task-specs の UI-01a・UI-06a、D-138 (1) |
| O-2 ホームの件数と在庫照会の在庫切れ・在庫少 | 件数の card は今のまま導線にしない（D-089）。ホームの入口 card の 4 枚目を「在庫切れ・在庫少」にし、件数の近くから 1 回でその一覧へ。件数の数え方の関数の重複は runtime の任意 | 53 §53.1・UI-00-D13、backlog の「商品と在庫の画面の統合の runtime」の任意の行 |
| O-3 サイドバーの在庫少一覧とチップ | 入口を「在庫切れ・在庫少」1 つにし、在庫切れにも同じ入口が付く。チップは 2 つ | 52 UI-12-D1・UI-06b の行、58 §58.1・UI-06a-D8、D-047 の Superseded in part の行 |
| O-4 1 商品の情報の 3 か所と戻り先 | 戻り先を直す（詳細の「商品修正」が `returnTo` を渡し、保存の後は来た一覧へ）。3 か所（行の詳細・商品修正・在庫変動履歴の見出し）は、見る・直す・増減を追うの別の問いに答えるので残す（P-23） | 50 UI-01a-D17、58 §58.7、51 UI-01b-D2 |
| O-5 PLU の列・絞り込みと PLU書出し・通知バー | 本 lane は変えない（D-117 の runtime）。統合の画面は PLU の列・絞り込み・一括の操作を持たない（D-117 が決めた撤去を写すだけ） | 50 UI-01a-D16、D-138 の Compatibility |
| O-6 一括価格改定と商品検索・一覧 | 残す（行ごとの確定と新しい値の入力を持つ作業の画面。P-03・P-23）。検索欄の当たる列だけ共通の規則 | 02 ⑨、77 の Empty の導線の label |
| O-7 入出庫 4 画面と棚卸しの商品を選ぶ欄 | 取引 4 画面の複数件候補と明細の先頭の列を「商品コード・商品名・部門・現在庫」にそろえる。棚卸しは除く（D-118 の (b)、並走の b1・b2） | 02 ⑮、61 §61.5、`docs/SCREEN_DESIGN.md` §3 入庫記録画面 |
| O-8 入出庫履歴と在庫変動履歴 | 残す（記録を探す問いと 1 商品の増減を追う問いは別）。在庫変動履歴の入口は統合の画面の行の詳細 | 66 §66.1、`docs/SCREEN_DESIGN.md` §3 入出庫履歴・在庫変動追跡 |
| E-01 在庫照会の「すべて」で何も出ない | 解消（決定 2 = a）。初めて開いたら全商品。旧契約 I を撤去 | 50 UI-01a-D1、58 §58.1 の失敗の 4、§58.7 の EmptySearchPlaceholder、§58.8 の #4、§58.10 契約 I |
| E-02 入出庫 4 画面の「在庫照会へ戻る」 | 解消（Q-2 = c）。保存の結果に保存した商品ごとの「在庫を見る」（その商品の行と状態を開く）。上の戻る導線は「商品と在庫へ」（全商品の一覧が出るので何も出ない画面に着かない） | 61 UI-02-D10・§61.3、62 UI-04-D12・§62.3、63 UI-03-D15・§63.3、64 UI-05-D11・§64.3、`docs/SCREEN_DESIGN.md` §2 の遷移・§3 入庫記録・返品・交換 |
| E-03 在庫少・在庫切れの 0 件の文 | 解消。0 件の説明を選んでいる条件に合わせる | 50 UI-01a-D18・§50.7、02 ⑥ の空状態の説明 |
| E-04 棚卸しに進行中が無い | 本 lane の外（D-118 の (b)） | なし |
| E-05 PLU書出しの 0 件 | 文を今の build の「PLU書出しが必要な商品はありません。」にそろえる（X-06） | 67 の Diff 0 件、`docs/SCREEN_DESIGN.md` §3 PLU書出し画面 |
| X-01 58 の `selected` の上限 20 | 58 を 100 に直した（code と 66 に合わせる） | 58 §58.4 |
| X-02 入庫の候補・明細に現在庫が無い | 正本に合わせて runtime で code を直す（先頭の 4 列の共通の規則） | 61 §61.5、02 ⑮、`docs/SCREEN_DESIGN.md` §3 入庫記録画面 |
| X-03 商品一覧の PLU の道具 | D-117 の runtime（本 lane は変えない。統合の画面は持たない） | なし（50 UI-01a-D10〜D12 のまま） |
| X-04 50 §50.6 の行のクリック | 置き換え（行のクリックは詳細を開く。商品修正は詳細から） | 50 §50.6・UI-01a-D17 |
| X-05 検索欄の当たる列がチップで変わる | 検索欄はどの状態でも 4 つの列（`maker_code` を含む） | 02 ⑨、58 UI-06a-D8・§58.6 |
| X-06 PLU書出しの 0 件の文言 | 正本を今の build の文に合わせた | 67、`docs/SCREEN_DESIGN.md` §3 PLU書出し画面 |

### 3.3 Q-2 の「最低限要る在庫の数に達したか」の扱い

正本の閾値の定義を実読した: 在庫少の基準は `app_settings` の `stock_low_threshold`（個数の単位の数、初期値 3）と `stock_low_threshold_fabric`（長さの単位の cm、初期値 500）の 2 つで、BIZ の `list_low_stock` が `stock_quantity <= ?` で比べる（`docs/function-design/10-common-rules.md` の在庫少の基準の行、`docs/DB_DESIGN.md` の初期値と適用ルールの行、[69](../../function-design/69-ui-threshold-settings.md)）。商品ごとの基準は無い。

- confirmed（新しい設定・field を足さない）: 統合の画面の状態の欄を、どの状態でも在庫少の集合で決める（58 契約 H の置換え）。「すべて」でも `list_low_stock(true)` を読み、集合に入る在庫ありの商品を「在庫少」、入らない商品を「在庫あり」（基準を上回る）と出す。入庫などの保存の結果の「在庫を見る」はその商品の行を「すべて」で開くので、状態の欄が「今の基準に達したか」に答える。今の契約 H のままだと「すべて」は在庫少を出さず（「在庫あり」は基準以上を保証しない）、この確かめに答えない。集合の読込み中は Skeleton、失敗は「—」と知らせで、言い切らない。
- owner への問いの候補（Open Questions の Q-3）: 店主の言う「最低限要る数」が商品ごとに違うなら、今の基準（単位の種類ごとの 2 つの値）では表せない。商品ごとの基準は商品の field・商品フォーム・`list_low_stock`・商品 CSV を変える別の lane になる。出所は店主の答え（TD-423）で、AI の review の起源の要件ではない。答えが無い間は confirmed の形で進め、本 lane と runtime の lane を止めない。

### 3.4 段 3 で決めた細部と根拠

owner の採否（A・a）の中で、画面の細部は design-system の原則から決めた（見え方は後続の runtime の Windows native L3 で owner が確かめる。P-22）。

- route は `/products`（UI-01a-D14）: URL の道具が多い側で、商品修正の `returnTo` の受け口と子の route 5 つが既にある。子の route の path は移さない（デスクトップのアプリで URL は利用者に見えない）。`/stock` は寄せる（UI-01a-D19）。
- サイドバーは「商品と在庫」を毎日の業務に、「在庫切れ・在庫少」を入出庫に（UI-01a-D14、52 UI-12-D1）: 使う頻度（SCREEN_DESIGN §1）と、今の在庫照会・在庫少一覧の位置を保つ（壊さない優先）。商品管理のエリアから商品検索・一覧を外す（2 つの入口が同じ画面を指さない。P-09）。
- ホームの入口 card は 3・4 枚目を「商品と在庫」「在庫切れ・在庫少」（53 UI-00-D13）: 2 枚が同じ画面を指さず、件数の近くから発注の準備の一覧へ 1 回で行ける。月次売上は日次売上のタブから 1 回で行ける。
- 詳細の導線の label は今のまま（「商品修正」「在庫変動履歴」「入庫記録」）: 店主が見たことのある語を変えない（壊さない優先）。
- 「在庫切れ・在庫少」では検索欄・状態・部門だけを出し、URL の値は消さない（UI-01a-D15）: 効かない道具を見せない（P-24）、足さない（QU-004、P-02）、行き来で条件を失わない。並べ順は「取引先順」と文字で出す（P-13）。
- 「在庫切れ・在庫少」の列は取引先・メーカー品番を先頭側に（UI-01a-D16）: 2.4 の案 A の表のとおり（発注書は取引先ごと、番号はメーカー品番）。
- 保存の結果の「在庫を見る」は商品ごとの link（61 UI-02-D10）: 入れた商品だけを出した一覧（2.5 の K-5 の a・c の形）は、商品コードの一覧で絞る検索が無く command の型を変える。Q-2 の答えは「限られた商品を時々」で、確かめたい商品を 1 回で開く形がその用途に合い、command の型を変えない。
- 入庫の事前入力は完全一致の `get_stock_detail` で読み、URL から外す（61 UI-02-D18）: 部分一致で似たコードの商品を入れない、F5 で 2 行目を入れない。廃番は入れない（入庫の商品追加欄は廃番を出さない）。
- 「すべて」の在庫少の集合は廃番を含めて読む（58 契約 H）: 「すべて」は廃番の表示を持つので、廃番の在庫少を「在庫あり」と出さない。

### 3.5 Open Questions

- Q-3（owner へ。店主の答え TD-423 から。決定でなく確かめ）: 「最低限要る在庫の数は、商品ごとに違いますか」— a 今の基準（個数の単位で 1 つ、生地で 1 つ）でよい ／ b 商品ごとに数を持ちたい（商品の欄を足す。別の lane、R3 以上、DB の列）／ c 部門ごとに持ちたい（別の lane）。推奨 a（precondition-dependent: 店主が確かめる商品の最低限の数が基準と大きく違うと答えたら b か c）。答えが無いときの既定は a（D-138 の形）で、本 lane の plan-gate と runtime の lane を止めない。答えの置き場は backlog の「商品ごとの最低限の在庫数」。

### 3.6 旧い言い方の sweep

置き換えた契約の旧い言い方を docs 全体で `rg` した。分類: 置換え（本 PR で直した）／今の build（統合の前の振舞いの記述として残し、runtime の lane が直す。backlog の項目の清掃の基準〈`rg` の 2 つの command〉に入る）／履歴（日付つきの経緯・更新履歴・decision-log の本文。append-only）／一般名（`list_low_stock` の機能の名や一覧の種類の名で、画面の名でない）／対象外。数は Plan Review round 3 の是正の後の head（base 同期の後）で数えた（A〜D の印の 6 行は Plan Review closure の是正の後の head で数えた。他の 12 行の数はこの head でも同じ）: `rg -n -F '<語>' docs --glob '!docs/archive/**' --glob '!docs/research/**' --glob '!docs/archive/plans/2026-10-24-screen-elements-redesign.md' --glob '!docs/archive/plans/test-matrices/2026-10-24-screen-elements-redesign.md'` の行数と file ごとの行数。

| 旧い言い方（`rg -F`） | hit の行数（file ごと） | 分類 |
|---|---|---|
| `別の目的` | 2（SCREEN_DESIGN 1、decision-log 1） | 履歴（§4 と D-138 の、置き換えた経緯の文）。旧 UI-01a-D1 の文は 0 |
| `機械的に横展開しない` | 1（decision-log 1） | 履歴（D-138 の Context） |
| `契約 I` | 22（58 13、FUNCTION_DESIGN 2、decision-log 2、50 1、backlog 1、10 1、35 1、templates/plan-packet 1） | 58（冒頭・§58.1・§58.10 は撤去の文、§58.2 の file 表・§58.5 の疑似コードは今の build、§58.7・§58.8 は撤去の注記つきの今の build、更新履歴は履歴）、50 UI-01a-D1（置き換えた旨）、FUNCTION_DESIGN の 2 行（撤去の注記）、backlog の起源の項目（解消の矢印）、decision-log（D-138 の経緯）: 置換え／今の build／履歴。10・35・templates の hit は `契約 ID` で対象外 |
| `検索駆動` | 5（FUNCTION_DESIGN 2、58 2、50 1） | 置換え（撤去の注記）／今の build／履歴 |
| `EmptySearchPlaceholder` | 11（58 7、02 2、59 1、backlog 1） | 今の build（部品の撤去とともに runtime が消す。02 ⑥ と 59 の適用除外を含む）。backlog は runtime の清掃の基準の command |
| `在庫照会へ戻る` | 23（66 4、62 2、63 2、64 2、61 1、SCREEN_DESIGN 2、decision-log 1、01 1、diagrams/current-system 2、design-system/reference の mockup 6） | 61〜64・66・SCREEN_DESIGN は「今の build は…」の注記、decision-log は D-138 の Context（履歴）、01 DSR-18 の例と diagrams は今の build（runtime）、mockup は対象外（初期提案の歴史資料） |
| `在庫少一覧` | 56（52 9、ui-task-specs 4、58 4、decision-log 3、ARCHITECTURE 2、SCREEN_DESIGN 2、backlog 2、03 2、69 2、diagrams/current-system 2、50 1、73 1、44 1、cmd-task-specs 1、UI_TECH_STACK 1、DEV_SETUP_CHECKLIST 1、design-system/reference 18） | 52 §52.4 の表・diagrams は今の build（runtime）、52 §52.3・§52.4 の型の例と §52.4 の項目の表の前の「今の build の項目」の注記・ARCHITECTURE・ui-task-specs・UI_TECH_STACK・SCREEN_DESIGN §2・50 UI-01a-D14 は「今の build は…」の注記、58 の §58.12 と 03・44・69・cmd-task-specs は一般名（CMD の機能・一覧の種類）、58・73 の更新履歴・decision-log・DEV_SETUP_CHECKLIST・backlog・SCREEN_DESIGN §5 は履歴、reference は対象外 |
| `基準以上を保証しない`（旧契約 H） | 1（58 1） | 今の build（契約 H の「今の build は…」の 1 行） |
| `行クリックは商品修正`（旧 §50.6） | 1（50 1） | 置換え（50 UI-01a-D17 の理由の欄の旧文の引用。有効な旧い文は 0） |
| `差分はありません`（X-06） | 0 | 置換え（67・SCREEN_DESIGN） |
| `include_discontinued = false`（旧廃番除外） | 1（58 1） | 今の build（廃番除外の「今の build は…」の 1 行） |
| `「在庫照会」`（画面名） | 17（ACCEPTANCE_SCENARIO 8、58 3、52 3、SCREEN_DESIGN 1、ui-task-specs 1、decision-log 1） | ACCEPTANCE_SCENARIO の手順は今の build（runtime）、58・52・SCREEN_DESIGN・ui-task-specs は「今の build は…」の注記、decision-log は履歴 |
| `同じ商品に着`（A: 業務記録詳細の `returnTo` の超過も同じ商品に着くとした旧い文） | 2（66 2） | 置換え。66:71（UI-06c-D9）と 66:85（§66.3）は在庫変動履歴の受け口の文で、同じ行に業務記録詳細の超過は `/inventory/records` に着くと書き分けてある。業務記録詳細について同じ商品に着くとする live の文は 0 |
| `ホームと在庫照会`（B: 69 の区画の説明の旧い画面名） | 1（69 1） | 今の build（69 §69.9 の D-138 の後の文に併記した「今の build は…」の 1 句）。D-138 の後の文としての live は 0 |
| `商品一覧の通常表示`（B: 02 ⑧ の廃番の確認の説明の旧い画面名） | 1（02 1） | 今の build（02 ⑧ の構造の例。直後の文が D-138 の後の説明の文を持ち、構造の例は今の build と書く）。D-138 の後の文としての live は 0 |
| `商品名、商品コード`（C: 取引 4 画面の候補・明細の旧い列の順） | 1（66 1） | 対象外（66:47 は在庫変動履歴の見出しのサマリで、catalog ⑮ の取引 4 画面の候補・明細でない）。取引 4 画面の live は 0 |
| `ORDER BY p.product_code`（D: 58 UI-06a-D4 の「すべて」の固定順） | 4（20-io-product-repo 4） | 対象外（20 の 4 行は PLU dirty と有効な商品の一覧の別の IO 関数で、`search_products` でない）。58 では 0 |
| `在庫少 / 在庫切れ`（D: 旧い 2 チップの言い方） | 2（58 2） | 58:412 は今の build（§58.7 の `StockInquiryPage` の項。冒頭の注記が挙げる項目）、58:611 は対象外（日付つきの決定 UI-06a-D1 の Rejected の文で、`status=low_stock` に pagination を当てない意味は統合の後も同じ）。§58.7 の Pagination の bullet は Plan Review closure の是正で直した |

## Contract Ledger

段 3 で置き換えた契約。実装の列は本 PR の docs の置換え、自動 test の列は Matrix の Test Matrix の行（契約の列で引く。test の名は Matrix にあり、後続の runtime の lane が足す）。

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| 58 契約 I（「すべて」の検索駆動表示）→ 撤去、UI-01a-D1 の置換え | 50 §50.2 UI-01a-D1、58 §58.10 契約 I・§58.1・§58.8 | docs（本 PR）: 50、58、`docs/SCREEN_DESIGN.md` §3 | Matrix の UI-01a-D1 の 2 行（今の `src/features/stock-inquiry/hooks/useStockInquiry.test.tsx` の status=all+q 空で enabled=false は runtime で反転） | runtime の L3（開いて一覧が出る） |
| UI-01a-D14・D19（統合と `/stock` の寄せ）、D-047・UI-12-D1 の先 | 50 §50.2、52 §52.3・§52.6、D-047 の Superseded in part、D-138 | docs（本 PR）: 50、52、58 冒頭・§58.1、decision-log、ARCHITECTURE、ui-task-specs | Matrix の UI-01a-D19・UI-12-D1 の行 | runtime の L3（サイドバーの 2 項目） |
| UI-01a-D15・§50.4（URL state と「在庫切れ・在庫少」の道具） | 50 §50.2・§50.4 | docs（本 PR）: 50 | Matrix の UI-01a-D15・§50.4 の行 | runtime の L3 |
| UI-01a-D16・UI-06a-D8・廃番除外（列・チップ・検索欄の列） | 50 §50.2・§50.6、58 §58.6・§58.7・§58.10 | docs（本 PR）: 50、58、02 ⑨ | Matrix の UI-01a-D16・UI-06a-D8 の行 | runtime の L3（8 列が 16px で収まるか） |
| 58 契約 H（「すべて」でも在庫少） | 58 §58.6・§58.8・§58.10・§58.12 | docs（本 PR）: 58、`docs/SCREEN_DESIGN.md` §3・§4 | Matrix の契約 H の行（集合の所属・取得の引数・集合の失敗・集合の読込み中・「在庫切れ・在庫少」の一覧の失敗） | runtime の L3（状態の見分け） |
| 契約 H の集合の key の D-052 の invalidation | 58 §58.10 契約 H、`src/lib/invalidation-contract.ts`（`queryKeys.lowStock(false)` を挙げる 16 の entry。`rg -c 'queryKeys.lowStock\(false\)' src/lib/invalidation-contract.ts` の出力 16。round 3 の base 同期で b1 の `stocktakeCountUpdate` が加わった） | docs（本 PR）: 58、backlog。code は runtime（production SSOT と独立 oracle を同じ PR で） | Matrix の契約 H / D-052 の行 | 非対象（自動 test で足りる） |
| 58 §58.4 の `selected` ライフサイクル（検索前ガードの撤去、部分一致の限定） | 58 §58.4・UI-06a-D7、50 §50.4 | docs（本 PR）: 58 | Matrix の UI-01a-D15 / §50.4 の条件の変更の行と 58 §58.4 の行 | 非対象 |
| UI-01a-D17・UI-06a-D7・UI-06c-D9・UI-01b-D2（行の詳細と戻り先。商品修正の `returnTo` は作る側・受け口に `status`・`selected` を足し作る側は手組み、在庫変動履歴の fallback に `discontinued=all`、`recount_required` の回復の後は `/products` の例外、在庫変動履歴の `returnTo` は 500 字を超えたら fallback の限定） | 50 §50.2・§50.6、58 §58.7・§58.10、66 §66.1・UI-06c-D9・§66.3、51 UI-01b-D2 | docs（本 PR）: 50、58、66、51、backlog | Matrix の UI-01a-D17・UI-01b-D2・UI-06c-D9 の行 | runtime の L3（2 click） |
| UI-01a-D18（何も出ない画面を作らない、0 件の説明） | 50 §50.2・§50.7、02 ⑥ | docs（本 PR）: 50、02 | Matrix の UI-01a-D18 の行 | 非対象（自動 test で足りる） |
| UI-02-D10・UI-04-D12・UI-03-D15・UI-05-D11（保存の結果の「在庫を見る」）、UI-02-D18（入庫の事前入力） | 61〜64 の該当の行・§xx.3・§61.7 | docs（本 PR）: 61、62、63、64、`docs/SCREEN_DESIGN.md` §3 | Matrix の UI-02-D10 ほか 4 画面・UI-02-D18 の行 | runtime の L3 |
| UI-02-D4 系と SPEC-SUGGEST（取引 4 画面の候補・明細の列） | 02 ⑮、61 §61.5・62 §62.5・63 §63.5・64 §64.5、`docs/SCREEN_DESIGN.md` の返品・交換画面の明細 | docs（本 PR）: 02、61〜64、SCREEN_DESIGN | Matrix の catalog ⑮ の行 | runtime の L3。棚卸し（73 UI-10-D2・UI-10-D12）は対象外（D-118 の (b)） |
| UI-00-D13（ホームの入口 card）。D-089 は変えない | 53 §53.1・§53.8 | docs（本 PR）: 53 | Matrix の UI-00-D13 の行 | runtime の L3 |
| 67 の Diff 0 件の文言（X-06、正本を今の build の文に合わせた） | 67 の Diff 0 件、`docs/SCREEN_DESIGN.md` §3 PLU書出し画面 | docs（本 PR）: 67、SCREEN_DESIGN。code は変えない | 非対象（code を変えない。この文を assert する test は今は無い〈`rg -ln 'PLU書出しが必要な商品はありません' src` の出力は `src/features/plu-export/PluExportPage.tsx` だけ〉。旧い文の再導入は 3.6 の `差分はありません` の行の sweep で見る） | 非対象 |
| 77 の 0 件の導線の label（「商品と在庫を開く」、先の route は同じ）と、`/products` へ行く他の導線・文言の画面名（一括インポートの「商品と在庫へ戻る」、PLU書出しの `confirmed` の「商品と在庫へ戻る」、在庫少の基準の画面の説明・L3-1、廃番の確認の説明） | 77 の Empty、60 UI-01c-D11・§60.5、67 §67.7、69 §69.2・§69.9・§69.12、02 ⑧、`docs/SCREEN_DESIGN.md` の商品一括インポート画面 | docs（本 PR）: 77、60、67、69、02、SCREEN_DESIGN。code は runtime | Matrix の 77 Empty の行（既存 `src/features/products/PriceRevisionPage.test.tsx` の 0 件の導線の test を test 名で引いて改める）と 60 UI-01c-D11 / 商品フォームの not found の行 | runtime の L3 |
| SCREEN_DESIGN §4「在庫照会のREQ統合」→「商品と在庫の統合」 | `docs/SCREEN_DESIGN.md` §4 | docs（本 PR） | 非対象（文書の判断の記録） | 非対象 |
| UI-06a-D1・UI-06a-D6（ページと件数） | 58 §58.10 | 変えない（「すべて」は 50 のページ、「在庫切れ・在庫少」は「全 N 件」のまま） | 既存の test のまま | 非対象 |

## Contract Probe

外部 library の振舞いで設計の根拠にしたのは 1 つ: `@tanstack/router-core` の search の直列化（`stringifySearchWith`）が、JSON として読める string（`1234567890123` のような数字だけの商品コード）を `JSON.stringify` し直して引用符つきで出すこと（51 UI-01b-D2 の作る側を手組みのままにする理由、Matrix の例の選び方）。版は lockfile の 1.171.3（`package-lock.json:3391-3392`）。確かめ方: `npm pack @tanstack/router-core@1.171.3` を展開した `src/searchParams.ts:58-80` を読み（`:70-75` が parser で読める string を stringify し直す分岐）、本体の checkout の古い `node_modules`（1.168.15）の同じ関数と diff が無いことを見た（Plan Review round 2 の是正の run）。Plan Review round 2 の両 reviewer は隔離の場所の `npm ci --ignore-scripts`（1.171.3）で `q:"1234567890123"` が `q=%221234567890123%22` になる往復を probe で再現した。runtime の lane は Matrix の `test_ui01b_d2_return_to_round_trips_status_selected_numeric_code` と `test_products_route_numeric_code_round_trip` を実 router で回し、この前提を test で固定する。それ以外の段 1 の事実は repo の code と正本の現物から取り、OS・機器の振舞いを前提にしていない。店の事実は Contract Probe でなく、repo 外の回答台帳か owner への問い合わせで確かめる（AGENTS.md の Decision and Approval Boundaries）。

## Data Safety

- owner の所感の原文は公開 repo に書かない（repo 外の回答台帳 TD-413・TD-415 の ID で引く）。
- 店の商品名・JAN・金額・商品数・部門の中身を書かない。段 2 で店の事実を使うときも台帳の ID と要約の型（「発注は在庫少から見る」のような、店を特定しない形）にする。
- 例示の値は合成のもの（`HZ-0047` のような既存の正本の例）だけを使う。

## Implementation Results

本 lane は docs だけの design lane で、画面の code・wire・migration は変えていない。owner の採否（TD-424・TD-425）と店主の答え（TD-422・TD-423）を受け、在庫照会と商品検索・一覧を 1 つの画面「商品と在庫」にし、初めて開いたら全商品の一覧を出す形を D-138 として decision-log に置き、設計正本・design-system・`docs/SCREEN_DESIGN.md` の該当の文を置き換えた（箇所は段 3 の 3.2 と Scope）。統合の実装は後続の runtime の lane（backlog の「商品と在庫の画面の統合の runtime」）が Matrix の test の設計に沿って行う。商品ごとの最低限の在庫数（Q-3）は D-138 の既定のまま、backlog の候補（owner の確認の前に必須にしない）に残した。

## Review Response

### Plan Review round 1（2026-10-24、起点 `5abacb8f`）

reviewer: Claude 側（fresh Opus 5.5。reject、P2 1 / P3 12）と Codex 側（reject、P2 8 / P3 1）。全 finding を受理した。直し方は Coordinator の裁定（相談役 Fable 5 の反例の検討を材料にした）。重なりの組: Codex 1 = Opus 1、Codex 3 = Opus 4、Codex 9 ≈ Opus 7、Codex 6 ∩ Opus 9（77 の Test Matrix の行）、Opus 10 ∩ Opus 1（数字だけの商品コードの往復）、Opus 2 ∩ Codex 2（`selected` のライフサイクル）。

| finding | 出所 | 受理 | 直した所 / 採らなかった案 |
|---|---|---|---|
| 商品修正の `returnTo` の作る側と受け口（受け口が `status`・`selected` を落とす。router の href は数字だけの値を引用符つきで出す） | Opus 1（P2）= Codex 1（P2） | 受理 | 51 UI-01b-D2 を段落ごと書き直した（作る側 `buildProductListReturnTo` は手組みのまま、両方の関数に `status`・`selected` を足す、許す範囲は変えない）。58 §58.7 の商品修正の link を `buildProductListReturnTo` と名指し、在庫変動履歴の `location.href` との規約の違いを書いた。50 UI-01a-D17、Boundary / Wire Contract（consumer・wire type・round-trip path）、Design Readiness の隣の仕様との比べ、Matrix の Contracts Under Test・UI-01a-D17 の行・Adjacent Pattern Audit、Contract Ledger の戻り先の行、backlog。採らなかった案: 受け口を引用符つきの値も復元する形に広げる（利用者が引用符ごと検索した語と区別できない） |
| `q=<code>&selected=<code>` が今のページに無い（部分一致の 1 ページ目・廃番） | Codex 2（P2） | 受理 | 58 §58.4 の `selected` ライフサイクルを統合の後の規則として書き直し、部分一致で 1 ページ目に無ければ外れる限定を書いた。61 UI-02-D10・66 UI-06c-D9 の理由に同じ限定。66 の fallback（§66.1・UI-06c-D9・§66.3・§66.5 の Header）に `discontinued=all` を足した。Matrix の UI-06c-D9 の行に廃番の商品。採らなかった案: 完全一致の 1 行表示の mode を足す（画面の形を変え、owner が採った形の外に状態を作る） |
| §58.4 の検索前ガード・UI-06a-D7 の Rejected が印なしで残る | Opus 2（P3） | 受理 | 58 §58.4 に「検索語なしの `selected` は外さない（検索前ガードは D-138 で要らない）」、UI-06a-D7 の Why・Rejected を書き直した。冒頭の注記の「今の build」を節でなく項目で示した。Matrix に `test_selected_without_q_opens_detail`・FM16 |
| `/stock` の寄せの `page` が別の行を指す | Codex 3（P2）= Opus 4（P3） | 受理 | 50 UI-01a-D19 に `page` を写すときは `perPage=50` を足す、と限定（100・200 は再現できない）。Boundary / Wire Contract の compatibility、Matrix の UI-01a-D19 の行・Compatibility Checks、backlog。採らなかった案: `page` を写さない（2 ページ目以降の `selected` も外れる） |
| 状態の変更の `page` の reset と D15 の衝突 | Codex 4（P2） | 受理 | reset 側に揃えた: 50 UI-01a-D15 の「URL の値を消さない」から pager・`page` を外し、`page` は §50.4 の条件の変更の規則で 1 に戻ると書いた。§50.4 と Matrix の Contracts Under Test・Compatibility Checks も同じ。status だけ `page` を残す例外は作らない |
| 契約 H の Matrix の行が変異に弱い | Codex 5（P2） | 受理 | Matrix の集合の失敗・読込み中の行の Would fail に「『—』と知らせ／Skeleton 以外の label（在庫少を含む）が出る」を足し、`status=low_stock` の `listLowStock` の失敗の Alert と読込み中の行を足した。Contract Ledger の「4 行」を行の中身の列挙に直した（Opus の問い B の「4 行で覆う」とは Codex の側に揃えた） |
| 67 の空の文言・77 の空の導線の契約が Ledger・Matrix に無い | Codex 6（P2）∩ Opus 9（P3） | 受理 | Contract Ledger に 2 行。Matrix に 77 の行（`test_ui14_empty_link_label`、既存 `PriceRevisionPage.test.tsx:774` を改める）と FM17。67 は code を変えないので test を足さない。発注は「既存の test のまま」と書く指示だったが、今この文を assert する test は無い（`rg -ln 'PLU書出しが必要な商品はありません' src` の出力は `src/features/plu-export/PluExportPage.tsx` だけ）ので、事実のとおり「非対象（code を変えない、旧い文の再導入は 3.6 の sweep で見る）」と書いた |
| b1 の回復の経路で「来た一覧へ戻る」保証が外れる | Codex 7（P2） | 受理 | 51 UI-01b-D2・Matrix の Contracts Under Test・Boundary / Wire Contract の round-trip path に明示の例外（`recount_required` の回復で棚卸しを経た後は `/products`、b1 の Plan Review round 3 の決定。b1 の merge の後の base 同期で 51 の b1 の文と並べて確かめる）。入れ子の `returnTo` で保証を完全にする案は b1 の決定を変えるので本 lane では書かず、backlog の候補の行（出所つき、owner の確認の前に必須にしない） |
| 旧単独の商品一覧の規範（D3 の CMD、D8 の「状態」列） | Codex 8（P2） | 受理 | 50 UI-01a-D3 を読取りの 3 つの CMD（`searchProducts`・`listLowStock`・`getStockDetail`）に置き換え、wire を変えない枠は保った。UI-01a-D8 と §50.8 の D8 の禁止を「廃番の専用の列」に書き直した（在庫の「状態」列は D16）。§50.1 の後の段落・§50.5 も同じ。Matrix に `test_ui01a_d8_discontinued_badge_not_column`（既存 `ProductTable.test.tsx:113` の「状態」見出しの不在の assert を改める） |
| runtime の清掃の一覧が網羅しない | Codex 9（P3）≈ Opus 7（P3） | 受理 | backlog の列挙を `rg` の 2 つの command（`D-138` の注記の全 hit と、印の無い旧語の hit）を基準にする形に置き換えた。58 §58.12 の画面名を「商品と在庫」に。相談役が挙げた 2 か所: 58 のテンプレ判定の表は「今の build」の印を付けた（統合の後の file・hook の形は runtime が決める範囲で、query の数と URL state の key だけを統合の後の契約〈契約 H・50 §50.4〉から 1 文で示した）。§58.1 の「呼び出す CMD」の行は統合の後の形に置き換えた（契約 H と UI-06a-D8 で決まっている） |
| 「すべて」の集合の key が D-052 の invalidation に無い | Opus 3（P3） | 受理 | 58 契約 H に義務（`queryKeys.lowStock(false)` を挙げる 15 の entry の全部と独立 oracle）、Contract Ledger の行、Matrix の `test_d052_low_stock_set_key_invalidated`・FM15・State Lifecycle、backlog |
| 0 件の文言の優先の順 | Opus 5（P3） | 受理 | 50 §50.7 の Empty を順つきに書き直した（`status=low_stock` では `discontinued` を判定に入れず「在庫切れ・在庫少の商品はありません」を先にする） |
| 入力ドキュメントの節名 | Opus 6（P3） | 受理 | 50・58・66 の入力ドキュメントを「商品と在庫画面」に |
| 触った file の触らなかった live の文 | Opus 8（P3） | 受理 | ui-task-specs の UI-06b、58 の StatusChips の見出し・取引先列・`colSpan`・UI-06a-D4、53 §53.8 の大ボタン、52 の UI-06c の行と型の例 |
| 数字だけの値の href の test が正しい実装を落とす | Opus 10（P3） | 受理 | Matrix の `test_result_panel_view_stock_links_per_saved_item` と 61 §61.9 を、押した先の parse の後の値と行が開くことで見る形に。Matrix に `test_ui01b_d2_return_to_round_trips_status_selected_numeric_code` |
| 3.6 の表が実測と合わない | Opus 11（P3） | 受理 | 3.6 の表を command と行数つきに置き換えた |
| 62〜64 の括弧の中の 2 つの話 | Opus 12（P3） | 受理 | 61 と同じく括弧の外の別の文にした |
| （起草役が是正中に見つけた）Matrix の数字だけの商品コードの例 `00123` は `JSON.parse` が先頭の 0 で失敗し、引用符つきの直列化の筋を通らない | 起草役（本 run） | — | Matrix の例を `1234567890123`（JSON の数として読める合成の値）に替え、`00123` が筋を通らないことを書いた（`node -e` で `JSON.parse("00123")` が例外、`JSON.parse("1234567890123")` が成功を確かめた） |

この変更で不要になった記述: 51 UI-01b-D2・packet・Matrix の「受け口は不変・変えない」、58 UI-06a-D7 の Rejected の「検索前ガードを撤去しない」、58 §58.4 の検索前ガードの bullet（統合の後の規則から外し、今の build との違いの 1 行に）、53 §53.8 の「商品管理は Phase 3 UI-01a 実装まで pending disabled」（実装済みの古い文）、backlog の清掃の file の列挙。

### Plan Review round 2（2026-10-24、起点 `c733d58c`）

reviewer: Claude 側（fresh Opus 5.5。reject、P2 1 / P3 6）と Codex 側（reject、P2 6 / P3 1）。全 finding を受理した。直し方は Coordinator の裁定（相談役 Fable 5 の反例の検討を材料にした）。次の round 3 が天井なので、画面の振舞いを変える直しは入れず、新しい状態を作らない。重なりの組: Codex 7 = Opus 2、Codex 1 ≈ Opus 6、Codex 5 ≈ Opus 3 の一部（66:178-179）、(A) の束〈Codex 3・4・5、Opus 3・4。58 冒頭の注記の「それ以外は統合の後の規則」が印の無い旧い規則の行で偽になっていた〉。

| finding | 出所 | 受理 | 直した所 / 採らなかった案 |
|---|---|---|---|
| UI-06a-D5 の自動展開の key が旧 4 項目で、`perPage` を URL 非依存とする | Codex 3（P2）、(A) | 受理 | 58 UI-06a-D5 の決定・境界・conditionKey を「§58.4 の条件の key〈50 §50.4〉と `page`」への参照に置き換え、`perPage` は URL の key と書いた。旧 4 項目は「今の build は…」の 1 句に。58 §58.7 の DepartmentFilter と UI-06a-D2 の決定の「`page` / `q` / `dept` / `status` に関わらず不変」も同じ参照に（sweep で見つけた） |
| 空配列を一律「該当する商品がありません」とする旧い規則 | Codex 4（P2）、(A) | 受理 | 58 §58.1 の失敗 4 状態の 2 と §58.8 の表の #2 を 50 §50.7 の Empty の順への参照に置き換え、一律の文言は「今の build は…」の 1 句に |
| 66 §66.7 の test 観点が `/stock` 以外を pin 不一致とする | Codex 5（P2）≈ Opus 3 の一部、(A) | 受理 | 66:178-179 を :180 の先例と同じ形（D-138 の後の契約 + 「今の build の test は…」）に書き直し、501 字の `returnTo` の fallback を足した。66 UI-06c-D8 の L3 の「在庫照会からの導線」にも印 |
| 清掃の基準の command が印の無い旧い言い方を網羅しない、印の無い live な行 | Opus 3（P3）、(A) | 受理 | backlog の清掃の基準を「command の出力は下限で、58 の冒頭の注記が挙げる項目と、印の付いた今の build の値はすべて対象」に直し、command (2) に `商品検索・一覧`・`stockInquirySearchSchema`・backtick の `/stock`（`\x60`）を足した（足しても完全性の根拠にしない。「商品一覧」は一般名と混ざるので Matrix の Adjacent Pattern Audit の行で見る）。名指された行: 52 §52.3 の数（:63）・§52.4 の項目の表と icon の表・§52.6 UI-12-D1 の下の bullet・§52.3 の UI-01b（編集）の行、58 UI-06a-D5 の境界、`docs/SCREEN_DESIGN.md` の商品登録・修正画面の保存の後、51 の冒頭、DSR-18 の例（`docs/design-system/01-decision-rules.md` の木の図）に印か統合の後の文 |
| §58.4 の page の reset と範囲外 page で維持する key が旧 3 項目 | Opus 4（P3）、(A) | 受理 | 58 §58.4 の 2 つの bullet と §58.8 の範囲外 page の行を 50 §50.4 の条件の key に書き換えた（「在庫切れ・在庫少」に pagination を当てないことは UI-01a-D15・UI-06a-D1 を引いた） |
| Negative Paths が `status=stockout` を不正値とする | Codex 6（P2） | 受理 | Matrix の invalid input から外し、互換の入力（`low_stock` として読む）として Compatibility Checks の old schema/input と `test_stock_redirects_to_products_with_mapping` の Would fail に直接の `/products?status=stockout` を足した（50 §50.4 の `status` の行と Boundary Checks の status/policy enum と同じ） |
| `/products` への link の他の site（一括インポート）と旧い画面名の文言を Audit が拾っていない | Opus 1（P2） | 受理 | Matrix の Adjacent Pattern Audit の `/products` への link の行を `rg -n 'to="/products"' src --glob '!*.test.*'` の出力 3 site（`PriceRevisionPage.tsx:125`、`ProductImportPage.tsx:26`・`:97`）と商品フォームの戻る・not found の案内（`ProductFormPage.tsx:203`）に替え、Ported に画面名を入れた。`/products` への link の外の可視の旧い画面名（廃番の確認の文、在庫少の基準の画面の説明）の行を足した。60 UI-01c-D11・UI-01c-D13・§60.5 の result と `docs/SCREEN_DESIGN.md` の商品一括インポート画面を「商品と在庫へ戻る」（今の build の印つき。完了の link の今の label「商品一覧を確認」は 60 の文と違っていたので印に書いた）。FM17 を広げ、`test_old_screen_name_labels_absent` の行、Contract Ledger の 77 の行、Contracts Under Test、backlog に 1 句 |
| 入庫の `productCode` の `replace` が離脱の guard に掛かる | Opus 5（P3） | 受理 | 61 UI-02-D18 に「`replace` は明細を入れるのと同じ event の中か前に行い、別の effect に分けない」と理由（`src/hooks/useUnsavedChangesWarning.ts:15`、`ReceivingPage.tsx:204`）を足した |
| `returnTo` の 500 字の上限と統合の画面の URL の長さ | Codex 2（P2） | 受理 | 超えたら既定の戻り先で同じ商品に着く（今の挙動、fail-safe）限定を 66 §66.3・UI-06c-D9 に書いた（上限は D-138 の前からあり、統合の画面の URL は key が増えて少し早く届く）。Matrix の Boundary Checks の min/max と `test_ui06c_d9_back_pins_products` の Would fail（500 字は戻る・501 字は fallback）、Contracts Under Test、packet の round-trip path、Contract Ledger の戻り先の行。採らなかった案: 上限を今上げる（入れ子の深さごとの長さの算術の設計が要る。backlog の runtime の候補に、出所つきで owner の確認の前に必須にしない） |
| 部分一致で 1 ページ目に無いと `selected` が外れ、「1 回で開く」と食い違う | Codex 1（P2）≈ Opus 6（P3） | 受理 | round 1 の限定（58 §58.4、61 UI-02-D10、66 UI-06c-D9。一覧と pager で届き、データは失わない）を保ち、D-138 の Decision (6) に同じ限定の 1 句を、Why の「1 回で開き」に「(6) の限定を除く」を足して決定の文と限定を揃えた。採らなかった案: 1 ページ目に無ければ既存の `StockDetailCard` で詳細を出す（`StockDetailContent` は状態を出さず、状態の表示を足す設計が要る。61 UI-02-D10 の捨てた案との差を起票で確かめる。backlog の runtime の候補、出所つき、owner の確認の前に必須にしない）、`q` なしの `/products?selected=<code>` にする（既定の一覧の 1 ページ目に無ければ §58.4 で即外れ、E-02 と D-138 (6) の記録と衝突する） |
| router-core の版が lockfile と違う、Contract Probe の文が設計の根拠と合わない | Codex 7（P3）= Opus 2（P3） | 受理 | 51 UI-01b-D2 と Boundary / Wire Contract の wire type を lockfile の 1.171.3（`src/searchParams.ts:58-80`）に直した。Contract Probe を、根拠にした library の振舞い（直列化）・版・確かめ方（`npm pack @tanstack/router-core@1.171.3` を展開して読み、古い `node_modules` の 1.168.15 と同じ関数に diff が無いことを見た。round 2 の両 reviewer の probe）を書く形に置き換え、Impact Review Lenses の Fact check の行から参照した |
| 66 UI-06c-D9 の中に 2 つの版の契約が継ぎ足されている | Opus 7（P3） | 受理（書き直しは runtime） | 今回は全面の書き直しをしない（round 3 の前に大きな差分を作らない）。区切りの文「以下の `/stock` と『在庫照会へ戻る』は今の build の値」で印は完結しており、runtime の清掃（backlog の基準の「印の付いた今の build の値」）で今の build の部分を消す |

この変更で不要になった記述: 58 UI-06a-D5 の「`perPage`（page ローカル state、URL 非依存）は対象外」と旧 4 項目の key の規則、58 §58.1・§58.8 の空配列の一律の文言の規則、58 §58.4 の「`q` / `dept` / `status` の変更で `page=1`」と範囲外 page の旧 3 項目、66 §66.7 の「`/stock` 以外の pathname は pin 不一致」、Matrix の不正値の一覧の `status=stockout`、backlog の「対象は command の出力で決める」の完全性の主張、51・packet の router-core 1.168.15、Contract Probe の「外部 library の振舞いを前提にしていない」（統合の後の規則に置き換えるか今の build の 1 句にした）。

### Plan Review round 3（2026-10-25、起点 `5a7de800`。天井）

reviewer: Claude 側（fresh Opus 5.5。reject、P2 1 / P3 5）と Codex 側（reject、P2 4 / P3 1）。両者とも画面の構成は決め直さず文の置換えで閉じると判定し、Coordinator は天井の disposition として全 finding を受理し一括で是正すると裁定した。画面の構成・runtime の挙動・上限の値は変えていない。是正の前に `origin/main`（`8db97271`、b1 の archive を含む）を単段で merge した（衝突なし）。重なりの組: Codex 1 = Opus 1、Codex 2 + Opus 3、Codex 4 + Opus 2。

| finding | 出所 | 受理 | 直した所 / 採らなかった案 |
|---|---|---|---|
| A: 業務記録詳細の `detailReturnTo` が 500 字を超えても同じ商品に着くと書く（偽。詳細の既定は `/inventory/records`） | Codex 1（P2）= Opus 1（P2） | 受理 | 66 UI-06c-D9 の限定と §66.3 の 1 つ目の bullet を受け口ごとに書き分けた（在庫変動履歴の `returnTo` の超過は fallback で同じ商品、業務記録詳細の `returnTo`〈入れ子の `detailReturnTo`〉の超過は詳細の既定 `/inventory/records` で在庫変動履歴と商品を失う。どちらも今の挙動）。Matrix の Boundary Checks の min/max、packet の round-trip path も同じ。採らなかった案: 上限を上げる（既存の backlog の候補のまま、項目は足さない） |
| B: 69・02 ⑧ の画面に出す文が旧い画面名のまま、Matrix の「商品と在庫に直す」の新しい文が正本に無い | Codex 2（P2）+ Opus 3（P3） | 受理 | 69 §69.2・§69.9 の区画の説明・§69.12 L3-1 に D-138 の後の文（「ホームと商品と在庫の…」、proposed・未実装、今の build の文を併記）。02 ⑧ の構造の例の直後に D-138 の後の説明の文（構造の例は今の build の canonical のまま）と更新履歴。Contract Ledger の 77 の行に 69・02・67 を足した |
| C: 返品明細と残る 3 画面の候補の先頭が「商品名、商品コード」 | Codex 3（P2） | 受理 | `docs/SCREEN_DESIGN.md` の返品・交換画面の明細、62・63・64 §xx.5 の候補を「商品コード、商品名、…」に置換 |
| D: 58 UI-06a-D4 の「すべて」の固定順、旧い 4 key・状態の値・2 チップの言い方 | Codex 4（P2）+ Opus 2（P3） | 受理 | UI-06a-D4 の「すべて」の並び順とページを「50 §50.4 に従う」へ置換し、Why の固定順の理由の文を削った。D4 の条件を `status === "low_stock"`、D6 を「在庫切れ・在庫少の絞り込み時」、§58.5 の後の部門候補の bullet を「§58.4 の条件の key（50 §50.4）と `page`」、正規化の契約の bullet から撤去する `EmptySearchPlaceholder 判定` を削った |
| E: Scope の固定の件数（20）と一覧の漏れ | Codex 5（P3） | 受理 | 固定の件数を削り、`01-decision-rules.md`・`60-ui-product-import.md` と round 3 で触った `69-ui-threshold-settings.md`・62〜64 の §xx.5・66 §66.3・67 §67.7・02 ⑧ を一覧へ |
| F: 67 の `confirmed` の「商品一覧へ戻る」 | Opus 4（P3） | 受理 | 「商品と在庫へ戻る（D-138 の後。proposed・未実装、今の build に戻る導線は無い）」に置換し、Matrix の Adjacent Pattern Audit の `/products` への link の行に正本の label の site として足した |
| G: 3.6 の `在庫少一覧` の内訳 `design-system/reference 21` | Opus 5（P3） | 受理 | 3.6 の command で数え直して 18（他の内訳 38 + 18 = 56）。round 3 の是正で `EmptySearchPlaceholder` が 12 → 11（58 8 → 7）になったので同じ表の行と数えた head の文も直した |
| H: 51 UI-01b-D2 の「b1 の merge の後の base 同期で…確かめる」の予告 | Opus 6（P3） | 受理 | base 同期の後の 51 の冒頭の b1 の文（「商品フォーム自身の `returnTo` は棚卸しへ渡さず、戻った後の保存は `/products` へ戻る」）と UI-01b-D2 の例外が 1 通りに両立することを確かめ（b1 の code も棚卸しへ `returnTo=/products/{商品コード}/edit` だけを渡す）、予告の括弧を「本書の冒頭の b1 の文と同じ」に置換 |
| （base 同期で変わった数）`queryKeys.lowStock(false)` を挙げる entry の数 | 是正の run | — | b1 の `stocktakeCountUpdate` が加わり 15 → 16。58 契約 H と Contract Ledger の数を直した（契約は「entry の全部」のまま）。Matrix の `ProductFormPage.tsx:203` も b1 の後の `:222` に直した |

この変更で不要になった記述: 66 の「業務記録詳細の入れ子の `detailReturnTo` が超えても同じ商品に着く」、58 UI-06a-D4 の「すべて」の `ORDER BY p.product_code` と「並び順を変えない」の理由、58 §58.5 の後の `EmptySearchPlaceholder 判定` と旧 4 key の部門候補の文、packet の Scope の固定の件数、51 の base 同期の予告。

### Plan Review closure（2026-10-25、対象 `2f359519`）

reviewer: Claude 側（fresh Opus 5.5。reject〈形式上〉、新しい P1 0 / P2 0 / P3 3。A〜H の中身は閉じた、C・D は固定が無く未 closure）と Codex 側（reject、新しい finding 0。A〜H の本文は閉じた、A〜D は固定が無く未 closure）。Coordinator の裁定: 固定の試行の規則を守り A〜D の固定（drift sweep の記録と、新しい文を名指す Matrix の test の行）を足し、P3 の N1〜N3 も直す。本文の契約・画面の構成・runtime の挙動は変えていない。

| finding | 出所 | 受理 | 直した所 / 採らなかった案 |
|---|---|---|---|
| A の固定: 業務記録詳細の側の期待（501 字で `/inventory/records`）を固定する test の行が無い | Codex（closure） | 受理 | Matrix の在庫変動履歴の行（`test_ui06c_d9_back_pins_products`）の直後に対の行 `test_ui06c_d9_record_detail_over_limit_returns_to_records` を足した。使えない `returnTo` で `/inventory/records` を指すことは既存 `ReturnToFlow.test.tsx` の T9 が assert しているので test 名で名指し、501 字を入れる既存の test は無いので runtime の lane で足すと書いた。Boundary Checks の min/max の「runtime の test の行を足さない」をこの行への参照に置き換えた。今の挙動なので L3 には足さない |
| B の固定: Matrix の文言の行が 69・02 ⑧ の新しい文を名指さない | Codex（closure） | 受理 | Matrix の `test_old_screen_name_labels_absent` の行に 69 §69.2・§69.9・§69.12 と 02 ⑧ の D-138 の後の文を名指し、改める既存の assert を test 名で足した |
| C の固定: 列の順の test の行が入庫だけ | Codex（closure）、Opus（closure） | 受理 | Matrix の catalog ⑮ の行を 4 画面（61〜64 の §xx.5 と `docs/SCREEN_DESIGN.md` の返品・交換画面の明細）に広げ、test の名を `test_transaction_candidate_and_line_columns` にした（Adjacent Pattern Audit の行も同じ名）。3 画面の明細の順は既存の「⑮ SC19: …」が assert している |
| D の固定: 戻った一覧の実際の並びと、条件変更の `dir` | Codex（closure）、Opus（closure） | 受理 | Matrix の `test_ui01a_d15_low_stock_hides_tools_and_keeps_url` に、「すべて」へ戻した一覧が商品名の降順（payload `sort_key=Name`・`sort_order=Desc`、行の順）でない期待を、`test_ui01a_d15_condition_change_resets_page_and_selected` に `dir` を足した |
| A〜D の drift sweep の記録が 3.6 に無い | Codex（closure）、Opus（closure） | 受理 | 3.6 の表に A〜D の旧い言い方の 6 行（command と hit の数、分類）を足した |
| N1: `PriceRevisionPage.test.tsx:774` が base 同期で古い | Opus N1（P3） | 受理 | Matrix の 77 Empty の行と Contract Ledger の 77 の行を、行番号でなく test 名で引く形にした（`docs/DOC_STYLE_GUIDE.md` §1）。round 1 の Review Response の `:774` は履歴なので直さない |
| N2: 69 の「ホームと商品と在庫の…」が 3 つの画面とも読める | Opus N2（P3） | 受理 | 69:15・§69.9 の区画の説明・§69.12 L3-1 の画面名を括弧で囲み「ホームと「商品と在庫」の…」にした。02 ⑧ は「商品と在庫」が 1 か所だけなので変えない |
| N3: 58 §58.7 の Pagination の bullet の旧い 2 チップの言い方 | Opus N3（P3） | 受理 | 「在庫切れ・在庫少〈`status=low_stock`〉は…」にした |

この変更で不要になった記述: Matrix の Boundary Checks の「runtime の test の行を足さない」、Matrix と Contract Ledger の `PriceRevisionPage.test.tsx:774`、Matrix の `test_receiving_candidate_and_line_columns`（4 画面の名に置き換えた）。

### Plan Review closure 2（2026-10-25、対象 `edbd6046`、Coordinator）

Claude 側（fresh Opus 5.5）= approve（P1 0 / P2 0 / P3 2）。Codex は利用上限（解除 2026-10-31）で、owner の決定（repo 外の回答台帳 TD-438）により reset の後に Codex の closure を流す。P3 の 2 件を受理して直した: (1) Contract Ledger の UI-02-D4 系の行の出典と docs に 62〜64 の §x.5 と SCREEN_DESIGN の返品・交換画面の明細を足した（Matrix の行に合わせた）。(2) Matrix の `test_ui01a_d15_low_stock_hides_tools_and_keeps_url` の行の並びの期待を、mock が返した順を見るだけの同語反復にならない形にした（mock は商品名の降順で返し、3 商品は取引先順と違う順にして、行は mock の順のまま出す〈画面で並べ直さない〉）。

Final Review と merge（2026-10-27、closeout で追記）: Final Review broad（head `892fd56e`）は Claude 側 Fable 5 = approve（P3 2、`63976d9e` で直した）と Codex 588 = approve（P3 1）。base 同期（`50f97a76`）と Fable の P3 の是正の後の closure（head `77131caa`）は Codex 592 = approve（新規 P1 / P2 0、P3 1）で、helper の review の record は pass。Ready の後の hosted CI の `Merge gate` が success で、owner の承認（repo 外の回答台帳 TD-448）の後に helper 経由で merge した（[PR #202](https://github.com/kosei-w90607/inventory-system-desktop/pull/202)）。merge の前に受けた P3 のうち次の 2 件は本 closeout で直した: (1) Codex 588: Matrix の Mutation Probes の「分岐の反転」の検出先が、その分岐を通らない component（`test_ui01a_d16_columns_by_status`）と純関数（`test_ui06a_d8_*`）の test だったので、実際の一覧の結果を見る hook / page の test（`test_ui01a_d1_default_lists_all_active_products`）に替えた。(2) Codex 592: Workflow State の Final Reviewer 行の「Fable 5.1」を、実際に Final を行った Fable 5（座組表の当面の値、TD-410）にした。
