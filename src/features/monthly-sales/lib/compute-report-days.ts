// src/features/monthly-sales/lib/compute-report-days.ts
//
// 月の中の取り込んだ日と取り込んでいない日（UI-09b-D11）。
// completed・distinct は BIZ-05 の official_report_dates が保証するので、ここでは暦の補集合と
// 今日と今日より後の日の除外だけを行う。
// 設計: docs/function-design/57-ui-monthly-sales.md §57.6 compute-report-days

const WEEKDAYS = ["日", "月", "火", "水", "木", "金", "土"] as const;

export interface ReportDay {
  /** "YYYY-MM-DD" */
  date: string;
  /** "23日（月）" */
  label: string;
}

export interface ReportDays {
  imported: ReportDay[];
  notImported: ReportDay[];
  isCurrentMonth: boolean;
}

const EMPTY: ReportDays = { imported: [], notImported: [], isCurrentMonth: false };

/// month: "YYYY-MM"、importedDates: BIZ の official_report_dates、today: local の "YYYY-MM-DD"。
/// 不正な月は空の結果（throw しない、G-9）。
export function computeReportDays(
  month: string,
  importedDates: readonly string[],
  today: string,
): ReportDays {
  if (!/^\d{4}-\d{2}$/.test(month)) return EMPTY;
  const year = Number(month.slice(0, 4));
  const monthNum = Number(month.slice(5, 7));
  if (monthNum < 1 || monthNum > 12) return EMPTY;

  // compute-period-label と同じ月末の求め方（翌月の 0 日 = 当月末日）
  const lastDay = new Date(year, monthNum, 0).getDate();
  const importedSet = new Set(importedDates);
  const imported: ReportDay[] = [];
  const notImported: ReportDay[] = [];
  for (let day = 1; day <= lastDay; day += 1) {
    const date = `${month}-${String(day).padStart(2, "0")}`;
    // 曜日は local の Date で引く（"YYYY-MM-DD" を UTC として parse しない）
    const label = `${String(day)}日（${WEEKDAYS[new Date(year, monthNum - 1, day).getDay()]}）`;
    if (importedSet.has(date)) {
      imported.push({ date, label });
    } else if (date < today) {
      // 前日まで。今日と今日より後の日は取り込んでいない日に入れない（TD-593）
      notImported.push({ date, label });
    }
  }
  return { imported, notImported, isCurrentMonth: month === today.slice(0, 7) };
}
