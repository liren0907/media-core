<script lang="ts">
  import { PageContent, TabBar } from '$lib/components/ui';
  import { DashboardOverview, MonitorView } from '$lib/components/features/dashboard';

  let activeTab = $state('overview');

  const tabs = [
    { id: 'overview', label: 'Overview', icon: 'dashboard' },
    { id: 'monitor', label: 'Monitor', icon: 'monitor' },
  ];
</script>

<svelte:head>
  <title>{activeTab === 'overview' ? 'Dashboard' : 'Monitor'}</title>
</svelte:head>

<PageContent>
  <TabBar {tabs} {activeTab} onchange={(id) => activeTab = id} />
  {#if activeTab === 'overview'}
    <DashboardOverview />
  {:else}
    <MonitorView />
  {/if}
</PageContent>
