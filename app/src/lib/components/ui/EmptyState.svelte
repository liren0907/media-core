<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    icon: string;
    message: string;
    children?: Snippet;
  }

  let { icon, message, children }: Props = $props();
</script>

<div class="flex flex-col items-center py-12 text-slate-500">
  <span class="material-symbols-outlined text-3xl mb-2">{icon}</span>
  <p class="text-xs">{message}</p>
  {#if children}{@render children()}{/if}
</div>
