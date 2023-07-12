import __wbg_init, { wiiu_inject } from "fp-web-patcher";

self.onmessage = (e: MessageEvent) => {
  const s = e.data;
  __wbg_init().then(() => {
    postMessage(wiiu_inject(s));
  });
};

self.onerror = (e) => {
  console.error("Wii U worker error:", e);
};

export {};
