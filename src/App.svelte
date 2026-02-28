<script lang="ts">
  import { onMount, setContext } from "svelte";
  import { getLatestTag, getS3File, readFileAsUint8Array } from "./lib/util";
  import { Label } from "flowbite-svelte";

  import { n64Patcher } from "./lib/platforms/n64";
  import { createWiiPatcher } from "./lib/platforms/wii";
  import { createWiiUPatcher } from "./lib/platforms/wiiu";
  import { setupWorkerHandler } from "./lib/worker";

  import { PatcherState } from "./lib/patcherState.svelte";
  import FileUploader from "./components/FileUploader.svelte";
  import PlatformSettings from "./components/PlatformSettings.svelte";
  import BuildControls from "./components/BuildControls.svelte";
  import InfoModal from "./components/InfoModal.svelte";
  import Footer from "./components/Footer.svelte";
  import type { PlatformPatcher } from "./lib/platforms/types";
  import {
    ROM_HEADER_BYTESWAPPED,
    ROM_HEADER_LITTLEENDIAN,
  } from "./lib/constants";
  const patcher = new PatcherState();
  setContext("patcher", patcher);

  onMount(async () => {
    patcher.tag = await getLatestTag();
  });

  function showError(message: string) {
    patcher.alertText = message;
    patcher.isVisible = true;
    patcher.reset();
  }

  function swapBytes(input: Uint8Array) {
    const dataView = new DataView(input.buffer);
    const head = dataView.getUint32(0);
    if (head == ROM_HEADER_BYTESWAPPED) {
      for (let i = 0; i < dataView.byteLength; i += 2) {
        dataView.setUint16(i, dataView.getUint16(i), true);
      }
    } else if (head == ROM_HEADER_LITTLEENDIAN) {
      for (let i = 0; i < dataView.byteLength; i += 4) {
        dataView.setUint32(i, dataView.getUint32(i), true);
      }
    }
  }

  // https://stackoverflow.com/a/62176999
  function saveUint8ArrayToFile(uint8Array: Uint8Array, fileName: string) {
    const blob = new Blob([uint8Array], { type: "application/octet-stream" });
    const url = URL.createObjectURL(blob);

    const a = document.createElement("a");
    a.href = url;
    a.download = fileName;

    document.body.appendChild(a);
    a.click();

    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  function savePatchedFile(event: Uint8Array) {
    saveUint8ArrayToFile(event, patcher.outFileName);
    patcher.reset();
  }

  async function buildFp() {
    patcher.buttonText = "Building...";
    patcher.showLoading = true;
    patcher.disableButton = true;
    patcher.isVisible = false;
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
      setupWorkerHandler(worker, savePatchedFile, showError);
    } catch (error) {
      showError(error instanceof Error ? error.message : String(error));
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
