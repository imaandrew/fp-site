<script lang="ts">
  import { Alert, GradientButton, Spinner } from "flowbite-svelte";
  import { CloseCircleSolid } from "flowbite-svelte-icons";
  import { getContext } from "svelte";
  import { slide } from "svelte/transition";

  import type { PatcherState } from "$lib/patcherState.svelte";

  const patcher = getContext<PatcherState>("patcher");
  let { buildFn }: { buildFn: () => void } = $props();
</script>

<div class="grid auto-cols-max grid-cols-5 items-center gap-4">
  <GradientButton
    color="greenToBlue"
    size="xl"
    class="col-span-3"
    disabled={patcher.disableButton}
    onclick={() => {
      buildFn();
    }}
  >
    {patcher.buttonText}
  </GradientButton>
  <Spinner
    class="mr-3 {patcher.building ? '' : 'invisible'}"
    size="4"
    color="green"
  />
</div>
{#if patcher.isAlertVisible}
  <Alert
    dismissable
    bind:alertStatus={patcher.isAlertVisible}
    transition={slide}
    color="red"
    class="mt-6"
  >
    {#snippet icon()}
      <CloseCircleSolid class="h-5 w-5" />
    {/snippet}
    {patcher.alertText}
  </Alert>
{/if}
