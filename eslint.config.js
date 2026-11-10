// @ts-check
import js from "@eslint/js";
import tseslint from "typescript-eslint";
import react from "eslint-plugin-react";
import reactHooks from "eslint-plugin-react-hooks";
import jsxA11y from "eslint-plugin-jsx-a11y";
import prettier from "eslint-config-prettier";
import globals from "globals";

export default tseslint.config(
  {
    ignores: [
      "dist/**",
      "node_modules/**",
      "src-tauri/**",
      "src/routeTree.gen.ts",
      "src/lib/bindings.ts",
      // .agents/ と .claude/ はハーネス内部運用（.prettierignore と揃える）
      ".agents/**",
      ".claude/**",
      // .local/ は owner の手元だけの file（.gitignore 済み）。flat config は .gitignore を読まないので明記する（prettier は .gitignore を読む）
      ".local/**",
    ],
  },
  // Type-aware linting は TS ファイル限定（JS には適用しない）
  {
    files: ["**/*.{ts,tsx}"],
    extends: [
      js.configs.recommended,
      ...tseslint.configs.strictTypeChecked,
      ...tseslint.configs.stylisticTypeChecked,
    ],
    languageOptions: {
      parserOptions: {
        project: ["./tsconfig.json", "./tsconfig.node.json"],
        tsconfigRootDir: import.meta.dirname,
      },
      globals: { ...globals.browser },
    },
    plugins: { react, "react-hooks": reactHooks, "jsx-a11y": jsxA11y },
    settings: { react: { version: "detect" } },
    rules: {
      ...react.configs.flat.recommended.rules,
      ...react.configs.flat["jsx-runtime"].rules,
      "react-hooks/rules-of-hooks": "error",
      "react-hooks/exhaustive-deps": "warn",
      "react-hooks/refs": "error",
      ...jsxA11y.flatConfigs.recommended.rules,
    },
  },
  // JS ファイル（eslint.config.js など）は type-aware ルール無効
  {
    files: ["**/*.{js,mjs,cjs}"],
    extends: [js.configs.recommended, tseslint.configs.disableTypeChecked],
    languageOptions: { globals: { ...globals.node } },
  },
  // vite.config.ts は Node globals も必要
  {
    files: ["vite.config.ts"],
    languageOptions: { globals: { ...globals.node } },
  },
  // Vitest test files: describe / it / expect / vi 等の globals を no-undef から除外
  // Phase 1 7-7a Vitest 初期化、`globals` package の vitest namespace を使用
  {
    files: ["src/**/*.test.{ts,tsx}", "vitest.config.ts", "src/test/**/*.ts"],
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.vitest,
      },
    },
  },
  // PR-C C3 (i): palette 外の生 Tailwind 色 class と生 <button> を禁止
  // （DSR-08 / docs/design-system/00-foundations.md / catalog ③ sort header）。
  // stone は palette 内のため ban list に含めない。token utility（bg-warning-soft 等）は
  // <family>-<数値shade> パターン非該当で誤検出しない。test ファイルは class assert 文字列や
  // ダミー <button> が引っかかるため除外。動的 shade 補間（`bg-${c}-${n}`）は AST Literal に
  // ならず検出外（ast-grep 将来項目、design-system/README.md 参照）。生 primitive の初期 scope は
  // <button> 限定（親 packet C-lint-2、<input>/<select> は棚卸しの上で段階拡大判断）。
  {
    files: ["src/features/**/*.{ts,tsx}", "src/components/patterns/**/*.{ts,tsx}"],
    ignores: ["src/features/**/*.test.{ts,tsx}", "src/components/patterns/**/*.test.{ts,tsx}"],
    rules: {
      "no-restricted-syntax": [
        "error",
        {
          selector:
            "Literal[value=/\\b(amber|rose|emerald|red|green|orange|yellow|lime|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|slate|gray|zinc|neutral)-(50|100|200|300|400|500|600|700|800|900|950)\\b/]",
          message:
            "palette 外の生 Tailwind 色 class は禁止。docs/design-system/00-foundations.md の semantic token（bg-warning-soft / text-success-emphasis / text-destructive 等）を使うこと（DSR-08 / PR-C）。",
        },
        {
          selector: "JSXOpeningElement[name.name='button']",
          message:
            "生 <button> は禁止。shadcn Button（@/components/ui/button）を使うこと（PR-C C-lint-2、style 中立化の前例は SortableHeader 置換参照）。",
        },
      ],
    },
  },
  // ⑫ 衛生 batch 2 S2: palette 外の生 Tailwind 色 class を ui/ layout/ へも拡張（DSR-08）。
  // 既存 block（features/ patterns/）は無変更のまま維持し、files が重複しないこの新規
  // block で色 selector のみを追加する（ESLint flat config は同一 files に一致する block 間で
  // no-restricted-syntax を完全置換するため、生 <button> selector はここに含めない —
  // ui/ layout/ は primitive 層で生 <button> の実装が正当なため raw-button ban の scope外）。
  {
    files: ["src/components/ui/**/*.{ts,tsx}", "src/components/layout/**/*.{ts,tsx}"],
    ignores: ["src/components/ui/**/*.test.{ts,tsx}", "src/components/layout/**/*.test.{ts,tsx}"],
    rules: {
      "no-restricted-syntax": [
        "error",
        {
          selector:
            "Literal[value=/\\b(amber|rose|emerald|red|green|orange|yellow|lime|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|slate|gray|zinc|neutral)-(50|100|200|300|400|500|600|700|800|900|950)\\b/]",
          message:
            "palette 外の生 Tailwind 色 class は禁止。docs/design-system/00-foundations.md の semantic token（bg-warning-soft / text-success-emphasis / text-destructive 等）を使うこと（DSR-08 / PR-C）。",
        },
      ],
    },
  },
  // 衛生 batch 3 S1: eslint 保守性 rule（complexity/max-depth/max-lines-per-function/max-params）。
  // phase 1 は features/ patterns/（test 除外）へ warn のみで導入し、既存 3 rule block とは
  // 異なる rule id のため files が既存 block（:79）と重複しても rule merge の完全置換に抵触しない
  // （10a 起票時実測 §1・§7 で実効設定差分 0 を確認）。閾値は既存負債を一度に押し付けない暫定開始値
  // であり、良い設計の目標値ではない（段階強化は Backlog）。
  {
    files: ["src/features/**/*.{ts,tsx}", "src/components/patterns/**/*.{ts,tsx}"],
    ignores: ["src/features/**/*.test.{ts,tsx}", "src/components/patterns/**/*.test.{ts,tsx}"],
    rules: {
      complexity: ["warn", 40],
      "max-depth": ["warn", 3],
      "max-lines-per-function": [
        "warn",
        { max: 650, skipBlankLines: true, skipComments: true, IIFEs: true },
      ],
      "max-params": ["warn", 4],
    },
  },
  // PR-C C3 (ii): patterns/ ui/ の barrel index.ts 作成を禁止（直接 path import 統一の恒久化、
  // prior art PR #48 c5f3786 の invoke-fallback 限定形を任意 source 形へ一般化）。
  // 両 index.ts は現在不在 → 作成された瞬間に lint error になる予防 gate。
  {
    files: ["src/components/patterns/index.ts", "src/components/ui/index.ts"],
    rules: {
      "no-restricted-syntax": [
        "error",
        {
          selector: "ExportNamedDeclaration[source.value=/.*/]",
          message:
            "patterns/ ui/ に barrel index.ts を作らない。各 component を直接 path import すること（PR-C）。",
        },
        {
          selector: "ExportAllDeclaration[source.value=/.*/]",
          message: "patterns/ ui/ に barrel index.ts を作らない（PR-C）。",
        },
      ],
    },
  },
  prettier,
);
