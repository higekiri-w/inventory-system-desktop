## 7. UI-01b: 商品登録・修正

### 計数の置き場と在庫の求め直しの契約（D-118。1〜3 つ目の項目は b1、在庫数の表示は b2、2 つ目の項の案内の場所・文・focus は画面の小口の lane で実装済み）

[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)（SPEC-STK-CNT-D2・D4・D8）の商品フォームの側。拒否と回復先は [30](30-biz-product-service.md) と [40](40-cmd-product.md) から受ける。旧 ADR（時点証拠）の `ej_unverified` の間の在庫連動の提案値 false と拒否の文言、非連動化の警告と「在庫数は未調整です」の案内は持たない（置き換えた）。

- 共有 JAN: create / update が `validation`（field `pos_stock_sync`）で拒否されたら、BIZ の message（「同じ JAN の商品（{商品コード}）があるため、在庫連動にできません」）を該当の欄の近くに出す（今の field 付きの validation の出し方。message を解析しない）。入力値と dirty の状態を保ち、成功の toast や一覧への遷移をしない。在庫連動を黙って off にして再送しない（UI-01b-D26 のダイアログは、「POS販売で在庫を減らす」が「減らさない」になることを名指して利用者が「独自コードで登録する」を押してから送る）。create で入れた JAN を他の商品が使っているときは、この拒否の前に UI-01b-D26 の知らせが出る。
- 在庫連動を始める（false → true）の保存が `recount_required` で拒否されたら、在庫の節の「数量単位」と「POS販売で在庫を減らす」の行のすぐ下（同じ節の中、全幅）に warning の案内（見出し「「POS販売で在庫を減らす」にできません」、文「「減らさない」の間の売上が在庫の数に入っているか確認できません。「今の数を入れる」で棚の数を入れてから、もう一度「減らす」にして保存してください」。「入っていません」と断定しない: `recount_required` は、売上が効く計数に入らないときのほか、入るか決まらないとき・効く計数どうしが衝突しているときにも返る〈ADR D8、30〉）と、「今の数を入れる」の導線（回復先の `active_count`・`count_now` のどちらも [73](73-ui-stocktake.md) の「今の数を入れる」の入口。`/stocktake/` の search に `product`〈商品コード〉と `returnTo`〈`/products/{商品コード}/edit`〉を渡す）を出し、案内へ focus を移す（保存の button から離れた所に出るので、出たことを見落とさない。focus は案内の領域で、導線の button ではない: Enter で離れない）。画面の語は欄の名と値（「減らす」「減らさない」）で書き、画面に無い「在庫連動」を使わない。商品フォームは BIZ の message（[30](30-biz-product-service.md)。商品一括 import の行の error に出し、そこでは列名「POS在庫連動」〈[60](60-ui-product-import.md)〉が画面にある）を出さず、型（`recount_required`）で分岐して上の文を出す（画面の小口の lane〈[packet](../plans/2026-10-27-stocktake-screen-fixes.md)〉で、b1 の L3 の owner の所感〈repo 外の回答台帳 TD-433〉から決めた）。入力値を保つ。導線は route の遷移なので、既存の離脱の確認（UI-USW-D1・D2。在庫連動の checkbox を変えたので dirty）を通る: 「破棄して移動」で離れ、「編集を続ける」なら留まる（guard を外さない。未保存の入力を持ち越さない）。戻った商品フォームは DB の値（在庫連動は付いていない）で開くので、利用者は在庫連動を付け直して保存する。商品フォーム自身の `returnTo`（UI-01b-D2）は棚卸しへ渡さず、戻った後の保存は `/products` へ戻る（b1 の Plan Review round 3 の是正）。
- 在庫連動をやめる（true → false）は警告も確認も出さない。
- 在庫数の表示（b2）: 修正の画面の読取り専用の「現在庫」は、写しが要再確認（`stock_recheck`）なら範囲（棚卸しの画面の要再確認の一覧と同じ `formatStockOrRange` の形「{low}〜{high} {単位}」。例「8〜9 個」「4〜4.5 m」）を出し、欄の label の隣に「要再確認」の①状態の badge（文字と icon。色だけで示さない。[66](66-ui-stock-movements.md) と同じ部品 `StockRecheckBadge`）を出す。要再確認でなければ今の 1 つの値。範囲の文字は form の値の baseline と同じ値なので、何も変えなければ dirty にならず離脱の確認も出ない（検証も送信もしない）。badge の要否は `ProductFormPage` が商品の結果の `stock_recheck` を `ProductForm` の prop `stockRecheck` で渡す（[40](40-cmd-product.md) の商品の結果の列）。登録の画面は変えない。

> 対応仕様: REQ-101 / REQ-102 / REQ-106 / UI-01b
>
> 入力ドキュメント: `docs/architecture/ui-task-specs.md` UI-01b、`docs/SCREEN_DESIGN.md` 商品登録・修正画面、`docs/function-design/30-biz-product-service.md` `create_product` / `update_product` / `toggle_discontinue` / `get_product`、`docs/function-design/40-cmd-product.md` CMD-01、`docs/function-design/20-io-product-repo.md` products / departments / suppliers

商品登録・修正は、商品マスタを作成・更新する operator-facing form である。商品と在庫の画面（UI-01a、D-138。今の build は商品検索・一覧）からの新規登録 / 修正導線を受け、既存の BIZ-01 / CMD-01 契約を generated binding 経由で呼び出す。

説明書作成時は、商品登録と入庫記録の役割差を明記する。商品登録は商品マスタ作成と初期在庫の設定に使い、通常の仕入で在庫を増やす作業は UI-02 入庫記録で行う。

## 7.1 設計判断

| Spec | Decision ID | 決定 | 理由 / 不採用案 |
|---|---|---|---|
| REQ-101 / UI-01b | UI-01b-D1 | route は `/products/new` と `/products/$code/edit` に分ける。file route は `src/routes/products/new.tsx` と `src/routes/products/$code.edit.tsx`。 | create / edit の状態差が大きいため path で mode を明示する。query param だけで mode を切り替える案は、直接 URL 共有時に意図が読み取りにくいため不採用。 |
| REQ-101 / REQ-102 | UI-01b-D2 | 保存成功後は来た一覧へ戻る。`returnTo` search param は `/products` 一覧 route とその search params だけ許可し、それ以外は `/products` に戻す（許す範囲は D-138 でも変えない）。作る側は `buildProductListReturnTo`（`src/features/products/lib/return-to.ts:30-42`、`URLSearchParams` の手組み）で、受け口は `parseProductListSearchFromReturnTo`（同 `:44-65`）が既知の key を型つきで読み直して `navigate({ to: "/products", search })` へ渡す（`src/routes/products/$code.edit.tsx:29`。中身は router の parseSearch を通らない）。D-138 の後（proposed・未実装）は、両方の関数に統合の画面の URL state の `status`・`selected`（[50](50-ui-product-list.md) §50.4）を足し、行の詳細の「商品修正」はこの作る側で組んだ値を渡す（50 UI-01a-D17、[58](58-ui-stock-inquiry.md) §58.7）。在庫切れ・在庫少の一覧（`status=low_stock`）や開いていた行から直しても、保存・戻るで来た一覧とその行へ戻る。例外: 在庫連動を始める拒否（`recount_required`）の回復で棚卸しを経た後の保存は `/products` に着く（b1〈PR #188〉の Plan Review round 3 の決定。本書の冒頭の b1 の文と同じ）。今の build は 2 つの key を持たず、在庫照会から来ると `returnTo` が無く商品一覧へ着く。 | UI-01a の検索条件 URL state を保って戻れるようにする。一方でフォーム route、import route、外部 URL、他画面 route への遷移は不要で、保存後の循環や誤遷移を生むため許可しない。作る側に router の href（`location.href`）を使わない: router の search の直列化は JSON として読める文字列（`1234567890123` のような数字だけの商品コード）を引用符つきで出し（lockfile の `@tanstack/router-core` 1.171.3 の `src/searchParams.ts` の `stringifySearchWith`。JSON として読める string を `stringify` し直す）、受け口の `URLSearchParams.get` は引用符ごと受けるので、検索語が引用符つきになり 0 件になる。受け口を引用符つきの値も外す形に広げる案は採らない（利用者が引用符ごと検索した語と区別できない）。入れ子の `returnTo`（棚卸しへ渡す `returnTo` の中に一覧の `returnTo` を持たせる）で例外をなくす案は b1 の決定を変えるので本書では採らず、[backlog](../backlog.md) の「商品と在庫の画面の統合の runtime」の候補に置いた。 |
| CMD-01 / UI-01b | UI-01b-D3 | UI は tauri-specta で生成済みの `commands.createProduct` / `commands.updateProduct` / `commands.toggleDiscontinue` / `commands.listSuppliers` を使う。generated binding の正本は `src/lib/bindings.ts` とする。 | `typedInvoke` fallback は Phase 2 closeout で退役済み。UI-01b だけ ad hoc invoke に戻す案は contract drift のため不採用。 |
| REQ-101 | UI-01b-D4 | create mode の商品コード入力は「JANコードあり」と「JANなし独自コード自動発番」に分ける。JAN blank + 選択部門に `code_prefix` がある場合だけサーバー発番を使い、`code_prefix` がない部門では保存前 validation で止める。 | 現行 `ProductCreateRequest` は任意 `product_code` 手入力を受けない。UI が手入力欄を出しても backend に送れないため不採用。JANなし商品は独自コード発番対象部門を選ぶ。 |
| REQ-102 / SP-102-04 | UI-01b-D5 | edit mode では `product_code` と `jan_code` と `stock_quantity` を読取専用にする。`stock_unit` は長さの単位の商品だけ読取専用（個数の単位の商品は UI-01b-D23）。 | `product_code` 変更不可は DB 設計と仕様で確定済み。`ProductUpdateRequest` は `stock_quantity` を更新しない（`stock_unit` は個数の単位どうしの変更だけ。BIZ-01-D8）。単位変更の範囲は D-119 で個数の単位どうしに限って決めた（UI-01b-D23。種類をまたぐ変更は在庫履歴・閾値・POS 連動の数の意味を変えるので採らない）。`stock_unit` の読取専用は UI-01b-D23（D-119）が個数の単位の商品について置き換える（UI-01b-D22 の末尾の「edit mode の単位は読取専用」も同じ）。 |
| REQ-102 / D-119 | UI-01b-D23 | （owner の決定 U2 = b）edit mode で、今の単位が個数の種類の商品は、数量単位の select を個数の単位だけ（D1 の表の順）で出して変えられる。長さの単位（`m`）の商品は今どおり読取専用（select は押せず、今の語が読める）。変えたときだけ `updateProduct` の `stock_unit` に値を入れる（変えていなければ送らない）。送る判定は登録の単位で行い、登録の単位が長さの商品は form の値によらず `stock_unit` を送らない（`buildUpdateProductRequest`。読取専用の select を経ない値を送らない）。select の下に 1 文 `単位の語だけが変わり、在庫の数は変わりません。過去の記録も新しい語で表示されます。` を出す（個数の単位の商品の edit mode だけ）。BIZ が拒んだとき（`在庫単位は個数の単位どうしでだけ変えられます`）は保存の error として出す。「現在庫」の欄は保存するまで登録の単位の語のまま出す（保存すると一覧へ戻る。UI-01b-D14）。在庫の節の説明文: create は `初期在庫と数量単位を設定します。登録後は、数量単位を個数の単位どうし（例: 個 → 玉）でだけ変更できます。`、edit の個数の単位の商品は `現在庫は変更できません。数量単位は個数の単位どうしでだけ変更できます。`、edit の長さの単位の商品は今の `現在庫と数量単位は登録後に変更できません。` のまま。保存の後、productUpdate の無効化の範囲（[UI_TECH_STACK](../UI_TECH_STACK.md) の D-052）に無い画面（記録詳細・売上の画面）は次の再取得まで古い語で出ることがある（数は同じ。範囲を広げない）。 | 在庫 Excel から初期投入した毛糸が「個」で入り、「玉」に直せない（2026-10 の見直しの文書の食い違い）。個数の単位は価格の基準数量がどれも 1 で、数の意味が変わらない（[共通規則](10-common-rules.md) SPEC-UNIT-D2）。不採用: 在庫 0 なら種類をまたいで直せる（在庫 0 でも過去の明細の意味が変わる）。 |
| REQ-101 / REQ-102 / D-119 | UI-01b-D24 | （owner の決定 U7 = b。原価の lane で実装、`ProductForm.tsx` の `CostPackHelper` と `src/lib/unit-quantity.ts` の `unitCostFromPack`）選んでいる数量単位が個数の単位のときだけ、原価の欄の下に「袋の値段から出す」の補助を置く（長さの単位〈`m`〉では出さない。create mode で単位を長さの単位に変えたら補助を閉じて隠し、原価の欄は変えない）。開くと `袋の値段（円）` と `入り数` の 2 つの欄と「原価に入れる」が出て、押すと [共通規則](10-common-rules.md) SPEC-UNIT-D5 の入力の補助の値（1/100 円未満を四捨五入）を原価の欄に入れる。袋の値段は原価と同じ入力の規則、入り数は 1 以上 `9999999` 以下の整数で、外れれば `入り数は1以上の整数で入力してください`・原価の欄と同じ文を補助の欄に出し、原価の欄を変えない。2 つの欄の値は保存しない。補助を閉じて開き直すと 2 つの欄は空から始まる（前の値を持ち越さない。原価の lane の起票で決めた: 保存しない値を画面に残すと、別の商品の袋の値段で原価を入れうる）。 | 店はばらす商品の 1 個の原価を袋の値段 ÷ 入り数で手で出している（2026-09-29 の店主の回答）。原価の入力は原価の lane で変わるので同じ所に足し、持つ値は増やさない。不採用: 入り数を商品に持つ（SPEC-UNIT-D5 の不採用 (a)）。 |
| REQ-101 / REQ-102 / D-122 | UI-01b-D25 | （proposed・未実装、JAN の別名の runtime の lane）商品フォームは前の JAN の問い（[30](30-biz-product-service.md) BIZ-01-D9）を出さない。edit mode の JAN は読取専用のまま（UI-01b-D11）で、JAN を直すのは商品 CSV の上書き（UI-01c-D17）だけ。create で入れた JAN がほかの商品の前の JAN の別名だったときは、DB の trigger が別名を消し、その番号の売上は新しい商品へ入る（警告は出さない）。edit で JAN を変えられるようにするときは、同じ問いと答えの運び方を足す（BIZ-01-D9 の Revisit）。 | create の初回の保存は、入れた JAN が他の商品の JAN か商品コードにあれば UI-01b-D26 の知らせで止まり、無ければ `product_code = jan_code` にする（2026-10-27、D-141 で「商品コードの重複で止まる」を置き換えた）。知らせは別名（`product_jan_aliases`）を見ないので、入れた JAN がほかの商品の前の JAN の別名にだけあるときは、知らせずに登録し trigger が別名を消す。知らせの後に利用者が「独自コードで登録する」を押した登録でも、新しい商品の JAN がほかの商品の前の JAN の別名なら同じく trigger が別名を消す（例: 元の商品コードが J・今の JAN が K・前の JAN の別名が J の商品があるとき、J を入れると商品コードの一致で知らせが出て、確認の後の新しい商品が J を持つ）。どちらの取り違えも商品の二重登録と同じ種類の誤りで別名に固有でないので、create の結果の wire に別名に固有の警告の field を足さず、`warnings` にも別名の行を足さない（`warnings` は BIZ-01-D11 で外した商品の行だけ。商品 CSV の取込みは既存の `warnings` で知らせる）。 |
| REQ-101 / D-141 | UI-01b-D26 | （D-141 は proposed〈owner の確認待ち〉・実装済み〈PR #212、L3 待ち〉）create の保存が kind `duplicate`・field `jan_code`（[30](30-biz-product-service.md) BIZ-01-D11 の `JanInUse`）で返ったら、確認のダイアログ（`SharedJanCustomCodeDialog`、shadcn AlertDialog）を開く。title `この JAN はほかの商品が使っています`、本文の 1 段目は BIZ の message（使っている商品の名指しと印「商品コードだけ同じ」「POS販売で在庫を減らす」「廃番」。BIZ が画面の語で作る〈BIZ-01-D11 の文の語の項〉ので、冒頭の `recount_required` の項の「BIZ の message を出さない」の対象でない。解析しない）、2 段目は `色やサイズの違う別の品なら、独自コードで登録できます。JAN はこの商品にも残ります。JAN が同じ商品はレジの売上でどれが売れたか見分けられないため、この商品と、JAN が同じ商品は全て「POS販売で在庫を減らす」が「減らさない」になります（在庫は棚卸しと手入力で合わせます）。「商品コードだけ同じ」の商品は変わりません。`、3 段目は `同じ品なら、新しく登録せずに、商品一覧で JAN を検索してその商品を直してください（廃番なら「表示に戻す」）。`。button は `独自コードで登録する`（主）と `戻る`。使っている商品を省略せず並べるので、ダイアログの高さは viewport の中に収め（`max-h-[90vh]`）、文の 3 段だけを縦に scroll する名前の付いた region（`使っている商品と変わる所`）に入れ（Tab・Shift+Tab で region に入り、矢印の key で scroll できる）、title と 2 つの button は scroll の外に固定する（DSR-24 の picker dialog の footer の固定と同じ形）。`独自コードで登録する` は同じ入力の値で `createProduct` を `shared_jan_custom_code: true`・`pos_stock_sync: false` にして送り直す（form の在庫連動の checkbox の値によらない。checkbox の表示は変えない）。成功は UI-01b-D14 の toast（商品コードは振られた独自コード）に、`warnings` があれば 1 件 1 行で toast の説明に添えて、一覧へ戻る（invalidation は UI-01b-D15 の SSOT の entry `productCreateSharedJan`。[UI_TECH_STACK](../UI_TECH_STACK.md) の冒頭の段落）。選んでいる部門に `code_prefix` が無いときは `独自コードで登録する` を出さず、`部門「{部門名}」は独自コードを振れません。独自コードを振れる部門を選び直してから、もう一度「登録する」を押してください。` を出す。`戻る` と Esc はダイアログを閉じ、入力値と dirty を保つ（送り直さない）。送り直しが失敗したらダイアログを閉じ、今の保存の error の出し方（form-level alert・field error）で出す。 | 2 色目を商品 CSV でしか入れられない（repo 外の回答台帳 TD-349 の起点）。ダイアログは、登録の外の商品（同じ JAN の商品）の在庫連動を変えることを押す前に名指して同意を得る（DSR-07 の確認ダイアログの先例 UI-01b-D13）。form の上の alert に button を置く案は、保存の button から離れた画面の上に出て、長い form では押した所から見えないので採らない。checkbox を自動で off にして再送する案は、利用者が理由を読まずに在庫連動を失うので採らない（冒頭の共有 JAN の項）。 |
| REQ-101 / pos_stock_sync | UI-01b-D6 | create mode で長さの単位（`stock_unit='m'`）にした場合は `pos_stock_sync=false` を提案し、利用者は toggle で true に戻せる。BIZ は UI から受け取った値を尊重する。 | 長さの単位だけで POS 在庫同期を決める案は、DB_DESIGN の `pos_stock_sync` 明示フラグ方針に反する。 |
| REQ-101 / REQ-106 / suppliers | UI-01b-D7 | 取引先候補は `commands.listSuppliers()` 由来の complete master data とし、取引先ピッカー dialog（DSR-24）内の「新しい取引先を追加」から `CreateSupplierDialog` を経由して `commands.createSupplier(name)` を呼べる。 | SPEC-PRV-D6 / UI-01b-D21 で漸進追加を scope 化した。約 80 社の事前投入はせず、必要なメーカー/ブランドだけを form の文脈で追加する。 |
| REQ-101 / REQ-102 | UI-01b-D8 | form は部分障害を分ける。部門候補取得失敗は保存不可、取引先候補取得失敗は取引先未指定なら保存可能、edit の `getProduct` 失敗は form 本体を出さず一覧へ戻る導線を出す。 | 商品登録で部門は必須、取引先は任意。全取得が成功するまで画面全体を空にする案は、復旧操作を阻害するため不採用。 |
| REQ-101 / REQ-102 | UI-01b-D9 | 日本語入力を伴う form のため、実装 PR では Windows native L3 を計画する。 | Tauri 2 Linux WebView には IME 制約があり、商品名・メーカー品番・取引先名の入力品質は Linux だけでは判断できない。 |
| REQ-102 | PRODUCT-PATCH-D1 | edit保存はgenerated `ProductUpdateRequest_Deserialize`を直接使う。通常fieldはomitted/null=no update・value=set、`supplier_id` / `maker_code`はomitted=no update・null=clear・value=setとする。 | `Partial<ProductUpdateRequest_Deserialize>` と保存時castはgenerated wire契約の誤りを隠すため不採用。 |
| REQ-101 / REQ-102 | UI-01b-D10 | form を「商品の識別」「分類と取引先」「価格」「在庫」の基本 4 セクションに分割し、edit mode は第 5 セクション「価格履歴」（UI-01b-D20）を末尾に加える。各セクション見出しは h2 `text-xl font-semibold` + Separator + 1 行説明とし、メーカー品番は「商品の識別」に置く。 | 1 列のフラットな入力列は、非IT利用者が入力順を把握しづらい。意味の塊でグルーピングし、入力の流れを明示する。section ごとの説明で登録後変更できない項目を予告する。 |
| REQ-102 | UI-01b-D11 | read-only 入力（商品コード / edit の JANコード・現在庫）は `readOnly` + `bg-muted` で示す。数量単位 select は、個数の単位の商品は個数の単位だけの候補で変更でき、長さの単位の商品は `disabled` で押せず今の語が読める（D-119。UI-01b-D23）。 | `disabled` の opacity-50 は値が読みづらく、read-only と操作不能の意味が混ざる。表示専用の値は読める muted 背景にする。`<select>` は readOnly 属性が効かないため数量単位だけ `disabled` を残す。 |
| REQ-101 / REQ-102 | UI-01b-D12 | 必須項目（商品名・部門・売価・原価、create 時は初期在庫）のラベルに `（必須）` を付ける。色で必須を符号化しない。JANコードは対象外。 | 非IT利用者には必須項目が分かりやすい方がよい。色（赤 * 等）だけの符号化は [design-system/00-foundations.md §業務ステータスの視認性](../design-system/00-foundations.md) に反するため、テキストで明示する。JAN は任意（独自コード発番経路がある）ため必須にしない。 |
| REQ-102 | UI-01b-D13 | 「廃番にする」操作は確認ダイアログ（`DiscontinueConfirmDialog`、shadcn AlertDialog）を通す。「表示に戻す」は確認なしで直接実行する。 | 廃番化は一覧の通常表示から外れる片方向の状態変更で、誤操作の影響が大きいため確認する。復帰は影響が小さく、戻す操作にさらに確認を挟むと操作が重くなるため直接実行する。共通判断は design-system DSR-07 / component catalog ⑧を参照する。 |
| REQ-101 / REQ-102 | UI-01b-D14 | 保存成功は `toast.success`（`id: "product-save-success"`、`duration: 5000`）で示し、navigate より前に発火する。create は登録した商品コードを含め、edit は保存完了を伝える。 | 保存後すぐ一覧へ戻るため、成功は一覧に遷移してからでなく form 上の通知で確実に伝える。id 固定で連打時に通知が積み重ならないようにする。alert を画面内に残すと遷移と二重になるため toast を使う。 |
| REQ-101 / REQ-102 | UI-01b-D15 | create、update、廃番化・復帰の成功時 invalidation は [D-052](../decision-log.md) C1/C2 と `src/lib/invalidation-contract.ts` を正本とし、form 内に query key 集合を列挙しない。 | 商品・在庫・PLU・棚卸し consumer への追随を mutation 単位の SSOT で保証し、page と設計書の二重管理を避ける。 |
| REQ-101 / REQ-402 | UI-01b-D16 | create の JAN 欄は全角数字を半角へ正規化する。適用点は composition 中でない onChange（キー入力・paste を含む全経路）と onCompositionEnd の確定値の 2 点。値全体が数字（半角/全角）の場合のみ写像し、混在値は無変換、composition 中は不加工。NFKC・trim 変更・記号/かな変換・チェックディジット補正は行わない。util は `normalizeComposedDigits` / `isComposedDigitsOnly` の既存実装を import 流用し、複製・挙動変更を禁止する。 | 日本語 IME 下のスキャナ/手入力・paste で全角数字が残ると保存 validation（UI-01b-D17）と PLU 提案（UI-01b-D18）が誤動作する（PR #65 初回 L3 の IME blocker 起源）。catalog ⑮ SPEC-SUGGEST-D11/D12 と同一写像で挙動を揃える。JAN 欄専用の別実装は D11 と drift するため不採用。 |
| REQ-101 | UI-01b-D17 | create 保存時、jan_code 非 null なら「trim -> UI-01b-D16 と同一写像の全角→半角正規化 -> 検証」の順で適用し、正規化後の値が ASCII 数字 8 桁または 13 桁かつモジュラス 10 チェックディジット整合であることを必須とする。保存値は正規化後の値とする。この pipeline は frontend（`src/features/products/lib/jan-code.ts` 新設 + `product-form-request.ts`）が実行し、BIZ（BIZ-01-D1）は正規化後 wire 値を検証のみ行い再正規化しない。既存の blank + code_prefix 規則・文言は不変。ISBN-10 は許容しない（本は 13 桁 JAN〈EAN-13/ISBN-13〉で登録 = owner 裁定 2026-08-11）。 | 手入力 typo・読み違いの防御（実バーコードはチェックディジット整合が保証される）。warn-only は防御にならず不採用。JAN 欄空白 + 独自コード発番部門の既存経路が escape hatch として残る。 |
| REQ-402 | UI-01b-D18 | `suggestPluTarget` は trim と UI-01b-D16 写像を適用した candidate、すなわち正規化適用後の値で評価し、ASCII 数字 13 桁のみ true とする（JAN-8 は false 維持 = PLU 書出し 13 桁前提との整合）。BIZ `should_default_plu_target`（BIZ-01-D2）と同一意味論の意図的二重実装とし、実装統合はせず、両側に同一ケース表の独立転記 oracle drift-guard test を置く。composition 中の評価は transient で onCompositionEnd の正規化で収束し、追加の抑制はしない。 | 全角 13 桁入力で PLU 提案が false になる既知問題を入力正規化で根治する。wire 越えの SSOT 化は bindings 定数 export の重さに見合わず不採用（判定は 1 行規模）。前提の訂正（D-117 (7)）: 「JAN-8 はもう使わない」は確かめた事実ではなく、店の資料（2026-10-04）に検査の通る 8 桁の JAN がある。今の設計では 8 桁の商品は PLU 書出しの要修正（`JANが13桁ではありません`）に出る。8 桁を PLU に載せるかは JAN の無い商品の店内コードの lane と一緒に決める（[backlog](../backlog.md)）。 |
| REQ-402 / D-117 | UI-01b-D19 | edit mode は memory No.（「レジメモリNo.」）を表示しない（D-117 (5) で撤去。runtime の lane が表示・DTO の `plu_memory_no`・test を外す）。廃番解除は `plu_target` を自動復帰させず、必要なら利用者が明示して再対象化する（再対象化すると前と同じ PLU の枠を使う）。 | 店主が番号を見る場面が無い（売上は JAN で商品へたどる）。廃番解除だけで意図せずレジ再登録されることを防ぐ。 |
| REQ-102 / SPEC-PRV-D9 | UI-01b-D20 | edit mode に第 5 セクション「価格履歴」を置き、直近 10 件を表示する。「すべて表示」は limit 100 で再取得し、create mode ではセクション自体を出さない。空は「価格履歴はまだありません」、取得中は「読み込み中…」、取得失敗は inline error と「再試行」を表示する。 | 過去の価格を商品修正の文脈で確認し、紙の前年リスト参照を置き換える。price_history に契機カラムがないため変更契機の列は表示しない。 |
| REQ-106 / SPEC-PRV-D6 | UI-01b-D21 | 「分類と取引先」セクションの取引先ピッカー dialog（DSR-24）内に「新しい取引先を追加」導線を置く。name は trim、空文字を拒否し、同名は既存行を返したうえで complete master data を再取得する。 | `suppliers` はメーカー/ブランドを漸進補完する。改名・統合は UI-15（[78-ui-supplier-management.md](78-ui-supplier-management.md)）で扱う。約 80 社の事前一括投入は扱わない。 |
| REQ-101 / REQ-102 / D-113 | UI-01b-D22 | 数量単位の select は [共通規則](10-common-rules.md) SPEC-UNIT-D1 の 10 個を表示の語で並べる（個・枚・本・袋・箱・組・セット・玉・丁・m。`玉` は `決定（owner、D-113 J1）`。巻・cm は vB で外した: D-128）。既定は `pcs`（個）。UI-01b-D6 の `pos_stock_sync` の自動提案は、長さの単位（m）で `false`・個数の単位で `true`。（D-113 で予定した「`get_pos_stock_readiness` が `ej_unverified` を返す間は全単位で `false`」の分岐は、D-118 で準備照会と `ej_unverified` を持たなくなったので作らない。）利用者が checkbox を触った後（touched）は提案で値を変えない（今の UI-01b-D6 と冒頭の規則のまま）。長さの単位のとき、売価・原価の label に `（1 m あたり）` を添える（例: `売価（必須、1 m あたり）`）。初期在庫は m の商品では m で受けて cm に直す（SPEC-UNIT-D3）。原価の欄の label は `原価（税抜、必須）`（長さの単位では `原価（税抜、必須、1 m あたり）`。D-113・D-128。`決定（owner、D-113 J2）`）、価格の節の説明は `売価は税込の円の整数、原価は税抜の円で小数 2 桁まで入力します。長さ（m）の商品は 1 m あたりの値段です。`。原価は [共通規則](10-common-rules.md) SPEC-UNIT-D5 の入力の規則で受け、原価の欄は `inputMode="decimal"`。売価は `999999`、原価は `999999.99`（1/100 円で `99999999`）、初期在庫は `999999999`（m の商品は cm にした値）を上限にする。edit mode の単位は、長さの単位の商品は読取専用、個数の単位の商品は個数の単位どうしで変えられ（SPEC-UNIT-D2、UI-01b-D23）、現在庫は単位つきで出す（`27.5 m`・`20 個`。SPEC-UNIT-D3 の表示する所の表の 13）。 | 単位の語は店の答え（Store Premises Facts、2026-09-14）の語から、在庫の Excel で 0 行の巻・cm を外した 10 個（D-119・D-128）。長さの商品の値札・伝票は 1 m あたりで、価格欄に基準が無いと 1 cm あたりと読み違える（Backlog「原価 × 数量の式が価格の基準数量を持たない残り」(4)）。説明を `税抜の整数` と書くと master-tables の売価〈税込〉と食い違う。不採用: 単位を 2 段の select（種類 → 語）にする（10 個は 1 段で選べる）。 |

## 7.2 Component / Route 構成

```
src/
  routes/
    products/
      new.tsx              -- create route
      $code.edit.tsx       -- edit route
  features/
    products/
      ProductFormPage.tsx
      components/
        ProductForm.tsx                -- create は 4、edit は価格履歴を加えた 5 セクション（UI-01b-D10 / D20）
        PriceHistorySection.tsx        -- edit mode の価格履歴（UI-01b-D20）
        StockUnitField.tsx
        DiscontinueConfirmDialog.tsx   -- 廃番確認（UI-01b-D13）
        SharedJanCustomCodeDialog.tsx  -- JAN を他の商品が使っているときの確認（UI-01b-D26、D-141 proposed・実装済み）
```

既存 UI-01a は `src/features/products` 配下に実装済みのため、UI-01b も同 feature に置く。商品コード / 部門 / 取引先の入力は `ProductForm` 内のセクションに inline で持たせ、shared 化は複数画面で再利用が必要になった時点で別 PR に切り出す。`ProductForm` は内部の `FormSection`（h2 見出し + Separator + 説明）で「商品の識別」「分類と取引先」「価格」「在庫」を構成し、edit mode だけ第 5 セクション「価格履歴」を末尾に加える（UI-01b-D10 / UI-01b-D20）。

## 7.3 Route / State

### `/products/new`

- mode: `create`
- search params:
  - `returnTo?: string`
- 初期値:
  - `jan_code = ""`
  - `name = ""`
  - `department_id = null`
  - `selling_price = 0`
  - `cost_price_centi = 0`
  - `tax_rate = "10"`
  - `stock_unit = "pcs"`
  - `initial_stock = 0`
  - `maker_code = ""`
  - `supplier_id = null`
  - `pos_stock_sync = true`

### `/products/$code/edit`

- mode: `edit`
- path param:
  - `code: string`
- search params:
  - `returnTo?: string`
- 初回表示時に `commands.getProduct(code)` を呼ぶ。
- `product_code`, `jan_code`, `stock_quantity`, `created_at`, `updated_at` は表示のみ。`stock_unit` は長さの単位の商品だけ表示のみ（個数の単位の商品は個数の単位どうしで変えられる。UI-01b-D23）。

## 7.4 Command / DTO Contract

UI-01b は以下の generated binding を使用する。

| Command | Existing backend | UI-01b usage |
|---|---|---|
| `commands.getProduct(productCode)` | generated 済み | edit 初期表示 |
| `commands.listDepartments()` | generated 済み | 必須部門候補 |
| `commands.listSuppliers()` | generated 済み | 任意取引先候補 |
| `commands.createSupplier(name)` | 実装 A で生成 | 新しい取引先（メーカー/ブランド）の追加 |
| `commands.listPriceHistory(productCode, limit)` | 実装 A で生成 | edit mode の価格履歴（直近 10 件 / すべて表示 100 件） |
| `commands.createProduct(req)` | generated 済み | create 保存 |
| `commands.updateProduct(productCode, req)` | generated 済み | edit 保存 |
| `commands.toggleDiscontinue(productCode)` | generated 済み | edit 廃番 / 復帰 |

`ProductCreateRequest` / `ProductCreateResult` / `ProductUpdateRequest` / `ProductUpdateResult` / `Supplier` は `specta::Type` を付与し、`src/lib/bindings.ts` に生成して commit する。

`PRODUCT-PATCH-D1` により、`commands.updateProduct` の入力は全property optionalのgenerated `ProductUpdateRequest_Deserialize`を正とする。`ProductUpdateRequest_Serialize`、手書きの `Partial` alias、保存時の型assertionをcommand入力契約の代用にしない。

## 7.5 Form Behavior

### Create

1. departments / suppliers を取得する。
2. 商品コード欄:
   - JANコードあり: JANコードを入力する。保存時は `jan_code=Some(value)` になり、BIZ が `product_code=jan_code` として登録する。
   - JANコードなし: JAN欄を空にし、独自コード発番対象部門を選ぶ。UI は「保存時に自動発番」と表示する。
   - JAN欄が空で、選択部門に `code_prefix` がない場合は保存しない。
   - JAN 欄の全角数字は UI-01b-D16 に従い半角へ正規化する（composition 中でない onChange と onCompositionEnd の 2 点で適用、paste 経由を含む。composition 中は不加工）。
3. 必須項目を入力する。
4. 長さの単位（`m`）に変えた場合、`pos_stock_sync=false` を提案する。個数の単位に変えた場合は既定の `true` を復元する。自動提案は `onPosStockSyncSuggest`（touched を立てない）経路を使い、利用者の checkbox 操作（touched を立てる `onPosStockSyncChange`）と区別する。利用者が toggle を触った後（touched=true）はどちらの単位変更でも提案を発火しない。
4b. `plu_target`（レジにバーコード登録する）は JAN 欄の値から初期値を提案する: 13 桁数字なら on、それ以外（空・不正形式）は off。利用者は変更できる。自動提案は pos_stock_sync と同じ touched 区別パターンを使う（D-028。詳細仕様と実装は後続 R3 PR / UI-01b 実装 PR で確定）。判定の candidate は UI-01b-D18 に従い trim + 正規化適用後の値とする（全角 13 桁も正規化後に on 提案）。
5. 保存時に `commands.createProduct(req)` を呼ぶ（`shared_jan_custom_code: false`）。入れた JAN を他の商品が使っていれば UI-01b-D26 のダイアログを開き、利用者が `独自コードで登録する` を押したときだけ `shared_jan_custom_code: true`・`pos_stock_sync: false` で送り直す。
6. 成功時は保存された `product_code` を含む成功 toast（`toast.success`、`id: "product-save-success"`、`duration: 5000`）を navigate より前に発火し、safe な `returnTo` または `/products` へ遷移する。safe な `returnTo` は `/products` 一覧 route とその search params のみ（UI-01b-D14）。

### Edit

1. `commands.getProduct(productCode)` で既存商品を取得する。
2. editable fields:
   - `name`
   - `department_id`
   - `selling_price`
   - `cost_price_centi`
   - `tax_rate`
   - `maker_code`
   - `supplier_id`
   - `pos_stock_sync`
   - `stock_unit`（個数の単位の商品だけ。候補は個数の単位だけ、UI-01b-D23）
   - `plu_target`（D-028。off→on に変更した場合、その商品は PLU 未反映（plu_dirty=1）扱いになり UI-08 の差分に現れることを補足文言で示す）
3. read-only fields:
   - `product_code`
   - `jan_code`
   - `stock_quantity`
   - `stock_unit`（長さの単位の商品。UI-01b-D23）
4. 保存時は変更されたfieldだけをgenerated `ProductUpdateRequest_Deserialize`へ入れる（PRODUCT-PATCH-D1）。
   - 通常field: unchangedはomitted、valueは設定。nullもbackendではno updateだが、unchangedの代わりにnullを送らない。
   - supplier未指定へ戻す: `supplier_id: null`（Rustの `Some(None)`、clear）。
   - maker_code空へ戻す: `maker_code: null`（Rustの `Some(None)`、clear）。
   - clear可能fieldのunchangedはomittedにし、nullと区別する。
5. 廃番 / 復帰は edit mode だけに出す。状態は色だけでなく「廃番」「表示中」の日本語 badge と button label で示す。「廃番にする」は確認ダイアログ（`DiscontinueConfirmDialog`）を通し、「表示に戻す」は確認なしで直接実行する（UI-01b-D13）。
6. 第 5 セクション「価格履歴」で `commands.listPriceHistory(productCode, 10)` の結果を新しい順に表示する。「すべて表示」で limit 100 を再取得する。create mode では表示しない（UI-01b-D20）。

### 取引先ピッカーからの追加（Create / Edit 共通、UI-01b-D21）

- 取引先の選択は取引先ピッカー dialog（DSR-24）を経由し、『新しい取引先を追加』は picker dialog 内から既存 `CreateSupplierDialog`（`src/features/suppliers/components/CreateSupplierDialog.tsx` を正、`onCreated: (supplier) => Promise<void>` 契約へ統合）を開く。現行の inline 常設パネル（`showSupplierInput` state、独立した 3 つ目の実装）は撤去し、trim・空文字拒否・同名衝突・失敗時入力保持の validation は `CreateSupplierDialog` に委譲する。
- 入力 name は trim し、空文字は field error として CMD を呼ばない
- `commands.createSupplier(name)` 成功後は `listSuppliers` を再取得し、返された supplier を選択状態にできる
- 追加失敗時は入力を保持して再試行できる。既存の商品 form 保存値を失わない
- 改名・統合は UI-15（[78-ui-supplier-management.md](78-ui-supplier-management.md)）で扱う。約 80 社の事前一括投入は扱わない

## 7.6 Validation / Error / Recovery

Frontend validation:

- `name`: 空文字 → `商品名を入力してください`
- `department_id`: 未選択 → `部門を選択してください`
- `selling_price`: 0 未満または整数でない → `売価は0以上の整数で入力してください`。`999999` を越える → `売価が大きすぎます`
- `cost_price_centi`: 原価の欄が `^[0-9]+(\.[0-9]{1,2})?$`（前後の空白を除く）に合わない → `原価は0以上で、小数は2桁までで入力してください`。`999999.99` を越える → `原価が大きすぎます`（SPEC-UNIT-D5。欄の値は円、送る値は 1/100 円）
- `initial_stock`: create mode で 0 未満または整数でない → `初期在庫は0以上の整数で入力してください`
  - `m` の商品は m で小数 2 桁まで受けて cm に直す（0 は受ける、既定の `0` のまま登録できる）。error の文は [共通規則](10-common-rules.md) SPEC-UNIT-D3 の 0 を受ける欄の文 `数量は0以上で、小数は2桁までの m で入力してください`。個数の単位の商品は上の文。在庫の数量が `999999999` を越える → `在庫数が大きすぎます`
- `tax_rate`: `"10" | "8" | "0"` 以外 → `税率を選択してください`
- `stock_unit`: create mode で 10 個の code（UI-01b-D22）以外 → `数量単位を選択してください`。edit mode の個数の単位の商品は select が個数の単位だけを出す（UI-01b-D23。種類をまたぐ値は BIZ が拒み、保存の error に出す）
- `jan_code` blank + `department.code_prefix == null` → `JANコードを入力するか、独自コード発番対象の部門を選択してください`
- `jan_code` 非 null（create のみ、UI-01b-D17）: trim と正規化を適用した後の値が ASCII 数字 8 桁または 13 桁でない → `JANコードは13桁または8桁で入力してください`
- `jan_code` 非 null（create のみ、UI-01b-D17）: 桁数を満たすがモジュラス 10 チェックディジット整合でない → `JANコードのチェックディジットが一致しません。入力値を確認してください`
- 上記 2 検査の実体は `src/features/products/lib/jan-code.ts`（named export、UI-01b-D18 の suggest 評価 pipeline と共用）が所有し、`product-form-request.ts` への inline 実装は禁止する。

**enum 契約化（D-061）**: `tax_rate` / `stock_unit` の SSOT は generated union で、frontend は bindings 由来型を利用する。select guard・validation 文言・UI 挙動は不変。

Error recovery:

- 部門候補取得失敗: 部門必須のため保存不可。再読み込み / 一覧へ戻る導線を出す。
- 取引先候補取得失敗: 警告を出す。取引先未指定なら保存可能。
- `getProduct` not found: form を出さず「商品が見つかりません」を表示し、一覧へ戻る導線を出す。
- save validation / duplicate: field error または form-level alert を出し、入力値は保持する。ただし create の kind `duplicate`・field `jan_code`（JAN を他の商品が使っている）は UI-01b-D26 のダイアログで出す。
- save internal error: form-level alert を出し、保存ボタンを再度押せる状態に戻す。
- 取引先追加エラー: inline error を出し、取引先名と商品 form の入力を保持して再試行可能にする。
- 商品 form の初期値 / 読込み値を baseline として `useUnsavedChangesWarning` へ接続し、create / update 成功時は一覧遷移前に提出値へ baseline を同期して離脱ガードを解除する（UI-USW-D1）。

## 7.7 Non-scope / Follow-up

- 商品コードの手入力登録。
- edit mode での `stock_quantity` 変更。
- 長さの表示の切替 UI（長さの単位は `m` だけで、表示は m の形。SPEC-UNIT-D3）。
- dedicated scanner UX / 連続スキャン検知。
- supplier / department select の shared component 化。
- 部門 17（本）のバーコードなし本・ISBN-10 本の登録経路（owner 裁定 2026-08-11 で対象外。Plans.md backlog で追跡、要望発生時に code_prefix 付与 or ISBN-10 対応を再裁定）。

## 7.8 Test Focus

- route mode: `/products/new` は create、`/products/$code/edit` は edit。
- generated command: `createProduct` / `updateProduct` / `toggleDiscontinue` / `listSuppliers` が binding に存在する。
- generated patch contract: `ProductUpdateRequest_Deserialize`の全propertyがoptionalで、builderの戻り値と`commands.updateProduct`入力がこの型へcastなしで直結する（PRODUCT-PATCH-D1）。
- create payload: JANあり / JANなし自動発番 / prefixなし部門 validation。
- edit payload: read-only fieldsを送らず、通常fieldのunchangedをomitし、`stock_unit` は登録の単位が個数の単位の商品で変えたときだけ送り（UI-01b-D23）、`supplier_id` / `maker_code`のomitted/null/valueを区別できる。
- 長さの単位（`stock_unit='m'`）時の `pos_stock_sync=false` 提案と手動 override。
- 原価の 1/100 円（`83.33` → 8333、修正の画面の初期値 `83.33`）、税抜の label と説明、「袋の値段から出す」の補助（個数の単位だけ、`m` で閉じて隠れる、開き直すと空、保存の request に入らない）（UI-01b-D22・D24、SPEC-UNIT-D5）。
- 部門取得失敗 / 取引先取得失敗 / `getProduct` not found / duplicate save error の復旧。
- UI-01b-D26: kind `duplicate`・field `jan_code` でダイアログが開き BIZ の message を出す（field の無い `duplicate` は form-level alert のまま）。`独自コードで登録する` は `shared_jan_custom_code: true`・`pos_stock_sync: false` で送り直し、成功で toast（`warnings` の行を含む）と一覧への遷移。`戻る` は送らずに入力を保つ。`code_prefix` の無い部門では主の button が無く部門を名指す。
- 廃番状態は日本語 badge + button label で示し、色だけにしない。
- form は create で基本 4 セクション、edit で「価格履歴」を加えた 5 セクションに分かれ、h2 見出しを持つ（UI-01b-D10 / D20）。
- read-only 入力（商品コード / edit の JANコード・現在庫）は `readonly` 属性を持ち、数量単位 select は、個数の単位の商品は個数の単位だけの候補で変更でき、長さの単位の商品は `disabled` で押せず今の語が読める（UI-01b-D11・UI-01b-D23）。
- 必須項目（商品名・部門・売価・原価、create 時は初期在庫）のラベルに `（必須）` が含まれる（UI-01b-D12）。
- 「廃番にする」は確認ダイアログを出し、キャンセルで状態が変わらない。「表示に戻す」は確認なしで直接実行する（UI-01b-D13）。
- 保存成功時に `toast.success`（`id: "product-save-success"`）が navigate より前に発火する（UI-01b-D14）。
- UI-01b-D20: edit mode のみ第 5 セクション「価格履歴」が表示され、直近 10 件と「すべて表示」100 件を新しい順に取得する。create mode では表示しない。
- UI-01b-D21: 取引先ピッカー dialog 内の「新しい取引先を追加」から `CreateSupplierDialog` を開き、trim・空文字拒否・同名既存行返却・候補再取得・失敗時入力保持が成立する。改名・統合は UI-15（[78-ui-supplier-management.md](78-ui-supplier-management.md)）で扱い、約 80 社の事前一括投入は扱わない。
- Windows native L3: 日本語入力、Tab 移動、保存後遷移、廃番 / 復帰の視認性。

## 7.9 変更履歴

| 日付 | 版 | 内容 |
|---|---|---|
| 2026-10-31 | JAN が重複したときの独自コードでの新規登録の lane の実装（PR #212） | UI-01b-D26 と §7.2 の file tree の印を「D-141 proposed・実装済み」にした（D-141 の Status は owner の確認まで proposed）。ダイアログの高さの上限と文の scroll・button の固定を UI-01b-D26 に足した（Final Review の是正）。 |
| 2026-10-27 | JAN が重複したときの独自コードでの新規登録の lane の起票（D-141） | UI-01b-D26（JAN を他の商品が使っているときの確認のダイアログと、独自コードで送り直す道）を追加（proposed・未実装）。冒頭の共有 JAN の項の field を `pos_stock_sync` だけにし、UI-01b-D25 の「商品コードの重複で止まる」、§7.5 Create 5・§7.6・§7.8 を合わせた。 |
| 2026-10-27 | 画面の小口の実装（PR #210） | D-118 の節の 2 つ目の項（在庫連動を始める拒否の案内の場所・見出しと文・focus）を実装済みの記述にした（[Plan Packet](../plans/2026-10-27-stocktake-screen-fixes.md)）。 |
| 2026-10-21 | 原価の lane の実装（PR #184） | UI-01b-D22 の 10 個の単位・税抜の label と説明・原価の 1/100 円の入力、UI-01b-D24 の補助、UI-01b-D6 の長さの単位（m）を現行の記述に直した。§7.6 の `cost_price_centi` の検証の文と上限、§7.7・§7.8 の `cm` の記述を直した。 |
| 2026-10-21 | 原価の lane の起票（D-128） | UI-01b-D22 に原価の lane の後の原価の label（`原価（税抜、必須）`）と価格の節の説明を置いた（原価は税抜、店の答え TD-331）。UI-01b-D24 に、補助を開き直すと 2 つの欄が空から始まることを足した（proposed・未実装のまま）。 |
| 2026-10-17 | JAN の別名の design lane（D-122） | UI-01b-D25（商品フォームは前の JAN の問いを出さず、edit の JAN は読取専用のまま、create で別名の JAN を入れたときの扱い）を追加。 |
| 2026-10-17 | 単位の語の直しの lane の起票 | UI-01b-D23 に、送る判定を登録の単位で行うこと、在庫の節の説明文 3 つ、「現在庫」の欄の語、productUpdate の無効化の範囲の外の画面の語を足した（proposed・未実装、D-119。D-119 の Final Review の P3〈Fable #1〉の是正を含む）。Plan Review round 1 の是正で、長さの単位の商品の select の候補の文を外し（disabled の並びは決めない）、UI-01b-D11 と §7.8 の「数量単位 select は disabled を維持」に本 lane の後の規則を proposed で添えた。 |
| 2026-10-17 | 単位・数量・原価の精度の見直しの design lane | UI-01b-D23（個数の単位の商品の単位の語を edit mode で直す）と UI-01b-D24（原価の「袋の値段から出す」の補助）を追加し、UI-01b-D5 の読取専用を D23 が置き換えると書いた（proposed・未実装、D-119）。 |
| 2026-10-08 | 単位の拡張 design lane | UI-01b-D22 の `玉`・原価の小数を owner の決定（D-113 J1・J2、TD-195）に改めた。 |
| 2026-10-07 | 単位の拡張 design lane | UI-01b-D22 を追加（proposed・未実装、D-113）: 数量単位 12 種、長さの単位の価格欄の `1 m あたり`、初期在庫の m 入力、価格の節の説明の文。 |
| 2026-09-10 | PR #50 | 取引先ピッカー runtime を反映し、CreateSupplierDialog の canonical path を suppliers 配下へ統合。 |
| 2026-09-10 | PR #49 取引先ピッカー design | 取引先選択を DSR-24 経由へ改訂。独立した 3 つ目の inline 実装の撤去と CreateSupplierDialog への validation 委譲を明記。 |
| 2026-08-22 | 価格改定支援 design-first | SPEC-PRV-D6 / D9 を昇格。UI-01b-D7 を inline 追加対応へ改訂し、UI-01b-D20（価格履歴）/ D21（取引先追加）を追加。 |
| 2026-08-11 | JAN 専用欄正規化 design | UI-01b-D16〜D18 を追加（JAN 欄の全角→半角正規化 / 保存時 JAN-8/13 + チェックディジット validation / PLU 提案の正規化後評価と BIZ 二重実装契約）。§7.5 / §7.6 / §7.7 を整合。実装は後続 PR。 |
| 2026-08-03 | UI safety net implementation | 商品 form の dirty 判定を共通離脱ガードへ接続し、保存成功時の baseline 同期を実装。 |
| 2026-06-09 | UI-01b Design Phase | TanStack Router route、generated command 方針、supplier 候補、JANなし商品コード、edit read-only fields、cm / m defer、Windows native L3 を Design Phase 基準で整理。 |
| 2026-06-12 | UI-01b polish | UI-01b-D10〜D12 を追加（4 セクション分割、read-only 表示、必須項目ラベル）。§7.2 を実装構成へ更新。 |
| 2026-06-12 | UI-01b polish | UI-01b-D13 を追加（廃番にするの確認ダイアログ、表示に戻すは直接実行）。§7.5 step 5 を整合。 |
| 2026-06-12 | UI-01b polish | UI-01b-D14 を追加（保存成功 toast）。§7.5 Create step 6 を整合。 |
| 2026-06-12 | PR #95 Codex P2 対応 | §7.5 Create step 4 を更新: 自動提案（`onPosStockSyncSuggest`）と利用者 override（`onPosStockSyncChange`）を分離。pcs→cm で false 提案、cm→pcs で true 復元。touched=true 後は提案を発火しない。 |
| 2026-07-03 | D-028 Design Phase | §7.5 Create 4b と Edit editable fields に `plu_target`（レジにバーコード登録する）を追加。JAN 13桁数字判定からの初期値提案 + 変更可、off→on で PLU未反映扱い。実装は後続 R3 PR。 |
| 2026-06-12 | PR #95 lost update 修正 | `ProductFormProps.onValuesChange` を `(values: ProductFormValues) => void` から `React.Dispatch<React.SetStateAction<ProductFormValues>>` に変更。`update()` を functional update（`onValuesChange((prev) => ({ ...prev, [key]: value }))`）に変更し、同一 tick の連続 state 更新（単位変更 + POS 同期提案）による lost update を解消。 |
| 2026-06-25 | UI-02 L3 feedback | 説明書作成時に、商品登録は商品マスタ作成、入庫記録は通常仕入の在庫加算であることを明記する注意を追加。 |
| 2026-07-29 | PRODUCT-PATCH-D1 follow-up | §7.1 / §7.4 の UI-01b command statusを現行generated bindingに同期。 |
| 2026-08-16 | PR #79 | Design-system adjacency: UI-01b-D13の先例参照をDSR-07 / component catalog ⑧へ正本化。 |
