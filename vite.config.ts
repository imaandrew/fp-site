import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";
import topLevelAwait from "vite-plugin-top-level-await";
import wasmPack from "vite-plugin-wasm-pack";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [svelte(), wasmPack([], ["fp-web-patcher"]), topLevelAwait()],
});
