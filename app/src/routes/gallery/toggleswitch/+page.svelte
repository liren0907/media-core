<script lang="ts">
  import { ToggleSwitch } from '$lib/components/ui';
  import { GallerySection, Showcase } from '$lib/components/features/gallery';

  let toggleOn = $state(true);
</script>

<svelte:head>
  <title>ToggleSwitch · Gallery</title>
</svelte:head>

<GallerySection id="toggleswitch" title="ToggleSwitch" description="Bindable on/off switch. Props: checked (bindable), label.">
  <Showcase code={'<ToggleSwitch bind:checked={value} label="Enable feature" />'}>
    <ToggleSwitch bind:checked={toggleOn} label="Enable feature" />
    <span class="text-meta">checked = {toggleOn}</span>
  </Showcase>
</GallerySection>
