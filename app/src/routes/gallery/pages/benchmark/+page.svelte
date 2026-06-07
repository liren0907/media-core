<script lang="ts">
  import { GallerySection } from '$lib/components/features/gallery';
</script>

<svelte:head>
  <title>Benchmark · Gallery</title>
</svelte:head>

<GallerySection
  id="benchmark"
  title="Benchmark"
  description="The /benchmark route has no dedicated feature components."
>
  <div class="text-body text-slate-500 dark:text-slate-400">
    This page is built directly from base UI primitives (Panel, StatCard, RunButton, TabBar, ProgressBar, …).
    See the <a href="/gallery/card" class="text-status-info hover:underline">Components</a> group in the sidebar for those.
  </div>
</GallerySection>
