import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import type { ImportRow } from "@/lib/bindings";
import { ProductImportPreview } from "./ProductImportPreview";

function row(code: string, pluTarget: boolean | null, warnings: string[] = []): ImportRow {
  return {
    line_no: 2,
    product_code: code,
    name: `商品 ${code}`,
    department_id: 1,
    selling_price: 100,
    cost_price_centi: 50,
    tax_rate: "10",
    stock_unit: "pcs",
    initial_stock: null,
    jan_code: null,
    maker_code: null,
    supplier_id: null,
    pos_stock_sync: true,
    plu_target: pluTarget,
    warnings,
  };
}

function tableRowFor(code: string): HTMLElement {
  const tableRow = screen.getByText(code).closest("tr");
  if (tableRow === null) throw new Error(`row not found: ${code}`);
  return tableRow;
}

describe("ProductImportPreview", () => {
  it("REQ-907 B-P1: shows all PLU target preview states and warning with an icon", () => {
    render(
      <ProductImportPreview
        filename="synthetic.csv"
        preview={{
          valid_rows: [
            row("P-TARGET", true),
            row("P-EXCLUDED", false, ["JAN が13桁でないため対象外として取り込みます"]),
            row("P-DEFAULT", null),
          ],
          duplicate_rows: [],
          error_rows: [],
        }}
        overwriteCodes={[]}
        targetCount={3}
        isCommitting={false}
        onToggleOverwrite={vi.fn()}
        onCommit={vi.fn()}
        onReselect={vi.fn()}
      />,
    );
    expect(screen.getByRole("columnheader", { name: "PLU対象" })).toBeInTheDocument();
    expect(within(tableRowFor("P-TARGET")).getByText("対象")).toBeInTheDocument();
    expect(within(tableRowFor("P-EXCLUDED")).getByText("対象外")).toBeInTheDocument();
    expect(
      within(tableRowFor("P-DEFAULT")).getByText("既定（13桁JANなら対象）"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("PLU対象が空欄の場合は、13桁JANの商品を対象とする既定ルールを適用します。"),
    ).toBeInTheDocument();
    const warning = screen.getByText("JAN が13桁でないため対象外として取り込みます");
    expect(warning.parentElement?.querySelector("svg[aria-hidden='true']")).not.toBeNull();
  });

  it("DSR-20 D-C/D-D T7/T9: 上書き確認は destructive Action とキャンセル文言を使う", async () => {
    const user = userEvent.setup();
    render(
      <ProductImportPreview
        filename="synthetic.csv"
        preview={{
          valid_rows: [],
          duplicate_rows: [
            {
              line_no: 2,
              import_row: row("P-DUP", true),
              existing_product_code: "P-DUP",
            },
          ],
          error_rows: [],
        }}
        overwriteCodes={["P-DUP"]}
        targetCount={1}
        isCommitting={false}
        onToggleOverwrite={vi.fn()}
        onCommit={vi.fn()}
        onReselect={vi.fn()}
      />,
    );

    // SC3 / DSR-22 / D-CE3: 上書き件数は③強調の琥珀 pill（--warning-emphasis の塗り）。
    expect(screen.getByText("上書き 1 件")).toHaveClass(
      "border-warning",
      "bg-warning-emphasis",
      "text-primary-foreground",
    );
    expect(screen.getByText("上書き 1 件")).not.toHaveClass("bg-primary");
    await user.click(screen.getByRole("button", { name: "インポート実行" }));
    const dialog = screen.getByRole("alertdialog");
    expect(within(dialog).getByRole("button", { name: "キャンセル" })).toBeInTheDocument();
    expect(within(dialog).queryByRole("button", { name: "戻る" })).not.toBeInTheDocument();
    expect(within(dialog).getByRole("button", { name: "上書きして実行" })).toHaveAttribute(
      "data-variant",
      "destructive",
    );
  });

  it("REQ-104 UI-01c-D18: エラー行は 50 行の制限に掛けず全件と取り込まない旨を出す", () => {
    const lineNos = Array.from({ length: 60 }, (_, index) => index + 2);
    render(
      <ProductImportPreview
        filename="synthetic.csv"
        preview={{
          valid_rows: lineNos.map((lineNo) => ({
            ...row(`V-${String(lineNo)}`, null),
            line_no: lineNo,
          })),
          duplicate_rows: [],
          error_rows: lineNos.map((lineNo) => ({
            line_no: lineNo,
            raw_data: {},
            errors: [`合成の誤り ${String(lineNo)} 行目`],
          })),
        }}
        overwriteCodes={[]}
        targetCount={60}
        isCommitting={false}
        onToggleOverwrite={vi.fn()}
        onCommit={vi.fn()}
        onReselect={vi.fn()}
      />,
    );

    const errorCard = screen.getByText("エラー行").closest<HTMLElement>("[data-slot='card']");
    if (errorCard === null) throw new Error("error card not found");
    expect(
      within(errorCard).getByText(
        "エラー行は取り込みません。内容の文のとおりに CSV を直し、「ファイルを選び直す」で読み込み直してください",
      ),
    ).toBeInTheDocument();
    expect(within(errorCard).getByText("合成の誤り 61 行目")).toBeInTheDocument();
    expect(within(errorCard).getByText("合成の誤り 2 行目")).toBeInTheDocument();
    expect(within(errorCard).queryByText(/^他 /)).not.toBeInTheDocument();

    // UI-01c-D5: 新規登録候補の表は先頭 50 行のまま
    const validCard = screen.getByText("新規登録候補").closest<HTMLElement>("[data-slot='card']");
    if (validCard === null) throw new Error("valid card not found");
    expect(within(validCard).getByText("V-51")).toBeInTheDocument();
    expect(within(validCard).queryByText("V-52")).not.toBeInTheDocument();
  });
});
