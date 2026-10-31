// src/features/daily-sales/components/DailyReportSheet.test.tsx
//
// UI-09a-D17 日報の紙。期待の文字列は 56 UI-09a-D17 の紙の並び・表・知らせの表から転記する。

import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import type { DailyReportSheet as Sheet, DailyReportSheetRow } from "@/lib/bindings";

import { DailyReportSheet } from "./DailyReportSheet";

/// 見出しの区切りの全角空白（U+3000。56 UI-09a-D17 から転記）
const SP = "\u3000";
const DATE = "2026年3月21日（土）";
const PRINTED_AT = new Date(2026, 2, 21, 18, 5, 0);

function row(
  position: number,
  label: string,
  amount: number | null,
  overrides: Partial<DailyReportSheetRow> = {},
): DailyReportSheetRow {
  return { position, label, quantity: null, count: 1, amount, import_seq: null, ...overrides };
}

function buildSheet(overrides: Partial<Sheet> = {}): Sheet {
  return {
    date: "2026-03-21",
    import_count: 1,
    summary_rows: [row(1, "総売", 1234, { quantity: 1.3, count: null })],
    payment_rows: [row(1, "現金", 1000), row(2, "", 5)],
    department_rows: [row(1, "部門A", -1234, { quantity: 2, count: null })],
    label_changed_sources: [],
    unsummed_sources: [],
    plu_rows: [
      {
        product_code: "SHEET-P01",
        name: "合成P01",
        department_name: "その他",
        department_id: 1,
        stock_unit: "pcs",
        quantity: 2,
        amount: 600,
        source: "auto",
      },
    ],
    plu_import: { import_count: 1, skipped_count: 0 },
    ...overrides,
  };
}

function sheetRoot() {
  return screen.getByTestId("daily-report-sheet");
}

function tableByHeading(heading: string) {
  // 見出しの全角空白を潰さずに完全一致で探す
  const th = Array.from(sheetRoot().querySelectorAll("th")).find(
    (element) => element.textContent === heading,
  );
  const table = th?.closest("table");
  if (!table) throw new Error(`${heading} の表が無い`);
  return table;
}

function cellTexts(rows: NodeListOf<HTMLTableRowElement>) {
  return Array.from(rows).map((tr) => Array.from(tr.cells).map((cell) => cell.textContent));
}

function headerRows(table: HTMLTableElement) {
  return cellTexts(table.querySelectorAll("thead > tr"));
}

function bodyRows(table: HTMLTableElement) {
  return cellTexts(table.querySelectorAll("tbody > tr"));
}

describe("DailyReportSheet UI-09a-D17", () => {
  it("DailyReportSheet renders tables, notices and no yen sign (UI-09a)", () => {
    const view = render(
      <DailyReportSheet
        sheet={buildSheet({ label_changed_sources: ["Z002"] })}
        printedAt={PRINTED_AT}
      />,
    );

    const summary = tableByHeading(`日計（Z001）${SP}${DATE}`);
    expect(headerRows(summary)).toEqual([
      [`日計（Z001）${SP}${DATE}`],
      ["名称", "個数/件数", "金額"],
    ]);
    expect(bodyRows(summary)).toEqual([["総売", "1.3", "1,234"]]);
    expect(within(summary).queryByText(/¥/)).not.toBeInTheDocument();

    const payment = tableByHeading(`支払集計（Z002）${SP}${DATE}`);
    expect(bodyRows(payment)).toEqual([
      [
        "この日の精算の間で、帳票の行の名称か並びが変わっています。同じ名称の行が 2 行あれば、別々に足しています。",
      ],
      ["現金", "1", "1,000"],
      ["（名前なし）", "1", "5"],
    ]);
    expect(bodyRows(tableByHeading(`部門別集計（Z005）${SP}${DATE}`))).toEqual([
      ["部門A", "2", "-1,234"],
    ]);
    const plu = tableByHeading(`PLU の売上（Z004）${SP}${DATE}`);
    expect(headerRows(plu)[1]).toEqual(["商品名", "数量", "金額"]);
    expect(bodyRows(plu)[0][0]).toBe("合成P01");
    expect(bodyRows(plu)[0][2]).toBe("600");
    view.unmount();

    // 足さなかった帳票は import_seq ごとの表。知らせは最初の表に 3 の文だけ
    render(
      <DailyReportSheet
        sheet={buildSheet({
          import_count: 2,
          label_changed_sources: ["Z002"],
          unsummed_sources: ["Z002"],
          payment_rows: [
            row(1, "項目A", 10, { import_seq: 1 }),
            row(1, "項目X", 5, { import_seq: 2 }),
          ],
        })}
        printedAt={PRINTED_AT}
      />,
    );
    const first = tableByHeading(`支払集計（Z002）${SP}1回目の取込み${SP}${DATE}`);
    const second = tableByHeading(`支払集計（Z002）${SP}2回目の取込み${SP}${DATE}`);
    expect(bodyRows(first)).toEqual([
      [
        "この日の精算の間で、帳票の行の名称か並びが変わり、1 回の精算の中に同じ名称の行が 2 行以上あります。別の項目を足さないよう、この帳票は足さずに取込みごとに載せています。",
      ],
      ["項目A", "1", "10"],
    ]);
    expect(bodyRows(second)).toEqual([["項目X", "1", "5"]]);
  });

  it("DailyReportSheet title on unsummed day (UI-09a)", () => {
    const view = render(
      <DailyReportSheet
        sheet={buildSheet({ import_count: 2, unsummed_sources: ["Z002"] })}
        printedAt={PRINTED_AT}
      />,
    );
    expect(
      within(sheetRoot()).getByText("2回の取込みを合算（取込みごとに載せた帳票を除く）"),
    ).toBeInTheDocument();
    expect(within(sheetRoot()).getByText("売上日報")).toBeInTheDocument();
    expect(within(sheetRoot()).getByText(DATE)).toBeInTheDocument();
    expect(within(sheetRoot()).getByText("印刷 2026-03-21 18:05")).toBeInTheDocument();
    view.unmount();

    render(<DailyReportSheet sheet={buildSheet({ import_count: 2 })} printedAt={PRINTED_AT} />);
    expect(within(sheetRoot()).getByText("2回の取込みを合算")).toBeInTheDocument();
  });

  it("DailyReportSheet PLU notices by import state (UI-09a)", () => {
    const notImported = render(
      <DailyReportSheet
        sheet={buildSheet({ plu_rows: [], plu_import: { import_count: 0, skipped_count: 0 } })}
        printedAt={PRINTED_AT}
      />,
    );
    expect(
      within(sheetRoot()).getByText("この日の PLU の売上は取り込まれていません。"),
    ).toBeInTheDocument();
    notImported.unmount();

    const noneSold = render(
      <DailyReportSheet
        sheet={buildSheet({ plu_rows: [], plu_import: { import_count: 1, skipped_count: 0 } })}
        printedAt={PRINTED_AT}
      />,
    );
    expect(
      within(sheetRoot()).getByText("この日に売れた PLU の商品はありません。"),
    ).toBeInTheDocument();
    noneSold.unmount();

    render(
      <DailyReportSheet
        sheet={buildSheet({ plu_import: { import_count: 1, skipped_count: 3 } })}
        printedAt={PRINTED_AT}
      />,
    );
    expect(
      within(sheetRoot()).getByText(
        "数えていない PLU の行が 3 行あります（入出庫履歴の CSV取込みの記録で見られます）。",
      ),
    ).toBeInTheDocument();
  });

  it("DailyReportSheet is print-only and min 9pt (UI-09a)", () => {
    render(<DailyReportSheet sheet={buildSheet()} printedAt={PRINTED_AT} />);
    const root = sheetRoot();
    expect(root.parentElement).toBe(document.body);
    expect(root).toHaveClass("hidden");
    expect(root).toHaveClass("print:block");
    expect(root).toHaveClass("text-[9pt]");
    const classNames = [root, ...Array.from(root.querySelectorAll("*"))].map(
      (element) => element.getAttribute("class") ?? "",
    );
    for (const className of classNames) {
      expect(className).not.toMatch(/\btext-(xs|sm)\b/);
      for (const match of className.matchAll(/text-\[(\d+(?:\.\d+)?)pt\]/g)) {
        expect(Number(match[1])).toBeGreaterThanOrEqual(9);
      }
      // 色を使わない（黒の文字と罫線だけ）
      expect(className).not.toMatch(
        /\b(text|bg|border)-(red|green|blue|amber|primary|destructive)/,
      );
    }
  });
});
