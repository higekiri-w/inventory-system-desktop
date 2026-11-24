// src/features/ej-receipts/components/EjDateNavigator.tsx
//
// UI-09c-D2: 前の日・次の日（記録のある隣の日へ飛ぶ。無ければ disabled）と日付の入力。
// 形は UI-09a の DateNavigator（56 §56.7）を写し、±1 日でなく応答の prev_date・next_date を使う。

import { Button } from "@/components/ui/button";
import { formatJpDate } from "@/features/daily-sales/lib/date-nav";

export interface EjDateNavigatorProps {
  date: string;
  prevDate: string | null;
  nextDate: string | null;
  onChange: (newDate: string) => void;
}

export function EjDateNavigator({ date, prevDate, nextDate, onChange }: EjDateNavigatorProps) {
  return (
    <div className="flex flex-wrap items-center gap-2">
      <Button
        type="button"
        variant="outline"
        size="sm"
        disabled={prevDate === null}
        onClick={() => {
          if (prevDate !== null) onChange(prevDate);
        }}
      >
        前の日
      </Button>
      <span className="min-w-[10rem] text-center text-sm font-medium" aria-live="polite">
        {formatJpDate(date)}
      </span>
      <input
        type="date"
        value={date}
        onChange={(e) => {
          if (/^\d{4}-\d{2}-\d{2}$/.test(e.target.value)) {
            onChange(e.target.value);
          }
        }}
        className="rounded-md border border-input bg-control-surface px-3 py-1 text-sm focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
        aria-label="日付を選択"
      />
      <Button
        type="button"
        variant="outline"
        size="sm"
        disabled={nextDate === null}
        onClick={() => {
          if (nextDate !== null) onChange(nextDate);
        }}
      >
        次の日
      </Button>
    </div>
  );
}
