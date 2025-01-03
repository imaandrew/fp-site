import __wbg_init, { n64_decode } from "fp-web-patcher";

export type N64Settings = {
  input: Uint8Array;
  patch: Uint8Array;
};

self.onmessage = (e: MessageEvent<N64Settings>) => {
  const rom = e.data.input;
  const patch = e.data.patch;
  __wbg_init()
    .then(() => {
      postMessage(n64_decode(rom, patch));
    })
    .catch((e) => {
      console.error(e);
      postMessage(e);
    });
};

self.onerror = (e) => {
  console.error("N64 worker error:", e);
};
