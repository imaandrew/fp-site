import { createWorker } from "../worker";
import type { PlatformPatcher } from "./types";
export const n64Patcher: PlatformPatcher = {
  async patch(input, patchFile) {
    const worker = createWorker("./worker_n64.ts");
    worker.postMessage({ input, patch: patchFile });
    return worker;
  },
};
