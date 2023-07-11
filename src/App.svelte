<script lang="ts">
  import { onMount } from "svelte";
  import { getTags, getCrc, getS3File, readFileAsUint8Array } from "./lib/util";
  import { n64_decode, wii_inject, wiiu_inject } from "fp-web-patcher";
  import { slide } from "svelte/transition";
  import {
    Fileupload,
    Label,
    Helper,
    Select,
    Radio,
    Input,
    GradientButton,
    Checkbox,
    Modal,
  } from "flowbite-svelte";
  let inputFile: File;
  let ver: string;
  let tag: string;
  let channelId: string;
  let title: string;
  let romHashMessage = "Choose base file";
  let outFileName: string;
  let requiredPlatform: string;
  let selectedPlatform: string;
  let tagList = null;
  let disableButton = true;
  let returnZip: boolean;
  let enableDarkFilter: boolean;
  let enableWidescreen: boolean;
  let clickOutsideModal = false;

  const handleFileSelect = async (event: Event) => {
    const target = event.target as HTMLInputElement;
    const f = target.files[0];
    if (f) {
      switch (target.id) {
        case "fileInput":
          inputFile = f;
          await assignFileHash(f);
          handleVersionChange();
          break;
      }
    }
  };

  const getTagList = async () => {
    getTags().then(async function (tags: string[]) {
      tagList = tags.map((x) => ({ name: x, value: x }));
    });
  };

  const assignFileHash = async (file: File) => {
    try {
      const crc = await getCrc(file);
      switch (crc) {
        case 0xa7f5cd7e:
          ver = "us";
          romHashMessage = "Valid US ROM";
          requiredPlatform = "n64";
          selectedPlatform = "n64";
          break;
        case 0xbd60ca66:
          ver = "jp";
          romHashMessage = "Valid JP ROM";
          requiredPlatform = "n64";
          selectedPlatform = "n64";
          break;
        case 0xd469a9e1:
          ver = "us";
          romHashMessage = "Valid US WAD";
          requiredPlatform = "wii";
          selectedPlatform = "wii";
          channelId = "FPUS";
          title = "fp-US";
          break;
        case 0x6566e39a:
          ver = "jp";
          romHashMessage = "Valid JP WAD";
          requiredPlatform = "wii";
          selectedPlatform = "wii";
          channelId = "FPJP";
          title = "fp-JP";
          break;
        default:
          if (file.name.split(".").pop() === "zip") {
            romHashMessage = "Wii U Archive";
            requiredPlatform = "wiiu";
            selectedPlatform = "wiiu";
            returnZip = true;
          } else if (file.name.split(".").pop() === "tar") {
            romHashMessage = "Wii U Archive";
            requiredPlatform = "wiiu";
            selectedPlatform = "wiiu";
            returnZip = false;
          } else {
            ver = "unk";
            romHashMessage = "Unknown base file";
            console.log(+crc.toString(16));
          }
          break;
      }
    } catch (error) {
      console.error(error);
    }
  };

  function handleVersionChange() {
    if (tag === "" || ver === "" || outFileName === "") {
      return;
    }
    switch (selectedPlatform) {
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

  onMount(() => {
    selectedPlatform = "n64";
    getTagList();
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
    if (ver === "unk" || ver == null || inputFile === null) {
      disableButton = true;
    } else if (tag === null || tag === "") {
      disableButton = true;
    } else if (outFileName === null || outFileName === "") {
      disableButton = true;
    } else if (
      selectedPlatform === "wii" &&
      (title === null || title === "" || channelId === null || channelId === "")
    ) {
      disableButton = true;
    } else {
      disableButton = false;
    }
  }

  const buildFp = () => {
    const outFile = getS3File(`fp/${tag}/${ver}.xdelta`).then(async function (
      patchFile: Uint8Array
    ) {
      const input = await readFileAsUint8Array(inputFile);
      if (selectedPlatform === "n64") {
        return n64_decode(input, patchFile);
      } else if (selectedPlatform === "wii") {
        const memPatch = await getS3File(`gzi/mem_patch.gzi`);
        const settings = {
          wad: input,
          xdelta_patch: patchFile,
          gzi_patch: memPatch,
          channel_id: channelId,
          title: title,
        };
        return wii_inject(settings);
      } else if (selectedPlatform === "wiiu") {
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
        return wiiu_inject(settings);
      }
    });

    outFile.then(function (file: Uint8Array) {
      saveUint8ArrayToFile(file, outFileName);
    });
  };
</script>

<main class="min-h-screen overflow-hidden flex items-center justify-center">
  <div
    class="container flex flex-col items-center justify-center border border-sky-500 rounded max-w-md p-8 dark:text-white"
  >
    <h1 class="text-4xl pb-8 font-bold">fp patcher</h1>
    <div style="flex items-center">
      <Label for="fileInput" class="pb-2"
        >{romHashMessage}
        <button id="b3" on:click={() => (clickOutsideModal = true)}>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="currentColor"
            class="w-4 h-4 ml-1"
            ><path
              fill-rule="evenodd"
              d="M2.25 12c0-5.385 4.365-9.75 9.75-9.75s9.75 4.365 9.75 9.75-4.365 9.75-9.75 9.75S2.25 17.385 2.25 12zm11.378-3.917c-.89-.777-2.366-.777-3.255 0a.75.75 0 01-.988-1.129c1.454-1.272 3.776-1.272 5.23 0 1.513 1.324 1.513 3.518 0 4.842a3.75 3.75 0 01-.837.552c-.676.328-1.028.774-1.028 1.152v.75a.75.75 0 01-1.5 0v-.75c0-1.279 1.06-2.107 1.875-2.502.182-.088.351-.199.503-.331.83-.727.83-1.857 0-2.584zM12 18a.75.75 0 100-1.5.75.75 0 000 1.5z"
              clip-rule="evenodd"
            /></svg
          >
          <span class="sr-only">Show file format information</span></button
        ></Label
      >
      <Fileupload
        id="fileInput"
        class="mb-2"
        on:change={handleFileSelect}
        on:change={blockBuild}
      />
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
      extension. Try swapping it<a
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

    <div class="flex items-center pb-5">
      <p class="text-left mr-3">Version:</p>
      <Select
        class="mt-2"
        items={tagList !== null ? tagList : []}
        bind:value={tag}
        on:change={handleVersionChange}
        on:change={blockBuild}
      />
    </div>
    <div class="platform-settings grid gap-1 mb-6 md:grid-cols-[100px_auto]">
      <ul
        class="flex flex-col justify-center gap-4"
        on:change={handleVersionChange}
        on:change={blockBuild}
      >
        <li>
          <Radio
            name="platform"
            value="n64"
            disabled={requiredPlatform !== "n64"}
            bind:group={selectedPlatform}>N64</Radio
          >
        </li>
        <li>
          <Radio
            name="platform"
            value="wii"
            disabled={requiredPlatform !== "wii"}
            bind:group={selectedPlatform}>Wii</Radio
          >
        </li>
        <li>
          <Radio
            name="platform"
            value="wiiu"
            disabled={requiredPlatform !== "wiiu"}
            bind:group={selectedPlatform}>Wii U</Radio
          >
        </li>
        <!--
        <li>
          <Radio
            name="platform"
            value="switch"
            disabled={requiredPlatform !== null &&
              requiredPlatform !== "switch"}
            bind:group={selectedPlatform}>Switch</Radio
          >
        </li>
      -->
      </ul>
      <div>
        <div class="flex items-center pb-5 whitespace-nowrap">
          <p class="text-left mr-3">Output file:</p>
          <Input
            type="text"
            id="outfile"
            bind:value={outFileName}
            on:change={blockBuild}
            required
          />
        </div>
        {#if selectedPlatform === "wii"}
          <div transition:slide>
            <div class="flex items-center pb-5 whitespace-nowrap">
              <p class="text-left mr-3">Channel title:</p>
              <Input
                type="text"
                id="channel-title"
                bind:value={title}
                on:change={blockBuild}
                required
              />
            </div>
            <div class="flex items-center pb-5 whitespace-nowrap">
              <p class="text-left mr-3">Channel id:</p>
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
        {#if selectedPlatform === "wiiu"}
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
    </div>
    <GradientButton
      type="submit"
      color="greenToBlue"
      size="xl"
      class="w-1/3"
      disabled={disableButton}
      on:click={() => buildFp()}>Build</GradientButton
    >
  </div>
</main>
