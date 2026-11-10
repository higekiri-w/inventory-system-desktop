# Plan Packet: JAN を直した商品の前の JAN の枠で売れた分を同じ商品へたどる（前の JAN の別名）の設計を正本に置く

2026-10-17 に同時に起票した 6 本の lane（repo 外の回答台帳 TD-304・TD-295）の 1 本で、wave に属さない単独の lane として扱う（design lane、docs だけ。runtime は後続の lane）。owner の決定（repo 外の回答台帳 TD-284・TD-286）を、売上の取込み・商品 CSV・商品フォームとの継ぎ目まで含めて最初から設計する。並走の lane は「単位の語の直し」の runtime（branch `agent/stock-unit-relabel`）と「売上の SD からの一括取込みと EJ の控えの閲覧」の design（branch `agent/sd-bulk-import-design`）ほか 4 本で、重なる file は下の「節の所有表」で分ける。本 lane の branch は `agent/jan-alias-design`（起点 main `f5d7aaf4`）。

規模と手法（起票の最初に数えた。file の数は `git diff --name-only f5d7aaf4 HEAD -- docs ':(exclude)docs/plans'` の 17 file）: 触る設計正本は 15 file（db-design の master-tables・plu-tables・pos-tables、`DB_DESIGN.md`、function-design の 20・22・30・32・33・42・51・60、architecture の biz-task-specs・cmd-task-specs、`SCREEN_DESIGN.md`）と decision-log・backlog。画面 1（UI-01c 商品一括インポートの重複行の問い。UI-01b は振舞いを変えず 1 行の決定だけ）。migration 1（表 1 つと trigger 3 つ。後続の runtime の lane が足す。22 §18）。wire の型 2（`ImportDuplicateRow.old_jan_question` と `commit_import` の 3 つ目の引数 `old_jan_aliases`（`{product_code, old_jan}` の組））。横断の約束がかかる経路は 3 つ: (a) 商品が JAN を持つ経路（商品フォームの新規、商品 CSV の新規・上書き、将来の経路）、(b) 売上の引き当て（Z004 の preview と D-118 の確定の TX の取り直し）、(c) `products` を作り直す migration（vB・D-118 の写しの列の migration・後の全て）。**1 lane のまま進め、横断の約束は各 1 か所に置く**: (a) は DB の trigger 1 か所（master-tables §1a）、(b) は引き当ての関数 1 つ（32 §15.3 手順 4 b）、(c) は 22 §18 の規則 1 つと最新の版での trigger の存在の test。分ける案（表と trigger の lane と、問いと UI の lane）は採らない: 問いの条件と不変条件は同じ規則の両面で、分けると片方の lane が他方の未設計の前提に依る。前の lane（D-117）の教訓（別名を枠の列に置き、消す処理を経路ごとに書いた形が round 2 で 4 件の継ぎ目を出した）に従い、別名の置き場（枠 → 商品の側の表）と消し方（経路ごと → trigger）の両方の形を変えた。

## Workflow State

- Phase: archive
- Risk: R3
- Plan Commit: af1479a5ff3c8605c7794bb61a113f70dc4a8b29
- Amendments: none
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（本 run の起草役）
- Plan Reviewer: fresh Opus 5.5 + Codex（GPT-6.1 Sol / high。`.local` の model の選び方の「R3 で複数の層にまたがる契約の初回」の行）
- Final Reviewer: Fable 5.1（Claude 側、R3・design lane）+ Codex（GPT-6.1 Sol 既定）。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Review Minimum: 1
- Human Gate: ready,merge
- Branch: agent/jan-alias-design

遷移の記録:

1. kickoff → spec-check（2026-10-17、起草役）: owner の決定（repo 外の回答台帳 TD-284 の c、TD-286 の β、TD-296 の 6b）と、D-117 の Plan Review round 2 の 4 件（`docs/backlog.md` の「TD-284 の lane」の項目の (1)〜(4)）を Scope にし、Risk を R3 と記録した（下の Risk）。
2. spec-check → design（2026-10-17、起草役）: 触る正本（32 §15.3 手順 4 b の引き当て、30 §4.8・§4.9 の商品 CSV の preview と commit、42 §22.6 の `commit_import`、60 UI-01c、20 の product_repo、master-tables、22、33 §16.6）は別名を持たず、「前の JAN を持つ商品が無ければ `unmatched_product`」で閉じている。別名の表・不変条件・問い・答えの wire・引き当ての段が無いので、同じ commit で設計正本を更新した（下の Design Readiness）。
3. design → plan-draft（2026-10-17、起草役、同じ commit）: 条件「設計の出力が正本にある」は Design Readiness の引く正本と D-122 で満たす。条件「未解決の設計の問いが無い」は、owner の決定（TD-284・TD-286・TD-296 の 6b）で方向が決まり、境界の 4 件を正本の規則で閉じ、残る owner への確かめ（下の「owner への確かめ」の Q1）は答えが無い間の既定を D-117 の受け入れ済みの限界に置いたことで満たす。plan-gate への遷移は Coordinator。
4. plan-draft → plan-gate（2026-10-17、Coordinator）: packet・Matrix と正本は起草役の commit（`412d2a36`）にある。並走の 367（`agent/sd-direct-read-runtime`）も 22 に §17・MNT-03-D15 を新設していたので、本 lane の節を §18・MNT-03-D16 に振り直した（本 commit。22・master-tables・decision-log・backlog・本 packet・Matrix の参照、計 20 行）。Coordinator は D-122 の決定・置き場（商品の側の表と trigger 3 つ）・問いを出す条件・`commit_import` の確かめ直しを読み、owner の決定（TD-284 の c、TD-286 の β、TD-296 の 6b）の写しに食い違いが無いこと、D-117 の round 1 で崩れた継ぎ目（reserved の枠の別名、経路ごとの消し方、共有の同時変更、CMD・UI の境界）がそれぞれ置き場か条件で塞がれていること、code・migration に触れていないことを確かめた。owner に聞く Q1（前の JAN の枠の値段）は既定 a（D-117 の受け入れ済みの限界）で進める。Plan Review round 1 へ出す。
5. plan-gate（round 1 の是正、2026-10-18、Coordinator）: Plan Review round 1（Claude 側 fresh Opus 5.5・Codex GPT-6.1 Sol）の是正（Review Response 参照）。是正の前に相談役（Fable 5.1）に反例探しを頼んだ。起草役が `f50d0bfd` で直し、Coordinator が差分（30 §4.9 の手順 0b・2 e. と §4.9.2 の preview の判定・答えの運び方・commit の判定の表・回復・そのほかの限界、42 §22.6、60 UI-01c-D17、20 の DbError の対応、master-tables の別名の UPDATE の段、Matrix の追加の行）を読んだ。置き場（商品の側の表と trigger）と Scope の範囲は変えず、その場で直したので plan-gate に留める。Plan Review round 2 へ出す。
6. plan-gate（round 2 の是正、2026-10-18、Coordinator）: Plan Review round 2（Claude 側 fresh Opus 5.5・Codex GPT-6 Astra）の是正（Review Response 参照）。起草役が `dd2df899` で直し、Coordinator が差分（30 §4.9.2 の commit の判定表 J1〜J7 と、§4.9・42 §22.6・60 UI-01c-D17 がそれを引く形、問いの条件と成立の条件の分け方、backlog の runtime の順と依存、Matrix の T4・T5・T8）を読んだ。判定・error・回復の置き場は判定表の 1 か所になった。Scope の範囲は変えず、その場で直したので plan-gate に留める。Plan Review round 3（天井）へ出す。
7. plan-gate（round 3 の是正、2026-10-18、Coordinator）: Plan Review round 3（Claude 側 fresh Opus 5.5・Codex GPT-6.1 Sol）の是正（Review Response 参照）。round 3 は round の天井で、owner が「直して、追加の round なしで plan-approved」と決めた（repo 外の回答台帳 TD-317、介入 1 回目 / 予算 9 回）。起草役が `8f050ee6` で直し、Coordinator が差分（60 UI-01c-D17 の warning の全件のまとめと問いの出し入れの範囲、30 の J4 の回復、32 の引き当ての結果の 3 つの区別、master-tables の読む経路、Matrix の追加の行）を読んだ。
8. plan-gate → plan-approved（2026-10-18、Coordinator、本 commit）: 独立の Plan Reviewer（Writer と別の context）のうち Claude 側が round 3 で P1・P2 が無いと報告し、Codex の P2 2 件は owner の決定（repo 外の回答台帳 TD-317）どおり直した（Review Response 参照）。Plan Commit は round 3 の是正と記録の後の版。
9. plan-approved → implementing（2026-10-18、Coordinator、本 commit）: docs だけの design lane で、設計の出力は Plan Commit までに正本にある。状態だけを進め、Final Review（Minimum 1）へ出す。

owner への確かめ（設計を止めない。答えが無い間は既定で進む）:

- Q1（前の JAN の枠の値段）: 別名にした前の JAN の枠は、レジに残った前の値段のまま売れる。a = 今のまま（アプリは前の JAN の枠を書き出さない。売るのをやめた商品と同じ受け入れ済みの限界、D-117・TD-285）／b = 別名の商品の今の値段で前の JAN の枠も書き出し続ける（D-117 (1) の「対象の商品を持たない枠は書き出さない」を変え、PLU の書出しの見直しの runtime の範囲が増える）。推奨 a（前のバーコードの在庫は売り切れば終わり、値段を変えた時だけ差が出る）。答えが無いときの既定 = a。

## Owner Effort Budget

- 介入回数上限: 9（既定の 6 から owner が改定、repo 外の回答台帳 TD-296 の 5a）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 9（owner の改定、TD-296） | 3（closeout 時点。Plan Review round 3 の天井の決定 1〈TD-317「直して進める」〉、Ready 1、merge 1。L3・manual は無し。plan-approved は TD-296 の 4a で Coordinator が進めた。Q1 は聞かず既定 a。TD-284・TD-286 は D-117 の lane の介入に数えた。TD-296 は起票の前の先取りの問い） | 0 | 6 | 9 = 3 + 0 + 6 |

既定値・数え方・上限に届くときの扱いは `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
Risk Tiers の R3 の行の「DB」（新しい表と trigger の migration）、「POS CSV」（Z004 の商品の引き当て）、「Tauri command DTO」（`commit_import` の引数と `ImportDuplicateRow` の field）、「operator workflow」（商品 CSV の上書きの問い）の契約を決める設計。本 lane は docs だけだが、後続の runtime が写す契約を決めるので R3。R4 ではない: 本 lane はデータを動かさず、設計した migration は表と trigger を足すだけで既存の行を変えない。trigger が消すのは別名の行だけで、別名は店の問いの答えから作られ、商品がその JAN を持った時点で要らなくなる（売上・在庫・商品の行は消さない）。この変更で required gate の green / red は変わらない: docs だけで、classifier は function-design の変更で traceability の検査（rust_drift）を回すが、REQ の参照は増減していない（AC10）。設計書の code block に足した関数（`find_jan_alias`・`insert_jan_alias`）は code に無いが、設計照合 test は設計書にあって code に無い関数を INFO として扱い失敗にしない（`src-tauri/tests/design_compliance_test.rs:8`）。Scope の予定 path を `scripts/ci/classify-changes.sh` の workflow の一覧（`AGENTS.md`・`CLAUDE.md`・`docs/DEV_WORKFLOW.md`・`docs/ci.md`・`docs/AGENT_OPERATING_MANUAL.md`・`docs/code_review.md`・`docs/quality/review-checklist.md`・`docs/project-profile.md`・`docs/agent-guidance/*`・`docs/templates/*`・`.agents/*`・`.claude/{rules,commands,skills}/*`・PR template と、full に倒す `scripts/*`・`.github/*`）に当てて、当たる file は無い。Final Review Minimum 1。

## Goal

Goal Invariant:

### 最小完了条件

- 後続の runtime の lane（backlog の「JAN の別名の runtime」）の Writer が、チャットの履歴と repo 外の台帳を見ずに、source docs だけで次を実装できる: 表 `product_jan_aliases` と 3 つの trigger の migration（master-tables §1a、22 §18 MNT-03-D16）、`products` を作り直す migration の規則、IO-01-D5 の 2 関数、BIZ-03 の引き当ての関数（ヒット 0 件の行で別名を引き、preview と D-118 の確定の TX の取り直しが共有する。32 §15.3 手順 4 b）、BIZ-01 の preview の `old_jan_question` と別名の warning、`commit_import` の `old_jan_aliases` と手順 0b・2 e.（30 §4.8・§4.9・§4.9.2 BIZ-01-D9）、CMD の引数（42 §22.6）、UI-01c-D17。
- D-117 の Plan Review round 2 の 4 件が、正本の規則で閉じている: (1) 別名は枠と独立で枠の CHECK に関わらない、(2) 消すのは trigger だけで JAN を書く全経路を覆う、(3) 共有 JAN には聞かず、同じ取込みで別の商品が持つ JAN は別名にせず（新規の行が持つなら preview が聞かず、上書きを選んだ別の行が持つなら commit が名指しの error で戻す。30 §4.9.2 の判定表）、たどる先は PRIMARY KEY で 1 つ、(4) 答えは CMD・BIZ・UI・generated の呼出しの正本に同じ名前で書かれ、既定は空 =「ない」。
- 「ない」（既定）と別名の無い全ての行は、今の設計どおり `unmatched_product` の行エラーで知らせる（黙って誤った商品へ付けない）。
- D-122 が decision-log にあり、D-117 の Revisit から本決定を指す 1 行がある。後続の runtime の lane の範囲・順（PLU の書出しの見直しの runtime と同じ lane かその後。その runtime が clear 行を撤去するまでは「ある」と答えても前の JAN の枠がレジから消える）・Risk が backlog にある。

### 失敗定義

- 別名を PLU の枠の列・状態・照合・書出しのどれかと結びつける（枠の遷移が別名を読む・書く、問いの条件が枠の状態に依る）。
- 商品がその JAN を持った後も別名が残る経路、または消えた別名が生き返る経路が残る（JAN を書く経路のどれか、将来の経路、`products` の作り直し）。
- 共有 JAN に問いを出す、同じ取込みで別の商品が前の JAN を持つ場合に別名を作る（問いは上書きの選択に依らない条件で出し、選択に依る条件は commit が判定する）、または 1 つの前の JAN が複数の商品を指しうる。
- 答えの field が CMD・BIZ・UI・generated の呼出しのどれかの正本に無い、または名前・既定が食い違う。答えの合わない commit が答えを黙って捨てる。
- 「ない」の行、別名の無い行の扱い（`unmatched_product`）を変える。ヒット 1 件・共有 JAN の行の引き当てを変える。
- 設計照合 test・traceability の検査を red にする変更を入れる。
- 並走の lane が所有する節（下の所有表）を書き換える。
- 公開 repo に店の運用の細部・金額・商品名・JAN・部門名・発言の原文を書く。

### 非目的

- code・migration・bindings・test の変更（後続の runtime の lane）。
- 商品フォームの edit で JAN を変えられるようにすること、別名の一覧・削除の画面、売上の引き当て以外（商品検索・棚卸し・入庫）で前の JAN を引くこと（D-122 の Revisit）。
- 前の JAN の枠の書出しの規則を変えること（Q1 の a が既定）。D-117 の枠の規則・照合の表の変更。
- 共有 JAN の扱い（D-118・TD-296 の 6b）と店内コード（backlog の別の項目）の設計。
- 売れた日の JAN の履歴を持つこと（引き当ては取込みの時点の DB で行う今の規則のまま）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

設計を含む変更なので、JAN を直す日と普段の締めの操作列を置く。本 lane は docs だけで、表の通常運用は後続の runtime（JAN の別名の runtime）の後に成り立つ。**この文書を完了できる**（設計正本がそろう）ことと、**通常運用を達成できる**（店が JAN を直したときに前のバーコードの売上が同じ商品へ入る）ことは別で、後者は本 lane では未達。Z004 の確定と取消は停止 ADR が止めている間は preview の引き当てまでで、在庫に入るのはその停止を外す配布の単位（D-118 の runtime）の後。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| JAN の別名の runtime を入れた版（PLU の書出しの見直しの runtime と同じ lane かその後。backlog の依存）。商品 P は JAN A でレジの枠を持つ。メーカーが番号を B に変え、B の入荷がある。A のバーコードの在庫が棚に残る。その日の締めの取込みは済んでいる | 店主が P の JAN を B にした商品 CSV を選び、preview する | P の重複行に「スキップ / 上書き」。上書きを選ぶと同じ行に「前の番号（A）のバーコードが付いた現物は店にありますか」と `ある` / `ない`（既定 `ない`）が出る | `ある` を選んで「選択した内容で取り込む」→ 上書きの確認 | UI-01c-D17、BIZ-01-D9 |
| 同じ | 取り込む | P の JAN が B になり、A が P の別名になる（別名が成り立つかは commit の TX の中で 30 §4.9.2 の判定表が確かめる）。P は未反映になり、次の PLU の差分で B が新しい枠に書かれる。A の枠はレジに前の値段のまま残る（Q1 の既定 a） | 次の PLU の書出し | P3（レジは前の JAN の枠の登録で売り、Z004 にその JAN の行が出る） |
| 締めの SD の読取り（または Z004 の取込み） | Z004 を preview する | A の行（前のバーコードの在庫）も B の行も P に引き当たる。A の行は行エラーにならない | 確定（停止の解除の後） | 32 §15.3 手順 4 b |
| 商品 P の JAN を打ち間違えて A で登録していた（現物のバーコードは B） | 商品 CSV で P の JAN を B に直し、問いに `ない`（既定）のまま取り込む | 別名は作られない。A の枠で売れる行が出たら（A を本当に持つ別の品物がレジで A の枠の登録で売れた等）、Z004 の preview の `unmatched_product` の行エラーで知らせる | 店主が行エラーを見て確かめる | TD-284 の c の理由 |
| 色違いの商品 P1・P2 が JAN A を共有する | 商品 CSV で P1・P2 の JAN を別々の値に直す | 問いは出ない（たどる先が 1 つに決まらない）。A の行は行エラーで知らせる | — | BIZ-01-D9 の条件 2、TD-296 の 6b |
| A が P の別名。後でメーカーが A を別の商品に使い、店がその商品 Q を A で登録する | 商品 CSV で Q（JAN A）を preview する | Q の行に「JAN A は商品 P の前の番号として売上をたどっています。取り込むと、この番号の売上はこの行の商品へ入ります」の warning | 取り込む → DB の trigger が別名を消し、A の行は Q に引き当たる | master-tables §1a |
| preview（P に A の問い）の後に、別の取込みで P の JAN が A から C へもう変わっていた（同じ PC で先に別の CSV を取り込んだ等） | `ある` の答えのまま取り込む | 答えは問いで見せた A と組で届き、DB の今の JAN（C）と合わないので「商品 P の JAN が確認の後に変わりました。もう一度プレビューしてください」で何も変わらない（問いで見せていない C を別名にしない）。preview は画面に残る | ファイルを選び直して preview し直す | BIZ-01-D9 の commit の判定 |
| DB で P = A、Q = C。CSV で P → B、Q → A（同じ取込みで Q が A を使う） | preview する | P に A の問いが出る（問いは上書きの選択に依らない条件で出る）。Q を `スキップ` にして P に `ある` なら、確定で A が P の別名になる | — | BIZ-01-D9 の preview の判定 |
| 同じで、Q も `上書き` にして P に `ある` | 取り込む | 「商品 Q の行が前の番号 A を使うので、商品 P の問いは『ない』にしてください」で何も変わらない。preview・答え・選択は画面に残る | P の答えを `ない` に直すか Q を `スキップ` にして、preview をやり直さずにもう一度取り込む（`ない` なら P = B、Q = A で確定し、A の行は Q） | BIZ-01-D9 の commit の判定、UI-01c-D17 |
| その日の締めの取込みの前に JAN を直し、問いに `ない` と答えた | 締めで Z004 を preview する | 直す前に A で売れた分は行エラーで知らせる（取込みは今の DB で引き当てる。D-122 の Guarantee range の限界） | JAN を直すのは締めの取込みの後にする（上の 1 行目の初期状態） | D-122 の Guarantee range |

## Scope

本 lane は docs だけ。下の file の、下の節を書き換えた（全部 plan-first の commit）。

1. 別名の持ち方（D-122 (1)(3)）: [master-tables](../../db-design/master-tables.md) に「1a. product_jan_aliases」（役割・カラム・不変条件と 3 つの trigger・migration・設計意図）を足し、[DB_DESIGN](../../DB_DESIGN.md) のテーブル一覧と詳細の対応表に行を足した。[22](../../function-design/22-mnt-migration.md) に §18（MNT-03-D16。表と trigger の migration、`products` を作り直す migration の規則、テスト）を足した。[20](../../function-design/20-io-product-repo.md) に IO-01-D5（`find_jan_alias`・`insert_jan_alias`、`insert_product`・`update_product` は別名を扱わない）とモジュール構成の行を足した。
2. 作る経路・使う経路（D-122 (4)(5)(7)）: [30](../../function-design/30-biz-product-service.md) の §4.8 に `ImportDuplicateRow` の構造体（`old_jan_question`）と手順 5 の判定の行、§4.9 に引数 `old_jan_aliases` と `OldJanAliasAnswer` の構造体・手順 0b・手順 2 e.・エラーハンドリングの行、新しい §4.9.2（BIZ-01-D9: 問いの条件・経路・preview の判定・warning・答えの運び方・commit の判定・共有 JAN との関係・理由と却下・Revisit）を足した。[32](../../function-design/32-biz-csv-import-service.md) §15.3 手順 4 b に、ヒット 0 件の行で別名を引く段と、引き当ての規則を preview と D-118 の確定の取り直しで共有する文、更新履歴の行を足した。[42](../../function-design/42-cmd-sales-stocktake.md) §22.6 の `commit_import` の signature と手順、[60](../../function-design/60-ui-product-import.md) の UI-01c-D17・§60.0・§60.3・§60.4・§60.9・§60.10、[51](../../function-design/51-ui-product-form.md) の UI-01b-D25 と変更履歴、[SCREEN_DESIGN](../../SCREEN_DESIGN.md) の商品一括インポート画面、architecture の [biz-task-specs](../../architecture/biz-task-specs.md)（BIZ-01 の一括インポートと BIZ-03 の Stage 2）と [cmd-task-specs](../../architecture/cmd-task-specs.md)（`commit_import` の行）、[pos-tables](../../db-design/pos-tables.md) の B-1 のマスタ照合の行を、同じ規則に揃えた。
3. PLU の枠との関係（D-122 (2)）: [33](../../function-design/33-biz-plu-export-service.md) §16.6 の「同じ商品へたどる経路は TD-284 の別の小さな設計 lane で扱う」を、別名は枠と独立で §16.3 の照合・prepare・confirm は別名を読まず書かない、前の JAN の枠の値段は前の値段のまま（D-117 の受け入れ済みの限界）に置き換えた。[plu-tables](../../db-design/plu-tables.md) の「products との結合」に、別名を結合キーに入れない 1 文を足した。D-117 の枠の規則・照合の表は変えていない。
4. 後続の lane の範囲・順・Risk: [backlog](../../backlog.md) の「TD-284 の lane」の項目に本 lane で置いた設計の要約を足し、その直後に「JAN の別名の runtime」（R3、PLU の書出しの見直しの runtime と同じ lane かその後〈round 2 の是正〉、触る file と L3）を足した。
5. 決定の記録: `docs/decision-log.md` の末尾に D-122（Decision (1)〜(8)、Why、Alternatives、Guarantee range、Compatibility、owner の決定、Revisit）を足し、D-117 の Revisit の次に本決定を指す 1 行を足した。D-117 の本文は変えていない。

ついでに直したもの: 無し。

変えない正本と理由: [40](../../function-design/40-cmd-product.md)（`commit_import` の CMD は 42 §22.6 にあり、40 は商品の個別の command と共通 error の契約で、別名で変わる所が無い）、[26](../../function-design/26-io-product-csv-importer.md)（商品 CSV の列は変えない。問いは CSV の列でなく UI-01c の preview で聞く）、`docs/inventory_system_erd.html`（実装の schema の図。表が入る runtime の lane が直す。backlog の runtime の項目に書いた）、[55](../../function-design/55-ui-csv-import.md) の `unmatched_product` の表示（「商品マスタへの新規追加で次回取込み時に紐付け」は別名の後も正しい）、`FUNCTION_DESIGN.md` の UI-01c の行（signature を書いていない）、[60](../../function-design/60-ui-product-import.md) の UI-01c-D2 の行（generated の binding を使う方針の決定の記録で、3 つ目の引数は UI-01c-D17 と §60.0・§60.4 が持つ）。

型・JSON・関数の形を変えるもの（後続の runtime の lane が触る。本 lane は変えない。`rg` で全件を数えた）:

- `commit_import` の引数（BIZ と CMD）: 作る所 `src-tauri/src/biz/product_service.rs:1469`（定義）、`src-tauri/src/cmd/product_cmd.rs:244`（CMD）、`src/lib/bindings.ts:55`（generated、`cargo run --bin generate_bindings` で再生成）。呼ぶ所 `src/features/products/import/useProductImportFlow.ts:78`、`src-tauri/src/cmd/product_cmd.rs:253`。test の呼び方 `src-tauri/src/biz/product_service.rs` の test（`rg -c 'commit_import\(' src-tauri/src/biz/product_service.rs` が 30〈定義と doc comment を含む〉）、`src-tauri/src/biz/plu_export_service.rs:1922`、`src/features/products/ProductImportPage.test.tsx` の `commitImport` の mock（`:27`・`:33`・`:158` の test）。同じ名前の別物（Z004 の `src-tauri/src/biz/csv_import_service/tests/rollback_tests.rs` の局所関数 `commit_import`、`src/features/csv-import/hooks/useCsvImportFlow.ts` の `commitImport` の mutation）は対象外。
- `ImportDuplicateRow` の field `old_jan_question`: 作る所 `src-tauri/src/biz/product_service.rs:1445`（preview）、定義 `:1167`、generated `src/lib/bindings.ts:753`。読む所 `src/features/products/import/ProductImportPreview.tsx`、`src/features/products/import/useProductImportFlow.ts`。test の fixture（TS の型で必須の field になるので `old_jan_question: null` を足す）: `src/features/products/ProductImportPage.test.tsx:166`、`src/features/products/import/ProductImportPreview.test.tsx:80`、`src/features/products/import/reducer.test.ts:29`。
- 新しい表示文言（「前の番号」「現物は店に」「もう一度プレビュー」「の前の番号として」「上書きする行にだけ」）は `rg -F '<語>' src --glob '*.test.*'` でどれも 0 件（衝突する matcher は無い）。
- 新しい repo 関数・BIZ の引き当ての関数・migration・`src-tauri/src/db/migration.rs` の `expected_tables`（`:668`）: backlog の runtime の項目。

並走の lane と重なる file と節の所有（D-055 の所有表。本 lane は JAN と別名の節だけを所有する）:

| file | 並走の lane が所有する節 | 本 lane が触った所 | merge の後に揃える所 |
|---|---|---|---|
| `docs/function-design/30-biz-product-service.md` | 単位の語の直し（365）: 「単位と原価の精度の契約」の節、§4.4 の `stock_unit`、BIZ-01-D7・D8、§4.9 の手順 2 d. の文 | §4.8 の `ImportDuplicateRow` の block と手順 5 の最後の行、§4.9 の signature の 4 つ目の引数の行・手順 0b・手順 2 e.（d. の次の別の行）・エラーハンドリングの最後の行、§4.9.2 の全部 | 無し（行が分かれる） |
| `docs/function-design/20-io-product-repo.md` | 365: `ProductUpdates.stock_unit` の行 | §2.1 のモジュール構成の `product_repo.rs` の行、`find_by_jan_code` の後の「前の JAN の別名（IO-01-D5）」の節 | 無し |
| `docs/function-design/51-ui-product-form.md` | 365: UI-01b-D23・D11 の行と、その実装に伴う §の文 | UI-01b-D24 の次の UI-01b-D25 の行、変更履歴の先頭の行 | 変更履歴の行は両方を残す |
| `docs/function-design/60-ui-product-import.md` | 365: 上書きの行の単位の error の表示（触るなら） | UI-01c-D17 の行、§60.0 の signature の文、§60.3 の state と payload、§60.4 の 2 行、§60.9・§60.10 の行 | 無し |
| `docs/function-design/32-biz-csv-import-service.md` | SD 一括取込み（364）: 取込みの単位・順・失敗の扱い（§15.3 の手順 1〜3・5〜7、§15.4・§15.5、D-118 の節の取込みの流れ） | §15.3 の手順 4 b の「ヒット0件」の下の行と「引き当ての規則」の行、更新履歴の最後の行 | 364 が引き当てを呼ぶ順を変えるなら、引き当ては手順 4 b の関数を使う（本 lane の文） |
| `docs/db-design/pos-tables.md` | 364・日報の紙（363）: 日報・取込みの流れの節 | B-1 の「マスタ照合」の「ヒット0件」の行 | 無し |
| `docs/architecture/biz-task-specs.md` | 364・365: それぞれの節 | BIZ-01 の一括インポートの「前の JAN の別名」の段、BIZ-03 の Stage 2 の「ヒット0件」の行 | 無し |
| `docs/function-design/22-mnt-migration.md` | SD 直読みの runtime（367）: D-111 の migration の節（足すなら末尾） | 末尾の §18（§17・MNT-03-D15 は 367 の `settlement_no` の migration。Coordinator が 2026-10-17 に振り分けた） | 末尾は両方を残す（§17 = 367、§18 = 本 lane） |
| `docs/decision-log.md` | 363・364・365・367: 末尾の D-120・D-121・D-123・D-124 | D-117 の Revisit の次の 1 行、末尾の D-122 | 末尾は番号の順に並べる |
| `docs/backlog.md` | 各 lane の自分の項目 | 「TD-284 の lane」の項目の末尾の文と、直後の「JAN の別名の runtime」の項目 | 無し |

## Non-scope

- code・migration・bindings・test（backlog の「JAN の別名の runtime」）。
- 商品フォームの edit の JAN の変更、別名の一覧・削除の画面、売上の引き当て以外での前の JAN の引き当て（D-122 の Revisit）。
- 前の JAN の枠の書出し（Q1）、D-117 の枠の規則・照合の表、PLU の書出しの見直しの runtime の範囲。
- 共有 JAN の扱い・店内コード（D-118・backlog）、D-118 の確定・取消の節（参照だけ）。
- 並走の lane が所有する節（上の所有表）、`docs/Plans.md`（lane の行を足さない。D-097）。

## Acceptance Criteria

数は開始 HEAD `f5d7aaf4`（`git archive f5d7aaf4 docs` を `$TMPDIR/old` に展開した版）と起票後の版を同じ数え方（その file の中で式に当たる行の数、`rg -c`）で測った。出力が無いのは 0。

- AC1（表と trigger と migration）: `rg -c '^## 1a\. product_jan_aliases' docs/db-design/master-tables.md` が 1、`rg -c 'product_jan_aliases_reject_held' docs/db-design/master-tables.md` が 2、`rg -c 'MNT-03-D16' docs/function-design/22-mnt-migration.md` が 1、`rg -c 'DROP TRIGGER product_jan_aliases_reject_held' docs/function-design/22-mnt-migration.md` が 1、`rg -c 'product_jan_aliases' docs/DB_DESIGN.md` が 2（開始はどれも 0）。
- AC2（IO と引き当て）: `rg -c '^fn find_jan_alias\(' docs/function-design/20-io-product-repo.md`・`rg -c '^fn insert_jan_alias\(' docs/function-design/20-io-product-repo.md`・`rg -c 'find_jan_alias' docs/function-design/32-biz-csv-import-service.md` がどれも 1（開始 0）。`rg -c '前の JAN の別名' docs/db-design/pos-tables.md` が 1、`rg -c '前の JAN の別名' docs/architecture/biz-task-specs.md` が 2（開始 0）。round 3 の是正（引き当ての結果の形と読む経路）: `rg -cF '0 件 / 1 件（別名を含む）/ 複数（先頭と件数）' docs/function-design/32-biz-csv-import-service.md` が 1、`rg -c 'J5' docs/db-design/master-tables.md` が 1、`rg -c 'BIZ-01 の問いの判定だけ' docs/db-design/master-tables.md` の出力が空（開始はどれも 0）。
- AC3（BIZ の問いと commit。round 1 の是正、Review Response 参照）: `rg -c 'BIZ-01-D9' docs/function-design/30-biz-product-service.md` が 1、`rg -c 'old_jan_aliases' docs/function-design/30-biz-product-service.md` が 5、`rg -c 'old_jan_question' docs/function-design/30-biz-product-service.md` が 5、`rg -c 'OldJanAliasAnswer' docs/function-design/30-biz-product-service.md` が 3、`rg -cF 'P.jan_code = A' docs/function-design/30-biz-product-service.md` が 1、`rg -c '共有になりました' docs/function-design/30-biz-product-service.md` が 1、`rg -cF 'field: "old_jan_aliases"' docs/function-design/30-biz-product-service.md` が 1、`rg -c '別名になりました' docs/function-design/30-biz-product-service.md` が 1（開始はどれも 0）。`rg -c 'old_jan_alias_codes|oldJanAliasCodes|全ての重複行を上書き|問いを出したい商品の行だけ' docs/function-design/30-biz-product-service.md docs/decision-log.md docs/backlog.md` の出力が空（旧い答えの形と、撤回した限界の文が残らない。backlog は round 2 の是正で足した。round 1 の是正の後の版 `9b163cd3` の backlog は 2）。round 2 の是正（判定表を唯一の置き場にする）: `rg -c '^\| J[1-7] \|' docs/function-design/30-biz-product-service.md` が 7、`rg -c '3 つの message' docs/function-design/30-biz-product-service.md` の出力が空、`rg -c '確認の後に変わりました|共有になりました|別名になりました|取込みの後も持つ' docs/function-design/60-ui-product-import.md docs/function-design/42-cmd-sales-stocktake.md` の出力が空（message は 30 §4.9.2 の表だけ）、`rg -n '取込みの後に誰も持たない|誰も持たない商品にだけ出す|どちらにも問いは出ない' docs --glob '!docs/archive/**' --glob '!docs/plans/2026-10-17-jan-alias-design.md'` の出力が空（撤回した問いの条件が正本・Matrix に残らない。この packet は AC の式そのものを含むので外す）。round 3 の是正: `rg -n 'UI は上書きの選択に応じて出し入れしない|選択に応じて UI が問いを出し入れする|UI は選択に応じて問いを出し入れしない|の売上は行エラーで知らせる' docs --glob '!docs/archive/**' --glob '!docs/plans/2026-10-17-jan-alias-design.md'` の出力が空（撤回した 2 つの文が正本・Matrix に残らない）、`rg -c '\*\*ほかの\*\*重複行の上書きの選択' docs/function-design/30-biz-product-service.md docs/function-design/60-ui-product-import.md docs/plans/test-matrices/2026-10-17-jan-alias-design.md` がどれも 1、`rg -cF '既存の共有 / 1 件ヒットの規則' docs/function-design/30-biz-product-service.md` が 1（開始はどれも 0）。
- AC4（wire と UI。round 1 の是正、Review Response 参照）: `rg -c 'old_jan_aliases' docs/function-design/42-cmd-sales-stocktake.md` が 3、`rg -c 'old_jan_aliases' docs/architecture/cmd-task-specs.md` が 1、`rg -c 'UI-01c-D17' docs/function-design/60-ui-product-import.md` が 6、`rg -c 'oldJanAliases' docs/function-design/60-ui-product-import.md` が 6、`rg -c 'old_jan_aliases' docs/function-design/60-ui-product-import.md` が 2、`rg -c 'overwriteCodeSet' docs/function-design/60-ui-product-import.md` が 4、`rg -c 'oldJanAliasCodes' docs/function-design/60-ui-product-import.md` が 0、`rg -c 'UI-01c-D17' docs/SCREEN_DESIGN.md` が 1、`rg -c 'UI-01b-D25' docs/function-design/51-ui-product-form.md` が 2（開始はどれも 0）。P3 の是正: `rg -c 'BEFORE UPDATE' docs/db-design/master-tables.md` が 1、`rg -c 'ForeignKeyViolation' docs/function-design/20-io-product-repo.md` が 2（開始 1）、`rg -c 'D-122 からの申し送り' docs/backlog.md` が 1（開始 0）。`BEFORE UPDATE` の開始も 0。round 3 の是正（warning を 50 行の制限と別に見せる）: `rg -c '先頭 50 行の制限' docs/function-design/60-ui-product-import.md` が 1、`rg -c '51 件目' docs/function-design/60-ui-product-import.md` が 2、`rg -c '50 行の表示の制限' docs/SCREEN_DESIGN.md` が 1、`rg -c 'FM12' docs/plans/test-matrices/2026-10-17-jan-alias-design.md` が 2（開始はどれも 0）。
- AC5（PLU の枠と独立）: `rg -c 'TD-284 の別の小さな設計 lane' docs/function-design/33-biz-plu-export-service.md` が 0（開始 1）、`rg -c 'D-122' docs/function-design/33-biz-plu-export-service.md` が 1、`rg -c '別名' docs/db-design/plu-tables.md` が 1（開始 0）。D-117 の round 1 の名前を枠の側へ戻していない: `rg -l 'alias_product_code|asks_old_jan_stock|old_jan_on_hand_codes' docs --glob '!docs/archive/**' --glob '!docs/plans/**' --glob '!docs/decision-log.md'` の出力が空（開始も空）。`docs/decision-log.md` の hit は D-122 の Context の履歴の 1 行だけ（`rg -c 'alias_product_code' docs/decision-log.md` が 1、開始 0）。
- AC6（決定の記録）: `rg -c '^## D-122: ' docs/decision-log.md` が 1、`rg -c '^- Revisit への追記（2026-10-17、D-122）' docs/decision-log.md` が 1（開始 0）。D-117 の本文が変わっていない: `diff <(awk '/^## D-117/,/^- Revisit/' "$TMPDIR/old/docs/decision-log.md") <(awk '/^## D-117/,/^- Revisit/' docs/decision-log.md)` の出力が空。
- AC7（backlog の後続）: `rg -c '^- \*\*JAN の別名の runtime\*\*' docs/backlog.md` が 1（開始 0）。runtime の順（round 2 の是正）: `rg -c '「PLU の書出しの見直しの runtime」より前に入れる' docs/backlog.md` の出力が空、`rg -c '「PLU の書出しの見直しの runtime」と同じ lane か、その後に入れる' docs/backlog.md` が 1（開始 0）。
- AC8（並走の lane の節を書き換えない）: `git diff f5d7aaf4 HEAD -- <file> | grep -c '^-[^-]'`（削除の行の数）が、`docs/function-design/30-biz-product-service.md`・`51-ui-product-form.md`・`32-biz-csv-import-service.md` でどれも 0、`60-ui-product-import.md` で 6（§60.0 の signature の文・§60.3 の state の文・§60.4 の 2 行を、同じ行の後ろに D-122 の文を足す形で置き換えた分と、round 1 の是正〈Claude #4〉で §60.3 の payload の block の Set の filter と配列化の 2 行を直した分）、`20-io-product-repo.md` で 1（§2.1 のモジュール構成の行）。
- AC9（docs だけ）: `git diff --name-only f5d7aaf4 HEAD -- src src-tauri scripts package.json package-lock.json` の出力が空。
- AC10（検査）: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan` が ERROR 0・WARN 0（起票の版で実測。開始 HEAD の数は未実測）。`cd src-tauri && cargo run --bin generate_traceability -- --check` が OK。`cd src-tauri && cargo test --test design_compliance_test` が pass。

実測（Plan Review round 3 の是正、2026-10-18、commit の前の作業 tree。上の command を逐語で実行し、出力をそのまま並べた。空は「空」。AC8 は `git diff f5d7aaf4 -- <file>`〈作業 tree〉で測った。round 3 で直した行は全て本 lane が足した行なので、削除の行の数は commit の後の `f5d7aaf4 HEAD` と同じ）:

```text
AC1: 1 / 2 / 1 / 1 / 2
AC2: 1 / 1 / 1 / 1 / 2
AC2（round 3）: 0 件 / 1 件（別名を含む）/ 複数（先頭と件数） 1 / master-tables J5 1 / BIZ-01 の問いの判定だけ 空（f5d7aaf4 はどれも空）
AC3: BIZ-01-D9 1 / old_jan_aliases 5 / old_jan_question 5 / OldJanAliasAnswer 3 / P.jan_code = A 1 / 共有になりました 1 / field: "old_jan_aliases" 1 / 別名になりました 1
AC3（撤回語、30・decision-log・backlog）: 空
AC3（round 2）: ^| J[1-7] | 7 / 3 つの message 空 / 60・42 の message 空 / 撤回した問いの条件 空
AC3（round 3）: 撤回した 2 つの文 空 / **ほかの**重複行の上書きの選択 Matrix 1・60 1・30 1 / 既存の共有 / 1 件ヒットの規則 1（f5d7aaf4 はどれも空）
AC4: 42 old_jan_aliases 3 / cmd-task-specs 1 / UI-01c-D17 6 / oldJanAliases 6 / 60 old_jan_aliases 2 / overwriteCodeSet 4 / oldJanAliasCodes 空 / SCREEN_DESIGN 1 / UI-01b-D25 2
AC4（P3）: BEFORE UPDATE 1（f5d7aaf4 は 0）/ ForeignKeyViolation 2（f5d7aaf4 は 1）/ D-122 からの申し送り 1（f5d7aaf4 は 0）
AC4（round 3）: 先頭 50 行の制限 1 / 51 件目 2 / SCREEN_DESIGN 50 行の表示の制限 1 / FM12 2（f5d7aaf4 は 60・SCREEN_DESIGN とも空、Matrix は無い）
AC5: TD-284 の別の小さな設計 lane 空 / 33 D-122 1 / plu-tables 別名 1 / rg -l 空 / decision-log alias_product_code 1
AC6: ^## D-122: 1 / Revisit への追記 1 / D-117 の diff 空
AC7: JAN の別名の runtime 1 / より前に入れる 空 / 同じ lane か、その後に入れる 1
AC8: 30 0 / 51 0 / 32 0 / 60 6 / 20 1
AC9: 空
AC10: doc-consistency-check exit=0 WARN 0 ERROR 0「結果: 全チェック通過」/ --target plan exit=0 WARN 0 ERROR 0「結果: 全チェック通過」/ traceability check: OK（ERROR 0 件 / WARN 0 件）/ design_compliance_test: test result: ok. 1 passed; 0 failed
```

## Design Readiness

設計の完了条件は `docs/DEV_WORKFLOW.md` Design Phase Rules の Design completion criteria。

- 引用する設計正本（節まで）: master-tables §1a（役割・カラム・不変条件と trigger・migration・設計意図）／22 §18（MNT-03-D16、`products` を作り直す migration の規則、テスト）／20 IO-01-D5／30 §4.8（`ImportDuplicateRow`、手順 5）・§4.9（signature、手順 0b・2 e.、エラーハンドリング）・§4.9.2（BIZ-01-D9）／32 §15.3 手順 4 b／42 §22.6 `commit_import`／60 UI-01c-D17・§60.3・§60.4／51 UI-01b-D25／33 §16.6／plu-tables の「products との結合」／D-117・D-118（共有 JAN の不変条件と確定の段）・D-122。
- 必要な設計成果物（Design artifact selection の当たる行）: BIZ・IO の振舞い = updated in this PR（30・32・20）／command・DTO = updated in this PR（42 §22.6、30 §4.8・§4.9。wire の変更は後続の lane、下の Boundary / Wire Contract）／表・migration = updated in this PR（master-tables §1a、22 §18。番号と実装は後続の lane）／operator workflow・文言 = updated in this PR（60 UI-01c-D17、SCREEN_DESIGN、51 UI-01b-D25）／durable な判断 = updated in this PR（D-122、D-117 の Revisit への追記）。
- plan にしかない durable な判断の昇格先: 決めたことは全て正本と D-122 に置いた。packet にだけあるのは、規模と手法の数え、後続の lane への file の申し送り（Scope の「型・JSON・関数の形を変えるもの」）、所有表、AC の数、Q1。
- 前提・制約と、延期した design gap の follow-up: レジが前の JAN の枠の登録で売り、Z004 にその JAN の行が出ること（Contract Probe の P3。D-117 の正本の前提）。前の JAN の枠の値段は Q1（既定 a）。商品フォームの edit の JAN の変更・別名の一覧と削除・引き当て以外の前の JAN は D-122 の Revisit。延期が安全な理由: どれも今の code の振舞いを変えず、既定は今の振舞い（行エラーで知らせる）のまま。
- 絶対保証（cannot happen / always happens）の例外と escape hatch の自己点検: 「どの商品も持たない JAN だけが別名」は、商品の側の書込み（INSERT と `jan_code` の UPDATE）の trigger と別名の表の INSERT の trigger で両方向を縛る。escape hatch は (i) `products` を作り直す migration（DROP TABLE で trigger が消える）で、22 §18 の規則と最新の版での trigger の存在の test で塞ぐ（忘れると改名が失敗する側に倒れる。Contract Probe P2）、(ii) trigger を経ない書込み（`PRAGMA` で trigger を止める等）は repo の code に無く、足さない。(iii) 別名の UPDATE は DB で拒まない（repo に更新の関数を置かない。更新の経路を足すときは `BEFORE UPDATE` の拒否を足す。master-tables §1a）。(iv) 問いで見せた前の JAN と別名にする JAN の取り違えは、答えを `{product_code, old_jan}` の組で運び commit が `P.jan_code = old_jan` を照らすことで塞ぐ。「別名でたどるのは JAN を持つ商品が無い行だけ」は不変条件から従う（別名の JAN を商品が持つ状態が無い）。「答えを黙って捨てない」は手順 0b と 2 e. の 2 つの拒否で、どちらも DB を変えない（0b は TX の前、2 e. は TX 全体を戻す）。
- 判定（ready / not ready）と理由: ready（plan-draft に進める）。理由: 設計の出力が上の正本と D-122 にあり、owner の判断事項は TD-284・TD-286・TD-296 の 6b で決まり、Q1 は答えが無い間の既定（D-117 の受け入れ済みの限界）を持ち、残る外部前提（P3）は D-117 の正本の前提で本 lane の設計の分岐を変えない。

## Registration / Generation Obligations

本 lane: REQ の参照を増減していないので `docs/function-design/90-traceability.md` を再生成しない（`cargo run --bin generate_traceability -- --check` で確かめる、AC10）。function-design の file の新設・改名・削除は無い。後続の義務（backlog の runtime の項目）: Tauri command `commit_import` の引数を足した後に `cargo run --bin generate_bindings` で `src/lib/bindings.ts` を再生成する（`collect_commands` の登録と属性は既存のまま）。migration を `src-tauri/src/db/migration.rs` の registry に番号を付けて登録し、`expected_tables` に表名を足す。`docs/inventory_system_erd.html` に表を足す。

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | Which concepts belong to replaceable external adapters, and which concepts are stable app-core contracts? | Architecture, function design, decision-log, Plan Packet | 当たる。別名は app-core の引き当ての規則（BIZ-03・BIZ-01）で、レジの枠（PLU の adapter の状態）と独立にした。Z004 の形（IO-02）は変えない | D-122 (2)、master-tables §1a |
| Fact check / design decision split | Which claims are observed facts from hardware/tool/files, and which are app decisions that need source-doc promotion? | Investigation doc, source design docs, decision-log | 当たる。レジが前の JAN の枠で売り Z004 にその JAN の行が出ることは D-117 の正本の前提（実機で確かめていない、P3）。trigger の振舞いと作り直しで改名が失敗することは本 run の probe で観測（P1・P2）。問い・既定・不変条件は app の決定（owner の決定 TD-284） | Contract Probe |
| Lifecycle / retry | What happens before, during, after, and after failure for import/export, duplicate input, rollback, retry, cancellation, and re-run? | Function design, DB design, UI design, Test Matrix | 当たる。preview → commit の間の DB の変化は commit の判定で TX 全体を戻す。Z004 の取込みの取消と取り込み直しは、その時点の別名と JAN で引き当て直す。別名は商品が JAN を持つと消え戻らない | 30 §4.9.2、32 §15.3 手順 4 b、Matrix の時系列の表 |
| Operator workflow | What does the operator do in the real sequence across app, external tool, media, print/export, backup, and recovery? | Screen/UI design, function design, Plan Packet manual checks | 当たる。店主の操作は商品 CSV の上書きで JAN を直す行に 1 問増えるだけ（既定「ない」）。JAN を直すのは締めの取込みの後にする | Ordinary Operation、60 UI-01c-D17 |
| Replacement path | If the external system changes, which files/modules/docs are replaced and which app-core contracts remain stable? | Architecture, function design, decision-log | 当たる（小）。レジを替えても別名の規則（売上の行の JAN から商品へ）は残る。枠の adapter を替えても別名は影響を受けない | D-122 (2) |
| Data safety / evidence | How can the claim be supported by anonymized shape/count/hash/procedure evidence without committing real store data? | Plan Packet Data Safety, investigation doc, review evidence | 当たる。tracked に JAN・商品名・金額を書いていない（例は A・B・P・Q の記号）。probe は合成の in-memory の DB | Data Safety |
| Reporting / accounting semantics | Are totals, summaries, item records, returns, corrections, and inventory movements modeled separately enough to avoid false business meaning? | DB design, function design, report design, Test Matrix | 当たる。別名で引き当てた行はヒット 1 件と同じ形で売上・movement になり、売上の記録は商品コードを持つ（JAN を持たない）ので集計の意味は変わらない。共有 JAN の行の扱い（D-118）は変えない | 32 §15.3 手順 4 b |
| Manual verification | Which assertions cannot be proven by automated tests and require Windows native L3, external tool import, or real-device confirmation? | Plan Packet, Test Matrix, PR body | 当たる（後続）。UI-01c の問いの見た目と文言、レジで前のバーコードを売った Z004 の引き当ては後続の lane の L3。本 lane は docs だけで L3 は無い | backlog の runtime の項目、Matrix の Manual |
| 環境・再現性 | 新設の環境依存（toolchain / CI runner / OS 差異等）を repo-pinned config で強制するか、明示的に defer するか | repo-pinned config, Plan Packet | 不該当。環境依存を足さない（trigger は同梱の SQLite の既存の機能） | — |

## Boundary / Wire Contract

本 lane は wire を変えない（bindings も変えない）。後続の lane が変える wire の契約を正本に書いた。

- producer: BIZ-01 `preview_import`（`ImportDuplicateRow.old_jan_question`）、UI-01c（`commitImport` の 3 つ目の引数）
- consumer: UI-01c（重複行の問い）、CMD-01 `commit_import` → BIZ-01 `commit_import`
- wire type: `ImportDuplicateRow.old_jan_question: Option<String>`（TS `string | null`。Serialize だけ）。`commit_import(valid_rows, overwrite_codes, old_jan_aliases: Vec<OldJanAliasAnswer>)`、`OldJanAliasAnswer { product_code: String, old_jan: String }`（TS `commitImport(validRows, overwriteCodes, oldJanAliases: OldJanAliasAnswer[])`。省略できる引数にしない）。増える error は field `old_jan_aliases` の validation error の 1 種類（既存の `ValidationFailedAt` の変換）
- internal type: `product_jan_aliases(old_jan TEXT PK, product_code TEXT FK, created_at TEXT)`。IO-01-D5 の `find_jan_alias` は `Option<String>`（商品コード）
- precision/range: JAN は文字列のまま（数に変えない。先頭の 0 と 8 桁・13 桁を保つ）。別名の照合は `products.jan_code` と同じ文字列の一致で、Z004 の 13 桁の `normalized_jan` との比べ方は今の商品の照合と同じ（JAN-8 の扱いを変えない）。`old_jan_aliases` は組の一覧（数・件数の上限を足さない。同じ組の重複は 1 つとして扱う）
- round-trip path: preview の `old_jan_question` → UI の問い（既定「ない」）→ `oldJanAliases`（「`ある` ∩ 上書きを選んだ ∩ 問いが出ている」の 1 式。`old_jan` は `old_jan_question` の値のまま）→ CMD → BIZ の手順 0b → TX の中の判定（30 §4.9.2 の判定表の J3〜J7。行の前に `P.jan_code = old_jan`、全行の後に実際の選択で条件 3）→ `insert_jan_alias` → Z004 の preview の引き当て（`find_jan_alias`）→ MatchedRow の商品コード
- invalid input: 形の合わない組と commit の判定の失敗は、30 §4.9.2 の判定表（J1〜J7）の行の error（型・field・message・回復は表が持つ。J1・J2 は TX の前で DB を変えず、J3〜J7 は TX 全体を戻す）。field `old_jan_aliases` は J6 だけで、ほかは field null。どれも既存の `validation` kind（42 §22.3 の変換が field を保つ）
- compatibility: 1 つの build で古い UI は無い。商品 CSV の列は変えない。Z004 の形は変えない。`old_jan_question` は既存の field の後に足す（読む側は field 名で読む）。別名の表が無い版の DB は migration で空の表を得るだけ

## Test Plan

Test Design Matrix: [2026-10-17-jan-alias-design](test-matrices/2026-10-17-jan-alias-design.md)（後続の lane が実装する test の設計。本 lane は docs の検査だけ）。

- targeted tests: 本 lane は `bash scripts/doc-consistency-check.sh` と `--target plan`、`cargo run --bin generate_traceability -- --check`、`cargo test --test design_compliance_test`（AC10）。
- negative tests: Matrix の Negative Paths（後続）。
- compatibility checks: Matrix の Compatibility Checks（別名の無い行・ヒット 1 件・共有 JAN の引き当てが変わらない、`products` を作り直す migration の後に trigger がある）。
- data safety checks: 本 lane の差分に実データ（商品名・金額・JAN・部門名・発言の原文）を入れない。
- main wiring/integration checks: 後続の lane（UI-01c から BIZ の commit まで、Z004 の preview の引き当て、空の DB から最新の版まで）。

## Review Focus

- Ordinary Operation の列が、正常な条件で目的（メーカーの番号の変更で前のバーコードの在庫の売上が同じ商品へ入り、入力の誤りの訂正では入らない）を達成できるか。
- D-117 の round 2 の 4 件が、正本の規則で閉じているか（枠と独立・trigger・共有と同じ取込みの条件・wire の 4 か所）。同じ型の穴（別名が生き返る、たどる先が 2 つ、答えが消える）が別の経路に残っていないか。
- 不変条件の escape hatch（`products` を作り直す migration、trigger を経ない書込み）が塞がっているか。22 §18 の規則が vB・D-118 の runtime の migration に効くか。
- preview の判定（上書きの選択に依らない条件だけ）と commit の判定（実際の選択で条件 3）の分担が 1 通りに読めるか。field `old_jan_aliases` の error の後に答えを直して確定し直す経路で、別名・JAN が誤らないか。commit の判定の「前」と「後」の状態の取り方が 1 通りに読めるか。
- 引き当ての関数を preview と D-118 の確定の取り直しで共有する文が、D-118 の節（参照だけ）と矛盾しないか。
- 公開 repo に実データを書いていないか。並走の lane の節を触っていないか（AC8、所有表）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| master-tables §1a（表・不変条件・3 つの trigger） | master-tables §1a、DB_DESIGN の一覧 | runtime の lane: migration、`schema` の SQL | runtime の lane（Matrix の C1・C2） | 非対象 |
| MNT-03-D16（migration）と `products` を作り直す migration の規則 | 22 §18 | runtime の lane: migration の registry、`expected_tables` | runtime の lane（Matrix の C3） | 非対象 |
| IO-01-D5（`find_jan_alias`・`insert_jan_alias`） | 20 | runtime の lane: `product_repo.rs` | runtime の lane（Matrix の C4） | 非対象 |
| BIZ-03 の引き当て（ヒット 0 件の行で別名、関数の共有） | 32 §15.3 手順 4 b、pos-tables B-1、biz-task-specs の BIZ-03 | runtime の lane: `csv_import_service/parse.rs`（D-118 の runtime が先なら確定の取り直しも） | runtime の lane（Matrix の C5） | L3: 前のバーコードの Z004 の preview |
| BIZ-01-D9（問いの条件・preview の判定・warning） | 30 §4.8・§4.9.2、biz-task-specs の BIZ-01 | runtime の lane: `product_service.rs` の `preview_import` | runtime の lane（Matrix の C6・C7） | 非対象 |
| BIZ-01-D9（commit の判定、手順 0b・2 e.） | 30 §4.9・§4.9.2 | runtime の lane: `product_service.rs` の `commit_import` | runtime の lane（Matrix の C8） | 非対象 |
| CMD `commit_import` の引数 | 42 §22.6、cmd-task-specs | runtime の lane: `product_cmd.rs`、bindings | runtime の lane（Matrix の C9） | 非対象 |
| UI-01c-D17 | 60、SCREEN_DESIGN | runtime の lane: `src/features/products/import/` | runtime の lane（Matrix の C10） | L3: 問いの見た目と文言・既定 |
| UI-01b-D25 | 51 | 変えない（商品フォームは問いを出さない） | 既存の UI-01b の test（JAN の read-only） | 非対象 |
| PLU の枠と独立（D-122 (2)） | 33 §16.6、plu-tables の「products との結合」 | 変えない（枠の規則は D-117 のまま） | runtime の lane（Matrix の C11） | 非対象 |
| D-122、D-117 の Revisit への追記 | decision-log | 本 lane の記録 | AC6 | 非対象 |
| 隣接の除外: D-118 の共有 JAN の不変条件と確定の段、D-117 の照合の表・§16.6 の表、BIZ-01-D5・D8、商品 CSV の列（26）、`unmatched_product` の種別（pos-tables の CHECK） | 30 冒頭の D-118 の節、32 の D-118 の節、33 §16.3・§16.6、26 | 変えない（参照だけ） | 既存 | 非対象 |

## Contract Probe

本 run は実機・持ち帰りデータの新しい集計を行っていない。前提ごとに確かめた結果を書く。

- P1（SQLite の trigger で不変条件を両方向に縛れる: `AFTER INSERT` / `AFTER UPDATE OF jan_code` で別名を消し、別名の表の `BEFORE INSERT` の `RAISE(ABORT)` で商品が持つ JAN を拒み、TX を戻すと消した別名も戻る）: `$TMPDIR` の合成の in-memory の DB（Python の `sqlite3`、SQLite 3.51.3）で、商品が持つ JAN の別名の INSERT が `jan_alias_held_by_product` で拒まれ、JAN を変えた後は入り、`jan_code` を同じ値で SET しても別名が残り、別の商品の INSERT と UPDATE で消え、TX の rollback で戻り、空文字が CHECK で拒まれることを観測した。→ 観測（本 run の probe）。同梱の SQLite（3.45.0、`libsqlite3-sys` 0.28.0、22 §15 の記録）では本 run は確かめていないが、Plan Review round 1 の Codex がビルド済みの rusqlite にリンクした同梱版 3.45.0 でも拒否・消去・rollback・非復活・作り直し後の保持と 3 つの trigger の存在を確かめた（Review Response 参照）。repo は同じ rusqlite の bundled で `RAISE(ABORT, …)` の trigger を test に使っている（`src-tauri/src/biz/product_service.rs:2847`）。runtime の lane の migration の test が同梱の版で確かめる。
- P2（`products` を作り直すと trigger が消え、別名の表の側の trigger が残ると改名が失敗する）: 同じ probe で、`products_new` を作って移し `DROP TABLE products` すると `products` の 2 つの trigger が消え、`ALTER TABLE products_new RENAME TO products` が `error in trigger …: no such table: main.products` で失敗し、別名の表の側の trigger を先に消すと作り直しが通る（その後は 3 つとも無い）こと、`RENAME COLUMN` は 3 つの trigger を保つことを観測した。→ 観測（本 run の probe）。22 §18 の規則はこれに依る。
- P3（レジは JAN を直した商品の前の JAN の枠の登録で前のバーコードを売り、その売上は Z004 に前の JAN の行で出る）: D-117 の正本（33 §16.6「前の JAN の枠のバーコードでレジが売った分は、次の Z004 にその JAN の行で出る」）の前提。本 run は実機で確かめていない。→ 未確認の前提（D-117 の正本の文）。別名の設計はこの前提が崩れても誤らない（前の JAN の行が出なければ、別名は使われないだけ）。
- P4（店で JAN を直す場面の頻度とどちらの場面が多いか）: 台帳と open-questions に記録が無い（2026-10-17 に `rg` で確かめた）。→ 未確認。設計の分岐を変えない（問いは場面を問わず 1 回）。

## Data Safety

- tracked に書かない: JAN・商品名・金額・部門名、店主・owner の発言の原文、repo 外の台帳の本文（ID で引いた）。例は記号（A・B・P・Q）だけ。
- local-only: repo 外の回答台帳、review の報告（`.local/reports/`）。
- synthetic-only: probe は in-memory の合成の DB。後続の lane の test は合成の DB と合成の商品だけを使う。

## Implementation Results

- 結果: owner の決定（repo 外の回答台帳 TD-284 の c、TD-286 の β、TD-296 の 6b）を、前の JAN を商品の側の表 `product_jan_aliases` と 3 つの trigger に置き、PLU の枠と独立にする形で設計正本に置き、D-122 に記録した。商品 CSV の上書きで JAN を直す行に前の番号の現物の有無を 1 回聞き（既定「ない」）、答えは `commit_import` の `old_jan_aliases`（`{product_code, old_jan}` の組）で運び、commit の判定・error・回復は 30 §4.9.2 の判定表（J1〜J7）の 1 か所に置いた。Z004 の引き当てはヒット 0 件の行で別名を引き、preview と D-118 の確定の取り直しで同じ関数を使う。`products` を作り直す migration の規則は 22 §18 に置いた。D-117 の本文は変えず、Revisit に本決定を指す 1 行を足した。本 lane は docs だけで、code・migration・bindings は変えていない（runtime は後続の lane）。
- 既存の保護: 実装は変えていないので、店の動作は今のまま（前の JAN の売上は `unmatched_product` の行エラーで知らせる）。設計照合 test と traceability の検査は green のまま。
- 未検証: Contract Probe の P3（レジが前の JAN の枠の登録で売り、Z004 にその JAN の行が出る）と、UI-01c の問いの見た目と文言は、runtime の lane の L3。owner への Q1（前の JAN の枠の値段）は既定 a のまま。
- review・CI・merge（closeout、2026-10-18）: Plan Review は round 3（天井）までで、round 3 の残りは owner の決定（TD-317）で直して plan-approved。Final Review は broad 1 回で是正なし（下の Review Response の Closeout の段落）。helper の record は review = pass。hosted の `Merge gate` は Ready の後の run で success で、owner の Ready・merge の承認の後に helper 経由の squash merge。PR: [#174](https://github.com/higekiri-w/inventory-system-desktop/pull/174)
- 後続: `docs/backlog.md` の「JAN の別名の runtime」（Final Review の P3 4 件の申し送りを含む。「PLU の書出しの見直しの runtime」と同じ lane か、その後）。

## Review Response

### Plan Review round 1（2026-10-17、head `7f331023`）

Claude 側 fresh Opus 5.5（reject、P1 0・P2 1・P3 3）と Codex GPT-6.1 Sol（reject、P1 1・P2 1）。Claude #1 と Codex #1 は同じ件で、重い方の P1 とした。全て採った（Coordinator の裁定。Codex #2 は下の経緯）。是正の前に相談役（Fable 5.1）に反例探しを頼んだ。起草役が `f50d0bfd` で直した。

| # | 出所 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|
| R1 | Codex #1（P1）・Claude #1（P2） | 採る | 答えが商品コードだけで、commit は前の JAN を DB の今の値から取り直す。preview の後に別の取込みで JAN が A → C に変わると、問いで見せていない C を別名にする | 答えを `old_jan_aliases`（`{product_code, old_jan}`、`old_jan` は問いで見せた値をそのまま返す）で運び、commit は行を処理する前に `P.jan_code = old_jan` を確かめ、違えば原因を名指しする message で TX 全体を戻す。30・42・60・cmd-task-specs・Matrix を揃えた |
| R2 | Codex #2（P2） | 採る（是正の形は owner の決定 TD-309） | 全ての重複行を上書きすると見なす preview の判定が、スキップした行まで理由にして問いを消す | preview の問いは上書きの選択に依らない条件だけで出す（広めに出る）。上書きを選んだ別の行が前の JAN を持つ場合は、commit が実際の選択で判定し、field `old_jan_aliases` の名指しの error で戻す。利用者は preview・答え・選択を保ったまま、答えを「ない」に直すかその行をスキップして確定し直せる |
| R3 | Claude #2（P3） | 採る | 別名の UPDATE は trigger を経ない | master-tables §1a に、別名の UPDATE は DB では拒まず、更新の経路を足すときは `BEFORE UPDATE` の拒否の trigger を同じ migration の規則で足す、と書いた |
| R4 | Claude #3（P3） | 採る | DbError の対応が現物の変換と違う | 既存の `From<rusqlite::Error>` のまま（外部キー違反 → `ForeignKeyViolation`、PRIMARY KEY 重複 → `DuplicateKey`）に直した |
| R5 | Claude #4（P3） | 採る | UI の擬似コードが配列に `.has` を呼ぶ | Set で filter する形に直した |

Codex #2 の経緯: Coordinator は当初「限界として書く」（業務規則を UI に置かない）と裁定したが、owner が「システム側でサポートしないと無理筋」と否定し（repo 外の回答台帳 TD-309、2026-10-17）、上の R2 の形に差し替えた。

あわせて owner の設計の原則（repo 外の回答台帳 TD-310）で、preview の後に別の取込みでできた別名を commit が名指しして戻す形（warning を見せずに別名を消さない）を足した。

起草役が TD-310 に照らして見つけ、本 lane の範囲を越える次の件（owner に諮る候補。本 round の review が通った後に他の packet の問いとまとめて聞く。答えが無い間は D-122 の Guarantee range の受け入れ済みの限界のまま）:

- 「ない」と答えた後の前の JAN の売上が、後の Z004 の preview で初めて行エラーになる。
- 締めの前に JAN を直すと、その日の売上が行エラーになる（今は操作の順に頼る）。
- 「ない」と答えた後に「ある」にする手段が無い。
- 商品フォームの新規作成で、別名の JAN が警告なしに消える。

### Plan Review round 2（2026-10-18、head `9b163cd3`）

Claude 側 fresh Opus 5.5（reject、P1 0・P2 3・P3 3）と Codex GPT-6 Astra（reject、P1 0・P2 2・P3 1）。Claude #2 と Codex #2、Claude #4 と Codex #1 は同じ件。全て採った（Coordinator の裁定）。起草役が `dd2df899` で直した。

| # | 出所 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|
| R6 | Codex #1（P2）・Claude #4（P3） | 採る | 手順 2 e. とエラーハンドリングが一律の型と件数を書き、§4.9.2 の表と食い違う（競合は field つき）。後からできた別名の検査が手順と表に無い | §4.9.2 の commit の判定を J1〜J7 の表（時点・条件・error の型と field と message・回復）にし、§4.9・42 §22.6・60 UI-01c-D17 は表を引くだけにした。後からできた別名の検査を J5 に移した |
| R7 | Claude #2・Codex #2（P2） | 採る | 撤回した前の問いの条件が packet・biz-task-specs・D-122・Matrix T5 に残る | 問いの条件（選択に依らない）と成立の条件（commit が実際の選択で判定）を分けて書き、T5 の期待を「両方に問いが出る」に直した。同じ型の文を全文書で洗った |
| R8 | Claude #1（P2） | 採る | backlog の後続の項目に撤回した答えの形が残る | `old_jan_aliases` / `oldJanAliases` に直し、AC3 の検索の対象に backlog を足した |
| R9 | Claude #3（P2） | 採る（runtime の順を直す） | 別名の runtime を PLU の書出しの見直しの runtime より前に入れると、その間は JAN を直すと前の枠が消え、問いの約束が果たされない | owner の決定（repo 外の回答台帳 TD-284・TD-286）の「より前」は設計の lane の順の話と読み（起草役が原文を読み、Coordinator も同じ読み）、backlog の別名の runtime を PLU の書出しの見直しの runtime と同じ lane かその後にし、依存を書いた |
| R10 | Claude #5（P3） | 採る | warning の照らし方が一意でない | J5 を「今の別名の商品で作った warning の文と一致する要素がある」にした |
| R11 | Claude #6（P3） | 採る | Mutation の節が表に無い test 名を指す | 表の名前に揃えた |
| R12 | Codex #3（P3） | 採る | AC4 の baseline が現物と合わない | base で測り直して直した |

### Plan Review round 3（2026-10-18、head `a8f2ebdf`）

Claude 側 fresh Opus 5.5（approve、P1 0・P2 0・P3 3）と Codex GPT-6.1 Sol（reject、P1 0・P2 2・P3 1）。Codex #2 と Claude #1 は同じ件で、重い方の P2 とした。全て採った（Coordinator の裁定）。round の天井で P2 が残ったので owner に諮り、「直して、追加の round なしで plan-approved（跡は Final Review）」と決まった（repo 外の回答台帳 TD-317）。起草役が `8f050ee6` で直した。

| # | 出所 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|
| R13 | Codex #1（P2） | 採る | 別名の warning を見せた証拠（J5）にしているが、preview の一覧の表示の制限の外の行の warning は見えないまま commit に届く | UI-01c-D17 に、warning を持つ行を表示の制限と別に一覧の上へ全件まとめて出す契約を置き、Matrix に制限の外の行の表示の検査を足した |
| R14 | Codex #2（P2）・Claude #1（P3） | 採る | 「その行で上書きを選んだときだけ問いを出す」と「上書きの選択に応じて出し入れしない」が食い違う | 後者を「ほかの重複行の上書きの選択から判定し直さない」に限り、60・30・Matrix を揃えた |
| R15 | Codex #3（P3） | 採る | J4 の回復の説明が引き当ての規則と違う | 「問いは出なくなり、売上は 32 の既存の共有と単独のヒットの規則に従う」にした |
| R16 | Claude #2（P3） | 採る | 引き当ての関数の返す形が決まっていない | ヒットなし・単独（別名を含む）・複数（先頭と件数）を区別して返すと書いた |
| R17 | Claude #3（P3） | 採る | 読む経路の一覧に warning と J5 が無い | 足した。あわせて、問いの判定は別名の表を読まないので一覧から外した（起草役が発注を越えて直し、Coordinator が問いの条件と J3・J4・J6・J7 が別名を読まないことを確かめて採った） |

Closeout（2026-10-18）: Final Review は broad 1 回で、Claude 側（Fable 5.1）= approve（P3 3）、Codex（Astra）= approve（P3 1）。是正は無し。helper の record は review = pass。hosted の `Merge gate` は Ready の後の run で success で、owner の Ready・merge の承認の後に helper 経由で squash merge した。P3 4 件は全件 follow-up（Coordinator の裁定）で、`docs/backlog.md` の「JAN の別名の runtime」へ申し送った: Claude 側 1（22 §16 の `products` の作り直しの手順〈vU の手順 5・vB の手順 4・「作り直しを 1 回にまとめる順」〉に §18 の trigger の規則への参照が無い）、Claude 側 2（§18 の作り直しの規則が `PRAGMA foreign_keys = OFF` を前提にしていると書いていない）、Claude 側 3（Matrix の Main Wiring が `find_by_jan_code` の Z004 側の呼出しを `parse.rs` だけ挙げ、`time_evidence.rs` の test 専用の呼出し〈D-118 の runtime (b) で外す〉を除外と書いていない）、Codex 1（Matrix の wire type の行に旧形の `string[]` が残る。正しくは `old_jan_question: string | null` と `oldJanAliases: OldJanAliasAnswer[]`）。packet の本文で直す所は無い。Owner Effort Budget の表は closeout の時点の値（上限 9、消費 3）に直した。
