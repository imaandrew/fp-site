<script lang="ts">
  import { onMount } from "svelte";
  import { getTags } from "./lib/github";
  import { getMd5 } from "./lib/md5";
  import Select from "svelte-select";
  import { n64_decode } from "fp-web-patcher";
  let fileName: string;
  let file: File;
  let fileInput: HTMLInputElement;
  let ver: string;
  let tag: { label: string };
  let romHashMessage = "";
  let outFileName;

  const handleFileSelect = async (event: Event) => {
    const target = event.target as HTMLInputElement;
    const f = target.files[0];
    if (f) {
      file = f;
      fileName = f.name;
      assignFileHash(f);
    }
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
        default:
          ver = "unk";
          romHashMessage = "Unknown ROM";
          break;
      }
    } catch (error) {
      console.error(error);
    }
  };

  let selectedOption: string;

  function handleInputChange(event: Event) {
    const target = event.target as HTMLInputElement;
    selectedOption = target.value;
  }

  function handleVersionChange() {
    outFileName = `${tag.label}.z64`;
  }

  onMount(() => {
    selectedOption = "n64";
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

  async function downloadFile(url: string): Promise<Uint8Array> {
    const response = await fetch(url);
    const arrayBuffer = await response.arrayBuffer();
    const uint8Array = new Uint8Array(arrayBuffer);
    return uint8Array;
  }

  function saveUint8ArrayToFile(uint8Array: Uint8Array, fileName: string) {
    const blob = new Blob([uint8Array], { type: "application/octet-stream" });
    const url = URL.createObjectURL(blob);

    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = fileName;

    document.body.appendChild(anchor);
    anchor.click();

    document.body.removeChild(anchor);
    URL.revokeObjectURL(url);
  }

  const buildFp = () => {
    const rom = downloadFile(
      `http://localhost:3000/get-patch/${tag.label}/${ver}`
    ).then(function (patch_file: Uint8Array) {
      return readFileAsUint8Array(file).then(function (bytes: Uint8Array) {
        return n64_decode(bytes, patch_file);
      });
    });

    rom.then(function (file: Uint8Array) {
      saveUint8ArrayToFile(file, outFileName);
    });
  };
</script>

<h1>cool site</h1>

<div class="container">
  <div style="display: flex; align-items: center;">
    <label for="fileInput" style="text-align: left; margin-right: 3%;"
      >ROM:</label
    >
    <button class="fileButton" on:click={() => fileInput.click()}
      >Open File</button
    >
    <label for="fileInput" style="text-align: left; margin-left: 3%;"
      >{fileName || "No file selected."}</label
    >
  </div>
  <p>{romHashMessage}</p>
  <input
    type="file"
    id="fileInput"
    bind:this={fileInput}
    on:change={handleFileSelect}
    style="display: none;"
  />

  <div style="display: flex; align-items: center;">
    <p style="text-align: left; margin-right: 3%;">Version:</p>
    <Select
      loadOptions={getTags}
      bind:value={tag}
      on:change={handleVersionChange}
      --background="black"
      --list-background="black"
      --item-hover-color="black"
    />
  </div>
  <div style="display: flex">
    <div class="radio-buttons">
      <label for="n64">
        <input
          type="radio"
          bind:group={selectedOption}
          on:change={handleInputChange}
          value="n64"
        />
        N64
      </label>

      <label for="wii">
        <input
          type="radio"
          bind:group={selectedOption}
          on:change={handleInputChange}
          value="wii"
        />
        Wii
      </label>

      <label for="wiiu">
        <input
          type="radio"
          bind:group={selectedOption}
          on:change={handleInputChange}
          value="wiiu"
        />
        Wii U
      </label>

      <label for="switch">
        <input
          type="radio"
          bind:group={selectedOption}
          on:change={handleInputChange}
          value="switch"
        />
        Switch
      </label>
    </div>
    <div class="platform-settings">
      <div style="display: flex">
        <label
          for="outfile"
          style="margin-left: 10%; margin-right: 5%; white-space: nowrap"
          >Output file:</label
        >
        <input
          type="text"
          style="height:30px; width:300px;"
          id="outfile"
          name="outfile"
          bind:value={outFileName}
        /><br /><br />
      </div>
      <div
        class="settings"
        style="height: {selectedOption === 'wii' ? '120px' : '0px'}"
      >
        <div
          style="display: flex; margin-left: 10%; margin-right: 5%; white-space: nowrap"
        >
          <label
            for="channel-title"
            style="white-space: nowrap; margin-right: 5%">Channel title:</label
          >
          <input
            type="text"
            style="height: 30px; width: 300px;"
            id="channel-title"
            name="channel-title"
          />
          <br /><br />
        </div>
        <div
          style="display: flex; margin-left: 10%; margin-right: 5%; white-space: nowrap"
        >
          <label for="channel-id" style="white-space: nowrap; margin-right: 5%"
            >Channel id:</label
          >
          <input
            type="text"
            style="height: 30px; width: 300px;"
            id="channel-id"
            name="channel-id"
          />
          <br /><br />
        </div>
      </div>
      <div
        class="settings"
        style="height: {selectedOption === 'wiiu' ? '120px' : '0px'}"
      >
        <p>Coming soon</p>
      </div>
      <div
        class="settings"
        style="height: {selectedOption === 'switch' ? '120px' : '0px'}"
      >
        <p>Coming soon</p>
      </div>
    </div>
  </div>
  <button class="injectButton" on:click={() => buildFp()}>Build</button>
</div>

<style>
  .container {
    flex-direction: column;
    column-gap: 20px;
    row-gap: 20px;
    justify-content: space-between;
    display: inline-flex;
    height: 30%;
    max-width: 500px;
  }

  .fileButton {
    width: 30%;
  }

  .radio-buttons {
    display: flex;
    flex-direction: column;
  }

  .radio-buttons label {
    display: flex;
    align-items: center;
    padding: 5%;
  }

  .injectButton {
    background-color: seagreen;
  }

  .platform-settings {
    flex-direction: column;
    row-gap: 15%;
    display: inline-flex;
  }

  .settings {
    flex-direction: column;
    row-gap: 10px;
    justify-content: space-between;
    display: inline-flex;
    transition: height 0.5s;
    overflow: hidden;
  }
</style>
