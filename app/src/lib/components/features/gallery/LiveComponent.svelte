<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    /** Component name shown as the stage label. */
    name: string;
    /** Optional note, e.g. a prop summary or caveat. */
    note?: string;
    children: Snippet;
  }

  let { name, note, children }: Props = $props();
</script>

<div class="flex flex-col gap-2">
  <div class="flex items-baseline justify-between gap-3">
    <span class="text-stat-label">{name}</span>
    {#if note}<span class="text-meta text-right">{note}</span>{/if}
  </div>
  <!-- Recessed stage. Feature components often need live data/backend, so we wrap
       them in an error boundary: anything that throws on render degrades gracefully
       instead of taking down the whole catalog page. -->
  <div class="rounded-lg border border-slate-200 dark:border-[#2a3441] bg-slate-100/70 dark:bg-[#0d1117] p-4 overflow-x-auto">
    <svelte:boundary>
      {@render children()}
      {#snippet failed(error)}
        <div class="flex flex-col gap-1 text-meta text-status-warning">
          <div class="flex items-center gap-2">
            <span class="material-symbols-outlined text-[16px]">warning</span>
            <span>Couldn't render in isolation — needs live data / backend.</span>
          </div>
          <code class="text-code break-all opacity-80">{error instanceof Error ? error.message : String(error)}</code>
        </div>
      {/snippet}
    </svelte:boundary>
  </div>
</div>
