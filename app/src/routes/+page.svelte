<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { PageContent, StatCard, Panel, StatusBadge, ProgressBar } from '$lib/components/ui';

  // Dashboard Overview - Tactical Command Style Redesign

  let currentTime = $state(new Date());

  // System metrics state
  interface SystemMetrics {
    cpu: { usagePercent: number; coreCount: number; frequencyMhz: number | null };
    memory: { totalGb: number; usedGb: number; availableGb: number; usagePercent: number };
    disk: { totalGb: number; usedGb: number; availableGb: number; usagePercent: number; readSpeedMbps: number | null; writeSpeedMbps: number | null };
    uptimeSeconds: number;
  }

  interface StreamStatus {
    id: string;
    name: string;
    status: string;
    streamType: string;
    codec: string | null;
    resolution: string | null;
    fps: number | null;
    bitrateKbps: number | null;
    durationSeconds: number | null;
    latencyMs: number | null;
  }

  interface StreamStats {
    activeCount: number;
    totalCount: number;
    avgLatencyMs: number;
    totalBitrateKbps: number;
    streams: StreamStatus[];
  }

  interface ThroughputPoint {
    timestamp: number;
    networkMbps: number;
    fps: number;
    cpuPercent: number;
  }

  interface ThroughputHistory {
    points: ThroughputPoint[];
    periodSeconds: number;
  }

  let metrics = $state<SystemMetrics | null>(null);
  let streamStats = $state<StreamStats | null>(null);
  let throughputHistory = $state<ThroughputHistory | null>(null);
  let cpuHistory: number[] = $state([40, 60, 45, 30, 70, 45, 55, 35]);

  async function fetchMetrics() {
    try {
      metrics = await invoke('get_system_metrics');
      if (metrics) {
        cpuHistory = [...cpuHistory.slice(1), metrics.cpu.usagePercent];
      }
    } catch (e) {
      console.error('Failed to fetch system metrics:', e);
    }
  }

  async function fetchStreamStats() {
    try {
      streamStats = await invoke('get_stream_stats');
    } catch (e) {
      console.error('Failed to fetch stream stats:', e);
    }
  }

  async function fetchThroughputHistory() {
    try {
      throughputHistory = await invoke('get_throughput_history', { periodSeconds: 3600 });
    } catch (e) {
      console.error('Failed to fetch throughput history:', e);
    }
  }

  function formatUptime(seconds: number): string {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  }


  $effect(() => {
    const clockInterval = setInterval(() => {
      currentTime = new Date();
    }, 1000);

    fetchMetrics();
    fetchStreamStats();
    fetchThroughputHistory();

    const metricsInterval = setInterval(() => {
      fetchMetrics();
      fetchStreamStats();
    }, 2000);

    return () => {
      clearInterval(clockInterval);
      clearInterval(metricsInterval);
    };
  });

  // Reactive values with fallbacks
  let cpuPercent = $derived(metrics?.cpu.usagePercent ?? 0);
  let memoryUsedGb = $derived(metrics?.memory.usedGb ?? 0);
  let memoryPercent = $derived(metrics?.memory.usagePercent ?? 0);
  let diskPercent = $derived(metrics?.disk.usagePercent ?? 0);
  let diskTotalGb = $derived(metrics?.disk.totalGb ?? 0);
  let writeSpeed = $derived(metrics?.disk.writeSpeedMbps ?? 0);
</script>

<svelte:head>
	<title>Media Core Dashboard</title>
</svelte:head>

<PageContent>
    <!-- Stats Strip: 4 columns -->
    <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
        <StatCard label="CPU" icon="memory" iconColor="text-[#137fec]" value="{cpuPercent.toFixed(0)}%" sub={metrics ? `${metrics.cpu.coreCount}C` : ''}>
            {#snippet extra()}
                <div class="h-5 w-full flex items-end gap-0.5 mt-2">
                    {#each cpuHistory as value, i}
                        <div
                            class="w-full rounded-sm transition-all {i === cpuHistory.length - 1 ? 'bg-[#137fec]' : 'bg-[#137fec]/20'}"
                            style="height: {value}%"
                        ></div>
                    {/each}
                </div>
            {/snippet}
        </StatCard>

        <StatCard label="Memory" icon="developer_board" iconColor="text-orange-500" value="{memoryPercent.toFixed(0)}%">
            {#snippet extra()}
                <ProgressBar percent={memoryPercent} color="bg-orange-500" />
                <div class="mt-1.5 text-[10px] text-slate-500 font-mono">
                    {memoryUsedGb.toFixed(1)} / {metrics?.memory.totalGb?.toFixed(1) ?? '0'} GB
                </div>
            {/snippet}
        </StatCard>

        <StatCard label="Disk" icon="hard_drive" iconColor="text-purple-500" value="{diskPercent.toFixed(0)}%" sub="{diskTotalGb.toFixed(0)} GB">
            {#snippet extra()}
                <ProgressBar percent={diskPercent} color="bg-purple-500" />
                <div class="mt-1.5 text-[10px] text-slate-500 font-mono">
                    {writeSpeed > 0 ? `${writeSpeed.toFixed(0)} MB/s write` : 'Idle'}
                </div>
            {/snippet}
        </StatCard>

        <StatCard label="System" icon="schedule" iconColor="text-green-500" value={currentTime.toLocaleTimeString('en-GB', { hour12: false })}>
            {#snippet extra()}
                <div class="mt-1.5 text-[10px] text-slate-500 font-mono">
                    {#if metrics}
                        Uptime {formatUptime(metrics.uptimeSeconds)}
                    {:else}
                        --:--:--
                    {/if}
                </div>
            {/snippet}
        </StatCard>
    </div>

    <!-- Throughput Chart -->
    <Panel title="Throughput" icon="show_chart">
        {#snippet actions()}
            <div class="flex items-center gap-4">
                <div class="flex items-center gap-3 text-[10px]">
                    <span class="flex items-center gap-1 text-slate-500">
                        <span class="size-1.5 rounded-full bg-[#137fec]"></span> Network
                    </span>
                    <span class="flex items-center gap-1 text-slate-500">
                        <span class="size-1.5 rounded-full bg-slate-400"></span> FPS
                    </span>
                </div>
                <select class="bg-transparent text-[10px] font-bold uppercase text-slate-500 border-none outline-none cursor-pointer hover:text-slate-700 dark:hover:text-white focus:ring-0 p-0">
                    <option>1H</option>
                    <option>24H</option>
                </select>
            </div>
        {/snippet}
        <div class="p-4 relative w-full h-[180px]">
            <svg class="w-full h-full" preserveAspectRatio="none" viewBox="0 0 800 180">
                <defs>
                    <linearGradient id="chartGradient" x1="0" x2="0" y1="0" y2="1">
                        <stop offset="0%" stop-color="#137fec" stop-opacity="0.2"></stop>
                        <stop offset="100%" stop-color="#137fec" stop-opacity="0"></stop>
                    </linearGradient>
                </defs>
                <line stroke="#2a3441" stroke-dasharray="2 4" stroke-width="1" x1="0" x2="800" y1="145" y2="145" opacity="0.5"></line>
                <line stroke="#2a3441" stroke-dasharray="2 4" stroke-width="1" x1="0" x2="800" y1="100" y2="100" opacity="0.5"></line>
                <line stroke="#2a3441" stroke-dasharray="2 4" stroke-width="1" x1="0" x2="800" y1="55" y2="55" opacity="0.5"></line>
                {#if throughputHistory && throughputHistory.points.length > 0}
                    {@const points = throughputHistory.points}
                    {@const pathData = points.map((p, i) => {
                        const x = (i / (points.length - 1)) * 800;
                        const y = 160 - (p.networkMbps / 30 * 130);
                        return `${i === 0 ? 'M' : 'L'} ${x} ${y}`;
                    }).join(' ')}
                    <path d="{pathData} V 180 H 0 Z" fill="url(#chartGradient)"></path>
                    <path d="{pathData}" fill="none" stroke="#137fec" stroke-width="2"></path>
                {:else}
                    <path d="M0 140 C 100 125, 200 155, 300 105 S 500 55, 600 85 S 700 40, 800 55 V 180 H 0 Z" fill="url(#chartGradient)"></path>
                    <path d="M0 140 C 100 125, 200 155, 300 105 S 500 55, 600 85 S 700 40, 800 55" fill="none" stroke="#137fec" stroke-width="2"></path>
                {/if}
                <path d="M0 130 C 120 135, 250 115, 350 120 S 550 105, 650 100 S 750 115, 800 108" fill="none" opacity="0.5" stroke="#94a3b8" stroke-dasharray="5 5" stroke-width="2"></path>
            </svg>
            <div class="flex justify-between mt-1 text-[10px] text-slate-500 font-mono">
                <span>{new Date(Date.now() - 3600000).toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' })}</span>
                <span>{new Date(Date.now() - 2700000).toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' })}</span>
                <span>{new Date(Date.now() - 1800000).toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' })}</span>
                <span>{new Date(Date.now() - 900000).toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' })}</span>
                <span>{currentTime.toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' })}</span>
            </div>
        </div>
    </Panel>

    <!-- Active Streams Table -->
    <Panel title="Active Streams" icon="list_alt">
        {#snippet actions()}
            <div class="flex items-center gap-2">
                {#if streamStats}
                    <span class="text-[10px] px-1.5 py-0.5 rounded bg-[#137fec]/10 text-[#137fec] font-bold">{streamStats.activeCount} / {streamStats.totalCount}</span>
                {/if}
                <a href="/multi-stream-viewer" class="text-[10px] font-bold uppercase tracking-wider text-[#137fec] hover:text-blue-400">View All</a>
            </div>
        {/snippet}
        <div class="overflow-x-auto">
            <table class="w-full text-left text-xs font-mono">
                <thead class="text-slate-500 border-b border-slate-100 dark:border-[#2a3441]">
                    <tr>
                        <th class="px-3 py-1.5 font-medium w-32">STREAM ID</th>
                        <th class="px-3 py-1.5 font-medium">TYPE</th>
                        <th class="px-3 py-1.5 font-medium">STATUS</th>
                        <th class="px-3 py-1.5 font-medium">CODEC</th>
                        <th class="px-3 py-1.5 font-medium">LATENCY</th>
                        <th class="px-3 py-1.5 font-medium text-right w-12"></th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-slate-100 dark:divide-[#2a3441]">
                    {#if streamStats && streamStats.streams.length > 0}
                        {#each streamStats.streams as stream}
                            <tr class="hover:bg-slate-50 dark:hover:bg-[#1f2937]/50 transition-colors">
                                <td class="px-3 py-1.5 text-slate-900 dark:text-white font-bold">{stream.id}</td>
                                <td class="px-3 py-1.5 text-slate-600 dark:text-slate-400 capitalize">{stream.streamType}</td>
                                <td class="px-3 py-1.5">
                                    <StatusBadge status={stream.status} />
                                </td>
                                <td class="px-3 py-1.5 text-slate-600 dark:text-slate-400">{stream.codec ?? 'N/A'}</td>
                                <td class="px-3 py-1.5 text-slate-600 dark:text-slate-400">{stream.latencyMs ? `${stream.latencyMs}ms` : '--'}</td>
                                <td class="px-3 py-1.5 text-right">
                                    <button class="text-slate-400 hover:text-[#137fec] transition-colors">
                                        <span class="material-symbols-outlined text-[16px]">more_vert</span>
                                    </button>
                                </td>
                            </tr>
                        {/each}
                    {:else}
                        <tr>
                            <td colspan="6" class="px-3 py-4 text-center text-slate-500">
                                No active streams. <a href="/stream" class="text-[#137fec] hover:underline">Add a stream</a> to get started.
                            </td>
                        </tr>
                    {/if}
                </tbody>
            </table>
        </div>
    </Panel>
</PageContent>
