import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { commands } from "@/lib/bindings";
import type { CmdError, Supplier, SupplierWithUsage } from "@/lib/bindings";
import { toast } from "sonner";
import { SupplierManagementPage } from "./SupplierManagementPage";

vi.mock("sonner", () => ({ toast: { success: vi.fn(), error: vi.fn() } }));
vi.mock("@/lib/bindings", () => ({
  commands: {
    listSuppliersWithUsage: vi.fn(),
    createSupplier: vi.fn(),
    renameSupplier: vi.fn(),
    mergeSuppliers: vi.fn(),
  },
}));

const mockList = vi.mocked(commands.listSuppliersWithUsage);
const mockCreate = vi.mocked(commands.createSupplier);
const mockRename = vi.mocked(commands.renameSupplier);
const mockMerge = vi.mocked(commands.mergeSuppliers);

const SUPPLIERS: SupplierWithUsage[] = [
  { id: 1, name: "あ取引先", product_count: 2, receiving_record_count: 1 },
  { id: 2, name: "か取引先", product_count: 1, receiving_record_count: 0 },
  { id: 3, name: "さ取引先", product_count: 0, receiving_record_count: 0 },
];
const cmdError = (kind: CmdError["kind"], message: string): CmdError => ({
  kind,
  message,
  field: null,
  error_id: null,
  stocktake_recovery: null,
});

function arrangeList(rows: SupplierWithUsage[] = SUPPLIERS) {
  mockList.mockResolvedValue({ status: "ok", data: rows });
  mockCreate.mockResolvedValue({
    status: "ok",
    data: { id: 4, name: "新取引先", created_at: "2026-08-25T10:00:00" },
  });
  mockRename.mockResolvedValue({
    status: "ok",
    data: { id: 1, name: "新名称", created_at: "2026-08-25T10:00:00" },
  });
  mockMerge.mockResolvedValue({
    status: "ok",
    data: { products_updated: 2, receiving_records_updated: 1 },
  });
}

function renderPage() {
  const client = new QueryClient({
    defaultOptions: {
      queries: { retry: false, gcTime: Number.POSITIVE_INFINITY },
      mutations: { retry: false },
    },
  });
  const view = render(
    <QueryClientProvider client={client}>
      <SupplierManagementPage />
    </QueryClientProvider>,
  );
  return { ...view, client };
}

async function openMerge(user: ReturnType<typeof userEvent.setup>, rowId = 1) {
  const row = await screen.findByTestId(`supplier-row-${String(rowId)}`);
  await user.click(within(row).getByRole("button", { name: "統合" }));
}

async function selectMergeTarget(
  user: ReturnType<typeof userEvent.setup>,
  targetName = "か取引先",
) {
  await user.click(screen.getByRole("combobox", { name: "残す取引先" }));
  await user.click(await screen.findByRole("option", { name: targetName }));
}

beforeEach(() => {
  vi.clearAllMocks();
  arrangeList();
});

describe("SupplierManagementPage UI-15 / REQ-107", () => {
  it("⑰ SC4 / UIDISP-D4: 改名の保存・再試行はキャンセルの後に並ぶ", async () => {
    const user = userEvent.setup();
    renderPage();
    const row = await screen.findByTestId("supplier-row-1");
    await user.click(within(row).getByRole("button", { name: "名前を変更" }));
    expect(
      within(row)
        .getAllByRole("button")
        .slice(0, 2)
        .map((button) => button.textContent),
    ).toEqual(["キャンセル", "保存"]);
    await user.clear(within(row).getByLabelText("あ取引先の新しい取引先名"));
    await user.click(within(row).getByRole("button", { name: "保存" }));
    expect(
      within(row)
        .getAllByRole("button")
        .slice(0, 2)
        .map((button) => button.textContent),
    ).toEqual(["キャンセル", "再試行"]);
  });

  it("REQ-107 一覧は name 昇順で取引先名・関連商品数・入庫記録数を N件 表示し 0 件も 0件 と明示する", async () => {
    renderPage();
    const rows = await screen.findAllByTestId(/supplier-row-/);
    expect(rows.map((row) => within(row).getAllByRole("cell")[0]?.textContent)).toEqual([
      "あ取引先",
      "か取引先",
      "さ取引先",
    ]);
    expect(within(rows[0]).getByText("2件")).toBeInTheDocument();
    expect(within(rows[0]).getByText("1件")).toBeInTheDocument();
    expect(within(rows[2]).getAllByText("0件")).toHaveLength(2);
    expect(
      within(rows[0])
        .getAllByRole("button")
        .map((button) => button.textContent),
    ).toEqual(["名前を変更", "統合"]);
    expect(screen.queryByRole("button", { name: /削除/ })).not.toBeInTheDocument();
  });

  it("新しい取引先を追加は trim して createSupplier を呼び成功後に usage 一覧だけを再取得する", async () => {
    mockList
      .mockReset()
      .mockResolvedValueOnce({ status: "ok", data: SUPPLIERS })
      .mockResolvedValue({
        status: "ok",
        data: [
          ...SUPPLIERS,
          { id: 4, name: "新取引先", product_count: 0, receiving_record_count: 0 },
        ],
      });
    const user = userEvent.setup();
    const { client } = renderPage();
    const invalidate = vi.spyOn(client, "invalidateQueries");
    await screen.findByText("あ取引先");
    // SC12/SC13/SC14 / DSR-01: 補助操作と画面の主操作を区別する。
    expect(screen.getByRole("button", { name: "新しい取引先を追加" })).toHaveAttribute(
      "data-variant",
      "default",
    );
    await user.click(screen.getByRole("button", { name: "新しい取引先を追加" }));
    await user.type(screen.getByLabelText("取引先名"), "  新取引先  ");
    await user.click(screen.getByRole("button", { name: "追加する" }));
    await waitFor(() => {
      expect(mockCreate).toHaveBeenCalledWith("新取引先");
    });
    await waitFor(() => {
      expect(mockList.mock.calls.length).toBeGreaterThanOrEqual(2);
    });
    expect(await screen.findByText("新取引先")).toBeInTheDocument();
    expect(invalidate).not.toHaveBeenCalled();
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(toast.success).toHaveBeenCalledWith("取引先「新取引先」を追加しました");
  });

  it("取引先名が空白のみなら createSupplier を呼ばず field error を出す", async () => {
    const user = userEvent.setup();
    renderPage();
    await screen.findByText("あ取引先");
    await user.click(screen.getByRole("button", { name: "新しい取引先を追加" }));
    await user.type(screen.getByLabelText("取引先名"), "   ");
    await user.click(screen.getByRole("button", { name: "追加する" }));
    expect(await screen.findByText("取引先名を入力してください")).toBeInTheDocument();
    expect(mockCreate).not.toHaveBeenCalled();
    expect(screen.getByLabelText("取引先名")).toHaveValue("   ");
  });

  it("追加失敗時は dialog と入力を保持して再試行を出す", async () => {
    mockCreate.mockRejectedValue(new Error("synthetic failure"));
    const user = userEvent.setup();
    renderPage();
    await screen.findByText("あ取引先");
    await user.click(screen.getByRole("button", { name: "新しい取引先を追加" }));
    await user.type(screen.getByLabelText("取引先名"), "保持する取引先名");
    await user.click(screen.getByRole("button", { name: "追加する" }));
    expect(await screen.findByRole("button", { name: "再試行" })).toBeInTheDocument();
    expect(screen.getByRole("dialog")).toBeInTheDocument();
    expect(screen.getByLabelText("取引先名")).toHaveValue("保持する取引先名");
    expect(toast.success).not.toHaveBeenCalled();
  });

  it("名前を変更で行が入力欄になり保存で renameSupplier を呼び Escape で現在名に戻す", async () => {
    mockList
      .mockReset()
      .mockResolvedValueOnce({ status: "ok", data: SUPPLIERS })
      .mockResolvedValue({
        status: "ok",
        data: [{ ...SUPPLIERS[0], name: "新名称" }, SUPPLIERS[1], SUPPLIERS[2]],
      });
    const user = userEvent.setup();
    renderPage();
    const row = await screen.findByTestId("supplier-row-1");
    await user.click(within(row).getByRole("button", { name: "名前を変更" }));
    const input = within(row).getByLabelText("あ取引先の新しい取引先名");
    expect(input).toHaveValue("あ取引先");
    await user.clear(input);
    await user.type(input, "新名称");
    await user.click(within(row).getByRole("button", { name: "保存" }));
    await waitFor(() => {
      expect(mockRename).toHaveBeenCalledWith(1, "新名称");
    });
    expect(await within(row).findByText("新名称")).toBeInTheDocument();
    // T6 (UI 表示磨き batch Scope 6): 統合（78.7）の完了通知と対称の toast を出す（78.6 追記）。
    expect(toast.success).toHaveBeenCalledWith("取引先名を変更しました");

    await user.click(within(row).getByRole("button", { name: "名前を変更" }));
    const reopened = within(row).getByLabelText("新名称の新しい取引先名");
    await user.clear(reopened);
    await user.type(reopened, "取消名称{Escape}");
    expect(mockRename).toHaveBeenCalledTimes(1);
    expect(within(row).getByText("新名称")).toBeInTheDocument();

    await user.click(within(row).getByRole("button", { name: "名前を変更" }));
    const enterInput = within(row).getByLabelText("新名称の新しい取引先名");
    await user.clear(enterInput);
    await user.type(enterInput, "Enter名称{Enter}");
    await waitFor(() => {
      expect(mockRename).toHaveBeenLastCalledWith(1, "Enter名称");
    });
  });

  it("他行と同名の改名は統合案内の validation 文言を表示し入力を保持する", async () => {
    mockRename.mockResolvedValue({
      status: "error",
      error: cmdError(
        "validation",
        "同じ名前の取引先があります。重複している場合は「統合」を使ってください。",
      ),
    });
    const user = userEvent.setup();
    renderPage();
    const row = await screen.findByTestId("supplier-row-1");
    await user.click(within(row).getByRole("button", { name: "名前を変更" }));
    const input = within(row).getByLabelText("あ取引先の新しい取引先名");
    await user.clear(input);
    await user.type(input, "か取引先");
    await user.click(within(row).getByRole("button", { name: "保存" }));
    expect(
      await within(row).findByText(
        "同じ名前の取引先があります。重複している場合は「統合」を使ってください。",
      ),
    ).toBeInTheDocument();
    expect(input).toHaveValue("か取引先");
  });

  it("改名失敗時は行の編集状態と入力を保持し再試行を出す", async () => {
    mockRename.mockRejectedValue(new Error("synthetic failure"));
    const user = userEvent.setup();
    renderPage();
    const row = await screen.findByTestId("supplier-row-1");
    const other = screen.getByTestId("supplier-row-2");
    await user.click(within(row).getByRole("button", { name: "名前を変更" }));
    const input = within(row).getByRole("textbox");
    await user.clear(input);
    await user.type(input, "保持名称");
    await user.click(within(row).getByRole("button", { name: "保存" }));
    expect(await within(row).findByRole("button", { name: "再試行" })).toBeInTheDocument();
    expect(input).toHaveValue("保持名称");
    expect(within(other).getByRole("button", { name: "名前を変更" })).toBeEnabled();
  });

  it("改名 pending 中は同じ行だけ無効化する", async () => {
    let resolveRename:
      | ((value: { status: "ok"; data: Supplier } | { status: "error"; error: CmdError }) => void)
      | undefined;
    mockRename.mockImplementation(
      () =>
        new Promise((resolve) => {
          resolveRename = resolve;
        }),
    );
    const user = userEvent.setup();
    renderPage();
    const row = await screen.findByTestId("supplier-row-1");
    const other = screen.getByTestId("supplier-row-2");
    await user.click(within(row).getByRole("button", { name: "名前を変更" }));
    await user.click(within(row).getByRole("button", { name: "保存" }));
    expect(within(row).getByRole("textbox")).toBeDisabled();
    expect(within(row).getByRole("button", { name: "キャンセル" })).toBeDisabled();
    expect(within(other).getByRole("button", { name: "名前を変更" })).toBeEnabled();
    resolveRename?.({ status: "ok", data: { id: 1, name: "あ取引先", created_at: "x" } });
  });

  it("統合は残す側を選ぶ段階 1 を経ないと実行できない", async () => {
    const user = userEvent.setup();
    renderPage();
    await openMerge(user);
    expect(screen.queryByRole("option", { name: "あ取引先" })).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "次へ" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "次へ" })).toHaveAttribute("data-variant", "default");
    expect(screen.queryByRole("button", { name: "統合する" })).not.toBeInTheDocument();
    await user.click(screen.getByRole("combobox", { name: "残す取引先" }));
    expect(screen.queryByRole("option", { name: "あ取引先" })).not.toBeInTheDocument();
  });

  it("REQ-107 Matrix T2: 段階 2 は source 削除・両参照の引き継ぎ・不可逆性を表示する", async () => {
    const user = userEvent.setup();
    renderPage();
    await openMerge(user);
    await selectMergeTarget(user);
    await user.click(screen.getByRole("button", { name: "次へ" }));
    expect(screen.getByText("2件の商品 / 1件の入庫記録が付け替わります")).toBeInTheDocument();
    expect(
      screen.getByText(
        "統合すると、取引先「あ取引先」は取引先一覧から削除され、商品・入庫記録は取引先「か取引先」へ引き継がれます。",
      ),
    ).toBeInTheDocument();
    expect(screen.getByText(/この操作は元に戻せません/)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "残す取引先を選び直す" }));
    expect(screen.getByRole("button", { name: "次へ" })).toBeInTheDocument();
  });

  it("統合するは mergeSuppliers を 1 回だけ呼び成功で dialog を閉じ結果件数と一致する完了通知を出す", async () => {
    let resolveMerge:
      | ((value: {
          status: "ok";
          data: { products_updated: number; receiving_records_updated: number };
        }) => void)
      | undefined;
    mockMerge.mockImplementation(
      () =>
        new Promise((resolve) => {
          resolveMerge = resolve;
        }),
    );
    mockList
      .mockReset()
      .mockResolvedValueOnce({ status: "ok", data: SUPPLIERS })
      .mockResolvedValue({
        status: "ok",
        data: [{ ...SUPPLIERS[1], product_count: 3, receiving_record_count: 1 }, SUPPLIERS[2]],
      });
    const user = userEvent.setup();
    renderPage();
    await openMerge(user);
    await selectMergeTarget(user);
    await user.click(screen.getByRole("button", { name: "次へ" }));
    await user.click(screen.getByRole("button", { name: "統合する" }));
    await waitFor(() => {
      expect(mockMerge).toHaveBeenCalledTimes(1);
    });
    expect(mockMerge).toHaveBeenCalledWith(1, 2);
    expect(screen.getByRole("button", { name: "統合中" })).toBeDisabled();
    expect(screen.getByRole("dialog")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Close" })).not.toBeInTheDocument();
    await user.keyboard("{Escape}");
    expect(screen.getByRole("dialog")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "統合中" }));
    expect(mockMerge).toHaveBeenCalledTimes(1);
    resolveMerge?.({
      status: "ok",
      data: { products_updated: 2, receiving_records_updated: 1 },
    });
    await waitFor(() => {
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });
    expect(toast.success).toHaveBeenCalledWith("取引先を統合しました（2件の商品 / 1件の入庫記録）");
    await waitFor(() => {
      expect(mockList.mock.calls.length).toBeGreaterThanOrEqual(2);
    });
    expect(screen.queryByText("あ取引先")).not.toBeInTheDocument();
    const targetRow = screen.getByTestId("supplier-row-2");
    expect(within(targetRow).getByText("3件")).toBeInTheDocument();
    expect(within(targetRow).getByText("1件")).toBeInTheDocument();
  });

  it("統合失敗時は段階 2 と選択・件数表示を保持し統合できませんでしたと再試行を出す", async () => {
    mockMerge.mockRejectedValue(new Error("synthetic failure"));
    const user = userEvent.setup();
    renderPage();
    await openMerge(user);
    await selectMergeTarget(user);
    await user.click(screen.getByRole("button", { name: "次へ" }));
    await user.click(screen.getByRole("button", { name: "統合する" }));
    expect(await screen.findByText("統合できませんでした")).toBeInTheDocument();
    expect(screen.getByText("2件の商品 / 1件の入庫記録が付け替わります")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "再試行" })).toBeInTheDocument();
  });

  it("一覧取得失敗は取引先を読み込めませんでしたと再試行を出し 0 件と誤認させない", async () => {
    mockList.mockResolvedValue({ status: "error", error: cmdError("internal", "取得失敗") });
    const user = userEvent.setup();
    renderPage();
    expect(
      await screen.findByText("取引先を読み込めませんでした", {}, { timeout: 3_000 }),
    ).toBeInTheDocument();
    expect(screen.queryByText("取引先はまだ登録されていません")).not.toBeInTheDocument();
    mockList.mockResolvedValue({ status: "ok", data: SUPPLIERS });
    await user.click(screen.getByRole("button", { name: "再試行" }));
    expect(await screen.findByText("あ取引先")).toBeInTheDocument();
  });

  it("0 件は取引先はまだ登録されていませんと追加導線を出す", async () => {
    arrangeList([]);
    renderPage();
    expect(await screen.findByText("取引先はまだ登録されていません")).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "新しい取引先を追加" }).length).toBeGreaterThan(0);
  });

  it("not-found 失敗時は一覧が古い可能性の文言と再取得 action を出す", async () => {
    mockRename.mockResolvedValue({
      status: "error",
      error: cmdError("not_found", "取引先が見つかりません"),
    });
    const user = userEvent.setup();
    renderPage();
    const row = await screen.findByTestId("supplier-row-1");
    await user.click(within(row).getByRole("button", { name: "名前を変更" }));
    await user.click(within(row).getByRole("button", { name: "保存" }));
    expect(await within(row).findByText("一覧が古い可能性があります。")).toBeInTheDocument();
    await user.click(within(row).getByRole("button", { name: "一覧を再取得" }));
    await waitFor(() => {
      expect(mockList.mock.calls.length).toBeGreaterThanOrEqual(2);
    });

    await user.click(within(row).getByRole("button", { name: "キャンセル" }));
    mockMerge.mockResolvedValue({
      status: "error",
      error: cmdError("not_found", "統合元の取引先が見つかりません"),
    });
    await openMerge(user);
    await selectMergeTarget(user);
    await user.click(screen.getByRole("button", { name: "次へ" }));
    await user.click(screen.getByRole("button", { name: "統合する" }));
    expect(await screen.findByText("一覧が古い可能性があります。")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "一覧を再取得" }));
    await waitFor(() => {
      expect(mockList.mock.calls.length).toBeGreaterThanOrEqual(3);
    });
  });
});

it("⑮ SC5: 取引先の説明をヘッダー内へまとめる", () => {
  renderPage();
  expect(
    screen
      .getByText("メーカー・ブランドの追加、名称変更、重複した取引先の統合を行います。")
      .closest("header"),
  ).not.toBeNull();
});

it("SPEC-SUP-D2: filters suppliers by name and shows no-match message", async () => {
  const user = userEvent.setup();
  renderPage();
  const search = await screen.findByRole("searchbox", { name: "取引先名で検索" });
  await user.type(search, " あ ");
  expect(screen.getByTestId("supplier-row-1")).toBeInTheDocument();
  expect(screen.queryByTestId("supplier-row-2")).not.toBeInTheDocument();
  await user.clear(search);
  await user.type(search, "不一致");
  expect(screen.getByText("該当する取引先はありません")).toBeInTheDocument();
  expect(screen.queryByText("取引先はまだ登録されていません")).not.toBeInTheDocument();
  await user.clear(search);
  expect(screen.getByTestId("supplier-row-2")).toBeInTheDocument();
  expect(screen.getByRole("table").closest(".overflow-auto")).toHaveClass(
    "max-h-[50vh]",
    "overflow-auto",
  );
  expect(mockList).toHaveBeenCalledTimes(1);
});
