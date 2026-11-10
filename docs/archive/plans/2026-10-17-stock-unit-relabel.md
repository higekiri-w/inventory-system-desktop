# Plan Packet: 単位の語の直しの runtime（個数の単位どうしの変更）

wave に属さない単独の lane（2026-10-17 起票。同じ日に Coordinator が 6 本の lane を同時に起票した。owner の決定は repo 外の回答台帳 TD-304「動かせるだけ動かす」、TD-295「並列にできるものは並列に」）。設計は merge 済みの D-119（PR #168、[decision-log](../../decision-log.md) D-119、owner の決定 U2 = b、repo 外の回答台帳 TD-297）で、本 lane はその単位の語の直しの分を実装する。branch は `agent/stock-unit-relabel`（main `f5d7aaf4` から）。同じ日に起票された他の lane は `agent/daily-report-sheet-design`・`agent/sd-bulk-import-design`・`agent/jan-alias-design`・`agent/sd-direct-read-runtime`・`agent/msi-distribution-docs`。file が重なるのは `agent/jan-alias-design`（下の「節の所有表」）。新しい D 番号は使わない（D-123 は要らない。下の Design Readiness）。

**規模と手法（1 lane のまま。横断の約束の表は作らない）**: 2026-10-17 に本 worktree（HEAD `f5d7aaf4`）で数えた。

- 触る Rust は 2 file（`src-tauri/src/biz/product_service.rs`・`src-tauri/src/db/product_repo.rs`）。`ProductUpdateRequest` / `ProductUpdates` の struct literal を `rg -n 'ProductUpdates \{|ProductUpdateRequest \{' src-tauri/src` で全部挙げ、`..Default::default()` を持たないのは 2 か所（`product_service.rs:1514` の `commit_import` の UPDATE、`product_service.rs:2787` の test `req_ii`）。ほかの 40 か所は `..Default::default()` で足りる（`product_service.rs` 27・`plu_export_service.rs` 5・`product_repo.rs` 8。数えた script は scratchpad の使い捨てで、結果だけを書く）。
- 触る TS は 4 file（`product-form-request.ts`・`ProductForm.tsx`・`StockUnitField.tsx`・生成物 `src/lib/bindings.ts`）と test 5 file（`product-form-request.test.ts`・`product-update-patch-contract.test.ts`・`StockUnitField.test.tsx`・`ProductForm.test.tsx`・`ProductFormPage.test.tsx`。2026-10-18 に本 worktree〈HEAD `17ae3d11`、code は `f5d7aaf4` と同じ〉で Matrix の F1〜F6 の行から数えた: `rg '^\| F[1-6] ' docs/plans/test-matrices/2026-10-17-stock-unit-relabel.md | rg -o '[A-Za-z-]+\.test\.tsx?' | sort -u` → `ProductForm.test.tsx`・`ProductFormPage.test.tsx`・`StockUnitField.test.tsx`・`product-form-request.test.ts`・`product-update-patch-contract.test.ts` の 5 行。S4 と下の「既存 test の書換え」）。
- wire の型の変更は 1 つ（`ProductUpdateRequest` に `stock_unit: Option<ProductStockUnit>`）。migration は無い（`products.stock_unit` の CHECK は 12 個の code のまま）。
- 横断の約束（範囲・0・失敗の順序・単位の混在）がかかる経路は 3 つ（商品修正の command、商品 CSV の preview、商品 CSV の commit）。数量・金額を変えない（個数の単位の価格の基準数量はどれも 1。共通規則 SPEC-UNIT-D2）ので、範囲と 0 は当たらず、効くのは失敗の順序と単位の種類の混在だけ。3 経路は 1 つの判定（今の単位と新しい単位がどちらも個数の種類か）を共有するので、BIZ に判定を 1 つ置き、3 経路から呼ぶ（下の Scope の S1）。1 lane・1 判定で足り、表は作らない。

## Workflow State

- Phase: archive
- Risk: R3
- Plan Commit: 290c8baa57ad74742bf122e40086316f81254377
- Amendments: none
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（subagent_type: writer）
- Plan Reviewer: fresh Opus 5.5 + Codex（GPT-6.1 Sol / high。`.local` の model の選び方の「R3 で複数の層にまたがる契約の初回」の行）。座組表どおり
- Final Reviewer: Fable 5.1（Claude 側、R3 以上）+ Codex（GPT-6.1 Sol 既定）。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Review Minimum: 1
- Human Gate: ready,merge,manual
- Branch: agent/stock-unit-relabel

遷移の記録:

1. kickoff → spec-check（2026-10-17、起草役）: Coordinator の発注（D-119 の U2 = b の runtime）を Scope にし、規模を数えて 1 lane のままと決め、Risk を R3 と記録した（下の Risk）。
2. spec-check → design（2026-10-17、起草役）: 対象の設計正本は共通規則 SPEC-UNIT-D2、[30](../../function-design/30-biz-product-service.md) BIZ-01-D7・BIZ-01-D8・§4.4・§4.9、[51](../../function-design/51-ui-product-form.md) UI-01b-D5・D22・D23、[20](../../function-design/20-io-product-repo.md) §2.3、[74](../../function-design/74-ui-operation-logs.md) §74.8。正本と code を突き合わせ、実装の前に決めておく穴が 7 つあった（下の Design Readiness の「本 commit で正本に足したもの」）。
3. design → plan-draft（2026-10-17、起草役、本 commit）: 上の 7 つを本 packet と同じ commit で正本（30・51・20・74）に置いた。owner に聞く設計の問いは残っていない（下の「owner の判断事項」）。plan-gate へは Coordinator が進める。
4. plan-draft → plan-gate（2026-10-17、Coordinator）: packet・Matrix と正本の直しは起草役の commit（`d6f57bbd`）にある。Coordinator は正本の差分（30 の上書きの段の `plu_dirty` の比べる列と操作ログの summary、51 UI-01b-D23 の送る判定と説明文、20・74 の行）と packet の前文・Workflow State・Owner Effort Budget を読み、D-119 の U2 = b の範囲を越えていないこと、366 との節の所有表があること、code・migration に触れていないことを確かめた。起草役が決めた 4 点（操作ログの既知の key に足さない、「現在庫」の欄の語、無効化の範囲の外の画面の古い語、spec-check の穴の追記）はそのまま Plan Review の判定に出す。Plan Review round 1 へ出す。
5. plan-gate（round 1 の是正、2026-10-17、Coordinator）: Plan Review round 1（Claude 側 fresh Opus 5.5・Codex GPT-6.1 Sol）の是正（Review Response 参照）。是正の前に相談役（Fable 5.1）に反例探しを頼み、test の穴 3 つ（長さどうしの preview・commit、ログの全行の mutant、L3 の記録詳細を開く順）を是正に入れた。起草役が `5fe0eae1` で直し、Coordinator が差分（30:18、51 の D11・§7.8・D23、DSR の実例、Matrix の T5・T7・T8・T10・T11・T14、AC6 の glob、L3 の fixture）を読んだ。Scope の範囲と方向は変えず、その場で直したので plan-gate に留める。Plan Review round 2 へ出す。
6. plan-gate（round 2 の是正、2026-10-18、Coordinator）: Plan Review round 2（Claude 側 fresh Opus 5.5・Codex GPT-6.1 Sol）の是正（Review Response 参照）。起草役が `ca7bdcc6` で直し、Coordinator が差分（Matrix の Failure Modes・T1・T4・T9 と Mutation の節、packet の前文の数・Ordinary Operation の 3 行目・S5・AC6 と baseline・L3-1・L3-2 の入力物）を読んだ。直したのは検証の計画と数と字面で、Scope の設計は変えていないので plan-gate に留める。Plan Review round 3（天井）へ出す。
7. plan-gate（round 3 の是正、2026-10-18、Coordinator）: Plan Review round 3（Claude 側 fresh Opus 5.5・Codex GPT-6.1 Sol）の是正（Review Response 参照）。round 3 は round の天井で、owner が「直して、追加の round なしで plan-approved」と決めた（repo 外の回答台帳 TD-315、介入 1 回目 / 予算 9 回）。起草役が `9cb6ac2c` で直し、Coordinator が差分（S5 と所有表の §7.5、AC3 の subshell と baseline、S1〜S3 の compile の単位、Writer への助言の定義、Matrix の anchor、comment の一覧）を読んだ。
8. plan-gate → plan-approved（2026-10-18、Coordinator、本 commit）: 独立の Plan Reviewer（Writer と別の context）のうち Claude 側が round 3 で P1・P2 が無いと報告し、Codex の P2 1 件は owner の決定（repo 外の回答台帳 TD-315）どおり直した（Review Response 参照）。Plan Commit は round 3 の是正と記録の後の版。実装の commit はまだ無い。
9. plan-approved → implementing（2026-10-18、Coordinator、本 commit。state だけ）: 第 1 便（#170・#173・#174・#175・#177）の後の main `44f1e666` を merge した（`4d843b2a`。衝突は 51 §7.9 の変更履歴の表の 1 行だけで、両方の行を残した。code の差分は無い）。発注の直前に AC1・AC3〜AC6 の baseline を逐語で測り直し、上の baseline と一致した（AC1 は 125 passed、AC4 は 6 file・96 passed、AC3 の再生成の差分は無し）。Writer（Opus 5.5 subagent、`subagent_type: writer`）へ Scope の S1〜S5 の実装を発注する。

## Owner Effort Budget

- 介入回数上限: 9（repo 外の回答台帳 TD-296 の 5a。既定 6 から上げた）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 9 | 5（closeout 時点。Plan Review round 3 の天井の判断 1〈TD-315〉、L3 の round 1〈TD-322〉、base 同期の後の L3 の round 2〈TD-324〉、Ready 1、merge 1〈Ready・merge は TD-325〉。lane の選択 TD-304 は本 lane の前の判断で数えない） | 0 | 4 | 9 = 5 + 0 + 4 |

plan-approved は Coordinator が進める（TD-296 の 4a）。owner は 2026-10-20 から手が使いにくい（記号か短文で答える）。L3 は Draft のまま溜め、owner の手が戻った後に他の lane の L3 と 1 回の session にまとめる（repo 外の地図 `.local/reports/autonomy-2026-10/map.md` §7.3。Final Review の pass までは L3 を待たずに進める）。介入のたびに、やろうとしていることが全体を顧みない対症療法になっていないかを考え直す（TD-296 の 5a）。

既定値・数え方・上限に届くときの扱いは `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の R3 の行の「Tauri command DTO」（`update_product` の入力 `ProductUpdateRequest` に field を足し、generated bindings が変わる）と「operator workflow」（商品修正の画面で単位を変えられるようになり、商品 CSV の上書きの行の error と結果が変わる）に当たる。DB の列・CHECK・migration は変えず、既存の値を書き換える経路も無い（単位の列を UPDATE するのは利用者が選んだ 1 商品か、上書きを選んだ CSV の行だけで、数量・金額は変えない）ので R4 に当たらない。実店舗データの露出は無い。

required gate の green / red: gate の定義は変えない。bindings の再生成と traceability の再生成を落とすと既存の required gate（bindings・traceability）が red になる（下の Registration / Generation Obligations）。Scope の予定 path を `scripts/ci/classify-changes.sh:55` の workflow の一覧と、`.local/checklists/kickoff.md` の一覧（AGENTS・CLAUDE・DEV_WORKFLOW・MANUAL・code_review・project-profile・agent-guidance・templates・`.agents`・`.claude/{rules,commands,skills}`・PR template）に当てると当たらない。Final Review Minimum は 1（R3、workflow gate の変更でない）。

Human Gate に `manual` を足す: 商品修正の画面（operator の画面）が変わるので、`docs/DEV_WORKFLOW.md` の Human Visual Confirmation の目視の枠が要り、Coordinator の発注も L3 の項目を求めた。L3 の項目は下の Test Plan。

## Goal

Goal Invariant:

### 最小完了条件

- 店の利用者が、商品修正の画面で、個数の単位の商品（例: 「個」で初期投入した毛糸）の単位を別の個数の単位（例: 「玉」）に直して保存でき、在庫の数・過去の明細の数・売価・原価は変わらず、在庫照会・商品一覧・記録の画面が新しい語で出る（共通規則 SPEC-UNIT-D2、UI-01b-D23）。
- 商品 CSV の上書きで、個数の単位どうしの `在庫単位` の違いを受けて単位を上書きできる（BIZ-01-D8 の上書きの段）。
- どの経路でも、種類をまたぐ変更（個数 ↔ 長さ）と長さの単位どうしの変更は保存されない（商品修正は `在庫単位は個数の単位どうしでだけ変えられます`、商品 CSV の preview は行の error、commit は TX 全体を戻す）。
- 単位を変えた記録が操作ログ（`product_update`、key `stock_unit`）に残る。単位だけの変更では PLU の未反映（`plu_dirty`）にならない。

### 失敗定義

- 種類をまたぐ単位の変更か、長さの単位どうしの変更が、どれか 1 つの経路（UI を通らない command・preview を通らない commit の wire を含む）で保存される。
- 単位の変更で、在庫・明細・売価・原価・price_history の値のどれかが変わる。
- 単位を変えたのに操作ログに `stock_unit` の前後が残らない（商品 CSV の上書きは TX の中で）、または単位を変えていないのに `stock_unit` の key が書かれる。
- 単位だけを変えた商品が `plu_dirty=1` になる、または単位の変更で既存の `plu_dirty` の規則（売価の変更・PLU 対象化・上書きの他の列の変更）が崩れる。
- 長さの単位の商品の修正画面で単位を変えられる、または `stock_unit` が送られる。

### 非目的

- 種類をまたぐ単位の変更（在庫 0 のときを含む。UI-01b-D23 の不採用）。巻・cm を単位の一覧から外すこと（原価の lane、D-119）。
- 原価の 1/100 円（原価の lane）。
- 操作ログ画面の商品修正専用の表示（名前の解決・「旧 → 新」の整形。74 §74.8 で決めた。BIZ-01-D7 の Revisit）。
- productUpdate の無効化の範囲（D-052）を記録詳細・売上の画面へ広げること（UI-01b-D23 の末尾）。
- 商品修正での在庫数の変更。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

設計を含む変更の実装なので、店の通常の操作列を置く。例の値は説明用の合成の値。本 lane の merge の後に下の 5 行が成り立つ。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 毛糸を単位「個」・初期在庫 0 で登録し、入庫画面で 12 を入庫した後（在庫 Excel から「個」で入った状態の再現） | 商品修正の画面で数量単位の select を開く | 個数の単位（個・枚・本・袋・箱・巻・組・セット・玉・丁）だけが選べ、m・cm は選べない。select の下に `単位の語だけが変わり、在庫の数は変わりません。過去の記録も新しい語で表示されます。` | select が押せる | UI-01b-D23 |
| 同じ商品 | 「玉」を選んで保存する | 商品一覧へ戻り、保存の toast。在庫照会・商品一覧で `12 玉`。単位を直した後に初めて開く入庫の記録詳細も「玉」で出る（数は同じ） | 保存できる | SPEC-UNIT-D2。前に開いて cache にある記録詳細は次の再取得の後（UI-01b-D23 の末尾）。CSV の初期在庫は参照の無い在庫変動だけを作り入庫の記録を作らない（`product_service.rs:1574` 付近）ので、記録詳細を見るには入庫画面で記録した商品を使う |
| 同じ商品 | 操作ログ画面で直近の「商品を更新しました」を開く | 要約に `stock_unit` と、old が `pcs`・new が `ball` の JSON（key の順は契約にしない。BIZ-01-D7） | — | 74 §74.8（key は訳さない） |
| 反物が単位 m で登録済み | 商品修正の画面を開く | 数量単位は読取専用、在庫の節の説明は `現在庫と数量単位は登録後に変更できません。` | — | UI-01b-D23 |
| 毛糸（個）と反物（m）が登録済み | 商品 CSV で毛糸の行を `在庫単位` = `玉`、反物の行を `玉` にして読み、反物の行のエラーを確かめてから毛糸の行だけを「上書き」にする | 毛糸の行は重複行（上書きを選べる）、反物の行は preview のエラー行 `在庫単位は個数の単位どうしでだけ変えられます（登録: m、CSV: 玉）`（エラー行には上書きの選択が出ない）。毛糸を上書きすると単位が「玉」になり、他の列が同じなら PLU の未反映にならない | 取り込める | BIZ-01-D8 の上書きの段 |

失敗と再試行: 種類をまたぐ変更を UI から選ぶ道は無い（長さの商品の select は読取専用、個数の商品の select は個数の単位だけ）。UI を通らない command で送られたときは BIZ が拒み、何も書かない。商品 CSV のエラー行は直して読み直す。commit の再検証で外れたときは TX 全体を戻し、どの行も書かない（今の BIZ-01-D5・D8 の型）。

Plan Review は、この列が「正常な条件で目的を達成できるか」と「危険な結果を出さないか」を別々に答える（`docs/DEV_WORKFLOW.md` Review Rules）。

## owner の判断事項

無い。製品の振舞い（個数の単位どうしだけを許す、U2 = b）は TD-297 で決まり、D-119 と正本にある。本 lane で owner が答えるのは Human Gate（L3-1〜L3-3、Ready、merge）だけ。

店主に新しく聞く必要がある事項: 無い（毛糸の「個」→「玉」の事実は `docs/project-memory.md:119`）。

起票時に起草役が決めたこと（Coordinator が覆せる。どれも製品の振舞いの範囲を変えない、正本の穴埋め）:

- 操作ログ画面の既知の key に `stock_unit` を足さない（74 §74.8 に理由。発注の「推奨と理由を書く」）。推奨の理由: 商品修正の他の key（`name`・`selling_price`・`tax_rate` 等）も辞書に無く（`src/features/operation-logs/OperationLogsPage.tsx:49`〜`:56` の `KNOWN_KEYS` は backup 系と `product_code`・`record_id`・`record_type` だけ）、`stock_unit` だけを訳すと同じ記録の中で読み方が分かれる。値の code（`pcs`）を語（個）にする整形も、商品修正専用の表示を作るときにまとめて行う方が 1 回で済む。足す案の利点（ラベルが日本語になる）は、値が code のまま残るので半分しか効かない。
- 「現在庫」の欄は保存するまで登録の単位の語のまま（UI-01b-D23）。理由: select のすぐ下の文が「在庫の数は変わりません」と伝え、保存すると一覧へ戻る（UI-01b-D14）。select に合わせて語を変える案は、edit の「現在庫」が form の値（文字列）として作られる今の形（`product-form-request.ts:96`）では在庫の数を別に渡す配線が要る。
- 商品 CSV の上書きで単位を変えた商品の操作ログの summary は商品修正と同じ文（30 BIZ-01-D8）。

## Scope

実装の順序（S1〜S5）。S1〜S3 は 1 つの compile の単位とする（S1 は S3 で足す `ProductUpdates.stock_unit` を使い、途中では compile が通らない）。S1〜S3 を終えた後に対象の Rust の test（AC1・AC2）を通し、その後に S4、S5 の順で、各段で対象 test を通してから次へ進む。

### S1 BIZ: 商品修正（`src-tauri/src/biz/product_service.rs`）

- `ProductUpdateRequest` に通常 field `pub stock_unit: Option<ProductStockUnit>` を足す（`#[serde(default)]` の struct の通常 field。PRODUCT-PATCH-D1 の omitted / null は更新しない）。
- 個数の単位どうしの判定を 1 つの private 関数にする（例: `stock_unit_change_allowed(existing, new) -> bool` = 同じか、どちらも `stock_unit_kind` = `Count`）。S1 と S2 の 3 経路がこれを呼ぶ。`stock_unit_kind` は既存（`src-tauri/src/biz/unit_amount.rs:18`）。
- `validate_update_request` は今の検査（`product_service.rs:1092`〜`:1127` の商品名・売価・原価・部門・取引先）の後に単位の検査を置く。`update_product` は `existing` を読んだ後に `validate_update_request` を呼ぶ（`:332`〜`:336`）ので、単位の検査には `existing.product.stock_unit` を渡す（関数の引数を足すか、`update_product` の中で検査する。どちらでもよいが、BEGIN の前）。外れれば `BizError::ValidationFailed("在庫単位は個数の単位どうしでだけ変えられます")`。
- `ProductUpdates { stock_unit: … }` に、値を持ち今の単位と違うときだけ入れる（同じなら入れない。UPDATE の SET に出さない）。`plu_dirty` の条件（`:383`〜`:385`）は変えない（単位は `plu_dirty` の理由にならない）。
- 操作ログ: `record("stock_unit", json!(prev.stock_unit.as_str()), req.stock_unit.map(|v| json!(v.as_str())))` を他の key と同じ closure で足す（BIZ-01-D7。値が今と同じなら書かない）。

### S2 BIZ: 商品 CSV（同じ file）

- preview（`product_service.rs:1400`〜`:1413`）: 既存の商品の行で単位が違うとき、判定が許せば error にしない（重複行になる）。許さなければ行の error `在庫単位は個数の単位どうしでだけ変えられます（登録: {登録の語}、CSV: {値}）`（`{値}` は今どおり CSV の生の値 `raw_stock_unit`）。`stock_unit_changed_message`（`:994`〜`:1000`）の文を差し替える（旧い文 `在庫単位は登録の後に変えられません` は消す）。
- commit の再検証（`import_row_commit_error`、`:1027`〜`:1053`）: 上書きの行で `row.stock_unit` が `Some(code)` のとき、(1) `ProductStockUnit::from_code(code)` が `None` なら `在庫単位が正しくありません: {code}`（新規の行と同じ文、BIZ-01-D8 の上書きの段）、(2) 既存と違い判定が許さなければ上の文（`{値}` は code）。どちらも呼び出し側（`:1499`〜`:1506`）が `ValidationFailed` で TX 全体を戻す（今の流れのまま）。
- commit の UPDATE（`:1514`〜`:1529`、`..Default::default()` を持たない literal）: `stock_unit` を足し、wire の行の単位が既存と違うときだけ `Some(unit)`。`plu_dirty` は「単位だけの変更」なら `None`（変えない）、それ以外は今どおり `Some(true)`。「単位だけの変更」の判定は 30 BIZ-01-D8 の列の一覧（JAN・商品名・部門・売価・原価・税率は必ず比べ、取引先・メーカー品番・在庫連動・PLU対象は wire の行が `Some` のときだけ比べる）を既存の商品（`existing`、`:1500`）と比べる。単位が同じか空の行は「単位だけの変更」に当たらない（今どおり `plu_dirty=1`）。
- 単位を変えた商品ごとに、UPDATE が 1 行に当たった（`updated == true`）後、同じ TX の中で `system_repo::insert_operation_log` を呼ぶ（`operation_type` = `product_update`、summary = `商品を更新しました: {商品コード}`、detail_json = `{"stock_unit":{"old":code,"new":code}}` を serde_json で組む）。TX の外の件数の要約のログ（`:1618` の `product_import`）は変えない。

### Writer への助言（Plan Review round 1 の相談役の反例探しから。契約は増やさない）

- 上書きの「単位だけの変更」は、列の一覧を code に別に書かず、UPDATE に渡す `updates` の `Some` の field を `existing` の値と比べて求める（`updates.stock_unit` が `Some` で、かつ `stock_unit` と `plu_dirty` を除く `Some` が全部既存と同じなら単位だけ）。こうすると 30 BIZ-01-D8 の列の一覧と UPDATE が書く列が 1 か所で一致する。JAN は `updates.jan_code` が常に `Some`（CSV の JAN が空なら `Some(None)`）なので、既存に JAN がある商品の空の JAN の行は「単位だけ」にならない。
- 判定は `stock_unit_change_allowed(existing, new) -> bool` の 1 つ（同じか、どちらも個数）。commit は先に `ProductStockUnit::from_code` で未知の code を `在庫単位が正しくありません: {code}` で落とし、その後に判定を呼ぶ。
- commit の商品ごとの `product_update` のログは TX の中で `?` で返す（失敗すれば TX 全体が戻る）。TX の外の件数の要約のログの `tracing::warn!` で握りつぶす型（`product_service.rs:1626`）を真似ない。

### S3 IO: `src-tauri/src/db/product_repo.rs`

- `ProductUpdates`（`:196`〜`:211`）に `pub stock_unit: Option<ProductStockUnit>` を足し、`update_product`（`:1023`〜）の SET の動的構築に `stock_unit = ?n`（`unit.as_str()`）を足す。repo は値を検査しない（BIZ-01-D8）。CHECK は 12 個の code なので、enum から来る値は CHECK を満たす。

### S4 bindings と frontend

- repo の root から `(cd src-tauri && cargo run --bin generate_bindings)` で `src/lib/bindings.ts` を再生成する（`ProductUpdateRequest_Deserialize` に `stock_unit?: ProductStockUnit | null`、`_Serialize` に `stock_unit: ProductStockUnit | null`）。手で編集しない。
- `src/features/products/lib/product-form-request.ts` の `buildUpdateProductRequest`（`:166`〜`:201`）: `isLengthUnit(original.stock_unit)` が偽で、`values.stockUnit !== original.stock_unit` のときだけ `request.stock_unit = values.stockUnit`（UI-01b-D23 の「送る判定は登録の単位で行う」）。`isLengthUnit` は既存（`src/lib/unit-quantity`）。
- `src/features/products/components/StockUnitField.tsx`: edit mode で個数の単位の商品は select を押せ、選択肢を個数の単位だけ（`STOCK_UNIT_LABELS` の順から長さの単位を除く）にする。長さの単位の商品は今どおり `disabled`（押せず、今の語が読める。disabled の select の選択肢の並びは正本も本 packet も決めない）。edit の個数の単位の商品だけ select の下に UI-01b-D23 の 1 文を出す。在庫連動の自動提案（create だけ）は変えない。
- `src/features/products/components/ProductForm.tsx`: StockUnitField に渡す `readOnly={mode === "edit"}`（`:431`。`:418` の現在庫の input の readOnly は変えない）を「edit かつ長さの単位」にする。在庫の節の説明文（`:404`〜`:409`）を UI-01b-D23 の 3 つにする（edit は `isLengthUnit(values.stockUnit)` で分ける。edit の個数の商品の select は個数の単位しか出さないので、form の値で分けても登録の単位で分けても同じ）。
- `ProductFormPage.tsx` は変えない（保存は `buildUpdateProductRequest` の結果を渡すだけ。BIZ の拒否は今の `describeError` → form の alert で出る。`isDirty` は form の値の全 field を比べる〈`:183`〜`:185`〉ので単位の変更も離脱の警告に入る）。

### S5 正本の同期と生成物（実装の commit と同じ PR）

- 実装した契約の `proposed・未実装` を外し、今の規則の文を新しい規則に置き換える。外す・置き換える文（2026-10-17 の main で `rg` した。着手時に数え直す）:
  - `rg -n 'proposed・未実装.{0,2}単位の語の直しの lane' docs --glob '!docs/archive/**' --glob '!docs/plans/**'` → 12 行: `10-common-rules.md:94`、`20-io-product-repo.md:173`、`30-biz-product-service.md:18`・`:23`・`:178`・`:184`・`:216`、`51-ui-product-form.md:29`・`:37`（UI-01b-D11）・`:218`（§7.8 の UI-01b-D11 の箇条）、`design-system/01-decision-rules.md:184`（DSR-05 の実例の 1 文だけ。規則の本体は変えない）、`design-system/02-component-catalog.md:253`（disabled の例の句だけ）。UI-01b-D11 と §7.8 は「個数の単位の商品は個数の単位だけの候補で変更でき、長さの単位の商品は押せず今の語が読める」に、design-system の 2 つは「長さの単位の商品の数量単位 select は `disabled`」にする。
  - `rg -n '単位の語の直しの lane の後|今は単位を一切変えない|今は update で単位を変えない|`stock_unit` 変更は在庫履歴|上書き（§4.9 の `overwrite_codes`）の今の規則' docs --glob '!docs/archive/**' --glob '!docs/plans/**'` → 8 行: `10-common-rules.md:93`（今の規則の文。「種類をまたぐ変更は許さない」は残し、「今は単位を一切変えない」以下を置き換える）、`30-biz-product-service.md:17`・`:22`（今の上書きの規則の段を消し、`:23` の段を今の規則にする）・`:23`・`:528`（§4.9 の手順 2 d. の括弧の単位の分）、`51-ui-product-form.md:48`（UI-01b-D22 の末尾）、`SCREEN_DESIGN.md:334`（edit mode の読取専用の行）、`project-memory.md:119`（「単位の語の直しの lane の後」）。
  - `rg` の型に当たらない 51 の文: UI-01b-D5 の理由の列（`ProductUpdateRequest` は `stock_unit` / `stock_quantity` を更新しない）、§7.3 の「表示のみ」の列挙（`stock_unit`）、§7.5 の Edit の手順 2 の editable fields の列挙（`stock_unit` を個数の単位の商品について足す）と手順 3 の read-only fields の列挙（`stock_unit` を無条件に read-only とする句〈`51-ui-product-form.md:152`〉を長さの単位の商品に限る）、§7.6 の `stock_unit` の検査（create だけの文はそのまま、edit は select が個数の単位だけを出すことを足す）、§7.7 の Non-scope「edit mode での `stock_unit` / `stock_quantity` 変更」（`stock_quantity` だけにする）、§7.8 の edit payload（`stock_unit` は個数の単位どうしの変更だけ送る）。
  - `rg` の型に当たらない 10 の文: `10-common-rules.md:66` の契約 ID の段のうち D2 の句（「単位の語は登録の後に個数の単位どうしで直せるようにした（D2）。どれも proposed・未実装で」の D2 の分だけを今の契約にする。D1・D5 の句は原価の lane の所有なので触らない）。
  - code の comment（実装の commit で直す。正本の sweep の `rg` には当たらない）: `product_service.rs:1035`（「上書きの行: 単位は登録の後に変えない」）・`:1400`（「既存の商品の行（上書きの候補）は単位を変えられない」）・`:3713`（改名する test `test_import_req104_overwrite_cannot_change_stock_unit` の comment「上書きで単位は変えられない」。種類をまたぐ変更の拒否を表す文にする）、`StockUnitField.tsx` 冒頭の comment（edit の個数の単位の候補を足す）、`ProductForm.tsx:5` と `ProductForm.test.tsx:68`（「数量単位 select は disabled 維持」を長さの単位の商品に限る）。
  - 30 §4.4 の部分更新 wire 契約の表と BIZ-01-D7（30:216）の `stock_unit` の注記 2 つを外す: key の列挙の `proposed・未実装〈単位の語の直しの lane、D-119〉`（上の 1 本目の `rg` に当たる）と、値の型の列挙の `` `stock_unit`（proposed・未実装、D-119） ``（lane の名前が無く 1 本目の `rg` に当たらない。AC6 の 3 本目）。lane の名前の無い同じ形の注記は、2026-10-18 に `rg -n 'proposed・未実装、D-119' docs --glob '!docs/archive/**' --glob '!docs/plans/**'` で洗い、30:216 の他は 51 §7.9 の変更履歴の表の 2 行（`:230`・`:231`、履歴なので書き換えない）だけ。30 §4.9 の手順 2 d.（単位だけの行は `plu_dirty` を変えず、単位を変えた商品ごとに log）。
- 履歴の節（51 §7.9 の表、10 の変更履歴の表、decision-log の D-113・D-119、`docs/Plans.md:32`、`docs/backlog.md` の D-119 の段）は書き換えない。
- `docs/backlog.md` の「単位の語の直しの lane」の項目は closeout が消す（本 lane の実装の commit では触らない）。
- REQ の増減は無い見込み（既存の REQ-102・REQ-104 の test を足す）。test に REQ を書いたら `cd src-tauri && cargo run --bin generate_traceability` で `docs/function-design/90-traceability.md` を再生成する（AUTO-GENERATED、手で編集しない）。

### 作る所・読む所・fixture・呼び方（`ProductUpdateRequest`・`ProductUpdates` の形の変更。2026-10-17 の main で `rg -n 'ProductUpdateRequest|ProductUpdates' src-tauri src` を全件見た）

| 種類 | 場所 | 本 lane での扱い |
|---|---|---|
| 定義 | `product_service.rs:52`（`ProductUpdateRequest`）、`product_repo.rs:196`（`ProductUpdates`） | field を足す（S1・S3） |
| 作る所（production） | `product_service.rs:371`（`update_product`）、`:523`（`revise_product_price`）、`:571`（`toggle_discontinue`）、`:664`・`:677`（bulk の PLU 対象）、`:1514`（`commit_import`）、`plu_export_service.rs:659` | `:371` と `:1514` だけ値を入れる。他は `..Default::default()` のまま（単位を変えない） |
| 読む所 | `cmd/product_cmd.rs:35`（command の関数。入力の型は `:38`、BIZ の呼出しは `:44`。薄いまま変えない）、`product_repo.rs:1023`（`update_product`）、`src/lib/bindings.ts:10`・`:1296`〜`:1323`（生成物） | S3・S4 |
| frontend の作る所 | `src/features/products/lib/product-form-request.ts:166`〜`:201` | S4 |
| test の呼び方（Rust、`..Default::default()` あり） | `product_service.rs` の test 22 か所（`:2407`〜`:4955`）、`plu_export_service.rs:1827`・`:1843`・`:1871`・`:2413`、`product_repo.rs` の test 8 か所（`:2199`〜`:2526`） | 変えない（field の追加で壊れない） |
| test の呼び方（Rust、literal） | `product_service.rs:2787`（`test_update_product_req102_detail_json_unchanged_fields_omitted` の `req_ii`、全 field を更新前と同じ値で送る） | `stock_unit: Some(<更新前の単位>)` を足す（「全 field を同じ値で送ると log は NULL」の意図を保つ。`None` にすると全 field の意図が欠ける） |
| test の呼び方（Rust、JSON） | `product_service.rs:2891`〜`:2953`（serde の omitted / null の test） | 変えない。`stock_unit` の omitted / null / value の行を足す（Matrix の T4） |
| test の fixture（TS） | `src/features/products/lib/test-fixtures.ts:21`（`stock_unit: "pcs"`） | 変えない。既定が個数の単位なので、edit の select が押せるようになる（下の既存 test の書換え） |
| test（TS、生成物の形） | `product-update-patch-contract.test.ts:24`〜`:32`（`ORDINARY_FIELDS`）・`:87`・`:95` の「nine」 | `ORDINARY_FIELDS` に `stock_unit` を足し、test 名の数を「ten」にする |
| test（Rust の設計照合） | `src-tauri/tests/design_compliance_test.rs:72`（`cmd::product_cmd::update_product` の対応） | signature は変えないので変えない |

### 既存 test の書換え（意図を変えずに正本の変更に合わせる。弱めない）

| test | 今の期待 | 書換え | 理由 |
|---|---|---|---|
| `product_service.rs` `test_import_req104_overwrite_cannot_change_stock_unit`（`:3712`） | m の商品を `玉` で上書きすると preview の error `在庫単位は登録の後に変えられません（登録: m、CSV: 玉）`、commit は拒む | 名前を種類をまたぐ変更の拒否を表す名前（例: `test_import_req104_overwrite_rejects_cross_kind_stock_unit`）にし、文を新しい文 `在庫単位は個数の単位どうしでだけ変えられます（登録: m、CSV: 玉）` にする。在庫・明細の金額が変わらない assert は残す | 長さ → 個数の拒否は新しい規則でも成り立つ。文だけが変わる（BIZ-01-D8） |
| `ProductForm.test.tsx:44`〜`:71`（edit の `数量単位` が `toBeDisabled()`、fixture は `stock_unit: "cm"`） | 長さの単位の商品の select は押せない | 変えない（長さの単位の商品は読取専用のまま。`:68` の comment だけ S5 で直す）。`pcs` の商品で select が押せる test を別に足す（Matrix の F3） | UI-01b-D23（個数の単位の商品は押せる、長さは読取専用） |
| `StockUnitField.test.tsx` の edit の行（`:300`〜`:325`、`readOnly: mode === "edit"` で渡す） | edit は readOnly | 渡し方を「edit かつ長さの単位」に合わせる。edit の `m` で提案の案内が出ない期待は残す | 同上 |
| `product-form-request.test.ts:128`〜`:158`（cm の商品の form 値を `pcs` にしても `stock_unit` を送らない） | 送らない | 変えない（UI-01b-D23 の「登録の単位が長さなら送らない」がこの期待を正本にした） | — |
| `product-update-patch-contract.test.ts` の 2 つの「nine」の test | 9 field | 10 field | wire の型の変更 |

`FormSection.test.tsx:40`〜`:41` は商品フォームの在庫の節の文を例の文字列として使うが、`ProductForm` を render しない（`FormSection` 単体の test）。新しい文と衝突しない（下の matcher の確かめ）ので変えない。

### 表示の文言と test の matcher の衝突（`.local/checklists/kickoff.md` の行）

2026-10-17 の main で確かめた: `rg -F '単位の語だけが変わり' src --glob '*.test.*'` → 0、`rg -F '個数の単位どうし' src --glob '*.test.*'` → 0、`rg -F '在庫の数は変わりません' src` → 0、`rg -n 'getByText\(/|queryByText\(/|findByText\(/' src/features/products --glob '*.test.*'` の正規表現 matcher は 7 件（2026-10-18 に本 worktree〈code は `f5d7aaf4` と同じ〉で数え直した。`… | wc -l` → `7`）: `ProductFormPage.test.tsx:123` `/独自コード発番対象/`、`ProductListPage.test.tsx:149` `/現在の絞り込み条件に一致する 37 件が対象です/`・`:150` `/現在の絞り込みに一致しない商品は変更されません/`・`:151` `/PLU 書出しと PC ツールの取込みが別途必要です/`、`components/ProductTable.test.tsx:215` `/pcs/`、`components/StockUnitField.test.tsx:322` `/在庫同期しない設定を提案します/`、`components/ProductForm.test.tsx:117` `/商品「在庫商品」は商品一覧の通常表示から外れます/`。新しい文（`変更できます`・`変更できません`・`変わりません`・`単位の語だけが変わり`）のどれにも当たらない（`/pcs/` は表の行の中の語の検査で、新しい文は `pcs` を含まない）。`rg -n '登録後に変更できません|登録後は数量単位' src` は `ProductForm.tsx:177`（商品の識別の節。変えない）・`:407`・`:408` と `FormSection.test.tsx:11`・`:17`・`:40`・`:41`（上の段落）だけ。

## 節の所有表（他の lane と重なる file）

`agent/jan-alias-design`（Coordinator の発注 366、D-122、docs だけ）が JAN の別名を 30・20・51・商品 CSV の正本に足す見込み。本 lane は下の「本 lane」の列の節だけを書き換え、他の節を書き換えない。同じ文の中に両方の話題がある所は、文の中の自分の話題の句だけを書き換え、merge の順で解く。

| file | 本 lane が書き換える節・句 | `agent/jan-alias-design` に残す節・句 |
|---|---|---|
| `docs/function-design/30-biz-product-service.md` | 冒頭の「単位と原価の精度の契約（BIZ-01-D8）」のうち単位の変更の段（`stock_unit` の段・update の段・上書きの段）、§4.4 の `ProductUpdateRequest` の field の列挙・部分更新 wire 契約の表・手順 2・BIZ-01-D7 の key の列挙、§4.9 の手順 2 d. の括弧のうち単位の句（「単位の語の直しの lane の後は …」） | §4.4 の手順 4b（JAN の遷移）、§4.9 の手順 2 d. の JAN の句（「1→0 と JAN 変更は …」「JAN の変更もこれで未反映になる」）、§4.8・§4.9 の JAN の検査、別名の新しい節 |
| `docs/function-design/20-io-product-repo.md` | §2.3 の `ProductUpdates` の `stock_unit` の行だけ | `ProductUpdates` の `jan_code` の行（今の文書に無い。code にはある〈`product_repo.rs:197`〉。足すかは JAN の側の判断）と別名の repo の節 |
| `docs/function-design/51-ui-product-form.md` | UI-01b-D5 の `stock_unit` の句、UI-01b-D11 と §7.8 の数量単位 select の句、UI-01b-D22 の末尾、UI-01b-D23、§7.3・§7.6・§7.7・§7.8 の `stock_unit` の句、§7.5 の Edit の手順 2・3（editable / read-only fields の列挙）の `stock_unit` の句、§7.9 の表の本 lane の行 | UI-01b-D5 の `jan_code` の句、JAN の入力・検査（UI-01b-D17・D18）、別名の問いの UI |
| `docs/backlog.md` | 「単位の語の直しの lane」の項目、「原価の lane（C）」の項目の下に足した 1 行 | 「TD-284 の lane」の項目 |

ほかの 4 本（`agent/daily-report-sheet-design`・`agent/sd-bulk-import-design`・`agent/sd-direct-read-runtime`・`agent/msi-distribution-docs`）は、発注書の Scope の file（32・37・29・pos-tables・io の SD・MSI の手順の docs）が本 lane の Scope の file と重ならない（発注書 363・364・367・368 の Scope を読んで当てた）。`docs/decision-log.md` は本 lane で触らない。原価の lane（未起票）は `ProductUpdateRequest` を触るので、本 lane が先に入る前提を `docs/backlog.md` の原価の lane の項目に書いた。

## Non-scope

- 種類をまたぐ単位の変更、巻・cm の削除、原価の 1/100 円、操作ログ画面の商品修正専用の表示、productUpdate の無効化の範囲の拡大（Goal の非目的）。
- `ProductUpdates` の `jan_code` を 20 に足すこと（上の所有表。JAN の側）。
- 商品 CSV の preview の重複行に、単位が変わることを示す表示を足すこと（正本に無い。上書きの確認ダイアログは今のまま）。
- `src/components/patterns/FormSection.test.tsx` の例の文字列の更新。
- `docs/design-system/reference/mockup-d-forms-a.html:201`・`:203` の数量単位 select の disabled の表示（静的な mock で、正本の契約ではない）。

## Acceptance Criteria

- AC1（BIZ の商品修正と商品 CSV）: `cd src-tauri && cargo test --lib biz::product_service` が exit 0 で、Matrix の T1〜T12・T14 の test（名前は Matrix）が通る。種類をまたぐ変更・長さどうしの変更・未知の code の wire を拒む test と、個数どうしの変更が在庫・明細・売価・原価・price_history を変えない test を含む。
- AC2（IO）: `cd src-tauri && cargo test --lib db::product_repo` が exit 0 で、`ProductUpdates.stock_unit` が SET に出る test（Matrix の T13）が通る。
- AC3（bindings）: repo の root で `(cd src-tauri && cargo run --bin generate_bindings)` を実行し（cargo の実行は subshell に閉じ、cwd は root のまま）、続けて root で `git diff --exit-code src/lib/bindings.ts` が exit 0（commit 済みの生成物と一致）。root で `rg -c 'stock_unit\?: ProductStockUnit \| null' src/lib/bindings.ts` が 1 以上（`ProductUpdateRequest_Deserialize` の行）。
- AC4（frontend）: `npm test -- src/features/products/lib/product-form-request.test.ts src/features/products/lib/product-update-patch-contract.test.ts src/features/products/components/StockUnitField.test.tsx src/features/products/components/ProductForm.test.tsx src/features/products/ProductFormPage.test.tsx src/components/patterns/FormSection.test.tsx` が exit 0。
- AC5（旧い文の削除と新しい文）: `rg -n '在庫単位は登録の後に変えられません' src-tauri/src src` が 0 行。`rg -c '在庫単位は個数の単位どうしでだけ変えられます' src-tauri/src/biz/product_service.rs` が 1 以上。
- AC6（正本の同期）: S5 の 2 つの `rg`（`proposed・未実装.{0,2}単位の語の直しの lane` と `単位の語の直しの lane の後|今は単位を一切変えない|…`）が docs（archive と `docs/plans/` を除く）で 0 行。`` rg -n '`stock_unit`（proposed・未実装、D-119）' docs/function-design/30-biz-product-service.md `` が 0 行（baseline 1 行、30:216 の値の型の列挙）。
- AC7（docs と生成物の検査）: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan` が exit 0（ERROR 0）。`cd src-tauri && cargo run --bin generate_traceability -- --check` と `cargo test --test design_compliance_test` が exit 0。
- AC8（全体の gate）: `docs/DEV_WORKFLOW.md` Verification Gates の Rust/backend と Frontend の行の command が exit 0。L3 を含むので Writer は `cargo check --release` を通す（`docs/DEV_WORKFLOW.md` Implementation Rules）。

baseline（2026-10-17、Plan Review round 1 の是正の commit の直前に本 worktree〈HEAD `734671ed` に是正の docs の変更を載せた状態。code は `f5d7aaf4` と同じ〉で逐語で測り直した。実装の前の観測値。AC6 の 1 本目は是正で正本に proposed の注記を 4 つ足したので 12 行）:

```
$ cd src-tauri && CARGO_TARGET_DIR=<tmp> cargo test --lib biz::product_service
test result: ok. 125 passed; 0 failed; 0 ignored; 0 measured; 1114 filtered out; finished in 43.53s
$ npm test -- <AC4 の 6 file>
 Test Files  6 passed (6)
      Tests  96 passed (96)
$ rg -n '在庫単位は登録の後に変えられません' src-tauri/src src | wc -l
3
$ rg -c '在庫単位は個数の単位どうしでだけ変えられます' src-tauri/src/biz/product_service.rs; echo "exit=$?"
exit=1
$ rg -c 'stock_unit\?: ProductStockUnit \| null' src/lib/bindings.ts; echo "exit=$?"
exit=1
$ rg -n 'proposed・未実装.{0,2}単位の語の直しの lane' docs --glob '!docs/archive/**' --glob '!docs/plans/**' | wc -l
12
$ rg -n '単位の語の直しの lane の後|今は単位を一切変えない|今は update で単位を変えない|`stock_unit` 変更は在庫履歴|上書き（§4.9 の `overwrite_codes`）の今の規則' docs --glob '!docs/archive/**' --glob '!docs/plans/**' | wc -l
8
```

Plan Review round 2 の是正（2026-10-18、本 worktree〈HEAD `17ae3d11` に是正の docs の変更を載せた状態〉）で、`rg` の AC（AC3 の `rg`・AC5・AC6 の 3 本）を逐語で測り直した。code は `f5d7aaf4` と同じ（`git diff --stat f5d7aaf4 -- src src-tauri | wc -l` → `0`）なので、AC1・AC4 の test は上の round 1 の観測のまま測り直していない:

```
$ rg -n '在庫単位は登録の後に変えられません' src-tauri/src src | wc -l
3
$ rg -c '在庫単位は個数の単位どうしでだけ変えられます' src-tauri/src/biz/product_service.rs; echo "exit=$?"
exit=1
$ rg -c 'stock_unit\?: ProductStockUnit \| null' src/lib/bindings.ts; echo "exit=$?"
exit=1
$ rg -n 'proposed・未実装.{0,2}単位の語の直しの lane' docs --glob '!docs/archive/**' --glob '!docs/plans/**' | wc -l
12
$ rg -n '単位の語の直しの lane の後|今は単位を一切変えない|今は update で単位を変えない|`stock_unit` 変更は在庫履歴|上書き（§4.9 の `overwrite_codes`）の今の規則' docs --glob '!docs/archive/**' --glob '!docs/plans/**' | wc -l
8
$ rg -n '`stock_unit`（proposed・未実装、D-119）' docs/function-design/30-biz-product-service.md | wc -l
1
```

Plan Review round 3 の是正（2026-10-18、本 worktree〈HEAD `a3753d16` に是正の docs の変更を載せた状態。code は `f5d7aaf4` と同じ〉）で、形を変えた AC3 を repo の root から逐語で測った（cargo の target は並走と共有の `CARGO_TARGET_DIR=/tmp/shared-cargo-target`。実装の前なので `rg` は 0 件で exit 1 が期待どおり）:

```
$ (cd src-tauri && CARGO_TARGET_DIR=/tmp/shared-cargo-target cargo run --bin generate_bindings)
TS bindings exported to src/lib/bindings.ts
$ git diff --exit-code src/lib/bindings.ts; echo "exit=$?"
exit=0
$ rg -c 'stock_unit\?: ProductStockUnit \| null' src/lib/bindings.ts; echo "exit=$?"
exit=1
```

test の本数は AC にしない（参考の観測値。完了の判定は exit と Matrix の test の PASS）。AC2・AC3 の generate_bindings と AC7・AC8 は起票の時点で回していない（変更の前は意味が無い、または doc の検査は下の検証で回した）。

## Design Readiness

- 引用する設計正本（節まで）: [共通規則](../../function-design/10-common-rules.md) SPEC-UNIT-D2（単位の変更の段）・SPEC-UNIT-D1（12 個の code と語）・D10、[30](../../function-design/30-biz-product-service.md) の「単位と原価の精度の契約（D-113、BIZ-01-D8）」・§4.4（PRODUCT-PATCH-D1、BIZ-01-D7）・§4.8・§4.9、[20](../../function-design/20-io-product-repo.md) §2.3、[51](../../function-design/51-ui-product-form.md) UI-01b-D5・D14・D22・D23・§7.3〜§7.8、[74](../../function-design/74-ui-operation-logs.md) §74.8、[SCREEN_DESIGN](../../SCREEN_DESIGN.md) の「商品登録・修正画面」、[decision-log](../../decision-log.md) D-119。
- 必要な設計成果物（`docs/DEV_WORKFLOW.md` Design artifact selection）:
  - BIZ・IO・validation・error の変更 → 30・20: updated in this PR（本 commit で穴を埋め、実装の commit で `proposed・未実装` を外す）。
  - Tauri command の DTO・generated binding → 30 §4.4 と本 packet の Boundary / Wire Contract: existing sufficient（D-119 で置いた）。
  - persistence（audit/log、TX）→ 30 BIZ-01-D8 の上書きの段（TX の中の log）: updated in this PR（summary の文を足した）。DB の表・列は変えない（DB_DESIGN・db-design は existing sufficient）。
  - operator workflow・日本語の文 → 51 UI-01b-D23・SCREEN_DESIGN: updated in this PR（51 は本 commit、SCREEN_DESIGN は実装の commit で読取専用の行を直す）。
  - durable な横断の判断 → 無い（D-119 で足りる。D-123 は要らない）。
- 本 commit で正本に足したもの（spec-check で見つけた穴。実装の前に決める必要があった）:
  1. 30 BIZ-01-D8 の update の段: 単位の検査を手順 2 の今の検査の後に置く（失敗の順序）、同じ単位なら UPDATE も log も書かない。
  2. 30 BIZ-01-D8 の上書きの段: commit の文の `{値}` は wire の code、wire の単位が 12 個の code に無ければ `在庫単位が正しくありません: {code}`（未知の code は個数とも長さとも言えず、旧い規則では「違えば拒む」で塞がっていた穴）。
  3. 30 BIZ-01-D8 の上書きの `plu_dirty`: 「上書きで書く列」の一覧を §4.9 の手順 2 a. の UPDATE と同じに書いた。
  4. 30 BIZ-01-D8 の上書きの操作ログ: summary の文と、同じ行で他の列が変わっても単位の key だけを書くこと。
  5. 51 UI-01b-D23: 送る判定を登録の単位で行う、在庫の節の説明文 3 つ（D-119 の Final Review の P3、Fable #1）、「現在庫」の欄の語、productUpdate の無効化の範囲の外の画面の語。
  6. 20 §2.3: `ProductUpdates.stock_unit` の型を `Option<ProductStockUnit>` にした（発注の項目。proposed の注記つき）。
  7. 74 §74.8: 商品修正の key（`stock_unit` を含む）を辞書に入れない（発注の項目）。
- plan にしかない durable な判断の昇格先: 上の 7 つは正本に置いた。packet にだけある判断は無い。
- 前提・制約と、延期した design gap の follow-up: 前提は共通規則 SPEC-UNIT-D2 の「個数の単位は価格の基準数量がどれも 1」（`src-tauri/src/biz/unit_amount.rs:34`〜`:39` の `price_basis_quantity` で確かめた: `Count => 1`、`Length => 100`。出所 = 正本と実物の観測、2026-10-17）。productUpdate の無効化の範囲の外の画面は次の再取得まで古い語（UI-01b-D23 の末尾、受容した制約）。操作ログの key の整形は BIZ-01-D7 の Revisit へ延期（74 §74.8）。
- 絶対保証の例外と escape hatch の自己点検: 「種類をまたぐ変更は保存されない」の入口は 3 つ（`update_product` の command、商品 CSV の preview、commit）で、どれも BIZ の 1 つの判定を通す。単位の列を UPDATE する他の経路は無い（2026-10-17 の main で `rg -n 'stock_unit\s*=' src-tauri/src --glob '!**/schema_v*.rs' --glob '!**/migration*.rs'` を当て、production の SET は無い。当たる UPDATE は test の fixture の 5 か所〈`integrity_service.rs:851`・`sales_service.rs:1560`・`stocktake_service.rs:954`・`inventory_service/list.rs:817`・`inventory_service/test_support.rs:80`〉で、どれも `mod tests` か `#[cfg(test)] mod test_support` の中。`ProductUpdates` は今 `stock_unit` を持たない）。backup の restore は DB 全体を戻すので単位の変更の経路ではない。「過去の記録も新しい語で出る」の例外は上の cache（受容）。
- 判定: ready。上の 7 つの穴は正本に置き、未解決の設計の問いは無い。

## Registration / Generation Obligations

| 変更対象 | 義務 | 本 lane |
|---|---|---|
| Tauri command の DTO（`ProductUpdateRequest`） | `cargo run --bin generate_bindings` で `src/lib/bindings.ts` を再生成 | S4。AC3 |
| REQ / coverage の参照の増減 | `cargo run --bin generate_traceability` で `90-traceability.md` を再生成 | test に REQ を書けば S5。AC7 |
| command の新設・function-design doc の新設・route・operator 画面の新設 | — | 該当なし（command と画面は既存。route・navigation は変えない） |

## Impact Review Lenses

not applicable。実機調査・外部の道具・POS の取込み・CSV の形の変更から始まる lane ではない（商品 CSV の列と形は変えず、上書きの行の検査の結果だけが変わる。D-119 の design lane で lens を当てた）。

## Boundary / Wire Contract

- producer: frontend の `buildUpdateProductRequest`（`commands.updateProduct` の入力）。商品 CSV は IO-03 の file（形は変えない）→ BIZ の preview が作る `ImportRow` → frontend → `commit_import` の wire。
- consumer: CMD `update_product`（薄いまま）→ BIZ `update_product`。BIZ `commit_import`。
- wire type: `ProductUpdateRequest_Deserialize.stock_unit?: ProductStockUnit | null`（12 個の literal の union）。`ImportRow.stock_unit: string | null`（今のまま、code）。
- internal type: Rust `ProductUpdateRequest.stock_unit: Option<ProductStockUnit>`、`ProductUpdates.stock_unit: Option<ProductStockUnit>`、DB `products.stock_unit TEXT`（CHECK 12 個）。
- precision/range: 数でないので範囲は無い。値域は enum（command）と `ProductStockUnit::from_code`（CSV の wire）で閉じる。
- round-trip path: 商品修正の画面 → command → DB → `get_product` → form（`productToFormValues` が `stock_unit` を読む）。
- invalid input: command は enum の外を serde が拒む（今の `tax_rate` と同じ）。種類をまたぐ値は BIZ の `ValidationFailed`。CSV の wire の未知の code は `ValidationFailed`（BIZ-01-D8）。
- compatibility: field の追加だけで、omitted / null は「更新しない」（PRODUCT-PATCH-D1）。今の frontend（`stock_unit` を送らない）の request はそのまま受ける。`ProductUpdateRequest_Serialize` は command の入力に使わない（30 §4.4）。

## Test Plan

Test Design Matrix: [2026-10-17-stock-unit-relabel](test-matrices/2026-10-17-stock-unit-relabel.md)。

- targeted tests: AC1〜AC4 の command。
- negative tests: 種類をまたぐ変更（個数 → 長さ、長さ → 個数）、長さどうし（m → cm）、未知の code の wire、同じ単位を送った request、長さの商品の form で値を変えても送らない。
- compatibility checks: `stock_unit` の omitted / null / value（serde）、今の上書き（単位が空・同じ）の結果が変わらない。
- data safety checks: test は合成の値だけ（商品名・JAN は合成）。
- main wiring/integration checks: bindings の再生成、`buildUpdateProductRequest` → `commands.updateProduct` の型の直結（`product-update-patch-contract.test.ts`）。
- Writer は L3 の前に `cargo check --release` を通す。

L3（Windows の実機。owner の手が戻った後に他の lane の L3 と 1 回の session にまとめる。Draft のまま溜める、repo 外の地図 §7.3）:

| 項目 | 到達経路 | 入力物 | 依存する既知 backlog | 合格（PASS / FAIL） |
|---|---|---|---|---|
| L3-1 商品修正で単位を直す | 商品登録 → 入庫画面 → 商品一覧 → 商品 A の「修正」 → 数量単位の select → 「玉」 → 保存 → 在庫照会 → 入庫の記録詳細（単位を直した後に初めて開く） → 操作ログ | 画面から登録する商品 A（固定の合成の値: JAN `2000000009018`、商品名 `L3 合成の糸`、部門 毛糸（ID 3。`src-tauri/src/db/schema_v1.rs:269` の seed。画面は名前で選ぶ）、売価 500、原価 300、税率 10%、単位「個」、初期在庫 0）。入庫画面で商品 A を数量 12・原価 300 で記録する | 無し | select に個数の単位だけが選べる、select の下の文と在庫の節の説明が読める、保存の後に在庫照会で `12 玉`、入庫の記録詳細の数量が `12` で単位が「玉」、操作ログに `stock_unit` の前後 |
| L3-2 長さの商品は直せない | 商品登録 → 商品一覧 → 商品 B の「修正」 | 画面から登録する商品 B（固定の合成の値: JAN `2000000009025`、商品名 `L3 合成の布`、部門 毛糸（ID 3）、売価 700、原価 400、税率 10%、単位 m、初期在庫 `25`） | 無し | 数量単位が押せず今の語（m）が読める、在庫の節の説明が `現在庫と数量単位は登録後に変更できません。` |
| L3-3 商品 CSV の上書き | 商品一括インポート → CSV を選ぶ → preview（商品 B の行のエラーを確かめる） → 商品 A の行だけを「上書き」 → 取込み → 在庫照会 | L3-1・L3-2 の後の商品 A（単位「玉」）・商品 B と、合成の CSV 1 本（CP932。列は 商品コード・商品名・部門ID・売価・原価・税率・在庫単位・JANコード。商品 A の行 = `2000000009018,L3 合成の糸,3,500,300,10,袋,2000000009018`、商品 B の行 = `2000000009025,L3 合成の布,3,700,400,10,玉,2000000009025`。JANコードの列を必ず入れる〈上書きは JAN を CSV の値で書き、空なら JAN を消す。§4.9 の手順 2 a.〉。Coordinator が上の固定の値で作り、Ready の依頼と同時に渡す） | 無し | 商品 B の行が preview のエラー行で文が `在庫単位は個数の単位どうしでだけ変えられます（登録: m、CSV: 玉）`（エラー行には上書きの選択が出ない）、商品 A の行は上書きを選べ、取込みの後に在庫照会で商品 A の単位が「袋」・在庫は 12 のまま |

`plu_dirty` が変わらないことは L3 にしない（PLU の書出しを済ませた商品の用意が要り、L3 Eligibility の (3) に近い。自動 test の T9・T10 が受け持つ）。種類をまたぐ変更の command からの拒否も UI から到達しないので自動 test（T3）が受け持つ。

## Review Focus

- 3 つの入口（command・preview・commit）が 1 つの判定を通し、どの入口でも種類をまたぐ変更・長さどうしの変更・未知の code が保存されないか。
- 単位の変更で数量・金額・price_history・`plu_dirty` の既存の規則が変わらないか（特に commit の「単位だけの変更」の列の比べ方が 30 BIZ-01-D8 の一覧と一致するか）。
- 操作ログの key `stock_unit` が「変えたときだけ」書かれ、上書きの経路は TX の中で商品ごとに 1 行か。
- 長さの商品の form で `stock_unit` が送られないか（UI と builder の両方）。
- 正本の `proposed・未実装` と今の規則の文が残っていないか（S5 の sweep）、節の所有表の外の節を書き換えていないか。

## Contract Ledger

隣接する契約の sweep: `stock_unit` を読む・書く所を `rg -n 'stock_unit' src-tauri/src --glob '!**/schema_v*.rs'` で当て、単位の変更に影響しうる契約を足した。在庫少の判定（SPEC-UNIT-D10。個数の単位はどれも一般の基準で、個数どうしの変更で基準が変わらない）、集計（SPEC-UNIT-D3。個数の単位は `count_points` に足すので変わらない）、価格の基準数量（SPEC-UNIT-D2。個数は 1）、PLU の書出し（file は単位を持たない、33）、在庫連動の自動提案（create だけ、UI-01b-D6・D22）は、個数どうしの変更で値が変わらないので行にせず、ここで除外の理由を書く。

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| SPEC-UNIT-D2（個数どうしだけを許す） | 10 の単位の変更の段 | S1・S2 の判定 | T1・T2・T3・T6・T7・T8 | L3-1・L3-3 |
| BIZ-01-D8（update の段: field・検査・順序・同じ単位） | 30 の単位と原価の精度の契約 | S1 | T1・T2・T3・T4・T5 | L3-1 |
| BIZ-01-D8（上書きの段: preview・commit・未知の code・`plu_dirty`・log） | 同上 | S2 | T6・T7・T8・T9・T10・T11・T12・T14 | L3-3 |
| BIZ-01-D7（key `stock_unit`、変えたときだけ） | 30 §4.4 | S1 | T1・T4・T5 | L3-1 |
| PRODUCT-PATCH-D1（omitted / null は更新しない） | 30 §4.4 | S1・S4 | T4・F5 | — |
| `plu_dirty` の規則（単位は理由にならない） | 30 BIZ-01-D8・§4.4 手順 4・4b・§4.9 手順 2 d. | S1・S2 | T1・T9・T10 | 非対象（自動 test） |
| 20 §2.3 `ProductUpdates.stock_unit` | 20 §2.3 | S3 | T13 | — |
| UI-01b-D23（select・文・送る判定・説明文・現在庫の語） | 51 | S4 | F1・F2・F3・F4・F6 | L3-1・L3-2 |
| UI-01b-D5・D22 の読取専用の置き換え | 51 | S4・S5 | F2・F3 | L3-2 |
| UI-01b-D11・§7.8 と DSR-05 の実例（disabled は長さの単位の商品だけ） | 51、`design-system/01-decision-rules.md` DSR-05、`02-component-catalog.md` | S4・S5 | F3（`ProductForm.test.tsx` の既存の `cm` の disabled の test と、`pcs` の有効化の test） | L3-1・L3-2 |
| 74 §74.8（key を辞書に入れない） | 74 | 変えない（`OperationLogsPage.tsx` の `KNOWN_KEYS` を触らない） | 非対象（変更が無い） | L3-1（raw key で出る） |
| bindings の生成物 | 30 §4.4、本 packet の Boundary | S4 | AC3・F5 | — |

## Contract Probe

N/A: 外部の前提に依らない。`Option<enum>` の field の specta の生成は既存の `tax_rate: Option<ProductTaxRate>` と同じ形で、生成物の形は AC3 と `product-update-patch-contract.test.ts` が確かめる。

## Data Safety

- commit しないもの: 実店舗の商品 CSV・DB・backup・log。L3-3 の CSV は合成の値で作り、repo に入れない（Coordinator が local で渡す）。
- local-only: L3 の DB（owner の Windows の開発・試験 DB）。
- synthetic-only: 自動 test の商品名・JAN・CSV の行（今の test の `make_import_row`・`make_csv` の型）。

## Implementation Results

- 結果: 商品修正と商品 CSV の上書きで、個数の単位どうしの単位の変更を許した（D-119、共通規則 SPEC-UNIT-D2、30 BIZ-01-D8、51 UI-01b-D23）。種類をまたぐ変更と長さの単位どうしの変更は、商品修正の command・商品 CSV の preview・commit の 3 つの入口が BIZ の 1 つの判定を通して拒む。`ProductUpdateRequest`・`ProductUpdates` に `stock_unit` を足して bindings を再生成し、商品フォームの数量単位の select は個数の単位の商品で個数の単位だけを選べるようにした（長さの単位の商品は読取専用のまま）。単位を変えた記録は操作ログ（`product_update`、key `stock_unit`）に残り、商品 CSV の上書きは単位を変えた商品ごとに TX の中で書く。単位だけの変更では `plu_dirty` を変えない。実装した契約の `proposed・未実装` の注記を正本から外した。migration は無い。
- 既存の保護: 単位の変更で在庫・明細・売価・原価・price_history の値は変わらない（自動 test）。今の frontend が `stock_unit` を送らない request はそのまま受ける（PRODUCT-PATCH-D1）。
- 受容した制約: productUpdate の無効化の範囲の外の画面は、次の再取得まで前の語で出る（UI-01b-D23 の末尾）。操作ログ画面は key `stock_unit` と値の code をそのまま出す（74 §74.8）。
- review・CI・merge（closeout、2026-10-19）: Final Review は broad 1 回で、Codex（Sol）= approve（P0）、Claude 側（Fable 5.1）= approve（P3 2）。PR #176 の merge の後に `origin/main` を単一の merge で取り込んで base を同期し、同期の後の closure（Claude 側 Fable 5.1）= approve（P0）。L3-1〜L3-3 は PASS（base 同期の前の head で 1 回、衝突の解消があったため manual を再利用せず同期の後の head でもう 1 回。repo 外の回答台帳 TD-322・TD-324）。helper の record は review・manual = pass。hosted の `Merge gate` は Ready の後の run で pass で、owner の Ready・merge の承認（TD-325）の後に helper 経由で squash merge した。PR: [#171](https://github.com/higekiri-w/inventory-system-desktop/pull/171)
- 後続: `docs/backlog.md` の「単位の語の直しの lane」の項目に、51 UI-01b-D5 の理由の列の句を次の docs の棚卸しで直す 1 行（Final Review の Claude 側 P3 1）。

## Review Response

### Plan Review round 1（2026-10-17、head `734671ed`）

Claude 側 fresh Opus 5.5（reject、P1 0・P2 2・P3 3）と Codex GPT-6.1 Sol（reject、P1 0・P2 5・P3 2）。重なりは AC6 の自己一致（Codex #4 = Claude #1）と disabled の旧い文（Codex #2 = Claude #3）。全て採った（Coordinator の裁定）。是正の前に相談役（Fable 5.1）に反例探しを頼み、裁定どおり全受理でよいとの結果と、test の穴 3 つを是正に入れた。新しい契約は足さず、言い回しの限定・旧い文の sweep・test の網羅・事実の訂正で直した。

| # | 出所 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|
| R1 | Codex #1（P2） | 採る | 30:18 の「同じ単位なら UPDATE も log も書かない」が request 全体の no-op と読め、§4.4・T5 と衝突する | 省くのは `stock_unit` の SET と detail の key だけ、通常の UPDATE とログ行は今の契約のまま、に限定。Matrix の Contracts Under Test と T5 も揃え、同じ単位と新しい商品名の行を足した |
| R2 | Codex #2（P2）・Claude #3（P3） | 採る | UI-01b-D11・§7.8 の「数量単位の select は disabled を維持」が D23 と食い違い、sweep と所有表から漏れていた | 51 の D11・§7.8、DSR-05 の実例、component catalog の同じ型の例に、本 lane の後の規則を proposed の注記で添えた。D23 は「長さの単位の商品は読取専用（押せず、今の語が読める）」に。所有表・Ledger・S5 に足し、code の comment 5 か所は実装の commit で直す所として S5 に。mockup は Non-scope |
| R3 | Codex #3（P2） | 採る | CSV の上書きでログの INSERT が失敗したときに全部戻ることを確かめる test が無い | T14（既存の `fail_product_update_log` trigger で上書きのログだけを失敗させ、商品・同じ取込みの新規の行・ログが全部戻る）を足した |
| R4 | Codex #4・Claude #1（P2） | 採る | AC6 の 2 本目の `rg` が packet 自身に当たり、0 にならない | 2 本の `rg` に `--glob '!docs/plans/**'` を足し、baseline を測り直した。Matrix の anchor の旧い文は数える範囲を `src/features/products` に絞った |
| R5 | Claude #2（P2）・相談役 | 採る | 上書きの「単位だけ」を 10 列で比べる契約に、T10 が 2 列しか当てていない | T10 を 10 列の表駆動に。JAN の行は「既存に JAN あり・CSV の JAN が空」 |
| R6 | Codex #5（P2） | 採る | 過去の記録の表示を確かめる入庫の記録が fixture に無い | L3 の fixture を「初期在庫 0 で登録 → 入庫画面で記録 → 単位を直す → 入庫の記録詳細を初めて開く」にし、合成の値を固定で書いた。Ordinary Operation も揃えた |
| R7 | Codex #6・Claude #4（P3） | 採る | 長さの行は error に入り上書きの対象にできない。L3 の CSV を一意に作れない | 長さの行は preview の error を確かめ、個数の行だけを上書きにする手順と CSV の行を書いた（JAN の列を入れる。無いと上書きが JAN を消す） |
| R8 | Codex #7・Claude #5（P3） | 採る | 現状の事実の誤記 | test の単位の UPDATE は 5 か所、ProductForm の既存の read-only の test の fixture は `cm`、行番号を直した |
| R9 | 相談役 | 採る | 長さどうしの変更が preview・commit の test に無い／「全行にログ」の mutant が残る／L3 で記録詳細を先に開くと旧い語が出る | T7・T8 に `m`→`cm` の行、T11 の「変えない商品」を商品名を変える行に、L3-1 の開く順を明記。Writer への助言の節を Scope に足した |

### Plan Review round 2（2026-10-18、head `17ae3d11`）

Claude 側 fresh Opus 5.5（approve、P1 0・P2 0・P3 3）と Codex GPT-6.1 Sol（reject、P1 0・P2 2・P3 1）。どちらも Ordinary Operation は成立と判定した。全て採った（Coordinator の裁定）。起草役が `ca7bdcc6` で直した。

| # | 出所 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|
| R10 | Codex #1（P2） | 採る | Failure Modes と T4 が、null の update でログ行が書かれることを失敗としている。§4.4 の手順 6 と本 packet の 30:18・T5 は、変更が無くてもログ行を書き detail を NULL にする | Coordinator が `update_product` の操作ログの記録で、値が違うときだけ key を足しログ行は書くことを確かめた。Failure Modes と T4 を「単位は変わらず、ログ行は足され、detail は NULL」に揃えた |
| R11 | Codex #2（P2） | 採る | T9 が初期 `plu_dirty=0` だけで、維持と 0 への強制を区別できない | T9 を `plu_target=1` の商品の初期 `plu_dirty` 0 と 1 の表駆動にし、両方で前の値を保つ期待にした。Mutation の節にも足した |
| R12 | Codex #3（P3） | 採る | 前文の test の file の数と matcher の件数が現物と違う | 数え直し、command と出力を併記した |
| R13 | Claude #1（P3） | 採る | 操作ログの要約の JSON を字面で書いているが、key の順は serde_json の既定で変わる | 「old が `pcs`・new が `ball` の JSON（key の順は契約にしない）」にし、T1 も JSON として比べる形にした |
| R14 | Claude #2（P3） | 採る | 30 BIZ-01-D7 の行の未実装の注記 2 つのうち、2 つ目が S5 と AC6 に当たらない | S5 に 2 つ目も外すことを足し、AC6 に 3 本目の `rg` を足した |
| R15 | Claude #3（P3） | 採る | L3 の入力物の部門が ID だけで、画面では名前で選ぶ | 「部門 毛糸（ID 3）」にした |

### Plan Review round 3（2026-10-18、head `a3753d16`）

Claude 側 fresh Opus 5.5（approve、P1 0・P2 0・P3 3）と Codex GPT-6.1 Sol（reject、P1 0・P2 1・P3 2）。どちらも Ordinary Operation は成立と判定した。全て採った（Coordinator の裁定）。round の天井で P2 が残ったので owner に諮り、「直して、追加の round なしで plan-approved」と決まった（repo 外の回答台帳 TD-315）。起草役が `9cb6ac2c` で直した。

| # | 出所 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|
| R16 | Codex #1（P2） | 採る | S5 と節の所有表が editable / read-only の列挙の直し先を `51-ui-product-form.md` §7.4 としているが、その句は §7.5 の Edit の手順 2・3 にある | S5 の参照と所有表を §7.5 の Edit の手順 2・3 に直し、その句を S5 の sweep に入れた |
| R17 | Codex #2（P3） | 採る | AC3 の生成の後の cwd が書かれていない | cargo の実行を subshell に閉じ、後の検査は repo の root で行うと書いた（S4 も同じ形） |
| R18 | Codex #3（P3） | 採る | 「各段で test を通して次へ」と S1〜S3 の型の依存が合わない | S1〜S3 を 1 つの compile の単位にした |
| R19 | Claude #1（P3） | 採る | Writer への助言の「単位だけの変更」の定義に、単位が実際に変わる前提が無い | 定義の頭に「`updates.stock_unit` が `Some` で、かつ」を足した |
| R20 | Claude #2（P3） | 採る | oracle の anchor の文の production の件数を単数に決め打ちしている | 複数の literal を許す形（定数にまとめるなら単数）にした |
| R21 | Claude #3（P3） | 採る | 改名する test の comment が直す一覧に無い | 一覧に `product_service.rs:3713` を足した |

Closeout（2026-10-19）: Final Review は broad 1 回で、Codex（Sol）= approve（P0）、Claude 側（Fable 5.1）= approve（P3 2）。base 同期の後の closure（Claude 側 Fable 5.1）= approve（P0）。是正は無し。P3 2 件の disposition（Coordinator の裁定）: Claude 側 1（`docs/function-design/51-ui-product-form.md:28` の UI-01b-D5 の理由の列に「単位変更は在庫履歴・閾値・POS 連動への影響が大きいため別 Design Phase」が live な文として残る）は follow-up で、`docs/backlog.md` の「単位の語の直しの lane」へ申し送った。Claude 側 2 は本段落に記録する: Matrix の oracle の anchor の数え方「`#[cfg(test)]` より前の範囲で数える」は、`product_service.rs` に `mod tests` より前の `#[cfg(test)]`（failpoint）があるため字面どおりでは 0 件になる。正しくは「`mod tests {` より前の範囲」で数える。あわせて、base 同期で PR #176 が `ProductUpdates.stock_quantity` を外したので、`overwrite_changes_unit_only` は在庫を比べなくなった（上書きは常に `None` を渡していたので、「単位だけの変更」の判定は変わらない）。packet の本文（Scope・Contract Ledger・AC）は当時の契約の記録として書き換えない。Owner Effort Budget の表は closeout の時点の値（上限 9、消費 5）に直した。
