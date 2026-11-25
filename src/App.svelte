<script lang="ts">
  import { Label } from "flowbite-svelte";
  import { onMount, setContext } from "svelte";

  import { runPatch } from "$lib/patch";
  import { builders } from "$lib/patch.settings";
  import { PatcherState } from "$lib/patcherState.svelte";
  import {
    getLatestTag,
    getS3File,
    readFileAsUint8Array,
    saveUint8ArrayToFile,
  } from "$lib/util";

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
    patcher.isAlertVisible = false;
    patcher.reloadAlert++;
    if (!patcher.inputFile) return;
    if (!patcher.region) return;
    if (!patcher.platform) return;
    patcher.building = true;
    patcher.buttonText = "Building...";
    patcher.showLoading = true;

    try {
      const patchFile = await getS3File(
        `fp/${patcher.tag}/${patcher.region}.xdelta`,
      );
      const input = await readFileAsUint8Array(patcher.inputFile);

      savePatchedFile(
        await runPatch(
          await builders[patcher.platform](patcher, input, patchFile),
        ),
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
    <Label class="pb-8">current fp version: {patcher.tag}</Label>

    <FileUploader />

    <InfoModal />

    <PlatformSettings />

    <BuildControls buildFn={buildFp} />
  </div>

  <Footer />
</main>
