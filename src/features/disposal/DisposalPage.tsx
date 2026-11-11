// src/features/disposal/DisposalPage.tsx
//
// UI-05 廃棄・破損 page。設計: docs/function-design/64-ui-disposal.md

import { useEffect, useRef, useState } from "react";
import { Link, useRouterState } from "@tanstack/react-router";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowLeft, Eye, RotateCcw, Search, Trash2 } from "lucide-react";
import { toast } from "sonner";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
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
  formatCostYen,
  formatStockDisplay,
  formatStockUnitLabel,
} from "@/features/stock-inquiry/lib/format-stock-display";
import { quantityInputAttributes } from "@/lib/unit-quantity";
import { useUnsavedChangesWarning } from "@/hooks/useUnsavedChangesWarning";
import { commands, type DisposalCreateResult, type ProductWithRelations } from "@/lib/bindings";
import { formatDateTime } from "@/lib/date-time";
import { describeError } from "@/lib/describe-error";
import { invalidateByContract, invalidationContract } from "@/lib/invalidation-contract";
import { unwrapResult } from "@/lib/invoke";
import { scrollPageToTop } from "@/lib/page-scroll";
import { queryKeys } from "@/lib/query-keys";
import {
  addProductToDisposalRows,
  removeDisposalRow,
  updateDisposalRow,
} from "./lib/disposal-row-utils";
import {
  buildDisposalRequest,
  buildDisposalSignature,
  calculateLossTotal,
  createDisposalIdempotencyKey,
  getLocalDateString,
} from "./lib/disposal-request";
import type {
  DisposalFormErrors,
  DisposalFormValues,
  DisposalType,
  ProductCandidate,
} from "./types";

const PRODUCT_SEARCH_QUERY = {
  department_id: null,
  is_discontinued: false,
  sort_key: "ProductCode" as const,
  sort_order: "Asc" as const,
  page: 1,
  per_page: 10,
};

const DISPOSAL_TYPE_LABELS: Record<DisposalType, string> = {
  disposal: "廃棄",
  damage: "破損",
  other: "その他",
};

function createEmptyForm(): DisposalFormValues {
  return {
    disposalDate: getLocalDateString(),
    rows: [],
  };
}

function formatYen(value: number): string {
  return `¥${value.toLocaleString("ja-JP")}`;
}

function rowErrorSignature(row: DisposalFormValues["rows"][number]): string {
  return JSON.stringify({
    rowId: row.rowId,
    quantity: row.quantity,
    costPrice: row.costPrice,
    reason: row.reason,
    disposalType: row.disposalType,
  });
}

function clearStaleRowErrors(
  errors: DisposalFormErrors,
  prevValues: DisposalFormValues,
  nextValues: DisposalFormValues,
): DisposalFormErrors {
  const nextErrors = { ...errors };
  if (errors.rows !== undefined) {
    const prevSignatures = new Map(
      prevValues.rows.map((row) => [row.rowId, rowErrorSignature(row)]),
    );
    const nextSignatures = new Map(
      nextValues.rows.map((row) => [row.rowId, rowErrorSignature(row)]),
    );
    const retainedRows = Object.fromEntries(
      Object.entries(errors.rows).filter(([rowId]) => {
        const prevSignature = prevSignatures.get(rowId);
        return prevSignature !== undefined && nextSignatures.get(rowId) === prevSignature;
      }),
    );
    if (Object.keys(retainedRows).length > 0) nextErrors.rows = retainedRows;
    else delete nextErrors.rows;
  }
  if (nextValues.rows.length > 0) delete nextErrors.items;
  return nextErrors;
}

export function DisposalPage() {
  const queryClient = useQueryClient();
  const returnTo = useRouterState({ select: (state) => state.location.href });
  const searchInputRef = useRef<HTMLInputElement>(null);
  const isFormLockedRef = useRef(false);
  // ponytail: 直接検索の lock と世代は画面ごとに持つ（UI-05-D15・catalog ⑮ D9）。同じ gate を持つ画面が
  // 6 つ（振替を入れて）を超えるか、gate の条件が増えたら共有 hook を検討する。
  const searchGenerationRef = useRef(0);
  const [initialValues, setInitialValues] = useState<DisposalFormValues>(createEmptyForm);
  const [values, setValues] = useState<DisposalFormValues>(initialValues);
  const [errors, setErrors] = useState<DisposalFormErrors>({});
  const [saveError, setSaveError] = useState<string | null>(null);
  const [searchText, setSearchText] = useState("");
  const [searchMessage, setSearchMessage] = useState<string | null>(null);
  const [candidates, setCandidates] = useState<ProductCandidate[]>([]);
  const [idempotencyKey, setIdempotencyKey] = useState(createDisposalIdempotencyKey);
  const [failedSignature, setFailedSignature] = useState<string | null>(null);
  const [result, setResult] = useState<DisposalCreateResult | null>(null);

  const recentQuery = useQuery({
    queryKey: queryKeys.disposals.recent(),
    queryFn: () =>
      unwrapResult(commands.listDisposals(1, 10, null, null), {
        source: "commands",
        cmd: "list_disposals",
      }),
    staleTime: 30_000,
    gcTime: 5 * 60_000,
    retry: 0,
  });

  const createMutation = useMutation({
    mutationFn: async () => {
      const built = buildDisposalRequest(values, idempotencyKey);
      setErrors(built.errors);
      if (built.request === null) throw new Error("validation");
      setFailedSignature(built.signature);
      return unwrapResult(commands.createDisposal(built.request), {
        source: "commands",
        cmd: "create_disposal",
      });
    },
    onSuccess: async (data) => {
      scrollPageToTop();
      setResult(data);
      setSaveError(null);
      setFailedSignature(null);
      setIdempotencyKey(createDisposalIdempotencyKey());
      toast.success("廃棄・破損を保存しました", { id: "disposal-save-success" });
      await invalidateByContract(queryClient, invalidationContract.disposal());
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
  const lossTotal = calculateLossTotal(values.rows);
  const canSubmit = values.rows.length > 0 && values.disposalDate.trim() !== "" && !isFormLocked;
  const recentRecords = recentQuery.data?.items ?? [];
  const hasInputChanges =
    values.rows.length > 0 || values.disposalDate !== initialValues.disposalDate;
  const unsavedChanges = useUnsavedChangesWarning(hasInputChanges && !isFormLocked);

  useEffect(() => {
    if (recentQuery.isError) {
      toast.error("直近の廃棄・破損記録の取得に失敗しました", {
        id: "disposal-recent-error",
      });
    } else if (recentQuery.isSuccess) {
      toast.dismiss("disposal-recent-error");
    }
  }, [recentQuery.isError, recentQuery.isSuccess]);

  function updateValues(updater: (prev: DisposalFormValues) => DisposalFormValues) {
    setValues((prev) => {
      const next = updater(prev);
      setErrors((current) => clearStaleRowErrors(current, prev, next));
      if (failedSignature !== null && buildDisposalSignature(next) !== failedSignature) {
        setIdempotencyKey(createDisposalIdempotencyKey());
        setFailedSignature(null);
      }
      return next;
    });
    setResult(null);
  }

  function addProduct(product: ProductWithRelations) {
    if (isFormLockedRef.current) return;
    updateValues((prev) => ({ ...prev, rows: addProductToDisposalRows(prev.rows, product) }));
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
      setSearchMessage("候補から廃棄・破損する商品を選んでください");
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
    setIdempotencyKey(createDisposalIdempotencyKey());
    window.setTimeout(() => searchInputRef.current?.focus(), 0);
  }

  return (
    <PageShell>
      <UnsavedChangesDialog warning={unsavedChanges} />
      <PageHeader
        title="廃棄・破損"
        subtitle="販売ではない理由で在庫を減らし、ロス理由と原価を記録します"
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
        <section className="space-y-3 rounded-md border bg-muted/30 p-4">
          <div className="flex flex-wrap items-center gap-2">
            <h2 className="text-xl font-semibold">廃棄・破損を保存しました</h2>
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
              <span className="text-muted-foreground">ロス原価合計</span>
              <div className="font-medium">{lossTotal === null ? "—" : formatYen(lossTotal)}</div>
            </div>
            <div>
              <span className="text-muted-foreground">在庫警告</span>
              <div className="font-medium">{result.stock_warnings.length} 件</div>
            </div>
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
              続けて廃棄・破損
            </Button>
            <Button asChild type="button" variant="outline">
              <Link
                to="/inventory/disposal/records/$recordId"
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
        <h2 className="text-xl font-semibold">廃棄・破損内容</h2>
        <div className="space-y-2">
          <Label htmlFor="disposal-date">廃棄日</Label>
          <Input
            id="disposal-date"
            type="date"
            value={values.disposalDate}
            disabled={isFormLocked}
            aria-invalid={errors.disposalDate !== undefined}
            onChange={(event) => {
              updateValues((prev) => ({ ...prev, disposalDate: event.target.value }));
            }}
          />
          {errors.disposalDate !== undefined ? (
            <p className="text-sm text-destructive">{errors.disposalDate}</p>
          ) : null}
        </div>
      </section>

      <section className="space-y-4 rounded-md border p-4">
        <div className="flex flex-wrap items-end gap-2">
          <div className="min-w-[18rem] flex-1 space-y-2">
            <Label htmlFor="disposal-product-search">商品追加</Label>
            <ProductAddSuggest
              controller={productSuggest}
              onComposedDigitsCommit={(normalized) => {
                void handleProductSearch(normalized);
              }}
            >
              <Input
                ref={searchInputRef}
                id="disposal-product-search"
                value={searchText}
                disabled={isFormLocked}
                placeholder="商品コード・JAN・商品名を入力"
                aria-label="廃棄・破損商品検索"
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
                  未登録商品の場合は、商品マスタに登録してから廃棄・破損へ戻って追加します。
                </span>
                {values.rows.length > 0 ? (
                  <span className="text-warning-emphasis">
                    未保存の廃棄・破損内容があります。商品登録へ進むとこの画面の入力は残りません。
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
                  <TableHead>原価</TableHead>
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
                    <TableCell>{formatCostYen(candidate.cost_price_centi)}</TableCell>
                    <TableCell className="text-right">
                      <Button
                        type="button"
                        size="sm"
                        disabled={isFormLocked}
                        onClick={() => {
                          addProduct(candidate);
                        }}
                      >
                        <Trash2 aria-hidden="true" />
                        廃棄・破損に追加
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
            title="廃棄・破損する商品がありません"
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
                  <TableHead>種別</TableHead>
                  <TableHead>数量</TableHead>
                  <TableHead>原価</TableHead>
                  <TableHead>理由</TableHead>
                  <TableHead className="text-right">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {values.rows.map((row) => (
                  <TableRow key={row.rowId}>
                    <TableCell className="font-medium">{row.productCode}</TableCell>
                    <TableCell>{row.productName}</TableCell>
                    <TableCell>{row.departmentName}</TableCell>
                    <TableCell>
                      {formatStockDisplay(row.currentStockQuantity, row.stockUnit)}
                    </TableCell>
                    <TableCell>
                      <Select
                        value={row.disposalType}
                        disabled={isFormLocked}
                        onValueChange={(value) => {
                          updateValues((prev) => ({
                            ...prev,
                            rows: updateDisposalRow(prev.rows, row.rowId, {
                              disposalType: value as DisposalType,
                            }),
                          }));
                        }}
                      >
                        <SelectTrigger aria-label={`${row.productCode} の種別`} className="w-28">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="disposal">{DISPOSAL_TYPE_LABELS.disposal}</SelectItem>
                          <SelectItem value="damage">{DISPOSAL_TYPE_LABELS.damage}</SelectItem>
                          <SelectItem value="other">{DISPOSAL_TYPE_LABELS.other}</SelectItem>
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
                          aria-invalid={errors.rows?.[row.rowId] !== undefined}
                          className="w-24"
                          onChange={(event) => {
                            updateValues((prev) => ({
                              ...prev,
                              rows: updateDisposalRow(prev.rows, row.rowId, {
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
                    <TableCell>
                      <div className="flex items-center gap-1">
                        <Input
                          type="number"
                          inputMode="decimal"
                          min="0"
                          step="0.01"
                          value={row.costPrice}
                          disabled={isFormLocked}
                          aria-label={`${row.productCode} の原価`}
                          aria-invalid={errors.rows?.[row.rowId] !== undefined}
                          className="w-28"
                          onChange={(event) => {
                            updateValues((prev) => ({
                              ...prev,
                              rows: updateDisposalRow(prev.rows, row.rowId, {
                                costPrice: event.target.value,
                              }),
                            }));
                          }}
                        />
                        <span className="text-sm text-muted-foreground">円</span>
                      </div>
                    </TableCell>
                    <TableCell>
                      <Input
                        value={row.reason}
                        disabled={isFormLocked}
                        aria-label={`${row.productCode} の理由`}
                        aria-invalid={errors.rows?.[row.rowId] !== undefined}
                        className="min-w-36"
                        onChange={(event) => {
                          updateValues((prev) => ({
                            ...prev,
                            rows: updateDisposalRow(prev.rows, row.rowId, {
                              reason: event.target.value,
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
                            rows: removeDisposalRow(prev.rows, row.rowId),
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
                {Object.entries(errors.rows).map(([rowId, message]) => {
                  const row = values.rows.find((candidate) => candidate.rowId === rowId);
                  return (
                    <p key={rowId}>
                      {row?.productCode ?? rowId}: {message}
                    </p>
                  );
                })}
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
            <Trash2 aria-hidden="true" />
            {isSaving ? "保存中..." : "廃棄・破損を保存"}
          </Button>
        </div>
      </section>

      <section className="space-y-3 rounded-md border p-4">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <h2 className="text-xl font-semibold">直近の廃棄・破損</h2>
          <Button asChild variant="outline" size="sm">
            <Link to="/inventory/records" search={{ recordType: "disposal_record" }}>
              すべての履歴を見る
            </Link>
          </Button>
        </div>
        <p className="text-sm text-muted-foreground">
          直近 10 件の廃棄・破損を新しい順に表示します。
        </p>
        {recentQuery.isLoading ? (
          <div className="space-y-2">
            <Skeleton className="h-9 w-full" />
            <Skeleton className="h-9 w-full" />
          </div>
        ) : recentQuery.isError ? (
          <Alert variant="destructive">
            <AlertTitle>直近の廃棄・破損を取得できません</AlertTitle>
            <AlertDescription>
              保存は可能です。必要に応じて画面を開き直してください。
            </AlertDescription>
          </Alert>
        ) : recentRecords.length === 0 ? (
          <EmptyState
            title="直近の廃棄・破損はありません"
            description="保存するとここに最新の記録が表示されます"
          />
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>廃棄日</TableHead>
                <TableHead>記録ID</TableHead>
                <TableHead>記録日時</TableHead>
                <TableHead className="text-right">操作</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {recentRecords.map((item) => (
                <TableRow key={item.id}>
                  <TableCell>{item.disposal_date}</TableCell>
                  <TableCell>{item.id}</TableCell>
                  <TableCell className="font-mono tabular-nums">
                    {formatDateTime(item.created_at)}
                  </TableCell>
                  <TableCell className="text-right">
                    <Button asChild variant="outline" size="sm">
                      <Link
                        to="/inventory/disposal/records/$recordId"
                        params={{ recordId: String(item.id) }}
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
        )}
      </section>
    </PageShell>
  );
}
