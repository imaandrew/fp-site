<h1>FP injection site</h1>

<script lang="ts">
  import { onMount } from 'svelte';
  import { getTags } from './lib/github';
  import { getMd5 } from './lib/md5';
  import Select from 'svelte-select';
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

  const setActiveTab = (tabIndex: number) => {
    activeTab = tabIndex;
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

  const buildFp = () => {

  };

</script>

<div class="container">
  <div style="display: flex; align-items: center;">
    <label for="fileInput" style="text-align: left; margin-right: 3%;">ROM:</label>
    <button class="fileButton" on:click={() => fileInput.click()}>Open File</button>
    <label for="fileInput" style="text-align: left; margin-left: 3%;">{fileName || 'No file selected.'}</label>
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
    <Select loadOptions={getTags} --background="black" --list-background="black" --item-hover-color="black"/>
  </div>
  <div class="file-info">
    <div class="tabs">
      <div class="tab" class:selected={activeTab === 0} on:click={() => setActiveTab(0)}>N64</div>
      <div class="tab" class:selected={activeTab === 1} on:click={() => setActiveTab(1)}>Wii</div>
      <div class="tab" class:selected={activeTab === 2} on:click={() => setActiveTab(2)}>Wii U</div>
      <div class="tab" class:selected={activeTab === 3} on:click={() => setActiveTab(3)}>Switch</div>
    </div>
    <div class="asdfasdf">
      <label for="outfile">Output file:</label>
      <input type="text" id="outfile" name="outfile"><br><br>
      {#if activeTab === 0}
      {/if}
      {#if activeTab === 1}
      {/if}
      {#if activeTab === 2}
      {/if}
      {#if activeTab === 3}
      {/if}
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
    max-width: max-content;
  }

  .tabs {
    display: flex;
    align-items: flex-start;
    justify-content: space-around;
    width: 100%;
    padding-top: 0px;
  }

  .tab {
    padding: 10px;
    border: 1px solid #ccc;
    border-start-end-radius: 5px;
    cursor: pointer;
    user-select: none;
    width: 25%;
    align-self: first baseline;
  }

  .file-info {
    border: 1px solid #ccc;
    border-radius: 5px;
    width: 450px;
    height: 300px;
    flex-direction: column;
  }

  .fileButton {
    width: 30%;
  }

  .injectButton {
    background-color: seagreen
  }
  
  .asdfasdf {
    padding-top: 1%
  }
</style>