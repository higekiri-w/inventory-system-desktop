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
  /** 手で選ぶ EJ の取込みの待ちの間は route の移動が止まるので、欄と前後のボタンを押せなくする（80 §80.7） */
  disabled: boolean;
  onChange: (newDate: string) => void;
}

export function EjDateNavigator({
  date,
  prevDate,
  nextDate,
  disabled,
  onChange,
}: EjDateNavigatorProps) {
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

  // 前後のボタンは表示中の日から 1 回だけ移り、打ちかけの下書きは捨てる。mousedown で欄の focus を外さず、
  // blur で打ちかけの日へ移ってから隣へ移る 2 回の移動（履歴に見ていない日が入る）を起こさない。
  // ponytail: WebView2 で mousedown の preventDefault が欄の blur を止めるかは L3 で見る。効かなければ
  // blur の側で、押されたのが前後のボタンか（relatedTarget）を見て確定を飛ばす形にする。
  const step = (to: string | null) => {
    if (to === null) return;
    setDraft(date);
    onChange(to);
  };

  return (
    <div className="flex flex-wrap items-center gap-2">
      <Button
        type="button"
        variant="outline"
        size="sm"
        disabled={disabled || prevDate === null}
        onMouseDown={(e) => {
          e.preventDefault();
        }}
        onClick={() => {
          step(prevDate);
        }}
      >
        前の日
      </Button>
      <span className="min-w-[10rem] text-center text-sm font-medium" aria-live="polite">
        {formatJpDate(date)}
      </span>
      <input
        type="date"
        disabled={disabled}
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
        disabled={disabled || nextDate === null}
        onMouseDown={(e) => {
          e.preventDefault();
        }}
        onClick={() => {
          step(nextDate);
        }}
      >
        次の日
      </Button>
    </div>
  );
}
