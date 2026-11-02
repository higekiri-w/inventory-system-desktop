## 21. BIZ-07: 整合性チェックロジック

### 計数の置き場と在庫の求め直しの契約（D-118。(a) で実装済み）

[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)（SPEC-STK-CNT-D2）と D-051 の Superseded in part（D-118）。この節の全部が (a)（ADR SPEC-STK-CNT-D12）で入り、§21.2〜§21.7 を同じ契約に直した（b1 で、`derive_stock` が精算の窓を持つことと、最後の項目の共有 JAN の違反の一覧を足した。ほかの項目の形は変わらない）。

- `run_integrity_check` は、各商品の写し（`stock_quantity`・`stock_recheck`・`stock_range_low`・`stock_range_high`）と、[31](31-biz-inventory-service.md) の `derive_stock`（`recompute_stock` と同じ規則で、写しを書かない読取りの関数）が求める値を比べ、違う商品を返す。写しは在庫を動かす全ての TX で求め直すので、違いは不具合か DB の外からの編集でだけ起きる。
- 比べる項目は 4 つ（数 `stock_quantity`、印 `stock_recheck`、範囲 `stock_range_low`・`stock_range_high`）で、どれか 1 つでも違えば不一致とする。`IntegrityMismatch` は、写しの 4 項目（`copy`）と求めた 4 項目（`derived`）と、違う項目の一覧（`differing_fields`: `quantity` / `recheck` / `range` の generated enum）を持つ。数が同じで印や範囲だけ違う商品も一覧に出る（「差異なし」と出さない）。
- `fix_integrity` は、違う商品について `recompute_stock` の結果を写しへ書く（movement を作らない）。`StockAdjustment` と同じ TX の `integrity_fix` 操作ログ（D-051 の Audit）は、4 項目の old / new を持つ。確認の操作は変えない。画面の表し方は [75](75-ui-integrity-check.md) の新契約。
- 計数は movement を作らないので、「SUM と一致」は計数のある商品では成り立たない。突合の相手を SUM にしない。
- 整合性の補正を計数の証拠にしない（計数を作らない）。
- 型の形（D-118 の runtime の (a)）: 写しの 4 項目を 1 つの型 `StockCopy`（`stock_quantity: i64`、`stock_recheck: bool`、`stock_range_low: Option<i64>`、`stock_range_high: Option<i64>`）にまとめ、`IntegrityMismatch` は `product_code`・`name`・`stock_unit`・`copy: StockCopy`（写し）・`derived: StockCopy`（求めた値）・`differing_fields: Vec<IntegrityField>`（`quantity` / `recheck` / `range` の generated enum。重複なし、この順）を持つ。`StockAdjustment` は `product_code`・`stock_unit`・`before: StockCopy`・`after: StockCopy` を持つ（前の `old_stock`・`new_stock`・`adjustment` を置き換えた。差は持たず、画面は `before` → `after` を並べる。[75](75-ui-integrity-check.md) §75.6）。`integrity_check` と `integrity_fix` の操作ログの detail_json の各要素も同じ key と入れ子で書く。key の無い記録済みの log（`old_stock`・`new_stock`・`adjustment` の形）は読み手がそのまま読む（[74](74-ui-operation-logs.md)）。
- 移行の知らせ（ADR SPEC-STK-CNT-D10）: (a) の migration（[22 §19](22-mnt-migration.md)）が写しを直したときは、`integrity_fix` の操作ログの detail_json に `"trigger": "migration"` を足す（`fix_integrity` が書く行には `trigger` を書かない）。整合性の画面がこの行を読んで知らせる（[75](75-ui-integrity-check.md)）。
- 共有 JAN の違反の一覧（ADR SPEC-STK-CNT-D10 の最後の文・D4。b1 で実装済み）: `run_integrity_check` は、在庫連動（`pos_stock_sync=1`）の商品と同じ `jan_code` を持つ別の商品がある組を今の DB から求め、`IntegrityResult.shared_jan_conflicts`（要素は `jan_code` と、その JAN を持つ全ての商品の `product_code` の一覧〈商品コードの昇順〉。JAN の昇順）で返す。BIZ-01 の不変条件（[30](30-biz-product-service.md)）で新しい組は作られないので、移行の前からある組だけが出て、利用者が JAN か在庫連動を直すと消える。`fix_integrity` の対象にしない（自動では変えない）。写しの比較（`mismatches`）とは別の結果で、組があっても写しの比較の結果は変えない。`integrity_check` の操作ログは今の形のまま（組を記録しない）。

### 21.1 モジュール構成

```
src-tauri/src/
  biz/
    integrity_service.rs  -- 在庫整合性チェック・補正
```

### 21.2 型定義

型の正本は `src-tauri/src/biz/integrity_service.rs`（`specta::Type`。bindings は `src/lib/bindings.ts` に生成）。

**IntegrityResult構造体**:

```
struct IntegrityResult {
    mismatches: Vec<IntegrityMismatch>,  // 不整合商品リスト
    mismatch_count: usize,              // 不整合件数
    checked_count: usize,               // チェック対象商品数
}
```

**StockCopy構造体**（写しの 4 項目。保存している値にも、記録から求め直した値にも使う）:

```
struct StockCopy {
    stock_quantity: i64,
    stock_recheck: bool,             // 要再確認なら true
    stock_range_low: Option<i64>,    // stock_recheck が true なら Some（DB の CHECK で印と対）
    stock_range_high: Option<i64>,
}
```

**IntegrityField列挙**（generated enum は snake_case: `quantity` / `recheck` / `range`）:

```
enum IntegrityField { Quantity, Recheck, Range }
```

**IntegrityMismatch構造体**:

```
struct IntegrityMismatch {
    product_code: String,
    name: String,
    stock_unit: ProductStockUnit,          // 商品の単位（長さの商品は m で出す）
    copy: StockCopy,                       // products の写し（保存している値）
    derived: StockCopy,                    // derive_stock の値（記録から求め直した値）
    differing_fields: Vec<IntegrityField>, // 違う項目。重複なし、quantity・recheck・range の順
}
```

**IntegrityFixResult構造体**:

```
struct IntegrityFixResult {
    fixed_count: usize,             // 補正した商品数
    skipped_count: usize,           // 写しと求めた値が同じ（補正不要）か商品が無くてスキップした商品数
    adjustments: Vec<StockAdjustment>, // 補正内容の詳細
}
```

**StockAdjustment構造体**:

```
struct StockAdjustment {
    product_code: String,
    stock_unit: ProductStockUnit,
    before: StockCopy,      // 補正前の写し
    after: StockCopy,       // 補正後の写し（= recompute_stock の結果）
}
```

単位と範囲（D-113）: `IntegrityMismatch` と `StockAdjustment` は `stock_unit: ProductStockUnit` を持ち（`find_all_stock_quantities` が写しの 4 項目と単位・名前を返す）、`integrity_fix` の detail_json の各 adjustment と、`integrity_check` の detail_json の各 mismatch にも `stock_unit` を書く（`integrity_check` の log の表示は今のまま）。差は保存しない（画面も差を出さず、保存している値 → 記録から求め直した値を並べる。[75](75-ui-integrity-check.md) §75.6）。`fix_integrity` は、求め直した在庫か範囲の端の絶対値が在庫の上限 `999999999` を越えれば、`recompute_stock` が写しを書く前に `ValidationFailed`（`在庫数が大きすぎます`）で補正の TX 全体を戻す（その商品を対象から外せば、ほかの商品は補正できる）。`run_integrity_check` の読取りは範囲で拒まない（上限の外の値は DB を直接書き換えたか壊れたときだけ。共通規則の「入力の上限と安全な整数の範囲」の (b)(d)(e)）。長さの商品の数量を m で出すため（[共通規則](10-common-rules.md) SPEC-UNIT-D3 の「表示する所」の表の 8・11・12 行目、SPEC-UNIT-D11）。

---

### 21.3 run_integrity_check

**関数要求**: 全商品の写し（products の 4 項目）と、記録から求め直した値（[31](31-biz-inventory-service.md) の `derive_stock`）を突合し、違う商品を検出する。読み取り専用で、在庫のデータの変更は行わない

**シグネチャ**:
```
fn run_integrity_check(conn: &DbConnection) -> Result<IntegrityResult, BizError>
```

**前提条件**:
- conn は &DbConnection（autocommit）。読み取り専用クエリのためTX不要
- 写しは在庫を動かす全ての TX で `recompute_stock` が書く（[31](31-biz-inventory-service.md)）。違いは不具合か DB の外からの編集でだけ起きる

**処理ステップ**:

1. **全商品の写しを取得**
   - product_repo::find_all_stock_quantities(conn) → 商品ごとの `product_code`・`name`・`stock_unit` と写しの 4 項目
2. **突合**
   - 各商品について `derive_stock(conn, product_code)` を求め、`StockCopy` の形にして写しと比べる
   - 違う項目（`differing_fields`）が 1 つ以上 → mismatches に追加（数が同じで印か範囲だけが違う商品も出す）
3. **結果返却**
   - IntegrityResult { mismatches, mismatch_count: mismatches.len(), checked_count: 全商品数 }
4. **TX外: 操作ログ記録**
   - system_repo::insert_operation_log(conn, &NewOperationLog { operation_type: "integrity_check", summary: "整合性チェック実行: {checked_count}件中{mismatch_count}件の不整合", detail_json: Some(`{"checked_count","mismatch_count","mismatches":[{"product_code","stock_unit","copy","derived","differing_fields"}]}`) })。失敗は `tracing::warn!` で記録し、結果は返す

**エラーハンドリング**:
- DB読み取り失敗 → BizError::DatabaseError(DbError)
- 求め直しが `phase = 1` の movement を読んだ → BizError::DatabaseError（(a) の間の契約。[31](31-biz-inventory-service.md)）

**設計判断 — 記録の無い商品**:
- 計数も movement も無い商品の求め直しの値は 0（印 0・範囲なし）。写しが 0 なら不整合なし、0 でなければ不整合（初期投入が漏れている可能性）

**設計判断 — 突合の相手を SUM にしない**:
- 計数（`stock_counts`）は movement を作らないので、計数のある商品では写しが movement の和と一致しない。突合の相手は写しを書くのと同じ規則の `derive_stock` にする。計数が無い商品では、求め直しの値は `is_voided = 0` の movement の和と同じ

---

### 21.4 fix_integrity

**関数要求**: 指定された商品の写しを、記録から求め直した値に合わせて**直接更新**で補正する（`recompute_stock` の結果を書く）。補正は実入出庫・棚卸しではないため inventory_movements に行を追加しない（BIZ-07-D2）。補正内容は同一 TX 内の `integrity_fix` 操作ログへ before / after 付きで必須記録する（BIZ-07-D3）。意味論の durable 判断は [D-051](../decision-log.md)（D-118 で Superseded in part）

**シグネチャ**:
```
fn fix_integrity(
    conn: &mut DbConnection,
    product_codes: &[String],
) -> Result<IntegrityFixResult, BizError>
```

**前提条件**:
- conn は &mut DbConnection（TX制御）
- 利用者が run_integrity_check の結果を確認し、補正対象を選択した後に呼ばれる
- 自動実行しない。必ず利用者確認を挟む（architecture/biz-task-specs.md BIZ-07「制御構造」）。移行（[22 §19](22-mnt-migration.md) の手順 8）が写しを直すのは、この関数でなく migration の同じ TX の中（`trigger: migration` の行）

**処理ステップ**:

1. **入力バリデーション**
   - product_codes が空 → BizError::ValidationFailed("補正対象の商品が指定されていません")
   - この空入力条件はBIZ-07だけが所有する（**BIZ-07-VAL-D1**）
2. **TX開始**（conn.transaction()）
3. **各商品について補正**
   - TX の中で全商品の写しを `product_repo::find_all_stock_quantities` で読み、product_codes の各 product_code について:
   a. 商品が存在しない → スキップ（skipped_count++）
   b. 写し（before）と `derive_stock` の値が同じ → スキップ（skipped_count++）
   c. 違う → `recompute_stock(&tx, product_code)` で写しを書き（BIZ-07-D2: direct update のみ）、その結果を after として adjustments に追加（product_code / stock_unit / before / after）
      - **movement 行は追加しない**。補正 movement を挿入すると記録そのものが変わり、再チェックが収束しない（監査 P7-1 の算術、[D-051](../decision-log.md)）
4. **TX内: 操作ログ記録（必須、BIZ-07-D3）**
   - system_repo::insert_operation_log(&tx, &NewOperationLog { operation_type: "integrity_fix", summary: "{fixed_count}件の在庫を補正しました", detail_json: Some(`{"fixed_count","skipped_count","adjustments":[{"product_code","stock_unit","before","after"}]}`。`trigger` は書かない) })
   - ログ記録失敗は補正ごと rollback して BizError::DatabaseError を返す。**旧・第4段階先決事項D-6（操作ログは best-effort）の明示例外**: 補正は movement 行を残さないため操作ログが唯一の監査痕跡であり、痕跡なしの在庫直接書換えを構造的に禁止する（TX 内必須ログの既存 precedent は BIZ-01 product_service の商品更新系 + BIZ-02 inventory_service の業務記録 4 系。TX 境界の現状整理は D-051 audit 小見出し参照）
5. **COMMIT**（tx.commit()）
6. **結果返却**
   - IntegrityFixResult { fixed_count, skipped_count, adjustments }

**エラーハンドリング**:
- 補正対象なし → BizError::ValidationFailed(メッセージ)
- 求め直しの値か範囲の端が在庫の上限の外 → BizError::ValidationFailed("在庫数が大きすぎます")（TX 全体を戻す）
- DB更新失敗 → BizError::DatabaseError(DbError)（TX自動ロールバック）
- 操作ログ記録失敗 → BizError::DatabaseError(DbError)（TXロールバック、補正は一切確定しない — BIZ-07-D3）

**設計判断 — 「仮想棚卸し」概念の退役（BIZ-07-D5、旧・先決事項D-3の撤回）**:
- 旧設計は補正を reference_type='stocktake', reference_id=0 の「仮想棚卸し」movement として記録するとしていたが、movement を作らない意味論（BIZ-07-D2 / D-051）では識別設計そのものが不要になった
- 補正の識別は operation_logs の operation_type='integrity_fix' が担う
- このため補正は在庫変動履歴（[65-inventory-record-traceability.md](65-inventory-record-traceability.md) の追跡対象）に現れず、追跡は operation_logs（UI-11c 操作ログ画面）で行う

**設計判断 — 記録から求め直した値に合わせる方向（BIZ-07-D1）**:
- 記録（inventory_movements と stock_counts）が在庫の**原本**、products の写しは記録から求め直す**派生値**（ADR SPEC-STK-CNT-D2。D-051 の「movements の和」を D-118 で置き換えた）。補正は写しを原本から求め直した値へ合わせる操作であり、原本には触れない

---

### 21.5 非目的

このモジュールが**やらないこと**を明示する。責務境界の誤解を防ぐため。

| やらないこと | 理由 | 責務を持つモジュール |
|------------|------|-----------------|
| 棚卸し確定時の自動トリガー | PR-5 で BIZ-06 に統合（先決事項D-2） | BIZ-06 complete_stocktake |
| CSV取込み完了時の自動トリガー | UI層から CMD 経由で呼び出し（先決事項D-2） | CMD-11 / UI |
| 利用者確認なしの補正 | 危険。必ず確認を挟む（移行の手順 8 は migration の TX の中で直し、操作ログと整合性の画面で知らせる） | — |
| 計数の作成 | 整合性の補正を計数の証拠にしない | — |
| inventory_movements の物理DELETE | 論理無効化のみ（INV-4） | — |
| products の物理DELETE | INV-8 禁止 | — |

### 21.6 対応不変条件

| 不変条件 | 本モジュールでの対応 |
|---------|-----------------|
| BIZ-07-D1: 原本/写し | 記録（inventory_movements・stock_counts）= 原本、products の写しの 4 項目 = 派生値（D-118）。補正は写しを `recompute_stock` で求め直した値へ直接更新し、原本には触れない |
| BIZ-07-D4: 収束性 | fix_integrity 成功直後（介在 write なし）の run_integrity_check の mismatches に補正対象商品が現れない（補正の後の写しは `derive_stock` と同じ値で、同関数は違う項目のある商品だけを返す） |
| INV-2: stock_after算出責任 | fix_integrity は movement を作らないため stock_after の算出自体が発生しない（apply_stock_change 非経由） |
| INV-3: 負在庫ポリシー | 求め直した値がマイナスになる場合でも補正を実行（記録が真値） |
| INV-4: is_voided の使用範囲 | 求め直しは `is_voided = 0` の movement だけを読む。fix_integrity は is_voided を操作しない |
| INV-8: products物理DELETE禁止 | 写しの更新のみ。DELETE なし |

### 21.7 テスト方針（完了条件）

以下の oracle を持つテストを完了条件とする（`src-tauri/src/biz/integrity_service.rs` の test。D-118 の (a) の packet の Test Matrix I1〜I6 を含む）:

- **突合の相手（I1〜I3）**: 計数のある商品で写し = 求めた値なら不一致に出ない、違えば出る。数が同じで印か範囲だけが違う商品が `differing_fields` 付きで出る。項目は重複なし・`quantity`・`recheck`・`range` の順
- **失敗系（BIZ-07-D3）**: `integrity_fix` 操作ログの INSERT を注入失敗させ（SQLite trigger 等）、`BizError::DatabaseError` の返却、全対象商品の写し不変、inventory_movements 行数増 0 を assert する
- **成功系（BIZ-07-D3 / D2、I4・I5）**: detail_json.adjustments[] の product_code / stock_unit / before / after（それぞれ 4 項目）を具体値で検証し、`trigger` が無いことを確かめる。あわせて**補正前後で inventory_movements の総行数と各対象商品の行数が不変**（新規 movement row 不存在）を assert する — D-051 rejected ②（quantity=0 marker 行）の実装を green のまま通さないため（Contract Audit 2 pass P1-2）
- **check のログの形（I6）**: `integrity_check` の detail の各 mismatch が `copy`・`derived`・`differing_fields` を持つ
- **収束系（BIZ-07-D4）**: 補正成功直後（介在 write なし）に run_integrity_check を実行し、adjustments[].product_code が result.mismatches に存在しないことを assert する。補助 oracle として同一 committed state で写しと `derive_stock` の値が同じことを対象商品ごとに検査する（計数の無い商品では `stock_quantity = SUM(quantity WHERE is_voided = 0)` と同じ）
