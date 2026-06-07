<script lang="ts">
  import { GallerySection, LiveComponent } from '$lib/components/features/gallery';
  import HardwareSettings from '$lib/components/features/settings/HardwareSettings.svelte';
  import PathSettingsForm from '$lib/components/features/settings/PathSettingsForm.svelte';
  import StreamSettingsForm from '$lib/components/features/settings/StreamSettingsForm.svelte';
  import ThemeSelector from '$lib/components/features/settings/ThemeSelector.svelte';
</script>

<svelte:head>
  <title>Settings · Gallery</title>
</svelte:head>

<GallerySection
  id="settings"
  title="Settings"
  description="Feature components composed by the /settings route. Heads-up: ThemeSelector and the *SettingsForm components read/write global app state — interacting with them here affects real settings."
>
  <LiveComponent name="ThemeSelector" note="no props · writes global theme"><ThemeSelector /></LiveComponent>
  <LiveComponent name="HardwareSettings" note="no props · self-fetches"><HardwareSettings /></LiveComponent>
  <LiveComponent name="PathSettingsForm" note="onsave? · writes appConfig on save"><PathSettingsForm onsave={() => {}} /></LiveComponent>
  <LiveComponent name="StreamSettingsForm" note="onsave? · writes appConfig on save"><StreamSettingsForm onsave={() => {}} /></LiveComponent>
</GallerySection>
