// src/features/products/ProductFormPage.tsx
//
// UI-01b 商品登録・修正 page。

import { useEffect, useRef, useState } from "react";
import { flushSync } from "react-dom";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { AlertTriangle, ClipboardCheck } from "lucide-react";
import { toast } from "sonner";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { PageHeader } from "@/components/patterns/PageHeader";
import { UnsavedChangesDialog } from "@/components/patterns/UnsavedChangesDialog";
import { PageShell } from "@/components/patterns/PageShell";
import { useUnsavedChangesWarning } from "@/hooks/useUnsavedChangesWarning";
import { commands, type ProductCreateRequest, type ProductWithRelations } from "@/lib/bindings";
import { describeError } from "@/lib/describe-error";
import { invalidateByContract, invalidationContract } from "@/lib/invalidation-contract";
import { isInvokeError, unwrapResult } from "@/lib/invoke";
import { queryKeys } from "@/lib/query-keys";
import { ProductForm } from "./components/ProductForm";
import { SharedJanCustomCodeDialog } from "./components/SharedJanCustomCodeDialog";
import { useProductFormOptions } from "./hooks/useProductFormOptions";
import {
  buildCreateProductRequest,
  buildUpdateProductRequest,
  createProductFormDefaults,
  productToFormValues,
  type ProductFormValues,
} from "./lib/product-form-request";
import { sanitizeProductListReturnTo } from "./lib/return-to";

export interface ProductFormPageProps {
  mode: "create" | "edit";
  productCode?: string;
  returnTo?: string;
  onNavigateToList: (returnTo: string) => void;
  /** 51 の D-118 の節: 在庫連動を始める拒否（`recount_required`）の「今の数を入れる」の導線（73 の入口） */
  onNavigateToStocktake?: (search: { product: string; returnTo: string }) => void;
}

/** 51 の D-118 の節の見出しと文（在庫連動を始める拒否）。画面の語で書き、BIZ の message は出さない */
const RECOUNT_REQUIRED_TITLE = "「POS販売で在庫を減らす」にできません";
const RECOUNT_REQUIRED_TEXT =
  "「減らさない」の間の売上が在庫の数に入っているか確認できません。「今の数を入れる」で棚の数を入れてから、もう一度「減らす」にして保存してください";

export function ProductFormPage({
  mode,
  productCode,
  returnTo,
  onNavigateToList,
  onNavigateToStocktake,
}: ProductFormPageProps) {
  const queryClient = useQueryClient();
  const safeReturnTo = sanitizeProductListReturnTo(returnTo);
  const { departmentsQuery, suppliersQuery, departments, suppliers } = useProductFormOptions();
  const [values, setValues] = useState<ProductFormValues>(createProductFormDefaults);
  const [savedValues, setSavedValues] = useState<ProductFormValues>(createProductFormDefaults);
  const [formErrors, setFormErrors] = useState<Partial<Record<keyof ProductFormValues, string>>>(
    {},
  );
  const [saveError, setSaveError] = useState<string | null>(null);
  // recount_required の回復の導線を出すか（回復先は action によらず 73 の「今の数を入れる」）
  const [recountRequired, setRecountRequired] = useState(false);
  const recountNoticeRef = useRef<HTMLDivElement>(null);
  const [posSyncTouched, setPosSyncTouched] = useState(false);
  const [pluTargetTouched, setPluTargetTouched] = useState(false);
  const [currentProduct, setCurrentProduct] = useState<ProductWithRelations | null>(null);
  // UI-01b-D26: JAN を他の商品が使っているときの知らせと、送り直しに使う 1 回目の request
  const [sharedJan, setSharedJan] = useState<{
    message: string;
    request: ProductCreateRequest;
  } | null>(null);

  // 案内は保存の button から離れた欄の下に出るので、案内の領域へ focus を移す（51）
  useEffect(() => {
    if (recountRequired) recountNoticeRef.current?.focus();
  }, [recountRequired]);

  const productQuery = useQuery({
    queryKey: queryKeys.productForm.product(productCode ?? ""),
    queryFn: () =>
      unwrapResult(commands.getProduct(productCode ?? ""), {
        source: "commands",
        cmd: "get_product",
      }),
    enabled: mode === "edit" && productCode !== undefined,
    staleTime: 0,
    gcTime: 5 * 60_000,
    retry: 0,
  });

  useEffect(() => {
    if (mode === "create") {
      setValues(createProductFormDefaults);
      setSavedValues(createProductFormDefaults);
      setCurrentProduct(null);
      setPluTargetTouched(false);
      return;
    }
    if (productQuery.data !== undefined) {
      const nextValues = productToFormValues(productQuery.data);
      setCurrentProduct(productQuery.data);
      setValues(nextValues);
      setSavedValues(nextValues);
      setPluTargetTouched(false);
    }
  }, [mode, productQuery.data]);

  const createMutation = useMutation({
    mutationFn: (request: ProductCreateRequest) =>
      unwrapResult(commands.createProduct(request), {
        source: "commands",
        cmd: "create_product",
      }),
    onSuccess: async (result, request) => {
      setSaveError(null);
      setSharedJan(null);
      // UI-USW-D1: 保存成功後の画面遷移を離脱ガードで遮らないよう、先に基準値を同期する。
      flushSync(() => {
        setSavedValues({ ...values });
      });
      // UI-01b-D14: 保存成功 toast は navigate より前に発火する。UI-01b-D26: 外した商品は 1 件 1 行で説明に添える
      toast.success(
        `商品「${values.name.trim()}」を登録しました（商品コード: ${result.product_code}）`,
        {
          id: "product-save-success",
          duration: 5000,
          description:
            result.warnings.length > 0 ? (
              <ul>
                {result.warnings.map((warning) => (
                  <li key={warning}>{warning}</li>
                ))}
              </ul>
            ) : undefined,
        },
      );
      if (request.shared_jan_custom_code) {
        // C23（D-141）: 外した商品の修正の画面と要再確認の一覧も古くなる
        await invalidateByContract(queryClient, invalidationContract.productCreateSharedJan());
      } else {
        await invalidateByContract(queryClient, invalidationContract.productCreate());
      }
      onNavigateToList(safeReturnTo);
    },
    onError: (error, request) => {
      // 型で分岐し message を解析しない。field の無い duplicate は通常の error として出す（40、UI-01b-D26）
      if (
        !request.shared_jan_custom_code &&
        isInvokeError(error) &&
        error.cmdError.kind === "duplicate" &&
        error.cmdError.field === "jan_code"
      ) {
        // 1 段目は BIZ の message（BIZ-01-D11 が画面の語で作る）。表示は describeError を通す（UI-ERR-D1）
        setSharedJan({ message: describeError(error), request });
        return;
      }
      setSharedJan(null);
      setSaveError(describeError(error));
    },
  });

  const submitCreate = () => {
    const built = buildCreateProductRequest(values, departments);
    setFormErrors(built.errors);
    if (built.request !== null) createMutation.mutate(built.request);
  };

  const updateMutation = useMutation({
    mutationFn: async () => {
      if (currentProduct === null || productCode === undefined) {
        throw new Error("商品が見つかりません");
      }
      const built = buildUpdateProductRequest(values, currentProduct);
      setFormErrors(built.errors);
      if (built.request === null) throw new Error("validation");
      return unwrapResult(commands.updateProduct(productCode, built.request), {
        source: "commands",
        cmd: "update_product",
      });
    },
    onSuccess: async () => {
      setSaveError(null);
      // UI-USW-D1: 保存成功後の画面遷移を離脱ガードで遮らないよう、先に基準値を同期する。
      flushSync(() => {
        setSavedValues({ ...values });
      });
      // UI-01b-D14: 保存成功 toast は navigate より前に発火する
      toast.success(`商品「${values.name.trim()}」を保存しました`, {
        id: "product-save-success",
        duration: 5000,
      });
      await invalidateByContract(
        queryClient,
        invalidationContract.productUpdate(productCode ?? ""),
      );
      onNavigateToList(safeReturnTo);
    },
    onError: (error) => {
      if (error instanceof Error && error.message === "validation") return;
      // 型で分岐し message を解析しない。未知の code は通常の error として出す（40）
      if (
        isInvokeError(error) &&
        error.cmdError.kind === "stocktake_guard" &&
        error.cmdError.stocktake_recovery?.code === "recount_required"
      ) {
        setRecountRequired(true);
        return;
      }
      setSaveError(describeError(error));
    },
  });

  const toggleMutation = useMutation({
    mutationFn: async () => {
      if (productCode === undefined) throw new Error("商品が見つかりません");
      return unwrapResult(commands.toggleDiscontinue(productCode), {
        source: "commands",
        cmd: "toggle_discontinue",
      });
    },
    onSuccess: async () => {
      await productQuery.refetch();
      await invalidateByContract(
        queryClient,
        invalidationContract.productUpdate(productCode ?? ""),
      );
    },
    onError: (error) => {
      setSaveError(describeError(error));
    },
  });

  const title = mode === "create" ? "商品登録" : "商品修正";
  const supplierWarning = suppliersQuery.isError
    ? "取引先を指定しない登録・保存は続行できます。"
    : null;
  const saveDisabled = departmentsQuery.isError || departmentsQuery.isLoading;
  const isDirty = (Object.keys(values) as (keyof ProductFormValues)[]).some(
    (field) => values[field] !== savedValues[field],
  );
  const unsavedChanges = useUnsavedChangesWarning(isDirty);
  const sharedJanDepartment =
    sharedJan === null
      ? undefined
      : departments.find((department) => department.id === sharedJan.request.department_id);

  if (mode === "edit" && productQuery.isLoading) {
    return (
      <PageShell>
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-32 w-full" />
      </PageShell>
    );
  }

  if (mode === "edit" && productQuery.isError) {
    return (
      <PageShell>
        <PageHeader title={title} />
        <Alert variant="destructive">
          <AlertTitle>商品が見つかりません</AlertTitle>
          <AlertDescription>商品一覧へ戻って、もう一度選択してください。</AlertDescription>
        </Alert>
        <Button
          type="button"
          variant="outline"
          onClick={() => {
            onNavigateToList(safeReturnTo);
          }}
        >
          一覧へ戻る
        </Button>
      </PageShell>
    );
  }

  return (
    <PageShell>
      <UnsavedChangesDialog warning={unsavedChanges} />
      <SharedJanCustomCodeDialog
        open={sharedJan !== null}
        message={sharedJan?.message ?? ""}
        departmentName={sharedJanDepartment?.name ?? ""}
        canAssignCustomCode={sharedJanDepartment?.code_prefix != null}
        isPending={createMutation.isPending}
        onConfirm={() => {
          if (sharedJan === null) return;
          // 同じ入力の値で、在庫連動の checkbox によらず在庫連動なしで送り直す（UI-01b-D26）
          createMutation.mutate({
            ...sharedJan.request,
            shared_jan_custom_code: true,
            pos_stock_sync: false,
          });
        }}
        onCancel={() => {
          setSharedJan(null);
        }}
      />
      <PageHeader title={title} />
      {departmentsQuery.isError ? (
        <Alert variant="destructive">
          <AlertTitle>部門候補を取得できませんでした</AlertTitle>
          <AlertDescription>部門は必須項目のため、この状態では保存できません。</AlertDescription>
        </Alert>
      ) : null}
      {mode === "edit" ? (
        <div className="max-w-sm space-y-2">
          <label htmlFor="plu-memory-no" className="text-sm font-medium">
            レジメモリNo.
          </label>
          <input
            id="plu-memory-no"
            className="h-10 w-full rounded-md border bg-muted px-3 text-sm"
            value={currentProduct?.plu_memory_no ?? "未割当"}
            readOnly
          />
        </div>
      ) : null}
      <ProductForm
        mode={mode}
        values={values}
        departments={departments}
        suppliers={suppliers}
        errors={formErrors}
        saveError={saveError}
        supplierWarning={supplierWarning}
        productCodeLabel={currentProduct?.product_code}
        productName={currentProduct?.name}
        isDiscontinued={currentProduct?.is_discontinued ?? false}
        stockRecheck={currentProduct?.stock_recheck ?? false}
        stockSyncNotice={
          recountRequired && productCode !== undefined ? (
            <Alert
              ref={recountNoticeRef}
              variant="warning"
              aria-label={RECOUNT_REQUIRED_TITLE}
              tabIndex={-1}
            >
              <AlertTriangle aria-hidden="true" />
              <AlertTitle>{RECOUNT_REQUIRED_TITLE}</AlertTitle>
              <AlertDescription className="space-y-3">
                <p>{RECOUNT_REQUIRED_TEXT}</p>
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => {
                    // route の遷移なので離脱の確認（UI-USW-D1）を通る。baseline を揃えない
                    onNavigateToStocktake?.({
                      product: productCode,
                      returnTo: `/products/${productCode}/edit`,
                    });
                  }}
                >
                  <ClipboardCheck aria-hidden="true" />
                  今の数を入れる
                </Button>
              </AlertDescription>
            </Alert>
          ) : undefined
        }
        isSaving={createMutation.isPending || updateMutation.isPending}
        isTogglePending={toggleMutation.isPending}
        saveDisabled={saveDisabled}
        posSyncTouched={posSyncTouched}
        pluTargetTouched={pluTargetTouched}
        showPluTargetEnableNote={currentProduct?.plu_target === false && values.pluTarget}
        onValuesChange={setValues}
        onPosSyncTouchedChange={setPosSyncTouched}
        onPluTargetTouchedChange={setPluTargetTouched}
        onSubmit={() => {
          setSaveError(null);
          setRecountRequired(false);
          if (mode === "create") {
            submitCreate();
          } else {
            updateMutation.mutate();
          }
        }}
        onCancel={() => {
          onNavigateToList(safeReturnTo);
        }}
        onToggleDiscontinue={() => {
          toggleMutation.mutate();
        }}
      />
    </PageShell>
  );
}
