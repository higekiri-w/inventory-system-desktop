// REQ-301 / BIZ-01-D5 / D-151: frontend の文字列の長さの検査は UTF-16 code unit で数える。
// zod 4.5 以降の `.max()`・`.min()`・`.length()` は境界の近くで code point で数え直すので使わない。
import { z } from "zod";

export type Utf16StringBounds = { min: number; max?: number } | { min?: number; max: number };

/** `String.prototype.length`（UTF-16 code unit）で長さを検査する zod の文字列の schema。 */
export function utf16String({ min, max }: Utf16StringBounds) {
  return z
    .string()
    .refine(
      (value) =>
        (min === undefined || value.length >= min) && (max === undefined || value.length <= max),
      { error: "文字列の長さが範囲の外です" },
    );
}
