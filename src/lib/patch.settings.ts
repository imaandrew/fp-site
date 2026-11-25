import type { PatcherState, Platform } from "$lib/patcherState.svelte";
import { getS3File, swapBytes } from "$lib/util";

import type {
  PatcherSettings,
} from "../../wasm/pkg/fp_web_patcher";

export interface PlatformPatcher {
  patch(input: Uint8Array, patchFile: Uint8Array): Promise<Worker>;
}
type SettingsBuilder = (
  s: PatcherState,
  input: Uint8Array,
  romPatch: Uint8Array,
) => Promise<PatcherSettings>;

const n64: SettingsBuilder = async (_s, input, romPatch) => {
  swapBytes(input);
  return new Promise((resolve) =>
    { resolve({ n64: { input, rom_patch: romPatch } }); },
  );
};

const wii: SettingsBuilder = async (s, input, romPatch) => {
  const [memPatch, hbPatch, hbBin] = await Promise.all([
    getS3File("gzi/mem_patch.gzi"),
    getS3File(`gzi/hb_${s.ver}.gzi`),
    getS3File(`hb/${s.ver}.bin`),
  ]);

  const concatPatch = new Uint8Array(memPatch.length + hbPatch.length);
  concatPatch.set(memPatch, 0);
  concatPatch[memPatch.length - 1] = 10;
  concatPatch.set(hbPatch, memPatch.length);

  return {
    wii: {
      input,
      rom_patch: romPatch,
      gzi_patch: concatPatch,
      channel_id: s.channelId,
      title: s.title,
      dol_patch: {
        dol_num: 1,
        load_addr: 0x90000800,
        data: hbBin,
      },
    },
  };
};

const wiiu: SettingsBuilder = async (s, input, romPatch) => {
  let frameLayoutPath = "wiiu/FrameLayout";
  if (s.enableDarkFilter) {
    frameLayoutPath += "_dark";
  }
  if (s.enableWidescreen) {
    frameLayoutPath += "_wide";
  }

  const [config, frameLayout] = await Promise.all([
    getS3File(`wiiu/${s.ver}.ini`),
    getS3File(`${frameLayoutPath}.arc`),
  ]);

  return {
    wiiu: {
      input,
      rom_patch: romPatch,
      enable_dark_filter: s.enableDarkFilter,
      enable_widescreen: s.enableWidescreen,
      name: `fp-${s.tag}-${s.ver}`,
      config,
      result_type: s.resultType,
      frame_layout: frameLayout,
    },
  };
};

export const builders: Record<Platform, SettingsBuilder> = { n64, wii, wiiu };
