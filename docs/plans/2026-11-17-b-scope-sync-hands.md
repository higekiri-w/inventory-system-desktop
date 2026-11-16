# Plan Packet: merge-evidence の B (ii) の Scope の文を、同期の手が持ち込む中身で見る形に置き換える

wave に属さない単独の lane。正本の文（`docs/agent-guidance/merge-evidence.md` の「base同期だけでheadが変わる場合」の「manualの使い回し」の B の (ii) の Scope の 1 文と、`docs/decision-log.md`）だけを変える。helper（`scripts/pr-gate.py`）・その test・classifier・CI の job・製品の runtime は変えない。

出所は owner の決定（原文は repo 外の回答台帳にあり、ここには ID と要旨で引く）:

- TD-743（2026-11-16）: PR #194（R4）の B の closure。同期の merge の中で、main（#235）が計画の後に足した test に、lane の既にある振舞い（RootLayout の hook）に合わせた mock を足した手を、reviewer の 1 本が「Scope の予定 path に無い」として B (ii) の P2 にし、もう 1 本は合格。owner は、main が後から足した同じ型の test を lane の既にある振舞いに合わせて直したのは Scope の中と、その回について裁定した（PR #194 issuecomment-6919572220）。
- TD-746（2026-11-17）: PR #216（R4）の B の closure。同期で手で解いた `docs/backlog.md` と `src/test/page-root-pageshell-sweep.test.ts` が packet の Scope の予定 path に無い（packet は backlog を変えないと書いていた）として Codex が P2、reviewer-fable は合格（食い違いは同期の前からで L3 の版にも当たり、手は L3 の版の lane の変更と main の変更を重ねただけ）。owner は今回も Scope の中と裁定した（PR #216 issuecomment-6920507823）。
- TD-747（2026-11-17）: 2 回続けて止まったので、規則の文を直す lane を起こす。
- TD-750（2026-11-17）: 方向の承認（candidate）。同期の手が (1) L3 の版にあった lane の変更と main 側の変更を重ねること、(2) main が後から足した・変えた test と docs を lane の承認済みの振舞いへ機械的に追随させること（例: 件数の更新、lane の hook の mock）は Scope の外と数えない。それを超えて新しい中身を足したら今までどおり Scope と照らし、外なら Gated Amendment。(ii) の 1 文の置き換えで、条件を継ぎ足さない。helper の code は変えない見込み。

decision-log の番号: 本 lane は D-155 を予約する（下の「調べ」）。

安全の境界: B の closure の (ii) の Scope の照らしは、`docs/DEV_WORKFLOW.md` Workflow State（D-098）が挙げる安全の境界のうち「古い証拠の拒否」と「独立 review」の運用（L3 の後の lane の手を、独立の closure が見て owner が承認したときだけ使い回す）に関わる。本 lane はその closure の不合格の条件の 1 つを緩める向きに改訂する（同期の手が L3 の版の lane の変更と main 側の変更を重ねただけのもの、main 側の test・docs を lane の承認済みの振舞いへ機械的に合わせただけのものを、Scope の外と数えない）。改訂は通常の packet・PR の経路で行う（本 packet。owner の決定の日付は上のとおり）。次は変えない:

- 集合の判定（helper が機械で確かめる入口。集合の外は L3 のやり直し）。
- (i)（L3 の項目に影響しない。D-146）。
- (ii) の残り（手で解いた path は両側の変更を落とさない、test を消す・skip する・弱めることをしない）と、新しい中身の Scope の照らし（外なら Gated Amendment が通るまで不合格）。
- B の closure の本数・見る範囲・不合格の後の一手・承認と証跡、Plan Commit の固定、R4 の `r4` の承認、fail-closed（集合の外・判定不能は L3 のやり直しに倒れる）、A・C、review の closure の機械の記録。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R3

Reason:
gate の文書 `docs/agent-guidance/merge-evidence.md` を変える（classifier が `workflow=true` を出すので R2 にできない。下の「調べ」）。R4 にしない理由: 破壊的な操作・実データ・secret・不可逆な git の操作を伴わず、helper の code・RecordV1・GitHub への書込みの経路も変えない。

## Goal

Goal Invariant: B の closure の (ii) が、lane の手の変更（足す・変える・消す）のうち新しい中身を今までどおり Scope と照らし、L3 の版の lane の変更と main 側の変更を重ねた手と、main 側の test・docs を lane の承認済みの振舞いへ機械的に合わせた手を Scope の外と数えない。

### 最小完了条件

- merge-evidence の B の (ii) の Scope の文が、下の「規則の形」の新しい文に置き換わる。TD-743 と TD-746 の形の手は新しい文で (ii) の Scope の照らしに合格し、owner のその回限りの裁定を要らない。
- 同期の merge の中で、Scope に無い新しい機能の test や仕様の文を足した手は、今までどおり (ii) で不合格になる（Gated Amendment が通るまで）。
- decision-log に D-155 があり、D-154 に `Superseded in part by: D-155` の 1 行がある。

### 失敗定義

- 新しい文の下で、同期の merge の中か L3 の後の lane の commit で、Scope の外の新しい振舞い・新しい assertion・mock の返り値で test が確かめる振舞いを lane の承認済みの振舞いに無いものへ変える変更・main 側の文の契約を変える直し・Scope の外の main 側の文を消す変更が、(ii) を合格する（読める文になっている）。
- 新しい文の下で、TD-743 の形（main 側の test で lane の hook を止める stub）や、直す commit で両側を戻して最初から正しく解いた merge と同じ結果になった版が、(ii) で不合格になる（読める文になっている）。
- 新しい文が、手の目的（同期の辻褄合わせか lane の新しい変更か）を closure に判定させる形になる（D-154 の Alternatives で TD-723 がやめた形）。
- (ii) の残り（両側を落とさない、test を消す・skip する・弱めない）、(i)、集合の判定、本数、Plan Commit の固定、R4 の承認、fail-closed のどれかの文が変わる。
- 規則の文が merge-evidence の 1 か所の外（decision-log の D-155 の本文、helper の文、ほかの docs）に言い直されて、正本が 2 つになる（D-155 は要旨と参照）。
- 条件・例外を (ii) の後ろに継ぎ足し、Scope の文が 2 つの規則（今の文と例外）に分かれる。

### 非目的

- helper（`scripts/pr-gate.py`）の code・案内の文・test を変えること（B の基準は merge-evidence の 1 か所だけにあり、helper は名指す 1 行だけを出す。D-154）。
- A・C・集合・B の closure の本数・見る範囲・承認と証跡・review の closure の機械の記録を変えること。
- L3 の版の lane の変更が Scope に合うかの判定を B の closure に移すこと（その版の review の範囲のまま）。
- `docs/templates/subagent-review-packet.md` と `.local` の発注書の雛形（repo 外の運用の文は Coordinator が合わせる）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 調べ（2026-11-17 に測った）

`git fetch origin main` の後の `origin/main`（`0cf0a26186029cea09c1dab9d0fc4506a3197c5e`）で測った。

- decision-log の番号:
  - `git grep -n "^## D-15" origin/main -- docs/decision-log.md` の出力（見出しの文は略す）: `origin/main:docs/decision-log.md:1335:## D-150: …`、`:1346:## D-151: …`、`:1357:## D-152: …`、`:1368:## D-154: …`（exit 0）。D-153 の見出しは main に無い。
  - `git grep -n -E "D-15[5-9]" origin/main -- docs`: 当たり無し（exit 1）。
  - `git grep -n -E "D-153" origin/main -- docs`: 当たりは `docs/plans/2026-11-14-sync-docs-conflict.md:16` の 1 行だけ（PR #250〈`agent/window-close-guard`〉の D-153 の予約を記録した文）。
  - 補助: 手元の remote の ref（`origin/agent/window-close-guard`・`stock-transfer`・`sync-docs-conflict`・`backup-offsite-runtime-b`・`docs-prelaunch-drift`・`cost-label-tax-excl`・`backlog-import-preview`・`backlog-single-instance`、`origin/probe/window-close-guard`）に `git grep -n -E "D-15[3-9]" <ref…> -- docs/decision-log.md docs/plans` を当てると、D-155〜D-159 は 0 件、D-153 は `agent/window-close-guard` の見出し（`## D-153: native の window の close は…`）だけ。この ref は本 run で fetch していない（main だけを fetch した）ので古い可能性がある。
  - よって D-155 を予約する（D-153 は PR #250 が予約済み）。
- classifier（`printf '%s\n' <path> | bash scripts/ci/classify-changes.sh --files-from-stdin | grep '=true'`）: `docs/agent-guidance/merge-evidence.md` は `docs=true`・`workflow=true`、`docs/decision-log.md` は `docs=true`、本 packet は `docs=true`。
- helper は B の基準を持たない: `rg -n -i "scope" scripts/pr-gate.py` の当たりは 5 行（`rg -c` が 5）で、`:316`（`plan_texts`）と `:358`（`plan_sections`）の計画の節の名の列（承認の後の計画の節の byte の比較、D-145 (7)）、`:444` の `workflow_scope`（active な packet の列挙）、`:794`・`:795` の `record` の command の引数の組立て（`scope = [...]` と `*scope`）だけ。B の (ii) の語（「承認済みの Scope」「普通の正しさ」）は無い。B を名指すのは `rg -n "manualの使い回し|見るものと合否" scripts/pr-gate.py` の `:64`（`B_RULE`）と `:790`（「closure N 本が見るものと合否: {B_RULE}」の 1 行）だけで、基準を言い直さない（D-154）。
- 同じ規則を言い直す文: `rg -n "普通の正しさ" . --glob '!.local/**'` の当たりは、`docs/agent-guidance/merge-evidence.md:165`（正本）と、merge 済みの packet `docs/plans/2026-11-14-sync-docs-conflict.md:47`・`:143`（D-154 の lane の計画の記録。変えない）だけ。`rg -n "両側.*落とさ|承認済みの Scope|Scope の外の変更|Scope の中にある" --glob '!docs/templates/subagent-review-packet.md' --glob '!.local/**' --glob '!docs/plans/2026-11-14-sync-docs-conflict.md' --glob '!docs/archive/**' .` の当たりは `docs/agent-guidance/merge-evidence.md:165` の 1 行だけ。`docs/DEV_WORKFLOW.md` の L3 の表・Stacked train と `AGENTS.md`・`docs/AGENT_OPERATING_MANUAL.md` に (ii) の Scope の言い直しは無い（`grep -n "Scope" docs/DEV_WORKFLOW.md` は Plan Packet Rules・計画の固定・Review Rules・Contract Ledger の計画の節の話だけ）。

## 規則の形

言葉: merge-evidence の今の定義を使う。「lane の手」は手で解いた path と C に当たらない lane の commit（merge-evidence の「lane の手と集合」）。「L3 の版」は manual の `source_head`（無ければ旧 record の head）。「取り込む main 側」は同期の merge が取り込んだ main 側の変更。新しく 1 つだけ、「lane の承認済みの振舞い」を新しい (ii) の文の中で「L3 の版の lane にある振舞い」と定義する（merge-evidence に定義が無く、「承認済みの Scope の振舞い」と読むと TD-746 の形が外れるため。round 1 の O-P2-2）。

新しい (ii) の文（置き換えの全文。今の (ii) の 1 つの箇条を丸ごと置き換え、前の 2 文は今と同じ）:

> - (ii) lane の手の普通の正しさ。手で解いた path は両側（lane 側と main 側）の変更を落とさない。test を消す・skip する・弱めることをしない。lane の手の変更（足す・変える・消す）は、次の 2 つのほかは新しい中身として承認済みの Scope（packet の `## Scope` 節。packet の無い R0〜R2 の lane は PR body の 5 行の「範囲」の行）と照らす。(1) L3 の版にあった lane の変更（Scope の予定 path に無かったものを含む）と取り込む main 側の変更を重ねること（両側を落とさないためだけの最小のつなぎの言葉を含む）。(2) 取り込む main 側の test と docs を lane の承認済みの振舞い（L3 の版の lane にある振舞い）へ合わせる直しで、test は assertion の意図を変えないもの（例: 件数の更新、lane に既にある hook・module をその test が確かめる対象の外として止める・置き換える、隣の test と同じ形の mock）、docs は main 側の文に lane の承認済みの振舞いを件数・名・参照として入れ、main 側の文が決める振舞いの契約を変えないもの。新しい assertion、mock の返り値でその test が確かめる振舞いを lane の承認済みの振舞いに無いものへ変えること、main 側の文の契約を変えることは新しい中身に当たる。判定は L3 の版と取り込む main 側から来た中身と最後の結果で行い、手を入れた commit（同期の merge か、その後の lane の commit か）で分けない（TD-750・TD-751、D-155）。Scope の外の新しい中身は今の経路（Gated Amendment で計画を変える）で扱い、それが通るまで (ii) は不合格。

文の形: 今の (ii) の Scope の 1 文（「…の中にある。Scope の外の変更は…不合格」）を、同じ箇条の中で「lane の手の変更は (1)(2) のほかは新しい中身として Scope と照らす」の形に言い換えた。条件を箇条の後ろに足していない。長くなったのは (2) の test と docs の線を名指した所で、round 1 の O-P2-1・O-P2-4 が求めた範囲に留めた。

TD-750 の (1)(2) を closure が判定できる言葉にした所:

- 対象: lane の手の変更の全部（足す・変える・消す）。「持ち込む」と書くと足す向きだけに読め、lane の手が main 側の文を消す・書き換える変更が抜けるので、向きを名指した（round 1 の O-P2-3）。
- (1)「重ねる」: 手の結果の中身が、L3 の版の lane 側にあった変更と、取り込む main 側の変更から来ていること。両側を落とさないためだけの最小のつなぎの言葉（どちらの側の意味も足さないもの。例: 2 つの変更を並べるための接続の句、見出しの並べ替え）は (1) に含め、新しい中身に数えない（round 1 の O-P3-1）。L3 の版の lane の変更は、packet の予定 path に無かったものも含む（TD-746 の形）。
- (2)「機械的な追随」: 取り込む main 側の test・docs が対象で、変える向きは lane の承認済みの振舞い（L3 の版の lane にある振舞い）へ合わせることだけ。
  - test: assertion の意図（その test が何を確かめるか）を変えない。例は件数の更新（lane が足した画面・行を数に入れる）と、lane に既にある hook・module を、その test が確かめる対象の外として止める・置き換える mock（隣の test と同じ形）。mock の返り値が lane の振舞いと同じである必要はない（TD-743 の手は hook を止める stub。round 1 の O-P2-1）。超える例は、新しい assertion を足す、mock の返り値でその test が確かめる振舞いを lane の承認済みの振舞いに無いものへ変える。
  - docs: main 側の文に lane の承認済みの振舞いを件数・名・参照として入れる直しだけで、main 側の文が決める振舞いの契約は変えない。main 側の文の契約を変える直し（例: main 側の function-design の文の契約を lane の契約に書き換える）は新しい中身として Scope と照らす（round 1 の O-P2-4）。
- 判定は、L3 の版と取り込む main 側から来た中身と最後の結果で行い、手の目的（同期の辻褄合わせか lane の新しい変更か）を問わない。D-154 の Alternatives が TD-723 でやめた「手の編集の目的を closure に判定させる」に戻さない。
- L3 の版と Scope の食い違い（TD-746 の形。packet の予定 path に無かった lane の変更が L3 の版にある）は、同期の前からあって L3 の版の review の範囲のもので、(ii) の合否に数えない。B の closure が気づいたら、packet の docs の follow-up（P3）として報告する。食い違いが契約に及ぶ（L3 の版の lane の振舞いが計画の契約と違う）ときは、中身に応じて (i) か通常の finding として扱う（round 1 の O-P3-3。Coordinator の裁定）。この扱いは新しい (ii) の文に入れず、D-155 の Decision の要旨に 1 句で置く（文の (1) が「Scope の予定 path に無かったものを含む」で合否を既に決めていて、報告の置き場は合否の基準でないため。文を長くしない）。

C に当たらない lane の commit の扱い: 同期の merge の中の手と同じ線で見る（今の文の「lane の手」が両方を指すので、文を分けない。owner の決定 TD-751）。理由:

- TD-723 は、L3 の後の変更が「同期の辻褄合わせか lane の新しい変更か」を区別しないと決めた。区別しないで済むのは、線を手の置き場所（merge の commit か、その後の 1 親の commit か）でなく中身と最後の結果に引くとき。同じ結果（例: 同期の後で CI が red になり、main が足した test に lane の hook を止める mock を足す commit）を、merge の中なら合格・後の commit なら Scope の外とすると、置き場所で区別することになる。
- 線の中身は (1) と (2) だけで、lane の commit が新しい振舞い・新しい assertion を持ち込むか、Scope の外の main 側の文を消す・書き換えれば、今までどおり Scope と照らす。緩むのは、最後の結果が (1)(2) で説明できる commit だけ。
- 親の数では分けない。「直す commit」（B の不合格の後に lane に足す commit）も、L3 の版と取り込む main 側から来た中身と最後の結果で見る。例: 同期の merge で main 側の文を落とし（両側を落とさないの不合格）、不合格の後の 1 親の commit でその文を戻した版は、最初から正しく解いた merge と結果が同じなので、(1) に当たり合格（round 1 の C-P2-1。下の Test Plan の R5）。

TD-743 の手と TD-746 の手を新しい文に当てる（下の Test Plan の R1・R2）:

- TD-743: 取り込む main 側（#235）が足した test（`src/features/daily-report-import/DailyReportImportRecordsPage.test.tsx`）に、lane に既にある RootLayout の hook を止める mock を足した。現物（`0cf0a26` の 26〜32 行）は `vi.mock("@/features/backup-restore/useAutoBackupCheck", …)` で `useAutoBackupCheck: () => undefined` の stub と、`resumeAutoBackupCheck`・`suspendAutoBackupCheckAndWait`（`vi.fn().mockResolvedValue(undefined)`）・`holdAutoBackupCheckForRestore`・`releaseAutoBackupCheckAfterRestore` の `vi.fn()` 群で、comment は「この画面の test の外なので hook を止める（隣の route の test と同じ）」。同じ stub は `src/lib/app-router.test.tsx` と `src/features/inventory-records/OtherRecordDetailRoutes.test.tsx` にもある（`rg -l "useAutoBackupCheck: \(\) => undefined" src` の当たり 3 件）。対象は main 側の test、hook は test が確かめる対象の外、assertion は足していない → (2) → 新しい中身に当たらない → (ii) の Scope の照らしに合格。
- TD-746: `docs/backlog.md` と `src/test/page-root-pageshell-sweep.test.ts` の手は、L3 の版にあった lane の変更（backlog は packet の予定 path に無かった）と取り込む main 側の変更を重ねただけ → (1) → 新しい中身に当たらない → 合格。packet の予定 path と L3 の版の食い違いは (ii) の合否に数えず、気づいたら P3 の follow-up（PR #216 では #264 がそれに当たる）。

## Scope

- `docs/agent-guidance/merge-evidence.md`: 「manualの使い回し」の B の (ii) の箇条（今の 165 行）を、上の「規則の形」の新しい文に置き換える。ほかの文（(i)、見る範囲、不合格の後、本数、承認と証跡、MG-D13 の表の行、「lane の手と集合」、C の節）は変えない。
- `docs/decision-log.md`:
  - 末尾に D-155（見出しの案: 「同期の手が L3 の版の lane の変更と main 側の変更を重ねるか、main 側の test・docs を lane の承認済みの振舞いへ機械的に合わせるだけなら、B の (ii) で Scope の外と数えない（2026-11-17）」）。Status（owner の決定 TD-743・746・747・750・751 と、本 packet の lane で起票したこと。TD-750 は方向の承認〈candidate〉で、計画の承認の確認を経て書く）。Context（TD-743・TD-746 の 2 回、B の closure が (ii) の Scope で割れ、owner がその回限りで裁定した）。Decision（「規則の形」の要旨: lane の手の変更〈足す・変える・消す〉は (1)(2) のほかは新しい中身として Scope と照らす、(1) は両側を落とさないためだけの最小のつなぎの言葉を含む、(2) は test は assertion の意図を変えない直し・docs は main 側の文に lane の承認済みの振舞いを件数・名・参照として入れる直し、超える例、判定は L3 の版と main 側から来た中身と最後の結果で行い lane の commit・直す commit も同じ線、L3 の版と Scope の食い違いは (ii) の合否に数えず気づいたら P3 の follow-up として報告し契約に及ぶ食い違いは (i) か通常の finding として中身に応じて扱う、基準の文は merge-evidence の 1 か所が持ち D-155 は要旨と参照〈D-146・D-154 と同じ置き方〉）。Why（L3 の版の lane の変更は同期の前からあり L3 の版の review の範囲、main 側の test・docs を lane の振舞いに合わせるのは同期で両側を落とさないための手で新しい振舞いを持ち込まない、置き場所で線を引くと TD-723 に反する）。Alternatives（今のまま回ごとに owner が裁定する〈2 回続けて止まった、TD-747〉、Scope の予定 path に同期の手を足す Gated Amendment を毎回出す〈同期のたびに計画の review が要る〉、(ii) の Scope の照らしを外す〈新しい中身が Scope の外でも通る〉、手の目的を closure に判定させる〈TD-723 でやめた〉、例外の文を (ii) の後ろに継ぎ足す〈D-127〉、helper に Scope の path を照らさせる〈中身の判定は機械にできない。D-154 の helper の境界〉）。Guarantee range（(1)(2) の当てはめは B の closure と owner の判断で、機械の保証でない。(2) の「assertion の意図を変えない」の線の上で reviewer が割れうる。そのときの扱いは今の B と同じ）。Compatibility（D-154 (4) が merge-evidence に置く B の基準のうち (ii) の Scope の照らしを置き換える。D-154 の (1)〜(3)・(5)〜(7)、D-146、D-144、D-145 (6) の C、集合・本数・helper は変えない。D-098 の安全の境界のうち「古い証拠の拒否」「独立 review」の運用を通常の packet・PR の経路で緩める向きに改訂した。変えないものは本 packet の前文のとおり）。owner の決定（TD の ID と要旨）。Revisit（(2) で合格した手の後に、merge の後でその test・docs が lane の振舞いと合わないと分かったとき。(1)(2) の線の上で closure がまた割れたとき。B の closure が新しい中身を (1)(2) と読んで見落としたとき）。
  - D-154 に `Superseded in part by: D-155（2026-11-17。(4) が merge-evidence に置く B の基準のうち (ii) の Scope の照らし〈lane の手の変更のうち、L3 の版の lane の変更と main 側の変更を重ねるものと、main 側の test・docs を lane の承認済みの振舞いへ合わせる直しのほかを Scope と照らす〉）` の形の 1 行だけを足す（D-144 の `Superseded in part by: D-154（…）` と同じ形。D-154 の本文は書き換えない）。
- 本 packet（本 commit）。

Non-scope: `scripts/pr-gate.py` と `scripts/tests/**`、`docs/DEV_WORKFLOW.md`、`AGENTS.md`、`docs/AGENT_OPERATING_MANUAL.md`、`docs/templates/**`、merge 済みの packet、`docs/backlog.md`（本 lane の項目は無い。`rg -n "TD-74[3-9]|TD-750" docs/backlog.md docs/Plans.md` は当たり無し〈exit 1〉。`docs` 全体に広げると、本 packet のほかに merge 済みの packet 2 件〈`docs/plans/2026-10-23-backup-offsite-runtime-b.md:255` の TD-743、`docs/plans/2026-10-29-stock-transfer.md:123` の TD-746。どちらもその回の owner の裁定の記録で、変えない〉に当たる）、`Plans.md`。

この変更で不要になるもの:

- TD-743・TD-746 のような、B の closure の (ii) の Scope の割れに owner がその回限りで裁定する手（新しい文で closure が決められる）。
- ほかに不要になる記述・条件: 調べた範囲（上の「調べ」の `rg`）で無い。(ii) の Scope の言い直しは merge-evidence の 1 か所だけで、helper の案内は B を名指す 1 行だけなので、合わせて消す文が無い。D-154 の Alternatives の「手の編集の目的を closure に判定させる（TD-723 でやめた）」は、本 lane が目的でなく中身で判定するので今も正しく、残す。

## Acceptance Criteria

- AC1: `git diff origin/main -- docs/agent-guidance/merge-evidence.md` の差分が B の (ii) の箇条の 1 行だけで、その行が「規則の形」の新しい文と同じ。(ii) の前の 2 文（両側を落とさない、test を消す・skip する・弱めない）と、最後の「Scope の外の…それが通るまで (ii) は不合格」の文の趣旨が残る。
- AC2（削除の検査）: 旧い文の固有の句「の中にある。Scope の外の変更は今の経路」が `rg -n "の中にある。Scope の外の変更は今の経路" docs/agent-guidance docs/decision-log.md` で 0 件（exit 1）、新しい文の固有の句「次の 2 つのほかは新しい中身として」が `docs/agent-guidance/merge-evidence.md` に 1 件。
- AC3: `docs/decision-log.md` の差分が、末尾の D-155 の節と、D-154 の節の `Superseded in part by: D-155（…）` の 1 行だけ。D-155 は基準の文を言い直さず、merge-evidence の B を名指す（`rg -n "次の 2 つのほかは新しい中身として" docs scripts --glob '!**/plans/**' --glob '!**/archive/**'` の当たりが merge-evidence の 1 行だけ。glob は本 packet の引用を外すため）。起票の時に隔離した copy（`git archive -o <tmp>.tar HEAD`〈`9064774`〉を展開し、merge-evidence の (ii) の 1 行を「規則の形」の新しい文に置き換えた版）で確かめた: 同じ `rg` の出力は `docs/agent-guidance/merge-evidence.md:165:  - (ii) lane の手の普通の正しさ。…` の 1 行（exit 0）、AC2 の旧い句の `rg` は exit 1。
- AC4: `git diff --stat origin/main -- scripts .github AGENTS.md docs/DEV_WORKFLOW.md` が空。
- AC5: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-11-17-b-scope-sync-hands.md` が ERROR なし（WARN は件数を報告に書く）、`bash scripts/check-workflow-git.sh` が exit 0。
- AC6: 計画の review と Final の review の報告（PR の comment）が、下の Test Plan の review の項目 R1〜R7 を `docs/agent-guidance/merge-evidence.md` の新しい (ii) の文に当てた行ごとの判定を書き、期待から外れた行が 0（文の判定は機械の test を置かない契約。下の Test Plan の末尾）。

## Design Readiness

- 引用する設計正本（節まで）: `docs/agent-guidance/merge-evidence.md` の「base同期だけでheadが変わる場合」の「lane の手と集合」と「manualの使い回し」の B（見る範囲・(i)・(ii)・不合格の後・本数・承認と証跡）、`docs/decision-log.md` D-154・D-144・D-146・D-098、`docs/DEV_WORKFLOW.md` の Workflow State（D-098 の安全の境界、承認の後の計画の変更）・Plan Packet Rules、`AGENTS.md` Working Rules の「継ぎ足さない」と Decision and Approval Boundaries。
- 必要な設計成果物: merge-evidence の (ii) の文と D-155 を updated in this PR（計画の固定の後の実装の commit で入れる）。
- plan にしかない durable な判断の昇格先: 「規則の形」の (1)(2) の線と lane の commit の扱い → merge-evidence の (ii) の文と D-155（Why・Alternatives）。調べの command と出力は本 packet に残す。
- 前提・制約と延期した design gap: (1)(2) の当てはめは B の closure と owner の判断で、機械で判定しない（helper の code を変えない。D-154 の helper の境界）。(2) の「assertion の意図を変えない」の線の上で reviewer が割れうることは残る（D-155 の Guarantee range と Revisit）。
- 絶対保証の例外と抜け道: 「新しい中身は今までどおり Scope と照らす」は closure の読みに依る。抜け道は、新しい振舞いを (2) の形（main 側の test の mock・件数、main 側の docs の件数・名・参照）に見せかけて持ち込むことで、文は「mock の返り値でその test が確かめる振舞いを lane の承認済みの振舞いに無いものへ変える」ことと「main 側の文の契約を変える」ことを新しい中身に当てて塞ぐ。消す向きの抜け道（Scope の外の main 側の文を消す）は「足す・変える・消す」で塞ぐ。(i) が L3 の項目への影響を別に見るので、(ii) を抜けても L3 の項目を変える変更は (i) で止まる。
- 判定: ready（未決の問いは下の「Owner Effort Budget」の確認だけ。TD-750 は方向の承認〈candidate〉で、計画の承認で確かめる）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| MG-D13 B (ii) | merge-evidence「manualの使い回し」の B の (ii) | (ii) の箇条の置き換え | 無し（文の判定。Test Plan の R1〜R7 と AC1・AC2） | 非対象 |
| MG-D13 B (i)・見る範囲・本数・承認 | merge-evidence「manualの使い回し」の B | 変えない | AC1（差分が (ii) の 1 行だけ） | 非対象 |
| MG-D13 集合 | merge-evidence「lane の手と集合」 | 変えない | AC4（helper と test の差分なし） | 非対象 |
| D-154 | decision-log D-154 (4) | `Superseded in part by: D-155` の 1 行 | AC3 | 非対象 |
| D-155 | decision-log（新規） | 本 lane の決定 | AC3・AC5 | 非対象 |
| D-146 | decision-log D-146（(i) の「影響なし」） | 変えない | AC1 | 非対象 |
| D-098 | DEV_WORKFLOW Workflow State の安全の境界（古い証拠の拒否・独立 review） | 緩める向きの改訂を本 packet・PR で行い、変えないものを前文に列挙 | R5・AC1・AC4 | 非対象 |
| D-127 継ぎ足さない | AGENTS.md Working Rules | (ii) の 1 箇条の置き換えで、例外の文を足さない | R6 | 非対象 |

隣の契約の sweep: merge-evidence の「reviewのclosure」（B の closure が監査の closure を兼ねる）、「L3 の使い回し C」・「docs の closure」（機械の経路）、DEV_WORKFLOW の L3 の表（B の行は Scope を言わない）、`AGENTS.md` の Ready の承認の範囲（helper の判定の出どころ）は、(ii) の文を引かないので触らない（上の「調べ」の `rg`）。

## Test Plan

規則の文だけを変える lane なので、test は review の項目と doc の検査。review の項目は、計画の review と Final の review が新しい文に当てて期待を確かめる。

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| MG-D13 B (ii) TD-743 | main 側が足した test（`src/features/daily-report-import/DailyReportImportRecordsPage.test.tsx`）に、lane に既にある RootLayout の hook を止める mock（`useAutoBackupCheck: () => undefined` の stub と `vi.fn()` 群。隣の route の test と同じ形）を足した手が、Scope の予定 path に無い、または返り値が lane の振舞いと違うとして不合格になる | review | R1 TD-743 の形は合格 | 新しい文が (2) を「Scope の予定 path の中」に限る、「main 側の test を変える手」を全部新しい中身に数える、または mock に lane の振舞いと同じ返り値を求める |
| MG-D13 B (ii) TD-746 | L3 の版で予定 path に無かった lane の変更（backlog）と main 側の変更を重ねた手が、不合格になる | review | R2 TD-746 の形は合格 | 新しい文が (1) の「L3 の版にあった lane の変更」を予定 path の中のものに限る |
| MG-D13 B (ii) 反例 | 同期の merge の中で、Scope に無い lane の新しい機能の test、または Scope に無い仕様の文を足した手が合格する | review | R3 Scope の外の新しい中身は不合格 | 新しい文が「同期の merge の中の手」を全部 Scope の外と数えない、または新しい assertion・新しい振舞いを (2) に読める |
| MG-D13 B (ii) 反例の境 | 不合格のはずの (a)・(b)・(e) が合格する、または合格のはずの (c)・(d) が不合格になる。(a) mock の返り値で、main 側の test が確かめる振舞いを lane の承認済みの振舞いに無いものへ変えた手（Scope の外）。(b) main 側の test に、Scope に無い振舞いを確かめる新しい assertion を足した手。(c) 同期の merge の中で、Scope の中の振舞いの新しい test を足した手（新しい中身だが Scope と照らして中）。(d) main 側の sweep の test の件数に、L3 の版で lane が足した画面を入れた手（(2) の件数の更新）。(e) main 側の function-design の文の契約を lane の契約に書き換えた手（新しい中身。Scope の外なら不合格） | review | R4 線の両側 | 新しい文が mock の返り値の制約か assertion の制約を落とす、docs の (2) を「件数・名・参照」に限らず契約の書換えを (2) に読める、件数の更新を新しい中身に数える、または新しい中身を Scope と照らさずに一律に不合格にする |
| MG-D13 B lane の commit | C に当たらない lane の commit が、同じ結果でも merge の中の手と別の判定になる。例: (a) 同期の後の 1 親の commit で main 側の test に lane の hook を止める mock を足す → 合格のはず。(b) Scope の外の新しい振舞いを足す commit → 不合格のはず。(c) 同期の merge で main 側の文を落とし、不合格の後の 1 親の直す commit でその文を戻した版（最初から正しく解いた merge と結果が同じ）→ 合格のはず（round 1 の C-P2-1） | review | R5 lane の commit も同じ線 | 新しい文が対象を「手で解いた path」に限る、lane の commit を Scope の照らしから外す、または親の数・手を入れた commit で判定を分ける（(c) が新しい中身として Scope と照らされる） |
| MG-D13 B (ii) 消す向き | L3 の後の lane の commit が、Scope の外の main 側の docs の文を消す変更が合格する（不合格のはず） | review | R7 消す向きも新しい中身 | 新しい文が「持ち込む」「足す」だけを新しい中身にし、消す・書き換える変更を照らしから外す |
| D-098・D-127 | (ii) の残り・(i)・本数・集合・承認・fail-closed の文が変わる、または例外の文が (ii) の後ろに継ぎ足される、または手の目的を closure に判定させる文になる | review・doc | R6 変えないものと置き換えの形 | 差分が (ii) の箇条の外に及ぶ（AC1）、旧い文が残って新しい文と並ぶ（AC2）、「目的」「辻褄合わせか」を判定の語にする |
| doc の形 | packet・docs の形の ERROR | CLI | D1 doc-consistency（全体と `--target plan`）、D2 check-workflow-git | link 切れ・節の欠け・Workflow State の形の誤り |

機械の test を置かない契約: (ii) の当てはめ（(1)(2) に当たるか、新しい中身が Scope の中か）は B の closure と owner の判断で、helper は判定しない（D-154）。実装の PR では、AC1〜AC3 の差分の確かめで、文が merge-evidence の 1 か所だけにあり、ほかに言い直しが無いことを見る。

L3: 置かない。operator の画面・app の build を変えず、規則の文だけを変える（DEV_WORKFLOW L3 Eligibility の (1) を満たさない）。

## Owner Effort Budget

- owner の手は、計画の承認の確認（TD-750 の candidate を confirmed にする。特に下の 1 点）と、実装の後の Ready の承認の 2 回の見込み。merge-evidence は gate の文書（`workflow=true`）なので、AGENTS.md の R0 の常設の委任に当たらず、Ready の承認は要る。
- C に当たらない lane の commit も同じ線で見ること（「規則の形」の「C に当たらない lane の commit の扱い」。TD-750 の文が「同期の手」と書く範囲を広げる所）: owner が決めた（repo 外の回答台帳 TD-751、2026-11-17。起票の後、計画の review の前）。
