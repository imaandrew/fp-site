import __wbg_init, { wii_inject } from "fp-web-patcher";

self.onmessage = (e: MessageEvent) => {
  const s = e.data;
  __wbg_init().then(() => {
    postMessage(wii_inject(s));
  });
};

self.onerror = (e) => {
  console.error("Wii worker error:", e);
};

export {};
