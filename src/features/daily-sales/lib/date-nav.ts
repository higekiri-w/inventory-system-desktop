// src/features/daily-sales/lib/date-nav.ts
//
// 日付ナビ純関数 + useTodayDate hook。
// useTodayDate は UI-00 useYesterdayDate と同型（Visibility API listener、24:00 またぎ再計算）。
// useTodayDate の使い手は日次売上と月次売上（57 UI-09b-D11）の 2 つで、2 つでは移さない。
// 3 つめの使い手ができたら src/lib/dates/ へ移す（56 §56.5）。
// 設計: docs/function-design/56-ui-daily-sales.md §56.5 + §56.6 + §56.7

import { useEffect, useState } from "react";

function computeToday(): string {
  const d = new Date();
  // Swedish locale が ISO 8601 互換 "YYYY-MM-DD" を返す
  return d.toLocaleDateString("sv-SE");
}

/// JST 今日の YYYY-MM-DD を返す。
/// Visibility API listener が `visible` 検知時に再計算 → setState で queryKey
/// を変化させて TanStack Query が自動再 fetch する。
export function useTodayDate(): string {
  const [date, setDate] = useState<string>(() => computeToday());

  useEffect(() => {
    const handleVisibilityChange = () => {
      if (document.visibilityState === "visible") {
        setDate(computeToday());
      }
    };
    document.addEventListener("visibilitychange", handleVisibilityChange);
    return () => {
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, []);

  return date;
}

/// YYYY-MM-DD 形式の日付に N 日加算した文字列を返す。
/// 月またぎ / 年またぎ / 閏年 2/29 を Date オブジェクトの自動繰り上がりで処理。
/// 不正な date 入力（NaN）は呼び出し側で防御する想定、ここではそのまま invalid date を返す。
export function addDays(date: string, days: number): string {
  const d = new Date(`${date}T00:00:00`);
  d.setDate(d.getDate() + days);
  return d.toLocaleDateString("sv-SE");
}

/// 日本語表記の日付ラベル（例: "2026年5月17日（日）"）。
export function formatJpDate(date: string): string {
  const d = new Date(`${date}T00:00:00`);
  if (Number.isNaN(d.getTime())) return date;
  const formatter = new Intl.DateTimeFormat("ja-JP", {
    year: "numeric",
    month: "long",
    day: "numeric",
    weekday: "short",
  });
  return formatter.format(d);
}
