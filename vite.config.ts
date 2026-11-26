import path from "path";

import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [tailwindcss(), svelte()],
  resolve: {
    alias: {
      $lib: path.resolve(import.meta.dirname, "src/lib"),
      $wasm: path.resolve(import.meta.dirname, "wasm/pkg"),
    },
  },
});
