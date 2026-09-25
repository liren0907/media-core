<script lang="ts">
  import { ErrorAlert } from '$lib/components/ui';
  import { GallerySection, Showcase } from '$lib/components/features/gallery';
</script>

<svelte:head>
  <title>ErrorAlert · Gallery</title>
</svelte:head>

<GallerySection id="erroralert" title="ErrorAlert" description="Inline error message banner. Prop: message.">
  <Showcase code={'<ErrorAlert message="Something went wrong." />'}>
    <div class="w-full"><ErrorAlert message="Failed to load source: directory not found." /></div>
  </Showcase>
</GallerySection>
