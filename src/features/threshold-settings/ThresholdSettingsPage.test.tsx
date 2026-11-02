// src/features/threshold-settings/ThresholdSettingsPage.test.tsx
//
// SPEC-DISP-B2-1 / D-B1・D-B6: 見出しと入力検証4系統の文言。
// UI-11a Test Design Matrix T1〜T10, T12（docs/plans/2026-07-07-ui11a-threshold-settings-implementation.md）

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { commands } from "@/lib/bindings";
import type { CmdError } from "@/lib/bindings";
import { d052InvalidationOracle, expectExactInvalidations } from "@/test/invalidation-oracle";

import { ThresholdSettingsPage } from "./ThresholdSettingsPage";

vi.mock("@/hooks/useUnsavedChangesWarning", () => ({
  useUnsavedChangesWarning: () => ({
    isBlocked: false,
    continueEditing: vi.fn(),
    discardAndProceed: vi.fn(),
  }),
}));

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

vi.mock("@/lib/bindings", () => ({
  commands: {
    getSettings: vi.fn(),
    updateSetting: vi.fn(),
  },
}));

const mockGetSettings = vi.mocked(commands.getSettings);
const mockUpdateSetting = vi.mocked(commands.updateSetting);

function ok<T>(data: T) {
  return { status: "ok" as const, data };
}

function cmdError(message: string): { status: "error"; error: CmdError } {
  return {
    status: "error" as const,
    error: { kind: "internal", message, field: null, error_id: null, stocktake_recovery: null },
  };
}

function defaultSettings() {
  return ok([
    { key: "stock_low_threshold", value: "3", updated_at: "2026-07-06T00:00:00" },
    { key: "stock_low_threshold_fabric", value: "500", updated_at: "2026-07-06T00:00:00" },
    { key: "backup_enabled", value: "1", updated_at: "2026-07-06T00:00:00" },
  ]);
}

function renderWithClient(ui: ReactNode) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
  const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
  return {
    queryClient,
    invalidateSpy,
    ...render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>),
  };
}

beforeEach(() => {
  mockGetSettings.mockReset();
  mockUpdateSetting.mockReset();
  mockGetSettings.mockResolvedValue(defaultSettings());
  mockUpdateSetting.mockResolvedValue(ok(null));
});

afterEach(() => {
  vi.restoreAllMocks();
});

async function renderReady() {
  const utils = renderWithClient(<ThresholdSettingsPage />);
  await screen.findByLabelText("一般商品の基準（必須）");
  return utils;
}

describe("ThresholdSettingsPage (UI-11a / QR系 / D-4)", () => {
  it("ui11a validation empty input shows required FieldError and does not call updateSetting", async () => {
    const user = userEvent.setup();
    await renderReady();

    const generalInput = screen.getByLabelText("一般商品の基準（必須）");
    await user.clear(generalInput);
    await user.click(screen.getByRole("button", { name: "保存する" }));

    expect(await screen.findByText("1〜99999の整数を入力してください")).toBeInTheDocument();
    expect(mockUpdateSetting).not.toHaveBeenCalled();
  });

  it("ui11a validation decimal or non-numeric input shows integer FieldError and does not call updateSetting", async () => {
    const user = userEvent.setup();
    await renderReady();

    const generalInput = screen.getByLabelText("一般商品の基準（必須）");
    await user.clear(generalInput);
    await user.type(generalInput, "1.5");
    await user.click(screen.getByRole("button", { name: "保存する" }));

    expect(await screen.findByText("1〜99999の整数を入力してください")).toBeInTheDocument();
    expect(mockUpdateSetting).not.toHaveBeenCalled();
  });

  it("ui11a validation zero input shows integer FieldError and does not call updateSetting", async () => {
    const user = userEvent.setup();
    await renderReady();

    const generalInput = screen.getByLabelText("一般商品の基準（必須）");
    await user.clear(generalInput);
    await user.type(generalInput, "0");
    await user.click(screen.getByRole("button", { name: "保存する" }));

    expect(await screen.findByText("1〜99999の整数を入力してください")).toBeInTheDocument();
    expect(mockUpdateSetting).not.toHaveBeenCalled();
  });

  it("ui11a validation over-max input shows max FieldError and does not call updateSetting", async () => {
    const user = userEvent.setup();
    await renderReady();

    const generalInput = screen.getByLabelText("一般商品の基準（必須）");
    await user.clear(generalInput);
    await user.type(generalInput, "100000");
    await user.click(screen.getByRole("button", { name: "保存する" }));

    expect(await screen.findByText("1〜99999の整数を入力してください")).toBeInTheDocument();
    expect(mockUpdateSetting).not.toHaveBeenCalled();
  });

  it("ui11a save button is disabled when pristine and enabled after edit", async () => {
    const user = userEvent.setup();
    await renderReady();

    const saveButton = screen.getByRole("button", { name: "保存する" });
    expect(saveButton).toBeDisabled();

    const generalInput = screen.getByLabelText("一般商品の基準（必須）");
    await user.clear(generalInput);
    await user.type(generalInput, "5");

    expect(saveButton).toBeEnabled();
  });

  it("ui11a save calls updateSetting only for the dirty key when only one field is edited", async () => {
    const user = userEvent.setup();
    await renderReady();

    const generalInput = screen.getByLabelText("一般商品の基準（必須）");
    await user.clear(generalInput);
    await user.type(generalInput, "5");
    await user.click(screen.getByRole("button", { name: "保存する" }));

    await waitFor(() => {
      expect(mockUpdateSetting).toHaveBeenCalledTimes(1);
    });
    expect(mockUpdateSetting).toHaveBeenCalledWith({ key: "stock_low_threshold", value: "5" });
  });

  it("ui11a save sends the trimmed value when the input has surrounding whitespace", async () => {
    // BIZ 側 parse::<i64>() は前後空白を受けず既定値に fallback するため、
    // " 5 " は trim 済みの "5" で保存されなければならない（Spec Contract）。
    const user = userEvent.setup();
    await renderReady();

    const generalInput = screen.getByLabelText("一般商品の基準（必須）");
    await user.clear(generalInput);
    await user.type(generalInput, " 5 ");
    await user.click(screen.getByRole("button", { name: "保存する" }));

    await waitFor(() => {
      expect(mockUpdateSetting).toHaveBeenCalledTimes(1);
    });
    expect(mockUpdateSetting).toHaveBeenCalledWith({ key: "stock_low_threshold", value: "5" });
    expect(generalInput).toHaveValue("5");
  });

  it("ui11a save success shows toast with saved values and invalidates settings + low-stock queries", async () => {
    const user = userEvent.setup();
    const { invalidateSpy } = await renderReady();
    const { toast } = await import("sonner");

    const generalInput = screen.getByLabelText("一般商品の基準（必須）");
    const fabricInput = screen.getByLabelText("生地の基準（必須）");
    await user.clear(generalInput);
    await user.type(generalInput, "5");
    await user.clear(fabricInput);
    await user.type(fabricInput, "600");
    await user.click(screen.getByRole("button", { name: "保存する" }));

    await waitFor(() => {
      expect(mockUpdateSetting).toHaveBeenCalledTimes(2);
    });
    expect(mockUpdateSetting).toHaveBeenNthCalledWith(1, {
      key: "stock_low_threshold",
      value: "5",
    });
    expect(mockUpdateSetting).toHaveBeenNthCalledWith(2, {
      key: "stock_low_threshold_fabric",
      value: "600",
    });

    await waitFor(() => {
      expect(toast.success).toHaveBeenCalledWith(
        "在庫少の基準を保存しました（一般商品: 5個以下 / 生地: 600cm以下）",
        { id: "threshold-save-success" },
      );
    });

    expectExactInvalidations(invalidateSpy.mock.calls, d052InvalidationOracle.thresholdSave());

    // 保存成功後はフォームが pristine 化し、保存ボタンが再び disabled になる
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "保存する" })).toBeDisabled();
    });
  });

  it("ui11a partial save failure shows the failed field name and keeps the saved fact for the other field", async () => {
    const user = userEvent.setup();
    const { invalidateSpy } = await renderReady();
    mockUpdateSetting.mockResolvedValueOnce(ok(null));
    mockUpdateSetting.mockResolvedValueOnce(cmdError("保存できませんでした"));

    const generalInput = screen.getByLabelText("一般商品の基準（必須）");
    const fabricInput = screen.getByLabelText("生地の基準（必須）");
    await user.clear(generalInput);
    await user.type(generalInput, "5");
    await user.clear(fabricInput);
    await user.type(fabricInput, "600");
    await user.click(screen.getByRole("button", { name: "保存する" }));

    expect(
      await screen.findByText(
        "生地の基準の保存に失敗しました。一般商品の基準は保存済みです。もう一度保存してください",
      ),
    ).toBeInTheDocument();
    expect(mockUpdateSetting).toHaveBeenCalledTimes(2);
    await waitFor(() => {
      expectExactInvalidations(invalidateSpy.mock.calls, d052InvalidationOracle.thresholdSave());
    });
  });

  it("D-052-C13 does not invalidate when no threshold field was saved", async () => {
    const user = userEvent.setup();
    const { invalidateSpy } = await renderReady();
    mockUpdateSetting.mockResolvedValueOnce(cmdError("保存できませんでした"));

    const generalInput = screen.getByLabelText("一般商品の基準（必須）");
    const fabricInput = screen.getByLabelText("生地の基準（必須）");
    await user.clear(generalInput);
    await user.type(generalInput, "5");
    await user.clear(fabricInput);
    await user.type(fabricInput, "600");
    await user.click(screen.getByRole("button", { name: "保存する" }));

    await waitFor(() => {
      expect(mockUpdateSetting).toHaveBeenCalledTimes(1);
      expect(screen.getByRole("button", { name: "保存する" })).toBeEnabled();
    });
    expect(invalidateSpy).not.toHaveBeenCalled();
    expect(screen.getByText("保存に失敗しました。もう一度保存してください")).toBeInTheDocument();
  });

  it("ui11a getSettings failure shows a top alert with a retry action", async () => {
    mockGetSettings.mockReset();
    mockGetSettings.mockResolvedValue(cmdError("設定を取得できませんでした"));

    renderWithClient(<ThresholdSettingsPage />);

    expect(await screen.findByText("設定を読み込めませんでした")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "再試行" })).toBeInTheDocument();
    expect(screen.queryByLabelText("一般商品の基準（必須）")).not.toBeInTheDocument();
  });

  it("ui11a shows a blank field and recovery FieldError when the stored value is not numeric", async () => {
    mockGetSettings.mockReset();
    mockGetSettings.mockResolvedValue(
      ok([
        { key: "stock_low_threshold", value: "abc", updated_at: "2026-07-06T00:00:00" },
        { key: "stock_low_threshold_fabric", value: "500", updated_at: "2026-07-06T00:00:00" },
      ]),
    );

    renderWithClient(<ThresholdSettingsPage />);

    const generalInput = await screen.findByLabelText("一般商品の基準（必須）");
    expect(generalInput).toHaveValue("");
    expect(
      screen.getByText("現在の設定値が読み取れません。正しい値を入力して保存してください"),
    ).toBeInTheDocument();
  });

  it("ui11a wording shows the h1 and required field labels matching UI-11a-D6", async () => {
    await renderReady();

    // SPEC-DISP-B2-1 / D-B1 GA3: h1 を残し、区画は説明文のみとする。
    expect(screen.getByRole("heading", { level: 1, name: "在庫少の基準" })).toBeInTheDocument();
    expect(screen.queryByRole("heading", { level: 2 })).not.toBeInTheDocument();
    expect(
      screen.getByText("保存すると、ホームと在庫照会の在庫少の判定にすぐ反映されます"),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("一般商品の基準（必須）")).toBeInTheDocument();
    expect(screen.getByLabelText("生地の基準（必須）")).toBeInTheDocument();
    expect(
      screen.getByText("在庫がこの個数以下になったら在庫少（初期値: 3個）"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("在庫がこの長さ以下になったら在庫少（初期値: 500cm = 5m）"),
    ).toBeInTheDocument();
  });

  it("renders every threshold descriptor in save order", async () => {
    await renderReady();

    expect(
      screen
        .getAllByRole("textbox")
        .map((input) => input.getAttribute("aria-labelledby") ?? input.id),
    ).toEqual(["stock-low-threshold", "stock-low-threshold-fabric"]);
    expect(screen.getByLabelText("一般商品の基準（必須）")).toHaveValue("3");
    expect(screen.getByLabelText("生地の基準（必須）")).toHaveValue("500");
  });
});
