# Plan Packet: デザインの決まりを「見る人の受け取り方」から組み直す（design-first、docs、R2）

2026-09-24 起草。起源は owner と Coordinator が 2026-09-24 に進めたデザインの見直し（owner と作った非公開の canvas 2 枚: 棚卸し画面の再設計案と、デザインの決まりの見直し案）。owner の判断の要旨と日付は本 packet の「設計判断の出典」に置き、発言の原文は公開 repository に置かない（owner 決定 2026-09-21 の方針に従う）。

本 lane は design-first の文書改訂だけを扱う。`src/styles/globals.css` の token の値の変更・全画面への適用・書体の同梱・棚卸し画面 D1 の実装は、本 packet の [Non-scope](#non-scope) で切り分けた後続 lane が持つ。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

- Evidence Mode: github
- Phase: archive
- Risk: R2
- Execution Mode: fable-window
- Plan Commit: ba6532e0acc3d9fe15276519ccd56ee5d8cd3f21
- Amendments: cbcbabc80fbacb7055b418100878044ac0e47cf9
- Coordinator: Opus 5.5（Claude Code main session）
- Writer: Opus 5.5 subagent（worktree run）
- Plan Reviewer: Opus 5.5（fork でない fresh subagent、Writer と別 context）
- Final Reviewer: Opus 5.5 fresh subagent + Codex（互いに独立、Plan Reviewer とも別の fresh context）
- Final Review Minimum: 2
- Human Gate: ready,merge

manual なし: 製品 runtime・画面・配布物は変わらない文書改訂で、Windows native で確かめる対象がない。画面の見た目の確認（owner の目・実機 L3）は後続の runtime lane の Human Gate に置く。

座組の根拠と現行規則との衝突（明記）:

- 座組は owner 決定 2026-09-23（Opus = Opus 5.5 を Coordinator / Writer / review に全面解禁）と 2026-09-24（差し戻し時は相談役として Fable 5.1 を入れる）による。
- 現行の tracked 規則とは衝突する。`docs/AGENT_OPERATING_MANUAL.md` §3 の高自律・低制約適性 slot の項（D-056）は Opus を read-only の Reviewer / Explorer 専任とし、§3.4 の表は Opus を Claude Opus 5 とする。この衝突を解く規則改訂は別 lane `agent/harness-overhaul` が持ち、本 packet は owner 決定を根拠に先行して座組を使う（並走 lane `agent/docs-restore-and-rules` の packet と同じ扱い）。
- Execution Mode は現行 enum から選ぶ。Fable 5.1 が相談役として利用可能なため §3.2 の定義上 `fable-window` とする。
- Plan Reviewer と Writer は同じ vendor・同じ model。`docs/DEV_WORKFLOW.md` Review Rules の vendor 条項（D-062）は Writer が Codex の packet に掛かるもので、本 packet には literal に掛からない。同 vendor の Plan Review で packet の前提誤りが通過する残余（PR #88 の dogfood 所見）は、Final Review の Codex が別 vendor の目で受ける。
- Final Review Minimum は R2 なら 1 を選べ、`scripts/ci/classify-changes.sh` は本 lane の予定 file をすべて `docs=true`・`workflow=false` に分類する（`docs/design-system/*` と `docs/quality/*` は policy docs の列挙に含まれない）ため helper も 2 を要求しない。owner 決定の Final Review 座組（fresh Opus + Codex）に合わせて 2 とする。Codex を待たない判断をするなら Plan Gate 前に 1 へ下げる（Plan Gate 後は helper が変更を拒む）。

遷移記録（append-only）:

- kickoff → spec-check → design → plan-draft（2026-09-24、Coordinator 起草）: 設計判断は owner と canvas で 2026-09-24 に確定した（「設計判断の出典」）。本 lane は design-only scope で、その判断を source docs へ反映すること自体が実装にあたり、[Spec Contract](#spec-contract) が反映内容を固定する。未解決の設計質問は書体の最終採用だけで、runtime lane の実機比較へ明示的に defer した（SPEC-DSR-RENEW-D3）。
- plan-draft → plan-gate（2026-09-24、Coordinator）: packet と Test Design Matrix を plan-first commit で確定し、`docs/Plans.md` の `## 次の行動` に登録。fresh Opus の Plan Review へ。
- plan-gate → plan-approved → implementing（2026-09-24、Coordinator、state-only）: Plan Review round 1（fresh Opus、P1 0 / P2 7 / P3 7）→ 差し戻しのため相談役 Fable 5.1 に是正方針を検討させた（src に当てた 12 場面・移行中の混在 3 経路）→ 是正 `fe43d11b` → round 2（別の fresh Opus、P1 0 / P2 2 / P3 7、owner 回答: 検索欄はボタンなしで確定、lane A は 1 PR）→ 是正 `0026386e` → round 3（別の fresh Opus、天井、P1 0 / P2 0 / P3 10）→ Plan Gate 前に直す P3 を `ba6532e0` で反映。Plan Commit = `ba6532e0`。round 天井 3 に到達し通過。実装は Opus 5.5 subagent の worktree run。
- Gated Amendment 1（2026-09-25、Coordinator、Final Review broad〈Opus `5ae34b7d` P2 2 / P3 5、Codex `5ae34b7d` P2 3 / P3 3〉、owner 判断 F2、相談役 Fable の検討）: Phase は implementing のまま（design への backtrack はしない。13 件はどれも規則の答えが 0 か 2 つになる箇所の書き足し・限定・記録漏れで、owner の設計判断と Scope の file の集合は変わらない）。相談役 Fable 5.1 が 13 件を 5 つの根（A 役割色の文字だけの表示、B 移行の列の読み分け、C 1 つの要素に 2 つの役割、D 検査の検出力、E 統合時の記録漏れ）に束ね、Coordinator が全件採用した。契約の変更は Goal・Ordinary Operation の行 2・Scope（S1〜S4・S6・S7・S9・S10）・処分表 U14・D1・D2・D5・D7・D8・D10〜D12・D12 の表（19 → 23 場面）・AC13・AC14・Matrix T13・T15 と変異の見込み・Adjacent Pattern Audit で、裁定と Writer の反映先は [Review Response](#review-response) の「Final Review broad round 1」に置く。見た目に触れる 3 点（取込みの手順表示のいまのステップ = 進行中の段 2、詳細を開いた行の 3 点目は開いた詳細と「詳細を閉じる」の類の文言が担い badge を足さない、badge の weight は現行 500 で旧 600 を置換）は Fable の推す既定を試しとして置き、runtime lane A の L3 で owner に見せて確かめる。この後は Writer（Opus 5.5 subagent の worktree run）が正本へ反映し、Final Review broad を 2 本（fresh Opus と Codex、互いに独立）取り直して閉じる。closure だけでは閉じない: `scripts/pr-gate.py` は broad の `plan_commit` と `amendments` が packet の現在値と一致しなければ `broad Plan contract changed; fresh broad required` で拒み（`docs/agent-guidance/merge-evidence.md` の「Plan契約が変わった場合…は、新しいbroadを必要とする」）、`5ae34b7d` の 2 本は `amendments` が空のため GA1 の後は使えない。Final Review Minimum 2 は変えないため、取り直す broad も 2 本が要る。Opus 側は GA1 の差分を中心に見てよいが、helper の上では broad として記録する。
- implementing → archive（closeout、本 commit）: PR #98 squash merge `0aae02da`（2026-09-25）。packet / Matrix を `docs/archive/plans/` へ移送し、Implementation Results / Review Response を記録した。Final Review closure（Fable 5.1、`f60cee69`）の新規 P3 3 件と、1 回目の base 同期 `d5c4709d` に当てた closure の P3-3 を処理した: C1（icon 必須の「Alert」に「非中立の」が掛からない）と C3（`PluExportPage` の `info` 家族）は `docs/backlog.md` の runtime lane A の対象へ 1 句ずつ足し、C2（Spec Contract D8・D11 と是正 `68c152cf` の食い違い）と P3-3（backlog の行番号）は Review Response の結論の段落に注記した。

## Owner Effort Budget

- 介入回数上限: 8（消費 5: 起票時 2〈ランキング 1 位を試しとして了承した 1 回、検索欄と lane A の 1 PR を 1 回の質問でまとめて答えた 1 回〉、Final Review の F2 の判断 1、上限の引き上げの承認 1、数え直しの確認 1。ほかに Codex の Final Review relay 1〈発注書 81、消費済み〉、Ready 1 と merge 1〈owner 2026-09-25「Ready から締めまで」で承認済み〉）
- 上限の変更（Plan Review round 3 の Coordinator 裁定、2026-09-24）: 起草時の上限 3 から 5 へ上げた。理由 = design-first の lane で、owner にしか決められない見た目の判断（試しの了承・確定）が要り、消費済み 2 と見込み 3 の合計が起草時の上限 3 を超えるため。
- 上限の変更（Gated Amendment 1、owner 承認 2026-09-25）: 5 から 8 へ上げた。owner 2026-09-25 が引き上げと、上の数え方での記録を承認した。GA1 の後の Codex broad の取り直しは relay 1 と介入 1 を使い、上の 8 と relay 往復上限 2 に対して介入は 9 回目、relay は 2 回目になる。取り直しの発注の前に、Coordinator が介入の上限の扱いを owner に確かめる。
- 実働時間上限: 20分（文書だけの変更で、owner の作業は relay と Ready / merge の判断。規則の文面が canvas の判断どおりかは Ready 判断の中で見る）
- relay 往復上限: 2
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R2

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers と `docs/project-profile.md` High-risk Changes を当てた。変更は design-system の文書（色の役割・強調の段階・書体・ラベルと値・原則・DSR の並び）と、その参照元の文書の同期だけで、runtime 契約・DB・CSV/TSV・command・route/search・画面の振舞い・画面の文言を変えない。画面の見た目を変える runtime lane は別 packet（R3、manual 付き）とする。

gate への影響を確かめた（Risk は file の種類でなく gate と利用者への影響で決める）:

- 本 lane の予定 file を `scripts/ci/classify-changes.sh` に渡すと `docs=true`・`workflow=false`・`frontend=false` になる（同 script の 59 行目の policy docs 列挙に `docs/design-system/*`・`docs/quality/*`・`docs/UI_TECH_STACK.md`・`docs/decision-log.md`・`docs/backlog.md` は含まれない）。
- merge gate の合否に効く唯一の経路は `scripts/doc-consistency-check.sh` の DS1〜DS4 で、checker は `docs/design-system/00-foundations.md` / `01-decision-rules.md` / `02-component-catalog.md` の file 名と `## DSR-NN` 見出しを固定で読む。file を改名・分割すると DS2 / DS3 は「見つかりません。スキップ」の INFO で素通りし、検査が黙って弱まる。本 packet はこれを防ぐため file 名・見出し形式・token 表の書式を [Spec Contract](#spec-contract) の SPEC-DSR-RENEW-D9 で固定し、AC で DS2 / DS3 の実行結果（件数）を確かめる。gate の定義（script）は変えない。
- 先例: design-system の文書だけを改訂した lane は R2（`docs/archive/plans/2026-09-05-ui-conventions-batch-design.md`、`2026-08-20-design-backbone-reference.md`、`2026-07-04-design-system-drift-sync.md`）。runtime へ反映する lane は R3（`2026-09-08-ui-conventions-runtime.md`）。
- 規約文書は後続の実装とレビューの判断基準になるため R1 ではない。Test Design Matrix は R2 で任意だが、複数 file にまたがる番号の付け替え・参照の同期・「置かないもの」の検査を機械で確かめるため必須として付ける。

## Goal

Goal Invariant:

### 最小完了条件

- 新しい画面や修正を設計する人が、改訂後の design-system 文書だけを読んで、色（どの役割か）・強調（何段目か）・書体（いまは何を使い、何が採用前か）・補足情報の見せ方（ラベルと値の組）を迷わず選べる。選んだ理由を「見せ方 → 見る人の受け取り方 → 狙う効果 → 根拠」の 4 列で説明できる。
- 原則が 1 か所にまとまり、DSR-01〜24 は番号を変えずに話題から引ける。README が「どこを読めばよいか」の索引になっている。
- 現行の実装（amber の primary 等）と改訂後の決まりが食い違う箇所は、文書の上で「runtime lane 待ち」と明示され、読む人が現行の見た目と狙いの見た目を取り違えない。
- runtime lane A の merge 前に画面を作る人が、「現行の token と 02 の部品の形で作る」か「lane A の merge を待つ」かを 1 つに決められる（[D8](#spec-contract)）。
- 今の色の割り当て（在庫切れ・取消済み・マイナスの増減・取得失敗の赤、tone family の注意と完了）は変わらない。変えるもの（owner が試しとして了承したランキング 1 位と詳細を開いた行）と変える案（[D12](#spec-contract) の他の試しの行）は、runtime lane A の L3 で owner が実画面を見て決める試しとして書かれる。owner の現行の決定がある試しの 4 行は、00 の役割の列に owner の現行の決定を書き、試しの答えを移行の列に置く（D8）。owner の決定が無い取込みの手順表示は、役割の列に Coordinator の既定を書き、試しであることを移行の列に書く（D8）。
- `bash scripts/doc-consistency-check.sh` の DS1〜DS4 が改訂前と同じ強さで働き続ける（file 名・DSR 見出し・token 表の突合件数が減らない）。

### 失敗定義

- 改訂後の文書だけでは、上の選択のどれかで答えが 2 つ以上になる、または答えが無い（例: 「進行中」を何色で示すかが読めない、書体が決まったのか採用前なのか読めない）。
- canvas の候補値（未実装の HEX）が canonical docs（`docs/design-system/*.md`・`docs/quality/review-checklist.md`）に載り、DS3 の検査対象外の書式で未実装値が正本に紛れ込む（2026-09-03 の Codex P2-2 と同型）。
- file 改名・見出しの書式変更・token 表の行の削除で、DS2 / DS3 の検査対象が黙って減る。
- 旧原則番号を引いている文書が、番号の付け替えで別の原則を指したまま残る。
- owner の判断（1 色 1 役割、操作と進行中は同じ緑寄り、黒の 2px 枠と長いバッジは避ける、書体は Noto Sans JP が好みで最終は実機、ラベルと値の組）と食い違う規則を書く。
- 今の色の割り当て（上の最小完了条件）を、owner の L3 を経ずに変える規則を書く。または、撤回・置換した旧文（[D13](#spec-contract)）が改訂後の文書に現行の規則として残る。
- 発言の原文・canvas の URL・`.local/` の path・店の実データを tracked file に書く。

### 非目的

- 画面の見た目を変えること（token の値・部品・画面は runtime lane）。
- 書体を決め切ること（実機比較の後に runtime lane が決める）。
- 02 部品カタログを新しい token へ書き換えること（token が実装されたときに runtime lane が同期する）。
- DSR の新設・廃止・番号の付け替え、DSR 本文の全面的な書き直し（矛盾の解消に必要な改訂だけ）。
- DSR 24 本それぞれへの 4 列の付与（4 列は 00 土台の token と規則に持たせる。DSR は既存の Why が受け取り方を担う）。
- doc-consistency-check の検査の追加・変更。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

この文書を完了できること（本 lane）と、店主が画面を見て狙いどおりに受け取れること（通常運用の目的）は別である。後者は runtime lane の token 反映と実機 L3 まで達成しない。下表は本 lane の完了後に、設計する人（Coordinator・Writer・reviewer）が改訂後の決まりだけで普通の作業を進める列である。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 新しい画面を設計し始める。改訂後の design-system だけを開く | README の索引から読む順（00 土台 → 04 原則 → 01 判断ルール → 02 部品 → 03 出典）と「迷ったらここ」を引く | どの問いをどの節で解くかが 1 か所で分かる | 索引の全行が実在の節へ届く | なし（AC4 で索引の行と節の対応を確かめる） |
| 押すボタン・作業中の行・確かめてほしい行・済んだ知らせ・在庫切れの行・待ちの表示を置く | 要素（領域・行・badge・ボタン・部品の状態）ごとに、00 の色の役割表の「使う場面」列で役割を 1 つ選び、強調の段階表で段を選ぶ。部品の状態（focus ring・checked・進み具合の棒・保存中のボタン・減衰）と、役割色の文字だけの表示（入力欄のエラー文・増減の ±・セルの数字・リンク）と、現行と狙いが分かれる場面は 00 の「迷いやすい場面」表で引く | 各要素の役割（操作 / 進行中 / 注意・確認 / 完了 / 危険・失敗 / ふつう・補足 のうち 1 つ）と段（0〜4、または「段に数えない」）、1 画面あたりの上限、受け取り方と狙う効果と根拠が 1 つに決まる。行の役割と、その行の中の badge の役割は別の要素として選ぶ（例: 入力中の行 = 進行中、その行の在庫切れの badge = 危険・失敗） | 同じ要素に 2 つの役割・段が当たらない。「迷いやすい場面」表の各行で役割と段が 1 つに決まる（行の中で条件ごとに 1 つを書く場合を含む、AC14） | 進行中の token は runtime lane A まで未実装（D8）。場面の答えは 2026-09-24 の src 実測に基づく（Contract Probe）。詳細を開いた行を進行中にするのは owner 2026-09-25 の試し、取込みの手順表示のいまのステップを進行中にするのは Coordinator の既定の試しで、どちらも lane A の L3 で owner が見る（GA1） |
| lane A の merge 前に、新しい画面や修正を作る | README の「移行中の作り方」と、00 の役割表の「現行の実装」「移行」列を読む | 現行の token と 02 の部品の形で作ると分かる（操作 = `--primary`、現在行 = DSR-22 の 3 点表示で canonical は取引先選択 dialog の現在行）。進行中の地・作業中の囲みを新しく要する画面は lane A の merge 後に作ると分かる | 「現行の部品の形で作る」か「lane A を待つ」かの 1 つに決まる。新しい見た目の先取り（候補色の直書き・新 token 名の先取り）をしない（review-checklist カテゴリ 9） | 候補値（緑寄りの色）は canonical docs に無く、D-091 にだけある |
| 補足情報（商品名の下の部門・コード・前回数えた数・数えた日）か、サマリカードの主値を置く | 00 のラベルと値の節を当てる | 補足情報はラベル（小さく薄く）と値（ふつうの濃さ）の組、サマリカードの主値はラベル（小さく薄く）と値（metric 30px）の組になる。どちらも何の値かをラベルで言い切る | 日付・数量のそれぞれに、何の日付・何の数かのラベルが付く | なし |
| 書体を選ぶ | 00 の書体の節を読む | いまは現行の system font stack を使うこと、候補 2 つ（Noto Sans JP / BIZ UDPゴシック）と選定条件、最終採用は runtime lane B の実機比較で決まることが分かる | 「決まっていない」ことが明示され、勝手に書体を差し替えない | BIZ UDPゴシックが Windows 標準に含まれるという外部の事実と、店の PC に入っているか、Noto Sans JP の同梱サイズ（いずれも 未実測）は runtime lane B で確かめる |
| 既存の DSR を探す | 01 の話題別の索引（配置と一覧 / 入力 / 知らせ方 / 移動と戻り / 色と状態）から DSR 番号を引く | 旧番号のまま該当の DSR に届く | 24 本すべてが索引に 1 回ずつ載る | なし |
| 古い文書やコードのコメントに「04-backbone 原則 15」等の旧番号を見つける | 04 の旧番号対応表を引く | 新しい原則番号へ辿り着ける。2 つの規則を持っていた旧 15 は規則ごとに行き先が分かれ（現在行の 3 点 → 新 10、ラベル小・値大 → 新 5 と 00 のラベルと値）、参照している文の主題で 1 つに決まる。撤回・置換した旧文は対応表の「撤回・置換」列で置換先が分かる | 旧 1〜16 の全行が対応表にあり、2 つに分かれる行は規則ごとに行き先を持つ | なし |

## Scope

Writer は下の file だけを編集する。対象を使う呼出し側（link・番号参照・checker）は起草時に `rg` で洗い出し、[Test Design Matrix](test-matrices/2026-09-24-design-rules-renewal.md) の Adjacent Pattern Audit に列挙した。

- S1 `docs/design-system/00-foundations.md`（土台）: 色の役割表（SPEC-DSR-RENEW-D1）、強調の段階（D2）、迷いやすい場面の表（D12）、書体の節の改訂（D3）、ラベルと値の組と進み具合（D4）を新設・改訂する。新しい規則の節は「見せ方 / 受け取り方 / 狙う効果 / 根拠」の 4 列の表を持つ。既存のカラーパレット表・セマンティックカラー表は行も HEX も変えず（DS3 の突合対象）、各行が役割表のどの役割に属するかを示す列を足す。撤回・置換表（D13）の 00 の行（セマンティックカラー表の Primary と Warning の用途、4 色エリアモデルの節のサイドバーの文、ウォーム系の論拠の節）を直す（ニュートラルの stone はウォームのまま、役割色は色相で役割を分ける）。04 から反映待ちだった 3 点（caption・ページ余白・icon）を src の実測に合わせて直す（D14）。metric 行の「原則 15（04-backbone）」は新しい行き先（新 5 と本書のラベルと値の節）へ直す。
- S2 `docs/design-system/04-backbone.md`（原則）: 03 の哲学と 04 の 16 原則を 10 本前後の原則に統合する（D5）。各原則は 1 文 + 受け取り方と効果 + 根拠（出典は 03）で書く。旧番号対応表（旧 04 原則 1〜16 → 新番号。2 つの規則を持つ旧 15 は規則ごとに 2 つの行き先。旧 03 の各節 → 新原則または 03 の出典。撤回・置換した旧文は D13 の行を「撤回・置換」列に書く）を置く。役目を終えた節（「foundations への追記分（token）」「00〜03 への反映先」「適用の順序」）は削る。削る前に、そこに書かれた未反映の約束を下の「[旧 04 の未反映の約束の処分](#旧-04-の未反映の約束の処分)」表のとおりに処分し、処分を 04 の旧番号対応表（撤回は「撤回・置換」列）か更新履歴に書く。削った旧文は `git show dda8560a:docs/design-system/04-backbone.md` で引けると更新履歴に 1 行書く。file 名は変えない。
- S3 `docs/design-system/03-philosophy.md`（根拠の出典）: 原則を持たない「根拠の出典」へ改める（何を取り、何を取らないかは維持）。Laws of UX 日本語版（『UXデザインの法則』第 2 版、オライリー・ジャパン、2025-01）と、DSR が引く既存の出典（原田秀司『UIデザインの教科書［新版］』、NN/g、GOV.UK、WCAG）を一覧に揃える。`02-component-catalog.md` 72 行目が引く「japanese-webdesign の適用境界」の節は残す。file 名は変えない。
- S4 `docs/design-system/01-decision-rules.md`（判断ルール）: 冒頭に話題別の索引を置き、DSR の節を話題の順に並べ直す（D6）。矛盾する DSR を改訂する（D7。DSR-22 の「選択行」を操作対象として開いている 1 行に限ること、現在地と現在行を色でなく置き場所と文言で区別することを含む）。`読み方` の Why の接地先を「00 の役割表と 03 の出典」へ直す。DSR-22 のルール（438 行付近）の「04 原則 14」を新番号 6 へ直す（456 行付近の「04-backbone 原則 6」は新番号も 6 で変えない。509・510 行の更新履歴は書き換えない）。DSR-21 の Why（428 行付近）が引く 00 の「アクティブ項目のみ Primary アクセント 1 色」は D13 の W7 で置き換わるため、W7 と同じ commit で Why の引用を置換後の文へ直す。
- S5 `docs/design-system/README.md`（索引）: 読む順・「迷ったらここ」（問い → 節）・移行中の読み方と「移行中の作り方」（D8）・各 file の責務表を書く。機械強制の節は残す。
- S6 `docs/design-system/02-component-catalog.md`: 旧原則番号の参照（889・895・897 行付近の原則 15 → 新 10、1012・1016 行付近の原則 11 → 新 7。874 行の原則 4 は新番号も 4 で変えない）を新番号へ直し、冒頭に「現行実装の canonical であり、00 の移行列が runtime lane 待ちとする項目は 00 / 04 / 01 を狙い、本書を現行として読む」旨の 1 文を置く。それ以外の部品の記述は変えない。
- S7 `docs/quality/review-checklist.md` カテゴリ 9: 旧原則番号と「③強調=琥珀pill」の記述を新規則へ合わせ、移行中の許容を「現行の部品の形だけ可。新しい見た目の先取りは不可」に限って明記し（D8）、色の役割と強調の段階を見る 1 行を足す。その 1 行は「役割の無い色・00 に登録されていない token を使っていないか」も見る。全行に DSR 参照を保つ（DS4）。
- S8 `docs/UI_TECH_STACK.md`: §1「デザインシステム」表のカラーパレット行とアイコンサイズ行（51 行目の「16/20/24px 3段階」を D14 の 5 段へ）と「哲学のスタック」節、§4 の索引表を、README の索引と 00 の役割表へ向ける。先頭の「時点証拠契約」節（並走の ADR lane が編集中）には触れない。
- S9 `docs/decision-log.md`: D-091 を追加する（D10）。
- S10 `docs/backlog.md`: 後続 lane 3 件（runtime lane A / B、棚卸し画面 D1）を D11 の題と内容で起票し、41 行目の「`04-backbone.md` の token 表 min-height 40px」の参照を新しい置き場所へ直す。旧 04 の未了の作業（処分表 U13・U14）を 1 行で起票する（U12 は既存の 41 行目で扱う）。

Gated Amendment 1 の Scope の増分（編集する file の集合は変えない。各項目の文は Spec Contract の同じ番号の D と [Review Response](#review-response) の反映先の一覧に置く）:

- S1 00: 色の役割表の操作・進行中・注意・確認・危険・失敗の行（D1）、「どの状態も文字と icon を添え」の箇条（D1）、「段に数えない」の列挙（D2）、迷いやすい場面の節の導入文と試しの書き方（D8・D12）と表の 4 行の追加（D12）。
- S2 04: 原則 7 の hover 背景の 1 文、旧番号の対応表の旧 8 の行と「旧 04 の反映待ちの行き先」表の最終行を右端 chevron だけに、同表に badge の weight の行（D5）。
- S3 03: 「主に根拠にする原則（04）」列を 04 の各原則の「根拠:」行から写し直す（D5）。04 の「旧 03 の節の行き先」表の番号も同じ写し方で合わせる（S2 の file）。
- S4 01: DSR-22 の現在行の段落に、詳細を開いた行の 3 点目の 1 句（D7）。
- S6 02: ⑫ 行インライン展開の「使用トークン」の末尾に狙いの 1 文だけを足す（D8。S6 の「それ以外の部品の記述は変えない」の例外はこの 1 文に限る）。
- S7 review-checklist: カテゴリ 9 の色の役割と強調の段の行の「段 3 と段 4 は 1 画面に 1 つ」を D2 の数え方へ直す（DSR 参照は保つ、DS4）。
- S9 decision-log: D-091 の場面の数・試しの列挙・badge の weight（D10）。
- S10 backlog: lane A の対象・AC・L3 項目の追加（D11）、「旧 04 の未了の見た目の作業」の行の右端 chevron の文（処分表 U14）。

`docs/Plans.md` は Coordinator が Workflow State の遷移と一緒に更新し、Writer は触らない。packet と Matrix は Writer が編集しない。

### 旧 04 の未反映の約束の処分

旧 04（`dda8560a`）の「foundations への追記分（token）」「00〜03 への反映先」「適用の順序」の節と原則本文が、00〜03 へ反映すると約束したもの。S2 で節を削る前に、各行を下の処分で閉じる。反映済みの判定は 2026-09-24 の実物による。

| # | 旧 04 の約束（場所） | 処分 | 行き先 |
|---|---|---|---|
| U1 | 00 の caption 行を 14px muted へ（原則 1） | D14 で閉じる | 00 タイポグラフィ表 |
| U2 | 00 のページ余白を `space-6`（`PageShell` の `p-6`）へ（原則 6） | D14 で閉じる | 00 スペーシング表 |
| U3 | icon を 16 / 20 / 24 の 3 段へ（原則 10、token 表の icon 行） | D14 で閉じる（5 段の表へ置き換え） | 00 アイコンサイズ表 |
| U4 | success 家族の token 登録（原則 2、token 表の success 行） | 反映済み（2026-09-05、00 セマンティックカラー表の success の 4 行） | 00 |
| U5 | token は使う前に 00 の色表へ登録する（原則 2、review-checklist への反映先） | 本 lane の新 2 の本文に残す | 04 新 2 |
| U6 | 密度は業務データ優先、行高 40px（原則 12、00 への反映先） | 本 lane の新 8 の本文に残す | 04 新 8 |
| U7 | DSR-01 に 0 primary の画面の昇格を追記（原則 5、01 への反映先） | 本 lane の D7 で DSR-01 へ（0 primary の画面は昇格を検討し、入口のホームは最重要導線 1 つだけを操作の強調にする） | 01 DSR-01 |
| U8 | 検索欄は全画面で live 型、Enter を押させる commit 型を残さない（原則 7、01・02 ⑨ への反映先） | 反映済み（02 ⑨ の canonical は live 型で、commit 型の採用箇所は 0） | 02 ⑨、04 新 7 |
| U9 | 検索ボタン併記（原則 7、token 表の検索欄の行、02 ⑨ の `SearchBar` を live + ボタン併記の単一形へ） | 撤回（owner 2026-09-24、検索欄は今のボタンなしで確定。02 ⑨ と `SearchBar` の live 型はボタンなし） | D13 の W13、旧番号対応表の撤回・置換列、D-091 |
| U10 | PageHeader の subtitle 基準（原則 9、01 と 02 ① への反映先） | 反映済み（02 ① の `subtitle` と `actions` の併用、PR #63 で runtime 反映） | 02 ① |
| U11 | 02 への反映先の原則 4（⑬ badge 3 種）・6（⑨ 検索行の器）・11（`ListSkeleton`）と、review-checklist の本文 16px（原則 1） | 反映済み（02 ⑬・⑯ の toolbar の枠・⑥ と ⑯ の `ListSkeleton`、review-checklist カテゴリ 9 の 16px の行） | 02、review-checklist |
| U12 | 操作目標 min-height 40px（原則 8、token 表の操作目標の行。`button.tsx` の既定は今も `h-9`） | backlog の既存 41 行目（ボタンの最小サイズの不一致）で扱う。参照先を S10 で直す | backlog |
| U13 | 部門 select の幅を全画面同一に（token 表の検索欄の行。実物は `w-[10rem]` と `w-[11rem]` が混在）。適用の順序 4 段目の sidebar ラベルの折返し・未使用の `src/App.css`（実在し参照 0）の撤去・月数回と年数回の画面の個別 sweep | backlog に 1 行で起票する（S10） | backlog |
| U14 | 押せる行は hover 背景 + 右端 chevron（原則 8 の本文と、適用の順序 2 段目の chevron。旧 04 の 24 行目と 58 行目。実物の一覧の行に右端 chevron は無い） | 右端 chevron は未了のため backlog へ送る（U13 と同じ 1 行、S10）。hover 背景は `TableRow` の既定（`hover:bg-muted/50`）で反映済みのため、04 の新 7 の本文に残す（GA1。`TableRow` の hover は押せない行にも付くため、押せる印の役目は cursor と右端 chevron に置く） | backlog、04 新 7 |

## Non-scope

- `src/styles/globals.css` の token の値の変更・進行中 token の新設・`--rank-top-*` の撤去、それに伴う 00 の HEX の更新と DS3 の同期、横断部品（button・badge の `defaultVariants`・checkbox・progress・`--ring`・待ちの spinner・Home の `ActionButton`・ナビの現在地・現在行）と全画面への適用、D12 の「試し」の行を実画面で見せること → runtime lane A（R3、manual 付き、before / after を並べて owner が見る L3。対象と進め方は D11）。
- 02 部品カタログの tone family 表（日報の「取込み済み」= 注意、owner 決定 2026-09-06）と Alert 節（Home の前日分の未取込み = 危険、日次・月次の未取込み = 注意、owner 決定 2026-09-06）→ 現行の canonical として変えない。変える案は D12 の「試し」として lane A の L3 で owner が見る。
- badge 以外の 12px の文字（`text-xs`、2026-09-24 実測 33 行）を caption 14px へ寄せる作業 → 本 lane は 00 の caption 行を直すだけで、画面は各画面を触る runtime lane が触れたときに直す（lane A の対象に入れない）。
- 書体の実機比較・採用・同梱（依存追加は D-030 の名指し・`--save-exact` に従う）・等幅数字の適用 → runtime lane B。
- 棚卸し画面 D1（owner 採用 2026-09-24）の実装 → ㉘ 以降の棚卸し実装 lane と組にする（挙動も変わるため見た目だけ先に変えない）。
- `docs/SCREEN_DESIGN.md` の画面ごとの色の記述（ホームの入口 card の「primary 枠 + warning soft 背景」、ランキング 1 位の黄色バッジ、「手動」の黄色バッジ、§のウォーム系セマンティックの要約）→ 画面を変える runtime lane A が画面と同時に直す。本 lane では変えない（並走 3 lane も同 file を編集中）。入口 card・ランキング 1 位・ウォーム系の要約は現行の実装を記述しているが、「手動」の黄色バッジ（225 行目）は実物（`ProductTable.tsx` の `variant="secondary"`、②分類）と食い違っている。この食い違いは 00 の側を D13 の W12 で直し、`docs/SCREEN_DESIGN.md` の側は lane A が直す（D11 の対象に名指し）。
- `src/components/patterns/PageShell.tsx` のコメント「04-backbone.md 原則 6」→ 新番号も 6 のため変更不要（旧番号対応表で確認できる）。`docs/function-design/73-ui-stocktake.md` の「04-backbone.md 原則 4」も新番号 4 のため変更不要。
- `docs/design-system/reference/` の mockup と提案原文（正本ではない）。
- `scripts/doc-consistency-check.sh` の DS 検査の変更（file 名を変えないため不要）。
- 02 部品カタログの改題（canvas 案の「03 部品カタログ」）と 01 の改題（「02 判断ルール」）。checker が file 名を固定で読むため採らない（D9）。

## Acceptance Criteria

- AC1 `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan` が ERROR 0。WARN は改訂前（`dda8560a`、full で WARN 0）から増えない。
- AC2 DS 検査が弱まらない: 同 full 出力の DS2 行が `定義 24 件、孤立 0 / 壊れ参照 0`、DS3 行の突合件数が改訂前（`dda8560a` で `27 件突合`）以上、DS4 行が `カテゴリ 9 全 N 項目に DSR 参照あり`（N は改訂後の項目数、改訂前は 22）、DS1 が ERROR なし。
- AC3 DSR の見出しが不変: `diff <(git show dda8560a:docs/design-system/01-decision-rules.md | rg '^## DSR-' | sort) <(rg '^## DSR-' docs/design-system/01-decision-rules.md | sort)` が空出力。
- AC4 索引が全体を覆う: 01 の `## 話題別の索引` 節から `rg -o 'DSR-[0-9]{2}'` で取り出した番号が 01〜24 の 24 種で各 1 回。README の「迷ったらここ」の各行の link 先の節が実在する（Writer が各 link を `rg` で見出しと照合し PR に記録）。
- AC5 役割表: `docs/design-system/00-foundations.md` の色の役割表に 6 役割（操作 / 進行中 / 注意・確認 / 完了 / 危険・失敗 / ふつう・補足）が各 1 行あり、表の見出しに `使う場面`・`見せ方`・`受け取り方`・`狙う効果`・`根拠`・`現行の実装`・`移行` の列がある。危険・失敗の行に `在庫切れ`・`取消済み`・`マイナス`・`取得失敗`・`二重` があり、完了の行に `取込み済み` が無い。役割表の行は `#` で始まる 6 桁 HEX を含まない。
- AC6 強調の段階: 00 の強調の段階表に段 0〜4 が各 1 行あり、段 0 に `stone`・`減衰`・`②分類`、段 1 に `役割の領域`、段 2 に `バー` と `面積`、段 3 に「1 画面に 1 か所」と「黒・文字色で引かない」、段 4 に「押すボタンだけ」・`DSR-01`・`DSR-20`（危険の塗りは dialog の実行ボタンだけ）がある。表の直後に「段に数えない」ものとして `focus ring`・`操作枠`・`checked`・`進み具合の棒`・`spinner` が列挙されている。
- AC7 候補値が canonical docs に無い: `rg -n -i '#1D5C63|#2F7F86|#E6F0F0|#7FB0B4|#123E43' docs/design-system docs/quality/review-checklist.md docs/UI_TECH_STACK.md` が 0 件。候補値から計算したコントラスト比も canonical docs に無い（`rg -n '7\.28:1|4\.47:1|10\.07:1|2\.29:1|1\.11:1|1\.52:1' docs/design-system docs/quality/review-checklist.md docs/UI_TECH_STACK.md` が 0 件、起草時点で 0 件）。候補値と計算結果は `docs/decision-log.md` の D-091 にだけある（`rg -c -i '#1D5C63' docs/decision-log.md` が 1 以上）。
- AC8 書体の旧理由の撤回: `rg -n '読み込み遅延のリスク' docs/design-system docs/UI_TECH_STACK.md` が 0 件。00 の書体の節に `Noto Sans JP` と `BIZ UDPゴシック` がともにあり、最終採用が runtime lane の実機比較で決まることと、それまで現行の system font stack を使うことが書かれている。
- AC9 原則の統合: 04 の原則が 10 本前後（8〜12 本）で、旧番号対応表が旧 04 原則 1〜16 の 16 行を持ち、旧原則に紐づかない W（D13）は対応表の末尾に別行で置かれる。S2 の処分表の U1〜U14 が、それぞれの行き先（00・01・02・04・review-checklist・backlog）で引け、U9 の撤回が旧番号対応表の「撤回・置換」列にある。03 が原則を持たない出典の一覧になっている（`rg -n '核心4本柱|補助3原則' docs/design-system/03-philosophy.md docs/UI_TECH_STACK.md` の hit が、更新履歴の行〈`docs/UI_TECH_STACK.md` 840 行目の 2026-04-16 の行〉を除き 0 件）。
- AC10 旧番号の参照が別の原則を指したまま残らない: Writer は `rg -n '原則 ?[0-9]+' docs/design-system docs/quality/review-checklist.md docs/UI_TECH_STACK.md docs/backlog.md` の全 hit を「新番号で正しい」「旧番号対応表の行」「更新履歴の行」「過去の記録の行（`docs/backlog.md` 235・237 行目の owner 回答の記録）」「Q 番号（原田本の Q 原則）」のどれかに分類して PR に記録し、どれにも当たらない hit が 0 件である。
- AC11 DSR の矛盾の解消: `rg -n 'amber|琥珀|primary|Primary|1 位|1位' docs/design-system/01-decision-rules.md docs/design-system/04-backbone.md` の各 hit が「狙いの規則」（D12 の試しの行で owner の現行の決定を規則として書いたものを含む）「現行の実装（runtime lane 待ち）の説明」「更新履歴の行」のどれかである（Writer が hit ごとに分類し PR に記録、Matrix T11 と同じ式と分類）。DSR-01 / 08 / 16 / 21 / 22 が [Spec Contract](#spec-contract) D7 のとおり改訂されている。
- AC12 データ安全: `rg -n 'claude\.ai|\.local/' docs/design-system docs/quality/review-checklist.md docs/UI_TECH_STACK.md docs/backlog.md docs/decision-log.md` の新規 hit が 0 件（`git diff dda8560a -- …` の追加行で確かめる）。owner の発言は要旨と日付だけで書かれている。
- AC13 後続 lane の起票: `docs/backlog.md` に D11 の題（`デザインの決まり runtime lane A（色と強調）`・`デザインの決まり runtime lane B（書体）`・`棚卸し画面 D1 の実装`）の entry が各 1 行あり、lane A の行の「対象 =」から「。AC =」までの部分に `--ring`・`defaultVariants`・`ActionButton`・`progress`・`spinner`・`alert.tsx`・`table.tsx`・`StepIndicator`・`ProductListTable`、lane A の行全体に `着手条件`・`1 PR`・`origin/main`・`L3`・`variant="destructive"`・`has-aria-expanded`、lane B の行に `着手条件`・`D-030`・`L3`、D1 の行に `着手条件` と `lane A` がある（Matrix T13 の loop で確かめる）。
- AC14 迷いやすい場面: 00 の `## 迷いやすい場面` 節の表がちょうど D12 の 23 場面の行を持ち（表の第 1 列が 23 行で、D12 の場面の語のそれぞれが第 1 列の 1 行だけに当たる。Matrix T15）、各行で役割と段（または `段に数えない`）が 1 つに決まり（行の中で条件ごとに 1 つを書く場合を含む。例: 進み具合の棒は作業の進みと比率で、減衰は押せないボタンと廃番の行・空状態で分ける）、`移行` の値を持つ。条件は要素・画面で分ける。部品の一時的な状態（busy・disabled・hover・focus）で役割を切り替えない。業務の状態が変わる要素（在庫状態の badge、記録状態の badge、0 件と 1 件以上で色が変わる件数）は状態ごとに役割表で役割を選ぶ。「試し」の 7 行（Home の入口 card・日報の取込み済み・Home の前日分の未取込み・ランキング 1 位・最新と上書き件数・詳細を開いた行・取込みの手順表示）の移行の列に `lane A の L3` がある。owner の現行の決定がある 4 行（Home の入口 card・日報の取込み済み・Home の前日分の未取込み・最新と上書き件数）は、役割の列が owner の現行の決定（`owner` と日付を含む）で、試しの答えが移行の列にある。ランキング 1 位・詳細を開いた行・取込みの手順表示の 3 行は、役割の列に試しの答えがあり、試しであることと出典（owner の了承の日付か Coordinator の既定）が移行の列にある。
- AC15 04 からの反映待ちを閉じる: 00 のタイポグラフィ表の caption 行が `14px`、スペーシング表の `space-6` の行に `ページ余白`（`PageShell`）があり `space-8` の行に `ページ余白` が無い、アイコンサイズ表が `12px`・`16px`・`20px`・`24px`・`32px` の 5 行で、32px の行に `spinner` がある。`rg -n 'batch 1' docs/design-system/04-backbone.md` の hit が更新履歴の行だけである。
- AC16 撤回・置換: `rg -n 'info|色は家族|琥珀|新しい色相|ウォーム系主アクセント|Primary アクセント|手動バッジ|ボタン併記' docs/design-system/00-foundations.md docs/design-system/04-backbone.md docs/design-system/01-decision-rules.md docs/quality/review-checklist.md` の全 hit を「置換後の規則」「owner の現行の決定（D12 の試しの行、lane A の L3 まで規則として残す）」「現行の実装（lane A 待ち）の説明」「旧番号対応表の行」「更新履歴の行」のどれかに分類して PR に記録し、どれにも当たらない hit が 0 件。04 に Q7 との関係の 1 文（`Q7` を含む）がある。
- AC17 移行中の作り方: `docs/design-system/README.md` に `移行中の作り方` の節があり、`現行の token`・`lane A の merge 後`・`src/features/suppliers/components/SupplierPickerDialog.tsx`（現在行の canonical、DS1 が実在を検査する）を含む。`rg -n '新しい見た目の先取り' docs/quality/review-checklist.md` が 1 件以上。

## Design Sources

- Requirements / spec: なし（REQ を増減しない）。利用者前提は `docs/project-memory.md` と `docs/design-system/00-foundations.md`「業務ステータスの視認性」。
- Architecture: 変更なし。
- Function / command / DTO: 変更なし。
- DB: 変更なし。
- Screen / UI: `docs/design-system/README.md`、`00-foundations.md`、`01-decision-rules.md`、`02-component-catalog.md`、`03-philosophy.md`、`04-backbone.md`、`docs/quality/review-checklist.md` カテゴリ 9、`docs/UI_TECH_STACK.md` §1・§4、`docs/SCREEN_DESIGN.md`（参照のみ）、`src/styles/globals.css`（参照のみ、token の現行値）。
- Decision log / ADR: D-091（本 lane で追加、番号は予約。下の「連番の予約」）。先例として 2026-09-03 の Codex P2-2 裁定（`docs/archive/plans/2026-09-03-ui-list-backbone-d-lane1-refresh.md`「Token 候補値」節）。

設計判断の出典（要旨と日付、原文は置かない）:

- owner 2026-09-24（デザインの決まりの見直し canvas）: 色の役割が少なすぎ、amber 1 色に「押す・注意・現在地・1 位」が重なっている。操作と進行中は利用者がいま自分でしていることとして同じ緑寄りの色で揃える（ボタンを緑・枠を青に分けると色がそろわず一体に見えない）。黒の 2px 枠と長いバッジは強すぎて煩わしい。書体は Noto Sans JP が好み（Coordinator の推奨は BIZ UDPゴシック、同梱と実機での読み分けが条件）。商品名の下の情報はラベルと値の組が見やすく、日付は何の日付かをラベルで言い切る。決まりは「どう見せるか」でなく「見る人がどう受け取るか・その効果」から組む（見せ方 → 受け取り方 → 効果 → 根拠）。
- owner 2026-09-24（起票後）: ランキング 1 位を色でなく順位と太字で示す案を、試しとして了承。runtime lane A の L3 で実画面を見て最終判断する（D12）。
- owner 2026-09-24（棚卸し画面 canvas）: D1（今のアプリの外枠の中で、部門をページ内の列にした案）を採用。囲みの中で状態が変わる形を基本にする。実装すると挙動も変わる点は owner も認識。
- owner 2026-09-06（既存の決定、本 lane で変えない）: badge の tone family は感情で分ける（緑 = 済んだことのプラスの報告、琥珀 = 手を止めて確かめる注意、赤 = 警告）。日報の「取込み済み」は注意、Home の前日分の未取込みは危険、日次・月次の未取込みは注意（`docs/design-system/02-component-catalog.md` ⑬ と ⑥）。
- owner の既存の決定（本 lane で変えない、試しの行の役割の列の出典）: Home の入口 card は mockup-c の採用（2026-09-11、`docs/SCREEN_DESIGN.md` のホームの節）で、最重要の入口 1 つを primary の線と注意の薄い地で強調する。③強調は琥珀 pill（旧 04 原則 4、owner 採用 2026-08-20。枠色は 2026-09-05 の owner v3 決定で `--warning`、02 ⑬）。
- Coordinator 裁定 2026-09-24（Plan Review round 1 と相談役の検討）: 在庫切れ等の今の赤の割り当ては変えない。変えるものは試しとして lane A の L3 で owner に見せる。Home の入口 card は操作の仲間の薄い地 + 1px 線（試し）。日報の取込み済みの badge は Alert と同じ役割へ揃える方向（試し）。
- Coordinator 裁定 2026-09-24（Plan Review round 2）: 試しのうちランキング 1 位を除く 4 項目（Home の入口 card、日報の取込み済みの badge、Home の前日分の未取込み、最新と上書き件数）は Coordinator の裁定だけで owner の了承が無く、うち 2 つは owner 2026-09-06 の理由付きの決定と逆向きになる。このため 00 の役割の列は owner の現行の決定のままにし、試しの答えは移行の列に置いて lane A の L3 で見せる（D8・D12）。
- owner 2026-09-24（Plan Review round 2 の論点への回答）: 検索欄は今のボタンなしで確定（旧 04 原則 7 の検索ボタン併記は撤回、D13 の W13）。runtime lane A は 1 PR でよい（D11）。
- owner 2026-09-25（Final Review の F2 への判断）: 在庫照会で詳細を開いた行は、進行中（緑寄り）の役割にそろえる形で試す。問題があれば L3 で言うので、L3 の確認事項に含める。試しであることは役割の列でなく移行の列に書く。DSR-22 の「開いている行」は据え置き、02 ⑫ の狙いを進行中へ向ける（D8・D12）。
- Coordinator 裁定 2026-09-25（Final Review broad round 1 と相談役 Fable の検討、Gated Amendment 1）: 13 件を全件採用し、design へ backtrack しない。見た目に触れる 3 点は Fable の推す既定を試しとして置き、runtime lane A の L3 で owner に見せる: 取込みの手順表示のいまのステップ = 進行中の段 2（押すボタンでない塗りの例外を作らない。owner の「進行中 = いま自分がしていること」に沿う）、詳細を開いた行の 3 点目は開いた詳細と「詳細を閉じる」の類の文言が担い badge を足さない（長いバッジは煩わしいという owner の判断に沿う）、badge の weight は現行 500（`font-medium`、2026-09-05 以降の L3 で owner が見てきた形）で旧 04 の 600 を置換する。
- 理論の出典: Laws of UX（日本語版『UXデザインの法則』第 2 版、2025-01、owner 既選定）。theory → 正本 → 実装の順で形作る方針（owner 2026-08-29）。

連番の予約: 本 lane は `docs/decision-log.md` に D-091 を予約する。起草時点で main と並走 lane（`agent/docs-restore-and-rules`、`agent/harness-overhaul`、`agent/harness-pr0-dead-docs`、`agent/stk-runtime-stop-dangerous-ops`、`agent/stocktake-time-evidence-adr-fix`、`agent/ej-parser-core`）の最終 entry はいずれも D-090（各 branch の `docs/decision-log.md` を `git show` で確認）。先に他 lane が D-091 を merge した場合は、本 lane が origin/main を取り込んだ後に採番し直し、Gated Amendment と同一 packet 内の sweep を記録する（`docs/DEV_WORKFLOW.md` Review Rules の連番 registry 規則）。

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | なし | existing sufficient（触れない） |
| Command / DTO / generated binding / wire shape | なし | existing sufficient（触れない） |
| DB / transaction / audit / rollback / migration | なし | existing sufficient（触れない） |
| Screen / UI / route state / Japanese wording | `docs/design-system/*.md`、`docs/quality/review-checklist.md`、`docs/UI_TECH_STACK.md` | updated in this PR（S1〜S8）。画面ごとの記述（`docs/SCREEN_DESIGN.md`）は intentionally deferred（runtime lane A） |
| CSV / TSV / report / import / export format | なし | existing sufficient（触れない） |
| Durable decision / ADR | `docs/decision-log.md` D-091 | updated in this PR（S9） |

## Registration / Generation Obligations

- source doc 新設・改名・削除: 新設・改名・削除なし（D9 で file 名を固定）。design-system の親索引（`docs/design-system/README.md`）と `docs/UI_TECH_STACK.md` §4 の索引表は内容の変化に合わせて更新する（S5・S8）。
- REQ / coverage、Tauri command、function-design doc、route、operator 画面: 該当なし。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| SPEC-DSR-RENEW | 00 色の役割表 | SPEC-DSR-RENEW-D1 | 1 色 1 役割で、押す・注意・現在地・1 位の重なりを解く。却下: 進行中だけ青寄り（色数が増え、ボタンと枠の色がそろわない、owner 2026-09-24）。却下: 今の赤の割り当ての変更（壊さない優先、Coordinator 裁定 2026-09-24） | S1 | Matrix T5・T7 |
| SPEC-DSR-RENEW | 00 強調の段階 | SPEC-DSR-RENEW-D2 | 目立つものが多いと何も目立たない。却下: 黒・文字色の 2px 枠（強すぎて煩わしい、owner 2026-09-24） | S1 | Matrix T6 |
| SPEC-DSR-RENEW | 00 書体 | SPEC-DSR-RENEW-D3 | 旧理由（網からの読み込み遅延）は desktop 配布で当たらない。却下: 本 lane で書体を決め切る（実機の読み分けを見ていない） | S1 | Matrix T8 |
| SPEC-DSR-RENEW | 00 ラベルと値 | SPEC-DSR-RENEW-D4 | 何の値か迷わない。却下: 1 行に詰める現行形（どれが大事か分からない） | S1 | Matrix T5 |
| SPEC-DSR-RENEW | 04 原則 / 03 出典 | SPEC-DSR-RENEW-D5 | 原則が 3 か所に分かれ上下が分からない。却下: 04 と 03 を 1 file へ統合して片方を削除（外部参照の link が切れる） | S2・S3 | Matrix T9・T10 |
| SPEC-DSR-RENEW | 01 話題別の索引 | SPEC-DSR-RENEW-D6 | 出来事ごとの追加順では探せない。却下: 番号の付け替え（src・docs の DSR 参照が壊れる） | S4 | Matrix T3・T4 |
| SPEC-DSR-RENEW | DSR-01 / 08 / 16 / 21 / 22 | SPEC-DSR-RENEW-D7 | 役割表・強調の段階と矛盾する記述を解く | S4 | Matrix T11 |
| SPEC-DSR-RENEW | README / 00 移行列 | SPEC-DSR-RENEW-D8 | 規則と実装の差を読めるようにする。移行中に画面を作る人が現行の形で作るか lane A を待つかを 1 つに決める。却下: 候補 HEX を canonical docs に書く（2026-09-03 Codex P2-2） | S1・S5・S6・S7 | Matrix T7・T18 |
| SPEC-DSR-RENEW | checker との境界 | SPEC-DSR-RENEW-D9 | DS 検査を弱めない | S1〜S6 | Matrix T1・T2 |
| SPEC-DSR-RENEW | decision-log D-091 | SPEC-DSR-RENEW-D10 | 横断の判断と候補値の durable な置き場 | S9 | Matrix T7 |
| SPEC-DSR-RENEW | backlog | SPEC-DSR-RENEW-D11 | 後続 lane の切り方。却下: token を値ごと先に変えて部品を後から追う分割 merge（途中の main で操作・注意・現在地の色が混ざる） | S10 | Matrix T13 |
| SPEC-DSR-RENEW | 00 迷いやすい場面 | SPEC-DSR-RENEW-D12 | 役割表と段の表だけでは部品の状態・現行と狙いの差で答えが 0 か 2 以上になる場面を、1 つの答えに固定する。却下: 場面ごとに DSR を新設（DSR の新設は非目的） | S1 | Matrix T15 |
| SPEC-DSR-RENEW | 撤回・置換表 | SPEC-DSR-RENEW-D13 | 役割表と矛盾する旧文を現行の規則として残さない。却下: 旧文を残して新しい規則を足す（2 つの答えが並ぶ） | S1・S2・S4・S7 | Matrix T17 |
| SPEC-DSR-RENEW | 00 タイポグラフィ・スペーシング・アイコン | SPEC-DSR-RENEW-D14 | 旧 04 が約束したまま 00 に反映されなかった 3 点を実装の多数派で閉じる。却下: 旧 04 の値で src を直す（画面が変わるため本 lane の非目的） | S1・S2 | Matrix T16 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: 改訂後の 00（4 列）・04（原則と旧番号対応）・D-091（決定・候補値・却下案・Revisit）で答えられる。
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: D1〜D14 は S1〜S10 で source docs と D-091 へ移す。packet だけに残る判断はない（D13 の撤回・置換表は 04 の旧番号対応表の「撤回・置換」列と D-091 へ移す）。
- Assumptions and constraints: 候補値のコントラストは起草時に計算した（D-091 候補値の欄。値は本 packet の Contract Probe）。店の PC の書体・同梱サイズは 未実測。
- Deferred design gaps, risk, and follow-up target: 書体の最終採用・候補色の確定・画面ごとの適用は runtime lane A / B（backlog、S10）。
- Test Design Matrix can cite design decision IDs or source doc sections: 可（Matrix の Contract 列が D1〜D11 を引く）。
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: 「1 色 1 役割」の恒久の例外は置かない。部品の状態（focus ring・checked・進み具合の棒・待ちの spinner）は段の数に入れないが、役割は 1 つ持つ（D2・D12）。runtime lane までの間は現行の token が複数役割を兼ねる（`--primary` と `--warning-emphasis` と `--ring` が同値、`--rank-top-*` が amber、`progress` の棒が `--warning`、Home の入口 card が操作の枠と注意の地の混在）ことを 00 の移行列と D12 に明示し、これを例外でなく移行中の状態として扱う。試しの行は lane A の L3 で owner が採るか決める。ランキング 1 位・詳細を開いた行・取込みの手順表示は採らなければ lane A が 00 ほか D8 に挙げた文書を直し、owner の現行の決定がある 4 行は採れば lane A が 00 を試しの答えへ直す（どちらでも規則と実装が一致して終わる、D8）。owner の現行の決定を規則として残す 2 点は、移行中の例外として D12 に名指しする: Home の入口 card の地（操作の要素に注意の薄い地を使う）と、③強調の琥珀 pill（押すボタンでない badge の塗りで、段 2 に数える）。lane A の L3 で owner がこの 2 点の試しを採らなければ、lane A が D-091 に恒久の例外として記録する。

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | not applicable（外部 adapter に触れない） | なし |
| Fact check / design decision split | canvas は判断の材料で、決まりの正本は source docs。canvas の説明文にある外部の事実（BIZ UDPゴシックが Windows 標準に含まれること・その時期、Noto Sans JP の同梱が数 MB）は起草時に確かめていない（未実測）ため、規則には「runtime lane B が vendor の公式資料と店の PC で確かめる前提」として書き、事実として書かない（D3） | runtime lane B |
| Lifecycle / retry | not applicable（状態遷移を持たない文書改訂） | なし |
| Operator workflow | 店主の見え方は本 lane では変わらない。棚卸し D1 の挙動の変化は ㉘ 以降の lane が持つ | backlog D1 entry |
| Replacement path | 書体と役割色の値は token 経由で差し替わる（コードの直接色は stone だけで、他は token 経由。handoff の所見）。runtime lane A はこの前提を `rg` で確かめ直す | runtime lane A |
| Data safety / evidence | canvas の URL・`.local/`・原文を tracked に置かない（AC12） | 本 packet Data Safety |
| Reporting / accounting semantics | not applicable | なし |
| Manual verification | 狙った受け取り方になったかは実機でしか確かめられない（canvas 9 の結び）。本 lane には manual を置かず runtime lane A / B の L3 に置く | runtime lane A / B |
| 環境・再現性 | 書体の同梱は配布物と依存を変えるため runtime lane B で pin と D-030 を扱う | runtime lane B |

## Design Readiness

- Existing design docs are sufficient because: 本 lane は design-only scope で、改訂内容は Spec Contract が固定する。
- Source docs updated in this PR: S1〜S10。
- Design gaps intentionally deferred: 書体の採用、候補色の確定値、画面ごとの適用（runtime lane A / B）、棚卸し D1（㉘ 以降）。
- Durable decisions discovered in this plan and promoted to source docs: D-091。

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): 影響なし。
- Backend function design: 影響なし。
- Command / data contract: 影響なし。
- Persistence / transaction / audit impact: 影響なし。
- Operator workflow / Japanese UI wording: 画面の文言は変えない。規則の文面（ラベルで言い切る等）は後続の画面設計に効く。
- Error, empty, retry, and recovery behavior: 影響なし（DSR-03 / 11 / 19 / 20 の本文は変えない）。
- Testability and traceability IDs: REQ なし。Matrix の oracle は `rg` / doc check。

## Contract Probe

R2 のため必須ではない。起草時に次を確かめた。

- DS 検査は file 名を固定で読む: `scripts/doc-consistency-check.sh` 1796〜1797・1842 行を読み、`01-decision-rules.md`・`02-component-catalog.md`・`00-foundations.md` の path を固定で参照し、無ければ INFO でスキップすることを確認 → file 名を変えない（D9）。
- DS3 の抽出規則: 同 script 1833〜1878 行を読み、「backtick の `--name` を含む行の最初の #hex」を `globals.css` `:root` と突合することを確認 → 役割表の行に HEX を置かない（AC5）。
- 改訂前の DS の結果: `bash scripts/doc-consistency-check.sh`（`dda8560a`）→ `DS1: src/ path 参照 44 件すべて実在` / `DS2: DSR 参照整合 OK（定義 24 件、孤立 0 / 壊れ参照 0）` / `DS3: token HEX 整合 OK（27 件突合）` / `DS4: カテゴリ 9 全 22 項目に DSR 参照あり` / `結果: 全チェック通過`。
- 分類: `scripts/ci/classify-changes.sh` 59 行目の policy docs 列挙に本 lane の予定 file が含まれないことを読んで確認 → `workflow=false`。
- 候補色のコントラスト（WCAG 2.x の相対輝度式、`python3` の 1 行計算、2026-09-24）: 操作の塗りと背景 `#fafaf9` = 7.28:1、進行中の枠と背景 = 4.47:1、進行中の文字と進行中の地 = 10.07:1、進行中の線と背景 = 2.29:1、進行中の地と背景 = 1.11:1、操作の塗りと完了の緑（`--success`）= 1.52:1。参考に現行の `--primary` と背景 = 4.81:1。含意: 進行中の線は 3:1 未満のため操作枠や現在行のバーの単独信号にできない、進行中の地は背景とほぼ同じで文字と併用が必須、操作と完了は色で区別できず文字と icon が必須。候補 HEX は D-091 に置く。
- ③強調の badge の移し先（Plan Review round 3、同じ式で計算、2026-09-24）: 塗りの上の文字 `--primary-foreground`（`#fafaf9`）との比は、`--warning-emphasis`（`#b45309`）= 4.81:1、`--warning`（`#d97706`）= 3.05:1。後者は 4.5:1 に届かないため、移し先は `--warning-emphasis`（D11）。値は D-091 に置き、canonical docs には書かない。
- 文書の規模: `wc -l docs/design-system/*.md`（`dda8560a`）→ 合計 1953 行（00: 158 / 01: 520 / 02: 1068 / 03: 78 / 04: 71 / README: 58）。
- 旧原則番号の参照元: `rg -n '原則 ?[0-9]+' docs --glob '!docs/archive/**'` と `rg -n '04-backbone' src docs` で洗い出した（Matrix の Adjacent Pattern Audit）。新番号 4・6 を旧番号と同じ主題に置くため、`src/components/patterns/PageShell.tsx`（原則 6）と `docs/function-design/73-ui-stocktake.md`（原則 4）は変えずに済む。
- Plan Review round 1 の是正で src を当てた（`85e087f5`、2026-09-24、起草役の再測。相談役の実測と照合し、食い違いは Review Response に記録）:
  - 操作の役割色の消費: `src/styles/globals.css` の `--ring` は `--primary` と同値（`#b45309`）。`badge.tsx` の `defaultVariants` は `variant: "default"`（= `bg-primary`）。`checkbox.tsx` の checked は `bg-primary`。ナビの現在地は `selection-tone.ts` の `CURRENT_LOCATION_ACCENT = "border-l-primary"`。features・patterns・layout の primary の消費は数え方で揺れる（`rg -n '(text|bg|border|ring|fill|stroke)-primary\b' src/features src/components/patterns` → 18 行、`rg -n 'primary' src/features src/components/patterns src/components/layout` → 23 行、相談役の数え方で 25 site）ため、lane A の Plan で式を固定して数え直す（D11）。
  - 進行中へ移る表示: 待ちの spinner は `rg -n 'Loader2.*size-8.*text-primary|size-8 animate-spin text-primary' src` → 5 site（csv 取込みの 2 step・日報取込み 2・整合性チェック 1）。進み具合の棒は `progress.tsx` の indicator が `bg-warning`、消費は `rg -n '<Progress' src/features` → 3 site（棚卸し・整合性チェック・月次の部門比率）。保存中のボタンは `rg -n '保存中|確定しています' src --glob '!*.test.*'` → 6 site（元の役割のまま文言と spinner）。
  - 危険の塗り: `rg -n 'variant="destructive"' src` から `Alert` を除いた 10 行はすべて dialog の実行ボタン（`AlertDialogAction` 9、取引先の統合 dialog の `DialogFooter` 内の `Button` 1）。画面上に危険の塗りのトリガーは無い。
  - 現在行の 3 点表示: `rg -n 'border-l-primary bg-row-current' src` → `SupplierPickerDialog.tsx` の 1 site だけ（canonical）。一括価格改定の「入力中」は outline badge 1 点で、3 点表示でない。
  - ③強調の badge: `rg -n 'variant="default" className="border-warning"' src` → 最新（バックアップ）と上書き件数（商品取込み）、ランキング 1 位は `ProductRankingTable.tsx` の `rank-top` custom class。`toast.info` / `toast.warning` は `rg -n 'toast\.(info|warning)' src` → 0 件。
  - 04 からの反映待ち（D14）: caption は `rg -n 'text-xs' src/features src/components/patterns --glob '!*.test.*'` → 33 行、`rg -n 'text-sm text-muted-foreground' src --glob '!*.test.*'` → 101 行で 14px muted が多数派。ページ余白は `PageShell.tsx` の `p-6`（24px）。icon は `size-4` 35 行・`size-5` 4 行・`size-6` 2 行・`size-8` 9 行（`rg -n 'size-N\b' src` を N ごとに数えた）、`button.tsx` の svg 既定が `size-4`、`badge.tsx` の svg が `size-3`（12px）、`EmptyState.tsx` が `size={24}`、ナビの icon が `size-4`。

## Test Plan

- targeted tests: `bash scripts/doc-consistency-check.sh` と `--target plan`、Matrix の `rg` / `diff` oracle（T1〜T18）。
- negative tests: 候補 HEX が canonical docs に無い（T7）、候補値から計算した比が canonical docs に無い（T7）、撤回・置換した旧文が現行の規則として残らない（T17）、旧書体理由が残らない（T8）、旧原則番号が別の原則を指したまま残らない（T10）。
- compatibility checks: DSR 見出しと番号が不変（T3）、file 名が不変（T2）、外部の link 先（`03-philosophy.md` の japanese-webdesign 節、`04-backbone.md`）が実在（T12）。
- data safety checks: canvas URL・`.local/`・原文が追加行に無い（T14）。
- main wiring/integration checks: DS2 / DS3 が改訂後の file を実際に読んで件数を出す（T1）。

## Review Focus

- Plan Review の冒頭で、[Ordinary Operation](#ordinary-operation) の操作列が目的を達成できるかを `成立 / 具体的な反例あり / 外部前提が未確認` で答える。
- 役割表・強調の段階・書体・ラベルと値が owner の判断（「設計判断の出典」）と食い違わないか。
- 候補値を canonical docs に置かない方針（D8）と、読む人が現行と狙いを取り違えない書き方が両立しているか。
- DS 検査が弱まらないか（D9、AC2）。
- 原則の統合で、旧 16 原則と 03 の哲学の内容が落ちていないか（旧番号対応表で辿れるか）。
- DSR-01 / 08 / 16 / 21 / 22 の改訂が役割表・強調の段階と一貫し、他の DSR と新たな矛盾を作らないか。
- runtime lane の切り方（D11）が、画面・挙動・依存の変化をそれぞれの Human Gate に正しく載せているか。
- GA1 の後の Final Review broad で特に見る点: 13 件（Opus F1〜F7、Codex #1〜#6）が根 A〜E の是正で閉じたか。新しい 4 行と状態切替の文が他の場面・役割表・段の表と新たに 2 つの答えを作らないか（とくに同日追加確認と日報の二重取込みの Alert、詳細を開いた行と DSR-22 の 3 点、取込みの手順表示と段 4「押すボタンだけ」）。試しの 3 点（手順表示・詳細を開いた行の 3 点目・badge の weight）と F2 の詳細を開いた行が、役割の列でなく移行の列（badge の weight は 04 の反映待ちの表と D-091）で試しと読めるか。T13・T15 の新しい oracle が欠落の変異を検出するか。
- round 3 で特に見る点: 試しの 4 行を「役割の列 = owner の現行の決定」へ反転した結果、D1・D2・D5 の新 4・D12・D13 が互いに矛盾しないか（とくに③強調の琥珀 pill を注意・確認に置き段 2 に数える例外と、Home の入口 card の地の例外）。S2 の処分表 U1〜U13 が旧 04 の約束を漏らさないか。AC14 の「条件ごとに 1 つ」が 1 要素 2 役割を許す抜け道にならないか。
- round 2 で特に見る点: D12 の 19 場面の答えが役割表・段の表と一致し、今の赤の割り当てを変えていないか。「試し」の行が lane A の L3 まで現行のまま読めるか。D13 の撤回・置換表に漏れが無いか。D14 の値が src の実測と合うか。役割名「危険・失敗」が「危険・戻せない」と「失敗・欠け」の両方の場面を 1 行で持てているか。

## Spec Contract

Contract ID: SPEC-DSR-RENEW

| Contract | 内容 | Test |
|---|---|---|
| SPEC-DSR-RENEW-D1 色の役割表 | 00 に 6 役割の表を置く。役割は要素（領域・行・badge・ボタン・部品の状態）ごとに 1 つ選び、行の役割とその行の中の badge の役割は別に選ぶ。操作（押すボタン・リンク・ナビの現在地・フォーム部品の checked と focus ring。塗りは押すものだけ。リンクは文字色と hover の下線）/ 進行中（利用者の作業が続いている領域・行: 数えている・取り込んでいる・入力中の行・待ちの spinner・作業の進み具合の棒・詳細を開いた行〈試し、D12〉。操作と同じ色の仲間で、薄い地で表す。部品自身の busy〈保存中のボタン〉は元の役割のまま spinner と文言で示す。お知らせ一般に使わない）/ 注意・確認（手を止めて確かめてほしいもの: 在庫少・数え直し・未入力・同日データあり〈同日追加確認の Alert と badge、DSR-03〉・未反映・部分成功・日次と月次の未取込みの知らせ・日報の取込み済みの badge〈owner 決定 2026-09-06〉。怖くはない。③強調の琥珀 pill〈最新・上書き件数〉も owner の現行の決定として lane A の L3 までこの役割に置く、D12）/ 完了（済んだことのプラスの報告: 保存した・取込みが成功した・反映済み・補正済み・数え終わり。段 2 まで、塗りにしない。「取込み済み」は二重取込みを防ぐ知らせで、完了に入れない）/ 危険・失敗（二つの場面を 1 行に持つ。危険・戻せない = 戻せない操作の確認〈dialog の実行ボタン、DSR-20〉。失敗・欠け = その行・画面の値が成り立たないもの〈取得失敗・在庫切れ・取消済み・マイナスの増減〉と、進めない〈日報の二重取込みの Alert・取得失敗。DSR-03 のデータ安全系のうち止めるもの〉、Home の前日分の未取込みの Alert〈owner 決定 2026-09-06〉。日報の取込み済みの badge はこの役割に入れない〈注意・確認、D12〉。確認すれば進める同日追加確認は注意・確認）/ ふつう・補足（説明・件数・日付・ランキング・分類の badge・選択状態〈DSR-21〉・手を止めなくてよいお知らせ一般〈Alert の既定の stone〉・比率の棒）。列 = 役割・使う場面・見せ方・受け取り方・狙う効果・根拠・現行の実装（token 名、HEX なし）・移行（済 / runtime lane A 待ち / lane A の L3 で試し）。危険・失敗の移行は、badge と dialog の実行ボタンは済、Alert は runtime lane A 待ち（D12 の「危険・失敗の Alert」の行）。注意・確認と危険・失敗の見せ方の末尾に「文字だけの形は迷いやすい場面の行に従う」を置く（GA1）。今の赤・琥珀・緑の割り当て（在庫切れ・取消済み・マイナスの増減・取得失敗の赤、02 ⑬ の tone family 表の注意と完了、owner 決定 2026-09-06）は変えない。現行と狙いが分かれる場面は D12 に置く。「どの状態も文字か記号を添え、色だけに頼らない。badge と Alert の icon は DSR-08 と 02 ⑬ ⑥ に従う」（GA1。`FieldError` は icon を持たない〈02 ④〉ため「文字と icon」から改めた）「進行中は info ではない」を明記。既存の token 表の各行に役割の列を足す（HEX と行は変えない）。受け取り方・効果・根拠の文は canvas 9 の表（操作: Hick の法則 / Von Restorff 効果、進行中: Zeigarnik 効果 / Von Restorff 効果、注意: Von Restorff 効果 / WCAG 1.4.1、完了: Peak-End の法則）を元にする。canvas 9 の表に無い 2 役割は次で書く。危険・失敗: 受け取り方 = 止まる、戻せない・成り立っていないと分かる。根拠 = Von Restorff 効果（最も強い色を危険と失敗だけに残す）、WCAG 1.4.1（文字と icon を添える）、既存の DSR-20（戻せない操作の確認 dialog）と DSR-03（Toast と Alert の使い分け、データ安全系の知らせ）。ふつう・補足: 受け取り方 = 読むだけでよい、手を止めなくてよい。根拠 = Von Restorff 効果（ふつうのものに色を使わないことで役割色が際立つ）、美的ユーザビリティ効果（色の少ない画面は落ち着いて読める）、既存の DSR-21（選択状態は stone） | Matrix T5・T7 |
| SPEC-DSR-RENEW-D2 強調の段階 | 00 に段 0〜4 を置く。0 そのまま（地と線は stone。ただのまとまり〈card・区画・Alert の既定〉と②分類の badge〈恒常的な属性で状態を知らせない。stone の地と `--border` の枠、02 ⑬〉も段 0。減衰のうち廃番の行・空状態は段 0 の muted で弱め、役割色を足さない）/ 1 役割の薄い地（その役割の領域だけ、線なし。例: 取込み中の領域の進行中の地）/ 2 薄い地 + 同じ役割の 1px 線、または細いバー（状態を知らせる行・badge・Alert と、Home の入口 card〈最重要の入口 1 つ、D12〉と、現在地・現在行の左端のバー〈DSR-21 / 22〉。線の濃さは面積で決める: badge は `-border`、Alert・行は役割の base。完了は段 2 まで）/ 3 役割色の 2px 枠（いま操作している 1 つだけ、1 画面に 1 か所、黒・文字色で引かない）/ 4 塗り（押すボタンだけ。1 画面の主要ボタンは 1 つ = DSR-01 で、この 1 つは操作の塗りだけを数える。危険の塗りは dialog の実行ボタンだけ〈DSR-20〉で、画面上の確定・削除のトリガーは操作の役割。badge の塗りは③強調の琥珀 pill〈owner の現行の決定、D12 の試しの行〉だけが lane A の L3 まで残り、段 2 に数える）。押せないボタンは元の役割・元の段のまま opacity で弱める（主要ボタンが押せないときも段 4 の 1 つに数える）。表の直後に「段に数えない」もの（1 画面の上限の数に入れない）を列挙する: focus ring（`--ring`、操作の役割）・操作枠（`--border-strong`、DSR-22 の 3:1）・checkbox / radio の checked の小さな塗り（操作の役割）・進み具合の棒の塗り（作業の進み = 進行中、比率 = stone）・待ちの spinner（進行中）・役割色の文字だけの表示（GA1。入力欄のエラー文・増減の ±・セルの数字・件数・リンク。役割は D12 の「役割色の文字だけの表示」の行に従い、上限は置かない。記号・文言・下線のどれかを必ず添える）。上の段ほど 1 画面に置ける数を減らす。バッジの文は短く（目安 8 文字）、補足は下に小さく分ける。各段に 4 列（根拠: Von Restorff 効果、美的ユーザビリティ効果）。DSR-16（囲みは意味階層ごとに 1 つ、薄い線を単独のグループ信号にしない）と矛盾しないこと: 段 1 は線を引かず役割の領域だけに使い、ただのまとまりは段 0 の stone、段 2 の線は状態の行・badge・Alert と Home の入口 card に限る | Matrix T6 |
| SPEC-DSR-RENEW-D3 書体 | 00 の書体の節を改める: 選定条件（見間違えにくさ〈数字と似た形のカナ〉、太字でつぶれない、長時間読める、数字は等幅で桁がそろう、配布の方法〈Windows 標準か同梱か〉）。候補は Noto Sans JP（owner の好み、同梱が要る）と BIZ UDPゴシック（Coordinator の推奨。Windows 標準に含まれ同梱が要らないという前提は、起草時に確かめていない外部の事実として書き、runtime lane B が vendor の公式資料と店の PC で確かめる）。旧理由「カスタム Web フォントは読み込み遅延のリスクで不採用」は、Tauri の desktop 配布では網から読まないため当たらないとして撤回し、判断軸を同梱サイズと店の PC での読み分けに置き換える。最終採用は runtime lane B の実機比較（店主が通常距離で読み分けられるか、owner の目）で決め、それまでは現行の system font stack を使う。同梱サイズ・店の PC の書体の有無は 未実測 と書く | Matrix T8 |
| SPEC-DSR-RENEW-D4 ラベルと値・進み具合 | 00 に置く。適用範囲は 2 つ: (a) 補足情報（商品名の下の部門・コード・前回数えた数・数えた日のような副情報）はラベルと値の組にし、ラベルは小さく薄く、値はふつうの濃さ（本文の大きさ）。(b) サマリカードの主値はラベル（小さく薄く）と値（metric 30px、00 のタイポグラフィ表）の組で、旧 04 原則 15 の後半（ラベル小・値大）の行き先とする。どちらも何の値かを言葉で言い切る（日付なら何の日付か。例: 前回数えた数 / 数えた日）。1 行に詰めて同じ薄さで並べない。根拠: チャンク化 / 近接の法則。進み具合は中くらいの大きさで数と棒の両方（根拠: 目標勾配効果、大きすぎると急かされる。棒の塗りは D2 の段に数えない）。いずれも 4 列 | Matrix T5 |
| SPEC-DSR-RENEW-D5 原則の統合 | 04 を「原則」にし、次の 11 本とする（番号 4・6 は旧番号と同じ主題に置く）: 1 読める大きさ（本文 16px の最低線、caption は 14px muted、12px は badge の中だけ、書体、icon は 00 の表の段だけ）/ 2 1 色 1 役割、色だけに頼らない、役割の無い色を足さない、token は使う前に 00 の色表へ登録する（処分表 U5）/ 3 強調は段階で決め、上の段ほど少なく（主要ボタンは 1 つ）/ 4 badge は 3 種（③強調の現行は琥珀 pill で、owner の決定。ランキング 1 位は色を使わず順位と太字で示す〈owner 了承の試し〉。最新・上書き件数を stone の pill と太字へ移す案は lane A の L3 で owner が決める試しで、採れば新 4 と 00 を直す）/ 5 何をする画面か・何の値かを言葉で言い切る（ラベルと値、D4）/ 6 器は 1 つ（PageShell・ListShell）/ 7 同じ操作は同じ顔と挙動（検索欄は live 型でボタンなし〈owner 2026-09-24 確定、処分表 U8・U9〉・待ち時間・押せる顔と当たり判定。GA1: 押せる行は `cursor-pointer` と hover 背景〈`TableRow` の既定 `hover:bg-muted/50`〉を持ち、押せない行にも hover が付くため押せる印の役目は cursor と右端 chevron〈未了、処分表 U14〉に置く）/ 8 密度は業務データ優先（行高 40px のまま、処分表 U6）/ 9 枠と線は少なく、操作枠は 3:1 / 10 いま扱っているものは進行中で示す（現在行・作業中の囲み。選択行は操作対象として開いている 1 行に限る）/ 11 低視力を前提に実機で確かめる。旧番号対応表: 旧1→1、旧2→2、旧3→2、旧4→4、旧5→3、旧6→6、旧7→7、旧8→7、旧9→5、旧10→1、旧11→7、旧12→8、旧13→9 と 2、旧14→6、旧15→10（前半: 現在行の 3 点）と 5（後半: ラベル小・値大、00 のラベルと値の節）、旧16→11。対応表の列は「旧番号・旧文の要旨・新番号（規則ごと）・撤回・置換（D13 の行）」。旧 03 の核心 4 本柱・補助 3 原則・観点借用は 03 の出典に残し、対応表で各々がどの新原則の根拠かを示す。旧原則の細則（token 名・path・DSR への委譲）は落とさず新原則の本文か 00 / DSR へ移す。GA1: 03 の「主に根拠にする原則（04）」列と 04 の「旧 03 の節の行き先」表の番号は、04 の各原則の「根拠:」行を正として写し直す（04 の根拠の行は変えない。列は残す）。04 の「旧 04 の反映待ちの行き先」表に 1 行足す: 旧 token 表の badge の行（12px / 600 / pill）→ 12px と pill は 02 ⑬ と `badge.tsx`、weight は 600 を現行の 500（`font-medium`）で置換（2026-09-05 以降の L3 で owner が見てきた形。lane A の L3 の before / after で owner が違和感を言えば見直す）。600 は見出し（00 のタイポグラフィ表）と `AlertTitle`（02 ⑥）が使い、badge には使わない。同表の最終行と旧番号の対応表の旧 8 の行は「右端 chevron（hover 背景は反映済み）」にする | Matrix T9・T10 |
| SPEC-DSR-RENEW-D6 DSR の並べ直し | 01 に `## 話題別の索引` を置き、DSR を次の話題の順に並べる。配置と一覧: DSR-04, 09, 12, 13, 16, 22。入力: DSR-02, 05, 06, 10, 14, 23, 24。知らせ方: DSR-03, 07, 11, 19, 20。移動と戻り: DSR-15, 17, 18。色と状態: DSR-01, 08, 21。`## DSR-NN 題` の見出しの文字列と番号は変えない。話題の区切りの見出しを置く場合も DSR の見出しは `## ` のまま。title の「DSR-01〜24」は維持 | Matrix T3・T4 |
| SPEC-DSR-RENEW-D7 矛盾する DSR の改訂 | DSR-01: 「Primary は amber 系」を「Primary は操作の役割色（`--primary`、現行値は 00 の役割表）」へ。「1 画面 1 primary」は操作の塗りだけを数え、dialog の危険の実行ボタンは DSR-20 に従うと 1 文加える。旧 04 原則 5 の未反映分（処分表 U7）を加える: 0 primary の画面は昇格を検討し、入口のホームは最重要導線 1 つだけを操作の強調（Home の入口 card、D12）にする。DSR-08: badge の tone family の説明を 00 の役割表へ向け、進行中を加える。今の tone family の割り当て（owner 2026-09-06）は変えない。ランキングの強調は色でなく順位と太字（試し、移行中は現行）とする。DSR-16: 強調の段階との関係を 1 文加える（段 1 は役割の領域だけ、ただのまとまりは段 0）。DSR-21: 現在地は操作の役割色の細いバー（段 2）のまま、Why の「Primary（amber-700）は warning 系と同系色」は runtime lane A までの現状として書き、役割表により色相が分かれることを書く。現在地と現在行は色で区別せず、置き場所（ナビか一覧の行か）と文言で区別する（DSR-22 と対象階層で分ける既存の記述を保つ）。DSR-22: 現在行（入力中 / 開いている行 / 選択行）を進行中の役割に属させ、「選択行」は操作対象として開いている 1 行に限る。GA1: 詳細を開いた行（02 ⑫ の行インライン展開、操作ログの詳細の開閉）はこの「開いている行」に当たる（owner 2026-09-25 の試し、D12）。3 点表示の 3 点目は直下に開いた詳細と「詳細を閉じる」の類の文言が担い、badge を足さない、の 1 句を DSR-22 に足す（「開いている行」の割り当ては owner の判断どおり据え置き）。checkbox / radio の複数選択の行は選択状態（DSR-21 の stone と部品の checked）で、現在行に含めない。3 点表示（バー + 薄い地 + 文言）を維持、作業中の囲みは段 3 で 1 画面 1 か所とする。現行の `--row-current` と primary のバーは runtime lane A で進行中の token へ移すと書く。バーと作業中の囲みの枠は隣接背景に対し 3:1 以上の色で引く（既存の操作枠の要件と同じ）とだけ書き、候補値から計算した比は DSR-22 に書かず D-091 に置く。その他の DSR は `rg -n 'amber\|琥珀\|primary\|Primary\|1 位\|1位' docs/design-system/01-decision-rules.md` の各 hit を確かめ、変えない理由を PR に記録する | Matrix T11 |
| SPEC-DSR-RENEW-D8 移行中の読み方と作り方 | canonical docs は狙いの規則を書き、現行の実装との差は 00 の役割表の「現行の実装」「移行」列と D12 の場面表に集める。候補の HEX は canonical docs に書かず D-091 に置く（2026-09-03 Codex P2-2 の先例）。README に「移行が runtime lane 待ちの項目は、00 / 04 / 01 を狙い、02 と画面を現行として読む」を書き、02 の冒頭に同旨の 1 文を置く。README に `移行中の作り方` の節を置く: lane A の merge 前に画面を作るときは、現行の token と 02 の部品の形で作る。現在行は DSR-22 の 3 点表示で、canonical は `src/features/suppliers/components/SupplierPickerDialog.tsx` の現在行（一括価格改定の「入力中」は outline badge 1 点のため canonical にしない）。進行中の地・作業中の囲み（進行中の段 1〜3）を新しく要する画面は lane A の merge 後に作る。review-checklist カテゴリ 9 の移行中の許容は「現行の部品の形だけ可。新しい見た目の先取り（候補色の直書き・新 token 名の先取り）は不可」に限る。D12 の試しの行は lane A の L3 で owner が採るか決める。書き方と直す向きは 2 通り。(1) 役割の列に試しの答えを書き、試しであることと出典を移行の列に書く行: ランキング 1 位（owner が試しとして了承済み）、詳細を開いた行（owner 2026-09-25 が試しとして了承、GA1）、取込みの手順表示（owner の決定が無い場面で Coordinator の既定、GA1）。owner が採らなければ lane A が直す: ランキング 1 位は 00 の役割表と場面表、04 の新 4、DSR-08 のランキングの文、review-checklist カテゴリ 9 の badge の行（W10）を現行へ。詳細を開いた行は 00 の役割表と場面表、02 ⑫ の狙いの 1 文、DSR-22 の 3 点目の 1 句を stone（ふつう・補足の段 0）へ。取込みの手順表示は 00 の場面表の行を owner の選んだ形へ直し、押すボタンでない塗り（段 4）を残すなら D-091 に例外として記録する。00 の迷いやすい場面の節の導入文（ランキング 1 位の書き方の箇条）は (1) の 3 行を並べる形へ直す（GA1）。02 ⑫ の「使用トークン」の末尾に「狙いは 00 の進行中の役割（迷いやすい場面「詳細を開いた行」。runtime lane A の L3 で試し）。本書は現行」の旨の 1 文を置く（GA1、S6 の例外）。(2) 他の 4 行（Home の入口 card・日報の取込み済みの badge・Home の前日分の未取込み・最新と上書き件数）は役割の列に owner の現行の決定を書き、試しの答えを移行の列に置く。owner が採れば lane A が 00 の役割表と場面表（と 02・04 の該当文）を試しの答えへ直し、採らなければそのまま残す（Home の入口 card の地と③強調の琥珀 pill は、lane A が D-091 に恒久の例外として記録する）。棚卸し画面 D1 の実装 lane の着手条件にも lane A の merge を置く（D11） | Matrix T7・T18 |
| SPEC-DSR-RENEW-D9 checker との境界 | `00-foundations.md`・`01-decision-rules.md`・`02-component-catalog.md`・`03-philosophy.md`・`04-backbone.md`・`README.md` の file 名を変えない。DSR の見出しは `## DSR-NN` のまま。00 の既存 token 表の行と HEX を変えず、新しい表の行に backtick の `--name` と #hex を同じ行に並べない。review-checklist カテゴリ 9 の全行に DSR 参照を保つ。`scripts/doc-consistency-check.sh` の M3 検査（`check_stale_markers` の `markers`）は `docs/*.md`（本 lane が触る `decision-log.md`・`backlog.md`・`UI_TECH_STACK.md` を含む）・`docs/quality/*.md`・`docs/design-system/*.md` を、曖昧表現検査（`ambiguous_words`、句読点の直前の「など」を含む）は `docs/design-system/*.md` を走査するため、そこで列挙される語を新しい文に入れない（WARN が増える）。canvas の役割名「注意・」に続く 3 文字は前者の語に当たるため、役割名は「注意・確認」と書く | Matrix T1・T2 |
| SPEC-DSR-RENEW-D10 D-091 | decision-log に置く: 決定（1 色 1 役割の 6 役割、強調の段階 0〜4 と段に数えないもの、迷いやすい場面の答え、書体の選定条件と runtime lane での採用、ラベルと値、原則の統合と DSR の話題別の並び、D13 の撤回・置換と Q7 との関係、D14 の 3 点）・理由（見る人の受け取り方から決める。今の割り当ては壊さず、変えるものは試し）・候補値（操作 塗り #1D5C63、進行中 地 #E6F0F0 / 線 #7FB0B4 / 枠 #2F7F86 / 文字 #123E43、注意 amber-50 / amber-200 / amber-900 + icon、完了 green-50 / green-200 / green-900、危険 red 系、ふつう stone。runtime lane A の実測と L3 で確定）と起草時のコントラストの計算結果（本 packet の Contract Probe の値。進行中の線が 3:1 未満のため、バーと囲みの枠に使えないことを含む）・③強調の badge の移し先（`--warning-emphasis` と `--warning` の、塗りの上の文字との比。Contract Probe の値）・試しの項目と L3 で決める人（owner）・却下案（進行中だけ青寄り、黒の 2px 枠と長いバッジ、本 lane での書体の決め切り、file の改名、今の赤の割り当ての変更、token の分割 merge）・Revisit（runtime lane A / B の L3 で狙った受け取り方にならなかったとき）。GA1: D-091 の迷いやすい場面の数を 23 にし、試しの列挙に詳細を開いた行を進行中へ（owner 了承 2026-09-25、3 点目は開いた詳細が担い badge を足さない）と取込みの手順表示のいまのステップを進行中の段 2 へ（Coordinator の既定 2026-09-25）を足し、旧 04 の badge の weight 600 を現行 500 で置換したことを 1 句で記録する | Matrix T7 |
| SPEC-DSR-RENEW-D11 後続 lane の切り方 | backlog に 3 entry を各 1 行で置く（題は下の太字の文字列どおり）。**デザインの決まり runtime lane A（色と強調）**: 色と強調の token・横断部品・全画面、R3、manual、before / after を並べて owner が見る L3。着手条件 = 本 lane の merge。進め方 = 新 token を別名で追加 → 部品・画面を新 token へ移す → 最後に `--primary` / `--ring` の値を変えて旧 token（`--warning-emphasis` の操作との兼用・`--rank-top-*`）を削る。merge は 1 PR（途中の状態を main に置かない。全画面を 1 PR で出すことは owner 了承 2026-09-24）。対象に名指しするもの = `globals.css`、`button`、`badge`（`defaultVariants` を `default` から外す）、`checkbox`、`progress`（棒の塗り）、`--ring`、待ちの spinner（`Loader2` の `text-primary`）、Home の `ActionButton`（操作の段 2。地を操作の仲間へ揃えるのは試し）、ナビの現在地（`CURRENT_LOCATION_ACCENT`）、現在行（`SupplierPickerDialog`）、ランキング（`ProductRankingTable`）、③強調の badge（最新・上書き件数。今は `variant="default"` で `--primary` の塗りのため、`--primary` の値を変えても owner が試しを採るまで琥珀のままになるよう `--warning-emphasis` へ移す。`--warning` は塗りの上の文字〈`--primary-foreground`〉との比が 4.5:1 に届かないため移し先にしない。比の計算結果は D-091 に置く）、日報の取込み済みの badge、Home の未取込みの Alert、`alert.tsx`（`destructive` の Alert の地・線・icon。D12 の「危険・失敗の Alert」の行、GA1）、詳細を開いた行（`table.tsx` の `data-[state=selected]:bg-muted` と `has-aria-expanded:bg-muted/50`、在庫照会の `ProductListTable`、操作ログの詳細の開閉。GA1）、取込みの手順表示（`StepIndicator`、GA1）、00 に登録されていない token の既存の drift（`IntegrityCheckPage` の `text-warning-foreground`、GA1）、`docs/SCREEN_DESIGN.md` の画面ごとの色の記述（「手動」の黄色バッジの記述は現行の②分類と食い違っている）と 02 の該当節、00 の HEX と DS3。AC = merge 直前に origin/main を取り込み、`primary`・`ring`・`warning-emphasis`・`rank-top`・`row-current`・`toast.info`・`variant="destructive"`・`data-\[state=selected\]`・`has-aria-expanded`（後ろの 3 つは GA1）の rg を再実行して hit を全て分類済みにする（式は Plan で固定）。L3 項目 = before / after の並べ比べ、ランキング 1 位（順位と太字）、「最新」と②分類の「手動」を言い分けられるか、Home の入口 card（3 状態を並べる: 現行 = 琥珀の線と注意の薄い地、lane A の token 反映後に owner の現行の決定を保った形 = 操作の新しい線と注意の薄い地、試し = 操作の線と操作の仲間の薄い地）、日報の取込み済みの badge と Alert の役割、Home の未取込みの知らせの色、操作の塗りと完了の緑を文字と icon で言い分けられるか、進行中の囲みが「まだ終わっていない」と受け取られるか、在庫照会で詳細を開いた行の緑寄りの見え方（owner が違和感を言えば stone へ戻す。3 点目に badge を足さない形もあわせて見る。GA1）、取込みの手順表示のいまのステップが進行中の段 2 で目に留まるか（GA1）、destructive Alert の soft の地と icon の見本（GA1）、badge の文字の太さ（現行 500、GA1）。既存の backlog 項目（card の面色を白へ寄せる、destructive Alert の soft 塗り）と token が重なるため、同じ PR に入れるかを lane A の Plan で決める。**デザインの決まり runtime lane B（書体）**: 書体の実機比較と採用・同梱、依存を足すなら D-030、L3 = 店の PC で店主が通常距離で読み分けられるか。着手条件 = 本 lane の merge。A と別 lane にする理由 = 依存と配布物が変わり、店の PC での確認が要る。**棚卸し画面 D1 の実装**: ㉘ 以降の棚卸し実装 lane と組。着手条件 = 本 lane と lane A の merge（進行中の地・作業中の囲みを使うため）。開始前・中断から戻った画面は canvas で未作成 | Matrix T13 |
| SPEC-DSR-RENEW-D12 迷いやすい場面 | 00 に `## 迷いやすい場面` の節を置き、[下の表](#spec-dsr-renew-d12-迷いやすい場面の答え)の 23 場面（GA1 で 4 行を足した）を、場面・役割・段・現行の実装・移行の列で書く。各行で役割と段が 1 つに決まる（行の中で条件ごとに 1 つを書く場合を含む。部品の状態は「段に数えない」）。00 の節の導入文の「条件は要素・画面で分け、同じ要素の状態で役割を切り替えない」を次へ改める（GA1）: 条件は要素・画面で分ける。部品の一時的な状態（busy・disabled・hover・focus）で役割を切り替えない。業務の状態が変わる要素（在庫状態の badge の在庫切れ / 在庫少 / 在庫あり、記録状態の badge、0 件と 1 件以上で色が変わる件数）は状態ごとに役割表で役割を選ぶ（02 ⑬ の tone family）。表の第 1 列（場面）は、下の表の場面の語がそれぞれ 1 行だけに当たるように書く（Matrix T15。たとえば危険・失敗の Alert の行の第 1 列に「未取込み」を書かない）。試しの行の書き方は D8 の 2 通りに従う。現行の実装の列は component 名と token 名で書き、`src/` の path を書くなら実在するものだけにする（DS1 が検査する） | Matrix T15 |
| SPEC-DSR-RENEW-D13 撤回・置換 | [下の表](#spec-dsr-renew-d13-撤回置換する旧文)の旧文を、改訂後の file で現行の規則として残さない。04 の旧番号対応表の「撤回・置換」列と D-091 に行を写す。旧 04 の原則に紐づかない W（00・01・review-checklist の旧文、W6〜W12）は、対応表の旧 1〜16 の行に混ぜず末尾に別行で置く。Q7 との関係を 04 の新 2 に 1 文で書く: Q7 原則①「色数をむやみに増やさない」と両立する。色は役割の数を上限とし、役割を持たない色を足さない。進行中の色相は、amber 1 色が操作・注意・現在地・1 位を兼ねていた重なりを解くために足すもので、有彩色の系統は 3（amber・green・red）から 4 へ 1 つ増えるだけである | Matrix T17 |
| SPEC-DSR-RENEW-D14 04 からの反映待ちを閉じる | 旧 04 が「batch 1 で 00 を改める」としたまま 00 に反映されなかった 3 点を、2026-09-24 の src 実測（Contract Probe）の多数派で 00 に書く。caption = 14px muted（`text-sm text-muted-foreground`。12px は badge の中だけ。badge 以外の 12px は現行に残り、画面を触る lane が 14px へ寄せる）。ページ余白 = `space-6`（`PageShell` の `p-6`）、`space-8` は大セクション区切りだけ。icon = 12px（badge の中）/ 16px（ボタン・ナビ・Alert・表内・ラベル隣接）/ 20px（見出し・結果の行の隣）/ 24px（Home の入口 card・空状態）/ 32px（待ちの spinner・ファイル選択・画面単位のエラー・dialog の media icon〈`AlertDialogMedia` の svg 既定 `size-8`、消費は `UnsavedChangesDialog`〉）。旧 04 原則 10 の「16 / 20 / 24 の 3 段だけ」と 00 の旧表（ボタン内 16・テーブルセル 20）の食い違いはこの表で置き換える。Writer は icon の各段を `rg` で数え直して PR に記録する | Matrix T16 |

### SPEC-DSR-RENEW-D12 迷いやすい場面の答え

Plan Review round 1 と相談役の検討で、役割表と段の表だけでは答えが 0 か 2 以上になった場面。現行の実装の列は 2026-09-24 の src（Contract Probe）。「試し」の行は lane A の L3 で owner が採るか決め、それまでは現行の実装のまま作る（D8）。ランキング 1 位（owner 了承の試し）は役割と段の列に試しの答えを書く。他の試しの 4 行は役割と段の列に owner の現行の決定を書き、試しの答えを移行の列に置く（Plan Review round 2 の Coordinator 裁定）。GA1 で 4 行（危険・失敗の Alert、詳細を開いた行、取込みの手順の表示、役割色の文字だけの表示）を足した。詳細を開いた行（owner 2026-09-25 了承の試し）と取込みの手順の表示（Coordinator の既定の試し）はランキング 1 位と同じ書き方で、試しであることと出典は移行の列に書く（D8 の (1)）。

| 場面 | 役割 | 段 | 現行の実装 | 移行 |
|---|---|---|---|---|
| focus ring | 操作 | 段に数えない（部品の状態） | `--ring` は `--primary` と同値 | runtime lane A 待ち（`--primary` と同時に動かす） |
| 操作枠（入力欄・outline ボタンの枠） | ふつう・補足 | 段に数えない（DSR-22 の 3:1） | `--border-strong` | 済 |
| 保存中のボタン（部品自身の busy） | 元の役割のまま（保存ボタンなら操作） | 元の段のまま。spinner と「保存中...」の文言を添える（02 ⑥ Spinner） | 入庫・手動販売ほかの保存ボタン、棚卸しの確定ボタン | 済 |
| 待ちの spinner（取込み中・照合中の大きな spinner） | 進行中 | 段に数えない | `Loader2` の `text-primary`（32px） | runtime lane A 待ち |
| 進み具合の棒 | 作業の進み（棚卸し・整合性チェック）= 進行中、比率（月次の部門比率）= ふつう・補足 | 段に数えない | `progress` の棒は `bg-warning` | runtime lane A 待ち |
| checkbox・radio の checked | 操作 | 段に数えない（小さな塗り） | `checkbox` の checked は `bg-primary` | runtime lane A 待ち（`--primary` の値と一緒に変わる） |
| 複数選択の行（上書きする行を checkbox で選ぶ） | ふつう・補足（選択状態、DSR-21） | 段 0（行に色を付けず、部品の checked で示す。現在行に含めない） | 商品取込みの上書き選択列 | 済 |
| Home の入口 card の強調（最重要の入口 1 つ） | 操作（mockup-c、owner 採用 2026-09-11） | 段 2（操作の 1px 線 + 薄い地。地は mockup-c の注意の薄い地で、lane A の L3 までの例外） | `ActionButton` の primary は `border-primary bg-warning-soft` | lane A の L3 で試し（地を操作の仲間の薄い地へ揃える案。採れば 00 を直す） |
| 画面上の確定・削除のトリガー | 操作 | 主要ボタンなら段 4（1 画面 1 つ）、それ以外は DSR-01 の降格 | 危険の塗りのボタンはすべて dialog の中 | 済 |
| dialog の実行ボタン（戻せない操作） | 危険・失敗 | 段 4（DSR-20） | `variant="destructive"` | 済 |
| 日報の取込み済み（二重取込みの防止） | Alert = 危険・失敗、badge = 注意・確認（owner 決定 2026-09-06） | 段 2 | Alert は `destructive`、badge は注意の tone | lane A の L3 で試し（badge を Alert と同じ危険・失敗へ揃える案。採れば 00 を直す） |
| 未取込みの知らせ（日次・月次・Home の前日分） | 日次・月次 = 注意・確認、Home の前日分 = 危険・失敗（owner 決定 2026-09-06） | 段 2 | 日次・月次は注意の Alert、Home は `destructive` の Alert | Home だけ lane A の L3 で試し（注意・確認へ揃える案。採れば 00 を直す） |
| 危険・失敗の Alert（取得失敗・二重取込み・Home の前日分） | 危険・失敗（どの知らせがこの役割かは色の役割表と各場面の行に従う） | 段 2（薄い地 + 役割の base の線 + icon + 文言） | `Alert variant="destructive"` は `bg-card text-destructive` で地と線が無い。Home は icon あり、日報の二重取込みは icon なし | runtime lane A 待ち（soft の地と icon は backlog の既存項目「destructive Alert の soft 塗り + 三角 icon」〈owner 所感 2026-09-15〉と束ねるかを lane A の Plan で決め、見本は lane A の L3 で並べる。役割は変えない） |
| 減衰（押せないボタン・廃番の行・空状態） | 押せないボタン = 元の役割のまま、廃番の行・空状態 = ふつう・補足（役割色を足さない） | 押せないボタン = 元の段のまま opacity で弱める（主要ボタンなら段 4 の 1 つに数える）、廃番の行・空状態 = 段 0 の muted | `disabled:opacity-50`、`EmptyState` の stone | 済 |
| 注意の Alert と注意の badge の線 | 注意・確認 | 段 2（線の濃さは面積で決める: badge は `-border`、Alert・行は base） | Alert は `border-warning`、badge は `border-warning-border` | 済 |
| 完了の知らせ（保存・取込みの成功・反映済み） | 完了 | 段 2 まで（塗りにしない。操作の塗りとは文字と icon で区別する） | 成功の toast、success tone の badge | 済 |
| 現在地（ナビ）と現在行（一覧の 1 行） | 現在地 = 操作、現在行 = 進行中 | 段 2（細いバー）。色で区別せず、置き場所と文言で区別する | どちらも `border-l-primary` | runtime lane A 待ち |
| 詳細を開いた行（在庫照会の行インライン展開 02 ⑫、操作ログの詳細の開閉） | 進行中（操作対象として開いている 1 行、DSR-22） | 段 2（左端のバー + 薄い地。3 点目の文言は直下に開いた詳細と「詳細を閉じる」の類の文言が担い、badge を足さない） | `ProductListTable` は `data-state="selected"` と展開行の `bg-muted`（stone）、操作ログは `TableRow` の `has-aria-expanded:bg-muted/50` | lane A の L3 で試し（owner 了承 2026-09-25。読むだけの照会に緑寄りの色が付く違和感を owner が言えば stone〈ふつう・補足の段 0〉へ戻し、runtime lane A が本書と 02 ⑫ と DSR-22 を直す。3 点目に badge を足さない形も同じ L3 で見る） |
| 取込みの手順の表示（`StepIndicator`、押せないステップの並び） | いまのステップ = 進行中（取り込んでいる作業の現在の段。ナビの現在地〈操作、DSR-21〉ではない）、済んだステップと先のステップ = ふつう・補足 | いまのステップは段 2（薄い地 + 1px 線、番号と名前は太字）、ほかは段 0（済んだステップの番号も muted） | いまのステップは `bg-primary` の塗り、済んだステップは `bg-primary/10 text-primary` | lane A の L3 で試し（Coordinator の既定 2026-09-25。塗りは押すボタンだけ〈段 4〉のため段 2 へ移す案で、いまのステップが目に留まるかを owner が before / after で見る。採らなければ runtime lane A が本行を直し、押すボタンでない塗りを残すなら decision-log に例外として記録する） |
| ランキング 1 位 | ふつう・補足 | 段 0（順位と太字、色を使わない） | 琥珀の pill と `--rank-top-*` の行の地 | lane A の L3 で試し（owner 了承 2026-09-24） |
| 最新・上書き件数の強調（③強調の badge） | 注意・確認（③強調の琥珀 pill、owner 採用 2026-08-20、枠色は owner 決定 2026-09-05。owner の現行の見た目を保つ移行中の置き場所） | 段 2 に数える（badge の塗りの pill で押すボタンでない。lane A の L3 までの例外、D2） | 琥珀の pill（`variant="default"` と `border-warning`） | lane A の L3 で試し（stone の pill と太字 = ふつう・補足の段 2 へ移す案。「最新」と②分類の「手動」を言い分けられるか。採れば 00 を直す） |
| お知らせ一般（手を止めなくてよい知らせ） | ふつう・補足 | 段 0（Alert の既定） | Alert の既定は `bg-card`。`toast.info` は使っていない | 済 |
| 役割色の文字だけの表示（入力欄のエラー文・増減の ±・在庫少と在庫切れのセルの数字・Home のサマリの件数・記録詳細のリンク） | エラー文と − と在庫切れ = 危険・失敗、+ = 完了、在庫少 = 注意・確認、リンク = 操作（0 件の件数は色を付けず、ふつう・補足） | 段に数えない（文字色は補助の信号。記号・文言・下線のどれかを必ず添え、icon は要しない。DSR-08） | `FieldError` の `text-destructive`、± の `text-success-strong` / `text-destructive-strong`（DSR-08）、セルと件数の `text-warning-emphasis` / `text-destructive`、リンクの `text-primary` と hover の下線 | 済（リンクの `text-primary` は runtime lane A 待ちで、`--primary` の値と一緒に変わる） |

### SPEC-DSR-RENEW-D13 撤回・置換する旧文

旧文の場所は改訂前（`dda8560a`）。「置換後」は改訂後の置き場所。

| # | 旧文の場所 | 旧文の要旨 | 扱い | 置換後 |
|---|---|---|---|---|
| W1 | 04 原則 2 | 色は 3 家族。info 家族は作らず、お知らせ・注意喚起は warning トーン | 置換 | 色は役割で使う（新 2、00 の役割表）。手を止めて確かめてほしい知らせは注意・確認、手を止めなくてよいお知らせ一般はふつう・補足（Alert の既定）。進行中は info ではない。soft・border・strong の段の形は 00 の token 表に残す |
| W2 | 04 原則 4 の③ | ③強調 = 琥珀 pill（ランキング 1 位 / 最新） | ランキング 1 位だけ置換 | ランキング 1 位は色を使わず順位と太字（新 4、owner 了承の試し、移行中は現行の琥珀 pill）。最新・上書き件数の琥珀 pill は owner の現行の決定として新 4 に残し、stone の pill と太字へ移す案は D12 の試し |
| W3 | 04 原則 5 | 1 画面に primary（琥珀塗り）は 1 つ | 色の語だけ置換 | 1 画面に操作の塗りは 1 つ（新 3、DSR-01）。「琥珀」は 00 の現行の実装の列にだけ残す |
| W4 | 04 原則 13 | 新しい色相は追加しない（Q7 原則①） | 置換 | 役割の無い色を足さない、色は役割の数を超えない（新 2 と新 9、D13 の Q7 の 1 文） |
| W5 | 04 原則 15 | 左 4px の primary バー + `--row-current` + badge または文言 | 置換 | 現在行は進行中の役割（新 10、DSR-22）。3 点表示は維持し、色は lane A で進行中の token へ。現行は primary のバーと `--row-current` |
| W6 | 00 セマンティックカラー表の Primary 行の用途 | 主要ボタン、ハイライト。ウォーム系主アクセント | 用途の列だけ置換（行と HEX は不変） | 操作（押すボタン・リンク・ナビの現在地）。「ハイライト」は撤回（③強調の琥珀は操作の色でなく注意・確認の色で表す、D12） |
| W7 | 00 4 色エリアモデルの節 | サイドバーはアクティブ項目のみ Primary アクセント 1 色 | 置換 | ナビの現在地は操作の役割色の細いバー（段 2、DSR-21）。単色 stone のサイドバーと「色でなく構造で区分」は維持 |
| W8 | 00 ウォーム系採用の論拠の節 | UI の色を暖色でそろえる | 一部置換 | ニュートラルの stone はウォームのまま。役割色は色相で役割を分ける |
| W9 | 00 書体 | カスタム Web フォントは読み込み遅延のリスクで不採用 | 撤回 | D3 の選定条件と runtime lane B での採用 |
| W10 | review-checklist カテゴリ 9 の badge の行 | ③強調 = 琥珀 pill + `--warning` 枠 | 一部置換 | 新 4 に合わせる（ランキング 1 位は順位と太字の試し、最新・上書き件数は現行の琥珀 pill）。移行中は現行の部品の形だけ可 |
| W11 | DSR-21 の Why | Primary（amber-700）は warning 系と同系色 | 現状の説明として残す | runtime lane A までの現状と書き、役割表で色相が分かれることを足す（D7） |
| W12 | 00 セマンティックカラー表の Warning 行の用途 | PLU通知、手動バッジ、在庫少 | 用途の列だけ置換（行と HEX は不変） | 「手動バッジ」を外す。実物の「手動」は②分類の `variant="secondary"`（`ProductTable.tsx`）で段 0 |
| W13 | 04 原則 7 と token 表の検索欄の行 | 検索欄は live 型 + 検索ボタン併記、02 ⑨ を併記の単一形へ | 撤回（owner 2026-09-24） | 検索欄は live 型でボタンなし（新 7、02 ⑨ の現行、処分表 U9） |

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-DSR-RENEW-D1 | S1 | T5・T7 | owner の判断との一致 | full doc check 出力、`rg` 出力 |
| SPEC-DSR-RENEW-D2 | S1 | T6 | DSR-16 との整合 | `rg` 出力 |
| SPEC-DSR-RENEW-D3 | S1 | T8 | 採用前であることの明示 | `rg` 出力 |
| SPEC-DSR-RENEW-D4 | S1 | T5 | 言い切るラベル | `rg` 出力 |
| SPEC-DSR-RENEW-D5 | S2・S3 | T9・T10 | 内容の落ち | 旧番号対応表、hit の分類記録 |
| SPEC-DSR-RENEW-D6 | S4 | T3・T4 | 番号と見出しの不変 | `diff` 空出力 |
| SPEC-DSR-RENEW-D7 | S4 | T11 | DSR 間の一貫性、選択行の限定、現在地と現在行 | hit の分類記録 |
| SPEC-DSR-RENEW-D8 | S1・S5・S6・S7 | T7・T18 | 候補値の置き場、移行中の作り方 | `rg` 0 件、README と checklist の `rg` 出力 |
| SPEC-DSR-RENEW-D9 | S1〜S10 | T1・T2 | gate を弱めない | DS1〜DS4 の出力行 |
| SPEC-DSR-RENEW-D10 | S9 | T7 | 候補値・計算結果・試し・却下案 | D-091 |
| SPEC-DSR-RENEW-D11 | S10 | T13 | Human Gate の載せ先、lane A の進め方 | backlog entry と T13 の loop 出力 |
| SPEC-DSR-RENEW-D12 | S1 | T15 | 答えが 1 つに決まるか、今の赤を変えていないか | 場面の `rg` 出力 |
| SPEC-DSR-RENEW-D13 | S1・S2・S4・S7 | T17 | 旧文の残り | hit の分類記録 |
| SPEC-DSR-RENEW-D14 | S1・S2 | T16 | src の実測との一致 | `rg` 出力と icon の数え直しの記録 |

## Data Safety

- 書かないもの: canvas の URL、`.local/` の path の詳細、引継ぎ書・台帳の発言原文（短い語句の引用を含む）、店の実データ（JAN・実商品名・価格・原価・売上・取引先名・個人名）。canvas の例示の商品名も tracked file に写さない（例が要るときは「商品名」「前回数えた数」等の一般語を使う）。
- local-only: 引継ぎ書、canvas の scratchpad への保存物。
- synthetic-only: 例示の数値（前回 14 等）は架空の値として扱い、実データと混同しない書き方にする。

## Test Design Matrix

[Test Design Matrix](test-matrices/2026-09-24-design-rules-renewal.md)

## Implementation Results

[PR #98](https://github.com/higekiri-w/inventory-system-desktop/pull/98) で実装し squash merge 済み（`0aae02da`、2026-09-25）。S1〜S10 を文書だけで反映した: `docs/design-system/00-foundations.md` に色の役割表（操作 / 進行中 / 注意・確認 / 完了 / 危険・失敗 / ふつう・補足の 6 役割。見せ方・受け取り方・狙う効果・根拠・現行の実装・移行の列）、強調の段階（段 0〜4 と段に数えないもの）、迷いやすい場面（23 行。試しの行は移行の列に出典を置く）を置き、既存の token 表に役割の列を足した（HEX と行は不変）。`docs/design-system/04-backbone.md` の 16 原則を 11 本へ統合し、旧番号の対応表（旧 1〜16 と旧原則に紐づかない W）と旧 04 の反映待ちの行き先を置いた。03 を原則を持たない出典の一覧にし、01 の DSR を話題別に並べ直して役割表と矛盾する記述を直し、README を読む順・迷ったらここ・移行中の読み方と作り方にした。02・review-checklist・`docs/UI_TECH_STACK.md` を合わせ、`docs/decision-log.md` に D-091 を記録した。`docs/backlog.md` に後続 3 entry（runtime lane A〈色と強調〉、runtime lane B〈書体〉、棚卸し画面 D1 の実装）と旧 04 の未了の見た目の作業の 1 行を起票した。Final Review broad round 1 の 13 件は Gated Amendment 1（`cbcbabc8`）で契約を直して Writer が反映し（`d328f1f1`〜`2f02ee7a`）、GA1 後の broad の 7 件は Amendment なしで `68c152cf` / `f60cee69` で是正した。token の値・部品・画面は変えていない。

## Review Response

- Findings Freeze: not yet frozen; post-freeze exceptions: none.
- Plan Review round 1（2026-09-24、fresh Opus 5.5 の read-only review、対象 `85e087f5`）: 冒頭 3 値 = Ordinary Operation の行 1 / 4 / 5 / 6 成立、行 2（役割と段を選ぶ）・行 3（今の見た目を確かめる）・行 7（旧番号を引く）具体的な反例あり。P1 0 / P2 7 / P3 7。Coordinator 裁定（2026-09-24）= 全件採用。反映先:
  - P2-1 旧 15 の 2 規則 → D5 の対応表（旧 15 → 新 10 と新 5）、D4 の適用範囲 (a)(b)、S1 の metric 行の参照先、Ordinary Operation の行 3・行 7。
  - P2-2 撤回・置換する旧文 → D13 の表（W1〜W11）と Q7 との関係の 1 文、お知らせ一般の行き先（D1 のふつう・補足、D12）、AC16、Matrix T17。
  - P2-3 今の赤 → 役割名を「危険・失敗」とし、危険・戻せないと失敗・欠けの 2 場面を 1 行に持たせた（D1）。今の割り当てを変えない（Goal・D1・D7）。完了の例から「取込み済み」を外した（D1、AC5）。
  - P2-4 ③強調 → 変えるのはランキング 1 位（owner 了承の試し）で、最新・上書き件数も試し案として lane A の L3 に載せた（D5 の新 4、D12、D11 の L3 項目）。
  - P2-5 移行中の作り方 → D8、README の `移行中の作り方`、review-checklist の許容の限定（S7）、D11 の棚卸し D1 の着手条件、Ordinary Operation の行 2、AC17、Matrix T18。
  - P2-6 バー → D2 の段 2 の一形態。段 1 は役割の領域だけ、ただのまとまりは段 0 の stone（AC6）。
  - P2-7 04 から 00 への反映待ち → D14（src の実測で caption 14px・ページ余白 `space-6`・icon 12 / 16 / 20 / 24 / 32）、AC15、Matrix T16、Non-scope（badge 以外の 12px）。
  - P3 ランキングの出典 → 設計判断の出典に owner の了承（試し）を追加。P3 BIZ UDPゴシック → D3・Impact Review Lenses・Ordinary Operation の行 5 で、確かめていない外部の事実として扱う。P3 D9 の走査範囲 → D9 に `docs/*.md` を追加。P3 DSR-22 の候補値由来の数値 → D7 で DSR-22 に比を書かず D-091 へ置き、AC7 に比の検査を追加。P3 focus ring と操作枠 → D2 の「段に数えない」。P3 owner の短い言い回し → 設計判断の出典と Design Intent Trace を言い換えた。P3 T13 の検出力 → AC13 と Matrix T13 を、題の文字列と行ごとの必須語の loop に強めた。
- 相談役の検討（Fable 5.1、read-only、2026-09-24）: src に当てた 12 場面・受け取り方の混乱 3 点・移行中の混在 3 経路・P2-7 の実測。Coordinator 裁定 = 全件採用（1-6 = Home の入口 card を操作の段 2 にする試し、1-9 = 日報の取込み済みの badge を Alert と同じ役割へ揃える試し。どちらも lane A の L3 で最終）。反映先: 1-1・1-4・1-5・1-11・1-12 → D2。1-2・1-3・1-7・1-8・1-10 → D1・D12（1-7 は D7 の DSR-22 にも）。1-6・1-9 → D12 の試しの行。2-1・2-2 → D2・D7・D12。2-3 → D11 の L3 項目と D12。3-1・3-2 → D11。3-3 → D8。P2-7 の実測 → D14 と Contract Probe。
- 起草役の再測で分かった食い違い（実物を正とした）: 棚卸しの「確定しています」は `StocktakePage.tsx:319`（相談役は 317）。危険の塗りのボタンは相談役が挙げた `AlertDialogAction` 4 site のほかに `AlertDialogAction` 5 site と取引先の統合 dialog の `Button` 1 site があり、すべて dialog の中で 1-8 の結論は変わらない。primary の消費数と caption の数は数え方で揺れる（Contract Probe）が、結論は変わらない。日報の取込み済みの badge が注意、Home の前日分の未取込みが危険なのは owner 決定 2026-09-06（`02-component-catalog.md` の ⑬ と ⑥）で、1-9 と 1-10 の向きはこの既存決定と逆になるため、試し（owner が採らなければ現行のまま）として扱った。
- 起草役が新たに決めたこと（round 2 で見てほしい点）: 役割名「危険・失敗」。D12 の 19 場面の答え（とくに操作枠 = ふつう・補足、お知らせ一般 = 段 0、最新と上書き件数 = stone の pill で段 2）。D14 の icon の 5 段。token の分割 merge の却下（D11）。試しを owner が採らなかった場合は lane A が 00 を現行に合わせて直す規則（D8）。
- Plan Review round 2（2026-09-24、fresh Opus 5.5 の read-only review、対象 `fe43d11b`）: 冒頭 3 値 = Ordinary Operation の行 2 だけに具体的な反例あり（軽微、P3-2・P3-3・P3-4 に対応）、他の行は成立。round 1 の P2 7・P3 7 と相談役の 18 件はすべて閉鎖（根拠行つき）。DS1 44 / DS2 24 / DS3 27 / DS4 22 は起草時の記録と一致。P1 0 / P2 2 / P3 7。owner への論点 3 つ。Coordinator 裁定（2026-09-24）= P2・P3 は全件採用（文の整合の直しで設計の差し戻しではないため相談役なし）。論点 (1) は既定を反転する。論点 (2)(3) は owner に確認した。反映先:
  - P2-1 旧 04 の未反映の約束 → Scope の「旧 04 の未反映の約束の処分」表（U1〜U13）と S2・S10、D5 の新 2・7・8、D7 の DSR-01、D13 の W13、AC9、Matrix T9。
  - P2-2 各行の役割と段 → AC14・D12 の契約文・Ordinary Operation の行 2 を「各行で役割と段が 1 つに決まる（行の中で条件ごとに 1 つを書く場合を含む）」へ。Matrix T15 を合わせた。
  - P3-1 AC11 → T11 と同じ式と分類へ。P3-2 ②分類の badge → D2 の段 0（AC6・T6）。P3-3 押せないボタン → D2 と D12 の減衰の行で、元の段のまま opacity、廃番の行と空状態は段 0。P3-4 → D2 の段 2 の線に Home の入口 card。P3-5 → D13 の W12、Non-scope の `docs/SCREEN_DESIGN.md` の記述、D11 の lane A の対象。P3-6 → S4（01:438）、S6（02:889・1016）、S8（`docs/UI_TECH_STACK.md` 51 行目）、D14 の 32px（dialog の media icon）、Matrix の Adjacent Pattern Audit。P3-7 → D1 の危険・失敗の「日報の二重取込み」を Alert に限り、危険・失敗とふつう・補足の受け取り方と根拠を指定した。
  - 論点 (1) 試しの 4 項目（Home の入口 card、日報の取込み済みの badge、Home の前日分の未取込み、最新と上書き件数）→ 役割の列を owner の現行の決定にし、試しの答えを移行の列へ移した（D12 の表、D1 の注意・確認と危険・失敗、D2、D5 の新 4、D13 の W2・W6・W10、Goal、Design Intent Audit、設計判断の出典）。D8 の規則は、ランキング 1 位は「採らなければ 00 を現行へ直す」、4 項目は「採れば 00 を試しの答えへ直す」の 2 通りにした。
  - 論点 (2) owner 回答（2026-09-24）= 検索欄は今のボタンなしで確定 → 旧 04 原則 7 の検索ボタン併記は撤回（処分表 U9、D13 の W13、D5 の新 7）。
  - 論点 (3) owner 回答（2026-09-24）= lane A は 1 PR でよい → D11 に出典として追記。
- 起草役の再測で分かった食い違い（round 2、実物を正とした）: reviewer の挙げた file:line はすべて実物と一致した（04:48-53、00:32、02:889・1016、01:438、`docs/UI_TECH_STACK.md` 51、`alert-dialog.tsx` 122 の media の svg 既定 `size-8`）。reviewer の挙げていない旧番号参照は、`docs/function-design/59-ui-shared-patterns.md`・`src/test/page-root-pageshell-sweep.test.ts`・`PageShell.test.tsx`・`PageShell.tsx` 3 行目（いずれも原則 6）、review-checklist 70 行目（原則 1）、01:456（原則 6）で、新番号も同じ主題のため変えない（Matrix の Adjacent Pattern Audit）。旧 04 の約束のうち reviewer が挙げていないもの（部門 select の幅の統一、sidebar ラベルの折返し、未使用の `src/App.css` の撤去、操作目標 min-height 40px）は処分表 U12・U13 で backlog へ送った。
- 起草役が新たに決めたこと（round 3 で見てほしい点）: ③強調の琥珀 pill を owner の現行の決定として注意・確認の役割に置き、badge の塗りだが段 2 に数える例外（D2・D12）。Home の入口 card の役割を操作とし、注意の薄い地を lane A の L3 までの例外とした（D12）。lane A が `--primary` の値を変えても最新・上書き件数の badge が琥珀のまま残るよう、注意の token へ移す作業を D11 に足した。
- Plan Review round 3（天井、2026-09-24、fresh Opus 5.5 の read-only review、対象 `0026386e`）: 冒頭 3 値 = Ordinary Operation の全行成立。round 2 の P2 2・P3 7・論点 (1)〜(3) はすべて閉鎖。DS1 44 / DS2 24 / DS3 27 / DS4 22 は起草時の記録と一致し、D-091 は main と並走 lane で空き。round 3 で見る点 (a)〜(e) は両立（(e) は 1 文を足す）。P1 0 / P2 0 / P3 10、Plan Gate 可。Coordinator 裁定（2026-09-24）= 全件採用。Plan Gate 前にまとめて直すもの（本 commit）と Writer run で扱うもの（S4・S7 に指示を足した）に分けた。反映先:
  - P3-1 AC9 の 03 の式（`rg -n '核心4本柱|補助3原則'`）→ 更新履歴の行を除き 0 件（AC9、Matrix T9）。AC10 の分類に「過去の記録の行」（backlog の owner 回答の記録）を足した（AC10、Matrix T10）。
  - P3-2 旧 04 原則 8 と適用の順序 2 段目の右端 chevron → 処分表 U14（未了、backlog へ）、S10、AC9、Matrix T9 と Adjacent Pattern Audit。
  - P3-3 Design Intent Audit の「例外は置かない」と「移行中の例外 2 点」の食い違い → 恒久の例外は置かず、移行中の例外 2 点は L3 で試しを採らなければ lane A が D-091 に恒久の例外として記録する（Design Intent Audit、D8）。
  - P3-4 D12 の③強調の行の役割の列 → owner の現行の見た目を保つ移行中の置き場所と明記。
  - P3-5 D11 の L3 項目 → Home の入口 card の 3 状態（現行、owner の現行の決定を保った token 反映後、試し）を並べる。
  - P3-6 ③強調の badge の移し先 → `--warning-emphasis` と名指し（D11）。`--warning` を採らない理由の比は Contract Probe と D-091 に置き、正本には書かない（D10）。
  - P3-7 ランキング 1 位を採らないときの直し先 → 00 に加えて 04 の新 4・DSR-08・review-checklist の badge の行（W10）（D8、Design Intent Audit）。
  - P3-8 旧 04 の原則に紐づかない W（W6〜W12）→ 旧番号対応表の末尾に別行（D13、AC9、Matrix T9）。
  - (e) AC14・Matrix T15・D12 の契約文に「条件は要素・画面で分け、同じ要素の状態で役割を切り替えない」を足した。
  - P3-10 Owner Effort Budget → 消費済みの介入（ランキングの了承と、検索欄・lane A への回答）を記録し、`介入回数上限` を 3 から 5 へ（理由は同節）。
  - Writer run で扱うもの: P3-9 は S7 の review-checklist の 1 行に「役割の無い色・00 に登録されていない token を使っていないか」を足す指示。DSR-21 の Why（01 の 428 行付近）は W7 と同じ commit で直す指示を S4 に足した。
- 起草役の再測で分かった食い違い（round 3、実物を正とした）: reviewer の挙げた file:line（`docs/UI_TECH_STACK.md` 840、`docs/backlog.md` 235、01:428、旧 04 の 24・58、本 packet の Design Intent Audit と D12 の③強調の行）はすべて実物と一致した。ただし `docs/UI_TECH_STACK.md` 840 行目は AC9 の式（`核心4本柱|補助3原則`）に当たり、AC10 の式（`原則 ?[0-9]+`）には当たらない。AC10 の式には `docs/backlog.md` 235 行目に加えて 237 行目（同じ owner 回答の記録）も当たる。このため 840 行目は AC9 の除外に、235・237 行目は AC10 の「過去の記録の行」に置いた。比の値は `--primary-foreground` に対し 4.81:1 と 3.05:1 で、reviewer の値と一致した。一覧の行の右端 chevron は src に無い（`Chevron` の hit は選択ボタンと select の開く印・accordion・pager・操作ログの詳細の開閉だけ）。
- Final Review broad round 1（2026-09-25、対象 `5ae34b7d`、2 本とも互いに独立）: Opus（fresh Opus 5.5 の read-only review）は冒頭 3 値 = Ordinary Operation の行 2（役割と段を選ぶ）に具体的な反例あり（F1・F2）、他の行は成立。P1 0 / P2 2 / P3 5。Codex（PR review〈Contract Audit〉と inline 6 件）は冒頭 3 値 = 行 2 に #1・#2、行 3（移行中の読み分け）に #3 の反例あり、書体の外部前提は未確認だが lane B に明示済みで追加の blocker にしない。P1 0 / P2 3 / P3 3。AC1〜AC17 と T2 は両方とも構造を満たす（DS1 44 / DS2 24 / DS3 27 / DS4 23、full と `--target plan` は exit 0）。Opus の F2 は owner の見た目の判断に当たるため owner に尋ね、owner 判断 F2（2026-09-25、設計判断の出典）を得た。相談役 Fable 5.1（read-only）が 13 件を 5 つの根 A〜E に束ね、是正の最小形と反例を示した。Coordinator 裁定（2026-09-25）= 13 件を全件採用、design への backtrack はしない、Gated Amendment 1 で packet と Matrix を直し、Writer が正本へ反映する。
- Final Review broad round 1 の裁定（根ごとの反映先。契約の文は Spec Contract の各 D にある）:
  - A 役割色の文字だけの表示（Opus F1、Codex #2）→ D12 の表の「役割色の文字だけの表示」の行、D2 の「段に数えない」、D1 の「文字か記号を添え」の文と操作のリンクの 1 句と注意・確認・危険・失敗の見せ方の末尾の 1 句、Ordinary Operation の行 2。
  - B 移行の列の読み分け: Codex #3（destructive Alert を「済」としていた）→ D1 の危険・失敗の移行（Alert は runtime lane A 待ち）、D12 の表の「危険・失敗の Alert」の行、D11 の `alert.tsx` と AC の `variant="destructive"`。Opus F2（詳細を開いた行、owner 判断 F2）→ D12 の表の「詳細を開いた行」の行（進行中、試しの出典は移行の列）、D1 の進行中、D7 の DSR-22 の 1 句、D8 の (1)、S6 の例外（02 ⑫ の 1 文）、D10、D11 の対象・AC・L3。
  - C 1 つの要素に 2 つの役割: Codex #1（同日追加確認）→ D1 の危険・失敗と注意・確認の使う場面。Codex #4（状態による役割切替の射程）→ D12 の契約文と AC14 の状態切替の文を部品の一時的な状態に限り、業務の状態は状態ごとに選ぶと書いた（Matrix T15 も同じ文）。Opus F5（`StepIndicator`）→ D12 の表の「取込みの手順の表示」の行（Coordinator の既定の試し）、D8 の (1)、D11 の対象と L3。
  - D 検査の検出力（Opus F4、Codex #5）→ Matrix T13（lane A の行の「対象 =」から「。AC =」までを切り出して検査）、T15（表の第 1 列の行数 23 と、場面の語のそれぞれが第 1 列の 1 行だけに当たること）、変異の見込みの 2 文、AC13・AC14。
  - E 統合時の記録漏れ・文の不整合: Codex #6（badge の weight 600）→ D5 の反映待ちの表の 1 行と D10（現行 500 で置換）。Opus F7（03 と 04 の根拠の不一致）→ D5（03 の列と 04 の旧 03 の表を 04 の根拠の行から写す。列は残す）。Opus F6（hover 背景の脱落）→ D5 の新 7、処分表 U14、backlog の旧 04 の未了の行。Opus F3（checklist の段 3・4）→ S7。
- Final Review broad round 1 の是正の起草で相談役の案から変えたこと（GA1 の起草、2026-09-25）: (a) 「badge と Alert の icon は DSR-22 と 02 ⑬ ⑥ に従う」の DSR-22 を DSR-08 へ直した（badge の icon と日本語の文言を定めるのは DSR-08 で、DSR-22 は一覧の器と現在行）。(b) 詳細を開いた行の試しの出典は、相談役の案の役割の列でなく移行の列に置いた（owner 判断 F2 の書き方）。取込みの手順の表示も同じ書き方にした。(c) badge の weight の置換の文を「600 は `AlertTitle` だけ」から「600 は見出しと `AlertTitle` が使い、badge には使わない」へ直した（00 のタイポグラフィ表の見出しも 600）。(d) 相談役が任意とした DSR-22 の 3 点目の 1 句は足す（DSR-22 の「3 点」の字面と 00 の行が食い違うと、GA1 の後の broad が同じ点を再び指摘するため。owner の「据え置き」は開いている行の役割の割り当てについてで、3 点目の細則は別）。(e) 00 に登録されていない token の既存の drift（`IntegrityCheckPage` の `text-warning-foreground`）は runtime lane A の対象に 1 句で入れた。(f) 候補一覧の active 行（`ProductAddSuggest` の `bg-accent`、stone）は指摘に無いため足さない。
- Final Review broad round 1 の Writer への反映先（Writer は下の file と節だけを直し、文は Spec Contract の該当 D に従う。行番号は `5ae34b7d` の実物）:

  | file | 節・箇所 | 直す内容 | 契約 |
  |---|---|---|---|
  | `docs/design-system/00-foundations.md` | 色の役割表の操作の行（16 行目） | 使う場面か見せ方に「リンクは文字色と hover の下線」 | D1 |
  | 同 | 色の役割表の進行中の行（17 行目） | 使う場面に「詳細を開いた行（試し、迷いやすい場面）」 | D1 |
  | 同 | 色の役割表の注意・確認の行（18 行目） | 使う場面の「同日データあり」を「同日データあり（同日追加確認の Alert と badge、DSR-03）」へ。見せ方の末尾に「文字だけの形は迷いやすい場面の行に従う」 | D1 |
  | 同 | 色の役割表の危険・失敗の行（20 行目） | 使う場面の「進めない・二重計上の防止（…）」を「進めない（日報の二重取込みの Alert・取得失敗。DSR-03 のデータ安全系のうち止めるもの）。確認すれば進める同日追加確認は注意・確認」へ。見せ方の末尾の 1 句。移行の列を「badge と dialog の実行ボタンは済。Alert は runtime lane A 待ち（迷いやすい場面）」へ | D1 |
  | 同 | 役割表の下の箇条（23 行目） | 「どの状態も文字か記号を添え、色だけに頼らない。badge と Alert の icon は DSR-08 と 02 ⑬ ⑥ に従う」へ | D1 |
  | 同 | 強調の段階の「段に数えない」の列挙（46〜50 行目） | 「役割色の文字だけの表示（迷いやすい場面の行に従う。上限は置かない）」を足す | D2 |
  | 同 | 迷いやすい場面の導入文（60 行目） | 状態切替の文を D12 の文へ | D12 |
  | 同 | 迷いやすい場面の試しの書き方（64・65 行目） | 1 つ目の箇条を「ランキング 1 位と詳細を開いた行（owner が試しとして了承済み）と取込みの手順の表示（Coordinator の既定）」の書き方と直し先へ | D8 |
  | 同 | 迷いやすい場面の表 | packet D12 の表の 4 行（危険・失敗の Alert、詳細を開いた行、取込みの手順の表示、役割色の文字だけの表示）を足し 23 行に | D12 |
  | `docs/design-system/04-backbone.md` | 原則 7（35 行目） | 押せる行は `cursor-pointer` と hover 背景（`TableRow` の既定）を持つ、の 1 文を右端 chevron の文の前へ | D5 |
  | 同 | 旧番号の対応表の旧 8 の行、旧 04 の反映待ちの行き先の最終行 | 「右端 chevron（hover 背景は反映済み）」へ | D5 |
  | 同 | 旧 04 の反映待ちの行き先 | badge の weight の行（12px と pill は 02 ⑬ と `badge.tsx`、weight は 600 を現行 500 で置換） | D5 |
  | 同 | 旧 03 の節の行き先の表 | 番号を 04 の各原則の「根拠:」行から写し直す | D5 |
  | `docs/design-system/03-philosophy.md` | 出典の一覧の「主に根拠にする原則（04）」列 | 04 の各原則の「根拠:」行から写し直す（04 は変えない） | D5 |
  | `docs/design-system/01-decision-rules.md` | DSR-22 の現在行の段落（136 行目） | 詳細を開いた行の 3 点目は開いた詳細と「詳細を閉じる」の類の文言が担い badge を足さない（試し、00 の迷いやすい場面）、の 1 句 | D7 |
  | `docs/design-system/02-component-catalog.md` | ⑫ 行インライン展開の「使用トークン」（827 行目） | 末尾に狙いの 1 文だけ（他の記述は変えない） | D8・S6 |
  | `docs/quality/review-checklist.md` | カテゴリ 9 の色の役割と強調の段の行（88 行目） | 「段 3 と段 4 は 1 画面に 1 つ」を「段 3 は 1 画面に 1 か所、段 4 は操作の塗りを 1 画面に 1 つで dialog の危険の塗りは数に入れない（DSR-20）」へ。DSR 参照を保つ | D2・S7 |
  | `docs/decision-log.md` | D-091 | 場面の数 23、試しの列挙 2 つ、badge の weight の置換 | D10 |
  | `docs/backlog.md` | runtime lane A の行（51 行目） | 対象に `alert.tsx`・詳細を開いた行（`table.tsx`・`ProductListTable`・操作ログ）・`StepIndicator`・`IntegrityCheckPage` の未登録 token、AC の rg に `variant="destructive"`・`data-\[state=selected\]`・`has-aria-expanded`、L3 項目に詳細を開いた行・手順表示・destructive Alert の見本・badge の文字の太さ | D11 |
  | 同 | 旧 04 の未了の見た目の作業の行（54 行目）の (e) | 「押せる行の右端 chevron（hover 背景は `TableRow` の既定で全行に付いており、押せない行にも出るため、押せる印としては chevron か cursor が担う。採るかを含めて決める）」へ | 処分表 U14 |

- Final Review broad round 1 の後の閉じ方: Writer の反映の後、Final Review broad を 2 本（fresh Opus と Codex）取り直す。理由は Workflow State の遷移記録の Gated Amendment 1 の段落（`scripts/pr-gate.py` が Amendments の変わった broad を拒む）。Codex の取り直しは介入の上限 8 を 1 回超えるため、発注の前に Coordinator が owner に確かめる（Owner Effort Budget）。

### Final Review broad（GA1 後、2026-09-25、Codex と Opus 5.5 fresh、対象 `2f02ee7a`、裁定 Coordinator）

- 結果: Codex P1 0 / P2 1 / P3 0、Opus P1 0 / P2 0 / P3 6。
- Codex #1（P2、`FieldError` のような文字だけの表示を 00:94 は icon なしで適合とし、04 原則 2 は icon + 日本語 + 色の 3 点を一律に求めるため、同じ部品の適合の答えが 2 つになる）= 採用。D5 の原則 2 は「色だけに頼らない」だけを契約にしており、3 点の一律の要求は本文の書き過ぎなので、Amendment なしで本文を D5 と 00:23 へそろえる。04 原則 2 と受け取り方の文、旧番号対応表の旧 3 の行、01 DSR-08 の冒頭と Why を「文字か記号を添え、色だけに頼らない。icon の必須は非中立の状態 badge と Alert」へ（Writer `68c152cf`）。
- Opus N1（P3、`TableRow` の既定 `has-aria-expanded` が行の中の選択欄を開いた行にも当たる）= 採用。00 の詳細を開いた行の現行の実装の列と、backlog の lane A の対象に明示の class の 1 句（`68c152cf`）。
- Opus N2・N3・N4（P3、取込みの手順の結果のステップ、月次売上の前月比のセル、失敗でない赤の知らせの役割と段が決まらない）= 採用、行き先は lane A の Plan。00 の迷いやすい場面の表は D12 の 23 行を契約にしているため行を足さず、backlog の lane A の対象に 3 つを名指しし、L3 項目に済んだステップと先のステップの見分けを足した（`68c152cf`）。
- Opus N5（P3、T13 が `has-aria-expanded` を行全体で確かめるため、AC の rg の並びから語が抜けても通る）= 採用。Matrix T13 で「AC =」から「。L3 項目」までを切り出して AC の rg の 3 語を確かめる。AC13 の文面は変えない（検査を厳しくするだけ）。語を抜いた変異で red を確認した。
- Opus N6（P3、試しを採らないときの直し先が DSR-22 の 3 点目の句だけで、「開いている行」の部分が残る）= 採用。00 の直し先を DSR-22 の詳細を開いた行の 1 文全体にし、採らないときは DSR-22 と 04 原則 10 の「開いている行」を「入力や編集のために開いている行」と書き分けると足した（`68c152cf`）。
- 閉じ方: Plan Commit と Amendments は変わらないため、`merge-evidence.md` の closure 規則どおり、保持した broad 2 本に現在 head の独立 closure 1 本を足して閉じる。closure は Fable 5.1（Writer とも Coordinator とも別 context、owner 決定 2026-09-25: 2 周目以降は Codex の修正案を Claude 側で裏取りして取り入れ、Claude 側の独立 reviewer で締める）。確認条件は Codex #1 の「新規画面で 02 ④ の `FieldError` を置いたとき、00・01・04 のどれを読んでも適合の 1 つの答えになる」。

Final Review の closeout 時点の結論（2026-09-25、Coordinator）: helper の専用 record（`#issuecomment-5822866485`）の review は pass。broad は GA1 後の 2 本で、互いに独立した Double Audit。(1) Opus 5.5 fresh の broad（対象 `2f02ee7a`、`#issuecomment-5822846399`、P1 0 / P2 0 / P3 6。N1〜N6 を全件採用し `68c152cf` / `f60cee69` で是正）。(2) Codex broad（GPT-6 Astra high、対象 `2f02ee7a`、`#pullrequestreview-5308866419`、P1 0 / P2 1 / P3 0。#1 を採用し `68c152cf` で是正）。Plan Commit と Amendments が変わらないため、2 本に現在 head の独立 closure として Fable 5.1（対象 `f60cee69`、`#issuecomment-5822846708`、新規 P1 0 / P2 0 / P3 3。Codex #1 の確認条件を 00・01・04・02 に当てて 1 つの答えになること、Opus N1〜N6 の閉鎖、AC の 3 語を抜く変異が red になることを確認）を足した。GA1 前の broad 2 本（対象 `5ae34b7d`）は `amendments` が空で GA1 の後は使えないため record に入れない（裁定は上の round 1 の節）。record の closure は base 同期後の fresh Opus（対象 `ce6c34f4`、`#issuecomment-5823183852`、新規 P1 0 / P2 0 / P3 0）。manual / R4 は not-required。1 回目の base 同期 `d5c4709d`（origin/main `e40e50b9` = PR #93 の merge の取込み）は #93 の closeout（PR #101）の前で、#93 の active packet が `docs/plans/` に混ざり `scripts/check-workflow-git.sh`（PK5）が fail した。PR #101 の merge 後の origin/main `dd10758f` を `-X no-renames` で取り込み直した（`ce6c34f4`。既定の rename 検出は本 lane の packet を #93 の packet の移動と取り違え、`--remerge-diff` で 18719 行の誤差分を出す）。取込み後の `docs/plans/` は本 lane の packet と Matrix だけで、`f60cee69..ce6c34f4` の `docs/plans/` の差分は 0、`docs/backlog.md` の衝突は両側の行を合わせて解消した（base 同期 closure で確認）。Fable closure（`f60cee69`）の新規 P3 3 件は head を動かさず本 closeout で処理した: C1（DSR-08〈`docs/design-system/01-decision-rules.md` 482 行〉・04 原則 2・旧番号の対応表の旧 3 の行で、icon を必須にする「Alert」に「非中立の（warning・destructive）」が掛からない。中立の既定 Alert〈お知らせ一般〉は現行 canonical で icon なし）= `docs/backlog.md` の runtime lane A の対象に 1 句（lane A が `alert.tsx` の icon を扱うときに文言を直す）。C2（本 packet の Spec Contract D8・D11 が是正 `68c152cf` の文面を反映していない）= D8・D11 は `docs/design-system/00-foundations.md` 65 行目（試しを採らないときの直し先）と `docs/backlog.md` の runtime lane A の行を正とし、本 packet の該当文はそれに置き換わった。C3（`src/features/plu-export/PluExportPage.tsx` 366 行の Alert の `border-info`・`bg-info-soft`・`text-info-strong`。`info` 家族は `globals.css` にも 00 にも無い）= lane A の対象の drift の括弧に 1 句。1 回目の base 同期 `d5c4709d` に当てた closure の P3-3 = 本 packet と Matrix が参照する `docs/backlog.md` の行番号（Review Response の表の「51 行目」「54 行目」ほか）は `2f02ee7a` 時点のもので、現行では別の entry を指す。
