# ドキュメント書き方ガイド

> **目的**: 設計ドキュメントの書き方ルールを一元管理し、記述のブレを防ぐ
> **対象**: docs/ 配下の全設計書、src-tauri/src/ 内の doc comment（§1 の箇所の引き方は `src/` の comment と backlog の申し送りも対象）
> **自動検証**: `scripts/doc-consistency-check.sh` で機械チェック可能な項目には ✅ を付記

---

## 0. ドキュメント階層と命名規約

### エントリーポイントパターン

親文書が**索引 + 設計方針**、サブ文書が**詳細定義**を持つ2層構造。

```
ARCHITECTURE.md（索引）→ docs/architecture/{layer}-task-specs.md（詳細）
FUNCTION_DESIGN.md（索引）→ docs/function-design/{NN}-{layer}-{module}.md（詳細）
DB_DESIGN.md（索引）→ docs/db-design/{category}-tables.md（詳細）
design-system/README.md（索引）→ design-system/{NN}-{topic}.md（詳細）
```

### サブファイル命名規約

| ディレクトリ | 命名パターン | 例 |
|------------|------------|---|
| `function-design/` | `{NN}-{layer}-{module}.md` | `20-io-product-repo.md`, `31-biz-inventory-service.md` |
| `db-design/` | `{category}-tables.md` | `master-tables.md`, `pos-tables.md` |
| `architecture/` | `{layer}-task-specs.md` | `io-task-specs.md`, `cmd-task-specs.md` |
| `design-system/` | `{NN}-{topic}.md` | `00-foundations.md`, `02-component-catalog.md` |

番号の先頭2桁はレイヤーを示す: `1x`=共通, `2x`=IO層, `3x`=BIZ層, `4x`=CMD層, `5x`=UI層, `7x`=MNT層, `0x`=デザイン基盤

---

## 1. 参照の書き方

### コード内 doc comment（Rust） ✅R0 ✅R1

**サブファイルを直接参照する。親文書名（FUNCTION_DESIGN.md 等）は使わない。**

```rust
// Good: サブファイルを直接参照
/// 20-io-product-repo.md §2.3 find_by_product_code
//! 21-io-inventory-repo.md §2.7 に基づく実装。
/// db-design/tracking-system-tables.md: inventory_movements.movement_type CHECK制約に対応

// Bad: 親文書を経由する旧形式（R0 で検出される）
/// FUNCTION_DESIGN.md セクション2.3 find_by_product_code
/// DB_DESIGN.md セクション14
```

- セクション記号は `§` を使用（`セクション` は旧形式）
- 構造体の doc comment にはテーブル定義の参照先を記載: `/// db-design/{file}.md {テーブル名}`

### Markdown リンク ✅R3

```markdown
<!-- Good: 相対パスでリンク先ファイルが実在する -->
[IO-01: 商品リポジトリ](function-design/20-io-product-repo.md)

<!-- Bad: リンク先が存在しない（R3 で検出される） -->
[IO-01: 商品リポジトリ](function-design/20-io-product.md)
```

### 親文書リンク（サブドキュメント冒頭）

全てのサブドキュメントの冒頭に親文書への参照を記載する。

```markdown
> **親文書**: [ARCHITECTURE.md](../ARCHITECTURE.md)
> **入力ドキュメント**: `docs/spec/requirements.md`、`docs/spec/requirements-coverage.md`、DB_DESIGN.md
```

### タスクID・要件ID

- **タスクID**: `BIZ-01`, `IO-02`, `CMD-07` など ARCHITECTURE.md で定義されたID
- **要件ID**: `REQ-NNN` 形式
- テスト関数名に REQ 番号を含める: `test_create_product_req101_normal`
- テスト内にコメントで仕様ID記載: `// REQ-101-01: JANコード入力で商品登録`

### code と文書の箇所の引き方（行番号で引かない）

設計の正本（`docs/` の設計書・workflow の正本・runbook）と code の comment（`src/`・`src-tauri/src/`）は、箇所を行番号（`path:N`・`` `:N` ``・`#LN`）で引かない。行番号は code や文書の変更で動き、読み手を別の箇所へ導く（D-125）。

- code は repo root からの path と symbol（関数・型・定数・test の名前）で引く。範囲は symbol と、その中の目印の語で書く
- 文書は節の ID（`§12.3`・`BIZ-01-D8`・`DSR-22` 等）か節の見出しで引く
- `docs/backlog.md` の申し送りで行番号を使うときは、書いた時点の main の commit を同じ行に添える（例: 「`18db621e` の時点で `src-tauri/src/biz/product_service.rs` の 965 行」）。`docs/Plans.md` は導入の前の関門・ブロッカー・製品の未決判断を持つ所で、行番号の申し送りを置かない
- 時点を書いた記録（commit・library の版・日付を書いた観測、dated の調査・提案・L3 の記録、decision-log の append-only の本文）の行番号は、その時点の観測としてそのまま残す
- 対象外: dated の Plan Packet と Test Design Matrix（起票の時点の観測）、data の行番号（CSV・EJ の `line_no` 等の製品の契約）、review の finding と subagent の報告の `path:line`（その時点の版に対する一時の出力。[code_review.md](code_review.md)、[DEV_WORKFLOW.md](DEV_WORKFLOW.md) の Subagent Budget）

---

## 2. 関数設計書テンプレート（function-design/） ✅M2

### ファイル構成

```markdown
> **親文書**: [FUNCTION_DESIGN.md](../FUNCTION_DESIGN.md)
> **入力ドキュメント**: ARCHITECTURE.md（タスク仕様）、DB_DESIGN.md（テーブル定義）

## NN. {タスクID}: {タスク名}

### モジュール構成
（ファイルツリー）

### 型定義
（struct / enum）

### N.M {関数名}

**関数要求**: （一文で「何をするか」と「なぜ必要か」）

**シグネチャ**:
fn name(args) -> Result<T, E>

**前提条件**: （ステートレス、DB接続不要 等）

**処理ステップ**:
1. ステップ1
2. ステップ2
   2a. サブステップ

**エラーハンドリング**:
- 条件 → Err(エラー型::バリアント)

### N.M+1 非目的（IO層で推奨）

| やらないこと | 理由 | 責務を持つモジュール |
|------------|------|-----------------|

### 更新履歴

| 日付 | PR | 内容 |
|------|-----|------|
```

### 必須項目（M2 テンプレートチェック対象）

| レイヤー | 必須セクション |
|---------|-------------|
| BIZ/IO層 | 関数要求 + シグネチャ + 処理ステップ + エラー |
| CMD層 | シグネチャ + 処理ステップ |
| UI層 | 処理ステップ or コンポーネント構造 or 画面 |

### 推奨項目

- **設計判断**: 「なぜこの設計か」「検討した代替案」
- **入出力例**: JSON または疑似コード
- **対応不変条件**: INV-N との対応

---

## 3. テーブル定義書テンプレート（db-design/）

```markdown
> **親文書**: [DB_DESIGN.md](../DB_DESIGN.md)

## N. {テーブル名}

### 役割
（1-2文で存在意義を明示）

### カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|------|------|

### 設計意図
- FK構成、INDEX戦略、NULLABLE条件の理由

### 困りそうなケースと対応方針
（運用時の例外パターンと対処方法）

### 業務シナリオ例
（初期化 → 変更 → 派生のデータフロー）
```

---

## 4. タスク仕様テンプレート（architecture/）

```markdown
> **親文書**: [ARCHITECTURE.md](../ARCHITECTURE.md)

### {タスクID}: {タスク名}

**タスク要求**: （責務の一文定義）

**【データ構造】**
（入出力の型定義）

**【処理構造】**
（ステップ表記）

**【制御構造】**
（エラーハンドリング、トランザクション、排他制御）
```

---

## 5. 禁止事項

### 曖昧表現 ✅M1

以下の表現は設計書本文で使用禁止（コードブロック・テーブル内は除外）:

`適切に` / `TBD` / `など。`（文末の「など。」）/ `必要に応じて` / `適宜`

### 未確定マーカー ✅M3

以下のマーカーは解決してから本文に残さない（取消線 `~~TODO~~` は許容）:

`TODO` / `FIXME` / `HACK` / `未確定` / `TBD`

### レイヤー境界違反 ✅チェック6

- IO層のドキュメントに `BizError` / `CmdError` を記載しない
- CMD層のドキュメントに `DbError` を記載しない

### 旧形式参照 ✅R0

- `FUNCTION_DESIGN.md セクション` — サブファイル直接参照に変更
- `FUNCTION_DESIGN.md §` — 同上
- `DB_DESIGN.md セクション` — `db-design/*.md` を参照

---

## 6. 自動チェック（doc-consistency-check.sh）

`scripts/doc-consistency-check.sh` は inventory design checks と active Plan Packet checks を実行する。**このガイドに従えばチェックは通る。**

| カテゴリ | ID | チェック内容 |
|---------|-----|-----------|
| 用語・型 | 1-8 | CSV/TSV統一、エラー型整合、レイヤー境界、TX境界 等 |
| 適合性 | C1 | DBスキーマ参照（table.column が DB_DESIGN.md に実在） |
| 適合性 | C2 | 関数シグネチャ（呼び出し先が function-design に定義済み） |
| 追跡性 | H1-H3 | REQトレーサビリティ、INV参照漏れ、エラーバリアント網羅 |
| 保守性 | M1-M3 | 曖昧表現、テンプレート準拠、未確定マーカー |
| **参照整合** | **R0** | **親文書の直接参照がコード内に残っていないか** |
| **参照整合** | **R1** | **コード内 docs/ パス参照が実在するか** |
| **参照整合** | **R3** | **Markdownリンク先ファイルが実在するか** |
| **Plan 構造** | **PK1** | **`docs/plans/` 直下の dated active plan が Risk 行と必須セクションを持つか** |
| **Plan 内容** | **PK2** | **R2+ の packet（新しい packet は R3/R4）に未編集 placeholder / 空 bullet が残っていないか** |
| **Plan 警告** | **PK3** | **R3/R4 plan の Contract Ledger（旧 Trace Matrix）と Acceptance evidence を warning で見える化する** |

実行方法:
```bash
./scripts/doc-consistency-check.sh                       # 設計書チェック + active Plan Packet チェック
./scripts/doc-consistency-check.sh --target plan [file]  # プランチェック + active Plan Packet チェック
```
