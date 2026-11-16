// src/features/home/components/OffsiteBackupWarning.tsx
//
// UI-00-D12: PC の外の控えが古いときのホームの知らせ。stale = true のときだけ warning の Alert を出し、
// 原因の文はバックアップ画面の card と同じ選び方（「バックアップ画面の」を付ける）。日数は計算しない。
// query の失敗は Alert を出さない（toast は HomePage、53 §53.5）。
// 設計: docs/function-design/53-ui-home.md §53.1 / §53.5 UI-00-D12

import { Link } from "@tanstack/react-router";
import { AlertTriangle } from "lucide-react";

import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import type { OffsiteBackupStatus } from "@/lib/bindings";
import { offsiteCauseText } from "@/features/backup-restore/offsite-status-text";

export function OffsiteBackupWarning({ status }: { status: OffsiteBackupStatus | undefined }) {
  const cause = status ? offsiteCauseText(status, "home") : null;
  if (cause === null) return null;
  return (
    <Alert variant="warning">
      <AlertTriangle />
      <AlertDescription className="flex flex-wrap items-center justify-between gap-2">
        <span>{cause}</span>
        <Button variant="outline" size="sm" asChild>
          <Link to="/settings/backup">バックアップ画面へ</Link>
        </Button>
      </AlertDescription>
    </Alert>
  );
}
