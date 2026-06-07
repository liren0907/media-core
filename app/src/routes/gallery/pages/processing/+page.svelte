<script lang="ts">
  import { GallerySection, LiveComponent } from '$lib/components/features/gallery';
  import BatchTab from '$lib/components/features/processing/BatchTab.svelte';
  import ExtractTab from '$lib/components/features/processing/ExtractTab.svelte';
</script>

<svelte:head>
  <title>Processing · Gallery</title>
</svelte:head>

<GallerySection
  id="processing"
  title="Processing"
  description="Feature components composed by the /processing route. Both are self-contained (no props); BatchTab polls pipeline progress and subscribes to events on mount."
>
  <LiveComponent name="BatchTab" note="no props · polls + events"><BatchTab /></LiveComponent>
  <LiveComponent name="ExtractTab" note="no props"><ExtractTab /></LiveComponent>
</GallerySection>
