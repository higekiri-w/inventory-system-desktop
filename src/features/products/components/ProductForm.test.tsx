// src/features/products/components/ProductForm.test.tsx

import React from "react";
import {
  act,
  createEvent,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { commands, type PriceHistoryEntry } from "@/lib/bindings";

vi.mock("sonner", () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

vi.mock("@/lib/bindings", () => ({
  commands: {
    listPriceHistory: vi.fn().mockResolvedValue({ status: "ok", data: [] }),
    createSupplier: vi.fn(),
    listSuppliers: vi.fn().mockResolvedValue({ status: "ok", data: [] }),
  },
}));

import { makeMockDepartment, makeMockSupplier } from "../lib/test-fixtures";
import {
  buildCreateProductRequest,
  createProductFormDefaults,
  productToFormValues,
  type ProductFormValues,
} from "../lib/product-form-request";
import { makeMockProductWithRelations } from "../lib/test-fixtures";
import { ProductForm } from "./ProductForm";

describe("ProductForm (UI-01b)", () => {
  it("shows edit read-only fields and discontinued state text", () => {
    const product = makeMockProductWithRelations({
      product_code: "P-001",
      jan_code: "4901234567890",
      stock_quantity: 20,
      stock_unit: "m",
      is_discontinued: true,
    });

    render(
      <ProductForm
        mode="edit"
        values={productToFormValues(product)}
        departments={[makeMockDepartment()]}
        suppliers={[makeMockSupplier()]}
        errors={{}}
        saveError={null}
        supplierWarning={null}
        productCodeLabel={product.product_code}
        isDiscontinued
        isSaving={false}
        posSyncTouched={false}
        onValuesChange={vi.fn()}
        onPosSyncTouchedChange={vi.fn()}
        onSubmit={vi.fn()}
        onCancel={vi.fn()}
      />,
    );

    expect(screen.getByLabelText("商品コード")).toHaveValue("P-001");
    // UI-01b-D11: read-only 入力は readOnly 属性で示す（長さの単位の商品の数量単位 select は disabled）
    expect(screen.getByLabelText("JANコード")).toHaveAttribute("readonly");
    expect(screen.getByLabelText("現在庫")).toHaveAttribute("readonly");
    expect(screen.getByLabelText("数量単位")).toBeDisabled();
    expect(screen.getByText("廃番")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "表示に戻す" })).toBeInTheDocument();

    // UI-01b-D10: 4 セクション見出し
    expect(screen.getByRole("heading", { name: "商品の識別" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "分類と取引先" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "価格" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "在庫" })).toBeInTheDocument();
  });

  function renderStockSection(mode: "create" | "edit", stockUnit: "pcs" | "m") {
    const product = makeMockProductWithRelations({ stock_unit: stockUnit, stock_quantity: 12 });
    render(
      <ProductForm
        mode={mode}
        values={
          mode === "edit"
            ? productToFormValues(product)
            : { ...createProductFormDefaults, stockUnit }
        }
        departments={[makeMockDepartment()]}
        suppliers={[makeMockSupplier()]}
        errors={{}}
        saveError={null}
        supplierWarning={null}
        productCodeLabel={mode === "edit" ? product.product_code : "自動"}
        isDiscontinued={false}
        isSaving={false}
        posSyncTouched={false}
        onValuesChange={vi.fn()}
        onPosSyncTouchedChange={vi.fn()}
        onSubmit={vi.fn()}
        onCancel={vi.fn()}
      />,
    );
  }

  it("UI-01b-D23: edit の個数の単位の商品は数量単位 select が押せる", () => {
    renderStockSection("edit", "pcs");
    expect(screen.getByLabelText("数量単位")).toBeEnabled();
    expect(screen.getByLabelText("現在庫")).toHaveAttribute("readonly");
  });

  it.each([
    [
      "create",
      "pcs",
      "初期在庫と数量単位を設定します。登録後は、数量単位を個数の単位どうし（例: 個 → 玉）でだけ変更できます。",
    ],
    ["edit", "pcs", "現在庫は変更できません。数量単位は個数の単位どうしでだけ変更できます。"],
    ["edit", "m", "現在庫と数量単位は登録後に変更できません。"],
  ] as const)("UI-01b-D23: 在庫の節の説明文は mode と単位で 3 つ（%s・%s）", (mode, unit, text) => {
    renderStockSection(mode, unit);
    expect(screen.getByText(text)).toBeInTheDocument();
    // 旧い create の文（読点の無い「登録後は数量単位を変更…」）は出ない
    expect(screen.queryByText(/登録後は数量単位を変更/)).toBeNull();
  });

  it("discontinue requires confirmation and cancel keeps state", async () => {
    const user = userEvent.setup();
    const onToggleDiscontinue = vi.fn();
    const product = makeMockProductWithRelations({
      product_code: "P-200",
      name: "在庫商品",
      is_discontinued: false,
    });

    render(
      <ProductForm
        mode="edit"
        values={productToFormValues(product)}
        departments={[makeMockDepartment()]}
        suppliers={[makeMockSupplier()]}
        errors={{}}
        saveError={null}
        supplierWarning={null}
        productCodeLabel={product.product_code}
        productName={product.name}
        isDiscontinued={false}
        isSaving={false}
        posSyncTouched={false}
        onValuesChange={vi.fn()}
        onPosSyncTouchedChange={vi.fn()}
        onSubmit={vi.fn()}
        onCancel={vi.fn()}
        onToggleDiscontinue={onToggleDiscontinue}
      />,
    );

    // UI-01b-D13: 「廃番にする」は確認ダイアログを通す
    await user.click(screen.getByRole("button", { name: "廃番にする" }));
    expect(await screen.findByRole("alertdialog")).toBeInTheDocument();
    expect(
      screen.getByText(/商品「在庫商品」は商品一覧の通常表示から外れます/),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "廃番にする" })).toHaveAttribute(
      "data-variant",
      "destructive",
    );

    // キャンセルで toggle は呼ばれない
    await user.click(screen.getByRole("button", { name: "キャンセル" }));
    expect(onToggleDiscontinue).not.toHaveBeenCalled();
  });

  // REQ-101 / UI-01b-D6: pcs→m→pcs（checkbox 未操作）で posStockSync が true に復元される
  // 自動提案経路（onPosStockSyncSuggest）は onPosSyncTouchedChange を呼ばない。
  // functional update 化後は updater function が渡されるため、prev を適用して実値を検証する。
  it("create pcs→m→pcs without checkbox: posStockSync restores to true (no touched set)", async () => {
    const user = userEvent.setup();
    // updater function を prev に適用して最終的な values を得るヘルパー
    function applyUpdater(
      prev: ProductFormValues,
      updater: ProductFormValues | ((p: ProductFormValues) => ProductFormValues),
    ): ProductFormValues {
      return typeof updater === "function" ? updater(prev) : updater;
    }
    let latestValues = createProductFormDefaults;
    const captureUpdater = (updater: React.SetStateAction<ProductFormValues>) => {
      latestValues = applyUpdater(latestValues, updater);
    };

    const { rerender } = render(
      <ProductForm
        mode="create"
        values={createProductFormDefaults}
        departments={[makeMockDepartment()]}
        suppliers={[makeMockSupplier()]}
        errors={{}}
        saveError={null}
        supplierWarning={null}
        isSaving={false}
        posSyncTouched={false}
        onValuesChange={captureUpdater}
        onPosSyncTouchedChange={vi.fn()}
        onSubmit={vi.fn()}
        onCancel={vi.fn()}
      />,
    );

    // pcs→m: update が 2 回発火する（stockUnit の更新 + suggest による posStockSync 更新）。
    // functional update なので各呼び出しは prev を受け取り、順次適用される。
    await user.click(screen.getByLabelText("数量単位"));
    await user.click(await screen.findByRole("option", { name: "m" }));
    // 2 回の updater を順次適用した結果として両フィールドが更新されている
    expect(latestValues.stockUnit).toBe("m");
    expect(latestValues.posStockSync).toBe(false);

    latestValues = { ...createProductFormDefaults, stockUnit: "m", posStockSync: false };

    // m→pcs: suggest で posStockSync=true が復元される（touched は立たない）
    rerender(
      <ProductForm
        mode="create"
        values={{ ...createProductFormDefaults, stockUnit: "m", posStockSync: false }}
        departments={[makeMockDepartment()]}
        suppliers={[makeMockSupplier()]}
        errors={{}}
        saveError={null}
        supplierWarning={null}
        isSaving={false}
        posSyncTouched={false}
        onValuesChange={captureUpdater}
        onPosSyncTouchedChange={vi.fn()}
        onSubmit={vi.fn()}
        onCancel={vi.fn()}
      />,
    );

    await user.click(screen.getByLabelText("数量単位"));
    await user.click(await screen.findByRole("option", { name: "個" }));
    expect(latestValues.stockUnit).toBe("pcs");
    expect(latestValues.posStockSync).toBe(true);
  });

  // REQ-101 / UI-01b-D6: 統合テスト — stateful wrapper で actual state を検証する
  // ProductForm + stateful wrapper で「数量単位変更 → state が正しく反映される」ことを確認。
  // mock の呼び出し capture だけでは lost update（同一 tick の連続 setState で後勝ち）を検出できないため、
  // 実際の useState で state を持つ wrapper を使う。
  describe("stateful wrapper (lost-update integration)", () => {
    function StatefulProductForm({
      initialPosSyncTouched = false,
    }: {
      initialPosSyncTouched?: boolean;
    }) {
      const [values, setValues] = React.useState<ProductFormValues>(createProductFormDefaults);
      const [posSyncTouched, setPosSyncTouched] = React.useState(initialPosSyncTouched);

      return (
        <>
          <div data-testid="stock-unit">{values.stockUnit}</div>
          <div data-testid="pos-stock-sync">{String(values.posStockSync)}</div>
          <div data-testid="pos-sync-touched">{String(posSyncTouched)}</div>
          <ProductForm
            mode="create"
            values={values}
            departments={[makeMockDepartment()]}
            suppliers={[makeMockSupplier()]}
            errors={{}}
            saveError={null}
            supplierWarning={null}
            isSaving={false}
            posSyncTouched={posSyncTouched}
            onValuesChange={setValues}
            onPosSyncTouchedChange={setPosSyncTouched}
            onSubmit={vi.fn()}
            onCancel={vi.fn()}
          />
        </>
      );
    }

    // REQ-101 / UI-01b-D6: create mode で pcs→m に変更すると
    // stockUnit="m" かつ posStockSync=false に両方更新される（lost update が起きないこと）
    it("pcs→m: both stockUnit and posStockSync update correctly without lost update", async () => {
      const user = userEvent.setup();
      render(<StatefulProductForm />);

      // 初期状態: pcs / posStockSync=true
      expect(screen.getByTestId("stock-unit")).toHaveTextContent("pcs");
      expect(screen.getByTestId("pos-stock-sync")).toHaveTextContent("true");

      // m を選択
      await user.click(screen.getByLabelText("数量単位"));
      await user.click(await screen.findByRole("option", { name: "m" }));

      // lost update がないなら: stockUnit="m" かつ posStockSync=false になる
      expect(screen.getByTestId("stock-unit")).toHaveTextContent("m");
      expect(screen.getByTestId("pos-stock-sync")).toHaveTextContent("false");
      // touched は立っていない（suggest 経路）
      expect(screen.getByTestId("pos-sync-touched")).toHaveTextContent("false");
    });

    // REQ-101 / UI-01b-D6: pcs→m→pcs（checkbox 未操作）で posStockSync が true に復元される
    it("pcs→m→pcs without checkbox: posStockSync restores to true", async () => {
      const user = userEvent.setup();
      render(<StatefulProductForm />);

      await user.click(screen.getByLabelText("数量単位"));
      await user.click(await screen.findByRole("option", { name: "m" }));
      expect(screen.getByTestId("stock-unit")).toHaveTextContent("m");
      expect(screen.getByTestId("pos-stock-sync")).toHaveTextContent("false");

      await user.click(screen.getByLabelText("数量単位"));
      await user.click(await screen.findByRole("option", { name: "個" }));
      expect(screen.getByTestId("stock-unit")).toHaveTextContent("pcs");
      expect(screen.getByTestId("pos-stock-sync")).toHaveTextContent("true");
      expect(screen.getByTestId("pos-sync-touched")).toHaveTextContent("false");
    });

    // REQ-101 / UI-01b-D6: checkbox を明示操作後は単位変更でも利用者の値が保持される
    it("after checkbox interaction (touched=true), unit change preserves user override", async () => {
      const user = userEvent.setup();
      render(<StatefulProductForm />);

      // まず checkbox を操作して touched を立てる（posStockSync を false に変更）
      await user.click(screen.getByLabelText("POS販売で在庫を減らす"));
      expect(screen.getByTestId("pos-sync-touched")).toHaveTextContent("true");
      expect(screen.getByTestId("pos-stock-sync")).toHaveTextContent("false");

      // 単位を m に変更しても suggest が発火しないので利用者の false が保持される
      await user.click(screen.getByLabelText("数量単位"));
      await user.click(await screen.findByRole("option", { name: "m" }));
      expect(screen.getByTestId("stock-unit")).toHaveTextContent("m");
      expect(screen.getByTestId("pos-stock-sync")).toHaveTextContent("false");

      // 単位を pcs に戻しても suggest が発火しないので false のまま
      await user.click(screen.getByLabelText("数量単位"));
      await user.click(await screen.findByRole("option", { name: "個" }));
      expect(screen.getByTestId("stock-unit")).toHaveTextContent("pcs");
      expect(screen.getByTestId("pos-stock-sync")).toHaveTextContent("false");
    });
  });

  it("restore is direct without confirmation dialog", async () => {
    const user = userEvent.setup();
    const onToggleDiscontinue = vi.fn();
    const product = makeMockProductWithRelations({
      product_code: "P-201",
      name: "廃番商品",
      is_discontinued: true,
    });

    render(
      <ProductForm
        mode="edit"
        values={productToFormValues(product)}
        departments={[makeMockDepartment()]}
        suppliers={[makeMockSupplier()]}
        errors={{}}
        saveError={null}
        supplierWarning={null}
        productCodeLabel={product.product_code}
        productName={product.name}
        isDiscontinued
        isSaving={false}
        posSyncTouched={false}
        onValuesChange={vi.fn()}
        onPosSyncTouchedChange={vi.fn()}
        onSubmit={vi.fn()}
        onCancel={vi.fn()}
        onToggleDiscontinue={onToggleDiscontinue}
      />,
    );

    // UI-01b-D13: 「表示に戻す」は確認なしで直接実行
    await user.click(screen.getByRole("button", { name: "表示に戻す" }));
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    expect(onToggleDiscontinue).toHaveBeenCalledTimes(1);
  });

  it("REQ-402 suggests PLU target from 13 digit JAN and preserves manual override", async () => {
    const user = userEvent.setup();
    function StatefulProductForm() {
      const [values, setValues] = React.useState<ProductFormValues>(createProductFormDefaults);
      const [pluTargetTouched, setPluTargetTouched] = React.useState(false);

      return (
        <>
          <div data-testid="plu-target">{String(values.pluTarget)}</div>
          <div data-testid="plu-target-touched">{String(pluTargetTouched)}</div>
          <ProductForm
            mode="create"
            values={values}
            departments={[makeMockDepartment()]}
            suppliers={[makeMockSupplier()]}
            errors={{}}
            saveError={null}
            supplierWarning={null}
            isSaving={false}
            posSyncTouched={false}
            pluTargetTouched={pluTargetTouched}
            onValuesChange={setValues}
            onPosSyncTouchedChange={vi.fn()}
            onPluTargetTouchedChange={setPluTargetTouched}
            onSubmit={vi.fn()}
            onCancel={vi.fn()}
          />
        </>
      );
    }

    render(<StatefulProductForm />);

    await user.type(screen.getByLabelText("JANコード"), "4901234567894");
    expect(screen.getByTestId("plu-target")).toHaveTextContent("true");

    await user.click(screen.getByLabelText("レジにバーコード登録する"));
    expect(screen.getByTestId("plu-target")).toHaveTextContent("false");
    expect(screen.getByTestId("plu-target-touched")).toHaveTextContent("true");

    await user.clear(screen.getByLabelText("JANコード"));
    await user.type(screen.getByLabelText("JANコード"), "4901234567894");
    expect(screen.getByTestId("plu-target")).toHaveTextContent("false");
  });

  it("REQ-101 UI-01b-D16 normalizes whole fullwidth digits outside composition", () => {
    function StatefulProductForm() {
      const [values, setValues] = React.useState<ProductFormValues>(createProductFormDefaults);
      return (
        <ProductForm
          mode="create"
          values={values}
          departments={[makeMockDepartment()]}
          suppliers={[makeMockSupplier()]}
          errors={{}}
          saveError={null}
          supplierWarning={null}
          isSaving={false}
          posSyncTouched={false}
          onValuesChange={setValues}
          onPosSyncTouchedChange={vi.fn()}
          onSubmit={vi.fn()}
          onCancel={vi.fn()}
        />
      );
    }

    render(<StatefulProductForm />);
    fireEvent.change(screen.getByLabelText("JANコード"), {
      target: { value: "４９０１２３４５６７８８７" },
    });

    expect(screen.getByLabelText("JANコード")).toHaveValue("4901234567887");
    expect(screen.getByLabelText("レジにバーコード登録する")).toBeChecked();
  });

  it("REQ-101 UI-01b-D16 preserves mixed and composing input", () => {
    function StatefulProductForm() {
      const [values, setValues] = React.useState<ProductFormValues>(createProductFormDefaults);
      return (
        <ProductForm
          mode="create"
          values={values}
          departments={[makeMockDepartment()]}
          suppliers={[makeMockSupplier()]}
          errors={{}}
          saveError={null}
          supplierWarning={null}
          isSaving={false}
          posSyncTouched={false}
          onValuesChange={setValues}
          onPosSyncTouchedChange={vi.fn()}
          onSubmit={vi.fn()}
          onCancel={vi.fn()}
        />
      );
    }

    render(<StatefulProductForm />);
    const input = screen.getByLabelText("JANコード");
    fireEvent.change(input, { target: { value: "１２A３" } });
    expect(input).toHaveValue("１２A３");

    const composingChange = createEvent.change(input, {
      target: { value: "４９０１２３４５６７８８７" },
    });
    Object.defineProperty(composingChange, "isComposing", { value: true });
    fireEvent(input, composingChange);
    expect(input).toHaveValue("４９０１２３４５６７８８７");
  });

  it("REQ-101 UI-01b-D16 normalizes once at compositionend", () => {
    const onValuesChange = vi.fn();
    render(
      <ProductForm
        mode="create"
        values={createProductFormDefaults}
        departments={[makeMockDepartment()]}
        suppliers={[makeMockSupplier()]}
        errors={{}}
        saveError={null}
        supplierWarning={null}
        isSaving={false}
        posSyncTouched={false}
        onValuesChange={onValuesChange}
        onPosSyncTouchedChange={vi.fn()}
        onSubmit={vi.fn()}
        onCancel={vi.fn()}
      />,
    );

    fireEvent.compositionEnd(screen.getByLabelText("JANコード"), {
      target: { value: "４９０１２３４５６７８８７" },
    });

    expect(onValuesChange).toHaveBeenCalledOnce();
    const updater = onValuesChange.mock.calls[0]?.[0] as (
      values: ProductFormValues,
    ) => ProductFormValues;
    expect(updater(createProductFormDefaults)).toMatchObject({
      janCode: "4901234567887",
      pluTarget: true,
    });
  });

  it("REQ-402 suggests PLU target from normalized candidates and keeps JAN-8 false", () => {
    function StatefulProductForm() {
      const [values, setValues] = React.useState<ProductFormValues>(createProductFormDefaults);
      return (
        <ProductForm
          mode="create"
          values={values}
          departments={[makeMockDepartment()]}
          suppliers={[makeMockSupplier()]}
          errors={{}}
          saveError={null}
          supplierWarning={null}
          isSaving={false}
          posSyncTouched={false}
          onValuesChange={setValues}
          onPosSyncTouchedChange={vi.fn()}
          onSubmit={vi.fn()}
          onCancel={vi.fn()}
        />
      );
    }

    render(<StatefulProductForm />);
    const input = screen.getByLabelText("JANコード");

    fireEvent.change(input, { target: { value: " ４９０１２３４５６７８８７ " } });
    expect(screen.getByLabelText("レジにバーコード登録する")).toBeChecked();

    fireEvent.change(input, { target: { value: "９６３８５０７４" } });
    expect(screen.getByLabelText("レジにバーコード登録する")).not.toBeChecked();
  });
});

describe("ProductForm price history and supplier picker (REQ-102 / REQ-106)", () => {
  const mockListPriceHistory = vi.mocked(commands.listPriceHistory);
  const mockCreateSupplier = vi.mocked(commands.createSupplier);
  const mockListSuppliers = vi.mocked(commands.listSuppliers);

  beforeEach(() => {
    mockListPriceHistory.mockReset();
    mockCreateSupplier.mockReset();
    mockListSuppliers.mockReset();
    mockListPriceHistory.mockResolvedValue({ status: "ok", data: [] });
    mockListSuppliers.mockResolvedValue({ status: "ok", data: [] });
  });

  function renderStateful(mode: "create" | "edit" = "edit") {
    // round 3 P1: onValuesChange を Harness の外で hoist する。Harness 内で
    // vi.fn(setValues) を inline 生成すると test scope から参照できないため、
    // 実 state 更新（setValues）と spy 呼び出し（onValuesChange）を両立させる。
    const onValuesChange = vi.fn();
    const onSubmit = vi.fn();
    function Harness() {
      const [values, setValues] = React.useState<ProductFormValues>({
        ...createProductFormDefaults,
        name: "入力済みの商品名",
      });
      return (
        <ProductForm
          mode={mode}
          values={values}
          departments={[makeMockDepartment()]}
          suppliers={[makeMockSupplier({ id: 1, name: "既存取引先" })]}
          errors={{}}
          saveError={null}
          supplierWarning={null}
          productCodeLabel={mode === "edit" ? "PRICE-001" : undefined}
          isSaving={false}
          posSyncTouched={false}
          onValuesChange={(next) => {
            onValuesChange(next);
            setValues(next);
          }}
          onPosSyncTouchedChange={vi.fn()}
          onSubmit={onSubmit}
          onCancel={vi.fn()}
        />
      );
    }
    return { ...render(<Harness />), onValuesChange, onSubmit };
  }

  it("shows history only in edit mode and calls listPriceHistory with 10", async () => {
    const entry: PriceHistoryEntry = {
      id: 2,
      old_selling_price: 100,
      new_selling_price: 120,
      old_cost_price_centi: 6000,
      new_cost_price_centi: 7000,
      changed_at: "2026-07-06T17:15:16",
    };
    mockListPriceHistory.mockResolvedValue({ status: "ok", data: [entry] });
    const edit = renderStateful("edit");
    expect(await screen.findByRole("heading", { name: "価格履歴" })).toBeInTheDocument();
    expect(mockListPriceHistory).toHaveBeenCalledWith("PRICE-001", 10);
    expect(await screen.findByText("2026-07-06 17:15:16")).toBeInTheDocument();
    // ⑮ SC16: 列見出しとold→newを独立literalで固定。
    expect(screen.getAllByRole("columnheader").map((cell) => cell.textContent)).toEqual([
      "変更日時",
      "売価",
      "原価",
    ]);
    // SC16 Amendment 2: 売価は既存 JPY formatter の円記号を維持し、原価は SPEC-UNIT-D5 の半角の ¥。
    // 接頭語と ISO の T を除く。
    const cells = screen.getAllByRole("cell");
    expect(cells.map((cell) => cell.textContent)).toEqual([
      "2026-07-06 17:15:16",
      "￥100 → ￥120",
      "¥60 → ¥70",
    ]);
    expect(cells[0]).not.toHaveTextContent("T");
    expect(cells[1]).toHaveTextContent(/^￥/);
    expect(cells[2]).toHaveTextContent(/^¥/);
    for (const cell of cells.slice(1)) {
      expect(cell).not.toHaveTextContent(/売価|原価/);
    }
    edit.unmount();
    mockListPriceHistory.mockClear();
    renderStateful("create");
    expect(screen.queryByRole("heading", { name: "価格履歴" })).not.toBeInTheDocument();
    expect(mockListPriceHistory).not.toHaveBeenCalled();
  });

  it("keeps the returned newest-first row order", async () => {
    mockListPriceHistory.mockResolvedValue({
      status: "ok",
      data: [
        {
          id: 9,
          old_selling_price: 200,
          new_selling_price: 210,
          old_cost_price_centi: 10000,
          new_cost_price_centi: 10500,
          changed_at: "2026-07-06T17:15:16",
        },
        {
          id: 8,
          old_selling_price: 180,
          new_selling_price: 200,
          old_cost_price_centi: 9000,
          new_cost_price_centi: 10000,
          changed_at: "2026-07-05T09:10:11",
        },
      ],
    });
    renderStateful();
    const newest = await screen.findByText("2026-07-06 17:15:16");
    const older = screen.getByText("2026-07-05 09:10:11");
    const newestRow = newest.closest("tr");
    const olderRow = older.closest("tr");
    if (newestRow === null || olderRow === null) throw new Error("expected table structure");
    expect(
      within(newestRow)
        .getAllByRole("cell")
        .map((cell) => cell.textContent),
    ).toEqual(["2026-07-06 17:15:16", "￥200 → ￥210", "¥100 → ¥105"]);
    expect(
      within(olderRow)
        .getAllByRole("cell")
        .map((cell) => cell.textContent),
    ).toEqual(["2026-07-05 09:10:11", "￥180 → ￥200", "¥90 → ¥100"]);
    expect(newest.compareDocumentPosition(older) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });

  it("reloads all history with limit 100", async () => {
    mockListPriceHistory.mockResolvedValue({
      status: "ok",
      data: [
        {
          id: 1,
          old_selling_price: 1,
          new_selling_price: 2,
          old_cost_price_centi: 1,
          new_cost_price_centi: 2,
          changed_at: "ROW",
        },
      ],
    });
    const user = userEvent.setup();
    renderStateful();
    // ⑮ SC15: 取得limitと説明の件数を一緒に更新する。
    expect(
      screen.getByText("直近 10 件の売価・原価の変更を新しい順に表示します。"),
    ).toBeInTheDocument();
    await user.click(await screen.findByRole("button", { name: "すべて表示" }));
    expect(
      screen.getByText("直近 100 件の売価・原価の変更を新しい順に表示します。"),
    ).toBeInTheDocument();
    expect(mockListPriceHistory).toHaveBeenCalledWith("PRICE-001", 100);
  });

  it("shows the exact empty message", async () => {
    renderStateful();
    expect(await screen.findByText("価格履歴はまだありません")).toBeInTheDocument();
  });

  it("shows an inline error and retries", async () => {
    mockListPriceHistory
      .mockResolvedValueOnce({
        status: "error",
        error: {
          kind: "internal",
          message: "synthetic history failure",
          field: null,
          error_id: null,
          stocktake_recovery: null,
        },
      })
      .mockResolvedValueOnce({ status: "ok", data: [] });
    const user = userEvent.setup();
    renderStateful();
    expect(await screen.findByRole("alert")).toHaveTextContent("価格履歴を取得できませんでした");
    await user.click(screen.getByRole("button", { name: "再試行" }));
    expect(await screen.findByText("価格履歴はまだありません")).toBeInTheDocument();
    expect(mockListPriceHistory).toHaveBeenCalledTimes(2);
  });

  it("UI-01b-D21: rejects whitespace supplier names inside the picker without calling createSupplier", async () => {
    const user = userEvent.setup();
    renderStateful("create");
    await user.click(screen.getByLabelText("取引先"));
    await user.click(screen.getByRole("button", { name: "新しい取引先を追加" }));
    await user.type(screen.getByLabelText("取引先名"), "   ");
    expect(screen.getByRole("button", { name: "追加する" })).toHaveAttribute(
      "data-variant",
      "default",
    );
    await user.click(screen.getByRole("button", { name: "追加する" }));
    expect(screen.getByRole("alert")).toHaveTextContent("取引先名を入力してください");
    expect(mockCreateSupplier).not.toHaveBeenCalled();
  });

  it("UI-01b-D21: creates a supplier from the picker, refreshes options and auto-selects it", async () => {
    mockCreateSupplier.mockResolvedValue({
      status: "ok",
      data: { id: 44, name: "新規取引先", created_at: "2026-08-22T00:00:00" },
    });
    mockListSuppliers.mockResolvedValue({
      status: "ok",
      data: [{ id: 44, name: "新規取引先", created_at: "2026-08-22T00:00:00" }],
    });
    const user = userEvent.setup();
    const { onValuesChange, onSubmit } = renderStateful("create");
    await user.click(screen.getByLabelText("取引先"));
    await user.click(screen.getByRole("button", { name: "新しい取引先を追加" }));
    await user.type(screen.getByLabelText("取引先名"), "  新規取引先  ");
    expect(screen.getByRole("button", { name: "追加する" })).toHaveAttribute(
      "data-variant",
      "default",
    );
    await user.click(screen.getByRole("button", { name: "追加する" }));
    expect(mockCreateSupplier).toHaveBeenCalledWith("新規取引先");
    expect(mockListSuppliers).toHaveBeenCalledTimes(1);
    // onValuesChange は React.Dispatch<SetStateAction<...>> なので functional updater
    // (prev) => ({ ...prev, supplierId: created.id }) を受け取る。updater を解決して
    // supplierId の数値そのもの（off-by-one 等）を検査する。
    await waitFor(() => {
      const lastCall = onValuesChange.mock.calls[onValuesChange.mock.calls.length - 1]?.[0] as
        | ProductFormValues
        | ((prev: ProductFormValues) => ProductFormValues)
        | undefined;
      const resolved =
        typeof lastCall === "function" ? lastCall(createProductFormDefaults) : lastCall;
      expect(resolved).toEqual(expect.objectContaining({ supplierId: 44 }));
    });
    expect(screen.getByLabelText("取引先")).toHaveTextContent("新規取引先");
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("UI-01b-D21: preserves nested supplier input and other form values after create failure", async () => {
    mockCreateSupplier.mockResolvedValue({
      status: "error",
      error: {
        kind: "internal",
        message: "synthetic supplier failure",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });
    const user = userEvent.setup();
    renderStateful("create");
    await user.click(screen.getByLabelText("取引先"));
    await user.click(screen.getByRole("button", { name: "新しい取引先を追加" }));
    await user.type(screen.getByLabelText("取引先名"), "保持する取引先名");
    await user.click(screen.getByRole("button", { name: "追加する" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("synthetic supplier failure");
    expect(screen.getByLabelText("取引先名")).toHaveValue("保持する取引先名");
    expect(screen.getByLabelText("商品名（必須）")).toHaveValue("入力済みの商品名");
  });
});

describe("ProductForm native input tokens（Lane 5 SC4e）", () => {
  it("SC4e: 部門/取引先/税率の3箇所すべてがbg-control-surfaceでbg-backgroundを持たない", () => {
    const product = makeMockProductWithRelations({
      product_code: "P-001",
      jan_code: "4901234567890",
      stock_quantity: 20,
      stock_unit: "m",
    });

    render(
      <ProductForm
        mode="edit"
        values={productToFormValues(product)}
        departments={[makeMockDepartment()]}
        suppliers={[makeMockSupplier()]}
        errors={{}}
        saveError={null}
        supplierWarning={null}
        productCodeLabel={product.product_code}
        isDiscontinued={false}
        isSaving={false}
        posSyncTouched={false}
        onValuesChange={vi.fn()}
        onPosSyncTouchedChange={vi.fn()}
        onSubmit={vi.fn()}
        onCancel={vi.fn()}
      />,
    );

    const fields = [
      screen.getByLabelText("部門（必須）"),
      screen.getByLabelText("取引先"),
      screen.getByLabelText("税率"),
    ];
    for (const field of fields) {
      expect(field).toHaveClass("bg-control-surface");
      expect(field).not.toHaveClass("bg-background");
    }
  });
});

describe("ProductForm SC8a/SC8b/SC8c 部門・税率 select と取引先 picker（⑧、L8-D2, L8-D5）", () => {
  function renderForm(
    initialValues: ProductFormValues = createProductFormDefaults,
    overrides: Partial<React.ComponentProps<typeof ProductForm>> = {},
  ) {
    const onValuesChange = vi.fn();
    let externalSetValues!: React.Dispatch<React.SetStateAction<ProductFormValues>>;
    function Harness() {
      const [values, setValues] = React.useState<ProductFormValues>(initialValues);
      externalSetValues = setValues;
      return (
        <ProductForm
          mode="create"
          values={values}
          departments={[makeMockDepartment({ id: 1, name: "毛糸", code_prefix: "Y" })]}
          suppliers={[makeMockSupplier({ id: 1, name: "既存取引先" })]}
          errors={{}}
          saveError={null}
          supplierWarning={null}
          isSaving={false}
          posSyncTouched={false}
          onValuesChange={(next) => {
            onValuesChange(next);
            setValues(next);
          }}
          onPosSyncTouchedChange={vi.fn()}
          onSubmit={vi.fn()}
          onCancel={vi.fn()}
          {...overrides}
        />
      );
    }
    render(<Harness />);
    // P2-4: 親からの外部 reset（setValues 直接呼び出し）を再現するための handle。
    return { onValuesChange, setValues: externalSetValues };
  }

  it("SC8a: 部門selectは未選択時にplaceholderを表示し偽optionを持たず、実在部門選択でsuffix込み表示とnumber復元がround-tripする（L8-D2, L8-D5）", async () => {
    const user = userEvent.setup();
    const { onValuesChange, setValues } = renderForm(createProductFormDefaults, {
      errors: { departmentId: "部門を選択してください" },
    });

    const trigger = screen.getByLabelText("部門（必須）");
    expect(trigger).toHaveAttribute("data-slot", "select-trigger");
    expect(trigger.tagName).toBe("BUTTON");
    expect(trigger).toHaveTextContent("選択してください");
    expect(screen.getByText("部門を選択してください")).toBeInTheDocument();

    await user.click(trigger);
    expect(screen.queryByRole("option", { name: "選択してください" })).not.toBeInTheDocument();
    await user.click(screen.getByRole("option", { name: "毛糸（独自コード可）" }));

    expect(trigger).toHaveTextContent("毛糸（独自コード可）");
    // onValuesChange は React.Dispatch<SetStateAction<...>> なので functional updater
    // (prev) => ({ ...prev, departmentId: Number(value) }) を受け取る。updater を解決して
    // departmentId が number（"1" でなく 1）であることを検査する
    // （src/features/products/components/ProductForm.tsx の部門の Select の onValueChange の Number() 欠落 mutant を kill する）。
    const lastCall = onValuesChange.mock.calls[onValuesChange.mock.calls.length - 1][0] as (
      prev: ProductFormValues,
    ) => ProductFormValues;
    expect(lastCall(createProductFormDefaults)).toEqual(
      expect.objectContaining({ departmentId: 1 }),
    );

    // P2-4: 部門選択後に親が departmentId を null へ reset した経路
    // （src/features/products/components/ProductForm.tsx の部門の Select の controlled value "" → undefined mutant を kill する）。
    act(() => {
      setValues((prev) => ({ ...prev, departmentId: null }));
    });
    expect(trigger).toHaveTextContent("選択してください");
  });

  it("UI-01b-D7: selects a supplier through the picker and keeps 取引先なし as null", async () => {
    const user = userEvent.setup();
    const { onValuesChange } = renderForm({ ...createProductFormDefaults, supplierId: 1 });

    const trigger = screen.getByLabelText("取引先");
    expect(trigger).toHaveAttribute("aria-haspopup", "dialog");
    expect(trigger.tagName).toBe("BUTTON");
    expect(trigger).toHaveTextContent("既存取引先");

    await user.click(trigger);
    await user.click(screen.getByRole("button", { name: "取引先なし" }));

    expect(trigger).toHaveTextContent("取引先なし");
    const update = onValuesChange.mock.lastCall?.[0] as
      | ((prev: ProductFormValues) => ProductFormValues)
      | undefined;
    expect(update?.(createProductFormDefaults).supplierId).toBeNull();
  });

  it("SC8b: 取引先pickerで実在取引先を手動選択するとsupplierIdがnumberになる（round-trip、L8-D5）", async () => {
    const user = userEvent.setup();
    const { onValuesChange } = renderForm(createProductFormDefaults, {
      suppliers: [
        makeMockSupplier({ id: 1, name: "既存取引先" }),
        makeMockSupplier({ id: 2, name: "新取引先" }),
      ],
    });

    const trigger = screen.getByLabelText("取引先");
    await user.click(trigger);
    await user.click(await screen.findByRole("button", { name: "新取引先" }));

    expect(trigger).toHaveTextContent("新取引先");
    // picker の手動選択も数値 ID をそのまま渡す。
    const lastCall = onValuesChange.mock.calls[onValuesChange.mock.calls.length - 1][0] as (
      prev: ProductFormValues,
    ) => ProductFormValues;
    expect(lastCall(createProductFormDefaults)).toEqual(expect.objectContaining({ supplierId: 2 }));
  });

  it("SC8b: supplierWarningがある間はtrigger がdisabledになる", () => {
    renderForm(createProductFormDefaults, {
      supplierWarning: "取引先の在庫連携に問題があります",
    });
    expect(screen.getByLabelText("取引先")).toBeDisabled();
  });

  it("SC8c: 税率selectはtriggerを開くと3件（10/8/0%）のoptionを持ち選択でtaxRateが対応する値に更新される（P2-2）", async () => {
    const user = userEvent.setup();
    const { onValuesChange } = renderForm();

    const trigger = screen.getByLabelText("税率");
    expect(trigger).toHaveAttribute("data-slot", "select-trigger");
    expect(trigger.tagName).toBe("BUTTON");
    await user.click(trigger);
    expect((await screen.findAllByRole("option")).map((option) => option.textContent)).toEqual([
      "10%",
      "8%",
      "0%",
    ]);

    await user.click(screen.getByRole("option", { name: "8%" }));
    expect(trigger).toHaveTextContent("8%");
    const afterEight = onValuesChange.mock.calls[onValuesChange.mock.calls.length - 1][0] as (
      prev: ProductFormValues,
    ) => ProductFormValues;
    expect(afterEight(createProductFormDefaults)).toEqual(
      expect.objectContaining({ taxRate: "8" }),
    );

    await user.click(trigger);
    await user.click(screen.getByRole("option", { name: "0%" }));
    expect(trigger).toHaveTextContent("0%");
    const afterZero = onValuesChange.mock.calls[onValuesChange.mock.calls.length - 1][0] as (
      prev: ProductFormValues,
    ) => ProductFormValues;
    expect(afterZero(createProductFormDefaults)).toEqual(expect.objectContaining({ taxRate: "0" }));

    // P2-R2-1: 8%→0%以外に10%も検査する（round 1 は8/0の2件で打ち切っていた）。
    await user.click(trigger);
    await user.click(screen.getByRole("option", { name: "10%" }));
    expect(trigger).toHaveTextContent("10%");
    const afterTen = onValuesChange.mock.calls[onValuesChange.mock.calls.length - 1][0] as (
      prev: ProductFormValues,
    ) => ProductFormValues;
    expect(afterTen(createProductFormDefaults)).toEqual(expect.objectContaining({ taxRate: "10" }));
  });
});

// UI-01b-D22 / REQ-101: 長さの単位の価格欄は 1 m あたり、m の初期在庫は小数の入力、修正の現在庫は単位つき
describe("ProductForm (UI-01b-D22)", () => {
  function renderForm(mode: "create" | "edit", values: ProductFormValues) {
    render(
      <ProductForm
        mode={mode}
        values={values}
        departments={[makeMockDepartment()]}
        suppliers={[makeMockSupplier()]}
        errors={{}}
        saveError={null}
        supplierWarning={null}
        productCodeLabel={mode === "edit" ? "P-001" : undefined}
        isSaving={false}
        posSyncTouched={false}
        onValuesChange={vi.fn()}
        onPosSyncTouchedChange={vi.fn()}
        onSubmit={vi.fn()}
        onCancel={vi.fn()}
      />,
    );
  }

  it.each(["m"] as const)(
    "UI-01b-D22: 単位 %s では売価・原価の label に「1 m あたり」が付く",
    (stockUnit) => {
      renderForm("create", { ...createProductFormDefaults, stockUnit });
      expect(screen.getByLabelText("売価（必須、1 m あたり）")).toBeInTheDocument();
      expect(screen.getByLabelText("原価（税抜、必須、1 m あたり）")).toBeInTheDocument();
    },
  );

  it("UI-01b-D22: 個数の単位では「1 m あたり」を出さず、価格の節の説明は税込・税抜と書く", () => {
    renderForm("create", { ...createProductFormDefaults, stockUnit: "ball" });
    expect(screen.getByLabelText("売価（必須）")).toBeInTheDocument();
    expect(screen.getByLabelText("原価（税抜、必須）")).toBeInTheDocument();
    expect(
      screen.getByText(
        "売価は税込の円の整数、原価は税抜の円で小数 2 桁まで入力します。長さ（m）の商品は 1 m あたりの値段です。",
      ),
    ).toBeInTheDocument();
  });

  it.each([
    ["create", "m", "decimal"],
    ["create", "pcs", "numeric"],
  ] as const)(
    "UI-01b-D22: %s で単位 %s の初期在庫の欄の inputMode は %s",
    (mode, stockUnit, inputMode) => {
      renderForm(mode, { ...createProductFormDefaults, stockUnit });
      expect(screen.getByLabelText("初期在庫（必須）")).toHaveAttribute("inputmode", inputMode);
    },
  );

  it.each([
    ["m", 2750, "27.5 m", "m"],
    ["pcs", 20, "20 個", "個"],
    ["m", 0, "0 m", "m"],
    ["pcs", 0, "0 個", "個"],
    ["sheet", 3, "3 枚", "枚"],
  ] as const)(
    "UI-01b-D22: 修正の画面は %s・在庫 %s の現在庫を %s、単位を「%s」で出す",
    (stock_unit, stock_quantity, expected, unitLabel) => {
      renderForm(
        "edit",
        productToFormValues(makeMockProductWithRelations({ stock_unit, stock_quantity })),
      );
      const stock = screen.getByLabelText("現在庫");
      expect(stock).toHaveValue(expected);
      expect(stock).toHaveAttribute("readonly");
      expect(stock).toHaveAttribute("inputmode", "numeric");
      expect(screen.getByLabelText("数量単位")).toHaveTextContent(unitLabel);
    },
  );
});

// Matrix C14 / UI-01b-D22・D24 / D-128: 原価の 1/100 円、税抜の label、「袋の値段から出す」の補助
describe("ProductForm cost centi, label and pack helper (UI-01b-D24)", () => {
  function renderCostForm(mode: "create" | "edit", initial: ProductFormValues) {
    const latest: { values: ProductFormValues } = { values: initial };
    function Harness() {
      const [values, setValues] = React.useState<ProductFormValues>(initial);
      latest.values = values;
      return (
        <ProductForm
          mode={mode}
          values={values}
          departments={[makeMockDepartment()]}
          suppliers={[makeMockSupplier()]}
          errors={{}}
          saveError={null}
          supplierWarning={null}
          productCodeLabel={mode === "edit" ? "P-001" : undefined}
          isSaving={false}
          posSyncTouched={false}
          onValuesChange={setValues}
          onPosSyncTouchedChange={vi.fn()}
          onSubmit={vi.fn()}
          onCancel={vi.fn()}
        />
      );
    }
    render(<Harness />);
    return latest;
  }

  const department = makeMockDepartment({ id: 1, code_prefix: "KM" });
  const createValues = {
    ...createProductFormDefaults,
    name: "合成の試験品",
    departmentId: 1,
    sellingPrice: "100",
  };

  it("Matrix C14: 個数の単位で 1000 円 ÷ 12 を原価に入れると 83.33 で、request は 8333 だけを送る", async () => {
    const user = userEvent.setup();
    const latest = renderCostForm("create", createValues);
    await user.click(screen.getByRole("button", { name: "袋の値段から出す" }));
    await user.type(screen.getByLabelText("袋の値段（円）"), "1000");
    await user.type(screen.getByLabelText("入り数"), "12");
    await user.click(screen.getByRole("button", { name: "原価に入れる" }));
    expect(screen.getByLabelText("原価（税抜、必須）")).toHaveValue("83.33");
    const request = buildCreateProductRequest(latest.values, [department]).request;
    expect(request?.cost_price_centi).toBe(8333);
    expect(JSON.stringify(request)).not.toMatch(/1000|"12"|pack/);
    expect(screen.getByLabelText("原価（税抜、必須）")).toHaveAttribute("inputmode", "decimal");
  });

  it("Matrix C14: 入り数の誤りは補助の欄に出して原価の欄を変えず、閉じて開き直すと 2 つの欄は空", async () => {
    const user = userEvent.setup();
    renderCostForm("create", { ...createValues, costPrice: "50" });
    const toggle = screen.getByRole("button", { name: "袋の値段から出す" });
    await user.click(toggle);
    await user.type(screen.getByLabelText("袋の値段（円）"), "1000");
    await user.type(screen.getByLabelText("入り数"), "0");
    await user.click(screen.getByRole("button", { name: "原価に入れる" }));
    expect(screen.getByText("入り数は1以上の整数で入力してください")).toBeInTheDocument();
    expect(screen.getByLabelText("原価（税抜、必須）")).toHaveValue("50");
    await user.click(toggle);
    expect(screen.queryByLabelText("袋の値段（円）")).toBeNull();
    await user.click(toggle);
    expect(screen.getByLabelText("袋の値段（円）")).toHaveValue("");
    expect(screen.getByLabelText("入り数")).toHaveValue("");
  });

  it("Matrix C14: 単位を m に変えると補助は閉じて隠れ、label は 1 m あたり", async () => {
    const user = userEvent.setup();
    renderCostForm("create", createValues);
    await user.click(screen.getByRole("button", { name: "袋の値段から出す" }));
    expect(screen.getByLabelText("袋の値段（円）")).toBeInTheDocument();
    await user.click(screen.getByLabelText("数量単位"));
    await user.click(await screen.findByRole("option", { name: "m" }));
    expect(screen.queryByRole("button", { name: "袋の値段から出す" })).toBeNull();
    expect(screen.queryByLabelText("袋の値段（円）")).toBeNull();
    expect(screen.getByLabelText("原価（税抜、必須、1 m あたり）")).toBeInTheDocument();
  });

  it("Matrix C14: 修正の画面を原価 8333 の商品で開くと原価の欄は 83.33", () => {
    renderCostForm(
      "edit",
      productToFormValues(makeMockProductWithRelations({ cost_price_centi: 8333 })),
    );
    expect(screen.getByLabelText("原価（税抜、必須）")).toHaveValue("83.33");
  });
});
