<script lang="ts">
  import { GallerySection } from '$lib/components/features/gallery';

  const typographyTokens = [
    { cls: 'text-page-title', sample: 'Page Title' },
    { cls: 'text-section-title', sample: 'Section Title' },
    { cls: 'text-card-title', sample: 'Card Title' },
    { cls: 'text-stat-label', sample: 'Stat Label' },
    { cls: 'text-stat-value', sample: '1,234' },
    { cls: 'text-body', sample: 'Body — the quick brown fox jumps over the lazy dog.' },
    { cls: 'text-caption', sample: 'Caption — supplementary detail.' },
    { cls: 'text-meta', sample: 'meta · 2026-06-07 · 12.3ms' },
    { cls: 'text-code', sample: 'const x = 42;' },
    { cls: 'text-button', sample: 'BUTTON' },
    { cls: 'text-nav', sample: 'Navigation' },
    { cls: 'text-badge', sample: 'BADGE' },
  ];
</script>

<svelte:head>
  <title>Typography · Gallery</title>
</svelte:head>

<GallerySection id="typography" title="Typography" description="Semantic role tokens from app.css. Apply the class; color & dark-mode are baked in.">
  <div class="flex flex-col divide-y divide-slate-100 dark:divide-[#2a3441]">
    {#each typographyTokens as t}
      <div class="flex items-baseline justify-between gap-6 py-2">
        <span class={t.cls}>{t.sample}</span>
        <span class="text-meta shrink-0">.{t.cls}</span>
      </div>
    {/each}
  </div>
</GallerySection>
