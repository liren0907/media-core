<script lang="ts">
  import { Button } from '$lib/components/shadcn/button';
  import { GallerySection, Showcase } from '$lib/components/features/gallery';
</script>

<svelte:head>
  <title>shadcn Button · Gallery</title>
</svelte:head>

<GallerySection
  id="shadcn-button"
  title="Button (shadcn-svelte)"
  description="bits-ui–backed button styled with tailwind-variants. The `default` variant uses the brand --primary (#137fec). This is owned source in src/lib/components/shadcn/button/."
>
  <div class="flex flex-col gap-6">
    <Showcase label="variants" code={'<Button variant="secondary">…</Button>'}>
      <div class="flex flex-wrap items-center gap-2">
        <Button>Default</Button>
        <Button variant="secondary">Secondary</Button>
        <Button variant="outline">Outline</Button>
        <Button variant="ghost">Ghost</Button>
        <Button variant="destructive">Destructive</Button>
        <Button variant="link">Link</Button>
      </div>
    </Showcase>

    <Showcase label="sizes" code={'<Button size="sm">…</Button>'}>
      <div class="flex flex-wrap items-center gap-2">
        <Button size="sm">Small</Button>
        <Button size="default">Default</Button>
        <Button size="lg">Large</Button>
      </div>
    </Showcase>

    <Showcase label="disabled" code={'<Button disabled>…</Button>'}>
      <Button disabled>Disabled</Button>
    </Showcase>
  </div>
</GallerySection>
