// src/features/products/lib/return-to.test.ts

import { describe, expect, it } from "vitest";

import {
  buildProductListReturnTo,
  parseProductListSearchFromReturnTo,
  sanitizeProductListReturnTo,
} from "./return-to";

describe("sanitizeProductListReturnTo (UI-01b-D2)", () => {
  it("allows only product list route with search params", () => {
    expect(sanitizeProductListReturnTo("/products?q=布&page=2")).toBe(
      "/products?q=%E5%B8%83&page=2",
    );
    expect(sanitizeProductListReturnTo("/products/")).toBe("/products");
  });

  it("rejects product form/import, external URL, and unrelated routes", () => {
    expect(sanitizeProductListReturnTo("/products/new")).toBe("/products");
    expect(sanitizeProductListReturnTo("/products/ABC/edit")).toBe("/products");
    expect(sanitizeProductListReturnTo("/products/import")).toBe("/products");
    expect(sanitizeProductListReturnTo("https://example.com/products?q=布")).toBe("/products");
    expect(sanitizeProductListReturnTo("/reports/daily")).toBe("/products");
  });

  it("round-trips product list search params for navigation", () => {
    const returnTo = buildProductListReturnTo({
      q: "はさみ",
      dept: 2,
      discontinued: "all",
      sort: "selling_price",
      dir: "desc",
      page: 3,
      perPage: 100,
    });

    expect(returnTo).toBe(
      "/products?q=%E3%81%AF%E3%81%95%E3%81%BF&dept=2&discontinued=all&sort=selling_price&dir=desc&page=3&perPage=100",
    );
    expect(parseProductListSearchFromReturnTo(returnTo)).toEqual({
      q: "はさみ",
      dept: 2,
      discontinued: "all",
      sort: "selling_price",
      dir: "desc",
      page: 3,
      perPage: 100,
    });
  });

  it("round-trips all seven product list search params", () => {
    const returnTo = buildProductListReturnTo({
      q: "リボン",
      dept: 3,
      discontinued: "all",
      sort: "name",
      dir: "desc",
      page: 4,
      perPage: 200,
    });

    expect(parseProductListSearchFromReturnTo(returnTo)).toEqual({
      q: "リボン",
      dept: 3,
      discontinued: "all",
      sort: "name",
      dir: "desc",
      page: 4,
      perPage: 200,
    });
  });

  it("REQ-907 / UI-01a-D10〜D12（撤去）: 戻り先に残った plu を無視し、運ばない", () => {
    const parsed = parseProductListSearchFromReturnTo("/products?q=%E5%B8%83&plu=pending&page=2");
    expect(parsed).toEqual({
      q: "布",
      dept: undefined,
      discontinued: undefined,
      sort: undefined,
      dir: undefined,
      page: 2,
      perPage: undefined,
    });
    expect(buildProductListReturnTo(parsed)).toBe("/products?q=%E5%B8%83&page=2");
  });
});
