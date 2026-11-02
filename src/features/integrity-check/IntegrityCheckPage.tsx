import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import {
  AlertTriangle,
  CheckCircle2,
  ClipboardCheck,
  Info,
  Loader2,
  RotateCcw,
  ShieldCheck,
} from "lucide-react";
import { useMemo, useState } from "react";

import { LIST_PER_PAGE_OPTIONS } from "@/components/patterns/list-per-page";
import { PageHeader } from "@/components/patterns/PageHeader";
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
import { Progress } from "@/components/ui/progress";
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
import { Pagination, PaginationSummary } from "@/components/patterns/Pagination";
import { formatLengthOrPlain } from "@/features/stock-inquiry/lib/format-stock-display";
import {
  commands,
  type IntegrityField,
  type IntegrityFixResult,
  type IntegrityResult,
  type ProductStockUnit,
  type StockCopy,
} from "@/lib/bindings";
import { formatDateTime } from "@/lib/date-time";
import { describeError } from "@/lib/describe-error";
import { invalidateByContract, invalidationContract } from "@/lib/invalidation-contract";
import { unwrapResult } from "@/lib/invoke";
import { scrollPageToTop } from "@/lib/page-scroll";
import { queryKeys } from "@/lib/query-keys";

type IntegrityPhase = "idle" | "running" | "completed";
type PendingOperation = "check" | "fix" | null;
interface OperationError {
  operation: "check" | "fix";
  message: string;
}

// 個数の商品は今どおり 3 桁区切りの数、長さの商品は m の形（SPEC-UNIT-D3 の表の 8）
function groupedNumber(value: number): string {
  return value.toLocaleString("ja-JP");
}

// 75 / 36: 違う項目は文字で示す（色だけで示さない）
const FIELD_LABELS: Record<IntegrityField, string> = {
  quantity: "在庫数",
  recheck: "要再確認",
  range: "範囲",
};

function fieldsLabel(fields: IntegrityField[]): string {
  return fields.map((field) => FIELD_LABELS[field]).join("・");
}

// 在庫数と、要再確認なら後ろに「（要再確認 {low}〜{high}）」（75、OperationLogsPage と同じ書き方）
function formatStockCopy(
  copy: StockCopy,
  unit: ProductStockUnit,
  plain?: (value: number) => string,
): string {
  const quantity = formatLengthOrPlain(copy.stock_quantity, unit, plain);
  if (!copy.stock_recheck || copy.stock_range_low === null || copy.stock_range_high === null)
    return quantity;
  const low = formatLengthOrPlain(copy.stock_range_low, unit, plain);
  const high = formatLengthOrPlain(copy.stock_range_high, unit, plain);
  return `${quantity}（要再確認 ${low}〜${high}）`;
}

// 75 冒頭の注記: 移行（vL）が写しを直した integrity_fix の行（新しい順の最新 100 行の中で最新のもの）
const MIGRATION_NOTICE_QUERY = {
  page: 1,
  per_page: 100,
  operation_type: "integrity_fix",
  start_date: null,
  end_date: null,
};

interface MigrationNotice {
  createdAt: string;
  fixedCount: number;
}

function findMigrationNotice(
  items: { detail_json: string | null; created_at: string }[],
): MigrationNotice | null {
  for (const item of items) {
    if (!item.detail_json) continue;
    try {
      const detail: unknown = JSON.parse(item.detail_json);
      if (!detail || typeof detail !== "object" || Array.isArray(detail)) continue;
      const { trigger, fixed_count: fixedCount } = detail as Record<string, unknown>;
      if (
        trigger === "migration" &&
        typeof fixedCount === "number" &&
        Number.isSafeInteger(fixedCount)
      )
        return { createdAt: item.created_at, fixedCount };
    } catch {
      continue;
    }
  }
  return null;
}

export function IntegrityCheckPage() {
  const queryClient = useQueryClient();
  const [phase, setPhase] = useState<IntegrityPhase>("idle");
  const [pendingOperation, setPendingOperation] = useState<PendingOperation>(null);
  const [result, setResult] = useState<IntegrityResult | null>(null);
  const [selectedCodes, setSelectedCodes] = useState<Set<string>>(() => new Set());
  const [page, setPage] = useState(1);
  const [perPage, setPerPage] = useState<(typeof LIST_PER_PAGE_OPTIONS)[number]>(100);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [fixResult, setFixResult] = useState<IntegrityFixResult | null>(null);
  const [fixedCodes, setFixedCodes] = useState<Set<string>>(() => new Set());
  const [operationError, setOperationError] = useState<OperationError | null>(null);

  const latestCheckQuery = useQuery({
    queryKey: queryKeys.operationLogs.latestIntegrityCheck(),
    queryFn: () =>
      unwrapResult(
        commands.listLogs({
          page: 1,
          per_page: 1,
          operation_type: "integrity_check",
          start_date: null,
          end_date: null,
        }),
        { source: "commands", cmd: "list_logs" },
      ),
    staleTime: 0,
    gcTime: 300_000,
    retry: 0,
  });

  // 読取りに失敗したら知らせを出さず、チェックの操作は止めない（75）
  const migrationNoticeQuery = useQuery({
    queryKey: queryKeys.operationLogs.list(MIGRATION_NOTICE_QUERY),
    queryFn: () =>
      unwrapResult(commands.listLogs(MIGRATION_NOTICE_QUERY), {
        source: "commands",
        cmd: "list_logs",
      }),
    staleTime: 0,
    gcTime: 300_000,
    retry: 0,
  });
  const migrationNotice = migrationNoticeQuery.data
    ? findMigrationNotice(migrationNoticeQuery.data.items)
    : null;

  const isBusy = pendingOperation !== null;
  const mismatches = useMemo(() => result?.mismatches ?? [], [result]);
  const visibleMismatches = useMemo(
    () => mismatches.slice((page - 1) * perPage, page * perPage),
    [mismatches, page, perPage],
  );
  const selectedMismatches = useMemo(
    () => mismatches.filter((item) => selectedCodes.has(item.product_code)),
    [mismatches, selectedCodes],
  );

  async function handleCheck() {
    if (isBusy) return;
    const previousPhase = phase;
    setPendingOperation("check");
    setPhase("running");
    setResult(null);
    setSelectedCodes(new Set());
    setPage(1);
    setFixResult(null);
    setFixedCodes(new Set());
    setOperationError(null);
    try {
      const nextResult = await unwrapResult(commands.runIntegrityCheck(), {
        source: "commands",
        cmd: "run_integrity_check",
      });
      setResult(nextResult);
      setPhase("completed");
      void latestCheckQuery.refetch();
    } catch (error) {
      setPhase(previousPhase);
      setOperationError({
        operation: "check",
        message: describeError(error, "処理を完了できませんでした。もう一度お試しください。"),
      });
    } finally {
      setPendingOperation(null);
    }
  }

  async function handleFix() {
    if (isBusy || selectedCodes.size === 0) return;
    const productCodes = Array.from(selectedCodes);
    setDialogOpen(false);
    setPendingOperation("fix");
    setPhase("running");
    setOperationError(null);
    try {
      const nextFixResult = await unwrapResult(commands.fixIntegrity(productCodes), {
        source: "commands",
        cmd: "fix_integrity",
      });
      const adjustedCodes = new Set(
        nextFixResult.adjustments.map((adjustment) => adjustment.product_code),
      );
      setFixResult(nextFixResult);
      setFixedCodes((current) => new Set([...current, ...adjustedCodes]));
      setSelectedCodes(
        (current) => new Set(Array.from(current).filter((code) => !adjustedCodes.has(code))),
      );
      await invalidateByContract(queryClient, invalidationContract.integrityFix());
    } catch (error) {
      setOperationError({
        operation: "fix",
        message: describeError(error, "処理を完了できませんでした。もう一度お試しください。"),
      });
    } finally {
      setPhase("completed");
      setPendingOperation(null);
    }
  }

  function toggleSelected(code: string, checked: boolean) {
    setSelectedCodes((current) => {
      const next = new Set(current);
      if (checked) next.add(code);
      else next.delete(code);
      return next;
    });
  }

  let latestCheckText = "読み込み中です";
  if (latestCheckQuery.isError) latestCheckText = "取得できませんでした";
  else if (latestCheckQuery.data)
    latestCheckText = latestCheckQuery.data.items[0]
      ? formatDateTime(latestCheckQuery.data.items[0].created_at)
      : "まだ実行されていません";

  return (
    <PageShell className="relative">
      <div className="space-y-2">
        <PageHeader
          title="在庫整合性検証"
          actions={
            <Button type="button" disabled={isBusy} onClick={() => void handleCheck()}>
              {phase === "completed" ? (
                <RotateCcw aria-hidden="true" />
              ) : (
                <ShieldCheck aria-hidden="true" />
              )}
              {phase === "completed" ? "再度チェック" : "整合性チェック実行"}
            </Button>
          }
        />
        <p className="text-sm text-muted-foreground">
          保存している在庫数と、入出庫などの記録から求め直した在庫数を照合し、差異を確認します。
        </p>
        <p className="text-sm font-medium">直近の確認日時: {latestCheckText}</p>
      </div>

      {migrationNotice ? (
        <Alert role="status">
          <Info aria-hidden="true" />
          <AlertDescription>
            <p>
              {`更新のときに在庫数を記録から求め直し、${migrationNotice.fixedCount.toLocaleString("ja-JP")}商品の在庫数を直しました（${formatDateTime(migrationNotice.createdAt)}）。直した前後は操作ログで見られます。`}
            </p>
            <Button asChild variant="outline" size="sm">
              <Link to="/settings/logs">操作ログを開く</Link>
            </Button>
          </AlertDescription>
        </Alert>
      ) : null}

      {operationError ? (
        <Alert variant="destructive">
          <AlertTitle>処理を完了できませんでした</AlertTitle>
          <AlertDescription>
            <p>{operationError.message}</p>
            <Button
              type="button"
              variant="outline"
              disabled={isBusy}
              onClick={() =>
                void (operationError.operation === "fix" ? handleFix() : handleCheck())
              }
            >
              <RotateCcw aria-hidden="true" />
              {operationError.operation === "fix" ? "補正を再試行" : "再試行"}
            </Button>
          </AlertDescription>
        </Alert>
      ) : null}

      {phase === "completed" && result !== null ? (
        <div className="grid gap-1">
          <label htmlFor="integrity-check-per-page" className="text-sm text-muted-foreground">
            表示件数
          </label>
          <Select
            value={String(perPage)}
            onValueChange={(value) => {
              const next = LIST_PER_PAGE_OPTIONS.find((option) => String(option) === value);
              if (next === undefined) return;
              setPerPage(next);
              setPage(1);
              scrollPageToTop();
            }}
          >
            <SelectTrigger id="integrity-check-per-page" className="w-[7rem]">
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
      ) : null}

      {phase === "completed" && result !== null && result.shared_jan_conflicts.length > 0 ? (
        <Alert variant="warning" aria-label="共有 JAN の商品">
          <AlertTriangle aria-hidden="true" />
          <AlertTitle>在庫連動の商品と同じ JAN の商品があります</AlertTitle>
          <AlertDescription className="space-y-2">
            <p>
              在庫連動の商品と同じ JAN
              の商品があります。レジの売上で在庫が減りません。どちらかの商品の JAN
              を直すか、在庫連動を外してください。
            </p>
            <ul className="list-disc pl-5">
              {result.shared_jan_conflicts.map((conflict) => (
                <li key={conflict.jan_code}>
                  JAN <span className="font-mono">{conflict.jan_code}</span>:{" "}
                  <span className="font-mono">{conflict.product_codes.join("、")}</span>
                </li>
              ))}
            </ul>
          </AlertDescription>
        </Alert>
      ) : null}

      {phase === "completed" && result !== null ? (
        result.mismatch_count === 0 ? (
          <Alert role="status" className="border-success bg-success-soft text-success">
            <CheckCircle2 aria-hidden="true" className="text-success" />
            <AlertTitle>差異はありません</AlertTitle>
            <AlertDescription>
              {result.checked_count.toLocaleString("ja-JP")}件の商品を確認しました。
            </AlertDescription>
          </Alert>
        ) : (
          <div className="space-y-5">
            <Alert role="status" variant="warning">
              <AlertTriangle aria-hidden="true" />
              <AlertTitle>差異が見つかりました</AlertTitle>
              <AlertDescription>
                {result.mismatch_count.toLocaleString("ja-JP")}件の商品を確認してください。
              </AlertDescription>
            </Alert>

            {fixResult ? (
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <ClipboardCheck aria-hidden="true" className="size-5 text-success" />
                    {fixResult.fixed_count.toLocaleString("ja-JP")}件を補正しました
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-3">
                  {fixResult.adjustments.length > 0 ? (
                    <ul className="divide-y">
                      {fixResult.adjustments.map((adjustment) => (
                        <li
                          key={adjustment.product_code}
                          className="flex flex-wrap items-center justify-between gap-2 py-2"
                        >
                          <span className="font-mono font-medium">{adjustment.product_code}</span>
                          <span>
                            {formatStockCopy(adjustment.before, adjustment.stock_unit)} →{" "}
                            {formatStockCopy(adjustment.after, adjustment.stock_unit)}
                          </span>
                        </li>
                      ))}
                    </ul>
                  ) : (
                    <p className="text-sm text-muted-foreground">補正された商品はありません。</p>
                  )}
                </CardContent>
              </Card>
            ) : null}

            {fixResult && fixResult.skipped_count > 0 ? (
              <Alert variant="warning">
                <AlertTriangle aria-hidden="true" />
                <AlertTitle>一部の商品は補正されませんでした</AlertTitle>
                <AlertDescription>
                  {fixResult.skipped_count.toLocaleString("ja-JP")}
                  件は状態が変わっていたため、補正を見送りました。
                </AlertDescription>
              </Alert>
            ) : null}

            <section aria-labelledby="integrity-difference-heading" className="space-y-4">
              <div className="space-y-1">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <h2
                    id="integrity-difference-heading"
                    className="min-w-0 flex-1 text-xl font-semibold"
                  >
                    差異のある商品
                  </h2>
                  <Button
                    type="button"
                    className="shrink-0"
                    disabled={isBusy || selectedCodes.size === 0}
                    onClick={() => {
                      setDialogOpen(true);
                    }}
                  >
                    補正を確定
                  </Button>
                </div>
                <p className="text-sm text-muted-foreground">
                  補正する商品を行ごとに選び、内容を確認して確定してください。
                </p>
              </div>

              <PaginationSummary page={page} perPage={perPage} totalCount={mismatches.length} />

              <div className="overflow-x-auto rounded-lg border">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-32">商品コード</TableHead>
                      <TableHead className="min-w-56">商品名</TableHead>
                      <TableHead className="min-w-40">違う項目</TableHead>
                      <TableHead className="w-28 text-right">保存している値</TableHead>
                      <TableHead className="w-28 text-right">記録から求め直した値</TableHead>
                      <TableHead className="w-32 text-center">操作</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {visibleMismatches.map((item) => {
                      const isFixed = fixedCodes.has(item.product_code);
                      return (
                        <TableRow key={item.product_code}>
                          <TableCell className="font-mono font-medium">
                            {item.product_code}
                          </TableCell>
                          <TableCell>{item.name}</TableCell>
                          <TableCell>
                            <Badge variant="outline">{fieldsLabel(item.differing_fields)}</Badge>
                          </TableCell>
                          <TableCell className="text-right tabular-nums">
                            {formatStockCopy(item.copy, item.stock_unit, groupedNumber)}
                          </TableCell>
                          <TableCell className="text-right tabular-nums">
                            {formatStockCopy(item.derived, item.stock_unit, groupedNumber)}
                          </TableCell>
                          <TableCell>
                            <div className="flex items-center justify-center gap-2">
                              {isFixed ? (
                                <Badge variant="outline" tone="success">
                                  <CheckCircle2 aria-hidden="true" />
                                  補正済み
                                </Badge>
                              ) : (
                                <label
                                  htmlFor={`integrity-fix-${item.product_code}`}
                                  className="flex cursor-pointer items-center gap-2"
                                >
                                  <Checkbox
                                    id={`integrity-fix-${item.product_code}`}
                                    aria-label={`${item.product_code}を補正する`}
                                    checked={selectedCodes.has(item.product_code)}
                                    disabled={isBusy}
                                    onCheckedChange={(checked) => {
                                      toggleSelected(item.product_code, checked === true);
                                    }}
                                  />
                                  <span className="text-sm">補正する</span>
                                </label>
                              )}
                            </div>
                          </TableCell>
                        </TableRow>
                      );
                    })}
                  </TableBody>
                </Table>
              </div>

              <Pagination
                page={page}
                perPage={perPage}
                totalCount={mismatches.length}
                onPageChange={setPage}
              />
            </section>
          </div>
        )
      ) : null}

      <AlertDialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>在庫数を記録から求め直した値に合わせて補正します</AlertDialogTitle>
            <AlertDialogDescription className="sr-only">
              補正すると元に戻せません。選択した商品の保存している値を記録から求め直した値に合わせて更新し、操作ログに記録します。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <Alert variant="warning">
            <AlertTriangle aria-hidden="true" />
            <AlertTitle>補正すると元に戻せません</AlertTitle>
            <AlertDescription>
              選択した商品の保存している値を記録から求め直した値に合わせて更新し、操作ログに記録します。
            </AlertDescription>
          </Alert>
          <div className="space-y-2">
            <p className="text-sm text-muted-foreground">
              補正する商品（保存している値 → 記録から求め直した値）
            </p>
            <div className="max-h-72 overflow-y-auto rounded-md border">
              <ul className="divide-y">
                {selectedMismatches.map((item) => (
                  <li key={item.product_code} className="space-y-1 px-3 py-2">
                    <div className="flex flex-wrap items-center justify-between gap-2">
                      <span className="font-mono font-medium">{item.product_code}</span>
                      <span className="font-medium tabular-nums">
                        {formatStockCopy(item.copy, item.stock_unit)} →{" "}
                        {formatStockCopy(item.derived, item.stock_unit)}
                      </span>
                    </div>
                    <p className="text-sm text-muted-foreground">{item.name}</p>
                  </li>
                ))}
              </ul>
            </div>
          </div>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={isBusy}>キャンセル</AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              disabled={isBusy}
              onClick={() => void handleFix()}
            >
              補正を実行する
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {isBusy ? (
        <div
          role="status"
          aria-live="polite"
          className="absolute inset-0 z-40 flex items-center justify-center bg-background/85 p-6 backdrop-blur-[1px]"
        >
          <div className="w-full max-w-md space-y-4 rounded-lg border bg-card p-6 text-center shadow-lg">
            <Loader2 aria-hidden="true" className="mx-auto size-8 animate-spin text-ongoing" />
            <p className="text-lg font-semibold">
              {pendingOperation === "fix" ? "補正を記録しています" : "在庫データを確認しています"}
            </p>
            <p className="text-sm text-muted-foreground">完了するまでこの画面でお待ちください。</p>
            <Progress
              aria-label="処理中"
              className="before:absolute before:inset-y-0 before:left-1/4 before:w-1/2 before:animate-pulse before:bg-ongoing before:content-['']"
            />
          </div>
        </div>
      ) : null}
    </PageShell>
  );
}
