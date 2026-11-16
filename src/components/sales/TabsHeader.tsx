// src/components/sales/TabsHeader.tsx
//
// 売上レポート 2 画面 (UI-09a 日次 / UI-09b 月次) で共通使用する Tabs ヘッダ。
// router-driven `<Link>` で /reports/daily ⇄ /reports/monthly を切替、active 表現は URL 由来。
// activeOptions.includeSearch:false: 日次/月次は date/month 等の search params を持つため、
// TanStack デフォルト includeSearch:true だと search 不一致で active が外れる。path 一致のみで判定する。
// month: 日次の画面が表示中の日の月（YYYY-MM）を渡すと、月次の link がその月を開く（57 UI-09b-D11）。
// 設計: docs/function-design/57-ui-monthly-sales.md §57.2 (Q-3 共通化 + label「日次/月次」)

import { Link } from "@tanstack/react-router";

import {
  segmentedControlActiveClass,
  segmentedControlInactiveClass,
  segmentedControlItemClass,
  segmentedControlListClass,
} from "@/components/ui/segmented-control";

export function TabsHeader({ month }: { month?: string }) {
  return (
    <nav aria-label="売上レポート切替" className={segmentedControlListClass}>
      <Link
        to="/reports/daily"
        activeOptions={{ exact: true, includeSearch: false }}
        className={segmentedControlItemClass}
        activeProps={{ className: segmentedControlActiveClass }}
        inactiveProps={{ className: segmentedControlInactiveClass }}
      >
        日次
      </Link>
      <Link
        to="/reports/monthly"
        search={month === undefined ? undefined : { month }}
        activeOptions={{ exact: true, includeSearch: false }}
        className={segmentedControlItemClass}
        activeProps={{ className: segmentedControlActiveClass }}
        inactiveProps={{ className: segmentedControlInactiveClass }}
      >
        月次
      </Link>
    </nav>
  );
}
