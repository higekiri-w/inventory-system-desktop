## 16. BIZ-04: PLU書出しロジック

> **D-072 / D-117 / REQ-907**: app 管理 slot は app、既存登録はレジを authority とする。毎日の締めの SD の読取りで Z004 の全スロットの占有を照合した後、JAN 単位の memory No. を永続予約し、Diff / Full とも CV17 へ投入できる（SPEC-PLS-D2、D3、D5）。アプリは枠を消さず（clear 行を書かない）、別の JAN に使い回さない（D-117 (1)）。
>
> **runtime の状態（2026-10-16）**: 本節は D-117 の後の形。code には撤去の対象（clear 行・`release_pending`・解放の trigger・Z004 を選ぶ command・件数の要約・`over_limit_warning`）がまだあり、後続の runtime の lane が code・migration・bindings・test と一緒に外す。占有の自動の照合（§16.3）は SD を直接読む runtime（D-111 の後続の lane）に依り、それまで code は今の FilePicker の経路を残す。

### 16.1 モジュール構成

```
src-tauri/src/
  biz/
    plu_export_service.rs
```

BIZ-04 は transaction と業務状態遷移を所有する。固定幅 Z004 の解釈は IO-02、slot CRUD は `db::plu_slot_repo`、CV17 向け 11 field 生成は IO-04 に委譲する。

**関数要求**: レジ占有 snapshot の照合、PLU slot の冪等予約、Diff / Full 行準備、保存後 confirm を一貫した transaction 境界で提供する。

**シグネチャ**: 各公開関数の Rust 形状は §16.3〜§16.5 に定義する。

**処理ステップ**: 占有の照合（毎日の SD の読取り）→ prepare → external save / register operation → confirm の順で処理し、詳細は §16.3〜§16.6 に定義する。

**エラーハンドリング**: validation / snapshot conflict / no-free-slot / IO・DB failure を区別し、transaction failure は部分状態を残さず rollback する。回復契約は §16.8 を正本とする。

### 16.2 型定義

```rust
enum PluExportMode { Diff, Full }

enum PluSlotStatus { Free, External, Reserved, Active }

enum PluExcludedReason {
    MissingJan,
    InvalidJanFormat,
    InvalidCheckDigit,
    GroupPriceMismatch,
    NoFreeSlot,
}

struct PluPreparedRow {
    memory_no: i64,
    target_product_codes: Vec<String>,
}

struct PluExportPreparedResult {
    plu_output: PluFileOutput,
    count: usize,
    target_product_codes: Vec<String>,
    prepared_rows: Vec<PluPreparedRow>,
    excluded: Vec<PluExcludedProduct>,
}

struct PluRegisterSnapshotSummary {
    snapshot_at: Option<String>,
}
```

`PluRegisterSnapshotSummary` は最後に照合した日時だけを返す。空き・外部・アプリ管理・競合の件数は返さず、画面に出さない（D-117 (5)、UI-08-D11）。件数は照合の operation_logs にだけ残す（§16.3）。

`NoFreeSlot` の operator 文言は「レジの空きスロットがありません」とする。件数上限との比較ではなく、snapshot 後に `free` slot がないという実状態を表す。

「対象の JAN」は、`plu_target=1`・未廃番の商品が持つ、13 桁数字かつ check digit 有効な JAN とする（§16.4 step 2 と同じ）。

### 16.3 占有の照合（BIZ-04-D3 / SPEC-PLS-D2、D-117 (3)）

#### 入力の経路

毎日の締めの売上データ取込みで、CMD-12 がレジの SD を読んだとき（[45 §45.6a](45-cmd-daily-report-import.md) `scan_register_sd`）に照合する。BIZ-08 の SD の読取り（[37 §37.9](37-biz-daily-report-import-service.md)）は、読取りの窓の中の Z004（`XZ` と `XZ_BKUP`）を読取り専用で読み、bytes を scan の snapshot の `plu_register_z004s` に入れて返す（DB へは書かない）。CMD-12 は scan の成功の後、同じ DB 接続を持ったまま `refresh_plu_register_snapshot_from_sd` を呼ぶ。店主が Z004 の file を選んで読む step は持たない（UI-08-D11）。

```rust
pub fn refresh_plu_register_snapshot_from_sd(
    conn: &mut DbConnection,
    z004_files: &[Vec<u8>],
) -> Result<Option<PluRegisterSnapshotSummary>, BizError>
```

1. 各 bytes の精算の識別メタ（IO-02 の `settlement_metadata` の `settlement_no`。[23](23-io-z004-parser.md) の「精算の識別メタ」。23 の型は文字列のまま変えない）を読み、BIZ-04 の中で 10 進の整数として読む（ASCII の数字だけで、`i64` に収まる値。前の 0 は許す）。読めない bytes・読めない値は使わない。
2. 読めた中で `settlement_no` が最大の 1 本を選ぶ（同じ値が 2 本以上あれば、bytes が同じならどれでもよく、違えば照合しない。`tracing::warn!` で記録する）。精算日時や file 名の日付では選ばない（レジの時計に依らない）。選んだ 1 本の精算回数を、`app_settings` の key `plu_register_snapshot_settlement_no`（最後に照合した Z004 の精算回数、10 進の整数の文字列）と比べる: key が無ければ照合する。key の値より大きいときだけ照合する。小さければ照合しない（`tracing::warn!`、前回の照合を保つ。古い SD を読んだとき）。等しければ照合しない（既に写した観測。warn しない。同じ日に SD を読み直したとき、書込みの前のレジを写した Z004 で、confirm したばかりの枠を `missing_on_register` で未反映へ戻さないため）。key の値が整数として読めなければ照合しない（`tracing::warn!`）。
3. 照合しないとき・使える Z004 が無いときは `Ok(None)`（何も変えない。前回の照合を保つ）。照合するときは下の `import_plu_register_snapshot` に bytes と精算回数を渡し、その結果を `Some` で返す。
4. CMD-12 は `Err` を scan の失敗にしない（日報の候補の一覧は返す）。`tracing::warn!` で記録し、前回の照合を保つ。利用者に見えるのは UI-08 の最終照合日時（照合が古いときの警告を含む。UI-08-D11）だけである。

照合済みの記録は、最後に照合した Z004 の精算回数（上の key）だけを持つ。レジの初期化・修理で精算回数が戻ると、その後の Z004 は key より小さいので照合が止まる。UI-08 の最終照合日時が古くなって警告で見え、開発者が key を消して照合を戻す（D-117 の Revisit。自動の脱出は持たない）。占有の照合は Z004 の取込み（`csv_imports`・売上・在庫）を作らない（D-118 で、旧 ADR〈時点証拠〉の資料の受領と全数読取りは作らないことになった）。占有の照合（[37](37-biz-daily-report-import-service.md) §37.9 の手順 10 で読んだ Z004 を BIZ-04 が照合する）の後に Z004 を取り込む順と TX の分け方は、37 §37.10（手順 3。Z004 の bytes を持つ `z004_files` は PLU の照合の後も残す）と BIZ-08-D7（D-121。項目ごとの TX、順は日報 → Z004 → EJ）が決めた。runtime は [backlog](../backlog.md) の「Z004 を SD の一括取込みに入れる runtime（B）」。停止 ADR の Z004 の業務 commit の停止（SPEC-STOP-D1）は、売上・在庫を作らない占有の照合に及ばない。

#### import_plu_register_snapshot

```rust
fn import_plu_register_snapshot(
    conn: &mut DbConnection,
    raw_bytes: &[u8],
    settlement_no: i64,
) -> Result<PluRegisterSnapshotSummary, BizError>
```

BIZ の `import_plu_register_snapshot` は照合の本体として残し、`refresh_plu_register_snapshot_from_sd` だけが呼ぶ。同名の CMD（[41 §17.6](41-cmd-pos.md)、店主が FilePicker で選んだ Z004 を渡す command）は撤去する。

IO-02 の全スロット占有読取り mode が返す `(memory_no, raw_code)` を、**1 transaction** で `plu_slots` と照合する。途中失敗時は slot、app_settings、operation_logs をまとめて rollback する。

| レジ観測 | 現在 status | 結果 |
|---|---|---|
| 空 | free / external | free。scanning_code を NULL |
| 空 | reserved | reserved を維持 |
| 空 | active | active を維持し `missing_on_register`。その JAN の `plu_target=1` の商品を dirty |
| 同一コード | reserved | active。activated_at を更新 |
| 同一コード | active / external | 現 status を維持 |
| occupied | free | 対象の JAN と一致し、その JAN に `reserved` / `active` slot が無ければ active として採用（activated_at）。その JAN が `reserved` / `active` slot を既に持つなら、観測した枠は external（観測コードを保持。アプリの枠と memory No. は変えない。`duplicate_on_register`）。それ以外は external |
| 別コード | reserved | 観測値を external とし `reservation_dropped`。その JAN は次の prepare で枠を取り直す |
| 別コード | active（その JAN に対象の商品がある） | app の active を維持し conflict。その JAN の `plu_target=1` の商品を dirty |
| 別コード | active（その JAN に対象の商品が無い。売るのをやめた商品・JAN を直した商品の前の JAN の枠） | 観測値を external とする（`slot_released_on_register`）。アプリが書く商品が無いので、レジ側の登録を書き直さない |
| 別コード | external | 観測値で external を更新 |

表の判定は、どの枠も照合の前の状態（今の TX が始まる前の `plu_slots`）で行い、同じ照合の中で先に変えた枠の結果を後の枠の判定に使わない。ただし同じ Z004 の中で同じ JAN を複数の枠で観測したときの扱いは、次の最小番号の規則が 1 つに決める。free slot 上で同一の対象の JAN が複数観測され、その JAN に `reserved` / `active` slot が無い場合は、最小 memory No. だけを active として採用し、残りを external にする（観測コードを保持。`plu_slots` の partial UNIQUE は `reserved` / `active` だけにかかるので、同じコードの external は複数あってよい。[plu-tables](../db-design/plu-tables.md)）。

照合の規則は、割当て後・反映前のレジ側の変化（予約中の枠に別のコードが入る、アプリが持つ JAN をレジが別の枠に持つ）を含めて D-117 (4) で残す。アプリはレジ側の重複を消さない（clear 行を持たない）。売上は JAN で商品へたどるので在庫は誤らないが、レジがどちらの枠で売るかによってレジの値段が前の値段になりうる（D-117 の Guarantee range）。

照合後、同じ TX で `app_settings` の key `plu_register_snapshot_at` と `plu_register_snapshot_settlement_no`（渡された精算回数）を更新し、件数（free / external / app 管理 / conflict / missing / dropped / duplicate / released）を operation_logs（`plu_register_snapshot_import`）に記録する。実コード・名称・単価は log に含めない。

一度も照合していない状態の `prepare_plu_export` は `BizError::ValidationFailed`（reason=`register_snapshot_required`、文言「レジの登録状況がまだ読まれていません。売上データ取込みでレジの SD を一度読んでください」）で拒否する（読む前に空きを仮定しない gate。D-117 (3)）。

```rust
pub fn get_plu_slot_summary(conn: &DbConnection) -> Result<PluRegisterSnapshotSummary, BizError>
```

`plu_register_snapshot_at` だけを read-only で返す（未照合は `None`）。

### 16.4 prepare_plu_export（BIZ-04-D4 / SPEC-PLS-D3、D5）

```rust
fn prepare_plu_export(
    conn: &mut DbConnection,
    req: PluExportPrepareRequest,
) -> Result<PluExportPreparedResult, BizError>
```

prepare は **1 transaction** で対象抽出、JAN bucket / dedup、slot 予約、IO-04 行生成まで行う。

1. snapshot gate を検査する。
2. `plu_target=1`、未廃番、13 桁数字かつ check digit 有効な JAN を対象にする。JAN 不備と同一 JAN の売価・税率不一致は理由付き `excluded` とし、既存 slot を変更せず出力もしない。
3. 同一 JAN 群は D-028 の代表行規則を維持し、`target_product_codes` へ群の全 member を持つ。
4. 既存 `reserved` / `active` slot は sticky allocation として再利用する（対象外にした後に対象へ戻した商品も、前と同じ枠を使う）。
5. `reserved` / `active` を持たない JAN は、同一コードの `external` slot があれば新規予約せず、memory No. が最小のその slot を `active`（activated_at=now）として採用する（レジ既存登録の app 管理への移行）。`external` も無ければ memory No. が最小の `free` slot を `reserved` にする。空きがなければ `NoFreeSlot` へ積み、他の JAN の生成を続ける。固定件数との比較で prepare 全体を拒否しない。
6. Diff は dirty な対象 product 行、Full は対象の JAN の `reserved` / `active` slot の product 行を作る。`external` / `free` と、対象の商品を持たない JAN の slot（売るのをやめた商品・JAN を変えた商品の前の JAN）はどちらにも含めない。
7. 要修正判定中の JAN に既存 slot がある場合、その slot を維持し、Diff / Full とも出力しない。
8. IO-04 へ `memory_no` 付き行を渡す。成功時に予約を commit し、同じ snapshot と商品状態での再 prepare は同じ memory No. を返す。

対象行がない場合は validation error とする。prepare 成功は CV17 / レジ反映の証明ではなく、商品 `plu_dirty` と `plu_exported_at` は confirm まで変更しない。

### 16.5 confirm_plu_export_saved（BIZ-04-D5 / SPEC-PLS-D4、D5）

```rust
fn confirm_plu_export_saved(
    conn: &mut DbConnection,
    req: PluExportConfirmRequest,
) -> Result<PluExportConfirmResult, BizError>
```

prepare が返した exact `memory_no` / product_code set を **1 transaction** で再検証する。重複、範囲外、存在しない slot、期待 status と不一致の payload は全体を拒否する。

- product 行: `reserved -> active`、`activated_at=now`（照合で既に active なら維持）。対象群を `plu_dirty=0`, `plu_exported_at=now` にする。
- retry: 既に同じ結果へ遷移済みなら idempotent success とする。別 JAN / 別 status への変化は拒否する。
- operation_logs: mode、product 件数、confirmed_at を記録し、実コードを記録しない。

書出しと「未反映から外す」の 2 段は D-027 の理由で残す（CV17 の取込みに失敗しても、外すまでは同じ商品が差分に残る）。通常の経路では confirm で既に `active` なので、次の締めの照合は「同一コード × active」（維持）になり、反映したコードと名称が観測で裏づく。confirm をしなかった枠だけが、照合の「同一コード × reserved」で `reserved -> active` になる。単価・課税方式・部門リンクの反映は Z004 に無く、精算の前には分からないので、照合を「未反映から外す」の代わりにしない（D-117 (6)・(7)）。

### 16.6 商品の状態が変わったときの枠（BIZ-04-D6 / SPEC-PLS-D4、D-117 (1)）

枠は JAN ごとに永続し、アプリは解放しない。BIZ-01 から BIZ-04 の解放を呼ぶ契機は無い。

| 商品の変化 | 枠 | 商品 |
|---|---|---|
| `plu_target: 1 -> 0`、廃番化（同時に `plu_target=0`） | 状態を変えない（`reserved` / `active` のまま）。Diff / Full に出さない。レジには前の登録が残る | — |
| `plu_target: 0 -> 1`（廃番解除の後の再対象化を含む） | その JAN の枠があれば次の prepare で同じ枠を使う | `plu_dirty=1` |
| `plu_target=1` の商品の `jan_code` の変更 | 前の JAN の枠は状態を変えず残す（対象の商品を持たない JAN の枠になる）。新しい JAN は次の prepare で枠を予約する | `plu_dirty=1` |

前の JAN の枠のバーコードでレジが売った分は、次の Z004 にその JAN の行で出る。その JAN を持つ商品が無ければ、取込みは `unmatched_product` の行エラーで知らせる（[32](32-biz-csv-import-service.md) §15.3 手順 4 b〈Stage 2〉。黙って誤った商品へ付けない）。同じ商品へたどる経路は前の JAN の別名（D-122、proposed・未実装。[30](30-biz-product-service.md) BIZ-01-D9、[master-tables](../db-design/master-tables.md) §1a）: JAN を直すときに前の番号の現物が「ある」と答えた商品だけ、取込みがその JAN の行を別名で同じ商品へ入れる。別名は枠と独立で、本節の枠の扱い（状態を変えず残し、書き出さない）・§16.3 の照合・prepare・confirm は別名を読まず書かない。前の JAN の枠の値段はレジに残った前の値段のまま（売るのをやめた商品と同じ受け入れ済みの限界、D-117 の Guarantee range）。廃番解除は `plu_target` を自動復帰させない。

撤去の対象（D-117 (1)。runtime の lane が code と一緒にこの signature を消す。それまでは設計照合 test〈`src-tauri/tests/design_compliance_test.rs`〉が code の pub fn と突き合わせるので signature だけを残す）:

```rust
pub fn release_plu_slot_for_jan(conn: &DbConnection, jan_code: &str) -> Result<(), BizError>
```

#### 16.6.1 list_plu_dirty

```rust
fn list_plu_dirty(conn: &DbConnection) -> Result<Vec<Product>, BizError>
```

`plu_target=1 AND plu_dirty=1` の商品を product_code 順で返す read-only query。UI-00 の `PluNotificationBar` と UI-08 の既存 preview 契約を維持し、slot 予約・snapshot・集計状態を変更しない。未反映の商品は溜まり、店主は PLU を変えるとき（新商品・値段の変更）に CV17 をまとめて開いて 1 回で反映する（D-117 (6)）。

### 16.7 IO / repository への依存

| 依存 | 用途 |
|---|---|
| `io::z004_parser::parse_plu_register_snapshot` | layout A 全スロット占有の読取り |
| `io::z004_parser` の精算の識別メタ | 照合に使う Z004 の選択（§16.3 手順 1） |
| `db::plu_slot_repo` | snapshot 照合、最小空き予約、sticky 取得、confirm の状態遷移 |
| `db::product_repo` | PLU target 抽出、同一 JAN 群、dirty 更新、slot JOIN |
| `io::plu_formatter::generate_plu_tsv` | memory No. 付き product 行の 11 field 出力 |
| `db::operation_log_repo` | snapshot / prepare / confirm の要約記録 |

### 16.8 Error / Recovery

- 保存キャンセル・保存失敗: 同じ mode を再 prepare すると予約済み memory No. を再利用する。
- CV17 取込み失敗: Diff か Full を書き出し直す（前に保存した file は使わない。保存の後の SD の読取りの照合で枠が変わりうるため〈予約中の枠の `reservation_dropped`、対象の商品を持たない JAN の枠の external 化。§16.3〉。書き出し直しは sticky で同じ memory No. を使い、照合で枠が変わっていれば新しい枠を取る。`plu_dirty` は confirm まで残る〈D-027〉）。
- DB の restore の後: 2 つの key（`plu_register_snapshot_at`・`plu_register_snapshot_settlement_no`）を消し、既存の gate（§16.3 の `register_snapshot_required`）で次の SD の読取りまで書出しを止める。runtime の lane が 71 §71.7 `restore_backup` に足す（本 lane は 71 を触らない。D-117 の Guarantee range と backlog の runtime の項目の写し）。
- snapshot conflict: active slot を勝手に external / free へ変えず、商品を dirty にする（次の書出しでアプリのコードを書き直す）。
- 占有の照合の失敗: BIZ-08 は IO-09-D4 の形の検査で外れた Z004 と読めない Z004 を除いて渡す（37 §37.9 手順 10）。行数と中身の検査は IO-02 `parse_plu_register_snapshot` が行い、`ImportError` なら照合しない。どちらも前回の照合を保ち、CMD-12 の scan は失敗にしない（§16.3）。
- `NoFreeSlot`: 該当 JAN だけを excluded とし、既存 slot と他の生成行を保持する。

### 16.9 対応不変条件

- 同一 JAN は同一 memory No. を sticky に使い、prepare 順序や画面 filter で番号を変えない。
- app は Z004 snapshot なしに空き slot を仮定しない。
- app は枠を消さず、アプリ管理の枠（`reserved` / `active`）の JAN をアプリの操作で別の JAN に変えない（使い回さない）。アプリ管理の枠が別のコードの枠になるのは、照合でレジ側に別のコードを観測したとき（予約中の枠の `reservation_dropped`、対象の商品を持たない JAN の枠の external 化）だけである。`external` と `free` の観測の更新は §16.3 の照合の表に従う。
- `external` / `free` と対象の商品を持たない JAN の slot は出力しない。要修正中の既存 slot は維持して出力しない。
- Diff / Full とも CV17 投入可能。Full は対象の JAN の app 管理 slot 全体、Diff は未反映の対象。
- slot 状態は売上、在庫、会計集計へ影響させない（SPEC-PLS-D8）。
