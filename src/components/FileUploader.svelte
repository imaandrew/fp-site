<script lang="ts">
  import { getContext } from "svelte";
  import { Label, Fileupload, Helper } from "flowbite-svelte";
  import type { PatcherState } from "../lib/patcherState.svelte";
  import { getCrc } from "../lib/util";
  import { ROM_CRC } from "../lib/constants";

  const patcher: PatcherState = getContext("patcher");

  async function assignFileHash(file: File) {
    try {
      const crc = await getCrc(file);
      switch (crc) {
        case ROM_CRC.US_Z64:
        case ROM_CRC.US_V64:
        case ROM_CRC.US_N64:
          patcher.ver = "us";
          patcher.romHashMessage = "Valid US ROM";
          patcher.platform = "n64";
          break;
        case ROM_CRC.JP_Z64:
        case ROM_CRC.JP_V64:
        case ROM_CRC.JP_N64:
          patcher.ver = "jp";
          patcher.romHashMessage = "Valid JP ROM";
          patcher.platform = "n64";
          break;
        case ROM_CRC.US_WAD:
          patcher.ver = "us";
          patcher.romHashMessage = "Valid US WAD";
          patcher.platform = "wii";
          patcher.channelId = "FPUS";
          patcher.title = "fp-US";
          break;
        case ROM_CRC.JP_WAD:
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
          break;
      }
    }
  }
</script>

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
