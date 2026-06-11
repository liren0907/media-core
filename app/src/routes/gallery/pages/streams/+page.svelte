<script lang="ts">
  import { GallerySection, LiveComponent, Showcase, CompositionTree } from '$lib/components/features/gallery';
  import type { CompositionNode } from '$lib/components/features/gallery/CompositionTree.svelte';
  import { Panel, StatCard, StatusBadge, FormField, ToggleSwitch, TabBar } from '$lib/components/ui';
  import { SparklineBar } from '$lib/components/data';
  import { VideoPlayer } from '$lib/components/media';
  import CaptureTab from '$lib/components/features/streams/CaptureTab.svelte';
  import LocalTab from '$lib/components/features/streams/LocalTab.svelte';
  import MultiViewTab from '$lib/components/features/streams/MultiViewTab.svelte';
  import PlayerTab from '$lib/components/features/streams/PlayerTab.svelte';
  import ActiveStreamsList from '$lib/components/features/stream/ActiveStreamsList.svelte';
  import StreamStatsStrip from '$lib/components/features/stream/StreamStatsStrip.svelte';
  import type { StreamStats } from '$lib/types';

  // What the /streams route is actually made of, per the source.
  const composition: CompositionNode[] = [
    { name: 'PageContent', href: '/gallery/pagecontent', note: 'page root wrapper' },
    { name: 'TabBar', href: '#tabbar', note: 'Capture / Player / Multi-View / Local tabs' },
    {
      name: 'CaptureTab',
      note: 'features/streams · Capture tab',
      children: [
        {
          name: 'StreamStatsStrip',
          note: 'features/stream',
          children: [
            { name: 'StatCard', href: '#statcard' },
            { name: 'ProgressBar', href: '/gallery/progressbar', note: 'latency' },
          ],
        },
        {
          name: 'ActiveStreamsList',
          note: 'features/stream',
          children: [
            { name: 'Panel', href: '#panel' },
            { name: 'StatusBadge', href: '#statusbadge' },
          ],
        },
        { name: 'Panel', href: '#panel', note: '“Configure Stream”' },
        { name: 'FormField', href: '#formfield', note: 'RTSP URLs / output dir / duration' },
        { name: 'ToggleSwitch', href: '#toggleswitch', note: 'Preview / Custom FPS' },
      ],
    },
    {
      name: 'PlayerTab',
      note: 'features/streams · Player tab',
      children: [
        { name: 'Panel', href: '#panel' },
        { name: 'StatCard', href: '#statcard', note: 'with StatusBadge in extra snippet' },
        { name: 'StatusBadge', href: '#statusbadge' },
        { name: 'FormField', href: '#formfield' },
        { name: 'ProgressBar', href: '/gallery/progressbar' },
        { name: 'ErrorAlert', href: '/gallery/erroralert' },
        { name: 'SparklineBar', href: '#sparklinebar', note: 'bandwidth history' },
      ],
    },
    {
      name: 'MultiViewTab',
      note: 'features/streams · Multi-View tab',
      children: [
        { name: 'Panel', href: '#panel' },
        { name: 'StatCard', href: '#statcard' },
        { name: 'StatusBadge', href: '#statusbadge' },
        { name: 'EmptyState', href: '/gallery/emptystate' },
      ],
    },
    {
      name: 'LocalTab',
      note: 'features/streams · Local tab',
      children: [
        { name: 'Panel', href: '#panel' },
        { name: 'StatCard', href: '#statcard' },
        { name: 'VideoPlayer', href: '#videoplayer', note: 'components/media' },
      ],
    },
  ];

  // Static demo state so every section renders fully without a backend.
  let demoTab = $state('capture');
  const demoTabs = [
    { id: 'capture', label: 'Capture', icon: 'videocam' },
    { id: 'player', label: 'Player', icon: 'play_circle' },
    { id: 'multiview', label: 'Multi-View', icon: 'grid_view' },
    { id: 'local', label: 'Local', icon: 'movie' },
  ];
  let demoRtspUrl = $state('rtsp://example.com/stream');
  let demoPreview = $state(true);
  let demoCustomFps = $state(false);
  const demoBandwidth = [35, 55, 40, 80, 65, 90, 70, 60, 85, 75, 95, 50];
  const inputClass = 'bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#2a3441] rounded-lg px-3 py-2 text-sm text-slate-900 dark:text-white focus:outline-none focus:border-[#137fec] focus:ring-1 focus:ring-[#137fec]';

  const tabBarCode = `const tabs = [
  { id: 'capture', label: 'Capture', icon: 'videocam' },
  { id: 'player', label: 'Player', icon: 'play_circle' },
  { id: 'multiview', label: 'Multi-View', icon: 'grid_view' },
  { id: 'local', label: 'Local', icon: 'movie' },
];

<TabBar {tabs} {activeTab} onchange={(id) => activeTab = id} />`;

  const panelCode = `<Panel title="Configure Stream" icon="settings">
  {#snippet actions()}
    <button disabled={isStreaming} onclick={startCapture} class="…">Start</button>
  {/snippet}
  <!-- form body -->
</Panel>`;

  const statCardCode = `<StatCard label="Stream" icon="cell_tower" iconColor="text-[#137fec]" value={hlsStatus.status.toUpperCase()}>
  {#snippet extra()}<StatusBadge status={hlsStatus.status === 'active' ? 'active' : 'idle'} />{/snippet}
</StatCard>`;

  const statusBadgeCode = `<StatusBadge status={stream.status} />`;

  const formFieldCode = `<FormField label="Main RTSP URL" id="mainRtspUrl">
  <input id="mainRtspUrl" type="text" bind:value={rtspConfig.rtsp_url} class="{inputClass} flex-1" placeholder="rtsp://example.com/stream" />
</FormField>`;

  const toggleCode = `<ToggleSwitch bind:checked={rtspConfig.show_preview} label="Preview" />
<ToggleSwitch bind:checked={rtspConfig.use_fps} label="Custom FPS" />`;

  const sparklineCode = `<SparklineBar values={bandwidthHistory.map(bw => (bw / Math.max(...bandwidthHistory, 1)) * 100)} height="h-16" />`;

  const videoPlayerCode = `<VideoPlayer src={videoSrc} placeholderIcon="movie" placeholderText="Select a video file to play" />`;

  // Mock data for the assembled feature components.
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

<div class="flex flex-col gap-6">
  <GallerySection
    id="composition"
    title="Composition"
    description="How the /streams route is assembled — four self-contained tabs. Blue chips jump to that component's section below (or to its gallery page); plain chips are feature-layer pieces."
  >
    <CompositionTree nodes={composition} />
  </GallerySection>

  <GallerySection
    id="tabbar"
    title="TabBar"
    description="Four tabs — Capture, Player, Multi-View, Local — each rendering its own feature component."
    link={{ href: '/gallery/tabbar', label: 'Component page' }}
  >
    <Showcase code={tabBarCode}>
      <div class="w-full">
        <TabBar tabs={demoTabs} activeTab={demoTab} onchange={(id) => (demoTab = id)} />
        <p class="text-meta mt-2">Active tab: {demoTab}</p>
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="panel"
    title="Panel"
    description="The workhorse container of every tab — CaptureTab's “Configure Stream” puts a Start button in the actions snippet."
    link={{ href: '/gallery/panel', label: 'Component page' }}
  >
    <Showcase code={panelCode}>
      <div class="w-full">
        <Panel title="Configure Stream" icon="settings">
          {#snippet actions()}
            <button class="flex items-center gap-1 px-2 py-1 bg-[#137fec] hover:bg-blue-600 text-white rounded text-[10px] font-bold transition-colors">Start</button>
          {/snippet}
          <div class="p-3 text-caption">Panel body — the stream configuration form lives here.</div>
        </Panel>
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="statcard"
    title="StatCard"
    description="PlayerTab's status strip nests a StatusBadge inside the extra snippet; StreamStatsStrip uses the same card for latency / bitrate."
    link={{ href: '/gallery/statcard', label: 'Component page' }}
  >
    <Showcase code={statCardCode}>
      <div class="w-full grid grid-cols-2 lg:grid-cols-4 gap-3">
        <StatCard label="Stream" icon="cell_tower" iconColor="text-[#137fec]" value="ACTIVE">
          {#snippet extra()}<StatusBadge status="active" />{/snippet}
        </StatCard>
        <StatCard label="Latency" icon="speed" iconColor="text-orange-500" value="48ms" sub="avg" />
        <StatCard label="Bitrate" icon="swap_vert" iconColor="text-purple-500" value="8.2" sub="Mbps" />
        <StatCard label="Streams" icon="videocam" iconColor="text-green-500" value="2 / 3" />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="statusbadge"
    title="StatusBadge"
    description="One per stream row in ActiveStreamsList and Multi-View — driven straight by the stream's status string."
    link={{ href: '/gallery/statusbadge', label: 'Component page' }}
  >
    <Showcase code={statusBadgeCode}>
      <StatusBadge status="active" />
      <StatusBadge status="idle" />
      <StatusBadge status="connecting" />
      <StatusBadge status="error" />
    </Showcase>
  </GallerySection>

  <GallerySection
    id="formfield"
    title="FormField"
    description="CaptureTab's configuration form — labelled inputs for RTSP URLs, output directory, and segment duration."
    link={{ href: '/gallery/formfield', label: 'Component page' }}
  >
    <Showcase code={formFieldCode}>
      <div class="w-full max-w-md">
        <FormField label="Main RTSP URL" id="demo-rtsp-url">
          <input id="demo-rtsp-url" type="text" bind:value={demoRtspUrl} class="{inputClass} w-full" placeholder="rtsp://example.com/stream" />
        </FormField>
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="toggleswitch"
    title="ToggleSwitch"
    description="Capture options — preview on/off and custom-FPS override."
    link={{ href: '/gallery/toggleswitch', label: 'Component page' }}
  >
    <Showcase code={toggleCode}>
      <ToggleSwitch bind:checked={demoPreview} label="Preview" />
      <ToggleSwitch bind:checked={demoCustomFps} label="Custom FPS" />
    </Showcase>
  </GallerySection>

  <GallerySection
    id="sparklinebar"
    title="SparklineBar"
    description="components/data — PlayerTab normalizes its bandwidth history to percentages and renders the taller h-16 variant."
  >
    <Showcase code={sparklineCode}>
      <div class="w-72">
        <SparklineBar values={demoBandwidth} height="h-16" />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="videoplayer"
    title="VideoPlayer"
    description="components/media — LocalTab's player. Without a src it renders its icon placeholder."
  >
    <Showcase code={videoPlayerCode}>
      <div class="w-full max-w-md">
        <VideoPlayer placeholderIcon="movie" placeholderText="Select a video file to play" />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="live"
    title="Assembled result"
    description="The feature components rendered whole. The tabs self-invoke on user action; the presentational list & strip are driven by mock StreamStats."
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
</div>
