<script lang="ts">
  import { Card } from '$lib/components/ui';
  import { GallerySection, Showcase } from '$lib/components/features/gallery';
</script>

<svelte:head>
  <title>Card · Gallery</title>
</svelte:head>

<GallerySection id="card" title="Card" description="Generic surface container. Prop: padding = 'normal' | 'tight' | 'none'.">
  <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
    <Showcase label="normal" code={'<Card>…</Card>'}>
      <div class="w-full"><Card><span class="text-body">padding: normal</span></Card></div>
    </Showcase>
    <Showcase label="tight" code={'<Card padding="tight">…</Card>'}>
      <div class="w-full"><Card padding="tight"><span class="text-body">padding: tight</span></Card></div>
    </Showcase>
    <Showcase label="none" code={'<Card padding="none">…</Card>'}>
      <div class="w-full"><Card padding="none"><span class="text-body block p-2">padding: none</span></Card></div>
    </Showcase>
  </div>
</GallerySection>
