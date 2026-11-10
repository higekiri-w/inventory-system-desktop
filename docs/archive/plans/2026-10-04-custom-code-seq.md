# Plan Packet: 独自コードの自動採番が既存の番号と衝突して止まる

2026-10-04 起票。2026-10-04 の並走の lane E（owner 2026-10-04 の lane 選択 TD-104: A・D・E を並走、B → C は直列）。起点は `76de30d8`（main）。材料は持ち帰りデータの見落としチェック（2026-10-04）の店のシート・一覧の報告の 4（「CSV で独自コードを入れると自動採番が止まりうる」、推測・未実行）。本 packet の Contract Probe で実際に止まり続けることを確かめた。実装は plan-approved の後に Writer（別 context）が行う。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

実装後の状態はPR native state / 専用record / CIが所有し、trackedに書かない。

- Phase: archive
- Risk: R3
- Plan Commit: 90990977b33aecf717a8bd88a16f76db1821784a
- Amendments: none
- Coordinator: Opus 5.5（Claude Code main session、effort high）
- Writer: Opus 5.5 subagent（`subagent_type: writer`、worktree は本 lane の隔離 worktree、branch `agent/custom-code-seq`）
- Plan Reviewer: Opus 5.5（fresh `subagent_type: reviewer`、Writer と別 context）+ Codex（`.local/codex-orders/MODEL-SELECTION.md` の表で round ごとに選ぶ。round 1 = GPT-6.1 Sol、round 2 = GPT-6 Astra。repo 外）
- Final Reviewer: Fable 5.1（fresh subagent）+ Codex（GPT-6.1 Sol）。互いに独立で、後の reviewer に先の結果を見せない
- Final Review Minimum: 1
- Human Gate: ready,merge
- Branch: agent/custom-code-seq

Final Review Minimum は規則どおり 1（R4 でない。予定の file に `scripts/ci/classify-changes.sh` が workflow と判定する path が無い。どの required gate の green / red も変えない）。Human Gate に manual を足さない: 画面・文言・command・DTO を変えず、変わるのは BIZ が返すコードだけで、自動 test で確かめられる（L3 Eligibility の 1 つ目〈Windows / Tauri の実機でしか見えない〉に当たらない）。

遷移記録（append-only）:
- kickoff → spec-check → design → plan-draft（本 commit、plan-first、2026-10-04、起草役）: Risk R3 を記録。Contract Probe で不具合の再現を確かめた後、設計正本が足りない（30-biz §4.3 の step 5 が既存の番号で止まる設計そのもの）ため design を経た。30-biz §4.2 / §4.3 / §4.9、20-io §2.3 / §2.4、`docs/db-design/master-tables.md` の departments、`docs/decision-log.md` の D-106 を同じ commit で更新し、未解決の設計の問いは無い。packet と Test Design Matrix を同じ commit に置く。decision-log の番号は D-106（Coordinator の割当て）。
- plan-draft → plan-gate（2026-10-04、Coordinator）: packet と Test Design Matrix は plan-first commit `018f5197` で揃い、doc check（`--target plan` と full）は Coordinator の再実行でも exit 0。Plan Reviewer の Codex を表の当てはめ（R3 の初回）で GPT-6.1 Sol に直した。
- plan-gate（round 1 の是正、2026-10-04、起草役、本 commit）: Plan Review round 1 は Opus approve（P3 2）・Codex reject（P1 1・P2 3・P3 1）。Coordinator の裁定と owner 決定 TD-108・TD-109 を反映した。Plan Commit は pending のまま。
- plan-gate（round 2 の是正、2026-10-04、Coordinator、本 commit）: round 2 の是正（Review Response 参照）。P3 のみ。Plan Commit は pending のまま。
- plan-gate → plan-approved（2026-10-04、Coordinator、本 commit）: Plan Review round 2（対象 `09418db6`）で 2 本とも P1 / P2 = 0。round 2 の後の是正（`90990977`、P3 のみ）を Coordinator が現物で確かめた（doc check の `--target plan` と full が ERROR 0、helper の `parse_packet` が通る、pre-push の PK5 と traceability が OK）。是正の独立の再確認は Final Review が兼ねる。owner の plan-approved の判断（2026-10-04）。Plan Commit は `90990977`。
- plan-approved → implementing（2026-10-04、Coordinator、state-only、本 commit）: Writer（Opus 5.5 subagent、`subagent_type: writer`）へ実装を発注する。発注の直前に AC1〜AC4・AC7 の command を逐語で再実行し、上の baseline と一致した（AC1・AC2・AC3 exit 0 で新しい test 名は 0 行、AC4 `traceability check: OK`、AC7 は 0 行と 4 file）。

## Owner Effort Budget

- 介入回数上限: 6（既定 6）
- 実働時間上限: 30分（既定 30 分。画面の確認が無く、Ready と merge の判断だけ）
- Plan Review round 天井: 3（既定 3）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 6 | 4（起票の判断 TD-104〈本 lane を含む lane 選択〉、Plan Review round 1 の後の番号の振り方 TD-108〈抜けを埋めない〉・TD-109〈9999 を超えたら 5 桁以上〉、plan-approved 1〈2026-10-04、予備を使った〉） | 2（Ready、merge） | 0 | 6 = 4 + 2 + 0 |

既定値・数え方・上限に届くときの扱いは `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。Codex の起動（relay）は数えない。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
DB に書く値（`departments.next_seq`）の決め方と、operator の通常の操作（JAN の無い商品の登録）の結果を変える。DEV_WORKFLOW Risk Tiers の R3 の「DB」「operator workflow」に当たる。R2 は runtime の契約を変えない変更なので当たらない。商品コードの形（`{接頭辞}-{4 桁以上}`）・既存の商品コードと next_seq の値・schema・Tauri の command と DTO・CSV の形は変えず、既存データの書換えも migration も無いため R4 ではない（破壊的な data lifecycle・実データの露出に当たらない）。

## Goal

Goal Invariant:

### 最小完了条件

- 店が商品 CSV で `{接頭辞}-NNNN` の形の独自コードの商品を取り込んだ後も、商品登録の画面で JAN を空欄にして同じ接頭辞の部門の商品を登録でき、既存のどのコードとも重ならない独自コードが振られる。続けて登録しても毎回登録できる。

### 失敗定義

- 取り込んだ番号と同じコードを振ろうとして登録が「この商品コードは既に使用されています」で止まる（1 回でも、続けてでも）。
- 取り込んだ番号の間の抜けた番号を新しい商品に振る（使わなくなった番号を別の商品に回す。owner 決定 TD-108、BIZ-01-D6）。
- 同じ接頭辞の既存の独自コードの番号の最大 + 1 が next_seq 以下の部門で、振られる番号が今までと変わる。
- 登録が途中で失敗したのに next_seq の引上げだけが残る（TX の外へ漏れる）。
- 既存の商品コード・next_seq の値を一括で書き換える、または schema・command・DTO・`src/lib/bindings.ts` に diff が出る。

### 非目的

- 商品 CSV の取込み（preview / commit）の振舞い・検証・文言を変えること（next_seq を進めることも含む。BIZ-01-D6 の却下 (b)）。
- 独自コードの形（接頭辞・桁数）を変えること、部門の接頭辞や next_seq を画面で直せるようにすること。
- Excel から初期投入する経路の設計（材料の報告の別の項目。本 lane の範囲外）。
- 同時に 2 つの登録が走る競合への備え（単一の端末・単一の起動〈single-instance〉で、発番は 1 つの TX の中）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 部門「ヘア雑貨」（接頭辞 HZ）に、商品 CSV で `HZ-0001`・`HZ-0002` を取り込んだ直後。next_seq は 1 のまま（取込みは進めない） | 商品登録で JAN を空欄、部門「ヘア雑貨」、必須項目を入れて保存 | `HZ-0003` で登録され、保存の結果にコードが出る。今は `HZ-0001` の重複で毎回止まる | 続けて次の商品を登録すると `HZ-0004` | Probe P1・P2（今の失敗の再現）。T1 |
| 取り込んだコードが `HZ-0001` と `HZ-0005`（間が抜けている） | 同じく登録 | `HZ-0006`。抜けた `HZ-0002`〜`HZ-0004` は使わない | — | BIZ-01-D6。T2 |
| `HZ-0007` を別の部門（その他小物）の部門 ID で取り込んだ | 「ヘア雑貨」で登録 | `HZ-0008`（コードは部門をまたいで一意なので飛ばす） | — | T5 |
| 同じ接頭辞の既存の独自コードの番号の最大 + 1 が next_seq 以下の部門（その他小物、接頭辞 KM。`KM-` のコードが無い） | 登録 | 振られる番号は今までと変わらない（next_seq の番号、`KM-0001` から） | — | Probe P2 の最後の行。既存 test `test_generate_custom_code_req101_normal` |
| `HZ-9999` を取り込んだ | 「ヘア雑貨」で登録 | `HZ-10000`（9999 を超えたら 5 桁以上で続ける。owner 決定 TD-109） | — | T8 |
| 取り込んだ `HZ-0003` がある部門で、登録の途中で DB の失敗が起きた | 画面に失敗が出る。もう一度保存する | next_seq の引上げも戻っており、再試行で `HZ-0004` が振られる | — | T6 |
| 上のどれかの後、アプリを閉じて翌日に起動 | 登録 | 同じ結果（毎回 DB の商品から数えるので、起動をまたぐ状態を持たない） | — | 状態は DB だけ（BIZ-01-D6） |

本 lane で Goal の運用は閉じる。実データに `{接頭辞}-NNNN` の形のコードがあるかは未確認（Contract Probe P3）だが、Goal は「取り込んだ場合に止まらない」ことで、実データの有無に依らない。

## 起票時実測（2026-10-04、base `76de30d8`）

| # | 対象 | command | 出力 |
|---|---|---|---|
| 1 | 発番の重複の判定の所在 | `rg -n 'DuplicateProductCode\(code\)' src-tauri/src/biz/product_service.rs` | 2 行: `:170`（`generate_custom_code` の旧 step 5）と `:198`（`create_product` の JAN の重複の判定） |
| 2 | next_seq を書く本番の経路 | `rg -n 'increment_next_seq\(' src-tauri/src --glob '!**/tests/**'` | 本番は `product_service.rs:165` だけ（他は `product_repo.rs` の定義と test） |
| 3 | 商品を INSERT する本番の経路 | `rg -n 'insert_product\(&tx' src-tauri/src/biz/product_service.rs` | 3 行: `:225`（`create_product`）、`:1348`（`commit_import`）、`:1595`（test）。CSV の取込みは next_seq に触れない（#2） |
| 4 | next_seq を直す画面 | `rg -n 'next_seq' src --glob '!*.test.*' --glob '!src/lib/bindings.ts'` | 2 行で、どちらも test 用の fixture（`src/features/stock-inquiry/lib/test-fixtures.ts:55`、`src/features/products/lib/test-fixtures.ts:42`）。画面の code には無い（`docs/UI_TECH_STACK.md:260` E6 も「現 UI は利用しない」） |
| 5 | 新しい repo 関数の不在 | 表の下の注 | 0 行（exit 1） |

#5 の command（表の外に置く。表の中では `|` を `\|` と書くので、生の文字列を写すと rg が字義の `|` を探して常に 0 行になる。round 1 の是正で直し、本 branch `8ded2dfc`〈code は base と同じ〉で 2026-10-04 に測り直した）: `rg -n 'fn (raise_next_seq|list_product_codes_by_prefix)' src-tauri/src`

## Scope

### S1 発番が既存の番号を飛ばす（BIZ-01-D6）

- `src-tauri/src/biz/product_service.rs` の `generate_custom_code` を 30-biz §4.3 の step 1〜8 にする。step 3 の番号の読み取り（`{接頭辞}-` の後ろが 1 文字以上の ASCII 数字だけで i64 として読めるもの）は同 file の private 関数にしてよい。上限の検査は step 3c（`floor = 最大.checked_add(1)`）と step 3d（`cand = max(next_seq, floor)`、`cand.checked_add(1)`）のとおりで、どちらかが None なら next_seq の SQL 更新（step 4・5）の前に `ValidationFailed("この部門の独自コードの番号を振れません")` を返す（`increment_next_seq` の SQL `next_seq + 1` は i64 を超えると next_seq を REAL にする）。関数の doc comment（今の「4桁ゼロ埋め連番」）も §4.3 に合わせる（4 桁以上、既存の番号を飛ばす）。signature（`&rusqlite::Transaction<'_>`）・step 1 / 2 / 7 と、`create_product` / `commit_import` の他の処理は変えない。
- `src-tauri/src/db/product_repo.rs` に 20-io §2.3 `list_product_codes_by_prefix` と §2.4 `raise_next_seq` を足す（SQL は設計正本の処理ステップのとおり。`LIKE` を使わない）。`increment_next_seq` は変えない。

### S2 test

- Matrix の新しい test（product_service の T1〜T8〈T7 は T7a〜T7c の 3 本〉、product_repo の R1・R2）を足す。test 名と各 test の comment に `REQ-101` を入れる（traceability）。CSV を通す test の bytes は合成で、既存の test（`product_service.rs` の UTF-8 BOM を付ける preview の test）と同じ形にする。
- 既存の `test_create_product_req101_duplicate_key_from_insert`（`product_service.rs:1887`）は、前提（既存の `HZ-0001` があると発番が `DuplicateProductCode` を返す）が BIZ-01-D6 と食い違うため、`test_create_product_req101_skips_directly_inserted_custom_code` に改名し、`HZ-0002` で登録できることを確かめる形に直す。この test は名前と違い INSERT 時の `DuplicateKey` の正規化（`product_service.rs:225-229`）に届いておらず（`:169-171` の重複の判定が先に返す）、直した後も正規化の経路は残す（JAN の競合のための防御。決定的に届く test は今も無い）。理由を test の comment に 1 行書く。
- 既存の `test_create_product_req101_rollback_after_insert`・`test_generate_custom_code_req101_*`・`test_increment_next_seq_req101_*`・`test_create_product_req101_duplicate_jan` は変えずに通す。

### S3 生成物

- `cd src-tauri && cargo run --bin generate_traceability` で `docs/function-design/90-traceability.md` を再生成する（S2 で REQ-101 の test が増えるため。手で編集しない）。他 lane との衝突は merge 後の再生成で解く（D-098）。

予定の file は上の 3 つ（`product_service.rs`・`product_repo.rs`・`90-traceability.md`）だけ。設計正本は本 packet の plan-first commit で更新済みで、実装の PR で設計と実装が食い違ったときは設計正本の方を先に Plan Review の経路で直す。

## Non-scope

- 商品 CSV の取込み（`preview_import` / `commit_import`）のコード。30-biz §4.9 には「next_seq を進めない」を書いたが、振舞いは今と同じ。
- `src-tauri/src/cmd/**`、`src/**`（frontend）、`src/lib/bindings.ts`、schema / migration。
- `DuplicateProductCode` の文言（`cmd/mod.rs:158-160`）。JAN の重複で今までどおり出る。
- 共有 file の所有（下の表）で他 lane が持つ所。

### 共有 file の所有（wave 14 の 4 lane 並走）

| file | 本 lane（E）が触る所 | 他 lane |
|---|---|---|
| `docs/decision-log.md` | 末尾に D-106 の 1 件だけ | A = D-103、B = D-104、D = D-105 をそれぞれ末尾に。merge の順で末尾の衝突を解く |
| `docs/db-design/pos-tables.md` | 触らない | A・B の持ち場 |
| `docs/db-design/master-tables.md` | departments の next_seq の行と設計意図の 1 文、独自コードルール（C-1） | 他 lane は触らない見込み |
| `docs/SCREEN_DESIGN.md` | §4 の独自コードの形の 1 行（:387） | 他 lane は触らない見込み |
| `docs/function-design/30-biz-product-service.md`・`20-io-product-repo.md` | §4.2 のエラーハンドリング・§4.3・§4.9 step 2c、§2.3 / §2.4 の新しい関数 | 他 lane は触らない見込み（A・B・D は POS の取込み・EJ） |
| `docs/function-design/90-traceability.md` | 再生成 | 他 lane も再生成しうる。merge 後の再生成で解く（D-098） |
| `docs/adr/` の stocktake / EJ の ADR、`docs/project-memory.md`、`docs/backlog.md`、`Plans.md`、`docs/PROJECT_HANDOFF.md` | 触らない | どの lane も触らない |

## Acceptance Criteria

baseline は 2026-10-04 に起草役が本 branch（base `76de30d8` と同じ code）で同じ command を逐語で実行した実測。test は全 PASS を求め、本数を AC にしない。

- AC1（Goal、S1・S2）: `cd src-tauri && cargo test --lib biz::product_service` が exit 0 で、Matrix の T1〜T8（T7 は `test_generate_custom_code_req101_rejects_seq_overflow`・`test_generate_custom_code_req101_rejects_overflow_after_max`・`test_generate_custom_code_req101_rejects_overflow_from_next_seq` の 3 本、T8 は `test_generate_custom_code_req101_continues_past_9999`）と改名した `test_create_product_req101_skips_directly_inserted_custom_code` が出力の `test ... ok` の行に出る。baseline: exit 0（`test result: ok.`、新しい test 名は 0 行）。
- AC2（S1・S2）: `cd src-tauri && cargo test --lib db::product_repo` が exit 0 で、R1・R2 が `ok`。baseline: exit 0（R1・R2 の名前は 0 行）。
- AC3（設計と実装の突合）: `cd src-tauri && cargo test --test design_compliance_test` が exit 0（新しい pub 関数 2 つが 20-io に載っている）。baseline: exit 0。
- AC4（S3）: `cd src-tauri && cargo run --bin generate_traceability -- --check` が exit 0（`traceability check: OK`）。baseline: exit 0（`traceability check: OK（ERROR 0 件 / WARN 0 件）`）。
- AC5（Rust の gate）: `cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings` が exit 0。
- AC6（範囲）: `git diff --name-only <Plan Commit>...HEAD` が `src-tauri/src/biz/product_service.rs`・`src-tauri/src/db/product_repo.rs`・`docs/function-design/90-traceability.md` と本 packet・Matrix（Implementation Results / Review Response の追記）だけ。`git diff --stat <Plan Commit>...HEAD -- src/ src-tauri/src/cmd src-tauri/src/db/schema_v1.rs src-tauri/src/db/migration.rs` が空。baseline: 実装前のため対象外。
- AC7（旧前提の除去）: `rg -n '通常は起きないが安全のため' docs/function-design/30-biz-product-service.md` が 0 行（exit 1）。baseline（本 commit の後）: 0 行。`rg -n 'BIZ-01-D6' docs/function-design/30-biz-product-service.md docs/function-design/20-io-product-repo.md docs/db-design/master-tables.md docs/decision-log.md` が 4 file すべてに 1 行以上。baseline（本 commit の後）: 4 file とも当たる。
- AC8（docs）: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan` が exit 0。baseline（本 commit）: 両方 exit 0。

## Design Readiness

- 引用する設計正本（節まで）: `docs/function-design/30-biz-product-service.md` §4.2（create_product、エラーハンドリング）・§4.3（generate_custom_code、BIZ-01-D6）・§4.9（commit_import step 2c）、`docs/function-design/20-io-product-repo.md` §2.3（`list_product_codes_by_prefix`）・§2.4（`increment_next_seq`・`raise_next_seq`）、`docs/db-design/master-tables.md` §2 departments（next_seq）、`docs/decision-log.md` D-106。owner 決定 TD-109 に合わせて直した: `docs/db-design/master-tables.md` 独自コードルール（C-1）、`docs/SCREEN_DESIGN.md` §4 の独自コードの形の 1 行。隣接して読んだが変えない: `docs/function-design/51-ui-product-form.md` UI-01b-D4（JAN 空欄 + 接頭辞のある部門はサーバーが発番）、`docs/UI_TECH_STACK.md` E6、`docs/db-design/master-tables.md` SP-102-04（独自コードは変更不可）、30-biz BIZ-01-D5（商品コードの長さ上限）。
- 必要な設計成果物: BIZ の振舞いの変更 → 30-biz を updated in this PR（plan-first commit）。新しい IO 関数 → 20-io を updated in this PR。持続の振舞い（next_seq の意味）→ `docs/db-design/master-tables.md` を updated in this PR（`docs/DB_DESIGN.md` の索引の 1 行「独自コード発番」は変わらないため existing sufficient）。durable な判断 → D-106。Tauri command / DTO / 画面 / CSV の形は変えないため該当なし。
- plan にしかない durable な判断の昇格先: 番号の振り方と却下案は 30-biz BIZ-01-D6 と D-106。owner 決定 TD-108（抜けを埋めない）は BIZ-01-D6 と D-106、TD-109（9999 を超えたら 5 桁以上）は master-tables C-1・30-biz §4.3 step 6・D-106 に書いた。packet にしかない判断は無い。
- 前提・制約と、延期した design gap の follow-up: 実データの独自コードの形は未確認（P3）。Excel から初期投入する経路の設計は本 lane の外（材料の報告の別項目。Coordinator が backlog 化を判断）。同時登録の競合は非目的（single-instance）。
- 絶対保証の自己点検: 「既存の商品に当たって止まらない」は、番号の最大を全商品から数え、`{接頭辞}-{数字}` の形だけが発番の結果と同じ文字列になりうることに依る。例外: (1) i64 を超える桁のコードは数えないが、発番の結果（i64 の 4 桁以上の 0 埋め）と同じ文字列にならない。(2) 0 埋めの桁が違うコード（`HZ-3` と `HZ-0003`）は別の文字列で、両方とも番号 3 と数えるので次は 4 以上になり当たらない。(3) 既存の番号の最大が i64::MAX のとき（step 3c）と、next_seq と最大 + 1 の大きい方が i64::MAX のとき（step 3d）は、next_seq の SQL 更新の前に `ValidationFailed` で止まる（商品・next_seq を変えない。登録できないが、現実の店の運用では届かない）。(4) step 7 の重複の判定は残す（届かないが、上の推論の誤りを `DuplicateProductCode` で止め、データを壊さない）。
- 判定: ready。上の設計正本だけで、何を作るか・なぜか・何を却下したかに答えられ、Matrix は BIZ-01-D6 と 20-io の処理ステップから導ける。

## Registration / Generation Obligations

| 変更対象 | 登録・生成義務 |
|---|---|
| REQ / coverage（REQ-101 の test が増える） | `cd src-tauri && cargo run --bin generate_traceability` で `docs/function-design/90-traceability.md` を再生成（S3、AC4） |
| pub 関数の追加（IO） | 20-io に signature を載せた（本 commit）。`design_compliance_test` が突合する（AC3）。allowlist に足さない |

Tauri command・route・画面・function-design doc の新設は無い。

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | Which concepts belong to replaceable external adapters, and which concepts are stable app-core contracts? | Architecture, function design, decision-log, Plan Packet | 当たる。独自コードの形と発番は app core（BIZ-01）。POS の 8 桁の独自コード（`docs/plu-export-and-real-csv-verification.md:127`）は adapter 側の別物で、本 lane は触らない | なし |
| Fact check / design decision split | Which claims are observed facts from hardware/tool/files, and which are app decisions that need source-doc promotion? | Investigation doc, source design docs, decision-log | 当たる。観測 = 今の発番が取込み後に止まり続ける（P1・P2）。判断 = 最大の次から振る・抜けを使わない（BIZ-01-D6 / D-106）。実データに該当の形があるかは未観測（P3） | BIZ-01-D6、D-106 |
| Lifecycle / retry | What happens before, during, after, and after failure for import/export, duplicate input, rollback, retry, cancellation, and re-run? | Function design, DB design, UI design, Test Matrix | 当たる。登録の失敗で next_seq の引上げも戻る（同じ TX、T6）。取込みの再実行（上書き）はコードを変えないので発番に影響しない。再起動をまたぐ状態は DB だけ | Matrix T6 |
| Operator workflow | What does the operator do in the real sequence across app, external tool, media, print/export, backup, and recovery? | Screen/UI design, function design, Plan Packet manual checks | 当たる。Excel → 商品 CSV → 取込み → 画面で JAN の無い商品を登録、の順（Ordinary Operation）。画面の操作は変わらない | なし |
| Replacement path | If the external system changes, which files/modules/docs are replaced and which app-core contracts remain stable? | Architecture, function design, decision-log | 不該当。外部の system に依らない（BIZ と DB だけ） | なし |
| Data safety / evidence | How can the claim be supported by anonymized shape/count/hash/procedure evidence without committing real store data? | Plan Packet Data Safety, investigation doc, review evidence | 当たる。probe と test は合成の `HZ-0001` 等だけ。持ち帰りデータは読んでいない（件数も書かない） | Data Safety |
| Reporting / accounting semantics | Are totals, summaries, item records, returns, corrections, and inventory movements modeled separately enough to avoid false business meaning? | DB design, function design, report design, Test Matrix | 不該当。在庫の移動・金額・集計に触れない | なし |
| Manual verification | Which assertions cannot be proven by automated tests and require Windows native L3, external tool import, or real-device confirmation? | Plan Packet, Test Matrix, PR body | 不該当。画面は変わらず、結果のコードは BIZ の test で確かめられる | なし |
| 環境・再現性 | 新設の環境依存（toolchain / CI runner / OS 差異等）を repo-pinned config で強制するか、明示的に defer するか | repo-pinned config, Plan Packet | 不該当。新しい環境依存は無い（SQLite の `substr` / `length` / `MAX` は bundled SQLite の標準の関数） | なし |

## Boundary / Wire Contract

- producer: BIZ `generate_custom_code`（`create_product` の中）が `products.product_code` と `departments.next_seq` を書く。
- consumer: `create_product` の結果の `ProductCreateResult.product_code`（frontend の保存の結果の表示）、以後の発番。
- wire type: 変えない（`ProductCreateRequest` / `ProductCreateResult` / `CmdError` の形と `src/lib/bindings.ts` は不変）。
- internal type: `departments.next_seq`（INTEGER）の意味を「次に発番する連番」から「次に発番する連番の下限」へ（master-tables）。`products.product_code` の形は不変（`{接頭辞}-{4 桁以上の 0 埋め}`）。
- precision/range: 番号は i64。step 3c の `最大.checked_add(1)` か step 3d の `cand.checked_add(1)` が None なら、next_seq の SQL 更新の前に `ValidationFailed`（商品・next_seq は不変）。番号の桁は 4 桁以上で、9999 を超えたら 5 桁以上で続ける（owner 決定 TD-109、T8）。
- round-trip path: 商品 CSV の取込み（コードをそのまま入れる）→ 発番（取り込んだ番号を飛ばす）。逆向き（発番したコードを CSV で上書き）は既存の上書きの経路で、コードは変わらない。
- invalid input: `{接頭辞}-` の後ろが数字以外を含む・空・i64 を超える桁のコードは番号として数えない（発番の結果と同じ文字列にならない）。
- compatibility: 既存の DB は migration なしで動く。同じ接頭辞の既存の独自コードの番号の最大 + 1 が next_seq 以下の部門では、振られる番号は今までと変わらない。古い版のアプリで同じ DB を開いても、next_seq は上がっただけなので古い発番の結果も既存と重ならない方向にしか動かない。

## Test Plan

Test Design Matrix: [2026-10-04-custom-code-seq](test-matrices/2026-10-04-custom-code-seq.md)。

- targeted tests: `cargo test --lib biz::product_service`（T1〜T8 と改名した 1 本。T7 は `test_generate_custom_code_req101_rejects_seq_overflow`・`test_generate_custom_code_req101_rejects_overflow_after_max`・`test_generate_custom_code_req101_rejects_overflow_from_next_seq`、T8 は `test_generate_custom_code_req101_continues_past_9999`）、`cargo test --lib db::product_repo`（R1・R2）、`cargo test --test design_compliance_test`。
- negative tests: 番号として数えないコード（T4）、i64 の上限（T7a〜T7c。商品数と next_seq が不変）、存在しない部門の `raise_next_seq`（R2）、`LIKE` の大文字・小文字・`%` の取り違え（R1）。
- compatibility checks: 同じ接頭辞の既存の独自コードの番号の最大 + 1 が next_seq 以下の部門では、振られる番号は今までと変わらない（既存の `test_generate_custom_code_req101_normal`・`_sequential`、next_seq が最大より大きい T3）。9999 の次は 5 桁（T8）。
- data safety checks: test の値は合成のコードだけ（Data Safety）。
- main wiring/integration checks: T1 が `preview_import` → `commit_import` → `create_product` を同じ DB で通す（画面の取込みと登録と同じ BIZ の入口）。CMD は薄く変えないため、CMD の test は足さない。

## Review Focus

- BIZ-01-D6 が Goal（取込み後も登録できる）を満たすか。「抜けた番号を使わない」の採否（却下 (a)）は owner 決定 TD-108 で解決。
- 番号の読み取りの規則（`{接頭辞}-` + ASCII 数字だけ、大文字・小文字を区別、部門を問わない）の穴。Design Readiness の絶対保証の例外 (1)〜(4) が漏れなく閉じているか。
- `raise_next_seq` と `list_product_codes_by_prefix` の SQL（`LIKE` を使わない理由、`MAX` で下げない、変更行数 0 で NotFound）。
- 既存 test `test_create_product_req101_duplicate_key_from_insert` の改名と書換えが「不都合な test の弱体化」でないか（S2 の理由）。
- T1〜T8・R1・R2 が Matrix の mutation（空き番号を埋める実装、`LIKE`、next_seq を下げる、TX の外での引上げ、数字以外を数える、`checked_add` を `+ 1` に戻す）で red になるか。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| BIZ-01-D6（最大の次と next_seq の大きい方、抜けを使わない〈TD-108〉） | 30-biz §4.3 step 3〜6・設計判断 | S1 `generate_custom_code` | T1、T2、T3 | — |
| BIZ-01-D6（番号として数えるコードの規則） | 30-biz §4.3 step 3a〜3b | S1 | T4、T5 | — |
| BIZ-01-D6（i64 の上限: step 3c・3d の `checked_add` が None なら next_seq の SQL 更新の前に `ValidationFailed`） | 30-biz §4.3 step 3c〜3d | S1 | T7a、T7b `test_generate_custom_code_req101_rejects_overflow_after_max`、T7c `test_generate_custom_code_req101_rejects_overflow_from_next_seq` | — |
| C-1（9999 を超えたら 5 桁以上で続ける、TD-109） | `docs/db-design/master-tables.md` C-1、30-biz §4.3 step 6 | 変えない（`{:04}` は 4 桁以上を出す） | T8 `test_generate_custom_code_req101_continues_past_9999` | — |
| 30-biz §4.3 step 7（重複の判定を残す） | 30-biz §4.3 step 7 | S1（変えない） | 届かないため直接の test なし（Design Readiness の例外 (4)） | — |
| 30-biz §4.3 step 1〜2（部門の不在・接頭辞の無い部門） | 30-biz §4.3 | 変えない | `test_generate_custom_code_req101_no_prefix`（既存） | — |
| 30-biz §4.3 signature（借りた transaction） | 30-biz §4.3 | 変えない | `test_generate_custom_code_req101_requires_borrowed_transaction`（既存） | — |
| 30-biz §4.2 step 2・エラーハンドリング（TX の rollback で next_seq の引上げも戻る） | 30-biz §4.2、§4.3 の注 | S1 | T6、`test_create_product_req101_rollback_after_insert`（既存） | — |
| 30-biz §4.2 エラーハンドリング（重複コードは JAN で起きる） | 30-biz §4.2 | 変えない | `test_create_product_req101_duplicate_jan`（既存）、S2 で改名する既存の test（Matrix の改名の行） | — |
| 30-biz §4.9 step 2c（取込みは next_seq を進めない） | 30-biz §4.9 | 変えない | T1（取込みの直後の next_seq が 1 のまま） | — |
| 20-io §2.3 `list_product_codes_by_prefix` | 20-io §2.3 | S1 | R1 | — |
| 20-io §2.4 `raise_next_seq` | 20-io §2.4 | S1 | R2 | — |
| 20-io §2.4 `increment_next_seq`（不変） | 20-io §2.4 | 変えない | `test_increment_next_seq_req101_normal`・`_nonexistent_department`（既存） | — |
| master-tables departments.next_seq（連番の下限） | `docs/db-design/master-tables.md` §2 | S1 | T1、T3 | — |
| BIZ-01-D5（商品コードの長さ上限） | 30-biz §4.9 | 変えない | 既存の BIZ-01-D5 の test | 非対象（発番の結果は上限に届かない） |
| UI-01b-D4（JAN 空欄 + 接頭辞のある部門はサーバーが発番） | 51-ui §UI-01b-D4 | 変えない | 既存の frontend の test | 非対象（画面と request は不変） |
| SP-102-04（独自コードは変更不可） | master-tables §1 | 変えない | — | 非対象（既存のコードを書き換えない） |
| `DuplicateProductCode` の CMD の文言 | 40-cmd / `cmd/mod.rs:158-160` | 変えない | 既存 `cmd/mod.rs:264` の test | 非対象 |

adjacent-contract sweep: 30-biz §4.2〜§4.3・§4.8〜§4.9、20-io §2.3〜§2.4、master-tables §1〜§2、51-ui UI-01b-D4、`docs/UI_TECH_STACK.md` E6 を読み、Scope が触り得る契約を上の行に足すか非対象と明記した。

## Contract Probe

Probe は scratch の仮の test（`product_service.rs` の test module の末尾に一時的に足し、実行後に `git checkout -- src-tauri/src/biz/product_service.rs` で消した。tracked に残していない）で、base `76de30d8` の code のまま `cd src-tauri && cargo test --lib probe_lane_e -- --nocapture` を実行した（2026-10-04、exit 0）。CSV は合成の UTF-8 BOM 付きの 3 行（header、`HZ-0001`、`HZ-0002`、部門 ID 2）。

- P1 商品 CSV の取込みは部門の next_seq を進めない: `preview_import` → `commit_import` の後の部門 2 の next_seq → 出力 `PROBE preview valid=2 err=0 dup=0` / `PROBE commit created=2` / `PROBE next_seq after import = 1`。
- P2 画面と同じ経路の登録（`create_product`、JAN なし、部門 2）は毎回止まり、next_seq は TX の rollback で戻る: 3 回続けて呼ぶ → 出力 `PROBE attempt 1 -> Err("DuplicateProductCode(\"HZ-0001\")") ; next_seq = 1`、attempt 2・3 も同じ。同じ DB で部門 1（接頭辞 KM）は `PROBE other dept (KM) -> Ok("KM-0001")`（`KM-` のコードが無く、既存の番号の最大 + 1 が next_seq 以下の部門は影響を受けない）。
- P3 実データに `{接頭辞}-NNNN` の形の独自コードがあるか: 未確認。店の Excel の在庫シートは持ち帰りに無い（材料の報告の結論 1）。POS の独自コードは 8 桁（`docs/plu-export-and-real-csv-verification.md:127`）で別の形。Goal は実データの有無に依らないため、確かめずに進める。
- P4 SQLite の `UPDATE departments SET next_seq = MAX(next_seq, ?2) WHERE id = ?1` は値が変わらなくても変更行数 1、id が無ければ 0。`substr(code, 1, length(?1)) = ?1` は大文字・小文字を区別し `%` を特殊文字にしない（`LIKE ?1 || '%'` は `hz-0009` も返す）: Python sqlite3（SQLite 3.51.3）の in-memory の DB で実行 → `changes(no-op raise)= 1 (5,)`、`changes(missing id)= 0`、`substr: [('HZ-0001',)]`、`substr H%-: [('H%-0001',)]`、`like HZ-%: [('HZ-0001',), ('hz-0009',)]`。アプリの rusqlite は bundled SQLite 3.45.0 で版が違うが、`changes()` の数え方と `substr` / `length` / `LIKE` の大文字・小文字の扱いは両版の公式の仕様で同じ。R1・R2 が bundled の版で確かめる。

## Data Safety

- 持ち帰りデータ（`~/downloads/inventory-field-check/`）は本 lane で読んでいない。件数・商品名・JAN・部門名・実際の商品コードを tracked に書かない。
- test・probe のコードは合成（`HZ-0001` 等の 2 文字の接頭辞 + 4 桁以上、部門は `schema_v1.rs` の初期データの ID）。
- local-only paths: probe の log と diff は Coordinator の scratchpad（tracked の外）。
- synthetic-only paths: `src-tauri/src/**` の `#[cfg(test)]` と tempdir の DB。

## Implementation Results

- 実装の要旨（closeout、2026-10-06）: `generate_custom_code` は、同じ接頭辞の既存の独自コード（`{接頭辞}-` + ASCII 数字だけ、部門を問わない）の番号の最大 + 1 と部門の next_seq の大きい方から振り、抜けた番号は埋めない（TD-108）。9999 の次は 5 桁以上で続け（TD-109）、i64 の上限は next_seq の更新の前に `ValidationFailed` で止める。`product_repo` に `list_product_codes_by_prefix`（`LIKE` を使わない）と `raise_next_seq`（next_seq を下げない）を足し、引上げは登録と同じ TX で戻る。前提が BIZ-01-D6 と食い違う既存 test 1 本を改名して書き直し、traceability を再生成した（D-106）。
- review・CI・merge: Final Review broad は互いに独立の 2 本がともに approve（Fable 5.1 は P3 1、Codex GPT-6.1 Sol は P 0）。Draft の run は success で、helper 経由の squash merge（owner の Ready・merge、2026-10-04）。manual は not-required。検証（AC1〜AC8、mutant M1〜M6）の実測値は PR の body が持つ。PR: [#138](https://github.com/higekiri-w/inventory-system-desktop/pull/138)（Writer = Opus 5.5 subagent）

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

## Review Response

Fill after review.
先行 round の結果・評価（判定・件数・採否・reviewer の意見）はこの節にだけ書く。前半の節と遷移記録には「round N の是正（Review Response 参照）」だけを書く（独立 review は `## Review Response` より前だけを読む）。

- Plan Review round 1（2026-10-04、対象 `8ded2dfc`、互いに独立の 2 本）: fresh Opus 5.5 = approve（P3 2）、Codex GPT-6.1 Sol（発注 199）= reject（P1 1 / P2 3 / P3 1）。Coordinator が現物で裏取りして全件を採用し、P2 2 件は owner 決定（TD-108・TD-109）で解いた。是正は相談役（Fable 5.1）の反例探しの後、起草役（Opus 5.5）が反映した（`09418db6`）。Phase は plan-gate のまま。
  - Codex #1（P1、上限の検査が最大 + 1 だけで、その後の `increment_next_seq` の `next_seq + 1` が i64 を超えて next_seq が REAL になる）・Opus #1（P3、同じ境界の 1 つずれ）: 採用。30-biz §4.3 を step 3c（`最大.checked_add(1)`）と step 3d（`cand = max(next_seq, floor)`、`cand.checked_add(1)`）の 1 通りにし、next_seq の SQL 更新の前に `ValidationFailed` で止める。Matrix に T7b・T7c と M6 を足した。
  - Codex #2（P2、Goal の「CSV 未取込みの部門は番号不変」と、部門を問わず全商品から最大を求める契約が矛盾する）: 採用。Goal と互換性の段を「既存の番号の最大 + 1 が next_seq 以下の部門」に直した。
  - Codex #3（P2、抜けを使わない判断を未確認の店の事実から確定している）: 採用。owner に諮り TD-108（抜けを埋めず最大の次）を得て、BIZ-01-D6・D-106 の根拠を置き換えた。
  - Codex #4（P2、master-tables の「4 桁・9999 まで」と新しい契約が同期していない）: 採用。owner に諮り TD-109（9999 を超えたら 5 桁以上で続ける）を得て、master-tables C-1・SCREEN_DESIGN・30-biz に反映した。
  - Codex #5（P3、不在確認の `rg` が `\|` を alternation として扱い、不在の証明にならない）: 採用。起票時実測 #5 の command を直した。
  - Opus #2（P3、T6 の「Would fail if」の数値が設計と合わない）: 採用。T6 の期待値を直した。
- Plan Review round 2（2026-10-04、対象 `09418db6`、互いに独立の 2 本）: fresh Opus 5.5 = approve（P3 2）、Codex GPT-6 Astra（発注 203。round 1 で Codex が P1 を出したので MODEL-SELECTION の表で Astra）= approve（P1 0 / P2 0 / P3 0）。Ordinary Operation は 2 本とも成立。P3 は Coordinator が全件採用し、本 commit で反映した。独立の再確認は Final Review が兼ねる。
  - Opus #1（P3、T4 に符号付きの形が無く、ASCII 数字の判定を省いて `str::parse::<i64>` だけで読む実装〈`+` を受ける〉が green のまま通る）: 採用。Matrix の T4 の既存コードに `HZ-+0009` を足し、「Would fail if」にその近道を足した（期待値 `HZ-0001` は変えない）。
  - Opus #2（P3、`generate_custom_code` の doc comment「4桁ゼロ埋め連番」〈`product_service.rs:152`〉が TD-109 と食い違い、S1 に comment の更新が無い）: 採用。S1 に doc comment を §4.3 に合わせる一句を足した。

- Closeout（2026-10-06）: Final Review broad（互いに独立の 2 本）は Fable 5.1 = approve（P1 / P2 = 0、P3 1）、Codex GPT-6.1 Sol = approve（P1 / P2 / P3 = 0）。2 本とも AC1〜AC8 と mutant M1〜M6 を独立に再測し、Matrix の予測どおり red になることを確かめた。専用 record に pass で記録した。P3（step 3d の上限の検査が値を捨てる式文で、正本の `cand` の名前が無い。可読性だけ）は採用しない（振舞いは 30-biz §4.3 のとおりで、M6b / M6c が red）。backlog へ送る follow-up は無い。PR #138 は helper 経由の squash merge（2026-10-04）で閉じた。
