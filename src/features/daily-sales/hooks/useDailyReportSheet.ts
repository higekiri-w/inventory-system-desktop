// src/features/daily-sales/hooks/useDailyReportSheet.ts
//
// UI-09a-D17 日報の紙の内容。日次売上を開いたときに日付ごとに読み、印刷ボタンを押したときに読み直す。
// 設計: docs/function-design/56-ui-daily-sales.md UI-09a-D17、34 §19.3a

import { useQuery } from "@tanstack/react-query";
import { commands } from "@/lib/bindings";
import { unwrapResult } from "@/lib/invoke";
import { queryKeys } from "@/lib/query-keys";

export function useDailyReportSheet(date: string) {
  return useQuery({
    queryKey: queryKeys.dailyReportSheet(date),
    queryFn: () =>
      unwrapResult(commands.getDailyReportSheet(date), {
        source: "commands",
        cmd: "get_daily_report_sheet",
      }),
  });
}
