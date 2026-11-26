<script lang="ts">
  import { Label } from "flowbite-svelte";
  import { onMount, setContext } from "svelte";

  import { runPatch } from "$lib/patch";
  import { buildSettings } from "$lib/patch.settings";
  import { PatcherState } from "$lib/patcherState.svelte";
  import { getLatestTag, getS3File, saveUint8ArrayToFile } from "$lib/util";

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
    const options = $state.snapshot(patcher.options);
    if (options?.region == null) return;
    patcher.building = true;

    try {
      const romPatch = await getS3File(
        `fp/${patcher.tag}/${options.region}.xdelta`,
      );
      const settings = await buildSettings(
        options,
        patcher.inputData,
        romPatch,
      );
      savePatchedFile(await runPatch(settings));
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
