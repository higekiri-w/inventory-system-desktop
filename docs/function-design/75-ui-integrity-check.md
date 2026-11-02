> **親文書**: [FUNCTION_DESIGN.md](../FUNCTION_DESIGN.md)
> **入力ドキュメント**: ARCHITECTURE.md（UI-13 / REQ-904）、SCREEN_DESIGN.md（在庫整合性検証）、36-biz-integrity-check.md（BIZ-07）、42-cmd-sales-stocktake.md §22.7（CMD-11 integrity）、59-ui-shared-patterns.md（共有 UI 部品）
> **Plan Packet**: [archived plan](../archive/plans/2026-07-15-ui13-integrity-check.md)（Design Phase 出典）

## 75. UI-13: 在庫整合性検証画面

> **計数の置き場と在庫の求め直し（D-118。(a) と b1 で実装済み）**: 比べる相手は「記録から求め直した値」（[36](36-biz-integrity-check.md)）で、比べる項目は数・要再確認の印・範囲の 4 つ。差異の一覧は商品ごとに、違う項目（`differing_fields`: 在庫数 / 要再確認 / 範囲）を文字で示し、保存している値（写し）と記録から求め直した値を並べる（数が同じで印や範囲だけが違う商品も一覧に出し、「差異なし」としない）。補正の確認 dialog と結果は、選んだ商品の 4 項目の補正前と補正後を出す。操作ログ（UI-11c）の `integrity_fix` の行は 4 項目の old / new を表示する（[74](74-ui-operation-logs.md) UI-11c-D14）。移行の知らせ: 開いたときに操作ログを `operation_type: "integrity_fix"` で読み（直近確認日時の取得〈`integrity_check`〉とは別の呼出し。§75.8a）、移行が写しを直した `integrity_fix` の行（detail_json の `trigger` が `migration`。[36](36-biz-integrity-check.md)・[22 §19](22-mnt-migration.md)）があれば、画面の上に、その日時と直した商品の数と「直した前後は操作ログで見られます」を文字で出し、操作ログ（UI-11c）への導線を置く（チェックの結果とは別の欄。色だけで示さない。閉じる操作は持たない）。下の §75.1〜§75.15 はこの契約で書く。共有 JAN の一覧（b1 で実装済み。[36](36-biz-integrity-check.md) の `shared_jan_conflicts`）: チェックの結果に組があれば、差異の一覧とは別の欄に「在庫連動の商品と同じ JAN の商品があります。レジの売上で在庫が減りません。どちらかの商品の JAN を直すか、在庫連動を外してください。」と、JAN ごとにその JAN を持つ商品コードを並べる（warning の Alert。icon と文字で示し、色だけで示さない）。写しの差異が無ければ「差異なし」の表示はこの欄と別に出す。組の欄は補正の操作を持たない。

### 75.1 概要

- **対応 REQ**: REQ-904
- **route**: `/settings/integrity`
- **PageHeader title**: 「在庫整合性検証」（サイドバーの表記「整合性検証」〈52 §52.3 メニュー正本表〉とは別に画面見出しとして正本化する）
- **呼び出す CMD**: `runIntegrityCheck()` / `fixIntegrity(productCodes)` / `listLogs(query)`
- **主動線**: idle で明示実行 → running overlay → completed で差異なし、または差異一覧 → 行単位選択 → 確認dialog → fix結果 → 必要なら手動再チェック。

**関数要求**: 商品の保存している在庫（写しの 4 項目）と、記録から求め直した在庫の整合性を、利用者の明示操作で検証する。補正は行単位の明示選択と確認dialogを必須とし、選択した商品コードだけを既存CMD-11へ渡す。成功、部分スキップ、失敗、再試行を日本語で区別し、色だけで状態を表現しない。

**シグネチャ**:

```ts
export function IntegrityCheckPage(): JSX.Element;
```

**処理ステップ**:

1. mount時にidle状態を表示し、`listLogs`から直近確認日時と移行の知らせ（§75.8a）を取得する。
2. 利用者の明示操作で`runIntegrityCheck`を実行し、running中は画面内overlayで他操作を抑止する。
3. 完了後、差異0件なら成功表示、差異ありなら100件単位の一覧を表示する。
4. 利用者が補正対象を行単位で選択し、確認dialogで商品コードと「保存している値 → 記録から求め直した値」の内訳を確認する。
5. confirm時だけ選択した商品コードを`fixIntegrity`へ渡し、補正結果・補正済み表示・部分スキップ警告を表示する。
6. 再確認は利用者が「再度チェック」を押した場合だけ実行し、旧結果と選択を破棄して新しいcheckを開始する。

**エラーハンドリング**: §75.9を参照。check失敗時は日本語messageと再試行、fix失敗時は選択を保持した日本語messageと補正再試行を表示し、直近確認日時の取得失敗はcheck主動線を塞がない。

### 75.2 Design Decisions

| ID | 決定 | 理由 / 棄却案 |
|---|---|---|
| UI-13-D1 | routeは`/settings/integrity`。URL search stateは持たず、check result、page、選択、fix resultは画面ローカルの一時状態とする。mount時は必ずidle。 | 結果はその場のDB状態を検証した実行セッション限り。URLへpage/resultを置くと古い結果を再現可能に見せるため棄却。 |
| UI-13-D2 | 直近確認日時は`listLogs({ page: 1, per_page: 1, operation_type: "integrity_check", start_date: null, end_date: null })`の最新行から導出する。 | 専用setting keyはbackend変更とlogとの二重管理を生む。`integrity_fix`は対象に含めない。 |
| UI-13-D3 | 差異行ごとに「補正する」checkboxを置く。select-allは置かず、未選択時は確定disabled。確認dialogへ商品コードと「保存している値 → 記録から求め直した値」を列挙し、confirm時だけ選択codeを送る。長さの商品の在庫（範囲の端を含む）・補正の結果は m で出す（[共通規則](10-common-rules.md) SPEC-UNIT-D3 の「表示する所」の表の 8・11 行目）。 | 「1件ずつ確認」は盲目的な全件補正を防ぐ意図。明示列挙付き複数選択は反復modalより確認しやすい。 |
| UI-13-D4 | check/fix中は画面内overlay、Progress、処理中文言を出す。pending中は実行・選択・確定・retryを無効にし、handlerでも二重発火を防ぐ。 | 旧結果・選択と新結果の混入を防ぐ。cancel可能には見せない。 |
| UI-13-D5 | fix成功後は自動再チェックしない。`fixed_count` / `adjustments` summaryと「補正済み」Badgeを表示し、「再度チェック」でのみ新checkを始める。 | checkは重い処理であり、利用者の意図なく暗黙実行しない。 |
| UI-13-D6 | `skipped_count > 0`は独立warningで表示する。CmdErrorは日本語messageとretryを表示し、fix失敗時の選択を保持する。 | 部分未補正と再選択負担を成功表示へ埋没させない。 |
| UI-13-D7 | frontendは生成済み`commands.*`と`unwrapResult`のみを使う。手書き`invoke`とfrontend独自DTOを禁止する。 | Rust型をwire contractのSSOTにする。 |
| UI-13-D8 | 「差異なし」「差異あり」、違う項目の語（「在庫数」「要再確認」「範囲」を「・」でつなぐ）、「補正済み」「一部未補正」を日本語で示し、icon/Badge/Alert/位置とsemantic色を併用する。違う項目の Badge は outline で色の tone を持たない（語で示す）。 | 非IT・高齢operatorが色識別に依存せず状態を言い分けられることを機能要件とする。 |
| UI-13-D9 | 確定ボタン・確認dialogから「棚卸し」語彙を排する。確定ボタンは「補正を確定」、dialog titleは「在庫数を記録から求め直した値に合わせて補正します」、説明は「補正すると元に戻せません。選択した商品の保存している値を記録から求め直した値に合わせて更新し、操作ログに記録します。」とする。 | 旧dialog title（棚卸し方式での記録をoperatorに約束する文言）は、movementを作らない直接更新 + 操作ログ記録という実挙動（BIZ-07-D2/D3、[D-051](../decision-log.md)）と乖離していた。表現と実挙動の一致を優先する。D-118 の (a) で比べる相手が記録から求め直した値になったので、語は一覧の列と同じ「保存している値」「記録から求め直した値」にした（旧「システム在庫」「入出庫の合計」は計数のある商品で意味が合わない）。UI-13-D8の色非依存原則は維持。 |
| UI-13-D10 | fix 成功時の consumer query invalidation は [D-052](../decision-log.md) C12 と `src/lib/invalidation-contract.ts` を正本とする。latest-check query は D-052-E1 の operation_logs 系除外に従う。 | `products.stock_quantity` の直接補正を読む consumer を一貫して stale 化し、page 側の key 列挙を禁止する。 |

### 75.3 State / Lifecycle

```ts
type IntegrityPhase = "idle" | "running" | "completed";
```

| Event | Before | After | 保持 / 破棄 |
|---|---|---|---|
| mount / remount | — | idle | result、selection、fix result、errorなし |
| check開始 | idle / completed | running | 旧result、selection、fix result、errorを破棄 |
| check成功 | running | completed | 新result、page=1、selection空 |
| check失敗 | running | 実行前phase | errorを表示。retryは新規check |
| fix開始 | completed | running相当の抑止 | resultとselectionを保持 |
| fix成功 | pending | completed | result、fix result、補正済みcode集合を保持。自動checkなし |
| fix失敗 | pending | completed | resultとselectionを保持しerror/retryを表示 |
| 再度チェック | completed | running | 旧result、selection、fix result、補正済み表示を破棄 |

- checkとfixを同時に走らせない。URL、localStorage、Zustand、Query cacheへresultを保存しない。
- 直近確認日時は画面表示時とcheck成功後に再取得する。取得失敗はcheck主動線を塞がない。

### 75.4 Data Flow / Wire Contract

```text
/settings/integrity
  └ IntegrityCheckPage
      ├ listLogs(integrity_check, latest 1) → OperationLog
      ├ listLogs(integrity_fix, latest 100) → OperationLog（移行の知らせ）
      ├ runIntegrityCheck()                 → IntegrityResult
      └ fixIntegrity(selected codes)        → IntegrityFixResult
```

- `IntegrityResult`: `mismatches`, `mismatch_count`, `checked_count`
- `StockCopy`: `stock_quantity`, `stock_recheck`, `stock_range_low`, `stock_range_high`（[36](36-biz-integrity-check.md)）
- `IntegrityMismatch`: `product_code`, `name`, `stock_unit`, `copy`（保存している値）, `derived`（記録から求め直した値）, `differing_fields`（`quantity` / `recheck` / `range`）
- `IntegrityFixResult`: `fixed_count`, `skipped_count`, `adjustments`
- `StockAdjustment`: `product_code`, `stock_unit`, `before`, `after`
- 値の書き方（一覧・dialog・結果で同じ）: 在庫数（`formatLengthOrPlain` で単位どおり）と、`stock_recheck` が true なら後ろに「（要再確認 {low}〜{high}）」（両端も単位どおり）。一覧の個数の商品は 3 桁区切り
- UIは補正量を再計算して送らない。fix入力は選択した`product_code[]`のみ。
- empty code集合はUIで到達不能にし、CMD/BIZ validationも防御境界として維持する。

### 75.5 Status Presentation

| 状態 | 主表示 | 非色シグナル |
|---|---|---|
| idle | 「整合性チェック実行」 | 説明文 + 主動線button |
| running | 「在庫データを確認しています」または「補正を記録しています」 | overlay + Progress + `role="status"` |
| 差異なし | 「差異はありません」 | Check icon + success Alert + checked件数 |
| 差異あり | 「差異が見つかりました」 | AlertTriangle icon + warning Alert + mismatch件数 |
| 違う項目 | 「在庫数」「要再確認」「範囲」（「・」でつなぐ） | 行の outline Badge の語（tone なし） |
| 移行の知らせ | 「更新のときに在庫数を記録から求め直し、{N}商品の在庫数を直しました（{日時}）。直した前後は操作ログで見られます。」 | Info icon + 既定の Alert（`role="status"`）+ 「操作ログを開く」 |
| 補正済み | 「補正済み」 | row Badge + fix summary |
| skippedあり | 「一部の商品は補正されませんでした」 | warning Alert + skipped件数 |

### 75.6 Difference Table / Pagination

業務値5列は、商品コード、商品名、違う項目、保存している値、記録から求め直した値。行ごとのcheckboxは別の操作列とする。差の列は持たない（数が同じで印・範囲だけが違う商品もあるため）。

- client-side paging は既定 100、`Select` で 50 / 100 / 200（L3-D6）。101件を既定値で表示する場合はpage 1に100件、page 2に1件。check再実行時と表示件数変更時はpage 1へ戻る。
- `Pagination`を`totalCount=mismatches.length`、`perPage={perPage}`（state、既定 100）で再利用する。
- tableは横overflow可能なwrapperと安定した列幅を持つ。商品名は回復不能なtruncateをしない。
- 違う項目は `differing_fields` の順（在庫数・要再確認・範囲）で語をつなぐ。
- UI-10棚卸し（73-ui-stocktake.md UI-10-D10）の列名「現在在庫」とは意図的に語彙を分ける。本画面は保存された数字自体の正しさを疑う場であり、「現在」と断言する語を使うと検証対象の数字を無条件に信頼しているように読める。「保存している値」は検証対象であることを示し、「記録から求め直した値」は計算根拠（入出庫・初期在庫・計数の記録）を伝える。

### 75.7 Selection / Confirmation / Fix Result

- checkboxのaccessible nameは「{商品コード}を補正する」、可視labelは「補正する」。headerにselect-allを置かない。
- 確定ボタン「補正を確定」（UI-13-D9）は選択0件またはpending中にdisabled。
- dialogはtitle「在庫数を記録から求め直した値に合わせて補正します」と説明（不可逆性 + 直接更新 + 操作ログ記録の3点、文言はUI-13-D9）を持ち、見出し「補正する商品（保存している値 → 記録から求め直した値）」の下に、選択行ごとに商品コード、商品名、`copy → derived`（§75.4 の値の書き方）を列挙する。
- cancelは副作用なし。confirmでのみ`fixIntegrity`を1回呼ぶ。
- success summaryはfixed件数と各adjustmentの`before → after`（§75.4 の値の書き方）を列挙する。
- `adjustments.product_code`だけを補正済み集合へ追加し、skipped codeを補正済みと表示しない。

### 75.8 Latest Check Time

- payloadは`page=1`, `per_page=1`, `operation_type="integrity_check"`, `start_date=null`, `end_date=null`固定。利用者入力を受けず、`per_page`の上限は1。
- 1件なら`created_at`の`T`を空白へ置換して表示。0件は「まだ実行されていません」、失敗は「取得できませんでした」。
- `integrity_fix`や他operation_typeの日時をfallbackに使わない。

### 75.8a Migration Notice

- payloadは`page=1`, `per_page=100`, `operation_type="integrity_fix"`, `start_date=null`, `end_date=null`固定（新しい順の最新 100 行。`per_page` の上限 200 の内）。直近確認日時とは別の呼出しで、query key は `queryKeys.operationLogs.list(payload)`。
- 行の中から、detail_json が JSON の object で `trigger` が `"migration"`、`fixed_count` が安全な整数のものを新しい順に探し、最初の行で知らせを出す（その後に別の `integrity_fix` の行があっても出す）。日時は `created_at` を `formatDateTime` で出す。`trigger` の無い行だけなら出さない。
- 読取りに失敗したら知らせを出さず、チェックの操作は止めない。閉じる操作は持たない。
- 移行の後に `integrity_fix` が 100 行を超えると古い移行の行を見つけない（開発・試験 DB にだけ起きる。記録は操作ログに残る）。

### 75.9 Error / Retry / Loading

| 状態 | 表示 / 回復 |
|---|---|
| check error | destructive Alert + CmdErrorの日本語message + 「再試行」 |
| fix error | destructive Alert + CmdErrorの日本語message + 「補正を再試行」。resultとselectionを保持 |
| latest log error | 補助文言だけを失敗表示し、主動線を塞がない |
| running | 画面内overlay、Progress、処理中文言。背景controlsをdisabled |

- `isInvokeError`なら`cmdError.message`、その他は日本語の汎用回復文言を表示する。
- retry handlerは二重実行guardを共有し、成功時にerrorを消す。

### 75.10 Accessibility / Keyboard / 非色

- 既存Button、Checkbox、AlertDialog、Table、Progressを再利用する。
- checkboxはlabelで商品を一意に特定し、Spaceで切替可能。dialogはRadixのfocus trapとEsc/cancelを使う。
- overlayは`role="status"`と可視文言を持つ。背景controlsはdisabledとし、pointer遮断だけに依存しない。
- 差異方向、結果、補正済み、skippedはtext/role/valueでtest可能な日本語labelを持つ。
- 初期1280x800、最小1024x720でtable overflowとdialog内訳scrollを確認する。

### 75.11 Traceability

- REQ-904 / UI-13-D1: initial idle、remount、rerun lifecycle。
- REQ-904 / UI-13-D2: latest integrity_check log導出。
- REQ-904 / UI-13-D3: selection、select-all不在、disabled、confirm内訳、selected codes。
- REQ-904 / UI-13-D4: overlay、操作抑止、二重発火防止。
- REQ-904 / UI-13-D5: fix summary、補正済みBadge、no auto recheck、手動再check。
- REQ-904 / UI-13-D6: skipped warning、CmdError、retry selection保持。
- REQ-904 / UI-13-D7: generated bindingsのみ、手書きinvoke不在、command登録一致。
- REQ-904 / UI-13-D8: status / 違う項目の日本語label（色だけで示さない）。
- REQ-904: 100件paging、5業務列（違う項目・保存している値・記録から求め直した値）。
- REQ-904 / §75.8a: 移行の知らせ（trigger、後の行、失敗の扱い）。
- REQ-902 / UI-11c-D14: 操作ログの before/after の形と記録済みの形。

### 75.12 Windows native L3

| # | 画面 / 到達手順 | 観測可能な合格基準 |
|---|---|---|
| L3-1 | synthetic DBで`/settings/integrity`を開きcheck実行 | 差異なし成功表示と直近確認日時が読める |
| L3-2 | 実行直後に他操作を試す | overlayと処理中文言が表示され、二重実行できない |
| L3-3 | idle / running / 差異なしをowner目視 | 非IT operatorが日本語文言、主動線、状態差を説明できる |

「差異あり → 選択補正」はDB直接操作によるfault injectionが必要なためL3対象外。component testと既存BIZ実SQLite testで担保する。

### 75.13 非目的

| やらないこと | 理由 / 責務 |
|---|---|
| POS部門別売上照合（REQ-403 / SP-403） | UI-13とは別のdeferred要求 |
| 自動・定期check、自動fix | 利用者の明示操作と確認が安全契約 |
| select-all / 全件一括補正 | 誤操作防御に反する |
| frontendで補正量を決定して送信 | BIZ-07がauthoritative |
| URL/localStorageへのresult保存 | ephemeral lifecycleに反する |
| CMD/BIZ/IO/DBロジック変更 | 既存contractで完結。不足時はDesign/Planへ戻す |

navigation（`src/config/navigation.ts`）の ui-13 entry は `/settings/integrity` で active 化済み（Amendment 4、2026-07-15）。owner の実機確認で `to: null` / `status: "pending"` のまま = 利用者が本画面に到達できない Goal Invariant 違反が発覚したため是正した。navigation からの除外は非目的ではない。

### 75.14 Adjacent Pattern Audit

| 観点 | 参照元 | UI-13採用 |
|---|---|---|
| PageHeader / retry Alert / table overflow | `OperationLogsPage` | 再利用 |
| Progress / pending disabled / confirmation dialog | `StocktakePage` | 再利用 |
| pagination | `Pagination` | client-side 既定 100 + `Select` で再利用 |
| generated command + unwrapResult | 既存features | 同一pattern。手書きinvoke禁止 |
| URL state | `OperationLogsPage`等 | ephemeralな別scenarioのため意図的に不採用 |
| query cache | logs / stocktake | latest logだけQuery。check/fix resultはlocal state |

### 75.15 Mutation / Anti-tautology Questions

1. 選択外codeをfixtureへ混ぜてもfix引数へ入らないか。
2. confirm前にfixを呼ぶmutationが失敗するか。
3. fix成功後にcheckを自動呼出しするmutationがcall countで失敗するか。
4. `skipped_count=0`と`>0`でwarning表示が変わるか。
5. retryでselectionをclearするmutationがcheckbox valueと再送引数で失敗するか。
6. 101件fixtureの先頭、100件目、101件目を別codeにしてslice境界を検出できるか。
7. 印・範囲だけの違いを「差異なし」にするmutation、違う項目を色だけで示すmutation、`copy`と`derived`を取り違えるmutationが失敗するか。
8. 移行の知らせの読取りの失敗でチェックを止めるmutationが失敗するか。

### 75.16 Negative-space Audit

- Q40の全アプリ共通Error Boundaryは本scope外。本画面固有のCmdError/retryを実装する。
- `src/config/navigation.ts`のui-13 entry有効化はAmendment 4で追加scope化済み（§75.13参照。原scopeはroute新設のみだったが、owner実機確認で到達導線の欠落がGoal Invariant違反と判明し是正）。
- fix成功と整合性再確認を同一視せず、「再チェック済み」を自動で主張しない。
- 差異0件はempty dataではなく正常な成功結果として表示する。

### 更新履歴

| 日付 | PR | 内容 |
|---|---|---|
| 2026-07-15 | UI-13 implementation | UI-13-D1〜D8、lifecycle、wire、paging、確認、部分失敗、retry、非色表示、L3境界を正本化 |
| 2026-10-18 | stocktake-count-runtime-a（PR #176、D-118 の (a)） | 比べる相手を記録から求め直した値と写しの 4 項目にし、一覧の列・違う項目の語・dialog と結果の前後・移行の知らせ（§75.8a）を現行に直した |
