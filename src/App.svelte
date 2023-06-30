<script lang="ts">
  import { onMount } from "svelte";
  import { getTags } from "./lib/github";
  import { getMd5 } from "./lib/md5";
  import { n64_decode, wii_inject } from "../pkg/fp_web_patcher";
  import {
    Fileupload,
    Label,
    Helper,
    Select,
    DarkMode,
    Radio,
    Input,
    GradientButton,
  } from "flowbite-svelte";
  import { getS3File } from "./lib/get_s3_file";
  let inputFile: File;
  let ver: string;
  let tag;
  let romHashMessage = "";
  let outFileName: string;
  let requiredPlatform: string = null;
  let selectedPlatform: string;
  let tagList = null;

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
          break;
        case "df54f17fb84fb5b5bcf6aa9af65b0942":
          ver = "jp";
          romHashMessage = "Valid JP ROM";
          break;
        case "2aad94a7fa5f05c7544ddc0dd269c366":
          ver = "us";
          romHashMessage = "Valid US WAD";
          requiredPlatform = "wii";
          selectedPlatform = "wii";
          break;
        case "161563b6cf9ba5ca22306a729896f47d":
          ver = "jp";
          romHashMessage = "Valid JP WAD";
          requiredPlatform = "wii";
          selectedPlatform = "wii";
          break;
        default:
          ver = "unk";
          romHashMessage = "Unknown ROM";
          break;
      }
    } catch (error) {
      console.error(error);
    }
  };

  function handleVersionChange() {
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

  const buildFp = () => {
    const outFile = getS3File(`fp/${tag.label}/${ver}.xdelta`).then(
      async function (patch_file: Uint8Array) {
        const input = await readFileAsUint8Array(inputFile);
        if (selectedPlatform === "n64") {
          return n64_decode(input, patch_file);
        } else if (selectedPlatform === "wii") {
          const memPatch = await getS3File(`gzi/mem_patch.gzi`);
          return wii_inject(input, patch_file, memPatch);
        }
      }
    );

    outFile.then(function (file: Uint8Array) {
      saveUint8ArrayToFile(file, outFileName);
    });
  };
</script>

<DarkMode />

<div
  class="container dark:bg-gray-900 flex flex-col items-center justify-center mx-auto border border-sky-500 rounded max-w-md p-8"
>
  <h1 class="text-4xl pb-8 font-bold dark:text-white">fp patcher</h1>
  <div style="flex items-center">
    <Label for="with_helper" class="pb-2">Choose base file</Label>
    <Fileupload id="with_helper" class="mb-2" on:change={handleFileSelect} />
    <Helper>Z64 or WAD.</Helper>
  </div>
  <p>{romHashMessage}</p>

  <div class="flex items-center pb-5">
    <p class="text-left mr-3 dark:text-white">Version:</p>
    <Select
      class="mt-2"
      items={tagList !== null ? tagList : []}
      bind:value={tag}
      on:change={handleVersionChange}
    />
  </div>
  <div class="platform-settings grid gap-6 mb-6 md:grid-cols-2">
    <ul class="flex flex-col justify-center gap-4">
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
          disabled={requiredPlatform !== null && requiredPlatform !== "switch"}
          bind:group={selectedPlatform}>Switch</Radio
        >
      </li>
    </ul>
    <div>
      <div>
        <Label for="outfile" class="mb-2">Output file</Label>
        <Input type="text" id="outfile" bind:value={outFileName} required />
      </div>
      {#if selectedPlatform === "wii"}
        <div>
          <Label for="channel-title" class="mb-2">Channel title</Label>
          <Input type="text" id="channel-title" required />
        </div>
        <div>
          <Label for="channel-id" class="mb-2">Channel id</Label>
          <Input type="text" id="channel-id" required />
        </div>
      {/if}
    </div>
  </div>
  <GradientButton
    type="submit"
    color="greenToBlue"
    size="xl"
    on:click={() => buildFp()}>Build</GradientButton
  >
</div>
