// src/features/return-exchange/ReturnExchangePage.tsx
//
// UI-03 返品・交換 page。設計: docs/function-design/63-ui-return-exchange.md

import { useEffect, useRef, useState } from "react";
import { Link, useRouterState } from "@tanstack/react-router";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowLeft, Eye, ImagePlus, RotateCcw, Search, Trash2 } from "lucide-react";
import { toast } from "sonner";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { FilePicker, type PickedFile } from "@/components/FilePicker";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { EmptyState } from "@/components/patterns/EmptyState";
import { PageHeader } from "@/components/patterns/PageHeader";
import { ProductAddSuggest } from "@/components/patterns/ProductAddSuggest";
import { UnsavedChangesDialog } from "@/components/patterns/UnsavedChangesDialog";
import { useProductAddSuggest } from "@/components/patterns/useProductAddSuggest";
import { PageShell } from "@/components/patterns/PageShell";
// 数量と単位の表記は共通formatterに揃え、単位列のある行は数値だけ描く。
import {
  formatStockDisplay,
  formatStockUnitLabel,
} from "@/features/stock-inquiry/lib/format-stock-display";
import { quantityInputAttributes } from "@/lib/unit-quantity";
import { useUnsavedChangesWarning } from "@/hooks/useUnsavedChangesWarning";
import { commands, type ProductWithRelations, type ReturnCreateResult } from "@/lib/bindings";
import { formatDateTime } from "@/lib/date-time";
import { describeError } from "@/lib/describe-error";
import { invalidateByContract, invalidationContract } from "@/lib/invalidation-contract";
import { unwrapResult } from "@/lib/invoke";
import { scrollPageToTop } from "@/lib/page-scroll";
import { queryKeys } from "@/lib/query-keys";
import { cn } from "@/lib/utils";
import { buildSaveImageRequest, getAllowedReceiptExtension } from "./lib/receipt-image";
import {
  buildReturnExchangeRequest,
  buildReturnExchangeSignature,
  createReturnExchangeIdempotencyKey,
  getLocalDateString,
} from "./lib/return-exchange-request";
import {
  addProductToReturnRows,
  changeReturnRowDirection,
  removeReturnRow,
  updateReturnRow,
} from "./lib/return-exchange-row-utils";
import type {
  ProductCandidate,
  ReceiptImageState,
  ReturnDirection,
  ReturnExchangeFormErrors,
  ReturnExchangeFormValues,
} from "./types";

const PRODUCT_SEARCH_QUERY = {
  department_id: null,
  is_discontinued: false,
  sort_key: "ProductCode" as const,
  sort_order: "Asc" as const,
  page: 1,
  per_page: 10,
};

function createEmptyForm(): ReturnExchangeFormValues {
  return {
    returnDate: getLocalDateString(),
    returnType: "return",
    registerProcessed: true,
    note: "",
    rows: [],
  };
}

function formatReturnType(value: string): string {
  return value === "exchange" ? "交換" : "返品";
}

function formatRegisterProcessed(value: boolean): string {
  return value ? "レジ戻し済み" : "レジ未処理";
}

function formatStockEffectDescription(value: boolean): string {
  return value
    ? "この保存では在庫数を変更しません。日次CSV取込みで返品分の在庫が反映されます。"
    : "この保存で在庫数を反映します。日次CSVに同じ返品を重ねて取込まない運用です。";
}

function formatStockEffectBadge(value: boolean): string {
  return value ? "CSV取込みで反映" : "この保存で反映";
}

// 空備考は他の記録画面と同じ「—」で示す。空白だけの値も空として扱う。
function formatNote(value: string | null | undefined): string {
  const trimmed = value?.trim() ?? "";
  return trimmed === "" ? "—" : trimmed;
}

function hasNote(value: string | null | undefined): boolean {
  return (value?.trim() ?? "") !== "";
}

function rowKey(row: { productCode: string; direction: ReturnDirection }): string {
  return `${row.productCode}:${row.direction}`;
}

function rowErrorSignature(row: ReturnExchangeFormValues["rows"][number]): string {
  return JSON.stringify({
    productCode: row.productCode,
    direction: row.direction,
    quantity: row.quantity,
  });
}

function createPreviewUrl(file: PickedFile): string {
  if (typeof URL !== "undefined" && "createObjectURL" in URL) {
    return URL.createObjectURL(new Blob([file.bytes.slice().buffer]));
  }
  return "";
}

function registerOptionClass(
  selected: boolean,
  locked: boolean,
  stockChangesOnSave: boolean,
): string {
  return cn(
    "flex cursor-pointer gap-3 rounded-md border p-3 text-sm transition-colors",
    "focus-within:border-ring focus-within:ring-[3px] focus-within:ring-ring",
    selected && stockChangesOnSave
      ? "border-warning-border bg-warning-soft"
      : selected
        ? "border-primary bg-primary/5"
        : "border-border bg-background hover:bg-muted/40",
    locked && "cursor-not-allowed opacity-60",
  );
}

function clearStaleRowErrors(
  errors: ReturnExchangeFormErrors,
  prevValues: ReturnExchangeFormValues,
  nextValues: ReturnExchangeFormValues,
): ReturnExchangeFormErrors {
  const nextErrors = { ...errors };
  if (errors.rows !== undefined) {
    const prevSignatures = new Map(
      prevValues.rows.map((row) => [rowKey(row), rowErrorSignature(row)]),
    );
    const nextSignatures = new Map(
      nextValues.rows.map((row) => [rowKey(row), rowErrorSignature(row)]),
    );
    const retainedRows = Object.fromEntries(
      Object.entries(errors.rows).filter(([key]) => {
        const prevSignature = prevSignatures.get(key);
        return prevSignature !== undefined && nextSignatures.get(key) === prevSignature;
      }),
    );
    if (Object.keys(retainedRows).length > 0) nextErrors.rows = retainedRows;
    else delete nextErrors.rows;
  }
  if (nextValues.rows.length > 0) delete nextErrors.items;
  return nextErrors;
}

// eslint-disable-next-line max-lines-per-function -- 衛生 batch 3 起票時実測（10a §3 item #2）: 画像保存・再送 key・返品方向変換・検索・結果表示が同居。画像/明細/保存 flow への分割は本 lane の scope 外、Backlog 参照
export function ReturnExchangePage() {
  const queryClient = useQueryClient();
  const returnTo = useRouterState({ select: (state) => state.location.href });
  const searchInputRef = useRef<HTMLInputElement>(null);
  const isFormLockedRef = useRef(false);
  // ponytail: 直接検索の lock と世代は画面ごとに持つ（UI-03-D24・catalog ⑮ D9）。同じ gate を持つ画面が
  // 6 つ（振替を入れて）を超えるか、gate の条件が増えたら共有 hook を検討する。
  const searchGenerationRef = useRef(0);
  const [initialValues, setInitialValues] = useState<ReturnExchangeFormValues>(createEmptyForm);
  const [values, setValues] = useState<ReturnExchangeFormValues>(initialValues);
  const [errors, setErrors] = useState<ReturnExchangeFormErrors>({});
  const [saveError, setSaveError] = useState<string | null>(null);
  const [searchText, setSearchText] = useState("");
  const [addDirection, setAddDirection] = useState<ReturnDirection>("in");
  const [searchMessage, setSearchMessage] = useState<string | null>(null);
  const [candidates, setCandidates] = useState<ProductCandidate[]>([]);
  const [receipt, setReceipt] = useState<ReceiptImageState | null>(null);
  const [idempotencyKey, setIdempotencyKey] = useState(createReturnExchangeIdempotencyKey);
  const [failedSignature, setFailedSignature] = useState<string | null>(null);
  const [result, setResult] = useState<ReturnCreateResult | null>(null);

  const recentQuery = useQuery({
    queryKey: queryKeys.returns.recent(),
    queryFn: () =>
      unwrapResult(commands.listReturns(1, 10, null, null), {
        source: "commands",
        cmd: "list_returns",
      }),
    staleTime: 30_000,
    gcTime: 5 * 60_000,
    retry: 0,
  });

  const createMutation = useMutation({
    mutationFn: async () => {
      let receiptImagePath = receipt?.savedReceiptPath ?? null;
      let built = buildReturnExchangeRequest(values, idempotencyKey, { receiptImagePath });
      setErrors(built.errors);
      if (built.request === null) throw new Error("validation");

      if (receipt !== null && receiptImagePath === null) {
        const saveRequest = buildSaveImageRequest(receipt.file);
        const saved = await unwrapResult(commands.saveReceiptImage(saveRequest), {
          source: "commands",
          cmd: "save_receipt_image",
        });
        receiptImagePath = saved.relative_path;
        setReceipt((prev) =>
          prev === null ? prev : { ...prev, savedReceiptPath: receiptImagePath },
        );
        built = buildReturnExchangeRequest(values, idempotencyKey, { receiptImagePath });
      }

      setErrors(built.errors);
      if (built.request === null) throw new Error("validation");
      setFailedSignature(built.signature);
      return unwrapResult(commands.createReturn(built.request), {
        source: "commands",
        cmd: "create_return",
      });
    },
    onSuccess: async (data) => {
      scrollPageToTop();
      setResult(data);
      setSaveError(null);
      setFailedSignature(null);
      setIdempotencyKey(createReturnExchangeIdempotencyKey());
      toast.success("返品・交換を保存しました", { id: "return-save-success" });
      await invalidateByContract(
        queryClient,
        invalidationContract.returnExchange(values.registerProcessed),
      );
    },
    onError: (error) => {
      isFormLockedRef.current = false;
      if (error instanceof Error && error.message === "validation") return;
      scrollPageToTop();
      setSaveError(describeError(error));
    },
  });

  const isSaving = createMutation.isPending;
  const isFormLocked = isSaving || result !== null;
  const canSubmit = values.rows.length > 0 && values.returnDate.trim() !== "" && !isFormLocked;
  const hasInputChanges =
    values.rows.length > 0 ||
    values.returnDate !== initialValues.returnDate ||
    values.returnType !== initialValues.returnType ||
    values.registerProcessed !== initialValues.registerProcessed ||
    values.note !== initialValues.note ||
    receipt !== null;
  const unsavedChanges = useUnsavedChangesWarning(hasInputChanges && !isFormLocked);
  const effectiveAddDirection: ReturnDirection =
    values.returnType === "exchange" ? addDirection : "in";
  const productSuggest = useProductAddSuggest({
    value: searchText,
    isLocked: () => isFormLockedRef.current,
    onSelect: (product) => {
      addProduct(product, effectiveAddDirection);
    },
  });

  useEffect(() => {
    if (recentQuery.isError) {
      toast.error("直近の返品・交換の取得に失敗しました", { id: "returns-recent-error" });
    } else if (recentQuery.isSuccess) {
      toast.dismiss("returns-recent-error");
    }
  }, [recentQuery.isError, recentQuery.isSuccess]);

  function maybeRotateKey(nextValues: ReturnExchangeFormValues, nextReceiptPath: string | null) {
    if (
      failedSignature !== null &&
      buildReturnExchangeSignature(nextValues, nextReceiptPath) !== failedSignature
    ) {
      setIdempotencyKey(createReturnExchangeIdempotencyKey());
      setFailedSignature(null);
    }
  }

  function rotateKeyAfterFailedAttempt() {
    if (failedSignature !== null) {
      setIdempotencyKey(createReturnExchangeIdempotencyKey());
      setFailedSignature(null);
    }
  }

  function updateValues(updater: (prev: ReturnExchangeFormValues) => ReturnExchangeFormValues) {
    setValues((prev) => {
      const next = updater(prev);
      setErrors((current) => clearStaleRowErrors(current, prev, next));
      maybeRotateKey(next, receipt?.savedReceiptPath ?? null);
      return next;
    });
    setResult(null);
  }

  function addProduct(product: ProductWithRelations, direction: ReturnDirection = "in") {
    if (isFormLockedRef.current) return;
    updateValues((prev) => ({
      ...prev,
      rows: addProductToReturnRows(
        prev.rows,
        product,
        prev.returnType === "exchange" ? direction : "in",
      ),
    }));
    setSearchText("");
    setCandidates([]);
    setSearchMessage(null);
    window.setTimeout(() => searchInputRef.current?.focus(), 0);
  }

  async function handleProductSearch(keywordOverride?: string) {
    const keyword = (keywordOverride ?? searchText).trim();
    if (keyword === "" || isFormLockedRef.current) return;
    const generation = searchGenerationRef.current;
    const isStale = () => generation !== searchGenerationRef.current || isFormLockedRef.current;
    setSearchMessage(null);
    setCandidates([]);
    try {
      const products = await unwrapResult(
        commands.searchProducts({ ...PRODUCT_SEARCH_QUERY, keyword }),
        {
          source: "commands",
          cmd: "search_products",
        },
      );
      if (isStale()) return;
      if (products.items.length === 0) {
        setSearchMessage("該当する商品がありません");
        return;
      }
      if (products.items.length === 1) {
        addProduct(products.items[0], effectiveAddDirection);
        return;
      }
      setCandidates(products.items);
      setSearchMessage("候補から返品・交換する商品を選んでください");
    } catch (error) {
      if (isStale()) return;
      setSearchMessage(describeError(error));
    }
  }

  function resetForm() {
    isFormLockedRef.current = false;
    searchGenerationRef.current += 1;
    const emptyValues = createEmptyForm();
    setInitialValues(emptyValues);
    setValues(emptyValues);
    setErrors({});
    setSaveError(null);
    setSearchText("");
    setAddDirection("in");
    setSearchMessage(null);
    setCandidates([]);
    setReceipt(null);
    setFailedSignature(null);
    setResult(null);
    setIdempotencyKey(createReturnExchangeIdempotencyKey());
    window.setTimeout(() => searchInputRef.current?.focus(), 0);
  }

  function handleReceiptFile(file: PickedFile) {
    if (isFormLockedRef.current) return;
    const extension = getAllowedReceiptExtension(file.filename);
    if (extension === null) {
      setErrors((prev) => ({
        ...prev,
        receipt: "jpg / jpeg / png / gif / webp の画像を選択してください",
      }));
      return;
    }
    setErrors((prev) => {
      const next = { ...prev };
      delete next.receipt;
      return next;
    });
    setReceipt({
      file,
      previewUrl: createPreviewUrl(file),
      extension,
      savedReceiptPath: null,
    });
    setResult(null);
    rotateKeyAfterFailedAttempt();
  }

  function removeReceiptImage() {
    if (isFormLocked) return;
    setReceipt(null);
    setErrors((prev) => {
      const next = { ...prev };
      delete next.receipt;
      return next;
    });
    setResult(null);
    rotateKeyAfterFailedAttempt();
  }

  return (
    <PageShell>
      <UnsavedChangesDialog warning={unsavedChanges} />
      <PageHeader
        title="返品・交換"
        subtitle="レジ戻し済みなら帳面記録だけ、未処理ならこの保存で在庫を反映します"
        actions={
          !isFormLocked ? (
            <Button asChild variant="outline">
              <Link to="/stock">
                <ArrowLeft aria-hidden="true" />
                在庫照会へ戻る
              </Link>
            </Button>
          ) : null
        }
      />

      {saveError !== null ? (
        <Alert variant="destructive">
          <AlertTitle>保存に失敗しました</AlertTitle>
          <AlertDescription>{saveError}</AlertDescription>
        </Alert>
      ) : null}

      {result !== null ? (
        <section aria-label="保存結果" className="space-y-3 rounded-md border bg-muted/30 p-4">
          <div className="flex flex-wrap items-center gap-2">
            <h2 className="text-xl font-semibold">返品・交換を保存しました</h2>
            {result.idempotent_replay ? <Badge variant="outline">再送結果</Badge> : null}
          </div>
          <div className="grid gap-3 text-sm sm:grid-cols-4">
            <div>
              <span className="text-muted-foreground">記録ID</span>
              <div className="font-medium">{result.record_id}</div>
            </div>
            <div>
              <span className="text-muted-foreground">明細数</span>
              <div className="font-medium">{values.rows.length} 件</div>
            </div>
            <div>
              <span className="text-muted-foreground">在庫反映</span>
              <div className="font-medium">{formatRegisterProcessed(values.registerProcessed)}</div>
              <p className="mt-1 text-xs text-muted-foreground">
                {formatStockEffectDescription(values.registerProcessed)}
              </p>
            </div>
            <div>
              <span className="text-muted-foreground">画像</span>
              <div className="font-medium">{receipt !== null ? "添付あり" : "なし"}</div>
            </div>
          </div>
          <div className="border-t pt-3 text-sm">
            <span className="text-muted-foreground">備考</span>
            <p
              className={cn(
                "mt-1 whitespace-pre-wrap text-foreground",
                !hasNote(values.note) && "text-muted-foreground",
              )}
            >
              {formatNote(values.note)}
            </p>
          </div>
          {result.stock_warnings.length > 0 ? (
            <ul className="list-disc space-y-1 pl-5 text-sm">
              {result.stock_warnings.map((warning) => (
                <li key={warning}>{warning}</li>
              ))}
            </ul>
          ) : null}
          <div className="flex flex-wrap gap-2">
            <Button type="button" onClick={resetForm}>
              <RotateCcw aria-hidden="true" />
              続けて返品・交換
            </Button>
            <Button asChild type="button" variant="outline">
              <Link
                to="/inventory/return/records/$recordId"
                params={{ recordId: String(result.record_id) }}
                search={{ returnTo }}
              >
                <Eye aria-hidden="true" />
                詳細を見る
              </Link>
            </Button>
            <Button asChild type="button" variant="outline">
              <Link to="/stock">
                <ArrowLeft aria-hidden="true" />
                在庫照会へ戻る
              </Link>
            </Button>
          </div>
        </section>
      ) : null}

      <section className="space-y-4 rounded-md border p-4">
        <h2 className="text-xl font-semibold">返品・交換内容</h2>
        <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(14rem,18rem)]">
          <div className="space-y-2">
            <Label htmlFor="return-date">返品日</Label>
            <Input
              id="return-date"
              type="date"
              value={values.returnDate}
              disabled={isFormLocked}
              aria-invalid={errors.returnDate !== undefined}
              onChange={(event) => {
                updateValues((prev) => ({ ...prev, returnDate: event.target.value }));
              }}
            />
            {errors.returnDate !== undefined ? (
              <p className="text-sm text-destructive">{errors.returnDate}</p>
            ) : null}
          </div>
          <div className="space-y-2">
            <Label htmlFor="return-type">種別</Label>
            <Select
              value={values.returnType}
              disabled={isFormLocked}
              onValueChange={(value) => {
                const nextReturnType = value === "exchange" ? "exchange" : "return";
                updateValues((prev) => {
                  let nextRows = prev.rows;
                  if (nextReturnType === "return") {
                    for (const row of prev.rows.filter((current) => current.direction === "out")) {
                      nextRows = changeReturnRowDirection(nextRows, row.productCode, "out", "in");
                    }
                  }
                  return {
                    ...prev,
                    returnType: nextReturnType,
                    rows: nextRows,
                  };
                });
                if (nextReturnType === "return") setAddDirection("in");
              }}
            >
              <SelectTrigger id="return-type" className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="return">返品</SelectItem>
                <SelectItem value="exchange">交換</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        <fieldset className="space-y-2">
          <legend className="text-sm font-medium">レジ戻し状況</legend>
          <div className="grid gap-3 md:grid-cols-2">
            <label className={registerOptionClass(values.registerProcessed, isFormLocked, false)}>
              <input
                type="radio"
                aria-label="レジ戻し済み"
                checked={values.registerProcessed}
                disabled={isFormLocked}
                className="mt-1 accent-primary"
                onChange={() => {
                  updateValues((prev) => ({ ...prev, registerProcessed: true }));
                }}
              />
              <span className="min-w-0 flex-1 space-y-1">
                <span className="flex flex-wrap items-center gap-2">
                  <span className="font-medium">レジ戻し済み</span>
                  <Badge variant="outline" className="border-stone-200 bg-stone-50 text-stone-700">
                    {formatStockEffectBadge(true)}
                  </Badge>
                </span>
                <span className="block text-muted-foreground">
                  {formatStockEffectDescription(true)}
                </span>
              </span>
            </label>
            <label className={registerOptionClass(!values.registerProcessed, isFormLocked, true)}>
              <input
                type="radio"
                aria-label="レジ未処理"
                checked={!values.registerProcessed}
                disabled={isFormLocked}
                className="mt-1 accent-primary"
                onChange={() => {
                  updateValues((prev) => ({ ...prev, registerProcessed: false }));
                }}
              />
              <span className="min-w-0 flex-1 space-y-1">
                <span className="flex flex-wrap items-center gap-2">
                  <span className="font-medium">レジ未処理</span>
                  <Badge
                    variant="outline"
                    className="border-warning-border bg-warning-soft text-warning-strong"
                  >
                    {formatStockEffectBadge(false)}
                  </Badge>
                </span>
                <span className="block text-muted-foreground">
                  {formatStockEffectDescription(false)}
                </span>
              </span>
            </label>
          </div>
        </fieldset>

        <div className="space-y-2">
          <Label htmlFor="return-note">備考</Label>
          <textarea
            id="return-note"
            value={values.note}
            disabled={isFormLocked}
            maxLength={200}
            rows={3}
            className="min-h-20 w-full rounded-md border border-input bg-control-surface px-3 py-2 text-sm shadow-xs transition-[color,box-shadow] outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50"
            placeholder="返品理由・交換理由・顧客対応メモを入力"
            onChange={(event) => {
              updateValues((prev) => ({ ...prev, note: event.target.value }));
            }}
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="receipt-image">レシート画像</Label>
          <FilePicker
            id="receipt-image"
            accept="image/*"
            ariaLabel="画像を選択（レシート画像）"
            buttonLabel="画像を選択"
            dialogFilterName="レシート画像"
            dropLabel="クリックして選択、またはここにドラッグ＆ドロップ"
            helperText="jpg / png / gif / webp"
            className="min-h-24 rounded-md bg-muted/20 p-3"
            disabled={isFormLocked}
            onSelect={handleReceiptFile}
          />
          {receipt !== null ? (
            <div className="flex flex-wrap items-center gap-3 rounded-md border p-3 text-sm">
              {receipt.previewUrl !== "" ? (
                <img
                  src={receipt.previewUrl}
                  alt="選択したレシート画像"
                  className="h-20 w-20 rounded border object-cover"
                />
              ) : null}
              <div className="min-w-0 flex-1 space-y-1 text-muted-foreground">
                <div className="flex flex-wrap items-center gap-2">
                  <ImagePlus aria-hidden="true" className="size-4" />
                  <span className="truncate">{receipt.file.filename}</span>
                </div>
                <span>{receipt.savedReceiptPath === null ? "未保存" : "保存済み"}</span>
              </div>
              <Button
                type="button"
                size="icon-sm"
                variant="ghost"
                disabled={isFormLocked}
                aria-label="レシート画像を削除"
                onClick={removeReceiptImage}
              >
                <Trash2 aria-hidden="true" />
              </Button>
            </div>
          ) : null}
          {errors.receipt !== undefined ? (
            <p className="text-sm text-destructive">{errors.receipt}</p>
          ) : null}
        </div>
      </section>

      <section className="space-y-4 rounded-md border p-4">
        <div className="flex flex-wrap items-end gap-2">
          <div className="min-w-[18rem] flex-1 space-y-2">
            <Label htmlFor="return-product-search">商品追加</Label>
            <ProductAddSuggest
              controller={productSuggest}
              onComposedDigitsCommit={(normalized) => {
                void handleProductSearch(normalized);
              }}
            >
              <Input
                ref={searchInputRef}
                id="return-product-search"
                value={searchText}
                disabled={isFormLocked}
                placeholder="商品コード・JAN・商品名を入力"
                aria-label="返品・交換商品検索"
                onChange={(event) => {
                  setSearchText(event.target.value);
                }}
                onKeyDown={(event) => {
                  if (event.nativeEvent.isComposing) return;
                  if (event.key === "Enter") {
                    event.preventDefault();
                    void handleProductSearch();
                  }
                }}
              />
            </ProductAddSuggest>
          </div>
          <div className="w-40 space-y-2">
            <Label htmlFor="return-add-direction">追加方向</Label>
            <Select
              value={effectiveAddDirection}
              disabled={isFormLocked || values.returnType === "return"}
              onValueChange={(value) => {
                setAddDirection(value === "out" ? "out" : "in");
              }}
            >
              <SelectTrigger id="return-add-direction" className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="in">戻り</SelectItem>
                <SelectItem value="out">渡し</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <Button
            type="button"
            variant="outline"
            disabled={isFormLocked}
            onClick={() => void handleProductSearch()}
          >
            <Search aria-hidden="true" />
            追加
          </Button>
        </div>

        {searchMessage !== null ? (
          <div className="flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
            <span>{searchMessage}</span>
            {searchMessage === "該当する商品がありません" ? (
              <>
                <span>
                  未登録商品の場合は、商品マスタに登録してから返品・交換へ戻って追加します。
                </span>
                {values.rows.length > 0 ? (
                  <span className="text-warning-emphasis">
                    未保存の返品・交換内容があります。商品登録へ進むとこの画面の入力は残りません。
                  </span>
                ) : null}
                {!isFormLocked ? (
                  <Button asChild variant="link" className="h-auto p-0">
                    <Link to="/products/new">商品登録へ進む</Link>
                  </Button>
                ) : null}
              </>
            ) : null}
          </div>
        ) : null}

        {candidates.length > 0 ? (
          <div className="rounded-md border">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>商品コード</TableHead>
                  <TableHead>商品名</TableHead>
                  <TableHead>部門</TableHead>
                  <TableHead>現在庫</TableHead>
                  <TableHead className="text-right">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {candidates.map((candidate) => (
                  <TableRow key={candidate.product_code}>
                    <TableCell className="font-medium">{candidate.product_code}</TableCell>
                    <TableCell>{candidate.name}</TableCell>
                    <TableCell>{candidate.department_name}</TableCell>
                    <TableCell>
                      {formatStockDisplay(candidate.stock_quantity, candidate.stock_unit)}
                    </TableCell>
                    <TableCell className="text-right">
                      <Button
                        type="button"
                        size="sm"
                        disabled={isFormLocked}
                        onClick={() => {
                          addProduct(candidate, effectiveAddDirection);
                        }}
                      >
                        {effectiveAddDirection === "out" ? "渡しに追加" : "戻りに追加"}
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        ) : null}

        {errors.items !== undefined ? (
          <p className="text-sm text-destructive">{errors.items}</p>
        ) : null}

        {values.rows.length === 0 ? (
          <EmptyState
            title="返品・交換する商品がありません"
            description="商品コード・JAN・商品名で検索して明細を追加してください"
          />
        ) : (
          <div className="rounded-md border">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>商品コード</TableHead>
                  <TableHead>商品名</TableHead>
                  <TableHead>部門</TableHead>
                  <TableHead>現在庫</TableHead>
                  <TableHead>方向</TableHead>
                  <TableHead>数量</TableHead>
                  <TableHead className="text-right">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {values.rows.map((row) => (
                  <TableRow key={rowKey(row)}>
                    <TableCell className="font-medium">{row.productCode}</TableCell>
                    <TableCell>{row.productName}</TableCell>
                    <TableCell>{row.departmentName}</TableCell>
                    <TableCell>
                      {formatStockDisplay(row.currentStockQuantity, row.stockUnit)}
                    </TableCell>
                    <TableCell>
                      <Select
                        value={row.direction}
                        disabled={isFormLocked || values.returnType === "return"}
                        onValueChange={(value) => {
                          const nextDirection = value === "out" ? "out" : "in";
                          updateValues((prev) => ({
                            ...prev,
                            rows: changeReturnRowDirection(
                              prev.rows,
                              row.productCode,
                              row.direction,
                              nextDirection,
                            ),
                          }));
                        }}
                      >
                        <SelectTrigger aria-label={`${row.productCode} の方向`}>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="in">戻り（在庫+）</SelectItem>
                          <SelectItem value="out">渡し（在庫-）</SelectItem>
                        </SelectContent>
                      </Select>
                    </TableCell>
                    <TableCell>
                      {/* 数量と単位を一つの値として読めるよう、同じ cell に添える。 */}
                      <div className="flex items-center gap-1">
                        <Input
                          type="number"
                          {...quantityInputAttributes(row.stockUnit)}
                          value={row.quantity}
                          disabled={isFormLocked}
                          aria-label={`${row.productCode} の数量`}
                          aria-invalid={errors.rows?.[rowKey(row)] !== undefined}
                          className="w-24"
                          onChange={(event) => {
                            updateValues((prev) => ({
                              ...prev,
                              rows: updateReturnRow(prev.rows, row.productCode, row.direction, {
                                quantity: event.target.value,
                              }),
                            }));
                          }}
                        />
                        <span className="text-sm text-muted-foreground">
                          {formatStockUnitLabel(row.stockUnit)}
                        </span>
                      </div>
                    </TableCell>
                    <TableCell className="text-right">
                      <Button
                        type="button"
                        size="icon-sm"
                        variant="ghost"
                        disabled={isFormLocked}
                        aria-label={`${row.productCode} を削除`}
                        onClick={() => {
                          updateValues((prev) => ({
                            ...prev,
                            rows: removeReturnRow(prev.rows, row.productCode, row.direction),
                          }));
                        }}
                      >
                        <Trash2 aria-hidden="true" />
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
            {errors.rows !== undefined ? (
              <div className="space-y-1 border-t p-3 text-sm text-destructive">
                {Object.entries(errors.rows).map(([key, message]) => (
                  <p key={key}>
                    {key}: {message}
                  </p>
                ))}
              </div>
            ) : null}
          </div>
        )}

        <div className="flex flex-wrap justify-end gap-2">
          <Button type="button" variant="outline" disabled={isFormLocked} onClick={resetForm}>
            <RotateCcw aria-hidden="true" />
            リセット
          </Button>
          <Button
            type="button"
            disabled={!canSubmit}
            onClick={() => {
              productSuggest.invalidateAndClose();
              setSaveError(null);
              isFormLockedRef.current = true;
              searchGenerationRef.current += 1;
              createMutation.mutate();
            }}
          >
            <RotateCcw aria-hidden="true" />
            {isSaving ? "保存中..." : "返品・交換を保存"}
          </Button>
        </div>
      </section>

      <section aria-label="直近の返品・交換" className="space-y-3 rounded-md border p-4">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <h2 className="text-xl font-semibold">直近の返品・交換</h2>
          <Button asChild variant="outline" size="sm">
            <Link to="/inventory/records" search={{ recordType: "return_record" }}>
              すべての履歴を見る
            </Link>
          </Button>
        </div>
        <p className="text-sm text-muted-foreground">
          直近 10 件の返品・交換を新しい順に表示します。
        </p>
        {recentQuery.isLoading ? (
          <div className="space-y-2">
            <Skeleton className="h-10 w-full" />
            <Skeleton className="h-10 w-full" />
          </div>
        ) : recentQuery.isError ? (
          <Alert variant="destructive">
            <AlertTitle>直近の返品・交換を取得できませんでした</AlertTitle>
            <AlertDescription>保存操作はこのまま続行できます。</AlertDescription>
          </Alert>
        ) : recentQuery.data?.items.length === 0 ? (
          <EmptyState
            title="直近の返品・交換はありません"
            description="保存するとここに表示されます"
          />
        ) : recentQuery.data ? (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>返品日</TableHead>
                <TableHead>種別</TableHead>
                <TableHead>レジ戻し</TableHead>
                <TableHead>備考</TableHead>
                <TableHead>記録日時</TableHead>
                <TableHead className="text-right">操作</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {recentQuery.data.items.map((record) => (
                <TableRow key={record.id}>
                  <TableCell className="font-medium">{record.return_date}</TableCell>
                  <TableCell>{formatReturnType(record.return_type)}</TableCell>
                  <TableCell>{formatRegisterProcessed(record.register_processed)}</TableCell>
                  <TableCell className="max-w-[24rem] min-w-[14rem] whitespace-normal">
                    <span
                      className={hasNote(record.note) ? "text-foreground" : "text-muted-foreground"}
                    >
                      {formatNote(record.note)}
                    </span>
                  </TableCell>
                  <TableCell className="font-mono tabular-nums">
                    {formatDateTime(record.created_at)}
                  </TableCell>
                  <TableCell className="text-right">
                    <Button asChild variant="outline" size="sm">
                      <Link
                        to="/inventory/return/records/$recordId"
                        params={{ recordId: String(record.id) }}
                        search={{ returnTo }}
                      >
                        <Eye aria-hidden="true" />
                        詳細を見る
                      </Link>
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        ) : null}
      </section>
    </PageShell>
  );
}
