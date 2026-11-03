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
- 返品交換で、レシート画像の読込みの間に保存が成功し、その後に画像の選択が届いても、result panel が残り、create は 1 回だけ（round 1 の是正で足した）。
- 上のどれの後でも、その後に始めた検索は今までどおり明細に入る（捨て過ぎない）。

### 失敗定義

- 上の経路のどれかで、古い応答の商品が明細に入る、古い候補の一覧・`該当する商品がありません`・検索の失敗の文が新しいフォームに出る。
- 保存成功の後に古い応答・遅れたレシート画像の選択で result panel が消え、2 回目の create が別の key で送れる。
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
- 検索の外の、await の後で form を変える入力（計画の review round 1 の P3-2 を受けて 2026-11-04 に `rg -n "await |\.then\(|setTimeout|onSelect=|onComposedDigitsCommit|FilePicker|useEffect\(" <4 画面の page>` と実読で確かめた）:
  - 返品交換のレシート画像: `src/components/FilePicker.tsx` の `chooseFile` は `await open(...)` と `await readFile(path)` の後で（:69-87）、`handleDrop` は `await file.arrayBuffer()` の後で（:89-101）`onSelect` を呼ぶ。`onSelect` は `handleReceiptFile`（`ReturnExchangePage.tsx:378-400`）で、選択を始めた render の `isFormLocked` を見てから `setReceipt`・`setResult(null)`・`rotateKeyAfterFailedAttempt()` を行う。画像の読込みの間に保存が成功すると result panel が消えて lock が外れ、保存成功で新しくなった key で同じ明細をもう一度保存できる（関門 13 と同じ型の二重の記録。窓は狭い。code を読んだ判断で、再現はしていない）。本 lane の Scope に入れた（Coordinator の裁定 2026-11-04）。`removeReceiptImage`（:403-413）は click の同期の handler で、lock 中は button が disabled なので当たらない。
  - 入庫の取引先: `SupplierPickerDialog` の新規作成の経路は `onCreated` の `await supplierQuery.refetch()`（`ReceivingPage.tsx:427-429`）の後で `onSelect(supplier.id)`（`SupplierPickerDialog.tsx:198-203`）を呼び、`updateValues` が `setResult(null)` と key の更新を行う。code の形は同じ型だが、この await は modal の picker の dialog（Radix の `Dialog`）が画面を覆っている間に走り、保存・リセットを押すには先に dialog を 2 つ閉じる必要がある。local の取引先の一覧の再取得の間にそれを済ませて保存を成功させる順は、人の操作では起きない見込み（未実測）。Scope に入れない（Coordinator の裁定 2026-11-04: dialog が画面を覆う間は保存・リセットに届かないので、D-147 の「要るか」で足さない。人の操作で起きると分かったら、同じ 1 行の lock の検査を `onSelect` の handler の先頭に置く）。
  - 廃棄・手動販売: 直接検索のほかに await の後で form を変える入力は無い（上の `rg` の当たりは `handleProductSearch`・保存の `mutationFn` と `onSuccess` の `invalidateByContract`・focus の `setTimeout`・数字確定の `onComposedDigitsCommit` だけ）。返品交換の `saveReceiptImage` の await は保存の `mutationFn` の中で、保存の event の lock の内側。

## 共有 hook か画面ごとか（D-147）

画面ごとに書く（推奨、本 packet の Scope）。D-147 の順に当てた:

1. 要るか: 要る（関門 13、上の再現）。
2. 既にある helper・pattern: 廃棄の UI-05-D15 の lock ref と、#216 の世代の確かめがある。共有の hook は無い。pattern をそのまま 3 画面に移し、廃棄には世代を足す。1 画面の差分は ref 2 つと、保存の event・`resetForm`・`onError`・（手動販売は確認待ち）での更新と、検索の 2 か所の照らしで、数行の見込み（未実測）。
3. 共有 hook（例 `useDirectSearchGuard`）を採らない理由: (a) 防ぐ失敗は「どの event で世代を進め、lock を外すか」の画面ごとの結線の誤りで、hook にしても結線の test は画面ごとに要り、確かめやすさが上がらない。(b) catalog ⑮ D9 が「既存 5 画面の commit 経路コードの共通化・移動は行わない」と決めている。(c) #216 が未 merge で、5 画面目の手本を本 lane の依存にしない。
4. 広げる条件は実装の `ponytail:` comment に書く: 同じ gate を持つ画面が 6 つ（#216 を入れて）を超えるか、gate の条件が増えたら共有 hook を検討する。

#216 が採るか: 本 lane は #216 の merge に依らない（4 画面の page と 61〜64 に #216 は触れない。下の「他の lane との重なり」）。#216 は保存の開始で世代を進めないので、保存の前に始めた検索の応答が保存の失敗の後に届くと採る（本 lane の廃棄の probe と同じ形。二重の記録にはならない）。揃えるかは #216 の側の判断で、Coordinator が #216 の closure か merge の後の小さな lane に回すのを推奨する（本 lane では決めない）。

## owner の判断事項

製品の問いは無い見込み（既存の正本〈UI-02-D9・UI-04-D11・UI-03-D14・UI-05-D10 と「保存成功後はフォーム本文を操作不可」〉に合わせる是正）。1 つだけ、見える振舞いの差を candidate として挙げた（選択肢と推奨は packet の起票の AI が出した）。owner の答え（2026-11-03、repo 外の回答台帳 TD-594）: O1 = a（保存を押したら捨てる）:

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
| 63 | 新設 UI-03-D24（PR #227 が UI-03-D23 を足すので D24。#227 が merge されなくても欠番のまま D24 を使う） | UI-02-D19 と同じ形に、保存 event が画像の保存を含むこと、交換の方向は Enter の時点の追加方向で、足す時点の種別が返品なら戻り、応答を待つ間の種別・方向の切替えでは世代を進めない、を足した。round 1 の是正で、レシート画像の選択（FilePicker の読込みの後の `onSelect`）も lock 中は捨てること、その限界（reset の後に届いた画像は新しいフォームに付く）を足した。 |
| 63 | UI-03-D21・§63.3・§63.6・§63.9・更新履歴 | lock source、状態の段、Error / Recovery の行、Test Focus、履歴。 |
| catalog ⑮ | 契約の見出しの注・D4・D10・更新履歴 | lock source を取引 4 画面の form lock ref に（取引 3 画面は未実装）。直接検索の世代は各画面の ID が持つと D4 に 1 文。 |
| 61・62・63・64（計画の review round 1 の是正、2026-11-04） | UI-02-D19・UI-04-D19・UI-03-D24・UI-05-D15 の理由、§61.9・§62.9・§63.9・§64.9 | lock ref を世代と別に残す理由（保存の click から pending の render までの窓、UI-05-D15 の型）を各 1 文。Test Focus の「保存 event が pending render より先に lock する」を、jsdom で観測できず構造の検査で見る旨に直した。64 §64.9 の UI-05-D15 は既存の test の今の役割（保存の event での破棄の退行の確認）に直した。 |
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
  - 返品交換の `handleReceiptFile` の先頭の検査を `isFormLockedRef.current` にする（lock 中なら何も set しない。UI-03-D24）。`FilePicker`（共有の部品）は変えない。
  - `ponytail:` comment（「共有 hook か画面ごとか」の 4）を各画面の世代の ref の所に 1 つ。
  - render の `isFormLocked`（disabled・離脱の guard・result の表示）は残す。
- test（新、画面ごとに 1 file。種は `.local/reports/stale-direct-search-probes/` の 4 本。`console.log` と `if` の分岐を外し、assert だけにする。test の名に REQ と UI の ID を付ける）: `src/features/disposal/DisposalPage.stale-search.test.tsx`、`src/features/receiving/ReceivingPage.stale-search.test.tsx`、`src/features/manual-sale/ManualSalePage.stale-search.test.tsx`、`src/features/return-exchange/ReturnExchangePage.stale-search.test.tsx`。中身は Test Plan。
- 生成物: `docs/function-design/90-traceability.md`（`cd src-tauri && cargo run --bin generate_traceability`。手で直さない。新しい test file が REQ-201〜204 の行に入る）。
- docs（実装の PR）: 61 UI-02-D19・D14、62 UI-04-D19・D16、63 UI-03-D24・D21、64 UI-05-D15、catalog ⑮ の注・D4・D10、各 §n.3・§n.6・§n.9 から本 lane の「未実装」を外し、履歴に実装の行を足す。backlog の項目を済みにし、Plans の関門 13 に「→ 済み: PR #N」を書く（gate 8 と同じ形）。
- 変えない既存 test: `DisposalPage.test.tsx` の UI-05-D15 の 5 本（`unlocks product search directly after a successful reset`・`ignores late product search results after the form is locked`・`locks late product results at the save event before the pending render`・`unlocks product search after frontend validation failure`・`unlocks product search after command failure`）、4 画面の `*.suggest.test.tsx`・`*.unsaved-guard.test.tsx`・`*.test.tsx`。弱めない・消さない。5 本の役割は Design Readiness の「この変更で不要になる記述・条件」の「残すと決めたもの」に書いた（`locks late product results at the save event before the pending render` は世代を足すと役割が変わる）。

Non-scope: `src/components/patterns/useProductAddSuggest.ts`・`ProductAddSuggest.tsx`・`src/components/FilePicker.tsx`（変えない）、`src/features/stock-transfer/`（#216）、`src/features/stocktake/`（調べていない。棚卸しの検索は `useFindStocktakeItem` で別の形）、BIZ・CMD・bindings、`src/lib/invalidation-contract.ts`、返品交換の `removeReceiptImage`（click の同期の handler で、lock 中は button が disabled）、入庫の取引先の新規作成の後の選択（「現状の調べ」の最後の項。Coordinator の裁定を待つ）、55/56/57/34/24/37/45（他の lane）。

他の lane との重なり（2026-11-03、`gh pr diff N --name-only` の目視）:

- PR #227（返品交換の `register_processed`、L3 待ち）: `docs/function-design/63-ui-return-exchange.md`・`docs/design-system/02-component-catalog.md`・`src/features/return-exchange/ReturnExchangePage.tsx`・`ReturnExchangePage.test.tsx`・`ReturnExchangePage.suggest.test.tsx`・`ReturnExchangePage.unsaved-guard.test.tsx`・`90-traceability.md`・backlog に触れる。63: #227 は UI-03-D8・D15・D17・D18 の行・D23 の新設・§63.0・§63.3 の state の表と form state の段・§63.5・§63.6 の新しい行・§63.7・§63.9・更新履歴。本 lane は UI-03-D21 の lock source・D22 の後の D24・§63.3 の form state の段の前の新しい段・§63.6 の末尾・§63.9 の末尾・更新履歴。末尾の追記と履歴の表は merge 順で両方を残す。§63.3 の段は隣り合うので、後に入る側が同期で手で解く見込み。catalog: #227 は ⑬ の除外の段だけで、⑮ と重ならない。`ReturnExchangePage.tsx`: #227 の hunk は form の型・レジ戻しの欄・保存の分岐（:246-259 の `onSuccess` の付近を含む）で、本 lane は ref・`handleProductSearch`・`addProduct`・`resetForm`・保存の `onClick`・`onError`。`onSuccess`/`onError` の付近で衝突しうる。実装は #227 の merge を待たない。後に入る側が単段の同期で解き、衝突の解消が runtime に及べば DEV_WORKFLOW の Stacked train の規則で Final Reviewer が差分を確かめ直す。新しい test file は別の名で衝突しない。
- PR #216（振替）: 4 画面の page・61〜64・catalog に触れない（`git diff origin/main origin/agent/stock-transfer --name-only` に無い）。`90-traceability.md` だけが重なる（生成物は取り込んだ後に再生成する、D-098）。
- PR #228・#229・#235・#236・#194: `docs/backlog.md`・`docs/Plans.md`（#229）・`90-traceability.md` だけ。backlog・Plans は別の項目の行で、生成物は再生成。

## Acceptance Criteria

- AC1（再現の閉じ）: 4 画面の `*.stale-search.test.tsx` の S1〜S5（調べの A〜E と同じ経路）が PASS。実装の前の main では入庫・手動販売・返品交換の S1〜S5 と廃棄の S3〜S5 が red（上の調べの probe の出力と同じ）。
- AC2（二重の記録が無い）: 入庫・手動販売・返品交換の S1 で、古い応答の後も result panel（`○○を保存しました`）が在り、保存の button が disabled、`createReceiving` / `createManualSale` / `createReturn` の呼出しが 1 回。
- AC3（未確認だった経路）: S6（保存の失敗の後）・S7（複数件・0 件）・S8（新しい検索は採る）が 4 画面の `*.stale-search.test.tsx` で、MS1・MS2（確認待ち）が `src/features/manual-sale/ManualSalePage.stale-search.test.tsx` で、RX1・RX2（方向）と RX3（遅れたレシート画像）が `src/features/return-exchange/ReturnExchangePage.stale-search.test.tsx` で PASS（`npx vitest run <file>` の exit 0）。
- AC4（lock source と保存の event の lock。構造の検査）: `rg -n "isLocked: \(\) => isFormLocked\b" src/features` が 0 件（4 画面とも ref を読む）。`rg -n "isFormLockedRef" src/features/{receiving,manual-sale,return-exchange}` が各画面で、保存の event・`onError`・`resetForm`・`handleProductSearch`・`addProduct`・suggest の `isLocked` の所に当たる（手動販売は `needs_confirmation` の枝にも、返品交換は `handleReceiptFile` にも）。`rg -n -B4 "createMutation\.mutate\(" src/features/disposal/DisposalPage.tsx src/features/receiving/ReceivingPage.tsx src/features/manual-sale/ManualSalePage.tsx src/features/return-exchange/ReturnExchangePage.tsx` の出力で、4 画面とも保存の呼出し（廃棄・入庫・返品交換は保存の button の `onClick`、手動販売は `submitCurrentForm`）の前の行に `isFormLockedRef.current = true` と `searchGenerationRef.current += 1` がある（`mutate` より後に置かない）。保存の event の lock は jsdom の test で観測できない（下の Test Plan の注）ので、この構造の検査で見張る。
- AC5（既存の振舞い）: 4 画面の既存の page・suggest・unsaved-guard の test が assert を変えずに PASS（廃棄の UI-05-D15 の 5 本を含む）。
- AC6（docs）: 実装の PR で `rg -n "関門 13 の lane" docs/function-design/6[1-4]-*.md docs/design-system/02-component-catalog.md` の当たりが履歴の行だけ（本 lane の「未実装」が残らない）。
- AC7（gate）: `bash scripts/doc-consistency-check.sh` と `--target plan docs/plans/2026-11-03-stale-direct-search-guard.md` が ERROR なし。Frontend の gate（`npm run typecheck && npm run lint && npm run format:check && npm test && npm run build`）と traceability（`cd src-tauri && cargo run --bin generate_traceability -- --check`）が通る（DEV_WORKFLOW Verification Gates）。

## Design Readiness

- 引用する設計正本（節まで）: 61 UI-02-D4・D5・D8・D9・D10・D14・D18・D19（新）・§61.3・§61.5・§61.6・§61.9、62 UI-04-D4・D5・D8・D9・D10・D11・D12・D16・D19（新）・§62.3・§62.5・§62.6・§62.9、63 UI-03-D6・D9・D10・D13・D14・D15・D21・D24（新）・§63.3・§63.5・§63.6・§63.9、64 UI-05-D5・D6・D10・D11・D15・D16・§64.3・§64.5・§64.6・§64.9、catalog ⑮ D1・D4・D9・D10・D11、AGENTS.md D-127・D-147。
- 必要な設計成果物: updated in this PR（plan-first: Scope の表）。decision-log の新しい D は置かない（4 画面の局所の判断で、UI-05-D15 の既存の型を移すだけ。横断の新しい選択は無い）。
- plan にしかない durable な判断の昇格先（本 commit で昇格した）: 世代を保存の開始でも進める理由と不採用の案 → UI-05-D15・UI-02-D19 の理由。共有 hook を採らない理由 → UI-02-D19 の不採用。PLU 確認待ちの扱い → UI-04-D19。交換の方向とレシート画像の選択の lock・その限界 → UI-03-D24。lock ref を残す理由（保存の click から pending の render までの窓）→ UI-02-D19・UI-04-D19・UI-03-D24・UI-05-D15 の各 1 文。lock source → D14・D16・D21・catalog ⑮ D4・D10。packet にだけ残るのは O1 の記録と「共有 hook か画面ごとか」の詳しい理由（正本は UI-02-D19 の不採用の文）。
- 前提・制約と延期した design gap: (i) `searching_product` の loading の表示は §n.3 にあるが今の build に無い（範囲の外、上の非目的）。(ii) 棚卸し・振替は別。(iii) UI-02-D18 の `getStockDetail` は未実装で、UI-02-D19 に従うと書いた（D-138 の runtime の lane が持つ）。
- 絶対保証の例外と抜け道: 「保存の開始・リセット・続けての前に始めた検索の応答は反映しない」は、`handleProductSearch` の中の 2 か所の照らしで保つ。抜け道: (a) 応答の後の `addProduct` の中の `window.setTimeout` の focus は照らしの後で、反映しない応答では呼ばれない。(b) 世代は保存の開始で進むので、`mutationFn` の validation の失敗（command を呼ばない）でも保存の前の検索は捨てる（O1 の (a) の範囲）。(c) 交換の方向の切替えは世代を進めない（UI-03-D24。足した行は方向の列で見える）。(d) live 候補は suggest 層の D4 が持ち、本 lane の世代の外（lock source だけを共有）。(e) レシート画像の選択は lock だけで捨て、世代は見ない。親は `FilePicker` の読込みが始まった時点を知らない（`onSelect` は読込みの後に 1 回だけ呼ばれる）ので、リセット・「続けて」の前に始めた選択が後に届くと新しいフォームに画像が付く。プレビューと削除の button で見え、二重の記録にはならない（UI-03-D24 に限界として書いた）。閉じるには共有の `FilePicker` に開始の通知を足す必要があり、本 lane では採らない。(f) 入庫の取引先の新規作成の後の選択（「現状の調べ」の最後の項）は Coordinator の裁定を待つ。
- 計画の review の固定の問いへの先回りの答え:
  - (1) 普通の一日: 成立。初期状態: 入庫の画面、明細 1 行 → 商品を Enter で検索 → 応答の前に「入庫を記録」→ 保存成功の result panel → 遅れた応答は捨てられ result panel が残る → 「続けて入庫」→ 空のフォームで新しい検索は入る → 保存は 1 回分の記録。翌日の起動: ref は画面の mount ごとに 0 と false から始まり、持ち越しは無い。再試行: 保存の失敗の後、同じ内容の再送は同じ key（UI-02-D8 のまま）、保存の前の検索は検索欄の文字で Enter をし直す。危険な結果を出さないか: 二重の記録は S1 で、確認待ちの token の食い違いは MS1 で見る。
  - (2) 登録と生成: command・bindings・route・navigation・REQ の追加は無い。新しい test file 4 本が `90-traceability.md` の再生成に入る（Scope）。新しい function-design の file は無い。
  - (3) 外の前提: 保存の click から pending の render までの窓は jsdom の test では観測できない（計画の review round 1 の実測: 入庫に計画の形を一時に入れ、create の mock の中で検索欄を見ると既に disabled〈`disabledAtCall: true`〉）。この窓は lock ref が閉じ、構造の検査（AC4）で見張る。Windows WebView2 の IPC の遅さは窓の長さを変えるだけで、照らしの論理は変えない。FilePicker の `open`・`readFile` の待ちの長さ（native の dialog が webview の操作を止めるか）は未確認で、lock の検査はそれに依らない。
  - (4) データの安全: 店の実データを使わない。test の値は合成（`makeMockProductWithRelations`）。probe の写しは repo の外の `.local/` にあり commit しない。
  - (5) 前提の出所: 関門と範囲は owner の決定 TD-586（台帳）。経路の事実は上の「現状の調べ」の file:行と probe の実行の出力（本 packet の起票の時に再実行した）。O1 は AI の候補で、owner の決定ではない。
- この変更で不要になる記述・条件: (i) 61 UI-02-D14・62 UI-04-D16・63 UI-03-D21・catalog ⑮ D4・D10 の「取引 3 画面は既存 `isFormLocked` 派生 state」（lock ref に置き換えた）。(ii) code の、検索の開始・`addProduct`・suggest の `isLocked` の render の `isFormLocked` の検査（ref に置き換える）。(iii) catalog ⑮ の注の「D1〜D10 本文不変」は D4・D10 で成り立たなくなったので、改めた旨を同じ注に足した（凍結の記録として消さない）。(iv) backlog の項目と Plans の関門 13 は実装の merge で済みにする。残すと決めたもの: render の `isFormLocked`（disabled・離脱の guard・result の表示に要る）。lock ref（観測できる経路は世代と render の disabled でほぼ覆われるが、保存の click から pending の render までの窓〈その間に始まる検索・候補の選択・遅れたレシート画像〉を閉じる。UI-05-D15 の型。61〜64 の各 D に 1 文で書いた）。廃棄の既存の UI-05-D15 の test 5 本（弱めない・消さない）: `unlocks product search directly after a successful reset`・`unlocks product search after frontend validation failure`・`unlocks product search after command failure` は lock を外す event を、`ignores late product search results after the form is locked` は保存成功の後の破棄を見張る（意味は変わらない）。`locks late product results at the save event before the pending render` は、世代を足すと、create の mock の中で resolve する検索が保存の前に始めた検索なので保存の開始の世代でも捨てられ、lock 単独を見張らなくなる（round 1 の実測。保存の event の lock を外しても green）。今後の役割は「保存の event と競合して届いた、保存の前の検索の応答を反映しない」の退行の確認で、保存の event の lock そのものは AC4 の構造の検査で見る。BIZ の冪等と token の検査（最後の防御）。
- 判定: ready。O1 は owner の答え（2026-11-03、TD-594）で a に決まった（既存の UI-05-D15 の意図と同じ）。ほかに未解決の設計の問いは無い。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| UI-05-D15（世代を足した） | 64 UI-05-D15・§64.3・§64.6 | `DisposalPage.tsx` の世代 ref・保存の event・`resetForm`・`handleProductSearch` | DP の S1〜S8、既存の UI-05-D15 の 5 本、保存の event の lock は AC4 の構造の検査 | 非対象（下の「目視を置かない理由」） |
| UI-05-D16（lock source、変えない） | 64 UI-05-D16 | 変えない | 既存の `DisposalPage.suggest.test.tsx` | 非対象 |
| UI-05-D10 | 64 UI-05-D10 | 変えない（render の `isFormLocked`） | 既存の `hides product registration recovery and locks editing while saving` | 非対象 |
| UI-02-D19（新） | 61 UI-02-D19・§61.3・§61.6 | `ReceivingPage.tsx` | RC の S1〜S8、保存の event の lock は AC4 の構造の検査 | 非対象 |
| UI-02-D14（lock source） | 61 UI-02-D14 | suggest の `isLocked` を ref に | AC4 の `rg`、既存の `ReceivingPage.suggest.test.tsx` | 非対象 |
| UI-02-D9・§61.5（保存中・保存成功の後の操作不可） | 61 UI-02-D9・§61.5 | ref の lock | RC の S1・S2 | 非対象 |
| UI-02-D8（key） | 61 UI-02-D8 | 変えない | RC の S1（create 1 回）、既存の key の test | 非対象 |
| UI-02-D4・D5（0・1・複数件、focus） | 61 UI-02-D4・D5 | 照らしの後の分岐は変えない | RC の S8、既存の test | 非対象 |
| UI-02-D18（未実装、gate に従う） | 61 UI-02-D18・D19 | 本 lane では実装しない | — | 非対象（D-138 の runtime の lane） |
| UI-04-D19（新） | 62 UI-04-D19・§62.3・§62.6 | `ManualSalePage.tsx`（`submitCurrentForm`・`needs_confirmation` の枝を含む） | MS の S1〜S8・MS1・MS2、保存の event の lock は AC4 の構造の検査 | 非対象 |
| UI-04-D16（lock source） | 62 UI-04-D16 | suggest の `isLocked` | AC4、既存の `ManualSalePage.suggest.test.tsx` | 非対象 |
| UI-04-D8・D9（確認待ち・編集で token を捨てる） | 62 UI-04-D8・D9 | 変えない（確認待ちで unlock） | MS1・MS2、既存の `shows PLU confirmation without result and resubmits with the same key and token`・`clears PLU confirmation after editing sale contents` | 非対象 |
| UI-04-D11・§62.5 | 62 UI-04-D11・§62.5 | ref の lock | MS の S1・S2 | 非対象 |
| UI-04-D10・D12 | 62 UI-04-D10・D12 | 変えない | MS の S1 | 非対象 |
| UI-03-D24（新） | 63 UI-03-D24・§63.3・§63.6 | `ReturnExchangePage.tsx` | RX の S1〜S8・RX1・RX2、保存の event の lock は AC4 の構造の検査 | 非対象 |
| UI-03-D24（レシート画像の選択、round 1 の是正で足した） | 63 UI-03-D24・§63.6 | `handleReceiptFile` の先頭の lock ref の検査（`FilePicker` は変えない） | RX3、AC4 の `rg` | 非対象 |
| UI-03-D3・D20（FilePicker の選択・読込み、変えない） | 63 UI-03-D3・D20 | 変えない | 既存の `ReturnExchangePage.test.tsx` の画像の test | 非対象 |
| UI-03-D21（lock source） | 63 UI-03-D21 | suggest の `isLocked` | AC4、既存の `ReturnExchangePage.suggest.test.tsx` | 非対象 |
| UI-03-D6（返品は戻りだけ） | 63 UI-03-D6 | 変えない（`addProduct` の updater） | RX1 | 非対象 |
| UI-03-D14・§63.5 | 63 UI-03-D14・§63.5 | ref の lock（画像の保存を含む） | RX の S1・S2 | 非対象 |
| UI-03-D13・D15 | 63 UI-03-D13・D15 | 変えない | RX の S1 | 非対象 |
| catalog ⑮ D4・D10（lock source） | catalog ⑮ | 3 画面の suggest の `isLocked` と保存の event の `invalidateAndClose()` | AC4、4 画面の既存の suggest の test | 非対象 |
| catalog ⑮ D9（commit 経路を共通化しない） | catalog ⑮ D9 | 画面ごとに書く | 構造（review で確かめる） | 非対象 |
| catalog ⑮ D11（数字確定の commit） | catalog ⑮ D11 | 同じ `handleProductSearch` を通る | S8 の 1 本を数字確定の経路で（DP） | 非対象 |
| UI-USW（離脱の guard の `!isFormLocked`） | 61・62・63・64 §n.7 | 変えない（render の state のまま） | 既存の `*.unsaved-guard.test.tsx` | 非対象 |

隣の契約の sweep（除外の理由）: live 候補の破棄条件（catalog ⑮ D2〜D4 の suggest の世代。変えない）、BIZ の冪等と token（`manual_sale.rs` ほか。変えない、最後の防御）、返品交換の画像の削除（`removeReceiptImage` は click の同期の handler で、lock 中は button が disabled。画像の選択は上の行で本 lane に入れた）、入庫の取引先の新規作成の後の選択（modal の dialog の間の await。「現状の調べ」の最後の項、Coordinator の裁定を待つ）、原価差分の dialog（UI-02-D15。保存成功の後の表示で、検索と関係しない）、振替（#216）と棚卸し（73）。

## Test Plan

画面の略: DP 廃棄、RC 入庫、MS 手動販売、RX 返品交換。S1〜S8 は 4 画面それぞれの `*.stale-search.test.tsx` に置く（同じ名で画面の ID を付ける。例 `REQ-201 UI-02-D19 S1 ...`）。遅れた応答は deferred の promise で作り、`LATE-001`・`遅れて届いた商品` のように設計の期待と見分けられる値にする。

「Would fail if」の「実測」は計画の review round 1（入庫に計画の形を一時に入れ、mutation を注入した Opus の実測。6 files / 55 tests PASS の版から）で、それ以外は未実測の見込み（Contract Audit で注入して確かめる）。直接検索の応答は「成功・失敗の側の照らし（世代と lock）」と「`addProduct` の lock ref の検査」の 2 つで守り、保存の開始と `resetForm` の 2 か所で世代を進めるので、1 か所だけ外しても red にならない test がある（下の各行に書いた）。

保存の event の先頭の lock（保存の click から pending の render までの窓）は jsdom の test で観測できない（round 1 の実測: create の mock の時点で検索欄が既に disabled）。この lock は AC4 の構造の検査で見張り、振舞いの test は置かない。

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| UI-xx-D19・D24・UI-05-D15、§n.5 | 保存成功の後（「続けて」の前）に届いた古い成功で明細に入り、result panel が消え、2 回目の create ができる | component | S1 `late search success after save success keeps the result panel and allows no second create`（明細 1 行 → `LATE-001` で検索を始める → 保存 → result → resolve → `遅れて届いた商品` が無い、result が在る、保存の button が disabled、create 1 回） | 成功の側の照らしと `addProduct` の lock ref の検査を両方外す（M5。main の RC・MS・RX がこの形で red を確かめた）。片方だけ外しても green（成功の側の照らしだけを外した M4 で green、round 1 の RC の実測） |
| 同上、UI-02-D9・UI-04-D11・UI-03-D14・UI-05-D10 | 保存中に届いた古い成功が明細に入る | component | S2 `late search success while saving is not added`（保存の promise を deferred にし、保存中に resolve） | S1 と同じ（M5 で red、M4 だけでは green。round 1 の RC の実測と main の RC・MS・RX の red） |
| 同上（「続けて」の経路の退行の確認） | 「続けて」の後の新しいフォームに古い成功が入り、次の保存に載る | component | S3 `late search success after continue is not added and not in the next payload`（「続けて○○」→ resolve → 無い → 新しい商品を足して保存 → payload に `LATE-001` が無い）。遅れた検索は保存の前に始めるので、保存の開始の世代でも `resetForm` の世代でも捨てられる。「続けて」の経路（result → `resetForm` → 新しいフォームと次の保存）が壊れていないことの確かめで、世代の 1 か所を見張る test ではない | 成功の側の照らしを外す（M4。round 1 の RC の実測で red）、保存の開始と `resetForm` の世代を両方外す（M1 と M2 の組。未実測）。M1 だけでは green（round 1 の RC の実測） |
| 同上 | リセットの後の古い成功が入る | component | S4 `late search success after reset is not added` | `resetForm` で世代を進めない（M1）、成功の側の照らしを外す（M4）。どちらも round 1 の RC の実測で red。main の 4 画面で red |
| 同上 | リセットの後の古い失敗の文が出る | component | S5 `late search failure after reset shows no message`（`遅れた検索の失敗` が無い） | `resetForm` で世代を進めない（M1。round 1 の RC の実測で red）、`catch` の照らしを外す（M3。未実測）。main の 4 画面で red |
| 同上（保存の開始で世代を進める、O1 = a） | 保存の失敗の後に、保存の前に始めた検索の応答が入る。失敗の後の新しい検索が入らない | component | S6 `search started before a failed save is discarded and a new search is added`（保存を command の error にする → 失敗の Alert → 古い resolve → 無い → 新しい検索 → 入る） | 保存の event で世代を進めない（M2）、成功の側の照らしを外す（M4）。どちらも round 1 の RC の実測で red。`onError` で lock を外さない（M6、後半が red。未実測）。main の DP で red を確かめた（RC・MS・RX は lock が無いので main でも red） |
| 同上（件数によらない） | リセットの後に古い複数件の候補の一覧と `候補から…` の文、古い 0 件の `該当する商品がありません` が出る | component | S7 `late multi-candidate and no-match responses after reset show nothing`（2 件の応答と 0 件の応答の 2 本） | `resetForm` で世代を進めない（M1）、成功の側の照らしを外す（M4。複数件・0 件は `addProduct` を通らないので lock ref の検査が守らない）。どちらも未実測。照らしを 1 件の分岐の中だけに置く誤りも red（main の DP の複数件で red を確かめた） |
| 同上（捨て過ぎない、世代は検索ごと） | リセットの後に始めた新しい検索まで捨てる。古い応答が後から届いて新しい結果を上書きする | component | S8 `a search started after reset is added even if the older one resolves later`（古い検索を始める → リセット → 新しい検索を始める → 新しい resolve → 入る → 古い resolve → 古いは無く新しいは残る）。DP では新しい検索を `onComposedDigitsCommit` の数字確定の経路で始める（catalog ⑮ D11） | `resetForm` で世代を進めない（M1）、成功の側の照らしを外す（M4）、世代を検索ごとに持たず共有の flag にする、`resetForm` で lock を外さない（前半が red）。どれも未実測 |
| UI-04-D19・D8・D9 | PLU 確認待ちの間に保存の前の検索の応答が入り、token と明細が食い違う | component | MS1 `search started before save is discarded during PLU confirmation and confirm resubmits the same items, key and token`（1 回目 `needs_confirmation` → 古い resolve → 無い、警告と `確認して保存` が在る → 押す → 2 回目の payload の items・key・token が 1 回目と同じ） | 保存の event で世代を進めない（main で adopted を確かめた） |
| UI-04-D19・D9 | 確認待ちで lock を外し忘れ、確認待ちの間の新しい検索が入らない。入っても token を捨てない | component | MS2 `a search started during PLU confirmation is added and discards the token`（確認待ち → 新しい検索 → 入る → 警告が消え、button が `手動販売を保存`、次の保存の token が null で key が新しい） | `needs_confirmation` の枝で lock を外さない |
| UI-03-D24・D6 | 交換・渡しで検索を始め、応答の前に返品へ替えると渡しの行ができる | component | RX1 `exchange-out search resolved after switching to return adds an in row`（payload の direction が `in`） | `addProduct` の updater が足す時点の種別を見なくなる |
| UI-03-D24 | 応答を待つ間に追加方向を替えると、行の方向が契約と違う、または検索が捨てられる | component | RX2 `direction at Enter is used when the add direction changes before the response`（交換・戻りで Enter → 渡しへ替える → resolve → 行の方向が `戻り（在庫+）`） | 方向の切替えで世代を進める（行が無い）、応答の時点の方向を読む（M8、`渡し`） |
| UI-03-D24（レシート画像の選択）・UI-03-D15 | 画像の読込みの間に保存が成功し、遅れた `onSelect` で result panel が消えて lock が外れ、新しい key で同じ明細をもう一度保存できる | component | RX3 `late receipt image after save success keeps the result panel and allows no second create`（明細 1 行 → `arrayBuffer()` を deferred にした File を drop zone に drop → 保存 → result → `arrayBuffer` を resolve → result panel が在り、保存の button が disabled、`選択したレシート画像` の img が無い、`createReturn` が 1 回、1 回目の payload の `receipt_image_path` が null） | `handleReceiptFile` が render の `isFormLocked` を見る（M10。今の main の形で、code を読んだ判断では red。未実測） |
| UI-xx-D14・D16・D21・catalog ⑮ D4・D10 | suggest 層と `addProduct` が別の lock を読む | grep と既存の test | AC4 の `rg`、4 画面の `*.suggest.test.tsx` | suggest の `isLocked` を render の `isFormLocked` に戻す（`rg` が当たる） |

状態の遷移: initial（空のフォーム）→ searching（deferred）・画像の読込み（deferred、RX3）→ submitting（S2）→ success・result（S1・RX3）→ continue・reset（S3・S4・S5・S7・S8）→ failure → retry（S6・新しい検索）→ needs_confirmation → confirm（MS1・MS2）。revisit・restart は ref が mount ごとに初期化されるだけで、持ち越す state が無い（新しい test は足さない）。invalidate・refetch は検索に cache を使わないので当たらない（直近の一覧の invalidate は既存の test のまま）。

mutation の注入（Contract Audit で実装に注入して red を確かめる。「実測」は round 1 の RC の実測、ほかは未実測の見込み）:

- M1 `resetForm` の世代の増加を消す → S4・S5（実測）、S7・S8（未実測）が red。S3 は green（実測。保存の開始の世代で捨てる）。
- M2 保存の event の世代の増加を消す → S6（実測）、MS1（未実測）が red。S1・S2 は green（lock で捨てる）。
- M3 `catch` の照らしを消す → S5 が red（未実測）。
- M4 成功の側の照らしを消す → S3・S4・S6（実測）、S7・S8（未実測）が red。S1・S2 は green（実測。`addProduct` の lock ref の検査が守る）。
- M5 成功の側の照らしと `addProduct` の lock ref の検査を両方消す → S1・S2 が red（main の RC・MS・RX がこの形で、probe の A・B で red を確かめた）。`addProduct` の lock ref の検査だけを消しても、直接検索の経路では成功の側の照らしが守るので red にならない見込み（未実測）。
- M6 `onError` の unlock を消す → S6 の後半が red（未実測）。手動販売の `needs_confirmation` の枝の unlock を消す → MS2 が red（未実測）。
- M7 は外した（保存の event の lock と開始の検査は jsdom で観測できない。round 1 の実測。AC4 の構造の検査で見張る）。
- M8 返品交換の 1 件の応答を応答の時点の方向で足す → RX2 が red（未実測）。
- M9 3 画面の suggest の `isLocked` を render の値に戻す → AC4 の `rg` が当たる（振舞いの test は窓が観測できず red にならない見込みで、`rg` で見張る）。
- M10 返品交換の `handleReceiptFile` の先頭の検査を render の `isFormLocked` に戻す → RX3 が red（未実測）、AC4 の `rg` の `handleReceiptFile` の当たりが消える。

## 目視を置かない理由

L3 の節は置かない（manual なし）。`docs/DEV_WORKFLOW.md` の L3 の節の「operator 向けの画面を作る・実質的に変える PR は、merge の前に人の目視を置く」に当たらない理由: 4 画面の見た目・文言・配置と、普通の操作（検索・明細の編集・保存・リセット・続けて・画像の選択）の振舞いは変わらず、変わるのは検索の応答や画像の読込みが保存・リセットと重なった狭い窓の扱いだけで、見える差は O1 の「保存を押す直前に読んだ商品が、保存に失敗したときに入らない」に限られ、これは deferred の promise で順を固定した自動 test（S6・MS1）で観測する。L3 Eligibility から見た理由: 失敗は検索の応答と保存・リセットの押下の順の競合で、ローカルの SQLite の検索の応答の窓（未実測、短い見込み）の中で人が保存やリセットを押す必要があり、Windows native で再現の手順を決められない。L3 Eligibility の (1)（Windows/Tauri native でしか観測できない）を満たさず、deferred の promise で順を固定した自動 test のほうが確かに観測できる。画面の見た目・文言・配置は変えない（disabled・result panel・案内の文は今のまま）。Coordinator が目視を求めるなら、R3 の packet の規則で `## L3` を足して Gated Amendment にする（その場合の項目は「入庫で明細を足して保存 → result panel → 続けて入庫 → 新しい検索が今までどおり入る」の退行の確認だけで、競合そのものは観測しない）。
