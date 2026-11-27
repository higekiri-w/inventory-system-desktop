import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

// UI-12 §52.1 / UI-CTX-D1: 全画面と root の errorComponent に効かせる mount の位置を source で見張る
describe("TextFieldContextMenu の mount (UI-12 / UI-CTX-D1)", () => {
  it("UI-CTX-D1: src/main.tsx が RouterProvider を TextFieldContextMenu で包む", () => {
    const source = readFileSync(join(process.cwd(), "src/main.tsx"), "utf8");
    expect(source).toMatch(
      /<TextFieldContextMenu>\s*<RouterProvider router=\{router\} \/>\s*<\/TextFieldContextMenu>/,
    );
  });

  it("UI-CTX-D3: clipboard の plugin を登録し、capability は文字の読取りだけを許す", () => {
    const lib = readFileSync(join(process.cwd(), "src-tauri/src/lib.rs"), "utf8");
    expect(lib).toContain(".plugin(tauri_plugin_clipboard_manager::init())");
    const capability = JSON.parse(
      readFileSync(join(process.cwd(), "src-tauri/capabilities/default.json"), "utf8"),
    ) as { permissions: string[] };
    // D-177: 書込み・画像の読取り・消去は許さない
    expect(capability.permissions.filter((p) => p.startsWith("clipboard-manager:"))).toEqual([
      "clipboard-manager:allow-read-text",
    ]);
  });
});
