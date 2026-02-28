<script lang="ts">
  import { Checkbox, Input, Radio } from "flowbite-svelte";
  import { getContext } from "svelte";
  import { slide } from "svelte/transition";

  import type { PatcherState } from "$lib/patcherState.svelte";

  const patcher = getContext<PatcherState>("patcher");
</script>

<div class="platform-settings mb-6 grid gap-1">
  <div class="flex items-center py-5 whitespace-nowrap">
    <p class="mr-3 text-left">Output file:</p>
    <Input type="text" id="outfile" bind:value={patcher.outFileName} required />
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
        <Radio name="version" value="us" bind:group={patcher.ver}>US</Radio>
        <Radio name="version" value="jp" bind:group={patcher.ver}>JP</Radio>
      </div>
      <Checkbox bind:checked={patcher.enableDarkFilter}>Dark filter</Checkbox>
      <Checkbox bind:checked={patcher.enableWidescreen}>Widescreen</Checkbox>
    </div>
  {/if}
</div>
