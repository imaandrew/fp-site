<script lang="ts">
  import { onMount, setContext } from "svelte";
  import {
    getLatestTag,
    getCrc,
    getS3File,
    readFileAsUint8Array,
  } from "./lib/util";
  import { slide } from "svelte/transition";
  import { CloseCircleSolid } from "flowbite-svelte-icons";
  import {
    Alert,
    Fileupload,
    Label,
    Helper,
    Radio,
    Input,
    GradientButton,
    Checkbox,
    Modal,
    Spinner,
    FooterLink,
    FooterLinkGroup,
  } from "flowbite-svelte";

  import { PatcherState } from "./lib/patcherState.svelte";
  const patcher = new PatcherState();
  setContext("patcher", patcher);

  async function handleFileSelect(event: Event) {
    const target = event.target as HTMLInputElement;
    if (!target.files) {
      return;
    }
    const f = target.files[0];
    if (f) {
      switch (target.id) {
        case "fileInput":
          patcher.inputFile = f;
          await assignFileHash(f);
          handleVersionChange();
          break;
      }
    }
  }

  async function assignFileHash(file: File) {
    try {
      const crc = await getCrc(file);
      switch (crc) {
        case 0xa7f5cd7e:
        case 0xd56d1c89:
        case 0xa4c948bf:
          patcher.ver = "us";
          patcher.romHashMessage = "Valid US ROM";
          patcher.platform = "n64";
          break;
        case 0xbd60ca66:
        case 0x1fb7e59a:
        case 0xca72bffc:
          patcher.ver = "jp";
          patcher.romHashMessage = "Valid JP ROM";
          patcher.platform = "n64";
          break;
        case 0xd469a9e1:
          patcher.ver = "us";
          patcher.romHashMessage = "Valid US WAD";
          patcher.platform = "wii";
          patcher.channelId = "FPUS";
          patcher.title = "fp-US";
          break;
        case 0x6566e39a:
          patcher.ver = "jp";
          patcher.romHashMessage = "Valid JP WAD";
          patcher.platform = "wii";
          patcher.channelId = "FPJP";
          patcher.title = "fp-JP";
          break;
        default:
          if (file.name.split(".").pop() === "zip") {
            patcher.romHashMessage = "Wii U Archive";
            patcher.platform = "wiiu";
            patcher.returnZip = true;
          } else if (file.name.split(".").pop() === "tar") {
            patcher.romHashMessage = "Wii U Archive";
            patcher.platform = "wiiu";
            patcher.returnZip = false;
          } else {
            patcher.ver = "unk";
            patcher.romHashMessage = "Unknown base file";
          }
          break;
      }
    } catch (error) {
      console.error(error);
    }
  }

  function handleVersionChange() {
    if (
      patcher.tag === "" ||
      patcher.ver === "" ||
      patcher.tag == null ||
      patcher.ver === "unk"
    ) {
      return;
    }
    switch (patcher.platform) {
      case "n64":
        patcher.outFileName = `${patcher.tag}-${patcher.ver}.z64`;
        break;
      case "wii":
        patcher.outFileName = `${patcher.tag}-${patcher.ver}.wad`;
        break;
      case "wiiu":
        if (patcher.returnZip) {
          patcher.outFileName = `${patcher.tag}-${patcher.ver}.zip`;
        } else {
          patcher.outFileName = `${patcher.tag}-${patcher.ver}.tar`;
        }
    }
  }

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
    <div class="w-5/6">
      <Label for="fileInput" class="pb-2"
        >{patcher.romHashMessage}
        <button id="b3" onclick={() => (patcher.clickOutsideModal = true)}>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="currentColor"
            class="ml-1 h-4 w-4"
            ><path
              fill-rule="evenodd"
              d="M2.25 12c0-5.385 4.365-9.75 9.75-9.75s9.75 4.365 9.75 9.75-4.365 9.75-9.75 9.75S2.25 17.385 2.25 12zm11.378-3.917c-.89-.777-2.366-.777-3.255 0a.75.75 0 01-.988-1.129c1.454-1.272 3.776-1.272 5.23 0 1.513 1.324 1.513 3.518 0 4.842a3.75 3.75 0 01-.837.552c-.676.328-1.028.774-1.028 1.152v.75a.75.75 0 01-1.5 0v-.75c0-1.279 1.06-2.107 1.875-2.502.182-.088.351-.199.503-.331.83-.727.83-1.857 0-2.584zM12 18a.75.75 0 100-1.5.75.75 0 000 1.5z"
              clip-rule="evenodd"
            /></svg
          >
          <span class="sr-only">Show file format information</span></button
        ></Label
      >
      <Fileupload id="fileInput" class="mb-2" onchange={handleFileSelect} />
      <Helper>Z64, WAD, ZIP, or TAR.</Helper>
    </div>

    <Modal
      title="Input File Formats"
      bind:open={patcher.clickOutsideModal}
      autoclose
      outsideclose
    >
      <h3 class="font-semibold text-gray-900 dark:text-white">N64</h3>
      ROMs in any format (big-endian, little-endian, byte-swapped) are supported
      <h3 class="font-semibold text-gray-900 dark:text-white">Wii</h3>
      Must provide a WAD file. An N64 ROM doesn't need to be provided, the patcher
      will use the one in the WAD
      <h3 class="font-semibold text-gray-900 dark:text-white">Wii U</h3>
      Must provide the decrypted game data packed in either a ZIP or TAR file. The
      three folders, (code, content, meta) do not need to be in the root of the archive,
      but they must all be in the same folder
    </Modal>

    <div class="platform-settings mb-6 grid gap-1">
      <div class="flex items-center py-5 whitespace-nowrap">
        <p class="mr-3 text-left">Output file:</p>
        <Input
          type="text"
          id="outfile"
          bind:value={patcher.outFileName}
          required
        />
      </div>
      {#if patcher.platform === "wii"}
        <div transition:slide>
          <div class="flex items-center pb-5 whitespace-nowrap">
            <p class="mr-3 text-left">Channel title:</p>
            <Input
              type="text"
              id="channel-title"
              bind:value={patcher.title}
              required
            />
          </div>
          <div class="flex items-center pb-5 whitespace-nowrap">
            <p class="mr-3 text-left">Channel id:</p>
            <Input
              type="text"
              id="channel-id"
              bind:value={patcher.channelId}
              required
            />
          </div>
        </div>
      {/if}
      {#if patcher.platform === "wiiu"}
        <div transition:slide class="space-y-4">
          <div class="flex gap-5">
            <Radio
              name="version"
              value="us"
              bind:group={patcher.ver}
              onchange={handleVersionChange}>US</Radio
            >
            <Radio
              name="version"
              value="jp"
              bind:group={patcher.ver}
              onchange={handleVersionChange}>JP</Radio
            >
          </div>
          <Checkbox bind:checked={patcher.enableDarkFilter}
            >Dark filter</Checkbox
          >
          <Checkbox bind:checked={patcher.enableWidescreen}>Widescreen</Checkbox
          >
        </div>
      {/if}
    </div>
    <div class="grid auto-cols-max grid-cols-5 items-center gap-4">
      <div></div>
      <GradientButton
        type="submit"
        color="greenToBlue"
        size="xl"
        class="col-span-3"
        disabled={patcher.disableButton}
        onclick={() => buildFp()}
      >
        {patcher.buttonText}
      </GradientButton>
      <Spinner
        class="mr-3 {patcher.showLoading ? '' : 'invisible'}"
        size="4"
        color="green"
      />
    </div>
    {#key patcher.reloadAlert}
      {#if patcher.isVisible}
        <Alert dismissable transition={slide} color="red" class="mt-6">
          {#snippet icon()}
            <CloseCircleSolid class="h-5 w-5" />
          {/snippet}
          {patcher.alertText}
        </Alert>
      {/if}
    {/key}
  </div>

  <FooterLinkGroup
    class="mt-3 flex flex-wrap items-center text-sm text-gray-500 sm:mt-0 dark:text-gray-400"
  >
    <FooterLink
      href="https://fp-web-patcher-files.s3.ca-central-1.amazonaws.com/fp-practice-saves.zip"
      >Practice Saves</FooterLink
    >
    <FooterLink href="https://fp-docs.starhaven.dev/" target="_blank"
      >Manual</FooterLink
    >
    <FooterLink href="https://github.com/imaandrew/fp-site" target="_blank"
      >Website GitHub</FooterLink
    >
    <FooterLink
      href="https://github.com/imaandrew/fp-web-patcher"
      target="_blank">Patcher GitHub</FooterLink
    >
  </FooterLinkGroup>
</main>
