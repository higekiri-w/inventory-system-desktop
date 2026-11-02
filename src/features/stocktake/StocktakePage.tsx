import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
  AlertTriangle,
  CalendarDays,
  CheckCircle2,
  ClipboardCheck,
  Info,
  Loader2,
  RotateCcw,
  Search,
  X,
} from "lucide-react";
import { useEffect, useRef, useState, type ReactNode, type RefObject } from "react";

import { DepartmentFilter } from "@/components/patterns/DepartmentFilter";
import { EmptyState } from "@/components/patterns/EmptyState";
import { FormSection } from "@/components/patterns/FormSection";
import { PageHeader } from "@/components/patterns/PageHeader";
import {
  ProductAddSuggest,
  type ProductAddSuggestController,
} from "@/components/patterns/ProductAddSuggest";
import { useProductAddSuggest } from "@/components/patterns/useProductAddSuggest";
import { PageShell } from "@/components/patterns/PageShell";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Progress } from "@/components/ui/progress";
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
import {
  formatLengthM,
  formatLengthOrPlain,
  formatStockDisplay,
} from "@/features/stock-inquiry/lib/format-stock-display";
import {
  commands,
  type CountRecoveryTarget,
  type LastStocktakeSummary,
  type ProductStockUnit,
  type ProductWithRelations,
  type RecountReason,
  type SettlementPendingHint,
  type StockCountKind,
  type StockCountSaveResult,
  type StocktakeItemDetail,
  type StocktakeRecovery,
  type StocktakeResult,
} from "@/lib/bindings";
import { describeError } from "@/lib/describe-error";
import { isInvokeError, unwrapResult } from "@/lib/invoke";
import { invalidateByContract, invalidationContract } from "@/lib/invalidation-contract";
import { scrollPageToTop } from "@/lib/page-scroll";
import { queryKeys } from "@/lib/query-keys";
import { getLocalDateString } from "@/lib/request-helpers";
import { parseQuantityInput } from "@/lib/unit-quantity";
import { Pagination, PaginationSummary } from "@/components/patterns/Pagination";
import { LIST_PER_PAGE_OPTIONS } from "@/components/patterns/list-per-page";

import { RecheckTargetList } from "./components/RecheckTargetList";
import { useCompleteStocktake } from "./hooks/useCompleteStocktake";
import {
  formatCountedAt,
  formatListDifference,
  formatRecountReasons,
  formatSettlementGap,
  formatStockOrRange,
  STOCK_COUNT_FIELD,
} from "./lib/stocktake-formatters";
import { useFindStocktakeItem } from "./hooks/useFindStocktakeItem";
import { useLastCompletedStocktake } from "./hooks/useLastCompletedStocktake";
import { useRecheckProducts } from "./hooks/useRecheckProducts";
import { useSaveStockCount } from "./hooks/useSaveStockCount";
import { useStocktakeItems } from "./hooks/useStocktakeItems";
import { useStocktakeStatus } from "./hooks/useStocktakeStatus";
import {
  refreshStocktakeItemsAfterValidation,
  refreshStocktakeStateAfterConflict,
} from "./stocktake-error-invalidation";
import type { StocktakeSearch } from "./types";

interface StocktakePageProps {
  search: StocktakeSearch;
  onSearchChange: (updater: (prev: StocktakeSearch) => StocktakeSearch) => void;
  /** 73「今の数を入れる」: 保存の後に `returnTo` へ戻る（route の遷移） */
  onReturnTo?: (to: string) => void;
}

function formatYen(value: number): string {
  return `¥${value.toLocaleString("ja-JP")}`;
}

function formatLastStocktake(last: LastStocktakeSummary | null | undefined): string {
  if (!last) {
    return "前回の記録はありません";
  }
  return `前回の棚卸し（${formatCountedAt(last.completed_at)}）: 仕入原価総額 ${formatYen(last.total_cost)}`;
}

function isStocktakeNotInProgressError(error: unknown): boolean {
  return isInvokeError(error) && error.cmdError.kind === "stocktake_not_in_progress";
}

const STOCKTAKE_NOT_IN_PROGRESS_MESSAGE = "この棚卸しは既に完了しています";

/** 73 の「完了」の行: アプリが判定できない最後の精算の取込み忘れを問う（ADR D6 の H2） */
const LAST_SETTLEMENT_QUESTION =
  "今日の最後の精算（締め）を取り込みましたか。取り込む前に完了すると、その精算の売上は棚卸しに入りません";

const PRODUCT_NAME_SEARCH_QUERY = {
  department_id: null,
  is_discontinued: null,
  sort_key: "ProductCode" as const,
  sort_order: "Asc" as const,
  page: 1,
  per_page: 10,
};

/**
 * 73「今の数を入れる」の `returnTo` は `/products/{product と同じ商品コード}/edit` だけを受ける
 * （UI-01b-D2 と同じく許す先を限る）。外れれば null（保存の後も `/stocktake/` に留まる）。
 */
export function allowedReturnTo(
  returnTo: string | undefined,
  product: string | undefined,
): string | null {
  if (returnTo === undefined || product === undefined) return null;
  const allowed = `/products/${product}/edit`;
  return returnTo === allowed || returnTo === `/products/${encodeURIComponent(product)}/edit`
    ? allowed
    : null;
}

/** 計数の欄が扱う商品の文脈（棚卸しの明細・要再確認の対象・placement の読取りのどれからも作る） */
interface CountTarget {
  productCode: string;
  name: string;
  detail: string | null;
  stockUnit: ProductStockUnit;
  stock: number;
  rangeLow: number | null;
  rangeHigh: number | null;
  reasons: RecountReason[];
  countValue: number | null;
  countKind: StockCountKind | null;
  countBusinessDate: string | null;
  stocktakeItemId: number | null;
}

function targetFromItem(item: StocktakeItemDetail): CountTarget {
  return {
    productCode: item.product_code,
    name: item.name,
    detail: item.department_name,
    stockUnit: item.stock_unit,
    stock: item.current_stock,
    rangeLow: item.stock_range_low,
    rangeHigh: item.stock_range_high,
    reasons: item.recount_reasons,
    countValue: item.count_value,
    countKind: item.count_kind,
    countBusinessDate: item.count_business_date,
    stocktakeItemId: item.id,
  };
}

function formatTargetStock(target: CountTarget): string {
  return formatStockOrRange({
    stock: target.stock,
    stock_range_low: target.rangeLow,
    stock_range_high: target.rangeHigh,
    stock_unit: target.stockUnit,
  });
}

/** 「今の数を入れる」の欄の文脈を読む（42 の `preview_stock_count_placement`。読取りだけ） */
async function loadStandaloneTarget(
  productCode: string,
  stocktakeItemId: number | null,
): Promise<CountTarget> {
  const preview = await unwrapResult(
    commands.previewStockCountPlacement(productCode, getLocalDateString()),
    { source: "commands", cmd: "preview_stock_count_placement" },
  );
  return {
    productCode,
    name: preview.product_name,
    detail: null,
    stockUnit: preview.stock_unit,
    stock: preview.stock,
    rangeLow: preview.stock_range_low,
    rangeHigh: preview.stock_range_high,
    reasons: preview.recount_reasons,
    countValue: preview.count_value,
    countKind: preview.count_kind,
    countBusinessDate: preview.count_business_date,
    stocktakeItemId,
  };
}

export function StocktakePage({ search, onSearchChange, onReturnTo }: StocktakePageProps) {
  const queryClient = useQueryClient();
  const stocktakeStatus = useStocktakeStatus();
  const activeStocktakeId = stocktakeStatus.activeStocktakeId;
  const [effectiveSearch, setEffectiveSearch] = useState<StocktakeSearch>(search);
  const [perPage, setPerPage] = useState<(typeof LIST_PER_PAGE_OPTIONS)[number]>(50);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isStarting, setIsStarting] = useState(false);
  const [isZeroing, setIsZeroing] = useState(false);
  const [isConfirmOpen, setIsConfirmOpen] = useState(false);
  const [completedResult, setCompletedResult] = useState<StocktakeResult | null>(null);
  const [blocked, setBlocked] = useState<StocktakeRecovery | null>(null);
  const [standaloneTarget, setStandaloneTarget] = useState<CountTarget | null>(null);
  // 前の保存の日付（画面を開いたまま日付が変わったら保存の前に確かめる。73 の「日付の変わり目」）
  const [lastSaveDay, setLastSaveDay] = useState<string | null>(null);
  const [lastStocktakeSnapshot, setLastStocktakeSnapshot] = useState<
    LastStocktakeSummary | null | undefined
  >(undefined);
  const productSuggestRef = useRef<ProductAddSuggestController | null>(null);
  // 「今の数を入れる」の欄の商品（route が別の商品へ変われば読込みの前からその商品）。遅い保存の応答の照合に使う（73）
  const standaloneCodeRef = useRef<string | null>(null);
  // 今の欄に出ている商品。route の product の読込みが失敗するか route の product が外れたら照合をこれへ戻す（73）
  const standaloneFieldCodeRef = useRef<string | null>(null);
  // 欄の商品の保存が、route の別の商品の読込みの決着より先に成功したときの後始末（閉じる・戻る）。
  // 読込みが失敗するか route の product が外れて欄がその商品のまま決まれば実行し、別の商品の欄が開けば捨てる（73）
  const standalonePendingCloseRef = useRef<{ productCode: string; finish: () => void } | null>(
    null,
  );
  // 欄の読込み（route の product・「入れ直す」）は最後の操作の応答だけを当てる（73）
  const standaloneSeqRef = useRef(0);
  // 保存の成功の後始末を行う時点の route の product と、画面が mount 中か（73）
  const routeProductRef = useRef(search.product);
  const mountedRef = useRef(true);

  useEffect(() => {
    // StrictMode の dev は cleanup の後に effect をもう一度走らせるので、body でも立てる
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
    };
  }, []);

  // route の product の effect より前に置く（その effect が実行する閉じ待ちが新しい route を読む）
  useEffect(() => {
    routeProductRef.current = search.product;
    setEffectiveSearch(search);
  }, [search]);

  useEffect(() => {
    standaloneFieldCodeRef.current = standaloneTarget?.productCode ?? null;
    standaloneCodeRef.current = standaloneFieldCodeRef.current;
    standalonePendingCloseRef.current = null;
  }, [standaloneTarget]);

  const departmentsQuery = useQuery({
    queryKey: queryKeys.stocktake.departments(),
    queryFn: () =>
      unwrapResult(commands.listDepartments(), { source: "commands", cmd: "list_departments" }),
  });
  const recheckQuery = useRecheckProducts();
  const lastCompletedQuery = useLastCompletedStocktake();
  const itemsQuery = useStocktakeItems(activeStocktakeId, effectiveSearch, perPage);
  const findMutation = useFindStocktakeItem();
  const saveMutation = useSaveStockCount();
  const completeMutation = useCompleteStocktake();

  const isCompleting = completeMutation.isPending;
  const itemsData = itemsQuery.data;
  const progress = itemsData?.progress ?? { total_items: 0, counted_items: 0, uncounted_items: 0 };
  const statusLoaded = !stocktakeStatus.isLoading;
  const searchProduct = search.product;

  useEffect(() => {
    if (!itemsQuery.isError || !isStocktakeNotInProgressError(itemsQuery.error)) return;
    setErrorMessage(STOCKTAKE_NOT_IN_PROGRESS_MESSAGE);
    void refreshStocktakeStateAfterConflict(queryClient);
  }, [itemsQuery.error, itemsQuery.isError, queryClient]);

  // 73「今の数を入れる」（商品を受けて開く）: 棚卸しの有無によらず fresh な計数の欄を開く。
  // stocktake_item_id は進行中の棚卸しにその商品の明細があるときだけ送る
  useEffect(() => {
    if (!statusLoaded) return;
    // route の product が外れても欄は残す（途中の読込みの応答は前の cleanup で捨てた）。
    // 欄は今の商品のまま決まるので、その商品の閉じ待ちがあれば実行する（73）
    if (searchProduct === undefined) {
      settleStandaloneField();
      return;
    }
    standaloneCodeRef.current = searchProduct;
    const seq = ++standaloneSeqRef.current;
    const isStale = () => seq !== standaloneSeqRef.current;
    void (async () => {
      try {
        const item =
          activeStocktakeId === null
            ? null
            : await unwrapResult(commands.findStocktakeItem(activeStocktakeId, searchProduct), {
                source: "commands",
                cmd: "find_stocktake_item",
              });
        const itemId = item !== null && item.product_code === searchProduct ? item.id : null;
        const target = await loadStandaloneTarget(searchProduct, itemId);
        if (!isStale()) setStandaloneTarget(target);
      } catch (error) {
        if (!isStale()) {
          setErrorMessage(describeError(error));
          settleStandaloneField();
        }
      }
    })();
    return () => {
      standaloneSeqRef.current += 1;
    };
  }, [searchProduct, activeStocktakeId, statusLoaded]);

  function updateSearch(updater: (prev: StocktakeSearch) => StocktakeSearch) {
    setEffectiveSearch((prev) => updater(prev));
    onSearchChange(updater);
  }

  // 欄が今の商品のまま決まったら（欄の読込み〈route の product・「入れ直す」〉の最後の操作が失敗した、route の product が外れた）、
  // 遅い保存の応答の照合を今の欄の商品へ戻し、その商品の閉じ待ちがあれば実行する（73）
  function settleStandaloneField() {
    standaloneCodeRef.current = standaloneFieldCodeRef.current;
    const pending = standalonePendingCloseRef.current;
    standalonePendingCloseRef.current = null;
    if (pending !== null && pending.productCode === standaloneCodeRef.current) pending.finish();
  }

  async function openRecount(target: CountRecoveryTarget) {
    setErrorMessage(null);
    const seq = ++standaloneSeqRef.current;
    try {
      const loaded = await loadStandaloneTarget(target.product_code, target.stocktake_item_id);
      if (seq === standaloneSeqRef.current) setStandaloneTarget(loaded);
    } catch (error) {
      if (seq === standaloneSeqRef.current) {
        setErrorMessage(describeError(error));
        settleStandaloneField();
      }
    }
  }

  function closeStandalone() {
    setStandaloneTarget(null);
    if (search.product !== undefined || search.returnTo !== undefined) {
      updateSearch((prev) => ({ ...prev, product: undefined, returnTo: undefined }));
    }
  }

  async function handleSaved(productCode: string) {
    setLastSaveDay(getLocalDateString());
    // 片付いた理由（数え直し・未計数）を完了の拒否の一覧から外す（精算の理由は取込みで解ける）
    setBlocked((prev) => {
      if (prev === null) return prev;
      const targets = prev.targets.filter(
        (target) =>
          target.product_code !== productCode || target.block_reason === "settlement_pending",
      );
      return targets.length === 0 ? null : { ...prev, targets };
    });
    await invalidateByContract(queryClient, invalidationContract.stocktakeCountUpdate());
  }

  async function handleStandaloneSaved(productCode: string) {
    const back = allowedReturnTo(search.returnTo, search.product);
    await handleSaved(productCode);
    const finish = () => {
      // 画面を離れた後は何もしない（戻る・閉じるの route の更新は、離れた先から連れ戻す。73）
      if (!mountedRef.current) return;
      // 実行する時点で route の product が外れていれば、保存の時の戻り先へ戻らずに留まる（73）
      if (
        back !== null &&
        productCode === search.product &&
        routeProductRef.current !== undefined
      ) {
        setStandaloneTarget(null);
        onReturnTo?.(back);
        return;
      }
      closeStandalone();
    };
    if (standaloneCodeRef.current === productCode) {
      finish();
      return;
    }
    // 保存の応答の保留中に欄が別の商品へ変わっていれば、その欄（未保存の入力）を閉じない・戻らない。
    // 欄がこの商品のまま別の商品の読込みを待っているなら、その読込みが失敗するか route の product が外れた時点で閉じる
    if (standaloneFieldCodeRef.current === productCode) {
      standalonePendingCloseRef.current = { productCode, finish };
    }
  }

  async function handleStart() {
    if (isStarting) return;
    setErrorMessage(null);
    setIsStarting(true);
    try {
      await unwrapResult(commands.startStocktake(), {
        source: "commands",
        cmd: "start_stocktake",
      });
      await invalidateByContract(queryClient, invalidationContract.stocktakeStart());
    } catch (error) {
      if (isInvokeError(error) && error.cmdError.kind === "stocktake_in_progress") {
        await refreshStocktakeStateAfterConflict(queryClient);
        return;
      }
      setErrorMessage(describeError(error));
    } finally {
      setIsStarting(false);
    }
  }

  async function handleCompleteConfirm() {
    productSuggestRef.current?.invalidateAndClose();
    if (activeStocktakeId === null) return;
    setErrorMessage(null);
    setBlocked(null);
    setIsConfirmOpen(false);
    try {
      const result = await completeMutation.mutateAsync({ stocktakeId: activeStocktakeId });
      setLastStocktakeSnapshot(lastCompletedQuery.data);
      setCompletedResult(result);
      await invalidateByContract(queryClient, invalidationContract.stocktakeComplete());
    } catch (error) {
      if (isStocktakeNotInProgressError(error)) {
        setErrorMessage(STOCKTAKE_NOT_IN_PROGRESS_MESSAGE);
        await refreshStocktakeStateAfterConflict(queryClient);
        return;
      }
      // 完了の拒否は理由ごとの一覧へ（型で分岐し message を解析しない。未知の code は文を出す。40）
      if (
        isInvokeError(error) &&
        error.cmdError.kind === "stocktake_guard" &&
        error.cmdError.stocktake_recovery?.code === "completion_blocked"
      ) {
        setBlocked(error.cmdError.stocktake_recovery);
        void refreshStocktakeItemsAfterValidation(queryClient);
        return;
      }
      // validation（合計金額が大きすぎます ほか）は一覧を invalidate し、次の完了で最新の状態を使う
      void refreshStocktakeItemsAfterValidation(queryClient);
      setErrorMessage(describeError(error));
    }
  }

  async function handleZero(productCodes: string[], businessDate: string) {
    if (activeStocktakeId === null || isZeroing) return;
    setErrorMessage(null);
    setIsZeroing(true);
    try {
      await unwrapResult(
        commands.zeroUncountedItems(activeStocktakeId, productCodes, businessDate),
        { source: "commands", cmd: "zero_uncounted_items" },
      );
      setBlocked((prev) => {
        if (prev === null) return prev;
        const targets = prev.targets.filter(
          (target) =>
            !(target.block_reason === "uncounted" && productCodes.includes(target.product_code)),
        );
        return targets.length === 0 ? null : { ...prev, targets };
      });
      await invalidateByContract(queryClient, invalidationContract.stocktakeCountUpdate());
    } catch (error) {
      if (isStocktakeNotInProgressError(error)) {
        setErrorMessage(STOCKTAKE_NOT_IN_PROGRESS_MESSAGE);
        await refreshStocktakeStateAfterConflict(queryClient);
        return;
      }
      setErrorMessage(describeError(error));
    } finally {
      setIsZeroing(false);
    }
  }

  if (completedResult !== null) {
    return <StocktakeResultPage result={completedResult} lastStocktake={lastStocktakeSnapshot} />;
  }

  // 保存（数・0 にする）・確定のどれかの保留中は、他の保存と確定を止める（73）
  const countDisabled = isCompleting || saveMutation.isPending || isZeroing;

  return (
    <PageShell>
      <PageHeader
        title="棚卸し"
        subtitle="商品コードまたはJANを読み取り、実際の在庫数を入力します"
      />

      {errorMessage !== null ? (
        <Alert variant="destructive">
          <AlertTitle>操作できませんでした</AlertTitle>
          <AlertDescription>{errorMessage}</AlertDescription>
        </Alert>
      ) : null}

      <RecheckTargetList
        title="数え直しが必要な商品"
        targets={recheckQuery.data ?? []}
        renderAction={(target) => (
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={countDisabled}
            onClick={() => void openRecount(target)}
          >
            入れ直す
          </Button>
        )}
      />

      {standaloneTarget !== null ? (
        <FormSection
          title="今の数を入れる"
          description="棚を今見た数を入れます（今日の日付で保存し、前に保存した数を置き換えます）"
        >
          <div className="flex justify-end">
            <Button
              type="button"
              variant="ghost"
              size="sm"
              disabled={countDisabled}
              onClick={closeStandalone}
            >
              <X aria-hidden="true" />
              閉じる
            </Button>
          </div>
          <CountFields
            key={standaloneTarget.productCode}
            target={standaloneTarget}
            freshOnly
            disabled={countDisabled}
            lastSaveDay={lastSaveDay}
            saveMutation={saveMutation}
            onError={setErrorMessage}
            onSaved={() => void handleStandaloneSaved(standaloneTarget.productCode)}
          />
        </FormSection>
      ) : null}

      {stocktakeStatus.isLoading ? (
        <div className="space-y-3">
          <Skeleton className="h-20 w-full" />
          <Skeleton className="h-48 w-full" />
        </div>
      ) : stocktakeStatus.isError ? (
        <Alert variant="destructive">
          <AlertTitle>棚卸し状態を読み込めませんでした</AlertTitle>
          <AlertDescription className="space-y-3">
            <p>{describeError(stocktakeStatus.error)}</p>
            <Button type="button" variant="outline" onClick={() => void stocktakeStatus.refetch()}>
              <RotateCcw />
              再試行
            </Button>
          </AlertDescription>
        </Alert>
      ) : activeStocktakeId === null ? (
        <StocktakeStartPanel
          lastStocktake={lastCompletedQuery.data}
          isLoadingLast={lastCompletedQuery.isLoading}
          isStarting={isStarting}
          onStart={() => void handleStart()}
        />
      ) : (
        <div className="space-y-6">
          <StocktakeProgressHeader
            startedAt={stocktakeStatus.activeStocktake?.started_at ?? ""}
            progress={progress}
          />

          {itemsQuery.isLoading ? (
            <div className="space-y-3">
              <Skeleton className="h-20 w-full" />
              <Skeleton className="h-48 w-full" />
            </div>
          ) : null}

          {itemsQuery.isError ? (
            <Alert variant="destructive">
              <AlertTitle>棚卸し一覧を読み込めませんでした</AlertTitle>
              <AlertDescription className="space-y-3">
                <p>もう一度読み込んでください。</p>
                <Button type="button" variant="outline" onClick={() => void itemsQuery.refetch()}>
                  <RotateCcw />
                  再試行
                </Button>
              </AlertDescription>
            </Alert>
          ) : null}

          <StocktakeCountEntry
            stocktakeId={activeStocktakeId}
            disabled={countDisabled}
            lastSaveDay={lastSaveDay}
            productSuggestRef={productSuggestRef}
            findMutation={findMutation}
            saveMutation={saveMutation}
            onError={setErrorMessage}
            onSaved={(productCode) => void handleSaved(productCode)}
          />

          {blocked !== null ? (
            <CompletionBlockedPanel
              recovery={blocked}
              disabled={countDisabled}
              onRecount={(target) => void openRecount(target)}
              onZero={(codes, businessDate) => void handleZero(codes, businessDate)}
            />
          ) : null}

          <StocktakeItemList
            items={itemsData?.items ?? []}
            departments={departmentsQuery.data ?? []}
            search={effectiveSearch}
            disabled={isCompleting}
            perPage={perPage}
            totalCount={itemsData?.total_count ?? 0}
            onPerPageChange={setPerPage}
            onSearchChange={updateSearch}
          />

          <div className="flex justify-end">
            <Button
              type="button"
              variant="outline"
              disabled={countDisabled}
              onClick={() => {
                setIsConfirmOpen(true);
              }}
            >
              {isCompleting ? <Loader2 className="animate-spin" /> : <ClipboardCheck />}
              棚卸しを確定する
            </Button>
          </div>

          {isCompleting ? (
            <p className="text-sm text-muted-foreground" role="status">
              確定しています
            </p>
          ) : null}

          <StocktakeCompleteDialog
            open={isConfirmOpen}
            disabled={countDisabled}
            onOpenChange={setIsConfirmOpen}
            onConfirm={() => void handleCompleteConfirm()}
          />
        </div>
      )}
    </PageShell>
  );
}

interface StocktakeStartPanelProps {
  lastStocktake: LastStocktakeSummary | null | undefined;
  isLoadingLast: boolean;
  isStarting: boolean;
  onStart: () => void;
}

export function StocktakeStartPanel({
  lastStocktake,
  isLoadingLast,
  isStarting,
  onStart,
}: StocktakeStartPanelProps) {
  return (
    <FormSection
      title="棚卸しの開始"
      description="開始すると、現在の商品マスタから棚卸し対象を作成します"
    >
      <div className="space-y-4">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">前回の棚卸し</CardTitle>
          </CardHeader>
          <CardContent>
            {isLoadingLast ? (
              <Skeleton className="h-5 w-72" />
            ) : (
              <p>{formatLastStocktake(lastStocktake)}</p>
            )}
          </CardContent>
        </Card>
        <Button type="button" onClick={onStart} disabled={isStarting}>
          {isStarting ? <Loader2 className="animate-spin" /> : <ClipboardCheck />}
          棚卸しを開始する
        </Button>
      </div>
    </FormSection>
  );
}

interface StocktakeProgressHeaderProps {
  startedAt: string;
  progress: { total_items: number; counted_items: number; uncounted_items: number };
}

export function StocktakeProgressHeader({ startedAt, progress }: StocktakeProgressHeaderProps) {
  const percent =
    progress.total_items > 0 ? (progress.counted_items / progress.total_items) * 100 : 0;
  return (
    <div className="space-y-2">
      <div className="space-y-1">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <h2 className="min-w-0 flex-1 text-xl font-semibold">
            棚卸し中（開始日: {formatCountedAt(startedAt)}）
          </h2>
          <Badge
            variant="outline"
            tone={progress.uncounted_items > 0 ? "warning" : "success"}
            className="shrink-0"
          >
            {progress.uncounted_items > 0 ? (
              <AlertTriangle aria-hidden="true" />
            ) : (
              <CheckCircle2 aria-hidden="true" />
            )}
            未入力 {progress.uncounted_items}
          </Badge>
        </div>
        <p className="text-sm text-muted-foreground">
          入力済み {progress.counted_items} / 全 {progress.total_items}
        </p>
      </div>
      <Progress value={percent} aria-label="棚卸し進捗" />
    </div>
  );
}

interface StocktakeCountEntryProps {
  stocktakeId: number;
  disabled: boolean;
  lastSaveDay: string | null;
  productSuggestRef: RefObject<ProductAddSuggestController | null>;
  findMutation: ReturnType<typeof useFindStocktakeItem>;
  saveMutation: ReturnType<typeof useSaveStockCount>;
  onError: (message: string | null) => void;
  onSaved: (productCode: string) => void;
}

export function StocktakeCountEntry({
  stocktakeId,
  disabled,
  lastSaveDay,
  productSuggestRef,
  findMutation,
  saveMutation,
  onError,
  onSaved,
}: StocktakeCountEntryProps) {
  const [code, setCode] = useState("");
  const [selectedItem, setSelectedItem] = useState<StocktakeItemDetail | null>(null);
  // 選び直すたびに欄（数量・日付・FieldError）を作り直す
  const [selection, setSelection] = useState(0);
  const [targetMessage, setTargetMessage] = useState<string | null>(null);
  const [savedMessage, setSavedMessage] = useState<string | null>(null);
  const [candidates, setCandidates] = useState<ProductWithRelations[]>([]);
  // 遅い保存の応答が別の商品の欄を消さないよう、応答の商品と今の欄の商品を照合する（73）
  const selectedCodeRef = useRef<string | null>(null);
  // 検索・候補の選択は最後の操作の応答だけを当てる（遅い応答が後の欄を置き換えない。73）
  const lookupSeqRef = useRef(0);
  const codeInputRef = useRef<HTMLInputElement>(null);
  const quantityInputRef = useRef<HTMLInputElement>(null);
  const productSuggest = useProductAddSuggest({
    value: code,
    isLocked: () => disabled,
    onSelect: (product) => selectCandidate(product.product_code),
    queryOverrides: { is_discontinued: null },
  });

  useEffect(() => {
    productSuggestRef.current = productSuggest;
    return () => {
      productSuggestRef.current = null;
    };
  }, [productSuggest, productSuggestRef]);

  useEffect(() => {
    codeInputRef.current?.focus({ preventScroll: true });
  }, []);

  useEffect(() => {
    selectedCodeRef.current = selectedItem?.product_code ?? null;
  }, [selectedItem]);

  function selectItem(item: StocktakeItemDetail) {
    setSelectedItem(item);
    setSelection((prev) => prev + 1);
    setCandidates([]);
    setTargetMessage(null);
    setSavedMessage(null);
    window.setTimeout(() => quantityInputRef.current?.focus(), 0);
  }

  async function resolveItem(codeOverride?: string) {
    const trimmed = (codeOverride ?? code).trim();
    if (trimmed.length === 0 || disabled) return;
    setTargetMessage(null);
    setSavedMessage(null);
    setCandidates([]);
    onError(null);
    const seq = ++lookupSeqRef.current;
    const isStale = () => seq !== lookupSeqRef.current;
    try {
      const item = await findMutation.mutateAsync({ stocktakeId, code: trimmed });
      if (isStale()) return;
      if (item !== null) {
        selectItem(item);
        return;
      }
      // 商品コード/JAN の完全一致で見つからない場合、商品名検索にフォールバックする（UI-10-D2/§73.5）
      const products = await unwrapResult(
        commands.searchProducts({ ...PRODUCT_NAME_SEARCH_QUERY, keyword: trimmed }),
        { source: "commands", cmd: "search_products" },
      );
      if (isStale()) return;
      if (products.items.length === 0) {
        setSelectedItem(null);
        setTargetMessage(
          "この商品は棚卸しの対象にありません。商品コードまたはJANを確認してください。新しく登録した商品は自動で追加されます",
        );
        return;
      }
      if (products.items.length === 1) {
        const resolved = await findMutation.mutateAsync({
          stocktakeId,
          code: products.items[0].product_code,
        });
        if (resolved !== null && !isStale()) selectItem(resolved);
        return;
      }
      setCandidates(products.items);
    } catch (error) {
      if (!isStale()) onError(describeError(error));
    }
  }

  async function selectCandidate(productCode: string) {
    if (disabled) return;
    onError(null);
    const seq = ++lookupSeqRef.current;
    try {
      const resolved = await findMutation.mutateAsync({ stocktakeId, code: productCode });
      if (resolved !== null && seq === lookupSeqRef.current) selectItem(resolved);
    } catch (error) {
      if (seq === lookupSeqRef.current) onError(describeError(error));
    }
  }

  return (
    <FormSection
      title="カウント入力"
      description="商品コードまたはJANを読み取り、実際の数を保存します"
    >
      <fieldset
        disabled={disabled}
        className="grid gap-4 disabled:opacity-70 md:grid-cols-[1fr_auto]"
      >
        <div className="space-y-2">
          <Label htmlFor="stocktake-code">商品を検索・スキャン</Label>
          <ProductAddSuggest
            controller={productSuggest}
            onComposedDigitsCommit={(normalized) => {
              void resolveItem(normalized);
            }}
          >
            <Input
              id="stocktake-code"
              ref={codeInputRef}
              value={code}
              placeholder="商品コード・JAN・商品名を入力"
              onChange={(event) => {
                setCode(event.target.value);
              }}
              onKeyDown={(event) => {
                if (event.nativeEvent.isComposing) return;
                if (event.key === "Enter") {
                  event.preventDefault();
                  void resolveItem();
                }
              }}
            />
          </ProductAddSuggest>
        </div>
        <div className="flex items-end">
          <Button type="button" variant="outline" onClick={() => void resolveItem()}>
            {findMutation.isPending ? <Loader2 className="animate-spin" /> : <Search />}
            対象を確認
          </Button>
        </div>
      </fieldset>

      {targetMessage !== null ? (
        <p className="mt-2 flex items-center gap-1.5 text-sm text-muted-foreground" role="status">
          <Info aria-hidden="true" className="size-4" />
          {targetMessage}
        </p>
      ) : null}

      {savedMessage !== null ? (
        <p className="mt-2 flex items-center gap-1.5 text-sm" role="status">
          <CheckCircle2 aria-hidden="true" className="size-4" />
          {savedMessage}
        </p>
      ) : null}

      {candidates.length > 0 ? (
        <div className="mt-2 space-y-2">
          <p className="text-sm text-muted-foreground">候補から商品を選んでください</p>
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
                    <TableCell>
                      {candidate.name}
                      {candidate.is_discontinued ? (
                        <Badge variant="secondary" className="ml-2">
                          廃番
                        </Badge>
                      ) : null}
                    </TableCell>
                    <TableCell>{candidate.department_name}</TableCell>
                    <TableCell className="text-right">
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        disabled={disabled}
                        onClick={() => void selectCandidate(candidate.product_code)}
                      >
                        選択
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        </div>
      ) : null}

      {selectedItem !== null ? (
        <CountFields
          key={selection}
          target={targetFromItem(selectedItem)}
          freshOnly={false}
          disabled={disabled}
          lastSaveDay={lastSaveDay}
          saveMutation={saveMutation}
          quantityInputRef={quantityInputRef}
          onError={onError}
          onSaved={(result, businessDate, countValue) => {
            const reasons = formatRecountReasons(result.recount_reasons);
            setSavedMessage(
              `${selectedItem.name} を ${formatStockDisplay(countValue, selectedItem.stock_unit)}（${businessDate}）で保存しました${
                reasons.length > 0 ? `。要再確認: ${reasons.join("。")}` : ""
              }`,
            );
            onSaved(selectedItem.product_code);
            if (selectedCodeRef.current !== selectedItem.product_code) return;
            setCode("");
            setSelectedItem(null);
            window.setTimeout(() => codeInputRef.current?.focus(), 0);
          }}
        />
      ) : null}
    </FormSection>
  );
}

interface CountFieldsProps {
  target: CountTarget;
  /** 要再確認の一覧・「今の数を入れる」から開いた欄は fresh な計数だけ（日付は今日） */
  freshOnly: boolean;
  disabled: boolean;
  lastSaveDay: string | null;
  saveMutation: ReturnType<typeof useSaveStockCount>;
  quantityInputRef?: RefObject<HTMLInputElement | null>;
  onError: (message: string | null) => void;
  onSaved: (result: StockCountSaveResult, businessDate: string, countValue: number) => void;
}

/**
 * 73 の D-118 の節の計数の欄: 「数を保存」（既定の計数。日付は既定で今日、「前の日の分を入れる」で選ぶ）と
 * 「入れ直す」（fresh な計数。日付は今日だけ）。保存の前に日付の変わり目を確かめる。
 */
function CountFields({
  target,
  freshOnly,
  disabled,
  lastSaveDay,
  saveMutation,
  quantityInputRef,
  onError,
  onSaved,
}: CountFieldsProps) {
  const queryClient = useQueryClient();
  const [quantity, setQuantity] = useState("");
  const [fieldError, setFieldError] = useState<string | null>(null);
  const [backDate, setBackDate] = useState<string | null>(null);
  const [laterCountExists, setLaterCountExists] = useState(false);
  const [pendingDateChange, setPendingDateChange] = useState<{
    fresh: boolean;
    value: number;
    previous: string;
    today: string;
  } | null>(null);
  // 日付の preview は最後に選んだ日付の応答だけを当てる（73）
  const previewSeqRef = useRef(0);
  const today = getLocalDateString();
  const isLength = target.stockUnit === "m";

  async function chooseBackDate(value: string) {
    const seq = ++previewSeqRef.current;
    setBackDate(value);
    setLaterCountExists(false);
    if (value === "" || value > getLocalDateString()) return;
    try {
      const placement = await unwrapResult(
        commands.previewStockCountPlacement(target.productCode, value),
        { source: "commands", cmd: "preview_stock_count_placement" },
      );
      if (seq === previewSeqRef.current) setLaterCountExists(placement.later_count_exists);
    } catch (error) {
      if (seq === previewSeqRef.current) onError(describeError(error));
    }
  }

  async function doSave(fresh: boolean, value: number, businessDate: string) {
    onError(null);
    try {
      const result = await saveMutation.mutateAsync({
        product_code: target.productCode,
        count_value: value,
        fresh,
        business_date: businessDate,
        stocktake_item_id: target.stocktakeItemId,
      });
      setQuantity("");
      onSaved(result, businessDate, value);
    } catch (error) {
      if (isStocktakeNotInProgressError(error)) {
        onError(STOCKTAKE_NOT_IN_PROGRESS_MESSAGE);
        void refreshStocktakeStateAfterConflict(queryClient);
        return;
      }
      onError(describeError(error));
    }
  }

  function saveCount(fresh: boolean) {
    if (disabled || saveMutation.isPending) return;
    if (quantity.trim() === "") {
      setFieldError("数量を入力してください");
      return;
    }
    const parsed = parseQuantityInput(quantity, target.stockUnit, STOCK_COUNT_FIELD);
    if (parsed.error !== null) {
      setFieldError(parsed.error);
      return;
    }
    const now = getLocalDateString();
    const businessDate = fresh || backDate === null || backDate === "" ? now : backDate;
    if (businessDate > now) {
      setFieldError("未来の日付には保存できません");
      return;
    }
    setFieldError(null);
    // 前の保存から PC の日付が変わったら、今日の分として保存するかを確かめる（前の日の分は選んだ日付）
    if (businessDate === now && lastSaveDay !== null && lastSaveDay !== now) {
      setPendingDateChange({ fresh, value: parsed.value, previous: lastSaveDay, today: now });
      return;
    }
    void doSave(fresh, parsed.value, businessDate);
  }

  const isBackDated = !freshOnly && backDate !== null;
  const shownDate = isBackDated && backDate !== "" ? backDate : today;
  // 同じ日付の違う値の既定の計数は要再確認（理由 3）になるので、数を直す導線は「入れ直す」（73 の D-118 の節）
  const canRecount = !freshOnly && !isBackDated && target.countValue !== null;

  return (
    <div className="mt-4 grid gap-4 md:grid-cols-[1fr_12rem_auto]">
      <div className="space-y-1">
        <p className="font-medium">{target.name}</p>
        <p className="text-sm text-muted-foreground">
          {target.productCode}
          {target.detail !== null ? ` / ${target.detail}` : ""} / 現在在庫{" "}
          {formatTargetStock(target)}
          {target.rangeLow !== null ? (
            <Badge variant="outline" tone="warning" className="ml-2">
              <AlertTriangle aria-hidden="true" />
              要再確認
            </Badge>
          ) : null}
        </p>
        {target.reasons.length > 0 ? (
          <p className="text-sm">{formatRecountReasons(target.reasons).join("。")}</p>
        ) : null}
        {target.countValue !== null ? (
          <p className="text-sm text-muted-foreground">
            前に保存した数: {formatStockDisplay(target.countValue, target.stockUnit)}（
            {target.countBusinessDate}
            {target.countKind === "fresh" ? "、入れ直し" : ""}）
          </p>
        ) : null}
        {canRecount ? (
          <p className="text-sm text-muted-foreground">
            数を直すときは「入れ直す」を押してください。同じ日付で「数を保存」から違う数を入れると、数え直しが必要になります
          </p>
        ) : null}
        <p className="flex flex-wrap items-center gap-2 text-sm">
          <CalendarDays aria-hidden="true" className="size-4" />
          保存の日付: {shownDate}
          {shownDate === today ? "（今日）" : ""}
          {!freshOnly && !isBackDated ? (
            <Button
              type="button"
              variant="outline"
              size="sm"
              disabled={disabled}
              onClick={() => {
                setBackDate("");
              }}
            >
              前の日の分を入れる
            </Button>
          ) : null}
        </p>
        {isBackDated ? (
          <div className="flex flex-wrap items-end gap-2">
            <div className="space-y-1">
              <Label htmlFor={`stocktake-back-date-${target.productCode}`}>数えた日</Label>
              <Input
                id={`stocktake-back-date-${target.productCode}`}
                type="date"
                max={today}
                value={backDate}
                disabled={disabled}
                onChange={(event) => void chooseBackDate(event.target.value)}
              />
            </div>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              disabled={disabled}
              onClick={() => {
                previewSeqRef.current += 1;
                setBackDate(null);
                setLaterCountExists(false);
              }}
            >
              今日の分に戻す
            </Button>
          </div>
        ) : null}
        {isBackDated && laterCountExists ? (
          <p className="flex items-center gap-1.5 text-sm" role="status">
            <Info aria-hidden="true" className="size-4" />
            後の数が既にあり、そちらが効きます
          </p>
        ) : null}
      </div>
      <div className="space-y-2">
        <Label htmlFor="stocktake-actual-count">
          実際の数{isLength ? "（m、小数 2 桁まで）" : ""}
        </Label>
        <Input
          id="stocktake-actual-count"
          ref={quantityInputRef}
          value={quantity}
          inputMode={isLength ? "decimal" : "numeric"}
          disabled={disabled}
          onChange={(event) => {
            setQuantity(event.target.value);
          }}
          onKeyDown={(event) => {
            if (event.nativeEvent.isComposing) return;
            if (event.key === "Enter") {
              event.preventDefault();
              saveCount(freshOnly);
            }
          }}
        />
        <div className="min-h-5" aria-live="polite">
          {fieldError !== null ? (
            <p className="text-sm text-destructive" role="alert">
              {fieldError}
            </p>
          ) : null}
        </div>
      </div>
      <div className="flex flex-col items-start gap-2">
        {/* Gated Amendment 1（S3 追加是正、案 a）: 数量欄 column（Label + Input + FieldError slot）と
            同じ行構造を持たせる。invisible な Label 高さ分の spacer でボタン上端を Input 上端に揃え、
            下に min-h-5 の空 slot を置いて行高さも対称にする。 */}
        <Label className="invisible" aria-hidden="true">
          実際の数
        </Label>
        <Button
          type="button"
          disabled={disabled || saveMutation.isPending}
          onClick={() => {
            saveCount(freshOnly);
          }}
        >
          {saveMutation.isPending ? <Loader2 className="animate-spin" /> : <CheckCircle2 />}
          {freshOnly ? "今の数を保存" : "数を保存"}
        </Button>
        {canRecount ? (
          <Button
            type="button"
            variant="outline"
            disabled={disabled || saveMutation.isPending}
            onClick={() => {
              saveCount(true);
            }}
          >
            <RotateCcw aria-hidden="true" />
            入れ直す
          </Button>
        ) : null}
        <div className="min-h-5" aria-hidden="true" />
      </div>

      <AlertDialog
        open={pendingDateChange !== null}
        onOpenChange={(open) => {
          if (!open) setPendingDateChange(null);
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>日付が変わりました</AlertDialogTitle>
            <AlertDialogDescription>
              {pendingDateChange !== null
                ? `日付が変わりました（${pendingDateChange.previous} → ${pendingDateChange.today}）。今日の分として保存しますか`
                : ""}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>キャンセル</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                const pending = pendingDateChange;
                setPendingDateChange(null);
                if (pending !== null) void doSave(pending.fresh, pending.value, pending.today);
              }}
            >
              今日の分として保存する
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

/** 差異の符号付きの表示（長さの商品は m の形「-0.5 m」。SPEC-UNIT-D3） */
function formatDifference(difference: number | null, unit: ProductStockUnit): string {
  if (difference === null || unit !== "m") return formatListDifference(difference);
  return `${difference > 0 ? "+" : ""}${formatLengthM(difference)}`;
}

interface StocktakeItemListProps {
  items: StocktakeItemDetail[];
  departments: { id: number; name: string }[];
  search: StocktakeSearch;
  disabled: boolean;
  perPage: (typeof LIST_PER_PAGE_OPTIONS)[number];
  totalCount: number;
  onPerPageChange: (value: (typeof LIST_PER_PAGE_OPTIONS)[number]) => void;
  onSearchChange: (updater: (prev: StocktakeSearch) => StocktakeSearch) => void;
}

export function StocktakeItemList({
  items,
  departments,
  search,
  disabled,
  perPage,
  totalCount,
  onPerPageChange,
  onSearchChange,
}: StocktakeItemListProps) {
  const page = search.page ?? 1;
  // filter-empty reset action（catalog ⑥、SPEC-UIBB-1/2）: 部門フィルタ / 未入力のみ toggle の
  // いずれかが既定値以外か。
  const isFilterDefault = search.dept === undefined && search.counted_only === undefined;
  // S5（round 1 是正）: Pagination.tsx と同じ式で totalPages を計算し、totalPages<=1 の
  // ときは <fieldset> 自体を描画しない（中身の無い空 <fieldset> が残るのを防ぐ）。
  // Codex P1 是正（PR #40）: 最終ページの未入力アイテムを保存すると invalidate 後に
  // totalPages が縮み page が範囲外に残ることがある。共有 src/components/patterns/Pagination.tsx の Pagination の早期 return はこの
  // page > totalPages を「前のページ」の回復条件として許容しているため、外側の
  // gate もこの条件を含めないと回復導線が消える。
  const totalPages = Math.max(1, Math.ceil(totalCount / perPage));

  return (
    <FormSection
      title="棚卸し一覧"
      description="一覧は進捗確認用です。カウント入力は上の入力欄で行います"
    >
      <div className="flex flex-wrap items-end gap-3 rounded-lg border bg-card p-4">
        <DepartmentFilter
          options={departments}
          selected={search.dept ?? null}
          disabled={disabled}
          idPrefix="stocktake-dept-filter"
          onChange={(dept) => {
            onSearchChange((prev) => ({ ...prev, dept: dept ?? undefined, page: 1 }));
          }}
        />
        <label
          htmlFor="stocktake-uncounted-only"
          className="flex h-9 items-center gap-2 self-end text-sm"
        >
          <Checkbox
            id="stocktake-uncounted-only"
            checked={search.counted_only === false}
            disabled={disabled}
            onCheckedChange={(checked) => {
              onSearchChange((prev) => ({
                ...prev,
                counted_only: checked === true ? false : undefined,
                page: 1,
              }));
            }}
          />
          未入力のみ表示
        </label>
        <div className="grid gap-1">
          <Label className="font-normal text-muted-foreground" htmlFor="stocktake-per-page">
            表示件数
          </Label>
          <Select
            value={String(perPage)}
            disabled={disabled}
            onValueChange={(value) => {
              const next = LIST_PER_PAGE_OPTIONS.find((option) => String(option) === value);
              if (next === undefined) return;
              onPerPageChange(next);
              onSearchChange((prev) => ({ ...prev, page: 1 }));
              scrollPageToTop();
            }}
          >
            <SelectTrigger id="stocktake-per-page" className="w-[7rem]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {LIST_PER_PAGE_OPTIONS.map((option) => (
                <SelectItem key={option} value={String(option)}>
                  {option} 件
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>

      {items.length === 0 ? (
        <EmptyState
          title="この条件に一致する商品がありません"
          description="部門フィルタまたは未入力のみ表示を解除してください"
          // 絞り込みが既定値以外のときだけ reset action を出す（catalog ⑥、SPEC-UIBB-1/2）。
          action={
            !isFilterDefault ? (
              <Button
                type="button"
                variant="outline"
                disabled={disabled}
                onClick={() => {
                  onSearchChange((prev) => ({
                    ...prev,
                    dept: undefined,
                    counted_only: undefined,
                    page: undefined,
                  }));
                }}
              >
                絞り込みを解除
              </Button>
            ) : undefined
          }
        />
      ) : (
        // S3a（round 2 是正、Opus P3）: この分岐は従来 <Table> を裸で返していたが、上部
        // PaginationSummary を追加するため Fragment で 2 要素をまとめる。
        <>
          {totalCount > 0 ? (
            <PaginationSummary page={page} perPage={perPage} totalCount={totalCount} />
          ) : null}
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>商品コード</TableHead>
                <TableHead>商品名</TableHead>
                <TableHead>部門</TableHead>
                <TableHead className="text-right">現在在庫</TableHead>
                <TableHead className="text-right">実際の数</TableHead>
                <TableHead className="text-right">差異</TableHead>
                <TableHead>最終カウント</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {items.map((item) => {
                // 差異は BIZ が求めた値（35「読取り」）。画面で計算しない
                const diff = item.difference;
                return (
                  <TableRow key={item.id}>
                    <TableCell>{item.product_code}</TableCell>
                    <TableCell>
                      {item.name}
                      {item.is_discontinued ? (
                        <Badge variant="secondary" className="ml-2">
                          廃番
                        </Badge>
                      ) : null}
                    </TableCell>
                    <TableCell>{item.department_name}</TableCell>
                    <TableCell className="text-right">
                      {item.stock_range_low !== null ? (
                        <>
                          {formatStockOrRange({
                            stock: item.current_stock,
                            stock_range_low: item.stock_range_low,
                            stock_range_high: item.stock_range_high,
                            stock_unit: item.stock_unit,
                          })}
                          <Badge variant="outline" tone="warning" className="ml-2">
                            <AlertTriangle aria-hidden="true" />
                            要再確認
                          </Badge>
                        </>
                      ) : (
                        formatLengthOrPlain(item.current_stock, item.stock_unit)
                      )}
                    </TableCell>
                    <TableCell className="text-right">
                      {item.actual_count === null
                        ? "未入力"
                        : formatLengthOrPlain(item.actual_count, item.stock_unit)}
                    </TableCell>
                    <TableCell
                      className={
                        diff === null
                          ? "text-right"
                          : diff > 0
                            ? "text-right text-success-strong"
                            : diff < 0
                              ? "text-right text-destructive-strong"
                              : "text-right text-muted-foreground"
                      }
                    >
                      {formatDifference(diff, item.stock_unit)}
                    </TableCell>
                    {/* 効く計数の業務日。未入力・業務日 null は「—」で、保存の時刻へ戻さない（73 UI-10-D10） */}
                    <TableCell className="text-muted-foreground">
                      {item.actual_count === null || item.count_business_date === null
                        ? "—"
                        : item.count_business_date}
                    </TableCell>
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        </>
      )}

      {totalPages > 1 || page > totalPages ? (
        <fieldset disabled={disabled}>
          <Pagination
            page={page}
            perPage={perPage}
            totalCount={totalCount}
            onPageChange={(nextPage) => {
              onSearchChange((prev) => ({ ...prev, page: nextPage }));
            }}
          />
        </fieldset>
      ) : null}
    </FormSection>
  );
}

interface StocktakeCompleteDialogProps {
  open: boolean;
  /** 確定・保存の保留中 */
  disabled: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => void;
}

export function StocktakeCompleteDialog({
  open,
  disabled,
  onOpenChange,
  onConfirm,
}: StocktakeCompleteDialogProps) {
  const warningTitle = "確定すると取り消せません";
  const bodyText = "入力した内容で棚卸しを確定します。";
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>棚卸しの確定</AlertDialogTitle>
          <AlertDialogDescription className="sr-only">
            {warningTitle}。{bodyText}
            {LAST_SETTLEMENT_QUESTION}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <Alert variant="warning">
          <AlertTriangle />
          <AlertTitle>{warningTitle}</AlertTitle>
          <AlertDescription>
            <p>{bodyText}</p>
            <p>{LAST_SETTLEMENT_QUESTION}</p>
          </AlertDescription>
        </Alert>
        <AlertDialogFooter>
          <AlertDialogCancel disabled={disabled}>キャンセル</AlertDialogCancel>
          <AlertDialogAction variant="destructive" disabled={disabled} onClick={onConfirm}>
            確定する
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

/** 「取り込んでいない精算がある」の直し方の文（40 の `SettlementPendingHint` から 73 の文） */
export function settlementPendingHintLines(hint: SettlementPendingHint | null): string[] {
  if (hint === null) return [];
  return [
    ...hint.missing.map((gap) => `${formatSettlementGap(gap)} を取り込んでください`),
    ...(hint.on_or_after !== null
      ? [`精算日が ${hint.on_or_after} 以降の精算を取り込んでください`]
      : []),
    ...(hint.before !== null
      ? [`精算日が ${hint.before} より前の精算を 1 本取り込んでください`]
      : []),
  ];
}

interface CompletionBlockedPanelProps {
  recovery: StocktakeRecovery;
  disabled: boolean;
  onRecount: (target: CountRecoveryTarget) => void;
  onZero: (productCodes: string[], businessDate: string) => void;
}

/** 73 の「完了」: completion_blocked を block_reason ごとの一覧で出す（理由を 1 つの一覧に混ぜない） */
function CompletionBlockedPanel({
  recovery,
  disabled,
  onRecount,
  onZero,
}: CompletionBlockedPanelProps) {
  const [zeroDate, setZeroDate] = useState(getLocalDateString());
  const [confirmZeroAll, setConfirmZeroAll] = useState(false);
  const recheck = recovery.targets.filter((target) => target.block_reason === "recheck");
  const uncounted = recovery.targets.filter((target) => target.block_reason === "uncounted");
  const pending = recovery.targets.filter((target) => target.block_reason === "settlement_pending");
  const hintLines = settlementPendingHintLines(recovery.settlement_pending);
  const today = getLocalDateString();

  return (
    <section
      aria-label="棚卸しを確定できない理由"
      className="space-y-4 rounded-md border border-destructive p-4"
    >
      <h2 className="flex items-center gap-1.5 text-lg font-semibold">
        <AlertTriangle aria-hidden="true" className="size-5" />
        棚卸しを確定できません。次の理由を片付けてください
      </h2>

      {recheck.length > 0 ? (
        <BlockGroup title="数え直しが必要">
          {recheck.map((target) => (
            <li key={target.product_code} className="flex flex-wrap items-center gap-2">
              <span className="font-medium">{target.product_name}</span>
              <span className="font-mono text-sm text-muted-foreground">{target.product_code}</span>
              <span className="tabular-nums">{formatStockOrRange(target)}</span>
              <span className="text-sm">
                {formatRecountReasons(target.recount_reasons).join("。")}
              </span>
              <Button
                type="button"
                variant="outline"
                size="sm"
                disabled={disabled}
                onClick={() => {
                  onRecount(target);
                }}
              >
                入れ直す
              </Button>
            </li>
          ))}
        </BlockGroup>
      ) : null}

      {uncounted.length > 0 ? (
        <BlockGroup title="まだ数えていない">
          <li className="flex flex-wrap items-end gap-2">
            <div className="space-y-1">
              <Label htmlFor="stocktake-zero-date">0 にする日付</Label>
              <Input
                id="stocktake-zero-date"
                type="date"
                max={today}
                value={zeroDate}
                disabled={disabled}
                onChange={(event) => {
                  setZeroDate(event.target.value);
                }}
              />
            </div>
            <Button
              type="button"
              variant="outline"
              disabled={disabled || zeroDate === "" || zeroDate > today}
              onClick={() => {
                setConfirmZeroAll(true);
              }}
            >
              残りをまとめて 0 にする
            </Button>
          </li>
          {uncounted.map((target) => (
            <li key={target.product_code} className="flex flex-wrap items-center gap-2">
              <span className="font-medium">{target.product_name}</span>
              <span className="font-mono text-sm text-muted-foreground">{target.product_code}</span>
              <Button
                type="button"
                variant="outline"
                size="sm"
                disabled={disabled || zeroDate === "" || zeroDate > today}
                onClick={() => {
                  onZero([target.product_code], zeroDate);
                }}
              >
                0 にする
              </Button>
            </li>
          ))}
        </BlockGroup>
      ) : null}

      {pending.length > 0 ? (
        <BlockGroup title="取り込んでいない精算がある">
          {hintLines.map((line) => (
            <li key={line} className="font-medium">
              {line}
            </li>
          ))}
          {pending.map((target) => (
            <li key={target.product_code} className="flex flex-wrap items-center gap-2">
              <span>{target.product_name}</span>
              <span className="font-mono text-sm text-muted-foreground">{target.product_code}</span>
            </li>
          ))}
        </BlockGroup>
      ) : null}

      <AlertDialog open={confirmZeroAll} onOpenChange={setConfirmZeroAll}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>残りをまとめて 0 にしますか</AlertDialogTitle>
            <AlertDialogDescription>
              まだ数えていない {uncounted.length}件を、{zeroDate} の数 0 として保存します。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>キャンセル</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                setConfirmZeroAll(false);
                onZero(
                  uncounted.map((target) => target.product_code),
                  zeroDate,
                );
              }}
            >
              0 にする
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </section>
  );
}

function BlockGroup({ title, children }: { title: string; children: ReactNode }) {
  return (
    <div className="space-y-2" role="group" aria-label={title}>
      <h3 className="flex items-center gap-1.5 font-semibold">
        <AlertTriangle aria-hidden="true" className="size-4" />
        {title}
      </h3>
      <ul className="space-y-2 pl-1">{children}</ul>
    </div>
  );
}

interface StocktakeResultPageProps {
  result: StocktakeResult;
  lastStocktake: LastStocktakeSummary | null | undefined;
}

export function StocktakeResultPage({ result, lastStocktake }: StocktakeResultPageProps) {
  return (
    <PageShell>
      <PageHeader title="棚卸し結果" />
      <Card>
        <CardHeader>
          <CardTitle>仕入原価総額</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
          <p className="text-3xl font-semibold">{formatYen(result.total_cost)}</p>
          <p>明細の数: {result.total_items.toLocaleString("ja-JP")}件</p>
          <p className="flex items-center gap-1.5 text-sm text-muted-foreground">
            <Info aria-hidden="true" className="size-4" />
            確定では在庫は動きません（数を保存したときに在庫へ入っています）
          </p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">
            {lastStocktake
              ? `前回の棚卸し（${formatCountedAt(lastStocktake.completed_at)}）`
              : "前回の棚卸し"}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-base font-semibold">
            {lastStocktake
              ? `仕入原価総額 ${formatYen(lastStocktake.total_cost)}`
              : "前回の記録はありません"}
          </p>
        </CardContent>
      </Card>

      <FormSection title="整合性チェック">
        {result.integrity_result === null ? (
          <p>整合性チェックは実行できませんでした</p>
        ) : (
          <p>
            {result.integrity_result.checked_count}件を確認しました。不整合{" "}
            {result.integrity_result.mismatch_count}件
          </p>
        )}
      </FormSection>
    </PageShell>
  );
}
