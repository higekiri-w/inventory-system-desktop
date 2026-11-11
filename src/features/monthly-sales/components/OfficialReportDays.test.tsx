// src/features/monthly-sales/components/OfficialReportDays.test.tsx
//
// REQ-502 / UI-09b-D11: 取り込んだ日と取り込んでいない日の表示（C1〜C3）。
// ケースの正本: docs/plans/2026-11-03-monthly-imported-days.md Test Plan

import { describe, expect, it } from "vitest";
import { screen, within } from "@testing-library/react";

import { renderWithRouter } from "@/test/render-with-router";

import { computeReportDays } from "../lib/compute-report-days";
import { OfficialReportDays } from "./OfficialReportDays";

const CURRENT_MONTH_NOTE = "今日と、今日より後の日は数えていません。";

function renderDays(month: string, importedDates: string[], today: string) {
  return renderWithRouter(
    <OfficialReportDays days={computeReportDays(month, importedDates, today)} />,
  );
}

describe("OfficialReportDays (REQ-502 UI-09b-D11)", () => {
  it("C1: 見出しの数・取り込んだ日の link・取り込んでいない日・休業日の文・直し方の link", async () => {
    renderDays("2026-03", ["2026-03-23", "2026-03-24"], "2026-11-03");

    expect(
      await screen.findByRole("heading", { name: "取り込んだ日（2日分）" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "取り込んでいない日（29日分）" }),
    ).toBeInTheDocument();

    const importedList = screen.getByRole("list", { name: "取り込んだ日" });
    const importedItems = within(importedList).getAllByRole("listitem");
    expect(importedItems.map((item) => item.textContent)).toEqual(["23日（月）、", "24日（火）"]);
    // 見出しと日の一覧は本文の大きさ、label の中では折り返さない（04 原則 1、L3-2）
    expect(importedList.closest(".text-sm")).toBeNull();
    expect(importedItems[0]).toHaveClass("whitespace-nowrap");
    expect(within(importedList).getByRole("link", { name: "23日（月）" })).toHaveAttribute(
      "href",
      "/reports/daily?date=2026-03-23",
    );

    const notImportedList = screen.getByRole("list", { name: "取り込んでいない日" });
    const notImportedItems = within(notImportedList).getAllByRole("listitem");
    expect(notImportedItems).toHaveLength(29);
    expect(notImportedItems[0]).toHaveTextContent("1日（日）");
    expect(within(notImportedList).queryByRole("link")).toBeNull();

    expect(screen.getByText("店を開けなかった日も含みます。")).toHaveClass(
      "text-sm",
      "text-muted-foreground",
    );
    expect(screen.getByRole("link", { name: "売上データ取込みを開く" })).toHaveAttribute(
      "href",
      "/csv-import?tab=daily-report",
    );
    expect(screen.queryByText(CURRENT_MONTH_NOTE)).toBeNull();
  });

  it("C2: 全部の日を取り込んだ月は「なし」で、直し方の link を出さない", async () => {
    const all = Array.from({ length: 28 }, (_, i) => `2026-02-${String(i + 1).padStart(2, "0")}`);
    renderDays("2026-02", all, "2026-11-03");

    expect(
      await screen.findByRole("heading", { name: "取り込んでいない日（0日分）" }),
    ).toBeInTheDocument();
    expect(screen.getByText("なし")).toBeInTheDocument();
    expect(screen.queryByRole("list", { name: "取り込んでいない日" })).toBeNull();
    expect(screen.queryByRole("link", { name: "売上データ取込みを開く" })).toBeNull();
  });

  it("C3: 今月だけ今日を数えない文を出し、取り込んでいない日は前日まで", async () => {
    const { unmount } = renderDays("2026-11", ["2026-11-01"], "2026-11-03");

    expect(await screen.findByText(CURRENT_MONTH_NOTE)).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "取り込んでいない日（1日分）" }),
    ).toBeInTheDocument();
    const notImportedList = screen.getByRole("list", { name: "取り込んでいない日" });
    expect(
      within(notImportedList)
        .getAllByRole("listitem")
        .map((item) => item.textContent),
    ).toEqual(["2日（月）"]);
    unmount();

    const past = renderDays("2026-03", ["2026-03-23"], "2026-11-03");
    expect(
      await screen.findByRole("heading", { name: "取り込んだ日（1日分）" }),
    ).toBeInTheDocument();
    expect(screen.queryByText(CURRENT_MONTH_NOTE)).toBeNull();
    past.unmount();

    renderDays("2026-12", [], "2026-11-03");
    expect(
      await screen.findByRole("heading", { name: "取り込んでいない日（0日分）" }),
    ).toBeInTheDocument();
    expect(screen.queryByText(CURRENT_MONTH_NOTE)).toBeNull();
  });
});
