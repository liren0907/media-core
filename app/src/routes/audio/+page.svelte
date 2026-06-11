<script lang="ts">
  import { PageContent, TabBar } from '$lib/components/ui';
  import { RecorderTab, MetadataTab, TranscribeTab } from '$lib/components/features/audio';

  let activeTab = $state('recorder');

  const tabs = [
    { id: 'recorder', label: 'Recorder', icon: 'mic' },
    { id: 'transcribe', label: 'Transcribe', icon: 'graphic_eq' },
    { id: 'metadata', label: 'Metadata', icon: 'info' },
  ];
</script>

<svelte:head>
  <title>Audio Tools</title>
</svelte:head>

<PageContent>
  <TabBar {tabs} {activeTab} onchange={(id) => activeTab = id} />
  {#if activeTab === 'recorder'}
    <RecorderTab />
  {:else if activeTab === 'transcribe'}
    <TranscribeTab />
  {:else}
    <MetadataTab />
  {/if}
</PageContent>
