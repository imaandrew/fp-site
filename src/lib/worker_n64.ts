import __wbg_init, { n64_decode } from "fp-web-patcher";

self.onmessage = (e: MessageEvent) => {
  const rom = e.data.input;
  const patch = e.data.patch;
  __wbg_init().then(() => {
    postMessage(n64_decode(rom, patch));
  });
};

self.onerror = (e) => {
  console.error("N64 worker error:", e);
};

export {};
