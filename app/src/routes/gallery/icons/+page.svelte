<script lang="ts">
  import { onDestroy } from 'svelte';
  import { icons } from '$lib/components/ui/icon-registry';
  import { Icon } from '$lib/components/ui';
  import { GallerySection } from '$lib/components/features/gallery';
  import { copyText } from '$lib/utils/clipboard';

  const names = Object.keys(icons).sort();

  let query = $state('');
  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    return q ? names.filter((n) => n.includes(q)) : names;
  });

  let copiedName = $state<string | null>(null);
  let timer: ReturnType<typeof setTimeout> | null = null;

  async function copy(name: string) {
    if (!(await copyText(`<Icon name="${name}" />`))) return;
    copiedName = name;
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => (copiedName = null), 1500);
  }

  onDestroy(() => {
    if (timer) clearTimeout(timer);
  });
</script>

<svelte:head>
  <title>Icons · Gallery</title>
</svelte:head>

<GallerySection
  id="icons"
  title="Icons"
  description="Every name registered in icon-registry.ts, rendered through the <Icon name> wrapper (Lucide underneath). Icons are em-sized — set font-size to scale. Click a tile to copy its usage."
>
  <!-- Search + count -->
  <div class="flex items-center gap-3">
    <div class="flex items-center gap-2 flex-1 max-w-xs rounded-md border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27] px-3 py-1.5 focus-within:border-[#137fec] transition-colors">
      <Icon name="search" class="text-[16px] text-slate-400 shrink-0" />
      <input
        bind:value={query}
        onkeydown={(e) => { if (e.key === 'Escape') query = ''; }}
        type="text"
        placeholder="Search icons…"
        class="w-full bg-transparent text-body outline-none placeholder:text-slate-400"
      />
      {#if query}
        <button onclick={() => (query = '')} aria-label="Clear search" class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 transition-colors">
          <Icon name="close" class="text-[14px]" />
        </button>
      {/if}
    </div>
    <span class="text-meta shrink-0">{filtered.length} / {names.length}</span>
  </div>

  {#if filtered.length > 0}
    <div class="grid grid-cols-[repeat(auto-fill,minmax(112px,1fr))] gap-2">
      {#each filtered as name (name)}
        {@const copied = copiedName === name}
        <button
          onclick={() => copy(name)}
          title={`Copy <Icon name="${name}" />`}
          class="flex flex-col items-center gap-2 rounded-lg border bg-white dark:bg-[#161e27] p-3 transition-colors {copied ? 'border-[#137fec]' : 'border-slate-200 dark:border-[#2a3441] hover:border-[#137fec]'}"
        >
          <Icon
            name={copied ? 'check' : name}
            class="text-2xl {copied ? 'text-status-success' : 'text-slate-700 dark:text-slate-200'}"
          />
          <span class="text-meta text-center break-all leading-tight {copied ? 'text-status-success' : ''}">
            {copied ? 'Copied!' : name}
          </span>
        </button>
      {/each}
    </div>
  {:else}
    <p class="text-caption">No icons match “{query}”.</p>
  {/if}
</GallerySection>
