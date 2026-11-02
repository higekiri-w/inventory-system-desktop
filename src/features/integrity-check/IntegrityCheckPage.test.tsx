import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import dateTimeSource from "./IntegrityCheckPage.tsx?raw";

import {
  commands,
  type IntegrityMismatch,
  type SharedJanConflict,
  type StockCopy,
} from "@/lib/bindings";
import { scrollPageToTop } from "@/lib/page-scroll";
import { d052InvalidationOracle, expectExactInvalidations } from "@/test/invalidation-oracle";
import { renderWithRouter } from "@/test/render-with-router";
import { IntegrityCheckPage } from "./IntegrityCheckPage";

vi.mock("@/lib/bindings", () => ({
  commands: { runIntegrityCheck: vi.fn(), fixIntegrity: vi.fn(), listLogs: vi.fn() },
}));
vi.mock("@/lib/page-scroll", () => ({ scrollPageToTop: vi.fn() }));

const runIntegrityCheck = vi.mocked(commands.runIntegrityCheck);
const fixIntegrity = vi.mocked(commands.fixIntegrity);
const listLogs = vi.mocked(commands.listLogs);
const mockScrollPageToTop = vi.mocked(scrollPageToTop);

// 写しの 4 項目（36 の StockCopy）。範囲を渡すと要再確認
function copy(stock: number, range: [number, number] | null = null): StockCopy {
  return {
    stock_quantity: stock,
    stock_recheck: range !== null,
    stock_range_low: range?.[0] ?? null,
    stock_range_high: range?.[1] ?? null,
  };
}

function mismatch(code: string, stock = 10, derived = 7): IntegrityMismatch {
  return {
    product_code: code,
    name: `合成商品 ${code}`,
    stock_unit: "pcs",
    copy: copy(stock),
    derived: copy(derived),
    differing_fields: ["quantity"],
  };
}

function checkResult(items: IntegrityMismatch[], sharedJanConflicts: SharedJanConflict[] = []) {
  return {
    status: "ok" as const,
    data: {
      mismatches: items,
      mismatch_count: items.length,
      checked_count: Math.max(items.length, 5),
      shared_jan_conflicts: sharedJanConflicts,
    },
  };
}

function fixResult(
  fixedCount = 1,
  skippedCount = 0,
  code = "SYN-001",
  oldStock = 10,
  newStock = 7,
) {
  return {
    status: "ok" as const,
    data: {
      fixed_count: fixedCount,
      skipped_count: skippedCount,
      adjustments:
        fixedCount > 0
          ? [
              {
                product_code: code,
                stock_unit: "pcs" as const,
                before: copy(oldStock),
                after: copy(newStock),
              },
            ]
          : [],
    },
  };
}

function renderPage() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return {
    client,
    ...render(
      <QueryClientProvider client={client}>
        <IntegrityCheckPage />
      </QueryClientProvider>,
    ),
  };
}

async function runCheck() {
  await userEvent.setup().click(screen.getByRole("button", { name: "整合性チェック実行" }));
}

async function selectForFix(code: string) {
  await userEvent.setup().click(screen.getByRole("checkbox", { name: `${code}を補正する` }));
}

async function openFixDialog() {
  await userEvent.setup().click(screen.getByRole("button", { name: "補正を確定" }));
}

async function confirmFix() {
  await userEvent.setup().click(screen.getByRole("button", { name: "補正を実行する" }));
}

beforeEach(() => {
  runIntegrityCheck.mockReset();
  fixIntegrity.mockReset();
  listLogs.mockReset();
  mockScrollPageToTop.mockReset();
  listLogs.mockResolvedValue({
    status: "ok",
    data: { items: [], total_count: 0, page: 1, per_page: 1 },
  });
});

describe("UI-13 REQ-904 在庫整合性検証", () => {
  it("test_integrity_page_req904_initial_idle_only_run_button", () => {
    renderPage();
    // Scope 9 UI 表示磨き batch: doc 正本（75 doc 見出し / SCREEN_DESIGN の「年に数回 / 初回のみ」・「全体構成（4エリア）」・「実装フェーズ分類」）に
    // 合わせて PageHeader title を「在庫整合性検証」へ統一（サイドバー表記「整合性検証」は不変）。
    expect(screen.getByRole("heading", { name: "在庫整合性検証" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "整合性チェック実行" })).toBeEnabled();
    expect(screen.queryByText("差異はありません")).not.toBeInTheDocument();
    expect(screen.queryByText("差異が見つかりました")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "補正を確定" })).toBeNull();
  });

  it("U14 shared jan conflicts are listed separately", async () => {
    // 75 の D-118 の節（ADR D10）: 共有 JAN の組は差異の一覧と別の欄。補正の操作を持たない
    runIntegrityCheck.mockResolvedValue(
      checkResult(
        [],
        [
          { jan_code: "4901234567894", product_codes: ["SYN-A01", "SYN-A02"] },
          { jan_code: "4909876543212", product_codes: ["SYN-B01", "SYN-B02", "SYN-B03"] },
        ],
      ),
    );
    renderPage();
    await runCheck();

    const conflicts = await screen.findByRole("alert", { name: "共有 JAN の商品" });
    expect(conflicts).toHaveAttribute("data-variant", "warning");
    expect(
      within(conflicts).getByText(
        "在庫連動の商品と同じ JAN の商品があります。レジの売上で在庫が減りません。どちらかの商品の JAN を直すか、在庫連動を外してください。",
      ),
    ).toBeInTheDocument();
    const rows = within(conflicts).getAllByRole("listitem");
    expect(rows.map((row) => row.textContent)).toEqual([
      "JAN 4901234567894: SYN-A01、SYN-A02",
      "JAN 4909876543212: SYN-B01、SYN-B02、SYN-B03",
    ]);
    expect(conflicts.querySelector('svg[aria-hidden="true"]')).not.toBeNull();
    expect(within(conflicts).queryByRole("button")).toBeNull();
    // 写しの差異が無い表示は組の欄と別に出る
    const noDiff = screen.getByText("差異はありません").closest('[data-slot="alert"]');
    expect(noDiff).not.toBeNull();
    expect(conflicts.contains(noDiff)).toBe(false);
    expect(screen.queryByRole("button", { name: "補正を確定" })).toBeNull();
  });

  it("U14 no shared jan block when there are no conflicts", async () => {
    runIntegrityCheck.mockResolvedValue(checkResult([]));
    renderPage();
    await runCheck();
    expect(await screen.findByText("差異はありません")).toBeInTheDocument();
    expect(screen.queryByText("在庫連動の商品と同じ JAN の商品があります")).toBeNull();
  });

  it("test_integrity_page_req904_remount_resets_to_idle", async () => {
    runIntegrityCheck.mockResolvedValue(checkResult([mismatch("SYN-OLD")]));
    const first = renderPage();
    await runCheck();
    expect(await screen.findByText("合成商品 SYN-OLD")).toBeInTheDocument();
    first.unmount();
    renderPage();
    expect(screen.getByRole("button", { name: "整合性チェック実行" })).toBeEnabled();
    expect(screen.queryByText("合成商品 SYN-OLD")).toBeNull();
    expect(runIntegrityCheck).toHaveBeenCalledTimes(1);
  });

  it("test_integrity_page_req904_rerun_replaces_result_and_clears_selection", async () => {
    runIntegrityCheck
      .mockResolvedValueOnce(checkResult([mismatch("SYN-OLD")]))
      .mockResolvedValueOnce(checkResult([mismatch("SYN-NEW", 4, 9)]));
    renderPage();
    await runCheck();
    await selectForFix("SYN-OLD");
    await userEvent.setup().click(screen.getByRole("button", { name: "再度チェック" }));
    expect(await screen.findByText("合成商品 SYN-NEW")).toBeInTheDocument();
    expect(screen.queryByText("合成商品 SYN-OLD")).toBeNull();
    expect(screen.getByRole("button", { name: "補正を確定" })).toBeDisabled();
  });

  it("test_integrity_page_req904_last_checked_from_integrity_check_log", async () => {
    listLogs.mockResolvedValue({
      status: "ok",
      data: {
        items: [
          {
            id: 7,
            operation_type: "integrity_check",
            summary: "合成チェック",
            detail_json: null,
            created_at: "2026-07-15T09:08:07",
          },
        ],
        total_count: 1,
        page: 1,
        per_page: 1,
      },
    });
    const first = renderPage();
    expect(await screen.findByText("直近の確認日時: 2026-07-15 09:08:07")).toBeInTheDocument();
    // ⑰ SC6 / UIDISP-D6: prose は等幅の表セル書体にしない。祖先の継承も検査する。
    const checkedAt = screen.getByText("直近の確認日時: 2026-07-15 09:08:07");
    for (let element: HTMLElement | null = checkedAt; element; element = element.parentElement) {
      expect(element).not.toHaveClass("font-mono");
      expect(element).not.toHaveClass("tabular-nums");
    }
    expect(listLogs).toHaveBeenCalledWith({
      page: 1,
      per_page: 1,
      operation_type: "integrity_check",
      start_date: null,
      end_date: null,
    });
    first.unmount();
    listLogs.mockResolvedValue({
      status: "ok",
      data: { items: [], total_count: 0, page: 1, per_page: 1 },
    });
    renderPage();
    expect(await screen.findByText("直近の確認日時: まだ実行されていません")).toBeInTheDocument();
  });

  it("test_integrity_page_req904_fix_requires_confirm_and_selected_codes_only", async () => {
    runIntegrityCheck.mockResolvedValue(
      checkResult([mismatch("SYN-001"), mismatch("SYN-002", 3, 8)]),
    );
    fixIntegrity.mockResolvedValue(fixResult());
    renderPage();
    await runCheck();
    await selectForFix("SYN-001");
    await openFixDialog();
    expect(fixIntegrity).not.toHaveBeenCalled();
    expect(screen.getByRole("button", { name: "補正を実行する" })).toHaveAttribute(
      "data-variant",
      "destructive",
    );
    await confirmFix();
    await waitFor(() => {
      expect(fixIntegrity).toHaveBeenCalledTimes(1);
    });
    expect(fixIntegrity).toHaveBeenCalledWith(["SYN-001"]);
  });

  it("D-052-C12 fix success invalidates the exact independent oracle set", async () => {
    runIntegrityCheck.mockResolvedValue(checkResult([mismatch("SYN-001")]));
    fixIntegrity.mockResolvedValue(fixResult());
    const { client } = renderPage();
    const invalidateSpy = vi.spyOn(client, "invalidateQueries");

    await runCheck();
    await selectForFix("SYN-001");
    await openFixDialog();
    await confirmFix();

    await waitFor(() => {
      expectExactInvalidations(invalidateSpy.mock.calls, d052InvalidationOracle.integrityFix());
    });
  });

  it("test_integrity_page_req904_t7_uses_integrity_fix_button_copy", async () => {
    runIntegrityCheck.mockResolvedValue(checkResult([mismatch("SYN-001")]));
    renderPage();
    await runCheck();

    expect(screen.getByRole("button", { name: "補正を確定" })).toBeVisible();
    const retiredLabel = ["棚卸し", "補正として確定"].join("");
    expect(screen.queryByRole("button", { name: retiredLabel })).toBeNull();
  });

  it("test_integrity_page_req904_t8_dialog_copy_is_visible_and_accessible", async () => {
    runIntegrityCheck.mockResolvedValue(checkResult([mismatch("SYN-001")]));
    renderPage();
    await runCheck();
    await selectForFix("SYN-001");
    await openFixDialog();

    const dialog = screen.getByRole("alertdialog");
    const dialogTitle = within(dialog).getByRole("heading", {
      name: "在庫数を記録から求め直した値に合わせて補正します",
    });
    const warningTitle = within(dialog).getByText("補正すると元に戻せません");
    expect(warningTitle.closest('[data-slot="alert"]')).toHaveAttribute("data-variant", "warning");
    const warningDescription = within(dialog).getByText(
      "選択した商品の保存している値を記録から求め直した値に合わせて更新し、操作ログに記録します。",
    );
    for (const visibleCopy of [dialogTitle, warningTitle, warningDescription]) {
      expect(visibleCopy).toBeVisible();
      expect(visibleCopy).not.toHaveClass("sr-only");
      expect(visibleCopy).not.toHaveAttribute("hidden");
      expect(visibleCopy).not.toHaveAttribute("aria-hidden");
    }
    expect(
      within(dialog).getByText(
        "補正すると元に戻せません。選択した商品の保存している値を記録から求め直した値に合わせて更新し、操作ログに記録します。",
      ),
    ).toHaveClass("sr-only");
    expect(
      within(dialog).getByText("補正する商品（保存している値 → 記録から求め直した値）"),
    ).toBeVisible();
  });

  it("test_integrity_page_req904_no_select_all_and_disabled_when_none_selected", async () => {
    runIntegrityCheck.mockResolvedValue(
      checkResult([mismatch("SYN-001"), mismatch("SYN-002", 3, 8)]),
    );
    renderPage();
    await runCheck();
    expect(screen.getAllByRole("checkbox")).toHaveLength(2);
    expect(screen.queryByRole("checkbox", { name: /すべて|全選択/ })).toBeNull();
    expect(screen.getByRole("button", { name: "補正を確定" })).toBeDisabled();
  });

  it("test_integrity_page_req904_confirm_dialog_lists_selected_adjustments", async () => {
    runIntegrityCheck.mockResolvedValue(
      checkResult([mismatch("SYN-001", 10, 7), mismatch("SYN-002", 3, 8)]),
    );
    renderPage();
    await runCheck();
    await selectForFix("SYN-001");
    await selectForFix("SYN-002");
    await openFixDialog();
    const dialog = screen.getByRole("alertdialog");
    for (const text of ["SYN-001", "10 → 7", "SYN-002", "3 → 8"]) {
      expect(within(dialog).getByText(text)).toBeInTheDocument();
    }
  });

  it("test_integrity_page_req904_running_overlay_blocks_actions", async () => {
    let resolveCheck: ((value: ReturnType<typeof checkResult>) => void) | undefined;
    let resolveFix: ((value: ReturnType<typeof fixResult>) => void) | undefined;
    runIntegrityCheck.mockImplementation(() => new Promise((resolve) => (resolveCheck = resolve)));
    fixIntegrity.mockImplementation(() => new Promise((resolve) => (resolveFix = resolve)));
    renderPage();
    await runCheck();
    expect(screen.getByRole("status")).toHaveTextContent("在庫データを確認しています");
    expect(screen.getByRole("progressbar")).not.toHaveAttribute("aria-valuenow");
    const button = screen.getByRole("button", { name: "整合性チェック実行" });
    expect(button).toBeDisabled();
    await userEvent.setup().click(button);
    expect(runIntegrityCheck).toHaveBeenCalledTimes(1);
    resolveCheck?.(
      checkResult(
        Array.from({ length: 101 }, (_, index) =>
          mismatch(`SYN-${String(index).padStart(3, "0")}`),
        ),
      ),
    );
    expect(await screen.findByText("差異が見つかりました")).toBeInTheDocument();
    // SC20 / DSR-08: 既存文言・roleを保ってwarning variantへ移行する。
    expect(screen.getByText("差異が見つかりました").closest('[data-slot="alert"]')).toHaveAttribute(
      "data-variant",
      "warning",
    );
    expect(screen.getByText("差異が見つかりました").closest('[data-slot="alert"]')).toHaveAttribute(
      "role",
      "status",
    );
    await selectForFix("SYN-001");
    await openFixDialog();
    await confirmFix();
    expect(screen.getByText("補正を記録しています").closest('[role="status"]')).not.toBeNull();
    expect(screen.queryByRole("button", { name: "次のページ" })).toBeNull();
    resolveFix?.(fixResult());
    expect(await screen.findByText("1件を補正しました")).toBeInTheDocument();
  });

  it("test_integrity_page_req904_fix_success_no_auto_recheck_shows_summary", async () => {
    runIntegrityCheck.mockResolvedValue(checkResult([mismatch("SYN-001")]));
    fixIntegrity.mockResolvedValue(fixResult());
    renderPage();
    await runCheck();
    await selectForFix("SYN-001");
    await openFixDialog();
    await confirmFix();
    expect(await screen.findByText("1件を補正しました")).toBeInTheDocument();
    const adjustmentRow = screen.getByText("10 → 7").closest("li");
    expect(adjustmentRow).not.toBeNull();
    expect(adjustmentRow).not.toHaveClass("rounded-md", "border");
    expect(adjustmentRow?.parentElement).toHaveClass("divide-y");
    expect(screen.queryByText("一部の商品は補正されませんでした")).toBeNull();
    expect(runIntegrityCheck).toHaveBeenCalledTimes(1);
  });

  it("test_integrity_page_req904_fixed_rows_badged_and_recheck_affordance", async () => {
    runIntegrityCheck.mockResolvedValue(checkResult([mismatch("SYN-001")]));
    fixIntegrity.mockResolvedValue(fixResult());
    renderPage();
    await runCheck();
    await selectForFix("SYN-001");
    await openFixDialog();
    await confirmFix();
    expect(await screen.findByText("補正済み")).toBeInTheDocument();
    // SC7 / DSR-22: 完了は直塗りでなくsoft success + icon。
    const fixedBadge = screen.getByText("補正済み");
    expect(fixedBadge).toHaveAttribute("data-variant", "outline");
    expect(fixedBadge).toHaveAttribute("data-tone", "success");
    expect(fixedBadge).toHaveClass(
      "border-success-border",
      "bg-success-soft",
      "text-success-strong",
    );
    expect(fixedBadge).not.toHaveClass("bg-success");
    expect(fixedBadge.querySelector('svg[aria-hidden="true"]')).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "再度チェック" })).toBeEnabled();
  });

  it("test_integrity_page_req904_skipped_count_warning_visible", async () => {
    runIntegrityCheck.mockResolvedValue(checkResult([mismatch("SYN-001")]));
    fixIntegrity.mockResolvedValue(fixResult(0, 1));
    renderPage();
    await runCheck();
    await selectForFix("SYN-001");
    await openFixDialog();
    await confirmFix();
    const warning = await screen.findByRole("alert");
    expect(warning).toHaveTextContent("一部の商品は補正されませんでした");
    expect(warning).toHaveAttribute("data-variant", "warning");
    expect(warning).toHaveTextContent("1件");
  });

  it("test_integrity_page_req904_cmderror_japanese_message_retry_keeps_selection", async () => {
    runIntegrityCheck
      .mockResolvedValueOnce({
        status: "error",
        error: {
          kind: "internal",
          message: "合成チェックエラー。もう一度お試しください",
          field: null,
          error_id: "E-20260726-204901-a1b2",
          stocktake_recovery: null,
        },
      })
      .mockResolvedValueOnce(checkResult([mismatch("SYN-001")]));
    fixIntegrity
      .mockResolvedValueOnce({
        status: "error",
        error: {
          kind: "internal",
          message: "合成補正エラー。もう一度お試しください",
          field: null,
          error_id: "E-20260726-204902-c3d4",
          stocktake_recovery: null,
        },
      })
      .mockResolvedValueOnce(fixResult());
    renderPage();
    await runCheck();
    expect(
      await screen.findByText(
        "合成チェックエラー。もう一度お試しください（エラーID: E-20260726-204901-a1b2）。詳細は診断ログに記録されています。",
      ),
    ).toBeInTheDocument();
    await userEvent.setup().click(screen.getByRole("button", { name: "再試行" }));
    expect(await screen.findByText("合成商品 SYN-001")).toBeInTheDocument();
    expect(runIntegrityCheck).toHaveBeenCalledTimes(2);
    await selectForFix("SYN-001");
    await openFixDialog();
    await confirmFix();
    expect(
      await screen.findByText(
        "合成補正エラー。もう一度お試しください（エラーID: E-20260726-204902-c3d4）。詳細は診断ログに記録されています。",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("checkbox", { name: "SYN-001を補正する" })).toBeChecked();
    await userEvent.setup().click(screen.getByRole("button", { name: "補正を再試行" }));
    await waitFor(() => {
      expect(fixIntegrity).toHaveBeenCalledTimes(2);
    });
    expect(fixIntegrity).toHaveBeenLastCalledWith(["SYN-001"]);
  });

  it("test_integrity_page_req904_pagination_100_per_page", async () => {
    const items = Array.from({ length: 101 }, (_, index) =>
      mismatch(`SYN-${String(index).padStart(3, "0")}`),
    );
    runIntegrityCheck.mockResolvedValue(checkResult(items));
    renderPage();
    await runCheck();
    expect(await screen.findByText("合成商品 SYN-000")).toBeInTheDocument();
    expect(screen.getByText("合成商品 SYN-099")).toBeInTheDocument();
    expect(screen.queryByText("合成商品 SYN-100")).toBeNull();
    // S3g（round 2/3 是正）: 上部 PaginationSummary + 下部 Pagination の両方に同じ件数
    // 文言が現れるため、単数一致の getByText ではなく getAllByText の件数で確認する。
    expect(screen.getAllByText("全 101 件のうち 1〜100 件を表示（1 / 2 ページ）")).toHaveLength(2);
    await userEvent.setup().click(screen.getByRole("button", { name: "次のページ" }));
    expect(await screen.findByText("合成商品 SYN-100")).toBeInTheDocument();
    expect(screen.queryByText("合成商品 SYN-000")).toBeNull();
  });

  it("SC4f: 表示件数を50へ変更すると表示行を50件以下に制限し追加requestを発火しない", async () => {
    runIntegrityCheck.mockResolvedValue(
      checkResult(
        Array.from({ length: 101 }, (_, index) =>
          mismatch(`SYN-${String(index).padStart(3, "0")}`),
        ),
      ),
    );
    const user = userEvent.setup();
    renderPage();
    await runCheck();
    expect(await screen.findByText("合成商品 SYN-099")).toBeInTheDocument();
    const listLogsCallsBeforeSelection = listLogs.mock.calls.length;

    await user.click(screen.getByRole("combobox", { name: "表示件数" }));
    await user.click(screen.getByRole("option", { name: "50 件" }));

    await waitFor(() => {
      expect(screen.getAllByRole("checkbox")).toHaveLength(50);
    });
    expect(screen.queryByText("合成商品 SYN-050")).toBeNull();
    expect(listLogs).toHaveBeenCalledTimes(listLogsCallsBeforeSelection);
  });

  it("SC7: 差異0件でも表示件数Selectを表示し選択後も一覧行は空のまま", async () => {
    runIntegrityCheck.mockResolvedValue(checkResult([]));
    const user = userEvent.setup();
    renderPage();
    await runCheck();
    expect(await screen.findByText("差異はありません")).toBeInTheDocument();
    expect(screen.queryByRole("checkbox", { name: /を補正する/ })).toBeNull();

    await user.click(screen.getByRole("combobox", { name: "表示件数" }));
    await user.click(screen.getByRole("option", { name: "50 件" }));

    expect(screen.getByRole("combobox", { name: "表示件数" })).toHaveTextContent("50 件");
    expect(screen.queryByRole("checkbox", { name: /を補正する/ })).toBeNull();
  });

  it("test_integrity_page_req904_mismatch_columns_complete", async () => {
    runIntegrityCheck.mockResolvedValue(checkResult([mismatch("SYN-001")]));
    renderPage();
    await runCheck();
    for (const heading of [
      "商品コード",
      "商品名",
      "違う項目",
      "保存している値",
      "記録から求め直した値",
    ]) {
      expect(screen.getByRole("columnheader", { name: heading })).toBeInTheDocument();
    }
  });

  it("test_integrity_page_req904_state_labels_not_color_only", async () => {
    runIntegrityCheck
      .mockResolvedValueOnce(
        checkResult([
          mismatch("SYN-PLUS", 10, 7),
          { ...mismatch("SYN-MINUS", 3, 3), differing_fields: ["recheck", "range"] },
        ]),
      )
      .mockResolvedValueOnce(checkResult([]));
    fixIntegrity.mockResolvedValue(fixResult(1, 0, "SYN-PLUS"));
    renderPage();
    await runCheck();
    expect(await screen.findByText("差異が見つかりました")).toBeInTheDocument();
    expect(screen.getByText("在庫数")).toBeInTheDocument();
    expect(screen.getByText("要再確認・範囲")).toBeInTheDocument();
    await selectForFix("SYN-PLUS");
    await openFixDialog();
    await confirmFix();
    expect(await screen.findByText("補正済み")).toBeInTheDocument();
    await userEvent.setup().click(screen.getByRole("button", { name: "再度チェック" }));
    expect(await screen.findByText("差異はありません")).toBeInTheDocument();
  });
});

describe("IntegrityCheckPage Lane 4 S1g/S3g: table wrapper rounded-lg, top summary, Select wrapper excluded from frame", () => {
  it("SC4g: table wrapper has overflow-x-auto rounded-lg border, no bg-card, no rounded-md leftover", async () => {
    runIntegrityCheck.mockResolvedValue(checkResult([mismatch("SYN-001")]));
    const { container } = renderPage();
    await runCheck();
    await screen.findByText("合成商品 SYN-001");
    const wrapper = container.querySelector(".overflow-x-auto");
    expect(wrapper).not.toBeNull();
    const tokens = (wrapper?.className ?? "").split(/\s+/);
    expect(tokens).toContain("rounded-lg");
    expect(tokens).toContain("border");
    expect(tokens).not.toContain("rounded-md");
    expect(tokens).not.toContain("bg-card");
  });

  it("SC3g: renders the top PaginationSummary above the table", async () => {
    runIntegrityCheck.mockResolvedValue(checkResult([mismatch("SYN-001")]));
    renderPage();
    await runCheck();
    expect(
      await screen.findByText("全 1 件のうち 1〜1 件を表示（1 / 1 ページ）"),
    ).toBeInTheDocument();
  });

  it("round 2 是正回帰: the per-page select wrapper does not receive the bg-card frame", async () => {
    runIntegrityCheck.mockResolvedValue(checkResult([mismatch("SYN-001")]));
    const { container } = renderPage();
    await runCheck();
    const perPageTrigger = await screen.findByRole("combobox", { name: "表示件数" });
    const selectWrapper = perPageTrigger.closest(".grid.gap-1");
    expect(selectWrapper).not.toBeNull();
    expect((selectWrapper?.className ?? "").split(/\s+/)).not.toContain("bg-card");
    expect(container.querySelectorAll(".rounded-lg.border.bg-card.p-4")).toHaveLength(0);
  });
});

describe("IntegrityCheckPage perPage scroll（UI-13）", () => {
  it("SC9a: 表示件数変更で画面を先頭へ戻す", async () => {
    runIntegrityCheck.mockResolvedValue(
      checkResult(Array.from({ length: 101 }, (_, index) => mismatch(`SYN-${String(index)}`))),
    );
    renderPage();
    await runCheck();
    await screen.findByText("合成商品 SYN-0");

    const user = userEvent.setup();
    await user.click(screen.getByRole("combobox", { name: "表示件数" }));
    await user.click(screen.getByRole("option", { name: "50 件" }));

    expect(mockScrollPageToTop).toHaveBeenCalledTimes(1);
  });
});

// UI-13 / REQ-904 / Matrix U5: 違う項目は Badge の色でなく文字の語で示す
it("違う項目を文字で示す", async () => {
  runIntegrityCheck.mockResolvedValue(
    checkResult([
      { ...mismatch("U5-Q"), differing_fields: ["quantity"] },
      { ...mismatch("U5-RR", 5, 5), differing_fields: ["recheck", "range"] },
      { ...mismatch("U5-R", 5, 5), differing_fields: ["range"] },
    ]),
  );
  renderPage();
  await runCheck();
  for (const [code, label] of [
    ["U5-Q", "在庫数"],
    ["U5-RR", "要再確認・範囲"],
    ["U5-R", "範囲"],
  ] as const) {
    const row = (await screen.findByText(`合成商品 ${code}`)).closest("tr");
    if (!row) throw new Error("difference row missing");
    const badge = within(row).getByText(label);
    expect(badge).toHaveAttribute("data-variant", "outline");
    expect(badge).not.toHaveAttribute("data-tone");
  }
});

it("⑰ SC6 / UIDISP-D6: 共有 formatDateTime を import しローカル定義を持たない", () => {
  expect(dateTimeSource).toMatch(
    /import\s*\{[^}]*\bformatDateTime\b[^}]*\}\s*from\s*"@\/lib\/date-time"/,
  );
  expect(dateTimeSource).not.toMatch(/function\s+(?:formatDateTime|formatCheckedAt)\s*\(/);
  expect(dateTimeSource).not.toContain("formatCheckedAt");
});

it("SPEC-FILTER-LABEL-RT-1 D-RT6: 差異の見出し行の下に説明を全幅で置く", async () => {
  runIntegrityCheck.mockResolvedValue(checkResult([mismatch("SYN-001")]));
  renderPage();
  await runCheck();
  const heading = screen.getByRole("heading", { level: 2, name: "差異のある商品" });
  const description = screen.getByText(
    "補正する商品を行ごとに選び、内容を確認して確定してください。",
  );
  expect(heading).toHaveAttribute("id", "integrity-difference-heading");
  expect(heading).toHaveClass("min-w-0", "flex-1");
  expect(heading.parentElement).toHaveClass(
    "flex",
    "flex-wrap",
    "items-start",
    "justify-between",
    "gap-3",
  );
  expect(description.tagName).toBe("P");
  expect(heading.parentElement?.nextElementSibling).toBe(description);
  expect(description.parentElement).toHaveClass("space-y-1");
  expect(screen.getByRole("button", { name: "補正を確定" })).toHaveClass("shrink-0");
});

// UI-13-D3 / REQ-904 / SPEC-UNIT-D3 の表の 8・11: 長さの商品は m の形、個数の商品は今どおり語を付けない数
it("UI-13-D3: m の商品の不一致は 27.5 m・27 m、補正の結果は 27.5 m → 27 m", async () => {
  runIntegrityCheck.mockResolvedValue(
    checkResult([
      { ...mismatch("SYN-M", 2750, 2700), stock_unit: "m" },
      { ...mismatch("SYN-CM", 100, 150), stock_unit: "m" },
      mismatch("SYN-PCS", 1234, 1230),
    ]),
  );
  fixIntegrity.mockResolvedValue({
    status: "ok",
    data: {
      fixed_count: 2,
      skipped_count: 0,
      adjustments: [
        { product_code: "SYN-M", stock_unit: "m", before: copy(2750), after: copy(2700) },
        { product_code: "SYN-PCS", stock_unit: "pcs", before: copy(1234), after: copy(1230) },
      ],
    },
  });
  renderPage();
  await runCheck();

  const cells = (code: string) =>
    Array.from((screen.getByText(code).closest("tr") ?? document.body).querySelectorAll("td")).map(
      (cell) => cell.textContent,
    );
  await screen.findByText("合成商品 SYN-M");
  // 列は 商品コード・商品名・違う項目・保存している値・記録から求め直した値（75。差の列は無い）
  expect(cells("SYN-M").slice(3, 5)).toEqual(["27.5 m", "27 m"]);
  expect(cells("SYN-CM").slice(3, 5)).toEqual(["1 m", "1.5 m"]);
  expect(cells("SYN-PCS").slice(3, 5)).toEqual(["1,234", "1,230"]);

  await selectForFix("SYN-M");
  await selectForFix("SYN-PCS");
  await openFixDialog();
  const dialog = screen.getByRole("alertdialog");
  expect(within(dialog).getByText("27.5 m → 27 m")).toBeInTheDocument();
  expect(within(dialog).getByText("1234 → 1230")).toBeInTheDocument();
  await confirmFix();
  expect(await screen.findByText("27.5 m → 27 m")).toBeInTheDocument();
  expect(screen.getByText("1234 → 1230")).toBeInTheDocument();
});

// Matrix U1〜U8（75 の冒頭の注記・§75.4〜§75.7、36 の型の形）。値は 36 の例と違う合成の値
describe("UI-13 REQ-904 写しの 4 項目と記録から求め直した値", () => {
  function renderPageWithRouter() {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    return renderWithRouter(
      <QueryClientProvider client={client}>
        <IntegrityCheckPage />
      </QueryClientProvider>,
      "/settings/integrity",
    );
  }

  function logsResult(items: { detail_json: string | null; created_at: string }[]) {
    return {
      status: "ok" as const,
      data: {
        items: items.map((item, index) => ({
          id: 100 - index,
          operation_type: "integrity_fix",
          summary: "合成の補正",
          ...item,
        })),
        total_count: items.length,
        page: 1,
        per_page: 100,
      },
    };
  }

  const migrationRow = {
    detail_json: JSON.stringify({
      fixed_count: 3,
      skipped_count: 0,
      adjustments: [],
      trigger: "migration",
    }),
    created_at: "2026-10-19T08:30:00",
  };
  const ordinaryFixRow = {
    detail_json: JSON.stringify({ fixed_count: 1, skipped_count: 0, adjustments: [] }),
    created_at: "2026-10-20T10:00:00",
  };
  const noticeText =
    "更新のときに在庫数を記録から求め直し、3商品の在庫数を直しました（2026-10-19 08:30:00）。直した前後は操作ログで見られます。";

  function mockLogs(fixRows: { detail_json: string | null; created_at: string }[] | "error") {
    listLogs.mockImplementation((query) => {
      if (query.operation_type !== "integrity_fix") return Promise.resolve(logsResult([]));
      if (fixRows === "error")
        return Promise.resolve({
          status: "error" as const,
          error: {
            kind: "internal",
            message: "合成の読取りエラー",
            field: null,
            error_id: null,
            stocktake_recovery: null,
          },
        });
      return Promise.resolve(logsResult(fixRows));
    });
  }

  it("違う項目と、保存している値・記録から求め直した値を並べる", async () => {
    runIntegrityCheck.mockResolvedValue(
      checkResult([
        {
          ...mismatch("U1-001"),
          copy: copy(6),
          derived: copy(5, [4, 5]),
          differing_fields: ["quantity", "recheck", "range"],
        },
      ]),
    );
    renderPage();
    await runCheck();
    for (const heading of ["違う項目", "保存している値", "記録から求め直した値"]) {
      expect(screen.getByRole("columnheader", { name: heading })).toBeInTheDocument();
    }
    const row = (await screen.findByText("合成商品 U1-001")).closest("tr");
    if (!row) throw new Error("difference row missing");
    expect(within(row).getByText("在庫数・要再確認・範囲")).toBeInTheDocument();
    expect(within(row).getByText("6")).toBeInTheDocument();
    expect(within(row).getByText("5（要再確認 4〜5）")).toBeInTheDocument();
    expect(screen.queryByText(/システム在庫が多い|入出庫の合計が多い|入出庫の合計/)).toBeNull();
  });

  it("数が同じで要再確認だけが違う商品も一覧に出す", async () => {
    runIntegrityCheck.mockResolvedValue(
      checkResult([
        {
          ...mismatch("U2-001"),
          copy: copy(8),
          derived: copy(8, [7, 9]),
          differing_fields: ["recheck", "range"],
        },
      ]),
    );
    renderPage();
    await runCheck();
    expect(await screen.findByText("差異が見つかりました")).toBeInTheDocument();
    const row = screen.getByText("合成商品 U2-001").closest("tr");
    if (!row) throw new Error("difference row missing");
    expect(within(row).getByText("要再確認・範囲")).toBeInTheDocument();
    expect(within(row).getByText("8（要再確認 7〜9）")).toBeInTheDocument();
    expect(screen.queryByText("差異なし")).toBeNull();
    expect(screen.queryByText("差異はありません")).toBeNull();
  });

  it("補正の確認で保存している値から記録から求め直した値への前後を出す", async () => {
    runIntegrityCheck.mockResolvedValue(
      checkResult([
        {
          ...mismatch("U3-CM"),
          stock_unit: "m",
          copy: copy(1250),
          derived: copy(1200, [1150, 1300]),
          differing_fields: ["quantity", "recheck", "range"],
        },
        mismatch("U3-PCS", 12, 9),
      ]),
    );
    renderPage();
    await runCheck();
    await selectForFix("U3-CM");
    await openFixDialog();
    const dialog = screen.getByRole("alertdialog");
    expect(within(dialog).getByText("U3-CM")).toBeInTheDocument();
    expect(within(dialog).getByText("12.5 m → 12 m（要再確認 11.5 m〜13 m）")).toBeInTheDocument();
    expect(within(dialog).queryByText("U3-PCS")).toBeNull();
  });

  it("補正の結果を before と after から出す", async () => {
    runIntegrityCheck.mockResolvedValue(checkResult([mismatch("U4-001", 5, 6)]));
    fixIntegrity.mockResolvedValue({
      status: "ok",
      data: {
        fixed_count: 1,
        skipped_count: 0,
        adjustments: [
          {
            product_code: "U4-001",
            stock_unit: "pcs",
            before: copy(5, [4, 6]),
            after: copy(6),
          },
        ],
      },
    });
    renderPage();
    await runCheck();
    await selectForFix("U4-001");
    await openFixDialog();
    await confirmFix();
    expect(await screen.findByText("1件を補正しました")).toBeInTheDocument();
    const resultRow = screen.getByText("5（要再確認 4〜6） → 6").closest("li");
    expect(resultRow).toHaveTextContent("U4-001");
  });

  it("run_integrity_check の copy と derived を表示に使う", async () => {
    runIntegrityCheck.mockResolvedValue(
      checkResult([{ ...mismatch("U7-001"), copy: copy(13), derived: copy(21) }]),
    );
    renderPage();
    await runCheck();
    const row = (await screen.findByText("合成商品 U7-001")).closest("tr");
    if (!row) throw new Error("difference row missing");
    const cells = Array.from(row.querySelectorAll("td")).map((cell) => cell.textContent);
    expect(cells.slice(3, 5)).toEqual(["13", "21"]);
  });

  it("移行の integrity_fix があれば日時と商品の数と操作ログへの導線を出す", async () => {
    mockLogs([migrationRow]);
    renderPageWithRouter();
    expect(await screen.findByText(noticeText)).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "操作ログを開く" })).toHaveAttribute(
      "href",
      "/settings/logs",
    );
    expect(listLogs).toHaveBeenCalledWith({
      page: 1,
      per_page: 100,
      operation_type: "integrity_fix",
      start_date: null,
      end_date: null,
    });
  });

  it("移行の行の後に別の integrity_fix があっても知らせを出す", async () => {
    mockLogs([ordinaryFixRow, migrationRow]);
    const first = renderPageWithRouter();
    expect(await screen.findByText(noticeText)).toBeInTheDocument();
    first.unmount();

    // trigger の無い行だけでは知らせを出さない
    mockLogs([ordinaryFixRow]);
    renderPageWithRouter();
    expect(await screen.findByRole("button", { name: "整合性チェック実行" })).toBeEnabled();
    await waitFor(() => {
      expect(
        listLogs.mock.calls.filter(([query]) => query.operation_type === "integrity_fix"),
      ).toHaveLength(2);
    });
    expect(screen.queryByText(/更新のときに在庫数を記録から求め直し/)).toBeNull();
    expect(screen.queryByRole("link", { name: "操作ログを開く" })).toBeNull();
  });

  it("ログの読取りに失敗してもチェックを止めない", async () => {
    mockLogs("error");
    runIntegrityCheck.mockResolvedValue(checkResult([]));
    renderPageWithRouter();
    const button = await screen.findByRole("button", { name: "整合性チェック実行" });
    await waitFor(() => {
      expect(listLogs).toHaveBeenCalledWith(
        expect.objectContaining({ operation_type: "integrity_fix" }),
      );
    });
    expect(button).toBeEnabled();
    await userEvent.setup().click(button);
    expect(await screen.findByText("差異はありません")).toBeInTheDocument();
    expect(screen.queryByText(/更新のときに在庫数を記録から求め直し/)).toBeNull();
  });
});
