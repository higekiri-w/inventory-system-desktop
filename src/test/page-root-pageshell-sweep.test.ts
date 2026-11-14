// src/test/page-root-pageshell-sweep.test.ts
//
// SC2b: page root 全置換（04 原則 6、D-1）。src/features/**/*Page.tsx（test 除外）の
// root class として p-6 を直書きする箇所が PageShell 経由以外に残っていないことを
// fs scan で保証する。許容 2 箇所（非 root: overlay / card、IntegrityCheckPage.tsx）は
// 件数だけでなく file path + class 文字列の allowlist で固定する（Codex Final Review P2-4）。
// Plan Packet: docs/plans/2026-09-03-ui-list-backbone-d-lane2.md S2
// 共有部品の contract test。traceability 上は Lane 2 pilot = UI-01a へ紐付け（Gated Amendment 1）。

import { readFileSync, readdirSync } from "node:fs";
import { extname, join, relative } from "node:path";
import { describe, expect, it } from "vitest";

const REPO_ROOT = join(__dirname, "../..");
const FEATURES_ROOT = join(REPO_ROOT, "src/features");

// 許容される非 root p-6 direct className（root ではないため PageShell 化の対象外）。
// 件数（2）だけでなく file path + 実 class 文字列そのものを allowlist として固定する
// （件数一致だけでは別の非 root p-6 が紛れ込んでも検出できない、Codex Final Review P2-4）。
const ALLOWED_NON_ROOT_HITS = [
  'src/features/integrity-check/IntegrityCheckPage.tsx: className="absolute inset-0 z-40 flex items-center justify-center bg-background/85 p-6 backdrop-blur-[1px]"',
  'src/features/integrity-check/IntegrityCheckPage.tsx: className="w-full max-w-md space-y-4 rounded-lg border bg-card p-6 text-center shadow-lg"',
];

// PageShell 全置換の機械抽出値（AC2 と突合、機械抽出表は Implementation Results に貼る）。
const EXPECTED_PAGESHELL_FILE_COUNT = 29;
const EXPECTED_PAGESHELL_OCCURRENCE_COUNT = 44;

function collectPageFiles(dir: string): string[] {
  const entries = readdirSync(dir, { withFileTypes: true });
  const files: string[] = [];
  for (const entry of entries) {
    const fullPath = join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...collectPageFiles(fullPath));
    } else if (
      entry.isFile() &&
      extname(entry.name) === ".tsx" &&
      entry.name.endsWith("Page.tsx") &&
      !entry.name.endsWith(".test.tsx")
    ) {
      files.push(fullPath);
    }
  }
  return files;
}

describe("SC2b: no feature page file declares a p-6 root className outside PageShell (UI-01a pilot / 共有 page root)", () => {
  it("p-6 直書き className の hit は非 root allowlist の 2 箇所のみ（file path + class 文字列で一致）", () => {
    const pageFiles = collectPageFiles(FEATURES_ROOT);
    const rootClassPattern = /className="[^"]*\bp-6\b[^"]*"/g;
    const hits: string[] = [];

    for (const file of pageFiles) {
      const source = readFileSync(file, "utf8");
      const matches = source.match(rootClassPattern);
      if (matches) {
        for (const match of matches) {
          hits.push(`${relative(REPO_ROOT, file)}: ${match}`);
        }
      }
    }

    expect(hits.sort()).toEqual([...ALLOWED_NON_ROOT_HITS].sort());
  });

  it("29 file / 44 箇所が PageShell を import して page root に使う", () => {
    const pageFiles = collectPageFiles(FEATURES_ROOT);
    const pageShellOccurrencePattern = /<PageShell\b/g;
    let totalOccurrences = 0;
    let fileCount = 0;

    for (const file of pageFiles) {
      const source = readFileSync(file, "utf8");
      const matches = source.match(pageShellOccurrencePattern);
      if (matches) {
        fileCount += 1;
        totalOccurrences += matches.length;
      }
    }

    expect(fileCount).toBe(EXPECTED_PAGESHELL_FILE_COUNT);
    expect(totalOccurrences).toBe(EXPECTED_PAGESHELL_OCCURRENCE_COUNT);
  });
});

// GA3b-5（Lane 4 Gated Amendment 3、旧 GA1e の反転。GA4b で高さ方針を viewport 基準へ
// 改訂）: stickyHeader の箱は viewport 基準の max-h（`max-h-[calc(100vh-6.75rem)]`）で
// 自立するようになり、page root からの高さ継承（PageShell へ flex/h-full/
// min-h-0/flex-col 系 class を渡すこと）はもう不要——渡したまま残っていると死んだ
// class の残存（GA3b-1 の撤去漏れ）を示す regression になる。fs scan の性質上、
// stickyHeader を渡す JSX と PageShell の JSX が同一 file 内にある場合にのみ機能する
// （限界: 配線を別 file の子 component へ分離する画面はこの guard で検出できない、
// packet「Gated Amendment 1」節参照）。
describe("GA3b-5: stickyHeader を渡す Page は同一 file 内で PageShell に高さ継承 class を渡さない（旧 GA1e の逆転）", () => {
  it("stickyHeader を持つ *Page.tsx はいずれも PageShell へ flex/h-full/min-h-0/flex-col 系 class を渡していない", () => {
    const pageFiles = collectPageFiles(FEATURES_ROOT);
    const violations: string[] = [];

    for (const file of pageFiles) {
      const source = readFileSync(file, "utf8");
      if (!/\bstickyHeader\b/.test(source)) continue;

      const pageShellClassMatches = source.match(/<PageShell\b[^>]*className="([^"]*)"/g) ?? [];
      const hasHeightChain = pageShellClassMatches.some(
        (match) =>
          /\bflex\b/.test(match) &&
          /\bh-full\b/.test(match) &&
          /\bmin-h-0\b/.test(match) &&
          /\bflex-col\b/.test(match),
      );
      if (hasHeightChain) violations.push(relative(REPO_ROOT, file));
    }

    expect(violations).toEqual([]);
  });
});
