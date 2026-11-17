import { z } from "zod";
import { utf16String } from "@/lib/utf16-string";

export const INVENTORY_RECORD_TYPE_OPTIONS = [
  { value: "all", label: "すべて" },
  { value: "receiving_record", label: "入庫" },
  { value: "return_record", label: "返品・交換" },
  { value: "manual_sale", label: "手動販売出庫" },
  { value: "disposal_record", label: "廃棄・破損" },
  { value: "transfer_record", label: "詰め替え・切り分け" },
  { value: "csv_import", label: "CSV取込み" },
  { value: "stocktake", label: "棚卸し" },
] as const;
export const INVENTORY_RECORD_STATUS_OPTIONS = [
  { value: "all", label: "すべて" },
  { value: "active", label: "有効" },
  { value: "canceled", label: "取消済み" },
  { value: "in_progress", label: "進行中" },
] as const;

export type InventoryRecordType = (typeof INVENTORY_RECORD_TYPE_OPTIONS)[number]["value"];
export type InventoryRecordStatus = (typeof INVENTORY_RECORD_STATUS_OPTIONS)[number]["value"];

function descriptorValues<
  const T extends readonly [{ readonly value: string }, ...{ readonly value: string }[]],
>(descriptors: T): { [K in keyof T]: T[K]["value"] } {
  return descriptors.map(({ value }) => value) as { [K in keyof T]: T[K]["value"] };
}

const INVENTORY_RECORD_TYPES = descriptorValues(INVENTORY_RECORD_TYPE_OPTIONS);
const INVENTORY_RECORD_STATUSES = descriptorValues(INVENTORY_RECORD_STATUS_OPTIONS);

export const inventoryRecordsSearchSchema = z.object({
  recordType: z.enum(INVENTORY_RECORD_TYPES).optional().catch(undefined),
  dateFrom: z
    .string()
    .regex(/^\d{4}-\d{2}-\d{2}$/)
    .optional()
    .catch(undefined),
  dateTo: z
    .string()
    .regex(/^\d{4}-\d{2}-\d{2}$/)
    .optional()
    .catch(undefined),
  q: utf16String({ max: 100 }).optional().catch(undefined),
  recordId: z.coerce.number().int().positive().optional().catch(undefined),
  departmentId: z.coerce.number().int().positive().optional().catch(undefined),
  status: z.enum(INVENTORY_RECORD_STATUSES).optional().catch(undefined),
  page: z.coerce.number().int().positive().optional().catch(undefined),
});

export type InventoryRecordsSearch = z.output<typeof inventoryRecordsSearchSchema>;

const validRecordTypes = new Set<InventoryRecordType>(INVENTORY_RECORD_TYPES);
const validStatuses = new Set<InventoryRecordStatus>(INVENTORY_RECORD_STATUSES);

export function normalizeInventoryRecordsSearch(
  search: InventoryRecordsSearch,
): Required<Pick<InventoryRecordsSearch, "recordType" | "status" | "page">> &
  Pick<InventoryRecordsSearch, "dateFrom" | "dateTo" | "q" | "recordId" | "departmentId"> {
  const recordId =
    Number.isInteger(search.recordId) && search.recordId !== undefined && search.recordId > 0
      ? search.recordId
      : undefined;
  const departmentId =
    Number.isInteger(search.departmentId) &&
    search.departmentId !== undefined &&
    search.departmentId > 0
      ? search.departmentId
      : undefined;
  const page =
    Number.isInteger(search.page) && search.page !== undefined && search.page > 0 ? search.page : 1;

  return {
    recordType:
      search.recordType !== undefined && validRecordTypes.has(search.recordType)
        ? search.recordType
        : "all",
    status: search.status !== undefined && validStatuses.has(search.status) ? search.status : "all",
    dateFrom: search.dateFrom?.match(/^\d{4}-\d{2}-\d{2}$/) ? search.dateFrom : undefined,
    dateTo: search.dateTo?.match(/^\d{4}-\d{2}-\d{2}$/) ? search.dateTo : undefined,
    q: search.q?.trim() ? search.q.trim() : undefined,
    recordId,
    departmentId,
    page,
  };
}

export function formatRecordStatus(status: string): string {
  const searchStatus = INVENTORY_RECORD_STATUS_OPTIONS.find(
    ({ value }) => value !== "all" && value === status,
  );
  if (searchStatus !== undefined) return searchStatus.label;
  if (status === "canceled") return "取消済み";
  if (status === "corrected") return "訂正済み";
  return status;
}

export function formatRecordType(recordType: string): string {
  return (
    INVENTORY_RECORD_TYPE_OPTIONS.find(({ value }) => value !== "all" && value === recordType)
      ?.label ?? recordType
  );
}

export function formatYen(value: number): string {
  return `¥${value.toLocaleString("ja-JP")}`;
}
