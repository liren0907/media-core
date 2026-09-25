<script lang="ts">
  import { EmptyState } from '$lib/components/ui';
  import { GallerySection, Showcase } from '$lib/components/features/gallery';
</script>

<svelte:head>
  <title>EmptyState · Gallery</title>
</svelte:head>

<GallerySection id="emptystate" title="EmptyState" description="Placeholder for empty content. Props: icon, message, children? (action slot).">
  <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
    <Showcase label="basic" code={'<EmptyState icon="inbox" message="Nothing here yet" />'}>
      <div class="w-full"><EmptyState icon="inbox" message="Nothing here yet" /></div>
    </Showcase>
    <Showcase label="with action" code={'<EmptyState icon="folder_open" message="No sources">\n  <button>Add Source</button>\n</EmptyState>'}>
      <div class="w-full">
        <EmptyState icon="folder_open" message="No sources added">
          <button class="mt-2 px-3 py-1 rounded bg-[#137fec] text-white text-button">Add Source</button>
        </EmptyState>
      </div>
    </Showcase>
  </div>
</GallerySection>
