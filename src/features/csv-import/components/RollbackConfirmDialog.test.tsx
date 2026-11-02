import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, expect, it, vi } from "vitest";

import { commands } from "@/lib/bindings";
import type { CmdError, CountRecoveryTarget, RollbackPreview } from "@/lib/bindings";
import { RollbackConfirmDialog } from "./RollbackConfirmDialog";

vi.mock("@/lib/bindings", () => ({
  commands: { previewRollbackCsvImport: vi.fn() },
}));

const mockPreview = vi.mocked(commands.previewRollbackCsvImport);

const RECHECK_HEADING =
  "取り消した後、次の商品は数え直しが必要です（今すでに必要な商品を含みます）";
const STOCKTAKE_SENTENCE =
  "完了した棚卸し（2026-12-31）の評価額は変わりません。棚卸しの記録の詳細の「差異」「ロス原価」と「完了の後に入った記録による差」が変わることがあります。";

function target(overrides: Partial<CountRecoveryTarget> = {}): CountRecoveryTarget {
  return {
    product_code: "RB-A",
    product_name: "合成の糸A",
    stock_unit: "pcs",
    stock: 9,
    stock_range_low: 9,
    stock_range_high: 10,
    stocktake_item_id: null,
    stocktake_id: null,
    action: "count_now",
    block_reason: null,
    recount_reasons: ["undetermined_settlement"],
    ...overrides,
  };
}

function preview(overrides: Partial<RollbackPreview> = {}): RollbackPreview {
  return {
    csv_import_id: 31,
    settlement_no: 0,
    recheck_targets: [],
    affected_stocktakes: [],
    ...overrides,
  };
}

function cmdError(kind: CmdError["kind"], message: string): CmdError {
  return { kind, message, field: null, error_id: null, stocktake_recovery: null };
}

function renderDialog() {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const view = render(
    <QueryClientProvider client={queryClient}>
      <RollbackConfirmDialog
        csvImportId={31}
        settlementDate="2026-12-30"
        filename="z004-1230.csv"
        totalItems={2}
        totalAmount={2000}
        triggerLabel="この取込みを取り消す"
        onConfirm={vi.fn()}
        isRollingBack={false}
      />
    </QueryClientProvider>,
  );
  return { ...view, queryClient };
}

async function openDialog(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByRole("button", { name: "この取込みを取り消す" }));
  return screen.getByRole("alertdialog");
}

beforeEach(() => {
  mockPreview.mockReset();
});

it("REQ-401: 取消の前の読取りの間は「取り消す」を押せず、確かめている文を出す", async () => {
  const user = userEvent.setup();
  mockPreview.mockReturnValue(new Promise(() => undefined));
  renderDialog();

  const dialog = await openDialog(user);

  expect(mockPreview).toHaveBeenCalledWith(31);
  expect(within(dialog).getByText("取り消した後のようすを確かめています…")).toBeInTheDocument();
  expect(within(dialog).getByRole("button", { name: "取り消す" })).toBeDisabled();
});

it("REQ-401: 対象の項目・精算回数・要再確認の一覧・取り込み直し・完了した棚卸しを出す", async () => {
  const user = userEvent.setup();
  mockPreview.mockResolvedValue({
    status: "ok",
    data: preview({
      recheck_targets: [
        target(),
        target({
          product_code: "RB-C",
          product_name: "合成の糸C",
          stock: 5,
          stock_range_low: 5,
          stock_range_high: 6,
          recount_reasons: ["conflicting_counts"],
        }),
      ],
      affected_stocktakes: [{ stocktake_id: 8, completed_at: "2026-12-31T18:00:00" }],
    }),
  });
  renderDialog();

  const dialog = await openDialog(user);

  expect(await within(dialog).findByText("精算 0")).toBeInTheDocument();
  expect(dialog).toHaveTextContent("31");
  expect(dialog).toHaveTextContent("2026-12-30");
  expect(dialog).toHaveTextContent("z004-1230.csv");
  expect(dialog).toHaveTextContent("2 件");
  expect(dialog).toHaveTextContent("¥2,000");
  expect(within(dialog).getAllByText(RECHECK_HEADING)).toHaveLength(1);
  expect(within(dialog).queryByText(/取り消すと、次の商品は数え直しが必要になります/)).toBeNull();
  const rowA = within(dialog).getByRole("row", { name: /合成の糸A/ });
  expect(within(rowA).getByText("9〜10 個")).toBeInTheDocument();
  expect(within(rowA).getByText("数えた日に売り買い（入出庫）がありました")).toBeInTheDocument();
  const badge = within(rowA).getByText("要再確認").closest('[data-slot="badge"]');
  expect(badge?.querySelector('svg[aria-hidden="true"]')).toBeInTheDocument();
  const rowC = within(dialog).getByRole("row", { name: /合成の糸C/ });
  expect(within(rowC).getByText("5〜6 個")).toBeInTheDocument();
  expect(within(rowC).getByText("同じ日に違う数が保存されています")).toBeInTheDocument();
  // 確認の中では入れ直しへ進めない（RecheckTargetList を renderAction なしで使う）
  expect(within(dialog).queryByRole("columnheader", { name: "操作" })).toBeNull();
  expect(within(dialog).queryByText("棚の数を確かめて入れ直してください。")).toBeNull();
  expect(within(dialog).queryByRole("link", { name: "棚卸しの画面で入れ直す" })).toBeNull();
  expect(dialog).toHaveTextContent(
    "正しい版のファイルがあれば、取り消した後に「商品別CSV取込み（Z004）」で取り込み直してください（精算 0）。取り込み直した結果に、数え直しが必要な商品が出ることがあります。",
  );
  expect(within(dialog).getAllByText(STOCKTAKE_SENTENCE)).toHaveLength(1);
  expect(dialog).not.toHaveTextContent("「完了の後に入った記録による差（参考）」");
  expect(dialog).not.toHaveTextContent("が変わります。");
  expect(within(dialog).getByRole("button", { name: "取り消す" })).toBeEnabled();
});

it("REQ-401: 要再確認も完了した棚卸しも無ければ、無いことを言い切り、棚卸しの文を出さない", async () => {
  const user = userEvent.setup();
  mockPreview.mockResolvedValue({ status: "ok", data: preview({ settlement_no: 101 }) });
  renderDialog();

  const dialog = await openDialog(user);

  expect(
    await within(dialog).findByText("取り消しても、数え直しが必要になる商品はありません"),
  ).toBeInTheDocument();
  expect(within(dialog).queryByText(RECHECK_HEADING)).toBeNull();
  expect(dialog).not.toHaveTextContent("完了した棚卸し");
  expect(within(dialog).getByRole("button", { name: "取り消す" })).toBeEnabled();
});

it("REQ-401: 読取りの失敗と取消済みでは「取り消す」を押せない", async () => {
  const user = userEvent.setup();
  mockPreview.mockResolvedValue({ status: "error", error: cmdError("internal", "合成の失敗") });
  const { unmount } = renderDialog();

  let dialog = await openDialog(user);
  expect(
    await within(dialog).findByText(
      "取り消した後のようすを読み込めませんでした。閉じてから、もう一度開いてください",
    ),
  ).toBeInTheDocument();
  expect(within(dialog).getByRole("button", { name: "取り消す" })).toBeDisabled();
  unmount();

  mockPreview.mockResolvedValue({
    status: "error",
    error: cmdError("validation", "この取込みは取り消し済みです"),
  });
  renderDialog();
  dialog = await openDialog(user);
  expect(await within(dialog).findByText("この取込みは取り消し済みです")).toBeInTheDocument();
  expect(within(dialog).getByRole("button", { name: "取り消す" })).toBeDisabled();
});

it("REQ-401: 確認を開くたびに取消の前の読取りをやり直す", async () => {
  const user = userEvent.setup();
  mockPreview.mockResolvedValueOnce({
    status: "ok",
    data: preview({ recheck_targets: [target({ product_name: "一回目の商品" })] }),
  });
  let resolveSecond: (value: { status: "ok"; data: RollbackPreview }) => void = () => undefined;
  mockPreview.mockReturnValueOnce(
    new Promise((resolve) => {
      resolveSecond = resolve;
    }),
  );
  renderDialog();

  let dialog = await openDialog(user);
  expect(await within(dialog).findByText("一回目の商品")).toBeInTheDocument();
  await user.click(within(dialog).getByRole("button", { name: "キャンセル" }));

  dialog = await openDialog(user);
  expect(within(dialog).getByRole("button", { name: "取り消す" })).toBeDisabled();
  expect(within(dialog).queryByText("一回目の商品")).toBeNull();

  resolveSecond({
    status: "ok",
    data: preview({ recheck_targets: [target({ product_name: "二回目の商品" })] }),
  });
  expect(await within(dialog).findByText("二回目の商品")).toBeInTheDocument();
  expect(mockPreview).toHaveBeenCalledTimes(2);
  expect(within(dialog).getByRole("button", { name: "取り消す" })).toBeEnabled();
});

it("REQ-401: 読めた後の読み直しが失敗したら、前の読取りを使わず「取り消す」を押せない", async () => {
  const user = userEvent.setup();
  mockPreview.mockResolvedValueOnce({
    status: "ok",
    data: preview({ recheck_targets: [target({ product_name: "一回目の商品" })] }),
  });
  mockPreview.mockResolvedValueOnce({ status: "error", error: cmdError("internal", "合成の失敗") });
  const { queryClient } = renderDialog();

  const dialog = await openDialog(user);
  expect(await within(dialog).findByText("一回目の商品")).toBeInTheDocument();
  expect(within(dialog).getByRole("button", { name: "取り消す" })).toBeEnabled();

  // 開いたままの読み直し（接続の復帰や再取得）が失敗する
  await act(() => queryClient.refetchQueries());

  expect(
    await within(dialog).findByText(
      "取り消した後のようすを読み込めませんでした。閉じてから、もう一度開いてください",
    ),
  ).toBeInTheDocument();
  expect(mockPreview).toHaveBeenCalledTimes(2);
  expect(within(dialog).queryByText("一回目の商品")).toBeNull();
  expect(within(dialog).getByRole("button", { name: "取り消す" })).toBeDisabled();
});
