// src/features/monthly-sales/components/OfficialReportDays.tsx
//
// 公式部門集計の節の、取り込んだ日と取り込んでいない日の 2 行（UI-09b-D11）。
// 状態は見出しの文字で示し、色・アイコンだけで分けない。一覧は表の行にしない（<ul>）。
// 設計: docs/function-design/57-ui-monthly-sales.md UI-09b-D11

import { Link } from "@tanstack/react-router";

import type { ReportDay, ReportDays } from "../lib/compute-report-days";

const linkClass = "font-medium text-primary underline underline-offset-4";

function DayList({
  name,
  days,
  linkToDaily,
}: {
  name: string;
  days: ReportDay[];
  linkToDaily: boolean;
}) {
  if (days.length === 0) return <p>なし</p>;
  return (
    <ul aria-label={name} className="leading-relaxed">
      {days.map((day, index) => (
        <li key={day.date} className="inline">
          {linkToDaily ? (
            <Link to="/reports/daily" search={{ date: day.date }} className={linkClass}>
              {day.label}
            </Link>
          ) : (
            day.label
          )}
          {index < days.length - 1 ? "、" : ""}
        </li>
      ))}
    </ul>
  );
}

export function OfficialReportDays({ days }: { days: ReportDays }) {
  return (
    <div className="space-y-3 text-sm">
      <div className="space-y-1">
        <h3 className="font-semibold">{`取り込んだ日（${String(days.imported.length)}日分）`}</h3>
        <DayList name="取り込んだ日" days={days.imported} linkToDaily />
      </div>
      <div className="space-y-1">
        <h3 className="font-semibold">
          {`取り込んでいない日（${String(days.notImported.length)}日分）`}
        </h3>
        <DayList name="取り込んでいない日" days={days.notImported} linkToDaily={false} />
        <p className="text-muted-foreground">店を開けなかった日も含みます。</p>
        {days.isCurrentMonth && (
          <p className="text-muted-foreground">今日と、今日より後の日は数えていません。</p>
        )}
        {days.notImported.length > 0 && (
          <p>
            <Link to="/csv-import" search={{ tab: "daily-report" }} className={linkClass}>
              売上データ取込みを開く
            </Link>
          </p>
        )}
      </div>
    </div>
  );
}
