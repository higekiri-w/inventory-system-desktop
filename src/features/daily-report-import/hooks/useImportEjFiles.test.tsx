// 手で選ぶ EJ の共有の hook（80 §80.7・55 §55.7、UI-09c-D5 / UI-07-D17）
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import { toast } from "sonner";
import { open } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import { commands, CSV_IMPORT_FILE_SIZE_LIMIT } from "@/lib/bindings";
import { useImportEjFiles } from "./useImportEjFiles";

vi.mock("@tanstack/react-router", () => ({ useBlocker: vi.fn() }));
vi.mock("sonner", () => ({ toast: { error: vi.fn(), success: vi.fn() } }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
vi.mock("@tauri-apps/plugin-fs", () => ({ readFile: vi.fn() }));
vi.mock("@/lib/bindings", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/bindings")>();
  return { ...actual, commands: { importEjFiles: vi.fn() } };
});

function wrapper({ children }: { children: ReactNode }) {
  return <QueryClientProvider client={new QueryClient()}>{children}</QueryClientProvider>;
}

describe("useImportEjFiles UI-09c-D5 REQ-908", () => {
  it("U10: 20MB を超えるファイルは importEjFiles を呼ばず、上限の文を出して待ちを解く", async () => {
    vi.mocked(open).mockResolvedValue(["C:\\tmp\\EJ261119.TXT"]);
    // 20MB を実際に確保しない（hook が見るのは byteLength だけ）
    vi.mocked(readFile).mockResolvedValue({
      byteLength: CSV_IMPORT_FILE_SIZE_LIMIT + 1,
    } as Uint8Array<ArrayBuffer>);
    const { result } = renderHook(() => useImportEjFiles(), { wrapper });

    await act(() => result.current.choose());

    expect(vi.mocked(commands.importEjFiles)).not.toHaveBeenCalled();
    expect(result.current.error).toBe("ファイルサイズが上限(20MB)を超えています");
    expect(vi.mocked(toast.error)).toHaveBeenCalledWith("ファイルサイズが上限(20MB)を超えています");
    expect(result.current.pending).toBe(false);
  });
});
