<script lang="ts">
  import PageContent from '$lib/components/ui/PageContent.svelte';
  import TabBar from '$lib/components/ui/TabBar.svelte';
  import PhotoDemo from '$lib/components/features/demo360/PhotoDemo.svelte';
  import VideoDemo from '$lib/components/features/demo360/VideoDemo.svelte';

  const tabs = [
    { id: 'photo', label: 'Photo', icon: 'image' },
    { id: 'video', label: 'Video', icon: 'movie' },
  ];
  let activeTab = $state('photo');
</script>

<svelte:head>
  <title>360° Media Demo</title>
</svelte:head>

<PageContent>
  <div class="flex items-center justify-between gap-3 flex-wrap">
    <div class="flex items-center gap-3">
      <h1 class="text-page-title text-lg flex items-center gap-2">
        <span class="material-symbols-outlined text-status-info">panorama_photosphere</span>
        360° Media Demo
      </h1>
      <span class="px-2 py-0.5 rounded-full bg-amber-500/10 text-badge text-status-warning border border-amber-500/20 uppercase tracking-wider">Experimental</span>
    </div>
    <TabBar {tabs} {activeTab} onchange={(id) => (activeTab = id)} />
  </div>

  <div class="rounded border border-amber-200/60 dark:border-amber-500/20 bg-amber-50/60 dark:bg-amber-500/5 px-3 py-2 text-[11px] text-amber-800 dark:text-amber-300 leading-relaxed">
    Standalone sandbox for previewing equirectangular 360° photos and videos. This page is isolated from the rest of the app — no production routes or data are touched.
  </div>

  {#if activeTab === 'photo'}
    <PhotoDemo />
  {:else if activeTab === 'video'}
    <VideoDemo />
  {/if}
</PageContent>
