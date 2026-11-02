// src/features/stock-inquiry/hooks/useStockInquiry.test.tsx
//
// REQ-301/302: useStockInquiry の正規化型 + enabled gate + 1 件自動展開検証。
// SPEC-UIBB-3/4/9: page 反映 + 部門候補 listDepartments 化（DSR-10）の検証。
// renderHook + QueryClientProvider（navigate は arg 注入のため Router 不要）。
// 設計: docs/function-design/58-ui-stock-inquiry.md §58.5 / §58.9

import { describe, it, expect, vi, beforeEach } from "vitest";
import type { ReactNode } from "react";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import { commands } from "@/lib/bindings";
import { queryKeys } from "@/lib/query-keys";
import { useStockInquiry } from "./useStockInquiry";
import {
  makeMockDepartment,
  makeMockProductWithRelations,
  makeMockStockDetail,
} from "../lib/test-fixtures";

vi.mock("@/lib/bindings", () => ({
  commands: {
    searchProducts: vi.fn(),
    listLowStock: vi.fn(),
    getStockDetail: vi.fn(),
    listDepartments: vi.fn(),
  },
}));

const mockSearch = vi.mocked(commands.searchProducts);
const mockLowStock = vi.mocked(commands.listLowStock);
const mockDetail = vi.mocked(commands.getStockDetail);
const mockListDepartments = vi.mocked(commands.listDepartments);

function makeWrapper(qc?: QueryClient) {
  const client =
    qc ??
    new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
    });
  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
  };
}

beforeEach(() => {
  mockSearch.mockReset();
  mockLowStock.mockReset();
  mockDetail.mockReset();
  mockListDepartments.mockReset();
  mockListDepartments.mockResolvedValue({ status: "ok", data: [] });
});

describe("useStockInquiry (REQ-301/302)", () => {
  it("REQ-301: search 結果を StockInquiryListResult に正規化（source=search / totalCount）", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-001" })],
        total_count: 80,
        page: 1,
        per_page: 50,
      },
    });
    const navigate = vi.fn();
    const { result } = renderHook(
      () =>
        useStockInquiry({
          status: "all",
          q: "毛糸",
          dept: null,
          page: 1,
          perPage: 50,
          selected: null,
          navigate,
        }),
      { wrapper: makeWrapper() },
    );
    await waitFor(() => {
      expect(result.current.listQuery.isSuccess).toBe(true);
    });
    const data = result.current.listQuery.data;
    expect(data?.source).toBe("search");
    expect(data?.totalCount).toBe(80);
    expect(data?.items.map((p) => p.product_code)).toEqual(["P-001"]);
  });

  it("REQ-302: low_stock 結果を配列正規化（source=low_stock / totalCount=null / sub-filter）", async () => {
    mockLowStock.mockResolvedValue({
      status: "ok",
      data: [
        makeMockProductWithRelations({ product_code: "L-001", stock_quantity: 0 }),
        makeMockProductWithRelations({ product_code: "L-002", stock_quantity: 3 }),
      ],
    });
    const navigate = vi.fn();
    const { result } = renderHook(
      () =>
        useStockInquiry({
          status: "stockout",
          q: "",
          dept: null,
          page: 1,
          perPage: 50,
          selected: null,
          navigate,
        }),
      { wrapper: makeWrapper() },
    );
    await waitFor(() => {
      expect(result.current.listQuery.isSuccess).toBe(true);
    });
    const data = result.current.listQuery.data;
    expect(data?.source).toBe("low_stock");
    expect(data?.totalCount).toBeNull();
    // status=stockout の sub-filter で stock<=0 のみ
    expect(data?.items.map((p) => p.product_code)).toEqual(["L-001"]);
  });

  it("REQ-301: status=all + q 空文字は search_products を呼ばない（enabled gate / isAllEmpty）", async () => {
    const navigate = vi.fn();
    const { result } = renderHook(
      () =>
        useStockInquiry({
          status: "all",
          q: "",
          dept: null,
          page: 1,
          perPage: 50,
          selected: null,
          navigate,
        }),
      { wrapper: makeWrapper() },
    );
    expect(result.current.isAllEmpty).toBe(true);
    // enabled=false のため fetch されない
    await waitFor(() => {
      expect(result.current.listQuery.fetchStatus).toBe("idle");
    });
    expect(mockSearch).not.toHaveBeenCalled();
  });

  it("SPEC-UIBB-3/4: page が queryKey とクエリ引数に反映される", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-001" })],
        total_count: 60,
        page: 2,
        per_page: 50,
      },
    });
    const navigate = vi.fn();
    const { result } = renderHook(
      () =>
        useStockInquiry({
          status: "all",
          q: "毛糸",
          dept: null,
          page: 2,
          perPage: 50,
          selected: null,
          navigate,
        }),
      { wrapper: makeWrapper() },
    );
    await waitFor(() => {
      expect(result.current.listQuery.isSuccess).toBe(true);
    });
    expect(mockSearch).toHaveBeenCalledWith(expect.objectContaining({ page: 2 }));
  });

  it("REQ-301: 結果 1 件で詳細カード自動展開（navigate に selected 渡し）", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "SOLO-1" })],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });
    const navigate = vi.fn();
    renderHook(
      () =>
        useStockInquiry({
          status: "all",
          q: "SOLO",
          dept: null,
          page: 1,
          perPage: 50,
          selected: null,
          navigate,
        }),
      { wrapper: makeWrapper() },
    );
    await waitFor(() => {
      expect(navigate).toHaveBeenCalledWith({ selected: "SOLO-1" });
    });
  });

  it("SC5a (UI-06a-D5): 手動クローズ後、同一検索条件では自動展開が再発火しない", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "SOLO-1" })],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });
    const navigate = vi.fn();
    const { result, rerender } = renderHook(
      (args: Parameters<typeof useStockInquiry>[0]) => useStockInquiry(args),
      {
        wrapper: makeWrapper(),
        initialProps: {
          status: "all",
          q: "SOLO",
          dept: null,
          page: 1,
          perPage: 50,
          selected: null,
          navigate,
        },
      },
    );
    await waitFor(() => {
      expect(navigate).toHaveBeenCalledWith({ selected: "SOLO-1" });
    });
    expect(navigate).toHaveBeenCalledTimes(1);
    // navigate が実際に URL の selected を更新したと仮定して反映（自動展開後）
    rerender({
      status: "all",
      q: "SOLO",
      dept: null,
      page: 1,
      perPage: 50,
      selected: "SOLO-1",
      navigate,
    });
    // 手動クローズ相当: 検索条件は変えず selected だけ null に戻す
    rerender({
      status: "all",
      q: "SOLO",
      dept: null,
      page: 1,
      perPage: 50,
      selected: null,
      navigate,
    });
    // guard がなければここで再度 navigate({ selected: "SOLO-1" }) が呼ばれてしまう。
    // list query は既に成功済みで isSuccess は変わらないため、これを安定条件として待つ。
    await waitFor(() => {
      expect(result.current.listQuery.isSuccess).toBe(true);
    });
    expect(navigate).toHaveBeenCalledTimes(1);
  });

  it("SC5b (UI-06a-D5): 検索条件が変化した後は新しい条件で自動展開が再度発火する", async () => {
    mockSearch.mockImplementation(
      (query: { keyword: string | null; department_id: number | null }) => {
        if (query.department_id === 7) {
          return Promise.resolve({
            status: "ok",
            data: {
              items: [makeMockProductWithRelations({ product_code: "DEPT-1" })],
              total_count: 1,
              page: 1,
              per_page: 50,
            },
          });
        }
        if (query.keyword === "SOLO") {
          return Promise.resolve({
            status: "ok",
            data: {
              items: [makeMockProductWithRelations({ product_code: "SOLO-1" })],
              total_count: 1,
              page: 1,
              per_page: 50,
            },
          });
        }
        return Promise.resolve({
          status: "ok",
          data: {
            items: [makeMockProductWithRelations({ product_code: "OTHER-1" })],
            total_count: 1,
            page: 1,
            per_page: 50,
          },
        });
      },
    );
    const navigate = vi.fn();
    const { rerender } = renderHook(
      (args: Parameters<typeof useStockInquiry>[0]) => useStockInquiry(args),
      {
        wrapper: makeWrapper(),
        initialProps: {
          status: "all",
          q: "SOLO",
          dept: null,
          page: 1,
          perPage: 50,
          selected: null,
          navigate,
        },
      },
    );
    await waitFor(() => {
      expect(navigate).toHaveBeenCalledWith({ selected: "SOLO-1" });
    });
    // 検索条件（q）を変更。selected は null のまま（新条件では再度自動展開してよい）
    rerender({
      status: "all",
      q: "OTHER",
      dept: null,
      page: 1,
      perPage: 50,
      selected: null,
      navigate,
    });
    await waitFor(() => {
      expect(navigate).toHaveBeenCalledWith({ selected: "OTHER-1" });
    });
    // 検索条件（dept）を変更。q/status/page は変えず selected は null のまま
    // （conditionKey から dept 成分を落とす mutant を検出する）
    rerender({
      status: "all",
      q: "OTHER",
      dept: 7,
      page: 1,
      perPage: 50,
      selected: null,
      navigate,
    });
    await waitFor(() => {
      expect(navigate).toHaveBeenCalledWith({ selected: "DEPT-1" });
    });
  });

  it("Codex P2-2: status のみ変化しても conditionKey の変化として自動展開が再度発火する", async () => {
    mockLowStock.mockResolvedValue({
      status: "ok",
      data: [
        makeMockProductWithRelations({ product_code: "STOCKOUT-1", stock_quantity: 0 }),
        makeMockProductWithRelations({ product_code: "LOWSTOCK-1", stock_quantity: 5 }),
      ],
    });
    const navigate = vi.fn();
    const { rerender } = renderHook(
      (args: Parameters<typeof useStockInquiry>[0]) => useStockInquiry(args),
      {
        wrapper: makeWrapper(),
        initialProps: {
          status: "stockout",
          q: "",
          dept: null,
          page: 1,
          perPage: 50,
          selected: null,
          navigate,
        },
      },
    );
    await waitFor(() => {
      expect(navigate).toHaveBeenCalledWith({ selected: "STOCKOUT-1" });
    });
    // q / dept / page は変えず status だけ変更する（conditionKey から status を落とす mutant を検出）
    rerender({
      status: "low_stock",
      q: "",
      dept: null,
      page: 1,
      perPage: 50,
      selected: null,
      navigate,
    });
    await waitFor(() => {
      expect(navigate).toHaveBeenCalledWith({ selected: "LOWSTOCK-1" });
    });
  });

  it("Codex P2-2: page のみ変化しても conditionKey の変化として自動展開が再度発火する", async () => {
    mockSearch.mockImplementation((query: { page: number }) => {
      if (query.page === 2) {
        return Promise.resolve({
          status: "ok",
          data: {
            items: [makeMockProductWithRelations({ product_code: "PAGE2-1" })],
            total_count: 2,
            page: 2,
            per_page: 1,
          },
        });
      }
      return Promise.resolve({
        status: "ok",
        data: {
          items: [makeMockProductWithRelations({ product_code: "PAGE1-1" })],
          total_count: 2,
          page: 1,
          per_page: 1,
        },
      });
    });
    const navigate = vi.fn();
    const { rerender } = renderHook(
      (args: Parameters<typeof useStockInquiry>[0]) => useStockInquiry(args),
      {
        wrapper: makeWrapper(),
        initialProps: {
          status: "all",
          q: "X",
          dept: null,
          page: 1,
          perPage: 1,
          selected: null,
          navigate,
        },
      },
    );
    await waitFor(() => {
      expect(navigate).toHaveBeenCalledWith({ selected: "PAGE1-1" });
    });
    // status / q / dept は変えず page だけ変更する（conditionKey から page を落とす mutant を検出）
    rerender({
      status: "all",
      q: "X",
      dept: null,
      page: 2,
      perPage: 1,
      selected: null,
      navigate,
    });
    await waitFor(() => {
      expect(navigate).toHaveBeenCalledWith({ selected: "PAGE2-1" });
    });
  });

  it("Codex P1-1: URL 復元済み selected（非 null）は 1 件でも自動展開せず、null に閉じても再展開しない", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "SOLO-1" })],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });
    mockDetail.mockResolvedValue({ status: "ok", data: makeMockStockDetail() });
    const navigate = vi.fn();
    const { result, rerender } = renderHook(
      (args: Parameters<typeof useStockInquiry>[0]) => useStockInquiry(args),
      {
        wrapper: makeWrapper(),
        initialProps: {
          status: "all",
          q: "SOLO",
          dept: null,
          page: 1,
          perPage: 50,
          selected: "SOLO-1",
          navigate,
        },
      },
    );
    await waitFor(() => {
      expect(result.current.listQuery.isSuccess).toBe(true);
    });
    // 選択行の再クリックで閉じる（R2-3）相当: 検索条件は変えず selected だけ null に戻す
    rerender({
      status: "all",
      q: "SOLO",
      dept: null,
      page: 1,
      perPage: 50,
      selected: null,
      navigate,
    });
    await waitFor(() => {
      expect(result.current.detailQuery.isFetching).toBe(false);
    });
    expect(navigate).not.toHaveBeenCalled();
  });

  it("Codex P1-2: 自動展開しない条件を挟んでも、元の条件に戻れば改めて自動展開する", async () => {
    mockSearch.mockImplementation(
      (query: { keyword: string | null; department_id: number | null }) => {
        if (query.keyword === "SOLO") {
          return Promise.resolve({
            status: "ok",
            data: {
              items: [makeMockProductWithRelations({ product_code: "SOLO-1" })],
              total_count: 1,
              page: 1,
              per_page: 50,
            },
          });
        }
        return Promise.resolve({
          status: "ok",
          data: {
            items: [
              makeMockProductWithRelations({ product_code: "OTHER-1" }),
              makeMockProductWithRelations({ product_code: "OTHER-2" }),
            ],
            total_count: 2,
            page: 1,
            per_page: 50,
          },
        });
      },
    );
    const navigate = vi.fn();
    const { rerender } = renderHook(
      (args: Parameters<typeof useStockInquiry>[0]) => useStockInquiry(args),
      {
        wrapper: makeWrapper(),
        initialProps: {
          status: "all",
          q: "SOLO",
          dept: null,
          page: 1,
          perPage: 50,
          selected: null,
          navigate,
        },
      },
    );
    await waitFor(() => {
      expect(navigate).toHaveBeenCalledWith({ selected: "SOLO-1" });
    });
    expect(navigate).toHaveBeenCalledTimes(1);
    // 手動クローズ相当: 検索条件は変えず selected だけ null に戻す
    rerender({
      status: "all",
      q: "SOLO",
      dept: null,
      page: 1,
      perPage: 50,
      selected: null,
      navigate,
    });
    // 条件 B（複数件、自動展開しない）を挟む
    rerender({
      status: "all",
      q: "OTHER",
      dept: null,
      page: 1,
      perPage: 50,
      selected: null,
      navigate,
    });
    // 条件 A に戻す
    rerender({
      status: "all",
      q: "SOLO",
      dept: null,
      page: 1,
      perPage: 50,
      selected: null,
      navigate,
    });
    await waitFor(() => {
      expect(navigate).toHaveBeenCalledTimes(2);
    });
    expect(navigate).toHaveBeenNthCalledWith(2, { selected: "SOLO-1" });
  });

  it("Codex round2 P2: ページ送りで前ページの selected が残ったままでも、新条件の単一結果が自動展開する", async () => {
    mockSearch.mockImplementation((query: { page: number }) => {
      if (query.page === 2) {
        return Promise.resolve({
          status: "ok",
          data: {
            items: [makeMockProductWithRelations({ product_code: "PAGE2-51" })],
            total_count: 51,
            page: 2,
            per_page: 50,
          },
        });
      }
      return Promise.resolve({
        status: "ok",
        data: {
          items: [
            makeMockProductWithRelations({ product_code: "PAGE1-1" }),
            makeMockProductWithRelations({ product_code: "PAGE1-2" }),
          ],
          total_count: 51,
          page: 1,
          per_page: 50,
        },
      });
    });
    mockDetail.mockResolvedValue({ status: "ok", data: makeMockStockDetail() });
    const navigate = vi.fn();
    const { result, rerender } = renderHook(
      (args: Parameters<typeof useStockInquiry>[0]) => useStockInquiry(args),
      {
        wrapper: makeWrapper(),
        initialProps: {
          status: "all",
          q: "X",
          dept: null,
          page: 1,
          perPage: 50,
          selected: null,
          navigate,
        },
      },
    );
    await waitFor(() => {
      expect(result.current.listQuery.isSuccess).toBe(true);
    });
    expect(navigate).not.toHaveBeenCalled();
    // 1 ページ目の商品をクリックして詳細を開く（手動選択、複数件のため自動展開は発火していない）
    rerender({
      status: "all",
      q: "X",
      dept: null,
      page: 1,
      perPage: 50,
      selected: "PAGE1-1",
      navigate,
    });
    // 次のページへ。page 側の既存 onPageChange は selected を維持したまま渡す
    // （新条件の結果には存在しない stale selected）。
    rerender({
      status: "all",
      q: "X",
      dept: null,
      page: 2,
      perPage: 50,
      selected: "PAGE1-1",
      navigate,
    });
    // 不在の selected を clear（C-P2-1 相当）
    await waitFor(() => {
      expect(navigate).toHaveBeenCalledWith({ selected: undefined });
    });
    // URL 反映を模擬（selected: null）
    rerender({
      status: "all",
      q: "X",
      dept: null,
      page: 2,
      perPage: 50,
      selected: null,
      navigate,
    });
    // 消費済み判定は現条件の結果に所属する selected にのみ限定されるべきで、
    // 不在だった stale selected は消費を成立させていないため、新条件の単一結果は自動展開されるべき。
    await waitFor(() => {
      expect(navigate).toHaveBeenCalledWith({ selected: "PAGE2-51" });
    });
  });

  it("REQ-301: selected 既存時は 1 件でも自動展開しない（重複発火回避）", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "SOLO-1" })],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });
    mockDetail.mockResolvedValue({ status: "ok", data: makeMockStockDetail() });
    const navigate = vi.fn();
    const { result } = renderHook(
      () =>
        useStockInquiry({
          status: "all",
          q: "SOLO",
          dept: null,
          page: 1,
          perPage: 50,
          selected: "SOLO-1",
          navigate,
        }),
      { wrapper: makeWrapper() },
    );
    await waitFor(() => {
      expect(result.current.listQuery.isSuccess).toBe(true);
    });
    expect(navigate).not.toHaveBeenCalled();
  });

  it("REQ-301: detail query は list と独立（部分障害許容、detail 失敗でも list 成功）", async () => {
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [makeMockProductWithRelations({ product_code: "P-001" })],
        total_count: 1,
        page: 1,
        per_page: 50,
      },
    });
    mockDetail.mockResolvedValue({
      status: "error",
      error: {
        kind: "not_found",
        message: "商品が見つかりません",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    const navigate = vi.fn();
    const { result } = renderHook(
      () =>
        useStockInquiry({
          status: "all",
          q: "P",
          dept: null,
          page: 1,
          perPage: 50,
          selected: "P-001",
          navigate,
        }),
      { wrapper: makeWrapper() },
    );
    // hook の retry:1（backoff ~1s）を待つため timeout 拡張
    await waitFor(
      () => {
        expect(result.current.detailQuery.isError).toBe(true);
      },
      { timeout: 5000 },
    );
    // detail 失敗でも list は成功（部分障害許容）
    expect(result.current.listQuery.isSuccess).toBe(true);
  });

  it("REQ-301: list 成功時に selected が現 list に不在なら clear（navigate selected:undefined、C-P2-1）", async () => {
    // stale/手打ち URL 相当: 現 list（P-001/P-002）に存在しない selected を渡す。
    mockSearch.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          makeMockProductWithRelations({ product_code: "P-001" }),
          makeMockProductWithRelations({ product_code: "P-002" }),
        ],
        total_count: 2,
        page: 1,
        per_page: 50,
      },
    });
    mockDetail.mockResolvedValue({ status: "ok", data: makeMockStockDetail() });
    const navigate = vi.fn();
    renderHook(
      () =>
        useStockInquiry({
          status: "all",
          q: "P",
          dept: null,
          page: 1,
          perPage: 50,
          selected: "STALE-999",
          navigate,
        }),
      { wrapper: makeWrapper() },
    );
    // list 成功後、selected が items に不在なので clear（複数件のため自動展開は発火しない）
    await waitFor(() => {
      expect(navigate).toHaveBeenCalledWith({ selected: undefined });
    });
  });

  it("REQ-301: 検索前（status=all + q 空）に selected 付き URL → clear + detail 走らせない（Codex Round1 P2-2）", async () => {
    // isAllEmpty（list は EmptySearchPlaceholder）なのに selected 付き手打ち/F5/bookmark URL のケース。
    const navigate = vi.fn();
    renderHook(
      () =>
        useStockInquiry({
          status: "all",
          q: "", // isAllEmpty = true
          dept: null,
          page: 1,
          perPage: 50,
          selected: "STALE-1",
          navigate,
        }),
      { wrapper: makeWrapper() },
    );
    // isAllEmpty + selected → clear
    await waitFor(() => {
      expect(navigate).toHaveBeenCalledWith({ selected: undefined });
    });
    // list も detail も走らせない（enabled = false、detail 空振り防止）
    expect(mockSearch).not.toHaveBeenCalled();
    expect(mockDetail).not.toHaveBeenCalled();
  });

  describe("SPEC-UIBB-9（部門候補 = listDepartments master 全件、DSR-10）", () => {
    it("部門候補は listDepartments 全件で page/q/dept/status の 4 条件に依存しない", async () => {
      mockListDepartments.mockResolvedValue({
        status: "ok",
        data: [
          makeMockDepartment({ id: 1, name: "毛糸" }),
          makeMockDepartment({ id: 2, name: "布" }),
        ],
      });
      mockSearch.mockResolvedValue({
        status: "ok",
        data: {
          items: [makeMockProductWithRelations({ product_code: "P-001", department_id: 1 })],
          total_count: 1,
          page: 1,
          per_page: 50,
        },
      });
      mockLowStock.mockResolvedValue({ status: "ok", data: [] });
      const navigate = vi.fn();
      const qc = new QueryClient({
        defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
      });
      const { result, rerender } = renderHook(
        (args: Parameters<typeof useStockInquiry>[0]) => useStockInquiry(args),
        {
          wrapper: makeWrapper(qc),
          initialProps: {
            status: "all",
            q: "",
            dept: null,
            page: 1,
            perPage: 50,
            selected: null,
            navigate,
          },
        },
      );
      await waitFor(() => {
        expect(result.current.departmentOptionsQuery.isSuccess).toBe(true);
      });
      const expected = [
        { id: 1, name: "毛糸" },
        { id: 2, name: "布" },
      ];
      expect(result.current.departmentOptions).toEqual(expected);

      // page を変更
      rerender({
        status: "all",
        q: "",
        dept: null,
        page: 2,
        perPage: 50,
        selected: null,
        navigate,
      });
      // q を変更
      rerender({
        status: "all",
        q: "毛糸",
        dept: null,
        page: 2,
        perPage: 50,
        selected: null,
        navigate,
      });
      // dept を選択
      rerender({
        status: "all",
        q: "毛糸",
        dept: 1,
        page: 2,
        perPage: 50,
        selected: null,
        navigate,
      });
      // status を変更（all → low_stock → stockout）
      rerender({
        status: "low_stock",
        q: "毛糸",
        dept: 1,
        page: 2,
        perPage: 50,
        selected: null,
        navigate,
      });
      rerender({
        status: "stockout",
        q: "毛糸",
        dept: 1,
        page: 2,
        perPage: 50,
        selected: null,
        navigate,
      });

      await waitFor(() => {
        expect(result.current.departmentOptions).toEqual(expected);
      });
      // query key 安定性の mutant 検出（round 2 P2-3）: 4 条件変更後も listDepartments は 1 回のみ
      expect(mockListDepartments).toHaveBeenCalledTimes(1);
    });

    it("選択中部門から別部門へ直接切替できる候補が維持される", async () => {
      mockListDepartments.mockResolvedValue({
        status: "ok",
        data: [
          makeMockDepartment({ id: 1, name: "毛糸" }),
          makeMockDepartment({ id: 2, name: "布" }),
        ],
      });
      mockSearch.mockResolvedValue({
        status: "ok",
        data: { items: [], total_count: 0, page: 1, per_page: 50 },
      });
      const navigate = vi.fn();
      const { result } = renderHook(
        () =>
          useStockInquiry({
            status: "all",
            q: "糸",
            dept: 1,
            page: 1,
            perPage: 50,
            selected: null,
            navigate,
          }),
        { wrapper: makeWrapper() },
      );
      await waitFor(() => {
        expect(result.current.departmentOptions).toEqual([
          { id: 1, name: "毛糸" },
          { id: 2, name: "布" },
        ]);
      });
      // dept=1 選択中でも他部門（id=2）が候補から消えていない
      expect(result.current.departmentOptions.some((option) => option.id === 2)).toBe(true);
    });
  });

  describe("query-keys unit（round 2 P1-1）", () => {
    it("SPEC-UIBB-9: departmentOptionsは無引数で一定keyを返す", () => {
      const key1 = queryKeys.stockInquiry.departmentOptions();
      const key2 = queryKeys.stockInquiry.departmentOptions();
      expect(key1).toEqual(["stock-inquiry", "department-options"]);
      expect(key2).toEqual(key1);
    });
  });
});
