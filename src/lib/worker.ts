import __wbg_init, {
  patch,
  type PatcherSettings,
} from "../../wasm/pkg/fp_web_patcher";

self.onmessage = (e: MessageEvent<PatcherSettings>) => {
  __wbg_init()
    .then(() => {
      postMessage({ ok: true, data: patch(e.data) });
    })
    .catch((e: unknown) => {
      console.error(e);
      postMessage({
        ok: false,
        error: e instanceof Error ? e.message : String(e),
      });
    });
};

self.onerror = (e) => {
  console.error("Worker error: ", e);
};
