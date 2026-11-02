// src/features/daily-sales/components/PrintUnsupportedNotice.tsx
//
// UI-09a-D18 Ctrl+P の案内。印刷では #root を隠す（globals.css）ので、「日報を印刷」を押さずに印刷すると
// 白紙になる。理由と直し方を印刷の紙に 1 行で出す（D-127）。#root の外（document.body の直下）へ portal で描き、
// 画面には出さず印刷のときだけ出す（hidden print:block）。日報の紙を描いている間は描かない。
// 設計: docs/function-design/56-ui-daily-sales.md UI-09a-D18

import { useSyncExternalStore } from "react";
import { createPortal } from "react-dom";

// ponytail: 紙を描く者は ExportBar の 1 つだけなので、module の 1 つの flag で持つ。
// 紙を描く画面が 2 つ以上になれば、描いている数を数える形にする。
let sheetPrinting = false;
const listeners = new Set<() => void>();

/// ExportBar が紙を描く前に true、印刷の後に false にする
export function setDailyReportSheetPrinting(printing: boolean) {
  if (sheetPrinting === printing) return;
  sheetPrinting = printing;
  for (const listener of listeners) listener();
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

function getSnapshot() {
  return sheetPrinting;
}

export function PrintUnsupportedNotice() {
  const printing = useSyncExternalStore(subscribe, getSnapshot);
  if (printing) return null;
  return createPortal(
    <p className="hidden p-4 text-[11pt] text-black print:block">
      この画面の印刷には対応していません。日報は日次売上の画面の「日報を印刷」から印刷してください。
    </p>,
    document.body,
  );
}
