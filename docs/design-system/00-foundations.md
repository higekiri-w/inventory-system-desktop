# デザインの土台（foundations）

> **親文書**: [README.md](README.md)
> **責務**: 色の役割・強調の段階・迷いやすい場面・ラベルと値・書体と、カラーパレット・セマンティックトークン・タイポグラフィ・スペーシング・アイコンサイズの正典。UI 実装で使うすべてのデザイントークンと、それをどの役割・どの段で使うかはここを一次参照とする。SegmentedControl 仕様は [02-component-catalog.md](02-component-catalog.md) ⑤ が正典。
> **規則の書き方**: 規則は「見せ方 → 見る人の受け取り方 → 狙う効果 → 根拠」の 4 列で書く。上位の原則は [04-backbone.md](04-backbone.md)、根拠の出典は [03-philosophy.md](03-philosophy.md)。
> **現行と狙い**: 本書の役割と段は規則である。色と強調は token・部品・画面へ反映済みで（2026-09-28、decision-log D-094）、色の役割表の「現行の実装」「移行」列と「迷いやすい場面」表に実装の component 名と token 名を記す。試しを経た行の採否と恒久の例外は D-094 に置く。移行が残るのは書体（本書「書体」）だけで、移行中に画面を作るときの決まりは [README.md「移行中の作り方」](README.md#移行中の作り方)。

---

## 色の役割

色は役割で使う。1 つの色は 1 つの役割だけを持ち、役割を持たない色を足さない（04 原則 2）。役割は要素（領域・行・badge・ボタン・部品の状態）ごとに 1 つ選ぶ。行の役割と、その行の中の badge の役割は別の要素として選ぶ（例: 入力中の行 = 進行中、その行の在庫切れの badge = 危険・失敗）。

| 役割 | 使う場面 | 見せ方 | 受け取り方 | 狙う効果 | 根拠 | 現行の実装 | 移行 |
|---|---|---|---|---|---|---|---|
| 操作 | 押すボタン・リンク・ナビの現在地・フォーム部品の checked と focus ring | 塗りは押すものだけ（1 画面の主要ボタンは 1 つ、DSR-01）。ナビの現在地は細いバー、checked は小さな塗り、focus ring は線。リンクは文字色と hover の下線 | 「ここを押せばよい」「いまここにいる」と迷わず分かる | 次の一手を選ぶ時間が短くなる | Hick の法則（選ぶものが少ないほど早く決まる）、Von Restorff 効果（1 つだけ違うものが目に留まる） | `--primary`（D-094）、`--ring`（`--primary` と同じ値）、`checkbox` の checked とネイティブの radio・checkbox の `accent-primary`、ナビの現在地の `CURRENT_LOCATION_ACCENT` | 済 |
| 進行中 | 利用者の作業が続いている領域と行: 数えている・取り込んでいる領域、入力中の行（現在行、DSR-22）、待ちの spinner、作業の進み具合の棒、詳細を開いた行（迷いやすい場面） | 操作と同じ色の仲間の薄い地で表す。現在行は左端のバー + 薄い地 + 文言、作業中の囲みは 2px 枠（1 画面 1 か所）。部品自身の busy（保存中のボタン）は元の役割のまま spinner と文言で示す。お知らせ一般に使わない | 「自分がいま手がけていて、まだ終わっていない」と分かる | 作業を途中で置き忘れず、戻る場所が分かる | Zeigarnik 効果（終わっていない作業は記憶に残る）、Von Restorff 効果 | `--ongoing` の家族（`--ongoing`・`--ongoing-soft`・`--ongoing-border`・`--ongoing-strong`）。現在行と詳細を開いた行は `border-l-ongoing` と `bg-ongoing-soft`、待ちの spinner は `text-ongoing`、作業の進み具合の棒は `progress` の `bg-ongoing`、取込みの手順のいまのステップは進行中の線・地・文字 | 済（作業中の囲み〈段 3〉と取込み中の領域の地〈段 1〉は、使う画面を作るときにこの token で作る） |
| 注意・確認 | 手を止めて確かめてほしいもの: 在庫少・数え直し・未入力・同日データあり（同日追加確認の Alert と badge、DSR-03）・未反映・部分成功、日次と月次の未取込みの知らせ、③強調の琥珀 pill（最新・上書き件数・ランキング 1 位、owner の現行の決定） | 薄い地 + 同じ役割の線 + icon + 文言。線の濃さは面積で決める（強調の段階 2）。文字だけの形は迷いやすい場面の行に従う | 怖くはないが、一度手を止めて確かめる | 見落としが減り、確かめた上で進める | Von Restorff 効果、WCAG 1.4.1（色だけに頼らず文字と icon を添える） | `--warning` の家族（soft・border・strong・emphasis）。ランキング 1 位の pill と行の地は `--rank-top-*` | 済（③強調の琥珀 pill のうち最新・上書き件数の塗りは、押すボタンでない badge の塗りの恒久の例外、D-094） |
| 完了 | 済んだことのプラスの報告: 保存した・取込みが成功した・反映済み・補正済み・数え終わり | 薄い地 + 1px の線 + icon + 文言まで（段 2 まで、塗りにしない）。操作の塗りとは文字と icon で区別する | 「終わった、うまくいった」と安心する | 作業の区切りがはっきりし、終わりの印象がよくなる | Peak-End の法則（体験の評価は山場と終わり方で決まる） | `--success` の家族（soft・border・strong） | 済 |
| 危険・失敗 | 二つの場面を持つ。危険・戻せない = 戻せない操作の確認（dialog の実行ボタン、DSR-20）。失敗・欠け = その行・画面の値が成り立たないもの（取得失敗・在庫切れ・取消済み・マイナスの増減）と、進めない（日報の二重取込みの Alert と取込み済みの badge・取得失敗。DSR-03 のデータ安全系のうち止めるもの。badge は L3 round 1 で試しを採った、D-094）、Home の前日分の未取込みの Alert（owner 決定 2026-09-06）。確認すれば進める同日追加確認は注意・確認 | 最も強い色。塗りは dialog の実行ボタンだけで、ほかは薄い地 + 線 + icon + 文言。文字だけの形は迷いやすい場面の行に従う | 止まる。戻せない・成り立っていないと分かる | 取り返しのつかない操作と欠けたデータを見逃さない | Von Restorff 効果（最も強い色を危険と失敗だけに残す）、WCAG 1.4.1（文字と icon を添える）、DSR-20（戻せない操作の確認 dialog）、DSR-03（Toast と Alert の使い分け、データ安全系の知らせ） | `--destructive` の家族（soft・border・strong）。Alert の `destructive` は薄い地 + 線 + 部品が描く三角 icon | 済 |
| ふつう・補足 | 説明・件数・日付・ランキング・分類の badge・選択状態（DSR-21）・手を止めなくてよいお知らせ一般（Alert の既定）・比率の棒・操作枠 | stone の地と線、ふつうの文字か muted の文字。役割色を使わない | 読むだけでよい、手を止めなくてよい | 役割色が付いたものが際立ち、画面が落ち着いて読める | Von Restorff 効果（ふつうのものに色を使わないことで役割色が際立つ）、美的ユーザビリティ効果（色の少ない画面は落ち着いて読める）、DSR-21（選択状態は stone） | stone のベースパレット（`--background`・`--card`・`--border`・`--border-strong`・`--muted-foreground`） | 済（ランキング 1 位は③強調の琥珀 pill、迷いやすい場面） |

- どの状態も文字か記号を添え、色だけに頼らない。badge と Alert の icon は DSR-08 と 02 ⑬ ⑥ に従う（WCAG 1.4.1、本書「業務ステータスの視認性」）。
- 進行中は info（お知らせ）ではない。手を止めて確かめてほしい知らせは注意・確認、手を止めなくてよいお知らせ一般はふつう・補足（Alert の既定）で示す。
- 「取込み済み」は二重取込みを防ぐ知らせで、完了に入れない（Alert と badge は危険・失敗。迷いやすい場面）。
- 今の赤・琥珀・緑の割り当て（在庫切れ・取消済み・マイナスの増減・取得失敗の赤、[02-component-catalog.md](02-component-catalog.md) ⑬ の tone family 表の注意と完了、owner 決定 2026-09-06）は変えない。日報の取込み済みの badge だけは、owner が色と強調の L3 round 1 で注意・確認から危険・失敗へ移した（D-094）。
- 有彩色の系統は役割の数を上限にする。進行中の色相は操作と同じ仲間で、amber・green・red の 3 系統に 1 つ加わるだけである（04 原則 2）。
- 色の値は本書のカラーパレット表とセマンティックカラー表に登録してある（2026-09-28）。候補の値と、そこから計算したコントラスト比は [decision-log](../decision-log.md) の D-091 と D-094 に置く。

---

## 強調の段階

目立つものが多いと何も目立たない。強調は段で決め、上の段ほど 1 画面に置ける数を減らす（04 原則 3）。

| 段 | 見せ方 | 使う場面 | 1 画面に置ける数 | 受け取り方 | 狙う効果 | 根拠 |
|---|---|---|---|---|---|---|
| 段 0 | そのまま。地と線は stone、文字はふつうか muted | ただのまとまり（card・区画・Alert の既定）、②分類の badge（恒常的な属性で状態を知らせない。stone の地と `--border` の枠、02 ⑬）、減衰のうち廃番の行と空状態（muted で弱め、役割色を足さない） | 上限なし | 読むだけでよい | 役割のある段が際立つ | Von Restorff 効果、美的ユーザビリティ効果 |
| 段 1 | 役割の薄い地だけ。線を引かない | その役割の領域だけ（例: 取込み中の領域の進行中の地） | 役割の領域ごとに 1 つ | 「この範囲はその役割の場所」と分かる | 囲みを増やさずに領域の意味が伝わる（DSR-16） | Von Restorff 効果、美的ユーザビリティ効果 |
| 段 2 | 薄い地 + 同じ役割の 1px 線、または細いバー。線の濃さは面積で決める（badge は `-border` の色、Alert と行は役割の base の色） | 状態を知らせる行・badge・Alert、Home の入口 card（最重要の入口 1 つ）、現在地と現在行の左端のバー（DSR-21 / DSR-22）。完了は段 2 まで。③強調の琥珀 pill（badge の塗り）は段 2 に数える（恒久の例外、D-094） | 状態の数だけ。バッジの文は短く（目安 8 文字）し、補足は下に小さく分ける | 状態が一目で分かり、読むと理由が分かる | 状態の見落としが減る | Von Restorff 効果、美的ユーザビリティ効果、WCAG 1.4.1 |
| 段 3 | 役割色の 2px 枠。黒・文字色で引かない | いま操作している 1 つだけ（作業中の囲み、DSR-22） | 1 画面に 1 か所 | 「いまここを扱っている」と迷わない | 目の置き場所が 1 つに決まる | Von Restorff 効果、美的ユーザビリティ効果（強すぎる枠と長いバッジは煩わしい） |
| 段 4 | 塗り。押すボタンだけ | 1 画面の主要ボタン 1 つ（DSR-01、操作の塗りだけを数える）。危険の塗りは dialog の実行ボタンだけ（DSR-20）で、画面上の確定・削除のトリガーは操作の役割 | 操作の塗りは 1 画面に 1 つ | 「これを押せば進む」と分かる | 主動線が 1 つに決まる | Von Restorff 効果、Hick の法則 |

**段に数えない**もの（1 画面に置ける数に入れない。役割は 1 つ持つ）:

- focus ring（`--ring`、操作）
- 操作枠（`--border-strong`、ふつう・補足。隣接背景に対し 3:1、DSR-22）
- checkbox / radio の checked の小さな塗り（操作）
- 進み具合の棒の塗り（作業の進みは進行中、比率はふつう・補足）
- 待ちの spinner（進行中）
- 役割色の文字だけの表示（入力欄のエラー文・増減の ±・セルの数字・件数・リンク。役割は迷いやすい場面の行に従い、上限は置かない。記号・文言・下線のどれかを必ず添える）

押せないボタンは元の役割・元の段のまま opacity で弱める（主要ボタンが押せないときも段 4 の 1 つに数える）。badge の塗りは③強調の琥珀 pill（owner の現行の決定）だけに残し、段 2 に数える（押すボタンでない塗りの恒久の例外、D-094）。

DSR-16 との関係: 段 1 は線を引かずその役割の領域だけに使い、ただのまとまりは段 0 の stone で示す。段 2 の線は状態を知らせる行・badge・Alert と Home の入口 card に限る。薄い線を単独のグループ信号にしない（DSR-16）。

---

## 迷いやすい場面

色の役割表と強調の段階表だけでは答えが 0 か 2 つ以上になる場面の答え。各行で役割と段は 1 つに決まる（行の中で条件ごとに 1 つを書く場合を含む）。条件は要素・画面で分ける。部品の一時的な状態（busy・disabled・hover・focus）で役割を切り替えない。業務の状態が変わる要素（在庫状態の badge の在庫切れ / 在庫少 / 在庫あり、記録状態の badge、0 件と 1 件以上で色が変わる件数）は状態ごとに色の役割表で役割を選ぶ（02 ⑬ の tone family）。現行の実装の列は 2026-09-24 の実装を component 名と token 名で書く。

試しを経た 7 行と説明文の muted の文字色の採否は、owner の L3 round 1（2026-09-28）で決まり、decision-log D-094 に記録してある。採ったのは詳細を開いた行・取込みの手順の表示・Home の入口 card・日報の取込み済みの badge、採らなかったのはランキング 1 位（順位と太字）・最新と上書き件数（stone の pill と太字）・Home の前日分の未取込み（注意・確認）で、muted の文字色は M2。恒久の例外は 1 点: ③強調の琥珀 pill のうち最新・上書き件数の塗り（押すボタンでない badge の塗りで、段 2 に数える）。

| 場面 | 役割 | 段 | 現行の実装 | 移行 |
|---|---|---|---|---|
| focus ring | 操作 | 段に数えない（部品の状態） | `--ring` は `--primary` と同じ値。枠を持つ部品は focus で 1px の枠が `border-ring` になり透過の ring を添える。枠の無い部品（button・accordion・scroll-area）と、focus の前後で枠の色が変わらない部品（checked の checkbox・選んだ選択カード・押された toggle）は不透明な `focus-visible:ring-ring`（DSR-22 の 3:1、UI_TECH_STACK §5.4、D-094） | 済 |
| 操作枠（入力欄・outline ボタンの枠） | ふつう・補足 | 段に数えない（DSR-22 の 3:1） | `--border-strong` | 済 |
| 保存中のボタン（部品自身の busy） | 元の役割のまま（保存ボタンなら操作） | 元の段のまま。spinner と「保存中...」の文言を添える（02 ⑥ Spinner） | 入庫・手動販売ほかの保存ボタン、棚卸しの確定ボタン | 済 |
| 待ちの spinner（取込み中・照合中の大きな spinner） | 進行中 | 段に数えない | `Loader2` の `text-ongoing`（32px） | 済 |
| 進み具合の棒 | 作業の進み（棚卸し・整合性チェック）は進行中、比率（月次の部門比率）はふつう・補足 | 段に数えない | `progress` の棒は既定で `bg-ongoing`。部門比率は `indicatorClassName="bg-muted-foreground"` | 済 |
| checkbox・radio の checked | 操作 | 段に数えない（小さな塗り） | `checkbox` の checked は `bg-primary`、ネイティブの radio・checkbox は `accent-primary` | 済 |
| 複数選択の行（上書きする行を checkbox で選ぶ） | ふつう・補足（選択状態、DSR-21） | 段 0（行に色を付けず、部品の checked で示す。現在行に含めない） | 商品取込みの上書き選択列 | 済 |
| Home の入口 card の強調（最重要の入口 1 つ） | 操作（mockup-c、owner 採用 2026-09-11） | 段 2（操作の 1px 線 + 操作の仲間の薄い地。地は進行中の地 `--ongoing-soft` を流用する、D-094） | `ActionButton` の primary は `border-primary bg-ongoing-soft`、icon は `text-primary` | 済（試しを採った、D-094） |
| 画面上の確定・削除のトリガー | 操作 | 主要ボタンなら段 4（1 画面 1 つ）、それ以外は DSR-01 の降格 | 危険の塗りのボタンはすべて dialog の中 | 済 |
| dialog の実行ボタン（戻せない操作） | 危険・失敗 | 段 4（DSR-20） | `variant="destructive"` | 済 |
| 日報の取込み済み（二重取込みの防止） | Alert と badge は危険・失敗（Alert は owner 決定 2026-09-06、badge は L3 round 1 で試しを採った、D-094）。同日追加確認の badge は注意・確認 | 段 2 | Alert は `destructive`、取込み済みの badge は危険の tone（`tone="destructive"`）、同日追加確認の badge は注意の tone | 済（試しを採った、D-094） |
| 未取込みの知らせ（日次・月次・Home の前日分） | 日次・月次は注意・確認、Home の前日分は危険・失敗（owner 決定 2026-09-06） | 段 2 | 日次・月次は注意の Alert、Home は `destructive` の Alert | 済（Home の試しは採らなかった、D-094） |
| 危険・失敗の Alert（取得失敗・二重取込み・Home の前日分） | 危険・失敗（どの知らせがこの役割かは色の役割表と各場面の行に従う） | 段 2（薄い地 + 役割の base の線 + icon + 文言） | `Alert variant="destructive"` は `bg-destructive-soft border-destructive text-destructive-strong` で、三角 icon（`TriangleAlert`、`aria-hidden`）を部品が最初の子に描く。site は icon を書かない | 済（D-094。役割は変えない） |
| 減衰（押せないボタン・廃番の行・空状態） | 押せないボタンは元の役割のまま、廃番の行と空状態はふつう・補足（役割色を足さない） | 押せないボタンは元の段のまま opacity で弱める（主要ボタンなら段 4 の 1 つに数える）、廃番の行と空状態は段 0 の muted | `disabled:opacity-50`、`EmptyState` の stone | 済 |
| 注意の Alert と注意の badge の線 | 注意・確認 | 段 2（線の濃さは面積で決める: badge は `-border` の色、Alert と行は base の色） | Alert は `border-warning`、badge は `border-warning-border` | 済 |
| 完了の知らせ（保存・取込みの成功・反映済み） | 完了 | 段 2 まで（塗りにしない。操作の塗りとは文字と icon で区別する） | 成功の toast、success tone の badge | 済 |
| 現在地（ナビ）と現在行（一覧の 1 行） | 現在地は操作、現在行は進行中 | 段 2（細いバー）。色で区別せず、置き場所と文言で区別する | 現在地は `border-l-primary`、現在行は `border-l-ongoing` と `bg-ongoing-soft`（`SupplierPickerDialog`） | 済 |
| 詳細を開いた行（在庫照会の行インライン展開 02 ⑫、操作ログの詳細の開閉） | 進行中（操作対象として開いている 1 行、DSR-22） | 段 2（左端のバー + 薄い地。3 点目の文言は直下に開いた詳細と「詳細を閉じる」の類の文言が担い、badge を足さない） | `ProductListTable` は選択行に `border-l-ongoing` と `data-[state=selected]:bg-ongoing-soft`、展開行に `bg-ongoing-soft`。操作ログは開いた行に `has-aria-expanded:bg-ongoing-soft`、詳細の行に `bg-ongoing-soft`。どちらも hover で色を動かさず、閉じた行は `border-l-transparent` で列をそろえる。`TableRow` の既定（`has-aria-expanded:bg-muted/50`・`data-[state=selected]:bg-muted`）は選択欄〈Radix Select〉を開いた入力行にも当たるため stone のまま | 済（試しを採った、D-094） |
| 取込みの手順の表示（`StepIndicator`、押せないステップの並び） | いまのステップは進行中（取り込んでいる作業の現在の段。ナビの現在地〈操作、DSR-21〉ではない）、済んだステップと先のステップはふつう・補足 | いまのステップは段 2（薄い地 + 1px 線、番号と名前は太字）、ほかは段 0（済んだステップの番号も muted） | いまのステップは `border-ongoing-border bg-ongoing-soft text-ongoing-strong` の丸と太字の名前、済んだステップと先のステップは `border-muted-foreground/30 text-muted-foreground`（`aria-current="step"` と位置で分ける） | 済（試しを採った、D-094） |
| ランキング 1 位 | 注意・確認（③強調の琥珀 pill。旧原則 4〈ランキング 1 位 / 最新〉の現行の形。2 位・3 位を含む見せ方は別 lane、`docs/backlog.md`） | 段 2（pill は琥珀の薄い地 + `--warning` の線で、塗りではない。行の地は線を引かない薄い地） | 1 位は `Badge` の `border-warning bg-rank-top-badge-bg text-rank-top-badge-text` の「1 位」と、行の `bg-rank-top-bg/40`（`--rank-top-*`）。2 位以下は `text-muted-foreground` の文字 | 済（順位と太字の試しは採らなかった、D-094） |
| 最新・上書き件数の強調（③強調の badge） | 注意・確認（③強調の琥珀 pill、owner 採用 2026-08-20、枠色は owner 決定 2026-09-05） | 段 2 に数える（押すボタンでない badge の塗りの pill。恒久の例外、D-094） | 琥珀の pill（`border-warning bg-warning-emphasis text-primary-foreground`。badge に既定の塗りは無い） | 済（stone の pill と太字の試しは採らなかった、D-094） |
| お知らせ一般（手を止めなくてよい知らせ） | ふつう・補足 | 段 0（Alert の既定） | Alert の既定は `bg-card`。`toast.info` は、入出庫の記録の取消で、状態を取り直すと取消済みだったときの拒否の文に使う（[65](../function-design/65-inventory-record-traceability.md) §65.5b） | 済 |
| 役割色の文字だけの表示（入力欄のエラー文・増減の ±〈月次の前月比を含む〉・在庫少と在庫切れのセルの数字・Home のサマリの件数・記録詳細のリンク・失敗ではない知らせの文〈未保存の案内〉） | エラー文と − と在庫切れは危険・失敗、+ は完了、在庫少と失敗ではない知らせの文は注意・確認、リンクは操作（0 件の件数と前月比の ±1.0% 未満は色を付けず、ふつう・補足） | 段に数えない（文字色は補助の信号。記号・文言・下線のどれかを必ず添え、icon は要しない。DSR-08） | `FieldError` の `text-destructive`、± の `text-success-strong` / `text-destructive-strong`（DSR-08）（前月比は地の chip を持たない `text-sm`）、セルと件数と未保存の案内の `text-warning-emphasis` / `text-destructive`、リンクの `text-primary` と hover の下線。進行中の地の上に載る注意・確認の文字は `-strong`（詳細を開いた行の在庫少のセルは `text-warning-strong`。`-emphasis` は進行中の地の上で 4.5:1 に届かない） | 済 |

---

## ラベルと値

補足情報とサマリカードの主値は、何の値かをラベルで言い切る（04 原則 5）。1 行に詰めて同じ薄さで並べない。

| 対象 | 見せ方 | 受け取り方 | 狙う効果 | 根拠 |
|---|---|---|---|---|
| 補足情報（商品名の下の部門・コード・前回数えた数・数えた日のような副情報） | ラベルと値の組にする。ラベルは小さく薄く（caption の 14px muted）、値はふつうの濃さ（本文の大きさ）。日付なら何の日付か、数なら何の数かをラベルで言い切る（例: ラベル「前回数えた数」と数、ラベル「数えた日」と日付） | どれが何の値かを迷わない | 読み違いと確かめ直しが減る | チャンク化（意味の組に分けると覚えやすい）、近接の法則（近いものは同じ組に見える） |
| サマリカードの主値 | ラベル（小さく薄く）と値（metric 30px、本書のタイポグラフィ表）の組にする。何の値かをラベルで言い切る | 大きな数が何の数かすぐ分かる | 画面をまたいで同じ型で読める | チャンク化、近接の法則 |
| 進み具合 | 中くらいの大きさで、数（済んだ数と全体の数）と棒の両方を見せる。棒の塗りは強調の段階に数えない | あとどれだけかが分かり、急かされない | 最後まで続けやすい | 目標勾配効果（ゴールが近いと分かると続けやすい。大きすぎると急かされる） |

---

## カラーパレット

**ベースパレット（Tailwind `stone` ベース + カスタムトークン）**:

| 用途 | 変数名 | Tailwind相当 | HEX | 根拠 | 役割 |
|------|-------|------------|-----|------|------|
| 背景 | `--background` | `stone-50` | #fafaf9 | 長時間凝視で目の負担が少ないウォームニュートラル | ふつう・補足 |
| 前景（本文） | `--foreground` | `stone-900` | #1c1917 | 実測 16.7:1（対 `--background`）/ 16.0:1（対 `--card`）、いずれも AAA（2026-09-03 訂正、旧「コントラスト比12.6:1（AAA+）」は対象背景の明記なしで誤記） | ふつう・補足 |
| カード背景 | `--card` | `stone-100` | #f5f5f4 | 背景との差分8% で情報ブロック識別 | ふつう・補足 |
| ボーダー（構造線） | `--border` | — | #cdc8c4 | 対 `--background` 実測 1.59:1（DSR-22「構造線は一段濃く」、補助的な区切り。単独のグループ信号にしない、DSR-16。2026-09-03 Lane 2 で旧 stone-200 相当〈≈1.20:1〉から濃化） | ふつう・補足（段 0 の線） |
| 操作枠 | `--border-strong` | — | #8a8480 | 対 `--background` 3.53:1・対 `--card` 3.38:1（DSR-22、`--input` が参照。2026-09-03 Lane 2 実装） | ふつう・補足（段に数えない） |
| 一覧 sticky 帯 | `--list-head` | `stone-200` | #e7e5e4 | 対 `--background` 1.20:1・対 `--foreground` 13.93:1（WCAG 相対輝度で実測。`thead` surface（件数行は `--background`）、Gated Amendment 2 S11 / Gated Amendment 5 S39、2026-09-03 Lane 2 実装） | ふつう・補足 |
| 操作面 | `--control-surface` | — | #fafaf9 | owner run 6 指定。対 `--card` #f5f5f4 1.02:1（面の差は僅少、枠 `--border-strong` 3.53:1 が操作対象の主信号）（Gated Amendment 7 S46） | ふつう・補足 |
| サブテキスト | `--muted-foreground` | — | #6b6560 | 対 `--card` 5.27:1・対 `--background` 5.50:1・対 `--ongoing-soft` 4.95:1（いずれも AA）、対 `--list-head` 4.58:1、対 `--foreground` 3.04:1（ラベルと値の濃さの差）。旧値 `stone-500` は対 `--card` 4.40:1 で AA に届かなかった。候補と選び方は D-094 | ふつう・補足 |
| 進行中の枠 | `--ongoing` | — | #2f7f86 | 対 `--background` 4.47:1、対 `--ongoing-soft` 4.02:1、対 棒の溝（`stone-200`）3.72:1（D-091 / D-094） | 進行中（現在行と詳細を開いた行の左端のバー、作業の進み具合の棒、待ちの spinner、作業中の囲みの枠） |
| 進行中の地 | `--ongoing-soft` | — | #e6f0f0 | 対 `--background` 1.11:1（地だけでは区別できず、バーか線と文言を添える）、対 `--foreground` 15.06:1 | 進行中（現在行・詳細を開いた行・いまのステップの地） |
| 進行中の線 | `--ongoing-border` | — | #7fb0b4 | 対 `--background` 2.29:1（3:1 未満のため、現在行のバーと作業中の囲みの枠に使わない） | 進行中（badge 大の要素の 1px 線: いまのステップの丸） |
| 進行中の文字 | `--ongoing-strong` | — | #123e43 | 対 `--ongoing-soft` 10.07:1 | 進行中（いまのステップの番号） |

forced-colors focus indicator: `globals.css` の unlayered `@media (forced-colors: active) { :focus-visible { outline: 2px solid Highlight; outline-offset: 2px } }`。component の `outline-none` はこの安全網を前提とし、unlayered で `outline` を上書きしない（Gated Amendment 3 追補 S16）。

**セマンティックカラー**（役割の列は本書「色の役割」の役割）:

| 状態 | `--{name}` | Tailwind | 用途 | 役割 |
|------|-----------|---------|------|------|
| Primary | `--primary` | — (#1d5c63) | 押すボタン・リンク・ナビの現在地・checked・focus ring（`--ring` は同じ値）。対 `--background` 7.28:1・対 `--card` 6.97:1（D-094） | 操作 |
| Success | `--success` | `green-700` (#15803d) | 取込み完了 | 完了 |
| Warning | `--warning` | `amber-600` (#d97706) | PLU通知、在庫少 | 注意・確認 |
| Destructive | `--destructive` | `red-700` (#b91c1c) | 在庫切れ | 危険・失敗 |
| Warning Soft | `--warning-soft` | `amber-50` (#fffbeb) | 在庫少 Badge soft 背景 | 注意・確認 |
| Warning Border | `--warning-border` | `amber-200` (#fde68a) | 在庫少 Badge outline | 注意・確認 |
| Warning Strong | `--warning-strong` | `amber-900` (#78350f) | warning 系強調テキスト | 注意・確認 |
| Warning Emphasis | `--warning-emphasis` | `amber-700` (#b45309) | 在庫少セル強調、未保存の案内、③強調の琥珀 pill の塗り（対 `--primary-foreground` 4.81:1） | 注意・確認 |
| Destructive Soft | `--destructive-soft` | `red-50` (#fef2f2) | 在庫切れ soft 背景 | 危険・失敗 |
| Destructive Border | `--destructive-border` | `red-200` (#fecaca) | 在庫切れ outline | 危険・失敗 |
| Destructive Strong | `--destructive-strong` | `red-900` (#7f1d1d) | 在庫切れ強調テキスト、増減数値マイナス（DSR-08） | 危険・失敗 |
| Success Soft | `--success-soft` | `green-50` (#f0fdf4) | ①状態 badge success tone の soft 背景 | 完了 |
| Success Border | `--success-border` | `green-200` (#bbf7d0) | ①状態 badge success tone outline | 完了 |
| Success Strong | `--success-strong` | `green-900` (#14532d) | ①状態 badge success tone 強調テキスト、増減数値プラス（DSR-08） | 完了 |
| Success Emphasis | `--success-emphasis` | `green-600` (#16a34a) | 増減用途は DSR-08 が置換、icon 用途 1 site（ProductImportPreview）残置 | 完了 |
| Rank Top BG | `--rank-top-bg` | `amber-50` (#fffbeb) | 1位行背景 | 注意・確認（③強調、ランキング 1 位。D-094） |
| Rank Top Badge BG | `--rank-top-badge-bg` | `amber-100` (#fef3c7) | 1位 Badge 背景 | 注意・確認（同上） |
| Rank Top Badge Text | `--rank-top-badge-text` | `amber-800` (#92400e) | 1位 Badge テキスト | 注意・確認（同上） |

各色の明色版（background 用）は `{color}-50` を使用し、コントラスト確保。soft・border・strong・emphasis の段の形は役割ごとの家族として残す。

**ニュートラル（stone）をウォームにする論拠（4根拠 × refactoring-ui §4 引用）**:

ニュートラルの stone はウォームのまま使う。役割色は暖色でそろえず、色相で役割を分ける（本書「色の役割」）。

> refactoring-ui §4 "Color": *"Pure grays look lifeless — add subtle saturation. For warm UIs, tint grays with yellow or brown; for cool UIs, use blue tint."*

1. **環境の一致**: 手芸店の商材（毛糸・布・木製道具）は暖色系。UIの色温度が店舗の雰囲気と一致することで、店主の感覚的ストレスが減る
2. **利用者との距離感**: テック系クール（Slate / Gray）は専門家向け感が強い。小売店主に対してウォームの方が親しみやすく、恐怖感を与えない
3. **長時間利用の疲労軽減**: 低彩度ウォームニュートラルは、青色光を強めるクール系に比べて長時間使用での目の疲労が少ない（一般論だが医学的裏付けあり）
4. **差別化**: 既存の在庫管理 SaaS は大半が Slate/Gray の没個性。Stoneベースは個別のアイデンティティを提供（IBM Carbon は Gray 10 ベース、Shopify は Gray、Polaris は Gray、Atlassian は Neutral N10）

---

## 4色エリアモデルの扱い

SCREEN_DESIGN.md §2 で定義された 4色エリア（緑=毎日の業務 / 青=商品管理 / オレンジ=入出庫 / 黄=システム管理）は、**画面遷移図（仕様書内の俯瞰図）限定**とする。

**実装UIでは使用しない**。理由:

> refactoring-ui §1 "Visual Hierarchy": *"Not everything can be important. Create hierarchy through size, weight, and color."*

4色全てがサイドバーに並ぶと、全エリアが同じ重要度で主張し、ヒエラルキーが崩壊する。サイドバーは **単色ウォームグレー**（`stone-100` 背景 + `stone-900` テキスト）で構成し、ナビの現在地だけを**操作の役割色の細いバー**（強調の段階 2、DSR-21）で示す。

グループ分類は**アイコン + テキスト見出し + 区切り線**で表現する（色ではなく構造で区分）。

---

## タイポグラフィ

| 階層 | サイズ | line-height | font-weight | 用途 |
|------|-------|------------|------------|------|
| h1 | 24px (1.5rem) | 1.3 | 600 | 画面タイトル（各ページ1つ） |
| h2 | 20px (1.25rem) | 1.35 | 600 | セクション見出し |
| h3 | 18px (1.125rem) | 1.4 | 600 | カード内見出し |
| body | 16px (1rem) | 1.5 | 400 | 本文、テーブルセル |
| label | 14px (0.875rem) | 1.5 | 500 | ラベル、ボタン、タブ |
| caption | 14px (0.875rem) | 1.5 | 400 | 補助説明、タイムスタンプ、ラベルと値のラベル。muted（`text-sm text-muted-foreground`）。12px は badge の中だけ。badge 以外の 12px は現行の画面に残り、その画面を触る lane が 14px へ寄せる |
| metric（数値強調） | 30px（既存 `text-3xl`） | 1.2 | 600 | サマリカードの主値。「ラベルは小さく muted、値は大きく」（04 原則 5、本書「ラベルと値」）が指す既存実装インスタンス。`src/features/stocktake/StocktakePage.tsx` の仕入原価総額の card の値（`formatYen(result.total_cost)`）で使用中の現行 Tailwind class を規範化（新規提案値ではない） |

**根拠**: refactoring-ui §2 "Hierarchy" *"Establish hierarchy through size, weight, and color — don't rely on size alone."* 本書では sizeとweightの組合せ（例: h1は size=24 + weight=600、body は size=16 + weight=400）で4段の階層を作る。

---

## 書体

**いま使う書体**: システムフォントスタック（`-apple-system, BlinkMacSystemFont, "Hiragino Kaku Gothic ProN", "Hiragino Sans", Meiryo, sans-serif`、`globals.css` の `--font-sans`）。最終採用が決まるまでは現行の system font stack を使い、画面ごとに書体を差し替えない。

**採用は決まっていない**: 候補は Noto Sans JP（owner の好み。同梱が要る）と BIZ UDPゴシック（Coordinator の推奨。Windows 標準に含まれ同梱が要らないという前提は確かめていない外部の事実で、runtime lane B が vendor の公式資料と店の PC で確かめる）。最終採用は runtime lane B の実機比較（店主が通常距離で読み分けられるか、owner の目）で決める。同梱したときのサイズと、店の PC にどの書体が入っているかは未実測。

**撤回した旧理由**: 旧版の「カスタム Web フォントは網から読み込むと表示が遅れるため採らない」は撤回した。Tauri の desktop 配布は書体を網から読まないため当たらない。判断軸は同梱サイズと店の PC での読み分けに置き換える。

選定条件（runtime lane B が候補をこの条件で比べる）:

| 選定条件 | 見せ方 | 受け取り方 | 狙う効果 | 根拠 |
|---|---|---|---|---|
| 見間違えにくさ | 数字と似た形のカナ・英字（0 と ロ・O、1 と l）を形で読み分けられる | 読み違えない | 数量とコードの読み違いが減る | 利用者前提（老眼、本書「業務ステータスの視認性」）、04 原則 11 |
| 太字でつぶれない | 見出しと強調の太字でも字の中の空きが残る | 太字の字も読める | 強調しても読みやすさが落ちない | refactoring-ui §2（大きさと太さで階層を作る） |
| 長時間読める | 本文 16px で字面が詰まりすぎない | 疲れにくい | 毎日の長い作業で読み続けられる | 利用者前提（長時間・繰り返しの操作）、03 の Fluent 2（Effortless） |
| 数字は等幅で桁がそろう | 表の数字を等幅で表示できる（等幅の数字の適用は runtime lane B） | 桁を縦に比べられる | 金額と数量の見比べが速くなる | 03 の IBM Carbon（データテーブル設計） |
| 配布の方法 | Windows 標準か同梱か、同梱ならそのサイズ | 見る人には見えない | 配布物と依存の変化を最小にする | npm 供給網ガード（D-030、依存を足すなら名指しと `--save-exact`） |

---

## スペーシング

Tailwind のデフォルトスケールを採用し、以下6段のみ使用:

| Token | px | 用途 |
|-------|----|------|
| `space-1` | 4 | アイコン隣接 |
| `space-2` | 8 | 要素間密着（ボタン内 padding 横） |
| `space-3` | 12 | 関連要素（ラベルとインプット） |
| `space-4` | 16 | セクション内の要素間 |
| `space-6` | 24 | ページ余白（`PageShell` の `p-6`）、セクション間、カード間 |
| `space-8` | 32 | 大セクション区切り |

**根拠**: refactoring-ui §3 *"Use a system, not guesswork."* 自由な値は許可せず、コードレビューで違反を検出。

---

## アイコンサイズ

| サイズ | 用途 |
|-------|------|
| 12px | badge の中（`badge` の svg 既定） |
| 16px | ボタン・ナビ・Alert・表内・ラベル隣接（`button` の svg 既定） |
| 20px | 見出し・結果の行の隣 |
| 24px | Home の入口 card・空状態（`EmptyState`） |
| 32px | 待ちの spinner・ファイル選択・画面単位のエラー・dialog の media icon（`AlertDialogMedia` の svg 既定、消費は `UnsavedChangesDialog`） |

表の 5 段以外のサイズは使わない。旧表（ボタン内 16・テーブルセル 20・画面タイトル横 24 の 3 段）は、2026-09-24 の実装の多数派に合わせてこの表で置き換えた。

---

## 業務ステータスの視認性

> 出典: `docs/SCREEN_DESIGN.md` §6（2026-06-07 追加）。横断規約として本ファイルへ移設。

**利用者前提**: 主利用者は非IT系の店舗オーナーで、日常業務中に長時間・繰り返し操作する。老眼や色の識別しづらさを前提に、読めること / 区別できることを機能要件として扱う。

**既存画面の視覚言語を継承する**: 新規画面や follow-up UI は、実装済み画面の共通レイアウト、テーブル / カード / チップ、spacing、typography、stone 系トークン、active / hover 表現を先に確認し、同じアプリとして見える範囲で改善する。ページごとにデザイン方向性が分裂する変更は、共有 UI 方針の変更として Plan Packet で明示する。

**色だけで意味を伝えない**: 在庫状態、警告、前月比、取込み結果などの業務ステータスは、赤 / 黄 / 緑などの hue だけで意味を符号化しない。日本語ラベル + アイコン / 形 / 位置 / バッジ / 状態列のいずれかを組み合わせる（WCAG 1.4.1）。

**色は二次シグナル**: セマンティックカラーは注意喚起の速度を上げるために使う。色を失っても「在庫切れ」「在庫少」「比較不可」などの意味が読める構造を優先する。

**L3 判定**: Windows native L3 で実利用者が状態を言い分けられない、または通常距離で読めない場合は polish ではなく機能欠陥として扱う。

関連: [01-decision-rules.md](01-decision-rules.md) DSR-08（semantic 色のみで意味を伝えない）。

---

## デスクトップアプリ前提の UI 設計制約

> 出典: `docs/SCREEN_DESIGN.md` §6（Phase 1 確定）。横断規約として本ファイルへ移設。

- **レスポンシブ不要**: 単一店舗 PC で動かす単一ウィンドウ前提。モバイル / タブレット対応はしない
- **初期ウィンドウ**: Tauri 起動時は 1280x800 / 最小 1024x720 の単一ウィンドウで開始する。業務テーブルの視認性を優先し、800x600 起動は採用しない
- **hover 許容**: タッチデバイス前提でないため hover 動作で情報補強してよい（tooltip / dropdown 等）
- **アクセシビリティ**: shadcn/ui + Radix UI primitives がキーボードナビ + ARIA 属性を担保。Vitest / RTL 基盤は PR #64 で導入済み

---

## URL 設計

> 出典: `docs/SCREEN_DESIGN.md` §6（feedback memory `feedback-desktop-app-url-design.md` 適用）。横断規約として本ファイルへ移設。

- **状態の URL 化**: タブ切替・フィルタ・選択中エンティティ等の状態は URL（route + search params）に持たせる。`useState` でローカル状態に閉じない
- **メリット**: テスト容易（URL で再現可能）/ F5 耐性 / queryKey 独立 / コード分割（route 単位 lazy load）/ 利用者が直接 URL 共有可能（将来 Web 拡張時の前提整備）
- **実装例**: 日次/月次レポートは `/reports/daily` / `/reports/monthly` の別 route。商品検索フィルタは search params
