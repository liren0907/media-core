<script lang="ts">
  import { GallerySection } from '$lib/components/features/gallery';

  const colorTokens = [
    { cls: 'text-status-info', label: 'text-status-info' },
    { cls: 'text-status-success', label: 'text-status-success' },
    { cls: 'text-status-warning', label: 'text-status-warning' },
    { cls: 'text-status-error', label: 'text-status-error' },
    { cls: 'text-muted', label: 'text-muted' },
  ];

  // shadcn-svelte semantic pairs: each renders bg-{name} + text-{name}-foreground.
  // Class strings are literal so Tailwind picks them up.
  const shadcnPairs = [
    { name: 'background', cls: 'bg-background text-foreground' },
    { name: 'card', cls: 'bg-card text-card-foreground' },
    { name: 'popover', cls: 'bg-popover text-popover-foreground' },
    { name: 'primary', cls: 'bg-primary text-primary-foreground' },
    { name: 'secondary', cls: 'bg-secondary text-secondary-foreground' },
    { name: 'muted', cls: 'bg-muted text-muted-foreground' },
    { name: 'accent', cls: 'bg-accent text-accent-foreground' },
    { name: 'destructive', cls: 'bg-destructive text-destructive-foreground' },
  ];

  const shadcnSingles = [
    { name: 'border', cls: 'bg-border' },
    { name: 'input', cls: 'bg-input' },
    { name: 'ring', cls: 'bg-ring' },
  ];
</script>

<svelte:head>
  <title>Colors · Gallery</title>
</svelte:head>

<div class="flex flex-col gap-6">
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

  <GallerySection
    id="shadcn-colors"
    title="shadcn-svelte theme colors"
    description="Semantic CSS variables from app.css, mapped to utilities via @theme inline. Each tile renders bg-NAME with text-NAME-foreground; values flip automatically with dark mode. shadcn components consume these internally."
  >
    <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
      {#each shadcnPairs as p (p.name)}
        <div class="flex flex-col items-center gap-2 p-3 rounded-lg border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27]">
          <div class="flex items-center justify-center w-full h-12 rounded-md border border-slate-200 dark:border-[#2a3441] text-sm font-bold {p.cls}">Aa</div>
          <span class="text-meta text-center break-all">{p.name}</span>
        </div>
      {/each}
    </div>

    <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
      {#each shadcnSingles as s (s.name)}
        <div class="flex flex-col items-center gap-2 p-3 rounded-lg border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27]">
          <div class="w-full h-12 rounded-md border border-slate-200 dark:border-[#2a3441] {s.cls}"></div>
          <span class="text-meta text-center break-all">{s.name}</span>
        </div>
      {/each}
    </div>
  </GallerySection>
</div>
