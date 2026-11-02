# 73. UI-10: 棚卸し画面（StocktakePage）

## 計数の置き場と在庫の求め直しの契約（D-118。b1 で実装済み。「今の数を入れる」の行の route の `product` が外れたときの閉じ方は画面の小口の lane で実装済み）

[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)（SPEC-STK-CNT-D3・D5・D6）の棚卸し画面の側。既存の検索 / HID・候補の確認・一覧・確定の結果を保ち、計数は商品ごとに「数を保存」で即保存する（今の Excel と同じく、数えたら入れる。既定の計数）。棚を今見た数で直すときは「入れ直す」（fresh な計数。ADR D3・D5）。旧 ADR（時点証拠）の「今から数える」（begin）・token・失効・応答不明の照会・legacy / auto_filled / force_fill の表示・非連動化の未調整の案内は持たない（置き換えた）。旧 UI-10-D10・§73.6 / §73.10 / §73.12 の current_stock の差異の計算、UI-10-F5・UI-10-D4 ほかの force_fill の確認 dialog と再送信、補正の一覧の結果画面、とそれらのテストの oracle は、b1（[Plan Packet](../archive/plans/2026-10-22-stocktake-count-runtime-b1.md)）で本節に置き換えた（以下の §73.1〜§73.14 は置き換えた後の現行の契約。決定の履歴の文は当時のまま残し、置き換えた所に注記した）。

| 場面 | 表示・入力・focus |
|---|---|
| 商品を特定 | 商品名・コード・単位、写しの在庫（要再確認なら範囲と「要再確認」）、前に保存した数（読取り）。数量欄は空で focus。保存の日付 D を常に出す（既定は今日） |
| 「前の日の分を入れる」 | 日付を選ぶ（今日より後は選べない）。選んだ日付の計数より後の計数が既にあれば、保存の前に「後の数が既にあり、そちらが効きます」（保存は拒まない。[42](42-cmd-sales-stocktake.md) の `preview_stock_count_placement`） |
| 商品の「入れ直す」 | 前に保存した数があり、棚を今見て直すとき（打ち直しを含む）。日付は今日だけで選べない。保存は `fresh=true`（それより先に保存した数を置き換え、保存の前に入れた今日の入出庫を含む数になる） |
| 「今の数を入れる」（商品を受けて開く） | 回復先（[51](51-ui-product-form.md) の在庫連動の拒否の案内の `active_count`・`count_now`）から開く入口。route `/stocktake/` の search の `product`（商品コード）と `returnTo` を受ける。棚卸し・既存の計数・要再確認の有無によらず、棚を見た数を `fresh=true` で保存できる（`stocktake_item_id` は進行中の棚卸しにその商品の明細があるときだけ送る）。保存の後は `returnTo` へ戻る。`returnTo` は `/products/{product と同じ商品コード}/edit` だけを受け、外れれば（無い・別の path・別の商品）保存の後も `/stocktake/` に留まる（UI-01b-D2 と同じく許す先を限り、誤った遷移を防ぐ）。戻った商品フォームは DB の値で開く（未保存の入力は 51 の離脱の確認で破棄されている。b1 の Plan Review round 3 の是正）。route の `product` が外れても（サイドバーの「棚卸し」の link で外れても、別の商品の読込みの途中でも、保存の応答の保留中でも）欄は開いたまま残り、その商品の保存の成功で閉じる（途中の読込みの応答は捨てる）。保存の成功で閉じる時点で route の `product` が外れていれば、保存を始めた時に捕らえた戻り先（`returnTo`）へ戻らずに閉じて `/stocktake/` に留まる。外れていなければ（route が別の商品へ変わり、その読込みが失敗した場合を含む）今のまま保存の時の `returnTo` へ戻る。保存の成功の後始末（閉じる・戻る）を行う時点で棚卸しの画面を離れていれば（サイドバーで別の画面へ移り、画面が unmount した）、戻らず、閉じる操作の route の更新もしない（どちらも route の遷移で、離れた先から商品フォームか棚卸しの画面へ連れ戻す）（画面の小口の lane〈[packet](../plans/2026-10-27-stocktake-screen-fixes.md)〉で決めた） |
| 保存 | 保存した数・在庫（写し・範囲）・差（35 の新契約の「読取り」）を反映し、保存済みの feedback を出して検索欄へ戻る。要再確認になれば理由を文字で出す |
| 日付の変わり目 | 画面を開いたまま PC の日付が前の保存から変わったら、保存の前に「日付が変わりました（{前} → {今日}）。今日の分として保存しますか」を確かめる |
| 要再確認の一覧 | 棚卸しの有無にかかわらず、画面の上部に「数え直しが必要な商品」の一覧（[35](35-biz-stocktake-service.md) の `list_recheck_products`。各行は商品・範囲〈単位どおり。[40](40-cmd-product.md) の `CountRecoveryTarget` の範囲〉・理由）。棚卸しの明細の無い商品の「今の数を入れる」の欄の商品の文脈は [42](42-cmd-sales-stocktake.md) の `preview_stock_count_placement` の結果から読む。各行に「入れ直す」（`fresh=true`。行が `stocktake_item_id` を持てば一緒に送り、明細の実数の表示を置き換わった計数に揃える。ADR D5） |
| 完了 | 既存の常時の確認 dialog と取消不可の警告を保ち、確認の文に「今日の最後の精算（締め）を取り込みましたか。取り込む前に完了すると、その精算の売上は棚卸しに入りません」を足す（アプリが判定できない最後の精算の取込み忘れ。ADR D6 の H2）。`completion_blocked` なら、`block_reason` ごとに「数え直しが必要」「まだ数えていない」「取り込んでいない精算がある」の一覧を出す。「取り込んでいない精算がある」には、[40](40-cmd-product.md) の `SettlementPendingHint` から直し方を文で添える（`missing` があれば「精算 {n}〜{m} を取り込んでください」〈1 つなら「精算 {n} を取り込んでください」〉、`on_or_after` があれば「精算日が {日付} 以降の精算を取り込んでください」、`before` があれば「精算日が {日付} より前の精算を 1 本取り込んでください」。b1 の Plan Review round 1 の是正）。未計数の一覧に「0 にする」（選んだ商品）と「残りをまとめて 0 にする」（確認 dialog を挟む）。0 にする日付は「前の日の分を入れる」と同じく選べ、既定は今日（休業日に押したときは前の営業日を選べる）。帳簿の数とみなす選択は置かない |
| 完了の結果 | 評価額（総額）と明細の数。完了は在庫を動かさない（保存のたびに在庫へ入っている）と分かる文言 |
| 完了の後 | 棚卸しの記録の詳細に、完了の後に入った記録による差のある明細と評価額の差の合計を「参考（評価額は変わりません）」として出す。決まらない記録による差は「要再確認」と差の範囲で別に出し、確定の差の合計に入れない（[65](65-inventory-record-traceability.md)） |

数量は整数・非負・wire 安全範囲を検査し（D-113: m の商品の実数は m で小数 2 桁まで受けて cm に直し、一覧と入力欄の在庫・実数・差異と確定の結果の一覧を m で出す〈owner の決定 TD-203〉。そのため `StocktakeItemDetail` と確定の結果に `stock_unit: ProductStockUnit` を足す〈今はどちらも単位を持たない〉。実数の上限は在庫の上限 `999999999`〈長さは `9999999.99` m〉になる。[共通規則](10-common-rules.md) SPEC-UNIT-D3・D11 と「入力の上限と安全な整数の範囲」）、IME 確定の Enter では送信しない。保存中（数の保存・0 にする）の連打・商品の変更・離脱・確定を抑止し、遅い応答が別の商品へ当たらないよう応答の商品コードを照合する（検索・「入れ直す」の読込み・前の日の分の日付の読取りは、最後の操作の応答だけを当てる）。既存のスキャン検索や部門の候補は検索結果の変化で選べなくしない。

- 要再確認・未計数・保存済みは日本語の label と icon を併用し、色だけで示さない。範囲は「4〜5」の形。
- UI の状態は未保存の入力だけで、保存済みの正は DB / query。D-052 の SSOT から、計数の保存・0 にする・完了・Z004 の取込みと取消・入出庫の保存で、写し（在庫・要再確認）を読む consumer と要再確認の一覧を invalidate する（entry は UI_TECH_STACK の D-118 の段落）。
- Windows L3 の対象（runtime）: (a) 検索 → 数 → 保存 → 次の商品、(b) 前の日の分の保存と「後の数が既にあり」の表示、(c) 締めの取込みの後の要再確認 → 一覧から入れ直して解消、(c2) 商品の「入れ直す」で打ち直し、(d) 完了の拒否の一覧 → 0 にする → 完了。(e) 日付の変わり目の確認は PC の時計を変える手順が L3 Eligibility の (3) に当たるので、L3 に入れず、時計を差し替える自動 test（Matrix の U5）と画面の写しで見る（b1 の起票で決めた）。
- b1 の画面の細部（2026-10-23、実装で決めた）: 「今の数を入れる」と要再確認の一覧の「入れ直す」は画面の上部の「今の数を入れる」の欄を開き（保存の button は「今の数を保存」。fresh な計数、日付は今日）、商品の文脈は `preview_stock_count_placement` から出す。検索で開く欄は「数を保存」（primary）と、前に保存した数があるときの「入れ直す」（outline）。「入れ直す」を出すときは、欄に「数を直すときは「入れ直す」を押してください。同じ日付で「数を保存」から違う数を入れると、数え直しが必要になります」を出す（同じ日付の違う値の既定の計数は要再確認〈ADR D3 の理由 3〉になるので、直す導線を名指す。b1 の Gated Amendment の 2 本目で、b1 の前の「入力済みの数を上書きできます」を置き換えた）。完了の拒否は「棚卸しを確定できない理由」の欄に `block_reason` ごとの group で出す。

> **親文書**: [FUNCTION_DESIGN.md](../FUNCTION_DESIGN.md)
> **入力ドキュメント**: [architecture/ui-task-specs.md](../architecture/ui-task-specs.md) UI-10、[DB_DESIGN.md](../DB_DESIGN.md)、[db-design/tracking-system-tables.md](../db-design/tracking-system-tables.md) §16-17（stocktakes / stocktake_items）、[35-biz-stocktake-service.md](35-biz-stocktake-service.md)、[42-cmd-sales-stocktake.md](42-cmd-sales-stocktake.md) §22.5、[UI_TECH_STACK.md](../UI_TECH_STACK.md) §7.2（10-4a）、[59-ui-shared-patterns.md](59-ui-shared-patterns.md)、[68-ui-backup-restore.md](68-ui-backup-restore.md) / [69-ui-threshold-settings.md](69-ui-threshold-settings.md)（構成の手本）
> **対応タスク / 仕様**: UI-10（REQ-205、棚卸し）
> **Design Phase**: 2026-07-07。運用一次入力は issue #135 Stage 5 ヒアリング（2026-07-07）+ 同日の価値提案訂正コメント。設計判断（UI-10-D1〜D9）は Design Phase packet `docs/archive/plans/2026-07-07-ui10-stocktake-design.md` で固定済み。runtime（route / components / hooks / specta 化と、新しい 3 command）は実装 PR #159（private archive、`docs/archive/plans/2026-07-07-ui10-stocktake-implementation.md`）で入り、本書の §73.1 以降（UI-10-D1〜D13 を含む）の契約は実装済み（冒頭の D-118 の節は D-118 の b1 で実装済み）。UI-10-D10 以降はその実装 PR のレビュー・契約監査で追記された。棚卸しの開始・入力・確定の停止（停止 ADR、superseded）は D-118 の b1 で外した。

## 73.1 目的

年1回（owner は多くても 2 回と述べた）・大晦日時点の棚卸し（税理士の指示で、報告するのは仕入原価の総額だけ。差異は棚卸しロスとして許容する。店主回答 2026-03、owner 2026-08-27）の作業を、開始・中断・再開・確定まで単独の operator が扱えるようにする。現運用は一人で商品マスタ全件（数千点規模）を回り、10月からの準備（値上げ品の原価・売価の訂正、新商品のリストへの追加、原価不明品の問い合わせ。準備の開始は 2026-08-15 の店主回答で前倒しされた）と、年末の正式カウント（大晦日に完了）に分かれる前提であり（ヒアリングシート C60/B60/Q12/Q13/Q20、店の運用の詳細は [project-memory](../project-memory.md) の Store Premises Facts）。UI は CMD-10 の 11 command（[42](42-cmd-sales-stocktake.md) の CMD-10。数の保存は `save_stock_count`、未計数の片付けは `zero_uncounted_items`）を生成された bindings から使う（UI-10-D7）。

**カウント対象の母集団（issue #91 owner 回答 2026-08-22）**: 画面がカウントさせる母集団は商品マスタ全件（[35-biz-stocktake-service.md](35-biz-stocktake-service.md) `find_stocktake_eligible_products` が返す全商品）であり、年数による除外はしない。実店舗で現物カウントの対象外になるのは、伝票保管義務範囲外で廃棄済み・取引先データなし・バーコードなし・販売に適さない見た目という原価根拠を欠く現物（例年 1〜2 点、多い年で 4〜5 点）で、これらは単品コードを付与せず商品マスタへ未登録のため、システム上は最初から母集団の外にある。除外を表す専用フラグや画面表示は追加しない。

## 73.2 関数要求

| ID | 要求 |
|---|---|
| UI-10-F1 | `/stocktake` 到達時に新規 CMD `get_active_stocktake()`（DB 問い合わせ、§73.8）で進行中棚卸しの有無を判定し、あれば継続表示（開始日時・進捗）、なければ開始 CTA + 前回完了サマリを表示する。 |
| UI-10-F2 | 開始 CTA 押下で `start_stocktake` を呼び、成功後は継続表示（カウント画面）へ切り替える。 |
| UI-10-F3 | 部門フィルタ・未入力のみ toggle・進捗（counted/total）付きの一覧を `get_stocktake_items` で取得・表示する。 |
| UI-10-F4 | 検索/HID スキャンで商品を特定し、数量入力 → `save_stock_count` で 1 件即保存する（既定の計数。前に保存した数を今見て直すときは「入れ直す」の fresh な計数。冒頭の D-118 の節）。counted 済み商品への再入力を同一導線で常時許可する。 |
| UI-10-F5 | 確定 CTA 押下時、常に確認ダイアログ（取消不可の警告と最後の精算の問い）を経て `complete_stocktake` を呼ぶ。拒否（`completion_blocked`）なら理由ごとの一覧と「0 にする」、成功なら結果画面（`total_cost` 主役 + 明細の数 + 在庫は動かない旨 + `integrity_result`）を表示する。 |
| UI-10-F6 | 開始前画面・確定結果画面に、前回完了棚卸しの `total_cost` / `completed_at`（`get_last_completed_stocktake`）を併記する。 |
| UI-10-F7 | 棚卸し中の商品登録・一括インポートによる明細自動追加（BIZ-01 実装済み）を、query invalidation / 再取得で自然に一覧へ反映する。専用通知は出さない。 |
| UI-10-F8 | `complete_stocktake` は単一 TX のため IPC channel を使わず、spinner + ボタン disabled で進行中を示す。 |

## 73.3 Design Decisions

### UI-10-D1: 開始/再開は同一画面での自動判別、明示 resume コマンドなし、中止機能なし

- **決定**: `/stocktake` 到達時に新規 CMD `get_active_stocktake()`（`stocktake_repo::find_active_stocktake` の BIZ 薄 wrapper、§73.8）を呼び、`Some(Stocktake)` なら継続表示（開始日時・進捗）にそのまま入る。`None` なら開始 CTA + 前回完了サマリを表示する。進行中判定の真実のソースは常に DB（`stocktakes.status`）であり、クライアント側の永続状態（localStorage 等）を判定に使わない。「再開する」という明示ボタン・コマンドは存在しない。棚卸しの途中破棄（中止）機能も作らない。
- **Why**: BIZ-06 に resume 専用の関数はなく、`start_stocktake` は進行中があれば `StocktakeInProgress` を返すだけである。実態にない機能を UI 側で作ると、存在しない中断状態（一時停止 vs 進行中）の区別を UI が勝手に発明することになる。中断・再開の実体は「1 件ずつ保存した計数をいつでも続けられる」ことそのものであり、専用 UI 状態を持つ必要がない。`find_active_stocktake` は IO 層に既存だが CMD として公開されていなかったため（本 Design Phase の初版で見落とし、2026-07-08 PR #159 レビューで顕在化）、新規 CMD として公開する。
- **Rejected**: 明示的な「一時中断」「再開」ボタン（BIZ に対応する状態がなく、フェイクの状態遷移を UI に作り込むことになる）。棚卸しの中止（途中破棄）機能（35-biz §20.7 の非目的を UI 側でも踏襲。年1回の作業を無に返す操作は非IT operator の事故リスクが高く、要求もない）。**進行中判定をクライアント側 `localStorage` に持たせ、復帰は `start_stocktake` の `stocktake_in_progress` エラーメッセージ文字列を正規表現でパースして ID を抽出する方式**（PR #159 で実装されたが却下。理由: ①バックエンドの人間可読エラーメッセージ（35-biz `StocktakeInProgress` の文言）をプログラム契約として利用しており、文言変更で復帰導線がサイレントに壊れる。②別端末・アプリ再インストール・WebView プロファイルリセット等で `localStorage` が失われると、進行中の棚卸し（数週間規模の単独作業）に到達する手段が「開始ボタンを押してエラー経由で推測復旧する」という回りくどい導線に落ちる。③本 Spec Contract の「status 自動判別」を実装が満たさない。新規 CMD 1 本（薄い wrapper）の追加コストの方が明確に小さい）。
- **Revisit trigger**: BIZ-06 に明示的な resume/中止 API が追加された場合。
- **契約監査追記（2026-07-08）**: PR #159 の契約監査で、継続表示ヘッダが本決定・UI-10-F1・73.10 Wording が明記する「開始日時」ではなく内部 `stocktake_id` を表示していた実装漏れが見つかった。バックエンド（`get_active_stocktake`）は `started_at` を返しているのに、`useStocktakeStatus` フックも `StocktakeProgressHeader` コンポーネントもこれを一切参照していなかった。`棚卸し中（ID: {stocktakeId}）` → `棚卸し中（開始日: {formatCountedAt(started_at)}）` に修正。

### UI-10-D2: カウント主動線は検索/スキャン→数量入力→即保存、counted 済みへの上書き再入力を常時許可

- **決定**: 主動線は「検索/HID スキャンで商品を特定 → 数量入力 → 1 件即保存」で、保存は `save_stock_count`（1 商品 1 TX。b1 で旧い数の上書きの command を置き換えた）。商品の特定は新規 CMD `find_stocktake_item({ stocktake_id, code })`（§73.8。`code` は商品コードまたは JAN の完全一致で該当明細を 1 発解決）で行う。counted 済み（`actual_count` が既に入っている）商品への再入力を、未入力商品と全く同じ導線・確認なしで常時許可する（上書き＝都度訂正の代替）。（b1 の注記: 同じ日付で「数を保存」から違う数を入れると要再確認〈ADR D3 の理由 3〉になるので、数を直す導線は「入れ直す」〈確認なし〉。欄の文は冒頭の D-118 の節）プレカウント専用の別機能は作らない。
- **Why**: 現運用は部門キー売り中心で、CSV/PLU が未導入の間は商品単位のレジ販売が `sale_records` / `inventory_movements` に反映されない（D-025）。そのため「カウント後に売れた分の訂正」はシステム側にも訂正対象データが存在せず、operator が現物を見て再入力する以外に手段がない（issue #135 の訂正コメント 2026-07-07 で明示）。上書き保存を特別扱いすると、まさに一番必要な訂正導線に確認ダイアログという追加の摩擦を課すことになる。
- **Rejected**: BIZ 確定ロジック側で `counted_at` 以降の在庫変動を自動織り込む（現運用では織り込むべき商品単位データ自体が存在しないため実装不能。将来 Z004/PLU 運用が始まったら再検討）。上書き時だけ確認ダイアログを挟む（都度訂正が前提の運用で毎回ダイアログを出すのは実質妨害になる。DSR-07 も可逆操作には確認なしを原則としており、数の保存は何度でも入れ直せる）。プレカウント専用モード（上書き入力で用が足りる）。クライアント側で全明細ページを順次取得して `product_code → stocktake_item_id` マップを構築する解決方式（数の保存の成功ごとに一覧 invalidation（§73.11）が走るため 1 件保存のたびに数千件・複数ページの再取得とマップ再構築が反復する。部門を跨いでスキャンする使い方にも弱い。薄い解決 CMD 1 本（BIZ wrapper 込み）の追加コストの方が明確に小さい）。
- **Revisit trigger**: Z004/PLU 運用が始まり商品単位の販売データが `sale_records` に載るようになった時。
- **契約監査追記（2026-07-08、owner L3 実機発見）**: §73.5 は Design Phase 初版から「商品名で探したい場合は既存の商品検索（`commands.searchProducts`）で候補から選び、その `product_code` を使う」と明記していたが、実装から丸ごと漏れていた。owner が実機 L3 で「棚卸し中に新商品を登録し、その商品名で棚卸し検索欄に入力する」という自然な操作を行った際、`find_stocktake_item` は商品コード/JAN の完全一致のみ検索するため `None` を返し、「この商品は棚卸しの対象にありません」という誤解を招くメッセージが出た（一覧には UI-10-D9 の自動追加で実際に存在しているのに、検索では見つからないという矛盾した体験）。`resolveItem` を拡張し、`find_stocktake_item` が `None` を返した場合に `commands.searchProducts`（`ReceivingPage.tsx` の商品名検索と同じ経路・クエリ）へフォールバックする。0 件は既存の「対象にありません」文言、1 件は自動的にその `product_code` で `find_stocktake_item` を再実行して選択、複数件は候補テーブル（商品コード/商品名/部門 + 選択ボタン、`ReceivingPage.tsx` の候補表示パターンを踏襲）を表示する。T21/T22 追加。
- **契約監査追記2（2026-07-08、Codex レビュー）**: 商品名検索欄に日本語 IME の変換確定 Enter でも `resolveItem` が発火する欠落があった（`event.nativeEvent.isComposing` guard 未実装）。`ReceivingPage.tsx` の商品検索欄には既にこの guard があり、今回追加した商品名検索フォールバックが「同じパターン」を謳う以上この点でも一致させる必要があった。検索欄・数量入力欄の両方の `onKeyDown` に `if (event.nativeEvent.isComposing) return;` を追加（数量欄は `inputMode="numeric"` だが全角数字入力等の IME 経由ケースへの防御として追加）。T23 追加。
- **契約監査追記3（本 PR、UI ガッツリ整えターン disposition culling 起源）**: 3 点を是正する。①`selectItem` が対象切替時に `setFieldError(null)` を呼ばず、live 候補選択・商品名検索フォールバックの単一候補自動選択を経由した場合に前商品の数量 FieldError が残留していた（PR #27 L3 owner 観察 2026-09-02）。対象商品が切り替わる全経路の集約点である `selectItem` へ `setFieldError(null)` を追加して是正する。②§73.9 の「対象にありません」はエラー扱いにしない契約だが、実装は `role="alert"` + `text-destructive` の destructive 表示になっていた（doc↔実装 drift）。`role="status"` + `text-muted-foreground` + `Info` icon の情報系表示へ是正する（DSR-08/DSR-11 準拠）。③商品名検索フォールバック（`PRODUCT_NAME_SEARCH_QUERY`）が `is_discontinued: false` 固定のため、棚卸し item 母集団（廃番商品を含む、§73.1 issue #91 回答済み）と商品コード/JAN 完全一致検索（`find_stocktake_item`、母集団の制限なし）には存在する廃番商品が、名前検索経路だけ候補から漏れる非対称があった（PR #27 L3 owner 観察 2026-09-02）。名前検索フォールバックの `PRODUCT_NAME_SEARCH_QUERY`（棚卸し画面ローカル）は `is_discontinued: null`（`ProductSearchQuery` の `None` は「全件、フィルタなし」を意味する、`src-tauri/src/db/product_repo.rs` の `search_products` の `query.is_discontinued` 実読で確定）へ変更する。live 候補プレビュー（catalog ⑮）側は 5 画面共有の `useProductAddSuggest.ts` の default `PRODUCT_SEARCH_QUERY`（`is_discontinued: false`）を直接変更せず、新設の optional `queryOverrides`（catalog ⑮ SPEC-SUGGEST-D13、下記 docs 是正）で棚卸し画面だけ `{ is_discontinued: null }` を渡す（他 4 画面は default のまま廃番を除外し続ける）。両経路の候補行に廃番 Badge（catalog `02-component-catalog.md` の ③ テーブル の例と同型）を追記して是正する。

### UI-10-D3: 一覧は進捗管理用、ソートは既存のまま変更しない

- **決定**: 一覧は部門フィルタ + 未入力のみ toggle（`counted_only`）+ 進捗（counted/total）を表示する進捗管理ビューとして設計する。並び順は `list_stocktake_items` の既存 `ORDER BY si.id ASC` のまま変更しない（IO 変更なし）。
- **Why**: 主動線が検索/スキャンによる商品特定であり、一覧をスクロールして目的の商品を探す使い方を前提にしない。棚の物理的な並びはアプリが把握できる情報ではなく、`id` 順（＝棚卸し明細の生成順）を並べ替える業務的な意味が薄い。
- **Rejected**: 商品名/コード順や未入力優先ソートの追加（IO 層の `ORDER BY` 変更を伴う。主動線が検索/スキャンである以上、一覧の並び順を工夫する投資対効果が低い）。
- **Revisit trigger**: 一覧を目視でスクロールして探す使い方が実際の運用で主動線になった場合。

### UI-10-D4: 確定は常時確認ダイアログ（文言は未入力件数で分岐）、結果画面は total_cost が主役

- **決定**: 確定 CTA「棚卸しを確定する」は 1 個（DSR-01）。押下時は**常に**確認ダイアログを挟む。`progress.uncounted_items > 0` なら「未入力の商品が{N}件あります。現在の在庫数で確定しますか？」（確認後 `force_fill=true`）、`uncounted_items === 0` なら「棚卸しを確定します。確定後は取り消せません。」（確認後 `force_fill=false`）。確定結果画面は `total_cost`（税理士報告値）を主役に、`adjusted_items` 差異一覧、`integrity_result`（None 時はフォールバック文言）を並べる。
- **Why**: `complete_stocktake` は在庫を書き換える確定操作で、**確定の取り消し API が BIZ-06 に存在しない**。誤タップで確定した場合の回復手段は「新しい棚卸しを開始して全件数え直す」しかなく、数週間分の単独作業が失われる非対称に大きい損失になる。DSR-07（不可逆操作の直前確認）の対象として、儀式化の懸念より誤操作コストを優先する。未入力がある場合はさらに force_fill の意味（未入力を現在庫数とみなす）を説明する必要があるため、文言を分岐する。
- **Rejected**: 全件入力済み時の確認なし直確定（誤タップ時の損失が数週間分の再カウントと非対称に大きく、確定取消 API がない。68-ui-backup-restore が復元操作に確認を課すのと同じ risk profile）。常時同一文言のダイアログ（未入力ありの場合は force_fill の意味説明が必要で、説明なしの汎用文言では判断材料にならない）。
- **Revisit trigger**: BIZ-06 に確定取消（reopen）API が追加された場合、全件入力済み時の確認省略を再検討してよい。
- **b1 で置き換えた（D-118）**: force_fill（未入力を現在庫数とみなす）と在庫を書き換える確定は無くなった。ダイアログは常に title「棚卸しの確定」+ warning Alert（「確定すると取り消せません」、本文「入力した内容で棚卸しを確定します。」と最後の精算の問い）。未計数・要再確認・取り込んでいない精算は BIZ が `completion_blocked` で拒み、画面は理由ごとの一覧と「0 にする」「残りをまとめて 0 にする」を出す。結果画面は `total_cost`・明細の数・在庫は動かない旨（補正の一覧は無い）。

### UI-10-D5: 前回完了棚卸しとの比較を軽量 CMD 1 本で追加する

- **決定**: 開始前画面と確定結果画面に、前回完了棚卸しの `total_cost` / `completed_at` を併記する。新規 CMD `get_last_completed_stocktake()`（IO query + BIZ 薄 wrapper + CMD）を追加し、戻り値は `Option<LastStocktakeSummary { stocktake_id, completed_at, total_cost }>`。
- **Why**: ヒアリング⑥「合計が前年と極端に違わないか見る」という既存の目視確認をシステム側で軽く支援する。棚卸し履歴の一覧画面や比較グラフのような重い機能は過剰。
- **Rejected**: 棚卸し履歴一覧画面の新設（過去複数回分の一覧・詳細は要求されておらず、直近1回との比較で足りる）。前回比較なし（ヒアリングの目視確認要求に応えられない）。
- **別経路での充足（2026-08-27 owner 裁定）**: `/inventory/records` への横断合流（[65-inventory-record-traceability.md](65-inventory-record-traceability.md) slice 4d）で代替充足し、専用一覧 `/stocktake/records` と `listStocktakeRecords` は runway 残置とする。
- **Revisit trigger**: 複数年分の推移を見たいという要望が出た時。
- **契約監査追記（2026-07-08、Codex）**: `complete_stocktake` 成功後、§73.11 の契約どおり `lastCompleted` query を invalidate すると、`get_last_completed_stocktake()`（`ORDER BY completed_at DESC, id DESC LIMIT 1`）は「今確定した棚卸し自身」を返す（確定処理が先に `stocktakes.status` を `completed` にするため）。結果画面はその再取得後の値をそのまま表示していたため、「前回の棚卸し」が今回確定した棚卸し自身に置き換わるバグがあった（既存 T11 の `mockGetLast` が常に同一値を返す実装だったため検出できていなかった）。`handleCompleteConfirm` で `complete_stocktake` 呼び出し直後・invalidate 前の `lastCompletedQuery.data` を local state にスナップショットし、結果画面にはそのスナップショットを渡すよう修正（invalidate 自体は次の未開始画面表示のために維持）。T19 追加。

### UI-10-D6: IPC channel 不採用（10-4a 判定を「不採用」で確定）

- **決定**: `complete_stocktake` は単一 TX で進捗の粒度が存在しないため、Tauri channel は使わない。spinner + ボタン disabled のみで完了を待つ。[UI_TECH_STACK.md §7.2](../UI_TECH_STACK.md) の 10-4a 判定はこの PR で「不採用」として閉じる。
- **Why**: UI-07（CSV 取込み）の 8-2b 判定と同根拠。`complete_stocktake` は完了の拒否の判定・評価額とカーソルの固定を 1 トランザクション内で行い（整合性チェックは TX の後）、途中経過を刻める中間状態がない。
- **Rejected**: channel 経由の進捗表示（実装コストに見合う体験改善がない。§7.2 の再検討トリガ「UI-10 で channel 採用が決まったら CSV 取込みへ展開」はこの判定により不成立で閉じる）。
- **Revisit trigger**: `complete_stocktake` が商品単位のバッチ処理に分割される設計変更が入った時。

### UI-10-D7: specta 化は既存 4 + 新規 3、生 invoke 禁止を維持

- **決定**: 既存 4 コマンド（`start_stocktake` / `get_stocktake_items` / `update_count` / `complete_stocktake`）と新規 `get_active_stocktake` / `find_stocktake_item` / `get_last_completed_stocktake` の計 7 本に `#[specta::specta]` を付け、bindings を生成する（b1 で数の上書きの command を外し、計数の 4 command〈42 の D-118 の節〉を足した。実装 PR #159〈private archive〉で付けた。今は `src-tauri/src/cmd/stocktake_cmd.rs` の 11 本〈42 の CMD-10〉に属性があり、`src/lib/bindings.ts` に生成された関数がある。Design Phase の時点では既存 4 本に属性が無かった）。UI からの生 `invoke()` 直呼びは既存の禁止方針（ADR-004）を維持する。
- **Why**: ADR-004 の生 invoke 禁止方針を UI-10 でも一貫させる。既存 4 本が specta 未対応のまま長期間放置されていた事実は本 Design Phase で顕在化したため、実装 PR の Rust 差分に明記する。
- **Rejected**: 実装 PR 内での暫定的な生 invoke 経由実装（撤去忘れの前例＝ 7-5c fallback を再演する）。

### UI-10-D8: エラーは kind 別に専用ハンドリングする

- **決定**: `CmdError.kind` が `stocktake_in_progress`（`start_stocktake` を進行中に呼んだ場合）なら、開始 CTA 経路を止めて継続表示へ切り替える。`stocktake_not_in_progress`（`save_stock_count` / `zero_uncounted_items` / `complete_stocktake` を完了済み棚卸しに対して呼んだ場合、他端末や再読み込みのタイミング差で発生し得る）なら、棚卸し状態を再取得して完了済み/未開始表示へ切り替える。`validation`（数量の形と範囲、日付、`page`/`per_page` 不正、合計金額の範囲の外）は FieldError か上部 Alert で回復する。`stocktake_guard`（`completion_blocked`）は理由ごとの一覧で回復する（b1）。`integrity_result: None`（整合性チェック失敗）は「整合性チェックは実行できませんでした」のフォールバック文言で表示する。
- **Why**: BIZ-06/CMD-10 は既に 3 つの意味あるエラー種別（`StocktakeInProgress` / `StocktakeNotInProgress` / `ValidationFailed`）を返すため、UI 側で汎用エラー表示にまとめると回復導線が失われる。
- **Rejected**: 全エラーを共通の destructive Alert にまとめる（`stocktake_in_progress` は実は「継続表示に戻ればよいだけ」の非エラー的状況であり、汎用エラー表示では誤解を招く）。
- **契約監査追記（2026-07-08）**: PR #159 の実装レビューで、UI-10-D11（フォーカス管理漏れ）を owner 確認質問で発見した後、73 全体を実装コードと突き合わせる監査を行った結果、`stocktake_not_in_progress` の実装漏れが追加で見つかった。`get_stocktake_items`（一覧取得）の `stocktake_not_in_progress` は invalidate 処理があったが、カウント保存と `complete_stocktake`（確定）で発生した場合は kind 判定・invalidate ともに未実装で、汎用エラー表示のまま画面が完了済み状態に固まっていた。加えて表示文言もバックエンドのエラーメッセージ文字列をそのまま出しており（たまたま UI 側の想定文言と一致していたため既存テストでは検出できなかった）、UI 固定文言としてハードコードされていなかった。3 箇所（一覧取得・カウント保存・確定）すべてで共通ヘルパー `isStocktakeNotInProgressError` による kind 判定 + UI 固定文言 + invalidate に統一した。§73.9 のエラーテーブルに詳細を追記。
- **追記2（2026-07-08、owner 指摘起因）**: force_fill 未入力超過 `validation` エラーについて、owner から「発生条件がそこまで珍しくないのでは」という指摘を受けて再検討した。当時の商品登録画面（`ProductFormPage.tsx`）は自身の商品一覧クエリしか invalidate しておらず、確定直前に新商品登録を挟んだ場合に到達し得た。D-052-C1/C3 で商品登録・一括 import から棚卸し一覧への追随を追加した後も、別端末更新や race への防御として `handleCompleteConfirm` の validation error refresh は維持する。kind 判別不能という制約から専用再試行ボタンは作らず、次回操作を最新 `uncounted_items` に収束させる。T20 で検証する。（b1 で force_fill を外し、未計数は `completion_blocked` の理由ごとの一覧で示す。T20 は契約ごと外し、U7 が止める。）

### UI-10-D9: カウント中の明細自動追加は通知なしで自然反映

- **決定**: 棚卸し中に商品登録・一括インポートが行われて `stocktake_items` に明細が追加される（BIZ-01 実装済み）動作は、UI 側では専用の通知を出さず、一覧の query invalidation / 再取得によって自然に反映されるだけとする。
- **Why**: 追加自体は BIZ-01 側で完結しており、UI が能動的に検知するイベント経路（push 通知等）を持たない。一覧を開き直す・フィルタを変える操作のたびに最新状態が見えれば十分であり、追加専用の通知は実装コストに見合わない。
- **Rejected**: 新規追加分をハイライトする専用 UI（追加検知のための差分比較ロジックが必要になり、投資対効果が低い）。

### UI-10-D10: 一覧の差異・最終カウント列 + 確定警告の視認性 + 前回比較の独立表示（実装レビュー起因の追記）

以下は決定の履歴。b1 で差異は BIZ が求める差異（[35](35-biz-stocktake-service.md) の「読取り」の `difference`）に置き換え、画面は計算しない。在庫列は写し（要再確認なら範囲と「要再確認」）。警告の視認性・前回比較の独立表示は維持する。

最終カウント列は、画面の小口の lane（[packet](../plans/2026-10-27-stocktake-screen-fixes.md)。b1 の L3 の owner の所感、repo 外の回答台帳 TD-433）で、保存の時刻（`counted_at`）から効く計数の業務日（`count_business_date`。「前の日の分を入れる」で選んだ日付、既定は保存した日）に置き換えた（owner の決定 2026-10-27、repo 外の回答台帳 TD-452: A〈業務日だけ〉）。
- **Why**: b1 から計数は業務日で効き（ADR D3）、「前に保存した数」の欄（§73.5 の 3）と棚卸しの記録の詳細（[65](65-inventory-record-traceability.md)）は業務日を出す。一覧だけが保存の時刻を出すと、前の日の分を入れたのに一覧に今日と出て、入れた日付が効いたかを一覧で確かめられない。
- **Rejected**: 保存の時刻のまま（上の迷いが残る）。業務日と保存の時刻の両方（1 つの列に日付と時刻の 2 つの種類が混ざり、どちらが効く日か読み分けが要る。保存の時刻は業務記録の側で引ける）。
- 計数が無い（`actual_count` が null）か業務日が null の行は「—」。保存の時刻へ戻して埋めない（列に 2 つの種類を混ぜない）。
- **Revisit trigger**: 同じ業務日に何度も入れ直す使い方で、どれが最後かを一覧で見分ける要望が出たとき。

- **決定**: 2026-07-08、Windows native L3 実機観察（PR #159）と実装コード確認を受けて 3 点を追加・是正する。
  1. **一覧に「差異」「最終カウント」列を追加**する。差異は `current_stock - actual_count`（`update_count` の `current_difference` と同一計算式、35-biz §20.4「差異の動的計算」を一覧にも適用）。`actual_count` が `null`（未入力）の行は差異・最終カウントとも「—」。列表示は符号付き数値のプレーンテキストのみとし、結果画面の `adjusted_items` テーブル（既存、色分けなし）と表現を揃える（新規に色分けを導入しない）。最終カウントは既存 `formatMovementDateTime`（`src/features/stock-movements/lib/movement-formatters.ts`）と同じ `T` 区切り→スペース変換を流用する。**既存の「システム在庫」列（`stocktake_items.system_stock`、開始時点の参考値）は「現在在庫」列に改め、表示値を `current_stock` に揃える**（カウント入力欄の選択商品情報表示も同様）。差異の計算根拠と画面表示の在庫値が異なる列に見えると、棚卸し中に在庫が動いた場合（35-biz §20.4 の前提どおり CSV 取込み等で変動しうる）に「現在在庫 10 / 実際 9 / 差異 +3」のような数値的に矛盾した表示になるため、両者を同一ソースに統一する（実装レビュー P2 是正、2026-07-08）。
  2. **確定確認ダイアログに warning Alert（`border-warning bg-warning-soft text-warning-strong` + `AlertTriangle`）を `AlertDialogContent` 内へネストし、不可逆性を視覚的に強調する**。実装（PR #159）は `AlertDialogTitle` が「未入力の商品があります」という状態説明に留まり、取り消せないことを伝えていなかった。一次是正として `RestoreConfirmDialog`（`BackupRestorePage.tsx`）のタイトル強調パターンに倣い「棚卸しを確定します（取り消せません）」に統一したが、owner が Before/After 比較 Artifact を見た上で **warning Alert ネスト版の視認性を明示的に支持**（2026-07-08）。`AlertDialogTitle` は未入力有無で状態を示す文言に戻し（未入力あり「未入力の商品があります」/ なし「棚卸しの確定」）、`AlertDialogContent` 直下に常時 warning Alert を配置して `AlertTitle`「確定すると取り消せません」+ `AlertDescription`（未入力有無で分岐）で不可逆性を伝える。
  3. **結果画面の前回比較を、総額カードから独立したカードに分離**する。既存実装は `formatLastStocktake()` の1文をコストカード内に注記として同居させており、独立した情報として認識しにくい。
- **Why**: L3 実機観察で「警告文言が小さく薄いグレーで正確な文言を記録できなかった」という指摘、および一覧に差異・カウント日時が出ないため実カウントとシステム在庫の乖離を目視で追えないという指摘。原因はコード確認の結果、①既存の design-system 警告パターン（`BackupRestorePage.tsx` の `RestoreConfirmDialog`）を踏襲していなかったこと、②本設計書（UI-10-D2〜D9 初版）が一覧の情報構造・確定ダイアログの文言強度をここまで具体的に指定していなかったこと、の両方に起因する。warning Alert ネストへの再変更は、タイトル強調のみの一次是正版と Artifact の warning Alert 版を owner が実際に見比べ、後者を明示的に選んだため（2026-07-08）。
- **Rejected**: 差異列の色分け（正=success系・負=destructive系）（結果画面の `adjusted_items` テーブルが無色のプレーンテキストであり、進行中一覧だけに新しい色分け表現を導入すると同じ「差異」という概念の表現が画面間で割れる。DSR-01 の視覚言語継承を優先）。
- **Revisit trigger**: 結果画面の差異テーブル自体に色分けを導入する設計変更が別途入った場合、進行中一覧の差異列も合わせて見直す。`RestoreConfirmDialog`（`BackupRestorePage.tsx`）も同様に warning Alert ネスト版へ揃えるかは、他の不可逆操作ダイアログでも同じ視認性懸念が出た時に横展開を検討する（本 PR のスコープでは UI-10 のみ変更）。

### UI-10-D11: 連続 HID スキャンのためのフォーカス自動遷移（実装漏れの是正）

- **決定**: 2026-07-08、owner からの確認質問（「他画面のようにカーソルを置いたまま連続スキャンできるのか」）を受けて実装を確認した結果、§73.5 が既に定めていた「解決できたら数量入力欄にフォーカスを移す」という契約が実装（PR #159）に反映されておらず、初期フォーカス・保存後のフォーカス復帰も未実装だったことが判明。`StocktakeCountEntry` に次の 3 つのフォーカス遷移を追加する。
  1. 画面表示時（コンポーネント mount 時）、検索/スキャン欄（`stocktake-code`）に自動フォーカスする。
  2. `find_stocktake_item` が該当明細を解決したら、数量入力欄（`stocktake-actual-count`）に自動フォーカスする（§73.5 の既存契約の実装漏れ是正）。
  3. 数の保存が成功したら、検索/スキャン欄に自動フォーカスを戻す（次のスキャンに備える）。
  1（mount 時）は `useEffect(() => { codeInputRef.current?.focus(); }, [])` で直接 `.focus()` を呼ぶ。mount 後の `useEffect` は DOM コミット後に実行されるため、`window.setTimeout` による遅延は不要。2・3（`find_stocktake_item` 解決成功後 / 数の保存の成功後）は、状態更新に伴い DOM 構造が切り替わる箇所であり、`ReceivingPage.tsx`（UI-02）の `addProduct` / `resetForm` と同じ理由で `useRef` + `window.setTimeout(() => ref.current?.focus(), 0)` パターンを踏襲する（state 更新後のレンダリング完了を待ってから focus する）。合わせて、数量入力欄に `onKeyDown` の Enter ハンドラを追加し、キーボードで数量を入力した場合も Enter で保存できるようにする（HID スキャナは数量欄では使わない想定だが、キーボード操作の対称性のため）。
- **Why**: HID スキャナ運用（`docs/UI_TECH_STACK.md` §5.3、memory `barcode_scanner_ux.md`）は「フォーカスが正しい入力欄にある」ことが前提条件。フォーカス管理が欠けていると、1 件目はスキャンできても 2 件目以降は毎回マウスでクリックし直す必要があり、「検索欄にカーソルを置いたまま連続スキャンする」という運用が成立しない。UI-02/UI-03（入庫記録・返品交換）では同じ課題に対して確立済みのパターンがあるのに、UI-10 だけそれが実装されていなかった。Codex の複数回のレビューでも検出されず、owner の実運用視点の質問で発覚した。
- **Rejected**: なし（実装漏れの是正であり、新規の設計判断選択肢はない）。
- **Revisit trigger**: `UI_TECH_STACK.md` §5.3 の「global scan detection」（連続文字入力の間隔での自動判定）を実店舗確認後に採用する場合、本フォーカス遷移ロジックとの整合を再検討する。

### UI-10-D12: 対象確認欄の live 候補プレビュー

- **決定**: 検索/スキャン欄（対象確認欄）に catalog ⑮（ProductAddSuggest、SPEC-SUGGEST-D1〜D11）の live 候補プレビューを追加する。suggest fetch は `commands.searchProducts`（部分一致）、候補確定は UI-10-D2 の `find_stocktake_item` 経由の既存経路で棚卸し対象化し、確定成功後の数量欄へのフォーカス遷移は UI-10-D11 の既存契約がそのまま発火する。候補確定後に解決が `None` の場合の無言 no-op 継承を含め、確定 semantics は既存 `selectCandidate` と同一。lock source は既存 `isCompleting`（`completeMutation.isPending`）派生 state（SPEC-SUGGEST-D4/D10）。
- **Why**: 商品名でしか探せない商品（JAN なし・コード不明）の対象確認が、現行の「Enter 検索 → フォールバック候補テーブル」より早く目視でき、誤 Enter 前に気付ける。コード/JAN スキャンの主動線（スキャン → 即解決 → 数量入力）は SPEC-SUGGEST-D1/D2 により不変。
- **Rejected**: 純 autocomplete 型（Enter 追加廃止）はスキャナ Enter が候補 async 読込み前に届くため退行、不採用（owner 方針 2026-08-04）。
- **Revisit trigger**: catalog ⑮ の契約変更時。

### UI-10-D13: 棚卸しリストの廃番 badge（2026-09-06）

- **決定**: `StocktakeItemDetail` に `is_discontinued: bool` を追加し（`find_stocktake_item_by_code` / `list_stocktake_items` 両 SQL 経路、既存 `JOIN products p` を利用、新規 JOIN 不要）、棚卸しカウント画面の一覧テーブル（§73.6）の商品名セルに、対象商品が廃番のとき「廃番」badge（`variant="secondary"` + `border-border`、icon なし）を表示する。同一 file 内の既存候補行 badge（対象確認欄の live 候補プレビュー、UI-10-D12）も同じ commit で `border-border` へ揃える。
- **Why**: 棚卸しは廃棄判断の基準になるため、廃番商品かどうかが一覧上で視認できる必要がある。badge 仕様は新規に決めず、design-system 正本（`04-backbone.md` 原則 4 / `01-decision-rules.md` DSR-22、②分類 badge の枠は tone 固有色または `--border`、icon は識別に必要な場合のみ可）をそのまま適用する。既存候補行 badge（icon なし、枠なし）も改定後ルールに未適合のため、同一画面に新旧 2 種類の badge が混在しないよう同じ commit で揃える。
- **Rejected**: `UncountedItem`（未計数の明細の内部構造体、frontend 非公開）への追従（消費者がなく不要）。他 file の既存 secondary badge（`ProductTable.tsx` / `ProductAddSuggest.tsx` 等）への枠 sweep（本 lane の対象外、Lane 3〜5 backlog のまま）。
- **Revisit trigger**: なし。

## 73.4 Route / 状態遷移 / Components

```
src/routes/stocktake.tsx                … layout route（Outlet だけ）
src/routes/stocktake/index.tsx          … 画面の route（validateSearch: dept/counted_only/page/product/returnTo、StocktakePage を描画）
src/routes/stocktake.records.$stocktakeId.tsx … 棚卸し記録の詳細（一覧は `/inventory/records` から入る。[65](65-inventory-record-traceability.md)）
src/features/stocktake/
  StocktakePage.tsx                     … 画面本体と部品を同じ file の export で持つ:
                                          StocktakePage（状態分岐のエントリポイント）
                                          StocktakeStartPanel（未開始時: 開始CTA + 前回完了サマリ）
                                          StocktakeProgressHeader（進行中: 開始日時 + 進捗〈counted/total〉+ 前回比較）
                                          StocktakeItemList（部門フィルタ + 未入力のみtoggle + 明細テーブル）
                                          StocktakeCountEntry（検索/スキャン入力 + 数量入力 + 即保存）
                                          CountFields（数量・保存の日付・「前の日の分を入れる」・「入れ直す」・日付の変わり目の確認）
                                          CompletionBlockedPanel（完了の拒否の理由ごとの一覧と「0 にする」）
                                          StocktakeCompleteDialog（確定確認ダイアログ〈取消不可の警告と最後の精算の問い〉）
                                          StocktakeResultPage（確定結果〈total_cost + 明細の数 + integrity_result + 前回比較〉）
  components/RecheckTargetList.tsx      … 要再確認の一覧（棚卸しの画面の上部と 55 の取込み・取消の結果）
  hooks/useStocktakeStatus.ts           … 進行中判定（CMD get_active_stocktake を叩く。DBが真実のソース。localStorage不使用）
  hooks/useStocktakeItems.ts            … getStocktakeItems を包む useQuery（部門/counted_only/page）
  hooks/useSaveStockCount.ts            … saveStockCount を包む useMutation（1件即保存）
  hooks/useCompleteStocktake.ts         … completeStocktake を包む useMutation
  hooks/useLastCompletedStocktake.ts    … get_last_completed_stocktake を包む useQuery
  hooks/useFindStocktakeItem.ts         … find_stocktake_item を包む解決 hook（§73.5）
  lib/stocktake-formatters.ts           … 表示の整形
```

- 共通部品は `patterns/PageHeader` / `patterns/DepartmentFilter` / `patterns/EmptyState` / `patterns/SearchBar`（既存 [59-ui-shared-patterns.md](59-ui-shared-patterns.md)）を再利用する。進捗表示は shadcn/ui `Progress`（[design-system/02-component-catalog.md](../design-system/02-component-catalog.md) 既存カタログ「確定的進捗」用途、CSV 取込み・バックアップに続く 3 例目の採用）。
- URL state（`dept` / `counted_only` / `page`）は [58-ui-stock-inquiry.md](58-ui-stock-inquiry.md) / [66-ui-stock-movements.md](66-ui-stock-movements.md) の `validateSearch`（zod 4）パターンを踏襲する。開始/未開始/確定結果のような画面全体の局面（`none` / `in_progress` / `completed`）は復元・共有する価値のある表示状態ではないため URL 化しない（ローカル state / query の有無で判定する）。

### シグネチャ（擬似コード）

```ts
// 進行中判定（新規CMD get_active_stocktake。DBへ直接問い合わせる。localStorage不使用）
function useStocktakeStatus(): UseQueryResult<Stocktake | null, InvokeError>

// 明細一覧（部門/counted_only/page）
function useStocktakeItems(params: {
  stocktakeId: number;
  departmentId?: number;
  countedOnly?: boolean;
  page: number;
  perPage: number; // IO 側で 200 上限にクランプされる（PAGINATION_MAX_PER_PAGE）
}): UseQueryResult<StocktakeItemListResponse, InvokeError>

// 計数の保存（1件即保存。既定の計数・fresh な計数）
function useSaveStockCount(): UseMutationResult<StockCountSaveResult, InvokeError, SaveStockCountRequest>

// 確定
function useCompleteStocktake(): UseMutationResult<StocktakeResult, InvokeError, {
  stocktakeId: number;
}>

// 前回完了棚卸し（CMD get_last_completed_stocktake）
function useLastCompletedStocktake(): UseQueryResult<LastStocktakeSummary | null, InvokeError>

// 検索/スキャン値 → 対象明細解決（CMD find_stocktake_item を包む）
function useFindStocktakeItem(): UseMutationResult<StocktakeItemDetail | null, InvokeError, {
  stocktakeId: number;
  code: string; // 商品コードまたは JAN の完全一致
}>
```

### 状態遷移（ローカル UI 状態）

| 状態 | 入る条件 | 表示 |
|---|---|---|
| `loading` | `get_active_stocktake` query 実行中 | Skeleton |
| `not_started` | `get_active_stocktake` が `None` | 開始 CTA + 前回完了サマリ（`get_last_completed_stocktake` が `None` なら「前回の記録はありません」） |
| `starting` | 開始 CTA 押下、`start_stocktake` 実行中 | ボタン disabled |
| `counting` | `get_active_stocktake` が `Some(Stocktake)` | 進捗ヘッダ + 一覧 + カウント入力 |
| `confirming_complete` | 確定 CTA 押下 | 確定確認ダイアログ（常に） |
| `blocked` | `complete_stocktake` が `completion_blocked` | 理由ごとの一覧と「0 にする」（`counting` の表示に重ねる） |
| `completing` | `complete_stocktake` 実行中 | spinner + 操作 disabled（UI-10-D6） |
| `completed_result` | `complete_stocktake` 成功 | 結果画面（`total_cost` / 明細の数 / `integrity_result` / 前回比較） |

## 73.5 カウント導線（処理ステップ）

1. `counting` 状態で、部門フィルタ・未入力のみ toggle を指定して `get_stocktake_items` を呼び、明細一覧（`StocktakeItemDetail[]` + `progress`）を取得する。
2. 検索/スキャン欄（対象確認欄。素の `Input` 直接実装で、catalog ⑮ ProductAddSuggest の live 候補プレビューを結線する — ⑨ SearchBar は一覧 filter 用のため本欄には用いない）に商品コードまたは JAN を入力する。HID スキャナは Enter キー入力として扱う既存パターンを踏襲する。商品名で探したい場合は既存の商品検索（`commands.searchProducts`、入出庫系画面の商品追加と同じ経路）で候補から選び、その `product_code` を使う。
3. 入力値（商品コード/JAN）を `find_stocktake_item({ stocktake_id, code })` に渡し、該当明細（`StocktakeItemDetail`）を 1 発解決する（UI-10-D2、§73.8）。解決できたら数量入力欄にフォーカスを移す。counted 済み商品を指定した場合も同じ導線で、数量欄は空のまま前に保存した数を読取りで出し、入力を許可する（UI-10-D2、冒頭の D-118 の節）。`None` の場合、`commands.searchProducts` で商品名検索にフォールバックする（UI-10-D2 契約監査追記）: 0 件は §73.9 の回復に従う。1 件は自動的にその `product_code` で再解決して選択。複数件は候補テーブル（商品コード/商品名/部門）を表示し、選択した `product_code` で再解決する。検索/スキャン欄は入力中に live 候補プレビュー（UI-10-D12、catalog ⑮）を表示し、プレビューからの候補確定も本 step の `find_stocktake_item` 解決と同じ経路を通る。
4. 数量入力後 Enter または「数を保存」で `save_stock_count({ product_code, count_value, fresh: false, business_date, stocktake_item_id })` を呼ぶ（「入れ直す」は `fresh: true`、日付は今日）。空欄・空白のみは送信前に FieldError「数量を入力してください」で止める（ST-C5-D1、DSR-05）。数量は 0 以上 `999999999`（m の商品は m の小数 2 桁を cm に直す）で、外れれば送信前に FieldError（BIZ 側も同じ検証を持つ防御的二重チェック）。保存の前に、前の保存から PC の日付が変わっていれば確認を挟む（冒頭の D-118 の節の「日付の変わり目」）。
5. 保存成功で「{商品名} を {数}（{日付}）で保存しました」（要再確認になれば理由を添える）を role="status" で出し、検索欄へ focus を戻す。1 件保存ごとの toast は出さない（4000 件規模の反復操作で toast が積み重なるとかえって妨げになるため）。
6. D-052 の SSOT の `stocktakeCountUpdate`（明細の一覧・写しの consumer・要再確認の一覧）を invalidate して次の入力に備える。

### 設計判断: 検索/スキャン値の解決は新規 CMD `find_stocktake_item` で行う

`get_stocktake_items` は `department_id` / `counted_only` / ページングのみを受け付け、商品コード・JAN・商品名による検索パラメータを持たない（`stocktake_repo::list_stocktake_items` 実装確認済み。`PAGINATION_MAX_PER_PAGE=200` のため商品マスタ全件を対象とする棚卸しでは 1 部門が複数ページに分かれ得る）。このギャップは主動線（UI-10-D2 の検索/スキャン → 即保存）の成立条件そのものであるため、クライアント側の回避策ではなく新規 CMD `find_stocktake_item({ stocktake_id, code })`（§73.8）で埋める。`code` は商品コードまたは JAN の完全一致で、該当明細を index 付き単一クエリで解決する。クライアント側マップ方式を却下した理由は UI-10-D2 の Rejected を参照。

## 73.6 一覧・フィルタ

| 要素 | 設計 |
|---|---|
| 部門フィルタ | `patterns/DepartmentFilter`。`department_id` を `get_stocktake_items` にそのまま渡す。 |
| 未入力のみ toggle | `counted_only=false` 相当（UI-10-D3。`counted_only` は 3 値: 指定なし/true/false）。ラベル「未入力のみ表示」。 |
| 進捗表示 | 「入力済み {counted_items} / 全 {total_items}」+ `Progress`（value = `counted_items / total_items * 100`、`02-component-catalog.md` 既存採用パターン）。`uncounted_items` は確定ダイアログの文言にのみ使う。 |
| 差異列（UI-10-D10） | BIZ の `difference`（35「読取り」。画面で計算しない）。計数が無ければ「—」。符号付きで、正負と 0 の tone を文字の符号と併用する（m の商品は m の形）。 |
| 在庫列 | 写し（`current_stock`）。要再確認なら範囲（「4〜5」）と「要再確認」の badge（icon 付き）。 |
| 最終カウント列（UI-10-D10） | 効く計数の業務日 `count_business_date`（`YYYY-MM-DD`、時刻を付けない）。`actual_count` が `null`（未入力）か業務日が `null` なら「—」（UI-10-D10 の画面の小口の段落）。 |
| 廃番 badge（UI-10-D13） | `item.is_discontinued === true` の行の商品名セルに「廃番」badge（`variant="secondary"` + `border-border`、icon なし）を表示する。非廃番行には表示しない。 |
| 並び順 | 変更しない（`ORDER BY si.id ASC`、UI-10-D3）。 |
| 表示件数 | catalog ⑩ canonical `Pagination`（旧 `ProductPagination`）（`docs/design-system/02-component-catalog.md` §⑩）+ `LIST_PER_PAGE_OPTIONS`（50/100/200、既定 50）の `Select`。変更時は `page` を 1 へ戻す。IO 側 200 クランプ（`PAGINATION_MAX_PER_PAGE`）は不変。配置は表の下（catalog 既定）のまま。 filter row 内の並び順は 部門フィルタ → 未入力のみ表示 → 表示件数（S4a）。 |
| 0 件表示 | `patterns/EmptyState`。「この条件に一致する商品がありません」+ フィルタ解除導線。**filter-empty reset action**（2026-08-03 batch B、[02-component-catalog.md](../design-system/02-component-catalog.md) ⑥）: 部門フィルタ / 未入力のみ toggle のいずれかが既定値以外、かつ結果 0 件のときのみ「絞り込みを解除」ボタンを表示し、押下で部門フィルタ / 未入力のみ toggle / `page` をすべて既定値へ戻す（`page` は `StocktakeSearch` の既存 param、既定 1）。既定値のまま 0 件（棚卸し明細が実在しない）のときは表示しない。 |

## 73.7 確定フロー

1. 確定 CTA「棚卸しを確定する」を押すと常に `confirming_complete` へ遷移する（UI-10-D4）。
2. ダイアログ title「棚卸しの確定」+ warning Alert（`AlertTitle`「確定すると取り消せません」+ 本文「入力した内容で棚卸しを確定します。」と「今日の最後の精算（締め）を取り込みましたか。取り込む前に完了すると、その精算の売上は棚卸しに入りません」）を表示し、「確定する」で `complete_stocktake({ stocktake_id })`、「キャンセル」で `counting` に戻る。
   - アクセシビリティ: Radix `AlertDialog` は `AlertDialogDescription` を `aria-describedby` に紐付けるため、`AlertDialogDescription` は `sr-only` にした上で「確定すると取り消せません。」+ warning Alert 本文を結合したテキストを持たせる（実装レビュー P2 是正、2026-07-08）。
3. `completing` 中は spinner + 全操作 disabled（UI-10-D6）。
4. `completion_blocked` なら、「棚卸しを確定できない理由」の欄に `block_reason` ごとに「数え直しが必要」「まだ数えていない」「取り込んでいない精算がある」の group を出す（冒頭の D-118 の節の「完了」の行。直し方の文、「0 にする」と「残りをまとめて 0 にする」〈確認 dialog、日付を選べる〉）。片付いた理由は一覧から消える。
5. 成功時は `completed_result` へ遷移し、`total_cost`（主役、大きめの表示）、明細の数、「確定では在庫は動きません（数を保存したときに在庫へ入っています）」、`integrity_result`（`None` ならフォールバック文言）、前回完了棚卸しの独立カード（`total_cost` / `completed_at`、UI-10-D5/D10）を表示する。
6. 失敗時は §73.9 のエラー処理に従う。

## 73.8 Command Contract

開始・一覧・確定のコマンド（`src-tauri/src/cmd/stocktake_cmd.rs` / `src-tauri/src/biz/stocktake_service.rs`。計数の 4 command は [42](42-cmd-sales-stocktake.md) の D-118 の節）:

| 呼び出し | 入力 | 戻り | 備考 |
|---|---|---|---|
| 開始 | `start_stocktake()` | `StartStocktakeResult { stocktake_id, item_count }` | 進行中あり → `CmdError.kind = "stocktake_in_progress"` |
| 一覧取得 | `get_stocktake_items({ stocktake_id, department_id?, counted_only?, page, per_page })` | `StocktakeItemListResponse { items: StocktakeItemDetail[], progress: StocktakeProgress, total_count, page, per_page }` | `page < 1` / `per_page < 1` は `kind = "validation"`。`per_page` は IO 側（`PAGINATION_MAX_PER_PAGE`）で 200 件を上限にクランプされる |
| 計数の保存 | `save_stock_count({ request })` ほか 42 の 4 command | `StockCountSaveResult` ほか | 形・範囲・日付は `kind = "validation"`。完了済み棚卸し対象なら `kind = "stocktake_not_in_progress"` |
| 確定 | `complete_stocktake({ stocktake_id })` | `StocktakeResult { total_cost, total_items, integrity_result: IntegrityResult \| null }` | 未計数・要再確認・取り込んでいない精算は `kind = "stocktake_guard"`（`completion_blocked`。40）。完了済みの二重確定は `kind = "stocktake_not_in_progress"` |

`IntegrityResult { mismatches: IntegrityMismatch[], mismatch_count, checked_count, shared_jan_conflicts }`（`biz::integrity_service`）。

これらのコマンドには `#[specta::specta]` が付いていて、`src/lib/bindings.ts` に生成された関数（`startStocktake` / `getStocktakeItems` / `saveStockCount` / `previewStockCountPlacement` / `zeroUncountedItems` / `listRecheckProducts` / `completeStocktake`）がある（UI-10-D7）。

新規コマンド（本書で設計し、実装済み。UI → CMD → BIZ → IO の層原則（CMD は「型変換 + BIZ 呼出し + エラー変換」のみ、[42-cmd-sales-stocktake.md](42-cmd-sales-stocktake.md) §22.5）に従い、BIZ に薄い wrapper を置く。読み取り専用でも BIZ 経由に揃えるのは、`get_stocktake_items` が CMD 直ラップから BIZ wrapper 経由へ修正された経緯（2026-04-13 `882cec6`、[35-biz-stocktake-service.md](35-biz-stocktake-service.md) §20.7）との CMD-10 内一貫性のため）:

| 呼び出し | 入力 | 戻り |
|---|---|---|
| `get_active_stocktake()` | なし | `Option<Stocktake { id: i64, started_at: String, completed_at: Option<String>, status: String, total_cost: Option<i64> }>` |
| `find_stocktake_item({ stocktake_id, code })` | `code`: 商品コードまたは JAN（完全一致） | `Option<StocktakeItemDetail>` |
| `get_last_completed_stocktake()` | なし | `Option<LastStocktakeSummary { stocktake_id: i64, completed_at: String, total_cost: i64 }>` |

- `get_active_stocktake` BIZ 側: `stocktake_service::get_active_stocktake(conn) -> Result<Option<db::stocktake_repo::Stocktake>, BizError>`。`stocktake_repo::find_active_stocktake` を呼ぶだけの薄い wrapper で業務ルールを持たない。
- `get_active_stocktake` IO 側: `stocktake_repo::find_active_stocktake(conn) -> Result<Option<Stocktake>, DbError>`（既存の実装をそのまま流用）。`Stocktake` 型は PR #159 で既に `specta::Type` 付与済み・`biz::mod` から re-export 済み（実コード確認済み）のため、型定義の追加作業は不要。
- `find_stocktake_item` BIZ 側: `stocktake_service::find_stocktake_item(conn, stocktake_id, code) -> Result<Option<StocktakeItemDetail>, BizError>`。`stocktake_repo::find_stocktake_item_by_code` の行に効く計数・理由・差異を足す（35「読取り」）。
- `find_stocktake_item` IO 側: `stocktake_repo::find_stocktake_item_by_code(conn, stocktake_id, code) -> Result<Option<StocktakeItemRow>, DbError>`。SQL は `list_stocktake_items` と同じ SELECT 列に `JOIN products p ON si.product_code = p.product_code` を加え、`WHERE si.stocktake_id = ?1 AND (p.product_code = ?2 OR p.jan_code = ?2) ORDER BY si.id ASC LIMIT 1`。`products.jan_code` / `stocktake_items(stocktake_id, product_code)` の既存インデックスで解決できる。同一 JAN が複数商品に付く場合は `si.id` 最小の 1 件を決定的に返す（既存 `find_by_jan_code` と同じ先勝ちの割り切り + 決定性の担保）。
- `get_last_completed_stocktake` BIZ 側: `stocktake_service::get_last_completed_stocktake(conn) -> Result<Option<LastStocktakeSummary>, BizError>`。`stocktake_repo::find_last_completed_stocktake` を呼ぶだけの薄い wrapper で業務ルールを持たない。
- `get_last_completed_stocktake` IO 側: `stocktake_repo::find_last_completed_stocktake(conn) -> Result<Option<LastStocktakeSummary>, DbError>`。SQL は `SELECT id, completed_at, total_cost FROM stocktakes WHERE status = 'completed' ORDER BY completed_at DESC, id DESC LIMIT 1`。`stocktakes.completed_at` / `total_cost` は `complete_stocktake`（IO）が同一 UPDATE で必ず両方書き込むため、`status='completed'` の行では両カラムとも非 NULL であることが保証される（[db-design/tracking-system-tables.md](../db-design/tracking-system-tables.md) §16-17）。DB スキーマ変更なし。
- CMD 側: いずれも `state.db.lock()` → BIZ 呼出し → `BizError` → `CmdError` 変換のみ（既存 4 コマンドと同型、42-cmd §22.5 の CMD-10 原則）。業務ルールを持たない。

## 73.9 エラー / 回復

| エラー | 回復 |
|---|---|
| `stocktake_in_progress`（開始時、他経路で既に開始済み） | 開始操作を止め、`get_active_stocktake` query を invalidate/再取得して `counting` 表示へ切り替える。エラーメッセージそのものは表示せず、パースもしない（状況が解決した形で見せる。UI-10-D1）。 |
| `stocktake_not_in_progress`（カウント/確定時、完了済み棚卸しに対する操作） | UI 固定文言「この棚卸しは既に完了しています」を表示する（バックエンドのエラーメッセージ文字列をそのまま出さない）。`get_active_stocktake` / `get_stocktake_items` query を invalidate/再取得して `not_started` 表示へ切り替える。数の保存・0 にする・`complete_stocktake` のどの呼び出しで発生した場合も同じ回復を行う。**結果表示への自動切り替えはしない**: 他端末で完了した棚卸しの `total_cost` はこのクライアントの `complete_stocktake` レスポンスとしてしか得られず、技術的に再現できないため（実装レビュー起因、2026-07-08 契約監査）。 |
| `find_stocktake_item` が `None`（棚卸し対象に存在しないコード/JAN） | 検索欄直下に「この商品は棚卸しの対象にありません。商品コードまたはJANを確認してください。新しく登録した商品は自動で追加されます」を表示する。エラー扱いにせず次の入力を受け付ける。 |
| UI 送信前（`actual_count` が空欄または空白のみ） | FieldError「数量を入力してください」。backend には到達しない（wire は `i64` で空欄の概念を持たない）。 |
| UI 送信前（数量の形・範囲、未来の日付） | 発生源直近の FieldError（「0以上の数値を入力してください」、m は「数量は0以上で、小数は2桁までの m で入力してください」、「数が大きすぎます」、「未来の日付には保存できません」。DSR-03）。送信しない。 |
| `stocktake_guard`（`complete_stocktake` の `completion_blocked`） | §73.7 の 4。理由ごとの一覧を出し、一覧 query（`itemsRoot`）を invalidate/再取得する。未知の code は message を上部 Alert に出し、完了として扱わない（40）。 |
| `validation`（`complete_stocktake` の合計金額の範囲の外ほか） | 上部 Alert に BIZ の文（`describeError`）を出し、一覧 query（`itemsRoot`）を invalidate/再取得する。 |
| `integrity_result: null` | 結果画面に「整合性チェックは実行できませんでした」を表示する。棚卸し確定自体は成功として扱う（BIZ 側で確定 TX とは独立して整合性チェックが実行されるため）。 |
| 一覧/状態取得の通信エラー | 上部 Alert（destructive）+ 再試行ボタン。 |

## 73.10 UI / Wording

| 場所 | 文言 |
|---|---|
| ナビ / タイトル / h1 | 棚卸し |
| 未開始時 CTA | 棚卸しを開始する |
| 前回サマリ（未開始時） | 前回の棚卸し（`formatCountedAt(completed_at)`、`T`→スペース変換）: 仕入原価総額 ¥{total_cost}／記録がなければ「前回の記録はありません」 |
| 継続表示ヘッダ | 棚卸し中（開始日: `formatCountedAt(started_at)`、`T`→スペース変換） |
| 進捗表示 | 入力済み {counted}/{total} |
| 未入力のみ toggle | 未入力のみ表示 |
| 在庫列 見出し（UI-10-D10、一覧 + カウント入力欄選択商品情報。値は `current_stock`） | 現在在庫 |
| 差異列 見出し（UI-10-D10） | 差異 |
| 最終カウント列 見出し（UI-10-D10） | 最終カウント |
| 未入力行の最終カウント・効く計数の無い行の差異（UI-10-D10） | — |
| 検索/スキャン欄 | 商品を検索・スキャン |
| 検索/スキャン欄 placeholder（UI-10-D2 契約監査追記） | 商品コード・JAN・商品名を入力 |
| 商品名検索フォールバック 複数候補時の案内（UI-10-D2 契約監査追記） | 候補から商品を選んでください |
| 数量入力欄 | 実際の数 |
| 確定 CTA | 棚卸しを確定する |
| 確定確認ダイアログ title（UI-10-D10、b1 で 1 つに） | 棚卸しの確定 |
| 確定確認ダイアログ warning Alert title（共通、UI-10-D10） | 確定すると取り消せません |
| 確定確認ダイアログ warning Alert 本文（UI-10-D10、b1） | 入力した内容で棚卸しを確定します。／今日の最後の精算（締め）を取り込みましたか。取り込む前に完了すると、その精算の売上は棚卸しに入りません |
| 確定確認ダイアログ sr-only description（UI-10-D10、実装レビュー P2 是正） | 「確定すると取り消せません。」+ warning Alert 本文（`aria-describedby` に不可逆性 title を含める） |
| 対象なし（find_stocktake_item None） | この商品は棚卸しの対象にありません。商品コードまたはJANを確認してください。新しく登録した商品は自動で追加されます |
| 結果画面 見出し | 棚卸し結果 |
| 結果画面 total_cost ラベル | 仕入原価総額 |
| 結果画面 前回比較カード ラベル（UI-10-D10） | 前回の棚卸し（`formatCountedAt(completed_at)`、`T`→スペース変換） |
| 結果画面 前回比較カード 記録なし（UI-10-D10） | 前回の記録はありません |
| 結果画面 明細の数（b1） | 明細の数: {N}件 |
| 結果画面 在庫を動かさない旨（b1） | 確定では在庫は動きません（数を保存したときに在庫へ入っています） |
| 要再確認の一覧 見出し（b1） | 数え直しが必要な商品 |
| 完了の拒否 見出し（b1） | 棚卸しを確定できません。次の理由を片付けてください |
| 整合性チェック失敗フォールバック | 整合性チェックは実行できませんでした |

- 主動線 CTA は 1 個（DSR-01）。counting 画面の primary は「数を保存」のみとし、「棚卸しを確定する」「選択」（候補行）は `variant="outline"` へ降格する（本 PR、UI-10-D2 契約監査追記3）。未開始画面の「棚卸しを開始する」は同一画面に他の primary が無いため引き続き primary。状態は色だけで表さず、「入力済み」「未入力のみ表示」「未入力 N」等の日本語ラベル + アイコンを主情報にする（DSR-08）。未入力 Badge は件数 0 で success tone（`bg-success` + `CheckCircle2`）、1 件以上で warning tone（`border-warning-border bg-warning-soft text-warning-strong` + `AlertTriangle`）に分岐する（本 PR）。
- 確定確認ダイアログは確定 CTA 押下時に**常に**表示する（UI-10-D4/D10。DSR-07 の不可逆操作直前確認 — 確定取消 API が存在しないため確認を省略しない）。`AlertDialogContent` 直下の warning Alert（常時表示）が `AlertTitle`「確定すると取り消せません」で不可逆性を明示する（b1 で未入力有無の分岐を外した。未計数は BIZ の拒否の一覧で示す）。DSR-08（色のみで意味を伝えない）を満たすため、warning の色は `AlertTriangle` アイコン + 太字テキストと併用する。

## 73.11 Query Invalidation

| Event | Query handling |
|---|---|
| `start_stocktake` 成功 | D-052-C15 の SSOT helper を適用。 |
| 数の保存・「0 にする」成功 | D-052-C16（`stocktakeCountUpdate`）の SSOT helper を適用（明細の一覧・写しの consumer・要再確認の一覧。UI_TECH_STACK の D-118 の段落）。 |
| `complete_stocktake` 成功 | D-052-C11 の SSOT helper を適用。 |
| 商品登録・一括インポートによる自動追加 | D-052-C1/C3 が棚卸し consumer を対象に含めるため一覧が再取得される（UI-10-D9、棚卸し画面から専用通知は出さない）。 |
| 商品更新・廃番／復帰成功 | D-052-C2 の SSOT helper により棚卸し一覧の状態を再取得する（棚卸し画面から専用通知は出さない）。 |

成功時の具体的な query key 集合は `src/lib/invalidation-contract.ts` だけに置く。conflict / not-in-progress と validation error の防御 refresh は success-path 契約の対象外で、named helper に隔離する。

## 73.12 テスト設計の起点

RTL（text / role / value assertion、色 class のみの assert は不可）:

- `get_active_stocktake` が `None` → 開始 CTA + 前回サマリ表示、`Some` → 継続表示（開始日時・進捗）。localStorage やエラーメッセージパースに依存せず、DB 問い合わせ（mock の戻り値）のみで状態が決まることを検証する（UI-10-D1）
- 開始 CTA 押下で `start_stocktake` が呼ばれ、成功後にカウント画面へ切り替わる
- 部門フィルタ変更・未入力のみ toggle で一覧取得パラメータが変わる（UI-10-D3）
- 検索/スキャン入力で `find_stocktake_item` が呼ばれ、解決成功で数量入力へフォーカス移動 → `save_stock_count` が該当の商品と `stocktake_item_id` で 1 回呼ばれる（UI-10-D2）
- 検索欄・数量入力欄とも、IME 変換確定の Enter（`isComposing: true`）では `find_stocktake_item` / `save_stock_count` が呼ばれない（UI-10-D2 契約監査追記2、Codex レビュー P2 是正）
- 画面表示時は検索/スキャン欄に自動フォーカスし、数の保存の成功後は検索/スキャン欄へフォーカスが戻る（連続 HID スキャン運用の検証、UI-10-D11、実装漏れ是正）
- `find_stocktake_item` が `None` → `searchProducts` で商品名検索にフォールバックする。0 件は「棚卸しの対象にありません」文言が表示され `save_stock_count` は呼ばれない、1 件は自動的にその `product_code` で再解決して選択（数量欄へフォーカス移動）、複数件は候補テーブルが表示され選択操作で解決する（UI-10-D2 契約監査追記）
- counted 済み商品を再度指定 → 入力 → 確認ダイアログなしで `save_stock_count` が呼ばれる（UI-10-D2）
- 進捗表示が `progress.counted_items` / `total_items` の値と一致し、`Progress` の value が同じ比率で表示される（UI-10-D10）
- 一覧の在庫列は `current_stock` を表示し（`system_stock` ではない）、差異列は BIZ の `difference` を出す（画面で計算しない）。未入力行の最終カウントの列は「—」（前の計数を持ち業務日のある未入力行も「—」）。差異の列は BIZ の `difference` で、効く計数が無ければ「—」になる（BIZ は業務日と差異を同じ効く計数から作るので、前の計数を持つ未入力行の差異は数。35「読取り」）。最終カウントは保存の時刻でなく効く計数の業務日を出す（画面の小口の lane）。`system_stock !== current_stock` のケースで在庫列と差異列の数値が矛盾しないことを検証する（UI-10-D10、実装レビュー P2 是正）
- 確定 CTA → 常に title「棚卸しの確定」+ warning Alert（最後の精算の問いを含む）のダイアログを経て `complete_stocktake({ stocktake_id })` が呼ばれる（UI-10-D4/D10、b1 の U7）
- 確定結果画面が `total_cost`・明細の数・在庫を動かさない旨を表示し、補正の一覧を出さない。前回比較が総額カードとは別の独立した要素として表示される（UI-10-D5/D10、b1 の U8）
- b1 の計数・要再確認・完了の拒否・「今の数を入れる」の振舞いは Matrix の U1〜U9・U15・U16（`StocktakePage.test.tsx`）
- `integrity_result` が `null` のときフォールバック文言が表示される
- `stocktake_in_progress` エラーで `get_active_stocktake` が invalidate/再取得され継続表示へ切り替わる（エラーメッセージのパースをしないことを assert する）、`stocktake_not_in_progress` エラーで UI 固定文言「この棚卸しは既に完了しています」が表示され状態再取得後の `not_started` 表示に切り替わる（UI-10-D8。数の保存の経路・`complete_stocktake` 経路の両方で検証、契約監査により追加）
- `complete_stocktake` の `stocktake_guard`（`completion_blocked`）で理由ごとの一覧が出て、一覧 query（`itemsRoot`）が invalidate/再取得される（b1 の U7）
- 数量に負数を入力 → FieldError が出て `save_stock_count` が呼ばれない
- 棚卸し中に商品登録/インポートが行われた想定で invalidation 後に一覧の対象件数が増える。専用通知は出ない（UI-10-D9）
- `complete_stocktake` 実行中は spinner 表示 + 全操作 disabled（UI-10-D6）

## 73.13 Windows Native L3

| # | 画面 / 見る場所 | 到達手順 | 合格基準 |
|---|---|---|---|
| L3-1 | `/stocktake`、継続表示ヘッダ | 棚卸しを開始する | ヘッダに「棚卸し中（開始日: YYYY-MM-DD HH:MM:SS）」の形式で**日時**が表示される（内部 ID の数字ではない）。進捗バーが表示される。アプリ再起動後も継続表示に戻る |
| L3-2 | `/stocktake`、検索/スキャン欄・数量入力欄 | 検索欄をクリックしてカーソルを置く → 商品コード（またはバーコードスキャナで JAN）を入力し Enter → 数量を入力し Enter、を**マウスに触らず**3商品分連続で行う | 1商品目: コード確定後、自動的にカーソルが「実際の数」欄へ移る。数量確定後、自動的にカーソルが検索欄へ戻る。この一連が3商品ともマウスクリックなしで完了する |
| L3-3 | `/stocktake`、一覧 | counted 済み（差異列に数値が入っている）で最終カウントが今日より前の日付の行（今日数えた行を今日入れ直しても日付は変わらず、更新を目で確かめられない）のコードを再度検索・スキャンし、「入れ直す」で別の数量を保存 | 確認ダイアログなしで保存でき、一覧の実カウント・差異・最終カウント（保存した日付）が更新される |
| L3-4 | `/stocktake`、確定ダイアログ・結果画面 | 「棚卸しを確定する」を押す | ダイアログ title が「棚卸しの確定」、直下に黄色い警告ボックス（⚠️ アイコン + 太字「確定すると取り消せません」）と最後の精算の問いが表示される。未計数があれば理由ごとの一覧が出て、片付けてから確定すると、結果画面の「前回の棚卸し」カードの金額が、画面上部の「仕入原価総額」（今回確定した金額）と**異なる値**になっている |
| L3-6 | `/stocktake`、一覧 | カウント中に別画面（商品登録）で新商品を1件登録し、棚卸し画面へ戻る | 一覧を開き直すと新規商品の明細が表示される（専用の通知は出ない） |
| L3-7 | `/stocktake`、検索/スキャン欄 | L3-6 で登録した新商品の**商品名**（コード/JANではなく）を検索欄に入力して「対象を確認」を押す | 「対象にありません」エラーにならず、1件だけ一致すれば自動的にその商品が選択され数量入力欄にフォーカスが移る。複数件一致する名前で試すと候補テーブル（商品コード/商品名/部門 + 選択ボタン）が表示され、選択すると対象が確定する |

実機で再現しにくいエラー経路（`stocktake_not_in_progress`）は、L3 では確認しない（自動テスト T13/T18）。b1 の計数・要再確認・完了の拒否・「今の数を入れる」の L3 は b1 の packet の L3 の台本（L3-1〜L3-13）で行う。

実店舗データを含む証跡は repo に残さない。必要な差分確認は synthetic / test DB で行う。

## 73.14 Non-scope

- 棚卸しの中止（途中破棄）機能（UI-10-D1、35-biz §20.7 の非目的踏襲）。
- 明示的な resume（再開）専用コマンド・UI（UI-10-D1）。
- BIZ-06 確定ロジックの変更（`counted_at` 以降の在庫変動の自動織り込み等、UI-10-D2）。
- 一覧のソート追加（UI-10-D3）。
- 棚卸し履歴一覧画面（複数回分の比較。UI-10-D5、前回 1 回分の比較のみ実装）。
  - 2026-08-27 owner 裁定により `/inventory/records` への横断合流（65 doc slice 4d）で代替充足し、専用一覧 `/stocktake/records` と `listStocktakeRecords` は runway 残置とする。
- IPC channel によるカウント/確定処理の進捗表示（UI-10-D6、10-4a は不採用で確定）。
- UI-11c / UI-13 の設計。
- ルーティング実装、コンポーネント実装、hooks 実装、specta 属性付与、テストコード（本書は Design Phase のみ。実装 PR (R3) の scope）。

### 更新履歴

| 日付 | PR | 内容 |
|------|-----|------|
| 2026-09-02 | 本 PR | UI ガッツリ整えターン disposition culling（棚卸し UX audit C1/C4/C8/C9/C10 + PR #27 L3 owner 観察 3 件）を消化。UI-10-D2 契約監査追記3（FieldError 残留是正・「対象にありません」情報表示化・廃番商品の名前検索包含）+ §73.6 表示件数行（catalog ⑩ 再利用）+ §73.10 主動線 CTA / 未入力 Badge tone 記述更新。C10（単位併記）は起票時実測で `StocktakeItemDetail` に単位 field が無いと確定し見送り。 |
| 2026-09-02 | 本 PR | Gated Amendment 1（owner Windows native L3 run 1、PR #30 comment 5509800525）。S3 に整列是正（ボタン上端を入力欄上端と揃える）を追加、S10 として一覧 filter row 順序を 部門→表示件数→未入力のみ表示 へ変更。 |
| 2026-07-07 | - | UI-10 Design Phase 初版（UI-10-D1〜D9、`docs/archive/plans/2026-07-07-ui10-stocktake-design.md` 準拠） |
| 2026-07-08 | #159 (private archive) レビュー起因 | 新規 CMD `get_active_stocktake()` を追加（UI-10-D1 更新）。実装 PR #159 で進行中判定が `localStorage` + `start_stocktake` のエラーメッセージ文字列パースで代替されていたのを P1 として却下し、DB 問い合わせ CMD による正式な進行中判定に差し替え。specta 化本数を既存4+新規2=6本 → 既存4+新規3=7本に更新。 |
| 2026-07-08 | #159 (private archive) 実装レビュー起因 | UI-10-D10 追加（一覧の差異・最終カウント列、確定ダイアログ title 統一、前回比較独立カード）。同日の Codex レビュー P2 是正で、一覧・カウント入力欄の在庫列を `system_stock` から `current_stock` へ統一（列名「システム在庫」→「現在在庫」）し、差異の計算根拠と表示在庫値の不一致を解消。 |
| 2026-07-08 | #159 (private archive) owner フィードバック起因 | UI-10-D10 の確定ダイアログを warning Alert ネスト構造に再変更（`RestoreConfirmDialog` 前例なしを理由に不採用としていた判断を訂正）。同日の Codex レビュー P2 是正で、`AlertDialogDescription`（sr-only）に不可逆性 title「確定すると取り消せません」を含めるよう修正（`aria-describedby` から警告 title が欠落していた）。 |
| 2026-07-08 | #159 (private archive) owner 確認質問起因 | UI-10-D11 追加。§73.5 が定めていた「解決できたら数量入力欄にフォーカスを移す」契約が実装に反映されておらず、初期フォーカス・保存後のフォーカス復帰も未実装だった実装漏れを是正（`ReceivingPage.tsx` の `useRef` + `window.setTimeout` パターンを踏襲）。連続 HID スキャン運用が成立しない状態だった。 |
| 2026-07-08 | #159 (private archive) Codex レビュー起因 | UI-10-D11 是正: mount 時の初期フォーカスは `useEffect` で直接 `.focus()`（`window.setTimeout` 不要）、解決成功後・保存成功後は `window.setTimeout` パターン、と実装の使い分けに合わせて記述を明確化（P3）。T17 の検索欄操作を `user.click`（対象を確認ボタン）から `{Enter}` 経由に変更し、実際の HID スキャン経路を検証するよう修正（P2）。 |
| 2026-07-08 | #159 (private archive) 契約監査（Fable 実施） | UI-10-D1 是正: 継続表示ヘッダが「開始日時」ではなく内部 `stocktake_id` を表示していた実装漏れを是正。UI-10-D8 是正: `stocktake_not_in_progress` が `update_count` / `complete_stocktake` 経路で kind 判定・invalidate 未実装のまま汎用エラー表示に留まっていた欠落を是正、UI 固定文言をハードコード（従来はバックエンドのエラーメッセージ文字列をそのまま表示）。§73.9 の `validation`（force_fill 未入力超過）専用再試行導線は、`kind` が `actual_count` 負数と共有で判別不能なため実装しない判断を明記（UI-10-D1 で却下した message パースのアンチパターンに該当するため）。T18 追加。 |
| 2026-07-08 | #159 (private archive) 契約監査（Codex 実施） | UI-10-D5 是正: `complete_stocktake` 成功後の `lastCompleted` invalidate/再取得により、結果画面の「前回の棚卸し」が今確定した棚卸し自身に置き換わるバグを是正（`handleCompleteConfirm` で invalidate 前に local state スナップショットを取り、結果画面にはそのスナップショットを渡す）。§73.12 の確定ダイアログ title 記述（旧「棚卸しを確定します（取り消せません）」表記）を UI-10-D10 の現行構造に同期。テスト冒頭コメントを T1〜T19 に更新。T19 追加。 |
| 2026-07-08 | #159 (private archive) owner 指摘起因 | UI-10-D8 追記2: force_fill 未入力超過の到達条件を再検討し「一人・一台でも、確定直前の商品登録 + 画面再訪後の自動再取得完了前の確定操作、という限定的なタイミングでは到達しうる」と明記。専用再試行ボタンは作らないまま、`handleCompleteConfirm` の `validation` エラー catch に `itemsRoot` invalidate を追加し、次回の確定操作が最新状態に基づいて正しく動くようにした（実質的な再試行導線）。§73.9 の validation 行を更新。T20 追加。 |
| 2026-07-08 | #159 (private archive) L3 チェックリスト刷新 | §73.13 を「画面/到達手順/観測可能な合格基準」形式に全面更新（memory `feedback-l3-checklist-eye-observable-absolute` 準拠）。開始日時表示、連続 HID スキャンのフォーカス遷移、確定ダイアログの title 分岐（未入力あり/なし）、前回比較の差し替わり確認を新規に L3 項目化。実機で再現しにくいエラー経路（`stocktake_not_in_progress`、force_fill validation）は L3 対象外とし自動テスト（T13/T18/T20）で担保する旨を明記。L3-4/5/6 追加（旧 5 項目 → 6 項目）。 |
| 2026-07-08 | #159 (private archive) owner L3 実機観察 | 開始前画面の「前回サマリ」と結果画面の「前回比較カード」に `completed_at` を `formatCountedAt` 未適用のまま生の ISO 8601（`T` 区切り）で表示していた見落としを是正。継続表示ヘッダ（UI-10-D1 是正で新設）には適用済みだったが、既存の前回比較表示 2 箇所は未適用のままだった。73.10 Wording の該当 2 行 + 継続表示ヘッダ行を `formatCountedAt` 適用済みである旨に更新。 |
| 2026-07-08 | #159 (private archive) owner L3 実機発見 | UI-10-D2 是正: §73.5 が Design Phase 初版から明記していた「商品名検索フォールバック（`commands.searchProducts`）」が実装から丸ごと漏れていた。owner が実機で「棚卸し中に新商品登録→その商品名で棚卸し検索」という自然な操作をした際、一覧には存在するのに検索では「対象にありません」と表示される矛盾した体験になっていた。`resolveItem` を拡張し、`find_stocktake_item` が `None` の場合に商品名検索へフォールバック（0件は既存メッセージ、1件は自動選択、複数件は候補テーブル表示）する形で是正（`ReceivingPage.tsx` の商品名検索パターンを踏襲）。T21/T22 追加。§73.13 L3-7 追加（商品名検索フォールバックの実機確認）。 |
| 2026-07-08 | #159 (private archive) Codex レビュー起因 | UI-10-D2 契約監査追記2: 商品名検索欄追加時に、`ReceivingPage.tsx` の商品検索欄に既にある `event.nativeEvent.isComposing` guard を移植し忘れており、日本語 IME の変換確定 Enter でも検索が発火していた。検索欄・数量入力欄の両方の `onKeyDown` に guard を追加。T23 追加。 |
| 2026-08-03 | ui-polish-batch-b（本 PR） | §73.6 一覧の 0 件表示行に filter-empty reset action（catalog ⑥、部門フィルタ + 未入力のみ表示が対象）を追記。 |
| 2026-08-03 | ui-polish-batch-b round 2 是正（本 PR） | round 2 P2-1 対応: §73.6 filter-empty reset action の戻す対象に `page`（`StocktakeSearch` 既存 param、既定 1）を追加し、部門フィルタ / 未入力のみ toggle / `page` の 3 者を既定値へ戻す契約に更新。 |
| 2026-08-30 | docs 整合性衛生 batch（本 PR） | §73.1 に棚卸しカウント対象の母集団（issue #91 owner 回答 2026-08-22）を明記。 |
| 2026-09-02 | stocktake-empty-count-guard（本 PR） | §73.5 step 4 と §73.9 に、数量の空欄・空白のみを `update_count` 送信前の FieldError「数量を入力してください」で止める契約（ST-C5-D1）を追加。 |
| 2026-09-06 | ⑨ 在庫少一覧の取引先列 + 棚卸し廃番 badge + R2-3（本 PR） | UI-10-D13 追加: `StocktakeItemDetail` に `is_discontinued: bool` を追加し、棚卸しカウント画面の一覧テーブルに廃番 badge（`variant="secondary"` + `border-border`、icon なし）を表示する（§73.3/§73.6）。同一 file 内の既存候補行 badge も同じ commit で `border-border` へ揃える。 |
| 2026-09-24 | ㉘ runtime ①（本 PR） | §73.15 現行buildの停止の節（SPEC-STOP-D4、案内の見出し・本文の正本、無効にする操作と使える操作）を追加。 |
| 2026-10-23 | stocktake-count-runtime-b1（D-118 の b1） | 冒頭の D-118 の節を現行に。§73.15 の停止の節を外し、F4・F5・D4・§73.4〜§73.13 の数の保存・確定・差異・結果の画面を計数の保存・完了の拒否の一覧・在庫を動かさない確定に置き換えた。L3 の (e) を自動 test へ移した（[Plan Packet](../archive/plans/2026-10-22-stocktake-count-runtime-b1.md)） |
| 2026-10-24 | stocktake-count-runtime-b1 の Final Review の是正 | §73.1 と UI-10-D7 の現在形の説明を CMD-10 の 11 command（`save_stock_count` ほか）に揃えた（廃止した `update_count` と「7 本」が残っていた）。 |
| 2026-10-27 | 画面の小口（導入の前の関門 6、起票） | 一覧の最終カウント列を効く計数の業務日にする決定（UI-10-D10 の段落、§73.6）と、route の `product` が外れたときの「今の数を入れる」の欄の閉じ方と、閉じる時点で外れていれば保存の時の戻り先へ戻らないこと（冒頭の D-118 の節。Plan Review round 3 の是正で、棚卸しの画面を離れた後に届いた保存の成功では何もしないことを足した）を置いた（proposed・未実装。[Plan Packet](../plans/2026-10-27-stocktake-screen-fixes.md)） |
| 2026-10-27 | 画面の小口の実装（PR #210） | UI-10-D10 の段落・§73.6 の最終カウント列と、冒頭の D-118 の節の「今の数を入れる」の行の route の `product` が外れたときの閉じ方を、実装済みの記述にした（[Plan Packet](../plans/2026-10-27-stocktake-screen-fixes.md)） |
