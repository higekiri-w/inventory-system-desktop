# Plan Packet: 右クリックで WebView2 の既定のメニューを出さず、文字の欄だけに自前の小さなメニュー（切り取り・コピー・貼り付け・すべて選択）を出す

wave に属さない単独の lane。出所は owner の決定（repo 外の回答台帳 TD-905。原文は台帳にあり、ここは ID と要旨だけ）: 今は右クリックで WebView2 の既定のメニュー（絵文字・パスワードのインポート・元に戻す・切り取り・コピー・貼り付け・プレーンテキストとして貼り付け・すべて選択・書き込み方向・その他のツール・開発者ツールで調査する）が出る（2026-11-25、開発の build の入荷の画面で owner が観測）。決定は、文字の欄の中では「切り取り・コピー・貼り付け・すべて選択」だけを出し、欄の外では何も出さない。根拠の原則は `docs/design-system/03-philosophy.md` の *Effortless*（コンテキストメニュー等の Windows 慣習を踏襲）。`docs/backlog.md` に項目は無く、この packet が持つ。貼り付けの読み方（下の O1）と、日付・月・時刻の欄の扱い（O2）は owner の答えの前で、計画の固定はその後。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

`Plan Commit`・`Amendments` は `scripts/plan-approve.sh` が書く（手で写さない）。

## Risk

Risk: R3

Reason:
全画面の文字の欄の操作（operator の業務の流れ）を変える横断の振舞いで、切り取り・貼り付けは React の制御された欄の state を書き換える（DEV_WORKFLOW の Risk Tiers の「状態を持つ画面・operator の業務の流れ」）。貼り付けの道によっては window の作り方（`tauri.conf.json`・`src-tauri/src/lib.rs`）か依存の追加（D-030）に触れる（O1）。R4 にしない理由: DB・migration・DTO・bindings・backup・店の実データに触れない。

## Goal

Goal Invariant:

### 最小完了条件

- どの画面・どの確認の窓でも、文字の欄の外で右クリック（とメニューキー・Shift+F10）をしても何も出ない（WebView2 の既定のメニューも出ない）。
- 文字の欄の中で右クリック（とメニューキー・Shift+F10）をすると、「切り取り」「コピー」「貼り付け」「すべて選択」の 4 つだけのメニューが出て、選んだ操作がその欄に効き、欄の値（React の state）に入る。選択が無いときの切り取り・コピーと、読取り専用の欄の切り取り・貼り付けは押せない形で出る。
- メニューは矢印・Enter・Esc で操作でき、閉じると focus が元の欄に戻り、caret と選択が閉じる前のまま（全選択にならない）。

### 失敗定義

- 欄の外で既定のメニューか自前のメニューが出る。欄の中で 5 つ目の項目（既定のメニューの項目）が出る。
- 貼り付け・切り取りの後に、画面の表示は変わったのに state に入らず、保存・検索に古い値が使われる。
- 貼り付けで許可の窓が出る、または貼り付けが黙って何もしない（失敗はその場で原因と直し方を名指す。AGENTS.md「使い手に押しつけない」）。
- メニューを閉じると欄の文字が全部選ばれる（次の 1 打で消える）。確認の窓の中の欄でメニューを開くと、確認の窓が閉じる。
- IME の変換中の文字が右クリックで消える・二重に入る。

### 非目的

- 欄の外の文字（表・見出し）をメニューでコピーすること（owner の決定は「欄の外では何も出さない」。Ctrl+C は今のまま効く）。
- 元に戻す・やり直しの項目、メニューの項目にキーの案内（Ctrl+X 等）を足すこと（O3）。
- ショートカット一覧（54 の `SHORTCUTS`）に Shift+F10 を足すこと。
- `contentEditable` の要素への対応（`rg -n 'contentEditable' src` が 0 件、2026-11-25。下の ponytail の限界）。
- 汎用の `src/components/ui/context-menu.tsx`（shadcn の recipe）を置くこと（使う所が 1 つなので置かない）。

## Scope

- 新しい component `src/components/layout/TextFieldContextMenu.tsx`（UI-CTX-D1〜D4）: `radix-ui` の `ContextMenu`（入れてある `radix-ui` 1.4.3 の `@radix-ui/react-context-menu` 2.2.16。package-lock の行で確かめた）の Root・Trigger・Portal・Content・Item を直に使う。Trigger は `asChild` で `<div className="contents">` に付け、子に app 全体を受ける。Trigger の `onContextMenu` で、target が文字の欄でなければ `preventDefault()` だけをする（Radix の `composeEventHandlers` は `defaultPrevented` なら自分の open を飛ばす。2.2.16 の `dist/index.mjs` の Trigger の `onContextMenu` で確かめた）。欄なら、その欄と `selectionStart`・`selectionEnd` を ref に控え、Radix に開かせる。項目の `onSelect` は、控えた欄に focus を戻し選択を戻してから、切り取り・コピーは `document.execCommand("cut"|"copy")`、貼り付けは O1 の道で読んだ文字を `document.execCommand("insertText", false, text)`（`input` event が出て React の `onChange` に届き、`maxLength` と元に戻すの履歴が効く）、すべて選択は `select()`。読めなかったときは Sonner の toast で「貼り付けられませんでした。Ctrl+V で貼り付けてください。」を出す（文言は計画の review で確かめる）。Content の `onCloseAutoFocus` で `preventDefault()` して控えた欄に focus だけを戻す（Radix の FocusScope は閉じるとき input に `select()` をかける。`@radix-ui/react-focus-scope` 1.1.7 の `focus(previouslyFocusedElement, { select: true })` で確かめた）。見た目は `src/components/ui/select.tsx` の `SelectContent`・`SelectItem` の class にそろえる。IME の変換中（`compositionstart`〜`compositionend`、wrapper の React の composition event で控える）は開かず `preventDefault()` だけ。`ponytail:` comment: 文字の欄は `HTMLTextAreaElement` と文字を打つ型の `HTMLInputElement` だけで、`contentEditable` を使う画面が出たら選択の Range の控えを足す。
- 同じ所の test `src/components/layout/TextFieldContextMenu.test.tsx`（Test Plan の T1〜T13）。
- mount: `src/main.tsx` で `<RouterProvider>` を `TextFieldContextMenu` で包む。RootLayout でなく main にする理由: root の `errorComponent`（`RouteErrorFallback` の fullScreen、RootLayout の外に出る）と RootLayout の外の兄弟（Toaster・ShortcutsDialog・WindowCloseDialog）も同じ Trigger の React の木の中に入り、Portal の確認の窓の event も React の木を通って届く。
- 貼り付けの読み方（O1 の答えで決まる。どれも計画の固定の前に Scope を書き切る）:
  - A（推奨の候補）: `src-tauri/tauri.conf.json` の `app.windows[0]` に `"create": false`、`src-tauri/src/lib.rs` の `setup` の最後で `WebviewWindowBuilder::from_config(app, &app.config().app.windows[0])?.enable_clipboard_access().build()?`。wry 0.55.1 の `webview2/mod.rs` は `clipboard` が真のとき `PermissionRequested` で `CLIPBOARD_READ` を許すので、`navigator.clipboard.readText()` が許可の窓なしに通る見込み（P1）。window の label は `main` のまま（`capabilities/default.json` の `windows`・single-instance の `get_webview_window("main")` を変えない）。`lib.rs` の startup の source の文を固定する test（`test_startup_req901_*` と 787・816・914 行の文の検査）の文を崩さない位置に置く。
  - B: `tauri-plugin-clipboard-manager` と `@tauri-apps/plugin-clipboard-manager` を足す（Cargo.toml・package.json・package-lock.json・`capabilities/default.json` の `clipboard-manager:allow-read-text`・`lib.rs` の plugin の登録。D-030 の名指しと owner の承認、`--save-exact`、lockfile の review、`npm audit --audit-level=high`）。
  - C: Rust に触れず `navigator.clipboard.readText()` を呼ぶ（WebView2 の既定の許可の扱いに任せる。P1 で許可の窓が出るなら失敗定義に当たる）。
- 設計正本（実装の前に plan-first で直す）:
  - `docs/UI_TECH_STACK.md` に §6.13「文字の欄の右クリックのメニュー」を §6.12 の後・§7 の前に新設し、UI-CTX-D1（欄の中の 4 項目と欄の外は何も出さない）・D2（文字の欄の判定: textarea と文字を打つ型の input。`disabled` は外、`readOnly` は切り取り・貼り付けを押せない形、選択の無いときは切り取り・コピーを押せない形、IME の変換中は開かない。O2 の答えを書く）・D3（操作の実現: `execCommand` と O1 の読み方、失敗の toast、閉じた後の focus と選択）・D4（キーボード: メニューキー・Shift+F10 で開く、矢印・Enter・Esc は Radix の menu の pattern）を置く。§7.4 の更新履歴に 1 行。
  - `docs/function-design/52-ui-shared-layout.md` §52.1 の接続点の列（`src/main.tsx` の行、43 行目の `installWindowCloseGuard()` の隣）に `TextFieldContextMenu` の mount を 1 行、更新履歴に 1 行。
  - `docs/decision-log.md` に D-171（WebView2 の既定のメニューは Tauri 2.11 の設定で止められないので frontend の `contextmenu` で止め、自前の 4 項目にする。O1 の答えと棄却した案）。
  - `docs/design-system/03-philosophy.md` は変えない（*Effortless* の今の文で足りる）。`docs/SCREEN_DESIGN.md` も変えない（画面ごとの振舞いでなく、UI_TECH_STACK の横断の節で足りる）。
- 生成物: `docs/function-design/90-traceability.md` は REQ の参照の数が変われば作り直す（test 名は UI-CTX の ID で、REQ を足さない見込み）。bindings は変わらない。

並走の lane と重なる file の所有（Wave Operation の one-writer。PR の file の一覧は `gh pr view <N> --json files` で 2026-11-25 に引いた）:

| file | この lane の所有 | 重なる lane |
|---|---|---|
| `docs/UI_TECH_STACK.md` | 新しい §6.13 の全体と、§7.4 の更新履歴の自分の 1 行 | #305（§2 の 245 行付近・§6.11 の UI-USW-D5・§7.4 の先頭に 1 行）、#300（§2 の 330 行付近・§7.2）。§7.4 の行は merge の順で両方を残す |
| `docs/function-design/52-ui-shared-layout.md` | §52.1 の接続点の `main.tsx` の 1 行と更新履歴の末尾の 1 行 | #305（§52.1 の表の行・§52.3・§52.4・更新履歴の末尾）。更新履歴は merge の順で両方を残す |
| `docs/decision-log.md` | D-171 の節だけ | #308・#300（それぞれの D の節） |
| `src-tauri/src/lib.rs`（O1 が A か B のとき） | A は `setup` の最後の window の作成、B は plugin の登録の 1 行 | #305（`export_bindings_to`・`run()` の command の登録の 1580 行付近）。行は重ならない見込み |
| `docs/function-design/90-traceability.md`（作り直すとき） | 生成物 | #305。手で解かず取り込んだ後に作り直す（D-098） |
| `src/main.tsx`・`src/components/layout/TextFieldContextMenu*.tsx`・`src-tauri/tauri.conf.json` | 全体 | なし（#305〜#308・#300 の file の一覧に無い） |

Non-scope: 各画面の file（入荷の画面 `ReceivingPage.tsx`〈#308〉を含む。全画面に効くのは main の 1 か所の mount による）、WebView2 の設定を Rust の `with_webview` で触ること（`webview2-com` を直の依存に足すことになる。Design Readiness）、`src/features/shortcuts/`。

## Acceptance Criteria

- AC1: `npx vitest run src/components/layout/TextFieldContextMenu.test.tsx` で T1〜T13 が全部 PASS。欄の外の `contextmenu` は `defaultPrevented` でメニューが無く（T1・T11）、欄の中は `role="menuitem"` の 4 つだけ（T2）、押せない形（`aria-disabled`）が D2 のとおり（T3・T4）、閉じた後に focus が欄に戻り `selectionStart`・`selectionEnd` が開く前と同じ（T9）。
- AC2: Test Plan の mutation M1〜M5 を 1 つずつ注入すると、名指しの test で `npx vitest run src/components/layout/TextFieldContextMenu.test.tsx` が exit 1、戻すと exit 0。
- AC3: 既存の frontend の test が全部 PASS（`npx vitest run`）、`npm run typecheck`・`npm run lint` が exit 0。O1 が A か B なら `cd src-tauri && cargo test --lib` が全部 PASS し、`cargo check --release` が exit 0（L3 の前の Writer の条件）。
- AC4: `bash scripts/doc-consistency-check.sh` と `--target plan` が ERROR なし。設計正本の UI-CTX-D1〜D4 と D-171 が Scope の置き場にある（`rg -n 'UI-CTX-D[1-4]' docs/UI_TECH_STACK.md` と `rg -n '^## D-171' docs/decision-log.md`）。
- AC5: L3-1〜L3-4（O1 が A なら L3-5 も）が owner の Windows の build で PASS し、`python3 scripts/pr-gate.py record --kind manual` の記録がある。

## Design Readiness

- 引用する設計正本（節まで）: `docs/design-system/03-philosophy.md` の ⑦ Fluent 2 の *Effortless*、`docs/UI_TECH_STACK.md` §2.3（Radix 基盤）・§5.2（標準キーマッピング）・§5.4（フォーカス管理）・§6.1（通知の使い分け）・§6.5.1（WebView2）、`docs/function-design/52-ui-shared-layout.md` §52.1、AGENTS.md「使い手に押しつけない」、D-030。
- 必要な設計成果物: UI_TECH_STACK §6.13（UI-CTX-D1〜D4）・52 §52.1 の 1 行・D-171 を updated in this PR（plan-first）。03-philosophy・SCREEN_DESIGN は existing sufficient。
- plan にしかない durable な判断の昇格先: 「既定のメニューは frontend で止める」「閉じた後は選択を保つ」「失敗はその場の toast」は UI-CTX-D1〜D3 と D-171 へ。
- 確かめた技術の前提（source を読んだ。runtime は未実測）:
  - 既定のメニューの止め方: wry 0.55.1 は `SetAreDefaultContextMenusEnabled(pl_attrs.default_context_menus)` を持つ（`src/webview2/mod.rs` 571 行、既定 true）が、tauri-runtime-wry 2.11.3 は `with_default_context_menus` を呼ばず、tauri-utils 2.9.3 の window の設定にも項目が無い（`rg` で 0 件）。Rust で止めるには `with_webview` から `ICoreWebView2Settings` に触ることになり `webview2-com` を直の依存に足す（新しい依存）。frontend の `contextmenu` の `preventDefault` で止める（WebView2 は `preventDefault` した `contextmenu` で既定のメニューを出さない、P0）。
  - 配る build の「開発者ツールで調査する」: `src-tauri/Cargo.toml` の `tauri` は `features = []` で `devtools` が無い。tauri-runtime-wry 2.11.3 は `#[cfg(any(debug_assertions, feature = "devtools"))]` のときだけ `with_devtools` を呼び、wry の既定は release で `devtools: false`、wry は `SetAreDevToolsEnabled(attributes.devtools)` を呼ぶ。よって release の build では開発者ツールの項目は出ない見込みだが、ほかの既定の項目（絵文字・パスワードのインポート等）は出る（P0 で確かめる）。owner の観測は開発の build。
  - 貼り付け: wry 0.55.1 は `clipboard` の属性が真のときだけ `PermissionRequested` で `COREWEBVIEW2_PERMISSION_KIND_CLIPBOARD_READ` を許す（`webview2/mod.rs` 497〜514 行）。tauri 2.11.1 は `WebviewWindowBuilder::enable_clipboard_access()` でだけ立て、`tauri.conf.json` の window の設定には無い（tauri-utils の config に `clipboard` が 0 件）。`tauri.conf.json` の window の `create`（tauri-utils の `pub create: bool`）を false にして `WebviewWindowBuilder::from_config` で作れば立てられる（O1 の A）。`document.execCommand("paste")` は Chromium の web の内容では通らない見込み（P1 で確かめる）。入れてある依存に clipboard の plugin は無い（`package.json`・`Cargo.toml`）。今の app は `navigator.clipboard.writeText` を `OperationLogsPage.tsx` で使う（書く側は許可の窓が要らない）。
  - React の制御された欄: `execCommand("insertText")`・`execCommand("cut")` は `input` event を出すので `onChange` に届く見込み（P2）。`value` を直に書くと React に届かないので使わない。
  - キーボードと部品: `radix-ui` 1.4.3 に ContextMenu があり、menu の role・矢印・Enter・Esc・typeahead・focus の閉じ込めを持つ。メニューキー・Shift+F10 は Chromium が focus の要素に `contextmenu` を出すので、同じ Trigger で開く見込みで、そのときの座標は未実測（P3）。
  - Radix の閉じるときの focus は input に `select()` をかける（`react-focus-scope` 1.1.7 の `focus(…, { select: true })`）ので、`onCloseAutoFocus` で止めて自分で戻す（T9・M3）。
- Contract Probe（実装の run の最初に行う。どれも `未実測`）:
  - P0: Windows の開発の build と release の build（`npm run tauri build` の MSI か `cargo build --release`）で、`preventDefault` した `contextmenu` で既定のメニューが出ないこと。release で開発者ツールの項目が無いこと。
  - P1: WebView2 で `navigator.clipboard.readText()` が、`enable_clipboard_access` 無しで許可の窓を出すか・拒むか、有りで黙って通るか。`document.execCommand("paste")` が false を返すか。O1 の推奨の前提（Windows の owner の PC が要る。Coordinator が手順を作り、owner の答えは PASS/FAIL）。
  - P2: WebView2 で `execCommand("cut"|"copy"|"insertText")` が制御された欄の state に入り、Ctrl+Z で戻ること。`type="number"` の欄で `insertText` の数字・非数字、`window.getSelection().toString()` で選択の有無が読めるか（`selectionStart` は number で null）。
  - P3: メニューキー・Shift+F10 の `contextmenu` の `clientX`・`clientY`（0,0 なら欄の左下に出す手当てを Amendment にする）。
  - P4: `display: contents` の wrapper で RootLayout の `h-full` の高さが変わらないこと（画面の高さとスクロール）。
  - P5: IME の変換中の右クリックで `contextmenu` が出るか、変換が確定されるか。
- 前提・制約と延期した design gap: P0〜P5 の結果で D2・D3 の書き方が変わりうる（変われば計画の review の後でも Gated Amendment）。`contentEditable` は非目的（ponytail の限界）。
- 絶対保証の例外と抜け道: 「欄の外で何も出ない」は React の木の中の event に限る。React の外の DOM（開発の build の devtools の部品）は外で、配る build には無い。`<input>` の `disabled` の上の右クリックは外として扱う。
- 判定: not ready。O1（貼り付けの読み方）と O2（日付・月・時刻の欄）の owner の答えと、O1 の前提の P1 の前は、Scope の Rust・依存の行と D2 の文が決まらない。答えの後に Scope を 1 つの道に書き切り、計画の review へ出す。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| UI-CTX-D1 欄の中の 4 項目・欄の外は何も出さない | UI_TECH_STACK §6.13（新設）、03-philosophy の *Effortless* | `TextFieldContextMenu` の `onContextMenu`・Item、`main.tsx` の mount | T1・T2・T11・T13 | L3-1・L3-2 |
| UI-CTX-D2 文字の欄の判定と押せない形 | UI_TECH_STACK §6.13（O2 の答え） | 欄の判定・`disabled` の Item・IME の控え | T3・T4・T11・T12 | L3-3 |
| UI-CTX-D3 操作の実現・失敗の表示・閉じた後の focus | UI_TECH_STACK §6.13、§6.1（toast）、AGENTS.md「使い手に押しつけない」 | `onSelect`・`onCloseAutoFocus`・O1 の読み方 | T5〜T9 | L3-2 |
| UI-CTX-D4 キーボード | UI_TECH_STACK §6.13、§5.2・§5.4 | Radix の menu、メニューキー・Shift+F10 の `contextmenu` | T10 | L3-4 |
| UI-12 §52.1 の接続点 | 52 §52.1 | `main.tsx` の 1 行 | — | 非対象（docs） |
| D-171（と O1 が B なら D-030） | decision-log | O1 の道 | — | 非対象（docs） |

隣の契約の sweep: UI-USW-D5（native の window の close。O1 が A なら window の作り方が config から `setup` に移るが、label `main` と close の listener の登録の順は変えない。T は既存の `WindowCloseDialog.test.tsx` と L3-5）、§5.3 のバーコードの入力（メニューは開いている間だけ focus を取る。スキャンは欄に focus があるときで、右クリックの間に読まない。非対象）、54 のショートカット（Ctrl+/ の listener と干渉しない。非対象）。

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| D1 欄の外 | 欄の外で既定のメニューかこのメニューが出る | unit（RTL） | T1 `UI-CTX-D1: 欄の外の右クリックは既定を止めて何も出さない` | 欄の判定を外す（M1） |
| D1 欄の中 | 4 つ以外の項目・項目の無いメニュー | unit | T2 `UI-CTX-D1: 文字の欄の右クリックで 4 つの項目だけが出る` | Item を足す・欠く |
| D2 選択の有無 | 選択が無いのに切り取り・コピーが押せる | unit | T3 `UI-CTX-D2: 選択が無いと切り取り・コピーは押せない形` | 選択の判定を外す（M2） |
| D2 読取り専用 | 読取り専用の欄に切り取り・貼り付けが効く | unit | T4 `UI-CTX-D2: readOnly の欄は切り取り・貼り付けを押せない形` | `readOnly` を見ない |
| D3 コピー・切り取り | 欄に focus と選択を戻さずに command を打つ | unit（`document.execCommand` の spy。happy-dom の実装の有無は未実測） | T5 `UI-CTX-D3: コピーは欄に focus と選択を戻してから打つ` | 選択の戻しを外す（M4） |
| D3 貼り付け | 読んだ文字が欄に入らない | unit（O1 の読み方の mock） | T6 `UI-CTX-D3: 貼り付けは読んだ文字を insertText で欄に入れる` | `insertText` を `value` の直の代入に変える |
| D3 貼り付けの失敗 | 失敗が黙って終わる | unit | T7 `UI-CTX-D3: 読めなかったら toast で Ctrl+V を案内し欄を変えない` | catch で何も出さない |
| D3 すべて選択 | 全部が選ばれない | unit | T8 `UI-CTX-D3: すべて選択で欄の全文が選ばれる` | `select()` を呼ばない |
| D3 閉じた後 | 閉じると全選択になる・focus が欄に戻らない | unit | T9 `UI-CTX-D3: 閉じると focus が欄に戻り選択が開く前のまま` | `onCloseAutoFocus` の `preventDefault` を外す（M3） |
| D4 キーボード | 矢印・Enter・Esc で操作できない、Esc の後に focus が戻らない | unit | T10 `UI-CTX-D4: focus の欄の contextmenu で開き矢印・Enter・Esc で操作できる` | Radix の Content を素の div に変える |
| D1・D2 欄でない input | checkbox・`disabled` の欄（と O2 の答えで日付の欄）でメニューが出る | unit | T11 `UI-CTX-D2: checkbox・disabled の欄では何も出さない` | 欄の型の判定を外す（M1） |
| D2 IME | 変換中に開いて変換中の文字に command を打つ | unit（composition event） | T12 `UI-CTX-D2: IME の変換中は開かない` | composition の控えを外す（M5） |
| D1 確認の窓の中 | Dialog の中の欄で開くと Dialog が閉じる・メニューが Esc で閉じない | unit（`Dialog` の中の欄） | T13 `UI-CTX-D1: 確認の窓の中の欄で開いても窓は開いたまま` | Portal・layer を外す |

状態の遷移: 閉じている → 開く（右クリック・キー）→ 項目を選ぶ（success）/ Esc・外の click（dismiss）→ 閉じる（focus の戻り）、貼り付けの読みの failure（T7）、開いたまま別の欄で右クリック（revisit。T2 の中で 2 つ目の欄に移ると控えが新しい欄になることを確かめる）。

mutation（実装の run で 1 つずつ注入して red を確かめ、戻す）: M1 欄の判定を常に真にする → T1・T11。M2 切り取り・コピーの `disabled` を外す → T3。M3 `onCloseAutoFocus` の `preventDefault` を外す → T9。M4 `onSelect` の focus と選択の戻しを外す → T5。M5 composition の控えを外す → T12。

## L3

- L3-1 入荷の画面・商品の編集の画面・ショートカット一覧の窓（Ctrl+/、欄の無い窓）/ Windows の build で、欄の外（見出し・表・余白・ボタン・確認の窓の本文）を右クリックし、メニューキーと Shift+F10 も押す / どれでも何も出ない（WebView2 の既定のメニューも出ない）。
- L3-2 入荷の画面の商品追加の欄 / 文字を打ち、一部を選んで右クリック → 「コピー」、別の欄で右クリック → 「貼り付け」/ メニューに 4 つだけが出る。貼り付けた文字が欄に入り、許可の窓は出ず、その値で検索・保存が動く（表示だけ変わって古い値で動くことが無い）。「すべて選択」→「切り取り」で欄が空になる。閉じた後に打った文字が caret の位置に入る（全部が置き換わらない）。
- L3-3 商品の編集の画面の読取り専用の欄と、空の欄 / 右クリック / 読取り専用では切り取り・貼り付けが押せない形、選択の無い欄では切り取り・コピーが押せない形で読める。
- L3-4 入荷の画面の商品追加の欄 / focus を置いてメニューキーか Shift+F10、矢印で「すべて選択」、Enter、もう一度開いて Esc / 欄の近くに開き、矢印・Enter・Esc が効き、閉じた後も欄に focus がある。
- L3-5（O1 が A のときだけ）起動と × / アプリを起動して × で閉じ、もう一度起動する / 窓が今までと同じ大きさ・題で開き、× と二重起動の止め（前の窓が前に出る）が今までどおり効く。
