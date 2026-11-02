# 77. UI-14: 一括価格改定

> 対応仕様: REQ-105, REQ-106
>
> 入力ドキュメント: `docs/function-design/20-io-product-repo.md`、`docs/function-design/30-biz-product-service.md`、`docs/function-design/40-cmd-product.md`、`docs/function-design/50-ui-product-list.md`、`docs/db-design/master-tables.md`、`docs/db-design/tracking-system-tables.md`、D-075

## 77.0 関数要求 / シグネチャ / 処理ステップの扱い

**関数要求**: UI-14 は取引先・部門・keyword 等の URL state から商品を取得し、行ごとの新売価 / 新原価（案）を `commands.reviseProductPrice` に渡し、確定結果を同じ行へ反映する。

**シグネチャ**: frontend の境界は generated `commands.searchProducts` / `commands.listSuppliers` / `commands.createSupplier` / `commands.reviseProductPrice` / `commands.listPriceHistory` と各 DTO である。CMD の inline 署名は複製せず、[40-cmd-product.md](40-cmd-product.md) を正とする。

**処理ステップ**: URL state から一覧を取得し、利用者が「新しい取引先を追加」を選んだ場合は取引先を追加し、現価格とリストを照合して 1 行ずつ確定する。成功後は行を再取得し、失敗時は入力を保持して同じ行から再試行する。詳細は §77.4〜77.7 に定義する。

**エラーハンドリング**: 一覧全体、取引先追加、行確定を別の失敗境界に分け、正常な他行と確定済み結果を隠さない。利用者向け日本語文言と再試行導線は §77.7 を正とする。

## 77.1 位置付け

UI-14 は、メーカー単位の紙 / PDF 値上げリストを手元に置き、JAN またはメーカー品番で商品へ到達して、売価・原価を商品マスタへ直接反映する operator-facing 画面である。紙の棚卸しリストへの転記を挟まず、数日かかる作業を行単位で安全に中断・再開できることを目的とする。

route は `/products/price-revision`、file route は `src/routes/products/price-revision.tsx`、画面本体は `src/features/products/PriceRevisionPage.tsx` とする。UI は CMD を generated binding 経由で呼び、価格・履歴・取引先の業務規則を BIZ / IO から引き上げない。

## 77.2 Design Intent Trace

| Spec / requirement ID | Decision ID | 設計判断 | 理由 / 捨てた案 |
|---|---|---|---|
| REQ-105 | SPEC-PRV-D3 | 取引先・部門・keyword・廃番を含む条件で絞り込み、在庫ゼロ商品も常に対象にする。取引先指定時は「取引先未設定の商品も含める」を既定 on にする。 | 初年度は supplier 紐付けが疎であり、取引先必須や設定済み行だけでは作業対象を見失う。 |
| REQ-105 | SPEC-PRV-D4 | 現掛率は導出表示、新原価案は整数除算で導出し、新売価は利用者が入力する。 | 掛率は商品ごとに異なる導出値で、永続化や浮動小数計算は円の再現性を損なう。 |
| REQ-105 | SPEC-PRV-D5 | 確定は行単位で `revise_product_price` を呼ぶ。 | 複数行を 1 request にまとめず、途中停止・部分失敗・再試行を商品単位で閉じる。 |
| REQ-106 | SPEC-PRV-D6 | 取引先 filter から「新しい取引先を追加」でき、未設定の商品にだけ選択中の取引先を漸進紐付けする。紐付け toggle は既定 off（owner 2026-09-16、D-088）。 | 約 80 社の事前投入を避け、値上げリストを処理する文脈でメーカー/ブランドを補完する。価格改定のついでに取引先が変わる副作用を避けるため opt-in にする。 |
| REQ-105 | SPEC-PRV-D7 | 確定済み状態は DB の価格と price_history から復元し、本日改定した行を「最近改定」と示す。 | 数日またぎでも追加の永続状態を持たず、確定済みデータを再開点にする。 |
| DSR-19 | SPEC-PRV-D8 | 行確定の成功時は DSR-19 に適合する完了 toast を出し、対象商品と価格改定が完了したことを示す。 | 入力欄の clear と「最近改定」badge だけでは、確定操作が成功した瞬間の完了を判別しにくい。 |

## 77.3 画面構成

```text
src/
  routes/products/price-revision.tsx
  features/products/
    PriceRevisionPage.tsx
    priceRevisionSearch.ts
    components/
      PriceRevisionFilters.tsx
      PriceRevisionTable.tsx
      CreateSupplierDialog.tsx
    hooks/
      usePriceRevisionList.ts
      useReviseProductPrice.ts
    lib/
      price-revision-math.ts
```

- PageHeader: title `一括価格改定`、値上げリストの 1 行ずつを照合・確定する画面であることを日本語の 1 行説明で示す
- 絞り込み枠: 取引先、部門、商品検索、廃番を含む、取引先未設定の商品を含める
- 一覧: 高密度 table を維持し、主要な識別値・現価格・入力・確定操作を 1 行で比較できる
- 行状態: `最近改定` / `入力中` / `確定失敗` を日本語 text と icon / badge / 行内位置で区別し、色だけでは表さない

## 77.4 URL State

| search state | 型 / 既定 | CMD への変換 |
|---|---|---|
| `q` | string / `""` | trim 後に空なら keyword = null。非空なら商品名、product_code、jan_code、maker_codeの部分一致（SPEC-PRV-D2） |
| `supplier` | number or undefined | 選択した supplier_id。未指定なら supplier 条件なし |
| `includeUnassigned` | boolean / 取引先指定時 true | true なら `supplier_id = X OR supplier_id IS NULL`、false なら `supplier_id = X` |
| `dept` | number or undefined | department_id |
| `discontinued` | boolean / false | true のとき廃番を含む |
| `sort` | string / product_code ascending | 既定は商品コード昇順 |
| `page` | positive integer / 1 | 1 始まり。絞り込み変更時は 1 に戻す |
| `perPage` | 50 / 100 / 200、既定 50 | UI-01a と同じ既存上限内で送る |

在庫数による除外条件は持たず、在庫ゼロ商品は常に対象とする。無効な search 値は既定へ回復し、F5 / 再訪時に確定済みの filter 状態を再現する。確定前の行入力は URL に載せない。

## 77.5 CMD / DTO 契約

CMD の署名と wire DTO の正本は [40-cmd-product.md](40-cmd-product.md) に置き、この UI doc には inline 署名を複製しない。

| generated command / DTO | 用途 |
|---|---|
| `commands.searchProducts(query)` / `ProductSearchQuery` | filter・sort・paging に一致する商品一覧。keyword は maker_code を含む |
| `commands.listSuppliers()` / `Supplier` | 取引先（メーカー/ブランド）の complete master data |
| `commands.createSupplier(name)` / `Supplier` | 取引先ピッカー dialog 内の「新しい取引先を追加」 |
| `commands.reviseProductPrice(input)` / `PriceRevisionInput` / `PriceRevisionResult` | 行単位確定。選択中の取引先を未設定商品へ渡す |
| `commands.listPriceHistory(productCode, limit)` / `PriceHistoryEntry` | 直近 changed_at から「最近改定」を導出する |

円は 0 以上の整数で wire に載せる。現掛率は wire field にせず UI で導出する。確定後は該当行と商品一覧を再取得し、DB の現売価・現原価を表示の正とする。

## 77.6 表示と操作

### 絞り込みと取引先の漸進補完（SPEC-PRV-D3 / D6）

- 取引先（単一選択・任意）、部門（単一選択・任意）、keyword、`廃番を含む`（既定 off）を置く
- 取引先を選ぶと `取引先未設定の商品も含める` を既定 on で表示する。抽出条件は `supplier_id = X OR supplier_id IS NULL`。off なら選択した取引先だけにする
- `確定した商品の取引先が未設定なら、この取引先を設定する` は取引先選択中だけ表示し、既定 off とする（取引先を変えても off へ戻る）。行確定時に商品の supplier_id が NULL の場合だけ設定し、既存値は上書きしない
- 取引先の選択は取引先ピッカー dialog（`01-decision-rules.md` DSR-24）を経由する。『すべての取引先』行は一覧先頭に残し、『取引先未設定の商品も含める』toggle はフィルタ列に残置する（dialog 内へは移動しない）。追加成功後は complete master data を再取得して選択できる
- 在庫ゼロ商品は常に対象とし、在庫数 filter は追加しない

### 一覧列と価格入力（SPEC-PRV-D4）

列は `商品コード` / `JAN` / `メーカー品番` / `商品名` / `現売価` / `現原価` / `現掛率` / `新売価` / `新原価（案）` / `確定` とする。横幅が不足する場合も商品識別・現価格・入力値・確定操作を recovery path なしで隠さず、table の横スクロールで到達できるようにする。

- 現掛率は `現原価 ÷ 現売価` を % 表示し、小数 1 桁に四捨五入する。現売価が 0 のときは `—` と表示する
- 新売価は空から手入力する。**新売価の自動提案はしない**
- 新売価の入力後、新原価（案）の初期値を、原価（1/100 円）を円の単位で切り捨てた `floor(新売価 × 現原価(1/100 円) ÷ (現売価 × 100)) × 100`（`price-revision-math.ts` の `deriveProposedCost`）で求め、新原価の欄には SPEC-UNIT-D5 の入力の欄の初期値の文字列（9100 → `91`）を入れる。現売価が 0 のときは現原価を初期値にし、どちらも利用者が上書きできる
  - 原価の lane（D-113・D-128、`決定（owner、D-113 J2）`）: 原価が 1/100 円になった後も、新原価（案）は円の単位で切り捨てる。メーカーの新原価は切り捨てが多い（台帳 L-135）ので、案に 1 円未満を出さない。利用者は小数 2 桁まで直せる（[共通規則](10-common-rules.md) SPEC-UNIT-D5）。現掛率の表示は `現原価(1/100 円) ÷ (現売価 × 100)` で今と同じ値。例: 現原価 8333（83.33 円）・現売価 100・新売価 110 → 新原価（案）`floor(110 × 8333 ÷ 10000) × 100 = 9100`（91 円。1/100 円の値を今の式へそのまま渡すと 9166 = 91.66 円になる）、現掛率 `83.3%`。現売価 0 → 新原価（案）は現原価 8333、現掛率は `—`。
- 新売価は 0 以上 `999999` の整数、新原価は 0 以上 `999999.99` の小数 2 桁までの円（送る値は 1/100 円の `new_cost_price_centi`、SPEC-UNIT-D5。形式の誤りは `原価は0以上で、小数は2桁までで入力してください`、欄は `inputMode="decimal"`・`step="0.01"`）で検証する。現原価の列は SPEC-UNIT-D5 の formatter（`¥83.33`）で出す。越えた欄は `売価が大きすぎます`・`原価が大きすぎます` を出し、直すまで確定できない（command を呼ばない）。新原価（案）が越えたときも同じで、案はそのまま欄に入れる（[共通規則](10-common-rules.md) の「入力の上限と安全な整数の範囲」。BIZ の側は [40](40-cmd-product.md)）
- 現売価は値上げリストの「旧売価」と目視で突合するために使う。不一致は上乗せ品または別商品の目印であり、追加の自動判定は行わない
- 価格は `¥` と桁区切りを表示し、入力 label / column header は日本語で意味を固定する

### 行確定と中断・再開（SPEC-PRV-D5 / D7 / D8）

- `確定` は該当行だけを送信する。pending 中は同じ行の入力と確定操作を無効化し、他行の閲覧は妨げない
- 成功後は DB の現売価・現原価を再表示し、価格が変わった行を完了として扱う。失敗は該当行に日本語 error と再試行導線を残す
- 成功後は DSR-19 に適合する完了 toast で、対象商品と価格改定が完了したことを示す。duration と toast id は DSR-19 の階層・適用基準に従う。runtime への追加は後続 R3 で行う
- **draft 保存テーブルは設けない**。確定済み行は再訪時にも更新後の現価格として取得できる
- 当該商品の直近 `price_history.changed_at` が本日（ローカル日付）なら、行頭に icon + text badge `最近改定` を表示する。これは price_history からの導出で、別 field を永続化しない
- `画面を再読み込みすると、確定前に入力した新売価・新原価は失われます。1行ずつ確定してください。` を絞り込み枠と一覧の間に常時表示する
- 共通離脱ガード（UI_TECH_STACK §6.11 useUnsavedChangesWarning）は UI-USW-D3 (c)（行単位の即時 DB 保存、棚卸しと同型）により適用せず、上記の常時文言で代替する。

## 77.7 Loading / Empty / Error

- Loading: 共通 ListSkeleton を使い、表の列構造を保つ。取引先候補だけの失敗で商品一覧全体を隠さない
- Empty: filter なしで商品 0 件なら商品一覧への導線（`/products`。D-138 の後は label を「商品と在庫を開く」にする。先の route は同じ。proposed・未実装）、filter ありなら `条件に一致する商品がありません` と `絞り込みを解除` を表示する
- 一覧取得失敗: ページ上部 Alert に日本語説明と `再試行` を出す。確定済みと断定しない
- 取引先追加失敗: dialog の入力を保持し、空文字 validation は field 近傍、内部 error は再試行可能な Alert で示す
- 行確定失敗: 他行を消さず、該当行だけ `確定できませんでした` + error 内容 + `再試行` を表示する。入力値はその場に保持する
- Loading / Empty / Error / `最近改定` は色だけで区別せず、日本語 text と role / icon / button label を伴わせる

## 77.8 テスト観点

- SPEC-PRV-D3: 取引先指定時の `取引先未設定の商品も含める` が既定 on で、toggle off が可能。部門 / keyword / 廃番 / page の URL 復元と在庫ゼロ商品の包含を確認する
- SPEC-PRV-D4: 現掛率 % の小数 1 桁、現売価 0 の `—`、新原価（案）の整数除算と現原価 fallback、新売価が空から始まることを独立 oracle で確認する
- SPEC-PRV-D5: 行確定が 1 商品だけを送り、成功後に現価格を再取得する。失敗行だけ入力保持・再試行できる
- SPEC-PRV-D6: 未設定商品だけへ supplier を設定し、既存 supplier を上書きしない。取引先追加の trim / 空文字拒否 / 同名既存行返却を確認する
- SPEC-PRV-D7: 本日の changed_at だけが icon + text `最近改定` になり、再読込で確定前入力が消える旨の常時文言がある
- SPEC-PRV-D8: 行確定成功時に対象商品と完了を示す toast が出て、duration / toast id が DSR-19 の基準に適合することを後続 R3 で確認する
- operator UI: column header、toggle、button、empty/error が日本語で、状態の assertion は text / role / value を使い Tailwind 色 class だけに依存しない
- navigation: `src/config/navigation.ts` の商品管理に UI-14 が存在し、REQ-105 の到達テストが通る
- Windows native L3（実装 PR B）: 400 行級で絞り込み → 旧売価突合 → 入力 → 行確定を反復し、紙の二重転記より速い主動線になっているかを確認する

## 77.9 Deferred

- PDF 自動解析、掛率・参考上代・改定予定日の専用 field、指定日の予約反映
- 新売価の算出補助、複数行の一括確定、改定前入力の長期保持
- 取引先の削除（統合で代替）、問屋チャネル、約 80 社の事前一括投入
- リニューアル品の同定支援（issue #66）

## 77.10 変更履歴

| 日付 | 版 | 内容 |
|---|---|---|
| 2026-09-16 | 取引先紐付け既定 off（D-088） | SPEC-PRV-D6 の toggle 既定を on → off、label 文言を明示。 |
| 2026-09-10 | PR #50 | 取引先ピッカー runtime を反映。選択・追加を dialog へ統合し、未設定商品を含める toggle はフィルタ列に維持。 |
| 2026-09-10 | PR #49 取引先ピッカー design | 取引先選択を DSR-24 の取引先ピッカー dialog 経由へ改訂し、すべての取引先行とフィルタ列の toggle 残置を明記。 |
| 2026-08-30 | PR #22 DSR-19 design sync | SPEC-PRV-D8 を追加し、行確定成功の完了 toast 契約を正本化（runtime は後続 R3）。 |
| 2026-08-22 | 価格改定支援 design-first | SPEC-PRV-D3〜D7 / REQ-105 / REQ-106 の UI-14 契約を新設。 |
| 2026-08-23 | 価格改定支援 実装 B | UI-14 一括価格改定画面、取引先 filter、行単位確定、最近改定表示を実装。 |
