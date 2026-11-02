import { useState } from "react";
import { AlertTriangle, CheckCircle2, FileWarning, RefreshCw } from "lucide-react";
import { FilePicker, type PickedFile } from "@/components/FilePicker";
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
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { ImportPreview } from "@/lib/bindings";

export interface ProductImportPreviewProps {
  filename: string;
  preview: ImportPreview;
  overwriteCodes: string[];
  targetCount: number;
  isCommitting: boolean;
  onToggleOverwrite: (productCode: string, checked: boolean) => void;
  onCommit: () => void;
  onReselect: (file: PickedFile) => void;
}

const MAX_VISIBLE_ROWS = 50;

export function ProductImportPreview({
  filename,
  preview,
  overwriteCodes,
  targetCount,
  isCommitting,
  onToggleOverwrite,
  onCommit,
  onReselect,
}: ProductImportPreviewProps) {
  const [confirmOpen, setConfirmOpen] = useState(false);
  const overwriteCount = overwriteCodes.length;
  const commitDisabled = isCommitting || targetCount === 0;

  function handleCommitClick() {
    if (overwriteCount > 0) {
      setConfirmOpen(true);
      return;
    }
    onCommit();
  }

  return (
    <div className="space-y-4">
      <div className="grid gap-3 md:grid-cols-4">
        <SummaryCard title="登録対象" value={`${targetCount.toLocaleString()} 件`} tone="success" />
        <SummaryCard title="新規候補" value={`${preview.valid_rows.length.toLocaleString()} 件`} />
        <SummaryCard title="重複" value={`${preview.duplicate_rows.length.toLocaleString()} 件`} />
        <SummaryCard title="エラー" value={`${preview.error_rows.length.toLocaleString()} 件`} />
      </div>

      <div className="flex flex-wrap items-center gap-2 text-sm">
        <Badge variant="secondary">ファイル: {filename}</Badge>
        {overwriteCount > 0 ? (
          <Badge className="border-warning bg-warning-emphasis text-primary-foreground">
            上書き {overwriteCount.toLocaleString()} 件
          </Badge>
        ) : null}
      </div>

      {targetCount === 0 ? (
        <Alert variant="destructive">
          <AlertTitle>登録できる行がありません</AlertTitle>
          <AlertDescription>
            エラー行を修正するか、重複行を上書き対象に選んでから再度実行してください。
          </AlertDescription>
        </Alert>
      ) : null}

      {preview.valid_rows.length > 0 ? (
        <Card>
          <CardHeader>
            <CardTitle>新規登録候補</CardTitle>
          </CardHeader>
          <CardContent>
            <ImportRowsTable rows={preview.valid_rows} />
          </CardContent>
        </Card>
      ) : null}

      {preview.duplicate_rows.length > 0 ? (
        <Card>
          <CardHeader>
            <CardTitle>既存商品との重複</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <Alert>
              <FileWarning className="size-4" aria-hidden="true" />
              <AlertTitle>初期状態では重複行をスキップします</AlertTitle>
              <AlertDescription>
                上書きする行だけ選択してください。選択していない重複行は登録しません。
              </AlertDescription>
            </Alert>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-24">上書き</TableHead>
                  <TableHead>行</TableHead>
                  <TableHead>商品コード</TableHead>
                  <TableHead>商品名</TableHead>
                  <TableHead className="text-right">売価</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {preview.duplicate_rows.slice(0, MAX_VISIBLE_ROWS).map((row) => {
                  const productCode = row.import_row.product_code;
                  return (
                    <TableRow key={`${String(row.line_no)}-${productCode}`}>
                      <TableCell>
                        <Checkbox
                          aria-label={`${productCode} を上書き`}
                          checked={overwriteCodes.includes(productCode)}
                          disabled={isCommitting}
                          onCheckedChange={(checked) => {
                            onToggleOverwrite(productCode, checked === true);
                          }}
                        />
                      </TableCell>
                      <TableCell>{row.line_no}</TableCell>
                      <TableCell className="font-mono">{productCode}</TableCell>
                      <TableCell>{row.import_row.name}</TableCell>
                      <TableCell className="text-right">
                        ¥{row.import_row.selling_price.toLocaleString()}
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
            {preview.duplicate_rows.length > MAX_VISIBLE_ROWS ? (
              <p className="text-xs text-muted-foreground">
                他 {(preview.duplicate_rows.length - MAX_VISIBLE_ROWS).toLocaleString()} 件
              </p>
            ) : null}
          </CardContent>
        </Card>
      ) : null}

      {preview.error_rows.length > 0 ? (
        <Card>
          <CardHeader>
            <CardTitle>エラー行</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            {/* UI-01c-D18: エラー行は直すための一覧なので 50 行の制限に掛けず全件を出す */}
            <p className="text-sm">
              {
                "エラー行は取り込みません。内容の文のとおりに CSV を直し、「ファイルを選び直す」で読み込み直してください"
              }
            </p>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>行</TableHead>
                  <TableHead>内容</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {preview.error_rows.map((row) => (
                  <TableRow key={row.line_no}>
                    <TableCell>{row.line_no}</TableCell>
                    <TableCell>{row.errors.join(" / ")}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      ) : null}

      <div className="flex flex-wrap gap-2">
        <Button type="button" onClick={handleCommitClick} disabled={commitDisabled}>
          {isCommitting ? "インポート中..." : "インポート実行"}
        </Button>
        <FilePicker
          accept=".csv,.txt"
          ariaLabel="ファイルを選び直す（商品マスタCSV）"
          buttonLabel="ファイルを選び直す"
          buttonIcon={<RefreshCw aria-hidden="true" />}
          dialogFilterName="CSV / TXT"
          dropEnabled={false}
          disabled={isCommitting}
          onSelect={onReselect}
        />
      </div>

      <AlertDialog open={confirmOpen} onOpenChange={setConfirmOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>選択した重複行を上書きしますか</AlertDialogTitle>
            <AlertDialogDescription>
              {overwriteCount.toLocaleString()}{" "}
              件の既存商品をCSVの内容で更新します。選択していない重複行はスキップします。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>キャンセル</AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              onClick={() => {
                setConfirmOpen(false);
                onCommit();
              }}
            >
              上書きして実行
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

function SummaryCard({ title, value, tone }: { title: string; value: string; tone?: "success" }) {
  return (
    <Card>
      <CardContent className="flex items-center gap-3 p-4">
        {tone === "success" ? (
          <CheckCircle2 className="size-5 text-success-emphasis" aria-hidden="true" />
        ) : null}
        <div>
          <p className="text-xs text-muted-foreground">{title}</p>
          <p className="text-lg font-semibold">{value}</p>
        </div>
      </CardContent>
    </Card>
  );
}

function ImportRowsTable({ rows }: { rows: ImportPreview["valid_rows"] }) {
  return (
    <div className="space-y-2">
      <p className="text-sm text-muted-foreground">
        PLU対象が空欄の場合は、13桁JANの商品を対象とする既定ルールを適用します。
      </p>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>行</TableHead>
            <TableHead>商品コード</TableHead>
            <TableHead>商品名</TableHead>
            <TableHead>部門ID</TableHead>
            <TableHead className="text-right">売価</TableHead>
            <TableHead>PLU対象</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {rows.slice(0, MAX_VISIBLE_ROWS).map((row) => (
            <TableRow key={`${String(row.line_no)}-${row.product_code}`}>
              <TableCell>{row.line_no}</TableCell>
              <TableCell className="font-mono">{row.product_code}</TableCell>
              <TableCell>{row.name}</TableCell>
              <TableCell>{row.department_id}</TableCell>
              <TableCell className="text-right">¥{row.selling_price.toLocaleString()}</TableCell>
              <TableCell>
                <div className="space-y-1">
                  <span>
                    {row.plu_target === true
                      ? "対象"
                      : row.plu_target === false
                        ? "対象外"
                        : "既定（13桁JANなら対象）"}
                  </span>
                  {(row.warnings ?? []).map((warning) => (
                    <span
                      key={warning}
                      className="flex items-start gap-1 text-xs text-warning-emphasis"
                    >
                      <AlertTriangle className="mt-0.5 size-3 shrink-0" aria-hidden="true" />
                      {warning}
                    </span>
                  ))}
                </div>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}
