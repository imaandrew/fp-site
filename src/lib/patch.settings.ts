import type { PatcherSettings } from "$wasm/fp_web_patcher";

import { getS3File, swapBytes } from "$lib/util";

import type {
  N64Options,
  PlatformOptions,
  WiiOptions,
  WiiUOptions,
} from "./platform";

function n64(
  _o: N64Options,
  input: Uint8Array,
  romPatch: Uint8Array,
): Promise<PatcherSettings> {
  swapBytes(input);
  return Promise.resolve({ n64: { input, rom_patch: romPatch } });
}

async function wii(
  o: WiiOptions,
  input: Uint8Array,
  romPatch: Uint8Array,
): Promise<PatcherSettings> {
  const [memPatch, hbPatch, hbBin] = await Promise.all([
    getS3File("gzi/mem_patch.gzi"),
    getS3File(`gzi/hb_${o.region}.gzi`),
    getS3File(`hb/${o.region}.bin`),
  ]);

  const concatPatch = new Uint8Array(memPatch.length + 1 + hbPatch.length);
  concatPatch.set(memPatch, 0);
  concatPatch[memPatch.length] = "\n".charCodeAt(0);
  concatPatch.set(hbPatch, memPatch.length + 1);

  return {
    wii: {
      input,
      rom_patch: romPatch,
      gzi_patch: concatPatch,
      channel_id: o.channelId,
      title: o.title,
      dol_patch: {
        dol_num: 1,
        load_addr: 0x90000800,
        data: hbBin,
      },
    },
  };
}

async function wiiu(
  o: WiiUOptions,
  input: Uint8Array,
  romPatch: Uint8Array,
): Promise<PatcherSettings> {
  if (!o.region) {
    throw new Error("region not set");
  }
  let frameLayoutPath = "wiiu/FrameLayout";
  if (o.enableDarkFilter) {
    frameLayoutPath += "_dark";
  }
  if (o.enableWidescreen) {
    frameLayoutPath += "_wide";
  }

  const [config, frameLayout] = await Promise.all([
    getS3File(`wiiu/${o.region}.ini`),
    getS3File(`${frameLayoutPath}.arc`),
  ]);

  return {
    wiiu: {
      input,
      rom_patch: romPatch,
      enable_dark_filter: o.enableDarkFilter,
      enable_widescreen: o.enableWidescreen,
      config,
      frame_layout: frameLayout,
      region: o.region,
    },
  };
}

export function buildSettings(
  o: PlatformOptions,
  input: Uint8Array,
  romPatch: Uint8Array,
) {
  switch (o.platform) {
    case "n64":
      return n64(o, input, romPatch);
    case "wii":
      return wii(o, input, romPatch);
    case "wiiu":
      return wiiu(o, input, romPatch);
  }
}
