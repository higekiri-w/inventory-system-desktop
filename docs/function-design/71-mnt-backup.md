## MNT-01: バックアップ・リストア

### 71.1 モジュール構成

```
src-tauri/src/
  mnt/
    mod.rs        -- pub mod backup・pub mod offsite
    backup.rs     -- バックアップ・リストア・自動チェック・控えの検査（本セクション、§71.12）
    offsite.rs    -- PC の外の控え（§71.11、MNT-01-D8・D11。D-114・D-130、runtime (b) で実装）
  db/
    system_repo.rs -- get_setting, upsert_setting, insert_operation_log を使用
  lib.rs          -- setup hook に check_auto_backup 呼び出しを追加
```

バックアップの対象は DB の 1 file（`VACUUM INTO`）だけで、アプリのデータ folder のレシート画像（`images/`、IO-06）と SD から読んだ原本の写し（`pos-sources/`、IO-10・BIZ-08-D5、D-111）は入らない。restore はそれらの file を変えない。PC の外の控え（§71.11）も同じ DB の backup file だけを写す。写しを backup と PC の外の控えに含めないことは owner が決めた（D-114 の owner の決定 2。D-111 の未決 B を一部置き換えた、2026-10-08、repo 外の回答台帳 TD-197）。理由と見直し契機は D-114。

---

### 71.2 依存クレート

追加なし。`chrono`（既存依存）と `rusqlite`（VACUUM INTO）を使用。

---

### 71.3 型定義

#### BackupResult構造体

```
#[derive(Debug, serde::Serialize)]
struct BackupResult {
    file_path: String,      // バックアップファイルの絶対パス
    file_name: String,      // ファイル名のみ（例: inventory_backup_20260413_130000.db）
    size_bytes: u64,        // ファイルサイズ
}
```

#### BackupInfo構造体

```
#[derive(Debug, serde::Serialize)]
struct BackupInfo {
    file_name: String,      // ファイル名
    file_path: String,      // 絶対パス
    size_bytes: u64,        // ファイルサイズ
    created_at: String,     // ファイル名から抽出した日時（YYYY-MM-DD HH:MM:SS）
}
```

#### バックアップファイル名規約

`inventory_backup_{YYYYMMDD}_{HHMMSS}.db`

例: `inventory_backup_20260413_130000.db`

名前の解析（一覧・掃除・今日の判定・作業名の掃除・媒体の写しの並び、PC の中と媒体の上で同じ関数）は、規約外の名前を `None` にし、panic しない。日時の部分は ASCII の数字と `_` だけなので、長さ・`_` の位置・ASCII かの確認を文字列の slice より前に行う（`extract_datetime_from_backup`・`extract_date_from_backup` は共有の判定で、stem が 15 bytes・9 文字目が `_`・ほかの 14 文字が ASCII の数字であることを確かめてから切り、chrono で日付・時刻として解析する。`inventory_backup_あああああ.db` のような 15 bytes の多バイトの名前でも文字境界の panic にならず、chrono の解析が受ける空白・符号の入った名前〈`inventory_backup_20260101_12 100.db`・`inventory_backup_+202 1 1_120000.db`〉も規約外になる。D-114 の作業名の掃除も同じ関数を使う）

---

### 71.4 create_backup

**関数要求**: SQLiteデータベースの安全なバックアップを作成する。WALモードでもデータ整合性を保証する

**シグネチャ**:
```
fn create_backup(
    conn: &DbConnection,
    backup_dir: &Path,
) -> Result<BackupResult, DbError>
```

**処理ステップ**:
0. 書く先を固定して確かめる（MNT-01-D10）: `pin_backup_dir(backup_dir)`（下の「書く先の固定」）で実体の path を得る。固定できない・レジの SD の上なら書かずに `DbError::QueryFailed`。手順 1〜5（folder の作成・作業名の掃除・`VACUUM INTO`・検査・公開・親 directory の sync・metadata の取得）は全部この実体の path で行う。返す `BackupResult.file_path` は今までどおり `{backup_dir}/{ファイル名}`（wire の値は変えない）
1. `backup_dir` が存在しなければ `std::fs::create_dir_all` で作成
2. 現在日時からファイル名を生成: `inventory_backup_{YYYYMMDD}_{HHMMSS}.db`
3. バックアップ先パスを構築: `{backup_dir}/{ファイル名}`。書込みは作業名 `{ファイル名}.partial` へ行う（MNT-01-D7）。`backup_dir` に残った作業名（前回の中断の残り）は先に消す。消すのは `.partial` を除いた名前が §71.3 の規約に完全に合う file（`extract_datetime_from_backup` が日時を返す名前に `.partial` を付けた名前）だけで、ほかの `*.partial`（別用途の file）は消さない（失敗は `tracing::warn!` で続行）
4. `VACUUM INTO '{作業名のパス}'` を実行
   - VACUUM INTO はWAL変更を取り込んだ単一.dbファイルを生成する（SQLite 3.27+）
   - rusqlite 0.31はSQLite 3.45+をバンドルしているため利用可能
4a. 作業名の file を共有の検査（§71.12 の「使える控え」の判定: `inspect_backup` が `Ok` で、`newer_than_app = false` かつ `quick_check_ok`）にかける（MNT-01-D7）。合格しなければ作業名の file を消して `DbError::QueryFailed` を返す（成功の log を書かない）
4b. 作業名の file を `sync_all` してから、正式名 `{ファイル名}` へ**上書きしない公開**（§71.4.1）で移し、親 directory を sync する（restore の `sync_parent` と同じ扱い。MNT-01-D7）。同じ名前の正式名が既にあれば（同じ秒に 2 回作った場合）既存の file を残し、作業名の file を消して `DbError::QueryFailed` を返す
5. バックアップファイルのメタデータ（サイズ）を取得
6. `system_repo::insert_operation_log` で記録:
   - `operation_type`: `"backup_create"`
   - `summary`: `"バックアップを作成しました: {ファイル名}"`
   - `detail_json`: `Some(json!({"file_name": ..., "size_bytes": ...}))`
7. `BackupResult` を返す

**書く先の固定**（手順 0、MNT-01-D10。PC の中の保存先と媒体の root〈§71.11.3〉で同じ関数を使う）:

```
pub fn pin_to_volume(dir: &Path) -> std::io::Result<PinnedPath>
pub fn pin_backup_dir(backup_dir: &Path) -> Result<PinnedPath, BackupDirError>
```

- `pin_to_volume(dir) -> Result<PinnedPath { normalized, actual }, io::Error>`: (1) `dir` を正規化する。Windows は `GetFullPathNameW`（絶対化、`.`・`..`・`/` の解決、末尾の空白・ピリオドの扱い。`\\?\` の path は文字の解釈を止めるので、正規化を先にしないと判定した folder と作る folder が別になる。https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file の Win32 File Namespaces）。Windows 以外は絶対化と `.`・`..` の字句の解決（同じ役割）。(2) 正規化した path のうち既に在る最も近い祖先を開き、実体の path を得る。Windows は `CreateFileW`（`FILE_FLAG_BACKUP_SEMANTICS`、`FILE_FLAG_OPEN_REPARSE_POINT` なし）で開いて `GetFinalPathNameByHandleW(…, VOLUME_NAME_GUID)` で `\\?\Volume{GUID}\…` を得て、handle を閉じる（https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew 、https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfinalpathnamebyhandlew ）。junction・symbolic link・mounted folder は辿った先の実体になる。handle は保たない（junction の target を指すので、保っても drive 文字の移動は防げない）。Windows 以外は `std::fs::canonicalize`（symbolic link を辿る）。(3) まだ無い末尾の component を、正規化した形のまま `Path::join` で付けて `actual` にする。どこかで失敗したら `Err`（固定できない）
- `pin_backup_dir(backup_dir) -> Result<PinnedPath, BackupDirError>`（`BackupDirError::{Unpinnable, RegisterSd { sd_folder: PathBuf }}`）: `pin_to_volume` が `Err` なら `Unpinnable`。`normalized` の祖先（自身から drive の root まで）と `actual` の祖先（自身から volume の root まで）の両方で、どれかの直下に `CASIO\SR500_550_4000` が directory としてあれば `RegisterSd`（junction が SD を指す・junction が SD の上にある、のどちらも拒む）。`sd_folder` は見つけた `{祖先}\CASIO\SR500_550_4000` の path（`normalized` の側で見つけたらその path、`actual` の側だけで見つけたら実体の path）で、CMD が原因の folder として名指す（43 §43.6）
- 作成・公開・掃除は `actual` で行い、返す `BackupResult.file_path`（wire・表示）は `{backup_dir}/{ファイル名}` のまま。drive 文字の割当ては抜き差しで変わり、volume GUID path は 1 つの volume しか指さない（https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-volume ）ので、判定の後に drive 文字が別の媒体（レジの SD を含む）へ移っても `actual` は元の volume しか指さず、元の volume が抜かれていれば書込みは失敗する（`DbError::QueryFailed`）
- 固定できない保存先（Mount Manager を通らない仮想 drive・RAM disk・network。`GetFinalPathNameByHandleW` の Remarks は、Mount Manager に無い volume は `VOLUME_NAME_GUID` で失敗しうるとする）には書かない。手動の backup では CMD が原因を名指す固定の文を出す（43 §43.6、68 §68.11）。network の保存先は旧い形でも `VACUUM INTO` が失敗する既知の問題（`docs/backlog.md` の UNC の保留）で、固定の追加による退行ではない
- test は Windows 以外の同じ関数を通す（symbolic link を junction の代わりにし、判定の後に link の指す先を替える注入で drive 文字の移動を模す。§71.8 の注入境界と同じ形）。`VACUUM INTO` が実体の path（`\\?\Volume{GUID}\…`）の作業名へ書けて上書きしない公開で正式名へ移せること（P7）、取外し可能な drive と NTFS の folder で `GetFinalPathNameByHandleW(VOLUME_NAME_GUID)` が成功すること（P8）、公開の後の親 directory の sync（P9）、§71.12 の URI の open（P10）は、Windows に同梱の SQLite の probe で 2026-10-23 に FAT32・NTFS、2026-10-31 に exFAT で PASS（runtime (a) の packet `docs/plans/2026-10-23-backup-offsite-runtime-a.md` の Contract Probe）。実の binary での確かめは同じ packet の L3。使う API（`GetFullPathNameW`・`CreateFileW`・`GetFinalPathNameByHandleW`・`MoveFileExW`）は既存の feature `Win32_Storage_FileSystem` にある（`windows-sys 0.61.2` の source、2026-11-01）

**エラーハンドリング**:
- ディレクトリ作成失敗 → `DbError::QueryFailed` に変換して返す
- VACUUM INTO が `Err` を返した（ディスク容量不足等）→ 作業名の file を消して（best-effort。消せなければ `tracing::warn!`）`DbError::QueryFailed` を返す。正式名の file は作られない（MNT-01-D7）
- process の中断（電源断・強制終了）と、消せなかった作業名 → 作業名の file だけが残り、次回の手順 3 が消す。一覧・掃除・今日の backup の判定は正式名だけを見るので、残りを成功の世代に数えない
- 作業名の file の検査（手順 4a）の失敗 → 作業名の file を消して `DbError::QueryFailed`（消せなければ `tracing::warn!`。次回の手順 3 が消す）
- 正式名への公開の失敗（同じ名前の正式名が既にある場合を含む）→ 既存の正式名の file は変えず、作業名の file を消して `DbError::QueryFailed`
- 公開の後の親 directory の sync の失敗 → `DbError::QueryFailed`（正式名の file は検査済みで残りうる。MNT-01-D6 の metadata の失敗と同じく、次回の一覧で観測できる）
- VACUUM INTO成功後の metadata 取得失敗 → `DbError::QueryFailed` を返し、
  `size_bytes=0` の成功結果や成功 operation log を返さない。正式名の file が
  残る可能性は許容し、次回一覧/cleanupで観測可能にする（MNT-01-D6。正式名の file は手順 4a で検査済み）
- 操作ログ記録失敗 → `tracing::warn!` で警告、バックアップ自体は成功扱い

**注意事項**:
- VACUUM INTO はパスをSQLリテラルとして渡す。パスにシングルクォートが含まれるケースを考慮し、エスケープまたはバリデーションを行う
- バックアップ先パスにシングルクォートが含まれる場合は `''` にエスケープする

**MNT-01-D7: backup は確かめてから正式名にする（作成途中・壊れた file を成功の世代に入れない、D-114）**

- 決定: `VACUUM INTO` は作業名 `{ファイル名}.partial` へ書き、共有の検査（§71.12 の「使える控え」: このアプリの backup であること・版・`PRAGMA quick_check` = `ok`）に通してから、`sync_all` → 正式名へ上書きしない公開（§71.4.1）→ 親 directory の sync の順で公開する。同じ名前の正式名が既にあれば既存を残して失敗にする。一覧（§71.6）・掃除（§71.5）・今日の backup の判定（§71.8 手順 3）・PC の外の控え（§71.11）は正式名（§71.3 の規約に完全に合う名前）だけを見る
- Why: この決定の前は `VACUUM INTO` が正式名へ直接書いていたので、途中で止まると壊れた file が正式名で残り、`check_auto_backup` が「今日の backup あり」と数えてその日の backup を作らず、一覧で復元の候補にも出ていた。「file がある」と「使える backup」を分けないと、PC の外へ写す控えも壊れた file になりうる
- Rejected alternatives: 正式名へ書いた後に検査し、だめなら消す（検査の前に中断すると壊れた file が正式名で残る）／ `std::fs::rename` で公開する（既存の同じ名前の backup を置き換える。§71.4.1）／ 公開の直前に存在を確かめてから rename する（確かめと rename の間の競合が残る）／ `PRAGMA integrity_check`（全件の検査で遅い。`quick_check` で page と構造の破損を見れば足り、索引の内容の不一致は `VACUUM INTO` が作り直すので起きにくい）／ 検査をしない（上の Why）
- Compatibility: file 名の規約・`BackupResult` / `BackupInfo` の形・操作ログの形は変えない。MNT-01-D6 の「生成済み file が残る可能性」は、未検査の残りについては作業名の file に限られる。公開の後の失敗（metadata の取得・親 directory の sync）では、検査済みの正式名が残りうる（§71.4 のエラーハンドリング）
- 見直し契機: backup の作り方を `rusqlite::backup` 等の接続 API へ替えるとき

**MNT-01-D10: backup の file をレジの SD の上に書かない（D-114）**

- 決定: backup の file を書く・消す処理（`create_backup` の手順 0、`cleanup_old_backups`〈§71.5〉、PC の外の控えの保存先の用意と写し〈§71.11〉）は、書く先の folder 自身とその祖先（volume の root まで）のどれかの直下に `CASIO\SR500_550_4000`（IO-09 がレジの SD と見なす folder 名、29 §29.7）が directory としてあれば、書かない・消さない。判定と、その後の作成・公開・掃除は、固定の関数（§71.4 の「書く先の固定」の `pin_to_volume`）で得た実体の path で行う。判定は正規化した元の path と実体の path の両方の祖先で行う（`pin_backup_dir`）。媒体の側は root の判定で足りる（書く先 `{root}\InventoryBackup` の祖先は root だけで、SD の目印は root にしかない。§71.11.2 手順 3、§71.11.3）。固定 disk の root の直下に置いた SD の写し（`C:\CASIO\SR500_550_4000` 等）でも書かない・消さない（reader が固定 disk に見える PC の SD と見分けられないため。29 §29.7.4 手順 3）。SD の写しは drive の root の直下に置かず、folder の中に置く（root の直下にあると、その drive の上の保存先への backup が全部止まる。止まった原因の folder と直し方は手動の backup の文が名指す、43 §43.6）。`create_backup` は `DbError::QueryFailed`、`cleanup_old_backups` は `io::Error`、§71.11 は `OffsiteError::RegisterSd` を返す。IO-09 と共有するのは folder 名の定数だけで、IO-09 の `resolve_register_sd_root`（選んだ path から下へ root を解決する関数、29 §29.7）は流用しない（`E:\backups` のような保存先の sub folder を SD と判定しない）
- Why: Windows の drive 文字は差した順で変わる。`backup_path` や PC の外の保存先を drive 文字で覚えると、USB メモリを抜いてレジの SD を差した日に、同じ文字の SD へ backup を書きうる。SD はレジが精算に使う媒体で、アプリは SD に書かない（IO-09-D3、D-111）
- Rejected alternatives: 取外し可能な drive を `backup_path` に選べなくする（既に選んだ人の設定を壊す。PC の外の控えは §71.11 の目印で別に扱う）／ volume の root の直下の判定を `DRIVE_REMOVABLE` の drive に限る（reader が固定 disk に見える PC〈29 §29.7.4 手順 3〉で SD に書く）／ 何もしない（上の Why）
- 見直し契機: レジの機種が変わり IO-09 の folder が変わるとき（同じ folder 名の定数を使い、文字列を二重に持たない）

#### 71.4.1 上書きしない公開（MNT-01-D7・D8 で共有）

正式名の backup file（PC の中の §71.4 手順 4b、媒体の §71.11.4 手順 4f）は、作業名から**既存の file を置き換えない**操作で正式名にする。`std::fs::rename` は使わない: Rust の公式資料は「`to` が既にあれば置き換える」とし、Windows では `MoveFileExW` を使う（https://doc.rust-lang.org/std/fs/fn.rename.html）ので、同じ名前の確かめ済みの backup を黙って上書きしうる。

- Windows: `MoveFileExW(作業名, 正式名, MOVEFILE_WRITE_THROUGH)`（`MOVEFILE_REPLACE_EXISTING` を付けない）。置き換えは `MOVEFILE_REPLACE_EXISTING` を付けたときだけ起き、`MoveFileW` の資料は「新しい名前は既に存在してはならない」とする（https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw 、https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefilew ）。存在の確認と移動を 1 つの呼出しで行うので、事前の存在確認と公開の間の競合が残らない。`windows-sys` の `Win32_Storage_FileSystem`（既存の feature。restore の `replace` が同じ関数を使う、`mnt/restore.rs` の `StdRestoreFileOps::replace`）
- Windows 以外（test だけ）: `std::fs::hard_link(作業名, 正式名)` の後に作業名を消す。Rust の公式資料は `link` の path が既にあれば error を返すとする（https://doc.rust-lang.org/std/fs/fn.hard_link.html）
- 正式名が既にあった（`ERROR_ALREADY_EXISTS` / `ERROR_FILE_EXISTS` / `io::ErrorKind::AlreadyExists`）: 既存の正式名の file を変えず、作業名の file を消して失敗を返す（§71.4 は `DbError::QueryFailed`、§71.11.4 は `Io`）
- 公開の後に親 directory を sync する。file 操作は restore の `RestoreFileOps` と同じ形の差し替え可能な操作（`publish_no_replace` を足す）で行い、test は失敗と呼出しの順を注入・記録する
- 目印と PC 側の状態の file（§71.11.1）は意図して置き換えるので、restore の `replace`（`MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH`）を使う。上書きしない公開は backup の正式名だけ

---

### 71.5 cleanup_old_backups

**関数要求**: 保持日数を超えた古いバックアップファイルを削除する

**シグネチャ**:
```
fn cleanup_old_backups(
    backup_dir: &Path,
    retention_days: u32,
) -> Result<u32, std::io::Error>
```

戻り値: 削除したファイル数

**処理ステップ**:
0. §71.4 手順 0 と同じ `pin_backup_dir` を通す（MNT-01-D10）。レジの SD の上・固定できないなら何も消さずに `io::Error`（`ErrorKind::Other`）。以後の一覧と削除は実体の path で行う
1. `backup_dir` 内のファイル一覧を `std::fs::read_dir` で取得
   - ディレクトリが存在しない → `Ok(0)` を返す
2. 各ファイルについて:
   a. ファイル名が `inventory_backup_YYYYMMDD_HHMMSS.db` パターンに一致するか確認
   b. パターン不一致 → スキップ
   c. ファイル名からYYYYMMDD部分を抽出し `chrono::NaiveDate` にパース
   d. パース失敗 → スキップ
   e. `chrono::Local::now().date_naive() - file_date > retention_days` → 削除対象
   f. `std::fs::remove_file` で削除
   g. 削除失敗 → `tracing::warn!` で警告。次のファイルに進む
3. 削除したファイル数を返す

---

### 71.6 list_backups

**関数要求**: バックアップディレクトリ内のバックアップファイル一覧を返す

**シグネチャ**:
```
fn list_backups(backup_dir: &Path) -> Result<Vec<BackupInfo>, std::io::Error>
```

**処理ステップ**:
1. `backup_dir` 内のファイル一覧を `std::fs::read_dir` で取得
   - ディレクトリが存在しない → `Ok(vec![])` を返す
2. 各ファイルについて:
   a. ファイル名が `inventory_backup_YYYYMMDD_HHMMSS.db` パターンに一致するか確認
   b. パターン不一致 → スキップ
   c. ファイル名からYYYYMMDD_HHMMSS部分を抽出 → `YYYY-MM-DD HH:MM:SS` 形式に変換して `created_at` に格納
   d. `std::fs::metadata` でファイルサイズを取得
      - metadata 取得失敗 → partial / inaccurate list を返さず
        `std::io::Error` を返す（MNT-01-D6）
   e. `BackupInfo` を作成してリストに追加
3. `created_at` の降順（新しい順）でソート
4. リストを返す

---

### 71.7 restore_backup

**関数要求**: バックアップファイルからDBを復元する。DB接続を新しいものに切り替える

**シグネチャ**:
```
fn restore_backup(
    current_conn: DbConnection,
    backup_path: &Path,
    db_path: &Path,
) -> Result<DbConnection, RestoreError>
```

注意: `current_conn` は所有権を取得する（dropしてファイルロックを解放するため）

**処理ステップ**:
1. バックアップファイルの存在確認。存在しなければ `RestoreError::Recovered` を返す
2. 現在の接続でWALをフラッシュ: `PRAGMA wal_checkpoint(TRUNCATE)`
   - 失敗 → `tracing::warn!` で警告して続行。**この非致命扱いの根拠は、ステップ4で旧DB一式（WAL含む）を退避することにある。したがってステップ4の退避が成功する場合に限り有効**（MNT-01-D1）
3. `current_conn` をdrop（ファイルロック解放）
3.5. durable manifest `{db_path}.restore_manifest` を作成する（MNT-01-D5: attempt ID + 退避対象存在集合 + phase=active を記録し、書込み → `sync_all` → 親 directory sync の完了後にのみ次へ進む。前回の manifest / manifest 一時ファイル（`.restore_manifest.tmp`）/ `.restore_backup` 遺物が残存していれば restore を開始せず Err）
4. 現在のDBファイル一式を退避する。**main / 存在する WAL / 存在する SHM のすべてで退避（rename）成功が必須**（rename は main → WAL → SHM の順、各 rename は親 directory sync で永続化。MNT-01-D5）:
   - `{db_path}` → `{db_path}.restore_backup`
   - `{db_path}-wal` → `{db_path}-wal.restore_backup`（存在する場合）
   - `{db_path}-shm` → `{db_path}-shm.restore_backup`（存在する場合）
   - いずれかの rename が失敗 → 退避済みファイルを元の名前へ巻き戻し、**本体置換に進まず** `RestoreError::Recovered` で restore を中止する（MNT-01-D1）。巻き戻し完了後は manifest を durable 削除する（Codex 再々レビュー P3）
   - 巻き戻し自体がさらに失敗した場合 → ステップ 8e と同等の致命的エラーとして扱う（`tracing::error!` 記録、`RestoreError::Unrecoverable`、アプリ再起動が必要。manifest は残置し、次回起動の reconcile に委ねる）

**MNT-01-D1: 退避は一式成功が必須、失敗時は置換前に中止**

- 決定: 上記ステップ2/4 のとおり。checkpoint 失敗の非致命扱いは「旧 DB 一式を退避できた場合」に限定し、WAL/SHM の退避失敗を warn 継続にしない。「checkpoint 失敗」には SQL としては成功したが `PRAGMA wal_checkpoint(TRUNCATE)` の戻り行（busy / log / checkpointed の 3 列）が busy = 1 を示す不完全 checkpoint を含む — SQL 実行の成否だけで checkpoint 完了と判定しない
- Why: checkpoint が失敗し WAL の退避も失敗した状態で本体だけ置換すると、旧 WAL が元の `{db_path}-wal` に残ったまま新 snapshot の `{db_path}` へ接続が開かれ、旧 WAL の再生で選択時点より後の変更が混入するか接続が失敗し得る（監査 P3b-2）。「指定 backup へ安全に復元」の成否を warn で決めてはならない
- Rejected alternatives: WAL 退避失敗時に WAL を削除して続行（checkpoint 失敗時の WAL は退避対象のデータそのものであり、削除は旧 DB 側の復元可能性を壊す）
- 見直し契機: restore の実装を接続 API ベース（`rusqlite::backup` 等）へ置き換えるとき

**MNT-01-D4: 失敗時の復旧再接続は no-create、復旧不能は recoverable に偽装しない（PR #14 Codex P1-1）**

- 決定: restore の失敗は「**退避復元済み**（元 DB 一式を元の名前へ戻せた）」と「**状態不明/未復旧**（巻き戻し失敗・二重失敗を含む）」を区別して呼び出し元へ伝える。CMD 層の復旧再接続は次の契約に従う:
  - 「退避復元済み」の場合のみ再接続を試みる。再接続は **create 能力のない open**（`SQLITE_OPEN_CREATE` を含まない `open_with_flags`）で行い、成功時のみ recoverable（再試行可能エラー）として返す
  - 「状態不明/未復旧」の場合、または no-create 再接続が失敗した場合は、再接続を試みず unrecoverable（`アプリを再起動してください` を含む既存文言）を返す
  - 区別の伝搬は message 文字列比較に依存せず型・variant レベルで行う。**実装 PR1 確定形**: MNT 層は `RestoreError::Recovered | Unrecoverable | DurabilityUnknown`、CMD wire は `CmdError.kind` の `restore_failed_recovered | restore_failed_unrecoverable | restore_durability_unknown` を用いる（`DbError` の汎用 variant は追加しない）。**CMD → UI の伝搬も同じ構造化分類識別子で行い、frontend が文言の部分一致で分岐しない**（68 §68.7 参照。Codex 再々レビュー P2-2）
- Why: 現行 CMD パターンの `db::init_database` による復旧は create 能力を持つため、二重失敗で main が `{db_path}.restore_backup` 側に残ったまま `{db_path}` が不在の状態では**空 DB を新規作成して migration まで成功**し、復旧不能な状態が recoverable として UI（68 §68.7 の `restore_failed_recovered`）に渡る。operator は「現在のデータに戻した」と誤認して空 DB へ入力を続ける — 本設計が塞ぐべき空 DB 隠蔽経路そのもの
- Rejected alternatives: 現行の create-capable `init_database` による復旧（上記の偽装経路）/ message 文字列での分岐追加のみ（文字列は契約として脆く、監査 P3-4 = 順 8 で是正予定の分裂をさらに深める）
- 見直し契機: 順 8（error 表示 contract 統一)で CmdError に相関 ID / kind 拡張が入るとき — 順 8 は PR #25（D-053）で消化済み。kind の generated enum 化（D-061、順14）でも本契約の variant 対応・分類意味論・wire 文字列表現は不変（型のみ `CmdErrorKind` へ強化）

**MNT-01-D5: restore の中断（process/power interruption）復旧契約（PR #14 Codex P1-2、再レビュー P1×3 で manifest 方式へ改訂）**

- 決定: 逐次 rename は I/O エラーには MNT-01-D1 で巻き戻せるが、プロセス中断・電源断には原子的でない。次の durable manifest + 起動時 reconcile で「元 snapshot または新 snapshot のどちらか一方が完全な形で残り再接続可能」の不変条件を再起動をまたいで保証する:
  - **前提: single-instance 保証**。本契約は同時に 1 プロセスのみが restore / reconcile / legacy 移行を実行することを前提とする。single-instance ガード（`tauri-plugin-single-instance` 等）の導入を**実装 PR1 の前提条件**とし、ガードなしで本契約を実装してはならない — 固定名の manifest / 退避名は多重プロセスに対して防御せず、後発 attempt の退避 rename が先発 attempt の旧 main を置換し得る（Codex 再レビュー P1-3）。**実装 PR1 確定形**: `tauri-plugin-single-instance = 2.4.3` を Rust plugin として mutation を行う setup より先に登録する（npm guest bindings 不要）。plugin 初期化が失敗した場合は setup mutation へ進まず起動を fail-closed 中止する
  - restore は最初のファイル mutation より前に durable manifest `{db_path}.restore_manifest` を作成する。manifest は (a) 一意 attempt ID（診断・テスト固定用）、(b) **退避対象の存在集合**（`{db_path}` / `-wal` / `-shm` それぞれの退避開始時点での有無）、(c) **phase**（`active` = 作成時 / `committed` = 新接続確立済み）を記録する。**manifest の不在を「復元完了」の判定に使ってはならない** — 現行実装（manifest 導入前）も同じ固定退避名 `.restore_backup` を使っており、manifest なしの退避遺物は「旧形式実装の任意時点中断の残骸（唯一の実データを含み得る）」と区別できない（Codex 再々レビュー P1-2）
  - **durability 契約**（Codex 再レビュー P1-2）: (a) manifest は「内容書込み → `sync_all` → 親 directory sync」の完了後にのみファイル mutation へ進む。(b) rename（退避・巻き戻し・元名復帰とも）は親 directory sync で永続化する。(c) 本体コピー完了後、`init_database` より前に新 main の `sync_all` + 親 directory sync を行う（userspace のコピー完了・page cache 経由の open 成功を永続化の根拠にしない）。(d) 成功時は新接続確立の直後（退避ファイル削除より前）に **phase=committed を原子的に durable 更新**（canonical 一時ファイル `{db_path}.restore_manifest.tmp` への書込み + `sync_all` + canonical 名への rename + 親 directory sync）する。cleanup は「退避ファイル群の unlink → **親 directory sync** → manifest unlink → 親 directory sync」の順で段階ごとに永続化する — 退避削除の永続化前に manifest を削除すると、電源断時の unlink 永続順序逆転で「manifest なし + 退避遺物あり」（fail-closed 行き）が**正常完了後に**出現し得る（Codex 第 4 round P2-1）。失敗時は巻き戻し完了後に manifest を durable 削除する。phase=committed の永続化前に退避ファイルを削除してはならない。(e) **cleanup 段階の失敗分類**（Codex 第 5 round P2-3 で精密化）: phase=committed への更新失敗は失敗点で 2 分する。(i) canonical 名への rename **前**の失敗（temp 書込み / sync / rename 自体の Err）= manifest は active **確定**。新接続を公開せず退避も削除せず Err — 次回起動の reconcile が active 一致分岐で旧データへ復帰する（既承認の巻き戻り受容窓と同じ挙動）。(ii) rename 成功**後**の親 directory sync 失敗 = **durability 不明**（名前空間上は committed 済みだが、電断をまたぐと active / committed のどちらへ回復するか確定できない）。新接続を公開せず退避も削除せず、unrecoverable（再起動必須）として返す — 事後状態を一意に断定せず、再起動時の reconcile が**実際に回復した canonical phase に従う**（active なら旧データ復帰、committed なら新 snapshot 採用。どちらも中核不変条件内）。この分類の operator 文言は**失敗を断定しない**: 「復元が完了したか確定できませんでした。アプリを再起動してください。」とし（unrecoverable 分類内の表示専用文言差し替え — 68 §68.7 の terminal 分岐・構造化識別子契約は不変）、結果は再起動後に確定する — 「失敗しました」と断定すると committed 回復（復元成功）時に事実と矛盾する（独立検証 round P2）。phase=committed 永続化後の cleanup 失敗（退避削除・manifest 削除）は復元成功を覆さない — committed manifest を残したまま新接続を返して warn を記録し、残骸は次回起動の reconcile（committed 分岐）が冪等に再処理する。ただし manifest unlink 成功後の親 directory sync 失敗では事後状態は「committed manifest 残置 **または** absent」のいずれか — committed なら次回 reconcile が再処理、absent なら（退避削除は既に durable のため）遺物なしの正常状態であり、どちらも安全に収束する。契約・テストはこの二値性を前提とし、単一状態を断定しない
  - 退避 rename は main → WAL → SHM の順で固定する（ファイル mutation は manifest 存在下でのみ行う）
  - 起動シーケンス（lib.rs）は `init_database` より前に reconcile を実行する: canonical manifest・**manifest 一時ファイル（`.restore_manifest.tmp`）**・`.restore_backup` 遺物のいずれかが存在する場合、**DB を開かず・新規作成もせず**、次の決定論的規則で解消してから通常起動に進む（temp を起動条件に含めないと temp-only 孤児が reconcile に到達せず残り続ける — Codex 第 5 round P2-1）
    - manifest **あり（phase=active）** + 退避側の実在集合が manifest 記録集合と**一致** = 退避完了後（本体コピー / 接続確立前）の中断。元名側に存在する DB 一式（main / WAL / SHM すべて — この attempt が生成した信頼できない世代）を削除してから、退避集合を rename で元名へ戻し、manifest を durable 削除する。記録集合に無い種別の元名側残骸（例: 元 DB が clean で WAL 無しと記録したのに `init_database` が生成した新世代 WAL が残る）もこの削除で必ず除去する — **存在ビットだけでは旧世代と attempt 生成世代を区別できないため、記録集合との一致/不一致を世代判定に使う**（Codex 再レビュー P1-1）
    - manifest **あり（phase=active）** + 退避側の実在集合が記録集合の**真部分集合**（退避ゼロ = mutation 未着手を含む） = 退避 rename 途中または未着手の中断。本体コピーは退避完了後にのみ始まるため、元名側に残るファイルは**旧世代の実データ** — 削除しない。退避側に実在するファイルのみ元名へ戻し（元名に同種別が存在する場合はそれを削除してから rename）、manifest を durable 削除する
    - manifest **あり（phase=active）** + 退避側の実在集合が記録集合の**部分集合でない**（記録に無い種別が退避側に存在する superset / mixed） = 本契約下では到達不能な状態（退避 rename は記録集合のファイルのみを対象とする）。自動解消せず起動中止（fail-closed + operator 可視化）とし、遺物を変更しない（Codex 再々レビュー P2-1）
    - manifest **あり（phase=committed）** = 復元は完了済みで掃除・記録だけが未完 → `{db_path}` 一式を正とし、退避遺物を削除する（reconcile はファイル操作のみで DB を開かない原則を保つ）。**manifest はここで削除しない** — committed manifest は operation_log 記録完了まで durable な pending marker として残す（メモリ上の記録要求は `init_database` 失敗や log 書込み前中断で恒久消失する — Codex 第 6 round P2 で棄却）。起動シーケンスは reconcile 完了 → legacy 移行判定 → `init_database` の後、確立した接続で**補完処理**を実行する。補完処理の結果は 3 値で定義する（Codex 第 7 round P2-1、行分類と集約は第 8 round P2）: 対象は `operation_type = "backup_restore"` の既存記録で、**行分類**を先に定義する — `detail_json` が NULL（現行実装 backup.rs が実際に生成する旧形式 row）/ valid JSON だが `attempt_id` キーなし / 別の attempt ID、はいずれも **`NoMatch`**（エラーではない）。malformed JSON / `attempt_id` の不正な型は **parse failure**。**集約順序**: (1) exact match の行が 1 件でもあれば **`AlreadyPresent`**（malformed 行が併存していても match が勝つ）/ (2) match が無く、lookup error または parse failure が 1 件でもあれば **`Failed`**（誤 INSERT による重複より安全側） / (3) 全行 NoMatch かつ error なしなら `system_repo::insert_operation_log`（summary に起動時確定の旨・detail_json に attempt_id）を実行し、成否で **`Inserted` | `Failed`**。`AlreadyPresent` と `Inserted` はどちらも**補完成功**として manifest を durable 削除する（一致記録の検出だけで manifest を残すと毎起動の再処理が続き残骸ゼロへ収束しない）。`Failed` のみ `tracing::warn!` で記録して manifest を残置し（次回起動で再試行）、**起動は中止しない** — 監査記録 1 件の補完のために可用性を犠牲にしない（独立検証 round 第 2 P2 — 持続失敗の escape hatch は restore 開始時検査の committed 例外が担う）。(e)(ii) の不確定文言を見た operator が再起動後に結果を確認する第一手段は操作ログの復元完了記録だが、この記録は **best-effort**（Codex 第 7 round P2-2）: `Failed` が持続する障害下では operation_log に現れないことがあり、その場合の確認手段は診断ログ（tracing）と復元対象日時のデータ内容確認である。「恒久欠落は構造的に発生しない」と絶対保証しない — escape hatch（committed 例外の最終試行失敗 + 削除）と両立する条件付き保証として規定する
    - manifest **なし** + 退避遺物あり = **旧形式実装（manifest 導入前）の中断残骸、または不明の遺物**。退避側が唯一の実データである可能性がある（現行実装で退避後・コピー前に中断したケース）ため、**自動削除せず**起動中止（fail-closed + operator 可視化）とする（Codex 再々レビュー P1-2。D5 実装の成功後掃除中断は phase=committed が識別するため、この分岐に落ちるのは旧形式・不明遺物のみ）
    - manifest が存在するが**読取・パース不能**（作成途中の中断による破損） = ファイル mutation は manifest の durable 化後にのみ始まるため、退避遺物が無ければ manifest のみ削除して通常起動へ進む。退避遺物が**ある**場合は自動解消せず起動中止（fail-closed + operator 可視化）とする
    - phase 更新の canonical 一時ファイル `{db_path}.restore_manifest.tmp` は **commit 判定に使わない**（Codex 第 4 round P2-2）: canonical manifest が存在する場合の temp は未 commit の残骸として durable 削除してから当該分岐を続行する。temp 単独（canonical manifest なし）の場合も先に durable 削除し、退避遺物の有無に応じて上記「manifest なし」系の規則を適用する — temp を残したまま reconcile を完了して「遺物ゼロ」を破ること、次回 restore の原子的更新（`create_new` 等）と衝突することを禁止する
  - **分岐要約表**（索引 — normative は上記の散文規則側。branch ID は §71.10 のテストが参照する。Codex 第 8 round 提案 #1 Adopt-with-changes）:

    | Branch | 観測条件 | Normative | Safety 分類 |
    |---|---|---|---|
    | T0 | temp（`.restore_manifest.tmp`）が存在 — **全分岐に先行**して durable 削除し、canonical・退避の判定はその後 | temp 規則 | 自動解消（前処理） |
    | R1 | manifest active + 退避実在集合 = 記録集合 | 一致分岐 | 自動解消（旧 snapshot 復帰） |
    | R2 | manifest active + 実在 ⊊ 記録 | 真部分集合分岐 | 自動解消（旧世代保全復帰） |
    | R3 | manifest active + 実在 ⊄ 記録（superset/mixed） | superset 分岐 | fail-closed 起動中止 |
    | R4 | manifest committed | committed 分岐 | 自動解消 + 補完継続（pending marker） |
    | R5 | manifest なし + 退避遺物あり | 旧形式・不明遺物分岐 | fail-closed 起動中止 |
    | R6 | manifest 読取・パース不能 + 退避なし | パース不能分岐 | 自動解消（manifest 削除のみ） |
    | R7 | manifest 読取・パース不能 + 退避あり | パース不能分岐 | fail-closed 起動中止 |

  - reconcile は**冪等**に設計する: 各分岐は現在の状態のみから解消先を決め、新たな中間状態を作らない。reconcile 自身が任意の時点で再中断されても（例: 一致分岐の巻き戻し途中で退避実在集合が真部分集合に減る）、再起動後の reconcile が同じ規則で残状態を一意に解消できる
  - restore 開始時に前回の manifest（phase=active）・manifest 一時ファイル（`.restore_manifest.tmp`）・`.restore_backup` 遺物のいずれかが残存している場合、restore を開始せず Err を返す（reconcile は起動時に完了しているはずで、実行中の残存は掃除失敗の兆候。fail-closed）。**例外: phase=committed の manifest のみ** — これは「復元完了済み・監査記録の補完だけが未完」の残骸であり（補完 INSERT の持続失敗で残り得る）、fail-closed で新規 restore を永久ブロックすると DB が健全なのに復元機能が使えなくなる。新規 restore 開始時に committed manifest が残っていた場合は補完処理（`AlreadyPresent | Inserted | Failed`）を最後に一度実行し、結果が `Failed` でも `tracing::warn!` を記録して durable 削除してから restore を開始する（operator が意図した新しい復元を旧 attempt の監査補完より優先する。この経路が log 保証を best-effort にする唯一の系列 — 独立検証 round 第 2 P2 + Codex 第 7 round P2-2）
  - reconcile 自体の失敗は起動中止（MNT-03-D4 と同じ fail-closed + operator 可視化）とし、遺物を残したまま `init_database` に進んで空 DB を作ることを禁止する
  - reconcile は **legacy 移行判定（22 §12）より前に**実行する。restore 中断で `{db_path}` が不在の間に legacy 移行判定が走ると「新 DB 無し」と誤認して旧 CWD DB を publish し得るため、順序は reconcile → legacy 移行判定 → `init_database` で固定する
- Why: 退避 rename 後・コピー完了前に中断すると `{db_path}` が不在になり、現行起動は `init_database` が空 DB を新規作成して実データ（退避側に無傷で存在）を隠蔽する。manifest の記録集合は (1) 「`{db_path}` を信頼してよいか」（manifest の有無）と (2) 「元名側のファイルが旧世代か attempt 生成世代か」（退避実在集合と記録集合の一致/不一致）の両方を決定論化し、全中断タイミングで解消先が一意に決まる。存在ビットのみの固定 marker（本 D5 の旧案）では (2) を判定できず、「巻き戻しで退避に無い元名を削除する」と退避途中中断の旧 WAL 実データを失い、「削除しない」と restore 成功後中断の新世代 WAL/SHM が旧 main と混在残存する — どちらの単純規則にも反例が成立する（Codex 再レビュー P1-1）。phase=committed を退避削除より前に永続化するのは、成功後の掃除中断を「main 優先」で解消するためであり、**manifest の不在を commit 判定に使うと旧形式実装（同じ固定退避名を使用）の中断残骸 — 唯一の実データを含み得る — を成功後残骸と誤認して削除する**アップグレード境界の反例が成立するため、不在は fail-closed に送る（Codex 再々レビュー P1-2）。なお「新接続確立直後〜phase=committed 永続化前」の中断だけは、完了していた復元が reconcile で旧データへ巻き戻る（不変条件には違反しない安全側の挙動。operator は復元を再実行すればよく、この挙動は受容して文書化する）
- Rejected alternatives: 存在ビットのみの固定 marker `{db_path}.restore_inprogress`（本 D5 の旧案。上記の世代判定不能で棄却）/ **phase なしの manifest + 「manifest なし + 退避あり = 掃除中断」規則**（本 D5 の第 2 案。アップグレード境界で旧形式残骸の唯一の実データを削除する反例で棄却 — Codex 再々レビュー P1-2）/ attempt ごとの一意 staging 名（single-instance 前提下では固定退避名 + manifest 記録集合で決定論を確保でき、staging 名の列挙・掃除の複雑さに見合わない）/ reconcile なしで「退避があれば常に戻す」（成功後の掃除中断で完了済みの復元が巻き戻り、operator の操作結果を無効化する）/ 同期巻き戻し（ステップ 8）を「存在する退避だけ戻す」軽量手順にする（部分 migration が生成した新世代 WAL/SHM を残し世代混在を作る — reconcile 一致分岐と同一手順に統合。Codex 再々レビュー P1-1）
- 見直し契機: restore を接続 API ベース（`rusqlite::backup` 等)へ置き換えるとき、または multi-instance 対応が要件化されるとき
5. バックアップファイルを `{db_path}` にコピー
6. `db::init_database(db_path)` で新しい接続を作成
   - PRAGMA再設定＋マイグレーション実行が含まれる
   - 新しすぎる版の backup は差し替え後の open で拒否され、現在の DB に戻る（MNT-03-D11、22 §3.2。ステップ8の巻き戻しで `RestoreError::Recovered`）
7. 成功の場合（**committed manifest は operation_log 記録完了まで durable な pending marker として保持する** — manifest を log より先に消すと「7d INSERT 前の中断で manifest / temp / 退避が全て無く reconcile 非起動 → log 恒久欠落」の反例が成立する。Codex 第 6 round P2）:
   a. manifest の phase を `committed` へ原子的に durable 更新する（MNT-01-D5: 退避ファイル削除より前が必須。**更新失敗時は新接続を公開せず退避も削除しない** — 失敗点により「rename 前 = active 確定の Err」と「rename 後 sync 失敗 = durability 不明の unrecoverable」に分類する。D5 durability 契約 (e)）
   b. 退避ファイルを削除（`.restore_backup` ファイル群の unlink 完了後、**親 directory sync で永続化**してから次へ進む — D5 durability 契約 (d)）
   c. `system_repo::insert_operation_log` で記録:
      - `operation_type`: `"backup_restore"`
      - `summary`: `"バックアップから復元しました: {ファイル名}"`（表示用 — attempt ID を自由文字列に埋め込まない）
      - `detail_json`: `attempt_id` キーに manifest の attempt ID を構造化格納する（`operation_logs` の相関データ規約 = tracking-system-tables の detail_json 用法に従う）。冪等突合は「`operation_type = "backup_restore"` で絞った記録の `detail_json.attempt_id` と manifest の attempt ID の一致」で行う — summary の部分一致照合は契約にしない（独立検証 round P2。restore 記録は低頻度のため索引の要否は実装 PR1 で判断）
   d. manifest を durable 削除する（unlink + 親 directory sync）。**7b〜7d の失敗は復元成功を覆さない** — committed manifest を残して warn 記録、新接続を返す（次回起動の reconcile が退避掃除・log 補完・manifest 削除を冪等に再処理）。manifest unlink 成功後の親 directory sync 失敗の事後状態は「committed 残置 / absent」の二値であり、どちらも安全に収束する（D5 (e)）
   e. 新しい `DbConnection` を返す
   - 注: 7c INSERT 前に中断しても committed manifest が durable に残るため、次回起動の reconcile → 起動シーケンスが補完処理（`AlreadyPresent | Inserted | Failed` — reconcile committed 分岐の定義参照）で回収する。中断のみでは log は欠落しない。ただし補完 `Failed` が持続する障害下では best-effort（committed 例外の escape hatch で manifest が削除され得る — 詳細と fallback は reconcile committed 分岐参照）
8. 失敗の場合（ステップ5-6でエラー）: 巻き戻しは **MNT-01-D5 reconcile の「一致」分岐と同一手順**で行う（存在する退避だけを戻す方式は、`init_database` の部分 migration が生成した新世代 WAL/SHM を元名側に残し、旧 main と混在させるため禁止 — Codex 再々レビュー P1-1）:
   a. 元名側の DB 一式（main / WAL / SHM すべて — この attempt が生成した信頼できない世代）を削除し、親 directory sync で永続化する
   b. manifest 記録集合の退避ファイルを rename で元名へ復帰する（`{db_path}.restore_backup` → `{db_path}`、WAL / SHM も記録集合に従う）
   c. 巻き戻し完了後に manifest を durable 削除する
   d. `RestoreError::Recovered` を返す（元のDBファイルは復元済みだが、接続は呼び出し元が再確立する必要がある）
   e. 巻き戻し（8a-8b）が失敗した場合 → `RestoreError::Unrecoverable` で致命的エラー（manifest は削除しない — 次回起動の reconcile が解消する）

**重要: 失敗時の契約**
- `restore_backup` は失敗時に「退避復元済み」か「状態不明/未復旧」かを区別できる `Err` を返す（MNT-01-D4）。有効なDbConnectionは返さない
- **CMD層が `?` で早期returnすると、Mutex内がdummy接続のまま残り、以降の全コマンドが失敗する**
- CMD層は必ず `match` で処理する。`Err` パスの再接続は MNT-01-D4 に従う: 「退避復元済み」の場合のみ **no-create open** で再接続し、それ以外（状態不明/未復旧、または no-create 再接続の失敗）は unrecoverable（再起動誘導文言）を返す。create 能力のある `init_database` を復旧再接続に使ってはならない

**CMD層での呼び出しパターン**（設計レベルの擬似コード。実装 PR1 の具体形は上記 `RestoreError` 3 variant + `CmdError.kind` 3 値）:
```
let mut guard = state.db.lock().map_err(|_| CmdError::internal(...))?;
let dummy = rusqlite::Connection::open_in_memory().map_err(...)?;
let old_conn = std::mem::replace(&mut *guard, dummy);
let db_path = app_data.join("inventory.db");  // ファイルパス（ディレクトリではない）

match mnt::backup::restore_backup(old_conn, &backup_path, &db_path) {
    Ok(new_conn) => {
        *guard = new_conn;
        Ok(())
    }
    Err(restore_err) if restore_err.is_evacuation_restored() => {
        // 退避復元済み: no-create open で再接続（空 DB を新規作成しない。MNT-01-D4）
        match db::open_existing(&db_path) {  // SQLITE_OPEN_CREATE なしの open + PRAGMA 再設定
            Ok(recovered) => {
                *guard = recovered;
                Err(CmdError::internal(&format!("バックアップの復元に失敗: {}", restore_err)))
            }
            Err(e2) => {
                tracing::error!(error = %e2, "DB接続の復旧にも失敗");
                Err(CmdError::internal(
                    "バックアップの復元に失敗し、DB接続の復旧もできませんでした。アプリを再起動してください",
                ))
            }
        }
    }
    Err(restore_err) => {
        // 状態不明/未復旧: 再接続を試みず unrecoverable（68 §68.7 の terminal 分岐へ）
        tracing::error!(error = %restore_err, "復元後の DB 状態が確定できません");
        Err(CmdError::internal(
            "バックアップの復元に失敗し、DB接続の復旧もできませんでした。アプリを再起動してください",
        ))
    }
}
```

**エラーハンドリング**:
- バックアップファイル不在 → `RestoreError::Recovered`
- コピー失敗 → 退避から復元を試みてから `Err` を返す
- init_database失敗 → 退避から復元を試みてから `Err` を返す
- 退避からの復元も失敗 → `RestoreError::Unrecoverable`（致命的。アプリ再起動が必要）
- phase=committed 更新失敗（rename 前） → 新接続を公開せず退避も削除せず `Err`（manifest は active 確定、次回 reconcile が旧データへ復帰 — D5 (e)(i)）
- phase=committed 更新の rename 後 directory sync 失敗 → durability 不明。新接続を公開せず退避も削除せず unrecoverable（再起動必須）を返す。再起動時の reconcile は実際に回復した canonical phase に従う — D5 (e)(ii)
- committed 後の cleanup / 記録（退避削除・log INSERT・manifest 削除）失敗 → 復元成功のまま warn 記録 + 新接続を返す。committed manifest が残っていれば次回起動が退避掃除・冪等 INSERT・manifest 削除を再処理する。manifest unlink 成功後の final sync 失敗のみ事後状態は「committed 残置 / absent」の二値（log は 7c で INSERT 済みのためどちらでも記録は保全、absent なら再処理も不要）— D5 (e)
- initial manifest write失敗後の canonical temp cleanup失敗 → 元の
  `RestoreError::Recovered` を維持し、temp path / cleanup errorを
  `tracing::warn!` へ記録する。error伝搬へ変えてD1/D4/D5の復旧分類を変えない

---

### 71.8 check_auto_backup

**関数要求**: 自動バックアップの条件を判定し、必要なら実行する。setup hook（起動時）とフロントエンドタイマー（60秒間隔。共通レイアウト〈UI-12〉が mount し、画面に依らない。UI-11b-D13）から呼ばれる

**シグネチャ**:
```
fn check_auto_backup(
    conn: &DbConnection,
    backup_dir: &Path,
) -> Result<bool, DbError>
```

戻り値: `true` = バックアップ実行、`false` = スキップ

**処理ステップ**:
1. `system_repo::get_setting(conn, "backup_enabled")` を取得
   - `None` or 値 ≠ "1" → `Ok(false)` を返す
2. 今日の日付を `YYYYMMDD` 形式で取得
3. `backup_dir` 内のファイルを走査し、今日のバックアップが存在するか確認
   - ファイル名が §71.3 の規約に完全に合い（`inventory_backup_{今日のYYYYMMDD}_{HHMMSS}.db`）、日付が今日のものがあるか。判定は一覧・掃除と同じ `extract_datetime_from_backup` の完全一致で行う（`backup.rs` の `collect_today_backup_names`）。作業名 `*.db.partial` と、規約に合わない名前（例 `inventory_backup_{今日}_manual.db`）は数えない（MNT-01-D7）
   - directory iterator の個別 entry error → 「entryなし」に変換せず
     `DbError::QueryFailed` を返し、backup作成・cleanup判定へ進まない
4. 今日のバックアップが1件もない場合:
   - `create_backup(conn, backup_dir)` を実行
   - `cleanup_old_backups` を実行（保持日数は **MNT-01-D3** の確定条件を満たす場合のみ）
   - `Ok(true)` を返す
5. 今日のバックアップがある場合:
   a. `system_repo::get_setting(conn, "backup_time")` を取得
   b. `None` or 空文字 → `Ok(false)` を返す（定時バックアップ未設定）
   c. `backup_time` を `HH:MM` 形式でパース。現在時刻と比較
   d. 現在時刻 < `backup_time` → `Ok(false)` を返す（まだ時間前）
   e. `backup_time` 以降に作成されたバックアップがあるか確認
      - ファイル名の `HHMMSS` 部分を `backup_time` と比較
   f. `backup_time` 以降のバックアップなし → `create_backup` + `cleanup_old_backups` を実行 → `Ok(true)`
   g. `backup_time` 以降のバックアップあり → `Ok(false)`

**エラーハンドリング**:
- `backup_dir` の読み取り失敗 → `DbError::QueryFailed` に変換
- directory iterator の個別 entry 取得失敗 → `DbError::QueryFailed` に変換し、
  今日のbackup有無を推測しない
- `backup_time` のパース失敗 → 定時バックアップをスキップ（`tracing::warn!` で警告）
- `create_backup` 失敗 → エラーをそのまま返す
- `backup_retention_days` の読取失敗・parse 失敗 → **MNT-01-D3** に従い cleanup をスキップ

**MNT-01-D6: filesystem failure の継続 / 伝搬境界**

- 決定:
  - restore開始前のmanifest temp cleanupのように、主失敗後の補助cleanupだけが
    失敗した場合はpath/error/context付きWARNを残し、元の復旧分類を維持する
  - `check_auto_backup`のentry走査失敗は「本日のbackupなし」にせず
    `DbError::QueryFailed`を返し、create/cleanupへ進まない
  - `create_backup` / `list_backups`のmetadata失敗は`size_bytes=0`へ変換せず、
    それぞれ既存の`DbError` / `std::io::Error`境界へ返す
- Why: 個別補助cleanupは後続reconcileで回収できるが、entry/metadata errorは
  backup有無・成功result・一覧内容を変え、「余分なbackup」または
  「size 0の成功」を作る。無観測の推測より明示的な一時失敗を選ぶ
- Compatibility: MNT-01-D1/D4/D5のrestore原子性、NotFound空、
  backup filename filter、保持日数、CMD/DTO/wire shape、利用者向け文言は不変
- Rejected alternatives: `.filter_map(|e| e.ok())` /
  metadata `.unwrap_or(0)` / restore temp cleanup errorを新しいfatal variantへ昇格
- 見直し契機: backup作成をatomic publish + explicit incomplete artifact管理へ
  置換するとき、またはCMD-11 service境界を再編するとき

**auto-backup entry failure 注入境界（監査順7 Plan Review P2-1）**:

production と test は次の同一 generic helper を通す。production 専用の別 collector、
test 専用 filename 判定、`#[cfg(test)]` だけの entry 処理関数を作らない。

```rust
fn collect_today_backup_names<I>(
    backup_dir: &Path,
    today_prefix: &str,
    entries: I,
) -> Result<Vec<String>, DbError>
where
    I: IntoIterator<Item = std::io::Result<PathBuf>>;
```

production は `read_dir` の各 `DirEntry` を
`entry.map(|entry| entry.path())` で上記 item 型へ変換して渡す。test は同じ helper
へ injected `Err(io::Error)` を渡し、`DbError::QueryFailed` と
create / cleanup 副作用なしを検証する。実装レビューでは public
`check_auto_backup` と failure-injection test の双方がこの helper を呼び、
entry error / filename filter の production-only / test-only 分岐が存在しないことを
明示確認する。

**MNT-01-D3: 破壊的 cleanup は保持日数を確定できた場合のみ実行**

- 決定: `cleanup_old_backups`（ファイル削除）を駆動する保持日数は、(a) `backup_retention_days` の読取が成功しかつ数値として parse できた、または (b) 設定行が存在しない（未設定 = 初期状態、既定 3 日を適用）、のどちらかの場合のみ確定とする。**DB error での読取失敗、および設定値はあるが数値として parse できない場合は、既定値へ fallback せず cleanup 自体をスキップ**して `tracing::warn!` を記録する（バックアップ作成の成否には影響させない）
- Why: 読取失敗を既定 3 日へ潰すと、例えば 90 日保持を設定済みの利用者の設定読取だけが失敗したとき、4 日目以降のバックアップを誤って削除する（監査 P3-1 の中核経路）。cleanup の skip は「バックアップが溜まる」方向の安全な失敗であり、次回成功時に自然回復する
- Rejected alternatives: 現行の `.ok().flatten().unwrap_or(3日)`（destructive fallback そのもの）/ parse 失敗も既定適用（未設定と設定破損を区別できず、破損時に削除が走る）
- 見直し契機: 設定値の書込み時 validation（数値以外を保存不能にする）が導入され、parse 失敗経路が構造的に消えたとき

---

### 71.9 lib.rs 起動シーケンスの変更

**追加箇所**: `src-tauri/src/lib.rs` の `run` の setup で、DB の初期化（`prepare_database`）と操作ログの自動削除（`mnt::log_manager::cleanup_old_logs`）の後、State 管理（`app.manage`）の前に `run_startup_auto_backup` を呼ぶ

```
// 自動バックアップチェック（起動時。lib.rs の run_startup_auto_backup の本体）
// backup_dir は設定値を優先、未設定/空ならデフォルト（app_data/backups）
// 設定読取の DB error 時はチェックをスキップして起動継続（MNT-01-D2）
match mnt::backup::resolve_backup_dir(&conn, &app_data) {
    Ok(backup_dir) => {
        if let Err(e) = mnt::backup::check_auto_backup(&conn, &backup_dir) {
            tracing::warn!(error = %e, "自動バックアップチェックに失敗");
        }
    }
    Err(e) => tracing::warn!(error = %e, "バックアップ保存先の設定読取に失敗（自動バックアップをスキップ）"),
}
```

**resolve_backup_dir（共通ヘルパー）**:
```
pub fn resolve_backup_dir(conn: &DbConnection, app_data: &Path) -> Result<PathBuf, DbError> {
    let setting = system_repo::get_setting(conn, "backup_path")?; // DB error は握りつぶさず返す
    Ok(setting
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| app_data.join("backups")))
}
```
全てのバックアップ操作（create/list/check/restore）はこのヘルパーで統一的にbackup_dirを決定する。

**MNT-01-D2: resolve_backup_dir は DB error と未設定を区別する（Result 化）**

- 決定: `get_setting` の DB error は `Err` として呼び出し元へ返し、既定ディレクトリへの fallback は「未設定または空文字」の場合に限る。本節の旧コード例（`.ok().flatten()` で両者を潰す形）は設計自体の欠陥だったため書き換えた（監査 P3-1 補強）。呼び出し元の契約:
  - lib.rs 起動時チェック: `Err` → `tracing::warn!` を記録して自動バックアップチェックをスキップし、起動は継続する
  - CMD 層（settings_cmd）: `Err` → internal error として返す（既存の error 変換規約どおり）
- Why: 設定済みの外部 backup path を DB error で読めないとき、無言で app data 配下へ fallback すると、バックアップの保存先誤認と、誤ったディレクトリに対する cleanup 実行につながる。「設定が無い」と「設定を読めない」は破壊的操作の前提として同値ではない
- D-032（復元前強制バックアップ、break-glass 含む）との整合: 当該経路は `create_backup` 呼び出し時に DB error が internal error として伝搬する既存挙動のままで矛盾しない
- Rejected alternatives: 現行どおり PathBuf を直接返し内部で warn だけ残す（呼び出し元が失敗を分岐できず、cleanup skip 等の安全側判断につなげられない）
- 見直し契機: backup 設定の保存構造を app_settings 以外へ移すとき

---

### 71.10 テスト方針

| テスト名 | 検証内容 |
|---------|---------|
| `test_create_backup_mnt01_creates_file` | VACUUM INTO でバックアップファイルが生成される |
| `test_create_backup_mnt01_filename_format` | ファイル名が `inventory_backup_YYYYMMDD_HHMMSS.db` 形式 |
| `test_create_backup_mnt01_data_integrity` | バックアップDBに現在のデータが含まれる |
| `test_create_backup_mnt01_logs_operation` | operation_type='backup_create' のログが記録される |
| `test_cleanup_old_backups_mnt01_deletes_expired` | 保持日数超過ファイルが削除される |
| `test_cleanup_old_backups_mnt01_keeps_recent` | 保持日数内のファイルが保持される |
| `test_list_backups_mnt01_returns_sorted` | 新しい順でBackupInfoが返される |
| `test_list_backups_mnt01_empty_dir` | 空ディレクトリで空Vecが返される |
| `test_restore_backup_mnt01_replaces_data` | リストア後にバックアップ時点のデータに戻る |
| `test_restore_backup_mnt01_nonexistent_file` | 存在しないファイルでNotFoundエラー |
| `test_restore_backup_mnt01_runs_migration` | 古いバックアップ復元時にマイグレーションが実行される |
| `test_check_auto_backup_mnt01_disabled` | backup_enabled=0 でスキップ |
| `test_check_auto_backup_mnt01_no_backup_today` | 今日のバックアップなしで即実行 |
| `test_check_auto_backup_mnt01_already_backed_up` | 今日のバックアップありでスキップ |
| `test_check_auto_backup_mnt01_scheduled_time` | backup_time到達で2回目のバックアップ実行 |
| `test_check_auto_backup_req901_entry_error_propagates_without_creating_backup` | 個別entry errorをQueryFailedへ返し、create/cleanup副作用なし |
| `test_create_backup_req901_metadata_error_propagates_without_success_log` | metadata errorをsize 0成功へ変換しない |
| `test_list_backups_req901_metadata_error_propagates` | metadata errorでpartial/inaccurate listを返さない |
| `test_restore_req901_manifest_temp_cleanup_failure_warns_and_preserves_recovered_error` | temp cleanup failureをwarnし、元のRecovered分類を維持 |

**失敗注入テスト（実装 PR の完了条件、監査 P8b-3 起源）**: 成功系・早期 NotFound 系だけでは MNT-01-D1〜D5 の契約を検証できない。以下を restore / cleanup / 設定読取の実装変更と同じ PR に含め、ファイル名・存在の構造検査ではなく「障害後に元 snapshot または新 snapshot のどちらか一方が完全な形で残り、再接続可能」という意味的完了条件を検証する。

**fixture / 注入の必須条件（PR #14 Codex P2-4）**: 偽陽性（旧実装でも green になるテスト）を防ぐため次を必須とする。
- 実 WAL fixture: SQLite は最後の接続の clean close で WAL を checkpoint して削除するため、「書いて閉じただけ」の DB は WAL frame を持たない。`wal_autocheckpoint=0` を設定するか作成側接続を開いたまま保持し、**テスト実行前に WAL ファイルが非自明なサイズ（frame を含む）で存在することを assert** してから対象処理を実行する
- ファイル操作の失敗注入: destination collision や権限変更は OS ごとに失敗にならない場合がある（Rust の `rename` は既存 destination を置換し得る）。rename / copy / remove の失敗は **注入可能な file-ops 抽象（failpoint）** で決定論的に起こす
- checkpoint の成否判定: `PRAGMA wal_checkpoint(TRUNCATE)` は SQL としては成功しても busy を返し得る。テストは戻り行 3 列（busy / log / checkpointed）を検査し、busy = 1 の不完全 checkpoint を明示的に作る系を含める

| テスト | 検証内容 |
|---------|---------|
| restore 退避失敗注入（MNT-01-D1） | 上記条件を満たす実 WAL fixture で、WAL/SHM の退避 rename を failpoint で失敗させ、本体置換が行われず元 DB が WAL 込みで再接続可能なことを検証 |
| restore 成功系の WAL 意味論（MNT-01-D1） | checkpoint 完了/busy 両系で、restore 後の DB がバックアップ時点のデータのみを持ち、旧 WAL の変更が混入しないことを再 open + row 検証で確認 |
| 二重失敗の unrecoverable 化（MNT-01-D4） | 巻き戻し失敗を注入して main 不在の状態を作り、CMD 復旧が空 DB を新規作成せず unrecoverable を返すことを検証（現行の create-capable 復旧では空 DB が作られ recoverable に化けることの回帰固定） |
| 中断 reconcile（MNT-01-D5 / R1・R2・R4） | 各ファイル mutation・sync・manifest 操作の直後で処理を打ち切る failpoint で中断状態（R1 = phase=active の一致 / R2 = 真部分集合（退避ゼロ含む） / R4 = phase=committed / reconcile 自身の巻き戻し途中再中断）を作り、起動時 reconcile 後に元 DB 一式（または完了済み restore 結果）が**世代混在なく**再接続可能で遺物ゼロなことを検証。元 DB の WAL/SHM 有無 × 中断点の全組合せを含み、特に「clean な元 DB（WAL 無し記録）× restore 成功後 phase=committed 前の中断」で新世代 WAL が残らないことを固定する |
| 同期巻き戻しの世代掃除（MNT-01-D5 / ステップ 8） | 退避完了後に `init_database` の部分 migration で新世代 WAL/SHM を生成させてから restore を失敗させ、同期巻き戻し後に元名側へ新世代 sidecar が残らない（旧 main + 記録集合のみ）ことを検証（Codex 再々レビュー P1-1 の回帰固定） |
| fail-closed reconcile 分岐（MNT-01-D5 / R3・R5・R7） | (a) R5 = manifest なし + 退避遺物あり（旧形式実装の中断残骸を模した fixture）、(b) R3 = 実在集合が記録集合の部分集合でない superset、(c) R7 = パース不能 manifest + 退避あり — いずれも遺物を変更せず起動中止 + operator 可視化することを検証。(a) は退避側の実データが削除されないことを必須 assert とする（Codex 再々レビュー P1-2 の回帰固定） |
| R6 単独解消（MNT-01-D5 / R6） | 読取・パース不能な manifest + 退避遺物なしの fixture で、manifest のみ durable 削除して通常起動へ継続する（fail-closed に入らない・遺物ゼロ）ことを検証（Codex 第 9 round P3 の traceability 補完） |
| 補完処理の行分類・集約（MNT-01-D5 / R4、Codex 第 8 round P2 の回帰固定） | 既存 `backup_restore` 記録の fixture 5 種 — (1) `detail_json = NULL` の旧形式 row（現行実装が実際に生成する形）、(2) valid JSON だが `attempt_id` キー欠落、(3) 別 attempt ID、(4) malformed JSON、(5) exact match + malformed row の併存 — に対し、(1)(2)(3) = NoMatch として INSERT へ進み記録がちょうど 2 件（旧 + 新）になる、(4) = Failed で manifest 残置、(5) = AlreadyPresent が勝ち INSERT なしで manifest 削除、を検証 |
| cleanup durability 順序（MNT-01-D5 (d)/(e) / T0・R1・R4） | phase=committed 更新と cleanup の**各操作直後**の failpoint 中断で「manifest なし + 退避あり」（fail-closed 誤爆）に入らないことを検証（temp 残骸の前処理 = T0、rename 前中断の復帰 = R1、committed 再開 = R4）。oracle は中断点で分割する（Codex 第 5 round P2-2）: canonical への rename **前**（temp write / sync 直後）= temp を durable 削除して active 一致分岐で旧 snapshot へ復帰。rename の dir sync **完了後** = committed 分岐で冪等に完了。rename 後〜dir sync 前 = 実際に回復した canonical phase に従い旧 / 新 snapshot の**双方を許容**（temp を commit 判定に使わないことを assert）。cleanup（退避 unlink / dir sync / manifest unlink / dir sync）各操作直後 = committed 分岐で冪等完了（manifest unlink 後の sync 中断は committed 残置 / absent の双方を許容）。temp 残骸（canonical あり / temp 単独 × 退避有無）は**実 startup dispatcher 経由**で reconcile に到達し durable 削除され遺物ゼロになることを統合テストで固定（Codex 第 5 round P2-1）。phase 更新失敗の 2 分類（rename 前 = active 確定 Err / rename 後 sync 失敗 = durability 不明 unrecoverable + 非断定文言）と cleanup 失敗時の復元成功維持 + warn を固定する。**log 補完の恒久性**（Codex 第 6 round P2 の回帰固定）: 7c INSERT 直前 / 直後・manifest 削除直前の各中断、および committed 分岐再開時の `init_database` 失敗 → 再々起動、の全系列で operation_log の `backup_restore` 記録が**ちょうど 1 件**（`detail_json.attempt_id` 冪等）存在し manifest 残骸ゼロへ収束することを assert する。補完処理の 3 値 oracle（Codex 第 7 round P2-1/P2-2 の回帰固定）: (1) 7c INSERT commit 成功 → 7d 前中断 → 次回起動で `AlreadyPresent` → manifest durable 削除、で記録ちょうど 1 件 + 残骸ゼロ（既存一致 row + committed marker の専用 fixture）。(2) committed recovery → 補完 `Failed` 持続 → 新規 restore 開始で最終補完も `Failed` → warn + durable 削除で restore がブロックされず、log は best-effort（診断ログに warn が残る）という採用方針どおりの挙動。(3) active / temp / 退避遺物では committed 例外が発火せず従来どおり fail-closed。起動が warn のみで継続すること、manifest 残置で次回再試行されることを含む |
| single-instance ガード（MNT-01-D5 前提） | 二重起動時に後発 instance が restore / reconcile / legacy 移行へ到達しないことを検証（`tauri-plugin-single-instance` 等の導入は実装 PR1 の前提条件） |
| retention 読取失敗（MNT-01-D3） | `backup_retention_days` の読取 DB error / 非数値値を注入し、cleanup が実行されず（削除 0 件）warn が記録されることを検証 |
| retention 未設定（MNT-01-D3） | 設定行なしで既定 3 日が適用されることを検証（既存挙動の固定） |
| resolve_backup_dir の DB error（MNT-01-D2） | `get_setting` の DB error 注入で `Err` が返ることを検証（未設定/空文字 → 既定 dir と区別） |

**D-114 の追加（(a) の行は runtime の (a) が、PC の外の控えの行は (b) が入れる。test 名は runtime の lane が決め、`req901` と決定 ID を含める）**:

| 対象 | 検証内容 |
|---|---|
| MNT-01-D7 作業名 | `VACUUM INTO` の失敗・検査の失敗を注入し、正式名の file が無く、作業名の file が消え、成功の操作ログが無い。作業名の削除の失敗も注入すると `tracing::warn!` で、次の `create_backup` の手順 3 が消す（中断の残りと同じ）。前回の作業名の掃除は規約に合う名前の `.partial` だけを消し、`notes.db.partial` 等を残す |
| MNT-01-D7 公開 | 同じ名前の正式名が既にあるとき、既存の file の bytes が前後で同じで、作業名が消え、`DbError::QueryFailed`（§71.4.1）。注入した file 操作の記録で `sync_file` → 上書きしない公開 → `sync_parent` の順。sync・公開・親 directory の sync の各点の失敗で、成功の操作ログが無い |
| MNT-01-D7 今日の判定 | 作業名 `inventory_backup_{今日}_{HHMMSS}.db.partial` だけ、または規約外の `inventory_backup_{今日}_manual.db` だけがある dir で `check_auto_backup` が backup を作る（今日の backup と数えない） |
| MNT-01-D10 | `{tmp}/CASIO/SR500_550_4000` を置き、`backup_dir = {tmp}/backups`（まだ無い sub folder）にした `create_backup`・`cleanup_old_backups` と、§71.11 の用意・写しが、何も書かず・消さずに失敗する（`backup_dir` の直下だけを見る実装と、IO-09 の `resolve_register_sd_root` を流用する実装が red）。書く先の固定（`pin_to_volume`）: 判定の後に `backup_dir` の途中の link の指す先をレジの SD を模した folder へ替える注入で、SD を模した folder の一覧と bytes が前後で同じ。別の folder への link を含む保存先は実体の folder に作る。`..`・`.` を含む保存先は判定と作成が同じ folder。固定できない保存先は何も書かない |
| MNT-01-D8 用意 | 取外し可能でない root・レジの SD の root を拒む。目印が既にある媒体は同じ `medium_id` を使い、写しの file を消さない |
| MNT-01-D8 写し | 用意していない（`NotPrepared`）・媒体が見えない（`MediumMissing`）ときは何も書かず `last_success` を変えない。写しの成功で正式名の file・`last_success`（hash を含む）が残る。同じ名前・同じ size が既にあれば写さない。写す元が共有の検査に通らない（壊れた・空の・このアプリのものでない正式名）と、媒体に何も書かず `last_failure` が `source_unverified`、`last_success` は前後で同じ |
| MNT-01-D8 中断 | 公開の前（copy・`sync_all`・公開そのもの）の失敗の注入: 正式名が無い、`last_success` が前後で同じ、`last_failure` が `io` / `storage_full`、次の呼出しで作業名が消えて写り `Copied`。公開の後（親 directory の sync）の失敗の注入: 検査済みの正式名が残り、`last_success` は前後で同じ、`last_failure` が `io`、次の呼出しは `UpToDate`（状態を変えない）、PC の中に新しい backup を置いた後の呼出しで `Copied` と `last_success` の更新 |
| MNT-01-D8 媒体の固定 | 探した後に媒体の目印の `medium_id` を別の値に替える（注入）と、その媒体に以後書かず、正式名・掃除の変化が無い |
| MNT-01-D8 reparse point | 媒体の `InventoryBackup` を別の一時 directory への symbolic link にすると、用意は `Redirected`、確認はその媒体を用意済みとしない。転送先の folder の一覧と bytes が前後で同じ |
| MNT-01-D6 metadata の既存 test の移し替え | `test_create_backup_req901_metadata_error_propagates_without_success_log`（`src-tauri/src/mnt/backup.rs`）は `fail_any_metadata()` で全部の metadata を失敗させるので、手順 4a の検査（作業名の size を読む）で先に失敗して作業名が消え、今の oracle（message の「サイズ」・dir に 1 file）が red になる。注入を作業名でない path（公開の後の正式名）だけに当てる形へ替え、oracle は変えない（弱めない）。4a の metadata の失敗は MNT-01-D7 作業名の行で見る |
| MNT-01-D8 照合 | 読み戻しの bytes を注入で変え、作業名の file が消え、正式名の file が無く、`last_failure` が `verify_mismatch` |
| MNT-01-D8 保持 | 正式名の写しが `OFFSITE_KEEP` を超えると古い順に消え、今写した file と作業名でない他の file（目印・名前の規約に合わない file）を消さない |
| MNT-01-D8 状態 | 状態の file が壊れていると写さずに `StateUnreadable` を返し、状態は「控えが古い」側に倒れる（黙って `NotPrepared` にならない）。`OFFSITE_STALE_DAYS` の境界（2 日前は古くない・3 日前は古い）。`local_backup_old` は PC の中の最新の正式名が無い・3 日以上前で `true`、2 日前で `false`、一覧の失敗で `false` |
| MNT-01-D8 やめる | 状態の file（読める・壊れた・`format: 2` の 3 つ）で `stop_offsite_backup` が、元の bytes を `offsite-backup.json.stopped-*` に残し、`offsite_status` が `prepared = false`・`stale = false`。媒体の `InventoryBackup` の一覧と bytes、PC の中の backup の一覧が前後で同じ。また用意すると同じ `medium_id` で写る。移せない注入で `StopFailed`、状態の file は元のまま。移した後の親 directory の sync の失敗の注入で `StopFailed`・`offsite_status` が `prepared = false` |
| MNT-01-D8 作り直し | 壊れた状態の file と目印のある媒体で `reset_offsite_state` が、元の bytes を `offsite-backup.json.unreadable-*` に copy し、状態の file を新しい内容に置き換え、媒体を登録し直し、写す（新しい backup があれば `Copied`）。`format` が 1 でない数の状態は `StateUnsupported`、媒体が 0 本なら `NoMediumAttached` で、どちらも状態の file の bytes と app data の一覧が前後で同じ。手順 4〜6 の置き換えまでの各点の失敗の注入で `StateResetFailed`、状態の file の bytes が元のまま、`offsite_status` が `state_unreadable`、押し直しで回復する。置き換えの後の親 directory の sync の失敗の注入で `StateResetFailed`・`offsite_status` が `prepared = true`・`last_failure_kind = None`（`state_unreadable` でない）。手順 7 の写しの失敗の注入で §71.11.4 の失敗・`offsite_status` がその `last_failure_kind`。手順 7 の中の成功の記録の置き換えの後の親 directory の sync の失敗の注入で、結果が `Copied`（成功のまま、§71.11.4 のエラーの表）・`offsite_status` が `stale = false`・`last_failure_kind = None` |
| MNT-01-D11 | `backup_dir` を媒体の `InventoryBackup` そのものにした場合と、別の一時 directory の symbolic link でそこを指した場合の両方で、用意が `BackupDirIsMediumFolder` で何も書かず（目印・状態の file ができない、媒体の一覧と bytes が前後で同じ）、用意の後に `backup_dir` をそこへ替えた確認が `BackupDirIsMediumFolder`・`last_failure.kind = backup_dir_is_medium_folder`・`last_success` と媒体の一覧と bytes が前後で同じ（判定を消すと `UpToDate`、文字列で比べると link の case が通って red）。`check_backup_dir_choice` は目印のある媒体の `InventoryBackup`（直接・link）を止め、`InventoryBackup` の中の folder・媒体の root・目印の無い媒体の `InventoryBackup` は `Ok`（判定を前方一致にすると red） |
| MNT-01-D9 | 新しすぎる版・`quick_check` の失敗・読めない file の 3 つを `inspect_backup` が区別し、どれも対象の file と folder を変えない（hash と folder の一覧が前後で同じ、journal の file を作らない）。空の file・`schema_versions` の無い DB・版 0 の DB・必須の表の無い DB は `Err`（読めない控え） |

---

### 71.11 PC の外の控え（MNT-01-D8、D-114）

**関数要求**: 確かめ済みの最新の backup file（§71.4、MNT-01-D7）を、利用者が用意した取外し可能な媒体（USB メモリ）へアプリが自動で写し、読み戻して同じ bytes かを確かめ、最後に PC の外へ写せた日時を画面に出せる形で残す。PC の故障・盗難・火事で PC の中の DB と backup を同時に失っても、媒体から戻せるようにする（`docs/backlog.md` の「backup に、PC の外のコピーと復元の実証が運用として設計されていない」、owner 決定 2026-10-07「外付けの保存先へ自動で書き、確かめて画面に出す」）。

アプリの仕組みは運用の型（差しっぱなし・2 本の入れ替え 等）に依らない: 用意した媒体が PC に見えている間に、まだ写していない最新の backup を写す。運用の型と脅威への耐性の比較は §71.11.6 と D-114。

#### 71.11.1 型と保存の形

```
// 媒体の目印。媒体の root の下の folder に置く: {root}\InventoryBackup\offsite-medium.json
#[derive(serde::Serialize, serde::Deserialize)]
struct OffsiteMediumMarker {
    format: u32,          // 1
    medium_id: String,    // 用意したときの uuid v4（既存の `uuid` crate）
    label: String,        // 「控え 1」「控え 2」…（PC 側の状態の札の最大の N + 1。媒体に貼る札と同じ名前）
    prepared_at: String,  // YYYY-MM-DD HH:MM:SS（ローカル時刻）
}

// PC 側の状態。DB の外に置く: {app_data_dir}\offsite-backup.json
#[derive(serde::Serialize, serde::Deserialize)]
struct OffsiteState {
    format: u32,                              // 1
    media: Vec<OffsiteMediumEntry>,           // この PC で用意した媒体
    last_success: Option<OffsiteCopyRecord>,
    last_failure: Option<OffsiteFailureRecord>,
}
struct OffsiteMediumEntry { medium_id: String, label: String, prepared_at: String }
struct OffsiteCopyRecord { at: String, file_name: String, medium_id: String, sha256: String }
struct OffsiteFailureRecord { at: String, kind: OffsiteFailureKind, medium_id: Option<String> }

enum OffsiteFailureKind {   // wire は snake_case
    RegisterSd,             // 書く先がレジの SD（MNT-01-D10）
    NotRemovable,           // 取外し可能な drive でない
    StorageFull,            // 媒体の空きが足りない（io::ErrorKind::StorageFull）
    VerifyMismatch,         // 読み戻しの bytes が元と違う
    Io,                     // その他の読み書きの失敗（写す途中の抜去・媒体の入れ替わり・同じ名前の正式名がある を含む）
    StateUnreadable,        // PC 側の状態の file が読めない・形が違う
    SourceUnverified,       // 写す元の backup が共有の検査（§71.12）に通らない
    BackupDirIsMediumFolder, // PC の中の保存先（backup_dir）が媒体の InventoryBackup と同じ実体の folder（MNT-01-D11）
}
// mnt::offsite の関数が返す失敗。上の 8 値と同じ名前の 8 つに、用意（§71.11.2）だけが返す 2 つと、記録の作り直し（§71.11.5a）だけが返す 3 つと、やめる（§71.11.5b）だけが返す 1 つを足す
enum OffsiteError {
    RegisterSd { sd_folder: PathBuf },   // 見つけた {root}\CASIO\SR500_550_4000（root は drive 文字の形。43 §43.8.2 の表の文が名指す）
    NotRemovable, StorageFull, VerifyMismatch, Io, StateUnreadable, SourceUnverified,
    BackupDirIsMediumFolder, // 用意・確認・保存先の選択の確かめ（§71.11.5c）が返す
    MarkerUnsupported,      // 用意だけ: 媒体の目印の format が 1 でない（新しい版のアプリが用意した媒体）
    Redirected,             // 用意だけ: 媒体の InventoryBackup・目印が reparse point（§71.11.3 の「reparse point を追わない」）
    StateUnsupported,       // 作り直しだけ: 状態の file の format が 1 でない数（新しい版のアプリの記録）
    NoMediumAttached,       // 作り直しだけ: 用意済みの目印のある媒体が 1 本も見えない（何も変えない）
    StateResetFailed,       // 作り直しだけ: 新しい記録の作業名・原本の退避の copy・置き換え・置き換えの後の親 directory の sync の失敗（置き換わったかは offsite_status で読む、§71.11.5a）
    StopFailed,             // やめるだけ: 状態の file を退避名へ移す・親 directory の sync の失敗（移ったかは offsite_status で読む、§71.11.5b）
}
// OffsiteError::kind() -> Option<OffsiteFailureKind>: 同じ名前の 8 つは Some(同じ値)、MarkerUnsupported・Redirected・StateUnsupported・NoMediumAttached・StateResetFailed・StopFailed は None
// （用意の失敗で何も書かないので last_failure に残さない。CMD は 43 §43.8.2 の表の固定の文に写す）

const OFFSITE_DIR_NAME: &str = "InventoryBackup";
const OFFSITE_KEEP: usize = 30;        // 媒体ごとに残す写しの数（新しい順）
const OFFSITE_STALE_DAYS: i64 = 3;     // これ以上前なら「控えが古い」（店主の許容〈3 日程度前まで戻れれば OK〉、repo 外の回答台帳 L-220）
```

- 写しの file 名は §71.3 の規約のまま（`inventory_backup_{YYYYMMDD}_{HHMMSS}.db`）。媒体の上の作業名は `{ファイル名}.partial`。
- 目印と状態の file は 2 つとも、作業名へ書いて `sync_all` → 置き換え（restore の `replace`。§71.4.1 の最後）→ 親 directory の sync の順で公開する（restore の manifest〈MNT-01-D5 の durability 契約 (a)・(d)、`mnt/restore.rs` の `write_manifest`〉と同じ。途中で止まっても前の内容が残り、rename の後の電源断で名前が消えない）。
- PC 側の状態の file の `format` が 1 でない・形が違うときは「読めない」（`StateUnreadable`）とし、上書きしない（新しい版のアプリの状態を古い版が壊さない）。読めない記録は、バックアップ画面の「記録を作り直す」（§71.11.5a）が元の bytes を別の名前へ写して残してから、新しい内容で置き換える。`format` が 1 でない数の記録は退避もしない（`StateUnsupported`）。媒体の目印も `format` が 1 でなければ書き換えず、その媒体に書かない（§71.11.2 手順 4、§71.11.3）。
- 状態を DB（`app_settings`）に置かない: 復元で DB が過去へ戻ると、媒体の一覧と「最後に写せた日」も戻り、用意した媒体を忘れて黙って写さなくなるため。

#### 71.11.2 prepare_offsite_medium

**関数要求**: 利用者が選んだ USB メモリを、PC の外の控えの保存先として用意する（目印を置き、PC 側の状態に登録する）。

**シグネチャ**:
```
fn prepare_offsite_medium(
    app_data_dir: &Path,
    backup_dir: &Path,      // 手順 6 の写しの元（§71.11.4 と同じ）
    selected: &Path,
) -> Result<OffsiteMediumView, OffsiteError>

struct OffsiteMediumView { label: String, drive_root: String, newest_copy: Option<String> }
// newest_copy: 媒体の InventoryBackup\ の最新の正式名（§71.3 の規約に完全に合う名前、reparse point の entry を除く）の file 名。無ければ None
```

**処理ステップ**:
1. `selected` をその drive の root（`E:\` 等）にする（root の下の folder を選んでも root を使う）。以後の手順の読み書きは、その root を `pin_to_volume`（§71.4 の「書く先の固定」）で固定した実体の path（§71.11.3 の「媒体の固定」）を通して行う
2. root の drive が取外し可能（Windows の `GetDriveTypeW` = `DRIVE_REMOVABLE`）でなければ `OffsiteError::NotRemovable`（固定 disk・network・PC の内蔵の drive は PC の外と言えない）
3. root に `CASIO\SR500_550_4000` があれば `OffsiteError::RegisterSd`（MNT-01-D10 の判定。書く先 `{root}\InventoryBackup` の祖先は root だけ）
3a. PC 側の状態を読む。file が無ければ空の状態として続ける。読めなければ、媒体に何も書かずに `OffsiteError::StateUnreadable`（手順 4 の目印を書く前に止める。上書きで媒体の一覧を失わない）
3b. `{root}\InventoryBackup` と目印の file が reparse point（junction・symbolic link・mounted folder）なら、追わずに何も書かず `OffsiteError::Redirected`（§71.11.3 の「reparse point を追わない」）
3c. `{root}\InventoryBackup` が `backup_dir` と同じ実体の folder（§71.11.3 の「PC の中の保存先と同じ folder の判定」）なら、何も書かずに `OffsiteError::BackupDirIsMediumFolder`（MNT-01-D11。folder がまだ無くても、`backup_dir` がその名前を指していれば止める）
4. `{root}\InventoryBackup\offsite-medium.json` を読む。`format` が 1 で `medium_id` と `label` が読めればそれを使い、目印を書き換えない（別の PC での登録し直し・入れ替えの後。知らない field は問わない）。`format` が 1 でない数（新しい版のアプリが用意した媒体）なら、目印・folder・状態を変えずに `OffsiteError::MarkerUnsupported`（新しい版のアプリが置いた目印を古い版が上書きしないため。field が読めても読めなくても作り直さない）。file が無い・JSON として読めない・`format` が無い・`format` が 1 で `medium_id` か `label` が無ければ folder を作り、新しい `medium_id` と `label` で目印を書く（§71.11.1 の公開の順）。`label` は「控え N」で、N は PC 側の状態の `media` の札の最大の N + 1（`media` が空なら 1。数 + 1 にすると、ある札の媒体を失った後に登録し直した札と重なる）。目印を置き換えても folder の写しは消さない
5. 手順 3a で読んだ状態に媒体を足す（同じ `medium_id` があれば変えない）。状態の file が無ければ作る
6. 続けて §71.11.4 を 1 回行い、用意した媒体へ最新の控えを写す（用意した直後に「写せた」まで見せる）。写しの失敗は用意を取り消さず、状態の `last_failure` に残す
7. `OffsiteMediumView` を返す

**エラーハンドリング**: `NotRemovable` / `RegisterSd` / `StateUnreadable` / `Redirected` / `BackupDirIsMediumFolder` / `MarkerUnsupported` は何も書かない（どれも手順 4 の書込みより前に判定する。`last_failure` にも残さない）。媒体の folder・目印の書込みの失敗は `OffsiteError::Io`（媒体の空きなしは `StorageFull`）。PC 側の状態の file の書込みの失敗は、空き不足を含め `Io`（`StorageFull` は 43 の文が「USB メモリの空き」を名指すので媒体に限る）。

#### 71.11.3 媒体を探す（内部）

- `GetLogicalDrives` で drive 文字を並べ、`GetDriveTypeW` が `DRIVE_REMOVABLE` の root だけを見る。root に `CASIO\SR500_550_4000` があれば見ない（MNT-01-D10）。目印が `format` 1 で読めて、その `medium_id` が PC 側の状態の `media` にある root だけを「用意済みの媒体」とする（目印の無い媒体・別の PC で用意した媒体・`format` が 1 でない目印の媒体には書かない）
- 媒体の入っていない読取り機・読めない root は黙って飛ばす（探すたびに error にしない）
- **reparse point を追わない**（媒体の上の規則）: 媒体の `InventoryBackup` が reparse point なら、その媒体を「用意済みの媒体」としない（目印を読まず、何も書かない）。`InventoryBackup` の中の entry（目印・正式名・作業名）が reparse point なら、その entry を目印・写し・作業名として数えず、開かず、消さない。判定は Windows では `GetFileAttributesW` の `FILE_ATTRIBUTE_REPARSE_POINT`。Microsoft の資料: directory が reparse point かは `GetFileAttributes` の戻り値の `FILE_ATTRIBUTE_REPARSE_POINT` で見る。symbolic link を指す path には link 自身の属性を返す。junction は同じ PC の別の local volume の directory も指せる（https://learn.microsoft.com/en-us/windows/win32/fileio/determining-whether-a-directory-is-a-volume-mount-point 、https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfileattributesw 、https://learn.microsoft.com/en-us/windows/win32/fileio/hard-links-and-junctions ）。volume GUID path への固定は root を固定するだけで、その下の junction の転送先を制限しないので、別に要る。Windows 以外（test）は `std::fs::symlink_metadata` の `file_type().is_symlink()`。`GetFileAttributesW` は既存の feature `Win32_Storage_FileSystem`。判定は目印の読み直しと同じ時点（作業名へ写す直前・公開の直前・掃除の直前）にも行う
- PC の中の側（`backup_dir`・`app_data_dir`）の規則: `backup_dir` 自身が reparse point かは見ない（利用者が選んだ保存先で、既存の契約〈MNT-01-D10 の棄却案と同じく、選んだ設定を壊さない〉）。entry の名前は媒体と同じ完全一致の規則（§71.3、§71.4 手順 3）で扱う。消す操作（作業名の掃除・保持日数の掃除）は entry が symbolic link なら link だけを消し、指す先を消さない（Microsoft の `DeleteFileW` の資料: symbolic link を指す path では link が消え、target は消えない。https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-deletefilew ）。写す元は共有の検査（§71.12）が中身を確かめる
- **error の dialog を出さない**: 媒体を探す・読み書きする間（§71.11.2〜§71.11.5c の各関数の本体）は、その thread の error mode を `SetThreadErrorMode(SEM_FAILCRITICALERRORS, &old)` にし、終わったら `old` へ戻す。Microsoft の資料: `SEM_FAILCRITICALERRORS` は critical-error-handler の message box を出さず error を呼んだ thread へ返す。`SetThreadErrorMode` は呼んだ thread だけの設定で（thread は process の error mode を継ぎ、process 全体の `SetErrorMode` より「system の通常の振舞いを乱さない」ので Windows 7 以降は推奨される）、Windows 7 以降（https://learn.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-setthreaderrormode 、https://learn.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-seterrormode ）。空の読取り機で dialog が出るかを前提にしない（packet の Contract Probe の P3）。`windows-sys` の `Win32::System::Diagnostics::Debug::SetThreadErrorMode` で、feature `Win32_System_Diagnostics_Debug` は #172 で既存の `windows-sys` に追加済みで、そのまま使う（依存・feature の追加は不要）
- drive 文字は覚えない（入れ替えで変わる。MNT-01-D10 の Why）
- **媒体の固定**: 探して照合した媒体への 1 回の確認の中の読み書きは、drive 文字ではなく volume の識別で行う。root（`E:\`）を `pin_to_volume`（§71.4 の「書く先の固定」。PC の中の保存先と同じ関数と test の注入点）に通して実体の volume GUID path（`\\?\Volume{GUID}\`）を得て、以後その path の下の `InventoryBackup\` を使う（媒体の `InventoryBackup` の reparse point の判定は下の規則のまま別に行う）。Microsoft の資料: drive 文字の割当ては volume の抜き差しで変わり、volume GUID path は 1 つの volume しか指さない（OS が volume の導入・format のときに割り当てる）（https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-volume）。途中で抜かれれば path が無くなって `Io` になり、同じ文字に別の媒体が割り当たってもそちらへ書かない。加えて、作業名へ写す直前・公開の直前・掃除の直前に目印を読み直し、`medium_id` が照合した値と違う・読めなければその媒体の残りをやめる（`Io`。Windows 以外の test は root の path を `pin_to_volume` で固定し、この読み直しで入れ替わりを見る）
- **PC の中の保存先と同じ folder の判定**（MNT-01-D11。用意〈§71.11.2 手順 3c〉・確認〈§71.11.4 手順 3〉・保存先の選択の確かめ〈§71.11.5c〉で同じ 1 つの関数を使う）: 比べる 2 つの path を、どちらも `pin_to_volume`（§71.4 の「書く先の固定」）で固定した `actual` にしてから比べる。片方は PC の中の保存先（`backup_dir`、または保存先に選ぼうとしている folder）を固定した `actual`、もう片方は上の「媒体の固定」で得た root の実体の path に `InventoryBackup` を付けた path。component ごとに同じなら同じ実体の folder とする（Windows は ASCII の大文字・小文字を区別しない。在る component は `GetFinalPathNameByHandleW` が disk の上の名前で返し、まだ無い末尾の component は正規化した形のまま付くので、まだ無い `inventorybackup` も同じ folder と見なす）。設定の `backup_path` の文字列・drive 文字の path は比べない（junction・symbolic link・drive 文字の違いで同じ folder を見落とさないため。`pin_to_volume` は辿った先の volume GUID path を返す）。保存先を固定できない（`pin_to_volume` が `Err`）ときは同じ folder と見なさない（固定できない保存先には PC の中の backup も書かれず〈§71.4 手順 0〉、その古さは `local_backup_old` が知らせる、§71.11.5）。同じ folder だけを止め、保存先が `InventoryBackup` の中の folder や媒体の root のときは止めない（PC の中の保持日数の掃除〈§71.5〉と媒体の掃除〈§71.11.4 手順 4g〉は、どちらも自分の folder の直下の規約の名前の file だけを見るので、互いの file を消さない）。Windows 以外（test）も `pin_to_volume`（在る祖先を `std::fs::canonicalize` し、まだ無い末尾を付ける）で同じ関数を通し、symbolic link を junction の代わりにする
- Windows 以外は空の列を返す。test は root の列を受ける内部関数（`check_offsite_backup_with_roots` 等）へ一時 directory を渡す。production と test は同じ内部関数を通し、test だけの判定を作らない（§71.8 の注入境界と同じ形）

#### 71.11.4 check_offsite_backup

**関数要求**: 用意した媒体が見えていれば、まだ写していない最新の backup を写して確かめる。共通レイアウトの確認（起動直後の 1 回と 60 秒ごと、UI-11b-D16）と §71.11.2 の手順 6 から呼ばれる。

**シグネチャ**:
```
fn check_offsite_backup(
    app_data_dir: &Path,
    backup_dir: &Path,
) -> Result<OffsiteCheckResult, OffsiteError>

enum OffsiteCheckResult {
    NotPrepared,                 // 用意した媒体が無い（何もしない）
    NoLocalBackup,               // 写す backup がまだ無い
    MediumMissing,               // 用意した媒体が見えない（何も書かない）
    UpToDate,                    // 見えている媒体に最新の backup が既にある
    Copied { file_name: String, labels: Vec<String> },
}
```

**処理ステップ**:
1. PC 側の状態を読む。file が無い・`media` が空 → `NotPrepared`。読めない → `last_failure` を書けないまま `OffsiteError::StateUnreadable`
2. 写す元 = `list_backups(backup_dir)`（§71.6。正式名だけ）の先頭。無ければ `NoLocalBackup`。`list_backups` が `Err` なら、どの媒体にも書かず `last_failure = { at, kind: Io, medium_id: None }` を書いて `OffsiteError::Io`（`NoLocalBackup` に倒さない）
3. 用意済みの媒体を探す（§71.11.3）。0 → `MediumMissing`（状態を変えない）。見つけた媒体のどれかの `InventoryBackup` が `backup_dir` と同じ実体の folder（§71.11.3 の「PC の中の保存先と同じ folder の判定」、MNT-01-D11）なら、どの媒体にも書かず・消さず（手順 3a の検査もしない）、`last_failure = { at, kind: BackupDirIsMediumFolder, medium_id: その媒体 }` を書いて `OffsiteError::BackupDirIsMediumFolder`（`last_success` を変えない。同じ folder のままでは、写す元と写す先が同じ file で手順 4a が毎回 `UpToDate` になり、原因を名指さずに `stale` になるため）。保存先は 1 つなので同じ folder の媒体は多くて 1 本で、ほかの媒体へも写さない（原因と直し方を 1 つにする。店の PC の空きの USB の口は 1 つで、2 本を同時に差さない〈§71.11.6〉）
3a. 見えている媒体のどれか 1 つにでも同じ名前・同じ size の正式名が無い（写す必要がある）ときだけ、写す元を共有の検査（§71.12 の「使える控え」）にかける。合格しなければ、どの媒体にも書かず、`last_failure = { at, kind: SourceUnverified, medium_id: None }` を書いて `Err`（`last_success` を変えない。壊れた・空の・このアプリのものでない正式名の file〈旧い実装の中断の残り等〉を、hash が一致するだけで「写せた」としない）
4. 見えている媒体ごとに、その volume の識別（§71.11.3 の「媒体の固定」）の下の `InventoryBackup\` へ:
   a. 同じ名前の正式名の file（reparse point でないもの）があり size が同じなら写さない（アプリが置く正式名は手順 f でしか作らないので確かめ済み。手で置いた規約の名前の file は例外で、§71.11.7）。同じ名前の entry が reparse point なら写したことにせず、手順 f の公開が既存の名前で失敗して `Io` になる
   b. 残っている作業名を消す。消すのは `.partial` を除いた名前が §71.3 の規約に完全に合う file だけ（§71.4 手順 3 と同じ。`unrelated.partial` 等は消さない）
   c. 元の file の SHA-256 を計算する（既存の `sha2` crate）
   d. 目印を読み直して照合し、`InventoryBackup` が reparse point でないことを確かめ（§71.11.3）、`{ファイル名}.partial` を `OpenOptions::create_new` で開いて `io::copy` → `sync_all` する（§71.11.5a 手順 5 と同じ、既存なら失敗する copy）。作業名の entry が残る・直前に現れる場合は、reparse point を含め開かず・上書きせず `Io`。手順 b が除外した link を辿る `std::fs::copy` は使わない
   e. 作業名の file を開き直して全部読み、SHA-256 を c と比べる。違えば作業名の file を消し `VerifyMismatch`
   f. 目印と reparse point を確かめ直し、正式名へ上書きしない公開（§71.4.1）をして、親 directory を sync する。同じ名前の正式名が現れていれば既存を残し、作業名を消して `Io`
   g. 掃除: 目印と reparse point を確かめ直し、正式名の写し（reparse point の entry を除く）を新しい順に並べ、`OFFSITE_KEEP` 本目より後を消す。今写した file・作業名でない他の file（目印、名前の規約に合わない file）は消さない。消せなければ `tracing::warn!` で続ける
5. 1 つ以上写せたら `last_success = { at: 今, file_name, medium_id: 最後に写した媒体, sha256 }`、`last_failure = None` を書き `Copied`。写すものが無ければ `UpToDate`（状態を変えない）
6. 手順 4 のどれかが失敗したら、その媒体の残りを止め、`last_failure = { at, kind, medium_id }` を書いて `Err`（他の媒体へ写せていれば `last_success` も書く）

**排他**: 用意（§71.11.2）と 60 秒の確認が重ならないよう、`mnt::offsite` の中の 1 つの `Mutex<()>` で直列にする（ponytail: process 全体の lock。媒体ごとの lock は複数の媒体を同時に扱う要件が出たとき）。lock は公開関数（`prepare_offsite_medium`・`check_offsite_backup`・`reset_offsite_state`・`stop_offsite_backup`）の入口で 1 回だけ取る。用意の手順 6 は lock を取らない内部関数（`check_offsite_backup` の本体）を呼ぶ（`std::sync::Mutex` は再入できず、入口の関数を呼ぶと自分を待って止まる）。守るデータの無い `Mutex<()>`（状態は file の側にあり、lock は直列にするためだけ）なので、lock を持った thread の panic による poison は `PoisonError::into_inner` で受けて続ける（止めると以後の確認・用意・作り直し・やめるが全部止まり、知らせだけが残る）。`offsite_status` は書かないので lock を取らない。DB の Mutex は持たない（CMD が設定の読取りの間だけ持ち、写す前に放す。43 §43.8.3）。60 秒の確認と競合しうる操作ごとの扱い:

| 操作 | 競合 | 扱い |
|---|---|---|
| 復元（控えの確かめの開始から終わりまで） | 媒体の掃除・PC の中の保持日数の掃除が、選んでいる控えを消す。復元の間に写す | 画面が確かめの開始の前に共通の確認を止め、実行中の回の完了を待つ。復元の card の「閉じる」（`ready` へ戻る）か復元の終わりまで止めたまま（68 UI-11b-D15・D16） |
| 用意（`prepare_offsite_medium`） | 同じ媒体へ同時に書く | 上の `Mutex<()>` |
| 記録の作り直し（`reset_offsite_state`、§71.11.5a） | 状態の file を退避・作り直す間に写す・状態を書く | 上の `Mutex<()>`（入口で 1 回。手順 7 は lock を取らない内部関数） |
| やめる（`stop_offsite_backup`、§71.11.5b） | 状態の file を移す間に写す・状態を書く | 上の `Mutex<()>`（入口で 1 回） |
| 手動の backup・事前バックアップ（`create_backup`） | 写す元の一覧に作成途中の file が見える | 作業名へ書いて検査の後にだけ正式名を公開する（MNT-01-D7）ので、写す元は正式名の確かめ済みの file だけ。PC の中の保持日数の掃除は日付で古い file だけを消し、写す元の最新（今日の file）は消さない。`checkAutoBackup` とは DB の Mutex で直列 |
| 設定の変更（`backup_path`） | 写しの途中で保存先が替わる | 確認は始めに読んだ `backup_dir` から写し終える（写す元は検査済みで、替わる前の保存先の file）。次の回から新しい保存先 |
| 控えを選んで確かめる（短い確かめ、§71.13） | 確かめている控えを媒体の掃除が消す | 復元と同じ入口（UI-11b-D15）なので、同じく止める |

**排他の test の注入境界**: 公開関数の各入口は、本番と test が共有する、上の同じ mutex を取る内部入口へ委ねる。その入口に媒体の root の列・時刻・`RestoreFileOps` を渡せるようにし、lock の取得から状態の読取り・書込み・解放までを test で通す（lock を取らない確認の本体だけの test では代えない）。手順 4d の `create_new` の copy を channel / barrier で待たせ、別 thread の用意・作り直し・停止が lock 取得を試みたことと、取得後の本体への入場を区別して記録する。取得前後の通知は test の観測点とし、mutex 自体は差し替えない。用意の手順 6・作り直しの手順 7 は lock を取らない本体へ進み、二重取得しない。停止の完了後、待機していた確認を含む全 thread を join しても状態の file が再作成されず、次の確認は `NotPrepared` であることを確かめる。

**エラーハンドリング**:

| 失敗 | 返り値 | 状態 |
|---|---|---|
| 状態の file が読めない | `StateUnreadable` | 変えない（読めないので書かない） |
| 写す元の一覧の失敗（`list_backups` の `Err`） | `Io` | `last_failure`（`medium_id: None`。どの媒体にも書かない、`last_success` は変えない） |
| 写す元が共有の検査に通らない | `SourceUnverified` | `last_failure`（どの媒体にも書かない、`last_success` は変えない） |
| 見えている媒体の `InventoryBackup` が `backup_dir` と同じ実体の folder（手順 3、MNT-01-D11） | `BackupDirIsMediumFolder` | `last_failure`（`medium_id` はその媒体。どの媒体にも書かず・消さない、`last_success` は変えない）。保存先を選び直した次の確認で直る |
| 公開の前の失敗: 媒体への copy・`sync_all`・公開そのもの（写す途中の抜去・目印や reparse point の確かめ直しの不一致・同じ名前の entry を含む） | `Io` / `StorageFull` | `last_failure`。正式名を作らず、作業名だけが残りうる（次の確認の手順 4b が消して写し直す）。`last_success` は前後で同じ |
| 公開の後の失敗: 親 directory の sync | `Io` | `last_failure`。検査済みの正式名は残し、`last_success` は書かない。次の確認は同じ名前・同じ size の正式名があるので手順 4a で写さず `UpToDate`（状態を変えない）。`last_success` は、次に新しい backup を写せた時に直る（それまでの日数は `stale` の判定に入る。安全側） |
| 読み戻しの不一致 | `VerifyMismatch` | `last_failure`、作業名の file を消す |
| 掃除の失敗 | 成功のまま | `tracing::warn!` |
| 状態の file の書込みの失敗（作業名の書込み・`sync_all`・置き換えそのもの） | `Io` | 写しは媒体に残る（次の確認で `UpToDate` になり、状態は次の成功で直る） |
| 手順 5 の成功の記録の置き換えの後の親 directory の sync の失敗 | 成功のまま（`Copied`） | `tracing::warn!`。状態の file は成功の記録の内容で読める。電源断で置き換えが戻れば前の記録に戻り、`stale` 側に倒れる（次の確認は `UpToDate` で状態を変えず、次に新しい backup を写せた時に直る。公開の後の親 directory の sync の失敗と同じ扱い） |

成功の記録の置き換えの後の sync の失敗を成功のままにする理由（runtime (b) の起票で決めた、2026-10-23）: 置き換えた後の状態の file は成功の記録を返すので、`Io` を返すと画面とホームは成功の日時を出し、同時に失敗の toast（UI-11b-D16）や作り直しの失敗の文（§71.11.5a、68 §68.4 手順 4）を出して食い違う（design の packet の closeout の申し送り (b)）。写しは照合済みで、戻りうるのは記録の永続化だけで、戻っても知らせる側に倒れる。棄却: `Io` のまま画面で文を消す（失敗の種類から画面の文を推し量らない、68 §68.4）。

#### 71.11.5 offsite_status

**関数要求**: 画面とホームが出す「PC の外の控え」の状態を返す。書込みはしない。

**シグネチャ**:
```
fn offsite_status(app_data_dir: &Path, backup_dir: &Path, today: chrono::NaiveDate) -> OffsiteBackupStatus

struct OffsiteBackupStatus {
    prepared: bool,                         // 用意した媒体が 1 つ以上ある
    last_success_at: Option<String>,        // YYYY-MM-DD HH:MM:SS
    last_success_label: Option<String>,     // 写した媒体の「控え N」
    days_since_last_success: Option<i64>,   // 今日の日付 − min(最後に写した日の日付, 写した控えの日付〈last_success.file_name の §71.3 の日付〉)
    stale: bool,                            // 下の規則
    attached: Vec<OffsiteMediumView>,       // 今見えている用意済みの媒体
    last_failure_kind: Option<OffsiteFailureKind>, // 最後の確認が失敗なら
    local_backup_old: bool,                 // PC の中の最新の backup が無いか、その日付が OFFSITE_STALE_DAYS 日以上前
}
```

**規則**: `stale = prepared && (last_success_at が無い || days_since_last_success >= OFFSITE_STALE_DAYS)`。日数は、写した日と写した控えの日付の古い方から数える（PC の中の backup が止まった後に古い控えを写しても、中身の古さで知らせる。例: 10 日前の控えを今日写せば 10 日。写した日だけで数えると、ホームが知らせずに 10 日前の中身のまま過ぎる）。知らせの文（UI-11b-D14、UI-00-D12）は原因で分ける: USB が抜けている・写せないときは USB の文、PC の中の backup が作られていないときは PC の中の backup の文（下の `local_backup_old`）。状態の file が読めないときは `prepared = true`、`stale = true`、`last_failure_kind = StateUnreadable` を返す（知らせる側に倒す）。状態の file が無いときは `prepared = false`、`stale = false`（用意は導入時の owner の作業で、用意の前にホームで知らせない）。

**PC の中の backup の古さ**（`local_backup_old`）: `list_backups(backup_dir)`（§71.6。正式名だけ）の先頭が無いか、その名前の日付（§71.3）が `today` から `OFFSITE_STALE_DAYS` 日以上前なら `true`（2 日前は `false`、3 日前は `true`）。`list_backups` が `Err` なら `false`（原因を決めつけない。PC の中の一覧の失敗は画面の一覧が出す）。`stale` の判定は変えない。`stale` のとき画面とホームが原因の文を選ぶ材料で（UI-11b-D14、UI-00-D12）、`true` なら USB を差し直しても直らない（写す元が古いので、写しても `UpToDate` のまま、または古い控えを写すだけ）ため、USB の文でなく PC の中の backup の直し方を出す。読む `backup_dir` は写す元と同じ（43 §43.8.4）。書かない

#### 71.11.5a reset_offsite_state（読めない記録を作り直す）

**関数要求**: PC 側の状態の file が読めない（§71.11.1。壊れた・形が違う）とき、バックアップ画面の操作だけで、読めない file の元の bytes を別の名前へ写して残し、差してある用意済みの媒体を目印から登録し直した記録で置き換えて、写しを再開する（D-127。owner が app data の file を手で動かさない）。

**シグネチャ**:
```
fn reset_offsite_state(app_data_dir: &Path, backup_dir: &Path) -> Result<OffsiteCheckResult, OffsiteError>
```

**処理ステップ**:
1. §71.11.4 の `Mutex<()>` を取る（入口で 1 回）
2. 状態の file を読む。無い・読める → 何もせず手順 7 へ（作り直す要が無い）。JSON として読めて `format` が 1 でない数 → 何も変えず `OffsiteError::StateUnsupported`（新しい版のアプリの記録を古い版が退避・上書きしない）
3. （作り直す要がある枝だけ）見えている媒体を §71.11.3 の規則で探す（取外し可能・レジの SD でない・reparse point でない・目印が `format` 1 で読める。状態の `media` との照合だけを外す）。0 本なら何も変えず `OffsiteError::NoMediumAttached`（作り直しは登録し直す相手の媒体が要る。記録は読めないまま残り、知らせも残る。画面は「USB メモリを差して「記録を作り直す」」と「PC の外の控えをやめる」〈§71.11.5b〉の 2 つを名指す、68 UI-11b-D14）
4. 各目印の `medium_id`・`label`・`prepared_at` で `media` を作り、`last_success` と `last_failure` の無い新しい状態を、状態の file の作業名へ書いて `sync_all`（前回の中断の作業名はここで上書きする）
5. 読めない原本を `offsite-backup.json.unreadable-{YYYYMMDD_HHMMSS}` へ **copy** する（移動でなく。`OpenOptions::create_new` で開いて `io::copy` → `sync_all` → 親 directory の sync。既にある名前は上書きしない）
6. 作業名で状態の file を置き換え（§71.4.1 の最後の `replace`）、親 directory を sync する
7. §71.11.4 の本体（lock を取らない内部関数）を 1 回行い、その結果を返す

- 手順 4〜6 のどれかが失敗したら（同じ秒の再試行で退避名が既にある場合を含む）`OffsiteError::StateResetFailed`。手順 7 の失敗は §71.11.4 の失敗（`Io`・`StorageFull` 等）をそのまま返す。失敗の後の状態は 1 通りで、画面は失敗の種類でなく取り直した `offsite_status` で決める（68 §68.4 手順 4）:
  - 手順 6 の置き換えより前の失敗: 状態の file は元の読めない file のまま。`offsite_status` は `state_unreadable` を返し続け、ホームの知らせと「記録を作り直す」が残り、押し直せる
  - 手順 6 の置き換えの後の親 directory の sync の失敗: 状態は新しい内容で読め（`media` だけ、`last_success`・`last_failure` なし）、`offsite_status` は `prepared = true`・`stale = true`・`last_failure_kind = None` を返す（画面とホームは UI-11b-D14 の (1) 以外の原因の文、「記録を作り直す」は出ない）。次の確認（60 秒以内）が写して直る
  - 手順 7 の失敗: 状態は新しい内容で読め、§71.11.4 が `last_failure` を書く（状態の書込みの失敗を除く）。`offsite_status` は §71.11.5 の式どおり（一部の媒体だけ写せたときは `last_success` があり `stale = false` になりうる）。手順 7 の中で成功の記録を置き換えた後の状態の file の親 directory の sync の失敗は、失敗でなく成功のまま（`Copied`、§71.11.4 のエラーの表）。「記録を作り直す」は出ず、原因の文は `stale` のときだけ出る（UI-11b-D14）
- 呼ぶ前に新しい backup を作る（CMD、43 §43.8.6）: 媒体に最新の写しが既にあると手順 7 が `UpToDate` になり、`last_success` が空のまま「まだ写せていません」が残るため。作成に失敗したら CMD は本関数を呼ばない（記録に触れない）
- 持ち帰っていて見えていない媒体は登録されない。次に差したときに「この USB メモリを控えの保存先にする」で登録し直す（目印の `medium_id` と札を引き継ぐ、§71.11.2 手順 4）。それまで新しい USB メモリを用意しない（札が重なる、§71.13 の本番の復元と同じ）
- 退避した file は消さない（数は作り直しを試した回数だけ。owner が中身を確かめて消してよい）

#### 71.11.5b stop_offsite_backup（PC の外の控えをやめる）

**関数要求**: USB の控えは任意（owner 決定 2026-10-22、repo 外の回答台帳 TD-381。USB が無くてもアプリは動き、USB を繋ぐことを必須にしない）。用意した PC の外の控えを、バックアップ画面の操作だけでやめる。やめた後は用意の前と同じで、写さず、ホームも知らせない。

**シグネチャ**:
```
fn stop_offsite_backup(app_data_dir: &Path) -> Result<(), OffsiteError>
```

**処理ステップ**:
1. §71.11.4 の `Mutex<()>` を取る（入口で 1 回）
2. 状態の file が無ければ `Ok`（既に用意の前）
3. 状態の file を、読める・読めない・`format` に依らず、`offsite-backup.json.stopped-{YYYYMMDD_HHMMSS}` へ上書きしない操作（§71.4.1 の `publish_no_replace`）で移し、親 directory を sync する。同じ名前が既にある・移せなければ、何も変えず `OffsiteError::StopFailed`（状態の file は元の名前に残り、`offsite_status` は用意の後のまま）。移した後の親 directory の sync の失敗も `OffsiteError::StopFailed` で、そのとき `offsite_status` は用意の前（`prepared = false`）を返し、電源断で移動が戻れば状態の file が元の名前に戻って用意の後になり、「PC の外の控えをやめる」をもう一度押せる
4. `Ok`。以後 `offsite_status` は `prepared = false`・`stale = false`、`check_offsite_backup` は `NotPrepared`

- 失敗の後の画面は、失敗の種類でなく取り直した `offsite_status` で決める（68 §68.4 手順 5）

- 媒体の上の `InventoryBackup\`（目印と写し）は消さない・書かない（やめる間に媒体が差してある必要も無い）。後でまた「この USB メモリを控えの保存先にする」で用意すれば、目印から同じ媒体（同じ `medium_id` と札）として使える（§71.11.2 手順 4）
- PC の中の backup（`backup_dir`・自動 backup・保持日数の掃除）には一切触れない
- 移した状態の file は消さない（owner が中身を確かめて消してよい）
- 読めない記録でも、作り直し（§71.11.5a）に要る媒体が無くても、やめられる

#### 71.11.5c check_backup_dir_choice（保存先に選ぼうとする folder を確かめる）

**関数要求**: バックアップ画面の「保存先を選ぶ」で選んだ folder が、差してある PC の外の控えの媒体の `InventoryBackup` と同じ実体の folder なら、保存する前に止める（MNT-01-D11。用意の後に保存先を選ぶ順でも、起きたその画面で原因と直し方を名指す、D-127）。書込みはしない。

**シグネチャ**:
```
fn check_backup_dir_choice(selected: &Path) -> Result<(), OffsiteError>
```

**処理ステップ**:
1. 見えている媒体を §71.11.5a 手順 3 と同じ規則で探す（取外し可能・レジの SD でない・`InventoryBackup` と目印が reparse point でない・目印が `format` 1 で読める。PC 側の状態の file は読まず、状態の `media` との照合を外す）。目印のある `InventoryBackup` は、この PC の控えでも別の PC の控えでも、PC の中の保持日数の掃除（§71.5）が日付で写しを消すため
2. どれかの媒体の `InventoryBackup` と `selected` が同じ実体の folder（§71.11.3 の「PC の中の保存先と同じ folder の判定」。`selected` 側は `selected` を固定した `actual`）なら `OffsiteError::BackupDirIsMediumFolder`
3. `Ok(())`

- lock を取らない（書かない。`offsite_status` と同じ）。thread の error mode の guard は §71.11.3 のとおり使う
- 媒体の探し方の失敗（読めない root）は §71.11.3 のとおり黙って飛ばし、`Ok` に倒す（保存を止めるのは同じ folder と分かったときだけ。用意した後に同じ folder になっても、次の確認〈§71.11.4 手順 3〉が止めて知らせる）
- 既に保存してある `backup_path` は書き換えない（MNT-01-D10 の棄却案と同じく、選んだ設定を壊さない。その組み合わせは確認が止めて知らせる）

#### 71.11.6 運用の型と脅威（D-114 の比較の要旨）

アプリの仕組みは同じで、店は C を採った（owner 決定 2026-10-08、repo 外の回答台帳 TD-196、D-114）。○ = 守る、△ = 条件つき、× = 守らない。

| 脅威・負担 | A 差しっぱなし（1 本、毎日自動） | B 週 1 回など手で差す（1 本） | C 2 本の入れ替え（1 本を差しっぱなし、1 本を owner が持ち帰り、訪問時に入れ替え） | C' 2 本の入れ替え（外した 1 本を店の中の PC と別の場所に置く） |
|---|---|---|---|---|
| PC の故障・DB の破損 | ○ 前日まで | △ 最大 1 週間を失う（店主の許容の 3 日を超える） | ○ 前日まで | ○ 前日まで |
| 古い時点へ戻したい（誤操作の後） | ○ 媒体に 30 世代 | △ 週ごとの世代 | ○ | ○ |
| PC の盗難 | × 差した媒体ごと持ち去られやすい | △ 媒体を別に置けば守る | ○ 持ち帰りの 1 本が残る（最後の入れ替えの時点まで） | △ 置き場所が見つからなければ守る |
| 火事・水害（店） | × | × 店の中なら | ○ 持ち帰りの 1 本（同上） | × |
| ランサムウェア | × 差している媒体も暗号化されうる | ○ 差していない間 | ○ 外した 1 本 | ○ 外した 1 本 |
| 店主の手間 | なし | 毎週差して抜く（できるか分からない、repo 外の回答台帳 TD-187） | なし（入れ替えは owner） | なし（入れ替えは owner） |
| 前提・残るもの | ノート PC を家へ持ち帰る日（TD-009）は媒体も一緒に動く | 店主の毎週の操作 | owner の訪問の間隔が、盗難・火事で失う期間になる。間隔は決まっていない（TD-199）ので、失う日数に上限が無い。店の外へ店のデータを持ち出す（「外部に保存するなら置き場所は店」TD-011 の見直しと、媒体を失くしたときの露出〈暗号化しない〉を owner が受けた、TD-196）。店の PC の空きの USB の口は 1 つ（TD-198）で差しっぱなしの 1 本が占めるので、用意と入れ替えは 1 本を抜いてから別の 1 本を差す | 店の中の別の置き場所。火事は守らない |

アプリ側の知らせは、どの型でも「差してある媒体へ写せたか」と「最後に写せた日」だけで、持ち帰った媒体の古さはアプリから見えない（C・C' の残るもの）。C で入れ替えが途絶えても、差しっぱなしの 1 本への毎日の写しとホームの知らせ（UI-00-D12）は続き、守りは A と同じに戻る（アプリの振舞いは変わらない）。

#### 71.11.7 保証の範囲

- 読み戻しの照合は、写した直後に OS が返す bytes が元と同じことを確かめる。Windows の file cache から返る可能性があり、媒体の記憶素子の故障までは保証しない。媒体を抜き差しした後の「控えを確かめる」（§71.12）は媒体から読み直す（packet の Contract Probe の P2）
- 写すのは確かめ済みの最新の backup で、その後の入力（その日の作業）は次の backup（起動時・設定時刻、§71.8）まで PC の中だけにある
- 目印の無い媒体・別の PC で用意した媒体・`format` が 1 でない目印の媒体・レジの SD・固定 disk・network には書かない（例外: 記録の作り直し〈§71.11.5a〉は状態との照合を外すので、別の PC で用意した目印の媒体も差してあれば登録して写す。owner の明示の操作）。レジの SD は、固定 disk の root の直下に置いた SD の写しも含めて書く先にしない（reader が固定 disk に見える PC と見分けられないため、MNT-01-D10）。SD の写しは root の直下に置かない。媒体の上でも `InventoryBackup\` の外には書かない。探して照合した後に抜かれ、同じ drive 文字に別の媒体が割り当たっても、その媒体には書かない（volume の識別と目印の読み直し、§71.11.3）
- 媒体の上の正式名は、共有の検査に通った写す元と同じ bytes で、既存の正式名を上書きしない（§71.4.1）。例外: 検査の後・copy の前に PC の中の写す元の file が替わった場合（同じ process の中で backup の file を書き換える経路は無く、利用者の手の操作に限る）と、利用者が `InventoryBackup\` に手で置いた規約の名前の file（下の `InventoryBackup\` の項。検査を通らないまま写しと数えられうる。復元の前の確かめ〈§71.12〉が壊れた・このアプリのものでない file を止める）。公開の後の親 directory の sync の失敗では、検査済みの正式名が媒体に残り、`last_success` は次に写せた時まで変わらない（§71.11.4 のエラーの表）
- 用意した後に状態の file が壊れたら、写さずに知らせる（黙って止まらない）。ホームとバックアップ画面は「記録を読めない」側の文と、直す操作「記録を作り直す」を出す（UI-00-D12、68 §68.11）。作り直しは読めない file を退避して差してある控えを登録し直す（§71.11.5a）。`format` が 1 でない数の記録（新しい版のアプリの記録）は作り直さず、新しい版のアプリを入れる文を出す。用意の前（状態の file が無い）は何もせず、ホームも知らせない
- 作り直しは用意済みの媒体が 1 本も見えないと止まる（`NoMediumAttached`。知らせが消える側でなく、残る側に倒した）。媒体を全部失ったとき・USB の控えを使わないときは、画面の「PC の外の控えをやめる」（§71.11.5b）で知らせを止める
- 作り直しのときに差していない媒体は登録されないまま残り、入れ替えの後はその媒体が「用意済み」にならず USB の確認の文が出る（差してあるのに未登録の目印の媒体を名指す文は範囲の外。出所: Final Review の closure〈AI の review〉が出した改善の候補で、owner の確認の前の要件にしない）
- 原因が PC の中の backup の古さ（自動 backup の無効・保存先への書込みの失敗の継続）のときは、USB を差し直しても直らないので、画面とホームは USB の文でなく PC の中の backup の直し方を出す（§71.11.5 の `local_backup_old`）。その文が導く「今すぐバックアップを作成」が失敗すれば、その文が原因を名指す（レジの SD の判定〈MNT-01-D10〉なら見つけた folder の path と直し方、43 §43.6）
- 媒体の `InventoryBackup\` はアプリが持つ。中の §71.3 の規約の名前の file は、手で置いた物も写しとして数え（同じ名前・同じ size なら写さない、§71.11.4 手順 4a）、新しい 30 本を残して古いものを消す（同 手順 4g）。手で保つ控え（更新の前の backup 等）は `InventoryBackup\` の外に置く
- PC の中の保存先（`backup_path`）を媒体の `InventoryBackup` と同じ folder にしない（MNT-01-D11）: 保存先を選ぶとき（§71.11.5c）・用意するとき（§71.11.2 手順 3c）・確認のたび（§71.11.4 手順 3）に、固定した path で比べて止め、card とホームが「保存先を選ぶ」で選び直す直し方を名指す（43 §43.8.2〜§43.8.7 の表、68 UI-11b-D14 の (4)、53 UI-00-D12 の (4)）。例外: 保存先を選んだ時点で目印の無かった folder（手で作った `InventoryBackup` 等）が後で用意された場合と、選んだ後に drive 文字の割当てが替わって保存先の path が別の用意済みの媒体の `InventoryBackup` を指すようになった場合は、選ぶときに止まらない。前者は用意が、後者は次の確認が止めて知らせる。確認が止めている間も PC の中の保持日数の掃除（§71.5。本決定は変えない）はその folder の古い写しを日付で消すので、保存先を選び直すまで、その媒体の写しは保持日数の分に減る（data は PC の中の backup と同じ file で、失うのは媒体の上の古い世代）
- 「PC の外の控えをやめる」（§71.11.5b）は全部の媒体の登録をまとめて外す。1 本だけを外す操作は持たない（1 本だけ外したいときは、やめてから残す媒体を用意し直す。見直し契機に置く）
- 新しい PC で戻した直後は、その PC の起動時の自動 backup と復元の事前バックアップ（どちらも空の DB の控え）が今日の backup と数えられ、写す元の最新になる。アプリは空の DB の控えを区別しない（店の記録の有無を判定する業務の規則を backup に持たない）ので、§71.13 の本番の復元の手順で、媒体を登録し直す前に手動の backup を作る
- 持ち帰った媒体の古さはアプリから見えない。店の型 C では、PC と差しっぱなしの 1 本を同時に失ったとき（盗難・火事）に戻れるのは最後の入れ替えの時点までで、訪問の間隔は決まっていない（TD-199）ので失う日数に上限は無い（§71.11.6、D-114 の Guarantee range）

**MNT-01-D8: PC の外の控えは、目印を置いた取外し可能な媒体へ、確かめ済みの backup を写して読み戻しで照合する（D-114）**

- 決定: 上の §71.11.1〜§71.11.7。媒体は目印（`InventoryBackup\offsite-medium.json` の `medium_id`）と PC 側の状態の一覧の両方で見分け、drive 文字で覚えず、1 回の確認の中は volume の識別に書込み先を固定する。写すのは共有の検査（§71.12）に通った正式名の backup（MNT-01-D7）だけで、作業名へ写して SHA-256 で読み戻しを照合してから上書きしない公開で正式名にする。状態は DB の外（`offsite-backup.json`）に置く。媒体ごとに新しい 30 本を残す。最後に写せた日（写した控えの日付の方が古ければその日付、§71.11.5）から 3 日以上で「控えが古い」とする
- Why: owner 決定（2026-10-07、repo 外の回答台帳 TD-190 の Q4）「外付けの保存先へ自動で書き、確かめて画面に出す」。店主が毎回手で backup を取れるかは分からない（TD-187）ので、差してあれば何もしなくても写る形にする。drive 文字は入れ替えで変わり、レジの SD と同じ文字になりうる（MNT-01-D10）。状態を DB に置くと復元で巻き戻り、用意した媒体を忘れて黙って写さなくなる。3 日は店主の許容（3 日程度前まで戻れれば OK、L-220）で、店の定休日（1 日）や連休の朝でも、起動時の確認ですぐ写るので知らせが出続けない
- Rejected alternatives: `backup_path` を USB メモリにする（照合が無く、作成途中の file が正式名で残り〈D7 の前〉、drive 文字が変わると別の drive やレジの SD へ書く）／ 固定 disk・network（UNC）も保存先にする（PC の内蔵の別 partition は PC の外でない。UNC は `VACUUM INTO` が失敗する既知の問題〈`docs/backlog.md` の保留〉）／ 写した後に照合しない（「file がある」と「使える控え」を分けない）／ 読み戻しの代わりに size だけ比べる（中身の化けを見ない）／ クラウドへ置く（外部のサービスに頼らない owner の方針、本 lane の範囲外）／ 媒体の上で日付で掃除する（入れ替えで久しぶりに差した媒体の古い写しを、今日の 1 本を除き一度に消す。数で残す方が古い時点を残す）／ 写しを暗号化する（鍵を PC と別に保つ運用が要る。持ち出すときの露出は owner が受けた〈D-114 の owner の決定 1、TD-196〉）
- Compatibility: 既存の command・DTO・`app_settings` の key・DB の schema は変えない（command と DTO を足す、43 §43.8.2〜§43.8.7）。`backup_path`（PC の中の保存先）とその一覧・掃除・復元は今のまま。DB の外に `offsite-backup.json` を足す（restore はこの file を変えない）
- 見直し契機: 媒体の容量が 30 本に足りないとき。持ち帰った媒体の古さもアプリで扱いたくなったとき（入れ替えの日を記録する等）。USB の HDD（固定 disk に見える）を使いたくなったとき。媒体を 1 本だけ登録から外す操作が要るとき。レジの機種が変わるとき（MNT-01-D10 の folder）

**MNT-01-D11: PC の中の保存先を、PC の外の控えの媒体の `InventoryBackup` と同じ folder にしない（D-130）**

- 決定: PC の中の保存先（`backup_dir`）と媒体の `{root}\InventoryBackup` を、どちらも `pin_to_volume` で固定した path で比べ（§71.11.3 の「PC の中の保存先と同じ folder の判定」）、同じ実体の folder なら書かずに `OffsiteError::BackupDirIsMediumFolder` を返す。判定の時点は 3 つで、同じ関数を使う: 保存先を選ぶとき（§71.11.5c。保存する前に止める）、用意するとき（§71.11.2 手順 3c。手順 4 の書込みの前）、確認のたび（§71.11.4 手順 3。写す元の検査と写しの前）。確認の失敗は `last_failure` に残し、card とホームは「保存先を選ぶ」で PC の中の別の folder を選び直す直し方を名指す（43 §43.8.2〜§43.8.7 の表、68 UI-11b-D14 の (4)、53 UI-00-D12 の (4)）
- Why: owner 決定（2026-10-23、repo 外の回答台帳 TD-398。同じ日の TD-396 の B〈起票の推奨〉を置き換えた）。同じ folder のままだと、PC の中の保持日数の掃除（§71.5、既定 3 日）が媒体の写しを日付で消し（`OFFSITE_KEEP` の 30 本が 3 日分になる）、写す元と写す先が同じ file なので手順 4a が毎回 `UpToDate` になって `last_success` が書かれず、3 日後に「差してあるのに写せない」の知らせ（UI-11b-D14 の (5)）が原因を名指さずに出る。回避を手順書の文に回さず、起きたその画面で原因と直し方を名指す（D-127）。保存先を選ぶ時点でも止めるのは、用意の後に保存先を選ぶ順では、確認が止めても保持日数の掃除が媒体の写しを消し続けるため（確認の失敗では掃除を止められない）
- Rejected alternatives: §71.11.7 に「PC の中の保存先にこの folder を選ばない」と 1 文足すだけ（起票の推奨の B。回避を文に回し、起きた画面で名指さない）／ backlog に残す（C）／ `backup_path` の文字列と drive 文字の path を比べる（junction・symbolic link・大文字・小文字の違いで見落とす）／ 用意と確認だけで止め、保存先の選択では止めない（上の Why の掃除）／ 保存先の選択だけで止める（用意の前に選んだ保存先と、drive 文字の割当ての変化を見落とす）／ 保存先に取外し可能な drive を選べなくする（既に選んだ人の設定を壊す。MNT-01-D10 の棄却案と同じ）／ 同じ folder のときは PC の中の保持日数の掃除を止める（§71.5 の契約を変え、PC の中の backup が無制限に溜まる）
- Compatibility: 既存の `backup_path` の値は書き換えない。用意の前の店（状態の file が無い）は、保存先の選択だけが目印のある媒体の `InventoryBackup` を止め、ほかは今のまま。`OffsiteFailureKind` に 1 値、`OffsiteError` に 1 値、command を 1 つ（43 §43.8.8）足す。`CmdErrorKind` は変えない
- 見直し契機: PC の中の保存先に USB メモリそのものを使う運用を owner が求めたとき。媒体の root や `InventoryBackup` の中の folder を保存先にした組み合わせで、互いの掃除が file を消す経路が見つかったとき

---

### 71.12 inspect_backup（控えを確かめる、MNT-01-D9）

**関数要求**: 選んだ backup file を変えずに開き、この版のアプリで戻せるか（版）、壊れていないか（`quick_check`）、何が入っているか（商品の数・最後の記録）を返す。復元の前の確かめ（UI-11b-D15）と、復元の予行演習の短い確かめ（§71.13）と、`create_backup` の作業名の検査（MNT-01-D7 の手順 4a）と、PC の外へ写す元の検査（§71.11.4 手順 3a）が使う。どの入口も同じ 1 つの関数と下の「使える控え」の判定を使い、別の判定を書かない。

**シグネチャ**:
```
fn inspect_backup(backup_path: &Path) -> Result<BackupInspection, DbError>

#[derive(Debug, serde::Serialize, specta::Type)]
struct BackupInspection {
    file_name: String,
    created_at: Option<String>,       // §71.3 の規約に合えば YYYY-MM-DD HH:MM:SS
    size_bytes: u64,
    schema_version: i64,
    app_max_version: i64,
    newer_than_app: bool,             // schema_version > app_max_version
    quick_check_ok: bool,
    product_count: Option<i64>,       // 版がアプリ以下で quick_check が ok のときだけ
    last_operation_at: Option<String>,// 同上。operation_logs の created_at の最大
}
```

**処理ステップ**:
1. `backup_path` を読取り専用・作成なし・`immutable=1` の URI で開く（journal・`-wal`・`-shm` を作らず、file と folder を変えない。USB の上でもそのまま開く）。URI の組み方: `backup_path` の文字列（Windows は `\` のまま。`\`→`/`・`//` の圧縮・drive 文字の前の `/` を行わない）の `%`・`?`・`#` を 1 回の走査で `%25`・`%3f`・`%23` に置き換え、前に `file:`、後ろに `?immutable=1` を付け、`SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_URI` で開く。理由: `create_backup` の手順 4a に渡る作業名は Windows では `\\?\Volume{GUID}\…`（§71.4 の「書く先の固定」）で `?` を含み、符号化しないと `?` 以降が query になる。SQLite の URI の解析は `file:` の直後が `//` のときだけ authority を見て、それ以外は `%HH` の復号と `?`・`#` の区切りだけで `\` を触らない（https://www.sqlite.org/uri.html ）。`\` を `/` に変えると `\\?\` の前置が Windows の VFS に認められず別の path に解決する。`SQLITE_OPEN_READ_ONLY` を落とすと `immutable=1` でも無い file を作る。URI を使わない読取り専用の open は WAL の header の file で `-wal`・`-shm` を作りうる（https://www.sqlite.org/wal.html ）ので、URI + `immutable=1` の形に限る
2. 版を `read_current_version_without_ddl`（22 §3.2 の手順 1。migrate と同じ関数を `pub(crate)` で共有し、同じ判定を二重に書かない）で読み、`app_max_version()` と比べる
2a. **このアプリの backup か**: 版が 0（`schema_versions` の表が無い〈空の file・別用途の SQLite の DB〉、または表が空。`read_current_version_without_ddl` はどちらも 0 を返す、`db/migration.rs` の `read_current_version_without_ddl`・`get_current_version`）なら `DbError::QueryFailed`（読めない控え）。版が `app_max_version()` 以下なら、必須の表 `products`・`operation_logs`・`app_settings`（v1 から在る表、`db/schema_v1.rs`）が `sqlite_master` に全部あることを確かめ、無ければ `DbError::QueryFailed`。新しすぎる版は表を見ない（新しい版の表の形を古い版が決めつけない）。版 0 を戻すと既存の restore の migrate が新規の DB として表を作り、店の記録の無い DB へ置き換わるため
3. `PRAGMA quick_check` の結果が 1 行の `ok` なら `quick_check_ok = true`
4. `newer_than_app = false` かつ `quick_check_ok` のときだけ `SELECT COUNT(*) FROM products` と `SELECT MAX(created_at) FROM operation_logs` を読む。失敗は `None`（検査の結果を変えない）
5. `BackupInspection` を返す

**使える控え**（共有の判定）: `inspect_backup` が `Ok` で、`newer_than_app = false` かつ `quick_check_ok = true`。復元の詳細（UI-11b-D15）はこれを満たす控えだけを確認の手順へ進め、`create_backup` の手順 4a と §71.11.4 の手順 3a は満たさない file を公開・写し・成功の記録・掃除へ進めない。

**エラーハンドリング**: 開けない・版を読めない（`schema_versions` の確認の失敗を含む）・このアプリの backup でない（手順 2a）→ `DbError`（CMD は「この控えを読めませんでした」、43 §43.8.5）。file の metadata の失敗 → `DbError::QueryFailed`（MNT-01-D6 と同じく 0 に倒さない）。

**MNT-01-D9: 復元の前に控えを確かめ、新しすぎる版と壊れた控えを、確認の手順へ進む前に固有の文言で止める（D-114）**

- 決定: 復元の詳細（UI-11b の `restore_detail`）は、選んだ控え（一覧の行・選んだ file のどちらも）に `inspect_backup` を行う。`newer_than_app` なら「この控えは、より新しい版のアプリで作られています。この版のアプリでは戻せません。新しい版のアプリを入れてから戻してください（今のデータは変わっていません）。」、`quick_check_ok = false` なら「この控えは壊れているため戻せません。別の控えを選んでください。」、`inspect_backup` が `Err`（このアプリの backup でない空の・別用途の DB を含む、§71.12 手順 2a）なら「この控えを読めませんでした。別の控えを選んでください。」を出し、復元へ進む button を出さない（事前バックアップも作らない）。差し替えの後の open の拒否（MNT-03-D11）は今のまま最後の守りとして残し、確かめた後に file が替わって新しすぎる版・開けない file になった場合などは、その経路の既存の文言（`restore_failed_recovered`）になる。確かめた後に file が替わった場合（同じ drive 文字の媒体の入れ替えを含む）、差し替えの後の open が止めるのは新しすぎる版と開けない file までで、このアプリのものでない DB は止まらない（restore の契約は変えない、Non-scope。復元の前の事前バックアップ〈D-032〉から戻せる）。作り直しの起点の migration（22 §16 の vB、§19 の v9）より前の版の開発・試験 DB の控えで、その migration の止まる検査（22 §16 の vB、§19）に当たるものも、確かめは「戻せます」になり、差し替えの後の migrate で止まって同じ文言になる（確かめは版・このアプリの backup か〈§71.12 手順 2a〉・`quick_check` を見て、migration の止まる条件は見ない）。店の初回の版はそれらの migration を含む（D-123 (4)）ので、店で作る控えはこの形にならない。作り直しの手順（22 §16）は古い控えも移す
- Why: 新しすぎる版の backup の復元は、事前バックアップ・2 段の確認・差し替えの後の open で初めて拒否され、画面は「もう一度お試しください」になる。同じ backup で何度試しても失敗するのに、版の事情が利用者に伝わらない（`docs/backlog.md` の「保存と起動の守りの follow-up」の (2)）。壊れた backup は差し替えの後の open で見つからず、壊れた DB に戻りうる。確認の手順へ進む前に止めれば、DB にも事前バックアップにも触れない
- Rejected alternatives: restore の error に新しい kind（例: 新しすぎる版）を足す（MNT-01-D4 の 3 値の wire と復旧の分類を変える。文言は事前バックアップと 2 段の確認の後にしか出ない）／ `restore_failed_recovered` の message の中身で分ける（MNT-01-D4 が禁じる文字列の判定）／ 版の判定を restore にもう 1 つ書く（MNT-03-D11 が避けた二重化。本決定は同じ関数を共有する）／ 確かめに `integrity_check` を使う（遅い。MNT-01-D7 と同じ理由で `quick_check`）
- Compatibility: `restore_backup` の command・`RestoreBackupRequest`・`RestoreError` の 3 値・`CmdErrorKind` の restore の 3 値・UI-11b-D2〜D5 の 2 段の確認と break-glass は変えない。MNT-03-D11 の「restore に版の事前検査は足さない」は「restore の中には足さず、復元の前の確かめ（本決定）が同じ関数で文言と早い停止を受け持つ」に改めた（22 §3.2）
- 見直し契機: 古い版のアプリで新しい DB を読む互換を設けるとき（MNT-03-D11 と同じ）。backup の file の形（`VACUUM INTO`）を替えるとき

---

### 71.13 復元の予行演習（D-114）

PC の外の控えが「ある」だけでなく「戻せる」ことを、店の今のデータを変えずに確かめる手順。実施者と頻度は下の各段のとおり（owner 決定 2026-10-08、repo 外の回答台帳 TD-197、D-114）。

| 段 | いつ | だれ | 手順 | 合格 |
|---|---|---|---|---|
| 短い確かめ | 媒体を入れ替えるたび（店の型は C、§71.11.6。差しっぱなしだけの A なら月に 1 回） | owner（店主でもできる） | (1) 持ち帰っていた媒体を差した直後、アプリが今日の控えを写す前（差してから 60 秒の確認の前）に、バックアップ画面の「控えを選んで確かめる」→ 差した媒体の `InventoryBackup` の最新の file を選ぶ（持ち帰っている間の媒体の記憶を、媒体から読ませて確かめる）。確かめの間は 60 秒の確認が止まる（68 UI-11b-D15）ので、結果を見たら詳細を閉じて画面を `ready` に戻す（確認が再開する）。(2) card が「差してある控え: 控え N」と今日の日時になるのを待つ（再開から 60 秒以内）。A は (1) の代わりに、差してある媒体の最新の file を選ぶ | (1)「この控えは戻せます」と、作成日時がこの媒体を前に差していた最後の日（前回の入れ替えの日。A なら前の営業日）以降で、最後の記録がその日の作業と合う。(1) で 60 秒の写しが先に済んでいた場合は、最新が今日の控えになるので、2 番目に新しい file で (1) を見る。(2) card に今日の日時と、差した媒体の札 |
| 通しの演習 | 運用を始める前（go-live の前）と、migration を含むアプリの更新の後 | owner | 店の PC とは別の PC に同じ版のアプリを入れ、「控えを選んで確かめる」で媒体の最新の控えを選び、確かめの結果を見てから 2 段の確認で戻し、ホーム・商品・日次売上の画面で最後の営業日の記録が見えることを確かめる。終わったらその PC のアプリのデータ folder を消す（店のデータを残さない） | 戻った後の画面で、最後の営業日の売上と商品の数が店の PC と合う |
| 本番の復元（PC を失ったとき） | — | owner | 新しい PC にアプリを入れて起動 → バックアップ画面の「控えを選んで確かめる」→ 媒体の最新の控え → 確かめの結果を見る → 2 段の確認 → 戻る → **バックアップ画面の「今すぐバックアップを作成」を押す**（新しい PC の起動時の自動 backup と復元の事前バックアップは空の DB の控えで、今日の backup と数えられ、押さないと媒体へ写る最新が空の控えになる）。押す前に「現在の保存先」が新しい PC の中の folder かを見て、古い PC の path（戻した DB の `backup_path`）なら「保存先を選ぶ」で選び直す →「この USB メモリを控えの保存先にする」で媒体を登録し直す（目印の `medium_id` と札の名前を引き継ぐ）。残っている控えはすべて登録し直してから、新しい USB メモリを用意する（先に新しい媒体を用意すると、後で登録し直す控えと札が重なる） | 戻った後、ホームの「PC の外の控え」の知らせが消える（次の確認で写る）。登録し直した後、「控えを選んで確かめる」で媒体の最新の控えを開き、商品の数と最後の記録の日時が戻した結果と合う（空の控えが写っていない） |

- 店の PC で通しの演習（実際の復元）をしない: 復元は店の今のデータを控えの時点へ戻す破壊的な操作（UI-11b、D-032）で、演習のために店の記録を巻き戻さない
- 別の PC に店のデータを置く間の扱い（消す時期）は owner の判断。repo に実データ・backup file を置かない（AGENTS.md の Safety）
