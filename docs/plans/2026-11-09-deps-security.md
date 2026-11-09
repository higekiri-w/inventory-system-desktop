# Plan Packet: 依存の脆弱性を直し、zod 4.6 の長さの数え方の変更から UTF-16 code unit の契約を守る

wave に属さない単独の lane。npm の依存 5 つ（`zod`・`vite`・`vitest`・`happy-dom`・間接の `seroval`）を、high の脆弱性が直る版へ名指しで上げる。`zod` 4.6 で文字列の長さの検査の数え方が変わるので、route search の長さの検査を今の数え方（UTF-16 code unit）のまま保つ helper に置き換える。Rust（`src-tauri/**`）は変えない。

出所: owner の決定 TD-606（2026-11-09、repo 外の回答台帳。脆弱性の lane を起こす）。Dependabot の open の alert 13 件（owner の terminal で `gh api 'repos/higekiri-w/inventory-system-desktop/dependabot/alerts?state=open&per_page=100'` を 2026-11-09 に実行。全部 npm の `package-lock.json`: `seroval` medium 2、`happy-dom` high 2・medium 1、`zod` high 1、`@vitest/mocker` medium 2、`vite` high 1・medium 3・low 1）と、`npm audit` の high 3 件（`happy-dom`・`vite`・`zod`）。

decision-log の番号: 本 lane は D-151 を予約する（main は D-150 まで。起票の時点で open の PR が足す D は D-148〈#227〉・D-149〈#228〉。`git diff origin/main...origin/agent/<branch> -- docs/decision-log.md` の `+## D-` と各 branch の `docs/plans` の差分の「予約」で確かめた）。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R3

Reason:
依存の更新（`package.json`・`package-lock.json`）と、route と search の状態（route search の schema の長さの検査）を変える。`docs/DEV_WORKFLOW.md` Risk Tiers の R3（「route と search の状態」「依存の追加と更新」）。依存に触れる変更は R2 にできない。R4 にしない理由: 破壊的なデータの操作・実データ・secret・不可逆な操作を伴わない。

## Goal

Goal Invariant:

### 最小完了条件

- `npm audit --audit-level=high` が 0 件で、Dependabot の open の 13 件の対象（`seroval`・`happy-dom`・`zod`・`@vitest/mocker`・`vite`）が、それぞれの alert の first patched の版以上になる。
- route search の文字列の長さの検査 13 か所が、更新の前と同じく UTF-16 code unit で数える。BMP の外の文字（例 `\u{2000B}`、2 unit）を含む文字列で、上限・下限の受理と拒否が更新の前と変わらない。在庫照会の `selected`・`q` と商品コードの上限（BIZ-01-D5）が食い違わない。
- frontend の既存の test が全部 PASS し、型の検査・build・route の生成が通る。

### 失敗定義

- `zod` 4.6 の code point の数え方のまま、BMP の外の文字で上限を超える文字列を受理する（または下限を下回る文字列を受理する）。
- 名指しでない更新、7 日の cooldown の前の版の採用、install script の実行、名指しの外の package の追加が lockfile に入る。
- `happy-dom` 21 で落ちる test を、skip・弱体化・削除で通す。

### 非目的

- `typescript-eslint` 経由の `minimatch` の moderate 7 件（Dependabot の対象外、high でない）。backlog に残す。
- 長さの数え方を code point に変えること（製品の振舞いの変更で、owner の判断が要る）。
- TanStack Router の版の更新（`seroval`・`zod` は Router の range〈`^1.6.7`・`^4.5.4`〉の中で lockfile だけが動く）。
- CI の npm を 12 系にすること（D-066 のまま CI は Node 24 同梱の npm 11 系）。
- Rust の長さの検査（`encode_utf16().count()`。変えない）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

### 依存の更新（`package.json`・`package-lock.json`）

手元の toolchain は D-066 の npm 12.0.2 と `.node-version` の Node 24.18.0（`mise exec node@24.18.0 -- …`。repo の root の `mise.local.toml` が無い checkout でも版を固定する）。D-030 のとおり 1 つずつ名指しで入れ、順に:

1. `npm install zod@4.6.7 --save-exact`
2. `npm install vite@7.3.7 --save-dev --save-exact`
3. `npm install vitest@4.1.12 --save-dev --save-exact`（`@vitest/mocker` 4.1.12 が付いてくる）
4. `npm install happy-dom@21.0.4 --save-dev --save-exact`
5. `npm update seroval`（名指し。lockfile だけ）

版はどれも publish から 7 日を過ぎている（`npm view <pkg> time --json`、2026-11-09: zod 4.6.7 は 2026-10-12、vite 7.3.7 は 2026-10-06、vitest 4.1.12 は 2026-10-15、happy-dom 21.0.4 は 2026-10-20）。`seroval` は `min-release-age=7` が解決する版（実測は 1.7.6）を採る。

実測（2026-11-09、`origin/main` の一時の worktree で上の 1〜5 を npm 12.0.2・Node 24.18.0 で流した）: `package.json` は `zod` `4.3.6`→`4.6.7`、`happy-dom` `^20.10.6`→`21.0.4`、`vite` `^7.3.6`→`7.3.7`、`vitest` `4.1.11`→`4.1.12`。`package-lock.json` の `packages` は足すもの 0、消すもの 2（`node_modules/@tanstack/router-generator/node_modules/zod`・`node_modules/@tanstack/router-plugin/node_modules/zod`。直接の zod に寄る）、版が変わるもの 14（`@vitest/expect`・`@vitest/mocker`・`@vitest/pretty-format`・`@vitest/runner`・`@vitest/snapshot`・`@vitest/spy`・`@vitest/utils` 4.1.11→4.1.12、`chai` 6.2.2→6.3.0、`happy-dom` 20.10.6→21.0.4、`seroval` 1.6.8→1.7.6、`tinyrainbow` 3.1.1→3.2.0、`vite` 7.3.6→7.3.7、`vitest` 4.1.11→4.1.12、`zod` 4.3.6→4.6.7）。`npm audit --audit-level=high` は exit 0（moderate 7 件は全部 `minimatch`）。`npm run test` は 2059 件中 1 件だけ失敗（`src/features/stock-inquiry/types.test.ts` の `selected` の UTF-16 の境界。下の zod の数え方の変更による）。型の検査・build・route の生成・`eslint src`・`npm ci --ignore-scripts --dry-run` は通った。

### zod の長さの数え方（外の前提、確かめ済み）

`zod` 4.6.7 の `node_modules/zod/v4/core/checks.js` の長さの検査（`maxLength`・`minLength`・`lengthEquals`）は、文字列の `length`（UTF-16 code unit）が境界を外れるとき `util.codePointLength(input)`（code point の数）で数え直して判定する。4.3.6 は `length` で判定する。BMP の外の文字は 1 code point・2 code unit なので、4.6.7 の `.max(n)` は 4.3.6 より長い文字列を受理し、`.min(n)` は短い文字列を受理しうる。

### helper と呼出し側（`src/**`）

- `src/lib/utf16-string.ts` に、UTF-16 code unit で長さを数える zod の文字列の schema を作る helper `utf16String({ min, max })` を足す（`max` は必須、`min` は任意。`.optional().catch(undefined)` を後ろに繋げられる）。判定は JavaScript の `String.prototype.length`（UTF-16 code unit）。
- 次の 13 か所の `z.string()…max/min(…)` を helper に置き換える。境界の値は変えない。
  - `q` の max 100: `src/features/inventory-records/types.ts`、`src/features/products/priceRevisionSearch.ts`、`src/features/products/search.ts`
  - 在庫照会の `q`・`selected` の min 1・max 100: `src/features/stock-inquiry/types.ts`（2 か所）
  - `returnTo` の max 500: `src/features/stock-movements/types.ts`、`src/routes/csv-import.records.$importId.tsx`、`src/routes/inventory/disposal/records/$recordId.tsx`、`src/routes/inventory/manual-sale.records.$recordId.tsx`、`src/routes/inventory/receiving.records.$recordId.tsx`、`src/routes/inventory/return.records.$recordId.tsx`、`src/routes/stocktake.records.$stocktakeId.tsx`
  - 棚卸しの `product` の min 1: `src/routes/stocktake/index.tsx`
- 一覧は `git grep -n -E 'z\.string\(\)[^;]*\.(max|min|length)\(' -- 'src/**/*.ts' 'src/**/*.tsx' ':!**/*.test.*'` の出力（2026-11-09、13 行）。
- 呼出し側・隣接: どの schema も route の `validateSearch` か feature の search の型の元で、`.optional().catch(undefined)` で不正な値を `undefined` に落とす（DSR-15 の returnTo の検証は `src/lib/return-to.ts` の `returnToLinkProps` が別に持ち、本 lane は変えない）。`src/routeTree.gen.ts` は生成物で、route の file の schema の書き換えで差分が出ないことを `npm run generate:routes` の後の `git diff --exit-code src/routeTree.gen.ts` で確かめる。

### test

- `src/lib/utf16-string.test.ts` に helper の test（下の Test Plan）。test の名前に `REQ-301 / BIZ-01-D5` を付ける（traceability の T4）。
- `src/lib/utf16-string-no-zod-length.test.ts` に静的な test（前例: `src/lib/describe-error-no-local-duplicates.test.ts`）: `src/features`・`src/routes`・`src/components`・`src/lib` の test でない `.ts`・`.tsx` に、`z.string()` から繋げた `.max(`・`.min(`・`.length(` が無い。
- 既存の `src/features/stock-inquiry/types.test.ts` の「`selected` は BIZ と同じ数え方で 100 まで受理する」を変えずに通す（この lane の回帰の test）。

### 正本の文

- `docs/function-design/30-biz-product-service.md` の BIZ-01-D5 の設計判断: 「数え方は zod `.max()` と同じ UTF-16 code unit」を、「数え方は UTF-16 code unit（frontend は `src/lib/utf16-string.ts` の `utf16String`。zod 4.6 の `.max()` は code point で数えるので使わない。D-151）」の趣旨に直す（本 commit）。
- `docs/function-design/58-ui-stock-inquiry.md` の route search の schema の例（`q`・`selected`）を helper の形に直す（本 commit）。
- `docs/decision-log.md`: 末尾に D-151（本 commit）。
- `docs/backlog.md`: `minimatch` の moderate の項目を 1 つ足す（本 commit）。
- 本 packet（本 commit）。

Non-scope: `docs/archive/**` の schema の引用（当時の記録）。`src-tauri/**`。`.npmrc`。CI の workflow。

## Acceptance Criteria

- AC1: `mise exec node@24.18.0 -- npm audit --audit-level=high` が exit 0。`package.json` の 4 つの版が Scope の値の exact（`^` なし）。`package-lock.json` の差分が Scope の実測と同じ集合（足すもの 0・消すもの 2・版が変わるもの 14。`seroval` の版は `min-release-age` が解決したもの）で、名指しの外の package が足されない。
- AC2: `mise exec node@24.18.0 -- npm ci --ignore-scripts` が通り、その後の `git status --porcelain package-lock.json` が空。
- AC3: helper が UTF-16 code unit で数える。上限 100 で `"A".repeat(100)`・`"\u{2000B}".repeat(50)`・`"あ".repeat(100)` を受理し、`"A".repeat(101)`・`"\u{2000B}".repeat(51)` を拒む。下限 1 で `""` を拒み、`"\u{2000B}"`（1 文字・2 unit）を受理する。上限 500 で `"\u{2000B}".repeat(250)` を受理し `"\u{2000B}".repeat(251)` を拒む。拒否は `.optional().catch(undefined)` を繋げた形で `undefined` になる。
- AC4: 静的な test が、Scope の 13 か所のどれかを `z.string().max(…)` などへ戻すと失敗する。13 か所の置き換えの後、`git grep -n -E 'z\.string\(\)[^;]*\.(max|min|length)\(' -- 'src/**/*.ts' 'src/**/*.tsx' ':!**/*.test.*'` が 0 件。
- AC5: `mise exec node@24.18.0 -- npm run test` が全 PASS（2059 件と新しい test。既存の test を消さず弱めない）。`npm run typecheck`・`npm run build`・`npm run generate:routes`（その後 `git diff --exit-code src/routeTree.gen.ts`）が通る。`./node_modules/.bin/eslint src` が error なし（`npm run lint` は repo の root の `.local/` を拾う手元の事情があるので、hosted の lint で全体を確かめる）。
- AC6: `bash scripts/doc-consistency-check.sh` と `--target plan` が ERROR なし。traceability の検査（`cd src-tauri && cargo run --bin generate_traceability -- --check`）が通る。
- AC7: 実装の報告に、mutation で red になることを書く。helper の判定を code point（`[...s].length`）にすると AC3 の test が red。13 か所のどれか 1 つを `z.string().max(…)` に戻すと静的な test が red。
- AC8（merge の後）: Dependabot の open の alert のうち、Scope の 5 package のものが 0 件になる（owner の terminal の `gh api` で確かめる）。

## Design Readiness

- 引用する設計正本（節まで）: `docs/function-design/30-biz-product-service.md` の BIZ-01-D5 の設計判断、`docs/function-design/58-ui-stock-inquiry.md` の route search の schema、`docs/design-system/01-decision-rules.md` の DSR-15・DSR-18、`docs/decision-log.md` D-030・D-066、`AGENTS.md` の npm 供給網ガード。
- 必要な設計成果物: BIZ-01-D5 の文・58 の例・D-151 を本 commit で書く（updated in this PR）。helper は新しい振舞いを足さず、今の数え方を保つ。
- plan にしかない durable な判断の昇格先: 「frontend の長さの検査は UTF-16 code unit を保ち、zod の長さの検査を使わない」を D-151 と BIZ-01-D5 の文へ。
- 前提・制約と延期した design gap: zod 4.6 の数え方の変更は現物で確かめた（上の「zod の長さの数え方」）。`happy-dom` 21 の変更は、実測で落ちた test が無かったことで確かめた（UI の振舞いの差は test の外。L3 は置かない: 変わるのは test の環境と dev server で、製品の bundle に入るのは `zod` だけ）。`returnTo` の 500 の数え方は正本に無く、今の振舞い（UTF-16 code unit）を保つ。
- 絶対保証の例外と抜け道: 静的な test は `z.string()` から 1 行で繋げた形を見る。複数行に割った書き方や `z.string()` を変数に置いてから繋げる書き方は拾わない（本 lane の 13 か所は全部 1 行の形）。
- 判定: ready。外の前提は実測で確かめ、正本の該当節は 2 か所と新しい D 1 件。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| BIZ-01-D5（在庫照会の `selected`・`q` の上限 100、UTF-16 code unit） | 30 の BIZ-01-D5 の設計判断、58 の route search | helper、`src/features/stock-inquiry/types.ts` | 既存の `selected` の境界の test、helper の test | 非対象 |
| 検索の語 `q` の上限 100 | 50・65・77 の route search（数え方は BIZ-01-D5 と同じに保つ） | helper、`src/features/products/search.ts`・`priceRevisionSearch.ts`・`src/features/inventory-records/types.ts` | helper の test、静的な test | 非対象 |
| DSR-15・DSR-18（`returnTo` の上限 500） | `docs/design-system/01-decision-rules.md` の DSR-15・DSR-18 | helper、`returnTo` の 7 か所 | helper の test（上限 500）、静的な test | 非対象 |
| 棚卸しの `product` の下限 1 | 73 の入口 | helper、`src/routes/stocktake/index.tsx` | helper の test（下限 1）、静的な test | 非対象 |
| D-030・D-066（npm 供給網ガードと手元の npm 12） | `AGENTS.md` の npm 供給網ガード、decision-log D-030・D-066 | 依存の更新の 5 つの command | 非対象（AC1・AC2 の command と lockfile の review） | 非対象 |
| D-151（frontend の長さの検査は UTF-16 code unit） | decision-log D-151 | helper と 13 か所 | helper の test、静的な test | 非対象 |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| BIZ-01-D5 の UTF-16 の上限 | zod 4.6 の code point の数え方で `"\u{2000B}".repeat(51)` を受理する | unit（既存） | `REQ-301 / BIZ-01-D5: selected は BIZ と同じ数え方で 100 まで受理する` | `selected` を `z.string().max(100)` のままにする（zod 4.6.7 で red を実測済み） |
| helper の上限・下限 | helper が code point で数える。上限・下限の境界が 1 つずれる | unit（`src/lib/utf16-string.test.ts`） | `REQ-301 / BIZ-01-D5: utf16String は UTF-16 code unit で長さを数える`（上限 100・500、下限 1 の境界を subTest） | 判定を `[...s].length` にする。`<=` を `<` にする |
| 13 か所が helper を使う | 後の変更で `z.string().max(…)` に戻り、BMP の外の文字で上限を超える | static（`src/lib/utf16-string-no-zod-length.test.ts`） | `REQ-301 / BIZ-01-D5: route search の文字列の長さは utf16String で検査する` | 13 か所のどれか 1 つを `z.string().max(…)` に戻す |
| 依存の更新の後の既存の振舞い | `happy-dom` 21・`vitest` 4.1.12 で test の意味が変わる | 既存の frontend の test 全部 | `npm run test` | 既存の test を skip・削除・弱体化する |
