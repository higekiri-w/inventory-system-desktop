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
});
