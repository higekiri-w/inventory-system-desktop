// src/features/stock-inquiry/components/StockRecheckBadge.test.tsx
//
// D-118 b2（Matrix U8）: 在庫の欄の共有の部品が商品の写しを b1 の formatStockOrRange へ渡す。
// 書式そのものは b1 の stocktake-formatters.test.ts。

import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import type { ProductStockUnit } from "@/lib/bindings";
import { StockWithRecheck } from "./StockRecheckBadge";

function copy(
  stock: number,
  stock_unit: ProductStockUnit,
  range: [number, number] | null,
): Parameters<typeof StockWithRecheck>[0]["product"] {
  return {
    stock_quantity: stock,
    stock_unit,
    stock_recheck: range !== null,
    stock_range_low: range?.[0] ?? null,
    stock_range_high: range?.[1] ?? null,
  };
}

describe("StockWithRecheck（D-118 b2）", () => {
  it.each([
    ["個数の要再確認", copy(9, "pcs", [7, 12]), "7〜12 個", true],
    ["長さの要再確認", copy(450, "m", [400, 450]), "4〜4.5 m", true],
    ["負の端", copy(0, "pcs", [-1, 0]), "-1〜0 個", true],
    ["要再確認でない", copy(3, "pcs", null), "3 個", false],
  ] as const)(
    "U8 stock cell passes copy range to formatStockOrRange and shows recheck badge（%s）",
    (_label, product, text, badge) => {
      // REQ-301 / REQ-303 / SPEC-STK-CNT-D3・SPEC-UNIT-D3（Matrix U8）
      const { container } = render(<StockWithRecheck product={product} />);
      expect(screen.getByText(text)).toBeInTheDocument();
      if (badge) {
        const element = screen.getByText("要再確認");
        expect(element).toHaveAttribute("data-tone", "warning");
        expect(element.querySelector('svg[aria-hidden="true"]')).toBeInTheDocument();
      } else {
        expect(container).not.toHaveTextContent("要再確認");
      }
    },
  );
});
