> **親文書**: [FUNCTION_DESIGN.md](../FUNCTION_DESIGN.md)
> **入力ドキュメント**: [architecture/cmd-task-specs.md §CMD-12](../architecture/cmd-task-specs.md), [37-biz-daily-report-import-service.md](37-biz-daily-report-import-service.md)

## 45. CMD-12: 日報取込みコマンド群

### 45.1 モジュール構成

```text
src-tauri/src/
  cmd/
    daily_report_import_cmd.rs
```

CMD-12 は薄いラッパーであり、Z001/Z002/Z005のsource判定、部門照合、重複判定、日報保存判断はBIZ-08へ委譲する。

### 45.2 AppState

既存のCSV preview cacheと同じAppState内に、型を分けた日報preview cacheを追加する。

```rust
struct AppState {
    db: Mutex<Connection>,
    preview_cache: Mutex<HashMap<String, CachedPreview>>,
    daily_report_preview_cache: Mutex<HashMap<String, CachedDailyReportPreview>>,
    register_sd_scan_cache: Mutex<HashMap<String, DailyReportSdScanSnapshot>>, // CMD-12-D1
}
```

DB Mutex と cache Mutex は同時に長時間保持しない。CMD-07と同じく、cache取得/更新は短時間で分ける。

### 45.3 parse_and_validate_daily_report

**関数要求**: フロントエンドから渡された3ファイルをBIZ-08へ渡し、previewとpreview_tokenを返す。

**シグネチャ**:

```rust
#[tauri::command]
fn parse_and_validate_daily_report(
    state: State<AppState>,
    files: Vec<DailyReportSourceFileRequest>,
) -> Result<DailyReportPreviewResponse, CmdError>
```

**入力型**:

```rust
struct DailyReportSourceFileRequest {
    filename: String,
    file_bytes: Vec<u8>,
    #[serde(default)]
    source_path: Option<String>, // D-111 で足す。native dialog で選んだ file の path（UI-07-D16）。省略は None
}
```

`source_path` は BIZ-08 §37.3 手順 1a が、選んだ file がレジの SD の上にあるかを確かめるためだけに使う（CMD は path を解釈しない）。CMD は `DailyReportInputFile { filename, bytes, sd_relative_path: None, source_path: source_path.map(PathBuf::from) }` を作って渡す。空文字の `source_path` は `None` と同じに扱う。

**出力型**:

```rust
struct DailyReportPreviewResponse {
    preview_data: DailyReportPreviewData,
    preview_token: String,
}
```

`DailyReportPreviewData`、`DailyReportImportResult`、`DailyReportRollbackResult`、`DailyReportImport` は [37.2](37-biz-daily-report-import-service.md#372-型定義) / `sales_repo` のDTOを所有元とする。CMD-12実装では、UIに返すDTOに `specta::Type` を付けてTauri wire型として公開する。`CachedDailyReportPreview` はAppState内部専用であり、wire型にはしない。

`DailyReportPreviewData.duplicate_check` のwire形状は `status: DailyReportDuplicateStatus` と `same_date_imports: Vec<SameDateDailyReportImportSummary>` で固定する。summaryは `id / source_filenames / gross_amount / net_amount / imported_at` を持ち、順序は `imported_at DESC, id DESC`。hashと単一parent fieldは公開しない。

**処理ステップ**:
1. `files.len()` が3以外なら `CmdError.kind="validation"`。
2. 各ファイルが20MBを超える場合は `CmdError.kind="validation"`。
3. DB接続を取得する。
4. BIZ-08 `parse_and_validate_daily_report` を呼ぶ。
5. 成功時、UUID preview_tokenを生成し `daily_report_preview_cache` に保存する。
6. preview responseを返す。

### 45.4 commit_daily_report_import

**シグネチャ**:

```rust
#[tauri::command]
fn commit_daily_report_import(
    state: State<AppState>,
    app: tauri::AppHandle, // D-111 で足す。Tauri が注入し、wire（bindings の引数）は変わらない
    preview_token: String,
    additional_import_confirmed: bool,
) -> Result<DailyReportImportResult, CmdError>
```

`app` は Tauri が注入する。手順 3a の `app_data_dir` は command が test 用の `commit_daily_report_import_with_state`（`src-tauri/src/cmd/daily_report_import_cmd.rs`）を呼ぶ前に取り、解決済みの `app_data_dir: &Path` を引数で渡す（取れなければ cache に触れずに `internal`。test は一時 directory を渡す）。このため手順 3a は番号に反して手順 1 より前に行い、観測できる差は token が不正（手順 1〜3 の失敗）かつ `app_data_dir` を取れないときに、手順 1〜3 の error でなく手順 3a の `internal` が返る（期限切れの token も cache から消えない）ことだけである（`src-tauri/src/cmd/daily_report_import_cmd.rs` の `commit_daily_report_import`）。

**処理ステップ**:
1. preview_tokenのUUID形式を検証する。
2. `daily_report_preview_cache` からcached previewを取得する。
3. cache miss / 期限切れは `CmdError.kind="import_error"`。
3a. `app.path().app_data_dir()` で写しの置き場所を取る（実行は手順 1 より前。上の注記。`settings_cmd.rs` の `get_backup_dir` / `restore_backup` と同じ取り方）。失敗は `CmdError::internal("アプリデータの保存先を取得できませんでした", e)` で、cache は残す。
4. DB接続を取得する。
5. `additional_import_confirmed` と手順 3a の `app_data_dir` をBIZ-08 `commit_daily_report_import` へ渡す（写しを書くかは BIZ-08-D5 が決め、CMD は書く規則を持たない）。Rust wire名はこのsnake_case、TypeScript生成名は `additionalImportConfirmed` とする。
6. 成功時、cacheからtokenを削除する。
7. 通常の失敗時はcacheを残して再試行可能にする。ただしBIZが `同日の取込み状況が変わりました。再度プレビューしてください` を返した場合はtokenを削除し、新しいpreview/tokenを要求する。

### 45.5 rollback_daily_report_import

```rust
#[tauri::command]
fn rollback_daily_report_import(
    state: State<AppState>,
    daily_report_import_id: i64,
) -> Result<DailyReportRollbackResult, CmdError>
```

BIZ-08 `rollback_daily_report_import` を呼ぶ。指定IDだけを取消し、同日の他importは残す。成功時の frontend query invalidation は [D-052](../decision-log.md) C10 と `src/lib/invalidation-contract.ts` を正本とする。sale_records / inventory_movements / products は変わらない。

### 45.6 list_daily_report_imports

```rust
#[tauri::command]
fn list_daily_report_imports(
    state: State<AppState>,
    page: i64,
    per_page: i64,
    date_from: Option<String>,
    date_to: Option<String>,
) -> Result<PaginatedResult<DailyReportImport>, CmdError>
```

**入力ガード**:
- page < 1 → `CmdError.kind="validation"`
- per_page < 1 → `CmdError.kind="validation"`
- per_page > 100 → `CmdError.kind="validation"`

status filter は第1スライスでは公開しない。BIZ-08のquery型には内部拡張用に `status` を残し、CMD-12からは `None` を渡す。

### 45.6a scan_register_sd（CMD-12-D1、D-111）

**関数要求**: レジの SD を BIZ-08 §37.9 で読み、候補の一覧と scan_token を返す。

```rust
#[tauri::command]
fn scan_register_sd(
    state: State<AppState>,
    selected_path: Option<String>, // None = 自動で探す。Some = 利用者が folder の選択で選んだ path
) -> Result<RegisterSdScanResponse, CmdError>

struct RegisterSdScanResponse {
    scan: DailyReportSdScan, // 37 §37.9
    scan_token: String,
}
```

**処理ステップ**:
1. `selected_path` を BIZ-08 の `DailyReportSdSelection`（37 §37.9。`None` → `Auto`、`Some` → `Selected(PathBuf)`）にする。CMD は IO-09 の型を使わない（`src-tauri/tests/architecture_test.rs` の LAYER_RULES で cmd → io は禁止）。空文字は `CmdError.kind="validation"`。
2. DB接続を取得し、BIZ-08 `scan_register_sd_daily_reports(conn, selection, PC の今日の日付)` を呼ぶ。SD を読む間（窓の範囲の小さな file と照合。利用者は 1 人）は DB の Mutex を持ったままでよい。
2a. 成功時、同じ DB 接続を持ったまま BIZ-04 `refresh_plu_register_snapshot_from_sd(conn, &snapshot.plu_register_z004s)` を呼ぶ（PLU の占有の照合、[33 §16.3](33-biz-plu-export-service.md)、D-117 (3)）。`Err` は scan の失敗にせず、`tracing::warn!` で記録して手順 3 へ進む（response は変えない）。照合の後、`plu_register_z004s` を空にしてから手順 3 で snapshot を cache に入れる（Z004 の bytes を cache に残さない）。本手順は PLU の書出しの見直しの runtime の lane が実装する（37 §37.9 手順 10 と同じ lane）。SD 直読みの runtime の lane（D-111）は手順 1・2・3・4 を実装し、手順 2 の後に手順 3 へ進む。
3. 成功時、UUID の scan_token を作り、`register_sd_scan_cache` を空にしてから snapshot を入れる（同時に持つ scan は最新の 1 つだけ）。
4. response を返す。

### 45.6b parse_and_validate_daily_report_from_sd（CMD-12-D1）

```rust
#[tauri::command]
fn parse_and_validate_daily_report_from_sd(
    state: State<AppState>,
    scan_token: String,
    candidate_key: String,
) -> Result<DailyReportPreviewResponse, CmdError>
```

**処理ステップ**:
1. scan_token の UUID 形式を検証する。
2. `register_sd_scan_cache` から snapshot を取得する。miss または作成から 30 分超は `CmdError.kind="import_error"`、message `SD を読んでから時間がたちました。もう一度 SD を読んでください。`。UI はこの kind・message で分岐せず、SD から来たほかの失敗と同じく一覧（`sd_list`）へ戻し、一覧の「SD から読む」で読み直させる（55 UI-07-D15 (4)・(5)）。
3. `files_by_candidate[candidate_key]` が無ければ `CmdError.kind="validation"`（取り込めない候補）。
4. 以降は §45.3 の手順 3〜6 と同じ（BIZ-08 `parse_and_validate_daily_report` に 3 本を `sd_relative_path` つきで渡し、preview_token を返す）。commit は §45.4（手順 3a で取った `app_data_dir` を BIZ-08 に渡す。写しの置き場所、BIZ-08-D5）。snapshot は消さない（同じ scan から別の候補を続けて取り込める）。

**CMD-12-D1**: CMD は scan の snapshot を AppState に置いて渡すだけで、SD の探し方・候補の規則・状態の判定を持たない（BIZ-08-D3）。CMD は IO-09 を直接呼ばない（ARCHITECTURE のレイヤー間の呼び出し原則、`src-tauri/tests/architecture_test.rs`）。snapshot を AppState に置くのは、SD を読み終えたらすぐレジへ戻せるようにするため（SD-18）。棄却案: scan の結果の bytes を UI へ返して UI から §45.3 を呼ぶ（取り込まない Z004 等は持たないが、日報だけでも wire に生バイトを往復させ、UI が束を組める余地を作る）、preview のたびに SD を読み直す（SD を差したままにする必要がある）。

### 45.6c 一括取込みと EJ の控えのコマンド（CMD-12-D2、D-121、proposed・未実装）

後続の runtime の lane（backlog の「SD の一括取込みと EJ の控えの runtime（A）」「Z004 を SD の一括取込みに入れる runtime（B）」）が足す。§45.6a・§45.6b（D-111、SD 直読みの runtime が実装する）は変えず、その merge の後に scan に次を足す: (A) BIZ-08 の scan が EJ も読み、`DailyReportSdScan` の `ej`（`RegisterSdEjSummary`）と snapshot の `ej_files`・`ej_unreadable_files` を返す（37 §37.10 の「§37.9 に足すもの」）。CMD は今どおり scan の結果を返し snapshot を cache に置くだけで、EJ を読まない。(B) §45.6a 手順 2a の後に Z004 の bytes を空にせず、相対 path つきで snapshot の `z004_files` に残す（PLU の照合は今どおり。BIZ-08 の scan は読めなかった Z004 の相対 path を `z004_unreadable_files` に返す。37 §37.10 の「§37.9 に足すもの」）。`DailyReportSdScan` の `z004_new_count`（hash が active な取込みに無い Z004 の数）と `z004_unreadable_count`（読めなかった Z004 の数）を UI のボタンの有効化に使う（どちらも 37 §37.10。同じ bytes の Z004 は BIZ が 1 つにまとめた後に数える）。32 の新契約の検査は scan では回さない（数が多めに出て、押しても何も入らない側に倒れるだけで害が無い）。

```rust
#[tauri::command]
fn commit_register_sd_bulk_import(
    app: tauri::AppHandle,
    state: State<AppState>,
    scan_token: String,
) -> Result<RegisterSdBulkResponse, CmdError> // 下の wire の型。37 §37.10 の内部の型から CMD が変える

struct RegisterSdBulkResponse { items: Vec<RegisterSdBulkItemDto> }
struct RegisterSdBulkItemDto { source: RegisterSdBulkSource, outcome: RegisterSdBulkOutcomeDto } // source は 37 の enum をそのまま公開
enum RegisterSdBulkOutcomeDto {
    ImportedDailyReport(DailyReportImportResult),
    ImportedZ004(ImportResult),   // (B)
    ImportedEj(EjImportResult),
    Skipped { message: String },
    NeedsIndividualConfirmation,
    Failed { kind: CmdErrorKind, message: String }, // BizError に §45.7 の変換を当てた CmdError の kind（生成 enum、D-061 (d)）と message
    NotAttempted,
}

#[tauri::command]
fn import_ej_files(
    app: tauri::AppHandle,
    state: State<AppState>,
    files: Vec<EjFileRequest>, // { filename: String, file_bytes: Vec<u8> }。手で選んだ EJ。1 file 20MB まで
) -> Result<EjImportResult, CmdError>

#[tauri::command]
fn list_ej_records(state: State<AppState>, date: Option<String>) -> Result<EjRecordDay, CmdError>

#[tauri::command]
fn find_ej_records_by_no(state: State<AppState>, record_no: String) -> Result<Vec<EjRecordRef>, CmdError>

#[tauri::command]
fn get_ej_record(state: State<AppState>, id: i64) -> Result<EjRecordDetail, CmdError>
```

**処理ステップ（`commit_register_sd_bulk_import`）**:
1. scan_token の UUID の形を検証し、`register_sd_scan_cache` から snapshot を取る。無い・30 分超は §45.6b 手順 2 と同じ kind・文。
2. `app_data_dir` を §45.4 手順 3a と同じく取る。
3. DB の接続を取り、BIZ-08 `commit_register_sd_bulk_import(conn, &snapshot, app_data_dir)` を呼ぶ。項目の成否は response の中で返し、`Err` だけを §45.7 で変える。
4. 成功の後、日報・Z004 の取込みの成功と同じ invalidation の対象（D-052 の daily / product / monthly / list / detail）と、控えの query の prefix `queryKeys.ejRecords.root()`（`["ej-records"]`。UI-09c の日の一覧・番号で探す・控えの文の query をこの下に置く）を UI が無効にする（UI-07-D17）。(A) が `src/lib/query-keys.ts` にこの key を、`src/lib/invalidation-contract.ts` に D-052 の entry を足す。snapshot は消さない（やり直しで同じ token を使える。確定済みの項目は飛ばされる。BIZ-08-D7）。

**処理ステップ（`import_ej_files`）**: 各 file の大きさを検証し（20MB を超えたら `validation`）、`EjSourceFile { filename, sd_relative_path: None, bytes }` にして BIZ-08 `import_ej_records` を呼ぶ。手で選んだ file は写さない（37 §37.11 手順 4）。SD の上の file かを確かめない（最後の記録を保留するかは BIZ が file 名で決める。37 §37.11 手順 2）。成功の後に UI が無効にするのは、上の手順 4 の控えの prefix `queryKeys.ejRecords.root()` だけ（日報・在庫・売上を変えない。D-052 の entry は (A) が足す）。

**CMD-12-D2**: 一括取込みと EJ の控えの command も CMD-12 に置く（同じ scan cache と SD の経路を使うため。閲覧の 3 command は読むだけ）。CMD は項目の順・飛ばす規則・EJ の比べ方を持たない（BIZ-08-D7・D9）。BIZ の `RegisterSdBulkOutcome::Failed(BizError)` は、CMD が §45.7 の `impl From<BizError> for CmdError` を 1 か所で当てて `Failed { kind, message }` にする（`SourceCopyFailed` は §45.7 の固定の文、`DatabaseError` は固定の文、`ImportError` などは BIZ の文のまま。io error の kind は診断にだけ出す。文字列の照合で種類を決めない）。有限の値は wire でも生成 enum にする（D-061・D-064）: `Failed` の `kind` は `CmdErrorKind`、`EjRecordSummary` の `kind`・`mode` は 37 §37.11 の `EjRecordKindCode`・`EjRecordModeCode`（response だけ）。`String` に戻さない。登録は §45.8 と同じく `collect_commands!` と `generate_handler!` の両方。

### 45.7 CmdError変換

| BIZ-08 error | CmdError.kind | message |
|---|---|---|
| ImportError(msg) | import_error | msgをそのまま使用 |
| IdempotencyConflict(msg) | idempotency_conflict | msgをそのまま使用 |
| ValidationFailed(msg) | validation | msgをそのまま使用 |
| NotFound(msg) | not_found | msgをそのまま使用 |
| DatabaseError(_) | internal | データベースエラーが発生しました。もう一度お試しください |
| SourceCopyFailed(kind)（D-111、BIZ-08-D5） | internal（`CmdError::internal`、`error_id` つき。io error の kind は診断にだけ出す） | SD から読んだファイルの写しを PC に保存できなかったため、取り込みませんでした。PC の空き容量を確かめて、もう一度取り込んでください。 |

`SourceCopyFailed` を `import_error` にしない: UI は commit の `import_error` を最初へ戻す（`src/features/daily-report-import/hooks/useDailyReportImportFlow.ts` の `decideRecoverTo`）ので、`import_error` にすると preview と token を失う。`internal` なら `decideRecoverTo` が `preview` を返し、§45.4 手順 7 の「通常の失敗は cache を残す」で token も残るので、同じ preview・同じ token で再試行できる（55 の日報取込みの利用者フローの手順 2）。`impl From<BizError> for CmdError`（`src-tauri/src/cmd/mod.rs`）に分岐を足す。

### 45.8 生成bindings

SPEC-SDI-D3を実装する同一commitでは `#[specta::specta]` と `specta::Type` deriveを維持し、`DailyReportDuplicateStatus` / `DailyReportDuplicateCheck` / `SameDateDailyReportImportSummary` / commit引数を含む `src/lib/bindings.ts` をgeneratorで再生成する。生成物の手編集は禁止する。D-111 の `scan_register_sd` / `parse_and_validate_daily_report_from_sd` は、`lib.rs` の specta の `collect_commands!`（今は `src-tauri/src/lib.rs` の `export_bindings_to`）と runtime の `tauri::generate_handler!`（同 `run`）の両方に登録し（どちらか一方だけだと、`scripts/check-command-drift.sh`〈`bash scripts/doc-consistency-check.sh` の Command registry drift、`D=` 宣言・`H=` generate_handler・`S=` collect_commands・`T=` bindings の 4 つの集合の一致〉が失敗する。件数は正本に書かず、この 4 つの集合が一致することだけを契約にする）、`DailyReportSdScan` / `DailyReportSdCandidate` / `DailyReportSdCandidateStatus` / `RegisterSdScanResponse` を含めて再生成する。既存の command と DTO の wire は、`DailyReportSourceFileRequest` に省略可の `source_path`（§45.3）を足すほかは変えない（足す field は `#[serde(default)]` で、省いた呼出しは今どおり PC 上の file として通る。`settlement_no` と `source_files_json` の `sd_relative_path`・`copy_path` は内部の cache と DB だけで、wire に出さない。37 §37.4 手順 6）。

対象:
- `parse_and_validate_daily_report`
- `commit_daily_report_import`
- `rollback_daily_report_import`
- `list_daily_report_imports`

### 更新履歴

| 日付 | PR | 内容 |
|---|---|---|
| 2026-08-16 | PR #79 | SPEC-SDI-D3/D4: same-date summary DTO、`additional_import_confirmed`、snapshot mismatch時のtoken破棄、per-import rollback、bindings再生成義務を正本化。 |
| 2026-10-06 | sd-direct-read（design、D-111） | CMD-12-D1: `scan_register_sd` と `parse_and_validate_daily_report_from_sd`、AppState の scan cache。 |
| 2026-10-17 | sd-direct-read-runtime（起票） | §45.6a 手順 2a（D-117 の占有の照合）の実装の lane を明記した。 |
| 2026-10-07 | sd-direct-read（Final Review の P3 の是正） | §45.6b 手順 2 の期限切れを UI が一覧へ戻す扱いを 55 UI-07-D15 に合わせた。§45.8 から command registry の件数を消し、4 つの集合の一致だけを契約にした。 |
| 2026-10-07 | sd-direct-read（Plan Review round 3 の後の同型指摘の一括是正） | §45.7 に `SourceCopyFailed` → `internal`（写しの失敗の後に UI が preview と token に戻れる）。§45.8 に新しい 2 command の `generate_handler!` への登録と command registry の gate。 |
| 2026-10-07 | sd-direct-read（Plan Review round 2 の是正） | §45.3 の `DailyReportSourceFileRequest.source_path`（手で選んだ SD 上の file を SD の入力にする、37 §37.3 手順 1a）。§45.6a の選択の enum を BIZ-08 の `DailyReportSdSelection` にした（cmd → io の禁止）。§45.8 の wire の契約を足す field に合わせた。 |
| 2026-10-07 | sd-direct-read（Plan Review round 1 の是正） | §45.4 の commit に `app: tauri::AppHandle` と `app_data_dir` の取り方（手順 3a）を足した（BIZ-08-D5 の写しの置き場所）。 |
| 2026-10-17 | sd-bulk-import-design（design、D-121） | §45.6c（CMD-12-D2）: 一括取込みと EJ の控えの command と、scan に後から足す EJ の数え・Z004 の bytes を proposed で足した。§45.6a・§45.6b は変えていない。Plan Review round 2 の是正で、`Failed` の kind と EJ の kind・mode を生成 enum にし、scan の `ej_unreadable_files` と手で選んだ EJ の保留の判定の置き場を書いた。Plan Review round 3 の是正で、一括取込みと手で選ぶ EJ の取込みの後に無効にする控えの query の key を書いた。Final Review の是正で、(B) の scan が返す `z004_unreadable_files` を書いた。あわせて `z004_unreadable_count` を書いた。 |
| 2026-10-24 | sd-direct-read-runtime（実装、PR #172） | §45.3・§45.4・§45.8 の「今の実装」の注記と lane の注記を実装の記述に直した。§45.4 手順 3a の `app_data_dir` は command が `commit_daily_report_import_with_state` を呼ぶ前に取る。 |
