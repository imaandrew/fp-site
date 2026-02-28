import { createWorker } from "$lib/worker";
import type { PlatformPatcher } from "./types";
export const n64Patcher: PlatformPatcher = {
  patch(input, patchFile) {
    const worker = createWorker("./worker_n64.ts");
    worker.postMessage({ input, patch: patchFile });
    return Promise.resolve(worker);
  },
};
