<script lang="ts">
  import { PageContent, TabBar } from '$lib/components/ui';
  import { MotionTab, SimilarityTab, CompareTab, InferenceTab, AnnotateTab } from '$lib/components/features/analysis';

  let activeTab = $state('motion');

  const tabs = [
    { id: 'motion', label: 'Motion', icon: 'motion_photos_on' },
    { id: 'similarity', label: 'Similarity', icon: 'group_work' },
    { id: 'compare', label: 'Compare', icon: 'compare' },
    { id: 'inference', label: 'Inference', icon: 'bar_chart' },
    { id: 'annotate', label: 'Annotate', icon: 'draw' },
  ];
</script>

<svelte:head>
  <title>Media Analysis</title>
</svelte:head>

<PageContent>
  <TabBar {tabs} {activeTab} onchange={(id) => activeTab = id} />
  {#if activeTab === 'motion'}
    <MotionTab />
  {:else if activeTab === 'similarity'}
    <SimilarityTab />
  {:else if activeTab === 'compare'}
    <CompareTab />
  {:else if activeTab === 'inference'}
    <InferenceTab />
  {:else}
    <AnnotateTab />
  {/if}
</PageContent>
