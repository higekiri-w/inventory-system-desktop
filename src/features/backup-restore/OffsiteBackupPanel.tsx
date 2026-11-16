// src/features/backup-restore/OffsiteBackupPanel.tsx
//
// UI-11b-D14: バックアップ画面の「PC の外の控え」の card（表示だけ。command の呼出しと invalidate は
// BackupRestorePage が持つ、§68.10）。状態の文と原因の文は offsite-status-text.ts の 1 か所から選ぶ。
// 設計: docs/function-design/68-ui-backup-restore.md UI-11b-D14 / §68.4 / §68.7 / §68.9

import { AlertTriangle, CheckCircle2, FileSearch, Loader2, RotateCcw, Usb } from "lucide-react";
import { useState } from "react";

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
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { OffsiteBackupStatus } from "@/lib/bindings";
import {
  offsiteAttachedText,
  offsiteCauseText,
  offsiteFailureText,
  offsiteStateLine,
} from "./offsite-status-text";

export type OffsiteAction = "preparing" | "resetting" | "stopping";

export interface OffsiteMessage {
  tone: "success" | "error";
  text: string;
}

export interface OffsiteBackupPanelProps {
  status: OffsiteBackupStatus | undefined;
  isError: boolean;
  onRetry: () => void;
  /** 用意・作り直し・やめるの 3 つ（復元の選択中・busy / fatal の間は disabled、§68.7） */
  operationsDisabled: boolean;
  /** 「控えを選んで確かめる」（busy / fatal の間は disabled、§68.7） */
  fileEntryDisabled: boolean;
  busy: OffsiteAction | null;
  message: OffsiteMessage | null;
  onPrepare: () => void;
  onReset: () => void;
  /** 確認の AlertDialog で「やめる」を押した後に呼ぶ */
  onStop: () => void;
  onChooseFile: () => void;
}

export function OffsiteBackupPanel({
  status,
  isError,
  onRetry,
  operationsDisabled,
  fileEntryDisabled,
  busy,
  message,
  onPrepare,
  onReset,
  onStop,
  onChooseFile,
}: OffsiteBackupPanelProps) {
  const [confirmStopOpen, setConfirmStopOpen] = useState(false);
  const cause = status ? offsiteCauseText(status, "card") : null;
  const attachedText = status ? offsiteAttachedText(status) : null;

  return (
    <Card>
      <CardHeader>
        <CardTitle>PC の外の控え</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {isError ? (
          <div className="space-y-2" role="status">
            <p className="text-sm">PC の外の控えの状態を取得できませんでした</p>
            <Button type="button" variant="outline" size="sm" onClick={onRetry}>
              <RotateCcw />
              再読込
            </Button>
          </div>
        ) : null}
        {!isError && !status ? (
          <p className="text-sm text-muted-foreground">読込み中です。</p>
        ) : null}

        {status ? (
          <div className="space-y-2">
            <p className="font-medium">{offsiteStateLine(status)}</p>
            {cause ? (
              <Alert variant="warning">
                <AlertTriangle />
                <AlertDescription>
                  <p>{cause}</p>
                </AlertDescription>
              </Alert>
            ) : null}
            {status.prepared ? (
              <div className="space-y-1 text-sm text-muted-foreground">
                {attachedText ? <p>{attachedText}</p> : null}
                {status.attached.map((medium) => (
                  <p key={medium.drive_root} className="text-xs break-all">
                    {medium.label}: {medium.drive_root}
                  </p>
                ))}
                {status.last_failure_kind ? (
                  <p>{offsiteFailureText(status.last_failure_kind)}</p>
                ) : null}
              </div>
            ) : null}
          </div>
        ) : null}

        <div className="flex flex-wrap gap-2">
          <Button type="button" onClick={onPrepare} disabled={operationsDisabled}>
            {busy === "preparing" ? <Loader2 className="animate-spin" /> : <Usb />}
            この USB メモリを控えの保存先にする
          </Button>
          <Button
            type="button"
            variant="outline"
            onClick={onChooseFile}
            disabled={fileEntryDisabled}
          >
            <FileSearch />
            控えを選んで確かめる
          </Button>
          {status?.last_failure_kind === "state_unreadable" ? (
            <Button type="button" variant="outline" onClick={onReset} disabled={operationsDisabled}>
              {busy === "resetting" ? <Loader2 className="animate-spin" /> : null}
              記録を作り直す
            </Button>
          ) : null}
          {status?.prepared ? (
            <Button
              type="button"
              variant="outline"
              onClick={() => {
                setConfirmStopOpen(true);
              }}
              disabled={operationsDisabled}
            >
              {busy === "stopping" ? <Loader2 className="animate-spin" /> : null}
              PC の外の控えをやめる
            </Button>
          ) : null}
        </div>

        {message ? (
          message.tone === "success" ? (
            <Alert className="border-success bg-success-soft text-success-strong">
              <CheckCircle2 />
              <AlertTitle>{message.text}</AlertTitle>
            </Alert>
          ) : (
            <Alert variant="destructive">
              <AlertTitle>操作に失敗しました</AlertTitle>
              <AlertDescription>
                <p>{message.text}</p>
              </AlertDescription>
            </Alert>
          )
        ) : null}
      </CardContent>

      <AlertDialog open={confirmStopOpen} onOpenChange={setConfirmStopOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>PC の外の控えをやめる</AlertDialogTitle>
            <AlertDialogDescription>
              PC の外の控えをやめますか。USB メモリへの写しとホームの知らせが止まります。USB
              メモリの上の控えと PC の中のバックアップは消えません。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>キャンセル</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                setConfirmStopOpen(false);
                onStop();
              }}
            >
              やめる
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </Card>
  );
}
