// REQ-208 / 65 §65.5b（D-163）: 入出庫の記録の詳細からの取消（Test Plan T10・T12）。
// 期待値は 65 §65.5b・79 §79.8 から写す。C25 の oracle は src/test/invalidation-oracle.ts。値は全て合成。

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, createRouter, RouterProvider } from "@tanstack/react-router";
import { configure, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState, type ReactElement } from "react";
import { toast } from "sonner";
import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  makeMockProductWithRelations,
  makeMockStockDetail,
} from "@/features/stock-inquiry/lib/test-fixtures";
import { commands } from "@/lib/bindings";
import type {
  CmdError,
  DisposalRecordDetail,
  ManualSaleRecordDetail,
  MovementRecord,
  ProductStockUnit,
  ReceivingRecordDetail,
  ReferenceType,
  ReturnRecordDetail,
  TransferRecordDetail,
} from "@/lib/bindings";
import { queryKeys } from "@/lib/query-keys";
import { getWindowCloseGuardKinds } from "@/lib/window-close-guard";
import { routeTree } from "@/routeTree.gen";
import { d052InvalidationOracle, expectExactInvalidations } from "@/test/invalidation-oracle";
import { renderWithRouter } from "@/test/render-with-router";
import { DisposalRecordDetailPage } from "./DisposalRecordDetailPage";
import { ManualSaleRecordDetailPage } from "./ManualSaleRecordDetailPage";
import { ReceivingRecordDetailPage } from "./ReceivingRecordDetailPage";
import { ReturnRecordDetailPage } from "./ReturnRecordDetailPage";
import { TransferRecordDetailPage } from "./TransferRecordDetailPage";

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ setTitle: vi.fn().mockResolvedValue(undefined) }),
}));

vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => ({ setZoom: vi.fn().mockResolvedValue(undefined) }),
}));

vi.mock("@/lib/bindings", () => ({
  commands: {
    getReceivingRecord: vi.fn(),
    getReturnRecord: vi.fn(),
    getManualSaleRecord: vi.fn(),
    getDisposalRecord: vi.fn(),
    getTransferRecord: vi.fn(),
    cancelInventoryRecord: vi.fn(),
    getStockDetail: vi.fn(),
    listReceivings: vi.fn(),
    listReturns: vi.fn(),
    listDisposals: vi.fn(),
    listSuppliers: vi.fn(),
    listInventoryRecords: vi.fn(),
    listDepartments: vi.fn(),
  },
}));

vi.mock("sonner", async (importOriginal) => {
  const actual = await importOriginal<typeof import("sonner")>();
  return {
    ...actual,
    toast: { success: vi.fn(), error: vi.fn(), info: vi.fn(), dismiss: vi.fn() },
  };
});

const mockCancel = vi.mocked(commands.cancelInventoryRecord);
const mockGetStockDetail = vi.mocked(commands.getStockDetail);
const mockToast = vi.mocked(toast);

const RECORD_ID = 5;
const BUSINESS_DATE = "2026-11-18";
const REASON = "打ち間違い";
const CANCELED_AT = "2026-11-21T10:00:00";
const FAILED_TOAST = "取り消しに失敗しました。もう一度お試しください";
const UNCONFIRMED_TEXT =
  "取り消せたか確かめられませんでした。「状態をもう一度確かめる」を押してください";
const READING_STOCK_TEXT = "取り消した後の在庫を確かめています…";
const STOCK_ALERT_TITLE = "取り消した後の在庫を確かめてください";
const CANCELED_EMPTY_TEXT = "取り消したので、この記録は在庫を動かしていません";
const REREAD_TEXT = "詳細を読み直せませんでした。「詳細を読み直す」を押してください";

type Reply<T> = { status: "ok"; data: T } | { status: "error"; error: CmdError };

function ok<T>(data: T): Reply<T> {
  return { status: "ok", data };
}

function cmdError(kind: CmdError["kind"], message: string): { status: "error"; error: CmdError } {
  return {
    status: "error",
    error: { kind, message, field: null, error_id: null, stocktake_recovery: null },
  };
}

function movement(
  recordType: ReferenceType,
  productCode: string,
  id = 900,
  quantity = 3,
): MovementRecord {
  return {
    id,
    product_code: productCode,
    movement_type: "receiving",
    quantity,
    stock_after: 13,
    reference_type: recordType,
    reference_id: RECORD_ID,
    source: null,
    note: null,
    created_at: "2026-11-18T10:00:00",
  };
}

/** 有効か取消済みかで変わる部分（取消済みは無効にした movement を返さない） */
function state(canceled: boolean, movements: MovementRecord[]) {
  return canceled
    ? { status: "canceled", canceled_at: CANCELED_AT, cancel_reason: REASON, movements: [] }
    : { status: "active", canceled_at: null, cancel_reason: null, movements };
}

const item = { id: 1, product_code: "RC-001", product_name: "合成の商品", department_name: "合成" };

function receivingDetail(canceled = false): ReceivingRecordDetail {
  return {
    id: RECORD_ID,
    receiving_date: BUSINESS_DATE,
    supplier_id: null,
    supplier_name: null,
    note: null,
    created_at: "2026-11-18T10:00:00",
    items: [
      { ...item, stock_unit: "pcs", quantity: 3, cost_price_centi: 300, line_cost_centi: "900" },
    ],
    total_cost: 9,
    ...state(canceled, [movement("receiving_record", "RC-001")]),
  };
}

function returnDetail(canceled = false): ReturnRecordDetail {
  return {
    id: RECORD_ID,
    return_type: "return",
    return_date: BUSINESS_DATE,
    register_processed: false,
    receipt_image_path: null,
    note: null,
    created_at: "2026-11-18T10:00:00",
    items: [{ ...item, stock_unit: "pcs", direction: "in", quantity: 3 }],
    ...state(canceled, [movement("return_record", "RC-001")]),
  };
}

function manualSaleDetail(canceled = false): ManualSaleRecordDetail {
  return {
    id: RECORD_ID,
    sale_date: BUSINESS_DATE,
    reason: "plu_unregistered",
    note: null,
    created_at: "2026-11-18T10:00:00",
    items: [{ ...item, stock_unit: "pcs", quantity: 1, amount: 500 }],
    total_amount: 500,
    ...state(canceled, [movement("manual_sale", "RC-001", 900, -1)]),
  };
}

function disposalDetail(canceled = false): DisposalRecordDetail {
  return {
    id: RECORD_ID,
    disposal_date: BUSINESS_DATE,
    created_at: "2026-11-18T10:00:00",
    items: [
      {
        ...item,
        stock_unit: "pcs",
        disposal_type: "disposal",
        quantity: 1,
        cost_price_centi: 300,
        reason: "汚れ",
        line_loss_cost_centi: "300",
      },
    ],
    total_loss_cost: 3,
    ...state(canceled, [movement("disposal_record", "RC-001", 900, -1)]),
  };
}

function transferDetail(canceled = false): TransferRecordDetail {
  return {
    id: RECORD_ID,
    transfer_date: BUSINESS_DATE,
    created_at: "2026-11-18T10:00:00",
    items: [
      { ...item, stock_unit: "pcs", direction: "out", quantity: 2 },
      { ...item, id: 2, product_code: "RC-002", stock_unit: "pcs", direction: "in", quantity: 4 },
    ],
    ...state(canceled, [
      movement("transfer_record", "RC-001", 900, -2),
      movement("transfer_record", "RC-002", 901, 4),
    ]),
  };
}

/** 5 つの詳細（同じ部品と hook を使う）。`read` はその詳細の読取りの mock */
const PAGES = [
  {
    label: "入庫",
    recordType: "receiving_record",
    page: <ReceivingRecordDetailPage recordId={RECORD_ID} />,
    read: vi.mocked(commands.getReceivingRecord) as ReturnType<typeof vi.fn>,
    detail: receivingDetail,
  },
  {
    label: "返品・交換",
    recordType: "return_record",
    page: <ReturnRecordDetailPage recordId={RECORD_ID} />,
    read: vi.mocked(commands.getReturnRecord) as ReturnType<typeof vi.fn>,
    detail: returnDetail,
  },
  {
    label: "手動販売",
    recordType: "manual_sale",
    page: <ManualSaleRecordDetailPage recordId={RECORD_ID} />,
    read: vi.mocked(commands.getManualSaleRecord) as ReturnType<typeof vi.fn>,
    detail: manualSaleDetail,
  },
  {
    label: "廃棄・破損",
    recordType: "disposal_record",
    page: <DisposalRecordDetailPage recordId={RECORD_ID} />,
    read: vi.mocked(commands.getDisposalRecord) as ReturnType<typeof vi.fn>,
    detail: disposalDetail,
  },
  {
    label: "詰め替え・切り分け",
    recordType: "transfer_record",
    page: <TransferRecordDetailPage recordId={RECORD_ID} />,
    read: vi.mocked(commands.getTransferRecord) as ReturnType<typeof vi.fn>,
    detail: transferDetail,
  },
] as const;

const receiving = PAGES[0];
const RECEIVING_DETAIL_KEY = queryKeys.inventoryRecords.receivingDetail(RECORD_ID);

function makeQueryClient() {
  return new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
}

function renderPage(page: ReactElement, queryClient = makeQueryClient()) {
  return renderWithRouter(<QueryClientProvider client={queryClient}>{page}</QueryClientProvider>);
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

/** 「この記録を取り消す」から確認を開き、理由を入れる。dialog を返す */
async function openDialog(user: ReturnType<typeof userEvent.setup>, reason = REASON) {
  await user.click(await screen.findByRole("button", { name: "この記録を取り消す" }));
  const dialog = screen.getByRole("alertdialog");
  if (reason !== "") await user.type(within(dialog).getByLabelText("取り消す理由（必須）"), reason);
  return dialog;
}

function stockOf(stocks: Partial<Record<string, { stock: number; unit?: ProductStockUnit }>>) {
  mockGetStockDetail.mockImplementation((code: string) => {
    const entry = stocks[code];
    return Promise.resolve(
      entry === undefined
        ? cmdError("internal", "合成の読取りの失敗")
        : ok(
            makeMockStockDetail({
              product: makeMockProductWithRelations({
                product_code: code,
                stock_quantity: entry.stock,
                stock_unit: entry.unit ?? "pcs",
              }),
            }),
          ),
    );
  });
}

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(commands.listInventoryRecords).mockResolvedValue(
    ok({ items: [], total_count: 0, page: 1, per_page: 50 }),
  );
  vi.mocked(commands.listDepartments).mockResolvedValue(ok([]));
  vi.mocked(commands.listSuppliers).mockResolvedValue(ok([]));
});

describe.each(PAGES)(
  "$label の記録の詳細の取消（REQ-208 / 65 §65.5b / T10）",
  ({ recordType, page, read, detail }) => {
    it("取消済みの記録には取消の button を出さず、文字の「取消済み」・取消の日時と理由・空の文を出す", async () => {
      read.mockResolvedValue(ok(detail(true)));
      renderPage(page);

      expect(await screen.findByText("この記録は取消済みです")).toBeInTheDocument();
      expect(screen.queryByRole("button", { name: "この記録を取り消す" })).toBeNull();
      // 状態の badge は文字（色だけにしない）
      expect(screen.getByText("取消済み")).toBeInTheDocument();
      expect(screen.getByText("2026-11-21 10:00:00")).toBeInTheDocument();
      expect(screen.getByText(REASON)).toBeInTheDocument();
      expect(
        screen.getByText("取り消したので、この記録は在庫を動かしていません"),
      ).toBeInTheDocument();
      // 振替の詳細も取消の代わりの案内を出さない（79 §79.8）。訂正の操作は持たない（TRACE-D5）
      expect(screen.queryByText("取消はできません")).toBeNull();
      expect(screen.queryByText(/入れ替えて、もう一度/)).toBeNull();
      expect(screen.queryByRole("button", { name: /訂正/ })).toBeNull();
    });

    it("dialog は戻す在庫・入れ直しの日付・棚卸しの評価額を言い、理由が空の間は「取り消す」を押せない", async () => {
      const user = userEvent.setup();
      read.mockResolvedValue(ok(detail()));
      renderPage(page);

      const dialog = await openDialog(user, "");
      expect(within(dialog).getByText(/この記録で動いた在庫を元に戻します。/)).toBeInTheDocument();
      expect(
        within(dialog).getByText(
          `記録は消さずに「取消済み」で残ります。取消を元に戻す操作はありません。打ち間違いを直すときは、取り消した後に作成の画面で入れ直してください。日付の打ち間違いでなければ、日付は ${BUSINESS_DATE} を選んでください（作成の画面の日付は今日が既定です）`,
        ),
      ).toBeInTheDocument();
      expect(
        within(dialog).getByText(
          "完了した棚卸しの評価額は変わりません（棚卸しの記録の詳細の差異・ロス原価・参考の差は変わることがあります）",
        ),
      ).toBeInTheDocument();
      // 売上の文は手動販売だけ
      const salesText = within(dialog).queryByText(
        new RegExp(`${BUSINESS_DATE} の日次売上と月次の売上から外れます。`),
      );
      expect(salesText !== null).toBe(recordType === "manual_sale");
      expect(screen.queryByText("取消はできません")).toBeNull();

      const submit = within(dialog).getByRole("button", { name: "取り消す" });
      expect(submit).toBeDisabled();
      // 半角・全角の空白だけも空
      await user.type(within(dialog).getByLabelText("取り消す理由（必須）"), " \u3000");
      expect(submit).toBeDisabled();
      await user.click(submit);
      expect(mockCancel).not.toHaveBeenCalled();
    });

    it("成功: 1 回だけ送り、送信中は離脱と close を止め、C25 で再取得して取消済みと負の在庫の文を出す", async () => {
      const user = userEvent.setup();
      read.mockResolvedValueOnce(ok(detail())).mockResolvedValue(ok(detail(true)));
      const reply = deferred<Awaited<ReturnType<typeof commands.cancelInventoryRecord>>>();
      mockCancel.mockReturnValueOnce(reply.promise);
      const queryClient = makeQueryClient();
      const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
      renderPage(page, queryClient);

      const dialog = await openDialog(user, `  ${REASON}\u3000`);
      await user.click(within(dialog).getByRole("button", { name: "取り消す" }));

      // 送信中: 「取り消す」をもう一度押せず、close の止めが入る
      const pending = await within(dialog).findByRole("button", { name: /取り消し中…/ });
      expect(pending).toBeDisabled();
      await user.click(pending);
      expect(getWindowCloseGuardKinds()).toEqual(["inventory-record-cancel"]);

      reply.resolve(ok({ stock_warnings: ["RC-001: 在庫がマイナスになりました（-2）"] }));

      expect(await screen.findByText("この記録は取消済みです")).toBeInTheDocument();
      expect(mockCancel).toHaveBeenCalledTimes(1);
      // 理由は前後の空白を除いて送る
      expect(mockCancel).toHaveBeenCalledWith(recordType, RECORD_ID, REASON);
      expect(mockToast.success).toHaveBeenCalledWith("記録を取り消しました", { duration: 8000 });
      await waitFor(() => {
        expectExactInvalidations(
          invalidateSpy.mock.calls,
          d052InvalidationOracle.inventoryRecordCancel(),
        );
      });
      expect(screen.queryByRole("alertdialog")).toBeNull();
      expect(screen.getByText(REASON)).toBeInTheDocument();
      // 成功の応答の stock_warnings（商品コードと数量）を取消済みの詳細に出す
      expect(screen.getByText("RC-001: 在庫がマイナスになりました（-2）")).toBeInTheDocument();
      expect(screen.queryByRole("button", { name: "この記録を取り消す" })).toBeNull();
      // 応答の後に close の止めは解ける
      await waitFor(() => {
        expect(getWindowCloseGuardKinds()).toEqual([]);
      });
    });

    it("成功の後に詳細の再取得だけが失敗: 取消済みと負の在庫の文を残し、詳細だけを読み直せる", async () => {
      const user = userEvent.setup();
      const warning = "RC-001: 在庫がマイナスになりました（-2）";
      read
        .mockResolvedValueOnce(ok(detail()))
        .mockResolvedValueOnce(cmdError("internal", "合成の読取りの失敗"))
        .mockResolvedValue(ok(detail(true)));
      mockCancel.mockResolvedValue(ok({ stock_warnings: [warning] }));
      renderPage(page);

      const dialog = await openDialog(user);
      await user.click(within(dialog).getByRole("button", { name: "取り消す" }));

      // 確かめた取消の結果は詳細の読取りと別に持つ（読取りの失敗で失わない）
      expect(await screen.findByText(REREAD_TEXT)).toBeInTheDocument();
      expect(screen.getByText("この記録は取消済みです")).toBeInTheDocument();
      expect(screen.getByText(warning)).toBeInTheDocument();
      expect(mockToast.success).toHaveBeenCalledWith("記録を取り消しました", { duration: 8000 });
      // 開き直させない。取消の前の詳細（取消の button）も出さない
      expect(
        screen.queryByText("記録IDを確認するか、入出庫履歴から開き直してください。"),
      ).toBeNull();
      expect(screen.queryByRole("button", { name: "この記録を取り消す" })).toBeNull();
      await waitFor(() => {
        expect(getWindowCloseGuardKinds()).toEqual([]);
      });

      // 詳細だけを読み直す（取消は送らず、在庫も読まない）
      await user.click(screen.getByRole("button", { name: "詳細を読み直す" }));
      expect(await screen.findByText("2026-11-21 10:00:00")).toBeInTheDocument();
      expect(screen.getByText(REASON)).toBeInTheDocument();
      expect(screen.getByText(warning)).toBeInTheDocument();
      expect(screen.queryByText(REREAD_TEXT)).toBeNull();
      expect(read).toHaveBeenCalledTimes(3);
      expect(mockCancel).toHaveBeenCalledTimes(1);
      expect(mockGetStockDetail).not.toHaveBeenCalled();
    });

    it("成功: 再取得を待つ間も「有効」と取消の前の movement を出さず、その間に開き直しても「この記録を取り消す」を出さない", async () => {
      const user = userEvent.setup();
      const warning = "RC-001: 在庫がマイナスになりました（-2）";
      // C25 の再取得は終わらせない
      const refresh = deferred<unknown>();
      read.mockResolvedValueOnce(ok(detail())).mockReturnValue(refresh.promise);
      mockCancel.mockResolvedValue(ok({ stock_warnings: [warning] }));
      const queryClient = makeQueryClient();
      const first = renderPage(page, queryClient);

      const dialog = await openDialog(user);
      await user.click(within(dialog).getByRole("button", { name: "取り消す" }));
      expect(await screen.findByText("この記録は取消済みです")).toBeInTheDocument();

      const expectCanceledView = () => {
        expect(screen.getByText("取消済み")).toBeInTheDocument();
        expect(screen.queryByText("有効")).toBeNull();
        expect(screen.getByText(CANCELED_EMPTY_TEXT)).toBeInTheDocument();
        // 取消の日時・理由は再取得した詳細が出す（それまで空の値を出さない）
        expect(screen.queryByText("取消の日時")).toBeNull();
        expect(screen.queryByRole("button", { name: "この記録を取り消す" })).toBeNull();
      };
      expectCanceledView();
      expect(screen.getByText(warning)).toBeInTheDocument();

      // 再取得が終わる前に画面を離れて、同じ詳細を開き直す
      first.unmount();
      renderPage(page, queryClient);
      expect(await screen.findByText("この記録は取消済みです")).toBeInTheDocument();
      expectCanceledView();
      // 成功の応答の負の在庫の文は、取り消したその画面の知らせ
      expect(screen.queryByText(warning)).toBeNull();
      expect(mockCancel).toHaveBeenCalledTimes(1);

      refresh.resolve(ok(detail(true)));
      expect(await screen.findByText("2026-11-21 10:00:00")).toBeInTheDocument();
      expect(screen.getByText(REASON)).toBeInTheDocument();
    });

    it("取消と取り直しの両方が失敗: 詳細の状態は「結果未確認」で、「有効」と言い切らない", async () => {
      const user = userEvent.setup();
      read
        .mockResolvedValueOnce(ok(detail()))
        .mockResolvedValue(cmdError("internal", "合成の読取りの失敗"));
      mockCancel.mockRejectedValue(new Error("合成の応答の消失"));
      renderPage(page);

      const dialog = await openDialog(user);
      await user.click(within(dialog).getByRole("button", { name: "取り消す" }));
      expect(await within(dialog).findByText(UNCONFIRMED_TEXT)).toBeInTheDocument();

      // dialog の後ろの詳細（支援技術からは隠れる）も、cache の active を「有効」と出さない
      expect(screen.getByText("結果未確認")).toBeInTheDocument();
      expect(screen.queryByText("有効")).toBeNull();
    });
  },
);

describe("取消の失敗・結果不明の後の取り直し（REQ-208 / 65 §65.5b / T10）", () => {
  it("取り直しが active: 拒否の文か失敗の Toast を出し、dialog と理由を残して同じ記録を再試行できる", async () => {
    const user = userEvent.setup();
    receiving.read.mockResolvedValue(ok(receivingDetail()));
    mockCancel
      .mockResolvedValueOnce(cmdError("internal", "合成の取消の失敗"))
      .mockResolvedValueOnce(cmdError("validation", "在庫数が大きすぎます"))
      .mockResolvedValue(ok({ stock_warnings: [] }));
    renderPage(receiving.page);

    const dialog = await openDialog(user);
    const submit = within(dialog).getByRole("button", { name: "取り消す" });
    await user.click(submit);
    await waitFor(() => {
      expect(mockToast.error).toHaveBeenCalledWith(FAILED_TOAST);
    });
    // 失敗のたびに同じ詳細を取り直す
    expect(receiving.read).toHaveBeenCalledTimes(2);
    expect(within(dialog).getByLabelText("取り消す理由（必須）")).toHaveValue(REASON);

    // 拒否の応答は、届いた理由の文を出す
    await user.click(submit);
    await waitFor(() => {
      expect(mockToast.error).toHaveBeenCalledWith("在庫数が大きすぎます");
    });
    expect(receiving.read).toHaveBeenCalledTimes(3);
    expect(screen.getByRole("alertdialog")).toBeInTheDocument();
    expect(within(dialog).getByLabelText("取り消す理由（必須）")).toHaveValue(REASON);

    await user.click(submit);
    await waitFor(() => {
      expect(mockCancel).toHaveBeenCalledTimes(3);
    });
    expect(mockCancel).toHaveBeenLastCalledWith("receiving_record", RECORD_ID, REASON);
  });

  it("取り直しが canceled: dialog を閉じて C25 を当て、取消の前の movement の商品だけ今の在庫を読んで負の在庫を知らせる", async () => {
    const user = userEvent.setup();
    // 関連 movement のある 3 つ（-1・0・長さの -30 cm）と、movement の無い明細（レジで読んで戻した商品、-1）
    const active: ReturnRecordDetail = {
      ...returnDetail(),
      items: [
        { ...item, id: 1, product_code: "RT-A", stock_unit: "pcs", direction: "in", quantity: 1 },
        { ...item, id: 2, product_code: "RT-Z", stock_unit: "pcs", direction: "in", quantity: 1 },
        { ...item, id: 3, product_code: "RT-L", stock_unit: "m", direction: "in", quantity: 30 },
        { ...item, id: 4, product_code: "RT-C", stock_unit: "pcs", direction: "in", quantity: 1 },
      ],
      movements: [
        movement("return_record", "RT-A", 901),
        movement("return_record", "RT-Z", 902),
        movement("return_record", "RT-L", 903),
      ],
    };
    const returns = PAGES[1];
    // 取り直しは無効にした movement を返さない（実際の読取りは is_voided = 0）
    returns.read
      .mockResolvedValueOnce(ok(active))
      .mockResolvedValue(ok({ ...active, ...state(true, []) } satisfies ReturnRecordDetail));
    // commit の後に応答を失い、もう一度押したら拒否の応答が届いた形
    mockCancel.mockResolvedValue(
      cmdError("validation", `この記録は ${CANCELED_AT} に取り消し済みです`),
    );
    stockOf({
      "RT-A": { stock: -1 },
      "RT-Z": { stock: 0 },
      "RT-L": { stock: -30, unit: "m" },
      "RT-C": { stock: -1 },
    });
    const queryClient = makeQueryClient();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    renderPage(returns.page, queryClient);

    const dialog = await openDialog(user);
    await user.click(within(dialog).getByRole("button", { name: "取り消す" }));

    expect(await screen.findByText("この記録は取消済みです")).toBeInTheDocument();
    expect(screen.queryByRole("alertdialog")).toBeNull();
    // 拒否の応答で届いた文はそのまま出す（失敗の Toast は出さない）
    expect(mockToast.info).toHaveBeenCalledWith(`この記録は ${CANCELED_AT} に取り消し済みです`);
    expect(mockToast.error).not.toHaveBeenCalled();
    await waitFor(() => {
      expectExactInvalidations(
        invalidateSpy.mock.calls,
        d052InvalidationOracle.inventoryRecordCancel(),
      );
    });
    // 作成の画面と同じ文。長さの商品は m。0 は知らせない
    expect(await screen.findByText("RT-A: 在庫がマイナスになりました（-1）")).toBeInTheDocument();
    expect(
      await screen.findByText("RT-L: 在庫がマイナスになりました（-0.3 m）"),
    ).toBeInTheDocument();
    expect(screen.queryByText(/RT-Z: /)).toBeNull();
    // movement の無い明細の商品は取消が在庫を動かしていないので、読まず、知らせない
    expect(screen.queryByText(/RT-C: /)).toBeNull();
    expect(new Set(mockGetStockDetail.mock.calls.map(([code]) => code))).toEqual(
      new Set(["RT-A", "RT-Z", "RT-L"]),
    );
  });

  it("取り直しが canceled で在庫を読めない商品: 取消済みの表示を保ち、その商品に読めなかったことを出して、その商品だけ読み直せる", async () => {
    const user = userEvent.setup();
    const transfers = PAGES[4];
    transfers.read
      .mockResolvedValueOnce(ok(transferDetail()))
      .mockResolvedValue(ok(transferDetail(true)));
    // 応答が届かない
    mockCancel.mockRejectedValue(new Error("合成の応答の消失"));
    // 増やした商品（RC-002）だけ読めない
    stockOf({ "RC-001": { stock: 8 } });
    // 取消の前に在庫照会で読んだ RC-002 の在庫が cache に残っている
    const queryClient = makeQueryClient();
    queryClient.setQueryData(
      queryKeys.stockInquiry.detail("RC-002"),
      makeMockStockDetail({
        product: makeMockProductWithRelations({ product_code: "RC-002", stock_quantity: -9 }),
      }),
    );
    renderPage(transfers.page, queryClient);

    const dialog = await openDialog(user);
    await user.click(within(dialog).getByRole("button", { name: "取り消す" }));

    expect(await screen.findByText("この記録は取消済みです")).toBeInTheDocument();
    expect(await screen.findByText("RC-002: 在庫を確かめられませんでした")).toBeInTheDocument();
    expect(screen.getByText("この記録は取消済みです")).toBeInTheDocument();
    expect(screen.queryByText(/RC-001: /)).toBeNull();
    // 読めなかった商品に、cache の前の数（-9）を今の在庫として出さない
    expect(screen.queryByText(/RC-002: 在庫がマイナスになりました/)).toBeNull();
    expect(mockToast.info).not.toHaveBeenCalled();

    // その商品だけ読み直す
    mockGetStockDetail.mockClear();
    stockOf({ "RC-001": { stock: 8 }, "RC-002": { stock: -2 } });
    await user.click(screen.getByRole("button", { name: "RC-002 の在庫をもう一度確かめる" }));

    expect(await screen.findByText("RC-002: 在庫がマイナスになりました（-2）")).toBeInTheDocument();
    expect(screen.queryByText("RC-002: 在庫を確かめられませんでした")).toBeNull();
    expect(mockGetStockDetail.mock.calls.map(([code]) => code)).toEqual(["RC-002"]);
    expect(screen.getByText("この記録は取消済みです")).toBeInTheDocument();
  });

  it("取消と取り直しの両方が失敗: 言い切らずに dialog と理由を残し、状態を確かめ直して active なら同じ理由で再試行できる", async () => {
    const user = userEvent.setup();
    receiving.read
      .mockResolvedValueOnce(ok(receivingDetail()))
      .mockResolvedValueOnce(cmdError("internal", "合成の読取りの失敗"))
      .mockResolvedValueOnce(cmdError("internal", "合成の読取りの失敗"))
      .mockResolvedValue(ok(receivingDetail()));
    mockCancel
      .mockRejectedValueOnce(new Error("合成の応答の消失"))
      .mockResolvedValue(ok({ stock_warnings: [] }));
    renderPage(receiving.page);

    const dialog = await openDialog(user);
    const submit = within(dialog).getByRole("button", { name: "取り消す" });
    await user.click(submit);

    expect(await within(dialog).findByText(UNCONFIRMED_TEXT)).toBeInTheDocument();
    // 詳細は error の表示に替えず、dialog と理由を残す。Toast で成否を言わない
    // 開いている dialog の後ろは支援技術から隠れるので、hidden を含めて探す
    expect(
      screen.getByRole("heading", { name: `入庫記録 #${String(RECORD_ID)}`, hidden: true }),
    ).toBeInTheDocument();
    expect(screen.queryByText("記録IDを確認するか、入出庫履歴から開き直してください。")).toBeNull();
    expect(within(dialog).getByLabelText("取り消す理由（必須）")).toHaveValue(REASON);
    expect(mockToast.error).not.toHaveBeenCalled();
    expect(mockToast.success).not.toHaveBeenCalled();
    // active を確かめるまで「取り消す」は押せない
    expect(submit).toBeDisabled();
    await user.click(submit);
    expect(mockCancel).toHaveBeenCalledTimes(1);

    // 「状態をもう一度確かめる」は読取りだけ（取消は送らない）。まだ読めなければ同じ表示のまま
    const recheck = within(dialog).getByRole("button", { name: "状態をもう一度確かめる" });
    await user.click(recheck);
    await waitFor(() => {
      expect(receiving.read).toHaveBeenCalledTimes(3);
    });
    expect(mockCancel).toHaveBeenCalledTimes(1);
    expect(submit).toBeDisabled();

    // 読めて active: 失敗の Toast を出し、同じ理由で「取り消す」を押せる
    await user.click(within(dialog).getByRole("button", { name: "状態をもう一度確かめる" }));
    await waitFor(() => {
      expect(mockToast.error).toHaveBeenCalledWith(FAILED_TOAST);
    });
    expect(within(dialog).queryByRole("button", { name: "状態をもう一度確かめる" })).toBeNull();
    expect(within(dialog).getByLabelText("取り消す理由（必須）")).toHaveValue(REASON);
    expect(submit).toBeEnabled();
    expect(mockCancel).toHaveBeenCalledTimes(1);
    await user.click(submit);
    await waitFor(() => {
      expect(mockCancel).toHaveBeenCalledTimes(2);
    });
    expect(mockCancel).toHaveBeenLastCalledWith("receiving_record", RECORD_ID, REASON);
  });

  it("取消と取り直しの両方が失敗した後に、状態を確かめ直して canceled なら取消済みの表示にする", async () => {
    const user = userEvent.setup();
    receiving.read
      .mockResolvedValueOnce(ok(receivingDetail()))
      .mockResolvedValueOnce(cmdError("internal", "合成の読取りの失敗"))
      .mockResolvedValue(ok(receivingDetail(true)));
    mockCancel.mockRejectedValue(new Error("合成の応答の消失"));
    stockOf({ "RC-001": { stock: -1 } });
    const queryClient = makeQueryClient();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    renderPage(receiving.page, queryClient);

    const dialog = await openDialog(user);
    await user.click(within(dialog).getByRole("button", { name: "取り消す" }));
    await user.click(await within(dialog).findByRole("button", { name: "状態をもう一度確かめる" }));

    expect(await screen.findByText("この記録は取消済みです")).toBeInTheDocument();
    expect(screen.queryByRole("alertdialog")).toBeNull();
    expect(mockCancel).toHaveBeenCalledTimes(1);
    await waitFor(() => {
      expectExactInvalidations(
        invalidateSpy.mock.calls,
        d052InvalidationOracle.inventoryRecordCancel(),
      );
    });
    expect(await screen.findByText("RC-001: 在庫がマイナスになりました（-1）")).toBeInTheDocument();
  });

  it("未確認のまま画面を離れて同じ詳細を開き直しても、状態を確かめ直すまで「取り消す」を押せない", async () => {
    const user = userEvent.setup();
    receiving.read
      .mockResolvedValueOnce(ok(receivingDetail()))
      .mockResolvedValueOnce(cmdError("internal", "合成の読取りの失敗"))
      .mockResolvedValue(ok(receivingDetail()));
    const returns = PAGES[1];
    returns.read.mockResolvedValue(ok(returnDetail()));
    mockCancel.mockRejectedValue(new Error("合成の応答の消失"));
    const queryClient = makeQueryClient();
    const first = renderPage(receiving.page, queryClient);

    const dialog = await openDialog(user);
    await user.click(within(dialog).getByRole("button", { name: "取り消す" }));
    expect(await within(dialog).findByText(UNCONFIRMED_TEXT)).toBeInTheDocument();
    first.unmount();

    // 未確認は記録の種類と ID ごと: 同じ ID の別の種類の記録には及ばない
    const other = renderPage(returns.page, queryClient);
    const otherDialog = await openDialog(user);
    expect(within(otherDialog).queryByText(UNCONFIRMED_TEXT)).toBeNull();
    expect(within(otherDialog).getByRole("button", { name: "取り消す" })).toBeEnabled();
    other.unmount();

    // 開き直した詳細は cache の active を出す（読み直していない）が、「取り消す」は押せない
    renderPage(receiving.page, queryClient);
    const reopened = await openDialog(user);
    expect(receiving.read).toHaveBeenCalledTimes(2);
    expect(within(reopened).getByText(UNCONFIRMED_TEXT)).toBeInTheDocument();
    const submit = within(reopened).getByRole("button", { name: "取り消す" });
    expect(submit).toBeDisabled();
    await user.click(submit);
    expect(mockCancel).toHaveBeenCalledTimes(1);

    // 新しい読取りで active を確かめると、押せる
    await user.click(within(reopened).getByRole("button", { name: "状態をもう一度確かめる" }));
    await waitFor(() => {
      expect(mockToast.error).toHaveBeenCalledWith(FAILED_TOAST);
    });
    expect(receiving.read).toHaveBeenCalledTimes(3);
    expect(within(reopened).queryByText(UNCONFIRMED_TEXT)).toBeNull();
    expect(submit).toBeEnabled();
  });

  it("取り直しが canceled: C25 の再取得を待たずに、在庫を確かめる商品を「確認中」にして待ちの止めを解く", async () => {
    const user = userEvent.setup();
    const refresh = deferred<Reply<ReceivingRecordDetail>>();
    receiving.read
      .mockResolvedValueOnce(ok(receivingDetail()))
      .mockResolvedValueOnce(ok(receivingDetail(true)))
      // C25 の再取得は終わらせない
      .mockReturnValue(refresh.promise);
    mockCancel.mockRejectedValue(new Error("合成の応答の消失"));
    const stock = deferred<Awaited<ReturnType<typeof commands.getStockDetail>>>();
    mockGetStockDetail.mockReturnValue(stock.promise);
    renderPage(receiving.page);

    const dialog = await openDialog(user);
    await user.click(within(dialog).getByRole("button", { name: "取り消す" }));

    expect(await screen.findByText("この記録は取消済みです")).toBeInTheDocument();
    expect(await screen.findByText(READING_STOCK_TEXT)).toBeInTheDocument();
    await waitFor(() => {
      expect(getWindowCloseGuardKinds()).toEqual([]);
    });

    // 読めて 0 なら警告を出さない（「確認中」と見分けられる）
    stock.resolve(
      ok(
        makeMockStockDetail({
          product: makeMockProductWithRelations({ product_code: "RC-001", stock_quantity: 0 }),
        }),
      ),
    );
    await waitFor(() => {
      expect(screen.queryByText(READING_STOCK_TEXT)).toBeNull();
    });
    expect(screen.queryByText(STOCK_ALERT_TITLE)).toBeNull();
    refresh.resolve(ok(receivingDetail(true)));
  });

  it("成功: Toast の後は C25 の再取得を待たずに待ちの止めを解き、取消済みを出して取消の button を消す", async () => {
    const user = userEvent.setup();
    const refresh = deferred<Reply<ReceivingRecordDetail>>();
    receiving.read.mockResolvedValueOnce(ok(receivingDetail())).mockReturnValue(refresh.promise);
    // 負の在庫が無い（stock_warnings が 0 件）
    mockCancel.mockResolvedValue(ok({ stock_warnings: [] }));
    renderPage(receiving.page);

    const dialog = await openDialog(user);
    await user.click(within(dialog).getByRole("button", { name: "取り消す" }));

    expect(await screen.findByText("この記録は取消済みです")).toBeInTheDocument();
    expect(mockToast.success).toHaveBeenCalledWith("記録を取り消しました", { duration: 8000 });
    await waitFor(() => {
      expect(getWindowCloseGuardKinds()).toEqual([]);
    });
    expect(screen.queryByRole("alertdialog")).toBeNull();
    expect(screen.queryByRole("button", { name: "この記録を取り消す" })).toBeNull();
    expect(screen.queryByText(STOCK_ALERT_TITLE)).toBeNull();
    expect(mockCancel).toHaveBeenCalledTimes(1);

    // 取消の日時と理由は、再取得した詳細が出す
    refresh.resolve(ok(receivingDetail(true)));
    expect(await screen.findByText("2026-11-21 10:00:00")).toBeInTheDocument();
    expect(screen.getByText(REASON)).toBeInTheDocument();
  });

  it("未確認の記録を開き直すと、dialog を開く前から詳細に「結果未確認」と「状態をもう一度確かめる」を出し、「有効」と言い切らない", async () => {
    const user = userEvent.setup();
    receiving.read
      .mockResolvedValueOnce(ok(receivingDetail()))
      .mockResolvedValueOnce(cmdError("internal", "合成の読取りの失敗"))
      .mockResolvedValue(ok(receivingDetail()));
    mockCancel.mockRejectedValue(new Error("合成の応答の消失"));
    const queryClient = makeQueryClient();
    const first = renderPage(receiving.page, queryClient);

    const dialog = await openDialog(user);
    await user.click(within(dialog).getByRole("button", { name: "取り消す" }));
    expect(await within(dialog).findByText(UNCONFIRMED_TEXT)).toBeInTheDocument();
    first.unmount();

    // 開き直した詳細は cache の active（読み直していない）: dialog を開く前から未確認を知らせる
    renderPage(receiving.page, queryClient);
    expect(await screen.findByText("結果未確認")).toBeInTheDocument();
    expect(screen.queryByText("有効")).toBeNull();
    expect(screen.queryByRole("alertdialog")).toBeNull();
    expect(screen.getByText(UNCONFIRMED_TEXT)).toBeInTheDocument();
    expect(receiving.read).toHaveBeenCalledTimes(2);

    // 詳細の画面の「状態をもう一度確かめる」は読取りだけ（取消は送らない）。active を読めたら「有効」に戻る
    await user.click(screen.getByRole("button", { name: "状態をもう一度確かめる" }));
    await waitFor(() => {
      expect(mockToast.error).toHaveBeenCalledWith(FAILED_TOAST);
    });
    expect(receiving.read).toHaveBeenCalledTimes(3);
    expect(mockCancel).toHaveBeenCalledTimes(1);
    expect(screen.getByText("有効")).toBeInTheDocument();
    expect(screen.queryByText("結果未確認")).toBeNull();
    expect(screen.queryByText(UNCONFIRMED_TEXT)).toBeNull();
  });

  it.each([
    {
      label: "詳細の cache が回収された後に開き直し、詳細の query が読み直す",
      arrange: (queryClient: QueryClient) => {
        queryClient.removeQueries({ queryKey: RECEIVING_DETAIL_KEY, exact: true });
      },
    },
    {
      label: "開き直す前に、詳細の cache が取消済みを持っている（mount の時点で canceled）",
      arrange: (queryClient: QueryClient) => {
        queryClient.setQueryData(RECEIVING_DETAIL_KEY, receivingDetail(true));
      },
    },
  ])(
    "未確認の記録を開き直した詳細が canceled（$label）: 印の商品の今の在庫を確かめて C25 を 1 回だけ当て、印を消す",
    async ({ arrange }) => {
      const user = userEvent.setup();
      receiving.read
        .mockResolvedValueOnce(ok(receivingDetail()))
        .mockResolvedValueOnce(cmdError("internal", "合成の読取りの失敗"))
        // 応答を失った取消は commit されていた
        .mockResolvedValue(ok(receivingDetail(true)));
      mockCancel.mockRejectedValue(new Error("合成の応答の消失"));
      const stock = deferred<Awaited<ReturnType<typeof commands.getStockDetail>>>();
      mockGetStockDetail.mockReturnValue(stock.promise);
      const queryClient = makeQueryClient();
      const first = renderPage(receiving.page, queryClient);

      const dialog = await openDialog(user);
      await user.click(within(dialog).getByRole("button", { name: "取り消す" }));
      expect(await within(dialog).findByText(UNCONFIRMED_TEXT)).toBeInTheDocument();
      first.unmount();

      arrange(queryClient);
      const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
      // main.tsx と同じく root を StrictMode で包む（mount の effect が 2 回走る）。それでも 1 回だけ
      configure({ reactStrictMode: true });
      try {
        renderPage(receiving.page, queryClient);
      } finally {
        configure({ reactStrictMode: false });
      }

      expect(await screen.findByText("この記録は取消済みです")).toBeInTheDocument();
      expect(await screen.findByText(READING_STOCK_TEXT)).toBeInTheDocument();
      stock.resolve(
        ok(
          makeMockStockDetail({
            product: makeMockProductWithRelations({ product_code: "RC-001", stock_quantity: -2 }),
          }),
        ),
      );
      expect(
        await screen.findByText("RC-001: 在庫がマイナスになりました（-2）"),
      ).toBeInTheDocument();
      expect(new Set(mockGetStockDetail.mock.calls.map(([code]) => code))).toEqual(
        new Set(["RC-001"]),
      );
      expectExactInvalidations(
        invalidateSpy.mock.calls,
        d052InvalidationOracle.inventoryRecordCancel(),
      );
      expect(
        queryClient.getQueryData(queryKeys.recordCancelUnconfirmed("receiving_record", RECORD_ID)),
      ).toBeUndefined();
      expect(screen.queryByText("結果未確認")).toBeNull();
      expect(screen.queryByRole("button", { name: "この記録を取り消す" })).toBeNull();
      expect(mockCancel).toHaveBeenCalledTimes(1);
    },
  );

  it("未確認の記録を開き直した読取りが失敗: error の表示に結果未確認の知らせを出し、関連 movement の無い記録は canceled を確かめても在庫を読まない", async () => {
    const user = userEvent.setup();
    const returns = PAGES[1];
    // レジで読んで戻した返品（この保存で在庫を動かしていない。D-148）: 関連 movement が 0 件
    const active: ReturnRecordDetail = {
      ...returnDetail(),
      register_processed: true,
      movements: [],
    };
    returns.read
      .mockResolvedValueOnce(ok(active))
      .mockResolvedValueOnce(cmdError("internal", "合成の読取りの失敗"))
      .mockResolvedValueOnce(cmdError("internal", "合成の読取りの失敗"))
      .mockResolvedValue(ok({ ...active, ...state(true, []) } satisfies ReturnRecordDetail));
    mockCancel.mockRejectedValue(new Error("合成の応答の消失"));
    const queryClient = makeQueryClient();
    const first = renderPage(returns.page, queryClient);

    const dialog = await openDialog(user);
    await user.click(within(dialog).getByRole("button", { name: "取り消す" }));
    expect(await within(dialog).findByText(UNCONFIRMED_TEXT)).toBeInTheDocument();
    first.unmount();

    // 詳細の cache が回収された後に開き直し、その読取りも失敗する
    queryClient.removeQueries({
      queryKey: queryKeys.inventoryRecords.returnDetail(RECORD_ID),
      exact: true,
    });
    renderPage(returns.page, queryClient);
    expect(await screen.findByText("取消の結果未確認")).toBeInTheDocument();
    expect(screen.getByText(UNCONFIRMED_TEXT)).toBeInTheDocument();
    expect(screen.queryByText("この記録は取消済みです")).toBeNull();

    await user.click(screen.getByRole("button", { name: "状態をもう一度確かめる" }));
    expect(await screen.findByText("この記録は取消済みです")).toBeInTheDocument();
    expect(screen.getByText("取消済み")).toBeInTheDocument();
    expect(screen.queryByText("取消の結果未確認")).toBeNull();
    // 商品が 0 件: 在庫は読まず、在庫の知らせ（「確認中」を含む）も出さない
    expect(mockGetStockDetail).not.toHaveBeenCalled();
    expect(screen.queryByText(STOCK_ALERT_TITLE)).toBeNull();
    expect(mockCancel).toHaveBeenCalledTimes(1);
  });

  it("未確認の印の項目は、未確認になった記録にだけ作って回収させず、確かめたら消す", async () => {
    const user = userEvent.setup();
    // 既定の gcTime が短い client でも、未確認の印だけは画面を離れた後に残る
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0 } },
    });
    const markers = () =>
      queryClient.getQueryCache().findAll({ queryKey: ["record-cancel-unconfirmed"] });
    let readFails = false;
    vi.mocked(commands.getReceivingRecord).mockImplementation((id: number) =>
      Promise.resolve(
        readFails ? cmdError("internal", "合成の読取りの失敗") : ok({ ...receivingDetail(), id }),
      ),
    );
    mockCancel.mockRejectedValue(new Error("合成の応答の消失"));

    // 取消を試していない記録を開いて閉じても、項目を作らない
    for (const recordId of [RECORD_ID, RECORD_ID + 1]) {
      const view = renderPage(<ReceivingRecordDetailPage recordId={recordId} />, queryClient);
      await screen.findByRole("button", { name: "この記録を取り消す" });
      view.unmount();
    }
    expect(markers()).toHaveLength(0);

    // 未確認になった記録の印は、画面を離れても回収されない
    const first = renderPage(receiving.page, queryClient);
    const dialog = await openDialog(user);
    readFails = true;
    await user.click(within(dialog).getByRole("button", { name: "取り消す" }));
    expect(await within(dialog).findByText(UNCONFIRMED_TEXT)).toBeInTheDocument();
    first.unmount();
    await new Promise((resolve) => setTimeout(resolve, 20));
    expect(markers().map((query) => query.gcTime)).toEqual([Number.POSITIVE_INFINITY]);

    // 開き直して active を確かめたら、項目ごと消す
    readFails = false;
    renderPage(receiving.page, queryClient);
    expect(await screen.findByText("結果未確認")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "状態をもう一度確かめる" }));
    await waitFor(() => {
      expect(mockToast.error).toHaveBeenCalledWith(FAILED_TOAST);
    });
    expect(markers()).toHaveLength(0);
  });
});

describe("別の記録の詳細へ移ったとき（REQ-208 / 65 §65.5b）", () => {
  it("同じ route の param が変わると、前の記録の取消の知らせを出さない", async () => {
    const user = userEvent.setup();
    let canceled = false;
    vi.mocked(commands.getReceivingRecord).mockImplementation((id: number) =>
      // 記録 5 は取消の後に取消済み、記録 6 は初めから取消済み
      Promise.resolve(ok({ ...receivingDetail(id !== RECORD_ID || canceled), id })),
    );
    mockCancel.mockImplementation(() => {
      canceled = true;
      return Promise.resolve(ok({ stock_warnings: ["RC-001: 在庫がマイナスになりました（-2）"] }));
    });
    function Switcher() {
      const [recordId, setRecordId] = useState(RECORD_ID);
      return (
        <>
          <button
            type="button"
            onClick={() => {
              setRecordId(RECORD_ID + 1);
            }}
          >
            次の記録
          </button>
          <ReceivingRecordDetailPage recordId={recordId} />
        </>
      );
    }
    renderPage(<Switcher />);

    const dialog = await openDialog(user);
    await user.click(within(dialog).getByRole("button", { name: "取り消す" }));
    expect(await screen.findByText("RC-001: 在庫がマイナスになりました（-2）")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "次の記録" }));
    expect(
      await screen.findByRole("heading", { name: `入庫記録 #${String(RECORD_ID + 1)}` }),
    ).toBeInTheDocument();
    expect(screen.getByText("この記録は取消済みです")).toBeInTheDocument();
    expect(screen.queryByText("RC-001: 在庫がマイナスになりました（-2）")).toBeNull();
  });
});

function renderApp(initialEntry: string) {
  const router = createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: [initialEntry] }),
  });
  render(
    <QueryClientProvider client={makeQueryClient()}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return router;
}

describe("取消の応答を待つ間の route の止め（REQ-208 / 65 §65.5b / UI-USW-D1 / T10）", () => {
  it("送信中は詳細の画面から移れず、応答の後は移れる", async () => {
    // memory history の止めは push と replace だけに効くので、link の押下で試す
    const user = userEvent.setup();
    receiving.read.mockResolvedValue(ok(receivingDetail()));
    const reply = deferred<Awaited<ReturnType<typeof commands.cancelInventoryRecord>>>();
    mockCancel.mockReturnValueOnce(reply.promise);
    const path = `/inventory/receiving/records/${String(RECORD_ID)}`;
    const router = renderApp(path);

    const dialog = await openDialog(user);
    await user.click(within(dialog).getByRole("button", { name: "取り消す" }));
    await within(dialog).findByRole("button", { name: /取り消し中…/ });
    void router.navigate({ to: "/inventory/records" });
    await waitFor(() => {
      expect(getWindowCloseGuardKinds()).toEqual(["inventory-record-cancel"]);
    });
    expect(router.state.location.pathname).toBe(path);

    reply.resolve(cmdError("internal", "合成の取消の失敗"));
    await waitFor(() => {
      expect(mockToast.error).toHaveBeenCalledWith(FAILED_TOAST);
    });
    await waitFor(() => {
      expect(getWindowCloseGuardKinds()).toEqual([]);
    });
    // 送信中の移動は止まったまま（応答の後に勝手に移らない）
    expect(router.state.location.pathname).toBe(path);
    await router.navigate({ to: "/inventory/records" });
    expect(router.state.location.pathname).toBe("/inventory/records");
  });
});

describe("詳細で取り消して作業画面へ戻る（REQ-208 / D-052 C25 / T12）", () => {
  // 作業画面の直近の一覧は receivings・returns・disposals の key で、入出庫履歴の key の下に無い。
  // 直近の一覧を読み込んだ（cache がある）後に詳細で取り消して戻ると、その行が「取消済み」で出る
  const listPage = <T,>(items: T[]) =>
    ok({ items, total_count: items.length, page: 1, per_page: 10 });

  it.each([
    {
      label: "入庫",
      path: "/inventory/receiving",
      detailPage: receiving,
      arrange: (canceled: boolean) => {
        vi.mocked(commands.listReceivings).mockResolvedValue(
          listPage([
            {
              id: RECORD_ID,
              supplier_id: null,
              supplier_name: null,
              receiving_date: BUSINESS_DATE,
              note: null,
              created_at: "2026-11-18T10:00:00",
              canceled_at: canceled ? CANCELED_AT : null,
            },
          ]),
        );
      },
    },
    {
      label: "返品・交換",
      path: "/inventory/return",
      detailPage: PAGES[1],
      arrange: (canceled: boolean) => {
        vi.mocked(commands.listReturns).mockResolvedValue(
          listPage([
            {
              id: RECORD_ID,
              return_type: "return" as const,
              return_date: BUSINESS_DATE,
              register_processed: false,
              note: null,
              created_at: "2026-11-18T10:00:00",
              canceled_at: canceled ? CANCELED_AT : null,
            },
          ]),
        );
      },
    },
    {
      label: "廃棄・破損",
      path: "/inventory/disposal",
      detailPage: PAGES[3],
      arrange: (canceled: boolean) => {
        vi.mocked(commands.listDisposals).mockResolvedValue(
          listPage([
            {
              id: RECORD_ID,
              disposal_date: BUSINESS_DATE,
              created_at: "2026-11-18T10:00:00",
              canceled_at: canceled ? CANCELED_AT : null,
            },
          ]),
        );
      },
    },
  ])(
    "$label の作業画面: 取り消した行が有効のまま残らない",
    async ({ path, detailPage, arrange }) => {
      const user = userEvent.setup();
      arrange(false);
      detailPage.read
        .mockResolvedValueOnce(ok(detailPage.detail()))
        .mockResolvedValue(ok(detailPage.detail(true)));
      mockCancel.mockImplementation(() => {
        // 取消の後の読取りは取消済みの行を返す
        arrange(true);
        return Promise.resolve(ok({ stock_warnings: [] }));
      });
      const router = renderApp(path);

      // 直近の一覧を読み込む（有効の行。「取消済み」は出ていない）
      await user.click(await screen.findByRole("link", { name: "詳細を見る" }));
      expect(screen.queryByText("取消済み")).toBeNull();
      const dialog = await openDialog(user);
      await user.click(within(dialog).getByRole("button", { name: "取り消す" }));
      expect(await screen.findByText("この記録は取消済みです")).toBeInTheDocument();

      await user.click(screen.getByRole("link", { name: "前の画面へ戻る" }));
      await waitFor(() => {
        expect(router.state.location.pathname).toBe(path);
      });
      const row = (await screen.findByRole("link", { name: "詳細を見る" })).closest("tr");
      expect(row).not.toBeNull();
      expect(await within(row as HTMLElement).findByText("取消済み")).toBeInTheDocument();
    },
  );
});
