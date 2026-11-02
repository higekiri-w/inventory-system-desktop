# 69. UI-11a: 閾値設定画面（ThresholdSettingsPage）

> **親文書**: [FUNCTION_DESIGN.md](../FUNCTION_DESIGN.md)
> **入力ドキュメント**: [architecture/ui-task-specs.md](../architecture/ui-task-specs.md) UI-11a、[DB_DESIGN.md](../DB_DESIGN.md) 設計方針メモ D-4、[db-design/tracking-system-tables.md](../db-design/tracking-system-tables.md) app_settings、[43-cmd-settings-log.md](43-cmd-settings-log.md)、[68-ui-backup-restore.md](68-ui-backup-restore.md)（設定エリアの先行画面）
> **対応タスク / 仕様**: UI-11a（QR系、D-4 在庫少閾値）

## 69.1 目的

在庫少判定の基準値（一般商品 = 個数、生地 = cm）を利用者が画面から変更できるようにする。基準値は `app_settings` の `stock_low_threshold` / `stock_low_threshold_fabric` に保存され、BIZ 層の在庫少判定（`list_low_stock`、UI-00 ホームサマリ、UI-06a 在庫少一覧）が参照する。開発側のタスク名は「閾値設定画面」だが、operator 向けの画面名・ナビゲーションラベルは「在庫少の基準」とする（UI-11a-D6）。

## 69.2 関数要求

- 現在の基準値 2 件を表示し、数値入力で変更・保存できる（D-4: 最小値 1、0 以下は拒否）。
- 保存は既存 CMD-11 `update_setting` のみで完結し、新規 CMD・DB 変更を伴わない。
- 保存後は在庫少判定を参照する画面（ホーム / 「商品と在庫」。D-138 の後〈proposed・未実装〉の画面名で、今の build は在庫照会）に次の取得から反映される。

## 69.3 Design Decisions

### UI-11a-D1: この画面が所有する app_settings key は 2 件に限定する

- **決定**: UI-11a が読み書きするのは `stock_low_threshold` と `stock_low_threshold_fabric` のみ。`getSettings` の戻りからこの 2 key だけを抽出し、他の key は無視する（68-ui の backup 4 key 抽出パターンの踏襲）。
- **Why**: 非 IT の利用者に内部 key を並べる汎用エディタは誤操作リスクが高い。UI-11b-D6 が backup 系 4 key の所有を UI-11b に固定した対称として、在庫少基準 2 key の所有を UI-11a に固定する。
- **Rejected**: app_settings 全 key の汎用テーブル編集（`backup_path` 等を壊せてしまう）。`tax_rate_standard` / `tax_rate_reduced` / `log_retention_days` の同居（在庫少とは業務文脈が別で、変更頻度と誤変更影響の検討をこの画面のスコープに持ち込まない。編集 UI が要ると分かった時点で該当ドメイン画面の Design Phase で扱う）。
- **Revisit trigger**: 商品個別閾値 `custom_low_threshold`（D-4 の将来拡張）の採用時、または利用者から他の運用パラメータ変更の要望が出た時。

### UI-11a-D2: 保存は単一「保存する」ボタン、変更した key のみ順次 update_setting

- **決定**: 主動線は「保存する」1 個（DSR-01）。保存時は変更（dirty）になった key だけを順に `updateSetting` で送る。確認ダイアログは挟まない（DSR-07: 値を入れ直して再保存すれば戻せる可逆操作）。
- **Why**: 既存 CMD は 1 key ずつの upsert であり、2 key のために一括更新 CMD を新設するのは過剰。変更していない key を送らないことで、部分失敗時に「何が変わったか」を事実どおりに説明できる。
- **Rejected**: 一括更新 CMD の新設（CMD 増殖、スライス肥大）。フロント側での擬似 all-or-nothing（DB 側に rollback 手段がなく、実際は片方だけ保存された状態を隠す嘘の原子性表示になる）。
- **Revisit trigger**: 設定 key が増えて順次送信の失敗組合せが説明困難になった時。

### UI-11a-D3: 入力検証は frontend（zod schema、既存フォームパターン）で整数 1〜99999、CMD/BIZ は変更しない

- **決定**: 両フィールドとも「整数のみ・最小 1・最大 99999・空欄拒否」。最小 1 は D-4 既定。最大 99999 は本設計で追加する sanity bound。`update_setting` 側には key 別の業務バリデーションを追加しない。
- **Why**: 最大値は、HID バーコードスキャナの誤爆（数値欄フォーカス中のスキャンで 13 桁 JAN が入力される）や打鍵ミスによる桁外れ値を遮断するための上限で、業務上の最大想定在庫（生地でも 99999cm = 約 1km）を大きく超えない。`update_setting` は汎用 KVS の薄いラッパーであり、key 別ルールを持たせると CMD 層に業務ルールを置かないレイヤー原則に反する。
- **Rejected**: CMD/BIZ 層での閾値専用バリデーション追加（レイヤー原則違反 + 本スライスの scope 外）。上限なし（誤爆事故を許す）。
- **Revisit trigger**: DB 直接操作以外の更新経路（インポート等）が増え、frontend 検証だけでは不正値混入を防げなくなった時。
- **実装 PR での確認事項**: BIZ 側（`list_low_stock` の閾値読み出し）が非数値の設定値に遭遇した場合の既存挙動を実コードで確認し、防御がなければ backlog に起票する（本設計では BIZ の挙動を変更しない）。

### UI-11a-D4: 反映は「保存 = 即時」、UI 側は query invalidation で追随

- **決定**: BIZ は在庫少判定のたびに `app_settings` を読むため（[44-cmd-inventory.md](44-cmd-inventory.md) `list_low_stock`）、保存が完了した時点で次の判定から新基準が使われる。UI 側は保存成功後に settings 取得 query と在庫少系 query を invalidate する（§69.10）。
- **Why**: backend にキャッシュ層がないので、追加の反映機構は不要。ui-task-specs.md の「変更は即時反映（次回の在庫少一覧から適用）」をそのまま満たす。
- **Rejected**: 反映専用のイベント通知や再起動要求（過剰）。

### UI-11a-D5: route は `/settings/thresholds`、URL search state なし

- **決定**: `/settings/backup`（UI-11b）と並ぶ `/settings/thresholds`。この画面は共有・復元したい表示状態を持たないフォーム単体のため、search param は定義しない。route は実装済みで、`src/config/navigation.ts` の `ui-11a` エントリも active 化済み（UI-11b と同じ運用）。
- **Why**: 状態の URL 化は「復元・共有する価値のある状態」に適用する原則であり、2 入力のフォームに該当する状態がない。
- **Rejected**: `/settings` 1 画面への閾値・バックアップ・ログの同居（ARCHITECTURE.md の UI-11a/11b/11c 分割と UI-11b-D6 の画面別所有に反する）。

### UI-11a-D6: operator 向け名称は「在庫少の基準」、「閾値」は画面に出さない

- **決定**: ナビゲーションラベル・ウィンドウタイトル・ページ h1 は「在庫少の基準」に統一済み（`navigation.ts` の `ui-11a` の label / title も「在庫少の基準」へ変更済み）。「閾値」という語は operator 向け画面文言に使わない。開発ドキュメント上のタスク名「閾値設定画面」は従来どおり。
- **Why**: 「閾値」は非 IT の利用者に読めない専門語。ドメイン語「在庫少」を主語にした表示名が実態と一致する（表示名と実態の一致原則）。sidebar・タイトル・h1 の不一致は迷いを生むため 3 点を同時に揃える。
- **Rejected**: 「閾値設定」のまま実装（専門語）。「設定」だけの汎称（バックアップ・ログと区別できない）。

### UI-11a-D7: Windows native L3 は軽量 2 項目

- **決定**: §69.12 の 2 項目のみ。数値入力だけで日本語 IME に依存しないため、IME 制約は L3 必須の根拠にしない。それでも保存 → 在庫少判定への反映という operator 導線は実機で 1 回目視する。
- **Why**: UI-11b の「native runtime でしか確認できない項目だけを L3 にする」方針の踏襲。この画面はファイル・DB 実体操作を伴わない。

### UI-11a-D8: field descriptor を schema・抽出・保存の単一所有者にする

- **決定**: UI-11a が所有する2 fieldは、field名・`app_settings` key・日本語label・保存順・validation schemaをfeature-localの単一descriptorから導出する。`ThresholdField`、初期値抽出、issue path guard、dirty field列挙、`update_setting` entryはdescriptor外へ同じfield集合を複製しない。
- **Why**: schemaだけにfieldを追加して保存されない、または保存側だけにfieldを追加してform値が存在しない片側変更を、型検査とdescriptor testで止めるため。既存の2 key、一般商品→生地の順、1〜99999、dirty-only、部分失敗表示は変えない。
- **Rejected**: Zod objectと保存用union/mapを別々に手書きしてtestだけで一致を監視する案（新field追加時の同期箇所が残る）。汎用settings editor化（UI-11a-D1に反する）。

## 69.4 Route / Components

```
src/routes/settings/thresholds.tsx      … route 定義（search param なし）
src/features/threshold-settings/
  ThresholdSettingsPage.tsx             … 画面本体（PageHeader + 入力区画〈`<section>`、見出しなし〉+ 保存ボタン）
  hooks/useThresholdSettings.ts         … getSettings を包む useQuery + 2 key 抽出
  hooks/useSaveThresholds.ts            … dirty key の順次 updateSetting を包む useMutation
  lib/extract-thresholds.ts             … AppSetting[] → { stockLowThreshold, stockLowThresholdFabric }（純関数）
  lib/threshold-form-schema.ts          … zod schema（整数 1〜99999、§69.7 の文言）
```

- 共通部品は `patterns/PageHeader` / `FieldError` を再利用する。入力区画は h2 が不要なため `FormSection`（パターン④、title 必須）を使わず、④ の DOM から h2 を除いた形（説明 `<p>` + `Separator` + 入力）を画面内に持つ（owner 判断 2026-09-15、PR #64）。
- ディレクトリは既存 feature の実配置規約（例: UI-11b = `src/features/backup-restore/`）に合わせた `src/features/threshold-settings/`。

### シグネチャ

擬似シグネチャ（TypeScript レベルの契約。ファイル配置と同様、細部は実装 PR で確定する）:

```ts
// AppSetting[] から所有 2 key を抽出（純関数、DB 非依存）
function extractThresholds(settings: AppSetting[]): {
  stockLowThreshold: string;        // stock_low_threshold の raw value
  stockLowThresholdFabric: string;  // stock_low_threshold_fabric の raw value
}

// getSettings を包む useQuery + 抽出
function useThresholdSettings(): UseQueryResult<ThresholdValues, InvokeError>

// dirty key のみ順次 updateSetting。失敗 key を含む結果を返す
function useSaveThresholds(): UseMutationResult<ThresholdSaveResult, InvokeError, ThresholdSaveRequest>
```

## 69.5 処理ステップ / 状態

**処理ステップ**:

1. 画面表示時に `commands.getSettings()` を実行し、`extract-thresholds` で 2 key を抽出してフォーム初期値にする
2. 利用者が数値を編集する（既存フォームパターン = useState + errors record。dirty 判定を未保存フラグとして保持。react-hook-form は本 repo で不使用のため導入しない）
3. 「保存する」押下 → zod 検証（§69.7）→ dirty key のみ順に `commands.updateSetting()`
4. 全件成功 → 成功 toast + D-052-C13 適用（§69.10）+ フォームを保存値で pristine 化。部分成功でも成功 field が 1 件以上なら同じ C13 を適用する
5. 一部失敗 → §69.8 の部分失敗表示

**画面状態**: `loading`（Skeleton）/ `load-error`（上部 Alert + 再試行）/ `ready`（pristine: 保存ボタン disabled、dirty: enabled）/ `saving`（保存ボタン disabled + 入力 disabled、二重送信防止）。UI-11b のような多段 state machine は持たない（get/update のみの単純画面）。

## 69.6 Command Contract

新規 CMD なし。既存 CMD-11（[43-cmd-settings-log.md](43-cmd-settings-log.md) §43.3-43.4、generated bindings 登録済み = PR #141）のみを使う。

| 呼び出し | generated binding | 戻り |
|---|---|---|
| 設定取得 | `commands.getSettings()` | `AppSetting[]`（`{ key, value, updated_at }`、value は文字列） |
| 設定保存 | `commands.updateSetting({ key, value })` | `null`（`system_repo` upsert） |

- wire 上の value は文字列。数値化・検証は UI 側の責務（UI-11a-D3）。
- 対象 key は `stock_low_threshold`（初期値 "3"）と `stock_low_threshold_fabric`（初期値 "500"）。

## 69.7 入力検証

| ルール | エラー文言（FieldError、`role="alert"`） |
|---|---|
| 空欄 | 「1〜99999の整数を入力してください」 |
| 整数以外（小数・文字） | 「1〜99999の整数を入力してください」 |
| 1 未満 | 「1〜99999の整数を入力してください」（D-4: 0 以下拒否） |
| 99999 超 | 「1〜99999の整数を入力してください」（UI-11a-D3 sanity bound） |

- 検証エラーは発生源直近のインライン 1 スロット（DSR-03）。検証エラーがある間は保存を実行しない。
- 保存済みの既存値が数値として読み取れない場合（DB 直接操作等の異常系）: 該当フィールドを空欄で表示し、FieldError「現在の設定値が読み取れません。正しい値を入力して保存してください」を出す。正しい値の保存で回復する。

## 69.8 保存フロー / Error / Recovery

- **取得失敗**（`getSettings` エラー）: 画面上部に destructive Alert「設定を読み込めませんでした」+ 再試行ボタン。フォームは出さない（DSR-03 回復導線つき Alert）。
- **保存失敗（全件）**: 保存ボタン近くの inline Alert（destructive）「保存に失敗しました。もう一度保存してください」+ `toast.error` 併用。入力値は保持し、再度「保存する」で再試行。
- **保存失敗（部分）**: 順次送信のうち失敗した key を日本語フィールド名で明示する。例: 「生地の基準の保存に失敗しました。一般商品の基準は保存済みです。もう一度保存してください」。settings query を refetch して保存済み実値を再表示し、失敗フィールドの入力値は保持する。再保存時は dirty な失敗分だけが送られる（UI-11a-D2 の帰結）。
- 保存成功・失敗いずれも画面遷移しない。`scrollPageToTop()` は使わない（フォームと結果表示が 1 画面に収まり、遷移後の見落とし問題が発生しない）。

## 69.9 UI / Wording

| 場所 | 文言 |
|---|---|
| ナビ / タイトル / h1 | 在庫少の基準 |
| PageHeader 説明 | 在庫がこの数以下になったら「在庫少」としてお知らせします |
| 区画 見出し | （なし、説明文のみ。owner 判断 2026-09-15） |
| 区画 説明 | 保存すると、ホームと「商品と在庫」の在庫少の判定にすぐ反映されます（D-138 の後の文。proposed・未実装で、今の build は「ホームと在庫照会の在庫少の判定」） |
| フィールド 1 ラベル | 一般商品の基準（必須） |
| フィールド 1 補足 | 在庫がこの個数以下になったら在庫少（初期値: 3個） |
| フィールド 2 ラベル | 生地の基準（必須） |
| フィールド 2 補足 | 在庫がこの長さ以下になったら在庫少（初期値: 500cm = 5m） |
| 保存ボタン | 保存する |
| 成功 toast | 在庫少の基準を保存しました（一般商品: {n}個以下 / 生地: {m}cm以下） |

- 必須はラベル内「（必須）」で示す（DSR-06）。単位（個 / cm）は入力欄の隣に常時表示し、色に意味を持たせない（DSR-08）。
- 成功 toast は保存値を含む具体文言、安定 id `threshold-save-success`（パターン⑦ id 規約）。
- フォームは 1 セクション（DSR-09: 意味境界が 1 つ）。数値入力の見た目は既存の数量入力パターン（入庫記録等）に合わせる。

## 69.10 Query Invalidation

1 件以上の設定保存に成功した場合（全成功・部分成功の両方）、[D-052](../decision-log.md) C13 の SSOT helper を適用する。具体的な query key 集合は `src/lib/invalidation-contract.ts` だけに置き、失敗だけで成功 field が 0 件の場合は invalidate しない。

## 69.11 テスト設計の起点

RTL（text / role / value assertion、色 class のみの assert は不可）:

- 検証 4 系統（空欄 / 非整数 / 0 / 100000）で FieldError 文言が出て `updateSetting` が呼ばれない
- pristine では保存ボタン disabled、編集で enabled
- 全成功と部分成功（成功 field 1 件以上）の両方で、実呼出し集合が D-052-C13 の独立 test oracle と完全一致する
- 片方の key だけ編集して保存 → `updateSetting` が該当 key のみ 1 回呼ばれる（UI-11a-D2）
- 部分失敗で失敗フィールド名を含む Alert が出る（§69.8 文言）
- `getSettings` 失敗で上部 Alert + 再試行が出る
- 既存値が非数値のとき空欄 + 回復文言が出る（§69.7）

## 69.12 Windows Native L3

| # | 確認項目 | 合格基準 |
|---|---|---|
| L3-1 | 基準値を変更して保存 → ホームと「商品と在庫」を開く（D-138 の後。今の build は在庫照会） | 在庫少の件数・一覧が新しい基準で変わって見える |
| L3-2 | アプリを再起動して本画面を開く | 保存した値が表示されている |

数値入力のみで日本語 IME に依存しないため、L3 はこの 2 項目に限定する（UI-11a-D7）。実店舗データを含む証跡は repo に残さない。

## 69.13 Non-scope

- `backup_enabled` / `backup_time` / `backup_path` / `backup_retention_days` の表示・編集（UI-11b 所有、UI-11b-D6）
- `tax_rate_standard` / `tax_rate_reduced` / `log_retention_days` / `log_last_cleanup_date` / `last_plu_export_at` の編集 UI（UI-11a-D1）
- 商品個別閾値 `custom_low_threshold`（D-4 の将来拡張）
- 未保存変更の離脱ガードの独自実装（UI_TECH_STACK §6.11 `useUnsavedChangesWarning` の適用対象 — 2026-08-03 UI 安全網 batch で既存 isDirty を共通 hook へ接続する。本画面が独自のガードを持つことはしない）
- 実装では既存の `values` / `savedValues` 比較をそのまま共通 `useUnsavedChangesWarning` へ接続し、保存成功時の `setSavedValues(submittedValues)` で解除する。
- CMD / BIZ 層への閾値バリデーション追加、BIZ の閾値読み出し挙動の変更（UI-11a-D3）

### 更新履歴

| 日付 | PR | 内容 |
|------|-----|------|
| 2026-09-15 | 表示小修正 batch 2 | owner 方針に基づき区画見出しを外し説明文のみ、入力エラーを「1〜99999の整数を入力してください」に統一。4種の保存拒否ルールは維持。§69.4 の画面構成を同期。 |
| 2026-08-03 | UI safety net implementation | 既存 isDirty を共通離脱ガードへ接続し、保存成功後の非 block を test で固定 |
| 2026-07-06 | - | UI-11a Design Phase 初版（UI-11a-D1〜D7） |
| 2026-07-07 | - | 実装反映の drift 修正: 実配置は `features/threshold-settings/`（`features/settings/` は存在せず、UI-11b 実体は `backup-restore/`）、フォームは既存パターン（useState + zod safeParse、RHF 不使用）。保存は最初の失敗 key で停止し「保存済み」表示を事実に限定 |
| 2026-07-29 | wave 4 plan-first | UI-11a-D8を追加。2 fieldのdescriptorをschema・抽出・保存の単一所有者とし、operator-visible挙動は不変 |
