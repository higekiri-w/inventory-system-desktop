// src/features/ej-receipts/components/EjDateNavigator.tsx
//
// UI-09c-D2: 前の日・次の日（記録のある隣の日へ飛ぶ。無ければ disabled）と日付の入力。
// 形は UI-09a の DateNavigator（56 §56.7）を写し、±1 日でなく応答の prev_date・next_date を使う。
// 日付の欄は打っている間は日を移さず、Enter か欄から出たときに移す（80 §80.1。WebView2 は日の 2 桁目を
// 待つ間も change を出し、そこで移って描き直すと打ちかけが捨てられる）。

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { formatJpDate } from "@/features/daily-sales/lib/date-nav";

export interface EjDateNavigatorProps {
  date: string;
  prevDate: string | null;
  nextDate: string | null;
  onChange: (newDate: string) => void;
}

export function EjDateNavigator({ date, prevDate, nextDate, onChange }: EjDateNavigatorProps) {
  const [draft, setDraft] = useState(date);
  // 最後に移った日。Enter の後の blur のように、同じ値で 2 回移らない
  const [sent, setSent] = useState(date);
  const [shownDate, setShownDate] = useState(date);
  // 表示中の日が外から替わった（前の日・次の日・link・戻る）ら、下書きをその日に合わせ直す
  if (date !== shownDate) {
    setShownDate(date);
    setDraft(date);
    setSent(date);
  }

  // ponytail: カレンダーの窓からの選択もキーの打ち込みも同じ change で届き、確かに見分ける手が無いので、
  // どちらも Enter か欄から出たときに移る。WebView2 で選択を見分ける確かな手が分かったら選択だけすぐ移す。
  const commit = () => {
    if (!/^\d{4}-\d{2}-\d{2}$/.test(draft)) {
      setDraft(date);
      return;
    }
    if (draft === sent) return;
    setSent(draft);
    onChange(draft);
  };

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
        value={draft}
        onChange={(e) => {
          setDraft(e.target.value);
        }}
        onKeyDown={(e) => {
          if (e.key === "Enter") commit();
        }}
        onBlur={commit}
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
