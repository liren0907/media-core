<script lang="ts">
  import { GallerySection, LiveComponent } from '$lib/components/features/gallery';
  import CaptureTab from '$lib/components/features/streams/CaptureTab.svelte';
  import LocalTab from '$lib/components/features/streams/LocalTab.svelte';
  import MultiViewTab from '$lib/components/features/streams/MultiViewTab.svelte';
  import PlayerTab from '$lib/components/features/streams/PlayerTab.svelte';
  import ActiveStreamsList from '$lib/components/features/stream/ActiveStreamsList.svelte';
  import StreamStatsStrip from '$lib/components/features/stream/StreamStatsStrip.svelte';
  import type { StreamStats } from '$lib/types';

  const streamStats: StreamStats = {
    activeCount: 2,
    totalCount: 3,
    avgLatencyMs: 48,
    totalBitrateKbps: 8200,
    streams: [
      { id: 's1', name: 'Front Door', status: 'active', streamType: 'rtsp', codec: 'H.264', resolution: '1920x1080', fps: 30, bitrateKbps: 4000, durationSeconds: 3600, latencyMs: 45 },
      { id: 's2', name: 'Lobby', status: 'idle', streamType: 'hls', codec: 'H.265', resolution: '1280x720', fps: 25, bitrateKbps: 2200, durationSeconds: 1800, latencyMs: 60 },
    ],
  };
</script>

<svelte:head>
  <title>Streams · Gallery</title>
</svelte:head>

<GallerySection
  id="streams"
  title="Streams"
  description="Feature components composed by the /streams route. The tabs self-invoke on user action; the presentational list & strip are driven by mock StreamStats."
>
  <LiveComponent name="CaptureTab" note="prop: streamStats">
    <CaptureTab {streamStats} />
  </LiveComponent>
  <LiveComponent name="LocalTab" note="no props">
    <LocalTab />
  </LiveComponent>
  <LiveComponent name="MultiViewTab" note="prop: streamStats">
    <MultiViewTab {streamStats} />
  </LiveComponent>
  <LiveComponent name="PlayerTab" note="no props · loads hls.js + polls">
    <PlayerTab />
  </LiveComponent>
  <LiveComponent name="ActiveStreamsList" note="stream/ · prop: streamStats">
    <ActiveStreamsList {streamStats} />
  </LiveComponent>
  <LiveComponent name="StreamStatsStrip" note="stream/ · streamStats + latencyPercent + healthStatus">
    <StreamStatsStrip {streamStats} latencyPercent={24} healthStatus="Healthy" />
  </LiveComponent>
</GallerySection>
