import { describe, expect, it } from "vitest";
import type { EjRecordKindCode, EjRecordModeCode } from "@/lib/bindings";
import { ejKindLabel, ejTime, formatEjAmount } from "./receipt-format";

describe("UI-09c-D2 receipt format REQ-908", () => {
  it.each<[EjRecordKindCode, EjRecordModeCode, string]>([
    ["sale", "normal", "取引"],
    ["sale", "return", "返品"],
    ["cancelled", "normal", "取引中止"],
    ["cancelled", "return", "取引中止"],
    ["cash_movement", "normal", "入金・出金"],
    ["receipt", "normal", "領収書"],
    ["settlement", "settlement", "精算"],
    ["inspection", "inspection", "点検"],
    ["settings", "program", "設定"],
    ["training", "training", "練習"],
    ["unclassified", "unrecognized", "その他"],
  ])("U5: %s × %s は「%s」", (kind, mode, label) => {
    expect(ejKindLabel(kind, mode)).toBe(label);
  });

  it("U5: 合計は null を「—」、0 を「0円」、1234 を「1,234円」にする", () => {
    expect(formatEjAmount(null)).toBe("—");
    expect(formatEjAmount(0)).toBe("0円");
    expect(formatEjAmount(1234)).toBe("1,234円");
    expect(formatEjAmount(-500)).toBe("-500円");
  });

  it("U3: 時刻は HH:MM", () => {
    expect(ejTime("2026-11-20 09:05")).toBe("09:05");
  });
});
