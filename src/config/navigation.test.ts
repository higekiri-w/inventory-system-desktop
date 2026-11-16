import { describe, expect, it } from "vitest";
import { navigation } from "./navigation";

describe("navigation config REQ-902", () => {
  it("marks ui-11c active at /settings/logs", () => {
    const item = navigation.flatMap((area) => area.items).find((entry) => entry.id === "ui-11c");
    expect(item).toMatchObject({ status: "active", to: "/settings/logs" });
  });
});

describe("navigation config REQ-904", () => {
  it("test_navigation_req904_ui13_active_at_settings_integrity", () => {
    // REQ-904: 在庫整合性検証画面への operator 到達導線
    const item = navigation.flatMap((area) => area.items).find((entry) => entry.id === "ui-13");
    expect(item).toMatchObject({ status: "active", to: "/settings/integrity" });
  });
});

describe("navigation config REQ-105 / UI-14", () => {
  it("test_navigation_req105_ui14_active_at_products_price_revision", () => {
    const item = navigation.flatMap((area) => area.items).find((entry) => entry.id === "ui-14");
    expect(item).toMatchObject({
      label: "一括価格改定",
      title: "一括価格改定",
      status: "active",
      to: "/products/price-revision",
    });
  });
});

describe("navigation config REQ-107 / UI-15", () => {
  it("test_navigation_req107_ui15_active_at_settings_suppliers", () => {
    const systemArea = navigation.find((area) => area.id === "system");
    const item = systemArea?.items.find((entry) => entry.id === "ui-15");
    expect(item).toMatchObject({
      label: "取引先管理",
      title: "取引先管理",
      status: "active",
      to: "/settings/suppliers",
    });
    expect(systemArea?.items[systemArea.items.length - 1]?.id).toBe("ui-15");
  });
});

describe("navigation config REQ-210 / UI-16", () => {
  it("test_navigation_req210_ui16_active_at_inventory_transfer_after_disposal", () => {
    // U1 / C-NAV（79 UI-16-D1・§79.7）: 入出庫の「廃棄・破損」の次に「詰め替え・切り分け」
    const inventoryArea = navigation.find((area) => area.id === "inventory");
    const ids = inventoryArea?.items.map((entry) => entry.id) ?? [];
    expect(ids.indexOf("ui-16")).toBe(ids.indexOf("ui-05") + 1);
    expect(inventoryArea?.items.find((entry) => entry.id === "ui-16")).toMatchObject({
      label: "詰め替え・切り分け",
      title: "詰め替え・切り分け",
      description: "仕入れた品を別の商品にした数を移します",
      status: "active",
      to: "/inventory/transfer",
    });
  });
});

describe("navigation config sidebar pending links", () => {
  it("test_navigation_req101_ui01b_active_at_products_new", () => {
    // REQ-101: 商品登録画面への operator 到達導線
    const item = navigation
      .flatMap((area) => area.items)
      .find((entry) => entry.id === "ui-01b-new");
    expect(item).toMatchObject({ status: "active", to: "/products/new" });
  });

  it("test_navigation_req302_ui06b_active_deep_link_low_stock", () => {
    // REQ-302: 在庫少一覧は UI-06a の low_stock filter へ deep-link する
    const items = navigation.flatMap((area) => area.items);
    expect(items.find((entry) => entry.id === "ui-06b")).toMatchObject({
      status: "active",
      to: "/stock",
      search: { status: "low_stock" },
      activeMatch: { searchKey: "status", is: "low_stock" },
    });
    expect(items.find((entry) => entry.id === "ui-06a")).toMatchObject({
      activeMatch: { searchKey: "status", isNot: "low_stock" },
    });
  });

  it("test_navigation_all_items_no_pending_status", () => {
    expect(navigation.flatMap((area) => area.items)).not.toContainEqual(
      expect.objectContaining({ status: "pending" }),
    );
  });
});
