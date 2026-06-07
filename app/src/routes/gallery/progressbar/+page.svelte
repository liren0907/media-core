<script lang="ts">
  import { ProgressBar } from '$lib/components/ui';
  import { GallerySection, Showcase } from '$lib/components/features/gallery';
</script>

<svelte:head>
  <title>ProgressBar · Gallery</title>
</svelte:head>

<GallerySection id="progressbar" title="ProgressBar" description="Thin progress indicator. Props: percent, color?.">
  <Showcase code={'<ProgressBar percent={50} />\n<ProgressBar percent={80} color="bg-green-500" />'}>
    <div class="w-full flex flex-col gap-3">
      <ProgressBar percent={25} />
      <ProgressBar percent={50} />
      <ProgressBar percent={80} color="bg-green-500" />
      <ProgressBar percent={100} color="bg-amber-500" />
    </div>
  </Showcase>
</GallerySection>
