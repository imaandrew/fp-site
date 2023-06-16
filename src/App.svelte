<script lang="ts">
  import { onMount } from "svelte";
  import { getTags } from "./lib/github";
  import { getMd5 } from "./lib/md5";
  import Select from "svelte-select";
  let fileName: string;
  let fileInput: HTMLInputElement;
  let ver: Version;
  let romHashMessage = "";

  enum Version {
    Us,
    Jp,
    Unknown,
  }

  const handleFileSelect = async (event: any) => {
    const file = event.target.files[0];
    if (file) {
      fileName = file.name;
      assignFileHash(file);
    }
  };

  const assignFileHash = async (file: File) => {
    try {
      switch (await getMd5(file)) {
        case "a722f8161ff489943191330bf8416496":
          ver = Version.Us;
          romHashMessage = "Valid US ROM";
          break;
        case "df54f17fb84fb5b5bcf6aa9af65b0942":
          ver = Version.Jp;
          romHashMessage = "Valid JP ROM";
          break;
        default:
          ver = Version.Unknown;
          romHashMessage = "Unknown ROM";
          break;
      }
    } catch (error) {
      console.error(error);
    }
  };

  const buildFp = () => {};
</script>

<h1>FP injection site</h1>

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
      --background="black"
      --list-background="black"
      --item-hover-color="black"
    />
  </div>
  <div class="radio-buttons">
    <label for="n64">
      <input type="radio" id="n64" name="platform" value="N64" />
      N64
    </label>

    <label for="wii">
      <input type="radio" id="wii" name="platform" value="Wii" />
      Wii
    </label>

    <label for="wiiu">
      <input type="radio" id="wiiu" name="platform" value="Wii U" />
      Wii U
    </label>

    <label for="switch">
      <input type="radio" id="switch" name="platform" value="Switch" />
      Switch
    </label>
  </div>
  <div class="asdfasdf">
    <label for="outfile">Output file:</label>
    <input
      type="text"
      style="height:30px; width:300px; margin-left: 3%"
      id="outfile"
      name="outfile"
    /><br /><br />
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
    max-width: max-content;
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
  }

  .injectButton {
    background-color: seagreen;
  }

  .asdfasdf {
    padding-top: 5%;
  }
</style>
