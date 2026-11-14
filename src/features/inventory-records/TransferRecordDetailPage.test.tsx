// UI-16b（79 §79.8）: 詰め替え・切り分けの記録の詳細。期待値は 79 §79.8・§79.10 から写す。

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { screen, within } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { commands } from "@/lib/bindings";
import type { TransferRecordDetail } from "@/lib/bindings";
import { renderWithRouter } from "@/test/render-with-router";
import { TransferRecordDetailPage } from "./TransferRecordDetailPage";

vi.mock("@/lib/bindings", () => ({
  commands: {
    getTransferRecord: vi.fn(),
  },
}));

const mockGetTransferRecord = vi.mocked(commands.getTransferRecord);

function renderWithClient(ui: ReactNode) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  return renderWithRouter(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>);
}

// 合成の値だけ（反物 1.5 m → 端切れ 3 個）
function makeDetail(): TransferRecordDetail {
  return {
    id: 7,
    transfer_date: "2026-10-27",
    status: "active",
    created_at: "2026-10-27T10:30:00",
    items: [
      {
        id: 1,
        product_code: "TR-M1",
        product_name: "合成の反物",
        department_name: "生地",
        stock_unit: "m",
        direction: "out",
        quantity: 150,
      },
      {
        id: 2,
        product_code: "TR-01",
        product_name: "合成の端切れ",
        department_name: "生地",
        stock_unit: "pcs",
        direction: "in",
        quantity: 3,
      },
    ],
    movements: [
      {
        id: 11,
        product_code: "TR-M1",
        movement_type: "transfer",
        quantity: -150,
        stock_after: 850,
        reference_type: "transfer_record",
        reference_id: 7,
        source: { label: "詰め替え・切り分け #7", route: "/inventory/transfer/records/7" },
        note: null,
        created_at: "2026-10-27T10:30:00",
      },
      {
        id: 12,
        product_code: "TR-01",
        movement_type: "transfer",
        quantity: 3,
        stock_after: 3,
        reference_type: "transfer_record",
        reference_id: 7,
        source: { label: "詰め替え・切り分け #7", route: "/inventory/transfer/records/7" },
        note: null,
        created_at: "2026-10-27T10:30:00",
      },
    ],
  };
}

beforeEach(() => {
  mockGetTransferRecord.mockReset();
});

describe("TransferRecordDetailPage（REQ-210 / UI-16b / U12）", () => {
  it("U12: 減らした商品と増やした商品の 2 つの表（単位つき）、関連在庫変動、取消の代わりの案内を出す", async () => {
    mockGetTransferRecord.mockResolvedValue({ status: "ok", data: makeDetail() });

    renderWithClient(<TransferRecordDetailPage recordId={7} />);

    expect(
      await screen.findByRole("heading", { name: "詰め替え・切り分け #7" }),
    ).toBeInTheDocument();
    expect(mockGetTransferRecord).toHaveBeenCalledWith(7);
    expect(screen.getByText("2026-10-27")).toBeInTheDocument();
    expect(screen.getByText("有効")).toBeInTheDocument();

    const outTable = screen.getByRole("table", { name: "減らした商品" });
    expect(within(outTable).getByText("TR-M1")).toBeInTheDocument();
    expect(within(outTable).getByText("1.5 m")).toBeInTheDocument();
    expect(within(outTable).queryByText("TR-01")).not.toBeInTheDocument();
    const inTable = screen.getByRole("table", { name: "増やした商品" });
    expect(within(inTable).getByText("TR-01")).toBeInTheDocument();
    expect(within(inTable).getByText("3 個")).toBeInTheDocument();
    expect(within(inTable).queryByText("TR-M1")).not.toBeInTheDocument();
    // 減らす表が増やす表より先
    expect(
      outTable.compareDocumentPosition(inTable) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();

    expect(screen.getByRole("heading", { name: "関連する在庫変動" })).toBeInTheDocument();
    expect(screen.getByText("-1.5 m")).toBeInTheDocument();
    expect(screen.getByText("+3")).toBeInTheDocument();

    expect(
      screen.getByText(
        "間違えたときは、この記録と同じ日付・同じ数量で、減らす商品と増やす商品を入れ替えて、もう一度詰め替え・切り分けを記録してください",
      ),
    ).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /取消|訂正/ })).not.toBeInTheDocument();
    expect(screen.getByRole("link", { name: "前の画面へ戻る" })).toHaveAttribute(
      "href",
      "/inventory/records",
    );
  });

  it("U12: 無い記録は not found の文と戻り導線を出す", async () => {
    mockGetTransferRecord.mockResolvedValue({
      status: "error",
      error: {
        kind: "not_found",
        message: "詰め替え・切り分けの記録が見つかりません",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });

    renderWithClient(<TransferRecordDetailPage recordId={404} />);

    expect(await screen.findByText("詰め替え・切り分けの記録が見つかりません")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "前の画面へ戻る" })).toHaveAttribute(
      "href",
      "/inventory/records",
    );
  });

  it.each([
    ["/inventory/transfer", "/inventory/transfer"],
    ["https://example.invalid/escape", "/inventory/records"],
    ["//example.invalid/escape", "/inventory/records"],
  ])("U11 / DSR-18: returnTo %s を %s へ正規化する", async (returnTo, expected) => {
    mockGetTransferRecord.mockResolvedValue({ status: "ok", data: makeDetail() });

    renderWithClient(<TransferRecordDetailPage recordId={7} returnTo={returnTo} />);

    expect(
      await screen.findByRole("heading", { name: "詰め替え・切り分け #7" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "前の画面へ戻る" })).toHaveAttribute("href", expected);
  });
});
