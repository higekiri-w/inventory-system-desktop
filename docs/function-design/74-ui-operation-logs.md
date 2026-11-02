> **親文書**: [FUNCTION_DESIGN.md](../FUNCTION_DESIGN.md)
> **入力ドキュメント**: ARCHITECTURE.md（UI-11c / QR-06 / REQ-902）、SCREEN_DESIGN.md（操作ログ画面 / 入出庫履歴・在庫変動追跡）、43-cmd-settings-log.md（CMD-11 list_logs）、20-io-product-repo.md §2.8（system_repo）、65-inventory-record-traceability.md（TRACE-D3、完成形 traceability contract）、59-ui-shared-patterns.md（EmptyState 等共有部品）、52-ui-shared-layout.md（navigation / route 一覧）
> **Plan Packet**: [archived plan](../archive/plans/2026-07-11-ui11c-operation-logs.md)（Design Phase 出典）

## 74. UI-11c: 操作ログ画面

### 74.1 概要

- **対応 REQ**: REQ-902（ログ管理: 操作ログ記録/一覧/自動削除、MNT-02 が担当）+ TRACE-D3（操作ログは業務記録の代替にしない）。画面タスク自体の ID は ARCHITECTURE.md タスク表の `QR-06`（UI-11c）。
- **route**: `/settings/logs`（`52-ui-shared-layout.md` §52.3 で確定済み、本書で route 実装契約を固定する）
- **呼び出す CMD**:
  - `listLogs(query: LogQuery)` — 既存 CMD-11。本書で `start_date` / `end_date` を追加する（§74.4、43-cmd-settings-log.md 側で正式化）
  - `listLogOperationTypes()` — **新規 CMD**（§74.5）。保持中の operation_logs 全体から distinct な operation_type を返す。現在ページ・現在の filter 済み結果からの生成を禁止するため新設する。
- **主動線**:
  1. サイドバー「システム管理」→「操作ログ」で `/settings/logs` を開く
  2. 既定で直近30暦日・全種別のログを新しい順に確認する
  3. 期間・種別で絞り込む
  4. 各行の「詳細を表示」ボタンで詳細（detail_json の要約 + 折りたたみ raw JSON）を展開する
  5. 関連業務記録がある行はその詳細へ遷移する
  6. ページングで過去ログを辿る
- **初回実装の非対象**（§74.16 で詳細化）: CSV 出力、操作ログの保持設定変更・削除、診断ログファイル/ディレクトリ導線（MNT-04）、業務記録の訂正・取消。

**関数要求**: 操作ログを URL search state 付きで期間・種別絞り込み表示し、各行の detail_json を安全に要約・展開し、明示的な contract がある場合だけ関連業務記録へ遷移できるようにする。操作ログは監査・保守ログであり、業務記録・在庫変動履歴の代替にしない（TRACE-D3）。

**シグネチャ**:

```ts
export function OperationLogsPage(props: {
  search: OperationLogsSearch;
  onSearchChange: (updater: (prev: OperationLogsSearch) => OperationLogsSearch) => void;
}): JSX.Element;

export function useOperationLogs(args: {
  search: NormalizedOperationLogsSearch;
}): {
  logsQuery: UseQueryResult<PaginatedResult<OperationLog>>;
  typesQuery: UseQueryResult<string[]>;
};
```

**処理ステップ**: §74.3〜§74.13 で URL state、期間 filter、registry、table/detail、pagination、empty/error/retry、a11y を個別に定義する。

**エラーハンドリング**: §74.11 を参照。

---

### 74.2 Design Decisions

| ID | 決定 | 理由 / 棄却案 |
|---|---|---|
| UI-11c-D1 | URL search state は `start_date` / `end_date` / `operation_type` / `page` の4キーに限定し、いずれかの変更で `page=1` に戻す。初期の両日付未指定だけは `start_date = today-29`, `end_date = today`（JST暦日）を既定生成する。日付inputを明示的にclearした状態はURLの空文字で保持し、正規化後はCMDへ`null`として渡す。 | Owner Decision #2/#5 の固定。初期未初期化と利用者が両日付をclearした状態を混同すると片側指定が失われるため、空文字はURL/UI専用sentinelとする。`InventoryRecordsPage` / `StockMovementsPage` の既存 URL state 契約を再利用する（§74.17 Adjacent Pattern Audit）。 |
| UI-11c-D2 | 期間は JST 暦日の `start` inclusive・`end` は翌日00:00:00 exclusive の predicate にする。 | Owner Decision #2 の明示固定（JST inclusive/exclusive）。保存の時刻の終端に時刻の文字列を足して比べる形は秒精度に依存し、将来 `created_at` にミリ秒等が付いた場合に境界が曖昧になるリスクがある。操作ログは監査用途のため境界の厳密さを優先する。在庫変動の履歴は D-118 の b2 で業務日の両端を含む比べに変わった（[44](44-cmd-inventory.md) の `list_movements`）。入出庫の記録の画面は本設計で変更しない（scenario が異なるため据え置き、§74.17）。 |
| UI-11c-D3 | `LogQuery` に `start_date` / `end_date`（`Option<String>`, `YYYY-MM-DD`）を追加する。両方省略時は現行動作を完全維持する。`items` と `total_count` は同一 predicate を使う。 | Owner Decision #2。行 query と件数 query が別 predicate になると pagination 契約が壊れるため、同一 WHERE 句を共有する実装を必須にする（43-cmd-settings-log.md / 20-io-product-repo.md 側で明文化）。 |
| UI-11c-D4 | operation_type の候補は現在ページ/現在の filter 済み結果から生成しない。保持中ログ全体の distinct operation_type を返す**新規 CMD `list_log_operation_types` + IO `find_distinct_operation_types`** を新設する。日本語ラベルとカテゴリ分類は frontend 所有の静的 registry（`OPERATION_TYPE_LABELS`）が持ち、backend が返す値のうち registry 未収載の値は raw 表示にする。 | Owner Decision #3 + Plan Packet 明示要求（「新CMD/IO契約の設計を含む」）。frontend 静的リストのみだと DB 実態との drift を検出できない。backend 側が日本語ラベルを持つ案は、ラベルが表示専用の UI 関心事であり CMD 層を厚くする（棄却）。 |
| UI-11c-D5 | Table は1行1ログとし、各行の明示的なnative button「詳細を表示」で detail をその場（同じ行の直下）に展開する。展開中の可視文言とaccessible nameは「詳細を閉じる」に変わる。Enter / Spaceはnative button契約で動作する。行全体clickは追加せず、同時に展開できる行は常に1件（新しい行を開くと前の展開行は閉じる）。 | キーボード・アクセシビリティ上の操作対象を明確にし、行内の関連記録リンクとのclick競合と誤展開を避ける。複数同時展開も棄却（ページが際限なく伸び、スクロール位置を見失うリスクが高い）。 |
| UI-11c-D6 | detail_json は既知 field の日本語要約を優先表示し、生 JSON は既定で折りたたみの「技術情報」に隠す。null/空/不正 JSON/巨大 payload は要約なしで安全に案内する。HTML として解釈せず常に text として描画する。 | Owner Decision #4。既存 detail_json は operation_type ごとに自由形式（`db-design/tracking-system-tables.md` §18 設計意図）であり、per-type スキーマを作る工数は viewing MVP の scope を超える。既知 key 名の共通マップ + 未知 key の raw 表示に留める。 |
| UI-11c-D7 | 関連業務記録リンクは、detail_json が `record_type`（`receiving_record` \| `return_record` \| `manual_sale` \| `disposal_record` のいずれか）とpositive safe integerの `record_id` を両方含む場合だけ表示する。それ以外（欠落・zero・negative・fractional・string・unsafe integer・未知 `record_type`）はリンクを出さず、summary 表示のみ継続する。入庫・返品交換・手動販売・廃棄の作成 producer は、許可リストに対応する `record_type` と当該業務記録 PK の `record_id` を detail_json に格納する（§74.9）。 | Missing UI item 7 は「任意の JSON key からの heuristic 推測」と文字列からの数値coercionを明示的に禁止する。既存 4 record_type は `InventoryRecordsSearch.recordType` / `65-inventory-record-traceability.md` §65.3 の実装済み detail route と一致させる。`csv_import` / `stocktake` は 2026-09-01 の producer 実効化 R3 では据置とし、初期 allow-list からの除外を維持する。それぞれの producer 側採用と registry 追加は別 follow-up で行う。 |
| UI-11c-D8 | pagination は既定 50 + `Select`（50 / 100 / 200）。URL search state は UI-11c-D1 の 4 キー限定を維持し perPage はローカル state（L3-D3）。範囲外 page（`items` 空 かつ `total_count > 0` かつ `page > 1`）は「このページには表示するログがありません」+「先頭ページに戻る」導線を出す。 | 表示件数を選べる一方、URL search schema は増やさない。範囲外 page 回復は既存2画面が持たない新規契約だが、Missing UI item 8 が明示要求するため追加する。 |
| UI-11c-D9 | 空状態は「ログそのものが0件」と「filter に該当するログが0件」を区別する。既定 filter（today-29..today、operation_type 未指定）と一致した状態で0件なら前者、それ以外の filter が適用された状態で0件なら後者の文言にする。エラー時は destructive Alert + 「再試行」ボタンを出し、再試行は現在の filter を保持したまま同一 query を再実行する。 | Missing UI item 9。既存2画面は単一 EmptyState 文言・retry ボタンなしだが、操作ログはトラブル対応で使う画面のため取得失敗時の再試行を明示的に用意する（`DailySalesPage` / `MonthlySalesPage` / `ThresholdSettingsPage` / `StocktakePage` の再試行 Button パターンを再利用、§74.17）。「全体0件かどうか」を判定する専用 CMD は追加しない（既定 filter 自体が30日範囲を持つため真の「全体0件」判定には無期限 count が必要になり、viewing MVP の scope を超えるため見送り。既定 filter との一致判定で近似する）。 |
| UI-11c-D10 | `queryKey` は `queryKeys.operationLogs.list(normalizedSearch)`、`staleTime: 0` / `gcTime: 5min` とし、自動ポーリングはしない。バックグラウンドの保持日数の cleanup は起動時1回のみのため、セッション中の total_count 変化は次回 fetch（filter 変更・page 変更・再訪・再試行）で自然に反映される。 | 事故調査という利用シーンでは表示時点の最新性を優先する（`UI-10 棚卸し進行中` の `staleTime: 0` と同じ判断）。ポーリングは新規複雑さの割に運用上の要求がない（単一 operator、トラブル発生後に開く画面）。 |
| UI-11c-D11 | operation_type filter は `<select>` によるドロップダウン選択のみとし、自由入力・IME 対応の検索欄は持たない。 | Owner Decision #3 は個別 exact value の選択を要求しており、`InventoryRecordsPage` の商品名部分一致検索のような自由入力欄は不要。IME `isComposing` 対応は本画面に適用対象がない（§74.13 で明示）。 |
| UI-11c-D12 | REQ-902 を操作ログの record/list/自動削除の canonical ID として確定する。既存 `settings_cmd.rs` の `test_list_logs_req905_pagination` / `test_list_logs_req905_filter` / `test_list_logs_req905_invalid_page_to_cmderror` は REQ-905（設定CRUD/エラー変換）ではなく REQ-902 に是正する（テスト名・コメントの是正は本 Design Phase では実施せず、次の実装 PR のタスクとして明記する）。 | `docs/spec/requirements.md` は REQ-902=「ログ管理（操作ログ記録/一覧/自動削除）」/ REQ-905=「設定管理（設定CRUD/エラー変換）」と定義し、`system_repo.rs` の IO 層テストは既に log 系関数を REQ-902、settings 系関数を REQ-905 で正しく分離している（`rg` 実測）。`65-inventory-record-traceability.md` §65.11 も「REQ-902 / TRACE-D3」を canonical として使用済み。CMD 層 3 テストだけが REQ-905（CMD-11 全体のタスク表マッピング）を機械的に継承しており、`list_logs` の呼び出し・エラー変換テストという中身は REQ-902 の対象。「REQ-905 のままエイリアスとして残す」案は、REQ-905 の定義（設定CRUD）と `list_logs`（ログ一覧）の実体が一致しないため棄却。 |
| UI-11c-D13 | `SCREEN_DESIGN.md` の QR-06 行にあった旧 CSV export/archive 記述を、閲覧 MVP + 保持日数の cleanup（archive 不要。当時は 365 日。今の既定は [72](72-mnt-log-manager.md) MNT-02-D1）に同期する。CSV 出力と MNT-04 診断ログ導線は別 task として明記する。 | Owner Decision #1 + Missing UI item 13。`72-mnt-log-manager.md` の実装済み `cleanup_old_logs` は物理削除のみで archive を作らない（DB設計上の事実）。 |
| UI-11c-D14 | `integrity_fix` の detail_json（adjustments の product_code / old_stock / new_stock / adjustment）は在庫整合性補正の**唯一の監査痕跡**（補正は inventory_movements に行を残さない — BIZ-07-D2/D3、[D-051](../decision-log.md)）。詳細表示は既知 key 要約（UI-11c-D6）に加え、adjustments を「商品コード / 旧在庫 → 新在庫 / 差分」の operator-readable な一覧として表示し、生 JSON は折りたたみ「技術情報」に残す。adjustment に `stock_unit` の key がある記録の長さの商品は m で出し（`旧在庫 27.5 m → 新在庫 27 m`・`差分 -0.5 m`）、key の無い記録済みの log は整数のまま（[共通規則](10-common-rules.md) SPEC-UNIT-D3 の「表示する所」の表の 12 行目）。D-118 の (a) の後に書く adjustment は `product_code`・`stock_unit`・`before`・`after`（それぞれ [36](36-biz-integrity-check.md) の `StockCopy` の 4 項目）で、「{商品コード} / {before} → {after}」と出す。各側は在庫数（単位どおり）と、その側の `stock_recheck` が true なら後ろに「（要再確認 {low}〜{high}）」（例: `5（要再確認 4〜6） → 5（要再確認 3〜7）`。差分は出さない）。記録済みの形（`old_stock`・`new_stock`・`adjustment`）は書き換えず今の表示で読み続け、1 つの log の中で要素ごとにどちらかの形で読む。どちらの形にも合わない要素（印と範囲が対にならない `StockCopy` を含む）が 1 つでもあれば一覧を出さず、生 JSON の要約と「技術情報」へ回す。移行の行（`trigger: migration`）の要約は summary の文のまま。 | 非IT operator が唯一の監査痕跡を読めることは、movement を残さない直接更新意味論の説明可能性の前提。rejected: 生 JSON のみの現状維持 = operator が補正内容を読めない。保持日数の cleanup（既定は [72](72-mnt-log-manager.md) MNT-02-D1、D-136。D-051 の retention を置き換えた）までが閲覧可能期間である。実装は follow-up PR（`OperationLogsPage.tsx` + test）。 |
| UI-11c-D15 | 逆転範囲で参照する「直前に commit 済みの valid normalized search」は state に保持し、valid render が commit した後の effect でだけ更新する。valid render 自身は現在の normalized search を query key / CMD引数 / pagination に直ちに使い、invalid render のときだけ commit 済み snapshot を使う。 | render 中の ref 更新は破棄された render の search を共有 ref に残す。state/effect 方式なら未commit renderは snapshot を更新しない。valid render で state snapshot の反映を待つ案は通常のfilter変更に1 renderの遅延を生むため棄却し、valid側は現在値を直接使う。effect commitからsnapshot反映までにinvalid更新が割り込む極小窓では、前回snapshotを一時表示した後に最新commit済みsnapshotへ同期し得る。この差を許容するか、`useLayoutEffect` でpaint前同期するかは Plan Gate のowner裁定対象とする。 |
| UI-11c-D16 | 「関連記録を見る」は、現在の `/settings/logs` URL（期間 `start_date` / `end_date`、`operation_type`、`page` の search state を含む）を `returnTo` として業務記録詳細へ送る。詳細の「前の画面へ戻る」で同じ検索 state を復元する（DSR-18）。 | 操作ログの調査中に関連記録を確認しても、期間・種別・page を組み直さず同じ調査位置へ戻れるようにする。link は UI-11c-D5 の展開行内の明示的な操作のままとし、行全体 click や展開 toggle を追加しない。 |

---

### 74.3 URL Search State

```ts
export interface OperationLogsSearch {
  start_date?: string; // YYYY-MM-DD、または明示clear用の空文字
  end_date?: string;    // YYYY-MM-DD、または明示clear用の空文字
  operation_type?: string;
  page?: number;
}

export interface NormalizedOperationLogsSearch {
  start_date?: string;
  end_date?: string;
  operation_type?: string;
  page: number;
}
```

- Route (`src/routes/settings/logs.tsx`) の `validateSearch` は、厳密な`YYYY-MM-DD`または明示clear用の空文字を `start_date` / `end_date` に、`z.string().optional().catch(undefined)` を `operation_type` に、`z.coerce.number().int().positive().optional().catch(undefined)` を `page` に適用する。
- 空文字はUI/URLでだけ保持し、CMDへは渡さない。不正な非空日付値は `catch(undefined)` でその片側を破棄する。両側が`undefined`になれば初期既定へfallbackし、もう片側が有効なら有効な片側指定を保持する。`operation_type` は任意の文字列を許容する（未知値は raw fallback 表示、§74.5）。
- 正規化（`normalizeOperationLogsSearch`）:
  - `start_date` / `end_date` が**ともに未指定** → JST 今日から29日前（`today-29`）/ JST 今日
  - 片側だけ有効値ならもう片側は未指定のままにする（CMD payloadは`null`）。
  - 明示clearの空文字は未指定へ正規化する。ただし空文字がsearch stateに残るため、両側clearと初期未初期化は区別できる。
  - `page` 未指定または `< 1` → `1`
  - `start_date > end_date`（正規化後の両方が揃っている場合）は fallback せず、**バリデーションエラー状態**として扱う（§74.4）。URL 自体は書き換えず、直前のvalid query結果を表示保持する。
- `start_date` / `end_date` / `operation_type` のいずれかを変更したら `page=1` に戻す（page 自体の変更ではリセットしない）。
- reload / back-forward: TanStack Router の search state は URL に一致するため、reload・ブラウザ相当の戻る/進むいずれも同じ query を再実行する。展開行の開閉状態のようなローカル state は URL に持たないため保持しない（§74.6）。

---

### 74.4 期間フィルタ設計

- **書式**: `YYYY-MM-DD`（`records.tsx` / `StockMovementsPage` と同一）。
- **JST 暦日 predicate**:
  - `start_date` が指定されている場合: `created_at >= '{start_date}T00:00:00'`
  - `end_date` が指定されている場合: `created_at < '{end_date + 1日}T00:00:00'`
  - 両方省略時は現行動作を完全維持（フィルタなし）。
  - `created_at` は既存 `db::init` 経由で `chrono::Local::now()` 由来の ISO 8601 文字列（JST ローカル時刻、サーバー/クライアント分離のないデスクトップアプリのため TZ 変換は不要。§74.4.1）。
- **逆転範囲**: 正規化後に `start_date > end_date` が判明した場合、CMD は防御境界として validation error を返す（§74.4.2）。ただしUIはURL直接改変を含めて送信前に検出し、入力欄で即時inline messageを出して`listLogs`を呼ばない。直前のvalid query結果（table、pagination、展開行）は保持する。
- **片側指定**: `start_date` のみ・`end_date` のみのどちらも許可する。
- **最大範囲**: 設けない。既存の `list_movements` / `InventoryRecordsPage` も範囲上限を持たず、pagination が総件数を吸収する。
- **row/count predicate 同一性**: `list_operation_logs` はデータ取得 SQL と `COUNT(*)` SQL に同じ `WHERE` 句（type + 期間）を適用する（既存 type フィルタと同じ実装パターンを踏襲、20-io-product-repo.md §74.4.3 参照）。
- **clock/test seam**: 「今日」は既存 `72-mnt-log-manager.md` の `cleanup_old_logs` と同様 `chrono::Local::now().date_naive()` を単一の取得点にする。UI 側の既定値計算（`today-29`）は Rust 側で計算せず、frontend の正規化関数が `new Date()` を1箇所で呼ぶ（テスト時は呼び出し元から日付を注入できるよう関数を純関数化し、`now` 引数を optional で受け取れるようにする）。

#### 74.4.1 タイムゾーンに関する前提

本アプリは単一店舗のローカル Windows デスクトップアプリであり、サーバー側 TZ 変換は発生しない。`created_at` は `chrono::Local::now()`（実行環境のローカル時刻、想定運用は JST）で記録されるため、「JST 暦日」は「アプリ実行環境のローカル暦日」と同義として扱う。海外展開・複数 TZ 運用は `UI_TECH_STACK.md` §3.5 で対象外と明記済みのため、明示的な TZ 変換ロジックは導入しない。

#### 74.4.2 バリデーションエラー contract

- CMD-11 `list_logs` は `start_date` / `end_date` のいずれかがASCII strict `YYYY-MM-DD`形式でない、または実在しない暦日の場合、`CmdError { kind: "validation", message: "開始日・終了日はYYYY-MM-DD形式で入力してください" }` を返す。
- `start_date > end_date` の場合、`CmdError { kind: "validation", message: "開始日は終了日と同じ日か、それより前の日付にしてください" }` を返す（`start_date == end_date` は1日指定として有効）。
- 日付形式検証と逆転範囲検証は CMD 層（薄いラッパー内の追加チェック）で行う。既存の `page`/`per_page` 検証は引き続き IO 層（`system_repo`）側に残り、この非対称（日付は CMD 層・page/per_page は IO 層）は本設計で解消しない。日付形式は利用者入力起因の validation であるため CMD 層で弾く判断とする。詳細は 43-cmd-settings-log.md。

#### 74.4.3 既存パターンとの差分

UI-11c は JST 暦日の inclusive/exclusive 境界と逆転範囲エラーを新規導入する。在庫変動の履歴（[44](44-cmd-inventory.md) の `list_movements`）は D-118 の b2 で、保存の時刻でなく業務日（`YYYY-MM-DD`）の両端を含む比べに変わった。入出庫の記録の画面（`InventoryRecordsPage`）は日付の列を `<= date_to` で比べ、逆転範囲チェックを持たない。このロジックは本設計では変更しない（scenario が異なる: 操作ログは監査境界の厳密性を要求される一方、records は業務日付で比べ運用上問題が出ていない）。将来 records 側で境界問題が実運用で顕在化した場合に横展開を検討する（decision-log D-037 参照）。

---

### 74.5 Canonical operation_type Registry

#### 74.5.1 Backend contract（新規）

- **IO**: `find_distinct_operation_types(conn: &DbConnection) -> Result<Vec<String>, DbError>` — `SELECT DISTINCT operation_type FROM operation_logs ORDER BY operation_type ASC`。0件でも `Ok(vec![])`。
- **CMD**: `list_log_operation_types(state: State<AppState>) -> Result<Vec<String>, CmdError>` — 薄いラッパー。フィルタ・ページングを持たない。
- この query は期間・種別 filter を適用しない（保持中ログ全体が対象）。したがって画面の期間 filter を変えても候補一覧は変化しない。
- PR #164で`#[specta::specta]`化、`generate_handler!`/`collect_commands!`登録、bindings再生成まで実装済み。CMDはvalidationとIO呼出しだけの薄い境界を維持する。

#### 74.5.2 Frontend ラベル registry（既存コード実測ベース）

`src/features/operation-logs/operation-type-labels.ts`（新規）に、現行コードベースで実際に使われている operation_type 値（`rg 'operation_type: "[a-z_]+"' src-tauri/src` で実測、`test_op` を除く29種）を初期 entries とする。

| カテゴリ | operation_type | 日本語ラベル |
|---|---|---|
| 商品管理 | `product_create` | 商品登録 |
| 商品管理 | `product_update` | 商品修正 |
| 商品管理 | `product_discontinue` | 廃番切替 |
| 商品管理 | `product_import` | 商品一括インポート |
| 商品管理 | `product_price_revise` | 一括価格改定 |
| 商品管理 | `product_bulk_plu_target` | PLU 対象一括切替 |
| 取引先管理 | `supplier_rename` | 取引先の改名 |
| 取引先管理 | `supplier_merge` | 取引先の統合 |
| 入出庫 | `receiving_create` | 入庫記録 |
| 入出庫 | `return_create` | 返品・交換記録 |
| 入出庫 | `manual_sale_create` | 手動販売出庫記録 |
| 入出庫 | `disposal_create` | 廃棄・破損記録 |
| 売上データ取込み | `csv_import` | 売上データ取込み |
| 売上データ取込み | `csv_import_failed` | 売上データ取込み失敗 |
| 売上データ取込み | `csv_import_parse_failed` | 売上データ解析失敗 |
| 売上データ取込み | `csv_rollback` | 売上データ取込み取消 |
| 売上データ取込み | `daily_report_import` | 日報取込み |
| 売上データ取込み | `daily_report_import_failed` | 日報取込み失敗 |
| 売上データ取込み | `daily_report_parse_failed` | 日報解析失敗 |
| 売上データ取込み | `daily_report_rollback` | 日報取込み取消 |
| 棚卸し | `stocktake_start` | 棚卸し開始 |
| 棚卸し | `stocktake_complete` | 棚卸し確定 |
| PLU書出し | `plu_export` | PLU書出し |
| PLU書出し | `plu_register_snapshot_import` | PLU 登録状態の取込み |
| 整合性検証 | `integrity_check` | 整合性チェック実行 |
| 整合性検証 | `integrity_fix` | 整合性補正 |
| システム管理 | `backup_create` | バックアップ作成 |
| システム管理 | `backup_restore` | バックアップ復元 |
| システム管理 | `log_cleanup` | 操作ログ自動削除 |

- **registry ownership**: frontend（`src/features/operation-logs/operation-type-labels.ts`）が単一の SSOT を持つ。backend は distinct 値の集合のみ返す。
- **初期 entries/順序**: 上表の29件、カテゴリ順（商品管理 → 取引先管理 → 入出庫 → 売上データ取込み → 棚卸し → PLU書出し → 整合性検証 → システム管理）→ カテゴリ内は表の記載順を初期表示順とする。
- **拡張ルール**: 新しい operation_type を biz/mnt 層で導入する実装 PR は、同じ PR で `operation-type-labels.ts` にカテゴリ + 日本語ラベルを追加する。追加を怠っても機能は壊れない（raw fallback）が、operator 可読性が下がるためレビュー観点に加える。
- **未知値 fallback**: registry 未収載の値は「その他（`{raw_value}`）」の形式でカテゴリ「その他」にグルーピングして表示する。フィルタ選択肢としても `{raw_value}` のまま選べる。
- **フィルタ選択肢のソース**: `typesQuery`（`list_log_operation_types` の結果）と `operation-type-labels.ts` を突き合わせ、`typesQuery` に実在する値だけを選択肢にする（未来のいつか使われるかもしれない registry entry を実在しないのに選べる状態にしない）。
- **`integrity_fix` の特記**: この entry の detail_json は在庫整合性補正の唯一の監査痕跡（補正は inventory_movements に行を残さない — D-051）。表示期待は UI-11c-D14 参照。

---

### 74.6 Data Flow

```
Route search: start_date, end_date, operation_type, page
  ↓
OperationLogsPage
  ├ typesQuery: commands.listLogOperationTypes()
  └ logsQuery: commands.listLogs({
       page,
       per_page: perPage,
       operation_type: operation_type ?? null,
       start_date: start_date ?? null,
       end_date: end_date ?? null,
     })
       ↓
OperationLogFilters + OperationLogTable（展開行1件） + Pagination
```

- `typesQuery` と `logsQuery` は独立 query として部分障害を許容する（`typesQuery` 失敗時は operation_type filter を「すべて」のみに縮退表示し、一覧本体は表示継続する。§74.11）。
- 逆転範囲のdraft URL stateは`logsQuery`のqueryKeyに使わず、直前にcommit済みのvalid normalized search stateをeffective queryとして使う。valid renderでは現在のnormalized searchを即時に使い、commit後のeffectでsnapshotだけを同期する。よってinvalid draftでは新しいCMD呼出しをせず、既存table/pagination/展開行を保持する。validへ戻った時点で新しいeffective queryへ切り替える（UI-11c-D15）。
- 行展開状態（どの `id` が展開中か）は URL に持たない、画面ローカル `useState<number | null>`。**validな**filter/page変更時はリセットする。逆転範囲のdraft変更だけではリセットしない。

---

### 74.7 Table 契約

列（左から）:

1. 日時（`created_at`、`formatDateTime` で `T` を半角スペースに置換した `YYYY-MM-DD HH:mm:ss` 表示。共有 helper `src/lib/date-time.ts` の `formatDateTime` を再利用する）
2. 種別（`operation-type-labels.ts` の日本語ラベル。未知値は「その他（raw値）」）
3. 概要（`summary`。1行表示、`min-w-0` + `truncate` で長文をトランケートし、`title` 属性でフルテキストを提供する。詳細行を展開すれば `summary` 自体も折り返し全文表示される）
4. 詳細（展開トグルボタン。`aria-expanded` + `aria-controls` で対応する詳細行 `id` を指す）

表示規則:

- 行全体をクリック可能にせず、詳細列のnative `<button>`だけを展開トリガーにする。閉状態は可視text / accessible nameとも「詳細を表示」、開状態はともに「詳細を閉じる」とする。誤操作防止のため `summary` 列テキスト選択（コピー）操作を妨げない。
- ボタンへフォーカスした状態のEnter / Spaceはnative buttonのclick activationに委ね、独自`onKeyDown`で二重実装しない。
- 展開した詳細内の「関連記録を見る」リンクを押しても展開toggleは発生しない。
- 展開行は該当行の直下に `colSpan` フルwidth の1行として挿入する（テーブル外へ切り離さない）。
- 同時に展開できるのは1行のみ（UI-11c-D5）。別の行を展開すると前の展開行は自動的に閉じる。
- 展開中に filter/page を変更した場合は展開状態をリセットする（§74.6）。

---

### 74.8 detail_json 契約

- **既知 field 要約**: `detail_json` が有効な JSON オブジェクトとしてパースできた場合、トップレベル key を共有の「既知 key 日本語ラベル辞書」（例: `file_name`→ファイル名, `size_bytes`→サイズ（バイト）, `count`→件数, `product_code`→商品コード, `record_id`→関連記録ID（§74.9）等、実装時に既存 detail_json 生成箇所を棚卸しして拡張可能な辞書として実装）で変換し、`ラベル: 値` の一覧として先頭表示する。辞書未収載の key は key 文字列そのものをラベル代わりに使う（raw key 表示、非表示にはしない）。商品修正の key（[30](30-biz-product-service.md) BIZ-01-D7。単位の語の直しの lane〈D-119〉で足す `stock_unit` を含む）は辞書に入れず、raw key と `{"old":…,"new":…}` の JSON のまま出す（2026-10-17、単位の語の直しの lane の起票で決めた）。理由: 商品修正の他の key（`name`・`selling_price` 等）も辞書に無く、`stock_unit` だけを訳すと同じ記録の中で読み方が分かれる。訳すなら商品修正専用の表示（名前の解決・「旧 → 新」の整形・code の語への変換）として key をまとめて扱う（BIZ-01-D7 の Revisit）。
- **値の描画**: 値が文字列/数値/真偽値ならそのまま表示。値がネストしたオブジェクト/配列の場合はその value だけ JSON 文字列化して等幅フォントで表示する（再帰的な既知 field 展開はしない。スコープを viewing MVP に収める）。
- **常に text-only**: 値・key のいずれも `dangerouslySetInnerHTML` 等の HTML 解釈を行わない。プレーンテキストノードとしてのみ描画する。
- **null / 空文字**: 「詳細情報はありません」を表示し、折りたたみの技術情報自体を出さない。
- **不正 JSON（パース失敗）**: 「詳細情報を解析できませんでした」を表示し、折りたたみの技術情報には生文字列をそのまま表示する（データを握りつぶさず、調査目的の生ログとして保持する）。
- **既知 field 要約 + 折りたたみ raw JSON**: 要約の下に「技術情報（JSON）」という折りたたみ（既定は閉じた状態）を置き、開くと整形済み（`JSON.stringify(parsed, null, 2)` 相当）の raw JSON を `<pre>` + 等幅フォントで表示する。パース失敗時は raw 文字列をそのまま `<pre>` に入れる。
- **巨大 payload**: `detail_json` の文字列長が **10,000 文字**を超える場合、既知 field 要約は先頭 20 key までに制限し「他 N 件のフィールドは技術情報でご確認ください」を追記する。折りたたみ raw JSON も **50,000 文字**を超える場合は先頭 50,000 文字 + 「以降は長すぎるため省略しました」を表示し、DOM に全文を流し込まない（大量 DOM によるレンダリング遅延防止）。
- **コピー導線**: 折りたたみを開いた状態でのみ「コピー」ボタン（`navigator.clipboard.writeText`）を raw JSON 表示の右上に出す。要約表示部分にはコピーボタンを置かない（複製すべきは技術情報であり要約は表示用途のため）。
- **synthetic-test policy**: 自動テストは常に synthetic な detail_json（実店舗データを含まない）を使う。null・空文字・不正 JSON・巨大 payload（文字列長超過）・既知 key・未知 key の組み合わせを個別ケースとしてテストする（Test Design Matrix 参照）。

---

### 74.9 関連業務記録リンク契約（UI-11c-D7 / UI-11c-D16）

- `detail_json` を JSON オブジェクトとしてパースできた場合のみ評価する。
- 次の**両方**を満たす場合だけ「関連記録を見る」ボタンを表示する:
  1. `record_type` が次の許可リストのいずれかである: `receiving_record` | `return_record` | `manual_sale` | `disposal_record`
  2. `record_id` がJavaScriptのpositive safe integerである（`typeof number`、`Number.isSafeInteger(record_id)`、`record_id > 0`）。numeric stringへのcoercionはしない
- 許可リストは `InventoryRecordsSearch.recordType`（`src/features/inventory-records/types.ts`）と一致させる。`csv_import` と `stocktake` の詳細 route は実装済みだが、2026-09-01 の producer 実効化 R3 では両種を据置とし、許可リストからの除外を維持する。csv_import / stocktake producer への `record_type` 採用と許可リスト追加は、それぞれ別 follow-up で行う。
- ルートマッピング（`65-inventory-record-traceability.md` §65.3 と同一）:

| `record_type` | 遷移先 |
|---|---|
| `receiving_record` | `/inventory/receiving/records/$recordId` |
| `return_record` | `/inventory/return/records/$recordId` |
| `manual_sale` | `/inventory/manual-sale/records/$recordId` |
| `disposal_record` | `/inventory/disposal/records/$recordId` |

- 「関連記録を見る」link は UI-11c-D16 / DSR-18 に従い、現在の `/settings/logs` と search state（`start_date` / `end_date` / `operation_type` / `page`）を直列化した `returnTo` を遷移先へ送る。詳細から戻ると同じ期間・種別・page を route search として復元する。link は展開行内の独立した操作であり、押下しても UI-11c-D5 の行展開 toggle を発火させない。
- 条件を満たさない場合（フィールド欠落、`record_type` が許可リスト外、`record_id` が0以下や非数値）はリンクを一切出さず、`summary` / detail 要約の表示のみ継続する。これは JSON の任意 key からの heuristic 推測ではなく、事前合意された2 field 名 + 許可リストという明示 contract に対する厳密一致判定である。
- **現状の producer 状況**: `receiving.rs` / `returns.rs` / `manual_sale.rs` / `disposal.rs` の各作成 producer は、§74.9 の許可リストに対応する `record_type` と当該業務記録 PK の `record_id` を detail_json に格納する。これにより新規ログは2 field の厳密一致条件を満たし、実データでも関連記録リンクの対象になる。その他の `insert_operation_log` 呼出し site は本契約の対象外であり、呼出し件数の転記は行わない（D-050）。

---

### 74.10 Pagination 契約

- 既定 `per_page = 50`（`Select` で 50 / 100 / 200）。
- `per_page` の上限は 200（`PAGINATION_MAX_PER_PAGE`、IO 層 `system_repo` で clamp、[43-cmd-settings-log.md](43-cmd-settings-log.md) §43.5）。
- 文言: `全 {total_count.toLocaleString("ja-JP")} 件のうち {from}〜{to} 件を表示（{page} / {totalPages} ページ）`（範囲付き統一形、`Pagination` をそのまま再利用）。
- **範囲外 page 回復**（UI-11c-D8）: `logsQuery.data.items.length === 0 && logsQuery.data.total_count > 0 && normalizedSearch.page > 1` の場合、通常の EmptyState ではなく専用メッセージ「このページには表示するログがありません」+ 「先頭ページに戻る」ボタン（`updateSearch({ page: 1 })`）を表示する。
- IO層は`page` / clamp後の`per_page`を`i64`へ変換してからoffsetを計算する。URL/CMD wireで表現可能な最大positive page（`u32::MAX`）でもRust側でpanic/wrapせず、SQLiteの範囲外offsetによる空`items`と上記回復導線へ到達させる。
- filter 変更時は常に `page=1`（§74.3）に戻るため、この回復導線は「filter 変更を伴わない外部要因（保持日数の cleanup、URL 直接改変、他端末からの delete 等）で総ページ数が減った」場合にのみ到達する。

---

### 74.11 Empty / Error / Retry / Loading

| 状態 | 表示 |
|---|---|
| loading（初回・filter変更・page変更） | table 領域に `Skeleton` 3行 |
| error（`listLogs` 失敗） | destructive `Alert`。`AlertTitle`「操作ログの取得に失敗しました」+ `AlertDescription` にエラーメッセージ + 「再試行」`Button`（`onClick`で`logsQuery.refetch()`）。現在の filter（start_date/end_date/operation_type/page）は保持したまま再試行する。 |
| 逆転範囲（UI送信前） | 入力欄付近のinline validation message。`listLogs` は呼ばず、直前のvalid query結果（table/pagination/展開行）を表示保持する。 |
| empty・既定 filter 一致（UI-11c-D9） | `EmptyState` title「この30日間の操作ログはありません」description「期間や種別を変更すると他のログを確認できます」。既定 filter 一致のため reset action は出さない（catalog ⑥、真にデータなし側） |
| empty・filter 適用中 | `EmptyState` title「該当する操作ログがありません」description「期間や種別を変更してください」+ `action`「絞り込みを解除」ボタン（2026-08-03 batch B、catalog ⑥ filter-empty reset action、SPEC-UIBB-1/2）。押下で `start_date` / `end_date` / `operation_type` / `page` をすべて既定値へ戻す |
| 範囲外 page（§74.10） | 専用メッセージ + 先頭ページに戻るボタン（EmptyState 系より優先して判定。filter-empty reset action とは対象ケースが異なり、同一画面内で共存する） |
| `typesQuery` 失敗 | operation_type filter を「すべて」固定・disabled にせず、既存 URL の `operation_type` 値があればそのまま select 選択肢に温存する（`typesQuery` が直っていない間も現在の絞り込みを失わない）。一覧本体（`logsQuery`）は独立して表示を継続する。 |

- **2026-08-03 batch B で追加**: 「絞り込みを解除」reset action（catalog ⑥ filter-empty reset action、SPEC-UIBB-1/2）を、既存の `defaultFilter` 判定（UI-11c-D9）の**非既定側にのみ**追加した（上表 empty・filter 適用中 行）。既定 filter 一致（真にデータなし）側には出さない。既存の範囲外 page 回復 action「先頭ページに戻る」（§74.10、UI-11c-D8）とは対象ケースが異なり（reset = filter 変更で 0 件、範囲外 page 回復 = 総件数減少で page が溢れた）、同一画面内で共存する。`StockMovementsPage` / `InventoryRecordsPage` / `ProductListPage` も同 batch で同じ reset action を追加済み（catalog ⑥ 6 site 統一、round 1 P1-1 で ProductList 追加）。

---

### 74.12 Lifecycle / staleTime

- `queryKey`: `queryKeys.operationLogs.list(effectiveNormalizedSearch)`、`queryKeys.operationLogs.types()`。逆転rangeのdraft searchはkeyを変えない。
- valid normalized search の snapshot 更新は commit 後の effect だけで行う。valid renderは現在値を直接 query key に使うため、通常のfilter/page変更に意図的な1 render遅延を加えない（UI-11c-D15）。
- `staleTime: 0` / `gcTime: 5 * 60_000`（両 query 共通）。`refetchOnWindowFocus: false`（既存グローバル既定を継承）。
- 自動ポーリング・自動再取得タイマーは持たない。保持日数の cleanup による total_count 変化は次回の filter 変更・page 操作・再訪・再試行で自然に反映される（§74.6）。

---

### 74.13 Accessibility / Keyboard / 非色

- 検索条件はすべて `<label htmlFor>` 付きの `<input type="date">` / `<select>`（自由入力の text input を持たない、UI-11c-D11）。IME `isComposing` 対応は本画面の入力コントロールには適用対象がない（自由入力欄がないため）。
- 行展開トリガーは詳細列のnative `<button>` とし、`aria-expanded` / `aria-controls` を持つ。Tabで到達可能、visible label / accessible nameは閉状態「詳細を表示」・開状態「詳細を閉じる」、Enter / Spaceはnative buttonのactivationで一度だけ展開トグルする。行全体clickは使わず、関連記録リンクのclickは展開toggleしない。
- 展開行のコピー・関連記録リンクボタンも通常の Tab 順序に含める。
- 種別・状態はラベル文字列を主情報にし、色のみに依存しない（`inventory-operator-ui` skill 準拠）。「その他（raw値）」は outline 系 Badge、既知カテゴリは stone 系ニュートラル Badge とし、色は補助のみ。
- コントラスト・フォーカスリングは UI_TECH_STACK §5.4 の outcome 契約（実装系統①、shadcn/ui primitive 既定の `focus-visible:ring-[3px]` 系）を継続利用する。

---

### 74.14 Traceability

- REQ-902 / TRACE-D3: 操作ログ画面自体、期間/種別 filter、pagination の automated test（Rust + RTL）。
- REQ-902（是正後） / UI-11c-D12: 既存 `test_list_logs_req905_pagination` / `test_list_logs_req905_filter` / `test_list_logs_req905_invalid_page_to_cmderror`（`src-tauri/src/cmd/settings_cmd.rs`）は次の実装 PR で関数名・コメントを REQ-902 に是正する。本 Design Phase では Rust コードを変更しないため、Test Design Matrix にこの是正を明示タスクとして記録する。
- REQ-902 / TRACE-D3 / UI-11c-D7: 関連業務記録リンクの表示条件・非表示条件。
- 新規追加する UI/Rust テストは `REQ-902` と該当 `UI-11c-Dn` を describe/it またはコメントに含める（`WF-TRACE-04` 準拠）。

---

### 74.15 Windows native L3 チェックリスト

| # | 画面 / 到達手順 | 観測可能な合格基準 |
|---|---|---|
| L3-1 | `/settings/logs` を開く（サイドバー「システム管理」→「操作ログ」） | 既定で直近30暦日・全種別のログが新しい順に一覧表示され、件数・ページ表記が読める |
| L3-2 | 開始日・終了日を個別に変更する（片側のみ、両方、逆転させる） | 片側指定は正常に絞り込まれる。逆転させた場合は保存前に入力欄付近でエラーメッセージが読め、一覧は書き換わらない |
| L3-3 | 種別ドロップダウンで既知の operation_type を選択する | 日本語ラベルで選択でき、選択後は該当種別のログのみ表示され `page=1` に戻る |
| L3-4 | 各行の「詳細を表示」ボタンをマウス、Enter、Spaceで操作し、別の行も展開する | 閉状態は「詳細を表示」、開状態は「詳細を閉じる」と可視textで分かる。既知 field の日本語要約が読め、「技術情報」は既定で閉じている。別行を開くと前の行は自動的に閉じ、詳細内の「関連記録を見る」を押しても意図しない展開toggleは起きない |
| L3-5 | 関連記録リンクが出るログ（テスト用に synthetic データを用意）と出ないログの両方を確認する | contract 通りの2 field が揃うログだけ「関連記録を見る」が表示され、遷移先の記録詳細が開く。それ以外のログはリンクが出ない |
| L3-6 | ページングで最終ページまで進み、意図的に既存データより大きい `page` を URL に直接入力する | 「このページには表示するログがありません」+「先頭ページに戻る」が表示され、ボタンで1ページ目に戻れる |
| L3-7 | §74.15.1のlocal-only SQLite exclusive-lock手順で一覧取得を失敗させ、lock解除後に再試行する | destructive Alert + 「再試行」ボタンが表示され、押すと現在の filter を保持したまま再取得され、成功すれば一覧が復帰する |
| L3-8 | §74.15.2のlocal-only synthetic setupでdefault emptyとfiltered emptyを別々に確認する | 前者は「この30日間の操作ログはありません」、後者は「該当する操作ログがありません」という異なる文言が表示され、いずれもfilter操作が残る |

---

#### 74.15.1 L3-7 error / retry synthetic lock procedure

対象はWindowsの**専用local development/demo DB（synthetic dataのみ）**であり、実店舗DBでは実行しない。appを開いたまま、別のPowerShell windowで次を実行する。SQLite WALでも`PRAGMA locking_mode=EXCLUSIVE`と`BEGIN EXCLUSIVE`の組み合わせは、既に開いている別connectionのreadを`database is locked`にすることをsynthetic WAL DBで確認済みである。`BEGIN IMMEDIATE`だけではWAL read failureを再現しないため使わない。

```powershell
$db = Join-Path $env:APPDATA 'com.inventory-system.desktop\inventory.db'
if (-not (Test-Path $db)) { throw "development/demo DB not found: $db" }
sqlite3 $db
```

SQLite promptで以下を入力し、このwindowを開いたままにする。DB内容は変更しない。

```sql
.bail on
PRAGMA busy_timeout = 0;
PRAGMA locking_mode = EXCLUSIVE;
BEGIN EXCLUSIVE;
SELECT COUNT(*) AS lock_probe FROM operation_logs;
```

appで`/settings/logs`を開き、開始日・終了日または種別を**validな値のまま**変更して一覧取得を起こす。app側connectionの`busy_timeout=5000`により最大約5秒待った後、destructive Alertと「再試行」が表示され、変更したfilter値が入力欄に残ることを確認する。lock windowで次を実行してlockを解除し、終了する。

```sql
ROLLBACK;
PRAGMA locking_mode = NORMAL;
.quit
```

appで「再試行」を押す。同じfilterのまま一覧または該当empty stateへ復帰すればpass。rowの投入・削除は行わない。PowerShell/SQLite output、DB、WAL/SHMをrepositoryやPRへ追加しない。

#### 74.15.2 L3-8 default / filtered empty synthetic procedure

live codeでは、初期schemaの`app_settings.key='backup_enabled'`は`value='1'`であり、起動時`src-tauri/src/lib.rs`から`mnt::backup::check_auto_backup`が呼ばれる。同関数は値が文字列`"1"`のときだけ自動バックアップを実行し、当日分がなければ`backup_create` operation logを作る。このためdefault-empty確認中だけ、**専用local development/demo DB**の当該値を文字列`"0"`へ変更し、終了時に取得済みの元値へ必ず復元する。実店舗DB、既存operator log、real dataには触れない。

以下はPowerShell変数だけに元値を保持し、`finally`でsynthetic行のcleanupと設定復元を保証する一連の手順である。assert用の`sqlite3`呼出しは`-batch -noheader`を指定し、単一値`SELECT`を見出しなしの1行として取得・parseする。DB / CLI outputはrepositoryやPRへ追加しない。

```powershell
# 1. アプリを停止してDB lockを解放する。
$db = Join-Path $env:APPDATA 'com.inventory-system.desktop\inventory.db'
if (-not (Test-Path $db)) { throw "development/demo DB not found: $db" }

# 実店舗DBへの誤適用防止。専用demo DBだと人が確認できる場合だけ続行する。
if ((Read-Host '専用local development/demo DBなら DEMO と入力') -ne 'DEMO') {
  throw 'L3-8 cancelled'
}

# 2. 現在値を取得し、PowerShell変数だけに記録する。
$backupEnabledBeforeOutput = @(& sqlite3 -batch -noheader $db "SELECT value FROM app_settings WHERE key='backup_enabled';")
$backupEnabledBeforeExit = $LASTEXITCODE
if ($backupEnabledBeforeExit -ne 0) {
  throw "backup_enabled read failed: sqlite3 exit=$backupEnabledBeforeExit"
}
if ($backupEnabledBeforeOutput.Count -ne 1) {
  throw "expected one backup_enabled result, got $($backupEnabledBeforeOutput.Count)"
}
$backupEnabledBefore = $backupEnabledBeforeOutput[0].Trim()
if ($backupEnabledBefore -notin @('0', '1')) {
  throw "unexpected backup_enabled contract value; abort: $backupEnabledBefore"
}

try {
  # 3. このdemo DB上だけでauto backupを一時無効化する。
  $disabledRowsOutput = @(& sqlite3 -batch -noheader $db "UPDATE app_settings SET value='0', updated_at=strftime('%Y-%m-%dT%H:%M:%S','now','localtime') WHERE key='backup_enabled'; SELECT changes();")
  $disabledRowsExit = $LASTEXITCODE
  if ($disabledRowsExit -ne 0) { throw "backup_enabled update failed: sqlite3 exit=$disabledRowsExit" }
  if ($disabledRowsOutput.Count -ne 1) { throw "expected one backup_enabled update result, got $($disabledRowsOutput.Count)" }
  $disabledRows = $disabledRowsOutput[0].Trim()
  if ($disabledRows -ne '1') { throw "backup_enabled update failed: changes=$disabledRows" }
  $disabledValueOutput = @(& sqlite3 -batch -noheader $db "SELECT value FROM app_settings WHERE key='backup_enabled';")
  $disabledValueExit = $LASTEXITCODE
  if ($disabledValueExit -ne 0) { throw "backup_enabled verification failed: sqlite3 exit=$disabledValueExit" }
  if ($disabledValueOutput.Count -ne 1) { throw "expected one backup_enabled verification result, got $($disabledValueOutput.Count)" }
  if ($disabledValueOutput[0].Trim() -ne '0') {
    throw 'backup_enabled was not disabled'
  }

  # 4. 既定30日内を含めoperation_logs全体が0件であることを確認する。
  $allLogsBeforeOutput = @(& sqlite3 -batch -noheader $db "SELECT COUNT(*) FROM operation_logs;")
  $allLogsBeforeExit = $LASTEXITCODE
  if ($allLogsBeforeExit -ne 0) { throw "initial operation_logs count failed: sqlite3 exit=$allLogsBeforeExit" }
  if ($allLogsBeforeOutput.Count -ne 1) { throw "expected one initial operation_logs count, got $($allLogsBeforeOutput.Count)" }
  $allLogsBefore = $allLogsBeforeOutput[0].Trim()
  if ($allLogsBefore -ne '0') { throw "clean demo DB required: operation_logs=$allLogsBefore" }

  # 5-6. アプリを起動し、URL searchなしで /settings/logs を開く。
  # 「この30日間の操作ログはありません」と開始日・終了日・種別filterが残ることを確認する。
  Read-Host 'アプリ起動→default-empty文言/filter確認→アプリ停止後、Enter'

  $allLogsAfterDefaultOutput = @(& sqlite3 -batch -noheader $db "SELECT COUNT(*) FROM operation_logs;")
  $allLogsAfterDefaultExit = $LASTEXITCODE
  if ($allLogsAfterDefaultExit -ne 0) { throw "post-default operation_logs count failed: sqlite3 exit=$allLogsAfterDefaultExit" }
  if ($allLogsAfterDefaultOutput.Count -ne 1) { throw "expected one post-default operation_logs count, got $($allLogsAfterDefaultOutput.Count)" }
  if ($allLogsAfterDefaultOutput[0].Trim() -ne '0') {
    throw 'default-empty確認後にoperation_logsが増えたため中止'
  }

  # filtered empty: 同じdemo DBへ当日のsynthetic rowを1件だけ投入する。
  $insertedRowsOutput = @(& sqlite3 -batch -noheader $db "INSERT INTO operation_logs (operation_type,summary,detail_json,created_at) VALUES ('backup_create','UI-11c L3-8 filtered-empty synthetic','{`"synthetic`":true}',strftime('%Y-%m-%dT%H:%M:%S','now','localtime')); SELECT changes();")
  $insertedRowsExit = $LASTEXITCODE
  if ($insertedRowsExit -ne 0) { throw "synthetic insert failed: sqlite3 exit=$insertedRowsExit" }
  if ($insertedRowsOutput.Count -ne 1) { throw "expected one synthetic insert result, got $($insertedRowsOutput.Count)" }
  $insertedRows = $insertedRowsOutput[0].Trim()
  if ($insertedRows -ne '1') { throw "synthetic insert failed: changes=$insertedRows" }

  # アプリを起動し、synthetic rowを確認後、終了日を昨日へ変更する。
  # 「該当する操作ログがありません」とfilterが残ること、当日へ戻すとrowが再表示されることを確認する。
  Read-Host 'アプリ起動→filtered-empty文言/filter復帰確認→アプリ停止後、Enter'
}
finally {
  # 7. ここへ来る前にアプリを停止する。8. 元値を必ず復元する。
  # cleanup側と設定復元側のerrorを別々に保持し、両方を試行した後でまとめて報告する。
  $cleanupErrors = @()
  $restoreErrors = @()

  try {
    # `changes()` は対象DELETEの直後に取得する。間に別のDMLを挟まない。
    try {
      $deletedRowsOutput = @(& sqlite3 -batch -noheader $db @"
DELETE FROM operation_logs
WHERE summary IN ('UI-11c L3-8 default-empty synthetic', 'UI-11c L3-8 filtered-empty synthetic');
SELECT changes() AS deleted_rows;
"@)
      $deletedRowsExit = $LASTEXITCODE
      if ($deletedRowsExit -ne 0) {
        $cleanupErrors += "synthetic DELETE failed: sqlite3 exit=$deletedRowsExit"
      } elseif ($deletedRowsOutput.Count -ne 1) {
        $cleanupErrors += "expected one deleted_rows result, got $($deletedRowsOutput.Count)"
      } else {
        $deletedRows = [int]::Parse($deletedRowsOutput[0].Trim())
        if ($deletedRows -ne 1) {
          $cleanupErrors += "expected deleted_rows=1, got $deletedRows"
        }
      }
    }
    catch {
      $cleanupErrors += "synthetic DELETE/parse failed: $($_.Exception.Message)"
    }

    # 9. synthetic行とdefault-emptyの全log状態が残っていないことを確認する。
    try {
      $remainingOutput = @(& sqlite3 -batch -noheader $db "SELECT COUNT(*) FROM operation_logs WHERE summary LIKE 'UI-11c L3-8 % synthetic';")
      $remainingExit = $LASTEXITCODE
      if ($remainingExit -ne 0) {
        $cleanupErrors += "synthetic remaining query failed: sqlite3 exit=$remainingExit"
      } elseif ($remainingOutput.Count -ne 1) {
        $cleanupErrors += "expected one synthetic remaining result, got $($remainingOutput.Count)"
      } else {
        $syntheticRemaining = [int]::Parse($remainingOutput[0].Trim())
        if ($syntheticRemaining -ne 0) {
          $cleanupErrors += "expected synthetic remaining=0, got $syntheticRemaining"
        }
      }
    }
    catch {
      $cleanupErrors += "synthetic remaining parse failed: $($_.Exception.Message)"
    }

    try {
      $allLogsOutput = @(& sqlite3 -batch -noheader $db "SELECT COUNT(*) FROM operation_logs;")
      $allLogsExit = $LASTEXITCODE
      if ($allLogsExit -ne 0) {
        $cleanupErrors += "all-log restore query failed: sqlite3 exit=$allLogsExit"
      } elseif ($allLogsOutput.Count -ne 1) {
        $cleanupErrors += "expected one all-log result, got $($allLogsOutput.Count)"
      } else {
        $allLogsAfter = [int]::Parse($allLogsOutput[0].Trim())
        if ($allLogsAfter -ne 0) {
          $cleanupErrors += "expected default-empty log state to be restored, got operation_logs=$allLogsAfter"
        }
      }
    }
    catch {
      $cleanupErrors += "all-log restore parse failed: $($_.Exception.Message)"
    }
  }
  finally {
    # cleanupの実行・parse・assertが失敗しても、設定復元と復元値確認は必ず試行する。
    try {
      $restoreRowsOutput = @(& sqlite3 -batch -noheader $db "UPDATE app_settings SET value='$backupEnabledBefore', updated_at=strftime('%Y-%m-%dT%H:%M:%S','now','localtime') WHERE key='backup_enabled'; SELECT changes();")
      $restoreRowsExit = $LASTEXITCODE
      if ($restoreRowsExit -ne 0) {
        $restoreErrors += "backup_enabled restore failed: sqlite3 exit=$restoreRowsExit"
      } elseif ($restoreRowsOutput.Count -ne 1) {
        $restoreErrors += "expected one backup_enabled restore result, got $($restoreRowsOutput.Count)"
      } else {
        $restoreRows = [int]::Parse($restoreRowsOutput[0].Trim())
        if ($restoreRows -ne 1) {
          $restoreErrors += "backup_enabled restore failed: changes=$restoreRows"
        }
      }
    }
    catch {
      $restoreErrors += "backup_enabled restore/parse failed: $($_.Exception.Message)"
    }

    try {
      $backupEnabledAfterOutput = @(& sqlite3 -batch -noheader $db "SELECT value FROM app_settings WHERE key='backup_enabled';")
      $backupEnabledAfterExit = $LASTEXITCODE
      if ($backupEnabledAfterExit -ne 0) {
        $restoreErrors += "backup_enabled verification failed: sqlite3 exit=$backupEnabledAfterExit"
      } elseif ($backupEnabledAfterOutput.Count -ne 1) {
        $restoreErrors += "expected one backup_enabled verification result, got $($backupEnabledAfterOutput.Count)"
      } elseif ($backupEnabledAfterOutput[0].Trim() -ne $backupEnabledBefore) {
        $restoreErrors += "backup_enabled restore failed: expected=$backupEnabledBefore got=$($backupEnabledAfterOutput[0].Trim())"
      }
    }
    catch {
      $restoreErrors += "backup_enabled verification/parse failed: $($_.Exception.Message)"
    }
  }

  if ($cleanupErrors.Count -gt 0 -or $restoreErrors.Count -gt 0) {
    $failureReport = @()
    if ($cleanupErrors.Count -gt 0) {
      $failureReport += "cleanup errors: $($cleanupErrors -join '; ')"
    }
    if ($restoreErrors.Count -gt 0) {
      $failureReport += "restore errors: $($restoreErrors -join '; ')"
    }
    throw ($failureReport -join [Environment]::NewLine)
  }
}
```

手順5-7の各`Read-Host`は、アプリ停止を確認してからEnterする。途中で確認に失敗しても外側`finally`内の二重`try/finally`を完走し、cleanup assertionが失敗しても`backup_enabled`復元を必ず試行する。DELETE直後の`deleted_rows=1`、synthetic row remaining count = 0、`backup_enabled`の元値復元、default-empty用の全`operation_logs`状態（0件）への復元をすべて確認し、cleanup errorとrestore errorは復元試行後にまとめてthrowする。いずれかが失敗した場合はL3完了扱いにしない。L3-7/L3-8とも実施結果をこのrepositoryへcommitせず、DB/CLI output/screenshotに実店舗情報を含めない。

---

### 74.16 非目的

| やらないこと | 理由 | 責務を持つモジュール |
|---|---|---|
| CSV 出力 | Owner Decision #1。viewing MVP に含めない別 task | 将来 IO-05 拡張 |
| 操作ログの保持設定変更・手動削除 | 本画面は閲覧専用。保持日数による自動削除は MNT-02 の契約のまま（[72](72-mnt-log-manager.md) MNT-02-D1） | `72-mnt-log-manager.md` |
| 診断ログファイル/ディレクトリ導線（MNT-04） | 別種のログ（アプリ診断ログ）であり operation_logs とは別責務 | `70-mnt-diagnostic-log.md` |
| 業務記録の訂正・取消 | 操作ログは監査ログであり業務記録の代替にしない（TRACE-D3） | `65-inventory-record-traceability.md` |
| `csv_import` / `stocktake` 記録種別の関連リンク | 両種の詳細 route は実装済みだが、2026-09-01 の producer 実効化 R3 では据置とし、許可リストからの除外を維持する。csv_import / stocktake producer への `record_type` 採用と関連リンク追加は、それぞれ別 follow-up で行う | `65-inventory-record-traceability.md` §65.10 実装スライス |
| records 画面の期間 predicate を JST inclusive/exclusive に統一 | scenario が異なるため本 PR では変更しない。実運用で境界問題が出た場合に横展開を検討 | `21-io-inventory-repo.md` / decision-log D-037 |

---

### 74.17 Adjacent Pattern Audit

`StockMovementsPage` と `InventoryRecordsPage` を比較対象にした。

| 観点 | `StockMovementsPage` | `InventoryRecordsPage` | UI-11c 採用 |
|---|---|---|---|
| URL state | zod、`.catch()`、filter変更でpage=1 | 同左 | 再利用（同一パターン） |
| filter reset ボタン | なし（設計時点） | なし（設計時点） | なし（横断 backlog に合わせる、§74.11。**2026-08-03 batch B で 3 画面とも filter-empty reset action を追加**、[02-component-catalog.md](../design-system/02-component-catalog.md) ⑥、§74.11 参照） |
| pagination | `Pagination` 既定 50 + `Select` | 同左 | 再利用 |
| retry ボタン | **なし**（Alert のみ、文言のみで再試行の導線なし） | **なし**（同左） | **追加する**（Missing UI item 9 の明示要求 + `DailySalesPage`/`MonthlySalesPage`/`ThresholdSettingsPage`/`StocktakePage` の既存 retry パターンを再利用。意図的な相違、§74.2 UI-11c-D9） |
| EmptyState | 単一文言 | 単一文言 | **2系統に分岐**（既定0件 vs filter該当0件。意図的な相違、UI-11c-D9） |
| 範囲外 page 回復 | なし | なし | **追加する**（意図的な相違、UI-11c-D8） |
| keyboard/行展開 | 該当機能なし | 該当機能なし | 新規パターン（§74.7、既存踏襲元なし） |
| 期間 predicate | 業務日の両端を含む（b2） | 日付の列の両端を含む（`<= date_to`） | **JST inclusive/exclusive に変更**（意図的な相違、UI-11c-D2） |
| 明示clear日付 | 日付キー欠落は既定値へfallback | 同左 | 片側・両側clearをURL空文字で明示し、初期未初期化と区別する（UI-11c-D1。既存画面への横展開はしない） |

---

### 74.18 Mutation / Anti-tautology Questions

Test Design Matrix 作成時に、以下が「モックの偶然一致」でグリーンにならないことを確認する:

1. `start_date`/`end_date` の一方だけを与えたテストと両方与えたテストで、SQL の `WHERE` 句が実際に異なる条件を含むか（`created_at >= X` のみ vs `created_at >= X AND created_at < Y`）を、モックではなく実 SQLite 接続で検証する。
2. `items` 取得 SQL と `COUNT(*)` SQL が同じ `WHERE` 句パラメータを使っているかを、期間 filter を変えて total_count が実際に変動することで検証する（固定値モックでは検出できない）。
3. `list_log_operation_types` のテストで、operation_logs に複数の同一 operation_type 行がある場合に結果が重複しない（distinct が効いている）ことを確認する。
4. detail_json の「既知 field 要約」テストは、既知 key と未知 key を混在させた synthetic JSON で、既知 key だけがラベル変換され未知 key は raw 表示されることを個別 assertion で確認する（どちらも同じ見た目になる実装ミスを検出する）。
5. 関連記録リンクのテストは、valid positive safe integer、zero、negative、fractional、numeric string、`Number.MAX_SAFE_INTEGER`超過、未知/欠落`record_type`、欠落`record_id`を独立caseにし、`> 0`・`Number.isSafeInteger`・非coercionの各guardを外すmutationを検出する。
6. 逆転range保持テストはpage > 1、`total_count > per_page`、複数pageのfixtureを使い、table / expanded rowだけでなくpage番号・total_count・前後pagination controlを保持する。`page={effectiveSearch.page}`をdraft normalized pageへ戻すmutationを検出する。
7. 範囲外 page 回復のテストは、`items: []` かつ `total_count: 0`（真の空）と `items: []` かつ `total_count > 0`（範囲外）を別ケースとして用意し、異なる文言/導線になることを確認する。

---

### 74.19 Negative-space Audit

- `db-design/tracking-system-tables.md` §18 が持つ `detail_json` の「変更前後の値等」という言及に対し、既知 field 要約辞書は初期実装時点では限定的（backup 系の `file_name`/`size_bytes` 等）であり、商品修正等の「変更前後」フィールドの辞書化は実装時の棚卸し対象として明記した（§74.8）。Ledger には「既知 field 辞書は初期実装時点の棚卸し範囲に限定」を明記する。
- PR #164で`src/config/navigation.ts`の`ui-11c`を`to: "/settings/logs"` + `status: "active"`へ更新し、専用navigation testで固定済み。
- `SCREEN_DESIGN.md` §18 行の QR-06 記述は本 Design Phase で同期する（§74.16 / 後述 SCREEN_DESIGN.md 更新）。

---

### 更新履歴

| 日付 | PR | 内容 |
|---|---|---|
| 2026-08-30 | PR #20 / DSR-18 | UI-11c-D16 を追加し、「関連記録を見る」が期間・種別・page を含む操作ログ URL を `returnTo` として送り、詳細から同じ調査 state へ戻る契約を確定。 |
| 2026-07-11 | Design Phase（本 PR） | 新規作成。UI-11c 操作ログ画面の route/URL state、JST 期間 predicate、canonical operation_type registry（新規 CMD/IO 含む）、table/detail_json/関連記録リンク契約、pagination/empty/error/retry、a11y、traceability 是正、Windows native L3 を確定 |
| 2026-07-12 | PR #164 final audit remediation | Owner裁定によりD5の展開正本を明示的な「詳細を表示／閉じる」buttonへ同期。D7をpositive safe integerへ厳格化し、L3-8を自動バックアップ無効化・復元を含むdemo DB限定手順へ更新 |
| 2026-07-12 | PR #164 final-audit remediation | CMD日付をASCII strict形式+実在暦日へ明確化。片側/明示clear URL state、逆転rangeで直前valid一覧を保持するlifecycle、3 filter個別page resetのtest契約、L3-7 exclusive-lock / L3-8 synthetic emptyの再現手順を追記 |
| 2026-08-03 | ui-polish-batch-b（本 PR） | §74.11 の「empty・filter 適用中」に filter-empty reset action（catalog ⑥、SPEC-UIBB-1/2）を追加し、既定 filter 一致側とは非表示のまま区別することを明記。既存の範囲外 page 回復 action（UI-11c-D8）との共存を明記。§74.17 Adjacent Pattern Audit の filter reset ボタン行を実装後の状態へ更新 |
| 2026-08-28 | docs-hygiene-sync（本 PR） | stocktake 除外理由 3 箇所（UI-11c-D7 表 row / §74.9 相当 bullet / §末尾表）の理由を、詳細 route の不在を根拠とする旧表現から「stocktake 系 `record_type` を書き込む producer が0件」〈65 §65.8.3 と同型〉へ差替え。結論〈許可リストからの除外維持〉は不変 |
