import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { commands } from "@/lib/bindings";
import { d052InvalidationOracle, expectExactInvalidations } from "@/test/invalidation-oracle";
import { open, save } from "@tauri-apps/plugin-dialog";
import { readFile, writeFile } from "@tauri-apps/plugin-fs";
import { PLU_EXPORT_PENDING_STORAGE_KEY, PluExportPage } from "./PluExportPage";

vi.mock("@tanstack/react-router", () => ({
  Link: ({ to, children }: { to: string; children: ReactNode }) => <a href={to}>{children}</a>,
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
  save: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-fs", () => ({
  readFile: vi.fn(),
  writeFile: vi.fn(),
}));

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

vi.mock("@/lib/bindings", () => ({
  commands: {
    listPluDirty: vi.fn(),
    preparePluExport: vi.fn(),
    confirmPluExportSaved: vi.fn(),
    importPluRegisterSnapshot: vi.fn(),
    getPluSlotSummary: vi.fn(),
  },
}));

const mockListPluDirty = vi.mocked(commands.listPluDirty);
const mockPreparePluExport = vi.mocked(commands.preparePluExport);
const mockConfirmPluExportSaved = vi.mocked(commands.confirmPluExportSaved);
const mockImportPluRegisterSnapshot = vi.mocked(commands.importPluRegisterSnapshot);
const mockGetPluSlotSummary = vi.mocked(commands.getPluSlotSummary);
const mockSave = vi.mocked(save);
const mockOpen = vi.mocked(open);
const mockReadFile = vi.mocked(readFile);
const mockWriteFile = vi.mocked(writeFile);
const mockScrollTo = vi.fn();

function installMemoryStorage() {
  const entries = new Map<string, string>();
  const storage: Storage = {
    get length() {
      return entries.size;
    },
    clear: () => {
      entries.clear();
    },
    getItem: (key: string) => entries.get(key) ?? null,
    key: (index: number) => Array.from(entries.keys())[index] ?? null,
    removeItem: (key: string) => {
      entries.delete(key);
    },
    setItem: (key: string, value: string) => {
      entries.set(key, value);
    },
  };
  Object.defineProperty(window, "localStorage", {
    value: storage,
    configurable: true,
  });
  Object.defineProperty(globalThis, "localStorage", {
    value: storage,
    configurable: true,
  });
}

function pendingExportJson(savedAt: string) {
  return JSON.stringify({
    version: 1,
    mode: "diff",
    savedAt,
    savedPath: "/home/kosei/PLU_20260701.txt",
    suggestedFilename: "PLU_20260701.txt",
    count: 1,
    encoding: "CP932",
    targetProductCodes: ["PLU-001"],
    preparedRows: [{ memory_no: 217, row_kind: "product", target_product_codes: ["PLU-001"] }],
    overLimitWarning: false,
  });
}

function expectStatusRegionBeforeContent() {
  const statusRegion = screen.getByRole("region", { name: "PLU書出し状態" });
  const contentRegion = screen.getByRole("region", { name: "PLU書出し内容" });
  expect(statusRegion.compareDocumentPosition(contentRegion)).toBe(
    Node.DOCUMENT_POSITION_FOLLOWING,
  );
  return statusRegion;
}

function renderWithClient(ui: ReactNode) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  return {
    queryClient,
    ...render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>),
  };
}

function mockDefaultCommands() {
  mockListPluDirty.mockResolvedValue({
    status: "ok",
    data: [
      {
        product_code: "PLU-001",
        jan_code: "4900000000001",
        name: "赤い毛糸",
        department_id: 1,
        selling_price: 550,
        cost_price_centi: 300,
        stock_unit: "pcs",
        stock_quantity: 12,
        plu_dirty: true,
        plu_exported_at: null,
      },
    ],
  });
  mockPreparePluExport.mockResolvedValue({
    status: "ok",
    data: {
      bytes_base64: "QUJD",
      suggested_filename: "PLU_20260701.txt",
      content_type: "text/tab-separated-values",
      encoding: "CP932",
      count: 1,
      target_product_codes: ["PLU-001"],
      prepared_rows: [{ memory_no: 217, row_kind: "product", target_product_codes: ["PLU-001"] }],
      excluded: [],
      over_limit_warning: false,
    },
  });
  mockConfirmPluExportSaved.mockResolvedValue({
    status: "ok",
    data: { updated_count: 1, confirmed_at: "2026-07-01T12:00:00" },
  });
  mockGetPluSlotSummary.mockResolvedValue({
    status: "ok",
    data: {
      snapshot_at: "2026-08-18T12:00:00",
      free_count: 4_780,
      external_count: 1,
      app_managed_count: 3,
      conflict_count: 0,
      release_pending_count: 0,
    },
  });
  mockImportPluRegisterSnapshot.mockResolvedValue({
    status: "ok",
    data: {
      snapshot_at: "2026-08-18T12:00:00",
      free_count: 4_780,
      external_count: 1,
      app_managed_count: 3,
      conflict_count: 0,
      release_pending_count: 0,
    },
  });
}

beforeEach(() => {
  installMemoryStorage();
  window.localStorage.clear();
  mockListPluDirty.mockReset();
  mockPreparePluExport.mockReset();
  mockConfirmPluExportSaved.mockReset();
  mockSave.mockReset();
  mockWriteFile.mockReset();
  mockScrollTo.mockReset();
  vi.stubGlobal("scrollTo", mockScrollTo);
  mockDefaultCommands();
});

afterEach(() => {
  window.localStorage.clear();
  vi.unstubAllGlobals();
});

describe("PluExportPage (UI-08 / REQ-402)", () => {
  it("REQ-402 shows JAN codes in the dirty product table", async () => {
    mockListPluDirty.mockResolvedValue({
      status: "ok",
      data: [
        {
          product_code: "PLU-001",
          jan_code: "4900000000001",
          name: "赤い毛糸",
          department_id: 1,
          selling_price: 550,
          cost_price_centi: 300,
          stock_unit: "pcs",
          stock_quantity: 12,
          plu_dirty: true,
          plu_exported_at: null,
        },
        {
          product_code: "NO-JAN-001",
          jan_code: null,
          name: "JANなし商品",
          department_id: 1,
          selling_price: 330,
          cost_price_centi: 180,
          stock_unit: "pcs",
          stock_quantity: 4,
          plu_dirty: true,
          plu_exported_at: null,
        },
      ],
    });

    renderWithClient(<PluExportPage />);

    expect(await screen.findByRole("columnheader", { name: "JANコード" })).toBeInTheDocument();
    expect(screen.getByText("4900000000001")).toBeInTheDocument();
    expect(screen.getByText("未登録")).toBeInTheDocument();
  });

  it("REQ-907 enables Diff for a release-pending slot when no product is dirty", async () => {
    // UI-08-D11: 登録解除待ちだけの状態でも operator が clear 行を書き出せる回帰 oracle.
    const user = userEvent.setup();
    mockListPluDirty.mockResolvedValue({ status: "ok", data: [] });
    mockGetPluSlotSummary.mockResolvedValue({
      status: "ok",
      data: {
        snapshot_at: "2026-08-20T17:34:00",
        free_count: 3_850,
        external_count: 933,
        app_managed_count: 1,
        conflict_count: 0,
        release_pending_count: 1,
      },
    });
    mockPreparePluExport.mockResolvedValue({
      status: "ok",
      data: {
        bytes_base64: "QUJD",
        suggested_filename: "PLU_20260820.txt",
        content_type: "text/tab-separated-values",
        encoding: "CP932",
        count: 1,
        target_product_codes: ["PLU-001"],
        prepared_rows: [{ memory_no: 217, row_kind: "clear", target_product_codes: ["PLU-001"] }],
        excluded: [],
        over_limit_warning: false,
      },
    });
    mockSave.mockResolvedValue("/synthetic/PLU_20260820.txt");
    mockWriteFile.mockResolvedValue(undefined);

    renderWithClient(<PluExportPage />);

    expect(
      await screen.findByText(
        "レジ登録解除待ちが1件あります。差分を書き出すと、レジから登録を外す行を作成します。",
      ),
    ).toBeInTheDocument();
    const diffButton = screen.getByRole("button", { name: "差分を書き出す" });
    expect(diffButton).toBeEnabled();
    expect(screen.getByText("差分対象").nextElementSibling).toHaveTextContent("1 件");

    await user.click(diffButton);
    await waitFor(() => {
      expect(mockPreparePluExport).toHaveBeenCalledWith("diff");
    });
  });

  it("REQ-907 keeps Diff disabled when app-managed slots exist without dirty or release-pending rows", async () => {
    // UI-08-D11: app-managed total must not be substituted for the release-pending count.
    mockListPluDirty.mockResolvedValue({ status: "ok", data: [] });
    mockGetPluSlotSummary.mockResolvedValue({
      status: "ok",
      data: {
        snapshot_at: "2026-08-20T17:34:00",
        free_count: 3_850,
        external_count: 933,
        app_managed_count: 1,
        conflict_count: 0,
        release_pending_count: 0,
      },
    });

    renderWithClient(<PluExportPage />);

    const diffButton = await screen.findByRole("button", { name: "差分を書き出す" });
    expect(diffButton).toBeDisabled();
    expect(screen.getByText("差分対象").nextElementSibling).toHaveTextContent("0 件");
    expect(screen.queryByText(/レジ登録解除待ちが/)).not.toBeInTheDocument();
  });

  it("REQ-402 does not confirm when the save dialog is cancelled", async () => {
    const user = userEvent.setup();
    mockSave.mockResolvedValue(null);

    renderWithClient(<PluExportPage />);
    await user.click(await screen.findByRole("button", { name: "差分を書き出す" }));

    await waitFor(() => {
      expect(mockPreparePluExport).toHaveBeenCalledWith("diff");
      expect(mockSave).toHaveBeenCalled();
    });
    expect(mockWriteFile).not.toHaveBeenCalled();
    expect(mockConfirmPluExportSaved).not.toHaveBeenCalled();
    expect(await screen.findByText("保存はキャンセルされました")).toBeInTheDocument();
    expect(screen.getByText("未反映商品は残っています。")).toBeInTheDocument();
    expectStatusRegionBeforeContent();
    expect(mockScrollTo).toHaveBeenCalledWith({ top: 0, left: 0, behavior: "smooth" });
  });

  it("REQ-402 preserves target products when file save fails", async () => {
    const user = userEvent.setup();
    mockSave.mockResolvedValue("/home/kosei/PLU_20260701.txt");
    mockWriteFile.mockRejectedValue(new Error("write failed"));

    renderWithClient(<PluExportPage />);
    await user.click(await screen.findByRole("button", { name: "差分を書き出す" }));

    expect(await screen.findByText("PLUファイルを保存できませんでした")).toBeInTheDocument();
    expect(screen.getByText("PCツールへ進む前に、もう一度保存してください。")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "もう一度保存する" })).toBeEnabled();
    expect(mockConfirmPluExportSaved).not.toHaveBeenCalled();
    expectStatusRegionBeforeContent();
    expect(mockScrollTo).toHaveBeenCalledWith({ top: 0, left: 0, behavior: "smooth" });
  });

  it("REQ-402 shows JAN correction guidance when all prepared targets are excluded", async () => {
    const user = userEvent.setup();
    mockPreparePluExport.mockResolvedValue({
      status: "error",
      error: {
        kind: "validation",
        message:
          "PLUファイルに書き出せる商品がありません。商品マスタで13桁JANを確認してください。対象: JAN-NONE（JAN未登録）、JAN-SHORT（JANが13桁ではありません）",
        field: null,
        error_id: null,
        stocktake_recovery: null,
      },
    });

    renderWithClient(<PluExportPage />);
    await user.click(await screen.findByRole("button", { name: "差分を書き出す" }));

    expect(await screen.findByText("PLU書出しを準備できませんでした")).toBeInTheDocument();
    expect(screen.getByText(/PLUファイルに書き出せる商品がありません/)).toBeInTheDocument();
    expect(screen.getByText(/商品マスタで13桁JANを確認してください/)).toBeInTheDocument();
    expect(screen.getByText(/JAN-NONE/)).toBeInTheDocument();
    expect(screen.getByText(/JAN未登録/)).toBeInTheDocument();
    expect(screen.getByText(/JAN-SHORT/)).toBeInTheDocument();
    expect(screen.getByText(/JANが13桁ではありません/)).toBeInTheDocument();
    expect(mockSave).not.toHaveBeenCalled();
    expect(mockWriteFile).not.toHaveBeenCalled();
    expect(mockConfirmPluExportSaved).not.toHaveBeenCalled();
    expectStatusRegionBeforeContent();
  });

  it("REQ-402 confirms only after operator marks saved PLU file exported", async () => {
    const user = userEvent.setup();
    mockSave.mockResolvedValue("/home/kosei/PLU_20260701.txt");
    mockWriteFile.mockResolvedValue(undefined);

    renderWithClient(<PluExportPage />);
    await user.click(await screen.findByRole("button", { name: "差分を書き出す" }));

    expect(await screen.findByText("PLUファイルを保存しました")).toBeInTheDocument();
    expect(mockSave).toHaveBeenCalledWith({
      defaultPath: "PLU_20260701.txt",
      filters: [{ name: "PLUテキスト", extensions: ["txt"] }],
    });
    expect(mockConfirmPluExportSaved).not.toHaveBeenCalled();
    expect(mockScrollTo).toHaveBeenCalledWith({ top: 0, left: 0, behavior: "smooth" });
    const statusRegion = expectStatusRegionBeforeContent();
    expect(
      within(statusRegion).getByRole("button", { name: "この書出しを未反映から外す" }),
    ).toBeEnabled();
    expect(screen.getByRole("button", { name: "差分を書き出す" })).toHaveAttribute(
      "data-variant",
      "outline",
    );
    expect(
      screen.getAllByText(
        "アプリで確認できるのはPLUファイル保存までです。PCツールへの取込み、SDカード書出し、レジ読込みは手動で確認してください。",
      ).length,
    ).toBeGreaterThanOrEqual(1);

    await user.click(screen.getByRole("button", { name: "この書出しを未反映から外す" }));

    await waitFor(() => {
      expect(mockConfirmPluExportSaved).toHaveBeenCalledWith(
        ["PLU-001"],
        [{ memory_no: 217, row_kind: "product", target_product_codes: ["PLU-001"] }],
      );
    });
    expect(await screen.findByText("未反映から外しました")).toBeInTheDocument();
    expect(mockScrollTo).toHaveBeenCalledWith({ top: 0, left: 0, behavior: "smooth" });
  });

  it("REQ-402 keeps a saved pending export recovery state without PLU file bytes", async () => {
    // 67 処理ステップ 8: 保存形の savedAt は ISO の UTC のまま（表示だけをローカル書式にする）。
    vi.useFakeTimers({ toFake: ["Date"] });
    vi.setSystemTime(new Date("2026-07-01T12:00:00.000Z"));
    try {
      const user = userEvent.setup();
      mockSave.mockResolvedValue("/home/kosei/PLU_20260701.txt");
      mockWriteFile.mockResolvedValue(undefined);

      renderWithClient(<PluExportPage />);
      await user.click(await screen.findByRole("button", { name: "差分を書き出す" }));

      await screen.findByText("PLUファイルを保存しました");
      const rawPending = window.localStorage.getItem(PLU_EXPORT_PENDING_STORAGE_KEY);
      expect(rawPending).toBeTruthy();
      expect(rawPending).not.toContain("QUJD");
      expect(rawPending).not.toContain("bytes_base64");
      expect(JSON.parse(rawPending ?? "{}")).toMatchObject({
        version: 1,
        mode: "diff",
        savedPath: "/home/kosei/PLU_20260701.txt",
        suggestedFilename: "PLU_20260701.txt",
        count: 1,
        encoding: "CP932",
        targetProductCodes: ["PLU-001"],
        preparedRows: [{ memory_no: 217, row_kind: "product", target_product_codes: ["PLU-001"] }],
        overLimitWarning: false,
        savedAt: "2026-07-01T12:00:00.000Z",
      });
    } finally {
      vi.useRealTimers();
    }
  });

  it("REQ-402 shows the last snapshot time as YYYY-MM-DD HH:mm:ss without time zone conversion", async () => {
    // 67 処理ステップ 3: snapshot_at は DB の時差なしの文字列で、Date に通さず T を半角スペースにする。
    // TZ を UTC 以外に固定し、UTC として解釈して直す誤りを runner の TZ に依らず捕まえる（pool は forks が前提）。
    const originalTz = process.env.TZ;
    process.env.TZ = "Asia/Tokyo";
    try {
      mockGetPluSlotSummary.mockResolvedValue({
        status: "ok",
        data: {
          snapshot_at: "2026-08-20T17:34:05",
          free_count: 4_780,
          external_count: 1,
          app_managed_count: 3,
          conflict_count: 0,
          release_pending_count: 0,
        },
      });

      renderWithClient(<PluExportPage />);

      expect(await screen.findByText("最終読込み日時: 2026-08-20 17:34:05")).toBeInTheDocument();
    } finally {
      if (originalTz === undefined) delete process.env.TZ;
      else process.env.TZ = originalTz;
    }
  });

  it("REQ-402 shows the saved time of a restored pending export in local YYYY-MM-DD HH:mm:ss", async () => {
    // 67 処理ステップ 8: savedAt（UTC）をローカル時刻へ直す。日付の境界を跨ぐ値で時差変換漏れを捕まえる。
    // 実行中の process.env.TZ の変更が Date に効くのは vitest の pool が forks（既定）であることに依る。
    const originalTz = process.env.TZ;
    process.env.TZ = "Asia/Tokyo";
    try {
      window.localStorage.setItem(
        PLU_EXPORT_PENDING_STORAGE_KEY,
        pendingExportJson("2026-12-31T15:05:09.000Z"),
      );

      renderWithClient(<PluExportPage />);

      expect(await screen.findByText("保存日時: 2027-01-01 00:05:09")).toBeInTheDocument();
    } finally {
      if (originalTz === undefined) delete process.env.TZ;
      else process.env.TZ = originalTz;
    }
  });

  it("REQ-402 shows an unparsable saved time of a restored pending export as is", async () => {
    window.localStorage.setItem(PLU_EXPORT_PENDING_STORAGE_KEY, pendingExportJson("保存時刻不明"));

    renderWithClient(<PluExportPage />);

    expect(await screen.findByText("保存日時: 保存時刻不明")).toBeInTheDocument();
  });

  it("REQ-402 restores saved pending export after returning to the page", async () => {
    const user = userEvent.setup();
    window.localStorage.setItem(
      PLU_EXPORT_PENDING_STORAGE_KEY,
      JSON.stringify({
        version: 1,
        mode: "diff",
        savedAt: "2026-07-01T12:00:00.000Z",
        savedPath: "/home/kosei/PLU_20260701.txt",
        suggestedFilename: "PLU_20260701.txt",
        count: 1,
        encoding: "CP932",
        targetProductCodes: ["PLU-001"],
        preparedRows: [{ memory_no: 217, row_kind: "product", target_product_codes: ["PLU-001"] }],
        overLimitWarning: false,
      }),
    );

    renderWithClient(<PluExportPage />);

    expect(await screen.findByText("保存済みで未確認のPLU書出しがあります")).toBeInTheDocument();
    // SC20 / DSR-08: 既存文言・roleを保ってwarning variantへ移行する。
    expect(
      screen.getByText("保存済みで未確認のPLU書出しがあります").closest('[data-slot="alert"]'),
    ).toHaveAttribute("data-variant", "warning");
    expect(
      screen.getByText("保存済みで未確認のPLU書出しがあります").closest('[data-slot="alert"]'),
    ).toHaveAttribute("role", "alert");
    const statusRegion = expectStatusRegionBeforeContent();
    expect(
      within(statusRegion).getByText("保存先: /home/kosei/PLU_20260701.txt"),
    ).toBeInTheDocument();
    expect(within(statusRegion).getByText("件数: 1 件")).toBeInTheDocument();
    expect(within(statusRegion).getByText("文字コード: CP932")).toBeInTheDocument();
    await user.click(
      within(statusRegion).getByRole("button", { name: "この書出しを未反映から外す" }),
    );

    await waitFor(() => {
      expect(mockConfirmPluExportSaved).toHaveBeenCalledWith(
        ["PLU-001"],
        [{ memory_no: 217, row_kind: "product", target_product_codes: ["PLU-001"] }],
      );
    });
    expect(window.localStorage.getItem(PLU_EXPORT_PENDING_STORAGE_KEY)).toBeNull();
  });

  it("REQ-402 lets the operator discard a restored pending export and re-export", async () => {
    const user = userEvent.setup();
    window.localStorage.setItem(
      PLU_EXPORT_PENDING_STORAGE_KEY,
      JSON.stringify({
        version: 1,
        mode: "diff",
        savedAt: "2026-07-01T12:00:00.000Z",
        savedPath: "/home/kosei/PLU_20260701.txt",
        suggestedFilename: "PLU_20260701.txt",
        count: 1,
        encoding: "CP932",
        targetProductCodes: ["PLU-001"],
        preparedRows: [{ memory_no: 217, row_kind: "product", target_product_codes: ["PLU-001"] }],
        overLimitWarning: false,
      }),
    );

    renderWithClient(<PluExportPage />);
    const statusRegion = await screen.findByRole("region", { name: "PLU書出し状態" });

    await user.click(within(statusRegion).getByRole("button", { name: "破棄して再書出し" }));

    expect(window.localStorage.getItem(PLU_EXPORT_PENDING_STORAGE_KEY)).toBeNull();
    expect(mockConfirmPluExportSaved).not.toHaveBeenCalled();
    expect(screen.queryByText("保存済みで未確認のPLU書出しがあります")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "差分を書き出す" })).toHaveAttribute(
      "data-variant",
      "default",
    );
  });

  it("REQ-402 clears an invalid saved pending export recovery state", () => {
    window.localStorage.setItem(PLU_EXPORT_PENDING_STORAGE_KEY, "{invalid");

    renderWithClient(<PluExportPage />);

    expect(screen.queryByText("保存済みで未確認のPLU書出しがあります")).not.toBeInTheDocument();
    expect(window.localStorage.getItem(PLU_EXPORT_PENDING_STORAGE_KEY)).toBeNull();
  });

  it("REQ-402 rejects saved pending export recovery state with non-allowed fields", () => {
    window.localStorage.setItem(
      PLU_EXPORT_PENDING_STORAGE_KEY,
      JSON.stringify({
        version: 1,
        mode: "diff",
        savedAt: "2026-07-01T12:00:00.000Z",
        savedPath: "/home/kosei/PLU_20260701.txt",
        suggestedFilename: "PLU_20260701.txt",
        count: 1,
        encoding: "CP932",
        targetProductCodes: ["PLU-001"],
        preparedRows: [{ memory_no: 217, row_kind: "product", target_product_codes: ["PLU-001"] }],
        overLimitWarning: false,
        bytes_base64: "QUJD",
        productName: "赤い毛糸",
        sellingPrice: 550,
      }),
    );

    renderWithClient(<PluExportPage />);

    expect(screen.queryByText("保存済みで未確認のPLU書出しがあります")).not.toBeInTheDocument();
    expect(window.localStorage.getItem(PLU_EXPORT_PENDING_STORAGE_KEY)).toBeNull();
  });

  it("REQ-402 shows confirm failure as a destructive top status with retry", async () => {
    const user = userEvent.setup();
    mockSave.mockResolvedValue("/home/kosei/PLU_20260701.txt");
    mockWriteFile.mockResolvedValue(undefined);
    mockConfirmPluExportSaved.mockRejectedValue(new Error("confirm failed"));

    renderWithClient(<PluExportPage />);
    await user.click(await screen.findByRole("button", { name: "差分を書き出す" }));
    await user.click(await screen.findByRole("button", { name: "この書出しを未反映から外す" }));

    expect(await screen.findByText("未反映から外せませんでした")).toBeInTheDocument();
    const statusRegion = expectStatusRegionBeforeContent();
    expect(
      within(statusRegion).getByText("confirm failed。詳細は診断ログに記録されています。"),
    ).toBeInTheDocument();
    expect(
      within(statusRegion).getByRole("button", { name: "もう一度未反映から外す" }),
    ).toBeEnabled();
    expect(within(statusRegion).queryByText("PLUファイルを保存しました")).not.toBeInTheDocument();
    expect(mockScrollTo).toHaveBeenCalledWith({ top: 0, left: 0, behavior: "smooth" });
  });

  it("REQ-907 D-052-C18 invalidates the slot summary after prepare succeeds", async () => {
    const user = userEvent.setup();
    mockSave.mockResolvedValue(null);

    const { queryClient } = renderWithClient(<PluExportPage />);
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await user.click(await screen.findByRole("button", { name: "差分を書き出す" }));

    await waitFor(() => {
      expectExactInvalidations(invalidateSpy.mock.calls, d052InvalidationOracle.pluExportPrepare());
    });
  });

  it("REQ-402 D-052-C14 invalidates the exact seven-key oracle after confirmation", async () => {
    const user = userEvent.setup();
    mockSave.mockResolvedValue("/home/kosei/PLU_20260701.txt");
    mockWriteFile.mockResolvedValue(undefined);

    const { queryClient } = renderWithClient(<PluExportPage />);
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await user.click(await screen.findByRole("button", { name: "差分を書き出す" }));
    await screen.findByText("PLUファイルを保存しました");
    invalidateSpy.mockClear();
    await user.click(await screen.findByRole("button", { name: "この書出しを未反映から外す" }));

    await waitFor(() => {
      expectExactInvalidations(invalidateSpy.mock.calls, d052InvalidationOracle.pluExportConfirm());
    });
  });

  it("REQ-907 shows Diff / Full import guidance and retires the Full-only warning", async () => {
    const user = userEvent.setup();

    renderWithClient(<PluExportPage />);
    await user.click(await screen.findByRole("button", { name: "全件" }));
    // SC20 / DSR-08: 既存文言・roleを保ってwarning variantへ移行する。
    expect(
      screen.getByText("Diff / Full ともレジへ投入できます").closest('[data-slot="alert"]'),
    ).toHaveAttribute("data-variant", "warning");
    expect(
      screen.getByText("Diff / Full ともレジへ投入できます").closest('[data-slot="alert"]'),
    ).toHaveAttribute("role", "alert");
    expect(screen.getByText("Diff / Full ともレジへ投入できます")).toBeInTheDocument();
    expect(
      screen.getByText("メモリNo.を永続化する前に作成した全件ファイルは再投入しないでください。"),
    ).toBeInTheDocument();
    expect(
      screen.queryByText(new RegExp(["全件書出し", "のファイルだけ"].join(""))),
    ).not.toBeInTheDocument();
  });

  it("REQ-402 shows products excluded from PLU export with Japanese reasons", async () => {
    const user = userEvent.setup();
    mockPreparePluExport.mockResolvedValue({
      status: "ok",
      data: {
        bytes_base64: "QUJD",
        suggested_filename: "PLU_20260701.txt",
        content_type: "text/tab-separated-values",
        encoding: "CP932",
        count: 1,
        target_product_codes: ["PLU-001"],
        prepared_rows: [{ memory_no: 217, row_kind: "product", target_product_codes: ["PLU-001"] }],
        excluded: [
          {
            product_code: "NO-JAN-001",
            jan_code: null,
            name: "JANなし商品",
            memory_no: null,
            reason: "missing_jan",
          },
          {
            product_code: "BAD-CD-001",
            jan_code: "4901234567890",
            name: "検査桁不正",
            memory_no: null,
            reason: "invalid_check_digit",
          },
        ],
        over_limit_warning: false,
      },
    });

    renderWithClient(<PluExportPage />);
    await user.click(await screen.findByRole("button", { name: "差分を書き出す" }));

    expect(
      await screen.findByRole("heading", { name: "書出しに含めなかった商品（要修正）" }),
    ).toBeInTheDocument();
    expect(screen.getByText("NO-JAN-001")).toBeInTheDocument();
    expect(screen.getByText("JAN未登録")).toBeInTheDocument();
    expect(screen.getByText("BAD-CD-001")).toBeInTheDocument();
    expect(screen.getByText("JANのチェックディジットが不正です")).toBeInTheDocument();
  });

  it("REQ-402 falls back for unknown excluded reason", async () => {
    const user = userEvent.setup();
    mockPreparePluExport.mockResolvedValue({
      status: "ok",
      data: {
        bytes_base64: "QUJD",
        suggested_filename: "PLU_20260701.txt",
        content_type: "text/tab-separated-values",
        encoding: "CP932",
        count: 1,
        target_product_codes: ["PLU-001"],
        prepared_rows: [{ memory_no: 217, row_kind: "product", target_product_codes: ["PLU-001"] }],
        excluded: [
          {
            product_code: "UNKNOWN-001",
            jan_code: "4901234567894",
            name: "未知理由",
            memory_no: null,
            reason: "future_reason",
          },
        ],
        over_limit_warning: false,
      },
    });

    renderWithClient(<PluExportPage />);
    await user.click(await screen.findByRole("button", { name: "差分を書き出す" }));

    expect(await screen.findByText("UNKNOWN-001")).toBeInTheDocument();
    expect(screen.getByText("要修正（詳細不明）")).toBeInTheDocument();
  });

  it("REQ-402 places the excluded list above the export content section", async () => {
    // UI-08-D10: 要修正一覧は注意情報としてページ上部圏（コンテンツ 2 カラムより前）に集約する
    const user = userEvent.setup();
    mockPreparePluExport.mockResolvedValue({
      status: "ok",
      data: {
        bytes_base64: "QUJD",
        suggested_filename: "PLU_20260701.txt",
        content_type: "text/tab-separated-values",
        encoding: "CP932",
        count: 1,
        target_product_codes: ["PLU-001"],
        prepared_rows: [{ memory_no: 217, row_kind: "product", target_product_codes: ["PLU-001"] }],
        excluded: [
          {
            product_code: "NO-JAN-001",
            jan_code: null,
            name: "JANなし商品",
            memory_no: null,
            reason: "missing_jan",
          },
        ],
        over_limit_warning: false,
      },
    });

    renderWithClient(<PluExportPage />);
    await user.click(await screen.findByRole("button", { name: "差分を書き出す" }));

    const excludedRegion = await screen.findByRole("region", {
      name: "書出しに含めなかった商品",
    });
    const contentRegion = screen.getByRole("region", { name: "PLU書出し内容" });
    expect(excludedRegion.compareDocumentPosition(contentRegion)).toBe(
      Node.DOCUMENT_POSITION_FOLLOWING,
    );
    expect(
      screen.getByText(/これらの商品は今回のPLUファイルに含めていません。/),
    ).toBeInTheDocument();
    // UI-08-D10 L3 P3: 修正対象は JAN だけでなく売価・税率も含む（除外理由と文言を一致させ、後半を強調表示）
    expect(
      screen.getByText(
        "商品マスタでJANコード・売価・税率を修正すると、次回の書出しから含まれます。",
      ),
    ).toBeInTheDocument();
  });

  it("REQ-402 shows the CV17 recovery note as a separate warning alert after save", async () => {
    // UI-08-D10: 事故防止の回復手順は成功 Alert に埋めず、warning トーンの独立 Alert で強調する
    const user = userEvent.setup();
    mockSave.mockResolvedValue("/home/kosei/PLU_20260701.txt");
    mockWriteFile.mockResolvedValue(undefined);

    renderWithClient(<PluExportPage />);
    await user.click(await screen.findByRole("button", { name: "差分を書き出す" }));

    await screen.findByText("PLUファイルを保存しました");
    expect(screen.getByText("PCツールに取り込めなかった場合の回復手順")).toBeInTheDocument();
    // SC20 / DSR-08: 既存文言・roleを保ってwarning variantへ移行する。
    expect(
      screen.getByText("PCツールに取り込めなかった場合の回復手順").closest('[data-slot="alert"]'),
    ).toHaveAttribute("data-variant", "warning");
    expect(
      screen.getByText("PCツールに取り込めなかった場合の回復手順").closest('[data-slot="alert"]'),
    ).toHaveAttribute("role", "alert");
    expect(
      screen.getByText(
        "PCツールに取り込めなかった場合は、保存済みファイルを再投入するか、差分または全件を書き出し直してください。",
      ),
    ).toBeInTheDocument();
  });

  it("REQ-402 shows the CV17 recovery note in the restored pending export alert", async () => {
    // UI-08-D10: PCツール取込み失敗に気づくのは復帰画面のことが多いため、pending recovery にも回復手順を出す
    window.localStorage.setItem(
      PLU_EXPORT_PENDING_STORAGE_KEY,
      JSON.stringify({
        version: 1,
        mode: "diff",
        savedAt: "2026-07-01T12:00:00.000Z",
        savedPath: "/home/kosei/PLU_20260701.txt",
        suggestedFilename: "PLU_20260701.txt",
        count: 1,
        encoding: "CP932",
        targetProductCodes: ["PLU-001"],
        preparedRows: [{ memory_no: 217, row_kind: "product", target_product_codes: ["PLU-001"] }],
        overLimitWarning: false,
      }),
    );

    renderWithClient(<PluExportPage />);

    expect(await screen.findByText("保存済みで未確認のPLU書出しがあります")).toBeInTheDocument();
    const statusRegion = screen.getByRole("region", { name: "PLU書出し状態" });
    expect(
      within(statusRegion).getByText(
        "PCツールに取り込めなかった場合は、保存済みファイルを再投入するか、差分または全件を書き出し直してください。",
      ),
    ).toBeInTheDocument();
  });

  it("REQ-907 imports FilePicker bytes and unlocks export after snapshot summary refresh", async () => {
    // REQ-907: A-V1 bytes are passed without a path.
    const user = userEvent.setup();
    mockGetPluSlotSummary
      .mockResolvedValueOnce({
        status: "ok",
        data: {
          snapshot_at: null,
          free_count: 0,
          external_count: 0,
          app_managed_count: 0,
          conflict_count: 0,
          release_pending_count: 0,
        },
      })
      .mockResolvedValue({
        status: "ok",
        data: {
          snapshot_at: "2026-08-18T13:00:00",
          free_count: 4_700,
          external_count: 20,
          app_managed_count: 64,
          conflict_count: 1,
          release_pending_count: 0,
        },
      });
    mockOpen.mockResolvedValue("/synthetic/Z004.csv");
    mockReadFile.mockResolvedValue(new Uint8Array([1, 2, 3]));

    const { queryClient } = renderWithClient(<PluExportPage />);
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    expect(await screen.findByText("レジ設定の読込みが必要です")).toBeInTheDocument();
    // SC20 / DSR-08: 既存文言・roleを保ってwarning variantへ移行する。
    expect(
      screen.getByText("レジ設定の読込みが必要です").closest('[data-slot="alert"]'),
    ).toHaveAttribute("data-variant", "warning");
    expect(
      screen.getByText("レジ設定の読込みが必要です").closest('[data-slot="alert"]'),
    ).toHaveAttribute("role", "alert");
    expect(screen.getByRole("button", { name: "差分を書き出す" })).toBeDisabled();

    await user.click(screen.getByRole("button", { name: "レジ登録状況のZ004を選ぶ" }));
    await waitFor(() => {
      expect(mockImportPluRegisterSnapshot).toHaveBeenCalledWith([1, 2, 3]);
    });
    await waitFor(() => {
      expectExactInvalidations(
        invalidateSpy.mock.calls,
        d052InvalidationOracle.pluRegisterSnapshot(),
      );
    });
    expect(await screen.findByText(/最終読込み日時:/)).toBeInTheDocument();
    expect(screen.getByText("4,700 件")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "差分を書き出す" })).toBeEnabled();
  });
});

it("SC20 / REQ-402: over-limit export retains its warning alert", async () => {
  mockPreparePluExport.mockResolvedValue({
    status: "ok",
    data: {
      bytes_base64: "QUJD",
      suggested_filename: "PLU_synthetic.txt",
      content_type: "text/tab-separated-values",
      encoding: "CP932",
      count: 1,
      target_product_codes: ["PLU-001"],
      prepared_rows: [{ memory_no: 217, row_kind: "product", target_product_codes: ["PLU-001"] }],
      excluded: [],
      over_limit_warning: true,
    },
  });
  mockSave.mockResolvedValue(null);
  renderWithClient(<PluExportPage />);
  await userEvent.setup().click(await screen.findByRole("button", { name: "差分を書き出す" }));
  const title = await screen.findByText("スキャニングPLU上限の4,784件を超えています");
  expect(title.closest('[data-slot="alert"]')).toHaveAttribute("data-variant", "warning");
  expect(title.closest('[data-slot="alert"]')).toHaveAttribute("role", "alert");
});

it("⑮ SC4: 確定済みの操作説明を全文表示する", () => {
  renderWithClient(<PluExportPage />);
  expect(
    screen.getByText(
      "レジのPLU登録状況を書き出すページです。『レジ登録状況を読み込む』でレジのCSV（Z004）を読み込み、空き・外部登録・アプリ管理・競合の件数を確認します。『差分を書き出す』（未反映の商品だけ）か『全件を書き出す』を選んで保存し、『この書出しを未反映から外す』を押して確定します。",
    ),
  ).toBeInTheDocument();
});

// UI-08 / REQ-402 / SPEC-UNIT-D3 の表の 9: 未書出しの一覧の在庫は、長さの商品だけ m の形
it("REQ-402: 未書出しの一覧は m の商品の在庫 2750 を 27.5 m、個数の商品は今どおりの数で出す", async () => {
  mockDefaultCommands();
  const base = {
    jan_code: "4900000000001",
    department_id: 1,
    selling_price: 550,
    cost_price_centi: 300,
    plu_dirty: true,
    plu_exported_at: null,
  };
  mockListPluDirty.mockResolvedValue({
    status: "ok",
    data: [
      { ...base, product_code: "PLU-M", name: "合成の反物", stock_unit: "m", stock_quantity: 2750 },
      {
        ...base,
        product_code: "PLU-PCS",
        name: "合成の小物",
        stock_unit: "pcs",
        stock_quantity: 12,
      },
    ],
  });
  renderWithClient(<PluExportPage />);

  const meterRow = (await screen.findByText("合成の反物")).closest("tr");
  const pieceRow = screen.getByText("合成の小物").closest("tr");
  expect(meterRow?.querySelector("td:last-child")).toHaveTextContent(/^27\.5 m$/);
  expect(pieceRow?.querySelector("td:last-child")).toHaveTextContent(/^12$/);
});
