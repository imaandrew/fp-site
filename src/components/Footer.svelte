<script lang="ts">
  import { FooterLink, FooterLinkGroup } from "flowbite-svelte";

  import { S3_URL } from "$lib/constants";
</script>

<FooterLinkGroup
  class="mt-3 flex flex-wrap items-center text-sm text-gray-500 sm:mt-0 dark:text-gray-400"
>
  <FooterLink href="{S3_URL}/fp-practice-saves.zip">Practice Saves</FooterLink>
  <FooterLink href="https://fp-docs.starhaven.dev/" target="_blank"
    >Manual</FooterLink
  >
  <FooterLink href="https://github.com/imaandrew/fp-site" target="_blank"
    >Website GitHub</FooterLink
  >
  <FooterLink href="https://github.com/imaandrew/fp-web-patcher" target="_blank"
    >Patcher GitHub</FooterLink
  >
</FooterLinkGroup>
