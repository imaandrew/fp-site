import { getS3File } from "$lib/util";
import { createWorker } from "$lib/worker";
import type { PlatformPatcher } from "./types";

export function createWiiPatcher(
  ver: string,
  channelId: string,
  title: string,
): PlatformPatcher {
  return {
    async patch(input, patchFile) {
      const [memPatch, hbPatch, hbBin] = await Promise.all([
        getS3File("gzi/mem_patch.gzi"),
        getS3File(`gzi/hb_${ver}.gzi`),
        getS3File(`hb/${ver}.bin`),
      ]);

      const concatPatch = new Uint8Array(memPatch.length + hbPatch.length);
      concatPatch.set(memPatch, 0);
      concatPatch[memPatch.length - 1] = 10;
      concatPatch.set(hbPatch, memPatch.length);

      const worker = createWorker("./worker_wii.ts");
      worker.postMessage({
        wad: input,
        xdelta_patch: patchFile,
        gzi_patch: concatPatch,
        channel_id: channelId,
        title: title,
        dol_patch: {
          dol_num: 1,
          load_addr: 0x90000800,
          data: hbBin,
        },
      });

      return worker;
    },
  };
}
