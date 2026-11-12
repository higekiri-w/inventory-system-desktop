// src/features/stock-transfer/StockTransferPage.tsx
//
// UI-16 詰め替え・切り分け page。設計: docs/function-design/79-ui-stock-transfer.md
// 廃棄の作成画面（src/features/disposal/DisposalPage.tsx）と同じ形で、明細の表と商品追加欄を
// 「減らす商品」「増やす商品」の 2 つに分ける（§79.2。新しい共通部品は作らない）。

import { useEffect, useRef, useState } from "react";
import { Link, useRouterState } from "@tanstack/react-router";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowLeft, ArrowLeftRight, Eye, Plus, RotateCcw, Search, Trash2 } from "lucide-react";
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
import { PageShell } from "@/components/patterns/PageShell";
import { ProductAddSuggest } from "@/components/patterns/ProductAddSuggest";
import { UnsavedChangesDialog } from "@/components/patterns/UnsavedChangesDialog";
import { useProductAddSuggest } from "@/components/patterns/useProductAddSuggest";
import {
  formatStockDisplay,
  formatStockUnitLabel,
} from "@/features/stock-inquiry/lib/format-stock-display";
import { useUnsavedChangesWarning } from "@/hooks/useUnsavedChangesWarning";
import {
  commands,
  type InventoryRecordQuery,
  type ProductWithRelations,
  type TransferCreateResult,
  type TransferDirection,
} from "@/lib/bindings";
import { formatDateTime } from "@/lib/date-time";
import { describeError } from "@/lib/describe-error";
import { invalidateByContract, invalidationContract } from "@/lib/invalidation-contract";
import { unwrapResult } from "@/lib/invoke";
import { scrollPageToTop } from "@/lib/page-scroll";
import { queryKeys } from "@/lib/query-keys";
import {
  ITEM_QUANTITY_FIELD,
  parseQuantityInput,
  quantityInputAttributes,
} from "@/lib/unit-quantity";
import {
  BOTH_SIDES_REQUIRED_MESSAGE,
  TRANSFER_SIDE_LABELS,
  addProductToTransfer,
  buildTransferRequest,
  buildTransferSignature,
  createTransferIdempotencyKey,
  getLocalDateString,
  hasBothSides,
  removeTransferRow,
  rowErrorKey,
  rowsOf,
  updateTransferRowQuantity,
  type TransferFormErrors,
  type TransferFormValues,
} from "./lib/transfer-request";

const PRODUCT_SEARCH_QUERY = {
  department_id: null,
  is_discontinued: false,
  sort_key: "ProductCode" as const,
  sort_order: "Asc" as const,
  page: 1,
  per_page: 10,
};

// UI-16-D9: 最近の記録は入出庫履歴と同じ一覧を種別で絞って読む
const RECENT_TRANSFERS_QUERY: InventoryRecordQuery = {
  record_type: "transfer_record",
  date_from: null,
  date_to: null,
  record_id: null,
  product_keyword: null,
  department_id: null,
  status: null,
  page: 1,
  per_page: 10,
};

const SIDES: readonly TransferDirection[] = ["out", "in"];

const SIDE_HEADINGS: Record<TransferDirection, string> = {
  out: "減らす商品（元）",
  in: "増やす商品（先）",
};

const RESULT_HEADINGS: Record<TransferDirection, string> = {
  out: "減らした商品",
  in: "増やした商品",
};

interface SideSearchState {
  text: string;
  message: string | null;
  candidates: ProductWithRelations[];
}

const EMPTY_SEARCH: SideSearchState = { text: "", message: null, candidates: [] };

function createEmptyForm(): TransferFormValues {
  return { transferDate: getLocalDateString(), outRows: [], inRows: [] };
}

export function StockTransferPage() {
  const queryClient = useQueryClient();
  const returnTo = useRouterState({ select: (state) => state.location.href });
  const searchInputRefs = {
    out: useRef<HTMLInputElement>(null),
    in: useRef<HTMLInputElement>(null),
  };
  // UI-16-D7: 2 つの商品追加欄が同じ lock を読む
  const isFormLockedRef = useRef(false);
  // UI-16-D7: 保存 event の先頭・リセット・「続けて」で世代を進め、それより前に始めた直接検索の応答は、
  // 保存の失敗で unlock した後に届いても採らない（64 UI-05-D15 と同じ。owner の決定 TD-655）。
  // 両欄で共有し、検索開始では進めない（連続スキャンで前の商品を落とさない）。
  const searchGenerationRef = useRef(0);
  const [initialValues, setInitialValues] = useState<TransferFormValues>(createEmptyForm);
  const [values, setValues] = useState<TransferFormValues>(initialValues);
  const [errors, setErrors] = useState<TransferFormErrors>({});
  const [saveError, setSaveError] = useState<string | null>(null);
  const [search, setSearch] = useState<Record<TransferDirection, SideSearchState>>({
    out: EMPTY_SEARCH,
    in: EMPTY_SEARCH,
  });
  const [idempotencyKey, setIdempotencyKey] = useState(createTransferIdempotencyKey);
  const [failedSignature, setFailedSignature] = useState<string | null>(null);
  const [result, setResult] = useState<TransferCreateResult | null>(null);

  const recentQuery = useQuery({
    queryKey: queryKeys.inventoryRecords.list({
      recordType: "transfer_record",
      page: 1,
      perPage: 10,
    }),
    queryFn: () =>
      unwrapResult(commands.listInventoryRecords(RECENT_TRANSFERS_QUERY), {
        source: "commands",
        cmd: "list_inventory_records",
      }),
    staleTime: 30_000,
    gcTime: 5 * 60_000,
    retry: 0,
  });

  const createMutation = useMutation({
    mutationFn: async () => {
      const built = buildTransferRequest(values, idempotencyKey);
      setErrors(built.errors);
      if (built.request === null) throw new Error("validation");
      setFailedSignature(built.signature);
      return unwrapResult(commands.createTransfer(built.request), {
        source: "commands",
        cmd: "create_transfer",
      });
    },
    onSuccess: async (data) => {
      scrollPageToTop();
      setResult(data);
      setSaveError(null);
      setFailedSignature(null);
      setIdempotencyKey(createTransferIdempotencyKey());
      toast.success("詰め替え・切り分けを保存しました", { id: "transfer-save-success" });
      await invalidateByContract(queryClient, invalidationContract.stockTransfer());
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
  const suggests = {
    out: useProductAddSuggest({
      value: search.out.text,
      isLocked: () => isFormLockedRef.current,
      onSelect: (product) => {
        addProduct("out", product);
      },
    }),
    in: useProductAddSuggest({
      value: search.in.text,
      isLocked: () => isFormLockedRef.current,
      onSelect: (product) => {
        addProduct("in", product);
      },
    }),
  };
  const bothSides = hasBothSides(values);
  const canSubmit = bothSides && values.transferDate.trim() !== "" && !isFormLocked;
  const recentRecords = recentQuery.data?.items ?? [];
  const hasInputChanges =
    values.outRows.length > 0 ||
    values.inRows.length > 0 ||
    values.transferDate !== initialValues.transferDate;
  const unsavedChanges = useUnsavedChangesWarning(hasInputChanges && !isFormLocked);

  useEffect(() => {
    if (recentQuery.isError) {
      toast.error("最近の詰め替え・切り分けの取得に失敗しました", { id: "transfer-recent-error" });
    } else if (recentQuery.isSuccess) {
      toast.dismiss("transfer-recent-error");
    }
  }, [recentQuery.isError, recentQuery.isSuccess]);

  function updateSearch(direction: TransferDirection, patch: Partial<SideSearchState>) {
    setSearch((prev) => ({ ...prev, [direction]: { ...prev[direction], ...patch } }));
  }

  function updateValues(updater: (prev: TransferFormValues) => TransferFormValues) {
    setValues((prev) => {
      const next = updater(prev);
      setErrors((current) => {
        const nextErrors = { ...current };
        // 入力が変わった行の error は外す（値の同じ行は残す）
        if (current.rows !== undefined) {
          const retained = Object.fromEntries(
            Object.entries(current.rows).filter(([key]) =>
              SIDES.some((direction) => {
                const before = rowsOf(prev, direction).find(
                  (row) => rowErrorKey(direction, row.productCode) === key,
                );
                const after = rowsOf(next, direction).find(
                  (row) => rowErrorKey(direction, row.productCode) === key,
                );
                return before !== undefined && after?.quantity === before.quantity;
              }),
            ),
          );
          if (Object.keys(retained).length > 0) nextErrors.rows = retained;
          else delete nextErrors.rows;
        }
        if (hasBothSides(next)) delete nextErrors.items;
        return nextErrors;
      });
      if (failedSignature !== null && buildTransferSignature(next) !== failedSignature) {
        setIdempotencyKey(createTransferIdempotencyKey());
        setFailedSignature(null);
      }
      return next;
    });
    setResult(null);
  }

  function addProduct(direction: TransferDirection, product: ProductWithRelations) {
    if (isFormLockedRef.current) return;
    updateSearch(direction, { text: "", candidates: [], message: null });
    updateValues((prev) => {
      const added = addProductToTransfer(prev, direction, product);
      // UI-16-D3: もう片方の表にある商品は足さず、この欄の近くに案内を出す
      if (added.values === null) updateSearch(direction, { message: added.blockedMessage });
      return added.values ?? prev;
    });
    window.setTimeout(() => searchInputRefs[direction].current?.focus(), 0);
  }

  async function handleProductSearch(direction: TransferDirection, keywordOverride?: string) {
    const keyword = (keywordOverride ?? search[direction].text).trim();
    // 保存の click から pending の render までは isFormLocked がまだ false なので、lock は ref で読む（U7）
    if (keyword === "" || isFormLockedRef.current) return;
    const generation = searchGenerationRef.current;
    const isStale = () => generation !== searchGenerationRef.current || isFormLockedRef.current;
    updateSearch(direction, { message: null, candidates: [] });
    try {
      const products = await unwrapResult(
        commands.searchProducts({ ...PRODUCT_SEARCH_QUERY, keyword }),
        { source: "commands", cmd: "search_products" },
      );
      if (isStale()) return;
      if (products.items.length === 0) {
        updateSearch(direction, { message: "該当する商品がありません" });
        return;
      }
      if (products.items.length === 1) {
        addProduct(direction, products.items[0]);
        return;
      }
      updateSearch(direction, {
        candidates: products.items,
        message: `候補から${TRANSFER_SIDE_LABELS[direction]}を選んでください`,
      });
    } catch (error) {
      if (isStale()) return;
      updateSearch(direction, { message: describeError(error) });
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
    setSearch({ out: EMPTY_SEARCH, in: EMPTY_SEARCH });
    setFailedSignature(null);
    setResult(null);
    setIdempotencyKey(createTransferIdempotencyKey());
    window.setTimeout(() => searchInputRefs.out.current?.focus(), 0);
  }

  function renderSide(direction: TransferDirection) {
    const rows = rowsOf(values, direction);
    const sideSearch = search[direction];
    const inputId = `transfer-${direction}-product-search`;
    return (
      <section key={direction} className="space-y-4 rounded-md border p-4">
        <h2 className="text-xl font-semibold">{SIDE_HEADINGS[direction]}</h2>
        <div className="flex flex-wrap items-end gap-2">
          <div className="min-w-[18rem] flex-1 space-y-2">
            <Label htmlFor={inputId}>商品コード・JAN・商品名で追加</Label>
            <ProductAddSuggest
              controller={suggests[direction]}
              onComposedDigitsCommit={(normalized) => {
                void handleProductSearch(direction, normalized);
              }}
            >
              <Input
                ref={searchInputRefs[direction]}
                id={inputId}
                value={sideSearch.text}
                disabled={isFormLocked}
                placeholder="商品コード・JAN・商品名を入力"
                aria-label={`${TRANSFER_SIDE_LABELS[direction]}の検索`}
                onChange={(event) => {
                  updateSearch(direction, { text: event.target.value });
                }}
                onKeyDown={(event) => {
                  if (event.nativeEvent.isComposing) return;
                  if (event.key === "Enter") {
                    event.preventDefault();
                    void handleProductSearch(direction);
                  }
                }}
              />
            </ProductAddSuggest>
          </div>
          <Button
            type="button"
            variant="outline"
            disabled={isFormLocked}
            aria-label={`${TRANSFER_SIDE_LABELS[direction]}の追加`}
            onClick={() => {
              void handleProductSearch(direction);
            }}
          >
            <Search aria-hidden="true" />
            追加
          </Button>
        </div>

        {sideSearch.message !== null ? (
          <div className="flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
            <span>{sideSearch.message}</span>
            {sideSearch.message === "該当する商品がありません" ? (
              <>
                <span>
                  未登録商品の場合は、商品マスタに登録してから詰め替え・切り分けへ戻って追加します。
                </span>
                {hasInputChanges ? (
                  <span className="text-warning-emphasis">
                    未保存の詰め替え・切り分けの内容があります。商品登録へ進むとこの画面の入力は残りません。
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

        {sideSearch.candidates.length > 0 ? (
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
                {sideSearch.candidates.map((candidate) => (
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
                          addProduct(direction, candidate);
                        }}
                      >
                        <Plus aria-hidden="true" />
                        {TRANSFER_SIDE_LABELS[direction]}に追加
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        ) : null}

        {rows.length === 0 ? (
          <EmptyState
            title={`${TRANSFER_SIDE_LABELS[direction]}がありません`}
            description="商品コード・JAN・商品名で検索して明細を追加してください"
          />
        ) : (
          <div className="rounded-md border">
            <Table aria-label={SIDE_HEADINGS[direction]}>
              <TableHeader>
                <TableRow>
                  <TableHead>商品コード</TableHead>
                  <TableHead>商品名</TableHead>
                  <TableHead>部門</TableHead>
                  <TableHead>現在庫</TableHead>
                  <TableHead>数量</TableHead>
                  <TableHead className="text-right">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {rows.map((row) => {
                  const errorKey = rowErrorKey(direction, row.productCode);
                  return (
                    <TableRow key={row.productCode}>
                      <TableCell className="font-medium">{row.productCode}</TableCell>
                      <TableCell>{row.productName}</TableCell>
                      <TableCell>{row.departmentName}</TableCell>
                      <TableCell>
                        {formatStockDisplay(row.currentStockQuantity, row.stockUnit)}
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
                            aria-invalid={errors.rows?.[errorKey] !== undefined}
                            className="w-24"
                            onChange={(event) => {
                              updateValues((prev) =>
                                updateTransferRowQuantity(
                                  prev,
                                  direction,
                                  row.productCode,
                                  event.target.value,
                                ),
                              );
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
                            updateValues((prev) =>
                              removeTransferRow(prev, direction, row.productCode),
                            );
                          }}
                        >
                          <Trash2 aria-hidden="true" />
                        </Button>
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
            {rows.some((row) => errors.rows?.[rowErrorKey(direction, row.productCode)]) ? (
              <div className="space-y-1 border-t p-3 text-sm text-destructive">
                {rows.map((row) => {
                  const message = errors.rows?.[rowErrorKey(direction, row.productCode)];
                  return message === undefined ? null : (
                    <p key={row.productCode}>
                      {row.productCode}: {message}
                    </p>
                  );
                })}
              </div>
            ) : null}
          </div>
        )}
      </section>
    );
  }

  return (
    <PageShell>
      <UnsavedChangesDialog warning={unsavedChanges} />
      <PageHeader
        title="詰め替え・切り分け"
        subtitle="仕入れた品を別の商品にしたとき、減らす商品と増やす商品の数を記録します。原価は変わりません"
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
            <h2 className="text-xl font-semibold">
              詰め替え・切り分けを記録しました（記録 #{result.record_id}）
            </h2>
            {result.idempotent_replay ? (
              <Badge variant="outline">同じ内容の再送として処理済み</Badge>
            ) : null}
          </div>
          {SIDES.map((direction) => (
            <div key={direction} className="space-y-1">
              <h3 className="font-semibold">{RESULT_HEADINGS[direction]}</h3>
              <ul className="space-y-1 text-sm">
                {rowsOf(values, direction).map((row) => {
                  const quantity = parseQuantityInput(
                    row.quantity,
                    row.stockUnit,
                    ITEM_QUANTITY_FIELD,
                  ).value;
                  return (
                    <li key={row.productCode} className="flex flex-wrap items-center gap-2">
                      <span className="font-medium">{row.productCode}</span>
                      <span>{row.productName}</span>
                      <span className="tabular-nums">
                        {quantity === null ? "—" : formatStockDisplay(quantity, row.stockUnit)}
                      </span>
                      <Button asChild variant="link" className="h-auto p-0">
                        <Link to="/products" search={{ q: row.productCode }}>
                          {row.productCode} の在庫を見る
                        </Link>
                      </Button>
                    </li>
                  );
                })}
              </ul>
            </div>
          ))}
          <div className="text-sm">
            <span className="text-muted-foreground">在庫警告</span>{" "}
            <span className="font-medium">{result.stock_warnings.length} 件</span>
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
              続けて詰め替え・切り分け
            </Button>
            <Button asChild type="button" variant="outline">
              <Link
                to="/inventory/transfer/records/$recordId"
                params={{ recordId: String(result.record_id) }}
                search={{ returnTo }}
              >
                <Eye aria-hidden="true" />
                詳細を見る
              </Link>
            </Button>
          </div>
        </section>
      ) : null}

      <section className="space-y-4 rounded-md border p-4">
        <div className="space-y-2">
          <Label htmlFor="transfer-date">詰め替え・切り分けた日</Label>
          <Input
            id="transfer-date"
            type="date"
            value={values.transferDate}
            disabled={isFormLocked}
            aria-invalid={errors.transferDate !== undefined}
            onChange={(event) => {
              updateValues((prev) => ({ ...prev, transferDate: event.target.value }));
            }}
          />
          {errors.transferDate !== undefined ? (
            <p className="text-sm text-destructive">{errors.transferDate}</p>
          ) : null}
        </div>
      </section>

      {SIDES.map((direction) => renderSide(direction))}

      <section className="space-y-2">
        {!bothSides && result === null ? (
          <p className="text-right text-sm text-muted-foreground">{BOTH_SIDES_REQUIRED_MESSAGE}</p>
        ) : null}
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
              suggests.out.invalidateAndClose();
              suggests.in.invalidateAndClose();
              setSaveError(null);
              createMutation.mutate();
            }}
          >
            <ArrowLeftRight aria-hidden="true" />
            {isSaving ? "詰め替え・切り分けを記録しています" : "詰め替え・切り分けを保存"}
          </Button>
        </div>
      </section>

      <section className="space-y-3 rounded-md border p-4">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <h2 className="text-xl font-semibold">最近の詰め替え・切り分け</h2>
          <Button asChild variant="outline" size="sm">
            <Link to="/inventory/records" search={{ recordType: "transfer_record" }}>
              すべての履歴を見る
            </Link>
          </Button>
        </div>
        <p className="text-sm text-muted-foreground">
          直近 10 件の詰め替え・切り分けを新しい順に表示します。
        </p>
        {recentQuery.isLoading ? (
          <div className="space-y-2">
            <Skeleton className="h-9 w-full" />
            <Skeleton className="h-9 w-full" />
          </div>
        ) : recentQuery.isError ? (
          <Alert variant="destructive">
            <AlertTitle>最近の詰め替え・切り分けを取得できません</AlertTitle>
            <AlertDescription>
              保存は可能です。必要に応じて画面を開き直してください。
            </AlertDescription>
          </Alert>
        ) : recentRecords.length === 0 ? (
          <EmptyState
            title="最近の詰め替え・切り分けはありません"
            description="保存するとここに最新の記録が表示されます"
          />
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>日付</TableHead>
                <TableHead>記録ID</TableHead>
                <TableHead>代表商品</TableHead>
                <TableHead>記録日時</TableHead>
                <TableHead className="text-right">操作</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {recentRecords.map((record) => (
                <TableRow key={record.record_id}>
                  <TableCell>{record.business_date}</TableCell>
                  <TableCell>{record.record_id}</TableCell>
                  <TableCell>{record.representative_item}</TableCell>
                  <TableCell className="font-mono tabular-nums">
                    {formatDateTime(record.created_at)}
                  </TableCell>
                  <TableCell className="text-right">
                    <Button asChild variant="outline" size="sm">
                      <Link
                        to="/inventory/transfer/records/$recordId"
                        params={{ recordId: String(record.record_id) }}
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
