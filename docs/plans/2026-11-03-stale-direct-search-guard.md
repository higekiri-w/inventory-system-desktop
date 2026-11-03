# Plan Packet: 作成画面 4 つ（廃棄・入庫・手動販売・返品交換）で古い直接検索の応答を捨てる（導入の前の関門 13）

wave に属さない単独の lane。導入の前の関門の 13（[Plans](../Plans.md#導入の前の関門)）。owner の決定 2026-11-03（repo 外の回答台帳 TD-586）で関門になった。出発点は `docs/backlog.md` の項目「作成画面 4 つの古い直接検索の応答の混入」と、それを確かめた 2026-11-03 の調べ（AI。repo 外の `.local/reports/stale-direct-search-4screens-2026-11-03.md`、再現の vitest の probe 4 本は `.local/reports/stale-direct-search-probes/`）。本 commit は Plan Packet の起票と plan-first の設計正本の修正（61・62・63・64・catalog ⑮・backlog・Plans。Scope の「設計正本の変更の一覧」）だけで、実装は計画の固定の後の別の run。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の R3 の「状態を持つ画面・operator の業務の流れ」に当たる: 4 つの作成画面の保存・リセット・「続けて」と非同期の商品検索の状態の扱いを変え、直らないと入庫・手動販売（売上と在庫）・返品交換の記録が二重にできる（下の「現状の調べ」）。R4 にしない理由: DB・migration・command の DTO・bindings・backup・店の実データに触れず、frontend の 4 画面の状態の扱いと test と docs だけを変える。

## Goal

Goal Invariant:

### 最小完了条件

- 4 画面とも、商品の直接検索（Enter・IME の数字確定・検索ボタン）を始めた後に、保存の開始・リセット・「続けて○○」のどれかがあれば、その検索の応答（成功の 1 件・複数件・0 件、失敗）は、明細・候補の一覧・案内の文のどれにも出ず、後の保存の payload に載らない。
- 入庫・手動販売・返品交換で、保存成功の後（「続けて」の前）に古い応答が届いても result panel が残り、フォームは操作不可のまま、create は 1 回だけ（同じ明細の記録が二重にできない）。
- 手動販売の PLU 確認待ちの間に保存の前の検索の応答が届いても、警告と token と明細が変わらず、「確認して保存」が同じ明細・同じ key・同じ token を送る。
- 上のどれの後でも、その後に始めた検索は今までどおり明細に入る（捨て過ぎない）。

### 失敗定義

- 上の経路のどれかで、古い応答の商品が明細に入る、古い候補の一覧・`該当する商品がありません`・検索の失敗の文が新しいフォームに出る。
- 保存成功の後に古い応答で result panel が消え、2 回目の create が別の key で送れる。
- PLU 確認待ちの間に古い応答で明細が変わり、「確認して保存」が `確認トークンが不正です` で止まる。
- リセット・保存の失敗・確認待ちの後に始めた新しい検索が捨てられる（世代の比べ違い・lock の外し忘れ）。
- 返品で渡しの行ができる（UI-03-D6 を破る）。

### 非目的

- live 候補（catalog ⑮ の suggest 層）の破棄条件の変更。共有の hook がもう守っている（D4。lock source の付け替えだけを行う）。
- 振替の画面（PR #216、79）・棚卸し（73）の検索。
- 4 画面の direct search の共有 hook への括り出し（下の「共有 hook か画面ごとか」）。
- 検索中の表示（loading の顔）を足すこと。§61.3 などの `searching_product` の「商品追加欄 loading」は今の build に無いが、本 lane の範囲の外。
- BIZ の冪等・PLU の token の検査の変更（最後の防御として今のまま）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 現状の調べ（2026-11-03、main `bb94b58f`。4 画面の file は `b5e8fa4c` から変わっていない）

- 廃棄 `src/features/disposal/DisposalPage.tsx`: `isFormLockedRef`（:138）を保存の click で true（:659）、`onError` で false（:183）、`resetForm` で false（:266）。`handleProductSearch` は応答の成功の側だけ `isFormLockedRef.current` を見る（:249）、`catch` は見ない（:260-261）。世代は無い。開始の検査は render の `isFormLocked`（:238）。live 候補は `isLocked: () => isFormLockedRef.current`（:194）。
- 入庫 `src/features/receiving/ReceivingPage.tsx`: lock の ref が無い。`addProduct` は検索を始めた render の `isFormLocked`（:229）を見る。`updateValues` は `setResult(null)`（:224）で result panel を消す。保存成功で key を新しくする（:180）。`onError` は lock を触らない（:184-188）。live 候補は `isLocked: () => isFormLocked`（:195）、保存の click は `invalidateAndClose()` を呼ぶ（:659）。
- 手動販売 `src/features/manual-sale/ManualSalePage.tsx`: 入庫と同じ形（`addProduct` :251、`setResult(null)` :247、key の更新 :201、live 候補 :216）。`needs_confirmation` の応答は `setConfirmation` と `setResult(null)` で確認待ちにする（:188-195）。`updateValues` は `confirmation`（:237）と `failedSignature`（:241）を、関数を作った render の値で見る。保存は `submitCurrentForm`（:303-307）。
- 返品交換 `src/features/return-exchange/ReturnExchangePage.tsx`: 入庫と同じ形（`addProduct` :318、`setResult(null)` :314、key の更新 :247、live 候補 :276）。1 件の応答は検索を始めた render の `effectiveAddDirection`（:272-273）で足す（:351）が、`addProduct` の updater は足す時点の `prev.returnType` を見て返品なら `"in"`（:324）。保存の click は :928-932。
- 再現（`.local/reports/stale-direct-search-probes/` の 4 本を一時に `src/features/*/` へ写して実行し、消した）: `npx vitest run src/features/disposal/DisposalPage.stale-probe.test.tsx src/features/receiving/ReceivingPage.stale-probe.test.tsx src/features/manual-sale/ManualSalePage.stale-probe.test.tsx src/features/return-exchange/ReturnExchangePage.stale-probe.test.tsx` → `Test Files  4 failed (4)` / `Tests  18 failed | 2 passed (20)`（PASS は廃棄の A・B）。入庫・手動販売・返品交換の A の出力: `{"adopted":true,"resultVisible":false,"createCalls":2,"keys":[<2 つの別の key>],...}` で、2 回目の items は 1 回目の明細に `LATE-001` を足したもの（3 画面とも）。A〜E の意味は Test Plan の S1〜S5。
- 調べの「未確認」を同じ方法で確かめた（一時の追加の probe。上と同じく実行の後に消した。`npx vitest run -t PROBE2 --silent=false ...` の出力）:
  - 廃棄で保存の前に始めた検索の応答が保存の失敗の後に届く → `adoptedAfterFailure= true`（今の lock だけでは、UI-05-D15 の理由の「保存開始時点から破棄する」が成り立たない）。
  - 廃棄でリセットの後に複数件の古い応答 → `staleCandidatesShown= true msg= true`（古い候補の一覧と `候補から…` の文が新しいフォームに出る）。
  - 手動販売で保存の前に始めた検索の応答が PLU 確認待ちの間に届く → `adopted= true confirmationStillShown= true buttonLabel= true`（明細に入るが token は捨てない）。BIZ は token を明細から作り直して照らす（`src-tauri/src/biz/inventory_service/manual_sale.rs:195-202`）ので、「確認して保存」は `確認トークンが不正です` で止まり、記録はできない（UI-04-D9 と合わないが、二重の記録にはならない）。
  - 返品交換で、交換・渡しで検索を始め、応答の前に種別を返品へ替える → payload の items は `[{"product_code":"LATE-001","direction":"in",...}]`（UI-03-D6 は保たれる）。交換・戻りで始め、応答の前に追加方向を渡しへ替える → 足した行の方向は `戻り（在庫+）`（Enter の時点の方向）。
- unmount・画面の移動は経路にならない（調べのコードを読んだ判断。React は unmount の後の set state を捨てる）。live 候補は `useProductAddSuggest.ts` が世代・入力値・lock で応答を照らす（:97-103、:113-117）ので当たらない。
- BIZ: 同じ key で中身の違う送信は `IdempotencyConflict`（`manual_sale.rs:115-131` ほか）。二重の記録は保存成功で key が新しくなった後にだけできる。
- 手本: PR #216（未 merge、`origin/agent/stock-transfer`）の `StockTransferPage.tsx` は lock の ref と `searchGenerationRef` を持ち、`isStale = () => generation !== searchGenerationRef.current || isFormLockedRef.current` を成功と `catch` の両方で照らし、`resetForm` で世代を進める。保存の開始では世代を進めない（lock だけ）。
- 新しい test の名と既存の test の衝突: 本 lane の新しい test file の名 `*.stale-search.test.tsx` は `src` に 0 件（`ls src/features/*/*.stale-search.test.tsx` が無い）。

## 共有 hook か画面ごとか（D-147）

画面ごとに書く（推奨、本 packet の Scope）。D-147 の順に当てた:

1. 要るか: 要る（関門 13、上の再現）。
2. 既にある helper・pattern: 廃棄の UI-05-D15 の lock ref と、#216 の世代の確かめがある。共有の hook は無い。pattern をそのまま 3 画面に移し、廃棄には世代を足す。1 画面の差分は ref 2 つと、保存の event・`resetForm`・`onError`・（手動販売は確認待ち）での更新と、検索の 2 か所の照らしで、数行の見込み（未実測）。
3. 共有 hook（例 `useDirectSearchGuard`）を採らない理由: (a) 防ぐ失敗は「どの event で世代を進め、lock を外すか」の画面ごとの結線の誤りで、hook にしても結線の test は画面ごとに要り、確かめやすさが上がらない。(b) catalog ⑮ D9 が「既存 5 画面の commit 経路コードの共通化・移動は行わない」と決めている。(c) #216 が未 merge で、5 画面目の手本を本 lane の依存にしない。
4. 広げる条件は実装の `ponytail:` comment に書く: 同じ gate を持つ画面が 6 つ（#216 を入れて）を超えるか、gate の条件が増えたら共有 hook を検討する。

#216 が採るか: 本 lane は #216 の merge に依らない（4 画面の page と 61〜64 に #216 は触れない。下の「他の lane との重なり」）。#216 は保存の開始で世代を進めないので、保存の前に始めた検索の応答が保存の失敗の後に届くと採る（本 lane の廃棄の probe と同じ形。二重の記録にはならない）。揃えるかは #216 の側の判断で、Coordinator が #216 の closure か merge の後の小さな lane に回すのを推奨する（本 lane では決めない）。

## owner の判断事項

製品の問いは無い見込み（既存の正本〈UI-02-D9・UI-04-D11・UI-03-D14・UI-05-D10 と「保存成功後はフォーム本文を操作不可」〉に合わせる是正）。1 つだけ、見える振舞いの差を candidate として挙げる（packet の起票の AI が出したもので、Human Gate の必須にしない）:

- **O1 保存を押す前に始めた検索**: (a) 保存の結果によらず捨てる。保存の失敗・PLU 確認待ちの後は検索欄の文字が残り、Enter で同じ検索をし直す【推奨】／(b) 保存の失敗・確認待ちで lock が外れた後に届いた応答は採る（#216 の形）。理由: (a) は UI-05-D15 の理由の「保存開始時点から破棄する」と同じで、手動販売の確認待ちの token の食い違い（上の調べ）と、応答を採る `updateValues` が古い render の `failedSignature`・`confirmation` を見る経路を閉じる。(b) は確認待ちの経路を別の条件で閉じる手当てが要る。見える差は、保存を押す直前に読んだ商品が、保存に失敗したときに入らず、もう一度 Enter が要ること（ローカルの SQLite の検索は短く、起きる窓は狭い。窓の長さは未実測）。変わる所: (b) なら 4 画面の正本の「世代 ref は保存 event の先頭と reset event で進める」から保存 event を外し、手動販売に確認待ちの別の手当てを足し、S6・M2 を作り直す。

## Scope

本 commit で変えた正本（plan-first、Design Phase Rules。新しい契約には「未実装」と書き、実装の PR で外す）:

| 正本 | 節 | 変更 |
|---|---|---|
| 64 | UI-05-D15 | 検索の世代（保存 event の先頭と reset event で進め、直接検索の成功・失敗の応答の両方で、件数によらず照らす）を足した。理由の段に世代を足す理由と不採用の案。 |
| 64 | §64.3・§64.6・§64.9・更新履歴 | 世代の 1 段、Error / Recovery の 2 行、Test Focus の UI-05-D15、履歴の行。 |
| 61 | 新設 UI-02-D19（`rg -n "UI-02-D19" docs src` が本 commit の前に 0 件） | 直接検索の応答の採否（lock ref と世代。どの event で lock・unlock・世代を進めるか、件数と成功・失敗によらず捨てる、足す handler は lock ref を読む、UI-02-D18 の `getStockDetail` も実装の時に従う）。理由に二重の記録の経路と不採用の案（照らし合わせ・共有 hook）。 |
| 61 | UI-02-D14 | lock source を「既存 `isFormLocked` 派生 state」から UI-02-D19 の lock ref に改めた（未実装）。 |
| 61 | §61.3・§61.6・§61.9・更新履歴 | 状態の段、Error / Recovery の行、Test Focus の UI-02-D19、履歴の行。 |
| 62 | 新設 UI-04-D19（0 件を確かめた） | UI-02-D19 と同じ形に、`needs_confirmation`（token が null の失敗を含む）で unlock して世代は戻さない、確認待ちの間に始めた検索は採って UI-04-D9 のとおり token を捨てる、を足した。 |
| 62 | UI-04-D16・§62.3・§62.6・§62.9・更新履歴 | lock source、状態の段、Error / Recovery の行、Test Focus、履歴。 |
| 63 | 新設 UI-03-D24（PR #227 が UI-03-D23 を足すので D24。#227 が merge されなくても欠番のまま D24 を使う） | UI-02-D19 と同じ形に、保存 event が画像の保存を含むこと、交換の方向は Enter の時点の追加方向で、足す時点の種別が返品なら戻り、応答を待つ間の種別・方向の切替えでは世代を進めない、を足した。 |
| 63 | UI-03-D21・§63.3・§63.6・§63.9・更新履歴 | lock source、状態の段、Error / Recovery の行、Test Focus、履歴。 |
| catalog ⑮ | 契約の見出しの注・D4・D10・更新履歴 | lock source を取引 4 画面の form lock ref に（取引 3 画面は未実装）。直接検索の世代は各画面の ID が持つと D4 に 1 文。 |
| backlog | 「作成画面 4 つの古い直接検索の応答の混入」 | 本 packet を引く 1 文。 |
| Plans | 関門 13 | 本 packet を引く 1 文（完了の行は足さない。D-145 (12)）。 |

実装（計画の固定の後）:

- 4 画面の page（下は共通。画面ごとの差は括弧）:
  - `isFormLockedRef`（廃棄は既存）と `searchGenerationRef` を持つ。
  - 保存の event（廃棄・入庫・返品交換は保存の button の `onClick`、手動販売は `submitCurrentForm`〈「手動販売を保存」と「確認して保存」〉）の先頭で、`mutate` より前に lock を true にして世代を 1 進める。既存の `invalidateAndClose()` はそのまま同じ event で呼ぶ。
  - `onError` で lock を false（廃棄は既存）。手動販売は `onSuccess` の `needs_confirmation` の枝（`confirmation_token` が null の枝を含む）でも false。世代は戻さない。
  - `resetForm`（リセットと「続けて○○」）で lock を false にして世代を 1 進める（廃棄は lock が既存）。
  - `handleProductSearch`: 開始の検査を `isFormLockedRef.current` にする。開始の時点の世代を持ち、`isStale = () => generation !== searchGenerationRef.current || isFormLockedRef.current` を `await` の後の成功の側の先頭（0 件・1 件・複数件の分岐の前）と `catch` の先頭で照らし、stale なら何も set しない。
  - `addProduct` の先頭の検査を `isFormLockedRef.current` にする（入庫・手動販売・返品交換。廃棄は既存）。
  - `useProductAddSuggest` の `isLocked` を `() => isFormLockedRef.current` にする（入庫・手動販売・返品交換）。
  - 返品交換の方向の扱い（`effectiveAddDirection` を検索の開始で持ち、`addProduct` の updater が `prev.returnType` を見る）は変えない（UI-03-D24 が今の振舞いを契約にした）。
  - `ponytail:` comment（「共有 hook か画面ごとか」の 4）を各画面の世代の ref の所に 1 つ。
  - render の `isFormLocked`（disabled・離脱の guard・result の表示）は残す。
- test（新、画面ごとに 1 file。種は `.local/reports/stale-direct-search-probes/` の 4 本。`console.log` と `if` の分岐を外し、assert だけにする。test の名に REQ と UI の ID を付ける）: `src/features/disposal/DisposalPage.stale-search.test.tsx`、`src/features/receiving/ReceivingPage.stale-search.test.tsx`、`src/features/manual-sale/ManualSalePage.stale-search.test.tsx`、`src/features/return-exchange/ReturnExchangePage.stale-search.test.tsx`。中身は Test Plan。
- 生成物: `docs/function-design/90-traceability.md`（`cd src-tauri && cargo run --bin generate_traceability`。手で直さない。新しい test file が REQ-201〜204 の行に入る）。
- docs（実装の PR）: 61 UI-02-D19・D14、62 UI-04-D19・D16、63 UI-03-D24・D21、64 UI-05-D15、catalog ⑮ の注・D4・D10、各 §n.3・§n.6・§n.9 から本 lane の「未実装」を外し、履歴に実装の行を足す。backlog の項目を済みにし、Plans の関門 13 に「→ 済み: PR #N」を書く（gate 8 と同じ形）。
- 変えない既存 test: `DisposalPage.test.tsx` の UI-05-D15 の 5 本（`unlocks product search directly after a successful reset`・`ignores late product search results after the form is locked`・`locks late product results at the save event before the pending render`・`unlocks product search after frontend validation failure`・`unlocks product search after command failure`）、4 画面の `*.suggest.test.tsx`・`*.unsaved-guard.test.tsx`・`*.test.tsx`。弱めない。

Non-scope: `src/components/patterns/useProductAddSuggest.ts`・`ProductAddSuggest.tsx`（変えない）、`src/features/stock-transfer/`（#216）、`src/features/stocktake/`（調べていない。棚卸しの検索は `useFindStocktakeItem` で別の形）、BIZ・CMD・bindings、`src/lib/invalidation-contract.ts`、返品交換の画像の選択・削除の handler（同期の event で、非同期の応答の採否でない）、55/56/57/34/24/37/45（他の lane）。

他の lane との重なり（2026-11-03、`gh pr diff N --name-only` の目視）:

- PR #227（返品交換の `register_processed`、L3 待ち）: `docs/function-design/63-ui-return-exchange.md`・`docs/design-system/02-component-catalog.md`・`src/features/return-exchange/ReturnExchangePage.tsx`・`ReturnExchangePage.test.tsx`・`ReturnExchangePage.suggest.test.tsx`・`ReturnExchangePage.unsaved-guard.test.tsx`・`90-traceability.md`・backlog に触れる。63: #227 は UI-03-D8・D15・D17・D18 の行・D23 の新設・§63.0・§63.3 の state の表と form state の段・§63.5・§63.6 の新しい行・§63.7・§63.9・更新履歴。本 lane は UI-03-D21 の lock source・D22 の後の D24・§63.3 の form state の段の前の新しい段・§63.6 の末尾・§63.9 の末尾・更新履歴。末尾の追記と履歴の表は merge 順で両方を残す。§63.3 の段は隣り合うので、後に入る側が同期で手で解く見込み。catalog: #227 は ⑬ の除外の段だけで、⑮ と重ならない。`ReturnExchangePage.tsx`: #227 の hunk は form の型・レジ戻しの欄・保存の分岐（:246-259 の `onSuccess` の付近を含む）で、本 lane は ref・`handleProductSearch`・`addProduct`・`resetForm`・保存の `onClick`・`onError`。`onSuccess`/`onError` の付近で衝突しうる。実装は #227 の merge を待たない。後に入る側が単段の同期で解き、衝突の解消が runtime に及べば DEV_WORKFLOW の Stacked train の規則で Final Reviewer が差分を確かめ直す。新しい test file は別の名で衝突しない。
- PR #216（振替）: 4 画面の page・61〜64・catalog に触れない（`git diff origin/main origin/agent/stock-transfer --name-only` に無い）。`90-traceability.md` だけが重なる（生成物は取り込んだ後に再生成する、D-098）。
- PR #228・#229・#235・#236・#194: `docs/backlog.md`・`docs/Plans.md`（#229）・`90-traceability.md` だけ。backlog・Plans は別の項目の行で、生成物は再生成。

## Acceptance Criteria

- AC1（再現の閉じ）: 4 画面の `*.stale-search.test.tsx` の S1〜S5（調べの A〜E と同じ経路）が PASS。実装の前の main では入庫・手動販売・返品交換の S1〜S5 と廃棄の S3〜S5 が red（上の調べの probe の出力と同じ）。
- AC2（二重の記録が無い）: 入庫・手動販売・返品交換の S1 で、古い応答の後も result panel（`○○を保存しました`）が在り、保存の button が disabled、`createReceiving` / `createManualSale` / `createReturn` の呼出しが 1 回。
- AC3（未確認だった経路）: S6（保存の失敗の後）・S7（複数件・0 件）・S8（新しい検索は採る）が 4 画面の `*.stale-search.test.tsx` で、MS1・MS2（確認待ち）が `src/features/manual-sale/ManualSalePage.stale-search.test.tsx` で、RX1・RX2（方向）が `src/features/return-exchange/ReturnExchangePage.stale-search.test.tsx` で PASS（`npx vitest run <file>` の exit 0）。
- AC4（lock source）: `rg -n "isLocked: \(\) => isFormLocked\b" src/features` が 0 件（4 画面とも ref を読む）。`rg -n "isFormLockedRef" src/features/{receiving,manual-sale,return-exchange}` が各画面で、保存の event・`onError`・`resetForm`・`handleProductSearch`・`addProduct`・suggest の `isLocked` の所に当たる（手動販売は `needs_confirmation` の枝にも）。
- AC5（既存の振舞い）: 4 画面の既存の page・suggest・unsaved-guard の test が assert を変えずに PASS（廃棄の UI-05-D15 の 5 本を含む）。
- AC6（docs）: 実装の PR で `rg -n "関門 13 の lane" docs/function-design/6[1-4]-*.md docs/design-system/02-component-catalog.md` の当たりが履歴の行だけ（本 lane の「未実装」が残らない）。
- AC7（gate）: `bash scripts/doc-consistency-check.sh` と `--target plan docs/plans/2026-11-03-stale-direct-search-guard.md` が ERROR なし。Frontend の gate（`npm run typecheck && npm run lint && npm run format:check && npm test && npm run build`）と traceability（`cd src-tauri && cargo run --bin generate_traceability -- --check`）が通る（DEV_WORKFLOW Verification Gates）。

## Design Readiness

- 引用する設計正本（節まで）: 61 UI-02-D4・D5・D8・D9・D10・D14・D18・D19（新）・§61.3・§61.5・§61.6・§61.9、62 UI-04-D4・D5・D8・D9・D10・D11・D12・D16・D19（新）・§62.3・§62.5・§62.6・§62.9、63 UI-03-D6・D9・D10・D13・D14・D15・D21・D24（新）・§63.3・§63.5・§63.6・§63.9、64 UI-05-D5・D6・D10・D11・D15・D16・§64.3・§64.5・§64.6・§64.9、catalog ⑮ D1・D4・D9・D10・D11、AGENTS.md D-127・D-147。
- 必要な設計成果物: updated in this PR（plan-first: Scope の表）。decision-log の新しい D は置かない（4 画面の局所の判断で、UI-05-D15 の既存の型を移すだけ。横断の新しい選択は無い）。
- plan にしかない durable な判断の昇格先（本 commit で昇格した）: 世代を保存の開始でも進める理由と不採用の案 → UI-05-D15・UI-02-D19 の理由。共有 hook を採らない理由 → UI-02-D19 の不採用。PLU 確認待ちの扱い → UI-04-D19。交換の方向 → UI-03-D24。lock source → D14・D16・D21・catalog ⑮ D4・D10。packet にだけ残るのは O1 の記録と「共有 hook か画面ごとか」の詳しい理由（正本は UI-02-D19 の不採用の文）。
- 前提・制約と延期した design gap: (i) `searching_product` の loading の表示は §n.3 にあるが今の build に無い（範囲の外、上の非目的）。(ii) 棚卸し・振替は別。(iii) UI-02-D18 の `getStockDetail` は未実装で、UI-02-D19 に従うと書いた（D-138 の runtime の lane が持つ）。
- 絶対保証の例外と抜け道: 「保存の開始・リセット・続けての前に始めた検索の応答は反映しない」は、`handleProductSearch` の中の 2 か所の照らしで保つ。抜け道: (a) 応答の後の `addProduct` の中の `window.setTimeout` の focus は照らしの後で、反映しない応答では呼ばれない。(b) 世代は保存の開始で進むので、`mutationFn` の validation の失敗（command を呼ばない）でも保存の前の検索は捨てる（O1 の (a) の範囲）。(c) 交換の方向の切替えは世代を進めない（UI-03-D24。足した行は方向の列で見える）。(d) live 候補は suggest 層の D4 が持ち、本 lane の世代の外（lock source だけを共有）。
- 計画の review の固定の問いへの先回りの答え:
  - (1) 普通の一日: 成立。初期状態: 入庫の画面、明細 1 行 → 商品を Enter で検索 → 応答の前に「入庫を記録」→ 保存成功の result panel → 遅れた応答は捨てられ result panel が残る → 「続けて入庫」→ 空のフォームで新しい検索は入る → 保存は 1 回分の記録。翌日の起動: ref は画面の mount ごとに 0 と false から始まり、持ち越しは無い。再試行: 保存の失敗の後、同じ内容の再送は同じ key（UI-02-D8 のまま）、保存の前の検索は検索欄の文字で Enter をし直す。危険な結果を出さないか: 二重の記録は S1 で、確認待ちの token の食い違いは MS1 で見る。
  - (2) 登録と生成: command・bindings・route・navigation・REQ の追加は無い。新しい test file 4 本が `90-traceability.md` の再生成に入る（Scope）。新しい function-design の file は無い。
  - (3) 外の前提: TanStack Query の `mutate` が `mutationFn` を pending の render より前に呼ぶこと（L1 の test が依る。廃棄の既存の test `locks late product results at the save event before the pending render` が同じ前提で PASS している）。Windows WebView2 の IPC の遅さは窓の長さを変えるだけで、照らしの論理は変えない。
  - (4) データの安全: 店の実データを使わない。test の値は合成（`makeMockProductWithRelations`）。probe の写しは repo の外の `.local/` にあり commit しない。
  - (5) 前提の出所: 関門と範囲は owner の決定 TD-586（台帳）。経路の事実は上の「現状の調べ」の file:行と probe の実行の出力（本 packet の起票の時に再実行した）。O1 は AI の候補で、owner の決定ではない。
- この変更で不要になる記述・条件: (i) 61 UI-02-D14・62 UI-04-D16・63 UI-03-D21・catalog ⑮ D4・D10 の「取引 3 画面は既存 `isFormLocked` 派生 state」（lock ref に置き換えた）。(ii) code の、検索の開始・`addProduct`・suggest の `isLocked` の render の `isFormLocked` の検査（ref に置き換える）。(iii) catalog ⑮ の注の「D1〜D10 本文不変」は D4・D10 で成り立たなくなったので、改めた旨を同じ注に足した（凍結の記録として消さない）。(iv) backlog の項目と Plans の関門 13 は実装の merge で済みにする。残すと決めたもの: render の `isFormLocked`（disabled・離脱の guard・result の表示に要る）、廃棄の既存の UI-05-D15 の test 5 本（世代を足しても意味が変わらず、lock の event の境を見張る）、BIZ の冪等と token の検査（最後の防御）。
- 判定: ready。O1 は既存の UI-05-D15 の意図に合わせた推奨の既定で書いており、owner が (b) を選べば Gated Amendment で直す（Human Gate の必須にしない）。ほかに未解決の設計の問いは無い。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| UI-05-D15（世代を足した） | 64 UI-05-D15・§64.3・§64.6 | `DisposalPage.tsx` の世代 ref・保存の event・`resetForm`・`handleProductSearch` | DP の S1〜S8・L1、既存の UI-05-D15 の 5 本 | 非対象（下の「目視を置かない理由」） |
| UI-05-D16（lock source、変えない） | 64 UI-05-D16 | 変えない | 既存の `DisposalPage.suggest.test.tsx` | 非対象 |
| UI-05-D10 | 64 UI-05-D10 | 変えない（render の `isFormLocked`） | 既存の `hides product registration recovery and locks editing while saving` | 非対象 |
| UI-02-D19（新） | 61 UI-02-D19・§61.3・§61.6 | `ReceivingPage.tsx` | RC の S1〜S8・L1 | 非対象 |
| UI-02-D14（lock source） | 61 UI-02-D14 | suggest の `isLocked` を ref に | AC4 の `rg`、既存の `ReceivingPage.suggest.test.tsx` | 非対象 |
| UI-02-D9・§61.5（保存中・保存成功の後の操作不可） | 61 UI-02-D9・§61.5 | ref の lock | RC の S1・S2 | 非対象 |
| UI-02-D8（key） | 61 UI-02-D8 | 変えない | RC の S1（create 1 回）、既存の key の test | 非対象 |
| UI-02-D4・D5（0・1・複数件、focus） | 61 UI-02-D4・D5 | 照らしの後の分岐は変えない | RC の S8、既存の test | 非対象 |
| UI-02-D18（未実装、gate に従う） | 61 UI-02-D18・D19 | 本 lane では実装しない | — | 非対象（D-138 の runtime の lane） |
| UI-04-D19（新） | 62 UI-04-D19・§62.3・§62.6 | `ManualSalePage.tsx`（`submitCurrentForm`・`needs_confirmation` の枝を含む） | MS の S1〜S8・L1・MS1・MS2 | 非対象 |
| UI-04-D16（lock source） | 62 UI-04-D16 | suggest の `isLocked` | AC4、既存の `ManualSalePage.suggest.test.tsx` | 非対象 |
| UI-04-D8・D9（確認待ち・編集で token を捨てる） | 62 UI-04-D8・D9 | 変えない（確認待ちで unlock） | MS1・MS2、既存の `shows PLU confirmation without result and resubmits with the same key and token`・`clears PLU confirmation after editing sale contents` | 非対象 |
| UI-04-D11・§62.5 | 62 UI-04-D11・§62.5 | ref の lock | MS の S1・S2 | 非対象 |
| UI-04-D10・D12 | 62 UI-04-D10・D12 | 変えない | MS の S1 | 非対象 |
| UI-03-D24（新） | 63 UI-03-D24・§63.3・§63.6 | `ReturnExchangePage.tsx` | RX の S1〜S8・L1・RX1・RX2 | 非対象 |
| UI-03-D21（lock source） | 63 UI-03-D21 | suggest の `isLocked` | AC4、既存の `ReturnExchangePage.suggest.test.tsx` | 非対象 |
| UI-03-D6（返品は戻りだけ） | 63 UI-03-D6 | 変えない（`addProduct` の updater） | RX1 | 非対象 |
| UI-03-D14・§63.5 | 63 UI-03-D14・§63.5 | ref の lock（画像の保存を含む） | RX の S1・S2 | 非対象 |
| UI-03-D13・D15 | 63 UI-03-D13・D15 | 変えない | RX の S1 | 非対象 |
| catalog ⑮ D4・D10（lock source） | catalog ⑮ | 3 画面の suggest の `isLocked` と保存の event の `invalidateAndClose()` | AC4、4 画面の既存の suggest の test | 非対象 |
| catalog ⑮ D9（commit 経路を共通化しない） | catalog ⑮ D9 | 画面ごとに書く | 構造（review で確かめる） | 非対象 |
| catalog ⑮ D11（数字確定の commit） | catalog ⑮ D11 | 同じ `handleProductSearch` を通る | S8 の 1 本を数字確定の経路で（DP） | 非対象 |
| UI-USW（離脱の guard の `!isFormLocked`） | 61・62・63・64 §n.7 | 変えない（render の state のまま） | 既存の `*.unsaved-guard.test.tsx` | 非対象 |

隣の契約の sweep（除外の理由）: live 候補の破棄条件（catalog ⑮ D2〜D4 の suggest の世代。変えない）、BIZ の冪等と token（`manual_sale.rs` ほか。変えない、最後の防御）、返品交換の画像の選択・削除（同期の event）、原価差分の dialog（UI-02-D15。保存成功の後の表示で、検索と関係しない）、振替（#216）と棚卸し（73）。

## Test Plan

画面の略: DP 廃棄、RC 入庫、MS 手動販売、RX 返品交換。S1〜S8 と L1 は 4 画面それぞれの `*.stale-search.test.tsx` に置く（同じ名で画面の ID を付ける。例 `REQ-201 UI-02-D19 S1 ...`）。遅れた応答は deferred の promise で作り、`LATE-001`・`遅れて届いた商品` のように設計の期待と見分けられる値にする。

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| UI-xx-D19・D24・UI-05-D15、§n.5 | 保存成功の後（「続けて」の前）に届いた古い成功で明細に入り、result panel が消え、2 回目の create ができる | component | S1 `late search success after save success keeps the result panel and allows no second create`（明細 1 行 → `LATE-001` で検索を始める → 保存 → result → resolve → `遅れて届いた商品` が無い、result が在る、保存の button が disabled、create 1 回） | 成功の側の照らしを外す、lock を保存成功で外す、`addProduct` が render の `isFormLocked` を見る（main の RC・MS・RX で red を確かめた） |
| 同上、UI-02-D9・UI-04-D11・UI-03-D14・UI-05-D10 | 保存中に届いた古い成功が明細に入る | component | S2 `late search success while saving is not added`（保存の promise を deferred にし、保存中に resolve） | 成功の側の照らしを外す（main の RC・MS・RX で red） |
| 同上 | 「続けて」の後の新しいフォームに古い成功が入り、次の保存に載る | component | S3 `late search success after continue is not added and not in the next payload`（「続けて○○」→ resolve → 無い → 新しい商品を足して保存 → payload に `LATE-001` が無い） | `resetForm` で世代を進めない（main の 4 画面で red） |
| 同上 | リセットの後の古い成功が入る | component | S4 `late search success after reset is not added` | `resetForm` で世代を進めない（main の 4 画面で red） |
| 同上 | リセットの後の古い失敗の文が出る | component | S5 `late search failure after reset shows no message`（`遅れた検索の失敗` が無い） | `catch` の照らしを外す（main の 4 画面で red） |
| 同上（保存の開始で世代を進める、O1 = a） | 保存の失敗の後に、保存の前に始めた検索の応答が入る。失敗の後の新しい検索が入らない | component | S6 `search started before a failed save is discarded and a new search is added`（保存を command の error にする → 失敗の Alert → 古い resolve → 無い → 新しい検索 → 入る） | 保存の event で世代を進めない（main の DP で red を確かめた。RC・MS・RX は lock が無いので main でも red）、`onError` で lock を外さない（後半が red） |
| 同上（件数によらない） | リセットの後に古い複数件の候補の一覧と `候補から…` の文、古い 0 件の `該当する商品がありません` が出る | component | S7 `late multi-candidate and no-match responses after reset show nothing`（2 件の応答と 0 件の応答の 2 本） | 照らしを 1 件の分岐の中だけに置く（main の DP の複数件で red を確かめた） |
| 同上（捨て過ぎない、世代は検索ごと） | リセットの後に始めた新しい検索まで捨てる。古い応答が後から届いて新しい結果を上書きする | component | S8 `a search started after reset is added even if the older one resolves later`（古い検索を始める → リセット → 新しい検索を始める → 新しい resolve → 入る → 古い resolve → 古いは無く新しいは残る）。DP では新しい検索を `onComposedDigitsCommit` の数字確定の経路で始める（catalog ⑮ D11） | 世代を検索ごとに持たず共有の flag にする、照らしを `!==` でなく `<` にする誤り、`resetForm` で lock を外さない |
| UI-xx の「保存 event の先頭で lock」 | 保存の click と pending の render の間に始まった検索の応答が保存中に入る | component | L1 `a search started inside the save event before the pending render is discarded`（create の mock の実装の中で検索欄に Enter を送り新しい検索を始めて即 resolve → 保存中に無い）。廃棄の既存の `locks late product results at the save event before the pending render` と同じ前提。届かない（`mutationFn` が pending の render の後に呼ばれ欄が disabled）と分かったら、その事実を PR に書き、L1 を既存の廃棄の形（検索を mock の中で resolve）に置き換える | 保存の event で lock を立てない、開始の検査を render の `isFormLocked` のままにする（未実測: この test が red になるかは実装の時に確かめる） |
| UI-04-D19・D8・D9 | PLU 確認待ちの間に保存の前の検索の応答が入り、token と明細が食い違う | component | MS1 `search started before save is discarded during PLU confirmation and confirm resubmits the same items, key and token`（1 回目 `needs_confirmation` → 古い resolve → 無い、警告と `確認して保存` が在る → 押す → 2 回目の payload の items・key・token が 1 回目と同じ） | 保存の event で世代を進めない（main で adopted を確かめた） |
| UI-04-D19・D9 | 確認待ちで lock を外し忘れ、確認待ちの間の新しい検索が入らない。入っても token を捨てない | component | MS2 `a search started during PLU confirmation is added and discards the token`（確認待ち → 新しい検索 → 入る → 警告が消え、button が `手動販売を保存`、次の保存の token が null で key が新しい） | `needs_confirmation` の枝で lock を外さない |
| UI-03-D24・D6 | 交換・渡しで検索を始め、応答の前に返品へ替えると渡しの行ができる | component | RX1 `exchange-out search resolved after switching to return adds an in row`（payload の direction が `in`） | `addProduct` の updater が足す時点の種別を見なくなる |
| UI-03-D24 | 応答を待つ間に追加方向を替えると、行の方向が契約と違う、または検索が捨てられる | component | RX2 `direction at Enter is used when the add direction changes before the response`（交換・戻りで Enter → 渡しへ替える → resolve → 行の方向が `戻り（在庫+）`） | 方向の切替えで世代を進める（行が無い）、応答の時点の方向を読む（`渡し`） |
| UI-xx-D14・D16・D21・catalog ⑮ D4・D10 | suggest 層と `addProduct` が別の lock を読む | grep と既存の test | AC4 の `rg`、4 画面の `*.suggest.test.tsx` | suggest の `isLocked` を render の `isFormLocked` に戻す（`rg` が当たる） |

状態の遷移: initial（空のフォーム）→ searching（deferred）→ submitting（S2・L1）→ success・result（S1）→ continue・reset（S3・S4・S5・S7・S8）→ failure → retry（S6・新しい検索）→ needs_confirmation → confirm（MS1・MS2）。revisit・restart は ref が mount ごとに初期化されるだけで、持ち越す state が無い（新しい test は足さない）。invalidate・refetch は検索に cache を使わないので当たらない（直近の一覧の invalidate は既存の test のまま）。

mutation の注入（Contract Audit で実装に注入して red を確かめる）: M1 `resetForm` の世代の増加を消す（S3・S4・S5・S7 が red）。M2 保存の event の世代の増加を消す（S6・MS1 が red。S1・S2 は lock で緑のまま）。M3 `catch` の照らしを消す（S5 が red）。M4 成功の側の照らしを消す（S1〜S4・S7 が red）。M5 手動販売の `needs_confirmation` の枝の unlock を消す（MS2 が red）。M6 `onError` の unlock を消す（S6 の後半が red）。M7 保存の event の lock を消し、開始の検査を render の値に戻す（L1 が red の見込み。未実測）。M8 返品交換の 1 件の応答を応答の時点の方向で足す（RX2 が red）。M9 3 画面の suggest の `isLocked` を render の値に戻す（AC4 の `rg` が当たる。振舞いの test は窓が狭く red にならない見込みで、`rg` で見張る）。

## 目視を置かない理由

L3 の節は置かない（manual なし）。理由: 失敗は検索の応答と保存・リセットの押下の順の競合で、ローカルの SQLite の検索の応答の窓（未実測、短い見込み）の中で人が保存やリセットを押す必要があり、Windows native で再現の手順を決められない。L3 Eligibility の (1)（Windows/Tauri native でしか観測できない）を満たさず、deferred の promise で順を固定した自動 test のほうが確かに観測できる。画面の見た目・文言・配置は変えない（disabled・result panel・案内の文は今のまま）。Coordinator が目視を求めるなら、R3 の packet の規則で `## L3` を足して Gated Amendment にする（その場合の項目は「入庫で明細を足して保存 → result panel → 続けて入庫 → 新しい検索が今までどおり入る」の退行の確認だけで、競合そのものは観測しない）。
