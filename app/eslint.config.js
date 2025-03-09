import typescript from "typescript";
import js from "@eslint/js";
import ts from "@typescript-eslint/eslint-plugin";
import svelte from "eslint-plugin-svelte3";

export default [
  js.configs.recommended,
  {
    files: ["*.ts", "*.svelte"],
    languageOptions: {
      parser: "@typescript-eslint/parser",
      parserOptions: {
        ecmaVersion: 2020,
        sourceType: "module",
        tsconfigRootDir: process.cwd(),
        project: ["./tsconfig.json"],
        extraFileExtensions: [".svelte"]
      }
    },
    plugins: {
      "@typescript-eslint": ts,
      svelte3: svelte
    },
    processor: "svelte3/svelte3",
    settings: {
      "svelte3/typescript": typescript,
      "svelte3/ignore-styles": () => true
    },
    rules: {
      ...ts.configs.recommended.rules,
      ...ts.configs["recommended-requiring-type-checking"].rules
    },
    ignores: ["node_modules", "build", "src-tauri"]
  }
];
