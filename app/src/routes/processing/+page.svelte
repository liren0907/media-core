<script lang="ts">
  import { PageContent, TabBar } from '$lib/components/ui';
  import { BatchTab, ExtractTab } from '$lib/components/features/processing';

  let activeTab = $state('batch');

  const tabs = [
    { id: 'batch', label: 'Batch', icon: 'conversion_path' },
    { id: 'extract', label: 'Extract', icon: 'photo_library' },
  ];
</script>

<svelte:head>
  <title>Processing</title>
</svelte:head>

<PageContent>
  <TabBar {tabs} {activeTab} onchange={(id) => activeTab = id} />
  {#if activeTab === 'batch'}
    <BatchTab />
  {:else}
    <ExtractTab />
  {/if}
</PageContent>
