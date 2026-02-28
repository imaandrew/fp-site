<script lang="ts">
  import { getContext } from "svelte";
  import type { PatcherState } from "../lib/patcherState.svelte";
  import { Modal } from "flowbite-svelte";

  const patcher: PatcherState = getContext("patcher");
</script>

<Modal
  title="Input File Formats"
  bind:open={patcher.clickOutsideModal}
  autoclose
  outsideclose
>
  <h3 class="font-semibold text-gray-900 dark:text-white">N64</h3>
  ROMs in any format (big-endian, little-endian, byte-swapped) are supported
  <h3 class="font-semibold text-gray-900 dark:text-white">Wii</h3>
  Must provide a WAD file. An N64 ROM doesn't need to be provided, the patcher will
  use the one in the WAD
  <h3 class="font-semibold text-gray-900 dark:text-white">Wii U</h3>
  Must provide the decrypted game data packed in either a ZIP or TAR file. The three
  folders, (code, content, meta) do not need to be in the root of the archive, but
  they must all be in the same folder
</Modal>
