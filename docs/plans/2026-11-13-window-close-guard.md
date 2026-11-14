# Plan Packet: native の window の close（×・Alt+F4）で取込み・取消・未保存の止めを横断して扱う

wave に属さない単独の lane。出発点は `docs/backlog.md` の項目「native の window の close で取込み・取消・未保存の 3 つの止めを横断して扱う」（owner の決定、repo 外の回答台帳 TD-458。着手の決定は 2026-11-13 の同じ台帳の TD-671。原文は台帳にあり、ここは ID と要旨だけ）。起票の commit（`65520303`）の後、owner の答え（同じ台帳の TD-689、2026-11-14。下の「owner の決定」）を入れ、設計正本を plan-first で直した（`5e359698`。下の Design Readiness）。計画の review round 1（2 本とも request changes。報告と裁定は PR #250 の issuecomment-6875162172）の裁定 #1〜#9 を 2026-11-14 に入れた（同日の Coordinator の裁定で、#9 は進行中の窓の外側の click を配線しない形に改め、O3 の確かめ方の文に「もう一度」を言い足した）。実装は計画の固定の後の run で、計画の固定は closure の review と Contract Probe（Design Readiness）の後。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の R3 の「状態を持つ画面・operator の業務の流れ」に当たり、Tauri の capabilities（`src-tauri/capabilities/default.json`）に権限を 1 つ足す: window の close を JS で止めると、閉じる操作そのものが JS の `destroy` の呼出しに移るので、誤れば店の PC で閉じられない窓ができる（下の「現状の調べ」(3)）。取込み・取消の 4 command の実行の thread も変える（属性だけ。DTO・wire・bindings・transaction の形は変えない。(10)）。R4 にしない理由: DB の schema・migration・DTO・bindings・backup・店の実データに触れない。「閉じる」で process が transaction の途中で終わりうるようになるが、取込みと取消は単一の transaction のままで、commit されずに終わった変更は SQLite が戻す（(5)。破壊的なデータの lifecycle を足さない）。backup の command の実行の thread は変えない（「案の比較」の M1 を採らない理由）。

## Goal

Goal Invariant:

### 最小完了条件

- 取込みの応答を待つ間（Z004・日報）と取消の応答を待つ間（Z004・日報）に × か Alt+F4 を押すと、command の途中でも、窓は閉じず、その場に「途中であること」と待てば閉じられることを示す確認の窓が出る。「待つ」で窓は残り、「閉じる」で閉じる（文言は UI-USW-D5、TD-689 の O2・O3。command の途中で窓を出すために、4 command を main thread の外で動かす。UI-USW-D5 の 8）。
- 未保存の入力がある画面（UI-USW-D3 の適用画面）で × か Alt+F4 を押すと、窓は閉じず、破棄の確認が出る。「破棄して閉じる」で閉じ、「編集を続ける」で入力が残る。
- 上のどれでもない時の × と Alt+F4 は、今と同じくすぐ閉じる。
- どの失敗の型（下の「失敗の型」F1〜F10）でも、閉じられない窓を作らない。止めの仕組みが壊れたときは「止めずに閉じる」側に倒れる（fail-open）。読み直しの後の登録の失敗だけは、× がそのまま効く形にできないので、アプリが自分で窓を閉じる（F7。例外と限界は Design Readiness の「絶対保証の例外と抜け道」）。

### 失敗定義

- 何もない時に × で閉じない（`destroy` の権限の欠け、handler の例外、listener の重なり）。
- 取込み・取消の応答を待つ間に × で窓が消え、取込みが済んだか分からない（取込み中にタブを切り替えた後の × を含む）。
- 取込み・取消の途中の × に窓が反応しない（main thread が塞がる）。
- 未保存の入力が × で確認なく消える。
- 確認の窓が出ているときの 2 回目の × で、確認の窓が重なる・閉じる・何も出ずに固まる。
- F5 などで webview を読み直した後に × で閉じなくなる。

### 非目的

- 3 つの止めの外の進行中の処理を close で止めること（TD-689 の O4。例はバックアップの作成・復元、PLU の書出し、棚卸しの保存で、全部を数えてはいない）。
- Windows のサインアウト・シャットダウン・タスクマネージャーでの終了を止めること（`CloseRequested` を経ない見込み。未確認）。
- route の移動と beforeunload の今の止め（UI-USW-D1・D2、55 §55.7）の振舞いを変えること（Z004 の取込み中のタブの lock だけは足す。Scope）。
- Rust の側に close の状態を持つこと（下の「案の比較」の C）。
- 4 command のほかの command を main thread の外に置くこと（`check_auto_backup` ほか backup の command を含む）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 現状の調べ

(1)〜(9) は 2026-11-13、main `2eee7e8b`。(5) の commit の前と後の書き分けと (10)〜(12) は 2026-11-14、`5e359698`（`git diff --stat 2eee7e8 5e35969 -- src src-tauri package.json` が空で、code は main `2eee7e8b` と同じ）。

(1) 今の止めの所在と状態の形（`rg -n "useBlocker|enableBeforeUnload|onCloseRequested|getCurrentWindow" src`、test を除く）:

| 止め | file:line | 条件（状態の形） |
|---|---|---|
| Z004 の取込み中 | `src/features/csv-import/hooks/useCsvImportFlow.ts:71-74` | reducer の `state.status === "importing"` |
| Z004 の取消の応答待ち | `src/features/csv-import/hooks/useRollbackCsvImport.ts:40-43` | `mutation.isPending`（結果の画面と CSV取込みの詳細の 2 つの入口に共通） |
| 日報の取込み中と取消の応答待ち | `src/features/daily-report-import/hooks/useDailyReportImportFlow.ts:206-210` | `blocking = state.status === "importing" \|\| rollbackMutation.isPending` |
| 未保存の入力 | `src/hooks/useUnsavedChangesWarning.ts:14-18`（`withResolver: true`） | 画面が渡す `isDirty: boolean`。使う画面は 6 つ（廃棄・手動販売・商品の登録編集・入庫・返品交換・閾値の設定。`src/hooks/unsaved-changes-guard-sweep.test.ts` の `APPLIED_PAGES`） |

- 4 つとも「boolean を返す関数」を `useBlocker` の `shouldBlockFn` と `enableBeforeUnload` に渡す同じ形。共有の状態は無く、`useBlocker` の直接使用は sweep の T16（同 file :92）が 4 file に限る。
- 共有の状態にまとめられる: 4 つの条件はどれも hook の中の boolean で、「どの種類（進行中 / 未保存）が今 active か」の集合にすれば足りる。種類で確認の窓の中身が変わる（進行中は待たせる、未保存は破棄を選ばせる）ので、種類を持つ。
- router の blockers（`@tanstack/history` 1.162.0 の `getBlockers`、`node_modules/@tanstack/history/dist/esm/index.js:294`）を読めば各 hook を触らずに済むが、`getBlockers` は公開の型 `RouterHistory`（同 `index.d.ts:14-31`）に無い内部の API で、種類も分からない。採らない（下の案 D）。
- 既にある module の 1 つの store の pattern: `src/features/daily-sales/components/PrintUnsupportedNotice.tsx:12-36`（module の変数 + `listeners` の Set + `useSyncExternalStore`、`ponytail:` comment 付き）。これに倣う（D-147 の 2 段目）。

(2) Tauri の版（`grep -n -A1 '^name = "tauri"$' src-tauri/Cargo.lock` → `2.11.1`、`package-lock.json` の `node_modules/@tauri-apps/api` → `2.11.1`、main の checkout の `node_modules/@tauri-apps/api/package.json` → `2.11.1`。wry は `0.55.1`）。

- JS の `onCloseRequested`（`node_modules/@tauri-apps/api/window.js:1632-1641`）: `tauri://close-requested` を listen し、`await handler(evt)` の後、`evt.isPreventDefault()` が false なら `await this.destroy()` を呼ぶ。
- Rust の側（`~/.cargo/registry/src/*/tauri-2.11.1/src/manager/window.rs:170-175`）: `WindowEvent::CloseRequested` で、その窓に JS の close-requested の listener が 1 つでもあれば `api.prevent_close()` してから event を送る。つまり **listener を 1 つでも登録すると、閉じるのは常に JS の `destroy` の仕事になる**（止めない時も）。
- `destroy` の権限: `core:window:allow-destroy` は `core:window:default` に無い（`tauri-2.11.1/permissions/window/autogenerated/reference.md` の Default Permission の一覧と :230）。今の `src-tauri/capabilities/default.json` は `core:default`・`core:window:allow-set-title`・`core:webview:allow-set-webview-zoom`・dialog・fs・opener だけで、`allow-destroy` は無い。listen は `core:default` の `core:event:default`（`allow-listen`・`allow-unlisten`、`permissions/event/autogenerated/reference.md`）で足りる。Coordinator の claim（destroy の権限が要る）は source で合っていた。差分は `"core:window:allow-destroy"` の 1 行。
- `close()` で閉じ直すと `CloseRequested` がもう一度出る（`src/window/mod.rs:1781` の doc）。止めた後に閉じるのは `destroy()`（同 :1786-1789、event を出さない）。

(3) 閉じられない窓になる型を source から確かめた:

- handler が例外を投げると `await handler(evt)` が reject し、`destroy` に届かない（上の `window.js:1636`）。Rust は既に `prevent_close` したので窓は残る。
- `destroy` が権限で拒まれても窓は残る（止めない時も含めて全部の close）。
- **読み直しの後に残る listener**: Rust の JS listener の表（`tauri-2.11.1/src/event/listener.rs:63` の `js_event_listeners`）から消すのは明示の `unlisten`（:239-251）だけで、page の読み直しで消す経路は source に無い（`rg -n "js_event_listeners" src` が :63・:81・:230・:241・:260・:281 だけ）。読み直した後は古い id の listener が Rust に残り、`has_js_listener` が true のまま close を止め、古い id への送り先は新しい page に無い（`src/event/mod.rs` の `event_initialization_script` は見つからない id を飛ばす）。新しい page が listener を登録し直せば、その handler が受けるので閉じられる。登録し直さない形（「止めが active の間だけ listener を置く」）は、読み直しの後に閉じられない窓になる。新しい page の登録が失敗した場合は (11)。
- 読み直しは店の PC で起こせる: WebView2 の browser accelerator keys（F5・Ctrl+R）は既定で有効（`wry-0.55.1/src/lib.rs:1687` `browser_accelerator_keys: true, // This is WebView2's default behavior`）で、tauri-runtime-wry 2.11.3・tauri 2.11.1 の source はこれを変えない（`rg -n browser_accelerator_keys` が wry の中だけ）。Windows の実機での読み直しの後の振舞いは未確認（L3-6 で見る）。

(4) React の StrictMode: `src/main.tsx:27` で `<React.StrictMode>` の中に `RouterProvider`。effect で listener を登録すると、開発では mount → cleanup → mount で 2 回走り、`onCloseRequested` は Promise で unlisten を返すので、cleanup の時点で unlisten がまだ無い（登録の Promise が後で解ける）。effect に置くなら「解けた時に cleanup 済みなら直ちに unlisten」の形が要る。module の初期化で 1 回だけ登録すれば StrictMode に関わらない（採る案）。止めの集合への出し入れは各 hook の `useEffect`（cleanup で外す）で行い、StrictMode の二重の effect でも出し入れが対になる。

(5) 閉じたときに DB は守られるか（commit の前と後で分けて書く。裁定 #8）: 取込みと取消は単一の transaction（`src-tauri/src/biz/csv_import_service/commit.rs:90`、同 `rollback.rs:111`、`daily_report_import_service/commit.rs:108`、同 `rollback.rs:28`。Z004 の `rollback.rs:74` は取消の前の読取りで、`finish` で必ず戻す）で、DB は WAL（`src-tauri/src/db/mod.rs:203`）。4 command を main thread の外に置く（(10)）と、「閉じる」で process が command の途中で終わりうる: 最後の窓が destroy されると event loop が `ControlFlow::Exit` にし（tauri-runtime-wry 2.11.3 `src/lib.rs:4309-4323`）、`RunEvent::Exit` の後始末（tauri 2.11.1 `src/app.rs:1100-1112` は resource と窓を片付けるだけで、async runtime の task を待たない）の後に tao が `std::process::exit` を呼ぶ（tao 0.35.3 `src/platform_impl/windows/event_loop.rs:230`）。worker の thread は待たれずに終わる。

- commit の前に終わった: `tx.commit()`（Z004 の取込み `commit.rs:221`、取消 `rollback.rs:113`、日報の取込み `commit.rs:243`、取消 `rollback.rs:31`）に届いていない変更は commit されていないので、次に開いたときには無い（SQLite の transaction の原子性。process が終われば rusqlite の `Transaction` の drop は走らないので、drop の rollback には頼っていない）。取込みは「取り込んだ記録」に行が無く、取消は取消済みにならない。起動し直してもう一度行えばよい（preview の cache は memory の中なので、file の選択からやり直す）。日報の SD の経路は transaction の前に写しを書く（`daily_report_import_service/commit.rs:72-84`、BIZ-08-D5）ので、DB の行の無い写しだけが残ることがある。今も transaction が失敗したときに起きる状態で、同じ bytes の写しは次の取込みで書き直さず、書きかけは最終の名前に残らない（29 §29.8.3 手順 2・4）。
- commit の後に終わった: 取込み・取消は済んでいる。transaction の外で best-effort に書く操作ログ（Z004 の取込み `commit.rs:242`、取消 `rollback.rs:127`、日報の取込み `commit.rs:258`、取消 `rollback.rs:44`）だけは欠けることがある。
- どちらでも半端な状態（行の一部だけ）は残らない。閉じた後の画面からはどちらになったか分からないので、店は次に開いた時に取込みの記録で確かめる（O2・O3 の判断材料。確かめる所の画面の文字は (9)）。process が途中で終わった後の DB が commit の前か後のどちらかになることは source を読んだ判断で、Windows の実機は Contract Probe の手順 3 で見る。

(6) 他の close の経路: Rust の側に `on_window_event`・`RunEvent`・`exit`・`restart` の処理は無い（`rg -n "on_window_event|CloseRequested|RunEvent|ExitRequested|\.restart\(\)|\.exit\(" src-tauri/src` が 0 件）。窓は 1 つ（`src-tauri/tauri.conf.json` の `windows` は 1 件で label の既定 `main`、capability の `windows: ["main"]`）。single-instance の plugin（`src-tauri/src/lib.rs:1437`）は 2 つ目の起動を前の窓へ寄せるだけで close に関わらない。

(7) root の error の fallback: `src/routes/__root.tsx:11` の `errorComponent` は `fullScreen` で RootLayout ごと差し替える。そのとき画面の hook は unmount され、止めの集合は空になる。確認の窓を RootLayout に置くので、fallback の間は窓を出す者がいない（F6 の手当てが要る）。

(8) 他の lane との重なり（2026-11-13、`gh pr diff N --name-only` の目視。#194 の `useAutoBackupCheck.ts` と 68 は 2026-11-14 に `git diff --stat HEAD origin/agent/backup-offsite-runtime-b` で足した）:

- #235（日報の取消の入口、未 merge）: `src/features/daily-report-import/hooks/useRollbackDailyReportImport.ts` を新設して日報の取消の `useBlocker` を flow から移し、55 §55.7 を書き換える。5 つ目の止めの hook になる。本 lane の実装は #235 の merge の後に始める（下の Scope の「順」）。docs の重なり（2026-11-14、`git diff --name-only origin/main origin/agent/daily-report-rollback-entry -- docs`）: 本 lane の plan-first が直す 55 §55.7 の「止める範囲」と §55.11、`docs/SCREEN_DESIGN.md` に #235 も触れ、#235 は §55.12（UI-07-D19）の「取消の実行と後」に「native の window の close は保証しない」の括弧を足す（同 branch の 55 の :573）。後に merge する側が同期で合わせ、#235 が先なら §55.12 の括弧を本 lane の同期で §55.11 と同じ形に直す（Design Readiness の表の条件つきの行）。
- #216（振替、未 merge）: `StockTransferPage.tsx` が `useUnsavedChangesWarning` を使い、`docs/UI_TECH_STACK.md` と `docs/function-design/52-ui-shared-layout.md` に触れる。hook の側で止めを持つので画面の追加の手当ては要らない。docs は別の節（本 lane は UI_TECH_STACK §6.11 と 52 §52.1 の表の RootLayout の行）で、末尾の更新履歴の追記は merge 順で両方を残す。
- #194（PC の外の控え、未 merge）: `src/components/layout/RootLayout.test.tsx` に触れる。本 lane も同じ test を触りうる（確認の窓の mount）。`src/features/backup-restore/useAutoBackupCheck.ts` とその test も大きく変える（確認の回に `checkOffsiteBackup` を足す）が、本 lane が 1 条件を足す `isHeld()` は #194 の版にも同じ形である（同 branch の file の :34・:144）。68 は UI-11b-D14〜D16 の行と冒頭の注を変え、本 lane が直す UI-11b-D13 の行は変えない（隣の行なので同期で衝突しうる）。`src-tauri/src/cmd/settings_cmd.rs` の source の検査の一覧に 5 command を足すので、本 lane はその test を触らない（Scope）。後に merge する側が同期で合わせる。

(9) 確かめる所の画面の文字（2026-11-14、`rg -n "取り込んだ記録を見る|日報の取込みの記録|取消済み" src --glob '!*.test.*'` と #235 の branch の `git show`）: Z004 は「売上データ取込み」（`CsvImportPage.tsx` の `PageHeader`）の「商品別CSV取込み（Z004）」タブの「取り込んだ記録を見る（取り消すときはここから）」（`src/features/csv-import/CsvImportPage.tsx:81`、入出庫履歴を種別「CSV取込み」で開く。状態の label「取消済み」は `src/features/inventory-records/types.ts:16`）。日報は main に記録の画面が無く、#235（`origin/agent/daily-report-rollback-entry`、head `fac35a17`）の「日報取込み」タブの「取り込んだ日報の記録を見る（取り消すときはここから）」（同 branch の `CsvImportPage.tsx:77`）が開く「日報の取込みの記録」（`DailyReportImportRecordsPage.tsx:67`、状態の列に「取消済み」:208）。日報の確かめ方の文は #235 の merge の後の画面の文字で、本 lane の実装は #235 の merge の後に始める（Scope の「順」）。

(10) 取込み・取消の command の実行の thread と、main thread を塞ぐ経路（裁定 #1）:

- 4 command は sync の `#[tauri::command] pub fn`（`src-tauri/src/cmd/csv_import_cmd.rs:114-116` `commit_csv_import`・`:202-204` `rollback_csv_import`、`daily_report_import_cmd.rs:205-207` `commit_daily_report_import`・`:299-301` `rollback_daily_report_import`）。tauri-macros の版は `grep -n -A1 '^name = "tauri-macros"$' src-tauri/Cargo.lock` → `2.6.3`。属性の無い command は `ExecutionContext::Blocking`（`~/.cargo/registry/src/*/tauri-macros-2.6.3/src/command/wrapper.rs:50`）で、IPC の handler の中でその場で実行される（`body_blocking`、同 :251。tauri 2.11.1 `src/webview/mod.rs:1909` の `run_invoke_handler` は custom protocol の handler の thread で動き、Windows では WebView2 の `WebResourceRequested`〈wry 0.55.1 `src/webview2/mod.rs:955`〉の thread = main thread）。repo の正本も同じ（43 §43.8.2「実行の thread」）。動いている間は `WM_CLOSE`（tao 0.35.3 `src/platform_impl/windows/event_loop.rs:1061` が `CloseRequested` にする）が処理されない。今の形では、取込み・取消の途中の × に確認の窓を出せない。
- `#[tauri::command(async)]` を付けると `ExecutionContext::Async`（同 `wrapper.rs:115-116`）になり、sync の関数のまま async runtime の task の中で実行される（`body_async`、同 :378・:389 の `respond_async_serialized` → tauri 2.11.1 `src/ipc/mod.rs:375` の `async_runtime::spawn`）。main thread は IPC を受けてすぐ戻る。前例は `inspect_backup`（`src-tauri/src/cmd/settings_cmd.rs:291`）と source の検査（同 :749 `test_inspect_backup_cmd_req901_runs_off_main_thread`）。
- 机上の確かめ（`git archive HEAD` の隔離した copy。tracked file は変えていない）: 4 command の属性を `#[tauri::command(async)]` に変えて `CARGO_TARGET_DIR=/tmp/shared-cargo-target cargo check --offline` → exit 0（`State<AppState>` と `tauri::AppHandle` を取る sync の関数のまま通る）。続けて `cargo run --offline --bin generate_bindings` → exit 0 で、`src/lib/bindings.ts` の sha256 は HEAD の版と同じ（`0ba02e13…`。wire は変わらない）。`scripts/check-command-drift.sh` の command の正規表現は `#[tauri::command(…)]` の形も拾う（同 :46）。窓が実際に反応するかは確かめていない（Contract Probe）。
- DB の Mutex: `AppState.db` は `Mutex<DbConnection>`（`src-tauri/src/lib.rs:1501`）で、4 command は transaction の間これを持つ。その間に main thread の sync の command が `state.db.lock()` を待つと、main thread が再び塞がる。取込み・取消の間に動きうる command を当たった:

  | 経路 | 調べ | main thread を塞ぐか |
  |---|---|---|
  | RootLayout の自動バックアップの確認 | `src/features/backup-restore/useAutoBackupCheck.ts:79-83` の 60 秒の interval が `commands.checkAutoBackup()` を呼ぶ。`check_auto_backup` は sync で DB の Mutex を取る（`src-tauri/src/cmd/settings_cmd.rs:240-252`） | 塞ぐ。取込みが tick に重なれば、operator の操作なしで起きる → 下の M2 |
  | Z004 の取込み中のタブの切替え | `src/features/csv-import/CsvImportPage.tsx:54` の `tabsLocked` は `isRollingBack` と日報の blocking だけで、Z004 の `importing` を含まない。切り替えると Z004 の flow が unmount される | 日報取込みタブは mount で command を呼ばない（`rg -n "useQuery\|commands\." src/features/daily-report-import --glob '!*.test.*'` の 7 件は `useQueryClient` の 2 件と mutation の中の `commands.*` の 5 件で、`useQuery` は無い）が、切り替えた先で「SD から読む」等を押せば sync の command が Mutex を待つ。それより前に、flow の unmount で close の止めが集合から外れ、command の途中の × が確認なく閉じる（sync の今は、command が終わるまで × が処理されないので起きない）→ Z004 の取込み中もタブを lock |
  | 取込み・取消の画面の中の他の操作 | Z004 の取込み中は `ImportingStep`（button なし）。取消の応答を待つ間は入口の button が disabled（`RollbackConfirmDialog.tsx:49`）で、確認の読取りの query は窓と一緒に unmount 済み、CSV取込みの詳細の query は取消の前に済んでいる。日報は取込み中・取消中ともタブが lock（55 §55.7） | 塞がない |
  | route の移動・preload | route の移動は `useBlocker` が止める。`defaultPreload: "intent"`（`src/lib/app-router.ts:78`）だが、route に `loader`・`beforeLoad` が無い（`rg -n "loader:\|beforeLoad" src --glob '!*.test.*'` が 0 件） | 塞がない |
  | TanStack Query の再取得 | `refetchOnWindowFocus: false`（`src/main.tsx:16`）。`refetchInterval` は 0 件（`rg -n "refetchInterval" src --glob '!*.test.*'`）。取込み・取消の後の invalidate は command が返った後 | 塞がない |
  | RootLayout のほかの呼出し | `setTitle` は tauri の window の command（async。DB を使わない）。ショートカットの窓・印刷の案内・Sidebar は command を呼ばない（`rg -n "commands\.[a-zA-Z]+" src/components/layout src/routes src/main.tsx` が 0 件） | 塞がない |
  | Rust の側 | DB を使うのは起動時の `check_auto_backup`（`src-tauri/src/lib.rs:128` の setup hook）だけ。timer・`on_window_event` は無い（(6)） | 塞がない |
  | #194（未 merge）の `checkOffsiteBackup` | 自動バックアップの確認と同じ回の中で呼ぶ。`#[tauri::command(async)]`（43 §43.8.3） | 塞がない。M2 で同じ回ごと飛ぶ |

- 「閉じる」の経路: `destroy()` は tauri の async の command（tauri 2.11.1 `src/window/plugin.rs:31-37`・`:93`）で、async runtime の worker で動く。runtime は tokio の multi thread（`src/async_runtime.rs:222-223` の `TokioRuntime::new()`、tokio 1.50.0 `src/runtime/runtime.rs:175` の `Builder::new_multi_thread()`）で、worker は既定で論理 core の数。4 command の 1 つが worker を 1 つ塞いでいても、ほかの worker で動く。論理 core が 1 つの PC では command が終わるまで待つ（店の PC の core の数は未確認）。

(11) 読み直しの後の登録の失敗（裁定 #2・#3）:

- 新しい page の登録が reject すると、古い listener が Rust に残ったまま（(3)）で、Rust が close を止め続け、新しい page には handler が無い。× で何も起きない窓が続く。起票の F1（「listener が無いので Rust は止めない」）は、起動して最初の page でだけ成り立つ。
- 古い listener を外せる公開の API は、登録した page が持つ `UnlistenFn` だけ。`onCloseRequested` は listener の id を返さず（`node_modules/@tauri-apps/api/event.js:71-81` の `listen` は id を closure に閉じ込める）、id が公開の型に出るのは event が届いた時の `Event.id`（`event.d.ts:25-29`）だけ。登録と解除の command は `plugin:event|listen`・`plugin:event|unlisten`（`event.js:44`・`:76`、Rust は tauri 2.11.1 `src/event/plugin.rs:13-31`）で、公開の型に無い。
- `getCurrentWindow()` は `window.__TAURI_INTERNALS__.metadata` を同期で読む（`node_modules/@tauri-apps/api/window.js:83-88`）。Tauri の外では同期の TypeError になる。起票の形 `installWindowCloseGuard(win = getCurrentWindow())` は、既定の引数の評価が関数の本体の前なので、本体を try/catch で包んでも捕まらない。

(12) Radix の AlertDialog の確かめ（裁定 #4・#6・#9。隔離した copy で `npm ci --ignore-scripts --offline` の後、使い捨ての test 3 本を `npx vitest run` → 3 passed。tracked file は変えていない。`@radix-ui/react-alert-dialog` 1.1.15・`@radix-ui/react-dismissable-layer` 1.1.11）:

- Esc: `onEscapeKeyDown` で `preventDefault` する窓と、しない窓のどちらでも、`fireEvent.keyDown` の戻り値は false（取消済み）だった。Radix は dismiss する時に自分でも `preventDefault` を呼ぶ（`react-dismissable-layer/dist/index.mjs:62-65`）ので、`defaultPrevented` は硬化の有無を判別しない。判別するのは `onOpenChange`（硬化した窓は 0 回、しない窓は `false` で 1 回）。
- 外側: overlay（`[data-slot="alert-dialog-overlay"]`。style は `pointer-events: auto`）の click・pointerdown と body の pointerdown で、`onOpenChange` は 0 回（`react-alert-dialog/dist/index.mjs:75-76` が `onPointerDownOutside`・`onInteractOutside` を content の prop の後で固定する。`docs/design-system/01-decision-rules.md` DSR-20 の「AlertDialog 系の制約」と同じ）。共有の `AlertDialogContent`（`src/components/ui/alert-dialog.tsx:39-60`）は overlay を中で描き、呼出し側から overlay へ prop を渡せない。進行中の窓の外側の click を「待つ」にするには共有の primitive に口を足す要があるので、足さずに、ほかの AlertDialog と同じく外側の click では閉じない形にする（Coordinator の裁定 2026-11-14 で裁定 #9 を改めた。D-147 の「要るか」と、DSR-20 の既にある決まりにそろえる。「待つ」は既定 focus で、Esc でも閉じる）。
- 2 枚の重なり: 後から開いた AlertDialog は DOM の後ろ（上）に入り、Esc は上の窓だけに届き、上の窓を閉じても下の窓は残った。

## 案の比較

| 案 | 形 | 閉じられない窓の型 | 見送る理由 |
|---|---|---|---|
| A（採る） | module の初期化で listener を 1 つだけ登録（`main.tsx` から 1 回）。handler は同期で、止めの集合の種類を読み、止めるときだけ `preventDefault` して確認の窓を開く。止めないときは何もせず、`onCloseRequested` の包みが `destroy` を呼ぶ | handler の例外（F2）・`destroy` の権限の欠け（F3）・JS が固まった時。前の 2 つは try/catch と capabilities の test で塞ぐ。読み直しの後は新しい page が登録し直すので閉じられる（登録が失敗したときは下の R2） | — |
| B | 止めが active の間だけ listener を置き、空になれば unlisten | 読み直しの後、古い listener が Rust に残り、新しい page は idle なので登録しない → 閉じられない（(3)） | 読み直しの型を作る。0↔1 の出入りの非同期の競合も増える |
| C | Rust の `on_window_event` で `CloseRequested` を受け、frontend が command で送る flag を見て `prevent_close` し、event で確認の窓を開かせる。閉じるのは独自の command | flag を page の読込みで戻せば読み直しに強い。JS が固まった時は同じ | command・DTO・bindings・Rust の状態が増え、未保存の入力の変化のたびに IPC が要る（D-147 の 4〜5 段目）。A で読み直しも閉じられるので要らない |
| D | listener の中で router の `history.getBlockers()` の `enableBeforeUnload` を評価し、各 hook を触らない | A と同じ | 内部の API（(1)）で、種類（進行中 / 未保存）が分からない。版上げで黙って壊れる |
| E | 画面・hook ごとに `onCloseRequested` を登録 | 登録の順と unlisten の漏れ、listener の重なりで 2 回 `destroy`・2 枚の窓 | backlog の種が既に棄てた形 |

A の handler の判定の順（同期、例外は全部捕まえる）:

1. 確認の窓を出す者（RootLayout の確認の窓の component）が mount されていなければ、止めない（F6）。
2. 止めの集合が空なら止めない。
3. 確認の窓が既に開いていれば `preventDefault` だけして何もしない（2 回目の ×、F5）。
4. 進行中が 1 つでもあれば進行中の窓、そうでなく未保存があれば未保存の窓を開き、`preventDefault`。
5. 1〜4 のどこかで例外なら `preventDefault` せずに返す（閉じる側に倒れる。F2）。

確認の窓の「破棄して閉じる」と進行中の「閉じる」（TD-689 の O2）は `getCurrentWindow().destroy()` を呼び、拒まれたら確認の窓を閉じ、error の Toast で「閉じられませんでした。もう一度 × を押してください。」（O5）を出す。

### 取込み・取消の途中の × に窓を出す手（裁定 #1）

| 案 | 形 | 見送る理由 |
|---|---|---|
| (i)（採る） | 4 command を `#[tauri::command(async)]` にする（関数は sync のまま。43 §43.8.2 の形）。属性 4 つと source の検査 | — 。型と bindings は机上で通った（(10)）。窓の反応は Contract Probe |
| (ii) | sync のまま、進行中の止めの振舞いを「途中の × は command が終わるまで効かず、終わった後に処理される」に変える | O2・O3 の答え（途中で「待つ」/「閉じる」を選べる）と変わるので owner に問い直す要がある。(i) が成り立つ間は採らない |

(i) の後に DB の Mutex を待って main thread が再び塞がる経路（(10) の表）の手:

| 案 | 形 | 見送る理由 |
|---|---|---|
| M1 | `check_auto_backup` も `#[tauri::command(async)]` にする | 属性 1 つで足り、下の競合も残らないが、backup の実行の thread を変える: 今は自動バックアップの間 main thread が塞がるので、× は backup が終わってから処理される。async にすると、止めの無い × で自動バックアップが途中で切れる経路ができる。backup の lifecycle は本 lane の外で（43 §43.8.2 が `create_backup`・`check_auto_backup` の実行の thread の見直しを範囲の外に置いた）、R4 にしない理由（Risk）とも合わない |
| M2（採る） | 進行中の止めが集合にある間は、`useAutoBackupCheck` がその回を呼ばない。`isHeld()` に store の同期の getter を 1 つ足し、state は持たずに回のたびに導く | — 。飛ばした回は 60 秒後の tick に回る。#194 の版でも `isHeld()` は同じ所にある（(8)） |
| M3 | Rust の sync の command を `try_lock` にして busy を返す | 全 command と UI の error の扱いに及ぶ |
| M4 | 全 command を async にする | 範囲が大きく、M1 と同じく backup を含む |

- Z004 の取込み中のタブ: `CsvImportFlowPanel` が上げる値を `isRollingBack` から `isImporting || isRollingBack` にして、2 つの `TabsTrigger` を disabled にする（日報取込みタブと Z004 の取消が既に持つ形、55 §55.7）。止めを mutation の寿命に結ぶ形（flow が unmount されても集合に残す）は、`useBlocker` と同じ条件という契約（T18）から外れ、画面に無い flow の窓が出るので採らない。
- 残る経路: 進行中に入る直前に送られた sync の command が後から main thread に届いた場合（tick の IPC が届く前に operator が「取り込む」を押した等）と、今後足される周期の command。main thread が塞がる間は × が効かず、4 command が終わった後に処理される。取込み・取消は済んでいるので、今の sync の形と同じ振舞いに落ちるだけで、データは守られる。UI-USW-D5 の 8 に規則（止めの間は呼ばないか、main thread の外に置く）を書いた。
- (i) は成り立つ（潰せない経路は見つからなかった）。

### 読み直しの後の登録の失敗の退避（裁定 #2）

| 案 | 形 | 見送る理由 |
|---|---|---|
| R1 | 登録の失敗でいつも `destroy()` を試す | 最初の page の失敗（古い listener が無く、× がそのまま効く）でも、アプリが起動のたびに自分で閉じる。F1 の fail-open を壊す |
| R2（採る） | `sessionStorage` の印で「読み直しの後」と分かるときだけ、登録の失敗で `destroy()` を試す。印は登録を試す前に書き、最初の page の失敗で消す | — 。公開の API と標準の機能だけで足りる。× がそのまま効く形にはならず、アプリが自分で閉じる（起動し直せば最初の page） |
| R3 | 古い listener の id を `sessionStorage` に持ち、読み直しの後に `unlisten` する | 登録が失敗しても × がそのまま効く（アプリは開いたまま）点は R2 より良い。ただし id を得るには `onCloseRequested` を使わずに `invoke("plugin:event\|listen")` を直接呼び、外すにも `invoke("plugin:event\|unlisten")` を直接呼ぶ要がある（(11)）。公開の型に無い内部の command で、案 D を棄てたのと同じ理由（版上げで黙って壊れる）。失敗の時だけでなく、いつもの登録まで内部の command に依ることになる |
| R4 | `pagehide` で `unlisten` する | 離脱の途中の IPC は届く保証が無い。`beforeunload` で外すと、離脱の確認で留まった時に止めが消える |
| R5 | 登録を retry する | 一時の失敗には効くが、続く失敗では閉じられない窓が残る |

- R2 の印は有無の 2 値: 無い（最初の page、または前の page の登録が失敗で終わった）/ 有る（前の page が登録を試み、成功したか、結果が出る前に読み直された）。「結果が出る前」を有る側に入れるのは、Rust が登録を済ませた後に読み直された場合に、閉じられない窓を残さないため。
- 保証の限界: `destroy()` も拒まれたとき（IPC ごと壊れている）と、`sessionStorage` を読めずに最初の page として扱ったときは、閉じられない窓が残る（タスクマネージャーが要る）。

## 失敗の型

| ID | 失敗 | 起きること | 手当て（倒れる向き） |
|---|---|---|---|
| F1 | 起動して最初の page での listener の登録の失敗（同期の例外・Promise の reject） | listener が無いので Rust は止めず、× で今どおり閉じる | 登録の本体を try/catch で包み、`console.warn` を出して `sessionStorage` の印を消す。止めは無いが閉じられる（fail-open） |
| F2 | handler の例外 | 包みが `destroy` に届かず窓が残る（(3)） | handler の全体を try/catch。例外なら `preventDefault` せず返す |
| F3 | `destroy` の拒否（権限の欠け・runtime の失敗） | 止めない時も含めて全部の close が失敗 | capabilities に `core:window:allow-destroy`。test で権限の行を見張る（T10）。確認の窓からの `destroy` の失敗は error の Toast で文を出す（O5） |
| F4 | 状態の読み違い（hook が unmount 後も集合に残る・StrictMode で二重に入る） | 何も無いのに止まり続ける | 集合の鍵は hook の instance ごと、出し入れは `useEffect` の cleanup で対にする（T8）。進行中が終われば確認の窓は自動で閉じる |
| F5 | 確認の窓の中の close（2 回目の ×・Alt+F4） | 窓が重なる・2 回目で閉じる | handler の 3（既に開いていれば止めるだけ）。2 回目の × で閉じる形（起票の時の O2 の c）は採らない |
| F6 | 確認の窓を出す者がいない（root の error の fallback、mount の前） | 止めたのに何も出ず閉じられない | handler の 1（出す者が mount されていなければ止めない） |
| F7 | 読み直しの後の古い listener | 新しい page が登録し直すまでの間の × は無反応。新しい page の登録が失敗すると、古い listener が close を止め続け、× がずっと無反応（(11)） | A は module の初期化で登録し直す（登録の後は閉じられる。L3-6）。登録が失敗したら、`sessionStorage` の印が有るときだけ `destroy()` でアプリが自分で閉じる（R2）。`destroy()` も失敗なら `console.warn` だけ（限界） |
| F8 | main thread が塞がる（sync の command が動いている・DB の Mutex を待っている） | × が処理されず、確認の窓も「閉じる」も出ない | 4 command を async にし（(i)）、進行中の止めの間は自動バックアップの確認を呼ばず（M2）、Z004 の取込み中もタブを lock する。残る経路では、4 command が終わった後に × が処理される（データは済んでいる） |
| F9 | 4 command が返らない（worker の thread の panic・固まり。panic の経路は未実測） | JS は進行中のままで、route の移動も close も止まり続ける | 進行中の窓の「閉じる」で閉じられる（O2 の (b) が塞ぐ型。main thread は空いている） |
| F10 | flow の unmount で止めが集合から外れる（Z004 の取込み中のタブの切替え・取込みの途中の読み直し） | command の途中の × が確認なく閉じ、取込みが済んだか分からない | タブは lock する。読み直しは離脱の確認（`enableBeforeUnload`）が先に出て、離れる側を選んだ後は止めない（抜け道。Rust の側に状態を持たない） |

## Scope

順: 設計正本の plan-first の修正（下の Design Readiness の表）は #235 の merge を待たずに本 lane の branch に入れた（2026-11-14。#235 は未 merge）。#235 に依る行（5 つ目の止めの hook、日報の確かめ方の文、§55.12 の括弧）は「#235 が merge されたら」の条件つきで書く（Coordinator の判断）。計画の review の closure と Contract Probe（Design Readiness）を経て計画を固定し、実装は #235 の merge の後に始める（実装の branch を #235 の後の main に同期してから）。

実装（計画の固定の後）:

- `src/lib/window-close-guard.ts`（新）: 止めの集合（鍵は hook の instance ごとの `Symbol`、値は種類 `WindowCloseGuardKind` = 進行中の `"z004-import" | "z004-rollback" | "daily-report-import" | "daily-report-rollback"` と未保存の `"dirty"`。O3 の文が取込みと取消、確かめる所が Z004 と日報で分かれるので、起票の時の `"busy"` 1 つを 4 つに分けた）、`useWindowCloseGuard(kind, active: boolean)`（`useEffect` で出し入れ、cleanup で外す）、確認の窓の状態（`null` か開いた時の種類。進行中が複数あれば集合に先に入った方）と `subscribe`・`getSnapshot`（`useSyncExternalStore` 用）、確認の窓を出す者の mount の登録、handler の判定（純関数、上の 1〜5）、進行中の種類が集合にあるかを返す同期の getter（`useAutoBackupCheck` が読む）。`@tauri-apps/api` を import しない（多くの画面の test が window を mock していないので、hook から Tauri を引かない）。`ponytail:` comment（module の 1 つの store。窓が 2 つ以上になれば窓の label ごとに持つ）。
- `src/lib/window-close-listener.ts`（新）: `installWindowCloseGuard(getWindow = getCurrentWindow)`。窓は本体の中で取る（既定の引数で `getCurrentWindow()` を呼ばない。(11)）。本体の全体を try/catch で包む。module の flag で 2 回目の呼出しを無視する。`sessionStorage` の印を読んでから書き、`onCloseRequested(handler)` を登録する。登録の失敗（同期の例外・Promise の reject）は `console.warn` を出し、前の印が有れば `destroy()`（その失敗も `console.warn` だけ）、無ければ印を消す。`sessionStorage` の読み書きの例外は印が無いものとして扱う（UI-USW-D5 の 2）。開発の HMR では `import.meta.hot?.dispose` で unlisten し、印を消す。
- `src/main.tsx`: `installWindowCloseGuard()` を render の前に 1 回。
- `src/components/layout/WindowCloseDialog.tsx`（新）: store を `useSyncExternalStore` で読む 1 つの AlertDialog。`onOpenChange` は 2 つの形とも「編集を続ける」/「待つ」と同じ処理（確認の窓を閉じる）へ配線する（DSR-20 の暗黙の硬化の禁止）。`"dirty"` は UI-USW-D2 と同じ硬化（`onEscapeKeyDown` の `preventDefault`。外側の click は AlertDialog の既定で閉じない）で、見出し「編集内容が保存されていません」、説明「このまま閉じると、入力した内容は破棄されます。」、「編集を続ける」〈既定 focus〉/「破棄して閉じる」〈destructive〉（TD-689 の O1）。進行中の 4 つは UI-USW-D5 の表の見出しと説明で「待つ」〈既定 focus〉/「閉じる」（O2・O3）。Esc は「待つ」と同じ（`onOpenChange` 経由）で、外側の click では閉じない（AlertDialog の既定のまま。overlay の click は配線せず、共有の `src/components/ui/alert-dialog.tsx` は変えない。DSR-20 の「AlertDialog 系の制約」）。進行中の種類が集合から外れたら窓を閉じる。「破棄して閉じる」と「閉じる」は `getCurrentWindow().destroy()`、拒まれたら窓を閉じて `toast.error("閉じられませんでした。もう一度 × を押してください。")`（O5）。mount の間だけ「出す者」として store に登録。route の窓（`UnsavedChangesDialog`）が出ている間に開いたときは、その上に出て、「編集を続ける」で自分だけ閉じる（route の blocker に触れない。UI-USW-D5 の 4）。既存の `src/components/patterns/UnsavedChangesDialog.tsx` を説明と操作の文の prop で使い回せるなら使い回す（D-147 の 2 段目。実装の時に props の形で決め、新しい component にする方が小さければそうする）。
- `src/components/layout/RootLayout.tsx`: `<WindowCloseDialog />` を Toaster と並べて mount。
- 止めの hook（各 1 行。`useBlocker` と同じ条件を渡す）: `src/hooks/useUnsavedChangesWarning.ts`（`"dirty"`, `isDirty`）、`src/features/csv-import/hooks/useCsvImportFlow.ts`（`"z004-import"`, importing）、`src/features/csv-import/hooks/useRollbackCsvImport.ts`（`"z004-rollback"`, `mutation.isPending`）、`src/features/daily-report-import/hooks/useDailyReportImportFlow.ts`（`"daily-report-import"`, importing）、#235 が merge されたら `src/features/daily-report-import/hooks/useRollbackDailyReportImport.ts`（`"daily-report-rollback"`, `mutation.isPending`。#235 の head `fac35a17` の同 file の `useBlocker` と同じ条件）。#235 が merge されないまま実装に入ることになれば（Scope の「順」の外。Coordinator が Gated Amendment で決める）、`useDailyReportImportFlow.ts` が今の 2 つの条件を 2 行に分けて持つ（`"daily-report-import"` に importing、`"daily-report-rollback"` に `rollbackMutation.isPending`）。hook の comment の「native の window の close は…保証しない」（`useRollbackCsvImport.ts:38`）と「Tauri webview beforeunload 連動」（`useCsvImportFlow.ts:70`）を直す。
- `src-tauri/src/cmd/csv_import_cmd.rs`・`src-tauri/src/cmd/daily_report_import_cmd.rs`: 4 command（`commit_csv_import`・`rollback_csv_import`・`commit_daily_report_import`・`rollback_daily_report_import`）の属性を `#[tauri::command(async)]` にする（関数は sync のまま。`#[specta::specta]`・登録・Mutex の取り方・BIZ の呼び方は変えない）。各 file の test に source の検査を 1 本（`settings_cmd.rs` の `test_inspect_backup_cmd_req901_runs_off_main_thread` と同じ判定: 属性が `tauri::command(async)` か、関数が `async fn`）。2 file で同じ判定になるので、判定の本体は `src-tauri/src/cmd/mod.rs` の `#[cfg(test)]` の関数 1 つに置く。`settings_cmd.rs` の既存の test は #194 が同じ一覧を触るので寄せない（`ponytail:` comment に、#194 の merge の後に寄せられることを書く）。
- `src/features/backup-restore/useAutoBackupCheck.ts`: `isHeld()` に「進行中の止めが集合にある」を足す（store の getter を読む 1 条件。module の state と世代番号は足さない。68 UI-11b-D13）。
- `src/features/csv-import/CsvImportPage.tsx`: `CsvImportFlowPanel` が上げる値を `isImporting || isRollingBack` にし、Z004 の取込み中も 2 つの `TabsTrigger` を disabled にする（55 §55.7。出所は owner の決定、repo 外の回答台帳 TD-701）。
- `src-tauri/capabilities/default.json`: `"core:window:allow-destroy"` を 1 行足す。ほかの権限（`allow-close` 等）は足さない。
- test（下の Test Plan）: `src/lib/window-close-guard.test.tsx`（新）、`src/lib/window-close-listener.test.ts`（新）、`src/components/layout/WindowCloseDialog.test.tsx`（新）、`src/hooks/unsaved-changes-guard-sweep.test.ts`（T18 を足す: `useBlocker(` を呼ぶ file は全部 `useWindowCloseGuard(` も呼ぶ。capabilities の行の検査も同じ file か新しい sweep に置く）、止めの hook の既存の test（`useUnsavedChangesWarning.test.tsx`・`useCsvImportFlow` / `useRollbackCsvImport` / `useDailyReportImportFlow` の test。#235 が merge されたら `useRollbackDailyReportImport` の test も）に集合の出し入れの 1 本ずつ、`src/features/backup-restore/useAutoBackupCheck.test.tsx`（T15）、`src/features/csv-import/CsvImportPage.test.tsx`（T20）、Rust の 2 file の test（T19）。
- 隣の test で見る所: `src/components/layout/RootLayout.test.tsx`・`src/lib/app-router.test.tsx`・`src/features/backup-restore/BackupRestorePage.flow.test.tsx` は `@tauri-apps/api/window` を mock している（`rg -n 'vi.mock\("@tauri-apps/api/window"' src`）。確認の窓は mount だけでは Tauri を呼ばないので、mock の形が変わらないことを確かめる。4 command の Rust の既存の test は関数を直接呼ぶので、属性の変更で変わらない。
- 生成物: `src/lib/bindings.ts` は変わらない（(10) の机上の確かめ。実装でも `cd src-tauri && cargo run --bin generate_bindings` の後に差分が無いことを確かめる）。test の名に REQ の ID を付けるので `docs/function-design/90-traceability.md` を再生成する（`cd src-tauri && cargo run --bin generate_traceability`。手で直さない）。
- docs（実装の PR）: Design Readiness の表で plan-first に入れた「未実装（PR #250 の実装で入る）」の注（UI_TECH_STACK UI-USW-D1・D4・D5、55 §55.7・§55.11、65 §65.5a、52 §52.1、41 §17.2、45 §45.4、68 UI-11b-D13、SCREEN_DESIGN の「Tauri capability」の行、decision-log D-153 の Status）を外し、41 §17.5 と 45 §45.4・§45.5 のシグネチャの属性を `#[tauri::command(async)]` に直し、各 file の更新履歴に実装の行。#235 が先に merge されていれば 55 §55.12 の括弧も同じ形に直す。backlog の項目を済みにする。

Non-scope: 4 command の属性と source の検査のほかの Rust の source（BIZ・DB・ほかの command。`check_auto_backup` ほか backup の command の実行の thread を含む）、DTO・bindings（wire は変わらない）、`src/components/patterns/useProductAddSuggest.ts` ほか画面の page（hook の側で止めを持つので page は触らない。`CsvImportPage.tsx` のタブの lock だけは触る）、TD-689 の O4 で外した他の進行中の処理、`tauri.conf.json`（browser accelerator keys を切ることは本 lane の外。読み直しは A で閉じられる）、Contract Probe の build の差分（本 lane の branch に入れない）。

## Acceptance Criteria

- AC1: `npx vitest run src/lib/window-close-guard.test.tsx src/lib/window-close-listener.test.ts src/components/layout/WindowCloseDialog.test.tsx src/hooks/unsaved-changes-guard-sweep.test.ts src/hooks/useUnsavedChangesWarning.test.tsx src/features/backup-restore/useAutoBackupCheck.test.tsx src/features/csv-import/CsvImportPage.test.tsx` と止めの hook の対象の test が全 PASS（exit 0）。既存の test を消さない・skip しない・弱めない。
- AC2: 自動 test で確かめる振舞い（Test Plan の T1〜T15・T18〜T20）: 止めない時は `preventDefault` しない、進行中・未保存で `preventDefault` して種類どおりの窓を開く、2 回目の close で窓を重ねない・差し替えない、handler の例外・出す者の不在・最初の page の登録の失敗で閉じる側に倒れる、読み直しの後の登録の失敗で `destroy` を呼ぶ、StrictMode で集合が対になる、「破棄して閉じる」が `destroy` を 1 回呼ぶ、`destroy` の失敗をその場で出す、route の窓の上に close の窓が出て「編集を続ける」で route の窓が残る、進行中の止めの間は自動バックアップの確認を呼ばない、Z004 の取込み中はタブを切り替えられない、capabilities に `core:window:allow-destroy` がある、`useBlocker(` を呼ぶ file が全部 `useWindowCloseGuard(` を呼ぶ、4 command が `#[tauri::command(async)]` である。
- AC3: `npm run typecheck`（`tsc --noEmit`）と `npm run lint` が exit 0。Rust は `cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings` と、2 つの cmd の module の test（T19 を含む）が exit 0。`cargo run --bin generate_bindings` の後に `git diff --exit-code src/lib/bindings.ts` が exit 0、`bash scripts/check-command-drift.sh` が exit 0、`cargo run --bin generate_traceability -- --check` が exit 0。`cargo check --release`（capabilities の検査は build で走る。`## L3` 節のある lane の Writer の完了条件、`CARGO_TARGET_DIR` は共有）が exit 0。
- AC4: `bash scripts/doc-consistency-check.sh` が ERROR なし、active な packet の `--target plan` が ERROR なし。
- AC5: Windows の native の close でしか確かめられない所は `## L3` の L3-1〜L3-6（× と Alt+F4 が `CloseRequested` を通ること、取込みの途中の × に実装の窓が出ること、`destroy` が権限で通ること、読み直しの後に閉じられること）。自動 test は Tauri の window を mock するので、これらの代わりにならない。

## Design Readiness

- 引用する設計正本（節まで）: `docs/UI_TECH_STACK.md` §6.11 UI-USW-D1〜D5、`docs/function-design/55-ui-csv-import.md` §55.7（止める範囲・UI-07-D18・解除）と §55.11 の「取消の実行と後」、UI-07-D15 (6)（#235 の後は UI-07-D19 も）、`docs/function-design/41-cmd-pos.md` §17.2（実行の thread）、`docs/function-design/45-cmd-daily-report-import.md` §45.4（実行の thread）、`docs/function-design/43-cmd-settings-log.md` §43.8.2（実行の thread の前例）、`docs/function-design/68-ui-backup-restore.md` UI-11b-D13（停止の理由）、`docs/function-design/65-inventory-record-traceability.md` §65.5a の「失敗」、`docs/function-design/52-ui-shared-layout.md` §52.1（RootLayout の責務）・§52.5（capability の記述の形）、`docs/SCREEN_DESIGN.md` の「Tauri capability」の行、`docs/decision-log.md` D-153、`docs/design-system/01-decision-rules.md` DSR-07・DSR-20（AlertDialog 系の制約）、`docs/design-system/02-component-catalog.md` の AlertDialog、AGENTS.md D-127・D-147。
- 必要な設計成果物（design-first が要る）: Design Phase Rules の表の「operator の流れ・日本語の文言・Windows native の操作」「Tauri command」「横断の durable な選択」に当たる。起票の時の正本は native の close を「保証しない」（UI-USW-D4）と書いていたので、2026-11-14 に本 lane の branch に plan-first で入れた（どの行にも「未実装（PR #250 の実装で入る）」の注を付け、実装の PR で外す。Scope の docs の行）:

  | 正本 | 節 | 変更 | 状態 |
  |---|---|---|---|
  | UI_TECH_STACK | UI-USW-D4 | 「非保証」を改め、native の close は beforeunload を経ず、UI-USW-D5 が止めると書いた | 入れた |
  | UI_TECH_STACK | UI-USW-D5（新。`rg -n "UI-USW-D5" docs src` が起票の commit の時に 0 件） | 案 A の契約: 止めの集合と 5 つの種類、listener は窓に 1 つで `main.tsx` から登録、handler の判定の順 1〜5、確認の窓の 2 つの形と文言（TD-689 の O1〜O3・O5）、fail-open と抜け道、`allow-destroy` の権限、範囲（O4）、`useBlocker` を使う hook は同じ条件で close の止めも持つ（T18 が強制）、棄てた案 B〜E と理由 | 入れた |
  | UI_TECH_STACK | UI-USW-D5 の 2・4・5・8（round 1 の是正） | 2: 登録の本体の try/catch と、`sessionStorage` の印で分ける登録の失敗の扱い（R2）。4: route の窓との重なり、進行中の窓は Esc と「待つ」で閉じ外側の click では閉じないこと、「閉じる」の後の commit の前と後、確かめ方の文の「もう一度」の言い足し。5: 閉じる側への倒れ方の 2 種類と抜け道 (a)〜(e)。8（新）: 4 command の実行の thread、進行中の止めの間の自動バックアップの確認とタブ、残る経路、今後の規則。棄てた案に R1・R3・(ii)・M1 | 入れた |
  | UI_TECH_STACK | UI-USW-D1 | hook が同じ `isDirty` で close の止めも持つことを 1 文 | 入れた |
  | 55 | §55.7 の「止める範囲」・§55.11 の「取消の実行と後」の括弧 | 止める範囲に native の close（UI-USW-D5）を足し、ガードの hook が同じ条件を `useWindowCloseGuard` に渡すこと（種類の名）と close の確認の窓の形を書いた。「Tauri の native の close は webview の `beforeunload` を経ない」の事実の文は残した | 入れた |
  | 55 | §55.7 の「止める範囲」・UI-07-D18 の段落（round 1 の是正） | 「閉じて失うのは結果の通知だけ」を commit の前と後で書き分け、取込みの確定と取消の command を main thread の外で動かすことを足した。Z004 タブは取込み中もタブを切り替えられないことを足した | 入れた |
  | 41 | §17.2 の「実行の thread」（新。round 1 の是正） | `commit_csv_import`・`rollback_csv_import` を `#[tauri::command(async)]` にする（43 §43.8.2 の形）。§17.5 のシグネチャの属性は実装の PR で直す | 入れた |
  | 45 | §45.4 の「実行の thread」（新。round 1 の是正） | `commit_daily_report_import`・`rollback_daily_report_import` を同じ形に。§45.4・§45.5 のシグネチャの属性は実装の PR で直す | 入れた |
  | 68 | UI-11b-D13 の「停止の理由」（round 1 の是正） | 理由を 3 つにし、state を持たずに導く 1 つ（進行中の止めが集合にある間は回を飛ばす）を足した | 入れた |
  | 65 | §65.5a の「失敗」の括弧（表に足した: 起票の表に無く、`rg -n "native の window" docs` で同じ「保証しない」の括弧が残っていた） | 止めに native の close を足した | 入れた |
  | 52 | §52.1 の表と接続点 | RootLayout の行に確認の窓の mount、接続点に `main.tsx` の listener の登録 | 入れた |
  | SCREEN_DESIGN | 「Tauri capability」の行 | `core:window:allow-destroy` と理由（止めた後に閉じる） | 入れた |
  | decision-log | D-153（番号は 2026-11-14 の main と open PR 8 本の branch の `docs/decision-log.md` の最後が D-152） | 横断の選択（JS の listener を module の初期化で 1 つ、Rust に状態を持たない、fail-open）と TD-689 の答え。round 1 の是正で (4) の読み直しの後の扱い、(7) 4 command の async、(8) 止めの間の規則と、Why・Alternatives・Compatibility・Guarantee range・Revisit を足した | 入れた |
  | backlog | 項目「native の window の close で…」の括弧（round 1 の是正、裁定 #7） | 「UI-USW-D4 の『別 backlog』の行き先」「確実に止める保証は無い」を、UI-USW-D5・D-153（未実装）を引く形に直した | 入れた |
  | 55 | §55.12 UI-07-D19 の「取消の実行と後」の括弧（#235 が足す。表に足した: #235 の branch の 55 の :573 が「native の window の close は保証しない」と書く） | #235 が merge されたら、§55.11 と同じ形（止めに native の close を含め、未実装の注）に直す | #235 の merge の後（本 lane の同期か実装の PR） |

- plan にしかない durable な判断の昇格先: 案の比較と失敗の型 → UI-USW-D5 の理由と D-153。packet にだけ残るのは調べの command と出力、Contract Probe の手順。
- 是正で足した契約の文の境界（4 観点）: (a) 整数の範囲 — 足した文は数を運ばない（印は有無の 2 値で、読めない値は「無い」。種類は enum。「集合にある」は 0 個か 1 個以上）。probe の 10 秒は probe の build だけ。(b) 0 の表示 — 表示に出す数は無い。0 に当たる場面は、集合が空（止めない）、印が無い・読めない（最初の page として扱う）、worker が空いていない（「閉じる」は command が終わるまで待つ）、取込み・取消が短くて × が間に合わない（L3 では未観測）。(c) 失敗の時点と順序 — 登録は「印を読む → 書く → 登録 → 失敗なら前の印で分ける → `destroy` の失敗は warn だけ」。自動バックアップの確認は tick の時点で集合を見る（tick の後に進行中に入った回は飛ばせない = 残る経路）。「閉じる」は `destroy` の拒否で確認の窓を閉じて Toast、通れば process が終わり DB は commit の前か後。タブの lock は `importing` に入った render から。(d) 単位や種類の混在 — 登録の失敗は 2 種類（同期の例外・Promise の reject）、閉じる側への倒れ方は 2 種類（× がそのまま閉じる・アプリが自分で閉じる）、「閉じる」は 2 つの対象（確認の窓・アプリの窓）、離れる手段は 4 種類（route の移動 = `useBlocker`、page の離脱 = `enableBeforeUnload`、タブ = `TabsTrigger` の disabled、native の close = UI-USW-D5）、thread は 3 種類（main・async runtime の worker・JS）を書き分けた。
- 前提・制約と延期した design gap: (i) × と Alt+F4 が Windows で `CloseRequested` を通ること、`destroy` が `allow-destroy` で通ること、読み直しの後に閉じられることは source から読んだ判断で、Windows の実機は未確認（L3-1〜L3-6。外れたら設計へ戻す）。(ii) サインアウト・シャットダウンは対象外（非目的）。(iii) 3 つの止めの外の進行中の処理は対象外（TD-689 の O4）。(iv) 日報の確かめ方の文が名指す「取り込んだ日報の記録を見る」は #235 の画面の文字で、実装は #235 の merge の後（Scope の「順」）。(v) 4 command を async にすると取込み・取消の途中の × に窓が反応すること、「閉じる」で process が transaction の途中で終わっても DB が commit の前か後のどちらかになることは source から読んだ判断（下の Contract Probe で実装の前に確かめる）。(vi) 店の PC の論理 core が 1 つなら、4 command の間の「閉じる」は command が終わるまで待つ（core の数は未確認。(10)）。
- 絶対保証の例外と抜け道: 「どの失敗でも閉じられない窓を作らない」の抜け道: (a) JS の thread が固まった時は handler が走らず、Rust が止めたまま閉じない（A・C とも同じ。タスクマネージャーが要る）。(b) F7 の登録までの短い間。(c) 読み直しの後の登録の失敗で `destroy()` も失敗したとき、またはそのとき `sessionStorage` の印を読めなかったとき（R2 の限界）。(d) main thread が塞がっている間（F8 の残る経路。4 command が終わるまで）。進行中の窓に「閉じる」を出す（TD-689 の O2）ので、取込み・取消の応答が返らない時（F9）も閉じられる。「進行中に × で閉じない」の抜け道: F1・F2・F6 の fail-open の時、読み直しの後の登録の失敗でアプリが自分で閉じる時（F7）、取込みの途中に読み直して離れた後（F10）、利用者が「閉じる」を選んだ時は閉じる（止めより閉じられることを優先する、Goal の 4 つ目）。
- この変更で不要になる記述・条件: plan-first で消した: UI-USW-D4 の「native close は非保証」と backlog への行き先、55 §55.7 の「native の window の close（×・Alt+F4）は…止めることを保証しない」と「別 backlog」、§55.11 と 65 §65.5a の「保証しない」の括弧。round 1 の是正で消した: UI-USW-D5 の 2 の「登録の失敗は `console.warn` だけ」、4 と 55 §55.7 と D-153 の「閉じて失うのは知らせ（結果の通知）だけ」、D-153 の「DB・command・bindings は変えない」、68 UI-11b-D13 の「理由は 2 つ」、backlog の項目の「UI-USW-D4 の『別 backlog』の行き先」「確実に止める保証は無い」。実装の PR で消す: `useRollbackCsvImport.ts:38` の同じ comment、`useCsvImportFlow.ts:70` の「Tauri webview beforeunload 連動」、#235 の `useRollbackDailyReportImport.ts` の「native の window の close は保証しない」の comment、backlog の項目（実装の merge で済み）、各正本の「未実装」の注。残すと決めたもの: `useBlocker` と `enableBeforeUnload`（route の移動と読み直しの止めは今の仕組みのまま。close の止めは別の経路で、置き換えではない）、UI-USW-D2 の route の窓の文言、sweep の T16、55 §55.7 の「本節の止めは全てこの範囲」（範囲に close を含めた形で残す）、43 §43.8.2 の「`create_backup`・`check_auto_backup` の実行 thread は変えない」（M1 を採らない）。
- 判定: ready（計画の review の closure に入れる）。round 1 の裁定 #1〜#9 を計画と正本に入れた。残る 1 行（55 §55.12 の括弧）は #235 の merge が条件で、本 lane の実装も #235 の merge の後に始める（Scope の「順」）ので、closure を止めない。計画の固定（Plan Commit を SHA にする）は、closure が P1/P2 = 0 で、かつ下の Contract Probe の手順 1〜4 が全部期待どおりだった後。

### Contract Probe（計画の固定の前提。裁定 #1）

(i) の 2 つの前提 — sync の command の間は × が処理されないこと、`#[tauri::command(async)]` にすると command の途中の × が処理され、「閉じる」で process が transaction の途中で終わっても DB が commit の前か後のどちらかになること — は source を読んだ判断で、Windows の実機で確かめていない。実装の前に owner の Windows の回が 1 回要る。結果は PR の comment に置く。1 つでも外れたら (i) を設計へ戻す（(ii) は owner に諮る）。

probe の build: 本 lane の branch に入れない（計画の固定の前の runtime の差分は先走りの検査が止める）。main から切った使い捨ての branch（PR にしない・merge しない）か手元の差分で、次の 4 点だけを足す。1 つの build で sync と async の両方を見られるよう、取込みの確定だけを async にする:

1. `commit_csv_import` の属性を `#[tauri::command(async)]` にする（`rollback_csv_import` は sync のまま）。
2. 時間を作る: Z004 の取込みの `tx.commit()` の直前（`csv_import_service/commit.rs:221`）と取消の `tx.commit()` の直前（同 `rollback.rs:113`）に `std::thread::sleep` を 10 秒（transaction の中）。fixture の行数では時間を作らない（行数と時間は未実測）。
3. `src-tauri/capabilities/default.json` に `core:window:allow-destroy`。
4. `src/main.tsx` に probe 用の listener: `getCurrentWindow().onCloseRequested` の handler が、× を受けるたびに画面の上端の帯に「probe: × を受けました（時刻）」を 1 行足して `preventDefault` する。直前の × から 3 秒以内の × だけは止めない（包みが `destroy` で閉じる）。

fixture（agent が probe の依頼と同時に渡す）: 開発用の DB で受理される合成の Z004 の CSV（CP932）を別の精算で 2 つ。L3-1 の fixture と同じ作りで、行は少なくてよい。店の実データを使わない。

手順（Windows の開発用の PC で `npm.cmd run tauri dev`。上から順に。手順 1 は × と Alt+F4 の両方で見る）:

| 手順 | 確かめること | 操作 | 期待 |
|---|---|---|---|
| 1 | async の command の途中の × に反応する | 「売上データ取込み」の「商品別CSV取込み（Z004）」タブで 1 つ目の CSV を選び、「取り込む」を押し、「取込み中…」が出ている間に × を 1 回 | 「取込み中…」のまま、すぐ帯に 1 行出る。窓は閉じない。そのまま待つと約 10 秒で取込みの結果の画面になる |
| 2 | sync の command の途中の × には反応しない | 手順 1 の結果の画面の「取り消す」→「取込みを取り消しますか？」の「取り消す」を押し、「取り消し中…」の間に × を 1 回 | 取消が終わるまで（約 10 秒）帯に行が足されず、窓も閉じない（title bar に「応答なし」が出てもよい）。取消が終わった後に 1 行足される |
| 3 | 途中で閉じても DB は commit の前 | 続けて 2 つ目の CSV を選んで「取り込む」を押し、「取込み中…」の間に × を続けて 2 回（1 回目で帯に 1 行、2 回目で閉じる） | 窓が消え、`tauri dev` の端末が終わる。もう一度 `npm.cmd run tauri dev` で普通に起動し、「取り込んだ記録を見る」に 2 つ目の取込みの行が無い（1 つ目の行は「取消済み」）。2 つ目の CSV をもう一度選んで取り込める |
| 4 | 待てば済む | 手順 3 の最後の取込みで、「取込み中…」の間に × を 1 回だけ押し、そのまま待つ | 帯に 1 行出て、約 10 秒で取込みの結果の画面になる。アプリを開き直すと「取り込んだ記録を見る」に行がある |

Linux（WSL の `npm run tauri dev`）で代えられるか: 代えられない。WSLg の窓の × は GTK の main loop を通り、Windows の `WM_CLOSE`・WebView2・Alt+F4 の経路を通らない。ただし同じ probe の build を先に WSL で流すと、OS に依らない部分（tauri-macros の sync / async の実行の thread、JS の listener への配送、`destroy` の後の process の終わり方、DB の戻り）が外れたときに、owner の Windows の回を使う前に分かる（予行）。予行の手順は上の表と同じで、`npm run tauri dev` の窓の × を人が押す。agent だけでは流せない: WSL に × を送る道具が無い（2026-11-14、`command -v wmctrl xdotool` が何も返さない。WSLg の表示は在る〈`DISPLAY=:0`〉）。Linux の IPC の handler の thread（wry の WebKitGTK の側）は読んでいないので、予行で手順 2 が再現するかは未確認。予行の結果は Windows の probe の代わりに数えない。

## owner の決定（Open Questions の答え）

起票の時に O1〜O5 を owner に問い、2026-11-14 に答えが出た（repo 外の回答台帳 TD-689。原文は台帳にあり、ここは ID と要旨だけ）。5 つとも起票の AI の推奨と同じ。未決の問いは残っていない。計画の review round 1 の是正では答えを変えていない: O2・O3 の振舞い（途中の × に「待つ」/「閉じる」の窓が出る）は、4 command を main thread の外に置くことで成り立つ（(10)）。「閉じる」は取込み・取消を commit の前で打ち切りうる（(5)）ので、O3 の確かめ方の文に、commit の前に閉じていた場合の手当てを言い足した（取込みは「記録に無ければ、もう一度取り込んでください。」、取消は「取消済みでなければ、もう一度取り消してください。」。全文は UI-USW-D5 の 4 の表）。O3 (a) の確かめ方の文の範囲の言い足しで、owner の答えの変更ではない（Coordinator の裁定 2026-11-14）。言い足した語は画面の文字に合わせた（`PreviewStep.tsx:112` と `DailyReportImportPage.tsx:508` の「取り込む」、`RollbackConfirmDialog.tsx:159` と `DailyReportImportPage.tsx:660` の「取り消す」、`src/features/inventory-records/types.ts:16` の「取消済み」）。

- **O1 未保存で × を押した時の窓の文言**: (a)。UI-USW-D2 に合わせ、見出し「編集内容が保存されていません」、説明「このまま閉じると、入力した内容は破棄されます。」、操作「編集を続ける」（既定 focus）/「破棄して閉じる」。→ UI-USW-D5 の 4、Scope の `WindowCloseDialog.tsx`、T3・T5、L3-3。
- **O2 取込み中・取消中に × を押した時に、閉じる道を出すか**: (b)。「待つ」を既定 focus にし、「閉じる」も出す。「閉じる」の前に、閉じると済んだかをこの画面で確かめられなくなることと確かめる所を書く。→ UI-USW-D5 の 4、55 §55.7 の止める範囲、T4、L3-1・L3-2。
- **O3 取込み中・取消中の窓の文言**: (a)。見出し「取込みの途中です」/「取消の途中です」と待つ文（「…が終わるまでお待ちください。終わると閉じられます。」）。O2 が (b) なので確かめ方の文を足す。確かめる所は Z004 が「売上データ取込み」の「商品別CSV取込み（Z004）」タブの「取り込んだ記録を見る」、日報が同じ画面の「日報取込み」タブの「取り込んだ日報の記録を見る」（#235 の画面。調べ (9)）。文の全文は UI-USW-D5 の 4 の表。→ 種類を進行中の 4 つに分けた（Scope の `window-close-guard.ts`）、T4、L3-1・L3-2。
- **O4 範囲**: (a)。取込み・取消・未保存の 3 つの止めだけ（TD-458）。ほかは要ると分かった lane で `useWindowCloseGuard` の 1 行で足す。→ UI-USW-D5 の 7、非目的。
- **Z004 の取込み中のタブの切替え**（round 1 の是正で出た問い。repo 外の回答台帳 TD-701、2026-11-14）: (a) 止める。Z004 の取込み中は「日報取込み」「商品別CSV取込み（Z004）」のタブの切替えも止め、取込み中に他の画面へ移れないのと同じ扱いにする。→ 55 §55.7 の UI-07-D18 の段落、Scope の `CsvImportPage.tsx`、T20、L3-1。
- **O5 「破棄して閉じる」「閉じる」で閉じられなかった時の文**: (a)。確認の窓を閉じ、「閉じられませんでした。もう一度 × を押してください。」を出す（出し方は error の Toast。RootLayout の Toaster が既にある）。→ UI-USW-D5 の 4、T13。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| UI-USW-D5（新） | UI_TECH_STACK §6.11 | `window-close-guard.ts`・`window-close-listener.ts`・`WindowCloseDialog.tsx`・`main.tsx`・`RootLayout.tsx` | T1〜T9・T11〜T14 | L3-1〜L3-6 |
| UI-USW-D5 の 8（実行の thread）・CMD-07・CMD-12 | UI_TECH_STACK §6.11、41 §17.2、45 §45.4 | `csv_import_cmd.rs`・`daily_report_import_cmd.rs` の 4 command の属性 | T19 | Contract Probe（計画の固定の前）、L3-1 |
| UI-11b-D13（停止の理由の 3 つ目） | 68 | `useAutoBackupCheck.ts` の `isHeld()`、`window-close-guard.ts` の getter | T15 | 非対象（60 秒の tick と取込みを重ねる手順は L3 Eligibility (3) に当たる） |
| UI-USW-D4（改） | UI_TECH_STACK §6.11 | 上と同じ（「非保証」を外す） | T2・T3 | L3-1〜L3-4 |
| UI-USW-D1 | UI_TECH_STACK §6.11 | `useUnsavedChangesWarning.ts` の 1 行 | T12（hook）・T18 | L3-3 |
| UI-USW-D2 | UI_TECH_STACK §6.11 | route の窓は変えない。close の窓が同じ硬化を持ち、route の窓の上に出る | T3・T5（Esc で閉じない）・T14 | L3-3 |
| UI-USW-D3 | UI_TECH_STACK §6.11 | 変えない（適用画面は同じ。hook の側で持つので画面を足すと自動で入る） | 既存の T17 | 非対象 |
| 55 §55.7 止める範囲・UI-07-D18 | 55 §55.7 | `useCsvImportFlow.ts`・`useRollbackCsvImport.ts` の 1 行ずつ、`CsvImportPage.tsx` の Z004 の取込み中のタブの lock | T12（hook）・T20 | L3-1・L3-2 |
| UI-07-D15 (6)（#235 が merge されたら UI-07-D19 も） | 55 §55.7（#235 が merge されたら §55.12 も） | `useDailyReportImportFlow.ts` の `"daily-report-import"`（#235 が merge されたら `useRollbackDailyReportImport.ts` の `"daily-report-rollback"`） | T12（hook）・T18 | L3-1・L3-2（日報の項目は置かない。Z004 と同じ store と窓を通る） |
| 55 §55.11 取消の実行と後 | 55 §55.11 | 文の括弧だけ | — | 非対象（docs） |
| 65 §65.5a の「失敗」 | 65 §65.5a | 文の括弧だけ（CSV取込みの詳細の取消は `useRollbackCsvImport` を通るので、上の §55.7 の行と同じ実装） | T12（hook） | L3-2 |
| 52 §52.1 | 52 | `RootLayout.tsx`・`main.tsx` | T6・T1 | L3-4 |
| capabilities（`core:window:allow-destroy`） | SCREEN_DESIGN「Tauri capability」・UI-USW-D5 | `src-tauri/capabilities/default.json` | T10 | L3-4（何もない時に閉じる） |
| DSR-20（dismiss の本則・硬化・AlertDialog 系の制約） | design-system 01 | `WindowCloseDialog.tsx`（`onOpenChange` の配線、未保存の Esc の硬化、外側の click は配線しない。共有の `alert-dialog.tsx` は変えない） | T4・T5 | 非対象 |
| D-153 | decision-log | — | — | 非対象（docs） |

隣の契約の sweep: UI-EB-D1（route の error の fallback）は F6 で触れる（fallback の間は止めない）。行は UI-USW-D5 の handler の 1 に含めた。54 のショートカット（Ctrl+/）は close に関わらない（除外）。43 §43.8.2（backup の command の実行の thread）は変えない（M1 を採らない）。71 §71.8（60 秒の確認）は、進行中の止めの間の回が飛ぶだけで、起動時の確認と次の tick は変わらない。

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| UI-USW-D5 | listener が 2 つになる | unit | T1 `installWindowCloseGuard` を 2 回呼んでも `onCloseRequested` は 1 回 | module の flag を外す |
| UI-USW-D5・D4 | 何もない時に止める | unit | T2 確認の窓の component を mount した上で、集合が空なら `preventDefault` を呼ばない | 判定の 2 を外す・常に止める |
| UI-USW-D5・D2 | 未保存が確認なく消える | unit + component | T3 `"dirty"` で `preventDefault` し未保存の窓（見出し「編集内容が保存されていません」・説明「このまま閉じると、入力した内容は破棄されます。」・既定 focus が「編集を続ける」）。「破棄して閉じる」で `destroy` 1 回、「編集を続ける」で `destroy` 0 回・窓が閉じ集合は残る | 種類の判定・文言・既定 focus・操作の結線を外す |
| UI-USW-D5・§55.7・DSR-20 | 取込み中に消える・外側の click で確認が消える | unit + component | T4 進行中の 4 つの種類ごとに `preventDefault` し、UI-USW-D5 の表の見出しと説明（確かめる所と「もう一度」の文まで）の窓（既定 focus が「待つ」）。`"dirty"` と進行中が同時なら進行中の窓。「待つ」と Esc で窓が閉じ `destroy` 0 回、集合からその種類が外れると窓が自動で閉じる。「閉じる」で `destroy` 1 回。外側の click（overlay `[data-slot="alert-dialog-overlay"]` の pointerdown と click）では窓が残り、`onOpenChange` が呼ばれない（1 件） | 進行中の優先・種類ごとの文言・自動で閉じる結線・「閉じる」の結線・Esc の `onOpenChange` の配線を外す。外側の click で閉じる配線を足す |
| UI-USW-D2・DSR-20 | Esc で確認を飛ばす | component | T5 未保存の窓（`onOpenChange` を「編集を続ける」と同じ処理へ配線してある）で Esc を押しても、窓が残り、store の窓の状態が変わらず、`destroy` 0 回。`defaultPrevented` は oracle にしない（Radix が dismiss の時に自分でも立てる。(12)） | `onEscapeKeyDown` の `preventDefault` を外す（外すと `onOpenChange(false)` が届いて窓が閉じる） |
| UI-USW-D5・UI-EB-D1 | 出す者がいないのに止める（F6） | unit | T6 集合を active にしたまま確認の窓の component を unmount すると、`preventDefault` しない | 判定の 1 を外す |
| UI-USW-D5 | 2 回目の close で重なる・差し替わる・閉じる（F5） | unit | T7 `"dirty"` の窓が開いている間に進行中の種類を集合に入れてから 2 回目の close → `preventDefault` し、窓の状態は `"dirty"` のまま（進行中に差し替わらない）、`destroy` 0 回 | 判定の 3 を外す（外すと判定の 4 が走り、窓が進行中に差し替わる） |
| UI-USW-D5 | StrictMode で集合が残る・同じ種類の止めが互いを消す（F4） | component | T8 `<React.StrictMode>` の中で `useWindowCloseGuard("dirty", true)` を 2 つ mount し、片方だけ unmount（または false）しても集合に `"dirty"` が残って close が止まり、両方外れると空で止まらない | cleanup を外す・鍵を種類で共有にする |
| UI-USW-D5 | handler の例外で閉じない（F2） | unit | T9 store の読みを throw にしても handler は reject せず `preventDefault` しない | try/catch を外す |
| capabilities | `destroy` の権限が無い（F3） | sweep | T10 `src-tauri/capabilities/default.json` の `permissions` に `core:window:allow-destroy` がある | 行を消す |
| UI-USW-D5 | 登録の失敗で app が止まる（F1）・読み直しの後に閉じられない（F7） | unit | T11 `sessionStorage` は test ごとに空にし、page の読み直しは module の読み直し（`vi.resetModules()`）で表す。(a) 印が無い + `onCloseRequested` の reject → throw せず `console.warn`、`destroy` 0 回、印が消える。(b) 印が無い + 窓の取得の同期の throw → throw せず `console.warn`、`destroy` 0 回。(c) 1 回目の登録が成功した後に module を読み直し、2 回目の登録が reject（古い listener の残り + 新しい登録の reject）→ `destroy` 1 回。(d) (c) で `destroy` も reject → throw せず `console.warn`。(e) 読み直しの後の登録が成功 → `destroy` 0 回。(f) `sessionStorage` の読みが throw + 登録の reject → `destroy` 0 回 | 本体の try/catch を外す・窓を既定の引数で取る（(b)）。印の読み書きを外す・`destroy` の枝を外す（(c)）。最初の page でも `destroy` する（(a)）。`destroy` の catch を外す（(d)） |
| UI-USW-D1・§55.7 | hook が集合に入れない・違う種類を入れる | hook | T12 各止めの hook で、`useBlocker` の mock の値でなく実際の条件（state・mutation）が true の間は集合にその種類が入り、false で外れる（hook ごとに 1 本）。日報の flow は取消の pending の間に `daily-report-import` を入れない | hook の 1 行を外す・条件を変える・種類を取り違える |
| UI-USW-D5 | `destroy` の失敗が黙る | component | T13 「破棄して閉じる」と「閉じる」の `destroy` が reject すると、窓が閉じ error の Toast で「閉じられませんでした。もう一度 × を押してください。」が出る | 失敗の枝を外す |
| UI-USW-D5・D2 | route の窓と close の窓の重なりで route の確認が消える・2 回 resolve する | component | T14 route の窓（`UnsavedChangesDialog`、blocked）が出ている間に close → close の窓が後ろ（上）に出る。「編集を続ける」で close の窓だけが消え、route の窓は残り、blocker の `proceed`・`reset` は 0 回、`destroy` 0 回。もう一度 close して「破棄して閉じる」で `destroy` 1 回 | close の窓の「編集を続ける」を route の blocker に結ぶ・route の窓が出ている間は close を止めない |
| UI-11b-D13・UI-USW-D5 の 8 | 取込み・取消の間に `checkAutoBackup` が main thread を塞ぐ（F8） | hook | T15 進行中の種類が集合にある間の tick は `checkAutoBackup` を呼ばず、外れた後の tick で呼ぶ。`"dirty"` だけのときの tick は呼ぶ | `isHeld()` の条件を外す・`"dirty"` でも止める |
| UI-USW-D5 | 新しい止めの hook が close を止めない | sweep | T18 `useBlocker(` を呼ぶ file は全部 `useWindowCloseGuard(` も呼ぶ | 1 つの hook から外す |
| UI-USW-D5 の 8・CMD-07・CMD-12 | 4 command が main thread で動く（F8） | Rust の source の検査 | T19 `csv_import_cmd.rs` の `commit_csv_import`・`rollback_csv_import` と、`daily_report_import_cmd.rs` の `commit_daily_report_import`・`rollback_daily_report_import` が `#[tauri::command(async)]`（または `async fn`）。file ごとに 1 本（REQ-401） | 属性を `#[tauri::command]` に戻す・4 つのどれかを一覧から外す |
| 55 §55.7 UI-07-D18 | Z004 の取込み中にタブを切り替えて止めが外れる（F10） | integration（`CsvImportPage.test.tsx` の今の flow の mock） | T20 Z004 の flow が `isImporting` の間は 2 つのタブが disabled で、切り替えられない（REQ-401。既存の取消の応答を待つ間の test と同じ形） | 上げる値から `isImporting` を外す |

状態の遷移: initial（集合が空）→ dirty / 進行中（T3・T4・T12）→ 窓が開く（T3・T4）→ 2 回目の close（T7）→ 続ける・待つ（T3・T4）→ 進行中の終わりで自動で閉じる（T4）→ 破棄・閉じる（T3・T4）→ 失敗（T13）。登録: 最初の page → 読み直し → 登録の成功 / 失敗（T11）。negative: T2・T6・T9・T11 の (a)(b)(f)・T15 の `"dirty"`。mutation の注入は「Would fail if」の列を実装に注入して red を確かめる（未実測。Contract Audit で）。T5 の oracle と外側の click の既定（閉じない）と 2 枚の重なりは、(12) の使い捨ての test で先に確かめた。

## L3

Windows の店の PC と同じ形の native の build で、各項目を × と Alt+F4 の両方で見る（2 通り × 項目）。fixture は agent が L3 の依頼と同時に渡す（取込みが数秒かかる合成の Z004 の CSV〈CP932〉を別の精算で 2 つ。同じ精算の 2 回目は §55.11 の順 4 の拒否で止まるため。行数と時間は未実測）。取込み・取消が短くて × が間に合わなかった回は「未観測」と返す（PASS にも FAIL にも数えない。窓の中身と結線は T4・T12、command の途中の × の反応は計画の固定の前の Contract Probe が見ている）。Contract Probe は probe 用の listener の build で、実装の窓を見る L3 の代わりにならない。

- L3-1 取込み中: サイドバーの「売上データ取込み」の「商品別CSV取込み（Z004）」タブで合成の CSV を選び、プレビューの「取り込む」を押し、「取込み完了まで他画面に移れません」が出ている間に × / Alt+F4 → 窓は閉じず、「取込みの途中です」と、待つ文・確かめ方の文（「…「商品別CSV取込み（Z004）」タブの「取り込んだ記録を見る」で確かめてください。記録に無ければ、もう一度取り込んでください。」）の窓が出て、focus は「待つ」にある。取込み中は 2 つのタブが押せない。「待つ」を押すか、取込みが終わると窓が消え、もう一度 × で閉じる。fixture の 2 つ目の CSV で同じく取り込み、同じ窓で「閉じる」を押すと閉じる。開き直して「取り込んだ記録を見る」を開くと、その取込みが済んでいれば行があり、済む前に閉じたなら行が無い（どちらでも PASS。行が無ければ同じ CSV をもう一度取り込める）。
- L3-2 取消中: 「売上データ取込み」の「商品別CSV取込み（Z004）」タブの「取り込んだ記録を見る（取り消すときはここから）」→ 入出庫履歴の L3-1 の取込みの行の「詳細を見る」→「この取込みを取り消す」→「取込みを取り消しますか？」の「取り消す」→「取り消し中…」の間に × / Alt+F4 → 窓は閉じず、「取消の途中です」と、待つ文・確かめ方の文（「…「取り込んだ記録を見る」で、取消済みかを確かめてください。取消済みでなければ、もう一度取り消してください。」）の窓が出る。取消が短くて × が間に合わなければ「未観測」と返す。
- L3-3 未保存: サイドバーの「入庫記録」で商品を検索して明細を 1 行足し、保存せずに × / Alt+F4 →「編集内容が保存されていません」と「このまま閉じると、入力した内容は破棄されます。」が出る。「編集を続ける」で入力が残り、もう一度 × で同じ窓。「破棄して閉じる」で閉じる。窓が出ている間にもう一度 × を押しても窓は 1 枚のまま。
- L3-4 何もない時: ホームで × / Alt+F4 → 確認なくすぐ閉じる。
- L3-5 保存の後: 「入庫記録」で保存して「入庫を保存しました」が出た後（入力が無い状態）に × → すぐ閉じる。
- L3-6 読み直しの後: 「入庫記録」で明細を 1 行足し、F5 を押して出た WebView2 の離脱の確認（文は WebView2 の既定で、アプリは変えられない。UI-USW-D2）で離れる側を押す → 読み直した画面で × → すぐ閉じる（読み直した直後にアプリが自分で閉じたら FAIL で、登録の失敗の退避〈F7〉が働いている）。

## 計画の review に問う点

round 1（発注 878、`5e359698`）で答えが出たもの（報告の全文と裁定は PR #250 の issuecomment-6875162172。番号は起票と plan-first の Writer の問い）:

1. 読み直しの後も閉じられる根拠 → 新しい登録が成功する限り正しい（2 本）。登録が失敗する場合は裁定 #2 で、R2 を入れた。
2. route の窓と close の窓の重なり → close の窓を上に出す（裁定 #6）。UI-USW-D5 の 4 と T14 に入れた。
3. T18 が条件まで見張らない点 → T12 で足りる（2 本）。T12 は `useBlocker` の mock でなく実際の条件から集合を見る形にした。
4. L3-1・L3-2 を L3 に置くか → 置く。短くて押せなかった回は未観測で PASS と分ける（裁定 #5）。
5. `UnsavedChangesDialog` を使い回すかを実装の時に決めること → それでよい（2 本）。
6. 種類を 5 つに分けたこと → 5 つが最小（2 本）。
7. 進行中の窓の Esc と外側の click = 「待つ」、O5 = Toast → 妥当。外側の click は AlertDialog が中で無効にするので配線が要る（裁定 #9）。その後 Coordinator の裁定（2026-11-14）で改めた: 配線は足さず、進行中の窓は Esc と「待つ」で閉じ、外側の click では閉じない（(12)。T4 は外側の click で窓が残ることを見る）。
8. #235 の待ち → 維持する。大きく遅れたら、main での日報の確認先を調べてから Gated Amendment（2 本）。

closure に問う点（round 1 の是正の Writer、2026-11-14）:

1. (i) と M2 とタブの lock で、取込み・取消の間に main thread を塞ぐ経路を潰し切れているか（(10) の表に漏れは無いか）。M1（`check_auto_backup` を async）を棄てた理由は妥当か。
2. Z004 の取込み中のタブの lock は、今は切り替えられる operator の操作を止める変更になる。(i) の帰結（flow の unmount で止めが外れる）として本 lane に入れた → owner が決めた（TD-701、2026-11-14。「owner の決定」の節）。止める。
3. R2 は、読み直しの後の登録の失敗でアプリが自分で閉じる。「止めずに閉じる」の契約の中の扱いとして受けられるか。R3（内部の command で古い listener を外す）を棄てた理由は妥当か。
4. 進行中の窓の外側の click → Coordinator の裁定（2026-11-14）で決まった: 配線しない（上の 7）。closure では、UI-USW-D5 の 4・Scope・T4 に配線の残りが無いことを見る。
5. T5 の oracle を `defaultPrevented` でなく「`onOpenChange` を配線した窓が Esc の後も残る」にした（(12) の実測。裁定 #4 の「event の取消を直接観測」は、`defaultPrevented` では判別できなかった）。
6. Risk は R3 のままにした。「閉じる」で process が transaction の途中で終わる経路ができることを R4 と見るか。
7. O3 の確かめ方の文 → Coordinator の裁定（2026-11-14）で、commit の前に閉じていた場合の「もう一度」を言い足した（「owner の決定」の節）。closure では、UI-USW-D5 の 4 の表・55 §55.7・L3-1・L3-2 の文がそろっていることを見る。
8. Contract Probe の形（1 つの build で取込みの確定だけを async にし、sleep で時間を作る）で (i) の 2 つの前提を確かめたことになるか。closure と probe の順。
9. 店の PC の論理 core が 1 つの場合の「閉じる」の待ち（(10)。core の数は未確認）を前提のままにしてよいか。
