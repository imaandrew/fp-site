<script lang="ts">
  import { onMount } from "svelte";
  import { getTags } from "./lib/github";
  import { getMd5 } from "./lib/md5";
  import { n64_decode, wii_inject } from "fp-web-patcher";
  import { slide } from "svelte/transition";
  import {
    Fileupload,
    Label,
    Helper,
    Select,
    Radio,
    Input,
    GradientButton,
  } from "flowbite-svelte";
  import { getS3File } from "./lib/get_s3_file";
  let inputFile: File;
  let ver: string;
  let tag: string;
  let channelId: string;
  let title: string;
  let romHashMessage = "Choose base file";
  let outFileName: string;
  let requiredPlatform: string = null;
  let selectedPlatform: string;
  let tagList = null;
  let disableButton = true;

  const handleFileSelect = async (event: Event) => {
    const target = event.target as HTMLInputElement;
    const f = target.files[0];
    if (f) {
      switch (target.id) {
        case "fileInput":
          inputFile = f;
          assignFileHash(f);
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
      switch (await getMd5(file)) {
        case "a722f8161ff489943191330bf8416496":
          ver = "us";
          romHashMessage = "Valid US ROM";
          requiredPlatform = "n64";
          selectedPlatform = "n64";
          break;
        case "df54f17fb84fb5b5bcf6aa9af65b0942":
          ver = "jp";
          romHashMessage = "Valid JP ROM";
          requiredPlatform = "n64";
          selectedPlatform = "n64";
          break;
        case "2aad94a7fa5f05c7544ddc0dd269c366":
          ver = "us";
          romHashMessage = "Valid US WAD";
          requiredPlatform = "wii";
          selectedPlatform = "wii";
          channelId = "FPUS";
          title = "fp-US";
          break;
        case "161563b6cf9ba5ca22306a729896f47d":
          ver = "jp";
          romHashMessage = "Valid JP WAD";
          requiredPlatform = "wii";
          selectedPlatform = "wii";
          channelId = "FPJP";
          title = "fp-JP";
          break;
        default:
          ver = "unk";
          romHashMessage = "Unknown base file";
          break;
      }
    } catch (error) {
      console.error(error);
    }
  };

  function handleVersionChange() {
    if (tag === "") {
      return;
    }
    switch (selectedPlatform) {
      case "n64":
        outFileName = `${tag}.z64`;
        break;
      case "wii":
        outFileName = `${tag}.wad`;
        break;
    }
  }

  onMount(() => {
    selectedPlatform = "n64";
    getTagList();
  });

  function readFileAsUint8Array(file: File): Promise<Uint8Array> {
    return new Promise<Uint8Array>((resolve, reject) => {
      const reader = new FileReader();

      reader.onload = (event) => {
        const arrayBuffer = event.target?.result as ArrayBuffer;
        const uint8Array = new Uint8Array(arrayBuffer);
        resolve(uint8Array);
      };

      reader.onerror = (event) => {
        reject(event.target?.error);
      };

      reader.readAsArrayBuffer(file);
    });
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

  function blockBuild() {
    if (ver === "unk" || ver == null || inputFile == null) {
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
      patch_file: Uint8Array
    ) {
      const input = await readFileAsUint8Array(inputFile);
      if (selectedPlatform === "n64") {
        return n64_decode(input, patch_file);
      } else if (selectedPlatform === "wii") {
        const memPatch = await getS3File(`gzi/mem_patch.gzi`);
        const settings = {
          wad: input,
          xdelta_patch: patch_file,
          gzi_patch: memPatch,
          channel_id: channelId,
          title: title,
        };
        return wii_inject(settings);
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
      <Label for="fileInput" class="pb-2">{romHashMessage}</Label>
      <Fileupload
        id="fileInput"
        class="mb-2"
        on:change={handleFileSelect}
        on:change={blockBuild}
      />
      <Helper>Z64 or WAD.</Helper>
    </div>

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
            disabled={requiredPlatform !== null && requiredPlatform !== "n64"}
            bind:group={selectedPlatform}>N64</Radio
          >
        </li>
        <li>
          <Radio
            name="platform"
            value="wii"
            disabled={requiredPlatform !== null && requiredPlatform !== "wii"}
            bind:group={selectedPlatform}>Wii</Radio
          >
        </li>
        <!--
        <li>
          <Radio
            name="platform"
            value="wiiu"
            disabled={requiredPlatform !== null && requiredPlatform !== "wiiu"}
            bind:group={selectedPlatform}>Wii U</Radio
          >
        </li>
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
