// src/features/ej-receipts/types.ts
//
// UI-09c レシートの控え画面の search（route は import する）。
// 設計: docs/SCREEN_DESIGN.md UI-09c-D1、docs/function-design/80-ui-ej-receipts.md §80.2

import { z } from "zod";

export const ejReceiptsSearchSchema = z.object({
  // 形と暦日の両方を見る（`2026-02-30`・`2026-13-01` を捨てる）。不正な値は既定（記録のある最も新しい日）へ
  date: z.iso.date().optional().catch(undefined),
  // 6 桁までの数字の文字列だけ。Router が数に読んだ値（`?no=12`）は捨てる（番号の URL はアプリの中の link だけが作る）
  no: z
    .string()
    .regex(/^[0-9]{1,6}$/)
    .optional()
    .catch(undefined),
});

export type EjReceiptsSearch = z.output<typeof ejReceiptsSearchSchema>;
