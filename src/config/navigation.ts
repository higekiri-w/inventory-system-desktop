import {
  AlertTriangle,
  ArrowLeftRight,
  BarChart3,
  BarChartBig,
  Building2,
  ClipboardList,
  CircleDollarSign,
  DatabaseBackup,
  FileDown,
  FileSpreadsheet,
  FileUp,
  Hand,
  Home,
  Package,
  PackagePlus,
  PackageSearch,
  RotateCcw,
  ScrollText,
  Scissors,
  Search,
  ShieldCheck,
  SlidersHorizontal,
  Sun,
  Trash2,
  Wrench,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { LOW_STOCK_FILTER } from "@/features/stock-inquiry/types";

/// UI-12 共通レイアウトのナビゲーション定義。
/// 設計: docs/function-design/52-ui-shared-layout.md §52.3 / §52.4
/// 設計合意: docs/archive/plans/2026-04-21-ui-12-design-agreement.md §2.1 / §4.1 / §4.2

export type NavStatus = "active" | "pending";

export interface NavItem {
  id: string;
  label: string;
  title: string;
  to: string | null;
  search?: Record<string, string>;
  activeMatch?: { searchKey: string; is?: string; isNot?: string };
  description?: string; // ホーム入口 card の 1 行説明（UI-00）。sidebar は読まない
  icon: LucideIcon;
  status: NavStatus;
}

export interface NavArea {
  id: "daily" | "products" | "inventory" | "system";
  label: string;
  icon: LucideIcon;
  items: readonly NavItem[];
}

/// 4 エリア × 22 項目。全画面は route 実装済みで active。
export const navigation: readonly NavArea[] = [
  {
    id: "daily",
    label: "毎日の業務",
    icon: Sun,
    items: [
      {
        id: "ui-00",
        label: "ホーム",
        title: "ホーム",
        to: "/",
        icon: Home,
        status: "active",
      },
      {
        id: "ui-07",
        description: "レジの日報CSVを読み込み、当日の売上を記録します",
        label: "売上データ取込み",
        title: "売上データ取込み",
        to: "/csv-import",
        icon: FileUp,
        status: "active",
      },
      {
        id: "ui-09a",
        description: "今日・昨日の売上明細と集計を確認します",
        label: "日次売上",
        title: "日次売上",
        to: "/reports/daily",
        icon: BarChart3,
        status: "active",
      },
      {
        id: "ui-06a",
        description: "商品の在庫数・在庫切れ / 在庫少を調べます",
        label: "在庫照会",
        title: "在庫照会",
        to: "/stock",
        activeMatch: { searchKey: "status", isNot: LOW_STOCK_FILTER },
        icon: Search,
        status: "active",
      },
      {
        id: "ui-09b",
        label: "月次売上",
        title: "月次売上",
        to: "/reports/monthly",
        icon: BarChartBig,
        status: "active",
      },
    ],
  },
  {
    id: "products",
    label: "商品管理",
    icon: Package,
    items: [
      {
        id: "ui-01a",
        description: "登録済み商品を探す・売価や在庫を確認します",
        label: "商品検索・一覧",
        title: "商品検索・一覧",
        to: "/products",
        icon: PackageSearch,
        status: "active",
      },
      {
        id: "ui-01b-new",
        label: "商品登録",
        title: "商品登録",
        to: "/products/new",
        icon: PackagePlus,
        status: "active",
      },
      {
        id: "ui-01c",
        label: "一括インポート",
        title: "一括インポート",
        to: "/products/import",
        icon: FileSpreadsheet,
        status: "active",
      },
      {
        id: "ui-08",
        label: "PLU書出し",
        title: "PLU書出し",
        to: "/products/plu-export",
        icon: FileDown,
        status: "active",
      },
      {
        id: "ui-14",
        label: "一括価格改定",
        title: "一括価格改定",
        to: "/products/price-revision",
        icon: CircleDollarSign,
        status: "active",
      },
    ],
  },
  {
    id: "inventory",
    label: "入出庫",
    icon: ArrowLeftRight,
    items: [
      {
        id: "ui-02",
        description: "仕入れた商品が届いたときに記録します",
        label: "入庫記録",
        title: "入庫記録",
        to: "/inventory/receiving",
        icon: PackagePlus,
        status: "active",
      },
      {
        id: "ui-03",
        description: "お客様からの返品・交換を記録します",
        label: "返品・交換",
        title: "返品・交換",
        to: "/inventory/return",
        icon: RotateCcw,
        status: "active",
      },
      {
        id: "ui-04",
        description: "レジを通さず売った商品の在庫を減らします",
        label: "手動販売出庫",
        title: "手動販売出庫",
        to: "/inventory/manual-sale",
        icon: Hand,
        status: "active",
      },
      {
        id: "ui-05",
        description: "傷んだ・破損した商品を在庫から除きます",
        label: "廃棄・破損",
        title: "廃棄・破損",
        to: "/inventory/disposal",
        icon: Trash2,
        status: "active",
      },
      {
        id: "ui-16",
        description: "仕入れた品を別の商品にした数を移します",
        label: "詰め替え・切り分け",
        title: "詰め替え・切り分け",
        to: "/inventory/transfer",
        icon: Scissors,
        status: "active",
      },
      {
        id: "ui-02b-05b",
        label: "入出庫履歴",
        title: "入出庫履歴",
        to: "/inventory/records",
        icon: ScrollText,
        status: "active",
      },
      {
        id: "ui-06b",
        label: "在庫少一覧",
        title: "在庫少一覧",
        to: "/stock",
        search: { status: LOW_STOCK_FILTER },
        activeMatch: { searchKey: "status", is: LOW_STOCK_FILTER },
        icon: AlertTriangle,
        status: "active",
      },
      {
        id: "ui-10",
        description: "実在庫を数えてシステム在庫と突き合わせます",
        label: "棚卸し",
        title: "棚卸し",
        to: "/stocktake",
        icon: ClipboardList,
        status: "active",
      },
    ],
  },
  {
    id: "system",
    label: "システム管理",
    icon: Wrench,
    items: [
      {
        id: "ui-11b",
        description: "データの控えを取る・控えから戻します",
        label: "バックアップ・復元",
        title: "バックアップ・復元",
        to: "/settings/backup",
        icon: DatabaseBackup,
        status: "active",
      },
      {
        id: "ui-11c",
        label: "操作ログ",
        title: "操作ログ",
        to: "/settings/logs",
        icon: ScrollText,
        status: "active",
      },
      {
        id: "ui-11a",
        description: "「在庫少」と判定する数量の基準を設定します",
        label: "在庫少の基準",
        title: "在庫少の基準",
        to: "/settings/thresholds",
        icon: SlidersHorizontal,
        status: "active",
      },
      {
        id: "ui-13",
        label: "整合性検証",
        title: "整合性検証",
        to: "/settings/integrity",
        icon: ShieldCheck,
        status: "active",
      },
      {
        id: "ui-15",
        label: "取引先管理",
        title: "取引先管理",
        to: "/settings/suppliers",
        icon: Building2,
        status: "active",
      },
    ],
  },
] as const;
