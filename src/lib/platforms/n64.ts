import type { PlatformPatcher } from "./types";

export const n64Patcher: PlatformPatcher = {
  patch(input, patchFile) {
    const worker = new Worker(new URL("$lib/worker_n64.ts", import.meta.url), {
      type: "module",
    });
    worker.postMessage({ input, patch: patchFile });
    return Promise.resolve(worker);
  },
};
