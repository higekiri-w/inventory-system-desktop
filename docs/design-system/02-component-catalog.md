# コンポーネントカタログ（16 パターン）

> **親文書**: [README.md](README.md)
> **責務**: 繰り返し使われる 16 パターンの canonical 定義。各パターンに使いどころ・JSX skeleton・使用トークン・全状態・a11y 要件・Do-Don't・canonical ファイル参照を記載する。
> **現行と狙い**: 本書は現行実装の canonical である。[00-foundations.md](00-foundations.md) の色の役割表と迷いやすい場面の「移行」列が runtime lane 待ちとする項目は、00 / 04 / 01 を狙いとして読み、本書を現行として読む（[README.md「移行中の読み方」](README.md#移行中の読み方)）。

---

## テンプレートの読み方

各パターンは以下の統一形式で記述する。

- **使いどころ**: いつこのパターンを選ぶか（1-2 文）
- **canonical**: 規範となる実装ファイル。新規実装はここを一次参照にする
- **構造**: 実装から要点を抽出した JSX skeleton。属性は規範部分のみ残し、省略箇所は `…` で示す
- **使用トークン**: 色・スペーシング・タイポを [00-foundations.md](00-foundations.md) の語彙で示す
- **状態**: hover / focus / active / disabled / error の規定。当てはまらない状態は「規定なし」と明記する
- **アクセシビリティ**: label 関連付け・role・非色シグナル
- **Do / Don't**: 守ること・避けること

skeleton の例示文言・コードはすべて合成データ（架空の商品名・コード）で書く。

---

## ① ページヘッダ

**使いどころ**: 各ページ最上部のタイトル行。画面名（h1）と、その画面の主動線アクション（あれば 1 個）を横並びに置く。

**canonical**: `src/components/patterns/PageHeader.tsx`（`PageHeader{title, subtitle?, description?, actions?}` の 4 variant）。適用例: `src/features/products/ProductListPage.tsx`（h1 + 主動線 actions）、`src/features/home/HomePage.tsx`（h1 + subtitle）

**構造**:

```tsx
<header className="space-y-1">
  <div className="flex flex-wrap items-start justify-between gap-3">
    <h1 className="min-w-0 flex-1 text-2xl font-semibold">商品検索・一覧</h1>
    <div className="shrink-0">
      <Button type="button" asChild>
        <Link to="/products/new" search={{ returnTo }}>
          <PackagePlus aria-hidden="true" />
          商品登録
        </Link>
      </Button>
    </div>
  </div>
  {subtitle !== undefined && <p className="text-sm text-muted-foreground">{subtitle}</p>}
  {description !== undefined && <p className="text-sm text-muted-foreground">{description}</p>}
</header>
```

主動線が無い画面（例: 商品登録・修正）は `<header className="space-y-1">` に h1 のみを置き、右側のアクションを省く。

**バリエーション: 説明セクション**（UI 磨き batch 3 design、L8-7/L8-8）: タイトル直下に 2〜3 文の操作説明が要る画面（商品一括インポート・PLU書出し・バックアップ復元等）は、`text-sm text-muted-foreground` の `<p>` を見出し行の下に**説明行**として全幅で描画する（**見出し行 + 説明行** の 2 段）。`subtitle` prop（1 行の短い副題、例: ホームの日付）とは用途を分け、説明セクションは複数文の操作説明に使う。`src/features/suppliers/SupplierManagementPage.tsx` の `PageHeader` の説明文は、本 PR（⑮）で外側 sibling `<p>` から `subtitle` prop へ移行済み。

**component gap の解消**: `actions` と `subtitle`/`description` の排他は本 PR（⑮）で解消済み。⑮ 時点では `PageHeader` の `actions` 分岐で `<h1>` + 条件付き `subtitle`/`description` を左 group の `<div className="min-w-0 flex-1 space-y-1">` にまとめた。⑮ Gated Amendment 2（owner L3 round 1）で外側 `<header>` を `flex flex-wrap items-start justify-between gap-3`、actions wrapper を `shrink-0` に変更した。⑮ 時点では長い description で actions が次行左へ折り返す問題に対し、説明を左列内で折り返し、actions を右上に留める構造を採っていた。`src/features/receiving/ReceivingPage.tsx` / `src/features/manual-sale/ManualSalePage.tsx` / `src/features/return-exchange/ReturnExchangePage.tsx` / `src/features/disposal/DisposalPage.tsx` の `PageHeader` の副題と、`src/features/suppliers/SupplierManagementPage.tsx` の `PageHeader` の `subtitle` への移行により、見出しと説明を同じグループで描画した。actions なしでも説明を描画する（`src/components/patterns/PageHeader.tsx` の `PageHeader` の `actions` なしの分岐）。呼び出し側で wrapper を都度書く使用パターンは不要になった。 **本 PR（⑳ Gated Amendment 2）で見出し行 + 説明行の 2 段配置へ改め、`items-start`（h1 と actions の上端揃え）/ canonical path / props は不変**。**runtime 反映済み（PR #63、`src/components/patterns/PageHeader.tsx` の `PageHeader` の `actions` の分岐）**。

**バリエーション: 詳細ルートの戻る導線**（PR #114-#115）: read-only の記録詳細ルート（`src/features/inventory-records/ReturnRecordDetailPage.tsx` ほか入出庫 4 詳細ページ）は、actions に「前の画面へ戻る」ボタン（outline）を置く。データ取得失敗時も PageHeader + 戻るボタンは表示したままにし、エラー Alert だけで終わらせない（利用者を行き止まりにしない）。戻り先の `returnTo` param は [01-decision-rules.md](01-decision-rules.md) DSR-15 の検証を通してから使う。

**バリエーション: セクション見出し（h2）**: page 内の sub-section（「直近の○○」テーブル / 公式部門集計 / 差異のある商品 / 棚卸し進捗 等）の見出しは h2（**`text-xl font-semibold`、④ / 00-foundations の h2 20px と同一 token**）+ 任意の右要素（Button / Badge）+ 任意の説明 `<p className="text-sm text-muted-foreground">` で構成する。配置は **見出し行 + 説明行** の 2 段: 見出し行は `flex flex-wrap items-start justify-between gap-3` に h2（`min-w-0 flex-1`。長い見出しは行内で折り返し、右要素を次行へ落とさない）と右要素（`shrink-0`）を置き、説明は見出し行の**下**に全幅で置く（外側 `space-y-1`）。説明を見出しと同じ flex 行の左 group に置いて右要素の横で折り返す形（旧 差異のある商品 / 棚卸し進捗）は採らない。`PageHeader` (c) も同じ 2 段配置（① 構造）であり、page 見出しと sub-section 見出しの規範は 1 本。「直近の○○」系 4 画面（③ テーブルの **「直近の○○」系4画面の統一** 段落の文型）は既にこの配置であり不変。**1 ページ 1 h1 は不変**であり、sub-section を `PageHeader` で描かない。Dialog の `AlertDialogTitle` は対象外。適用は class / 文型 / token の統一とし、component 化（`SectionHeader{title, description?, actions?}`）は現時点で採用しない（3 箇所目の独自要件が出た時点で再検討）

**使用トークン**: h1 = タイポ `h1`（24px / weight 600）。アクションボタンは Primary（`--primary`、操作の塗り）。要素間ギャップは `space-3`（12px）。

**状態**: h1 自体に状態変化は規定なし。主動線ボタンは hover / focus / disabled をボタン primitive の既定に従う。

**アクセシビリティ**: 画面タイトルは 1 ページ 1 個の h1 とする（タイポグラフィ階層の起点）。装飾アイコンには `aria-hidden="true"` を付け、ボタンの可読ラベルは日本語テキストで持つ。

**Do**:
- 主動線（primary button）は 1 ページ 1 個に絞る（[01-decision-rules.md](01-decision-rules.md) DSR-01）
- アクションが複数あるときは 1 つだけ Primary、残りは 3 段（`secondary` 中間段 → `outline` / `ghost`）で降格する

**Don't**:
- h1 を複数置かない
- ヘッダーに業務データを密に詰めない（ヘッダーは余白を取る領域、[03-philosophy.md](03-philosophy.md) japanese-webdesign の適用境界）

---

## ② サマリカード

**使いどころ**: ダッシュボードやレポート上部で、1 指標を 1 カードで見せる。回復導線の置き方は **query 構成で 3 パターン**に分かれる（PR-B D-B1 で規約化）:

1. **独立 query のカード** = per-card retry 必須（カード内 Alert + 再試行）
2. **同一 query 共有カード群** = per-card retry を採用してよい（許容例: ホームの在庫切れ・在庫少 2 カードは共有 `lowStock` query × per-card retry）
3. **カードが束ねられ個別回復導線が冗長な場合** = page-level Alert + 再試行を許容（canonical variant: 日次/月次売上の `SummaryCardsBar` — 単一系 query が 4 カードを供給するため、カード単位 retry は同一 query の再実行ボタンを 4 個並べる UX 過剰になる）

いずれのパターンでも「取得失敗からの回復導線（再試行）」がカード内またはページ内に必ず存在することが要件。部分障害許容（1 カードの失敗が他カードを巻き込まない）はパターン 1/2 に適用し、パターン 3 は page-level Alert がカード群を差し替える。

**canonical**: `src/components/patterns/SummaryCard.tsx`（パターン 1/2 用の単一カード）。パターン 3 の variant は `src/features/daily-sales/components/SummaryCardsBar.tsx` / `src/features/monthly-sales/components/SummaryCardsBar.tsx`（タイトルごと Skeleton 化 + sub 行 + Tooltip を持つ集計バー構造のため、patterns/SummaryCard とは構造非互換 — 統合は prop 肥大化を招くため見送り）

**構造**:

```tsx
<Card>
  <CardHeader>
    <CardTitle className="text-sm font-medium text-muted-foreground">{title}</CardTitle>
  </CardHeader>
  <CardContent>
    {isLoading ? (
      (loadingSkeleton ?? <Skeleton className="h-8 w-32" />)
    ) : isError ? (
      <Alert variant="destructive">
        <AlertDescription className="flex items-center justify-between gap-2">
          <span>取得失敗</span>
          <Button variant="outline" size="sm" onClick={onRetry}>再試行</Button>
        </AlertDescription>
      </Alert>
    ) : (
      children
    )}
  </CardContent>
</Card>
```

**使用トークン**: カード背景 `--card`（`stone-100`）。タイトルは `caption` 相当（`text-sm` + `text-muted-foreground`）。エラー時の Alert は `destructive` variant。カード間ギャップは `space-4`（16px）。

**状態**:
- **error**: パターン 1/2 では `isError` のときカード内 Alert（destructive）+「再試行」ボタン。パターン 3 では page-level Alert + 再試行がカード群を差し替える（`isError` を渡す画面は `onRetry` を必ず併せて渡す）
- **loading**: `loadingSkeleton` 未指定なら `h-8 w-32` の Skeleton（金額系の既定）。パターン 3 variant はタイトルごと Skeleton 化する集計バー構造
- hover / focus / active / disabled: カード自体には規定なし

**アクセシビリティ**: タイトルは `CardTitle`（見出し相当）で構造化する。「取得失敗」は色だけでなく日本語テキストで示し、回復導線（再試行ボタン）をカード内またはページ内に併置する。

**Do**:
- 取得失敗からの回復導線（再試行）をカード内 or ページ内に必ず置く（3 パターンのいずれかに従う）
- 独立 query のカードは loading / error / data を独立判定する（部分障害許容）

**Don't**:
- 独立 query のカード群で、1 カードの失敗により他カードまで非表示にしない
- 失敗を色やアイコンだけで伝えない
- 同一系 query が供給するカード群に、同じ query を再実行するだけの retry ボタンを複数並べない

---

## ③ テーブル

**使いどころ**: 商品一覧・在庫一覧・売上明細といった行 × 列の業務データを一覧表示する。

**canonical**: `src/features/products/components/ProductTable.tsx`

**sortable header canonical（UI-TABLE-D1）**: 売上系tableのsortable headerは
`src/components/sales/SortableHeader.tsx` を唯一のimplementation ownerとする。
`SortableHeader<T extends string>` は
`column` / `label` / `sortBy` / `sortDir` / `onClick` / optional `align`
だけを受け取り、daily/monthly固有の`SortColumn`をimportしない。
consumerは日次`ProductTable`、月次`DepartmentTable`、
`ProductRankingTable`の3tableに限定する。

**構造**:

```tsx
<Table>
  <TableHeader>
    <TableRow>
      <TableHead>商品コード</TableHead>
      <TableHead>商品名</TableHead>
      <TableHead>部門</TableHead>
      <TableHead className="text-right">売価</TableHead>
      <TableHead className="text-right">在庫数</TableHead>
    </TableRow>
  </TableHeader>
  <TableBody>
    {items.map((item) => (
      <TableRow
        key={item.product_code}
        className={item.is_discontinued ? "text-muted-foreground" : undefined}
      >
        <TableCell className="font-mono text-sm font-medium">{item.product_code}</TableCell>
        <TableCell className="min-w-[14rem] whitespace-normal">
          <div className="flex flex-wrap items-center gap-2">
            <span className="font-medium">{item.name}</span>
            {item.is_discontinued && <Badge variant="secondary">廃番</Badge>}
            {/* 分類 badge は枠線必須（`--border`、DSR-22）が規範。現状 variant="secondary" は border なし、Lane 3〜5 sweep 対象 */}
          </div>
        </TableCell>
        <TableCell>{item.department_name}</TableCell>
        <TableCell className="text-right tabular-nums">{yen.format(item.selling_price)}</TableCell>
        <TableCell className="text-right tabular-nums">
          {item.stock_quantity.toLocaleString("ja-JP")} {item.stock_unit}
        </TableCell>
      </TableRow>
    ))}
  </TableBody>
</Table>
```

**使用トークン**: コード列は等幅 `font-mono`。数値列は `tabular-nums` + `text-right` で桁を揃える。廃番行は `text-muted-foreground`（`stone-500`）で減衰させる。商品名セルは `min-w-[14rem]` で最小幅を確保し折り返す。数量 + 単位の表示は共通 `formatStockDisplay`/`formatStockUnitLabel`（`src/features/stock-inquiry/lib/format-stock-display.ts`）を使い、`unit` コードを直接文字列結合しない（UI 磨き batch 3 design L8-1）。

**状態**:
- 行の減衰: 廃番など従的状態の行は `text-muted-foreground` で薄くする
- hover / focus / active: 行自体には規定なし（行クリックで詳細展開する場合はパターン⑫を併用し、その規定に従う）
- disabled / error: テーブルには規定なし（取得失敗はパターン⑥でテーブルごと差し替える）

**列の状態表現の使い分け**:
- 状態が一覧の主情報なら独立した「状態列」を置く（DSR-04）
- 行の識別が主で状態が従なら、商品名セル内の `Badge`（例: `廃番`）+ 行全体の `text-muted-foreground` で示す

**バリエーション: 元記録リンク列**（PR #113/#115、canonical: `src/features/stock-movements/components/MovementTable.tsx`）: 在庫変動明細は、業務記録の詳細ページが `MovementTable{movements, returnTo?}`、在庫変動履歴が `StockHistoryTable`（D-118 の b2。movement と計数の行）を使い、元記録の欄（`MovementSourceLink`）と増減の欄（`MovementQuantity`）を `MovementTable.tsx` から共有する。`source`（`{ label, route } | null`。計数の行は `count.source`）が `null` なら「元記録なし」を表示し、値があれば returnTo 付きリンクで元業務記録の詳細へ遷移する。増減は矢印アイコン + 符号付き数値で示し、色のみに依存しない（DSR-08）。

**バリエーション: 直近実績サマリテーブル**（PR #116）: 業務入力画面（入庫 / 返品・交換 / 手動販売 / 廃棄の 4 画面で確立）の下部に「直近の{業務名}」見出しと「すべての履歴を見る」（outline、`/inventory/records` へ recordType 付き遷移）を横並びで置き、直近 N 件テーブル（Skeleton / Error / Empty / データの 4 状態、パターン⑥）と各行の「詳細を見る」導線を付ける。直近リストの取得失敗時は「入力中の内容はそのままです。保存や商品追加は続けられます」のように業務継続を保証する文言を出し、フォーム入力を壊さない。新規の業務入力画面でも同じ構成を踏襲する。

**備考列の規則**（UI 磨き batch 3 design、L8-6 + A1(a)(b)(c) 統合）: 備考は対象記録に note フィールドがある画面（入庫・返品交換・手動販売、および `MovementTable.tsx` を共有する在庫変動履歴・記録詳細 7 画面）で必須列とする。廃棄・破損は note フィールドを持たないため対象外（Non-scope、混同しない）。空欄表示は「—」に統一する（`src/features/stock-movements/components/MovementTable.tsx` の備考の `TableCell` の `"—"` 空値表示〈三項演算子〉が 7 画面共有かつアプリ全体でも主流。薄字「備考なし」（`ReturnExchangePage.tsx`）は不採用（owner culling 2026-09-06 で「—」に確定、本 PR（⑮）で「—」へ統一済み））。一定文字数超過時は truncate + `title` 属性で省略表示する（`src/features/operation-logs/OperationLogsPage.tsx` の概要の `TableCell`〈`truncate` + `title`〉相当のパターンを再利用。`src/features/stock-movements/components/MovementTable.tsx` の備考の `TableCell` は本 PR（⑮）で折り返し（`whitespace-normal break-words`）+ `title` 属性へ是正済み。空備考（null・空文字・空白のみ）は「—」で表示し、本文が非空白のときだけ `title` を付与する）。全文確認手段は note の種類で分かれる: 記録本体の note フィールド（例: `src/features/inventory-records/ManualSaleRecordDetailPage.tsx` の `detail.note`）は各記録詳細ページのヘッダー部が truncate なしで直接表示する。一方 `MovementTable.tsx` の行 note（`movement.note`）は、`source` が無い行に元記録リンクが無く（`docs/function-design/66-ui-stock-movements.md` UI-06c-D6、`src/features/stock-movements/components/MovementTable.tsx` の `movement.source` の分岐）、`src/features/inventory-records/StocktakeRecordDetailPage.tsx` 等の記録詳細ページ自身も同じ `MovementTable` を再利用し、修正前は truncate されていたため、「記録詳細ページに任せれば全文が見える」という代替経路が無い。したがって movement note の全文確認手段は `MovementTable` 自身が担う（`title` 属性に加え、折り返し・展開等の手段を持つ）ことを contract とする（DSR-12）。

**「直近の○○」系4画面の統一**（A1a/b/c）: 見出し直下に「直近 {N} 件の{対象}を新しい順に表示します。」の文型を置く（例: 価格履歴 = 「直近 10 件の売価・原価の変更を新しい順に表示します。」）。価格履歴を表構造へ変更したうえで `TableHead` 列見出しを付ける（本 PR（⑮）で反映済み）。囲み（`border`）は意味階層ごとに 1 つまでとし（DSR-16）、本 PR（⑮）で `ManualSalePage.tsx` の内側の追加枠を外し、他 3 画面（入庫・返品交換・廃棄）と揃える（他 3 画面へ箱を追加する方向は DSR-16 に反するため不採用）。見出しの token と右要素の扱いは ① セクション見出し variation に従う（説明は行の下のまま）

**opt-in（⑯ 一覧の器で使用）**: viewport を超える一覧では、`<thead>` を sticky にし（`position: sticky; top: 0`、z-index は header > 固定列 > 本文）、商品コード + 商品名等の識別列を `position: sticky; left: 0` で左固定できる（履歴系は日時 + 種別を固定）。2 列目以降の識別列は 1 列目の実測幅を `left` に反映し、固定 rem 直書きにしない。1 画面に収まる短い一覧には適用しない。適用条件・必須構成の全体は [⑯ 一覧の器（ListShell）](#⑯-一覧の器listshell) を参照（DSR-22）。

**アクセシビリティ**: `Table` primitive の `TableHead` でヘッダーセルを構造化する。sortable headerはactive asc/descを`aria-sort="ascending"` / `"descending"`、inactiveを`aria-sort="none"`で示し、可視indicator `▲` / `▼` は`aria-hidden="true"`とする。状態は色だけでなく日本語ラベル（`廃番` 等の Badge）で示す（DSR-08）。桁揃えは `tabular-nums` で読み取りを助ける。

**Do**:
- コード列は `font-mono`、数値列は `tabular-nums + text-right`
- 折り返しを意図する列にのみ `min-w-0`（または最小幅）+ `whitespace-normal` を付ける

**Don't**:
- すべての列に truncate を当てて主要値を隠さない（DSR-12）
- 状態を色のみで符号化しない

---

## ④ フォームセクション

**使いどころ**: 商品登録・修正フォームを、意味境界（識別 / 分類 / 価格 / 在庫）でセクション分割する。各セクションは見出し + 1 行説明 + 区切り線を持つ。

**canonical**: `src/components/patterns/FormSection.tsx`（`FormSection{title, description?, children}`、description 未指定時は `<p>` 非描画）+ `src/features/products/components/ProductForm.tsx`（`FieldError` / 必須表示の適用例）

**構造**:

```tsx
function FormSection({ title, description, children }: FormSectionProps) {
  return (
    <section className="space-y-3">
      <div className="space-y-1">
        <h2 className="text-xl font-semibold">{title}</h2>
        <p className="text-sm text-muted-foreground">{description}</p>
      </div>
      <Separator />
      {children}
    </section>
  );
}

// 各フィールド
<div className="space-y-1">
  <Label htmlFor="product-name">商品名（必須）</Label>
  <Input id="product-name" value={values.name} onChange={…} />
  <FieldError message={errors.name} />
</div>
```

**使用トークン**: セクション見出しはタイポ `h2`（20px / weight 600）。説明文は `text-sm text-muted-foreground`。セクション内の要素間は `space-3`（12px）、フィールド内ラベル↔説明は `space-1`（4px）。区切りに `Separator`。

**状態**:
- **error**: `FieldError` を入力直下に出す（`text-destructive` + `role="alert"`）
- **read-only**: 編集不可で値を見せる入力は `readOnly` + `bg-muted`（例: 編集時の商品コード・JAN・現在庫）
- **disabled**: 操作自体を不能化する入力は `disabled`（例: 編集時の長さの単位の商品の数量単位 select）
- hover / focus / active: 入力 primitive の既定に従う

**read-only と disabled の使い分け**: 値を見せて編集だけ不能にするなら `readOnly + bg-muted`、操作そのものを止めるなら `disabled`（DSR-05）。

**アクセシビリティ**: すべての入力は `Label htmlFor` で関連付ける。必須はラベルに「（必須）」と明記し、色で示さない（DSR-06）。エラーは `role="alert"` で読み上げ対象にする。

**Do**:
- セクションは意味境界で切る。4 セクション以上に増えたら構成を見直す（DSR-09）
- 各セクションに見出し + 1 行説明 + `Separator` を付ける

**Don't**:
- 必須を色だけで示さない
- read-only と disabled を混同しない
- native `<select>` を使わない（DSR-23）

---

## ⑤ SegmentedControl / 二択切替

**使いどころ**: アプリ内の二択切替（例: sales TabsHeader の日次/月次、monthly ModeTabs の商品別ランキング/部門別構成比）。これはこのアプリの「二択切替ボタン」の標準仕様であり、各画面で独自に padding / border / active tone を組まない。

**canonical**: `src/components/ui/segmented-control.tsx`

**構造**:

```tsx
// local view mode（button group）
<SegmentedControl
  ariaLabel="表示モード"
  value={mode}
  options={[
    { value: "by_product", label: "商品別ランキング" },
    { value: "by_department", label: "部門別構成比" },
  ]}
  onValueChange={setMode}
/>

// route-driven（Link に共有 class を適用）
<div className={segmentedControlListClass} role="group" aria-label="期間">
  <Link to="/sales/daily" className={cn(segmentedControlItemClass, isDaily ? active : inactive)}>
    日次
  </Link>
  <Link to="/sales/monthly" className={cn(segmentedControlItemClass, isMonthly ? active : inactive)}>
    月次
  </Link>
</div>
```

**使用トークン**:

| 項目 | 仕様 |
|---|---|
| root | `inline-flex h-9 w-fit rounded-lg bg-muted p-[3px]` |
| item | `h-[calc(100%-1px)] flex-none rounded-md px-3 py-1 text-sm font-medium` |
| active | `bg-stone-300 border-stone-300 text-stone-950 font-semibold` |
| inactive | `text-foreground/60 hover:text-foreground` |

**状態**:
- **active**: 選択中は `bg-stone-300` + `border-stone-300` + `font-semibold`
- **inactive**: `text-foreground/60`、hover で `text-foreground`
- **focus**: `border-border-strong` + soft ring。mouse click 後に濃い押しボタン状 outline が残らないこと
- disabled: item primitive の `disabled:opacity-50` に従う
- error: 規定なし

**実装ルール**:
- route-driven navigation（例: 日次/月次）は `<Link>` に `segmentedControlListClass` / `segmentedControlItemClass` / active / inactive class を適用する
- local view mode（例: 商品別ランキング/部門別構成比）は `SegmentedControl` の button group を使い、`aria-pressed` と `data-state=active|inactive` を出す
- SidebarLink の現在地は stone 系 selection tone に DSR-21 の Primary アクセントを重ねる。StatusChips / SegmentedControl は選択状態のため stone のままとし、状態 chip は `border-stone-400`、二択切替は押しボタン状の濃い外枠を避けて `border-stone-300` にする
- 役割色（琥珀の注意・確認、操作の色の主要アクションと DSR-21 の現在地アクセント）は、選択状態の背景色（stone）とは分離する

**アクセシビリティ**: `role="group"` + `aria-label` で群を識別（フィルタ toolbar 内で可視 Label を出す場合は `aria-labelledby` で span に紐付け、`aria-label` は出さない）。button group は `aria-pressed` で選択状態を伝える。Windows native L3 では active / inactive / hover / クリック後 focus の 4 状態を比較し、同じ二択切替パターンに見えることを確認する。tab / mode 切替として使う場合は可視 Label を持たない、フィルタ toolbar 内で使う場合は ⑨ の上置き Label 規範に従う（`ariaLabel` は常に必須）

**Do**:
- 二択は共有 visual primitive を使う
- 選択状態の背景色（stone 系）と役割色（操作・注意・確認 等）を分離する

**Don't**:
- 各画面で独自の padding / border / active tone を組まない
- 3 つ以上の切替や内容が異質な切替に流用しない（その場合は Tabs、DSR-02）

---

## ⑥ 空状態・エラー・ローディング

`AlertTitle` は variant を問わず `font-semibold`（600）で icon の真横の 1 行を本文（`AlertDescription`）より強く出す。

**使いどころ**: 一覧・カード・レポートの「取得中」「取得失敗」「該当 0 件」を一貫した形式で示す。

**canonical**: `src/features/products/ProductListPage.tsx`（Skeleton → Alert → 空状態 → データの分岐構造）+ `src/components/patterns/EmptyState.tsx`（空状態の到達形）

**構造**:

```tsx
{query.isLoading ? (
  <div className="space-y-2">
    <Skeleton className="h-10 w-full" />
    <Skeleton className="h-10 w-full" />
    <Skeleton className="h-10 w-full" />
  </div>
) : query.isError ? (
  <Alert variant="destructive">
    <AlertTitle>商品一覧の取得に失敗しました</AlertTitle>
    <AlertDescription>
      検索条件を変えるか、しばらくしてからもう一度お試しください。
    </AlertDescription>
  </Alert>
) : query.data?.items.length === 0 ? (
  <EmptyState
    icon={PackageSearch}
    title="該当する商品がありません"
    description="検索条件を変更するか、新しい商品を登録してください"
    action={/* 「商品を登録する」link button（主動線がある画面のみ） */}
  />
) : (
  <ProductTable items={query.data.items} />
)}
```

**空状態の 2 系統**（PR-B D-B5 で規約化）:
- **0 件成功** = `EmptyState`（pure テーブル component 内 or ページ分岐内）。テーブル系は `rows.length === 0` の内部分岐の中身として埋め込む — props 駆動の pure presentational 責務は不変
- **取得失敗** = ページ側 Alert で差し替え（パターン③テーブルの「取得失敗はテーブルごと差し替え」の射程）。`EmptyState` は使わない

**適用除外**: `EmptySearchPlaceholder`（検索前の操作指示 = 空結果ではない）と shortcuts の `emptyMessage`（navigation config 駆動で利用者アクション不能）は semantic が「取得結果 0 件」と異なるため EmptyState 化しない。明細 0 行系（例: 入庫 `ReceivingPage.tsx` の「入庫する商品がありません」）も同様に適用除外とする — 絞り込み結果が 0 件なのではなく、業務入力の明細行が単に空という別 semantic のため。

### filter-empty reset action（絞り込み解除、2026-08-03 batch B で新設）

一覧画面で「絞り込み条件を既定値以外にした結果、0 件になった」ケースに限定した回復導線。

**分類軸**（round 1 P1-1 対応。reset 対象を全数分類する判断基準）:

- **reset 対象** = 結果集合を狭める任意の絞り込み条件（検索語・フィルタ・状態チップ・期間絞り込み）を持つ一覧画面の filter-empty。絞り込みの既定値復帰が「全件（既定）表示」という自明な回復になる画面のみ。
- **除外(a) 範囲外 page 回復** = 別 semantic の既存専用導線（「先頭ページに戻る」、UI-11c-D8 等）が既にある。
- **除外(b) 期間主キーレポート** = 日付 / 月という対象選択が支配的で、絞り込み解除がデータ存在を保証しない（日報・月報系）。条件変更の案内は既存 description 文言が担う。
- **除外(c) 業務入力の明細 0 行** = 入力明細が空なだけで絞り込み結果ではない。
- **除外(d) 直近実績サマリ / 詳細画面の関連データ不在** = 絞り込み条件を持たない、真にデータなし系。
- **優先規則**（round 2 P2-2 対応、分類軸の排他性）: 対象選択条件（「すべて」という既定値を持たない必須の主キー = 日報の日付、月報の月）を持つレポート画面は、副次絞り込み（例: 日次売上の部門フィルタ）があっても除外(b) を優先して reset 対象外とする。理由 = reset の統一契約は「全絞り込み条件の既定復帰」であり、対象選択は既定復帰の対象になり得ない。副次絞り込みだけを戻す部分 reset を許すと、同じ「絞り込みを解除」ボタンが site ごとに違う挙動になる。部分 reset（「部門の絞り込みを解除」等の別文言）は要望発生時に別 change で再評価する。
- reset が戻すのは絞り込み条件のみ。並び替え（sort / dir）・表示件数（perPage）は結果集合を狭めないため対象外。

- **表示条件**: 絞り込み条件が既定値以外、かつ結果が 0 件のときのみ、EmptyState の `action` に絞り込み解除ボタン（`variant="outline"`）を置く。絞り込みが既定値のまま 0 件（真にデータなし）のときは action を出さない。
- **動作**: 押下でその画面の絞り込み条件をすべて既定値へ戻す。URL search param 画面は search param を既定値へ、local state 画面は state を既定値へ戻す。`page` を持つ画面は `page` も既定（1）へ戻す。並び替え（sort / dir）・表示件数（perPage）は変更しない（分類軸の対象外規定どおり）。
- **文言**: 「絞り込みを解除」で全採用箇所を統一する。
- **既存 action との共存規定**（round 1 P1-1 対応、新設）: 既存の主動線 action を持つ画面（商品一覧の「商品を登録する」）は、その action を常設のまま維持し、絞り込みが非既定のときだけ reset ボタンを `action` slot 内へ横並び（flex）で併置する。既存 action が先、reset ボタンが後。`action` slot に複数ボタンを置く場合は wrapper を `flex flex-wrap items-center justify-center gap-2` とし中央揃えにする（EmptyState 本体の中央揃えと整合させる。owner L3 2026-08-03）。
- **採用箇所**: 棚卸し（`StocktakePage`、部門フィルタ / 未入力のみ toggle / `page` を既定へ戻す。73 §73.6 round 2 P2-1 是正と整合）/ 在庫照会（`StockInquiryPage`）/ 入出庫履歴一覧（`InventoryRecordsPage`）/ 在庫変動履歴（`StockMovementsPage`）/ 操作ログ（`OperationLogsPage`）/ 商品一覧（`ProductListPage`、round 1 P1-1 で追加）の 6 画面（D-138 の後は在庫照会と商品一覧が「商品と在庫」の 1 画面になり 5 画面。状態も戻す条件に入る、50 §50.7）。商品一覧は上記共存規定に従い既存「商品を登録する」action と共存する。操作ログは既存の `defaultFilter` 判定の非既定側にのみ reset action が付く。既存の範囲外 page 回復 action「先頭ページに戻る」（§74 参照）とは対象ケースが異なり、同一画面内で共存する。
- **適用除外**: 本節冒頭の適用除外（`EmptySearchPlaceholder` / shortcuts `emptyMessage` / 明細 0 行系）および上記分類軸の除外(a)〜(d) と同じ。これらは「絞り込み結果 0 件」ではないため reset action の対象にしない。

**バリエーション: インライン選択エラー 1 スロット**（PR #125、canonical: `src/features/daily-report-import/DailyReportImportPage.tsx` の `SelectionErrorMessage`）: ファイル選択（DSR-14 の path-based 方式）など非フォーム文脈の入力検証エラーは、発生源（選択ボタン）直下の 1 スロットに `role="alert"` + destructive テキスト + アイコンで表示する。エラー state は選択試行のたびに置換し、成功で `null` にクリアする。画面上部の Alert 帯（データ安全系専用）とは役割を混ぜない（DSR-03 の 3 階層）。フォーム文脈の入力検証はパターン④の `FieldError`（入力直下）が既定で、本バリエーションは非フォーム文脈専用。

### Alert warning variant（画面上部の注意喚起、owner v4 決定で確定）

業務を止めないが読み飛ばされては困る注意文言（destructive ほど致命的ではない）には `Alert` `warning` variant を使う。`alertVariants` の `warning` は次を持つ（`data-variant` にも出力）: `warning: "bg-warning-soft border-warning text-warning-strong [&>svg]:text-warning *:data-[slot=alert-description]:text-warning-strong/90"`。①状態 badge と同じ soft/border/strong/icon の 4 点構造（`bg-warning-soft` 塗り + `border-warning` 枠 + `AlertTriangle`〈icon `text-warning`〉+ 本文 `text-warning-strong`）。枠は `--warning`（badge ①状態の `--warning-border` より一段濃い）。子要素は `AlertTriangle` icon + `AlertTitle` + `AlertDescription` の 2 段（DSR-11 に準拠）。

```tsx
<Alert variant="warning">
  <AlertTriangle aria-hidden="true" />
  <AlertTitle>ご注意</AlertTitle>
  <AlertDescription>
    画面を再読み込みすると、保存していない変更は失われます。
  </AlertDescription>
</Alert>
```

適用先: `src/features/products/PriceRevisionPage.tsx` の注意の `Alert`（移行前 anchor）。`<Alert variant="warning" role="note">` + icon + `AlertTitle`「ご注意」へ移行済みで、既存本文は `AlertDescription` に維持する。追加適用先（owner 決定、2026-09-06）: ホーム『前日分が未取込みです』（`HomePage.tsx`）は destructive のまま（警告。三角 icon は部品が描く）、日次 / 月次売上の『レジ日報は未取込みです』（`src/features/daily-sales/DailySalesPage.tsx`、`src/features/monthly-sales/MonthlySalesPage.tsx` の未取込みの表示）は warning variant。success Alert の本文色は現状（`text-success-strong`）を維持する（owner 保留、2026-09-06）。

**Alternatives considered**: (a) `bg-card` 据え置き + `text-warning-strong`（対 `bg-card` 8.32:1、AA 達成だが「薄い」という owner 指摘を soft 背景なしでは解消できず不採用）。(b) `bg-card` 据え置き + `text-foreground` 本文、枠と icon のみ amber（owner が「すっきり見えるが警告表示としての一貫性に欠ける」と評し不採用）。(d) タイトルのみ `text-warning-strong` bold、本文 `text-foreground`（見出しのみの強調では①状態 badge との視覚言語統一に届かず不採用）。

`destructive` variant は `warning` と対称の 4 点構造にする（decision-log D-094）: `destructive: "bg-destructive-soft border-destructive text-destructive-strong [&>svg]:text-destructive *:data-[slot=alert-description]:text-destructive-strong/90"`。destructive の Alert は部品が三角 icon を描き、site は icon を書かない（`variant === "destructive"` のとき `alert.tsx` が `TriangleAlert`〈`aria-hidden`〉を最初の子として描く。DSR-08 の icon 必須を 1 か所で守り、今後の site の書き忘れを構造で防ぐ）。`warning` と既定の variant は部品が icon を描かず、warning の site が `AlertTriangle` を書く。失敗でも戻せない操作でもない知らせ（例: 取込み中の移動制限。待ってほしい知らせ）は destructive にせず warning にする。

### ローディング状態の標準UI

| パターン | 用途 | コンポーネント |
|---------|------|-------------|
| **Skeleton** | 初回ロード（テーブル・カード・リスト） | shadcn/ui `Skeleton` |
| **Spinner** | ボタン内処理中（保存・送信） | lucide `Loader2` + `animate-spin` |
| **Progress Bar** | 確定的進捗（CSV取込み、バックアップ） | shadcn/ui `Progress` |
| **透過オーバーレイ** | 画面全体ロック必要時（データ整合性チェック中） | カスタム component + backdrop |

共有 `ListSkeleton`（所在: `src/components/patterns/ListSkeleton.tsx`）は表・一覧の列構造を保つ取得中表示に使い、props は `rows?: number`（既定 6）/ `columns?: number`（既定 8）とする。

**不使用**: ドット点滅、波形アニメ等の装飾ローディング（業務アプリの集中を妨げる）

### 空状態（Empty State）の標準UI

正典は `src/components/patterns/EmptyState.tsx`（`EmptyState{icon?, title, description?, action?}`）。各画面で一貫した形式とする:
- **アイコン**（lucide、24px、`stone-400`、`aria-hidden`、省略可）
- **見出し**（h3、`text-base font-medium text-stone-700`、例: "該当する商品がありません"）
- **説明**（`text-sm text-stone-500`、例: "検索条件を変更するか、新しい商品を登録してください"、省略可）
- **アクション**（slot、例: "商品を登録する" link button。**主動線が画面ヘッダ等の直上に既にある場合はボタンを重複させず description の文言で次の一手を示す**、省略可）

3 行以内で簡潔にまとめる。利用者が「次に何をすればいいか」を即座に理解できることが目標。

説明は利用者がいま選んでいる条件に合わせる。検索語を入れていない 0 件で「検索語を変えて」と促さない（部門・状態だけで絞ったなら「絞り込みを解除してください」）。状態の絞り込みだけで 0 件なら、何が無いかを見出しで言う（例「在庫切れ・在庫少の商品はありません」）（DSR-11、D-138。商品と在庫の画面は [50](../function-design/50-ui-product-list.md) §50.7。今の build の在庫照会の在庫少の 0 件は、検索語の有無に関わらず検索語を変えるよう促す）。

### 色だけに依存しない状態表示

WCAG 2.1 AA の前提として、状態・警告・選択・比較の意味を色だけで伝えない。グレースケール表示でも、日本語ラベル、アイコン形状、位置、罫線、バッジ形状などで識別できることを求める。

在庫照会の `在庫切れ` / `在庫少` はこの対象である。赤 / amber の文字色は補助として残してよいが、実利用者が赤黄を識別できない場合でも意味が読める実装にする。

**使用トークン**: Skeleton は shadcn/ui 既定。エラーは `Alert` destructive variant（`bg-destructive-soft` + `border-destructive` + `text-destructive-strong` + 部品が描く三角 icon）。注意喚起（業務は止めない）は `Alert` warning variant（`bg-warning-soft` + `border-warning` + `text-warning-strong` + `AlertTriangle`）。空状態は `EmptyState` 既定（囲み `rounded-md border p-12 text-center`、見出し `text-stone-700`、説明 `text-sm text-stone-500`、アイコン 24px `text-stone-400`）。

**状態**: 本パターン自体が loading / error / empty の 3 状態を表す。hover / focus / active / disabled は規定なし。

**アクセシビリティ**: エラー Alert は `AlertTitle`（原因）+ `AlertDescription`（次の一手）の 2 段で書く。空状態は「何が無いか + 次の一手」を文言で示し、意味を色やアイコンだけに閉じない（DSR-11）。

**Do**:
- loading は Skeleton、error は Alert（原因 + 解決策）、empty は文言 + 次の一手
- 空状態文言は「何が無いか」と「次に何をすればよいか」を書く

**Don't**:
- 装飾ローディング（ドット点滅・波形）を使わない
- エラーや空状態を 1 単語（例: "Error"）だけで済ませない

---

## ⑦ Toast

**使いどころ**: 保存・取込み・出力といった一過性の操作結果を低割込みで通知する。画面に居座らせる必要のない「完了しました」系。

**canonical**: `src/features/products/ProductFormPage.tsx`（保存成功 toast）、`src/lib/hooks/useExportFile.ts`（出力成功・失敗 toast + id 規約）

**構造**:

```tsx
import { toast } from "sonner";

// 保存成功
toast.success(`商品「${name}」を登録しました（商品コード: ${result.product_code}）`, {
  id: "product-save-success",
  duration: 5000,
});

// 出力成功 / 失敗（id は reportType ごとに dedup）
toast.success(`${label} を保存しました（${count} 件）`, { id: `export-${reportType}-success` });
toast.error(`出力に失敗しました: ${message}`, { id: `export-${reportType}-error` });
```

| 形式 | 割込み度 | 用途 | 実装 |
|------|---------|------|------|
| **Toast**（Sonner） | 低（自動消去） | 操作結果の通知。「保存しました」「取込み完了」 | 成功は緑系、警告は琥珀系、失敗は赤系 |

**判定基準**:
- **業務継続を妨げるか** = Yes → Dialog / No → Toast or Banner
- **認知されないと困るか** = Yes → Banner / No → Toast
- **利用者のアクション必須** = Yes → Dialog / No → Toast or Banner

**id 規約**: 連打・再試行・連続 callback で同一通知が重なり得る場合は、DSR-19 の適用基準に従って安定 id を付ける。保存系は `product-save-success`、出力系は `export-${reportType}-success` / `export-${reportType}-error` のように、利用者が個別に識別すべき別対象まで潰さない範囲で dedup する。

**使用トークン**: 成功 = Success（`green-700`）系、警告 = Warning（`amber-600`）系、失敗 = Destructive（`red-700`）系。色は Sonner の variant（`toast.success` / `toast.error`）経由で当て、生 Tailwind 色 class を書かない（DSR-08）。

**状態**: `RootLayout` の Toaster 全体既定は `duration={3000}`（3000ms）。個別上書きは DSR-19 の階層に従い、重要情報付きの成功通知を 5000ms、取消結果または非 IT operator の読了配慮が必要な通知を 8000ms とする。hover / focus / active / disabled / error は規定なし（error 内容は toast 文言で表す）。

**アクセシビリティ**: 通知文言は「何が」「どうなったか」を日本語で具体的に書く（例: 商品名 + 商品コードを含める）。Sonner が live region で読み上げる。

**Do**:
- 一過性の結果は toast、回復導線が要る状態は Alert（DSR-03）
- 連発し得る通知には安定 id を付ける

**Don't**:
- 利用者の操作が必須な内容を toast で流さない（Dialog にする）
- 自動消去で消えると困る持続的状態を toast にしない（Banner にする）

---

## ⑧ Dialog / 確認

**使いどころ**: 破壊的・不可逆な操作、または誤実行が会計・在庫の重複計上を生む高影響操作の直前に明示確認を挟む（例: 廃番化、売上の同日別ファイル追加）。復帰・再表示など影響の小さい可逆操作には確認を挟まず直接実行する。

加えて、選択・入力を画面から切り離して dialog に閉じる非確認 dialog も本節が扱う（picker dialog、DSR-24）。

**canonical**: `src/features/products/components/DiscontinueConfirmDialog.tsx`（廃番化の確認）。売上の同日追加確認は [55-ui-csv-import.md UI-07-D13](../function-design/55-ui-csv-import.md) を契約正本とし、後続実装で両タブ共通componentへ移行する。

**構造**:

```tsx
<AlertDialog open={open} onOpenChange={(next) => { if (!next) onCancel(); }}>
  <AlertDialogContent>
    <AlertDialogHeader>
      <AlertDialogTitle>この商品を廃番にしますか？</AlertDialogTitle>
      <AlertDialogDescription>
        商品「{productName}」は商品一覧の通常表示から外れます。あとから「表示に戻す」で戻せます。
      </AlertDialogDescription>
    </AlertDialogHeader>
    <AlertDialogFooter>
      <AlertDialogCancel onClick={onCancel}>キャンセル</AlertDialogCancel>
      <AlertDialogAction variant="destructive" onClick={onConfirm}>廃番にする</AlertDialogAction>
    </AlertDialogFooter>
  </AlertDialogContent>
</AlertDialog>
```

説明の画面名は D-138 の後（proposed・未実装）は「商品と在庫」にし、`商品「{productName}」は商品と在庫の通常表示から外れます。あとから「表示に戻す」で戻せます。` とする（上の構造と今の build は「商品一覧」）。

| 形式 | 割込み度 | 用途 | 実装 |
|------|---------|------|------|
| **Dialog** | 高（明示ボタン必須） | 確認・警告。「廃番にしますか」「整合性チェック差異あり」 | shadcn/ui `AlertDialog` |
| **Dialog（非確認）** | 中（明示ボタンで開閉、確認は挟まない） | 候補選択・追加導線の集約。「取引先を選択」 | shadcn/ui `Dialog` |
| **Banner** | 中（常時表示、手動閉鎖可） | 画面上部の持続的通知。「PLU未反映が3件」「バックアップ実行中」 | shadcn/ui `Alert` |

**確認境界**: 破壊的・不可逆操作のみ確認を通す。可逆操作（「表示に戻す」等）は直接実行する。廃番化 = 確認あり / 表示に戻す = 直接実行が実例（DSR-07）。

**使用トークン**: `AlertDialog` の shadcn/ui 既定。destructive 確認の実行 Action は `variant="destructive"`、キャンセルは `AlertDialogCancel` とする（DSR-20）。

**状態**:
- **open**: parent state で制御。`open=false` への経路は (1) onConfirm (2) onCancel（Esc / 外側クリック / キャンセルボタン）の 2 つ
- **配置**: 2 ボタンの DOM 順は Cancel → Action。`sm` 以上は Cancel 左 / Action 右、narrow 幅は footer の `flex-col-reverse` により Action 上 / Cancel 下とする。3 ボタンは取引先統合 stage 2 の secondary → Cancel → Action を先例とする
  inline 行操作（例: 取引先管理の名前変更）の 2 ボタンも同じ DOM 順（Cancel → Action）に揃える
- **硬化**: Esc / 外側クリックは cancel ブリッジが本則。選択前 dismiss が実害へ直結する保存結果、または未保存破棄の明示選択を求める場合だけ、DSR-20 の明示 prop（`onEscapeKeyDown` / `onPointerDownOutside`、close button を持つ場合は `showCloseButton={false}`）で硬化する
- hover / focus / active / disabled / error: ボタン primitive の既定に従う

**アクセシビリティ**: `AlertDialogTitle`（問い）+ `AlertDialogDescription`（影響と復帰可否）を必ず書く。Esc・外側クリックを `onOpenChange` 経由で cancel にブリッジし、キーボードだけで閉じられるようにする。DSR-20 の硬化条件に該当する例外では、Esc が無効であることを明示 prop と test から判読可能にする。確認ボタンの文言は操作内容そのもの（例: "廃番にする"）、Cancel は既定「キャンセル」または後状態を具体化する文言にする。

**Do**:
- 破壊的操作の直前にのみ確認を挟む
- destructive Action の variant・配置・文言を DSR-20 にそろえる
- 確認ボタンの文言を操作内容と一致させ、硬化は条件と実装手段を明示する

**Don't**:
- 可逆操作に確認を挟んで操作を重くしない（DSR-07）
- `onOpenChange` の非配線で暗黙に硬化しない（DSR-20）
- destructive Action を「OK」のような実行内容の分からない文言にしない

**比較用 variant（DSR-16 適用形）**: dialog 内に同型レコードを複数件示し、利用者が比較して判断する必要がある場合（例: 同じ日の既存取込みと今回分を見比べて重複を判断する）は、確認ダイアログの標準構造に加えて次を満たす。

- **canonical**: `src/features/csv-import/components/AdditionalImportConfirmDialog.tsx`（追加取込み確認、gated Amendment 3 で DSR-16 準拠へ再構成）
- 既存レコード群は per-card の `dl` 反復ではなく、列の揃った structured list（共通の列見出し、または揃った grid 行）で示し、囲みは全体で 1 つに留める（内部の反復は余白 / `border-b` の行区切り）
- 今回分（新規に追加しようとしている 1 件）は「今回」ラベル付きの独立領域として分離し、既存分との境界を明確にする
- 日時は machine 形式でなく人間向け表示（`YYYY-MM-DD HH:mm` 等）にする
- 列を並べて比較するため、既定の `AlertDialogContent` 幅では窮屈になりやすい。比較に必要な列数に応じて幅を拡張する（`sm:max-w-2xl` 等、既存 dialog の幅指定慣行に従う）

**Do**:
- 同型レコードの比較には列を揃えた structured list を使う（per-card 反復にしない、DSR-16）

**Don't**:
- 比較目的の複数件を、囲み（border/カード）だけを識別信号にした per-card 反復で並べない（DSR-16）

**picker dialog（複数候補から 1 件選ぶ + 追加導線、DSR-24）**

**使いどころ**: 取引先等、追加導線を伴う complete master data から 1 件を選ぶ場面（DSR-24 適用条件）。

**canonical**: `SupplierPickerDialog`（`src/features/suppliers/components/SupplierPickerDialog.tsx`）。

**構成**: 起動ボタンは現在値（取引先名、未指定時は文脈に応じた先頭行の文言）を表示する。右端に `ChevronDown`（`text-muted-foreground`（`SelectTrigger` と同じ薄さ）、開く控え）を置き、文言は `truncate` で枠内に収める。一覧から名前を解決できない場合（取得失敗 / 削除済み id）は「取引先を確認できません」と表示し、未指定の文言で断定しない。ヘッダ（title、例:「取引先を選択」）+ 本体（名前検索 input、live・client-side filter — `list_suppliers` は無引数のため取得済み一覧をここで絞り込む — + 検索 input の下・scroll 箱の**外**に「現在の選択」の固定帯（白地の箱（全周の枠 `--border-strong` + 角丸、背景は `--background`）+ 小見出し「現在の選択」+ 取引先名（medium）。一覧の現在行〈進行中の地 `--ongoing-soft` + 左 4px の進行中の枠 `--ongoing` のバー〉の語彙は帯に使わず、箱 = 要約 / 進行中の地 + バー = 現在行 と分ける。帯に badge は置かない。名前を解決できない場合は起動ボタンと同じ「取引先を確認できません」を帯にも同じ関数で出す。「すべての取引先」/「取引先なし」を選んでいればそれが帯に出る。）+ scroll 一覧（箱の見た目は商品一覧の表を流用、列見出しの「選択」文字は表示せず sr-only（読み上げ専用）にし列は ✓ + 「選択中」badge のみ、行に現在選択（DSR-22 の現在行 3 点: 左 4px の進行中の枠 `--ongoing` のバー + 進行中の地 `--ongoing-soft` + ✓ と「選択中」badge / 文言。色だけに頼らない）、フィルタ文脈では先頭に「すべての取引先」行、入力文脈では先頭に既存 sentinel 相当の行〈「取引先なし」/「指定なし」〉を維持）+ footer（枠外固定、左「新しい取引先を追加（`Plus` icon 付き）」**primary（操作の塗り（`--primary`）、inline SVG icon。全角「＋」文字は使わない）**・右「閉じる」outline）。一覧行クリック = 選択確定 + dialog を閉じる。footer は一覧の scroll と独立して常時固定表示する。picker dialog は独立した surface であり、その内部の主動線は追加ボタン 1 個。画面本体の主動線とは surface が異なるため 1 画面 1 主動線は維持される。`CreateSupplierDialog` 側の確定ボタンも同様に自 surface の主動線。

**動作**: 外クリック / Esc = 「閉じる」と同じ（選択は変更しない、DSR-20 の硬化対象ではない通常 Dialog）。「新しい取引先を追加」→ 既存 `CreateSupplierDialog` をそのまま開く（owner 仕様）。追加成功後は一覧を再取得し、新規取引先を自動選択して両方の dialog を閉じる（owner 確定 2026-09-10、A 案）。picker は確認 Action を持たず（一覧行クリックが確定）、footer 左は別 surface を開く primary（追加の確定へ進む導線、操作の塗り（`--primary`））、右が dismiss。⑧ の **配置** bullet が定める Cancel 左 / Action 右 は確認 dialog の 2 ボタン規則であり、picker footer には適用しない。

**dialog 重ね契約**（`CreateSupplierDialog` を picker dialog の上に開く）:
- **(A) 推奨**: Radix `Dialog.Root` を picker dialog の内側でネストする。owner 仕様「既存 CreateSupplierDialog をそのまま開く」に文字面で忠実。両 dialog とも `z-50` の overlay を持ち、後着 dialog が DOM 順で後にマウントされ視覚的に手前へ来る。ESC は Radix `DismissableLayer` のスタック管理により最前面の dialog のみを閉じる想定、フォーカスは `CreateSupplierDialog` を閉じたあと picker dialog へ戻る想定。**本アプリに dialog-in-dialog の先例が無いため、これらは実機未検証の期待値であり、Windows WebView2 で ESC・外クリック伝播・focus trap を確認する（runtime lane の Windows WebView2 実機確認で検証する）**。内側 dialog の overlay は既定のまま（二重 scrim を許容、owner 確定 2026-09-10）とする（根拠: `src/components/ui/dialog.tsx` の `DialogOverlay` `bg-black/50` + `z-50`）。
- **(B) fallback**: `MergeSupplierDialog`（単一 dialog 内 2-stage の既存先例）型の単一 dialog 内 2-stage（stage 1 = 一覧、stage 2 = 追加フォーム）。dialog を重ねないため WebView2 リスクを避けられるが、`CreateSupplierDialog` の実装（validation・toast・エラー表示）を picker dialog 内に複製することになり、owner 仕様「既存 CreateSupplierDialog をそのまま開く」との文字面が一致しない。(A) の実機確認で問題が出た場合のみ (B) に切替える。

**状態**: 取得中 = Skeleton、取得失敗 = dialog 内 Alert（UI-01b-D8 の「取引先未指定なら保存可能」契約は不変）、検索 0 件 = `EmptyState`（⑥ の 0 件成功系統。⑥ の「絞り込み解除 action」は置かず、検索 input のクリアで代替する）。

**a11y**: `DialogTitle` + `DialogDescription` を置く（⑧ アクセシビリティ節の `AlertDialogTitle` / `AlertDialogDescription` 規定を素の `Dialog` に拡張して適用）、open 時の初期 focus は検索 input（⑨ live 型と同じ、スキャナ入力を即受け付ける）、一覧選択または「閉じる」で閉じたら起動ボタンへ focus を戻す。`CreateSupplierDialog` を閉じたら picker の検索 input へ戻す。

**Do**: 追加導線を伴う候補は本パターンに統一する（DSR-24）。footer の 2 ボタンを枠外固定にし、一覧の scroll と独立させる。

**Don't**: Select 横に primary 色の追加ボタンを並べない（DSR-24 Why 参照）。追加導線を持たない候補（部門等）にまで本パターンを広げない。

---

## ⑨ 検索 + フィルタ

**使いどころ**: 一覧 / 記録画面の toolbar 全部、toolbar の全フィルタ入力。HID バーコードスキャナの Enter 確定に対応し、選択状態を URL state に連動させる。

**商品を探す検索欄の当たる列**（D-138）: 商品の一覧の検索欄（商品と在庫・一括価格改定）は、どの画面・どの状態のチップでも商品名・商品コード・JAN・メーカー品番の 4 つに当たる（SPEC-PRV-D2。伝票と発注書でいちばん見る番号はメーカー品番、repo 外の回答台帳 TD-362）。同じ検索欄が状態によって当たる列を変えない（今の build の在庫照会は、在庫切れ・在庫少でメーカー品番に当たらない。[58](../function-design/58-ui-stock-inquiry.md) UI-06a-D8）。記録を商品の語で探す入出庫履歴の検索欄は対象外（記録を探す別の問い）。

**canonical**: `src/components/patterns/SearchBar.tsx`（検索欄。`debounceMs` 未指定 = commit 型（Enter/ボタン確定 + trim、現在の採用箇所なし・機能残置）/ 指定 = live 型（debounce + Enter flush、商品一覧・在庫照会、`debounceMs=200`）。両モードとも `event.isComposing` ガードで IME 変換確定 Enter の誤発火を防止。商品一覧は 2026-08-03 owner L3 是正で commit 型から live 型へ統一）、`src/components/patterns/DepartmentFilter.tsx`（部門フィルタ。allLabel 既定「すべての部門」）

**構造**:

```tsx
// 検索欄（commit 型、現在の採用箇所なし・機能残置）: draft 内部保持 + Enter/検索ボタンで確定（trim あり）。
// 未指定時の id 既定は "search-input" — 採用時は呼び出し側で明示する
<SearchBar
  value={q}
  onSearchChange={(value) => updateSearch({ q: value })}
/>

// 検索欄（live 型、商品一覧・在庫照会）: debounce + Enter 即時 flush（trim なし）。可視 Label + input id あり、ボタンなし、type="search"
<SearchBar
  debounceMs={200}
  value={q}
  onSearchChange={onSearchChange}
/>

// 部門フィルタ: 候補は listDepartments の master 全件（filtered result 由来にしない）。
// 幅は部品が w-[11rem] に固定し、呼び出し側では渡さない（幅の prop を持たない）。idPrefix は呼び出し側で現値を維持する
<DepartmentFilter
  options={departmentOptions}
  selected={dept ?? null}
  onChange={(deptId) => updateSearch({ dept: deptId ?? undefined })}
  idPrefix="product-dept-filter"
/>

// DepartmentFilter 内部（上置き Label、runtime 反映済み（PR #63））: label は raw label 要素、htmlFor は SelectTrigger の id を指す
<div className="grid gap-1">
  <label className="text-sm text-muted-foreground" htmlFor={triggerId}>部門</label>
  <Select …>
    <SelectTrigger id={triggerId} className="w-[11rem]">…</SelectTrigger>
    …
  </Select>
</div>
// component 1 箇所の改修で 5 サイトが揃う。
```

**使用トークン**: commit 型は wrapper `min-w-[18rem] flex-1` + 要素間 `space-2`（8px）+ ラベル `text-muted-foreground`。**すべてのフィルタ入力**（検索 / Select / date / number / `DepartmentFilter` / 並び替え / 表示件数 / フィルタ toolbar 内の SegmentedControl）は `div.grid.gap-1` 配下に可視 label を上置きし、入力要素を続ける。可視 label は `<label className="text-sm text-muted-foreground" htmlFor={…}>`（weight 400、既存の上置きサイト〈入出庫履歴 / 操作ログ / 在庫変動〉と同じ見た目）。`Label` component を使う場合は `className="text-sm font-normal text-muted-foreground"` を渡して component 既定の `font-medium` を打ち消す（`cn()` の twMerge は同じ font-weight group の後勝ちで置換する。class を足すだけでは消えない）。呼び出し側 toolbar は `flex flex-wrap items-end gap-3` で入力欄の下辺を揃える（⑭ で live 型 SearchBar に導入した形を全入力へ拡張。owner 2026-09-08「検索ツールの場所の表記は揃えたい」）。例外 2 種: **Checkbox は label 内包の横並び**（`<label className="flex items-center gap-2 text-sm">` に `Checkbox` + 文言、文言は muted にしない、checkbox の慣行）を維持し、toolbar 内では `h-9 items-center self-end`（入力と同じ高さの箱を行の下端に揃え、その中で縦中央。上置き label を持たないため行の中央では浮く。owner L3 2026-09-15）。**tab / mode 切替として使う SegmentedControl（monthly ModeTabs、PLU書出しの書出しモード 等、⑤ 参照）は可視 Label を持たない**（`ariaLabel` 必須）。判定軸は選択肢の数ではなく文脈であり、フィルタ toolbar 内の SegmentedControl（廃番表示 / PLU表示 / 並び順）は `ariaLabel` の文言を上置き label として可視化する（廃番表示と PLU表示は隣接してどちらも「すべて」を持ち、label なしでは識別できない。並び順も同じ列にあるため揃える。owner 確認 2026-09-11 で確定。廃番表示 3 択 / PLU表示 5 択が SegmentedControl なのは DSR-02 drift〈2 択以外は Tabs〉であり、Tabs / Select 化しても同じ文脈軸を適用する）。SegmentedControl は `role="group"` の button 群で labelable 要素を持たないため、この label は `<span id={…} className="text-sm text-muted-foreground">` + `aria-labelledby` で紐付け、`htmlFor` は使わない。live 型 SearchBar は `Input`（`max-w-md` 維持）を続ける。`id` 未指定時は `useId()` でラベルとの対応を一意にし、`label` 未指定時は「商品を検索」。部門フィルタ（`DepartmentFilter`）の幅は部品が `w-[11rem]` に固定し、呼び出し側では変えない（旧 04 の token 表「検索欄」の行「部門 select 幅は全画面同一」。11rem は、初期データで最も長い部門名「ビューティ関連」〈7 文字〉と未選択の「すべての部門」を、trigger の `text-sm` の字幅に左右 padding と chevron を足しても切らずに収める見積もりの値で、runtime 反映時に owner の実機確認〈L3〉で確かめる。より長い部門名が入れば見直す）。部品を使わない入出庫履歴の部門 select は `w-44`（= 11rem）で同じ幅を持つ。そのほかのフィルタの Select は固定幅を呼び出し側で指定する。

**状態**:
- **disabled**: フィルタは候補ロード中 `disabled` にできる
- focus: 検索欄は両モードとも初期 focus を当て、スキャナ入力を即受け付ける
- hover / active / error: 規定なし（部門候補の取得失敗は呼び出し側で別途文言表示）

**入力の確定経路**: commit 型は draft（local state）→ Enter / 検索ボタンで確定（trim あり）。live 型は `debounceMs` の遅延反映 + Enter で即時 flush（trim なし）。HID スキャナの Enter はどちらのモードでも通常キーボードの Enter と同じ経路を通る。IME 変換中の Enter は両モードとも `isComposing` 除外で検索確定と混ざらないようにする。

**フィルタ候補のソース**: 部門候補は `listDepartments` の master 全件から作る。現在の絞込み結果（filtered result）から候補を作ると、選択値が候補から消えて他候補へ切り替えられなくなる縮退が起きるため、これを禁止する（DSR-10）。

**アクセシビリティ**: commit 型は可視 `Label htmlFor` + `aria-label="商品検索"` の両方を持つ（accname 計算順で `aria-label` が native label に優先するため、accessible name は aria-label が優先）。live 型は可視 `<label>`（既定文言『商品を検索』、画面ごとに上書き可）のみを accessible name とし、aria-label は持たない（WCAG 2.5.3 Label in Name）。`placeholder` はいずれのモードも入力例の補助に留め、識別の手段にしない。フィルタの未選択は「すべての部門」という日本語 default で示す。

**Do**:
- commit 型の検索は Enter 確定（スキャナ互換）+ ボタン確定の両経路を持つ
- live 型の検索は Enter 即時 flush を持つ（スキャナ互換は Enter 経路で担保）
- フィルタ候補は master 全件から作る（DSR-10）

**Don't**:
- フィルタ候補を絞込み結果から派生させない
- IME 変換中の Enter を検索確定と取り違えない
- live 型に `aria-label` を追加しない（可視 Label が唯一の accessible name、commit 型と live 型で識別方式を混在させない）
- native `<select>` を使わない（DSR-23）

---

## ⑩ ページネーション

**使いどころ**: 件数の多い一覧の前後送り。1 ページあたり表示件数（perPage）と前へ / 次へを提供する。一覧はすべて上部にも件数 + 現在位置を置く（DSR-22、Lane 4）。

**canonical**: `src/components/patterns/Pagination.tsx`（`Pagination` = 前へ / 次へ + 件数表示の下部 pager、`totalPages <= 1` かつ `page <= totalPages` では何も描画しない（`page > totalPages`〈範囲外ページからの回復〉のときは描画する、PR #40 Codex P1 是正）。`PaginationSummary` = 上部 text-only variant、`totalCount > 0` のとき常時 opt-in。Lane 2 で旧 features/products 配下の `ProductPagination` から移設し 8 caller を一括更新、D-9。Lane 4 で上下切り分けを実装）。perPage 切替（50 / 100 / 200）は呼び出し側ページの `Select` が担う（`src/features/products/ProductListPage.tsx` + `LIST_PER_PAGE_OPTIONS`）。上部 variant（件数 + 現在位置 text のみ、pager ボタンは置かない）は DSR-22 の裁定に基づく追加方針で、一覧すべてに常時適用する。

**構造**:

```tsx
// 上部 variant（PaginationSummary、totalCount > 0 のとき常時表示、DSR-22。件数 + 現在位置 text のみ、pager ボタンは持たない）
<div className="text-sm text-muted-foreground tabular-nums">
  全 {totalCount.toLocaleString("ja-JP")} 件のうち {from.toLocaleString("ja-JP")}〜{to.toLocaleString("ja-JP")} 件を表示（{page} / {totalPages} ページ）
</div>

// ページ送り（component 本体、下部。Pagination.tsx canonical。totalPages > 1、または page > totalPages〈範囲外ページからの回復〉のときだけ描画）
<div className="flex flex-wrap items-center justify-between gap-3 text-sm text-muted-foreground">
  <div className="tabular-nums">
    全 {totalCount.toLocaleString("ja-JP")} 件のうち {from.toLocaleString("ja-JP")}〜{to.toLocaleString("ja-JP")} 件を表示（{page} / {totalPages} ページ）
  </div>
  <div className="flex items-center gap-2">
    <Button variant="outline" size="sm" disabled={!canPrev} aria-label="前のページ" onClick={…}>
      <ChevronLeft aria-hidden="true" />
      前へ
    </Button>
    <span className="min-w-20 text-center font-medium text-foreground tabular-nums">{page} / {totalPages} ページ</span>
    <Button variant="outline" size="sm" disabled={!canNext} aria-label="次のページ" onClick={…}>
      次へ
      <ChevronRight aria-hidden="true" />
    </Button>
  </div>
</div>

// perPage 切替（呼び出し側ページ）
<Select value={String(perPage)} onValueChange={…}>
  <SelectTrigger id="product-per-page" className="w-[7rem]"><SelectValue /></SelectTrigger>
  <SelectContent>
    {LIST_PER_PAGE_OPTIONS.map((o) => <SelectItem key={o} value={String(o)}>{o} 件</SelectItem>)}
  </SelectContent>
</Select>
```

**件数文言**: 範囲付き統一形「全 {n} 件のうち {from}〜{to} 件を表示（{p} / {t} ページ）」（`n` / `from` / `to` は `ja-JP` locale、`tabular-nums`）。`from = (page-1)*perPage+1`、`to = min(page*perPage, totalCount)`。`totalPages <= 1` かつ `page <= totalPages`（`totalCount === 0` を含む）は下部 pager を描画しない。`page > totalPages`（範囲外ページからの回復）のときは描画する（Lane 4、PR #40 Codex P1 是正）。

**使用トークン**: 下部件数・ページ表示は `text-sm text-muted-foreground`（tabular-nums）。上部 `PaginationSummary` も同じ `text-sm text-muted-foreground tabular-nums`（Lane 4 で下部と統一）。ページボタンは outline variant、`size="sm"`。現在ページ表示は `min-w-20 text-center font-medium tabular-nums`。perPage 切替は `w-[7rem]`。例外: 在庫照会の絞り込み時（`list_low_stock` 経路、`totalCount` なしで範囲文言が成立しない）は上部に件数のみを `text-base font-semibold tabular-nums` で出す（58 §UI-06a-D6、2026-09-15。下部との統一は別 lane）。

**状態**:
- **disabled**: 先頭ページで「前へ」、末尾ページで「次へ」を `disabled` にする（`totalPages <= 1` かつ `page <= totalPages`（0 件含む）は下部 pager 自体を描画しない。`page > totalPages` の回復時は描画する）
- hover / focus / active / error: ボタン primitive の既定に従う

**perPage 規約**: 表示件数の選択肢は `LIST_PER_PAGE_OPTIONS = [50, 100, 200]`。リテラル直書きせず定数を参照する。共有定数は 1 本のまま維持し、既定値だけを画面ごとに変える（裁定・owner 了承 2026-09-03: 棚卸しは未入力を潰し切る全走査が主動線のため既定 50、商品一覧は 1 件探索が主動線のため既定 100、Lane 2 実装済み）。40 刻み化（40/80/120 等）は既存の固定文言 test への影響と移行コストに見合わないため不採用の裁定案。

**アクセシビリティ**: 前後ボタンに `aria-label`（"前のページ" / "次のページ"）を付け、方向アイコンには `aria-hidden`。現在地（page / totalPages）はテキストで常時可視にする。下部は複数ページ時、または range 外ページからの回復時のみ描画するため、上部 `PaginationSummary` と下部 `Pagination` の同時描画は `totalPages > 1` または `page > totalPages` のときに限られる（Lane 4、PR #40 Codex P1 是正で回復例外を明記）。

**Do**:
- 端ページで前後ボタンを disabled にする
- perPage は定数（50 / 100 / 200）を参照する
- 一覧はすべて上部に件数 + 現在位置 text を常時出す（DSR-22、Lane 4）

**Don't**:
- 現在ページ・総ページ数をアイコンだけで示さない
- perPage の選択肢をマジックナンバーで直書きしない
- 上部 variant に下部と同じボタン群を無条件で重ねる（選択肢が増え判断コストが上がる、Q12 §1）
- 下部を単一ページ（totalPages<=1）で表示しない

---

## ⑪ 日付・月ナビ

**使いどころ**: 日次 / 月次レポートの期間移動。前 / 次ボタン + ネイティブ日付・月 input + 日本語ラベルの 3 点セット。

**canonical**: `src/features/daily-sales/components/DateNavigator.tsx`（日付）、`src/features/monthly-sales/components/MonthNavigator.tsx`（月）

**構造**:

```tsx
<div className="flex items-center gap-2">
  <Button variant="outline" size="sm" onClick={() => onChange(addDays(date, -1))} aria-label="前日">
    前日
  </Button>
  <span className="min-w-[10rem] text-center text-sm font-medium" aria-live="polite">{label}</span>
  <input
    type="date"
    value={date}
    onChange={(e) => { if (/^\d{4}-\d{2}-\d{2}$/.test(e.target.value)) onChange(e.target.value); }}
    className="rounded-md border border-input bg-control-surface px-3 py-1 text-sm focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
    aria-label="日付を選択"
  />
  <Button variant="outline" size="sm" onClick={() => onChange(addDays(date, 1))} aria-label="翌日">
    翌日
  </Button>
</div>
```

月ナビは `type="month"` + `/^\d{4}-\d{2}$/` 検証 + `prevMonth` / `nextMonth` に置き換える（境界で前後値が無い場合は無操作）。

**使用トークン**: 前後ボタンは outline + `size="sm"`。日本語ラベルは `text-sm font-medium`、最小幅 `min-w-[10rem]`（日付）/ `min-w-[8rem]`（月）。input は `border-input bg-control-surface`。要素間 `space-2`。

**状態**:
- focus: input は `focus-visible:ring-2`
- hover / active / disabled: ボタン primitive の既定（月途中状態を見たい業務要件のため未来日/未来月のガードは置かない）
- error: 入力値が形式に合わなければ無操作（不正値で onChange を発火させない）

**アクセシビリティ**: 前後ボタンに `aria-label`（"前日" / "翌日" / "前月" / "翌月"）。現在期間ラベルは `aria-live="polite"` で変更を読み上げる。input にも `aria-label` を付ける。

**Do**:
- 前後ボタン + ネイティブ input + 日本語ラベルの 3 点を揃える
- 期間ラベルを `aria-live="polite"` にする

**Don't**:
- 期間を数値だけで示し日本語ラベルを省かない
- 形式不正の入力をそのまま onChange に流さない

---

## ⑫ 行インライン展開

**使いどころ**: 一覧で行を選ぶと、その行の直下に詳細を展開する。詳細カードを画面下部に固定するとスクロール往復が増えるため、選択行の直下にインライン展開する（実機デモ起源の方針）。

**canonical**: `src/features/stock-inquiry/components/ProductListTable.tsx`（選択行直下 colSpan 展開）+ `src/features/stock-inquiry/components/StockDetailContent.tsx`（展開内容、loading / error / data 内包）

**構造**:

```tsx
{items.map((item) => {
  const isSelected = item.product_code === selected;
  return (
    <Fragment key={item.product_code}>
      <TableRow
        data-state={isSelected ? "selected" : undefined}
        className={
          isSelected
            ? "cursor-pointer border-l-4 border-l-ongoing hover:bg-ongoing-soft data-[state=selected]:bg-ongoing-soft"
            : "cursor-pointer border-l-4 border-l-transparent"
        }
        onClick={() => onSelect(item.product_code)}
      >
        <TableCell className="font-mono text-sm font-medium">{item.product_code}</TableCell>
        <TableCell>{item.name}</TableCell>
        {/* …状態列 / 在庫数 / 売価… */}
      </TableRow>
      {isSelected && (
        <TableRow className="border-l-4 border-l-ongoing bg-ongoing-soft hover:bg-ongoing-soft">
          <TableCell colSpan={6} className="p-0 align-top whitespace-normal">
            <StockDetailContent query={detailQuery} />
          </TableCell>
        </TableRow>
      )}
    </Fragment>
  );
})}
```

**使用トークン**: 選択行と展開行は左端のバー（`border-l-ongoing`）+ 進行中の地（`bg-ongoing-soft`）で一体に見せる（00 の進行中の役割、迷いやすい場面「詳細を開いた行」、D-094）。選択行の地は `TableRow` の既定と同じ variant（`data-[state=selected]:`）で上書きする（既定の variant 付き class は素の `bg-*` より詳細度が高い）。閉じた行は `border-l-transparent` で列をそろえる。`TableBody` の末尾の規則は `[&_tr:last-child]:border-b-0` に限り、最後の行を開いても展開行の左のバーを消さない。展開行の上の在庫少のセルは `text-warning-strong`（`-emphasis` は進行中の地の上で 4.5:1 に届かない）。展開セルは `whitespace-normal` で table 既定の `whitespace-nowrap` を打ち消し、長い商品名 / CTA 群の横はみ出しを防ぐ。

**状態**:
- **selected（active 相当）**: 選択行に `data-state="selected"` と `data-[state=selected]:bg-ongoing-soft`、展開行に `bg-ongoing-soft` を明示固定する（table primitive の自動トリガに依存しない）。`TableRow` の既定（stone）は選択欄を開いた入力行にも当たるため変えない
- hover: 行は `cursor-pointer`。選択行と展開行は `hover:bg-ongoing-soft` で hover でも色が動かないようにする
- focus / disabled / error: 行自体に規定なし。詳細の error / loading は `StockDetailContent` 側（query 状態）が担う

**フォールバック**: 一覧取得が失敗し選択商品が残る場合は、インライン展開できないため `StockDetailCard`（`StockDetailContent` を `Card` で包む）で独立描画する（部分障害許容）。

**アクセシビリティ**: 選択状態を `data-state` で構造化する。展開行は colSpan で全幅を使い、詳細内の状態（在庫数・売価・最終入庫日）はラベル + 値の対で示す。展開内の将来ボタンは `aria-disabled` + Tooltip で不能理由を伝える。

**Do**:
- 詳細は選択行の直下にインライン展開する
- 展開行の背景色を明示固定し、選択行と一体に見せる

**Don't**:
- 詳細カードを画面下部に固定してスクロール往復を強いない
- 展開セルで横はみ出しを起こさない（`whitespace-normal` を当てる）

---

## ⑬ ステータスバッジ

**使いどころ**: 在庫状態・警告・処理結果・比較状態といった、利用者が業務判断に使うステータスを表示する。色は補助シグナルとし、意味は日本語ラベルと非色シグナルで伝える。

**canonical**: `src/features/stock-inquiry/components/StockStatusBadge.tsx`

**構造**:

```tsx
// status === "stockout"
<Badge variant="outline" tone="destructive" className="font-medium">
  <CircleAlertIcon aria-hidden="true" />
  在庫切れ
</Badge>

// status === "low"
<Badge variant="outline" tone="warning" className="font-medium">
  <TriangleAlertIcon aria-hidden="true" />
  在庫少
</Badge>

// status === "ok"
<Badge variant="outline" className="border-stone-200 bg-stone-50 font-medium text-stone-600">
  在庫あり
</Badge>
```

**badge 3 種構成**（`04-backbone.md` 原則 4 の具体化、新規 DSR は起草しない）: ①状態 = `variant="outline"` + icon + soft tone（tone 固有色の枠、在庫切れ・在庫少・PLU 未反映 等、遷移しうる状態）（非中立 tone は icon 必須、中立 tone は任意）。②分類 = `variant="secondary"` + `--border` 枠（icon は識別に必要な場合のみ、廃番・PLU 対象外・最近改定 等の恒常的な属性）。③強調 = 琥珀 pill（最新・上書き件数は `border-warning bg-warning-emphasis text-primary-foreground`、ランキング 1 位は `border-warning bg-rank-top-badge-bg text-rank-top-badge-text`。badge に既定の塗りは無く、class で指定する。枠色は owner v3 決定で `--warning`、D-094）。この 3 種以外を作らない。

`Badge` の `tone` prop（warning / success / destructive）は `variant` と独立の軸。①状態は `variant="outline"` と組み合わせる。未指定時は `data-tone` と tone class を出さず、②分類・③強調・中立状態には付けない。

tone family は感情で分ける: 緑 = 終わったことを伝えるプラスの報告、琥珀 = 「ちょっと待って」の注意、赤 = 警告。状態の名前を伝えるだけの badge は無色（owner 決定、2026-09-06 Human Gate (1)）。

**①状態の tone family マッピング表**（owner culling 列は Human Gate (1) の回答（原文 2、2026-09-06）を転記済み。file:line は2026-09-08起票時の移行前 anchor。runtime は tone prop へ移行済み）:

| tone family | 該当する状態 badge（file:line、実測文言） | owner culling（残す/外す/追加、原文回答） |
|---|---|---|
| warning（`border-warning-border bg-warning-soft text-warning-strong` + icon） | `src/features/stock-inquiry/components/StockStatusBadge.tsx`「在庫少」（実装済み）/ `src/features/stocktake/StocktakePage.tsx`「未入力 N」（実装済み）/ `src/features/csv-import/components/PreviewStep.tsx`「同日データあり」（実装済み）/ `src/features/products/components/ProductTable.tsx`「未反映」（tone propへ移行済み）/ `src/features/csv-import/components/ResultStep.tsx`「部分成功」（tone propへ移行済み）/ `src/features/inventory-records/CsvImportRecordDetailPage.tsx`「部分成功」（tone="warning"）/ `src/features/stock-inquiry/components/StockRecheckBadge.tsx`「要再確認」（D-118 の b2。商品一覧・在庫照会・商品修正の現在庫・在庫変動の見出しの在庫の欄。tone="warning"） | 原文 2:「終わったから緑で終わったと教えてあげる、これ取り込み済みだよと教えてあげるのは注意に値しそうだけども。あと反映済み橙にしてるなら緑にしちゃうのもありだな、感情的にプラスの報告は緑、ちょっと待ってねって言いたくなるのは琥珀というか橙？警告したいのは赤、みたいな感情の分け方になると思う」 |
| success（`border-success-border bg-success-soft text-success-strong` + icon） | `src/features/integrity-check/IntegrityCheckPage.tsx`「補正済み」（soft success toneへ移行済み）/ `src/features/stocktake/StocktakePage.tsx`（棚卸し全数完了時のsoft success tone）/ `src/features/products/components/ProductTable.tsx`「反映済み」（success toneへ移行済み）/ `src/features/csv-import/components/ResultStep.tsx`「成功」（success toneへ移行済み）/ `src/features/daily-report-import/DailyReportImportPage.tsx`「確認済み」（success toneへ移行済み） / `src/features/inventory-records/CsvImportRecordDetailPage.tsx`「成功」（tone="success"） / `src/features/daily-report-import/DailyReportImportPage.tsx` の完了の「成功」（tone="success"） | 原文 2:「終わったから緑で終わったと教えてあげる、これ取り込み済みだよと教えてあげるのは注意に値しそうだけども。あと反映済み橙にしてるなら緑にしちゃうのもありだな、感情的にプラスの報告は緑、ちょっと待ってねって言いたくなるのは琥珀というか橙？警告したいのは赤、みたいな感情の分け方になると思う」 |
| destructive（`border-destructive-border bg-destructive-soft text-destructive-strong` + icon） | `src/features/stock-inquiry/components/StockStatusBadge.tsx`「在庫切れ」（実装済み）/ `src/features/inventory-records/CsvImportRecordDetailPage.tsx` の `STATUS_LABELS` と状態の Badge「取消済み」（destructive toneへ移行済み）・明細の Badge「明細取消済み」（destructive toneへ移行済み） / `DailyReportImportPage.tsx`「取込み済み」（上部の Alert と同じ destructive tone、L3 round 1 で試しを採った、D-094） | 原文 2:「終わったから緑で終わったと教えてあげる、これ取り込み済みだよと教えてあげるのは注意に値しそうだけども。あと反映済み橙にしてるなら緑にしちゃうのもありだな、感情的にプラスの報告は緑、ちょっと待ってねって言いたくなるのは琥珀というか橙？警告したいのは赤、みたいな感情の分け方になると思う」 |
| 中立（家族なし、`variant="outline"` の既定枠色、無彩色 soft（`bg-stone-50` 等）可・icon 任意） | `src/features/stock-inquiry/components/StockStatusBadge.tsx`「在庫あり」（実装済み、`border-stone-200 bg-stone-50 text-stone-600`、icon なしで準拠）/ `src/features/inventory-records/types.ts` の `formatRecordStatus`（`active`="有効" 等、複数の記録詳細ページで共有、owner culling で個別確認） | 原文 2:「終わったから緑で終わったと教えてあげる、これ取り込み済みだよと教えてあげるのは注意に値しそうだけども。あと反映済み橙にしてるなら緑にしちゃうのもありだな、感情的にプラスの報告は緑、ちょっと待ってねって言いたくなるのは琥珀というか橙？警告したいのは赤、みたいな感情の分け方になると思う」 |

**表から除外した項目とその理由**: `src/features/integrity-check/IntegrityCheckPage.tsx` の違う項目の Badge（`fieldsLabel()`。「在庫数」「要再確認」「範囲」を「・」でつなぐ）は、違う項目の名前を示すだけの `variant="outline"` の無色の Badge（tone なし）で①状態ではないため、テーブルには含めない（D-118 の (a) で、前の差の向きの 3 値の Badge を置き換えた）。「入力中」（`src/features/products/components/PriceRevisionTable.tsx`）は `04-backbone.md` 原則 10「いま扱っているものは進行中で示す」（現在行の 3 点）の対象であり、tone family の対象外（下記クロスリファレンス参照）。「対象外」（`src/features/products/components/ProductTable.tsx`）は廃番と同じ恒常的属性のため②分類 note へ移す。「有効」（`src/features/inventory-records/CsvImportRecordDetailPage.tsx`）は Badge ではなく `<span className="text-muted-foreground">` の plain text のため除外。「レジ未処理」（`src/features/return-exchange/ReturnExchangePage.tsx` の `formatRegisterProcessed`（formatter）/ radio の `aria-label` / radio のラベルの `<span>`）も plain text の radio ラベルであり Badge ではないため除外（隣接する実際の Badge「この保存で反映」は owner 承認済みの現状維持、Non-scope）。

**②分類の note**（枠は `--border`、tone family 表とは別建て）: 廃番（`src/features/products/components/ProductTable.tsx` 等）・PLU 対象外（`src/features/products/components/ProductTable.tsx`）・最近改定（`src/features/products/components/PriceRevisionTable.tsx`）・手動（`src/features/daily-sales/components/ProductTable.tsx`、出どころの分類）は恒常的な属性・分類ラベルであり、`variant="secondary"` + `--border` 枠が正しい形（`badge.tsx` の `secondary` に枠を実装済み）。

**③強調の note**（枠色 `--warning`、owner v3 決定済み）: 最新（`BackupRestorePage.tsx`）・上書き件数（`ProductImportPreview.tsx`）は `border-warning bg-warning-emphasis text-primary-foreground`（琥珀 pill。塗りの上の文字 4.81:1、枠は対 `--background` 3.05:1）。押すボタンでない badge の塗りは③強調だけの恒久の例外で、段 2 に数える（00 の強調の段階、D-094）。1 位（`ProductRankingTable.tsx`）は `border-warning bg-rank-top-badge-bg text-rank-top-badge-text` の琥珀 pill（薄い地で塗りではない。枠は対 fill `#fef3c7` 2.86:1・対 `--background` 3.05:1）と行の `bg-rank-top-bg/40` で示す。順位と太字だけにする試しは L3 で採らなかった（D-094。2 位・3 位を含む見せ方は別 lane、`docs/backlog.md`）。

**非Badge除外のクロスリファレンス**: 「入力中」（`src/features/products/components/PriceRevisionTable.tsx`）は [04-backbone.md](04-backbone.md) 原則 10「いま扱っているものは進行中で示す」（現在行の 3 点）の対象であり、tone family 表・②分類・③強調のいずれにも含めない。

**関連**: DSR-08（semantic 色のみで意味を伝えない）/ 04-backbone 原則 10（現在行 3 点、`入力中` の帰属先）/ [01-decision-rules.md](01-decision-rules.md) DSR-22（枠の narrow 化、badge 枠線必須・非中立①状態 icon 必須）。

標準パターン:

- `Badge` + `lucide-react` アイコン + 日本語ラベルを第一候補にする。例: `CircleAlert` + `在庫切れ`、`TriangleAlert` + `在庫少`
- テーブル内の状態は、列追加がレイアウトを壊さない場合は状態列を優先する。列追加で商品名が読みにくくなる場合は、在庫数セル内ラベルや左罫線などの非色シグナルを検討する
- tooltip は補足に限る。tooltip だけに意味を閉じ込めない
- 新規 package は追加しない。現行の shadcn/Radix primitive、Tailwind、`lucide-react` で実装できる範囲を優先する
- テストは色クラスだけを assert しない。`在庫切れ` / `在庫少` などの text、role、label、値の invariant を assert する

**使用トークン**: 状態色は semantic shade token（`00-foundations.md`）で当てる。`在庫あり` は `border-stone-200 bg-stone-50 text-stone-600`（stone は palette 内で直書き可）、`在庫少` は `border-warning-border bg-warning-soft text-warning-strong`、`在庫切れ` は `border-destructive-border bg-destructive-soft text-destructive-strong`。soft 背景 + border + 濃いめテキストの 3 点セットで Badge outline を構成する。

> **是正済み（PR-C）**: 旧逸脱（`StockStatusBadge.tsx` / `ProductListTable.tsx` / `StockDetailContent.tsx` の `rose-` / `amber-` 直書き）は上記 semantic shade token へ移行済み（rose→red は意図的色補正、L3 承認）。生 Tailwind 色の再混入は eslint palette 外色 ban が機械防止する。

**状態**: バッジ自体は状態（stockout / low / ok）ごとに見た目を切り替える。hover / focus / active / disabled は規定なし。

**アクセシビリティ**: 状態は日本語ラベル（`在庫切れ` / `在庫少` / `在庫あり`）+ アイコン形状で示し、色だけに依存しない（WCAG 1.4.1）。アイコンには `aria-hidden`、意味はテキストが担う。実利用者が赤黄を識別できない場合でも意味が読める実装にする。

**Do**:
- Badge + アイコン + 日本語ラベルで状態を示す
- テストは text / role / label / 値の invariant を assert する

**Don't**:
- 状態を色（hue）だけで符号化しない（DSR-08）
- 意味を tooltip だけに閉じ込めない
- secondary（②分類）を①状態の soft tone 代わりに使わない

---

## ⑭ FilePicker

**使いどころ**: ファイル選択を伴う入力（CSV取込み、商品マスタ一括インポート、返品・交換のレシート画像添付）で使う統一 UI コンポーネント。**behavior / API 契約（入口 2 経路の詳細・`PickedFile` 出力契約・cancel 挙動・「local input / dropzone 新設禁止」の Do/Don't）の正典は [UI_TECH_STACK.md §6.5.4](../UI_TECH_STACK.md#654-ネイティブダイアログ)**（D-054、WebView2 の HTML file input 白画面バグ回避、cross-language validation 定数 SSOT）。本節は構造 / トークン / 状態 / a11y / 採用箇所のみを扱い、behavior / API 契約は二重記述しない（round 1 P2-2 対応）。

**canonical**: `src/components/FilePicker.tsx`（`patterns/` 配下ではなく `components/` 直下に置く。plugin-dialog / plugin-fs 副作用を持つため純表示部品規約〔[59-ui-shared-patterns.md](../function-design/59-ui-shared-patterns.md) §59.4〕の対象外、詳細は同 §59.3 参照）

**構造**:

```tsx
<FilePicker
  accept=".csv"
  ariaLabel="CSVファイルを選択"
  buttonLabel="ファイルを選択"
  onSelect={(file: PickedFile) => { /* file = { bytes, filename, size }。契約詳細は §6.5.4 参照 */ }}
  onError={(message) => toast.error(message)}
/>
```

**使用トークン**: 選択ボタンは `variant="outline"`。dropzone は `rounded-lg border-2 border-dashed p-8 text-center`、drag over 時 `border-primary bg-primary/5`、既定 `border-muted-foreground/30`。dropzone アイコンは lucide `Upload`（`size-8 text-muted-foreground`）。

**状態**:
- **disabled**: `disabled` prop でボタン・dropzone とも操作不能にする（dropzone は `cursor-not-allowed opacity-50`）
- **エラー**: エラー表示はボタン / dropzone 直下（呼び出し側の表示スロット）に置く。`onError` の意味論・未指定時のフォールバック文言は behavior / API 契約であり [UI_TECH_STACK.md §6.5.4](../UI_TECH_STACK.md#654-ネイティブダイアログ) が正典（round 2 P2-4 対応、本節は visual のみ）
- hover / focus / active: ボタン primitive の既定に従う

**アクセシビリティ**: ボタンは `ariaLabel` prop（必須）で識別する。dropzone のアイコンは `aria-hidden`。

**採用箇所**: 件数を断定せず、検索式 `rg -n --glob '!**/*.test.tsx' '<FilePicker' src` で実測する。2026-08-03 時点の列挙:
- `src/features/csv-import/components/FileDropzone.tsx`（CSV取込み、初期選択）
- `src/features/csv-import/components/PreviewStep.tsx`（CSV取込み、プレビュー段階の選び直し）
- `src/features/products/import/ProductImportDropzone.tsx`（商品マスタ一括インポート、初期選択）
- `src/features/products/import/ProductImportPreview.tsx`（商品マスタ一括インポート、プレビュー段階の選び直し）
- `src/features/return-exchange/ReturnExchangePage.tsx`（返品・交換、レシート画像添付）

---

## ⑮ 商品追加欄 live 候補プレビュー（ProductAddSuggest）

**使いどころ**: 取引 4 画面（入庫 61 / 手動販売 62 / 返品・交換 63 / 廃棄・破損 64）+ 棚卸し（73）の商品追加欄。commit 型（Enter 一挙動追加、UI-02-D4 系列 5 契約）を維持したまま、入力中に候補一覧を目視補助として表示する。⑨ SearchBar（一覧 filter 用）とは用途が異なる別 pattern — SearchBar は検索結果が一覧そのものを差し替える filter であり、⑮ は「明細追加という確定アクション」の前段のプレビュー。適用 5 site は UI-02-D14 / UI-04-D16 / UI-03-D21 / UI-05-D16 / UI-10-D12 を正とする。商品一覧・在庫照会・入出庫履歴の SearchBar live 型、および Enter commit 後の既存複数件候補テーブルは明示除外とする（Plan Packet は作業証跡であり durable manifest はここに置く）。

**canonical**: [`ProductAddSuggest.tsx`](../../src/components/patterns/ProductAddSuggest.tsx) + hook [`useProductAddSuggest.ts`](../../src/components/patterns/useProductAddSuggest.ts)（`components/patterns/` 配下の実装正本）

**採用の背景（variant B）**: 純 autocomplete 型（Enter 追加廃止）は、スキャナ Enter が候補 async 読込み前に届くため退行となり不採用（owner 方針 2026-08-04）。`cmdk` 等の新規 npm 依存は供給網防御（D-030）下の設計判断として不採用。安全性は下記 D2/D4 の software contract により supported sequence「バーコード文字列 + Enter」の範囲で timing 非依存に成立し、スキャナ物理挙動（方向キー有無・keystroke 間隔）への無条件保証は主張しない。

**契約（SPEC-SUGGEST-D1〜D12、凍結正本。D11 は 2026-08-09 gated Amendment 3〈PR #65 初回 L3 の IME blocker〉で追加、D12 は 2026-08-11 JAN 専用欄正規化 change で追加。D1〜D10 本文不変）**:

- D1（二層構造・不干渉）: live 候補プレビューは表示専用の追加層である。既存 Enter commit 経路（検索実行・0/1/複数分岐・行追加・focus 復帰・IME guard）のロジックは変更しない。suggest 層の失敗（fetch error 含む）は commit 経路へ波及せず、候補非表示に縮退する。
- D2（Enter 分岐）: 候補リストに active 候補（aria-activedescendant が指す行）が存在する場合のみ Enter は候補確定として動作する。active 候補は ↓/↑ キー操作によってのみ生成される（表示直後の自動 active 化・先頭行自動選択は禁止）。active 候補なしの Enter は常に既存 commit 経路を実行する。候補リストの内容が更新された場合（debounce 再 fetch による差し替え）、直前の active 候補は必ず解除し持ち越さない（更新直後の Enter が意図しない行を確定することを防ぐ）。新しいリストで active を得るには再度 ↓/↑ の操作を要する。さらに、入力値が変化した onChange event の時点で active 候補を同期的に解除する。同時に表示中のリストを close し、新しいリストは現在入力値に対する最新結果の採用後にのみ open する（旧リストの表示維持・click 操作は行わない — pointer 経路の stale 確定を構造的に排除）。この同期解除 + close により、Enter commit 経路の到達はスキャナ timing・debounce 残量に依存しない。
- D3（発火条件）: debounce 200ms（TRACE-D12 と同値）。入力 1 文字以上で発火。取得は per_page 5、総件数超過時は候補末尾に「ほか N 件（候補未選択で Enter: 従来の検索）」を表示する。0 件時は非表示（メッセージなし。0 件文言は既存 commit 経路の所掌）。
- D4（破棄条件）: suggest fetch は sequence token で直近要求のみ採用する。sequence generation は各入力変更時（debounce 開始前）に更新し、応答は token と検索語が現在入力値の双方に一致する場合のみ採用する。Enter commit 実行・入力欄 clear・候補確定・lock 成立・Esc・blur / Tab・unmount の各時点では、リスト / active の close に加え、pending debounce timer を cancel し、generation を進めて in-flight 応答（success / error とも）を不採用にする。保存 lock との整合は D10 の per-画面 lock source 契約に従う（disposal のみ UI-05-D15 の lock ref、取引 3 画面〈receiving / manual-sale / return-exchange〉は各画面の既存 `isFormLocked` 派生 state、棚卸しは既存 `isCompleting`〈`completeMutation.isPending`〉派生 state を単一 source とする）。
- D5（IME）: suggest 層の onKeyDown は冒頭で `event.nativeEvent.isComposing` を判定し、true の間は Enter / ↓ / ↑ / Esc を含む suggest キー処理全体を行わず IME に委ねる（変換候補操作の方向キーで active を生成しない。既存 commit 経路の Enter guard も従来どおり維持）。onChange / debounce 経路に composing guard は置かず、変換途中文字列での候補更新を許容する（SearchBar live 型の既定意味論 = PR #61 P1-2 裁定 a と同一）。
- D6（キーボード・a11y）: input は `role="combobox"` + `aria-expanded` + `aria-controls` + `aria-activedescendant`、リストは `role="listbox"`、行は `role="option"`。focus は常に input が保持する（リストへ focus 移動しない）。↓/↑ で active 移動（端で wrap しない）、Esc で close + active 解除、blur / Tab で close。候補行のクリックは active の有無に関わらず当該行の即時候補確定として扱い、D7 の同一 handler を呼ぶ（`onMouseDown` 時点で default を抑止し、input の blur による close との race を防ぐ）。マウス hover（mouseenter 等）は active 候補を生成しない（active 生成は D2 の ↓/↑ 経由に限定）。footer 行（「ほか N 件…」）は `role="option"` を持たない非選択の装飾行であり、↓/↑ による active 移動の対象外とする。footer 表示中に active 候補なしで Enter を押した場合の挙動は D2 の既定分岐（既存 commit 経路の実行）と同一であり、footer 文言の「候補未選択で Enter: 従来の検索」はその結果（既存 commit 経路へ戻ること）を指し、全一致件数の表示を保証しない。
- D7（候補行表示・確定 semantics）: 候補行は商品コード + 商品名 + 部門名の統一 3 項目（画面固有列は出さない）。候補確定時の挙動は当該画面の既存「複数件候補テーブルからの選択」と同一 handler を通す。
- D8（棚卸し）: 棚卸しの suggest fetch は `searchProducts`（部分一致）を用い、`queryOverrides: { is_discontinued: null }`（D13）で廃番商品も候補に含める。候補確定は既存 UI-10-D2 の `find_stocktake_item` 経由で棚卸し対象化する。UI-10-D11 の focus 遷移契約（解決成功で数量欄へ）は候補確定経由でも同一に発火する。候補確定後に `find_stocktake_item` が稀に `None` を返す場合は既存 `selectCandidate` の無言 no-op 挙動をそのまま継承する（既知の pre-existing gap、本 change の scope 外）。
- D9（実装形態・依存）: プレビュー層は新規共通 component + hook として 1 箇所に実装し 5 画面へ配線する。新規 npm 依存は追加しない（cmdk 不採用。radix Popover も不使用 — focus 非奪取要件から素の絶対配置要素で実装）。既存 5 画面の commit 経路コードの共通化・移動は行わない。
- D10（無効化条件）: form 保存中 lock / disabled 状態では suggest fetch を発火せず、表示中リストは close する。suggest 層は各画面の既存 lock 状態を単一 source として受け取り、新規の lock 機構を作らない。disposal では UI-05-D15 の lock ref がその source である（二重 lock の相互不整合を構造的に排除する）。catalog ⑮ の component / hook API は、候補操作・timer・応答採否の全てが読む同期 `isLocked(): boolean` と、保存 event から同期呼出しされる `invalidateAndClose()` を持つ。disposal は UI-05-D15 の lock ref を更新した同じ event 内で `invalidateAndClose()` を呼ぶ（これは新規 lock source ではない）。
- D11（IME 全角数字の正規化と compositionend commit、2026-08-09 追加）: 日本語 IME 有効時、HID スキャナのキー入力は IME に変換されて全角数字の composition となり、末尾 Enter は変換確定に消費される（Windows WebView2 + BC-BR1000U-W 実測）。suggest 層はこれを次の契約で吸収する。(a) 正規化 = 入力値全体が `[0-9０-９]+` に一致する場合に限り、全角数字（U+FF10〜U+FF19）を ASCII 数字へ文字写像する。英字・かな・記号を含む値は一切加工しない。NFKC 等の汎用正規化・trim 変更・連結コードの自動分割・チェックディジット自動修正・スキャナ/手入力の識別は行わない。composition 中（isComposing true）は値を加工しない。(b) 発火点 = component の onCompositionEnd。確定値が (a) の全体一致を満たし、かつ `isLocked()` が false の場合のみ、正規化値で既存 onChange 契約を 1 回発火し、続けて `onComposedDigitsCommit(normalized: string)`（画面側が既存の検索/解決関数を接続する optional prop）を 1 回呼ぶ。既に半角数字のみの確定値でも commit は発火する（IME 半角英数モードの変換確定を同様に吸収）。commit 発火時は D4 の close 系 event と同様に `invalidateAndClose()` を同時に呼び、pending debounce timer と表示状態を破棄する（Final Review 再走 P3-1 の明文化、2026-08-09）。(c) commit は既存 fallback 経路のみを通し、候補確定（selectSuggestion）を経由しない（composition 中は D5 により ↓/↑ が委譲され active は生成されないため、active なしが正常状態）。(d) 二重発火抑止 = compositionend 起点 commit の直後に isComposing=false の Enter keydown が届いた場合に commit が二重発火しない one-shot guard を置く（次の入力変更または別 keydown で解除）。実 IME のイベント順は RTL harness で再現できないため、guard の実機挙動検証は Windows native L3 の所掌とする。(e) 画面側の接続契約 = `onComposedDigitsCommit` を受ける画面の検索/解決関数は正規化済み文字列を optional 明示引数で受け取る（state の暗黙参照は同一 tick の stale closure を生むため禁止）。これは D1/D9 の「既存 commit 経路のロジック不変」に対する最小の追加拡張であり、引数省略時の挙動は従来と完全同一とする。paste 経由の全角数字は composition を経ないため本契約（D11）の対象外であり、D12（SPEC-SUGGEST-D12）が非 composition の onChange 経路で正規化する（既知の制限は 2026-08-11 に解消）。
- D12 = SPEC-SUGGEST-D12（paste 経由全角数字の正規化、2026-08-11 追加）: component は composition 中でない onChange について、入力値全体が `[0-9０-９]+` に一致する場合のみ D11 (a) と同一写像の正規化値で処理する（paste 経由を含む）。正規化値は親 onChange と suggest controller の `onInputChange` の双方へ同一値で渡す（片側だけ raw 値を渡すと、親 state は正規化済みなのに候補 fetch が旧全角値で走る — これを禁止する）。半角のみの値は写像で同値のため既存挙動不変。composition 中は D11 どおり不加工。D1〜D11 の契約意味論は変更しない。
- D13 = SPEC-SUGGEST-D13（画面別クエリ上書き、本 PR 追加）: hook は optional `queryOverrides?: Partial<ProductSearchQuery>` を受け取り、suggest fetch のクエリを `{ ...PRODUCT_SEARCH_QUERY, ...queryOverrides, keyword }` としてマージする。`queryOverrides` 未指定時は `PRODUCT_SEARCH_QUERY`（D3 の `per_page 5` を含む既定値）のみが使われ、既存 5 画面の挙動と byte-identical のまま変わらない。棚卸し（UI-10-D2）は `{ is_discontinued: null }` を渡し、名前検索候補に廃番商品を含める（棚卸し item 母集団に廃番が含まれる契約、§73.1、との整合）。取引 4 画面（入庫 61 / 手動販売 62 / 返品・交換 63 / 廃棄・破損 64）は `queryOverrides` を渡さず、廃番を候補から除外する既存業務判断を維持する。D1〜D12 の契約意味論（二層構造・Enter 分岐・IME・破棄条件等）は変更しない。

**Do**:
- 候補確定は既存 handler（各画面の複数件候補テーブル選択と同一）へ委譲する
- 入力値が変わったら active 解除 + リスト close を同期的に行う（D2）
- close 系 event では timer cancel + in-flight 不採用まで行う（D4）

**取引 4 画面の複数件候補テーブルと明細の先頭の列**（D-138）: 入庫・返品交換・手動販売・廃棄破損の Enter 経路の複数件候補テーブルと明細テーブルは、先頭の列を「商品コード・商品名・部門・現在庫」にそろえ、画面固有の列（単位・数量・原価・方向・種別・理由・販売金額・売価・操作）はその後ろに置く（Fluent 2 *Coherent*）。live 候補プレビューの 3 項目（D7）は変えない。棚卸し（73）の候補と一覧は対象外（画面の形は D-118 の (b) が持つ）。今の build の入庫は候補と明細に現在庫の列が無く、明細に部門の列が無い（[61](../function-design/61-ui-receiving.md) §61.5）。

**Don't**:
- 表示直後・hover での自動 active 化（スキャナ race 防御の核を壊す）
- 旧リストの表示維持・click 許可（stale 確定の再導入）
- 既存 commit 経路のロジック・focus 復帰・IME guard への変更（D1 違反）
- 新規 npm 依存の追加（D9 違反）

---

## ⑯ 一覧の器（ListShell）

**使いどころ**: pagination を持つ一覧画面（商品一覧 / 在庫照会 / 一括価格改定 / 棚卸し / 整合性チェック / 入出庫履歴 等）の共通の器。toolbar・件数表示・table・pager をこの構成でまとめる。適用範囲・発動条件（viewport 超過時のみ上部表示 / 横 overflow 時のみ識別列固定 / 画面→固定列 mapping）は DSR-22 を正本とし、本節では重複記載しない。

**canonical**: `src/components/patterns/ListShell.tsx`（Lane 2 実装、商品一覧を pilot 採用）。mockup D [`mockup-d-lists.html`](reference/mockup-d-lists.html) をお手本として置く。

**必須構成（6 項目。適用条件は DSR-22 を正本とする）**:

1. **toolbar 2 段**（検索条件 / 並び替え・件数を段で分け、`rounded-lg border bg-card p-4`（2 段時は段間 `space-y-3`）の 1 箱に入れる。検索欄を持たない画面は 1 段でよい。`toolbar` 省略時は枠なし）。node 構造の 2 段であり視覚行数ではない（PLU 一括操作 block は `basis-full` で独立行、125% 以上では視覚 3 行以上になりうる）。記録済み逸脱: mockup の `--card` は #fff、runtime は 00-foundations の #f5f5f4。toolbar 箱 #f5f5f4 / sticky 帯・th #e7e5e4 / ページ地 #fafaf9 の 3 段（1.04:1 / 1.15:1 / 1.20:1、Gated Amendment 2 S11 で分離）になる。`--card` の見直しは全画面波及のため Lane 3〜5 候補、L3 で owner が判定（Final Review round 1 P1-1）。操作面は `--control-surface` #fafaf9（Gated Amendment 7 S46）で箱 #f5f5f4 と区別する
2. **上下の件数・現在位置**（上部は `PaginationSummary`、範囲付き統一形「全 {n} 件のうち {from}〜{to} 件を表示（{p} / {t} ページ）」の text 表示。下部は `Pagination`、同文言 + pager フル装備。上部は `totalCount > 0` のとき常に、下部は `totalPages > 1`、または `page > totalPages`（範囲外ページからの回復導線）のときに描画する、Lane 4）
3. **sticky header**（`<table>` は単一（header を別 table に分けない）。件数行（`bg-background`、線なし）と `thead`（`--list-head` 面 + 2px 下線）を page 地を挟まず垂直に隣接させ、同一 inset（`px-2`）で隣接させる（sticky は `thead` のみ、帯は箱の外で構造的に固定）。灰色面は列見出しのみ（owner L3 run 4、Gated Amendment 5 S39）。件数行の上端の線は owner run 6 で撤去（Gated Amendment 7 S47）。記録済み逸脱: mockup `.tbl` の外枠（border + radius + `overflow:hidden`）は runtime では付けない（`overflow-hidden` が sticky を殺す、Plan Review round 2）。inset 値は mockup 12px / runtime 8px で、揃えるのは帯と thead の相互一致であって絶対値ではない）。table の wrapper は `w-min min-w-full`（横 overflow 時に table 幅へ追随、非 overflow 時は 100%）。件数行は箱の外・root 直下に置く（Lane 4 Gated Amendment 4）。帯文言の溢れは子 `PaginationSummary` で「…」にする（追補 S17）。列見出し面の左右上は `rounded-md`（owner run 5、Gated Amendment 6 S43）。列見出し面は th cell 背景 + tr 背景（最小幅で cell 間に出る subpixel seam を塞ぐ）+ corner mask（`list-shell-sticky` hook、角丸維持、owner run 6、Gated Amendment 7 S48）。商品一覧では帯は箱の縦横 scroll に対して固定（page scroll では追従、GA4a）。table の wrapper がさらに専用 scroll 箱（`max-h-[calc(100vh-6.75rem)] overflow-auto`）に包まれ、箱が横に常時・箱の高さを内容が超えたときのみ縦に scroll する（`<main>` は toolbar 分を縦に page scroll する、Lane 4 Gated Amendment 4）
4. **識別列 opt-in**（固定対象の画面→固定列 mapping は DSR-22 を正本とする。横スクロール時は固定列右端に影。Lane 2 では `identityColumns` prop を予約するのみで描画には影響しない、実装は Lane 3〜5）
5. **現在行 3 点**（左端バー + 淡い背景 + badge/文言、DSR-22。token は Lane 2 で提供、消費は Lane 3〜5）
6. **読込みは `ListSkeleton`**（04 原則 7、既定 skeleton。`isLoading` のとき children の代わりに描画）

**使用トークン**: `--border-strong`（操作枠、`--input` が参照）/ `--list-head`（`thead` surface、列見出しのみ、Gated Amendment 5 S39 で件数行は `--background` へ）/ `--background`（件数行の地色〈S47〉+ 列見出し corner mask の覆い色〈S48〉）。3 token とも `globals.css` に実装済み、値は [00-foundations.md](00-foundations.md) カラーパレット表を正本とする。

**状態**: 上部 pager の発動条件は DSR-22 を正本とする。読込みは `ListSkeleton`、空は既存 `EmptyState`（04 原則 7）。

**アクセシビリティ**: 現在地（page / totalPages）はテキストで常時可視にする。sticky header・固定列の影は視覚のみの補助であり、screen reader の読み順は table の DOM 順に従う。

**Do**:
- 必須構成 6 項目を満たす（発動条件は DSR-22 を正本とする）
- perPage 共有定数は 1 本のまま、既定値だけ画面ごとに変える（DSR-22 裁定）
- 上部・下部とも範囲付き統一形「全 {n} 件のうち {from}〜{to} 件を表示（{p} / {t} ページ）」を使う

**Don't**:
- DSR-22 の発動条件外で上部 pager や識別列固定を無条件適用する
- 範囲付き文言を component / caller / test のいずれかで不整合のまま部分導入する

**関連**: DSR-22「一覧の器・現在行・UI 部品枠のコントラスト」（適用条件・画面 mapping の正本）。パターン③テーブル / ⑩ページネーション / ⑥空状態・エラー・ローディング（`ListSkeleton`）。

---

## 小さい文字・密な表への対応（横断）

- 局所的な `text-lg` 化を初手にしない。行高、列幅、`min-w-0`、`truncate`、折り返し可否、主要値の回復手段を同時に設計する（DSR-12）
- 表示サイズ / webview zoom は全画面横断の別設計とする。Tauri capability、永続化、月次/日次など密画面の L3 を含めて扱う（DSR-13）
- 2026-06-07 H-6 feedback: daily 5 画面通し確認で、商品コードは小さいが他の視認性問題はないと確認。商品コード readability は Phase 2 blocker にせず、全体文字サイズ / 表示スケール option と合わせて調整し、表示の大きさの option（`src/components/layout/useDisplayScale.ts`、PR #77）で対応した
- 2026-06-07 follow-up: 商品コードは UI-06a / UI-09a の table cell で最小級 `text-xs` から通常 table text に上げる。全体表示は Sidebar footer の 3 段階 WebView zoom（`standard=1`, `large=1.15`, `extra_large=1.3`）で扱い、保存先は `localStorage`（`inventory.displayScale.v1`）とする。UI-11a/b/c の設定画面契約へ DB-backed に移すかは Phase 4 側で別途判断する

---

## 更新履歴

| 日付 | PR | 内容 |
|---|---|---|
| 2026-10-24 | 画面ごとの要素の見直し design lane（D-138、proposed・未実装） | ⑥ 空状態の説明を選んでいる条件に合わせる規則と filter-empty の採用箇所の統合の後の数、⑧ 廃番の確認の説明の画面名、⑨ 商品を探す検索欄の当たる列、⑮ 取引 4 画面の複数件候補と明細の先頭の列。 |
| 2026-09-28 | 色と強調の反映（decision-log D-094） | ⑥ destructive の Alert を warning と対称の 4 点構造にし、三角 icon を部品が描く形を記した。⑫ 選択行と展開行を進行中のバーと地へ。⑬ ③強調を `--warning-emphasis` の塗りの琥珀 pill へ（1 位は `--rank-top-*` の琥珀 pill と行の地のまま）、日報の取込み済みの badge を destructive の行へ。① ⑤ ⑧ ⑯ の操作の色と旧現在行背景の記述を直した |
| 2026-09-24 | デザインの決まりの組み直し | 冒頭に「本書は現行実装の canonical で、00 の移行列が runtime lane 待ちとする項目は 00 / 04 / 01 を狙いとして読む」旨を追加。04 の原則の統合に合わせ、旧原則 15（現在行 3 点）の参照を原則 10 へ、旧原則 11（待ち時間の顔）の参照を原則 7 へ付け替え（原則 4 は番号不変）。部品の記述は変えない |
| 2026-09-15 | 表示小修正 batch 2 | ⑥ AlertTitle の共通 weight を600、⑬ 在庫状態の中立 Badge を「在庫あり」に同期。 |
| 2026-09-11 | PR #50 | ⑲ Gated Amendment 4: owner L3 run 1 の是正（trigger icon / 固定帯） |
| 2026-09-11 | PR #50 | ⑲ Gated Amendment 3: 未解決時文言。 |
| 2026-09-10 | PR #50 | 取引先ピッカーの canonical 実装 path と起動ボタンの現在値表示を反映。 |
| 2026-09-10 | PR #49 | ⑧ に非確認 Dialog の使いどころ・形式行と picker dialog 小節を追加。自動選択 A/B を比較し、dialog 重ねの推奨・fallback と runtime lane の WebView2 実機確認義務を記録。closure 是正で a11y 規定の引用射程と検索 0 件時のクリア操作を明記。owner Human Gate round 1（2026-09-10）を反映し、A 案・二重 scrim・選択見出し sr-only・追加ボタン primary + Plus・現在の選択の固定帯を確定。closure 3・owner round 2 を反映し、footer 左の動作記述を primary に統一、Plus を inline SVG と指定し、固定帯の小見出しを明記。 |
| 2026-09-09 | PR #46 | ⑮ S5 備考「—」への同期。MovementTable の truncate 記述を修正前の理由説明へ更新。 |
| 2026-09-09 | PR #46（本 PR） | owner L3 round 1 / Gated Amendment 2: PageHeader (c) を items-start + 左 group min-w-0 flex-1 + actions shrink-0 に変更し、長い description でも actions を右上に留める構造へ同期。 |
| 2026-09-08 | PR #45 | owner L3 AC-L3-3 を受け live SearchBar を `grid gap-1` の Label 上置きへ変更し、呼び出し側 toolbar は `items-end` で入力欄の下辺を揃える。 |
| 2026-09-08 | PR #45 | Badge tone prop / 分類・強調枠、live SearchBar の Label と wrapper、Alert warning の runtime 反映を同期。取込み3状態の tone と移行前 anchor を訂正。既存更新履歴は維持。 |
| 2026-09-06 | UI 磨き batch 3 design | ① にページ説明セクション使用パターンと `PageHeader` の `actions`/`subtitle` 排他 component gap（5画面）を追加。③ に備考列規則（必須列・空欄「—」統一・truncate+`title`）・「直近 {N} 件の」文言統一・`ManualSalePage.tsx` 二重囲み是正方針・共通 formatter（`formatStockDisplay`/`formatStockUnitLabel`）使用ルールを追加 |
| 2026-09-05 | UI 一覧の背骨 D — Lane 4 | ⑩ 上部 `PaginationSummary` を `text-sm text-muted-foreground tabular-nums`（下部と統一）へ、`totalCount > 0` のとき常時表示・pager ボタンなしへ改訂。下部 `Pagination` は `totalPages <= 1`（0 件含む）で描画しない契約を追記。⑯ 必須構成 2 を「上部は totalCount > 0 で常時、下部は totalPages > 1 のときだけ」へ改訂（必須構成 3 の wrapper は無変更） |
| 2026-09-05 | 本 PR | UI 規約補強 design batch。① Do bullet を 3 段 CTA 表現へ同期。⑥ に `Alert` `warning` variant（`bg-warning-soft`+`border-warning`+`AlertTriangle`+`text-warning-strong` の 4 点構造、owner v4 決定）を新設し適用先候補 `PriceRevisionPage.tsx:112-116` を記録。⑨ アクセシビリティ節を owner C1 決定へ書き換え（live 型は可視 Label のみを accessible name とし `aria-label` 廃止、WCAG 2.5.3）。⑬ に badge 3 種構成と①状態 tone family マッピング表（owner culling 列つき）・②分類/③強調 note・原則15 クロスリファレンスを追加、Don't に secondary 誤用禁止を追加。JSX コメント（`:158` 相当）の枠線 3:1 記述を `--border` 必須へ更新 + Human Gate 回答反映（2026-09-06）: tone family を感情で分ける規約文を追加、反映済み→success 行・取込み済み→warning 行へ移動し owner culling 列に原文 2 を転記、Alert 節に未取込み通知 3 箇所の適用先と success 本文色維持を追記 |
| 2026-09-05 | ⑧ native select 統一 | ④ フォームセクション・⑨ 検索+フィルタの Don't に「native `<select>` を使わない（DSR-23）」を追記 |
| 2026-09-03 | UI 一覧の背骨 D — Lane 2 | ⑩ canonical を `src/components/patterns/Pagination.tsx`（`Pagination` + `PaginationSummary`）へ、件数文言を範囲付き統一形「{n} 件中 {from}〜{to} 件目 · {p} / {t} ページ」へ実装。⑯ canonical を `src/components/patterns/ListShell.tsx` へ（商品一覧 pilot 採用）、必須構成 1 の枠を `rounded-lg border bg-card p-4` へ、必須構成 3 の固定列影の記載を必須構成 4 側へ移動 |
| 2026-09-03 | 本 PR | Human Gate + Codex review 是正。⑩ 下部 skeleton を当時の canonical 文言（`{totalCount} 件中 {page} / {totalPages} ページ`、from/to 範囲なし）へ戻し、範囲付き統一形は後続 lane での移行対象と明記。⑯ の canonical を「なし（後続 lane で ListShell を新設予定）」へ、適用条件の記載を DSR-22 一本化に差替え |
| 2026-09-03 | 本 PR | ⑯「一覧の器（ListShell）」を新設（必須構成 6 項目）。title・責務を「16 パターン」に改訂。⑩ ページネーションへ上部 variant（件数 + 現在位置テキスト必須・pager 任意、viewport 超過一覧のみ opt-in）と perPage 既定値の画面別裁定注記を追記 |
| 2026-08-16 | PR #79 | SPEC-SDI-D5: パターン⑧を売上同日追加の高影響確認へ追随し、実装前の契約正本をUI-07へ接続。 |
| 2026-09-08 | PR #46 | PageHeader の副題・説明の併存、取引先説明の移行、備考の空欄・全文表示、直近件数・価格履歴表の runtime 反映と参照行を同期。 |
| 2026-09-10 | PR #51 | ⑨ に全フィルタ入力の Label 上置き規範と例外・DepartmentFilter 内部構造、① にセクション見出し variation、③ に参照句、⑤ に Label 規範参照を追加。Human Gate 確認用の draft と mockup-g を登録。 |
| 2026-09-11 | PR #51 | ⑳ Gated Amendment 2: Human Gate 回答の確定文化〈全フィルタの上置き Label・Checkbox の縦中央・セクション見出しと PageHeader (c) の見出し行 + 説明行〉。mockup-g を 4 状態へ更新。 |
| 2026-09-11 | PR #51 | ⑳ Gated Amendment 3: 見出しの shrink 保証 / runtime 未反映の明示。 |
| 2026-09-11 | PR #50 | ⑲ Gated Amendment 5: 固定帯を白地の箱へ / chevron を muted。 |
| 2026-09-15 | PR #63 | ㉑ フィルタ Label 上置き・PageHeader と section 見出しの2段配置・h2 token の runtime 反映。① props、⑤ 群名の紐付け、⑨ 適用サイトと mode 切替例を同期。 toolbar Checkbox `h-9 self-end`（L3 round 1）。 |
| 2026-09-28 | small-fixes-batch（plan 側で先に訂正） | ⑨ `DepartmentFilter` の幅を部品の `w-[11rem]` 固定にし、呼び出し側で幅を渡す記述と例をやめる（旧 04「部門 select 幅は全画面同一」）。入出庫履歴の独自 select の `w-44` を同じ幅として記す。 |
