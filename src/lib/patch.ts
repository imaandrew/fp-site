import type { PatcherSettings } from "$wasm/fp_web_patcher";

interface WorkerFailed {
  ok: false;
  error: string;
}

interface WorkerSucceeded {
  ok: true;
  data: Uint8Array;
}

type WorkerResponse = WorkerFailed | WorkerSucceeded;

export async function runPatch(settings: PatcherSettings): Promise<Uint8Array> {
  const worker = new Worker(new URL("./worker.ts", import.meta.url), {
    type: "module",
  });
  try {
    return await new Promise<Uint8Array>((resolve, reject) => {
      worker.onmessage = ({ data }: MessageEvent<WorkerResponse>) => {
        if (data.ok) {
          resolve(data.data);
        } else {
          reject(new Error(data.error));
        }
      };
      worker.onerror = (e) => {
        reject(new Error(e.message || "worker failed to load"));
      };
      worker.onmessageerror = () => {
        reject(new Error("could not read worker response"));
      };
      worker.postMessage(settings);
    });
  } finally {
    worker.terminate();
  }
}
