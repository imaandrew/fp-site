import js from "@eslint/js";
import eslintConfigPrettier from "eslint-config-prettier";
import svelte from "eslint-plugin-svelte";
import { defineConfig } from "eslint/config";
import globals from "globals";
import ts from "typescript-eslint";

import svelteConfig from "./svelte.config.js";

export default defineConfig(
  js.configs.recommended,
  ...ts.configs.strictTypeChecked,
  ...ts.configs.stylisticTypeChecked,
  ...svelte.configs.recommended,
  ...svelte.configs.prettier,
  eslintConfigPrettier,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
      },
    },
  },
  {
    files: ["src/**/*.{ts,svelte}"],
    languageOptions: {
      globals: globals.browser,
    },
  },
  {
    files: ["*.config.{js.ts}"],
    languageOptions: {
      globals: globals.node,
    },
  },
  {
    files: ["src/**/worker.ts", "src/**/*.worker.ts"],
    languageOptions: {
      globals: globals.worker,
    },
  },
  {
    files: ["**/*.svelte", "**/*.svelte.ts"],
    languageOptions: {
      parserOptions: {
        projectService: true,
        extraFileExtensions: [".svelte"],
        parser: ts.parser,
        svelteConfig,
      },
    },
  },
  {
    ignores: ["node_modules/", "dist/", "wasm/target/", "wasm/pkg/"],
  },
);
