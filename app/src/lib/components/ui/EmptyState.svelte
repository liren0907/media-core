<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';

  interface Props {
    icon: string;
    message: string;
    children?: Snippet;
  }

  let { icon, message, children }: Props = $props();
</script>

<div class="flex flex-col items-center py-12 text-slate-500">
  <Icon name={icon} class="text-3xl mb-2" />
  <p class="text-xs">{message}</p>
  {#if children}{@render children()}{/if}
</div>
