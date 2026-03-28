<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { PageContent, TabBar } from '$lib/components/ui';
  import { CaptureTab, PlayerTab, MultiViewTab, LocalTab } from '$lib/components/features/streams';
  import type { StreamStats } from '$lib/types';

  let activeTab = $state('capture');
  let streamStats = $state<StreamStats | null>(null);

  async function fetchStreamStats() {
    try { streamStats = await invoke('get_stream_stats'); } catch (e) { console.error('Failed to fetch stream stats:', e); }
  }

  $effect(() => {
    fetchStreamStats();
    const interval = setInterval(fetchStreamStats, 3000);
    return () => clearInterval(interval);
  });

  const tabs = [
    { id: 'capture', label: 'Capture', icon: 'videocam' },
    { id: 'player', label: 'Player', icon: 'play_circle' },
    { id: 'multiview', label: 'Multi-View', icon: 'grid_view' },
    { id: 'local', label: 'Local', icon: 'movie' },
  ];
</script>

<svelte:head>
  <title>Streams</title>
</svelte:head>

<PageContent>
  <TabBar {tabs} {activeTab} onchange={(id) => activeTab = id} />
  {#if activeTab === 'capture'}
    <CaptureTab {streamStats} />
  {:else if activeTab === 'player'}
    <PlayerTab />
  {:else if activeTab === 'multiview'}
    <MultiViewTab {streamStats} />
  {:else}
    <LocalTab />
  {/if}
</PageContent>
