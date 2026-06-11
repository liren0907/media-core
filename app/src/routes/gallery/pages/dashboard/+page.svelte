<script lang="ts">
  import { GallerySection, LiveComponent, Showcase, CompositionTree } from '$lib/components/features/gallery';
  import type { CompositionNode } from '$lib/components/features/gallery/CompositionTree.svelte';
  import { StatCard, Panel, StatusBadge, ProgressBar, TabBar } from '$lib/components/ui';
  import { SparklineBar } from '$lib/components/data';
  import { ThroughputChart } from '$lib/components/media';
  import DashboardOverview from '$lib/components/features/dashboard/DashboardOverview.svelte';
  import MonitorView from '$lib/components/features/dashboard/MonitorView.svelte';

  // What the / (Dashboard) route is actually made of, per the source. Anchor
  // hrefs jump to the per-component sections below (mirrored in the sidebar).
  const composition: CompositionNode[] = [
    { name: 'PageContent', href: '/gallery/pagecontent', note: 'page root wrapper' },
    { name: 'TabBar', href: '#tabbar', note: 'Overview / Monitor tabs' },
    {
      name: 'DashboardOverview',
      note: 'features/dashboard · Overview tab',
      children: [
        {
          name: 'StatCard',
          href: '#statcard',
          note: '× 4 — CPU / Memory / Disk / System',
          children: [
            { name: 'SparklineBar', href: '#sparklinebar', note: 'CPU history, via extra snippet' },
            { name: 'ProgressBar', href: '#progressbar', note: 'Memory + Disk, via extra snippet' },
          ],
        },
        {
          name: 'Panel',
          href: '#panel',
          note: '“Throughput”',
          children: [{ name: 'ThroughputChart', href: '#throughputchart', note: 'SVG area chart' }],
        },
        {
          name: 'Panel',
          href: '#panel',
          note: '“Active Streams”',
          children: [
            { name: 'StatusBadge', href: '#statusbadge', note: 'one per stream row' },
            { name: 'Icon', href: '/gallery/icons', note: 'row actions' },
          ],
        },
      ],
    },
    {
      name: 'MonitorView',
      note: 'features/dashboard · Monitor tab',
      children: [
        {
          name: 'MonitorStats',
          note: 'features/monitor',
          children: [
            { name: 'StatCard', href: '#statcard' },
            { name: 'ProgressBar', href: '#progressbar' },
            { name: 'SparklineBar', href: '#sparklinebar' },
          ],
        },
        {
          name: 'StreamStatusTable',
          note: 'features/monitor',
          children: [
            { name: 'Panel', href: '#panel' },
            { name: 'StatusBadge', href: '#statusbadge' },
          ],
        },
        {
          name: 'EventLog',
          note: 'features/monitor',
          children: [
            { name: 'Panel', href: '#panel' },
            { name: 'StatusBadge', href: '#statusbadge' },
          ],
        },
        {
          name: 'PipelineList',
          note: 'features/monitor',
          children: [
            { name: 'Panel', href: '#panel' },
            { name: 'StatusBadge', href: '#statusbadge' },
          ],
        },
        {
          name: 'Panel',
          href: '#panel',
          note: '“Throughput”',
          children: [{ name: 'ThroughputChart', href: '#throughputchart' }],
        },
      ],
    },
  ];

  // Static demo data so every section renders fully without a backend.
  let demoTab = $state('overview');
  const demoTabs = [
    { id: 'overview', label: 'Overview', icon: 'dashboard' },
    { id: 'monitor', label: 'Monitor', icon: 'monitor' },
  ];
  const demoCpuHistory = [40, 60, 45, 30, 70, 45, 55, 35];
  const demoStreams = [
    { id: 'cam-entrance', type: 'rtsp', status: 'active', latency: '42ms' },
    { id: 'cam-lobby', type: 'rtsp', status: 'connecting', latency: '--' },
    { id: 'vod-demo', type: 'hls', status: 'error', latency: '--' },
  ];

  const tabBarCode = `const tabs = [
  { id: 'overview', label: 'Overview', icon: 'dashboard' },
  { id: 'monitor', label: 'Monitor', icon: 'monitor' },
];

<TabBar {tabs} {activeTab} onchange={(id) => activeTab = id} />`;

  const statCardCode = `<StatCard label="CPU" icon="memory" iconColor="text-[#137fec]" value="42%" sub="8C">
  {#snippet extra()}
    <SparklineBar values={cpuHistory} />
  {/snippet}
</StatCard>`;

  const sparklineCode = `<SparklineBar values={cpuHistory} />`;

  const progressBarCode = `<ProgressBar percent={memoryPercent} color="bg-orange-500" />
<ProgressBar percent={diskPercent} color="bg-purple-500" />`;

  const panelCode = `<Panel title="Throughput" icon="show_chart">
  {#snippet actions()}
    <!-- legend / selects pinned to the header's right side -->
  {/snippet}
  <!-- body: ThroughputChart, or the streams table -->
</Panel>`;

  const throughputCode = `<ThroughputChart {throughputHistory} />`;

  const statusBadgeCode = `<StatusBadge status={stream.status} />`;
</script>

<svelte:head>
  <title>Dashboard · Gallery</title>
</svelte:head>

<div class="flex flex-col gap-6">
  <GallerySection
    id="composition"
    title="Composition"
    description="How the / (Dashboard) route is assembled. Blue chips jump to that component's section below (or to its gallery page); plain chips are feature- or data-layer pieces."
  >
    <CompositionTree nodes={composition} />
  </GallerySection>

  <GallerySection
    id="tabbar"
    title="TabBar"
    description="The route's only top-level control — switches between the Overview and Monitor feature components."
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
    id="statcard"
    title="StatCard"
    description="The stats strip — four cards (CPU / Memory / Disk / System), each filling the extra snippet with a sparkline, a progress bar, or plain text."
    link={{ href: '/gallery/statcard', label: 'Component page' }}
  >
    <Showcase code={statCardCode}>
      <div class="w-full grid grid-cols-2 lg:grid-cols-4 gap-3">
        <StatCard label="CPU" icon="memory" iconColor="text-[#137fec]" value="42%" sub="8C">
          {#snippet extra()}
            <SparklineBar values={demoCpuHistory} />
          {/snippet}
        </StatCard>
        <StatCard label="Memory" icon="developer_board" iconColor="text-orange-500" value="68%">
          {#snippet extra()}
            <ProgressBar percent={68} color="bg-orange-500" />
            <div class="mt-1.5 text-meta">10.9 / 16.0 GB</div>
          {/snippet}
        </StatCard>
        <StatCard label="Disk" icon="hard_drive" iconColor="text-purple-500" value="57%" sub="994 GB">
          {#snippet extra()}
            <ProgressBar percent={57} color="bg-purple-500" />
            <div class="mt-1.5 text-meta">210 MB/s write</div>
          {/snippet}
        </StatCard>
        <StatCard label="System" icon="schedule" iconColor="text-green-500" value="14:32:05">
          {#snippet extra()}
            <div class="mt-1.5 text-meta">Uptime 3d 4h 12m</div>
          {/snippet}
        </StatCard>
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="sparklinebar"
    title="SparklineBar"
    description="components/data — the CPU card's mini bar history. No standalone gallery page; it always rides inside a StatCard's extra snippet."
  >
    <Showcase code={sparklineCode}>
      <div class="w-56">
        <SparklineBar values={demoCpuHistory} />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="progressbar"
    title="ProgressBar"
    description="Fill indicator inside the Memory and Disk cards, tinted per metric."
    link={{ href: '/gallery/progressbar', label: 'Component page' }}
  >
    <Showcase code={progressBarCode}>
      <div class="w-64 flex flex-col gap-3">
        <div>
          <div class="text-meta mb-1">Memory · 68%</div>
          <ProgressBar percent={68} color="bg-orange-500" />
        </div>
        <div>
          <div class="text-meta mb-1">Disk · 57%</div>
          <ProgressBar percent={57} color="bg-purple-500" />
        </div>
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="panel"
    title="Panel"
    description="Used twice on this page — “Throughput” and “Active Streams”. Header takes an icon plus an actions snippet; the body is free-form."
    link={{ href: '/gallery/panel', label: 'Component page' }}
  >
    <Showcase code={panelCode}>
      <div class="w-full">
        <Panel title="Throughput" icon="show_chart">
          {#snippet actions()}
            <div class="flex items-center gap-3 text-[10px]">
              <span class="flex items-center gap-1 text-slate-500">
                <span class="size-1.5 rounded-full bg-[#137fec]"></span> Network
              </span>
              <span class="flex items-center gap-1 text-slate-500">
                <span class="size-1.5 rounded-full bg-slate-400"></span> FPS
              </span>
            </div>
          {/snippet}
          <div class="p-4 text-caption">Panel body — the dashboard drops ThroughputChart or the streams table in here.</div>
        </Panel>
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="throughputchart"
    title="ThroughputChart"
    description="components/media — the SVG area chart inside the Throughput panel. Renders a placeholder curve when no history is loaded."
  >
    <Showcase code={throughputCode}>
      <div class="w-full rounded-md border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27]">
        <ThroughputChart throughputHistory={null} gradientId="galleryDashboardGradient" />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="statusbadge"
    title="StatusBadge"
    description="One per row of the Active Streams table — active / connecting / error map to green / yellow / red."
    link={{ href: '/gallery/statusbadge', label: 'Component page' }}
  >
    <Showcase code={statusBadgeCode}>
      <div class="w-full">
        <Panel title="Active Streams" icon="list_alt">
          {#snippet actions()}
            <span class="text-[10px] px-1.5 py-0.5 rounded bg-[#137fec]/10 text-[#137fec] font-bold">1 / 3</span>
          {/snippet}
          <div class="overflow-x-auto">
            <table class="w-full text-left text-xs font-mono">
              <thead class="text-slate-500 border-b border-slate-100 dark:border-[#2a3441]">
                <tr>
                  <th class="px-3 py-1.5 font-medium w-32">STREAM ID</th>
                  <th class="px-3 py-1.5 font-medium">TYPE</th>
                  <th class="px-3 py-1.5 font-medium">STATUS</th>
                  <th class="px-3 py-1.5 font-medium">LATENCY</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-slate-100 dark:divide-[#2a3441]">
                {#each demoStreams as stream (stream.id)}
                  <tr>
                    <td class="px-3 py-1.5 text-slate-900 dark:text-white font-bold">{stream.id}</td>
                    <td class="px-3 py-1.5 text-muted capitalize">{stream.type}</td>
                    <td class="px-3 py-1.5"><StatusBadge status={stream.status} /></td>
                    <td class="px-3 py-1.5 text-muted">{stream.latency}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </Panel>
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="live"
    title="Assembled result"
    description="The two feature components rendered whole. Both self-fetch system metrics — real data inside the app, empty/zeroed states without a backend."
  >
    <LiveComponent name="DashboardOverview" note="no props · self-fetches metrics">
      <DashboardOverview />
    </LiveComponent>
    <LiveComponent name="MonitorView" note="no props · self-fetches + polls">
      <MonitorView />
    </LiveComponent>
  </GallerySection>
</div>
