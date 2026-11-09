# Plan Packet: 型生成（TS bindings）の失敗を失敗として検出する（R3）

2026-09-22 起草。出典は Backlog「やると決めたもの（順番未定）」の [型生成の失敗が成功扱いになる](../../backlog.md#やると決めたもの順番未定) entry（2026-09-22 の外部設計相談で指摘、Coordinator が現物で確認）。owner 決定 2026-09-22「次に着手する最初の小変更」で着手。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

- Evidence Mode: github
- Phase: archive
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: c9d7d02883d9d18d5c208d9f32b06860a8d8578b
- Amendments: 95f081ca1dc385968523db12ccf1485d18b2d404, a8544ed8734b3886965114abbcb9c3e0291460c1
- Coordinator: Fable 5.1
- Writer: Sonnet subagent（worktree run）
- Plan Reviewer: Sonnet（独立 fresh context）
- Final Reviewer: Codex + Opus（互いに独立の fresh context、Double Audit。GA2）
- Final Review Minimum: 2
- Human Gate: ready,merge

manual なし: CI は ubuntu-latest のみで、Windows native での `create_new` / rename による置換は未実測（closeout で訂正、pass B P3-6）。失敗しても CLI と CI は非 0 で検出し、`run()` は警告して起動を続けるため誤 green を作らない。画面・operator workflow・配布物への変化はない。

遷移記録（append-only）:

- kickoff → spec-check → design → plan-draft → plan-gate（本 commit、plan-first）: 設計正本に型生成の失敗時の振舞いを定める節は無く、既存の利用契約（`docs/DEV_WORKFLOW.md` Verification Gates「generate_bindings の後に `src/lib/bindings.ts` の diff を見る」、D-054 の「L1 の bindings clean diff 検査が cross-language 同期の機械検査を兼ねる」）が暗黙に前提とする「生成が失敗したら検査も失敗する」を実装が満たしていない、という実装側の欠陥。owner の設計判断を要する論点なし
- plan-gate → plan-approved → implementing（本 commit、state-only）: Plan Review round 1（Sonnet、P1 0 / P2 2 / P3 2）→ 是正 `9f68abdc` → round 2 closure（同 reviewer、P1/P2 = 0、P3 1）→ P3 を in-place 是正。Plan Commit = `c9d7d028`（plan-first `41b1ad89` → 是正を含む確定版）。実装は Sonnet subagent の worktree run で本 commit を起点にする
- Gated Amendment 1（本 commit）: Writer の初回実装 run（`368dec36`）の報告で、Test Plan T2 の失敗条件「存在しない親 directory」が現物と合わないと判明（`specta-typescript 0.0.11` `src/exporter.rs` の `export_to` が `create_dir_all(parent)` を実行、Coordinator が registry の source で確認）。T2 の条件を「親 directory を作れない」へ訂正し、T4 を既存 file の保護を直接見る独立 test に改める。Goal Invariant / Scope / AC / Risk / Final Review Minimum / Human Gate は不変。旧前提の sweep: `rg -n '存在しない親' ` で packet・Plans.md に残存なし
- Gated Amendment 2（本 commit）: Final Review（Codex、head `355a647d`、PR #88 `#pullrequestreview-5270529065`）の P2 2 件を Coordinator が採用し、owner が 2026-09-22 に Risk の引上げを承認。(a) finding #2: 本 change は required な `generated-bindings` gate の合否（生成失敗 + 旧 bindings 残存が green から red へ）を変える。`docs/project-profile.md` High-risk Changes の R3 例「Test/workflow gates that affect what may be merged」と R2 行の「merge gates を変えない developer script」、`docs/DEV_WORKFLOW.md` Risk Tiers の「迷ったら workflow gate に触れる場合は R3」に照らし、Risk を R3、Final Review Minimum を 2（workflow gate change の Double Audit）へ改める。起票時の R2 判断は CI 定義 file を編集しないことと classifier の workflow=false に依っていたが、どちらも影響による分類の免除にならない。R3 必須節（Spec Contract / Trace Matrix / Data Safety / Contract Coverage Ledger / Contract Probe / Test Design Matrix）を追加する。(b) finding #1: 固定名の一時 file を並走する生成が取り合い、成功 exit のまま壊れた bindings を公開し得る。Scope S2 に「一時 file は呼出しごとに一意」を加え、Test Plan に T7（並走）を追加、T4 の失敗条件を一意な名前と両立する形へ改める（実装は `64285112`）。(c) finding #3（P3、非 blocker）は Matrix の Residual Test Gaps と backlog へ回す。Plan 契約が変わるため、head `355a647d` の Codex review は helper の broad として記録せず、是正後の head で broad 2 本を取り直す。Owner Effort Budget の介入上限を 5 へ改める。Goal Invariant / Non-scope / Human Gate は不変
- implementing → archive（closeout、本 commit）: PR #88 squash merge `7d2002a0`（2026-09-22）。packet / Matrix を archive へ移送し、Implementation Results / Review Response を記録、broad 後の是正で現物とずれた記述を訂正

## Owner Effort Budget

- 介入回数上限: 5（GA2: 起票時 3。Codex relay 2 往復、Risk 引上げの判断、Ready、merge で 5 になるため）
- 実働時間上限: 10分
- relay 往復上限: 2（GA2: round 1 の Codex review 1 往復 + 取り直しの Codex broad 1 往復）
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
required な merge gate（local CI の `generated-bindings` / `generated-bindings-diff`、hosted CI の bindings drift step）が実行する生成 CLI の終了コードを変え、生成失敗 + 旧 bindings 残存を green から red に変える。gate の合否を変える workflow gate change として R3、Double Audit（Final Review Minimum 2）を適用する（GA2。`docs/project-profile.md` High-risk Changes、`docs/DEV_WORKFLOW.md` Risk Tiers）。runtime の契約（Tauri command DTO、`src/lib/bindings.ts` の生成内容、DB、operator 画面）は変えず、CI / local CI の定義 file も編集しない。

## Goal

Goal Invariant:

### 最小完了条件

- 型生成のどの段（specta export / 整形 / 定数追記 / 置換）が失敗しても、`cargo run --bin generate_bindings` が非 0 で終了し、完了 message を出さない。これにより local CI と hosted CI の bindings 検査が、古い `src/lib/bindings.ts` を残したまま green にならない。

### 失敗定義

- 生成のいずれかの段が失敗したのに CLI が 0 で終了する経路が残る。
- 成功時の `src/lib/bindings.ts` の内容が現行と 1 byte でも変わる。
- 生成の失敗が `tauri dev`（`run()` の debug 自動生成）の起動を止めるようになる。

### 非目的

- 生成内容・整形規則・定数の追加や変更。
- CI / local CI の step 定義の変更、新しい gate の追加。
- error 型の体系化（専用 enum の新設、`thiserror` の導入）。失敗した段が message で分かれば足りる。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

- S1 `src-tauri/src/lib.rs`: `export_specta_bindings()`（debug build 専用）が `Result` を返すようにする。specta export / `normalize_generated_bindings` / `append_generated_constants` / 置換のどの段の失敗も `Err` で返し、message から失敗した段と対象 path が分かるようにする。関数内の `eprintln!` による握りつぶしは無くす。
- S2 `src-tauri/src/lib.rs`: 生成は同じ directory 内の一時 file へ出力し、整形と定数追記まで完成させてから `src/lib/bindings.ts` へ rename で置換する。一時 file の名前は呼出しごとに一意にし、並走する別の生成（`tauri dev` の自動生成と CLI、同じ process 内の別 thread）の一時 file に触れない（GA2）。失敗時は既存の `bindings.ts` に触れず、自分の一時 file は best-effort で消す。path を引数に取る内部関数へ分け、test が tempdir に対して同じ経路を通せるようにする（公開 API は `export_specta_bindings()` のまま）。
- S3 `src-tauri/src/lib.rs` `run()`: 呼出し側で `Err` を受け、現行どおり警告を `eprintln!` して起動を続ける（`tauri dev` を止めない）。
- S4 `src-tauri/src/bin/generate_bindings.rs`: `Err` なら message を stderr へ出して `std::process::exit(1)`、`Ok` のときだけ完了 message を print する。冒頭の doc comment を実態に合わせる。
- S5 `src-tauri/src/lib.rs` の `bindings_generation_tests`: 下記 Test Plan の test を追加する。この module は起動失敗表示・DB 初期化の test も同居しているが、既存 test はどれも変更しない。
- S6 closeout（merge 後の別 PR）: `docs/backlog.md` の当該 entry の消し込みと `docs/Plans.md` の更新。

呼出し側の確認（2026-09-22、`git grep -n 'export_specta_bindings\|generate_bindings' origin/main -- src-tauri scripts .github`）: `export_specta_bindings` の呼出しは `src-tauri/src/lib.rs` の `run()` と `src-tauri/src/bin/generate_bindings.rs` の 2 箇所だけ。CLI の利用者は `.github/workflows/ci.yml:235` と `scripts/local-ci.sh:214`（`run_required`）で、どちらも非 0 終了を失敗として扱うため編集不要。

許容する書込み: `src-tauri/target/**`（build）、`.local/ci-evidence/**`（`scripts/local-ci.sh` の出力）、成功時に再生成される `src/lib/bindings.ts`（差分 0 が期待値）。

## Non-scope

- `.github/workflows/**`、`scripts/**`、`docs/DEV_WORKFLOW.md`、`docs/project-profile.md`、`docs/ci.md` ほか policy 文書の編集。
- `src/lib/bindings.ts` の内容変更、`src-tauri/src/bin/generate_traceability.rs` の同種の見直し。
- 新しい依存の追加（`tempfile` は既存の dev-dependency を使う）。
- Writer による本 packet の編集（不整合を見つけたら編集せず停止して報告する）。

## Acceptance Criteria

- AC1 `cd src-tauri && cargo run --bin generate_bindings` が exit 0 で完了 message を出し、`git diff --exit-code -- src/lib/bindings.ts` が差分 0（成功時の生成内容が不変）で、`git status --short -- src/lib` が空（一時 file が残らない）。
- AC2 失敗の実測（Writer が 1 回行い PR 本文へ command と exit code を記録）: `src/lib` を一時的に書込み不可にして（`chmod a-w src/lib`）`cargo run --bin generate_bindings` を実行すると、非 0 で終了し、完了 message `TS bindings exported` を出さず、stderr に失敗した段と path が出る。実行後に権限を戻し（`chmod u+w src/lib`）、`git status --short` が clean で、`src/lib` に一時 file が残っていない。この条件では一時 file の作成自体が失敗するため、作成後の後始末は T3 が担う。
- AC3 Test Plan の T1〜T8 が全 PASS し、`cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test` が成功する。
- AC4 `export_specta_bindings()` とその内部関数に `eprintln!` が残らず、bindings 生成の警告は `run()` の 1 箇所だけになる。baseline（origin/main `54ed8990`、`rg -n 'eprintln!' src-tauri/src/lib.rs`）= 223 / 348 / 354 / 361 / 908 行の 5 件で、うち 348 / 354 / 361 の 3 件が `export_specta_bindings()` 内。223（起動失敗の表示）と 908（診断ログ初期化の警告）は対象外で残す。
- AC5 `bash scripts/local-ci.sh changed` が成功する。

## Design Sources

- Requirements / spec: REQ-104 / D-054（`docs/decision-log.md`、bindings 生成が cross-language 定数同期の機械検査を兼ねる）
- Architecture: 該当なし（layer 境界に触れない）
- Function / command / DTO: 該当なし（command と DTO は不変）
- DB: 該当なし
- Screen / UI: 該当なし
- Decision log / ADR: D-054、`docs/DEV_WORKFLOW.md` Verification Gates の `Tauri command DTOs` 行

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | なし（開発ツールの失敗経路） | existing sufficient |
| Command / DTO / generated binding / wire shape | D-054、Verification Gates | existing sufficient（生成内容は不変） |
| DB / transaction / audit / rollback / migration | 該当なし | existing sufficient |
| Screen / UI / route state / Japanese wording | 該当なし | existing sufficient |
| CSV / TSV / report / import / export format | 該当なし | existing sufficient |
| Durable decision / ADR | 該当なし（新しい決定を作らない） | existing sufficient |

## Registration / Generation Obligations

該当なし（command / doc / REQ / route / 画面の追加・改名・削除を含まない。追加 test は既存の REQ-104 を参照するか、`test_` prefix なしの WF 系 meta-test 命名にする。REQ-104 参照の test を増やす場合は `cargo run --bin generate_traceability -- --check` を通し、drift が出たら `docs/function-design/90-traceability.md` の再生成を同じ commit に含める）。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| REQ-104 | `docs/decision-log.md` D-054 | D-054 | bindings の clean diff 検査を同期の機械検査として信用するには、生成の失敗が検査の失敗になる必要がある。却下: CI 側で stderr を grep する（生成側の終了コードが正しければ不要な二重化） | `export_specta_bindings()` / `generate_bindings` の `main` | T1〜T8、AC2 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: はい（D-054 と backlog entry）
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: なし
- Assumptions and constraints: `std::fs::rename` は同じ directory 内なら同一 filesystem で置換できる。debug build 専用（`cfg(debug_assertions)`）のまま
- Deferred design gaps, risk, and follow-up target: `generate_traceability` の失敗経路は未確認で本 change の対象外
- Test Design Matrix can cite design decision IDs or source doc sections: [Matrix](test-matrices/2026-09-22-bindings-export-failure-detection.md) が Spec Contract C1〜C6 と D-054 を引く（GA2）
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: `run()` だけは失敗を警告に落として続行する。理由は開発中の起動を止めないためで、gate としての検査は CLI 経路が担う

## Impact Review Lenses

not applicable: 実機調査・外部 tool の挙動・POS 連携・operator workflow の発見を起点にしない、開発ツール内部の失敗伝播の修正。

## Design Readiness

- Existing design docs are sufficient because: 生成内容と利用手順は変わらず、失敗時に失敗を返すという既存手順の暗黙の前提を実装が満たすだけ
- Source docs updated in this PR: なし
- Design gaps intentionally deferred: なし
- Durable decisions discovered in this plan and promoted to source docs: なし

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): 触れない
- Backend function design: 触れない
- Command / DTO / data contract: 不変（AC1 で生成物の差分 0 を確認）
- Persistence / transaction / audit impact: なし
- Operator workflow / Japanese UI wording: なし
- Error, empty, retry, and recovery behavior: 失敗時は既存の `bindings.ts` を保ち、再実行で回復する
- Testability and traceability IDs: REQ-104 / D-054

## Contract Probe

- `tauri_specta::Builder::export` の失敗条件: registry の source（`specta-typescript 0.0.11` `src/exporter.rs` `export_to`）を読む -> 既定 layout では出力文字列を作ってから `create_dir_all(parent)` + `std::fs::write(path)`。親 directory が無いだけでは失敗しない（GA1）。拡張子による分岐・formatter の起動は無く、一時 file 名は出力内容に影響しない（Codex が直接 `.ts` 出力と一時 file 経由の出力の byte 一致を実測）。
- 同じ directory 内の `std::fs::rename` による置換: WSL（ext4）で AC1 と T1 / T7 が成功 -> 成立。Windows native は未実測。Rust 1.94.1 の Windows 実装は `MoveFileExW` + `MOVEFILE_REPLACE_EXISTING` で既存 file を置換できるが、DELETE 共有を許さない handle が開いていると失敗し得る（Codex round 1 の調査）。失敗しても `run()` は警告して起動を続け、CLI は非 0 で終わるため Goal Invariant は破れない。Matrix の Residual Test Gaps に残す。
- 固定名の一時 file の並走: Codex が未変更 binary 2 本の並走で両方 exit 0・出力 63 bytes を実測、Writer が固定名へ戻した T7 で 10/10 red を実測 -> 一意な名前が必要（GA2）。一意な名前（process id + counter）だけでは足りなかった: process id は PID namespace 間で一意でないため、別 namespace の process が同じ候補を選び得ることが Final Review broad pass A で実測された（両方 exit 0 のまま bindings が壊れる）。是正は候補を `create_new` で排他確保し、既に他者が確保済みの候補は消さず次の候補へ進む形にした（closeout で訂正、D3）。

## Contract Coverage Ledger

touched source-doc section は D-054（`docs/decision-log.md`）の「L1 の bindings clean diff 検査が cross-language 同期の機械検査を兼ねる」と `docs/DEV_WORKFLOW.md` Verification Gates の `Tauri command DTOs` 行。隣接契約の sweep: D-054 ①の定数 export（`CSV_IMPORT_FILE_SIZE_LIMIT`）は生成内容として C2 で不変を見る。D-054 ②③（FilePicker、import CMD の上限）は本 Scope が触れない。

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| C1 どの段の失敗も `Err` になり CLI は非 0・完了 message なし | `export_bindings_to` / `generate_bindings` `main` | T3 / T5 / T6（置換・整形・定数追記段の `Err`）、T2 / T4 は一時 file の確保段で落ちる（closeout で訂正、D12。specta export 自体の `Err` 伝播を固定する自動 test は無い、Residual Test Gaps） | CLI の終了コードと合成関数内の伝播は AC2 の実測と review（Residual Test Gaps） |
| C2 成功時の生成内容は不変（D-054 ①の定数 export を含む） | `export_bindings_to` | T1、local CI `generated-bindings-diff` | AC1 |
| C3 失敗時は既存の `bindings.ts` を変えず、自分の一時 file を残さない | `export_bindings_to` の後始末 | T3 / T4 | AC2 |
| C4 並走する生成が互いの一時 file に触れず、成功した呼出しは完成物だけを公開する | `acquire_temp_path` / `temp_candidate_path`（closeout で訂正、D1。旧記述の `unique_temp_path` は存在しない関数名） | T7 / T8 | 別 process・別 PID namespace 間の並走は broad pass A の再現手順（PR #88 review）と Coordinator の実測で確認、自動 test は同一 process の thread（T7）と排他確保の回帰 test（T8、D2） |
| C5 `run()` は生成の失敗で起動を止めない | `run()` | なし | review 確認（Residual Test Gaps。GUI 起動を伴うため自動化しない） |
| C6 CI / local CI の step 定義・依存・生成内容を変えない | 該当なし | local CI `generated-bindings-diff` / `traceability` | 編集禁止 file の diff が無いことを review で確認 |

## Test Plan

Test Design Matrix: [test-matrices/2026-09-22-bindings-export-failure-detection.md](test-matrices/2026-09-22-bindings-export-failure-detection.md)（GA2）。

test は実装と同じ commit に入れてよい。すべて `tempfile::tempdir()` 上で path を引数に取る内部関数を通し、実物の `src/lib/bindings.ts` に触れない。

- targeted tests:
  - T1 成功経路: tempdir 内の出力先へ生成すると `Ok`、出力 file が存在し、`export const CSV_IMPORT_FILE_SIZE_LIMIT` の行を 1 つ含み、行末空白が無く、一時 file が残らない。
- negative tests:
  - T2 一時 file の確保の失敗: 出力先の親 directory を作れない条件（親の位置を同名の file が塞ぐ）にすると `Err`、message に出力先 path を含む。親 directory が存在しないだけでは失敗しない（`specta-typescript 0.0.11` の `export_to` は書込み前に `create_dir_all` を実行する、GA1）。一意な一時 file 名の是正後は、この条件では specta export の前に `acquire_temp_path` で落ちる（closeout で訂正、D12）。
  - T3 置換の失敗: 出力先 path に directory を置いておくと `Err`、その directory は残り、一時 file が残らない。
  - T4 既存 file の保護: 出力先に既存内容の file を置き、出力先の directory を書込み不可にして一時 file を作れなくすると `Err`、出力先の既存内容が変わらない（GA2: 一時 file 名が一意になり名前を事前に塞げないため、GA1 の条件から改める。一時 file の確保 = export 段で落ちる、closeout で訂正、D12。mode で権限を落とす条件は unix 限定）。書込み拒否が成立しない環境（root 実行など）では production 呼出しの前に probe file で書込み可否を判定し、書けてしまう場合は test を対象外として skip する（closeout で追記、D8）。
  - T5 整形の失敗: 存在しない path で `normalize_generated_bindings` が `Err`。
  - T6 定数追記の失敗: 存在しない path で `append_generated_constants` が `Err`。
  - T7 並走: 同じ出力先へ複数 thread から同時に生成すると全部 `Ok`、最終の出力が単独実行の出力と一致し、一時 file が残らない。固定名の一時 file へ戻すと red になる（GA2）。
  - T8（closeout で追加、D5）一時 file の排他確保: 最初の候補（counter=0）を他者が確保済みの状態で生成すると `Ok`、確保済み候補の内容は変わらず存在し続け、生成は次の候補（counter=1）を使う（Final Review broad pass A finding #1 の是正）。
- compatibility checks: AC1（実物の生成結果が差分 0）。
- data safety checks: 該当なし（実データ・DB に触れない）。
- main wiring/integration checks: AC2（CLI の終了コードの実測）。整形・定数追記の段は T5 / T6 で各関数が `Err` を返すことを固定し、合成関数がそれを `?` で伝播することは review で確認する（合成関数の途中へ失敗を注入する seam は足さない）。

## Boundary / Wire Contract

generated bindings に触れるが内容は不変。

- producer: `export_specta_bindings()`
- consumer: frontend の `src/lib/bindings.ts` import、CI / local CI の drift 検査
- wire type: 変更なし
- internal type: 関数の戻り値が `()` から `Result` へ
- precision/range: 該当なし
- round-trip path: 該当なし
- invalid input: 出力不能な path は `Err`
- compatibility: 成功時の出力は byte 単位で不変（AC1）

## Review Focus

- どの段の失敗も CLI の非 0 終了へ届くか（`?` の伝播に抜けがないか、`Ok` を返す早期 return が残っていないか）。
- 失敗時に既存の `bindings.ts` と作業 tree を汚さないか（一時 file の後始末）。
- 並走する生成が互いの一時 file に触れないか、成功した呼出しが自分の完成物だけを公開するか（GA2）。
- `run()` が失敗で起動を止めないか。
- 成功時の生成内容が不変か。
- 保守者が読んで意図が分かるか、seam や error 型を過剰に足していないか。

## Spec Contract

Contract ID: SPEC-BINDINGS-EXPORT-2026-09-22（packet-local）

- C1 型生成のどの段（specta export / 整形 / 定数追記 / 置換）の失敗も `export_specta_bindings()` の `Err` になり、`generate_bindings` は非 0 で終了して完了 message を出さない。
- C2 成功時の `src/lib/bindings.ts` は byte 単位で不変（D-054 ①の定数 export を含む）。
- C3 失敗時は既存の `bindings.ts` の内容を変えず、その呼出しが作った一時 file を残さない（best-effort）。
- C4 一時 file は呼出しごとに一意で、並走する生成が互いの一時 file に触れない。成功した呼出しが公開するのは自分が完成させた内容だけ。
- C5 `run()`（debug build の自動生成）は生成の失敗を警告として出し、起動を続ける。
- C6 CI / local CI の step 定義、依存、生成内容を変えない。

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| C1 | S1 / S4 | T2 / T3 / T4 / T5 / T6 | `?` の伝播、`Ok` を返す早期 return | AC2、AC3 |
| C2 | S2 | T1 | 生成内容の不変 | AC1、local CI `generated-bindings-diff` |
| C3 | S2 | T3 / T4 | 後始末、既存 file の保護 | AC2、AC3 |
| C4 | S2（GA2） | T7 / T8 | 一時 file の所有・排他確保 | AC3、PR #88 review の並走再現（broad pass A、closeout で訂正） |
| C5 | S3 | review | `run()` が止まらない | review |
| C6 | Non-scope | local CI | 編集禁止 file の非侵犯 | AC5 |

## Data Safety

- 実 POS / 店舗データ、DB、backup、log、secret に触れない。commit しない。
- local-only: `.local/ci-evidence/**`、`.local/codex-orders/**`、`src-tauri/target/**`。
- test は `tempfile::tempdir()` 上だけで file を作り、実物の `src/lib/bindings.ts` に触れない。AC2 の実測は `src/lib` の権限を一時的に変えるだけで、必ず戻す。

## Implementation Results

[PR #88](https://github.com/higekiri-w/inventory-system-desktop/pull/88) で実装し squash merge 済み（`7d2002a0`、2026-09-22）。`export_specta_bindings()` を `Result<(), String>` へ変え、specta export / 整形 / 定数追記 / 置換のどの段の失敗も `export_bindings_to` の `Err` として伝播するようにした。生成は同じ directory 内の一時 file（`temp_candidate_path` の候補を `acquire_temp_path` が `create_new` で排他確保、上限 32 回）へ出力してから rename で置換し、失敗時は既存の `bindings.ts` に触れず自分の一時 file だけを best-effort で消す。`generate_bindings` は `Err` で `exit(1)`・完了 message なし、`run()` は警告 (`eprintln!`) して起動を続ける。Gated Amendment 1 で Test Plan T2 の失敗条件を `specta-typescript` の `create_dir_all` 実装に合わせて訂正。Gated Amendment 2 で Final Review round 1（Codex）の finding #2（required gate の合否を変える）を owner が承認し Risk を R2 → R3・Final Review Minimum を 2 へ引上げ、finding #1（固定名の一時 file の並走破損）を一時 file 名の一意化で是正。是正後 head で取り直した Final Review broad pass A（Codex）が、process id が PID namespace 間で一意でないため別 namespace の process が同じ一時 file 名を選び得ることを実測で指摘し、`create_new` による排他確保へ是正した。

## Review Response

Plan Review round 1（Sonnet、P1 0 / P2 2 / P3 2）→ 是正 `9f68abdc` → round 2 closure（同 reviewer、P1/P2 0 / P3 1）→ in-place 是正 → plan-approved。Final Review round 1（Codex、head `355a647d`、PR #88 review）P1 0 / P2 2 / P3 1。finding #2（Risk 判定の見落とし）を採用して Gated Amendment 2、finding #1（固定名の一時 file の並走破損）を一時 file 名の一意化で是正、finding #3（P3）は Matrix の Residual Test Gaps と backlog へ回した。Plan 契約が変わったため round 1 は helper の broad として記録せず、是正後の head で broad 2 本を取り直した: pass A（Codex）P1 0 / P2 1 / P3 1（PID namespace 間の衝突を再現手順つきで実測）、pass B（Opus）P1 0 / P2 0 / P3 6。pass A の P2 を `create_new` による排他確保で是正し、closure（Opus、別 context）で P1 0 / P2 0 / P3 6、合格。残る P3（round 1 分含め計）は非 blocker で、docs の記述訂正（本 closeout、D1〜D12）と follow-up 4 件（backlog）へ回した。hosted CI 全 pass、helper Ready → merge。
- Findings Freeze: frozen after Broad Audit（broad 2 本の完了後）; post-freeze exceptions: none.
