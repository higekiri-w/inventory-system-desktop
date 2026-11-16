import type {
  ProductStockUnit,
  ProductWithRelations,
  ReturnDirection,
  ReturnExchangeType,
} from "@/lib/bindings";
import type { PickedFile } from "@/components/FilePicker";

export type { ReturnDirection, ReturnExchangeType };

export interface ReturnExchangeRow {
  productCode: string;
  productName: string;
  departmentName: string;
  stockUnit: ProductStockUnit;
  currentStockQuantity: number;
  direction: ReturnDirection;
  quantity: string;
}

export type ReturnExchangeFormErrors = Partial<{
  returnDate: string;
  returnType: string;
  registerProcessed: string;
  items: string;
  receipt: string;
  rows: Record<string, string>;
}>;

export interface ReturnExchangeFormValues {
  returnDate: string;
  returnType: ReturnExchangeType;
  // UI-03-D23: null は未選択（既定を置かない）。true = レジでバーコードを読んで戻した
  registerProcessed: boolean | null;
  note: string;
  rows: ReturnExchangeRow[];
}

export interface ReceiptImageState {
  file: PickedFile;
  previewUrl: string;
  extension: string;
  savedReceiptPath: string | null;
}

export type ProductCandidate = ProductWithRelations;
