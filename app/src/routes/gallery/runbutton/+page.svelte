<script lang="ts">
  import { RunButton } from '$lib/components/ui';
  import { GallerySection, Showcase } from '$lib/components/features/gallery';

  const noop = () => {};
</script>

<svelte:head>
  <title>RunButton · Gallery</title>
</svelte:head>

<GallerySection id="runbutton" title="RunButton" description="Primary action button with loading state. Props: loading, disabled?, label?, onclick.">
  <Showcase code={'<RunButton loading={false} label="Run" onclick={fn} />'}>
    <RunButton loading={false} label="Run" onclick={noop} />
    <RunButton loading={true} label="Running" onclick={noop} />
    <RunButton loading={false} disabled={true} label="Disabled" onclick={noop} />
    <RunButton loading={false} label="Fingerprint" onclick={noop} />
  </Showcase>
</GallerySection>
