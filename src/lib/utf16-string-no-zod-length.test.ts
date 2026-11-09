import { readFileSync, readdirSync } from "node:fs";
import { extname, join, relative, resolve } from "node:path";
import { describe, expect, it } from "vitest";

function sourceFiles(root: string): string[] {
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const path = join(root, entry.name);
    if (entry.isDirectory()) return sourceFiles(path);
    if (/\.test\.tsx?$/.test(entry.name)) return [];
    return [".ts", ".tsx"].includes(extname(path)) ? [path] : [];
  });
}

// `z.string(…)` の引数（入れ子 1 段の括弧まで）と、間に挟まる別の method の呼出し・空白を許して
// `.max(`・`.min(`・`.length(` を探す。
// ponytail: 1 行に書いた chain だけを見る。複数行に割った chain や、`z.string()` を変数に置いてから
// 繋げる書き方は拾わない（D-151 の lane の時点で 0 件）。そういう書き方が入ったら、TypeScript の AST で見る形に広げる。
const args = String.raw`\((?:[^()\n]|\([^()\n]*\))*\)`;
const zodLength = new RegExp(
  String.raw`\bz\.string${args}(?:\s*\.\w+${args})*\s*\.(?:max|min|length)\(`,
);

function offendingLines(source: string): string[] {
  return source.split("\n").filter((line) => zodLength.test(line));
}

describe("utf16String static boundary (REQ-301 / BIZ-01-D5 / D-151)", () => {
  it("REQ-301 / BIZ-01-D5: 検査の形が合成の source の zod の長さの検査を拾う", () => {
    for (const source of [
      "q: z.string().max(100).optional(),",
      'q: z.string({ error: "x" }).max(1),',
      "q: z.string() .max(1),",
      "q: z.string().trim().min(1),",
      "q: z.string().length(3),",
    ]) {
      expect(offendingLines(source)).toEqual([source]);
    }
    for (const source of [
      "q: utf16String({ max: 100 }).optional().catch(undefined),",
      "returnTo: z.string().optional().catch(undefined),",
      "const n = values.length;",
    ]) {
      expect(offendingLines(source)).toEqual([]);
    }
  });

  it("REQ-301 / BIZ-01-D5: route search の文字列の長さは utf16String で検査する", () => {
    const repoRoot = resolve(import.meta.dirname, "../..");
    const roots = ["src/features", "src/routes", "src/components", "src/lib"].map((dir) =>
      resolve(repoRoot, dir),
    );
    const offenders = roots.flatMap((root) =>
      sourceFiles(root).flatMap((path) =>
        offendingLines(readFileSync(path, "utf8")).map(
          (line) => `${relative(repoRoot, path)}: ${line.trim()}`,
        ),
      ),
    );

    expect(offenders).toEqual([]);
  });
});
