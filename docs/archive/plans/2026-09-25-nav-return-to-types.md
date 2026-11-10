# Plan Packet: 画面遷移の戻り先（returnTo）を型で守る（R3）

2026-09-25 起草。出典は Backlog「やると決めたもの（順番未定）」の 2 行（base `85b18a04` の `docs/backlog.md:28` = `returnToLinkProps` の「値なし」sentinel を型で強制する〈PR #78 Final Review round 2 Opus P3-4 起源〉、`:30` = `StockMovementsPage.tsx` の `detailReturnTo` を router の href へ寄せるか〈PR #78 Non-scope 起源〉）。wave 13 の lane として Coordinator が起票を指示した。backlog の 2 行は closeout で完了にする（本 packet の Scope に backlog の編集を含めない）。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

- Phase: archive
- Risk: R3
- Plan Commit: 9adb9769ce849ec0dd2066a395240694b66753eb
- Amendments: none
- Coordinator: Opus 5.5（Claude Code main session）
- Writer: Opus 5.5 subagent（fork でない fresh context、worktree `.claude/worktrees/nav-return-to-types`、branch `agent/nav-return-to-types`）
- Plan Reviewer: fresh Opus 5.5 subagent + Codex（GPT-6 Sol、effort high）。互いに独立で Writer と別 context、後の reviewer に先の結果を見せない
- Final Reviewer: Fable 5.1（fresh context）+ Codex（GPT-6 Sol）。互いに独立な Double Audit、後の reviewer に先の結果を見せない。Codex の合否判定は是正後も外さない
- Final Review Minimum: 2
- Human Gate: ready,merge

manual なし: 利用者から見える画面は変わらない。戻り link の label・行き先・href は変更前と同じ（業務記録詳細 6 画面と在庫変動履歴の既存 test の期待 href は書き換えない〈AC4〉。`detailReturnTo` の出力は `/` 始まりの `returnTo` と `returnTo` なしで手組みと 1 byte も同じ。`/` 始まりでない不正な `returnTo` では詳細 link の href だけが変わり、在庫照会へ戻る先は既定の戻り先のまま〈Contract Probe 2、Boundary / Wire Contract の compatibility〉）。PR #78 が manual を付けた理由（在庫照会起点の 3 段往復の実クリックを自動 test が持たなかった）は、本 lane で実 router の往復 test T7 を足して自動化する。r4 なし: DB・command・data を触らない。

遷移記録（append-only）:

- kickoff → spec-check → design → plan-draft → plan-gate（2026-09-25、本 commit、plan-first、起草役 = Opus 5.5 subagent）: Risk R3（下記 Risk）。設計正本の改訂（DSR-15 の判定フロー、DSR-18 の判定フローと共通 helper の段落、`01-decision-rules.md` 更新履歴、66 UI-06c-D9 / §66.5 / §66.7）を同じ plan-first commit に入れた。owner の設計判断を要する論点は無い（画面の振舞いは不変で、決めたのは helper の型と直列化の方法だけ。Design Readiness 参照）。`docs/Plans.md` の「次の行動」に本 packet と Matrix の link を 1 行足した。Plan Review へ。
- plan-gate（2026-09-25、Coordinator の指示で是正）: Plan Review round 1 は Claude 側 approve、Codex 側 reject。plan-gate のまま packet を是正した。findings と裁定の詳細は round 2 の完了後に Review Response へ記録する。
- plan-gate → plan-approved → implementing（2026-09-25、Coordinator、本 commit）: Plan Review round 2 で Codex 側（GPT-6 Sol、high）が approve（P1/P2/P3 = 0）。Claude 側は round 1 で approve（P1/P2 = 0、P3 3 は round 1 の是正 `9adb9769` で反映済み）で、P3 だけのため round 2 は再投入しない。Plan Commit = `9adb9769`（是正後の承認版。plan-first `53f3f353`）。次は Writer（Opus 5.5 subagent）の実装。
- implementing → archive（2026-09-26、Coordinator、closeout）: PR #106 を helper 経由で merge（squash `42eddd0a`）。Final Review の 2 本（Codex GPT-6 Sol・Fable 5.1）とも approve、finding なし。packet と Matrix を archive へ移し、Plans.md と backlog を同期。

## Owner Effort Budget

- 介入回数上限: 3（見込み: Plan Review の Codex relay 1、Final Review の Codex relay 1、Ready・merge 1）
- 実働時間上限: 30分
- relay 往復上限: 2
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
route/search state を変える（`docs/DEV_WORKFLOW.md` Risk Tiers の R3「route/search state」）。戻り link の props を作る共通 helper の署名と戻り値の型（業務記録詳細 6 画面と在庫変動履歴の 7 画面が使う）と、在庫変動履歴が業務記録詳細へ送る `returnTo`（入れ子の `returnTo` を含む URL search）の組み方を変える。backlog `:28` も R3 と記載。DB・Tauri command DTO・`bindings.ts`・route schema は変えない。R4 には当たらない。

## Goal

Goal Invariant:

### 最小完了条件

- 業務記録詳細 6 画面の「前の画面へ戻る」と在庫変動履歴の「在庫照会へ戻る」が、`returnTo` を検証できないとき空の `to` の link（押しても遷移しない）を描画する経路が型の上で無くなる: `returnToLinkProps` は不合格を `null` で返し、呼出側が既定の戻り先を書かないと `npm run typecheck` が落ちる。
- 在庫照会（数字だけの商品コードを含む）→ 在庫変動履歴 → 元記録の業務記録詳細 → 「前の画面へ戻る」→「在庫照会へ戻る」の往復で、変更前と同じ画面・同じ検索条件・同じ選択行に戻る。各段の href は変更前と文字列一致する。

### 失敗定義

- 戻り link のどれかで、行き先の pathname か search が変更前と変わる（既存 test の期待 href を書き換えないと通らない）。
- 入れ子の `returnTo` を運ぶ往復のどこかで `returnTo` が落ち、「在庫照会へ戻る」が既定の `/stock?q=<code>&selected=<code>` に落ちる。
- `returnToLinkProps` の戻り値の型から `null` が外れても、test と typecheck が green のまま残る。

### 非目的

- 画面の見た目・label・遷移先の変更、`history.back()` 化（DSR-17 (a) で不採用）。
- 商品一覧の `src/features/products/lib/return-to.ts`（typed parse-back、DSR-18 で存置）と、`normalizeReturnTo`（production caller 0、PR #78 で primitive として存置）。
- `returnTo` の送信側 site（`useRouterState` の href を送る既存 site）の変更、route schema（`returnTo: z.string().max(500)`）の見直し、`returnTo` の長さ・入れ子の深さの再設計。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

route/search state の契約（戻り link の組み方）を変える packet なので、変更後も成り立たなければならない通常の操作列を書く。画面の見え方と操作は変わらない。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 在庫照会で数字だけの商品コード（例 `2099000000019`）を検索し、その商品を選択している | 詳細の在庫変動履歴への link → 元記録（例「廃棄・破損 #7」）→「前の画面へ戻る」→「在庫照会へ戻る」 | 在庫変動履歴へ、次に在庫照会へ、出発時と同じ検索語・同じ選択行で戻る | 各段の `location.href` が行きの href と文字列一致し、在庫照会の search が `{ q: "2099000000019", selected: "2099000000019" }`（string）（T7） | Contract Probe 2（直列化の同一性） |
| 在庫変動履歴で期間・種別・page を絞り込んでいる（在庫照会から来た `returnTo` を持つ） | 元記録 → 業務記録詳細 →「前の画面へ戻る」 | 同じ期間・種別・page の在庫変動履歴へ戻り、「在庫照会へ戻る」も在庫照会を指したまま | 元記録 link の href が変更前と同一（T6、既存 test の期待値を書き換えない）、`detailReturnTo` が受け側 schema で元の search に戻る（T5） | Contract Probe 2 |
| 入出庫履歴・作業画面の recent list・操作ログから業務記録詳細を開いている（正当な `returnTo`） | 「前の画面へ戻る」 | 遷移元の一覧・作業画面へ、同じ検索条件で戻る | 既存 T11 系・`ReturnToFlow.test.tsx` T8 / T10 が期待値を変えずに pass | なし |
| 業務記録詳細を bookmark・直接 URL で開いている（`returnTo` が欠落・不正） | 「前の画面へ戻る」 | 入出庫履歴 `/inventory/records` へ戻る | 既存 T11 系の不正 case・`ReturnToFlow.test.tsx` T9 が期待値を変えずに pass | なし |
| 在庫変動履歴を直接 URL で開いている（`returnTo` が欠落・不正・`/stock` 以外） | 「在庫照会へ戻る」 | `/stock?q=<code>&selected=<code>` で在庫照会の同じ商品を開く | 既存 `SPEC-UI06C-D9-R1` 7 case と GA5 T2-num が期待値を変えずに pass | なし |
| 開発者が新しい詳細画面に戻り link を足す | `returnToLinkProps(returnTo)` の結果をそのまま `<Link {...x}>` へ渡し、既定の戻り先を書かない | 型検査で止まり、空の `to` の link は出荷されない | `npm run typecheck` が TS2322 で落ちる（Contract Probe 1）、戻り値の型を固定する T4 | Contract Probe 1 |

この packet を閉じると通常運用（上表）が変更前と同じに成立し、加えて空の `to` の戻り link を作る経路が型で閉じる。製品の目的のうち未達のものは無い。

## 起票時実測（2026-09-25、base `85b18a04`）

- helper 現物: `src/lib/return-to.ts:31-44` `returnToLinkProps(value, fallback, options?)` は `{ to: string; search: Record<string, unknown> }` を返し、`value` と `fallback` の両方が不合格なら `{ to: "", search: {} }`（`:43`）。コメント `:40`「空 fallback の to: "" は、呼出側で既定の戻り先を組むための「値なし」の印」。
- 呼出し元（`rg -n 'returnToLinkProps\(' src --glob '!*.test.*'` の出力、定義 1 + 呼出し 7）: `CsvImportRecordDetailPage.tsx:73` / `ManualSaleRecordDetailPage.tsx:48` / `ReceivingRecordDetailPage.tsx:40` / `DisposalRecordDetailPage.tsx:46` / `StocktakeRecordDetailPage.tsx:56` / `ReturnRecordDetailPage.tsx:64`（6 画面とも `returnToLinkProps(returnTo, "/inventory/records")`）、`StockMovementsPage.tsx:76`（`returnToLinkProps(search.returnTo, "", { pathname: "/stock" })`）、定義 `src/lib/return-to.ts:31`。
- 6 画面は `<Link {...backLinkProps}>` を素通しする（`rg -n '<Link \{\.\.\.backLinkProps\}>' src | wc -l` = 12。各画面の error 分岐と header の 2 箇所）。`StockMovementsPage.tsx:95-105` だけが `backLinkProps.to ? … : <Link to="/stock" search={{ q: productCode, selected: productCode }}>` で分岐する。
- `detailReturnTo` 現物: `StockMovementsPage.tsx:77-87`。`URLSearchParams` に `dateFrom` / `dateTo` / `type`（`all` 以外）/ `page`（2 以上）/ `returnTo`（あれば）の順で `set` し、`/stock/${encodeURIComponent(productCode)}/movements` に付ける（`rg -c 'URLSearchParams' src/features/stock-movements/StockMovementsPage.tsx` = 1）。消費者は `MovementTable`（`:261` `returnTo={detailReturnTo}`）で、`MovementTable.tsx:39-45` `sourceLinkProps` が `search: { returnTo }` として元記録 link に載せる。
- 入れ子 `returnTo` の parse 側: 業務記録詳細の「前の画面へ戻る」が `returnToLinkProps` の `defaultParseSearch(url.search)`（`return-to.ts:42`）で在庫変動履歴の search を object に戻し、router が stringify して在庫変動履歴 route の `validateSearch`（`src/features/stock-movements/types.ts:28-42` `stockMovementsSearchSchema`、`returnTo: z.string().max(500).optional().catch(undefined)`）へ渡す。在庫照会への送信側は `StockDetailContent.tsx:37` の `useRouterState(location.href)`。
- router の search 直列化は既定（`rg -n 'stringifySearch|parseSearch' src --glob '!*.test.*'` は 0 hit・exit 1。`src/lib/return-to.ts:2` の `defaultParseSearch` は大文字の `P` のため当たらない。`createRouter` は `src/lib/app-router.ts:76` と `src/test/render-with-router.tsx:21` で custom の直列化を渡さない）。`@tanstack/react-router` 1.168.23（`@tanstack/router-core` 1.168.15）。`defaultStringifySearch` は `@tanstack/react-router` から export されている（`node_modules/@tanstack/react-router/dist/esm/index.d.ts:1`）。
- router-core の実装（`node_modules/@tanstack/router-core/dist/esm/qss.js` / `searchParams.js`）: `defaultStringifySearch` は `new URLSearchParams()` に key を object の列挙順で `set` し `toString()` する。object の値は `JSON.stringify`、string の値は `JSON.parse` できるときだけ `JSON.stringify`（引用符付き）にし、それ以外はそのまま。`undefined` の key は出さない。`defaultParseSearch` は `URLSearchParams` で decode した各値を `JSON.parse` し、失敗したら string のまま。
- DSR-18 の手組み存置の理由の記述（`docs/design-system/01-decision-rules.md:447-449`）: 「自由入力の string 値を router の parseSearch に通さない既存 2 site」。`detailReturnTo` は業務記録詳細の戻りで `defaultParseSearch` を通るため、この理由は在庫変動履歴には当たらない。当たっているのは PR #78 packet の実測（値が日付 / enum / page / 入れ子 `returnTo` だけで、数字だけの string 値を持たない）の方。
- 既存 test の固定値: `StockMovementsPage.test.tsx:120-123` が元記録 link の href を `/inventory/disposal/records/7?returnTo=%2Fstock%2FBT0002%2Fmovements%3FdateFrom%3D2026-06-01%26dateTo%3D2026-06-30%26type%3Ddisposal%26page%3D2` で固定。`:443-466` は入れ子を `expect.stringContaining("%26returnTo%3D")` で見るだけ。在庫照会起点の実 router 往復の test は無い（`ReturnToFlow.test.tsx` は操作ログと入出庫履歴の往復だけ。`rg -n 'movements' src/features/inventory-records/ReturnToFlow.test.tsx` は mock の `movements: []` の 1 行だけ）。
- 6 画面の fallback の既存 test: `OtherRecordDetailPages.test.tsx` の `:198` / `:241` / `:290` の `it.each`（Receiving / Return / ManualSale）、`DisposalRecordDetailPage.test.tsx:101`（error 分岐・`returnTo` なし）と `:121` の `it.each`、`StocktakeRecordDetailPage.test.tsx:195`、`CsvImportRecordDetailPage.test.tsx:251` の `it.each`。いずれも不正 `returnTo` → href `/inventory/records` を固定する。

## Scope

行番号は base `85b18a04` のもの。

- **S1 helper の署名と戻り値**（`src/lib/return-to.ts`）: `returnToLinkProps(value, options?)` にし、`fallback` 引数を外す。戻り値は `{ to: string; search: Record<string, unknown> } | null`。`value` が `parseReturnTo` を通らない、または `options.pathname` と解決後の pathname が一致しないときは `null` を返す。合格時は現行どおり `{ to: url.pathname, search: defaultParseSearch(url.search) }`。URL 解析は 1 回、どの入力でも throw しない（SPEC-RETURNTO-HYGIENE-2026-09-17 C7 を維持）。`:24-30` と `:40` のコメントを新しい契約に合わせて書き直す（fallback の pin 再適用の注意書きは対象が消えるので削る）。`parseReturnTo` と `normalizeReturnTo` は変えない。名前付きの型を新たに export しない。
- **S2 業務記録詳細 6 画面**（下表の 6 file）: `const backLinkProps = returnToLinkProps(returnTo) ?? { to: "/inventory/records", search: {} };` にする（各 file の呼出し 1 行だけ。`<Link {...backLinkProps}>` の 2 箇所は変えない）。
- **S3 在庫変動履歴**（`src/features/stock-movements/StockMovementsPage.tsx`）:
  - `:76` を `returnToLinkProps(search.returnTo, { pathname: "/stock" })` にし、`:95` の分岐を `backLinkProps ? <Link {...backLinkProps}>… : <Link to="/stock" search={{ q: productCode, selected: productCode }}>…` にする（label・アイコン・fallback は現行のまま）。`:74-75` のコメントは「pin 不一致・欠落・不正で helper が `null`」に合わせる。
  - `:77-87` の `returnToParams` / `returnToQuery` を消し、`detailReturnTo` を `` `/stock/${encodeURIComponent(productCode)}/movements${defaultStringifySearch({ dateFrom: normalizedSearch.dateFrom, dateTo: normalizedSearch.dateTo, type: normalizedSearch.type === "all" ? undefined : normalizedSearch.type, page: normalizedSearch.page > 1 ? normalizedSearch.page : undefined, returnTo: search.returnTo })}` `` で組む（key の順は現行の `set` 順と同じにする。`defaultStringifySearch` は空 object で `""`、それ以外で `?` 付きを返すため、`?` の付け外しを自前で書かない）。`defaultStringifySearch` は `:7` の既存の `@tanstack/react-router` import に足す。
- **S4 test**（Matrix の T1〜T7）:
  - `src/lib/return-to.test.ts` の `describe("returnToLinkProps …")`（`:52-120`）を新しい署名で書き直す: T3 有効値の分解（`:53-63` の fallback 引数を外すだけ）/ T1 欠落（`undefined` / `null` / `""`）と不正値（`"relative"` / `"https://evil.example"` / `"/\\evil.example"` / `"/a/..//["` / `"//["`）で `null`、throw しない / T2 pin 不一致で `null`、pin 一致で props / T4 `expectTypeOf(returnToLinkProps("/x")).toEqualTypeOf<{ to: string; search: Record<string, unknown> } | null>()`（`vitest` から import。`npm run typecheck` で効く）。fallback 引数を前提にした 5 本（`:65-70` 欠落時の fallback 分解、`:72-77` 不正時の fallback 分解、`:79-84` query 付き fallback、`:86-91` malformed authority で throw せず fallback へ落ちる、`:100-106` pin を fallback へ再適用しない）と `:93-98` の不正 fallback の sentinel の計 6 本は、検査対象の引数が無くなるため削除し、`:86-91` の入力 `/a/..//[` と `:93-98` の入力群は T1 の不正値 case へ統合する（弱体化ではなく、消えた機能の test の撤去。6 画面の既定の戻り先は既存 T11 系が引き続き固定する）。`normalizeReturnTo` の describe（`:5-50`）は変えない。
  - `src/features/stock-movements/StockMovementsPage.test.tsx` に T5 を足す（`./types` から `stockMovementsSearchSchema` と `normalizeStockMovementsSearch` を値で import する。現行の `:14` は `import type { StockMovementsSearch }` だけ）。既存 `:89-124`（T6、期待 href の文字列）と `SPEC-UI06C-D9-R1` の describe（`:393-507`）は期待値を変えない。
  - `src/features/inventory-records/ReturnToFlow.test.tsx` に T7 を足す（実 `routeTree` + `createMemoryHistory`、既存 T8 の harness を再利用）。`vi.mock("@/lib/bindings")` の commands に在庫照会・在庫変動履歴・廃棄詳細が呼ぶものを足す（base の現物: `searchProducts` / `listLowStock` / `getStockDetail`〈`src/features/stock-inquiry/hooks/useStockInquiry.ts`〉、`listMovements`〈`src/features/stock-movements/hooks/useStockMovements.ts`〉、`getDisposalRecord`〈`DisposalRecordDetailPage.tsx:50`〉。`listDepartments` は既存）。
- **S5 traceability の再生成**: S4 で REQ-207 / REQ-303 付きの test を足す・消すため、`cd src-tauri && cargo run --bin generate_traceability` で `docs/function-design/90-traceability.md` を再生成する（AUTO-GENERATED、手で編集しない）。

設計正本の改訂（DSR-15 / DSR-18 / 更新履歴 / 66 UI-06c-D9・§66.5・§66.7）は本 packet と同じ plan-first commit に入っている。Writer は設計正本・packet・Matrix・`docs/Plans.md`・`docs/backlog.md` を編集しない。実装が設計正本の文面と合わない点を見つけたら、直さずに Coordinator へ返す。

### 呼出し元と import（base `85b18a04`、`rg -n 'returnToLinkProps|backLinkProps|detailReturnTo' src` の全 hit を file ごとに集約）

| file | 現在の import / 呼出し | 変更後 | 足す import | 消える識別子 |
| --- | --- | --- | --- | --- |
| `src/lib/return-to.ts` | `:2` `defaultParseSearch`、定義 `:31-44` | S1（署名 `(value, options?)`、戻り値 `… \| null`） | なし | 引数 `fallback`（`parseReturnTo` は引き続き 1 回呼ぶ） |
| `src/features/inventory-records/CsvImportRecordDetailPage.tsx` | `:39` import、`:73` 呼出し、`:109` / `:128` spread | `:73` だけ S2 | なし | なし |
| `src/features/inventory-records/ReturnRecordDetailPage.tsx` | `:31` import、`:64` 呼出し、`:98` / `:116` spread | `:64` だけ S2 | なし | なし |
| `src/features/inventory-records/ManualSaleRecordDetailPage.tsx` | `:31` import、`:48` 呼出し、`:82` / `:100` spread | `:48` だけ S2 | なし | なし |
| `src/features/inventory-records/ReceivingRecordDetailPage.tsx` | `:31` import、`:40` 呼出し、`:74` / `:92` spread | `:40` だけ S2 | なし | なし |
| `src/features/inventory-records/DisposalRecordDetailPage.tsx` | `:31` import、`:46` 呼出し、`:80` / `:98` spread | `:46` だけ S2 | なし | なし |
| `src/features/inventory-records/StocktakeRecordDetailPage.tsx` | `:30` import、`:56` 呼出し、`:92` / `:110` spread | `:56` だけ S2 | なし | なし |
| `src/features/stock-movements/StockMovementsPage.tsx` | `:7` `Link`（`@tanstack/react-router`）、`:27` import、`:76` 呼出し、`:85-87` `detailReturnTo`、`:95-96` `backLinkProps.to` / `.search`、`:261` `returnTo={detailReturnTo}` | S3 | `:7` に `defaultStringifySearch` | `returnToParams`、`returnToQuery`（`noUnusedLocals` に掛からないよう両方とも宣言ごと消す） |
| `src/features/stock-movements/components/MovementTable.tsx` | prop `returnTo?: string`（`:27`）、`sourceLinkProps`（`:39-45`） | 変えない（受け取る string の形は変わらない） | なし | なし |
| `src/lib/return-to.test.ts` | `:1` `describe, expect, it`（`vitest`）、`:3` 両関数 | S4（T1〜T4） | `:1` に `expectTypeOf` | なし（両関数とも引き続き使う） |
| `src/features/stock-movements/StockMovementsPage.test.tsx` | `:14` `import type { StockMovementsSearch }` | S4（T5） | `./types` から `stockMovementsSearchSchema`、`normalizeStockMovementsSearch` | なし |
| `src/features/inventory-records/ReturnToFlow.test.tsx` | `vi.mock("@/lib/bindings")` の commands 5 個 | S4（T7） | mock に S4 の 5 command | なし |

`backLinkProps` / `detailReturnTo` の名前は上表の file の外に無い（同じ `rg` の出力）。docs の参照は `docs/design-system/01-decision-rules.md`（DSR-15 / DSR-18）と `docs/function-design/66-ui-stock-movements.md`（UI-06c-D9 / §66.3 / §66.5 / §66.7）で、どちらも plan-first commit で同期済み。

## Non-scope

- `src/features/products/**` の returnTo（`buildProductListReturnTo` の手組みは DSR-18 の存置例外として残る）。
- `normalizeReturnTo` の署名と test、`parseReturnTo` の判定。
- route file（`src/routes/**`）と route schema、`MovementTable.tsx`、`StockDetailContent.tsx` ほかの送信側 site。
- `docs/design-system/01-decision-rules.md:127`（DSR-22 の表）の `StockMovementsPage.tsx:73-101` という行番号参照。2026-09-03 に書かれ、base 時点で既に内容とずれている。本 lane で行がさらにずれるが、DSR-22 の列の契約は変わらないため直さない（行番号参照の掃除は別件）。
- `docs/backlog.md` / `docs/Plans.md` の完了記録（closeout で行う）。

## Acceptance Criteria

前提（AC1・AC2・AC6 と Test Plan の vitest / typecheck）: clean な checkout には `src/routeTree.gen.ts`（gitignore 済み）が無く、`.npmrc` の `ignore-scripts=true` で `pretypecheck` / `pretest` の生成も走らないため、依存を用意（`npm ci --ignore-scripts`）してから `npm run generate:routes` を実行し、その後に各 command を回す（未生成だと vitest は import 解決に失敗し、typecheck は exit 2）。AC8 の `local-ci.sh full` は自前で `npm ci` と `npm run generate:routes` を実行する。

- **AC1** `npx vitest run src/lib/return-to.test.ts src/features/stock-movements src/features/inventory-records src/features/stock-inquiry` が pass し、T1〜T3・T5〜T7 を含む。
- **AC2** `npm run typecheck` が exit 0（T4 と 7 画面の呼出しを含む）。
- **AC3** `rg -n 'to: "", search: \{\}' src` が 0 hit（base の出力 = 2 hit: `src/lib/return-to.ts:43`、`src/lib/return-to.test.ts:96`）。
- **AC4** 既存の期待 href を書き換えない: `git diff $(git merge-base origin/main HEAD) -- src/features/stock-movements/StockMovementsPage.test.tsx src/features/inventory-records/OtherRecordDetailPages.test.tsx src/features/inventory-records/DisposalRecordDetailPage.test.tsx src/features/inventory-records/StocktakeRecordDetailPage.test.tsx src/features/inventory-records/CsvImportRecordDetailPage.test.tsx` を `rg '^-[^-]'` に通した出力が、空（T5 の値 import を別の行で足した場合）、または `StockMovementsPage.test.tsx` の `-import type { StockMovementsSearch } from "./types";`（値 import へ書き換えた場合）の 1 行だけになる（既存の期待 href・入力値の行を消さない。追加だけ）。base の出力 = 空・rg exit 1（merge-base `85b18a04`）。
- **AC5** `rg -c 'URLSearchParams' src/features/stock-movements/StockMovementsPage.tsx` が 0 hit（rg は 0 件の file を出力しないため出力なし・exit 1。base の出力 = `1`）。
- **AC6** mutant（Writer が注入 → 該当 test の FAIL を確認 → 復元、Final Reviewer が独立に再注入）: (1) S1 で不合格時に `null` の代わりに `{ to: "", search: {} }` を返す（戻り値の型は `| null` のまま）→ T1 が FAIL (2) S1 の戻り値の型から `| null` を外し不合格時に sentinel を返す → `npm run typecheck` が T4 で FAIL (3) S1 の `options.pathname` 判定を外す → T2 の pin 不一致 case が FAIL (4) S3 の `detailReturnTo` を base の `URLSearchParams` の手組みへ戻す → T5 の `returnTo: "123"` case が FAIL (5) S3 で `detailReturnTo` の object から `returnTo` を落とす → T7 と既存 `REQ-207 / UI-06c-D9: 元記録 link の returnTo に在庫変動履歴の returnTo を入れ子で含める` が FAIL (6) S2 の 1 画面（`DisposalRecordDetailPage.tsx`）で既定の戻り先を `/` にする → 既存 `REQ-207 / T11 DSR-18: DisposalRecordDetailPage の returnTo %s を安全に %s へ正規化する` の不正 case が FAIL。 (7) S3 の `page > 1` を `page >= 1` にする → T5 (c)（`detailReturnTo` = `/stock/BT0002/movements` の文字列一致）が FAIL。
- **AC7** `cd src-tauri && cargo run --bin generate_traceability -- --check` が exit 0（S5 の再生成後）。
- **AC8** `bash scripts/local-ci.sh full` が pass。
- **AC9** `bash scripts/doc-consistency-check.sh` が ERROR 0。

## Design Sources

- Requirements / spec: REQ-207（在庫変動履歴から元業務記録へ相互参照）/ REQ-303（商品ごとの在庫変動履歴）/ REQ-301（在庫照会）
- Architecture: `docs/ARCHITECTURE.md`（UI 層だけ。CMD / BIZ / IO は変えない）
- Function / command / DTO: `docs/function-design/66-ui-stock-movements.md` UI-06c-D9 / §66.3 / §66.5 / §66.7、`docs/function-design/65-inventory-record-traceability.md` TRACE-D11（変えない、既定 hub `/inventory/records` の根拠）
- DB: 該当なし
- Screen / UI: `docs/design-system/01-decision-rules.md` DSR-15 / DSR-17 (a)・(b) / DSR-18
- Decision log / ADR: 新しい D-n は立てない（DSR と UI-06c-D9 の改訂で足りる。PR #78 と同じ扱い）

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | 該当なし | existing sufficient |
| Command / DTO / generated binding / wire shape | 該当なし（`bindings.ts` 不変） | existing sufficient |
| DB / transaction / audit / rollback / migration | 該当なし | existing sufficient |
| Screen / UI / route state / Japanese wording | DSR-15 / DSR-18 / 66 UI-06c-D9 | updated in this PR（plan-first commit） |
| CSV / TSV / report / import / export format | 該当なし | existing sufficient |
| Durable decision / ADR | `01-decision-rules.md` 更新履歴 1 行 | updated in this PR（plan-first commit） |

## Registration / Generation Obligations

| 変更対象 | 登録・生成義務 |
|---|---|
| REQ / coverage の追加・変更・削除（REQ-207 / REQ-303 付き test の追加・削除） | `cd src-tauri && cargo run --bin generate_traceability` で `docs/function-design/90-traceability.md` 再生成（S5、AC7） |

他の行（Tauri command / function-design doc 新設 / source doc 新設・改名 / consultation relay / route 新設 / operator 画面新設）は該当なし。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| REQ-207 | 01-decision-rules DSR-15 / DSR-18 | DSR-18（改訂、2026-09-25） | 空の `to` の sentinel は型検査を通り、押しても遷移しない link になり得る（backlog `:28`）。`null` にすると spread も property 参照も型で止まる（Probe 1）。フォールバック引数を外す理由: `\| null` を返しつつ fallback 引数を残すと、呼出側は同じ既定 hub を `??` で二重に書くことになる。棄却: (a) 署名を保って両方不合格のときだけ `null`（上の二重記述）/ (b) sentinel のまま comment で注意（backlog の起点そのもの）/ (c) 呼出側で `!` を付ける（型で強制する意味が消える）/ (d) 既定 hub を Link props の定数として helper から export（6 画面の既定 hub は同じ `/inventory/records` だが、DSR-18 は遷移先ごとの既定 hub を呼出側の責務とするため、1 行の literal に留める） | S1 / S2 / S3 | T1 / T2 / T3 / T4、既存 T11 系 |
| REQ-303 | 66 UI-06c-D9 | UI-06c-D9（改訂、2026-09-25） | `detailReturnTo` は router の直列化へ寄せる。ただし `location.href` ではなく、props で受けた検証済み search を正規化した object を `defaultStringifySearch` で直列化する。`/` 始まりの `returnTo` と `returnTo` なしでは出力が手組みと 1 byte も変わらず、`/` 始まりでない不正値では詳細 link の href だけが変わって在庫照会へ戻る先は既定のまま（Probe 2、Boundary / Wire Contract の compatibility）。JSON として読める string だけに引用符が付く（将来 string の param を足したときに GA5 と同じ機序で型が落ちる罠を閉じる）。棄却: (a) 手組みを存置し DSR-18 の理由を書き直す（現状は無害だが罠が残り、DSR-18 の存置理由の記述〈parseSearch を通らない〉が現物と合わない）/ (b) `useRouterState` の `location.href`（URL の未検証の値・既定値の `page=1`・元の key 順をそのまま運び、入れ子 `returnTo` の形が変わる。page component が props ではなく router の現在地に依存し、既存 test の harness〈`renderWithRouter` の初期 path `/`〉と合わなくなる） | S3 | T5 / T6 / T7 |
| REQ-303 / REQ-207 | 66 UI-06c-D9、DSR-17 (b) | UI-06c-D9 | 在庫照会起点の 3 段往復で href が文字列一致すること（scroll 復元の key が `location.href`）を実 router で固定する。PR #78 では L3 だけが持っていた | S3 | T7 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: yes（DSR-15 / DSR-18 に helper の戻り値と `null` の理由、66 UI-06c-D9 に `detailReturnTo` の組み方と棄却案 2 つを書いた）
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: 「値なしは空の `to` で表さず `null`」「`detailReturnTo` は正規化した object を router の直列化で組み、`location.href` を使わない」を DSR-18 と UI-06c-D9 へ
- Assumptions and constraints: router の search 直列化は既定のまま（custom の `parseSearch` / `stringifySearch` を入れる change は、`returnToLinkProps` の `defaultParseSearch` と S3 の `defaultStringifySearch` を router の設定に合わせて同時に見直す）。TanStack Router の typed `Link` が `to` を必須とし、`\| null` の spread を TS2322 で拒む（Probe 1、1.168.23 / tsc 5.8.3）
- Deferred design gaps, risk, and follow-up target: DSR-22 の行番号参照（Non-scope）。既定 hub の literal の typo は `??` の右辺では型検査されない（Probe 1 の `typo` case。現行の fallback 文字列も同じで後退ではない。既存 T11 系の不正 case が 6 画面とも href `/inventory/records` を固定するため test で検出される）
- Test Design Matrix can cite design decision IDs or source doc sections: yes
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: 「空の `to` の戻り link を描画しない」は型（T4、AC6 (2)）と runtime（T1、AC6 (1)）の両方で検査。escape hatch = 呼出側の既定 hub（6 画面 `/inventory/records`、在庫変動履歴 `/stock?q&selected`）で、どちらも変えない。互換: 既存の期待 href を 1 本も書き換えない（AC4）

## Impact Review Lenses

Fact check / design decision split lens だけ該当: 外部 library（TanStack Router 1.168.23 / router-core 1.168.15 の search 直列化と `Link` の型）の挙動を Contract Probe 1〜2 と `node_modules` の実装で事実確認し、設計判断（S1〜S3）と分けて記録した。他の lens は not applicable（field 調査・実機・POS / CSV 起点ではなく、操作・data・配布物を変えない）。

## Design Readiness

- Existing design docs are sufficient because: 振舞いの本則（遷移元へ戻る、不正は遷移先ごとの既定 hub）は DSR-18 / TRACE-D11 / UI-06c-D9 で確定済みで変えない。本 lane は helper の型と直列化の組み方の是正
- Source docs updated in this PR: DSR-15（判定フロー）/ DSR-18（判定フローの送信側と不合格時、共通 helper の段落）/ `01-decision-rules.md` 更新履歴 / 66 UI-06c-D9・§66.5・§66.7（いずれも plan-first commit）
- Design gaps intentionally deferred: DSR-22 の古い行番号参照（Non-scope）
- Durable decisions discovered in this plan and promoted to source docs: 上記 Design Intent Audit の 2 点
- owner 決定待ち: なし（画面の振舞い・受容リスク・範囲に関わる選択が無い。backlog `:30` の寄せる / 寄せないは、正規の戻り先では出力が変わらず、不正値でも在庫照会へ戻る先が既定のままであることを Probe 2 で確かめられる技術判断として drafter が決めた）

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): UI 層だけ
- Backend function design: 該当なし
- Command / DTO / data contract: 不変
- Persistence / transaction / audit impact: なし
- Operator workflow / Japanese UI wording: 不変（label・行き先・href とも同じ）
- Error, empty, retry, and recovery behavior: 不正 `returnTo` は既定 hub へ（不変）。業務記録詳細の error 分岐の戻り link も同じ `backLinkProps` を使う（不変）
- Testability and traceability IDs: REQ-207 / REQ-303、SPEC-NAV-RETURN-TYPES-2026-09-25

## Contract Probe

- TanStack の typed `Link` は `{ to: string; search: … } | null` の spread を拒むか: base の `src` を scratch へ複製し、main checkout の `node_modules`（`@tanstack/react-router` 1.168.23、tsc 5.8.3）と `tsr generate` の routeTree で `tsc --noEmit -p tsconfig.json` -> `<Link {...nullable}>` は TS2322、`nullable.to` の参照は TS18047、`<Link>`（`to` なし）は TS2741、`nullable ?? { to: "/inventory/records", search: {} }` の spread と `nullable ? <Link {...nullable}> : <Link to="/stock" …>` は error なし（probe 以外の error 0）。`?? { to: "/inventory/recods", … }`（typo）の spread は error にならず、`<Link to="/inventory/recods">` の直書きだけが TS2820 になる。`expectTypeOf(非 null の値).toEqualTypeOf<… | null>()` は TS2344（T4 が型の後退を検出できる）
- `defaultStringifySearch` の出力は現行の手組みと同じか: node で base の手組み（`StockMovementsPage.tsx:77-87` と同じ `set` 順）と S3 の object 直列化を 6 入力で比較 -> `{}` / 日付 + `type` + `page` / `type` + `returnTo` / 数字だけの商品コードを引用符付きで持つ入れ子 `returnTo`（`/stock?q=%222099000000019%22&selected=%222099000000019%22`）/ 日本語と `+` を含む入れ子 `returnTo` の 5 入力は文字列一致（例 `?dateFrom=2026-06-01&type=disposal&page=3&returnTo=%2Fstock%3Fq%3D%25222099000000019%2522%26selected%3D%25222099000000019%2522`）。`returnTo: "123"`（JSON として読める string）だけ `?returnTo=123` と `?returnTo=%22123%22` で異なり、後者を `defaultParseSearch` すると string `"123"` に戻る（前者は number 123 になり、route schema で `undefined` に落ちる）。拡張比較（商品コード 4 × search 7 × `returnTo` 20 = 560 通り）: 差が出たのは JSON として読める `returnTo` の 9 値（`123` / `true` / `null` / `"x"` / `[1]` / `{}` / ` 1` / `-0` / `1e3`）だけで、どれも `/` 始まりでない。`/` 始まりの `returnTo` と `returnTo` なしは全件 1 byte 同じ（`/` 始まりの string は JSON として読めないため、新旧とも引用符なしでそのまま出る）。差の出る値は新旧とも不正な戻り先で、業務記録詳細との往復の後も「在庫照会へ戻る」は既定の戻り先に着地する（router-core 1.168.15 の `defaultStringifySearch` / `defaultParseSearch` の実物で `123` / `true` / `null` / `[]` / `"x"` / `[1]` / `{}` / ` 1` / `-0` / `1e3` を往復）。例外は JSON の文字列 literal として読める値（引用符ごとの `"/stock"`。二重に符号化した URL でだけ作れる）で、旧は往復で引用符を外して `/stock` へ着地させ、新は往復の前後とも既定の戻り先に着地する

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| DSR-15 / DSR-18 `returnToLinkProps` は合格値を `to`(pathname) / `search`(object) へ分解する | S1 | T3 | — |
| DSR-15 / DSR-18（2026-09-25）不合格（欠落・不正・解析不能）は `null`、throw しない | S1 | T1 | — |
| DSR-18 `options.pathname` 不一致は不正値（`null`） | S1 | T2 | — |
| DSR-18（2026-09-25）戻り値の型が `\| null` で、呼出側の分岐を型で強制する | S1 / S2 / S3 | T4、`npm run typecheck`（AC2） | — |
| DSR-18 / TRACE-D11 業務記録詳細 6 画面は正当な `returnTo` の遷移元へ戻る | S2 | 既存 T11 系（6 画面の有効 case）、`ReturnToFlow.test.tsx` T8 / T10 | — |
| DSR-18 / TRACE-D11 欠落・不正は既定 hub `/inventory/records` | S2 | 既存 T11 系の不正 case、`DisposalRecordDetailPage.test.tsx` の error 分岐（`returnTo` なし）、`ReturnToFlow.test.tsx` T9 | — |
| DSR-18 戻り link は文字列 `to` に query を埋め込まない | S2 / S3（`<Link {...backLinkProps}>` のまま） | 既存 T11 系（query 付き `returnTo` の href 一致） | — |
| DSR-18 送信側は router の直列化（手組みの存置は products だけ） | S3 | T5 | products は non-scope |
| DSR-17 (a) 戻りは `<Link>` push のまま | 変えない | 既存 `app-router.test.tsx`、`ReturnToFlow.test.tsx` T10 | — |
| DSR-17 (b) 戻り先の href が遷移元の href を再現する | S3 | T7（3 段の文字列一致）、既存 T8 | — |
| UI-06c-D9 「在庫照会へ戻る」は `/stock` に pin、欠落・不正・pin 不一致は `q` / `selected` fallback | S3 | 既存 `SPEC-UI06C-D9-R1` 7 case、GA5 T2-num | — |
| UI-06c-D9 `detailReturnTo` は正規化した search（既定値を載せない）と入れ子 `returnTo` を運ぶ | S3 | T6（href の文字列一致、既存）、T5、既存「元記録 link の returnTo に在庫変動履歴の returnTo を入れ子で含める」 | — |
| UI-06c-D9 filter 変更・page 送り・reset でも `returnTo` を保持 | 変えない（`updateSearch` / `resetFilters`） | 既存「%s 後も returnTo が残る」 | — |
| UI-06c-D9 `selected` は 100 文字まで | 変えない | 既存 `src/features/stock-inquiry/types.test.ts` | — |
| SPEC-RETURNTO-HYGIENE-2026-09-17 C7 どの入力でも throw しない | S1 | T1（不正・解析不能の入力で `null`） | — |
| DSR-15 `normalizeReturnTo(value, fallback)` は primitive として存置 | 変えない | 既存 `return-to.test.ts:5-50` | — |

Adjacent-contract sweep: DSR-15 / DSR-18 / 66 §66.2 UI-06c-D9・§66.3・§66.5・§66.7 の本文を `rg -n 'returnTo|returnToLinkProps|detailReturnTo'` で当て、上表に無い契約（UI-06c-D1〜D8 の movement 表示・query 変換）は本 Scope が触らないため除外した。

## Test Plan

[Test Design Matrix](test-matrices/2026-09-25-nav-return-to-types.md)。

- targeted tests: T1〜T7
- 前提: vitest / typecheck の前に依存の用意と `npm run generate:routes`（Acceptance Criteria 冒頭の前提）
- negative tests: T1（欠落・不正・解析不能）/ T2（pin 不一致）/ T5 の `/` 始まりでない不正値の case（(b) `returnTo: "123"`、(d) 在庫照会へ戻る先が新旧とも既定）
- compatibility checks: AC4（既存の期待 href を書き換えない）、T6、既存 T11 系・`SPEC-UI06C-D9-R1`・GA5 T2-num・`ReturnToFlow.test.tsx` T8〜T10
- data safety checks: 該当なし（synthetic 値だけ）
- main wiring/integration checks: T7（実 `routeTree` + memory history で在庫照会 → 在庫変動履歴 → 業務記録詳細 → 戻る 2 回）
- commit 構成（1 通り）: commit 1 = S1〜S4（実装と test を同じ commit）、commit 2 = S5（`90-traceability.md` の再生成だけ）。packet / Matrix / 設計正本 / `docs/Plans.md` / `docs/backlog.md` は Writer が編集しない
- 許容する書込み: `bash scripts/local-ci.sh full`（AC8）が `.local/ci-evidence/` へ書く証跡、`npm run generate:routes` が作る `src/routeTree.gen.ts`（gitignore 済み）、`cargo` の `src-tauri/target/`。いずれも commit しない
- Human Gate に manual は無いため、`cargo check --release` は Writer の完了条件に含めない

## Boundary / Wire Contract

- producer: 在庫照会 `StockDetailContent.tsx:37`（`location.href` → `returnTo`）/ 在庫変動履歴 `detailReturnTo`（S3、正規化 object → `defaultStringifySearch`）/ 業務記録詳細・在庫変動履歴の戻り link（`returnToLinkProps` → `<Link to search>` → router の stringify）
- consumer: 在庫変動履歴 route の `stockMovementsSearchSchema`、業務記録詳細 route の `returnTo: z.string().max(500)`、在庫照会 route の `stockInquirySearchSchema`
- wire type: URL search param `returnTo`（string、入れ子で 1 段 `returnTo` を含む）と在庫変動履歴の `dateFrom` / `dateTo` / `type` / `page`
- internal type: `returnToLinkProps` の戻り値 `{ to: string; search: Record<string, unknown> } | null`
- precision/range: `returnTo` は `max(500)`（不変）。`page` は正の整数（不変）
- round-trip path: 在庫照会 href → `returnTo` → 在庫変動履歴 → `detailReturnTo`（入れ子）→ 業務記録詳細 → `returnToLinkProps`（`defaultParseSearch`）→ `<Link>`（stringify）→ 在庫変動履歴 route の `validateSearch` → `returnToLinkProps`（pin `/stock`）→ 在庫照会 route の `validateSearch`
- invalid input: 欠落 / 空 / `/` 始まりでない / 外部 origin に解決される / 解析不能 / pin 不一致 → `null` → 呼出側の既定 hub
- compatibility: `/` 始まりの `returnTo`（正規の戻り先）と、`returnTo` が無い場合は、`detailReturnTo` の出力が変更前の手組みと 1 byte も変わらない。route schema が受ける `/` 始まりでない値（例 `"123"`、`"true"`、`"null"`、`"[]"`）は新旧とも不正な戻り先で、在庫照会へ戻る link は同じ既定の戻り先（`/stock?q=<code>&selected=<code>`）に着地する。この場合だけ詳細 link の href（と、href を key にする scroll 復元の key）が変わる。旧の手組みはこの値を number 等（`null`・配列を含む）に化けさせており、新は string のまま運ぶ。例外: JSON の文字列 literal として読める値（引用符ごとの `"/stock"` 等。二重に符号化した URL でだけ作れる）は、旧が業務記録詳細との往復で引用符を外して中身の `/stock…` へ着地させていた。新は往復の前後とも既定の戻り先に着地する（Probe 2）。bookmark 相当の既存 URL の受け側の扱いは変えない

## Review Focus

- `returnToLinkProps` の `null` の分岐が 7 画面すべてで、変更前と同じ既定の戻り先へ落ちるか（6 画面の `/inventory/records`、在庫変動履歴の `/stock?q&selected`）
- 撤去する `return-to.test.ts` の 6 本（fallback 引数の test 5 本と不正 fallback の sentinel）が、消えた機能だけを検査していたか。残る振舞い（既定 hub、query 付き値の型保存）が T1〜T3 と既存 T11 系で固定されているか
- `detailReturnTo` の key 順・既定値の省略が手組みと同じで、既存 test の期待 href を書き換えずに通るか（AC4、T6）
- T5 の `/` 始まりでない不正値の case（(b)・(d)）が、在庫照会へ戻る先の変わらない入力で直列化の可逆性を検査していることを test 名か comment で明示しているか（正規の戻り先〈`/` 始まり〉では差が出ない）
- T7 が実 router の往復を通り、mock の境界が generated `commands.*` にあるか
- 保守者が `StockMovementsPage.tsx` の `detailReturnTo` を読んで、手組みでも `location.href` でもない理由を 66 UI-06c-D9 から辿れるか

## Spec Contract

Contract ID: SPEC-NAV-RETURN-TYPES-2026-09-25

- C1: `returnToLinkProps(value, options?)` は DSR-15 の guard を通り、`options.pathname` があればそれと一致する値だけを `{ to: pathname, search: defaultParseSearch(search) }` で返し、それ以外は `null` を返す。どの入力でも throw しない
- C2: `returnToLinkProps` の戻り値の型は `{ to: string; search: Record<string, unknown> } | null` で、呼出側は `null` の分岐を書かないと typecheck を通らない
- C3: 業務記録詳細 6 画面は `null` のとき `/inventory/records`、在庫変動履歴は `null` のとき `/stock?q=<code>&selected=<code>` へ戻る（変更前と同じ）
- C4: `detailReturnTo` は `/stock/<code>/movements` に、正規化した search（`dateFrom` / `dateTo` / `all` 以外の `type` / 2 以上の `page`）と `returnTo` を router の既定の直列化で付けたもので、受け側の schema で parse すると元の search に戻る
- C5: 在庫照会 → 在庫変動履歴 → 業務記録詳細 → 「前の画面へ戻る」→「在庫照会へ戻る」で、各段の href が行きの href と文字列一致する

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| C1 | S1 | T1 / T2 / T3 | 撤去 test の妥当性 | AC1 / AC6 (1)・(3) |
| C2 | S1 / S2 / S3 | T4 | 7 画面の分岐 | AC2 / AC3 / AC6 (2) |
| C3 | S2 / S3 | 既存 T11 系、`SPEC-UI06C-D9-R1` | 既定の戻り先が不変 | AC1 / AC4 / AC6 (6) |
| C4 | S3 | T5 / T6 | key 順・既定値の省略 | AC1 / AC4 / AC5 / AC6 (4) |
| C5 | S3 | T7 | 実 router の往復 | AC1 / AC6 (5) |

## Data Safety

- 実店舗の商品コード・JAN・記録を test に使わない（`2099000000019` と `BT0002` は既存 test と同じ合成値）
- local-only: `.local/ci-evidence/`（AC8 の証跡）、scratch の probe 複製（commit しない）
- synthetic-only: test の fixture すべて

## Implementation Results

Fill after implementation.

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

## Review Response

- Review-only skipped because: R3 の review-only sub-agent は、Final Review の Claude 側（Fable 5.1）と Codex 側の 2 本の broad が兼ねる。
- Findings Freeze: frozen after Broad Audit（2 本とも finding なし）; post-freeze exceptions: none.

### Plan Review round 1（2026-09-25、対象 `53f3f353`）

- Claude 側（fresh Opus 5.5、read-only）: approve、P1 0 / P2 0 / P3 3。S1〜S3 を隔離 worktree で仮実装し、6 行の操作列が変更前と同じ href・検索条件・選択行へ戻ることを確認。P3 = (1) `return-to.test.ts:86-91` が撤去一覧に無い、(2) Matrix T5 の `defaultParseSearch` の戻り値は `AnySchema` で `.returnTo` が型エラー、(3) AC4 の条件と比較の基点。
- Codex 側（GPT-6 Sol、high）: reject、P1 0 / P2 2 / P3 0。(1) `/` 始まりでない不正な `returnTo`（例 `"123"`）では新旧で詳細 link の href が変わり、packet の「全ての値で不変」が書き過ぎ、(2) clean な checkout では `routeTree.gen.ts` が無く AC1・AC2 がそのままでは落ちる。
- 裁定（Coordinator）: 全件 accept。Codex (1) は直列化の変更を撤回せず、契約を「`/` 始まりの値と欠落時は 1 byte も変わらない。不正値は新旧とも既定の戻り先に着地し、href だけが変わる（旧は number に化けさせていた）」に正確化し T5 (d) を追加。Codex (2) は前提に `npm ci --ignore-scripts` と `npm run generate:routes`。Claude 側 P3 3 件は指摘どおり。是正 = `9adb9769`。

### Plan Review round 2（2026-09-25、対象 `9adb9769`）

- Codex 側（GPT-6 Sol、high）: approve、P1 0 / P2 0 / P3 0。型 probe と 560 通りの直列化比較を再現。AC8 の `local-ci.sh full` で本 lane と無関係の `OperationLogsPage.scroll-restoration.test.tsx:83` が 5 秒 timeout（変更前の base でも発生、並走 run の負荷の可能性）→ 実装後の gate で切り分ける。
- Claude 側: round 1 の P3 だけのため再投入しない（DEV_WORKFLOW Subagent Budget）。

### Final Review broad（2026-09-26、対象 `f5f69aed`、互いに独立）

- Codex 側（GPT-6 Sol、high）: approve、P1 0 / P2 0 / P3 0。AC1〜AC9 すべて PASS（local-ci full を含む）、AC6 の mutant 7 本すべて red、正規の戻り先で旧実装との 560 通りの比較の byte 差 0。[review](https://github.com/higekiri-w/inventory-system-desktop/pull/106#pullrequestreview-5320379437)
- Claude 側（Fable 5.1、fresh、read-only）: approve、P1 0 / P2 0 / P3 0。Ordinary Operation 6 行を実装後の現物で確認（行 6 は既定の戻り先を外すと typecheck が TS2322）。AC1〜AC7・AC9 を再実行、mutant 7/7 red、`detailReturnTo` の同一性を 196 通り + `/` 始まりのランダム 20,000 件で確認（差 0）。
- helper の record: review broad 2 本（pass）。owner の Ready・merge 指示（2026-09-26）で helper `ready` → hosted CI 成功 → helper `merge`。
