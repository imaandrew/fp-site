<script lang="ts">
  import { onMount, setContext } from "svelte";
  import { getLatestTag, getS3File, readFileAsUint8Array } from "./lib/util";
  import { Label } from "flowbite-svelte";

  import { PatcherState } from "./lib/patcherState.svelte";
  import FileUploader from "./components/FileUploader.svelte";
  import PlatformSettings from "./components/PlatformSettings.svelte";
  import BuildControls from "./components/BuildControls.svelte";
  import InfoModal from "./components/InfoModal.svelte";
  import Footer from "./components/Footer.svelte";
  const patcher = new PatcherState();
  setContext("patcher", patcher);

  onMount(async () => {
    patcher.tag = await getLatestTag();
  });

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

  function buildFp() {
    patcher.buttonText = "Building...";
    patcher.showLoading = true;
    patcher.disableButton = true;
    patcher.isVisible = false;
    patcher.reloadAlert ^= 1;
    getS3File(`fp/${patcher.tag}/${patcher.ver}.xdelta`)
      .then(async (patchFile: Uint8Array) => {
        if (!patcher.inputFile) {
          // error
          return;
        }
        const input = await readFileAsUint8Array(patcher.inputFile);
        const dataView = new DataView(input.buffer);
        const head = dataView.getUint32(0);
        if (head == 0x37804012) {
          for (let i = 0; i < dataView.byteLength; i += 2) {
            dataView.setUint16(i, dataView.getUint16(i), true);
          }
        } else if (head == 0x40123780) {
          for (let i = 0; i < dataView.byteLength; i += 4) {
            dataView.setUint32(i, dataView.getUint32(i), true);
          }
        }
        if (patcher.platform === "n64") {
          const worker = import.meta.env.DEV
            ? new Worker(new URL("./lib/worker_n64.ts", import.meta.url), {
                type: "module",
              })
            : new Worker(new URL("./lib/worker_n64.ts", import.meta.url), {
                type: "classic",
              });

          worker.onmessage = (event: MessageEvent<Uint8Array | string>) => {
            if (typeof event.data === "string") {
              patcher.alertText = event.data;
              patcher.isVisible = true;
              patcher.reset();
            } else {
              savePatchedFile(event.data);
            }
            worker.terminate();
          };

          worker.postMessage({
            input: input,
            patch: patchFile,
          });
        } else if (patcher.platform === "wii") {
          const memPatch = await getS3File("gzi/mem_patch.gzi");
          const hbPatch = await getS3File(`gzi/hb_${patcher.ver}.gzi`);
          const hbBin = await getS3File(`hb/${patcher.ver}.bin`);
          const concatPatch = new Uint8Array(memPatch.length + hbPatch.length);
          concatPatch.set(memPatch, 0);
          concatPatch[memPatch.length - 1] = 10;
          concatPatch.set(hbPatch, memPatch.length);
          const dolPatch = {
            dol_num: 1,
            load_addr: 0x90000800,
            data: hbBin,
          };

          const settings = {
            wad: input,
            xdelta_patch: patchFile,
            gzi_patch: concatPatch,
            channel_id: patcher.channelId,
            title: patcher.title,
            dol_patch: dolPatch,
          };

          const worker = import.meta.env.DEV
            ? new Worker(new URL("./lib/worker_wii.ts", import.meta.url), {
                type: "module",
              })
            : new Worker(new URL("./lib/worker_wii.ts", import.meta.url), {
                type: "classic",
              });

          worker.onmessage = (event: MessageEvent<Uint8Array | string>) => {
            if (typeof event.data === "string") {
              patcher.alertText = event.data;
              patcher.isVisible = true;
              patcher.reset();
            } else {
              savePatchedFile(event.data);
            }
            worker.terminate();
          };

          worker.postMessage(settings);
        } else if (patcher.platform === "wiiu") {
          const config = await getS3File(`wiiu/${patcher.ver}.ini`);
          let frameLayout: Uint8Array;
          if (patcher.enableWidescreen && patcher.enableDarkFilter) {
            frameLayout = await getS3File(`wiiu/FrameLayout_dark_wide.arc`);
          } else if (patcher.enableDarkFilter) {
            frameLayout = await getS3File(`wiiu/FrameLayout_dark.arc`);
          } else if (patcher.enableWidescreen) {
            frameLayout = await getS3File(`wiiu/FrameLayout_wide.arc`);
          } else {
            frameLayout = await getS3File(`wiiu/FrameLayout.arc`);
          }
          const settings = {
            input_archive: input,
            xdelta_patch: patchFile,
            enable_dark_filter: patcher.enableDarkFilter,
            enable_widescreen: patcher.enableWidescreen,
            name: `fp-${patcher.tag}-${patcher.ver}`,
            config: config,
            return_zip: patcher.returnZip,
            frame_layout: frameLayout,
          };

          const worker = import.meta.env.DEV
            ? new Worker(new URL("./lib/worker_wiiu.ts", import.meta.url), {
                type: "module",
              })
            : new Worker(new URL("./lib/worker_wiiu.ts", import.meta.url), {
                type: "classic",
              });

          worker.onmessage = (event: MessageEvent<Uint8Array | string>) => {
            if (typeof event.data === "string") {
              patcher.alertText = event.data;
              patcher.isVisible = true;
              patcher.reset();
            } else {
              savePatchedFile(event.data);
            }
            worker.terminate();
          };

          worker.postMessage(settings);
        }
      })
      .catch((error: Error) => {
        console.error(error.message);
      });
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
