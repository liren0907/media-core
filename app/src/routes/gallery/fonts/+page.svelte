<script lang="ts">
  import { GallerySection } from '$lib/components/features/gallery';

  const fonts = [
    { cls: 'font-display', label: 'font-display · Space Grotesk', sample: 'The quick brown fox jumps over the lazy dog' },
    { cls: 'font-sans', label: 'font-sans · Noto Sans (body)', sample: 'The quick brown fox jumps over the lazy dog' },
    { cls: 'font-mono', label: 'font-mono · Fira Mono', sample: 'const fox = () => jump(0x1234);' },
  ];
</script>

<svelte:head>
  <title>Fonts · Gallery</title>
</svelte:head>

<GallerySection id="fonts" title="Fonts" description="Three families wired via CSS variables and Tailwind fontFamily.">
  <div class="flex flex-col gap-4">
    {#each fonts as f}
      <div class="flex flex-col gap-1">
        <span class="text-meta">{f.label}</span>
        <span class="{f.cls} text-lg text-slate-900 dark:text-white">{f.sample}</span>
      </div>
    {/each}
  </div>
</GallerySection>
