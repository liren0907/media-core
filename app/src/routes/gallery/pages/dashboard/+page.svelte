<script lang="ts">
  import { GallerySection, LiveComponent } from '$lib/components/features/gallery';
  import DashboardOverview from '$lib/components/features/dashboard/DashboardOverview.svelte';
  import MonitorView from '$lib/components/features/dashboard/MonitorView.svelte';
</script>

<svelte:head>
  <title>Dashboard · Gallery</title>
</svelte:head>

<GallerySection
  id="dashboard"
  title="Dashboard"
  description="Feature components composed by the / (Dashboard) route. Both self-fetch system metrics — they show real data inside the app and empty/zeroed states without a backend."
>
  <LiveComponent name="DashboardOverview" note="no props · self-fetches metrics">
    <DashboardOverview />
  </LiveComponent>
  <LiveComponent name="MonitorView" note="no props · self-fetches + polls">
    <MonitorView />
  </LiveComponent>
</GallerySection>
