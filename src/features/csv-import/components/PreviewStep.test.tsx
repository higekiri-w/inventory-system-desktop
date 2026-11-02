import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import dateTimeSource from "./AdditionalImportConfirmDialog.tsx?raw";
import type { PreviewData } from "@/lib/bindings";
import { PreviewStep } from "./PreviewStep";

const preview: PreviewData = {
  file_info: {
    filename: "Z004_0104.CSV",
    settlement_date: "2026-03-21",
    file_hash: "a".repeat(64),
  },
  matched_summary: { count: 2, total_amount: 900, warnings: [] },
  error_summary: { count: 0, items: [] },
  settlement_gaps: [],
  settlement_date_after_today: false,
  preview_created_at: "2026-03-21T10:00:00",
};

describe("PreviewStep REQ-401 Z004（55 の D-118 の節）", () => {
  it("U10 z004 preview shows notices without additional confirm", async () => {
    // 知らせ（欠けた精算・日付のずれ）は取込みを止めない。同じ日の追加の確認の dialog を出さない（ADR D4）
    const user = userEvent.setup();
    const onConfirm = vi.fn();
    render(
      <PreviewStep
        preview={{
          ...preview,
          settlement_gaps: [
            { from: 103, to: 103 },
            { from: 105, to: 199 },
          ],
          settlement_date_after_today: true,
        }}
        filename="Z004_0200.CSV"
        onConfirm={onConfirm}
        onReselect={vi.fn()}
        isImporting={false}
      />,
    );

    const notice = screen.getByRole("alert", { name: "取込みの前の知らせ" });
    expect(notice).toHaveAttribute("data-variant", "warning");
    expect(notice.querySelector('svg[aria-hidden="true"]')).not.toBeNull();
    expect(
      within(notice)
        .getAllByRole("listitem")
        .map((item) => item.textContent),
    ).toEqual([
      "精算 103 が取り込まれていません",
      "精算 105〜199 が取り込まれていません",
      "レジと PC の日付がずれている可能性があります",
    ]);
    const importButton = screen.getByRole("button", { name: "取り込む" });
    expect(importButton).toBeEnabled();
    await user.click(importButton);
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    expect(screen.queryByText("同じ日の取込みがあります")).not.toBeInTheDocument();
    expect(onConfirm).toHaveBeenCalledTimes(1);
    expect(onConfirm).toHaveBeenCalledWith();
  });

  it("U10 z004 preview without notices has no notice block", () => {
    render(
      <PreviewStep
        preview={preview}
        filename="Z004_0104.CSV"
        onConfirm={vi.fn()}
        onReselect={vi.fn()}
        isImporting={false}
      />,
    );
    expect(screen.queryByRole("alert", { name: "取込みの前の知らせ" })).not.toBeInTheDocument();
    expect(screen.getByText("紐付け結果")).toBeInTheDocument();
  });

  it("REQ-401: importing 中は取り込むを押せない", async () => {
    const user = userEvent.setup();
    const onConfirm = vi.fn();
    render(
      <PreviewStep
        preview={preview}
        filename="Z004_0104.CSV"
        onConfirm={onConfirm}
        onReselect={vi.fn()}
        isImporting
      />,
    );
    const importButton = screen.getByRole("button", { name: "取り込む" });
    expect(importButton).toBeDisabled();
    await user.click(importButton);
    expect(onConfirm).not.toHaveBeenCalled();
  });
});

// AdditionalImportConfirmDialog は日報取込みタブが使い続ける（55・packet の「不要にならないと確かめたもの」）
it("⑰ SC6 / UIDISP-D6: 共有 formatDateTime を import しローカル定義を持たない", () => {
  expect(dateTimeSource).toMatch(
    /import\s*\{[^}]*\bformatDateTime\b[^}]*\}\s*from\s*"@\/lib\/date-time"/,
  );
  expect(dateTimeSource).not.toMatch(/function\s+(?:formatDateTime|formatCheckedAt)\s*\(/);
  expect(dateTimeSource).not.toContain("formatCheckedAt");
});
