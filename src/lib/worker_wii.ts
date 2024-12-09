import __wbg_init, { wii_inject } from "fp-web-patcher";

export type DolPatch = {
  dol_num: number;
  load_addr: number;
  data: Uint8Array;
};

export type WiiSettings = {
  wad: Uint8Array;
  xdelta_patch: Uint8Array;
  gzi_patch: Uint8Array;
  channel_id: string;
  title: string;
  dol_patch: DolPatch;
};

self.onmessage = (e: MessageEvent<WiiSettings>) => {
  __wbg_init()
    .then(() => {
      postMessage(wii_inject(e.data));
    })
    .catch((e) => {
      console.error(e);
      postMessage(e);
    });
};

self.onerror = (e) => {
  console.error("Wii worker error:", e);
};
