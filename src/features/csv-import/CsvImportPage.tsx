// src/features/csv-import/CsvImportPage.tsx
//
// UI-07 CSV取込み画面の最上位レイアウト。useCsvImportFlow() の戻り値を state.status で
// 4 step UI (Parse/Preview/Importing/Result) に振り分け、error variant は重ね描き。
// 設計: docs/function-design/55-ui-csv-import.md §55.1 / §55.4

import { Link } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { PageHeader } from "@/components/patterns/PageHeader";
import { PageShell } from "@/components/patterns/PageShell";
import type { PickedFile } from "@/components/FilePicker";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { DailyReportImportPage } from "@/features/daily-report-import/DailyReportImportPage";
import { useCsvImportFlow } from "./hooks/useCsvImportFlow";
import { ErrorState } from "./components/ErrorState";
import { ImportingStep } from "./components/ImportingStep";
import { ParseStep } from "./components/ParseStep";
import { PreviewStep } from "./components/PreviewStep";
import { ResultStep, RollbackResultCard } from "./components/ResultStep";
import { StepIndicator, type StepNumber } from "./components/StepIndicator";
import type { CsvImportState, CsvImportTab } from "./types";

/// state.status (error 時は previousState.status を使う) を 1/2/3 step に正規化。
/// 設計: 55-ui-csv-import.md §55.2 派生値 currentStep
function computeCurrentStep(status: CsvImportState["status"]): StepNumber {
  switch (status) {
    case "idle":
    case "parsing":
      return 1;
    case "preview":
      return 2;
    case "importing":
    case "result":
      return 3;
    case "error":
      // error は呼び出し側で previousState.status に展開済の想定。万一直接来た場合は idle 扱い。
      return 1;
  }
}

export interface CsvImportPageProps {
  /** search `tab`: 開いたときのタブ（初期選択だけ。切り替えても URL は書き換えない。55 §55.11） */
  tab?: CsvImportTab;
}

const DEFAULT_TAB: CsvImportTab = "daily-report";
const Z004_TAB: CsvImportTab = "z004";

export function CsvImportPage({ tab }: CsvImportPageProps) {
  // §55.7: Z004 の取消の応答を待つ間と、日報の取込み・取消の応答を待つ間（UI-07-D15 (6)）は
  // タブを切り替えない（router の遷移でなく blocker が効かず、選ばれていないタブは unmount されて flow ごと消える）
  const [isRollingBack, setIsRollingBack] = useState(false);
  const [isDailyReportBlocking, setIsDailyReportBlocking] = useState(false);
  const tabsLocked = isRollingBack || isDailyReportBlocking;
  return (
    <PageShell>
      <PageHeader
        title="売上データ取込み"
        subtitle="日報（Z001/Z002/Z005）と商品別CSV（Z004）を分けて取り込みます"
      />
      <Tabs defaultValue={tab ?? DEFAULT_TAB} className="w-full">
        <TabsList>
          <TabsTrigger value={DEFAULT_TAB} disabled={tabsLocked}>
            日報取込み
          </TabsTrigger>
          <TabsTrigger value={Z004_TAB} disabled={tabsLocked}>
            商品別CSV取込み（Z004）
          </TabsTrigger>
        </TabsList>
        <TabsContent value={DEFAULT_TAB}>
          <DailyReportImportPage onBlockingChange={setIsDailyReportBlocking} />
        </TabsContent>
        <TabsContent value={Z004_TAB} className="space-y-4">
          {/* 結果の画面を離れた後に誤った版に気づいたときの入口（55 §55.11、65 §65.5a） */}
          <p className="text-sm">
            <Link
              to="/inventory/records"
              search={{ recordType: "csv_import" }}
              className="font-medium text-primary underline underline-offset-4"
            >
              取り込んだ記録を見る（取り消すときはここから）
            </Link>
          </p>
          <CsvImportFlowPanel onRollingBackChange={setIsRollingBack} />
        </TabsContent>
      </Tabs>
    </PageShell>
  );
}

function CsvImportFlowPanel({
  onRollingBackChange,
}: {
  onRollingBackChange: (isRollingBack: boolean) => void;
}) {
  const flow = useCsvImportFlow();
  const { state, isRollingBack } = flow;
  useEffect(() => {
    onRollingBackChange(isRollingBack);
  }, [isRollingBack, onRollingBackChange]);

  // error variant の currentStep は previousState を見る。reducer 側で recoverTo に応じて
  // previousState を構築するため、importing 中の失敗でも recoverTo="preview" なら previousState
  // は再構築された preview variant となり step=2、recoverTo="idle" なら importing variant のまま
  // で step=3 が表示される (§55.2 import_failed reducer 経路の派生挙動)
  const visibleStatus = state.status === "error" ? state.previousState.status : state.status;
  const currentStep = computeCurrentStep(visibleStatus);

  // a11y: HTML5 main landmark は RootLayout の <main> が持つため <div> を採用 (HomePage.tsx と同方針)
  return (
    <div className="space-y-6">
      <StepIndicator currentStep={currentStep} />

      {renderBody(flow)}
    </div>
  );
}

/// state.status 6 variant の網羅描画。error は ErrorState で上書き、それ以外は対応 step component。
/// switch 内の exhaustiveness は TypeScript の discriminated union で静的検査される。
function renderBody(flow: ReturnType<typeof useCsvImportFlow>) {
  const {
    state,
    selectFile,
    confirmImport,
    rollback,
    rollbackResult,
    dismissError,
    isParsing,
    isImporting,
    isRollingBack,
  } = flow;

  const handleFileSelect = (file: PickedFile) => {
    selectFile(file);
  };

  switch (state.status) {
    case "error":
      return (
        <ErrorState error={state.error} recoverTo={state.recoverTo} onDismiss={dismissError} />
      );
    case "idle":
    case "parsing":
      return (
        <div className="space-y-6">
          {state.status === "idle" && rollbackResult !== null ? (
            <RollbackResultCard result={rollbackResult} />
          ) : null}
          <ParseStep isParsing={isParsing} onFileSelect={handleFileSelect} />
        </div>
      );
    case "preview":
      return (
        <PreviewStep
          preview={state.preview}
          filename={state.filename}
          onConfirm={confirmImport}
          onReselect={handleFileSelect}
          isImporting={isImporting}
        />
      );
    case "importing":
      return <ImportingStep filename={state.filename} />;
    case "result":
      return (
        <ResultStep
          result={state.result}
          settlementDate={state.settlementDate}
          filename={state.filename}
          onRollback={() => {
            rollback(state.result.csv_import_id);
          }}
          isRollingBack={isRollingBack}
        />
      );
  }
}
