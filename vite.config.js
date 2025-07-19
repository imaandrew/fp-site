import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import wasmPack from "vite-plugin-wasm-pack";

export default defineConfig({
  plugins: [tailwindcss(), svelte(), wasmPack([], ["fp-web-patcher"])],
});
