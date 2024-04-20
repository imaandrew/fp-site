import __wbg_init, { wiiu_inject } from "fp-web-patcher";

export type WiiUSettings = {
  input_archive: Uint8Array;
  xdelta_patch: Uint8Array;
  enable_dark_filter: boolean;
  enable_widescreen: boolean;
  name: string;
  config: Uint8Array;
  return_zip: boolean;
  frame_layout: Uint8Array;
};

self.onmessage = (e: MessageEvent<WiiUSettings>) => {
  __wbg_init()
    .then(() => {
      postMessage(wiiu_inject(e.data));
    })
    .catch(() => {});
};

self.onerror = (e) => {
  console.error("Wii U worker error:", e);
};
