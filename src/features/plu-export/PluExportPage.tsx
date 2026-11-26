import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { AlertTriangle, CheckCircle2, Download, RotateCcw, Save } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { PageHeader } from "@/components/patterns/PageHeader";
import { PageShell } from "@/components/patterns/PageShell";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { SegmentedControl } from "@/components/ui/segmented-control";
import {
  commands,
  type ExportMode,
  type PluExportPrepareResponse,
  type PluExportConfirmResponse,
  type PluPreparedRow,
} from "@/lib/bindings";
import { formatLengthOrPlain } from "@/features/stock-inquiry/lib/format-stock-display";
import { formatDateTime } from "@/lib/date-time";
import { describeError } from "@/lib/describe-error";
import { invalidateByContract, invalidationContract } from "@/lib/invalidation-contract";
import { unwrapResult } from "@/lib/invoke";
import { scrollPageToTop } from "@/lib/page-scroll";
import { queryKeys } from "@/lib/query-keys";
import { save } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";

type FlowStatus =
  | "idle"
  | "preparing"
  | "cancelled"
  | "prepare_failed"
  | "save_failed"
  | "saved"
  | "confirm_failed"
  | "confirmed";

interface PendingPluExport {
  version: 1;
  mode: ExportMode;
  savedAt: string;
  savedPath: string;
  suggestedFilename: string;
  count: number;
  encoding: string;
  targetProductCodes: string[];
  preparedRows: PluPreparedRow[];
}

export const PLU_EXPORT_PENDING_STORAGE_KEY = "inventory:plu-export:pending:v1";

const PENDING_PLU_EXPORT_KEYS = [
  "version",
  "mode",
  "savedAt",
  "savedPath",
  "suggestedFilename",
  "count",
  "encoding",
  "targetProductCodes",
  "preparedRows",
] as const;

/** UI-08-D4・D10（67 §67.9 の failure note before confirm）: 保存の後と復帰の Alert の両方に出す 1 つの文 */
const IMPORT_FAILURE_RECOVERY_NOTE =
  "PCツールに取り込めなかった場合は、差分または全件を書き出し直してください。前に保存したファイルは取り込まないでください（保存の後にレジの登録状況が変わっていると、レジの登録を上書きします）。";

const MODE_OPTIONS = [
  { value: "diff", label: "差分", description: "未反映の商品だけ" },
  { value: "full", label: "全件", description: "廃番を除く全商品" },
] as const;

function decodeBase64Bytes(value: string): Uint8Array {
  const binary = atob(value);
  return Uint8Array.from(binary, (char) => char.charCodeAt(0));
}

function isExportMode(value: unknown): value is ExportMode {
  return value === "diff" || value === "full";
}

function hasOnlyPendingPluExportKeys(value: Record<string, unknown>): boolean {
  const allowed = new Set<string>(PENDING_PLU_EXPORT_KEYS);
  return Object.keys(value).every((key) => allowed.has(key));
}

/** 復帰状態の 1 行。照合の値を持たない形・`row_kind` を持つ形（D-117 の前）は schema 不一致 */
function isPendingPreparedRow(value: unknown): value is PluPreparedRow {
  if (!value || typeof value !== "object") return false;
  const row = value as Record<string, unknown>;
  return (
    Object.keys(row).length === 3 &&
    Number.isInteger(row.memory_no) &&
    Array.isArray(row.target_product_codes) &&
    row.target_product_codes.every((code) => typeof code === "string" && code.length > 0) &&
    typeof row.content_hash === "string" &&
    row.content_hash.length > 0
  );
}

function isPendingPluExport(value: unknown): value is PendingPluExport {
  if (!value || typeof value !== "object") return false;
  if (!hasOnlyPendingPluExportKeys(value as Record<string, unknown>)) return false;
  const pending = value as Partial<PendingPluExport>;
  if (!Array.isArray(pending.targetProductCodes) || !Array.isArray(pending.preparedRows)) {
    return false;
  }
  const uniqueTargetCodes = new Set(pending.targetProductCodes);
  return (
    pending.version === 1 &&
    isExportMode(pending.mode) &&
    typeof pending.savedAt === "string" &&
    typeof pending.savedPath === "string" &&
    typeof pending.suggestedFilename === "string" &&
    typeof pending.count === "number" &&
    Number.isInteger(pending.count) &&
    pending.count >= 0 &&
    typeof pending.encoding === "string" &&
    pending.preparedRows.length > 0 &&
    uniqueTargetCodes.size === pending.targetProductCodes.length &&
    pending.count === pending.preparedRows.length &&
    pending.targetProductCodes.every((code) => typeof code === "string" && code.length > 0) &&
    pending.preparedRows.every(isPendingPreparedRow)
  );
}

function clearPendingPluExport() {
  try {
    window.localStorage.removeItem(PLU_EXPORT_PENDING_STORAGE_KEY);
  } catch {
    // localStorage can be unavailable in restricted WebView contexts.
  }
}

function loadPendingPluExport(): PendingPluExport | null {
  try {
    const raw = window.localStorage.getItem(PLU_EXPORT_PENDING_STORAGE_KEY);
    if (!raw) return null;
    const parsed: unknown = JSON.parse(raw);
    if (isPendingPluExport(parsed)) return parsed;
    clearPendingPluExport();
    return null;
  } catch {
    clearPendingPluExport();
    return null;
  }
}

function savePendingPluExport(pending: PendingPluExport) {
  try {
    window.localStorage.setItem(PLU_EXPORT_PENDING_STORAGE_KEY, JSON.stringify(pending));
  } catch {
    // Persistence is a recovery aid. The current in-memory confirmation path still works.
  }
}

function buildPendingPluExport(
  exportMode: ExportMode,
  targetPath: string,
  nextPrepared: PluExportPrepareResponse,
): PendingPluExport {
  return {
    version: 1,
    mode: exportMode,
    savedAt: new Date().toISOString(),
    savedPath: targetPath,
    suggestedFilename: nextPrepared.suggested_filename,
    count: nextPrepared.count,
    encoding: nextPrepared.encoding,
    targetProductCodes: nextPrepared.target_product_codes,
    preparedRows: nextPrepared.prepared_rows,
  };
}

// savedAt は toISOString() の UTC なので、ローカル時刻の YYYY-MM-DD HH:mm:ss へ直す（67 処理ステップ 8）。
function formatPendingSavedAt(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${String(date.getFullYear())}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
}

function formatExcludedReason(reason: string): string {
  switch (reason) {
    case "missing_jan":
      return "JAN未登録";
    case "invalid_jan_format":
      return "JANが13桁ではありません";
    case "invalid_check_digit":
      return "JANのチェックディジットが不正です";
    case "group_price_mismatch":
      return "同じJANの商品で売価または税率が一致していません";
    case "no_free_slot":
      return "レジの空きスロットがありません";
    default:
      return "要修正（詳細不明）";
  }
}

// eslint-disable-next-line complexity -- 衛生 batch 3 起票時実測（10a §3 item #1）: 保存・未確認復帰・確認の state/表示条件混在。flow hook / 状態 panel への分割は本 lane の scope 外、Backlog 参照
export function PluExportPage() {
  const queryClient = useQueryClient();
  const [mode, setMode] = useState<ExportMode>("diff");
  const [status, setStatus] = useState<FlowStatus>("idle");
  const [prepared, setPrepared] = useState<PluExportPrepareResponse | null>(null);
  const [pendingExport, setPendingExport] = useState<PendingPluExport | null>(() =>
    loadPendingPluExport(),
  );
  const [savedPath, setSavedPath] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [confirmResult, setConfirmResult] = useState<PluExportConfirmResponse | null>(null);

  const dirtyQuery = useQuery({
    queryKey: queryKeys.pluDirty(),
    queryFn: () =>
      unwrapResult(commands.listPluDirty(), { source: "commands", cmd: "list_plu_dirty" }),
  });

  const slotSummaryQuery = useQuery({
    queryKey: queryKeys.pluSlotSummary(),
    queryFn: () =>
      unwrapResult(commands.getPluSlotSummary(), {
        source: "commands",
        cmd: "get_plu_slot_summary",
      }),
  });

  const isBusy = status === "preparing";
  const dirtyCount = dirtyQuery.data?.length ?? 0;
  const snapshotAt = slotSummaryQuery.data?.snapshot_at ?? null;

  async function savePreparedFile(nextPrepared: PluExportPrepareResponse) {
    const targetPath = await save({
      defaultPath: nextPrepared.suggested_filename,
      filters: [{ name: "PLUテキスト", extensions: ["txt"] }],
    });

    if (!targetPath) {
      setStatus("cancelled");
      setSavedPath(null);
      setErrorMessage(null);
      scrollPageToTop();
      return;
    }

    try {
      await writeFile(targetPath, decodeBase64Bytes(nextPrepared.bytes_base64));
      const nextPending = buildPendingPluExport(mode, targetPath, nextPrepared);
      savePendingPluExport(nextPending);
      setPendingExport(nextPending);
      setStatus("saved");
      setSavedPath(targetPath);
      setErrorMessage(null);
      scrollPageToTop();
      toast.success("PLUファイルを保存しました");
    } catch (error) {
      setStatus("save_failed");
      setSavedPath(null);
      setErrorMessage(describeError(error));
      scrollPageToTop();
      toast.error("PLUファイルを保存できませんでした");
    }
  }

  async function handlePrepareAndSave() {
    setStatus("preparing");
    setErrorMessage(null);
    setSavedPath(null);
    try {
      const nextPrepared = await unwrapResult(commands.preparePluExport(mode), {
        source: "commands",
        cmd: "prepare_plu_export",
      });
      await invalidateByContract(queryClient, invalidationContract.pluExportPrepare());
      setPrepared(nextPrepared);
      await savePreparedFile(nextPrepared);
    } catch (error) {
      setStatus("prepare_failed");
      setErrorMessage(describeError(error));
      scrollPageToTop();
      toast.error("PLU書出しを準備できませんでした");
    }
  }

  async function handleRetrySave() {
    if (!prepared) return;
    setStatus("preparing");
    setErrorMessage(null);
    await savePreparedFile(prepared);
  }

  async function handleConfirm() {
    const targetProductCodes = pendingExport?.targetProductCodes ?? prepared?.target_product_codes;
    const preparedRows = pendingExport?.preparedRows ?? prepared?.prepared_rows;
    if (!targetProductCodes || !preparedRows || preparedRows.length === 0) return;
    setStatus("preparing");
    setErrorMessage(null);
    try {
      const result = await unwrapResult(
        commands.confirmPluExportSaved(targetProductCodes, preparedRows),
        { source: "commands", cmd: "confirm_plu_export_saved" },
      );
      await invalidateByContract(queryClient, invalidationContract.pluExportConfirm());
      clearPendingPluExport();
      setPendingExport(null);
      setConfirmResult(result);
      setStatus("confirmed");
      scrollPageToTop();
      if (result.updated_count > 0) toast.success("PLU未反映を更新しました");
    } catch (error) {
      setStatus("confirm_failed");
      setErrorMessage(describeError(error));
      scrollPageToTop();
      toast.error("未反映から外せませんでした");
    }
  }

  function handleDiscardPending() {
    clearPendingPluExport();
    setPendingExport(null);
    setPrepared(null);
    setSavedPath(null);
    setErrorMessage(null);
    setStatus("idle");
    scrollPageToTop();
  }

  const displayCount = pendingExport?.count ?? prepared?.count;
  const displayEncoding = pendingExport?.encoding ?? prepared?.encoding;
  const displaySavedPath = pendingExport?.savedPath ?? savedPath;
  // 33 §16.5: 保存の後に出力の行が変わっていて、未反映のまま残した商品
  const remainingProducts = confirmResult?.remaining_products ?? [];
  const excludedProducts = prepared?.excluded ?? [];
  const showPendingRecovery =
    pendingExport !== null &&
    status !== "saved" &&
    status !== "confirm_failed" &&
    status !== "confirmed" &&
    status !== "preparing";
  const hasTopStatus =
    showPendingRecovery ||
    status === "cancelled" ||
    status === "prepare_failed" ||
    status === "save_failed" ||
    status === "saved" ||
    status === "confirm_failed" ||
    status === "confirmed";
  const hasSavedExportPending =
    pendingExport !== null || status === "saved" || status === "confirm_failed";

  return (
    <PageShell>
      <PageHeader
        title="PLU書出し"
        description="レジに登録する商品のファイルを作るページです。レジの登録状況は、毎日の売上データ取込みでレジの SD を読むと自動で読み込まれます。『差分を書き出す』（未反映の商品だけ）か『全件を書き出す』を選んで保存し、レジスターツールで取り込んでレジへ反映した後、『この書出しを未反映から外す』を押して確定します。"
      />

      <Alert>
        <AlertTitle>PLUファイル保存後に手動確認が必要です</AlertTitle>
        <AlertDescription>
          アプリで確認できるのはPLUファイル保存までです。PCツールへの取込み、SDカード書出し、レジ読込みは手動で確認してください。
        </AlertDescription>
      </Alert>

      {/* UI-08-D11: 占有は売上データ取込みの SD の読取りで自動で照合する。ここは最終照合日時の 1 行だけ */}
      {slotSummaryQuery.isError ? (
        <p className="text-sm text-destructive">レジの登録状況を取得できませんでした。</p>
      ) : null}
      {slotSummaryQuery.isSuccess && snapshotAt !== null ? (
        <p className="text-sm">
          レジの登録状況: {formatDateTime(snapshotAt)}（売上データ取込みで SD を読んだとき）
        </p>
      ) : null}
      {slotSummaryQuery.isSuccess && snapshotAt === null ? (
        <Alert variant="warning">
          <AlertTriangle />
          <AlertTitle>レジの登録状況がまだ読まれていません</AlertTitle>
          <AlertDescription className="space-y-3">
            <p>売上データ取込みでレジの SD を一度読んでください。</p>
            <Button asChild variant="outline" size="sm">
              <Link to="/csv-import">売上データ取込みへ</Link>
            </Button>
          </AlertDescription>
        </Alert>
      ) : null}

      {hasTopStatus ? (
        <section aria-label="PLU書出し状態" className="space-y-3">
          {showPendingRecovery ? (
            <Alert variant="warning">
              <AlertTriangle />
              <AlertTitle>保存済みで未確認のPLU書出しがあります</AlertTitle>
              <AlertDescription className="space-y-3">
                <p>
                  PCツールでこのPLUファイルを扱う場合だけ、未反映から外してください。
                  違うPLUファイルでやり直す場合は破棄して再書出しします。
                </p>
                <p className="font-medium">{IMPORT_FAILURE_RECOVERY_NOTE}</p>
                <div className="grid gap-1">
                  <p>保存先: {pendingExport.savedPath}</p>
                  <p>件数: {pendingExport.count.toLocaleString()} 件</p>
                  <p>文字コード: {pendingExport.encoding}</p>
                  <p>保存日時: {formatPendingSavedAt(pendingExport.savedAt)}</p>
                </div>
                <div className="flex flex-wrap gap-2">
                  <Button type="button" size="sm" onClick={() => void handleConfirm()}>
                    この書出しを未反映から外す
                  </Button>
                  <Button type="button" variant="outline" size="sm" onClick={handleDiscardPending}>
                    破棄して再書出し
                  </Button>
                </div>
              </AlertDescription>
            </Alert>
          ) : null}

          {status === "cancelled" ? (
            <Alert>
              <AlertTitle>保存はキャンセルされました</AlertTitle>
              <AlertDescription>未反映商品は残っています。</AlertDescription>
            </Alert>
          ) : null}

          {status === "save_failed" ? (
            <Alert variant="destructive">
              <AlertTitle>PLUファイルを保存できませんでした</AlertTitle>
              <AlertDescription className="space-y-3">
                <p>PCツールへ進む前に、もう一度保存してください。</p>
                {errorMessage ? <p>{errorMessage}</p> : null}
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={() => void handleRetrySave()}
                >
                  <RotateCcw />
                  もう一度保存する
                </Button>
              </AlertDescription>
            </Alert>
          ) : null}

          {status === "prepare_failed" ? (
            <Alert variant="destructive">
              <AlertTitle>PLU書出しを準備できませんでした</AlertTitle>
              <AlertDescription className="space-y-3">
                <p>商品マスタのJANコードを確認してください。</p>
                {errorMessage ? <p>{errorMessage}</p> : null}
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={() => void handlePrepareAndSave()}
                >
                  <RotateCcw />
                  もう一度準備する
                </Button>
              </AlertDescription>
            </Alert>
          ) : null}

          {status === "saved" ? (
            <>
              <Alert className="border-success bg-success-soft text-success-strong">
                <Save />
                <AlertTitle>PLUファイルを保存しました</AlertTitle>
                <AlertDescription className="space-y-3">
                  {displaySavedPath ? <p>保存先: {displaySavedPath}</p> : null}
                  <p>
                    アプリで確認できるのはPLUファイル保存までです。PCツールへの取込み、SDカード書出し、レジ読込みは手動で確認してください。
                  </p>
                  <Button type="button" size="sm" onClick={() => void handleConfirm()}>
                    この書出しを未反映から外す
                  </Button>
                </AlertDescription>
              </Alert>
              <Alert variant="warning">
                <AlertTriangle />
                <AlertTitle>PCツールに取り込めなかった場合の回復手順</AlertTitle>
                <AlertDescription>{IMPORT_FAILURE_RECOVERY_NOTE}</AlertDescription>
              </Alert>
            </>
          ) : null}

          {status === "confirm_failed" ? (
            <Alert variant="destructive">
              <AlertTitle>未反映から外せませんでした</AlertTitle>
              <AlertDescription className="space-y-3">
                {displaySavedPath ? <p>保存済みPLUファイル: {displaySavedPath}</p> : null}
                <p>
                  保存済みPLUファイルは残っています。原因を確認してから、もう一度実行してください。
                </p>
                {errorMessage ? <p>{errorMessage}</p> : null}
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={() => void handleConfirm()}
                >
                  <RotateCcw />
                  もう一度未反映から外す
                </Button>
              </AlertDescription>
            </Alert>
          ) : null}

          {status === "confirmed" && remainingProducts.length > 0 ? (
            <Alert variant="warning">
              <AlertTriangle />
              <AlertTitle>未反映のまま残した商品があります</AlertTitle>
              <AlertDescription className="space-y-3">
                <p>
                  保存の後に商品の内容（名称・売価・税率・部門）が変わったため、次の商品は未反映から外していません。差分をもう一度書き出して、レジスターツールで取り込んでください。前に保存したファイルは取り込まないでください。
                </p>
                <ul className="list-disc pl-5">
                  {remainingProducts.map((product) => (
                    <li key={product.product_code}>
                      {product.name}（{product.product_code}）
                    </li>
                  ))}
                </ul>
              </AlertDescription>
            </Alert>
          ) : null}

          {status === "confirmed" && (confirmResult?.updated_count ?? 0) > 0 ? (
            <Alert className="border-success bg-success-soft text-success-strong">
              <CheckCircle2 />
              <AlertTitle>未反映から外しました</AlertTitle>
              <AlertDescription>
                保存したPLUファイルに含まれる商品だけをPLU未反映から外しました。
              </AlertDescription>
            </Alert>
          ) : null}
        </section>
      ) : null}

      {excludedProducts.length > 0 ? (
        <section aria-label="書出しに含めなかった商品" className="space-y-3">
          <h2 className="flex items-center gap-2 text-xl font-semibold">
            <AlertTriangle className="size-5 text-warning-strong" aria-hidden="true" />
            書出しに含めなかった商品（要修正）
          </h2>
          <p className="text-sm text-muted-foreground">
            これらの商品は今回のPLUファイルに含めていません。
            <span className="font-medium text-foreground">
              商品マスタでJANコード・売価・税率を修正すると、次回の書出しから含まれます。
            </span>
          </p>
          <div className="overflow-x-auto rounded-md border">
            <table className="w-full min-w-[760px] text-sm">
              <thead className="border-b bg-muted/40 text-left text-muted-foreground">
                <tr>
                  <th className="py-2 pr-4 pl-3 font-medium">商品コード</th>
                  <th className="py-2 pr-4 font-medium">JANコード</th>
                  <th className="py-2 pr-4 font-medium">商品名</th>
                  <th className="py-2 pr-3 font-medium">理由</th>
                </tr>
              </thead>
              <tbody className="divide-y">
                {excludedProducts.map((product) => (
                  <tr key={product.product_code}>
                    <td className="py-2 pr-4 pl-3 font-mono text-[0.95rem]">
                      {product.product_code}
                    </td>
                    <td className="py-2 pr-4 font-mono text-[0.95rem]">
                      {product.jan_code ?? (
                        <span className="font-sans text-muted-foreground">未登録</span>
                      )}
                    </td>
                    <td className="py-2 pr-4">{product.name}</td>
                    <td className="py-2 pr-3 text-warning-strong">
                      {formatExcludedReason(product.reason)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>
      ) : null}

      <section
        aria-label="PLU書出し内容"
        className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(320px,420px)]"
      >
        <Card>
          <CardHeader>
            <CardTitle>未反映商品</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {dirtyQuery.isLoading ? (
              <p className="text-sm text-muted-foreground">読込み中です。</p>
            ) : null}
            {dirtyQuery.isError ? (
              <p className="text-sm text-destructive">未反映商品を取得できませんでした。</p>
            ) : null}
            {dirtyQuery.isSuccess && dirtyCount === 0 ? (
              <p className="text-sm text-muted-foreground">PLU書出しが必要な商品はありません。</p>
            ) : null}
            {dirtyQuery.isSuccess && dirtyCount > 0 ? (
              <div className="overflow-x-auto">
                <table className="w-full min-w-[680px] text-sm">
                  <thead className="border-b text-left text-muted-foreground">
                    <tr>
                      <th className="py-2 pr-4 font-medium">商品コード</th>
                      <th className="py-2 pr-4 font-medium">JANコード</th>
                      <th className="py-2 pr-4 font-medium">商品名</th>
                      <th className="py-2 pr-4 text-right font-medium">売価</th>
                      <th className="py-2 text-right font-medium">在庫</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y">
                    {dirtyQuery.data.map((product) => (
                      <tr key={product.product_code}>
                        <td className="py-2 pr-4 font-mono text-[0.95rem]">
                          {product.product_code}
                        </td>
                        <td className="py-2 pr-4 font-mono text-[0.95rem]">
                          {product.jan_code ?? (
                            <span className="font-sans text-muted-foreground">未登録</span>
                          )}
                        </td>
                        <td className="py-2 pr-4">{product.name}</td>
                        <td className="py-2 pr-4 text-right">
                          {product.selling_price.toLocaleString()} 円
                        </td>
                        <td className="py-2 text-right">
                          {formatLengthOrPlain(
                            product.stock_quantity,
                            product.stock_unit,
                            (quantity) => quantity.toLocaleString(),
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : null}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>書出し設定</CardTitle>
          </CardHeader>
          <CardContent className="space-y-5">
            <SegmentedControl
              ariaLabel="書出しモード"
              value={mode}
              options={MODE_OPTIONS}
              onValueChange={setMode}
            />

            <Alert variant="warning">
              <AlertTriangle />
              <AlertTitle>Diff / Full ともレジへ投入できます</AlertTitle>
              <AlertDescription>
                メモリNo.を永続化する前に作成した全件ファイルは再投入しないでください。
              </AlertDescription>
            </Alert>

            <div className="grid gap-2 text-sm">
              <div className="flex justify-between gap-4">
                <span className="text-muted-foreground">差分対象</span>
                <strong>{dirtyCount.toLocaleString()} 件</strong>
              </div>
              {displayCount !== undefined && displayEncoding ? (
                <>
                  <div className="flex justify-between gap-4">
                    <span className="text-muted-foreground">今回の書出し件数</span>
                    <strong>{displayCount.toLocaleString()} 件</strong>
                  </div>
                  <div className="flex justify-between gap-4">
                    <span className="text-muted-foreground">文字コード</span>
                    <strong>{displayEncoding}</strong>
                  </div>
                </>
              ) : null}
            </div>

            <Button
              type="button"
              variant={hasSavedExportPending ? "outline" : "default"}
              className="w-full"
              disabled={
                isBusy ||
                snapshotAt === null ||
                (mode === "diff" && dirtyQuery.isSuccess && dirtyCount === 0)
              }
              onClick={() => void handlePrepareAndSave()}
            >
              <Download />
              {mode === "full" ? "全件を書き出す" : "差分を書き出す"}
            </Button>
          </CardContent>
        </Card>
      </section>
    </PageShell>
  );
}
