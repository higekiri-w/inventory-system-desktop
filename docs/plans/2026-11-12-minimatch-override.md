# Plan Packet: Dependabot alert 30 の minimatch を overrides で 11.0.x に上げる

wave に属さない単独の lane。出所は owner の決定 TD-657（2026-11-12、repo 外の回答台帳。Dependabot alert 30 を lane にする）。alert 30 は `minimatch`（scope development、manifest `package-lock.json`、relationship transitive、severity medium、GHSA-g3hv-75wg-m249「minimatch has potential denial of service via globparts amplification」、影響 `>= 7.0.0, < 11.0.1`、first patched `11.0.1`）。本 commit は Plan Packet の起票と backlog の 1 項目の書換えだけで、`package.json`・`package-lock.json`・`.npmrc` は計画の固定の後の実装で変える。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R3

Reason:
依存の更新（`package.json` の `overrides` と `package-lock.json`）で、`docs/DEV_WORKFLOW.md` Risk Tiers の R3 の「依存の追加と更新」に当たる。同じ節の R2 の行き先の規則で、`package.json`・`package-lock.json` に触れる変更は R2 にできない。R4 にしない理由: lint の道具（devDependency の推移的な依存）だけを動かし、製品の bundle・DB・店の実データ・secret・不可逆な操作に触れない。

## Goal

Goal Invariant:

### 最小完了条件

- `@typescript-eslint/typescript-estree` の下の `minimatch` が 11.0.1 以上の 11.0.x になり、`npm ls minimatch --all` に GHSA-g3hv-75wg-m249 の影響の範囲（`>= 7.0.0, < 11.0.1`）の版が残らない。
- `npm audit` の minimatch の moderate 7 件が消え、lint・型の検査・format・vitest・build・`npm ci --ignore-scripts` が今までどおり通り、lint の結果（file ごとの error・warning）が変わらない。
- merge の後、Dependabot alert 30 が閉じる。

### 失敗定義

- `minimatch` 10.2.5 が lockfile に残る、または override が `eslint` 系の `minimatch@3.1.5` まで動かす。
- 7 日の cooldown（`min-release-age=7`）の前の版、名指しでない版の範囲（`^`・`~`）、install script の実行が入る。
- lockfile に Scope の集合の外の package の追加・版の変更が入る。
- minimatch 11 の CJS の口・API の違いで `typescript-eslint` の parser の読込みが落ちる、または lint の結果が黙って変わる。

### 非目的

- `typescript-eslint` の版の更新（8.58.2 のまま。下の案 (c) は override を外す道として backlog に残す）。
- `eslint`・`eslint-plugin-react`・`eslint-plugin-jsx-a11y`・`@eslint/*` の `minimatch@3.1.5`（GHSA-g3hv-75wg-m249 の範囲の外。下の調べ）。
- `eslint.config.js` の変更（`projectService` への移行を含む）。
- CI の npm の版（D-066 のまま、CI は Node 24 同梱の npm 11 系）。
- `.npmrc` の変更・cooldown の迂回（`min-release-age-exclude[]`）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 調べ（2026-11-12、main `24e89091`）

測定の場所: 起票の worktree の HEAD から `git worktree add --detach "$TMPDIR/mm-849-wt" HEAD` で作った隔離した copy（`npm ci --ignore-scripts` の後。調べの後に `git worktree remove` で消した）。手元の toolchain は Node v24.18.0・npm 12.0.2（`node --version`・`npm --version`）。registry の時刻は `npm view <pkg> time --json`（2026-11-12 14:15Z 頃に実行）。

- 今の木（`npm ls minimatch --all`）: 影響の範囲に入るのは `typescript-eslint@8.58.2` → `@typescript-eslint/typescript-estree@8.58.2` → `minimatch@10.2.5`（nested の `node_modules/@typescript-eslint/typescript-estree/node_modules/minimatch`）だけ。ほかは `eslint-plugin-jsx-a11y@6.10.2`・`eslint-plugin-react@7.37.5`・`eslint@9.39.4`・`@eslint/config-array@0.21.2`・`@eslint/eslintrc@3.3.5` の下の `minimatch@3.1.5`（dedupe された 1 つ）。
- `npm audit`: exit 1、`7 moderate severity vulnerabilities`、minimatch の via は `GHSA-g3hv-75wg-m249`（range `>=7.0.0 <11.0.1`）だけ。残りの 6 件は `@typescript-eslint/*` の 6 package が「Depends on vulnerable versions」で数えられたもの。`npm audit --audit-level=high` は exit 0。`fix available via npm audit fix --force`（`typescript-eslint@8.36.0` への breaking change。AGENTS.md の npm 供給網ガードで禁止）。
- 3.1.5 の確かめ: `gh api '/advisories?ecosystem=npm&affects=minimatch'` の 6 件のどれの影響の範囲にも 3.1.5 は入らない（GHSA-g3hv-75wg-m249 は `>= 7.0.0`、ほかの high は `< 3.1.3`・`< 3.1.4`・`< 3.0.5`・`< 3.0.2` と 4 以上の範囲）。
- alert 30（`gh api repos/<owner>/<repo>/dependabot/alerts/30`、read だけ）: state open、`version_requirement` `10.2.5`、vulnerable range `>= 7.0.0, < 11.0.1`、first patched `11.0.1`。発注の写しと一致した。
- 上流の宣言: `@typescript-eslint/typescript-estree` の latest は 8.72.2（2026-11-09T17:06Z）で、`dependencies.minimatch` は `^10.2.2`（今の 8.58.2 と同じ）。canary の `8.72.3-alpha.0`（2026-11-09T23:57Z）は `^10.2.2`、`8.72.3-alpha.2`（2026-11-10T00:30Z）以降と `8.72.3-alpha.8`（2026-11-11T17:05Z）は `^11.0.1`。上流はもう 11 へ上げていて、stable の版にはまだ出ていない。
- minimatch の版と時刻（`npm view minimatch time --json`）: 10.x の最後は 10.2.6（2026-07-27）。11.0.0 2026-10-27T21:19Z、11.0.1 10-27T22:26Z、11.0.2 10-28T01:08Z、11.0.3 10-28T20:00Z、11.0.4 11-02T04:50Z、11.0.5 11-05T23:30Z、11.0.6 11-10T16:35Z、11.0.7 11-10T23:03Z、11.0.8 11-11T09:50Z。
- minimatch 11 の形（`npm view minimatch@<v> engines.node dependencies --json` と `npm view minimatch@11.0.5 type exports --json`）:
  - `"type": "module"`、`exports["."]` に `import`（`dist/esm`）と `require`（`dist/commonjs`）の口がある。
  - engines: 11.0.1〜11.0.3 は `18 || 20 || >=22`、11.0.4 以降は `20 || >=22`（この repo の `package.json` の engines は `>=24 <25`。どちらも満たす）。
  - **依存が 1 つ増える**: 11.0.1 以降は `brace-expansion` `^5.0.12` に加えて `@isaacs/nonrecursive-glob-match` を持つ（11.0.1 は `^1.0.1`、11.0.2 は `^1.0.2`、11.0.3・11.0.4 は `^1.0.3`、11.0.5〜11.0.8 は `^1.1.0`）。この package は 2026-10-26 に初めて publish され（1.0.0 10-26T23:55Z、1.0.1 10-27T04:24Z、1.0.2 10-28T01:01Z、1.0.3 10-28T19:50Z、1.1.0 11-05T21:38Z）、1.0.3 は依存なし、license `BlueOak-1.0.0`、publish の user は minimatch と同じ `isaacs`。scripts は `prepare`（`tshy && bash scripts/build.sh`）と test 系だけで、install の時に走る `preinstall`・`install`・`postinstall` は無い（`prepare` は registry の tarball の install では走らず、`ignore-scripts=true` でも止まる）。minimatch 11.0.4・10.2.5 とも npm の provenance の attestation は無い（`npm view minimatch@<v> dist.attestations` が空）。
- 使い方（隔離した copy の `node_modules/@typescript-eslint/typescript-estree/dist` を `grep -rn minimatch`）: `useProgramFromProjectService.js` の先頭の `require("minimatch")`（:41）と、`filePathMatchedBy` の `(0, minimatch_1.minimatch)(filePath, pattern, { dot: true })`（:204、`allowDefaultProject` の照合）だけ。`parser.js` が `useProgramFromProjectService` を `require` する（:22）ので、**minimatch は lint のたびに CJS の `require` で読み込まれる**（呼び出されるのは `projectService` の `allowDefaultProject` の照合だけ）。この repo の `eslint.config.js` は `parserOptions.project`（tsconfig 2 つ）で、`projectService`・`allowDefaultProject` を使わない（`git grep -n "projectService\|allowDefaultProject"` が 0 件）。呼出しの通り道は通らないが、読込みの口（CJS の `require` と named export の `minimatch`）は毎回通る。
- 案 (a) の起票の時の試し（隔離した copy だけ。`package.json` に `"overrides": { "@typescript-eslint/typescript-estree": { "minimatch": "11.0.4" } }` を足して `npm install --ignore-scripts`。版を限らない key と、推移的な依存を pin しない形で、Scope の形は下の「組を固定する仕組みの試し」で確かめ直した）:
  - `added 1 package, changed 1 package, and audited 514 packages`、`found 0 vulnerabilities`。
  - `npm ls minimatch @isaacs/nonrecursive-glob-match brace-expansion balanced-match --all`: `@typescript-eslint/typescript-estree@8.58.2` の下が `minimatch@11.0.4 overridden`（その下に `@isaacs/nonrecursive-glob-match@1.0.3`・`brace-expansion@5.0.12`・`balanced-match@4.0.4`）。eslint 系の `minimatch@3.1.5`・`brace-expansion@1.1.21`・`balanced-match@1.0.2` は変わらない。
  - `git diff package-lock.json`: `packages` の変更は 2 つだけ。`node_modules/@isaacs/nonrecursive-glob-match`（1.0.3）の追加と、`node_modules/@typescript-eslint/typescript-estree/node_modules/minimatch` の 10.2.5→11.0.4（resolved・integrity、dependencies に `@isaacs/nonrecursive-glob-match` `^1.0.3`、`brace-expansion` `^5.0.5`→`^5.0.12`、engines `18 || 20 || >=22`→`20 || >=22`）。lockfile の root（`packages[""]`）に `overrides` の記録は入らない（npm 12.0.2）。
  - `npm audit` exit 0（`found 0 vulnerabilities`）。
  - CJS の口: `node -e` で nested の minimatch を `require` すると `typeof m.minimatch` が `function`、`require.resolve("minimatch", { paths: [<typescript-estree>] })` が `.../typescript-estree/node_modules/minimatch/dist/commonjs/index.js`。
  - その後の `npm ci --ignore-scripts` exit 0、lockfile の差分は変わらない（`git diff --stat` が同じ 18 行）。
  - `npm run generate:routes`・`npm run typecheck`・`npm run format:check`（`All matched files use Prettier code style!`）・`npm run build`（`✓ built`）が exit 0。`npx --no-install vitest run` が exit 0（`Test Files  195 passed (195)`・`Tests  2127 passed (2127)`）。`npx --no-install eslint . -f json` が exit 0（495 file、error 0・warning 0）。
  - lint の比べ: 上の override の版と、override を外して `npm ci --ignore-scripts` した版で `eslint . -f json` を流し、file ごとの（path・errorCount・warningCount・messages の数）の列が一致（`identical True`、どちらも 495 file）。
  - `dist/` に `minimatch`・`nonrecursive-glob-match` の文字列は無い（`grep -rl`。製品の bundle に入らない）。
  - `allowDefaultProject` の通り道の probe（repo の外の scratchpad の `probe.cjs`。copy の中に一時の dir を作り `parseAndGenerateServices` を `projectService: { allowDefaultProject: ["*.ts", "*/dot.ts"] }` で 4 file に当て、dir を消す）: minimatch 11.0.4 と 10.2.5 で出力が同じ。`src/in.ts`（tsconfig の中）・`outside.ts`（`*.ts` に当たる）・`.hidden/dot.ts`（`*/dot.ts` に `dot: true` で当たる）は `ok:program`、`nomatch.mts`（どれにも当たらない）は `... was not found by the project service.`。
- 組の選び方の記録（計画の review round 1 の P2 の是正、2026-11-12 15:04〜15:12Z に実測）: `min-release-age=7` は exact の版（7 日に届かない版は `ETARGET`。round 1 の reviewer の実測）にも、推移的な依存の新しい解決にも効く。15:04Z の時点で入れられる組は 11.0.4 + `@isaacs/nonrecursive-glob-match` 1.0.3 だけ（1.1.0 は 11-12T21:38Z、11.0.5〈`^1.1.0`〉は 11-12T23:30Z、11.0.6 は 11-17T16:35Z、11.0.7 は 11-17T23:03Z、11.0.8 は 11-18T09:50Z から入る。各 publish の時刻 + 7 日）。この組に決めた（下の Scope）。理由: 今入れられ、調べ・試し・probe・lint の比べを全部この組で回した。古い組は時刻が進んでも入れられなくならない。
  - 組の現物（`npm view minimatch@11.0.4 dependencies engines scripts --json`、`npm view @isaacs/nonrecursive-glob-match@1.0.3 scripts dependencies license dist.attestations --json`）: minimatch 11.0.4 は publish 2026-11-02T04:50Z、dependencies `brace-expansion` `^5.0.12`・`@isaacs/nonrecursive-glob-match` `^1.0.3`、engines `20 || >=22`、`preinstall`・`install`・`postinstall` なし。nonrecursive-glob-match 1.0.3 は publish 2026-10-28T19:50Z、dependencies なし、license `BlueOak-1.0.0`、scripts は `prepare`・test 系だけ、attestation なし。
  - 変更の中身（`gh api repos/isaacs/minimatch/contents/changelog.md` と `gh api repos/isaacs/minimatch/compare/<tag>...<tag>` の commit の件名、read だけ）: changelog は `## 11.0` の 1 節で、11.0 の破壊的な変更は `makeRe()`・`Minimatch#regexp`・`AST`・`debug`・`matchOne()` などの削除と、照合を `@isaacs/nonrecursive-glob-match` の `Matcher` に替えたこと（typescript-estree が使う `minimatch()` の関数は残る）。v11.0.3...v11.0.4 は「drop support for node 18」だけ。**11.0.4 の後の版の直し**: 11.0.5 は glob split の上限（`bound total cells added by the glob split pass`・`reject incomplete glob split expansions`・`**/..` の展開の上限）と大きな数の brace range、11.0.6 は cache と OIDC の release（11.0.6 から provenance の attestation がある）、11.0.7 は `Quadratic compile time for long wildcard patterns (regression in 11.0.0 mergeRows)`、11.0.8 は cache の重みと escape の直し。どれも advisory は無い（上の `gh api '/advisories...'`）。11.0.4 には 11.0.7 が直した長い wildcard の pattern の compile が二次の時間になる退行が残るが、この repo では minimatch の照合を呼ばない（上の「使い方」）うえ、pattern は repo の設定からしか来ないので、受ける道が無い（下の「案の比較」(a) の (vii)）。
  - 組を固定する仕組みの試し（隔離した copy `$TMPDIR/mm-854-wt`、`npm ci --ignore-scripts` の後、Scope の `overrides` の block を書いて `npm install --ignore-scripts --save-exact`。調べの後に消した）: exit 0、`found 0 vulnerabilities`。`npm ls minimatch @isaacs/nonrecursive-glob-match --all` は `@typescript-eslint/typescript-estree@8.58.2 overridden` → `minimatch@11.0.4 overridden` → `@isaacs/nonrecursive-glob-match@1.0.3 overridden`、eslint 系は `minimatch@3.1.5` のまま。lockfile の差分は上の試しと同じ 2 entry（18 行。`@isaacs/nonrecursive-glob-match` 1.0.3 の `resolved` `https://registry.npmjs.org/@isaacs/nonrecursive-glob-match/-/nonrecursive-glob-match-1.0.3.tgz`・`integrity` `sha512-SI7kyKHq…`、minimatch 11.0.4 の `resolved` `https://registry.npmjs.org/minimatch/-/minimatch-11.0.4.tgz`・`integrity` `sha512-/UJ1dEoV…`）。npm 12.0.2 の lockfile には `overrides` の記録が入らない。
    - 推移的な依存の pin が効くこと: 同じ block で `@isaacs/nonrecursive-glob-match` を `1.0.2`（minimatch の宣言 `^1.0.3` の外）にすると `@isaacs/nonrecursive-glob-match@1.0.2 overridden` になった（override の値が親の宣言より強い）。1.1.0 が入れられる時刻の後も、この pin があれば 1.0.3 になる。
    - 版を限った key: key を `@typescript-eslint/typescript-estree@8.58.1`（今の親と違う版）にすると override が外れ、`npm install` の後の nested が `minimatch@10.2.6`、`npm audit` が exit 1 に戻った。
    - block を元に戻して `npm install --ignore-scripts --save-exact` をし直すと lockfile が byte まで同じ（`cmp`）。その後の `npm ci --ignore-scripts` exit 0 で lockfile は変わらない。
    - gate: `npm run generate:routes` の後に `npm run typecheck`・`npm run lint`・`npm run format:check`・`npx --no-install vitest run`（`195 passed`・`2127 passed`）・`npm run build` がどれも exit 0。`npx --no-install eslint . -f json` は 495 file、error 0・warning 0。上の probe は 4 file とも起票の時と同じ結果。
    - `brace-expansion` 5.0.12 は今の lockfile にもう在り、11.0.4 の `^5.0.12` を満たすので、install は動かさない（lockfile の差分に出ない）。pin しない。

## 案の比較

- **(a) 採る: `package.json` の `overrides` で `@typescript-eslint/typescript-estree@8.58.2` の下の `minimatch` だけを 11.0.4 に、その下の `@isaacs/nonrecursive-glob-match` を 1.0.3 に、どちらも exact で固定する。**
  - 効く範囲: override の key を `@typescript-eslint/typescript-estree@8.58.2` の下に置くので、eslint 系の `minimatch@3.1.5` は動かない。key が親の版を限るので、親を上げると override は効かなくなる（上の試しで確かめた）。
  - 危険: 上流の宣言 `^10.2.2` の外へ major を上げる。(i) API: 使う呼出しは `minimatch(path, pattern, { dot: true })` の 1 か所で、named export の `minimatch` は 11 の CJS の口にもある（試しで `function`）。(ii) CJS の口: typescript-estree は CJS で、11 は `"type": "module"` だが `exports.require` の口があり、上の `require.resolve` が `dist/commonjs` を指す。読込みは lint のたびに通るので、口の欠けは lint の起動で必ず落ちる（黙って変わらない）。(iii) engines: 11.0.4 以降の `20 || >=22` を repo の Node 24 は満たす。(iv) 呼出しの通り道（`allowDefaultProject`）はこの repo では通らない見込み（`eslint.config.js` が `project` を使う。上の probe で、通ったときも 10.2.5 と同じ結果）。(v) 推移的な依存が 1 つ増える（`@isaacs/nonrecursive-glob-match`、初の publish が 2026-10-26 の新しい package。同じ maintainer、依存なし、install の script なし、dev だけ）。(vi) override は上流の宣言を覆う。key を親の版に限るので、覆うのは 8.58.2 の間だけで、親を上げる lane で外れる（まだ影響の範囲の版に戻れば `npm audit` が red になって見える。上の試し）。(vii) 11.0.4 は最新でない（11.0.5〜11.0.8 の直しを持たない。上の「変更の中身」）。直しのどれも advisory が無く、照合を呼ばないこの repo では受ける道が無いので、今入れられる組を採る。新しい組へ替えるのは別の lane か Gated Amendment。(viii) minimatch と nonrecursive-glob-match の code は lint の process の中で `require` で毎回走る（`ignore-scripts` は install の時の script を止めるだけ）。dev の lint の道具の推移的な依存として受ける危険で、Ready の承認を求めるときに 2 つの package を名指して見せる。
  - 理由: 今日直せる。lockfile の差分が 2 つの entry で、確かめやすい。`typescript-eslint` の大きな更新（8.58.2→8.72.x）の lint の規則の変化を持ち込まない。上流の canary がもう `^11.0.1` なので、同じ major を先に入れるだけで、上流が 11 に上げたときに override を外しても版の向きが変わらない。
- **(b) 見送る: 上流が minimatch 11 に上げるのを待つ。** 上流の stable はまだ `^10.2.2`。いつ stable に出るかは分からない（`8.72.3` の stable の publish の日は未確認。出ても `min-release-age=7` で 7 日待つ）。待つ間 alert 30 は open のまま。owner の決定 TD-657 は alert 30 を lane にすることなので、今の手で閉じる (a) を採る。
- **(c) 見送る（override を外す道として残す）: `typescript-eslint` を名指しで上げる。** latest の 8.72.2 も `^10.2.2` で、今は解けない（`npm audit` も `@typescript-eslint/typescript-estree 6.16.0 - 8.72.2` を影響の範囲と出す）。`^11.0.1` の stable が出て 7 日を過ぎれば解けるが、8.58.2 から 14 minor 上がり、`typescript-eslint` の 6 package が全部動き、`strictTypeChecked`・`stylisticTypeChecked` の規則の変化を見る review が要る（範囲が大きい）。その更新の lane で override を外す。
- **(c') 見送る: `npm audit fix --force`。** `typescript-eslint@8.36.0` への downgrade で、AGENTS.md の npm 供給網ガードが禁止する。

## Scope

本 commit（plan-first）:

- 本 packet `docs/plans/2026-11-12-minimatch-override.md`。
- `docs/backlog.md` の保留の項目「`minimatch` の moderate の脆弱性（`typescript-eslint` 経由）」を、本 packet を引き、override を外す条件を持つ形に書き換える（「この変更で不要になるもの」）。

実装（計画の固定の後）:

- 名指した組: `minimatch` **11.0.4** と `@isaacs/nonrecursive-glob-match` **1.0.3**（選んだ時刻と理由は上の「組の選び方の記録」）。
- `package.json`: 末尾に次の 1 block を足す（他の key は変えない）。

  ```json
  "overrides": {
    "@typescript-eslint/typescript-estree@8.58.2": {
      "minimatch": {
        ".": "11.0.4",
        "@isaacs/nonrecursive-glob-match": "1.0.3"
      }
    }
  }
  ```

  key は親の版を限る（親を上げれば効かなくなる）。`"."` は minimatch 自身の版、その隣の key は minimatch の下の推移的な依存の版で、両方を exact にするので、install の時刻によらず同じ組になる（`@isaacs/nonrecursive-glob-match` 1.1.0 が入れられる時刻の後も 1.0.3。上の試し）。手順は `package.json` を Edit で書いてから `npm install --ignore-scripts --save-exact`（package を名指さない `install` は lockfile を `package.json` に合わせるだけ。`npm update` は使わない）。toolchain は D-066 の npm 12.0.2 と `.node-version` の Node 24.18.0。
- `package-lock.json`: 上の install の結果。差分は AC3 の 2 entry だけ。install が別の組・別の entry を出したら、直さずに止めて Coordinator に報告する（組を替えるのは Gated Amendment）。
- `docs/backlog.md`: 実装の merge で、上の項目を「override を外す条件」だけの項目に縮める（脆弱性の記述を済みにする）。

Non-scope: `.npmrc`、`eslint.config.js`、`typescript-eslint` を含む他の依存の版、`src/**`・`src-tauri/**`、CI の workflow、`docs/decision-log.md`（下の Design Readiness）、`docs/plans/2026-11-09-deps-security.md`（merge 済みの packet。非目的の行は当時の記録のまま）。

他の lane との重なり（2026-11-12、`gh pr list --state open` の 8 本の `gh pr diff N --name-only`）: `package.json`・`package-lock.json`・`.npmrc` に触れる open の PR は無い。`docs/backlog.md` には #216・#227・#228・#229・#235・#236 が触れるが、hunk は 75〜221 行の他の項目で、本 lane の項目（保留の節の先頭、153 行）と重ならない（`gh pr diff N` の backlog の `@@` の行）。

## Acceptance Criteria

- AC1（影響の範囲の版が残らない）: `npm ls minimatch @isaacs/nonrecursive-glob-match --all` の出力で、`@typescript-eslint/typescript-estree@8.58.2` の下が `minimatch@11.0.4 overridden`、その下が `@isaacs/nonrecursive-glob-match@1.0.3 overridden`、ほかの `minimatch` は `3.1.5` だけ。`7.0.0` 以上 `11.0.1` 未満の版が 1 つも無い。
- AC2（audit）: `npm audit` が exit 0 で `found 0 vulnerabilities`。`npm audit --audit-level=high` が exit 0。
- AC3（lockfile の差分）: `git diff origin/main -- package-lock.json` の変更が次の 2 entry と完全に一致し、ほかの行が動かない。(i) `node_modules/@typescript-eslint/typescript-estree/node_modules/minimatch` が `version` `11.0.4`・`resolved` `https://registry.npmjs.org/minimatch/-/minimatch-11.0.4.tgz`・`integrity` `sha512-/UJ1dEoVQTjPNDh6SGlUo+ueLMmoHTas4q+8yaAR/5t6nLAIiBDJtoYUh20IYleix7ehiQkgczb84PE3uzHkxg==`・dependencies `@isaacs/nonrecursive-glob-match` `^1.0.3` と `brace-expansion` `^5.0.12`・engines `20 || >=22`。(ii) 追加の `node_modules/@isaacs/nonrecursive-glob-match` が `version` `1.0.3`・`resolved` `https://registry.npmjs.org/@isaacs/nonrecursive-glob-match/-/nonrecursive-glob-match-1.0.3.tgz`・`integrity` `sha512-SI7kyKHq4XXDRozw+r9GlRZkia1ZkFrpLW98r2IC2Bpr8YkFGE3fvjzmxCG0x78A++69HmDNrDvSMYBZmHGpKA==`・`dev` `true`・`license` `BlueOak-1.0.0`。`package.json` の差分は Scope の `overrides` の block だけ。install が別の組・別の entry を出したら止めて Gated Amendment。
- AC4（再現の install）: `npm ci --ignore-scripts` が exit 0 で、その後の `git status --porcelain package.json package-lock.json` が空。hosted の CI（Node 24 同梱の npm 11 系、D-066）の install の step が同じ head で通る。
- AC5（gate と lint の挙動）: 実装の worktree で `npm run generate:routes` の後に `npm run typecheck`・`npm run lint`・`npm run format:check`・`npm test`（vitest の全体）・`npm run build` がどれも exit 0。lint は実装の前（`origin/main` の隔離した copy）と後の両側で、`npm run generate:routes` の後に `npx --no-install eslint . -f json` を流し、両側とも error 0・warning 0（Test Plan の T2。生成の前に流すと型の解決の error が出るので、両側とも生成の後に限る）。既存の test を skip・削除・弱体化しない。
- AC6（通り道の probe）: Test Plan の T3 の probe が実装の後の版で exit 0 になり、起票の時と同じ 4 つの結果（`src/in.ts`・`outside.ts`・`.hidden/dot.ts` が `ok:program`、`nomatch.mts` が `was not found by the project service`）を出す。
- AC7（docs）: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-11-12-minimatch-override.md` が ERROR なし。
- AC8（merge の後）: Dependabot alert 30 が `fixed`（owner の terminal か Coordinator の `gh api repos/<owner>/<repo>/dependabot/alerts/30 --jq .state`、read だけ）。

## Design Readiness

- 引用する設計正本（節まで）: `AGENTS.md` の npm 供給網ガード（D-030）、`docs/decision-log.md` D-029・D-030・D-066、`docs/DEV_WORKFLOW.md` の Risk Tiers（R2 の行き先の規則）と Verification Gates の Frontend の行。
- 必要な設計成果物: existing sufficient。製品の振舞い・契約・画面・データに触れず、npm 供給網ガードの各項目（下の Contract Ledger）の中で済む。
- plan にしかない durable な判断の昇格先: 「override を外す条件」→ `docs/backlog.md` の項目（本 commit で書き、merge の後も残る）。`overrides` を使うこと自体は D を置かない（1 package の 1 箇所の局所の手当てで、外す条件つき。npm の `overrides` を repo で一般に使う方針を決めるものではない）。今後 `overrides` が 2 つ目の package に要るなら、その lane で D-030 の補足の D を検討する。
- 前提・制約と延期した design gap: (i) `package.json` は JSON で comment を書けないので、`ponytail:` comment の代わりに backlog の項目が override の限界と外す条件を持つ。(ii) CI の npm 11 系が npm 12 の作った lockfile と `overrides` を同じに読むかは手元で確かめていない（未実測。AC4 の hosted の install で確かめる。D-066 の Revisit の「lockfile compatibility の反例」に当たれば、その時に扱う。round 1 の reviewer は npm 11.21.2 の `npm ci --ignore-scripts` が通ったと報告した）。(iii) 11.0.4 の後の版の直しは commit の件名までで読み、code は読んでいない（上の「変更の中身」）。
- 絶対保証の例外と抜け道: 「lint の挙動が変わらない」は AC5 の前後の両側の error 0・warning 0 で見る（両側とも診断が無いので、同じ数の診断の入替えの見逃しは起きない。基準が 0 でなくなったら、この見方は使えない）。「eslint 系の 3.1.5 は動かない」は AC1・AC3 で見る。「install の時刻によらず同じ組」は exact の 2 つの値（`"."` と推移的な依存）で保ち、AC3 の完全一致で見る。
- 計画の review の固定の問いへの先回りの答え:
  - (1) 普通の一日: 店の操作に関係しない（lint の道具だけ。製品の bundle に入らないことは上の調べの `dist/` の grep）。
  - (2) 登録と生成: command・bindings・route・REQ の追加は無い。`90-traceability.md` は変わらない（test を足さない）。
  - (3) 外の前提: minimatch 11 の CJS の口・engines・依存は registry の metadata と隔離した copy で確かめた（上の調べ）。CI の npm 11 系は (ii)。
  - (4) データの安全: 店の実データを使わない。probe は repo の外の scratchpad に置き、copy の中の一時の dir は probe が消す。
  - (5) 前提の出所: lane は owner の決定 TD-657、alert の事実は `gh api`、版と依存は `npm view`、木と gate は隔離した copy の実行の出力（上の調べ）。案の選択は AI（起票の Writer）の推奨で、owner の決定ではない。
- 判定: ready。外の前提は実測で確かめ、残る未実測（CI の npm 11 系）は AC4 で閉じる。組の 2 つの版はどちらも 7 日を十分に過ぎていて（11.0.4 は 2026-11-02、1.0.3 は 10-28）、`min-release-age` の境の秒に依らない。

この変更で不要になるもの（AGENTS.md「継ぎ足さない」）: (i) `docs/backlog.md` の保留の項目の「直すなら `typescript-eslint` を名指しで上げる」と「Dependabot の open の alert には無く」は今の事実と合わない（alert 30 が出て、上げても今は解けない）ので、本 commit で書き換える。(ii) 実装の merge の後、`overrides` の block は「親が 8.58.2 の間の橋」で、外す条件は「`typescript-eslint` を上げる lane で block を消す」だけ（key が親の版を限るので、親を上げれば block は効かなくなる。その lane で上げた先がまだ影響の範囲なら `npm audit` が red になって見える）。この条件は backlog の項目に置く。(iii) round 1 の是正で、規則だけの「版の選び方」の文（実装の日に選ぶ、Coordinator が引き直す）を消し、名指した組と「組の選び方の記録」に置き換えた。残すと決めたもの: `.npmrc` の 2 行（D-030）、eslint 系の `minimatch@3.1.5`（影響の範囲の外）。

## Contract Ledger

触れる製品の契約（REQ・SP・UI・BIZ・CMD・IO・MNT）は無い。隣の契約の sweep: `eslint.config.js` の `parserOptions.project`（変えない。Non-scope）、`docs/DEV_WORKFLOW.md` Verification Gates の Frontend の command（変えない。AC5 で回す）。npm 供給網ガードの各項目との合い方を行にする。

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| D-030 `ignore-scripts=true` | `AGENTS.md` npm 供給網ガード、`.npmrc` | `.npmrc` を変えない。install は `--ignore-scripts` | 非対象（AC4 の command と差分の review。追加の package に install の script が無いことは調べで確かめた） | 非対象 |
| D-030・D-029 `min-release-age=7` | 同上 | 名指した組（minimatch 11.0.4 は 2026-11-02、`@isaacs/nonrecursive-glob-match` 1.0.3 は 10-28 の publish で、どちらも 7 日を過ぎる）。`min-release-age-exclude[]` を使わない | 非対象（AC3 の版の完全一致） | 非対象 |
| D-030 package/version の名指し・exact・`--save-exact` | 同上 | `overrides` の 2 つの値を exact にし、install は `npm install --ignore-scripts --save-exact` | 非対象（AC3 の `package.json` の差分の review と実装の報告の command） | 非対象 |
| D-030 lockfile の差分の review | 同上 | `package-lock.json` の 2 entry | 非対象（AC3） | 非対象 |
| D-030 PR の前の `npm audit --audit-level=high` | 同上 | 実装の後に回す | 非対象（AC2） | 非対象 |
| D-030 禁止（`npm audit fix --force`・名指しでない `npm update` / `npm upgrade`・pin の無い `npx <package>`・install-script ガードの迂回） | 同上 | 使わない（`npx --no-install` は入れてある bin だけを使う） | 非対象（差分と実装の報告の command の review） | 非対象 |
| D-030 新規 runtime 依存の明示 | 同上 | runtime 依存は足さない。dev の推移的な依存 `@isaacs/nonrecursive-glob-match` を 1 つ足すことを本 packet と PR body に明示する | 非対象（AC3） | 非対象 |
| D-066 手元の npm 12・CI の npm 11 | `docs/decision-log.md` D-066 | 手元は npm 12.0.2 で lockfile を作る。CI は変えない | hosted の CI の install の step（AC4） | 非対象 |

## Test Plan

依存の更新で、製品の振舞いの test は足さない。lint の道具の挙動が変わらないことを、既存の gate と実装の前後の比べで見る。

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| GHSA-g3hv-75wg-m249 の範囲の版が残らない | override の key の誤り（例: top-level の `"minimatch"`、親の版の違い）で nested が 10.x のまま、または eslint 系の 3.1.5 まで 11 に上がる | CLI | T1: `npm ls minimatch @isaacs/nonrecursive-glob-match --all` と `npm audit`（AC1・AC2） | override を外す、key を `"minimatch"` の top-level にする（eslint 系が `overridden` になる）、key の版を親と違う値にする（起票の試しで `@8.58.1` にすると override が外れて `minimatch@10.2.6`、`npm audit` exit 1）、`"."` を 10.2.6 にする |
| 組が install の時刻によらない | 推移的な依存が install の時刻の新しい版（例 1.1.0）に解決され、評価していない組が入る | CLI | T1 の `npm ls` と AC3 の lockfile の完全一致 | `@isaacs/nonrecursive-glob-match` の pin を外す（1.1.0 が入れられる時刻の後なら 1.1.0 になる）、pin の値を変える（起票の試しで `1.0.2` にすると `1.0.2 overridden`） |
| lint の挙動 | minimatch 11 の CJS の口の欠け・API の違いで parser の読込みが落ちる、lint の結果が変わる | CLI | T2: `npm run lint` の exit 0 と、前後の両側で `npm run generate:routes` の後の `npx --no-install eslint . -f json` が error 0・warning 0（AC5） | nested の minimatch の `dist/commonjs` を壊す（`require` が落ちて lint が起動しない）、`eslint.config.js` の規則を変える、片側で生成を忘れる（型の解決の error が出て 0 にならない） |
| `allowDefaultProject` の通り道 | 照合（`minimatch(path, pattern, { dot: true })`）の結果が 11 で変わる | CLI（probe。repo の外の scratchpad の script を隔離した copy に当てる。commit しない） | T3: `parseAndGenerateServices` を `allowDefaultProject: ["*.ts", "*/dot.ts"]` で 4 file に当て、`src/in.ts`・`outside.ts`・`.hidden/dot.ts` が `ok:program`、`nomatch.mts` が `was not found by the project service`（AC6） | 照合が `dot: true` を無視する（`.hidden/dot.ts` が落ちる）、照合が常に true / false を返す（`nomatch.mts` が通る / `outside.ts` が落ちる） |
| 既存の gate | 依存の解決の変化で型の検査・test・build が落ちる | CLI（既存） | T4: `npm run typecheck`・`npm run format:check`・`npm test`・`npm run build`（AC5） | 追加の package が別の依存を巻き込む、lockfile が壊れる |
| 再現の install | lockfile と `package.json` の不一致、CI の npm 11 系との非互換 | CLI・hosted | T5: `npm ci --ignore-scripts` と hosted の install の step（AC4） | lockfile を手で直して `package.json` と食い違わせる、`overrides` の値と lockfile の版を食い違わせる |

T3 の probe の script は起票の時に scratchpad に書いて回した（調べの最後の項）。実装の Writer は同じ形の script を repo の外に書いて回し、出力を報告に貼る（repo に test として足さない。lint の設定が `projectService` を使わないので、常設の test にしても今の repo の振舞いを見張らない）。

## Open Questions

owner の判断が要る問いは無い見込み。lane にすることは TD-657 で決まり、残るのは手段の技術的な選択（Coordinator と計画の review が確かめる）。受容リスクとして owner に見える差は、dev の推移的な依存に新しい package `@isaacs/nonrecursive-glob-match`（初の publish 2026-10-26、minimatch と同じ maintainer、依存なし・install の script なし）が 1 つ入ることで、(a) を採る限り避けられない（minimatch 11.0.1 以降が全部持つ）。これを避けたいなら (b)（上流の stable を待ち、(c) で一緒に入る。そのときも同じ package が入る）になり、避ける手は無い。Coordinator が受容リスクの判断として owner に送るべきと見るなら、選択肢は (a) 今 override で閉じる【起票の推奨】／(b) 上流の stable と 7 日を待って (c) で閉じる。round 1 の 2 本の review はどちらも別の gate を足す根拠は無いとし、Ready の承認を求めるときに名指しの組（minimatch 11.0.4・`@isaacs/nonrecursive-glob-match` 1.0.3）と、lint の process の中で走る code であることを見せるのが妥当とした。
