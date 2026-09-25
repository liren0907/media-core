<script lang="ts">
  import * as Dialog from '$lib/components/shadcn/dialog';
  import { buttonVariants } from '$lib/components/shadcn/button';
  import { GallerySection, Showcase } from '$lib/components/features/gallery';
</script>

<svelte:head>
  <title>shadcn Dialog · Gallery</title>
</svelte:head>

<GallerySection
  id="shadcn-dialog"
  title="Dialog (shadcn-svelte)"
  description="Accessible modal via bits-ui — focus trap, Esc to close, scroll lock, and ARIA wiring out of the box. Composed from Dialog.Root / Trigger / Content / Header / Title / Description / Footer / Close."
>
  <Showcase label="modal" code={'<Dialog.Root> <Dialog.Trigger/> <Dialog.Content/> </Dialog.Root>'}>
    <Dialog.Root>
      <Dialog.Trigger class={buttonVariants({ variant: 'default' })}>
        Open dialog
      </Dialog.Trigger>
      <Dialog.Content>
        <Dialog.Header>
          <Dialog.Title>Delete scan source?</Dialog.Title>
          <Dialog.Description>
            This removes the scan source and all of its fingerprints. This action cannot be undone.
          </Dialog.Description>
        </Dialog.Header>
        <Dialog.Footer>
          <Dialog.Close class={buttonVariants({ variant: 'outline' })}>Cancel</Dialog.Close>
          <Dialog.Close class={buttonVariants({ variant: 'destructive' })}>Delete</Dialog.Close>
        </Dialog.Footer>
      </Dialog.Content>
    </Dialog.Root>
  </Showcase>
</GallerySection>
