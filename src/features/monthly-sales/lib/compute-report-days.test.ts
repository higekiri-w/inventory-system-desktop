// src/features/monthly-sales/lib/compute-report-days.test.ts
//
// REQ-502 / UI-09b-D11: 取り込んだ日と取り込んでいない日（57 §57.6 compute-report-days）。
// ケースの正本: docs/plans/2026-11-03-monthly-imported-days.md Test Plan の U1〜U6

import { describe, expect, it } from "vitest";

import { computeReportDays } from "./compute-report-days";

const dates = (days: { date: string }[]) => days.map((day) => day.date);
const range = (month: string, from: number, to: number) =>
  Array.from({ length: to - from + 1 }, (_, i) => `${month}-${String(from + i).padStart(2, "0")}`);

describe("computeReportDays (REQ-502 UI-09b-D11)", () => {
  it("U1: 過去の月は暦の補集合が取り込んでいない日になり、isCurrentMonth は false", () => {
    const result = computeReportDays("2026-03", ["2026-03-23", "2026-03-24"], "2026-11-03");

    expect(dates(result.imported)).toEqual(["2026-03-23", "2026-03-24"]);
    expect(dates(result.notImported)).toEqual([
      ...range("2026-03", 1, 22),
      ...range("2026-03", 25, 31),
    ]);
    expect(result.notImported).toHaveLength(29);
    expect(result.isCurrentMonth).toBe(false);
  });

  it("U2: 今月の取り込んでいない日は前日までで、今日と今日より後の日を入れない", () => {
    const result = computeReportDays("2026-11", ["2026-11-01"], "2026-11-03");

    expect(dates(result.imported)).toEqual(["2026-11-01"]);
    expect(dates(result.notImported)).toEqual(["2026-11-02"]);
    expect(result.isCurrentMonth).toBe(true);
  });

  it("U2b: 今日を取り込めば今日は取り込んだ日に並ぶ（今日での切り捨ては取り込んでいない日だけ）", () => {
    const result = computeReportDays("2026-11", ["2026-11-01", "2026-11-03"], "2026-11-03");

    expect(dates(result.imported)).toEqual(["2026-11-01", "2026-11-03"]);
    expect(dates(result.notImported)).toEqual(["2026-11-02"]);
  });

  it("U3: 来月より先の月は両方空で、isCurrentMonth は false", () => {
    const result = computeReportDays("2026-12", [], "2026-11-03");

    expect(result.imported).toEqual([]);
    expect(result.notImported).toEqual([]);
    expect(result.isCurrentMonth).toBe(false);
  });

  it("U4: 月の長さ（閏年の 2 月 29 日、平年 28 日、4 月 30 日）", () => {
    const today = "2030-01-01";
    expect(computeReportDays("2028-02", [], today).notImported).toHaveLength(29);
    expect(computeReportDays("2026-02", [], today).notImported).toHaveLength(28);
    expect(computeReportDays("2026-04", [], today).notImported).toHaveLength(30);
  });

  it("U5: 曜日は local の暦で引く（23日（月）の形）", () => {
    const result = computeReportDays("2026-03", ["2026-03-21", "2026-03-23"], "2026-11-03");

    expect(result.imported.map((day) => day.label)).toEqual(["21日（土）", "23日（月）"]);
    expect(result.notImported[0]).toEqual({ date: "2026-03-01", label: "1日（日）" });
  });

  it("U6: 不正な月は空の結果で throw せず、月の外の日は取り込んだ日に入れない", () => {
    expect(computeReportDays("2026-13", ["2026-13-01"], "2026-11-03")).toEqual({
      imported: [],
      notImported: [],
      isCurrentMonth: false,
    });
    expect(computeReportDays("2026-03", ["2026-02-28"], "2026-11-03").imported).toEqual([]);
  });
});
