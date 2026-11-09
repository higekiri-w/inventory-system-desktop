# Plan Packet: 店主・owner 回答台帳の「docs へ戻す候補」を正本へ戻す（R2）

2026-09-24 起草（slug の日付は台帳の確定日 2026-09-23）。出典は repo 外の回答台帳（`.local/reports/store-premises/answer-ledger.md`、以下「台帳」。owner 決定 2026-09-21 により原文を公開 repository に置かないため local-only）の §4「docs へ戻す候補」58 件と、2026-09-23 の引継ぎ「次の行動」1。規則・harness の改訂は owner 決定 2026-09-24 で別 lane `agent/harness-overhaul` へ分けたため、本 packet は店の事実の復元だけを扱う。Coordinator 裁定 2026-09-24 により、owner が追補2（TD-013 / TD-017）で答えた食い違い D-16 / D-21 の保留を解き、追補2 の未収載の事実（TD-001〜TD-022 のうち店の事実）と `docs/plu-export-and-real-csv-verification.md` 24 行の部門数の drift も本 lane で扱う。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

- Evidence Mode: github
- Phase: archive
- Risk: R2
- Execution Mode: fable-window
- Plan Commit: f7b58778714eaf15b76f7d28095cb40d041acf6b
- Amendments: none
- Coordinator: Opus 5.5（Claude Code main session）
- Writer: Opus 5.5 subagent（worktree run）
- Plan Reviewer: Opus 5.5（fork でない fresh subagent、Writer と別 context）
- Final Reviewer: Opus 5.5 fresh subagent + Codex（互いに独立、Plan Reviewer とも別の fresh context）
- Final Review Minimum: 2
- Human Gate: ready,merge

manual なし: 製品 runtime・画面・配布物への変化がない文書変更で、Windows native の確認対象がない。

座組の根拠と現行規則との衝突（明記）:

- 座組は owner 決定 2026-09-23（Opus = Opus 5.5 を Coordinator / Writer / review に全面解禁。Codex は rate limit 中のため Plan Review は fresh Opus だけで進め、Final Review だけ Codex を待つ）。
- 現行の tracked 規則とは衝突する。`docs/AGENT_OPERATING_MANUAL.md` §3 の高自律・低制約適性 slot の項（D-056）は Opus を read-only の Reviewer / Explorer 専任とし Writer / Coordinator に割り当てない、§3.4 の表は Opus を Claude Opus 5 とする。この衝突を解消する規則改訂は別 lane `agent/harness-overhaul` が持ち、本 packet は owner 決定 2026-09-23 を根拠に先行して座組を使う。
- Execution Mode は現行 enum から選ぶ。Fable 5.1 は難所の相談役として利用可能なため §3.2 の定義上 `fable-window` とする（本 lane の座組は Fable を使わない。値の整理は harness lane）。
- Plan Reviewer と Writer は同じ vendor・同じ model（Opus 5.5）。`docs/DEV_WORKFLOW.md` Review Rules の vendor 条項（D-062）は Writer が Codex の packet に掛かるもので、本 packet には literal に掛からない。PR #88 の dogfood 所見（同 vendor の Plan Review を packet の前提誤りが通過した）の残余は、Final Review の Codex が別 vendor の目を担うことで受ける。
- Final Review Minimum は R2 なら 1 を選べるが、owner 決定 2026-09-23 の Final Review 座組（fresh Opus + Codex）に合わせ 2 とする。Plan Gate 後は helper が変更を拒むため、Codex を待たない判断をするなら Plan Gate 前に 1 へ下げる。

遷移記録（append-only）:

- kickoff → spec-check → plan-draft（起草、未 commit）: Design Readiness のとおり既存の設計正本で足りるため design phase を経ない（spec-check → plan-draft の唯一の skip）。
- plan-draft → plan-gate（2026-09-24、Coordinator）: packet を plan-first commit で確定し、`docs/Plans.md` の wave 13 に登録。fresh Opus の Plan Review へ。
- plan-gate → plan-approved → implementing（2026-09-24、Coordinator、state-only）: Plan Review round 1（fresh Opus、P1 0 / P2 3 / P3 14、Ordinary Operation は `not applicable` 妥当）→ 是正 `d11502d6` → round 2（別の fresh Opus、P1 0 / P2 2 / P3 8）→ 是正 `c8048116`（owner 決定 2026-09-24: 評価額の丸めは `やると決めたもの（順番未定）`）→ round 3 closure（別の fresh Opus、P1/P2 = 0、P3 3）→ P3 を反映（P3-c の自動バックアップ timer の差は closeout で backlog へ）。Plan Commit = `f7b58778`。round 天井 3 に到達し通過。実装は Opus 5.5 subagent の worktree run。
- implementing → archive（closeout、本 commit）: PR #93 squash merge `e40e50b9`（2026-09-25）。packet を `docs/archive/plans/` へ移送し（R2 のため Matrix なし）、Implementation Results / Review Response を記録した。Final Review closure（Opus、`c58bd929`）の N-1（Review Response に Opus broad の裁定が無い、Codex の段落の位置）を本 closeout で直した。Non-scope の期間・動線の旧表記 7 か所と自動バックアップの timer の差（Plan Review round 3 P3-c、Opus broad の closeout 申し送り）を `docs/backlog.md` へ起票した。

## Owner Effort Budget

- 介入回数上限: 3（見込み: Codex の Final Review relay 1、Ready 1、merge 1）
- 実働時間上限: 15分（文書だけの変更で、owner の作業は relay と Ready / merge の判断に限られる見込み）
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
`docs/DEV_WORKFLOW.md` Risk Tiers と `docs/project-profile.md` High-risk Changes を当てた。変更は店の事実の記録（project-memory・backlog）と、設計正本の中の事実の記述（1 日の動線の文、業務シナリオ例の 1 語、棚卸しの期間の文）の訂正で、runtime 契約・DB・CSV/TSV・command・route/search・画面の振舞い・merge gate を変えない。R3 の「operator workflow」は画面の操作や状態遷移を決める・変える変更を指し、本変更は owner の回答に記述を合わせるだけで新しい操作を決めない。data safety の境界（公開 repository への店の情報の掲載）には触れるが、既存の owner 決定（原文は置かず要旨と確認日）と `docs/project-profile.md` Data Safety Boundary の範囲内で、境界自体を変えない。`scripts/ci/classify-changes.sh` に S1〜S6 と本 packet を渡すと `docs=true`・`traceability=true`・`rust_drift=true`・`workflow=false`（`docs/function-design/*` の変更が traceability と Rust の設計 drift の検査対象に分類されるため。hosted CI はその検査を回す）。

## Goal

Goal Invariant:

### 最小完了条件

- 台帳 §4 の 58 件のうち本 lane が扱う 46 件と TD-017 の 47 行（下表「復元する候補」）、台帳 追補2・追補3 の未収載の店の事実 5 件（下表「追補2・追補3 から足す事実」）の計 52 行が、指定の戻し先に要旨・確認日・誰の回答かつきで載り、次の設計・レビューで店の事実を引くとき tracked docs だけで同じ答えに辿り着ける。
- 台帳が誤りと判定した現行の記述 4 か所（`docs/project-memory.md` の約 929 件の否定、`docs/SCREEN_DESIGN.md` の 1 日の動線、`docs/db-design/master-tables.md` の業務シナリオ例、`docs/function-design/73-ui-stocktake.md` §73.1 の期間）と `docs/backlog.md` 単位の拡張の「店の回答由来ではない」が、台帳の現行版に合う。

### 失敗定義

- 復元した記述が台帳の要旨と食い違う、推測を事実として書く、旧版を現行として書く。
- owner 確認待ちの食い違い D-01 に依存する主張を確定として書く。
- S1〜S6 の file に、owner・店主の発言原文（短い語句の引用を含む）、台帳の ID（`L-nnn` / `TD-nnn` / `D-nn` / `Q-nnn`）、実データ（JAN・商品名・価格・原価・売上金額・取引先名・個人名）、棚卸しの具体的な日付（年末の基準日としての大晦日〈L-059 の PM `### いまの手作業` の行と F73 §73.1〉は可、開始日など他の具体的な日付は不可）、`.local/` の path・会話の session ID が入る。
- 他 lane が持つ記述（ADR lane の ADR と同期先、project-memory の Z004 layout A/B の行）を書き換える。

### 非目的

- 規則・harness の改訂（別 lane `agent/harness-overhaul`）。
- ADR `docs/adr/2026-09-18-stocktake-time-evidence.md` と同期先の時点証拠契約の修正（ADR lane）。
- 台帳 §4・追補2・追補3（TD-023）に無い事実の追加。追補3 の TD-024〜TD-026 は棚卸しの設計判断と未確認事項で、ADR lane・店主への確認に属するため扱わない。追補2 のうち設計判断（TD-003 / TD-005 / TD-006 / TD-008）と実機確認・記録の許可（TD-016 / TD-021）は店の事実ではないため扱わない。
- 設計・実装の変更（最小ウィンドウの寸法、単位 code、backup の保存先の設計、EJ 取込みの UI、評価額の丸めの実装）。backlog への記録に留める。
- 台帳自体の修正、`.local/` の file の変更。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

not applicable: operator の操作、data / command 契約、業務の状態遷移を決める・変える packet ではなく、workflow の変更でもない。`docs/SCREEN_DESIGN.md` の 1 日の動線の訂正は owner 回答（2026-09-22 / 2026-09-23）に記述を合わせるもので、新しい操作を定めない。本 packet を完了しても、製品の目的（日次の自動在庫連動を含む v1.0）の達成状況は変わらない。

## Scope

- S1 `docs/project-memory.md` の `## Store Premises Facts（現場の前提、owner確認 2026-09-19）` 配下: 下表の戻し先が project-memory の行を追記・訂正する。既存の書式（事実 — 確認日と回答者 — 出典）と、節ごとの既存の言語に合わせる。新しい行の事実の部分は、英語の節でも日本語で書いてよい（既存の節も日英が混在する）。AC2 の検索語は日本語のまま使う。出典欄は回答者の区分（店主回答 / owner回答 / owner 伝聞 / owner判断 / 推論 / 訪店記録の推奨）と確認日、sanitized な記録がある場合はその tracked path、公式の外部 URL（例: CASIO の公式ページ）だけを書き、台帳の ID・`.local/` の path・発言の引用符付き語句を書かない（下表の要旨欄の括弧内の ID と「解消」の注記は packet 内の照合用）。40 行目（Z004 layout A/B、ADR lane が所有）と 190 行目（layout A/B）には触れない。
- S2 `docs/backlog.md`: 下表の戻し先が backlog の entry へ追記・訂正する。ECR+ のサービス終了とリース満了の関係（TD-020）を `#### 機能・運用` の新しい entry に、評価額の丸めを店の規則に合わせる件（TD-023）を `#### やると決めたもの（順番未定）` の新しい entry にする（owner 決定 2026-09-24）。新しい entry は置き先の見出しの末尾へ置く（TD-023 は `#### やると決めたもの（順番未定）` の末尾、次の `#### ⑰ …` の直前）。24 行目（次の design lane）と 36 行目（layout B）には触れない。
- S3 `docs/SCREEN_DESIGN.md` `### 利用者の1日の動線`（78〜83 行の code block）: 81〜82 行の 2 行を次の 1 行に置き換える（文言は固定、Writer は変えない）。79〜80 行は変更しない。

  ```
  → レジ精算（店は開けたまま）→ 売上データ取込み（日報Z001/Z002/Z005、店を離れる前）→ 店を離れる
  ```

  code block の直後に次の 1 文を足す（文言は固定）: 「売上レポートの確認を1日のどの時点で行うかは決めていない。バックアップはアプリの自動バックアップ（起動時と設定時刻、`function-design/71-mnt-backup.md` §71.8）が取り、PC の外へのコピーを1日のどの時点で行うかは決めていない（backlog のバックアップの項）。」。文中の「backlog」に `docs/SCREEN_DESIGN.md` から見た相対 link（同じ directory の `backlog.md`）を、`function-design/71-mnt-backup.md` にも同じく相対 link を付ける。Excel への貼付け・印刷はアプリの動線に書かない（店の現行の手作業は project-memory が持つ）。
- S4 `docs/db-design/master-tables.md`: 86 行目の業務シナリオ例の語（L-298。「閉店後CSV取込み」を「精算後のCSV取込み」に置き換える）、`### 初期データ（全21部門、C-1/C-3 2026-03-29 確定）` 直後の注記（L-025）、`### 設計意図` の jan_code NULLABLE の理由の近く（L-175）。先頭の `## 時点証拠契約（proposed・未実装）` 節には触れない。
- S5 `docs/function-design/73-ui-stocktake.md` §73.1（44 行目）: L-059 / L-061 に合わせて期間と目的の文を訂正する。先頭の `## 時点証拠契約（proposed・未実装）` 節には触れない。
- S6 `docs/plu-export-and-real-csv-verification.md`: `### SDカード / PCツール保存領域` に L-042 の EJ の取込み経路と頻度を 1 項目追記する。24 行の「SR-S4000 は 20 部門」を、初期部門は 21（`docs/db-design/master-tables.md` の初期データ、2026-03-29 確定）で「20 部門」は機種仕様と推測される、に直す（台帳 D-03）。
- S7 本 packet・`docs/Plans.md`: 計画・現在地・次の行動の同期（Coordinator が担当。Writer は編集しない）。

対象を使う側の確認（起票時、main `3148347b`）: `rg -n 'レジ精算|閉店後CSV|数週間|10月〜大晦日|is stale' scripts src src-tauri/tests src-tauri/src` → `src-tauri/src/biz/stocktake_service.rs:4` の code comment（「10月〜大晦日」）だけ。test は対象の文言を assert しない（Non-scope の follow-up に記録）。`scripts/tests/reading-order-drift.test.sh` は project-memory の path 名だけを見る。生成物・REQ・route・bindings への波及なし。

### 復元する候補（47 行 = 台帳 §4 の 46 件 + TD-017）

要旨は台帳の該当行（`L-nnn`）の現行版から書く。Writer は台帳（main checkout の `.local/reports/store-premises/answer-ledger.md`、read-only）の該当行を開いて照合し、下表と台帳が食い違う場合は編集を止めて Coordinator へ返す（`docs/AGENT_OPERATING_MANUAL.md` §5.6「Writer が編集前に止まったとき」）。

| ID | 戻し先 | 書く要旨（公開可の要旨） |
|---|---|---|
| L-044 | PM `### レジ・レジスターツール` | レジの売上/EJ 保存設定は有効で、SD の Z004 と同じフォルダに EJ（電子ジャーナル）のファイルがある。中身は締めレシートのような普通のジャーナルのテキスト — owner確認 2026-09-17。実物は採取済みで repo 外の現場資料にある — owner回答2026-09-23 |
| L-042 | 同上 + PLUV `### SDカード / PCツール保存領域` | EJ は日報とは別の CV17「電子ジャーナルを閲覧する」操作で取り込み、取込み後は SD の `XZ_BKUP` と PC 側 `EcrDatas` に残る。SD 上に 2022-07 以降の約 4 年分がある — 2026-08-15 訪店の実機確認。店での EJ の PC への取込みは月 1 回程度で、日次ではない — 店主本人の話と EJ ファイルの日付で確認（owner回答2026-09-23、TD-013。D-16 を解消） |
| L-041 | PM `### レジ・レジスターツール` | レジの PLU 名称欄は 16 バイトまでで、動かせない制約 — owner確認 2026-09-17 |
| L-017 | 同上 | CV17 は日報のコピペ元としてだけ使われ、まともに使えていない。CV17 の更新（公式の最新は 2.0.1）は owner が行う — owner 2026-08-15 |
| L-006 | PM 187 行の訂正 | 「約 929 件は古い記録で現状と合わない」という否定を削る。既存のスキャニング PLU 約 929 件は死蔵で、運用上は存在しないものとして扱ってよい — owner 伝聞 2026-07-06。通常 PLU の 2 件はテスト登録（owner回答2026-09-19）は残す |
| L-010 | 同上 | 2026-08-20 の店 PC での確認で、スキャニング PLU 領域は既存登録 933 件（検証用 4 件を含む）・空き 3,851・アプリ管理 0 |
| L-170 | PM `### レジ・レジスターツール`（SD・CV17 の容量の事実）+ BL 50 行（保全単位） | PM へ: SD は 16GB・FAT32・使用率約 4%、PC 側の CV17 履歴は約 416MB — 2026-08-15 訪店記録。BL 50 行へ: 復旧に備える保全単位は SD の CASIO フォルダ全体と PC 側 CV17 の 2 フォルダの計 3 点 — 訪店記録の推奨（実施の記録なし） |
| L-018 | PM `### 在るもの` の ECR+ の行（101 行） | 店主は 2026-03 に、ECR+（スマホ）が無いと精算できないと答えている — 店主回答（2026-03 ヒアリング） |
| L-049 | PM 144 行の注記 | この行は 2026-09-22 の owner 回答（143 行）からの Coordinator の推論で、owner の発言ではない、と出典欄を直す |
| L-068 | PM `### いまの手作業` | 棚卸しで数える時間帯は決まっていない — owner 伝聞 2026-09-16 |
| L-059 | 同上 + F73 44 行 | 棚卸しは税理士の指示で年 1 回、大晦日時点で行い、報告は仕入原価の総額だけ。差異は棚卸しロスとして許容する — 店主回答（2026-03）、owner 2026-08-27（年 1 回、多くても 2 回） |
| L-060 | PM `### いまの手作業` | 年末の棚卸しの時期は毎年変わらない — owner回答2026-09-19 |
| L-061 | F73 44 行 | 「10月〜大晦日・数週間」を、準備（10 月〜）と年末の正式カウント（大晦日に完了）に分けて書く。準備の開始は 2026-08-15 の店主回答で前倒しされた（PM 149 行）。正式カウントの開始日など 12 月内の日付は書かない |
| L-062 | PM `### いまの手作業` | 10 月からの準備の中身は、値上げ品の原価・売価の確認と訂正、新商品のリストへの追加、原価不明品の問い合わせ — owner 伝聞 2026-07-07 |
| L-076 | PM 147 行の注記 | 147 行は前回の年末棚卸し（Excel シートを渡す前）の運用。2026-09-23 時点は owner が渡した Excel シートへ、机に置いたノート PC と棚を往復して入力している（棚の前で PC を持って入力するのではない）— owner回答2026-09-23。次の年末も Excel で行うかは未確認 |
| L-071 | PM 147 行の補足 | 新規品の書き足しは途中に手書きしたり最後にまとめたりで統一されていない（紙のリストに行を挿入できない前提の試行錯誤）。商品の場所はすべて分かっている — 店主回答・owner 伝聞 2026-08-20 |
| L-079 | PM 151 行の注記 | この転記期間の記録は、アプリを使えていない原始的な運用（Word や手書き）の頃のもので、前提としては正確とは言いにくい — owner 2026-09-23（疑問形の留保つき） |
| L-140 | PM `### いまの手作業` | 値上げの PDF は画面では作業しにくいので、印刷して書き込み、後で PC の棚卸しリストの売価・原価をまとめて直す（二重管理）— owner 伝聞 2026-08-21 |
| L-081 | PM `### いまの手作業` | 期限切れ・破損品は記録しておらず、棚卸しのときに在庫数を変えてロスとして計上してきた — 店主回答（2026-03） |
| L-174 | 同上 | 発注時はメーカー品番で照会し、分からなければ JAN の一部、それも無ければメーカー名＋商品名で仕入先へ照会する。品番と JAN の対応が載る資料は商品によって異なる — 店主回答（2026-03） |
| L-123 | 同上 | 在庫が少ないと感じる数は商品で違う（毛糸・スナップ・編み針・刺繍糸・はさみ類で目安が別、ミシン糸は定番色とそれ以外で分ける）— 店主回答（2026-08-15 フォーム）、`docs/evidence/issue-76/` の sanitized 記録 |
| L-082 | PM `### 利用者` | 店主はシステムと実在庫のずれを、棚卸しでロス（廃棄・万引き等）として出る範囲内にしてほしいと答えている — 店主回答（2026-03） |
| L-286 | PM 180 行の補足 | 実利用者は赤と黄色の区別がつかない。文字も小さいかもしれず、老眼もある — owner 伝聞 2026-06-06 |
| L-280 | PM `### 利用者` | CSV 取込み画面の「エラー N 件」が展開できる操作部だと owner 自身も最初は気づかなかった。owner は、店主はもっと気づきにくいと判断した — owner判断2026-08-03 |
| L-297 | PM `### 利用者` | owner は店と家の 2 拠点を行き来し、家で作業するときはノート PC を家へ持って行く（PC は 1 台のまま）— owner回答2026-09-23 |
| L-157 | PM `### 在るもの` | 店にネットワーク（Wi-Fi・LAN）がある — owner回答2026-09-19 |
| L-164 | 同上 | 店での運用を想定した市販の USB HID バーコードリーダーを用意し（owner 2026-08-09）、店 PC で Excel シートの JAN 入力に使っている — owner回答2026-09-23 |
| L-217 | PM `### 無いもの` 133 行 | 外部・ネットワークの保存先は今は使っていない。owner は Google Drive はセキュリティ面に不安があり、小さな AWS 等ならと述べた — owner回答2026-09-19。外部に保存するなら置き場所は店 — owner回答2026-09-23。暗号化したクラウド保存は今は見送り、v1.0 に含めない方向（費用の懸念。backup の設計 lane で再判断し得る）— owner判断2026-09-23。133 行の、採否がまだ決まっていないとする記述をこれに合わせる。199 行（`### 未確認` のネットワーク保存先）は変更しない |
| TD-017 | PM `### 無いもの` 128 行の補足 | 日々の在庫の増減の電子記録は無いが、前年の棚卸しリスト（Word 等）は店の PC に残っている。今の棚卸しは owner が新しく作って渡した Excel シートで行っている — owner回答2026-09-23（D-21 を解消） |
| L-239 | PM 127 行の補足 | Excel シートの定着について、owner は運用できる見込みが薄く、店主をどう導けばよいか分からないと述べた — owner回答2026-09-19 |
| L-104 | PM `### 決めた運用` | 人は間違える前提で、間違えたときに検知・停止・訂正できる仕組み（起きても補える・実行前に止める・そもそも操作できない）で守り、設計で防げるものを運用に任せない — owner 判断 2026-09-18〜2026-09-23 |
| L-298 | SCR 81〜82 行 + MT 86 行 | 売上の取込みは必ず店で、レジ締めの後・店を離れる前に行う — owner回答2026-09-23（2026-09-22 回答と合わせて）。SCR は S3 の固定文言、MT は S4 の置換だけを行う |
| L-025 | MT `### 初期データ（全21部門…）` の注記 | 部門の数や割当を今後変える可能性を owner は否定していない — owner回答2026-09-19 |
| L-175 | MT `### 設計意図`（jan_code NULLABLE の理由の近く） | JAN の無い商品は主に生地とヘア雑貨で、全体の約 2 割 — 店主回答（2026-03） |
| L-022 | BL 81 行 | リース満了の時期は大体決まっているはずだが年と月は記録に無い（owner 2026-09-19）。満了年月は店主への確認中 |
| L-014 | BL `#### 機能・運用` | 定額値引きを商品として追うなら通常 PLU が第一候補。その場の値引きは既存の「％-」キーで足りるので、業務の要求が出るまで作らない — owner 判断 2026-07-06 |
| L-057 | BL `#### 機能・運用`（PLU 関連 82〜83 行の近く） | PLU へ移した商品と未移行の商品の見分け方と誤操作の防止は、店主に考えさせず運用設計の側で案を作る — owner 判断 2026-08-14 |
| L-116 | BL 39 行 | レジ締め時の確認に使っている Excel 貼付け・印刷は、アプリで丸ごと置き換えるつもり（owner 2026-09-16。2026-08-01 には置き換えられるかは使い手次第とも述べた） |
| L-117 | BL 39 行 | 日報の印刷機能はあってもいい。紙はやめていく方針だが、目の悪さもあって紙が見やすい場面がある — owner 2026-09-19 |
| L-219 | BL 50 行 | 現行 CV17 のインストーラと公式利用ガイドのオフライン保管は訪店記録の推奨で、実施は未確認 |
| L-299 | BL 50 行 | バックアップを外部に保存するなら置き場所は店。PC が店にあるときは媒体と同じ場所になる残存リスクがある（推測として書く）— owner回答2026-09-23 |
| L-220 | BL 50 行 | 店主はデータが消えたとき 3 日程度前まで戻れれば OK と答えている（QR-04 の元の回答）— 店主回答（2026-03）。台帳の戻し先候補の `docs/spec/requirements-coverage.md` は「業務事実を転記しない」と冒頭で定めているため使わない |
| L-221 | BL 50 行 | 手作業に戻れれば OK かには「手作業の中身が分からないと判断できない」と答えている — 店主回答（2026-03） |
| L-154 | BL `#### 見た目・UX` | ノート PC で画面が小さいので、最小ウィンドウ（`docs/SCREEN_DESIGN.md` の 1024×720）をもう少し小さくできるとありがたい — owner 2026-08-20。寸法の変更は設計判断で本 lane では決めない |
| L-275 | BL `#### 見た目・UX` | 商品一覧の「PLU 対象にする／外す」は、押したらすぐ変わるのか文言から結果が想像しにくい — owner L3 所感 2026-09-03 |
| L-206 | BL 34 行 | 「現行 cm は…店の回答由来ではない」を直す。店主は 2026-03 に、切り売りは基本 m・販売は 10cm 単位でもする・棚卸しはメーカー記載の長さで数え cm までは数えないと答え、cm の整数管理で OK とも答えている（BL 259 行） |
| L-210 | BL 34 行 | 2026-08-15 の単位系の選択（表示用 unit_label の追加）は、その後の単位 code の設計（2026-09-09〜）で置き換えた、と明記する |

### 追補2・追補3 から足す事実（5 件）

台帳 追補2（部分台帳 `ledger-parts/today-2026-09-23.md` 末尾、TD-013〜TD-022）と同 file の TD-001〜TD-012、追補3（`ledger-parts/today-2026-09-24.md` の TD-023）のうち、上の候補に含まれない店の事実。TD-001 / TD-002 / TD-004 / TD-007 / TD-009 / TD-010 / TD-011 / TD-012 / TD-018 / TD-022 は上の候補（L-076 / L-079 / L-044 / L-104 / L-297 / L-298 / L-299 / L-217 / L-217 / L-164）の現行版として反映済み、TD-013 は L-042、TD-017 は上表の TD-017 行。

| ID | 戻し先 | 書く要旨（公開可の要旨） |
|---|---|---|
| TD-014 | PM `### 決めた運用` | 毎日の精算の作業に EJ の取込みを足すことを owner が了承した（SD からファイルを取るだけなので負担は小さい、という owner の判断。店主本人は未確認）。アプリがどう支援するかは EJ の設計 lane で決める — owner判断2026-09-23 |
| TD-015 | PM `### いまの手作業` | 棚卸しの途中でノート PC の蓋を閉じ（家へ持ち帰る場合を含む）、続きを後で入力することはありそう — owner回答2026-09-23 |
| TD-019 | PM `### 未確認` | 導入後、値上げのときに紙の前年リストを手で直す作業をやめるかは未確認。owner は、このアプリを渡すことがその代わりになるはずと考えている — owner回答2026-09-23 |
| TD-020 | PM `### レジ・レジスターツール` の ECR+ の行（193 行）+ BL `#### 機能・運用` の新 entry | CASIO の公式ページ（<https://web.casio.jp/ecr/ble/ecr.html>、2026-09-24 確認: 新規申込受付終了 2026年1月4日、サービス提供終了 2028年12月末予定）。店主は精算に ECR+ を要すると答えている（L-018）ため、レジのリース満了が 2028 年 12 月より後だと精算に困る恐れがある（推論）。owner は、ECR+ が終わるならレジの入替えと同時でないと困り、続かないなら代わりを自分たちで作るしかないと見ている — owner回答2026-09-23。リースの満了年月は店主へ確認中。backlog entry は、満了年月の回答を待って ECR+ 終了後の精算とジャーナル閲覧の代替を判断する、とする |
| TD-023 | PM `### いまの手作業` の 153 行（Rounding の行）の直後 + BL の「やると決めたもの」見出しの新 entry「評価額の丸めを店の規則に合わせる」 | PM へ: 153 行は価格を決めるときの端数処理（掛率は切り捨て、切り売りの端数は切り上げ）で、これとは別に棚卸しの評価額の丸めを 1 行足す。棚卸しの評価額は、商品別の金額を小数第 3 位で四捨五入して小数第 2 位まで持ち、全商品の金額を合計した最終合計で四捨五入する — owner 伝聞 2026-09-24（店主本人の答えを owner が確認）。BL へ（owner 決定 2026-09-24 で「やると決めたもの」見出しに置く）: 現行は原価が円の整数（`docs/db-design/master-tables.md` 34・53 行、`src-tauri/src/db/schema_v1.rs:36`・`:223` の INTEGER）で、確定時の総額は `valuation_cost_price × actual_count` の整数積和（`src-tauri/src/biz/stocktake_service.rs:464-475`、`docs/function-design/35-biz-stocktake-service.md` の新方式の total_cost の式と `complete_stocktake` の処理ステップ）。商品別の金額に小数が生じず丸めの段が無いため、数量や単位当たり原価に端数が出る場合（単位の拡張、BL 34 行: m の数量は小数 1 桁）に店の規則と一致しない。実装は本 lane の対象外 |
| TD-016 / TD-021 | — | 実機確認・記録の許可で店の事実ではないため扱わない。実機確認の計画は次の design lane（`docs/backlog.md` 24 行）が持つ |

### 他 lane が持つ候補（本 lane では書かない、10 件）

| ID | 持つ lane | 理由 |
|---|---|---|
| L-054 | ㉗ ADR 修正 lane（受入れ済み: 同 lane packet の Spec Contract R8 / S1） | 戻し先が ADR の Context（PC 時計・レジ時計の見立て） |
| L-095 / L-096 / L-097 / L-099 | ㉗ ADR 修正 lane（受入れ済み: R8 / S1） | 戻し先が ADR の Context / Rejected。棚卸しの時点証拠の owner 回答の理由・旧運用ルール |
| L-098 | ㉗ ADR 修正 lane（受入れ済み: R8 / S1） | 棚卸し設計の中心（いつ数えても年末へ繰り越す、二重減算しない）を ADR の Context に置く |
| L-107 | ㉗ ADR 修正 lane（受入れ済み: R3 / R4） | 判定不能は「適用＋要再確認」に一本化（owner 決定 2026-09-23、ADR 修正の本体） |
| L-108 | ㉗ ADR 修正 lane（受入れ済み: R3 / R4）/ ㉘ 最初の lane | OS 監視と legacy 専用復旧を外す（ADR 修正の本体、㉘ packet にも反映） |
| L-045 | 次の design lane（実測と POS 系列の対応）/ EJ parser core | EJ の取込みを店にどう示し支援するかという owner の問い。戻し先が design lane の論点 |
| L-053 | 次の design lane | Z004 の前後情報で内部計算できないかという owner の問いと見立て。戻し先が design lane の Context |

harness lane が持つ候補: なし（58 件の戻し先に `docs/DEV_WORKFLOW.md` / `docs/AGENT_OPERATING_MANUAL.md` / `CLAUDE.md` / `AGENTS.md` / templates / `docs/agent-guidance/` は無い）。

### owner 確認待ちで後回しにする候補（2 件）

| ID | 食い違い | 扱い |
|---|---|---|
| L-129 | D-01（店主の言う「取引先」が注文先〈問屋・仕入先〉か、店主フォーム Q-016 で確認中） | PM 114 行（「問屋ごとに 1 枚」）は変更しない |
| L-128 | D-01（発注方法は「取引先ごとに 4 通り」の取引先の意味が同じ争点） | 書かない |

D-16（EJ が PC に届く頻度）は TD-013、D-21（PC 上の前年の棚卸しリスト）は TD-017 の owner 回答で解消したため、L-042 は頻度つきで、L-140 は TD-017 とあわせて戻す（Coordinator 裁定 2026-09-24）。

## Non-scope

- 上記以外の tracked file。特に `docs/DEV_WORKFLOW.md`、`docs/AGENT_OPERATING_MANUAL.md`、`CLAUDE.md`、`AGENTS.md`、`docs/templates/**`、`docs/agent-guidance/**`、`.claude/**`（harness lane）、`docs/adr/**`、`docs/decision-log.md`、`scripts/**`、`.github/**`、`src/**`、`src-tauri/**`、`docs/spec/**`、`docs/evidence/**`。
- `docs/project-memory.md` 40 行・190 行（Z004 layout A/B。ADR lane）、114 行（D-01 保留）。
- `docs/db-design/master-tables.md` と `docs/function-design/73-ui-stocktake.md` の `## 時点証拠契約（proposed・未実装）` 節（ADR lane）。
- `docs/backlog.md` 24 行・36 行。
- `docs/SCREEN_DESIGN.md` の最小ウィンドウの寸法（L-154 は backlog への記録だけ）。
- 期間・動線の旧表記が残る次の箇所は本 lane で直さず、closeout で backlog の `#### workflow / test / lint / docs` へ follow-up として記録する: `src-tauri/src/biz/stocktake_service.rs:4`（code comment「10月〜大晦日」）、`docs/function-design/35-biz-stocktake-service.md:374`（「10月〜大晦日の長期作業」、ADR lane の同期先）、`docs/architecture/biz-task-specs.md:451`（同、ADR lane の同期先）、`docs/screen_mockups.html:259`（ボタン説明「閉店後にレジのCSVを読み込む」）、`docs/db-design/tracking-system-tables.md:137`（「10月〜大晦日の長期作業」）、`docs/screen_mockups.html:208`（図中の「10月〜大晦日」）、`docs/SCREEN_DESIGN.md:212`（「数週間スパン」）。 同じく closeout で、設定時刻の自動バックアップを確認する 60 秒 timer が `src/features/backup-restore/BackupRestorePage.tsx:167-187` にしか無く、バックアップ画面を開いている間しか動かない（`function-design/71-mnt-backup.md` §71.8 の「フロントエンドタイマー」との差）ことを backlog `#### やると決めたもの（順番未定）` へ記録する（Plan Review round 3 P3-c）。
- `docs/project-memory.md` 199 行（`### 未確認` のネットワーク保存先）。
- 評価額の丸めの実装・設計（TD-023 は事実と backlog entry だけ）。

## Acceptance Criteria

baseline は main `3148347b` の worktree（`.claude/worktrees/docs-rules`）で同じ command を実行した実測。

- AC1（約 929 件の否定の訂正）: `rg -n 'is stale and does not match' docs/project-memory.md` が一致なし（baseline: `187:` の 1 行）。`rg -n '933' docs/project-memory.md` が `### レジ・レジスターツール` 内に一致する（baseline: 一致なし）。
- AC2（EJ の事実）: `rg -n 'EJ|電子ジャーナル' docs/project-memory.md` が `### レジ・レジスターツール` 内に一致する（baseline: 一致なし、exit 1）。一致する行は取込みの頻度を「月 1 回程度」とし、出典を owner回答2026-09-23 とする。`rg -n '2028年12月末' docs/project-memory.md docs/backlog.md` が両 file に一致する（baseline: 一致なし、exit 1）。`rg -n '小数第 ?2 位|第2位' docs/project-memory.md` が `### いまの手作業` 内に、`rg -n '評価額の丸め' docs/backlog.md` が `#### やると決めたもの（順番未定）` 内の 1 entry に一致する（baseline: どちらも一致なし、exit 1）。
- AC3（1 日の動線）: `rg -n '→ レジ精算 → 閉店' docs/SCREEN_DESIGN.md` が一致なし（baseline: `81:` の 1 行）。`rg -n '閉店後CSV取込み' docs/db-design/master-tables.md` が一致なし（baseline: `86:` の 1 行）。訂正後の動線で売上データ取込みが店を離れる前に置かれる。
- AC4（棚卸しの期間）: `rg -n '数週間かけて' docs/function-design/73-ui-stocktake.md`（baseline: `44:` の 1 行）の §73.1（44 行付近）の文が準備と正式カウントを分けて書かれ、棚卸しの具体的な日付を含まない（年末の基準日としての大晦日〈L-059 の PM `### いまの手作業` の行と F73 §73.1〉は可、開始日など他の具体的な日付は不可）。67 / 91 / 92 行の「数週間」（確定操作の損失の文脈）は変更しない。
- AC5（単位の拡張）: `rg -n '店の回答由来ではない' docs/backlog.md` が一致なし（baseline: `34:` の 1 行）。
- AC6（網羅）: 「復元する候補」47 行と「追補2・追補3 から足す事実」5 行の計 52 行の各行について、戻し先の diff hunk に要旨が載ることを Test Plan T1 で確認する。後回しの 2 件と他 lane の 10 件は diff に現れない（T2）。
- AC7（他 lane・保留の行に触れない）: `for n in 40 114 190; do git show 3148347b:docs/project-memory.md | sed -n "${n}p" | grep -Fxq -f - docs/project-memory.md || echo "missing $n"; done` が何も出さない（3 行とも内容が新しい file に同じ 1 行として残る）。`git diff 3148347b...HEAD -- docs/db-design/master-tables.md docs/function-design/73-ui-stocktake.md` に `時点証拠契約` 節の差分が無い。
- AC8（data safety）: `git diff 3148347b...HEAD -- docs/project-memory.md docs/backlog.md docs/SCREEN_DESIGN.md docs/db-design/master-tables.md docs/function-design/73-ui-stocktake.md docs/plu-export-and-real-csv-verification.md | rg -n '^\+.*(\.local/|[0-9]{8,13}|session|jsonl|\b(L|TD|Q)-[0-9]{3}\b|\bD-(0[1-9]|1[0-9]|2[01])\b)'` が一致なし（exit 1）。追加行に owner の発言の逐語の引用符付き長文、棚卸しの具体的な日付、実データ（JAN・商品名・価格・取引先名・個人名）が無いことを T3 で確認する。
- AC9（footprint）: `git diff 3148347b...HEAD --name-only` が S1〜S7 の file だけを出す。`rg -n '20 部門' docs/plu-export-and-real-csv-verification.md` の 24 行が 21 部門と機種仕様の推測を書き分ける（baseline: `24:` の 1 行）。
- AC10: `git diff --check`、`bash scripts/doc-consistency-check.sh`、`bash scripts/doc-consistency-check.sh --target plan` が成功する。未解決の P1 / P2 なし。

## Design Sources

- Requirements / spec: 該当なし（要求を変えない）。
- Architecture: 該当なし。
- Function / command / DTO: `docs/function-design/73-ui-stocktake.md` §73.1（目的の文の期間の訂正だけ）。
- DB: `docs/db-design/master-tables.md` §1 業務シナリオ例・設計意図、§2 初期データ（事実の注記だけ）。
- Screen / UI: `docs/SCREEN_DESIGN.md` `### 利用者の1日の動線`（事実の訂正だけ）。
- Store facts: `docs/project-memory.md` `## Store Premises Facts`、`docs/backlog.md`、`docs/plu-export-and-real-csv-verification.md`。
- Decision log / ADR: owner 決定 2026-09-21（原文を置かず要旨と確認日。`docs/Plans.md`「直近の完了」の PR #84 の記録）。D-023（POS adapter boundary: EJ・CV17 の記述は adapter facts として project-memory / PLUV に置き、app core の契約にしない）。
- 台帳（local-only、read-only）: §4 の 58 件、§3 の食い違い D-01 / D-03 / D-16 / D-21、部分台帳 `ledger-parts/today-2026-09-23.md` の TD-001〜TD-022、`ledger-parts/today-2026-09-24.md` の TD-023。
- 外部: CASIO ECR+ 公式ページ <https://web.casio.jp/ecr/ble/ecr.html>（2026-09-24 確認）。

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | 該当なし | existing sufficient |
| Command / DTO / generated binding / wire shape | 該当なし | existing sufficient |
| DB / transaction / audit / rollback / migration | 該当なし（master-tables は例と注記の事実だけ） | existing sufficient |
| Screen / UI / route state / Japanese wording | `docs/SCREEN_DESIGN.md` 1 日の動線（事実の訂正） | updated in this PR |
| CSV / TSV / report / import / export format | 該当なし | existing sufficient |
| Durable decision / ADR | 該当なし（L-098 は ADR lane、L-104 は project-memory の「決めた運用」） | existing sufficient |

## Registration / Generation Obligations

該当なし。文書の新設・改名・削除なし、REQ / route / command / function-design doc の追加なし。bindings / route tree / traceability の再生成は対象外。decision-log へ追加しないため D-n の採番なし（並走する harness lane と registry を共有しない）。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| 店の事実（台帳 §4） | `docs/project-memory.md` Store Premises Facts | owner 決定 2026-09-21（要旨と確認日） | 台帳だけに置くと設計・レビューが tracked docs から辿れない。却下: 台帳を repo へ置く（原文を含むため不可） | S1 / S2 / S6 | T1 / T3 |
| 1 日の動線 | `docs/SCREEN_DESIGN.md` 利用者の1日の動線 | 台帳 D-09（L-298 で解消） | 旧記述「閉店 → 取込み」は 2026-03 の開発者の問いの前提で、店主は「OK」とだけ答えていた | S3 / S4 | AC3 / T1 |
| 棚卸しの期間 | `docs/function-design/73-ui-stocktake.md` §73.1 | 台帳 D-06 | 準備とカウントをまとめた「数週間」は唯一の直接回答（L-061）と合わない | S5 | AC4 / T1 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: 本 lane は設計を変えず、店の事実の所在を tracked docs に戻す。
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: なし（L-104 の原則は project-memory の「決めた運用」に置く）。
- Assumptions and constraints: 台帳の状態判定は main `3148347b` との照合で、反映 146 件の残り 126 件は裏取りが抜き取りだけ（台帳冒頭の注記）。本 lane は §4 の候補だけを扱い、反映済みの行の再照合はしない。
- Deferred design gaps, risk, and follow-up target: owner 確認待ち 2 件（D-01）、ADR lane・design lane の 10 件、stocktake_service.rs の comment。
- Test Design Matrix can cite design decision IDs or source doc sections: R2 のため Matrix を置かず、T1〜T4 の照合で足りる。
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: 「114 行に触れない」（D-01）を AC7 で機械確認する。

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | 適用。EJ・CV17・SD の事実は adapter facts として project-memory / PLUV に置き、BIZ/CMD の契約へ昇格しない（D-023） | T1 |
| Fact check / design decision split | 適用。owner の見立て・推測・訪店記録の推奨は、確定した事実と書き分ける（L-049・L-170・L-219・L-299 の「推測」「推奨」） | T1 |
| Lifecycle / retry | not applicable: 状態遷移を変えない | なし |
| Operator workflow | 適用。1 日の動線の訂正（L-298）は owner 回答に合わせるだけで、新しい操作を定めない | AC3 |
| Replacement path | not applicable | なし |
| Data safety / evidence | 適用。原文・実データ・棚卸しの日付・local path を公開 repository に入れない | AC8 / T3 |
| Reporting / accounting semantics | not applicable | なし |
| Manual verification | not applicable: 実機確認の対象なし | なし |
| 環境・再現性 | not applicable: 新しい環境依存なし | なし |

## Design Readiness

- Existing design docs are sufficient because: 事実の置き場所（project-memory の Store Premises Facts の節構成、backlog の見出し）は PR #84 で決まっており、本 lane はそこへ追記・訂正する。
- Source docs updated in this PR: S1〜S6。
- Design gaps intentionally deferred: 非目的と「後回しにする候補」「他 lane が持つ候補」。
- Durable decisions discovered in this plan and promoted to source docs: なし。

Minimum design checks for business-app work: 製品の layer / command / DB / operator の画面操作 / error 挙動に変更なし。Testability は AC の command と T1〜T4 の照合で担保する。

## Contract Probe

- N/A: R2 で、外部 library / OS / hardware の未確認の前提に依らない（事実の出典は台帳と既存の tracked 記録）。

## Test Plan

自動 test は追加しない（事実の記述の正しさは機械で判定できない）。Writer は S1〜S6 を 1 つ以上の commit で実装し、packet と `docs/Plans.md` を編集しない。packet と実装は別 commit（plan-first）。

- targeted tests: AC1〜AC5、AC7〜AC9 の command。
- T1（52 行の照合）: 「復元する候補」と「追補2・追補3 から足す事実」の各行について、戻し先の diff に要旨があり、台帳の該当行の現行版（上書きの関係欄が「旧版」の行でない）と意味が一致し、回答者の区分（店主本人 / owner / owner 伝聞 / owner 判断 / 推論 / 訪店記録の推奨）と確認日が落ちていない。
- negative tests: T2（後回しの 2 件〈D-01〉と他 lane の 10 件の要旨、追補2 の設計判断と許可が diff に現れない）。
- data safety checks: T3（AC8 の command に加え、追加行を目視で確認: 逐語の原文（短い語句の引用を含む）、棚卸しの具体的な日付、実データ、`.local/` の path、session ID が無い）。
- compatibility checks: T4（既存の行を消す訂正は L-006 の否定文、SCR 81〜82 行（S3 の固定文言 1 行に置換。79〜80 行は不変）、MT 86 行の語、F73 44 行の期間、BL 34 行の一文、PM 133 行の採否の記述、PLUV 24 行の部門数に限る。他の既存行は削らない。`git diff 3148347b...HEAD --numstat` の削除行数を hunk ごとに説明できる）。
- main wiring/integration checks: 追記した出典の file link が実在する（doc check）。

## Review Focus

- 各要旨が台帳の現行版と一致するか、旧版・推測・訪店記録の推奨を確定の事実として書いていないか（T1）。
- 保留の D-01 に依存する主張が混ざっていないか。TD-020 の ECR+ の終了時期を CASIO 公式ページの表記（2028年12月末予定）どおりに書き、リース満了の年月を推測で書いていないか（T2）。TD-023 の backlog entry が現行の挙動と店の規則を並べるだけで、実装方針を決めていないか。
- 公開 repository に置いてよい粒度か（原文・日付・実データ・local path、T3）。
- L-298 の動線の訂正が、新しい操作の決定（例: 取込みの順序や backup の時点の指定）へ踏み込んでいないか。
- 他 lane が持つ行・節（PM 40 / 190 行、時点証拠契約節）に触れていないか（AC7）。
- 普通の一日の観点: 次の設計 lane の reviewer が「店の取込みはいつ・どこで行うか」「EJ は SD にあるか」「棚卸しはいつからいつまでか」を tracked docs だけで引けるか。

## Data Safety

- commit しないもの: 台帳と部分台帳（`.local/reports/store-premises/**`）、owner・店主の発言原文、会話記録の session ID や path、実店舗のデータ（JAN・商品名・価格・原価・売上金額・取引先名・個人名）、棚卸しの具体的な日付、EJ の実ファイル。
- local-only paths: `.local/**`（台帳、発注書、review 報告の控え、gate の出力）。
- synthetic-only paths: 該当なし（fixture を追加しない）。

## Parallel Lanes

- 並走: ㉗ ADR 修正 lane（`agent/stocktake-time-evidence-adr-fix`）、㉘ 最初の lane、EJ parser core、harness lane（`agent/harness-overhaul`）。
- 同じ file を編集し得る組合せ: ADR lane と `docs/project-memory.md`（ADR lane は 40 行、本 lane は 40 / 190 行以外）、`docs/db-design/master-tables.md` / `docs/function-design/73-ui-stocktake.md`（ADR lane は時点証拠契約節、本 lane は §1 / §2 / §73.1）、`docs/backlog.md`（ADR lane は 24 行付近の可能性）。Draft PR #81（`chore/repo-path-lowercase`）は `docs/project-memory.md` 39 行と `docs/plu-export-and-real-csv-verification.md` の path を変える。hunk は重ならない見込みだが、現行の `docs/DEV_WORKFLOW.md` Wave Operation は同じ source document を編集する lane の同居を禁じており、4 lane 並走はその規則と衝突する（改訂は harness lane）。merge 順と base 同期は Coordinator が決める。
- helper の制約: `scripts/pr-gate.py:230` は PR head の `docs/plans/` に active packet が 1 つだけであることを要求する。先行 lane の merge 後、その closeout で packet が archive されるまで本 lane の base 同期を待つ。

## Implementation Results

[PR #93](https://github.com/higekiri-w/inventory-system-desktop/pull/93) で実装し squash merge 済み（`e40e50b9`、2026-09-25）。S1〜S6 を実装した: 「復元する候補」47 行と「追補2・追補3 から足す事実」5 行の計 52 行を、`docs/project-memory.md` の Store Premises Facts、`docs/backlog.md`、`docs/db-design/master-tables.md`、`docs/plu-export-and-real-csv-verification.md` へ要旨・確認日・回答者の区分つきで戻した（`40564b4a`）。誤りと判定された記述（約 929 件の否定、`docs/SCREEN_DESIGN.md` の 1 日の動線を S3 の固定文言へ、業務シナリオ例の語、`docs/function-design/73-ui-stocktake.md` §73.1 の期間を準備と年末の正式カウントに分けた文、単位の拡張の「店の回答由来ではない」、PLUV の部門数）を直した。Writer の自己照合で言い換えを台帳の意味に寄せ（`1fd8e5d3`）、Final Review broad（Opus）の P3 8 件を `da304f51` で是正した。他 lane の行（PM の Z004 layout A/B の 2 行、時点証拠契約節）と D-01 保留の行には触れていない（AC7）。

## Review Response

- Plan Review round 1（Opus 5.5、fresh subagent、`2a8c78aa`）: `Ordinary Operation` の `not applicable` は妥当。P1 0 / P2 3 / P3 14。Coordinator 裁定で全件採用し、`d11502d6` で是正（SCREEN_DESIGN の固定文言、AC の比較基準を merge-base に、棚卸しの期間の粒度、ほか P3）。同時に owner 回答 2026-09-24（評価額の丸め）を追加した。
- Plan Review round 2（closure、同 reviewer 系、`d11502d6`）: `not applicable` は妥当。P1 0 / P2 2 / P3 8。P2 は round 1 の P2-1（自動バックアップが実装済みであることを無視した固定文言）と P2-2 に当たる AC2 と S1 の言語の食い違い。Coordinator 裁定で全件採用し、本改訂で是正した（S3 の固定文言、S1 の言語・外部 URL、大晦日の扱い、TD-023 の置き場所と区分、follow-up 3 件、行数 52、L-006 の言い換え、classifier の理由、owner 決定 2026-09-24 による TD-023 の backlog の置き場所）。
- Final Review broad（2026-09-25、Opus 5.5 fresh、対象 `b20beefd`、裁定 Coordinator）: P1 0 / P2 0 / P3 9。復元した店の事実は台帳と一致し、誤記の是正は他 docs と整合、言い換えは許容範囲。P3-1〜P3-8（台帳の行の前段・条件・語の欠落、出典の区分の混在。PM の在庫精度の希望・外部保存・PLU 件数・EJ の取込み・ECR+ の終了、BL のバックアップの復旧許容・最小ウィンドウ、F73 §73.1 の棚卸しの回数）= 採用、`da304f51` で是正。P3-9（AC7〜AC9 の比較元が origin/main の取込み後に合わない）= 採用、記録だけ（lane の footprint を `dda8560a` 基準で PR body と本節に示す）。closeout への申し送り = SCREEN_DESIGN の期間表記の行（`数週間スパン`、取込み後は 213 行）と自動バックアップの timer の差を backlog へ。詳細は台帳の原文を含むため local-only。
- Final Review broad Codex 側（2026-09-25、GPT-6 Astra medium、対象 `da304f51`、裁定 Coordinator）: P1 0 / P2 0 / P3 1。Opus 側の結果を読まずに行った独立の 1 本。P3（AC9 と T4 の比較元 `3148347b` は origin/main の取込み〈`b20beefd`、`dda8560a` を取込み〉の後は他 lane の変更まで含め 29 file を出し、本 lane の scope 違反に見える。内容の scope 違反ではない）= 採用。AC と T4 の文は Plan Gate 時点の baseline として残し、取込み後の lane の比較は `git diff dda8560a HEAD --name-only` とする（`da304f51` で 8 file、すべて S1〜S7。Coordinator が再実行して一致を確認）。PR body の Validation も旧 command の実測と補正後の結果を分けて書く。

- Findings Freeze: not yet frozen; post-freeze exceptions: none.

Final Review の closeout 時点の結論（2026-09-25、Coordinator）: helper の専用 record（`#issuecomment-5819646789`）に載る broad は 2 本で、互いに独立した Double Audit。(1) Opus 5.5 fresh の broad（対象 `b20beefd`、`#issuecomment-5819635015`）と、同じく fresh の Opus による closure（対象 `c58bd929`、`#issuecomment-5819635472`、新規 P1 0 / P2 0 / P3 2。Opus P3-1〜P3-9 と Codex P3 の閉鎖を台帳の該当行と隣接行に照合して確認）。(2) Codex broad（GPT-6 Astra medium、対象 `da304f51`、`#pullrequestreview-5308205216`、上の段落）。record の closure は base 同期後の fresh Opus（対象 `5867414f`、`#issuecomment-5820295708`、新規 P1 0 / P2 0 / P3 0。2 回の base 同期〈origin/main `94e4d58a`、`a3f08cdd`〉とも lane の 8 file の差分は変わらず、取り込んだ PR #97 の checker / helper は本 packet を受理し、PR #94 の EJ 関連の文書と本 PR が戻した店の事実は食い違わない）。manual / R4 は not-required。Opus closure（`c58bd929`）の新規 P3 2 件: N-1（本節に Opus broad の裁定が無い、Codex の段落が箇条書きでなく Findings Freeze の行の後ろ）は本 closeout で直した。N-2（PR body の Validation の追記の位置と Review-only の欄が古い）は merge 前に PR body を更新して解消。後回しにした所見は `docs/backlog.md` へ起票した: 期間・動線の旧表記 7 か所（Non-scope の follow-up、`#### workflow / test / lint / docs`）、設定時刻の自動バックアップの timer がバックアップ画面を開いている間しか動かない差（Plan Review round 3 P3-c、`#### やると決めたもの（順番未定）`）。
