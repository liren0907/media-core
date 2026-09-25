<script lang="ts">
  import { Panel, RunButton } from '$lib/components/ui';
  import { GallerySection, Showcase } from '$lib/components/features/gallery';

  const noop = () => {};
</script>

<svelte:head>
  <title>Panel · Gallery</title>
</svelte:head>

<GallerySection id="panel" title="Panel" description="Titled container with optional icon and an actions slot. Props: title, icon?, actions? (snippet).">
  <Showcase label="basic" code={'<Panel title="Configuration" icon="tune">\n  …content…\n</Panel>'}>
    <div class="w-full max-w-md">
      <Panel title="Configuration" icon="tune">
        <div class="p-3 text-body">Panel body content goes here.</div>
      </Panel>
    </div>
  </Showcase>
  <Showcase label="with actions slot" code={'<Panel title="Results" icon="speed">\n  {#snippet actions()}<RunButton … />{/snippet}\n  …\n</Panel>'}>
    <div class="w-full max-w-md">
      <Panel title="Results" icon="speed">
        {#snippet actions()}
          <RunButton loading={false} label="Run" onclick={noop} />
        {/snippet}
        <div class="p-3 text-body">Body with a header action.</div>
      </Panel>
    </div>
  </Showcase>
</GallerySection>
