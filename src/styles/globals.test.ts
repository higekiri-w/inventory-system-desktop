// src/styles/globals.test.ts
//
// SC1: token 4 件（--border-strong / --border / --input / --list-head）+
// @theme map 2 件の fs literal oracle（--row-current は runtime lane A で撤去）。値は独立転記（production 定数を import しない）。
// 共有部品の contract test。traceability 上は Lane 2 pilot = UI-01a へ紐付け（Gated Amendment 1）。
// Plan Packet: docs/plans/2026-09-03-ui-list-backbone-d-lane2.md S1 / D-7

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

const REPO_ROOT = join(__dirname, "../..");
const GLOBALS_CSS = readFileSync(join(REPO_ROOT, "src/styles/globals.css"), "utf8");
const INPUT_TSX = readFileSync(join(REPO_ROOT, "src/components/ui/input.tsx"), "utf8");
const SELECT_TSX = readFileSync(join(REPO_ROOT, "src/components/ui/select.tsx"), "utf8");

describe("SC1: globals.css declares border-strong / border / input / list-head tokens (UI-01a pilot / 共有 token)", () => {
  it("declares the four :root token literals", () => {
    expect(GLOBALS_CSS).toContain("--border-strong: #8a8480;");
    expect(GLOBALS_CSS).toContain("--border: #cdc8c4;");
    expect(GLOBALS_CSS).toContain("--input: var(--border-strong);");
    expect(GLOBALS_CSS).toContain("--list-head: #e7e5e4;");
  });

  it("maps the tokens in @theme inline", () => {
    expect(GLOBALS_CSS).toContain("--color-border-strong: var(--border-strong);");
    expect(GLOBALS_CSS).toContain("--color-list-head: var(--list-head);");
  });

  // SPEC-COLOR-EMPHASIS-RT-1 / D-CE6: 現在行は進行中の token へ移し、--row-current は消す。
  it("no longer declares the retired --row-current token", () => {
    expect(GLOBALS_CSS).not.toContain("--row-current");
    expect(GLOBALS_CSS).not.toContain("--color-row-current");
  });
});

// SPEC-COLOR-EMPHASIS-RT-1（D-091 / D-CE1 / D-CE16）: 操作の色・進行中の家族・muted の文字色。
// 値は独立転記（production 定数を import しない）。
describe("SPEC-COLOR-EMPHASIS-RT-1: operation / ongoing / muted token literals (UI-12)", () => {
  it("sets --primary and --ring to the operation color #1d5c63", () => {
    expect(GLOBALS_CSS.toLowerCase()).toContain("--primary: #1d5c63;");
    expect(GLOBALS_CSS.toLowerCase()).toContain("--ring: #1d5c63;");
    expect(GLOBALS_CSS).toContain("--primary-foreground: #fafaf9;");
  });

  it("declares the four ongoing tokens in :root", () => {
    const css = GLOBALS_CSS.toLowerCase();
    expect(css).toContain("--ongoing: #2f7f86;");
    expect(css).toContain("--ongoing-soft: #e6f0f0;");
    expect(css).toContain("--ongoing-border: #7fb0b4;");
    expect(css).toContain("--ongoing-strong: #123e43;");
  });

  it("maps the four ongoing tokens in @theme inline", () => {
    expect(GLOBALS_CSS).toContain("--color-ongoing: var(--ongoing);");
    expect(GLOBALS_CSS).toContain("--color-ongoing-soft: var(--ongoing-soft);");
    expect(GLOBALS_CSS).toContain("--color-ongoing-border: var(--ongoing-border);");
    expect(GLOBALS_CSS).toContain("--color-ongoing-strong: var(--ongoing-strong);");
  });

  it("keeps --warning-emphasis for caution only (value unchanged)", () => {
    expect(GLOBALS_CSS).toContain("--warning-emphasis: #b45309;");
  });

  // D-094（L3 round 1 で D-CE13 の試しを採らない）: ランキング 1 位の琥珀の pill と行の地の token を残す。
  it("keeps the rank-top tokens for the ranking top row", () => {
    expect(GLOBALS_CSS).toContain("--rank-top-bg: #fffbeb;");
    expect(GLOBALS_CSS).toContain("--rank-top-badge-bg: #fef3c7;");
    expect(GLOBALS_CSS).toContain("--rank-top-badge-text: #92400e;");
    expect(GLOBALS_CSS).toContain("--color-rank-top-bg: var(--rank-top-bg);");
    expect(GLOBALS_CSS).toContain("--color-rank-top-badge-bg: var(--rank-top-badge-bg);");
    expect(GLOBALS_CSS).toContain("--color-rank-top-badge-text: var(--rank-top-badge-text);");
  });

  // D-CE16 / D-094: owner が L3 round 1 で候補 3 つ（M1 #6f6964 / M2 #6b6560 / M3 #57534e）から選んだ M2。
  it("sets --muted-foreground to the chosen darker stone", () => {
    expect(GLOBALS_CSS).toContain("--muted-foreground: #6b6560;");
  });
});

// SC13（Gated Amendment 3 追補 S16、Opus P1-1 / AC-L3-4）: forced-colors では box-shadow が
// 無効化され border 色も system color に置換されるため、`outline-none` + `focus-visible:border-ring`
// だけでは focus が消える。`@layer` 外（unlayered）の `:focus-visible` outline で
// `@layer utilities` の `outline-none` に勝つ。
// closure round 2 S20（Opus P2-1）: 「`@layer base {` より前」という位置プロキシは
// `@layer base { ... }` の出現位置に依存し揺れる。block 開始位置までの file 文字列で
// `{` / `}` の対応数が 0（nesting depth 0 = どの block にも入っていない）ことを直接検査する
// 形に替え、`outline: 2px solid Highlight` を literal で pin する
// （`outline: 0 solid Highlight` のような値差し替えを素通ししない）。
describe("SC13: globals.css declares an unlayered forced-colors focus outline (Gated Amendment 3 追補 S16 / AC-L3-4 / closure round 2 S20)", () => {
  it("declares exactly one forced-colors media block at brace nesting depth 0, pinning the Highlight outline literal", () => {
    const mediaLiteral = "@media (forced-colors: active)";
    const occurrences = GLOBALS_CSS.split(mediaLiteral).length - 1;
    expect(occurrences).toBe(1);

    const mediaIndex = GLOBALS_CSS.indexOf(mediaLiteral);
    expect(mediaIndex).toBeGreaterThan(-1);

    // block 開始位置までの file 文字列で `{` / `}` の対応数を数え、0（nesting depth 0）
    // であることを直接検査する（どの @layer / block にも入っていない）。
    const beforeMedia = GLOBALS_CSS.slice(0, mediaIndex);
    const opensBefore = (beforeMedia.match(/{/g) ?? []).length;
    const closesBefore = (beforeMedia.match(/}/g) ?? []).length;
    expect(opensBefore - closesBefore).toBe(0);

    // block は `{` から対応する `}` まで（prettier の折返しで複数行になり得るため
    // 改行ではなく波括弧の対応を数えて block の終端を求める）。
    const openIndex = GLOBALS_CSS.indexOf("{", mediaIndex);
    expect(openIndex).toBeGreaterThan(-1);
    let depth = 0;
    let closeIndex = -1;
    for (let i = openIndex; i < GLOBALS_CSS.length; i += 1) {
      if (GLOBALS_CSS[i] === "{") depth += 1;
      else if (GLOBALS_CSS[i] === "}") {
        depth -= 1;
        if (depth === 0) {
          closeIndex = i;
          break;
        }
      }
    }
    expect(closeIndex).toBeGreaterThan(-1);
    const block = GLOBALS_CSS.slice(mediaIndex, closeIndex + 1);
    expect(block).toContain(":focus-visible");
    // `outline: 2px solid Highlight` を literal で pin する（prettier の空白差には
    // robust にしつつ、`2px solid Highlight` の値は literal 固定で「0」等への
    // 差し替えを素通ししない）。
    expect(block).toMatch(/outline:\s*2px solid Highlight/);
  });
});

// SC15（Gated Amendment 6 S44、owner run 2 所感の要望化。Gated Amendment 7 S46 で
// owner run 6「検索欄と Select の色は FAFAF9 で」により #ffffff → #fafaf9 へ更新）:
// 入力欄・Select の操作面を独立 token（--background と同値だが分離）にする。
// file:bg-transparent（input の file-selector-button 疑似要素）
// は対象外の別 utility のため不変、base の bg-transparent（border-input と対になる面）のみ置換する。
describe("SC15: control-surface token + input/select surface (Gated Amendment 6 S44 / Amendment 7 S46)", () => {
  it("declares --control-surface: #fafaf9 in :root and maps it in @theme inline", () => {
    expect(GLOBALS_CSS).toContain("--control-surface: #fafaf9;");
    expect(GLOBALS_CSS).toContain("--color-control-surface: var(--control-surface);");
  });

  it("input.tsx uses bg-control-surface for the base surface, keeps file:bg-transparent and dark:bg-input/30", () => {
    expect(INPUT_TSX).toContain("border-input bg-control-surface");
    expect(INPUT_TSX).not.toContain("border-input bg-transparent");
    expect(INPUT_TSX).toContain("file:bg-transparent");
    expect(INPUT_TSX).toContain("dark:bg-input/30");
  });

  it("select.tsx SelectTrigger uses bg-control-surface for the base surface, keeps dark:bg-input/30", () => {
    expect(SELECT_TSX).toContain("border-input bg-control-surface");
    expect(SELECT_TSX).not.toContain("border-input bg-transparent");
    expect(SELECT_TSX).toContain("dark:bg-input/30");
  });
});

// SC17（Gated Amendment 7 S48、owner run 6「最小幅で部門と売価の間に白い細線」）:
// ListShell の thead tr 背景（seam 対策）を復活すると角丸の外側に四角く覗くため、
// list-shell-sticky hook（root class、ListShell.test.tsx で検証）配下の th 角にだけ
// page 色（--background）の corner mask を重ねる。SC13 と同じ手法で `@layer` 外（unlayered）
// であることを brace 対応数で直接検査する。
describe("SC17: globals.css declares an unlayered corner mask for the list-shell-sticky thead corners (Gated Amendment 7 S48)", () => {
  const selectors = [
    ".list-shell-sticky thead th:first-child::before",
    ".list-shell-sticky thead th:last-child::after",
  ];

  it("declares both corner mask rules at brace nesting depth 0 with radial-gradient + var(--background)", () => {
    for (const selector of selectors) {
      const occurrences = GLOBALS_CSS.split(selector).length - 1;
      expect(occurrences).toBeGreaterThanOrEqual(1);

      // 各 selector は shared block（content/position/top/width/height/pointer-events）と
      // 個別の background 宣言の 2 箇所に出現するため、最後（background を持つ方）を検査する。
      const selectorIndex = GLOBALS_CSS.lastIndexOf(selector);
      expect(selectorIndex).toBeGreaterThan(-1);

      const before = GLOBALS_CSS.slice(0, selectorIndex);
      const opensBefore = (before.match(/{/g) ?? []).length;
      const closesBefore = (before.match(/}/g) ?? []).length;
      expect(opensBefore - closesBefore).toBe(0);

      const openIndex = GLOBALS_CSS.indexOf("{", selectorIndex);
      expect(openIndex).toBeGreaterThan(-1);
      let depth = 0;
      let closeIndex = -1;
      for (let i = openIndex; i < GLOBALS_CSS.length; i += 1) {
        if (GLOBALS_CSS[i] === "{") depth += 1;
        else if (GLOBALS_CSS[i] === "}") {
          depth -= 1;
          if (depth === 0) {
            closeIndex = i;
            break;
          }
        }
      }
      expect(closeIndex).toBeGreaterThan(-1);
      const block = GLOBALS_CSS.slice(selectorIndex, closeIndex + 1);
      expect(block).toContain("radial-gradient(");
      expect(block).toContain("var(--background)");
    }
  });
});

// UI-09a-D17・D18（日報の紙）: 値は 56 UI-09a-D18 から独立転記。
describe("UI-09a: daily report sheet print rules", () => {
  it("globals.css declares A4 page and hides app root in print (UI-09a)", () => {
    expect(GLOBALS_CSS).toContain("size: A4 portrait;");
    expect(GLOBALS_CSS).toContain("margin: 10mm;");
    const printIndex = GLOBALS_CSS.indexOf("@media print {");
    expect(printIndex).toBeGreaterThan(-1);
    const printBlock = GLOBALS_CSS.slice(printIndex);
    expect(printBlock).toMatch(/#root\s*\{\s*display:\s*none;\s*\}/);
    expect(printBlock).toContain("overflow: visible;");
  });
});
