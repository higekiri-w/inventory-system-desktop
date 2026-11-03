// src/features/monthly-sales/hooks/useMonthlySalesReport.test.tsx
//
// REQ-502 P2-2: useMonthlySalesReport の sort 結線テスト (sortBy/sortDir 引数 →
// derived.ranking + derived.composition の双方に sortMonthlyItems 適用、raw items
// 非適用)。Page を介さず renderHook で hook 単体検証、Router 不要。
// sort 非対称根拠 (ranking + composition 双方 / raw items 非適用) は
// docs/plans/2026-05-19-pr-66-codex-r1-p2-fixes.md §2 commit 3 参照。

import { describe, it, expect, vi, beforeEach } from "vitest";
import type { ReactNode } from "react";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import { commands } from "@/lib/bindings";
import type { MonthlySalesReport } from "@/lib/bindings";

import { useMonthlySalesReport } from "./useMonthlySalesReport";

vi.mock("@/lib/bindings", () => ({
  commands: {
    getMonthlySales: vi.fn(),
  },
}));

const mockGetMonthlySales = vi.mocked(commands.getMonthlySales);

function makeWrapper() {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={qc}>{children}</QueryClientProvider>;
  };
}

function buildByProductReport(): MonthlySalesReport {
  return {
    month: "2026-05",
    mode: "by_product",
    items: [
      // BIZ row_number 順 = ranking 1, 2, 3
      {
        key: "P001",
        label: "商品A",
        count_points: 1,
        length_cm: 0,
        stock_unit: "pcs",
        amount: 5000,
        ranking: 1,
      },
      {
        key: "P002",
        label: "商品B",
        count_points: 10,
        length_cm: 0,
        stock_unit: "pcs",
        amount: 1000,
        ranking: 2,
      },
      {
        key: "P003",
        label: "商品C",
        count_points: 5,
        length_cm: 0,
        stock_unit: "pcs",
        amount: 3000,
        ranking: 3,
      },
    ],
    prev_month_comparison: null,
    official_department_totals: null,
    official_report_dates: [],
  };
}

function buildByDepartmentReport(): MonthlySalesReport {
  return {
    month: "2026-05",
    mode: "by_department",
    items: [
      // BIZ 側で部門単位で集計済 (key = department_id)
      {
        key: "1",
        label: "毛糸",
        count_points: 11,
        length_cm: 0,
        stock_unit: null,
        amount: 8000,
        ranking: 1,
      },
      {
        key: "2",
        label: "布",
        count_points: 5,
        length_cm: 0,
        stock_unit: null,
        amount: 1000,
        ranking: 2,
      },
      {
        key: "3",
        label: "手芸用品",
        count_points: 3,
        length_cm: 0,
        stock_unit: null,
        amount: 3000,
        ranking: 3,
      },
    ],
    prev_month_comparison: null,
    official_department_totals: null,
    official_report_dates: [],
  };
}

beforeEach(() => {
  mockGetMonthlySales.mockResolvedValue({ status: "ok", data: buildByProductReport() });
});

describe("useMonthlySalesReport (REQ-502 sort 結線)", () => {
  it("REQ-502: sortBy='amount' desc で derived.ranking が金額降順になる", async () => {
    const { result } = renderHook(
      () =>
        useMonthlySalesReport({
          month: "2026-05",
          mode: "by_product",
          sortBy: "amount",
          sortDir: "desc",
        }),
      { wrapper: makeWrapper() },
    );
    await waitFor(() => {
      expect(result.current.derived).not.toBeNull();
    });
    const ranking = result.current.derived?.ranking ?? [];
    expect(ranking.map((r) => r.amount)).toEqual([5000, 3000, 1000]);
  });

  it("REQ-502: sortBy=null で BIZ row_number 順 (ranking 1→2→3) が保持される", async () => {
    const { result } = renderHook(
      () =>
        useMonthlySalesReport({
          month: "2026-05",
          mode: "by_product",
          sortBy: null,
          sortDir: "asc",
        }),
      { wrapper: makeWrapper() },
    );
    await waitFor(() => {
      expect(result.current.derived).not.toBeNull();
    });
    const ranking = result.current.derived?.ranking ?? [];
    expect(ranking.map((r) => r.ranking)).toEqual([1, 2, 3]);
  });

  it("REQ-502: prev_month_comparison: [] (BIZ contract Some(空Vec)) で全行 prev_month_diff が null になる", async () => {
    // BIZ contract: src-tauri/src/biz/sales_service.rs の get_monthly_sales で前月データなしも Some(空Vec) を返す常時セット。
    // `null` は specta `Option<Vec<T>>` 境界の defensive guard で UI 側 compute-comparison が
    // 両ケースを safely 扱うが、本 test は通常 path の空配列ケース (Some(空Vec)) を test で固定。
    // compute-comparison.ts は prev === [] のとき各 cur item で prevMap.get() === undefined →
    // isComparable=false entry を全 cur key に作るため、Map.size === items.length となる。
    mockGetMonthlySales.mockResolvedValue({
      status: "ok",
      data: { ...buildByProductReport(), prev_month_comparison: [] },
    });
    const { result } = renderHook(
      () =>
        useMonthlySalesReport({
          month: "2026-05",
          mode: "by_product",
          sortBy: null,
          sortDir: "asc",
        }),
      { wrapper: makeWrapper() },
    );
    await waitFor(() => {
      expect(result.current.derived).not.toBeNull();
    });
    const ranking = result.current.derived?.ranking ?? [];
    // 各行 prev_month_diff が null (BIZ contract Some(空Vec) = 前月データなし)
    expect(ranking.every((r) => r.prev_month_diff === null)).toBe(true);
    // comparisonMap は items.length 件の entry を持ち、全 entry が isComparable=false
    const comparisonMap = result.current.derived?.comparisonMap;
    expect(comparisonMap?.size).toBe(3);
    for (const info of comparisonMap?.values() ?? []) {
      expect(info.isComparable).toBe(false);
      expect(info.diff).toBeNull();
      expect(info.ratio).toBeNull();
    }
  });

  it("REQ-502: sortBy='amount' desc は composition (部門別) にも適用される", async () => {
    mockGetMonthlySales.mockResolvedValue({ status: "ok", data: buildByDepartmentReport() });
    const { result } = renderHook(
      () =>
        useMonthlySalesReport({
          month: "2026-05",
          mode: "by_department",
          sortBy: "amount",
          sortDir: "desc",
        }),
      { wrapper: makeWrapper() },
    );
    await waitFor(() => {
      expect(result.current.derived).not.toBeNull();
    });
    const composition = result.current.derived?.composition ?? [];
    // amount desc 順 = 毛糸 8000 → 手芸用品 3000 → 布 1000
    expect(composition.map((r) => r.amount)).toEqual([8000, 3000, 1000]);
  });
});
