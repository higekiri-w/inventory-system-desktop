> **親文書**: [FUNCTION_DESIGN.md](../FUNCTION_DESIGN.md)
> **入力ドキュメント**: ARCHITECTURE.md（UI-09b タスク仕様）、SCREEN_DESIGN.md（月次売上レポート画面）、screen_mockups.html（モックアップ、historical reference）、34-biz-sales-service.md（BIZ-05 get_monthly_sales）、42-cmd-sales-stocktake.md（CMD-09 get_monthly_sales / export_sales_csv）、56-ui-daily-sales.md（UI-09a 業務ロジックあり版テンプレ）

## 57. UI-09b: 月次売上レポート画面

> **2026-06-30 REQ-401 redesign note**: 月次売上画面は、Z001/Z002/Z005日報由来の公式部門集計と、Z004/手動販売由来の商品ランキングを分けて表示する。日報取込み済みでも商品ランキングが空の場合があるため、ランキング空を月間売上ゼロとして扱わない。

### 本書のテンプレ判定（業務ロジックあり版・簡潔版、共通 6 項目）

| 観点 | 本画面の選択 | 根拠 |
|---|---|---|
| useQuery 数 | **1 useQuery**（`get_monthly_sales` 単一呼出、`prev_month_comparison` field を派生表示） | BIZ-05 が当月 + 前月比較を 1 call で返す設計（Q-5、UI-09a 2 useQuery 機械的横展開は誤り） |
| URL state | TanStack Router `validateSearch`（zod 4 直接渡し、`.optional().catch(undefined)` で不正値吸収） | desktop-app-ui-constraints.md「状態の URL 化」、テスト容易 + F5 耐性 + queryKey 独立 |
| 派生純関数 | 6 個（summary / period-label / comparison / composition / pick-top-ranking / sort-items / format-month-label） | UI-09a 同型、業務ロジックを純関数に閉じる |
| factory | 2 種類（`makeMockMonthlyItem` / `makeMockProductRankingRow` + `makeMockDeptCompositionRow`） | テスト DRY、UI 派生型は別 factory（G-12） |
| ファイル分離 | `src/features/monthly-sales/` 内閉鎖（types / lib / hooks / components）+ Progress wrapper は `src/components/ui/` + TabsHeader は `src/components/sales/` 共通化。今日の日付だけは `src/features/daily-sales/lib/date-nav.ts` の `useTodayDate` を読む（UI-09b-D11） | UI-09a 同型 + UI-09b で共通化先を分離 |
| Sonner dedup | id-based（`export-${reportType}-success` / `export-${reportType}-error`） | 8-7 useExportFile 共通化先で reportType ラベル付き id（PR #66 で `-csv-` セグメント削除） |

---

### 57.1 概要

- **対応 REQ**: REQ-502（SP-502-01〜06）
- **対応 task**: UI-09b（[ARCHITECTURE.md §UI-09b](../architecture/ui-task-specs.md#ui-09a09b-売上レポート画面群)）
- **呼び出す CMD**: `get_monthly_sales(month, mode)` + `export_sales_csv(reportType, target)`（8-7 共通化先 `useExportFile` 経由）
- **route**: `/reports/monthly`（TabsHeader 共通化で `/reports/daily` ⇄ `/reports/monthly` を `<Link>` 切替、router-driven）
- **主動線**: TabsHeader「日次」←→「月次」切替、UI-00 ホームの「月次売上」ボタンから遷移
- **失敗 4 状態**（Q-5）:
  1. API fail → 画面全体 Alert（Skeleton 撤去）
  2. items 空配列 → 「当月データなし」EmptyState
  3. `prev_month_comparison === null` または `[]` 空配列 → サマリ「比較不可」+ 各行「—」。**BIZ contract**: 通常 BIZ-05 は前月データなしも `Some(空Vec)` を返す（[`34-biz-sales-service.md` §1](34-biz-sales-service.md) + `src-tauri/src/biz/sales_service.rs` の `get_monthly_sales` 実装、`prev_month_comparison = Some(prev_items)` 常時セット）、`null` は specta `Option<Vec<T>>` 境界の defensive guard。UI 側 (`compute-comparison.ts`) は `null` / `[]` 両ケースを safely 扱う
  4. 前月だけ fail → 構造的不可能（1 useQuery 設計、Q-5）

**REQ-401再設計後の表示領域**:

| 領域 | データソース | 表示意味 |
|---|---|---|
| 公式部門集計 | `MonthlySalesReport.official_department_totals`（daily_report_department_lines） | 「取り込んだ日」（`official_report_dates`、UI-09b-D11）に並ぶ日の Z005 部門別売上の合計 |
| 商品ランキング | `MonthlySalesReport.items` when mode=by_product（sale_records） | Z004商品別CSVまたは手動販売出庫から得た商品別ランキング |
| 商品別由来の部門集計 | `MonthlySalesReport.items` when mode=by_department（sale_records） | 商品別明細が存在する範囲の部門集計 |

`UI-09b-D8`: 公式部門集計と商品ランキングを同一の正本として混ぜない。Z005は部門別売上を持つが商品別JANを持たず、商品ランキングはZ004/手動販売出庫が根拠になるため。

`UI-09b-D10`: 同一営業日の複数active importは月次公式部門集計と商品別集計の各series内で加算する。日次追加分が月次合計へ反映され、per-import rollback後は対象分だけが除かれることを回帰契約とする。公式日報seriesと商品別seriesは互いに加算しない。月の中の取り込んだ日とその日数（UI-09b-D11）は、BIZ-05 の `official_report_dates`（[34 §19.4](34-biz-sales-service.md)。completed の取込みのある日の distinct report date）から作り、import の件数を使わない。同じ日の複数回の取込みは 1 日と数える。

`UI-09b-D11`（取り込んだ日と取り込んでいない日。2026-11-03 の起票、導入の前の関門 12 の前提 B、owner の決定 TD-578 の O3、[packet](../plans/2026-11-03-monthly-imported-days.md)）: 公式部門集計の節で、月の中のどの日を取り込んだか（取り込んでいない日はどれか）と、公式部門集計にどの日が入っているかを、使い手が文字で確かめられるようにする。

- 置き場所: 公式部門集計の節の説明の下、部門の表（か `公式部門集計の行はありません。`）の上。`official_department_totals === null` の月は今の note（`この月のレジ日報は未取込みです。`）だけで、日の一覧を出さない。空配列の月は日の一覧と `公式部門集計の行はありません。` の両方を出す。取得の失敗（§57.8 の #1）では日の一覧を出さない。
- 取り込んだ日: 見出し `取り込んだ日（{N}日分）` の後に、`official_report_dates` の日を `{D}日（{曜}）`（曜は `日`〜`土` の 1 字）の形で `、` でつないで日付の順に並べる。各日は日次売上のその日（`/reports/daily` の search `date` に `YYYY-MM-DD`）への link。`{N}` は `official_report_dates` の数。
- 取り込んでいない日: 見出し `取り込んでいない日（{M}日分）` の後に、その月の 1 日から月末までの日のうち `official_report_dates` に無い日を同じ形で並べる（link にしない）。0 日なら `なし`。この行に `店を開けなかった日も含みます。` を添える（アプリは休業日を知らない。§57.10 Q-1）。1 日以上あるときは、その行の後に link `売上データ取込みを開く`（`/csv-import` の search `tab` に `daily-report`。[56](56-ui-daily-sales.md) UI-09a-D17 と同じ語）を置く。
- 今日と今日より後の日: 「取り込んでいない日」には PC の今日（日次売上と同じ `useTodayDate`、local の `YYYY-MM-DD`）より前の日（前日まで）だけを並べ、今日と今日より後の日は入れない（ホームの `前日分が未取込みです` と同じ基準）。理由: 日報は営業の終わりの締めで取り込む運用で、今日を数えると締めの前は毎日今日が並び、直し方の link も出る。今日を取り込めば、今日は「取り込んだ日」に並ぶ（「取り込んだ日」は `official_report_dates` のとおりで、今日で切らない）。今月（表示中の月が今日の月と同じとき）だけ `今日と、今日より後の日は数えていません。` を添える。来月より先の月の「取り込んでいない日」は空で、この文も出さない（`compute-report-days` の `isCurrentMonth`、§57.6）。
- 日次から月次への戻り: 取り込んだ日の link で日次売上へ進んだ後、日次の画面の `TabsHeader`（Q-3 の共通部品）の `月次` の link は、日次で表示中の日（`search.date`、無ければ今日）の月を search `month` で渡し、その月の月次売上を開く。日次で日付の入力・前日・翌日で日を動かした後は、動かした後の日の月を渡す。月次から日次へ（`日次` の link）は今のまま（search を渡さない）。月次の `mode`・並べ替えは渡さず、月次の既定に戻る。理由: search の無い `月次` の link は今月を開き（§57.4 の `month` の fallback）、使い手が月を選び直すことになる（D-127）。棄却: 戻る操作をブラウザの履歴（Alt+←）に任せる（WebView2 での動きが未確認で、画面に戻り方が見えない）、月次の月を保存して覚える（状態の置き場が増える）。
- 説明文: 見出しの下の説明を `「取り込んだ日」に並ぶ日の Z005 部門別売上の合計です。` にする（前の `日報取込み済み日の Z005 部門別売上合計です。` を置き換える。同じものを 2 つの語で呼ばない）。
- 書き方: 状態は見出しの文字で示し、色・アイコンだけで分けない。一覧は表の行にしない（`<ul>`、名前は `取り込んだ日`・`取り込んでいない日`）。数は `{N}日分` と書く（`12日` は 12 日目と読める）。「営業日」「日平均」の語を使わない（§57.10 Q-1）。曜日は `new Date(year, month - 1, day)` の local で引き、`YYYY-MM-DD` を UTC として parse しない。
- 層の境: completed・distinct・月の範囲は IO/BIZ（[24 §14.22a](24-io-csv-import-repo.md)・34 §19.4）が持つ。UI は BIZ が返した日と暦から「取り込んでいない日」を作り、今日と今日より後の日を除くだけ（純関数 `compute-report-days`、§57.6）。UI で取込みの行を絞って日を作らない（`list_daily_report_imports` を使わない理由は 34 §19.4 の設計判断）。
- owner の答えの扱い: 表示の形・今日と今日より後の日・休業日の文・直し方の link・説明文の 5 点は packet の O1〜O5。owner の答え（2026-11-03、repo 外の回答台帳 TD-588）: O1〜O5 とも a。O2 は同じ日に owner が改めた（repo 外の回答台帳 TD-593: 今日も数えない、前日まで。AI の推奨〈今日までの日を並べる〉を計画の review の closure round 2 の指摘で改めた）。出所（選択肢と推奨）: 5 点とも packet の起票の AI の推奨で、O4（直し方の link）は AGENTS.md の「使い手に押しつけない」（D-127）の当てはめ。owner の観点（TD-578 の O3）は「どの日を取り込んだか（取り込んでいない日はどれか）が分かり、どの日が入っているかを画面で確かめられる」までで、形は決めていない。
- 棄却: 月の暦の表（曜日の並びは見やすいが、表の部品と code が増え、日の文字の一覧で観点を満たせる）、日数だけ（どの日かが分からない）、全部の日を数える（今月のまだ来ていない日を取込み忘れと並べる）。

`UI-09b-D9`（導入前受入）: 月次画面は日次履歴を束ねて参照する経路であり、紙バインダーの見え方をそのまま再現することを目的にしない。operatorが過去月・対象日へ迷わず到達でき、取込み済み日の範囲と修正後の値を信頼できるかをgo-live前に確認する。この受入は紙を置き換えない: 日報の紙を今の Excel の紙からアプリの紙（UI-09a-D17）へ切り替えるのは運用開始の後の照合（UI-09a-D19 の (2)、repo 外の回答台帳 TD-560）の後で、月の売上の紙は今の作り方を続ける（下の「今の月の帳票との照合」、TD-578 の O2 (ii)）。

受入の合格基準（2026-11-02 の起票。UI-09a-D13 の受入と同じ回に、同じ開発用の Windows と店の SD の写しで行う。[packet](../plans/2026-11-02-daily-monthly-acceptance.md) の owner の答え、repo 外の回答台帳 TD-578。手順は [受入の手順書](../daily-monthly-acceptance-runbook.md)）:

- 過去月・対象日への到達: 月の入力と前月・翌月で取り込んだ日のある月へ着き、`日次` の tab から対象日へ着く。
- 取込み済み日の範囲: 月の公式部門集計にどの日が入っているか（取り込んでいない日はどれか）を、月次の画面の取り込んだ日と取り込んでいない日の一覧（UI-09b-D11）で見分けられることを確かめる（TD-578 の O3）。UI-09a-D13 の backup/restore 後の再現の復元の後に、比べる月の `取り込んだ日` にその月に取り込んだ日（比べる日と、修正・再取込み後の表示で取り込み直した日）だけが並び、後で取り込んだ日が `取り込んでいない日` に並び、`取り込んだ日` の日の link から日次売上のその日へ着く。取り込んだ日の無い月には `この月のレジ日報は未取込みです。` だけが出る。
- 修正後の値: 公式部門集計の部門ごとの額が、その月に取り込んだ日（比べる日と、UI-09a-D13 の修正・再取込み後の表示で取り込み直した日。どちらも紙を借りた 1 回精算の日）の借りた綴りの紙の部門別集計の額の和と同じ（紙から電卓で出す。電卓の結果は紙に控えず、その場で画面の行と比べる）。製品の契約は「月次の公式部門集計 = その月に取り込んだ日の日次の `部門別集計（Z005）` の和」（UI-09b-D10）のままで、受入では日次を借りた紙と比べている（UI-09a-D13）ので、月次も同じ紙の和と比べて確かめる。比べるのは、UI-09a-D13 の backup/restore 後の再現の順（取込みの後に作った backup → 比べる月の、まだ取り込んでいない日の束を 1 つ取り込む → その backup を復元）の復元の後の 1 回だけで、この 1 回でこの項目と UI-09a-D13 の backup/restore 後の再現の月次の両方を判定する（復元の後の値は backup を作った時の値で、取り消して取り込み直した日の分が入り、後で取り込んだ日の分が入らない）。この判定は、2 回以上精算した日を取り込む前に済ませる（UI-09a-D13 の公式集計の順）。同じ日の 2 回の加算と、取消の後に対象分だけが除かれること（どちらも UI-09b-D10）は、この受入の月次では見ず、既存の自動 test（rolled_back の取込みを和から除くことも見る）が持つ。完了の画面を離れた後の取消で月次の値が取込みの前へ戻ることは、取消の入口の lane（UI-09a-D13 の修正・再取込み後の表示）の後に、受入の手順書の「前提の lane の後に足す」の行が確かめる。
- 今の月の帳票との照合: 店が今使う月の売上の紙（[project-memory](../project-memory.md) の「導入後も紙で要るもの」の「月の売上」）が部門ごとの月の合計を持つかは店の事実で未確認（次の訪店で確かめる、TD-578）。確かめるまでこの照合は行わない。店の事実が決まり、照合が当たるときだけ行う（比べる月と並べ方は、そのとき受入の手順書の行で決める）。行わない間は、受入の手順書の行に「行わない（O2 (ii) 未確認）」か「行わない（当たらない）」と書き、全体の合否に数えない（全体の合格は、前提の行が残らず、行う行が全部合ったときだけ。UI-09a-D13 の判定の段と同じ）。月次の画面は印刷を持たないので、月の売上の紙は今の作り方を続け、この受入で置き換えない。

#### REQ-401 第2スライス表示詳細

- `MonthlySalesPage` は既存の商品ランキング / 商品別由来の部門集計の前に「公式部門集計（レジ日報由来）」セクションを表示する。
- `official_department_totals === null` の場合は「この月のレジ日報は未取込みです。」の軽量 note を表示する。
- `official_department_totals` が空配列の場合は「公式部門集計の行はありません。」を表示し、対象月に日報親はあるが部門行がない状態と未取込み状態を分ける。
- `official_department_totals` が null でない月は、説明の下に取り込んだ日と取り込んでいない日の一覧を出す（UI-09b-D11）。
- 公式部門集計は `daily_report_department_lines` 由来、商品ランキング / 部門別構成比は `sale_records` 由来として、見出し・説明文・表を分ける。日報のみの月でも `items` は水増しせず、既存ランキング側は空のまま扱う。

### 57.2 ファイル構成（命名規約 + 型 export）

#### bindings 由来の型（specta 出力、PR #66 commit 2 で生成）

```ts
// src/lib/bindings.ts
export type SalesMode = "by_product" | "by_department";

export type SalesReportType =
  | "daily"
  | "monthly_by_product"
  | "monthly_by_department";

export type MonthlySaleItem = {
  key: string;            // by_product = product_code、by_department = department_id 文字列
  label: string;          // by_product = 商品名、by_department = 部門名
  count_points: number;   // 個数の種類の数量の和（共通規則 SPEC-UNIT-D3・D11、§57.6 の「単位と数量」）
  length_cm: number;      // 長さの種類の数量の和（cm）
  stock_unit: ProductStockUnit | null; // by_product の行だけ値。by_department は null
  amount: number;
  ranking: number;        // BIZ-05 row_number、1-based、同順位なし前提
};

export type MonthlySalesReport = {
  month: string;          // "YYYY-MM"
  mode: SalesMode;
  items: MonthlySaleItem[];
  prev_month_comparison: MonthlySaleItem[] | null;  // Option<Vec<T>> → null
  official_department_totals: OfficialMonthlyDepartmentTotal[] | null; // Z005 日報由来。日報未取込みの月は null（REQ-401 第2スライス）
  official_report_dates: string[]; // "YYYY-MM-DD" の昇順・重複なし。completed の取込みのある日（UI-09b-D10・D11）
};
```

**重要**: `prev_month_comparison` は `MonthlySaleItem[] | null`（TypeScript で `null`、`undefined` ではない）。Rust 側 `Option<Vec<T>>` の specta 出力規約。UI 派生で `null` ガード必須（失敗 4 状態 #3）。

#### features/monthly-sales/

| 配置 | ファイル | 責務 | 規模 |
|---|---|---|---|
| 新規 | `src/routes/reports/monthly.tsx` | route + `validateSearch`（zod 4） | 40-60 |
| 新規 | `src/features/monthly-sales/MonthlySalesPage.tsx` | 最上位 page、Alert / EmptyState / Skeleton 出し分け（失敗 4 状態） | 80-110 |
| 新規 | `src/features/monthly-sales/types.ts` | `SortColumn` / `SortDirection` / `SalesViewMode` / `ProductRankingRow` / `DeptCompositionRow` | 30-45 |
| 新規 | `src/features/monthly-sales/lib/compute-summary.ts` | total quantity / amount 集計（純関数） | 20-30 |
| 新規 | `src/features/monthly-sales/lib/compute-period-label.ts` | 「YYYY/MM/DD-MM/DD」固定文言生成（Q-1 B 案、F-10 文字列操作 + G-9 不正月 fallback） | 30-45 |
| 新規 | `src/features/monthly-sales/lib/compute-comparison.ts` | key 突合 + `prev_amount <= 0` ガード（Q-7、Z004 返品超過月対策） | 50-70 |
| 新規 | `src/features/monthly-sales/lib/compute-composition.ts` | 部門別構成比 %（`grand_total === 0` ガード含む） | 30-45 |
| 新規 | `src/features/monthly-sales/lib/pick-top-ranking.ts` | 上位 10 抽出（BIZ-05 row_number 同順位なし前提） | 20-30 |
| 新規 | `src/features/monthly-sales/lib/sort-items.ts` | 列ソート + null 末尾配置（4 列 + ranking バッジ追従） | 40-60 |
| 新規 | `src/features/monthly-sales/lib/format-month-label.ts` | `formatMonthLabel("2026-01")` → `"2026年1月"`（H-3、zero-pad なし）、`prevMonth` / `nextMonth` 戻り値のみ `padStart(2, "0")` | 45-65 |
| 新規 | `src/features/monthly-sales/lib/test-fixtures.ts` | `makeMockMonthlyItem` + `makeMockProductRankingRow` / `makeMockDeptCompositionRow` factory（G-12） | 30-45 |
| 新規 | `src/features/monthly-sales/hooks/useMonthlySalesReport.ts` | 1 useQuery + 派生 6 純関数 orchestration + `derived` 生成 | 80-120 |
| 新規 | `src/features/monthly-sales/components/MonthNavigator.tsx` | 前月 button + `<input type="month">` + 翌月 button | 60-80 |
| 新規 | `src/features/monthly-sales/components/ModeTabs.tsx` | mode 切替（`?mode=by_product\|by_department`）。`SegmentedControl` で商品別ランキング / 部門別構成比の二択を描画し、active tone は shared stone selection tone | 40-55 |
| 新規 | `src/features/monthly-sales/components/SummaryCardsBar.tsx` | 4 カード（売上合計 / 販売点数 / 期間表示 / 前月比） | 100-140 |
| 新規 | `src/features/monthly-sales/components/DepartmentTable.tsx` | shadcn Table 5 列（部門・数量・売上・構成比・前月比）+ `<Progress>` 構成比バー + 前月比色分け（Q-4 商品数列は `MonthlySaleItem` DTO 不在で非対応、Plans.md Backlog 参照） | 110-150 |
| 新規 | `src/features/monthly-sales/components/ProductRankingTable.tsx` | 上位 10 + 1 位は琥珀の pill と行の地（`item.ranking === 1` 追従、G-3） | 90-120 |
| 新規 | `src/features/monthly-sales/components/ExportBar.tsx` | CSV 出力 button（active）+ 印刷 button（aria-disabled + Tooltip） | 50-70 |

#### components/ui + components/sales（共通化）

| 配置 | ファイル | 責務 | 規模 |
|---|---|---|---|
| 新規 | `src/components/ui/progress.tsx` | shadcn `<Progress>` wrapper（`import { Progress as ProgressPrimitive } from "radix-ui"` 統合パッケージ慣習、G-2） | 25-35 |
| 移送 | `src/components/sales/TabsHeader.tsx` | UI-09a 由来、router-driven `<Link>` 視覚表現、daily/monthly 共通使用（Q-3）。active tone は shared stone selection tone | 45-60 |
| 共通 | `src/components/sales/SortableHeader.tsx` | `SortableHeader<T extends string>`。DepartmentTable 3列 / ProductRankingTable 4列と日次ProductTable 5列のheader DOM・ARIA・indicatorを一元所有（UI-TABLE-D1） | feature固有型をimportしない |

#### lib/hooks（8-7 共通化）

| 配置 | ファイル | 責務 | 規模 |
|---|---|---|---|
| 新規 | `src/lib/hooks/useExportFile.ts` | `useMutation` + Blob ダウンロード共通化、`SalesReportType` を bindings から import（H-5 drift 耐性）、try/catch + Sonner id reportType ラベル化（G-4） | 80-110 |
| 更新 | `src/features/daily-sales/hooks/useExportDailySalesCsv.ts` | wrapper 化（`useExportFile({ reportType: "daily" })` 経由、20-30 行に縮退） | 20-30 |

#### 更新枠（既存 file）

| 配置 | ファイル | 責務 |
|---|---|---|
| 更新 | `src/lib/query-keys.ts` | `monthlySales(month, mode)` helper 追加 |
| 更新 | `src/config/navigation.ts` | UI-09b ブロック `to` / `status` 更新 |
| 更新 | `src/routes/reports/daily.tsx` | TabsHeader import 先変更（`features/daily-sales/components/` → `components/sales/`） |

#### 削除枠

| 配置 | ファイル | 理由 |
|---|---|---|
| 削除 | `src/features/daily-sales/components/TabsHeader.tsx` | `components/sales/` に移送、daily 専用版は廃止（Q-3） |

合計: 新規 20 + 移送 1 + 更新 5 + 削除 1 = **27 file** + Vitest 7 = **34 file 実体**。frontend code 1500-1800 行 + 関数設計 500-600 行 + test 320-400 行。

#### UI-09b-D11 で足した file

上の合計は PR #66 の時点の数で、この表の file を含めない。

| 配置 | ファイル | 責務 |
|---|---|---|
| 新規 | `src/features/monthly-sales/lib/compute-report-days.ts` | `computeReportDays(month, importedDates, today)`: 取り込んだ日・取り込んでいない日（前日まで）・今月か（§57.6） |
| 新規 | `src/features/monthly-sales/components/OfficialReportDays.tsx` | 取り込んだ日と取り込んでいない日の 2 行・日次への link・直し方の link（UI-09b-D11） |
| 更新 | `src/features/monthly-sales/MonthlySalesPage.tsx` | 公式部門集計の節に `OfficialReportDays` を置き、説明文を UI-09b-D11 の文にする。今日は `src/features/daily-sales/lib/date-nav.ts` の `useTodayDate` |
| 更新 | `src/components/sales/TabsHeader.tsx` | 任意の prop `month`（`YYYY-MM`）。渡されたときだけ `月次` の link に search `month` を付ける（UI-09b-D11 の日次から月次への戻り）。active の判定は変えない |
| 更新 | `src/features/daily-sales/DailySalesPage.tsx` | `TabsHeader` に表示中の日の月を渡す（56 §56.1 の `TabsHeader` の行） |
| 更新 | `src/features/daily-sales/lib/date-nav.ts` | 冒頭の comment の共通化の条件を今の判断（使い手 2 つでは移さず、3 つめで `src/lib/dates/` へ）に置き換える。code は変えない |

### 57.3 データフロー

```
URL search params (month, mode, sortBy, sortDir)
  ↓ validateSearch + zod 4 fallback
useMonthlySalesReport({ month, mode, sortBy, sortDir })
  ↓ useQuery(queryKeys.monthlySales(month, mode), () => commands.getMonthlySales(month, mode))
MonthlySalesReport DTO (month / mode / items / prev_month_comparison: T[] | null / official_department_totals / official_report_dates)
  ↓ 派生 6 純関数 orchestration（official_report_dates は page が useTodayDate と compute-report-days で日の一覧にする、UI-09b-D11）
{ query, derived: { summary, periodLabel, ranking, composition, comparisonMap } | null }
  ↓
MonthlySalesPage（Alert / EmptyState / Skeleton 出し分け）
  ↓ render
SummaryCardsBar + ModeTabs + DepartmentTable | ProductRankingTable + ExportBar
```

**派生 6 純関数の責務分担**:
1. `compute-summary` → `{ totalAmount, totalCountPoints, totalLengthCm }`（数量は種類ごとの 2 本。安全な整数を外れた欄は null）
2. `compute-period-label` → `"YYYY/MM/DD-MM/DD"` 固定文言（month の月初〜月末を派生、Q-1 B 案）
3. `compute-comparison` → `Map<key, { prevAmount, diff, ratio | null, isComparable: boolean }>`（key 突合 + `prev_amount <= 0` 比較不可ガード）
4. `compute-composition` → `{ key, label, count_points, length_cm, amount, ratio }[]`（部門別構成比、`grand_total === 0` ガード）
5. `pick-top-ranking` → `MonthlySaleItem[]` 上位 10（BIZ-05 row_number 順、同順位なし前提）
6. `sort-items` → 列ソート + null 末尾配置（4 列 + `item.ranking === 1` バッジ追従）。**sort 適用先: `ranking` + `composition` 双方** (raw `items` 非適用)。理由は (a) `ranking` 順位列の `ranking` field 保持で badge 追従 (G-3、`src/features/monthly-sales/lib/sort-items.test.ts` の「G-3: ranking field 残存確認」で検証済)、(b) `composition` 部門別 sort 軸は ranking と独立 (利用者は順位 sort と部門集計 sort を別操作)。UI-09a (フラット → グルーピング) vs UI-09b (multi-derived) の派生木差異に対応する非対称設計

### 57.4 URL state 設計（zod 4 直接渡し）

```ts
// src/routes/reports/monthly.tsx
import { z } from "zod";

const searchSchema = z.object({
  month: z
    .string()
    .regex(/^\d{4}-\d{2}$/)
    .optional()
    .catch(undefined),
  mode: z
    .enum(["by_product", "by_department"])
    .optional()
    .catch(undefined),
  sortBy: z
    .enum(["name", "quantity", "amount", "prev_month_diff"])
    .optional()
    .catch(undefined),
  sortDir: z.enum(["asc", "desc"]).optional().catch(undefined),
});
```

**enum 契約化（D-061）**: `mode` は bindings 由来の `SalesMode` generated union を利用する。値・fallback 挙動は不変。

- `month` undefined → 当月 fallback（`useCurrentMonth` hook）
- `mode` undefined → `"by_product"` fallback
- 不正値（例: `?month=2026-13`、`?mode=invalid`）→ zod `.catch(undefined)` で吸収、ユーザに通知せず黙って fallback（UI-09a 同方針）
- BIZ-05 `validate_month` が月番号 1-12 + 年範囲を保証（G-9）
- `sortBy` enum 4 値のうち `"quantity"` は ProductRankingTable と DepartmentTable の両方で UI 露出する（数量の列。構成比列はソート対象外）。数量の並べ替えは両 mode とも `(count_points, length_cm)` の辞書順（共通規則 SPEC-UNIT-D3 の集計の規則）で、`?sortBy=quantity&mode=by_department` もこの順で並ぶ

### 57.5 hook 設計

#### useMonthlySalesReport（1 useQuery + 派生 6 純関数）

```ts
export function useMonthlySalesReport(params: {
  month: string;
  mode: SalesMode;
  sortBy: SortColumn | null;
  sortDir: SortDirection;
}) {
  const query = useQuery({
    queryKey: queryKeys.monthlySales(params.month, params.mode),
    queryFn: () => unwrapResult(commands.getMonthlySales(params.month, params.mode), {
      source: "commands",
      cmd: "get_monthly_sales",
    }),
    staleTime: 60_000,    // 1 min（売上は数十秒単位で動かない）
    gcTime: 5 * 60_000,
    retry: 1,
  });

  const derived = useMemo(() => {
    if (!query.data) return null;
    const { items, prev_month_comparison: prev } = query.data;
    const summary = computeSummary(items);
    const periodLabel = computePeriodLabel(params.month);
    const comparisonMap = computeComparison(items, prev);  // prev: T[] | null OK
    // ranking / composition 双方に sort 適用 (raw items 非適用、§57.3 非対称根拠参照)
    const rankingRaw = pickTopRanking(items);
    const compositionRaw = computeComposition(items);
    const ranking = sortMonthlyItems(rankingRaw, params.sortBy, params.sortDir);
    const composition = sortMonthlyItems(compositionRaw, params.sortBy, params.sortDir);
    return { summary, periodLabel, comparisonMap, ranking, composition };
    // deps に sortBy/sortDir を含める: URL state 変更時の派生 memo 再計算担保、
    // primitive 個別追加で object reference equality trade-off を構造的に回避
  }, [query.data, params.month, params.sortBy, params.sortDir]);

  return { query, derived };
}
```

本 query を stale 化する mutation 集合は [D-052](../decision-log.md) と `src/lib/invalidation-contract.ts` を正本とし、本書へ producer 一覧を複製しない。`monthlySalesRoot()` は月・mode 別 key の prefix factory とする。

`official_report_dates`（UI-09b-D11）は同じ query の data に乗り、公式部門集計と同じ invalidate・refetch で変わる（別の query を足さない）。hook の `derived` には入れず、page が `computeReportDays` を呼ぶ（今日の日付を hook の memo の依存に足さないため）。

#### useExportFile（8-7 共通化、`SalesReportType` 引数化）

```ts
import type { SalesReportType, SalesExportResponse } from "@/lib/bindings";

export interface ExportFileArgs {
  reportType: SalesReportType;   // bindings 由来の literal union（H-5）
  target: string;
}

const REPORT_LABEL: Record<SalesReportType, string> = {
  daily: "日次売上",
  monthly_by_product: "月次売上（商品別）",
  monthly_by_department: "月次売上（部門別）",
};

export function useExportFile() {
  const mutation = useMutation<SalesExportResponse, Error, ExportFileArgs>({
    mutationFn: ({ reportType, target }) =>
      unwrapResult(commands.exportSalesCsv(reportType, target), {
        source: "commands",
        cmd: "export_sales_csv",
      }),
    onSuccess: (data, args) => {
      try {                          // G-4 try/catch + private 移送
        downloadBlobFromBase64(data);
        toast.success(`${REPORT_LABEL[args.reportType]} を保存しました（${data.record_count} 件）`, {
          id: `export-${args.reportType}-success`,
        });
      } catch (e) {
        const message = e instanceof Error ? e.message : String(e);
        toast.error(`出力に失敗しました: ${message}`, {
          id: `export-${args.reportType}-error`,
        });
      }
    },
    onError: (error, args) =>
      toast.error(`出力に失敗しました: ${error.message}`, {
        id: `export-${args.reportType}-error`,
      }),
  });
  return { exportFile: mutation.mutate, isExporting: mutation.isPending };
}
```

- `downloadBlobFromBase64` は private 関数（hook 内閉鎖）、UI-09a `useExportDailySalesCsv` から移送
- Sonner id `export-${reportType}-success/error` で reportType ごとに dedup（連続クリック時の重複 toast 0）

#### useCurrentMonth（monthly-sales 内閉鎖）

`useMemo(() => formatYearMonth(new Date()), [])`。JST 当月の `"YYYY-MM"` を返す（UI-00 `useYesterdayDate` 同型）。

### 57.6 純関数（テスト対象 6 + factory 2 + format-month-label 1 = 9 個）

#### compute-summary（[items] → { totalAmount, totalCountPoints, totalLengthCm }）

- 単位と数量（D-113、owner の決定 TD-203。[共通規則](10-common-rules.md) SPEC-UNIT-D3）: `MonthlySaleItem` の数量は `count_points`（個数の種類の和）と `length_cm`（長さの種類の和、cm）の 2 つ。月間販売点数（`compute-summary` は 2 本それぞれに足す）・部門別は `5 点・1.8 m` の形で出す（`N 点` の `N` は 3 桁区切りで `1,234 点`。owner の決定 TD-206・TD-207）。商品別の行はどちらか一方だけが値を持ち、表示は行の `stock_unit` で決める（長さの商品〈`m`〉は `1.3 m`、相殺して 0 なら `0 m`。個数の商品は `3 点`）。`compute-summary` の和が `Number.isSafeInteger` を外れたら `—`。数量の並べ替えは SPEC-UNIT-D3 の集計の規則の `(count_points, length_cm)` の順。

- 空配列 → `{ totalAmount: 0, totalCountPoints: 0, totalLengthCm: 0 }`
- 負数 amount（返品超過月）→ そのまま合計（純関数、業務判断は呼出側）

#### compute-period-label（[month: "YYYY-MM"] → string）
- F-10 文字列操作: `month.split("-")` → 数値分解
- 月末日数: `new Date(year, month, 0).getDate()`（JS Date は month 1-based の前月末日）
- 出力: `"2026/03/01-03/31"`（slash 区切り、月-日 zero-pad あり、固定文言、Q-1 B 案）
- G-9 不正月 fallback: 月番号 13 / 00 等は `"—"` 返却、throw new Error しない

#### compute-comparison（[currentItems, prevItems | null] → Map<key, ComparisonInfo>）
- `prevItems === null` → 全件 `isComparable: false`（失敗 4 状態 #3）
- `prevItems !== null` → key 突合（current.key === prev.key で match）
- **Q-7 ガード**（user P2 反映、Z004 返品超過月対策）:
  - `prev.amount === 0` → `isComparable: false`（除算ガード）
  - `prev.amount < 0` → `isComparable: false`（色分け逆転回避、業務上比較困難）
  - それ以外: `ratio = (current.amount - prev.amount) / prev.amount`
- 戻り値: `{ prevAmount, diff, ratio, isComparable }` per key

#### compute-composition（[items] → CompositionRow[]）
- `grandTotal = sum(items.amount)`、`grandTotal === 0` → 全件 `ratio: 0`
- 各 row `ratio = item.amount / grandTotal`、`<Progress value={ratio * 100} />` で描画

#### pick-top-ranking（[items] → MonthlySaleItem[]）
- BIZ-05 が `ranking` field を 1-based row_number で返す（同順位なし前提）
- UI 側は `items.filter((x) => x.ranking <= 10).sort((a, b) => a.ranking - b.ranking)`

#### sort-items（[rows, column, direction] → rows[]）
- 4 列: `name` / `quantity` / `amount` / `prev_month_diff`
- null 末尾配置（`prev_month_diff: number | null` の場合）
- `item.ranking === 1` は ProductRankingTable 側で `<Badge>` 強調（sort で順序が変わっても追従、G-3）

#### format-month-label（H-3 UI 表示「2026年1月」）
- `formatMonthLabel("2026-01")` → `"2026年1月"`（zero-pad なし、モックアップ + UI-09a Intl 出力準拠）
- `formatMonthLabel("2026-12")` → `"2026年12月"`
- `prevMonth("2026-01")` → `"2025-12"`（ISO 再構築のみ `padStart(2, "0")` 適用、年またぎ）
- `nextMonth("2025-12")` → `"2026-01"`（年またぎ）
- G-9 不正月: `formatMonthLabel("2026-13")` → `"—"`

#### makeMockMonthlyItem / makeMockProductRankingRow / makeMockDeptCompositionRow（factory 2 種類、G-12）
- `makeMockMonthlyItem(overrides: Partial<MonthlySaleItem>): MonthlySaleItem`
- `makeMockProductRankingRow(overrides: Partial<ProductRankingRow>): ProductRankingRow`
- `makeMockDeptCompositionRow(overrides: Partial<DeptCompositionRow>): DeptCompositionRow`
- DRY 原則: DTO 5 field + UI 派生 row（`prev_month_diff?: number | null` 含む）

#### compute-report-days（[month, importedDates, today] → { imported, notImported, isCurrentMonth }。UI-09b-D11）
- 月の日を 1 日から月末まで作る（月末は compute-period-label と同じ `new Date(year, month, 0).getDate()`）。`importedDates` のうちその月の日を `imported`、残りを `notImported` にする。月の外の日は数えない
- `notImported` は `today`（`YYYY-MM-DD`）より前の日（`date < today`、前日まで）だけ。今日と今日より後の日は入れない（UI-09b-D11 の「今日と今日より後の日」、owner の決定 TD-593）。来月より先の月は `notImported` が空
- `imported` は今日・今日より後の日でも `importedDates` のとおり入れる（今日を取り込めば今日が並ぶ。今日での切り捨ては `notImported` だけに当てる）
- `isCurrentMonth` は `month` が `today` の月（`today` の先頭 7 字）と同じときだけ true。`今日と、今日より後の日は数えていません。` はこれが true のときだけ出す（日を除いたかどうかで決めない。来月より先の月は全部の日を除くが、文は出さない）
- 各日は `{ date: "YYYY-MM-DD", label: "{D}日（{曜}）" }`。曜は `new Date(year, month - 1, day).getDay()` の local（UTC の parse をしない）
- completed・distinct は BIZ-05 が保証するので、ここで status や重複の規則を写さない
- G-9 不正月: 空の結果を返し、throw しない

### 57.7 UI コンポーネント

#### MonthlySalesPage（最上位、Alert / EmptyState / Skeleton 出し分け）

```tsx
function MonthlySalesPage() {
  const { month, mode } = Route.useSearch();
  const navigate = Route.useNavigate();
  const monthValue = month ?? useCurrentMonth();
  const modeValue = mode ?? "by_product";
  const { query, derived } = useMonthlySalesReport({ month: monthValue, mode: modeValue });

  return (
    <div className="...">
      <TabsHeader />                              {/* 共通 */}
      <MonthNavigator month={monthValue} onChange={(m) => navigate({ search: { month: m } })} />
      {query.isLoading && <Skeleton />}
      {query.isError && <Alert variant="destructive">取得に失敗しました</Alert>}
      {query.data && (
        <>
          <SummaryCardsBar summary={derived!.summary} periodLabel={derived!.periodLabel} prevComparison={query.data.prev_month_comparison} />
          <ModeTabs mode={modeValue} onChange={(m) => navigate({ search: { mode: m } })} />
          {query.data.items.length === 0 ? (
            <EmptyState message="当月データなし" />
          ) : modeValue === "by_department" ? (
            <DepartmentTable rows={derived!.composition} comparisonMap={derived!.comparisonMap} />
          ) : (
            <ProductRankingTable rows={derived!.ranking} comparisonMap={derived!.comparisonMap} />
          )}
          <ExportBar reportType={modeValue === "by_department" ? "monthly_by_department" : "monthly_by_product"} target={monthValue} />
        </>
      )}
    </div>
  );
}
```

#### MonthNavigator
- 前月 button → `prevMonth(month)` 戻り値で `navigate({ search: { month: prevMonth(month) } })`
- `<input type="month" value={month} onChange={...}>` → HTML5 native picker（F-11 Windows native 標準動作）
- 翌月 button → 同様。**未来月もガードなし**（business: 当月以後を選択可能、月途中状態を見たい場合あり）

#### ModeTabs
- `SegmentedControl` 2 値（商品別ランキング / 部門別構成比）、`?mode` URL state に反映
- active tone は日次/月次 TabsHeader と同じ shared segmented control tone を使い、同一画面内の「今見ている view」を一貫して示す。二択切替の list / item / active / inactive visual は `src/components/ui/segmented-control.tsx` を参照し、active border は押しボタン状に見えない `border-stone-300` とする
- 選択状態は色だけに依存せず、`aria-pressed` と `data-state=active|inactive` を出す。商品別から部門別、部門別から商品別へ直接切り替えられることを `ModeTabs.test.tsx` で検証する

#### SummaryCardsBar（4 カード）
- カード 1: 月間売上合計（`summary.totalAmount` 円表記）
- カード 2: 月間販売点数（`summary.totalCountPoints`・`summary.totalLengthCm` を `5 点・1.8 m` の形で。範囲外は `—`）
- カード 3: 期間表示「YYYY/MM/DD-MM/DD」固定文言（Q-1 B 案、`periodLabel`）
- カード 4: 前月比（`prev_month_comparison === null` または `[]` 空配列 → 「比較不可」灰、通常 BIZ contract は `Some(空Vec)` で空配列ケース、`null` は specta `Option<Vec<T>>` 境界の defensive guard）
- **カード溢れ対策（PR-3 (f)）**: `Card` は `flex flex-col` のため grid item の `min-width:auto` で縮まず、長い金額・期間ラベル（「2026/05/01-05/31」等）で `lg:grid-cols-4` のカードが溢れる（B2/B3、CSS 起因で論理バグでない）。`Card` + `CardContent` 両方に `min-w-0` を付与し（CardContent は flex child なので min-w-0 が確定で必須）、value div を `truncate` する。日次版 `daily-sales/components/SummaryCardsBar.tsx`（SimpleCard + CardWithTooltip）も同方針

#### DepartmentTable（5 列 + Progress バー、Q-4 商品数非対応）
- 列: 部門 / 数量（`5 点・1.8 m`）/ 売上 / 構成比（数値 + `<Progress>` バー）/ 前月比（商品数列は `MonthlySaleItem` DTO に `product_count` field 不在のため非対応、Plans.md Backlog 参照）
- **SortableHeader 適用列 (4 列、構成比列はソート対象外)**: 共通owner `src/components/sales/SortableHeader.tsx`（UI-TABLE-D1）を使い、部門名 (`name`) / 数量 (`quantity`) / 売上 (`amount`) / 前月比 (`prev_month_diff`)を結線する。click で `onSortChange(column)` 発火 → MonthlySalesPage の `handleSortChange` が `sortBy/sortDir` 切替 (同列再 click → desc toggle / 別列 → asc) を `onSearchChange` 経由で URL state 更新
- 前月比色分け（F-15 ±1.0% 閾値 + Q-7 prev <= 0 ガード。地の chip を持たない役割色の文字 + 記号、`text-sm font-medium`。design-system 00「役割色の文字だけの表示」、decision-log D-094）:
  - 緑（`text-success-strong`）: `ratio >= 0.01`
  - 灰（`text-muted-foreground`）: `-0.01 < ratio < 0.01`
  - 赤（`text-destructive-strong`）: `ratio <= -0.01`
  - **「—」灰**: `isComparable: false`（`prev === null` / `prev_amount = 0` / `prev_amount < 0`）

#### ProductRankingTable（上位 10 + 1 位の琥珀の pill）
- 列: 順位（`ranking`）/ 商品名 / 数量 / 金額 / 前月比 / -
- `item.ranking === 1` → 「1 位」を `<Badge className="border-warning bg-rank-top-badge-bg text-rank-top-badge-text hover:bg-rank-top-badge-bg">` の琥珀の pill で示し、行に `bg-rank-top-bg/40` の地を付ける（2 位以下は `text-sm text-muted-foreground` の文字。sort で順序変わっても `ranking === 1` 追従、G-3。design-system 00 の迷いやすい場面「ランキング 1 位」、decision-log D-094。順位と太字だけにする試しは L3 で採らなかった）
- **SortableHeader 適用列 (4 列、順位列はソート対象外)**: 共通owner `src/components/sales/SortableHeader.tsx`（UI-TABLE-D1）を使い、商品名 (`name`) / 数量 (`quantity`) / 金額 (`amount`) / 前月比 (`prev_month_diff`)を結線する。click で `onSortChange(column)` 発火 → MonthlySalesPage の `handleSortChange` が `sortBy/sortDir` 切替 (同列再 click → desc toggle / 別列 → asc) を `onSearchChange` 経由で URL state 更新

#### ExportBar
- CSV 出力 button: `exportFile({ reportType: "monthly_by_product" | "monthly_by_department", target: month })`、`isExporting === true` で button disabled + spinner
- 印刷 button: aria-disabled + Tooltip「準備中」 + onClick `preventDefault` + `cursor-not-allowed opacity-60`（memory `feedback-radix-tooltip-aria-disabled.md` 3 層パターン）

### 57.8 エラー処理（失敗 4 状態の網羅）

| 状態 | 描画 | 復旧手段 |
|---|---|---|
| **#1 API fail**（`query.isError`） | 画面全体 `<Alert variant="destructive">` + 「再試行」button（`query.refetch()`） | 再試行 button or ページ再読込 |
| **#2 items 空配列** | `<EmptyState message="当月データなし" />` + 「月を変える」案内 | MonthNavigator で別月選択 |
| **#3 prev_month_comparison === null または `[]`** | サマリ前月比カード「比較不可」灰、テーブル前月比列「—」 | 業務上正常（BIZ-05 は前月データなしも `Some(空Vec)` を返す = `src-tauri/src/biz/sales_service.rs` の `get_monthly_sales` で常時セット、`null` は specta `Option<Vec<T>>` 境界の defensive guard）、復旧不要 |
| **#4 前月だけ fail** | 構造的不可能（1 useQuery 設計、BIZ-05 が 1 call で両方返す、Q-5） | N/A |

- CSV 出力 fail（`useExportFile` `onError`）: Sonner toast.error、画面状態は維持
- 印刷 button click: aria-disabled でクリック不可、Tooltip で「準備中」案内のみ

### 57.9 テスト戦略（Vitest 純関数 42-55 ケース、7 file）

| file | ケース数 | 主内容 |
|---|---|---|
| `compute-summary.test.ts` | 4-5 | 空 / 1 件 / 複数 / 負数（返品月） / 大量（500 件） |
| `compute-period-label.test.ts` | 6-8 | 1 月（31 日） / 2 月閏年 2024（29 日） / 2 月非閏年 2025（28 日） / 12 月 / 月番号 13（「—」） / 00（「—」） / 2000 年（閏） / 2100 年（非閏） |
| `compute-comparison.test.ts` | 8-10 | 当月>前月 / <前月 / 同 / 前月 0 / 前月 null（全件） / +1000% / **Q-7: prev = 0 → 比較不可 / prev < 0（例 -500）→ 比較不可 / prev < 0 + current < 0 → 比較不可** |
| `compute-composition.test.ts` | 5-7 | 空 / 100%（1 件） / 合計 100% / 端数（合計 99.99%） / grand_total = 0 → 全件 ratio 0 |
| `pick-top-ranking.test.ts` | 5-7 | 空 / 5 件（全部 top10）/ 10 件 / 15 件（top10 抽出） / BIZ row_number 同順位なし前提 / ranking=11 除外 |
| `sort-items.test.ts` | 6-8 | 空 / 4 列ソート（name/quantity/amount/prev_month_diff）/ 同値（安定ソート保証）/ null 末尾 / **G-3: ranking バッジ追従テスト**（sort 後も `ranking === 1` 識別可能） |
| `format-month-label.test.ts` | 10-12 | 月またぎ（1→2） / 年またぎ（12→翌年 1）/ 不正値 13/00 → 「—」 / **H-3: `formatMonthLabel("2026-01")` → `"2026年1月"` zero-pad なし** / `"2026年12月"` / UTC/JST 同結果担保 / 閏年 2024-02 → "2024年2月" |

合計テストケース **44-57 ケース**（plan §7 表 42-55 に整合）。

UI-09b-D11 で足した test（ケースの正本は [packet](../plans/2026-11-03-monthly-imported-days.md) の Test Plan）: `compute-report-days.test.ts`（過去の月・今月の今日と今日より後の除外〈前日まで〉・今日を取り込んだ日が「取り込んだ日」に並ぶ・来月より先の月〈`isCurrentMonth` が false〉・閏年と月の長さ・曜日の local・不正月と月の外の日）、`OfficialReportDays.test.tsx`（見出しの数・日の link・`なし`・直し方の link・今月だけの文〈過去の月と来月より先の月では出ない〉）、`MonthlySalesPage.test.tsx`（null の月に一覧が無い、空配列の月に一覧が在る、invalidate の後の refetch、取得の失敗と再試行、月の切替え）、`src/components/sales/TabsHeader.test.tsx`（`month` を渡すと `月次` の link に search `month`、渡さないと今の href）、`src/features/daily-sales/DailySalesPage.test.tsx`（表示中の日の月を `TabsHeader` に渡す。前日・翌日・日付の入力で月が変わった後も追う）。

### 57.10 業務ルール

#### Q-1 営業日数の扱い（B 案採用）
- BIZ-05 が営業日数 / 日平均を返さないため、サマリカード #3 は「期間: YYYY/MM/DD-MM/DD」**固定文言**で代替（Q-1 B 案、user 確定）
- 「営業日数」「日平均」の語は本画面で**使わない**（誤解防止）
- 公式部門集計の `取り込んだ日（{N}日分）`（UI-09b-D11）は日報を取り込んだ日の数で、営業日数ではない（店を開けた日をアプリは知らない）。この規則のまま「営業日」の語を使わない
- 将来 BIZ-05 拡張で営業日数追加する場合は別 PR（Plans.md Backlog）

#### Q-4 部門情報の非対応
- `MonthlySaleItem` DTO に部門情報（`department_id` / `department_name`）が**含まれない**（BIZ-05 設計）
- → 部門フィルタ Select **非対応**（UI で派生不可能、UI → CMD → BIZ → IO 境界違反回避）
- → 商品ランキングテーブルに「部門」列**なし**（同上）
- 将来 BIZ-05 拡張で部門情報追加する場合は別 PR（Plans.md Backlog）

#### Q-7 prev_amount <= 0 ガード（Z004 返品超過月対策）
- Z004 売上 CSV は返品超過月で `amount < 0` になりうる（部門合計が月またぎ返品でマイナス）
- `(current - prev) / prev` で色分けすると、prev < 0 の場合に符号が逆転して誤誘導
- → `prev_amount <= 0`（0 含む）で「比較不可」灰「—」表示、業務上の異常値を symbolic に示唆

#### 前月比色分け閾値 ±1.0%（F-15）
- 緑: `ratio >= 0.01`（+1.0% 以上）
- 灰: `-0.01 < ratio < 0.01`（±1.0% 範囲内、誤差扱い）
- 赤: `ratio <= -0.01`（-1.0% 以下）
- 「比較不可」灰: `isComparable: false`（Q-7 ガード + null）

### 57.11 ショートカット

- グローバル `Ctrl+/`: ShortcutsDialog（UI-shortcuts で実装済、本画面では追加 hook 不要）
- 画面固有ショートカット: 本 Phase では未定義（次フェーズで判定）

### 57.12 表記揺れ + UI 表示フォーマット（H-3 + G-9）

| 系統 | 表示 | 内部 |
|---|---|---|
| 月ラベル UI 表示 | **「2026年1月」**（zero-pad なし、H-3） | ISO `"2026-01"` |
| 月ラベル不正月 | 「—」（G-9 fallback） | throw しない |
| ISO 再構築（prevMonth/nextMonth 戻り値） | `"2025-12"` / `"2026-01"`（`padStart(2, "0")`） | - |
| 期間表示（Q-1 B 案） | 「2026/03/01-03/31」（slash + 月日 zero-pad あり） | - |
| 前月比「比較不可」 | 「—」灰 | `isComparable: false` |
| 取り込んだ日・取り込んでいない日の日（UI-09b-D11） | 「23日（月）」（日は zero-pad なし、曜は全角の括弧） | `YYYY-MM-DD` |
| 取り込んだ日の数（UI-09b-D11） | 「2日分」 | `official_report_dates.length` |
| 金額 | 「¥1,234」 | `Intl.NumberFormat` |
| 数量 | 「12 点」 | integer |
| 構成比 | 「45.2%」 | `Math.round(ratio * 1000) / 10` |

**page h1**: 「月次売上」（PR-1 で mockup の「売上レポート」から分離、§56.12 と同方針）。日次/月次は TabsHeader で同居するため、h1 が「売上レポート」だと L3 デモで判別不能だった（U4）。サイドバー label・navigation.title とも「月次売上」で一致。

### 57.13 非目的（IO/BIZ 同型表）

| やらないこと | 理由 | 責務を持つモジュール |
|---|---|---|
| 月選択カスタム picker 実装 | HTML5 `<input type="month">` で十分（F-11 Windows native 標準）、非 IT 利用者向け minimal build（memory `feedback-non-it-user-feature-minimal-build.md`） | shadcn DatePicker 採用は将来判定（Plans.md Backlog） |
| chart library 導入（Chart.js / Recharts 等） | shadcn `<Progress>` バーで構成比は十分視覚化、月次推移グラフ等は Phase 3 以降の判定 | 将来別 PR |
| 営業日数 / 日平均算出 | BIZ-05 DTO に存在しない、UI で派生すると業務不整合（祝日マスタ等が必要） | BIZ-05 拡張別 PR（Q-1 Backlog） |
| 部門フィルタ Select | MonthlySaleItem DTO に部門情報不在、UI で派生不可能（DTO 境界違反） | BIZ-05 拡張別 PR（Q-4 Backlog） |
| 印刷機能本実装 | 仕様未定（Q40 障害時対応と合わせて Phase 4 で具体化、Plans.md Backlog） | UI-13 整合性検証画面と合わせて判定 |
| Sonner id 後方互換 wrapper | 内部 dedup key の実態整合性を優先（外部 API ではない）、memory `feedback-naming-must-match-reality.md` 系判断軸 | 廃止 |
| bindings trim 自動化 | `npm run format` の `.prettierignore` 除外設計を尊重、手動 `sed` または `prettier --ignore-path /dev/null` で対処（F-5） | scripts/pre-push.sh ④ で env-safety 同様 baseline 確認のみ |

### 更新履歴

| 日付 | PR | 内容 |
|---|---|---|
| 2026-05-19 | #66 | 新規作成（UI-09b 月次売上レポート、1 useQuery + 派生 6 純関数 + 失敗 4 状態 + 8-7 useExportFile 共通化 + TabsHeader 共通化 + Progress wrapper + SalesReportType specta 化 / Plan rally 6 round 累積 53 件発見 converged） |
| 2026-05-22 | PR-1 (tone/nav fix) | page h1 を「売上レポート」→「月次売上」に分離（U4、§57.12）。TabsHeader の `activeOptions.includeSearch:false` で search params 付き URL の active 維持（B1） |
| 2026-05-22 | PR-3 (tone/nav fix) | SummaryCardsBar カード溢れ対策（§57.7）: Card + CardContent に `min-w-0` + value div `truncate`（B2/B3、CSS 起因）。日次版も同方針 |
| 2026-06-08 | selection-tone follow-up | TabsHeader と ModeTabs の active tone を shared stone selection tone に統一し、Sidebar / StatusChips と同じ選択状態の視覚言語へ寄せた。日次/月次と商品別ランキング/部門別構成比の二択切替は `SegmentedControl` primitive を共有する |
| 2026-07-29 | 監査是正 順21a plan-first | UI-TABLE-D1として月次Department 3列 / Ranking 4列のheader implementation ownerを`src/components/sales/SortableHeader.tsx`へ正本化。列集合・sort callback・URL state・表示は不変 |
| 2026-08-16 | PR #79 | SPEC-SDI-D6: 同日複数active importの月次加算、per-import取消後の減算、series分離、将来coverageのdistinct-date契約を正本化。 |
| 2026-10-08 | 単位の拡張 design lane | §57.6 compute-summary に単位の拡張の後（proposed・未実装、D-113 / SPEC-UNIT-D3、owner の決定 TD-203）を追加: `count_points`・`length_cm` の 2 本の集計（TD-206・TD-207）、商品別の m。 |
| 2026-11-02 | 日報・月次の画面の導入前受入（起票） | UI-09b-D9 に受入の合格基準を足した（owner の決定 TD-570 で関門 12、packet の答え TD-578）。月の中の取り込んだ日を画面に出す lane を受入の前提と書き、月の売上の紙はこの受入で置き換えないと書いた。 |
| 2026-11-03 | 日報・月次の画面の導入前受入（計画の review round 1 の是正） | UI-09b-D9 の本文の「日次画面と合わせた受入が終わるまで、Excel印刷・バインダーを完全代替したとは扱わない」を、この受入が日報の紙と月の売上の紙のどちらも置き換えないと書く文に置き換えた（受入の後に完全代替と読めたため）。修正後の値の復元の文を UI-09a-D13 の順にそろえ、月の帳票との照合から AI が足した「窓の中に 1 か月の全部の日がそろう月」の条件を外した（TD-578 の「当たるときだけ」に戻した）。前提の lane が受入の手順書の行を書き足す義務と、前提の行が残る版では全体の判定が未了であることを足した。 |
| 2026-11-03 | 日報・月次の画面の導入前受入（計画の review の closure round 2 の是正） | 今の月の帳票との照合を行わない間は「行わない（理由）」と書いて全体の合否に数えないことと、全体の合格の規則（UI-09a-D13 の判定の段と同じ）を書いた。修正後の値の復元の後の値を、取り込んだ日の日次の部門別集計の和で確かめると書いた。 |
| 2026-11-03 | 日報・月次の画面の導入前受入（実装） | UI-09b-D9 の受入の段に受入の手順書（`docs/daily-monthly-acceptance-runbook.md`）への link を足した。修正後の値の backup/restore の順の足す日を「比べる月の、まだ取り込んでいない日」にそろえた（UI-09a-D13 と同じ字。計画の review の closure round 3 の P3）。 |
| 2026-11-03 | 月次の画面に取り込んだ日を出す（起票、関門 12 の前提 B） | UI-09b-D11（未実装）を足した: 公式部門集計の節に取り込んだ日と取り込んでいない日を文字で出す（owner の決定 TD-578 の O3。表示の形は packet の O1〜O5 で、owner の答え〈2026-11-03、repo 外の回答台帳 TD-588〉: O1〜O5 とも a）。UI-09b-D10 の「将来「取込み済み日数」を表示する場合は」を `official_report_dates` を使う今の契約に置き換えた。§57.2 の `MonthlySalesReport` の snippet に欠けていた `official_department_totals` と新しい `official_report_dates` を足した。計画の review round 1 の是正で、UI-09b-D11 に日次から月次への戻り（日次の `TabsHeader` の `月次` の link が表示中の日の月を search `month` で渡す）を足し、§57.6 の今日より後の文の条件を `isCurrentMonth`（今月だけ）にした。closure round 2 の是正で、owner の決定 TD-593（TD-588 の O2 を改める）により今月の「取り込んでいない日」を前日までにし（今日も数えない。§57.6 の境を `date < today` に）、今月の文を `今日と、今日より後の日は数えていません。` にした。冒頭の表の「monthly-sales 内閉鎖」に、今日の日付だけは daily-sales の `useTodayDate` を読むことを足した。 |
| 2026-11-04 | 月次の画面に取り込んだ日を出す（実装、関門 12 の前提 B） | UI-09b-D11・§57.2 の `official_report_dates` と足した file の表・§57.6 `compute-report-days`・§57.9 の test を実装し、「未実装」の注記を外した（[packet](../plans/2026-11-03-monthly-imported-days.md)）。 |
| 2026-11-11 | 日報・月次の画面の導入前受入（PR #229 の Gated Amendment 1、owner の決定 TD-650） | UI-09b-D9 の修正後の値の確かめ方を、その月に取り込んだ日の日次の部門別集計の和から、借りた綴りの紙の部門別集計の和（紙から電卓で出す）に替えた（画面の値を運ばせないため。製品の契約 UI-09b-D10 は変えない）。判定は紙を借りた 1 回精算の 2 日だけを取り込んだ状態で行い、同じ日の 2 回の加算は受入の月次で見ず既存の自動 test が持つ、と書いた。紙の和と比べるのは UI-09a-D13 の backup/restore 後の再現の復元の後の 1 回で、両方の項目を判定する（復元の後の日次は 2 日とも紙と比べる、UI-09a-D13）。 |
| 2026-11-11 | 月次の画面に取り込んだ日を出す（Final Review の是正） | REQ-401 再設計後の表示領域の表の「公式部門集計」の表示意味を UI-09b-D11 の説明文の語にした（[packet](../plans/2026-11-03-monthly-imported-days.md)）。 |
| 2026-11-12 | 日報・月次の画面の導入前受入（PR #229 の Gated Amendment 1 の closure の P3） | UI-09b-D9 の修正後の値の最後の文を置き換え、この受入の月次で見ないもの（同じ日の 2 回の加算と、取消の後に対象分だけが除かれること）と、それを持つ所（既存の自動 test。完了の画面を離れた後の取消の月次の戻りは、取消の入口の lane の後に受入の手順書へ足す行）を書いた。 |
| 2026-11-13 | 月次の画面に取り込んだ日を出す（PR #229 の merge の後の書き足し、AC5） | UI-09b-D9 の受入の段の取込み済み日の範囲の項を、UI-09b-D11 の画面で確かめる文に置き換えた（前提が満ちたので、今の画面が日を出さないこと・前提にすること・受入の手順書の行を書き足す義務・その行が残る版の未了の文を外した。全体の判定の規則は今の月の帳票との照合の項と 56 UI-09a-D13 が持つ）。受入の手順書の「月: 取込み済み日の範囲」の行を同じ画面の語で書き足した（[packet](../plans/2026-11-03-monthly-imported-days.md)）。 |
