<script lang="ts">
  import Breadcrumb from '$lib/components/ui/Breadcrumb.svelte';
  import { GallerySection, Showcase } from '$lib/components/features/gallery';
</script>

<svelte:head>
  <title>Breadcrumb · Gallery</title>
</svelte:head>

<GallerySection id="breadcrumb" title="Breadcrumb" description="Route-driven (no props). Derives segments from the current URL via nav.ts — shown here reflecting this page.">
  <Showcase code={'<Breadcrumb />'}>
    <Breadcrumb />
  </Showcase>
</GallerySection>
