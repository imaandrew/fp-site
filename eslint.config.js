import eslint from "@eslint/js";
import eslintConfigPrettier from "eslint-config-prettier";
import eslintPluginSvelte from "eslint-plugin-svelte";
import globals from "globals";
import svelte from "svelte-eslint-parser";
import tseslint from "typescript-eslint";

export default tseslint.config(
  { languageOptions: { globals: globals.browser } },
  eslint.configs.recommended,
  ...tseslint.configs.recommendedTypeChecked,
  ...eslintPluginSvelte.configs["flat/recommended"],
  ...eslintPluginSvelte.configs["flat/prettier"],
  eslintConfigPrettier,
  {
    languageOptions: {
      parserOptions: {
        project: true,
        tsconfigRootDir: import.meta.dirname,
        extraFileExtensions: [".svelte"],
        sourceType: "module",
      },
    },
  },
  {
    files: ["tailwind.config.js", "eslint.config.js"],
    languageOptions: {
      globals: globals.node,
      parserOptions: {
        sourceType: "script",
      },
    },
  },
  {
    files: ["**/*.svelte"],
    languageOptions: {
      parser: svelte,
      parserOptions: {
        parser: tseslint.parser,
      },
    },
  },
  {
    ignores: ["wasm/", "pkg/", "dist/"],
  },
  {
    files: ["**/*.js"],
    extends: [tseslint.configs.disableTypeChecked],
  },
);
