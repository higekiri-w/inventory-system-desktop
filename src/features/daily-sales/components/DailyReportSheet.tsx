// src/features/daily-sales/components/DailyReportSheet.tsx
//
// UI-09a-D17 日報の紙（1 日 1 枚）。BIZ-05 の get_daily_report_sheet をそのまま描き、UI で足し算・
// 並べ替え・絞り込みをしない。画面には出さず印刷のときだけ出す（hidden print:block、UI-09a-D18）。
// 文字は 9 pt 以上、見出しは 11 pt の太字、色を使わない。
// 設計: docs/function-design/56-ui-daily-sales.md UI-09a-D17、34 §19.3a

import type { ReactNode } from "react";
import { createPortal } from "react-dom";

import { formatStockDisplay } from "@/features/stock-inquiry/lib/format-stock-display";
import type {
  DailyReportSheet as Sheet,
  DailyReportSheetRow,
  DailyReportSourceKind,
} from "@/lib/bindings";

/// 見出しの区切りの全角空白（U+3000、56 UI-09a-D17）
const SP = "\u3000";

const WEEKDAYS = ["日", "月", "火", "水", "木", "金", "土"] as const;

/// `2026年3月21日（土）`（括弧は全角）
export function formatSheetDate(date: string): string {
  const [year, month, day] = date.split("-").map(Number);
  const weekday = WEEKDAYS[new Date(year, month - 1, day).getDay()];
  return `${String(year)}年${String(month)}月${String(day)}日（${weekday}）`;
}

/// `2026-03-21 18:05`（押した時刻）
export function formatPrintedAt(printedAt: Date): string {
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${String(printedAt.getFullYear())}-${pad(printedAt.getMonth() + 1)}-${pad(printedAt.getDate())} ${pad(printedAt.getHours())}:${pad(printedAt.getMinutes())}`;
}

const LABEL_CHANGED_NOTICE =
  "この日の精算の間で、帳票の行の名称か並びが変わっています。同じ名称の行が 2 行あれば、別々に足しています。";
const UNSUMMED_NOTICE =
  "この日の精算の間で、帳票の行の名称か並びが変わり、1 回の精算の中に同じ名称の行が 2 行以上あります。別の項目を足さないよう、この帳票は足さずに取込みごとに載せています。";

/// 帳票ごとの知らせ（SALES-SHEET-D2 の 2・3。3 の帳票には 3 の文だけ）
export function sourceNotice(sheet: Sheet, source: DailyReportSourceKind): string | null {
  if (sheet.unsummed_sources.includes(source)) return UNSUMMED_NOTICE;
  if (sheet.label_changed_sources.includes(source)) return LABEL_CHANGED_NOTICE;
  return null;
}

export const SOURCE_TITLES: Record<DailyReportSourceKind, string> = {
  Z001: "日計（Z001）",
  Z002: "支払集計（Z002）",
  Z005: "部門別集計（Z005）",
};

function formatNumber(value: number | null): string {
  return value === null ? "—" : value.toLocaleString("ja-JP");
}

const cell = "border border-black px-1 py-0.5";
const numberCell = `${cell} text-right tabular-nums`;

function SheetTable({
  heading,
  columns,
  notice,
  children,
}: {
  heading: string;
  columns: [string, string, string];
  notice: string | null;
  children: ReactNode;
}) {
  return (
    <table className="w-full border-collapse">
      <thead>
        <tr>
          <th colSpan={3} className={`${cell} text-left text-[11pt] font-bold`}>
            {heading}
          </th>
        </tr>
        <tr>
          <th className={`${cell} text-left font-normal`}>{columns[0]}</th>
          <th className={`${cell} text-right font-normal`}>{columns[1]}</th>
          <th className={`${cell} text-right font-normal`}>{columns[2]}</th>
        </tr>
      </thead>
      <tbody>
        {notice !== null && (
          <tr className="break-inside-avoid">
            <td colSpan={3} className={cell}>
              {notice}
            </td>
          </tr>
        )}
        {children}
      </tbody>
    </table>
  );
}

function ReportRows({ rows }: { rows: DailyReportSheetRow[] }) {
  return rows.map((row, index) => (
    <tr key={index} className="break-inside-avoid">
      <td className={cell}>{row.label === "" ? "（名前なし）" : row.label}</td>
      <td className={numberCell}>{formatNumber(row.quantity ?? row.count)}</td>
      <td className={numberCell}>{formatNumber(row.amount)}</td>
    </tr>
  ));
}

/// 1 つの帳票。足さなかった帳票（unsummed）は import_seq ごとに表を分ける
function SourceTables({
  sheet,
  source,
  rows,
  dateLabel,
}: {
  sheet: Sheet;
  source: DailyReportSourceKind;
  rows: DailyReportSheetRow[];
  dateLabel: string;
}) {
  const title = SOURCE_TITLES[source];
  const notice = sourceNotice(sheet, source);
  const columns: [string, string, string] = ["名称", "個数/件数", "金額"];
  if (!sheet.unsummed_sources.includes(source)) {
    return (
      <SheetTable heading={`${title}${SP}${dateLabel}`} columns={columns} notice={notice}>
        <ReportRows rows={rows} />
      </SheetTable>
    );
  }
  const seqs = [...new Set(rows.map((row) => row.import_seq ?? 0))];
  return seqs.map((seq, index) => (
    <SheetTable
      key={seq}
      heading={`${title}${SP}${String(seq)}回目の取込み${SP}${dateLabel}`}
      columns={columns}
      notice={index === 0 ? notice : null}
    >
      <ReportRows rows={rows.filter((row) => (row.import_seq ?? 0) === seq)} />
    </SheetTable>
  ));
}

function PluSection({ sheet, dateLabel }: { sheet: Sheet; dateLabel: string }) {
  const heading = `PLU の売上（Z004）${SP}${dateLabel}`;
  const skipped =
    sheet.plu_import.skipped_count >= 1
      ? `数えていない PLU の行が ${sheet.plu_import.skipped_count.toLocaleString("ja-JP")} 行あります（入出庫履歴の CSV取込みの記録で見られます）。`
      : null;
  const empty =
    sheet.plu_import.import_count === 0
      ? "この日の PLU の売上は取り込まれていません。"
      : sheet.plu_rows.length === 0
        ? "この日に売れた PLU の商品はありません。"
        : null;
  if (empty !== null) {
    return (
      <div className="break-inside-avoid">
        <p className="text-[11pt] font-bold">{heading}</p>
        <p>{empty}</p>
        {skipped !== null && <p>{skipped}</p>}
      </div>
    );
  }
  return (
    <SheetTable heading={heading} columns={["商品名", "数量", "金額"]} notice={skipped}>
      {sheet.plu_rows.map((item) => (
        <tr key={`${item.product_code}-${item.source}`} className="break-inside-avoid">
          <td className={cell}>{item.name}</td>
          <td className={numberCell}>{formatStockDisplay(item.quantity, item.stock_unit)}</td>
          <td className={numberCell}>{item.amount.toLocaleString("ja-JP")}</td>
        </tr>
      ))}
    </SheetTable>
  );
}

export interface DailyReportSheetProps {
  sheet: Sheet;
  printedAt: Date;
}

export function DailyReportSheet({ sheet, printedAt }: DailyReportSheetProps) {
  const dateLabel = formatSheetDate(sheet.date);
  const count = sheet.import_count.toLocaleString("ja-JP");
  const combined =
    sheet.import_count >= 2
      ? sheet.unsummed_sources.length > 0
        ? `${count}回の取込みを合算（取込みごとに載せた帳票を除く）`
        : `${count}回の取込みを合算`
      : null;

  return createPortal(
    <div
      data-testid="daily-report-sheet"
      className="daily-report-sheet hidden space-y-3 bg-white text-[9pt] text-black print:block"
    >
      <header className="flex items-start justify-between gap-4">
        <div>
          <p className="text-[11pt] font-bold">売上日報</p>
          <p>{dateLabel}</p>
          {combined !== null && <p>{combined}</p>}
        </div>
        <p>印刷 {formatPrintedAt(printedAt)}</p>
      </header>
      <div className="grid grid-cols-2 items-start gap-3">
        <div className="space-y-3">
          <SourceTables
            sheet={sheet}
            source="Z001"
            rows={sheet.summary_rows}
            dateLabel={dateLabel}
          />
          <SourceTables
            sheet={sheet}
            source="Z005"
            rows={sheet.department_rows}
            dateLabel={dateLabel}
          />
        </div>
        <div className="space-y-3">
          <SourceTables
            sheet={sheet}
            source="Z002"
            rows={sheet.payment_rows}
            dateLabel={dateLabel}
          />
        </div>
      </div>
      <PluSection sheet={sheet} dateLabel={dateLabel} />
    </div>,
    document.body,
  );
}
