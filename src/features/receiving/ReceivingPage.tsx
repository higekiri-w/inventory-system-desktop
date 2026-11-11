// src/features/receiving/ReceivingPage.tsx
//
// UI-02 入庫記録 page。設計: docs/function-design/61-ui-receiving.md

import { useEffect, useMemo, useRef, useState } from "react";
import { Link, useRouterState } from "@tanstack/react-router";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowLeft, ChevronDown, Eye, PackagePlus, RotateCcw, Search, Trash2 } from "lucide-react";
import { toast } from "sonner";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
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
import { formatStockUnitLabel } from "@/features/stock-inquiry/lib/format-stock-display";
import { quantityInputAttributes } from "@/lib/unit-quantity";
import { useUnsavedChangesWarning } from "@/hooks/useUnsavedChangesWarning";
import { commands, type ProductWithRelations, type ReceivingCreateResult } from "@/lib/bindings";
import { formatDateTime } from "@/lib/date-time";
import { describeError } from "@/lib/describe-error";
import { invalidateByContract, invalidationContract } from "@/lib/invalidation-contract";
import { unwrapResult } from "@/lib/invoke";
import { scrollPageToTop } from "@/lib/page-scroll";
import { queryKeys } from "@/lib/query-keys";
import {
  SupplierPickerDialog,
  supplierCurrentLabel,
} from "@/features/suppliers/components/SupplierPickerDialog";
import {
  addProductToRows,
  removeReceivingRow,
  updateReceivingRow,
} from "./lib/receiving-row-utils";
import {
  buildReceivingRequest,
  buildReceivingSignature,
  createReceivingIdempotencyKey,
  getLocalDateString,
} from "./lib/receiving-request";
import type { ProductCandidate, ReceivingFormErrors, ReceivingFormValues } from "./types";
import { CostDiffDialog } from "./CostDiffDialog";

const PRODUCT_SEARCH_QUERY = {
  department_id: null,
  is_discontinued: false,
  sort_key: "ProductCode" as const,
  sort_order: "Asc" as const,
  page: 1,
  per_page: 10,
};

function createEmptyForm(): ReceivingFormValues {
  return {
    supplierId: null,
    receivingDate: getLocalDateString(),
    note: "",
    rows: [],
  };
}

function formatSupplierName(name: string | null): string {
  return name ?? "取引先未指定";
}

function rowErrorSignature(row: ReceivingFormValues["rows"][number]): string {
  return JSON.stringify({
    productCode: row.productCode,
    quantity: row.quantity,
    costPrice: row.costPrice,
  });
}

function clearStaleRowErrors(
  errors: ReceivingFormErrors,
  prevValues: ReceivingFormValues,
  nextValues: ReceivingFormValues,
): ReceivingFormErrors {
  const nextErrors = { ...errors };
  if (errors.rows !== undefined) {
    const prevSignatures = new Map(
      prevValues.rows.map((row) => [row.productCode, rowErrorSignature(row)]),
    );
    const nextSignatures = new Map(
      nextValues.rows.map((row) => [row.productCode, rowErrorSignature(row)]),
    );
    const retainedRows = Object.fromEntries(
      Object.entries(errors.rows).filter(([productCode]) => {
        const prevSignature = prevSignatures.get(productCode);
        return prevSignature !== undefined && nextSignatures.get(productCode) === prevSignature;
      }),
    );
    if (Object.keys(retainedRows).length > 0) nextErrors.rows = retainedRows;
    else delete nextErrors.rows;
  }
  if (nextValues.rows.length > 0) delete nextErrors.items;
  return nextErrors;
}

export function ReceivingPage() {
  const queryClient = useQueryClient();
  const returnTo = useRouterState({ select: (state) => state.location.href });
  const searchInputRef = useRef<HTMLInputElement>(null);
  const isFormLockedRef = useRef(false);
  // ponytail: 直接検索の lock と世代は画面ごとに持つ（UI-02-D19・catalog ⑮ D9）。同じ gate を持つ画面が
  // 6 つ（振替を入れて）を超えるか、gate の条件が増えたら共有 hook を検討する。
  const searchGenerationRef = useRef(0);
  const [initialValues, setInitialValues] = useState<ReceivingFormValues>(createEmptyForm);
  const [values, setValues] = useState<ReceivingFormValues>(initialValues);
  const [errors, setErrors] = useState<ReceivingFormErrors>({});
  const [saveError, setSaveError] = useState<string | null>(null);
  const [searchText, setSearchText] = useState("");
  const [searchMessage, setSearchMessage] = useState<string | null>(null);
  const [candidates, setCandidates] = useState<ProductCandidate[]>([]);
  const [idempotencyKey, setIdempotencyKey] = useState(createReceivingIdempotencyKey);
  const [failedSignature, setFailedSignature] = useState<string | null>(null);
  const [result, setResult] = useState<ReceivingCreateResult | null>(null);
  const [supplierPickerOpen, setSupplierPickerOpen] = useState(false);
  const [isCostDiffDialogOpen, setIsCostDiffDialogOpen] = useState(false);

  const supplierQuery = useQuery({
    queryKey: queryKeys.productForm.suppliers(),
    queryFn: () =>
      unwrapResult(commands.listSuppliers(), {
        source: "commands",
        cmd: "list_suppliers",
      }),
    staleTime: 5 * 60_000,
    gcTime: 5 * 60_000,
    retry: 0,
  });

  const recentQuery = useQuery({
    queryKey: queryKeys.receivings.recent(),
    queryFn: () =>
      unwrapResult(commands.listReceivings(1, 10, null, null), {
        source: "commands",
        cmd: "list_receivings",
      }),
    staleTime: 30_000,
    gcTime: 5 * 60_000,
    retry: 0,
  });

  const supplierOptions = useMemo(
    () => [...(supplierQuery.data ?? [])].sort((a, b) => a.id - b.id),
    [supplierQuery.data],
  );

  const createMutation = useMutation({
    mutationFn: async () => {
      const built = buildReceivingRequest(values, idempotencyKey);
      setErrors(built.errors);
      if (built.request === null) throw new Error("validation");
      setFailedSignature(built.signature);
      return unwrapResult(commands.createReceiving(built.request), {
        source: "commands",
        cmd: "create_receiving",
      });
    },
    onSuccess: async (data) => {
      scrollPageToTop();
      setResult(data);
      setIsCostDiffDialogOpen(
        data.created && !data.idempotent_replay && data.cost_diffs.length > 0,
      );
      setSaveError(null);
      setFailedSignature(null);
      setIdempotencyKey(createReceivingIdempotencyKey());
      toast.success("入庫記録を保存しました", { id: "receiving-save-success" });
      await invalidateByContract(queryClient, invalidationContract.receiving());
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
  const productSuggest = useProductAddSuggest({
    value: searchText,
    isLocked: () => isFormLockedRef.current,
    onSelect: addProduct,
  });
  const canSubmit = values.rows.length > 0 && values.receivingDate.trim() !== "" && !isFormLocked;
  const hasInputChanges =
    values.rows.length > 0 ||
    values.supplierId !== initialValues.supplierId ||
    values.receivingDate !== initialValues.receivingDate ||
    values.note !== initialValues.note;
  const unsavedChanges = useUnsavedChangesWarning(hasInputChanges && !isFormLocked);

  useEffect(() => {
    if (recentQuery.isError) {
      toast.error("直近入庫の取得に失敗しました", { id: "receiving-recent-error" });
    } else if (recentQuery.isSuccess) {
      toast.dismiss("receiving-recent-error");
    }
  }, [recentQuery.isError, recentQuery.isSuccess]);

  function updateValues(updater: (prev: ReceivingFormValues) => ReceivingFormValues) {
    setValues((prev) => {
      const next = updater(prev);
      setErrors((current) => clearStaleRowErrors(current, prev, next));
      if (failedSignature !== null && buildReceivingSignature(next) !== failedSignature) {
        setIdempotencyKey(createReceivingIdempotencyKey());
        setFailedSignature(null);
      }
      return next;
    });
    setResult(null);
    setIsCostDiffDialogOpen(false);
  }

  function addProduct(product: ProductWithRelations) {
    if (isFormLockedRef.current) return;
    updateValues((prev) => ({ ...prev, rows: addProductToRows(prev.rows, product) }));
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
        addProduct(products.items[0]);
        return;
      }
      setCandidates(products.items);
      setSearchMessage("候補から入庫する商品を選んでください");
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
    setSearchMessage(null);
    setCandidates([]);
    setFailedSignature(null);
    setResult(null);
    setIdempotencyKey(createReceivingIdempotencyKey());
    window.setTimeout(() => searchInputRef.current?.focus(), 0);
  }

  return (
    <PageShell>
      <UnsavedChangesDialog warning={unsavedChanges} />
      {result !== null ? (
        <CostDiffDialog
          key={result.record_id}
          costDiffs={result.cost_diffs}
          open={isCostDiffDialogOpen}
          onOpenChange={setIsCostDiffDialogOpen}
        />
      ) : null}
      <PageHeader
        title="入庫記録"
        subtitle="届いた商品をまとめて入庫し、在庫へ反映します"
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

      {supplierQuery.isError ? (
        <Alert>
          <AlertTitle>取引先候補を取得できませんでした</AlertTitle>
          <AlertDescription>取引先を指定しない入庫記録は保存できます。</AlertDescription>
        </Alert>
      ) : null}

      {saveError !== null ? (
        <Alert variant="destructive">
          <AlertTitle>保存に失敗しました</AlertTitle>
          <AlertDescription>{saveError}</AlertDescription>
        </Alert>
      ) : null}

      {result !== null ? (
        <section className="space-y-3 rounded-md border bg-muted/30 p-4">
          <div className="flex flex-wrap items-center gap-2">
            <h2 className="text-xl font-semibold">入庫を保存しました</h2>
            {result.idempotent_replay ? <Badge variant="outline">再送結果</Badge> : null}
          </div>
          <div className="grid gap-3 text-sm sm:grid-cols-3">
            <div>
              <span className="text-muted-foreground">記録ID</span>
              <div className="font-medium">{result.record_id}</div>
            </div>
            <div>
              <span className="text-muted-foreground">明細数</span>
              <div className="font-medium">{values.rows.length} 件</div>
            </div>
            <div>
              <span className="text-muted-foreground">警告</span>
              <div className="font-medium">{result.stock_warnings.length} 件</div>
            </div>
          </div>
          {result.idempotent_replay ? (
            <p className="text-sm">同じ内容の再送として処理済み</p>
          ) : null}
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
              続けて入庫
            </Button>
            <Button asChild type="button" variant="outline">
              <Link
                to="/inventory/receiving/records/$recordId"
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
        <h2 className="text-xl font-semibold">入庫内容</h2>
        <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(14rem,18rem)]">
          <div className="space-y-2">
            <Label htmlFor="receiving-date">入庫日</Label>
            <Input
              id="receiving-date"
              type="date"
              value={values.receivingDate}
              disabled={isFormLocked}
              aria-invalid={errors.receivingDate !== undefined}
              onChange={(event) => {
                updateValues((prev) => ({ ...prev, receivingDate: event.target.value }));
              }}
            />
            {errors.receivingDate !== undefined ? (
              <p className="text-sm text-destructive">{errors.receivingDate}</p>
            ) : null}
          </div>
          <div className="space-y-2">
            <Label id="receiving-supplier-label" htmlFor="receiving-supplier">
              取引先
            </Label>
            <Button
              type="button"
              variant="outline"
              id="receiving-supplier"
              className="w-full justify-between bg-control-surface"
              aria-haspopup="dialog"
              aria-labelledby="receiving-supplier-label receiving-supplier"
              disabled={isFormLocked || supplierQuery.isLoading}
              onClick={() => {
                setSupplierPickerOpen(true);
              }}
            >
              <span className="truncate">
                {supplierCurrentLabel(supplierOptions, values.supplierId ?? null, "指定なし")}
              </span>
              <ChevronDown className="size-4 text-muted-foreground opacity-50" aria-hidden="true" />
            </Button>
            <SupplierPickerDialog
              open={supplierPickerOpen}
              onOpenChange={setSupplierPickerOpen}
              suppliers={supplierOptions}
              isLoading={supplierQuery.isLoading}
              isError={supplierQuery.isError}
              onRetry={() => void supplierQuery.refetch()}
              leadingLabel="指定なし"
              selected={values.supplierId}
              onSelect={(id) => {
                updateValues((prev) => ({ ...prev, supplierId: id }));
              }}
              onCreated={async () => {
                await supplierQuery.refetch();
              }}
            />
          </div>
        </div>

        <div className="space-y-2">
          <Label htmlFor="receiving-note">備考</Label>
          <Input
            id="receiving-note"
            value={values.note}
            disabled={isFormLocked}
            maxLength={200}
            onChange={(event) => {
              updateValues((prev) => ({ ...prev, note: event.target.value }));
            }}
          />
        </div>
      </section>

      <section className="space-y-4 rounded-md border p-4">
        <div className="flex flex-wrap items-end gap-2">
          <div className="min-w-[18rem] flex-1 space-y-2">
            <Label htmlFor="receiving-product-search">商品追加</Label>
            <ProductAddSuggest
              controller={productSuggest}
              onComposedDigitsCommit={(normalized) => {
                void handleProductSearch(normalized);
              }}
            >
              <Input
                ref={searchInputRef}
                id="receiving-product-search"
                value={searchText}
                disabled={isFormLocked}
                placeholder="商品コード・JAN・商品名を入力"
                aria-label="入庫商品検索"
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
          <Button
            type="button"
            variant="outline"
            disabled={isFormLocked}
            onClick={() => {
              void handleProductSearch();
            }}
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
                  未登録商品の場合は、商品マスタに登録してから入庫記録に戻って追加します。
                </span>
                {values.rows.length > 0 ? (
                  <span className="text-warning-emphasis">
                    未保存の入庫内容があります。商品登録へ進むとこの画面の入力は残りません。
                  </span>
                ) : null}
                <Button asChild variant="link" className="h-auto p-0">
                  <Link to="/products/new">商品登録へ進む</Link>
                </Button>
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
                  <TableHead className="text-right">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {candidates.map((candidate) => (
                  <TableRow key={candidate.product_code}>
                    <TableCell className="font-medium">{candidate.product_code}</TableCell>
                    <TableCell>{candidate.name}</TableCell>
                    <TableCell>{candidate.department_name}</TableCell>
                    <TableCell className="text-right">
                      <Button
                        type="button"
                        size="sm"
                        disabled={isFormLocked}
                        onClick={() => {
                          addProduct(candidate);
                        }}
                      >
                        <PackagePlus aria-hidden="true" />
                        入庫に追加
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
            title="入庫する商品がありません"
            description="商品コード・JAN・商品名で検索して明細を追加してください"
          />
        ) : (
          <div className="rounded-md border">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>商品コード</TableHead>
                  <TableHead>商品名</TableHead>
                  <TableHead>単位</TableHead>
                  <TableHead>数量</TableHead>
                  <TableHead>原価</TableHead>
                  <TableHead className="text-right">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {values.rows.map((row) => (
                  <TableRow key={row.productCode}>
                    <TableCell className="font-medium">{row.productCode}</TableCell>
                    <TableCell>{row.productName}</TableCell>
                    <TableCell>{formatStockUnitLabel(row.stockUnit)}</TableCell>
                    <TableCell>
                      <Input
                        type="number"
                        {...quantityInputAttributes(row.stockUnit)}
                        value={row.quantity}
                        disabled={isFormLocked}
                        aria-label={`${row.productCode} の数量`}
                        aria-invalid={errors.rows?.[row.productCode] !== undefined}
                        className="w-24"
                        onChange={(event) => {
                          updateValues((prev) => ({
                            ...prev,
                            rows: updateReceivingRow(prev.rows, row.productCode, {
                              quantity: event.target.value,
                            }),
                          }));
                        }}
                      />
                    </TableCell>
                    <TableCell>
                      <Input
                        type="number"
                        inputMode="decimal"
                        min="0"
                        step="0.01"
                        value={row.costPrice}
                        disabled={isFormLocked}
                        aria-label={`${row.productCode} の原価`}
                        aria-invalid={errors.rows?.[row.productCode] !== undefined}
                        className="w-28"
                        onChange={(event) => {
                          updateValues((prev) => ({
                            ...prev,
                            rows: updateReceivingRow(prev.rows, row.productCode, {
                              costPrice: event.target.value,
                            }),
                          }));
                        }}
                      />
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
                            rows: removeReceivingRow(prev.rows, row.productCode),
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
                {Object.entries(errors.rows).map(([productCode, message]) => (
                  <p key={productCode}>
                    {productCode}: {message}
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
              isFormLockedRef.current = true;
              searchGenerationRef.current += 1;
              productSuggest.invalidateAndClose();
              setSaveError(null);
              createMutation.mutate();
            }}
          >
            <PackagePlus aria-hidden="true" />
            {isSaving ? "保存中..." : "入庫を保存"}
          </Button>
        </div>
      </section>

      <section className="space-y-3 rounded-md border p-4">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <h2 className="text-xl font-semibold">直近の入庫</h2>
          <Button asChild variant="outline" size="sm">
            <Link to="/inventory/records" search={{ recordType: "receiving_record" }}>
              すべての履歴を見る
            </Link>
          </Button>
        </div>
        <p className="text-sm text-muted-foreground">直近 10 件の入庫を新しい順に表示します。</p>
        {recentQuery.isLoading ? (
          <div className="space-y-2">
            <Skeleton className="h-10 w-full" />
            <Skeleton className="h-10 w-full" />
          </div>
        ) : recentQuery.isError ? (
          <Alert variant="destructive">
            <AlertTitle>直近入庫を取得できませんでした</AlertTitle>
            <AlertDescription>保存操作はこのまま続行できます。</AlertDescription>
          </Alert>
        ) : recentQuery.data?.items.length === 0 ? (
          <EmptyState title="直近の入庫はありません" description="保存するとここに表示されます" />
        ) : recentQuery.data ? (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>入庫日</TableHead>
                <TableHead>取引先</TableHead>
                <TableHead>備考</TableHead>
                <TableHead>記録日時</TableHead>
                <TableHead className="text-right">操作</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {recentQuery.data.items.map((record) => (
                <TableRow key={record.id}>
                  <TableCell className="font-medium">{record.receiving_date}</TableCell>
                  <TableCell>{formatSupplierName(record.supplier_name)}</TableCell>
                  <TableCell
                    className="max-w-80 truncate"
                    title={record.note?.trim() ? record.note : undefined}
                  >
                    {record.note?.trim() ? record.note : "—"}
                  </TableCell>
                  <TableCell className="font-mono tabular-nums">
                    {formatDateTime(record.created_at)}
                  </TableCell>
                  <TableCell className="text-right">
                    <Button asChild variant="outline" size="sm">
                      <Link
                        to="/inventory/receiving/records/$recordId"
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
