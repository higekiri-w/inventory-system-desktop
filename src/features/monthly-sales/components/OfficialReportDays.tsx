// src/features/monthly-sales/components/OfficialReportDays.tsx
//
// 公式部門集計の節の、取り込んだ日と取り込んでいない日の 2 行（UI-09b-D11）。
// 状態は見出しの文字で示し、色・アイコンだけで分けない。一覧は表の行にしない（<ul>）。
// 見出しと日の一覧は本文の大きさ、補足の文と `なし` だけ caption（design-system 04 原則 1・00）。
// 設計: docs/function-design/57-ui-monthly-sales.md UI-09b-D11

import { Link } from "@tanstack/react-router";

import type { ReportDay, ReportDays } from "../lib/compute-report-days";

const linkClass = "font-medium text-primary underline underline-offset-4";
const noteClass = "text-sm text-muted-foreground";

function DayList({
  name,
  days,
  linkToDaily,
}: {
  name: string;
  days: ReportDay[];
  linkToDaily: boolean;
}) {
  if (days.length === 0) return <p className={noteClass}>なし</p>;
  return (
    <ul aria-label={name} className="leading-relaxed">
      {days.map((day, index) => (
        // label の中（`23日（月）`）では折り返さず、項目の間（`、` の後）で折り返す
        <li key={day.date} className="inline whitespace-nowrap">
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
    <div className="space-y-3">
      <div className="space-y-1">
        <h3 className="font-semibold">{`取り込んだ日（${String(days.imported.length)}日分）`}</h3>
        <DayList name="取り込んだ日" days={days.imported} linkToDaily />
      </div>
      <div className="space-y-1">
        <h3 className="font-semibold">
          {`取り込んでいない日（${String(days.notImported.length)}日分）`}
        </h3>
        <DayList name="取り込んでいない日" days={days.notImported} linkToDaily={false} />
        <p className={noteClass}>店を開けなかった日も含みます。</p>
        {days.isCurrentMonth && (
          <p className={noteClass}>今日と、今日より後の日は数えていません。</p>
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
