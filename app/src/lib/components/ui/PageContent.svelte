<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    children: Snippet;
  }

  let { children }: Props = $props();
</script>

<div class="flex-1 overflow-auto p-3 flex flex-col gap-3 bg-slate-50 dark:bg-[#0d1117] text-slate-800 dark:text-slate-200">
  {@render children()}
</div>
