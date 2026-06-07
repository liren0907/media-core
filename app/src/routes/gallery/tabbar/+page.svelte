<script lang="ts">
  import { TabBar } from '$lib/components/ui';
  import { GallerySection, Showcase } from '$lib/components/features/gallery';

  let activeTab = $state('overview');
  const demoTabs = [
    { id: 'overview', label: 'Overview', icon: 'dashboard' },
    { id: 'details', label: 'Details', icon: 'list' },
    { id: 'settings', label: 'Settings', icon: 'settings' },
  ];
</script>

<svelte:head>
  <title>TabBar · Gallery</title>
</svelte:head>

<GallerySection id="tabbar" title="TabBar" description="Segmented tab switcher. Props: tabs, activeTab, onchange.">
  <Showcase code={'<TabBar {tabs} {activeTab} onchange={(id) => (activeTab = id)} />'}>
    <div class="w-full flex flex-col gap-3">
      <TabBar tabs={demoTabs} {activeTab} onchange={(id) => (activeTab = id)} />
      <span class="text-meta">activeTab = {activeTab}</span>
    </div>
  </Showcase>
</GallerySection>
