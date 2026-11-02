// src/features/products/ProductFormPage.test.tsx

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { commands } from "@/lib/bindings";
import { d052InvalidationOracle, expectExactInvalidations } from "@/test/invalidation-oracle";
import { toast } from "sonner";
import {
  makeMockDepartment,
  makeMockProductWithRelations,
  makeMockSupplier,
} from "./lib/test-fixtures";
import { ProductFormPage } from "./ProductFormPage";

// U4: 離脱の確認に渡した dirty を記録する（何も変えないと false のまま）
const unsavedDirty = vi.hoisted(() => [] as boolean[]);
vi.mock("@/hooks/useUnsavedChangesWarning", () => ({
  useUnsavedChangesWarning: (dirty: boolean) => {
    unsavedDirty.push(dirty);
    return {
      isBlocked: false,
      continueEditing: vi.fn(),
      discardAndProceed: vi.fn(),
    };
  },
}));

vi.mock("@/lib/bindings", () => ({
  commands: {
    listDepartments: vi.fn(),
    listSuppliers: vi.fn(),
    getProduct: vi.fn(),
    createProduct: vi.fn(),
    updateProduct: vi.fn(),
    toggleDiscontinue: vi.fn(),
  },
}));

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn(), dismiss: vi.fn() },
}));

const mockToastSuccess = vi.mocked(toast.success);

/** BIZ の拒否の message（30）。画面は出さない（51） */
const BIZ_RECOUNT_MESSAGE =
  "連動しない間の売上があります。今の数を入れてから在庫連動にしてください";

const mockListDepartments = vi.mocked(commands.listDepartments);
const mockListSuppliers = vi.mocked(commands.listSuppliers);
const mockGetProduct = vi.mocked(commands.getProduct);
const mockCreateProduct = vi.mocked(commands.createProduct);
const mockUpdateProduct = vi.mocked(commands.updateProduct);
const mockToggleDiscontinue = vi.mocked(commands.toggleDiscontinue);

function renderWithClient(ui: ReactNode) {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  return {
    queryClient: qc,
    ...render(<QueryClientProvider client={qc}>{ui}</QueryClientProvider>),
  };
}

beforeEach(() => {
  mockListDepartments.mockReset();
  mockListSuppliers.mockReset();
  mockGetProduct.mockReset();
  mockCreateProduct.mockReset();
  mockUpdateProduct.mockReset();
  mockToggleDiscontinue.mockReset();
  mockToastSuccess.mockReset();
});

describe("ProductFormPage (UI-01b)", () => {
  it("REQ-907 shows the assigned register memory number as read-only", async () => {
    mockListDepartments.mockResolvedValue({
      status: "ok",
      data: [makeMockDepartment()],
    });
    mockListSuppliers.mockResolvedValue({ status: "ok", data: [] });
    mockGetProduct.mockResolvedValue({
      status: "ok",
      data: makeMockProductWithRelations({ plu_memory_no: 217 }),
    });

    renderWithClient(
      <ProductFormPage mode="edit" productCode="P-001" onNavigateToList={vi.fn()} />,
    );
    const memory = await screen.findByLabelText("レジメモリNo.");
    expect(memory).toHaveValue("217");
    expect(memory).toHaveAttribute("readonly");
  });

  it("REQ-907 shows unassigned for a product without a register slot", async () => {
    mockListDepartments.mockResolvedValue({ status: "ok", data: [makeMockDepartment()] });
    mockListSuppliers.mockResolvedValue({ status: "ok", data: [] });
    mockGetProduct.mockResolvedValue({
      status: "ok",
      data: makeMockProductWithRelations({ plu_memory_no: null }),
    });
    renderWithClient(
      <ProductFormPage mode="edit" productCode="P-001" onNavigateToList={vi.fn()} />,
    );
    expect(await screen.findByLabelText("レジメモリNo.")).toHaveValue("未割当");
  });

  it("blocks create submit when JAN is blank and department has no code_prefix", async () => {
    const user = userEvent.setup();
    mockListDepartments.mockResolvedValue({
      status: "ok",
      data: [makeMockDepartment({ id: 2, name: "通常部門", code_prefix: null })],
    });
    mockListSuppliers.mockResolvedValue({ status: "ok", data: [] });

    renderWithClient(<ProductFormPage mode="create" onNavigateToList={vi.fn()} />);

    await user.type(await screen.findByLabelText(/^商品名/), "テスト商品");
    await user.click(screen.getByLabelText(/^部門/));
    await user.click(await screen.findByRole("option", { name: "通常部門" }));
    await user.clear(screen.getByLabelText(/^売価/));
    await user.type(screen.getByLabelText(/^売価/), "500");
    await user.clear(screen.getByLabelText(/^原価/));
    await user.type(screen.getByLabelText(/^原価/), "300");
    await user.click(screen.getByRole("button", { name: "登録する" }));

    expect(await screen.findByText(/独自コード発番対象/)).toBeInTheDocument();
    expect(mockCreateProduct).not.toHaveBeenCalled();
  });

  it("allows no-supplier create when supplier options fail", async () => {
    const user = userEvent.setup();
    const onNavigateToList = vi.fn();
    mockListDepartments.mockResolvedValue({
      status: "ok",
      data: [makeMockDepartment({ id: 1, code_prefix: "Y" })],
    });
    mockListSuppliers.mockResolvedValue({
      status: "error",
      error: {
        kind: "internal",
        message: "取引先取得失敗",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    mockCreateProduct.mockResolvedValue({
      status: "ok",
      data: { product_code: "Y-0001", warnings: [] },
    });

    const { queryClient } = renderWithClient(
      <ProductFormPage
        mode="create"
        returnTo="/products?q=布"
        onNavigateToList={onNavigateToList}
      />,
    );
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await user.type(await screen.findByLabelText(/^商品名/), "テスト商品");
    await user.click(screen.getByLabelText(/^部門/));
    await user.click(await screen.findByRole("option", { name: "毛糸（独自コード可）" }));
    await user.clear(screen.getByLabelText(/^売価/));
    await user.type(screen.getByLabelText(/^売価/), "500");
    await user.clear(screen.getByLabelText(/^原価/));
    await user.type(screen.getByLabelText(/^原価/), "300");
    await user.click(screen.getByRole("button", { name: "登録する" }));

    await waitFor(() => {
      expect(mockCreateProduct).toHaveBeenCalledWith(
        expect.objectContaining({ supplier_id: null }),
      );
    });
    await waitFor(() => {
      expect(onNavigateToList).toHaveBeenCalledWith("/products?q=%E5%B8%83");
    });
    // UI-01b-D14: 保存成功 toast（id 固定）
    expect(mockToastSuccess).toHaveBeenCalledWith(
      expect.stringContaining("Y-0001"),
      expect.objectContaining({ id: "product-save-success" }),
    );
    expectExactInvalidations(invalidateSpy.mock.calls, d052InvalidationOracle.productCreate());
  });

  it("blocks save when department options fail", async () => {
    mockListDepartments.mockResolvedValue({
      status: "error",
      error: {
        kind: "internal",
        message: "部門取得失敗",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    mockListSuppliers.mockResolvedValue({ status: "ok", data: [] });

    renderWithClient(<ProductFormPage mode="create" onNavigateToList={vi.fn()} />);

    expect(await screen.findByText("部門候補を取得できませんでした")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "登録する" })).toBeDisabled();
  });

  it("shows recovery when edit target is not found", async () => {
    const user = userEvent.setup();
    const onNavigateToList = vi.fn();
    mockListDepartments.mockResolvedValue({ status: "ok", data: [makeMockDepartment()] });
    mockListSuppliers.mockResolvedValue({ status: "ok", data: [] });
    mockGetProduct.mockResolvedValue({
      status: "error",
      error: {
        kind: "not_found",
        message: "商品が見つかりません",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });

    renderWithClient(
      <ProductFormPage
        mode="edit"
        productCode="P-MISSING"
        returnTo="/products?page=2"
        onNavigateToList={onNavigateToList}
      />,
    );

    expect(await screen.findByText("商品が見つかりません")).toBeInTheDocument();
    expect(screen.getByText("商品一覧へ戻って、もう一度選択してください。")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "一覧へ戻る" }));

    expect(onNavigateToList).toHaveBeenCalledWith("/products?page=2");
  });

  it("edit save sends only supported changed fields and preserves form on duplicate error", async () => {
    const user = userEvent.setup();
    mockListDepartments.mockResolvedValue({ status: "ok", data: [makeMockDepartment()] });
    mockListSuppliers.mockResolvedValue({ status: "ok", data: [makeMockSupplier({ id: 1 })] });
    mockGetProduct.mockResolvedValue({
      status: "ok",
      data: makeMockProductWithRelations({ product_code: "P-001", name: "変更前" }),
    });
    mockUpdateProduct.mockResolvedValue({
      status: "error",
      error: {
        kind: "duplicate",
        message: "この商品コードは既に使用されています",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });

    const { queryClient } = renderWithClient(
      <ProductFormPage mode="edit" productCode="P-001" onNavigateToList={vi.fn()} />,
    );
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    const name = await screen.findByLabelText(/^商品名/);
    await user.clear(name);
    await user.type(name, "変更後");
    await user.click(screen.getByRole("button", { name: "保存する" }));

    await waitFor(() => {
      expect(mockUpdateProduct).toHaveBeenCalledWith("P-001", { name: "変更後" });
    });
    expect(await screen.findByText("この商品コードは既に使用されています")).toBeInTheDocument();
    expect(screen.getByDisplayValue("変更後")).toBeInTheDocument();
    expect(invalidateSpy).not.toHaveBeenCalled();
  });

  it("UI-01b-D23: 単位を変えて保存すると stock_unit を送り、拒否は保存の error に出る", async () => {
    const user = userEvent.setup();
    mockListDepartments.mockResolvedValue({ status: "ok", data: [makeMockDepartment()] });
    mockListSuppliers.mockResolvedValue({ status: "ok", data: [] });
    mockGetProduct.mockResolvedValue({
      status: "ok",
      data: makeMockProductWithRelations({ product_code: "P-001", stock_unit: "pcs" }),
    });
    mockUpdateProduct.mockResolvedValue({
      status: "error",
      error: {
        kind: "validation",
        message: "在庫単位は個数の単位どうしでだけ変えられます",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });

    renderWithClient(
      <ProductFormPage mode="edit" productCode="P-001" onNavigateToList={vi.fn()} />,
    );

    await user.click(await screen.findByLabelText("数量単位"));
    await user.click(await screen.findByRole("option", { name: "玉" }));
    await user.click(screen.getByRole("button", { name: "保存する" }));

    await waitFor(() => {
      expect(mockUpdateProduct).toHaveBeenCalledWith("P-001", { stock_unit: "ball" });
    });
    expect(
      await screen.findByText("在庫単位は個数の単位どうしでだけ変えられます"),
    ).toBeInTheDocument();
  });

  it("D-052-C2a update success invalidates the exact independent oracle set", async () => {
    const user = userEvent.setup();
    const onNavigateToList = vi.fn();
    mockListDepartments.mockResolvedValue({ status: "ok", data: [makeMockDepartment()] });
    mockListSuppliers.mockResolvedValue({ status: "ok", data: [] });
    mockGetProduct.mockResolvedValue({
      status: "ok",
      data: makeMockProductWithRelations({ product_code: "P-001", name: "変更前" }),
    });
    mockUpdateProduct.mockResolvedValue({ status: "ok", data: { warnings: [] } });

    const { queryClient } = renderWithClient(
      <ProductFormPage mode="edit" productCode="P-001" onNavigateToList={onNavigateToList} />,
    );
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    const name = await screen.findByLabelText(/^商品名/);
    await user.clear(name);
    await user.type(name, "変更後");
    await user.click(screen.getByRole("button", { name: "保存する" }));

    await waitFor(() => {
      expect(onNavigateToList).toHaveBeenCalled();
      expectExactInvalidations(
        invalidateSpy.mock.calls,
        d052InvalidationOracle.productUpdate("P-001"),
      );
    });
  });

  it("D-052-C2b discontinue toggle invalidates the same exact update oracle set", async () => {
    const user = userEvent.setup();
    mockListDepartments.mockResolvedValue({ status: "ok", data: [makeMockDepartment()] });
    mockListSuppliers.mockResolvedValue({ status: "ok", data: [] });
    mockGetProduct.mockResolvedValue({
      status: "ok",
      data: makeMockProductWithRelations({ product_code: "P-001", name: "廃番対象" }),
    });
    mockToggleDiscontinue.mockResolvedValue({ status: "ok", data: true });

    const { queryClient } = renderWithClient(
      <ProductFormPage mode="edit" productCode="P-001" onNavigateToList={vi.fn()} />,
    );
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await user.click(await screen.findByRole("button", { name: "廃番にする" }));
    await user.click(screen.getByRole("button", { name: "廃番にする" }));

    await waitFor(() => {
      expect(mockToggleDiscontinue).toHaveBeenCalledWith("P-001");
      expectExactInvalidations(
        invalidateSpy.mock.calls,
        d052InvalidationOracle.productUpdate("P-001"),
      );
    });
  });

  it("D-052-C2b restore toggle invalidates the same exact update oracle set", async () => {
    const user = userEvent.setup();
    mockListDepartments.mockResolvedValue({ status: "ok", data: [makeMockDepartment()] });
    mockListSuppliers.mockResolvedValue({ status: "ok", data: [] });
    mockGetProduct.mockResolvedValue({
      status: "ok",
      data: makeMockProductWithRelations({
        product_code: "P-001",
        name: "復帰対象",
        is_discontinued: true,
      }),
    });
    mockToggleDiscontinue.mockResolvedValue({ status: "ok", data: false });

    const { queryClient } = renderWithClient(
      <ProductFormPage mode="edit" productCode="P-001" onNavigateToList={vi.fn()} />,
    );
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await user.click(await screen.findByRole("button", { name: "表示に戻す" }));

    await waitFor(() => {
      expect(mockToggleDiscontinue).toHaveBeenCalledWith("P-001");
      expectExactInvalidations(
        invalidateSpy.mock.calls,
        d052InvalidationOracle.productUpdate("P-001"),
      );
    });
  });

  // 51 の D-118 の節: recount_required の拒否（mock の message は BIZ の文で、画面の文と違う）
  function mockRecountRequiredRejection() {
    mockListDepartments.mockResolvedValue({ status: "ok", data: [makeMockDepartment()] });
    mockListSuppliers.mockResolvedValue({ status: "ok", data: [] });
    mockGetProduct.mockResolvedValue({
      status: "ok",
      data: makeMockProductWithRelations({ product_code: "P-005", pos_stock_sync: false }),
    });
    mockUpdateProduct.mockResolvedValue({
      status: "error",
      error: {
        kind: "stocktake_guard",
        message: BIZ_RECOUNT_MESSAGE,
        field: null,
        error_id: null,
        stocktake_recovery: {
          code: "recount_required",
          targets: [
            {
              product_code: "P-005",
              product_name: "連動なしの糸",
              stock_unit: "pcs",
              stock: 7,
              stock_range_low: null,
              stock_range_high: null,
              stocktake_item_id: null,
              stocktake_id: null,
              action: "count_now",
              block_reason: null,
              recount_reasons: [],
            },
          ],
          settlement_pending: null,
        },
      },
    });
  }

  async function saveWithPosSync() {
    const user = userEvent.setup();
    const checkbox = await screen.findByLabelText("POS販売で在庫を減らす");
    expect(checkbox).not.toBeChecked();
    await user.click(checkbox);
    await user.click(screen.getByRole("button", { name: "保存する" }));
    await waitFor(() => {
      expect(mockUpdateProduct).toHaveBeenCalledWith("P-005", { pos_stock_sync: true });
    });
    return user;
  }

  it("SF2 (U12) stock link start rejection names the POS field and offers the recount link (51)", async () => {
    const onNavigateToList = vi.fn();
    const onNavigateToStocktake = vi.fn();
    mockRecountRequiredRejection();
    renderWithClient(
      <ProductFormPage
        mode="edit"
        productCode="P-005"
        onNavigateToList={onNavigateToList}
        onNavigateToStocktake={onNavigateToStocktake}
      />,
    );
    const user = await saveWithPosSync();

    const notice = await screen.findByRole("alert", {
      name: "「POS販売で在庫を減らす」にできません",
    });
    expect(
      within(notice).getByText(
        "「減らさない」の間の売上が在庫の数に入っているか確認できません。「今の数を入れる」で棚の数を入れてから、もう一度「減らす」にして保存してください",
      ),
    ).toBeInTheDocument();
    expect(screen.queryByText(BIZ_RECOUNT_MESSAGE)).not.toBeInTheDocument();
    expect(within(notice).queryByText(/在庫連動/)).not.toBeInTheDocument();
    expect(screen.getByLabelText("POS販売で在庫を減らす")).toBeChecked();
    expect(mockToastSuccess).not.toHaveBeenCalled();
    expect(onNavigateToList).not.toHaveBeenCalled();

    await user.click(within(notice).getByRole("button", { name: "今の数を入れる" }));
    expect(onNavigateToStocktake).toHaveBeenCalledWith({
      product: "P-005",
      returnTo: "/products/P-005/edit",
    });
  });

  it("SF3 recount notice renders right after the POS field inside the stock section and takes focus (51)", async () => {
    mockRecountRequiredRejection();
    renderWithClient(
      <ProductFormPage
        mode="edit"
        productCode="P-005"
        onNavigateToList={vi.fn()}
        onNavigateToStocktake={vi.fn()}
      />,
    );
    await saveWithPosSync();

    const notices = await screen.findAllByRole("alert", {
      name: "「POS販売で在庫を減らす」にできません",
    });
    expect(notices).toHaveLength(1);
    const notice = notices[0];
    const checkbox = screen.getByLabelText("POS販売で在庫を減らす");
    // 欄の後ろ（DOM で checkbox に続く）で、同じ在庫の節の中
    expect(
      checkbox.compareDocumentPosition(notice) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    const section = checkbox.closest("section");
    expect(section).not.toBeNull();
    expect(section?.contains(notice)).toBe(true);
    // focus は案内の領域で、導線の button ではない
    await waitFor(() => {
      expect(notice).toHaveFocus();
    });
    expect(within(notice).getByRole("button", { name: "今の数を入れる" })).not.toHaveFocus();
  });
});

describe("ProductFormPage の現在庫（D-118 b2、51）", () => {
  it("U4 edit form shows read-only stock range and recheck badge without dirty state", async () => {
    // REQ-102 / REQ-301 / SPEC-STK-CNT-D3（Matrix U4）: 修正の画面の読取り専用の「現在庫」に範囲、label の隣に badge。
    // 何も変えなければ dirty にならない（離脱の確認が出ない）
    mockListDepartments.mockResolvedValue({ status: "ok", data: [makeMockDepartment()] });
    mockListSuppliers.mockResolvedValue({ status: "ok", data: [] });
    mockGetProduct.mockResolvedValue({
      status: "ok",
      data: makeMockProductWithRelations({
        stock_quantity: 9,
        stock_recheck: true,
        stock_range_low: 7,
        stock_range_high: 12,
      }),
    });
    unsavedDirty.length = 0;
    renderWithClient(
      <ProductFormPage mode="edit" productCode="P-001" onNavigateToList={vi.fn()} />,
    );

    const stock = await screen.findByLabelText("現在庫");
    await waitFor(() => {
      expect(stock).toHaveValue("7〜12 個");
    });
    expect(stock).toHaveAttribute("readonly");
    const label = screen.getByText("現在庫", { selector: "label" });
    const badge = within(label.parentElement ?? label).getByText("要再確認");
    expect(badge).toHaveAttribute("data-tone", "warning");
    expect(unsavedDirty[unsavedDirty.length - 1]).toBe(false);
  });

  it("U4 create form has no recheck badge", async () => {
    // REQ-101（Matrix U4）: 登録の画面は今のまま
    mockListDepartments.mockResolvedValue({ status: "ok", data: [makeMockDepartment()] });
    mockListSuppliers.mockResolvedValue({ status: "ok", data: [] });
    renderWithClient(<ProductFormPage mode="create" onNavigateToList={vi.fn()} />);
    expect(await screen.findByLabelText("初期在庫（必須）")).toHaveValue("0");
    expect(screen.queryByText("要再確認")).not.toBeInTheDocument();
  });
});

describe("ProductFormPage の JAN を他の商品が使っているときのダイアログ（UI-01b-D26、D-141）", () => {
  const JAN = "4901234567887";
  const NAME = "青の布 2 色目";
  // BIZ の message（30 BIZ-01-D11）は mock の値をそのまま出す。画面の文は 51 の文字どおりの literal で確かめる
  const JAN_IN_USE_MESSAGE = `この JAN（${JAN}）はほかの商品が使っています: P-901「赤」（POS販売で在庫を減らす）`;
  const SECOND_TEXT =
    "色やサイズの違う別の品なら、独自コードで登録できます。JAN はこの商品にも残ります。JAN が同じ商品はレジの売上でどれが売れたか見分けられないため、この商品と、JAN が同じ商品は全て「POS販売で在庫を減らす」が「減らさない」になります（在庫は棚卸しと手入力で合わせます）。「商品コードだけ同じ」の商品は変わりません。";
  const THIRD_TEXT =
    "同じ品なら、新しく登録せずに、商品一覧で JAN を検索してその商品を直してください（廃番なら「表示に戻す」）。";
  const WARNING =
    "同じ JAN の商品 P-901「赤」の「POS販売で在庫を減らす」を「減らさない」にしました";

  function cmdError(kind: "duplicate" | "validation", message: string, field: string | null) {
    return {
      status: "error" as const,
      error: { kind, message, field, error_id: null, stocktake_recovery: null },
    };
  }

  function mockOptions() {
    mockListDepartments.mockResolvedValue({
      status: "ok",
      data: [
        makeMockDepartment({ id: 1, name: "布", code_prefix: "HZ" }),
        makeMockDepartment({ id: 2, name: "通常部門", code_prefix: null }),
      ],
    });
    mockListSuppliers.mockResolvedValue({ status: "ok", data: [] });
  }

  async function submitWithJan(departmentOption: string) {
    const user = userEvent.setup();
    await user.type(await screen.findByLabelText(/^商品名/), NAME);
    await user.click(screen.getByLabelText(/^部門/));
    await user.click(await screen.findByRole("option", { name: departmentOption }));
    await user.type(screen.getByLabelText(/^JANコード/), JAN);
    await user.clear(screen.getByLabelText(/^売価/));
    await user.type(screen.getByLabelText(/^売価/), "500");
    await user.clear(screen.getByLabelText(/^原価/));
    await user.type(screen.getByLabelText(/^原価/), "300");
    await user.click(screen.getByRole("button", { name: "登録する" }));
    return user;
  }

  function renderCreate(onNavigateToList = vi.fn()) {
    mockOptions();
    const rendered = renderWithClient(
      <ProductFormPage mode="create" onNavigateToList={onNavigateToList} />,
    );
    const invalidateSpy = vi.spyOn(rendered.queryClient, "invalidateQueries");
    return { invalidateSpy, onNavigateToList };
  }

  it("U2 REQ-101 / UI-01b-D26 opens the dialog on duplicate with field jan_code and keeps the input", async () => {
    mockCreateProduct.mockResolvedValueOnce(cmdError("duplicate", JAN_IN_USE_MESSAGE, "jan_code"));
    const { onNavigateToList } = renderCreate();
    await submitWithJan("布（独自コード可）");

    const dialog = await screen.findByRole("alertdialog");
    expect(
      within(dialog).getByRole("heading", { name: "この JAN はほかの商品が使っています" }),
    ).toBeInTheDocument();
    expect(within(dialog).getByText(JAN_IN_USE_MESSAGE)).toBeInTheDocument();
    expect(within(dialog).getByText(SECOND_TEXT)).toBeInTheDocument();
    expect(within(dialog).getByText(THIRD_TEXT)).toBeInTheDocument();
    expect(within(dialog).getByRole("button", { name: "独自コードで登録する" })).toBeEnabled();
    expect(within(dialog).getByRole("button", { name: "戻る" })).toBeEnabled();
    expect(onNavigateToList).not.toHaveBeenCalled();
    expect(screen.queryByText("保存できませんでした")).not.toBeInTheDocument();
    expect(screen.getByDisplayValue(NAME)).toBeInTheDocument();
  });

  it("U2 REQ-101 / UI-01b-D26 keeps the buttons outside a height-capped scroll box when many products use the JAN", async () => {
    // 合成の 20 件。happy-dom は layout を測らないので、上限と scroll の class・構造と focus の順だけを見る
    const manyUsers = Array.from(
      { length: 20 },
      (_, i) =>
        `P-${String(900 + i)}「長い名前の布 ${String(i + 1)} 色目」（POS販売で在庫を減らす）`,
    ).join("、");
    const manyMessage = `この JAN（${JAN}）はほかの商品が使っています: ${manyUsers}`;
    mockCreateProduct.mockResolvedValueOnce(cmdError("duplicate", manyMessage, "jan_code"));
    renderCreate();
    const user = await submitWithJan("布（独自コード可）");

    const dialog = await screen.findByRole("alertdialog");
    expect(dialog).toHaveClass("flex", "max-h-[90vh]", "flex-col");
    const scrollBox = within(dialog).getByText(manyMessage).parentElement;
    expect(scrollBox).toHaveClass("min-h-0", "overflow-y-auto");
    // 題を持つ header も縮められる（縮めないと文の箱に上限が掛からない）
    expect(scrollBox?.parentElement).toHaveClass("flex", "min-h-0", "flex-col");
    expect(scrollBox).toContainElement(within(dialog).getByText(THIRD_TEXT));
    for (const name of ["戻る", "独自コードで登録する"]) {
      expect(scrollBox).not.toContainElement(within(dialog).getByRole("button", { name }));
    }
    expect(scrollBox).not.toContainElement(
      within(dialog).getByRole("heading", { name: "この JAN はほかの商品が使っています" }),
    );
    // 本文の箱は名前の付いた region で、Tab と Shift+Tab で入れる（矢印の key の scroll は L3-1）
    expect(within(dialog).getByRole("region", { name: "使っている商品と変わる所" })).toBe(
      scrollBox,
    );
    const back = within(dialog).getByRole("button", { name: "戻る" });
    const confirm = within(dialog).getByRole("button", { name: "独自コードで登録する" });
    await waitFor(() => {
      expect(back).toHaveFocus();
    });
    await user.tab({ shift: true });
    expect(scrollBox).toHaveFocus();
    await user.tab();
    expect(back).toHaveFocus();
    await user.tab();
    expect(confirm).toHaveFocus();
    await user.tab();
    expect(scrollBox).toHaveFocus();
  });

  it("U3 REQ-101 / UI-01b-D26 resends with shared_jan_custom_code and pos_stock_sync false, toasts warnings, invalidates C23", async () => {
    mockCreateProduct
      .mockResolvedValueOnce(cmdError("duplicate", JAN_IN_USE_MESSAGE, "jan_code"))
      .mockResolvedValueOnce({
        status: "ok",
        data: { product_code: "HZ-0012", warnings: [WARNING] },
      });
    // UI-USW-D1: 遷移の時点で baseline が同期済み（離脱の確認に渡る dirty が false）
    unsavedDirty.length = 0;
    let dirtyAtNavigate: boolean | undefined;
    const { invalidateSpy, onNavigateToList } = renderCreate(
      vi.fn(() => {
        dirtyAtNavigate = unsavedDirty[unsavedDirty.length - 1];
      }),
    );
    const user = await submitWithJan("布（独自コード可）");
    const dialog = await screen.findByRole("alertdialog");
    // 在庫連動の checkbox は on のまま
    expect(screen.getByLabelText("POS販売で在庫を減らす")).toBeChecked();

    await user.click(within(dialog).getByRole("button", { name: "独自コードで登録する" }));

    await waitFor(() => {
      expect(onNavigateToList).toHaveBeenCalled();
    });
    expect(dirtyAtNavigate).toBe(false);
    expect(mockCreateProduct).toHaveBeenCalledTimes(2);
    const first = mockCreateProduct.mock.calls[0][0];
    expect(first).toMatchObject({
      jan_code: JAN,
      pos_stock_sync: true,
      shared_jan_custom_code: false,
    });
    expect(mockCreateProduct.mock.calls[1][0]).toEqual({
      ...first,
      shared_jan_custom_code: true,
      pos_stock_sync: false,
    });
    expect(mockToastSuccess).toHaveBeenCalledWith(
      `商品「${NAME}」を登録しました（商品コード: HZ-0012）`,
      expect.objectContaining({ id: "product-save-success" }),
    );
    const options = mockToastSuccess.mock.calls[0][1] as { description?: ReactNode };
    render(<>{options.description}</>);
    expect(screen.getByText(WARNING)).toBeInTheDocument();
    expectExactInvalidations(
      invalidateSpy.mock.calls,
      d052InvalidationOracle.productCreateSharedJan(),
    );
  });

  it("U3 REQ-101 / UI-01b-D26 shows no toast description when warnings are empty", async () => {
    mockCreateProduct
      .mockResolvedValueOnce(cmdError("duplicate", JAN_IN_USE_MESSAGE, "jan_code"))
      .mockResolvedValueOnce({ status: "ok", data: { product_code: "HZ-0013", warnings: [] } });
    const { onNavigateToList } = renderCreate();
    const user = await submitWithJan("布（独自コード可）");
    const dialog = await screen.findByRole("alertdialog");
    await user.click(within(dialog).getByRole("button", { name: "独自コードで登録する" }));

    await waitFor(() => {
      expect(onNavigateToList).toHaveBeenCalled();
    });
    const options = mockToastSuccess.mock.calls[0][1] as { description?: ReactNode };
    expect(options.description).toBeUndefined();
  });

  it("U4 REQ-101 / UI-01b-D26 back closes the dialog without resending and keeps the dirty input", async () => {
    mockCreateProduct.mockResolvedValueOnce(cmdError("duplicate", JAN_IN_USE_MESSAGE, "jan_code"));
    unsavedDirty.length = 0;
    renderCreate();
    const user = await submitWithJan("布（独自コード可）");
    const dialog = await screen.findByRole("alertdialog");

    await user.click(within(dialog).getByRole("button", { name: "戻る" }));

    await waitFor(() => {
      expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    });
    expect(mockCreateProduct).toHaveBeenCalledTimes(1);
    expect(screen.getByDisplayValue(NAME)).toBeInTheDocument();
    expect(screen.getByDisplayValue(JAN)).toBeInTheDocument();
    expect(unsavedDirty[unsavedDirty.length - 1]).toBe(true);
  });

  it("U5 REQ-101 / UI-01b-D26 names the department and hides the primary button without code_prefix", async () => {
    mockCreateProduct.mockResolvedValueOnce(cmdError("duplicate", JAN_IN_USE_MESSAGE, "jan_code"));
    renderCreate();
    await submitWithJan("通常部門");

    const dialog = await screen.findByRole("alertdialog");
    expect(
      within(dialog).queryByRole("button", { name: "独自コードで登録する" }),
    ).not.toBeInTheDocument();
    expect(
      within(dialog).getByText(
        "部門「通常部門」は独自コードを振れません。独自コードを振れる部門を選び直してから、もう一度「登録する」を押してください。",
      ),
    ).toBeInTheDocument();
    expect(within(dialog).getByRole("button", { name: "戻る" })).toBeInTheDocument();
  });

  it("U6 REQ-101 / UI-01b-D26 keeps the form-level alert for duplicate without field", async () => {
    mockCreateProduct.mockResolvedValueOnce(
      cmdError("duplicate", "この商品コードは既に使用されています: X", null),
    );
    renderCreate();
    await submitWithJan("布（独自コード可）");

    expect(await screen.findByText("保存できませんでした")).toBeInTheDocument();
    expect(screen.getByText("この商品コードは既に使用されています: X")).toBeInTheDocument();
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
  });

  it("U7 REQ-101 / UI-01b-D26 closes the dialog and shows the save error when the resend fails", async () => {
    mockCreateProduct
      .mockResolvedValueOnce(cmdError("duplicate", JAN_IN_USE_MESSAGE, "jan_code"))
      .mockResolvedValueOnce(
        cmdError("validation", "この部門の独自コードの番号を振れません", null),
      );
    const { onNavigateToList } = renderCreate();
    const user = await submitWithJan("布（独自コード可）");
    const dialog = await screen.findByRole("alertdialog");

    await user.click(within(dialog).getByRole("button", { name: "独自コードで登録する" }));

    expect(await screen.findByText("この部門の独自コードの番号を振れません")).toBeInTheDocument();
    expect(screen.getByText("保存できませんでした")).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    });
    expect(onNavigateToList).not.toHaveBeenCalled();
    expect(mockToastSuccess).not.toHaveBeenCalled();
  });
});
