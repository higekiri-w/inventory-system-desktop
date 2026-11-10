# Plan Packet: 棚卸しの評価額に価格の基準数量と店の丸め規則を入れる（R3）

2026-09-25 起草。出典は [Backlog](../../backlog.md) 「やると決めたもの」の 2 行（行番号は base `85b18a04`）: `:48`「棚卸しの評価額が価格の基準数量を持たない」（2026-09-22 の外部設計相談で指摘、Coordinator が現物で確認）と `:55`「評価額の丸めを店の規則に合わせる」（owner 決定 2026-09-24）。関連の `:35`「単位の拡張」（店の返答待ち）は本 lane の対象外。`:48` は「単位の拡張 lane の最初の成果物」とされていたが、評価額を独立した 1 本として先に進めることを owner が了承した（2026-09-25）。単位の拡張の店の返答が無くても成り立つ範囲（今ある単位 `pcs` / `cm`）で設計し、単位の拡張に依存する点は非目的か owner 決定待ちにした。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

- Phase: archive
- Risk: R3
- Plan Commit: 325861c5ccb8a0bf6ff68e44ffafe98ca570b3f1
- Amendments: 08a5cde47f1c5636aa96a9b0be6c78706946d570
- Coordinator: Opus 5.5（Claude Code main session）
- Writer: Opus 5.5 subagent（fork でない fresh context、worktree `/home/kosei/projects/inventory-system-public/.claude/worktrees/stocktake-valuation-basis`、branch `agent/stocktake-valuation-basis`）
- Plan Reviewer: fresh Opus 5.5 subagent + Codex（GPT-6 Sol、effort high）。互いに独立で Writer と別 context、後の reviewer に先の結果を見せない
- Final Reviewer: Fable 5.1（fresh context）+ Codex（GPT-6 Sol）。互いに独立な Double Audit、後の reviewer に先の結果を見せない。Codex の合否判定は是正後も外さない
- Final Review Minimum: 2
- Human Gate: ready,merge

manual なし: 画面・表示・文言・DTO・bindings を変えない（仕入原価総額は円の整数のまま。AC9 / AC10 で `src/` と `src/lib/bindings.ts` の差分 0 を確かめる）。現行 build では棚卸しの開始・保存・確定が停止中（[停止 ADR](../../adr/2026-09-23-legacy-stocktake-z004-write-stop.md) SPEC-STOP-D1〜D3）で、operator が新しい評価額を画面で見る経路も無い。r4 なし: DB schema・既存の保存値・破壊的操作を含まず、revert で戻せる。

遷移記録（append-only）:

- kickoff → spec-check → design（2026-09-25、起草役 = Opus 5.5 subagent、本 commit）: Risk R3（下記 Risk）。設計正本の更新（`docs/function-design/35-biz-stocktake-service.md` §20.5a 新設ほか 4 file、下記 Required Design Artifacts）を本 plan-first commit に同乗させた。owner 決定待ち 3 問（Design Readiness の Q1〜Q3）が残るため Phase は design に置く。3 問とも推奨案どおりなら packet・Matrix・設計正本は変えずに design → plan-draft → plan-gate の遷移だけを記録して Plan Review へ進める。推奨と違う答えなら、答えに合わせて設計正本と本 packet を直してから遷移する（Q2 が「商品ごとの列」なら migration・DTO・画面が Scope に入り、manual が要る）。
- design → plan-draft → plan-gate（2026-09-25、Coordinator）: owner が Q1〜Q3 に回答した。Q1 = はい（円未満を四捨五入して 1 円単位）、Q2 = はい（在庫単位で決め、商品ごとの列は足さない）、Q3 = はい（4 項目は含めず closeout で Backlog へ）。Q2 について、箱や袋で仕入れてばらして売る商品は実在する（店の回答 2026-09-14、project-memory の Store Premises Facts）。店が仕入れ伝票の単価と棚卸しの手計算をどの単位で扱うかを訪店の確認事項（Issue #105）に足し、答えにより単位の拡張 lane で商品ごとの基準数量の列を足す。3 問とも推奨どおりのため、packet・Matrix・設計正本は変えず Phase だけを進める。
- plan-gate（2026-09-25、Coordinator の指示で是正）: Plan Review round 1 は両 reviewer とも reject。plan-gate のまま packet を是正した。findings と裁定の詳細は round 2 の完了後に Review Response へ記録する。
- plan-gate（2026-09-25、Coordinator の指示で是正、round 2）: Plan Review round 2 は Claude 側 approve、Codex 側 reject（P2 1）。plan-gate のまま是正した。findings と裁定の詳細は Plan Review の完了後に Review Response へ記録する。
- plan-gate → plan-approved（2026-09-26、Coordinator、本 commit）: Plan Review round 3（上限、対象 `325861c5`）で Codex 側は reject（P2 1 = 35 の共通の `total_cost` 説明と biz-task-specs が評価数量を actual_count とし §20.5a の新方式と食い違う、P3 1 = Plans.md の wave 13 の段落の Plan Review の座組が古い）。round 天井に達したため追加の round は回さず、同型の一括是正として本 commit で 2 か所の評価数量の記述と Plans.md の文を直し、owner 承認（2026-09-26）のもと予算を relay 5・介入 7 に改めた。Claude 側は round 2 で approve（P3 のみ）。Plan Commit = `325861c5`（round 2 是正後の承認版）。
- plan-approved → implementing（2026-09-26、Coordinator、本 commit）: Writer（Opus 5.5 subagent）へ実装を発注する。
- Gated Amendment 1（2026-09-26、Coordinator）: `08a5cde4` を登録。main（#106 と closeout #107）の同期に合わせ、遷移記録の round 2 の行の key が round 1 の行と同じで helper が `duplicate packet fields` として packet を読めなかったため、round 2 の行の key に「、round 2」を足した。あわせて Final Review の 2 本の共通 P3（Plans.md の登録行の古い文言）を直した。Scope・AC・Risk・Final Review Minimum・Human Gate は変えない。差分は Codex の closure で確かめる。
- implementing → archive（2026-09-26、Coordinator、closeout）: PR #108 を helper 経由で merge（squash `3b94a58a`）。Final Review の 2 本（Codex GPT-6 Astra・Fable 5.1）とも approve、共通の P3 1 件は Gated Amendment 1 と同じ commit で是正し、Codex の closure が approve。packet と Matrix を archive へ移し、Plans.md と backlog を同期。

## Owner Effort Budget

- 介入回数上限: 7（owner 承認 2026-09-26。見込み: Q1〜Q3 の回答 1、予算の引上げ 1、Codex の relay 5〈Plan Review round 1・2・3、Final Review broad・closure〉は batch で他 lane と束ねる、Ready・merge 1。Plan Review を 3 round とも Codex に通した分が既定を超える理由）
- 実働時間上限: 15分（owner の作業は 3 問の回答と relay・Ready・merge の判断。画面の確認は無い）
- relay 往復上限: 5（owner 承認 2026-09-26。Plan Review の Codex round 1・2・3、Final Review の Codex broad 1・closure 1）
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
税理士へ報告する棚卸しの仕入原価総額（`stocktakes.total_cost` に保存する値）の式と、商品マスタの価格の意味（長さ商品の価格は 1 m あたり）を変える。`docs/project-profile.md` の High-risk Changes「stocktake ... semantics」「BIZ service behavior for ... stocktake」に当たり、`docs/DEV_WORKFLOW.md` Risk Tiers の「stable contract / data safety boundary に触れるなら R3」にも当たる。DB schema・command DTO・bindings・画面は変えない（推奨案どおりの場合）。破壊的操作・実データの露出・既存保存値の書換えを含まないので R4 ではない。

## Goal

Goal Invariant:

### 最小完了条件

- 棚卸しの確定が保存する仕入原価総額が、店の手計算と同じ規則で決まる: 長さ（`cm`）商品の原価を 1 m あたりとして数量を 100 で割り、商品別の金額を 1/100 円で四捨五入して合計し、最終合計を円未満で四捨五入する。個数（`pcs`）商品だけの棚卸しの総額は変更前と同じ値になる。
- この計算は `stocktake_service` の 1 組の関数（`price_basis_quantity` / `valuation_line_centi` / `valuation_total_yen`）にあり、現行の確定本体（旧本体 `legacy_complete_stocktake`）が使い、㉘ の新方式の確定も同じ関数を使うことが設計正本（35 §20.5a と「確定・legacyの取消の保留」）に書かれている。

### 失敗定義

- 長さ商品の評価額が 100 倍のまま、または個数商品だけの棚卸しの総額が変わる。
- 金額を浮動小数で計算する、丸めの段が商品別と最終合計の 2 段になっていない、四捨五入でなく切捨て・偶数丸めになる。
- 負の原価・数量や桁あふれ（i128 の中間・合計、円額の i64 への変換）で、誤った総額を黙って保存する（確定が止まらない）。
- 確定済みの `total_cost` / `valuation_cost_price` を再計算・変更する。
- 設計正本と実装の式が食い違う（旧式 `valuation_cost_price × actual_count` の記述が live な設計正本に残る）。

### 非目的

- 単位の拡張（新しい単位、m の入力・表示、Z004 の小数の数量、箱 → 個の入り数）。店の返答と単位の拡張 lane が決める。
- 商品ごとの「価格の基準数量」の列・migration・商品フォーム・商品 CSV の変更（Q2 の推奨案では作らない）。
- 入庫の原価小計・廃棄のロス原価・棚卸し記録詳細のロス原価への基準数量の適用（Q3、Backlog へ）。
- 商品フォームの価格欄に「1 m あたり」を出す表示（Q3、単位の拡張 lane の m 表示で扱う）。
- 棚卸しの停止の解除と新方式の確定の実装（㉘ の ② 〜 ⑤）。
- 商品別の金額の保存・表示、評価額の CSV 出力。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

数値は合成の例（長さ商品の原価 385 円/m・在庫 153 cm、個数商品の原価 300 円・在庫 5 個）。現行 build では棚卸しの開始・保存・確定が停止中（SPEC-STOP-D1〜D3）のため、(a) 本 lane の中で成立する操作列と、(b) ⑤ の停止解除の後に operator が得る結果を分ける。

(a) 本 lane の中で成立する操作列（利用者は ㉘ の Writer と test）

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 一時 DB に長さ商品（`cm`、原価 385、実カウント 153）と個数商品（`pcs`、原価 300、実カウント 5）の棚卸しがある | 旧本体の確定経路（`legacy_complete_stocktake`）を test から通す | `StocktakeResult.total_cost`・`stocktakes.total_cost`・操作ログの detail_json に `2089` が保存され（385 × 1.53 = 589.05、300 × 5 = 1,500、合計 2,089.05 を円未満で四捨五入）、長さ商品の `valuation_cost_price` は `385` のまま（T7） | なし | なし（T7 が観測する） |
| 確定済みの棚卸しがある | 商品の原価を 400 に変えてから保存値を読む | 確定済みの `total_cost` は `2089` のまま（再計算の経路が無い） | なし | 既存の snapshot（`valuation_cost_price`）と SPEC-STK-VAL-D6 |
| ㉘ の新方式の確定（③ / ④）を書く | Writer が 35 §20.5a の 3 関数を呼ぶ | 新方式の総額が旧本体と同じ規則で決まる | ㉘ の後続 lane の Contract Coverage Ledger で確かめる | 新方式は未実装。35「確定・legacyの取消の保留」が同じ関数を呼ぶことを縛る |
| 旧式（原価 × 数量の i64 積和）が確定できた、負の値を含まない大きな値の棚卸し（`pcs` 原価 92,233,720,368,547,759・数量 1 等） | 確定する | 新式でも確定でき、`pcs` だけなら総額は旧式と同じ（T12） | なし | Contract Probe 4 |

(b) ⑤ の停止解除の後に operator が得る結果

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 反物（在庫単位 cm）の仕入れ伝票・値札は 1 m あたり 385 円 | 商品の登録・修正で原価に 385 を入れる（現行の商品フォームのまま） | 原価 385 が「1 m あたり」として扱われる | なし（画面は変えない） | 利用者が 1 cm あたりに直さず伝票の値を入れること。店は残り・仕入れ・値札をすべて m で扱う（project-memory `:113`・`:190`）。フォームに「1 m あたり」の表示は無い（Q3） |
| 年末の棚卸しで反物 153 cm、個数商品 5 個を数えた | 棚卸しを確定する | 結果画面・前回サマリ・記録詳細の仕入原価総額が ¥2,089（円の整数のまま） | ⑤ の停止解除と、③ / ④ の新方式の確定が §20.5a の関数を呼ぶこと | 本 lane の時点では確定が停止中（SPEC-STOP-D1〜D3） |
| 確定済みの棚卸しがある | 税理士へ仕入原価総額を報告する（店の手計算と照合する） | 店の規則（商品別の金額を小数第 2 位まで、最終合計で四捨五入）と同じ額 | なし | 最終合計の四捨五入が円未満（1 円単位）であること（Q1） |
| 箱で仕入れて 1 個ずつ売る商品（店の回答 2026-09-14 で 1 種） | 原価を登録する | 本 lane では 1 個あたりの円の整数で登録する。箱あたりの原価は表せない | 単位の拡張 lane で店の記録の仕方を確かめる | 店がその原価を箱あたり・1 個あたりのどちらで記録しているか（未確認、SPEC-STK-VAL-D1 の見直す条件） |

この文書を完了できる条件は (a) の成立で、評価額の式が設計正本と確定本体（旧本体）の両方で店の規則に揃い、㉘ が同じ関数を使う土台になる。通常運用を達成できるのは (b) で、operator が通常の年末棚卸しでこの額を得るのは、棚卸しの停止を解く ⑤ の後である（製品の目的は本 lane だけでは未達。owner 了承 2026-09-25）。

## Scope

行番号は base `85b18a04` のもの。

- S1 `src-tauri/src/biz/stocktake_service.rs`（非公開関数 3 つの追加と旧本体の置換）
  - `fn price_basis_quantity(unit: ProductStockUnit) -> i64`: `ProductStockUnit::Pcs => 1`、`ProductStockUnit::Cm => 100`。wildcard arm を置かない（`crate::db::product_repo::ProductStockUnit`、定義は `product_repo.rs:35`）。
  - `fn valuation_line_centi(cost_price: i64, quantity: i64, basis: i64, product_code: &str) -> Result<i128, BizError>`: `cost_price < 0` か `quantity < 0` なら、関数の中で商品コード入りの負の error を作って返す（検査と文言はここ 1 か所）。`i128::from(cost_price) * i128::from(quantity)`（plain の `*`。「i64 の積は i128 に収まる」の comment を 1 行置く）に `checked_mul(100)` を掛け、`basis` で割って四捨五入（i128 の商 q・余り r で `2r >= basis` なら q + 1。`r < basis ≤ i64::MAX` なので `2r` は i128 で溢れない。実際の呼出しは basis 1 / 100、単体 test は合成値〈最大 250〉を直接渡す）。× 100 の桁あふれは既存のオーバーフローの error（文言は 35 §20.5a SPEC-STK-VAL-D5）。私有の error 型は作らない。
  - `fn valuation_total_yen(lines: &[i128]) -> Result<i64, BizError>`: 商品別の金額を i128 の `checked_add` で合計し、円未満で四捨五入（商 `total / 100`・余り `total % 100` で余りが 50 以上なら商 + 1。各行は負の検査を通った 0 以上の値）、円額を `i64::try_from` で変換する。合計の桁あふれと変換の失敗は既存のオーバーフローの error（文言を固定。既存の `test_complete_req205_total_cost_overflow` は新式では変換で初めて error になる）。0 行なら `Ok(0)`。
  - `legacy_complete_stocktake`（`:440`）のステップ 5〜6: `let mut total_cost`（`:495`）を商品別の金額の `Vec<i128>`（`lines`）にし、`:504-520` の `checked_mul` / `checked_add` を `valuation_line_centi(valuation_cost_price, item.actual_count, price_basis_quantity(product.product.stock_unit), &item.product_code)?` の push に置き換え、`:552` の直前で `valuation_total_yen(&lines)?` を `total_cost` にする。`valuation_cost_price` の保存（`:505`）、操作ログの文言・detail_json、StocktakeResult の型は変えない。
  - 3 関数は `pub` にしない（`design_compliance_test.rs:482` は `syn::Visibility::Public` だけを突合する）。非 test build の dead_code は Contract Probe 1 のとおり警告にならないので、属性を足さない。clippy が警告を出したら属性を足さずに Coordinator へ返す。
- S2 test（同じ file の `#[cfg(test)] mod tests`）: Matrix の T1〜T12。長さ商品の fixture は既存の `seed_product_custom`（`:916`）に倣い、`stock_unit: "cm"` を渡せる test helper を足す。既存 test の期待値を変えない（`test_complete_req205_total_cost_multiple_products` の 3500、`test_complete_req205_total_cost_overflow` の「オーバーフロー」）。test 名か本文に `REQ-205` と `SPEC-STK-VAL-Dn` を付ける。
- S3 `docs/function-design/90-traceability.md` の再生成（`cd src-tauri && cargo run --bin generate_traceability`）。S2 で REQ-205 の test が増えるため。手で編集しない。並走 lane が同じ生成物を再生成した場合は、main 同期の後に再生成し直す。
- 設計正本の更新（本 plan-first commit で済み。Writer は編集しない）: `docs/function-design/35-biz-stocktake-service.md`（§20.5a 新設、§20.2 `total_cost`、§20.5 ステップ 5〜6 とエラー、設計判断の overflow、「確定・legacyの取消の保留」の新方式の式、更新履歴）、`docs/db-design/tracking-system-tables.md`（`total_cost` / `valuation_cost_price` の説明と設計意図）、`docs/db-design/master-tables.md`（`selling_price` / `cost_price` の説明と設計意図「価格の基準数量」）、`docs/architecture/biz-task-specs.md`（BIZ-06 の確定の式）。
- 登録（plan-first commit に同乗）: `docs/Plans.md` の「次の行動」に本 packet と Matrix の link を 1 行。

呼出し側の確認: 3 関数は新設で呼出し元は旧本体だけ。旧本体の呼出し元は test だけ（`rg -n 'legacy_complete_stocktake' src-tauri` = `stocktake_service.rs` の test 群、`csv_import_service/tests/legacy_stop_tests.rs:221,331`、`csv_import_service/tests/cross_feature_tests.rs:484`）。後 2 file は `total_cost` を assert しない（`rg -n 'total_cost' src-tauri/src/biz/csv_import_service/tests/` = 0 hit）ため変えない。署名を変える関数・移動する関数は無い。

## Non-scope

- DB migration、`products` / `stocktake_items` の列追加、`schema_v1.rs` ほか schema file。
- command DTO・`src/lib/bindings.ts`・`src/` の frontend 全体（棚卸し記録詳細のロス原価の計算〈`StocktakeRecordDetailPage.tsx:181-183`〉を含む）。
- 入庫の `line_cost` / `total_cost`（`receiving_repo.rs:229`・`:250`）、廃棄の `line_loss_cost`（`disposal_repo.rs:647`）と入力画面の合計（`src/features/disposal/lib/disposal-request.ts:40`）。
- `src-tauri/src/seed_demo.rs`（INSERT は列を明示し、`stock_unit` の値の意味だけが変わる。長さ商品の demo 価格〈100〜5000 円〉は 1 m あたりとしても成り立つ）と既存の test fixture（`pcs` の値は変わらない）。
- 旧本体の停止・解除（SPEC-STOP-D1〜D6）、新方式の確定の実装、`stocktake_repo` の関数。
- `docs/backlog.md` の `:48` / `:55` の完了処理と、次の Backlog 起票（closeout で行う）: Q3 の 4 項目（入庫の原価小計・原価合計は式の正本 `docs/function-design/21-io-inventory-repo.md:103`、廃棄のロス原価は同 `:354`）、手動販売の金額の初期値（`docs/function-design/62-ui-manual-sale.md:26`・`:165` UI-04-D6、`src/features/manual-sale/lib/manual-sale-row-utils.ts:24,40`）、⑤ で `total_cost` を wire・表示まで正確に扱うこと（Boundary / Wire Contract の precision/range）。

## Acceptance Criteria

- **AC1** `cd src-tauri && cargo test --lib biz::stocktake_service` が pass し、Matrix T1〜T12 の test を含む。
- **AC2** 既存の期待値を変えない: `rg -n 'assert_eq!\(result.total_cost, 3500\)' src-tauri/src/biz/stocktake_service.rs` が 1 hit（base で実行して 1 hit、`:1207`）で、`test_complete_req205_total_cost_overflow` が pass する。
- **AC3** 浮動小数を使わない: `rg -c 'f64|f32' src-tauri/src/biz/stocktake_service.rs` が 0 件（base で実行して出力なし = 0 件）。
- **AC4** mutant（Writer が注入 → 対象 test の FAIL を確認 → 復元、Final Reviewer が独立に再注入）: (1) `price_basis_quantity` の `Cm => 100` を `1` にする → T1・T7 が FAIL (2) 商品別の金額の四捨五入を切捨て（商だけ）にする → T2 が FAIL (3) `valuation_total_yen` を切捨てにする → T3・T8 が FAIL (4) `valuation_total_yen` を偶数丸めにする → T3（`50` → 1 円）・T8 が FAIL (5) `valuation_line_centi` の負の検査を外す → T5・T10 が FAIL。乗算・合計・変換は段ごとに分ける: (7b) 商品別の金額の `× 100` の `checked_mul` を `wrapping_mul` にする → T6 が FAIL (7c) `valuation_total_yen` の合計の `checked_add` を `wrapping_add` にする → T12 の 3 行の入力が FAIL (7d) `valuation_total_yen` の `i64::try_from` を `as i64` にする → T11・T12・既存の `test_complete_req205_total_cost_overflow` が FAIL（既存 test が red になるのは、新式でこの入力が i64 への変換で初めて error になり、変換の失敗の文言が「オーバーフロー」に固定されているため）。(7a)〈原価 × 数量〉は i64 の積が i128 に必ず収まり殺せないため置かない（Matrix Residual）。(7b)〜(7d) の red は Contract Probe 4 の試作で確かめた。結果は PR body に 1 行ずつ書く。
- **AC5** `cd src-tauri && cargo run --bin generate_traceability -- --check` が exit 0（S3 の再生成後）。
- **AC6** `bash scripts/local-ci.sh full` が pass（fmt・clippy `-D warnings`・cargo test・design_compliance・bindings・traceability を含む）。
- **AC7** `bash scripts/doc-consistency-check.sh` が ERROR 0。
- **AC8** 旧式が live な設計正本に残らない: `rg -n 'valuation_cost_price × actual_count|valuation_cost_price \* actual_count|valuation_cost_price×actual_count|この値×actual_count|× 確定時評価原価|× 評価原価' docs --glob '!docs/archive/**' --glob '!docs/backlog.md' --glob '!docs/plans/**'` が 0 hit（plan-first commit の直前の base `85b18a04` で 7 hit〈`adr/2026-09-18-stocktake-time-evidence.md:58`、`biz-task-specs.md:487`、`35-biz-stocktake-service.md:36`・`:101`・`:337`、`tracking-system-tables.md:163`・`:170`〉、Plan Review round 1 の是正 commit の時点で 0 hit〈exit 1〉を確認済み。Final Review で再実行する）。
- **AC9** bindings を変えない: `git diff --exit-code origin/main -- src/lib/bindings.ts` が exit 0。
- **AC10** 変更 file が Scope の内側: `git diff --name-only origin/main...HEAD` が plan-first commit と Plan Review round 1・round 2 の是正 commit の 8 file（round 2 の是正 commit の時点で実測 8）と `src-tauri/src/biz/stocktake_service.rs`・`docs/function-design/90-traceability.md` だけ（`src/`・`src-tauri/src/db/` を含まない）。

## Design Sources

- Requirements / spec: REQ-205（棚卸しによる在庫数の補正、`docs/spec/requirements.md:24`）、SP-205（`docs/spec/requirements-coverage.md:52`）、SP-205-08（仕入原価総額、`tracking-system-tables.md` の設計意図）
- Architecture: `docs/architecture/biz-task-specs.md` BIZ-06「棚卸し確定」（BIZ 内の変更。UI / CMD / IO は不変）
- Function / command / DTO: `docs/function-design/35-biz-stocktake-service.md` §20.5a（新設）・§20.5・「確定・legacyの取消の保留」・§20.0（停止）
- DB: `docs/db-design/tracking-system-tables.md` stocktakes / stocktake_items、`docs/db-design/master-tables.md` products
- Screen / UI: `docs/function-design/73-ui-stocktake.md`（`total_cost` の表示、変更なし）
- Decision log / ADR: [時点証拠 ADR](../../adr/2026-09-18-stocktake-time-evidence.md) SPEC-STK-TIME-D7（過去評価を上書きしない）、[停止 ADR](../../adr/2026-09-23-legacy-stocktake-z004-write-stop.md) SPEC-STOP-D1〜D6。店の事実は `docs/project-memory.md` Store Premises Facts（`:108`・`:113`・`:129`・`:162`・`:190`）と `docs/evidence/hearing-2026-09-14-stock-units.sanitized.md`

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | 35 §20.5a（SPEC-STK-VAL-D1〜D6）、§20.5 ステップ 5〜6・エラー、biz-task-specs BIZ-06 | updated in this PR（plan-first commit） |
| Command / DTO / generated binding / wire shape | 該当なし（StocktakeResult の型・bindings 不変） | existing sufficient |
| DB / transaction / audit / rollback / migration | tracking-system-tables（`total_cost` の求め方）、master-tables（価格の基準数量）。schema・migration は無し | updated in this PR（plan-first commit） |
| Screen / UI / route state / Japanese wording | 73-ui-stocktake（円の整数の表示のまま） | existing sufficient |
| CSV / TSV / report / import / export format | 該当なし | existing sufficient |
| Durable decision / ADR | 35 §20.5a に決定・理由・不採用案・見直す条件を置き、master-tables の設計意図から参照する。decision-log は使わない（並走 lane が D-092 / D-093 を予約済みで、ほかの並走 lane の予約が未確認のため採番の衝突を避ける） | updated in this PR（plan-first commit） |

## Registration / Generation Obligations

| 変更対象 | 登録・生成義務 |
|---|---|
| REQ coverage の変更（REQ-205 の test が増える） | `cd src-tauri && cargo run --bin generate_traceability` で `docs/function-design/90-traceability.md` を再生成（S3、AC5） |

他の行（Tauri command / function-design doc 新設 / source doc 新設・改名 / consultation relay / route / operator 画面）は該当なし。35 に書いた `fn` 3 つは非公開のため、`design_compliance_test` の INFO（not-yet-implemented）に載り続けるが失敗しない（コード側の突合は `pub` だけ）。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| REQ-205 / SP-205-08 | 35 §20.5a、master-tables products 設計意図 | SPEC-STK-VAL-D1 | 店は長さ商品の残り・伝票・値札を m で扱い、在庫は cm の整数。単位で基準数量が決まる。不採用: 商品ごとの列（利用者が 100 を入れる誤りの入口、migration・DTO・画面）、原価を 1 cm あたりの小数で持つ | S1 `price_basis_quantity` | T1 / T7 |
| REQ-205 | 35 §20.5a | SPEC-STK-VAL-D2 | 浮動小数は 0.005 の境界を誤る（Probe 2）。中間・合計は i128 で、最後の円額だけを i64 へ変換する。不採用: f64、10 進小数 crate の追加、中間を i64 に限る（旧式が確定できた値を × 100 で拒む、Probe 4） | S1 | AC3 / T6 / T12 |
| REQ-205 | 35 §20.5a | SPEC-STK-VAL-D3 | 店の「商品別の金額を小数第 3 位で四捨五入」と同値の正確な四捨五入 | S1 `valuation_line_centi` | T2 / T4 |
| REQ-205 / SP-205-08 | 35 §20.5a、tracking-system-tables stocktakes | SPEC-STK-VAL-D4 | 最終合計で四捨五入（owner 決定 2026-09-24）。円の整数の列と表示を保つ | S1 `valuation_total_yen` | T3 / T8 |
| REQ-205 | 35 §20.5a、§20.5 エラー | SPEC-STK-VAL-D5 | 誤った総額を黙って保存しない。負の検査と文言は関数の 1 か所 | S1 | T5 / T6 / T10 / T11 / T12 |
| REQ-205 | 35 §20.5a、時点証拠 ADR D7 | SPEC-STK-VAL-D6 | 過去の報告値を変えない | 非接触（再計算の経路を作らない） | T9（既存値の不変） |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: yes。35 §20.5a に式・型・丸め・検査・非遡及と、決定の理由・不採用案・見直す条件がある。master-tables の products から価格の基準数量を引ける。
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: 「価格の基準数量は在庫単位で決まる（pcs = 1、cm = 100）」「金額は 1/100 円の整数」「入庫・廃棄・記録詳細のロス原価は未対応の既知の不整合」を 35 §20.5a と master-tables へ置いた。
- Assumptions and constraints: 長さ商品の価格は 1 m あたりで登録される（店の事実）。現行 build は確定が停止中で、新方式の確定は ㉘ の後続 lane が §20.5a の関数を呼ぶ。
- Deferred design gaps, risk, and follow-up target: Q3 の 4 項目（入庫・廃棄のロス原価と原価小計、記録詳細のロス原価、商品フォームの「1 m あたり」）と手動販売の金額の初期値（UI-04-D6）→ closeout で Backlog へ。JS の safe integer を越える `total_cost` の wire・表示 → ⑤（Design Readiness の申し送り、closeout で Backlog へ）。箱あたりの原価（SPEC-STK-VAL-D1 の見直す条件）→ 単位の拡張 lane。
- Test Design Matrix can cite design decision IDs or source doc sections: yes（SPEC-STK-VAL-D1〜D6）。
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: 「総額は店の規則と一致する」の例外 = 本契約より前に確定した記録（旧式のまま、D6。本番には無い）、箱あたりの原価（表せない、D1 の見直す条件）。「浮動小数を使わない」は AC3 で本 file に限って検査する（frontend の `formatYen` は円の整数を表示するだけ）。「個数商品の総額は不変」は基準数量 1 で商品別の金額が `原価 × 数量 × 100` の整数になり、円未満の端数が出ないことから成り立つ（T9）。

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | 該当なし。POS の数量（Z004・EJ）は触らない。`ej_parser.rs:502` の `q × p == amount` はレジの数量と単価の照合で、在庫単位の基準数量とは別 | なし |
| Fact check / design decision split | 店の事実（長さ商品は m で扱い値札は 1 m あたり、レジでも数量 1 = 1 m で打てる〈未運用〉、店の丸め規則）は project-memory と聞き取り記録から引き、式・型・関数の置き場所は設計判断として 35 §20.5a に分けた。丸め規則の読み方の残り（最終合計の単位）は Q1 | 35 §20.5a、Design Readiness Q1 |
| Lifecycle / retry | 確定は 1 TX。負・桁あふれは確定を止めて ROLLBACK（T10 で header・明細・在庫の不変を確認）。再試行は原因のデータを直して同じ確定を呼ぶ（既存と同じ） | T10 |
| Operator workflow | 画面・操作は変えない。長さ商品の原価を 1 m あたりで入れることは店の運用と一致するが、フォームに表示は無い（Q3） | Q3 |
| Replacement path | 単位を足すと `price_basis_quantity` の match が compile error になり、基準数量の決定を強制する。商品ごとの基準数量が要ると分かったら列を足し、この関数の呼出しを列の値へ置き換える（35 §20.5a の見直す条件） | 35 §20.5a |
| Data safety / evidence | 合成の価格・数量だけを test と packet に使う。店の実価格は使わない | Data Safety |
| Reporting / accounting semantics | 税理士へ報告するのは総額だけ（project-memory `:163`）。商品別の金額は保存しない。入庫・廃棄のロス原価は別の意味の金額で、店の端数の規則が未確認のため本契約を流用しない（Q3） | 35 §20.5a の範囲の外、Q3 |
| Manual verification | 該当なし。画面の変化が無く、確定の画面経路は停止中 | なし |
| 環境・再現性 | 新しい環境依存は無い。Probe 1 は rustc 1.94.1 の dead_code の挙動で、toolchain の pin は repo に無い。版が変わって警告が出れば clippy `-D warnings`（AC6）が落として検出する | AC6 |

## Design Readiness

- Existing design docs are sufficient because: 不十分だったため本 commit で更新した（下記）。
- Source docs updated in this PR: 35 §20.5a ほか、tracking-system-tables、master-tables、biz-task-specs（Scope の「設計正本の更新」）。
- Design gaps intentionally deferred: 箱あたりの原価（単位の拡張 lane）、Q3 の 4 項目と手動販売の金額の初期値（Backlog）。
- ⑤ への申し送り（後続必須条件）: ⑤ で `total_cost` を wire・表示まで正確に扱う（十進文字列等）。入力例: `cm` 原価 `100000000000000001` 円/m・数量 `100` cm は新式で保存でき、JS の `number` では 1 円ずれる。本 lane は関数の契約（i64 の範囲）を変えない。closeout で Backlog に起票する。
- Durable decisions discovered in this plan and promoted to source docs: SPEC-STK-VAL-D1〜D6。

owner 決定（2026-09-25 に回答済み。3 問とも推奨案どおりで、設計正本と本 packet は変えない。Q2 は店の回答で単位の拡張 lane が見直す）:

- **Q1 店の規則の「最終合計で四捨五入」は、円未満を四捨五入して 1 円単位にすることか**。推奨: はい。`stocktakes.total_cost` の円の整数と画面の表示をそのまま使える。10 円単位などの別の単位なら SPEC-STK-VAL-D4 を直す。
- **Q2 価格の基準数量を在庫単位で決める（`pcs` = 1 個あたり、`cm` = 1 m あたり）とし、商品ごとの列を足さないか**。推奨: はい。店は長さ商品を m で扱い値札は 1 m あたりなので、今ある単位では単位と基準数量が一対一で、利用者が商品ごとに 100 を入れて誤る入口を作らない。migration・DTO・商品フォーム・商品 CSV も変えずに済む。箱で仕入れて 1 個ずつ売る商品の原価を店が箱あたりで記録しているなら、そのとき（単位の拡張 lane）に商品ごとの列を足す。「今から列を持つ」を選ぶと、本 lane に migration・DTO・商品フォームが入り、画面の manual 確認が要る。
- **Q3 同じ「原価 × 数量」の誤りを持つ次の 4 つを本 lane に含めず、Backlog に起票するか**: (a) 入庫の原価小計・原価合計 (b) 廃棄のロス原価（入力画面の合計を含む） (c) 棚卸し記録詳細のロス原価（画面が計算している） (d) 商品フォームの価格欄に「1 m あたり」を出す表示。推奨: 含めない。(a)(b) は仕入れ伝票・廃棄の端数の規則が店に未確認で、棚卸しの規則を流用できない。(c) は記録詳細を作り直す ⑤ でロス原価を BIZ から返す形にするのが筋で、⑤ までは新しい確定が起きない。(d) は単位の拡張 lane が m の入力・表示と一緒に扱う。どれも本 lane の前から同じ誤りで、本 lane で悪化しない。

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): 評価額の規則は BIZ-06（`stocktake_service`）だけに置く。IO（`stocktake_repo`）は保存するだけ、CMD / UI は不変。
- Backend function design: 35 §20.5a のシグネチャと処理ステップ。
- Command / DTO / data contract: StocktakeResult.total_cost（i64、円）不変。`stocktakes.total_cost` の意味が「店の規則で求めた円の整数」になる。
- Persistence / transaction / audit impact: schema 不変。確定の TX 境界・操作ログは不変。error は TX ごと ROLLBACK。
- Operator workflow / Japanese UI wording: 画面の文言は不変。新しい error 文言 1 つ（SPEC-STK-VAL-D5、通常のデータでは出ない）。
- Error, empty, retry, and recovery behavior: 負・桁あふれは ValidationFailed で確定しない。`valuation_total_yen` に 0 行を渡した値は 0（T12。旧本体では start が 0 件を拒否するため到達しない）。
- Testability and traceability IDs: REQ-205、SPEC-STK-VAL-D1〜D6。

## Contract Probe

- 非公開 fn を `#[cfg_attr(not(test), expect(dead_code))]` の関数だけが呼ぶと、非 test build で呼ばれる側に dead_code 警告が出るか: scratchpad で private mod の `helper` を `expect(dead_code)` の `legacy` だけから呼ぶ lib を `rustc 1.94.1 --crate-type lib` で build -> exit 0、警告なし（`--test` build では test から呼ばない条件で両方が警告。本 lane では test が旧本体を呼ぶ）。
- 浮動小数の四捨五入の誤り: python3 `round(1.005, 2)` -> `1.0`（`1.005` の保持値は `1.00499999999999989342`）、`Decimal('1.005').quantize(Decimal('0.01'), ROUND_HALF_UP)` -> `1.01`。整数の四捨五入 `(2n + d) // (2d)` で `1000 × 5 × 100 / 12` -> `41667`（416.67 円）、`385 × 153 × 100 / 100` -> `58905`（589.05 円）。
- design_compliance が非公開 fn を突合しないこと: `src-tauri/tests/design_compliance_test.rs:482` が `syn::Visibility::Public(_)` だけを集める -> 35 に書いた非公開 fn は design_compliance の INFO（not-yet-implemented）に載り続けるが失敗しない（`extract_functions_from_design_doc` は 35 の `fn` をすべて拾い、コード側は `pub` だけを突き合わせる）。
- i128 の中間・合計と kill 入力（2026-09-25、`ca280c79` の使い捨て worktree で 3 関数を 35 §20.5a どおりに試作し、`cargo test --lib biz::stocktake_service` を実行。試作は削除済み）: 正しい実装で既存の test（3500・overflow を含む）と試作の test 41 件がすべて pass。
  - T6: `valuation_line_centi(1 << 62, 737_869_762_948_382_065, 100, _)` -> オーバーフローの Err。`(i64::MAX, 1, 1, _)` -> `Ok(i64::MAX × 100)`、`(i64::MAX, 1, 100, _)` -> `Ok(i64::MAX)`。旧 T6 の入力 `(i64::MAX / 100 + 1, 1, 1, _)` は新式で `Ok(9223372036854775900)` になるため差し替えた。
  - T11: pcs 2 品 原価 `i64::MAX / 2 + 1`・数量 1 を旧本体で確定 -> オーバーフローの Err、header は `in_progress`。旧 T11 の入力（原価 `i64::MAX / 200 + 1`）は新式で `Ok(92233720368547760)` になるため差し替えた。
  - T12: `valuation_total_yen(&[i64::MAX × 100 + 49])` -> `Ok(i64::MAX)`、`+ 50` -> オーバーフローの Err、`&[]` -> `Ok(0)`、`&[valuation_line_centi(92_233_720_368_547_759, 1, 1, _)]` -> `Ok(92_233_720_368_547_759)`（中間を i64 に限ると × 100 で桁あふれする値）。原価 `1 << 60`・数量 `983_826_350_597_842_753`・basis 1 の行 3 本 -> オーバーフローの Err（2 本でも Err）。
  - (7b) `× 100` の `checked_mul` を `wrapping_mul` -> T6 だけが FAIL（mutant の `valuation_line_centi` は 1/100 円で `1,660,206,966,633,859,645`〈約 1.66×10^18、円にして約 1.66 京円〉を Ok で返す。i128 の wrapping を python で再計算）。
  - (7c) 合計の `checked_add` を `wrapping_add` -> T12 の 3 行だけが FAIL（mutant の合計は正へ wrap し、約 5.07 × 10^17 円を Ok で返す）。
  - (7d) `i64::try_from` を `as i64` -> T11・T12・既存の `test_complete_req205_total_cost_overflow` の 3 件が FAIL（mutant は `i64::MIN` を Ok で返す）。

## Contract Coverage Ledger

adjacent-contract sweep: 35 の §20.5a・§20.5（ステップ 1〜10、エラー、設計判断 4 つ）・§20.2 StocktakeResult・「確定・legacyの取消の保留」、tracking-system-tables の stocktakes / stocktake_items、master-tables の products 価格列を走査した。本 Scope が行使する契約を下表に置き、行使しない契約（force_fill の補完・差異 movement・整合性チェック・停止）は既存 test のまま non-scope とした。

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| SPEC-STK-VAL-D1 基準数量は単位で決まる（pcs 1 / cm 100、wildcard なし） | S1 `price_basis_quantity` | T1、T7 | — |
| SPEC-STK-VAL-D2 金額は整数（1/100 円・円）、浮動小数なし。中間・合計は i128、円額だけ i64 | S1 | AC3、T6、T12 | — |
| SPEC-STK-VAL-D3 商品別の金額は正確な値を 1/100 円で四捨五入 | S1 `valuation_line_centi` | T2、T4 | — |
| SPEC-STK-VAL-D4 総額は合計を円未満で四捨五入、列と表示は円の整数のまま | S1 `valuation_total_yen`、旧本体ステップ 6 | T3、T7、T8 | — |
| SPEC-STK-VAL-D5 負・桁あふれ（i128 の × 100・合計、円額の i64 への変換）は ValidationFailed で ROLLBACK。負の検査と文言は `valuation_line_centi` の 1 か所 | S1 | T5、T6、T10、T11、T12、既存 `test_complete_req205_total_cost_overflow` | — |
| SPEC-STK-VAL-D6 確定済みの値を再計算しない | 非接触 | 既存の記録詳細・前回サマリの test（読取りは保存値を返す） | — |
| 35 §20.5 ステップ 5c-d 確定時評価原価の snapshot（`valuation_cost_price` = 確定時の `cost_price`、基準数量あたりの値のまま保存） | S1（保存は不変） | T7（保存値 385 を確認） | — |
| 35 §20.5 ステップ 8 操作ログの `total_cost`（summary・detail_json） | 不変（新しい total_cost が入る） | T7（detail_json の total_cost） | — |
| 35「確定・legacyの取消の保留」新方式の total_cost は §20.5a の関数 | 設計のみ（実装は ㉘ の後続 lane） | なし（新方式が未実装） | non-scope（㉘） |
| master-tables 価格の基準数量（selling_price / cost_price の意味） | 設計のみ（価格を使う他の計算は範囲の外） | なし | non-scope（Q3） |
| 35 §20.5 設計判断「valuation_cost_price のタイミング」（確定時の原価） | 不変 | 既存 | — |
| 35 §20.0 停止（公開関数は最初の文で停止） | 非接触 | 既存 | — |

## Test Plan

[Test Design Matrix](test-matrices/2026-09-25-stocktake-valuation-basis.md)。

- targeted tests: T1〜T12（`cd src-tauri && cargo test --lib biz::stocktake_service`）
- negative tests: T5 / T6 / T10 / T11 / T12
- compatibility checks: T9（既存の `pcs` の総額 3500 が不変）、既存の overflow test、`csv_import_service/tests` の旧本体の呼出し（total_cost を見ない）
- data safety checks: 合成の価格・数量だけ
- main wiring/integration checks: T7 / T8 / T10 / T11 は旧本体の確定（TX・保存・操作ログ）を通す。新方式の配線は ㉘ の後続 lane
- commit 構成: 実装（S1）と test（S2）は同じ 1 commit（subject 例 `fix(stocktake): 棚卸しの評価額に価格の基準数量と店の丸めを入れる`）。`90-traceability.md` の再生成（S3）は次の別 commit。packet / Matrix / `Plans.md` / 設計正本は Writer が編集しない（食い違いを見つけたら編集せずに Coordinator へ返す）
- 許容する書込み: 指定 command が作る untracked / gitignore の出力（`bash scripts/local-ci.sh full` が書く `.local/ci-evidence/`、`src-tauri/target/`）。tracked で書いてよいのは Scope の 2 file だけ

## Boundary / Wire Contract

- producer: BIZ-06 の確定（旧本体。後に新方式）
- consumer: `stocktake_repo::complete_stocktake` → `stocktakes.total_cost`、StocktakeResult.total_cost → CMD → 結果画面、`get_last_completed_stocktake` / 記録詳細（保存値の読取り）、操作ログ
- wire type: `total_cost` は i64（bindings `number`）、DB は INTEGER。不変
- internal type: 中間・商品別の金額・合計は 1/100 円の i128（非公開、保存しない）。最後の円額だけを i64 へ検査付きで変換する
- precision/range: 円の整数。i128 の中間・合計は checked、円額の i64 への変換は検査付き。現実の worst case（35 設計判断: 約 40 兆円）は JS の safe integer（約 9 × 10^15 円）を越えないが、i64 の範囲で確定できる成功値は越えうる。例: `cm` 原価 `100000000000000001` 円/m・数量 `100` cm は新式で `100000000000000001` 円を保存でき、JS の `number` で読むと 1 円ずれる（`100000000000000000`）。本 lane は wire・画面を変えないため、関数の契約（i64 の範囲）は変えず、⑤ で `total_cost` を wire・表示まで正確に扱う（十進文字列等）ことを後続必須条件にする（Design Readiness の ⑤ への申し送り、Matrix Residual、closeout の Backlog 候補）
- round-trip path: 確定 → DB → 読取り → 画面。値の変換は無い
- invalid input: 負の原価・数量、桁あふれ → ValidationFailed、書込み 0
- compatibility: 既存の完了済み記録の total_cost は変えない（D6）。`pcs` だけの棚卸しは同じ値。`cm` 商品を含む開発・demo DB の新しい確定は値が変わる（本番 DB は無い、project-memory `:129`）

## Review Focus

- 「今ある単位では商品別の丸めが起きない」の主張が正しいか、2 段の丸めが関数の形で守られているか（T4 は合成の basis で確かめる）。
- 旧本体を変えることが停止 ADR の「旧本体は既知の不具合を再現し続ける」（SPEC-STOP-D3 の説明）と衝突しないか（評価額は STK-1 / STK-2 の再現に関わらない、という判断の妥当性）。
- Q2 の推奨（単位で決める）が、店の事実と単位の拡張 lane の選択肢を狭めていないか。
- 負の検査の追加で、既存の正常なデータの確定が止まる経路が無いか（原価は BIZ-01 の入力で 0 以上、数量は `update_count` で 0 以上）。
- 保守者可読性: 四捨五入の式が 1 箇所にあり、`2r >= basis` の意図が読めるか。第 1 段の plain `*` が「i64 の積は i128 に収まる」の comment と併せて読めるか。

## Spec Contract

Contract ID: SPEC-STK-VAL-2026-09-25（決定の正本は 35 §20.5a の SPEC-STK-VAL-D1〜D6）

| Contract | Plan Step | Test |
|---|---|---|
| C1: `price_basis_quantity(Pcs) = 1`、`price_basis_quantity(Cm) = 100` | S1 | T1 |
| C2: `valuation_line_centi` は正確な値を 1/100 円で四捨五入する（416.666… → 41667、0.005 円 → 1、0.004975… → 0） | S1 | T2 |
| C3: `valuation_total_yen` は 1/100 円の合計を円未満で四捨五入する（`[58950]` → 590、`[58949]` → 589、`[50]` → 1、`[250]` → 3、`[0]` → 0） | S1 | T3 |
| C4: 丸めは商品別と最終合計の 2 段（0.004 円の行 200 本は 0 円で、合計してから丸める 1 円ではない） | S1 | T4 |
| C5: 長さ商品を含む確定の total_cost は店の規則の値（385 円/m × 153 cm + 300 円 × 5 個 = ¥2,089）で、`valuation_cost_price` は 385 のまま保存される | S1 | T7 |
| C6: 総額の四捨五入が確定の保存値に効く（0.50 円 → 1 円、0.49 円 → 0 円） | S1 | T8 |
| C7: 個数商品だけの確定の total_cost は変更前と同じ | S1 | T9 |
| C8: 負の原価・数量（文言に商品コード）、桁あふれ（i128 の × 100・合計、円額の i64 への変換）は ValidationFailed で、header は in_progress・total_cost NULL・在庫不変のまま | S1 | T5 / T6 / T10 / T11 / T12 |
| C9: 負の値を含まない入力で旧式が確定できたものは新式でも確定でき、`pcs` だけなら総額は旧式と同じ（円額 `i64::MAX` まで確定でき、それを越えると error） | S1 | T12 |

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| C1 / SPEC-STK-VAL-D1 | S1 | T1 / T7 | 単位と基準数量の一対一 | AC1 / AC4 (1) |
| C2 / SPEC-STK-VAL-D3 | S1 | T2 | 正確な四捨五入 | AC1 / AC4 (2) |
| C3 / SPEC-STK-VAL-D4 | S1 | T3 | 円未満の四捨五入 | AC1 / AC4 (3)(4) |
| C4 / SPEC-STK-VAL-D3・D4 | S1 | T4 | 2 段の丸め | AC1 |
| C5 / SPEC-STK-VAL-D1・D4 | S1 | T7 | 旧本体の配線と snapshot | AC1 / AC4 (1) |
| C6 / SPEC-STK-VAL-D4 | S1 | T8 | 保存値の丸め | AC1 / AC4 (3)(4) |
| C7 / SPEC-STK-VAL-D6 | S1 | T9 | pcs の不変 | AC2 |
| C8 / SPEC-STK-VAL-D5 | S1 | T5 / T6 / T10 / T11 / T12 | ROLLBACK | AC1 / AC4 (5)(7b)(7c)(7d) |
| C9 / SPEC-STK-VAL-D2・D5 | S1 | T12 | 旧式が確定できた値を狭めない | AC1 / AC4 (7d) |
| SPEC-STK-VAL-D2 | S1 | 静的検査 | 浮動小数なし | AC3 |
| 設計正本の式 | 本 commit | 静的検査 | 旧式の残存 | AC8 |

## Data Safety

- 実店舗の商品名・価格・原価・数量を test・packet・PR に入れない。例の価格（385 円/m、300 円）と数量は合成値
- local-only paths: なし（`.local/ci-evidence/` は local-ci が書く untracked の出力で、commit しない）
- synthetic-only paths: `src-tauri/src/biz/stocktake_service.rs` の test fixture（一時 DB）

## Implementation Results

Fill after implementation.

## Review Response

- Findings Freeze: frozen after Broad Audit（両方とも approve、共通の P3 は是正済み）; post-freeze exceptions: none.

### Plan Review（2026-09-25〜26、Claude 側 = fresh Opus 5.5、Codex 側 = GPT-6 Sol high、互いに独立）

- round 1（`ca280c79`）: Claude reject（P2 2 / P3 3）、Codex reject（P1 1 / P2 1、Ordinary Operation に反例）。主な是正: 中間を i128 にし最後だけ i64 へ検査付き変換・mutant を段ごとに分け kill 入力を Rust の試作で確認・時点証拠 ADR の旧式・Ordinary Operation を (a) 本 lane で成立する列と (b) ⑤ の後の列に分ける・負の検査を関数の 1 か所に。是正前に Fable 5.1 が裁定案の反例を探した。
- round 2（`91bb8618`）: Claude approve（P3 4）、Codex reject（P2 1 = 2^53 を超える円額の JS 表示 → ⑤ への申し送り）。是正 = `325861c5`。
- round 3（`325861c5`、上限）: Codex reject（P2 1 = 評価数量の記述の食い違い、P3 1）。同型の一括是正と owner 承認の予算改定で plan-approved。

### Final Review（2026-09-26、互いに独立）

- broad Codex 側（GPT-6 Astra）on `f6914e22`: approve、P1 0 / P2 0 / P3 1（Plans.md の登録行の古い文言）。AC1〜AC10・local-ci full PASS、mutant 8/8 red。[review](https://github.com/higekiri-w/inventory-system-desktop/pull/108#pullrequestreview-5320720846)
- broad Claude 側（Fable 5.1、fresh、read-only）on `f6914e22`: approve、P1 0 / P2 0 / P3 1（同じ）。python の Decimal の oracle と整数式を fuzz 20,000 件で不一致 0、pcs だけの総額 = 旧式も 20,000 件で不一致 0、mutant 8/8 red。
- 裁定: P3 accept → main 同期の後の `08a5cde4` で是正。あわせて遷移記録の key の重複（helper が `duplicate packet fields` で packet を読めない）を解消し、Gated Amendment 1 として登録（`3eb982ca`）。
- closure Codex 側（GPT-6 Astra）on `3eb982ca`: approve、P1/P2/P3 = 0。実装の差分は main 側と一致、packet は指定の 3 点だけ、P3 は解消。[review](https://github.com/higekiri-w/inventory-system-desktop/pull/108#pullrequestreview-5320804744)
- helper の record: review broad 2 本（pass）。owner の指示（2026-09-26「Codex 通してレビューも通ってるならマージや締めまで済ませて OK」）で helper `ready` → hosted CI 成功 → helper `merge`。
