import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import path from "path";
import { defineConfig } from "vite";
import wasmPack from "vite-plugin-wasm-pack";

export default defineConfig({
  resolve: {
    alias: {
      $lib: path.resolve(import.meta.dirname, "./src/lib"),
    },
  },
  plugins: [tailwindcss(), svelte(), wasmPack([], ["fp-web-patcher"])],
});
