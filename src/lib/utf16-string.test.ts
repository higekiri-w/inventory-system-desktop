import { describe, expect, it } from "vitest";

import { utf16String, type Utf16StringBounds } from "./utf16-string";

// "\u{2000B}" は BMP の外の 1 文字（1 code point・2 UTF-16 code unit）。
const astral = "\u{2000B}";

const cases: { bounds: Utf16StringBounds; accepted: string[]; rejected: string[] }[] = [
  {
    bounds: { max: 100 },
    accepted: ["A".repeat(100), astral.repeat(50), "あ".repeat(100)],
    rejected: ["A".repeat(101), astral.repeat(51)],
  },
  { bounds: { min: 1 }, accepted: ["A", astral, "A".repeat(10000)], rejected: [""] },
  { bounds: { min: 2 }, accepted: [astral, "AA"], rejected: ["A", ""] },
  {
    bounds: { min: 1, max: 100 },
    accepted: ["A", astral.repeat(50)],
    rejected: ["", astral.repeat(51)],
  },
  { bounds: { max: 500 }, accepted: ["", astral.repeat(250)], rejected: [astral.repeat(251)] },
];

describe("utf16String (REQ-301 / BIZ-01-D5 / D-151)", () => {
  describe.each(cases)(
    "REQ-301 / BIZ-01-D5: utf16String は UTF-16 code unit で長さを数える $bounds",
    ({ bounds, accepted, rejected }) => {
      const schema = utf16String(bounds).optional().catch(undefined);

      it.each(accepted.map((value) => [value.length, value] as const))(
        "受理する（%i unit）",
        (_, value) => {
          expect(schema.parse(value)).toBe(value);
        },
      );

      it.each(rejected.map((value) => [value.length, value] as const))(
        "拒否して undefined にする（%i unit）",
        (_, value) => {
          expect(schema.parse(value)).toBeUndefined();
        },
      );
    },
  );

  it("REQ-301 / BIZ-01-D5: 文字列でない値と値の欠落は undefined になる", () => {
    const schema = utf16String({ max: 100 }).optional().catch(undefined);
    expect(schema.parse(undefined)).toBeUndefined();
    expect(schema.parse(123)).toBeUndefined();
  });
});
