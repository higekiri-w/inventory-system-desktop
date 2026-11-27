# Plan Packet: 右クリックで WebView2 の既定のメニューを出さず、文字の欄だけに自前の小さなメニュー（切り取り・コピー・貼り付け・すべて選択）を出す

wave に属さない単独の lane。出所は owner の決定（repo 外の回答台帳 TD-905。原文は台帳にあり、ここは ID と要旨だけ）: 今は右クリックで WebView2 の既定のメニュー（絵文字・パスワードのインポート・元に戻す・切り取り・コピー・貼り付け・プレーンテキストとして貼り付け・すべて選択・書き込み方向・その他のツール・開発者ツールで調査する）が出る（2026-11-25、開発の build の入荷の画面で owner が観測）。決定は、文字の欄の中では「切り取り・コピー・貼り付け・すべて選択」だけを出し、欄の外では何も出さない。根拠の原則は `docs/design-system/03-philosophy.md` の *Effortless*（コンテキストメニュー等の Windows 慣習を踏襲）。`docs/backlog.md` に項目は無く、この packet が持つ。2026-11-25 の owner の決定（同じ台帳）: 貼り付けは Rust・依存に触れず `navigator.clipboard.readText()` で読む（O1 = C、P1 の実測 TD-907）、日付・月・時刻の欄は文字の欄に含めない（O2 = a、TD-909）、項目の横にキーの案内を出さない（O3 = a）。2026-11-28 の Gated Amendment（owner の決定、同じ台帳 TD-952）: Windows の L3（head `ac1123b6`、`tauri dev`）で、右クリックの「貼り付け」に WebView2 の許可の窓（「クリップボードにコピーされたテキストや画像の参照」ブロック / 許可）が最初の貼り付けから出て、× で閉じると貼り付けが全部拒まれた（Ctrl+V は効く）。O1 = C の前提（TD-907）が崩れ D-171 の Revisit に当たったので、貼り付けは Tauri の公式の clipboard の plugin（`tauri-plugin-clipboard-manager`）の `readText` で Rust 側から読む（owner の選んだ A。許可の窓を出さない）。D-177 が D-171 の貼り付けの部分を置き換える。

## Workflow State

- Risk: R3
- Plan Commit: 6b8c786c218e33c636b33f198bb7f9819441bc96
- Amendments: none

`Plan Commit`・`Amendments` は `scripts/plan-approve.sh` が書く（手で写さない）。

## Risk

Risk: R3

Reason:
全画面の文字の欄の操作（operator の業務の流れ）を変える横断の振舞いで、切り取り・貼り付けは React の制御された欄の state を書き換える（DEV_WORKFLOW の Risk Tiers の「状態を持つ画面・operator の業務の流れ」）。R4 にしない理由: DB・migration・DTO・bindings・backup・店の実データに触れない。Rust は公式の clipboard の plugin の登録の 1 行だけ、依存はその npm と crate の 1 対だけで、capability は文字の読取りの 1 つだけを許す（書込み・画像は許さない。Gated Amendment、TD-952）。Cargo.lock には plugin を含めて 33 個の crate（推移で `arboard` と画像の decoder ほか）が入るが、capability で文字の読取りだけに絞る。

## Goal

Goal Invariant:

### 最小完了条件

- どの画面・どの確認の窓でも、文字の欄の外（日付・月・時刻の欄を含む）で右クリック（とメニューキー・Shift+F10）をしても何も出ない（WebView2 の既定のメニューも出ない）。
- 文字の欄の中で右クリック（とメニューキー・Shift+F10）をすると、「切り取り」「コピー」「貼り付け」「すべて選択」の 4 つだけのメニューが出て、選んだ操作がその欄に効き、欄の値（React の state）に入る。選択が無いときの切り取り・コピーと、読取り専用の欄の切り取り・貼り付けは押せない形で出る。
- メニューは矢印・Enter・Esc で操作でき、閉じると focus が元の欄に戻り、caret と選択が閉じる前のまま（全選択にならない）。

### 失敗定義

- 欄の外で既定のメニューか自前のメニューが出る。欄の中で 5 つ目の項目（既定のメニューの項目）が出る。
- 貼り付け・切り取りの後に、画面の表示は変わったのに state に入らず、保存・検索に古い値が使われる。貼り付けの読みの間に別の欄へ移ると、移った先の欄に入る。
- 配る build（release）で、貼り付けで許可の窓が出る、または読めないときに toast（貼り付ける文字が無いことと、コピーし直す案内）が出ず黙って何もしない（失敗はその場で原因と直し方を名指す。AGENTS.md「使い手に押しつけない」）。
- メニューを閉じると欄の文字が全部選ばれる（次の 1 打で消える）。確認の窓の中の欄でメニューを開くと、確認の窓が閉じる。
- IME の変換中の文字が右クリックで消える・二重に入る。

### 非目的

- 欄の外の文字（表・見出し）をメニューでコピーすること（owner の決定は「欄の外では何も出さない」。Ctrl+C は今のまま効く）。
- 元に戻す・やり直しの項目、メニューの項目にキーの案内（Ctrl+X 等）を足すこと（O3 = a）。
- ショートカット一覧（54 の `SHORTCUTS`）に Shift+F10 を足すこと。
- `contentEditable` の要素への対応（`rg -n 'contentEditable' src` が 0 件、2026-11-25。下の ponytail の限界）。
- 汎用の `src/components/ui/context-menu.tsx`（shadcn の recipe）を置くこと（使う所が 1 つなので置かない）。

## Scope

- 新しい component `src/components/layout/TextFieldContextMenu.tsx`（UI-CTX-D1〜D4）: `radix-ui` の `ContextMenu`（入れてある `radix-ui` 1.4.3 の `@radix-ui/react-context-menu` 2.2.16。package-lock の行で確かめた）の Root・Trigger・Portal・Content・Item を直に使う。Trigger は `asChild` で `<div className="contents">` に付け、子に app 全体を受ける。既定のメニューは `document` の `contextmenu` の listener で常に `preventDefault()` する 1 か所で止める（開いたメニュー〈Portal の Content〉の上と、開いている間 `body` が `pointer-events: none` で Trigger に届かない要素の上も塞ぐ。bubble で付ける: capture だと Radix が先に `defaultPrevented` を見て開かない）。Trigger の `onContextMenu` は開くかどうかだけを決める: target が文字の欄（D2）でなければ `preventDefault()` して Radix の open を飛ばし（Radix の `composeEventHandlers` は `defaultPrevented` なら自分の open を飛ばす。2.2.16 の `dist/index.mjs` の Trigger の `onContextMenu` で確かめた）、欄なら、その欄を ref に控え、Radix に開かせる。項目の `onSelect` は選んだ操作を ref に控えるだけ（開いている間は FocusScope が focus を menuitem に引き戻すので、ここで欄に戻さない）。Content の `onCloseAutoFocus` で `preventDefault()`（Radix の FocusScope は閉じるとき input に `select()` をかける。`@radix-ui/react-focus-scope` 1.1.7 の `focus(previouslyFocusedElement, { select: true })` で確かめた）→ 控えた欄に focus だけを戻す（選択は欄が持ったまま。P2）→ 控えた操作を打つ。切り取り・コピーは `document.execCommand("cut"|"copy")`、すべて選択は `select()`。貼り付けは clipboard の plugin の `readText()`（`@tauri-apps/plugin-clipboard-manager`。Rust 側で読むので WebView2 の許可の窓を通らない。`navigator.clipboard.readText()` は使わない。TD-952）で読み（`insertText` は focus の欄に入るので、欄に focus を戻してから読み・入れる）、読み終えた時点で `document.activeElement` が控えた欄のときだけ、読んだ文字を `document.execCommand("insertText", false, text)`（`input` event が出て React の `onChange` に届き、`maxLength` と元に戻すの履歴が効く）で入れる（別の欄へ移っていれば両方の欄を変えない。空の文字は入れない）。Esc・外の click で閉じたときは操作が控えられていないので、focus を戻すだけ。読めなかったとき（reject。欄を移っていても）は `toast.error("貼り付ける文字がありません。貼り付けたい文字をコピーしてから、もう一度お試しください。", { id: "text-field-paste-error", duration: 8000 })`（02-component-catalog ⑦）を出し、欄を変えない。何もコピーしていない・画像だけのとき（plugin の `readText()` は `arboard` の `ContentNotAvailable` で reject する）の貼り付けも、この文が出て欄は変わらない（plugin で読むので許可の窓で読めない場面は無く、Ctrl+V の案内は出さない。owner の決定、TD-958）。見た目は `src/components/ui/select.tsx` の `SelectContent`・`SelectItem` の class にそろえる。IME の変換中（`compositionstart`〜`compositionend`、wrapper の React の composition event で控える）は開かず `preventDefault()` だけ。`ponytail:` comment: 文字の欄は `HTMLTextAreaElement` と D2 の型の `HTMLInputElement` だけで、`contentEditable` を使う画面が出たらその判定を足す。
- 貼り付けの読取りの plugin（Gated Amendment、TD-952）: 新しい runtime 依存は npm の `@tauri-apps/plugin-clipboard-manager@2.3.3`（`--save-exact`）と crate の `tauri-plugin-clipboard-manager = "=2.3.3"` の 1 対。2.3.3 を選ぶ理由: 2.4.0 以降の crate は `tauri ^2.12` を求め、lockfile の `tauri` 2.11.1 を上げることになる（`index.crates.io` の deps で確かめた）。2.3.3 は `tauri ^2.10`・`tauri-plugin ^2.5`、npm の 2.3.3 は `@tauri-apps/api ^2.11.0` で、今の版のまま入り、公開は 2026-08-31（7 日を超える）。`src-tauri/src/lib.rs` の `run()` の builder に `.plugin(tauri_plugin_clipboard_manager::init())` を 1 行、`src-tauri/capabilities/default.json` に `clipboard-manager:allow-read-text` を 1 つ（書込み・画像の読取り・消去は許さない）。切り取り・コピー・すべて選択は今の `execCommand`・`select()` のまま変えない。
- 同じ所の test `src/components/layout/TextFieldContextMenu.test.tsx`（Test Plan の T1〜T14）。`describe` に `UI-12` を併記する（UI-CTX だけだと `generate_traceability --check` が未参照の file と数えて落ちる。round 1 の Codex の実測 0 → 1 → 0）。
- mount: `src/main.tsx` で `<RouterProvider>` を `TextFieldContextMenu` で包む。RootLayout でなく main にする理由: root の `errorComponent`（`RouteErrorFallback` の fullScreen、RootLayout の外に出る）も同じ Trigger の木に入る。
- 設計正本（実装の前に plan-first で直す）:
  - `docs/UI_TECH_STACK.md` に §6.13「文字の欄の右クリックのメニュー」を §6.12 の後・§7 の前に新設し、UI-CTX-D1（欄の中の 4 項目と欄の外は何も出さない）・D2（文字の欄の判定: textarea と、input の型が未指定・`text`・`search`・`number`。`disabled` は外、`readOnly` は切り取り・貼り付けを押せない形、選択の無いときは切り取り・コピーを押せない形〈`number` は P2 で `getSelection()` が読めればそれで決め、読めなければ押せる形〉、IME の変換中は開かない。`type="date"`・`month`・`time` 等ほかの型は外でメニューも既定のメニューも出さない）・D3（操作の実現: `onSelect` は控えるだけで、閉じた後に欄に focus だけを戻して打つ。`execCommand`、clipboard の plugin の `readText()` の後に同じ欄のときだけ入れる貼り付け、失敗の toast、Esc・外の click の後の focus）・D4（キーボード: メニューキー・Shift+F10 で開く、矢印・Enter・Esc は Radix の menu の pattern）を置く。§7.4 の更新履歴に 1 行。
  - `docs/function-design/52-ui-shared-layout.md` §52.1 の接続点の列（`src/main.tsx` の行、43 行目の `installWindowCloseGuard()` の隣）に `TextFieldContextMenu` の mount を 1 行、更新履歴に 1 行。
  - `docs/decision-log.md` に D-171（WebView2 の既定のメニューは Tauri 2.11 の設定で止められないので frontend の `contextmenu` で止め、自前の 4 項目にし、貼り付けは `navigator.clipboard.readText()` で読む。Alternatives: 既定のメニューの項目を減らすには WebView2 の `ContextMenuRequested` を Rust から直に使う要があり〈`webview2-com` を直の依存に足す Windows だけの code〉Tauri・wry からは届かないので棄却。貼り付けの `enable_clipboard_access` で window を作り直す案と clipboard の plugin を足す案は、P1 の実測で `readText()` が許可の窓なしに通ったので棄却）。D-171 の本文は書き換えない。
  - `docs/decision-log.md` に D-177（Gated Amendment、TD-952: 貼り付けは clipboard の plugin の `readText` で Rust 側から読む。Context は L3 の許可の窓、Alternatives は `navigator.clipboard.readText()` のまま〈許可の窓が出る〉・`enable_clipboard_access`〈window の作り直しで、許可の窓が出なくなる保証の根拠が無い〉）を足し、D-171 に `Superseded in part by: D-177`（貼り付けの読み方）の 1 行。UI_TECH_STACK §6.13 の UI-CTX-D3 の貼り付けの文を置き換え、§7.4 の更新履歴に 1 行。
  - `docs/design-system/03-philosophy.md` は変えない（*Effortless* の今の文で足りる）。`docs/SCREEN_DESIGN.md` も変えない（画面ごとの振舞いでなく、UI_TECH_STACK の横断の節で足りる）。
- 生成物: `docs/function-design/90-traceability.md` は REQ の参照の数が変われば作り直す（REQ を足さない見込み。test の `describe` の `UI-12` は既存の ID）。bindings は変わらない。

並走の lane と重なる file の所有（Wave Operation の one-writer。PR の file の一覧は `gh pr view <N> --json files` で 2026-11-25 に引いた）:

| file | この lane の所有 | 重なる lane |
|---|---|---|
| `docs/UI_TECH_STACK.md` | 新しい §6.13 の全体と、§7.4 の更新履歴の自分の 1 行 | #305（§2 の 245 行付近・§6.11 の UI-USW-D5・§7.4 の先頭に 1 行）、#300（§2 の 330 行付近・§7.2）。§7.4 の行は merge の順で両方を残す |
| `docs/function-design/52-ui-shared-layout.md` | §52.1 の接続点の `main.tsx` の 1 行と更新履歴の末尾の 1 行 | #305（§52.1 の表の行・§52.3・§52.4・更新履歴の末尾）。更新履歴は merge の順で両方を残す |
| `docs/decision-log.md` | D-171 の節と D-177 の節だけ | #308・#300（それぞれの D の節）。D-176 は #315 が予約（2026-11-28） |
| `package.json`・`package-lock.json`・`src-tauri/Cargo.toml`・`src-tauri/Cargo.lock`・`src-tauri/capabilities/default.json` | clipboard の plugin の 1 対と許可の 1 つ | なし（2026-11-28 に開いている #306・#307・#310〜#312・#315 の file の一覧に無い） |
| `src-tauri/src/lib.rs` | `run()` の builder の `.plugin(...)` の 1 行 | #307（`export_bindings_to` と `invoke_handler` の行。重ならない） |
| `docs/function-design/90-traceability.md`（作り直すとき） | 生成物 | #305。手で解かず取り込んだ後に作り直す（D-098） |
| `src/main.tsx`・`src/components/layout/TextFieldContextMenu*.tsx` | 全体 | なし（#305〜#308・#300 の file の一覧に無い） |

Non-scope: 各画面の file（入荷の画面 `ReceivingPage.tsx`〈#308〉を含む。全画面に効くのは main の 1 か所の mount による）、`src-tauri/**` のうち上の plugin の 1 対・登録・許可の外（`tauri.conf.json`、`tauri` 本体の版）、`src/features/shortcuts/`。

## Acceptance Criteria

- AC1: `npx vitest run src/components/layout/TextFieldContextMenu.test.tsx` で T1〜T14 が全部 PASS。欄の外と開いたメニューの上の `contextmenu` は `defaultPrevented` でメニューが無く（T1・T11）、欄の中は `role="menuitem"` の 4 つだけ（T2）、押せない形（`aria-disabled`）が D2 のとおり（T3・T4）、閉じた後に focus が欄に戻り `selectionStart`・`selectionEnd` が開く前と同じ（T9）。
- AC2: Test Plan の mutation M1〜M10（M4 は欠番）を 1 つずつ注入すると、名指しの test で `npx vitest run src/components/layout/TextFieldContextMenu.test.tsx` が exit 1、戻すと exit 0。
- AC3: 既存の frontend の test が全部 PASS（`npx vitest run`）、`npm run typecheck`・`npm run lint`・`cd src-tauri && cargo run --bin generate_traceability -- --check` が exit 0。
- AC4: `bash scripts/doc-consistency-check.sh` と `--target plan` が ERROR なし。設計正本の UI-CTX-D1〜D4 と D-171・D-177 が Scope の置き場にある（`rg -n 'UI-CTX-D[1-4]' docs/UI_TECH_STACK.md` と `rg -n '^## D-17[17]' docs/decision-log.md`）。依存の追加で `npm audit --audit-level=high` が exit 0、`cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings`・`cargo test` が exit 0。
- AC5: L3-1〜L3-4 が owner の Windows の配る build（release）で PASS し、`python3 scripts/pr-gate.py record --kind manual` の記録がある。

## Design Readiness

- 引用する設計正本（節まで）: `docs/design-system/03-philosophy.md` の ⑦ Fluent 2 の *Effortless*、`docs/UI_TECH_STACK.md` §2.3（Radix 基盤）・§5.2（標準キーマッピング）・§5.4（フォーカス管理）・§6.1（通知の使い分け）・§6.5.1（WebView2）、`docs/function-design/52-ui-shared-layout.md` §52.1、AGENTS.md「使い手に押しつけない」、D-030。
- 必要な設計成果物: UI_TECH_STACK §6.13（UI-CTX-D1〜D4）・52 §52.1 の 1 行・D-171 を updated in this PR（plan-first）。Gated Amendment（TD-952）で UI-CTX-D3 の貼り付けの文・D-177・D-171 の `Superseded in part by` の 1 行を updated in this PR。03-philosophy・SCREEN_DESIGN は existing sufficient。
- plan にしかない durable な判断の昇格先: 「既定のメニューは frontend で止める」「閉じた後は選択を保つ」「失敗はその場の toast」は UI-CTX-D1〜D3 と D-171 へ。
- 確かめた技術の前提（source を読んだ。runtime は P1 だけ実測）:
  - 既定のメニューの止め方: wry 0.55.1 は `SetAreDefaultContextMenusEnabled(pl_attrs.default_context_menus)` を持つ（`src/webview2/mod.rs` 571 行、既定 true）が、tauri-runtime-wry 2.11.3 は `with_default_context_menus` を呼ばず、tauri-utils 2.9.3 の window の設定にも項目が無い（`rg` で 0 件）。Rust で止めるには `with_webview` から `ICoreWebView2Settings` に触ることになり `webview2-com` を直の依存に足す（新しい依存）。frontend の `contextmenu` の `preventDefault` で止める（WebView2 は `preventDefault` した `contextmenu` で既定のメニューを出さない、P0）。既定のメニューの項目を減らすには WebView2 の `ContextMenuRequested` を Rust から直に使う要があり（`webview2-com` を直の依存に足す Windows だけの code）、Tauri・wry からは届かないので、自前の 4 項目を作って当てる（TD-910 の後の owner の確認）。
  - 貼り付け（P1 の実測 TD-907 と、L3 の観測 TD-952）: 開発の build の DevTools の Console からの `navigator.clipboard.readText()` は許可の窓なしに通った（TD-907）が、右クリックのメニューの「貼り付け」からの呼出しでは WebView2 の許可の窓が最初の貼り付けから出て、× で閉じると全部拒まれた（TD-952、head `ac1123b6`、`tauri dev`）。よって Tauri の公式の clipboard の plugin の `readText()` で Rust 側（`arboard`）から読み、WebView2 の clipboard の許可を通らない。配る build（release）で許可の窓なしに読めることは L3-2 で確かめる。
  - React の制御された欄: `execCommand("insertText")`・`execCommand("cut")` は `input` event を出すので `onChange` に届く見込み（P2）。`value` を直に書くと React に届かないので使わない。
  - キーボードと部品: `radix-ui` 1.4.3 に ContextMenu があり、menu の role・矢印・Enter・Esc・typeahead・focus の閉じ込めを持つ。メニューキー・Shift+F10 は Chromium が focus の要素に `contextmenu` を出すので、同じ Trigger で開く見込みで、そのときの座標は未実測（P3）。
  - Radix の FocusScope は、開いている間は focus を menuitem に引き戻し（`onSelect` の中で欄に focus しても戻らない。round 1 で両 reviewer が happy-dom で再現）、閉じるときは input に `select()` をかける（`react-focus-scope` 1.1.7 の `focus(…, { select: true })`）。よって操作は `onCloseAutoFocus` で止めて自分で戻した後に打つ（T5・T6・T9、M3・M6・M8・M9）。
- Contract Probe（P1 は TD-907。P0 の開発の build・P2・P3・P5 は owner の実測 TD-913〈PR #309 の issuecomment-7167285898〉で計画の形のとおり成り立ち、Gated Amendment は要らなかった。残るのは P0 の配る build と P4 で、L3 で確かめる）:
  - P0: Windows の開発の build と release の build（`npm run tauri build` の MSI か `cargo build --release`）で、`preventDefault` した `contextmenu` で既定のメニューが出ないこと。メニューを開いたまま別の欄・表を右クリックしても既定のメニューが出ないこと（別の欄のときは閉じるだけでよく、もう一度の右クリックで開く。そのときの focus の在り処を記録する）。
  - P1（TD-907 の実測は Console からの呼出しだけで、メニューからの呼出しでは許可の窓が出た。TD-952）: 貼り付けは clipboard の plugin の `readText()` で読む（上の「貼り付け」）。開発の build・配る build で許可の窓なしに読めることは L3-2。
  - P2: WebView2 で `execCommand("cut"|"copy"|"insertText")` が制御された欄の state に入り、Ctrl+Z で戻ること。`onCloseAutoFocus` の中で打つ `execCommand("cut"|"copy")` が効くこと（user activation の内か）。`type="number"` の欄で `insertText` の数字・非数字、`window.getSelection().toString()` で選択の有無が読めるか（読めれば D2 の切り取り・コピーの判定に使い、読めなければ押せる形のまま）。閉じた後に `focus()` だけで選択が開く前のままか（戻らなければ Gated Amendment）。
  - P3: メニューキー・Shift+F10 の `contextmenu` の `clientX`・`clientY`（0,0 なら欄の左下に出す手当てを Amendment にする）。
  - P4: `display: contents` の wrapper で RootLayout の `h-full` の高さが変わらないこと（画面の高さとスクロール）。
  - P5: IME の変換中の右クリックで `contextmenu` が出るか、変換が確定されるか。
- 前提・制約と延期した design gap: P0・P2〜P5 の結果で D2・D3 の書き方が変わりうる（変われば計画の review の後でも Gated Amendment）。`contentEditable` は非目的（ponytail の限界）。
- 絶対保証の例外と抜け道: 既定のメニューは `document` の listener で止めるので、React の木の外の要素の上も止まる。`<input>` の `disabled` の上の右クリックは外として扱う。
- 判定: ready。O1〜O3 は owner が決め（2026-11-25）、P0 の開発の build・P2・P3・P5 は TD-913 で計画の形のとおり成り立ち、P1 は TD-952 で崩れて貼り付けを plugin で読む形に改めたので、P0 の配る build・P4 と、plugin で読む貼り付け（許可の窓が出ないこと・文字の無いときの toast）は Windows の L3（L3-1〜L3-4）で取り直す。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| UI-CTX-D1 欄の中の 4 項目・欄の外は何も出さない | UI_TECH_STACK §6.13（新設）、03-philosophy の *Effortless* | `document` の `contextmenu` の listener、`TextFieldContextMenu` の `onContextMenu`・Item、`main.tsx` の mount | T1・T2・T11・T13 | L3-1・L3-2 |
| UI-CTX-D2 文字の欄の判定と押せない形 | UI_TECH_STACK §6.13（日付・月・時刻の欄は外、数字の欄は内） | 欄の判定・`disabled` の Item・IME の控え | T3・T4・T11・T12・T14 | L3-3 |
| UI-CTX-D3 操作の実現・失敗の表示・閉じた後の focus | UI_TECH_STACK §6.13、§6.1（toast）、AGENTS.md「使い手に押しつけない」 | `onSelect`（控えるだけ）・`onCloseAutoFocus`（戻して打つ）・clipboard の plugin の `readText()` の後の欄の確かめ、`lib.rs` の plugin の登録、capability の `clipboard-manager:allow-read-text` | T5〜T9・T21 | L3-2 |
| UI-CTX-D4 キーボード | UI_TECH_STACK §6.13、§5.2・§5.4 | Radix の menu、メニューキー・Shift+F10 の `contextmenu` | T10 | L3-4 |
| UI-12 §52.1 の接続点 | 52 §52.1 | `main.tsx` の 1 行 | test の `describe` に併記（`generate_traceability --check`） | 非対象（docs） |
| D-171・D-177 | decision-log | — | — | 非対象（docs） |

隣の契約の sweep: UI-USW-D5（native の window の close。window の作り方に触れないので非対象）、§5.3 のバーコードの入力（メニューは開いている間だけ focus を取る。スキャンは欄に focus があるときで、右クリックの間に読まない。非対象）、54 のショートカット（Ctrl+/ の listener と干渉しない。非対象）。

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| D1 欄の外 | 欄の外か開いたメニューの上で既定のメニューかこのメニューが出る | unit（RTL） | T1 `UI-CTX-D1: 欄の外と開いたメニューの上の右クリックは既定を止めて何も出さない` | 欄の判定を外す（M1）、`document` の listener を外す（M7） |
| D1 欄の中 | 4 つ以外の項目・項目の無いメニュー | unit（revisit は開き直したときに控えが新しい欄になることだけを確かめる。focus を一つ目に置いたまま二つ目で開く経路も、H7） | T2 `UI-CTX-D1: 文字の欄の右クリックで 4 つの項目だけが出る` | Item を足す・欠く、`onCloseAutoFocus` の `preventDefault` だけを外す |
| D2 選択の有無 | 選択が無いのに切り取り・コピーが押せる | unit | T3 `UI-CTX-D2: 選択が無いと切り取り・コピーは押せない形` | 選択の判定を外す（M2） |
| D2 読取り専用 | 読取り専用の欄に切り取り・貼り付けが効く | unit | T4 `UI-CTX-D2: readOnly の欄は切り取り・貼り付けを押せない形` | `readOnly` を見ない |
| D3 コピー・切り取り | 欄に focus を戻さずに command を打つ | unit（`document.execCommand` の spy。happy-dom の実装の有無は未実測） | T5 `UI-CTX-D3: コピーは閉じた後に欄に focus を戻してから打つ` | command を `onSelect` の中で打つ（focus を戻す前。M9） |
| D3 貼り付け | 読んだ文字が欄に入らない、欄に focus が戻る前に読む、読みの間に移った先の欄に入る、WebView2 の許可の窓を通る `navigator.clipboard.readText()` で読む | unit（`@tauri-apps/plugin-clipboard-manager` の `readText` の mock。呼ばれた時の `activeElement` を控える。読みを待たせて別の欄へ移る形も。`navigator.clipboard.readText` は spy にして呼ばれないことを確かめる） | T6 `UI-CTX-D3: 貼り付けは閉じた後に欄に focus を戻してから clipboard の plugin で読み、同じ欄のときだけ insertText で入れる（待たせて別の欄へ移った後は insertText の spy が呼ばれない。読んだ文字が空なら呼ばない）` | `insertText` を `value` の直の代入に変える、`readText()` を focus を戻す前に呼ぶ（M6）、読み終えた時点の `activeElement` の確かめを外す（M8）、空の文字の判定を外す、`navigator.clipboard.readText()` に戻す（M10） |
| D3 貼り付けの失敗 | 失敗が黙って終わる、Ctrl+V でも入らないのに Ctrl+V を案内する | unit（plugin の `readText` の reject） | T7 `UI-CTX-D3: 読めなかったら toast でコピーし直しを案内し欄を変えない` | catch で何も出さない、toast を前の文（Ctrl+V の案内）に戻す |
| D3 plugin の登録と許可（Gated Amendment、TD-952） | plugin を登録しない・許可が無くて実機で読めない、書込み・画像の読取りまで許す | unit（source の検査。T17 と同じ形） | T21 `UI-CTX-D3: clipboard の plugin を登録し、capability は文字の読取りだけを許す`（`TextFieldContextMenu.mount.test.ts`） | `lib.rs` の `.plugin(tauri_plugin_clipboard_manager::init())` を外す、`clipboard-manager:allow-read-text` を外す、`clipboard-manager:` の許可を足す |
| D3 すべて選択 | 全部が選ばれない | unit | T8 `UI-CTX-D3: すべて選択で欄の全文が選ばれる` | `select()` を呼ばない |
| D3 閉じた後 | 閉じると全選択になる・focus が欄に戻らない | unit | T9 `UI-CTX-D3: 項目・Esc・外の click で閉じると focus が欄に戻り選択が開く前のまま` | `onCloseAutoFocus` の復帰の処理ごと外す（M3。`preventDefault` だけを外す形は round 1 の試しで red にならなかった） |
| D4 キーボード | 矢印・Enter・Esc で操作できない、Esc の後に focus が戻らない | unit | T10 `UI-CTX-D4: focus の欄の contextmenu で開き矢印・Enter・Esc で操作できる` | Radix の Content を素の div に変える |
| D1・D2 欄でない input | checkbox・`disabled`・無効の `fieldset` の中・`date`・`month`・`time` の欄でメニューが出る | unit | T11 `UI-CTX-D2: checkbox・disabled・日付・月・時刻の欄では何も出さない` | 欄の型の判定を外す（M1） |
| D2 IME | 変換中に開いて変換中の文字に command を打つ | unit（composition event） | T12 `UI-CTX-D2: IME の変換中は開かない` | composition の控えを外す（M5） |
| D1 確認の窓の中 | Dialog の中の欄で開くと Dialog が閉じる・メニューが Esc で閉じない | unit（`Dialog` の中の欄） | T13 `UI-CTX-D1: 確認の窓の中の欄で開いても窓は開いたまま` | Portal・layer を外す |
| D2 数字の欄 | 選択があるのに切り取り・コピーを押せない・選択が無いのに押せる | unit（`type="number"` の欄。`window.getSelection()` の spy。Contract Probe の P2 で `getSelection()` が選択を読めた、TD-913） | T14 `UI-CTX-D2: 数字の欄は選択があるときだけ切り取り・コピーを押せる形で開く` | `number` の選択を `getSelection()` で読まない（M2 でも red） |
| D3 閉じる動きの間（Final round 1 の H1） | 閉じる動きの間に別の欄で開き直し、前の操作が後の欄に効く | unit（閉じる animation の模型: animation-name の style と `animationend`） | T15 `UI-CTX-D3: 閉じる動きの間に別の欄で右クリックしても控えた欄は替わらない` | 生きている間の開き直しの止めを外す |
| D1 長押し（H2） | touch・pen の長押しで欄の外にも 4 項目が出て、前の欄に効く | unit（fake timers で 800 ms） | T16 `UI-CTX-D1: touch・pen の長押しではメニューを出さない` | 欄を控えていないときも Content を描く |
| UI-12 mount（H5） | `main.tsx` の包みを外しても全部 green | unit（source の正規表現。`unsaved-changes-guard-sweep.test.ts` と同じ形） | T17 `UI-CTX-D1: src/main.tsx が RouterProvider を TextFieldContextMenu で包む`（`TextFieldContextMenu.mount.test.ts`） | 包みを外す |
| D3 blur で保存する欄（H6） | 保存日数の欄でメニューを開くだけで打ちかけの値が保存される | unit（`BackupRestorePage` を包む） | T18 `REQ-901 / UI-CTX-D3: 保存日数の欄で右クリックのメニューを開いても打ちかけの値を保存せず、離れたときに保存する`（`BackupRestorePage.test.tsx`） | `relatedTarget` の確かめを外す |
| D3 閉じる動きの間のキー（Final closure round 2 の J1） | 閉じかけに Enter・矢印を押すと、Esc の後に操作が打たれる・確定した操作が別の項目に替わる | unit（T15 と同じ閉じる animation の模型） | T19 `UI-CTX-D3: Esc で閉じる動きの間に Enter を押しても何も打たない`、T20 `UI-CTX-D3: コピーを選んで閉じる動きの間に ↑・Enter を押しても、コピーを 1 回だけ打つ` | 閉じかけの `onSelect` の止めを外す |
| D4 フォーカスの印（H4） | 矢印で動かした先の項目が見分けられない | L3（happy-dom は描画しない） | L3-4 で見る | 項目の focus の枠の class を外す |

状態の遷移: 閉じている → 開く（右クリック・キー）→ 項目を選ぶ（操作を控える）/ Esc・外の click（控えない）→ 閉じる（`onCloseAutoFocus` で focus を戻し、控えた操作があれば打つ）、貼り付けの読みの failure（T7。toast を出し欄は変えない）、読みの間に別の欄へ移る（T6。両方の欄を変えない）、開いたまま別の欄で右クリック（実機は開いている間の `pointer-events: none` で閉じるだけになりうる〈P0・L3-2〉。T2 の revisit は開き直したときに控えが新しい欄になることだけを確かめる）。

mutation（実装の run で 1 つずつ注入して red を確かめ、戻す）: M1 欄の判定を常に真にする → T1・T11。M2 切り取り・コピーの `disabled` を外す → T3。M3 `onCloseAutoFocus` の復帰の処理ごと外す → T9。M4 は欠番。M5 composition の控えを外す → T12。M6 `readText()` を `onSelect` の中で呼ぶ（focus を戻す前）→ T6。M7 `document` の `contextmenu` の listener を外す → T1。M8 読み終えた時点の `activeElement` の確かめを外す → T6。M9 コピー・切り取りの command を `onSelect` の中で打つ（focus を戻す前）→ T5。M10 貼り付けの読みを `navigator.clipboard.readText()` に戻す → T6。

## L3

- L3-1 入荷の画面・商品の編集の画面・ショートカット一覧の窓（Ctrl+/、欄の無い窓）/ Windows の build で、欄の外（見出し・表・余白・ボタン・確認の窓の本文）を右クリックし、メニューキーと Shift+F10 も押す / どれでも何も出ない（WebView2 の既定のメニューも出ない）。
- L3-2 入荷の画面の商品追加の欄 / 文字を打ち、一部を選んで右クリック → 「コピー」、別の欄で右クリック → 「貼り付け」、何もコピーしていない（または画像だけをコピーした）状態で欄を右クリック → 「貼り付け」/ メニューに 4 つだけが出る。配る build でも貼り付けた文字が欄に入り、許可の窓は出ず、その値で検索・保存が動く（表示だけ変わって古い値で動くことが無い）。何もコピーしていない・画像だけのときの貼り付けでは「貼り付ける文字がありません。貼り付けたい文字をコピーしてから、もう一度お試しください。」が出て欄は変わらない。「すべて選択」→「切り取り」で欄が空になる。閉じた後に打った文字が caret の位置に入る（全部が置き換わらない）。メニューを開いたまま別の欄を右クリックしたときは閉じるだけでよい（もう一度の右クリックで開く）。そのときの focus の在り処を記録する。メニューが閉じかけのまま開き直って元の欄の横に残り、操作が別の欄に効くのは FAIL。入荷の画面で窓を縮め、下端までスクロールして操作のボタンに届く（`display: contents` の包みで画面の高さとスクロールが変わらない。P4）。観察（合否に数えない）: Excel の cell をコピーして貼り付けたときに、末尾に空白（改行）が付くかを記録する。
- L3-3 商品の編集の画面の読取り専用の欄と、空の欄と、入荷の画面の数量の欄 / 右クリック / 読取り専用では切り取り・貼り付けが押せない形、選択の無い欄では切り取り・コピーが押せない形で読める。数量の欄で数字を選んでコピー → 別の数量の欄に貼り付けると、その数字が入る。
- L3-4 入荷の画面の商品追加の欄 / focus を置いてメニューキーか Shift+F10、矢印で「すべて選択」、Enter、もう一度開いて Esc / 欄の近くに開き、矢印・Enter・Esc が効き、矢印で動かした先の項目が濃い色の枠で囲まれ、どれを選んでいるか見分けられる。閉じた後も欄に focus がある。