# Plan Packet: 部門 12（宅急便）の初期データの `code_prefix` を正本の NULL に合わせる

wave に属さない単独の lane。owner の決定（repo 外の回答台帳 TD-395・TD-402、2026-10-23）で起票した。起源は docs の小口の lane（PR #192、`docs/archive/plans/2026-10-23-docs-small-batch.md` の S7-1）で、同じ PR が `docs/backlog.md` の「着手対象」に項目「部門 12（宅急便）の初期データの `code_prefix` を正本の NULL に合わせる」を足した。branch は `agent/dept12-code-prefix`（main `eed07d37` から）。新しい decision は無い（D-133 は使わない）。新しい設計判断は [22 §3.3](../../function-design/22-mnt-migration.md) の MNT-03-D21（本 commit）。本 run は起票（packet・Matrix・plan-first の正本の直し）と Draft PR までで、実装は plan-approved の後の別の run。

**規模と手法（1 lane・1 PR。実装は test を先に赤にしてから literal を直す 1 段）**: 2026-10-23 に本 worktree（HEAD `eed07d37`）で数えた（command と出力は下の AC の baseline と Scope の表）。

- 触る設計正本: plan-first（本 commit）で 1 file（22 §3.3 に MNT-03-D21）。実装の PR では生成物 1（`docs/function-design/90-traceability.md`）だけ。
- code: 3 file（`src-tauri/src/db/schema_v1.rs` の 1 行、`src-tauri/src/biz/product_service.rs` の test 1 本の追加、`src-tauri/src/db/migration.rs` の test 1 本の期待の拡張）。画面・command・wire の型・migration の追加は無い。
- 手法: 直す値は 1 つで、経路は 1 本（v1 の初期データ → `departments` → `generate_custom_code` と商品フォームの部門の select）。段に分けず、Matrix の C1・C2 を先に書いて赤を確かめ（M1）、literal を直して緑にする。

## Workflow State

- Phase: archive
- Risk: R3
- Plan Commit: 8e086991e873c0b0254420e6b030abbbae8cf496
- Amendments: none
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（起票は fresh context の Writer。実装の Writer は plan-approved の後に Coordinator が発注する）
- Plan Reviewer: fresh Opus 5.5 + Codex（model は発注時に決める）。座組表どおり
- Final Reviewer: Fable 5.1（Claude 側、R3 以上）+ Codex（GPT-6.1 Sol 既定）。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Review Minimum: 1
- Human Gate: ready,merge
- Branch: agent/dept12-code-prefix

遷移の記録:

1. kickoff → spec-check（2026-10-23、起草役）: Coordinator の発注（TD-395・TD-402）を Scope にし、規模を数えて 1 lane・1 PR と決め、Risk を R3 と記録した（下の Risk）。
2. spec-check → design（2026-10-23、起草役）: 対象の正本は [master-tables](../../db-design/master-tables.md) の「初期データ（全21部門）」、[ARCHITECTURE](../../ARCHITECTURE.md) §4 の宅急便部門の項、[30 §4.3](../../function-design/30-biz-product-service.md) の処理ステップ 2、[51](../../function-design/51-ui-product-form.md) UI-01b-D4 とエラーハンドリング、[22 §3.3](../../function-design/22-mnt-migration.md)、[decision-log](../../decision-log.md) D-123。値の正本（NULL）は既にあるが、「既存の DB を直す migration を足すか」の判断の置き場が正本に無い（下の Design Readiness）。
3. design → plan-draft（2026-10-23、起草役、本 commit）: 上の判断を MNT-03-D21 として 22 §3.3 に置いた（下の「本 commit で変えた正本」）。owner に聞く設計の問いは残っていない（下の「owner の判断事項」は推奨を既定にした 1 件）。plan-gate へは Coordinator が進める。
4. plan-draft → plan-gate（2026-10-23、Coordinator、本 commit）: owner の判断事項（L3 を置くか）は、owner の方針（repo 外の回答台帳 TD-401: しばらく問わずに推奨で進め、後でまとめて見せる）で推奨の (a) 置かないを既定にした。packet と Matrix が揃い commit されている。Plan Review round 1（fresh Opus 5.5 + Codex）に出す。
5. plan-gate → plan-approved（2026-10-23、Coordinator、本 commit）: round 1 は Claude 側 fresh Opus 5.5 = approve（P3 4）、Codex = approve（P3 1）で P1/P2 = 0（下の Review Response）。Plan Commit は両方が監査した `8e086991`。P3 は implementing の run で直す（Plan Commit を動かさない）。
6. plan-approved → implementing（2026-10-23、Coordinator、本 commit）: 実装の Writer を発注する。

本 commit で変えた正本（plan-first）:

- `docs/function-design/22-mnt-migration.md` §3.3: MNT-03-D21（初期データの直しはリリースの前は v1 の literal で行い migration を足さない。理由・棄却案・範囲・再検討の時）。

## Owner Effort Budget

- 介入回数上限: 6（既定）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 6 | 0（2026-10-23 の起票の時点） | 2（Ready・merge。L3 は置かない〈下の「owner の判断事項」の既定〉） | 4 | 6 = 0 + 2 + 4 |

既定値・数え方・上限に届くときの扱いは `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

owner の操作の回数を減らす工夫: Ready と merge の 2 つの decision point は、同じ便の他の lane と 1 回の問い合わせに束ねてよい（数は lane ごとに 2 のまま）。L3 を置かないので、owner の PC での操作は無い。

## Risk

Risk: R3

Reason:
DB の初期データ（`departments` の 1 行）を変え、商品登録の振舞い（宅急便の部門で独自コードを発番できるか）と商品フォームの部門の select の表示が変わる（Risk Tiers の「DB」と「operator workflow」）。値のある DB を移す・消すことは無く（migration を足さない。MNT-03-D21）、R4 の「Destructive data lifecycle」には当たらない。

Rollback notes: PR を revert すると `schema_v1.rs` の literal が `'TK'` に戻る。revert までに作った DB は NULL のまま残るが、違いは宅急便の部門で独自コードを発番できないことだけで、記録の値と意味は変わらない（Z005 の部門別売上は `code_prefix` を読まない。[pos-tables](../../db-design/pos-tables.md) §12e）。戻す migration も DB の操作も要らない。

この変更でどれかの required gate の green / red が変わるか: 変わらない（製品の初期データと test だけで、gate の定義・helper・classifier・CI・workflow の正本を触らない）。

## Goal

Goal Invariant:

### 最小完了条件

- 新しく作る DB（店の PC の初回の導入を含む）で、部門 12（宅急便）の `code_prefix` が SQL の NULL になり、宅急便の部門を選んで JAN を空にした商品登録が、画面でも BIZ でも止まる。ほかの 20 部門の初期データは変わらない。

### 失敗定義

- 新しい DB で宅急便の `code_prefix` が `'TK'` か空文字（`''`。`generate_custom_code` は `None` だけを拒むので `-0001` を発番してしまう）のまま。
- 宅急便以外の部門の初期データ（名前・`z005_name`・`code_prefix`・`next_seq`）が 1 つでも変わる。
- 宅急便の部門で JAN を入れた商品の登録や、Z005 の部門別売上の取込みが変わる。

### 非目的

- 既存の開発・試験 DB の宅急便の値を直すこと（作り直しで揃う。MNT-03-D21・D-123）。
- 宅急便の部門の商品を登録できなくすること、部門の編集の画面を作ること、他の部門の `code_prefix` を見直すこと。
- ARCHITECTURE §4 から決まった項目（宅急便部門の項）を外すこと（他の決まった項目〈IO-04 ほか〉も同じ節に残っており、本 lane の値の直しと別の整理）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 新しい版を入れ、DB の無い PC（店の PC の初回の導入、作り直した後の開発・試験の PC） | 起動し、商品登録の画面で部門の select を開く | 宅急便に「（独自コード可）」が付かない（`ProductForm` の部門の option は `code_prefix` が NULL の部門に何も付けない。51 UI-01b-D4） | 次の行 | なし（v1 は DB の無い起動で当たる。下の Contract Probe の確かめ） |
| 同じ | 宅急便を選び、JAN を空のまま「登録する」 | 画面が「JANコードを入力するか、独自コード発番対象の部門を選択してください」で止め、何も保存しない（51 のエラーハンドリング）。画面を通らずに BIZ に届いても `ValidationFailed`（「この部門は独自コード発番に対応していません」、30 §4.3 の処理ステップ 2）で、商品・`next_seq`・在庫の記録は何も変わらない | JAN を入れて登録し直す | なし |
| 同じ | 宅急便を選び、JAN を入れて登録する | 今と同じく登録できる（JAN の商品は `code_prefix` を使わない） | 完了 | なし |
| 同じ | Z005（部門別売上）を取り込む | 宅急便の行が今と同じく部門別売上に入る（`daily_report_department_lines` は `code_prefix` を読まない。pos-tables §12e） | 完了 | なし |
| 直す前の版で作った開発・試験 DB | 新しい版で起動する | 宅急便は `'TK'` のまま（独自コード可）。業務の記録は変わらない。22 §16 の作り直しの後は 1 行目と同じ | 作り直し（D-123、他の lane の作り直しと同じ手順で、本 lane のための作り直しは求めない） | MNT-03-D21 |

## Scope

対象の code と、読む所・test の呼び方（2026-10-23、`eed07d37`）。`code_prefix` の型・wire の形は変えないので、作る所・読む所は値の行き先の確かめとして挙げる。

| file | 変えること | 根拠 |
|---|---|---|
| `src-tauri/src/db/schema_v1.rs` | `get_initial_schema` の `INSERT INTO departments` の部門 12 の行の `'TK'` を `NULL` にする（列の並びと他の 20 行は変えない） | master-tables の「初期データ（全21部門）」の部門 12 |
| `src-tauri/src/db/migration.rs` | test `test_initial_departments_req903_data` の期待を、ヘア雑貨と毛糸の 2 行の標本から、master-tables の表から写した 21 行（`id`・`name`・`z005_name`・`code_prefix`・`next_seq` = 1）の全部の比較に広げる（Matrix の C1） | 標本の 2 行だけでは部門 12 の食い違いを止められなかった |
| `src-tauri/src/biz/product_service.rs` | test module の FUNC-4.3（`generate_custom_code`）の test 群の後に、test `test_create_product_req101_takkyubin_rejects_custom_code` を足す（Matrix の C2）。本体の code は変えない | 30 §4.3 の処理ステップ 2 |
| `docs/function-design/90-traceability.md` | `cd src-tauri && cargo run --bin generate_traceability` で再生成する（REQ-101 の test が 1 本増える。手で直さない） | Registration / Generation Obligations |

値の行き先（変えない。`rg -n 'code_prefix' src-tauri/src src --glob '!src/lib/bindings.ts'` の 2026-10-23 の出力から）:

- 作る所: `src-tauri/src/db/schema_v1.rs` の初期データだけ。`departments` を書く code は `product_repo` の `increment_next_seq`・`raise_next_seq`（`next_seq` だけ）で、`code_prefix` を書く経路は無い（`rg -n 'UPDATE departments' src-tauri/src` の出力は `product_repo.rs` の 2 つと、test の `next_seq` の UPDATE 2 つ）。後の migration（v2〜v10）は `departments` を作り直さない（`rg -n 'departments' src-tauri/src/db/schema_v*.rs` の v1 の外の hit は test〈v2・v10〉と外部キーの参照〈v4・v8・v10〉だけで、`departments` を書く・作り直す文は無い）。
- 読む所: `src-tauri/src/db/product_repo.rs` の `Department`（`code_prefix: Option<String>`）と部門の 2 つの SELECT、`src-tauri/src/biz/product_service.rs` の `generate_custom_code`、frontend の `src/features/products/components/ProductForm.tsx`（「（独自コード可）」）と `src/features/products/lib/product-form-request.ts`（JAN が空で `code_prefix` が NULL なら止める）。
- test の fixture・呼び方: Rust の test は `init_database` で実の v1 を当てる（`product_service.rs` の `setup_test_db`、`migration.rs` の test）。frontend の test は `makeMockDepartment` の mock で部門を作り、宅急便を使わない。`TK-` の商品コードを持つ fixture は無い（2026-10-23、`8748d781` で `rg -n '\bTK-[0-9]' src-tauri src docs --glob '!docs/archive/**' --glob '!docs/plans/**'` は出力なし、exit 1。`docs/plans/**` を除くのは本 packet と Matrix が `TK-0001` を例に書くため）。demo の seed（`src-tauri/src/seed_demo.rs` の `DEPARTMENT_SEED_PLAN`）は部門 12 を使わない。

既存の部門 12 の商品の扱い:

- 開発・試験 DB に `TK-` の商品があり得るか: 直す前の版で作った DB では、宅急便を選び JAN を空にして登録すれば `TK-0001` などができる（画面が宅急便に「（独自コード可）」を付けていた）。repo の fixture・test には無い（上の `rg`）。
- 扱い: 本 lane は既存の DB を直さない（MNT-03-D21）。直す前に作った DB では宅急便は `'TK'` のままで、`TK-` の商品も発番もそのまま残る。作り直した DB（22 §16）は空から始まるので、古い商品は旧 DB の folder に残り、新しい DB には無い。新しい DB で `TK-` の商品ができるのは商品 CSV の取込み（`商品コード` は CSV の値をそのまま使い、`code_prefix` を見ない）だけで、その商品は残り、止まるのは宅急便の部門での発番だけ。仮に migration で既存の DB を NULL にしても、既存の商品は残り発番だけが止まる（`code_prefix` は商品の行の検査に使われない）。

並走との重なり（`comm -12`、2026-10-23）: 入力は本 lane の予定 path 7（上の表の 4 と、本 packet・Matrix・22）と、各 PR の branch の差分（`git diff --name-only --no-renames origin/main...origin/<branch>`）と packet が名指す path の和（#188 は packet の「予定 path の一覧」148。local に未 push の commit があるので packet の Scope で取った）。

- #188（`agent/stocktake-count-runtime-b`）と 4: `docs/function-design/22-mnt-migration.md`・`docs/function-design/90-traceability.md`・`src-tauri/src/biz/product_service.rs`・`src-tauri/src/db/migration.rs`
- #172（`agent/sd-direct-read-runtime`）と 3: `docs/function-design/22-mnt-migration.md`・`docs/function-design/90-traceability.md`・`src-tauri/src/db/migration.rs`
- #190（`agent/backup-offsite-runtime`）と 2: `docs/function-design/90-traceability.md`・`src-tauri/src/db/migration.rs`
- #194（`agent/backup-offsite-runtime-b`）と 2: `docs/function-design/22-mnt-migration.md`・`docs/function-design/90-traceability.md`
- #191（`agent/msi-fix-1023`）と 1: `docs/function-design/22-mnt-migration.md`
- #193（`agent/stocktake-count-runtime-b2`）と 1: `docs/function-design/90-traceability.md`
- #192（`agent/docs-small-batch-1023`）と 1: `src-tauri/src/db/schema_v1.rs`（#192 は S7-1 の文で名指すだけで、編集しない）

節の所有表:

| file | 他の lane の節 | 本 lane の節 | 扱い |
|---|---|---|---|
| `docs/function-design/22-mnt-migration.md` | #188: §3.1 の module の一覧と §20。#172: §17。#191・#194: §16 の作り直しの手順の文 | §3.3（MNT-03-D21） | 節が別 |
| `src-tauri/src/db/migration.rs` | #188・#172: `migrations()` の登録と版の期待の test。#190: `read_current_version_without_ddl` の公開 | test `test_initial_departments_req903_data` だけ | 関数が別 |
| `src-tauri/src/biz/product_service.rs` | #188: 共有 JAN の検査（create・update・一括 import の本体と test） | test module の FUNC-4.3 の test 群の後に 1 本（module の末尾に足さない） | 関数が別 |
| `docs/function-design/90-traceability.md` | #188・#172・#190・#193・#194: 生成物 | 生成物 | 衝突は手で解かず、取り込んだ後に再生成する（D-098） |
| `src-tauri/src/db/schema_v1.rs` | #188・#192: 引くだけ | 部門 12 の行 | 重ならない |
| `docs/backlog.md` | #192: 本 lane の起源の項目を足す。#172・#190: 別の項目 | 触らない | 本 lane の merge の後の closeout で起源の項目を外す（#192 が先に merge していれば。後なら #192 の closeout で外す） |

新しい decision の番号・migration の番号は使わない（D-133 と §23 の予約は不要になった。MNT-03-D21 だけを 22 §3.3 で使う）。

継ぎ足さない（D-127）: この変更で不要になる記述・条件・code は無いか。

- 正本: master-tables と ARCHITECTURE は既に NULL で、直す文は無い。backlog の起源の項目は上の所有表のとおり closeout で外す。
- code: `schema_v1.rs` の `'TK'` が不要になる唯一の値。`generate_custom_code` の NULL の分岐と frontend の NULL の扱いは NULL の他の 10 部門（毛糸ほか）が使うので残る。
- test: `test_initial_departments_req903_data` の 2 行の標本は 21 行の比較に置き換わる（足すのでなく置き換える）。

## owner の判断事項

1 件。推奨を既定にして書き進めた（TD-401）。

- **L3（Windows の実機の目視）を置くか**: (a) 置かない（推奨・既定）。(b) 置く: 作り直した DB の商品登録の画面で、宅急便に「（独自コード可）」が付かず、JAN を空にした登録が止まることを見る。推奨の理由: 変わる表示は `code_prefix` の値から決まり、表示の code は変わらない。値は Rust の test（実の v1、C1・C2）、NULL の部門の表示と登録の停止は既存の frontend の test（Matrix の C3）が確かめ、Windows の実機でしか見えない所が無い（`docs/DEV_WORKFLOW.md` の L3 Eligibility の (1) に当たらない）。(b) にすると owner の PC に作り直した DB が要り（直す前の DB では `'TK'` のまま）、介入が 1 つ増える。(b) を選ぶなら、他の lane の L3 で作り直した DB を使う日に 1 項目として束ね、作り直しを本 lane のために増やさない。答えで変わる所: Workflow State の Human Gate（(b) なら `ready,merge,manual`）、Owner Effort Budget の見込み（(b) なら 3）、Test Plan の L3 の項、Review Focus の L3 の観点。

migration を足すかは技術判断として決めた（MNT-03-D21。owner に諮らない）。

## Non-scope

- 既存の開発・試験 DB を直す migration（MNT-03-D21 の棄却案）。
- 部門の追加・編集の画面、他の部門の `code_prefix` の見直し、宅急便の部門の商品の登録の禁止。
- 商品 CSV の取込みで `商品コード` の接頭辞を部門の `code_prefix` と照らすこと（今も照らさない。変えない）。
- ARCHITECTURE §4 の決まった項目の整理、22 §3.3 の「18テーブル」の数の古さ（本 lane の値の直しと別）。

## Acceptance Criteria

- AC-1（literal）: `rg -n "'TK'" src-tauri/src/db/schema_v1.rs` が何も出さない（exit 1）。baseline（2026-10-23、`eed07d37`）は 1 行: `278:(12, '宅急便',         '宅急便',         'TK', 1, '2026-04-03T00:00:00'),`。あわせて `rg -c "\(12, '宅急便', +'宅急便', +NULL," src-tauri/src/db/schema_v1.rs` が `1`（baseline は出力なし、exit 1）。
- AC-2（初期データの全行）: `cd src-tauri && cargo test --lib test_initial_departments_req903_data` が PASS し、test は master-tables の表から写した 21 行と DB の 21 行を `id` 順に比べる（Matrix の C1）。
- AC-3（発番の停止）: `cd src-tauri && cargo test --lib test_create_product_req101_takkyubin_rejects_custom_code` が PASS（Matrix の C2: `ValidationFailed` の message が「この部門は独自コード発番に対応していません」、`products` の行 0、部門 12 の `next_seq` が 1、`inventory_movements` の行 0）。
- AC-4（感度）: Matrix の M1（部門 12 を `'TK'` に戻す）と M2（`''` にする）を `schema_v1.rs` に注入すると C1・C2 が両方 red、M3（部門 2 の `'HZ'` を NULL にする）で C1 が red になり、戻すと green。結果は PR の Validation に書く（tracked に test の数を書かない）。
- AC-5（gate）: `cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test` が exit 0。`cd src-tauri && cargo run --bin generate_traceability -- --check` が exit 0（再生成の後）。
- AC-6（docs）: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-10-23-dept12-code-prefix.md` が exit 0。`rg -c 'MNT-03-D21' docs/function-design/22-mnt-migration.md` が `1`（baseline は出力なし、exit 1。本 commit で 1）。
- AC-7（範囲）: `git diff --name-only origin/main...HEAD` が Scope の 4 file・本 packet・Matrix・22 の 7 path の内に収まる。

## Design Readiness

設計の完了条件は `docs/DEV_WORKFLOW.md` Design Phase Rules の Design completion criteria。当たらない箇条は 1 行の理由で閉じる。

- 引用する設計正本（節まで）: [master-tables](../../db-design/master-tables.md) §2 departments の「初期データ（全21部門）」と `code_prefix` の列の説明（NULL なら独自コード発番対象外）、[ARCHITECTURE](../../ARCHITECTURE.md) §4 の宅急便部門の項（在庫管理対象外、`code_prefix=NULL`）、[30 §4.3](../../function-design/30-biz-product-service.md) `generate_custom_code` の処理ステップ 2、[51](../../function-design/51-ui-product-form.md) UI-01b-D4 と「エラーハンドリング」の JAN 空 + `code_prefix` NULL の文、[pos-tables](../../db-design/pos-tables.md) §12e、[22 §3.3](../../function-design/22-mnt-migration.md) MNT-03-D21、[decision-log](../../decision-log.md) D-123 (1)・(4)・(5)。
- 必要な設計成果物: 「New or changed table, column, index, migration, … persistence behavior」の行 = updated in this PR（22 §3.3 の MNT-03-D21。migration を足さない判断と、直す前の DB との値の違い）。BIZ・CMD・UI の行は existing sufficient（振舞いの契約は 30 §4.3・51 が既に NULL の部門を定め、本 lane は値を正本に合わせるだけ）。
- plan にしかない durable な判断の昇格先: MNT-03-D21（22 §3.3）。decision-log に足さない理由: D-123 の (1)・(4)・(5) の当てはめで、新しい横断の決定ではない。
- 前提・制約と、延期した design gap の follow-up: 直す前の版で作った開発・試験 DB は `'TK'` のまま残る（作り直しで揃う。D-123）。延期した gap は無い。
- 絶対保証の自己点検: 「新しい DB で宅急便は発番できない」の例外は、(i) 直す前の版で作った DB（MNT-03-D21 の範囲の文）、(ii) legacy の DB の取込み（`migrate_legacy_db` は旧 DB の file をそのまま移すので (i) と同じ）、(iii) 復元（backup は直す前の DB の写しなら `'TK'`。71 の復元は file を差し替えるだけで初期データを当て直さない）。どれも「直す前に作った DB」に含まれ、作り直しで揃う。
- 判定: ready。値・振舞い・判断の置き場が正本にあり、Matrix は正本から導ける。

## Registration / Generation Obligations

| 変更対象 | 登録・生成義務 |
|---|---|
| REQ / coverage の追加・変更・削除 | test 名・comment に REQ-101（C2）・REQ-903（C1）を付ける。`cd src-tauri && cargo run --bin generate_traceability` で `docs/function-design/90-traceability.md` を再生成し、`-- --check` を AC-5 に含める |

ほかの行（Tauri command・function-design doc 新設・source doc 新設・route・operator 画面）は該当なし。

## Impact Review Lenses

not applicable: 起点は docs の点検で見つけた正本と code の食い違い（PR #192 の S7-1）で、実機・外部の tool・POS・CSV の形式・現場の業務の発見ではなく、正本の前提も変えない（正本の NULL に code を合わせる）。

## Boundary / Wire Contract

DB-backed compatibility に当たる（初期データの値）。型・wire の形は変えない。

- producer: `src-tauri/src/db/schema_v1.rs` の `get_initial_schema`（v1、DB の無い起動で 1 回だけ当たる）
- consumer: `product_repo` の部門の SELECT → `generate_custom_code`、command `list_departments`（`src-tauri/src/cmd/product_cmd.rs`）→ frontend の `Department`（`code_prefix: string | null`）→ `ProductForm`・`product-form-request`
- wire type: `code_prefix: string | null`（`src/lib/bindings.ts`。変えない）。部門 12 の値が `"TK"` から `null` になる
- internal type: `Option<String>`（`product_repo` の `Department`）。SQL の NULL が `None` になる。空文字は `Some("")` になり発番を通すので使わない
- precision/range: 該当なし（文字列の有無だけ。`next_seq` は 1 のまま）
- round-trip path: v1 の INSERT → `departments` → SELECT → `None` → `ValidationFailed` / wire の `null` → 「（独自コード可）」なし
- invalid input: 該当なし（利用者が `code_prefix` を書く経路は無い）
- compatibility: 直す前に作った DB は `'TK'` のまま（MNT-03-D21）。frontend と BIZ は両方の値を今と同じく扱う。型・wire の形を変えないので、作る所・読む所・fixture の全件は Scope の「値の行き先」に置いた

## Test Plan

Test Design Matrix: [2026-10-23-dept12-code-prefix](test-matrices/2026-10-23-dept12-code-prefix.md)

- targeted tests: Matrix の C1（`test_initial_departments_req903_data` を 21 行の比較に）、C2（`test_create_product_req101_takkyubin_rejects_custom_code`）。順は C1・C2 を書いて M1 の状態（今の `'TK'`）で red を確かめてから literal を直す。
- negative tests: C2（JAN が空の宅急便の登録が何も書かずに止まる）、M2（空文字の注入で C1・C2 が red）。
- compatibility checks: C3（既存の frontend の test が NULL の部門の表示と停止を確かめている。変えない）、C4（既存の `generate_custom_code` の test を変えずに PASS）。
- data safety checks: 合成の DB（`tempfile` と `init_database`）だけ。実の DB・店の値を読まない。
- main wiring/integration checks: C1・C2 は `init_database` で実の v1 を当てた DB で走り、mock の部門を使わない。

L3 は置かない（上の「owner の判断事項」の既定）。Human Visual Confirmation For Screen Changes（`docs/DEV_WORKFLOW.md`）も当たらない: 画面の code・route・文言を変えず、変わるのは部門 12 の option の「（独自コード可）」の有無というデータの値だけで、operator-facing の画面を作る・実質的に変える PR ではない（NULL の部門の表示は C3 が守る）。`cargo check --release` は L3 の build の前の Writer の完了条件なので、L3 が無い本 lane では要らない。

## Review Focus

- MNT-03-D21: migration を足さない判断が D-123 の (1)・(4)・(5) と合うか。直す前に作った DB の値の違い（`'TK'`）の害が開発・試験 DB の発番だけに収まるか。
- C1 の期待が master-tables の表から写され、`schema_v1.rs` から写されていないか（anti-tautology）。21 行の全部の列を比べるか。
- C2 が発番の前に何も書かないこと（`next_seq`・`products`・`inventory_movements`）を見るか。message を 30 §4.3 の文と比べるか。
- NULL と空文字の区別（Goal の失敗定義）を C1・C2 が止めるか（M2）。
- この変更で不要になる記述・条件・code は無いか（Scope の末尾の答え）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| 初期データの部門 12 の `code_prefix` = NULL | master-tables §2「初期データ（全21部門）」の id 12、ARCHITECTURE §4 の宅急便部門の項 | `schema_v1.rs` の部門 12 の行 | `test_initial_departments_req903_data`（C1）、`test_create_product_req101_takkyubin_rejects_custom_code`（C2） | L3 なし（owner の判断事項の既定） |
| 初期データの他の 20 部門（adjacent-contract sweep で足した） | master-tables §2「初期データ（全21部門）」 | 変えない | `test_initial_departments_req903_data`（C1） | — |
| BIZ-01 FUNC-4.3 処理ステップ 2（NULL の部門は `ValidationFailed`、書込みの前） | 30 §4.3 | 変えない（本体の code） | `test_create_product_req101_takkyubin_rejects_custom_code`（C2）、`test_generate_custom_code_req101_no_prefix`（C4、毛糸） | — |
| UI-01b-D4・51 のエラーハンドリング（JAN 空 + NULL の部門は保存しない、NULL の部門に「（独自コード可）」を付けない） | 51 | 変えない | `src/features/products/ProductFormPage.test.tsx` の「blocks create submit when JAN is blank and department has no code_prefix」、`src/features/products/lib/product-form-request.test.ts` の「allows JAN blank only when selected department has code_prefix」（C3） | — |
| MNT-03-D21（初期データの直しはリリースの前は v1 の literal、migration を足さない） | 22 §3.3（本 commit） | `schema_v1.rs` だけを直し、`migrations()` に版を足さない | 該当なし（判断の契約。AC-7 で migration の file が増えないことを見る） | — |
| D-123 (1)・(4)・(5) | decision-log D-123 | 引くだけ | — | 非対象（MNT-03-D21 の根拠） |
| BIZ-01-D6（接頭辞ごとの最大 + 1） | 30 §4.3 処理ステップ 3、master-tables の独自コードルール | 変えない | — | 非対象（NULL の部門はステップ 2 で止まり届かない） |
| Z005 の部門別売上（`daily_report_department_lines`） | pos-tables §12e | 変えない | — | 非対象（`code_prefix` を読まない。Scope の「値の行き先」の `rg`） |

## Contract Probe

N/A: 外部の library・OS・hardware の前提に依らない。repo の中の前提は実読で確かめた（2026-10-23、`eed07d37`）: DB の無い起動は `src-tauri/src/db/mod.rs` の `init_database` → `configure_database` → `migrate` で、`src-tauri/src/db/migration.rs` の `migrations()` の版 1 が `schema_v1::get_initial_schema()`。v2〜v10 は `departments` を作り直さず行も書かない（Scope の「値の行き先」の `rg`）。

## Data Safety

- commit しない: 実の DB・backup・店の部門の値・L3 の DB の写し。
- local だけ: 無い（本 lane は owner の PC の DB に触らない）。
- 合成だけ: C1・C2 の DB は `tempfile` の一時の folder に `init_database` で作る。

## Implementation Results

2026-10-23、実装の Writer（Opus 5.5 subagent）。

- 実装: `schema_v1.rs` の部門 12 の `'TK'` を `NULL` にした。C1 `test_initial_departments_req903_data`（`migration.rs`、2 行の標本を master-tables §2 から写した 21 行の全列の比較に置き換え）、C2 `test_create_product_req101_takkyubin_rejects_custom_code`（`product_service.rs`、FUNC-4.3 の最初の test 群の後）。本体の code・migration の版は変えていない。
- 赤の確かめ: C1・C2 を書いて literal を直す前（M1 と同じ状態）に走らせ、両方 FAILED（C2 は `Ok(... product_code: "TK-0001" ...)`、C1 は部門 12 が `Some("TK")`）。literal を直して両方 ok。
- 感度（AC-4。clean な tree に `schema_v1.rs` の literal を 1 つずつ注入し、`git checkout` で戻して `git diff --exit-code` が 0）: M1（部門 12 を `'TK'`）= C1・C2 FAILED。M2（部門 12 を `''`）= C1・C2 FAILED。M3（部門 2 の `'HZ'` を NULL）= C1 FAILED（C4 の `test_generate_custom_code_req101_normal` も FAILED、C2 は ok）。戻した後は 3 つとも ok。
- 実装の run で変えた正本: 22 §3.3 MNT-03-D21 の範囲に 1 文（直す前に作った DB を見分けて止まる検査は無く、残す DB の有無は初回の導入の前に確かめる。round 1 の P3 F1）。
- 申し送り: 初回の導入の前に残す DB が無いことを確かめる手順は、MSI の lane（PR #191）の runbook が所有する（本 lane では書かない。MNT-03-D21 の範囲の文）。
- 生成物: `cargo run --bin generate_traceability` で `90-traceability.md` を再生成（REQ-101 の行に C2 が加わる 1 行の差分）、別 commit。

| AC | command | 結果 |
|---|---|---|
| AC-1 | `rg -n "'TK'" src-tauri/src/db/schema_v1.rs` / `rg -c "\(12, '宅急便', +'宅急便', +NULL," src-tauri/src/db/schema_v1.rs` | 出力なし exit 1 / `1` exit 0。PASS |
| AC-2 | `cd src-tauri && cargo test --lib test_initial_departments_req903_data` | exit 0。PASS |
| AC-3 | `cd src-tauri && cargo test --lib test_create_product_req101_takkyubin_rejects_custom_code` | exit 0。PASS |
| AC-4 | 上の感度の M1〜M3 | 期待どおり red、戻して green。PASS |
| AC-5 | `cargo fmt --check` / `cargo clippy --all-targets --all-features -- -D warnings` / `cargo test` / `cargo run --bin generate_traceability -- --check` | 4 つとも exit 0（`cargo test` は全 suite で failed 0、traceability は ERROR 0 / WARN 0）。PASS |
| AC-6 | `bash scripts/doc-consistency-check.sh` / `--target plan docs/plans/2026-10-23-dept12-code-prefix.md` / `rg -c 'MNT-03-D21' docs/function-design/22-mnt-migration.md` | exit 0（ERROR なし、WARN は 22 の原価の節〈§3.3 の外〉の既存の 12）/ exit 0（ERROR なし、WARN 1 は `## Review Response` の round 1 の段落の M3 マーカー）/ `1`。PASS |
| AC-7 | `git diff --name-only origin/main...HEAD` | 22・90・本 packet・Matrix・`product_service.rs`・`migration.rs`・`schema_v1.rs` の 7 path。PASS |

追加の検査: `cargo test --test design_compliance_test` exit 0、`bash scripts/check-workflow-git.sh` exit 0。

round 1 の P3 F4（`docs/ARCHITECTURE.md` §4 の宅急便部門の項を外す）は本 run で行っていない: 本 packet の非目的・Non-scope（ARCHITECTURE §4 の決まった項目の整理）と AC-7 の 7 path に当たるため。扱いは Coordinator が決める。

closeout（2026-10-24）:

- 結果: 新しく作る DB で部門 12（宅急便）の `code_prefix` が正本どおり NULL になり、宅急便の部門で JAN を空にした商品登録が BIZ で止まり何も書かないことと、21 部門の初期データの全列を master-tables の表と比べる test で固定した。migration は足さず（MNT-03-D21）、直す前に作った開発・試験 DB は作り直しで揃う（D-123）。画面・command・wire の型は変えない。
- review・CI・merge: Plan Review round 1（fresh Opus 5.5 と Codex）は両者 approve。Final Review の broad は Claude 側 Fable 5（owner の決定 2026-10-24 で当面 Fable 5.1 でなく 5）が approve（P3 1）。PR #192 の merge の後に `origin/main` を単段 merge で同期し（PR の差分は同期の前と同じ）、同期の closure（fresh Opus 5.5）が approve（相互作用なし）。hosted CI は緑で、helper 経由で squash merge した。PR: [#195](https://github.com/kosei-w90607/inventory-system-desktop/pull/195)
- 後続: round 1 の P3 F4（`docs/ARCHITECTURE.md` §4 の宅急便部門の項の整理）は `docs/backlog.md` に 1 項目として置いた。初回の導入の前に残す DB が無いことを確かめる手順は MSI の lane（PR #191）の runbook へ申し送り済み。backlog の起源の項目（「部門 12（宅急便）の初期データの `code_prefix` を正本の NULL に合わせる」）は closeout で完了として外した。

## Review Response

先行 round の結果・評価（判定・件数・採否・reviewer の意見）はこの節にだけ書く。前半の節と遷移記録には「round N の是正（Review Response 参照）」だけを書く（独立 review は `## Review Response` より前だけを読む）。

round 1（`8e086991`）: Claude 側 fresh Opus 5.5 = approve（P1 0 / P2 0 / P3 4）、Codex = approve（P1 0 / P2 0 / P3 1）。P3: F1（MNT-03-D21 に止まる検査が無いことと、残す DB の有無を初回の導入の前に確かめることを 1 文。runbook の手順は MSI の lane〈PR #191〉か backlog へ）、F2（Human Visual Confirmation が当たらない理由を 1 行）、F3 と Codex #1（「v1 の外の hit は test の 2 つ」を「test と外部キーの参照だけで、departments を書く・作り直す文は無い」に、`TK-` の rg の時点）、F4（`docs/ARCHITECTURE.md` の「未確定事項」の宅急便の項を本 PR で外すか backlog へ）。implementing の run で直す。

Final Review と closeout（2026-10-24）: Final Review（Minimum 1）の broad は Claude 側 Fable 5（owner の決定 2026-10-24 で当面 Fable 5.1 でなく 5。Workflow State の Final Reviewer の行は起票の時点の座組の記録として残す）= approve（P3 1）。PR #192 が先に merge したので `origin/main` を単段 merge で同期し（PR の差分は同期の前と byte で一致）、同期の closure（fresh Opus 5.5）= approve（相互作用なし）。round 1 の P3 F4 は Writer が非目的・Non-scope で止め、Coordinator が backlog の 1 項目へ送った（closeout で足した）。hosted CI は緑で、helper 経由で squash merge した（2026-10-24）。packet の本文（Scope・Contract Ledger・AC）は当時の契約の記録として書き換えない。
