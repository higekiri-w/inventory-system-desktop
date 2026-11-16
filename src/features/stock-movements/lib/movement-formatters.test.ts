import { describe, expect, it } from "vitest";

import { formatMovementQuantity, formatMovementType } from "./movement-formatters";

describe("movement-formatters (REQ-303)", () => {
  it("REQ-303: formatMovementQuantity returns sign and Japanese label", () => {
    expect(formatMovementQuantity(5)).toEqual({ value: "+5", label: "増加" });
    expect(formatMovementQuantity(-3)).toEqual({ value: "-3", label: "減少" });
    expect(formatMovementQuantity(0)).toEqual({ value: "0", label: "変動なし" });
  });

  it("REQ-303: formatMovementType maps generated movement types", () => {
    expect(formatMovementType("receiving")).toBe("入庫");
    expect(formatMovementType("sale_auto")).toBe("POS売上");
  });

  it("U13 / 66・65 TRACE-D14: 振替の movement は「詰め替え・切り分け」", () => {
    expect(formatMovementType("transfer")).toBe("詰め替え・切り分け");
  });
});
