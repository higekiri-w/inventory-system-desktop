# Plan Packet: 単位を 12 種へ広げる runtime（単位の lane、U）

wave に属さない単独の lane（2026-10-11 起票、owner の lane 選択は repo 外の回答台帳 TD-222）。設計は merge 済み（PR #154、[decision-log](../../decision-log.md) D-113）で、本 lane は [共通規則](../../function-design/10-common-rules.md) SPEC-UNIT-D1〜D11 のうち単位の分（U）を実装する。原価の 1/100 円（C）・Z004 と EJ の数量（P）・棚卸しの再開（⑤）は別 lane。branch は `agent/unit-runtime`（main `7070382f` から）。並走は `agent/pos-series-design`（実測と POS 系列の対応の design）、ほかに `agent/backup-offsite`（PR #155 Draft、D-114、docs）・`agent/plu-clear`（D-110、docs）。D 番号は使わない（新しい durable な決定が無い。D-115 は Coordinator の予約のまま）。

**規模と手法（1 lane のまま。横断の約束は下の 1 つの表に置く）**: 2026-10-11 に本 worktree（HEAD `7070382f`）で数えた。触る既存の Rust は 41 file（`{ rg -l 'stock_unit|StockUnit' src-tauri/src src-tauri/tests; ls <単位を持たないが変える 14 file>; } | sort -u | wc -l` → `41`。うち `rg -l 'stock_unit|StockUnit' src-tauri/src src-tauri/tests | wc -l` → `28`）に新設 2（`biz/unit_amount.rs`・`db/schema_v8.rs`）、TS は 100 file（`rg -l 'stock_unit|stockUnit|StockUnit|grand_total|\bsubtotal\b|DailySaleItem|MonthlySaleItem|DeptSubtotal|GrandTotal|MovementTable|formatMovementQuantity|line_cost|line_loss_cost|parseRequiredSafeInteger|parseNonNegativeInteger|old_stock|new_stock|movements_sum|calculateUnitPrice|computeSummary|ProductRankingRow' src | wc -l` → `100`。test・fixture 45、それ以外 55）、D-113 を引く設計正本は 35 file（`rg -l 'D-113|SPEC-UNIT' docs --glob '!docs/archive/**' --glob '!docs/research/**' | wc -l` → `35`）。migration 1（v8）、wire の型の変更 13 種、横断の約束（範囲・0・失敗の順序・単位の混在）がかかる経路は在庫を書く 4 か所（`rg -n 'update_stock_quantity\(' src-tauri/src --glob '!*test*' | rg -v 'fn update_stock_quantity|db/inventory_repo.rs' | wc -l` → `4`: `common.rs:64`・`commit.rs:274`・`integrity_service.rs:173`・`stocktake_service.rs:571`。除いたのは定義と、同じ file の test の呼出し）と初期在庫 2 か所、数量を検証する 5 か所、売価・原価を検証する 10 か所（下の表の X5・X6）、数量を出す画面 11 行（共通規則 SPEC-UNIT-D3 の「表示する所」の表の 7 を除く）。分ける条件（1 段目だけが merge された main で (a) 店の画面の振舞いが今と変わらない (b) 新しい単位の商品を登録・取込みできる経路が開かない (c) 全 gate が緑）を現物に当てた結果:

- 発注の例の切れ目（1 段目 = 型・migration・`biz::unit_amount`・上限・checked、2 段目 = m の入力と表示・集計・商品フォーム・商品 CSV）は (a)(b) が成り立たない。(b) の反例: 商品 CSV の `在庫単位` は文字列のまま INSERT へ渡り（`src-tauri/src/biz/product_service.rs:1199`〜`:1202`・`:1402`）、今の守りは DB の CHECK だけ（同 `:3324` の test が rollback を固定）。CHECK を 12 code に広げた時点で `在庫単位` に `m` を書いた CSV が取り込め、m の入力の無い画面で cm の整数として扱われる。塞ぐには 1 段目に「新しい 10 code を拒む」仮の検査と test を足し 2 段目で消すことになり、正本（SPEC-UNIT-D10）と逆の code を R4 の lane に通す。(a) の反例: 上限は `数量が大きすぎます` 等の新しい error を出し、入庫・廃棄の行の金額を BIZ へ移すと既存の `cm` の商品の記録詳細の金額が 1/100 になる（既知の不整合の修正だが、画面の値は変わる）。
- (a)(b)(c) をすべて満たす切れ目は「振舞いを変えない土台だけ」（`biz::unit_amount` への 3 関数の移設、記録詳細 6 型の `stock_unit` の enum 化、formatter の param の型、集計の checked）に限られる。これは上の約 180 file のうち 21 file（新設の `unit_amount.rs`、`biz/mod.rs`、`stocktake_service.rs`、`time_evidence.rs`、`design_compliance_test.rs`、`10-common-rules.md`、repo 6、`bindings.ts`、`format-stock-display.ts` とその test、行の型 4、`sales_service.rs`、`integrity_service.rs`。fixture の追随は数えていない）の機械的な変更で、横断の約束（上限・m・集計・migration）は 1 つも 2 段目から減らない。Plan Review・Final Review・owner の Ready と merge がもう 1 周増えるだけなので分けない。
- 代わりに、(1) 実装の順序を下の Scope の S1〜S7 に固定して土台から積み、(2) DB・wire に出す値ごとの「範囲・0・失敗の時点と順序・単位の混在を保証する所」を下の「横断の約束」の表 1 つにまとめ、Contract Ledger と Matrix はその表の行を引く。
- 条件 (a) を外せば成り立つ別の切れ目がある（A = 既存の 2 単位のままで効く分〈上限・金額・m の表示・集計。migration なし、R3〉、B = v8・12 code・m の入力・商品フォーム・商品 CSV〈R4〉）。A だけの main は「`cm` の商品は cm で入力し m で表示する」という設計どおりの状態になるが、画面の振舞いは変わるので発注の条件では採れない。採るかは Coordinator の判断（S1〜S7 の境目で切れる。下の Scope）。

## Workflow State

- Phase: archive
- Risk: R4
- Plan Commit: 211233182811a777153e1f6392b55224ffed4cf6
- Amendments: 7bbecac1e3abb4e5b0fcaf5a21bb50ad62c172f9
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（subagent_type: writer）
- Plan Reviewer: fresh Opus 5.5 + Codex（model は発注時に決める）
- Final Reviewer: Fable 5.1（Claude 側、R3 以上）+ Codex（GPT-6.1 Sol 既定）。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Review Minimum: 2
- Human Gate: ready,merge,manual,r4
- Branch: agent/unit-runtime

遷移の記録:

1. kickoff → spec-check（2026-10-11、起草役）: Coordinator の発注（単位 U の runtime。C・P・⑤ は別 lane）を Scope にし、規模を実測して 1 lane のままと決め、Risk を R4 と記録した（下の Risk）。
2. spec-check → design（2026-10-11、起草役）: 対象の設計正本は共通規則 SPEC-UNIT-D1〜D11、[22](../../function-design/22-mnt-migration.md) §16、各 function-design の D-113 の proposed の注記（下の Design Readiness）。設計 lane の Final Review の P3 のうち U の分（61〜64 の明細表の pointer、22 §16 の行番号、共通規則の「表の外」の 1 行）は正本の更新が要るので、本 packet と同じ commit で直した。
3. design → plan-draft（2026-10-11、起草役）: 上の 3 点は正本にあり、設計の値を変えない。owner の判断事項は残っていない（下の「owner の判断事項」）。起票時に決めた lane の割当て（棚卸しの計数の画面と型・棚卸し記録詳細のロス原価・棚卸しの実数の上限を ⑤ へ）は設計の値を変えず、`docs/backlog.md` に申し送った。未解決の設計の問いは無い。plan-gate へは Coordinator が進める。
4. plan-draft → plan-gate（2026-10-11、Coordinator）: packet と Matrix は起票の commit にある。Coordinator は起草役の割当て（1 lane のまま、棚卸しの計数の画面と型・棚卸し記録詳細のロス原価・棚卸しの実数の上限を ⑤ へ、回復の手順書を本 lane の Scope に置く）を packet と現物（`src-tauri/src/biz/product_service.rs:1199`〜`:1202`・`:1402`）で確かめて採った。前文の A / B の切れ目（条件 (a) を外す形）は採らない（A だけの main の「`cm` の商品は cm で入力し m で表示する」状態は Ordinary Operation に無く、その状態の L3 と Ready・merge が 1 周増える）。Contract Probe の P3（同梱の SQLite と v7 の列での作り直し）は、plan-approved の前に Coordinator が実測して P3 に結果を書く。
5. plan-gate（round 1 の是正、2026-10-11）: Plan Review round 1 の是正（Review Response 参照）。
6. plan-gate（round 2 の是正、2026-10-11）: Plan Review round 2 の是正（Review Response 参照）。
7. plan-gate（round 3 の是正、2026-10-12）: Plan Review round 3 の是正（Review Response 参照）。
8. plan-gate → plan-approved（2026-10-12、Coordinator、本 commit）: 独立の Plan Reviewer 2 本（Writer と別の context）が round 3 で P1・P2 が無いと報告した（Review Response 参照）。Plan Commit は round 3 の是正と記録の後の版。実装の commit はまだ無い。
9. plan-approved → implementing（2026-10-12、Coordinator、本 commit。state だけ）: Writer への発注の開始 HEAD を本 commit にする。実装は Scope の S1〜S7 の順。
10. implementing のまま Gated Amendment 1（2026-10-12、Coordinator）: Scope と Matrix の「既存 test の書換え」の表の漏れ 2 つと、正本と packet が決めていなかった表示の 1 点を足した（Review Response の「Gated Amendment 1」）。Goal・AC・設計の値は変えない。`Plan Commit` は変えず、起草役の commit を `Amendments` に足した。

## Owner Effort Budget

- 介入回数上限: 6（既定）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 6 | 1（2026-10-12: 集計の `点` の 3 桁区切り〈GA1-3。Coordinator が決めて伝え、owner が賛成した。予備から〉。lane の選択 TD-222 は本 lane の前の判断で数えない） | 4（R4 の承認 1、L3 の round 1〈L3-1〜L3-4 と、L3-3 の丸めの受入れの PASS / FAIL を 1 回で〉、Ready 1、merge 1） | 1 | 6 = 1 + 4 + 1 |

設計 lane（#154）は上限を 14 まで上げた。その主因だった「計算の後の値が安全な整数の範囲を越える経路」は入力の上限の形（TD-213）で正本が閉じており、本 lane は下の「横断の約束」の表で実装の所在を 1 か所にまとめる。L3 が 2 round 目に入るか、Plan Review が round 天井に達するときは、その判断と上限の改定を同じ 1 回で諮る。

既定値・数え方・上限に届くときの扱いは `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R4

Reason:
`products` の CHECK を変えるための表の作り直し（migration v8。v7 の時点で 8 つの表が FK で参照する親の表を DROP して作り直す。Contract Probe P3）と、移行の前の上限の検査で起動を止める経路、失敗と回復の契約（22 §16）を実装する。Risk Tiers の「Destructive data lifecycle」に当たる。Rust の enum・Tauri command の DTO・generated bindings・商品 CSV の検査・売上の CSV 出力も変える（R3 の要素）。Human Gate は `ready,merge,manual,r4`（owner が migration と回復の実装を承認し、Windows の実機で L3-1〜L3-4 を確かめる）、Final Review Minimum 2（R4 の Double Audit）。本番の DB は無い（`docs/project-memory.md`）が、移行の前の検査・失敗時に何も変えない・回復の手順は設計どおりに実装して test で確かめる。実店舗データの露出は無い。

required gate の green / red: gate の定義は変えない。実装は bindings と traceability の再生成、`design_compliance_test.rs` の対応表の更新を伴い、どれかを落とすと既存の required gate（Rust・bindings・traceability）が red になる（下の Registration / Generation Obligations）。Scope の予定 path を `scripts/ci/classify-changes.sh` の workflow の一覧（`:55`: `.github/workflows/*`・`scripts/ci/*`・`scripts/tests/*`・`scripts/*.sh` の gate・`scripts/pr-gate.py`・`.codex/bin/*`・`.claude/settings.json`・`.claude/hooks/*`・`.claude/agents/*`）に当てると当たらない（`src-tauri/tests/design_compliance_test.rs` は `scripts/tests/*` ではない）。Final Review Minimum 2 は R4 による。

## Goal

Goal Invariant:

### 最小完了条件

- 店の利用者が、12 個の単位で商品を登録でき、長さの商品を m で入庫・返品・手動販売・廃棄でき、在庫は cm の整数のまま保存され、数量を出す画面（棚卸しの計数を除く）が長さを m・単価を 1 m あたりで出す（共通規則 SPEC-UNIT-D1〜D3、TD-203）。
- 長さの商品の入庫の原価小計・廃棄のロス原価・手動販売の金額の初期値が 1 m あたりの価格で正しく出る（既知の不整合 (1)(2)(4)(5) の解消。SPEC-UNIT-D6・D7）。
- 数量・在庫・売価・原価（円）と入庫・廃棄の円の合計が、入力の上限と保存の前の検査で安全な整数の範囲に収まり、移行の前の DB も同じ範囲であることを v8 が確かめる（共通規則「入力の上限と安全な整数の範囲」、22 §16）。
- 商品 CSV が 12 個の単位（code と表示の語）と `m` の `初期在庫` を受け、不正な単位と単位の変更を行の error にする（SPEC-UNIT-D10、BIZ-01-D8）。

### 失敗定義

- 在庫の数量が浮動小数を通る、既存の `pcs`・`cm` の行の値か意味が変わる、新しい単位が在庫少から漏れる。
- v8 が行・列・index・FK を欠く、TX の中で失敗したのに表・値・版のどれかが変わるか `foreign_keys` が元の値に戻らない、上限の外の保存済みの値を通す。「失敗したら何も変わらない」は TX の中の失敗に限る: COMMIT が成功した後の `foreign_keys` の復元・再読取の失敗（22 §3.2 MNT-03-D1。`src-tauri/src/db/migration_tx.rs:136`〜`:172`）は `MigrationFailed`（接続破棄必須）を返すが、版 8 は記録済みで DB は新しい版として完全である。接続を破棄して再起動すれば開け（起動の `configure_database` が `foreign_keys = ON` を設定し直す: `src-tauri/src/db/mod.rs:197`）、旧版のアプリでは開けない（これは失敗定義に当たらない）。
- 上限の外の数量・在庫・売価・原価・円の合計が保存される経路が残る（UI を通らない command を含む）。
- 長さの商品の数量を cm の整数のまま出す画面（棚卸しの計数を除く）か、長さを点に足す集計が残る。
- 同じ明細から、記録詳細（BIZ の値）と、画面が TS の twin で求める金額（廃棄の保存の後の結果の欄のロス原価合計、手動販売の金額の初期値）で違う金額が出る。

### 非目的

- 原価の 1/100 円（migration vC、`_centi` の列と wire、原価の入力と表示、操作ログの原価の key。C の lane）。
- Z004 の取込みの再開と EJ の取込みの配線、POS の数量の 100 倍の整数と換算（`stock_quantity_from_pos`。P の lane）。
- 棚卸しの計数の画面の m の入力と表示、`StocktakeItemDetail`・`AdjustedItem` の `stock_unit`、棚卸しの実数の上限、確定の総額の検査、棚卸しの補正の note、棚卸し記録詳細のロス原価（⑤）。
- 商品フォームの在庫連動の自動提案のうち、準備照会（`get_pos_stock_readiness` の `ej_unverified`）に依る分岐（51 UI-01b-D22 の第 1 分岐）。準備照会の command は code にも bindings にも無く（`rg -l 'get_pos_stock_readiness|ej_unverified' src src-tauri/src` が 0 file）、入れるのは ㉘ ④（Z004・EJ の lane。`docs/backlog.md` の「㉘ ③ 〜 ⑤ の申し送り」）。
- 売価の小数、重さの単位、商品の単位の登録後の変更、在庫少の基準の画面、10 cm 刻みの検査。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

設計を含む変更の実装なので、店の通常の操作列を置く。例の金額は説明用の合成の値。本 lane の merge の後に下の 7 行が成り立つ。棚卸し（計数と確定）は停止中のまま（`src-tauri/src/biz/stocktake_service.rs:257`〜`:290`）で、m の棚卸しは ⑤ の後、原価の小数は C の後に成り立つ（**この文書を完了できる**ことと、**12 単位と m の通常運用をすべて達成できる**ことは別で、棚卸しと原価の小数は本 lane では未達）。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
|---|---|---|---|---|
| 商品登録画面 | 反物を単位 `m`、売価 700・原価 400（どちらも「1 m あたり」の label）、初期在庫 `25` で登録する | 在庫 2,500 cm で保存され、在庫照会で `25 m` | 保存できる | UI-01b-D22、SPEC-UNIT-D3 |
| 入庫画面 | 同じ反物を数量 `2.5`（m）・原価 333 で入庫する。保存の後の原価差分のダイアログは「見送って閉じる」を選ぶ | 在庫 2,750 cm（`27.5 m`）。入庫記録詳細の行の原価小計 `832.50 円`、原価合計 `¥833`。関連する在庫変動は `+2.5 m`・`27.5 m` | 保存できる | SPEC-UNIT-D6（J3）、UI-02-D15 |
| 手動販売画面（反物は JAN が無く PLU にしない） | 反物を追加して数量を `1.3` にする | 数量 1.3 m、金額の初期値 `910`。金額を直さず保存すると在庫 2,620 cm | 保存できる | UI-04-D18、SPEC-UNIT-D7（J4） |
| 日次売上画面（販売日） | 上の手動販売の日を開く | 反物の行が数量 `1.3 m`・単価 `¥700/m`・金額 `¥910`。販売点数は、その日の売上が反物だけなら `1.3 m`、ほかに個数の商品 3 個と 2 枚があれば `5 点・1.3 m` | 画面が出る | SPEC-UNIT-D3 の集計の規則（TD-206/207） |
| 廃棄画面 | 反物を `0.5` m、原価 333 で廃棄する | 在庫 2,570 cm（`25.7 m`）。保存の後の結果の欄のロス原価合計 `¥167`、記録詳細の行 `166.50 円`・合計 `¥167` | 保存できる | 廃棄の丸めは TD-023 に合わせた案。L3-3 で owner が確かめる |
| 商品 CSV の取込み | `在庫単位` に `玉`・`m`・`kg` を書いた CSV を読む（`m` の行の `初期在庫` は `25`） | 在庫照会で `m` の商品が `25 m`、毛糸の商品の単位が `玉`。`kg` の行と、既存の商品と違う単位で上書きする行は preview のエラー行（ほかの行は取り込める） | 取り込める | SPEC-UNIT-D10、BIZ-01-D8 |
| 在庫照会の在庫少 | 単位 `ball` の毛糸が一般の基準以下になる | 在庫少の一覧に出る | — | SPEC-UNIT-D10 |

失敗と再試行: 上限を越えた入力は画面が保存の前に入力の error を出し、直して保存し直す（BIZ は何も書かない）。更新の起動で v8 が上限の検査で止まったときは、DB は変わらず、更新を保留して管理者が対応を決める（22 §16「止まった後の進め方」）。v8 の COMMIT の後で `foreign_keys` の復元に失敗したときは、起動は「データベースの準備に失敗」の案内で止まり（`src-tauri/src/lib.rs:58`・`:192`）、再起動すれば v8 の DB として開く。

Plan Review は、この列が「正常な条件で目的を達成できるか」と「危険な結果を出さないか」を別々に答える（`docs/DEV_WORKFLOW.md` Review Rules）。

## owner の判断事項

無い。製品の振舞いの判断（J1〜J4、表示の範囲 TD-203、集計 TD-206/207、入力の上限 TD-213・TD-217）は設計 lane で決まり、正本と D-113 にある。本 lane で owner が答えるのは Human Gate（R4 の承認、L3-1〜L3-4、Ready、merge）だけで、廃棄の丸めの受入れは L3-3 の PASS / FAIL に含める。

店主に新しく聞く必要がある事項: 無い（単位・m・丸め・上限はどれも台帳の答えで足りる。TD-058・TD-147・TD-192・TD-195・TD-203・TD-206/207・TD-213・TD-217、L-135・L-136・L-214・L-215）。

起票時に起草役が決めたこと（設計の値を変えない lane の割当て。Coordinator が覆せる）:

- 棚卸しの計数の画面と型は ⑤ へ送る（archive の packet の申し送りは「U（型と画面）・⑤（新方式の確定）」）。理由: 棚卸しの開始・保存・確定は停止中で（`stocktake_service.rs:257`〜`:290`）、止まっているのは「新しく始める・数を保存する・確定する」の 3 つ（画面も開始・計数の入力・確定を押せなくする: `src/features/stocktake/StocktakePage.tsx:270`・`:302`・`:327`）。進行中の棚卸しを持つ既存の DB を開けば計数の一覧は出て（`:312`〜`:321` の一覧は停止では隠れない）、商品登録と商品 CSV の取込みは進行中の棚卸しへ明細を足す（`src-tauri/src/biz/product_service.rs:293`〜`:300`・`:1444`〜`:1446`）。その一覧の単位の表示は ⑤ まで今のまま。店は初導入で本番の DB はまだ無く（`docs/project-memory.md:135`）、進行中の棚卸しを新しく始められないので、L3 の到達手順（空の DB と画面の入力）では計数の一覧に行を出せない。新方式の計数は `#[cfg(test)]` の `time_evidence.rs` が別の型で持ち（`BeginCountResult.stock_unit` は既に `ProductStockUnit`: `src-tauri/src/biz/stocktake_service/time_evidence.rs:68`）、通常の command・画面・bindings への配線は ⑤ の仕事なので、U が旧い型と画面に `stock_unit` と m の入力を足しても ⑤ で作り直す。
- 棚卸し記録詳細のロス原価は ⑤ へ送る（申し送りは「U か棚卸し ⑤」）。理由: `docs/backlog.md` の「原価 × 数量 … 残り」の (3) が「⑤ で BIZ から返す形にするのが筋」と書き、合計の範囲の検査は棚卸しの確定（⑤）に置く（共通規則 (c) の「円の合計を wire・DB に新しく足す lane」）。棚卸しは停止中で新しい記録は増えない。
- 棚卸しの実数の上限は ⑤（申し送りは「U か ⑤」。`docs/backlog.md` の P3 が ⑤ に揃えると書く）。U は `time_evidence.rs` の N の範囲・文言・`safe`（`:264`〜`:271`・`:453`〜`:455`）と `time_evidence_tests.rs` を変えない。
- `ProductUpdates.stock_unit`（`src-tauri/src/db/product_repo.rs:167`・`:1045`〜`:1046`）は field ごと消す（申し送りは「常に `None`」）。単位を UPDATE する SQL が repo から無くなり、SPEC-UNIT-D2「登録の後に変えない」を型で守れる。`NewProduct.stock_unit: String`（`:136`）は内部の型のまま（BIZ が enum の `as_str` と、CSV の正規化した code から作る。D-061 / D-064 の二層）。
- migration の番号は v8（今の最大は v7: `src-tauri/src/db/migration.rs:71`〜`:73`。時点証拠の migration は未配線で、番号を引数で受ける: `src-tauri/src/db/schema_time_evidence.rs:165`）。

## 横断の約束（DB・wire に出す値ごとに、どこが保証するか）

共通規則「入力の上限と安全な整数の範囲」の (a)〜(e) を、本 lane が実装する所に当てた表。行番号は 2026-10-11 の HEAD `7070382f` の現物。Contract Ledger の SPEC-UNIT-D11 の行と Matrix の行はこの表の # を引く。上限の 4 つの値は定数で 1 か所に置く（Rust は `src-tauri/src/constants.rs`〈`CSV_IMPORT_LINE_LIMIT` が `:45`〉、TS は `src/lib/` の 1 file）。

| # | 値（DB の列 / wire） | 範囲と、保証する所 | 0 と符号 | 失敗の時点と順序 | 単位・種類の混在 |
|---|---|---|---|---|---|
| X1 | 1 明細の数量（`receiving_items`・`return_items`・`manual_sale_items`・`disposal_items` の `quantity`、wire の各 `*ItemInput.quantity`） | 1 以上 `9999999`。BIZ の 4 つの create の明細の検証（今の `quantity <= 0` の隣: `biz/inventory_service/receiving.rs:112`・`returns.rs:89`〈`validate_return_request_shape`〉・`manual_sale.rs:131`・`disposal.rs:109`）。画面は同じ定数で先に止める | 0 と負は今どおり `数量は1以上必要です` | TX を開く前の検証（入庫は `receiving.rs:111`〜`:124` が `:142` の `transaction()` より前）。在庫を動かす分岐より前に全明細を回るので、レジ処理済みの返品（`returns.rs:245` の分岐の外）も通る。何も書かない | 在庫の数量の単位で数える（個数は 1 単位、長さは cm）。BIZ は単位を引かず 1 つの上限。m の商品は画面が cm にして送る |
| X2 | POS 由来の数量（`sale_records.quantity`） | 絶対値 `9999999`。商品別売上 CSV の preview が行を採る所（`biz/csv_import_service/parse.rs:92`〜`:134`） | 数量 0・金額 0 の行は今どおり捨てる（`:94`）。符号は今のまま | preview の行の error（`invalid_number`）。その行は `sale_records` にも在庫にも書かない。ほかの行は取り込む。確定と取消は今の build で停止中（`commit.rs:18`〜`:31`）で、test は `legacy_commit_csv_import`（`:40`）を通す | Z004 の数量は整数のまま（P の換算は別 lane）。在庫に連動しない商品の行も同じ検査 |
| X3 | 在庫（`products.stock_quantity`、`inventory_movements.stock_after`、wire の `stock_quantity`・`stock_after`） | 絶対値 `999999999`。在庫を書く所: `apply_stock_change` の計算の後（`biz/inventory_service/common.rs:54`〜`:58`。入庫・返品・手動販売・廃棄・商品別売上 CSV の取込みが通る）、商品登録の初期在庫（`biz/product_service.rs:957` の隣）、商品 CSV の初期在庫（preview `:1204`〜`:1213` と、commit の新規の行 `:1403`）、取込みの取消の補正（`biz/csv_import_service/commit.rs:257`〜`:274`。和と補正後を checked で求めて検査）、整合性の補正（`biz/integrity_service.rs:172`〜`:173`）。repo の `update_product` も `ProductUpdates.stock_quantity`（`db/product_repo.rs:166`・`:1040`〜`:1044`）で在庫を書けるが、本体で `Some` を入れる所は無く（`rg -n 'stock_quantity: Some' src-tauri/src` が 0 件。唯一の構築は `product_service.rs:1382` の `None`）到達経路が無いので検査を置かない。この field を消すのは時点証拠の契約（20 `:9`、⑤）で決まっていて、本 lane は消さない | 在庫は 0 を受け、負は INV-3 のまま `-999999999` まで。**初期在庫は 0 以上**（負は拒む）: create は今も拒む（`product_service.rs:957`〜`:961`）。商品 CSV は今は拒まない（preview は `parse::<i64>` の値をそのまま採り、commit は `unwrap_or(0)`）ので、本 lane で preview と commit に足す | 書く前に `ValidationFailed`（`在庫数が大きすぎます`）。TX の中で見つけたら TX 全体を戻す。商品 CSV の preview は行の error: 負と規則に合わない文字列は `初期在庫の値が不正です: '{値}'`（`pcs,-1` も `m,-1` も同じ文）、上限超えは `在庫数が大きすぎます`。commit は新規の行の `initial_stock` を TX の中で 0 以上 `999999999` と確かめ、外れれば `ValidationFailed` で TX 全体を戻す。上書きの行は `初期在庫` を使わない（`:1382` の `stock_quantity: None`）: preview は新規の行と同じ規則で行の error にし、commit は読まない。`在庫単位` が行の error の行は `初期在庫` を読まない（error は単位の 1 つだけ） | 個数も長さも同じ上限（長さは `9999999.99` m）。`初期在庫` を m で読むのは正規化した `在庫単位` が `m` の行だけ（空と `cm` と個数は整数）。棚卸しの確定の後の在庫（`stocktake_service.rs:571`、旧本体は test だけ）と実数は ⑤ |
| X4 | 変動の数量（`inventory_movements.quantity`） | 個別の検査を置かない（X1・X3 から絶対値 `1999999998` 以下。共通規則 (b)）。v8 の上限の検査だけが見る | — | — | — |
| X5 | 売価（`products.selling_price`、`price_history.old_selling`・`new_selling`、wire の `selling_price`・`new_selling_price`） | 0 以上 `999999`。商品の create・update・価格改定・商品 CSV（今の負の検査の隣: `product_service.rs:947`・`:994`・`:471`・`:1178`〜`:1183`）と、商品 CSV の commit（`commit_import` が新規の行〈`:1402` 以降の INSERT の前〉と上書きの行〈`:1370`〜`:1386` の UPDATE の前〉の両方を TX の中で 0 以上 `999999` と確かめる。今は commit に検査が無く、負も通る）。画面も同じ定数（商品フォームと、一括価格改定の画面の新売価: `src/features/products/components/PriceRevisionTable.tsx:25`〜`:29` の `parsePrice`） | 0 は受ける | 検証で `ValidationFailed`（`売価が大きすぎます`）、商品 CSV は行の error。何も書かない。画面は同じ文を欄の error に出し、command を呼ばない | 値は「価格の基準数量」ぶん（長さは 1 m あたり）。単位で上限を変えない |
| X6 | 原価（円。`products.cost_price`、`receiving_items`・`disposal_items` の `cost_price`、`price_history.old_cost`・`new_cost`、wire の `cost_price`・`new_cost_price`） | 0 以上 `999999`。`product_service.rs:952`・`:1001`・`:476`・`:1185`〜`:1190`、商品 CSV の commit（X5 と同じ site で、新規と上書きの全行）、入庫と廃棄の明細（`receiving.rs:118`・`disposal.rs:115`）。画面は商品フォーム、入庫・廃棄の原価欄、一括価格改定の画面の新原価（利用者の入力と、自動で入る案の両方。案が `999999` を越えたら欄の error で、直すまで確定できない） | 0 は受ける | X5 と同じ（`原価が大きすぎます`） | C の lane が 1/100 円の `99999999` へ書き換える。本 lane は円の整数のまま |
| X7 | 行の金額（wire の `line_cost_centi`・`line_loss_cost_centi`。DB には持たない） | 1/100 円の整数の十進の文字列（i128 を `to_string`）。BIZ の記録詳細（`biz/inventory_service/list.rs:112`・`:181` の後）が `cost_line_centi` で求める。X1・X6 から `999998900000100` 以下 | 数量は X1 で 1 以上。原価 0 は `"0"` | 読取りは範囲で失敗しない。負の原価・数量は保存の時点で入らない（`cost_line_centi` は負を `ValidationFailed` にする） | 行ごとに、その商品の単位の基準数量で割る（個数 1、長さ 100）。単位の混ざった伝票も行ごと |
| X8 | 円の合計（wire の `total_cost`・`total_loss_cost`。入庫・廃棄は DB に持たない） | `±9007199254740991`。`create_receiving`（`receiving.rs:53`）と `create_disposal`（`disposal.rs:62`）が、最初の書込みの前に明細から `cost_line_centi`・`cost_total_yen` で求めて `check_safe_integer` に通す | 合計 0 は通る | 保存の前に `ValidationFailed`（`合計金額が大きすぎます`）で何も書かない。記録詳細の読取りには検査を置かない（保存の前に通した値と同じになる） | 行を 1/100 円にしてから足すので、単位をまたいで足せる |
| X9 | 売上の数量の集計（wire の `count_points`・`length_cm`、`DailySaleItem.quantity`） | 読取りを範囲で拒まない（共通規則 (d)）。BIZ の和は checked（今は plain: `biz/sales_service.rs:229`・`:244`） | 部門別・合計は両方 0 なら `0 点`。月次の商品別は `stock_unit` で `0 m` か `0 点`。集計の数量の表示（`N 点`、`N 点・L m`。部門小計・合計・販売点数のカード〈日次・ホーム〉・月間販売点数・月次の商品別と部門別の行）の `N` は 3 桁区切りを付ける（長さの部分の D3 の形 `1,234.56 m` と同じ向き。符号は先頭で、区切りは絶対値の桁に付ける）: `0 点`、`999 点`、`1000` → `1,000 点`、`1234` → `1,234 点`、`1234`・`180` → `1,234 点・1.8 m`、返品が上回る日は `-5 点`・`-1234` → `-1,234 点`。今の画面は所によって違う（月次の商品別の行は `toLocaleString("ja-JP")` で区切りあり: `src/features/monthly-sales/components/ProductRankingTable.tsx:89`。日次の表の小計・合計・明細も区切りありで語なし: `src/features/daily-sales/components/ProductTable.tsx:90`・`:110`・`:136`。販売点数のカード 3 つは `String()` で区切りなし: `src/features/daily-sales/components/SummaryCardsBar.tsx:54`・`src/features/home/components/SummaryCards.tsx:43`・`src/features/monthly-sales/components/SummaryCardsBar.tsx:50`）ので 1 つの形に揃える。見た目が変わるのは 1,000 点以上（か -1,000 点以下）のカードだけ（`1234 点` → `1,234 点`）。売上の CSV の月次の部門別の行は、`length_cm` が 0 なら今と同じ整数（`count_points`。両方 0 は `0`。区切りも語も付けない: `1234`）、0 でなければ画面と同じ文字列（`N` の 3 桁区切りを含む: `1,234 点・1.8 m`。`,` を含む field は今の exporter が `"…"` で括る: `src-tauri/src/io/report_csv_exporter.rs:52`〜`:53`。長さが 1,000 m 以上の `1,234.56 m` と同じ扱い）。売上の CSV の商品の行の長さの数は、単位の語と桁区切りを付けない m の数（`130` → `1.3`、`123456` → `1234.56`、`200` → `2`、`0` → `0`、返品の `-130` → `-1.3`）。画面と在庫警告の `format_length_m`（`1,234.56 m`）とは別の形 | i64 の溢れは `ValidationFailed`、SQL の `SUM` の溢れは SQLite の error（今どおり）。TS の再集計は `Number.isSafeInteger` を外れたら `—` | 個数の 10 単位は `count_points`、`m`・`cm` は `length_cm`。種類は BIZ が `stock_unit_kind` で決め、長さを点に足さない（repo は単位ごとの行を返すだけで種類を知らない）。数量の並べ替えは日次の明細・月次の商品別・月次の部門別とも `(count_points, length_cm)` の辞書順（日次の明細は行の単位で数量をこの組に当てる） |
| X10 | 整合性の差と補正（wire の `IntegrityMismatch`・`StockAdjustment`、`integrity_fix` の detail_json の各 adjustment、`integrity_check` の detail_json の各 mismatch） | 差の演算を checked に（今は plain: `integrity_service.rs:77`・`:165`・`:172`）。補正の後の在庫は X3 | 差 0 は今どおり不一致に出さない | 読取り（`run_integrity_check`）は範囲で拒まない。補正は X3 の検査で TX 全体を戻す | 行ごとに `stock_unit` を運び、画面が行の単位で出す。key の無い記録済みの log は整数のまま。`integrity_check` の log は key を書くだけで、操作ログ画面の表示は今のまま（36） |
| X11 | 手動販売の数量の初期値と金額の初期値（画面だけ。wire は今の `quantity`・`amount`） | 追加したときの数量の初期値は価格の基準数量ぶん: 個数の商品 `1`、`m` の商品 `1`（100 cm）、`cm` の商品 `100`（62 UI-04-D18。今は全部 `"1"`: `src/features/manual-sale/lib/manual-sale-row-utils.ts:23`）。金額 = `売価 × 数量 ÷ 基準数量` を円未満で四捨五入（X1・X5 から `9999989000001` 以下で、number の整数の演算で正確） | 長さの商品で数量が入力の error（空・0・規則に合わない・上限超え）の間は金額を求めず、未編集の金額欄は空にする（数量が直れば式の値に戻る）。編集済みの金額欄は変えない | 画面の twin。保存は数量の入力の error で止まる。順序: 追加 → 数量を変えるたびに未編集なら求め直す → 金額を編集したら追従を止める → 同じ商品の再追加は数量に基準数量ぶんを足し（個数 +1、`m` は `1.3` → `2.3`、`cm` は `130` → `230`）、金額は長さの商品で未編集なら式で求め直し、それ以外（個数の商品、金額を編集した長さの商品）は売価を足す | 求め直すのは長さの商品（`m`・`cm`）だけ（個数の商品は今の振舞い）。入庫・返品・廃棄の数量の初期値と再追加は今どおり（`cm` の商品は `1` と +1、`m` の商品は `1` と +1 m） |
| X12 | 単位の code（`products.stock_unit`、wire の `ProductStockUnit`） | 12 個の code。DB の CHECK（v8）、Rust の enum、repo の `parse_stock_unit`（`db/product_repo.rs:62`）、商品 CSV の preview と commit の検査。全 variant の列挙は `ProductStockUnit::ALL`（S2） | 商品 CSV の空は `pcs`（今どおり） | 一覧に無い値: 商品 CSV は preview の行の error（その行の `初期在庫` は読まず、error は単位の 1 つ）、wire は deserialize の error、DB の読取りは error | 登録の後に変えない。商品 CSV の上書きで既存と違う単位は行の error、commit は TX の中で再検証して `ValidationFailed` |
| X13 | 移行の前に保存済みの値 | v8 の手順 2 が、X1〜X6 の列と `stocktakes.total_cost`、入庫・廃棄の記録ごとの和を上限で確かめる（22 §16 の「vU の上限の検査」の表） | NULL の行は除く | 何かを変える前に ROLLBACK して `MigrationFailed`（message に `上限の検査` と表名・列名）。表・値・版が変わらず、`foreign_keys` を元の値に戻す（TX の中の失敗。版は 7 のままで旧版でそのまま開ける） | — |

画面の入力の error の文（上限超え）は BIZ と同じ 4 つを使う（共通規則 (a) の文の並びのとおり）: 数量欄は `数量が大きすぎます`（`m` の欄も同じ）、商品登録の初期在庫の欄は `在庫数が大きすぎます`、売価の欄は `売価が大きすぎます`、原価の欄は `原価が大きすぎます`。形式の error の文は今のまま（個数・`cm` の数量欄 `数量は1以上の整数で入力してください`、`m` の欄は SPEC-UNIT-D3 の 2 つの文、一括価格改定の画面 `0以上の整数で入力してください`）。商品別売上 CSV の preview（X2）の行の error の message は `数量が大きすぎます`。

## Scope

本 lane が変える file と目的（行番号は 2026-10-11 の HEAD `7070382f` の現物で数え直した）。実装は S1 → S7 の順に積む（S1 は振舞いを変えない土台。前文の「別の切れ目」を採る場合は、A = S1・S3・S4 と S5・S6 の既存 2 単位の分、B = S2 と S5・S6 の m の入力・商品フォーム・商品 CSV で切れる）。

### S1 土台（振舞いを変えない）

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| `biz::unit_amount`（新 module） | 共通規則の「シグネチャ」の関数。`StockUnitKind`・`stock_unit_kind`・`price_basis_quantity`・`cost_line_centi`・`cost_total_yen`・`sale_amount_yen`・`check_safe_integer`・`format_length_m`。`stock_quantity_from_pos` は作らない（P） | `src-tauri/src/biz/unit_amount.rs`（新設）、`src-tauri/src/biz/mod.rs` | 今の 3 関数（`biz/stocktake_service.rs:441`・`:453`・`:480`、`valuation_overflow` `:448`）を移す。呼出し: 旧本体 `legacy_complete_stocktake` の `:561`〜`:564`・`:599`、新方式の `biz/stocktake_service/time_evidence.rs:8`（import）・`:702`〜`:711`。どちらも原価の引数を `i128::from(円) * 100` にして値を変えない。今の test（`stocktake_service.rs:1407`〜`:1487`）は `unit_amount` へ移し、同じ期待で回す |
| 設計と code の検査の対応表 | `10-common-rules.md` を `SKIP_DOCS` から外し、`build_doc_to_modules_map()` に `"10-common-rules.md" => ["biz::unit_amount"]` を足す | `src-tauri/tests/design_compliance_test.rs:25`〜`:26`・`:108` | 未文書化の `pub fn` は `:726`〜`:738` で fail する |
| 上限の定数 | 1 明細の数量 `9999999`・在庫 `999999999`・売価 `999999`・原価（円）`999999`、安全な整数 `9007199254740991` | `src-tauri/src/constants.rs`、TS は `src/lib/` の 1 file（新設） | S3・S5 の検証がここから引く |

### S2 migration v8・enum・在庫少・商品 CSV の単位

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| migration v8（vU） | 22 §16 の手順 1〜7 と「vU の上限の検査」。関数は `pub(crate)`。`foreign_keys` の退避と復元は v2 と同じ helper（`db/migration_tx.rs`）を使う。22 §16 の文を SQL に落とすときの解釈は次の 1 通り: (1) 手順 2 の TX は 22 の文のとおり `BEGIN`（v7 と同じ: `db/schema_v7.rs:18`） (2) 手順 6 (c) が比べる「作り直しの前の `stock_unit` の値ごとの件数」は、手順 2 で行数と一緒に読む (3) `products_new` の列の順は v7 の時点の `pragma_table_info(products)` の順（v1 の 17 列の後に v3 の `plu_target` が 18 番目: `db/schema_v3.rs:16`）。作り直しの後は `sqlite_master` の `products` の定義の表名が引用符つきになるので、test は定義の文字列を照合せず `pragma_table_info`・index・動作で比べる。COMMIT と ROLLBACK は共有の helper（`commit_transaction`・`rollback_after_error`）だけを使う（MNT-03-D1） | `src-tauri/src/db/schema_v8.rs`（新設）、`src-tauri/src/db/mod.rs`、`src-tauri/src/db/migration.rs:12`〜`:18`（use）・`:71`〜`:73` の後（登録） | `schema_v1.rs:40` の CHECK は書き換えない（migration の履歴）。既存の test `migration.rs:1378`〜`:1386`（`init_database` の DB で `kg` を CHECK が拒む）は、v8 の後も `kg` が 12 code に無いので期待は同じ。最新の版を 7 と固定する既存の test は 11 行（`rg -n 'assert_eq!\(.*\b7\b' src-tauri/src/db/migration.rs src-tauri/src/db/schema_v2.rs` の 10 行と、2 行に分かれた `migration.rs:722`〜`:723`: `migration.rs:706`・`:716`・`:723`・`:758`・`:794`・`:858`・`:874`、`schema_v2.rs:378`・`:586`・`:596`・`:601`）で、8 に書き換える（`schema_v2.rs` は test の 4 行だけを変え、migration の本体は変えない）。MNT-03-D1 の構造の test（`migration.rs:358`〜`:383`）の file の列挙に `schema_v8.rs` を足す。`app_max_version() + 1` を使う helper と時点証拠の test（`db/test_support.rs:14`〜`:23`・`:60`〜`:66`、`db/schema_time_evidence.rs` の test）は版を数字で書かないので変えない。新しい test は Matrix の MNT-03-D13 の行（注入は今の `FailurePoint` だけを使い、`db/migration_tx.rs` は変えない） |
| `ProductStockUnit`（12 variant） | SPEC-UNIT-D1 の 12 code。全 variant の列挙 `ProductStockUnit::ALL`（12 個の定数の配列）を同じ file に足す（今は列挙の出どころが無い: `:35`〜`:46`） | `src-tauri/src/db/product_repo.rs:35`（定義）・`:41`〜`:46`（`as_str`）・`:62`〜`:69`（`parse_stock_unit`） | `biz/product_service.rs:32`・`:248`、`db/product_repo.rs:89`・`:305`。test: `src-tauri/src/finite_enum_contract_tests.rs:63`（対の表）・`:134`（`rejects!`）・`:188`〜`:189`（`parity!`）を 12 個に。`ALL` が全 variant を持つことは、同じ file の test で、wildcard の無い `match` で variant を数える関数と `ALL` の長さ・重複なし・対の表の variant の並びとの一致で止める。bindings `src/lib/bindings.ts:1272`（再生成） |
| `ProductUpdates.stock_unit` を消す | SPEC-UNIT-D2 | `db/product_repo.rs:167`・`:1045`〜`:1046` | 作る所は `biz/product_service.rs:1377` だけ（`rg -n 'stock_unit: row.stock_unit' src-tauri/src` で確かめる） |
| 在庫少の SQL | 個数の 10 code を一般の基準、`m`・`cm` を生地の基準で比べる。db の層から biz を呼ばない（`src-tauri/tests/architecture_test.rs:35`〜`:39` が db → biz を禁じる）ので、BIZ の `list_low_stock`（`biz/product_service.rs:910`〜`:925`）が `ProductStockUnit::ALL` を `stock_unit_kind` で個数と長さの 2 つの並びに分けて repo へ渡し、repo は受けた並びで 2 つの `IN (…)` を組む（正本の SPEC-UNIT-D10・44 の SQL の形のまま）。シグネチャ: `list_low_stock_products(conn, count_units: &[ProductStockUnit], threshold_count: i64, length_units: &[ProductStockUnit], threshold_length: i64, include_discontinued: bool)`。code は bind で渡す | `db/product_repo.rs:1230`〜`:1235`（シグネチャ）・`:1258`〜`:1259`（SQL）。呼ぶ所は `biz/product_service.rs:920` だけ（`rg -n 'list_low_stock_products\(' src-tauri/src`） | `stock_unit_kind` は wildcard の無い `match` なので、variant を足すと compile で止まり、`ALL` の抜けは上の行の test が止める。test: `cmd/inventory_cmd.rs:201`〜`:218`（`LS-CM`）は編集せずに通り、`ball`・`m` と 12 個の code を 1 つずつ置く行を足す |
| 商品 CSV の `在庫単位`・`初期在庫`・上書き | code か表示の語を受けて code に正規化、不正は preview の行の error。`m` の行の `初期在庫` は m の数。`初期在庫` は 0 以上（負は行の error。X3）で、`在庫単位` が行の error の行では読まない。既存の商品と違う単位は行の error、commit は wire の行を信頼せず TX の中で再検証する: 単位（新規の行は 12 code、上書きの行は既存と同じ）、売価・原価は新規と上書きの全行で 0 以上 `999999`、初期在庫は新規の行だけ 0 以上 `999999999`（上書きの行の初期在庫は読まない）。外れは最初に見つけた 1 件で `ValidationFailed` にして TX 全体を戻す。文は preview と同じ（`売価が大きすぎます`・`原価が大きすぎます`・`在庫数が大きすぎます`、負は今の preview の文 `売価は0以上で入力してください`・`原価は0以上で入力してください`、初期在庫の負は `初期在庫の値が不正です: '{値}'`） | `biz/product_service.rs:1199`〜`:1202`（`在庫単位`）・`:1204`〜`:1213`（`初期在庫`）・`:1307`（duplicate_rows）・`:1370`〜`:1386`（上書きの UPDATE。`:1374`〜`:1375` が売価・原価、`:1377` が単位）・`:1402`〜`:1403`（空 → `pcs`、`initial_stock` の `unwrap_or(0)`）。`ImportRow.stock_unit`（`:1041`）は `Option<String>` のまま（正規化した code を入れる） | 既存の test: `:3294`（空 → `pcs`、そのまま）、`:3324`（`kg` で取込み全体が rollback → preview の行の error と、preview を通さない commit の `ValidationFailed` に書き換える。契約の変更、SPEC-UNIT-D10）、`:3376`〜`:3394`（上書きで単位を保つ、そのまま）。足す test は Matrix |
| demo の seed | `"cm"` の 2 分岐を数量の種類の分岐に、売上の金額を `sale_amount_yen` で | `src-tauri/src/seed_demo.rs:237`・`:246`・`:469` | `src-tauri/tests/seed_test.rs`。seed の商品の単位（`:46` ほか）は増やさない |

### S3 入力の上限と検査（BIZ）

横断の約束の X1〜X3・X5・X6・X8・X9・X10 の「保証する所」の列が、変える file:line の全部。加えて:

- 価格改定の既存の test `test_revise_product_price_req105_persists_large_values_exactly`（`biz/product_service.rs:4261`。売価 9,999,999・原価 9,999,998 の保存の成功を固定）を、上限の端（売価 999999・原価 999999）の保存の成功と、1 つ外（1000000）の拒否に書き換える（契約の変更による書換え、SPEC-UNIT-D11 の続きの (a)。値が正確に保存されることの確認は残す）。
- 既存の `biz/inventory_service/common.rs` の test（`:146`〜`:298` の `apply_stock_change` の呼出し）は上限の外の値を使わない（`rg -n 'i64::MAX|999_999|9_999' src-tauri/src/biz/inventory_service/` が 0 件）ので、期待を変える必要は無い見込み。変える必要が出たら Writer は止まって返す。
- 文言: `数量が大きすぎます`・`在庫数が大きすぎます`・`売価が大きすぎます`・`原価が大きすぎます`・`合計金額が大きすぎます`（明細の検証は今の `明細{n}: ` を前に付ける）。

### S4 wire の型と BIZ の値

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| 記録詳細の明細 6 型の `stock_unit: String` → `ProductStockUnit` | SPEC-UNIT-D11 | `db/receiving_repo.rs:52`・`:243`、`db/disposal_repo.rs:111`・`:661`、`db/return_repo.rs:110`・`:294`、`db/manual_sale_repo.rs:68`・`:180`、`db/sales_repo.rs:134`・`:512`、`db/stocktake_repo.rs:116`・`:613`（`parse_stock_unit` を `pub(crate)` にして使う） | bindings の 6 型（`src/lib/bindings.ts:465`・`:663`・`:910`・`:1360`・`:1432`・`:1631` の定義）。TS の fixture: `src/features/inventory-records/` の `CsvImportRecordDetailPage.test.tsx`・`DisposalRecordDetailPage.test.tsx`・`DisposalRecordDetailRoute.test.tsx`・`OtherRecordDetailPages.test.tsx`・`ReturnToFlow.test.tsx`・`StocktakeRecordDetailPage.test.tsx` |
| 入庫・廃棄の行の金額（IO → BIZ） | IO の掛け算をやめ、BIZ が `cost_line_centi`・`cost_total_yen` で求める。wire は `line_cost` → `line_cost_centi: String`、`line_loss_cost` → `line_loss_cost_centi: String`。合計は円の整数のまま。型の置き場: wire の型（`ReceivingRecordDetail`・`ReceivingRecordDetailItem`・`DisposalRecordDetail`・`DisposalRecordDetailItem`）は今の repo の file に置いたまま名前を変えず、repo は数量・原価・単位を読んで金額の field を空（`String::new()`・0）で返し、BIZ が必ず埋める（今の `movement.source` を BIZ が埋めるのと同じ形: `list.rs:113`〜`:117`）。別の行の型は作らない | IO: `db/receiving_repo.rs:55`・`:229`・`:246`・`:250`・`:287`、`db/disposal_repo.rs:116`・`:647`・`:666`・`:670`・`:704`。BIZ: `biz/inventory_service/list.rs:112`・`:181` | repo の test `db/receiving_repo.rs:603`〜`:604`、`db/disposal_repo.rs:1094`・`:1098`（値は BIZ の test へ移す。個数の商品の値は今と同じ）。frontend: `src/features/inventory-records/ReceivingRecordDetailPage.tsx:164`、`DisposalRecordDetailPage.tsx:170`。TS の fixture は上の行の 4 file（`rg -l 'line_cost\|line_loss_cost' src --glob '*.test.*'`） |
| 売上の wire と集計 | `DailySaleItem` に `stock_unit`。`DeptSubtotal`・`GrandTotal`・`MonthlySaleItem` の `quantity` を `count_points`・`length_cm` に置き換え、`MonthlySaleItem` に `stock_unit: Option<ProductStockUnit>`。和は種類ごとに checked。前月の比較（`prev_month_comparison: Option<Vec<MonthlySaleItem>>`、`:135`・`:285`）も同じ型で変わる。月次の部門別は、repo が部門と商品の単位で GROUP BY した行を返し、BIZ が部門ごとに種類で分けて checked で足し（金額も）、金額の降順に並べ直す。金額が同じ行の並びは今のまま変えない（`key` の昇順: `biz/sales_service.rs:574`。部門別の `key` は部門 ID を文字列にしたもの〈`:562`〉なので文字列の比較で、部門 ID 2 と 10 が同額なら 10 が先）（SQL に単位の並びを書かない。34 冒頭が併記する `SUM(CASE …)` の案は採らない）。CSV の `数量` は 34 冒頭と X9 のとおり。集計の行（月次の部門別）の文字列を作る `csv_total_quantity`（`biz/sales_service.rs:477`〜`:483`）は、`count_points` と `length_cm` の両方が 0 でない枝の `N 点` の `N` に 3 桁区切りを付ける（今は `format!("{count} 点・{}", …)` で区切りなし: `:481`。X9。`length_cm` が 0 の枝の整数と、`count_points` が 0 の枝の `format_length_m` は変えない） | `biz/sales_service.rs:40`（`:45`）・`:62`（`:65`）・`:71`（`:72`）・`:141`（`:144`）、構築 `:208`・`:229`〜`:230`・`:237`・`:244`・`:552`・`:564`、CSV `:337`・`:364`・`:388`。repo の SELECT に `p.stock_unit`: `db/sales_repo.rs:997`（`:1004`・`:1009`）・`:1340`（`:1348`・`:1352`）・`:1373`（`:1381`・`:1386`）と行の型 `:909`・`:921`・`:930` | Rust の test: `sales_service.rs:707`・`:737`・`:801`（`grand_total.quantity` → `count_points`。個数の商品だけなので値は同じ、`length_cm` 0）・`:760`・`:766`（部門小計、同じ）、月次の `:976`（`report.items[0].quantity` 4 → `count_points` 4・`length_cm` 0・`stock_unit` が `pcs`）、`biz/csv_import_service/tests/cross_feature_tests.rs:327`・`:571`〜`:572`（`grand_total.quantity` → `count_points`）。repo の test（`db/sales_repo.rs:2977`〜`:3081` の 8 つの呼出し）は行の field を読むだけで、個数の商品だけの fixture なので行の数と値は変わらない（行の型に `stock_unit` が増えるだけ）。型に合わせた書換えで弱めない。足す test は Matrix |
| 整合性チェック | `IntegrityMismatch`・`StockAdjustment` に `stock_unit`。新しく書く操作ログの detail_json にも `stock_unit` を書く: `integrity_fix` の各 adjustment と、`integrity_check` の各 mismatch（36 `:62`。`integrity_check` の JSON は応答の DTO と別に組むので、DTO に足すだけでは入らない） | `biz/integrity_service.rs:26`・`:44`・`:67`・`:79`〜`:85`（mismatch の構築）・`:92`〜`:104`（`integrity_check` の JSON）・`:175`〜`:180`・`:185`〜`:192`（`integrity_fix` の JSON）、`db/product_repo.rs:1149`〜`:1151`（`find_all_stock_quantities` の戻り値を 3 つ組から単位つきへ。呼ぶ所は `integrity_service.rs:67` だけ） | 書き換える test: `integrity_service.rs:524`〜`:527`（`integrity_fix` の adjustments の完全一致 → `stock_unit` の key を足した期待）、`db/product_repo.rs:2515`〜`:2529`（`test_find_all_stock_quantities_req301`。3 つ組の完全一致 → 単位つきの同じ値）。`:635`〜`:636`・`:762` は field を読むだけで変わらない。足す test: `integrity_check` の JSON の各 mismatch の `stock_unit`（Matrix） |
| PLU 書出しの未書出しの一覧 | `ProductResponse` に `stock_unit`。cmd の層は `crate::db` を引けない（`src-tauri/tests/architecture_test.rs:44`〜`:47`）ので、`ProductStockUnit` を `src-tauri/src/biz/mod.rs:20`・`:22` の `pub use crate::db::product_repo::{…}` に足し、cmd は `crate::biz::ProductStockUnit` で引く。この re-export は D-083 の allow list（`src-tauri/tests/architecture_test.rs:260`〜`:403` の `DB_IO_REEXPORT_ALLOWLIST`。今は 30 エントリ: `biz/mod.rs` 28・`biz/product_service.rs` 1・`mnt/backup.rs` 1）に `("biz/mod.rs", "ProductStockUnit", "crate::db::product_repo::ProductStockUnit")` の 1 エントリを足して通す（31 エントリ。`docs/decision-log.md:687` の D-083 の Impact が定める追加の経路: 「allow list への今後の追加は architecture_test.rs の const 編集で行い、無審査の追加を防ぐことが目的のため追加自体を禁止しない」）。検査（`biz_mnt_direct_db_io_reexport_allowlist`、`:514`）は re-export と allow list の両方向の差分なので、re-export と allow list を同じ commit で足す（片方だけでは落ちる）。`stock_unit` は必須の field で型は `ProductStockUnit`（`String` にしない: SPEC-UNIT-D11） | `src-tauri/src/cmd/plu_export_cmd.rs:62`・`:198`〜`:199` 付近、`src-tauri/src/biz/mod.rs:20`・`:22`、`src-tauri/tests/architecture_test.rs:260`〜`:403`（allow list の 1 エントリ。Matrix の書換えの表） | TS の fixture: `src/features/backup-restore/BackupRestorePage.flow.test.tsx`、`src/features/home/HomePage.test.tsx`、`src/features/home/hooks/useHomeSummary.test.tsx`、`src/features/home/components/PluNotificationBar.test.tsx`、`src/features/plu-export/PluExportPage.test.tsx`（`rg -l 'listPluDirty\|makePluDirty\|list_plu_dirty' src --glob '*.test.*'` の 5 file。型で止まる所に `stock_unit` を足す） |
| 在庫警告の文 | 長さの商品の `stock_after` を `format_length_m` で整形 | `biz/inventory_service/receiving.rs:209`・`returns.rs:264`・`manual_sale.rs:280`・`disposal.rs:201` | 個数の商品の文は今どおり。4 つの BIZ の test に `m`・`cm` の例 |
| bindings | `cd src-tauri && cargo run --bin generate_bindings` | `src/lib/bindings.ts` | 上の全部 |

### S5 frontend の純関数（`src/lib/` と formatter）

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| m の数量の入力（文字列 → cm）と、入力の行の加算 | SPEC-UNIT-D3 の入力と加算の規則。欄ごとの上限と 0 の扱いを引数で受ける | `src/lib/` の新しい file（`src/lib/request-helpers.ts:15` の `parseRequiredSafeInteger` の隣。個数と `cm` の欄にも同じ上限を掛ける）。`parseRequiredSafeInteger` 自体の契約は変えない（`src/lib/request-helpers.test.ts:81`〜`:85` が `String(Number.MAX_SAFE_INTEGER)` を受けることを固定している）。上限は新しい関数の側に置く | 使う所は S6。今の文 `数量は1以上の整数で入力してください` を期待する test（`rg -c -F` で `ReceivingPage.test.tsx` 6・`ManualSalePage.test.tsx` 7・`request-helpers.test.ts` 4・`DisposalPage.test.tsx` 2・`manual-sale-request.test.ts` 1）は個数・`cm` の商品の文として残る。4 つの request module の `parseRequiredSafeInteger(` の出現回数を完全一致で固定する test（`src/lib/request-helpers.test.ts:33`〜`:50` の `REQUEST_MODULES` の `integerCallCount` と `:314`〜`:326`。今は receiving 2・manual-sale 2・disposal 4・return-exchange 1）は、数量の欄の parse が新しい関数へ移るので件数の期待が変わる（Matrix の書換えの表。同じ test のほかの条件は変えない） |
| formatter | param を `ProductStockUnit` にして `default` を消す（`never` で受ける）。長さは `m`・`cm` とも m の形。`count_points`・`length_cm` を `5 点・1.8 m` にする関数（`N 点` の `N` は 3 桁区切り: `1,234 点`・`1,234 点・1.8 m`・`-1,234 点`、1,000 未満は `999 点`。X9。販売点数のカード 3 つ・月次の商品別と部門別の行・日次の表の小計と合計の `点` は、どれも formatter の同じ区切りの規則で出し、画面の側で `String()` と `点` をつないで組まない。両方が 0 でない組は Rust の `csv_total_quantity` と同じ golden）、単価 `¥700/m`、行の金額（1/100 円の文字列 → `832.50 円`。`^[0-9]+$` に合わない入力〈空の文字列を含む〉は `—`） | `src/features/stock-inquiry/lib/format-stock-display.ts:15`・`:21`・`:34`・`:40` | 既存の test `format-stock-display.test.ts:25`〜`:26`・`:43`〜`:44`（`"kg"`・`""` → `—`）は、一覧に無い値が型で渡せなくなるので消し、12 個の表の test に置き換える（契約の変更、SPEC-UNIT-D9）。`N cm` を期待する test（`rg -c '[0-9] cm' src --glob '*.test.*'` の 9 file・12 行）は m の形へ書き換える（SPEC-UNIT-D3 の表示、TD-203） |
| 金額の twin | 廃棄の画面の合計（保存の後の結果の欄の「ロス原価合計」: `src/features/disposal/DisposalPage.tsx:320`〜`:321`。入力の途中には出ない。`BigInt` で行を 1/100 円・合計を円で丸め、今と同じ `¥` の形で出す。範囲外は `—`）、手動販売の金額の初期値（number の整数の演算） | `src/features/disposal/lib/disposal-request.ts:38`〜`:40`、`src/features/manual-sale/lib/manual-sale-row-utils.ts:9`・`:24`・`:40` | golden の表を Rust と TS の test に独立に写す（SPEC-UNIT-D9） |

### S6 frontend の画面

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| 行の型の `stockUnit: string` → `ProductStockUnit` | SPEC-UNIT-D11 | `src/features/disposal/types.ts:10`、`receiving/types.ts:6`、`return-exchange/types.ts:10`、`manual-sale/types.ts:9` | 構築: `disposal-row-utils.ts:48`、`receiving-row-utils.ts:13`、`return-exchange-row-utils.ts:31`、`manual-sale-row-utils.ts:20` |
| 入庫・返品・手動販売・廃棄の数量欄と加算 | m の商品は m の入力。再追加と統合は cm の整数で足す。数量欄の属性（今は 4 画面とも `type="number"`・`inputMode="numeric"`・`min="1"`: `ReceivingPage.tsx:578`〜`:580`・`DisposalPage.tsx:550`〜`:552`・`ManualSalePage.tsx:617`〜`:619`・`ReturnExchangePage.tsx:865`〜`:867`）は、`m` の商品の行だけ `step="0.01"`・`min="0.01"`・`inputMode="decimal"` にし、個数と `cm` の行は今のまま。4 画面に `<form>` は無く（`rg -n '<form' src/features/receiving src/features/disposal src/features/manual-sale src/features/return-exchange --glob '!*.test.*'` が 0 件）、`min`・`step` は native の検証に掛からず spinner と矢印 key にだけ効く。値の検証は m の入力の純関数が持つ | 加算: `receiving-row-utils.ts:4`（`:27`）、`return-exchange-row-utils.ts:4`（`:49`）・`:9`（`:91`）、`disposal-row-utils.ts:22`（`:69`）・`:27`（`:94`）、`manual-sale-row-utils.ts:4`（`:39`）。request: `receiving-request.ts:51`〜`:54`、`disposal-request.ts:38`・`:60`〜`:67`、`return-exchange-request.ts:74`〜`:75`、`manual-sale-request.ts:52`〜`:55` | 画面: `ReceivingPage.tsx:575`、`DisposalPage.tsx:466`・`:521`・`:568`、`ReturnExchangePage.tsx:782`・`:833`・`:883`、`ManualSalePage.tsx:558`・`:611`・`:635`。test: 各 `*-row-utils.test.ts`・`*-request.test.ts`（廃棄の row-utils は test を新設） |
| 手動販売の数量と金額 | X11 のとおり。追加したときの数量の初期値は個数 `1`・`m` `1`・`cm` `100`、金額の初期値は式（`cm` の商品・売価 700 → 数量 `100`・金額 700）。長さの商品は未編集の間、数量に合わせて金額を求め直す（`130` → 910）。金額を編集したら追従を止める（row に編集の有無を持つ）。再追加は数量に基準数量ぶん（`cm` は 130 → 230、`m` は `1.3` → `2.3`、個数は +1）、金額は長さの商品で未編集なら式で求め直し、それ以外は売価を足す。数量が入力の error の間、長さの商品の未編集の金額欄は空 | `manual-sale-row-utils.ts:4`〜`:13`（`nextQuantity`・`nextAmount`）・`:23`〜`:24`（初期値）・`:39`〜`:40`（再追加）、`manual-sale/types.ts`、`ManualSalePage.tsx` の数量・金額の入力欄（単位を出す `:635` の付近） | 書き換える test: `manual-sale-row-utils.test.ts:8`〜`:28`（`cm` の商品の初期値 `quantity: "1"`・`amount: "120"` → `"100"`・`"120"`。UI-04-D18）。`:31`〜`:40`（個数の商品の再追加 `2`・`300`）はそのまま。足す test は Matrix の X11 の行。`ManualSalePage.test.tsx` |
| 商品フォーム | select を 12 個の表示の語に、長さの単位の価格欄に「1 m あたり」、`m` の初期在庫は m の入力、売価・原価・初期在庫に上限。在庫連動の自動提案は今の規則を 12 単位へ広げるだけ: 未操作の間、長さの単位（`m`・`cm`）を選ぶと `false`、個数の単位を選ぶと `true`（今は `cm` → `false`・それ以外 → `true`）。説明の文も長さの単位で出す。準備照会に依る分岐（`ej_unverified` の間は全単位で `false`）は実装しない（㉘ ④）。修正の画面は既存の商品の単位をそのまま出す（2 値の写像をやめる）。価格の節の説明「売価と原価は税抜の整数で入力します。」（`ProductForm.tsx:349`）を UI-01b-D22 の文の、原価の lane の前の形（売価は税込の円の整数、原価も円の整数、長さの商品は 1 m あたり）に直す。修正の画面の read-only の「現在庫」（今は `String(product.stock_quantity)` で数だけ: `product-form-request.ts:71`、欄は `ProductForm.tsx:404`〜`:416`）は、12 単位とも `formatStockDisplay(stock_quantity, stock_unit)` の形で出す（`m`・2750 → `27.5 m`、`pcs`・20 → `20 個`、在庫 0 → `0 m`・`0 個`、負 → `-0.3 m`）。edit では現在庫を検証も送信もしない（今どおり。`buildUpdateProductRequest` は売価・原価だけを読む: `:142`〜`:143`）。初期在庫の欄の `inputMode`（今は `numeric`: `ProductForm.tsx:409`。`type="number"` ではない）は、create で単位が `m` のときだけ `decimal` | `src/features/products/components/StockUnitField.tsx:45`〜`:51`（提案）・`:62`（2 値の写像）・`:69`〜`:70`（select の 2 項目）・`:88`〜`:92`（説明の文）、`ProductForm.tsx:349`・`:352`・`:364`・`:404`〜`:416`（初期在庫・現在庫の欄）、`src/features/products/lib/product-form-request.ts:44`・`:52`・`:70`（2 値の写像）・`:71`（現在庫）・`:84`〜`:86`・`:126`・`:142`〜`:143` | `StockUnitField.test.tsx`（`pcs` → `cm` で `false`・`cm` → `pcs` で `true` の今の test はそのまま通る）、`ProductForm.test.tsx`、`product-form-request.test.ts` |
| formatter の呼出し 22 か所 | 型を合わせる（表示は formatter が決める） | `rg -n 'formatStock(Display\|UnitLabel)\(' src --glob '!*.test.*'` から定義の 2 行を除いた 22 行: 上の入出庫 4 画面の 10、`inventory-records/` の `CsvImportRecordDetailPage.tsx:209`・`ManualSaleRecordDetailPage.tsx:174`・`ReceivingRecordDetailPage.tsx:158`・`DisposalRecordDetailPage.tsx:164`・`StocktakeRecordDetailPage.tsx:46`・`:50`・`:193`・`ReturnRecordDetailPage.tsx:208`、`stock-inquiry/components/StockDetailContent.tsx:82`・`ProductListTable.tsx:100`、`products/components/ProductTable.tsx:79`、`stock-movements/StockMovementsPage.tsx:136` | 各画面の test |
| 在庫変動の表 | `MovementTable` に必須の prop `stockUnitByProductCode: ReadonlyMap<string, ProductStockUnit>`。行の商品の単位で数量と変動後の在庫を出す | `src/features/stock-movements/components/MovementTable.tsx:62`・`:74`・`:79`、`lib/movement-formatters.ts:20` | 呼ぶ所 7 つ: `StockMovementsPage.tsx:261`、`inventory-records/` の `ReceivingRecordDetailPage.tsx:190`・`ReturnRecordDetailPage.tsx:234`・`ManualSaleRecordDetailPage.tsx:201`・`DisposalRecordDetailPage.tsx:197`・`CsvImportRecordDetailPage.tsx:286`・`StocktakeRecordDetailPage.tsx:233`（記録詳細は `detail.items` から対応を作る）。test: `MovementTable.test.tsx` と記録詳細の test |
| 売上の画面 | 明細は数量 `1.3 m`・単価 `¥700/m`。小計・合計・販売点数は点と m の 2 本。月次の部門別の表に「数量」の列を足す（部門・数量・売上・構成比・前月比の 5 列。`5 点・2.5 m`）。数量の列は商品別と同じく並べ替えの対象にし、`?sortBy=quantity&mode=by_department` は `(count_points, length_cm)` の辞書順で並ぶ（今の「部門別では数量で並べない」防御の分岐は無くなる） | 日次: `src/features/daily-sales/components/ProductTable.tsx:33`・`:90`・`:110`・`:136`、`lib/calculate-unit-price.ts:15`〜`:16`、`lib/group-items.ts:30`・`:39`、`lib/sort-items.ts:53`、`components/SummaryCardsBar.tsx:54`、`DailySalesPage.tsx:129`、`src/features/home/components/SummaryCards.tsx:43`。月次: `monthly-sales/components/ProductRankingTable.tsx:89`、`lib/compute-summary.ts:16`、`components/SummaryCardsBar.tsx:50`、`lib/sort-items.ts:62`、`lib/pick-top-ranking.ts:26`、`types.ts:20`〜`:26`（並べ替えの定義 `MONTHLY_SORT_DESCRIPTORS` の数量: `departmentLabel: null`・`modes: ["by_product"]` → 部門別でも `数量`）・`:95`（`ProductRankingRow.quantity`）・`:103`〜`:109`（`DeptCompositionRow` に `count_points`・`length_cm`）、`lib/compute-composition.ts:22`〜`:31`、`components/DepartmentTable.tsx:57`〜`:83`（見出し）と行、`lib/compute-comparison.ts`・`hooks/useMonthlySalesReport.ts:78`〜`:84`（型の追随）、`MonthlySalesPage.tsx:107` | fixture と test: `rg -l 'grand_total\|subtotal\|DailySaleItem\|MonthlySaleItem\|DeptSubtotal\|GrandTotal' src --glob '*.test.*' --glob '**/test-fixtures.ts'` と月次の `quantity` を作る test を合わせた 20 file（`rg -l 'MonthlySaleItem\|makeMockMonthlyItem\|prev_month_comparison\|ProductRankingRow\|makeMockProductRankingRow\|totalQuantity\|DailySaleItem\|makeMockDailyItem\|grand_total\|subtotals?\b\|DeptSubtotal\|GrandTotal\|DeptCompositionRow' src --glob '*.test.*' --glob '**/test-fixtures.ts'`: `daily-sales/` の 5、`home/` の 3、`monthly-sales/` の 11〈`MonthlySalesPage.test.tsx`・`hooks/useMonthlySalesReport.test.tsx`・`components/` の `ProductRankingTable.test.tsx`・`SummaryCardsBar.test.tsx`・`DepartmentTable.test.tsx`・`lib/` の `compute-comparison.test.ts`・`compute-composition.test.ts`・`compute-summary.test.ts`・`pick-top-ranking.test.ts`・`sort-items.test.ts`・`test-fixtures.ts`〉、`backup-restore/BackupRestorePage.flow.test.tsx`）。`DepartmentTable.test.tsx:115`〜`:124` の「`sortBy='quantity'` は入力順のまま」の test は、数量の順に並ぶ期待へ書き換える。並べ替えの定義の完全一致（`src/features/products/search.test.ts:230`〜`:257`）と、部門別の見出しの列挙と呼出し回数（`DepartmentTable.test.tsx:16`〜`:20`・`:64`〜`:71`）も期待が変わる（Matrix の書換えの表）。公式日報の数量（`DailySalesPage.tsx:233`〜`:285`、`MonthlySalesPage.tsx:194`）は D-104 のまま変えない |
| 一括価格改定の画面の上限 | 新売価・新原価を 0 以上 `999999` で検証する（X5・X6）。越えた欄は `売価が大きすぎます`・`原価が大きすぎます` を出し、確定の button は押せず command を呼ばない。新売価から自動で入る新原価（案）が `999999` を越えた場合も同じ（案はそのまま欄に入れ、欄の error を出す。利用者が直せば確定できる）。案の式（SPEC-PRV-D4）は変えない | `src/features/products/components/PriceRevisionTable.tsx:25`〜`:29`（`parsePrice`）・`:49`〜`:50`（invalid の判定）・`:68`〜`:77`（`payload` が null なら確定しない）・`:136`〜`:138`・`:155`〜`:157`（欄の error の文） | `src/features/products/PriceRevisionPage.test.tsx` に境界の test を足す（Matrix の X5・X6 の行） |
| 整合性チェック・操作ログ・PLU 書出し | 行の単位で出す。操作ログは `stock_unit` の key がある記録だけ m | `src/features/integrity-check/IntegrityCheckPage.tsx:296`・`:368`・`:371`・`:385`・`:460`、`src/features/operation-logs/OperationLogsPage.tsx:90`〜`:120`（parse）・`:182`〜`:183`（旧在庫 → 新在庫）・`:186`〜`:187`（差分。`stock_unit` の key がある長さの商品は `差分 -0.5 m`）、`src/features/plu-export/PluExportPage.tsx:676` | `IntegrityCheckPage.test.tsx`、`OperationLogsPage.test.tsx` |
| 記録詳細の金額 | 行の金額を文字列の formatter で出す（個数の商品も `222.00 円` の形。今は `¥222`）。同じ行に原価（`¥111`、今の `formatYen` のまま）と行の金額（`222.00 円`）が並び、合計は `¥1,221` のまま。行の金額だけを `N.NN 円` にするのは正本（SPEC-UNIT-D6 の行の金額の wire と表示）のとおりで、本 lane で揃え直さない | `ReceivingRecordDetailPage.tsx:164`、`DisposalRecordDetailPage.tsx:170`（合計 `:123`・`:125` は今の `formatYen`） | 上の fixture。期待が変わる test: `OtherRecordDetailPages.test.tsx:183`・`:187`（行の金額 `¥222`・`¥999` → `222.00 円`・`999.00 円`。Matrix の書換えの表）。廃棄の記録詳細の test に行の金額の表示の assert は無い（`rg -n '¥' src/features/inventory-records/DisposalRecordDetail*.test.tsx` の hit は合計だけ） |

### S7 設計正本の proposed の注記を現行の記述へ直す（実装と同じ PR）

本 lane が実装した契約の「proposed・未実装、D-113」を外し、現行の本文にする。C・P・⑤ の分は残し、注記に lane の名前（`原価の lane`・`Z004・EJ の lane`・`棚卸しの再開`）を入れて区別する（AC8）。対象は `rg -n 'proposed' docs/function-design docs/db-design docs/DB_DESIGN.md docs/SCREEN_DESIGN.md | rg 'D-113'` の行（2026-10-11 の起票の commit の後で 62 行。うち更新履歴の表の行 7 は書き換えない）:

| file | 直す節・行 | 残す分 | 他 lane との重なり |
|---|---|---|---|
| `docs/function-design/10-common-rules.md` | 「単位・数量・金額の共通規則」の見出しと前文、SPEC-UNIT-D1〜D4・D6・D7・D9〜D11、「入力の上限と安全な整数の範囲」、シグネチャ、更新履歴。D10 の「SQL の code の並びは全 variant から作り」は、並びを作るのが BIZ（`ProductStockUnit::ALL` と `stock_unit_kind`）で repo は受けるだけ、と書く。D3 の「表示する所」の表に、商品修正の画面の現在庫（F の行: `ProductForm` の read-only の欄、`formatStockDisplay` の形）を 1 行足す（TD-203 の「数量を出す画面はすべて」の範囲で、値は変えない）。D6 の「棚卸し記録詳細のロス原価（…どの lane が BIZ へ移すかは Plan Packet の申し送り）」（`:132`）と (a) の「棚卸しの実数の上限と確定の検査は棚卸しの側（Plan Packet の申し送り）」（`:166`）を「棚卸しの再開（⑤）」に直す。D3 の「表の外」に、数量の生の値で比べるだけの所（在庫少の一覧の並べ替え `src/features/stock-inquiry/lib/filter-low-stock-list.ts:62`〜`:63`、商品検索の「在庫数」の並べ替え `src/features/products/search.ts:30`。単位をまたいで数の大小で並ぶのは今のまま）を 1 行足す。D3 の集計の規則の表示の文（`:115`「表示は、`count_points ≠ 0` か両方 0 なら `N 点`…」）に、`N` は 3 桁区切りを付ける（`1,234 点`・`1,234 点・1.8 m`・`-1,234 点`。CSV の集計の行の文字列も同じで、`length_cm` が 0 の部門の整数には付けない）を書き、同じ項の「個数の商品だけの日は値も表示も今と同じ」を、販売点数のカードは 1,000 点以上で区切りが付く、と正す（X9） | D5（C）、D8 と `stock_quantity_from_pos`（P）、D3 の表の 7・D11 の `StocktakeItemDetail`・`AdjustedItem`・棚卸しの確定の検査（⑤） | なし |
| `docs/function-design/22-mnt-migration.md` | §16 の見出しと vU（版 v8、MIGRATIONS 登録順の 1 文を足す）。手順 2 に「`stock_unit` の値ごとの件数も読む」を足す（手順 6 (c) の比べる相手。値は変えない）。回復の冒頭の文（`:371`「失敗すれば失敗した migration の変更と版だけが戻る」）を、TX の中の失敗と、COMMIT の後の `foreign_keys` の復元の失敗（MNT-03-D1: 版は残り、DB は新しい版として完全で、接続を破棄して再起動すれば開ける）に分ける（既存の D1 を引くだけで、設計の値は変えない） | vC（C） | `agent/backup-offsite` が §3 の `:62` の 1 行を変える。本 lane の所有は §16 だけ |
| `docs/db-design/master-tables.md`、`docs/DB_DESIGN.md`（`:104`・`:188`）、`docs/db-design/transaction-tables.md` | CHECK の 12 code、D-4 の適用ルール、行の金額と合計の丸め | 原価の 6 列（C） | なし |
| `docs/function-design/30-biz-product-service.md`・`31-biz-inventory-service.md`・`34-biz-sales-service.md`・`36-biz-integrity-check.md`（`:62`）・`21-io-inventory-repo.md`（`:106`・`:358`）・`20-io-product-repo.md`（`:940`）・`40-cmd-product.md`（`:189`）・`44-cmd-inventory.md`（`:543`・`:658`・`:964`） | 冒頭の「単位…の契約」の節と proposed の注記のうち、単位・上限・行の金額・集計の分を本文へ。30 `:25` の「今はどれも負だけを拒む（… 初期在庫 `:957`・`:1204`〜`:1213`）」は商品 CSV の初期在庫について現物と違う（今は負を拒まない）ので、現行の文に直すときに「create と商品 CSV の初期在庫は 0 以上」を事実どおりに書く。34 冒頭（`:9`）の月次の部門別は「部門と単位で GROUP BY して BIZ で種類ごとに足す」の 1 つにし、`SUM(CASE …)` の併記を落とす。44 §23.9（`:938`〜`:962`）のシグネチャと SQL の概要を S2 の形（2 つの並びを受ける）に直し、`:964` の「code の並びは全 variant から作る」を「BIZ が `ProductStockUnit::ALL` から作って渡す」に合わせる | 原価の `_centi`（C） | `agent/backup-offsite` は 43 を変える（本 lane は触らない） |
| `docs/function-design/51-ui-product-form.md`（`:47`・`:177`・`:180`）・`58-ui-stock-inquiry.md`（§58.6）・`61`（`:27`・`:116`）・`62`（`:38`・`:121`・`:156`・`:176`）・`63`（`:33`・`:128`）・`64`（`:28`・`:115`）・`66`（`:65`）・`74`（`:64`）・`75`（`:40`）・`56`（`:113`・`:336`・`:504`）・`57`（`:71`・`:296`） | 該当の行の proposed の括弧書きを現行の文に。51 §7.5 の手順 4（`:129`「`stock_unit='cm'` に変更した場合…`pcs` に戻した場合…」）を「長さの単位に変えたら `false` を提案、個数の単位に変えたら `true` を復元」に直す。51 に、修正の画面の現在庫を単位つきで出す 1 行を足す。57 の「部門別に数量の列が無い」を固定する行（`:105` の 4 列、`:160` の `compute-composition` の出力、`:194` の並べ替えの注、`:397`〜`:398` の列と SortableHeader の 3 列）を 5 列・数量を並べ替えの対象にする形に直す | 61・64 の原価の小数（C）、51 UI-01b-D22（`:47`）の原価の小数の文（C）。51 `:47` の「`get_pos_stock_readiness` が `ej_unverified` を返す間は全単位で `false`」の分岐は未実装のまま残し、注記を `proposed・未実装（Z004・EJ の lane〈㉘ ④〉、D-113）` にする（冒頭の `:7`・`:11` の準備照会の文は D-113 の注記ではなく、触らない） | なし |
| `docs/function-design/53-ui-home.md`（`:236`） | 昨日売上カードの点数の括弧書き | — | `agent/backup-offsite` が `:38`・`:211`〜`:218`・`:325` 付近を足す。本 lane の所有は `:236` の括弧書きだけ |
| `docs/function-design/67-ui-plu-export.md`（`:64`） | `PluDirtyProductTable` の括弧書き | — | `agent/plu-clear` が `:52`・`:132`・`:187` 付近を足す。本 lane の所有は `:64` の括弧書きだけ |
| `docs/SCREEN_DESIGN.md`（`:251`・`:285`・`:308`・`:324`） | 生地の表示の 4 行の括弧書き | — | `agent/backup-offsite` が `:84`・`:113`・`:163` 付近を変える。本 lane の所有は上の 4 行だけ |
| `docs/function-design/35-biz-stocktake-service.md`（§20.5a `:473`） | 3 関数が `biz::unit_amount` にあること | 確定の新方式と note（⑤） | なし |
| `docs/function-design/73-ui-stocktake.md`（`:17`）、`23-io-z004-parser.md`、`29-io-ej-parser.md`、`77-ui-bulk-price-revision.md` | ⑤・P・C の契約は触らない（注記に lane の名前を入れる 1 語の書換えだけ）。77 は、新売価・新原価の入力の項に上限の 1 行を足す（0 以上 `999999`、越えた欄は error で確定できない、案が越えたときも同じ。画面の契約は 77 が持つ。BIZ の側は 40 `:189`） | 全部 | `agent/pos-series-design` が 23・29・32 の付近を変える見込み。本 lane は 23・29 の該当の括弧書きの 1 語だけで、衝突したら取り込んだ後に書き直す |
| `docs/migration-recovery-runbook.md`（新設）、`docs/PROJECT_HANDOFF.md`（参照先の 1 行） | 22 §16 の回復の 7 手順を、版 v8 と file の名前で写した管理者の手順書（正本は 22 §16 で、写しであることを冒頭に書く。古い backup を新しい版のアプリで復元すると復元の中で migration が当たり、上限の検査で止まれば復元は巻き戻されて今の DB のまま、の 1 文を入れる。22 §16「runtime の lane が L3 の前に手順書へ写す」） | — | なし |
| `docs/function-design/90-traceability.md` | 再生成（手で編集しない） | — | 生成物。衝突したら取り込んだ後に再生成する |
| `docs/backlog.md`（「単位の拡張」と「原価 × 数量 … 残り」の項）、`docs/decision-log.md` | backlog は本 lane の項だけ。decision-log は触らない | — | backlog は `agent/backup-offsite`（`:85`〜`:90` 付近）・`agent/pos-series-design` と別の項。merge 順で両方を残す |

起票の commit で直したもの（設計 lane の Final Review の P3 の U の分）: `61`〜`64` の明細表の「生地は `cm` 単位を主表示にする。」の文に SPEC-UNIT-D3 の proposed の pointer（`61-ui-receiving.md:116`・`62-ui-manual-sale.md:121`・`63-ui-return-exchange.md:128`・`64-ui-disposal.md:115`）、22 §16 の `stocktake_recounts` の列の行番号（`schema_time_evidence.rs:120`〜`:124`）、共通規則の (e) 表の外に在庫少の基準の 1 行、`docs/backlog.md` の「単位の拡張」の項。

触らない: `docs/Plans.md`（D-097）、`docs/spec/requirements.md`、`docs/db-design/tracking-system-tables.md`、`docs/function-design/71-mnt-backup.md`・`69-ui-threshold-settings.md`・`25-io-plu-formatter.md`、`src-tauri/src/db/schema_v1.rs`〜`schema_v7.rs`・`schema_time_evidence.rs`、`src-tauri/src/io/`（Z004・EJ・日報の parser）、`src/features/stocktake/`、`src-tauri/src/biz/stocktake_service/time_evidence_tests.rs`。

予期しない拡張（上の表に無い file を変える必要が出た）は、Writer が編集前に止まって Coordinator へ返す。

### C・P・⑤ へ送るもの

| 送り先 | 内容 | 根拠 |
|---|---|---|
| C（原価） | migration vC、原価の 6 列と Rust の型・wire の 14 型・frontend、原価の上限 `999999` 円 → `99999999`（1/100 円）への書換え、操作ログの原価の key、`seed_demo.rs:232`、SPEC-PRV-D4、L3-6、`docs/db-design/tracking-system-tables.md` の列の表 | archive の packet `docs/archive/plans/2026-10-07-unit-extension.md` の申し送りの表の C の行 |
| P（Z004・EJ） | `stock_quantity_from_pos` と Z004・EJ の数量の 100 倍の整数、EJ の照合、POS の数量の換算の後の上限、設計 lane の Contract Probe P3（レジの丸め）。取込みの取消の結果の `StockCorrection`（`src-tauri/src/biz/csv_import_service/mod.rs:246`〜`:250`。在庫の値を持つが単位が無い）と `csv_rollback` の detail_json の `stock_corrections`（`rollback.rs:77`〜`:81`）に単位を足すこと（取消は停止中: `rollback.rs:17`）。商品フォームの在庫連動の提案のうち準備照会（`ej_unverified`）に依る分岐（㉘ ④。`docs/backlog.md` の「㉘ ③ 〜 ⑤ の申し送り」に書いた） | 同、P の行。共通規則 SPEC-UNIT-D8。準備照会の割当ては `docs/backlog.md` の ④ と ADR `docs/adr/2026-09-18-stocktake-time-evidence.md` の「㉘ への引継ぎ」（`:388`〜`:398`） |
| ⑤（棚卸しの再開） | 棚卸しの計数の画面の m の入力と表示（`src/features/stocktake/StocktakePage.tsx`・`lib/stocktake-formatters.ts`・`hooks/useUpdateCount.ts`、73、SPEC-UNIT-D3 の表の 7）、`StocktakeItemDetail`・`AdjustedItem` の `stock_unit`、棚卸しの補正の note の m の形、棚卸しの実数の上限（`time_evidence.rs` の N の範囲と文言、`time_evidence_tests.rs` の端の値）と `safe` を `check_safe_integer` に寄せること、確定の総額の検査、棚卸し記録詳細のロス原価を BIZ から返す形と合計の検査、L3-5 | 上の「起票時に起草役が決めたこと」。`docs/backlog.md` の「単位の拡張」の項に書いた |

## Non-scope

- 上の「C・P・⑤ へ送るもの」の全部。
- 売価の小数、重さの単位、箱と、ばらした個数を別々に数える model、巻と m の換算、商品の単位の登録後の変更、在庫少の基準の画面の語と m の表示、10 cm 刻みの検査（D-113 の非目的のまま）。
- レジの日報（Z001・Z005）の数量（D-104）、PLU の単価（SPEC-UNIT-D4、[25](../../function-design/25-io-plu-formatter.md)）。
- 記録済みの文字列と値の書換え（在庫変動の `note`、操作ログの detail_json・summary）。
- 依存の追加・更新（`package.json`・`Cargo.toml`・lockfile は変えない）。

## Acceptance Criteria

baseline は 2026-10-11 に本 worktree（HEAD `7070382f3444b5fb648f2787c705af83e1cdf3c5`、code は未編集）で逐語に実行した出力。完了時の期待値と分けて書く。

- AC1（単位 12 種が DB から bindings まで通る）: `rg -o '"(pcs|sheet|hon|bag|box|roll|kumi|set|ball|cho|m|cm)"' <(rg '^export type ProductStockUnit' src/lib/bindings.ts) | sort -u | wc -l` が `12`（baseline `2`）。`cd src-tauri && cargo test --lib finite_enum_contract_tests` が全 PASS。
- AC2（migration v8）: `rg -c 'version: 8,' src-tauri/src/db/migration.rs` が `1`（baseline: 一致なしで exit `1`）。`cd src-tauri && cargo test --lib db::` が全 PASS で、Matrix の MNT-03-D13 の行の test を含む。
- AC3（IO と画面が金額を掛け算しない）: `rg -n 'quantity \* (ri|di)\.cost_price|quantity \* costPrice' src-tauri/src src --glob '!*.test.*'` が 0 行（baseline 3 行: `src-tauri/src/db/receiving_repo.rs:229`・`src-tauri/src/db/disposal_repo.rs:647`・`src/features/disposal/lib/disposal-request.ts:40`）。Matrix の SPEC-UNIT-D6 の行の test が PASS（長さの商品 2.5 m・原価 333 の行が `"83250"`、合計 833）。
- AC4（formatter と行の型が enum）: `rg -c 'default:' src/features/stock-inquiry/lib/format-stock-display.ts` が一致なし（baseline `2`）。`rg -n 'stockUnit: string' src --glob '!*.test.*' | wc -l` が `0`（baseline `4`）。`rg -n 'pub stock_unit: String' src-tauri/src/db/receiving_repo.rs src-tauri/src/db/disposal_repo.rs src-tauri/src/db/return_repo.rs src-tauri/src/db/manual_sale_repo.rs src-tauri/src/db/sales_repo.rs src-tauri/src/db/stocktake_repo.rs | wc -l` が `0`（baseline `6`）。
- AC5（在庫少が全単位を分ける）: `rg -n "stock_unit = '(pcs|cm)'" src-tauri/src --glob '!*test*'` が 0 行（baseline 2 行: `src-tauri/src/db/product_repo.rs:1258`・`:1259`）。Matrix の SPEC-UNIT-D10 の行の test が PASS。
- AC6（上限と検査）: Matrix の「横断の約束」X1〜X3・X5・X6・X8・X13 の行の test が全 PASS（UI を通らない command の呼出しで、上限の端は保存でき、1 つ外は `ValidationFailed` で何も書かれない）。
- AC7（生成物）: `cd src-tauri && cargo run --bin generate_bindings && git diff --exit-code -- ../src/lib/bindings.ts` が exit 0、`cd src-tauri && cargo run --bin generate_traceability -- --check` が exit 0。
- AC8（設計正本が現行の記述になっている）: `rg -n 'proposed・未実装' docs/function-design docs/db-design docs/DB_DESIGN.md docs/SCREEN_DESIGN.md | rg 'D-113' | rg -v '^[^:]+:[0-9]+:\| 20' | rg -v '原価の lane|Z004・EJ の lane|棚卸しの再開' | wc -l` が `0`（baseline `49`、起票の commit の作業木。1 つ目の `rg -v` は更新履歴の表の行を除く）。本 lane が実装した契約の注記は外れ、残る注記は lane の名前を持つ。この式で見えない行は、S7 の後に行ごとに読んで、U の分が現行の文になっていることを確かめる: U と別の lane の契約が同じ行にある行（今 lane の名前を含む 5 行: `20-io-product-repo.md:940`・`35-biz-stocktake-service.md:473`・`40-cmd-product.md:189`・`44-cmd-inventory.md:543`・`:658`。S7 の後は `51-ui-product-form.md:47` も）と、式に掛からない書き方の行（`62-ui-manual-sale.md:176` の `（proposed、D-113）`）。
- AC9（gate）: `cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test` と `npm run typecheck && npm run lint && npm run format:check && npm test && npm run build` が exit 0。L3 の前に `cd src-tauri && cargo check --release` が exit 0。`bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan` が ERROR 0。
- AC10（L3）: 下の L3-1〜L3-4 を owner が Windows の実機で PASS とし、`python3 scripts/pr-gate.py status --pr NUMBER` の出力の manual が pass になる。
- 既存の保護: 既存の `pcs`・`cm` の商品の値と意味を変えない（v8 の前後で `products` の全列の値と定義が同じ、棚卸しの評価額の今の test が同じ期待で通る）。既存 test の期待を変える所は Matrix の「既存 test の書換え」の表が全部で、それ以外の既存 test は消さず・弱めず全 PASS。

## Design Readiness

- 引用する設計正本（節まで）: `docs/function-design/10-common-rules.md` の「単位・数量・金額の共通規則」SPEC-UNIT-D1〜D4・D6・D7・D9〜D11 と「入力の上限と安全な整数の範囲」・シグネチャ／`22-mnt-migration.md` §16（MNT-03-D13 の vU・vU の上限の検査・回復・テスト）と §3.2（`migrate`）／`docs/db-design/master-tables.md` の「単位と原価の精度の契約」／`docs/DB_DESIGN.md` の CHECK 制約方針・D-4／`30-biz-product-service.md` BIZ-01-D8・§4.8・§4.9／`31-biz-inventory-service.md` の「単位と原価の精度の契約」・§12.2・§12.6a／`21-io-inventory-repo.md` §10.2・§10.5／`34-biz-sales-service.md` 冒頭・§19.5／`36-biz-integrity-check.md`／`44-cmd-inventory.md`／`51-ui-product-form.md` UI-01b-D5・D6・D22／`56-ui-daily-sales.md` §56.10／`57-ui-monthly-sales.md` §57.6／`58-ui-stock-inquiry.md` §58.6／`61`〜`64`（UI-02-D7・UI-04-D18・UI-03-D12・UI-05-D8）／`66` UI-06c-D5／`67`／`74` UI-11c-D14／`75` UI-13-D3／`35-biz-stocktake-service.md` §20.5a（値は変えない）／decision-log D-061・D-064・D-104・D-113。
- 必要な設計成果物: function-design・DB・画面の設計 = existing sufficient（PR #154 で更新済み。本 lane は実装と同じ PR で proposed の注記を現行へ直す = updated in this PR）／migration の番号（v8）= updated in this PR（22 §16）／回復の手順書 = updated in this PR（新設）／decision-log = existing sufficient（D-113）。
- plan にしかない durable な判断の昇格先: 無い。起票時に決めた lane の割当て（棚卸しの分を ⑤ へ）は `docs/backlog.md` の「単位の拡張」の項に書いた。v8 の番号は実装の PR で 22 §16 に書く。
- 前提・制約と、延期した design gap: 棚卸しの計数の m（⑤）、原価の小数（C）、POS の数量（P）は別 lane。延期が安全な理由: 棚卸しの書込みと商品別売上 CSV の確定は今の build で停止中で、原価は円の整数のまま上限 `999999` 円を本 lane が掛ける（C は桁を足すだけ）。
- 絶対保証の例外と escape hatch の自己点検: 「在庫の数量は浮動小数を通らない」= 入力は文字列から整数、表示は整数の演算（例外なし）。「既存の行の値と意味を変えない」= v8 は値を写すだけで、基準数量は今と同じ（`pcs` 1・`cm` 100）。「上限の外の値は保存されない」の例外は DB を直接書き換えた場合で、共通規則 (e) のとおり契約の外（溢れは既存の error）。「新しい単位が在庫少から漏れない」= BIZ が `ProductStockUnit::ALL` を wildcard の無い `stock_unit_kind` で 2 つに分けて渡し、`ALL` の抜けと 12 個の code の在庫少を test で止める（例外: CHECK を外して DB に直接入れた一覧に無い単位の行はどちらの並びにも入らず在庫少に出ない。共通規則 (e) の契約の外）。「金額はどの画面でも同じ」の例外は、画面が TS の twin で求める 2 つ（廃棄の保存の後の結果の欄のロス原価合計〈画面の入力値から求める: `DisposalPage.tsx:195`・`:321`〉、手動販売の金額の初期値）で、golden の表で BIZ と揃える。記録詳細は BIZ の値。「v8 が失敗したら何も変わらない」は TX の中の失敗に限る: TX の中の失敗（手順 2〜6 の失敗と、COMMIT の失敗で transaction を戻せた場合）は表・値・版を変えず、`foreign_keys` を元の値に戻す。例外は 2 つ: COMMIT が成功した後の `foreign_keys` の復元・再読取の失敗（22 §3.2 MNT-03-D1。`src-tauri/src/db/migration_tx.rs:136`〜`:172`）は `MigrationFailed`（接続破棄必須）を返すが、版 8 は記録済みで DB は新しい版として完全である。接続を破棄して再起動すれば開け（起動の `configure_database` が `foreign_keys = ON` を設定し直す: `src-tauri/src/db/mod.rs:197`）、旧版のアプリでは開けない。ROLLBACK も失敗したときは MNT-03-D1 のとおり transaction の状態が不明で、`foreign_keys` を戻さず接続を破棄する。
- 判定（ready / not ready）と理由: ready。実装に要る契約は上の正本にあり、owner の判断事項は残っていない。同梱の SQLite での `products` の作り直しは実測済み（下の Contract Probe P3）。

## Registration / Generation Obligations

| 変更対象 | 登録・生成義務（本 lane） |
|---|---|
| Tauri command の DTO（command の新設は無い） | `cd src-tauri && cargo run --bin generate_bindings` で `src/lib/bindings.ts` を再生成（`ProductStockUnit` の 12 値、記録詳細 6 型の `stock_unit`、`line_cost_centi`・`line_loss_cost_centi`、`DailySaleItem`・`IntegrityMismatch`・`StockAdjustment`・`ProductResponse`・`MonthlySaleItem` の `stock_unit`、`DeptSubtotal`・`GrandTotal`・`MonthlySaleItem` の `count_points`・`length_cm`） |
| 設計と code の検査（function-design doc の対応） | `src-tauri/tests/design_compliance_test.rs` の `SKIP_DOCS`（`:25`〜`:26`）から `10-common-rules.md` を外し、`build_doc_to_modules_map()`（`:108`）に `"10-common-rules.md" => ["biz::unit_amount"]` を足す。`unit_amount` の `pub fn` は 10 の code block のシグネチャと一致させる（未文書化は `:726`〜`:738` で fail。まだ作らない `stock_quantity_from_pos` は未実装の INFO）。migration の関数は `pub(crate)`（`db::schema_v8` を map に足さない。v7 と同じ: `src-tauri/src/db/schema_v7.rs:13`）。既存の module に `pub fn` を足すなら、その設計書の code block にシグネチャを書く。biz・mnt からの `crate::db`・`crate::io` の直接 re-export の登録面は D-083 の allow list（`src-tauri/tests/architecture_test.rs:260` の `DB_IO_REEXPORT_ALLOWLIST`）で、S4 の `ProductStockUnit` の re-export と同じ commit で `("biz/mod.rs", "ProductStockUnit", "crate::db::product_repo::ProductStockUnit")` の 1 エントリを足す（30 → 31 エントリ。本 lane が足す re-export はこの 1 つだけ） |
| migration | `src-tauri/src/db/migration.rs` の `migrations()` に v8 を登録（`:71`〜`:73` の後）、`src-tauri/src/db/mod.rs` に module。22 §16 に登録順と description を書く |
| source doc 新設 | `docs/migration-recovery-runbook.md` を `docs/PROJECT_HANDOFF.md` の参照先に 1 行 |
| REQ / coverage | test の名前に `_reqNNN` を足す・変えるので `cd src-tauri && cargo run --bin generate_traceability` で `docs/function-design/90-traceability.md` を再生成（T4 の baseline を含む。手で編集しない） |
| route・operator 画面の新設 | 該当なし（画面と route は増えない） |

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | Which concepts belong to replaceable external adapters, and which concepts are stable app-core contracts? | Architecture, function design, decision-log, Plan Packet | 本 lane は core（数量の種類・基準数量・上限・丸め）だけを実装する。レジの数量の書式（adapter、IO-02・IO-08）は触らない | P の lane |
| Fact check / design decision split | Which claims are observed facts from hardware/tool/files, and which are app decisions that need source-doc promotion? | Investigation doc, source design docs, decision-log | 店の事実は設計 lane で台帳から引いて正本にある。本 lane の新しい事実は code の現物（行番号）と Contract Probe の結果だけ | Contract Probe |
| Lifecycle / retry | What happens before, during, after, and after failure for import/export, duplicate input, rollback, retry, cancellation, and re-run? | Function design, DB design, UI design, Test Matrix | v8 は 1 つの TX で、TX の中の失敗（手順 2〜6 の失敗と、COMMIT の失敗で transaction を戻せた場合）は表・値・版を変えず、`foreign_keys` を元の値に戻す。COMMIT が成功した後の `foreign_keys` の復元・再読取の失敗（22 §3.2 MNT-03-D1。`src-tauri/src/db/migration_tx.rs:136`〜`:172`）は `MigrationFailed`（接続破棄必須）を返すが、版 8 は記録済みで DB は新しい版として完全である。接続を破棄して再起動すれば開け（起動の `configure_database` が `foreign_keys = ON` を設定し直す: `src-tauri/src/db/mod.rs:197`）、旧版のアプリでは開けない。再実行で重複適用しない。上限の検査で止まった DB は、v8 がその起動の最初の migration なので版も変わらず旧版でそのまま開ける。成功の後に旧版へ戻すのは 22 §16 の回復の手順だけ。v8 のアプリで v7 の時点の backup を復元すると、復元の中で差し替えた file を開くとき（`src-tauri/src/mnt/restore.rs:491`〜`:492` の `open_existing_database` → `migrate`）に v8 が当たる。上限の検査で止まれば、今の「復元した DB を開けなければ退避した今の DB へ巻き戻す」経路（`:495`〜`:501` → `rollback_after_failure` `:324`〜`:354`。巻き戻せれば `RestoreError::Recovered`）に入る。実読で確かめた経路で、v8 での実行は**未実測**（v8 の本体が無い。復元の巻き戻しそのものは今の test `restore.rs:1275`〜`:1278` 付近が持つ）。入出庫の冪等の fingerprint は数量が cm の整数のままなので変わらない | Matrix の State Lifecycle |
| Operator workflow | What does the operator do in the real sequence across app, external tool, media, print/export, backup, and recovery? | Screen/UI design, function design, Plan Packet manual checks | 上の Ordinary Operation。更新の前に旧版で backup を作る（回復の手順 1）ことを手順書に書く | L3 の表、手順書 |
| Replacement path | If the external system changes, which files/modules/docs are replaced and which app-core contracts remain stable? | Architecture, function design, decision-log | レジが替わっても本 lane の code は変わらない（D-113 の Revisit は P の側） | — |
| Data safety / evidence | How can the claim be supported by anonymized shape/count/hash/procedure evidence without committing real store data? | Plan Packet Data Safety, investigation doc, review evidence | test と L3 の商品・CSV は合成。例の金額は説明用 | Data Safety |
| Reporting / accounting semantics | Are totals, summaries, item records, returns, corrections, and inventory movements modeled separately enough to avoid false business meaning? | DB design, function design, report design, Test Matrix | 数量の集計を点と m の 2 本に分け、長さを点に足さない。原価 × 数量は 1 つの関数で同じ丸め。売上の金額は POS・入力の値のままで作り直さない。公式日報（D-104）は変えない | Matrix |
| Manual verification | Which assertions cannot be proven by automated tests and require Windows native L3, external tool import, or real-device confirmation? | Plan Packet, Test Matrix, PR body | m の入力と表示の読みやすさ、廃棄の丸めの受入れ、商品 CSV の CP932 の取込み。回復の手順は fault-injection に当たるので owner の L3 にせず、Writer が合成の DB で 1 度通して PR に残す（L3 Eligibility） | L3 の表 |
| 環境・再現性 | 新設の環境依存（toolchain / CI runner / OS 差異等）を repo-pinned config で強制するか、明示的に defer するか | repo-pinned config, Plan Packet | 新しい環境依存は無い。同梱の SQLite は `libsqlite3-sys` 0.28.0 の 3.45.0（Contract Probe）で、依存は変えない | — |

## Boundary / Wire Contract

- producer: BIZ-01（商品・商品 CSV の preview）、BIZ-02（入出庫の create と §12.6a の記録詳細）、BIZ-05（売上の数量と単位、CSV 出力）、整合性チェック、PLU 書出しの CMD の DTO、repo（`stock_unit` の読取り）。
- consumer: 商品フォーム・商品一覧・在庫照会・入庫・返品・手動販売・廃棄・業務記録詳細 6 画面・在庫変動・日次売上・月次売上・ホーム・整合性チェック・操作ログ・PLU 書出し。
- wire type: `ProductStockUnit` = 12 個の string literal。数量は今どおり整数（長さは cm）。`count_points`・`length_cm` は整数、`MonthlySaleItem.stock_unit` は `ProductStockUnit | null`。`line_cost_centi`・`line_loss_cost_centi` は 1/100 円の整数の十進の文字列（`^[0-9]+$`）。`total_cost`・`total_loss_cost` と原価・売価は円の整数（今の型）。`integrity_fix` の detail_json の各 adjustment と `integrity_check` の detail_json の各 mismatch に `stock_unit`。売上の CSV の `数量` は、商品の行が表示の単位の数（長さは m の数 `1.3`）、月次の部門別の行が `5 点・1.8 m` の文字列（`length_cm` が 0 の部門は今と同じ整数）。商品 CSV の `在庫単位` は code か表示の語、`m` の行の `初期在庫` は m の数（小数 2 桁まで）。
- internal type: `StockUnitKind`、金額の中間は i128、`ImportRow.stock_unit` は `Option<String>`（正規化した code）。
- precision/range: 上の「横断の約束」の表。長さの最小は 1 cm、m の入力は小数 2 桁まで。
- round-trip path: 画面の文字列（m）→ TS の純関数で cm の整数 → wire → BIZ の検証 → DB → repo → wire → TS の formatter（整数の演算）→ 画面。表示した文字列を保存し直す経路は作らない。
- invalid input: m の小数 3 桁以上・指数・符号・空、0 を受けない欄の 0、上限超えは画面の入力の error。UI を通らない command の上限超えは `ValidationFailed` で何も書かない。一覧に無い単位は、wire では deserialize の error、DB では読取りの error、商品 CSV では preview の行の error。
- compatibility: 既存の `pcs`・`cm` の値は変えない。v8 を適用した DB は旧版のアプリでは開けない（MNT-03-D11）。`quantity`（売上の集計）と `line_cost`・`line_loss_cost` は名前を残さず、古い読み手を TS の型検査で止める。`stock_unit` の key が無い記録済みの操作ログは今どおり整数で出す。商品 CSV の列の並びと必須の列は変えない。

## Test Plan

Test Design Matrix: [2026-10-11-unit-runtime](test-matrices/2026-10-11-unit-runtime.md)。

- targeted tests: `cd src-tauri && cargo test --test architecture_test`（db の層が biz を呼ばない。D-083 の allow list が `ProductStockUnit` を足した 31 エントリと両方向で一致する）・`cargo test --lib unit_amount`・`db::schema_v8`・`db::migration`・`biz::product_service`・`biz::inventory_service`・`biz::sales_service`・`biz::integrity_service`・`finite_enum_contract_tests`、`npm test -- <変えた feature の test>`。
- negative tests: Matrix の Negative Paths と「横断の約束」の行（上限の 1 つ外、一覧に無い単位、単位の変更、m の不正な文字列）。
- compatibility checks: v8 の前後で全列が同じ、棚卸しの評価額の今の test が同じ期待、個数の商品だけの日の売上は、集計の値（`count_points`）が今の `quantity` と同じで、販売点数のカード（`N 点`。日次・ホーム・月間販売点数）は 1,000 点未満なら今と同じ表示で、1,000 点以上は 3 桁区切りが付く（`1234 点` → `1,234 点`。X9）。日次の表は正本（56 §56.10）どおりに変わる: 明細の数量 `3` → `3 個`、部門小計・合計 `5` → `5 点`。月次の商品別（`N 点`）は今と同じ。
- data safety checks: 差分と fixture に実データ（JAN・商品名・金額・店主の発言の原文）を入れない。
- main wiring/integration checks: `migrations()` に v8 が登録され起動時の `migrate` で当たる、`biz::unit_amount` が記録詳細・create・売上・在庫警告から呼ばれる、bindings の型が画面まで届く（`npm run typecheck`）。
- Human Gate に manual を含むので、Writer は L3 の前に `cd src-tauri && cargo check --release` を回す。回復の手順（22 §16）は Writer が合成の DB で 1 度通し、結果を PR に残す。

L3（owner が Windows の実機で確かめる。fixture は Coordinator が Ready の依頼と同時に渡す）:

| # | 到達経路 | 入力物 | 期待（PASS の基準） | fixture | 依存する既知 backlog |
|---|---|---|---|---|---|
| L3-1 | 商品登録 → 入庫 → 在庫照会 → 入庫記録詳細 → 商品修正 | 長さの商品を画面で登録（単位 `m`、売価 700・原価 400、初期在庫 `25`）。入庫 `2.5`・原価 333。保存の後の原価差分のダイアログは「見送って閉じる」 | 売価・原価の label に「1 m あたり」（`売価（必須、1 m あたり）`）。在庫照会 `27.5 m`。入庫記録詳細の行が数量 `2.5 m`・原価 `¥333`・行の金額 `832.50 円`、原価合計 `¥833`、関連する在庫変動 `+2.5 m`・`27.5 m`。商品一覧からこの商品の修正を開くと、現在庫が `27.5 m` | 画面の入力だけ（空の DB で可） | なし |
| L3-2 | 手動販売 → 日次売上 → 月次売上 → 在庫照会 | L3-1 の商品を `1.3`（金額の初期値 `910` のまま保存） | 日次売上の行が `1.3 m`・`¥700/m`・`¥910`、販売点数がその日この行だけなら `1.3 m`（個数の商品の売上が 5 点あれば `5 点・1.3 m`）、在庫照会 `26.2 m`。月次売上（その月）を開き、商品別の行が `1.3 m`、部門別のその部門の数量が `1.3 m`（個数の商品の売上が 5 点あれば `5 点・1.3 m`） | 同上 | なし |
| L3-3 | 廃棄 → 廃棄記録詳細 → 在庫照会 → 返品・交換 → 返品・交換の記録詳細 → 在庫照会 | 廃棄: L3-1 の商品を `0.5`・原価 333。返品・交換（廃棄と在庫照会の後）: 種別は「返品」のまま、レジ戻し状況を「レジ未処理」にし、L3-1 の商品を追加して数量を `0.3` にして保存（画像は付けない） | 廃棄: 保存の後の結果の欄の「ロス原価合計」が `¥167`（合計は入力の途中には出ない）。廃棄記録詳細の行が数量 `0.5 m`・原価 `¥333`・行の金額 `166.50 円`、合計 `¥167`。在庫照会 `25.7 m`。owner がこの丸めを受け入れるかを PASS / FAIL で答える。返品・交換: 数量の欄が `0.3` を受けて保存でき、保存の結果の欄の「詳細を見る」で開いた記録詳細の行が方向 `戻り（在庫+）`・数量 `0.3 m`、関連する在庫変動 `+0.3 m`・`26 m`。在庫照会 `26 m` | 同上 | 廃棄の丸めは TD-023 に合わせた案（店の直接の答えは無い） |
| L3-4 | 商品 CSV の取込み → 在庫照会 | `在庫単位` に `玉`・`m`・`kg` を書いた 3 行の CSV（`m` の行の `初期在庫` は `25`、`玉` の行は `3`、`kg` の行は `1`） | preview で `kg` の行がエラー行、ほかの 2 行を取り込める。在庫照会で `m` の商品が `25 m`、毛糸の商品の在庫が `3 玉` | 合成の CP932 の CSV（実商品名を使わない。列は今の商品 CSV の必須の列 + `在庫単位`・`初期在庫`。部門は初期データの部門 ID） | なし |

## Review Focus

- Ordinary Operation の 7 行が正常な条件で成り立つか。危険な結果（在庫の値の変化、既存の `cm` の意味の変化、在庫少の漏れ、上限の外の保存）を出さないか。
- 「横断の約束」の表の各行が、保証する所の code と Matrix の test に 1 対 1 で当たるか。在庫を書く所・数量を検証する所に、表に無い経路が無いか（`rg -n 'update_stock_quantity\(|insert_.*item|quantity <= 0' src-tauri/src --glob '!*test*'`）。
- v8 の手順と上限の検査が 22 §16 のとおりか。失敗の各点で表・値・版・`foreign_keys` が変わらないか。
- 既存 test の期待の書換え（Matrix の表）が契約の変更だけで、弱めていないか。形が変わる wire の型・field・関数の戻り値を読む所（Rust・TS・test・fixture）に、Scope と書換えの表に無い所が無いか。
- 棚卸しの分を ⑤ へ送った判断で、U の後・⑤ の前の build に危険な状態が残らないか（停止中の画面に m の商品が出る経路の有無）。
- S7 の後に、実装済みの契約へ proposed の注記が残っていないか、C・P・⑤ の注記を外していないか。

## Contract Ledger

隣接する契約の sweep: 共通規則の節（SPEC-UNIT-D1〜D11 と続き）、22 §16、30・31・34・36・44・51・56・57・58・61〜64・66・67・74・75 の D-113 の注記の行を当て、本 lane が触り得る契約を足した。除外は表の末尾。

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| SPEC-UNIT-D1 | 10 共通規則、master-tables、DB_DESIGN | S2: enum 12・v8 の CHECK。S1: `stock_unit_kind`・`price_basis_quantity` | Matrix の SPEC-UNIT-D1 の 2 行 | L3-1（単位 `m` の登録）・L3-4（`玉`・`m` の取込みと `kg` の行の error）。ほかの単位は自動 test のみ |
| SPEC-UNIT-D2 | 10 共通規則、30 §4.9 | S2: `ProductUpdates.stock_unit` を消す、CSV の上書きの検査 | Matrix の SPEC-UNIT-D2 の行 | 非対象 |
| SPEC-UNIT-D3（入力・加算・表示・表示する所の表の 1〜6・8〜12・集計・単価） | 10 共通規則、34・36・51・53・56・57・58・61〜64・66・67・74・75、SCREEN_DESIGN | S4〜S6（表の 4 の月次の部門別の数量の列、表の 12 の操作ログの差分と 36 の `integrity_check` の log の key、S7 で表に足す商品修正の画面の現在庫を含む） | Matrix の SPEC-UNIT-D3 の行 | L3-1〜L3-4（m の入力と、表の 1〜4・6 のうち到達経路が通る画面）。表の 5・8〜12 と、表の 1〜4・6 のうち到達経路が通らない画面は自動 test のみ（理由は Matrix の Residual Test Gaps）。表の 7（棚卸し）は ⑤ |
| UI-02-D7・UI-03-D12・UI-05-D8（数量の入力）、UI-06c-D5、UI-13-D3、UI-11c-D14 | 61・63・64・66・75・74 | S6 | Matrix の入力・加算・表示の行 | UI-02-D7 は L3-1、UI-05-D8 と UI-03-D12 は L3-3。UI-06c-D5 は、共有の `MovementTable` を L3-1 の入庫記録詳細と L3-3 の返品・交換の記録詳細の「関連する在庫変動」で見て、商品別の在庫変動の画面の経路は RTL のみ。UI-13-D3・UI-11c-D14 は RTL のみ（理由は Matrix の Residual Test Gaps） |
| SPEC-UNIT-D4 | 10 共通規則、25 | 変えない | 既存の PLU の formatter の test | 非対象 |
| SPEC-UNIT-D6 | 10 共通規則、21 §10.2・§10.5、31 §12.6a、35 §20.5a | S1・S4・S5 | Matrix の SPEC-UNIT-D6 の行 | L3-1・L3-3 |
| SPEC-UNIT-D7・UI-04-D18 | 10 共通規則、62 | S1・S5・S6 | Matrix の SPEC-UNIT-D7 の行 | L3-2 |
| SPEC-UNIT-D9 | 10 共通規則、58 §58.6 | S1・S5 | Matrix の golden の行 | 非対象 |
| SPEC-UNIT-D10・BIZ-01-D8（単位・CSV） | 10 共通規則、30、44、DB_DESIGN D-4 | S2 | Matrix の SPEC-UNIT-D10 の行 | L3-4（商品 CSV の `在庫単位`・`初期在庫`）。在庫少の種類の分岐は自動 test のみ |
| SPEC-UNIT-D11（wire の型） | 10 共通規則、20・34・36・40・44 | S4・S6（`StocktakeItemDetail`・`AdjustedItem` は ⑤） | Matrix の型検査の行 | 非対象 |
| SPEC-UNIT-D11 の続き（入力の上限と安全な整数の範囲） | 10 共通規則 (a)〜(e)、30・31・34・36・40 | S1・S3・S5。「横断の約束」X1〜X12 | Matrix の X の行 | 非対象。棚卸しの実数と確定は ⑤、POS の換算の後は P、原価の 1/100 円は C |
| MNT-03-D13（vU と上限の検査、回復） | 22 §16 | S2・S7（手順書、回復の冒頭の文）。MNT-03-D1（COMMIT・ROLLBACK・`foreign_keys` の復元の失敗）は共有の helper で v8 にも当てる | Matrix の MNT-03-D13 の行（X13） | R4 の承認。回復の手順は Writer が合成の DB で通す。vC は C |
| UI-01b-D22（`ej_unverified` の分岐を除く。その分岐は Z004・EJ の lane〈㉘ ④〉） | 51 | S6 | Matrix の UI-01b-D22 の行 | L3-1 |
| 隣接の除外: SPEC-UNIT-D5・SPEC-PRV-D4（案の式と 1/100 円） | 10、77 | 変えない（C）。77 の画面の新売価・新原価の上限だけ本 lane（X5・X6、S6） | Matrix の X5・X6 の行 | 非対象 |
| 隣接の除外: SPEC-UNIT-D8 | 10、23、29 | 変えない（P） | — | 非対象 |
| 隣接の除外: SPEC-STK-VAL-D2〜D6 | 35 §20.5a | 値を変えない（関数を移すだけ） | 今の `stocktake_service.rs:1407`〜`:1487` の test を移して同じ期待で回す | 非対象 |
| 隣接の除外: SPEC-STOP-D1〜D3、SPEC-STK-TIME | 35 §20.0、tracking | 変えない（停止と新方式は ⑤） | 既存 | 非対象 |
| 隣接の除外: D-104（日報の個数） | 29 IO-07-D2、pos-tables | 変えない | 既存の日報の test | 非対象 |
| 隣接の除外: UI-11a（在庫少の基準の画面） | 69 | 変えない | — | 非対象 |
| 隣接の除外: UI-01b-D5（単位は登録後に変えない） | 51 | 変えない | 既存 | 非対象 |
| 隣接の除外: MNT-03-D11（新しい版の DB を旧版が開かない） | 22 §12.4 | 変えない（v8 で app_max が 8 になるだけ） | 既存 | 非対象 |

## Contract Probe

- P1 同梱の SQLite の版: **実測（2026-10-11）**。`grep -m1 -n 'define SQLITE_VERSION ' ~/.cargo/registry/src/*/libsqlite3-sys-0.28.0/sqlite3/sqlite3.h` → `149:#define SQLITE_VERSION        "3.45.0"`。`src-tauri/Cargo.toml:32` は `rusqlite = { version = "0.31", features = ["bundled"] }`、`src-tauri/Cargo.lock` は `rusqlite` 0.31.0・`libsqlite3-sys` 0.28.0。
- P2 同梱の SQLite で、子の表に参照される親の表を `foreign_keys` OFF → 作り直し → DROP → RENAME → `foreign_key_check` → 復元の手順が通るか: **実測（2026-10-11、既存の test）**。`cd src-tauri && cargo test --lib db::schema_v2` → `test result: ok. 14 passed; 0 failed`（`receiving_records`・`return_records`・`manual_sales` の作り直し: `src-tauri/src/db/schema_v2.rs:139`〜`:198`。`test_v2_req903_backfill_with_parent_child_fk`・`foreign_keys_restored_on_success`・`foreign_keys_restored_on_failure` を含む）。
- P3 22 §16 の vU の手順が同梱の SQLite で `products`（8 つの表が FK で参照する）に通るか: **実測（2026-10-11、Coordinator が発注した使い捨ての test。commit していない。証跡は repo 外の `.local/reports/contract-probe/unit-runtime-p3-2026-10-11.log`）**。`setup_test_db()` の v7 の DB（`schema_versions` の最大 7）、`SELECT sqlite_version()` = `3.45.0`、`cargo test --lib probe_v8_products_rebuild` が 1 passed。22 §16 の手順 1・3〜7（版の記録を除く）のとおりに作り直した結果: (1) 3 行 × 18 列が前後で同じ（`pragma_table_info` と `products` 自身の FK も同じ） (2) `foreign_key_check` が TX の中と COMMIT の後で 0 行 (3) `products` を参照する表は前後とも 8 (4) `products_new` の名前の残り 0 (5) 名前付きの index 3 つと自動の index 1 つが前後で同じ (6) `kg` の INSERT・UPDATE は CHECK で拒否、12 個の code は全部入る (7) `foreign_keys` を戻した後、子のある商品の DELETE は FK で拒否。作り直しの後、`sqlite_master` の `products` の定義は表名が引用符つきになる（動作は同じ）。手順の文を SQL に落とすときに解釈が要った 3 点（TX の始め方、手順 6 (c) の件数を読む時点、列の順）は S2 の migration v8 の行に 1 通りに決めて書いた。実測していないもの: 上限の検査（手順 2）と版の記録を含む v8 の本体（実装の test で確かめる）。
- P4 `products` に trigger・view が無いこと（作り直しで失うものが無い）: **実測（2026-10-11）**。`rg -n 'CREATE TRIGGER|CREATE VIEW' src-tauri/src/db/` → `migration.rs:1224` の 1 行（v7 の test の中で作る trigger）だけ。`products` の index は 3 つ（`rg -n 'ON products' src-tauri/src/db/schema_*.rs` → `schema_v1.rs:251`〜`:253`）、`products` に列を足す migration は v3 だけ（`rg -n 'ALTER TABLE products' src-tauri/src/db/schema_v*.rs` → `schema_v3.rs:16`）。未配線の時点証拠の migration は 2 列を足す（`schema_time_evidence.rs:44`・`:48`）が、v8 より後に入るので v8 は写さない（22 §16 の手順 3「後に入る lane の義務」）。
- P5 `design_compliance_test` が `SKIP_DOCS` の設計書の `pub fn` を未文書化として落とすこと: **実読（2026-10-11）**。`src-tauri/tests/design_compliance_test.rs:25`〜`:26`（`SKIP_DOCS` に `10-common-rules.md`）・`:549`（skip の判定）・`:726`〜`:738`（未文書化で panic）。是正を仮に当てた end-to-end の実行は**未実測**（`biz::unit_amount` がまだ無い）。S1 の最初に、module と対応表を足した状態で `cd src-tauri && cargo test --test design_compliance_test` が PASS することを確かめる。
- P6 JS の number で正確に運べる最大の整数は `9007199254740991`: 言語の定数で probe は要らない。上限どうしの演算の値（`999999 × 9999999 = 9999989000001`、`(999999 × 100) × 9999999 = 999998900000100`、900 行 `8999990100000900`、901 行 `9009990089000901`）は 2026-10-11 に python の整数で計算し直して共通規則 (b)(c) と一致した。
- 観測済みで probe の要らない前提: 店の 12 種・m・丸め・上限（台帳の番号。設計 lane で正本化）。棚卸しと商品別売上 CSV の確定の停止（`stocktake_service.rs:257`〜`:290`、`csv_import_service/commit.rs:18`〜`:31`）。

## Data Safety

- tracked に書かない: 実 JAN・商品名・金額・件数の細部、店主の発言の原文、持ち帰りデータの中身、DB・backup・log の file。店の事実は台帳の番号と要旨だけ。
- local-only: repo 外の回答台帳（`.local/reports/store-premises/`）、L3 の fixture の実体（Coordinator が Ready の依頼と同時に渡す合成の CSV）、回復の手順を通す合成の DB と backup。
- synthetic-only: test の商品・明細・CSV は合成（商品コードは `TEST-`・`LS-` 等の接頭辞、名前は「テスト商品」等）。L3 の CSV は合成の CP932。
- rollback / recovery（R4）: v8 は 1 つの TX で、TX の中の失敗（手順 2〜6 の失敗と、COMMIT の失敗で transaction を戻せた場合）は表・値・版を変えず、`foreign_keys` を元の値に戻す。COMMIT が成功した後の `foreign_keys` の復元・再読取の失敗（22 §3.2 MNT-03-D1。`src-tauri/src/db/migration_tx.rs:136`〜`:172`）は `MigrationFailed`（接続破棄必須）を返すが、版 8 は記録済みで DB は新しい版として完全である。接続を破棄して再起動すれば開け（起動の `configure_database` が `foreign_keys = ON` を設定し直す: `src-tauri/src/db/mod.rs:197`）、旧版のアプリでは開けない（この場合に旧版へ戻すなら、成功した場合と同じ下の手順）。v8 が成功した DB は旧版のアプリで開けず、旧版へ戻すのは 22 §16 の回復の手順（アプリを止め、復元の中断の遺物が無いことと backup の版・`integrity_check` を確かめてから、今の DB・WAL・SHM を別の folder へ移し、更新の前の backup を置く）だけ。更新の後に入れたデータは戻らず、保全した file にだけ残る。逆変換の migration は作らない。code の rollback は PR の revert（v8 を当てた開発用の DB は上の手順で戻す）。本番の DB は無い（`docs/project-memory.md`）ので、初回の導入は v8 を含む版から始まる。

## Implementation Results

- 結果: 単位を 12 種へ広げ（migration v8、Rust の enum と bindings の `ProductStockUnit`、在庫少が全単位を分ける）、長さの商品を在庫は cm の整数のまま m で入力・表示し、単価を 1 m あたりで出すようにした（共通規則 SPEC-UNIT-D1〜D3）。入庫の原価小計・廃棄のロス原価・手動販売の金額の初期値を BIZ の共通の丸め（SPEC-UNIT-D6・D7）で求め、長さの商品で金額が 100 倍になる既知の不整合 (1)(2)(4)(5) を解いた。数量・在庫・売価・原価（円）と入庫・廃棄の円の合計に入力の上限と保存の前の検査を掛け、v8 が移行の前の DB の値も同じ範囲かを確かめる。集計は数量の種類ごとに分け（点と m、点の 3 桁区切りは GA1-3）、商品 CSV の `在庫単位` は 12 個の単位を受け、不正な単位と単位の変更を行の error にする（SPEC-UNIT-D10、BIZ-01-D8）。設計正本の単位の分は proposed の注記を外して現行の記述にし（S7）、旧版へ戻す管理者の手順を `docs/migration-recovery-runbook.md` に置いた。原価の 1/100 円（C）・Z004 と EJ の数量（P）・棚卸しの計数の画面と確定（⑤）は含めない。
- 既存の保護: 既存の `pcs`・`cm` の商品の値と意味は変えていない（v8 の前後で `products` の全列の値と定義が同じ）。既存 test の期待を変えたのは Matrix の「既存 test の書換え」の表の行だけ。棚卸しは停止のまま（m の棚卸しは ⑤ の後）。
- 未検証: 回復の手順（22 §16）は合成の DB で 1 度通したが、「旧版のアプリ」は同じ `init_database` を通る開発用の bin で代え、Windows の実機での installer の入替えは通していない。S7 は packet の表が挙げる節・行だけを直したので、同じ file の表の外に 2 単位だった頃の例が残る所がある（`docs/backlog.md` の follow-up）。
- review・L3・CI・merge（closeout、2026-10-15）: Plan Review は round 3（上限）で 2 本とも approve、実装の途中に Gated Amendment 1。Final Review は broad 2 本（Codex GPT-6 Astra と Claude 側 Fable 5.1）がどちらも approve（P1 / P2 0）。base の同期（docs だけの衝突の解消）は delta ack で閉じた。L3 は owner が Windows の実機で L3-1〜L3-4 を PASS とし、廃棄の丸めを受け入れた。helper の record は review = pass、manual = pass、r4 = pass（owner の R4 の承認 2026-10-15）。Ready の後の hosted run の `Merge gate` は success で、owner の承認の後に helper 経由の squash merge。検査の結果は PR と GitHub が持つ。PR: [#158](https://github.com/higekiri-w/inventory-system-desktop/pull/158)
- 後続: 原価の lane（C）・Z004 と EJ の lane（P）・棚卸しの再開（⑤）への申し送りは上の「C・P・⑤ へ送るもの」と `docs/backlog.md` の「単位の拡張」の項目。Final Review の P3 と未確認事項、作りの重複の整理は同じ項目の「単位の runtime の lane の follow-up」。

## Review Response

### Plan Review round 1（2026-10-11、対象は plan-gate へ進めた版）

- Claude 側（fresh Opus 5.5、fork でない）: reject、P1 0 / P2 4 / P3 4。Codex（GPT-6 Astra / high）: reject、P1 0 / P2 7 / P3 1。Ordinary Operation は 2 本とも「成立」。1 lane のままという手法、Risk・Human Gate・Final Review Minimum は 2 本とも妥当と判定した。
- 裁定（Coordinator）: 2 本の指摘の重なりをまとめると、下の表の A〜I（P2）と P3 の行になる。全ての行を現物で裏取りし、採用した。

| ID | 出どころ | 要旨 | 採否 |
|---|---|---|---|
| A | Claude 側・Codex | 最新の版を 7 と固定する既存 test と、月次の集計の test が書換えの表に無い。作れない test の前提がある | 採用 |
| B | Claude 側・Codex | 手動販売の `cm` の商品の数量の初期値と再追加が、正本 UI-04-D18 と Matrix で逆 | 採用 |
| C | Claude 側 | 在庫少の SQL の並びを repo の中で作ると db → biz の依存になる | 採用（BIZ が並びを渡す形） |
| D | Claude 側 | 初期在庫が負を通す。正本は 0 以上 | 採用 |
| E | Codex | 月次売上の部門別の数量の表示が Scope に無い | 採用（数量の列を足し、並べ替えの対象にする） |
| F | Codex | 商品フォームの在庫連動の提案が、現物に無い準備照会に依る | 採用（準備照会に依る分岐は ㉘ ④ へ送る） |
| G | Codex | 「v8 が失敗したら何も変わらない」が、COMMIT の後の `foreign_keys` の復元の失敗と合わない | 採用（TX の中の失敗と分けて書く） |
| H | Codex | 一括価格改定の画面に上限の検証が無い | 採用（Claude 側は「BIZ の error に任せ非対象と明記」を提案したが、共通規則の「画面も同じ定数で先に止める」に合わせた） |
| I | Codex | `integrity_check` の detail_json の `stock_unit` が Scope に無い | 採用 |
| P3 | Claude 側 4・Codex 1 | 記録詳細の型の置き場、AC8 の式で見えない行、画面の上限超えの error の文、L3 の追加、57 の切れた anchor | 全件採用 |

- 相談役（Fable 5.1）に、是正を書く前の反例探しを頼んだ。方針のうち E（57 が「数量の列なし」を固定している行も直す範囲に入れる）・F（送り先は ⑤ ではなく ㉘ ④）・G（22 §16 の回復の冒頭の文を S7 で直す）に直しが入った。相談役の案のうち採らなかったもの: `migrate` を割る変更（読み替えで足り、MNT-03-D11 の機構は既存の test が持つ）、`FailurePoint` への注入点の追加（親の無い子の行で手順 6 の (b) を落とせる）、在庫少の `CASE … ELSE` の形（正本の SQL の形のまま `IN` を選んだ）。
- 指摘は「変わる型・値を読む所の漏れ」と「正本の値の写し漏れ・逆」の 2 つの型だったので、個別の是正に加えて、型ごとに全体を見直した。追加で直した主なもの: 月次の `quantity` を作る TS の test の列挙（12 → 20 file）、`prev_month_comparison`、PLU 書出しの fixture（3 → 5 file）、整合性の補正の「対象から外せば、ほかの商品は補正できる」、「`在庫単位` が行の error の行は `初期在庫` を読まない」、日次の明細の並べ替え、売上の CSV の部門の行の境界、手動販売の数量が error の間の金額欄。
- scope の追加・除外の照合（Review Rules）: E の追加は正本 57 の D-113 の注記（部門別は点と m の 2 本。owner の決定 TD-206・TD-207）、H の追加は共通規則「入力の上限と安全な整数の範囲」の (a)、F の除外は `docs/backlog.md` の「㉘ ③ 〜 ⑤ の申し送り」と時点証拠 ADR の「㉘ への引継ぎ」の割当てと照合した。
- Contract Probe の P3 は、同梱の SQLite（3.45.0）と v7 の DB で実測して 7 点とも通った（Contract Probe の節）。round 1 の 2 本も、それぞれ独立に同じ作り直しが通ることを確かめた。
- 是正は起草役の commit「単位の runtime の計画を、読む所の漏れと正本の値の写し漏れについて直す」。round 2 へ進む（round 天井 3）。

### Plan Review round 2（2026-10-11、対象は round 1 の是正と記録の後の版）

- Codex（GPT-6 Astra / high）: approve、P1 0 / P2 0 / P3 2。Claude 側（fresh Opus 5.5、fork でなく round 1 とも別の context）: reject、P1 0 / P2 3 / P3 6。Ordinary Operation は 2 本とも「成立」。round 1 の指摘は 2 本とも閉じた。
- 裁定（Coordinator）: Claude 側の P2 の 3 行を現物で裏取りし、P3 も含めて全て採用した。

| ID | 出どころ | 要旨 | 採否 |
|---|---|---|---|
| F1 | Claude 側 | 商品修正の画面の read-only の「現在庫」が、長さの商品でも cm の整数のまま出る | 採用（12 単位とも `formatStockDisplay` の形で出す） |
| F2 | Claude 側 | 期待が変わる既存 test が書換えの表に無い所がある（入庫記録詳細の行の金額、月次の並べ替えの定義、部門別の見出し） | 採用 |
| F3 | Claude 側 | 商品 CSV の commit の上書きの行で、売価・原価の上限の再検証が抜ける読み方ができる | 採用（売価・原価は新規と上書きの全行、初期在庫は新規の行） |
| P3 | Claude 側・Codex | 合計の表記、`m` の数量欄の属性、cmd が単位の型を引く経路、売上 CSV の長さの数の形、作り直しの前後の列の定義の比較、「今と同じ」の言い過ぎ、BIZ の checked の加算の test、Design Readiness に残った「未実測」 | 全て採用 |

- 相談役（Fable 5.1）に、同じ型の残りの洗い出し（数量を文字にする所、期待が変わる既存 test、UI を通らない書込みの入口）と是正方針の反例探しを頼んだ。方針への反例は無く、残りとして操作ログの補正の量の行、行の金額の formatter の不正な入力、使われていない在庫の書込みの field（`ProductUpdates.stock_quantity`。X3 に「到達経路なし」と書き、消すのは時点証拠の契約どおり ⑤）が挙がり、採用した。v7 の backup を v8 のアプリへ復元する経路は、起草役が現物を読んで「復元の中で v8 が当たり、止まれば今の巻き戻しの経路に入る」と確かめ、実行は `未実測` として Contract Probe と回復の手順書の Scope に置いた。
- 起草役は仕上げとして、「全部」「今と同じ」と書いた文と L3 の期待の文字列を現物で読み直し、言い過ぎと誤りを直した（`common.rs` の test の主張、`migration.rs` の既存 test の説明、「金額はどの画面でも同じ」の例外）。
- 是正は起草役の commit「単位の runtime の計画に、商品修正の画面の現在庫と、取込みの確定の上書きの行の検証を足す」。round 3（上限）へ進む。

### Plan Review round 3（上限、2026-10-12、対象は round 2 の是正と記録の後の版）

- Codex（GPT-6 Astra / high）: approve、P1 0 / P2 0 / P3 1。Claude 側（fresh Opus 5.5、fork でなく round 1・round 2 とも別の context）: approve、P1 0 / P2 0 / P3 3。Ordinary Operation は 2 本とも「成立」。
- 実測の分担: Codex は隔離 worktree で cargo の全 test・frontend の全検査・`cargo check --release`・bindings と traceability の生成の検査を回し、同梱の SQLite と v7 の DB で `products` の作り直しを独立に実測して通した。Claude 側は Coordinator の指示で cargo と npm を回さず（同じ時刻に Codex が同じ tree を build していて、host のメモリが逼迫した）、cargo が要る確認を claim として扱い、実読と `rg` の全数照合・数値の独立計算・python の SQLite での最小の作り直しで見た。
- 裁定（Coordinator）: P3 の 4 行を現物で裏取りし、全て採用した。どれも文を現物に合わせる直しで、Goal・Scope・AC・設計の値を変えない。

| ID | 出どころ | 要旨 | 採否 |
|---|---|---|---|
| R3-1 | Claude 側 | 月次の部門別の同額の並びを「部門 ID の昇順」と書くが、現物は `key`（部門 ID の文字列）の昇順（`src-tauri/src/biz/sales_service.rs:562`・`:574`） | 採用（本 lane は並びを変えず、文を現物に合わせる） |
| R3-2 | Claude 側 | X8 の成功の端の fixture が商品の数を書かず、1 つの商品に積むと在庫の上限（X3）が先に拒む | 採用（在庫 0 の商品 10 個に 100 行以下ずつ分ける） |
| R3-3 | Claude 側 | Contract Ledger が、L3 の到達手順の通らない画面（返品・交換、操作ログ、整合性チェック）を L3 に当てている | 採用（当て先を通る画面だけにし、通らない契約は RTL のみと理由。返品・交換は L3-3 に 1 手順足す） |
| R3-4 | Codex | 棚卸しの計数を ⑤ へ送る理由の「計数の画面に行が出る状態を今の build で作れない」は、進行中の棚卸しを持つ既存の DB では成り立たない | 採用（止まっているのは開始・保存・確定で、既存の一覧は出ると書く。店は初導入で本番の DB は無い） |

- R3-1 の関連: [34](../../function-design/34-biz-sales-service.md) の「部門IDの昇順でソート」は日次の部門小計の文で、現物（数値の昇順）と合う。月次の同額の並びは正本に書かれておらず、食い違いではない（起草役が実読、Coordinator が `sales_service.rs:562` を確認）。
- R3-3 で L3-3 に返品・交換の 1 手順（数量 `0.3` の返品の保存と記録詳細・在庫照会の確認）が増えた。L3 の round の数と Owner Effort Budget の上限は変えない。
- 是正は起草役の commit「単位の runtime の計画の文を現物に合わせる（同額の並び・円の合計の fixture・L3 の当て先・棚卸しの一覧）」。round 3 で 2 本とも P1・P2 が無いので、次の round は開始せず plan-approved へ進む。是正の後の文の再確認は Final Review（R4、独立 2 本）が行う。

### Gated Amendment 1（2026-10-12、実装の 1 段目の Writer が編集せずに止めた項目から）

- 経緯: 実装を 2 段に分けて発注した（1 段目 = S1〜S4 の Rust と bindings の再生成、2 段目 = S4 の残り・S5〜S7・全 gate）。1 段目の Writer は、S4 の「PLU 書出しの未書出しの一覧」（`ProductResponse` に `stock_unit`）を編集せずに止めて返した。packet の指示どおり `ProductStockUnit` を `src-tauri/src/biz/mod.rs` の `pub use` に足すと、`src-tauri/tests/architecture_test.rs` の `biz_mnt_direct_db_io_reexport_allowlist`（D-083 の allow list）が落ち、allow list に足す必要があるが、その file が Scope にも「既存 test の書換え」の表にも無かった（`docs/AGENT_OPERATING_MANUAL.md` §5.6「Writer が編集前に止まったとき」の 5 に当たり、発注の訂正では通せない）。1 段目のほかの分は commit 済みで、Coordinator が同じ終点で `cargo fmt --check`・`cargo clippy`・`cargo test` を回し直して通ることを確かめた。
- 相談役（Fable 5.1）に、書く前の反例探しと、同じ型の残り（Scope にも書換えの表にも無い機械検査の期待値）の洗い出しを頼んだ。allow list への追加が最小で、D-083 の Impact が定める経路であること（cmd の本体から `crate::db` を引く許された経路はほかに無い）を確かめ、同じ型の残りを 1 つ挙げた（下の GA1-2）。1 段目の Writer が報告した packet との食い違いの候補は、どれも packet に規定が無く Writer の裁量の範囲と判定した。
- 足したもの（起草役の commit「単位の runtime の packet と Matrix に allow list・呼出し件数の test・集計の点の桁区切りを足す」）:

| ID | 内容 | 置いた所 |
|---|---|---|
| GA1-1 | D-083 の allow list に `ProductStockUnit` の re-export を足す（re-export と同じ commit） | Scope S4 の PLU 書出しの行、Registration / Generation Obligations、Test Plan、Matrix の書換えの表 |
| GA1-2 | `src/lib/request-helpers.test.ts` が request module ごとに固定している `parseRequiredSafeInteger(` の件数は、数量の parse を S5 の新しい関数に替えると変わる。`parseRequiredSafeInteger` 自体の契約は変えない | Scope S5、Matrix の書換えの表 |
| GA1-3 | 集計の数量の表示（`N 点`・`N 点・L m`）の `N` は 3 桁区切りを付ける（正本と packet に規定が無く、今の画面は所によって違った。長さの部分の形と同じ向き）。見た目が変わるのは 1,000 点以上のときだけ（販売点数のカード 3 つと、売上の CSV の集計の行の文字列） | 横断の約束 X9、Scope S4・S5・S7、Test Plan の compatibility checks、Matrix の集計・売上の CSV・Boundary・Mutation・書換えの表 |
| GA1-4 | 原価の lane（C）への申し送り: v8 が履歴として持つ上限と runtime の定数の一致を止める test は、C が原価の上限を変えると落ちる | `docs/backlog.md` の単位の拡張の項目 |

- GA1-3 は Coordinator が決め、owner が賛成した（repo 外の回答台帳 TD-235）。
- Goal・AC・設計の値・Risk・Final Review Minimum・Human Gate は変えていない。broad の Final Review はまだ取っていないので、Gated Amendment 1 を含む版を Final Review（R4、独立 2 本）が見る。
- 実装: 2 段目の Writer が GA1-1〜GA1-3 を含めて行う。

先行 round の結果・評価（判定・件数・採否・reviewer の意見）はこの節にだけ書く。前半の節と遷移記録には「round N の是正（Review Response 参照）」だけを書く（独立 review は `## Review Response` より前だけを読む）。

Closeout（2026-10-15）: Final Review は broad 1 回で、Codex GPT-6 Astra と Claude 側 Fable 5.1 の独立 2 本（head `c36b0d00`）がどちらも approve（P1 / P2 0）。P3 と未確認事項は本 lane では直さず、`docs/backlog.md` の「単位の拡張」の項目へ送った（S7 の対象の行の外に残る古い型・SQL・表示例の同期、手動販売の金額〈円〉に上限が無いこと、手動販売の長さの商品の金額の求め直しの基準の違い〈未確認事項〉、over-engineering の読みで出た作りの重複の整理〈振舞いを変えない〉）。2026-10-15 に #159・#161 の merge で `docs/backlog.md` の 1 行が衝突し、`origin/main` の単段 merge で両側を残して解消した（head `ce116f51`、差は docs だけ）。docs だけの解消なので delta ack（Claude 側 Sonnet 5.5）で可とし、closure を record した。L3（owner、2026-10-15、head `ce116f51` の Windows native）は L3-1〜L3-4 が PASS で、廃棄の丸め（行 166.50 円 → 合計 ¥167）を owner が受け入れた。L3-3 の返品は、1 回目に既定の「レジ戻し済み」のまま保存されて在庫が動かなかった（設計どおり）ため、「レジ未処理」で返品だけを再実施して PASS。R4 は owner が承認した（2026-10-15）。review・manual・R4 を record し、helper の `ready` → `Merge gate` pass → helper の `merge`。
