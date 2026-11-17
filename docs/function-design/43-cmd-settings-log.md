## CMD-11残り: 設定・ログ・バックアップ・画像コマンド群

### 計数の置き場と在庫の求め直しの契約（D-118。(a) は実装済み、(b) は proposed・未実装）

(a) で入ったのは、写しを記録から求め直す形と、整合性の補正が求め直しの結果を写しへ書く形（`biz/integrity_service.rs` の `recompute_stock` の呼出し、`biz/stock_ledger.rs`）。本節の残りは (a) でも (b) でも同じ。[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md) は計数の context（token・DB 世代）を持たないので、§43.9 の DB 接続の交換で失効させるものは無い（旧 ADR〈時点証拠〉の context の失効の契約は置き換えた）。写しは DB の中で記録から求め直すので、復元した DB では復元した記録の写しがそのまま正しい。fix_integrity の command 本体は 42 にあり、写しの書込みは 36 → 31 の求め直しの責務である。

> CMD-11の整合性チェック部分（run_integrity_check, fix_integrity）は `42-cmd-sales-stocktake.md` に記載済み。
> 本セクションは設定CRUD・操作ログ閲覧・バックアップ・画像保存コマンドを扱う。

### 43.1 モジュール構成

```
src-tauri/src/
  cmd/
    mod.rs             -- pub mod settings_cmd を追加
    settings_cmd.rs    -- 設定・ログ・バックアップ・画像コマンド（本セクション）
    integrity_cmd.rs   -- 既存（CMD-11の整合性チェック部分）
  biz/
    system_service.rs  -- BIZ-09 設定・操作ログロジック（38-biz-system-service.md）
  lib.rs               -- invoke_handler に8コマンドを追加
```

**層経路（D-060）**: 層経路の正本は `ARCHITECTURE.md`「レイヤー間の呼び出し原則」。設定・操作ログ系 4 command は `biz::system_service`（BIZ-09）経由の標準経路、backup/restore 系 5 command は CMD→MNT 正規経路（DB 接続所有権の交換を要する保守 orchestration 限定）、画像保存は base64 decode（CMD の wire 型変換）+ BIZ validation で構成する。

**実装状態**: D-060 に基づき、設定・操作ログ系 4 command は BIZ-09 経由、画像保存は BIZ-02 経由で実装されている（packet: 2026-07-31-settings-service-boundary-design）。

---

### 43.2 型定義

#### リクエスト型

```
#[derive(Debug, serde::Deserialize, specta::Type)]
struct UpdateSettingRequest {
    key: String,
    value: String,
}

#[derive(Debug, serde::Deserialize, specta::Type)]
struct LogQuery {
    page: u32,
    per_page: u32,  // system_repo がD-031共有定数により200クランプ
    operation_type: Option<String>,
    start_date: Option<String>,  // UI-11c-D3。YYYY-MM-DD。JST暦日 start inclusive
    end_date: Option<String>,    // UI-11c-D3。YYYY-MM-DD。JST暦日 end-next-day exclusive
}
```

`start_date` / `end_date` は [74-ui-operation-logs.md](74-ui-operation-logs.md) §74.4 の Design Phase で追加が確定した。既存呼び出し元は両方 `None` を渡すことで現行動作を完全維持する（生成 binding 再生成後、既存 Rust テスト・フロントエンド呼び出しはこの2フィールドを明示的に渡す必要がある）。

```

#[derive(Debug, serde::Deserialize, specta::Type)]
struct RestoreBackupRequest {
    backup_path: String,
}

#[derive(Debug, serde::Deserialize, specta::Type)]
struct SaveImageRequest {
    image_base64: String,     // Base64エンコードされた画像データ
    extension: String,        // "jpg", "png" 等
}
```

注意: `extension` は generated binding 上も `string` として出る。許可拡張子の UI 側候補は `jpg|jpeg|png|gif|webp` に絞るが、最終 validation は BIZ 層（`biz::inventory_service`、D-060 (c)、[31-biz-inventory-service.md](31-biz-inventory-service.md)）が `String` 値を検証して validation error を返す。IO 層の拡張子 check は防御として維持する（DB CHECK 制約と同型、正常系で到達しない）。

#### レスポンス型

```
#[derive(Debug, serde::Serialize, specta::Type)]
struct SaveImageResponse {
    relative_path: String,    // DBに格納する相対パス
}
```

注意: `AppSetting`, `OperationLog`, `PaginatedResult`, `BackupResult`, `BackupInfo` は各モジュールで定義済みの型を再利用する。

---

### 43.3 get_settings

**シグネチャ**:
```
#[tauri::command]
fn get_settings(state: State<AppState>) -> Result<Vec<AppSetting>, CmdError>
```

**処理**: `biz::system_service::get_all_settings(conn)` を呼び出して返す（D-060 (b)。関数契約は 38-biz-system-service.md）。

---

### 43.4 update_setting

**シグネチャ**:
```
#[tauri::command]
fn update_setting(state: State<AppState>, request: UpdateSettingRequest) -> Result<(), CmdError>
```

**処理**: `biz::system_service::upsert_setting(conn, &request.key, &request.value)` を呼び出す（D-060 (b)）。

---

### 43.5 list_logs

**シグネチャ**:
```
#[tauri::command]
fn list_logs(state: State<AppState>, query: LogQuery) -> Result<PaginatedResult<OperationLog>, CmdError>
```

**処理**: CMD は `LogQuery` の型変換のみを行い、`biz::system_service::list_operation_logs(conn, &query)` を呼び出す（D-060 (b)/(c)）。日付 validation（ASCII strict 形式・実在暦日・範囲順序。[74-ui-operation-logs.md](74-ui-operation-logs.md) §74.4.2）は BIZ 層（BIZ-09）が所有し、条件・文言・field は移動前と不変とする。per_page の上限クランプ（200）は従来どおり IO 層（system_repo）が D-031 の `PAGINATION_MAX_PER_PAGE = 200` で実行し、レスポンスの per_page もクランプ後の値を返す。

**エラーハンドリング（追加分、UI-11c-D2/D3）**: validation は BIZ 層（BIZ-09）が所有し、`BizError::ValidationFailed` を CMD が `From<BizError>` で変換して返す（D-060 (c)）。wire 契約（kind / message / field）は移動前と不変:
- `start_date` / `end_date` が strict `YYYY-MM-DD` 形式でない、または実在しない暦日 → `CmdError { kind: "validation", message: "開始日・終了日はYYYY-MM-DD形式で入力してください" }`
- 両方 `Some` かつ `start_date > end_date` → `CmdError { kind: "validation", message: "開始日は終了日と同じ日か、それより前の日付にしてください" }`
- 両方 `None` の場合は既存動作を完全維持する（既存呼び出し元・既存テストへの影響なし）。

---

### 43.5.1 list_log_operation_types（PR #164で実装済み、UI-11c-D4）

**シグネチャ**:
```
#[tauri::command]
fn list_log_operation_types(state: State<AppState>) -> Result<Vec<String>, CmdError>
```

**処理**: `biz::system_service::list_distinct_operation_types(conn)` を呼び出して返す薄いラッパー（D-060 (b)）。フィルタ・ページングを持たない。

**目的**: [74-ui-operation-logs.md](74-ui-operation-logs.md) §74.5 の operation_type filter 候補は、現在ページや現在の filter 済み結果から生成してはならない（Missing UI item 4）。保持中の operation_logs 全体から distinct な operation_type を返すことで、canonical registry（frontend 日本語ラベル辞書）と突き合わせる際の実在値ソースにする。

**非目的**: 日本語ラベル・カテゴリ分類は持たない（frontend `operation-type-labels.ts` の責務）。

---

### 43.6 create_backup

**シグネチャ**:
```
#[tauri::command]
fn create_backup(state: State<AppState>, app_handle: tauri::AppHandle) -> Result<BackupResult, CmdError>
```

**処理ステップ**:
1. `app_handle.path().app_data_dir()` でapp_data_dirを取得
2. `backup_dir = app_data_dir.join("backups")` でバックアップディレクトリ決定
   - `backup_path`設定があればそちらを優先（空文字ならデフォルト使用）
3. `mnt::backup::pin_backup_dir(&backup_dir)`（71 §71.4 の「書く先の固定」）で書く先を確かめ、`Err` なら下の固定の文の `validation` を返す（D-114。手動の backup で原因を名指す）
4. `mnt::backup::create_backup(conn, &backup_dir)` を呼び出す（中で同じ固定と判定をもう一度行う）

| `BackupDirError` | `CmdError` | 固定の文 |
|---|---|---|
| `Unpinnable` | `validation` | 「この保存先は PC の中の folder として確かめられません。PC の中の folder を選び直してください」 |
| `RegisterSd { sd_folder }` | `validation` | 「この保存先はレジの SD カードの上にあります（{sd_folder} が見つかりました）。レジの SD の写しを PC に置いているなら、写しを drive の直下でなく folder の中へ移してください。そうでなければ「保存先を選ぶ」で PC の中の別の folder を選び直してください」（`{sd_folder}` は 71 §71.4 の `pin_backup_dir` が見つけた `{祖先}\CASIO\SR500_550_4000` の path。判定に使った path の形で、junction の先だけで見つかったときは `\\?\Volume{GUID}\…` の形になる〈保存先を選び直す直し方は同じ〉） |

---

### 43.7 check_auto_backup

**シグネチャ**:
```
#[tauri::command]
fn check_auto_backup(state: State<AppState>, app_handle: tauri::AppHandle) -> Result<bool, CmdError>
```

**処理**: `mnt::backup::check_auto_backup(conn, &backup_dir)` を呼び出す。フロントエンドの `setInterval(60秒)` から呼ばれる。

---

### 43.8 list_backups

**シグネチャ**:
```
#[tauri::command]
fn list_backups(state: State<AppState>, app_handle: tauri::AppHandle) -> Result<Vec<BackupInfo>, CmdError>
```

**処理**: `resolve_backup_dir`でbackup_dirを決定し、`mnt::backup::list_backups(&backup_dir)` を呼び出す。
※ `backup_path`設定の読み取りにDB接続が必要なため`state`引数を追加。

---

### 43.8.1 get_effective_backup_dir

**関数要求**: `backup_path` 未設定時にアプリが実際にバックアップを保存するディレクトリ（アプリ既定フォルダ）を利用者へ提示するための読み取り専用コマンド。UI-11b の常時表示 follow-up（PR #144 L3 / Fable 裁定起源）で追加。

**シグネチャ**:
```
#[tauri::command]
fn get_effective_backup_dir(state: State<AppState>, app_handle: tauri::AppHandle) -> Result<String, CmdError>
```

**処理ステップ**:
1. 既存ヘルパ `get_backup_dir(&conn, &app_handle)`（§43 冒頭ヘルパー、内部で `mnt::backup::resolve_backup_dir` を呼ぶ）を呼び出す。
2. 返された `PathBuf` を `to_string_lossy().to_string()` で文字列化して返す。

**非目的**: バリデーション・パス加工・設定更新は行わない。CMD層は薄いラッパーのまま。

---

### 43.8.2〜43.8.7 PC の外の控えと控えの確かめ（D-114、MNT-01-D8・D9）

6 command とも MNT-01（71 §71.11・§71.12）を呼ぶだけの薄い入口で、規則を持たない（CMD → MNT の直接呼出しは backup 系の正規経路、D-060 (a)）。DB の Mutex は設定（`backup_path`）の読取りの間だけ持ち、媒体へ写す・file を開く前に放す（写しの数秒の間、他の command を止めない）。

**実行の thread**: 6 command は `#[tauri::command(async)]` で定義し、main thread で copy・読み戻し・SHA-256・`quick_check`・drive の列挙をしない。Tauri v2 の公式資料: async の無い command は `#[tauri::command(async)]` で定義しない限り main thread で実行される（https://v2.tauri.app/develop/calling-rust/ ）。D-114 の前の repo の command はすべて sync で main thread で動いていたので、その形に倣うと写しの間に画面が固まる（`inspect_backup` の登録の後は、async の無い既存の command だけが main thread で動く）。関数は sync のまま `(async)` を付ける形にし（`async fn` にして `State` の借用の制約を持ち込まない）、`#[specta::specta]` と `collect_commands!` / `generate_handler!` への登録は他の command と同じ。既存の `create_backup`・`check_auto_backup` の実行 thread は変えない（DB の Mutex の中で動く既存の契約。見直しは D-114 の範囲外）

| § | command | 入力 → 出力 | 処理 |
|---|---|---|---|
| 43.8.2 | `prepare_offsite_medium` | `PrepareOffsiteMediumRequest { selected_path: String }` → `OffsiteMediumView` | `app_data_dir` を取り、`get_backup_dir` で `backup_dir` を読んで lock を放し、`mnt::offsite::prepare_offsite_medium(&app_data_dir, &backup_dir, Path::new(&selected_path))` を呼ぶ（`backup_dir` は 71 §71.11.2 手順 6 の写しの元）。空の `selected_path` は `validation` |
| 43.8.3 | `check_offsite_backup` | なし → `OffsiteCheckResult` | `backup_dir` を読んで lock を放し、`mnt::offsite::check_offsite_backup(&app_data_dir, &backup_dir)`。共通レイアウトの確認（UI-11b-D16）が `checkAutoBackup` の後に呼ぶ |
| 43.8.4 | `get_offsite_backup_status` | なし → `OffsiteBackupStatus` | `get_backup_dir` で `backup_dir` を読んで lock を放し、`mnt::offsite::offsite_status(&app_data_dir, &backup_dir, 今日のローカル日付)`（`backup_dir` は写す元と同じで、`local_backup_old` の材料。71 §71.11.5） |
| 43.8.5 | `inspect_backup` | `InspectBackupRequest { backup_path: String }` → `BackupInspection` | `mnt::backup::inspect_backup(Path::new(&backup_path))`。DB を使わない。空の `backup_path` は `validation` |
| 43.8.6 | `reset_offsite_state` | なし → `OffsiteCheckResult` | DB の Mutex を取り、`get_backup_dir` で `backup_dir` を読み、既存の `create_backup_at(conn, &backup_dir)`（§43.6 と同じ書く先の固定・SD の判定・backup の作成と error 写像。操作ログ `backup_create`）を行ってから lock を放し、`mnt::offsite::reset_offsite_state(&app_data_dir, &backup_dir)`（71 §71.11.5a）を呼ぶ。作成が失敗したら MNT の作り直しを呼ばず、§43.6 と同じ写像で返す（記録に触れない）。バックアップ画面の「記録を作り直す」（UI-11b-D14）が呼ぶ |
| 43.8.7 | `stop_offsite_backup` | なし → `null` | `mnt::offsite::stop_offsite_backup(&app_data_dir)`（71 §71.11.5b）。DB を使わない（lock を取らない）。PC の中の backup に触れない。バックアップ画面の「PC の外の控えをやめる」（確認の後、UI-11b-D14）が呼ぶ。作り直しの command に mode を持たせず別にしたのは、やめるには backup の作成も媒体も要らず、mode にすると request DTO と CMD の分岐が増えるため |

```
#[tauri::command(async)]
fn prepare_offsite_medium(state: State<AppState>, app_handle: tauri::AppHandle, request: PrepareOffsiteMediumRequest) -> Result<OffsiteMediumView, CmdError>
#[tauri::command(async)]
fn check_offsite_backup(state: State<AppState>, app_handle: tauri::AppHandle) -> Result<OffsiteCheckResult, CmdError>
#[tauri::command(async)]
fn get_offsite_backup_status(state: State<AppState>, app_handle: tauri::AppHandle) -> Result<OffsiteBackupStatus, CmdError>
#[tauri::command(async)]
fn inspect_backup(request: InspectBackupRequest) -> Result<BackupInspection, CmdError>
#[tauri::command(async)]
fn reset_offsite_state(state: State<AppState>, app_handle: tauri::AppHandle) -> Result<OffsiteCheckResult, CmdError>
#[tauri::command(async)]
fn stop_offsite_backup(app_handle: tauri::AppHandle) -> Result<(), CmdError>
```

**エラーの写像**（`CmdError.kind` の値は既存の `validation` / `internal` だけを使い、`CmdErrorKind` に値を足さない）:

| MNT の失敗 | `CmdError` | 固定の文（UI が出す。UI-11b-D14〜D16） |
|---|---|---|
| `OffsiteError::NotRemovable` | `validation` | 「USB メモリを選んでください（この場所は PC の外の控えに使えません）」 |
| `OffsiteError::RegisterSd { sd_folder }` | `validation` | 「これはレジの SD カードです（{sd_folder} が見つかりました）。控えは USB メモリに作ってください。USB メモリにレジの SD の写しを置いているなら、写しを drive の直下でなく folder の中へ移してください」 |
| `OffsiteError::StorageFull` | `internal` | 「USB メモリの空きが足りません」 |
| `OffsiteError::VerifyMismatch` / `Io` | `internal` | 「PC の外の控えを写せませんでした」 |
| `OffsiteError::Redirected` | `validation` | 「この USB メモリの InventoryBackup フォルダは別の場所へのリンクのため使えません」 |
| `OffsiteError::BackupDirIsMediumFolder` | `validation` | 「PC の中の保存先が、USB メモリの InventoryBackup フォルダと同じです。「保存先を選ぶ」で PC の中の別のフォルダを選び直してください」（用意・確認と §43.8.8 で同じ文。MNT-01-D11） |
| `OffsiteError::MarkerUnsupported` | `validation` | 「この USB メモリは新しい版のアプリで用意されています。新しい版のアプリを入れてください」 |
| `OffsiteError::StateUnreadable` | `internal` | 「PC の外の控えの記録を読めませんでした。「記録を作り直す」を押してください」 |
| `OffsiteError::StateUnsupported` | `validation` | 「PC の外の控えの記録は新しい版のアプリが書いたものです。新しい版のアプリを入れてください」 |
| `OffsiteError::NoMediumAttached` | `validation` | 「用意した USB メモリが差さっていません。USB メモリを差して「記録を作り直す」を押すか、使わないなら「PC の外の控えをやめる」を押してください」 |
| `OffsiteError::StopFailed` | `internal` | 「PC の外の控えをやめる操作を終えられませんでした。「PC の外の控えをやめる」がまだ出ていれば、もう一度押してください」（記録の状態は断言しない。card は取り直した status で決める、68 §68.4 手順 5） |
| `OffsiteError::StateResetFailed` | `internal` | 「PC の外の控えの記録の作り直しを終えられませんでした。「記録を作り直す」がまだ出ていれば、もう一度押してください」（記録の状態は断言しない。card は取り直した status で決める、68 §68.4 手順 4） |
| `OffsiteError::SourceUnverified` | `internal` | 「PC の中の最新のバックアップを確かめられなかったため、PC の外へ写していません」 |
| `inspect_backup` の `DbError` | `validation` | 「この控えを読めませんでした。別の控えを選んでください。」 |

`check_offsite_backup` の失敗の種類は、UI が文言を分けるために `OffsiteBackupStatus.last_failure_kind`（43.8.4）から読む。`CmdError.message` の文字列で分けない（MNT-01-D4 と同じ方針）。

**DTO の形**（`#[derive(serde::Serialize, specta::Type)]`、field は snake_case、日時は `YYYY-MM-DD HH:MM:SS` の文字列。`i64` の field のうち、件数〈`product_count`。1 つの SQLite の DB の行の数で、DB の大きさの上限〈約 281 TB、https://www.sqlite.org/limits.html 〉から 2^53 に届かない〉と日数〈`days_since_last_success`。chrono の日付の差〉は JS の安全な整数の範囲に収まる。版〈`BackupInspection.schema_version`・`app_max_version`〉は外の DB の値で範囲を仮定せず、UI は判定にも表示にも使わない〈戻せるかの判定は MNT の `newer_than_app`、71 §71.12。UI-11b-D15 の文は版の数を出さない〉）: `OffsiteMediumView`・`OffsiteCheckResult`（`kind` で分ける tagged enum: `not_prepared` / `no_local_backup` / `medium_missing` / `up_to_date` / `copied { file_name, labels }`）・`OffsiteBackupStatus`・`OffsiteFailureKind`（8 値）・`BackupInspection` は 71 §71.11.1・§71.11.2・§71.11.4・§71.11.5・§71.12 の定義のまま。

### 43.8.8 check_backup_dir_choice（保存先に選ぼうとする folder の確かめ、D-130、MNT-01-D11）

| 入力 → 出力 | 処理 |
|---|---|
| `BackupDirChoiceRequest { selected_path: String }` → `null` | `mnt::offsite::check_backup_dir_choice(Path::new(&selected_path))`（71 §71.11.5c）を呼ぶだけ。DB を使わない（lock を取らない）。設定を保存しない（保存は UI が `Ok` の後に §43.4 の `update_setting` で行う、68 §68.4 の backup_path 変更）。空の `selected_path` は `validation`。`OffsiteError::BackupDirIsMediumFolder` は上のエラーの写像の表の同じ行の `validation` |

```
#[tauri::command(async)]
fn check_backup_dir_choice(request: BackupDirChoiceRequest) -> Result<(), CmdError>
```

drive の列挙をするので、§43.8.2〜§43.8.7 の「実行の thread」と同じく `#[tauri::command(async)]` + `#[specta::specta]` で定義する。`update_setting` に `backup_path` の key だけの分岐を足さないのは、`update_setting` が key に依らない設定の保存の入口（BIZ-09）で、main thread で動く既存の command の実行 thread を変えないため。

---

### 43.9 restore_backup

**シグネチャ**:
```
#[tauri::command]
fn restore_backup(
    state: State<AppState>,
    app_handle: tauri::AppHandle,
    request: RestoreBackupRequest,
) -> Result<(), CmdError>
```

**処理ステップ**:
1. `app_handle.path().app_data_dir()` でapp_data_dirを取得
2. `db_path = app_data_dir.join("inventory.db")`（ファイルパス。ディレクトリではない）
3. DB Mutexのロックを取得
4. `std::mem::replace` で現在の `DbConnection` を取り出す（一時的にin-memory接続を入れる）
5. `match mnt::backup::restore_backup(old_conn, &backup_path, &db_path)` で分岐する。`Ok` / `Err` の処理・復旧再接続（no-create open）・kind 3 値 wire（`restore_failed_recovered` / `restore_failed_unrecoverable` / `restore_durability_unknown`）は本 doc で再規定しない — 復旧規則・分類の正本は 71-mnt-backup.md §71.7（MNT-01-D1/D4/D5、「CMD層での呼び出しパターン」）であり、CMD は同 § の呼び出しパターンを実行するのみで独自の規則を定義しない（D-060 (a)/(d)）
6. **`?`で早期returnしてはならない**（Mutex内のdummy接続が残るため。71 §71.7「重要: 失敗時の契約」参照）

**注意**: リストア中は他のコマンドがブロックされる（Mutexロック中）。単一ユーザーアプリのため問題なし。本 command の CMD→MNT 直接呼び出しは D-060 (a) の正規経路（DB 接続所有権の交換を要する保守 orchestration に該当）。

---

### 43.10 save_receipt_image

**シグネチャ**:
```
#[tauri::command]
fn save_receipt_image(
    app_handle: tauri::AppHandle,
    request: SaveImageRequest,
) -> Result<SaveImageResponse, CmdError>
```

**処理ステップ**:
1. `request.image_base64` をBase64デコード → バイト列に変換（wire 型変換として CMD 残留。D-060 (c)、ARCHITECTURE.md「wire型変換の失敗…はCMD境界で拒否してよい」に整合）
   - デコード失敗 → `CmdError { kind: "validation", message: "画像データが不正です" }`
2. `app_handle.path().app_data_dir()` でapp_data_dirを取得
3. `biz::inventory_service::save_receipt_image(&app_data_dir, &image_bytes, &request.extension)` を呼び出す（D-060 (b)/(c)。拡張子 validation は BIZ 所有 — [31-biz-inventory-service.md](31-biz-inventory-service.md)。BIZ が `io::image_manager` を呼ぶ）
4. 返された相対パスを `SaveImageResponse` に格納して返す

**エラーハンドリング**:
- Base64デコード失敗 → validation エラー（CMD の wire 型変換失敗）
- 拡張子不正 → BIZ の `BizError::ValidationFailed` を `From<BizError>` で変換 → validation エラー（利用者入力起因のため。wire 契約は移動前と不変）
- その他IO::Error → BIZ 経由で internal エラー

---

### 43.11 lib.rs invoke_handler への登録

PR #164 以降、この文書の command はすべて `#[specta::specta]` を付ける。`tauri::generate_handler!` と `tauri_specta::collect_commands!` の両方へ登録し、UI は generated `commands.*` を使う。

```
.invoke_handler(tauri::generate_handler![
    // ... 既存の21コマンド ...
    // CMD-11 残り: 設定・ログ・バックアップ・画像
    cmd::settings_cmd::get_settings,
    cmd::settings_cmd::update_setting,
    cmd::settings_cmd::list_logs,
    cmd::settings_cmd::list_log_operation_types,
    cmd::settings_cmd::create_backup,
    cmd::settings_cmd::check_auto_backup,
    cmd::settings_cmd::list_backups,
    cmd::settings_cmd::get_effective_backup_dir,
    cmd::settings_cmd::inspect_backup,   // §43.8.5、#[tauri::command(async)]
    cmd::settings_cmd::prepare_offsite_medium,     // §43.8.2〜§43.8.4・§43.8.6〜§43.8.8 も #[tauri::command(async)]
    cmd::settings_cmd::check_offsite_backup,
    cmd::settings_cmd::get_offsite_backup_status,
    cmd::settings_cmd::reset_offsite_state,
    cmd::settings_cmd::stop_offsite_backup,
    cmd::settings_cmd::check_backup_dir_choice,
    cmd::settings_cmd::restore_backup,
    cmd::settings_cmd::save_receipt_image,
])
```

D-114・D-130 の 7 command（§43.8.2〜§43.8.8）は `#[tauri::command(async)]`（§43.8.2 の「実行の thread」）+ `#[specta::specta]` を付け、`generate_handler!` と `collect_commands!` の両方へ登録する（`scripts/check-command-drift.sh` が 4 つの集合の一致を見る）。`inspect_backup`（§43.8.5）は runtime (a)、残りの 6 command（§43.8.2〜§43.8.4・§43.8.6〜§43.8.8）は PC の外の控えの runtime (b) で登録した。

---

### 43.12 テスト方針

CMD層のテストは主にBizError → CmdError変換とパラメータの受け渡しを検証する。

**production CMD test 規範（D-060 / SPEC-CMD11-D5 (iii)）**: settings_cmd の test は production CMD test 規範（`tauri::test::mock_builder` + `AppState` + 実 command の production path。順5 = PR #22 で確立）に従う。validation 条件の単体検証（日付形式・実在暦日・範囲順序・拡張子）は BIZ 層 test（38-biz-system-service.md / 31-biz-inventory-service.md）が所有し、CMD test は変換と受け渡しを検証する。

| テスト名 | 検証内容 | REQ |
|---------|---------|---|
| `test_get_settings_req905_cmd11` | 設定一覧が取得できる | REQ-905 |
| `test_update_setting_req905_cmd11` | 設定値の更新と読み戻し | REQ-905 |
| `test_list_logs_req902_pagination` | ページングパラメータの受け渡し | REQ-902（是正済み、§43.12.1） |
| `test_list_logs_req902_filter` | operation_typeフィルタの受け渡し（日付両方省略） | REQ-902（是正済み、§43.12.1） |
| `test_list_logs_req902_invalid_page_to_cmderror` | 不正pageのCmdError変換 | REQ-902（是正済み、§43.12.1） |
| `test_create_backup_cmd11` | バックアップ作成とBackupResult返却 | REQ-905 |
| `test_list_backups_cmd11` | バックアップ一覧の返却 | REQ-905 |
| `test_save_receipt_image_req906_cmd11_valid` | 正常なBase64画像の保存と相対パス返却 | REQ-906 |
| `test_save_receipt_image_req906_cmd11_invalid_base64` | 不正Base64でvalidationエラー | REQ-906 |

UI-11c 実装 PR で追加・確認した実テスト（結合testはbranchごとのassertionを保持。Test Design Matrix参照）:

| 実テスト名 | 検証内容 | REQ |
|---|---|---|
| `test_list_logs_req902_date_validation_contract` | 同日許可、不正形式、`start_date > end_date`の各validation branch | REQ-902 |
| `test_list_operation_logs_req902_date_range_row_count_predicate_equivalence` | 両日指定 + operation_type複合条件でitems/countのpredicate一致 | REQ-902 |
| `test_list_operation_logs_req902_one_sided_and_end_exclusive` | start/end片側指定とend翌日exclusive境界 | REQ-902 |
| `test_list_operation_logs_req902_filter_type` | 日付両方省略時の既存operation_type filter | REQ-902 |
| `test_find_distinct_operation_types_req902_dedup_order_unknown_and_empty` | empty、重複排除、昇順、未知値保持 | REQ-902 |

#### 43.12.1 REQ-902/REQ-905 traceability 是正（UI-11c-D12）

`list_logs` のpagination / operation_type filter / invalid page変換を検証する3テストは、実体がREQ-902（ログ管理）であるため、UI-11c実装PRで関数名・コメントを`req905`から`req902`へ是正済み。現行実名は上表の`test_list_logs_req902_pagination` / `test_list_logs_req902_filter` / `test_list_logs_req902_invalid_page_to_cmderror`である。根拠は [74-ui-operation-logs.md](74-ui-operation-logs.md) §74.2 UI-11c-D12 および `docs/decision-log.md` D-036を参照。
