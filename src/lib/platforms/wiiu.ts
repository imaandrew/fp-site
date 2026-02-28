import { getS3File } from "$lib/util";
import { createWorker } from "$lib/worker";
import type { PlatformPatcher } from "./types";

export function createWiiUPatcher(
  ver: string,
  tag: string,
  enableWidescreen: boolean,
  enableDarkFilter: boolean,
  returnZip: boolean,
): PlatformPatcher {
  return {
    async patch(input, patchFile) {
      let frameLayoutPath = "wiiu/FrameLayout";
      if (enableWidescreen && enableDarkFilter) {
        frameLayoutPath += "_dark_wide";
      } else if (enableDarkFilter) {
        frameLayoutPath += "_dark";
      } else if (enableWidescreen) {
        frameLayoutPath += "_wide";
      }

      const [config, frameLayout] = await Promise.all([
        getS3File(`wiiu/${ver}.ini`),
        getS3File(`${frameLayoutPath}.arc`),
      ]);

      const worker = createWorker("./worker_wiiu.ts");
      worker.postMessage({
        input_archive: input,
        xdelta_patch: patchFile,
        enable_dark_filter: enableDarkFilter,
        enable_widescreen: enableWidescreen,
        name: `fp-${tag}-${ver}`,
        config: config,
        return_zip: returnZip,
        frame_layout: frameLayout,
      });
      return worker;
    },
  };
}
