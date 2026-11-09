# Plan Packet: ㉙ returnTo 衛生（guard の origin 判定 / 戻り link の search object 化 / 数字だけの検索語の往復、R3）

2026-09-17 起草。出典は Backlog「やると決めたもの（順番未定）」の [returnTo 衛生](../../backlog.md) 行（NAV-1〈PR #75〉Final Review round 1 pass B / round 2 pass B の P3 5 件）。owner 2026-09-17「returnTo 衛生やろうか」で着手。起票時の現物調査で同じ failure class の実害 1 件（S5、入出庫履歴の数字だけの検索語が戻りで消える）を見つけ、同 lane に含める。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

- Evidence Mode: github
- Phase: archive
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: 5ceb30addb87dbb40e24a9332426f08946cb2b54
- Amendments: de9182724484a01f327a61ccf0ac8ad1e9b3e057, ba40c2b5203ecf31db98bde122ac22c72fd4edd8, bd42f7d9d16313b0b2f1b8aaa07d20f2b7ef70aa
- Coordinator: Fable 5.1
- Writer: Sonnet subagent（初回実装 `1839de7d` / `bd56e4da` / `b4364544`）+ Codex（GA2: Final Review round 1 の是正 run、発注書 63。round 2 の closure は Opus + Sonnet の独立 fresh context が担い、Codex は自分の是正を採点しない）
- Plan Reviewer: Sonnet（独立 fresh context）
- Final Reviewer: round 1 = Codex + Opus（独立 fresh context。GA1）/ round 2 = Opus + Sonnet（GA2: 是正の Writer が Codex のため、Codex を reviewer から外す）
- Final Review Minimum: 2
- Human Gate: ready,merge,manual

manual = owner Windows native L3 1 往復（AC-L3-1）。route/search state の変更で、NAV-1 では自動 test が全 pass のまま L3 で欠陥（数字だけの商品コード）が出た実績があるため省略しない。

遷移記録（append-only）:

- kickoff → spec-check → design → plan-draft → plan-gate（本 commit、plan-first）: 設計正本の改訂点（DSR-15 / DSR-18 / 66 UI-06c-D9 / 60 系の入出庫履歴 returnTo）は Scope S6 に列挙し、実装と同じ PR で同期する。owner の設計判断を要する論点なし（挙動は「戻り先が同じ画面・同じ条件のまま」で不変、不正値の拒否範囲が広がるだけ）
- plan-gate → plan-approved → implementing（本 commit、state-only）: Plan Review round 1（Sonnet、P1/P2 = 0、P3 3）→ in-place 是正 `5ceb30ad`。P3 のみのため reviewer 再投入なし。Plan Commit = `5ceb30ad`（plan-first `b5cc6b5e` → 是正を含む確定版）。実装は Sonnet subagent の worktree run で本 commit を起点にする
- implementing → archive（closeout、本 commit）: PR #78 squash merge `a68ba291`（2026-09-17）。packet / Matrix を archive へ移送し、Implementation Results / Review Response を記録

## Owner Effort Budget

- 介入回数上限: 3
- 実働時間上限: 10分
- relay 往復上限: 3（GA1: Final Review の Codex pass 1 往復 / GA2: 是正 run 1 往復 / GA3: 是正 run の再実行 1 往復〈Coordinator 起因の空振り、既定 2 を超える理由は GA3〉。起票時は 0）
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
route/search state（`returnTo` の検証と、戻り link が router へ渡す `to` / `search` の形）を変える。DB / command DTO / bindings は不変。

## Goal

Goal Invariant:

### 最小完了条件

- 「前の画面へ戻る」「在庫照会へ戻る」は、これまで戻れていた遷移元（pathname + 検索条件）へ同じように戻る
- 入出庫履歴で数字だけの語（JAN・商品コード）を検索 → 詳細 → 「前の画面へ戻る」で、検索語が残った一覧へ戻る（現状は消える）
- `returnTo` に `/\host` や tab 入りの値が来ても app 外の origin を指す link を描画せず、既定 hub へ fallback する

### 失敗定義

- 既存の戻り導線（業務記録詳細 6 画面 / 在庫変動履歴 / 商品 form）のどれかで、戻り先の pathname か検索条件が変わる・失われる
- 「在庫照会へ戻る」が `/stock` 以外へ着地する
- guard を強めた結果、正当な `returnTo`（入れ子の `returnTo` を含む 3 段往復）が fallback へ落ちる

### 非目的

- 商品一覧の typed parse-back（`src/features/products/lib/return-to.ts`）の変更。DSR-18 が exact-allowlist として存置を決めている
- `returnTo` の送信側 site の追加・削除、label 文言の変更、`history.back()` 化（DSR-17 (a) で不採用）
- `returnTo` の長さ上限（`max(500)`）と入れ子の深さの再設計

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 起票時実測（2026-09-17、`d49913e7`）

- guard 現物: `src/lib/return-to.ts` の `normalizeReturnTo` は `value?.startsWith("/") && !value.startsWith("//")` のみ
- Contract Probe 1（node、`new URL(v, "http://inventory.local")`）: `"/\\evil.example"` → 現 guard 通過・origin `http://evil.example` / `"/\t/evil.example"` → 現 guard 通過・origin `http://evil.example` / `"/%09/evil.example"`（percent 表記のまま）→ origin は local で pathname `/%09/evil.example`（無害。backlog の `/%09/` は decode 後の tab を指す）/ `"//evil.example"`・`"https://evil.example"` → 現 guard で拒否済み / `"/a/../b?x=1#h"` → local、pathname `/b`、search `?x=1`、hash `#h`
- 文字列 `to` の site: `rg -n '<Link to=\{(backHref|safeReturnTo)\}>' src/features` = 13 hit（業務記録詳細 6 画面 × 2 + `StockMovementsPage.tsx:94`）。詳細 6 画面の label は「前の画面へ戻る」、在庫変動履歴だけ「在庫照会へ戻る」
- canonical（`to` = pathname、`search` = object）: `InventoryRecordsPage.tsx:69-78` `buildDetailLinkProps` / `MovementTable.tsx:38-45` `sourceLinkProps`
- router の search 直列化は既定（`rg -n 'parseSearch|stringifySearch' src --glob '!routeTree.gen.ts'` = 0 hit）。`@tanstack/react-router` 1.168.23
- Contract Probe 2（node）: `defaultStringifySearch({q:"123",selected:"BT0002",page:2,returnTo:"/stock?q=%22123%22"})` = `?q=%22123%22&selected=BT0002&page=2&returnTo=%2Fstock%3Fq%3D%2522123%2522`、これを `defaultParseSearch` すると `{"q":"123","selected":"BT0002","page":2,"returnTo":"/stock?q=%22123%22"}` に戻る（router が作った href は往復する）
- Contract Probe 3（node、S5 の実害）: `defaultParseSearch("?q=2099000000019&page=2")` = `{"q":2099000000019,"page":2}`、`typeof q` = `number`、`z.string().max(100).optional().catch(undefined).parse(q)` = `undefined`。`InventoryRecordsPage.tsx:54-67` `buildInventoryRecordsReturnTo` は `URLSearchParams` で手組みするため `q=2099000000019`（引用符なし）を作る。route schema は `src/features/inventory-records/types.ts:43` `q: z.string().max(100)`。NAV-1 GA5 と同じ機序
- 他の手組み site: `StockMovementsPage.tsx:75-85` `detailReturnTo`（値は日付 / enum / page / 入れ子 `returnTo` のみで、数字だけの string 値を持たない）/ `src/features/products/lib/return-to.ts` `buildProductListReturnTo`（受け側が `parseProductListSearchFromReturnTo` の typed parse-back で router の `parseSearch` を通らない）
- `selected` 上限: `src/features/stock-inquiry/types.ts:37` `selected: z.string().min(1).max(20)`、同 `:31` `q` は `max(100)`。商品コードの長さ上限は DB（`schema_v1.rs:30` `product_code TEXT PRIMARY KEY`）にも BIZ（`product_service.rs:1067` は空判定のみ）にも無い。20 文字超の商品コードでは `selected` だけ落ちて fallback の選択が外れる
- T2-num 現物: `StockMovementsPage.test.tsx:419-433`。href の `q` / `selected` を `JSON.parse` して string を確認するだけで、`stockInquirySearchSchema` までは通していない

## Scope

- **S1 guard を origin 判定へ**（`src/lib/return-to.ts`）: `normalizeReturnTo(value, fallback)` は `new URL(value, "http://inventory.local")` が throw せず、`value` が `/` 始まりで、origin が base と一致し、**解決後の pathname が `//` 始まりでない**（GA2）ときだけ通す。返す値は `pathname + search`（hash は落とす。app は hash を使わない）。先例 = `src/features/products/lib/return-to.ts:12-21`。署名と既存 8 case の期待値は不変
- **S2 戻り link の props helper**（同 file）: `returnToLinkProps(value, fallback, options?)` を追加し `{ to: string; search: Record<string, unknown> }` を返す。`to` = pathname、`search` = `defaultParseSearch(url.search)`。fallback も同じ経路で分解する。**URL の解析は値 1 つにつき 1 回とし、検証済みの `pathname + search` を文字列に戻して再解析しない。helper はどの入力でも throw しない**（GA2）。`options.pathname` を渡した場合、解決した pathname がそれと一致しなければ不正値として扱う（S3 の pin）
- **S3 consumer 7 file の置換**: 業務記録詳細 6 画面（`Receiving` / `Return` / `ManualSale` / `Disposal` / `CsvImport` / `Stocktake` `RecordDetailPage.tsx`）の `<Link to={backHref}>` 12 箇所を `<Link {...returnToLinkProps(returnTo, "/inventory/records")}>` 相当へ。`StockMovementsPage.tsx:74,93-103` は `options.pathname = "/stock"` で pin し、不一致・欠落・不正は現行の `<Link to="/stock" search={{ q, selected }}>` fallback へ
- **S4 `selected` 上限を `q` と揃える**（`src/features/stock-inquiry/types.ts:37`）: `max(20)` → `max(100)`
- **S5 入出庫履歴の returnTo を router の href から取る**（`InventoryRecordsPage.tsx`）: `buildInventoryRecordsReturnTo` の手組みをやめ、ほかの送信側と同じ `useRouterState({ select: (s) => s.location.href })` にする（closeout 実測: `rg -n 'useRouterState\(\{ select: \(state\) => state\.location\.href \}\)' src/features --glob '!*.test.*'` から本 file を除くと 6 file。DSR-18「現在の pathname + search state を returnTo に直列化」の本則どおり。手組み関数は削除）
- **S6 設計正本の同期**（同 PR、docs commit は分けてよい）: `docs/design-system/01-decision-rules.md` DSR-15（判定を origin 一致へ、`/\` と制御文字の例）/ DSR-18（helper の最低基準、`to` は pathname・`search` は object、送信側は router の href）/ 改訂履歴 1 行。`docs/function-design/66-*.md` UI-06c-D9（pathname pin、`<Link to={safeReturnTo}>` の記述を置換）。入出庫履歴の function-design に returnTo の生成方法の記述があれば同期（Writer が `rg -n 'buildInventoryRecordsReturnTo|returnTo' docs/function-design` で特定し、無ければ追記しない）
- **S8 GA2 の docs 同期**: `docs/function-design/66-ui-stock-movements.md:83`（§66.3 の「`//` 始まり」「prefix」の旧記述を origin + pathname pin へ）/ `docs/quality/review-checklist.md:83`（「DSR-15 の prefix 検証」の旧表現）/ DSR-18 の送信側の記述に、手組みを存置する 2 site（`StockMovementsPage.tsx` `detailReturnTo` / `products/lib/return-to.ts` `buildProductListReturnTo`）の例外を 1 句
- **S7 test**: Test Design Matrix の T1〜T8（GA2 で T1 / T2 / T4 に case を追加、T9 を新設）。REQ 付き test 名を追加・変更するため `cargo run --bin generate_traceability` で `90-traceability.md` を再生成する

## Non-scope

- `src/features/products/**` の returnTo（typed parse-back、DSR-18 で存置）
- `StockMovementsPage.tsx` の `detailReturnTo` 手組み（起票時実測のとおり数字だけの string 値を持たない。S5 と同じ href 化は入れ子の形が変わるため別判断）
- route schema の `returnTo: z.string().max(500)` の見直し、`src/routes/**` の変更
- 送信側の既存 site（`useRouterState` の href を送る既存 site。closeout 実測で本 lane が触った `InventoryRecordsPage.tsx` を除き 6 file）
- Backlog 同行の closeout 以外の `Plans.md` / `backlog.md` 編集（closeout PR で行う）

## Acceptance Criteria

- **AC1** `npx vitest run src/lib/return-to.test.ts` が pass し、T1 の新 case（`/\evil.example` / `/\t/evil.example` / `/ok#frag` → hash なし）と T2（`returnToLinkProps`）を含む
- **AC2** `rg -c '<Link to=\{(backHref|safeReturnTo)\}>' src/features` の合計が 0（baseline 13、逐語実行で確認: `rg -n '<Link to=\{(backHref|safeReturnTo)\}>' src/features | wc -l` = 13）
- **AC3** `rg -n 'buildInventoryRecordsReturnTo' src` = 0 hit（baseline: 2 hit、`InventoryRecordsPage.tsx:54,82`）
- **AC4** `npx vitest run src/features/inventory-records src/features/stock-movements src/features/stock-inquiry src/lib` が pass。既存 test の期待 href を変える場合は、変更前後の href を `defaultParseSearch` した結果が deep-equal であることを PR body に 1 行ずつ示す（percent-encoding の差だけを許す。router が作った href を入力にした case は文字列一致を要求する、T3 / T8）
- **AC5** mutant（Writer が注入 → FAIL を確認 → 復元、Final Reviewer が独立に再注入）: (1) S1 の origin 判定を旧 `startsWith` に戻す → T1 の `/\` case が FAIL (2) S2 の `options.pathname` 判定を外す → T4 が FAIL (3) S5 を手組みへ戻す → T6 が FAIL (4) S4 を `max(20)` へ戻す → T7 が FAIL (5) S2 の `search` を `{}` 固定にする → T3（検索条件つき returnTo）が FAIL (6)（GA2）pin の比較を `startsWith` に緩める → T4 の `/stocktake?page=2` case が FAIL (7)（GA2）fallback を分解せず `{ to: fallback, search: {} }` で返す → T2 の query 付き fallback case が FAIL (8)（GA2）pathname の `//` 判定を外す → T1 の `/..//evil.example` case が FAIL
- **AC6** `bash scripts/local-ci.sh full` が pass（traceability 再生成を含む生成系検査が clean）
- **AC7** `bash scripts/doc-consistency-check.sh` が ERROR 0
- **AC-L3-1**（owner、Windows native）: 結果は `python3 scripts/pr-gate.py record` の manual record に残す。(a) 入出庫履歴で数字だけの語（13 桁 JAN か数字だけの商品コード）を検索 → 任意の行の詳細 → 「前の画面へ戻る」→ 検索語と結果が残っている (b) 在庫照会で検索 → 商品を選択 → 在庫変動履歴 → 元記録の詳細 → 「前の画面へ戻る」→ 「在庫照会へ戻る」で、検索条件と選択行が戻る（NAV-1 の 3 段往復が壊れていない）

## Design Sources

- Requirements / spec: REQ-207（業務記録の参照と戻り導線）/ REQ-303（在庫変動履歴）/ REQ-301（在庫照会）
- Architecture: `docs/ARCHITECTURE.md`（UI 層のみ。CMD / BIZ / IO 不変）
- Function / command / DTO: `docs/function-design/66-*.md` UI-06c-D2 / UI-06c-D9
- DB: 該当なし
- Screen / UI: `docs/design-system/01-decision-rules.md` DSR-15 / DSR-17 (a) / DSR-18
- Decision log / ADR: 新規 D-xxx なし（DSR の extend で足りる）

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | 該当なし | existing sufficient |
| Command / DTO / generated binding / wire shape | 該当なし（bindings 不変） | existing sufficient |
| DB / transaction / audit / rollback / migration | 該当なし | existing sufficient |
| Screen / UI / route state / Japanese wording | DSR-15 / DSR-18 / 66 UI-06c-D9 | updated in this PR（S6） |
| CSV / TSV / report / import / export format | 該当なし | existing sufficient |
| Durable decision / ADR | DSR 改訂履歴 1 行 | updated in this PR |

## Registration / Generation Obligations

| 新規追加物 | 登録・生成義務 |
|---|---|
| REQ coverage 追加（設計書・テスト追加） | `cargo run --bin generate_traceability` で `90-traceability.md` 再生成（S7） |

他の行（command / function-design doc 新設 / route 新設 / operator 画面新設 / consultation relay）は該当なし。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| REQ-207 | 01-decision-rules DSR-15 | DSR-15（改訂） | prefix 判定は `/\` と tab を通す（Probe 1）。URL 解析後の origin 一致は products 側で確立済みの形。棄却: 拒否文字の列挙（漏れる） | S1 | T1 |
| REQ-207 | 01-decision-rules DSR-18 | DSR-18（改訂） | 文字列 `to` は規約外（`InventoryRecordsPage.tsx:69-71`）。helper 1 つで guard と分解を同時に行い、site ごとの複製を作らない | S2 / S3 | T2 / T3 / T5 |
| REQ-303 | 66 UI-06c-D9 | UI-06c-D9（改訂） | label「在庫照会へ戻る」と行き先の一致。棄却: label を「前の画面へ戻る」へ変える（NAV-1 で不採用済み） | S3 | T4 |
| REQ-301 | 66 / stock-inquiry types | UI-06c-D9 | 商品コードに長さ上限が無いのに `selected` だけ 20。`q` と同じ 100 に揃える。棄却: 商品コードに上限を新設（DB / import の契約変更で別 lane） | S4 | T7 |
| REQ-207 | 01-decision-rules DSR-18 | DSR-18（改訂） | 送信側は router の href（引用符付き直列化）を使う本則。手組みは数字だけの string を number にする（Probe 3） | S5 | T6 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: S6 の改訂後は yes（DSR-15 / DSR-18 / UI-06c-D9 に判定方法・理由・例が入る）
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: 「hash は落とす」「送信側は router の href」を DSR-18 へ
- Assumptions and constraints: router の search 直列化は既定のまま（起票時実測）。custom `parseSearch` を入れる change は本 helper を同時に見直す
- Deferred design gaps, risk, and follow-up target: `detailReturnTo` の手組み（Non-scope、現状は無害）/ 商品コードの長さ上限の不在（backlog 候補、Coordinator が closeout で起票）
- Test Design Matrix can cite design decision IDs or source doc sections: yes
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: 「app 外 origin を描画しない」は T1 の拒否 case と AC5 (1) で検査。escape hatch = 呼出側 fallback（不変）。products の exact-allowlist は非接触

## Impact Review Lenses

Fact check / design decision split lens のみ該当: 外部 library（TanStack Router 1.168.23）の search 直列化の挙動を Contract Probe 1〜3 で事実確認し、設計判断（S1〜S5）と分けて記録した。他の lens は not applicable（field 調査・実機・POS / CSV 起点ではない）。

## Design Readiness

- Existing design docs are sufficient because: 挙動の本則（遷移元へ戻る、不正は fallback）は DSR-18 で確定済み。本 lane は検証強度と組み方の是正
- Source docs updated in this PR: DSR-15 / DSR-18 / 66 UI-06c-D9（S6）
- Design gaps intentionally deferred: `detailReturnTo` 手組み / 商品コード長の上限
- Durable decisions discovered in this plan and promoted to source docs: hash を落とす / 送信側は router の href

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): UI のみ
- Backend function design: 該当なし
- Command / DTO / data contract: 不変
- Persistence / transaction / audit impact: なし
- Operator workflow / Japanese UI wording: 文言不変
- Error, empty, retry, and recovery behavior: 不正 `returnTo` は既定 hub へ fallback（不変、拒否範囲のみ拡大）
- Testability and traceability IDs: REQ-207 / REQ-301 / REQ-303、SPEC-RETURNTO-HYGIENE-2026-09-17

## Contract Probe

- `new URL` が `/\host` と tab 入りを外部 origin に解決する: node で 9 値を解析 -> 起票時実測 Probe 1 のとおり（現 guard は 2 値を通す）
- router が作った href は `defaultParseSearch` → `<Link search>` で同じ search に戻る: node で stringify → parse -> Probe 2 のとおり往復
- 手組みの `q=<数字だけ>` は route schema で落ちる: node で parse → zod -> Probe 3 のとおり `undefined`
- 未実測（Writer が実装時に確認し PR body へ記録）: `<Link to={pathname} search={parsed}>` の描画 href が、既存 test の期待 href と `defaultParseSearch` 後に deep-equal であること（AC4）

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| DSR-15 不正 returnTo は fallback（外部 URL / `//` / `/\` / 制御文字） | S1 | T1 | — |
| DSR-18 遷移元の pathname + search へ戻る（業務記録詳細 6 画面） | S2 / S3 | T3 / T5、既存 T11 系（6 file の `returnTo %s を安全に %s へ正規化する`） | AC-L3-1 (a) |
| DSR-18 送信側は現在の pathname + search state を直列化 | S5 | T6 | AC-L3-1 (a) |
| DSR-18 products の exact-allowlist は存置 | 非接触 | 既存 `src/features/products/lib/return-to.test.ts` | non-scope |
| DSR-17 (a) / (b) 戻り先の href が遷移元の href を再現する（href key の scroll 復元と相互補完） | S2 / S3（router が作った href を `returnTo` にした場合、戻り link の href は元の href と文字列一致） | T3 / T8（文字列一致の assertion） | AC-L3-1 (a) |
| DSR-17 (a) 戻りは `<Link>` push のまま | S3（`history.back()` にしない） | 既存 `app-router.test.tsx` T10 | — |
| UI-06c-D9 「在庫照会へ戻る」は `/stock` に着地、欠落・不正は `q` / `selected` fallback | S3 | T4、既存 `SPEC-UI06C-D9-R1` 4 case + T2-num | AC-L3-1 (b) |
| UI-06c-D9 入れ子 `returnTo` の 3 段往復 | S1（入れ子を壊さない） | T1（入れ子 `returnTo` つきの値が guard を通る case）、既存 `StockMovementsPage.test.tsx:435`（href に `%26returnTo%3D` を含む静的 assertion） | AC-L3-1 (b)（実クリックの 3 段往復） |
| UI-06c-D2 `returnTo` は検索 query に渡さない | 非接触 | 既存 | — |
| stock-inquiry `selected` は商品コードを落とさない | S4 | T7 | — |
| SPEC-RETURNTO-HYGIENE-2026-09-17 C7 どの入力でも throw しない | S1 / S2 | T1・T2 の throw-safety case 群 / T9 | — |

## Test Plan

[Test Design Matrix](test-matrices/2026-09-17-return-to-hygiene.md)。

- targeted tests: T1〜T9
- negative tests: T1（拒否 case）/ T4（pathname 不一致）
- compatibility checks: 既存の期待 href（AC4）、products 側 test 非接触
- data safety checks: 該当なし（synthetic 値のみ）
- main wiring/integration checks: T8（実 routeTree + memory history。既存 `ReturnToFlow.test.tsx` の harness〈現状は T10 の 2 段往復 1 本〉を再利用して入出庫履歴の往復を足す。在庫照会起点の 3 段往復の実クリックは自動 test に無く、AC-L3-1 (b) が受け持つ）
- commit 構成: test と実装は同 commit でよい。docs（S6）と `90-traceability.md` 再生成は別 commit でよい。packet / Matrix / `Plans.md` は Writer が編集しない

## Boundary / Wire Contract

- producer: 送信側 site（`useRouterState` の `location.href`。S5 で入出庫履歴も同じ形）
- consumer: `returnToLinkProps` / `normalizeReturnTo`（業務記録詳細 6 画面、在庫変動履歴）
- wire type: URL search param `returnTo`（string、route schema `max(500)`、不変）
- internal type: `{ to: string; search: Record<string, unknown> }`
- precision/range: pathname + search のみ（hash は落とす）。origin は `http://inventory.local` 基準で一致必須
- round-trip path: 一覧 href → `returnTo` → 詳細の戻り link（`to` + `search`）→ router stringify → 一覧 route の `validateSearch`
- invalid input: 欠落 / 空 / `/` 始まりでない / 外部 origin に解決される / URL として解析不能 / `options.pathname` 不一致 → fallback
- compatibility: 既存の bookmark 相当（手組み形式の `returnTo`）は引き続き受理する（数字だけの値が落ちる既存挙動は受信側では直さない。送信側 S5 で発生源を止める）

## Review Focus

- guard を強めて正当な入れ子 `returnTo` を落としていないか（T8）
- 既存 test の期待 href を書き換えた箇所が encoding 差だけか（AC4）
- helper が 1 箇所に収まり、site ごとの分解ロジックの複製が無いか（保守者可読性）
- 戻り link の href が元の一覧 href と文字列一致するか（DSR-17 (b) の scroll 復元 key は `location.href`。AC4 が許す percent-encoding 差は手組み形式の旧 `returnTo` に限り、router が作った href では差を許さない。差が出た場合の影響は scroll 位置が先頭へ落ちるだけで Goal Invariant 外）
- S5 で href を使うことで `returnTo` に既定値の search（`recordType=all` 等）が乗る場合、戻り先の表示が変わらないか

## Spec Contract

Contract ID: SPEC-RETURNTO-HYGIENE-2026-09-17

- C1: `normalizeReturnTo` は base origin に解決され、かつ解決後の pathname が `//` 始まりでない `/` 始まりの値だけを `pathname + search` で返し、それ以外は fallback を返す。返り値を再び `normalizeReturnTo` に通しても同じ値になる（GA2）
- C2: `returnToLinkProps` は C1 を通った値を `to`（pathname）と `search`（`defaultParseSearch`）へ分解し、`options.pathname` 不一致は不正値として扱う
- C7（GA2）: `normalizeReturnTo` / `returnToLinkProps` はどの string 入力でも throw せず、不正値で詳細画面を ErrorBoundary へ落とさない
- C3: 業務記録詳細 6 画面と在庫変動履歴の戻り link は文字列 `to` に query を含めない
- C4: 「在庫照会へ戻る」は常に `/stock` に着地する
- C5: 入出庫履歴が送る `returnTo` は、数字だけの `q` を string のまま往復させる
- C6: `stockInquirySearchSchema.selected` は 100 文字まで受理する

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| C1 | S1 | T1 | 拒否範囲と既存 8 case 不変 | AC1 / AC5 (1) |
| C2 | S2 | T2 / T3 | search の分解 | AC1 / AC5 (5) |
| C3 | S3 | T5 | 複製なし | AC2 |
| C4 | S3 | T4 | pin | AC5 (2) / AC-L3-1 (b) |
| C5 | S5 | T6 | 既定値 search の混入 | AC3 / AC5 (3) / AC-L3-1 (a) |
| C6 | S4 | T7 | — | AC5 (4) |
| C1〜C5 | S1〜S5 | T8 | 3 段往復 | AC4 / AC-L3-1 (b) |
| C7 | S1 / S2 | T9 / T2 | throw 安全性 | AC5 (7)(8) |

## Data Safety

- 実店舗の JAN・商品名・金額を test / docs / PR に入れない。数字だけの検索語は既存 fixture の `2099000000019`（synthetic）を使う
- local-only paths: なし
- synthetic-only paths: `src/**/*.test.tsx` の fixture

## Implementation Results

[PR #78](https://github.com/higekiri-w/inventory-system-desktop/pull/78) で実装し squash merge 済み（`a68ba291`、2026-09-17）。初回実装（S1〜S8）は Sonnet subagent（commit `1839de7d` / `bd56e4da` / `b4364544`）。Final Review round 1 の是正は Codex（GA2、発注書 63）が担当し、`normalizeReturnTo` / `returnToLinkProps` の URL 解析を 1 回へ集約、解決後 pathname の `//` 拒否、throw しない契約（C7）、pin の近傍負例、query 付き fallback の分解 test、66 / review-checklist の旧記述同期を実装した。Final Review round 2 の P3 3 件（コメントの site 数表現、DSR-15 の主語、pin 併用時 fallback の責務コメント）は Sonnet subagent が是正した。

## Gated Amendments

### Gated Amendment 1（2026-09-17、Final Reviewer を Codex + Opus へ）

- 経緯: 起票時に Coordinator が relay 0 を優先し、Writer = Sonnet subagent / Final Reviewer = Sonnet + Opus と組んだ。owner 2026-09-17「実装も Sonnet に任せてるならレビューは外にお願いしないとまずくない？」「大事なのは成果物の質だよ。relay を 0 回にしてもその作業の質が担保できないと有効な修正とは言いにくい」
- 判断: AGENT_OPERATING_MANUAL §3 の独立性制約（Writer ≠ Final Reviewer、fresh context）と D-062（vendor 分離は Writer = Codex の packet 限定）の文言には反しないが、同系統が自分の成果物を採点する盲点の重なりは D-062 の Why と同じ機序で Sonnet にも当てはまる。owner 既定の UI 座組（Sonnet Writer / Opus レビュー / Codex ロジックレビュー 1 回）にも合わせる
- 是正: Final Reviewer を Codex（Contract Audit、pass A）+ Opus（pass B）へ。Final Review Minimum 2 は不変。relay 往復上限 0 → 1。Scope / 設計判断 / AC / Matrix / Spec Contract は不変
- Owner Effort Budget: 介入 1/3（本指摘を decision point として計上）、relay 0/1
- 時点: Writer の実装 run は完了済み（commit `1839de7d` / `bd56e4da` / `b4364544`）。Coordinator が mutant 5 本を clean tree で独立に再注入し、Writer 報告どおりの red / green を確認した後の登録

### Gated Amendment 2（2026-09-17、Final Review round 1 の是正）

- round 1 の結果: pass A = Codex（[review](https://github.com/higekiri-w/inventory-system-desktop/pull/78#pullrequestreview-5233517717)）P1 0 / P2 2 / P3 2、pass B = Opus P1 0 / P2 1 / P3 2。両 pass とも AC1〜AC7 と mutant 5 本の Writer 報告を独立に再現し、実 routeTree の実クリックで 3 段往復（数字だけの商品コード、記号入りの検索語）が文字列一致で戻ることを確認した
- **根は 1 つ、原因は packet の設計指示**: S1 を「origin 一致」だけで書き、S2 で「`normalizeReturnTo` の返り値を `decompose` が再解析する」2 段構えを許した。`/..//evil.example` は 1 回目の解析で base origin に解決されて guard を通り、dot-segment の畳み込みで返り値が `//evil.example` になる（Opus P2）。`/a/..//[` では 2 回目の `new URL` が throw し、実 route で詳細画面が ErrorBoundary へ落ちて戻り link を失う（Codex F1。baseline では描画できていたので本 PR の退行）。先例 `products/lib/return-to.ts` は pathname allowlist の 2 段目を持つため同じ入力で安全で、Coordinator が先例の 2 段目を S1 へ移植し損ねた
- 裁定（全件 accept）: Codex F1 + Opus P2 → S1 / S2 / C1 / C7（解析は 1 回、`//` 始まりの pathname を拒否、throw しない）。Codex F2（pin の近傍負例が無く `startsWith` へ緩める mutant が 339 本 green のまま生存）→ T4 と AC5 (6)。Codex F3 + Opus P3（fallback の分解・検証に test が無い / fallback を無検証で `new URL` する）→ T2 と AC5 (7)、fallback も同じ 1 回解析の経路を通す。Codex F4 + Opus P3（66 §66.3 / review-checklist:83 の旧「prefix」「`//` 始まり」、DSR-18 の送信側の記述と手組み 2 site の食い違い）→ S8
- Scope の増分: `docs/quality/review-checklist.md`（1 行）。実装 file の集合は不変（`src/lib/return-to.ts` と test）。consumer 7 file は helper の署名が変わらなければ触らない
- Writer: 是正 run は Codex（発注書 63）。owner 2026-09-17「大事なのは成果物の質」。round 2 の closure は Codex を外し Opus + Sonnet
- Owner Effort Budget: 介入 1/3、relay 1/2（発注書 62 の review で 1 消費）

### Gated Amendment 3（2026-09-17、是正 run の再実行と relay 上限 2 → 3）

- 経緯: Codex 発注書 63 run 1 が実装前に fail-closed 停止（commit 0、HEAD `2d6c3f78` のまま、作業 worktree は除去済み）。発注書の「`.local/**` を編集しない」と、必須手順の `bash scripts/local-ci.sh full`（`scripts/local-ci.sh:25` が `.local/ci-evidence/` へ log を書く）が literal に衝突した。Codex の停止判断は正しく、原因は Coordinator が発注書 61 から禁止行を写したこと
- 是正: 発注書 63 の禁止を tracked の具体 path に絞り、「許容する書込み」節（指定 command が自動生成する untracked / gitignore 出力）を明示。Scope / 設計判断 / AC / Matrix / Spec Contract は不変。同 commit で Test Plan の `targeted tests` を `T1〜T9` へ（GA2 の sweep 漏れ）
- Owner Effort Budget: relay 上限は hard stop のため owner に選択肢（上限を 3 にして Codex 再実行 / Sonnet Writer で relay を増やさない）を示し、owner 2026-09-17「1で」= 上限 3 で Codex 再実行。理由 = guard の核心の直しで質を優先（owner「大事なのは成果物の質」）。介入 2/3、relay 2/3（review 1 + 空振り 1）
- 教訓: 発注書の禁止一覧は、検証節の各 command の出力先と 1 行ずつ突合してから出す

## Review Response

- Findings Freeze: frozen after Broad Audit（round 2）; post-freeze exceptions: closure の P3 2 件を closeout で対応。

### Plan Review round 1（2026-09-17、Sonnet、独立 fresh context、read-only、対象 `b5cc6b5e`）

- P1 0 / P2 0 / P3 3 → Plan Gate 通過可。Contract Probe 1〜3 と AC2 / AC3 の baseline を逐語再実行で一致確認、router-core の source で `to` に埋めた query が path として echo されるだけで `search` props 側が状態遷移に使われることを確認、引用した既存 test の実在を確認
- P3（DSR-17 (b) の scroll 復元 key = `location.href` との交差契約が Ledger に無い）= accept → Ledger に DSR-17 (a) / (b) 行、Review Focus、AC4、Matrix T3 / T8 に「router が作った href は文字列一致」を追加
- P3（Matrix T8 / Test Plan の「既存の 3 段往復 case」が実態より強い表現。実在は T10 の 2 段往復 1 本と `StockMovementsPage.test.tsx:435` の静的 assertion）= accept → 表現を実態へ、T1 に入れ子 `returnTo` つきの通過 case を追加、3 段往復の実クリックは AC-L3-1 (b) が受け持つと明記
- P3（Impact Review Lenses の not applicable が簡略）= accept → Fact check / design decision split lens を Contract Probe で実施と明記
- P3 のみのため reviewer 再投入なし（Subagent Budget）

### Final Review round 1（2026-09-17、head `1c01f0ce`）

- pass A（Codex、Contract Audit、[review](https://github.com/higekiri-w/inventory-system-desktop/pull/78#pullrequestreview-5233517717)）: P1 0 / P2 2 / P3 2。F1（`return-to.ts:25` 相当、正規化後の pathname を再度 `new URL` に通すと `/a/..//[` が throw し詳細画面が ErrorBoundary へ落ちる）/ F2（`/stock` の exact pin に近傍負例が無く、`!==` を `startsWith` へ緩める mutant が AC4 全 339 本 green のまま生存）/ F3（fallback を無検証で分解する test が無く、fallback 未分解 mutant が生存）/ F4（66 §66.3 / review-checklist:83 に旧「prefix」「`//` 始まり」表現が残存）
- pass B（Opus）: P1 0 / P2 1 / P3 2。P2 は Codex F1 と同根（`/..//evil.example` が dot-segment 畳み込みで `//evil.example` に読み替わり guard を回避）。P3 2 件は F2 / F4 と同種の指摘
- 裁定（GA2、全件 accept）: 根は 1 つ（S1 の origin 判定と S2 の再解析 2 段構えを packet が許していた）。F1 + Opus P2 → URL 解析を 1 回に集約し `//` 始まり pathname を拒否・throw しない契約（C7）へ。F2 → pin の近傍負例 test。F3 → fallback 分解 test。F4 → docs 3 file 同期。是正 Writer = Codex（発注書 63、GA3 で禁止行の衝突を是正し再実行）

### Final Review round 2 — broad（2026-09-17、head `2d329aa5`）

- pass A（Sonnet、Contract Audit、[comment](https://github.com/higekiri-w/inventory-system-desktop/pull/78#issuecomment-5713123096)）: P1 0 / P2 0 / P3 2。round 1 の 7 件すべて closed を確認。新規 P3: Contract Coverage Ledger / Trace Matrix に C7（no-throw）の独立行が無い（本 commit で追加）/ `options.pathname` 指定時に fallback へ pin が再適用されない（コード変更不要、コメントで責務明示）
- pass B（Opus、[comment](https://github.com/higekiri-w/inventory-system-desktop/pull/78#issuecomment-5713123393)）: P1 0 / P2 0 / P3 4。round 1 の 7 件すべて closed を確認。新規 P3: `InventoryRecordsPage.tsx:67` のコメントの「8 site」が実数と合わない / `65-inventory-record-traceability.md:114` の TRACE-D11 が「`//` で始まる URL」のまま 66 との粒度不整合 / `normalizeReturnTo` の production caller が 0 件（削除は非推奨、DSR-15 の主語の寄せ先） / sentinel `{ to: "", search: {} }` を型で強制する案（署名変更）
- 裁定: 両 pass とも P1/P2 0 のため review = pass 可。round 2 の P3 6 件は「この PR で直す（3 件）/ closeout・backlog へ（3 件）」に振り分け、closure で反映を確認

### Closure review（2026-09-17、Opus、独立 fresh context、[comment](https://github.com/higekiri-w/inventory-system-desktop/pull/78#issuecomment-5713358531)）

- 判定: P1 0 / P2 0 / P3 2 — closure pass。delta = `2d329aa5..d71437bc`（6 file・30 行、src の実コード差分 0 行、コメントのみ）
- round 2 の P3 6 件の裁定（Coordinator）: この PR で直す = Opus P3-1（コメントの「8 site」）/ Opus P3-3（DSR-15 の主語）/ Sonnet P3（pin 指定時の fallback の責務コメント）→ すべて closed（`2c9ffc9b` / `9beeb794` / コメント追記）。closeout・backlog へ = Opus P3-2（65 TRACE-D11、Scope 外）/ Sonnet P3（packet の Ledger に C7 行。packet 改訂は GA になり broad を無効化するため見送り）/ Opus P3-4（sentinel の型強制、署名変更を伴う）
- 新規 P3 2 件（非 blocker、closeout へ）: `InventoryRecordsPage.tsx:67` の「送信側はほかの送信側と同じく」が同語反復 / packet `:94` `:104` に「送信側 8 site」が残存（archive 移送時に実測つきの数へ）

### helper record（2026-09-17）

- github mode の専用 record: review = pass（closure の裁定を反映）、manual = pass（下記 owner L3）で登録。R4 = not-required。GitHub PR #78 の Ready 化・hosted CI green・owner merge を経て `a68ba291` へ squash merge

### owner L3（Windows native、AC-L3-1、2026-09-17、[comment](https://github.com/higekiri-w/inventory-system-desktop/pull/78#issuecomment-5713473067)）

- 対象 head `d71437bc`。owner 原文「(a) OK / (b) OK」= PASS。(a) 入出庫履歴で数字だけの語を検索 → 詳細 → 「前の画面へ戻る」で検索語と結果が残った一覧へ戻る。(b) 在庫照会で検索・商品選択 → 在庫変動履歴 → 元記録の詳細 → 「前の画面へ戻る」→「在庫照会へ戻る」で絞り込み・検索条件・選択行が残る（NAV-1 の 3 段往復）
- 任意項目（scroll 位置の復元）は owner の言及なし = 未確認のまま

### Closeout（本 commit、Findings 解消）

- closure P3 2 件のうち「`InventoryRecordsPage.tsx:67` の同語反復コメント」は本 commit の src コメント修正で対応。「packet の『送信側 8 site』」は本 commit の archive 移送で実測つきの表現へ訂正
- round 2 pass A P3（Ledger / Trace Matrix に C7 行が無い）は本 commit で追加
- round 2 pass B P3-2（65 TRACE-D11 の「`//` で始まる URL」）は本 commit で docs 修正
- round 2 pass B P3-4（sentinel `{ to: "", search: {} }` の型強制）は署名変更を伴う runtime のため backlog へ起票
