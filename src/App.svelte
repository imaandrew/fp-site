<script lang="ts">
  import { onMount } from "svelte";
  import {
    getLatestTag,
    getCrc,
    getS3File,
    readFileAsUint8Array,
  } from "./lib/util";
  import { slide } from "svelte/transition";
  import { writable } from "svelte/store";
  import {
    Fileupload,
    Label,
    Helper,
    Radio,
    Input,
    GradientButton,
    Checkbox,
    Modal,
    Spinner,
    Footer,
    FooterLink,
    FooterLinkGroup,
  } from "flowbite-svelte";
  let inputFile: File;
  let ver: string;
  let tag: string;
  let channelId: string;
  let title: string;
  let romHashMessage = "Choose base file";
  let outFileName: string;
  let platform: string;
  let disableButton = true;
  let returnZip: boolean;
  let enableDarkFilter: boolean;
  let enableWidescreen: boolean;
  let clickOutsideModal = false;
  let showLoading = false;
  let buttonText = writable("Build");

  async function handleFileSelect(event: Event) {
    const target = event.target as HTMLInputElement;
    if (!target.files) {
      return;
    }
    const f = target.files[0];
    if (f) {
      switch (target.id) {
        case "fileInput":
          inputFile = f;
          await assignFileHash(f);
          handleVersionChange();
          blockBuild();
          break;
      }
    }
  }

  async function assignFileHash(file: File) {
    try {
      const crc = await getCrc(file);
      switch (crc) {
        case 0xa7f5cd7e:
          ver = "us";
          romHashMessage = "Valid US ROM";
          platform = "n64";
          break;
        case 0xbd60ca66:
          ver = "jp";
          romHashMessage = "Valid JP ROM";
          platform = "n64";
          break;
        case 0xd469a9e1:
          ver = "us";
          romHashMessage = "Valid US WAD";
          platform = "wii";
          channelId = "FPUS";
          title = "fp-US";
          break;
        case 0x6566e39a:
          ver = "jp";
          romHashMessage = "Valid JP WAD";
          platform = "wii";
          channelId = "FPJP";
          title = "fp-JP";
          break;
        default:
          if (file.name.split(".").pop() === "zip") {
            romHashMessage = "Wii U Archive";
            platform = "wiiu";
            returnZip = true;
          } else if (file.name.split(".").pop() === "tar") {
            romHashMessage = "Wii U Archive";
            platform = "wiiu";
            returnZip = false;
          } else {
            ver = "unk";
            romHashMessage = "Unknown base file";
          }
          break;
      }
    } catch (error) {
      console.error(error);
    }
  }

  function handleVersionChange() {
    if (
      tag === "" ||
      ver === "" ||
      outFileName === "" ||
      tag == null ||
      ver === "unk"
    ) {
      return;
    }
    switch (platform) {
      case "n64":
        outFileName = `${tag}-${ver}.z64`;
        break;
      case "wii":
        outFileName = `${tag}-${ver}.wad`;
        break;
      case "wiiu":
        if (returnZip) {
          outFileName = `${tag}-${ver}.zip`;
        } else {
          outFileName = `${tag}-${ver}.tar`;
        }
    }
  }

  onMount(async () => {
    platform = "n64";
    tag = await getLatestTag();
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

  function blockBuild() {
    if (ver === "unk" || ver == null || inputFile == null) {
      disableButton = true;
    } else if (tag == null || tag === "") {
      disableButton = true;
    } else if (outFileName == null || outFileName === "") {
      disableButton = true;
    } else if (
      platform === "wii" &&
      (title == null || title === "" || channelId == null || channelId === "")
    ) {
      disableButton = true;
    } else {
      disableButton = false;
    }
  }

  function savePatchedFile(event: MessageEvent<Uint8Array>) {
    saveUint8ArrayToFile(event.data, outFileName);
    buttonText.set("Build");
    showLoading = false;
    disableButton = false;
  }

  function buildFp() {
    buttonText.set("Building...");
    showLoading = true;
    disableButton = true;
    getS3File(`fp/${tag}/${ver}.xdelta`)
      .then(async (patchFile: Uint8Array) => {
        const input = await readFileAsUint8Array(inputFile);
        if (platform === "n64") {
          const worker = import.meta.env.DEV
            ? new Worker(new URL("./lib/worker_n64.ts", import.meta.url), {
                type: "module",
              })
            : new Worker(new URL("./lib/worker_n64.ts", import.meta.url), {
                type: "classic",
              });

          worker.onmessage = (event: MessageEvent<Uint8Array>) => {
            savePatchedFile(event);
            worker.terminate();
          };

          worker.postMessage({
            input: input,
            patch: patchFile,
          });
        } else if (platform === "wii") {
          const memPatch = await getS3File("gzi/mem_patch.gzi");
          const hbPatch = await getS3File(`gzi/hb_${ver}.gzi`);
          const hbBin = await getS3File(`hb/${ver}.bin`);
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
            channel_id: channelId,
            title: title,
            dol_patch: dolPatch,
          };

          const worker = import.meta.env.DEV
            ? new Worker(new URL("./lib/worker_wii.ts", import.meta.url), {
                type: "module",
              })
            : new Worker(new URL("./lib/worker_wii.ts", import.meta.url), {
                type: "classic",
              });

          worker.onmessage = (event: MessageEvent<Uint8Array>) => {
            savePatchedFile(event);
            worker.terminate();
          };

          worker.postMessage(settings);
        } else if (platform === "wiiu") {
          const config = await getS3File(`wiiu/${ver}.ini`);
          let frameLayout: Uint8Array;
          if (enableWidescreen && enableDarkFilter) {
            frameLayout = await getS3File(`wiiu/FrameLayout_dark_wide.arc`);
          } else if (enableDarkFilter) {
            frameLayout = await getS3File(`wiiu/FrameLayout_dark.arc`);
          } else if (enableWidescreen) {
            frameLayout = await getS3File(`wiiu/FrameLayout_wide.arc`);
          } else {
            frameLayout = await getS3File(`wiiu/FrameLayout.arc`);
          }
          const settings = {
            input_archive: input,
            xdelta_patch: patchFile,
            enable_dark_filter: enableDarkFilter,
            enable_widescreen: enableWidescreen,
            name: `fp-${tag}-${ver}`,
            config: config,
            return_zip: returnZip,
            frame_layout: frameLayout,
          };

          const worker = import.meta.env.DEV
            ? new Worker(new URL("./lib/worker_wiiu.ts", import.meta.url), {
                type: "module",
              })
            : new Worker(new URL("./lib/worker_wiiu.ts", import.meta.url), {
                type: "classic",
              });

          worker.onmessage = (event: MessageEvent<Uint8Array>) => {
            savePatchedFile(event);
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
    <Label class="pb-8 {tag != null ? '' : 'invisible'}"
      >current fp version: {tag}</Label
    >
    <div class="w-5/6">
      <Label for="fileInput" class="pb-2"
        >{romHashMessage}
        <button id="b3" on:click={() => (clickOutsideModal = true)}>
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
      <Fileupload id="fileInput" class="mb-2" on:change={handleFileSelect} />
      <Helper>Z64, WAD, ZIP, or TAR.</Helper>
    </div>

    <Modal
      title="Input File Formats"
      bind:open={clickOutsideModal}
      autoclose
      outsideclose
    >
      <h3 class="font-semibold text-gray-900 dark:text-white">N64</h3>
      ROM must be in the Z64 format (big endian). If the file ends in .z64 but the
      patcher doesn't recognize it, it might be in a different format with the wrong
      extension. Try swapping it
      <!-- prettier-ignore -->

      <a
        href="https://hack64.net/tools/swapper.php"
        target="_blank"
        class="text-primary-600 underline dark:text-primary-500 hover:no-underline"
        >here</a
      >
      <h3 class="font-semibold text-gray-900 dark:text-white">Wii</h3>
      Must provide a WAD file. An N64 ROM doesn't need to be provided, the patcher
      will use the one in the WAD
      <h3 class="font-semibold text-gray-900 dark:text-white">Wii U</h3>
      Must provide the decrypted game data packed in either a ZIP or TAR file. The
      three folders, (code, content, meta) do not need to be in the root of the archive,
      but they must all be in the same folder
    </Modal>

    <div class="platform-settings mb-6 grid gap-1">
      <div class="flex items-center whitespace-nowrap py-5">
        <p class="mr-3 text-left">Output file:</p>
        <Input
          type="text"
          id="outfile"
          bind:value={outFileName}
          on:change={blockBuild}
          required
        />
      </div>
      {#if platform === "wii"}
        <div transition:slide>
          <div class="flex items-center whitespace-nowrap pb-5">
            <p class="mr-3 text-left">Channel title:</p>
            <Input
              type="text"
              id="channel-title"
              bind:value={title}
              on:change={blockBuild}
              required
            />
          </div>
          <div class="flex items-center whitespace-nowrap pb-5">
            <p class="mr-3 text-left">Channel id:</p>
            <Input
              type="text"
              id="channel-id"
              bind:value={channelId}
              on:change={blockBuild}
              required
            />
          </div>
        </div>
      {/if}
      {#if platform === "wiiu"}
        <div transition:slide class="space-y-4">
          <div class="flex gap-5">
            <Radio
              name="version"
              value="us"
              bind:group={ver}
              on:change={handleVersionChange}>US</Radio
            >
            <Radio
              name="version"
              value="jp"
              bind:group={ver}
              on:change={handleVersionChange}>JP</Radio
            >
          </div>
          <Checkbox bind:checked={enableDarkFilter}>Dark filter</Checkbox>
          <Checkbox bind:checked={enableWidescreen}>Widescreen</Checkbox>
        </div>
      {/if}
    </div>
    <div class="grid auto-cols-max grid-cols-5 items-center gap-4">
      <div />
      <GradientButton
        type="submit"
        color="greenToBlue"
        size="xl"
        class="col-span-3"
        disabled={disableButton}
        on:click={() => buildFp()}
      >
        {$buttonText}
      </GradientButton>
      <Spinner
        class="mr-3 {showLoading ? '' : 'invisible'}"
        size="4"
        color="green"
      />
    </div>
  </div>
  <Footer>
    <FooterLinkGroup
      ulClass="flex flex-wrap items-center mt-3 text-sm text-gray-500 dark:text-gray-400 sm:mt-0"
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
  </Footer>
</main>
