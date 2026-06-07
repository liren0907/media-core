<script lang="ts">
  import { GallerySection } from '$lib/components/features/gallery';

  const colorTokens = [
    { cls: 'text-status-info', label: 'text-status-info' },
    { cls: 'text-status-success', label: 'text-status-success' },
    { cls: 'text-status-warning', label: 'text-status-warning' },
    { cls: 'text-status-error', label: 'text-status-error' },
    { cls: 'text-muted', label: 'text-muted' },
  ];
</script>

<svelte:head>
  <title>Colors · Gallery</title>
</svelte:head>

<GallerySection id="colors" title="Colors" description="Color-only tokens (compose with role tokens) plus the brand blue.">
  <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3">
    <div class="flex flex-col items-center gap-2 p-4 rounded-lg border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27]">
      <div class="size-9 rounded-lg bg-[#137fec]"></div>
      <span class="text-meta text-center">#137fec</span>
      <span class="text-meta text-center text-slate-400">brand</span>
    </div>
    {#each colorTokens as c}
      <div class="flex flex-col items-center gap-2 p-4 rounded-lg border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27]">
        <span class="text-2xl font-bold {c.cls}">Aa</span>
        <span class="text-meta text-center break-all">{c.label}</span>
      </div>
    {/each}
  </div>
</GallerySection>
