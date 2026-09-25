<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    headers: Snippet;
    children: Snippet;
    empty?: Snippet;
    isEmpty?: boolean;
  }

  let { headers, children, empty, isEmpty = false }: Props = $props();
</script>

<div class="overflow-x-auto">
  <table class="w-full text-left text-xs font-mono">
    <thead class="text-slate-500 border-b border-slate-100 dark:border-[#2a3441]">
      <tr>
        {@render headers()}
      </tr>
    </thead>
    <tbody class="divide-y divide-slate-100 dark:divide-[#2a3441]">
      {#if isEmpty && empty}
        <tr><td colspan="99" class="text-center py-6 text-slate-500">{@render empty()}</td></tr>
      {:else}
        {@render children()}
      {/if}
    </tbody>
  </table>
</div>
