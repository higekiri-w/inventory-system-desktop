// src/features/daily-sales/components/ExportBar.tsx
//
// CSV 出力 button + 日報の印刷ボタン（UI-09a-D17）。押せない理由はボタンの下に文で出す（Tooltip だけにしない）。
// 押したときは (1) 紙の内容を読み直し、(2) 失敗か 0 件なら印刷せず、(3) 読み直した値と押した時刻で
// 紙を描き直し、(4) DOM に反映された後に window.print() を呼ぶ（SALES-SHEET-D6）。
// 設計: docs/function-design/56-ui-daily-sales.md UI-09a-D17・D18、§56.7

import { Link } from "@tanstack/react-router";
import type { UseQueryResult } from "@tanstack/react-query";
import { useEffect, useState } from "react";

import { Button } from "@/components/ui/button";
import type { DailyReportSheet as Sheet, DailyReportSourceKind } from "@/lib/bindings";
import { describeError } from "@/lib/describe-error";
import { DailyReportSheet, SOURCE_TITLES, sourceNotice } from "./DailyReportSheet";

export interface ExportBarProps {
  onExportCsv: () => void;
  isExporting: boolean;
  date: string;
  sheetQuery: Pick<UseQueryResult<Sheet>, "data" | "error" | "isError" | "isPending" | "refetch">;
}

export function ExportBar({ onExportCsv, isExporting, date, sheetQuery }: ExportBarProps) {
  // 読み直した紙の内容と押した時刻。印刷する間だけ紙を描き、描いた後の effect で印刷する。
  // 親の query の再描画を待たず、refetch の結果そのものを描く（古い内容で印刷しない、SALES-SHEET-D6）
  const [printing, setPrinting] = useState<{ sheet: Sheet; printedAt: Date } | null>(null);
  const [isRefetching, setIsRefetching] = useState(false);

  useEffect(() => {
    if (printing === null) return;
    window.print();
    setPrinting(null);
  }, [printing]);

  const sheet = sheetQuery.data;
  const canPrint =
    !sheetQuery.isPending && !sheetQuery.isError && sheet !== undefined && sheet.import_count >= 1;

  const handlePrint = async () => {
    setIsRefetching(true);
    const result = await sheetQuery.refetch();
    setIsRefetching(false);
    if (result.isError || result.data === undefined || result.data.import_count === 0) return;
    setPrinting({ sheet: result.data, printedAt: new Date() });
  };

  return (
    <div className="flex flex-col items-end gap-2">
      <div className="flex items-center gap-2">
        <Button
          type="button"
          variant="default"
          onClick={onExportCsv}
          disabled={isExporting}
          aria-label="CSV を保存"
        >
          {isExporting ? "出力中..." : "CSV 出力"}
        </Button>
        <Button
          type="button"
          variant="outline"
          aria-disabled={!canPrint || isRefetching}
          onClick={() => {
            if (!canPrint || isRefetching) return;
            void handlePrint();
          }}
        >
          日報を印刷
        </Button>
      </div>
      <PrintStatus date={date} sheetQuery={sheetQuery} />
      {printing !== null && (
        <DailyReportSheet sheet={printing.sheet} printedAt={printing.printedAt} />
      )}
    </div>
  );
}

function PrintStatus({
  date,
  sheetQuery,
}: {
  date: string;
  sheetQuery: ExportBarProps["sheetQuery"];
}) {
  if (sheetQuery.isPending) return null;
  if (sheetQuery.isError) {
    return (
      <div className="flex items-center gap-2 text-sm" role="alert">
        <span>{describeError(sheetQuery.error)}</span>
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={() => {
            void sheetQuery.refetch();
          }}
        >
          再試行
        </Button>
      </div>
    );
  }
  const sheet = sheetQuery.data;
  if (sheet === undefined) return null;
  if (sheet.import_count === 0) {
    return (
      <p className="text-sm">
        <span>この日のレジ日報は未取込みのため、印刷できません。</span>{" "}
        <Link to="/csv-import" className="font-medium text-primary underline underline-offset-4">
          売上データ取込みを開く
        </Link>
      </p>
    );
  }
  const changed = sheet.label_changed_sources;
  const skipped = sheet.plu_import.skipped_count;
  if (changed.length === 0 && skipped === 0) return null;
  return (
    <div className="space-y-1 text-sm">
      {/* 紙の上の知らせと同じ文を帳票ごとに 1 行（UI-09a-D17） */}
      {changed.map((source: DailyReportSourceKind) => (
        <p key={source}>{`${SOURCE_TITLES[source]}：${sourceNotice(sheet, source) ?? ""}`}</p>
      ))}
      {skipped >= 1 && (
        <p>
          <span>{`数えていない PLU の行が ${skipped.toLocaleString("ja-JP")} 行あります`}</span>{" "}
          <Link
            to="/inventory/records"
            search={{ recordType: "csv_import", status: "active", dateFrom: date, dateTo: date }}
            className="font-medium text-primary underline underline-offset-4"
          >
            取込みの記録を開く
          </Link>
        </p>
      )}
    </div>
  );
}
