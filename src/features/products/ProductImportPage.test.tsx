import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { commands, type ImportPreview, type ImportRow } from "@/lib/bindings";
import { d052InvalidationOracle, expectExactInvalidations } from "@/test/invalidation-oracle";
import { ProductImportPage } from "./ProductImportPage";

vi.mock("@tanstack/react-router", () => ({
  Link: ({ to, children }: { to: string; children: ReactNode }) => <a href={to}>{children}</a>,
}));

vi.mock("sonner", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

vi.mock("@/lib/bindings", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/bindings")>();
  return {
    ...actual,
    commands: {
      previewImport: vi.fn(),
      commitImport: vi.fn(),
    },
  };
});

const mockPreviewImport = vi.mocked(commands.previewImport);
const mockCommitImport = vi.mocked(commands.commitImport);

function renderWithClient(ui: ReactNode) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  return {
    queryClient,
    ...render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>),
  };
}

function makeImportRow(overrides: Partial<ImportRow> = {}): ImportRow {
  return {
    line_no: 2,
    product_code: "P-001",
    name: "はさみ",
    department_id: 1,
    selling_price: 500,
    cost_price_centi: 300,
    tax_rate: "10",
    stock_unit: null,
    initial_stock: null,
    jan_code: null,
    maker_code: null,
    supplier_id: null,
    pos_stock_sync: null,
    ...overrides,
  };
}

function makePreview(overrides: Partial<ImportPreview> = {}): ImportPreview {
  return {
    valid_rows: [makeImportRow()],
    duplicate_rows: [],
    error_rows: [],
    ...overrides,
  };
}

function createDeferred<T>() {
  let resolve: (value: T) => void = () => {
    throw new Error("deferred promise is not initialized");
  };
  const promise = new Promise<T>((res) => {
    resolve = res;
  });
  return { promise, resolve };
}

function uploadCsv() {
  const file = new File(["商品コード,商品名\nP-001,はさみ"], "products.csv", {
    type: "text/csv",
  });
  fireEvent.drop(screen.getByTestId("file-picker-dropzone"), {
    dataTransfer: { files: [file] },
  });
}

beforeEach(() => {
  mockPreviewImport.mockReset();
  mockCommitImport.mockReset();
});

describe("ProductImportPage (UI-01c / REQ-104)", () => {
  it("UI-01c-D8: 有効行があれば商品インポートを確定できる", async () => {
    const user = userEvent.setup();
    const preview = makePreview();
    mockPreviewImport.mockResolvedValue({ status: "ok", data: preview });
    mockCommitImport.mockResolvedValue({
      status: "ok",
      data: { created_count: 1, updated_count: 0, skipped_count: 0 },
    });

    const { queryClient } = renderWithClient(<ProductImportPage />);
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    uploadCsv();

    expect(await screen.findByText("P-001")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "インポート実行" }));

    await waitFor(() => {
      expect(mockCommitImport).toHaveBeenCalledWith(preview.valid_rows, []);
    });
    expect(await screen.findByText("インポート完了")).toBeInTheDocument();
    expect(screen.getByText("新規登録")).toBeInTheDocument();
    expect(screen.getByText("上書き更新")).toBeInTheDocument();
    expect(screen.getByText("スキップ")).toBeInTheDocument();
    expect(screen.getByText("1 件")).toBeInTheDocument();
    expect(screen.getAllByText("0 件")).toHaveLength(2);
    await waitFor(() => {
      expectExactInvalidations(invalidateSpy.mock.calls, d052InvalidationOracle.productImport());
    });
  });

  it("UI-01c-D12: commit 中は離脱導線と再選択を出さない", async () => {
    const user = userEvent.setup();
    const preview = makePreview();
    const commitDeferred = createDeferred<{
      status: "ok";
      data: { created_count: number; updated_count: number; skipped_count: number };
    }>();
    mockPreviewImport.mockResolvedValue({ status: "ok", data: preview });
    mockCommitImport.mockReturnValue(commitDeferred.promise);

    renderWithClient(<ProductImportPage />);
    uploadCsv();
    await user.click(await screen.findByRole("button", { name: "インポート実行" }));

    await waitFor(() => {
      expect(screen.queryByRole("link", { name: "商品一覧へ戻る" })).not.toBeInTheDocument();
      expect(screen.getByRole("button", { name: "インポート中..." })).toBeDisabled();
      expect(
        screen.getByRole("button", { name: "ファイルを選び直す（商品マスタCSV）" }),
      ).toBeDisabled();
    });

    act(() => {
      commitDeferred.resolve({
        status: "ok",
        data: { created_count: 1, updated_count: 0, skipped_count: 0 },
      });
    });
  });

  it("UI-01c-D7: 選択した重複行だけを commit 対象と overwriteCodes に含める", async () => {
    const user = userEvent.setup();
    const duplicateRow = makeImportRow({ line_no: 3, product_code: "P-002", name: "布" });
    const preview = makePreview({
      duplicate_rows: [
        {
          line_no: 3,
          import_row: duplicateRow,
          existing_product_code: "P-002",
        },
      ],
    });
    mockPreviewImport.mockResolvedValue({ status: "ok", data: preview });
    mockCommitImport.mockResolvedValue({
      status: "ok",
      data: { created_count: 1, updated_count: 1, skipped_count: 0 },
    });

    renderWithClient(<ProductImportPage />);
    uploadCsv();

    await user.click(await screen.findByLabelText("P-002 を上書き"));
    await user.click(screen.getByRole("button", { name: "インポート実行" }));
    await user.click(await screen.findByRole("button", { name: "上書きして実行" }));

    await waitFor(() => {
      expect(mockCommitImport).toHaveBeenCalledWith(
        [...preview.valid_rows, duplicateRow],
        ["P-002"],
      );
    });
  });

  it("J3 commit rejection shows the named row and keeps the preview", async () => {
    // 60 の D-118 の節: 確定の TX で行が拒まれたら Alert に BIZ の文をそのまま出し、
    // 「戻る」で preview と上書きの選択に戻る（kind validation は preview へ戻る今の分岐）
    const user = userEvent.setup();
    const duplicateRow = makeImportRow({ line_no: 3, product_code: "P-002", name: "布" });
    const preview = makePreview({
      duplicate_rows: [{ line_no: 3, import_row: duplicateRow, existing_product_code: "P-002" }],
    });
    const message =
      "商品 P-002 の行: 同じ JAN の商品（P-009）があるため、在庫連動にできません。読み込みの後に登録か売上が変わったか、上書きを選んだほかの行と重なったため、取込みを止めました（どの行も取り込んでいません）。その行をスキップにするか、ファイルを選び直してもう一度読み込んでください";
    mockPreviewImport.mockResolvedValue({ status: "ok", data: preview });
    mockCommitImport.mockResolvedValue({
      status: "error",
      error: {
        kind: "validation",
        message,
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });

    renderWithClient(<ProductImportPage />);
    uploadCsv();
    await user.click(await screen.findByLabelText("P-002 を上書き"));
    await user.click(screen.getByRole("button", { name: "インポート実行" }));
    await user.click(await screen.findByRole("button", { name: "上書きして実行" }));

    expect(await screen.findByText("インポートに失敗しました")).toBeInTheDocument();
    expect(screen.getByText(message)).toBeInTheDocument();
    expect(screen.queryByText("インポート完了")).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "戻る" }));
    expect(await screen.findByLabelText("P-002 を上書き")).toBeChecked();
    expect(screen.getByText("P-001")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "インポート実行" })).toBeEnabled();
  });

  it("UI-01c-D8: エラー行だけの preview では commit を実行しない", async () => {
    mockPreviewImport.mockResolvedValue({
      status: "ok",
      data: makePreview({
        valid_rows: [],
        duplicate_rows: [],
        error_rows: [{ line_no: 2, raw_data: {}, errors: ["商品コードが空です"] }],
      }),
    });

    renderWithClient(<ProductImportPage />);
    uploadCsv();

    expect(await screen.findByText("登録できる行がありません")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "インポート実行" })).toBeDisabled();
    expect(mockCommitImport).not.toHaveBeenCalled();
  });
});

it("⑮ SC4: 確定済みの操作説明を全文表示する", () => {
  renderWithClient(<ProductImportPage />);
  expect(
    screen.getByText(
      "CSVファイルから複数の商品をまとめて登録・更新するページです。ファイルを選ぶと新規登録候補・既存商品との重複・エラー行の3つに分けて内容を確認でき、重複行は初期状態でスキップされるので上書きする行だけ個別に選んで取り込みます。取り込みを実行すると、新規登録・上書き更新・スキップの件数が画面に表示されます。",
    ),
  ).toBeInTheDocument();
});
