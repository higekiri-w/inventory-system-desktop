// src/features/backup-restore/useOffsiteBackupStatus.ts
//
// UI-11b-D14・UI-00-D12: PC の外の控えの status の読取り（card とホームが同じ query key）。
// invalidate はしない（useAutoBackupCheck と BackupRestorePage が持つ、§68.10）。

import { useQuery } from "@tanstack/react-query";

import { commands } from "@/lib/bindings";
import { unwrapResult } from "@/lib/invoke";
import { queryKeys } from "@/lib/query-keys";

export function useOffsiteBackupStatus() {
  return useQuery({
    queryKey: queryKeys.backupRestore.offsiteStatus(),
    queryFn: () =>
      unwrapResult(commands.getOffsiteBackupStatus(), {
        source: "commands",
        cmd: "get_offsite_backup_status",
      }),
  });
}
