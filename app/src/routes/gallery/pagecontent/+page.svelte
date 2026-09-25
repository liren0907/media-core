<script lang="ts">
  import { GallerySection, Showcase } from '$lib/components/features/gallery';
</script>

<svelte:head>
  <title>PageContent · Gallery</title>
</svelte:head>

<GallerySection id="pagecontent" title="PageContent" description="Page root wrapper — provides the standard padding, vertical gap between sections, and page background. Wrap every route's content in it.">
  <Showcase code={'<PageContent>\n  …page content…\n</PageContent>'}>
    <span class="text-caption">Layout primitive — not rendered live here to avoid nesting a scroll container inside the page. See any route's <span class="text-code">+page.svelte</span> for usage.</span>
  </Showcase>
</GallerySection>
