<script lang="ts">
  import { Label } from "flowbite-svelte";
  import { onMount, setContext } from "svelte";

  import { PatcherState } from "$lib/patcherState.svelte";
  import { n64Patcher } from "$lib/platforms/n64";
  import type { PlatformPatcher } from "$lib/platforms/types";
  import { createWiiPatcher } from "$lib/platforms/wii";
  import { createWiiUPatcher } from "$lib/platforms/wiiu";
  import {
    getLatestTag,
    getS3File,
    readFileAsUint8Array,
    saveUint8ArrayToFile,
    swapBytes,
  } from "$lib/util";
  import { setupWorkerHandler } from "$lib/worker";

  import BuildControls from "./components/BuildControls.svelte";
  import FileUploader from "./components/FileUploader.svelte";
  import Footer from "./components/Footer.svelte";
  import InfoModal from "./components/InfoModal.svelte";
  import PlatformSettings from "./components/PlatformSettings.svelte";

  const patcher = new PatcherState();
  setContext("patcher", patcher);

  onMount(async () => {
    try {
      patcher.tag = await getLatestTag();
    } catch (error) {
      patcher.showError(error instanceof Error ? error.message : String(error));
    }
  });

  function savePatchedFile(event: Uint8Array) {
    saveUint8ArrayToFile(event, patcher.outFileName);
    patcher.reset();
  }

  async function buildFp() {
    patcher.buttonText = "Building...";
    patcher.showLoading = true;
    patcher.disableButton = true;
    patcher.isAlertVisible = false;
    patcher.reloadAlert ^= 1;

    try {
      const patchFile = await getS3File(
        `fp/${patcher.tag}/${patcher.ver}.xdelta`,
      );
      const input = await readFileAsUint8Array(patcher.inputFile!);
      swapBytes(input);

      let platformPatcher: PlatformPatcher;
      switch (patcher.platform) {
        case "n64":
          platformPatcher = n64Patcher;
          break;
        case "wii":
          platformPatcher = createWiiPatcher(
            patcher.ver,
            patcher.channelId,
            patcher.title,
          );
          break;
        case "wiiu":
          platformPatcher = createWiiUPatcher(
            patcher.ver,
            patcher.tag,
            patcher.enableWidescreen,
            patcher.enableDarkFilter,
            patcher.returnZip,
          );
          break;
        default:
          throw new Error(`Unknown platform: ${patcher.platform}`);
      }

      const worker = await platformPatcher.patch(input, patchFile);
      setupWorkerHandler(worker, savePatchedFile, (msg) =>
        patcher.showError(msg),
      );
    } catch (error) {
      patcher.showError(error instanceof Error ? error.message : String(error));
    }
  }
</script>

<main
  class="flex h-screen flex-col items-center justify-center overflow-hidden"
>
  <div
    class="container flex max-w-md flex-col items-center justify-center rounded border border-sky-500 p-8 dark:text-white"
  >
    <h1 class="text-4xl font-bold">fp web patcher</h1>
    <Label class="pb-8 {patcher.tag != null ? '' : 'invisible'}"
      >current fp version: {patcher.tag}</Label
    >

    <FileUploader />

    <InfoModal />

    <PlatformSettings />

    <BuildControls buildFn={buildFp} />
  </div>

  <Footer />
</main>
