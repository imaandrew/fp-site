import path from "path";

import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import topLevelAwait from "vite-plugin-top-level-await";
import wasm from "vite-plugin-wasm";

export default defineConfig({
  plugins: [tailwindcss(), svelte(), wasm(), topLevelAwait()],
  resolve: {
    alias: {
      $lib: path.resolve(import.meta.dirname, "src/lib"),
      "wasm-lib": path.resolve(import.meta.dirname, "wasm/pkg"),
    },
  },
});
